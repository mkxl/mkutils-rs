use proc_macro2::Ident;
use syn::{
    Error as SynError, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct Cat3<X, Y, Z>(pub X, pub Y, pub Z);

// TODO: use [derive_more::Into]?
impl<X, Y, Z> Cat3<X, Y, Z> {
    pub fn into_tuple(self) -> (X, Y, Z) {
        (self.0, self.1, self.2)
    }
}

impl<X: Parse, Y: Parse, Z: Parse> Parse for Cat3<X, Y, Z> {
    fn parse(parse_stream: ParseStream) -> Result<Self, SynError> {
        Ok(Self(
            parse_stream.parse()?,
            parse_stream.parse()?,
            parse_stream.parse()?,
        ))
    }
}

pub type Assignment<L, R> = Cat3<L, Token![=], R>;
pub type IdentAssignment<T> = Assignment<Ident, T>;
pub type Comma = Token![,];
pub type CommaPunctuated<T> = Punctuated<T, Comma>;
