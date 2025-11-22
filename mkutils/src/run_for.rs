use crate::utils::Utils;
use derive_more::Constructor;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

#[derive(Constructor)]
pub struct RunForError<T>(T);

impl<T> Debug for RunForError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        std::write!(f, "RunForError<{type_name}>", type_name = Self::type_name())
    }
}

impl<T> Display for RunForError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        std::write!(
            f,
            "{type_name} future resolved before the allotted time",
            type_name = Self::type_name()
        )
    }
}
