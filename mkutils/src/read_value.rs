use derive_more::Constructor;
use std::io::Error as IoError;

#[derive(Constructor)]
pub struct ReadValue<P> {
    pub filepath: P,
    pub result: Result<String, IoError>,
}
