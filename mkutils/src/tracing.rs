use console_subscriber::{ConsoleLayer, Server as ConsoleServer};
use std::{
    io::{StderrLock, StdoutLock},
    net::IpAddr,
};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{
        FormatEvent, FormatFields, Layer, MakeWriter,
        format::{FmtSpan, Format},
    },
    layer::{Layer as _, SubscriberExt},
    registry::Registry,
    util::SubscriberInitExt,
};

type StdoutLockFn = fn() -> StdoutLock<'static>;
type StderrLockFn = fn() -> StderrLock<'static>;

/// A macro for emitting trace events with dynamic log levels.
///
/// This macro allows you to specify the log level at runtime rather than compile-time,
/// which is useful when the level is determined by configuration or other runtime factors.
///
/// # Syntax
///
/// ```ignore
/// trace!(level = level_expr, "message", field1 = value1, ...)
/// ```
///
/// # Examples
///
/// ```rust
/// use mkutils::trace;
/// use tracing::Level;
///
/// let level = Level::INFO;
/// trace!(level = level, "User logged in", user_id = 123);
///
/// // Equivalent to calling the appropriate macro based on level:
/// // tracing::info!("User logged in", user_id = 123);
/// ```
#[macro_export]
macro_rules! trace {
    (level = $level:expr, $($rest:tt)*) => {{
        match $level {
            ::tracing::Level::TRACE => tracing::trace!($($rest)*),
            ::tracing::Level::DEBUG => tracing::debug!($($rest)*),
            ::tracing::Level::INFO  => tracing::info!($($rest)*),
            ::tracing::Level::WARN  => tracing::warn!($($rest)*),
            ::tracing::Level::ERROR => tracing::error!($($rest)*),
        }
    }};
}

/// A builder for configuring and initializing the tracing subscriber.
///
/// `Tracing` provides a fluent interface for setting up structured logging and tracing
/// with support for:
/// - Configurable log levels
/// - JSON or human-readable output
/// - Custom output destinations (stdout, stderr, or custom writers)
/// - Optional tokio-console integration for async runtime debugging
///
/// # Type Parameters
///
/// - `W`: The writer type for log output (defaults to `StderrLockFn`)
///
/// # Examples
///
/// ```rust
/// use mkutils::Tracing;
/// use tracing_subscriber::filter::LevelFilter;
///
/// // Basic setup with defaults (INFO level to stderr)
/// Tracing::default().init();
///
/// // Custom configuration
/// Tracing::default()
///     .with_level_filter(LevelFilter::DEBUG)
///     .with_json_enabled(true)
///     .with_stdout_lock_writer()
///     .init();
/// ```
pub struct Tracing<W = StderrLockFn> {
    level_filter: LevelFilter,
    span_events: FmtSpan,
    json_enabled: bool,
    tokio_console_enabled: bool,
    tokio_console_ip_addr: IpAddr,
    tokio_console_port: u16,
    writer: W,
}

impl Tracing<StderrLockFn> {
    pub const DEFAULT_JSON_ENABLED: bool = false;
    pub const DEFAULT_LEVEL_FILTER: LevelFilter = LevelFilter::INFO;
    pub const DEFAULT_TOKIO_CONSOLE_ENABLED: bool = false;
    pub const DEFAULT_TOKIO_CONSOLE_IP_ADDR: IpAddr = ConsoleServer::DEFAULT_IP;
    pub const DEFAULT_TOKIO_CONSOLE_PORT: u16 = ConsoleServer::DEFAULT_PORT;
}

impl Default for Tracing<StderrLockFn> {
    fn default() -> Self {
        Self {
            level_filter: Self::DEFAULT_LEVEL_FILTER,
            json_enabled: Self::DEFAULT_JSON_ENABLED,
            span_events: Self::default_span_events(),
            tokio_console_enabled: Self::DEFAULT_TOKIO_CONSOLE_ENABLED,
            tokio_console_ip_addr: Self::DEFAULT_TOKIO_CONSOLE_IP_ADDR,
            tokio_console_port: Self::DEFAULT_TOKIO_CONSOLE_PORT,
            writer: Self::stderr_lock_writer,
        }
    }
}

impl<W> Tracing<W> {
    fn default_span_events() -> FmtSpan {
        FmtSpan::NEW | FmtSpan::CLOSE
    }

    fn stdout_lock_writer() -> StdoutLock<'static> {
        std::io::stdout().lock()
    }

    fn stderr_lock_writer() -> StderrLock<'static> {
        std::io::stderr().lock()
    }

    /// Sets the minimum log level filter.
    ///
    /// Only events at or above this level will be logged.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mkutils::Tracing;
    /// use tracing_subscriber::filter::LevelFilter;
    ///
    /// let tracing = Tracing::default()
    ///     .with_level_filter(LevelFilter::DEBUG);
    /// ```
    #[must_use]
    pub const fn with_level_filter(mut self, level_filter: LevelFilter) -> Self {
        self.level_filter = level_filter;

        self
    }

    /// Enables or disables JSON output format.
    ///
    /// When enabled, logs are formatted as JSON objects. When disabled (default),
    /// logs use a human-readable format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mkutils::Tracing;
    ///
    /// let tracing = Tracing::default()
    ///     .with_json_enabled(true);
    /// ```
    #[must_use]
    pub const fn with_json_enabled(mut self, json_enabled: bool) -> Self {
        self.json_enabled = json_enabled;

        self
    }

    /// Sets the port for the tokio-console server.
    ///
    /// Only used when tokio-console is enabled. Defaults to 6669.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mkutils::Tracing;
    ///
    /// let tracing = Tracing::default()
    ///     .with_tokio_console_enabled(true)
    ///     .with_tokio_console_port(6670);
    /// ```
    #[must_use]
    pub const fn with_tokio_console_port(mut self, tokio_console_port: u16) -> Self {
        self.tokio_console_port = tokio_console_port;

        self
    }

    /// Enables or disables tokio-console integration.
    ///
    /// When enabled, starts a tokio-console server that provides real-time
    /// monitoring and debugging of async tasks. You can connect to it with
    /// the `tokio-console` CLI tool.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mkutils::Tracing;
    ///
    /// let tracing = Tracing::default()
    ///     .with_tokio_console_enabled(true);
    /// ```
    #[must_use]
    pub const fn with_tokio_console_enabled(mut self, tokio_console_enabled: bool) -> Self {
        self.tokio_console_enabled = tokio_console_enabled;

        self
    }

    /// Sets a custom writer for log output.
    ///
    /// Allows you to direct logs to any custom writer implementation.
    ///
    /// # Type Parameters
    ///
    /// - `W2`: The new writer type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mkutils::Tracing;
    /// use std::io::stderr;
    ///
    /// let tracing = Tracing::default()
    ///     .with_writer(stderr);
    /// ```
    #[must_use]
    pub fn with_writer<W2>(self, writer: W2) -> Tracing<W2> {
        // TODO: any better way to do this?
        Tracing {
            level_filter: self.level_filter,
            json_enabled: self.json_enabled,
            span_events: self.span_events,
            tokio_console_enabled: self.tokio_console_enabled,
            tokio_console_ip_addr: self.tokio_console_ip_addr,
            tokio_console_port: self.tokio_console_port,
            writer,
        }
    }

    /// Configures the tracing output to write to stdout.
    ///
    /// This is useful when you want logs on stdout instead of stderr
    /// (the default).
    pub fn with_stdout_lock_writer(self) -> Tracing<StdoutLockFn> {
        self.with_writer(Self::stdout_lock_writer)
    }

    /// Configures the tracing output to write to stderr.
    ///
    /// This is the default output destination, but this method can be useful
    /// to explicitly reset the writer after calling `with_writer`.
    pub fn with_stderr_lock_writer(self) -> Tracing<StderrLockFn> {
        self.with_writer(Self::stderr_lock_writer)
    }

    fn init_helper<N, F, T, W0>(self, layer: Layer<Registry, N, Format<F, T>, W0>)
    where
        N: 'static + for<'a> FormatFields<'a> + Send + Sync,
        Format<F, T>: 'static + FormatEvent<Registry, N> + Send + Sync,
        W0: 'static + for<'a> MakeWriter<'a> + Send + Sync,
        W: 'static + for<'a> MakeWriter<'a> + Send + Sync,
    {
        let layer = layer
            .with_span_events(self.span_events)
            .with_writer(self.writer)
            .with_filter(self.level_filter);
        let layer = tracing_subscriber::registry().with(layer);

        if self.tokio_console_enabled {
            let tokio_console_server_addr = (self.tokio_console_ip_addr, self.tokio_console_port);
            let console_layer = ConsoleLayer::builder().server_addr(tokio_console_server_addr).spawn();
            let layer = layer.with(console_layer);

            layer.init();
        } else {
            layer.init();
        }
    }

    /// Consumes the builder and initializes the global tracing subscriber.
    ///
    /// This must be called to activate logging. It sets up the global subscriber
    /// with all the configured options and, if enabled, starts the tokio-console server.
    ///
    /// # Panics
    ///
    /// Panics if a global subscriber has already been set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mkutils::Tracing;
    ///
    /// Tracing::default().init();
    ///
    /// // Now you can use tracing macros
    /// tracing::info!("Application started");
    /// ```
    pub fn init(self)
    where
        W: 'static + for<'a> MakeWriter<'a> + Send + Sync,
    {
        let layer = tracing_subscriber::fmt::layer();

        if self.json_enabled {
            self.init_helper(layer.json());
        } else {
            self.init_helper(layer);
        }
    }
}
