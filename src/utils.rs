use crate::{debugged::Debugged, is::Is};
use anyhow::{Context, Error};
use poem_openapi::payload::Json as PoemJson;
use reqwest::Response;
use serde::Serialize;
use serde_json::{Error as SerdeJsonError, Value as Json};
use std::{
    borrow::Borrow,
    ffi::OsStr,
    fmt::{Debug, Display},
    fs::File,
    future::Future,
    io::Error as IoError,
    marker::Unpin,
    path::{Path, PathBuf},
    str::Utf8Error,
};
use tokio::{io::AsyncReadExt, sync::oneshot::Sender as OneshotSender, task::JoinHandle};

pub trait Utils {
    fn absolute(&self) -> Result<PathBuf, IoError>
    where
        Self: AsRef<Path>,
    {
        std::path::absolute(self)?.ok()
    }

    #[allow(async_fn_in_trait)]
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
        str::from_utf8(self.as_ref())
    }

    fn cat<T: Display>(&self, rhs: T) -> String
    where
        Self: Display,
    {
        std::format!("{self}{rhs}")
    }

    // NOTE: [https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.error_for_status]
    #[allow(async_fn_in_trait)]
    async fn check_status(self) -> Result<Response, Error>
    where
        Self: Into<Response>,
    {
        let response = self.into();
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

    fn create(&self) -> Result<File, IoError>
    where
        Self: AsRef<Path>,
    {
        File::create(self)
    }

    fn debug(&self) -> Debugged<Self> {
        Debugged::new(self)
    }

    fn file_name_ok(&self) -> Result<&OsStr, Error>
    where
        Self: AsRef<Path>,
    {
        self.as_ref().file_name().context("path has no file_name")
    }

    fn into_string(self) -> Result<String, Error>
    where
        Self: Is<PathBuf> + Sized,
    {
        match self.get().into_os_string().into_string() {
            Ok(string) => string.ok(),
            Err(os_string) => os_string.invalid_utf8_err(),
        }
    }

    fn invalid_utf8_err<T>(&self) -> Result<T, Error>
    where
        Self: Debug,
    {
        anyhow::bail!("{self:?} is not valid utf-8")
    }

    fn json_byte_str(&self) -> Result<Vec<u8>, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_vec(self)
    }

    fn json(&self) -> Result<Json, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_value(self)
    }

    #[must_use]
    fn log_error<T, C: Display, E: Debug + Display>(self, context: C) -> Self
    where
        Self: Borrow<Result<T, E>> + Sized,
    {
        if let Err(err) = self.borrow() {
            tracing::warn!(?err, "{context}: {err}");
        }

        self
    }

    fn map_into<Y, X: Into<Y>>(self) -> Option<Y>
    where
        Self: Is<Option<X>> + Sized,
    {
        self.get().map(X::into)
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

    fn ok<E>(self) -> Result<Self, E>
    where
        Self: Sized,
    {
        Ok(self)
    }

    fn pair<T>(self, rhs: T) -> (Self, T)
    where
        Self: Sized,
    {
        (self, rhs)
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

    #[allow(async_fn_in_trait)]
    async fn read_string(&mut self) -> Result<String, Error>
    where
        Self: AsyncReadExt + Unpin,
    {
        let mut string = String::new();

        self.read_to_string(&mut string).await?;

        string.ok()
    }

    fn send_to_oneshot(self, sender: OneshotSender<Self>) -> Result<(), Error>
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

    fn to_str_ok(&self) -> Result<&str, Error>
    where
        Self: AsRef<Path>,
    {
        let path = self.as_ref();

        match path.to_str() {
            Some(string) => string.ok(),
            None => path.invalid_utf8_err(),
        }
    }

    fn to_uri(&self) -> Result<String, Error>
    where
        Self: AsRef<Path>,
    {
        "file://".cat(self.absolute()?.to_str_ok()?).ok()
    }

    fn unit(&self) {}
}

impl<T: ?Sized> Utils for T {}
