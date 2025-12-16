use crate::utils::Utils;
use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
};

/// A tri-state result type for operations that can succeed, end successfully, or fail.
///
/// `Output<T, E>` is similar to `Option<Result<T, E>>` but with better ergonomics and
/// `?` operator support. It's particularly useful for streaming operations where you need
/// to distinguish between:
///
/// - `Ok(T)`: A successful value that should be processed
/// - `EndOk`: The stream/operation ended successfully (like `None` but signaling completion)
/// - `EndErr(E)`: The stream/operation ended with an error
///
/// # Examples
///
/// ```rust
/// use mkutils::Output;
///
/// fn process_stream_item(data: Output<i32, String>) -> Result<(), String> {
///     match data {
///         Output::Ok(value) => {
///             println!("Processing: {}", value);
///             Ok(())
///         }
///         Output::EndOk => {
///             println!("Stream ended successfully");
///             Ok(())
///         }
///         Output::EndErr(err) => Err(err),
///     }
/// }
/// ```
#[derive(Debug)]
pub enum Output<T, E> {
    /// A successful value.
    Ok(T),
    /// The operation ended successfully without a value.
    EndOk,
    /// The operation ended with an error.
    EndErr(E),
}

impl<T, E> Output<T, E> {
    /// Converts `Output<T, E>` to `Option<Result<T, E>>`.
    ///
    /// - `Ok(T)` becomes `Some(Ok(T))`
    /// - `EndOk` becomes `None`
    /// - `EndErr(E)` becomes `Some(Err(E))`
    pub fn into_option(self) -> Option<Result<T, E>> {
        match self {
            Self::Ok(ok) => ok.ok().some(),
            Self::EndOk => None,
            Self::EndErr(err) => err.err().some(),
        }
    }

    /// Converts to a `ControlFlow` for use in loops and control flow operations.
    ///
    /// - `Ok(T)` becomes `Continue(T)`
    /// - `EndOk` becomes `Break(Ok(()))`
    /// - `EndErr(E)` becomes `Break(Err(E))`
    pub fn into_control_flow(self) -> ControlFlow<Result<(), E>, T> {
        match self {
            Self::Ok(ok) => ok.into_continue(),
            Self::EndOk => ().ok().into_break(),
            Self::EndErr(err) => err.err().into_break(),
        }
    }

    /// Extracts the end state as a `Result<(), E>`.
    ///
    /// Returns `Ok(())` for both `Ok(_)` and `EndOk`, and `Err(e)` for `EndErr(e)`.
    /// Useful when you only care about whether the operation ended with an error.
    pub fn into_end(self) -> Result<(), E> {
        if let Self::EndErr(err) = self {
            err.err()
        } else {
            ().ok()
        }
    }
}

impl<T: Default, E> Output<T, E> {
    /// Converts to a `Result<T, E>`, using `T::default()` for the `EndOk` case.
    ///
    /// - `Ok(T)` becomes `Ok(T)`
    /// - `EndOk` becomes `Ok(T::default())`
    /// - `EndErr(E)` becomes `Err(E)`
    pub fn into_result(self) -> Result<T, E> {
        match self {
            Self::Ok(ok) => ok.ok(),
            Self::EndOk => T::default().ok(),
            Self::EndErr(err) => err.err(),
        }
    }
}

impl<T, E> From<T> for Output<T, E> {
    fn from(ok: T) -> Self {
        Self::Ok(ok)
    }
}

impl<T, E> From<Option<T>> for Output<T, E> {
    fn from(option: Option<T>) -> Self {
        if let Some(ok) = option {
            Self::Ok(ok)
        } else {
            Self::EndOk
        }
    }
}

impl<T, E> From<Option<Result<T, E>>> for Output<T, E> {
    fn from(opt_res: Option<Result<T, E>>) -> Self {
        match opt_res {
            Some(Ok(ok)) => Self::Ok(ok),
            Some(Err(err)) => Self::EndErr(err),
            None => Self::EndOk,
        }
    }
}

impl<T, E> From<Result<T, E>> for Output<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(ok) => Self::Ok(ok),
            Err(err) => Self::EndErr(err),
        }
    }
}

impl<T, E> Try for Output<T, E> {
    type Output = T;
    type Residual = Result<(), E>;

    fn from_output(output: Self::Output) -> Self {
        output.into()
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        self.into_control_flow()
    }
}

// NOTE:
// - enables the question mark operator to be applied to [Output<_, E2>]
//   when the enclosing function's return value is [Output<T, E>]
// - [Output<T, E>: FromResidual<<Output<T, E> as Try>::Residual>] is a
//   required implementation per
//   [https://doc.rust-lang.org/stable/std/ops/trait.FromResidual.html]
//   and is a specific case of the below impl when [E = E2]
// - [<Output<_, E2> as Try>::Residual = Result<(), E2>] per above
impl<T, E2, E: From<E2>> FromResidual<Result<(), E2>> for Output<T, E> {
    fn from_residual(residual: Result<(), E2>) -> Self {
        match residual {
            Ok(()) => Self::EndOk,
            Err(err) => Self::EndErr(err.into()),
        }
    }
}

// NOTE:
// - enables the question mark operator to be applied to [Result<_, E2>]
//   when the enclosing function's return value is [Output<T, E>]
// - [<Result<_, E2> as Try>::Residual = Result<Infallible, E2>] per
//   [https://doc.rust-lang.org/stable/std/result/enum.Result.html#associatedtype.Residual]
impl<T, E2, E: From<E2>> FromResidual<Result<Infallible, E2>> for Output<T, E> {
    fn from_residual(residual: Result<Infallible, E2>) -> Self {
        match residual {
            Err(err) => Self::EndErr(err.into()),
        }
    }
}

// NOTE:
// - enables the question mark operator to be applied to [Option<_>] when
//   the enclosing function's return value is [Output<T, E>]
// - [<Option<_> as Try>::Residual = Option<Infallible>] per
//   [https://doc.rust-lang.org/stable/std/option/enum.Option.html#associatedtype.Residual]
impl<T, E> FromResidual<Option<Infallible>> for Output<T, E> {
    fn from_residual(_residual: Option<Infallible>) -> Self {
        Self::EndOk
    }
}
