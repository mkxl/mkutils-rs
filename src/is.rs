pub trait Is<T>: Sized {
    fn into_self(self) -> T;
}

impl<T> Is<T> for T {
    fn into_self(self) -> T {
        self
    }
}
