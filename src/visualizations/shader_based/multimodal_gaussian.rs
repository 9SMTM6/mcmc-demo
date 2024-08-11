use std::num::NonZero;

use eframe::egui_wgpu::{CallbackTrait, RenderState};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, BufferBinding, BufferDescriptor, BufferUsages, RenderPipeline,
    RenderPipelineDescriptor,
};

use crate::shaders::{
    fullscreen_quad, multimodal_gaussian,
    types::{NormalDistribution, ResolutionInfo},
};
use crate::target_distributions::multimodal_gaussian::MultiModalGaussian;

use super::{resolution_uniform::get_resolution_pair, WgpuBufferBindGroupPair};

struct MultiModalGaussPipeline {
    pipeline: RenderPipeline,
    resolution_bind_group: BindGroup,
    target_bind_group: BindGroup,
    resolution_buffer: Buffer,
    target_buffer: Buffer,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct MultiModalGaussianDisplay {
    // pub color: Color32,
}

impl MultiModalGaussianDisplay {
    pub fn paint(&self, distr: &MultiModalGaussian, painter: &egui::Painter, rect: egui::Rect) {
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderCall {
                px_size: rect.size().into(),
                elements: distr.gaussians.clone(),
            },
        ));
    }
}

/// this can also be used elsewhere, e.g. diff_display.
pub(super) fn get_gaussian_target_pair(
    device: &wgpu::Device,
    distr: Option<&[NormalDistribution]>,
) -> WgpuBufferBindGroupPair {
    let webgpu_debug_name = Some(file!());

    let buf_use = BufferUsages::COPY_DST | BufferUsages::STORAGE;

    let buffer = match distr {
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
    };

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: webgpu_debug_name,
        layout: &multimodal_gaussian::WgpuBindGroup1::get_bind_group_layout(device),
        entries: &multimodal_gaussian::WgpuBindGroup1Entries::new(
            multimodal_gaussian::WgpuBindGroup1EntriesParams {
                gauss_bases: BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: NonZero::new(buffer.size()),
                },
            },
        )
        .as_array(),
    });

    WgpuBufferBindGroupPair { bind_group, buffer }
}

impl MultiModalGaussianDisplay {
    pub fn init_gaussian_pipeline(render_state: &RenderState) {
        let device = &render_state.device;

        let webgpu_debug_name = Some(file!());

        let layout = multimodal_gaussian::create_pipeline_layout(device);

        // chrome: Bgra8Unorm
        // native linux vulkan: Rgba8Unorm
        // yup, its different.
        // log::warn!("{0:?}", render_state.target_format);

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
            cache: None,
        });

        let WgpuBufferBindGroupPair {
            bind_group: resolution_bind_group,
            buffer: resolution_buffer,
        } = get_resolution_pair(device);

        let WgpuBufferBindGroupPair {
            bind_group: elements_bind_group,
            buffer: elements_buffer,
        } = get_gaussian_target_pair(device, None);

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
                    target_bind_group: elements_bind_group,
                    resolution_buffer,
                    target_buffer: elements_buffer,
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
            ref mut target_bind_group,
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
            let WgpuBufferBindGroupPair { buffer, bind_group } =
                get_gaussian_target_pair(device, Some(target));
            *target_buffer = buffer;
            *target_bind_group = bind_group;
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
            ref resolution_bind_group,
            target_bind_group: ref elements_bind_group,
            ..
        } = callback_resources.get().unwrap();

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, resolution_bind_group, &[]);
        render_pass.set_bind_group(1, elements_bind_group, &[]);
        render_pass.draw(0..fullscreen_quad::NUM_VERTICES, 0..1);
    }
}
