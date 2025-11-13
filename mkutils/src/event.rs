use crate::utils::Utils;
use derive_more::Constructor;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct EventReceiver {
    is_set: bool,
    notify: Arc<Notify>,
}

impl EventReceiver {
    const INITIAL_IS_SET: bool = false;
    const TERMINAL_IS_SET: bool = true;

    const fn new(notify: Arc<Notify>) -> Self {
        let is_set = Self::INITIAL_IS_SET;

        Self { is_set, notify }
    }

    pub async fn wait(&mut self) {
        if self.is_set {
            return;
        }

        self.notify.notified().await;

        self.is_set = Self::TERMINAL_IS_SET;
    }
}

#[derive(Constructor)]
pub struct EventSender {
    notify: Arc<Notify>,
}

impl EventSender {
    pub fn set(&self) {
        self.notify.notify_one();
    }
}

pub struct Event;

impl Event {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new() -> (EventSender, EventReceiver) {
        let notify = Notify::new().arc();
        let event_sender = EventSender::new(notify.clone());
        let event_receiver = EventReceiver::new(notify);

        (event_sender, event_receiver)
    }
}
