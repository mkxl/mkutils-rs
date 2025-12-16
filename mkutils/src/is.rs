/// A marker trait used to constrain types in the `Utils` trait methods.
///
/// `Is<T>` is automatically implemented for `T` (i.e., `impl<T> Is<T> for T`),
/// allowing it to act as a type equality constraint in trait method bounds.
///
/// This trait is primarily an internal implementation detail used throughout
/// mkutils to allow methods on the `Utils` trait to be constrained to specific
/// types while maintaining the blanket implementation `impl<T: ?Sized> Utils for T`.
///
/// # Examples
///
/// ```rust
/// use mkutils::{Utils, is::Is};
///
/// // This method is only available when Self is actually an Option<X>
/// fn process_option<T, X: Into<T>>(opt: impl Is<Option<X>>) -> Option<T> {
///     opt.into_self().map(Into::into)
/// }
/// ```
pub trait Is<T>: Sized {
    /// Converts `self` into the underlying type `T`.
    ///
    /// Since `Is<T>` is only implemented for `T` itself, this effectively
    /// acts as an identity function constrained by the type system.
    fn into_self(self) -> T;
}

impl<T> Is<T> for T {
    fn into_self(self) -> T {
        self
    }
}
