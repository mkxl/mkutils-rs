use crate::geometry::PointUsize;
use ratatui::text::Line as RatatuiLine;
use std::{borrow::Borrow, ops::Range};

pub trait Content {
    type Line<'a>: Borrow<RatatuiLine<'a>>
    where
        Self: 'a;

    fn size(&self) -> PointUsize;

    fn lines(&self, rows: Range<usize>, cols: Range<usize>) -> Vec<Self::Line<'_>>;
}
