use crate::utils::Utils;
use derive_more::From;

#[derive(Debug, Default, From)]
pub struct OptionalResult<T, E>(Option<Result<T, E>>);

impl<T, E> OptionalResult<T, E> {
    pub fn into_inner(self) -> Option<Result<T, E>> {
        self.0
    }
}

impl<T: Default, E> OptionalResult<T, E> {
    pub fn into_result(self) -> Result<T, E> {
        match self.0 {
            Some(Ok(ok)) => ok.ok(),
            Some(Err(err)) => err.err(),
            None => T::default().ok(),
        }
    }
}

impl<T, E> From<T> for OptionalResult<T, E> {
    fn from(ok: T) -> Self {
        ok.ok().some().into()
    }
}

#[cfg(feature = "nightly")]
mod nightly {
    use super::{OptionalResult, Utils};
    use std::{
        convert::Infallible,
        ops::{ControlFlow, FromResidual, Try},
    };

    impl<T, E> Try for OptionalResult<T, E> {
        type Output = T;
        type Residual = Result<(), E>;

        fn from_output(output: Self::Output) -> Self {
            output.ok().some().into()
        }

        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self.0 {
                Some(Ok(ok)) => ok.into_continue(),
                Some(Err(err)) => err.err().into_break(),
                None => ().ok().into_break(),
            }
        }
    }

    // NOTE:
    // - required implementation: [https://doc.rust-lang.org/stable/std/ops/trait.FromResidual.html]
    // - <Self as Try>::Residual = Result<(), E>
    impl<T, E> FromResidual<Result<(), E>> for OptionalResult<T, E> {
        fn from_residual(residual: Result<(), E>) -> Self {
            match residual {
                Ok(()) => None,
                Err(err) => err.err().some(),
            }
            .into()
        }
    }

    // NOTE:
    // - [https://doc.rust-lang.org/stable/std/result/enum.Result.html#associatedtype.Residual]
    // - <Result<T, F> as Try>::Residual = Result<Infallible, F>
    impl<T, F, E: From<F>> FromResidual<Result<Infallible, F>> for OptionalResult<T, E> {
        fn from_residual(residual: Result<Infallible, F>) -> Self {
            match residual {
                Err(err) => err.convert::<E>().err().some().into(),
            }
        }
    }

    // NOTE:
    // - [https://doc.rust-lang.org/stable/std/option/enum.Option.html#associatedtype.Residual]
    // - <Option<T> as Try>::Residual = Option<Infallible>
    impl<T, E> FromResidual<Option<Infallible>> for OptionalResult<T, E> {
        fn from_residual(_residual: Option<Infallible>) -> Self {
            None.into()
        }
    }
}
