mod app;
mod bg_task;
mod helpers;
pub mod profile;
mod settings;
mod simulation;
mod target_distributions;
mod visualizations;
pub use app::McmcDemo;
#[cfg(feature = "tracing")]
pub use profile::tracing::{define_subscriber, set_default_and_redirect_log};
pub use visualizations::INITIAL_RENDER_SIZE;
pub use helpers::html_bindings;

#[cfg(not(any(feature = "rng_pcg", feature = "rng_xorshift", feature = "rng_xoshiro")))]
compile_error!("no rng compiled in.");
