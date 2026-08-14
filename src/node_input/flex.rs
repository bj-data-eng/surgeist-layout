use super::super::{DefaultScalar, LayoutScalar, NonNegativeFiniteScalarErrorOf};
use super::validate_numeric_property;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Flex direction selects a container-local logical main axis.
///
/// Physical-axis selection requires the container's resolved flow and is owned
/// by the crate-private flex algorithm.
///
/// ```compile_fail
/// use surgeist_layout::{FlexDirection, PhysicalAxis};
/// let _: PhysicalAxis = FlexDirection::Row.main_axis();
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{FlexDirection, PhysicalAxis};
/// let _: PhysicalAxis = FlexDirection::Column.cross_axis();
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl FlexDirection {
    #[must_use]
    pub const fn is_row(self) -> bool {
        matches!(self, Self::Row | Self::RowReverse)
    }

    #[must_use]
    pub const fn is_column(self) -> bool {
        matches!(self, Self::Column | Self::ColumnReverse)
    }

    #[must_use]
    pub const fn is_reverse(self) -> bool {
        matches!(self, Self::RowReverse | Self::ColumnReverse)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexGrowOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

pub type FlexGrow = FlexGrowOf<DefaultScalar>;

impl<S: LayoutScalar> FlexGrowOf<S> {
    pub const ZERO: Self = Self { value: S::ZERO };

    pub fn try_new(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self {
            value: validate_numeric_property(value)?,
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }
}

impl<S: LayoutScalar> Default for FlexGrowOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexShrinkOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

pub type FlexShrink = FlexShrinkOf<DefaultScalar>;

impl<S: LayoutScalar> FlexShrinkOf<S> {
    pub const ONE: Self = Self { value: S::ONE };

    pub fn try_new(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self {
            value: validate_numeric_property(value)?,
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }
}

impl<S: LayoutScalar> Default for FlexShrinkOf<S> {
    fn default() -> Self {
        Self::ONE
    }
}

/// A normalized, layout-ready flex-layout participation fact.
///
/// This is not authored or computed CSS `visibility`. [`Self::Normal`] is the
/// default. Only in-flow children of flex containers consume
/// [`Self::Collapsed`]; other contexts preserve their existing behavior.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FlexItemCollapse {
    #[default]
    Normal,
    Collapsed,
}
