use egui::{self, Pos2, Shadow, Vec2};

use crate::{
    settings::{self, DistrEditKind, Settings},
    shaders::types::NormalDistribution,
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

        cc.egui_ctx.style_mut(|style| {
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

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Reset State").clicked() {
                    *self = Default::default();
                }
                egui::warn_if_debug_build(ui);
            })
        });

        #[allow(clippy::collapsible_else_if)]
        egui::Window::new("Settings").show(ctx, |ui| {
            if matches!(self.settings, Settings::EditDistribution(_)) {
                if ui.button("Stop Editing Distribution").clicked() {
                    self.settings = Settings::Default;
                }
                ui.add_enabled(false, egui::Button::new("Add Element"))
                    .on_hover_text("TODO");
            } else {
                if ui.button("Edit Distribution").clicked() {
                    self.settings = Settings::EditDistribution(settings::DistrEditKind::Stateless);
                };
            };
            // egui::global_dark_light_mode_buttons(ui);
        });

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
                        // last painted element wins.
                        let painter = ui.painter();
                        let current_spot: Pos2 = [300.0, 400.0].into();
                        self.gaussian.paint(painter, rect * ctx.pixels_per_point());
                        visualizations::Arrow::new(current_spot, [100.0, 100.0])
                            .paint(painter, rect);
                        visualizations::PredictionVariance::new(current_spot, 200.0)
                            .paint(painter, rect);
                        visualizations::SamplingPoint::new(current_spot, 0.65).paint(painter, rect);
                        if let Settings::EditDistribution(ref mut distr_edit_kind) = self.settings {
                            // dunno where this is placed, which coordinate system this uses etc.
                            // But when combined with sensing a drag_and_drop this SHOULD provide me with enough info to find
                            // the gauss center (if any) that drags correspond to.
                            let map_to_ndc = |loc| canvas_coord_to_ndc(loc, rect.size());
                            let (start_loc, current_loc) = ui.input(|input_state| {
                                (
                                    input_state.pointer.press_origin().map(map_to_ndc),
                                    input_state.pointer.interact_pos().map(map_to_ndc),
                                )
                            });

                            if _response.drag_started() {
                                let el =
                                    self.gaussian.gaussians.iter().enumerate().find(|(_, el)| {
                                        // todo: move to pixels here, precisely the size of the centers I draw, or something proportional to that.
                                        (start_loc.unwrap().to_vec2() - Vec2::from(el.position))
                                            .length()
                                            < 0.03
                                    });
                                if let Some((
                                    idx,
                                    NormalDistribution {
                                        position: orig_location,
                                        ..
                                    },
                                )) = el
                                {
                                    *distr_edit_kind = DistrEditKind::MoveCenter {
                                        idx,
                                        orig_location: *orig_location,
                                    }
                                }
                            }

                            if _response.drag_stopped() {
                                *distr_edit_kind = DistrEditKind::Stateless;
                            };

                            if let DistrEditKind::MoveCenter { idx, orig_location } =
                                *distr_edit_kind
                            {
                                let new_pos = Pos2::from(orig_location)
                                    + if _response.dragged() {
                                        current_loc.unwrap().to_vec2()
                                            - start_loc.unwrap().to_vec2()
                                    } else {
                                        Vec2::splat(0.0)
                                    };
                                self.gaussian.gaussians[idx].position = new_pos.into();
                            }

                            // draw centers of gaussians
                            for ele in self.gaussian.gaussians.iter_mut() {
                                painter.circle_filled(
                                    ndc_to_canvas_coord(ele.position.into(), rect.size()),
                                    5.0,
                                    egui::Color32::RED,
                                );
                            }
                        }
                    });
            });
    }
}

fn ndc_to_canvas_coord(ndc: egui::Pos2, canvas_size: egui::Vec2) -> egui::Pos2 {
    ((ndc.to_vec2() + egui::Vec2::splat(1.0)) / 2.0 * canvas_size.max_elem()).to_pos2()
}

#[allow(dead_code)]
fn canvas_coord_to_ndc(canvas_coord: egui::Pos2, canvas_rect: egui::Vec2) -> egui::Pos2 {
    (canvas_coord / canvas_rect.max_elem()) * 2.0 - Vec2::splat(1.0)
}

#[cfg(test)]
mod test {
    use egui::Pos2;

    use super::{canvas_coord_to_ndc, ndc_to_canvas_coord};

    fn close_enough(Pos2 { x: x_1, y: y_1 }: Pos2, Pos2 { x: x_2, y: y_2 }: Pos2) -> bool {
        ((x_1 - x_2).abs() < f32::EPSILON) && ((y_1 - y_2).abs() < f32::EPSILON)
    }

    #[test]
    fn invert_each_other() {
        let rect = egui::Rect {
            min: [0.0, 0.0].into(),
            max: [1920.0, 1080.0].into(),
        };

        let size = rect.size();

        let start_canvas_coord = egui::Pos2::from([450.0, 670.0]);
        let start_ndc = [-0.634, 0.232].into();

        assert_eq!(
            ndc_to_canvas_coord(canvas_coord_to_ndc(start_canvas_coord, size), size),
            start_canvas_coord
        );
        assert!(close_enough(
            canvas_coord_to_ndc(ndc_to_canvas_coord(start_ndc, size), size),
            start_ndc
        ));
    }
}
