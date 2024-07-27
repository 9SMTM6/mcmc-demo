#![allow(unused)]
use std::{marker::PhantomData, mem::size_of, num::NonZero};

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use egui::{Color32, Pos2};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, BufferBinding, BufferDescriptor, BufferUsages, RenderPipeline,
    RenderPipelineDescriptor,
};

use crate::{
    app::ndc_to_canvas_coord,
    shaders::{
        self, diff_display, fullscreen_quad,
        types::{NormalDistribution, RWMHAcceptRecord, RWMHCountInfo, ResolutionInfo},
    },
    simulation::random_walk_metropolis_hastings::{AcceptRecord, Rwmh},
    target_distributions::multimodal_gaussian::MultiModalGaussian,
    visualizations::{
        self,
        shader_based::{
            multimodal_gaussian::get_gaussian_target_pair, resolution_uniform::get_resolution_pair,
            WgpuBufferBindGroupPair,
        },
        CanvasPainter,
    },
};

use super::resolution_uniform;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct DiffDisplay {
    pub window_radius: f32,
    prevent_construct: PhantomData<()>,
}

fn get_approx_triple(
    device: &wgpu::Device,
    approx_points: &Vec<shaders::types::RWMHAcceptRecord>,
) -> (BindGroup, Buffer, Buffer) {
    let webgpu_debug_name = Some(file!());

    let accept_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: webgpu_debug_name,
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        contents: bytemuck::cast_slice(approx_points.as_slice()),
    });

    let info_buffer_size = size_of::<shaders::types::RWMHCountInfo>() as u64;

    let info_buffer = device.create_buffer(&BufferDescriptor {
        label: webgpu_debug_name,
        size: info_buffer_size,
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });

    let approx_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: webgpu_debug_name,
        layout: &diff_display::bind_groups::WgpuBindGroup2::get_bind_group_layout(device),
        entries: &diff_display::bind_groups::WgpuBindGroupLayout2 {
            accepted: BufferBinding {
                buffer: &accept_buffer,
                offset: 0,
                size: NonZero::new(size_of_val(approx_points.as_slice()) as u64),
            },
            count_info: BufferBinding {
                buffer: &info_buffer,
                offset: 0,
                size: NonZero::new(info_buffer_size),
            },
        }
        .entries(),
    });

    (approx_bind_group, accept_buffer, info_buffer)
}

struct PipelineStateHolder {
    pipeline: RenderPipeline,
    resolution_bind_group: BindGroup,
    target_bind_group: BindGroup,
    approx_bind_group: BindGroup,
    resolution_buffer: Buffer,
    target_buffer: Buffer,
    approx_accepted_buffer: Buffer,
    approx_info_buffer: Buffer,
}

impl DiffDisplay {
    #[allow(unused_variables)]
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
                targets: target.gaussians.clone(),
            },
        ));
    }

    pub fn init_pipeline(
        // TODO: this doesnt actually do anything yet
        window_radius: f32,
        distr: &MultiModalGaussian,
        algo: &Rwmh,
        render_state: &RenderState,
    ) -> Self {
        let device = &render_state.device;

        let webgpu_debug_name = Some(file!());

        let layout = diff_display::create_pipeline_layout(device);

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: fullscreen_quad::vertex_state(
                &fullscreen_quad::create_shader_module_embed_source(device),
                &fullscreen_quad::fullscreen_quad_entry(),
            ),
            fragment: Some(diff_display::fragment_state(
                &diff_display::create_shader_module_embed_source(device),
                &diff_display::fs_main_entry([Some(render_state.target_format.into())]),
            )),
            label: webgpu_debug_name,
            layout: Some(&layout),
            depth_stencil: None,
            multiview: None,
            multisample: Default::default(),
            primitive: Default::default(),
        });

        let WgpuBufferBindGroupPair {
            bind_group: resolution_bind_group,
            buffer: resolution_buffer,
        } = get_resolution_pair(device);

        let WgpuBufferBindGroupPair {
            bind_group: target_bind_group,
            buffer: target_buffer,
        } = get_gaussian_target_pair(device, distr);

        let (approx_bind_group, approx_accepted_buffer, approx_info_buffer) =
            get_approx_triple(device, &algo.history);

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our struct, we insert it into the
        // `callback_resources` type map, which is stored alongside the render pass.
        // let None =
        render_state
            .renderer
            .write()
            .callback_resources
            .insert(PipelineStateHolder {
                pipeline,
                resolution_bind_group,
                target_bind_group,
                resolution_buffer,
                target_buffer,
                approx_accepted_buffer,
                approx_bind_group,
                approx_info_buffer,
            });
        // else {
        //     panic!("pipeline already present?!")
        // };
        Self {
            window_radius,
            prevent_construct: PhantomData,
        }
    }
}

struct RenderCall {
    px_size: [f32; 2],
    targets: Vec<NormalDistribution>,
    algo_state: Rwmh,
}

impl CallbackTrait for RenderCall {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        // doesn't hold the viewport size
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let PipelineStateHolder {
            resolution_buffer,
            target_buffer,
            approx_accepted_buffer,
            approx_info_buffer,
            ..
        } = callback_resources.get().unwrap();
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
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        let PipelineStateHolder {
            pipeline,
            resolution_bind_group,
            target_bind_group,
            approx_bind_group,
            ..
        } = callback_resources.get().unwrap();

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, resolution_bind_group, &[]);
        render_pass.set_bind_group(1, target_bind_group, &[]);
        render_pass.set_bind_group(2, approx_bind_group, &[]);
        render_pass.draw(0..fullscreen_quad::NUM_VERTICES, 0..1);
    }
}