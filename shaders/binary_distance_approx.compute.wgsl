#import "canvas_ndc_override.wgsl";
#import "binary_distance_approx.wgsl";

@group(0) @binding(0)
var<storage, read_write> output_buffer: array<f32>;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ndc = canvas_coord_to_ndc_override_int(global_id.xy);
    let index = u32(resolution_width) + u32(resolution_height) * u32(global_id.y);
    output_buffer[index] = binary_distance_approx(ndc);
}
