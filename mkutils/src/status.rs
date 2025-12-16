use std::fmt::{Display, Error as FmtError, Formatter};
use tracing::Level;

/// A wrapper around `Result<T, E>` that provides tracing integration.
///
/// `Status` wraps a result and provides methods to determine the appropriate
/// tracing level and display format based on whether the result is `Ok` or `Err`.
///
/// # Examples
///
/// ```rust,ignore
/// use mkutils::{Status, Utils};
/// use tracing::event;
///
/// let result: Result<(), String> = Ok(());
/// let status = result.into_status();
///
/// event!(status.level(), "{}", status);
/// ```
pub struct Status<T, E>(pub Result<T, E>);

impl<T, E> Status<T, E> {
    /// Returns the appropriate tracing level for this status.
    ///
    /// Returns `Level::INFO` for `Ok` results and `Level::WARN` for `Err` results.
    #[must_use]
    pub const fn level(&self) -> Level {
        if self.0.is_ok() { Level::INFO } else { Level::WARN }
    }
}

impl<T, E: Display> Display for Status<T, E> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match &self.0 {
            Ok(_ok) => "ok".fmt(formatter),
            Err(err) => std::write!(formatter, "error: {err}"),
        }
    }
}
