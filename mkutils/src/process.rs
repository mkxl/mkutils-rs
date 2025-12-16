use crate::utils::Utils;
use anyhow::{Context, Error as AnyhowError};
use std::{
    borrow::Borrow,
    ffi::OsStr,
    path::Path,
    process::{ExitStatus, Stdio},
};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

/// A wrapper around a tokio child process with convenient access to stdio streams.
///
/// `Process` spawns a child process with piped stdin, stdout, and stderr, providing
/// direct access to these streams for communication with the process.
///
/// # Examples
///
/// ```rust,no_run
/// use mkutils::{Process, Utils};
/// use tokio::io::AsyncWriteExt;
///
/// #[tokio::main]
/// async fn main() -> Result<(), anyhow::Error> {
///     let mut process = Process::new(
///         "cat",
///         vec![],
///         vec![],
///         None::<&str>,
///     )?;
///
///     // Write to stdin
///     process.stdin_mut().write_all(b"hello\n").await?;
///
///     // Read from stdout and wait for completion
///     let status = process.run().await?;
///     println!("Exit status: {}", status);
///     Ok(())
/// }
/// ```
pub struct Process {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

impl Process {
    /// Creates a new `Process` by spawning a command.
    ///
    /// # Parameters
    ///
    /// - `cmd`: The command to execute
    /// - `args`: Iterator of command arguments
    /// - `env`: Iterator of environment variables as `(key, value)` pairs
    /// - `current_dirpath`: Optional working directory for the process
    ///
    /// # Errors
    ///
    /// Returns an error if the process fails to spawn or stdio streams cannot be captured.
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

    /// Returns a mutable reference to the process's stdin stream.
    pub const fn stdin_mut(&mut self) -> &mut ChildStdin {
        &mut self.stdin
    }

    /// Returns a mutable reference to the process's stdout stream.
    pub const fn stdout_mut(&mut self) -> &mut ChildStdout {
        &mut self.stdout
    }

    /// Returns a mutable reference to the process's stderr stream.
    pub const fn stderr_mut(&mut self) -> &mut ChildStderr {
        &mut self.stderr
    }

    /// Consumes the `Process` and returns its constituent parts.
    ///
    /// Returns a tuple of `(Child, ChildStdin, ChildStdout, ChildStderr)`,
    /// allowing for manual management of the process and its streams.
    #[must_use]
    pub fn into_parts(self) -> (Child, ChildStdin, ChildStdout, ChildStderr) {
        (self.child, self.stdin, self.stdout, self.stderr)
    }

    /// Waits for the process to complete and returns its exit status.
    ///
    /// # Errors
    ///
    /// Returns an error if waiting for the process fails.
    pub async fn run(&mut self) -> Result<ExitStatus, AnyhowError> {
        self.child.wait().await?.ok()
    }
}
