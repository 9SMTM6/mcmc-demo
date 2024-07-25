#import constants::PI;
#import canvas_ndc_conversion::canvas_coord_to_ndc;
#import helpers::percentage_logscaled;
#import multimodal_gaussian::{gauss_bases, calc_gaussian_density}
#import "fullscreen_quad.vertex.wgsl";
#import types::{RWAcceptRecord, RWRejectRecord, RWCountInfo, DiffDisplayOptions};

@group(2) @binding(0)
var<uniform> diff_display_options: DiffDisplayOptions;

@group(3) @binding(0)
var<storage, read> accepted: array<RWAcceptRecord>;

// @group(3) @binding(1)
// var<storage, read> rejected: array<RWRejectRecord>;

@group(3) @binding(1)
var<uniform> count_info: RWCountInfo;

fn calc_approx_density(ndc_coord: vec2<f32>) -> f32 {
    let total_point_count = count_info.total_point_count;

    let window_radius = diff_display_options.window_radius;

    var prob_unnorm: u32 = 0;

    for (var i = 0u; i < arrayLength(&accepted); i+=1u) {
        let el = accepted[i];

        let position = el.position;
        let remain_count = el.remain_count;

        if distance(ndc_coord, position) <= window_radius {
            prob_unnorm += remain_count + 1;
        }
    }
    return f32(prob_unnorm) / f32(total_point_count);
}

@fragment
fn fs_main(@builtin(position) canvas_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_device_coords = canvas_coord_to_ndc(canvas_coords.xy);

    let target_density = calc_gaussian_density(normalized_device_coords);

    let approx_density = calc_approx_density(normalized_device_coords);

    let diff = target_density - approx_density;

    return vec4(vec3(0, abs(diff), 0), 1.0);
}
