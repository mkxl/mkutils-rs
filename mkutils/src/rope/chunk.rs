use crate::{
    rope::{
        chunk_summary::{ChunkSummary, Length, LengthExtendedGraphemes, LineLengthSummary},
        extended_grapheme_iter::ExtendedGraphemeIter,
    },
    utils::Utils,
};
use arrayvec::{ArrayString, ArrayVec, CapacityError};
use num::{Zero, traits::SaturatingSub};
use std::{ops::Range, str::FromStr};
use zed_sum_tree::{Item, Summary};

// NOTE: [Self: Clone] required for [Self: Item]
#[derive(Clone, Default)]
pub struct Chunk {
    string: ArrayString<{ Self::CAPACITY }>,
    extended_grapheme_byte_index_intervals: ArrayVec<Range<LengthExtendedGraphemes>, { Self::CAPACITY }>,
    newline_extended_grapheme_offsets: ArrayVec<LengthExtendedGraphemes, { Self::CAPACITY }>,
}

impl Chunk {
    const CAPACITY: usize = 256;

    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> Length {
        Length::new(
            self.newline_extended_grapheme_offsets.len().into(),
            self.extended_grapheme_byte_index_intervals.len().into(),
        )
    }

    #[must_use]
    pub const fn extended_grapheme_iter(&self) -> ExtendedGraphemeIter<'_> {
        ExtendedGraphemeIter::new(self)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    #[must_use]
    pub fn extended_grapheme_byte_index_intervals(&self) -> &[Range<LengthExtendedGraphemes>] {
        &self.extended_grapheme_byte_index_intervals
    }

    #[must_use]
    pub fn newline_extended_grapheme_offsets_geq(
        &self,
        extended_grapheme_offset: LengthExtendedGraphemes,
    ) -> &[LengthExtendedGraphemes] {
        // NOTE: [index] is the index of the first [newline_extended_grapheme_offset] that is greater than or equal to
        // [extended_grapheme_offset]
        let index = self
            .newline_extended_grapheme_offsets
            .partition_point(|newline_extended_grapheme_offset| {
                newline_extended_grapheme_offset < &extended_grapheme_offset
            });

        &self.newline_extended_grapheme_offsets[index..]
    }

    pub fn try_push_extended_grapheme<'a>(
        &mut self,
        extended_grapheme: &'a str,
    ) -> Result<&mut Self, CapacityError<&'a str>> {
        let extended_grapheme_byte_index_interval_begin = self.string.len().convert::<LengthExtendedGraphemes>();
        let extended_grapheme_offset = self.len().extended_graphemes();

        // NOTE:
        // - [self.string] will always be weakly longer than [self.extended_grapheme_byte_index_intervals] (with
        //   equality only when each extended grapheme is a single byte) and because we only push to
        //   [self.extended_grapheme_byte_index_intervals] after successfully pushing to [self.string], we will never
        //   panic
        // - same holds for [self.newline_extended_grapheme_offsets]
        self.string.try_push_str(extended_grapheme)?;
        extended_grapheme_byte_index_interval_begin
            .range_from_len(extended_grapheme.len().into())
            .push_to(&mut self.extended_grapheme_byte_index_intervals);

        if extended_grapheme.is_newline() {
            self.newline_extended_grapheme_offsets.push(extended_grapheme_offset);
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
    pub fn line_lengths(&self) -> LineLengthSummary {
        let length = self.len().extended_graphemes();
        let first_line_length = self
            .newline_extended_grapheme_offsets
            .first()
            .copied()
            .unwrap_or(length);
        let last_line_length =
            if let Some(last_newline_extended_grapheme_offset) = self.newline_extended_grapheme_offsets.last() {
                length
                    .saturating_sub(last_newline_extended_grapheme_offset)
                    .decremented()
            } else {
                length
            };
        let mut max_line_length = first_line_length.max(last_line_length);

        for window in self.newline_extended_grapheme_offsets.windows(2) {
            let line_length = window[1].saturating_sub(&window[0]).decremented();

            max_line_length.max_assign(line_length);
        }

        LineLengthSummary::new(first_line_length, last_line_length, max_line_length)
    }

    #[must_use]
    pub fn chunk_summary(&self) -> ChunkSummary {
        ChunkSummary::new(self.len(), self.line_lengths())
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn split(&self, extended_grapheme_offset: LengthExtendedGraphemes) -> (Self, Self) {
        if extended_grapheme_offset.is_zero() {
            return Self::empty().pair(self.clone());
        }

        if self.len().extended_graphemes() <= extended_grapheme_offset {
            return self.clone().pair(Self::empty());
        }

        let extended_grapheme_offset = extended_grapheme_offset.convert::<usize>();
        let extended_grapheme_byte_index = self.extended_grapheme_byte_index_intervals[extended_grapheme_offset]
            .start
            .convert::<usize>();
        let (left_str, right_str) = self.as_str().split_at(extended_grapheme_byte_index);
        let left_chunk = left_str.parse::<Self>().unwrap();
        let right_chunk = right_str.parse::<Self>().unwrap();

        left_chunk.pair(right_chunk)
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
    type Summary = ChunkSummary;

    fn summary(&self, _context: <Self::Summary as Summary>::Context<'_>) -> Self::Summary {
        self.chunk_summary()
    }
}
