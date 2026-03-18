use crate::{geometry::PointU16, utils::Utils};
use crossterm::{
    QueueableCommand,
    cursor::{Hide, Show},
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{BufWriter, Error as IoError, StdoutLock, Write};

pub type Stdout = BufWriter<StdoutLock<'static>>;
pub type ScreenTerminal<'a> = Terminal<CrosstermBackend<&'a mut Stdout>>;

#[derive(Default)]
pub struct ScreenConfig {
    mouse_capture: bool,
    bracketed_paste: bool,
}

impl ScreenConfig {
    #[must_use]
    pub const fn with_mouse_capture(self, mouse_capture: bool) -> Self {
        Self { mouse_capture, ..self }
    }

    #[must_use]
    pub const fn with_bracketed_paste(self, bracketed_paste: bool) -> Self {
        Self {
            bracketed_paste,
            ..self
        }
    }

    pub fn build(self) -> Result<Screen, IoError> {
        Screen::new(self)
    }
}

pub struct Screen {
    stdout: Stdout,
    config: ScreenConfig,
}

impl Screen {
    const CLEAR: Clear = Clear(ClearType::All);
    const PUSH_KEYBOARD_ENHANCEMENT_FLAGS: PushKeyboardEnhancementFlags =
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES);

    #[must_use]
    pub fn config() -> ScreenConfig {
        ScreenConfig::default()
    }

    fn new(config: ScreenConfig) -> Result<Self, IoError> {
        let stdout = std::io::stdout().lock().buf_writer();
        let mut screen = Self { stdout, config };

        screen.on_new()?;

        screen.ok()
    }

    fn on_new(&mut self) -> Result<(), IoError> {
        ratatui::crossterm::terminal::enable_raw_mode()?;

        if self.config.mouse_capture {
            self.stdout.queue(EnableMouseCapture)?;
        }

        if self.config.bracketed_paste {
            self.stdout.queue(EnableBracketedPaste)?;
        }

        self.stdout
            .queue(EnterAlternateScreen)?
            .queue(Self::PUSH_KEYBOARD_ENHANCEMENT_FLAGS)?
            .queue(Hide)?
            .queue(Self::CLEAR)?
            .flush()?
            .ok()
    }

    fn on_drop_impl(&mut self) -> Result<(), IoError> {
        ratatui::crossterm::terminal::disable_raw_mode()?;

        if self.config.mouse_capture {
            self.stdout.queue(DisableMouseCapture)?;
        }

        if self.config.bracketed_paste {
            self.stdout.queue(DisableBracketedPaste)?;
        }

        self.stdout
            .queue(LeaveAlternateScreen)?
            .queue(PopKeyboardEnhancementFlags)?
            .queue(Show)?
            .flush()?
            .ok()
    }

    fn on_drop(&mut self) {
        self.on_drop_impl().log_if_error().unit();
    }

    pub const fn writer_mut(&mut self) -> &mut BufWriter<StdoutLock<'static>> {
        &mut self.stdout
    }

    pub fn size() -> Result<PointU16, IoError> {
        ratatui::crossterm::terminal::size()?.convert::<PointU16>().ok()
    }

    pub fn terminal(&mut self) -> Result<ScreenTerminal<'_>, IoError> {
        let backend = CrosstermBackend::new(&mut self.stdout);
        let terminal = Terminal::new(backend)?;

        terminal.ok()
    }

    // NOTE:
    // - [https://chatgpt.com/c/695cfdcf-be98-8331-90dc-123981ab6ba8]
    // - [https://chatgpt.com/c/695d51f6-c0b0-832d-865f-31462c278e0a]
    #[must_use]
    pub fn into_stdout(mut self) -> Stdout {
        self.on_drop();

        let mut this = self.into_manually_drop();

        unsafe {
            this.config.as_ptr_mut().drop_in_place();

            this.stdout.as_ptr().read()
        }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.on_drop();
    }
}
