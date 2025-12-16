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

/// A trait for request/response communication patterns over a socket.
///
/// Implement this trait to define a request type with its associated response type
/// and serialized form. The `Socket` can then use these types for type-safe communication.
///
/// # Associated Types
///
/// - `Response`: The type of the response, defaults to `()`
/// - `Serialized`: The serialized form of the request that can be sent over the wire
///
/// # Examples
///
/// ```rust,ignore
/// use mkutils::Request;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct GetUser {
///     id: u64,
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// impl Request for GetUser {
///     type Response = User;
///     type Serialized = Self;
/// }
/// ```
pub trait Request: Sized {
    /// The type of the response to this request.
    type Response: DeserializeOwned + Serialize = ();
    /// The serialized form of this request.
    type Serialized: From<Self> + Serialize;
}

/// A Unix domain socket for MessagePack-encoded request/response communication.
///
/// `Socket` provides a high-level interface for communicating over Unix domain sockets
/// using length-delimited MessagePack encoding. It implements the `Sink` trait for
/// sending messages and provides methods for request/response patterns.
///
/// # Examples
///
/// ```rust,no_run
/// use mkutils::{Socket, Utils};
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> Result<(), anyhow::Error> {
///     let mut socket = Socket::connect(Path::new("/tmp/my.sock")).await?;
///
///     // Send and receive arbitrary serializable types
///     let response: String = socket.exchange("hello").await?;
///     println!("Got response: {}", response);
///     Ok(())
/// }
/// ```
#[derive(From)]
pub struct Socket {
    frames: Framed<UnixStream, LengthDelimitedCodec>,
}

impl Socket {
    /// Connects to a Unix domain socket at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    pub async fn connect(filepath: &Path) -> Result<Self, IoError> {
        UnixStream::connect(filepath).await?.convert::<Self>().ok()
    }

    /// Receives and deserializes a message from the socket.
    ///
    /// Returns an `Output<T, AnyhowError>` which can be `Ok(value)`, `EndOk`, or `EndErr(error)`.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> Output<T, AnyhowError> {
        self.frames.next().await??.to_value_from_rmp_slice::<T>()?.into()
    }

    /// Sends a value and waits for a response of a different type.
    ///
    /// This is a convenience method for request/response patterns.
    ///
    /// # Errors
    ///
    /// Returns an error if sending fails, receiving fails, or if the socket closes
    /// before a response is received.
    pub async fn exchange<X: Serialize, Y: DeserializeOwned>(&mut self, input: impl Into<X>) -> Result<Y, AnyhowError> {
        self.send(input.into()).await?;

        self.recv().await.into_option().check_next()?
    }

    /// Sends a request and waits for its associated response type.
    ///
    /// Uses the `Request` trait to determine the response type.
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails.
    pub async fn request<T: Request>(&mut self, request: T) -> Result<T::Response, AnyhowError> {
        self.exchange::<T::Serialized, T::Response>(request).await
    }

    /// Sends a response to a request.
    ///
    /// # Errors
    ///
    /// Returns an error if sending fails.
    pub async fn respond<T: Request>(&mut self, response: &T::Response) -> Result<(), AnyhowError> {
        self.send(response).await
    }

    /// Serializes and sends a request without waiting for a response.
    ///
    /// # Errors
    ///
    /// Returns an error if sending fails.
    pub async fn serialize<T: Request>(&mut self, request: T) -> Result<(), AnyhowError> {
        let request = request.convert::<T::Serialized>();

        self.send(request).await
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
        self.frames.start_send_unpin(item.to_rmp_byte_str()?.into())?.ok()
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.frames.poll_flush_unpin(context).map_err(Self::Error::from)
    }

    fn poll_close(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.frames.poll_close_unpin(context).map_err(Self::Error::from)
    }
}
