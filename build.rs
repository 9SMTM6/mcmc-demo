use miette::{ErrReport, IntoDiagnostic, Result};
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use wgsl_bindgen::{RustWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};
use wgsl_to_wgpu::{create_shader_module_embedded, WriteOptions};

fn main() -> Result<()> {
    let start = std::time::Instant::now();
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
    println!(
        "build.rs runtime {runtime} ms",
        runtime = start.elapsed().as_millis()
    );
    Ok(())
}

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

    #[derive(Debug, Clone)]
    struct UnfinalizedPass {
        cleaned_source: String,
        /// for recording the imports in the order they should be resolved.
        resolved_imports_in_order: Vec<OsString>,
        /// These imports still had unresolved imports last pass.
        unresolved_imports: Vec<OsString>,
    }

    let include_regex = regex::Regex::new(r#"\#import \"(.+)\.wgsl";"#).unwrap();
    let include_regex_error1 = regex::Regex::new(r#"\#import \"(.+)\.wgsl""#).unwrap();

    let mut wgsl_files: HashMap<OsString, UnfinalizedPass> = wgsl_files
        .into_iter()
        .map(|el| {
            let source = fs::read_to_string(&el).unwrap();
            let mut first_actual_sourcecode = None::<usize>;
            let mut direct_imports_in_order = Vec::<OsString>::new();
            for (idx, line) in source.lines().enumerate() {
                if let Some((_, [matched])) = include_regex.captures(line).map(|cap| cap.extract())
                {
                    assert!(
                        first_actual_sourcecode.is_none(),
                        "Import after actual sourcecode"
                    );
                    direct_imports_in_order.push(OsString::from(matched));
                } else if let Some((_, [_])) =
                    include_regex_error1.captures(line).map(|cap| cap.extract())
                {
                    panic!("error in import syntax, {el:?}:{idx}.")
                } else if first_actual_sourcecode.is_none() && line.trim() != "" {
                    first_actual_sourcecode = Some(idx);
                }
            }

            let cleaned_source = first_actual_sourcecode
                .map(|start_line| {
                    source
                        .lines()
                        .skip(start_line)
                        .map(String::from)
                        .reduce(|accum, line| accum + "\n" + &line)
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            let el = el.strip_prefix(directory).unwrap().file_stem().unwrap();

            (
                el.to_os_string(),
                UnfinalizedPass {
                    cleaned_source,
                    unresolved_imports: direct_imports_in_order,
                    resolved_imports_in_order: Vec::new(),
                },
            )
        })
        .collect();

    let mut loop_count = 0;
    const MAX_IMPORT_DEPTH: usize = 6;

    loop {
        let mut still_unresolved = false;
        let mut old_wgsl_files = HashMap::<OsString, UnfinalizedPass>::new();
        std::mem::swap(&mut wgsl_files, &mut old_wgsl_files);
        for (
            filename,
            &UnfinalizedPass {
                ref cleaned_source,
                ref resolved_imports_in_order,
                ref unresolved_imports,
            },
        ) in old_wgsl_files.iter()
        {
            let mut new_unresolved = Vec::<OsString>::new();
            let mut new_resolved = resolved_imports_in_order.clone();
            let mut unresolved_beforehand = false;
            for unresolved_import in unresolved_imports {
                let import = &old_wgsl_files.get(unresolved_import).unwrap();
                if import.unresolved_imports.is_empty() && !unresolved_beforehand {
                    new_resolved.extend(import.resolved_imports_in_order.clone().into_iter());
                    new_resolved.push(unresolved_import.clone());
                } else {
                    unresolved_beforehand = true;
                    new_unresolved.push(unresolved_import.clone());
                };
            }
            let old_content: Option<UnfinalizedPass> = wgsl_files.insert(
                filename.clone(),
                UnfinalizedPass {
                    // thats potentially expensive...
                    // we could reintroduce a first pass,
                    // and then map that into a IntermediatePass,
                    // that only holds the filenames.
                    // Or simply go wild with Rc's.
                    cleaned_source: cleaned_source.clone(),
                    resolved_imports_in_order: new_resolved,
                    unresolved_imports: new_unresolved,
                },
            );
            assert!(old_content.is_none(), "Double insert");
            still_unresolved = still_unresolved || unresolved_beforehand;
        }
        loop_count += 1;

        if loop_count > MAX_IMPORT_DEPTH {
            panic!("import depth exceeded");
        }
        if !still_unresolved {
            break;
        }
    }

    let resolved_wgsl: HashMap<OsString, String> = wgsl_files
        .iter()
        .map(
            |(
                filename,
                &UnfinalizedPass {
                    ref cleaned_source,
                    ref resolved_imports_in_order,
                    ref unresolved_imports,
                },
            )| {
                assert!(unresolved_imports.is_empty());
                let resolved_imports = resolved_imports_in_order
                    .iter()
                    .map(|import_name| wgsl_files.get(import_name).unwrap().cleaned_source.clone())
                    .filter(|el| el.trim() != "")
                    .reduce(|accum, line| accum + "\n" + &line)
                    .map(|el| el + "\n")
                    .unwrap_or_default();

                let resolved_source = resolved_imports + cleaned_source;
                (filename.clone(), resolved_source)
            },
        )
        .collect();

    Ok(resolved_wgsl)
}

fn bindgen_generation(resolved_shaders_dir: &Path) -> Result<(), ErrReport> {
    let shader_entries = [
        "multimodal_gaussian.fragment",
        "fullscreen_quad.vertex",
        "diff_display.fragment",
        "binary_distance_approx.compute",
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
    let mut out_file = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    out_file.push("wgsl_bindgen_generated.rs");

    let bindgen = bindgen.output(out_file).build().unwrap();

    bindgen.generate().into_diagnostic()
}

#[allow(dead_code)]
fn wgsl_to_wgpu_generation(resolved_shaders: &HashMap<OsString, String>, bindings_dir: &Path) {
    let shader_entries = [
        "multimodal_gaussian.fragment",
        "fullscreen_quad.vertex",
        "diff_display.fragment",
        "binary_distance_approx.compute",
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
