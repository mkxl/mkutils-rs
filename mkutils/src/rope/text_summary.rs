use crate::utils::Utils;
use derive_more::{Add, Constructor, From, Into, Mul, Sub};
use mkutils_macros::{ConstAssoc, FromChain, SaturatingAdd, SaturatingSub};
use num::{Zero, traits::SaturatingAdd};
use zed_sum_tree::ContextLessSummary;

macro_rules! dimension_type_impls {
    ($dimension_type:ident, $dimension_field:ident) => {
        #[derive(
            Add,
            Clone,
            Constructor,
            ConstAssoc,
            Copy,
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
        #[const_assoc(pub ZERO: Self = Self::new(0))]
        #[const_assoc(pub ONE: Self = Self::new(1))]
        #[from_chain(bool, usize)]
        #[mul(forward)]
        pub struct $dimension_type(usize);

        impl $dimension_type {
            pub const fn get(&self) -> usize {
                self.0
            }
        }

        impl From<$dimension_type> for TextSummary {
            fn from(dimension_value: $dimension_type) -> Self {
                let mut text_summary = Self::ZERO;

                text_summary.$dimension_field = dimension_value;

                text_summary
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

        impl<'a> ::zed_sum_tree::Dimension<'a, TextSummary> for $dimension_type {
            fn zero(_context: <TextSummary as ::zed_sum_tree::Summary>::Context<'a>) -> Self {
                Self::ZERO
            }

            fn add_summary(
                &mut self,
                text_summary: &'a TextSummary,
                _context: <TextSummary as ::zed_sum_tree::Summary>::Context<'a>,
            ) {
                self.saturating_add_assign(&text_summary.$dimension_field);
            }
        }
    };
}

#[derive(Clone, Constructor, ConstAssoc, Copy, Default)]
#[const_assoc(pub ZERO: Self = Self::new(
    LengthNewlines::ZERO,
    LengthExtendedGraphemes::ZERO,
    LengthExtendedGraphemes::ZERO,
    LengthExtendedGraphemes::ZERO,
    LengthExtendedGraphemes::ZERO
))]
pub struct TextSummary {
    len_newlines: LengthNewlines,
    len_extended_graphemes: LengthExtendedGraphemes,
    len_first_line: LengthExtendedGraphemes,
    len_last_line: LengthExtendedGraphemes,
    max_line_len: LengthExtendedGraphemes,
}

impl TextSummary {
    pub fn from_extended_grapheme(extended_grapheme: &str) -> Self {
        let len_extended_graphemes = LengthExtendedGraphemes::ONE;
        let (len_newlines, len_first_line, len_last_line) = if extended_grapheme.is_newline() {
            (
                LengthNewlines::ONE,
                LengthExtendedGraphemes::ONE,
                LengthExtendedGraphemes::ZERO,
            )
        } else {
            (
                LengthNewlines::ZERO,
                LengthExtendedGraphemes::ONE,
                LengthExtendedGraphemes::ONE,
            )
        };
        let max_line_len = len_first_line.max(len_last_line);

        Self {
            len_newlines,
            len_extended_graphemes,
            len_first_line,
            len_last_line,
            max_line_len,
        }
    }

    pub const fn from_len_extended_graphemes_one_line(len_extended_graphemes: LengthExtendedGraphemes) -> Self {
        Self::new(
            LengthNewlines::ZERO,
            len_extended_graphemes,
            len_extended_graphemes,
            len_extended_graphemes,
            len_extended_graphemes,
        )
    }

    fn set_len_first_line(&mut self, len_first_line: LengthExtendedGraphemes) {
        self.len_first_line.assign(len_first_line);
        self.max_line_len.max_assign(len_first_line);
    }

    fn set_len_last_line(&mut self, len_last_line: LengthExtendedGraphemes) {
        self.len_last_line.assign(len_last_line);
        self.max_line_len.max_assign(len_last_line);
    }
}

impl ContextLessSummary for TextSummary {
    fn zero() -> Self {
        Self::ZERO
    }

    fn add_summary(&mut self, other: &Self) {
        let new_len_last_line = if self.len_newlines.is_zero() {
            let len_joined_line = self.len_last_line.saturating_add(&other.len_first_line);

            self.set_len_first_line(len_joined_line);

            self.len_last_line.saturating_add(&other.len_last_line)
        } else {
            other.len_last_line
        };

        self.len_newlines.saturating_add_assign(&other.len_newlines);
        self.len_extended_graphemes
            .saturating_add_assign(&other.len_extended_graphemes);
        self.set_len_last_line(new_len_last_line);
    }
}

dimension_type_impls!(LengthNewlines, len_newlines);
dimension_type_impls!(LengthExtendedGraphemes, len_extended_graphemes);
