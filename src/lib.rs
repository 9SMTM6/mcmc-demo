mod app;
mod helpers;
pub mod profile;
mod simulation;
mod target_distributions;
pub mod visualizations;
pub use app::McmcDemo;
pub use helpers::gpu_task;
#[cfg(target_arch = "wasm32")]
pub use helpers::html_bindings;
#[cfg(feature = "tracing")]
pub use profile::tracing::{define_subscriber, set_default_and_redirect_log};
pub use visualizations::INITIAL_RENDER_SIZE;

#[cfg(not(any(feature = "rng_pcg", feature = "rng_xorshift", feature = "rng_xoshiro")))]
compile_error!("no rng compiled in.");
