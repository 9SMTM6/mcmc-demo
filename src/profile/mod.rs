#[cfg(feature = "profile")]
pub mod backend_panel;
#[cfg(feature = "profile")]
pub mod frame_history;
#[cfg(feature = "tracing")]
pub mod tracing;

#[macro_export]
#[allow(clippy::module_name_repetitions)]
#[allow(unknown_lints)] // not a lint of stable...
#[allow(edition_2024_expr_fragment_specifier)]
macro_rules! profile_scope {
    ($scope_name:expr) => {
        #[cfg(feature = "profile")]
        puffin::profile_scope!($scope_name);
    };
}

const PUFFIN_URL: &str = "127.0.0.1:8585";

#[cfg(feature = "profile")]
// puffin server doesn't exist on web.
// On the web we've got tracing_web that installed a tracing subscriber that reports to the [performance api](https://developer.mozilla.org/en-US/docs/Web/API/Performance).
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::missing_const_for_fn)]
pub fn start_puffin_server() {
    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new(PUFFIN_URL) {
        Ok(puffin_server) => {
            let res = std::process::Command::new("puffin_viewer")
            .arg("--url")
            .arg(PUFFIN_URL)
            .spawn()
            .map_err(|_| format!("puffin_viewer not found.
                
Run:  cargo install puffin_viewer && puffin_viewer --url {PUFFIN_URL}
            "));

            if let Some(msg) = res.err() {
                eprintln!("{msg}")
            }

            // We can store the server if we want, but in this case we just want
            // it to keep running. Dropping it closes the server, so let's not drop it!
            #[allow(clippy::mem_forget)]
            std::mem::forget(puffin_server);
        }
        Err(err) => {
            eprintln!("Failed to start puffin server: {err}");
        }
    };
}
