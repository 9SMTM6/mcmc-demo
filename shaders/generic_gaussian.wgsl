#import constants::PI;
#import resolution_uniform::resolution_info;
#import fullscreen_quad;

struct NormalDistribution {
    position: vec2<f32>,
    variance: f32,
    // this will lead to a weight in relation to the other normal distributions
    scale: f32,
}

@group(1) @binding(0)
var<storage, read> gauss_bases: array<NormalDistribution>;

@fragment
fn fs_main(@builtin(position) pixel_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let resolution = resolution_info.resolution;
    let normalized_device_coords = pixel_coords.xy / max(resolution.x, resolution.y) * 2 - 1;

    return vec4(vec3(0, 0.2, 0), 1.0);
}
