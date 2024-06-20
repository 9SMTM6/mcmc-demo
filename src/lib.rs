#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod custom3d_wgpu;
mod fullscreen_quad;
mod test_fixed_gaussian;
pub use app::TemplateApp;

pub const RENDER_SIZE: [f32; 2] = [640.0, 480.0];
