use crate::{geometry::PointU16, utils::Utils};
use anyhow::Error as AnyhowError;
use ratatui::crossterm::{
    QueueableCommand,
    cursor::{Hide, Show},
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{BufWriter, Error as IoError, StdoutLock, Write};

pub struct Screen {
    stdout: BufWriter<StdoutLock<'static>>,
    with_mouse_capture: bool,
}

impl Screen {
    const CLEAR: Clear = Clear(ClearType::All);
    const PUSH_KEYBOARD_ENHANCEMENT_FLAGS: PushKeyboardEnhancementFlags =
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES);

    pub fn new(with_mouse_capture: bool) -> Result<Self, AnyhowError> {
        let stdout = std::io::stdout().lock().buf_writer();
        let mut screen = Self {
            stdout,
            with_mouse_capture,
        };

        screen.on_new()?;

        screen.ok()
    }

    fn on_new(&mut self) -> Result<(), IoError> {
        ratatui::crossterm::terminal::enable_raw_mode()?;

        if self.with_mouse_capture {
            self.stdout.queue(EnableMouseCapture)?;
        }

        self.stdout
            .queue(EnterAlternateScreen)?
            .queue(Self::PUSH_KEYBOARD_ENHANCEMENT_FLAGS)?
            .queue(Hide)?
            .queue(Self::CLEAR)?
            .flush()?
            .ok()
    }

    fn on_drop(&mut self) -> Result<(), IoError> {
        ratatui::crossterm::terminal::disable_raw_mode()?;

        if self.with_mouse_capture {
            self.stdout.queue(DisableMouseCapture)?;
        }

        self.stdout
            .queue(LeaveAlternateScreen)?
            .queue(PopKeyboardEnhancementFlags)?
            .queue(Show)?
            .flush()?
            .ok()
    }

    pub const fn writer(&mut self) -> &mut BufWriter<StdoutLock<'static>> {
        &mut self.stdout
    }

    pub fn size() -> Result<PointU16, IoError> {
        ratatui::crossterm::terminal::size()?.convert::<PointU16>().ok()
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.on_drop().log_if_error().unit();
    }
}
