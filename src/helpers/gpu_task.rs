use std::rc::Rc;

// use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Timer;

pub(crate) trait GpuTask {
    async fn run(&self, compute_device: Rc<wgpu::Device>, compute_queue: Rc<wgpu::Queue>);
}

macro_rules! register_gpu_tasks {
    ($($gpu_task: ident),+) => {
        pub enum GpuTaskEnum {
            $($gpu_task($gpu_task)),+
        }

        impl GpuTask for GpuTaskEnum {
            async fn run(&self, compute_device: Rc<wgpu::Device>, compute_queue: Rc<wgpu::Queue>) {
                use GpuTaskEnum as D;
                match self {
                    $(&D::$gpu_task(ref inner) => inner.run(compute_device, compute_queue).await),+
                }
            }
        }
    };
}

use crate::{visualizations::shader_based::BdaComputeTask, GPU_TASK_CHANNEL};

register_gpu_tasks!(BdaComputeTask);

#[embassy_executor::task]
pub async fn ticker_task() {
    let mut counter = 0;
    loop {
        log::info!("tick {}", counter);
        counter += 1;

        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
/// embassy API doenst really allow this to be handled in a struct with drop etc.
///
/// Oh well.
#[allow(
    clippy::used_underscore_binding,
    reason = "That lint seems problematic on nightly right now"
)]
pub async fn gpu_scheduler(_spawner: embassy_executor::Spawner) {
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

    let compute_device = Rc::new(compute_device);
    let compute_queue = Rc::new(compute_queue);

    loop {
        let task = GPU_TASK_CHANNEL.receive().await;
        log::info!("Received GPU task");
        // IDK whether it'd be better to spawn a number of worker tasks that can submit parallel work, or handle parallelism in here.
        // worker tasks with await might be better for backpressure.
        task.run(compute_device.clone(), compute_queue.clone())
            .await;
        // spawner.spawn(run_gpu_task(task, compute_device.clone(), compute_queue.clone())).unwrap();
    }
}
