use std::sync::Arc;

pub(crate) trait GpuTask {
    async fn run(&mut self, compute_device: Arc<wgpu::Device>, compute_queue: Arc<wgpu::Queue>);
}

macro_rules! register_gpu_tasks {
    ($($gpu_task: ident),+) => {
        pub enum GpuTaskEnum {
            $($gpu_task($gpu_task)),+
        }

        impl GpuTask for GpuTaskEnum {
            async fn run(&mut self, compute_device: Arc<wgpu::Device>, compute_queue: Arc<wgpu::Queue>) {
                use GpuTaskEnum as D;
                match self {
                    $(&mut D::$gpu_task(ref mut inner) => inner.run(compute_device, compute_queue).await),+
                }
            }
        }
    };
}

use crate::visualizations::shader_based::BdaComputeTask;

register_gpu_tasks!(BdaComputeTask, DebugTask);

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
/// Todo: Consider moving this to be a struct instead, with cancel on drop etc.
/// This being a function was originally required from embassy-rs, which is now replaced with tokio.
///
/// # Panics
/// If no wgpu adapter is found, if no wgpu device could be found with the provided settings, if the gpu_task channel was closed.
pub async fn gpu_scheduler(mut rx: tokio::sync::mpsc::Receiver<GpuTaskEnum>) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        #[cfg(target_arch = "wasm32")]
        backends: wgpu::Backends::BROWSER_WEBGPU,
        ..Default::default()
    });

    let adapter = instance.request_adapter(&Default::default()).await.unwrap();

    let (compute_device, compute_queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some(file!()),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();

    // compute_device.start_capture();

    #[cfg_attr(
        target_arch = "wasm32",
        allow(
            clippy::arc_with_non_send_sync,
            reason = "Future needs to be Send on native, where wgpu types are send"
        )
    )]
    let compute_device = Arc::new(compute_device);
    #[cfg_attr(
        target_arch = "wasm32",
        allow(
            clippy::arc_with_non_send_sync,
            reason = "Future needs to be Send on native, where wgpu types are send"
        )
    )]
    let compute_queue = Arc::new(compute_queue);

    loop {
        let Some(mut task) = rx.recv().await else {
            tracing::debug!("Finalizing GPU Task, as sending channel was closed");
            break;
        };
        // let task = GPU_TASK_CHANNEL.receive().await;
        tracing::debug!("Received GPU task");
        // IDK whether it'd be better to spawn a number of worker tasks that can submit parallel work, or handle parallelism in here.
        // worker tasks with await might be better for backpressure.
        // No longer waiting here as for some reason on native the first task seems to block forever.
        let task_future = {
            let compute_device = compute_device.clone();
            let compute_queue = compute_queue.clone();
            async move {
                task.run(compute_device.clone(), compute_queue.clone())
                    .await;
                // tracing::debug!("Task finished");
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        tokio::task::spawn(task_future);
        #[cfg(target_arch = "wasm32")]
        tokio::task::spawn_local(task_future);
        // spawner.spawn(run_gpu_task(task, compute_device.clone(), compute_queue.clone())).unwrap();
    }
}
