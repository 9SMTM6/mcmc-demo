fn canvas_coord_to_ndc(canvas_coord: vec2<f32>, canvas_res: vec2<f32>) -> vec2<f32> {
    return (canvas_coord / max(canvas_res.x, canvas_res.y)) * 2.0 - 1.0;
}