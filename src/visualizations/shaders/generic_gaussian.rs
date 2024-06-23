use std::marker::PhantomData;

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use wgpu::{util::DeviceExt, BindGroup, Buffer};
use wgpu::{FragmentState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor};

use crate::visualizations::CanvasPainter;

use super::fullscreen_quad::FULLSCREEN_QUAD;
use super::resolution_uniform::{
    create_buffer_init_descr, get_bind_group_entry, RESOLUTION_UNIFORM_FRAGMENT,
};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Default)]
pub struct GenericGaussian {
    forbid_construct: PhantomData<GaussPipeline>,
}

impl CanvasPainter for GenericGaussian {
    fn paint(&mut self, painter: &egui::Painter, rect: egui::Rect) {
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            GenericGaussianRenderCall {
                px_size: rect.size().into(),
            },
        ));
    }
}

struct GaussPipeline {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    uniform_buffer: Buffer,
}

impl GenericGaussian {
    pub fn new(render_state: &RenderState) -> Self {
        let device = &render_state.device;

        let webgpu_debug_name = Some(file!());

        let vertex_shader = device.create_shader_module(FULLSCREEN_QUAD.shader_module);

        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: webgpu_debug_name,
            source: wgpu::ShaderSource::Wgsl(include_str!("generic_gaussian.wgsl").into()),
        });

        let res_bind_group_layout = device.create_bind_group_layout(&RESOLUTION_UNIFORM_FRAGMENT);

        let distr_data_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(file!()),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            // pad with two zeros for compat:
                            // required for wgpu webgl fallback compatibility,
                            // no padding is fine on chrome with webgpu enabled
                            16,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            64
                        ),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: webgpu_debug_name,
            bind_group_layouts: &[&res_bind_group_layout, &distr_data_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: (FULLSCREEN_QUAD.get_vertex_state)(&vertex_shader),
            fragment: Some(FragmentState {
                module: &fragment_shader,
                compilation_options: Default::default(),
                entry_point: "fs_main",
                targets: &[Some(render_state.target_format.into())],
            }),
            label: webgpu_debug_name,
            layout: Some(&pipeline_layout),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: Default::default(),
            primitive: Default::default(),
        });

        let uniform_buffer = device.create_buffer_init(&create_buffer_init_descr());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: webgpu_debug_name,
            layout: &res_bind_group_layout,
            entries: &[get_bind_group_entry(&uniform_buffer)],
        });

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our struct, we insert it into the
        // `callback_resources` type map, which is stored alongside the render pass.
        let None = render_state
            .renderer
            .write()
            .callback_resources
            .insert(GaussPipeline {
                pipeline,
                bind_group,
                uniform_buffer,
            })
        else {
            panic!("pipeline already present?!")
        };

        Self {
            forbid_construct: PhantomData,
        }
    }
}

struct GenericGaussianRenderCall {
    px_size: [f32; 2],
}

impl CallbackTrait for GenericGaussianRenderCall {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        // doesn't hold the viewport size
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let GaussPipeline { uniform_buffer, .. } = callback_resources.get().unwrap();
        queue.write_buffer(
            uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.px_size[0], self.px_size[1], 0.0, 0.0]),
        );
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        let GaussPipeline {
            pipeline,
            bind_group,
            ..
        } = callback_resources.get().unwrap();

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(FULLSCREEN_QUAD.shader_vertice_num, 0..1);
    }
}
