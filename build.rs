use std::fmt::Write;

use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};
use wgsl_to_wgpu::{create_shader_module, MatrixVectorTypes, WriteOptions};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
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
// include!(concat!(env!("OUT_DIR"), "/fullscreen_quad.wgsl.rs"));
    for source in shader_sources {
        bindgen.add_entry_point(format!("{shader_root_dir}/{source}.wgsl"));
    }
    let bindgen = bindgen
        .output(&format!("{out_dir}/shaders.rs"))
        .build()
        .unwrap();

    bindgen.generate().unwrap();
    
    // for source in shader_sources {
    //     println!("cargo:rerun-if-changed={shader_root_dir}/{source}.wgsl");
    //     let wgsl_source = std::fs::read_to_string(&format!("{shader_root_dir}/{source}")).unwrap();

    //     // Generate the Rust bindings and write to a file.
    //     let mut text = String::new();
    //     text += &create_shader_module(
    //         &wgsl_source,
    //         &format!("{source}"),
    //         WriteOptions {
    //             derive_bytemuck_vertex: true,
    //             matrix_vector_types: MatrixVectorTypes::Rust,
    //             derive_bytemuck_host_shareable: true,
    //             ..Default::default()
    //         },
    //     )
    //     .unwrap();

    //     std::fs::write(format!("{out_dir}/{source}.rs"), text.as_bytes()).unwrap();
    // }
}