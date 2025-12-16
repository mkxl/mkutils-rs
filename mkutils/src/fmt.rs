use derive_more::Constructor;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

/// A wrapper that implements `Display` by using the `Debug` implementation.
///
/// `Debugged<T>` allows you to use a type's `Debug` implementation in contexts
/// that require `Display`, such as formatting with `{}` instead of `{:?}`.
///
/// # Examples
///
/// ```
/// use mkutils::{Utils, fmt::Debugged};
///
/// let numbers = vec![1, 2, 3];
/// let display_wrapper = numbers.debug(); // Returns Debugged<Vec<i32>>
/// println!("{}", display_wrapper); // Prints: [1, 2, 3]
/// ```
#[derive(Constructor)]
pub struct Debugged<'a, T: ?Sized>(&'a T);

impl<T: Debug + ?Sized> Display for Debugged<'_, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        self.0.fmt(formatter)
    }
}

/// A wrapper that implements `Display` for `Option<T>` with a custom "none" representation.
///
/// Instead of displaying nothing or panicking when the option is `None`,
/// `OptionalDisplay` displays the string "none".
///
/// # Examples
///
/// ```
/// use mkutils::{Utils, fmt::OptionalDisplay};
///
/// let some_value: Option<i32> = Some(42);
/// let none_value: Option<i32> = None;
///
/// println!("{}", some_value.optional_display()); // Prints: 42
/// println!("{}", none_value.optional_display()); // Prints: none
/// ```
#[derive(Constructor)]
pub struct OptionalDisplay<'a, T: ?Sized>(&'a T);

impl<T: Display> Display for OptionalDisplay<'_, Option<T>> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        if let Some(value) = self.0 {
            value.fmt(formatter)
        } else {
            Display::fmt("none", formatter)
        }
    }
}
