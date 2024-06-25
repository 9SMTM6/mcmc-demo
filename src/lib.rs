#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod visualizations;
mod shaders;
pub use app::TemplateApp;
pub use visualizations::INITIAL_RENDER_SIZE;
