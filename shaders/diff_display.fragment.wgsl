#import constants::PI;
#import canvas_ndc_conversion::canvas_coord_to_ndc;
#import helpers::percentage_logscaled;
#import multimodal_gaussian::{gauss_bases, calc_gaussian_density}
#import "fullscreen_quad.vertex.wgsl";
#import types::{RWAcceptRecord, RejectRecord};

@group(2) @binding(0)
var<storage, read> accepted: array<RWAcceptRecord>;

// @group(2) @binding(1)
// var<storage, read> rejected: array<RejectRecord>;

@group(2) @binding(1)
var<uniform> max_remain_count: u32;

@fragment
fn fs_main(@builtin(position) canvas_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_device_coords = canvas_coord_to_ndc(canvas_coords.xy);

    let combined_prob_density = calc_gaussian_density(normalized_device_coords);

    return vec4(vec3(0, combined_prob_density, 0), 1.0);
}
