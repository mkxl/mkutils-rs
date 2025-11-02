use std::fmt::{Display, Error as FmtError, Formatter};
use tracing::Level;

pub struct Status<T, E>(pub Result<T, E>);

impl<T, E> Status<T, E> {
    #[must_use]
    pub const fn level(&self) -> Level {
        if self.0.is_ok() { Level::INFO } else { Level::WARN }
    }
}

impl<T, E: Display> Display for Status<T, E> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match &self.0 {
            Ok(_ok) => "success".fmt(formatter),
            Err(err) => err.fmt(formatter),
        }
    }
}
