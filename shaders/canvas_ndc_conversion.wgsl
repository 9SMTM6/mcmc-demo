#import types::ResolutionInfo;

@group(0) @binding(0) 
var<uniform> resolution_info: ResolutionInfo;

fn canvas_coord_to_ndc(canvas_coord: vec2<f32>) -> vec2<f32> {
    let canvas_res = resolution_info.resolution;
    return (canvas_coord / max(canvas_res.x, canvas_res.y)) * 2.0 - 1.0;
}
