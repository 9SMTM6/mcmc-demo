use eframe::egui_wgpu::{CallbackTrait, RenderState};
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, RenderPipeline, RenderPipelineDescriptor,
};

use crate::{
    create_shader_module, simulation::random_walk_metropolis_hastings::Rwmh,
    target_distributions::multimodal_gaussian::MultiModalGaussian,
};

use super::{
    diff_display::{get_approx_buffers, shader_bindings::RWMHCountInfo},
    fullscreen_quad,
    multimodal_gaussian::{
        get_normaldistr_buffer, shader_bindings::ResolutionInfo, NormalDistribution,
    },
    resolution_uniform::get_resolution_buffer,
    INITIAL_RENDER_SIZE,
};

create_shader_module!("binary_distance_approx.compute", compute_bindings);
create_shader_module!("binary_distance_approx.fragment", fragment_bindings);

pub fn get_compute_buffer_size(resolution: &[f32; 2]) -> u64 {
    (resolution[0] * resolution[1]) as u64 * size_of::<f32>() as u64
}

pub(super) fn get_compute_output_buffer(
    device: &wgpu::Device,
    current_res: Option<&[f32; 2]>,
) -> wgpu::Buffer {
    let res = match current_res {
        None => &INITIAL_RENDER_SIZE,
        Some(res) => res,
    };

    device.create_buffer(&BufferDescriptor {
        label: Some(file!()),
        usage: BufferUsages::STORAGE,
        mapped_at_creation: false,
        size: get_compute_buffer_size(res),
    })
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default)]
pub struct BDAComputeDiffDisplay {}

struct PipelineStateHolder {
    compute_pipeline: ComputePipeline,
    fragment_pipeline: RenderPipeline,
    compute_group_0: compute_bindings::BindGroup0,
    compute_group_1: compute_bindings::BindGroup1,
    fragment_group_0: fragment_bindings::BindGroup0,
    fragment_group_1: fragment_bindings::BindGroup1,
    compute_output_buffer: Buffer,
    resolution_buffer: Buffer,
    target_buffer: Buffer,
    approx_accepted_buffer: Buffer,
    approx_info_buffer: Buffer,
}

impl BDAComputeDiffDisplay {
    pub fn paint(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        algo: &Rwmh,
        target: &MultiModalGaussian,
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

    pub fn init_pipeline(render_state: &RenderState) {
        let device = &render_state.device;

        let webgpu_debug_name = Some(file!());

        let compute_layout = compute_bindings::create_pipeline_layout(device);
        let fragment_layout = fragment_bindings::create_pipeline_layout(device);

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            module: &compute_bindings::create_shader_module(device),
            entry_point: "cs_main",
            compilation_options: Default::default(),
            label: webgpu_debug_name,
            layout: Some(&compute_layout),
            cache: None,
        });

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

        let (approx_accepted_buffer, approx_info_buffer) = get_approx_buffers(device, None);

        let compute_output_buffer = get_compute_output_buffer(device, None);

        let compute_group_0 = compute_bindings::BindGroup0::from_bindings(
            device,
            compute_bindings::BindGroupEntries0 {
                resolution_info: resolution_buffer.as_entire_buffer_binding(),
            },
        );

        let compute_group_1 = compute_bindings::bind_groups::BindGroup1::from_bindings(
            device,
            compute_bindings::BindGroupEntries1 {
                accepted: approx_accepted_buffer.as_entire_buffer_binding(),
                count_info: approx_info_buffer.as_entire_buffer_binding(),
                compute_output: compute_output_buffer.as_entire_buffer_binding(),
            },
        );

        let fragment_group_0 = fragment_bindings::BindGroup0::from_bindings(
            device,
            fragment_bindings::bind_groups::BindGroupEntries0 {
                resolution_info: resolution_buffer.as_entire_buffer_binding(),
            },
        );

        let fragment_group_1 = fragment_bindings::BindGroup1::from_bindings(
            device,
            fragment_bindings::BindGroupEntries1 {
                gauss_bases: target_buffer.as_entire_buffer_binding(),
                compute_output: compute_output_buffer.as_entire_buffer_binding(),
            },
        );

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our struct, we insert it into the
        // `callback_resources` type map, which is stored alongside the render pass.
        let None = render_state
            .renderer
            .write()
            .callback_resources
            .insert(PipelineStateHolder {
                compute_group_0,
                compute_group_1,
                fragment_group_0,
                fragment_group_1,
                fragment_pipeline,
                compute_pipeline,
                resolution_buffer,
                compute_output_buffer,
                target_buffer,
                approx_accepted_buffer,
                approx_info_buffer,
            })
        else {
            unreachable!("pipeline already present?!")
        };
    }
}

struct RenderCall {
    px_size: [f32; 2],
    target_distr: Vec<NormalDistribution>,
    algo_state: Rwmh,
}

impl CallbackTrait for RenderCall {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // doesn't hold the viewport size
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let &mut PipelineStateHolder {
            ref compute_pipeline,
            ref resolution_buffer,
            ref mut target_buffer,
            ref mut approx_accepted_buffer,
            ref mut approx_info_buffer,
            ref mut compute_output_buffer,
            ref compute_group_0,
            ref mut compute_group_1,
            ref mut fragment_group_1,
            ..
        } = callback_resources.get_mut().unwrap();
        let target = self.target_distr.as_slice();
        if target_buffer.size() as usize != size_of_val(target) {
            let normdistr_buffer = get_normaldistr_buffer(device, Some(target));
            *target_buffer = normdistr_buffer;
        }
        let approx_accepted = self.algo_state.history.as_slice();
        let approx_changed = approx_accepted_buffer.size() as usize != size_of_val(approx_accepted);
        if approx_changed {
            let (accept_buffer, info_buffer) = get_approx_buffers(device, Some(approx_accepted));
            *approx_accepted_buffer = accept_buffer;
            *approx_info_buffer = info_buffer;
            // these will change in size if the underlying data changes,
            // so we only write the buffers in that case.
            queue.write_buffer(
                approx_accepted_buffer,
                0,
                bytemuck::cast_slice(self.algo_state.history.as_slice()),
            );
            queue.write_buffer(
                approx_info_buffer,
                0,
                bytemuck::cast_slice(&[RWMHCountInfo {
                    max_remain_count: self.algo_state.max_remain_count,
                    total_point_count: self.algo_state.total_point_count,
                }]),
            );
        }
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
            let mut compute_pass = egui_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some(file!()),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(compute_pipeline);
            *compute_group_1 = compute_bindings::BindGroup1::from_bindings(
                device,
                compute_bindings::BindGroupEntries1 {
                    compute_output: compute_output_buffer.as_entire_buffer_binding(),
                    accepted: approx_accepted_buffer.as_entire_buffer_binding(),
                    count_info: approx_info_buffer.as_entire_buffer_binding(),
                },
            );
            compute_group_0.set(&mut compute_pass);
            compute_group_1.set(&mut compute_pass);
            compute_pass.dispatch_workgroups(self.px_size[0] as u32, self.px_size[1] as u32, 1);
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
            fragment_bindings::BindGroupEntries1 {
                compute_output: compute_output_buffer.as_entire_buffer_binding(),
                gauss_bases: target_buffer.as_entire_buffer_binding(),
            },
        );
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
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
