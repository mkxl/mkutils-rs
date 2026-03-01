use crate::{
    rope::{
        chunk::Chunk,
        distance::{Distance, NumExtendedGraphemes, NumNewlines},
    },
    utils::Utils,
};
use num::traits::SaturatingSub;

pub struct ExtendedGraphemeIter<'c> {
    chunk: &'c Chunk,
    chunk_offset: Distance,
}

impl<'c> ExtendedGraphemeIter<'c> {
    const INITIAL_CHUNK_OFFSET: Distance = Distance::ZERO;

    #[must_use]
    pub const fn new(chunk: &'c Chunk) -> Self {
        let chunk_offset = Self::INITIAL_CHUNK_OFFSET;

        Self { chunk, chunk_offset }
    }

    fn newline_extended_grapheme_offsets_geq(&self) -> &'c [NumExtendedGraphemes] {
        self.chunk
            .newline_extended_grapheme_offsets_geq(self.chunk_offset.extended_graphemes)
    }

    pub fn advance_to_start_of_next_line(&mut self) -> Option<Distance> {
        let start_of_next_line_extended_grapheme_offset =
            self.newline_extended_grapheme_offsets_geq().first()?.incremented();

        if self.chunk.num_extended_graphemes() <= start_of_next_line_extended_grapheme_offset {
            return None;
        }

        let num_extended_graphemes_advanced = self
            .chunk_offset
            .extended_graphemes
            .translate_to(start_of_next_line_extended_grapheme_offset);
        let distance_advanced = Distance::new(NumNewlines::ONE, num_extended_graphemes_advanced);

        self.chunk_offset.newlines.increment();

        distance_advanced.some()
    }

    pub fn advance_to_end_of_chunk(&mut self) -> Distance {
        let num_newlines_advanced = self.chunk_offset.newlines.translate_to(self.chunk.num_newlines());
        let num_extended_graphemes_advanced = self
            .chunk_offset
            .extended_graphemes
            .translate_to(self.chunk.num_extended_graphemes());

        Distance::new(num_newlines_advanced, num_extended_graphemes_advanced)
    }

    // NOTE: returns [Ok(...)] if we've advanced the given number of extended graphemes or we've hit the end of a line
    // and [Err(...)] otherwise (we've hit the end of the chunk without advancing the the given number of extended graphemes)
    pub fn advance_within_line(&mut self, num_extended_graphemes: NumExtendedGraphemes) -> Result<Distance, Distance> {
        if let Some(newline_extended_grapheme_offset) = self.newline_extended_grapheme_offsets_geq().first() {
            let start_of_next_line_extended_grapheme_offset = newline_extended_grapheme_offset.incremented();
            let num_extended_graphemes_to_start_of_next_line =
                start_of_next_line_extended_grapheme_offset.saturating_sub(&self.chunk_offset.extended_graphemes);
            let distance = if num_extended_graphemes < num_extended_graphemes_to_start_of_next_line {
                Distance::new(NumNewlines::ZERO, num_extended_graphemes)
            } else {
                Distance::new(NumNewlines::ONE, num_extended_graphemes_to_start_of_next_line)
            };

            self.chunk_offset.saturating_add_assign(&distance);

            distance.ok()
        } else {
            let num_extended_graphemes_to_end_of_chunk = self
                .chunk
                .num_extended_graphemes()
                .saturating_sub(&self.chunk_offset.extended_graphemes);

            if num_extended_graphemes <= num_extended_graphemes_to_end_of_chunk {
                self.chunk_offset
                    .extended_graphemes
                    .saturating_add_assign(&num_extended_graphemes);

                num_extended_graphemes.convert::<Distance>().ok()
            } else {
                self.chunk_offset
                    .extended_graphemes
                    .saturating_add_assign(&num_extended_graphemes_to_end_of_chunk);

                num_extended_graphemes_to_end_of_chunk.convert::<Distance>().err()
            }
        }
    }
}

impl<'r> Iterator for ExtendedGraphemeIter<'r> {
    type Item = &'r str;

    fn next(&mut self) -> Option<Self::Item> {
        let extended_grapheme_offset = self.chunk_offset.extended_graphemes.convert::<usize>();
        let extended_grapheme = self
            .chunk
            .extended_grapheme_byte_offset_intervals()
            .get(extended_grapheme_offset)?
            .clone()
            .map_range(usize::from)
            .index_into(self.chunk.as_str());

        self.chunk_offset.extended_graphemes.increment();

        if extended_grapheme.is_newline() {
            self.chunk_offset.newlines.increment();
        }

        extended_grapheme.some()
    }
}
