mod helpers;

pub use helpers::BgCommunicate;

pub struct BgTaskHandle<Final = ()> {
    /// Needs to be saved to keep the thread alive on web (?),
    /// Cant be saved in temporary storage because of the Copy requirements on IdTypeMap::insert_temp.
    /// Works like this for the moment, since theres only one of these.
    background_thread: Option<wasm_thread::JoinHandle<Final>>,
    pub communicate: BgCommunicate,
}

pub trait BgTask {
    type Final;

    fn execute(self, communicate: BgCommunicate) -> Self::Final
    where
        Self: Sized;
}

impl<Final, F: FnOnce(BgCommunicate) -> Final + Sized> BgTask for F {
    type Final = Final;

    fn execute(self, communicate: BgCommunicate) -> Self::Final {
        self(communicate)
    }
}

pub enum Progress<Final> {
    Pending(f32),
    Finished(Final),
}

impl<Final: Send + 'static> BgTaskHandle<Final> {
    pub fn new(task: impl BgTask<Final = Final> + Send + 'static, total_steps: usize) -> Self {
        let communicate = BgCommunicate::new(total_steps);
        let background_thread = Some(wasm_thread::spawn({
            let communicate = communicate.clone();
            move || task.execute(communicate)
        }));
        Self {
            background_thread,
            communicate,
        }
    }

    #[must_use]
    pub fn get_progress(&mut self) -> Progress<Option<Final>> {
        let progress = self.communicate.get_progress();
        if progress < 1.0 {
            Progress::Pending(progress)
        } else {
            // the way I understand it, this join should be fine even on the web, as long as the task is actually finished.
            if self.background_thread.is_some() {
                Progress::Finished(self.background_thread.take().unwrap().join().ok())
            } else {
                Progress::Finished(None)
            }
        }
    }
}

impl<T> Drop for BgTaskHandle<T> {
    fn drop(&mut self) {
        self.communicate.abort();
    }
}
