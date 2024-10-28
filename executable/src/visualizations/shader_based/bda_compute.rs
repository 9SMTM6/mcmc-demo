//! BDA = binary distance approximation, my made up term for the kind of approximation I make based on the RWMH results.
//!
//! The diff at the end of the BDA(Compute)Diff is meant to signify that we umtilately look at the difference between the approximation of and the target probability density

use std::{ops::Deref, sync::Arc};

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use macros::{cfg_educe_debug, cfg_persistence_derive};
use tokio::sync::{oneshot, watch};
use tracing::Instrument;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor,
    ComputePipelineDescriptor, RenderPipeline, RenderPipelineDescriptor,
};

use crate::{
    cfg_sleep, create_shader_module,
    helpers::{task_spawn, GpuTask, RepaintToken, TaskDispatcher},
    simulation::random_walk_metropolis_hastings::Rwmh,
    target_distr,
    visualizations::AlgoPainter,
};

use super::{
    bda_immediate::{get_approx_buffers, shader_bindings::RWMHCountInfo},
    fullscreen_quad,
    resolution_uniform::get_resolution_buffer,
    target_distr::{get_normaldistr_buffer, NormalDistribution},
    INITIAL_RENDER_SIZE,
};

create_shader_module!("binary_distance_approx.compute", compute_bindings);
create_shader_module!("binary_distance_approx.fragment", fragment_bindings);

pub use compute_bindings::ResolutionInfo;

pub fn compute_buffer_size_in_bytes(resolution: &[f32; 2]) -> u64 {
    (resolution[0] * resolution[1]) as u64 * size_of::<f32>() as u64
}

#[macro_export]
macro_rules! definition_location {
    () => {
        concat!("Defined at: ", file!(), ":", line!())
    };
}

pub(super) fn create_compute_output_buffer(
    device: &wgpu::Device,
    current_res: Option<&[f32; 2]>,
) -> wgpu::Buffer {
    let res = current_res.unwrap_or(&INITIAL_RENDER_SIZE);

    device.create_buffer(&BufferDescriptor {
        label: Some(definition_location!()),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        mapped_at_creation: false,
        size: compute_buffer_size_in_bytes(res),
    })
}

#[cfg_persistence_derive]
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
    gpu_tx: TaskDispatcher<ComputeTask>,
    compute_results_tx: watch::Sender<Option<ComputeBufCpuRepr>>,
    compute_results_rx: watch::Receiver<Option<ComputeBufCpuRepr>>,
    prev_approx_len: usize,
}

impl AlgoPainter for BDAComputeDiff {
    fn paint(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        algo: Arc<Rwmh>,
        target: &target_distr::Gaussian,
    ) {
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderCall {
                algo_state: algo.clone(),
                px_res: rect.size().into(),
                target_distr: target.gaussians.clone(),
            },
        ));
    }
}

impl BDAComputeDiff {
    pub fn init_pipeline(
        render_state: &RenderState,
        gpu_tx: TaskDispatcher<ComputeTask>,
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

        let compute_output_buffer = create_compute_output_buffer(device, None);

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

        let (compute_results_tx, compute_results_rx) = watch::channel(None);

        let refresh_on_finished = {
            let compute_results_rx = compute_results_rx.clone();
            let repaint_token = RepaintToken::new(ctx);
            async move {
                let mut compute_results_rx = compute_results_rx;
                while compute_results_rx.changed().await.is_ok() {
                    tracing::info!("Requesting repaint after finish");
                    repaint_token.request_repaint();
                    cfg_sleep!().await;
                }
                tracing::error!("Refresh loop canceled");
            }
        }
        .in_current_span();

        task_spawn(refresh_on_finished);

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
                compute_results_rx,
                compute_results_tx,
                prev_approx_len: 0,
            })
        else {
            unreachable!("pipeline already present?!")
        };
    }
}

struct RenderCall {
    px_res: [f32; 2],
    target_distr: Vec<NormalDistribution>,
    algo_state: Arc<Rwmh>,
}

impl CallbackTrait for RenderCall {
    #[expect(
        clippy::cognitive_complexity,
        reason = "Honestly I cant seem to find a better way of structuring it without making the actual complexity worse IMO"
    )]
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
            ref compute_results_tx,
            ref mut compute_results_rx,
            ref mut prev_approx_len,
            ..
        } = callback_resources
            .get_mut()
            .expect("Should've been seeded.");
        let target = self.target_distr.as_slice();
        if target_buffer.size() as usize != size_of_val(target) {
            let normdistr_buffer = get_normaldistr_buffer(device, Some(target));
            *target_buffer = normdistr_buffer;
        }
        let accepted_approx = self.algo_state.history.as_slice();
        let curr_approx_len = accepted_approx.len();
        let approx_changed = curr_approx_len != *prev_approx_len;
        *prev_approx_len = curr_approx_len;
        let res_changed =
            compute_output_buffer.size() != compute_buffer_size_in_bytes(&self.px_res);
        if res_changed {
            *compute_output_buffer = create_compute_output_buffer(device, Some(&self.px_res));
            queue.write_buffer(
                resolution_buffer,
                0,
                bytemuck::cast_slice(&[ResolutionInfo {
                    resolution: self.px_res,
                    _pad: [0.0; 2],
                }]),
            );
        }
        if res_changed || approx_changed {
            let compute_results_tx = compute_results_tx.clone();
            // old value is now outdated.
            tracing::info!("resetting compute result");
            compute_results_tx.send(None).unwrap();
            let rx = dispatch_approximation_gpu(gpu_tx, self);
            wasm_thread::spawn(move || dispatch_maxnorm_rayon(rx, &compute_results_tx));
        }
        if compute_results_rx
            .has_changed()
            .expect("channel should never be closed")
        {
            compute_results_rx.mark_unchanged();
            if let &Some(ref val) = compute_results_rx.borrow().deref() {
                if compute_output_buffer.size() != (val.as_slice().len() * 4) as u64 {
                    tracing::error!("Resolution mismatch.");
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
        // TODO: only reassign if required.
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

fn dispatch_maxnorm_rayon(
    rx: oneshot::Receiver<ComputeBufCpuRepr>,
    compute_results_tx: &watch::Sender<Option<ComputeBufCpuRepr>>,
) {
    use rayon::prelude::*;
    // TODO: ensure this doesn't copy when sending over the channel.
    // Otherwise I will have to find an alternative.
    let Ok(mut prob_buffer) = rx.blocking_recv() else {
        tracing::info!("Closing Maxnorm worker main-thread, as sender channel-end closed");
        return;
    };
    tracing::info!("calculating maxnorm");
    let max = *prob_buffer
        .par_iter()
        .max_by(|lhs, rhs| lhs.total_cmp(rhs))
        .unwrap_or(&1.0);
    prob_buffer
        .par_iter_mut()
        .for_each(|unnorm_prob| *unnorm_prob /= max);
    compute_results_tx
        .send(Some(prob_buffer))
        .map_err(|_err| "Channel Closed")
        .unwrap();
}

fn dispatch_approximation_gpu(
    gpu_tx: &TaskDispatcher<ComputeTask>,
    render_call: &RenderCall,
) -> oneshot::Receiver<Vec<f32>> {
    let (tx, rx) = oneshot::channel::<ComputeBufCpuRepr>();
    match gpu_tx.dispatch_task_blocking(crate::visualizations::BdaComputeTask {
        px_size: render_call.px_res,
        algo_state: render_call.algo_state.clone(),
        result_tx: Some(tx),
    }) {
        Ok(_) => {}
        Err(_err) => {
            tracing::warn!("GpuTasks filled");
        }
    }
    rx
}

#[cfg_educe_debug]
pub struct ComputeTask {
    px_size: [f32; 2],
    algo_state: Arc<Rwmh>,
    result_tx: Option<oneshot::Sender<ComputeBufCpuRepr>>,
}

impl GpuTask for ComputeTask {
    #[cfg_attr(
        all(feature = "tracing", not(rust_analyzer)),
        tracing::instrument(name = "BDA GPU Task", skip(device_arc, queue))
    )]
    async fn run(&mut self, device_arc: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        tracing::info!("Starting");
        let webgpu_debug_name = Some(definition_location!());

        let device = device_arc.as_ref();
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

        let compute_output_buffer = create_compute_output_buffer(device, Some(&self.px_size));

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
        // The drop impl is in [`wgpu::ComputePassInner`].
        drop(compute_pass);
        let compute_buffer = compute_encoder.finish();
        queue.submit([compute_buffer]);
        // asyncify the callback from read_buffer
        // Doing this to allow for backpressure and to abort when a newer callback was already started
        let (buffer_tx, buffer_rx) = oneshot::channel();
        // Debugging lack of callback getting called:
        // This does also use queue.submit, however its used before an BufferSlice::map_async.
        // If one reads BufferSlice::map_async, then it requires either an queue.submit, an instance.poll or an device.poll
        // afterwards, to complete the callback:

        // > For the callback to complete, either queue.submit(..), instance.poll_all(..), or device.poll(..) must be called elsewhere in the runtime, possibly integrated into an event loop or run on a separate thread.
        // > The callback will be called on the thread that first calls the above functions after the gpu work has completed. There are no restrictions on the code you can run in the callback, however on native the call to the function will not complete until the callback returns, so prefer keeping callbacks short and used to set flags, send messages, etc.

        // I'm not ENTIRELY sure what 'into an event loop or run on a separate thread.' means, does that mean I cant do that on this very thread that called map_async?
        // Regarding 'The callback will be called on the thread that first calls the above functions after the gpu work has completed', this, to me, has a different connotation to the original ask.
        // I get the impression that these functions might be required to be called after the GPU is finished, not at any time.
        // Re: 'however on native the call to the function will not complete until the callback returns' makes sense, so on native that call would ensure backpressure. But not on the web? That might be annoying...
        // On the web I might otherwise get a deadlock when using a channel to wait here, while also calling poll here.
        // So perhaps I have to use some global variable... But thats also annoying, as that might lead to race conditions, if I'm not careful
        // (should not with how I intend to do things right now, but still).
        //
        // Also note that I think I need to submit the compute queue first (as-is), otherwise the encoder.copy_buffer_to_buffer will be in the queue before the compute.
        wgpu::util::DownloadBuffer::read_buffer(device, queue, &compute_output_buffer.slice(..), {
            let current_span = tracing::Span::current();
            move |val| {
                let _guard = current_span.enter();
                tracing::info!("Executing callback");
                let val = val.unwrap();
                let val: &[f32] = bytemuck::cast_slice(&val);
                let val = val.to_vec();
                if let Err(_err) = buffer_tx.send(val) {
                    tracing::info!("Failed send on closed channel");
                }
            }
        });
        // run this GPU task.
        #[cfg(not(target_arch = "wasm32"))]
        tokio::task::spawn_blocking(move || {
            // TODO: this will do for now, in the future it might be problematic.
            // Potential alternatives I considered:
            // 1. Patching (and upstreaming) changes to DownloadBuffer::read_buffer to return SubmitIndex from the copy op subm_idx. then doing wgpu::Maintain::WaitForSubmissionIndex(subm_idx),
            // 2. calling poll in an event loop somewhere else, however I don't want that to end in busy waiting, so perhaps add a sleep inbetween, that however increases lag...
            // Also see <github link TODO>
            device_arc.poll(wgpu::Maintain::Wait);
            // device_arc.poll(wgpu::Maintain::WaitForSubmissionIndex(subm_idx));
        });

        let result_buf = buffer_rx
            .await
            .expect("embedding ought to avoid drop of channel");
        self.result_tx
            .take()
            .unwrap()
            .send(result_buf)
            .map_err(|_val| ())
            .unwrap();
        tracing::info!("Finishing");
    }
}
