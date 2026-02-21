use crate::{geometry::PointUsize, utils::Utils};
use ratatui::text::Line as RatatuiLine;
use std::ops::Range;

pub trait Content {
    fn size(&self) -> PointUsize;

    fn lines(&self, rows: Range<usize>, cols: Range<usize>) -> Vec<RatatuiLine<'_>>;
}

impl Content for Vec<RatatuiLine<'_>> {
    fn size(&self) -> PointUsize {
        let num_rows = self.len();
        let num_cols = self
            .iter()
            .flat_map(|line| line.spans.as_slice())
            .map(|span| span.content.len_extended_graphemes())
            .sum();

        PointUsize::new(num_cols, num_rows)
    }

    fn lines(&self, rows: Range<usize>, cols: Range<usize>) -> Vec<RatatuiLine<'_>> {
        self[rows].iter().map(|line| line.subline(cols.clone())).collect()
    }
}
