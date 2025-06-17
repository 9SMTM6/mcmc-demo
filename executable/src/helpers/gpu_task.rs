use std::sync::Arc;

pub(crate) trait GpuTask {
    async fn run(&mut self, compute_device: Arc<wgpu::Device>, compute_queue: Arc<wgpu::Queue>);
}

use crate::{
    definition_location, diagnostics::cfg_gpu_profile::required_wgpu_features,
    visualizations::BdaComputeTask,
};

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

/// # Panics
/// If no wgpu device could be found with the provided settings, if the gpu_task channel was closed.
pub(crate) async fn get_compute_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some(definition_location!()),
                required_features: required_wgpu_features(adapter),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap()
}

#[allow(
    clippy::used_underscore_binding,
    reason = "That lint seems problematic on nightly right now"
)]
/// TODO: Consider moving this to be a struct instead, with cancel on drop etc.
/// This being a function was originally required from embassy-rs, which is now replaced with tokio.
pub(crate) async fn gpu_scheduler(
    (compute_device, compute_queue): (wgpu::Device, wgpu::Queue),
    rxs: GpuTaskReceivers,
) {
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
