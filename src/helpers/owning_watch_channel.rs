use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

// Custom error types for send and receive operations
#[derive(Debug)]
pub enum SendError {
    ReveiverDropped,
}

#[derive(Debug)]
pub enum RecvError {
    SenderDropped,
}

pub struct Sender<T> {
    /// Shared data container
    data: Arc<Mutex<Option<T>>>,
    /// Notification mechanism
    notify: Arc<Notify>,
    /// Track if the receiver is still active
    is_active: Arc<AtomicBool>,
}

pub struct Receiver<T> {
    /// Shared data container
    data: Arc<Mutex<Option<T>>>,
    /// Notification mechanism
    notify: Arc<Notify>,
    /// Track if the receiver is still active
    is_active: Arc<AtomicBool>,
}

impl<T> Sender<T> {
    // Send a new value and notify the receiver, returning a Result
    pub async fn send(&self, value: T) -> Result<(), SendError> {
        if self.is_active.load(Ordering::Acquire) {
            let mut data = self.data.lock().await;
            *data = Some(value);
            self.notify.notify_one();
            Ok(())
        } else {
            Err(SendError::ReveiverDropped)
        }
    }
}

impl<T> Receiver<T> {
    // Wait for the update and take ownership of the value, returning a Result
    pub async fn recv(&self) -> Result<T, RecvError> {
        loop {
            if !self.is_active.load(Ordering::Acquire) {
                return Err(RecvError::SenderDropped);
            }

            //  See if theres a value present, if so return it
            let mut data = self.data.lock().await;
            if let Some(value) = data.take() {
                return Ok(value);
            };

            // Drop the lock and wait for the notification
            drop(data);
            self.notify.notified().await;
            tracing::debug!("Spinning in {}", file!());
        }
    }
}

// Factory function to create a new Sender and Receiver
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let data = Arc::new(Mutex::new(None));
    let notify = Arc::new(Notify::new());
    let is_active = Arc::new(AtomicBool::new(true));

    let sender = Sender {
        data: Arc::clone(&data),
        notify: Arc::clone(&notify),
        is_active: Arc::clone(&is_active),
    };

    let receiver = Receiver {
        data: Arc::clone(&data),
        notify: Arc::clone(&notify),
        is_active: Arc::clone(&is_active),
    };

    (sender, receiver)
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.is_active.store(false, Ordering::Release);
        self.notify.notify_waiters(); // Wake up any waiting receivers
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.is_active.store(false, Ordering::Release);
        self.notify.notify_waiters(); // Wake up any waiting senders
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn main() {
        // Create the channel
        let (sender, receiver) = channel::<String>();

        // Sender task
        let sender_task = tokio::spawn(async move {
            let data = "Hello, World!".to_string();
            if let Err(_) = sender.send(data).await {
                panic!("Sender error");
            }
        });

        // Receiver task
        let receiver_task = tokio::spawn(async move {
            match receiver.recv().await {
                Ok(value) => println!("Receiver: Got value: {}", value),
                _ => panic!("Didnt get expected value"),
            }
        });

        // Await tasks
        drop(tokio::join!(sender_task, receiver_task));
    }
}
