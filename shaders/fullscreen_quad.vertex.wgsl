const NUM_VERTICES: u32 = 6;

@vertex
fn fullscreen_quad(@builtin(vertex_index) vertexIndex : u32) -> @invariant @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, NUM_VERTICES>(
        vec2(-1.0,  1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0, -1.0),
        vec2(-1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2( 1.0, -1.0),
    );
    return vec4(positions[vertexIndex], 0.0, 1.0);
}
