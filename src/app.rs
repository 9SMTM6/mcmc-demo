use egui::{self, Shadow, Vec2};

use crate::{
    settings::{self, Settings},
    simulation::{
        random_walk_metropolis_hastings::{ProgressMode, Rwmh},
        SRngGaussianIter, SRngPercIter,
    },
    target_distributions::multimodal_gaussian::MultiModalGaussian,
    visualizations::{
        egui_based::point_display::PointDisplay,
        shader_based::{diff_display::DiffDisplay, multimodal_gaussian::MultiModalGaussianDisplay},
    },
};

#[cfg_attr(feature="persistence",
    // We derive Deserialize/Serialize so we can persist app state on shutdown.
    derive(serde::Deserialize, serde::Serialize),
    // if we add new fields, give them default values when deserializing old state
    serde(default),
)]
pub struct TemplateApp {
    algo: Rwmh,
    drawer: PointDisplay,
    target_distr: MultiModalGaussian,
    #[allow(dead_code)]
    target_distr_render: Option<MultiModalGaussianDisplay>,
    #[allow(dead_code)]
    diff_render: Option<DiffDisplay>,
    settings: settings::Settings,
    gaussian_distr_iter: SRngGaussianIter<rand_pcg::Pcg32>,
    uniform_distr_iter: SRngPercIter<rand_pcg::Pcg32>,
    backend_panel: super::profile::backend_panel::BackendPanel,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            algo: Default::default(),
            drawer: Default::default(),
            target_distr: Default::default(),
            target_distr_render: None,
            diff_render: None,
            settings: Default::default(),
            gaussian_distr_iter: SRngGaussianIter::<rand_pcg::Pcg32>::new([42; 16]),
            uniform_distr_iter: SRngPercIter::<rand_pcg::Pcg32>::new([42; 16]),
            backend_panel: Default::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = Self::get_state(cc);
        // TODO: this doesnt work as intended...
        // assert!(state.target_distr_render.());
        Self {
            target_distr_render: Some(MultiModalGaussianDisplay::init_gaussian_pipeline(
                &state.target_distr,
                cc.wgpu_render_state.as_ref().unwrap(),
            )),
            ..state
        }
    }

    fn backend_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // The backend-panel can be toggled on/off.
        // We show a little animation when the user switches it.
        let is_open = self.backend_panel.open || ctx.memory(|mem| mem.everything_is_visible());

        egui::SidePanel::left("backend_panel")
            .resizable(false)
            .show_animated(ctx, is_open, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("💻 Backend");
                });

                ui.separator();
                self.backend_panel_contents(ui, frame);
            });
    }

    fn backend_panel_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.backend_panel.ui(ui, frame);

        ui.separator();

        ui.horizontal(|ui| {
            if ui
                .button("Reset egui")
                .on_hover_text("Forget scroll, positions, sizes etc")
                .clicked()
            {
                ui.ctx().memory_mut(|mem| *mem = Default::default());
                ui.close_menu();
            }
        });
    }

    pub fn get_state(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        cc.egui_ctx.style_mut(|style| {
            let visuals = &mut style.visuals;
            // for fill_color in [
            //     &mut visuals.window_fill,
            //     // &mut visuals.widgets.noninteractive.bg_fill,
            //     // &mut visuals.widgets.noninteractive.weak_bg_fill,
            //     // &mut visuals.widgets.active.weak_bg_fill,
            //     &mut visuals.widgets.open.weak_bg_fill,
            //     &mut visuals.extreme_bg_color,
            // ] {
            //     *fill_color = fill_color.gamma_multiply(0.40);
            // }
            visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Reset State").clicked() {
                    *self = Default::default();
                }
                ui.toggle_value(&mut self.backend_panel.open, "Backend");
                egui::warn_if_debug_build(ui);
            })
        });

        self.backend_panel.update(ctx, frame);

        self.backend_panel(ctx, frame);

        self.backend_panel.end_of_frame(ctx);

        #[allow(clippy::collapsible_else_if)]
        egui::Window::new("Simulation").show(ctx, |ui| {
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
            let ProgressMode::Batched { ref mut size } = self.algo.params.progress_mode;
            ui.add(
                // egui::DragValue::new(&mut laaa)
                //     .range(range),
                unsafe {
                    egui::Slider::new(
                        size.get_inner_mut(),
                        // TODO: Increase limit of... storage buffer size or whatever to maximum allowed,
                        // use that maximum here to determine slider maximum, by determining how much space is left, roughly.
                        1..=(usize::MAX / usize::MAX.ilog2() as usize),
                    )
                }
                .logarithmic(true)
                .text("batchsize"),
            );
            if ui.button("Batch step").clicked() {
                for _ in 0..size.get_inner() {
                    self.algo.step(
                        &self.target_distr,
                        &mut self.gaussian_distr_iter,
                        &mut self.uniform_distr_iter,
                    )
                }
                // self.algo.step(&self.gaussian, , accept_rng)
            }
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
                        // TODO: consider moving Settings::Edit* state into `ui.data_mut()` or similar.
                        // ui.data(|map| {
                        //     map.get_temp(id)
                        //     map.insert_temp(id, value)
                        // });
                        // see: https://github.com/emilk/egui/blob/37b1e1504db14697c39ce1c3bb5e58f4f2b819bf/crates/egui_demo_lib/src/demo/password.rs#L46
                        let px_size = ui.available_size();
                        let (rect, response) =
                            ui.allocate_exact_size(px_size, egui::Sense::hover());
                        // last painted element wins.
                        let painter = ui.painter();
                        // TODO: this initialization is still completely screwed up.
                        // Now it crashes.
                        // Why, oh why, does there not seem to be a proper way to manage these wgpu resources in an egui app?
                        self.target_distr_render
                            .as_ref()
                            .unwrap_or(&MultiModalGaussianDisplay::init_gaussian_pipeline(
                                &self.target_distr,
                                frame.wgpu_render_state().unwrap(),
                            ))
                            .paint(&self.target_distr, painter, rect * ctx.pixels_per_point());
                        // self.diff_render
                        //     .as_ref()
                        //     .unwrap_or(&DiffDisplay::init_pipeline(
                        //         0.1,
                        //         &self.target_distr,
                        //         &self.algo,
                        //         _frame.wgpu_render_state().unwrap(),
                        //     ))
                        //     .paint(
                        //         painter,
                        //         rect * ctx.pixels_per_point(),
                        //         &self.algo,
                        //         &self.target_distr,
                        //     );

                        self.drawer.paint(painter, rect, &self.algo);
                        if let Settings::EditDistribution(_) = self.settings {
                            // // draw centers of gaussians
                            for (idx, ele) in self.target_distr.gaussians.iter_mut().enumerate() {
                                let pos = ndc_to_canvas_coord(ele.position.into(), rect.size());
                                const CIRCLE_SIZE: f32 = 5.0;
                                let pos_sense_rect =
                                    egui::Rect::from_center_size(pos, Vec2::splat(CIRCLE_SIZE));
                                let mut pos_resp = ui
                                    .interact(
                                        pos_sense_rect,
                                        response.id.with(idx),
                                        egui::Sense::drag(),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::Grab);
                                if pos_resp.dragged() {
                                    pos_resp = pos_resp
                                        .on_hover_and_drag_cursor(egui::CursorIcon::Grabbing);
                                }
                                // .on_hover_and_drag_cursor(egui::CursorIcon::Grabbing);
                                let pos = rect.clamp(pos + pos_resp.drag_delta());

                                let ndc_pos = canvas_coord_to_ndc(pos, rect.size());

                                ele.position[0] = ndc_pos.x;
                                ele.position[1] = ndc_pos.y;

                                let pos_active =
                                    pos_resp.clicked() || pos_resp.dragged() || pos_resp.hovered();

                                painter.circle_stroke(
                                    pos,
                                    CIRCLE_SIZE,
                                    egui::Stroke {
                                        color: egui::Color32::RED.gamma_multiply(if pos_active {
                                            1.0
                                        } else {
                                            0.9
                                        }),
                                        width: if pos_active { 2.0 } else { 1.0 },
                                    },
                                );
                            }
                        }
                    });
            });
    }
}

pub fn ndc_to_canvas_coord(ndc: egui::Pos2, canvas_size: egui::Vec2) -> egui::Pos2 {
    ((ndc.to_vec2() + egui::Vec2::splat(1.0)) / 2.0 * canvas_size.max_elem()).to_pos2()
}

#[allow(dead_code)]
pub fn canvas_coord_to_ndc(canvas_coord: egui::Pos2, canvas_rect: egui::Vec2) -> egui::Pos2 {
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
