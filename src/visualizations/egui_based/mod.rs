pub mod point_display;

use egui::{
    epaint::{ColorMode, PathShape, PathStroke},
    Color32, Pos2, Shape, Stroke, Vec2,
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
        let base = Shape::LineSegment {
            points: [start, start + direction],
            stroke: PathStroke {
                width: 1.5,
                color: ColorMode::Solid(Color32::RED),
            },
        };
        painter.extend([base, head]);
    }
}

pub struct SamplingPoint {
    pos: Pos2,
    ///shall be the number of samples at this point (=how long it stayed there/ how often a move away was rejected) divided by
    /// the maximum of that count among all sample points.
    sample_count_fract: f32,
}

impl SamplingPoint {
    pub fn new(pos: impl Into<Pos2>, sample_count_fract: f32) -> Self {
        Self {
            pos: pos.into(),
            sample_count_fract,
        }
    }
}

impl CanvasPainter for SamplingPoint {
    fn paint(&self, painter: &egui::Painter, _rect: egui::Rect) {
        let Self {
            pos,
            sample_count_fract,
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
    radius: f32,
}

impl PredictionVariance {
    pub fn new(pos: impl Into<Pos2>, radius: f32) -> Self {
        Self {
            pos: pos.into(),
            radius,
        }
    }
}

impl CanvasPainter for PredictionVariance {
    fn paint(&self, painter: &egui::Painter, _rect: egui::Rect) {
        let Self { pos, radius } = *self;
        painter.circle(
            pos,
            radius,
            Color32::TRANSPARENT,
            Stroke {
                color: Color32::WHITE,
                width: 1.0,
            },
        );
    }
}
