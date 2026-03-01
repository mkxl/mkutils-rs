use crate::{rope::rope::Rope, utils::Utils};
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    marker::Unpin,
    ops::Range,
};
use tokio::io::AsyncReadExt;

const DEFAULT_BUFFER_SIZE: usize = 8192;

pub struct RopeBuilder<R, const N: usize = DEFAULT_BUFFER_SIZE> {
    reader: R,
    rope: Rope,
    buffer: [u8; N],
    unprocessed_byte_offsets: Range<usize>,
    seen_eof: bool,
}

impl<R: AsyncReadExt + Unpin, const N: usize> RopeBuilder<R, N> {
    pub const DEFAULT_BUFFER_SIZE: usize = DEFAULT_BUFFER_SIZE;

    const INITIAL_BUFFER: [u8; N] = [0; N];
    const INITIAL_SEEN_EOF: bool = false;
    const INITIAL_UNPROCESSED_BYTE_OFFSETS: Range<usize> = 0..0;

    pub fn new(reader: R) -> Self {
        let rope = Rope::new();
        let buffer = Self::INITIAL_BUFFER;
        let unprocessed_byte_offsets = Self::INITIAL_UNPROCESSED_BYTE_OFFSETS.clone();
        let seen_eof = Self::INITIAL_SEEN_EOF;

        Self {
            reader,
            rope,
            buffer,
            unprocessed_byte_offsets,
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

    // NOTE: returns the byte index of the last extended grapheme boundary in [text] or 0 if the entire string is a
    // single, potentially incomplete, extended grapheme.
    fn last_extended_grapheme_byte_offset(text: &str) -> usize {
        text.extended_grapheme_byte_offsets().last().unwrap_or(0)
    }

    fn feed(unprocessed_byte_offsets: &mut Range<usize>, rope: &mut Rope, text: &str, seen_eof: bool) {
        // NOTE: when we've seen EOF there can be no further input that would extend the last
        // grapheme, so we flush everything; otherwise, split before the last grapheme in case it
        // straddles the buffer boundary
        let flush_end = if seen_eof {
            text.len()
        } else {
            Self::last_extended_grapheme_byte_offset(text)
        };

        if 0 < flush_end {
            rope.push_extended_graphemes(&text[..flush_end]);
            unprocessed_byte_offsets.start += flush_end;
        }

        if unprocessed_byte_offsets.immutable().is_empty() {
            *unprocessed_byte_offsets = Self::INITIAL_UNPROCESSED_BYTE_OFFSETS.clone();
        }
    }

    async fn read(&mut self) -> Result<(), IoError> {
        if self.seen_eof {
            return ().ok();
        }

        if 0 < self.unprocessed_byte_offsets.start && self.unprocessed_byte_offsets.end == self.buffer.len() {
            let unprocessed_length = self.unprocessed_byte_offsets.len();

            if 0 < unprocessed_length {
                self.buffer.copy_within(self.unprocessed_byte_offsets.clone(), 0);
            }

            self.unprocessed_byte_offsets = 0..unprocessed_length;
        }

        let io_res = self
            .reader
            .read(&mut self.buffer[self.unprocessed_byte_offsets.end..])
            .await;

        match io_res {
            Ok(0) => self.seen_eof.set_true(),
            Ok(num_bytes_read) => self.unprocessed_byte_offsets.end.saturating_add_assign(&num_bytes_read),
            Err(io_error) => io_error.err()?,
        }

        ().ok()
    }

    pub async fn build(mut self) -> Result<Rope, IoError> {
        loop {
            if self.unprocessed_byte_offsets.is_empty() {
                if self.seen_eof {
                    return self.rope.ok();
                }

                self.read().await?;
            } else {
                let byte_str = &self.buffer[self.unprocessed_byte_offsets.clone()];

                match byte_str.as_utf8() {
                    Ok(text) => Self::feed(&mut self.unprocessed_byte_offsets, &mut self.rope, text, self.seen_eof),
                    Err(utf8_error) => {
                        let end_byte_offset = utf8_error.valid_up_to();
                        let byte_substr = &byte_str[..end_byte_offset];
                        let text = unsafe { std::str::from_utf8_unchecked(byte_substr) };

                        if 0 < end_byte_offset {
                            Self::feed(&mut self.unprocessed_byte_offsets, &mut self.rope, text, false);
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
