use crate::{
    rope::{
        atoms::Atoms,
        distance::{NumExtendedGraphemes, NumNewlines},
        line::Line,
        rope::Rope,
    },
    utils::Utils,
};
use std::{iter::Take, ops::Range};

#[allow(clippy::struct_field_names)]
pub struct Lines<'r> {
    atoms: Atoms<'r>,
    lines: Range<NumNewlines>,
    extended_graphemes: Range<NumExtendedGraphemes>,
    next_line_offset: NumNewlines,
}

impl<'r> Lines<'r> {
    #[must_use]
    pub fn new(rope: &'r Rope, lines: Range<NumNewlines>, extended_graphemes: Range<NumExtendedGraphemes>) -> Self {
        let atoms = rope.atoms_at_line(lines.start.into());
        let next_line_offset = lines.start;

        Self {
            atoms,
            lines,
            extended_graphemes,
            next_line_offset,
        }
    }

    pub fn next_line<'l>(&'l mut self) -> Option<Take<Line<'r, 'l>>> {
        if self.lines.end <= self.next_line_offset {
            return None;
        }

        while self.atoms.rope_offset().newlines < self.next_line_offset {
            self.atoms.advance_to_start_of_next_line();
        }

        self.next_line_offset.increment();

        let mut line = self.atoms.line();

        line.advance(self.extended_graphemes.start);

        line.take(self.extended_graphemes.len_range().into()).some()
    }

    pub fn to_vec(&mut self) -> Vec<String> {
        let mut lines = Vec::new();

        while let Some(mut line) = self.next_line() {
            lines.push(line.collect_atoms());
        }

        lines
    }
}
