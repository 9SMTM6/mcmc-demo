struct ResolutionInfo {
    resolution: vec2<f32>,
    // See corresponding bindinggroup for reason
    _pad: vec2<f32>,
}

@group(0) @binding(0) 
var<uniform> resolution_info: ResolutionInfo;

// blocked (see readme)
// override resolution_width: u32;
// override resolution_height: u32;

fn canvas_coord_to_ndc(canvas_coord: vec2<f32>) -> vec2<f32> {
    let canvas_res = resolution_info.resolution;
    let min_res = min(canvas_res.x, canvas_res.y);
    let center_offset = (canvas_res - vec2(min_res)) / 2.0;
    return ((canvas_coord - center_offset) / min_res) * 2.0 - 1.0;

    // let canvas_res = resolution_info.resolution;
    // return (canvas_coord / max(resolution_height, resolution_width)) * 2.0 - 1.0;
}
