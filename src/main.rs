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

use embassy_executor::Executor;

#[cfg(feature = "tracing")]
const DEFAULT_TRACE_LEVEL: Option<&'static str> = Some("wgpu_core=warn,wgpu_hal=warn,info");

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::missing_panics_doc)]
pub fn main() {
    use egui::IconData;
    use mcmc_demo::INITIAL_RENDER_SIZE;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(DEFAULT_TRACE_LEVEL));
    #[cfg(not(feature = "tracing"))]
    env_logger::init();

    wasm_thread::spawn(|| {
        // Need &'static mut, this is the easiest way. If that gets to be an issue theres the alternative static_cell, or unsafe with a mut static.
        let executor = Box::leak(Box::new(Executor::new()));
        executor.run(|spawner| {
            spawner
                .spawn(mcmc_demo::gpu_task::gpu_scheduler(spawner))
                .unwrap();
        });
    });

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
}

#[cfg(target_arch = "wasm32")]
// If I use:
// #[embassy_executor::main]
// on the web I get compilation issues due to multiple instances of main being around.
// This is caused by that attribute being written for the 'usual' case of a library entrypoint, which isn't how trunk does things.
// I could move things into lib.rs, but then I get larger differences between native and web.
// Thus this loader function instead, which loads a 'main' task.
fn main() {
    // Need &'static mut, this is the easiest way. If that gets to be an issue theres the alternative static_cell, or unsafe with a mut static.
    let executor = Box::leak(Box::new(Executor::new()));
    // Don't ask me why they're are named differently. They do the same AFAICT.
    executor.start(|spawner| {
        spawner.spawn(wasm_main_task(spawner)).unwrap();
    });
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
// If I use
#[embassy_executor::task]
async fn wasm_main_task(spawner: embassy_executor::Spawner) {
    use mcmc_demo::html_bindings::*;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(DEFAULT_TRACE_LEVEL));
    #[cfg(not(feature = "tracing"))]
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Info).ok();

    std::panic::set_hook(Box::new(move |panic_info| {
        try_display_panic(panic_info);

        console_error_panic_hook::hook(panic_info);
    }));

    // TODO: sending data to that gpu task is proving difficult...
    // For starters on native this will require layered channels, one Channel to the thread (which will impose Send + Sync!) and one from that thread to the task,
    // And then theres the issue of the lifetimes of the channel.
    // Considering how embassy is implemented, its probably not as if it will work with smaller than static lifetimes for any executor related stuff anyways,
    // So perhaps I will just stick these into globals, which allows me to be static anywhere.
    // TODO: if this makes sense (I can make the gpu task blocking), consider matching this size to the maximum task in the executor
    // let _channel = embassy_sync::channel::Channel::<NoopRawMutex, GpuTaskEnum, 4>::new();

    spawner
        .spawn(mcmc_demo::gpu_task::gpu_scheduler(spawner))
        .unwrap();

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
}
