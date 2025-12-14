#[cfg(feature = "derive_more")]
use derive_more::Constructor;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

#[cfg_attr(feature = "derive_more", derive(Constructor))]
pub struct Debugged<'a, T: ?Sized>(&'a T);

impl<T: Debug + ?Sized> Display for Debugged<'_, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        self.0.fmt(formatter)
    }
}

#[cfg_attr(feature = "derive_more", derive(Constructor))]
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
