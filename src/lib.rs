#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod visualizations;
pub use app::TemplateApp;

pub const INITIAL_RENDER_SIZE: [f32; 2] = [640.0, 480.0];
