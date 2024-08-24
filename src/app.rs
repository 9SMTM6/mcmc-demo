use std::{sync::Arc, time::Duration};

use egui::{self, mutex::Mutex, ProgressBar, Shadow, Vec2};
use rand::SeedableRng;
use rand_distr::StandardNormal;
use rand_pcg::Pcg32;

use crate::{
    helpers::{
        bg_task::{BgCommunicate, BgTaskHandle, Progress},
        egui_temp_state::TempStateExtDelegatedToDataMethods,
    }, profile::backend_panel::BackendPanel, settings::{self, Settings}, simulation::{
        random_walk_metropolis_hastings::{ProgressMode, Rwmh},
        Percentage, RngIter, WrappedRng, WrappedRngDiscriminants,
    }, target_distributions::multimodal_gaussian::MultiModalGaussian, visualizations::{
        egui_based::point_display::PointDisplay,
        shader_based::{
            diff_display::DiffDisplay,
            multimodal_gaussian::{shader_bindings::NormalDistribution, MultiModalGaussianDisplay},
        },
    }
};

#[cfg_attr(feature="persistence",
    // We derive Deserialize/Serialize so we can persist app state on shutdown.
    derive(serde::Deserialize, serde::Serialize),
    // if we add new fields, give them default values when deserializing old state
    serde(default),
)]
pub struct McmcDemo {
    // TODO: to make things more modular, switch to a composite struct for the simulation.
    // That struct will hold the algo, the data, the rngs and maybe the display (or an vector of displays, pointdisplay, targetdistr display, diff display).
    // It'll then implement legal transitions, e.g. changing the target distribution will lead to data reset etc.
    algo: Rwmh,
    drawer: PointDisplay,
    target_distr: MultiModalGaussian,
    #[allow(dead_code)]
    target_distr_render: MultiModalGaussianDisplay,
    #[allow(dead_code)]
    diff_render: DiffDisplay,
    settings: settings::Settings,
    gaussian_distr_iter: RngIter<StandardNormal>,
    uniform_distr_iter: RngIter<Percentage>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    // TODO: Create TempState struct that holds a generic field and another optional marker generic.
    // This SHOULD have a unique TypeId per type it holds (in addition with the generic I can have other things be unique too).
    // Then I can create a method on this type that takes a UI handle,
    // and then both renders the content (think of how to provide the rendering code for that)
    // as well as keeps the state in ui.data::IdTypeMap.
    // This way I dont have one big monolith such as here currently, where every small thing like background tasks have their own field and
    // kindof have to have the same lifetime (or be Option)
    bg_task: Option<BgTaskHandle<Rwmh>>,
    // #[cfg_attr(feature = "persistence", serde(skip))]
    // gpu_task: Option<BgTaskHandle<()>>,
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
            gaussian_distr_iter: RngIter::new(
                WrappedRngDiscriminants::Pcg32.seed_from_u64(42),
                StandardNormal,
            ),
            uniform_distr_iter: RngIter::new(
                WrappedRngDiscriminants::Pcg32.seed_from_u64(42),
                Percentage,
            ),
            bg_task: None,
            // gpu_task: None,
            // bg_tasks: vec![]
        }
    }
}

impl McmcDemo {
    /// Called once before the first frame.
    #[allow(clippy::missing_panics_doc)]
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

    pub fn get_state(#[allow(unused_variables)] cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
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
        #[allow(unused_variables)] frame: &mut eframe::Frame,
    ) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            #[allow(clippy::shadow_unrelated)]
            ui.horizontal(|ui| {
                if ui.button("Reset State").clicked() {
                    *self = Default::default();
                }
                #[cfg(feature = "profile")]
                {
                    let temp_state = ctx.temp_state::<Arc<Mutex<BackendPanel>>>();
                    let is_opened = temp_state.get().is_some();
                    let mut toggle_proxy = is_opened;
                    ui.toggle_value(&mut toggle_proxy, "Backend");
                    if toggle_proxy != is_opened {
                        if toggle_proxy {
                            temp_state.create_default();
                        } else {
                            temp_state.remove();
                        }
                    }
                }
                egui::warn_if_debug_build(ui);
            });
        });

        #[cfg(feature = "profile")]
        {
            if let Some(backend) = ctx.temp_state::<Arc<Mutex<BackendPanel>>>().get() {
                let mut backend = backend.lock();
                backend.update(ctx, frame);
                backend.backend_panel(ctx, frame);
                backend.end_of_frame(ctx);
            }
        }

        #[allow(clippy::collapsible_else_if)]
        egui::Window::new("Simulation").show(ctx, |ui| {
            WrappedRng::Pcg32(Pcg32::from_entropy()).settings_ui(ui);
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
                ui.add(ProgressBar::new(match bg_task.get_progress() {
                    Progress::Pending(progress) => progress,
                    Progress::Finished => {
                        self.algo = self.bg_task.take().unwrap().get_value();
                        // process is finished, but because of the control flow I can't show the button for the next batchstep yet.
                        // So this will have to do.
                        // Alternative would be moving the batch step UI put of this gigantic function and using this here,
                        // moving the ProgressBar rendering back into the Pending branch.
                        // But thats too much work for something still in the flow.
                        1.0
                    }
                }));
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
                        #[allow(clippy::shadow_unrelated)]
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
                            struct ElementSettingsOpened(usize);

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
                                        ui.temp_state::<ElementSettingsOpened>().create(ElementSettingsOpened(idx));
                                    };
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
                                if let Some(ElementSettingsOpened(idx)) = ui.temp_state().get()
                                {
                                    let close_planel = |ui: &mut egui::Ui| {
                                        ui.temp_state::<ElementSettingsOpened>().remove();
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
