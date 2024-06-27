use std::sync::atomic::{AtomicBool, Ordering};

use egui::{self, Pos2, Shadow, Vec2};

use crate::{
    settings::{self, Settings},
    visualizations::{self, CanvasPainter, MultiModalGaussian},
};

#[cfg_attr(feature="persistence", 
    // We derive Deserialize/Serialize so we can persist app state on shutdown.
    derive(serde::Deserialize, serde::Serialize),
    // if we add new fields, give them default values when deserializing old state
    serde(default),
)]
#[derive(Default)]
pub struct TemplateApp {
    gaussian: MultiModalGaussian,
    settings: settings::Settings,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = Self::get_state(cc);
        state
            .gaussian
            .init_gaussian_pipeline(cc.wgpu_render_state.as_ref().unwrap());
        state
    }

    pub fn get_state(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

static FIRST_RENDER: AtomicBool = AtomicBool::new(true);

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            // set some styling. I wasn't able to find a spot to do it earlier,
            // while 'just' doing this here leads to reduction every render, leading to full transparency.
            // rust is rust, so I protect with this super complex atomic...
            // I wonder whether I actually did it right.
            let first_render =
                FIRST_RENDER.compare_exchange(true, false, Ordering::SeqCst, Ordering::Relaxed);
            if let Ok(true) = first_render {
                ctx.style_mut(|style| {
                    let visuals = &mut style.visuals;
                    for fill_color in [
                        &mut visuals.window_fill,
                        // &mut visuals.widgets.noninteractive.bg_fill,
                        // &mut visuals.widgets.noninteractive.weak_bg_fill,
                        // &mut visuals.widgets.active.weak_bg_fill,
                        &mut visuals.widgets.open.weak_bg_fill,
                        &mut visuals.extreme_bg_color,
                    ] {
                        *fill_color = fill_color.gamma_multiply(0.40);
                    }
                    visuals.window_shadow = Shadow::NONE;
                });
            }
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(debug_assertions)]
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        self.settings.draw(ctx);

        egui::CentralPanel::default()
            // remove margins
            .frame(Default::default())
            .show(ctx, |ui| {
                egui::Frame::canvas(ui.style())
                    // remove margins here too
                    .inner_margin(egui::Margin::default())
                    .outer_margin(egui::Margin::default())
                    .show(ui, |ui| {
                        let px_size = ui.available_size();
                        let (rect, _response) =
                            ui.allocate_exact_size(px_size, egui::Sense::click_and_drag());
                        // let rect = egui::Rect::from_min_size(ui.cursor().min, px_size) / ctx.pixels_per_point();
                        // last painted element wins.
                        let painter = ui.painter();
                        let current_spot: Pos2 = [300.0, 400.0].into();
                        self.gaussian.paint(painter, rect * ctx.pixels_per_point());
                        visualizations::Arrow::new(current_spot, [100.0, 100.0])
                            .paint(painter, rect);
                        visualizations::PredictionVariance::new(current_spot, 200.0)
                            .paint(painter, rect);
                        visualizations::SamplingPoint::new(current_spot, 0.65).paint(painter, rect);
                        if self.settings == Settings::EditDistribution {
                            // draw centers of gaussians
                            for ele in self.gaussian.gaussians.iter_mut() {
                                let painter = ui.painter();
                                painter.circle_filled(
                                    ndc_to_canvas_coord(ele.position.into(), rect),
                                    5.0,
                                    egui::Color32::RED,
                                );
                            }

                            // response.

                            // do dragndrop with the centers of the gaussians aka
                            // (this is ironically from the wgpu render demo, I originally stripped it out):
                            // let (rect, response) =
                            //     ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

                            // self.angle += response.drag_motion().x * 0.01;
                            // ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                            //     rect,
                            //     CustomTriangleCallback { angle: self.angle },
                            // ));
                        }
                    });
            });
    }
}

fn ndc_to_canvas_coord(ndc: egui::Pos2, canvas_rect: egui::Rect) -> egui::Pos2 {
    ((ndc.to_vec2() + egui::Vec2::from([1.0, 1.0])) / 2.0 * canvas_rect.size().max_elem()).to_pos2()
}

#[allow(dead_code)]
fn canvas_coord_to_ndc(canvas_coord: egui::Pos2, canvas_rect: egui::Rect) -> egui::Pos2 {
    (canvas_coord / canvas_rect.size().max_elem()) * 2.0 - Vec2::splat(1.0)
}

#[cfg(test)]
mod test {
    use egui::Pos2;

    use super::{canvas_coord_to_ndc, ndc_to_canvas_coord};

    fn close_enough(Pos2 { x: x_1, y: y_1 }: Pos2, Pos2 { x: x_2, y: y_2 }: Pos2) -> bool {
        ((x_1 - x_2) < f32::EPSILON) && ((y_1 - y_2) < f32::EPSILON)
    }

    #[test]
    fn invert_each_other() {
        let rect = egui::Rect {
            min: [0.0, 0.0].into(),
            max: [1920.0, 1080.0].into(),
        };

        let start_canvas_coord = egui::Pos2::from([450.0, 670.0]);
        let start_ndc = [-0.634, 0.232].into();

        assert_eq!(
            ndc_to_canvas_coord(canvas_coord_to_ndc(start_canvas_coord, rect), rect),
            start_canvas_coord
        );
        assert!(close_enough(
            canvas_coord_to_ndc(ndc_to_canvas_coord(start_ndc, rect), rect),
            start_ndc
        ));
    }
}
