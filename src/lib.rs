#![warn(clippy::all, rust_2018_idioms, rust_2024_compatibility)]

mod app;
pub mod profile;
mod settings;
mod simulation;
mod target_distributions;
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
pub use app::McmcDemo;
pub use visualizations::INITIAL_RENDER_SIZE;
