use super::{
    DefaultScalar, Direction, Edges, FlowAxes, LayoutScalar, LogicalAxis, Overflow, PhysicalAxis,
    Point, Size,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollUnsupportedFeature {
    InvalidScrollRect,
    InvalidScrollGeometry,
    OverflowAuto,
    OverflowClipMargin,
    ScrollbarGutterStable,
    ScrollbarGutterBothEdges,
    ScrollPadding,
    ScrollMargin,
    ScrollSnap,
    LayoutOwnedMixedAxisOverflowCoupling,
}

impl ScrollUnsupportedFeature {
    #[must_use]
    pub const fn is_phase_one_deferred(self) -> bool {
        matches!(
            self,
            Self::OverflowAuto
                | Self::OverflowClipMargin
                | Self::ScrollbarGutterStable
                | Self::ScrollbarGutterBothEdges
                | Self::ScrollPadding
                | Self::ScrollMargin
                | Self::ScrollSnap
                | Self::LayoutOwnedMixedAxisOverflowCoupling
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRectOf<S: LayoutScalar = DefaultScalar> {
    origin: Point<S>,
    size: Size<S>,
}

pub type ScrollRect = ScrollRectOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollRectOf<S> {
    pub fn new(origin: Point<S>, size: Size<S>) -> Result<Self, ScrollUnsupportedFeature> {
        if !origin.x.is_finite()
            || !origin.y.is_finite()
            || !size.width.is_finite()
            || !size.height.is_finite()
            || size.width < S::ZERO
            || size.height < S::ZERO
        {
            return Err(ScrollUnsupportedFeature::InvalidScrollRect);
        }

        Ok(Self { origin, size })
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
            x: canonical_scroll_zero(x),
            y: canonical_scroll_zero(y),
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
            inline: canonical_scroll_zero(inline),
            block: canonical_scroll_zero(block),
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
    const fn new(minimum: S, maximum: S) -> Self {
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
    const fn new(minimum: S, maximum: S) -> Self {
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
            x: PhysicalScrollAxisRangeOf::new(
                canonical_scroll_zero(x_minimum),
                canonical_scroll_zero(x_maximum),
            ),
            y: PhysicalScrollAxisRangeOf::new(
                canonical_scroll_zero(y_minimum),
                canonical_scroll_zero(y_maximum),
            ),
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
                canonical_scroll_zero(inline_minimum),
                canonical_scroll_zero(inline_maximum),
            ),
            block: FlowRelativeScrollAxisRangeOf::new(
                canonical_scroll_zero(block_minimum),
                canonical_scroll_zero(block_maximum),
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

fn validate_physical_scroll_range<S: LayoutScalar>(
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

fn canonical_scroll_zero<S: LayoutScalar>(value: S) -> S {
    if value == S::ZERO { S::ZERO } else { value }
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
        canonical_scroll_zero(projected)
    }

    fn project_scroll_range_bounds<S: LayoutScalar>(
        self,
        axis: PhysicalAxis,
        minimum: S,
        maximum: S,
    ) -> (S, S) {
        if self.physical_axis_progression(axis).is_decreasing() {
            (
                canonical_scroll_zero(-maximum),
                canonical_scroll_zero(-minimum),
            )
        } else {
            (
                canonical_scroll_zero(minimum),
                canonical_scroll_zero(maximum),
            )
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollOverflowExposure {
    Visible,
    ClipOnly,
    ScrollableClip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollContainerAxis {
    exposure: ScrollOverflowExposure,
}

impl ScrollContainerAxis {
    pub const VISIBLE: Self = Self {
        exposure: ScrollOverflowExposure::Visible,
    };

    #[must_use]
    pub const fn exposure(self) -> ScrollOverflowExposure {
        self.exposure
    }

    #[must_use]
    pub const fn exposes_scroll_range(self) -> bool {
        matches!(self.exposure, ScrollOverflowExposure::ScrollableClip)
    }

    #[must_use]
    pub const fn clips_overflow(self) -> bool {
        matches!(
            self.exposure,
            ScrollOverflowExposure::ClipOnly | ScrollOverflowExposure::ScrollableClip
        )
    }

    pub const fn from_overflow(overflow: Overflow) -> Result<Self, ScrollUnsupportedFeature> {
        Ok(Self {
            exposure: match overflow {
                Overflow::Visible => ScrollOverflowExposure::Visible,
                Overflow::Clip => ScrollOverflowExposure::ClipOnly,
                Overflow::Hidden | Overflow::Scroll | Overflow::Auto => {
                    ScrollOverflowExposure::ScrollableClip
                }
            },
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollContainerFacts {
    x: ScrollContainerAxis,
    y: ScrollContainerAxis,
}

impl ScrollContainerFacts {
    #[must_use]
    pub const fn new(x: ScrollContainerAxis, y: ScrollContainerAxis) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(self) -> ScrollContainerAxis {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> ScrollContainerAxis {
        self.y
    }

    #[must_use]
    pub fn accepts_range<S: LayoutScalar>(self, range: PhysicalScrollRangeOf<S>) -> bool {
        (self.x.exposes_scroll_range()
            || range.x().minimum() == S::ZERO && range.x().maximum() == S::ZERO)
            && (self.y.exposes_scroll_range()
                || range.y().minimum() == S::ZERO && range.y().maximum() == S::ZERO)
    }

    #[must_use]
    pub const fn requires_overflow_clip(self) -> bool {
        self.x.clips_overflow() || self.y.clips_overflow()
    }

    #[must_use]
    pub const fn accepts_overflow_clip<S: LayoutScalar>(
        self,
        overflow_clip: Option<ScrollRectOf<S>>,
    ) -> bool {
        !self.requires_overflow_clip() || overflow_clip.is_some()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollOverflowCouplingPolicy {
    RootPreResolved,
    LayoutOwnedVisibleToAutoCoupling,
}

impl ScrollOverflowCouplingPolicy {
    pub const PHASE_ONE: Self = Self::RootPreResolved;

    #[must_use]
    pub const fn unsupported_feature(self) -> Option<ScrollUnsupportedFeature> {
        match self {
            Self::RootPreResolved => None,
            Self::LayoutOwnedVisibleToAutoCoupling => {
                Some(ScrollUnsupportedFeature::LayoutOwnedMixedAxisOverflowCoupling)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarGutterRectsOf<S: LayoutScalar = DefaultScalar> {
    horizontal: Option<ScrollRectOf<S>>,
    vertical: Option<ScrollRectOf<S>>,
}

pub type ScrollbarGutterRects = ScrollbarGutterRectsOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollbarGutterRectsOf<S> {
    #[must_use]
    pub const fn new(
        horizontal: Option<ScrollRectOf<S>>,
        vertical: Option<ScrollRectOf<S>>,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    #[must_use]
    pub const fn horizontal(self) -> Option<ScrollRectOf<S>> {
        self.horizontal
    }

    #[must_use]
    pub const fn vertical(self) -> Option<ScrollRectOf<S>> {
        self.vertical
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub struct ScrollbarReservationOf<S: LayoutScalar = DefaultScalar> {
    size: Size<S>,
    inset: Edges<S>,
}

#[allow(dead_code)]
pub type ScrollbarReservation = ScrollbarReservationOf<DefaultScalar>;

#[allow(dead_code)]
impl<S: LayoutScalar> ScrollbarReservationOf<S> {
    #[must_use]
    pub fn from_overflow(
        overflow: Point<Overflow>,
        scrollbar_width_value: S,
        direction: Direction,
    ) -> Self {
        let size = scrollbar_size_from_overflow(overflow, scrollbar_width_value);
        Self {
            size,
            inset: scrollbar_inset_from_size(size, direction),
        }
    }

    #[must_use]
    pub const fn size(self) -> Size<S> {
        self.size
    }

    #[must_use]
    pub const fn inset(self) -> Edges<S> {
        self.inset
    }
}

#[must_use]
#[allow(dead_code)]
pub fn scrollbar_size_from_overflow<S: LayoutScalar>(
    overflow: Point<Overflow>,
    scrollbar_width_value: S,
) -> Size<S> {
    Size::new(
        if overflow.y == Overflow::Scroll {
            scrollbar_width_value
        } else {
            S::ZERO
        },
        if overflow.x == Overflow::Scroll {
            scrollbar_width_value
        } else {
            S::ZERO
        },
    )
}

#[must_use]
#[allow(dead_code)]
pub fn scrollbar_inset_from_size<S: LayoutScalar>(size: Size<S>, direction: Direction) -> Edges<S> {
    match direction {
        Direction::Ltr => Edges {
            right: size.width,
            bottom: size.height,
            ..Edges::<S>::ZERO
        },
        Direction::Rtl => Edges {
            left: size.width,
            bottom: size.height,
            ..Edges::<S>::ZERO
        },
    }
}

#[must_use]
#[allow(dead_code)]
pub fn content_box_inset_with_scrollbar<S: LayoutScalar>(
    padding: Edges<S>,
    border: Edges<S>,
    reservation: ScrollbarReservationOf<S>,
) -> Edges<S> {
    padding + border + reservation.inset()
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub struct ScrollBoxRectsOf<S: LayoutScalar = DefaultScalar> {
    border_box: ScrollRectOf<S>,
    padding_box: ScrollRectOf<S>,
    content_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
    gutters: ScrollbarGutterRectsOf<S>,
}

#[allow(dead_code)]
pub type ScrollBoxRects = ScrollBoxRectsOf<DefaultScalar>;

#[allow(dead_code)]
impl<S: LayoutScalar> ScrollBoxRectsOf<S> {
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
    pub const fn gutters(self) -> ScrollbarGutterRectsOf<S> {
        self.gutters
    }
}

#[allow(dead_code)]
pub fn scroll_box_rects_from_border_box<S: LayoutScalar>(
    border_box: ScrollRectOf<S>,
    padding: Edges<S>,
    border: Edges<S>,
    reservation: ScrollbarReservationOf<S>,
) -> Result<ScrollBoxRectsOf<S>, ScrollUnsupportedFeature> {
    let padding_box = inset_scroll_rect(border_box, border)?;
    let content_box = inset_scroll_rect(
        border_box,
        content_box_inset_with_scrollbar(padding, border, reservation),
    )?;
    let scrollport = inset_scroll_rect(padding_box, reservation.inset())?;
    let gutters = scrollbar_gutter_rects_from_padding_box(padding_box, reservation)?;

    Ok(ScrollBoxRectsOf {
        border_box,
        padding_box,
        content_box,
        scrollport,
        gutters,
    })
}

fn inset_scroll_rect<S: LayoutScalar>(
    rect: ScrollRectOf<S>,
    inset: Edges<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let origin = rect.origin();
    let size = rect.size();
    ScrollRectOf::new(
        Point::new(origin.x + inset.left, origin.y + inset.top),
        Size::new(
            (size.width - inset.horizontal_sum()).max(S::ZERO),
            (size.height - inset.vertical_sum()).max(S::ZERO),
        ),
    )
}

fn scrollbar_gutter_rects_from_padding_box<S: LayoutScalar>(
    padding_box: ScrollRectOf<S>,
    reservation: ScrollbarReservationOf<S>,
) -> Result<ScrollbarGutterRectsOf<S>, ScrollUnsupportedFeature> {
    let origin = padding_box.origin();
    let size = padding_box.size();
    let gutter_size = reservation.size();
    let inset = reservation.inset();

    let vertical = if gutter_size.width > S::ZERO {
        let x = if inset.left > S::ZERO {
            origin.x
        } else {
            origin.x + (size.width - gutter_size.width).max(S::ZERO)
        };
        Some(ScrollRectOf::new(
            Point::new(x, origin.y),
            Size::new(
                gutter_size.width.min(size.width),
                (size.height - gutter_size.height).max(S::ZERO),
            ),
        )?)
    } else {
        None
    };

    let horizontal = if gutter_size.height > S::ZERO {
        let x = origin.x + inset.left.min(size.width);
        Some(ScrollRectOf::new(
            Point::new(
                x,
                origin.y + (size.height - gutter_size.height).max(S::ZERO),
            ),
            Size::new(
                (size.width - gutter_size.width).max(S::ZERO),
                gutter_size.height.min(size.height),
            ),
        )?)
    } else {
        None
    };

    Ok(ScrollbarGutterRectsOf::new(horizontal, vertical))
}

/// Layout-produced scroll-container geometry in local physical coordinates.
///
/// `scrollport`, `overflow_clip`, `scrollable_overflow`, and `gutters` are all
/// physical x/y rectangles. `physical_range` is a signed physical x/y range:
/// positive x is rightward and positive y is downward, so a reversed logical
/// progression can produce a negative physical interval. `flow_axes` retains
/// the writing-mode and direction context used to project that range.
///
/// This value describes layout geometry only. It does not retain a current
/// scroll offset; root and integration layers own live scroll state, host-event
/// conversion, and CSSOM policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollGeometryOf<S: LayoutScalar = DefaultScalar> {
    flow_axes: FlowAxes,
    container: ScrollContainerFacts,
    scrollport: ScrollRectOf<S>,
    overflow_clip: Option<ScrollRectOf<S>>,
    scrollable_overflow: ScrollRectOf<S>,
    physical_range: PhysicalScrollRangeOf<S>,
    gutters: ScrollbarGutterRectsOf<S>,
}

pub type ScrollGeometry = ScrollGeometryOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollGeometryOf<S> {
    #[allow(clippy::too_many_arguments)]
    /// Constructs physical scroll geometry with its retained flow context.
    ///
    /// Every rectangle is expressed in local physical x/y coordinates.
    /// `physical_range` permits any finite ordered signed bounds, rather than
    /// requiring a zero origin; `flow_axes` records how those physical bounds
    /// relate to the layout's inline and block progression. Construction checks
    /// the container's current exposure and clipping invariants, but does not
    /// own live scroll offset state or later range-origin policy.
    pub fn new(
        flow_axes: FlowAxes,
        container: ScrollContainerFacts,
        scrollport: ScrollRectOf<S>,
        overflow_clip: Option<ScrollRectOf<S>>,
        scrollable_overflow: ScrollRectOf<S>,
        physical_range: PhysicalScrollRangeOf<S>,
        gutters: ScrollbarGutterRectsOf<S>,
    ) -> Result<Self, ScrollUnsupportedFeature> {
        if !container.accepts_range(physical_range) {
            return Err(ScrollUnsupportedFeature::InvalidScrollGeometry);
        }
        if !container.accepts_overflow_clip(overflow_clip) {
            return Err(ScrollUnsupportedFeature::InvalidScrollGeometry);
        }

        Ok(Self {
            flow_axes,
            container,
            scrollport,
            overflow_clip,
            scrollable_overflow,
            physical_range,
            gutters,
        })
    }

    #[must_use]
    /// Returns the writing-mode and direction context retained with this
    /// physical geometry.
    pub const fn flow_axes(self) -> FlowAxes {
        self.flow_axes
    }

    #[must_use]
    /// Returns the physical-axis overflow exposure facts for this container.
    pub const fn container(self) -> ScrollContainerFacts {
        self.container
    }

    #[must_use]
    /// Returns the local physical x/y scrollport rectangle.
    pub const fn scrollport(self) -> ScrollRectOf<S> {
        self.scrollport
    }

    #[must_use]
    /// Returns the local physical x/y overflow clip rectangle when clipping is
    /// required by the container.
    pub const fn overflow_clip(self) -> Option<ScrollRectOf<S>> {
        self.overflow_clip
    }

    #[must_use]
    /// Returns the local physical x/y scrollable-overflow rectangle produced by
    /// the current layout calculation.
    pub const fn scrollable_overflow(self) -> ScrollRectOf<S> {
        self.scrollable_overflow
    }

    #[must_use]
    /// Returns the signed local physical x/y scroll range.
    ///
    /// Positive x is rightward and positive y is downward. Use `flow_axes()` to
    /// relate these bounds to inline/block progression; this geometry does not
    /// store a live current offset.
    pub const fn physical_range(self) -> PhysicalScrollRangeOf<S> {
        self.physical_range
    }

    #[must_use]
    /// Returns local physical x/y rectangles reserved for scrollbars.
    pub const fn gutters(self) -> ScrollbarGutterRectsOf<S> {
        self.gutters
    }
}

pub fn scroll_container_facts_from_overflow(
    overflow: Point<Overflow>,
) -> Result<ScrollContainerFacts, ScrollUnsupportedFeature> {
    Ok(ScrollContainerFacts::new(
        ScrollContainerAxis::from_overflow(overflow.x)?,
        ScrollContainerAxis::from_overflow(overflow.y)?,
    ))
}

#[allow(dead_code)]
pub fn scroll_rect_union<S: LayoutScalar>(
    a: ScrollRectOf<S>,
    b: ScrollRectOf<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let a_origin = a.origin();
    let b_origin = b.origin();
    let a_size = a.size();
    let b_size = b.size();
    let min_x = a_origin.x.min(b_origin.x);
    let min_y = a_origin.y.min(b_origin.y);
    let max_x = (a_origin.x + a_size.width).max(b_origin.x + b_size.width);
    let max_y = (a_origin.y + a_size.height).max(b_origin.y + b_size.height);

    ScrollRectOf::new(
        Point::new(min_x, min_y),
        Size::new((max_x - min_x).max(S::ZERO), (max_y - min_y).max(S::ZERO)),
    )
}

#[allow(dead_code)]
pub fn scrollable_overflow_from_content_size<S: LayoutScalar>(
    content_box: ScrollRectOf<S>,
    content_size: Size<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    scroll_rect_union(
        content_box,
        ScrollRectOf::new(
            content_box.origin(),
            Size::new(
                content_box.size().width.max(content_size.width),
                content_box.size().height.max(content_size.height),
            ),
        )?,
    )
}

#[allow(dead_code)]
pub fn scrollable_overflow_from_layout_content_size<S: LayoutScalar>(
    direction: Direction,
    overflow: Point<Overflow>,
    border_box_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    scrollbar_width_value: S,
    content_size: Size<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let reservation =
        ScrollbarReservationOf::from_overflow(overflow, scrollbar_width_value, direction);
    let rects = scroll_box_rects_from_border_box(
        ScrollRectOf::new(Point::ZERO, border_box_size)?,
        padding,
        border,
        reservation,
    )?;
    scrollable_overflow_from_content_size(rects.content_box(), content_size)
}

fn physical_scroll_range_from_overflow_rects<S: LayoutScalar>(
    flow_axes: FlowAxes,
    container: ScrollContainerFacts,
    scrollport: ScrollRectOf<S>,
    scrollable_overflow: ScrollRectOf<S>,
) -> Result<PhysicalScrollRangeOf<S>, ScrollUnsupportedFeature> {
    let scrollport_origin = scrollport.origin();
    let scrollport_size = scrollport.size();
    let scrollable_origin = scrollable_overflow.origin();
    let scrollable_size = scrollable_overflow.size();
    let x_magnitude = if container.x().exposes_scroll_range() {
        ((scrollable_origin.x + scrollable_size.width)
            - (scrollport_origin.x + scrollport_size.width))
            .max(S::ZERO)
    } else {
        S::ZERO
    };
    let y_magnitude = if container.y().exposes_scroll_range() {
        ((scrollable_origin.y + scrollable_size.height)
            - (scrollport_origin.y + scrollport_size.height))
            .max(S::ZERO)
    } else {
        S::ZERO
    };
    let inline_magnitude = match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => x_magnitude,
        PhysicalAxis::Vertical => y_magnitude,
    };
    let block_magnitude = match flow_axes.block_axis() {
        PhysicalAxis::Horizontal => x_magnitude,
        PhysicalAxis::Vertical => y_magnitude,
    };
    let flow_range = FlowRelativeScrollRangeOf::<S>::try_new(
        S::ZERO,
        inline_magnitude,
        S::ZERO,
        block_magnitude,
    )
    .map_err(|_| ScrollUnsupportedFeature::InvalidScrollGeometry)?;
    Ok(flow_axes.physical_scroll_range(flow_range))
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn scroll_geometry_from_layout<S: LayoutScalar>(
    flow_axes: FlowAxes,
    overflow: Point<Overflow>,
    border_box_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    scrollbar_width_value: S,
    scrollable_overflow: ScrollRectOf<S>,
) -> Result<ScrollGeometryOf<S>, ScrollUnsupportedFeature> {
    let container = scroll_container_facts_from_overflow(overflow)?;
    let reservation = ScrollbarReservationOf::from_overflow(
        overflow,
        scrollbar_width_value,
        flow_axes.direction(),
    );
    let rects = scroll_box_rects_from_border_box(
        ScrollRectOf::new(Point::ZERO, border_box_size)?,
        padding,
        border,
        reservation,
    )?;
    let physical_range = physical_scroll_range_from_overflow_rects(
        flow_axes,
        container,
        rects.scrollport(),
        scrollable_overflow,
    )?;
    let overflow_clip = container
        .requires_overflow_clip()
        .then_some(rects.scrollport());

    ScrollGeometryOf::new(
        flow_axes,
        container,
        rects.scrollport(),
        overflow_clip,
        scrollable_overflow,
        physical_range,
        rects.gutters(),
    )
}

pub fn round_scroll_geometry<S: LayoutScalar>(
    geometry: ScrollGeometryOf<S>,
    cumulative_origin: Point<S>,
) -> Result<ScrollGeometryOf<S>, ScrollUnsupportedFeature> {
    let scrollport = round_scroll_rect(geometry.scrollport(), cumulative_origin)?;
    let overflow_clip = geometry
        .overflow_clip()
        .map(|rect| round_scroll_rect(rect, cumulative_origin))
        .transpose()?;
    let scrollable_overflow = round_scroll_rect(geometry.scrollable_overflow(), cumulative_origin)?;
    let gutters = ScrollbarGutterRectsOf::new(
        geometry
            .gutters()
            .horizontal()
            .map(|rect| round_scroll_rect(rect, cumulative_origin))
            .transpose()?,
        geometry
            .gutters()
            .vertical()
            .map(|rect| round_scroll_rect(rect, cumulative_origin))
            .transpose()?,
    );
    let physical_range = physical_scroll_range_from_overflow_rects(
        geometry.flow_axes(),
        geometry.container(),
        scrollport,
        scrollable_overflow,
    )?;

    ScrollGeometryOf::new(
        geometry.flow_axes(),
        geometry.container(),
        scrollport,
        overflow_clip,
        scrollable_overflow,
        physical_range,
        gutters,
    )
}

fn round_scroll_rect<S: LayoutScalar>(
    rect: ScrollRectOf<S>,
    cumulative_origin: Point<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let origin = rect.origin();
    let size = rect.size();
    let rounded_origin = Point::new(
        round(cumulative_origin.x + origin.x) - round(cumulative_origin.x),
        round(cumulative_origin.y + origin.y) - round(cumulative_origin.y),
    );
    let rounded_end = Point::new(
        round(cumulative_origin.x + origin.x + size.width) - round(cumulative_origin.x),
        round(cumulative_origin.y + origin.y + size.height) - round(cumulative_origin.y),
    );
    ScrollRectOf::new(
        rounded_origin,
        Size::new(
            (rounded_end.x - rounded_origin.x).max(S::ZERO),
            (rounded_end.y - rounded_origin.y).max(S::ZERO),
        ),
    )
}

fn round<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}
