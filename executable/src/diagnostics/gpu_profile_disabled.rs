//! The profiler must be available at high abstractions - e.g. for ending the current frame -,
//! but also at the lower levels - e.g. for binding to the compute/render passes its supposed to profile.
//!
//! I want to avoid doing this via global variables, if possible, but also dont want to create 2 versions of every function inbetween.
//!
//! Thus I create 2 versions of this module, one for gpu profiling enabled, one for without.
//! The disabled variant will contain eg. noop versions functions, or a common type alias with the active version,
//! but that aliases contains the unit type for the disabled variant.

pub type CfgProfiler = ();

#[expect(clippy::missing_const_for_fn, reason = "Other variant cant be const")]
pub fn get_profiler(
    _backend: wgpu::Backend,
    _device: &wgpu::Device,
    _queue: &wgpu::Queue,
) -> CfgProfiler {
}
