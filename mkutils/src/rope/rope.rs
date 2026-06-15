use crate::{
    geometry::{PointIsize, PointUsize},
    rope::{
        atoms::{Atoms, ExtendedGraphemeDimensions},
        chunk::Chunk,
        lines::Lines,
        text_summary::{DirectedTextSummary, Length, LengthBytes, LengthExtendedGraphemes, LineLengthSet, TextSummary},
    },
    saturating_add_signed::SaturatingAddSigned,
    utils::Utils,
};
use derive_more::{Constructor, From, Into};
use num::traits::{ConstZero, SaturatingAdd};
use std::{
    borrow::Cow,
    fmt::{Display, Error as FmtError, Formatter},
    iter::Flatten,
    ops::Range,
    option::IntoIter as OptionIntoIter,
};
use tree_sitter::{Node, Point};
use tree_sitter_highlight::ChunkedSource;
use zed_sum_tree::{Bias, Cursor, Dimension, Dimensions, Item, SumTree, Summary};

#[derive(Clone, Constructor, Debug)]
pub struct LineInfo {
    pub begin: TextSummary,
    pub length: TextSummary,
    pub end: TextSummary,
}

#[derive(Debug, Default, From, Into)]
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
    pub fn len_bytes(&self) -> usize {
        self.text_summary().length.bytes
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
            let mut should_push_new_chunk = self.chunk_sum_tree.is_empty();
            let update_last = |last_chunk: &mut Chunk| match last_chunk.try_push_extended_graphemes(extended_graphemes)
            {
                Ok(_last_chunk) => extended_graphemes.assign(""),
                Err(remaining_extended_graphemes) => {
                    extended_graphemes.assign(remaining_extended_graphemes);
                    should_push_new_chunk.set_true();
                }
            };

            self.chunk_sum_tree.update_last(update_last, Self::CONTEXT);

            if should_push_new_chunk {
                self.push(Chunk::new());
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
            return prefix_rope.pair(Self::new());
        };

        let chunk_extended_grapheme_offset = index.saturating_sub(prefix_rope_len_extended_graphemes);
        let (prefix_chunk, suffix_chunk) = chunk.split(chunk_extended_grapheme_offset);
        let mut suffix_rope = Self::new();

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

    #[must_use]
    pub fn point_from_index(&self, extended_grapheme_index: usize) -> PointUsize {
        let extended_grapheme_index = extended_grapheme_index.min(self.len_extended_graphemes());
        let mut chunk_cursor = self.chunk_sum_tree.cursor::<ExtendedGraphemeDimensions>(Self::CONTEXT);
        let extended_grapheme_index_dim = extended_grapheme_index.convert::<LengthExtendedGraphemes>();

        chunk_cursor.seek(&extended_grapheme_index_dim, Self::BIAS);

        let Dimensions(_extended_grapheme_index, offset, _unit) = chunk_cursor.start();
        let mut offset = offset.clone();
        let chunk_extended_grapheme_index = extended_grapheme_index.saturating_sub(offset.length.extended_graphemes);

        if let Some(chunk) = chunk_cursor.item() {
            chunk
                .offset(chunk_extended_grapheme_index)
                .saturating_add_assign_to(offset.ref_mut());
        }

        PointUsize::new(offset.line_lengths.last, offset.length.newlines)
    }

    #[must_use]
    pub fn index_from_point(&self, point: PointUsize) -> usize {
        let mut atoms = self.atoms_at_line(point.y);

        atoms.advance_within_line(point.x).into_ok_err();

        atoms.offset().length.extended_graphemes
    }

    #[must_use]
    pub fn clamp_index(&self, index: usize) -> usize {
        self.len_extended_graphemes().min(index)
    }

    fn distance_between(&self, begin_index: usize, end_index: usize) -> TextSummary {
        let begin_index = self.clamp_index(begin_index);
        let end_index = self.clamp_index(end_index);

        if end_index <= begin_index {
            return TextSummary::ZERO;
        }

        let len_extended_graphemes = end_index.saturating_sub(begin_index);
        let (_prefix_rope, subrope) = self.split(begin_index);
        let (subrope, _suffix_rope) = subrope.split(len_extended_graphemes);

        subrope.text_summary().clone()
    }

    #[must_use]
    pub fn translated(&self, begin_index: usize, delta: &PointIsize) -> DirectedTextSummary {
        let begin_point = self.point_from_index(begin_index);
        let begin_index = self.clamp_index(begin_index);
        let end_point = begin_point.saturating_add_signed(delta);
        let end_index = self.index_from_point(end_point);
        let (is_forward, begin_index, end_index) = begin_index.sorted(end_index);
        let text_summary = self.distance_between(begin_index, end_index);

        DirectedTextSummary::new(is_forward, text_summary)
    }

    #[must_use]
    pub fn line_info(&self, line_index: usize) -> Option<LineInfo> {
        if self.len_newlines() < line_index {
            return None;
        }

        let mut atoms = self.atoms_at_line(line_index);
        let begin = atoms.offset().clone();
        let length = atoms.advance_to_start_of_next_line().into_ok_err();
        let end = begin.saturating_add(&length);
        let line_info = LineInfo::new(begin, length, end);

        line_info.some()
    }

    fn get_chunks_cursor<'r, T: Dimension<'r, TextSummary> + Ord>(
        &'r self,
        offset: impl Into<T>,
    ) -> Cursor<'r, 'static, Chunk, T> {
        let mut chunks_cursor = self.chunk_sum_tree.cursor::<T>(Self::CONTEXT);

        chunks_cursor.seek(&offset.into(), Self::BIAS);

        chunks_cursor
    }

    fn get_bytes_at_indices(
        &self,
        within_rope_byte_indices: Range<usize>,
    ) -> Flatten<OptionIntoIter<BytesAtIndices<'_>>> {
        if within_rope_byte_indices.is_empty() {
            None
        } else {
            BytesAtIndices::new(self, within_rope_byte_indices).some()
        }
        .into_iter_flatten()
    }
}

pub struct BytesAtIndices<'r> {
    chunks_cursor: Cursor<'r, 'static, Chunk, LengthBytes>,
    within_rope_byte_indices: Range<usize>,
}

impl<'r> BytesAtIndices<'r> {
    fn new(rope: &'r Rope, within_rope_byte_indices: Range<usize>) -> Self {
        let chunks_cursor = rope.get_chunks_cursor::<LengthBytes>(within_rope_byte_indices.start);

        Self {
            chunks_cursor,
            within_rope_byte_indices,
        }
    }
}

impl<'r> Iterator for BytesAtIndices<'r> {
    type Item = &'r [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.chunks_cursor.item()?;
        let within_rope_byte_index_of_chunk_begin = self.chunks_cursor.start().get();

        if self.within_rope_byte_indices.end <= within_rope_byte_index_of_chunk_begin {
            return None;
        }

        let within_chunk_byte_index_of_chunk_begin = self
            .within_rope_byte_indices
            .start
            .saturating_sub(within_rope_byte_index_of_chunk_begin);
        let within_chunk_byte_index_of_chunk_end = self
            .within_rope_byte_indices
            .end
            .saturating_sub(within_rope_byte_index_of_chunk_begin);
        let within_chunk_byte_indices = within_chunk_byte_index_of_chunk_begin..within_chunk_byte_index_of_chunk_end;
        let bytes = chunk.as_bytes().clamped_index(within_chunk_byte_indices);

        self.chunks_cursor.next();

        bytes.some()
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

impl<'r> ChunkedSource<'r> for &'r Rope {
    type Chunk = &'r [u8];
    type Chunks = Flatten<OptionIntoIter<BytesAtIndices<'r>>>;

    fn len(&self) -> usize {
        self.len_bytes()
    }

    fn chunk_at(&mut self, within_rope_byte_index: usize, _position: Point) -> Self::Chunk {
        self.get_bytes_at_indices(within_rope_byte_index..self.len_bytes())
            .next()
            .unwrap_or_default()
    }

    fn chunks_for_node(&mut self, node: Node) -> Self::Chunks {
        self.get_bytes_at_indices(node.byte_range())
    }

    #[allow(clippy::unused_peekable)]
    fn text_for_range(&self, within_rope_byte_indices: Range<usize>) -> Cow<'r, [u8]> {
        let mut bytes_iter = self.get_bytes_at_indices(within_rope_byte_indices);
        let Some(bytes) = bytes_iter.next() else { return Cow::default() };
        let mut bytes_iter = bytes_iter.peekable();

        if bytes_iter.is_empty() {
            return bytes.to_cow_borrowed();
        }

        let mut bytes = Vec::from(bytes);

        bytes.extend(bytes_iter.flatten());

        bytes.into_cow_owned()
    }
}
