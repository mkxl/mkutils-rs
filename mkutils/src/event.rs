use crate::utils::Utils;
#[cfg(feature = "derive_more")]
use derive_more::From;
use tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender};

#[derive(Clone)]
#[cfg_attr(feature = "derive_more", derive(From))]
pub struct Event {
    sender: WatchSender<bool>,
    receiver: WatchReceiver<bool>,
}

impl Event {
    #[cfg(feature = "derive_more")]
    const INITIAL_VALUE: bool = false;
    const SET_VALUE: bool = true;

    #[cfg(feature = "derive_more")]
    #[must_use]
    pub fn new() -> Self {
        tokio::sync::watch::channel(Self::INITIAL_VALUE).into()
    }

    #[must_use]
    pub fn is_set(&self) -> bool {
        *self.receiver.borrow()
    }

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

    pub fn set(&self) {
        // NOTE-8e447d
        self.sender.send(Self::SET_VALUE).unit();
    }
}

#[cfg(feature = "derive_more")]
impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}
