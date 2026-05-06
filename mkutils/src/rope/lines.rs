use crate::{
    rope::{atoms::Atoms, line::Line, rope::Rope},
    utils::Utils,
};
use std::{iter::Take, ops::Range};

pub struct Lines<'r> {
    atoms: Atoms<'r>,
    line_indices: Range<usize>,
    extended_grapheme_indices: Range<usize>,
    next_line_index: usize,
}

impl<'r> Lines<'r> {
    #[must_use]
    pub fn new(rope: &'r Rope, line_indices: Range<usize>, extended_grapheme_indices: Range<usize>) -> Self {
        let atoms = rope.atoms_at_line(line_indices.start);
        let next_line_index = line_indices.start;

        Self {
            atoms,
            line_indices,
            extended_grapheme_indices,
            next_line_index,
        }
    }

    pub fn next_line<'l>(&'l mut self) -> Option<Take<Line<'r, 'l>>> {
        if self.line_indices.end <= self.next_line_index {
            return None;
        }

        while self.atoms.offset().length.newlines < self.next_line_index {
            if self.atoms.advance_to_start_of_next_line().is_err() {
                return None;
            }
        }

        self.next_line_index.increment();

        let mut line = self.atoms.line();

        line.advance(self.extended_grapheme_indices.start);

        line.take(self.extended_grapheme_indices.len_range()).some()
    }

    #[must_use]
    pub fn into_vec(mut self) -> Vec<String> {
        let mut lines = Vec::new();

        while let Some(mut line) = self.next_line() {
            lines.push(line.collect_atoms());
        }

        lines
    }
}
