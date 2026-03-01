use crate::utils::Utils;
use derive_more::{Add, Constructor, From, Into, Mul, Sub};
use getset::CopyGetters;
use mkutils_macros::{FromChain, SaturatingAdd, SaturatingSub};
use zed_sum_tree::ContextLessSummary;

macro_rules! dimension_type_impls {
    ($summary_type:ident, $dimension_type:ident, $dimension_field:ident) => {
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

        impl From<$dimension_type> for Distance {
            fn from(dimension_value: $dimension_type) -> Self {
                let mut distance = Self::default();

                distance.$dimension_field = dimension_value;

                distance
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

        impl<'a> ::zed_sum_tree::Dimension<'a, $summary_type> for $dimension_type {
            fn zero(_context: <$summary_type as ::zed_sum_tree::Summary>::Context<'a>) -> Self {
                Self::ZERO
            }

            fn add_summary(
                &mut self,
                summary: &'a $summary_type,
                _context: <$summary_type as ::zed_sum_tree::Summary>::Context<'a>,
            ) {
                self.saturating_add_assign(&summary.$dimension_field);
            }
        }
    };
}

dimension_type_impls!(Distance, NumNewlines, newlines);
dimension_type_impls!(Distance, NumExtendedGraphemes, extended_graphemes);

#[derive(Add, Clone, Constructor, Copy, Default, SaturatingAdd, SaturatingSub, Sub)]
pub struct Distance {
    pub newlines: NumNewlines,
    pub extended_graphemes: NumExtendedGraphemes,
}

impl Distance {
    pub const ZERO: Self = Self::new(NumNewlines::ZERO, NumExtendedGraphemes::ZERO);

    #[must_use]
    pub fn from_extended_grapheme(extended_grapheme: &str) -> Self {
        Self::new(extended_grapheme.is_newline().into(), NumExtendedGraphemes::ONE)
    }
}

impl ContextLessSummary for Distance {
    fn zero() -> Self {
        Self::ZERO
    }

    fn add_summary(&mut self, other: &Self) {
        self.saturating_add_assign(other);
    }
}
