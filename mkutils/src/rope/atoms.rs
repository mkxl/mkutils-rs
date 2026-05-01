use crate::{
    rope::{
        chunk::Chunk,
        extended_grapheme_iter::ExtendedGraphemeIter,
        length_summary::{LengthNewlines, LengthSummary},
        line::Line,
        rope::Rope,
    },
    utils::Utils,
};
use getset::CopyGetters;
use mkutils_macros::Constructor;
use zed_sum_tree::{Bias, Cursor, Dimensions};

type Dims = Dimensions<LengthNewlines, LengthSummary>;

#[derive(Constructor, CopyGetters)]
#[get_copy = "pub"]
pub struct Atom<'r> {
    extended_grapheme: &'r str,
    length_summary: LengthSummary,
}

#[derive(Constructor)]
pub struct Atoms<'r> {
    chunk_extended_grapheme_iter: Option<ExtendedGraphemeIter<'r>>,
    chunk_cursor: Cursor<'r, 'static, Chunk, Dims>,
    length_summary: LengthSummary,
}

impl<'r> Atoms<'r> {
    const BIAS: Bias = Bias::Left;

    #[must_use]
    pub fn from_rope(rope: &'r Rope, target_line_offset: usize) -> Self {
        let chunk_sum_tree = rope.sum_tree();
        let mut chunk_cursor = chunk_sum_tree.cursor::<Dims>(Rope::CONTEXT);
        let target_line_offset_dim = target_line_offset.convert::<LengthNewlines>();

        chunk_cursor.seek(&target_line_offset_dim, Self::BIAS);

        let Dimensions(_line_offset, mut length_summary, ()) = chunk_cursor.start();

        'outer: while let Some(chunk) = chunk_cursor.next_iter() {
            let mut chunk_extended_grapheme_iter = chunk.extended_grapheme_iter();

            while length_summary.newlines != target_line_offset {
                if let Some(distance_advanced) = chunk_extended_grapheme_iter.advance_to_start_of_next_line() {
                    length_summary.saturating_add_assign(&distance_advanced);
                } else {
                    length_summary.saturating_add_assign(&chunk_extended_grapheme_iter.advance_to_end_of_chunk());

                    continue 'outer;
                }
            }

            return Self::new(chunk_extended_grapheme_iter.some(), chunk_cursor, length_summary);
        }

        Self::new(None, chunk_cursor, chunk_sum_tree.summary().clone())
    }

    #[must_use]
    pub const fn length_summary(&self) -> LengthSummary {
        self.length_summary
    }

    fn next_chunk(&mut self) {
        let Some(chunk) = self.chunk_cursor.next_iter() else {
            self.chunk_extended_grapheme_iter = None;

            return;
        };

        self.chunk_extended_grapheme_iter = chunk.extended_grapheme_iter().some();
    }

    pub fn advance_within_line(&mut self, mut len_extended_graphemes: usize) -> Result<LengthSummary, LengthSummary> {
        let mut total_distance_advanced = LengthSummary::ZERO;

        while let Some(chunk_extended_grapheme_iter) = &mut self.chunk_extended_grapheme_iter {
            match chunk_extended_grapheme_iter.advance_within_line(len_extended_graphemes) {
                Ok(distance_advanced) => {
                    self.length_summary.saturating_add_assign(&distance_advanced);
                    total_distance_advanced.saturating_add_assign(&distance_advanced);
                    len_extended_graphemes.saturating_sub_assign(&distance_advanced.extended_graphemes);

                    return total_distance_advanced.ok();
                }
                Err(distance_advanced) => {
                    self.length_summary.saturating_add_assign(&distance_advanced);
                    total_distance_advanced.saturating_add_assign(&distance_advanced);
                    len_extended_graphemes.saturating_sub_assign(&distance_advanced.extended_graphemes);
                }
            }

            self.next_chunk();
        }

        total_distance_advanced.err()
    }

    // NOTE: returns [Ok(...)] if we've successfully advanced to the start of the next line and [Err(...)] otherwise
    pub fn advance_to_start_of_next_line(&mut self) -> Result<LengthSummary, LengthSummary> {
        let mut total_distance_advanced = LengthSummary::ZERO;

        while let Some(chunk_extended_grapheme_iter) = &mut self.chunk_extended_grapheme_iter {
            if let Some(distance_advanced) = chunk_extended_grapheme_iter.advance_to_start_of_next_line() {
                self.length_summary.saturating_add_assign(&distance_advanced);
                total_distance_advanced.saturating_add_assign(&distance_advanced);

                return total_distance_advanced.ok();
            }

            let distance_advanced = chunk_extended_grapheme_iter.advance_to_end_of_chunk();

            self.length_summary.saturating_add_assign(&distance_advanced);
            total_distance_advanced.saturating_add_assign(&distance_advanced);
            self.next_chunk();
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
        while let Some(chunk_extended_grapheme_iter) = &mut self.chunk_extended_grapheme_iter {
            if let Some(extended_grapheme) = chunk_extended_grapheme_iter.next() {
                let distance_advanced = LengthSummary::from_extended_grapheme(extended_grapheme);
                let atom = Atom::new(extended_grapheme, self.length_summary).some();

                self.length_summary.saturating_add_assign(&distance_advanced);

                return atom;
            }

            self.next_chunk();
        }

        None
    }
}
