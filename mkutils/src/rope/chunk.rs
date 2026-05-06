use crate::{
    rope::{
        chunk_extended_grapheme_iter::ChunkExtendedGraphemeIter,
        text_summary::{Length, LineLengthSet, TextSummary},
    },
    utils::Utils,
};
use arrayvec::{ArrayString, ArrayVec, CapacityError};
use num::Zero;
use std::{ops::Range, str::FromStr};
use zed_sum_tree::{Item, Summary};

#[derive(Clone, Default)]
pub struct Chunk {
    string: ArrayString<{ Self::CAPACITY }>,
    byte_index_intervals: ArrayVec<Range<usize>, { Self::CAPACITY }>,
    newline_indices: ArrayVec<usize, { Self::CAPACITY }>,
}

impl Chunk {
    const CAPACITY: usize = 256;

    #[must_use]
    pub const fn empty() -> Self {
        let string = ArrayString::new_const();
        let byte_index_intervals = ArrayVec::new_const();
        let newline_indices = ArrayVec::new_const();

        Self {
            string,
            byte_index_intervals,
            newline_indices,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    #[must_use]
    pub const fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    #[must_use]
    pub fn byte_index_intervals(&self) -> &[Range<usize>] {
        &self.byte_index_intervals
    }

    #[must_use]
    pub const fn len_newlines(&self) -> usize {
        self.newline_indices.len()
    }

    #[must_use]
    pub const fn len_extended_graphemes(&self) -> usize {
        self.byte_index_intervals.len()
    }

    #[must_use]
    pub const fn length(&self) -> Length {
        Length::new(self.len_newlines(), self.len_extended_graphemes())
    }

    #[must_use]
    pub const fn extended_grapheme_iter(&self) -> ChunkExtendedGraphemeIter<'_> {
        ChunkExtendedGraphemeIter::new(self)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    // NOTE: returns the index of the first value in [self.newline_indices] greater than or equal
    // to [index]
    fn newline_index_parition_point(&self, index: usize) -> usize {
        self.newline_indices.partition_point(index.predicate_lt())
    }

    fn newline_indices_between(&self, begin: usize, end: usize) -> &[usize] {
        let begin = self.newline_index_parition_point(begin);
        let end = self.newline_index_parition_point(end);

        &self.newline_indices[begin..end]
    }

    #[must_use]
    pub fn newline_indices_geq(&self, index: usize) -> &[usize] {
        self.newline_indices_between(index, self.len_extended_graphemes())
    }

    pub fn try_push_extended_grapheme<'a>(
        &mut self,
        extended_grapheme: &'a str,
    ) -> Result<&mut Self, CapacityError<&'a str>> {
        let extended_grapheme_byte_indexinterval_begin = self.string.len();
        let index = self.len_extended_graphemes();

        // NOTE:
        // - [self.string] will always be weakly longer than [self.byte_index_intervals] (with
        //   equality only when each extended grapheme is a single byte) and because we only push to
        //   [self.byte_index_intervals] after successfully pushing to [self.string], we will never
        //   panic
        // - same holds for [self.newline_indices]
        self.string.try_push_str(extended_grapheme)?;
        extended_grapheme_byte_indexinterval_begin
            .range_from_len(extended_grapheme.len())
            .push_to(self.byte_index_intervals.ref_mut());

        if extended_grapheme.is_newline() {
            self.newline_indices.push(index);
        }

        self.ok()
    }

    pub fn try_push_extended_graphemes<'a>(&mut self, extended_graphemes: &'a str) -> Result<&mut Self, &'a str> {
        for (byte_index, extended_grapheme) in extended_graphemes.extended_grapheme_and_byte_index_pairs() {
            if self.try_push_extended_grapheme(extended_grapheme).is_err() {
                return extended_graphemes[byte_index..].ref_immut().err();
            }
        }

        self.ok()
    }

    #[must_use]
    pub fn text_summary(&self) -> TextSummary {
        self.distance_between(0, self.len_extended_graphemes())
    }

    // TODO-205a89
    #[must_use]
    pub fn split(&self, index: usize) -> (Self, Self) {
        if index.is_zero() {
            return Self::empty().pair(self.clone());
        }

        if self.len_extended_graphemes() <= index {
            return self.clone().pair(Self::empty());
        }

        // NOTE: okay to parse unchecked as [prefix_chunk] and [suffix_chunk] are both strictly shorter than [self]
        let byte_index = self.byte_index_intervals[index].start;
        let (prefix_str, suffix_str) = self.as_str().split_at(byte_index);
        let prefix_chunk = prefix_str.parse_unchecked::<Self>();
        let suffix_chunk = suffix_str.parse_unchecked::<Self>();

        prefix_chunk.pair(suffix_chunk)
    }

    #[must_use]
    pub fn distance_between(&self, begin: usize, end: usize) -> TextSummary {
        let end = end.min(self.len_extended_graphemes());
        let extended_graphemes = end.saturating_sub(begin);
        let newline_indices = self.newline_indices_between(begin, end);
        let length = Length::new(newline_indices.len(), extended_graphemes);
        let first = if let Some(first_newline_index) = newline_indices.first() {
            first_newline_index.saturating_sub(begin)
        } else {
            extended_graphemes
        };
        let last = if let Some(last_newline_index) = newline_indices.last() {
            end.saturating_sub(last_newline_index.copied()).decremented()
        } else {
            extended_graphemes
        };
        let mut max = first;

        for [curr_newline_index, next_newline_index] in newline_indices.array_windows() {
            next_newline_index
                .saturating_sub(curr_newline_index.copied())
                .decremented()
                .max_assign_to(max.ref_mut());
        }

        let line_lengths = LineLengthSet::new(first, last, max);

        TextSummary::new(length, line_lengths)
    }
}

impl FromStr for Chunk {
    type Err = CapacityError<()>;

    fn from_str(extended_graphemes: &str) -> Result<Self, Self::Err> {
        match Self::empty().try_push_extended_graphemes(extended_graphemes) {
            Ok(chunk) => chunk.mem_take().ok(),
            Err(_remaining_extended_graphemes) => CapacityError::new(()).err(),
        }
    }
}

impl Item for Chunk {
    type Summary = TextSummary;

    fn summary(&self, _context: <Self::Summary as Summary>::Context<'_>) -> Self::Summary {
        self.text_summary()
    }
}
