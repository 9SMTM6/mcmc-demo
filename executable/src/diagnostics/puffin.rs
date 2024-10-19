#[macro_export]
macro_rules! profile_scope {
    ($scope_name:expr) => {
        #[cfg(all(feature = "performance_profile", not(target_arch = "wasm32")))]
        puffin::profile_scope!($scope_name);
    };
}

#[cfg(all(not(target_arch = "wasm32"), feature = "performance_profile"))]
const PUFFIN_URL: &str = "127.0.0.1:8585";

#[cfg(all(not(target_arch = "wasm32"), feature = "performance_profile"))]
// puffin server doesn't exist on web.
// On the web we've got tracing_web that installed a tracing subscriber that reports to the [performance api](https://developer.mozilla.org/en-US/docs/Web/API/Performance).
pub fn start_puffin_server() {
    use std::process::Child;

    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new(PUFFIN_URL) {
        Ok(puffin_server) => {
            // I could go through the trouble of managing to hold this in temporary storage somewhere, but frankly I dont think its worth the effort.
            // Having a detached thread somewhere that maanges it should be fine.
            wasm_thread::spawn(|| {
                let viewer_process = std::process::Command::new("puffin_viewer")
                    .arg("--url")
                    .arg(PUFFIN_URL)
                    .spawn();

                #[allow(clippy::needless_lifetimes, reason = "Nightly stable mismatch")]
                match viewer_process {
                    Ok(mut viewer_process) => {
                        // TODO: properly handle that stuff, so that puffin closes on exit. TO do that I need to move away from the detached thread workflow.
                        struct KillOnClose<'a> {
                            process: &'a mut Child,
                        }
                        impl<'a> KillOnClose<'a> {
                            fn wait(&mut self) -> Result<std::process::ExitStatus, std::io::Error> {
                                self.process.wait()
                            }
                        }
                        impl<'a> Drop for KillOnClose<'a> {
                            fn drop(&mut self) {
                                drop(self.process.kill());
                            }
                        }
                        let mut handle = KillOnClose {
                            process: &mut viewer_process,
                        };
                        let viewer_res: Result<std::process::ExitStatus, std::io::Error> =
                            handle.wait();
                        if let Err(err) = viewer_res {
                            eprintln!("{err}");
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "Failed to start puffin_viewer: {err}

Try:  cargo install puffin_viewer && puffin_viewer --url {PUFFIN_URL}"
                        );
                    }
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
