//! Entrypoint for native application and also web (wasm32).
//! This initializes things like the logging infrastructure and some permanent resources such as executors and wires up custom panic handlers - to show panics on the website.
//!
//! While I give my best to keep things unified between web and native, there are a few significant differences between the execution that show in here:
//!
//! 1. WebGPU isnt thread safe on the web, and using it from a background thread is even far less well supported then webgpu itself is.
//! 2. eframe Initialization APIs differ significantly on web and native.
//!    On the web it requires async and doesn't block (the main function exits, having spawned an eframe event loop that will keep going),
//!    on native eframe requires a blocking call that runs the event loop in place.
//! 3. This, combined with the cooperative multitasking from embassy-rs,
//!    means that I need to have embassy in a background thread on native (unless I manage to integrate embassy and eframex event loops, unlikely),
//!    while on the web it can't be in a background thread for compatibility reasons.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(feature = "tracing")]
const DEFAULT_TRACE_LEVEL: Option<&'static str> = Some("wgpu_core=warn,wgpu_hal=warn,info");

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[allow(
    clippy::missing_panics_doc,
    reason = "This is the entry point, noone else calls this."
)]
#[tokio::main]
pub async fn main() {
    use egui::IconData;
    use mcmc_demo::INITIAL_RENDER_SIZE;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(DEFAULT_TRACE_LEVEL));
    #[cfg(not(feature = "tracing"))]
    env_logger::init();

    // TODO: reconsider whether to do that, or use tokio::task::spawn on native.
    let local_set = tokio::task::LocalSet::new();

    local_set.spawn_local(async {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_min_inner_size(INITIAL_RENDER_SIZE)
                .with_icon(
                    // Not keen on converting the svg to a png on top.
                    // Not as if this currently works under wayland anyways.
                    IconData::default(), // NOTE: Adding an icon is optional
                                         // eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                                         //     .expect("Failed to load icon"),
                ),
            ..Default::default()
        };
        eframe::run_native(
            "mcmc-demo",
            native_options,
            Box::new(|cc| Ok(Box::new(mcmc_demo::McmcDemo::new(cc)))),
        )
        .unwrap();
    });

    local_set.await;
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use mcmc_demo::html_bindings::*;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(DEFAULT_TRACE_LEVEL));
    #[cfg(not(feature = "tracing"))]
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Info).ok();

    std::panic::set_hook(Box::new(move |panic_info| {
        try_display_panic_str(&panic_info.to_string());

        console_error_panic_hook::hook(panic_info);
    }));

    wasm_bindgen_futures::spawn_local(async move {
        let local_set = tokio::task::LocalSet::new();

        wasm_bindgen_futures::JsFuture::from(wasm_bindgen_rayon::init_thread_pool(
            wasm_thread::available_parallelism().unwrap().into(),
        ))
        .await
        .unwrap();

        local_set.spawn_local(async {
            let web_options = eframe::WebOptions::default();

            eframe::WebRunner::new()
                .start(
                    get_egui_canvas(),
                    web_options,
                    Box::new(|cc| Ok(Box::new(mcmc_demo::McmcDemo::new(cc)))),
                )
                .await
                .or_else(|err| {
                    let fmt_err = format!("{err:?}");
                    if fmt_err.contains("wgpu") {
                        // should've been handled in js
                        Ok(())
                    } else {
                        Err(err)
                    }
                })
                .unwrap();
            remove_loading_state();
        });
        local_set.await;
    });
}
