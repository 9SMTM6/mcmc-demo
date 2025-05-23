mod async_last_task_processor;
mod bg_task;
mod gpu_task;
pub mod html_bindings;
mod temp_ui_state;

use std::future::Future;

use tokio::task;

pub use async_last_task_processor::TaskDispatcher;
pub use bg_task::{BackgroundTaskManager, BgTaskHandle, TaskProgress};
pub(crate) use gpu_task::{
    GpuTask, GpuTaskSenders, get_compute_queue, get_gpu_channels, gpu_scheduler,
};
pub use temp_ui_state::TempStateDataAccess;

// use crate::{definition_location, diagnostics::cfg_gpu_profile};

#[macro_export]
macro_rules! cfg_sleep {
    ($duration: expr) => {
        ::shared::cfg_if_expr!(
            => [all(feature = "debounce_async_loops", not(target_arch = "wasm32"))]
            ::tokio::time::sleep($duration)
            => [not]
            ::std::future::ready(())
        )
    };
    () => {
        $crate::cfg_sleep!(std::time::Duration::from_secs(1) / 3)
    }
}

/// This function emits warnings when a feature and/or target configuration is chosen, that will not work as expected.
#[allow(
    clippy::missing_const_for_fn,
    reason = "False positives depending on configuration"
)]
#[allow(
    clippy::cognitive_complexity,
    reason = "Yeah I'm not gonna make different functions for all features, just cause this lint seems to consider cfgs to be so difficult"
)]
pub fn warn_feature_config() {
    #[cfg(all(feature = "debounce_async_loops", target_arch = "wasm32"))]
    tracing::warn!(
        r#"Feature "debounce_async_loops" enabled, however other configuration disables this implicitly. Requires #not(target_arch = "wasm32")."#
    );

    #[cfg(all(
        feature = "tokio_console",
        not(all(tokio_unstable, not(target_arch = "wasm32")))
    ))]
    tracing::warn!(
        r#"Feature "tokio_console" enabled, however other configuration disables this implicitly. Requires #all(tokio_unstable, not(target_arch = "wasm32"))."#
    );

    #[cfg(all(feature = "tracy", target_arch = "wasm32"))]
    tracing::warn!(
        r#"Feature "tracy" enabled, however other configuration disables this implicitly. Requires #not(target_arch = "wasm32")."#
    );

    #[cfg(all(feature = "wgpu_profile", target_arch = "wasm32"))]
    tracing::warn!(
        r#"Feature "wgpu_profile" enabled, however other configuration disables this implicitly. Requires #not(target_arch = "wasm32")."#
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn task_spawn<U>(future: U) -> task::JoinHandle<U::Output>
where
    U: Future + Send + 'static,
    U::Output: Send,
{
    task::spawn(future)
}

#[cfg(target_arch = "wasm32")]
pub fn task_spawn<U>(future: U) -> task::JoinHandle<U::Output>
where
    U: Future + 'static,
{
    task::spawn_local(future)
}

pub fn wgpu_options() -> eframe::egui_wgpu::WgpuConfiguration {
    Default::default()
    // eframe::egui_wgpu::WgpuConfiguration {
    //     device_descriptor: Arc::new(|adapter| wgpu::DeviceDescriptor {
    //         label: Some(definition_location!()),
    //         required_features: cfg_gpu_profile::required_wgpu_features(adapter),
    //         ..Default::default()
    //     }),
    //     ..Default::default()
    // }
}
