use crate::{output::Output, utils::Utils};
use anyhow::Error as AnyhowError;
use derive_more::From;
use futures::{Sink, SinkExt, StreamExt};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::Error as IoError,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub trait Request: Serialize {
    type Response: DeserializeOwned;
}

#[derive(From)]
pub struct Socket {
    frames: Framed<UnixStream, LengthDelimitedCodec>,
}

impl Socket {
    pub async fn new(filepath: &Path) -> Result<Self, IoError> {
        UnixStream::connect(filepath).await?.convert::<Self>().ok()
    }

    pub async fn recv<T: DeserializeOwned>(&mut self) -> Output<T, AnyhowError> {
        self.frames
            .next()
            .await??
            .to_value_from_postcard_byte_str::<T>()?
            .into()
    }

    pub async fn request<T: Request>(&mut self, request: T) -> Result<T::Response, AnyhowError> {
        self.send(request).await?;

        self.recv().await.into_option().check_next()?
    }
}

impl From<UnixStream> for Socket {
    fn from(unix_stream: UnixStream) -> Self {
        unix_stream.into_length_delimited_frames().into()
    }
}

impl<T: Serialize> Sink<T> for Socket {
    type Error = AnyhowError;

    fn poll_ready(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.frames.poll_ready_unpin(context).map_err(Self::Error::from)
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.frames.start_send_unpin(item.to_postcard_byte_str()?.into())?.ok()
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.frames.poll_flush_unpin(context).map_err(Self::Error::from)
    }

    fn poll_close(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.frames.poll_close_unpin(context).map_err(Self::Error::from)
    }
}
