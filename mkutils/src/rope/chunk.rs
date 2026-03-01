use crate::{
    rope::{
        distance::{Distance, NumExtendedGraphemes, NumNewlines},
        extended_grapheme_iter::ExtendedGraphemeIter,
    },
    utils::Utils,
};
use arrayvec::{ArrayString, ArrayVec, CapacityError};
use std::ops::Range;
use zed_sum_tree::{Item, Summary};

// NOTE: [Self: Clone] required for [Self: Item]
#[derive(Clone, Default)]
pub struct Chunk {
    string: ArrayString<{ Self::CAPACITY }>,
    extended_grapheme_byte_offset_intervals: ArrayVec<Range<NumExtendedGraphemes>, { Self::CAPACITY }>,
    newline_extended_grapheme_offsets: ArrayVec<NumExtendedGraphemes, { Self::CAPACITY }>,
}

impl Chunk {
    const CAPACITY: usize = 1024;

    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn num_extended_graphemes(&self) -> NumExtendedGraphemes {
        NumExtendedGraphemes::new(self.extended_grapheme_byte_offset_intervals.len())
    }

    #[must_use]
    pub const fn num_newlines(&self) -> NumNewlines {
        NumNewlines::new(self.newline_extended_grapheme_offsets.len())
    }

    #[must_use]
    pub const fn length(&self) -> Distance {
        Distance::new(self.num_newlines(), self.num_extended_graphemes())
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
    pub fn extended_grapheme_byte_offset_intervals(&self) -> &[Range<NumExtendedGraphemes>] {
        &self.extended_grapheme_byte_offset_intervals
    }

    #[must_use]
    pub fn newline_extended_grapheme_offsets_geq(
        &self,
        extended_grapheme_offset: NumExtendedGraphemes,
    ) -> &[NumExtendedGraphemes] {
        // NOTE: [index] is the index of the first [newline_extended_grapheme_offset] that is greater than or equal to
        // [extended_grapheme_offset]
        let index = self
            .newline_extended_grapheme_offsets
            .partition_point(|newline_extended_grapheme_offset| {
                newline_extended_grapheme_offset < &extended_grapheme_offset
            });

        &self.newline_extended_grapheme_offsets[index..]
    }

    // TODO:
    // - it's potentially the case that the last codepoint in self and the argument extended_grapheme don't have a
    //   proper grapheme boundary; in that case, will need to repair the join boundary, without pushing to
    //   [self.extended_grapheme_byte_offset_intervals]; see chatgpt
    //   - consider the strings "foo" (split into ["f", "o", "o"]) and "bar" (split into ["b", "a", "r"]) and assume the
    //     trailing "o" and the leading "b" don't have a proper boundary
    //   - then after pushing the trailing "o" and the leading "b" i should repair chunk so that the contained graphemes
    //     are ["f", "o", "ob", "a", "r"]
    //   - silly example, but could be the case if i'm reading strings piece by piece from a large file and the reader
    //     splits on code points?
    //   - in that case, should probably just have a note to make sure that [rope.push_str()] is only called with
    //     strings not split between extended graphemes
    //   - (***) could have my own rope builder that maintains a grapheme cursor(?) to ensure this doesn't happen
    pub fn try_push_extended_grapheme<'a>(&mut self, extended_grapheme: &'a str) -> Result<(), CapacityError<&'a str>> {
        let extended_grapheme_byte_offset_interval_begin = self.string.len().convert::<NumExtendedGraphemes>();
        let extended_grapheme_offset = self.num_extended_graphemes();

        // NOTE:
        // - [self.string] will always be weakly longer than [self.extended_grapheme_byte_offset_intervals] (with
        //   equality only when each extended grapheme is a single byte) and because we only push to
        //   [self.extended_grapheme_byte_offset_intervals] after successfully pushing to [self.string], we will never
        //   panic
        // - same holds for [self.newline_extended_grapheme_offsets]
        self.string.try_push_str(extended_grapheme)?;
        extended_grapheme_byte_offset_interval_begin
            .range_from_len(extended_grapheme.len().into())
            .push_to(&mut self.extended_grapheme_byte_offset_intervals);

        if extended_grapheme.is_newline() {
            self.newline_extended_grapheme_offsets.push(extended_grapheme_offset);
        }

        ().ok()
    }
}

impl Item for Chunk {
    type Summary = Distance;

    fn summary(&self, _context: <Self::Summary as Summary>::Context<'_>) -> Self::Summary {
        self.length()
    }
}
