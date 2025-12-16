#[cfg(feature = "serde")]
use crate::as_valuable::AsValuable;
#[cfg(feature = "fmt")]
use crate::fmt::{Debugged, OptionalDisplay};
#[cfg(feature = "ropey")]
use crate::geometry::PointUsize;
#[cfg(feature = "async")]
use crate::into_stream::IntoStream;
use crate::is::Is;
#[cfg(feature = "socket")]
use crate::socket::{Request, Socket};
#[cfg(feature = "tracing")]
use crate::status::Status;
#[cfg(feature = "async")]
use crate::{read_value::ReadValue, run_for::RunForError};
#[cfg(feature = "anyhow")]
use anyhow::{Context, Error as AnyhowError};
#[cfg(feature = "async")]
use bytes::Buf;
#[cfg(feature = "poem")]
use bytes::Bytes;
#[cfg(feature = "fs")]
use camino::{Utf8Path, Utf8PathBuf};
#[cfg(any(feature = "async", feature = "poem"))]
use futures::Stream;
#[cfg(feature = "async")]
use futures::{Sink, SinkExt, StreamExt, TryFuture, future::Either, stream::Filter};
#[cfg(feature = "num")]
use num::traits::{SaturatingAdd, SaturatingSub};
#[cfg(feature = "poem")]
use poem::{Body as PoemBody, Endpoint, IntoResponse, web::websocket::Message as PoemMessage};
#[cfg(feature = "poem")]
use poem_openapi::{
    error::ParseRequestPayloadError,
    payload::{Binary as PoemBinary, Json as PoemJson},
};
#[cfg(feature = "tui")]
use ratatui::{
    layout::Rect,
    text::{Line, Span},
};
#[cfg(feature = "reqwest")]
use reqwest::{RequestBuilder, Response};
#[cfg(feature = "rmp")]
use rmp_serde::{decode::Error as RmpDecodeError, encode::Error as RmpEncodeError};
#[cfg(feature = "ropey")]
use ropey::{
    Rope, RopeSlice,
    iter::{Chunks, Lines},
};
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(any(feature = "rmp", feature = "serde"))]
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "poem", feature = "serde"))]
use serde_json::Error as SerdeJsonError;
#[cfg(feature = "serde")]
use serde_json::{Value as Json, value::Index};
#[cfg(feature = "serde")]
use serde_yaml_ng::Error as SerdeYamlError;
use std::{
    borrow::{Borrow, BorrowMut, Cow},
    collections::HashMap,
    error::Error as StdError,
    fmt::Display,
    fs::File,
    future::{Future, Ready},
    hash::Hash,
    io::{BufReader, BufWriter, Error as IoError, Read, Write},
    iter::{Once, Repeat},
    marker::Unpin,
    ops::{Add, ControlFlow, Range},
    path::Path,
    pin::Pin,
    str::Utf8Error,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::Poll,
    time::Instant,
};
#[cfg(feature = "anyhow")]
use std::{fmt::Debug, path::PathBuf};
#[cfg(feature = "async")]
use std::{fs::Metadata, time::Duration};
#[cfg(feature = "async")]
use tokio::sync::oneshot::Sender as OneshotSender;
#[cfg(feature = "async")]
use tokio::{
    fs::File as TokioFile,
    io::{
        AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader as TokioBufReader, BufWriter as TokioBufWriter,
    },
    task::{JoinHandle, JoinSet, LocalSet},
    time::{Sleep, Timeout},
};
#[cfg(feature = "async")]
use tokio_util::{
    codec::{Framed, LengthDelimitedCodec, LinesCodec},
    io::StreamReader,
};
#[cfg(feature = "tracing")]
use tracing::Level;
#[cfg(feature = "unicode-segmentation")]
use unicode_segmentation::{Graphemes, UnicodeSegmentation};
#[cfg(feature = "serde")]
use valuable::Value;

#[allow(async_fn_in_trait)]
pub trait Utils {
    /// Aborts all tasks in a `JoinSet` and waits for them to complete.
    ///
    /// This method calls `abort_all()` on the `JoinSet`, then waits for all
    /// tasks to finish by consuming all join results.
    #[cfg(feature = "async")]
    async fn abort_all_and_wait<T: 'static>(&mut self)
    where
        Self: BorrowMut<JoinSet<T>>,
    {
        let join_set = self.borrow_mut();

        join_set.abort_all();

        while join_set.join_next().await.is_some() {}
    }

    /// Converts a path to an absolute UTF-8 path.
    ///
    /// If the path is already absolute, returns a borrowed reference. Otherwise,
    /// resolves it to an absolute path and returns an owned value.
    #[cfg(feature = "fs")]
    fn absolute_utf8(&self) -> Result<Cow<'_, Utf8Path>, IoError>
    where
        Self: AsRef<Utf8Path>,
    {
        let path = self.as_ref();

        if path.is_absolute() {
            path.borrowed().ok()
        } else {
            camino::absolute_utf8(path)?.owned::<Utf8Path>().ok()
        }
    }

    /// Chains two futures together, awaiting the first then the second.
    ///
    /// Awaits `self`, discards its output, then awaits `rhs` and returns its output.
    async fn achain<T: Future>(self, rhs: T) -> T::Output
    where
        Self: Future + Sized,
    {
        self.await;

        rhs.await
    }

    /// Adds a span to a ratatui `Line`.
    ///
    /// Converts `self` into a `Line`, appends the given span, and returns the modified line.
    #[cfg(feature = "tui")]
    fn add_span<'a, T: Into<Span<'a>>>(self, span: T) -> Line<'a>
    where
        Self: Into<Line<'a>>,
    {
        let mut line = self.into();

        line.spans.push(span.into());

        line
    }

    /// Converts a `Result` with any error type into a `Result<T, AnyhowError>`.
    ///
    /// Maps the error variant using `Into<AnyhowError>`.
    #[cfg(feature = "anyhow")]
    fn anyhow_result<T, E: Into<AnyhowError>>(self) -> Result<T, AnyhowError>
    where
        Self: Is<Result<T, E>>,
    {
        self.into_self().map_err(E::into)
    }

    /// Wraps `self` in an `Arc`.
    fn arc(self) -> Arc<Self>
    where
        Self: Sized,
    {
        Arc::new(self)
    }

    /// Awaits `self`, then awaits `next` and returns its output.
    ///
    /// Similar to `achain`, but the second future is passed as an `impl Future`.
    async fn async_with<T>(self, next: impl Future<Output = T>) -> T
    where
        Self: Future + Sized,
    {
        self.await;

        next.await
    }

    /// Extracts a borrowed reference from a `Cow`.
    ///
    /// Returns `&B` from `Cow<'a, B>`, regardless of whether the Cow is borrowed or owned.
    fn as_borrowed<'a, B: ?Sized + ToOwned>(&'a self) -> &'a B
    where
        Self: Borrow<Cow<'a, B>>,
    {
        self.borrow().borrow()
    }

    /// Converts a byte slice to a UTF-8 string slice.
    ///
    /// Returns an error if the bytes are not valid UTF-8.
    fn as_utf8(&self) -> Result<&str, Utf8Error>
    where
        Self: AsRef<[u8]>,
    {
        std::str::from_utf8(self.as_ref())
    }

    /// Converts `self` to a `&Utf8Path` reference.
    #[cfg(feature = "fs")]
    fn as_utf8_path(&self) -> &Utf8Path
    where
        Self: AsRef<Utf8Path>,
    {
        self.as_ref()
    }

    /// Creates a `RopeSlice` spanning the entire `Rope`.
    #[cfg(feature = "ropey")]
    fn as_slice(&self) -> RopeSlice<'_>
    where
        Self: Borrow<Rope>,
    {
        self.borrow().slice(..)
    }

    /// Converts `self` to a `valuable::Value`.
    #[cfg(feature = "serde")]
    fn as_valuable(&self) -> Value<'_>
    where
        Self: AsValuable,
    {
        AsValuable::as_valuable(self)
    }

    /// Wraps `self` in a borrowed `Cow`.
    fn borrowed(&self) -> Cow<'_, Self>
    where
        Self: ToOwned,
    {
        Cow::Borrowed(self)
    }

    /// Wraps `self` in a `BufReader`.
    fn buf_reader(self) -> BufReader<Self>
    where
        Self: Read + Sized,
    {
        BufReader::new(self)
    }

    /// Wraps `self` in a tokio `BufReader`.
    #[cfg(feature = "async")]
    fn buf_reader_async(self) -> TokioBufReader<Self>
    where
        Self: AsyncRead + Sized,
    {
        TokioBufReader::new(self)
    }

    /// Wraps `self` in a `BufWriter`.
    fn buf_writer(self) -> BufWriter<Self>
    where
        Self: Write + Sized,
    {
        BufWriter::new(self)
    }

    /// Wraps `self` in a tokio `BufWriter`.
    #[cfg(feature = "async")]
    fn buf_writer_async(self) -> TokioBufWriter<Self>
    where
        Self: AsyncWrite + Sized,
    {
        TokioBufWriter::new(self)
    }

    /// Casts a reference from `&Self` to `&T`.
    ///
    /// # Safety
    ///
    /// Requires that `Self` and `T` have the same memory layout (e.g., via `#[repr(transparent)]`).
    ///
    /// See: [https://stackoverflow.com/questions/79593399/implement-valuablevaluable-on-serde-jsonvalue]
    // NOTE: requires that [Self] and [T] have the same layout (provided by [#[repr(transparent)]]):
    // [https://stackoverflow.com/questions/79593399/implement-valuablevaluable-on-serde-jsonvalue]
    fn cast_ref<T>(&self) -> &T {
        let ptr = std::ptr::from_ref(self).cast::<T>();
        let value = unsafe { ptr.as_ref() };

        value.unwrap()
    }

    /// Concatenates two displayable values into a `String`.
    fn cat<T: Display>(&self, rhs: T) -> String
    where
        Self: Display,
    {
        std::format!("{self}{rhs}")
    }

    /// Converts an `Option<T>` to a `Result<T, AnyhowError>`.
    ///
    /// Returns an error if the Option is `None`, with a message indicating
    /// that the sequence of items is exhausted.
    #[cfg(feature = "anyhow")]
    fn check_next<T>(self) -> Result<T, AnyhowError>
    where
        Self: Is<Option<T>>,
    {
        match self.into_self() {
            Some(item) => item.ok(),
            None => anyhow::bail!(
                "sequence of {type_name} items is exhausted",
                type_name = Self::type_name(),
            ),
        }
    }

    /// Converts an `Option<T>` to a `Result<T, AnyhowError>`.
    ///
    /// Returns an error if the Option is `None`, with a message indicating
    /// that the keyed entry is not present in the collection.
    #[cfg(feature = "anyhow")]
    fn check_present<T>(self) -> Result<T, AnyhowError>
    where
        Self: Is<Option<T>>,
    {
        match self.into_self() {
            Some(item) => item.ok(),
            None => anyhow::bail!(
                "keyed entry of type {type_name} is not present in the collection",
                type_name = Self::type_name(),
            ),
        }
    }

    /// Checks if a reqwest `Response` has a successful HTTP status code.
    ///
    /// Returns the response if the status is not a client or server error.
    /// Otherwise, reads the response body and returns an error with the status and text.
    ///
    /// See: [https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.error_for_status]
    // NOTE: [https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.error_for_status]
    #[cfg(feature = "reqwest")]
    async fn check_status(self) -> Result<Response, AnyhowError>
    where
        Self: Is<Response>,
    {
        let response = self.into_self();
        let status = response.status();

        if !status.is_client_error() && !status.is_server_error() {
            return response.ok();
        }

        let text = match response.text().await {
            Ok(text) => text,
            Err(error) => std::format!("unable to read response text: {error}"),
        };

        anyhow::bail!("({status}) {text}")
    }

    /// Converts `self` to type `T` using `From<Self>`.
    fn convert<T: From<Self>>(self) -> T
    where
        Self: Sized,
    {
        self.into()
    }

    /// Finds the first element in a slice equal to `query`.
    ///
    /// Returns the index and reference to the element, or `None` if not found.
    fn find_eq<Q, K>(&self, query: Q) -> Option<(usize, &K)>
    where
        Self: AsRef<[K]>,
        for<'a> &'a K: PartialEq<Q>,
    {
        self.as_ref().iter().enumerate().find(|(_index, key)| *key == query)
    }

    /// Checks if a slice contains an element equal to `query`.
    fn contains_eq<Q, K>(&self, query: Q) -> bool
    where
        Self: AsRef<[K]>,
        for<'a> &'a K: PartialEq<Q>,
    {
        self.find_eq(query).is_some()
    }

    /// Adds context with a path to a `Result`.
    ///
    /// Formats the context as "`context`: `path`" and attaches it to the error.
    #[cfg(feature = "anyhow")]
    fn context_path<T, E, C: 'static + Display + Send + Sync, P: AsRef<Path>>(
        self,
        context: C,
        path: P,
    ) -> Result<T, AnyhowError>
    where
        Self: Context<T, E> + Sized,
    {
        let context = std::format!("{context}: {path}", path = path.as_ref().display());

        self.context(context)
    }

    /// Creates a new file at the path specified by `self`.
    ///
    /// If the file already exists, it will be truncated.
    fn create(&self) -> Result<File, IoError>
    where
        Self: AsRef<Path>,
    {
        File::create(self)
    }

    /// Creates a directory at the path specified by `self`, including all parent directories.
    fn create_dir_all(&self) -> Result<(), IoError>
    where
        Self: AsRef<Path>,
    {
        std::fs::create_dir_all(self)
    }

    /// Wraps `self` in a `Debugged` wrapper for displaying with the `Debug` trait.
    #[cfg(feature = "fmt")]
    fn debug(&self) -> Debugged<'_, Self> {
        Debugged::new(self)
    }

    /// Wraps `self` in `Err`.
    fn err<T>(self) -> Result<T, Self>
    where
        Self: Sized,
    {
        Err(self)
    }

    /// Expands a path starting with `~/` to use the actual home directory.
    ///
    /// If the path starts with `~/`, replaces it with the user's home directory path.
    /// Otherwise, returns the path unchanged.
    #[cfg(feature = "fs")]
    fn expand_user(&self) -> Cow<'_, Utf8Path>
    where
        Self: AsRef<str>,
    {
        let path_str = self.as_ref();

        if let Some(relative_path_str) = path_str.strip_prefix("~/")
            && let Some(home_dirpath) = Self::home_dirpath()
        {
            home_dirpath.join(relative_path_str).owned()
        } else {
            path_str.as_utf8_path().borrowed()
        }
    }

    /// Contracts a path to use `~` for the home directory.
    ///
    /// If the path starts with the user's home directory, replaces it with `~`.
    /// Otherwise, returns the path unchanged.
    #[cfg(feature = "fs")]
    fn unexpand_user(&self) -> Cow<'_, Utf8Path>
    where
        Self: AsRef<str>,
    {
        let path_str = self.as_ref();

        if let Some(home_dirpath) = Self::home_dirpath()
            && let Some(relative_path_str) = path_str.strip_prefix(home_dirpath.as_str())
        {
            if relative_path_str.is_empty() {
                "~".convert::<Utf8PathBuf>().owned()
            } else if relative_path_str.as_utf8_path().is_absolute() {
                "~".cat(relative_path_str).convert::<Utf8PathBuf>().owned()
            } else {
                path_str.as_utf8_path().borrowed()
            }
        } else {
            path_str.as_utf8_path().borrowed()
        }
    }

    /// Returns an iterator over the extended grapheme clusters of a string.
    #[cfg(feature = "unicode-segmentation")]
    fn extended_graphemes(&self) -> Graphemes<'_>
    where
        Self: AsRef<str>,
    {
        self.as_ref().graphemes(true)
    }

    /// Returns an iterator over a range of extended grapheme clusters in a `RopeSlice`.
    #[cfg(feature = "ropey")]
    fn extended_graphemes_at<'a>(self, extended_graphemes_index_range: Range<usize>) -> impl Iterator<Item = &'a str>
    where
        Self: Is<RopeSlice<'a>>,
    {
        let extended_graphemes_index_range = extended_graphemes_index_range.borrow();

        self.into_self()
            .chunks()
            .flat_map(str::extended_graphemes)
            .skip(extended_graphemes_index_range.start)
            .take(extended_graphemes_index_range.len())
    }

    /// Returns an iterator over grapheme clusters in a rectangular region of a `RopeSlice`.
    ///
    /// The outer iterator yields lines, and each inner iterator yields grapheme clusters
    /// within the specified column range for that line.
    #[cfg(feature = "ropey")]
    fn extended_graphemes_at_rect<'a>(
        self,
        lines_index_range: Range<usize>,
        extended_graphemes_index_range: Range<usize>,
    ) -> impl Iterator<Item = impl Iterator<Item = &'a str>>
    where
        Self: Is<RopeSlice<'a>>,
    {
        self.into_self()
            .saturating_lines_at(lines_index_range.start)
            .take(lines_index_range.len())
            .map(move |line_rope_slice| line_rope_slice.extended_graphemes_at(extended_graphemes_index_range.clone()))
    }

    /// Filters a stream using a synchronous predicate function.
    ///
    /// Wraps a synchronous predicate in `Ready` to work with async stream filtering.
    #[cfg(feature = "async")]
    fn filter_sync(
        self,
        mut func: impl FnMut(&Self::Item) -> bool,
    ) -> Filter<Self, Ready<bool>, impl FnMut(&Self::Item) -> Ready<bool>>
    where
        Self: Sized + StreamExt,
    {
        // TODO: figure out why [func.pipe(bool::ready)] won't work
        self.filter(move |x| func(x).ready())
    }

    /// Extracts the file name from a path.
    ///
    /// Returns an error if the path has no file name component.
    #[cfg(feature = "fs")]
    fn file_name_ok(&self) -> Result<&str, AnyhowError>
    where
        Self: AsRef<Utf8Path>,
    {
        self.as_ref().file_name().context("path has no file name")
    }

    /// Checks if an `Instant` has occurred (is in the past or now).
    fn has_happened(self) -> bool
    where
        Self: Is<Instant>,
    {
        self.into_self() <= Instant::now()
    }

    /// Returns the user's home directory path.
    #[cfg(feature = "fs")]
    #[must_use]
    fn home_dirpath() -> Option<Utf8PathBuf> {
        home::home_dir()?.try_convert::<Utf8PathBuf>().ok()
    }

    /// Returns `true_value` if `self` is `true`, otherwise returns `false_value`.
    fn if_else<T>(self, true_value: T, false_value: T) -> T
    where
        Self: Is<bool>,
    {
        if self.into_self() { true_value } else { false_value }
    }

    /// Converts a mutable reference to an immutable reference.
    fn immutable(&mut self) -> &Self {
        self
    }

    /// Atomically increments an `AtomicUsize` and returns the previous value.
    fn inc(&self) -> usize
    where
        Self: Borrow<AtomicUsize>,
    {
        self.borrow().fetch_add(1, Ordering::SeqCst)
    }

    /// Inserts a key-value pair into a `HashMap` and returns a mutable reference to the value.
    fn insert_mut<'a, K: 'a + Eq + Hash, V>(&'a mut self, key: K, value: V) -> &'a mut V
    where
        Self: BorrowMut<HashMap<K, V>>,
    {
        self.borrow_mut().entry(key).insert_entry(value).into_mut()
    }

    /// Wraps `self` in `ControlFlow::Break`.
    fn into_break<C>(self) -> ControlFlow<Self, C>
    where
        Self: Sized,
    {
        ControlFlow::Break(self)
    }

    /// Wraps `self` in `ControlFlow::Continue`.
    fn into_continue<B>(self) -> ControlFlow<B, Self>
    where
        Self: Sized,
    {
        ControlFlow::Continue(self)
    }

    /// Converts `self` into a Poem `Endpoint` that always returns the same response.
    #[cfg(feature = "poem")]
    fn into_endpoint(self) -> impl Endpoint<Output = Self>
    where
        Self: Clone + IntoResponse + Sync,
    {
        let func = move |_request| self.clone();

        poem::endpoint::make_sync(func)
    }

    /// Wraps `self` in `Either::Left`.
    #[cfg(feature = "async")]
    fn into_left<R>(self) -> Either<Self, R>
    where
        Self: Sized,
    {
        Either::Left(self)
    }

    /// Converts `self` into a ratatui `Line`.
    #[cfg(feature = "tui")]
    fn into_line<'a>(self) -> Line<'a>
    where
        Self: Into<Cow<'a, str>>,
    {
        self.into().into()
    }

    /// Converts a stream of byte buffers into an `AsyncRead` stream reader.
    #[cfg(feature = "async")]
    fn into_stream_reader<B: Buf, E: Into<IoError>>(self) -> StreamReader<Self, B>
    where
        Self: Sized + Stream<Item = Result<B, E>>,
    {
        StreamReader::new(self)
    }

    /// Wraps an async I/O type with a length-delimited frame codec.
    #[cfg(feature = "async")]
    fn into_length_delimited_frames(self) -> Framed<Self, LengthDelimitedCodec>
    where
        Self: Sized,
    {
        Framed::new(self, LengthDelimitedCodec::new())
    }

    /// Wraps an async I/O type with a line-based frame codec.
    #[cfg(feature = "async")]
    fn into_line_frames(self) -> Framed<Self, LinesCodec>
    where
        Self: Sized,
    {
        Framed::new(self, LinesCodec::new())
    }

    /// Wraps `self` in `Either::Right`.
    #[cfg(feature = "async")]
    fn into_right<L>(self) -> Either<L, Self>
    where
        Self: Sized,
    {
        Either::Right(self)
    }

    /// Converts a `serde_json` result into a Poem `ParseRequestPayloadError` result.
    ///
    /// See: [https://docs.rs/poem-openapi/latest/src/poem_openapi/payload/json.rs.html]
    // NOTE: [https://docs.rs/poem-openapi/latest/src/poem_openapi/payload/json.rs.html]
    #[cfg(feature = "poem")]
    fn into_parse_request_payload_result<T>(self) -> Result<T, ParseRequestPayloadError>
    where
        Self: Is<Result<T, SerdeJsonError>>,
    {
        match self.into_self() {
            Ok(value) => value.ok(),
            Err(serde_json_error) => ParseRequestPayloadError {
                reason: serde_json_error.to_string(),
            }
            .err(),
        }
    }

    /// Races two futures and returns the output of whichever completes first.
    ///
    /// Uses `tokio::select!` to await both futures concurrently.
    #[cfg(feature = "async")]
    async fn into_select<T: Future>(self, rhs: T) -> Either<Self::Output, T::Output>
    where
        Self: Future + Sized,
    {
        tokio::select! {
            value = self => value.into_left(),
            value = rhs => value.into_right(),
        }
    }

    /// Wraps a `Result` in a `Status` for tracing integration.
    #[cfg(feature = "tracing")]
    fn into_status<T, E>(self) -> Status<T, E>
    where
        Self: Is<Result<T, E>>,
    {
        Status(self.into_self())
    }

    /// Converts `self` into a stream.
    #[cfg(feature = "async")]
    fn into_stream(self) -> Self::Stream
    where
        Self: IntoStream + Sized,
    {
        IntoStream::into_stream(self)
    }

    /// Converts a `PathBuf` into a `String`.
    ///
    /// Returns an error if the path is not valid UTF-8.
    #[cfg(feature = "anyhow")]
    fn into_string(self) -> Result<String, AnyhowError>
    where
        Self: Is<PathBuf>,
    {
        match self.into_self().into_os_string().into_string() {
            Ok(string) => string.ok(),
            Err(os_string) => os_string.invalid_utf8_err(),
        }
    }

    /// Deserializes a value from a `serde_json::Value`.
    #[cfg(feature = "serde")]
    fn into_value_from_json<T: DeserializeOwned>(self) -> Result<T, SerdeJsonError>
    where
        Self: Is<Json>,
    {
        serde_json::from_value(self.into_self())
    }

    /// Returns an error indicating that `self` is not valid UTF-8.
    #[cfg(feature = "anyhow")]
    fn invalid_utf8_err<T>(&self) -> Result<T, AnyhowError>
    where
        Self: Debug,
    {
        anyhow::bail!("{self:?} is not valid utf-8")
    }

    /// Converts `self` into an `io::Error`.
    fn io_error(self) -> IoError
    where
        Self: Into<Box<dyn StdError + Send + Sync>>,
    {
        IoError::other(self)
    }

    /// Converts a `Result<T, E>` into a `Result<T, IoError>`.
    fn io_result<T, E: Into<Box<dyn StdError + Send + Sync>>>(self) -> Result<T, IoError>
    where
        Self: Is<Result<T, E>>,
    {
        self.into_self().map_err(E::io_error)
    }

    /// Awaits all futures in an iterator and collects their results.
    #[cfg(feature = "async")]
    async fn join_all<T>(self) -> T
    where
        Self: IntoIterator<Item: Future> + Sized,
        T: FromIterator<<Self::Item as Future>::Output>,
    {
        futures::future::join_all(self).await.into_iter().collect()
    }

    /// Returns the number of extended grapheme clusters in a string.
    #[cfg(feature = "unicode-segmentation")]
    fn len_extended_graphemes(&self) -> usize
    where
        Self: AsRef<str>,
    {
        self.as_ref().extended_graphemes().count()
    }

    /// Returns a tracing `Level` based on whether a `Result` is `Ok` or `Err`.
    ///
    /// Returns `Level::INFO` for `Ok`, `Level::WARN` for `Err`.
    #[cfg(feature = "tracing")]
    fn level<T, E>(&self) -> Level
    where
        Self: Borrow<Result<T, E>>,
    {
        if self.borrow().is_ok() {
            Level::INFO
        } else {
            Level::WARN
        }
    }

    /// Logs `self` as a warning-level error using tracing.
    #[cfg(feature = "tracing")]
    fn log_error(&self)
    where
        Self: Display,
    {
        tracing::warn!(error = %self, "error: {self:#}");
    }

    /// Logs an error if `self` is an `Err` result, then returns `self` unchanged.
    #[cfg(feature = "tracing")]
    #[must_use]
    fn log_if_error<T, E: Display>(self) -> Self
    where
        Self: Borrow<Result<T, E>> + Sized,
    {
        if let Err(error) = self.borrow() {
            error.log_error();
        }

        self
    }

    /// Maps each item using `func` and collects the results into type `T`.
    fn map_collect<Y, T: FromIterator<Y>>(self, func: impl FnMut(Self::Item) -> Y) -> T
    where
        Self: IntoIterator + Sized,
    {
        self.into_iter().map(func).collect::<T>()
    }

    /// Maps an `Option<X>` to `Option<Y>` using `Into`.
    fn map_into<Y, X: Into<Y>>(self) -> Option<Y>
    where
        Self: Is<Option<X>>,
    {
        self.into_self().map(X::into)
    }

    /// Maps an `Option<X>` to `Option<&Y>` using `AsRef`.
    fn map_as_ref<'a, Y: ?Sized, X: 'a + AsRef<Y>>(&'a self) -> Option<&'a Y>
    where
        Self: Borrow<Option<X>>,
    {
        self.borrow().as_ref().map(X::as_ref)
    }

    /// Takes the value out of `self`, replacing it with its default value.
    #[must_use]
    fn mem_take(&mut self) -> Self
    where
        Self: Default,
    {
        std::mem::take(self)
    }

    /// Asynchronously reads the metadata for a file at the path specified by `self`.
    #[cfg(feature = "async")]
    async fn metadata_async(&self) -> Result<Metadata, IoError>
    where
        Self: AsRef<Path>,
    {
        tokio::fs::metadata(self).await
    }

    /// Returns the dimensions of a `RopeSlice` as a `PointUsize`.
    ///
    /// The x coordinate is the maximum line width in extended grapheme clusters.
    /// The y coordinate is the number of lines.
    #[cfg(feature = "ropey")]
    fn num_lines_and_extended_graphemes<'a>(self) -> PointUsize
    where
        Self: Is<RopeSlice<'a>>,
    {
        let rope_slice = self.into_self();
        let y = rope_slice.len_lines();
        let x = rope_slice
            .lines()
            .map(|line_rope| line_rope.chunks().map(str::len_extended_graphemes).sum())
            .max()
            .unwrap_or(0);

        PointUsize::new(x, y)
    }

    /// Wraps `self` in `Ok`.
    fn ok<E>(self) -> Result<Self, E>
    where
        Self: Sized,
    {
        Ok(self)
    }

    /// Creates an iterator that yields `self` exactly once.
    fn once(self) -> Once<Self>
    where
        Self: Sized,
    {
        std::iter::once(self)
    }

    /// Opens a file at the path specified by `self` for reading.
    fn open(&self) -> Result<File, IoError>
    where
        Self: AsRef<Path>,
    {
        File::open(self)
    }

    /// Asynchronously opens a file at the path specified by `self` for reading.
    #[cfg(feature = "async")]
    async fn open_async(&self) -> Result<TokioFile, IoError>
    where
        Self: AsRef<Path>,
    {
        TokioFile::open(self).await
    }

    /// Wraps `self` in an `OptionalDisplay` for conditional display formatting.
    #[cfg(feature = "fmt")]
    fn optional_display(&self) -> OptionalDisplay<'_, Self> {
        OptionalDisplay::new(self)
    }

    /// Wraps `self` in an owned `Cow`.
    fn owned<B: ?Sized + ToOwned<Owned = Self>>(self) -> Cow<'static, B>
    where
        Self: Sized,
    {
        Cow::Owned(self)
    }

    /// Creates a tuple pair `(self, rhs)`.
    fn pair<T>(self, rhs: T) -> (Self, T)
    where
        Self: Sized,
    {
        (self, rhs)
    }

    /// Boxes and pins `self`.
    fn pin(self) -> Pin<Box<Self>>
    where
        Self: Sized,
    {
        Box::pin(self)
    }

    /// Composes two functions, piping the output of `self` into `func`.
    fn pipe<X, Y, Z, F: FnMut(Y) -> Z>(mut self, mut func: F) -> impl FnMut(X) -> Z
    where
        Self: Sized + FnMut(X) -> Y,
    {
        move |x| self(x).pipe_into(&mut func)
    }

    /// Wraps `self` in `Poll::Ready`.
    fn poll_ready(self) -> Poll<Self>
    where
        Self: Sized,
    {
        Poll::Ready(self)
    }

    /// Pipes `self` into a function, returning the function's result.
    fn pipe_into<T, F: FnOnce(Self) -> T>(self, func: F) -> T
    where
        Self: Sized,
    {
        func(self)
    }

    /// Wraps `self` in a Poem `Binary` payload.
    #[cfg(feature = "poem")]
    fn poem_binary(self) -> PoemBinary<Self>
    where
        Self: Sized,
    {
        PoemBinary(self)
    }

    /// Converts a byte vector into a Poem websocket binary message.
    #[cfg(feature = "poem")]
    fn poem_binary_message(self) -> PoemMessage
    where
        Self: Is<Vec<u8>>,
    {
        PoemMessage::Binary(self.into_self())
    }

    /// Wraps `self` in a Poem `Json` payload.
    #[cfg(feature = "poem")]
    fn poem_json(self) -> PoemJson<Self>
    where
        Self: Sized,
    {
        PoemJson(self)
    }

    /// Converts a stream into a Poem binary body.
    #[cfg(feature = "poem")]
    fn poem_stream_body<O: 'static + Into<Bytes>, E: 'static + Into<IoError>>(self) -> PoemBinary<PoemBody>
    where
        Self: 'static + Send + Sized + Stream<Item = Result<O, E>>,
    {
        PoemBody::from_bytes_stream(self).poem_binary()
    }

    /// Converts a string into a Poem websocket text message.
    #[cfg(feature = "poem")]
    fn poem_text_message(self) -> PoemMessage
    where
        Self: Is<String>,
    {
        PoemMessage::Text(self.into_self())
    }

    /// Prints `self` to stdout with a newline.
    fn println(&self)
    where
        Self: Display,
    {
        std::println!("{self}");
    }

    /// Prints `self` to stdout without a newline.
    fn print(&self)
    where
        Self: Display,
    {
        std::print!("{self}");
    }

    /// Pushes `self` onto the end of a vector.
    fn push_to(self, values: &mut Vec<Self>)
    where
        Self: Sized,
    {
        values.push(self);
    }

    /// Adds multiple query parameters with the same name to a reqwest request.
    #[cfg(feature = "reqwest")]
    fn query_all<T: Serialize>(self, name: &str, values: impl IntoIterator<Item = T>) -> RequestBuilder
    where
        Self: Is<RequestBuilder>,
    {
        let mut request_builder = self.into_self();

        for value in values {
            request_builder = request_builder.query_one(name, value);
        }

        request_builder
    }

    /// Adds a query parameter to a reqwest request if the value is `Some`.
    #[cfg(feature = "reqwest")]
    fn query_one<T: Serialize>(self, name: &str, value: impl Into<Option<T>>) -> RequestBuilder
    where
        Self: Is<RequestBuilder>,
    {
        let query: &[(&str, T)] = if let Some(value) = value.into() {
            &[(name, value)]
        } else {
            &[]
        };
        let request_builder = self.into_self();

        request_builder.query(query)
    }

    /// Asynchronously reads all bytes from `self` into a `String`.
    #[cfg(feature = "async")]
    async fn read_string_async(&mut self) -> Result<String, IoError>
    where
        Self: AsyncReadExt + Unpin,
    {
        let mut string = String::new();

        self.read_to_string(&mut string).await?;

        string.ok()
    }

    /// Asynchronously reads the contents of a file into a string.
    ///
    /// Returns a `ReadValue` containing both the path and the result.
    #[cfg(feature = "async")]
    async fn read_to_string_async(self) -> ReadValue<Self>
    where
        Self: AsRef<Path> + Sized,
    {
        let result = tokio::fs::read_to_string(self.as_ref()).await;

        ReadValue::new(self, result)
    }

    /// Asynchronously reads a file or stdin into a string.
    ///
    /// If `self` is `Some(path)`, reads from that path. If `None`, reads from stdin.
    #[cfg(feature = "async")]
    async fn read_to_string_else_stdin_async<P: AsRef<Path>>(self) -> ReadValue<Option<P>>
    where
        Self: Is<Option<P>> + Sized,
    {
        if let Some(filepath) = self.into_self() {
            tokio::fs::read_to_string(filepath.as_ref()).await.pair(filepath.some())
        } else {
            tokio::io::stdin()
                .buf_reader_async()
                .read_string_async()
                .await
                .pair(None)
        }
        .reversed()
        .into()
    }

    /// Creates a `Range<T>` starting at `self` with the specified length.
    fn range_from_len<T: Add<Output = T> + Copy>(self, len: impl Into<T>) -> Range<T>
    where
        Self: Into<T>,
    {
        let start = self.into();
        let end = start + len.into();

        start..end
    }

    /// Converts a `(width, height)` tuple into a ratatui `Rect` at origin `(0, 0)`.
    #[cfg(feature = "tui")]
    fn ratatui_rect(self) -> Rect
    where
        Self: Into<(u16, u16)>,
    {
        let (width, height) = self.into();
        let x = 0;
        let y = 0;

        Rect { x, y, width, height }
    }

    /// Wraps `self` in a `Ready` future that is immediately ready.
    fn ready(self) -> Ready<Self>
    where
        Self: Sized,
    {
        std::future::ready(self)
    }

    /// Creates an iterator that endlessly repeats `self`.
    fn repeat(self) -> Repeat<Self>
    where
        Self: Clone,
    {
        std::iter::repeat(self)
    }

    /// Reverses a tuple pair from `(X, Y)` to `(Y, X)`.
    fn reversed<X, Y>(self) -> (Y, X)
    where
        Self: Is<(X, Y)>,
    {
        let (x, y) = self.into_self();

        y.pair(x)
    }

    /// Runs a future for a specified duration.
    ///
    /// Returns `Err` if the future completes before the duration elapses.
    /// Returns `Ok(future)` if the duration elapses first, allowing the future to be resumed.
    #[cfg(feature = "async")]
    async fn run_for(mut self, duration: Duration) -> Result<Self, RunForError<Self::Output>>
    where
        Self: Future + Sized + Unpin,
    {
        tokio::select! {
            output = &mut self => RunForError::new(output).err(),
            () = tokio::time::sleep(duration) => self.ok(),
        }
    }

    /// Runs a future on a tokio `LocalSet`.
    #[cfg(feature = "async")]
    async fn run_local(self) -> Self::Output
    where
        Self: Future + Sized,
    {
        LocalSet::new().run_until(self).await
    }

    /// Removes a file at the path specified by `self`.
    fn remove_file(&self) -> Result<(), IoError>
    where
        Self: AsRef<Path>,
    {
        std::fs::remove_file(self)
    }

    /// Sends `self` as a response to a socket request.
    #[cfg(feature = "socket")]
    async fn respond_to<T: Request<Response = Self>>(
        &self,
        mut socket: impl BorrowMut<Socket>,
    ) -> Result<(), AnyhowError> {
        socket.borrow_mut().respond::<T>(self).await
    }

    /// Returns chunks starting at a specified extended grapheme index, saturating at the end.
    // TODO-ac2072:
    // - add [AsRopeSlice] trait that both [Rope] and [RopeSlice<'_>] implement
    // - i was doing this, but it didn't work due to some use of tempoarary variables error
    #[cfg(feature = "ropey")]
    fn saturating_chunks_at_extended_grapheme<'a>(self, extended_grapheme_index: usize) -> Chunks<'a>
    where
        Self: Is<RopeSlice<'a>>,
    {
        self.saturating_chunks_at_char(extended_grapheme_index)
    }

    /// Returns chunks starting at a specified char index, saturating at the end.
    // TODO-ac2072
    #[cfg(feature = "ropey")]
    fn saturating_chunks_at_char<'a>(self, char_index: usize) -> Chunks<'a>
    where
        Self: Is<RopeSlice<'a>>,
    {
        let rope_slice = self.into_self();
        let char_index = rope_slice.len_chars().min(char_index);

        rope_slice.chunks_at_char(char_index).0
    }

    /// Returns lines starting at a specified line index, saturating at the end.
    // TODO-ac2072
    #[cfg(feature = "ropey")]
    fn saturating_lines_at<'a>(self, line_index: usize) -> Lines<'a>
    where
        Self: Is<RopeSlice<'a>>,
    {
        let rope_slice = self.into_self();
        let line_index = rope_slice.len_lines().min(line_index);

        rope_slice.lines_at(line_index)
    }

    /// Performs saturating addition or subtraction in place, clamped to a maximum value.
    ///
    /// If `add` is true, adds `rhs`. If false, subtracts `rhs`. The result is clamped to `max_value`.
    #[cfg(feature = "num")]
    fn saturating_add_or_sub_in_place_with_max(&mut self, rhs: Self, max_value: Self, add: bool)
    where
        Self: Ord + SaturatingAdd + SaturatingSub + Sized,
    {
        let value = if add {
            self.saturating_add(&rhs)
        } else {
            self.saturating_sub(&rhs)
        };

        *self = value.min(max_value);
    }

    /// Sends `self` to a sink.
    #[cfg(feature = "async")]
    async fn send_to<T: Sink<Self> + Unpin>(self, mut sink: T) -> Result<(), T::Error>
    where
        Self: Sized,
    {
        sink.send(self).await
    }

    /// Sends `self` through a tokio oneshot channel.
    #[cfg(feature = "async")]
    fn send_to_oneshot(self, sender: OneshotSender<Self>) -> Result<(), AnyhowError>
    where
        Self: Sized,
    {
        // NOTE: drop error variant which wraps [Self] and may not implement [StdError]
        sender
            .send(self)
            .ok()
            .context("unable to send value over oneshot channel")
    }

    /// Creates a tokio `Sleep` future for the specified duration.
    #[cfg(feature = "async")]
    fn sleep(self) -> Sleep
    where
        Self: Is<Duration>,
    {
        tokio::time::sleep(self.into_self())
    }

    /// Wraps `self` in `Some`.
    fn some(self) -> Option<Self>
    where
        Self: Sized,
    {
        Some(self)
    }

    /// Spawns a future as a tokio task.
    #[cfg(feature = "async")]
    fn spawn_task(self) -> JoinHandle<Self::Output>
    where
        Self: 'static + Future + Sized + Send,
        Self::Output: 'static + Send,
    {
        tokio::spawn(self)
    }

    /// Finds the byte interval of a substring within `self`.
    ///
    /// Returns `Some((start, end))` if found, or `None` otherwise.
    // TODO-4eef0b: permit reverse search
    fn substr_interval(&self, query: &[u8]) -> Option<(usize, usize)>
    where
        Self: AsRef<[u8]>,
    {
        let bytes = self.as_ref();
        let predicate = |substr| substr == query;
        let query_len = query.len();
        let begin = bytes.windows(query_len).position(predicate)?;
        let end = begin + query_len;

        (begin, end).some()
    }

    /// Takes a value from a JSON object at the specified index, deserializing it.
    ///
    /// Replaces the value at the index with `Json::Null`.
    #[cfg(feature = "serde")]
    fn take_json<T: DeserializeOwned>(&mut self, index: impl Index) -> Result<T, SerdeJsonError>
    where
        Self: BorrowMut<Json>,
    {
        self.borrow_mut()
            .get_mut(index)
            .unwrap_or(&mut Json::Null)
            .mem_take()
            .into_value_from_json()
    }

    /// Wraps a future with a timeout.
    ///
    /// Returns an error if the future does not complete within the specified duration.
    #[cfg(feature = "async")]
    fn timeout(self, duration: Duration) -> Timeout<Self>
    where
        Self: Future + Sized,
    {
        tokio::time::timeout(duration, self)
    }

    /// Toggles a boolean value in place.
    fn toggle(&mut self)
    where
        Self: BorrowMut<bool>,
    {
        let bool_value = self.borrow_mut();

        *bool_value = !*bool_value;
    }

    /// Serializes `self` into a `serde_json::Value`.
    #[cfg(feature = "serde")]
    fn to_json(&self) -> Result<Json, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_value(self)
    }

    /// Serializes `self` into JSON as a byte vector.
    #[cfg(feature = "serde")]
    fn to_json_byte_str(&self) -> Result<Vec<u8>, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_vec(self)
    }

    /// Wraps `self` in a JSON object with a single key.
    #[cfg(feature = "serde")]
    fn to_json_object(&self, key: &str) -> Json
    where
        Self: Serialize,
    {
        serde_json::json!({key: self})
    }

    /// Serializes `self` into a JSON string.
    #[cfg(feature = "serde")]
    fn to_json_str(&self) -> Result<String, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_string(self)
    }

    /// Serializes `self` into MessagePack format as a byte vector.
    #[cfg(feature = "rmp")]
    fn to_rmp_byte_str(&self) -> Result<Vec<u8>, RmpEncodeError>
    where
        Self: Serialize,
    {
        rmp_serde::to_vec(self)
    }

    /// Converts a path to a `file://` URI.
    #[cfg(feature = "fs")]
    fn to_uri(&self) -> Result<String, IoError>
    where
        Self: AsRef<Utf8Path>,
    {
        "file://".cat(self.absolute_utf8()?).ok()
    }

    /// Deserializes a value from a JSON byte slice.
    #[cfg(feature = "serde")]
    fn to_value_from_json_slice<'a, T: Deserialize<'a>>(&'a self) -> Result<T, SerdeJsonError>
    where
        Self: AsRef<[u8]>,
    {
        serde_json::from_slice(self.as_ref())
    }

    /// Deserializes a value from a YAML byte slice.
    #[cfg(feature = "serde")]
    fn to_value_from_yaml_slice<'a, T: Deserialize<'a>>(&'a self) -> Result<T, SerdeYamlError>
    where
        Self: AsRef<[u8]>,
    {
        serde_yaml_ng::from_slice(self.as_ref())
    }

    /// Deserializes a value from a JSON reader.
    #[cfg(feature = "serde")]
    fn to_value_from_json_reader<T: DeserializeOwned>(self) -> Result<T, SerdeJsonError>
    where
        Self: Read + Sized,
    {
        serde_json::from_reader(self)
    }

    /// Deserializes a value from a MessagePack byte slice.
    #[cfg(feature = "rmp")]
    fn to_value_from_rmp_slice<'a, T: Deserialize<'a>>(&'a self) -> Result<T, RmpDecodeError>
    where
        Self: AsRef<[u8]>,
    {
        rmp_serde::from_slice(self.as_ref())
    }

    /// Converts `self` to another type by serializing to JSON and deserializing.
    #[cfg(feature = "serde")]
    fn to_value_from_value<T: DeserializeOwned>(&self) -> Result<T, SerdeJsonError>
    where
        Self: Serialize,
    {
        self.to_json()?.into_value_from_json()
    }

    /// Attempts to convert `self` to type `T` using `TryFrom<Self>`.
    fn try_convert<T: TryFrom<Self>>(self) -> Result<T, T::Error>
    where
        Self: Sized,
    {
        self.try_into()
    }

    /// Awaits all fallible futures in an iterator, short-circuiting on the first error.
    #[cfg(feature = "async")]
    async fn try_join_all<T, E>(self) -> Result<T, E>
    where
        Self: IntoIterator<Item: TryFuture> + Sized,
        T: FromIterator<<Self::Item as TryFuture>::Ok>,
        E: From<<Self::Item as TryFuture>::Error>,
    {
        futures::future::try_join_all(self)
            .await?
            .into_iter()
            .collect::<T>()
            .ok()
    }

    /// Awaits a `JoinHandle` and unwraps its `Result`.
    ///
    /// Propagates both join errors and the task's error result.
    #[cfg(feature = "async")]
    async fn try_wait<T, E: 'static + Send + Sync>(self) -> Result<T, AnyhowError>
    where
        Self: Is<JoinHandle<Result<T, E>>>,
        AnyhowError: From<E>,
    {
        self.into_self().await??.ok()
    }

    /// Returns the type name of `Self` as a string.
    #[must_use]
    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Does nothing. Useful for explicitly ignoring a value.
    fn unit(&self) {}

    /// Awaits a future returning `Option<T>`, unwrapping or pending forever if `None`.
    async fn wait_then_unwrap_or_pending<T>(self) -> T
    where
        Self: Future<Output = Option<T>> + Sized,
    {
        match self.await {
            Some(value) => value,
            None => std::future::pending().await,
        }
    }

    /// Awaits an optional future, or pends forever if `None`.
    async fn unwrap_or_pending_then_wait<F: Future + Unpin>(&mut self) -> F::Output
    where
        Self: BorrowMut<Option<F>>,
    {
        if let Some(future) = self.borrow_mut().as_mut() {
            future.await
        } else {
            std::future::pending().await
        }
    }

    /// Returns `value`, ignoring `self`. Useful for side-effecting method chains.
    fn with<T>(&self, value: T) -> T {
        value
    }

    /// Pushes an item onto a vector and returns the modified vector.
    fn with_item_pushed<T>(self, item: T) -> Vec<T>
    where
        Self: Is<Vec<T>>,
    {
        let mut vec = self.into_self();

        vec.push(item);

        vec
    }

    /// Appends a string slice to a `String` and returns the modified string.
    fn with_str_pushed(self, rhs: &str) -> String
    where
        Self: Is<String>,
    {
        let mut string = self.into_self();

        string.push_str(rhs);

        string
    }

    /// Serializes `self` as JSON and writes it to a writer.
    #[cfg(feature = "serde")]
    fn write_as_json_to<T: Write>(&self, writer: T) -> Result<(), SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_writer(writer, self)
    }

    /// Writes all bytes to `self` and flushes the writer.
    fn write_all_and_flush<T: AsRef<[u8]>>(&mut self, byte_str: T) -> Result<(), IoError>
    where
        Self: Write + Unpin,
    {
        self.write_all(byte_str.as_ref())?;

        self.flush()
    }

    /// Asynchronously writes all bytes to `self` and flushes the writer.
    #[cfg(feature = "async")]
    async fn write_all_and_flush_async<T: AsRef<[u8]>>(&mut self, byte_str: T) -> Result<(), IoError>
    where
        Self: AsyncWriteExt + Unpin,
    {
        self.write_all(byte_str.as_ref()).await?;

        self.flush().await
    }
}

impl<T: ?Sized> Utils for T {}
