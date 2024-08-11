use miette::{ErrReport, IntoDiagnostic, Result};
use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};

fn bindgen_generation() -> Result<(), ErrReport> {
    let shader_root_dir = "shaders/";
    let shader_entries = [
        "multimodal_gaussian.fragment",
        "fullscreen_quad.vertex",
        "diff_display.fragment",
    ];

    println!("cargo:rerun-if-changed={shader_root_dir}*");

    let mut bindgen = WgslBindgenOptionBuilder::default();
    bindgen
        .workspace_root(shader_root_dir)
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .derive_serde(cfg!(feature = "persistence"))
        .type_map(RustWgslTypeMap);
    for source in shader_entries {
        bindgen.add_entry_point(format!("{shader_root_dir}{source}.wgsl"));
    }
    let bindgen = bindgen.output("src/shaders.rs").build().unwrap();

    bindgen.generate().into_diagnostic()
}

#[allow(dead_code)]
fn wgsl_to_wgpu_generation() {
    todo!("will for certain require different shader source files. Doesnt do validation, doesnt do imports. IIRC wanted everything to be in one big shader, which caused me to go to bindgen, in addition to debug labels")
}

fn main() -> Result<()> {
    bindgen_generation()?;
    Ok(())
}
