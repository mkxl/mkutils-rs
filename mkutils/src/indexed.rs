use derive_more::{Constructor, From};

#[derive(Constructor, From)]
pub struct Indexed<T> {
    pub index: usize,
    pub value: T,
}
