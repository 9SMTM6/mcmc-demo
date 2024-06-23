use std::{mem::size_of_val, num::NonZeroU64};

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use egui::epaint::{ColorMode, PathShape, PathStroke};
use egui::{Color32, Margin, Painter, Pos2, Shape, Stroke, Vec2};
use wgpu::{util::DeviceExt, BindGroup, Buffer};
use wgpu::{FragmentState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor};

use crate::fullscreen_quad::{get_fullscreen_quad_vertex, FULLSCREEN_QUAD_VERTEX};
use crate::INITIAL_RENDER_SIZE;

struct GaussPipeline {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    uniform_buffer: Buffer,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Default)]
pub struct FixedGaussian {}

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
                    // pad for compat
                    min_binding_size: NonZeroU64::new(
                        (size_of_val(&INITIAL_RENDER_SIZE) * 2) as u64,
                    ),
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
            contents: bytemuck::cast_slice(&[
                INITIAL_RENDER_SIZE[0],
                INITIAL_RENDER_SIZE[1],
                0.0,
                0.0,
            ]),
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
        render_pass.draw(0..6, 0..1);
    }
}

impl FixedGaussian {
    pub fn draw(&self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style())
            // remove margins here too
            .inner_margin(Margin::default())
            .outer_margin(Margin::default())
            .show(ui, |ui| {
                let px_size = ui.available_size();
                let rect = egui::Rect::from_min_size(ui.cursor().min, px_size);
                let px_size = <[f32; 2]>::from(px_size);
                // last painted element wins.
                let painter = ui.painter();
                painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
                    rect,
                    FixedGaussianRenderCall { px_size },
                ));
                let current_spot: Pos2 = [300.0, 400.0].into();
                arrow(painter, current_spot, [100.0, 100.0]);
                prediction_variance(painter, current_spot, 200.0);
                sampling_point(painter, current_spot, 0.65);
            });
    }
}

/// In contrast to the egui arrow, this arrow has an arrow head of constant size.
/// Note that the head will be added on top of start + direction, otherwise drawing an arrow of zero length is kinda awkward.
fn arrow(painter: &Painter, start: impl Into<Pos2>, direction: impl Into<Vec2>) {
    let start = start.into();
    let direction = direction.into();
    const HALF_HEAD_THICKNESS: f32 = 4.0;
    let dir_only = direction.normalized();

    let head = Shape::Path(PathShape {
        points: vec![
            start + direction + dir_only.rot90() * (-HALF_HEAD_THICKNESS),
            start + direction + dir_only * (2.0 * HALF_HEAD_THICKNESS),
            start + direction + dir_only.rot90() * HALF_HEAD_THICKNESS,
        ],
        closed: true,
        fill: Color32::RED,
        stroke: PathStroke::NONE,
    });
    let base = Shape::LineSegment {
        points: [start, start + direction],
        stroke: PathStroke {
            width: 1.5,
            color: ColorMode::Solid(Color32::RED),
        },
    };
    painter.extend([base, head]);
}

/// [`sample_count_fract`] shall be the number of samples at this point (=how long it stayed there/ how often a move away was rejected) divided by
/// the maximum of that count among all sample points.
fn sampling_point(painter: &Painter, pos: impl Into<Pos2>, sample_count_fract: f32) {
    painter.circle(
        pos.into(),
        4.0,
        Color32::WHITE.gamma_multiply(1.0 - sample_count_fract),
        Stroke::NONE,
    );
}

fn prediction_variance(painter: &Painter, pos: impl Into<Pos2>, radius: f32) {
    painter.circle(
        pos.into(),
        radius,
        Color32::TRANSPARENT,
        Stroke {
            color: Color32::WHITE,
            width: 1.0,
        },
    );
}
