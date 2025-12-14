mod context;
mod from_chain;
mod type_assoc;
mod utils;

use crate::{from_chain::FromChain, type_assoc::TypeAssoc};
use proc_macro::TokenStream;

// TODO: add documentation
#[proc_macro_attribute]
pub fn context(args_token_stream: TokenStream, input_token_stream: TokenStream) -> TokenStream {
    crate::context::context(args_token_stream, input_token_stream)
}

/// Implement `::std::convert::From` through a chain of intermediate types.
///
///
/// # Example
///
/// ```rust
/// #[derive(FromChain)]
/// #[from(Foo, Bar, Baz)]
/// struct MyStruct;
/// ```
///
/// expands to
///
/// ```rust
/// struct MyStruct;
///
/// impl From<Foo> for MyStruct {
///     fn from(foo: Foo) -> Self {
///         Self::from(Baz::from(Bar::from(foo)))
///     }
/// }
/// ```
#[proc_macro_derive(FromChain, attributes(from))]
pub fn from_chain(input_token_stream: TokenStream) -> TokenStream {
    FromChain::derive(input_token_stream)
}

/// Implements traits that only have associated types.
///
///
/// # Example
///
/// ```rust
/// #[derive(TypeAssoc)]
/// #[type_assoc(trait = Foo, Item = Vec<u8>)]
/// struct MyStruct;
/// ```
///
/// expands to
///
/// ```rust
/// struct MyStruct;
///
/// impl Foo for MyStruct {
///     type Item = Vec<u8>;
/// }
/// ```
#[proc_macro_derive(TypeAssoc, attributes(type_assoc))]
pub fn type_assoc(input_token_stream: TokenStream) -> TokenStream {
    TypeAssoc::derive(input_token_stream)
}
