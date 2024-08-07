#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use mcmc_demo::INITIAL_RENDER_SIZE;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(Some("info")));
    #[cfg(not(feature = "tracing"))]
    env_logger::init();
    #[cfg(feature = "profile")]
    mcmc_demo::profile::start_puffin_server();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(INITIAL_RENDER_SIZE)
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "mcmc-demo",
        native_options,
        Box::new(|cc| Ok(Box::new(mcmc_demo::McmcDemo::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use mcmc_demo::html_bindings::*;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(Some("info")));
    #[cfg(not(feature = "tracing"))]
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Info).ok();

    std::panic::set_hook(Box::new(move |panic_info| {
        try_display_panic(panic_info);

        console_error_panic_hook::hook(panic_info);
    }));

    let webgpu_supported = web_sys::window().unwrap().navigator().gpu().is_truthy();

    let web_options = eframe::WebOptions::default();

    if webgpu_supported {
        wasm_bindgen_futures::spawn_local(async {
            match eframe::WebRunner::new()
                .start(
                    get_egui_canvas(),
                    web_options,
                    Box::new(|cc| Ok(Box::new(mcmc_demo::McmcDemo::new(cc)))),
                )
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    let fmt_err = format!("{err:?}");
                    if fmt_err.contains("wgpu") {
                        display_failing_wgpu_info();
                    } else {
                        panic!("{fmt_err}")
                    }
                }
            };
        });
    } else {
        display_failing_wgpu_info();
    }
    remove_loading_el();
}
