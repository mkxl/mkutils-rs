use crate::{
    rope::{
        chunk::Chunk,
        chunk_summary::{ChunkSummary, Distance, NumExtendedGraphemes, NumNewlines},
        extended_grapheme_iter::ExtendedGraphemeIter,
        line::Line,
        rope::Rope,
    },
    utils::Utils,
};
use getset::CopyGetters;
use mkutils_macros::Constructor;
use zed_sum_tree::{Bias, Cursor, Dimensions};

type Dims = Dimensions<NumNewlines, ChunkSummary>;

#[derive(Constructor, CopyGetters)]
#[get_copy = "pub"]
pub struct Atom<'r> {
    extended_grapheme: &'r str,
    offset: Distance,
}

#[derive(Constructor)]
pub struct Atoms<'r> {
    chunk_extended_grapheme_iter: Option<ExtendedGraphemeIter<'r>>,
    chunk_cursor: Cursor<'r, 'static, Chunk, Dims>,
    rope_offset: Distance,
}

impl<'r> Atoms<'r> {
    const BIAS: Bias = Bias::Left;

    #[must_use]
    pub fn from_rope(rope: &'r Rope, target_line_offset: NumNewlines) -> Self {
        let chunk_sum_tree = rope.sum_tree();
        let mut chunk_cursor = chunk_sum_tree.cursor::<Dims>(Rope::CONTEXT);

        chunk_cursor.seek(&target_line_offset, Self::BIAS);

        let Dimensions(_line_offset, chunk_summary, ()) = chunk_cursor.start();
        let mut rope_offset = chunk_summary.length();

        'outer: while let Some(chunk) = chunk_cursor.next_iter() {
            let mut chunk_extended_grapheme_iter = chunk.extended_grapheme_iter();

            while rope_offset.newlines() != target_line_offset {
                if let Some(distance_advanced) = chunk_extended_grapheme_iter.advance_to_start_of_next_line() {
                    rope_offset.saturating_add_assign(&distance_advanced);
                } else {
                    rope_offset.saturating_add_assign(&chunk_extended_grapheme_iter.advance_to_end_of_chunk());

                    continue 'outer;
                }
            }

            return Self::new(chunk_extended_grapheme_iter.some(), chunk_cursor, rope_offset);
        }

        Self::new(None, chunk_cursor, chunk_sum_tree.summary().length())
    }

    #[must_use]
    pub const fn rope_offset(&self) -> Distance {
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
        mut num_extended_graphemes: NumExtendedGraphemes,
    ) -> Result<Distance, Distance> {
        let mut total_distance_advanced = Distance::ZERO;

        while let Some(chunk_extended_grapheme_iter) = &mut self.chunk_extended_grapheme_iter {
            match chunk_extended_grapheme_iter.advance_within_line(num_extended_graphemes) {
                Ok(distance_advanced) => {
                    self.rope_offset.saturating_add_assign(&distance_advanced);
                    total_distance_advanced.saturating_add_assign(&distance_advanced);
                    num_extended_graphemes.saturating_sub_assign(&distance_advanced.extended_graphemes());

                    return total_distance_advanced.ok();
                }
                Err(distance_advanced) => {
                    self.rope_offset.saturating_add_assign(&distance_advanced);
                    total_distance_advanced.saturating_add_assign(&distance_advanced);
                    num_extended_graphemes.saturating_sub_assign(&distance_advanced.extended_graphemes());
                }
            }

            self.next_chunk();
        }

        total_distance_advanced.err()
    }

    pub fn advance_to_start_of_next_line(&mut self) {
        while let Some(chunk_extended_grapheme_iter) = &mut self.chunk_extended_grapheme_iter {
            if let Some(distance_advanced) = chunk_extended_grapheme_iter.advance_to_start_of_next_line() {
                self.rope_offset.saturating_add_assign(&distance_advanced);

                break;
            }

            self.rope_offset
                .saturating_add_assign(&chunk_extended_grapheme_iter.advance_to_end_of_chunk());
            self.next_chunk();
        }
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
                let distance_advanced = Distance::from_extended_grapheme(extended_grapheme);
                let atom = Atom::new(extended_grapheme, self.rope_offset).some();

                self.rope_offset.saturating_add_assign(&distance_advanced);

                return atom;
            }

            self.next_chunk();
        }

        None
    }
}
