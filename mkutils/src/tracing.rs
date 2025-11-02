use console_subscriber::{ConsoleLayer, Server as ConsoleServer};
use std::{io::StderrLock, net::IpAddr};
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

pub struct Tracing<W> {
    level_filter: LevelFilter,
    span_events: FmtSpan,
    json_enabled: bool,
    tokio_console_enabled: bool,
    tokio_console_ip_addr: IpAddr,
    tokio_console_port: u16,
    writer: W,
}

impl Tracing<fn() -> StderrLock<'static>> {
    pub const DEFAULT_JSON_ENABLED: bool = false;
    pub const DEFAULT_LEVEL_FILTER: LevelFilter = LevelFilter::INFO;
    pub const DEFAULT_TOKIO_CONSOLE_ENABLED: bool = false;
    pub const DEFAULT_TOKIO_CONSOLE_IP_ADDR: IpAddr = ConsoleServer::DEFAULT_IP;
    pub const DEFAULT_TOKIO_CONSOLE_PORT: u16 = ConsoleServer::DEFAULT_PORT;
}

impl Default for Tracing<fn() -> StderrLock<'static>> {
    fn default() -> Self {
        Self {
            level_filter: Self::DEFAULT_LEVEL_FILTER,
            json_enabled: Self::DEFAULT_JSON_ENABLED,
            span_events: Self::default_span_events(),
            tokio_console_enabled: Self::DEFAULT_TOKIO_CONSOLE_ENABLED,
            tokio_console_ip_addr: Self::DEFAULT_TOKIO_CONSOLE_IP_ADDR,
            tokio_console_port: Self::DEFAULT_TOKIO_CONSOLE_PORT,
            writer: Self::default_writer,
        }
    }
}

impl<W> Tracing<W> {
    fn default_span_events() -> FmtSpan {
        FmtSpan::NEW | FmtSpan::CLOSE
    }

    fn default_writer() -> StderrLock<'static> {
        std::io::stderr().lock()
    }

    #[must_use]
    pub const fn with_level_filter(mut self, level_filter: LevelFilter) -> Self {
        self.level_filter = level_filter;

        self
    }

    #[must_use]
    pub const fn with_json_enabled(mut self, json_enabled: bool) -> Self {
        self.json_enabled = json_enabled;

        self
    }

    #[must_use]
    pub const fn with_tokio_console_port(mut self, tokio_console_port: u16) -> Self {
        self.tokio_console_port = tokio_console_port;

        self
    }

    #[must_use]
    pub const fn with_tokio_console_enabled(mut self, tokio_console_enabled: bool) -> Self {
        self.tokio_console_enabled = tokio_console_enabled;

        self
    }

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
