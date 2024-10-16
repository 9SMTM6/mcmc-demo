use std::f32::consts::PI;

use macros::cfg_persistence_derive;

use crate::simulation::random_walk_metropolis_hastings::AlgoVec;

use crate::visualizations::shader_based::target_distr::NormalDistribution;

#[cfg_persistence_derive]
#[derive(Clone)]
pub struct GaussianTargetDistr {
    pub gaussians: Vec<NormalDistribution>,
}

impl Default for GaussianTargetDistr {
    fn default() -> Self {
        Self {
            gaussians: [
                NormalDistribution {
                    position: [-0.6, -0.8],
                    scale: 0.5,
                    variance: 0.14,
                },
                NormalDistribution {
                    position: [1.0, 0.8],
                    scale: 0.6,
                    variance: 0.2,
                },
                NormalDistribution {
                    position: [0.9, -0.3],
                    scale: 0.4,
                    variance: 0.01,
                },
                NormalDistribution {
                    position: [0.1, 0.0],
                    scale: 0.8,
                    variance: 0.07,
                },
                NormalDistribution {
                    position: [-1.0, 0.5],
                    scale: 1.4,
                    variance: 0.1,
                },
            ]
            .into(),
        }
    }
}

impl GaussianTargetDistr {
    pub fn probability(&self, position: AlgoVec) -> f32 {
        let mut combined_prob_density = 0.0;

        let mut normalization = 0.0;

        for &NormalDistribution {
            position: ref gauss_pos,
            scale,
            variance,
        } in self.gaussians.iter()
        {
            let gauss_pos = AlgoVec::new(gauss_pos[0], gauss_pos[1]);
            let gauss_normalize = 1.0 / f32::sqrt(2.0 * PI * variance);
            let sq_dist = f32::powi(position.metric_distance(&gauss_pos), 2);

            let prob_contrib = gauss_normalize * f32::exp(-sq_dist / (2.0 * variance));
            combined_prob_density += scale * prob_contrib;
            normalization += scale;
        }

        combined_prob_density /= normalization;
        combined_prob_density
    }

    /// this is NOT limited to legal range, cause its really not required.
    pub fn acceptance_ratio(&self, proposal: AlgoVec, current: AlgoVec) -> f32 {
        self.probability(proposal) / self.probability(current)
    }
}
