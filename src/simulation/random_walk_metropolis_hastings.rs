use egui::{Color32, Pos2};
use rand::distributions::Distribution;

use crate::{app::ndc_to_canvas_coord, target_distributions::multimodal_gaussian::MultiModalGaussian, visualizations::CanvasPainter};

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
    fn propose(&self, start_loc: AlgoVec) -> AlgoVec {
        let Self::GaussianProposal { sigma } = self;
        // TODO: dont reconstruct for every proposal.
        // Perhaps simply use one rand_distr::StandardNormal
        let normal_x = rand_distr::Normal::new(start_loc.x, *sigma).unwrap();
        let normal_y = rand_distr::Normal::new(start_loc.y, *sigma).unwrap();
        AlgoVec::new(
            normal_x.sample(&mut rand::thread_rng()),
            normal_y.sample(&mut rand::thread_rng()),
        )
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Algo {
    pub accepted: Vec<AlgoVec>,
    params: AlgoParams,
    target_distr: MultiModalGaussian,
}

impl Default for Algo {
    fn default() -> Self {
        Self {
            accepted: vec![AlgoVec::from_element(0.0)],
            params: AlgoParams::GaussianProposal { sigma: 0.4 },
            target_distr: MultiModalGaussian::default(),
        }
    }
}

impl Algo {
    pub fn step(&mut self) {
        let current = *self.accepted.last().unwrap();
        let proposal = self.params.propose(current);
        let acceptance_ratio = self.target_distr.acceptance_ratio(proposal, current);
        let accept_distr = rand_distr::Uniform::new_inclusive(0.0, 1.0);
        let accept = accept_distr.sample(&mut rand::thread_rng()) <= acceptance_ratio;
        self.accepted.push(if accept {proposal} else {current})
    }
}

impl CanvasPainter for Algo {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        for step in self.accepted.iter() {
            let step = ndc_to_canvas_coord(Pos2::new(step.x, step.y), rect.size());
            painter.circle_filled(step, 3.0, Color32::BLUE);
        }
    }
}
