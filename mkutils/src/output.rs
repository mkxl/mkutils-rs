use crate::utils::Utils;
use std::ops::ControlFlow;

pub enum Output<T, E> {
    Ok(T),
    EndOk,
    EndErr(E),
}

impl<T, E> Output<T, E> {
    pub fn into_option(self) -> Option<Result<T, E>> {
        match self {
            Self::Ok(ok) => ok.ok().some(),
            Self::EndOk => None,
            Self::EndErr(err) => err.err().some(),
        }
    }

    pub fn into_control_flow(self) -> ControlFlow<Result<(), E>, T> {
        match self {
            Self::Ok(ok) => ok.into_continue(),
            Self::EndOk => ().ok().into_break(),
            Self::EndErr(err) => err.err().into_break(),
        }
    }

    pub fn into_end(self) -> Result<(), E> {
        if let Self::EndErr(err) = self {
            err.err()
        } else {
            ().ok()
        }
    }
}

impl<T: Default, E> Output<T, E> {
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

#[cfg(feature = "nightly")]
mod nightly {
    use super::Output;
    use std::{
        convert::Infallible,
        ops::{ControlFlow, FromResidual, Try},
    };

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
}
