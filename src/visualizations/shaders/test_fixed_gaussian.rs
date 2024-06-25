use std::marker::PhantomData;
use std::num::NonZero;

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use wgpu::{util::DeviceExt, BindGroup, Buffer};
use wgpu::{BufferBinding, RenderPipeline, RenderPipelineDescriptor};

use crate::shaders::resolution_uniform::ResolutionInfo;
use crate::shaders::{fullscreen_quad, test_fixed_gaussian};
use crate::visualizations::CanvasPainter;

use super::fullscreen_quad::FULLSCREEN_QUAD;
use super::resolution_uniform::create_buffer_init_descr;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Default)]
pub struct FixedGaussian {
    forbid_construct: PhantomData<GaussPipeline>,
}

impl CanvasPainter for FixedGaussian {
    fn paint(&mut self, painter: &egui::Painter, rect: egui::Rect) {
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            FixedGaussianRenderCall {
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

impl FixedGaussian {
    pub fn new(render_state: &RenderState) -> Self {
        let device = &render_state.device;

        let webgpu_debug_name = Some(file!());

        let layout = test_fixed_gaussian::create_pipeline_layout(device);

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: fullscreen_quad::vertex_state(
                &fullscreen_quad::create_shader_module_embed_source(device),
                &fullscreen_quad::fullscreen_quad_entry(),
            ),
            fragment: Some(test_fixed_gaussian::fragment_state(
                &test_fixed_gaussian::create_shader_module_embed_source(device),
                &test_fixed_gaussian::fs_main_entry([Some(render_state.target_format.into())]),
            )),
            label: webgpu_debug_name,
            layout: Some(&layout),
            depth_stencil: None,
            multiview: None,
            multisample: Default::default(),
            primitive: Default::default(),
        });

        let uniform_buffer = device.create_buffer_init(&create_buffer_init_descr());

        let bindings = test_fixed_gaussian::bind_groups::WgpuBindGroupLayout0 {
            resolution_info: BufferBinding {
                buffer: &uniform_buffer,
                offset: 0,
                size: NonZero::new(16),
            },
        };

        // dunno what that is for...
        // let bind_group = test_fixed_gaussian::bind_groups::WgpuBindGroup0::from_bindings(device, bindings);

        let bind_group_layout =
            test_fixed_gaussian::bind_groups::WgpuBindGroup0::get_bind_group_layout(device);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: webgpu_debug_name,
            layout: &bind_group_layout,
            entries: &bindings.entries(),
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

struct FixedGaussianRenderCall {
    px_size: [f32; 2],
}

impl CallbackTrait for FixedGaussianRenderCall {
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
            bytemuck::cast_slice(&[ResolutionInfo {
                resolution: self.px_size,
                _pad: [0.0; 2],
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
