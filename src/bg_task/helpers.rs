use std::sync::{self, atomic};
use std::sync::atomic::{AtomicBool, AtomicUsize};

#[derive(Clone)]
pub struct BgCommunicate {
    abort: AbortSignal,
    progress: StepProgress,
}

impl BgCommunicate {
    pub(super) fn new(total_steps: usize) -> Self {
        Self {
            abort: AbortSignal::new(),
            progress: StepProgress::new(total_steps)
        }
    }

    /// Used to inform main thread of current progress, and to be informed whether this task should be aborted. 
    #[must_use]
    pub fn checkup_bg(&mut self, current_steps: usize) -> bool {
        self.progress.set_progress(current_steps);
        self.abort.should_abort()
    }

    #[must_use]
    pub(super) fn get_progress(&self) -> f32 {
        self.progress.get_progress()
    }

    pub(super) fn abort(&mut self) {
        self.abort.abort()
    }
}

#[derive(Clone)]
struct AbortSignal(sync::Arc<AtomicBool>);

impl AbortSignal {
    fn new() -> Self {
        Self(sync::Arc::new(false.into()))
    }

    fn abort(&self) {
        self.0.store(true, atomic::Ordering::SeqCst);
    }

    fn should_abort(&self) -> bool {
        self.0.load(atomic::Ordering::SeqCst)
    }
}


#[derive(Clone)]
struct StepProgress {
    total_steps: usize,
    current_steps: sync::Arc<AtomicUsize>,
}

impl StepProgress {
    fn new(total_steps: usize) -> Self {
        Self {
            total_steps,
            current_steps: sync::Arc::new(0.into()),
        }
    }

    // fn increment(&mut self, added_steps: usize) {
    //     let prev = self.current_steps.fetch_add(added_steps, atomic::Ordering::SeqCst);
    //     assert!(prev + added_steps <= self.total_steps);
    // }

    fn set_progress(&mut self, value: usize) {
        assert!(value <= self.total_steps);
        self.current_steps.store(value, atomic::Ordering::SeqCst);
    }

    fn get_progress(&self) -> f32 {
        let current = self.current_steps.load(atomic::Ordering::SeqCst);
        current as f32 / self.total_steps as f32
    }
}