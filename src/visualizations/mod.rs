pub use egui_based::{Arrow, PredictionVariance, SamplingPoint};

mod egui_based;
mod shader_based;

pub use shader_based::INITIAL_RENDER_SIZE;

pub trait CanvasPainter {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect);
}
