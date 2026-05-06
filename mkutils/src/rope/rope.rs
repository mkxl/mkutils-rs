use crate::{
    geometry::PointUsize,
    rope::{
        atoms::Atoms,
        chunk::Chunk,
        lines::Lines,
        text_summary::{Length, LengthExtendedGraphemes, LineLengthSet, TextSummary},
    },
    utils::Utils,
};
use derive_more::{From, Into};
use num::traits::ConstZero;
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
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn chunk_sum_tree(&self) -> &SumTree<Chunk> {
        &self.chunk_sum_tree
    }

    #[must_use]
    pub fn atoms(&self) -> Atoms<'_> {
        self.atoms_at_line(usize::ZERO)
    }

    #[must_use]
    pub fn atoms_at_line(&self, line_index: usize) -> Atoms<'_> {
        Atoms::from_rope(self, line_index)
    }

    #[must_use]
    pub fn lines(&self, line_indices: Range<usize>, extended_grapheme_indices: Range<usize>) -> Lines<'_> {
        Lines::new(self, line_indices, extended_grapheme_indices)
    }

    fn text_summary(&self) -> &TextSummary {
        self.chunk_sum_tree.summary()
    }

    #[must_use]
    pub fn len_extended_graphemes(&self) -> usize {
        self.text_summary().length.extended_graphemes
    }

    #[must_use]
    pub fn len_newlines(&self) -> usize {
        self.text_summary().length.newlines
    }

    #[must_use]
    pub fn length(&self) -> &Length {
        &self.text_summary().length
    }

    #[must_use]
    pub fn line_lengths(&self) -> &LineLengthSet {
        &self.text_summary().line_lengths
    }

    #[must_use]
    pub fn size(&self) -> PointUsize {
        let text_summary = self.text_summary();

        PointUsize::new(text_summary.line_lengths.max, text_summary.length.newlines)
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

    pub fn push(&mut self, chunk: Chunk) -> &mut Self {
        self.chunk_sum_tree.push(chunk, Self::CONTEXT);

        self
    }

    pub fn append(&mut self, other: Self) -> &mut Self {
        self.chunk_sum_tree.append(other.chunk_sum_tree, Self::CONTEXT);

        self
    }

    // TODO-205a89: figure out how to rename this [split_at()] and not have it conflict with [Utils::split_at()]
    #[must_use]
    fn split(&self, index: usize) -> (Self, Self) {
        let mut chunks_cursor = self.chunk_sum_tree.cursor::<LengthExtendedGraphemes>(Self::CONTEXT);
        let index_dim = index.convert::<LengthExtendedGraphemes>();
        let mut prefix_rope = chunks_cursor.slice(&index_dim, Self::BIAS).convert::<Self>();
        let prefix_rope_len_extended_graphemes = chunks_cursor.start().copied().convert::<usize>();

        if prefix_rope_len_extended_graphemes == index {
            return prefix_rope.pair(chunks_cursor.suffix().into());
        }

        let Some(chunk) = chunks_cursor.item() else {
            return prefix_rope.pair(Self::empty());
        };

        let chunk_extended_grapheme_offset = index.saturating_sub(prefix_rope_len_extended_graphemes);
        let (prefix_chunk, suffix_chunk) = chunk.split(chunk_extended_grapheme_offset);
        let mut suffix_rope = Self::empty();

        if prefix_chunk.is_not_empty() {
            prefix_rope.push(prefix_chunk);
        }

        chunks_cursor.next();

        if suffix_chunk.is_not_empty() {
            suffix_rope.push(suffix_chunk);
        }

        suffix_rope.append(chunks_cursor.suffix().into());

        prefix_rope.pair(suffix_rope)
    }

    fn assign_joint(&mut self, mut prefix_rope: Self, suffix_rope: Self) {
        prefix_rope.append(suffix_rope);
        self.assign(prefix_rope);
    }

    pub fn insert(&mut self, index: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        let (mut prefix_rope, suffix_rope) = self.split(index);

        prefix_rope.push_extended_graphemes(text);
        self.assign_joint(prefix_rope, suffix_rope);
    }

    // NOTE:
    // - this calls split_chunks_at twice on the original rope (before mutation)
    // - this is safe because we only assign to self.chunks at the very end. the sumtree is persistent/immutable under
    //   the hood (arc-based structural sharing), so both splits read from the same stable snapshot
    pub fn delete(&mut self, indices: Range<usize>) {
        if indices.is_empty() {
            return;
        }

        let (prefix_rope, _suffix_rope) = self.split(indices.start);
        let (_prefix_rope, suffix_rope) = self.split(indices.end);

        self.assign_joint(prefix_rope, suffix_rope);
    }

    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        let (mut prefix_rope, _suffix) = self.split(range.start);
        let (_prefix, suffix_rope) = self.split(range.end);

        prefix_rope.push_extended_graphemes(text);
        self.assign_joint(prefix_rope, suffix_rope);
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
        let mut rope = Self::empty();

        for string in iter {
            rope.push_extended_graphemes(string);
        }

        rope
    }
}
