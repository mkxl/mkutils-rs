use crate::{as_rope_slice::AsRopeSlice, debugged::Debugged, geometry::PointUsize, is::Is, status::Status};
use anyhow::{Context, Error as AnyhowError};
use futures::{Sink, SinkExt, StreamExt, TryFuture};
use num::traits::{SaturatingAdd, SaturatingSub};
use poem::{Endpoint, IntoResponse};
use poem_openapi::{error::ParseRequestPayloadError, payload::Json as PoemJson};
use postcard::Error as PostcardError;
use reqwest::{RequestBuilder, Response};
use ropey::{
    Rope,
    iter::{Chunks, Lines},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Error as SerdeJsonError, Value as Json};
use std::{
    borrow::{Borrow, BorrowMut, Cow},
    collections::HashMap,
    ffi::OsStr,
    fmt::{Debug, Display},
    fs::{File, Metadata},
    future::{Future, Ready},
    hash::Hash,
    io::{BufReader, BufWriter, Error as IoError, Read, Write},
    iter::Once,
    marker::Unpin,
    ops::ControlFlow,
    path::{Path, PathBuf},
    pin::Pin,
    str::Utf8Error,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::{
    fs::File as TokioFile,
    io::{
        AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader as TokioBufReader, BufWriter as TokioBufWriter,
    },
    sync::oneshot::Sender as OneshotSender,
    task::JoinHandle,
};
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

    fn as_utf8(&self) -> Result<&str, Utf8Error>
    where
        Self: AsRef<[u8]>,
    {
        std::str::from_utf8(self.as_ref())
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

    fn cat<T: Display>(&self, rhs: T) -> String
    where
        Self: Display,
    {
        std::format!("{self}{rhs}")
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
        Debugged(self)
    }

    fn err<T>(self) -> Result<T, Self>
    where
        Self: Sized,
    {
        Err(self)
    }

    fn extended_graphemes(&self) -> Graphemes<'_>
    where
        Self: AsRef<str>,
    {
        self.as_ref().graphemes(true)
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

    fn invalid_utf8_err<T>(&self) -> Result<T, AnyhowError>
    where
        Self: Debug,
    {
        anyhow::bail!("{self:?} is not valid utf-8")
    }

    fn file_name_ok(&self) -> Result<&OsStr, AnyhowError>
    where
        Self: AsRef<Path>,
    {
        self.as_ref().file_name().context("path has no file_name")
    }

    fn len_extended_graphemes(&self) -> usize
    where
        Self: AsRef<str>,
    {
        self.as_ref().extended_graphemes().count()
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
        match self.borrow() {
            Some(x) => x.as_ref().some(),
            None => None,
        }
    }

    async fn metadata_async(&self) -> Result<Metadata, IoError>
    where
        Self: AsRef<Path>,
    {
        tokio::fs::metadata(self).await
    }

    async fn next_item_async(&mut self) -> Result<Self::Item, AnyhowError>
    where
        Self: StreamExt + Unpin,
    {
        match self.next().await {
            Some(item) => item.ok(),
            None => anyhow::bail!(
                "{type_name} stream is empty",
                type_name = std::any::type_name::<Self::Item>()
            ),
        }
    }

    fn next_item(&mut self) -> Result<Self::Item, AnyhowError>
    where
        Self: Iterator,
    {
        match self.next() {
            Some(item) => item.ok(),
            None => anyhow::bail!(
                "{type_name} iterator is empty",
                type_name = std::any::type_name::<Self::Item>()
            ),
        }
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

    fn pipe<T, F: FnOnce(Self) -> T>(self, func: F) -> T
    where
        Self: Sized,
    {
        func(self)
    }

    fn poem_json(self) -> PoemJson<Self>
    where
        Self: Sized,
    {
        PoemJson(self)
    }

    fn println(&self)
    where
        Self: Display,
    {
        std::println!("{self}");
    }

    fn query_once<T: Serialize, S: Into<Option<T>>>(self, name: &str, value: S) -> RequestBuilder
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

    async fn read_to_string_async(&self) -> Result<String, IoError>
    where
        Self: AsRef<Path>,
    {
        tokio::fs::read_to_string(self).await
    }

    fn ready(self) -> Ready<Self>
    where
        Self: Sized,
    {
        std::future::ready(self)
    }

    fn remove_file(&self) -> Result<(), IoError>
    where
        Self: AsRef<Path>,
    {
        std::fs::remove_file(self)
    }

    fn num_lines_and_extended_graphemes(&self) -> PointUsize
    where
        Self: Borrow<Rope>,
    {
        let rope = self.borrow();
        let y = rope.len_lines();
        let x = rope
            .lines()
            .map(|line_rope| line_rope.chunks().map(str::len_extended_graphemes).sum())
            .max()
            .unwrap_or(0);

        PointUsize::new(x, y)
    }

    fn saturating_chunks_at_extended_grapheme(&self, _extended_grapheme_idx: usize) -> Chunks<'_>
    where
        Self: AsRopeSlice,
    {
        std::todo!()
    }

    fn saturating_chunks_at_char(&self, char_idx: usize) -> Chunks<'_>
    where
        Self: AsRopeSlice,
    {
        let rope_slice = self.as_rope_slice();
        let char_idx = rope_slice.len_chars().min(char_idx);

        rope_slice.chunks_at_char(char_idx).0
    }

    fn saturating_lines_at(&self, line_idx: usize) -> Lines<'_>
    where
        Self: AsRopeSlice,
    {
        let rope_slice = self.as_rope_slice();
        let line_idx = rope_slice.len_lines().min(line_idx);

        rope_slice.lines_at(line_idx)
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

    fn to_json_str(&self) -> Result<String, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_string(self)
    }

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
        let value = serde_json::to_value(self)?;
        let value = serde_json::from_value::<T>(value)?;

        value.ok()
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

    fn write_as_json_to<T: Write>(&self, writer: T) -> Result<(), SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_writer(writer, self)
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
