struct ResolutionInfo {
    resolution: vec2<f32>,
    // See corresponding bindinggroup for reason
    _pad: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> resolution_info: ResolutionInfo;

const PI = radians(180.0);

struct NormalDistribution {
    position: vec2<f32>,
    variance: f32,
    // this will lead to a weight in relation to the other normal distributions
    scale: f32,
}

@group(1) @binding(0)
var<uniform> len: u32;

@group(1) @binding(1)
var<uniform> gauss_bases: array<NormalDistribution, len>;

@fragment
fn fs_main(@builtin(position) pixel_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let resolution = resolution_info.resolution;
    let normalized_device_coords = pixel_coords.xy / max(resolution.x, resolution.y) * 2 - 1;

    return vec4(vec3(0, 0.2, 0), 1.0);
}
