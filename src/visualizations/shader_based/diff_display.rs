#![allow(unused)]
use egui::{Color32, Pos2};

use crate::{
    app::ndc_to_canvas_coord,
    simulation::random_walk_metropolis_hastings::{AcceptRecord, Algo},
    visualizations::{self, CanvasPainter},
};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct DiffDisplay {
    pub window_radius: f32,
}

impl DiffDisplay {
    #[allow(unused_variables)]
    pub fn paint(&self, painter: &egui::Painter, rect: egui::Rect, algo: &Algo) {
        todo!()
    }
}
