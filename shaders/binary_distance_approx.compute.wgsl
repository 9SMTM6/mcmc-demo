#import "canvas_ndc_conversion.wgsl";
#import "binary_distance_approx.wgsl";

@group(1) @binding(40)
var<storage, read_write> output_buffer: array<f32>;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ndc = canvas_coord_to_ndc_int(global_id.xy);
    let index = u32(resolution_info.resolution.x) + u32(resolution_info.resolution.y) * u32(global_id.y);
    output_buffer[index] = binary_distance_approx(ndc);
}
