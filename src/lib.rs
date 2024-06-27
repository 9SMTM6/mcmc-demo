#![warn(clippy::all, rust_2018_idioms, rust_2024_compatibility)]

mod app;
mod settings;
mod visualizations;
// fix compilation warnings from generated code
#[allow(
    elided_lifetimes_in_paths,
    clippy::redundant_static_lifetimes,
    clippy::approx_constant,
    clippy::needless_borrow
)]
#[rustfmt::skip]
mod shaders;
pub use app::TemplateApp;
pub use visualizations::INITIAL_RENDER_SIZE;
