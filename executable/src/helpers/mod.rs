pub mod async_last_task_processor;
pub mod bg_task;
pub mod gpu_task;
pub mod html_bindings;
pub mod temp_ui_state;

#[macro_export]
macro_rules! cfg_sleep {
    ($duration: expr) => {
        ::shared::cfg_if_expr!(
            => [all(feature = "debounce_async_loops", not(target_arch = "wasm32"))]
            ::tokio::time::sleep($duration)
            => [not]
            ::std::future::ready(())
        )
    };
    () => {
        $crate::cfg_sleep!(std::time::Duration::from_secs(1) / 3)
    }
}

/// This function emits warnings when a feature and/or target configuration is chosen, that will not work as expected.
#[allow(
    clippy::missing_const_for_fn,
    reason = "False positives depending on configuration"
)]
pub fn warn_feature_config() {
    #[cfg(not(all(feature = "debounce_async_loops", not(target_arch = "wasm32"))))]
    tracing::warn!(
        r#"Feature "debounce_async_loops" enabled, however other configuration disables this implicitly"#
    );

    #[cfg(not(all(feature = "tokio_console", tokio_unstable, not(target_arch = "wasm32"))))]
    tracing::warn!(
        r#"Feature "tokio_console" enabled, however other configuration disables this implicitly"#
    );
}
