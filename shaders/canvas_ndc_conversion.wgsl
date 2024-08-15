fn canvas_coord_to_ndc(canvas_coord: vec2<f32>, canvas_res: vec2<f32>) -> vec2<f32> {
    let min_res = min(canvas_res.x, canvas_res.y);
    let center_offset = (canvas_res - vec2(min_res)) / 2.0;
    return ((canvas_coord - center_offset) / min_res) * 2.0 - 1.0;
}
