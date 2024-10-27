use std::sync::Arc;

pub(crate) trait GpuTask {
    async fn run(&mut self, compute_device: Arc<wgpu::Device>, compute_queue: Arc<wgpu::Queue>);
}

use crate::{definition_location, visualizations::BdaComputeTask};

use super::async_last_task_processor::{self, TaskDispatcher, TaskExecutorFactory};

pub(crate) struct GpuTaskReceivers {
    pub bda_compute: TaskExecutorFactory<BdaComputeTask>,
}

#[allow(clippy::module_name_repetitions, reason = "Easier auto-import")]
pub struct GpuTaskSenders {
    pub bda_compute: TaskDispatcher<BdaComputeTask>,
}

pub(crate) fn get_gpu_channels() -> (GpuTaskSenders, GpuTaskReceivers) {
    let (send_gpu_task, gpu_task_runner) = async_last_task_processor::get::<BdaComputeTask>();

    (
        GpuTaskSenders {
            bda_compute: send_gpu_task,
        },
        GpuTaskReceivers {
            bda_compute: gpu_task_runner,
        },
    )
}

pub struct DebugTask;

impl GpuTask for DebugTask {
    async fn run(&mut self, _compute_device: Arc<wgpu::Device>, _compute_queue: Arc<wgpu::Queue>) {
        tracing::info!("I'm actually running");
    }
}

#[allow(
    clippy::used_underscore_binding,
    reason = "That lint seems problematic on nightly right now"
)]
/// TODO: Consider moving this to be a struct instead, with cancel on drop etc.
/// This being a function was originally required from embassy-rs, which is now replaced with tokio.
///
/// # Panics
/// If no wgpu adapter is found, if no wgpu device could be found with the provided settings, if the gpu_task channel was closed.
pub(crate) async fn gpu_scheduler(adapter: Arc<wgpu::Adapter>, rxs: GpuTaskReceivers) {
    let (compute_device, compute_queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some(definition_location!()),
                #[cfg(all(feature = "wgpu_profile", not(target_arch = "wasm32")))]
                required_features: adapter.features()
                    & wgpu_profiler::GpuProfiler::ALL_WGPU_TIMER_FEATURES,
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();

    // compute_device.start_capture();

    #[cfg_attr(
        target_arch = "wasm32",
        expect(
            clippy::arc_with_non_send_sync,
            reason = "Future needs to be Send on native, where wgpu types are send"
        )
    )]
    let compute_device = Arc::new(compute_device);
    #[cfg_attr(
        target_arch = "wasm32",
        expect(
            clippy::arc_with_non_send_sync,
            reason = "Future needs to be Send on native, where wgpu types are send"
        )
    )]
    let compute_queue = Arc::new(compute_queue);

    // Add a tokio::join when more GPU tasks are added.
    rxs.bda_compute
        .attach_task_executor(|| {
            let compute_device = compute_device.clone();
            let compute_queue = compute_queue.clone();
            |mut task: BdaComputeTask| async move {
                task.run(compute_device, compute_queue).await;
            }
        })
        .start_processing_loop()
        .await;
}

#[derive(Clone)]
pub struct RepaintToken {
    inner: egui::Context,
}

impl RepaintToken {
    pub const fn new(inner: egui::Context) -> Self {
        Self { inner }
    }

    pub fn request_repaint(&self) {
        self.inner.request_repaint();
    }
}
