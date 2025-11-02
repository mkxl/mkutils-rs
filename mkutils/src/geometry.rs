use num::traits::ConstZero;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub const fn const_new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn new<X: Into<T>, Y: Into<T>>(x: X, y: Y) -> Self {
        Self::const_new(x.into(), y.into())
    }

    // TODO: would like to have an [impl<T, S: Into<T>> From<Point<S>> for Point<T>]
    // impl but conflicts with [From<T> for T] when [S = T] in the first
    pub fn into_point<S: From<T>>(self) -> Point<S> {
        Point::new(self.x, self.y)
    }

    pub const fn get(&self, orientation: Orientation) -> &T {
        match orientation {
            Orientation::Horizontal => &self.x,
            Orientation::Vertical => &self.y,
        }
    }

    pub const fn get_mut(&mut self, orientation: Orientation) -> &mut T {
        match orientation {
            Orientation::Horizontal => &mut self.x,
            Orientation::Vertical => &mut self.y,
        }
    }
}

impl<T, X: Into<T>, Y: Into<T>> From<(X, Y)> for Point<T> {
    fn from((x, y): (X, Y)) -> Self {
        Self::new(x, y)
    }
}

impl<T, X: From<T>, Y: From<T>> From<Point<T>> for (X, Y) {
    fn from(point: Point<T>) -> Self {
        (point.x.into(), point.y.into())
    }
}

impl<T: ConstZero> Point<T> {
    pub const ORIGIN: Self = Self { x: T::ZERO, y: T::ZERO };
}

pub type PointU16 = Point<u16>;
pub type PointUsize = Point<usize>;
