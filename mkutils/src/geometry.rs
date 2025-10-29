use crate::utils::Utils;
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

    pub fn embed<S: From<T>>(self) -> Point<S> {
        Point::new(self.x, self.y)
    }

    pub fn into_pair<X: From<T>, Y: From<T>>(self) -> (X, Y) {
        (self.x.into(), self.y.into())
    }

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

impl<T, T1: From<T>, T2: From<T>> From<Point<T>> for (T1, T2) {
    fn from(point: Point<T>) -> Self {
        point.x.convert::<T1>().pair(point.y.into())
    }
}

impl<T: ConstZero> Point<T> {
    pub const ORIGIN: Self = Self { x: T::ZERO, y: T::ZERO };
}

pub type PointU16 = Point<u16>;
pub type PointUsize = Point<usize>;
