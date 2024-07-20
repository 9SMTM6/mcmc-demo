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
#[derive(Default, Clone)]
struct AcceptRecord {
    location: AlgoVec,
    remain_count: u32,
}


#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Algo {
    current_loc: AcceptRecord,
    max_remain_count: u32,
    // should be HashMap<AlgoVec, i32> or similar,
    // but this is an issue as the f32 in AlgoVec isnt Eq.
    // So IDK how to do this right.
    accepted: Vec<AcceptRecord>,
    rejected: Vec<AlgoVec>,
    params: AlgoParams,
}

impl Default for Algo {
    fn default() -> Self {
        Self {
            // TODO: make start point configurable
            current_loc: AcceptRecord {
                location: AlgoVec::from_element(0.0),
                ..Default::default()
            },
            max_remain_count: 0,
            accepted: vec![],
            rejected: vec![],
            params: AlgoParams::GaussianProposal { sigma: 0.2 },
        }
    }
}

impl Algo {
    pub fn step(
        &mut self,
        target_distr: &MultiModalGaussian,
        proposal_rng: &mut SRngGaussianIter<impl Rng + SeedableRng>,
        accept_rng: &mut SRngPercIter<impl Rng + SeedableRng>,
    ) {
        let current = &mut self.current_loc;
        let proposal = self.params.propose(current.location, proposal_rng);
        let acceptance_ratio = target_distr.acceptance_ratio(proposal, current.location);
        let accept = accept_rng.unwrapped_next() <= acceptance_ratio;
        // self.current_loc = if accept { proposal } else { current };
        if accept {
            self.accepted.push(self.current_loc.clone());
            self.current_loc = AcceptRecord {
                location: proposal,
                remain_count: 0,
            };
        } else {
            current.remain_count+=1;
            self.max_remain_count = self.max_remain_count.max(current.remain_count);
            self.rejected.push(proposal);
        };
    }
}

const LOWEST_ALPHA: f32 = 0.3;

impl CanvasPainter for Algo {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        for AcceptRecord {location, remain_count} in self.accepted.iter() {
            let canvas_loc = ndc_to_canvas_coord(Pos2::new(location.x, location.y), rect.size());
            let factor = (*remain_count + 1) as f32 / (self.max_remain_count + 1) as f32;
            // with the above there may be a point where most accepted points are very close to 0, this seeks to always have them above a certain threshold.
            let log_factor = f32::log2(1.0 + factor) / f32::log2(2.0);
            let renormalized_factor = log_factor * (1.0 - LOWEST_ALPHA) + LOWEST_ALPHA;
            painter.circle_filled(canvas_loc, 3.0, Color32::RED.gamma_multiply(renormalized_factor));
        }
        for step in self.rejected.iter() {
            let step = ndc_to_canvas_coord(Pos2::new(step.x, step.y), rect.size());
            painter.circle_filled(step, 3.0, Color32::YELLOW.gamma_multiply(LOWEST_ALPHA));
        }
    }
}
