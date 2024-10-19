mod app;
pub mod diagnostics;
mod helpers;
mod simulation;
mod target_distributions;
pub mod visualizations;
pub use app::McmcDemo;
#[cfg(feature = "tracing")]
pub use diagnostics::tracing::{define_subscriber, set_default_and_redirect_log};
pub use helpers::gpu_task;
#[cfg(target_arch = "wasm32")]
pub use helpers::html_bindings;
pub use visualizations::INITIAL_RENDER_SIZE;

#[cfg(not(any(feature = "rng_pcg", feature = "rng_xorshift", feature = "rng_xoshiro")))]
compile_error!("no rng compiled in.");
