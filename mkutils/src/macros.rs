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
    };
}
