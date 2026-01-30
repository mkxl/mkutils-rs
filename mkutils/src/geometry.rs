use crate::utils::Utils;
use derive_more::IsVariant;
use num::traits::{ConstZero, SaturatingSub};
use ratatui::layout::{Constraint, Layout, Size};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, IsVariant)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    #[must_use]
    pub const fn toggled(&self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }

    pub const fn toggle(&mut self) {
        *self = self.toggled();
    }

    pub fn layout<I: IntoIterator<Item: Into<Constraint>>>(&self, constraints: I) -> Layout {
        self.is_horizontal()
            .if_else::<fn(I) -> Layout>(Layout::horizontal, Layout::vertical)(constraints)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    // TODO: would like to have an [impl<T, S: Into<T>> From<Point<S>> for Point<T>]
    // impl but conflicts with [From<T> for T] when [S = T] in the first
    pub fn into_point<S: From<T>>(self) -> Point<S> {
        self.x.pair(self.y).into()
    }

    pub fn into_pair(self) -> (T, T) {
        self.into()
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

impl<T: ConstZero> Point<T> {
    pub const ORIGIN: Self = Self { x: T::ZERO, y: T::ZERO };
}

impl<T: Clone + SaturatingSub> Point<T> {
    #[must_use]
    pub fn saturating_sub(&self, rhs: &Self) -> Self {
        let x = self.x.saturating_sub(&rhs.x);
        let y = self.y.saturating_sub(&rhs.y);

        Self::new(x, y)
    }
}

impl<T, X: Into<T>, Y: Into<T>> From<(X, Y)> for Point<T> {
    fn from((x, y): (X, Y)) -> Self {
        Self::new(x.into(), y.into())
    }
}

impl<T: From<u16>> From<Size> for Point<T> {
    fn from(size: Size) -> Self {
        size.convert::<(u16, u16)>().into()
    }
}

impl<T, X: From<T>, Y: From<T>> From<Point<T>> for (X, Y) {
    fn from(point: Point<T>) -> Self {
        point.x.convert::<X>().pair(point.y.into())
    }
}

pub type PointU16 = Point<u16>;
pub type PointUsize = Point<usize>;
