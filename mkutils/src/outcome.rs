use crate::utils::Utils;
use std::ops::{ControlFlow, FromResidual, Try};

#[derive(Debug, Default)]
pub enum Outcome<T, E> {
    #[default]
    None,
    Ok(T),
    Err(E),
}

impl<T, E> Outcome<T, E> {}

impl<T, E2, E1: From<E2>> From<Result<T, E2>> for Outcome<T, E1> {
    fn from(result: Result<T, E2>) -> Self {
        match result {
            Ok(ok) => Self::Ok(ok),
            Err(err) => Self::Err(err.into()),
        }
    }
}

impl<T, E2, E1: From<E2>> From<Option<Result<T, E2>>> for Outcome<T, E1> {
    fn from(result_opt: Option<Result<T, E2>>) -> Self {
        match result_opt {
            Some(Ok(ok)) => Self::Ok(ok),
            Some(Err(err)) => Self::Err(err.into()),
            None => Self::None,
        }
    }
}

impl<T, E> FromResidual<E> for Outcome<T, E> {
    fn from_residual(residual: E) -> Self {
        Self::Err(residual)
    }
}

impl<T, E> FromResidual<Option<E>> for Outcome<T, E> {
    fn from_residual(residual: Option<E>) -> Self {
        residual.map_or_default(Self::Err)
    }
}

// impl<T, R, E: From<R>> FromResidual<Option<R>> for Outcome<T, E> {
//     fn from_residual(residual: Option<R>) -> Self {
//         match residual {
//             Some(residual) => Self::Err(residual.into()),
//             None => Self::None,
//         }
//     }
// }

impl<T, E> Try for Outcome<T, E> {
    type Output = T;
    type Residual = Option<E>;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::None => None.into_break(),
            Self::Ok(ok) => ok.into_continue(),
            Self::Err(err) => err.some().into_break(),
        }
    }
}
