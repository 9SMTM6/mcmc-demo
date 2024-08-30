#import "canvas_ndc_conversion.wgsl";

fn to_buffer_idx(pixel_loc: vec2<u32>) -> u32 {
    return u32(resolution_info.resolution.x) * pixel_loc.y + pixel_loc.x;
}
