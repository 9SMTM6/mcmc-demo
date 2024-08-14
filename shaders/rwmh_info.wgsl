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

@group(2) @binding(0)
var<storage, read> accepted: array<RWMHAcceptRecord>;

// @group(2) @binding(1)
// var<storage, read> rejected: array<RWMHRejectRecord>;

@group(2) @binding(1)
var<uniform> count_info: RWMHCountInfo;
