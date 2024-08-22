use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{self, atomic};

#[derive(Clone)]
pub struct BgCommunicate {
    abort: AbortSignal,
    progress: StepProgress,
}

#[derive(Debug)]
pub enum Progress {
    Pending(f32),
    Finished,
}

impl BgCommunicate {
    pub(super) fn new(total_steps: usize) -> Self {
        Self {
            abort: AbortSignal::new(),
            progress: StepProgress::new(total_steps),
        }
    }

    /// Used to inform main thread of current progress, and to be informed whether this task should be aborted.
    #[must_use]
    pub fn checkup_bg(&mut self, current_steps: usize) -> bool {
        self.progress.set_progress(current_steps);
        self.abort.should_abort()
    }

    #[must_use]
    pub(super) fn get_progress(&self) -> Progress {
        self.progress.get_progress()
    }

    pub(super) fn abort(&mut self) {
        self.abort.abort();
    }

    pub(super) fn finished(&self) {
        self.progress.finished();
    }
}

#[derive(Clone)]
struct AbortSignal(sync::Arc<AtomicBool>);

impl AbortSignal {
    fn new() -> Self {
        Self(sync::Arc::new(false.into()))
    }

    fn abort(&self) {
        self.0.store(true, atomic::Ordering::Release);
    }

    fn should_abort(&self) -> bool {
        self.0.load(atomic::Ordering::Acquire)
    }
}

#[derive(Clone)]
struct StepProgress {
    total_steps: usize,
    current_steps_or_done: sync::Arc<(AtomicUsize, AtomicBool)>,
}

impl StepProgress {
    fn new(total_steps: usize) -> Self {
        Self {
            total_steps,
            current_steps_or_done: sync::Arc::new((0.into(), false.into())),
        }
    }

    // fn increment(&mut self, added_steps: usize) {
    //     let prev = self.current_steps.fetch_add(added_steps, atomic::Ordering::SeqCst);
    //     assert!(prev + added_steps <= self.total_steps);
    // }

    fn set_progress(&mut self, value: usize) {
        assert!(value <= self.total_steps);
        self.current_steps_or_done
            .0
            .store(value, atomic::Ordering::Release);
    }

    fn get_progress(&self) -> Progress {
        if self.current_steps_or_done.1.load(atomic::Ordering::Acquire) {
            Progress::Finished
        } else {
            let current = self.current_steps_or_done.0.load(atomic::Ordering::Acquire);
            let progress = current as f32 / self.total_steps as f32;
            assert!(progress <= 1.0);
            Progress::Pending(progress)
        }
    }

    fn finished(&self) {
        self.current_steps_or_done
            .1
            .store(true, atomic::Ordering::Release);
    }
}
