use derive_more::{Constructor, From};
use std::io::Error as IoError;

/// A container for a file path and its associated read result.
///
/// `ReadValue` pairs a file path with the result of attempting to read that file
/// as a UTF-8 string. This is useful for batch file operations where you want to
/// keep track of which files succeeded or failed to read.
///
/// # Type Parameters
///
/// - `P`: The path type (typically `PathBuf`, `&Path`, or similar)
///
/// # Examples
///
/// ```rust
/// use mkutils::ReadValue;
/// use std::path::PathBuf;
///
/// let read_value = ReadValue {
///     filepath: PathBuf::from("example.txt"),
///     result: Ok(String::from("file contents")),
/// };
///
/// match read_value.result {
///     Ok(contents) => println!("Read {} from {:?}", contents.len(), read_value.filepath),
///     Err(e) => eprintln!("Failed to read {:?}: {}", read_value.filepath, e),
/// }
/// ```
#[derive(Constructor, From)]
pub struct ReadValue<P> {
    /// The path to the file that was read or attempted to be read.
    pub filepath: P,
    /// The result of reading the file as a UTF-8 string.
    pub result: Result<String, IoError>,
}
