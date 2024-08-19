#[cfg(feature = "profile")]
pub mod backend_panel;
#[cfg(feature = "profile")]
pub mod frame_history;
#[cfg(feature = "tracing")]
pub mod tracing;

#[macro_export]
#[allow(clippy::module_name_repetitions)]
#[allow(unknown_lints)] // not a lint on stable...
#[allow(edition_2024_expr_fragment_specifier)]
macro_rules! profile_scope {
    ($scope_name:expr) => {
        #[cfg(feature = "profile")]
        puffin::profile_scope!($scope_name);
    };
}

#[cfg(all(feature = "profile", not(target_arch = "wasm32")))]
const PUFFIN_URL: &str = "127.0.0.1:8585";

#[cfg(all(feature = "profile", not(target_arch = "wasm32")))]
// puffin server doesn't exist on web.
// On the web we've got tracing_web that installed a tracing subscriber that reports to the [performance api](https://developer.mozilla.org/en-US/docs/Web/API/Performance).
pub fn start_puffin_server() {
    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new(PUFFIN_URL) {
        Ok(puffin_server) => {
            // I could go throught the trouble of managing to hold this in temporary storage somewhere, but frankly I dont think its worth the effort.
            // Having a detached thread somewhere that maanges it should be fine.
            wasm_thread::spawn(|| {
                let viewer_process = std::process::Command::new("puffin_viewer")
                    .arg("--url")
                    .arg(PUFFIN_URL)
                    .spawn();

                match viewer_process {
                    Ok(mut viewer_process) => {
                        let viewer_res = viewer_process.wait();
                        if let Err(err) = viewer_res {
                            eprintln!("{err}");
                        }
                    },
                    Err(err) => {
                        eprintln!("Failed to start puffin_viewer: {err}

Try:  cargo install puffin_viewer && puffin_viewer --url {PUFFIN_URL}")
                    },
                };

                drop(puffin_server);
                puffin::set_scopes_on(false);
            });
        }
        Err(err) => {
            eprintln!("Failed to start puffin server: {err}");
        }
    };
}
