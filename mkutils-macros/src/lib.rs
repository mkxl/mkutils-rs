mod basic;
mod const_assoc;
mod constructor;
mod context;
mod default;
mod error;
mod from_chain;
mod set_variant;
mod toggle;
mod tokio_main;
mod type_assoc;
mod utils;

use crate::{
    basic::Basic, const_assoc::ConstAssoc, constructor::Constructor, default::Default, from_chain::FromChain,
    set_variant::SetVariant, toggle::Toggle, type_assoc::TypeAssoc,
};
use proc_macro::TokenStream;

// TODO: add documentation
#[proc_macro_attribute]
pub fn context(attr_args_token_stream: TokenStream, input_token_stream: TokenStream) -> TokenStream {
    crate::context::context(attr_args_token_stream, input_token_stream)
}

/// Implement `::std::convert::From` through a chain of intermediate types.
///
///
/// # Example
///
/// ```rust
/// #[derive(FromChain)]
/// #[from_chain(Foo, Bar, Baz)]
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
#[proc_macro_derive(FromChain, attributes(from_chain))]
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

/// Adds associated constants to a type via an inherent impl block.
///
///
/// # Example
///
/// ```rust
/// #[derive(ConstAssoc)]
/// #[const_assoc(pub MAX_SIZE: usize = 1024)]
/// #[const_assoc(DEFAULT_NAME: &str = "unnamed")]
/// struct MyStruct;
/// ```
///
/// adds
///
/// ```rust
/// impl MyStruct {
///     pub const MAX_SIZE: usize = 1024;
///     const DEFAULT_NAME: &str = "unnamed";
/// }
/// ```
#[proc_macro_derive(ConstAssoc, attributes(const_assoc))]
pub fn const_assoc(input_token_stream: TokenStream) -> TokenStream {
    ConstAssoc::derive(input_token_stream)
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

/// Adds `set_*()` methods for each unit variant on the given enum.
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
/// ```
#[proc_macro_derive(SetVariant)]
pub fn set_variant(input_token_stream: TokenStream) -> TokenStream {
    SetVariant::derive(input_token_stream)
}

/// Adds a `toggled()` method that maps each enum variant to the next unit variant.
///
/// # Example
///
/// ```rust
/// #[derive(Toggle)]
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
///   pub fn toggle(&self) -> Self {
///     match self {
///         Self::Foo => Self::Bar,
///         Self::Bar => Self::Foo,
///         Self::Baz(_string) => Self::Foo,
///     }
///   }
///
///   pub fn toggle(&mut self) -> &mut Self {
///     *self = self.toggled();
///
///     self
///   }
/// }
/// ```
#[proc_macro_derive(Toggle)]
pub fn toggle(input_token_stream: TokenStream) -> TokenStream {
    Toggle::derive(input_token_stream)
}

/// Implements `num::traits::SaturatingAdd` for a struct by delegating to each field.
///
/// # Example
///
/// ```rust
/// #[derive(SaturatingAdd)]
/// struct MyStruct(usize);
/// ```
///
/// adds
///
/// ```rust
/// impl num::traits::SaturatingAdd for MyStruct {
///     fn saturating_add(&self, v: &Self) -> Self {
///         Self(self.0.saturating_add(&v.0))
///     }
/// }
/// ```
#[proc_macro_derive(SaturatingAdd)]
pub fn saturating_add(input_token_stream: TokenStream) -> TokenStream {
    Basic::derive(input_token_stream, "::num::traits::SaturatingAdd", "saturating_add")
}

/// Implements `num::traits::SaturatingSub` for a struct by delegating to each field.
///
/// # Example
///
/// ```rust
/// #[derive(SaturatingSub)]
/// struct MyStruct(usize);
/// ```
///
/// adds
///
/// ```rust
/// impl num::traits::SaturatingSub for MyStruct {
///     fn saturating_sub(&self, v: &Self) -> Self {
///         Self(self.0.saturating_sub(&v.0))
///     }
/// }
/// ```
#[proc_macro_derive(SaturatingSub)]
pub fn saturating_sub(input_token_stream: TokenStream) -> TokenStream {
    Basic::derive(input_token_stream, "::num::traits::SaturatingSub", "saturating_sub")
}

/// Adds a `new()` constructor that accepts each field as a parameter.
/// The method is private by default. Use `#[new("pub")]` or `#[new("pub(crate)")]`
/// to set a custom visibility.
///
/// # Example
///
/// ```rust
/// #[derive(Constructor)]
/// struct MyStruct {
///     name: String,
///     count: i32,
/// }
/// ```
///
/// adds
///
/// ```rust
/// impl MyStruct {
///     fn new(name: String, count: i32) -> Self {
///         Self { name, count }
///     }
/// }
/// ```
///
/// With a visibility attribute:
///
/// ```rust
/// #[derive(Constructor)]
/// #[new(pub(crate))]
/// struct MyStruct(usize);
/// ```
///
/// adds
///
/// ```rust
/// impl MyStruct {
///     pub(crate) fn new(_0: usize) -> Self {
///         Self(_0)
///     }
/// }
/// ```
#[proc_macro_derive(Constructor, attributes(new))]
pub fn constructor(input_token_stream: TokenStream) -> TokenStream {
    Constructor::derive(input_token_stream)
}

/// # Example
///
/// ```rust
/// const THREAD_STACK_SIZE = lits::bytes!("8 MiB");
///
/// #[mkutils_macros::tokio_main(thread_stack_size = THREAD_STACK_SIZE)]
/// fn main() {
///     ...
/// }
/// ```
#[proc_macro_attribute]
pub fn tokio_main(attr_args_token_stream: TokenStream, item_token_stream: TokenStream) -> TokenStream {
    crate::tokio_main::tokio_main(attr_args_token_stream, item_token_stream)
}
