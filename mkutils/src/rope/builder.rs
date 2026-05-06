use crate::{rope::rope::Rope, utils::Utils};
use std::{io::Error as IoError, ops::Range};

pub struct RopeBuilder {
    rope: Rope,
    bytes: Vec<u8>,
    unprocessed_bytes_indices: Range<usize>,
    finished: bool,
}

impl RopeBuilder {
    const DEFAULT_CAPACITY: usize = 8192;
    const INITIAL_BYTE: u8 = 0;
    const INITIAL_FINISHED: bool = false;
    const INITIAL_UNPROCESSED_BYTE_INDICES: Range<usize> = 0..0;

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let rope = Rope::empty();
        let bytes = std::vec![Self::INITIAL_BYTE; capacity];
        let unprocessed_bytes_indices = Self::INITIAL_UNPROCESSED_BYTE_INDICES;
        let finished = Self::INITIAL_FINISHED;

        Self {
            rope,
            bytes,
            unprocessed_bytes_indices,
            finished,
        }
    }

    fn extended_graphemes_except_last(text: &str) -> &str {
        let Some(last_extended_grapheme_byte_index) = text.extended_grapheme_byte_indices().last() else {
            return "";
        };

        &text[..last_extended_grapheme_byte_index]
    }

    pub fn buffer_mut(&mut self) -> Option<&mut [u8]> {
        if self.finished {
            None
        } else {
            self.bytes[self.unprocessed_bytes_indices.end..].ref_mut().some()
        }
    }

    pub fn on_read(&mut self, num_bytes_read: usize) -> Result<(), IoError> {
        self.unprocessed_bytes_indices
            .end
            .saturating_add_assign(&num_bytes_read);

        let unprocessed_bytes = &self.bytes[self.unprocessed_bytes_indices.clone()];

        if num_bytes_read == 0 {
            if !unprocessed_bytes.is_empty() {
                self.rope
                    .push_extended_graphemes(unprocessed_bytes.as_utf8().io_result()?);
                self.finished.set_true();
            }

            return ().ok();
        }

        let utf8_bytes_len = match unprocessed_bytes.as_utf8() {
            Ok(text) => text.len(),
            Err(utf8_err) => {
                // NOTE: if [utf8_err.error_len()] is [Some(..)] then there is explicitly an invalid utf-8
                // character and we need to raise immediately, wheraes if it's [None], then the supplied bytes are
                // short some bytes which may be provided by a later call to [.read()]
                if utf8_err.error_len().is_some() {
                    return utf8_err.io_error().err();
                }

                utf8_err.valid_up_to()
            }
        };

        if utf8_bytes_len.is_positive() {
            // NOTE: use all but the last extended grapheme, as it might continue on in the next read
            let utf8_bytes_indices = self.unprocessed_bytes_indices.start.range_from_len(utf8_bytes_len);
            let utf8_bytes = &self.bytes[utf8_bytes_indices];
            let text = unsafe { std::str::from_utf8_unchecked(utf8_bytes) };
            let extended_graphemes = Self::extended_graphemes_except_last(text);

            if !extended_graphemes.is_empty() {
                self.rope.push_extended_graphemes(extended_graphemes);
                self.unprocessed_bytes_indices
                    .start
                    .saturating_add_assign(&extended_graphemes.len());
            }
        }

        let num_unprocessed_bytes = self.unprocessed_bytes_indices.len();

        if num_unprocessed_bytes.is_positive() {
            self.bytes.copy_within(self.unprocessed_bytes_indices.clone(), 0);
        }

        self.unprocessed_bytes_indices = 0..num_unprocessed_bytes;

        ().ok()
    }

    #[must_use]
    pub fn build(self) -> Rope {
        self.rope
    }
}

impl Default for RopeBuilder {
    fn default() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }
}
