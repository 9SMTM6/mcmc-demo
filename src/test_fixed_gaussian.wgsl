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

// struct VertexOut {
//     color: vec4<f32>,
//     @builtin(position) position: vec4<f32>,
// };

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> @builtin(position) vec4<f32> {
    return vec4(0.0);
}

@fragment
fn fs_main(@builtin(position) pixel_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_coords = pixel_coords.xy / max(resolution.x, resolution.y) * 2 - 1;
    // let normalized_coords = vec2(
    //     pixel_coords.x / max(resolution.x, resolution.y) * 2 - 1.0,
    //     pixel_coords.y / max(resolution.x, resolution.y) * 2 - 1.0,
    // );

    let color = 1 - distance(normalized_coords, vec2(0.0));
    // vec3(color)
    return vec4(vec3(color), 0.0);
}
