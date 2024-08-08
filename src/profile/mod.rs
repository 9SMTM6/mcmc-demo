#[cfg(feature = "profile")]
pub mod backend_panel;
#[cfg(feature = "profile")]
pub mod frame_history;
#[cfg(feature = "tracing")]
pub mod tracing;

#[macro_export]
#[allow(clippy::module_name_repetitions, reason = "makes autoimport nicer")]
#[allow(edition_2024_expr_fragment_specifier)]
macro_rules! profile_scope {
    ($scope_name:expr) => {
        #[cfg(feature = "profile")]
        puffin::profile_scope!($scope_name);
    };
}

#[cfg(feature = "profile")]
mod if_featured {
    #[allow(
        clippy::missing_const_for_fn,
        reason = "false positive if compiling for wasm"
    )]
    pub fn start_puffin_server() {
        // puffin server doesnt exist on web, so make that a noop there.
        // We've got tracing_web that installed a tracing subscriber that reports to the [performance api](https://developer.mozilla.org/en-US/docs/Web/API/Performance).
        #[cfg(not(target_arch = "wasm32"))]
        {
            puffin::set_scopes_on(true); // tell puffin to collect data

            match puffin_http::Server::new("127.0.0.1:8585") {
                Ok(puffin_server) => {
                    eprintln!(
                        "Run:  cargo install puffin_viewer && puffin_viewer --url 127.0.0.1:8585"
                    );

                    std::process::Command::new("puffin_viewer")
                        .arg("--url")
                        .arg("127.0.0.1:8585")
                        .spawn()
                        .ok();

                    // We can store the server if we want, but in this case we just want
                    // it to keep running. Dropping it closes the server, so let's not drop it!
                    #[allow(clippy::mem_forget, reason = "we want to keep it running")]
                    std::mem::forget(puffin_server);
                }
                Err(err) => {
                    eprintln!("Failed to start puffin server: {err}");
                }
            };
        }
    }
}

#[cfg(feature = "profile")]
pub use if_featured::*;
