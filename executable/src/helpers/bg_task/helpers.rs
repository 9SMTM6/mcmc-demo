use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{self, atomic};

#[derive(Clone)]
pub struct BackgroundTaskManager {
    abort: AbortFlag,
    progress: StepProgress,
}

#[derive(Debug)]
pub enum TaskProgress {
    Pending(f32),
    Finished,
}

impl BackgroundTaskManager {
    pub(super) fn new(total_steps: usize) -> Self {
        Self {
            abort: AbortFlag::new(),
            progress: StepProgress::new(total_steps),
        }
    }

    /// Used to inform main thread of current progress, and to be informed whether this task should be aborted.
    #[must_use]
    pub fn update_progress_and_check_abort(&mut self, steps_completed: usize) -> bool {
        self.progress.update_progress(steps_completed);
        self.abort.is_abort_requested()
    }

    #[must_use]
    pub(super) fn current_progress(&self) -> TaskProgress {
        self.progress.current_progress()
    }

    pub(super) fn request_abort(&mut self) {
        self.abort.request_abort();
    }

    pub(super) fn mark_as_finished(&self) {
        self.progress.mark_as_finished();
    }
}

#[derive(Clone)]
struct AbortFlag(sync::Arc<AtomicBool>);

impl AbortFlag {
    fn new() -> Self {
        Self(sync::Arc::new(false.into()))
    }

    fn request_abort(&self) {
        self.0.store(true, atomic::Ordering::Release);
    }

    fn is_abort_requested(&self) -> bool {
        self.0.load(atomic::Ordering::Acquire)
    }
}

#[derive(Clone)]
struct StepProgress {
    max_steps: usize,
    progress_data: sync::Arc<(AtomicUsize, AtomicBool)>,
}

impl StepProgress {
    fn new(max_steps: usize) -> Self {
        Self {
            max_steps,
            progress_data: sync::Arc::new((0.into(), false.into())),
        }
    }

    // fn increment(&mut self, added_steps: usize) {
    //     let prev = self.steps_completed.fetch_add(added_steps, atomic::Ordering::SeqCst);
    //     assert!(prev + added_steps <= self.total_steps);
    // }

    fn update_progress(&mut self, value: usize) {
        assert!(value <= self.max_steps);
        self.progress_data.0.store(value, atomic::Ordering::Release);
    }

    fn current_progress(&self) -> TaskProgress {
        if self.progress_data.1.load(atomic::Ordering::Acquire) {
            TaskProgress::Finished
        } else {
            let current = self.progress_data.0.load(atomic::Ordering::Acquire);
            let progress = current as f32 / self.max_steps as f32;
            assert!(progress <= 1.0);
            TaskProgress::Pending(progress)
        }
    }

    fn mark_as_finished(&self) {
        self.progress_data.1.store(true, atomic::Ordering::Release);
    }
}
