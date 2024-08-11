use std::num::NonZero;

use wgpu::{util::DeviceExt, BufferBinding};

use crate::shaders::{multimodal_gaussian, canvas_ndc_conversion::ResolutionInfo};

use super::WgpuBufferBindGroupPair;

pub const INITIAL_RENDER_SIZE: [f32; 2] = [640.0, 480.0];

fn create_buffer_init_descr() -> wgpu::util::BufferInitDescriptor<'static> {
    wgpu::util::BufferInitDescriptor {
        label: Some(file!()),
        contents: bytemuck::cast_slice(&[ResolutionInfo {
            resolution: INITIAL_RENDER_SIZE,
            _pad: [0.0; 2],
        }]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    }
}

pub fn get_resolution_pair(device: &wgpu::Device) -> WgpuBufferBindGroupPair {
    let webgpu_debug_name = Some(file!());

    let buffer = device.create_buffer_init(&create_buffer_init_descr());

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: webgpu_debug_name,
        // I use the bindgroups out of the multimodal_gaussian namespace here,
        // but since these bind-groups are defined in common files, these can also be used for other
        // shaders, such as diff_display.
        //
        // Sadly I could not find a way to structure the files in such a way that I could make this easy to tell.
        // Rusts nominal type-checking is also none-the-wiser, since the generic wgpu types for buffer and bindgroup erase this info.
        layout: &multimodal_gaussian::WgpuBindGroup0::get_bind_group_layout(device),
        entries: &multimodal_gaussian::WgpuBindGroup0Entries::new(
            multimodal_gaussian::WgpuBindGroup0EntriesParams {
                resolution_info: BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: NonZero::new(16),
                },
            },
        )
        .as_array(),
    });

    WgpuBufferBindGroupPair { bind_group, buffer }
}
