#import "canvas_ndc_conversion.wgsl";

// TODO: remove again. Its probably not worth the considerable complication.
override resolution_width: f32 = 1920.0;
override resolution_height: f32 = 1080.0;

fn canvas_coord_to_ndc_override_int(canvas_coord: vec2<u32>) -> vec2<f32> {
    return canvas_coord_to_ndc(vec2(f32(canvas_coord.x), f32(canvas_coord.y)), vec2(resolution_width, resolution_height));
}
