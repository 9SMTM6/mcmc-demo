use miette::{ErrReport, IntoDiagnostic, Result};
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};
use wgsl_to_wgpu::{create_shader_module_embedded, WriteOptions};

fn main() -> Result<()> {
    let shaders_dir = PathBuf::from("./shaders");
    let resolved_shaders = handle_c_pragma_once_style_imports(&shaders_dir).unwrap();
    let mut out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    out_dir.push(shaders_dir.file_name().unwrap());

    let resolved_shaders_dir = append_to_last_dir(&out_dir, "_resolved");

    fs::create_dir_all(resolved_shaders_dir.clone()).unwrap();

    for (el, resolved_source) in resolved_shaders.iter() {
        let mut file_path = resolved_shaders_dir.clone();
        let mut file_name = el.clone();
        file_name.push(".wgsl");
        file_path.push(file_name);
        fs::write(file_path, resolved_source).unwrap();
    }
    // first use wgsl_bindgen to get nice errors.
    // TODO: remove once we find a good alternative
    // TODO: to actually support pipeline overridable constants, these have to be sanitized out.
    bindgen_generation(&resolved_shaders_dir)?;
    let bindings_dir = append_to_last_dir(&out_dir, "_bindings");

    fs::create_dir_all(&bindings_dir).unwrap();

    wgsl_to_wgpu_generation(&resolved_shaders, &bindings_dir);
    Ok(())
}

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

    Ok(wgsl_files)
}

fn bindgen_generation(resolved_shaders_dir: &Path) -> Result<(), ErrReport> {
    let shader_entries = [
        "multimodal_gaussian.fragment",
        "fullscreen_quad.vertex",
        "diff_display.fragment",
    ];

    let mut bindgen = WgslBindgenOptionBuilder::default();
    bindgen
        .workspace_root(resolved_shaders_dir)
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .derive_serde(cfg!(feature = "persistence"))
        .type_map(RustWgslTypeMap);
    for source in shader_entries {
        bindgen.add_entry_point(format!(
            "{resolved_shaders_dir}/{source}.wgsl",
            resolved_shaders_dir = resolved_shaders_dir.to_string_lossy()
        ));
    }
    let bindgen = bindgen.output("src/shaders.rs").build().unwrap();

    bindgen.generate().into_diagnostic()
}

#[allow(dead_code)]
fn wgsl_to_wgpu_generation(resolved_shaders: &HashMap<OsString, String>, bindings_dir: &Path) {
    let shader_entries = [
        "multimodal_gaussian.fragment",
        "fullscreen_quad.vertex",
        "diff_display.fragment",
    ]
    .map(OsString::from);

    shader_entries.into_iter().for_each(|entrypoint| {
        let wgsl_source = resolved_shaders.get(&entrypoint).unwrap();
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
        let mut new_path = bindings_dir.to_owned();
        new_path.push(new_filename);
        fs::write(new_path, rust_bindings).unwrap();
    });
    // // Generate the Rust bindings and write to a file.
    // let text = create_shader_module_embedded(wgsl_source, WriteOptions::default()).unwrap();
    // let out_dir = std::env::var("OUT_DIR").unwrap();
    // std::fs::write(format!("{out_dir}/model.rs"), text.as_bytes()).unwrap();
    // todo!("will for certain require different shader source files. Doesnt do validation, doesnt do imports. IIRC wanted everything to be in one big shader, which caused me to go to bindgen, in addition to debug labels")
}

fn append_to_last_dir(directory: &Path, appendage: impl AsRef<OsStr>) -> PathBuf {
    let mut new_path = directory.parent().unwrap().to_path_buf();

    let mut new_dir = directory.file_name().unwrap().to_owned();

    new_dir.push(appendage);

    new_path.push(new_dir);
    new_path
}
