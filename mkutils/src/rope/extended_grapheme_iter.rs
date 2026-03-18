use crate::{
    rope::{
        chunk::Chunk,
        chunk_summary::{Length, LengthExtendedGraphemes, LengthLines},
    },
    utils::Utils,
};
use num::traits::SaturatingSub;

pub struct ExtendedGraphemeIter<'c> {
    chunk: &'c Chunk,
    chunk_offset: Length,
}

impl<'c> ExtendedGraphemeIter<'c> {
    const INITIAL_CHUNK_OFFSET: Length = Length::ZERO;

    #[must_use]
    pub const fn new(chunk: &'c Chunk) -> Self {
        let chunk_offset = Self::INITIAL_CHUNK_OFFSET;

        Self { chunk, chunk_offset }
    }

    fn newline_extended_grapheme_offsets_geq(&self) -> &'c [LengthExtendedGraphemes] {
        self.chunk
            .newline_extended_grapheme_offsets_geq(self.chunk_offset.extended_graphemes())
    }

    pub fn advance_to_start_of_next_line(&mut self) -> Option<Length> {
        let start_of_next_line_extended_grapheme_offset =
            self.newline_extended_grapheme_offsets_geq().first()?.incremented();

        if self.chunk.len().extended_graphemes() <= start_of_next_line_extended_grapheme_offset {
            return None;
        }

        let len_extended_graphemes_advanced = self
            .chunk_offset
            .extended_graphemes_mut()
            .translate_to(start_of_next_line_extended_grapheme_offset);
        let length_advanced = Length::new(LengthLines::ONE, len_extended_graphemes_advanced);

        self.chunk_offset.lines_mut().increment();

        length_advanced.some()
    }

    pub fn advance_to_end_of_chunk(&mut self) -> Length {
        let len_lines_advanced = self.chunk_offset.lines_mut().translate_to(self.chunk.len().lines());
        let len_extended_graphemes_advanced = self
            .chunk_offset
            .extended_graphemes_mut()
            .translate_to(self.chunk.len().extended_graphemes());

        Length::new(len_lines_advanced, len_extended_graphemes_advanced)
    }

    // NOTE: returns [Ok(...)] if we've advanced the given number of extended graphemes or we've hit the end of a line
    // and [Err(...)] otherwise (we've hit the end of the chunk without advancing the the given number of extended graphemes)
    pub fn advance_within_line(&mut self, len_extended_graphemes: LengthExtendedGraphemes) -> Result<Length, Length> {
        if let Some(newline_extended_grapheme_offset) = self.newline_extended_grapheme_offsets_geq().first() {
            let start_of_next_line_extended_grapheme_offset = newline_extended_grapheme_offset.incremented();
            let len_extended_graphemes_to_start_of_next_line =
                start_of_next_line_extended_grapheme_offset.saturating_sub(&self.chunk_offset.extended_graphemes());
            let length = if len_extended_graphemes < len_extended_graphemes_to_start_of_next_line {
                Length::new(LengthLines::ZERO, len_extended_graphemes)
            } else {
                Length::new(LengthLines::ONE, len_extended_graphemes_to_start_of_next_line)
            };

            self.chunk_offset.saturating_add_assign(&length);

            length.ok()
        } else {
            let len_extended_graphemes_to_end_of_chunk = self
                .chunk
                .len()
                .extended_graphemes()
                .saturating_sub(&self.chunk_offset.extended_graphemes());

            if len_extended_graphemes <= len_extended_graphemes_to_end_of_chunk {
                self.chunk_offset
                    .extended_graphemes_mut()
                    .saturating_add_assign(&len_extended_graphemes);

                len_extended_graphemes.convert::<Length>().ok()
            } else {
                self.chunk_offset
                    .extended_graphemes_mut()
                    .saturating_add_assign(&len_extended_graphemes_to_end_of_chunk);

                len_extended_graphemes_to_end_of_chunk.convert::<Length>().err()
            }
        }
    }
}

impl<'r> Iterator for ExtendedGraphemeIter<'r> {
    type Item = &'r str;

    fn next(&mut self) -> Option<Self::Item> {
        let extended_grapheme_offset = self.chunk_offset.extended_graphemes().convert::<usize>();
        let extended_grapheme = self
            .chunk
            .extended_grapheme_byte_index_intervals()
            .get(extended_grapheme_offset)?
            .clone()
            .map_range(usize::from)
            .index_into(self.chunk.as_str());

        self.chunk_offset.extended_graphemes_mut().increment();

        if extended_grapheme.is_newline() {
            self.chunk_offset.lines_mut().increment();
        }

        extended_grapheme.some()
    }
}
