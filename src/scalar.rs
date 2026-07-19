use core::fmt::Debug;
use core::ops::{Add, Div, Mul, Neg, Sub};

/// Numeric precision contract for one layout computation.
pub trait LayoutScalar:
    private::Sealed
    + Copy
    + Clone
    + Debug
    + Default
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + 'static
{
    const ZERO: Self;
    const ONE: Self;
    const INFINITY: Self;
    const NAN: Self;
    const EPSILON: Self;

    fn from_f32(value: f32) -> Self;
    fn from_f64(value: f64) -> Self;
    fn from_usize(value: usize) -> Self;
    fn abs(self) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn is_finite(self) -> bool;
    fn floor_to_usize_saturating(self) -> usize;
    fn to_f64(self) -> f64;
}

pub(crate) fn canonical_zero<S: LayoutScalar>(value: S) -> S {
    if value == S::ZERO { S::ZERO } else { value }
}

#[inline]
pub(crate) fn round_layout_coordinate<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

macro_rules! impl_layout_scalar {
    ($ty:ty) => {
        impl private::Sealed for $ty {}

        impl LayoutScalar for $ty {
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const INFINITY: Self = <$ty>::INFINITY;
            const NAN: Self = <$ty>::NAN;
            const EPSILON: Self = <$ty>::EPSILON;

            fn from_f32(value: f32) -> Self {
                value as Self
            }

            fn from_f64(value: f64) -> Self {
                value as Self
            }

            fn from_usize(value: usize) -> Self {
                value as Self
            }

            fn abs(self) -> Self {
                <$ty>::abs(self)
            }

            fn min(self, other: Self) -> Self {
                <$ty>::min(self, other)
            }

            fn max(self, other: Self) -> Self {
                <$ty>::max(self, other)
            }

            fn floor(self) -> Self {
                <$ty>::floor(self)
            }

            fn ceil(self) -> Self {
                <$ty>::ceil(self)
            }

            fn round(self) -> Self {
                <$ty>::round(self)
            }

            fn is_finite(self) -> bool {
                <$ty>::is_finite(self)
            }

            fn floor_to_usize_saturating(self) -> usize {
                if <$ty>::is_nan(self) || self <= 0.0 {
                    0
                } else if !<$ty>::is_finite(self) || self >= usize::MAX as Self {
                    usize::MAX
                } else {
                    <$ty>::floor(self) as usize
                }
            }

            fn to_f64(self) -> f64 {
                self as f64
            }
        }
    };
}

impl_layout_scalar!(f32);
impl_layout_scalar!(f64);

mod private {
    pub trait Sealed {}
}
