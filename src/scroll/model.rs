use super::{
    AutoScrollbarOverflowObservation, CanonicalScrollGeometryErrorOf,
    CanonicalScrollGeometrySourceOf, OptionalPhysicalContributionIntervalsOf, UsedOverflow,
};
use crate::{
    DefaultScalar, Edges, FlowAxes, LayoutScalar, LogicalAxis, Overflow, PhysicalAxis, Point,
    ScrollMarginOf, ScrollSnapAlign, ScrollSnapStop, ScrollSnapType, Size, scalar::canonical_zero,
};

/// Atomic construction error for a finite physical scroll rectangle.
///
/// Every variant identifies the physical axis and rejected scalar. A
/// non-finite end additionally preserves the finite origin and size whose sum
/// overflowed.
///
/// ```compile_fail
/// use surgeist_layout::ScrollRectError;
/// let _ = ScrollRectError::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollRectErrorOf<S: LayoutScalar = DefaultScalar> {
    NonFiniteOrigin {
        axis: PhysicalAxis,
        value: S,
    },
    NonFiniteSize {
        axis: PhysicalAxis,
        value: S,
    },
    NegativeSize {
        axis: PhysicalAxis,
        value: S,
    },
    NonFiniteEnd {
        axis: PhysicalAxis,
        value: S,
        origin: S,
        size: S,
    },
}

pub type ScrollRectError = ScrollRectErrorOf<DefaultScalar>;

impl<S: LayoutScalar> core::fmt::Display for ScrollRectErrorOf<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (axis, message) = match self {
            Self::NonFiniteOrigin { axis, .. } => (*axis, "origin must be finite"),
            Self::NonFiniteSize { axis, .. } => (*axis, "size must be finite"),
            Self::NegativeSize { axis, .. } => (*axis, "size must be non-negative"),
            Self::NonFiniteEnd { axis, .. } => (*axis, "end must be finite"),
        };
        let axis = match axis {
            PhysicalAxis::Horizontal => "horizontal",
            PhysicalAxis::Vertical => "vertical",
        };
        write!(f, "scroll rectangle {axis} {message}")
    }
}

impl<S: LayoutScalar> std::error::Error for ScrollRectErrorOf<S> {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRectOf<S: LayoutScalar = DefaultScalar> {
    origin: Point<S>,
    size: Size<S>,
}

pub type ScrollRect = ScrollRectOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollRectOf<S> {
    pub fn try_new(origin: Point<S>, size: Size<S>) -> Result<Self, ScrollRectErrorOf<S>> {
        if !origin.x.is_finite() {
            return Err(ScrollRectErrorOf::NonFiniteOrigin {
                axis: PhysicalAxis::Horizontal,
                value: origin.x,
            });
        }
        if !origin.y.is_finite() {
            return Err(ScrollRectErrorOf::NonFiniteOrigin {
                axis: PhysicalAxis::Vertical,
                value: origin.y,
            });
        }
        if !size.width.is_finite() {
            return Err(ScrollRectErrorOf::NonFiniteSize {
                axis: PhysicalAxis::Horizontal,
                value: size.width,
            });
        }
        if !size.height.is_finite() {
            return Err(ScrollRectErrorOf::NonFiniteSize {
                axis: PhysicalAxis::Vertical,
                value: size.height,
            });
        }
        if size.width < S::ZERO {
            return Err(ScrollRectErrorOf::NegativeSize {
                axis: PhysicalAxis::Horizontal,
                value: size.width,
            });
        }
        if size.height < S::ZERO {
            return Err(ScrollRectErrorOf::NegativeSize {
                axis: PhysicalAxis::Vertical,
                value: size.height,
            });
        }

        let x_end = origin.x + size.width;
        if !x_end.is_finite() {
            return Err(ScrollRectErrorOf::NonFiniteEnd {
                axis: PhysicalAxis::Horizontal,
                value: x_end,
                origin: origin.x,
                size: size.width,
            });
        }
        let y_end = origin.y + size.height;
        if !y_end.is_finite() {
            return Err(ScrollRectErrorOf::NonFiniteEnd {
                axis: PhysicalAxis::Vertical,
                value: y_end,
                origin: origin.y,
                size: size.height,
            });
        }

        Ok(Self {
            origin: Point::new(canonical_zero(origin.x), canonical_zero(origin.y)),
            size: Size::new(canonical_zero(size.width), canonical_zero(size.height)),
        })
    }

    #[must_use]
    pub const fn origin(self) -> Point<S> {
        self.origin
    }

    #[must_use]
    pub const fn size(self) -> Size<S> {
        self.size
    }
}

/// A finite ordered physical clip interval.
///
/// Construction is crate-private so callers cannot create or mutate an
/// interval that violates its ordering or finite-value invariant.
///
/// ```compile_fail
/// use surgeist_layout::PhysicalClipAxis;
/// let _ = PhysicalClipAxis { range: todo!() };
/// ```
///
/// ```compile_fail
/// use surgeist_layout::PhysicalClipAxis;
/// fn mutate(mut value: PhysicalClipAxis) { value.range = todo!(); }
/// ```
///
/// ```compile_fail
/// use surgeist_layout::PhysicalClipAxis;
/// let _ = PhysicalClipAxis::try_new(0.0, 1.0);
/// ```
///
/// ```compile_fail
/// use surgeist_layout::PhysicalClipAxis;
/// let _ = PhysicalClipAxis::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalClipAxisOf<S: LayoutScalar = DefaultScalar> {
    pub(super) range: PhysicalScrollAxisRangeOf<S>,
}

pub type PhysicalClipAxis = PhysicalClipAxisOf<DefaultScalar>;

impl<S: LayoutScalar> PhysicalClipAxisOf<S> {
    #[must_use]
    pub const fn minimum(self) -> S {
        self.range.minimum()
    }

    #[must_use]
    pub const fn maximum(self) -> S {
        self.range.maximum()
    }
}

/// Independent optional finite clip intervals for physical x and y axes.
///
/// ```compile_fail
/// use surgeist_layout::OverflowClip;
/// let _ = OverflowClip { x: None, y: None };
/// ```
///
/// ```compile_fail
/// use surgeist_layout::OverflowClip;
/// fn mutate(mut value: OverflowClip) { value.x = None; }
/// ```
///
/// ```compile_fail
/// use surgeist_layout::OverflowClip;
/// let _ = OverflowClip::new(None, None);
/// ```
///
/// ```compile_fail
/// use surgeist_layout::OverflowClip;
/// let _ = OverflowClip::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverflowClipOf<S: LayoutScalar = DefaultScalar> {
    pub(super) x: Option<PhysicalClipAxisOf<S>>,
    pub(super) y: Option<PhysicalClipAxisOf<S>>,
}

pub type OverflowClip = OverflowClipOf<DefaultScalar>;

impl<S: LayoutScalar> OverflowClipOf<S> {
    #[must_use]
    pub const fn x(self) -> Option<PhysicalClipAxisOf<S>> {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> Option<PhysicalClipAxisOf<S>> {
        self.y
    }
}

#[cfg(test)]
impl<S: LayoutScalar> PartialEq<Option<ScrollRectOf<S>>> for OverflowClipOf<S> {
    fn eq(&self, other: &Option<ScrollRectOf<S>>) -> bool {
        match other {
            None => self.x.is_none() && self.y.is_none(),
            Some(rect) => {
                let origin = rect.origin();
                let size = rect.size();
                self.x.is_some_and(|x| {
                    x.minimum() == origin.x && x.maximum() == origin.x + size.width
                }) && self.y.is_some_and(|y| {
                    y.minimum() == origin.y && y.maximum() == origin.y + size.height
                })
            }
        }
    }
}

/// Immutable layout-produced geometry and metadata for one scroll target.
///
/// The nested value preserves local physical border geometry and scroll margin
/// together with the target's flow axes and semantic snap metadata. Root owns
/// retained association, transformed coordinates, oversized-target rules and
/// live snap selection.
///
/// ```compile_fail
/// use surgeist_layout::ScrollTargetGeometry;
/// let _ = ScrollTargetGeometry {
///     border_box: todo!(),
///     scroll_margin: todo!(),
///     flow_axes: todo!(),
///     snap_align: todo!(),
///     snap_stop: todo!(),
/// };
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{FlowAxes, ScrollTargetGeometry};
/// fn mutate(mut value: ScrollTargetGeometry, axes: FlowAxes) { value.flow_axes = axes; }
/// ```
///
/// ```compile_fail
/// use surgeist_layout::ScrollTargetGeometry;
/// let _ = ScrollTargetGeometry::new();
/// ```
///
/// ```compile_fail
/// use surgeist_layout::ScrollTargetGeometry;
/// let _ = ScrollTargetGeometry::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollTargetGeometryOf<S: LayoutScalar = DefaultScalar> {
    pub(super) border_box: ScrollRectOf<S>,
    pub(super) scroll_margin: ScrollMarginOf<S>,
    pub(super) flow_axes: FlowAxes,
    pub(super) snap_align: ScrollSnapAlign,
    pub(super) snap_stop: ScrollSnapStop,
}

pub type ScrollTargetGeometry = ScrollTargetGeometryOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollTargetGeometryOf<S> {
    #[must_use]
    pub const fn border_box(self) -> ScrollRectOf<S> {
        self.border_box
    }

    #[must_use]
    pub const fn scroll_margin(self) -> ScrollMarginOf<S> {
        self.scroll_margin
    }

    #[must_use]
    pub const fn flow_axes(self) -> FlowAxes {
        self.flow_axes
    }

    #[must_use]
    pub const fn snap_align(self) -> ScrollSnapAlign {
        self.snap_align
    }

    #[must_use]
    pub const fn snap_stop(self) -> ScrollSnapStop {
        self.snap_stop
    }
}

/// Immutable physical-edge scrollbar gutter output.
///
/// Layout constructs this value together with the rest of canonical scroll
/// geometry. Callers can inspect each edge independently but cannot construct
/// or mutate gutter geometry. The rectangles reflect the explicit normalized
/// gutter policy and scrollbar thickness supplied in layout input; no host UI
/// or live scrollbar state is retained here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarGutterRectsOf<S: LayoutScalar = DefaultScalar> {
    pub(super) top: Option<ScrollRectOf<S>>,
    pub(super) right: Option<ScrollRectOf<S>>,
    pub(super) bottom: Option<ScrollRectOf<S>>,
    pub(super) left: Option<ScrollRectOf<S>>,
}

pub type ScrollbarGutterRects = ScrollbarGutterRectsOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollbarGutterRectsOf<S> {
    #[must_use]
    pub const fn top(self) -> Option<ScrollRectOf<S>> {
        self.top
    }

    #[must_use]
    pub const fn right(self) -> Option<ScrollRectOf<S>> {
        self.right
    }

    #[must_use]
    pub const fn bottom(self) -> Option<ScrollRectOf<S>> {
        self.bottom
    }

    #[must_use]
    pub const fn left(self) -> Option<ScrollRectOf<S>> {
        self.left
    }

    #[cfg(test)]
    pub(crate) const fn vertical(self) -> Option<ScrollRectOf<S>> {
        match self.right {
            Some(right) => Some(right),
            None => self.left,
        }
    }
}

/// Construction error for a signed physical or flow-relative scroll coordinate.
///
/// This error has no default because each variant records the coordinate space,
/// axis, and finite value or endpoint that failed validation.
///
/// ```compile_fail
/// use surgeist_layout::ScrollCoordinateErrorOf;
/// let _ = ScrollCoordinateErrorOf::<f32>::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollCoordinateErrorOf<S: LayoutScalar = DefaultScalar> {
    NonFinitePhysicalOffset {
        axis: PhysicalAxis,
        value: S,
    },
    NonFiniteFlowRelativeOffset {
        axis: LogicalAxis,
        value: S,
    },
    NonFinitePhysicalRangeMinimum {
        axis: PhysicalAxis,
        value: S,
    },
    NonFinitePhysicalRangeMaximum {
        axis: PhysicalAxis,
        value: S,
    },
    NonFiniteFlowRelativeRangeMinimum {
        axis: LogicalAxis,
        value: S,
    },
    NonFiniteFlowRelativeRangeMaximum {
        axis: LogicalAxis,
        value: S,
    },
    InvertedPhysicalRange {
        axis: PhysicalAxis,
        minimum: S,
        maximum: S,
    },
    InvertedFlowRelativeRange {
        axis: LogicalAxis,
        minimum: S,
        maximum: S,
    },
}

pub type ScrollCoordinateError = ScrollCoordinateErrorOf<DefaultScalar>;

/// Finite physical x/y scroll offset. Positive x is rightward and positive y is downward.
///
/// ```compile_fail
/// use surgeist_layout::PhysicalScrollOffset;
/// let _ = PhysicalScrollOffset::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalScrollOffsetOf<S: LayoutScalar = DefaultScalar> {
    x: S,
    y: S,
}

pub type PhysicalScrollOffset = PhysicalScrollOffsetOf<DefaultScalar>;

impl<S: LayoutScalar> PhysicalScrollOffsetOf<S> {
    pub fn try_new(x: S, y: S) -> Result<Self, ScrollCoordinateErrorOf<S>> {
        if !x.is_finite() {
            return Err(ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
                axis: PhysicalAxis::Horizontal,
                value: x,
            });
        }
        if !y.is_finite() {
            return Err(ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
                axis: PhysicalAxis::Vertical,
                value: y,
            });
        }

        Ok(Self {
            x: canonical_zero(x),
            y: canonical_zero(y),
        })
    }

    #[must_use]
    pub const fn x(self) -> S {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> S {
        self.y
    }
}

/// Finite flow-relative inline/block scroll offset.
///
/// ```compile_fail
/// use surgeist_layout::FlowRelativeScrollOffset;
/// let _ = FlowRelativeScrollOffset::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowRelativeScrollOffsetOf<S: LayoutScalar = DefaultScalar> {
    inline: S,
    block: S,
}

pub type FlowRelativeScrollOffset = FlowRelativeScrollOffsetOf<DefaultScalar>;

impl<S: LayoutScalar> FlowRelativeScrollOffsetOf<S> {
    pub fn try_new(inline: S, block: S) -> Result<Self, ScrollCoordinateErrorOf<S>> {
        if !inline.is_finite() {
            return Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeOffset {
                axis: LogicalAxis::Inline,
                value: inline,
            });
        }
        if !block.is_finite() {
            return Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeOffset {
                axis: LogicalAxis::Block,
                value: block,
            });
        }

        Ok(Self {
            inline: canonical_zero(inline),
            block: canonical_zero(block),
        })
    }

    #[must_use]
    pub const fn inline(self) -> S {
        self.inline
    }

    #[must_use]
    pub const fn block(self) -> S {
        self.block
    }
}

/// Finite closed physical-axis scroll interval.
///
/// ```compile_fail
/// use surgeist_layout::PhysicalScrollAxisRange;
/// let _ = PhysicalScrollAxisRange::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalScrollAxisRangeOf<S: LayoutScalar = DefaultScalar> {
    minimum: S,
    maximum: S,
}

pub type PhysicalScrollAxisRange = PhysicalScrollAxisRangeOf<DefaultScalar>;

impl<S: LayoutScalar> PhysicalScrollAxisRangeOf<S> {
    pub(super) const fn new(minimum: S, maximum: S) -> Self {
        Self { minimum, maximum }
    }

    #[must_use]
    pub const fn minimum(self) -> S {
        self.minimum
    }

    #[must_use]
    pub const fn maximum(self) -> S {
        self.maximum
    }

    #[must_use]
    fn clamp(self, value: S) -> S {
        value.max(self.minimum).min(self.maximum)
    }
}

/// Finite closed flow-relative-axis scroll interval.
///
/// ```compile_fail
/// use surgeist_layout::FlowRelativeScrollAxisRange;
/// let _ = FlowRelativeScrollAxisRange::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowRelativeScrollAxisRangeOf<S: LayoutScalar = DefaultScalar> {
    minimum: S,
    maximum: S,
}

pub type FlowRelativeScrollAxisRange = FlowRelativeScrollAxisRangeOf<DefaultScalar>;

impl<S: LayoutScalar> FlowRelativeScrollAxisRangeOf<S> {
    pub(super) const fn new(minimum: S, maximum: S) -> Self {
        Self { minimum, maximum }
    }

    #[must_use]
    pub const fn minimum(self) -> S {
        self.minimum
    }

    #[must_use]
    pub const fn maximum(self) -> S {
        self.maximum
    }

    #[must_use]
    fn clamp(self, value: S) -> S {
        value.max(self.minimum).min(self.maximum)
    }
}

/// Finite closed x/y scroll intervals in physical coordinates.
///
/// A standalone range accepts any finite ordered endpoints. The range nested in
/// [`ScrollGeometryOf`] additionally contains the zero initial anchor. Its
/// canonical physical scroll-size components are `x.maximum() - x.minimum()`
/// and `y.maximum() - y.minimum()`, including zero for a non-scrollable axis.
///
/// ```compile_fail
/// use surgeist_layout::PhysicalScrollRange;
/// let _ = PhysicalScrollRange::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalScrollRangeOf<S: LayoutScalar = DefaultScalar> {
    x: PhysicalScrollAxisRangeOf<S>,
    y: PhysicalScrollAxisRangeOf<S>,
}

pub type PhysicalScrollRange = PhysicalScrollRangeOf<DefaultScalar>;

impl<S: LayoutScalar> PhysicalScrollRangeOf<S> {
    pub fn try_new(
        x_minimum: S,
        x_maximum: S,
        y_minimum: S,
        y_maximum: S,
    ) -> Result<Self, ScrollCoordinateErrorOf<S>> {
        validate_physical_scroll_range(PhysicalAxis::Horizontal, x_minimum, x_maximum)?;
        validate_physical_scroll_range(PhysicalAxis::Vertical, y_minimum, y_maximum)?;

        Ok(Self {
            x: PhysicalScrollAxisRangeOf::new(canonical_zero(x_minimum), canonical_zero(x_maximum)),
            y: PhysicalScrollAxisRangeOf::new(canonical_zero(y_minimum), canonical_zero(y_maximum)),
        })
    }

    #[must_use]
    pub const fn x(self) -> PhysicalScrollAxisRangeOf<S> {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> PhysicalScrollAxisRangeOf<S> {
        self.y
    }

    #[must_use]
    pub fn clamp(self, offset: PhysicalScrollOffsetOf<S>) -> PhysicalScrollOffsetOf<S> {
        PhysicalScrollOffsetOf {
            x: self.x.clamp(offset.x()),
            y: self.y.clamp(offset.y()),
        }
    }
}

/// Finite closed inline/block scroll intervals in flow-relative coordinates.
///
/// ```compile_fail
/// use surgeist_layout::FlowRelativeScrollRange;
/// let _ = FlowRelativeScrollRange::default();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowRelativeScrollRangeOf<S: LayoutScalar = DefaultScalar> {
    inline: FlowRelativeScrollAxisRangeOf<S>,
    block: FlowRelativeScrollAxisRangeOf<S>,
}

pub type FlowRelativeScrollRange = FlowRelativeScrollRangeOf<DefaultScalar>;

impl<S: LayoutScalar> FlowRelativeScrollRangeOf<S> {
    pub fn try_new(
        inline_minimum: S,
        inline_maximum: S,
        block_minimum: S,
        block_maximum: S,
    ) -> Result<Self, ScrollCoordinateErrorOf<S>> {
        validate_flow_relative_scroll_range(LogicalAxis::Inline, inline_minimum, inline_maximum)?;
        validate_flow_relative_scroll_range(LogicalAxis::Block, block_minimum, block_maximum)?;

        Ok(Self {
            inline: FlowRelativeScrollAxisRangeOf::new(
                canonical_zero(inline_minimum),
                canonical_zero(inline_maximum),
            ),
            block: FlowRelativeScrollAxisRangeOf::new(
                canonical_zero(block_minimum),
                canonical_zero(block_maximum),
            ),
        })
    }

    #[must_use]
    pub const fn inline(self) -> FlowRelativeScrollAxisRangeOf<S> {
        self.inline
    }

    #[must_use]
    pub const fn block(self) -> FlowRelativeScrollAxisRangeOf<S> {
        self.block
    }

    #[must_use]
    pub fn clamp(self, offset: FlowRelativeScrollOffsetOf<S>) -> FlowRelativeScrollOffsetOf<S> {
        FlowRelativeScrollOffsetOf {
            inline: self.inline.clamp(offset.inline()),
            block: self.block.clamp(offset.block()),
        }
    }
}

pub(super) fn validate_physical_scroll_range<S: LayoutScalar>(
    axis: PhysicalAxis,
    minimum: S,
    maximum: S,
) -> Result<(), ScrollCoordinateErrorOf<S>> {
    if !minimum.is_finite() {
        return Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMinimum {
            axis,
            value: minimum,
        });
    }
    if !maximum.is_finite() {
        return Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMaximum {
            axis,
            value: maximum,
        });
    }
    if minimum > maximum {
        return Err(ScrollCoordinateErrorOf::InvertedPhysicalRange {
            axis,
            minimum,
            maximum,
        });
    }

    Ok(())
}

fn validate_flow_relative_scroll_range<S: LayoutScalar>(
    axis: LogicalAxis,
    minimum: S,
    maximum: S,
) -> Result<(), ScrollCoordinateErrorOf<S>> {
    if !minimum.is_finite() {
        return Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeRangeMinimum {
            axis,
            value: minimum,
        });
    }
    if !maximum.is_finite() {
        return Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeRangeMaximum {
            axis,
            value: maximum,
        });
    }
    if minimum > maximum {
        return Err(ScrollCoordinateErrorOf::InvertedFlowRelativeRange {
            axis,
            minimum,
            maximum,
        });
    }

    Ok(())
}

impl FlowAxes {
    #[must_use]
    pub fn physical_scroll_offset<S: LayoutScalar>(
        self,
        flow_relative: FlowRelativeScrollOffsetOf<S>,
    ) -> PhysicalScrollOffsetOf<S> {
        let inline =
            self.project_scroll_offset_component(self.inline_axis(), flow_relative.inline());
        let block = self.project_scroll_offset_component(self.block_axis(), flow_relative.block());

        match self.inline_axis() {
            PhysicalAxis::Horizontal => PhysicalScrollOffsetOf {
                x: inline,
                y: block,
            },
            PhysicalAxis::Vertical => PhysicalScrollOffsetOf {
                x: block,
                y: inline,
            },
        }
    }

    #[must_use]
    pub fn flow_relative_scroll_offset<S: LayoutScalar>(
        self,
        physical: PhysicalScrollOffsetOf<S>,
    ) -> FlowRelativeScrollOffsetOf<S> {
        let (inline, block) = match self.inline_axis() {
            PhysicalAxis::Horizontal => (physical.x(), physical.y()),
            PhysicalAxis::Vertical => (physical.y(), physical.x()),
        };

        FlowRelativeScrollOffsetOf {
            inline: self.project_scroll_offset_component(self.inline_axis(), inline),
            block: self.project_scroll_offset_component(self.block_axis(), block),
        }
    }

    #[must_use]
    pub fn physical_scroll_range<S: LayoutScalar>(
        self,
        flow_relative: FlowRelativeScrollRangeOf<S>,
    ) -> PhysicalScrollRangeOf<S> {
        let inline = self.project_scroll_range_bounds(
            self.inline_axis(),
            flow_relative.inline().minimum(),
            flow_relative.inline().maximum(),
        );
        let block = self.project_scroll_range_bounds(
            self.block_axis(),
            flow_relative.block().minimum(),
            flow_relative.block().maximum(),
        );

        match self.inline_axis() {
            PhysicalAxis::Horizontal => PhysicalScrollRangeOf {
                x: PhysicalScrollAxisRangeOf::new(inline.0, inline.1),
                y: PhysicalScrollAxisRangeOf::new(block.0, block.1),
            },
            PhysicalAxis::Vertical => PhysicalScrollRangeOf {
                x: PhysicalScrollAxisRangeOf::new(block.0, block.1),
                y: PhysicalScrollAxisRangeOf::new(inline.0, inline.1),
            },
        }
    }

    #[must_use]
    pub fn flow_relative_scroll_range<S: LayoutScalar>(
        self,
        physical: PhysicalScrollRangeOf<S>,
    ) -> FlowRelativeScrollRangeOf<S> {
        let (inline, block) = match self.inline_axis() {
            PhysicalAxis::Horizontal => (physical.x(), physical.y()),
            PhysicalAxis::Vertical => (physical.y(), physical.x()),
        };
        let inline = self.project_scroll_range_bounds(
            self.inline_axis(),
            inline.minimum(),
            inline.maximum(),
        );
        let block =
            self.project_scroll_range_bounds(self.block_axis(), block.minimum(), block.maximum());

        FlowRelativeScrollRangeOf {
            inline: FlowRelativeScrollAxisRangeOf::new(inline.0, inline.1),
            block: FlowRelativeScrollAxisRangeOf::new(block.0, block.1),
        }
    }

    fn project_scroll_offset_component<S: LayoutScalar>(self, axis: PhysicalAxis, value: S) -> S {
        let projected = if self.physical_axis_progression(axis).is_decreasing() {
            -value
        } else {
            value
        };
        canonical_zero(projected)
    }

    fn project_scroll_range_bounds<S: LayoutScalar>(
        self,
        axis: PhysicalAxis,
        minimum: S,
        maximum: S,
    ) -> (S, S) {
        if self.physical_axis_progression(axis).is_decreasing() {
            (canonical_zero(-maximum), canonical_zero(-minimum))
        } else {
            (canonical_zero(minimum), canonical_zero(maximum))
        }
    }
}

/// Immutable canonical scroll-container and target geometry in local physical coordinates.
///
/// Layout owns construction from source facts. Public callers can inspect the
/// coherent result but cannot manufacture or mutate its derived parts. Computed
/// overflow input has already become private used-axis values; box nesting,
/// independent clips, gutters, the zero-anchored signed range, optimal viewing
/// region and nested target metadata are one canonical result.
///
/// This value deliberately carries no retained identity, transform, current
/// offset, snap selection, CSSOM state, host scrollbar UI or events. Root owns
/// those live concerns and consumes this geometry rather than recomputing it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollGeometryOf<S: LayoutScalar = DefaultScalar> {
    pub(super) source: CanonicalScrollGeometrySourceOf<S>,
    pub(super) flow_axes: FlowAxes,
    pub(super) used_overflow: UsedOverflow,
    pub(super) border_box: ScrollRectOf<S>,
    pub(super) padding_box: ScrollRectOf<S>,
    pub(super) content_box: ScrollRectOf<S>,
    pub(super) scrollport: ScrollRectOf<S>,
    pub(super) overflow_clip: OverflowClipOf<S>,
    pub(super) scrollable_overflow: ScrollRectOf<S>,
    pub(super) physical_range: PhysicalScrollRangeOf<S>,
    pub(super) auto_scrollbar_observation: AutoScrollbarOverflowObservation,
    pub(super) gutters: ScrollbarGutterRectsOf<S>,
    pub(super) aggregate_reservation: Size<S>,
    pub(super) resolved_scroll_padding: Edges<S>,
    pub(super) optimal_viewing_region: ScrollRectOf<S>,
    pub(super) scroll_snap_type: ScrollSnapType,
    pub(super) target: ScrollTargetGeometryOf<S>,
}

pub type ScrollGeometry = ScrollGeometryOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollGeometryOf<S> {
    #[must_use]
    pub const fn flow_axes(self) -> FlowAxes {
        self.flow_axes
    }

    #[must_use]
    pub const fn used_overflow_x(self) -> Overflow {
        self.used_overflow.x().value()
    }

    #[must_use]
    pub const fn used_overflow_y(self) -> Overflow {
        self.used_overflow.y().value()
    }

    #[must_use]
    pub const fn border_box(self) -> ScrollRectOf<S> {
        self.border_box
    }

    #[must_use]
    pub const fn padding_box(self) -> ScrollRectOf<S> {
        self.padding_box
    }

    #[must_use]
    pub const fn content_box(self) -> ScrollRectOf<S> {
        self.content_box
    }

    #[must_use]
    pub const fn scrollport(self) -> ScrollRectOf<S> {
        self.scrollport
    }

    #[must_use]
    pub const fn overflow_clip(self) -> OverflowClipOf<S> {
        self.overflow_clip
    }

    #[must_use]
    pub const fn scrollable_overflow(self) -> ScrollRectOf<S> {
        self.scrollable_overflow
    }

    #[must_use]
    pub const fn physical_range(self) -> PhysicalScrollRangeOf<S> {
        self.physical_range
    }

    #[must_use]
    pub const fn gutters(self) -> ScrollbarGutterRectsOf<S> {
        self.gutters
    }

    #[must_use]
    pub const fn scrollbar_size(self) -> Size<S> {
        self.aggregate_reservation
    }

    #[must_use]
    pub const fn resolved_scroll_padding(self) -> Edges<S> {
        self.resolved_scroll_padding
    }

    #[must_use]
    pub const fn optimal_viewing_region(self) -> ScrollRectOf<S> {
        self.optimal_viewing_region
    }

    #[must_use]
    pub const fn scroll_snap_type(self) -> ScrollSnapType {
        self.scroll_snap_type
    }

    #[must_use]
    pub const fn target(self) -> ScrollTargetGeometryOf<S> {
        self.target
    }

    #[must_use]
    pub(crate) const fn propagatable_descendant_intervals(
        self,
    ) -> OptionalPhysicalContributionIntervalsOf<S> {
        self.source
            .contributions
            .propagatable_descendant_intervals()
    }

    pub(crate) fn canonical_content_size(
        self,
    ) -> Result<Size<S>, CanonicalScrollGeometryErrorOf<S>> {
        self.source
            .contributions
            .content_size_from_anchor(self.content_box.origin())
            .map_err(CanonicalScrollGeometryErrorOf::Contribution)
    }
}

#[cfg(test)]
mod fri05_c02_carrier_tests {
    use super::*;
    use crate::{Direction, ScrollSnapAlignValue, WritingMode};

    fn assert_fri06_mr02_signed_zero_scroll_boundaries<S: LayoutScalar>(largest: S) {
        let zero_rect =
            ScrollRectOf::try_new(Point::new(-S::ZERO, S::ZERO), Size::new(-S::ZERO, S::ZERO))
                .unwrap_or_else(|error| panic!("signed zero rectangle is valid: {error:?}"));
        for value in [
            zero_rect.origin().x,
            zero_rect.origin().y,
            zero_rect.size().width,
            zero_rect.size().height,
        ] {
            assert_eq!(value, S::ZERO);
            assert!(!value.to_f64().is_sign_negative());
        }

        let finite = ScrollRectOf::try_new(
            Point::new(S::from_f64(-6.5), S::from_f64(4.25)),
            Size::new(S::from_f64(8.0), S::from_f64(9.5)),
        )
        .unwrap_or_else(|error| panic!("finite rectangle: {error:?}"));
        assert_eq!(finite.origin().x, S::from_f64(-6.5));
        assert_eq!(finite.origin().y, S::from_f64(4.25));
        assert_eq!(finite.size().width, S::from_f64(8.0));
        assert_eq!(finite.size().height, S::from_f64(9.5));

        assert!(matches!(
            ScrollRectOf::try_new(
                Point::new(S::INFINITY, -S::INFINITY),
                Size::new(S::NAN, S::ZERO),
            ),
            Err(ScrollRectErrorOf::NonFiniteOrigin {
                axis: PhysicalAxis::Horizontal,
                value,
            }) if value == S::INFINITY
        ));
        assert!(matches!(
            ScrollRectOf::try_new(Point::ZERO, Size::new(S::NAN, S::INFINITY)),
            Err(ScrollRectErrorOf::NonFiniteSize {
                axis: PhysicalAxis::Horizontal,
                value,
            }) if value.to_f64().is_nan()
        ));
        assert!(matches!(
            ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(-1.0), S::ZERO)),
            Err(ScrollRectErrorOf::NegativeSize {
                axis: PhysicalAxis::Horizontal,
                value,
            }) if value == S::from_f64(-1.0)
        ));
        assert!(matches!(
            ScrollRectOf::try_new(
                Point::new(largest, S::ZERO),
                Size::new(largest, S::ZERO),
            ),
            Err(ScrollRectErrorOf::NonFiniteEnd {
                axis: PhysicalAxis::Horizontal,
                value,
                origin,
                size,
            }) if !value.is_finite() && origin == largest && size == largest
        ));

        let offset = PhysicalScrollOffsetOf::try_new(-S::ZERO, S::from_f64(-3.0))
            .unwrap_or_else(|error| panic!("finite physical offset: {error:?}"));
        assert_eq!(offset.x(), S::ZERO);
        assert!(!offset.x().to_f64().is_sign_negative());
        assert_eq!(offset.y(), S::from_f64(-3.0));

        let range =
            PhysicalScrollRangeOf::try_new(S::from_f64(-4.0), S::from_f64(7.0), -S::ZERO, S::ZERO)
                .unwrap_or_else(|error| panic!("finite ordered physical range: {error:?}"));
        assert_eq!(range.x().minimum(), S::from_f64(-4.0));
        assert_eq!(range.x().maximum(), S::from_f64(7.0));
        for value in [range.y().minimum(), range.y().maximum()] {
            assert_eq!(value, S::ZERO);
            assert!(!value.to_f64().is_sign_negative());
        }
    }

    #[test]
    fn fri06_mr02_signed_zero_scroll_validation_ranges_and_order_are_preserved() {
        assert_fri06_mr02_signed_zero_scroll_boundaries::<f32>(f32::MAX);
        assert_fri06_mr02_signed_zero_scroll_boundaries::<f64>(f64::MAX);
    }

    fn clip_axis<S: LayoutScalar>(
        axis: PhysicalAxis,
        minimum: S,
        maximum: S,
    ) -> PhysicalClipAxisOf<S> {
        let range = match axis {
            PhysicalAxis::Horizontal => {
                PhysicalScrollRangeOf::try_new(minimum, maximum, S::ZERO, S::ZERO)
                    .map(PhysicalScrollRangeOf::x)
            }
            PhysicalAxis::Vertical => {
                PhysicalScrollRangeOf::try_new(S::ZERO, S::ZERO, minimum, maximum)
                    .map(PhysicalScrollRangeOf::y)
            }
        }
        .unwrap_or_else(|error| {
            panic!("test clip source must be a finite ordered physical range: {error:?}")
        });
        PhysicalClipAxisOf { range }
    }

    #[test]
    fn fri05_c02_carrier_clip_axes_are_finite_ordered_and_scalar_generic() {
        fn assert_scalar<S: LayoutScalar>() {
            let minimum = -S::from_f64(4.5);
            let maximum = S::from_f64(9.25);
            for axis in [PhysicalAxis::Horizontal, PhysicalAxis::Vertical] {
                let interval = clip_axis(axis, minimum, maximum);
                assert_eq!(interval.minimum(), minimum);
                assert_eq!(interval.maximum(), maximum);
            }

            assert!(
                PhysicalScrollRangeOf::<S>::try_new(S::INFINITY, maximum, S::ZERO, S::ZERO,)
                    .is_err()
            );
            assert!(
                PhysicalScrollRangeOf::<S>::try_new(minimum, S::INFINITY, S::ZERO, S::ZERO,)
                    .is_err()
            );
            assert!(
                PhysicalScrollRangeOf::<S>::try_new(maximum, minimum, S::ZERO, S::ZERO,).is_err()
            );
        }

        assert_scalar::<f32>();
        assert_scalar::<f64>();

        let zero = clip_axis(PhysicalAxis::Horizontal, -0.0_f32, -0.0_f32);
        assert_eq!(zero.minimum().to_bits(), 0.0_f32.to_bits());
        assert_eq!(zero.maximum().to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn fri05_c02_carrier_overflow_clip_keeps_axes_independently_optional() {
        fn assert_scalar<S: LayoutScalar>() {
            let x = clip_axis(
                PhysicalAxis::Horizontal,
                -S::from_f64(3.0),
                S::from_f64(11.0),
            );
            let y = clip_axis(PhysicalAxis::Vertical, S::from_f64(5.0), S::from_f64(17.0));

            let x_only = OverflowClipOf {
                x: Some(x),
                y: None,
            };
            assert_eq!(x_only.x(), Some(x));
            assert_eq!(x_only.y(), None);

            let y_only = OverflowClipOf {
                x: None,
                y: Some(y),
            };
            assert_eq!(y_only.x(), None);
            assert_eq!(y_only.y(), Some(y));

            let both = OverflowClipOf {
                x: Some(x),
                y: Some(y),
            };
            assert_eq!((both.x(), both.y()), (Some(x), Some(y)));

            let neither = OverflowClipOf::<S> { x: None, y: None };
            assert_eq!((neither.x(), neither.y()), (None, None));
        }

        assert_scalar::<f32>();
        assert_scalar::<f64>();
    }

    #[test]
    fn fri05_c02_carrier_target_retains_exact_layout_metadata_in_both_scalar_lanes() {
        fn assert_scalar<S: LayoutScalar>() {
            let border_box = ScrollRectOf::<S>::try_new(
                Point::new(-S::from_f64(2.0), S::from_f64(3.5)),
                Size::new(S::from_f64(40.25), S::from_f64(19.75)),
            )
            .unwrap();
            let scroll_margin = ScrollMarginOf::<S>::try_new(
                -S::from_f64(1.0),
                S::from_f64(2.0),
                -S::from_f64(3.0),
                S::from_f64(4.0),
            )
            .unwrap();
            let flow_axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl);
            let snap_align =
                ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
            let snap_stop = ScrollSnapStop::Always;
            let target = ScrollTargetGeometryOf {
                border_box,
                scroll_margin,
                flow_axes,
                snap_align,
                snap_stop,
            };

            assert_eq!(target.border_box(), border_box);
            assert_eq!(target.scroll_margin(), scroll_margin);
            assert_eq!(target.flow_axes(), flow_axes);
            assert_eq!(target.snap_align(), snap_align);
            assert_eq!(target.snap_stop(), snap_stop);
        }

        assert_scalar::<f32>();
        assert_scalar::<f64>();
    }
}
