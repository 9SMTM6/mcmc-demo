fn canvas_coord_to_ndc(canvas_coord: vec2<f32>, canvas_res: vec2<f32>) -> vec2<f32> {
    return (canvas_coord / max(canvas_res.x, canvas_res.y)) * 2.0 - 1.0;
}

fn percentage_logscaled(uniformscaled_perc: f32) -> f32 {
    // Adding 1 to the density to start at 0 for density zero, otherwise this is using illegal colorspaces.
    // Then normalizing to maximum.
    return log(1 + uniformscaled_perc) / log(2.0);
}