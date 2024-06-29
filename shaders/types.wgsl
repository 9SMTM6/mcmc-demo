struct NormalDistribution {
    position: vec2<f32>,
    variance: f32,
    // this will lead to a weight in relation to the other normal distributions
    scale: f32,
}

struct ResolutionInfo {
    resolution: vec2<f32>,
    // See corresponding bindinggroup for reason
    _pad: vec2<f32>,
}
