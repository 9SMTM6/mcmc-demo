use egui::Ui;

use crate::{
    app::{canvas_coord_to_ndc, ndc_to_canvas_coord},
    helpers::temp_ui_state::TempStateExtDelegatedToDataMethods,
    visualizations::shader_based::multimodal_gaussian::NormalDistribution,
};

#[derive(Clone, Copy)]
pub struct ElementSettings(usize);

impl ElementSettings {
    fn open_for(idx: usize, ui: &Ui) {
        ui.temp_ui_state::<Self>().create(Self(idx));
    }

    fn remove(ui: &Ui) {
        ui.temp_ui_state::<Self>().remove();
    }

    pub fn show_if_open(
        gaussians: &mut Vec<NormalDistribution>,
        ui: &egui::Ui,
        rect: egui::Rect,
        ctx: &egui::Context,
    ) {
        if let Some(Self(idx)) = ui.temp_ui_state().get() {
            #[allow(clippy::shadow_unrelated)]
            let close_planel = |ui: &egui::Ui| {
                Self::remove(ui);
            };
            // a proxy for (the presence of) ElementSettings (required because of the api of window).
            // has a deferred close at the end of the scope.
            let mut opened_proxy = true;
            egui::Window::new(format!("Settings for Gauss-Element {idx}"))
                .open(&mut opened_proxy)
                .fixed_pos(ndc_to_canvas_coord(
                    gaussians
                        .get(idx)
                        .expect("Guaranteed to be present")
                        .position
                        .into(),
                    rect.size(),
                ))
                .collapsible(false)
                .show(
                    ctx,
                    #[allow(clippy::shadow_unrelated)]
                    |ui| {
                        let el = gaussians.get_mut(idx).unwrap();
                        ui.add(egui::Slider::new(&mut el.scale, f32::EPSILON..=1.0).text("Scale"));
                        ui.add(
                            egui::Slider::new(&mut el.variance, f32::EPSILON..=4.0)
                                .logarithmic(true)
                                .text("Variance"),
                        );
                        if ui.button("delete").clicked() {
                            gaussians.remove(idx);
                            close_planel(ui);
                        }
                    },
                );
            if !opened_proxy {
                close_planel(ui);
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum DistrEdit {
    #[default]
    Editing,
}

impl DistrEdit {
    pub fn settings_ui(gaussians: &mut Vec<NormalDistribution>, ui: &mut Ui) {
        if DistrEdit::is_present(ui) {
            if ui.button("Stop Editing Distribution").clicked() {
                DistrEdit::remove(ui);
            }
            if ui.button("Add Element").clicked() {
                gaussians.push(NormalDistribution {
                    position: [0.0, 0.0],
                    scale: 0.5,
                    variance: 0.2,
                });
            }
        } else if ui.button("Edit Distribution").clicked() {
            DistrEdit::open(ui);
        };
    }

    fn open(ui: &Ui) {
        ui.temp_ui_state::<Self>().create_default();
    }

    fn remove(ui: &Ui) {
        ui.temp_ui_state::<Self>().remove();
        ElementSettings::remove(ui);
    }

    fn is_present(ui: &Ui) -> bool {
        ui.temp_ui_state::<DistrEdit>().get().is_some()
    }

    pub fn show_if_open(
        gaussians: &mut [NormalDistribution],
        ui: &Ui,
        response: &egui::Response,
        rect: egui::Rect,
        painter: &egui::Painter,
    ) {
        if let Some(Self::Editing) = ui.temp_ui_state::<Self>().get() {
            let res_id = response.id;
            // draw centers of gaussians, move them if dragged, open more settings if clicked
            for (idx, ele) in gaussians.iter_mut().enumerate() {
                let pos = ndc_to_canvas_coord(ele.position.into(), rect.size());
                const CIRCLE_SIZE: f32 = 5.0;
                let pos_sense_rect =
                    egui::Rect::from_center_size(pos, egui::Vec2::splat(CIRCLE_SIZE));
                let mut pos_resp = ui
                    .interact(
                        pos_sense_rect,
                        res_id.with(idx),
                        egui::Sense::click_and_drag(),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if pos_resp.dragged() {
                    pos_resp = pos_resp.on_hover_and_drag_cursor(egui::CursorIcon::Grabbing);
                }
                if pos_resp.clicked() {
                    ElementSettings::open_for(idx, ui);
                };
                let pos = rect.clamp(pos + pos_resp.drag_delta());

                let ndc_pos = canvas_coord_to_ndc(pos, rect.size());

                ele.position[0] = ndc_pos.x;
                ele.position[1] = ndc_pos.y;

                let pos_active = pos_resp.clicked() || pos_resp.dragged() || pos_resp.hovered();

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
    }
}
