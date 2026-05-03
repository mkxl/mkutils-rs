#[macro_export]
macro_rules! when {
    ($($tokens:tt)*) => {{
        if $($tokens)* { true } else { false }
    }};
}

#[cfg(feature = "async")]
#[macro_export]
macro_rules! loop_select {
    ( $($tt:tt)* ) => {
        loop {
            ::tokio::select! { $($tt)* }
        }
    }
}

#[macro_export]
macro_rules! max {
    ($value:expr $(,)?) => { $value };
    ($head:expr, $($tail:expr),+ $(,)?) => {
        ::std::cmp::max($head, $crate::max!($($tail),+))
    };
}
