@group(0) @binding(0) var<uniform> resolution: vec2<f32>;

const len = 5;

const PI = radians(180.0);

// var because of https://github.com/gfx-rs/wgpu/issues/4337
var<private> gauss_centers: array<vec2<f32>, len> = array(
    vec2(-1.0, -1.0),
    vec2(0.2, -0.2),
    vec2(0.9, -0.3),
    vec2(0.1, -0.6),
    vec2(-1.0, 0.5),
);

var<private> gauss_scales: array<f32, len> = array(
    .5,
    .6,
    .0,
    .0,
    .0,
);

var<private> gauss_variance: array<f32, len> = array(
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

    var combined_prob_density = 0.0;

    var normalization = 0.0;

    // TODO: consider how to do log-density display, which is what original does.
    // it does a bunch of math which 1. takes time to implement and 
    // 2. I'm not sure whether it would be faster to calculate this on the gpu in every shader
    //  and reduce data transfer or to calculate that on the cpu beforehand (and for that to work we also need to pass in the arrays anyways) 

    for (var i=0; i<len; i+=1) {
        let scale = gauss_scales[i];
        let variance = gauss_variance[i];
        let position = gauss_centers[i];

        // required to calculate here because these use arrays, and max doesnt accept these.
        // in addition, top level constants still dont work with wgpu.
        let gauss_normalize = inverseSqrt(2 * PI * variance);
        let sq_dist = pow(distance(normalized_coords, position), 2.0);

        let prob_contrib = gauss_normalize * exp(-sq_dist / (2 * variance));
        combined_prob_density+= scale * prob_contrib;
        normalization += scale;
    }

    combined_prob_density /= normalization;

    return vec4(vec3(0, combined_prob_density, 0), 1.0);
}
