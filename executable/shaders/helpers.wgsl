fn percentage_logscaled(uniformscaled_perc: f32) -> f32 {
    // Adding 1 to the density to start at 0 for density zero, otherwise this is using illegal colorspaces.
    // Then normalizing to maximum.
    return log(1 + uniformscaled_perc) / log(2.0);
}
