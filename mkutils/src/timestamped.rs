use derive_more::{Constructor, From};
use std::time::Instant;

#[derive(Constructor, From)]
pub struct Timestamped<T> {
    pub instant: Instant,
    pub value: T,
}
