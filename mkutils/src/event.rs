use crate::utils::Utils;
use derive_more::From;
use tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender};

/// A simple async event primitive for signaling between tasks.
///
/// `Event` provides a lightweight way to signal from one task to potentially many
/// waiting tasks. It uses a tokio watch channel internally to provide efficient
/// notification.
///
/// # Examples
///
/// ```rust,no_run
/// use mkutils::Event;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let event = Event::new();
///     let mut event_clone = event.clone();
///
///     tokio::spawn(async move {
///         tokio::time::sleep(Duration::from_secs(1)).await;
///         event.set();
///     });
///
///     event_clone.wait().await; // Waits until set() is called
///     println!("Event triggered!");
/// }
/// ```
#[derive(Clone, From)]
pub struct Event {
    sender: WatchSender<bool>,
    receiver: WatchReceiver<bool>,
}

impl Event {
    const INITIAL_VALUE: bool = false;
    const SET_VALUE: bool = true;

    /// Creates a new `Event` in the unset state.
    #[must_use]
    pub fn new() -> Self {
        tokio::sync::watch::channel(Self::INITIAL_VALUE).into()
    }

    /// Returns `true` if the event has been set.
    ///
    /// This is a non-blocking check of the event's current state.
    #[must_use]
    pub fn is_set(&self) -> bool {
        *self.receiver.borrow()
    }

    /// Waits asynchronously until the event is set.
    ///
    /// If the event is already set when this is called, returns immediately.
    /// Otherwise, blocks until `set()` is called on any clone of this event.
    pub async fn wait(&mut self) {
        if self.is_set() {
            return;
        }

        // NOTE:
        // - [self.receiver.changed()] must resolve with [Ok(true)]
        //   - it won't resolve with [Ok(false)] because the only way for the value to have changed
        //     is by having [self.set()] called which sets the changed value to
        //     [Self::SET_VALUE = true]
        //   - it won't resolve with [Err(RecvError)] because it will only do so if the
        //     corresponding [WatchSender] is dropped but that can't happen because this [Event]
        //     instance also owns that [WatchSender] and never drops it [NOTE-8e447d]
        // - thus, when this resolves we know it has been set to true
        self.receiver.changed().await.unit();
    }

    /// Sets the event, waking all tasks waiting on it.
    ///
    /// This will cause all current and future `wait()` calls to return immediately
    /// until the event is reset (which is not currently supported - events are one-shot).
    pub fn set(&self) {
        // NOTE-8e447d
        self.sender.send(Self::SET_VALUE).unit();
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}
