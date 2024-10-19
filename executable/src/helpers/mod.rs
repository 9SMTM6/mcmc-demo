pub mod async_last_task_processor;
pub mod bg_task;
pub mod gpu_task;
pub mod html_bindings;
pub mod temp_ui_state;

#[macro_export]
macro_rules! cfg_sleep {
    ($duration: expr) => {
        ::shared::cfg_if_expr!(
            => [feature = "debounce_async_loops"]
            ::tokio::time::sleep($duration)
            => [not]
            ::std::future::ready(())
        )
    };
    () => {
        $crate::cfg_sleep!(std::time::Duration::from_secs(1) / 3)
    }
}
