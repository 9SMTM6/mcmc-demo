use std::borrow::Cow;

use wgpu::{ShaderModule, ShaderModuleDescriptor, VertexState};

pub const FULLSCREEN_QUAD_VERTEX: ShaderModuleDescriptor<'static> = ShaderModuleDescriptor {
    label: Some(file!()),
    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("fullscreen_quad.wgsl"))),
};

pub fn get_fullscreen_quad_vertex<'shader>(shader: &'shader ShaderModule) -> VertexState<'shader> {
    VertexState {
        module: shader,
        buffers: &[],
        compilation_options: Default::default(),
        entry_point: "fullscreen_quad",
    }
}
