use crate::utils::Utils;
use anyhow::Error as AnyhowError;
use crossterm::{
    QueueableCommand,
    cursor::{Hide, Show},
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{BufWriter, Error as IoError, StdoutLock, Write};

pub struct Screen {
    stdout: BufWriter<StdoutLock<'static>>,
}

impl Screen {
    const CLEAR: Clear = Clear(ClearType::All);
    const PUSH_KEYBOARD_ENHANCEMENT_FLAGS: PushKeyboardEnhancementFlags =
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES);

    pub fn new() -> Result<Self, AnyhowError> {
        let stdout = std::io::stdout().lock().buf_writer();
        let mut screen = Self { stdout };

        screen.on_new()?;

        screen.ok()
    }

    fn on_new(&mut self) -> Result<(), IoError> {
        crossterm::terminal::enable_raw_mode()?;

        self.stdout
            .queue(EnterAlternateScreen)?
            .queue(Self::PUSH_KEYBOARD_ENHANCEMENT_FLAGS)?
            .queue(Hide)?
            .queue(Self::CLEAR)?
            .flush()?
            .ok()
    }

    fn on_drop(&mut self) -> Result<(), IoError> {
        crossterm::terminal::disable_raw_mode()?;

        self.stdout
            .queue(LeaveAlternateScreen)?
            .queue(PopKeyboardEnhancementFlags)?
            .queue(Show)?
            .flush()?
            .ok()
    }

    pub fn write_all_and_flush(&mut self, byte_str: &[u8]) -> Result<(), IoError> {
        self.stdout.write_all_and_flush(byte_str)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.on_drop().log_if_error().unit();
    }
}
