use crate::{
    rope::{chunk::Chunk, text_summary::TextSummary},
    utils::Utils,
};
use num::traits::ConstOne;

pub struct ChunkExtendedGraphemeIter<'c> {
    chunk: &'c Chunk,
    offset: TextSummary,
}

impl<'c> ChunkExtendedGraphemeIter<'c> {
    const INITIAL_OFFSET: TextSummary = TextSummary::ZERO;

    #[must_use]
    pub const fn new(chunk: &'c Chunk) -> Self {
        let offset = Self::INITIAL_OFFSET;

        Self { chunk, offset }
    }

    const fn index(&self) -> usize {
        self.offset.length.extended_graphemes
    }

    fn start_of_next_line_index(&self) -> Option<usize> {
        self.chunk
            .newline_indices_geq(self.index())
            .first()?
            .incremented()
            .some()
    }

    fn advance_to(&mut self, index: usize) -> TextSummary {
        self.chunk
            .distance_between(self.index(), index)
            .add_to(self.offset.ref_mut())
    }

    fn advance_forward(&mut self, count: usize) -> TextSummary {
        let index = self.index().saturating_add(count);

        self.advance_to(index)
    }

    #[must_use]
    pub fn get(&self) -> Option<&'c str> {
        self.chunk
            .byte_index_intervals()
            .get(self.offset.length.extended_graphemes)?
            .clone()
            .index_into(self.chunk.as_str())
            .some()
    }

    pub fn advance_to_start_of_next_line(&mut self) -> Option<TextSummary> {
        let start_of_next_line_index = self.start_of_next_line_index()?;

        if self.chunk.len_extended_graphemes() <= start_of_next_line_index {
            return None;
        }

        self.advance_to(start_of_next_line_index).some()
    }

    pub fn advance_to_end_of_chunk(&mut self) -> TextSummary {
        self.advance_to(self.chunk.len_extended_graphemes())
    }

    // NOTE: returns [Ok(...)] if we've advanced the given number of extended graphemes or we've hit the end of a line
    // and [Err(...)] otherwise (we've hit the end of the chunk without advancing the the given number of extended
    // graphemes)
    pub fn advance_within_line(&mut self, count: usize) -> Result<TextSummary, TextSummary> {
        // NOTE-fn-951d43
        let (into_result, max_end_index) = if let Some(start_of_next_line_index) = self.start_of_next_line_index() {
            (Self::into_fn_ptr(Ok), start_of_next_line_index)
        } else {
            (Self::into_fn_ptr(Err), self.chunk.len_extended_graphemes())
        };
        let end_index = self.index().saturating_add(count).min(max_end_index);
        let distance = self.advance_to(end_index);

        into_result(distance)
    }
}

impl<'c> Iterator for ChunkExtendedGraphemeIter<'c> {
    type Item = &'c str;

    fn next(&mut self) -> Option<Self::Item> {
        let extended_grapheme = self.get();

        self.advance_forward(usize::ONE);

        extended_grapheme
    }
}
