pub use egui_expressed::{Arrow, PredictionVariance, SamplingPoint};

mod egui_expressed;
mod shaders;

pub use shaders::{multimodal_gaussian::MultiModalGaussian, INITIAL_RENDER_SIZE};

pub trait CanvasPainter {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect);
}
