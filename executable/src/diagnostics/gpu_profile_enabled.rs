//! The profiler must be available at high abstractions - e.g. for ending the current frame -,
//! but also at the lower levels - e.g. for binding to the compute/render passes its supposed to profile.
//!
//! I want to avoid doing this via global variables, if possible, but also dont want to create 2 versions of every function inbetween.
//!
//! Thus I create 2 versions of this module, one for gpu profiling enabled, one for without.
//! The disabled variant will contain eg. noop versions functions, or a common type alias with the active version,
//! but that aliases contains the unit type for the disabled variant.

pub type CfgProfiler = wgpu_profiler::GpuProfiler;

/// Comes in parts from https://github.com/Wumpf/wgpu-profiler/blob/main/examples/demo.rs
///
/// # Panics
///
/// If no profiler (including without tracy) can be created.
pub fn get_profiler(
    _backend: wgpu::Backend,
    _device: &wgpu::Device,
    _queue: &wgpu::Queue,
) -> CfgProfiler {
    // let Ok(gpu_context) = wgpu_profiler::create_tracy_gpu_client(backend, device, queue);
    // CfgProfiler::new_with_tracy_client_v2(Default::default(), gpu_context);
    // CfgProfiler::new_with_tracy_client(Default::default(), backend, device, queue).unwrap_or_else(
    //     |err| match err {
    //         wgpu_profiler::CreationError::TracyClientNotRunning
    //         | wgpu_profiler::CreationError::TracyGpuContextCreationError(_) => {
    //             tracing::warn!("Failed to connect to Tracy. Continuing without Tracy integration.");
    //             CfgProfiler::new(Default::default()).expect("Failed to create fallback profiler")
    //         }
    //         _ => {
    //             panic!("Failed to create profiler: {}", err);
    //         }
    //     },
    // )
    unimplemented!()
}

pub fn required_wgpu_features(adapter: &wgpu::Adapter) -> wgpu::Features {
    adapter.features() & wgpu_profiler::GpuProfiler::ALL_WGPU_TIMER_FEATURES
}
