use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};

fn main() {
    let shader_root_dir = "src/visualizations/shaders/";
    let shader_sources = [ 
        "generic_gaussian",
        "fullscreen_quad",
        "test_fixed_gaussian",
    ];


    let mut bindgen = WgslBindgenOptionBuilder::default();
    bindgen
        .workspace_root(shader_root_dir)
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .type_map(RustWgslTypeMap)
    ;
    for source in shader_sources {
        bindgen.add_entry_point(format!("{shader_root_dir}/{source}.wgsl"));
    }
    let bindgen = bindgen
        .output(&format!("src/shaders.rs"))
        .build()
        .unwrap();

    bindgen.generate().unwrap();
}