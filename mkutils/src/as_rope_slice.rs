use ropey::{Rope, RopeSlice};

pub trait AsRopeSlice {
    fn as_rope_slice(&self) -> RopeSlice<'_>;
}

impl AsRopeSlice for Rope {
    fn as_rope_slice(&self) -> RopeSlice<'_> {
        self.slice(..)
    }
}

impl AsRopeSlice for RopeSlice<'_> {
    fn as_rope_slice(&self) -> RopeSlice<'_> {
        *self
    }
}
