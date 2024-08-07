pub mod diff_display;
pub mod multimodal_gaussian;
mod resolution_uniform;

pub use resolution_uniform::INITIAL_RENDER_SIZE;

struct WgpuBufferBindGroupPair {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}
