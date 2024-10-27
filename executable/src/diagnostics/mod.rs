#[cfg(feature = "backend_panel")]
pub mod backend_panel;
#[cfg(all(feature = "wgpu_profile", not(target_arch = "wasm32")))]
#[path = "gpu_profile_enabled.rs"]
pub mod cfg_gpu_profile;
#[cfg(not(all(feature = "wgpu_profile", not(target_arch = "wasm32"))))]
#[path = "gpu_profile_disabled.rs"]
pub mod cfg_gpu_profile;
#[cfg(feature = "backend_panel")]
pub mod frame_history;
pub mod puffin;
pub mod tracing;
