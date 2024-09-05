#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use embassy_time::Timer;
use embassy_executor::{Executor, Spawner};

#[embassy_executor::task]
pub async fn ticker() {
    let mut counter = 0;
    loop {
        log::info!("tick {}", counter);
        counter += 1;
        
        Timer::after_secs(1).await;
    }
}

#[cfg(feature = "tracing")]
const DEFAULT_TRACE_LEVEL: Option<&'static str> = Some("wgpu_core=warn,wgpu_hal=warn,info");

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    // let mut executor = embassy_executor::Executor::new();
    // executor.start(|spawner: Spawner| {
    //     spawner.spawn(ticker()).unwrap();        
    // });
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
    
    spawner.spawn(ticker()).unwrap();
    
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

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[embassy_executor::task]
pub async fn main_task(spawner: Spawner) {
    use egui::IconData;
    use mcmc_demo::INITIAL_RENDER_SIZE;
    
    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(DEFAULT_TRACE_LEVEL));
    #[cfg(not(feature = "tracing"))]
    env_logger::init();
    
    spawner.spawn(ticker()).unwrap();
    
    // TODO: this is the cause of the execution failure of the executor.
    // It blocks the executor from starting, since `run_native` never returns.
    // Moving this to another thread doesn't work, since I can't start a UI application from that.
    // So likely I will have to move the executor spawn to another thread, though I'm somewhat unwilling to do so right now,
    // as that means I can't use async during initialization of the application.
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
    ).unwrap();
}

fn main() {
    // was using static-cell, but I want to avoid that dependency.
    let executor = Box::leak(Box::new(Executor::new()));
    // Don't ask me why they're are named differently. They do the same AFAICT.
    #[cfg(target_arch = "wasm32")]
    executor.start(|spawner| {
        spawner.spawn(main_task(spawner)).unwrap();
    });
    #[cfg(not(target_arch = "wasm32"))]
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner)).unwrap();
    });
}
