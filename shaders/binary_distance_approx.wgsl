#import "rwmh_info.wgsl";

struct DiffDisplayOptions {
    window_radius: f32,
}

// if reintroduced, increase bindgroup id on others again.
// @group(2) @binding(0)
// var<uniform> diff_display_options: DiffDisplayOptions;

fn binary_distance_approx(ndc_coord: vec2<f32>) -> f32 {
    let total_point_count = count_info.total_point_count;

    let window_radius = 0.1;
    // let window_radius = diff_display_options.window_radius;

    var prob_unnorm: u32 = 0;

    // REALLY ugly fix, but I need to start at 1 so that I never submit
    // a zero sized buffer, which otherwise causes WebGPU to refuse the draw call.
    for (var i = 1u; i < arrayLength(&accepted); i+=1u) {
        let el = accepted[i];

        let position = el.position;
        let remain_count = el.remain_count;

        if distance(ndc_coord, position) <= window_radius {
            prob_unnorm += remain_count + 1;
        }
    }
    // count_info.max_remain_count
    return f32(prob_unnorm) / f32(count_info.max_remain_count * 30);
}
