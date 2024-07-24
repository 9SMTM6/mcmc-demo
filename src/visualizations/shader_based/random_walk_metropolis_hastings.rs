use egui::{Color32, Pos2};

use crate::{
    app::ndc_to_canvas_coord,
    simulation::random_walk_metropolis_hastings::{AcceptRecord, Algo},
};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct PointDisplay {
    lowest_alpha: f32,
    radius: f32,
    accept_color: Color32,
    reject_display: Option<Color32>,
}

impl Default for PointDisplay {
    fn default() -> Self {
        Self {
            accept_color: Color32::RED,
            lowest_alpha: 0.3,
            radius: 3.0,
            reject_display: Some(Color32::YELLOW),
        }
    }
}

impl PointDisplay {
    pub fn paint(&self, painter: &egui::Painter, rect: egui::Rect, algo: &Algo) {
        for AcceptRecord {
            location,
            remain_count,
        } in algo.history.iter()
        {
            let canvas_loc = ndc_to_canvas_coord(Pos2::new(location.x, location.y), rect.size());
            let factor = (*remain_count + 1) as f32 / (algo.max_remain_count + 1) as f32;
            // with the above there may be a point where most accepted points are very close to 0, this seeks to always have them above a certain threshold.
            let log_factor = f32::log2(1.0 + factor) / f32::log2(2.0);
            let renormalized_factor = log_factor * (1.0 - self.lowest_alpha) + self.lowest_alpha;
            painter.circle_filled(
                canvas_loc,
                self.radius,
                self.accept_color.gamma_multiply(renormalized_factor),
            );
        }
        if let Some(color) = self.reject_display {
            for step in algo.rejected_history.iter() {
                let step = ndc_to_canvas_coord(Pos2::new(step.x, step.y), rect.size());
                painter.circle_filled(step, self.radius, color.gamma_multiply(self.lowest_alpha));
            }
        }
    }
}