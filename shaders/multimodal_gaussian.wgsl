#import constants::PI;
#import helpers::{percentage_logscaled};
#import canvas_ndc_conversion::canvas_coord_to_ndc;
#import "fullscreen_quad.vertex.wgsl";
#import types::NormalDistribution;

@group(1) @binding(0)
var<storage, read> gauss_bases: array<NormalDistribution>;

fn calc_gaussian_density(ndc_coord: vec2<f32>) -> f32 {
    var combined_prob_density = 0.0;

    var normalization = 0.0;

    for (var i = 0u; i < arrayLength(&gauss_bases); i+=1u) {
        let el = gauss_bases[i];

        let scale = el.scale;
        let variance = el.variance;
        let position = el.position;

        // for now we calculate this here, we might test later if this is better or worse than calculating it once on the cpu and delivering it on each render.
        let gauss_normalize = inverseSqrt(2 * PI * variance);
        let sq_dist = pow(distance(ndc_coord, position), 2.0);

        let prob_contrib = gauss_normalize * exp(-sq_dist / (2 * variance));
        combined_prob_density+= scale * prob_contrib;
        normalization += scale;
    }

    combined_prob_density /= normalization;

    return combined_prob_density;
}
