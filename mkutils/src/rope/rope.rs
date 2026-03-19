use crate::{
    geometry::PointUsize,
    rope::{
        atoms::Atoms,
        chunk::Chunk,
        chunk_summary::{Length, LengthExtendedGraphemes, LengthLines},
        lines::Lines,
    },
    utils::Utils,
};
use derive_more::{From, Into};
use num::traits::SaturatingSub;
use std::{
    fmt::{Display, Error as FmtError, Formatter},
    ops::Range,
};
use zed_sum_tree::{Bias, Item, SumTree, Summary};

#[derive(Default, From, Into)]
pub struct Rope {
    chunk_sum_tree: SumTree<Chunk>,
}

impl Rope {
    pub const CONTEXT: <<Chunk as Item>::Summary as Summary>::Context<'static> = ();
    pub const BIAS: Bias = Bias::Right;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_extended_graphemes(&mut self, mut extended_graphemes: &str) {
        while !extended_graphemes.is_empty() {
            let mut should_push_empty_chunk = self.chunk_sum_tree.is_empty();
            let update_last = |last_chunk: &mut Chunk| match last_chunk.try_push_extended_graphemes(extended_graphemes)
            {
                Ok(_last_chunk) => extended_graphemes.assign(""),
                Err(remaining_extended_graphemes) => {
                    extended_graphemes.assign(remaining_extended_graphemes);
                    should_push_empty_chunk.set_true();
                }
            };

            self.chunk_sum_tree.update_last(update_last, Self::CONTEXT);

            if should_push_empty_chunk {
                self.push(Chunk::empty());
            }
        }
    }

    #[must_use]
    pub const fn sum_tree(&self) -> &SumTree<Chunk> {
        &self.chunk_sum_tree
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
    pub fn lines(&self, line_offsets: Range<usize>, extended_grapheme_offsets: Range<usize>) -> Lines<'_> {
        let line_offsets = line_offsets.map_range(LengthLines::from);
        let extended_grapheme_offsets = extended_grapheme_offsets.map_range(LengthExtendedGraphemes::from);

        Lines::new(self, line_offsets, extended_grapheme_offsets)
    }

    #[must_use]
    pub fn size(&self) -> PointUsize {
        let chunk_summary = self.chunk_sum_tree.summary();

        PointUsize::new(
            chunk_summary.line_lengths().max().into(),
            chunk_summary.len().lines().into(),
        )
    }

    fn push(&mut self, chunk: Chunk) -> &mut Self {
        self.chunk_sum_tree.push(chunk, Self::CONTEXT);

        self
    }

    fn append(&mut self, rhs: Self) -> &mut Self {
        self.chunk_sum_tree.append(rhs.chunk_sum_tree, Self::CONTEXT);

        self
    }

    // TODO: figure out how to rename this [split_at()] and not have it conflict with [Utils::split_at()]
    #[must_use]
    fn split(&self, extended_grapheme_offset: LengthExtendedGraphemes) -> (Self, Self) {
        let mut chunks_cursor = self.chunk_sum_tree.cursor::<LengthExtendedGraphemes>(Self::CONTEXT);
        let mut prefix_rope = chunks_cursor
            .slice(&extended_grapheme_offset, Self::BIAS)
            .convert::<Self>();
        let prefix_len_extended_graphemes = *chunks_cursor.start();

        if prefix_len_extended_graphemes == extended_grapheme_offset {
            return prefix_rope.pair(chunks_cursor.suffix().into());
        }

        let Some(chunk) = chunks_cursor.item() else {
            return prefix_rope.pair(Self::new());
        };

        let chunk_extended_grapheme_offset = extended_grapheme_offset.saturating_sub(&prefix_len_extended_graphemes);
        let (left_chunk, right_chunk) = chunk.split(chunk_extended_grapheme_offset);
        let mut suffix_rope = Self::new();

        if left_chunk.len().extended_graphemes().is_positive() {
            prefix_rope.push(left_chunk);
        }

        chunks_cursor.next();

        if right_chunk.len().extended_graphemes().is_positive() {
            suffix_rope.push(right_chunk);
        }

        suffix_rope.append(chunks_cursor.suffix().into());

        prefix_rope.pair(suffix_rope)
    }

    pub fn insert(&mut self, extended_grapheme_offset: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        let (mut prefix_rope, suffix_rope) = self.split(extended_grapheme_offset.into());

        prefix_rope.push_extended_graphemes(text);
        prefix_rope.append(suffix_rope);
        self.assign(prefix_rope);
    }

    // NOTE:
    // - this calls split_chunks_at twice on the original rope (before mutation)
    // - this is safe because we only assign to self.chunks at the very end. the sumtree is persistent/immutable under
    //   the hood (arc-based structural sharing), so both splits read from the same stable snapshot
    pub fn delete(&mut self, extended_grapheme_offsets: Range<usize>) {
        if extended_grapheme_offsets.is_empty() {
            return;
        }

        let (mut prefix_rope, _suffix_rope) = self.split(extended_grapheme_offsets.start.into());
        let (_prefix_rope, suffix_rope) = self.split(extended_grapheme_offsets.end.into());

        prefix_rope.append(suffix_rope);
        self.assign(prefix_rope);
    }

    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        let (mut prefix_rope, _suffix) = self.split(range.start.into());
        let (_prefix, suffix_rope) = self.split(range.end.into());

        prefix_rope.push_extended_graphemes(text);
        prefix_rope.append(suffix_rope);
        self.assign(prefix_rope);
    }

    #[must_use]
    pub fn len(&self) -> Length {
        self.chunk_sum_tree.summary().len()
    }
}

impl From<&str> for Rope {
    fn from(string: &str) -> Self {
        string.once().collect()
    }
}

impl Display for Rope {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        for chunk in self.chunk_sum_tree.iter() {
            formatter.write_str(chunk.as_str())?;
        }

        ().ok()
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
