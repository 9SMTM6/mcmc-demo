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
mod html_bindings {
    use web_sys::{wasm_bindgen::JsCast, Element};

    const CANVAS_ID: &'static str = "egui_canvas";

    pub(super) fn get_element_by_id(id: &str) -> Option<web_sys::Element> {
        web_sys::window()?.document()?.get_element_by_id(id)
    }

    pub(super) fn get_egui_canvas() -> web_sys::HtmlCanvasElement {
        get_element_by_id(&CANVAS_ID).expect("Unable to find root canvas").dyn_into::<web_sys::HtmlCanvasElement>().ok().expect("Root element is no canvas")
    }

    pub(super) fn get_oob_text_el() -> Option<web_sys::Element> {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("oob_communication"))
    }

    pub(super) fn remove_el_if_present(id: &str) {
        get_element_by_id(id).as_ref().map(Element::remove);
    }

    pub(super) fn remove_loading_el() {
        remove_el_if_present("loading_animation");
    }

    pub(super) fn remove_canvas() {
        remove_el_if_present(CANVAS_ID);
    }

    pub(super) fn display_failing_wgpu_info() {
        let Some(oob_el_ref) = get_oob_text_el() else {
            unreachable!("Could not find warning element");
        };

        oob_el_ref.set_inner_html(
            r#"
    <p> This application currently requires WebGPU. </p>
    <p> At the time of writing, this means Chrome (or Chromium based browsers). </p>
    <p> On Linux, you also need to start Chrome with --enable-unsafe-webgpu or set the appropriate command flag in its <a style="color: #ffffff" href="chrome://flags/#enable-unsafe-webgpu">settings</a>. </p>
    <p> Alternatively, you can download an executable from the <a style="color: #ffffff" href="https://github.com/9SMTM6/mcmc-demo/releases">Github release</a> page. </p>
"#
        );
    }

    pub(super) fn try_display_panic(panic_info: &std::panic::PanicHookInfo<'_>) {
        if let Some(oob_el_ref) = get_oob_text_el() {
            oob_el_ref.set_inner_html(&format!(
                r#"
        <p> The app has crashed.</p>
        <p style="font-size:12px">
            <div style="background: black; color=white; font-family: monospace; text-align: left">{panic_info}</div>
        </p>
        <p style="font-size:14px">
            See the developer console for more details.
        </p>
        <p style="font-size:14px">
            Reload the page to try again.
        </p>
    "#
            ));
            remove_loading_el();
            remove_canvas();
        };
    }
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use html_bindings::*;

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
                            .await {
                Ok(_) => {},
                Err(err) => {
                    let fmt_err = format!("{err:?}");
                    if fmt_err.contains("wgpu") {
                        display_failing_wgpu_info();
                    } else {
                        panic!("{fmt_err}")
                    }
                },
            };
        });
    } else {
        display_failing_wgpu_info();
    }
    remove_loading_el();
}
