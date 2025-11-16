use derive_more::Constructor;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

#[derive(Constructor)]
pub struct Debugged<'a, T: ?Sized>(&'a T);

impl<T: Debug + ?Sized> Display for Debugged<'_, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        self.0.fmt(formatter)
    }
}

#[derive(Constructor)]
pub struct DisplayOptional<'a, T: ?Sized>(&'a T);

impl<T: Display> Display for DisplayOptional<'_, Option<T>> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        if let Some(value) = self.0 {
            value.fmt(formatter)
        } else {
            Display::fmt("None", formatter)
        }
    }
}
