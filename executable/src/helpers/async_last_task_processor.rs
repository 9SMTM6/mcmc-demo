use shared::cfg_if_expr;
use std::future::Future;
use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Mutex, Notify};

pub(crate) trait ComputeTask<T> {
    async fn run(&self, value: T);
}

/// Do NOT implement clone on this.
pub struct TaskDispatcher<T> {
    change_notify: Arc<Notify>,
    data: Arc<Mutex<Option<T>>>,
    is_other_end_active: Arc<AtomicBool>,
}

/// Do NOT implement clone on this.
pub struct TaskExecutor<T, C: ComputeTask<T>> {
    change_notify: Arc<Notify>,
    data: Arc<Mutex<Option<T>>>,
    task: C,
    is_other_end_active: Arc<AtomicBool>,
}

#[must_use]
/// If you drop this Factory, the Dispatcher will never be notified that the other end is inactive.
///
/// Can't figure out how to solve that properly. Implementing Drop means I can't move out of this struct.
pub(crate) struct TaskExecutorFactory<T> {
    change_notify: Arc<Notify>,
    data: Arc<Mutex<Option<T>>>,
    is_other_end_active: Arc<AtomicBool>,
}

impl<T> TaskExecutorFactory<T> {
    pub fn attach_task_executor<C: ComputeTask<T>>(self, task: C) -> TaskExecutor<T, C> {
        TaskExecutor {
            task,
            change_notify: self.change_notify,
            data: self.data,
            is_other_end_active: self.is_other_end_active,
        }
    }
}

impl<T> Drop for TaskDispatcher<T> {
    fn drop(&mut self) {
        // Mark sender as inactive
        self.is_other_end_active.store(false, Ordering::SeqCst);
        tracing::info!("Dropped TaskSender");
        // Notify the change, so the runner can finalize
        self.change_notify.notify_one();
    }
}

impl<T, C: ComputeTask<T>> Drop for TaskExecutor<T, C> {
    fn drop(&mut self) {
        // Mark sender as inactive
        self.is_other_end_active.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub enum TaskDispatchError {
    TaskRunnerDropped,
}

// Constructor for the sender/runner system
impl<T> TaskDispatcher<T> {
    // Sends a task update
    pub async fn dispatch_task(&self, new_task: T) -> Result<(), TaskDispatchError> {
        if !self.is_other_end_active.load(Ordering::SeqCst) {
            Err(TaskDispatchError::TaskRunnerDropped)
        } else {
            {
                let mut data_guard = self.data.lock().await;
                *data_guard = Some(new_task);
            }
            self.change_notify.notify_one();
            Ok(())
        }
    }

    pub fn dispatch_task_blocking(&self, new_task: T) -> Result<(), TaskDispatchError> {
        futures::executor::block_on(self.dispatch_task(new_task))
    }
}

#[cfg(feature = "more_debug_impls")]
mod cond_trait_impl {
    use std::fmt::Debug;

    /// A little helper ensuring that the bound type implements debug, if that support was compiled in with `feature = "more_debug_impls"`.
    /// Required since I can't seem to conditionally include a bound.
    pub trait DebugBoundIfCompiled: Debug {}

    impl<T: Debug> DebugBoundIfCompiled for T {}
}
#[cfg(not(feature = "more_debug_impls"))]
mod cond_trait_impl {
    /// A little helper ensuring that the bound type implements debug, if that support was compiled in with `feature = "more_debug_impls"`.
    /// Required since I can't seem to conditionally include a bound.
    pub trait DebugBoundIfCompiled {}

    impl<T> DebugBoundIfCompiled for T {}
}

pub use cond_trait_impl::*;

impl<T, C> TaskExecutor<T, C>
where
    T: DebugBoundIfCompiled,
    C: ComputeTask<T>,
{
    /// Initializes the compute loop
    #[expect(
        clippy::cognitive_complexity,
        reason = "Doesn't seem THAT complex to me, most of the complexity comes from features and/or debugging"
    )]
    pub async fn start_processing_loop(self) {
        // Wait until notified of a task change
        let mut recorded_notify = false;
        loop {
            if !recorded_notify {
                self.change_notify.notified().await;
            }
            recorded_notify = false;
            tracing::debug!("looped");

            if let Some(task_data) = {
                let mut data_guard = self.data.lock().await;
                data_guard.take()
            } {
                cfg_if_expr!(
                    => [feature = "more_debug_impls"]
                    tracing::info!(?task_data, "task received")
                    => [not]
                    tracing::info!("task received")
                );
                tokio::select! {
                    _ = self.change_notify.notified() => {
                        tracing::debug!("New task arrived, re-enter loop");
                        recorded_notify = true;
                        // New task arrived, re-enter loop
                    },
                    _ = self.task.run(task_data) => {
                        // Process the current task
                    },
                }
            } else if !self.is_other_end_active.load(Ordering::SeqCst) {
                tracing::debug!("Finalizing Scheduler, as sending end was closed");
                break;
            } else {
                tracing::error!("Unexpected State");
                crate::cfg_sleep!().await;
            }
        }
    }
}

pub fn get<T>() -> (TaskDispatcher<T>, TaskExecutorFactory<T>) {
    let change_notify = Arc::new(Notify::new());
    let data = Arc::new(Mutex::new(None));
    let is_other_end_active = Arc::new(AtomicBool::new(true));

    (
        TaskDispatcher {
            change_notify: Arc::clone(&change_notify),
            data: Arc::clone(&data),
            is_other_end_active: Arc::clone(&is_other_end_active),
        },
        TaskExecutorFactory {
            change_notify,
            data,
            is_other_end_active,
        },
    )
}

impl<T, F: Future, H, J> ComputeTask<T> for J
where
    F: Future,
    H: FnOnce(T) -> F,
    J: Fn() -> H,
{
    async fn run(&self, value: T) {
        self()(value).await;
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    #[cfg(feature = "more_debug_impls")]
    #[tokio::test]
    #[ignore = "very flaky on CI. IDK why, current theory is its a race condition caused by too few actual threads in CI"]
    async fn executes_task_successfully() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (t_tx, runner) = get();
        let runner = runner.attach_task_executor(move || {
            let tx = tx.clone();
            |_val: ()| async move {
                tx.send(()).await.unwrap();
            }
        });
        let (f, g, h) = tokio::join!(
            tokio::spawn(runner.start_processing_loop()),
            tokio::spawn(async move {
                t_tx.dispatch_task(()).await.unwrap();
                drop(t_tx);
            }),
            #[allow(
                clippy::allow_attributes,
                reason = "This seems cleanest way to do this."
            )]
            #[allow(
                clippy::pattern_type_mismatch,
                reason = "Can't seem to fix this with tokio macro matching"
            )]
            tokio::spawn(async move {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(1200)) => {
                        panic!("Timeout");
                    }
                    Some(_) = rx.recv() => {},
                };
            })
        );
        g.unwrap();
        h.unwrap();
        f.unwrap();
    }

    #[tokio::test]
    async fn finalizes_on_dispatcher_drop() {
        let (t_tx, runner) = get();
        let runner = runner.attach_task_executor(move || |_val: ()| async move {});
        let (f, g) = tokio::join!(
            tokio::spawn(async move {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(200)) => {
                        panic!("Timeout");
                    }
                    _ = runner.start_processing_loop() => {},
                }
            }),
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                drop(t_tx);
            }),
        );
        f.unwrap();
        g.unwrap();
    }

    #[tokio::test]
    async fn errors_on_executor_drop() {
        let (t_tx, runner) = get();
        let runner = runner.attach_task_executor(move || |_val: ()| async move {});
        drop(runner);
        assert!(t_tx.dispatch_task(()).await.is_err());
    }
}
