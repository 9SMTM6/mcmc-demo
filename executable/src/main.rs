//! Entrypoint for native application and also web (wasm32).
//! This initializes things like the logging infrastructure and some permanent resources such as executors and wires up custom panic handlers - to show panics on the website.
//!
//! While I give my best to keep things unified between web and native, there are a few significant differences between the execution that show in here:
//!
//! 1. WebGPU isn't currently thread safe on the web, and using it from a background thread is even far less well supported then webgpu itself is.
//!    Also, on native we have to progress computes with a blocking device.poll, while on the web the browser does this for us.
//! 2. eframe Initialization APIs differ significantly on web and native.
//!    On the web it requires async and doesn't block (the main function exits, having spawned an eframe event loop that will keep going),
//!    on native eframe requires a blocking call that runs the event loop in place.
//! 3. the main thread isn't allowed to block on the web.
//! 4. multitasking from tokio, also isn't super well supported on the web (no multithreaded runtime, no time module (=> no async sleep)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(feature = "tracing")]
const DEFAULT_TRACE_LEVEL: Option<&'static str> = Some("info,wgpu_core=warn,wgpu_hal=warn");
use mcmc_demo::wgpu_options;
use shared::cfg_if_expr;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[allow(
    clippy::missing_panics_doc,
    reason = "This is the entry point, noone else calls this."
)]
pub fn main() {
    use std::time::Duration;

    use mcmc_demo::INITIAL_RENDER_SIZE;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    cfg_if_expr!(
        => [feature = "tracing"]
        mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(DEFAULT_TRACE_LEVEL))
        => [not]
        // A log only knows events, no thread/task etc traces.
        // Mostly there for the web, to get a smaller WASM binary.
        // Since theres no subscriber installed, tracing events will also be redirected to log.
        env_logger::init()
    );
    // egui must run on the main thread.
    // At the same time tokio is broken when running a long-running blocking task on its executor. The solution - spawn_blocking - will move the execution off the main thread, so also isnt an Option.
    // Thus the manual finangling.

    let mut tokio_rt = tokio::runtime::Builder::new_multi_thread();

    #[cfg(all(feature = "debounce_async_loops", not(target_arch = "wasm32")))]
    tokio_rt.enable_time();

    let tokio_rt = tokio_rt.build().unwrap();

    tokio_rt.block_on(async {
        // doing blocking IO SHOULD be fine in this future, since it doesn't run on a task executor:
        // > This runs the given future on the current thread, blocking until it is complete, and yielding its resolved result. Any tasks or timers which the future spawns internally will be executed on the runtime.
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_min_inner_size(INITIAL_RENDER_SIZE)
                .with_icon(
                    eframe::icon_data::from_png_bytes(
                        &include_bytes!("../assets/favicon-256.png")[..],
                    )
                    .expect("Failed to load icon"),
                ),
            wgpu_options: wgpu_options(),
            ..Default::default()
        };
        eframe::run_native(
            "mcmc-demo",
            native_options,
            // this abomination is simply what eframe requires for app_creator
            Box::new(|cc| {
                Ok(Box::new(
                    // this abomination is required to re-enter an async context in the callback.
                    // I need an async context, because I need the compute device and queue,
                    // which I can only get with an async call.
                    tokio::task::block_in_place(|| {
                        tokio_rt.block_on(async { mcmc_demo::McmcDemo::new(cc).await })
                    }),
                ))
            }),
        )
        .unwrap();
    });

    tracing::info!("shutting down");
    // TODO: This is somewhat inelegant, maybe I can find a better way.
    tokio_rt.shutdown_timeout(Duration::from_secs(1));
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use mcmc_demo::html_bindings::*;

    cfg_if_expr!(
        => [feature = "tracing"]
        // Log or trace to stderr (if you run with `RUST_LOG=debug`).
        mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(DEFAULT_TRACE_LEVEL))
        => [not]
        // Redirect `log` message to `console.log` and friends:
        // Since theres no subscriber installed, tracing events will also be redirected to log.
        eframe::WebLogger::init(
            // I choose this indirect way to avoid having to directly depend on env.
            // That way nobody gets tempted to use env macros instead of tracing ones.
            std::str::FromStr::from_str("Info").unwrap(),
        )
        .unwrap()
    );

    std::panic::set_hook(Box::new(move |panic_info| {
        try_display_panic_str(&panic_info.to_string());

        console_error_panic_hook::hook(panic_info);
    }));

    wasm_bindgen_futures::spawn_local(async move {
        let local_set = tokio::task::LocalSet::new();

        local_set.spawn_local(async {
            // Assembling these futures inside of tokio_join breaks rust-analyzer, so I create them outside and only assign them in the end.
            let rayon_threadpool =
                wasm_bindgen_futures::JsFuture::from(wasm_bindgen_rayon::init_thread_pool(
                    wasm_thread::available_parallelism().unwrap().into(),
                ));
            let webapp_init = async {
                let web_options = eframe::WebOptions {
                    wgpu_options: wgpu_options(),
                    ..Default::default()
                };
                eframe::WebRunner::new()
                    .start(
                        get_egui_canvas(),
                        web_options,
                        Box::new(|cc| {
                            futures::executor::block_on(async {
                                Ok(Box::new(mcmc_demo::McmcDemo::new(cc).await)
                                    as Box<dyn eframe::App>)
                            })
                        }),
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
            };
            let (result, _) = tokio::join!(rayon_threadpool, webapp_init,);
            result.unwrap();
        });
        local_set.await;
    });
}
