#import "canvas_ndc_conversion.wgsl";
#import "binary_distance_approx.bufferbinding.wgsl";
#import "binary_distance_approx.wgsl";

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ndc = canvas_coord_to_ndc_int(global_id.xy);
    let index = to_buffer_idx(global_id.xy);
    compute_output[index] = binary_distance_approx(ndc);
}
