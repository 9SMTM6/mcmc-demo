use crate::shaders::types::NormalDistribution;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
pub struct MultiModalGaussian {
    pub gaussians: Vec<NormalDistribution>,
}

impl Default for MultiModalGaussian {
    fn default() -> Self {
        Self {
            gaussians: [
                NormalDistribution {
                    position: [-1.0, -1.0],
                    scale: 0.5,
                    variance: 0.14,
                },
                NormalDistribution {
                    position: [0.2, -0.2],
                    scale: 0.6,
                    variance: 0.2,
                },
                NormalDistribution {
                    position: [0.9, -0.3],
                    scale: 0.4,
                    variance: 0.01,
                },
                NormalDistribution {
                    position: [0.1, -0.6],
                    scale: 0.8,
                    variance: 0.4,
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
