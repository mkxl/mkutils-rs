use std::fmt::{Display, Error as FmtError, Formatter};
use tracing::Level;

pub struct Status<'a, T, E>(pub &'a Result<T, E>);

impl<T, E> Status<'_, T, E> {
    #[must_use]
    pub fn level(&self) -> Level {
        if self.0.is_ok() { Level::INFO } else { Level::WARN }
    }
}

impl<T, E: Display> Display for Status<'_, T, E> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match &self.0 {
            Ok(_ok) => "success".fmt(formatter),
            Err(err) => err.fmt(formatter),
        }
    }
}
