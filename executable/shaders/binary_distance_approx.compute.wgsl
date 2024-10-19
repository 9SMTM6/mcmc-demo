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
}
