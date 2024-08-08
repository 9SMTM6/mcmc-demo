use std::time::Duration;

use egui::{self, ProgressBar, Shadow, Vec2};

use crate::{
    bg_task::{BgCommunicate, BgTaskHandle, Progress},
    settings::{self, Settings},
    shaders::types::NormalDistribution,
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
pub struct McmcDemo {
    algo: Rwmh,
    drawer: PointDisplay,
    target_distr: MultiModalGaussian,
    #[allow(dead_code)]
    target_distr_render: MultiModalGaussianDisplay,
    #[allow(dead_code)]
    diff_render: DiffDisplay,
    settings: settings::Settings,
    gaussian_distr_iter: SRngGaussianIter<rand_pcg::Pcg32>,
    uniform_distr_iter: SRngPercIter<rand_pcg::Pcg32>,
    #[cfg(feature = "profile")]
    backend_panel: super::profile::backend_panel::BackendPanel,
    #[cfg_attr(feature = "persistence", serde(skip))]
    bg_task: Option<BgTaskHandle<Rwmh>>,
    // bg_tasks: Vec<BgTask<String, String>>,
}

impl Default for McmcDemo {
    fn default() -> Self {
        Self {
            algo: Default::default(),
            drawer: Default::default(),
            target_distr: Default::default(),
            target_distr_render: MultiModalGaussianDisplay {},
            diff_render: DiffDisplay { window_radius: 5.0 },
            settings: Default::default(),
            gaussian_distr_iter: SRngGaussianIter::<rand_pcg::Pcg32>::new([42; 16]),
            uniform_distr_iter: SRngPercIter::<rand_pcg::Pcg32>::new([42; 16]),
            #[cfg(feature = "profile")]
            backend_panel: Default::default(),
            bg_task: None,
            // bg_tasks: vec![]
        }
    }
}

impl McmcDemo {
    /// Called once before the first frame.
    #[allow(clippy::missing_panics_doc, reason = "only used once")]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

        let state = Self::get_state(cc);
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("Compiling with WGPU enabled");
        MultiModalGaussianDisplay::init_gaussian_pipeline(render_state);
        DiffDisplay::init_pipeline(render_state);
        state
    }

    pub fn get_state(
        #[allow(
            unused_variables,
            reason = "Conditional compilation makes this sometimes unused"
        )]
        cc: &eframe::CreationContext<'_>,
    ) -> Self {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

#[cfg(feature = "profile")]
impl McmcDemo {
    fn backend_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // The backend-panel can be toggled on/off.
        // We show a little animation when the user switches it.
        let is_open = self.backend_panel.open || ctx.memory(|mem| mem.everything_is_visible());

        egui::SidePanel::left("backend_panel")
            .resizable(false)
            .show_animated(ctx, is_open, |ui| {
                #[allow(clippy::shadow_unrelated, reason = "false positive, is related.")]
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

        #[allow(clippy::shadow_unrelated, reason = "false positive, is related.")]
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
}

impl eframe::App for McmcDemo {
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // not sure if that'd be a good idea
        // Wait for all threads, otherwise program exits before threads finish execution.
        // We can't do blocking join on wasm main thread though, but the browser window will continue running.
        // #[cfg(not(target_arch = "wasm32"))]
        // self.background_thread.take().map(thread::JoinHandle::join);
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(
        &mut self,
        ctx: &egui::Context,
        #[allow(
            unused_variables,
            reason = "Conditional compilation makes this sometimes unused"
        )]
        frame: &mut eframe::Frame,
    ) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            #[allow(clippy::shadow_unrelated, reason = "false positive, is related.")]
            ui.horizontal(|ui| {
                if ui.button("Reset State").clicked() {
                    *self = Default::default();
                }
                #[cfg(feature = "profile")]
                ui.toggle_value(&mut self.backend_panel.open, "Backend");
                egui::warn_if_debug_build(ui);
            });
        });

        #[cfg(feature = "profile")]
        {
            self.backend_panel.update(ctx, frame);

            self.backend_panel(ctx, frame);

            self.backend_panel.end_of_frame(ctx);
        }

        #[allow(clippy::collapsible_else_if)]
        egui::Window::new("Simulation").show(ctx, |ui| {
            if matches!(self.settings, Settings::EditDistribution(_)) {
                if ui.button("Stop Editing Distribution").clicked() {
                    self.settings = Settings::Default;
                }
                if ui.button("Add Element").clicked() {
                    self.target_distr.gaussians.push(NormalDistribution {
                        position: [0.0, 0.0],
                        scale: 0.5,
                        variance: 0.2,
                    });
                }
            } else {
                if ui.button("Edit Distribution").clicked() {
                    self.settings = Settings::EditDistribution(settings::DistrEditKind::Stateless);
                };
            };
            let ProgressMode::Batched { ref mut size } = self.algo.params.progress_mode;
            ui.add(
                // Safety: the slider begins at 1.
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
            let size = size.get_inner();
            if let Some(bg_task) = self.bg_task.as_ref() {
                match bg_task.get_progress() {
                    Progress::Pending(progress) => {
                        ui.add(ProgressBar::new(progress));
                    }
                    Progress::Finished => {
                        self.algo = self.bg_task.take().unwrap().get_value();
                    }
                };
            } else if ui.button("Batch step").clicked() {
                // TODO: All this is at best an early experiment.
                let _ = self.bg_task.insert({
                    let mut algo = self.algo.clone();
                    let target_distr = self.target_distr.clone();
                    // TODO: HIGHLY problematic!
                    // This means that the random state doesnt progress
                    let mut gaussian_distr_iter = self.gaussian_distr_iter.clone();
                    let mut uniform_distr_iter = self.uniform_distr_iter.clone();
                    BgTaskHandle::new(
                        move |mut communicate: BgCommunicate| {
                            for curr_step in 0..size {
                                algo.step(
                                    &target_distr,
                                    &mut gaussian_distr_iter,
                                    &mut uniform_distr_iter,
                                );
                                if communicate.checkup_bg(curr_step) {
                                    break;
                                }
                            }
                            algo
                        },
                        size,
                    )
                });
            }
        });

        egui::CentralPanel::default()
            // remove margins
            .frame(Default::default())
            .show(ctx, |ui| {
                egui::Frame::canvas(ui.style())
                    // remove margins here too
                    .inner_margin(egui::Margin::default())
                    .outer_margin(egui::Margin::default())
                    .show(
                        ui,
                        #[allow(clippy::shadow_unrelated, reason = "false positive, is related.")]
                        |ui| {
                            let px_size = ui.available_size();
                            let (rect, response) =
                                ui.allocate_exact_size(px_size, egui::Sense::hover());
                            // last painted element wins.
                            let painter = ui.painter();
                            self.target_distr_render.paint(
                                &self.target_distr,
                                painter,
                                rect * ctx.pixels_per_point(),
                            );
                            // self.diff_render.paint(
                            //     painter,
                            //     rect * ctx.pixels_per_point(),
                            //     &self.algo,
                            //     &self.target_distr,
                            // );

                            #[derive(Clone, Copy)]
                            enum ElementSettings {
                                Opened(usize),
                                // Closed,
                            }

                            self.drawer.paint(painter, rect, &self.algo);
                            if let Settings::EditDistribution(_) = self.settings {
                                let res_id = response.id;
                                // draw centers of gaussians, move them if dragged, open more settings if clicked
                                for (idx, ele) in self.target_distr.gaussians.iter_mut().enumerate()
                                {
                                    let pos = ndc_to_canvas_coord(ele.position.into(), rect.size());
                                    const CIRCLE_SIZE: f32 = 5.0;
                                    let pos_sense_rect =
                                        egui::Rect::from_center_size(pos, Vec2::splat(CIRCLE_SIZE));
                                    let mut pos_resp = ui
                                        .interact(
                                            pos_sense_rect,
                                            res_id.with(idx),
                                            egui::Sense::click_and_drag(),
                                        )
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                                    if pos_resp.dragged() {
                                        pos_resp = pos_resp
                                            .on_hover_and_drag_cursor(egui::CursorIcon::Grabbing);
                                    }
                                    if pos_resp.clicked() {
                                        ui.data_mut(|type_map| {
                                            type_map
                                                .insert_temp(res_id, ElementSettings::Opened(idx));
                                        });
                                    }
                                    // .on_hover_and_drag_cursor(egui::CursorIcon::Grabbing);
                                    let pos = rect.clamp(pos + pos_resp.drag_delta());

                                    let ndc_pos = canvas_coord_to_ndc(pos, rect.size());

                                    ele.position[0] = ndc_pos.x;
                                    ele.position[1] = ndc_pos.y;

                                    let pos_active = pos_resp.clicked()
                                        || pos_resp.dragged()
                                        || pos_resp.hovered();

                                    painter.circle_stroke(
                                        pos,
                                        CIRCLE_SIZE,
                                        egui::Stroke {
                                            color: egui::Color32::RED
                                                .gamma_multiply(if pos_active { 1.0 } else { 0.9 }),
                                            width: if pos_active { 2.0 } else { 1.0 },
                                        },
                                    );
                                }
                                if let Some(ElementSettings::Opened(idx)) =
                                    ui.data(|type_map| type_map.get_temp::<ElementSettings>(res_id))
                                {
                                    let close_planel = |ui: &mut egui::Ui| {
                                        ui.data_mut(|type_map| {
                                            type_map.remove::<ElementSettings>(res_id);
                                        });
                                    };
                                    // a proxy for (the presence of) ElementSettings (required because of the api of window).
                                    // has a defered close at the end of the scope.
                                    let mut opened = true;
                                    let gaussians = &mut self.target_distr.gaussians;
                                    egui::Window::new(format!("Settings for Gauss-Element {idx}"))
                                        .open(&mut opened)
                                        .fixed_pos(ndc_to_canvas_coord(
                                            gaussians
                                                .get(idx)
                                                .expect("Guaranteed to be present")
                                                .position
                                                .into(),
                                            rect.size(),
                                        ))
                                        .collapsible(false)
                                        .show(ctx, |ui| {
                                            let el = gaussians.get_mut(idx).unwrap();
                                            ui.add(
                                                egui::Slider::new(
                                                    &mut el.scale,
                                                    f32::EPSILON..=1.0,
                                                )
                                                .text("Scale"),
                                            );
                                            ui.add(
                                                egui::Slider::new(
                                                    &mut el.variance,
                                                    f32::EPSILON..=4.0,
                                                )
                                                .logarithmic(true)
                                                .text("Variance"),
                                            );
                                            if ui.button("delete").clicked() {
                                                gaussians.remove(idx);
                                                close_planel(ui);
                                            }
                                        });
                                    if !opened {
                                        close_planel(ui);
                                    }
                                }
                            }
                        },
                    );
            });
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

pub fn ndc_to_canvas_coord(ndc: egui::Pos2, canvas_size: egui::Vec2) -> egui::Pos2 {
    let center = (canvas_size - egui::Vec2::splat(canvas_size.min_elem())) / 2.0;
    ((ndc.to_vec2() + egui::Vec2::splat(1.0)) / 2.0 * canvas_size.min_elem() + center).to_pos2()
}

pub fn canvas_coord_to_ndc(canvas_coord: egui::Pos2, canvas_size: egui::Vec2) -> egui::Pos2 {
    let center = (canvas_size - egui::Vec2::splat(canvas_size.min_elem())) / 2.0;
    ((canvas_coord - center) / canvas_size.min_elem()) * 2.0 - Vec2::splat(1.0)
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
