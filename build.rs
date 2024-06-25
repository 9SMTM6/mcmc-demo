use miette::{IntoDiagnostic, Result};
use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};

fn main() -> Result<()> {
    let shader_root_dir = "shaders";
    let shader_entries = ["multimodal_gaussian", "fullscreen_quad"];

    println!("cargo:rerun-if-changed={shader_root_dir}");

    let mut bindgen = WgslBindgenOptionBuilder::default();
    bindgen
        .workspace_root(shader_root_dir)
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .derive_serde(cfg!(feature="persistence"))
        .type_map(RustWgslTypeMap);
    for source in shader_entries {
        bindgen.add_entry_point(format!("{shader_root_dir}/{source}.wgsl"));
    }
    let bindgen = bindgen.output("src/shaders.rs").build().unwrap();

    bindgen.generate().into_diagnostic()
}
