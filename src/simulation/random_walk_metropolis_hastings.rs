use egui::{Color32, Pos2};
use rand::{Rng, SeedableRng};

use crate::{
    app::ndc_to_canvas_coord, target_distributions::multimodal_gaussian::MultiModalGaussian,
    visualizations::CanvasPainter,
};

use super::{SRngGaussianIter, SRngPercIter};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub enum AlgoParams {
    GaussianProposal { sigma: f32 },
}

impl Default for AlgoParams {
    fn default() -> Self {
        AlgoParams::GaussianProposal { sigma: 0.5 }
    }
}

// horrible name but I cant think of something better RN.
pub type AlgoVec = na::Vector2<f32>;

impl AlgoParams {
    fn propose(
        &self,
        start_loc: AlgoVec,
        gaussian_rng: &mut SRngGaussianIter<impl Rng + SeedableRng>,
    ) -> AlgoVec {
        let Self::GaussianProposal { sigma } = self;

        let normal_x = start_loc.x + gaussian_rng.unwrapped_next() * (*sigma);
        let normal_y = start_loc.y + gaussian_rng.unwrapped_next() * (*sigma);
        AlgoVec::new(normal_x, normal_y)
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Algo {
    pub accepted: Vec<AlgoVec>,
    pub rejected: Vec<AlgoVec>,
    params: AlgoParams,
}

impl Default for Algo {
    fn default() -> Self {
        Self {
            accepted: vec![AlgoVec::from_element(0.0)],
            rejected: vec![],
            params: AlgoParams::GaussianProposal { sigma: 0.2 },
        }
    }
}

impl Algo {
    pub fn step(
        &mut self,
        target_distr: &MultiModalGaussian,
        gaussian_rng: &mut SRngGaussianIter<impl Rng + SeedableRng>,
        accept_rng: &mut SRngPercIter<impl Rng + SeedableRng>,
    ) {
        let current = *self.accepted.last().unwrap();
        let proposal = self.params.propose(current, gaussian_rng);
        let acceptance_ratio = target_distr.acceptance_ratio(proposal, current);
        let accept = accept_rng.unwrapped_next() <= acceptance_ratio;
        self.accepted.push(if accept { proposal } else { current });
        if !accept {
            self.rejected.push(proposal);
        };
    }
}

impl CanvasPainter for Algo {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        for step in self.accepted.iter() {
            let step = ndc_to_canvas_coord(Pos2::new(step.x, step.y), rect.size());
            painter.circle_filled(step, 3.0, Color32::BLUE);
        }
        for step in self.rejected.iter() {
            let step = ndc_to_canvas_coord(Pos2::new(step.x, step.y), rect.size());
            painter.circle_filled(step, 3.0, Color32::YELLOW);
        }
    }
}
