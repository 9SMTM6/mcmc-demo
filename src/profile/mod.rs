#[macro_export]
macro_rules! profile_scope {
    ($scope_name:expr) => {
        #[cfg(feature = "profile")]
        puffin::profile_scope!($scope_name);
    };
}

#[cfg(feature = "profile")]
mod if_featured {
    pub fn start_puffin_server() {
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
                #[allow(clippy::mem_forget)]
                std::mem::forget(puffin_server);
            }
            Err(err) => {
                eprintln!("Failed to start puffin server: {err}");
            }
        };
    }
}

#[cfg(feature = "profile")]
pub use if_featured::*;
