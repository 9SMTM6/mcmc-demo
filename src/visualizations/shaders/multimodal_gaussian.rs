use std::mem::size_of_val;
use std::num::NonZero;

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use wgpu::util::BufferInitDescriptor;
use wgpu::{util::DeviceExt, BindGroup, Buffer};
use wgpu::{BufferBinding, BufferUsages, RenderPipeline, RenderPipelineDescriptor};

use crate::shaders::multimodal_gaussian::NormalDistribution;
use crate::shaders::resolution_uniform::ResolutionInfo;
use crate::shaders::{fullscreen_quad, multimodal_gaussian};
use crate::visualizations::CanvasPainter;

use super::fullscreen_quad::FULLSCREEN_QUAD;
use super::resolution_uniform::create_buffer_init_descr;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
pub struct MultiModalGaussian {
    pub gaussians: Vec<NormalDistribution>,
}

impl Default for MultiModalGaussian {
    fn default() -> Self {
        Self {
            gaussians: [
                NormalDistribution {
                    position: [-1.0, -1.0],
                    scale: 0.5,
                    variance: 0.14,
                },
                NormalDistribution {
                    position: [0.2, -0.2],
                    scale: 0.6,
                    variance: 0.2,
                },
                NormalDistribution {
                    position: [0.9, -0.3],
                    scale: 0.4,
                    variance: 0.01,
                },
                NormalDistribution {
                    position: [0.1, -0.6],
                    scale: 0.8,
                    variance: 0.4,
                },
                NormalDistribution {
                    position: [-1.0, 0.5],
                    scale: 1.4,
                    variance: 0.1,
                },
            ]
            .into(),
        }
    }
}

impl CanvasPainter for MultiModalGaussian {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderCall {
                px_size: rect.size().into(),
                elements: self.gaussians.clone(),
            },
        ));
    }
}

struct MultiModalGaussPipeline {
    pipeline: RenderPipeline,
    resolution_bind_group: BindGroup,
    elements_bind_group: BindGroup,
    resolution_buffer: Buffer,
    elements_buffer: Buffer,
}

impl MultiModalGaussian {
    pub fn init_gaussian_pipeline(&self, render_state: &RenderState) {
        let device = &render_state.device;

        let webgpu_debug_name = Some(file!());

        let layout = multimodal_gaussian::create_pipeline_layout(device);

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: fullscreen_quad::vertex_state(
                &fullscreen_quad::create_shader_module_embed_source(device),
                &fullscreen_quad::fullscreen_quad_entry(),
            ),
            fragment: Some(multimodal_gaussian::fragment_state(
                &multimodal_gaussian::create_shader_module_embed_source(device),
                &multimodal_gaussian::fs_main_entry([Some(render_state.target_format.into())]),
            )),
            label: webgpu_debug_name,
            layout: Some(&layout),
            depth_stencil: None,
            multiview: None,
            multisample: Default::default(),
            primitive: Default::default(),
        });

        let resolution_buffer = device.create_buffer_init(&create_buffer_init_descr());

        let resolution_bindings = multimodal_gaussian::bind_groups::WgpuBindGroupLayout0 {
            resolution_info: BufferBinding {
                buffer: &resolution_buffer,
                offset: 0,
                size: NonZero::new(16),
            },
        };

        // dunno what that is for...
        // let bind_group = test_fixed_gaussian::bind_groups::WgpuBindGroup0::from_bindings(device, bindings);

        let res_bind_group_layout =
            multimodal_gaussian::bind_groups::WgpuBindGroup0::get_bind_group_layout(device);

        let resolution_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: webgpu_debug_name,
            layout: &res_bind_group_layout,
            entries: &resolution_bindings.entries(),
        });

        let elements_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(file!()),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            contents: bytemuck::cast_slice(self.gaussians.as_slice()),
        });

        let el_bindings = multimodal_gaussian::bind_groups::WgpuBindGroupLayout1 {
            gauss_bases: BufferBinding {
                buffer: &elements_buffer,
                offset: 0,
                size: NonZero::new(size_of_val(self.gaussians.as_slice()) as u64),
            },
        };

        let el_bind_group_layout =
            multimodal_gaussian::bind_groups::WgpuBindGroup1::get_bind_group_layout(device);

        let elements_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: webgpu_debug_name,
            layout: &el_bind_group_layout,
            entries: &el_bindings.entries(),
        });

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our struct, we insert it into the
        // `callback_resources` type map, which is stored alongside the render pass.
        let None =
            render_state
                .renderer
                .write()
                .callback_resources
                .insert(MultiModalGaussPipeline {
                    pipeline,
                    resolution_bind_group,
                    elements_bind_group,
                    resolution_buffer,
                    elements_buffer,
                })
        else {
            panic!("pipeline already present?!")
        };
    }
}

struct RenderCall {
    px_size: [f32; 2],
    elements: Vec<NormalDistribution>,
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
        let MultiModalGaussPipeline {
            resolution_buffer,
            elements_buffer,
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
            elements_buffer,
            0,
            bytemuck::cast_slice(self.elements.as_slice()),
        );
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        let MultiModalGaussPipeline {
            pipeline,
            resolution_bind_group,
            elements_bind_group,
            ..
        } = callback_resources.get().unwrap();

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, resolution_bind_group, &[]);
        render_pass.set_bind_group(1, elements_bind_group, &[]);
        render_pass.draw(FULLSCREEN_QUAD.shader_vertice_num, 0..1);
    }
}
