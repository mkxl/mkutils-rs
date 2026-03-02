use crate::{
    geometry::PointUsize,
    rope::{
        atoms::Atoms,
        chunk::Chunk,
        chunk_summary::{NumExtendedGraphemes, NumNewlines},
        lines::Lines,
    },
    utils::Utils,
};
use std::ops::Range;
use zed_sum_tree::{Item, SumTree, Summary};

#[derive(Default)]
pub struct Rope {
    chunks: SumTree<Chunk>,
}

impl Rope {
    pub const CONTEXT: <<Chunk as Item>::Summary as Summary>::Context<'static> = ();

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_extended_graphemes(&mut self, extended_graphemes_str: &str) {
        let mut extended_graphemes = extended_graphemes_str.extended_graphemes().peekable();

        while extended_graphemes.is_non_empty() {
            let mut push_empty_chunk = self.chunks.is_empty();
            let update_last = |last_chunk: &mut Chunk| {
                while let Some(extended_grapheme) = extended_graphemes.peek() {
                    if last_chunk.try_push_extended_grapheme(extended_grapheme).is_ok() {
                        extended_graphemes.next();
                    } else {
                        push_empty_chunk.set_true();

                        break;
                    }
                }
            };

            self.chunks.update_last(update_last, Self::CONTEXT);

            if push_empty_chunk {
                self.chunks.push(Chunk::empty(), Self::CONTEXT);
            }
        }
    }

    #[must_use]
    pub const fn sum_tree(&self) -> &SumTree<Chunk> {
        &self.chunks
    }

    #[must_use]
    pub fn atoms(&self) -> Atoms<'_> {
        self.atoms_at_line(0)
    }

    #[must_use]
    pub fn atoms_at_line(&self, target_line_offset: usize) -> Atoms<'_> {
        Atoms::from_rope(self, target_line_offset.into())
    }

    #[must_use]
    pub fn lines(&self, lines: Range<usize>, extended_graphemes: Range<usize>) -> Lines<'_> {
        let lines = lines.map_range(NumNewlines::from);
        let extended_graphemes = extended_graphemes.map_range(NumExtendedGraphemes::from);

        Lines::new(self, lines, extended_graphemes)
    }

    #[must_use]
    pub fn size(&self) -> PointUsize {
        let chunk_summary = self.chunks.summary();

        PointUsize::new(
            chunk_summary.line_lengths().max().into(),
            chunk_summary.length().newlines().into(),
        )
    }
}

impl From<&str> for Rope {
    fn from(string: &str) -> Self {
        let mut rope = Self::new();

        rope.push_extended_graphemes(string);

        rope
    }
}

impl<'a> FromIterator<&'a str> for Rope {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut rope = Self::new();

        for string in iter {
            rope.push_extended_graphemes(string);
        }

        rope
    }
}
