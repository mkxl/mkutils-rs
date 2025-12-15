use derive_more::{Constructor, From};
use std::io::Error as IoError;

#[derive(Constructor, From)]
pub struct ReadValue<P> {
    pub filepath: P,
    pub result: Result<String, IoError>,
}
