use core::ops::{Add, Sub};

use super::{FlexDirection, Scalar};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Point<T = Scalar> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    #[must_use]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn map<R>(self, f: impl Fn(T) -> R) -> Point<R> {
        Point {
            x: f(self.x),
            y: f(self.y),
        }
    }

    #[must_use]
    pub fn transpose(self) -> Point<T> {
        Point {
            x: self.y,
            y: self.x,
        }
    }

    #[must_use]
    pub fn main(self, direction: FlexDirection) -> T {
        if direction.is_row() { self.x } else { self.y }
    }

    #[must_use]
    pub fn cross(self, direction: FlexDirection) -> T {
        if direction.is_row() { self.y } else { self.x }
    }
}

impl<T> Point<Option<T>> {
    pub const NONE: Self = Self { x: None, y: None };
}

impl Point<Scalar> {
    pub const ZERO: Self = Self::new(0.0, 0.0);
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size<T = Scalar> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    #[must_use]
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub fn map<R>(self, f: impl Fn(T) -> R) -> Size<R> {
        Size {
            width: f(self.width),
            height: f(self.height),
        }
    }

    #[must_use]
    pub fn zip_map<U, R>(self, other: Size<U>, f: impl Fn(T, U) -> R) -> Size<R> {
        Size {
            width: f(self.width, other.width),
            height: f(self.height, other.height),
        }
    }

    #[must_use]
    pub fn main(self, direction: FlexDirection) -> T {
        if direction.is_row() {
            self.width
        } else {
            self.height
        }
    }

    #[must_use]
    pub fn cross(self, direction: FlexDirection) -> T {
        if direction.is_row() {
            self.height
        } else {
            self.width
        }
    }
}

impl<T> Size<Option<T>> {
    pub const NONE: Self = Self {
        width: None,
        height: None,
    };
}

impl<T> Size<Option<T>> {
    #[must_use]
    pub const fn from_cross(direction: FlexDirection, value: Option<T>) -> Self {
        if direction.is_row() {
            Self {
                width: None,
                height: value,
            }
        } else {
            Self {
                width: value,
                height: None,
            }
        }
    }
}

impl Size<Scalar> {
    pub const ZERO: Self = Self::new(0.0, 0.0);
}

impl<T: Copy> Size<T> {
    pub const fn splat(value: T) -> Self {
        Self {
            width: value,
            height: value,
        }
    }
}

impl<U, T: Add<U>> Add<Size<U>> for Size<T> {
    type Output = Size<T::Output>;

    fn add(self, rhs: Size<U>) -> Self::Output {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<U, T: Sub<U>> Sub<Size<U>> for Size<T> {
    type Output = Size<T::Output>;

    fn sub(self, rhs: Size<U>) -> Self::Output {
        Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Edges<T = Scalar> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> Edges<T> {
    #[must_use]
    pub const fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub fn map<R>(self, f: impl Fn(T) -> R) -> Edges<R> {
        Edges {
            top: f(self.top),
            right: f(self.right),
            bottom: f(self.bottom),
            left: f(self.left),
        }
    }

    #[must_use]
    pub fn zip_size<U, R>(self, size: Size<U>, f: impl Fn(T, U) -> R) -> Edges<R>
    where
        U: Copy,
    {
        Edges {
            top: f(self.top, size.height),
            right: f(self.right, size.width),
            bottom: f(self.bottom, size.height),
            left: f(self.left, size.width),
        }
    }

    #[must_use]
    pub fn zip_inline_size<U, R>(self, size: Size<U>, f: impl Fn(T, U) -> R) -> Edges<R>
    where
        U: Copy,
    {
        Edges {
            top: f(self.top, size.width),
            right: f(self.right, size.width),
            bottom: f(self.bottom, size.width),
            left: f(self.left, size.width),
        }
    }
}

impl<U, T: Add<U>> Add<Edges<U>> for Edges<T> {
    type Output = Edges<T::Output>;

    fn add(self, rhs: Edges<U>) -> Self::Output {
        Edges {
            top: self.top + rhs.top,
            right: self.right + rhs.right,
            bottom: self.bottom + rhs.bottom,
            left: self.left + rhs.left,
        }
    }
}

impl<T: Copy> Edges<T> {
    #[must_use]
    pub const fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

impl<T> Edges<T>
where
    T: Add<Output = T> + Copy,
{
    #[must_use]
    pub fn horizontal_sum(self) -> T {
        self.left + self.right
    }

    #[must_use]
    pub fn vertical_sum(self) -> T {
        self.top + self.bottom
    }

    #[must_use]
    pub fn sum_axes(self) -> Size<T> {
        Size::new(self.horizontal_sum(), self.vertical_sum())
    }

    #[must_use]
    pub fn main_sum(self, direction: FlexDirection) -> T {
        if direction.is_row() {
            self.horizontal_sum()
        } else {
            self.vertical_sum()
        }
    }

    #[must_use]
    pub fn cross_sum(self, direction: FlexDirection) -> T {
        if direction.is_row() {
            self.vertical_sum()
        } else {
            self.horizontal_sum()
        }
    }
}

impl Edges<Scalar> {
    pub const ZERO: Self = Self::all(0.0);
}
