use std::future::Future;
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::{Mutex, Notify};

pub(crate) trait ComputeTask<T> {
    async fn run(&self, value: T);
}

/// Do NOT implement clone on this.
pub struct TaskSender<T> {
    change_notify: Arc<Notify>,
    data: Arc<Mutex<Option<T>>>,
    is_other_end_active: Arc<AtomicBool>,
}

/// Do NOT implement clone on this.
pub struct TaskRunner<T, C: ComputeTask<T>> {
    change_notify: Arc<Notify>,
    data: Arc<Mutex<Option<T>>>,
    task: C,
    is_other_end_active: Arc<AtomicBool>,
}

#[must_use]
/// If you drop this Factory, the Sender will never be notified that the other end is inactive.
///
/// Can't figure out how to solve that properly. Implementing Drop means I can't move out of this struct.
pub(crate) struct TaskRunnerFactory<T> {
    change_notify: Arc<Notify>,
    data: Arc<Mutex<Option<T>>>,
    is_other_end_active: Arc<AtomicBool>,
}

impl<T> TaskRunnerFactory<T> {
    pub fn bind_task<C: ComputeTask<T>>(self, task: C) -> TaskRunner<T, C> {
        TaskRunner {
            task,
            change_notify: self.change_notify,
            data: self.data,
            is_other_end_active: self.is_other_end_active,
        }
    }
}

impl<T> Drop for TaskSender<T> {
    fn drop(&mut self) {
        // Mark sender as inactive
        self.is_other_end_active.store(false, Ordering::SeqCst);
        tracing::info!("Dropped TaskSender");
        // Notify the change, so the runner can finalize
        self.change_notify.notify_one();
    }
}

impl<T, C: ComputeTask<T>> Drop for TaskRunner<T, C> {
    fn drop(&mut self) {
        // Mark sender as inactive
        self.is_other_end_active.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub enum TaskSendError {
    TaskRunnerDropped,
}

// Constructor for the sender/runner system
impl<T> TaskSender<T> {
    // Sends a task update
    pub async fn send_update(&self, new_task: T) -> Result<(), TaskSendError> {
        if !self.is_other_end_active.load(Ordering::SeqCst) {
            Err(TaskSendError::TaskRunnerDropped)
        } else {
            {
                let mut data_guard = self.data.lock().await;
                *data_guard = Some(new_task);
            }
            self.change_notify.notify_one();
            Ok(())
        }
    }

    pub fn send_update_blocking(&self, new_task: T) -> Result<(), TaskSendError> {
        futures::executor::block_on(self.send_update(new_task))
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
    use std::fmt::Debug;

    /// A little helper ensuring that the bound type implements debug, if that support was compiled in with `feature = "more_debug_impls"`.
    /// Required since I can't seem to conditionally include a bound.
    pub trait DebugBoundIfCompiled {}

    impl<T> DebugBoundIfCompiled for T {}
}

pub use cond_trait_impl::*;

impl<T, C> TaskRunner<T, C>
where
    T: DebugBoundIfCompiled,
    C: ComputeTask<T>,
{
    /// Initializes the compute loop
    pub async fn run_compute_loop(self) {
        // Wait until notified of a task change
        let mut recorded_notify = false;
        loop {
            if !recorded_notify {
                self.change_notify.notified().await;
            }
            recorded_notify = false;
            tracing::debug!("looped");

            if let Some(task) = {
                let mut data_guard = self.data.lock().await;
                data_guard.take()
            } {
                #[cfg(feature = "more_debug_impls")]
                tracing::info!(?task, "task received");
                #[cfg(not(feature = "more_debug_impls"))]
                tracing::info!("task received");
                tokio::select! {
                    _ = self.change_notify.notified() => {
                        tracing::debug!("New task arrived, re-enter loop");
                        recorded_notify = true;
                        // New task arrived, re-enter loop
                    },
                    _ = self.task.run(task) => {
                        // Process the current task
                    },
                }
            } else if !self.is_other_end_active.load(Ordering::SeqCst) {
                tracing::debug!("Finalizing Scheduler, as sending end was closed");
                break;
            } else {
                tracing::error!("Unexpected State");
                #[cfg(feature = "debounce_async_loops")]
                tokio::time::sleep(std::time::Duration::from_secs(1) / 3).await;
            }
        }
    }
}

pub fn get<T>() -> (TaskSender<T>, TaskRunnerFactory<T>) {
    let change_notify = Arc::new(Notify::new());
    let data = Arc::new(Mutex::new(None));
    let is_other_end_active = Arc::new(AtomicBool::new(true));

    (
        TaskSender {
            change_notify: Arc::clone(&change_notify),
            data: Arc::clone(&data),
            is_other_end_active: Arc::clone(&is_other_end_active),
        },
        TaskRunnerFactory {
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
    async fn runs_task_successfully() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (t_tx, runner) = get();
        let runner = runner.bind_task(move || {
            let tx = tx.clone();
            |_val: ()| async move {
                tx.send(()).await.unwrap();
            }
        });
        let (f, g, h) = tokio::join!(
            tokio::spawn(runner.run_compute_loop()),
            tokio::spawn(async move {
                t_tx.send_update(()).await.unwrap();
                drop(t_tx);
            }),
            tokio::spawn(async move {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(200)) => {
                        panic!("Timeout");
                    }
                    Some(()) = rx.recv() => {},
                };
            })
        );
        g.unwrap();
        h.unwrap();
        f.unwrap();
    }

    #[tokio::test]
    async fn cancels_on_sender_drop() {
        let (t_tx, runner) = get();
        let runner = runner.bind_task(move || |_val: ()| async move {});
        let (f, g) = tokio::join!(
            tokio::spawn(async move {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(200)) => {
                        panic!("Timeout");
                    }
                    _ = runner.run_compute_loop() => {},
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
    async fn errors_on_runner_drop() {
        let (t_tx, runner) = get();
        let runner = runner.bind_task(move || |_val: ()| async move {});
        drop(runner);
        assert!(t_tx.send_update(()).await.is_err());
    }
}
