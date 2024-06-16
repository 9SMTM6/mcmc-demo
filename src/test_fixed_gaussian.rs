use eframe::egui_wgpu::{CallbackTrait, RenderState};
use wgpu::{FragmentState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, VertexState};

struct GaussPipeline {
    pipeline: RenderPipeline,
}

#[derive(Clone, Copy)]
pub struct FixedGaussian{}

impl FixedGaussian {
    pub fn new(render_state: &RenderState) -> Self {
        let dev = &render_state.device;

        let shader = dev.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("test_fixed_gaussian.wgsl").into()),
        });

        let pipeline = dev.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: VertexState {
                module: &shader,
                buffers: &[],
                compilation_options: Default::default(),
                entry_point: "vs_main",
            },
            fragment: Some(FragmentState {
                module: &shader,
                compilation_options: Default::default(),
                entry_point: "fs_main",
                targets: &[Some(render_state.target_format.into())],
            }),
            label: None,
            layout: None,
            depth_stencil: None,
            multisample: Default::default(),
            multiview: Default::default(),
            primitive: Default::default(),
        });

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our struct, we insert it into the
        // `callback_resources` type map, which is stored alongside the render pass.
        let None = render_state.renderer.write().callback_resources.insert(GaussPipeline { pipeline }) else {
            panic!("pipeline already present?!")
        };

        Self {}
    }
}

impl CallbackTrait for FixedGaussian {
    fn paint<'a>(
            &'a self,
            _info: egui::PaintCallbackInfo,
            render_pass: &mut wgpu::RenderPass<'a>,
            callback_resources: &'a eframe::egui_wgpu::CallbackResources,
        ) {
        let GaussPipeline {pipeline} = callback_resources.get().unwrap();

        render_pass.set_pipeline(pipeline);
        render_pass.draw(0..6, 0..1);
    }
}

impl FixedGaussian {
    pub fn draw(&self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            // ([0.0, 0.0].into()
            let rect = egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::splat(800.0));
            // let (rect, response) =
            //     ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());
            ui.painter().add(eframe::egui_wgpu::Callback::new_paint_callback(rect, self.clone()))
        });
    }
}
