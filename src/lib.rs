mod app;
mod helpers;
pub mod profile;
mod simulation;
mod target_distributions;
pub mod visualizations;
pub use app::McmcDemo;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
pub use helpers::gpu_task;
use helpers::gpu_task::GpuTaskEnum;
#[cfg(target_arch = "wasm32")]
pub use helpers::html_bindings;
#[cfg(feature = "tracing")]
pub use profile::tracing::{define_subscriber, set_default_and_redirect_log};
pub use visualizations::INITIAL_RENDER_SIZE;

#[cfg(not(any(feature = "rng_pcg", feature = "rng_xorshift", feature = "rng_xoshiro")))]
compile_error!("no rng compiled in.");

// thread_local! {
// IDK how to use the NoopRawMutex variant properly... at least without threading it through everywhere, which #[embassy::task] makes difficult too.
pub static GPU_TASK_CHANNEL: Channel<CriticalSectionRawMutex, GpuTaskEnum, 4> = Channel::new();
// }
