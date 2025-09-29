pub trait Is<T>: Sized {
    fn get(self) -> T;
}

impl<T> Is<T> for T {
    fn get(self) -> T {
        self
    }
}
