#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(mcmc_demo::INITIAL_RENDER_SIZE)
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
        Box::new(|cc| Ok(Box::new(mcmc_demo::TemplateApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    fn get_loading_text() -> Option<web_sys::Element> {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"))
    }

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Show in the HTML that start has failed
        get_loading_text().map(|e| {
            e.set_inner_html(
                &format!(
r#"
    <p> The app has crashed. See the developer console for details. </p>
    <p style="font-size:10px" align="left">
        {panic_info}
    </p>
    <p style="font-size:14px">
        See the console for details.
    </p>
    <p style="font-size:14px">
        Reload the page to try again.
    </p>
"#))
        });
        // Propagate panic info to the previously registered panic hook
        previous_hook(panic_info);
    }));

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Ok(Box::new(mcmc_demo::TemplateApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");

        // loaded successfully, remove the loading indicator
        get_loading_text().map(|e| e.remove());
        });
}
