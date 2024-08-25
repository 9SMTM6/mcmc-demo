#import "canvas_ndc_conversion.wgsl";

@group(1) @binding(40)
var<storage, read_write> compute_output: array<f32>;

fn to_buffer_idx(pixel_loc: vec2<u32>) -> u32 {
    return u32(resolution_info.resolution.x) * pixel_loc.y + pixel_loc.x;
}
