use ropey::{Rope, RopeSlice};

pub trait AsRopeSlice {
    fn as_rope_slice(&self) -> RopeSlice<'_>;
}

impl AsRopeSlice for Rope {
    fn as_rope_slice(&self) -> RopeSlice<'_> {
        self.slice(..)
    }
}

impl<'a> AsRopeSlice for RopeSlice<'a> {
    fn as_rope_slice<'b>(&'b self) -> RopeSlice<'a> {
        *self
    }
}
