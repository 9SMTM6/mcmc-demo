use tr_sub::layer::SubscriberExt as _;
use tracing::{self, Subscriber};
use tracing_log;
use tracing_subscriber::{self as tr_sub, fmt::time::UtcTime};

#[cfg(target_arch = "wasm32")]
pub fn is_chromium() -> bool {
    let user_agent = web_sys::window()
        .and_then(|win| win.navigator().user_agent().ok())
        .unwrap_or_else(|| String::new())
        .to_lowercase();

    user_agent.contains("chrom")
}

pub fn define_subscriber(
    default_log_level: Option<&str>,
) -> impl tracing::Subscriber + Send + Sync {
    // The `with` method is provided by `SubscriberExt`, an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    tr_sub::Registry::default()
        // We are falling back to printing all spans at info-level or above
        // if the RUST_LOG environment variable has not been set.
        .with(
            tr_sub::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tr_sub::EnvFilter::new(default_log_level.unwrap_or("info"))),
        )
        .with({
            let base = tr_sub::fmt::layer().with_timer(UtcTime::rfc_3339());
            #[cfg(target_arch = "wasm32")]
            let used = base
                .with_ansi(is_chromium()) // chromium supports ANSI, Firefox does not seem to.
                .with_writer(tracing_web::MakeWebConsoleWriter::new());
            #[cfg(not(target_arch = "wasm32"))]
            let used = base;
            used
        })
        .with({
            #[cfg(target_arch = "wasm32")]
            let used = tracing_web::performance_layer()
                .with_details_from_fields(tr_sub::fmt::format::Pretty::default());
            #[cfg(not(target_arch = "wasm32"))]
            let used = tr_sub::layer::Identity::new();
            used
        })
}

#[allow(clippy::missing_panics_doc)]
pub fn set_default_and_redirect_log(subscriber: impl Subscriber + Send + Sync) {
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans.
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    // Redirect all `log`'s events to our subscriber
    tracing_log::LogTracer::init().expect("Failed to set logger");
}
