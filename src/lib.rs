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
    clippy::unreadable_literal,
    clippy::wrong_self_convention,
)]
#[rustfmt::skip]
mod shaders;
pub use app::McmcDemo;
#[cfg(feature = "tracing")]
pub use profile::tracing::{define_subscriber, set_default_and_redirect_log};
pub use visualizations::INITIAL_RENDER_SIZE;
