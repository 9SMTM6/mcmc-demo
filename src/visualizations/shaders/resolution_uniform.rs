//! The corresponding WGSL is:
//! ```wgsl
//! struct ResolutionInfo {
//!     resolution: vec2<f32>,
//!     // See corresponding bindinggroup for reason
//!     _pad: vec2<f32>,
//! }
//! 
//! @group(0) @binding(0) 
//! var<uniform> resolution_info: ResolutionInfo;
//! ```

pub const INITIAL_RENDER_SIZE: [f32; 2] = [640.0, 480.0];

/// An uniform (-BindGroupLayoutDescriptor) to pass the resolution into fragments at bind_id = 0
pub const RESOLUTION_UNIFORM_FRAGMENT : wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
    label: Some(file!()),
    entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: std::num::NonZeroU64::new(
                // pad with two zeros for compat:
                // required for wgpu webgl fallback compatibility,
                // no padding is fine on chrome with webgpu enabled
                std::mem::size_of::<[f32; 4]>() as u64
            ),
        },
        count: None,
    }],
};

pub fn create_buffer_init_descr() -> wgpu::util::BufferInitDescriptor<'static> {
    wgpu::util::BufferInitDescriptor {
        label: Some(file!()),
        contents: bytemuck::cast_slice(&[
            INITIAL_RENDER_SIZE[0],
            INITIAL_RENDER_SIZE[1],
            0.0,
            0.0,
        ]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    }
}

pub fn get_bind_group_entry(uniform_buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding: 0,
        resource: uniform_buffer.as_entire_binding(),
    }
}
