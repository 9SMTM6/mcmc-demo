#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default, PartialEq)]
pub enum Settings {
    EditDistribution,
    #[default]
    Default,
}

impl Settings {
    pub fn draw(&mut self, ctx: &egui::Context) {
        egui::Window::new("Settings")
            .show(ctx, |ui| {
                if *self == Self::EditDistribution {
                    if ui.button("Stop Editing Distribution").clicked() {
                        *self = Self::Default;
                    }
                    ui.add_enabled(false, egui::Button::new("Add Element"))
                        .on_hover_text("TODO");
                } else {
                    if ui.button("Edit Distribution").clicked() {
                        *self = Self::EditDistribution;
                    };
                };
                // egui::global_dark_light_mode_buttons(ui);
            });
    }
}
