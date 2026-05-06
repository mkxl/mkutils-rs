use crate::{
    rope::{
        chunk::Chunk,
        chunk_extended_grapheme_iter::ChunkExtendedGraphemeIter,
        line::Line,
        rope::Rope,
        text_summary::{LengthNewlines, TextSummary},
    },
    utils::Utils,
};
use derive_more::Constructor;
use zed_sum_tree::{Bias, Cursor, Dimensions};

type Dims = Dimensions<LengthNewlines, TextSummary>;

#[derive(Constructor)]
pub struct Atom<'r> {
    pub extended_grapheme: &'r str,
    pub offset: TextSummary,
}

#[derive(Constructor)]
pub struct Atoms<'r> {
    chunk_extended_grapheme_iter: Option<ChunkExtendedGraphemeIter<'r>>,
    chunk_cursor: Cursor<'r, 'static, Chunk, Dims>,
    offset: TextSummary,
}

impl<'r> Atoms<'r> {
    const BIAS: Bias = Bias::Left;

    #[must_use]
    pub fn from_rope(rope: &'r Rope, line_index: usize) -> Self {
        let chunk_sum_tree = rope.chunk_sum_tree();
        let mut chunk_cursor = chunk_sum_tree.cursor::<Dims>(Rope::CONTEXT);
        let line_index_dim = line_index.convert::<LengthNewlines>();

        chunk_cursor.seek(&line_index_dim, Self::BIAS);

        let Dimensions(_line_index, offset, _unit) = chunk_cursor.start();
        let mut offset = offset.clone();

        'outer: while let Some(chunk) = chunk_cursor.iter_next() {
            let mut chunk_extended_grapheme_iter = chunk.extended_grapheme_iter();

            while offset.length.newlines != line_index {
                if let Some(distance_advanced) = chunk_extended_grapheme_iter.advance_to_start_of_next_line() {
                    offset.saturating_add_assign(&distance_advanced);
                } else {
                    offset.saturating_add_assign(&chunk_extended_grapheme_iter.advance_to_end_of_chunk());

                    continue 'outer;
                }
            }

            return Self::new(chunk_extended_grapheme_iter.some(), chunk_cursor, offset);
        }

        Self::new(None, chunk_cursor, chunk_sum_tree.summary().clone())
    }

    #[must_use]
    pub const fn offset(&self) -> &TextSummary {
        &self.offset
    }

    // NOTE: advance's [self.chunk_extended_grapheme_iter] but does not update [self.offset]
    fn set_next_chunk_extended_grapheme_iter(&mut self) {
        let Some(chunk) = self.chunk_cursor.iter_next() else {
            self.chunk_extended_grapheme_iter = None;

            return;
        };

        self.chunk_extended_grapheme_iter = chunk.extended_grapheme_iter().some();
    }

    pub fn advance_within_line(&mut self, mut count: usize) -> Result<TextSummary, TextSummary> {
        let mut total_distance_advanced = TextSummary::ZERO;

        while let Some(chunk_extended_grapheme_iter) = self.chunk_extended_grapheme_iter.ref_mut() {
            match chunk_extended_grapheme_iter.advance_within_line(count) {
                Ok(distance_advanced) => {
                    distance_advanced
                        .saturating_add_assign_to_both(self.offset.ref_mut(), total_distance_advanced.ref_mut());
                    count.saturating_sub_assign(&distance_advanced.length.extended_graphemes);

                    return total_distance_advanced.ok();
                }
                Err(distance_advanced) => {
                    distance_advanced
                        .saturating_add_assign_to_both(self.offset.ref_mut(), total_distance_advanced.ref_mut());
                    count.saturating_sub_assign(&distance_advanced.length.extended_graphemes);
                    self.set_next_chunk_extended_grapheme_iter();
                }
            }
        }

        total_distance_advanced.err()
    }

    pub fn advance_to_start_of_next_line(&mut self) -> Result<TextSummary, TextSummary> {
        let mut total_distance_advanced = TextSummary::ZERO;

        while let Some(chunk_extended_grapheme_iter) = self.chunk_extended_grapheme_iter.ref_mut() {
            if let Some(distance_advanced) = chunk_extended_grapheme_iter.advance_to_start_of_next_line() {
                distance_advanced
                    .saturating_add_assign_to_both(self.offset.ref_mut(), total_distance_advanced.ref_mut());

                return total_distance_advanced.ok();
            }

            chunk_extended_grapheme_iter
                .advance_to_end_of_chunk()
                .saturating_add_assign_to_both(self.offset.ref_mut(), total_distance_advanced.ref_mut());
            self.set_next_chunk_extended_grapheme_iter();
        }

        total_distance_advanced.err()
    }

    pub const fn line<'a>(&'a mut self) -> Line<'r, 'a> {
        Line::new(self)
    }
}

impl<'r> Iterator for Atoms<'r> {
    type Item = Atom<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(chunk_extended_grapheme_iter) = self.chunk_extended_grapheme_iter.ref_mut() {
            if let Some(extended_grapheme) = chunk_extended_grapheme_iter.next() {
                let distance_advanced = TextSummary::from_extended_grapheme(extended_grapheme);
                let atom = Atom::new(extended_grapheme, self.offset.clone()).some();

                self.offset.saturating_add_assign(&distance_advanced);

                return atom;
            }

            self.set_next_chunk_extended_grapheme_iter();
        }

        None
    }
}
