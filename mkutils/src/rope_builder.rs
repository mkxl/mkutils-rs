use crate::Utils;
use ropey::{Rope, RopeBuilder as RopeyBuilder};
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    marker::Unpin,
    ops::Range,
    path::Path,
};
use tokio::{
    fs::File as TokioFile,
    io::{AsyncReadExt, BufReader as TokioBufReader},
};

/// An async builder for constructing `Rope` text structures from async readers.
///
/// `RopeBuilder` reads UTF-8 text asynchronously from a reader and builds a `Rope`
/// data structure. It handles UTF-8 validation and buffering internally, with a
/// configurable buffer size (default 8192 bytes).
///
/// # Type Parameters
///
/// - `R`: The async reader type
/// - `N`: Buffer size in bytes (default: 8192)
///
/// # Examples
///
/// ```rust,no_run
/// use mkutils::RopeBuilder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), std::io::Error> {
///     // Build from a file path
///     let builder = RopeBuilder::from_filepath("/path/to/file.txt").await?;
///     let rope = builder.build().await?;
///
///     println!("Loaded {} lines", rope.len_lines());
///     Ok(())
/// }
/// ```
pub struct RopeBuilder<R, const N: usize = 8192> {
    reader: R,
    builder: RopeyBuilder,
    buffer: [u8; N],
    unprocessed_byte_indices: Range<usize>,
    seen_eof: bool,
}

impl<R: AsyncReadExt + Unpin, const N: usize> RopeBuilder<R, N> {
    const INITIAL_BUFFER: [u8; N] = [0; N];
    const INITIAL_SEEN_EOF: bool = false;
    const INITIAL_UNPROCESSED_BYTE_INDICES: Range<usize> = 0..0;

    /// Creates a new `RopeBuilder` from an async reader.
    ///
    /// The reader will be consumed to build the rope when `build()` is called.
    pub fn new(reader: R) -> Self {
        let builder = RopeyBuilder::new();
        let buffer = Self::INITIAL_BUFFER;
        let unprocessed_byte_indices = Self::INITIAL_UNPROCESSED_BYTE_INDICES.clone();
        let seen_eof = Self::INITIAL_SEEN_EOF;

        Self {
            reader,
            builder,
            buffer,
            unprocessed_byte_indices,
            seen_eof,
        }
    }

    fn io_error_invalid_utf8() -> IoError {
        IoError::new(IoErrorKind::InvalidData, "input contained invalid utf-8")
    }

    fn io_error_unexpected_eof() -> IoError {
        IoError::new(
            IoErrorKind::UnexpectedEof,
            "input ended in the middle of a utf-8 code point",
        )
    }

    fn feed(unprocessed_byte_indices: &mut Range<usize>, builder: &mut RopeyBuilder, text: &str) {
        unprocessed_byte_indices.start += text.len();

        // NOTE: without the [.immutable()], this generates:
        // [warning: a method with this name may be added to the standard library in the future]
        if unprocessed_byte_indices.immutable().is_empty() {
            *unprocessed_byte_indices = Self::INITIAL_UNPROCESSED_BYTE_INDICES.clone();
        }

        builder.append(text);
    }

    async fn read(&mut self) -> Result<(), IoError> {
        if self.seen_eof {
            return ().ok();
        }

        if 0 < self.unprocessed_byte_indices.start && self.unprocessed_byte_indices.end == self.buffer.len() {
            let unprocessed_length = self.unprocessed_byte_indices.len();

            if 0 < unprocessed_length {
                self.buffer.copy_within(self.unprocessed_byte_indices.clone(), 0);
            }

            self.unprocessed_byte_indices = 0..unprocessed_length;
        }

        match self.reader.read(&mut self.buffer).await {
            Ok(0) => self.seen_eof = true,
            Ok(num_bytes_read) => self.unprocessed_byte_indices.end += num_bytes_read,
            Err(io_error) => io_error.err()?,
        }

        ().ok()
    }

    /// Consumes the builder and asynchronously constructs the final `Rope`.
    ///
    /// Reads all remaining data from the reader, validates UTF-8, and returns
    /// the completed rope.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Reading from the underlying reader fails
    /// - The input contains invalid UTF-8
    /// - The input ends in the middle of a UTF-8 code point
    pub async fn build(mut self) -> Result<Rope, IoError> {
        loop {
            if self.unprocessed_byte_indices.is_empty() {
                if self.seen_eof {
                    return self.builder.finish().ok();
                }

                self.read().await?;
            } else {
                let byte_str = &self.buffer[self.unprocessed_byte_indices.clone()];

                match byte_str.as_utf8() {
                    Ok(text) => Self::feed(&mut self.unprocessed_byte_indices, &mut self.builder, text),
                    Err(utf8_error) => {
                        let byte_substr_end_index = utf8_error.valid_up_to();
                        let byte_substr = &byte_str[..byte_substr_end_index];
                        let text = unsafe { std::str::from_utf8_unchecked(byte_substr) };

                        if 0 < byte_substr_end_index {
                            Self::feed(&mut self.unprocessed_byte_indices, &mut self.builder, text);
                        } else if utf8_error.error_len().is_some() {
                            return Self::io_error_invalid_utf8().err();
                        } else if self.seen_eof {
                            return Self::io_error_unexpected_eof().err();
                        } else {
                            self.read().await?;
                        }
                    }
                }
            }
        }
    }
}

impl<const N: usize> RopeBuilder<TokioBufReader<TokioFile>, N> {
    /// Creates a new `RopeBuilder` that will read from the file at the given path.
    ///
    /// This is a convenience method that opens the file asynchronously and wraps
    /// it in a buffered reader.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened.
    pub async fn from_filepath<T: AsRef<Path>>(filepath: T) -> Result<Self, IoError> {
        filepath
            .open_async()
            .await?
            .buf_reader_async()
            .pipe_into(Self::new)
            .ok()
    }
}
