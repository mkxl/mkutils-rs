use crate::geometry::{Orientation, Point};
use ratatui::layout::{Position, Rect, Size};

pub trait Transpose {
    #[must_use]
    fn to_transpose(&self) -> Self;
}

impl Transpose for Orientation {
    fn to_transpose(&self) -> Self {
        self.toggled()
    }
}

impl<T: Clone> Transpose for Point<T> {
    fn to_transpose(&self) -> Self {
        Self::new(self.y.clone(), self.x.clone())
    }
}

impl Transpose for Rect {
    fn to_transpose(&self) -> Self {
        Self::new(self.y, self.x, self.height, self.width)
    }
}

impl Transpose for Size {
    fn to_transpose(&self) -> Self {
        Self::new(self.height, self.width)
    }
}

impl Transpose for Position {
    fn to_transpose(&self) -> Self {
        Self::new(self.y, self.x)
    }
}
