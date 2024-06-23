struct ResolutionInfo {
    resolution: vec2<f32>,
    // pad to 16 byte for compatibility
    // required for wgpu webgl fallback compatibility,
    // no padding is fine on chrome with webgpu enabled
    pad: vec2<f32>,
}

@group(0) @binding(0) 
var<uniform> resolution_info: ResolutionInfo;

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
    .4,
    .8,
    1.4,
);

var<private> gauss_variance: array<f32, len> = array(
    .14,
    .2,
    .01,
    .4,
    .1,
);

@fragment
fn fs_main(@builtin(position) pixel_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let resolution = resolution_info.resolution;
    let normalized_device_coords = pixel_coords.xy / max(resolution.x, resolution.y) * 2 - 1;

    var combined_prob_density = 0.0;

    var normalization = 0.0;

    // TODO: consider how to do log-density display, which is what original does.
    // it does a bunch of math which 
    // 1. takes time to implement and 
    // 2. I'm not sure whether it would be faster to calculate this on the gpu in every shader
    //  and reduce data transfer or to calculate that on the cpu beforehand (and for that to work we also need to pass in the arrays anyways) 

    for (var i=0; i<len; i+=1) {
        let scale = gauss_scales[i];
        let variance = gauss_variance[i];
        let position = gauss_centers[i];

        // for now we calcualte this here, we might test later if this is better or worse than calculating it once on the cpu and delivering it on each render.
        let gauss_normalize = inverseSqrt(2 * PI * variance);
        let sq_dist = pow(distance(normalized_device_coords, position), 2.0);

        let prob_contrib = gauss_normalize * exp(-sq_dist / (2 * variance));
        combined_prob_density+= scale * prob_contrib;
        normalization += scale;
    }

    combined_prob_density /= normalization;

    // use log density instead. Adding 1 to the density to start at 0 for density zero, otherwise this is using illegal colorspaces.
    // Then normalizing to maximum.
    let log_combined_prob_density = log(1 + combined_prob_density) / log(2.0);

    return vec4(vec3(0, log_combined_prob_density, 0), 1.0);
}
