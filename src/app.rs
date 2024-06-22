use crate::{custom3d_wgpu::Custom3d, test_fixed_gaussian};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    // #[cfg(any(feature = "glow", feature = "wgpu"))]
    #[serde(skip)]
    custom3d: Option<crate::custom3d_wgpu::Custom3d>,

    #[serde(skip)]
    gaussian: test_fixed_gaussian::FixedGaussian,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            gaussian: test_fixed_gaussian::FixedGaussian {},
            custom3d: Default::default(),
        }
    }
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
            gaussian: test_fixed_gaussian::FixedGaussian::new(
                cc.wgpu_render_state.as_ref().unwrap(),
            ),
            custom3d: Custom3d::new(cc),
            ..Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        egui::CentralPanel::default()
            // remove margins
            .frame(Default::default())
            .show(ctx, |ui| {
                self.gaussian.draw(ui);
            });

        // #[cfg(any(feature = "glow", feature = "wgpu"))]
        // if let Some(custom3d) = &mut self.custom3d {
        //     vec.push((
        //         "🔺 3D painting",
        //         Anchor::Custom3d,
        //         custom3d as &mut dyn eframe::App,
        //     ));
        // }
        // self.custom3d.as_mut().unwrap().update(ctx, _frame);
    }
}
