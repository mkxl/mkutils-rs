use std::fmt::{Debug, Display, Error as FmtError, Formatter};

pub struct Debugged<'a, T: ?Sized>(pub &'a T);

impl<T: Debug + ?Sized> Display for Debugged<'_, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        self.0.fmt(formatter)
    }
}
