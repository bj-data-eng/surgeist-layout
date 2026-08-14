use super::super::{
    DefaultScalar, FlowAxes, LayoutScalar, PhysicalAxis, ScrollRectOf, scalar::canonical_zero,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Display {
    Block,
    #[default]
    Flex,
    Grid,
    GridLanes,
    InlineBlock,
    InlineGrid,
    InlineGridLanes,
    None,
}

impl Display {
    #[must_use]
    pub const fn is_inline_level(self) -> bool {
        matches!(
            self,
            Self::InlineBlock | Self::InlineGrid | Self::InlineGridLanes
        )
    }

    /// Returns the display used to dispatch this box's own layout algorithm.
    ///
    /// This does not describe parent-flow participation. For example,
    /// `InlineBlock` participates in its parent as an atomic inline-level box,
    /// while its contents are laid out by the block formatting context.
    #[must_use]
    pub const fn inner_display(self) -> Self {
        match self {
            Self::InlineBlock => Self::Block,
            Self::InlineGrid => Self::Grid,
            Self::InlineGridLanes => Self::GridLanes,
            display => display,
        }
    }

    #[must_use]
    pub const fn establishes_grid_formatting_context(self) -> bool {
        matches!(self.inner_display(), Self::Grid | Self::GridLanes)
    }

    #[must_use]
    pub const fn establishes_grid_lanes_formatting_context(self) -> bool {
        matches!(self.inner_display(), Self::GridLanes)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BoxSizing {
    ContentBox,
    #[default]
    BorderBox,
}

/// The already-resolved used inline direction for layout input.
///
/// This is not an authored CSS `direction` token. Root style and text
/// integration resolve authored direction effects before constructing a layout
/// input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

impl Direction {
    #[must_use]
    pub const fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Position {
    #[default]
    Relative,
    Absolute,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Float {
    #[default]
    None,
    Left,
    Right,
}

/// Layout-ready exclusion geometry selected for a floating box.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FloatExclusion {
    /// Exclude the float's physical margin box.
    #[default]
    MarginBox,
    /// Request bounded exclusion geometry from the layout tree.
    Shape,
}

/// Construction error for a float exclusion query or provider interval.
///
/// Band endpoints are physical coordinates on the containing flow's block
/// axis. Interval endpoints are physical coordinates on its inline axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatExclusionIntervalErrorOf<S: LayoutScalar = DefaultScalar> {
    NonFiniteBandMinimum {
        value: S,
    },
    NonFiniteBandMaximum {
        value: S,
    },
    InvertedBand {
        minimum: S,
        maximum: S,
    },
    NonFiniteIntervalMinimum {
        value: S,
    },
    NonFiniteIntervalMaximum {
        value: S,
    },
    InvertedInterval {
        minimum: S,
        maximum: S,
    },
    QueryMismatch {
        expected: FloatExclusionQueryOf<S>,
        actual: FloatExclusionQueryOf<S>,
    },
}

/// Default-scalar float exclusion interval construction error.
pub type FloatExclusionIntervalError = FloatExclusionIntervalErrorOf<DefaultScalar>;

impl<S: LayoutScalar> core::fmt::Display for FloatExclusionIntervalErrorOf<S> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::NonFiniteBandMinimum { .. } => "band minimum must be finite",
            Self::NonFiniteBandMaximum { .. } => "band maximum must be finite",
            Self::InvertedBand { .. } => "band minimum must not exceed its maximum",
            Self::NonFiniteIntervalMinimum { .. } => "interval minimum must be finite",
            Self::NonFiniteIntervalMaximum { .. } => "interval maximum must be finite",
            Self::InvertedInterval { .. } => "interval minimum must not exceed its maximum",
            Self::QueryMismatch { .. } => "provider interval query must match the requested query",
        };
        formatter.write_str(message)
    }
}

impl<S: LayoutScalar> std::error::Error for FloatExclusionIntervalErrorOf<S> {}

/// Validated physical geometry supplied to a shape exclusion provider.
///
/// The margin box is final physical geometry. The ordered band endpoints are
/// physical coordinates on [`FlowAxes::block_axis`]. This value is local query
/// state and intentionally carries no cache revision.
///
/// ```compile_fail
/// use surgeist_layout::FloatExclusionQuery;
/// let _ = FloatExclusionQuery::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatExclusionQueryOf<S: LayoutScalar = DefaultScalar> {
    margin_box: ScrollRectOf<S>,
    flow_axes: FlowAxes,
    band_minimum: S,
    band_maximum: S,
}

/// Default-scalar physical float exclusion query.
pub type FloatExclusionQuery = FloatExclusionQueryOf<DefaultScalar>;

impl<S: LayoutScalar> FloatExclusionQueryOf<S> {
    /// Constructs a query with finite ordered physical band endpoints.
    pub fn try_new(
        margin_box: ScrollRectOf<S>,
        flow_axes: FlowAxes,
        band_minimum: S,
        band_maximum: S,
    ) -> Result<Self, FloatExclusionIntervalErrorOf<S>> {
        if !band_minimum.is_finite() {
            return Err(FloatExclusionIntervalErrorOf::NonFiniteBandMinimum {
                value: band_minimum,
            });
        }
        if !band_maximum.is_finite() {
            return Err(FloatExclusionIntervalErrorOf::NonFiniteBandMaximum {
                value: band_maximum,
            });
        }
        if band_minimum > band_maximum {
            return Err(FloatExclusionIntervalErrorOf::InvertedBand {
                minimum: band_minimum,
                maximum: band_maximum,
            });
        }

        Ok(Self {
            margin_box,
            flow_axes,
            band_minimum: canonical_zero(band_minimum),
            band_maximum: canonical_zero(band_maximum),
        })
    }

    #[must_use]
    pub const fn margin_box(self) -> ScrollRectOf<S> {
        self.margin_box
    }

    #[must_use]
    pub const fn flow_axes(self) -> FlowAxes {
        self.flow_axes
    }

    #[must_use]
    pub const fn band_minimum(self) -> S {
        self.band_minimum
    }

    #[must_use]
    pub const fn band_maximum(self) -> S {
        self.band_maximum
    }
}

/// One validated physical inline-axis interval returned by a shape provider.
///
/// Construction clips the interval to the query's physical float margin box.
/// A disjoint or zero-length intersection is returned as `None`.
///
/// ```compile_fail
/// use surgeist_layout::FloatExclusionInterval;
/// let _ = FloatExclusionInterval::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatExclusionIntervalOf<S: LayoutScalar = DefaultScalar> {
    query: FloatExclusionQueryOf<S>,
    minimum: S,
    maximum: S,
}

/// Default-scalar physical float exclusion interval.
pub type FloatExclusionInterval = FloatExclusionIntervalOf<DefaultScalar>;

impl<S: LayoutScalar> FloatExclusionIntervalOf<S> {
    /// Validates and clips an interval to `query`'s physical margin box.
    pub fn try_new(
        query: FloatExclusionQueryOf<S>,
        minimum: S,
        maximum: S,
    ) -> Result<Option<Self>, FloatExclusionIntervalErrorOf<S>> {
        if !minimum.is_finite() {
            return Err(FloatExclusionIntervalErrorOf::NonFiniteIntervalMinimum { value: minimum });
        }
        if !maximum.is_finite() {
            return Err(FloatExclusionIntervalErrorOf::NonFiniteIntervalMaximum { value: maximum });
        }
        if minimum > maximum {
            return Err(FloatExclusionIntervalErrorOf::InvertedInterval { minimum, maximum });
        }

        let margin_box = query.margin_box();
        let origin = margin_box.origin();
        let size = margin_box.size();
        let (box_minimum, box_maximum) = match query.flow_axes().inline_axis() {
            PhysicalAxis::Horizontal => (origin.x, origin.x + size.width),
            PhysicalAxis::Vertical => (origin.y, origin.y + size.height),
        };
        let minimum = minimum.max(box_minimum);
        let maximum = maximum.min(box_maximum);
        if minimum >= maximum {
            return Ok(None);
        }

        Ok(Some(Self {
            query,
            minimum: canonical_zero(minimum),
            maximum: canonical_zero(maximum),
        }))
    }

    #[must_use]
    pub(crate) const fn originating_query(self) -> FloatExclusionQueryOf<S> {
        self.query
    }

    #[must_use]
    pub const fn minimum(self) -> S {
        self.minimum
    }

    #[must_use]
    pub const fn maximum(self) -> S {
        self.maximum
    }
}

impl Float {
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Clear {
    #[default]
    None,
    Left,
    Right,
    Both,
}
