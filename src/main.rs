#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(feature = "tracing")]
const DEFAULT_TRACE_LEVEL: Option<&'static str> = Some("wgpu_core=warn,wgpu_hal=warn,info");

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use egui::IconData;
    use mcmc_demo::INITIAL_RENDER_SIZE;

    // Log or trace to stderr (if you run with `RUST_LOG=debug`).
    // tracing has more and precise scope information, and works well with multithreading, where regular logging as a single threaded approach breaks.
    #[cfg(feature = "tracing")]
    mcmc_demo::set_default_and_redirect_log(mcmc_demo::define_subscriber(DEFAULT_TRACE_LEVEL));
    #[cfg(not(feature = "tracing"))]
    env_logger::init();

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
}

use embassy_time::Timer;

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

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    let executor = ::std::boxed::Box::leak(
        ::std::boxed::Box::new(::embassy_executor::Executor::new()),
    );
    executor
        .start(|spawner| {
            spawner.must_spawn(ticker());
        });
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

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
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
    });
    remove_loading_state();
}
