mod app;
mod bg_task;
pub mod html_bindings;
pub mod profile;
mod settings;
mod simulation;
mod target_distributions;
mod visualizations;
pub use app::McmcDemo;
#[cfg(feature = "tracing")]
pub use profile::tracing::{define_subscriber, set_default_and_redirect_log};
pub use visualizations::INITIAL_RENDER_SIZE;
