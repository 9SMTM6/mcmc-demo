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
