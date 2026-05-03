use crate::{
    rope2::{
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

    pub fn byte_index_intervals(&self) -> &[Range<usize>] {
        &self.byte_index_intervals
    }

    pub const fn len_newlines(&self) -> usize {
        self.newline_indices.len()
    }

    pub const fn len_extended_graphemes(&self) -> usize {
        self.byte_index_intervals.len()
    }

    pub const fn length(&self) -> Length {
        Length::new(self.len_newlines(), self.len_extended_graphemes())
    }

    pub const fn extended_grapheme_iter(&self) -> ChunkExtendedGraphemeIter<'_> {
        ChunkExtendedGraphemeIter::new(self)
    }

    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    pub fn newline_indices_geq(&self, index: usize) -> &[usize] {
        // NOTE: [geq_newline_index] is the index of the first [newline_index]
        // greater than or equal to [index]
        let geq_newline_index = self.newline_indices.partition_point(index.predicate_lt());

        &self.newline_indices[geq_newline_index..]
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

    pub fn text_summary(&self) -> TextSummary {
        let length = self.length();
        let first = self
            .newline_indices
            .first()
            .copied()
            .unwrap_or(length.extended_graphemes);
        let last = if let Some(last_newline_index) = self.newline_indices.last() {
            length
                .extended_graphemes
                .saturating_sub(last_newline_index.copied())
                .decremented()
        } else {
            length.extended_graphemes
        };
        let mut max = first;

        for [curr_newline_index, next_newline_index] in self.newline_indices.array_windows() {
            let len_line = next_newline_index
                .saturating_sub(curr_newline_index.copied())
                .decremented();

            max.max_assign(len_line);
        }

        let line_lengths = LineLengthSet::new(first, last, max);

        TextSummary::new(length, line_lengths)
    }

    pub fn split(&self, index: usize) -> (Self, Self) {
        if index.is_zero() {
            return Self::empty().pair(self.clone());
        }

        if self.len_extended_graphemes() <= index {
            return self.clone().pair(Self::empty());
        }

        let byte_index = self.byte_index_intervals[index].start;
        let (prefix_str, suffix_str) = self.as_str().split_at(byte_index);
        let prefix_chunk = prefix_str.parse_unchecked::<Self>();
        let suffix_chunk = suffix_str.parse_unchecked::<Self>();

        prefix_chunk.pair(suffix_chunk)
    }

    pub fn distance_between(&self, _begin: usize, _end: usize) -> TextSummary {
        std::todo!()
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
