use std::ops::Range;

pub struct ShaderParts {
    pub shader_vertice_num: Range<u32>,
}
// crate::shaders::fullscreen_quad::vertex_state(module, entry).create_shader_module_embed_source(device)

pub const FULLSCREEN_QUAD: ShaderParts = ShaderParts {
    shader_vertice_num: 0..6,
};
