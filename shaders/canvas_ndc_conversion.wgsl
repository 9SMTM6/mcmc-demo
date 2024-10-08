struct ResolutionInfo {
    resolution: vec2<f32>,
    // See corresponding bindinggroup for reason
    _pad: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> resolution_info: ResolutionInfo;

fn canvas_coord_to_ndc(canvas_coord: vec2<f32>) -> vec2<f32> {
    let canvas_res = resolution_info.resolution;
    let min_res = min(canvas_res.x, canvas_res.y);
    let center_offset = (canvas_res - vec2(min_res)) / 2.0;
    return ((canvas_coord - center_offset) / min_res) * 2.0 - 1.0;
}

fn canvas_coord_to_ndc_int(canvas_coord: vec2<u32>) -> vec2<f32> {
    return canvas_coord_to_ndc(vec2(f32(canvas_coord.x), f32(canvas_coord.y)));
}
