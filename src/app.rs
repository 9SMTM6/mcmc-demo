use egui::{Frame, Rounding, Shadow};

use crate::visualizations::{self, FixedGaussian};

#[cfg_attr(feature="persistence", 
    // We derive Deserialize/Serialize so we can persist app state on shutdown.
    derive(serde::Deserialize, serde::Serialize),
    // if we add new fields, give them default values when deserializing old state
    serde(default),
)]
#[derive(Default)]
pub struct TemplateApp {
    #[cfg_attr(feature = "persistence", serde(skip))]
    gaussian: FixedGaussian,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Self {
            gaussian: FixedGaussian::new(cc.wgpu_render_state.as_ref().unwrap()),
        }
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

        #[cfg(debug_assertions)]
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        egui::Window::new("Settings")
            .frame(Frame {
                fill: ctx.style().visuals.code_bg_color.gamma_multiply(0.8),
                shadow: Shadow::default(),
                rounding: Rounding::same(5.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                // egui::global_dark_light_mode_buttons(ui);
                ui.label("Hello World!");
            });

        egui::CentralPanel::default()
            // remove margins
            .frame(Default::default())
            .show(ctx, |ui| {
                visualizations::draw_all(ui, &mut self.gaussian);
            });
    }
}
