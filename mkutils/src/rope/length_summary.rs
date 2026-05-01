use derive_more::Constructor;
use mkutils_macros::ConstAssoc;
use num::traits::SaturatingAdd;
use std::ops::Add;
use zed_sum_tree::ContextLessSummary;

macro_rules! dimension_type_impls {
    ($dimension_type:ident, $dimension_field:ident) => {
        #[derive(
            ::derive_more::Add,
            ::derive_more::Constructor,
            ::derive_more::From,
            ::derive_more::Sub,
            ::mkutils_macros::ConstAssoc,
            ::mkutils_macros::SaturatingAdd,
            ::mkutils_macros::SaturatingSub,
            ::std::cmp::Eq,
            ::std::cmp::PartialEq,
            ::std::cmp::PartialOrd,
            ::std::cmp::Ord,
            ::std::clone::Clone,
            ::std::marker::Copy,
        )]
        #[const_assoc(pub ZERO: Self = Self::new(0))]
        #[const_assoc(pub ONE: Self = Self::new(1))]
        pub struct $dimension_type(usize);

        impl $dimension_type {
            pub fn get(&self) -> usize {
                self.0
            }
        }

        impl<'a> ::zed_sum_tree::Dimension<'a, LengthSummary> for $dimension_type {
            fn zero(_context: <LengthSummary as ::zed_sum_tree::Summary>::Context<'a>) -> Self {
                Self::ZERO
            }

            fn add_summary(
                &mut self,
                length_summary: &'a LengthSummary,
                _context: <LengthSummary as ::zed_sum_tree::Summary>::Context<'a>,
            ) {
                crate::utils::Utils::saturating_add_assign(&mut self, length_summary.$dimension_field)
            }
        }
    };
}

#[derive(Clone, ConstAssoc, Constructor)]
#[const_assoc(pub ZERO: Self = Self::new(0, 0, 0, 0, 0))]
pub struct LengthSummary {
    pub newlines: usize,
    pub extended_graphemes: usize,
    pub first_line: usize,
    pub last_line: usize,
    pub max_line: usize,
}

impl LengthSummary {
    pub fn from_extended_grapheme(extended_grapheme: &str) -> Self {
        let (newlines, first_line, last_line) = if extended_grapheme.is_newline() {
            (1, 1, 0)
        } else {
            (0, 1, 1)
        };
        let max_line = first_line.max(last_line);

        Self::new(newlines, 1, first_line, last_line, max_line)
    }
}

impl Add<Self> for LengthSummary {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let newlines = self.newlines.saturating_add(rhs.newlines);
        let extended_graphemes = self.extended_graphemes.saturating_add(rhs.extended_graphemes);
        let first_line = if self.newlines.is_zero() {
            self.last_line.saturating_add(&rhs.first_line);
        } else {
            self.first_line
        };
        let last_line = if rhs.newlines.is_zero() {
            self.last_line.saturating_add(&rhs.first_line);
        } else {
            rhs.last_line
        };
        let max_line = self.max_line.max(rhs.max_line).max(self.first_line).max(self.last_line);

        Self::new(newlines, extended_graphemes, first_line, last_line, max_line)
    }
}

impl SaturatingAdd for LengthSummary {
    fn saturating_add(&self, other: &Self) -> Self {
        self.add(&other)
    }
}

impl ContextLessSummary for LengthSummary {
    fn zero() -> Self {
        Self::ZERO
    }

    fn add_summary(&mut self, other: &Self) {
        self.add(other).assign_to(self)
    }
}

dimension_type_impls!(LengthNewlines, newlines);
dimension_type_impls!(LengthExtendedGraphemes, extended_graphemes);
