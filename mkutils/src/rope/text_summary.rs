use crate::utils::Utils;
use derive_more::{Add, Constructor};
use mkutils_macros::{ConstAssoc, SaturatingAdd as MkutilsSaturatingAdd};
use num::{
    Zero,
    traits::{ConstOne, ConstZero, SaturatingAdd},
};
use std::ops::Add;
use zed_sum_tree::ContextLessSummary;

macro_rules! dimension_type_impls {
    ($dimension_type:ident, $dimension_field:ident) => {
        #[derive(
            ::derive_more::Add,
            ::derive_more::Constructor,
            ::derive_more::From,
            ::derive_more::Into,
            ::mkutils_macros::ConstAssoc,
            ::mkutils_macros::SaturatingAdd,
            ::std::clone::Clone,
            ::std::cmp::Eq,
            ::std::cmp::Ord,
            ::std::cmp::PartialEq,
            ::std::cmp::PartialOrd,
            ::std::marker::Copy,
        )]
        #[const_assoc(pub ZERO: Self = Self::new(0))]
        // #[const_assoc(pub ONE: Self = Self::new(1))]
        pub struct $dimension_type(usize);

        impl $dimension_type {
            pub const fn get(&self) -> usize {
                self.0
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
                crate::utils::Utils::saturating_add_assign(&mut self.0, &text_summary.length.$dimension_field)
            }
        }
    };
}

#[derive(Add, Clone, ConstAssoc, Constructor, MkutilsSaturatingAdd)]
#[const_assoc(pub ZERO: Self = Self::new(0, 0))]
pub struct Length {
    pub newlines: usize,
    pub extended_graphemes: usize,
}

#[derive(Clone, ConstAssoc, Constructor)]
#[const_assoc(pub ZERO: Self = Self::uniform(0))]
pub struct LineLengthSet {
    pub first: usize,
    pub last: usize,
    pub max: usize,
}

impl LineLengthSet {
    pub const fn uniform(len: usize) -> Self {
        Self::new(len, len, len)
    }
}

#[derive(Clone, ConstAssoc, Constructor)]
#[const_assoc(pub ZERO: Self = Self::new(Length::ZERO, LineLengthSet::ZERO))]
pub struct TextSummary {
    pub length: Length,
    pub line_lengths: LineLengthSet,
}

impl TextSummary {
    #[must_use]
    pub fn from_extended_grapheme(extended_grapheme: &str) -> Self {
        let (newlines, first, last) = if extended_grapheme.is_newline() {
            (usize::ONE, usize::ONE, usize::ZERO)
        } else {
            (usize::ZERO, usize::ONE, usize::ONE)
        };
        let max = first.max(last);
        let length = Length::new(newlines, usize::ONE);
        let line_lengths = LineLengthSet::new(first, last, max);

        Self::new(length, line_lengths)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.length.extended_graphemes.is_zero()
    }

    #[must_use]
    pub fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    fn add_ref(&self, other: &Self) -> Self {
        let length = self.length.saturating_add(&other.length);
        let first = if self.length.newlines.is_zero() {
            self.line_lengths.last.saturating_add(other.line_lengths.first)
        } else {
            self.line_lengths.first
        };
        let last = if other.length.newlines.is_zero() {
            self.line_lengths.last.saturating_add(other.line_lengths.first)
        } else {
            other.line_lengths.last
        };
        let max = crate::max!(self.line_lengths.max, other.line_lengths.max, first, last);
        let line_lengths = LineLengthSet::new(first, last, max);

        Self::new(length, line_lengths)
    }

    #[must_use]
    pub fn add_to(self, other: &mut Self) -> Self {
        other.saturating_add_assign(self.ref_immut());

        self
    }
}

impl Add<Self> for TextSummary {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_ref(&rhs)
    }
}

impl SaturatingAdd for TextSummary {
    fn saturating_add(&self, v: &Self) -> Self {
        self.add_ref(v)
    }
}

impl ContextLessSummary for TextSummary {
    fn zero() -> Self {
        Self::ZERO
    }

    fn add_summary(&mut self, other: &Self) {
        self.saturating_add_assign(other);
    }
}

dimension_type_impls!(LengthNewlines, newlines);
dimension_type_impls!(LengthExtendedGraphemes, extended_graphemes);
