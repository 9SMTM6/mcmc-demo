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
    clippy::needless_borrow,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_const_for_fn,
    clippy::module_name_repetitions,
    clippy::pattern_type_mismatch,
)]
#[rustfmt::skip]
mod shaders;
pub use app::McmcDemo;
pub use visualizations::INITIAL_RENDER_SIZE;
