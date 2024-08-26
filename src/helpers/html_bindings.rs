#![cfg(target_arch = "wasm32")]

use web_sys::{wasm_bindgen::JsCast, Element};

const CANVAS_ID: &str = "egui_canvas";

pub fn get_element_by_id(id: &str) -> Option<web_sys::Element> {
    web_sys::window()?.document()?.get_element_by_id(id)
}

/// # Panics
///
/// If the element cant be found by its ID (it should be in `index.html` from the start), or if that element isnt a canvas.
pub fn get_egui_canvas() -> web_sys::HtmlCanvasElement {
    get_element_by_id(CANVAS_ID)
        .expect("Unable to find root canvas")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("Root element is no canvas")
}

pub fn remove_el_if_present(id: &str) {
    get_element_by_id(id).as_ref().map(Element::remove);
}

pub fn remove_loading_state() {
    if let Some(el) = get_element_by_id("loading_el") {
        el.set_inner_html("");
        el.set_text_content(None);
    }
}

pub(super) fn remove_canvas() {
    remove_el_if_present(CANVAS_ID);
}

/// # Panics
///
/// If the element cant be found by its ID (it should be in `index.html` from the start), or if removal fails
pub fn show_element_by_id(id: &str) {
    get_element_by_id(id)
        .map(|el| el.remove_attribute("hidden"))
        .unwrap()
        .unwrap();
}

pub fn try_display_panic(panic_info: &std::panic::PanicHookInfo<'_>) {
    try_display_panic_str(&panic_info.to_string());
}

#[allow(clippy::missing_panics_doc)]
pub fn try_display_panic_str(panic_info: &str) {
    if let Some(el) = get_element_by_id("panic_message") {
        el.set_text_content(Some(panic_info));
        show_element_by_id("panic_el");
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
