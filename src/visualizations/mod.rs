use egui::{Margin, Pos2};
use egui_expressed::{Arrow, PredictionVariance, SamplingPoint};

mod egui_expressed;
mod shaders;

pub use shaders::{test_fixed_gaussian::FixedGaussian, INITIAL_RENDER_SIZE};

trait CanvasPainter {
    fn paint(&mut self, painter: &egui::Painter, rect: egui::Rect);
}

fn paint_in_marginless_canvas(ui: &mut egui::Ui, canvas_painters: &mut [&mut dyn CanvasPainter]) {
    egui::Frame::canvas(ui.style())
        // remove margins here too
        .inner_margin(Margin::default())
        .outer_margin(Margin::default())
        .show(ui, |ui| {
            let px_size = ui.available_size();
            let rect = egui::Rect::from_min_size(ui.cursor().min, px_size);
            // last painted element wins.
            let painter = ui.painter();
            for canvas_painter in canvas_painters {
                (*canvas_painter).paint(painter, rect);
            }
        });
}

pub fn draw_all(ui: &mut egui::Ui, fixed_gaussian_raii_obj: &mut FixedGaussian) {
    let current_spot: Pos2 = [300.0, 400.0].into();
    let mut canvas_painters = [
        fixed_gaussian_raii_obj as &mut dyn CanvasPainter,
        &mut Arrow::new(current_spot, [100.0, 100.0]),
        &mut PredictionVariance::new(current_spot, 200.0),
        &mut SamplingPoint::new(current_spot, 0.65),
    ];
    paint_in_marginless_canvas(ui, &mut canvas_painters[..])
}
