mod helpers;

pub use helpers::{BackgroundTaskManager, TaskProgress};

pub struct BgTaskHandle<Final = ()> {
    /// Needs to be saved to keep the thread alive on web (?),
    /// Cant be saved in temporary storage because of the Copy requirements on IdTypeMap::insert_temp.
    /// Works like this for the moment, since theres only one of these.
    background_thread: wasm_thread::JoinHandle<Final>,
    pub communicate: BackgroundTaskManager,
}

pub trait BgTask {
    type Final;

    fn execute(self, communicate: BackgroundTaskManager) -> Self::Final
    where
        Self: Sized;
}

impl<Final, F: FnOnce(BackgroundTaskManager) -> Final + Sized> BgTask for F {
    type Final = Final;

    fn execute(self, communicate: BackgroundTaskManager) -> Self::Final {
        let ret = self(communicate.clone());
        communicate.mark_as_finished();
        ret
    }
}

impl<Final: Send + 'static> BgTaskHandle<Final> {
    pub fn new(task: impl BgTask<Final = Final> + Send + 'static, total_steps: usize) -> Self {
        let communicate = BackgroundTaskManager::new(total_steps);
        let background_thread = wasm_thread::spawn({
            let communicate = communicate.clone();
            move || task.execute(communicate)
        });
        Self {
            background_thread,
            communicate,
        }
    }

    #[must_use]
    pub fn get_progress(&self) -> TaskProgress {
        self.communicate.current_progress()
    }

    /// # Panics
    ///
    /// Panics if task is not finished or panicked
    #[must_use]
    pub fn get_value(self) -> Final {
        // Not the reason for the breakdown after a panic in thread on web.
        assert!(matches!(self.get_progress(), TaskProgress::Finished));
        self.background_thread.join().expect(
            "
While a join on an unfinished task is problematic on the web,
it should be fine if the thread is already gone.
Which is supposed to be the case here.
This is the reason I do this at all, if these assumptions turn out to be faulty I want to know.

Also, obvously if the thread panicked, I want to propagate it too.
        ",
        )
    }
}

/// This serves as a proxy for the BgTaskHandle getting dropped.
/// I can't implement drop on that, otherwise I can join in the JoinHandle, as moving out of a type is forbidden if that type implements drop.
impl Drop for BackgroundTaskManager {
    fn drop(&mut self) {
        self.request_abort();
    }
}
