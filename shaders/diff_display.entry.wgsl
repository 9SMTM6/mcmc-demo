#import constants::PI;
#import resolution_uniform_bind::resolution_info;
#import helpers::{canvas_coord_to_ndc, percentage_logscaled};
#import multimodal_gaussian::{gauss_bases, calc_gaussian_density}
#import fullscreen_quad;

@fragment
fn fs_main(@builtin(position) canvas_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_device_coords = canvas_coord_to_ndc(canvas_coords.xy, resolution_info.resolution);

    let combined_prob_density = calc_gaussian_density(normalized_device_coords);

    return vec4(vec3(0, combined_prob_density, 0), 1.0);
}
