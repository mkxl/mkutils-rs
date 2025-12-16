use crate::{geometry::PointU16, utils::Utils};
use anyhow::Error as AnyhowError;
use crossterm::{
    QueueableCommand,
    cursor::{Hide, Show},
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{BufWriter, Error as IoError, StdoutLock, Write};

/// A full-screen terminal interface manager using crossterm.
///
/// `Screen` sets up and manages a raw-mode terminal with an alternate screen buffer.
/// It handles:
/// - Entering/exiting raw mode
/// - Alternate screen buffer management
/// - Cursor visibility (hidden by default)
/// - Keyboard enhancement flags for better key event handling
/// - Optional mouse capture
///
/// When dropped, it automatically restores the terminal to its original state.
///
/// # Examples
///
/// ```rust,no_run
/// use mkutils::Screen;
///
/// fn main() -> Result<(), anyhow::Error> {
///     // Create a screen with mouse capture enabled
///     let mut screen = Screen::new(true)?;
///
///     // Write to the screen
///     use std::io::Write;
///     writeln!(screen.writer(), "Hello, terminal!")?;
///     screen.writer().flush()?;
///
///     // Screen is automatically cleaned up on drop
///     Ok(())
/// }
/// ```
pub struct Screen {
    stdout: BufWriter<StdoutLock<'static>>,
    with_mouse_capture: bool,
}

impl Screen {
    const CLEAR: Clear = Clear(ClearType::All);
    const PUSH_KEYBOARD_ENHANCEMENT_FLAGS: PushKeyboardEnhancementFlags =
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES);

    /// Creates a new `Screen`, entering raw mode and setting up the terminal.
    ///
    /// This method:
    /// - Enters raw mode (disables line buffering and echo)
    /// - Switches to the alternate screen buffer
    /// - Hides the cursor
    /// - Clears the screen
    /// - Enables keyboard enhancement flags for better key detection
    /// - Optionally enables mouse capture
    ///
    /// # Parameters
    ///
    /// - `with_mouse_capture`: If `true`, enables mouse event capture
    ///
    /// # Errors
    ///
    /// Returns an error if terminal setup fails.
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
        crossterm::terminal::enable_raw_mode()?;

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
        crossterm::terminal::disable_raw_mode()?;

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

    /// Returns a mutable reference to the buffered stdout writer.
    ///
    /// Use this to write content to the terminal. Remember to call `flush()`
    /// to ensure the content is displayed.
    pub const fn writer(&mut self) -> &mut BufWriter<StdoutLock<'static>> {
        &mut self.stdout
    }

    /// Returns the current terminal size in columns and rows.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal size cannot be determined.
    pub fn size() -> Result<PointU16, IoError> {
        crossterm::terminal::size()?
            .convert::<(u16, u16)>()
            .convert::<PointU16>()
            .ok()
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.on_drop().log_if_error().unit();
    }
}
