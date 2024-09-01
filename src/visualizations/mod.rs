pub use egui_based::{Arrow, PredictionVariance, SamplingPoint};

pub mod egui_based;
pub mod shader_based;

pub use shader_based::{INITIAL_RENDER_SIZE, diff_display::BDADiffDisplay, bda_compute::BDAComputeDiffDisplay, multimodal_gaussian::MultiModalGaussianDisplay};

use crate::{simulation::random_walk_metropolis_hastings::Rwmh, target_distributions::multimodal_gaussian::MultiModalGaussian};

pub trait CanvasPainter {
    fn paint(&self, painter: &egui::Painter, rect: egui::Rect);
}

macro_rules! bg_display {
    ($($struct_name: ident),+,) => {
        #[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
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
            
            pub fn display_name(&self) -> &str {
                match self {
                    $(Self::$struct_name => stringify!($struct_name),)+
                }
            }
        }
        
        impl BackgroundDisplay {
            pub fn paint(
                &self,
                painter: &egui::Painter,
                rect: egui::Rect,
                algo: &Rwmh,
                target: &MultiModalGaussian,
            ) {
                match self {
                    $(Self::$struct_name(inner) => {
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

bg_display!(
    MultiModalGaussianDisplay,
    BDAComputeDiffDisplay,
    BDADiffDisplay,
);

impl Default for BackgroundDisplay {
    fn default() -> Self {
        BackgroundDisplay::MultiModalGaussianDisplay(Default::default())
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