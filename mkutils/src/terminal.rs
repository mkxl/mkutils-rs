use crate::{geometry::PointU16, utils::Utils};
use ratatui::{
    Frame, Terminal as BaseRatatuiTerminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    crossterm::{QueueableCommand, terminal::SetTitle},
};
use std::{
    fmt::Display,
    io::{Error as IoError, Write},
};

type RatatuiTerminal = BaseRatatuiTerminal<CrosstermBackend<Vec<u8>>>;

pub struct Terminal {
    ratatui_terminal: RatatuiTerminal,
    byte_str: Vec<u8>,
}

impl Terminal {
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

    pub fn size(&self) -> Result<PointU16, IoError> {
        self.ratatui_terminal.size()?.convert::<PointU16>().ok()
    }

    pub fn resize(&mut self, num_cols: u16, num_rows: u16) -> Result<(), IoError> {
        let size = num_cols.pair(num_rows).convert::<PointU16>();

        self.ratatui_terminal.resize(size.ratatui_rect())?;

        ().ok()
    }

    // NOTE: no way to set the terminal title using ratatui, so must fallback to
    // crossterm methods
    pub fn set_title<T: Display>(&mut self, title: T) -> Result<(), IoError> {
        let set_title = SetTitle(title);

        self.byte_str.queue(set_title)?.flush()?;

        ().ok()
    }

    pub fn draw<F: FnOnce(&mut Frame) -> Result<(), IoError>>(&mut self, draw_fn: F) -> Result<Vec<u8>, IoError> {
        self.ratatui_terminal.try_draw(draw_fn)?;

        let mut byte_str = self.ratatui_terminal.backend_mut().writer_mut().split_off(0);

        byte_str.append(&mut self.byte_str);

        byte_str.ok()
    }
}
