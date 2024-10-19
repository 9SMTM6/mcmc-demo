#import "canvas_ndc_conversion.wgsl";
#import "binary_distance_approx.buffer.wgsl";
#import "binary_distance_approx.wgsl";

@group(1) @binding(40)
var<storage, read_write> compute_output: array<f32>;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ndc = canvas_coord_to_ndc_int(global_id.xy);
    let index = to_buffer_idx(global_id.xy);
    compute_output[index] = binary_distance_approx(ndc);
    // I need some kind of output, if this actually runs, to see the 'success' of the write
    // let screen_width_approx = u32(sqrt(f32(arrayLength(&compute_output))));
    // let array_idx = global_id.y * screen_width_approx + global_id.x;
    // // "stripes" along the x axis, proper shapes would require using more context, but htat context may be the reaosn for the issues 
    // if global_id.x / 20 % 2 != 1 {
    //     compute_output[array_idx] = 1.0;
    // } else {
    //     compute_output[array_idx] = 0.0;
    // }
}
