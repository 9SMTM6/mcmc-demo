use egui::{self, ProgressBar, Shadow, Vec2};
use macros::cfg_persistence_derive;
use std::{sync::Arc, time::Duration};
use tokio::sync::Notify;
use type_map::TypeMap;

use crate::{
    cfg_sleep,
    helpers::{
        BackgroundTaskManager, BgTaskHandle, GpuTaskSenders, TaskProgress, get_compute_queue,
        get_gpu_channels, gpu_scheduler, task_spawn, warn_feature_config,
    },
    simulation::random_walk_metropolis_hastings::{ProgressMode, Rwmh},
    target_distr,
    visualizations::{
        BDADiffState, BackgroundDisplay, BackgroundDisplayDiscr, BdaComputeState, DistrEdit,
        ElementSettings, MMGState, SamplePointVisualizer,
    },
};

#[cfg_persistence_derive]
#[serde(default)]
pub struct McmcDemo {
    // TODO: to make things more modular, switch to a composite struct for the simulation.
    // That struct will hold the algo, the data, the rngs and maybe the display (or an vector of displays, pointdisplay, targetdistr display, diff display).
    // It'll then implement legal transitions, e.g. changing the target distribution will lead to data reset etc.
    /// Note that this Arc is used in a copy-on-write fashion, with only atomic reassignments.
    algo: Arc<Rwmh>,
    point_display: Option<SamplePointVisualizer>,
    target_distr: target_distr::Gaussian,
    background_display: BackgroundDisplay,
    /// This holds resource managers for the main thread.
    ///
    /// If you want to hold copyable temporary ui state, use [`TempStateExtDelegatedToDataMethods`] instead.
    #[serde(skip)]
    local_resources: TypeMap,
}

impl Default for McmcDemo {
    fn default() -> Self {
        Self {
            algo: Default::default(),
            point_display: Some(Default::default()),
            target_distr: Default::default(),
            background_display: Default::default(),
            local_resources: TypeMap::new(),
        }
    }
}

// struct ComputeProfiler(Arc<wgpu_profiler::GpuProfiler>);
// struct GUIProfiler(Rc<wgpu_profiler::GpuProfiler>);

macro_rules! assert_none {
    ($expr:expr) => {
        assert!($expr.is_none());
    };
}

impl McmcDemo {
    /// Called once before the first frame.
    #[expect(clippy::missing_panics_doc, reason = "only used once")]
    pub async fn new(cc: &eframe::CreationContext<'_>) -> Self {
        warn_feature_config();

        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();

        let adapter = wgpu_render_state.adapter.clone();

        let (GpuTaskSenders { bda_compute }, gpu_rx) = get_gpu_channels();

        task_spawn(async move {
            let (compute_device, compute_queue) = get_compute_queue(&adapter).await;

            // I might end up creating a profiler for every workload.
            // Reason is that many relevant APIs require mutable access, making sharing annoying.
            // And, calling end_frame does not actually need to be called at top level, from my current understanding.
            // just call it at the end of the compute/render in these methods.
            // let msg = "Settings are statically chosen, I only build the profiler with the purpose to target tracy - together with tracing - and if the wgpu handles are invalid nothing else is gonna work either way";
            // let shared_tracy_context = wgpu_profiler::GpuProfiler::create_tracy_context(
            //     adapter.get_info().backend,
            //     &wgpu_render_state.device,
            //     &wgpu_render_state.queue,
            // )
            // .expect(msg);

            // let cfg_profiler_gui = Rc::new(
            //     wgpu_profiler::GpuProfiler::new_with_shared_tracy_context(
            //         Default::default(),
            //         shared_tracy_context.clone(),
            //     )
            //     .expect(msg),
            // );

            // let cfg_profiler_compute = Arc::new(
            //     wgpu_profiler::GpuProfiler::new_with_shared_tracy_context(
            //         Default::default(),
            //         shared_tracy_context.clone(),
            //     )
            //     .expect(msg),
            // );

            // _cfg_profiler_gui.scope("target_distr", encoder_or_pass, device)
            // TODO: Also make a profiler for compute (its actually the main purpose).

            let gpu_scheduler = gpu_scheduler((compute_device, compute_queue), gpu_rx);

            gpu_scheduler.await;
        });

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
            // TODO: make both themes work
            *visuals = egui::Visuals::dark();
            visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
            visuals.window_shadow = Shadow::NONE;
        });

        let state = Self::get_state(cc);

        // state
        //     .local_resources
        //     .insert(ComputeProfiler(cfg_profiler_compute));
        // state.local_resources.insert(GUIProfiler(cfg_profiler_gui));

        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("Compiling with WGPU enabled");

        let refresh_token = Arc::new(Notify::new());

        let refresh_on_finished = {
            let ctx = cc.egui_ctx.clone();
            let reshresh_token = refresh_token.clone();
            async move {
                while Arc::strong_count(&reshresh_token) > 1 {
                    reshresh_token.notified().await;
                    tracing::info!("Requesting repaint after finish");
                    ctx.request_repaint();
                    cfg_sleep!().await;
                }
                tracing::info!("Refresh loop canceled");
            }
        };

        task_spawn(refresh_on_finished);

        // let _callback_resources = state_guard
        //     .callback_resources
        //     .entry::<PipelineStateHolder>();

        // matches!(callback_resources, Entry::Vacant(_));

        // callback_resources.or_insert_with(|| {
        //     todo!();
        // });

        // TODO: consider dynamically initializing/uninitializing instead.

        {
            let mut state_guard = render_state.renderer.write();
            let callback_resources = &mut state_guard.callback_resources;

            let device = &render_state.device;
            let target_format: wgpu::ColorTargetState = render_state.target_format.into();

            assert_none!(callback_resources.insert(BdaComputeState::create(
                device,
                target_format.clone(),
                bda_compute,
                refresh_token.clone(),
            )));

            assert_none!(callback_resources.insert(MMGState::create(
                device,
                target_format.clone(),
                refresh_token.clone(),
            )));

            assert_none!(callback_resources.insert(BDADiffState::create(
                device,
                target_format.clone(),
                refresh_token.clone(),
            )));
        }

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
        // frame.wgpu_render_state().as_ref().expect("Compiling with WGPU enabled")
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://egui.rs

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Reset State").clicked() {
                    // We keep local_resources, as thats not really data.
                    *self = Self {
                        local_resources: std::mem::take(&mut self.local_resources),
                        ..Default::default()
                    };
                    ui.data_mut(|type_map| type_map.clear());
                }
                #[cfg(feature = "backend_panel")]
                {
                    use crate::diagnostics::backend_panel::BackendPanel;
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
                ui.centered_and_justified(|ui| {
                    ui.label(concat!(
                        env!("CARGO_PKG_NAME"),
                        " v",
                        env!("CARGO_PKG_VERSION")
                    ));
                });
            });
        });

        #[cfg(feature = "backend_panel")]
        if let Some(backend) = self
            .local_resources
            .get_mut::<crate::diagnostics::backend_panel::BackendPanel>()
        {
            backend.update(ctx, frame);
            backend.backend_panel(ctx, frame);
            backend.end_of_frame(ctx);
        }

        egui::Window::new("Simulation").show(
            ctx,
            #[expect(clippy::shadow_unrelated, reason = "false positive, is related.")]
            |ui| {
                let ProgressMode::Batched { ref mut size } =
                    Arc::make_mut(&mut self.algo).params.progress_mode;
                ui.add(
                    // Safety: the slider begins at 1.
                    unsafe {
                        egui::Slider::new(
                            size.get_inner_mut(),
                            // TODO: use default webgpu maximum size here to determine slider maximum, by determining how much space is left, roughly.
                            1..=100_000,
                        )
                    }
                    .logarithmic(true)
                    .text("batch size"),
                );
                let size = size.get_inner();
                struct BatchJob(BgTaskHandle<Arc<Rwmh>>);

                let bg_task = self.local_resources.get::<BatchJob>();
                if let Some(&BatchJob(ref bg_task)) = bg_task {
                    ui.add(
                        ProgressBar::new(match bg_task.get_progress() {
                            TaskProgress::Pending(progress) => progress,
                            TaskProgress::Finished => {
                                let params = self.algo.params.clone();
                                let mut thread_result = self
                                    .local_resources
                                    .remove::<BatchJob>()
                                    .unwrap()
                                    .0
                                    .get_value();
                                Arc::make_mut(&mut thread_result).params = params;
                                self.algo = thread_result;
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
                } else if ui.button("batch step").clicked() {
                    let existing = self.local_resources.insert(BatchJob({
                        let mut algo = self.algo.clone();
                        let target_distr = self.target_distr.clone();
                        BgTaskHandle::new(
                            move |mut communicate: BackgroundTaskManager| {
                                let algo_ref = Arc::make_mut(&mut algo);
                                for curr_step in 0..size {
                                    algo_ref.step(&target_distr);
                                    if communicate.update_progress_and_check_abort(curr_step) {
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
                if ui.button("reset simulation").clicked() {
                    self.local_resources.remove::<BatchJob>();
                    let params = self.algo.params.clone();
                    *Arc::make_mut(&mut self.algo) = Rwmh {
                        params,
                        ..Default::default()
                    };
                }
                ui.collapsing("background display", |ui| {
                    let prev_bg = BackgroundDisplayDiscr::from(&self.background_display);
                    let new_bg = prev_bg.selection_ui(ui);
                    if new_bg != prev_bg {
                        self.background_display = new_bg.into();
                    };
                });
                ui.collapsing("approximation point-display", |ui| {
                    if let Some(ref mut point_display) = self.point_display {
                        if ui.button("remove point display").clicked() {
                            self.point_display = None;
                        } else {
                            let mut accept_color_fullspace =
                                egui::Rgba::from(point_display.accepted_point_color)
                                    .to_rgba_unmultiplied();
                            ui.label("set acceptance color");
                            ui.color_edit_button_rgba_unmultiplied(&mut accept_color_fullspace);
                            let [r, g, b, a] = accept_color_fullspace;
                            point_display.accepted_point_color =
                                egui::Rgba::from_rgba_unmultiplied(r, g, b, a).into();
                            ui.add(
                                egui::Slider::new(&mut point_display.point_radius, 0.5..=5.0)
                                    .text("point radius"),
                            );
                            ui.add(
                                egui::Slider::new(&mut point_display.min_opacity, 0.1..=0.9)
                                    .text("minimum point alpha"),
                            );
                            if let Some(ref mut reject_color) = point_display.rejected_point_color {
                                if ui.button("remove display rejections display").clicked() {
                                    point_display.rejected_point_color = None;
                                } else {
                                    let mut reject_color_fullspace =
                                        egui::Rgba::from(*reject_color).to_array();
                                    ui.label("set rejection color");
                                    ui.color_edit_button_rgba_unmultiplied(
                                        &mut reject_color_fullspace,
                                    );
                                    let [r, g, b, a] = reject_color_fullspace;
                                    *reject_color =
                                        egui::Rgba::from_rgba_unmultiplied(r, g, b, a).into();
                                }
                            } else if ui.button("display rejections").clicked() {
                                point_display.rejected_point_color = Some(egui::Color32::YELLOW);
                            };
                        }
                    } else if ui.button("show point display").clicked() {
                        self.point_display = Some(Default::default());
                    }
                });
                egui::CollapsingHeader::new("target distribution")
                    .default_open(true)
                    .show(ui, |ui| {
                        DistrEdit::settings_ui(&mut self.target_distr.gaussians, ui);
                    });
                ui.collapsing("proposal probability", |ui| {
                    let prop = &mut Arc::make_mut(&mut self.algo).params.proposal;
                    ui.add(egui::Slider::new(&mut prop.sigma, 0.0..=1.0).text("Proposal sigma"));
                    prop.rng.rng.settings_ui(ui, ui.id());
                });
                ui.collapsing("acceptance probability", |ui| {
                    Arc::make_mut(&mut self.algo)
                        .params
                        .accept
                        .rng
                        .settings_ui(ui, ui.id());
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
                    .show(ui, |ui| {
                        let px_size = ui.available_size();
                        let (rect, response) =
                            ui.allocate_exact_size(px_size, egui::Sense::hover());
                        // last painted element wins.
                        let painter = ui.painter();
                        self.background_display.paint(
                            painter,
                            rect * ctx.pixels_per_point(),
                            self.algo.clone(),
                            &self.target_distr,
                        );

                        if let Some(ref point_display) = self.point_display {
                            point_display.paint(painter, rect, &self.algo);
                        }

                        let gaussians = &mut self.target_distr.gaussians;

                        DistrEdit::show_if_open(gaussians, ui, &response, rect, painter);

                        ElementSettings::show_if_open(gaussians, ui, rect, ctx);
                    });
            });
        // let ComputeProfiler(_compute_profiler) = self.local_resources.get().expect("blah");
        // let GUIProfiler(_gui_profiler) = self.local_resources.get().expect("blah");
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
