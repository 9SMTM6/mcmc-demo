use std::sync::Arc;
use std::{mem::size_of_val, num::NonZeroU64};

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use egui::mutex::Mutex;
use egui::Vec2;
use wgpu::{util::DeviceExt, BindGroup, Buffer};
use wgpu::{
    FragmentState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
};

use crate::fullscreen_quad::{get_fullscreen_quad_vertex, FULLSCREEN_QUAD_VERTEX};
use crate::RENDER_SIZE;

struct GaussPipeline {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    uniform_buffer: Buffer,
}

#[derive(Clone)]
pub struct FixedGaussian {
    // theres certainly more elegant solutions than a mutex, but honestly I'm over it at this point. It works. Hopefully.
    pub px_size: Arc<Mutex<Vec2>>,
}

impl FixedGaussian {
    pub fn new(render_state: &RenderState) -> Self {
        let device = &render_state.device;

        let webgpu_debug_name = Some(file!());

        let vertex_shader = device.create_shader_module(FULLSCREEN_QUAD_VERTEX);

        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
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
            vertex: get_fullscreen_quad_vertex(&vertex_shader),
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

        Self {
            px_size: Arc::new(Mutex::new(RENDER_SIZE.into())),
        }
    }
}

impl CallbackTrait for FixedGaussian {
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
        let px_size = {
            self.px_size.lock().clone()
        };
        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&<[f32;2]>::from(px_size)));
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        // callback resources aint mutable, no way to get data to prepare :-/
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        {
            // I need to get this to prepare, where I can write the uniform buffer which can be read from wgsl
            *self.px_size.lock() = info.viewport.size();
        }
        let GaussPipeline {
            pipeline,
            bind_group,
            ..
        } = callback_resources.get().unwrap();

        // might be interesting
        // render_pass.set_bind_group(index, bind_group, offsets);
        // this wont work. Needs native: https://docs.rs/wgpu-types/0.20.0/wgpu_types/struct.Features.html#associatedconstant.PUSH_CONSTANTS
        // render_pass.set_push_constants(stages, offset, data)

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

impl FixedGaussian {
    pub fn draw(&self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let rect = egui::Rect::from_min_size(ui.cursor().min, ui.available_size());
            ui.painter()
                .add(eframe::egui_wgpu::Callback::new_paint_callback(
                    rect,
                    self.clone(),
                ))
        });
    }
}
