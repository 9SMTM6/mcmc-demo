use egui::{Color32, Pos2};

use crate::{
    app::ndc_to_canvas_coord,
    simulation::random_walk_metropolis_hastings::{AcceptRecord, Rwmh},
    visualizations::{self, CanvasPainter},
};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct PointDisplay {
    pub lowest_alpha: f32,
    pub radius: f32,
    pub accept_color: Color32,
    pub reject_display: Option<Color32>,
}

impl Default for PointDisplay {
    fn default() -> Self {
        Self {
            accept_color: Color32::RED,
            lowest_alpha: 0.3,
            radius: 3.0,
            reject_display: None,
        }
    }
}

impl PointDisplay {
    pub fn paint(&self, painter: &egui::Painter, rect: egui::Rect, algo: &Rwmh) {
        for &AcceptRecord {
            position,
            remain_count,
            ..
        } in algo.history.iter().skip(1)
        // skipping the first empty element I added to avoid WebGPU bind exceptions (see shader for explanation!)
        {
            let canvas_loc = ndc_to_canvas_coord(Pos2::new(position[0], position[1]), rect.size());
            let factor = (remain_count + 1) as f32 / (algo.max_remain_count + 1) as f32;
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
        #[expect(unreachable_code)]
        if false {
            todo!();
            let current_spot: Pos2 = [300.0, 400.0].into();
            visualizations::Arrow::new(current_spot, [100.0, 100.0]).paint(painter, rect);
            visualizations::PredictionVariance::new(current_spot, 200.0).paint(painter, rect);
            visualizations::SamplingPoint::new(current_spot, 0.65).paint(painter, rect);
        }
    }
}
