#import "binary_distance_approx.wgsl";
#import "multimodal_gaussian.wgsl";

fn approx_target_diff(ndc_coord: vec2<f32>) -> f32 {
    let target_density = calc_gaussian_density(ndc_coord);

    let approx_density = binary_distance_approx(ndc_coord);

    return target_density - approx_density;
}
