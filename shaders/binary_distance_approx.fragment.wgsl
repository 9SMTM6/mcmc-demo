#import "canvas_ndc_conversion.wgsl";
#import "fullscreen_quad.vertex.wgsl";
#import "binary_distance_approx.bufferbinding.wgsl";
#import "multimodal_gaussian.wgsl";
#import "helpers.wgsl";

@fragment
fn fs_main(@builtin(position) canvas_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let index = to_buffer_idx(vec2(u32(canvas_coords.x), u32(canvas_coords.y)));
    let approx_density =  compute_output[index];
    let ndc_coord = canvas_coord_to_ndc(canvas_coords.xy);
    let target_density = calc_gaussian_density(ndc_coord);
    let diff = target_density - approx_density;

    let diff_paint = percentage_logscaled(abs(diff));

    var color = vec3(0.0);

    if sign(diff) == 1 {
        color[1] = diff_paint;
    } else {
        color[2] = sqrt(diff_paint);
        color[0] = sqrt(diff_paint);
    }

    return vec4(color, 1.0);
}
