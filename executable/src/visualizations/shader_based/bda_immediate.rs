use std::sync::Arc;

use eframe::egui_wgpu::CallbackTrait;
use macros::cfg_persistence_derive;
use tokio::sync::Notify;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferDescriptor, BufferUsages, Device, RenderPipeline, RenderPipelineDescriptor,
};

use crate::{
    create_shader_module, profile_scope,
    simulation::random_walk_metropolis_hastings::Rwmh,
    target_distr,
    visualizations::{
        shader_based::{
            resolution_uniform::get_resolution_buffer,
            target_distr::{get_normaldistr_buffer, shader_bindings::NormalDistribution},
        },
        AlgoPainter,
    },
};

use super::fullscreen_quad;

create_shader_module!("diff_display.fragment");

use shader_bindings::{
    bind_groups::{BindGroup0, BindGroup1},
    BindGroupLayout0, BindGroupLayout1, RWMHAcceptRecord, RWMHCountInfo, ResolutionInfo,
};

#[cfg_persistence_derive]
#[derive(Default)]
pub struct BDADiff {
    // pub window_radius: f32,
}

pub fn get_approx_buffers(
    device: &wgpu::Device,
    approx_points: Option<&[RWMHAcceptRecord]>,
) -> (wgpu::Buffer, wgpu::Buffer) {
    let webgpu_debug_name = Some(file!());

    let accept_buf_use = BufferUsages::COPY_DST | BufferUsages::STORAGE;

    let accept_buffer = match approx_points {
        Some(approx_points) => device.create_buffer_init(&BufferInitDescriptor {
            label: webgpu_debug_name,
            usage: accept_buf_use,
            contents: bytemuck::cast_slice(approx_points),
        }),
        None => device.create_buffer(&BufferDescriptor {
            label: webgpu_debug_name,
            usage: accept_buf_use,
            mapped_at_creation: false,
            size: 4,
        }),
    };

    let info_buffer = device.create_buffer(&BufferDescriptor {
        label: webgpu_debug_name,
        size: size_of::<RWMHCountInfo>() as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });

    (accept_buffer, info_buffer)
}

pub struct PipelineStateHolder {
    pipeline: RenderPipeline,
    bind_group_0: shader_bindings::bind_groups::BindGroup0,
    bind_group_1: shader_bindings::bind_groups::BindGroup1,
    resolution_buffer: Buffer,
    target_buffer: Buffer,
    approx_accepted_buffer: Buffer,
    approx_info_buffer: Buffer,
}

impl AlgoPainter for BDADiff {
    fn paint(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        algo: std::sync::Arc<Rwmh>,
        target: &target_distr::Gaussian,
    ) {
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderCall {
                algo_state: algo.clone(),
                px_size: rect.size().into(),
                targets: target.gaussians.clone(),
            },
        ));
    }
}

impl PipelineStateHolder {
    pub fn create(
        device: &Device,
        target_format: wgpu::ColorTargetState,
        _refresh_token: Arc<Notify>,
    ) -> Self {
        let webgpu_debug_name = Some(file!());

        let layout = shader_bindings::create_pipeline_layout(device);

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: fullscreen_quad::vertex_state(
                &fullscreen_quad::create_shader_module(device),
                &fullscreen_quad::fullscreen_quad_entry(),
            ),
            fragment: Some(shader_bindings::fragment_state(
                &shader_bindings::create_shader_module(device),
                &shader_bindings::fs_main_entry([Some(target_format)]),
            )),
            label: webgpu_debug_name,
            layout: Some(&layout),
            depth_stencil: None,
            multiview: None,
            multisample: Default::default(),
            primitive: Default::default(),
            cache: None,
        });

        let resolution_buffer = get_resolution_buffer(device);

        let normdistr_buffer = get_normaldistr_buffer(device, None);

        let (approx_accepted_buffer, approx_info_buffer) = get_approx_buffers(device, None);

        let bind_group_0 = BindGroup0::from_bindings(
            device,
            BindGroupLayout0 {
                resolution_info: resolution_buffer.as_entire_buffer_binding(),
            },
        );

        let bind_group_1 = BindGroup1::from_bindings(
            device,
            BindGroupLayout1 {
                accepted: approx_accepted_buffer.as_entire_buffer_binding(),
                count_info: approx_info_buffer.as_entire_buffer_binding(),
                gauss_bases: normdistr_buffer.as_entire_buffer_binding(),
            },
        );

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our struct, we insert it into the
        // `callback_resources` type map, which is stored alongside the render pass.
        Self {
            pipeline,
            bind_group_0,
            bind_group_1,
            resolution_buffer,
            target_buffer: normdistr_buffer,
            approx_accepted_buffer,
            approx_info_buffer,
        }
    }
}

struct RenderCall {
    px_size: [f32; 2],
    targets: Vec<NormalDistribution>,
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
            ref mut approx_accepted_buffer,
            ref mut approx_info_buffer,
            ref mut bind_group_1,
            ..
        } = callback_resources.get_mut().unwrap();
        let target = self.targets.as_slice();
        if target_buffer.size() as usize != size_of_val(target) {
            let normdistr_buffer = get_normaldistr_buffer(device, Some(target));
            *target_buffer = normdistr_buffer;
        }
        let approx_accepted = self.algo_state.history.as_slice();
        if approx_accepted_buffer.size() as usize != size_of_val(approx_accepted) {
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
        // TODO: only write these if they actually changed.
        queue.write_buffer(
            resolution_buffer,
            0,
            bytemuck::cast_slice(&[ResolutionInfo {
                resolution: self.px_size,
                _pad: [0.0; 2],
            }]),
        );
        queue.write_buffer(
            target_buffer,
            0,
            bytemuck::cast_slice(self.targets.as_slice()),
        );
        // TODO: only reassign if required.
        // If that actually speeds things up, I dunno.
        *bind_group_1 = BindGroup1::from_bindings(
            device,
            BindGroupLayout1 {
                accepted: approx_accepted_buffer.as_entire_buffer_binding(),
                count_info: approx_info_buffer.as_entire_buffer_binding(),
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
        profile_scope!("draw diff_display");
        let &PipelineStateHolder {
            ref pipeline,
            ref bind_group_0,
            ref bind_group_1,
            ..
        } = callback_resources.get().unwrap();
        render_pass.set_pipeline(pipeline);
        bind_group_0.set(render_pass);
        bind_group_1.set(render_pass);
        render_pass.draw(0..fullscreen_quad::NUM_VERTICES, 0..1);
    }
}
