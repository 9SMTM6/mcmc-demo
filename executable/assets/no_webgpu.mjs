// check for webgpu compatibility, if not found, display the warning message.
const show_warning = () => {
    document.getElementById("no_webgpu")?.removeAttribute("hidden");
    document.getElementById("egui_canvas")?.remove();
    document.getElementById("panic_el")?.remove();
    document.getElementById("loading_el")?.remove();
};
navigator.gpu?.requestAdapter()?.catch(show_warning) ?? show_warning()
