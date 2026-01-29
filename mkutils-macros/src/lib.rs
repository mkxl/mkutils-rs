mod context;
mod default;
mod from_chain;
mod set_variant;
mod type_assoc;
mod utils;

use crate::{default::Default, from_chain::FromChain, set_variant::SetVariant, type_assoc::TypeAssoc};
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
/// adds
///
/// ```rust
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
/// adds
///
/// ```rust
/// impl Foo for MyStruct {
///     type Item = Vec<u8>;
/// }
/// ```
#[proc_macro_derive(TypeAssoc, attributes(type_assoc))]
pub fn type_assoc(input_token_stream: TokenStream) -> TokenStream {
    TypeAssoc::derive(input_token_stream)
}

/// Implements `Default` for a struct, using `Default::default()` for each field
/// unless a `#[default(...)]` attribute provides a custom expression.
///
///
/// # Example
///
/// ```rust
/// #[derive(Default)]
/// struct MyStruct {
///     name: String,
///     #[default(42)]
///     count: i32,
///     #[default(vec![1, 2, 3])]
///     items: Vec<i32>,
/// }
/// ```
///
/// adds
///
/// ```rust
/// impl Default for MyStruct {
///     fn default() -> Self {
///         Self {
///             name: ::core::default::Default::default(),
///             count: 42,
///             items: vec![1, 2, 3],
///         }
///     }
/// }
/// ```
#[proc_macro_derive(Default, attributes(default))]
pub fn default(input_token_stream: TokenStream) -> TokenStream {
    Default::derive(input_token_stream)
}

///
/// # Example
///
/// ```rust
/// #[derive(SetVariant)]
/// enum MyEnum {
///   Foo,
///   Bar,
///   Baz(String),
/// }
/// ```
///
/// adds
///
/// ```rust
/// impl MyEnum {
///   pub fn set_foo(&mut self) -> &mut Self {
///     *self = Self::Foo;
///
///     self
///   }
///
///   pub fn set_bar(&mut self) -> &mut Self {
///     *self = Self::Bar;
///
///     self
///   }
/// }
///
/// ```
#[proc_macro_derive(SetVariant)]
pub fn set_variant(input_token_stream: TokenStream) -> TokenStream {
    SetVariant::derive(input_token_stream)
}
