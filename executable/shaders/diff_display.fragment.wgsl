#import "canvas_ndc_conversion.wgsl";
#import "fullscreen_quad.vertex.wgsl";
#import "diff_display.wgsl";
#import "helpers.wgsl";

@fragment
fn fs_main(@builtin(position) canvas_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_device_coords = canvas_coord_to_ndc(canvas_coords.xy);

    let diff = approx_target_diff(normalized_device_coords);

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
