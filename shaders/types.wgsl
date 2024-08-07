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

struct RWMHAcceptRecord {
    position: vec2<f32>,
    remain_count: u32,
    _pad: array<u32, 1>,
}

// has issues.
// 1. generates code that doesnt support serde
// 2. max_remain_count causes some (alignment?) checks to fail 
// struct RW_MCMC_Accepted {
//     max_remain_count: u32,
//     history: array<RWAcceptRecord>,
// }

struct RWMHRejectRecord {
    location: vec2<f32>,
}

struct RWMHCountInfo {
    max_remain_count: u32,
    total_point_count: u32,
}

struct DiffDisplayOptions {
    window_radius: f32,
}