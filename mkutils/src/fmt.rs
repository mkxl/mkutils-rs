use derive_more::Constructor;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

#[derive(Constructor)]
pub struct Debugged<T>(T);

impl<T: Debug> Display for Debugged<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        self.0.fmt(formatter)
    }
}

#[derive(Constructor)]
pub struct OptionDisplay<T>(T);

impl<T: Display> Display for OptionDisplay<Option<T>> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        if let Some(value) = &self.0 {
            value.fmt(formatter)
        } else {
            Display::fmt("none", formatter)
        }
    }
}

#[derive(Constructor)]
pub struct ResultDisplay<T>(T);

impl<T: Display, E: Display> Display for ResultDisplay<Result<T, E>> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match &self.0 {
            Ok(ok) => ok.fmt(formatter),
            Err(err) => err.fmt(formatter),
        }
    }
}
