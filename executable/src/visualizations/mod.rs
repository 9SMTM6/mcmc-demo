mod egui_based;
mod shader_based;

use std::sync::Arc;

use macros::cfg_persistence_derive;

pub use egui_based::{
    Arrow, DistrEdit, ElementSettings, PredictionVariance, SamplePointVisualizer, SamplingPoint,
};

pub use shader_based::{
    BDAComputeDiff, BDADiff, BDADiffState, BdaComputeState, BdaComputeTask, INITIAL_RENDER_SIZE,
    MMGState, NormalDistribution, RWMHAcceptRecord, TargetDistribution,
};

use crate::{simulation::random_walk_metropolis_hastings::Rwmh, target_distr};

pub trait CanvasPainter {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect);
}

macro_rules! bg_display {
    ($($struct_name: ident),+,) => {
        #[cfg_persistence_derive]
        pub enum BackgroundDisplay {
            $($struct_name($struct_name),)+
        }


        #[derive(PartialEq, Clone, Copy)]
        #[repr(u8)]
        pub enum BackgroundDisplayDiscr {
            $($struct_name,)+
        }

        impl BackgroundDisplayDiscr {
            pub const VARIANTS: &'static [Self] = &[$(Self::$struct_name),+,];

            pub const fn display_name(&self) -> &str {
                match *self {
                    $(Self::$struct_name => stringify!($struct_name),)+
                }
            }
        }

        impl BackgroundDisplay {
            pub fn paint(
                &self,
                painter: &egui::Painter,
                rect: egui::Rect,
                algo: Arc<Rwmh>,
                target: &target_distr::Gaussian,
            ) {
                match self {
                    $(&Self::$struct_name(ref inner) => {
                        //  * ctx.pixels_per_point()
                        inner.paint(
                            painter,
                            rect,
                            algo,
                            target,
                        );
                    })+
                }
            }
        }

        impl From<&BackgroundDisplay> for BackgroundDisplayDiscr {
            fn from(value: &BackgroundDisplay) -> Self {
                use BackgroundDisplay as T;
                use BackgroundDisplayDiscr as D;
                match value {
                    $(&T::$struct_name(_) => D::$struct_name),+,
                }
            }
        }

        impl From<BackgroundDisplayDiscr> for BackgroundDisplay {
            fn from(value: BackgroundDisplayDiscr) -> Self {
                use BackgroundDisplay as T;
                use BackgroundDisplayDiscr as D;
                match value {
                    $(D::$struct_name => T::$struct_name(Default::default())),+,
                }
            }
        }
    }
}

trait AlgoPainter {
    fn paint(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        algo: Arc<Rwmh>,
        target: &target_distr::Gaussian,
    );
}

bg_display!(TargetDistribution, BDAComputeDiff, BDADiff,);

impl Default for BackgroundDisplay {
    fn default() -> Self {
        BackgroundDisplay::TargetDistribution(Default::default())
    }
}

impl BackgroundDisplayDiscr {
    pub fn selection_ui(mut self, ui: &mut egui::Ui) -> Self {
        for ele in Self::VARIANTS.iter() {
            ui.selectable_value(&mut self, *ele, ele.display_name());
        }
        self
    }

    // pub fn selection_ui(&mut self, ui: &mut egui::Ui) {
    //     for ele in Self::VARIANTS.iter() {
    //         ui.selectable_value(self, *ele, ele.display_name());
    //     }
    // }
}
