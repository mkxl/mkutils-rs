use crate::{
    fmt::{Debugged, OptionalDisplay},
    geometry::PointUsize,
    is::Is,
    read_value::ReadValue,
    status::Status,
};
use anyhow::{Context, Error as AnyhowError};
use bytes::{Buf, Bytes};
use camino::{Utf8Path, Utf8PathBuf};
use futures::{Sink, SinkExt, Stream, StreamExt, TryFuture, stream::Filter};
use num::traits::{SaturatingAdd, SaturatingSub};
use poem::{Body as PoemBody, Endpoint, IntoResponse, web::websocket::Message as PoemMessage};
use poem_openapi::{
    error::ParseRequestPayloadError,
    payload::{Binary as PoemBinary, Json as PoemJson},
};
use postcard::Error as PostcardError;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
};
use reqwest::{RequestBuilder, Response};
use ropey::{
    Rope, RopeSlice,
    iter::{Chunks, Lines},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Error as SerdeJsonError, Value as Json, value::Index};
use std::{
    borrow::{Borrow, BorrowMut, Cow},
    collections::HashMap,
    error::Error as StdError,
    ffi::OsStr,
    fmt::{Debug, Display},
    fs::{File, Metadata},
    future::{Future, Ready},
    hash::Hash,
    io::{BufReader, BufWriter, Error as IoError, Read, Write},
    iter::Once,
    marker::Unpin,
    ops::{Add, ControlFlow, Range},
    path::{Path, PathBuf},
    pin::Pin,
    str::Utf8Error,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::Poll,
    time::Duration,
};
use tokio::{
    fs::File as TokioFile,
    io::{
        AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader as TokioBufReader, BufWriter as TokioBufWriter,
    },
    sync::oneshot::Sender as OneshotSender,
    task::{JoinHandle, LocalSet},
};
use tokio_util::{
    codec::{Framed, LengthDelimitedCodec, LinesCodec},
    io::StreamReader,
};
use tracing::Level;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

macro_rules! try_get {
    ($expr:expr, $method:ident, $key:ident) => {{
        // NOTE: can't have [$key] in format string
        match $expr.$method($key) {
            Some(value) => value.ok(),
            None => ::anyhow::bail!("key {} is not present in the hashmap", $key),
        }
    }};
}

#[allow(async_fn_in_trait)]
pub trait Utils {
    fn absolute(&self) -> Result<Cow<'_, Path>, IoError>
    where
        Self: AsRef<Path>,
    {
        let path = self.as_ref();

        if path.is_absolute() {
            path.borrowed().ok()
        } else {
            std::path::absolute(path)?.owned::<Path>().ok()
        }
    }

    async fn achain<T: Future>(self, rhs: T) -> T::Output
    where
        Self: Future + Sized,
    {
        self.await;

        rhs.await
    }

    fn add_span<'a, T: Into<Span<'a>>>(self, span: T) -> Line<'a>
    where
        Self: Into<Line<'a>>,
    {
        let mut line = self.into();

        line.spans.push(span.into());

        line
    }

    fn anyhow_result<T, E: Into<AnyhowError>>(self) -> Result<T, AnyhowError>
    where
        Self: Is<Result<T, E>>,
    {
        self.into_self().map_err(E::into)
    }

    fn arc(self) -> Arc<Self>
    where
        Self: Sized,
    {
        Arc::new(self)
    }

    fn as_borrowed<'a, B: ?Sized + ToOwned>(&'a self) -> &'a B
    where
        Self: Borrow<Cow<'a, B>>,
    {
        self.borrow().borrow()
    }

    fn as_utf8(&self) -> Result<&str, Utf8Error>
    where
        Self: AsRef<[u8]>,
    {
        std::str::from_utf8(self.as_ref())
    }

    fn as_utf8_path(&self) -> &Utf8Path
    where
        Self: AsRef<Utf8Path>,
    {
        self.as_ref()
    }

    fn as_slice(&self) -> RopeSlice<'_>
    where
        Self: Borrow<Rope>,
    {
        self.borrow().slice(..)
    }

    fn borrowed(&self) -> Cow<'_, Self>
    where
        Self: ToOwned,
    {
        Cow::Borrowed(self)
    }

    fn buf_reader(self) -> BufReader<Self>
    where
        Self: Read + Sized,
    {
        BufReader::new(self)
    }

    fn buf_reader_async(self) -> TokioBufReader<Self>
    where
        Self: AsyncRead + Sized,
    {
        TokioBufReader::new(self)
    }

    fn buf_writer(self) -> BufWriter<Self>
    where
        Self: Write + Sized,
    {
        BufWriter::new(self)
    }

    fn buf_writer_async(self) -> TokioBufWriter<Self>
    where
        Self: AsyncWrite + Sized,
    {
        TokioBufWriter::new(self)
    }

    fn cast_ref<T>(&self) -> &T {
        let ptr = std::ptr::from_ref(self).cast::<T>();
        let value = unsafe { ptr.as_ref() };

        value.unwrap()
    }

    fn cat<T: Display>(&self, rhs: T) -> String
    where
        Self: Display,
    {
        std::format!("{self}{rhs}")
    }

    fn check_next<T>(self) -> Result<T, AnyhowError>
    where
        Self: Is<Option<T>>,
    {
        match self.into_self() {
            Some(item) => item.ok(),
            None => anyhow::bail!(
                "sequence of {type_name} items is exhausted",
                type_name = std::any::type_name::<T>(),
            ),
        }
    }

    // NOTE: [https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.error_for_status]
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

    fn convert<T: From<Self>>(self) -> T
    where
        Self: Sized,
    {
        self.into()
    }

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

    fn create(&self) -> Result<File, IoError>
    where
        Self: AsRef<Path>,
    {
        File::create(self)
    }

    fn create_dir_all(&self) -> Result<(), IoError>
    where
        Self: AsRef<Path>,
    {
        std::fs::create_dir_all(self)
    }

    fn debug(&self) -> Debugged<'_, Self> {
        Debugged::new(self)
    }

    fn err<T>(self) -> Result<T, Self>
    where
        Self: Sized,
    {
        Err(self)
    }

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
            path_str.convert::<&Utf8Path>().borrowed()
        }
    }

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
            } else if relative_path_str.convert::<&Utf8Path>().is_absolute() {
                "~".cat(relative_path_str).convert::<Utf8PathBuf>().owned()
            } else {
                path_str.convert::<&Utf8Path>().borrowed()
            }
        } else {
            path_str.convert::<&Utf8Path>().borrowed()
        }
    }

    fn extended_graphemes(&self) -> Graphemes<'_>
    where
        Self: AsRef<str>,
    {
        self.as_ref().graphemes(true)
    }

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

    fn file_name_ok(&self) -> Result<&OsStr, AnyhowError>
    where
        Self: AsRef<Path>,
    {
        self.as_ref().file_name().context("path has no file_name")
    }

    #[must_use]
    fn home_dirpath() -> Option<Utf8PathBuf> {
        home::home_dir()?.try_convert::<Utf8PathBuf>().ok()
    }

    fn immutable(&mut self) -> &Self {
        self
    }

    fn inc(&self) -> usize
    where
        Self: Borrow<AtomicUsize>,
    {
        self.borrow().fetch_add(1, Ordering::SeqCst)
    }

    fn insert_mut<'a, K: 'a + Eq + Hash, V>(&'a mut self, key: K, value: V) -> &'a mut V
    where
        Self: BorrowMut<HashMap<K, V>>,
    {
        self.borrow_mut().entry(key).insert_entry(value).into_mut()
    }

    fn into_break<C>(self) -> ControlFlow<Self, C>
    where
        Self: Sized,
    {
        ControlFlow::Break(self)
    }

    fn into_continue<B>(self) -> ControlFlow<B, Self>
    where
        Self: Sized,
    {
        ControlFlow::Continue(self)
    }

    fn into_endpoint(self) -> impl Endpoint<Output = Self>
    where
        Self: Clone + IntoResponse + Sync,
    {
        let func = move |_request| self.clone();

        poem::endpoint::make_sync(func)
    }

    fn into_line<'a>(self) -> Line<'a>
    where
        Self: Into<Cow<'a, str>>,
    {
        self.into().into()
    }

    fn into_stream_reader<B: Buf, E: Into<IoError>>(self) -> StreamReader<Self, B>
    where
        Self: Sized + Stream<Item = Result<B, E>>,
    {
        StreamReader::new(self)
    }

    fn into_length_delimited_frames(self) -> Framed<Self, LengthDelimitedCodec>
    where
        Self: Sized,
    {
        Framed::new(self, LengthDelimitedCodec::new())
    }

    fn into_line_frames(self) -> Framed<Self, LinesCodec>
    where
        Self: Sized,
    {
        Framed::new(self, LinesCodec::new())
    }

    // NOTE: [https://docs.rs/poem-openapi/latest/src/poem_openapi/payload/json.rs.html]
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

    async fn into_select<T: Future<Output = Self::Output>>(self, rhs: T) -> Self::Output
    where
        Self: Future + Sized,
    {
        tokio::select! {
            value = self => value,
            value = rhs => value,
        }
    }

    fn into_status<T, E>(self) -> Status<T, E>
    where
        Self: Is<Result<T, E>>,
    {
        Status(self.into_self())
    }

    fn into_string(self) -> Result<String, AnyhowError>
    where
        Self: Is<PathBuf>,
    {
        match self.into_self().into_os_string().into_string() {
            Ok(string) => string.ok(),
            Err(os_string) => os_string.invalid_utf8_err(),
        }
    }

    fn into_value_from_json<T: DeserializeOwned>(self) -> Result<T, SerdeJsonError>
    where
        Self: Is<Json>,
    {
        serde_json::from_value(self.into_self())
    }

    fn invalid_utf8_err<T>(&self) -> Result<T, AnyhowError>
    where
        Self: Debug,
    {
        anyhow::bail!("{self:?} is not valid utf-8")
    }

    fn io_error(self) -> IoError
    where
        Self: Into<Box<dyn StdError + Send + Sync>>,
    {
        IoError::other(self)
    }

    fn io_result<T, E: Into<Box<dyn StdError + Send + Sync>>>(self) -> Result<T, IoError>
    where
        Self: Is<Result<T, E>>,
    {
        self.into_self().map_err(E::io_error)
    }

    async fn join_all<T>(self) -> Vec<T>
    where
        Self: IntoIterator<Item: Future> + Sized,
        Self::Item: Future<Output = T>,
    {
        futures::future::join_all(self).await
    }

    fn len_extended_graphemes(&self) -> usize
    where
        Self: AsRef<str>,
    {
        self.as_ref().extended_graphemes().count()
    }

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

    fn log_error(&self)
    where
        Self: Display,
    {
        tracing::warn!(error = %self, "error: {self:#}");
    }

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

    fn map_collect<Y, T: FromIterator<Y>>(self, func: impl FnMut(Self::Item) -> Y) -> T
    where
        Self: IntoIterator + Sized,
    {
        self.into_iter().map(func).collect::<T>()
    }

    fn map_into<Y, X: Into<Y>>(self) -> Option<Y>
    where
        Self: Is<Option<X>>,
    {
        self.into_self().map(X::into)
    }

    fn map_as_ref<'a, Y: ?Sized, X: 'a + AsRef<Y>>(&'a self) -> Option<&'a Y>
    where
        Self: Borrow<Option<X>>,
    {
        self.borrow().as_ref().map(X::as_ref)
    }

    #[must_use]
    fn mem_take(&mut self) -> Self
    where
        Self: Default,
    {
        std::mem::take(self)
    }

    async fn metadata_async(&self) -> Result<Metadata, IoError>
    where
        Self: AsRef<Path>,
    {
        tokio::fs::metadata(self).await
    }

    fn ok<E>(self) -> Result<Self, E>
    where
        Self: Sized,
    {
        Ok(self)
    }

    fn once(self) -> Once<Self>
    where
        Self: Sized,
    {
        std::iter::once(self)
    }

    fn open(&self) -> Result<File, IoError>
    where
        Self: AsRef<Path>,
    {
        File::open(self)
    }

    async fn open_async(&self) -> Result<TokioFile, IoError>
    where
        Self: AsRef<Path>,
    {
        TokioFile::open(self).await
    }

    fn optional_display(&self) -> OptionalDisplay<'_, Self> {
        OptionalDisplay::new(self)
    }

    fn owned<B: ?Sized + ToOwned<Owned = Self>>(self) -> Cow<'static, B>
    where
        Self: Sized,
    {
        Cow::Owned(self)
    }

    fn pair<T>(self, rhs: T) -> (Self, T)
    where
        Self: Sized,
    {
        (self, rhs)
    }

    fn pin(self) -> Pin<Box<Self>>
    where
        Self: Sized,
    {
        Box::pin(self)
    }

    fn pipe<X, Y, Z, F: FnMut(Y) -> Z>(mut self, mut func: F) -> impl FnMut(X) -> Z
    where
        Self: Sized + FnMut(X) -> Y,
    {
        move |x| self(x).pipe_into(&mut func)
    }

    fn poll_ready(self) -> Poll<Self>
    where
        Self: Sized,
    {
        Poll::Ready(self)
    }

    fn pipe_into<T, F: FnOnce(Self) -> T>(self, func: F) -> T
    where
        Self: Sized,
    {
        func(self)
    }

    fn poem_binary(self) -> PoemBinary<Self>
    where
        Self: Sized,
    {
        PoemBinary(self)
    }

    fn poem_binary_message(self) -> PoemMessage
    where
        Self: Is<Vec<u8>>,
    {
        PoemMessage::Binary(self.into_self())
    }

    fn poem_json(self) -> PoemJson<Self>
    where
        Self: Sized,
    {
        PoemJson(self)
    }

    fn poem_stream_body<O: 'static + Into<Bytes>, E: 'static + Into<IoError>>(self) -> PoemBinary<PoemBody>
    where
        Self: 'static + Send + Sized + Stream<Item = Result<O, E>>,
    {
        PoemBody::from_bytes_stream(self).poem_binary()
    }

    fn poem_text_message(self) -> PoemMessage
    where
        Self: Is<String>,
    {
        PoemMessage::Text(self.into_self())
    }

    fn println(&self)
    where
        Self: Display,
    {
        std::println!("{self}");
    }

    fn print(&self)
    where
        Self: Display,
    {
        std::print!("{self}");
    }

    fn push_to(self, values: &mut Vec<Self>)
    where
        Self: Sized,
    {
        values.push(self);
    }

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

    async fn read_string_async(&mut self) -> Result<String, AnyhowError>
    where
        Self: AsyncReadExt + Unpin,
    {
        let mut string = String::new();

        self.read_to_string(&mut string).await?;

        string.ok()
    }

    async fn read_to_string_async(self) -> ReadValue<Self>
    where
        Self: AsRef<Path> + Sized,
    {
        let result = tokio::fs::read_to_string(self.as_ref()).await;

        ReadValue::new(self, result)
    }

    fn range_from_len<T: Add<Output = T> + Copy>(self, len: impl Into<T>) -> Range<T>
    where
        Self: Into<T>,
    {
        let start = self.into();
        let end = start + len.into();

        start..end
    }

    fn ratatui_rect(self) -> Rect
    where
        Self: Into<(u16, u16)>,
    {
        let (width, height) = self.into();
        let x = 0;
        let y = 0;

        Rect { x, y, width, height }
    }

    fn ready(self) -> Ready<Self>
    where
        Self: Sized,
    {
        std::future::ready(self)
    }

    async fn run_for(mut self, duration: Duration) -> Result<Self, Self::Output>
    where
        Self: Future + Sized + Unpin,
    {
        tokio::select! {
            output = &mut self => output.err(),
            () = tokio::time::sleep(duration) => self.ok(),
        }
    }

    async fn run_local(self) -> Self::Output
    where
        Self: Future + Sized,
    {
        LocalSet::new().run_until(self).await
    }

    fn remove_file(&self) -> Result<(), IoError>
    where
        Self: AsRef<Path>,
    {
        std::fs::remove_file(self)
    }

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

    // TODO-ac2072:
    // - add [AsRopeSlice] trait that both [Rope] and [RopeSlice<'_>] implement
    // - i was doing this, but it didn't work due to some use of tempoarary variables error
    fn saturating_chunks_at_extended_grapheme<'a>(self, extended_grapheme_index: usize) -> Chunks<'a>
    where
        Self: Is<RopeSlice<'a>>,
    {
        self.saturating_chunks_at_char(extended_grapheme_index)
    }

    // TODO-ac2072
    fn saturating_chunks_at_char<'a>(self, char_index: usize) -> Chunks<'a>
    where
        Self: Is<RopeSlice<'a>>,
    {
        let rope_slice = self.into_self();
        let char_index = rope_slice.len_chars().min(char_index);

        rope_slice.chunks_at_char(char_index).0
    }

    // TODO-ac2072
    fn saturating_lines_at<'a>(self, line_index: usize) -> Lines<'a>
    where
        Self: Is<RopeSlice<'a>>,
    {
        let rope_slice = self.into_self();
        let line_index = rope_slice.len_lines().min(line_index);

        rope_slice.lines_at(line_index)
    }

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

    async fn send_to<T: Sink<Self> + Unpin>(self, mut sink: T) -> Result<(), T::Error>
    where
        Self: Sized,
    {
        sink.send(self).await
    }

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

    fn some(self) -> Option<Self>
    where
        Self: Sized,
    {
        Some(self)
    }

    fn spawn_task(self) -> JoinHandle<Self::Output>
    where
        Self: 'static + Future + Sized + Send,
        Self::Output: 'static + Send,
    {
        tokio::spawn(self)
    }

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

    fn take_json<T: DeserializeOwned>(&mut self, index: impl Index) -> Result<T, SerdeJsonError>
    where
        Self: BorrowMut<Json>,
    {
        self.borrow_mut()
            .get_mut(index)
            .map_or_default(std::mem::take)
            .into_value_from_json()
    }

    fn to_json(&self) -> Result<Json, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_value(self)
    }

    fn to_json_byte_str(&self) -> Result<Vec<u8>, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_vec(self)
    }

    fn to_postcard_byte_str(&self) -> Result<Vec<u8>, PostcardError>
    where
        Self: Serialize,
    {
        postcard::to_stdvec(self)
    }

    fn to_json_object(&self, key: &str) -> Json
    where
        Self: Serialize,
    {
        serde_json::json!({key: self})
    }

    fn to_json_str(&self) -> Result<String, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_string(self)
    }

    #[allow(clippy::option_if_let_else)]
    fn to_str_ok(&self) -> Result<&str, AnyhowError>
    where
        Self: AsRef<OsStr>,
    {
        let path = self.as_ref();

        match path.to_str() {
            Some(string) => string.ok(),
            None => path.invalid_utf8_err(),
        }
    }

    fn to_uri(&self) -> Result<String, AnyhowError>
    where
        Self: AsRef<Path>,
    {
        "file://".cat(self.absolute()?.as_os_str().to_str_ok()?).ok()
    }

    fn to_value_from_json_byte_str<'a, T: Deserialize<'a>>(&'a self) -> Result<T, SerdeJsonError>
    where
        Self: AsRef<[u8]>,
    {
        serde_json::from_slice(self.as_ref())
    }

    fn to_value_from_json_reader<T: DeserializeOwned>(self) -> Result<T, SerdeJsonError>
    where
        Self: Read + Sized,
    {
        serde_json::from_reader(self)
    }

    fn to_value_from_postcard_byte_str<'a, T: Deserialize<'a>>(&'a self) -> Result<T, PostcardError>
    where
        Self: AsRef<[u8]>,
    {
        postcard::from_bytes(self.as_ref())
    }

    fn to_value_from_value<T: DeserializeOwned>(&self) -> Result<T, SerdeJsonError>
    where
        Self: Serialize,
    {
        self.to_json()?.into_value_from_json()
    }

    fn try_convert<T: TryFrom<Self>>(self) -> Result<T, T::Error>
    where
        Self: Sized,
    {
        self.try_into()
    }

    // NOTE: would prefer to do [Result<Vec<Self::Item::Ok>, Self::Item::Error>] but getting
    // [error[E0223]: ambiguous associated type]
    async fn try_join_all<T, E>(self) -> Result<Vec<T>, E>
    where
        Self: IntoIterator + Sized,
        Self::Item: TryFuture<Ok = T, Error = E>,
    {
        futures::future::try_join_all(self).await
    }

    fn try_get<'a, V, Q: Display + Eq + Hash, K: 'a + Borrow<Q> + Eq + Hash>(
        &'a self,
        key: &Q,
    ) -> Result<&'a V, AnyhowError>
    where
        Self: Borrow<HashMap<K, V>>,
    {
        try_get!(self.borrow(), get, key)
    }

    fn try_get_mut<'a, V, Q: Display + Eq + Hash, K: 'a + Borrow<Q> + Eq + Hash>(
        &'a mut self,
        key: &Q,
    ) -> Result<&'a mut V, AnyhowError>
    where
        Self: BorrowMut<HashMap<K, V>>,
    {
        try_get!(self.borrow_mut(), get_mut, key)
    }

    async fn try_wait<T, E: 'static + Send + Sync>(self) -> Result<T, AnyhowError>
    where
        Self: Is<JoinHandle<Result<T, E>>>,
        AnyhowError: From<E>,
    {
        self.into_self().await??.ok()
    }

    fn unit(&self) {}

    async fn unwrap_or_pending<T>(self) -> T
    where
        Self: Future<Output = Option<T>> + Sized,
    {
        match self.await {
            Some(value) => value,
            None => std::future::pending().await,
        }
    }

    fn with<T>(&self, value: T) -> T {
        value
    }

    fn with_item_pushed<T>(self, item: T) -> Vec<T>
    where
        Self: Is<Vec<T>>,
    {
        let mut vec = self.into_self();

        vec.push(item);

        vec
    }

    fn with_str_pushed(self, rhs: &str) -> String
    where
        Self: Is<String>,
    {
        let mut string = self.into_self();

        string.push_str(rhs);

        string
    }

    fn write_as_json_to<T: Write>(&self, writer: T) -> Result<(), SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_writer(writer, self)
    }

    fn write_all_and_flush<T: AsRef<[u8]>>(&mut self, byte_str: T) -> Result<(), IoError>
    where
        Self: Write + Unpin,
    {
        self.write_all(byte_str.as_ref())?;

        self.flush()
    }

    async fn write_all_and_flush_async<T: AsRef<[u8]>>(&mut self, byte_str: T) -> Result<(), IoError>
    where
        Self: AsyncWriteExt + Unpin,
    {
        self.write_all(byte_str.as_ref()).await?;

        self.flush().await
    }
}

impl<T: ?Sized> Utils for T {}
