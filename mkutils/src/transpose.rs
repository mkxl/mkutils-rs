use crate::geometry::{Orientation, Point};
use ratatui::layout::{Position, Rect, Size};

pub trait Transpose {
    #[must_use]
    fn transpose(&self) -> Self;
}

impl Transpose for Orientation {
    fn transpose(&self) -> Self {
        self.toggled()
    }
}

impl<T: Clone> Transpose for Point<T> {
    fn transpose(&self) -> Self {
        Self::new(self.y.clone(), self.x.clone())
    }
}

impl Transpose for Rect {
    fn transpose(&self) -> Self {
        Self::new(self.y, self.x, self.height, self.width)
    }
}

impl Transpose for Size {
    fn transpose(&self) -> Self {
        Self::new(self.height, self.width)
    }
}

impl Transpose for Position {
    fn transpose(&self) -> Self {
        Self::new(self.y, self.x)
    }
}
