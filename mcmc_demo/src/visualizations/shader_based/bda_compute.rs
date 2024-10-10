use std::{ops::Deref, sync::Arc};

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use tokio::sync::{oneshot, watch};
use tracing::Instrument;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor,
    ComputePipelineDescriptor, RenderPipeline, RenderPipelineDescriptor,
};

use crate::{
    create_shader_module,
    gpu_task::{GpuTask, RepaintToken},
    helpers::async_last_task_processor::TaskSender,
    simulation::random_walk_metropolis_hastings::Rwmh,
    target_distributions::multimodal_gaussian::GaussianTargetDistr,
    visualizations::AlgoPainter,
};

use super::{
    diff_display::{get_approx_buffers, shader_bindings::RWMHCountInfo},
    fullscreen_quad,
    resolution_uniform::get_resolution_buffer,
    target_distr::{get_normaldistr_buffer, shader_bindings::ResolutionInfo, NormalDistribution},
    INITIAL_RENDER_SIZE,
};

create_shader_module!("binary_distance_approx.compute", compute_bindings);
create_shader_module!("binary_distance_approx.fragment", fragment_bindings);

pub fn get_compute_buffer_size(resolution: &[f32; 2]) -> u64 {
    (resolution[0] * resolution[1]) as u64 * size_of::<f32>() as u64
}

#[macro_export]
macro_rules! definition_location {
    () => {
        concat!("Defined at: ", file!(), ":", line!())
    };
}

pub(super) fn get_compute_output_buffer(
    device: &wgpu::Device,
    current_res: Option<&[f32; 2]>,
) -> wgpu::Buffer {
    let res = current_res.unwrap_or(&INITIAL_RENDER_SIZE);

    device.create_buffer(&BufferDescriptor {
        label: Some(definition_location!()),
        // todo: consider splitting definition, MIGHT improve perf.
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: get_compute_buffer_size(res),
    })
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default)]
pub struct BDAComputeDiff {}

// I dont need to read this properly on the CPU right now
type ComputeBufCpuRepr = Vec<f32>;

struct PipelineStateHolder {
    fragment_pipeline: RenderPipeline,
    fragment_group_0: fragment_bindings::BindGroup0,
    fragment_group_1: fragment_bindings::BindGroup1,
    compute_output_buffer: Buffer,
    resolution_buffer: Buffer,
    target_buffer: Buffer,
    gpu_tx: TaskSender<ComputeTask>,
    compute_tx: watch::Sender<Option<ComputeBufCpuRepr>>,
    compute_rx: watch::Receiver<Option<ComputeBufCpuRepr>>,
    last_approx_len: usize,
}

impl AlgoPainter for BDAComputeDiff {
    fn paint(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        algo: Arc<Rwmh>,
        target: &GaussianTargetDistr,
    ) {
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderCall {
                algo_state: algo.clone(),
                px_size: rect.size().into(),
                target_distr: target.gaussians.clone(),
            },
        ));
    }
}

impl BDAComputeDiff {
    pub fn init_pipeline(
        render_state: &RenderState,
        gpu_tx: TaskSender<ComputeTask>,
        ctx: egui::Context,
    ) {
        let device = &render_state.device;

        let webgpu_debug_name = Some(definition_location!());

        let fragment_layout = fragment_bindings::create_pipeline_layout(device);

        let fragment_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: fullscreen_quad::vertex_state(
                &fullscreen_quad::create_shader_module(device),
                &fullscreen_quad::fullscreen_quad_entry(),
            ),
            fragment: Some(fragment_bindings::fragment_state(
                &fragment_bindings::create_shader_module(device),
                &fragment_bindings::fs_main_entry([Some(render_state.target_format.into())]),
            )),
            label: webgpu_debug_name,
            layout: Some(&fragment_layout),
            depth_stencil: None,
            multiview: None,
            multisample: Default::default(),
            primitive: Default::default(),
            cache: None,
        });

        let resolution_buffer = get_resolution_buffer(device);

        let target_buffer = get_normaldistr_buffer(device, None);

        let compute_output_buffer = get_compute_output_buffer(device, None);

        let fragment_group_0 = fragment_bindings::BindGroup0::from_bindings(
            device,
            fragment_bindings::bind_groups::BindGroupLayout0 {
                resolution_info: resolution_buffer.as_entire_buffer_binding(),
            },
        );

        let fragment_group_1 = fragment_bindings::BindGroup1::from_bindings(
            device,
            fragment_bindings::BindGroupLayout1 {
                gauss_bases: target_buffer.as_entire_buffer_binding(),
                compute_output: compute_output_buffer.as_entire_buffer_binding(),
            },
        );

        // TODO: This is the actual issue here. I think. I wrote the async_last_task_processor to replace this, why is it still around?
        let (compute_tx, compute_rx) = watch::channel(None);

        let refresh_on_finished = {
            let mut compute_rx = compute_rx.clone();
            let repaint_token = RepaintToken::new(ctx);
            async move {
                while let Ok(_) = compute_rx.wait_for(|val| val.is_some()).await {
                    tracing::debug!("Requesting repaint after finish");
                    repaint_token.request_repaint();
                }
                tracing::debug!("Refresh loop canceled");
            }
        }
        .in_current_span();
        #[cfg(target_arch = "wasm32")]
        tokio::task::spawn_local(refresh_on_finished);
        #[cfg(not(target_arch = "wasm32"))]
        tokio::task::spawn(refresh_on_finished);

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our struct, we insert it into the
        // `callback_resources` type map, which is stored alongside the render pass.
        let None = render_state
            .renderer
            .write()
            .callback_resources
            .insert(PipelineStateHolder {
                fragment_group_0,
                fragment_group_1,
                fragment_pipeline,
                resolution_buffer,
                compute_output_buffer,
                target_buffer,
                gpu_tx,
                compute_rx,
                compute_tx,
                last_approx_len: 0,
            })
        else {
            unreachable!("pipeline already present?!")
        };
    }
}

struct RenderCall {
    px_size: [f32; 2],
    target_distr: Vec<NormalDistribution>,
    algo_state: Arc<Rwmh>,
}

impl CallbackTrait for RenderCall {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // doesn't hold the viewport size
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let &mut PipelineStateHolder {
            ref resolution_buffer,
            ref mut target_buffer,
            ref mut compute_output_buffer,
            ref mut fragment_group_1,
            ref gpu_tx,
            ref compute_tx,
            ref mut compute_rx,
            ref mut last_approx_len,
            ..
        } = callback_resources
            .get_mut()
            .expect("Should've been seeded.");
        let target = self.target_distr.as_slice();
        if target_buffer.size() as usize != size_of_val(target) {
            let normdistr_buffer = get_normaldistr_buffer(device, Some(target));
            *target_buffer = normdistr_buffer;
        }
        let approx_accepted = self.algo_state.history.as_slice();
        let curr_approx_len = approx_accepted.len();
        let approx_changed = curr_approx_len != *last_approx_len;
        *last_approx_len = curr_approx_len;
        let res_changed = compute_output_buffer.size() != get_compute_buffer_size(&self.px_size);
        if res_changed {
            *compute_output_buffer = get_compute_output_buffer(device, Some(&self.px_size));
            queue.write_buffer(
                resolution_buffer,
                0,
                bytemuck::cast_slice(&[ResolutionInfo {
                    resolution: self.px_size,
                    _pad: [0.0; 2],
                }]),
            );
        }
        if res_changed || approx_changed {
            // old value is now outdated.
            compute_tx.send(None).unwrap();
            let (tx, rx) = oneshot::channel::<ComputeBufCpuRepr>();
            match gpu_tx.send_update_blocking(crate::visualizations::BdaComputeTask {
                px_size: self.px_size,
                algo_state: self.algo_state.clone(),
                result_tx: Some(tx),
            }) {
                Ok(_) => {}
                Err(_err) => {
                    tracing::warn!("GpuTasks filled");
                }
            }

            wasm_thread::spawn({
                let compute_tx = compute_tx.clone();
                move || {
                    use rayon::prelude::*;
                    // TODO: ensure this doesn't copy when sending over the channel.
                    // Otherwise I will have to find an alternative.
                    let Ok(mut prob_buffer) = rx.blocking_recv() else {
                        tracing::debug!(
                            "Closing Maxnorm worker main-thread, as sender channel-end closed"
                        );
                        return;
                    };

                    let max = *prob_buffer
                        .par_iter()
                        .max_by(|lhs, rhs| lhs.total_cmp(rhs))
                        .unwrap_or(&1.0);
                    prob_buffer
                        .par_iter_mut()
                        .for_each(|unnorm_prob| *unnorm_prob /= max);
                    compute_tx
                        .send(Some(prob_buffer))
                        .map_err(|_err| "Channel Closed")
                        .unwrap();
                }
            });
        }
        if compute_rx
            .has_changed()
            .expect("channel should never be closed")
        {
            if let &Some(ref val) = compute_rx.borrow().deref() {
                if compute_output_buffer.size() != (val.as_slice().len() * 4) as u64 {
                    tracing::error!("TODO: Fix this mismatch. Might just go away when I assure that only the latest render continues");
                } else {
                    queue.write_buffer(
                        compute_output_buffer,
                        0,
                        bytemuck::cast_slice(val.as_slice()),
                    );
                }
            } else {
                tracing::debug!("Clearing Buffer because of empty watch channel");
                // clear the buffer, instead of leaving wrong values there.
                // There ought to be a better way....
                queue.write_buffer(
                    compute_output_buffer,
                    0,
                    &vec![0u8; compute_output_buffer.size() as usize],
                );
            }
        }

        queue.write_buffer(
            target_buffer,
            0,
            bytemuck::cast_slice(self.target_distr.as_slice()),
        );
        // TODO: only reassign of required.
        // If that actually speeds things up, I dunno.
        *fragment_group_1 = fragment_bindings::BindGroup1::from_bindings(
            device,
            fragment_bindings::BindGroupLayout1 {
                compute_output: compute_output_buffer.as_entire_buffer_binding(),
                gauss_bases: target_buffer.as_entire_buffer_binding(),
            },
        );
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        let &PipelineStateHolder {
            ref fragment_pipeline,
            ref fragment_group_0,
            ref fragment_group_1,
            ..
        } = callback_resources.get().unwrap();
        render_pass.set_pipeline(fragment_pipeline);
        fragment_group_0.set(render_pass);
        fragment_group_1.set(render_pass);
        render_pass.draw(0..fullscreen_quad::NUM_VERTICES, 0..1);
    }
}

#[cfg_attr(feature = "more_debug_impls", derive(Debug))]
pub struct ComputeTask {
    px_size: [f32; 2],
    algo_state: Arc<Rwmh>,
    result_tx: Option<oneshot::Sender<ComputeBufCpuRepr>>,
}

impl GpuTask for ComputeTask {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "BDA GPU Task", skip(device, queue))
    )]
    async fn run(&mut self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        tracing::info!("Starting");
        let webgpu_debug_name = Some(definition_location!());

        let device = device.as_ref();
        let queue = queue.as_ref();

        let compute_layout = compute_bindings::create_pipeline_layout(device);

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            module: &compute_bindings::create_shader_module(device),
            entry_point: "cs_main",
            compilation_options: Default::default(),
            label: webgpu_debug_name,
            layout: Some(&compute_layout),
            cache: None,
        });

        let mut compute_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(definition_location!()),
        });
        let mut compute_pass = compute_encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some(definition_location!()),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&compute_pipeline);

        let resolution_buffer = get_resolution_buffer(device);

        let approx_accepted = self.algo_state.history.as_slice();
        let (accept_buffer, info_buffer) = get_approx_buffers(device, Some(approx_accepted));

        let compute_output_buffer = get_compute_output_buffer(device, Some(&self.px_size));

        let compute_group_0 = compute_bindings::BindGroup0::from_bindings(
            device,
            compute_bindings::BindGroupLayout0 {
                resolution_info: resolution_buffer.as_entire_buffer_binding(),
            },
        );
        queue.write_buffer(
            &resolution_buffer,
            0,
            bytemuck::cast_slice(&[ResolutionInfo {
                resolution: self.px_size,
                _pad: [0.0; 2],
            }]),
        );
        queue.write_buffer(
            &accept_buffer,
            0,
            bytemuck::cast_slice(self.algo_state.history.as_slice()),
        );
        queue.write_buffer(
            &info_buffer,
            0,
            bytemuck::cast_slice(&[RWMHCountInfo {
                max_remain_count: self.algo_state.max_remain_count,
                total_point_count: self.algo_state.total_point_count,
            }]),
        );
        // TODO: we need to remove the compute from the render queue.
        // Unsure how to achieve this. I was hoping having a separate commandencoder would siffice, but evidently not.
        // AFAIK sharing buffers isnt gonna work with separate queues, as the way to get a queue is bundled with the device creation. Both are created from the adapter.
        // Aside that, its also annoying, since I either need to coordinate things, or I need to smuggle the new device and queue in here.
        let compute_group_1 = compute_bindings::BindGroup1::from_bindings(
            device,
            compute_bindings::BindGroupLayout1 {
                compute_output: compute_output_buffer.as_entire_buffer_binding(),
                accepted: accept_buffer.as_entire_buffer_binding(),
                count_info: info_buffer.as_entire_buffer_binding(),
            },
        );
        compute_group_0.set(&mut compute_pass);
        compute_group_1.set(&mut compute_pass);
        compute_pass.dispatch_workgroups(self.px_size[0] as u32, self.px_size[1] as u32, 1);
        // I dont understand precisely why, but this is required.
        // It must do some management in the drop impl.
        drop(compute_pass);
        let compute_buffer = compute_encoder.finish();
        queue.submit([compute_buffer]);
        // asyncify the callback from read_buffer
        // Doing this to allow for backpressure.
        let (buffer_tx, buffer_rx) = oneshot::channel();
        // alternative to DownloadBuffer
        // compute_output_buffer.slice(..).map_async(wgpu::MapMode::Read, callback)
        wgpu::util::DownloadBuffer::read_buffer(
            device,
            queue,
            &compute_output_buffer.slice(..),
            |val| {
                tracing::info!("executed");
                let val = val.unwrap();
                let val: &[f32] = bytemuck::cast_slice(&val);
                let val = val.to_vec();
                if let Err(_err) = buffer_tx.send(val) {
                    tracing::debug!("Failed send on closed channel");
                }
            },
        );
        tracing::info!("reached");
        // TODO this is blocking on native for the very first task, and IDK why.
        let result_buf = buffer_rx
            .await
            .expect("embedding ought to avoid drop of channel");
        tracing::info!("reached2");
        self.result_tx
            .take()
            .unwrap()
            .send(result_buf)
            .map_err(|_val| ())
            .unwrap();
        tracing::info!("Finishing");
    }
}
