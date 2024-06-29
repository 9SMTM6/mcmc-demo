// // I want to force the generation of some bindings that would otherwise not be used.
// // This is an entry point that uses them in host-settable ways to force the generation of bindings.
#import types::NormalDistribution;

// struct NormalDistribution {
//     position: vec2<f32>,
//     variance: f32,
//     // this will lead to a weight in relation to the other normal distributions
//     scale: f32,
// }

@group(0) @binding(0)
var<uniform> normal_distr: NormalDistribution;

@vertex
fn fake_main() -> @builtin(position) vec4<f32> {
    return vec4(normal_distr.position, 0.0, 0.0);
}
