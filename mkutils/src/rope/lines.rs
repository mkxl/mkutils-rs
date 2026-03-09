use crate::{
    rope::{
        atoms::Atoms,
        chunk_summary::{NumExtendedGraphemes, NumNewlines},
        line::Line,
        rope::Rope,
    },
    utils::Utils,
};
use std::{iter::Take, ops::Range};

#[allow(clippy::struct_field_names)]
pub struct Lines<'r> {
    atoms: Atoms<'r>,
    line_offsets: Range<NumNewlines>,
    extended_grapheme_offsets: Range<NumExtendedGraphemes>,
    next_line_offset: NumNewlines,
}

impl<'r> Lines<'r> {
    #[must_use]
    pub fn new(
        rope: &'r Rope,
        line_offsets: Range<NumNewlines>,
        extended_grapheme_offsets: Range<NumExtendedGraphemes>,
    ) -> Self {
        let atoms = rope.atoms_at_line(line_offsets.start.into());
        let next_line_offset = line_offsets.start;

        Self {
            atoms,
            line_offsets,
            extended_grapheme_offsets,
            next_line_offset,
        }
    }

    pub fn next_line<'l>(&'l mut self) -> Option<Take<Line<'r, 'l>>> {
        if self.line_offsets.end <= self.next_line_offset {
            return None;
        }

        while self.atoms.rope_offset().newlines() < self.next_line_offset {
            if self.atoms.advance_to_start_of_next_line().is_err() {
                return None;
            }
        }

        self.next_line_offset.increment();

        let mut line = self.atoms.line();

        line.advance(self.extended_grapheme_offsets.start);

        line.take(self.extended_grapheme_offsets.len_range().into()).some()
    }

    pub fn to_vec(&mut self) -> Vec<String> {
        let mut lines = Vec::new();

        while let Some(mut line) = self.next_line() {
            lines.push(line.collect_atoms());
        }

        lines
    }
}
