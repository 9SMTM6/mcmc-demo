struct ResolutionInfo {
    resolution: vec2<f32>,
    // See corresponding bindinggroup for reason
    _pad: vec2<f32>,
}

@group(0) @binding(0) 
var<uniform> resolution_info: ResolutionInfo;
