use crate::utils::Utils;
use std::time::Instant;

pub struct Timer {
    message: &'static str,
    instant: Instant,
}

impl Timer {
    #[must_use]
    pub fn new(message: &'static str) -> Self {
        let instant = Instant::now();

        Self { message, instant }
    }

    pub fn log(&mut self) {
        let begin = self.instant.mem_replace(Instant::now());
        let duration = self.instant - begin;

        tracing::info!(message = self.message, time.busy = ?duration);
    }
}
