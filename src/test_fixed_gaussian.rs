use std::{mem::size_of_val, num::NonZeroU64};

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use wgpu::{util::DeviceExt, BindGroup, Buffer};
use wgpu::{
    FragmentState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, VertexState,
};

struct GaussPipeline {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    uniform_buffer: Buffer,
}

#[derive(Clone, Copy)]
pub struct FixedGaussian {}

const RENDER_SIZE: [f32; 2] = [64000.0, 48000.0];

impl FixedGaussian {
    pub fn new(render_state: &RenderState) -> Self {
        let device = &render_state.device;

        let webgpu_debug_name = Some("test_gaussian");

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: webgpu_debug_name,
            source: wgpu::ShaderSource::Wgsl(include_str!("test_fixed_gaussian.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: webgpu_debug_name,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(size_of_val(&RENDER_SIZE) as u64),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: webgpu_debug_name,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: VertexState {
                module: &shader,
                buffers: &[],
                compilation_options: Default::default(),
                entry_point: "vs_main",
            },
            fragment: Some(FragmentState {
                module: &shader,
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

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: webgpu_debug_name,
            contents: bytemuck::cast_slice(&RENDER_SIZE),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: webgpu_debug_name,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
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

        Self {}
    }
}

impl CallbackTrait for FixedGaussian {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let GaussPipeline { uniform_buffer, .. } = callback_resources.get().unwrap();
        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&RENDER_SIZE));
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
        render_pass.draw(0..1, 0..1);
    }
}

impl FixedGaussian {
    pub fn draw(&self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // ([0.0, 0.0].into()
            let rect = egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::from(RENDER_SIZE));
            // let (rect, response) =
            //     ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());
            ui.painter()
                .add(eframe::egui_wgpu::Callback::new_paint_callback(
                    rect,
                    self.clone(),
                ))
        });
    }
}
