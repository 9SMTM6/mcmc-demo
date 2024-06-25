use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};

fn main() {
    let shader_root_dir = "shaders";
    let shader_entries = ["generic_gaussian", "fullscreen_quad", "test_fixed_gaussian"];

    println!("cargo:rerun-if-changed={shader_root_dir}");

    let mut bindgen = WgslBindgenOptionBuilder::default();
    bindgen
        .workspace_root(shader_root_dir)
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .type_map(RustWgslTypeMap);
    for source in shader_entries {
        bindgen.add_entry_point(format!("{shader_root_dir}/{source}.wgsl"));
    }
    let bindgen = bindgen.output("src/shaders.rs").build().unwrap();

    bindgen.generate().unwrap();
}
