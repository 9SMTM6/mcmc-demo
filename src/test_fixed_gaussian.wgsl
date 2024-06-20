@group(0) @binding(0) var<uniform> resolution: vec2<f32>;

const len = 5;

var<private> gauss_centers: array<vec2<f32>, len> = array(
    vec2(0.0, 0.0),
    vec2(0.2, 0.4),
    vec2(0.9, 0.3),
    vec2(0.1, 0.6),
    vec2(1.0, 0.5),
);

const gauss_scales: array<f32, len> = array(
    .5,
    .6,
    .8,
    .2,
    .5,
);

const gauss_variance: array<f32, len> = array(
    .5,
    .2,
    .5,
    .8,
    .6,
);

@vertex
fn fullscreen_quad_vertex(@builtin(vertex_index) vertexIndex : u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 6>(
        vec2(-1.0,  1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0, -1.0),
        vec2(-1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2( 1.0, -1.0),
    );
    return vec4(positions[vertexIndex], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pixel_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_coords = pixel_coords.xy / max(resolution.x, resolution.y) * 2 - 1;

    let color = 1 - distance(normalized_coords, vec2(0.0));
    return vec4(vec3(color), 1.0);
}
