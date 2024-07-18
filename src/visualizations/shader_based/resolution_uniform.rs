use crate::shaders::types::ResolutionInfo;

pub const INITIAL_RENDER_SIZE: [f32; 2] = [640.0, 480.0];

pub fn create_buffer_init_descr() -> wgpu::util::BufferInitDescriptor<'static> {
    wgpu::util::BufferInitDescriptor {
        label: Some(file!()),
        contents: bytemuck::cast_slice(&[ResolutionInfo {
            resolution: INITIAL_RENDER_SIZE,
            _pad: [0.0; 2],
        }]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    }
}
