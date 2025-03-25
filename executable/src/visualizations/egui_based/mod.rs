mod point_display;
mod target_distrib_settings;

pub use point_display::SamplePointVisualizer;
pub use target_distrib_settings::{DistrEdit, ElementSettings};

use egui::{
    Color32, Pos2, Shape, Stroke, Vec2,
    epaint::{ColorMode, PathShape, PathStroke},
};

use super::CanvasPainter;

/// In contrast to the egui arrow, this arrow has an arrow head of constant size.
/// Note that the head will be added on top of start + direction, otherwise drawing an arrow of zero length is kinda awkward.
pub struct Arrow {
    start: Pos2,
    direction: Vec2,
}

impl Arrow {
    pub fn new(start: impl Into<Pos2>, direction: impl Into<Vec2>) -> Self {
        Self {
            direction: direction.into(),
            start: start.into(),
        }
    }
}

impl CanvasPainter for Arrow {
    fn paint(&self, painter: &egui::Painter, _rect: egui::Rect) {
        let Self { direction, start } = *self;
        const HALF_HEAD_THICKNESS: f32 = 4.0;
        let dir_only = direction.normalized();

        let head = Shape::Path(PathShape {
            points: vec![
                start + direction + dir_only.rot90() * (-HALF_HEAD_THICKNESS),
                start + direction + dir_only * (2.0 * HALF_HEAD_THICKNESS),
                start + direction + dir_only.rot90() * HALF_HEAD_THICKNESS,
            ],
            closed: true,
            fill: Color32::RED,
            stroke: PathStroke::NONE,
        });
        let shaft = Shape::LineSegment {
            points: [start, start + direction],
            stroke: PathStroke {
                width: 1.5,
                color: ColorMode::Solid(Color32::RED),
                ..Default::default()
            },
        };
        painter.extend([shaft, head]);
    }
}

pub struct SamplingPoint {
    pos: Pos2,
    /// shall be the number of samples at this point (=how long it stayed there/ how often a move away was rejected) divided by
    /// the maximum of that count among all sample points.
    normalized_sample_count: f32,
}

impl SamplingPoint {
    pub fn new(pos: impl Into<Pos2>, sample_count_fract: f32) -> Self {
        Self {
            pos: pos.into(),
            normalized_sample_count: sample_count_fract,
        }
    }
}

impl CanvasPainter for SamplingPoint {
    fn paint(&self, painter: &egui::Painter, _rect: egui::Rect) {
        let Self {
            pos,
            normalized_sample_count: sample_count_fract,
        } = *self;
        painter.circle(
            pos,
            4.0,
            Color32::WHITE.gamma_multiply(1.0 - sample_count_fract),
            Stroke::NONE,
        );
    }
}

pub struct PredictionVariance {
    pos: Pos2,
    variance_radius: f32,
}

impl PredictionVariance {
    pub fn new(pos: impl Into<Pos2>, variance_radius: f32) -> Self {
        Self {
            pos: pos.into(),
            variance_radius,
        }
    }
}

impl CanvasPainter for PredictionVariance {
    fn paint(&self, painter: &egui::Painter, _rect: egui::Rect) {
        let Self {
            pos,
            variance_radius,
        } = *self;
        painter.circle(
            pos,
            variance_radius,
            Color32::TRANSPARENT,
            Stroke {
                color: Color32::WHITE,
                width: 1.0,
            },
        );
    }
}
