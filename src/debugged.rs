use std::fmt::{Debug, Display, Error as FmtError, Formatter};

pub struct Debugged<'a, T: ?Sized>(&'a T);

impl<'a, T: ?Sized> Debugged<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self(value)
    }
}

impl<T: Debug + ?Sized> Display for Debugged<'_, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        self.0.fmt(formatter)
    }
}
