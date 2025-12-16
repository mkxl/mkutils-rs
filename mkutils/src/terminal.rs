use crate::{geometry::PointU16, utils::Utils};
use crossterm::terminal::SetTitle;
use ratatui::{
    Frame, Terminal as BaseRatatuiTerminal, TerminalOptions, Viewport, backend::CrosstermBackend,
    crossterm::QueueableCommand,
};
use std::{
    fmt::Display,
    io::{Error as IoError, Write},
};

type RatatuiTerminal = BaseRatatuiTerminal<CrosstermBackend<Vec<u8>>>;

/// A terminal UI renderer using ratatui with in-memory rendering.
///
/// `Terminal` wraps ratatui's terminal to render UI to an in-memory buffer
/// instead of directly to the screen. This is useful for testing, generating
/// terminal output as bytes, or when you need control over when output is displayed.
///
/// # Examples
///
/// ```rust,no_run
/// use mkutils::{Terminal, Point};
/// use ratatui::widgets::{Block, Borders};
///
/// fn main() -> Result<(), std::io::Error> {
///     let mut terminal = Terminal::new(Point::new(80, 24))?;
///
///     let bytes = terminal.draw(|frame| {
///         let block = Block::default()
///             .title("Hello")
///             .borders(Borders::ALL);
///         frame.render_widget(block, frame.area());
///         Ok(())
///     })?;
///
///     // bytes now contains the rendered terminal output
///     std::io::stdout().write_all(&bytes)?;
///     Ok(())
/// }
/// ```
pub struct Terminal {
    ratatui_terminal: RatatuiTerminal,
    byte_str: Vec<u8>,
}

impl Terminal {
    /// Creates a new `Terminal` with the specified size.
    ///
    /// # Parameters
    ///
    /// - `size`: A `Point<u16>` representing the terminal dimensions (width, height)
    ///
    /// # Errors
    ///
    /// Returns an error if terminal initialization fails.
    pub fn new(size: PointU16) -> Result<Self, IoError> {
        let ratatui_terminal = Self::ratatui_terminal(size)?;
        let byte_str = Vec::new();
        let terminal = Self {
            ratatui_terminal,
            byte_str,
        };

        terminal.ok()
    }

    fn ratatui_terminal(size: PointU16) -> Result<RatatuiTerminal, IoError> {
        let backend = CrosstermBackend::new(Vec::new());
        let rect = size.ratatui_rect();
        let viewport = Viewport::Fixed(rect);
        let options = TerminalOptions { viewport };
        let mut ratatui_terminal = RatatuiTerminal::with_options(backend, options)?;

        ratatui_terminal.resize(rect)?;

        ratatui_terminal.ok()
    }

    /// Returns the current size of the terminal.
    ///
    /// # Errors
    ///
    /// Returns an error if the size cannot be determined.
    pub fn size(&self) -> Result<PointU16, IoError> {
        self.ratatui_terminal
            .size()?
            .convert::<(u16, u16)>()
            .convert::<PointU16>()
            .ok()
    }

    /// Resizes the terminal to the specified dimensions.
    ///
    /// # Parameters
    ///
    /// - `num_cols`: New width in columns
    /// - `num_rows`: New height in rows
    ///
    /// # Errors
    ///
    /// Returns an error if resizing fails.
    pub fn resize(&mut self, num_cols: u16, num_rows: u16) -> Result<(), IoError> {
        let size = num_cols.pair(num_rows).convert::<PointU16>();

        self.ratatui_terminal.resize(size.ratatui_rect())?;

        ().ok()
    }

    /// Sets the terminal title.
    ///
    /// This generates terminal escape sequences to set the title, which will be
    /// included in the output of the next `draw()` call.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the title fails.
    // NOTE: no way to set the terminal title using ratatui, so must fallback to
    // crossterm methods
    pub fn set_title<T: Display>(&mut self, title: T) -> Result<(), IoError> {
        let set_title = SetTitle(title);

        self.byte_str.queue(set_title)?.flush()?;

        ().ok()
    }

    /// Renders the terminal UI and returns the output as bytes.
    ///
    /// The provided closure receives a ratatui `Frame` and should render widgets to it.
    /// After rendering, this method returns a byte vector containing all terminal
    /// escape sequences needed to display the rendered output.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails or if the draw function returns an error.
    pub fn draw<F: FnOnce(&mut Frame) -> Result<(), IoError>>(&mut self, draw_fn: F) -> Result<Vec<u8>, IoError> {
        self.ratatui_terminal.try_draw(draw_fn)?;

        let mut byte_str = self.ratatui_terminal.backend_mut().writer_mut().split_off(0);

        byte_str.append(&mut self.byte_str);

        byte_str.ok()
    }
}
