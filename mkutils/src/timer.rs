use crate::utils::Utils;
use std::time::Instant;

pub struct Timer {
    instant: Instant,
}

impl Timer {
    #[must_use]
    pub fn now() -> Self {
        let instant = Instant::now();

        Self { instant }
    }

    pub fn log(&mut self, event: &str) {
        let begin = self.instant.mem_replace(Instant::now());
        let duration = self.instant - begin;

        tracing::info!(message = "close", event, time.busy = ?duration);
    }
}
