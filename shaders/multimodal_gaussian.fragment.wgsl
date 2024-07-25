#import constants::PI;
#import multimodal_gaussian::calc_gaussian_density;
#import helpers::percentage_logscaled;
#import canvas_ndc_conversion::canvas_coord_to_ndc;
#import "fullscreen_quad.vertex.wgsl";
#import types::NormalDistribution;

@fragment
fn fs_main(@builtin(position) canvas_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_device_coords = canvas_coord_to_ndc(canvas_coords.xy);

    let combined_prob_density = calc_gaussian_density(normalized_device_coords);

    return vec4(vec3(0, percentage_logscaled(combined_prob_density), 0), 1.0);
}
