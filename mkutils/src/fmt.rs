use derive_more::Constructor;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

#[derive(Constructor)]
pub struct Debugged<'a, T>(&'a T);

impl<T: Debug> Display for Debugged<'_, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        self.0.fmt(formatter)
    }
}

#[derive(Constructor)]
pub struct OptionDisplay<'a, T>(&'a Option<T>);

impl<T: Display> Display for OptionDisplay<'_, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        if let Some(value) = &self.0 {
            value.fmt(formatter)
        } else {
            Display::fmt("none", formatter)
        }
    }
}

#[derive(Constructor)]
pub struct ResultDisplay<'a, T, E>(&'a Result<T, E>);

impl<T: Display, E: Display> Display for ResultDisplay<'_, T, E> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match &self.0 {
            Ok(ok) => ok.fmt(formatter),
            Err(err) => err.fmt(formatter),
        }
    }
}

#[derive(Constructor)]
pub struct StatusDisplay<'a, T, E>(&'a Result<T, E>);

impl<T, E: Display> Display for StatusDisplay<'_, T, E> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match self.0 {
            Ok(_ok) => Display::fmt("ok", formatter),
            Err(err) => std::write!(formatter, "error: {err}"),
        }
    }
}
