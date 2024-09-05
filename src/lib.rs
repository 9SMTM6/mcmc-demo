mod app;
mod helpers;
pub mod profile;
mod simulation;
mod target_distributions;
mod visualizations;
pub use app::McmcDemo;
#[cfg(target_arch = "wasm32")]
pub use helpers::html_bindings;
#[cfg(feature = "tracing")]
pub use profile::tracing::{define_subscriber, set_default_and_redirect_log};
pub use visualizations::INITIAL_RENDER_SIZE;

#[cfg(not(any(feature = "rng_pcg", feature = "rng_xorshift", feature = "rng_xoshiro")))]
compile_error!("no rng compiled in.");

use embassy_time::Timer;
use embassy_executor::Spawner;

#[embassy_executor::task]
async fn ticker() {
    let window = web_sys::window().expect("no global `window` exists");

    let mut counter = 0;
    loop {
        let document = window.document().expect("should have a document on window");
        let list = document.get_element_by_id("log").expect("should have a log element");

        let li = document.create_element("li").expect("error creating list item element");
        li.set_text_content(Some(&format!("tick {}", counter)));

        list.append_child(&li).expect("error appending list item");
        log::info!("tick {}", counter);
        counter += 1;

        Timer::after_secs(1).await;
    }
}

#[cfg(feature = "tracing")]
const DEFAULT_TRACE_LEVEL: Option<&'static str> = Some("wgpu_core=warn,wgpu_hal=warn,info");

// When compiling to web using trunk:
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // let mut executor = embassy_executor::Executor::new();
    // executor.start(|spawner: Spawner| {
    //     spawner.spawn(ticker()).unwrap();        
    // });
    use crate::html_bindings::*;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    crate::set_default_and_redirect_log(crate::define_subscriber(DEFAULT_TRACE_LEVEL));
    #[cfg(not(feature = "tracing"))]
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Info).ok();

    std::panic::set_hook(Box::new(move |panic_info| {
        try_display_panic(panic_info);

        console_error_panic_hook::hook(panic_info);
    }));

    spawner.spawn(ticker()).unwrap();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                get_egui_canvas(),
                web_options,
                Box::new(|cc| Ok(Box::new(crate::McmcDemo::new(cc)))),
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
    });
    remove_loading_state();
}
