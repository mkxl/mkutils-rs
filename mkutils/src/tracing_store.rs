use crate::utils::Utils;
use anyhow::Error as AnyhowError;
use std::{
    collections::VecDeque,
    io::{Error as IoError, Write},
    ops::Deref,
};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    oneshot::Sender as OneshotSender,
};
use tracing_subscriber::fmt::MakeWriter;

enum Message {
    WriteEvent(String),
    ReadEvents(OneshotSender<Vec<String>>),
}

pub struct EventWriter {
    buffer: Vec<u8>,
    message_sender: UnboundedSender<Message>,
}

impl EventWriter {
    const TRIM_END_PATTERN: &[char] = &['\r', '\n'];

    const fn new(message_sender: UnboundedSender<Message>) -> Self {
        let buffer = Vec::new();

        Self { buffer, message_sender }
    }
}

impl Write for EventWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.buffer.flush()
    }
}

impl Drop for EventWriter {
    fn drop(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let event = self
            .buffer
            .deref()
            .pipe_into(String::from_utf8_lossy)
            .trim_end_matches(Self::TRIM_END_PATTERN)
            .to_owned();

        if !event.is_empty() {
            event
                .pipe_into(Message::WriteEvent)
                .pipe_into_method(&self.message_sender, UnboundedSender::send)
                .mem_drop();
        }
    }
}

#[derive(Clone)]
pub struct TracingStore {
    message_sender: UnboundedSender<Message>,
}

impl TracingStore {
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        let (message_sender, message_receiver) = tokio::sync::mpsc::unbounded_channel();

        Self::read_messages(message_receiver, max_size).spawn_task();

        Self { message_sender }
    }

    async fn read_messages(mut message_receiver: UnboundedReceiver<Message>, max_size: usize) {
        let mut events = VecDeque::with_capacity(max_size);

        // NOTE:
        // - if there's an error sending the events over the one shot channel
        //   in the [Message::ReadEvents] branch, just drop the error
        // - i'm unable to send the error itself over the consumed channel and
        //   persisting the errors to try and send later doesn't feel like the
        //   right choice
        while let Some(message) = message_receiver.recv().await {
            match message {
                Message::WriteEvent(event) => events.push_bounded(event, max_size),
                Message::ReadEvents(events_sender) => events
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()
                    .pipe_into_method(events_sender, OneshotSender::send)
                    .mem_drop(),
            }
        }
    }

    pub async fn read(&self) -> Result<Vec<String>, AnyhowError> {
        let (events_sender, events_receiver) = tokio::sync::oneshot::channel();

        events_sender
            .pipe_into(Message::ReadEvents)
            .pipe_into_method(&self.message_sender, UnboundedSender::send)?;

        events_receiver.await?.ok()
    }
}

impl<'a> MakeWriter<'a> for TracingStore {
    type Writer = EventWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.message_sender.clone().pipe_into(EventWriter::new)
    }
}
