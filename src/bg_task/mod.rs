use std::sync::mpsc;

pub struct BgTaskHandle<Final = (), FromTaskMsg = f32, ToTaskMsg = (), const FROM_CHANNEL_SIZE: usize = 0, const TO_CHANNEL_SIZE: usize = 0> {
    /// Needs to be saved to keep the thread alive on web (?),
    /// Cant be saved in temporary storage because of the Copy requirements on IdTypeMap::insert_temp.
    /// Works like this for the moment, since theres only one of these.
    _background_thread: wasm_thread::JoinHandle<Final>,
    to_bg: mpsc::SyncSender<ToTaskMsg>,
    from_bg: std::sync::mpsc::Receiver<FromTaskMsg>,
}

pub trait BgTask<ToTaskMsg, FromTaskMsg> {
    type Final;

    fn execute(
        self,
        to_task: mpsc::Receiver<ToTaskMsg>,
        from_task: mpsc::SyncSender<FromTaskMsg>,
    ) -> Self::Final
    where
        Self: Sized;
}

impl<
        Final,
        ToTaskMsg,
        FromTaskMsg,
        F: FnOnce(mpsc::Receiver<ToTaskMsg>, mpsc::SyncSender<FromTaskMsg>) -> Final + Sized,
    > BgTask<ToTaskMsg, FromTaskMsg> for F
{
    type Final = Final;

    fn execute(
        self,
        to_task: mpsc::Receiver<ToTaskMsg>,
        from_task: mpsc::SyncSender<FromTaskMsg>,
    ) -> Self::Final {
        self(to_task, from_task)
    }
}

impl<Final: Send + 'static, ToTaskMsg: Send + 'static, FromTaskMsg: Send + 'static>
    BgTaskHandle<Final, FromTaskMsg, ToTaskMsg>
{
    pub fn new<const FROM_CHANNEL_SIZE: usize, const TO_CHANNEL_SIZE: usize>(task: impl BgTask<ToTaskMsg, FromTaskMsg, Final = Final> + Send + 'static) -> Self {
        let (to_tx, to_rx) = mpsc::sync_channel::<ToTaskMsg>(TO_CHANNEL_SIZE);
        let (from_tx, from_rx) = mpsc::sync_channel::<FromTaskMsg>(FROM_CHANNEL_SIZE);
        let _background_thread = wasm_thread::spawn(move || task.execute(to_rx, from_tx));
        Self {
            _background_thread,
            to_bg: to_tx,
            from_bg: from_rx,
        }
    }

    pub fn try_recv(&mut self) -> Result<FromTaskMsg, mpsc::TryRecvError> {
        self.from_bg.try_recv()
    }

    pub fn try_send(&mut self, msg: ToTaskMsg) -> Result<(), mpsc::TrySendError<ToTaskMsg>> {
        self.to_bg.try_send(msg)
    }
}
