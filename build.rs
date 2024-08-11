use miette::{ErrReport, IntoDiagnostic, Result};
use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};

fn bindgen_generation() -> Result<(), ErrReport> {
    let shader_root_dir = "shaders_resolved/";
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
    handle_c_pragma_once_style_imports(&PathBuf::from("./shaders")).unwrap();
    // todo!("will for certain require different shader source files. Doesnt do validation, doesnt do imports. IIRC wanted everything to be in one big shader, which caused me to go to bindgen, in addition to debug labels")
}

use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

/// TODO: Currently only supports direct imports.
fn handle_c_pragma_once_style_imports(directory: &Path) -> std::io::Result<()> {
    let mut wgsl_files = HashSet::<PathBuf>::new();
    let wgsl_ext: OsString = String::from("wgsl").into();
    for entry in fs::read_dir(directory)? {
        let entry = entry?.path();
        if entry.extension() == Some(wgsl_ext.as_os_str()) {
            wgsl_files.insert(entry);
        }
    }

    #[derive(Debug)]
    struct FirstPass {
        source: String,
        direct_imports: Vec<String>,
    }

    let include_regex = regex::Regex::new(r#"\#import \"(.+)\";"#).unwrap();

    let wgsl_files: HashMap<_, _> = wgsl_files
        .into_iter()
        .map(|el| {
            let source = fs::read_to_string(&el).unwrap();
            let direct_imports: Vec<_> = source
                .lines()
                .filter_map(|line| {
                    let captures = include_regex.captures(line);
                    captures.map(|cap| {
                        let (_, [matched]) = cap.extract();
                        matched.to_owned()
                    })
                })
                .collect();

            let el = el.strip_prefix(directory).unwrap();

            (
                el.to_string_lossy().to_string(),
                FirstPass {
                    source,
                    direct_imports,
                },
            )
        })
        .collect();

    let wgsl_files: HashMap<_, _> = wgsl_files
        .iter()
        .map(|(el, first_pass)| {
            let resolved_source: String = first_pass.source
                .lines()
                .map(|line| {
                    match include_regex.captures(line).map(|cap| cap.extract()) {
                        Some((_, [matched])) => {
                            let imported = wgsl_files.get(matched).unwrap();
                            assert_eq!(imported.direct_imports.len(), 0);
                            imported.source.clone()
                        },
                        None => line.to_owned(),
                    }
                })
                .map(|line| format!("{line}\n"))
                .collect();

            (el, resolved_source)
        })
        .collect();

    let mut new_path = directory.parent().unwrap().to_path_buf();

    let mut new_dir = directory.file_name().unwrap().to_owned();

    new_dir.push("_resolved");

    new_path.push(new_dir);

    fs::create_dir_all(new_path.clone()).unwrap();

    for (el, resolved_source) in wgsl_files {
        let mut file_path = new_path.clone();
        file_path.push(el);
        fs::write(file_path, resolved_source).unwrap();
    }

    Ok(())
}

fn main() -> Result<()> {
    wgsl_to_wgpu_generation();
    bindgen_generation()?;
    Ok(())
}
