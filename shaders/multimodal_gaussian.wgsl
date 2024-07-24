#import constants::PI;
#import helpers::canvas_coord_to_ndc;
#import resolution_uniform_bind::resolution_info;
#import fullscreen_quad;
#import types::NormalDistribution;

@group(1) @binding(0)
var<storage, read> gauss_bases: array<NormalDistribution>;

fn calc_gaussian_density(ndc_coord: vec2<f32>) -> f32 {
    var combined_prob_density = 0.0;

    var normalization = 0.0;

    // TODO: consider how to do log-density display, which is what original does.
    // it does a bunch of math which 
    // 1. takes time to implement and 
    // 2. I'm not sure whether it would be faster to calculate this on the gpu in every shader
    //  and reduce data transfer or to calculate that on the cpu beforehand (and for that to work we also need to pass in the arrays anyways) 

    for (var i = 0u; i < arrayLength(&gauss_bases); i+=1u) {
        let el = gauss_bases[i];

        let scale = el.scale;
        let variance = el.variance;
        let position = el.position;

        // for now we calcualte this here, we might test later if this is better or worse than calculating it once on the cpu and delivering it on each render.
        let gauss_normalize = inverseSqrt(2 * PI * variance);
        let sq_dist = pow(distance(ndc_coord, position), 2.0);

        let prob_contrib = gauss_normalize * exp(-sq_dist / (2 * variance));
        combined_prob_density+= scale * prob_contrib;
        normalization += scale;
    }

    combined_prob_density /= normalization;

    return combined_prob_density;
}

@fragment
fn fs_main(@builtin(position) canvas_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_device_coords = canvas_coord_to_ndc(canvas_coords.xy, resolution_info.resolution);

    let combined_prob_density = calc_gaussian_density(normalized_device_coords);

    // use log density instead. 
    // Adding 1 to the density to start at 0 for density zero, otherwise this is using illegal colorspaces.
    // Then normalizing to maximum.
    let log_combined_prob_density = log(1 + combined_prob_density) / log(2.0);

    return vec4(vec3(0, log_combined_prob_density, 0), 1.0);
}
