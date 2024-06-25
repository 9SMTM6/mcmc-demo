use std::sync::atomic::{AtomicBool, Ordering};

use egui::Shadow;

use crate::{settings, visualizations::{self, MultiModalGaussian}};

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
            // and rust is rust, so I procect with this super complex atomic...
            //  I think I did it right.
            let first_render = FIRST_RENDER.compare_exchange(true, false, Ordering::SeqCst, Ordering::Relaxed);
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
                visualizations::draw_all(ui, &mut self.gaussian);
            });
    }
}
