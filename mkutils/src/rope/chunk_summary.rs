use crate::utils::Utils;
use derive_more::{Add, Constructor, From, Into, Mul, Sub};
use getset::{CopyGetters, MutGetters};
use mkutils_macros::{FromChain, SaturatingAdd, SaturatingSub};
use num::traits::SaturatingAdd;
use zed_sum_tree::ContextLessSummary;

macro_rules! dimension_type_impls {
    ($dimension_type:ident, $dimension_field:ident) => {
        #[derive(
            Add,
            Clone,
            Constructor,
            Copy,
            CopyGetters,
            Debug,
            Default,
            Eq,
            From,
            FromChain,
            Into,
            Mul,
            Ord,
            PartialEq,
            PartialOrd,
            SaturatingAdd,
            SaturatingSub,
            Sub,
        )]
        #[get_copy = "pub"]
        #[from_chain(bool, usize)]
        #[mul(forward)]
        pub struct $dimension_type(usize);

        impl $dimension_type {
            pub const ZERO: Self = Self::new(0);
            pub const ONE: Self = Self::new(1);
        }

        impl From<$dimension_type> for Length {
            fn from(dimension_value: $dimension_type) -> Self {
                let mut length = Self::default();

                length.$dimension_field = dimension_value;

                length
            }
        }

        impl ::num::Zero for $dimension_type {
            fn zero() -> Self {
                Self::ZERO
            }

            fn is_zero(&self) -> bool {
                self == &Self::ZERO
            }
        }

        impl ::num::One for $dimension_type {
            fn one() -> Self {
                Self::ONE
            }
        }

        impl<'a> ::zed_sum_tree::Dimension<'a, ChunkSummary> for $dimension_type {
            fn zero(_context: <ChunkSummary as ::zed_sum_tree::Summary>::Context<'a>) -> Self {
                Self::ZERO
            }

            fn add_summary(
                &mut self,
                chunk_summary: &'a ChunkSummary,
                _context: <ChunkSummary as ::zed_sum_tree::Summary>::Context<'a>,
            ) {
                self.saturating_add_assign(&chunk_summary.len.$dimension_field);
            }
        }
    };
}

#[derive(Add, Clone, Constructor, Copy, CopyGetters, Default, MutGetters, SaturatingAdd, SaturatingSub, Sub)]
#[get_copy = "pub"]
#[get_mut = "pub"]
pub struct Length {
    lines: LengthLines,
    extended_graphemes: LengthExtendedGraphemes,
}

impl Length {
    pub const ZERO: Self = Self::new(LengthLines::ZERO, LengthExtendedGraphemes::ZERO);

    #[must_use]
    pub fn from_extended_grapheme(extended_grapheme: &str) -> Self {
        Self::new(extended_grapheme.is_newline().into(), LengthExtendedGraphemes::ONE)
    }
}

#[derive(Clone, Constructor, Copy, CopyGetters, Default)]
pub struct LineLengthSummary {
    first: LengthExtendedGraphemes,
    last: LengthExtendedGraphemes,
    #[get_copy = "pub"]
    max: LengthExtendedGraphemes,
}

impl LineLengthSummary {
    const fn set_all(&mut self, width: LengthExtendedGraphemes) {
        self.first = width;
        self.last = width;
        self.max = width;
    }
}

// NOTE:
// - [Self: Clone] required for [Self: ContextLessSummary]
#[derive(Clone, Constructor, CopyGetters, Default, MutGetters)]
#[get_copy = "pub"]
pub struct ChunkSummary {
    len: Length,
    line_lengths: LineLengthSummary,
}

impl ContextLessSummary for ChunkSummary {
    fn zero() -> Self {
        Self::default()
    }

    fn add_summary(&mut self, other: &Self) {
        match (self.len.lines.is_positive(), other.len.lines.is_positive()) {
            // NOTE
            // - [other] (single line) is being appended to [self] (single line)
            // - [self.line_lengths.first == self.line_lengths.last == self.line_lengths.max]
            // - [other.line_lengths.first == other.line_lengths.last == other.line_lengths.max]
            (false, false) => {
                let common_width = self.line_lengths.first.saturating_add(&other.line_lengths.first);

                self.line_lengths.set_all(common_width);
            }

            // NOTE:
            // - [other] (single line) is being appended to [self]'s last line
            // - [other.line_lengths.first == other.line_lengths.last == other.line_lengths.max]
            (true, false) => {
                self.line_lengths.last.saturating_add_assign(&other.line_lengths.first);
                self.line_lengths.max.max_assign(self.line_lengths.last);
            }

            // NOTE:
            // - [self] (single line) is being prepended to [other]'s first line
            // - [self.line_lengths.first == self.line_lengths.last == self.line_lengths.max]
            (false, true) => {
                let new_first_width = self.line_lengths.first.saturating_add(&other.line_lengths.first);
                let max_line_length_lower_bound = other.line_lengths.max.max(new_first_width);

                self.line_lengths.first.assign(new_first_width);
                self.line_lengths.last.assign(other.line_lengths.last);
                self.line_lengths.max.max_assign(max_line_length_lower_bound);
            }

            // NOTE:
            // - [self] and [other] are being concatenated
            // - [middle_line_length] corresponds to the width of the concatenation of the last line of [self] and the
            //   first line of [other]
            (true, true) => {
                let middle_line_length = self.line_lengths.last.saturating_add(&other.line_lengths.first);
                let max_line_length_lower_bound = other.line_lengths.max.max(middle_line_length);

                self.line_lengths.last.assign(other.line_lengths.last);
                self.line_lengths.max.max_assign(max_line_length_lower_bound);
            }
        }

        self.len.saturating_add_assign(&other.len);
    }
}

dimension_type_impls!(LengthLines, lines);
dimension_type_impls!(LengthExtendedGraphemes, extended_graphemes);
