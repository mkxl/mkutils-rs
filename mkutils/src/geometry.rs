use derive_more::Constructor;
use num::traits::ConstZero;

#[derive(Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Constructor, Copy)]
pub struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    pub fn get(&self, orientation: Orientation) -> &T {
        match orientation {
            Orientation::Horizontal => &self.x,
            Orientation::Vertical => &self.y,
        }
    }

    pub fn get_mut(&mut self, orientation: Orientation) -> &mut T {
        match orientation {
            Orientation::Horizontal => &mut self.x,
            Orientation::Vertical => &mut self.y,
        }
    }
}

impl<T> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}

impl<T: ConstZero> Point<T> {
    pub const ORIGIN: Self = Self { x: T::ZERO, y: T::ZERO };
}

pub type PointUsize = Point<usize>;
