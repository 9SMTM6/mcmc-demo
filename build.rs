use miette::{ErrReport, IntoDiagnostic, Result};
use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};
use wgsl_to_wgpu::{create_shader_module_embedded, WriteOptions};

fn bindgen_generation() -> Result<(), ErrReport> {
    let shader_entries = [
        "multimodal_gaussian.fragment",
        "fullscreen_quad.vertex",
        "diff_display.fragment",
    ];

    let mut out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let shader_dir = PathBuf::from("./shaders");

    out_dir.push(shader_dir.file_name().unwrap());

    let bindings_dir = append_to_last_dir(&out_dir, "_resolved")
        .to_str()
        .unwrap()
        .to_owned();

    let mut bindgen = WgslBindgenOptionBuilder::default();
    bindgen
        .workspace_root(&bindings_dir)
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .derive_serde(cfg!(feature = "persistence"))
        .type_map(RustWgslTypeMap);
    for source in shader_entries {
        bindgen.add_entry_point(format!("{bindings_dir}/{source}.wgsl"));
    }
    let bindgen = bindgen.output("src/shaders.rs").build().unwrap();

    bindgen.generate().into_diagnostic()
}

#[allow(dead_code)]
fn wgsl_to_wgpu_generation() {
    let shader_dir = PathBuf::from("./shaders");
    let resolved_files = handle_c_pragma_once_style_imports(&shader_dir).unwrap();

    let shader_entries = [
        "multimodal_gaussian.fragment",
        "fullscreen_quad.vertex",
        "diff_display.fragment",
    ]
    .map(OsString::from);

    let mut out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    out_dir.push(shader_dir.file_name().unwrap());

    let bindings_dir = append_to_last_dir(&out_dir, "_bindings");

    fs::create_dir_all(&bindings_dir).unwrap();

    shader_entries.into_iter().for_each(|entrypoint| {
        let wgsl_source = resolved_files.get(&entrypoint).unwrap();
        let rust_bindings = create_shader_module_embedded(
            wgsl_source,
            WriteOptions {
                derive_bytemuck_host_shareable: true,
                derive_serde: cfg!(feature = "persistence"),
                ..Default::default()
            },
        )
        .unwrap();
        let mut new_filename = entrypoint.clone();
        new_filename.push(".rs");
        let mut new_path = bindings_dir.clone();
        new_path.push(new_filename);
        fs::write(new_path, rust_bindings).unwrap();
    });
    // // Generate the Rust bindings and write to a file.
    // let text = create_shader_module_embedded(wgsl_source, WriteOptions::default()).unwrap();
    // let out_dir = std::env::var("OUT_DIR").unwrap();
    // std::fs::write(format!("{out_dir}/model.rs"), text.as_bytes()).unwrap();
    // todo!("will for certain require different shader source files. Doesnt do validation, doesnt do imports. IIRC wanted everything to be in one big shader, which caused me to go to bindgen, in addition to debug labels")
}

use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

/// TODO: Currently only supports direct imports.
///
/// TODO: Maybe I'll find a way to transport the info back to the original file (handle the input spans)
fn handle_c_pragma_once_style_imports(
    directory: &Path,
) -> std::io::Result<HashMap<OsString, String>> {
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

    let include_regex = regex::Regex::new(r#"\#import \"(.+)\.wgsl";"#).unwrap();

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

            let el = el.strip_prefix(directory).unwrap().file_stem().unwrap();

            (
                el.to_os_string(),
                // .to_string_lossy().to_string(),
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
            let resolved_source: String = first_pass
                .source
                .lines()
                .map(
                    |line| match include_regex.captures(line).map(|cap| cap.extract()) {
                        Some((_, [matched])) => {
                            let imported = wgsl_files.get(&OsString::from(matched)).unwrap();
                            assert_eq!(imported.direct_imports.len(), 0);
                            imported.source.clone()
                        }
                        None => line.to_owned(),
                    },
                )
                .reduce(|accum, line| accum + "\n" + &line)
                .unwrap_or_default();

            (el.to_owned(), resolved_source)
        })
        .collect();

    let mut out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    out_dir.push(directory.file_name().unwrap());

    let new_path = append_to_last_dir(&out_dir, "_resolved");

    fs::create_dir_all(new_path.clone()).unwrap();

    for (el, resolved_source) in wgsl_files.iter() {
        let mut file_path = new_path.clone();
        let mut file_name = el.clone();
        file_name.push(".wgsl");
        file_path.push(file_name);
        fs::write(file_path, resolved_source).unwrap();
    }

    Ok(wgsl_files)
}

fn append_to_last_dir(directory: &Path, appendage: impl AsRef<OsStr>) -> PathBuf {
    let mut new_path = directory.parent().unwrap().to_path_buf();

    let mut new_dir = directory.file_name().unwrap().to_owned();

    new_dir.push(appendage);

    new_path.push(new_dir);
    new_path
}

fn main() -> Result<()> {
    wgsl_to_wgpu_generation();
    bindgen_generation()?;
    Ok(())
}
