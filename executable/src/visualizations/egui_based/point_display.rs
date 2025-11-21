use egui::{Color32, Pos2};
use macros::cfg_persistence_derive;

use crate::{
    app::ndc_to_canvas_coord,
    simulation::random_walk_metropolis_hastings::{AcceptRecord, Rwmh},
    visualizations::{self, CanvasPainter},
};

#[cfg_persistence_derive]
pub struct SamplePointVisualizer {
    pub min_opacity: f32,
    pub point_radius: f32,
    pub accepted_point_color: Color32,
    pub rejected_point_color: Option<Color32>,
}

impl Default for SamplePointVisualizer {
    fn default() -> Self {
        Self {
            accepted_point_color: Color32::RED,
            min_opacity: 0.3,
            point_radius: 3.0,
            rejected_point_color: None,
        }
    }
}

impl SamplePointVisualizer {
    pub fn paint(&self, painter: &egui::Painter, rect: egui::Rect, algo: &Rwmh) {
        for &AcceptRecord {
            position,
            remain_count,
            ..
        } in algo.history.iter().skip(1)
        // skipping the first empty element I added to avoid WebGPU bind exceptions (see shader for explanation!)
        {
            let canvas_loc = ndc_to_canvas_coord(Pos2::new(position[0], position[1]), rect.size());
            let normalized_lifespan =
                (remain_count + 1) as f32 / (algo.max_remain_count + 1) as f32;
            // with the above there may be a point where most accepted points are very close to 0, this seeks to always have them above a certain threshold.
            let log_lifespan = f32::log2(1.0 + normalized_lifespan) / f32::log2(2.0);
            let point_opacity = log_lifespan * (1.0 - self.min_opacity) + self.min_opacity;
            painter.circle_filled(
                canvas_loc,
                self.point_radius,
                self.accepted_point_color.gamma_multiply(point_opacity),
            );
        }
        if let Some(color) = self.rejected_point_color {
            for step in algo.rejected_history.iter() {
                let step = ndc_to_canvas_coord(Pos2::new(step.x, step.y), rect.size());
                painter.circle_filled(
                    step,
                    self.point_radius,
                    color.gamma_multiply(self.min_opacity),
                );
            }
        }
        #[expect(unused, reason = "I want this to compile")]
        if false {
            todo!();
            let current_spot: Pos2 = [300.0, 400.0].into();
            visualizations::Arrow::new(current_spot, [100.0, 100.0]).paint(painter, rect);
            visualizations::PredictionVariance::new(current_spot, 200.0).paint(painter, rect);
            visualizations::SamplingPoint::new(current_spot, 0.65).paint(painter, rect);
        }
    }
}
