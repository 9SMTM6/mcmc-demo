use tr_sub::layer::SubscriberExt as _;
use tracing::{self, Subscriber};
use tracing_log;
use tracing_subscriber as tr_sub;

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
        .with(tr_sub::fmt::layer())
}

pub fn set_default_and_redirect_log(subscriber: impl Subscriber + Send + Sync) {
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans.
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    // Redirect all `log`'s events to our subscriber
    tracing_log::LogTracer::init().expect("Failed to set logger");
}