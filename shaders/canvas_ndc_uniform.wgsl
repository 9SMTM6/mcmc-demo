#import "canvas_ndc_conversion.wgsl";

struct ResolutionInfo {
    resolution: vec2<f32>,
    // See corresponding bindinggroup for reason
    _pad: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> resolution_info: ResolutionInfo;

fn canvas_coord_to_ndc_uniform(canvas_coord: vec2<f32>) -> vec2<f32> {
    return canvas_coord_to_ndc(canvas_coord, resolution_info.resolution);
}
