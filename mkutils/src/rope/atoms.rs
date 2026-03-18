use crate::{
    rope::{
        chunk::Chunk,
        chunk_summary::{ChunkSummary, Length, LengthExtendedGraphemes, LengthLines},
        extended_grapheme_iter::ExtendedGraphemeIter,
        line::Line,
        rope::Rope,
    },
    utils::Utils,
};
use getset::CopyGetters;
use mkutils_macros::Constructor;
use zed_sum_tree::{Bias, Cursor, Dimensions};

type Dims = Dimensions<LengthLines, ChunkSummary>;

#[derive(Constructor, CopyGetters)]
#[get_copy = "pub"]
pub struct Atom<'r> {
    extended_grapheme: &'r str,
    offset: Length,
}

#[derive(Constructor)]
pub struct Atoms<'r> {
    chunk_extended_grapheme_iter: Option<ExtendedGraphemeIter<'r>>,
    chunk_cursor: Cursor<'r, 'static, Chunk, Dims>,
    rope_offset: Length,
}

impl<'r> Atoms<'r> {
    const BIAS: Bias = Bias::Left;

    #[must_use]
    pub fn from_rope(rope: &'r Rope, target_line_offset: LengthLines) -> Self {
        let chunk_sum_tree = rope.sum_tree();
        let mut chunk_cursor = chunk_sum_tree.cursor::<Dims>(Rope::CONTEXT);

        chunk_cursor.seek(&target_line_offset, Self::BIAS);

        let Dimensions(_line_offset, chunk_summary, ()) = chunk_cursor.start();
        let mut rope_offset = chunk_summary.len();

        'outer: while let Some(chunk) = chunk_cursor.next_iter() {
            let mut chunk_extended_grapheme_iter = chunk.extended_grapheme_iter();

            while rope_offset.lines() != target_line_offset {
                if let Some(length_advanced) = chunk_extended_grapheme_iter.advance_to_start_of_next_line() {
                    rope_offset.saturating_add_assign(&length_advanced);
                } else {
                    rope_offset.saturating_add_assign(&chunk_extended_grapheme_iter.advance_to_end_of_chunk());

                    continue 'outer;
                }
            }

            return Self::new(chunk_extended_grapheme_iter.some(), chunk_cursor, rope_offset);
        }

        Self::new(None, chunk_cursor, chunk_sum_tree.summary().len())
    }

    #[must_use]
    pub const fn rope_offset(&self) -> Length {
        self.rope_offset
    }

    fn next_chunk(&mut self) {
        let Some(chunk) = self.chunk_cursor.next_iter() else {
            self.chunk_extended_grapheme_iter = None;

            return;
        };

        self.chunk_extended_grapheme_iter = chunk.extended_grapheme_iter().some();
    }

    pub fn advance_within_line(
        &mut self,
        mut len_extended_graphemes: LengthExtendedGraphemes,
    ) -> Result<Length, Length> {
        let mut total_length_advanced = Length::ZERO;

        while let Some(chunk_extended_grapheme_iter) = &mut self.chunk_extended_grapheme_iter {
            match chunk_extended_grapheme_iter.advance_within_line(len_extended_graphemes) {
                Ok(length_advanced) => {
                    self.rope_offset.saturating_add_assign(&length_advanced);
                    total_length_advanced.saturating_add_assign(&length_advanced);
                    len_extended_graphemes.saturating_sub_assign(&length_advanced.extended_graphemes());

                    return total_length_advanced.ok();
                }
                Err(length_advanced) => {
                    self.rope_offset.saturating_add_assign(&length_advanced);
                    total_length_advanced.saturating_add_assign(&length_advanced);
                    len_extended_graphemes.saturating_sub_assign(&length_advanced.extended_graphemes());
                }
            }

            self.next_chunk();
        }

        total_length_advanced.err()
    }

    // NOTE: returns [Ok(...)] if we've successfully advanced to the start of the next line and [Err(...)] otherwise
    pub fn advance_to_start_of_next_line(&mut self) -> Result<Length, Length> {
        let mut total_length_advanced = Length::ZERO;

        while let Some(chunk_extended_grapheme_iter) = &mut self.chunk_extended_grapheme_iter {
            if let Some(length_advanced) = chunk_extended_grapheme_iter.advance_to_start_of_next_line() {
                self.rope_offset.saturating_add_assign(&length_advanced);
                total_length_advanced.saturating_add_assign(&length_advanced);

                return total_length_advanced.ok();
            }

            let length_advanced = chunk_extended_grapheme_iter.advance_to_end_of_chunk();

            self.rope_offset.saturating_add_assign(&length_advanced);
            total_length_advanced.saturating_add_assign(&length_advanced);
            self.next_chunk();
        }

        total_length_advanced.err()
    }

    pub const fn line<'a>(&'a mut self) -> Line<'r, 'a> {
        Line::new(self)
    }
}

impl<'r> Iterator for Atoms<'r> {
    type Item = Atom<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(chunk_extended_grapheme_iter) = &mut self.chunk_extended_grapheme_iter {
            if let Some(extended_grapheme) = chunk_extended_grapheme_iter.next() {
                let length_advanced = Length::from_extended_grapheme(extended_grapheme);
                let atom = Atom::new(extended_grapheme, self.rope_offset).some();

                self.rope_offset.saturating_add_assign(&length_advanced);

                return atom;
            }

            self.next_chunk();
        }

        None
    }
}
