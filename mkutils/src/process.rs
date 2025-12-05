use crate::utils::Utils;
use anyhow::{Context, Error as AnyhowError};
use std::{
    borrow::Borrow,
    ffi::OsStr,
    path::Path,
    process::{ExitStatus, Stdio},
};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

pub struct Process {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

impl Process {
    pub fn new<
        Cmd: AsRef<OsStr>,
        Args: IntoIterator,
        Env: IntoIterator,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
        Cwd: AsRef<Path>,
    >(
        cmd: Cmd,
        args: Args,
        env: Env,
        current_dirpath: impl Into<Option<Cwd>>,
    ) -> Result<Self, AnyhowError>
    where
        Args::Item: AsRef<OsStr>,
        Env::Item: Borrow<(K, V)>,
    {
        let mut command = Command::new(cmd);

        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        for arg in args {
            command.arg(arg);
        }

        for env_entry in env {
            let (env_var_name, env_var_val) = env_entry.borrow();

            command.env(env_var_name, env_var_val);
        }

        if let Some(current_dirpath) = current_dirpath.into() {
            command.current_dir(current_dirpath);
        }

        let mut child = command.spawn()?;
        let stdin = Self::take_stdio(&mut child.stdin)?;
        let stdout = Self::take_stdio(&mut child.stdout)?;
        let stderr = Self::take_stdio(&mut child.stderr)?;
        let process = Self {
            child,
            stdin,
            stdout,
            stderr,
        };

        process.ok()
    }

    fn take_stdio<T>(stdio: &mut Option<T>) -> Result<T, AnyhowError> {
        stdio.take().context("unable to set up stdio for process")
    }

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

    pub async fn run(&mut self) -> Result<ExitStatus, AnyhowError> {
        self.child.wait().await?.ok()
    }
}
