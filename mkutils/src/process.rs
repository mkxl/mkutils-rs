use crate::utils::Utils;
use anyhow::{Context, Error as AnyhowError};
use derive_more::{Constructor, From};
use std::{
    borrow::Borrow,
    ffi::OsStr,
    io::Error as IoError,
    path::Path,
    process::{ExitStatus, Stdio},
};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

#[derive(From)]
pub struct ProcessBuilder {
    command: Command,
}

impl ProcessBuilder {
    const STDIO_ERROR_MESSAGE: &str = "unable to set up stdio for process";

    pub fn new(cmd: impl AsRef<OsStr>) -> Self {
        let mut command = Command::new(cmd);

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        command.into()
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.command.arg(arg);

        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut Self {
        self.command.args(args);

        self
    }

    pub fn env(&mut self, env: impl IntoIterator<Item: Borrow<(impl AsRef<OsStr>, impl AsRef<OsStr>)>>) -> &mut Self {
        for env_entry in env {
            let (env_var_name, env_var_val) = env_entry.borrow();

            self.command.env(env_var_name, env_var_val);
        }

        self
    }

    pub fn current_dirpath<P: AsRef<Path>>(&mut self, current_dirpath: impl Into<Option<P>>) -> &mut Self {
        if let Some(current_dirpath) = current_dirpath.into() {
            self.command.current_dir(current_dirpath);
        }

        self
    }

    fn take_stdio<T>(stdio: &mut Option<T>) -> Result<T, AnyhowError> {
        stdio.take().context(Self::STDIO_ERROR_MESSAGE)
    }

    pub fn build(&mut self) -> Result<Process, AnyhowError> {
        let mut child = self.command.spawn()?;
        let stdin = Self::take_stdio(&mut child.stdin)?;
        let stdout = Self::take_stdio(&mut child.stdout)?;
        let stderr = Self::take_stdio(&mut child.stderr)?;
        let process = Process::new(child, stdin, stdout, stderr);

        process.ok()
    }
}

#[derive(Constructor)]
pub struct Process {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

impl Process {
    pub const fn stdin_mut(&mut self) -> &mut ChildStdin {
        &mut self.stdin
    }

    pub const fn stdout_mut(&mut self) -> &mut ChildStdout {
        &mut self.stdout
    }

    pub const fn stderr_mut(&mut self) -> &mut ChildStderr {
        &mut self.stderr
    }

    #[must_use]
    pub fn into_parts(self) -> (Child, ChildStdin, ChildStdout, ChildStderr) {
        (self.child, self.stdin, self.stdout, self.stderr)
    }

    pub async fn run(mut self) -> Result<ExitStatus, IoError> {
        self.stdin.mem_drop();
        self.stdout.mem_drop();
        self.stderr.mem_drop();
        self.child.wait().await?.ok()
    }
}
