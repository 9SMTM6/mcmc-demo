#import constants::PI;
#import resolution_uniform_bind::resolution_info;
#import fullscreen_quad;

// What I want to have:
// @group(1) @binding(0)
// var<storage, read> gauss_bases: array<NormalDistribution>;

// What I have to use because of webgl backcompat:
@group(1) @binding(0)
var gauss_bases: texture_1d<f32>;

fn canvas_coord_to_ndc(canvas_coord: vec2<f32>, canvas_res: vec2<f32>) -> vec2<f32> {
    return (canvas_coord / max(canvas_res.x, canvas_res.y)) * 2.0 - 1.0;
}

@fragment
fn fs_main(@builtin(position) canvas_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let normalized_device_coords = canvas_coord_to_ndc(canvas_coords.xy, resolution_info.resolution);

    var combined_prob_density = 0.0;

    var normalization = 0.0;

    // for (var i = 0u; i < arrayLength(&gauss_bases); i+=1u) {
    for (var i = 0u; i < textureDimensions(gauss_bases); i+=1u) {
        // let el = gauss_bases[i];
        let texel = textureLoad(gauss_bases, i, 0);

        // let position = el.position;
        // let variance = el.variance;
        // let scale = el.scale;

        // since we send the data with bytemuck converted
        // NormalDistribution, we interpret the fields in that order.
        let position = vec2(texel.r, texel.g);
        let variance = texel.b;
        let scale = texel.a;

        // let position = vec2(bitcast<f32>(texel.r), bitcast<f32>(texel.g));
        // let variance = bitcast<f32>(texel.b);
        // let scale = bitcast<f32>(texel.a);

        // let position = vec2(f32(texel.r), f32(texel.g));
        // let variance = f32(texel.b);
        // let scale = f32(texel.a);


        // for now we calcualte this here, we might test later if this is better or worse than calculating it once on the cpu and delivering it on each render.
        let gauss_normalize = inverseSqrt(2 * PI * variance);
        let sq_dist = pow(distance(normalized_device_coords, position), 2.0);

        let prob_contrib = gauss_normalize * exp(-sq_dist / (2 * variance));
        combined_prob_density+= scale * prob_contrib;
        normalization += scale;
    }

    combined_prob_density /= normalization;

    // use log density instead. 
    // Adding 1 to the density to start at 0 for density zero, otherwise this is using illegal colorspaces.
    // Then normalizing to maximum.
    let log_combined_prob_density = log(1 + combined_prob_density) / log(2.0);

    return vec4(vec3(0, log_combined_prob_density, 0), 1.0);
}
