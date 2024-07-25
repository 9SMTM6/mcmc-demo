use std::num::NonZero;

use wgpu::{util::DeviceExt, BufferBinding};

use crate::shaders::{multimodal_gaussian, types::ResolutionInfo};

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

    let resolution_buffer =
        device.create_buffer_init(&create_buffer_init_descr());

    let resolution_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: webgpu_debug_name,
        layout: &multimodal_gaussian::bind_groups::WgpuBindGroup0::get_bind_group_layout(device),
        entries: &multimodal_gaussian::bind_groups::WgpuBindGroupLayout0 {
            resolution_info: BufferBinding {
                buffer: &resolution_buffer,
                offset: 0,
                size: NonZero::new(16),
            },
        }
        .entries(),
    });

    WgpuBufferBindGroupPair {
        bind_group: resolution_bind_group,
        buffer: resolution_buffer,
    }
}