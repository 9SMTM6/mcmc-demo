use eframe::egui_wgpu::{CallbackTrait, RenderState};
use tokio::sync::mpsc::Sender;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferDescriptor, BufferUsages, RenderPipeline, RenderPipelineDescriptor,
};

use crate::{
    gpu_task::GpuTaskEnum, simulation::random_walk_metropolis_hastings::Rwmh,
    target_distributions::multimodal_gaussian::GaussianTargetDistr, visualizations::AlgoPainter,
};

use super::{fullscreen_quad, resolution_uniform::get_resolution_buffer};

use crate::create_shader_module;

create_shader_module!("multimodal_gaussian.fragment");

use shader_bindings::{
    bind_groups::{BindGroup0, BindGroup1},
    BindGroupLayout0, BindGroupLayout1, ResolutionInfo,
};

pub use shader_bindings::NormalDistribution;

struct MultiModalGaussPipeline {
    pipeline: RenderPipeline,
    bind_group_0: BindGroup0,
    bind_group_1: BindGroup1,
    resolution_buffer: Buffer,
    target_buffer: Buffer,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default)]
pub struct TargetDistribution {
    // pub color: Color32,
}

impl AlgoPainter for TargetDistribution {
    fn paint(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        _algo: std::sync::Arc<Rwmh>,
        target: &GaussianTargetDistr,
    ) {
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderCall {
                px_size: rect.size().into(),
                elements: target.gaussians.clone(),
            },
        ));
    }
}

pub(super) fn get_normaldistr_buffer(
    device: &wgpu::Device,
    distr: Option<&[NormalDistribution]>,
) -> wgpu::Buffer {
    let webgpu_debug_name = Some(file!());

    let buf_use = BufferUsages::COPY_DST | BufferUsages::STORAGE;

    match distr {
        Some(distr) => device.create_buffer_init(&BufferInitDescriptor {
            label: webgpu_debug_name,
            usage: buf_use,
            contents: bytemuck::cast_slice(distr),
        }),
        None => device.create_buffer(&BufferDescriptor {
            label: webgpu_debug_name,
            usage: buf_use,
            mapped_at_creation: false,
            size: 4,
        }),
    }
}

impl TargetDistribution {
    pub fn init_pipeline(render_state: &RenderState, _: Sender<GpuTaskEnum>) {
        let device = &render_state.device;

        let webgpu_debug_name = Some(file!());

        let layout = shader_bindings::create_pipeline_layout(device);

        // chrome: Bgra8Unorm
        // native linux vulkan: Rgba8Unorm
        // yup, its different.
        // tracing::warn!("{0:?}", render_state.target_format);

        let pipeline: RenderPipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: fullscreen_quad::vertex_state(
                &fullscreen_quad::create_shader_module(device),
                &fullscreen_quad::fullscreen_quad_entry(),
            ),
            fragment: Some(shader_bindings::fragment_state(
                &shader_bindings::create_shader_module(device),
                &shader_bindings::fs_main_entry([Some(render_state.target_format.into())]),
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

        let bind_group_0 = BindGroup0::from_bindings(
            device,
            BindGroupLayout0 {
                resolution_info: resolution_buffer.as_entire_buffer_binding(),
            },
        );

        let bind_group_1 = BindGroup1::from_bindings(
            device,
            BindGroupLayout1 {
                gauss_bases: normdistr_buffer.as_entire_buffer_binding(),
            },
        );

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
                    bind_group_0,
                    bind_group_1,
                    resolution_buffer,
                    target_buffer: normdistr_buffer,
                })
        else {
            unreachable!("pipeline already present?!")
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // doesn't hold the viewport size (though something fairly similar?!)
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let &mut MultiModalGaussPipeline {
            ref mut resolution_buffer,
            ref mut target_buffer,
            ref mut bind_group_1,
            ..
        } = callback_resources.get_mut().unwrap();
        // TODO: figure that out
        // let &mut MultiModalGaussPipeline {
        //     ref mut resolution_buffer,
        //     ref mut target_buffer,
        //     ref mut target_bind_group,
        //     ..
        // } = {
        //     let ret = callback_resources.get_mut::<MultiModalGaussPipeline>();
        //     if ret.is_none() {
        //         MultiModalGaussianDisplay::init_gaussian_pipeline(todo!(
        //             "cant find no way to get this here"
        //         ));
        //         callback_resources
        //             .get_mut::<MultiModalGaussPipeline>()
        //             .unwrap()
        //     } else {
        //         ret.unwrap()
        //     }
        // };
        let target = self.elements.as_slice();
        if target_buffer.size() as usize != size_of_val(target) {
            let normdistr_buffer = get_normaldistr_buffer(device, Some(target));
            *target_buffer = normdistr_buffer;
        }
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
            bytemuck::cast_slice(self.elements.as_slice()),
        );
        // TODO: only reassign of required.
        // If that actually speeds things up, I dunno.
        // See https://github.com/ScanMountGoat/wgsl_to_wgpu/tree/main?tab=readme-ov-file#bind-groups
        // > Note that bind groups store references to their underlying resource bindings,
        // > so it is not necessary to recreate a bind group if the only the uniform or storage buffer contents change.
        // > Avoid creating new bind groups during rendering if possible for best performance.
        *bind_group_1 = BindGroup1::from_bindings(
            device,
            BindGroupLayout1 {
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
        let &MultiModalGaussPipeline {
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
