#[macro_export]
macro_rules! when {
    ($($tokens:tt)*) => {{
        if $($tokens)* { true } else { false }
    }};
}
