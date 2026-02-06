#[cfg(feature = "serde")]
use crate::as_valuable::AsValuable;
#[cfg(feature = "fmt")]
use crate::fmt::{Debugged, OptionalDisplay};
#[cfg(feature = "tui")]
use crate::geometry::Orientation;
#[cfg(any(feature = "ropey", feature = "tui"))]
use crate::geometry::PointUsize;
use crate::is::Is;
#[cfg(feature = "output")]
use crate::output::Output;
#[cfg(feature = "process")]
use crate::process::ProcessBuilder;
#[cfg(feature = "ropey")]
use crate::rope_builder::RopeBuilder;
#[cfg(feature = "tui")]
use crate::scrollable::{ScrollCount, ScrollWhen, Scrollable};
#[cfg(any(feature = "serde", feature = "tui"))]
use crate::seq_visitor::SeqVisitor;
#[cfg(feature = "socket")]
use crate::socket::{Request, Socket};
#[cfg(feature = "tracing")]
use crate::status::Status;
#[cfg(any(feature = "ropey", feature = "tui"))]
use crate::transpose::Transpose;
#[cfg(feature = "async")]
use crate::{into_stream::IntoStream, read_value::ReadValue, run_for::RunForError};
#[cfg(any(
    feature = "async",
    feature = "fs",
    feature = "process",
    feature = "reqwest",
    feature = "socket",
    feature = "tui",
))]
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
use futures::{Sink, SinkExt, StreamExt, TryFuture, future::Either, stream::Filter, stream::FuturesUnordered};
#[cfg(feature = "tui")]
use num::traits::{SaturatingAdd, SaturatingSub};
#[cfg(any(feature = "ropey", feature = "misc", feature = "tui"))]
use num::{Bounded, NumCast, ToPrimitive, Zero};
#[cfg(feature = "tui")]
use palette::IntoColor;
#[cfg(feature = "poem")]
use poem::{Body as PoemBody, Endpoint, IntoResponse, web::websocket::Message as PoemMessage};
#[cfg(feature = "poem")]
use poem_openapi::{error::ParseRequestPayloadError, payload::Binary as PoemBinary, payload::Json as PoemJson};
#[cfg(feature = "tui")]
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Widget},
};
#[cfg(feature = "reqwest")]
use reqwest::{RequestBuilder, Response};
#[cfg(feature = "rmp")]
use rmp_serde::{decode::Error as RmpDecodeError, encode::Error as RmpEncodeError};
#[cfg(feature = "ropey")]
use ropey::{Rope, RopeSlice, iter::Chunks, iter::Lines};
#[cfg(any(feature = "rmp", feature = "serde", feature = "tui"))]
use serde::Deserialize;
#[cfg(any(feature = "serde", feature = "tui"))]
use serde::Deserializer;
#[cfg(any(feature = "reqwest", feature = "rmp", feature = "serde"))]
use serde::Serialize;
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(any(feature = "poem", feature = "serde"))]
use serde_json::Error as SerdeJsonError;
#[cfg(feature = "serde")]
use serde_json::{Value as Json, value::Index};
#[cfg(feature = "serde")]
use serde_yaml_ng::Error as SerdeYamlError;
#[cfg(any(
    feature = "async",
    feature = "fs",
    feature = "process",
    feature = "reqwest",
    feature = "socket",
    feature = "tui",
))]
use std::fmt::Debug;
#[cfg(feature = "fs")]
use std::path::PathBuf;
#[cfg(feature = "process")]
use std::process::ExitStatus;
use std::{
    borrow::{Borrow, BorrowMut, Cow},
    collections::HashMap,
    error::Error as StdError,
    fmt::Display,
    fs::File,
    future::Ready,
    hash::Hash,
    io::{BufReader, BufWriter, Error as IoError, Read, Write},
    iter::{Once, Repeat},
    mem::ManuallyDrop,
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
#[cfg(feature = "async")]
use std::{fs::Metadata, time::Duration};
#[cfg(any(feature = "async", feature = "process"))]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(any(feature = "async", feature = "ropey"))]
use tokio::{
    fs::File as TokioFile,
    io::{AsyncRead, BufReader as TokioBufReader},
};
#[cfg(feature = "async")]
use tokio::{
    io::{AsyncWrite, BufWriter as TokioBufWriter},
    sync::oneshot::Sender as OneshotSender,
    task::JoinHandle,
    task::{JoinSet, LocalSet},
    time::{Interval, Sleep, Timeout},
};
#[cfg(feature = "async")]
use tokio_util::{
    codec::{Framed, LengthDelimitedCodec, LinesCodec},
    io::StreamReader,
};
#[cfg(feature = "tracing")]
use tracing::Level;
#[cfg(any(feature = "ropey", feature = "tui"))]
use unicode_segmentation::{GraphemeIndices, Graphemes, UnicodeSegmentation};
#[cfg(feature = "serde")]
use valuable::Value;

#[allow(async_fn_in_trait)]
pub trait Utils {
    const DEFAULT_ROPE_BUILDER_BUFFER_SIZE: usize = 8192;
    const IS_EXTENDED: bool = true;
    const NEWLINE: &str = "\n";
    const READ_FROM_CLIPBOARD_COMMAND: &str = "pbpaste";
    const WRITE_TO_CLIPBOARD_COMMAND: &str = "pbcopy";

    #[cfg(feature = "async")]
    async fn abort_all_and_wait<T: 'static>(&mut self)
    where
        Self: BorrowMut<JoinSet<T>>,
    {
        let join_set = self.borrow_mut();

        join_set.abort_all();

        while join_set.join_next().await.is_some() {}
    }

    #[cfg(feature = "fs")]
    fn absolute_utf8(&self) -> Result<Cow<'_, Utf8Path>, IoError>
    where
        Self: AsRef<Utf8Path>,
    {
        let path = self.as_ref();

        if path.is_absolute() {
            path.to_cow_borrowed().ok()
        } else {
            camino::absolute_utf8(path)?.into_cow_owned::<Utf8Path>().ok()
        }
    }

    async fn achain<T: Future>(self, rhs: T) -> T::Output
    where
        Self: Future + Sized,
    {
        self.await;

        rhs.await
    }

    #[cfg(feature = "tui")]
    fn add_span<'a, T: Into<Span<'a>>>(self, span: T) -> Line<'a>
    where
        Self: Into<Line<'a>>,
    {
        let mut line = self.into();

        line.spans.push(span.into());

        line
    }

    #[cfg(any(
        feature = "async",
        feature = "fs",
        feature = "process",
        feature = "reqwest",
        feature = "socket",
        feature = "tui",
    ))]
    fn anyhow_msg_error(self) -> AnyhowError
    where
        Self: 'static + Debug + Display + Send + Sized + Sync,
    {
        AnyhowError::msg(self)
    }

    #[cfg(any(
        feature = "async",
        feature = "fs",
        feature = "process",
        feature = "reqwest",
        feature = "socket",
        feature = "tui",
    ))]
    fn anyhow_error(self) -> AnyhowError
    where
        Self: Into<AnyhowError>,
    {
        self.into()
    }

    #[cfg(any(
        feature = "async",
        feature = "fs",
        feature = "process",
        feature = "reqwest",
        feature = "socket",
        feature = "tui",
    ))]
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

    async fn async_with<T>(self, next: impl Future<Output = T>) -> T
    where
        Self: Future + Sized,
    {
        self.await;

        next.await
    }

    fn as_borrowed<'a, B: ?Sized + ToOwned>(&'a self) -> &'a B
    where
        Self: Borrow<Cow<'a, B>>,
    {
        self.borrow().borrow()
    }

    fn as_immut(&mut self) -> &Self {
        &*self
    }

    fn as_ptr(&self) -> *const Self {
        std::ptr::from_ref(self)
    }

    fn as_ptr_mut(&mut self) -> *mut Self {
        std::ptr::from_mut(self)
    }

    fn as_utf8(&self) -> Result<&str, Utf8Error>
    where
        Self: AsRef<[u8]>,
    {
        std::str::from_utf8(self.as_ref())
    }

    #[cfg(feature = "fs")]
    fn as_utf8_path(&self) -> &Utf8Path
    where
        Self: AsRef<Utf8Path>,
    {
        self.as_ref()
    }

    #[cfg(feature = "ropey")]
    fn as_rope_slice(&self) -> RopeSlice<'_>
    where
        Self: Borrow<Rope>,
    {
        self.borrow().slice(..)
    }

    #[cfg(feature = "serde")]
    fn as_valuable(&self) -> Value<'_>
    where
        Self: AsValuable,
    {
        AsValuable::as_valuable(self)
    }

    #[cfg(feature = "tui")]
    fn bordered_block<'a>(self) -> Block<'a>
    where
        Self: Into<Line<'a>> + Sized,
    {
        Block::bordered().title(self)
    }

    fn buf_reader(self) -> BufReader<Self>
    where
        Self: Read + Sized,
    {
        BufReader::new(self)
    }

    #[cfg(any(feature = "async", feature = "ropey"))]
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

    #[cfg(feature = "async")]
    fn buf_writer_async(self) -> TokioBufWriter<Self>
    where
        Self: AsyncWrite + Sized,
    {
        TokioBufWriter::new(self)
    }

    #[cfg(any(feature = "ropey", feature = "misc", feature = "tui"))]
    fn cast_or_max<T: Bounded + NumCast>(self) -> T
    where
        Self: Sized + ToPrimitive,
    {
        match T::from(self) {
            Some(value) => value,
            None => T::max_value(),
        }
    }

    // NOTE: requires that [Self] and [T] have the same layout (provided by [#[repr(transparent)]]):
    // [https://stackoverflow.com/questions/79593399/implement-valuablevaluable-on-serde-jsonvalue]
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

    #[cfg(any(
        feature = "async",
        feature = "fs",
        feature = "process",
        feature = "reqwest",
        feature = "socket",
        feature = "tui",
    ))]
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

    #[cfg(any(
        feature = "async",
        feature = "fs",
        feature = "process",
        feature = "reqwest",
        feature = "socket",
        feature = "tui",
    ))]
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

    #[cfg(any(feature = "ropey", feature = "misc", feature = "tui"))]
    #[must_use]
    fn clamped(self, min: Self, max: Self) -> Self
    where
        Self: PartialOrd + Sized,
    {
        num::clamp(self, min, max)
    }

    fn convert<T: From<Self>>(self) -> T
    where
        Self: Sized,
    {
        self.into()
    }

    fn contains_eq<Q, K>(&self, query: Q) -> bool
    where
        Self: AsRef<[K]>,
        for<'a> &'a K: PartialEq<Q>,
    {
        self.find_eq(query).is_some()
    }

    #[cfg(any(
        feature = "async",
        feature = "fs",
        feature = "process",
        feature = "reqwest",
        feature = "socket",
        feature = "tui",
    ))]
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

    #[must_use]
    fn copied(&self) -> Self
    where
        Self: Copy + Sized,
    {
        *self
    }

    #[cfg(feature = "process")]
    async fn write_to_clipboard(&self) -> Result<ExitStatus, AnyhowError>
    where
        Self: AsRef<[u8]>,
    {
        let mut process = ProcessBuilder::new(Self::WRITE_TO_CLIPBOARD_COMMAND).build()?;

        process
            .stdin_mut()
            .write_all_then_async(self.as_ref())
            .await?
            .flush()
            .await?;

        process.run().await?.ok()
    }

    #[cfg(feature = "process")]
    #[must_use]
    async fn read_from_clipboard() -> Result<String, AnyhowError> {
        let mut process = ProcessBuilder::new(Self::READ_FROM_CLIPBOARD_COMMAND).build()?;
        let string = process.stdout_mut().read_to_string_async().await?;

        process.run().await?;

        string.ok()
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

    #[cfg(feature = "misc")]
    fn cycle_in_place(&mut self, amount: isize, total: usize)
    where
        Self: BorrowMut<usize>,
    {
        let current = self.borrow_mut();

        // NOTE: [isize::rem_euclid()] returns a nonnegative integer
        // [https://doc.rust-lang.org/stable/std/primitive.isize.html#method.rem_euclid] so casting as a [usize] is
        // fine
        *current = amount
            .saturating_add_unsigned(*current)
            .rem_euclid(total.cast_or_max())
            .cast_or_max();
    }

    #[cfg(feature = "fmt")]
    fn debug(&self) -> Debugged<'_, Self> {
        Debugged::new(self)
    }

    #[cfg(any(feature = "serde", feature = "tui"))]
    fn deserialize_from_seq<'de, D: Deserializer<'de>, X: Deserialize<'de>, Y, E: Display, F: Fn(X) -> Result<Y, E>>(
        deserializer: D,
        func: F,
    ) -> Result<Self, D::Error>
    where
        Self: Default + Extend<Y>,
    {
        let seq_visitor = SeqVisitor::new(func);

        deserializer.deserialize_seq(seq_visitor)
    }

    #[cfg(feature = "output")]
    fn end_err<T>(self) -> Output<T, Self>
    where
        Self: Sized,
    {
        Output::EndErr(self)
    }

    #[cfg(feature = "output")]
    fn end_ok<T, E>(&self) -> Output<T, E> {
        Output::EndOk
    }

    fn err<T>(self) -> Result<T, Self>
    where
        Self: Sized,
    {
        Err(self)
    }

    #[cfg(feature = "fs")]
    fn expand_user(&self) -> Cow<'_, Utf8Path>
    where
        Self: AsRef<str>,
    {
        let path_str = self.as_ref();

        if let Some(relative_path_str) = path_str.strip_prefix("~/")
            && let Some(home_dirpath) = Self::home_dirpath()
        {
            home_dirpath.join(relative_path_str).into_cow_owned()
        } else {
            path_str.as_utf8_path().to_cow_borrowed()
        }
    }

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
                "~".convert::<Utf8PathBuf>().into_cow_owned()
            } else if relative_path_str.as_utf8_path().is_absolute() {
                "~".cat(relative_path_str).convert::<Utf8PathBuf>().into_cow_owned()
            } else {
                path_str.as_utf8_path().to_cow_borrowed()
            }
        } else {
            path_str.as_utf8_path().to_cow_borrowed()
        }
    }

    #[cfg(any(feature = "ropey", feature = "tui"))]
    fn extended_graphemes(&self) -> Graphemes<'_>
    where
        Self: AsRef<str>,
    {
        self.as_ref().graphemes(Self::IS_EXTENDED)
    }

    #[cfg(any(feature = "ropey", feature = "tui"))]
    fn extended_grapheme_indices(&self) -> GraphemeIndices<'_>
    where
        Self: AsRef<str>,
    {
        self.as_ref().grapheme_indices(Self::IS_EXTENDED)
    }

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

    #[cfg(any(feature = "ropey", feature = "tui"))]
    fn extended_grapheme_substring(&self, range: Range<usize>) -> &str
    where
        Self: AsRef<str>,
    {
        let string = self.as_ref();
        let mut extended_grapheme_indices = string.extended_grapheme_indices().skip(range.start);
        let Some((begin_byte_offset, _begin_extended_grapheme)) = extended_grapheme_indices.next() else {
            return "";
        };
        let mut extended_grapheme_indices = extended_grapheme_indices.skip(range.len());
        let end_byte_offset = if let Some((end_byte_offset, _end_extended_grapheme)) = extended_grapheme_indices.next()
        {
            end_byte_offset
        } else {
            string.len()
        };

        &string[begin_byte_offset..end_byte_offset]
    }

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

    #[cfg(feature = "fs")]
    fn file_name_ok(&self) -> Result<&str, AnyhowError>
    where
        Self: AsRef<Utf8Path>,
    {
        self.as_ref().file_name().context("path has no file name")
    }

    fn find_eq<Q, K>(&self, query: Q) -> Option<(usize, &K)>
    where
        Self: AsRef<[K]>,
        for<'a> &'a K: PartialEq<Q>,
    {
        self.as_ref().iter().enumerate().find(|(_index, key)| *key == query)
    }

    fn has_happened(self) -> bool
    where
        Self: Is<Instant>,
    {
        self.into_self() <= Instant::now()
    }

    #[cfg(feature = "fs")]
    #[must_use]
    fn home_dirpath() -> Option<Utf8PathBuf> {
        home::home_dir()?.try_convert::<Utf8PathBuf>().ok()
    }

    fn if_else<T>(self, true_value: T, false_value: T) -> T
    where
        Self: Is<bool>,
    {
        if self.into_self() { true_value } else { false_value }
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

    #[cfg(any(feature = "ropey", feature = "misc", feature = "tui"))]
    #[must_use]
    fn interpolate(
        self,
        old_min: impl ToPrimitive,
        old_max: impl ToPrimitive,
        new_min: impl ToPrimitive,
        new_max: impl ToPrimitive,
    ) -> Self
    where
        Self: Bounded + NumCast + ToPrimitive,
    {
        let old_min = old_min.cast_or_max::<f64>();
        let old_max = old_max.cast_or_max::<f64>();
        let new_min = new_min.cast_or_max::<f64>();
        let new_max = new_max.cast_or_max::<f64>();
        let old_value = self.cast_or_max::<f64>().clamped(old_min, old_max);
        let new_value = new_min + (new_max - new_min) * (old_value - old_min) / (old_max - old_min);
        let new_value = new_value.clamped(new_min, new_max);

        new_value.cast_or_max()
    }

    fn into_box(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    fn into_break<C>(self) -> ControlFlow<Self, C>
    where
        Self: Sized,
    {
        ControlFlow::Break(self)
    }

    #[cfg(feature = "tui")]
    fn into_color<T>(self) -> T
    where
        Self: IntoColor<T>,
    {
        IntoColor::into_color(self)
    }

    fn into_continue<B>(self) -> ControlFlow<B, Self>
    where
        Self: Sized,
    {
        ControlFlow::Continue(self)
    }

    fn into_cow_owned<B: ?Sized + ToOwned<Owned = Self>>(self) -> Cow<'static, B>
    where
        Self: Sized,
    {
        Cow::Owned(self)
    }

    #[cfg(feature = "poem")]
    fn into_endpoint(self) -> impl Endpoint<Output = Self>
    where
        Self: Clone + IntoResponse + Sync,
    {
        let func = move |_request| self.clone();

        poem::endpoint::make_sync(func)
    }

    #[cfg(feature = "async")]
    fn into_interval(self) -> Interval
    where
        Self: Is<Duration>,
    {
        tokio::time::interval(self.into_self())
    }

    #[cfg(feature = "async")]
    fn into_left<R>(self) -> Either<Self, R>
    where
        Self: Sized,
    {
        Either::Left(self)
    }

    #[cfg(feature = "tui")]
    fn into_line<'a>(self) -> Line<'a>
    where
        Self: Into<Cow<'a, str>>,
    {
        self.into().into()
    }

    fn into_manually_drop(self) -> ManuallyDrop<Self>
    where
        Self: Sized,
    {
        ManuallyDrop::new(self)
    }

    #[cfg(feature = "async")]
    fn into_stream_reader<B: Buf, E: Into<IoError>>(self) -> StreamReader<Self, B>
    where
        Self: Sized + Stream<Item = Result<B, E>>,
    {
        StreamReader::new(self)
    }

    #[cfg(feature = "async")]
    fn into_length_delimited_frames(self) -> Framed<Self, LengthDelimitedCodec>
    where
        Self: Sized,
    {
        Framed::new(self, LengthDelimitedCodec::new())
    }

    #[cfg(feature = "async")]
    fn into_line_frames(self) -> Framed<Self, LinesCodec>
    where
        Self: Sized,
    {
        Framed::new(self, LinesCodec::new())
    }

    #[cfg(feature = "async")]
    fn into_right<L>(self) -> Either<L, Self>
    where
        Self: Sized,
    {
        Either::Right(self)
    }

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

    #[cfg(feature = "tracing")]
    fn into_status<T, E>(self) -> Status<T, E>
    where
        Self: Is<Result<T, E>>,
    {
        Status::new(self.into_self())
    }

    #[cfg(feature = "async")]
    fn into_stream(self) -> Self::Stream
    where
        Self: IntoStream + Sized,
    {
        IntoStream::into_stream(self)
    }

    #[cfg(feature = "fs")]
    fn into_string(self) -> Result<String, AnyhowError>
    where
        Self: Is<PathBuf>,
    {
        match self.into_self().into_os_string().into_string() {
            Ok(string) => string.ok(),
            Err(os_string) => os_string.invalid_utf8_err(),
        }
    }

    #[cfg(feature = "serde")]
    fn into_value_from_json<T: DeserializeOwned>(self) -> Result<T, SerdeJsonError>
    where
        Self: Is<Json>,
    {
        serde_json::from_value(self.into_self())
    }

    #[cfg(feature = "fs")]
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

    #[allow(clippy::wrong_self_convention)]
    fn is_less_than<T: PartialOrd>(self, rhs: T) -> bool
    where
        Self: Into<T>,
    {
        self.into() < rhs
    }

    #[cfg(any(feature = "ropey", feature = "misc", feature = "tui"))]
    fn is_positive(&self) -> bool
    where
        Self: PartialOrd + Zero,
    {
        &Self::zero() < self
    }

    #[cfg(feature = "async")]
    async fn join_all<T>(self) -> T
    where
        Self: IntoIterator<Item: Future> + Sized,
        T: FromIterator<<Self::Item as Future>::Output>,
    {
        futures::future::join_all(self).await.into_iter().collect()
    }

    #[cfg(any(feature = "ropey", feature = "tui"))]
    fn len_extended_graphemes(&self) -> usize
    where
        Self: AsRef<str>,
    {
        self.as_ref().extended_graphemes().count()
    }

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

    #[cfg(any(feature = "tui", feature = "tracing"))]
    fn log_error(&self)
    where
        Self: Display,
    {
        tracing::warn!(error = %self, "error: {self:#}");
    }

    #[cfg(any(feature = "tui", feature = "tracing"))]
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

    fn mem_drop(self)
    where
        Self: Sized,
    {
        std::mem::drop(self);
    }

    #[must_use]
    fn mem_replace(&mut self, value: Self) -> Self
    where
        Self: Sized,
    {
        std::mem::replace(self, value)
    }

    #[must_use]
    fn mem_take(&mut self) -> Self
    where
        Self: Default,
    {
        std::mem::take(self)
    }

    #[cfg(feature = "async")]
    async fn metadata_async(&self) -> Result<Metadata, IoError>
    where
        Self: AsRef<Path>,
    {
        tokio::fs::metadata(self).await
    }

    // NOTE: [https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.push_mut]
    fn mut_push<T>(&mut self, item: T) -> &mut T
    where
        Self: BorrowMut<Vec<T>>,
    {
        let vec = self.borrow_mut();

        vec.push(item);

        vec.last_mut().unwrap()
    }

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

    #[cfg(any(feature = "ropey", feature = "async"))]
    async fn open_async(&self) -> Result<TokioFile, IoError>
    where
        Self: AsRef<Path>,
    {
        TokioFile::open(self).await
    }

    #[cfg(feature = "fmt")]
    fn optional_display(&self) -> OptionalDisplay<'_, Self> {
        OptionalDisplay::new(self)
    }

    #[cfg(feature = "output")]
    fn output_ok<E>(self) -> Output<Self, E>
    where
        Self: Sized,
    {
        Output::Ok(self)
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

    #[cfg(feature = "poem")]
    fn poem_binary(self) -> PoemBinary<Self>
    where
        Self: Sized,
    {
        PoemBinary(self)
    }

    #[cfg(feature = "poem")]
    fn poem_binary_message(self) -> PoemMessage
    where
        Self: Is<Vec<u8>>,
    {
        PoemMessage::Binary(self.into_self())
    }

    #[cfg(feature = "poem")]
    fn poem_json(self) -> PoemJson<Self>
    where
        Self: Sized,
    {
        PoemJson(self)
    }

    #[cfg(feature = "poem")]
    fn poem_stream_body<O: 'static + Into<Bytes>, E: 'static + Into<IoError>>(self) -> PoemBinary<PoemBody>
    where
        Self: 'static + Send + Sized + Stream<Item = Result<O, E>>,
    {
        PoemBody::from_bytes_stream(self).poem_binary()
    }

    #[cfg(feature = "poem")]
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

    fn push_to<T: Extend<Self>>(self, collection: &mut T)
    where
        Self: Sized,
    {
        collection.extend(self.once());
    }

    fn push_all_to<T: Extend<Self::Item>>(self, collection: &mut T)
    where
        Self: IntoIterator + Sized,
    {
        collection.extend(self);
    }

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

    fn range_from_len(self, len: Self) -> Range<Self>
    where
        Self: Add<Output = Self> + Copy,
    {
        let end = self + len;

        self..end
    }

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

    #[cfg(any(feature = "async", feature = "process"))]
    async fn read_to_string_async(&mut self) -> Result<String, IoError>
    where
        Self: AsyncReadExt + Unpin,
    {
        let mut string = String::new();

        self.read_to_string(&mut string).await?;

        string.ok()
    }

    #[cfg(feature = "async")]
    async fn read_to_string_fs_async(self) -> ReadValue<Self>
    where
        Self: AsRef<Path> + Sized,
    {
        let result = tokio::fs::read_to_string(self.as_ref()).await;

        ReadValue::new(self, result)
    }

    fn ready(self) -> Ready<Self>
    where
        Self: Sized,
    {
        std::future::ready(self)
    }

    fn ref_immut(&self) -> &Self {
        self
    }

    fn ref_mut(&mut self) -> &mut Self {
        self
    }

    #[cfg(feature = "tui")]
    fn render_to(self, frame: &mut Frame, rect: Rect)
    where
        Self: Widget + Sized,
    {
        frame.render_widget(self, rect);
    }

    fn repeat(self) -> Repeat<Self>
    where
        Self: Clone,
    {
        std::iter::repeat(self)
    }

    fn reversed<X, Y>(self) -> (Y, X)
    where
        Self: Is<(X, Y)>,
    {
        let (x, y) = self.into_self();

        y.pair(x)
    }

    #[cfg(feature = "ropey")]
    async fn rope<const N: usize>(&self) -> Result<Rope, IoError>
    where
        Self: AsRef<Path>,
    {
        self.open_async()
            .await?
            .buf_reader_async()
            .pipe_into(RopeBuilder::<_, N>::new)
            .build()
            .await
    }

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

    #[cfg(feature = "async")]
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

    #[cfg(feature = "socket")]
    async fn respond_to<T: Request<Response = Self>>(
        &self,
        mut socket: impl BorrowMut<Socket>,
    ) -> Result<(), AnyhowError> {
        socket.borrow_mut().respond::<T>(self).await
    }

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

    #[cfg(feature = "tui")]
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

    #[cfg(feature = "tui")]
    fn scroll_count(&self, scroll_count: ScrollCount, orientation: Orientation) -> usize
    where
        Self: Scrollable,
    {
        match scroll_count {
            ScrollCount::Fixed(scroll_count) => scroll_count,
            ScrollCount::PageSize => *self.latest_content_render_size().get(orientation),
        }
    }

    #[cfg(feature = "tui")]
    fn max_scroll_offset(&self, scroll_when: ScrollWhen, content_size: PointUsize) -> PointUsize
    where
        Self: Scrollable,
    {
        match scroll_when {
            ScrollWhen::Always => content_size.saturating_sub_scalar(&1),
            ScrollWhen::ForLargeContent => content_size.saturating_sub(&self.latest_content_render_size()),
        }
    }

    #[cfg(feature = "tui")]
    fn scroll(
        &mut self,
        scroll_count: ScrollCount,
        scroll_when: ScrollWhen,
        content_size: PointUsize,
        orientation: Orientation,
        add: bool,
    ) where
        Self: Scrollable,
    {
        let scroll_count = self.as_immut().scroll_count(scroll_count, orientation);
        let max_scroll_offset = *self
            .as_immut()
            .max_scroll_offset(scroll_when, content_size)
            .get(orientation);

        self.scroll_offset_mut()
            .get_mut(orientation)
            .saturating_add_or_sub_in_place_with_max(scroll_count, max_scroll_offset, add);
    }

    #[cfg(feature = "tui")]
    fn scroll_down(&mut self, scroll_count: ScrollCount, content_size: PointUsize, scroll_when: ScrollWhen)
    where
        Self: Scrollable,
    {
        self.scroll(scroll_count, scroll_when, content_size, Orientation::Vertical, true);
    }

    #[cfg(feature = "tui")]
    fn scroll_up(&mut self, scroll_count: ScrollCount, content_size: PointUsize, scroll_when: ScrollWhen)
    where
        Self: Scrollable,
    {
        self.scroll(scroll_count, scroll_when, content_size, Orientation::Vertical, false);
    }

    #[cfg(feature = "tui")]
    fn scroll_left(&mut self, scroll_count: ScrollCount, content_size: PointUsize, scroll_when: ScrollWhen)
    where
        Self: Scrollable,
    {
        self.scroll(scroll_count, scroll_when, content_size, Orientation::Horizontal, false);
    }

    #[cfg(feature = "tui")]
    fn scroll_right(&mut self, scroll_count: ScrollCount, content_size: PointUsize, scroll_when: ScrollWhen)
    where
        Self: Scrollable,
    {
        self.scroll(scroll_count, scroll_when, content_size, Orientation::Horizontal, true);
    }

    #[cfg(feature = "async")]
    async fn select_all(self) -> <<Self as IntoIterator>::Item as Future>::Output
    where
        Self: IntoIterator + Sized,
        <Self as IntoIterator>::Item: Future,
    {
        self.into_iter()
            .collect::<FuturesUnordered<_>>()
            .next()
            .wait_then_unwrap_or_pending()
            .await
    }

    #[cfg(feature = "async")]
    async fn send_to<T: Sink<Self> + Unpin>(self, mut sink: T) -> Result<(), T::Error>
    where
        Self: Sized,
    {
        sink.send(self).await
    }

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

    fn set_true(&mut self) -> bool
    where
        Self: BorrowMut<bool>,
    {
        self.borrow_mut().mem_replace(true)
    }

    fn set_false(&mut self) -> bool
    where
        Self: BorrowMut<bool>,
    {
        self.borrow_mut().mem_replace(false)
    }

    #[cfg(feature = "async")]
    fn sleep(self) -> Sleep
    where
        Self: Is<Duration>,
    {
        tokio::time::sleep(self.into_self())
    }

    fn some(self) -> Option<Self>
    where
        Self: Sized,
    {
        Some(self)
    }

    #[cfg(feature = "async")]
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
        let byte_str = self.as_ref();
        let predicate = |substr| substr == query;
        let query_len = query.len();
        let begin = byte_str.windows(query_len).position(predicate)?;
        let end = begin + query_len;

        (begin, end).some()
    }

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

    #[cfg(feature = "async")]
    fn timeout(self, duration: Duration) -> Timeout<Self>
    where
        Self: Future + Sized,
    {
        tokio::time::timeout(duration, self)
    }

    fn to_cow_borrowed(&self) -> Cow<'_, Self>
    where
        Self: ToOwned,
    {
        Cow::Borrowed(self)
    }

    fn toggle(&mut self)
    where
        Self: BorrowMut<bool>,
    {
        let bool_value = self.borrow_mut();

        *bool_value = !*bool_value;
    }

    #[cfg(feature = "serde")]
    fn to_json(&self) -> Result<Json, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_value(self)
    }

    #[cfg(feature = "serde")]
    fn to_json_byte_str(&self) -> Result<Vec<u8>, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_vec(self)
    }

    #[cfg(feature = "serde")]
    fn to_json_object(&self, key: &str) -> Json
    where
        Self: Serialize,
    {
        serde_json::json!({key: self})
    }

    #[cfg(feature = "serde")]
    fn to_json_str(&self) -> Result<String, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_string(self)
    }

    #[cfg(feature = "rmp")]
    fn to_rmp_byte_str(&self) -> Result<Vec<u8>, RmpEncodeError>
    where
        Self: Serialize,
    {
        rmp_serde::to_vec(self)
    }

    #[cfg(feature = "fs")]
    fn to_uri(&self) -> Result<String, IoError>
    where
        Self: AsRef<Utf8Path>,
    {
        "file://".cat(self.absolute_utf8()?).ok()
    }

    #[cfg(feature = "serde")]
    fn to_value_from_json_slice<'a, T: Deserialize<'a>>(&'a self) -> Result<T, SerdeJsonError>
    where
        Self: AsRef<[u8]>,
    {
        serde_json::from_slice(self.as_ref())
    }

    #[cfg(feature = "serde")]
    fn to_value_from_json_reader<T: DeserializeOwned>(self) -> Result<T, SerdeJsonError>
    where
        Self: Read + Sized,
    {
        serde_json::from_reader(self)
    }

    #[cfg(feature = "rmp")]
    fn to_value_from_rmp_slice<'a, T: Deserialize<'a>>(&'a self) -> Result<T, RmpDecodeError>
    where
        Self: AsRef<[u8]>,
    {
        rmp_serde::from_slice(self.as_ref())
    }

    #[cfg(feature = "serde")]
    fn to_value_from_value<T: DeserializeOwned>(&self) -> Result<T, SerdeJsonError>
    where
        Self: Serialize,
    {
        self.to_json()?.into_value_from_json()
    }

    #[cfg(feature = "serde")]
    fn to_value_from_yaml_slice<'a, T: Deserialize<'a>>(&'a self) -> Result<T, SerdeYamlError>
    where
        Self: AsRef<[u8]>,
    {
        serde_yaml_ng::from_slice(self.as_ref())
    }

    #[cfg(feature = "serde")]
    fn to_value_from_yaml_reader<T: DeserializeOwned>(self) -> Result<T, SerdeYamlError>
    where
        Self: Read + Sized,
    {
        serde_yaml_ng::from_reader(self)
    }

    #[cfg(any(feature = "ropey", feature = "tui"))]
    #[must_use]
    fn transpose(&self) -> Self
    where
        Self: Transpose + Sized,
    {
        Transpose::to_transpose(self)
    }

    fn try_convert<T: TryFrom<Self>>(self) -> Result<T, T::Error>
    where
        Self: Sized,
    {
        self.try_into()
    }

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

    #[cfg(feature = "async")]
    async fn try_wait<T, E: 'static + Send + Sync>(self) -> Result<T, AnyhowError>
    where
        Self: Is<JoinHandle<Result<T, E>>>,
        AnyhowError: From<E>,
    {
        self.into_self().await??.ok()
    }

    #[must_use]
    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }

    fn unit(&self) {}

    // Future<Option<T>>(). wait_for_some
    async fn wait_then_unwrap_or_pending<T>(self) -> T
    where
        Self: Future<Output = Option<T>> + Sized,
    {
        match self.await {
            Some(value) => value,
            None => std::future::pending().await,
        }
    }

    // wait_if_some
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

    #[cfg(feature = "serde")]
    fn write_as_json_to<T: Write>(&self, writer: T) -> Result<(), SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_writer(writer, self)
    }

    fn write_all_then(&mut self, byte_str: &[u8]) -> Result<&mut Self, IoError>
    where
        Self: Write,
    {
        self.write_all(byte_str)?.with(self).ok()
    }

    #[cfg(any(feature = "async", feature = "process"))]
    async fn write_all_then_async(&mut self, byte_str: &[u8]) -> Result<&mut Self, IoError>
    where
        Self: AsyncWriteExt + Unpin,
    {
        self.write_all(byte_str).await?.with(self).ok()
    }
}

impl<T: ?Sized> Utils for T {}
