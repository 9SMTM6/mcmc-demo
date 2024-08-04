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

#[cfg(target_arch = "wasm32")]
mod wasm_helpers {
    use web_sys::wasm_bindgen::JsCast;
    pub(super) fn get_canvas_element_by_id(canvas_id: &str) -> Option<web_sys::HtmlCanvasElement> {
        let document = web_sys::window()?.document()?;
        let canvas = document.get_element_by_id(canvas_id)?;
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
    }

    pub(super) fn get_canvas_element_by_id_or_die(canvas_id: &str) -> web_sys::HtmlCanvasElement {
        get_canvas_element_by_id(canvas_id)
            .unwrap_or_else(|| panic!("Failed to find canvas with id {canvas_id:?}"))
    }

    pub(super) fn get_issue_text() -> Option<web_sys::Element> {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("issue_text"))
    }
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_helpers::*;
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let webgpu_supported = web_sys::window().unwrap().navigator().gpu().is_truthy();

    let web_options = eframe::WebOptions::default();

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Show in the HTML that start has failed
        let Some(issue_el_ref) = get_issue_text() else {
            unreachable!()
        };

        issue_el_ref.set_inner_html(&format!(
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
"#
        ));
        // Propagate panic info to the previously registered panic hook
        previous_hook(panic_info);
    }));

    if webgpu_supported {
        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    get_canvas_element_by_id_or_die("the_canvas_id"),
                    web_options,
                    Box::new(|cc| Ok(Box::new(mcmc_demo::McmcDemo::new(cc)))),
                )
                .await
                .expect("failed to start eframe");

            // loaded successfully, remove the loading indicator
            if let Some(e) = get_issue_text() { e.remove() };
        });
    } else {
        let Some(loading_el_ref) = get_issue_text() else {
            unreachable!()
        };

        loading_el_ref.set_inner_html(
            r#"
    <p> This application currently requires WebGPU. </p>
    <p> At the time of writing, this means Chrome (or Chromium). </p>
    <p> On Linux, you also need to start Chrome with --enable-unsafe-webgpu or set the appropriate command flag in its <a style="color: #ffffff" href="chrome://flags/#enable-unsafe-webgpu">settings</a>. </p>
    <p> Alternatively, you can download an executable from the <a style="color: #ffffff" href="https://github.com/9SMTM6/mcmc-demo/releases">Github release</a> page. </p>
"#
        );
    }
}
