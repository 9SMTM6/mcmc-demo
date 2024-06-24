use std::{borrow::Cow, ops::Range};

use wgpu::{ShaderModule, ShaderModuleDescriptor, VertexState};

pub struct ShaderParts {
    pub shader_module: ShaderModuleDescriptor<'static>,
    pub shader_vertice_num: Range<u32>,
    pub get_vertex_state: fn(&ShaderModule) -> VertexState<'_>,
}

pub const FULLSCREEN_QUAD: ShaderParts = ShaderParts {
    shader_module: ShaderModuleDescriptor {
        label: Some(file!()),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("fullscreen_quad.wgsl"))),
    },
    // Replace once https://github.com/gfx-rs/wgpu/pull/5872 lands
    //  wgpu::include_wgsl!("fullscreen_quad.wgsl"),
    get_vertex_state: |shader| VertexState {
        module: shader,
        buffers: &[],
        compilation_options: Default::default(),
        entry_point: "fullscreen_quad",
    },
    shader_vertice_num: 0..6,
};
