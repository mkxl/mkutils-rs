use crate::utils::Utils;
use console_subscriber::{ConsoleLayer, Server as ConsoleServer};
use std::net::IpAddr;
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{MakeWriter, format::FmtSpan},
    layer::{Layer, SubscriberExt},
    registry::Registry,
    util::SubscriberInitExt,
};

pub struct Tracing<T> {
    level_filter: LevelFilter,
    span_events: FmtSpan,
    tokio_console_enabled: bool,
    tokio_console_ip_addr: IpAddr,
    tokio_console_port: u16,
    writer: Option<T>,
}

impl Default for Tracing<()> {
    fn default() -> Self {
        Self {
            level_filter: Self::DEFAULT_LEVEL_FILTER,
            span_events: Self::default_span_events(),
            tokio_console_enabled: Self::DEFAULT_TOKIO_CONSOLE_ENABLED,
            tokio_console_ip_addr: Self::DEFAULT_TOKIO_CONSOLE_IP_ADDR,
            tokio_console_port: Self::DEFAULT_TOKIO_CONSOLE_PORT,
            writer: None,
        }
    }
}

impl<T> Tracing<T> {
    const DEFAULT_LEVEL_FILTER: LevelFilter = LevelFilter::INFO;
    const DEFAULT_TOKIO_CONSOLE_ENABLED: bool = false;
    const DEFAULT_TOKIO_CONSOLE_IP_ADDR: IpAddr = ConsoleServer::DEFAULT_IP;
    const DEFAULT_TOKIO_CONSOLE_PORT: u16 = ConsoleServer::DEFAULT_PORT;

    fn default_span_events() -> FmtSpan {
        FmtSpan::NEW | FmtSpan::CLOSE
    }

    #[must_use]
    pub fn with_writer<W>(self, writer: W) -> Tracing<W> {
        // TODO: any better way to do this?
        Tracing {
            level_filter: self.level_filter,
            span_events: self.span_events,
            tokio_console_enabled: self.tokio_console_enabled,
            tokio_console_ip_addr: self.tokio_console_ip_addr,
            tokio_console_port: self.tokio_console_port,
            writer: writer.some(),
        }
    }

    fn init_helper<L: Layer<Registry> + Send + Sync>(&self, log_layer: L) {
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

    pub fn init(mut self)
    where
        T: 'static + for<'a> MakeWriter<'a> + Send + Sync,
    {
        // TODO: [https://github.com/tokio-rs/tracing/pull/2739]
        let log_layer = tracing_subscriber::fmt::layer()
            .with_span_events(self.span_events.clone())
            .json();

        match self.writer.take() {
            Some(writer) => self.init_helper(log_layer.with_writer(writer).with_filter(self.level_filter)),
            None => self.init_helper(log_layer.with_filter(self.level_filter)),
        }
    }
}
