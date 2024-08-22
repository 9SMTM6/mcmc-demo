#![cfg(target_arch = "wasm32")]

use web_sys::{wasm_bindgen::JsCast, Element};

const CANVAS_ID: &str = "egui_canvas";

pub fn get_element_by_id(id: &str) -> Option<web_sys::Element> {
    web_sys::window()?.document()?.get_element_by_id(id)
}

/// # Panics
///
/// If the element cant be found by its ID (it should be in `index.html` fro mthe start), or if that element isnt a canvas.
pub fn get_egui_canvas() -> web_sys::HtmlCanvasElement {
    get_element_by_id(CANVAS_ID)
        .expect("Unable to find root canvas")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("Root element is no canvas")
}

pub fn get_oob_text_el() -> Option<web_sys::HtmlElement> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("oob_text"))
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
}

pub fn remove_el_if_present(id: &str) {
    get_element_by_id(id).as_ref().map(Element::remove);
}

pub fn remove_loading_state() {
    if let Some(el) = get_oob_text_el().as_ref() {
        el.set_inner_text("");
    };
    remove_el_if_present("loading_animation");
}

pub(super) fn remove_canvas() {
    remove_el_if_present(CANVAS_ID);
}

pub fn display_failing_wgpu_info() {
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

pub fn try_display_panic(panic_info: &std::panic::PanicHookInfo<'_>) {
    try_display_panic_str(&panic_info.to_string());
}

#[allow(clippy::missing_panics_doc)]
pub fn try_display_panic_str(panic_info: &str) {
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
        remove_loading_state();
        remove_canvas();
    } else {
        // not sure what panicing here does, but oh well.
        assert!(wasm_thread::is_web_worker_thread());
        // TODO: Find some way to transport these to the main thread.
        // JoinHandle::join only returns the value that was panicked with.
        // Location is difficult to extract.
        // Perhaps just pass a stringified panic somewhere.
    };
}
