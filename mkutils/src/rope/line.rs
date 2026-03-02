use crate::{
    rope::{
        atoms::{Atom, Atoms},
        chunk_summary::NumExtendedGraphemes,
    },
    utils::Utils,
};

pub struct Line<'r, 'a> {
    atoms: &'a mut Atoms<'r>,
    seen_newline: bool,
}

impl<'r, 'a> Line<'r, 'a> {
    const INITIAL_SEEN_NEWLINE: bool = false;

    pub const fn new(atoms: &'a mut Atoms<'r>) -> Self {
        let seen_newline = Self::INITIAL_SEEN_NEWLINE;

        Self { atoms, seen_newline }
    }

    pub fn advance(&mut self, num_extended_graphemes: NumExtendedGraphemes) {
        if self.seen_newline {
            return;
        }

        let distance_advanced = self.atoms.advance_within_line(num_extended_graphemes).into_ok_err();

        if distance_advanced.newlines().is_positive() {
            self.seen_newline.set_true();
        }
    }
}

impl<'r> Iterator for Line<'r, '_> {
    type Item = Atom<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seen_newline {
            return None;
        }

        let atom = self.atoms.next()?;

        if atom.extended_grapheme().is_newline() {
            self.seen_newline.set_true();
        }

        atom.some()
    }
}
