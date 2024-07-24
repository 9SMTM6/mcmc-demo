pub use egui_based::{Arrow, PredictionVariance, SamplingPoint};

pub mod egui_based;
pub mod shader_based;

pub use shader_based::INITIAL_RENDER_SIZE;

pub trait CanvasPainter {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect);
}
