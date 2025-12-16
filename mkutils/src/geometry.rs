use derive_more::IsVariant;
use num::traits::{ConstZero, SaturatingSub};
use serde::{Deserialize, Serialize};

/// Represents an orientation in 2D space.
///
/// Used to indicate whether an operation or measurement applies along the
/// horizontal (X) or vertical (Y) axis.
#[derive(Clone, Copy, IsVariant)]
pub enum Orientation {
    /// Horizontal orientation (X-axis)
    Horizontal,
    /// Vertical orientation (Y-axis)
    Vertical,
}

/// A 2D point with generic coordinate type.
///
/// `Point<T>` represents a position in 2D space with x and y coordinates of type `T`.
/// Common type aliases include `PointU16` for `Point<u16>` and `PointUsize` for `Point<usize>`.
///
/// # Examples
///
/// ```
/// use mkutils::{Point, PointUsize};
///
/// let origin = PointUsize::ORIGIN; // (0, 0)
/// let point = Point::new(10, 20);
/// let from_tuple: Point<u32> = (5, 15).into();
/// ```
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Point<T> {
    /// The x-coordinate (horizontal position)
    pub x: T,
    /// The y-coordinate (vertical position)
    pub y: T,
}

impl<T> Point<T> {
    /// Creates a new `Point` with the given coordinates (const version).
    ///
    /// This is a const function that takes coordinates directly without conversion.
    pub const fn const_new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Creates a new `Point` with the given coordinates.
    ///
    /// The coordinates are converted using `Into<T>`, allowing for flexible initialization.
    pub fn new<X: Into<T>, Y: Into<T>>(x: X, y: Y) -> Self {
        Self::const_new(x.into(), y.into())
    }

    /// Converts this point to a point with a different coordinate type.
    ///
    /// # Note
    ///
    /// We would like to have `impl<T, S: Into<T>> From<Point<S>> for Point<T>`,
    /// but it conflicts with the reflexive `From<T> for T` impl.
    // TODO: would like to have an [impl<T, S: Into<T>> From<Point<S>> for Point<T>]
    // impl but conflicts with [From<T> for T] when [S = T] in the first
    pub fn into_point<S: From<T>>(self) -> Point<S> {
        Point::new(self.x, self.y)
    }

    /// Gets a reference to the coordinate along the specified orientation.
    ///
    /// Returns `&x` for `Horizontal`, `&y` for `Vertical`.
    pub const fn get(&self, orientation: Orientation) -> &T {
        match orientation {
            Orientation::Horizontal => &self.x,
            Orientation::Vertical => &self.y,
        }
    }

    /// Gets a mutable reference to the coordinate along the specified orientation.
    ///
    /// Returns `&mut x` for `Horizontal`, `&mut y` for `Vertical`.
    pub const fn get_mut(&mut self, orientation: Orientation) -> &mut T {
        match orientation {
            Orientation::Horizontal => &mut self.x,
            Orientation::Vertical => &mut self.y,
        }
    }
}

impl<T: ConstZero> Point<T> {
    /// The origin point `(0, 0)`.
    pub const ORIGIN: Self = Self { x: T::ZERO, y: T::ZERO };
}

impl<T: Clone + SaturatingSub> Point<T> {
    /// Subtracts a value from one coordinate, saturating at the numeric bounds.
    ///
    /// Subtracts `diff` from the x coordinate if `orientation` is `Horizontal`,
    /// or from the y coordinate if `Vertical`. The other coordinate is cloned unchanged.
    #[must_use]
    pub fn saturating_sub(&self, diff: &T, orientation: Orientation) -> Self {
        if orientation.is_horizontal() {
            (self.x.saturating_sub(diff), self.y.clone())
        } else {
            (self.x.clone(), self.y.saturating_sub(diff))
        }
        .into()
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

/// A point with `u16` coordinates.
///
/// Commonly used for terminal and UI dimensions.
pub type PointU16 = Point<u16>;

/// A point with `usize` coordinates.
///
/// Commonly used for indexing and general-purpose 2D coordinates.
pub type PointUsize = Point<usize>;
