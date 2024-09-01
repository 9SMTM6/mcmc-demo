use egui::{self, ProgressBar, Shadow, Vec2};
use std::time::Duration;
use type_map::TypeMap;

use crate::{
    helpers::bg_task::{BgCommunicate, BgTaskHandle, Progress},
    simulation::random_walk_metropolis_hastings::{ProgressMode, Rwmh},
    target_distributions::multimodal_gaussian::GaussianTargetDistr,
    visualizations::{
        egui_based::{
            distrib_settings::{DistrEdit, ElementSettings},
            point_display::PointDisplay,
        },
        shader_based::{
            bda_compute::BDAComputeDiff, diff_display::BDADiff, target_distr::TargetDistribution,
        },
        BackgroundDisplay, BackgroundDisplayDiscr,
    },
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
    point_display: PointDisplay,
    target_distr: GaussianTargetDistr,
    background_display: BackgroundDisplay,
    /// This holds resource managers for the main thread.
    ///
    /// If you want to hold copyable temporary ui state, use [`TempStateExtDelegatedToDataMethods`] instead.
    #[cfg_attr(feature = "persistence", serde(skip))]
    local_resources: TypeMap,
}

impl Default for McmcDemo {
    fn default() -> Self {
        Self {
            algo: Default::default(),
            point_display: Default::default(),
            target_distr: Default::default(),
            background_display: Default::default(),
            local_resources: TypeMap::new(),
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
        TargetDistribution::init_gaussian_pipeline(render_state);
        BDADiff::init_pipeline(render_state);
        BDAComputeDiff::init_pipeline(render_state);
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
        // For inspiration and more examples, go to https://egui.rs

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            #[allow(clippy::shadow_unrelated)]
            ui.horizontal(|ui| {
                if ui.button("Reset State").clicked() {
                    *self = Default::default();
                }
                #[cfg(feature = "profile")]
                {
                    use crate::profile::backend_panel::BackendPanel;
                    let is_opened = self.local_resources.contains::<BackendPanel>();
                    let mut toggle_proxy = is_opened;
                    ui.toggle_value(&mut toggle_proxy, "Backend");
                    if toggle_proxy != is_opened {
                        if toggle_proxy {
                            let None = self
                                .local_resources
                                .insert::<BackendPanel>(Default::default())
                            else {
                                unreachable!()
                            };
                        } else {
                            self.local_resources.remove::<BackendPanel>();
                        }
                    }
                }
                egui::warn_if_debug_build(ui);
            });
        });

        #[cfg(feature = "profile")]
        if let Some(backend) = self
            .local_resources
            .get_mut::<crate::profile::backend_panel::BackendPanel>()
        {
            backend.update(ctx, frame);
            backend.backend_panel(ctx, frame);
            backend.end_of_frame(ctx);
        }

        egui::Window::new("Simulation").show(
            ctx,
            #[allow(clippy::shadow_unrelated)]
            |ui| {
                let ProgressMode::Batched { ref mut size } = self.algo.params.progress_mode;
                ui.add(
                    // Safety: the slider begins at 1.
                    unsafe {
                        egui::Slider::new(
                            size.get_inner_mut(),
                            // TODO: use default webgpu maximum size here to determine slider maximum, by determining how much space is left, roughly.
                            1..=(usize::MAX / usize::MAX.ilog2() as usize),
                        )
                    }
                    .logarithmic(true)
                    .text("batchsize"),
                );
                let size = size.get_inner();
                struct BatchJob(BgTaskHandle<Rwmh>);

                let bg_task = self.local_resources.get::<BatchJob>();
                if let Some(&BatchJob(ref bg_task)) = bg_task {
                    ui.add(
                        ProgressBar::new(match bg_task.get_progress() {
                            Progress::Pending(progress) => progress,
                            Progress::Finished => {
                                self.algo = self
                                    .local_resources
                                    .remove::<BatchJob>()
                                    .unwrap()
                                    .0
                                    .get_value();
                                // process is finished, but because of the control flow I can't show the button for the next batchstep yet.
                                // So this will have to do.
                                // Alternative would be moving the batch step UI put of this gigantic function and using this here,
                                // moving the ProgressBar rendering back into the Pending branch.
                                // But thats too much work for something still in the flow.
                                1.0
                            }
                        })
                        // this "fixes" the layout when displaying the progress bar.
                        // Without adding this, it will take up more horizontal space then the settings element took up originally,
                        // which looks very glitchy.
                        // There is probably a less hacky way that also works on other aspect ratios etc, but for now it'll have to do.
                        .desired_width(200.0),
                    );
                    ctx.request_repaint_after(Duration::from_millis(16));
                } else if ui.button("Batch step").clicked() {
                    let existing = self.local_resources.insert(BatchJob({
                        // TODO: HIGHLY problematic!
                        // This means that the random state doesnt progress
                        let mut algo = self.algo.clone();
                        let target_distr = self.target_distr.clone();
                        BgTaskHandle::new(
                            move |mut communicate: BgCommunicate| {
                                for curr_step in 0..size {
                                    algo.step(&target_distr);
                                    if communicate.checkup_bg(curr_step) {
                                        break;
                                    }
                                }
                                algo
                            },
                            size,
                        )
                    }));
                    assert!(
                        existing.is_none(),
                        "ought to be prevented from overriding this by UI logic"
                    );
                }
                ui.collapsing("Background Display", |ui| {
                    let prev_bg = BackgroundDisplayDiscr::from(&self.background_display);
                    let new_bg = prev_bg.selection_ui(ui);
                    if new_bg != prev_bg {
                        self.background_display = new_bg.into();
                    };
                });
                egui::CollapsingHeader::new("Target Distribution")
                    .default_open(true)
                    .show(ui, |ui| {
                        DistrEdit::settings_ui(&mut self.target_distr.gaussians, ui);
                    });
                ui.collapsing("Proposal Probability", |ui| {
                    let prop = &mut self.algo.params.proposal;
                    ui.add(egui::Slider::new(&mut prop.sigma, 0.0..=1.0).text("Proposal sigma"));
                    prop.rng.rng.settings_ui(ui, ui.id());
                });
                ui.collapsing("Acceptance Probability", |ui| {
                    self.algo.params.accept.rng.settings_ui(ui, ui.id());
                });
            },
        );

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
                            self.background_display.paint(
                                painter,
                                rect * ctx.pixels_per_point(),
                                &self.algo,
                                &self.target_distr,
                            );

                            self.point_display.paint(painter, rect, &self.algo);

                            let gaussians = &mut self.target_distr.gaussians;

                            DistrEdit::show_if_open(gaussians, ui, &response, rect, painter);

                            ElementSettings::show_if_open(gaussians, ui, rect, ctx);
                        },
                    );
            });
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
