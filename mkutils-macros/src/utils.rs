use proc_macro2::Ident;
use syn::{
    Error as SynError, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

macro_rules! declare_cat_type {
    ($name:ident < $($T:ident),+ >) => {
        pub struct $name<$($T),+>($(pub $T),+);

        impl<$($T),+> $name<$($T),+> {
            #[allow(non_snake_case)]
            pub fn into_tuple(self) -> ($($T),+) {
                let Self($($T),+) = self;

                ($($T),+)
            }
        }

        impl<$($T: Parse),+> Parse for $name<$($T),+> {
            fn parse(parse_stream: ParseStream) -> Result<Self, SynError> {
                Ok(Self(
                    $(parse_stream.parse::<$T>()?),+
                ))
            }
        }
    };
}

declare_cat_type!(Cat3<X, Y, Z>);
declare_cat_type!(Cat6<U, V, W, X, Y, Z>);

pub type Assignment<L, R> = Cat3<L, Token![=], R>;
pub type IdentAssignment<T> = Assignment<Ident, T>;
pub type Comma = Token![,];
pub type CommaPunctuated<T> = Punctuated<T, Comma>;
