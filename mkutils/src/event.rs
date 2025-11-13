use crate::utils::Utils;
use anyhow::Error as AnyhowError;
use derive_more::From;
use tokio::sync::oneshot::{
    Receiver as OneshotReceiver, Sender as OneshotSender, error::RecvError as OneshotRecvError,
};

#[derive(From)]
pub struct EventReceiver {
    oneshot_receiver: Option<OneshotReceiver<()>>,
}

impl EventReceiver {
    fn new(oneshot_receiver: OneshotReceiver<()>) -> Self {
        let oneshot_receiver = oneshot_receiver.some();

        Self { oneshot_receiver }
    }

    pub async fn wait(&mut self) -> Result<(), OneshotRecvError> {
        let Some(oneshot_receiver) = self.oneshot_receiver.take() else { return ().ok() };

        oneshot_receiver.await
    }
}

#[derive(From)]
pub struct EventSender {
    oneshot_sender: Option<OneshotSender<()>>,
}

impl EventSender {
    fn new(oneshot_sender: OneshotSender<()>) -> Self {
        let oneshot_sender = oneshot_sender.some();

        Self { oneshot_sender }
    }

    pub fn set(&mut self) -> Result<(), AnyhowError> {
        let Some(oneshot_sender) = self.oneshot_sender.take() else { return ().ok() };

        ().send_to_oneshot(oneshot_sender)
    }
}

pub struct Event;

impl Event {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new() -> (EventSender, EventReceiver) {
        let (oneshot_sender, oneshot_receiver) = tokio::sync::oneshot::channel();
        let event_sender = EventSender::new(oneshot_sender);
        let event_receiver = EventReceiver::new(oneshot_receiver);

        (event_sender, event_receiver)
    }
}
