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
/// struct Foo;
///
/// struct Bar;
///
/// struct Baz;
///
/// impl From<Foo> for Bar { fn from(_foo: Foo) -> Self { Self } }
///
/// impl From<Bar> for Baz { fn from(_bar: Bar) -> Self { Self } }
///
/// impl From<Baz> for MyStruct { fn from(_baz: Baz) -> Self { Self } }
///
/// #[derive(mkutils_macros::FromChain)]
/// #[from_chain(Foo, Bar, Baz)]
/// struct MyStruct;
///
/// // adds
/// // ```rust
/// // impl From<Foo> for MyStruct {
/// //     fn from(foo: Foo) -> Self {
/// //         Self::from(Baz::from(Bar::from(foo)))
/// //     }
/// // }
/// // ```
/// // as can be seen in
///
/// let _my_struct: MyStruct = Foo.into();
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
///
/// trait Foo {
///   type Item;
/// }
///
/// #[derive(mkutils_macros::TypeAssoc)]
/// #[type_assoc(impl_trait = Foo, Item = Vec<u8>)]
/// struct MyStruct;
///
/// // adds
/// // ```rust
/// // impl Foo for MyStruct {
/// //     type Item = Vec<u8>;
/// // }
/// // ```
/// // as can be seen in
///
/// fn consume_foo<T: Foo>(value: T) {}
///
/// consume_foo(MyStruct);
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
/// #[derive(mkutils_macros::ConstAssoc)]
/// #[const_assoc(pub MAX_SIZE: usize = 1024)]
/// #[const_assoc(DEFAULT_NAME: &str = "unnamed")]
/// struct MyStruct;
///
/// // adds
/// // ```rust
/// // impl MyStruct {
/// //     pub const MAX_SIZE: usize = 1024;
/// //     const DEFAULT_NAME: &str = "unnamed";
/// // }
/// // ```
/// // as can be seen in
///
/// std::assert_eq!(MyStruct::MAX_SIZE, 1024);
/// std::assert_eq!(MyStruct::DEFAULT_NAME, "unnamed");
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
/// #[derive(mkutils_macros::Default)]
/// struct MyStruct {
///     name: String,
///     #[default(42)]
///     count: i32,
///     #[default(std::vec![1, 2, 3])]
///     items: Vec<i32>,
/// }
///
/// // adds
/// // ```rust
/// // impl Default for MyStruct {
/// //     fn default() -> Self {
/// //         Self {
/// //             name: ::core::default::Default::default(),
/// //             count: 42,
/// //             items: std::vec![1, 2, 3],
/// //         }
/// //     }
/// // }
/// // ```
/// // as can be seen in
///
/// let default = MyStruct::default();
///
/// std::assert_eq!(default.name, "");
/// std::assert_eq!(default.count, 42);
/// std::assert_eq!(default.items, std::vec![1, 2, 3]);
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
/// #[derive(Debug, mkutils_macros::SetVariant, PartialEq)]
/// enum MyEnum {
///   Foo,
///   Bar,
///   Baz(String),
/// }
///
/// // adds
/// // ```rust
/// // impl MyEnum {
/// //   pub fn set_foo(&mut self) -> &mut Self {
/// //     *self = Self::Foo;
/// //
/// //     self
/// //   }
/// //
/// //   pub fn set_bar(&mut self) -> &mut Self {
/// //     *self = Self::Bar;
/// //
/// //     self
/// //   }
/// // }
/// // ```
/// // as can be seen in
///
/// let mut my_enum = MyEnum::Foo;
///
/// my_enum.set_bar();
///
/// std::assert_eq!(my_enum, MyEnum::Bar);
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
/// #[derive(Debug, mkutils_macros::Toggle, PartialEq)]
/// enum MyEnum {
///   Foo,
///   Bar,
///   Baz(String),
/// }
///
/// // adds
/// // ```rust
/// // impl MyEnum {
/// //   pub fn toggle(&self) -> Self {
/// //     match self {
/// //         Self::Foo => Self::Bar,
/// //         Self::Bar => Self::Foo,
/// //         Self::Baz(_string) => Self::Foo,
/// //     }
/// //   }
/// //
/// //   pub fn toggle(&mut self) -> &mut Self {
/// //     *self = self.toggled();
/// //
/// //     self
/// //   }
/// // }
/// // ```
/// // as can be seen in
///
/// std::assert_eq!(MyEnum::Foo.toggled(), MyEnum::Bar);
/// std::assert_eq!(MyEnum::Bar.toggled(), MyEnum::Foo);
/// std::assert_eq!(MyEnum::Baz(String::new()).toggled(), MyEnum::Foo);
/// ```
#[proc_macro_derive(Toggle)]
pub fn toggle(input_token_stream: TokenStream) -> TokenStream {
    Toggle::derive(input_token_stream)
}

/// Implements `num::traits::SaturatingAdd` for a struct by delegating to each field.
/// Supports setting bounds with `#[saturating_add(bound = "T: SomeTrait")]`
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Debug, mkutils_macros::SaturatingAdd, PartialEq)]
/// struct MyStruct(usize, usize);
/// ```
///
/// adds
///
/// ```rust,ignore
/// impl num::traits::SaturatingAdd for MyStruct {
///     fn saturating_add(&self, v: &Self) -> Self {
///         Self(self.0.saturating_add(&v.0), self.1.saturating_add(&v.1))
///     }
/// }
/// ```
///
/// as can be seen in
///
/// ```rust,ignore
/// std::assert_eq!(MyStruct(1, 1).saturating_add(MyStruct(2, 2)), MyStruct(3, 3));
/// ```
#[proc_macro_derive(SaturatingAdd, attributes(saturating_add))]
pub fn saturating_add(input_token_stream: TokenStream) -> TokenStream {
    Basic::derive(
        input_token_stream,
        "::num::traits::SaturatingAdd",
        "saturating_add",
        "Self",
        "saturating_add",
    )
}

/// Implements `num::traits::SaturatingSub` for a struct by delegating to each field.
/// Supports setting bounds with `#[saturating_sub(bound = "T: SomeTrait")]`
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Debug, mkutils_macros::SaturatingSub, PartialEq)]
/// struct MyStruct(usize, usize);
/// ```
///
/// adds
///
/// ```rust,ignore
/// impl num::traits::SaturatingSub for MyStruct {
///     fn saturating_sub(&self, v: &Self) -> Self {
///         Self(self.0.saturating_sub(&v.0), self.1.saturating_sub(&v.1))
///     }
/// }
/// ```
///
/// as can be seen in
///
/// ```rust,ignore
/// std::assert_eq!(MyStruct(1, 1).saturating_sub(MyStruct(2, 2)), MyStruct(0, 0));
/// ```
#[proc_macro_derive(SaturatingSub, attributes(saturating_sub))]
pub fn saturating_sub(input_token_stream: TokenStream) -> TokenStream {
    Basic::derive(
        input_token_stream,
        "::num::traits::SaturatingSub",
        "saturating_sub",
        "Self",
        "saturating_sub",
    )
}

#[allow(clippy::too_long_first_doc_paragraph)]
/// Implements `mkutils::SaturatingAddSigned` for a struct by delegating to each field.
/// Set the `Signed` associated type and bounds with
/// `#[saturating_add_signed(assoc(type Signed = Point<<T as SaturatingAddSigned>::Signed>)), bound = "T: SomeTrait"]`
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Debug, mkutils_macros::SaturatingAddSigned, PartialEq)]
/// #[saturating_add_signed(assoc(type Signed = MyStruct<<T as SaturatingAddSigned>::Signed>))]
/// struct MyStruct<T>(T, T);
/// ```
///
/// adds
///
/// ```rust,ignore
/// impl mkutils::SaturatingAddSigned for MyStruct<T> {
///     fn saturating_add_signed(&self, v: &Other) -> Self {
///         Self(self.0.saturating_add_signed(&v.0), self.1.saturating_add_signed(&v.1))
///     }
/// }
/// ```
///
/// as can be seen in
///
/// ```rust,ignore
/// std::assert_eq!(MyStruct(2, 2).saturating_add_signed(MyStruct(-1, -1)), MyStruct(1, 1));
/// ```
#[proc_macro_derive(SaturatingAddSigned, attributes(saturating_add_signed))]
pub fn saturating_add_signed(input_token_stream: TokenStream) -> TokenStream {
    Basic::derive(
        input_token_stream,
        "::mkutils::SaturatingAddSigned", // NOTE-ee355f
        "saturating_add_signed",
        "Self::Signed",
        "saturating_add_signed",
    )
}

/// Adds a `new()` constructor that accepts each field as a parameter.
/// The method is private by default. Use `#[new("pub")]` or `#[new("pub(crate)")]`
/// to set a custom visibility.
///
/// # Example
///
/// ```rust
/// #[derive(Debug, mkutils_macros::Constructor, PartialEq)]
/// #[new(pub(crate))]
/// struct MyStruct {
///     name: String,
///     count: i32,
/// }
///
/// // adds
/// // ```rust
/// // impl MyStruct {
/// //     pub(crate) fn new(name: String, count: i32) -> Self {
/// //         Self { name, count }
/// //     }
/// // }
/// // ```
/// // as can be seen in
///
/// let my_struct_literal = MyStruct { name: "hello".into(), count: 2 };
/// let my_struct_constructed = MyStruct::new("hello".into(), 2);
///
/// std::assert_eq!(my_struct_literal, my_struct_constructed);
/// ```
#[proc_macro_derive(Constructor, attributes(new))]
pub fn constructor(input_token_stream: TokenStream) -> TokenStream {
    Constructor::derive(input_token_stream)
}

/// # Example
///
/// ```rust,ignore
/// const THREAD_STACK_SIZE = lits::bytes!("8 MiB");
///
/// #[mkutils_macros::tokio_main(thread_stack_size = THREAD_STACK_SIZE)]
/// fn main() {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn tokio_main(attr_args_token_stream: TokenStream, item_token_stream: TokenStream) -> TokenStream {
    crate::tokio_main::tokio_main(attr_args_token_stream, item_token_stream)
}
