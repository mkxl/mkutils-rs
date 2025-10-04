use console_subscriber::{ConsoleLayer, Server as ConsoleServer};
use std::{io::StderrLock, net::IpAddr};
use tracing::Subscriber as SubscriberTrait;
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{
        FormatEvent, FormatFields, Layer, MakeWriter,
        format::{FmtSpan, Format},
    },
    layer::{Layer as LayerTrait, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
};

pub struct Tracing<T> {
    level_filter: LevelFilter,
    span_events: FmtSpan,
    json_enabled: bool,
    tokio_console_enabled: bool,
    tokio_console_ip_addr: IpAddr,
    tokio_console_port: u16,
    writer: T,
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

impl<T> Tracing<T> {
    fn default_span_events() -> FmtSpan {
        FmtSpan::NEW | FmtSpan::CLOSE
    }

    fn default_writer() -> StderrLock<'static> {
        std::io::stderr().lock()
    }

    #[must_use]
    pub fn with_tokio_console_port(mut self, tokio_console_port: u16) -> Self {
        self.tokio_console_port = tokio_console_port;

        self
    }

    #[must_use]
    pub fn with_tokio_console_enabled(mut self, tokio_console_enabled: bool) -> Self {
        self.tokio_console_enabled = tokio_console_enabled;

        self
    }

    #[must_use]
    pub fn with_writer<W>(self, writer: W) -> Tracing<W> {
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

    fn layer() {}

    fn init_helper<S, N, F, T2, W>(self, log_layer: Layer<S, N, Format<F, T2>, W>)
    where
        S: for<'a> LookupSpan<'a> + SubscriberTrait,
        N: 'static + for<'a> FormatFields<'a>,
        Format<F, T2>: 'static + FormatEvent<S, N>,
        T: 'static + for<'a> MakeWriter<'a>,
        W: 'static + for<'a> MakeWriter<'a>, // T: 'static + for<'a> MakeWriter<'a> + Send + Sync,
    {
        let log_layer = log_layer
            .with_span_events(self.span_events)
            .with_writer(self.writer)
            .with_filter(self.level_filter);
        let registry = tracing_subscriber::registry().with(log_layer);

        if self.tokio_console_enabled {
            let tokio_console_server_addr = (self.tokio_console_ip_addr, self.tokio_console_port);
            let console_layer = ConsoleLayer::builder().server_addr(tokio_console_server_addr).spawn();
            let registry = registry.with(console_layer);

            registry.init();
        } else {
            registry.init();
        }
    }

    pub fn init(self)
    where
        T: 'static + for<'a> MakeWriter<'a> + Send + Sync,
    {
        let log_layer = tracing_subscriber::fmt::layer();

        if self.json_enabled {
            self.init_helper(log_layer.json())
        } else {
            self.init_helper(log_layer)
        }
    }
}
