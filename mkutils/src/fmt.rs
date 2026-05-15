use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Error as FmtError, Formatter},
};

macro_rules! fmt_type {
    ($name:ident, $value_type:ident $(, $phantom_types:ident)* $(,)?) => {
        pub struct $name<$value_type $(, $phantom_types)*> {
            value: $value_type,
            phantom: ::std::marker::PhantomData<($($phantom_types,)*)>,
        }

        impl<$value_type $(, $phantom_types)*> $name<$value_type $(, $phantom_types)*> {
            pub const fn new(value: $value_type) -> Self {
                let phantom = ::std::marker::PhantomData;

                Self { value, phantom }
            }
        }
    };
}

fmt_type!(Debugged, D, T);
fmt_type!(OptionDisplay, O, T);
fmt_type!(ResultDisplay, R, T, E);
fmt_type!(StatusDisplay, R, T, E);

impl<T: Debug, D: Borrow<T>> Display for Debugged<D, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        self.value.borrow().fmt(formatter)
    }
}

impl<T: Display, O: Borrow<Option<T>>> Display for OptionDisplay<O, T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        if let Some(value) = self.value.borrow() {
            value.fmt(formatter)
        } else {
            Display::fmt("none", formatter)
        }
    }
}

impl<T: Display, E: Display, R: Borrow<Result<T, E>>> Display for ResultDisplay<R, T, E> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match &self.value.borrow() {
            Ok(ok) => ok.fmt(formatter),
            Err(err) => err.fmt(formatter),
        }
    }
}

impl<T, E: Display, R: Borrow<Result<T, E>>> Display for StatusDisplay<R, T, E> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match self.value.borrow() {
            Ok(_ok) => Display::fmt("ok", formatter),
            Err(err) => std::write!(formatter, "error: {err}"),
        }
    }
}
