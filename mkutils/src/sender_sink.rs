use crate::utils::Utils;
use derive_more::Constructor;
use futures::sink::Unfold;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub type SendFn<Sender, Item, E> = fn(&mut Sender, Item) -> Result<(), E>;

#[derive(Constructor)]
struct SendState<Sender, Item, E> {
    sender: Sender,
    send: SendFn<Sender, Item, E>,
    item: Item,
}

#[derive(Constructor)]
pub struct SendFuture<Sender, Item, E> {
    state: Option<SendState<Sender, Item, E>>,
}

// NOTE: automatic [Unpin] would require every generic field to be [Unpin]; no fields are
// structurally pinned, so this unconditional impl safely lets [poll] call [Pin::get_mut]
// without imposing those bounds on senders, callbacks, items, or errors
impl<Sender, Item, E> Unpin for SendFuture<Sender, Item, E> {}

pub type SenderSink<Sender, Item, E> = Unfold<
    (Sender, SendFn<Sender, Item, E>),
    fn((Sender, SendFn<Sender, Item, E>), Item) -> SendFuture<Sender, Item, E>,
    SendFuture<Sender, Item, E>,
>;

impl<Sender, Item, E> Future for SendFuture<Sender, Item, E> {
    type Output = Result<(Sender, SendFn<Sender, Item, E>), E>;

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut send_state = self.get_mut().state.take().expect(Self::POLLED_AFTER_COMPLETION_ERROR);

        (send_state.send)(send_state.sender.ref_mut(), send_state.item)?;

        send_state.sender.pair(send_state.send).ok().poll_ready()
    }
}

impl<Sender, Item, E> SendFuture<Sender, Item, E> {
    const POLLED_AFTER_COMPLETION_ERROR: &str = "future polled after completion";

    pub fn send((sender, send): (Sender, SendFn<Sender, Item, E>), item: Item) -> Self {
        SendState::new(sender, send, item).some().pipe_into(Self::new)
    }
}
