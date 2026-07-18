use super::{
    ComputedOverflow, DefaultScalar, Direction, Edges, FlowAxes, LayoutScalar, LengthPercentageOf,
    LogicalAxis, NumericResolutionOf, Overflow, OverflowClipBox, PercentageBasisOf, PhysicalAxis,
    PhysicalSide, Point, ScrollMarginOf, ScrollSnapAlign, ScrollSnapStop, ScrollSnapType,
    ScrollbarGutter, ScrollbarWidthOf, Size,
};
use crate::geometry::LogicalEdgesOf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UsedOverflowGutter {
    None,
    StableOnly,
    Conditional,
    Forced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UsedOverflowAxis {
    value: Overflow,
}

impl UsedOverflowAxis {
    const fn from_computed(value: Overflow, item_is_replaced: bool) -> Self {
        Self {
            value: if item_is_replaced && matches!(value, Overflow::Hidden) {
                Overflow::Clip
            } else {
                value
            },
        }
    }

    #[must_use]
    pub(crate) const fn value(self) -> Overflow {
        self.value
    }

    #[must_use]
    pub(crate) const fn clips_contents(self) -> bool {
        !matches!(self.value, Overflow::Visible)
    }

    #[must_use]
    pub(crate) const fn exposes_scroll_range(self) -> bool {
        matches!(
            self.value,
            Overflow::Hidden | Overflow::Scroll | Overflow::Auto
        )
    }

    #[must_use]
    pub(crate) const fn gutter_classification(self) -> UsedOverflowGutter {
        match self.value {
            Overflow::Visible | Overflow::Clip => UsedOverflowGutter::None,
            Overflow::Hidden => UsedOverflowGutter::StableOnly,
            Overflow::Auto => UsedOverflowGutter::Conditional,
            Overflow::Scroll => UsedOverflowGutter::Forced,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UsedOverflow {
    x: UsedOverflowAxis,
    y: UsedOverflowAxis,
}

impl UsedOverflow {
    #[must_use]
    pub(crate) const fn from_computed(computed: ComputedOverflow, item_is_replaced: bool) -> Self {
        Self {
            x: UsedOverflowAxis::from_computed(computed.x(), item_is_replaced),
            y: UsedOverflowAxis::from_computed(computed.y(), item_is_replaced),
        }
    }

    #[must_use]
    pub(crate) const fn x(self) -> UsedOverflowAxis {
        self.x
    }

    #[must_use]
    pub(crate) const fn y(self) -> UsedOverflowAxis {
        self.y
    }
}

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
            origin: Point::new(
                canonical_scroll_zero(origin.x),
                canonical_scroll_zero(origin.y),
            ),
            size: Size::new(
                canonical_scroll_zero(size.width),
                canonical_scroll_zero(size.height),
            ),
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
    range: PhysicalScrollAxisRangeOf<S>,
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
    x: Option<PhysicalClipAxisOf<S>>,
    y: Option<PhysicalClipAxisOf<S>>,
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
    border_box: ScrollRectOf<S>,
    scroll_margin: ScrollMarginOf<S>,
    flow_axes: FlowAxes,
    snap_align: ScrollSnapAlign,
    snap_stop: ScrollSnapStop,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SettledAutoScrollbarState {
    x: bool,
    y: bool,
}

impl SettledAutoScrollbarState {
    pub(crate) const INITIAL: Self = Self { x: false, y: false };

    #[must_use]
    #[cfg(test)]
    pub(crate) const fn new(x: bool, y: bool) -> Self {
        Self { x, y }
    }

    pub(crate) fn at(self, axis: PhysicalAxis) -> bool {
        match axis {
            PhysicalAxis::Horizontal => self.x,
            PhysicalAxis::Vertical => self.y,
        }
    }

    #[must_use]
    pub(crate) fn transition<S: LayoutScalar>(self, geometry: ScrollGeometryOf<S>) -> Self {
        Self {
            x: self.x
                || matches!(geometry.used_overflow_x(), Overflow::Auto)
                    && geometry
                        .auto_scrollbar_observation
                        .at(PhysicalAxis::Horizontal),
            y: self.y
                || matches!(geometry.used_overflow_y(), Overflow::Auto)
                    && geometry
                        .auto_scrollbar_observation
                        .at(PhysicalAxis::Vertical),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AutoScrollbarOverflowObservation {
    x: bool,
    y: bool,
}

impl AutoScrollbarOverflowObservation {
    fn from_range<S: LayoutScalar>(range: PhysicalScrollRangeOf<S>) -> Self {
        Self {
            x: range.x().maximum() > range.x().minimum(),
            y: range.y().maximum() > range.y().minimum(),
        }
    }

    fn at(self, axis: PhysicalAxis) -> bool {
        match axis {
            PhysicalAxis::Horizontal => self.x,
            PhysicalAxis::Vertical => self.y,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EffectiveScrollbarState {
    inline_start: bool,
    inline_end: bool,
    block_end: bool,
}

impl EffectiveScrollbarState {
    fn derive(
        flow_axes: FlowAxes,
        overflow: UsedOverflow,
        gutter: ScrollbarGutter,
        settled_auto: SettledAutoScrollbarState,
    ) -> Self {
        let block_overflow = used_overflow_at(overflow, flow_axes.block_axis());
        let inline_overflow = used_overflow_at(overflow, flow_axes.inline_axis());
        let block_settled = settled_auto.at(flow_axes.block_axis());
        let inline_settled = settled_auto.at(flow_axes.inline_axis());

        let inline_end = match block_overflow.gutter_classification() {
            UsedOverflowGutter::None => false,
            UsedOverflowGutter::StableOnly => !matches!(gutter, ScrollbarGutter::Auto),
            UsedOverflowGutter::Conditional => {
                block_settled || !matches!(gutter, ScrollbarGutter::Auto)
            }
            UsedOverflowGutter::Forced => true,
        };
        let block_end = match inline_overflow.gutter_classification() {
            UsedOverflowGutter::Conditional => inline_settled,
            UsedOverflowGutter::Forced => true,
            UsedOverflowGutter::None | UsedOverflowGutter::StableOnly => false,
        };

        Self {
            inline_start: inline_end && matches!(gutter, ScrollbarGutter::StableBothEdges),
            inline_end,
            block_end,
        }
    }
}

fn used_overflow_at(overflow: UsedOverflow, axis: PhysicalAxis) -> UsedOverflowAxis {
    match axis {
        PhysicalAxis::Horizontal => overflow.x(),
        PhysicalAxis::Vertical => overflow.y(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PhysicalEdgeReservationOf<S: LayoutScalar> {
    requested: Edges<S>,
}

impl<S: LayoutScalar> PhysicalEdgeReservationOf<S> {
    fn derive(
        flow_axes: FlowAxes,
        state: EffectiveScrollbarState,
        scrollbar_width: ScrollbarWidthOf<S>,
    ) -> Self {
        let width = scrollbar_width.get();
        let logical = LogicalEdgesOf::new(
            if state.inline_start { width } else { S::ZERO },
            if state.inline_end { width } else { S::ZERO },
            S::ZERO,
            if state.block_end { width } else { S::ZERO },
        );
        Self {
            requested: flow_axes.physical_edges(logical),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasuredLeafContentBoxInsetSourceOf<S: LayoutScalar> {
    pub(crate) flow_axes: FlowAxes,
    pub(crate) computed_overflow: ComputedOverflow,
    pub(crate) item_is_replaced: bool,
    pub(crate) scrollbar_gutter: ScrollbarGutter,
    pub(crate) scrollbar_width: ScrollbarWidthOf<S>,
    pub(crate) settled_auto_scrollbars: SettledAutoScrollbarState,
    pub(crate) padding: Edges<S>,
    pub(crate) border: Edges<S>,
}

#[must_use]
pub(crate) fn measured_leaf_content_box_inset<S: LayoutScalar>(
    source: MeasuredLeafContentBoxInsetSourceOf<S>,
) -> Edges<S> {
    let used_overflow =
        UsedOverflow::from_computed(source.computed_overflow, source.item_is_replaced);
    let scrollbar_state = EffectiveScrollbarState::derive(
        source.flow_axes,
        used_overflow,
        source.scrollbar_gutter,
        source.settled_auto_scrollbars,
    );
    let reservation = PhysicalEdgeReservationOf::derive(
        source.flow_axes,
        scrollbar_state,
        source.scrollbar_width,
    );
    source.border + source.padding + reservation.requested
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
    top: Option<ScrollRectOf<S>>,
    right: Option<ScrollRectOf<S>>,
    bottom: Option<ScrollRectOf<S>>,
    left: Option<ScrollRectOf<S>>,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ClipMarginSourceOf<S: LayoutScalar> {
    reference_box: OverflowClipBox,
    margin: S,
}

impl<S: LayoutScalar> ClipMarginSourceOf<S> {
    pub(crate) fn new(reference_box: OverflowClipBox, margin: S) -> Self {
        Self {
            reference_box,
            margin,
        }
    }
}

impl<S: LayoutScalar> Default for ClipMarginSourceOf<S> {
    fn default() -> Self {
        Self::new(OverflowClipBox::PaddingBox, S::ZERO)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum OptimalRegionInsetOf<S: LayoutScalar> {
    #[default]
    Auto,
    Value(LengthPercentageOf<S>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OptimalRegionInsetsOf<S: LayoutScalar> {
    top: OptimalRegionInsetOf<S>,
    right: OptimalRegionInsetOf<S>,
    bottom: OptimalRegionInsetOf<S>,
    left: OptimalRegionInsetOf<S>,
}

impl<S: LayoutScalar> OptimalRegionInsetsOf<S> {
    pub(crate) fn new(
        top: OptimalRegionInsetOf<S>,
        right: OptimalRegionInsetOf<S>,
        bottom: OptimalRegionInsetOf<S>,
        left: OptimalRegionInsetOf<S>,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl<S: LayoutScalar> Default for OptimalRegionInsetsOf<S> {
    fn default() -> Self {
        Self::new(
            OptimalRegionInsetOf::Auto,
            OptimalRegionInsetOf::Auto,
            OptimalRegionInsetOf::Auto,
            OptimalRegionInsetOf::Auto,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScrollBoxClipGutterSourceOf<S: LayoutScalar> {
    flow_axes: FlowAxes,
    used_overflow: UsedOverflow,
    border_box_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
    scrollbar_gutter: ScrollbarGutter,
    scrollbar_width: ScrollbarWidthOf<S>,
    settled_auto_scrollbars: SettledAutoScrollbarState,
    clip_margin: ClipMarginSourceOf<S>,
    optimal_region_insets: OptimalRegionInsetsOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalScrollBoxSourceOf<S: LayoutScalar> {
    pub(crate) flow_axes: FlowAxes,
    pub(crate) computed_overflow: ComputedOverflow,
    pub(crate) item_is_replaced: bool,
    pub(crate) border_box_size: Size<S>,
    pub(crate) border: Edges<S>,
    pub(crate) padding: Edges<S>,
    pub(crate) scrollbar_gutter: ScrollbarGutter,
    pub(crate) scrollbar_width: ScrollbarWidthOf<S>,
    pub(crate) settled_auto_scrollbars: SettledAutoScrollbarState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalScrollBoxOf<S: LayoutScalar> {
    border_box: ScrollRectOf<S>,
    padding_box: ScrollRectOf<S>,
    effective_border: Edges<S>,
    effective_padding: Edges<S>,
    effective_gutter: Edges<S>,
    scrollport: ScrollRectOf<S>,
    content_box: ScrollRectOf<S>,
}

impl<S: LayoutScalar> CanonicalScrollBoxOf<S> {
    #[must_use]
    pub(crate) const fn border_box(self) -> ScrollRectOf<S> {
        self.border_box
    }

    #[must_use]
    pub(crate) const fn padding_box(self) -> ScrollRectOf<S> {
        self.padding_box
    }

    #[must_use]
    pub(crate) const fn effective_border(self) -> Edges<S> {
        self.effective_border
    }

    #[must_use]
    pub(crate) const fn effective_padding(self) -> Edges<S> {
        self.effective_padding
    }

    #[must_use]
    pub(crate) const fn effective_gutter(self) -> Edges<S> {
        self.effective_gutter
    }

    #[must_use]
    pub(crate) const fn scrollport(self) -> ScrollRectOf<S> {
        self.scrollport
    }

    #[must_use]
    pub(crate) const fn content_box(self) -> ScrollRectOf<S> {
        self.content_box
    }

    #[must_use]
    pub(crate) fn content_box_inset(self) -> Edges<S> {
        self.effective_border + self.effective_gutter + self.effective_padding
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScrollBoxClipGutterResultOf<S: LayoutScalar> {
    border_box: ScrollRectOf<S>,
    padding_box: ScrollRectOf<S>,
    content_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
    effective_border: Edges<S>,
    effective_padding: Edges<S>,
    effective_reservation: Edges<S>,
    gutters: ScrollbarGutterRectsOf<S>,
    aggregate_reservation: Size<S>,
    overflow_clip: OverflowClipOf<S>,
    resolved_scroll_padding: Edges<S>,
    optimal_viewing_region: ScrollRectOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ScrollBoxClipGutterErrorOf<S: LayoutScalar> {
    Rect(ScrollRectErrorOf<S>),
    Clip(ScrollCoordinateErrorOf<S>),
    InvalidInset { side: PhysicalSide, value: S },
    InvalidClipMargin { value: S },
    InvalidPercentageBasis { axis: PhysicalAxis },
    InvalidOptimalRegionInset { side: PhysicalSide },
}

impl<S: LayoutScalar> From<ScrollRectErrorOf<S>> for ScrollBoxClipGutterErrorOf<S> {
    fn from(value: ScrollRectErrorOf<S>) -> Self {
        Self::Rect(value)
    }
}

fn derive_scroll_box_clip_gutter<S: LayoutScalar>(
    source: ScrollBoxClipGutterSourceOf<S>,
) -> Result<ScrollBoxClipGutterResultOf<S>, ScrollBoxClipGutterErrorOf<S>> {
    validate_insets(source.border)?;
    validate_insets(source.padding)?;
    if !source.clip_margin.margin.is_finite() || source.clip_margin.margin < S::ZERO {
        return Err(ScrollBoxClipGutterErrorOf::InvalidClipMargin {
            value: source.clip_margin.margin,
        });
    }

    let border_box = ScrollRectOf::try_new(Point::ZERO, source.border_box_size)?;
    let (effective_border, padding_box) = inset_scroll_rect_saturated(border_box, source.border)?;
    let scrollbar_state = EffectiveScrollbarState::derive(
        source.flow_axes,
        source.used_overflow,
        source.scrollbar_gutter,
        source.settled_auto_scrollbars,
    );
    let requested_reservation = PhysicalEdgeReservationOf::derive(
        source.flow_axes,
        scrollbar_state,
        source.scrollbar_width,
    );
    let (effective_reservation, scrollport) =
        inset_scroll_rect_saturated(padding_box, requested_reservation.requested)?;
    let (effective_padding, content_box) = inset_scroll_rect_saturated(scrollport, source.padding)?;
    let gutters = physical_gutter_rects(padding_box, scrollport, effective_reservation)?;
    let overflow_clip = derive_overflow_clip(
        source.used_overflow,
        source.clip_margin,
        border_box,
        padding_box,
        content_box,
        scrollport,
    )?;
    let resolved_scroll_padding =
        resolve_optimal_region_insets(source.optimal_region_insets, scrollport)?;
    let (_, optimal_viewing_region) =
        inset_scroll_rect_saturated(scrollport, resolved_scroll_padding)?;

    Ok(ScrollBoxClipGutterResultOf {
        border_box,
        padding_box,
        content_box,
        scrollport,
        effective_border,
        effective_padding,
        effective_reservation,
        gutters,
        aggregate_reservation: effective_reservation.sum_axes(),
        overflow_clip,
        resolved_scroll_padding,
        optimal_viewing_region,
    })
}

fn validate_insets<S: LayoutScalar>(edges: Edges<S>) -> Result<(), ScrollBoxClipGutterErrorOf<S>> {
    for (side, value) in [
        (PhysicalSide::Top, edges.top),
        (PhysicalSide::Right, edges.right),
        (PhysicalSide::Bottom, edges.bottom),
        (PhysicalSide::Left, edges.left),
    ] {
        if !value.is_finite() || value < S::ZERO {
            return Err(ScrollBoxClipGutterErrorOf::InvalidInset { side, value });
        }
    }
    Ok(())
}

fn inset_scroll_rect_saturated<S: LayoutScalar>(
    rect: ScrollRectOf<S>,
    requested: Edges<S>,
) -> Result<(Edges<S>, ScrollRectOf<S>), ScrollBoxClipGutterErrorOf<S>> {
    let size = rect.size();
    let (left, right) = saturate_opposing_edges(requested.left, requested.right, size.width);
    let (top, bottom) = saturate_opposing_edges(requested.top, requested.bottom, size.height);
    let effective = Edges::new(top, right, bottom, left);
    let origin = rect.origin();
    let inset = ScrollRectOf::try_new(
        Point::new(origin.x + left, origin.y + top),
        Size::new(
            (size.width - left - right).max(S::ZERO),
            (size.height - top - bottom).max(S::ZERO),
        ),
    )?;
    Ok((effective, inset))
}

fn saturate_opposing_edges<S: LayoutScalar>(start: S, end: S, dimension: S) -> (S, S) {
    if dimension == S::ZERO || start == S::ZERO && end == S::ZERO {
        return (S::ZERO, S::ZERO);
    }
    if start <= dimension && end <= dimension - start {
        return (canonical_scroll_zero(start), canonical_scroll_zero(end));
    }

    let largest = start.max(end);
    let start_share = start / largest;
    let end_share = end / largest;
    let effective_start = dimension * (start_share / (start_share + end_share));
    let effective_end = (dimension - effective_start).max(S::ZERO);
    (
        canonical_scroll_zero(effective_start),
        canonical_scroll_zero(effective_end),
    )
}

fn physical_gutter_rects<S: LayoutScalar>(
    padding_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
    reservation: Edges<S>,
) -> Result<ScrollbarGutterRectsOf<S>, ScrollBoxClipGutterErrorOf<S>> {
    Ok(ScrollbarGutterRectsOf {
        top: physical_gutter_rect(PhysicalSide::Top, reservation.top, padding_box, scrollport)?,
        right: physical_gutter_rect(
            PhysicalSide::Right,
            reservation.right,
            padding_box,
            scrollport,
        )?,
        bottom: physical_gutter_rect(
            PhysicalSide::Bottom,
            reservation.bottom,
            padding_box,
            scrollport,
        )?,
        left: physical_gutter_rect(
            PhysicalSide::Left,
            reservation.left,
            padding_box,
            scrollport,
        )?,
    })
}

fn physical_gutter_rect<S: LayoutScalar>(
    side: PhysicalSide,
    thickness: S,
    padding_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
) -> Result<Option<ScrollRectOf<S>>, ScrollBoxClipGutterErrorOf<S>> {
    if thickness == S::ZERO {
        return Ok(None);
    }

    let padding_origin = padding_box.origin();
    let padding_size = padding_box.size();
    let scrollport_origin = scrollport.origin();
    let scrollport_size = scrollport.size();
    let (origin, size) = match side {
        PhysicalSide::Top => (
            Point::new(scrollport_origin.x, padding_origin.y),
            Size::new(scrollport_size.width, thickness),
        ),
        PhysicalSide::Right => (
            Point::new(
                padding_origin.x + padding_size.width - thickness,
                scrollport_origin.y,
            ),
            Size::new(thickness, scrollport_size.height),
        ),
        PhysicalSide::Bottom => (
            Point::new(
                scrollport_origin.x,
                padding_origin.y + padding_size.height - thickness,
            ),
            Size::new(scrollport_size.width, thickness),
        ),
        PhysicalSide::Left => (
            Point::new(padding_origin.x, scrollport_origin.y),
            Size::new(thickness, scrollport_size.height),
        ),
    };
    Ok(Some(ScrollRectOf::try_new(origin, size)?))
}

fn derive_overflow_clip<S: LayoutScalar>(
    overflow: UsedOverflow,
    clip_margin: ClipMarginSourceOf<S>,
    border_box: ScrollRectOf<S>,
    padding_box: ScrollRectOf<S>,
    content_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
) -> Result<OverflowClipOf<S>, ScrollBoxClipGutterErrorOf<S>> {
    Ok(OverflowClipOf {
        x: derive_overflow_clip_axis(
            overflow.x(),
            PhysicalAxis::Horizontal,
            clip_margin,
            border_box,
            padding_box,
            content_box,
            scrollport,
        )?,
        y: derive_overflow_clip_axis(
            overflow.y(),
            PhysicalAxis::Vertical,
            clip_margin,
            border_box,
            padding_box,
            content_box,
            scrollport,
        )?,
    })
}

fn derive_overflow_clip_axis<S: LayoutScalar>(
    overflow: UsedOverflowAxis,
    axis: PhysicalAxis,
    clip_margin: ClipMarginSourceOf<S>,
    border_box: ScrollRectOf<S>,
    padding_box: ScrollRectOf<S>,
    content_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
) -> Result<Option<PhysicalClipAxisOf<S>>, ScrollBoxClipGutterErrorOf<S>> {
    let reference = match overflow.value() {
        Overflow::Visible => return Ok(None),
        Overflow::Clip => match clip_margin.reference_box {
            OverflowClipBox::ContentBox => content_box,
            OverflowClipBox::PaddingBox => padding_box,
            OverflowClipBox::BorderBox => border_box,
        },
        Overflow::Hidden | Overflow::Scroll | Overflow::Auto => scrollport,
    };
    let (mut minimum, mut maximum) = scroll_rect_axis_interval(reference, axis);
    if matches!(overflow.value(), Overflow::Clip) {
        minimum = minimum - clip_margin.margin;
        maximum = maximum + clip_margin.margin;
    }
    validate_physical_scroll_range(axis, minimum, maximum)
        .map_err(ScrollBoxClipGutterErrorOf::Clip)?;

    Ok(Some(PhysicalClipAxisOf {
        range: PhysicalScrollAxisRangeOf::new(
            canonical_scroll_zero(minimum),
            canonical_scroll_zero(maximum),
        ),
    }))
}

fn scroll_rect_axis_interval<S: LayoutScalar>(rect: ScrollRectOf<S>, axis: PhysicalAxis) -> (S, S) {
    match axis {
        PhysicalAxis::Horizontal => (rect.origin().x, rect.origin().x + rect.size().width),
        PhysicalAxis::Vertical => (rect.origin().y, rect.origin().y + rect.size().height),
    }
}

fn resolve_optimal_region_insets<S: LayoutScalar>(
    insets: OptimalRegionInsetsOf<S>,
    scrollport: ScrollRectOf<S>,
) -> Result<Edges<S>, ScrollBoxClipGutterErrorOf<S>> {
    let size = scrollport.size();
    let width_basis = PercentageBasisOf::definite(size.width).map_err(|_| {
        ScrollBoxClipGutterErrorOf::InvalidPercentageBasis {
            axis: PhysicalAxis::Horizontal,
        }
    })?;
    let height_basis = PercentageBasisOf::definite(size.height).map_err(|_| {
        ScrollBoxClipGutterErrorOf::InvalidPercentageBasis {
            axis: PhysicalAxis::Vertical,
        }
    })?;

    Ok(Edges::new(
        resolve_optimal_region_inset(insets.top, height_basis, PhysicalSide::Top)?,
        resolve_optimal_region_inset(insets.right, width_basis, PhysicalSide::Right)?,
        resolve_optimal_region_inset(insets.bottom, height_basis, PhysicalSide::Bottom)?,
        resolve_optimal_region_inset(insets.left, width_basis, PhysicalSide::Left)?,
    ))
}

fn resolve_optimal_region_inset<S: LayoutScalar>(
    value: OptimalRegionInsetOf<S>,
    basis: PercentageBasisOf<S>,
    side: PhysicalSide,
) -> Result<S, ScrollBoxClipGutterErrorOf<S>> {
    let resolution = match value {
        OptimalRegionInsetOf::Auto => NumericResolutionOf::Resolved(S::ZERO),
        OptimalRegionInsetOf::Value(value) => value.resolve_against(basis),
    };
    match resolution {
        NumericResolutionOf::Resolved(value) if value.is_finite() => {
            Ok(canonical_scroll_zero(value.max(S::ZERO)))
        }
        NumericResolutionOf::Resolved(_)
        | NumericResolutionOf::MissingBasis { .. }
        | NumericResolutionOf::InvalidNumeric { .. } => {
            Err(ScrollBoxClipGutterErrorOf::InvalidOptimalRegionInset { side })
        }
    }
}

type DefaultScrollBoxClipGutterDerivation = fn(
    ScrollBoxClipGutterSourceOf<DefaultScalar>,
) -> Result<
    ScrollBoxClipGutterResultOf<DefaultScalar>,
    ScrollBoxClipGutterErrorOf<DefaultScalar>,
>;
const _: DefaultScrollBoxClipGutterDerivation = derive_scroll_box_clip_gutter::<DefaultScalar>;
const _: OptimalRegionInsetOf<DefaultScalar> =
    OptimalRegionInsetOf::Value(LengthPercentageOf::ZERO);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PhysicalContributionIntervalOf<S: LayoutScalar> {
    minimum: S,
    maximum: S,
}

impl<S: LayoutScalar> PhysicalContributionIntervalOf<S> {
    fn from_rect(rect: ScrollRectOf<S>, axis: PhysicalAxis) -> Self {
        let (minimum, maximum) = scroll_rect_axis_interval(rect, axis);
        Self { minimum, maximum }
    }

    fn translated(
        self,
        axis: PhysicalAxis,
        offset: S,
    ) -> Result<Self, ScrollContributionErrorOf<S>> {
        let minimum = self.minimum + offset;
        let maximum = self.maximum + offset;
        validate_physical_scroll_range(axis, minimum, maximum)
            .map_err(ScrollContributionErrorOf::Coordinate)?;
        Ok(Self {
            minimum: canonical_scroll_zero(minimum),
            maximum: canonical_scroll_zero(maximum),
        })
    }

    fn include(&mut self, other: Self) {
        self.minimum = self.minimum.min(other.minimum);
        self.maximum = self.maximum.max(other.maximum);
    }

    #[must_use]
    pub(crate) const fn minimum(self) -> S {
        self.minimum
    }

    #[must_use]
    pub(crate) const fn maximum(self) -> S {
        self.maximum
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PhysicalContributionBoundsOf<S: LayoutScalar> {
    x: PhysicalContributionIntervalOf<S>,
    y: PhysicalContributionIntervalOf<S>,
}

impl<S: LayoutScalar> PhysicalContributionBoundsOf<S> {
    fn from_rect(rect: ScrollRectOf<S>) -> Self {
        Self {
            x: PhysicalContributionIntervalOf::from_rect(rect, PhysicalAxis::Horizontal),
            y: PhysicalContributionIntervalOf::from_rect(rect, PhysicalAxis::Vertical),
        }
    }

    fn include(&mut self, axis: PhysicalAxis, interval: PhysicalContributionIntervalOf<S>) {
        match axis {
            PhysicalAxis::Horizontal => self.x.include(interval),
            PhysicalAxis::Vertical => self.y.include(interval),
        }
    }

    #[must_use]
    const fn at(self, axis: PhysicalAxis) -> PhysicalContributionIntervalOf<S> {
        match axis {
            PhysicalAxis::Horizontal => self.x(),
            PhysicalAxis::Vertical => self.y(),
        }
    }

    #[must_use]
    const fn x(self) -> PhysicalContributionIntervalOf<S> {
        self.x
    }

    #[must_use]
    const fn y(self) -> PhysicalContributionIntervalOf<S> {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OptionalPhysicalContributionIntervalsOf<S: LayoutScalar> {
    x: Option<PhysicalContributionIntervalOf<S>>,
    y: Option<PhysicalContributionIntervalOf<S>>,
}

impl<S: LayoutScalar> OptionalPhysicalContributionIntervalsOf<S> {
    const NONE: Self = Self { x: None, y: None };

    fn include(&mut self, axis: PhysicalAxis, interval: PhysicalContributionIntervalOf<S>) {
        let destination = match axis {
            PhysicalAxis::Horizontal => &mut self.x,
            PhysicalAxis::Vertical => &mut self.y,
        };
        if let Some(existing) = destination {
            existing.include(interval);
        } else {
            *destination = Some(interval);
        }
    }

    fn set(&mut self, axis: PhysicalAxis, interval: PhysicalContributionIntervalOf<S>) {
        match axis {
            PhysicalAxis::Horizontal => self.x = Some(interval),
            PhysicalAxis::Vertical => self.y = Some(interval),
        }
    }

    #[must_use]
    pub(crate) const fn at(self, axis: PhysicalAxis) -> Option<PhysicalContributionIntervalOf<S>> {
        match axis {
            PhysicalAxis::Horizontal => self.x(),
            PhysicalAxis::Vertical => self.y(),
        }
    }

    #[must_use]
    pub(crate) const fn retain_physical_axes(self, horizontal: bool, vertical: bool) -> Self {
        Self {
            x: if horizontal { self.x } else { None },
            y: if vertical { self.y } else { None },
        }
    }

    #[must_use]
    const fn x(self) -> Option<PhysicalContributionIntervalOf<S>> {
        self.x
    }

    #[must_use]
    const fn y(self) -> Option<PhysicalContributionIntervalOf<S>> {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FinalInFlowEndOf<S: LayoutScalar> {
    side: PhysicalSide,
    coordinate: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PhysicalFinalInFlowEndsOf<S: LayoutScalar> {
    x: Option<FinalInFlowEndOf<S>>,
    y: Option<FinalInFlowEndOf<S>>,
}

impl<S: LayoutScalar> PhysicalFinalInFlowEndsOf<S> {
    const NONE: Self = Self { x: None, y: None };

    fn set(&mut self, end: FinalInFlowEndOf<S>) {
        match end.side.axis() {
            PhysicalAxis::Horizontal => self.x = Some(end),
            PhysicalAxis::Vertical => self.y = Some(end),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ScrollContributionErrorOf<S: LayoutScalar> {
    Rect(ScrollRectErrorOf<S>),
    Coordinate(ScrollCoordinateErrorOf<S>),
    NonFiniteMargin { side: PhysicalSide, value: S },
    NonFiniteFinalInFlowEnd { axis: LogicalAxis, value: S },
    InvalidTerminalPadding { side: PhysicalSide, value: S },
    NonFiniteTerminalEnd { side: PhysicalSide, value: S },
}

impl<S: LayoutScalar> From<ScrollRectErrorOf<S>> for ScrollContributionErrorOf<S> {
    fn from(value: ScrollRectErrorOf<S>) -> Self {
        Self::Rect(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContainerRangeBasis {
    PaddingBox,
    Scrollport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollContributionAccumulatorOf<S: LayoutScalar> {
    container_seed: PhysicalContributionBoundsOf<S>,
    container_range_basis: ContainerRangeBasis,
    propagatable_descendants: OptionalPhysicalContributionIntervalsOf<S>,
    final_in_flow_ends: PhysicalFinalInFlowEndsOf<S>,
    terminal_padding_overflow: OptionalPhysicalContributionIntervalsOf<S>,
    active_alignment_subjects: OptionalPhysicalContributionIntervalsOf<S>,
}

impl<S: LayoutScalar> ScrollContributionAccumulatorOf<S> {
    pub(crate) fn new(padding_box: ScrollRectOf<S>) -> Self {
        Self {
            container_seed: PhysicalContributionBoundsOf::from_rect(padding_box),
            container_range_basis: ContainerRangeBasis::PaddingBox,
            propagatable_descendants: OptionalPhysicalContributionIntervalsOf::NONE,
            final_in_flow_ends: PhysicalFinalInFlowEndsOf::NONE,
            terminal_padding_overflow: OptionalPhysicalContributionIntervalsOf::NONE,
            active_alignment_subjects: OptionalPhysicalContributionIntervalsOf::NONE,
        }
    }

    pub(crate) fn include_direct_line(&mut self, line: ScrollRectOf<S>) {
        self.include_descendant_rect(line);
    }

    pub(crate) fn include_in_flow_child(
        &mut self,
        child_location: Point<S>,
        child_border_box: ScrollRectOf<S>,
        margin: Edges<S>,
        child_descendants: OptionalPhysicalContributionIntervalsOf<S>,
        child_used_overflow: UsedOverflow,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        self.include_child(
            child_location,
            child_border_box,
            margin,
            child_descendants,
            child_used_overflow,
        )
    }

    pub(crate) fn include_in_flow_geometry(
        &mut self,
        child_location: Point<S>,
        margin: Edges<S>,
        child_geometry: ScrollGeometryOf<S>,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        self.include_in_flow_child(
            child_location,
            child_geometry.border_box(),
            margin,
            child_geometry
                .source
                .contributions
                .propagatable_descendant_intervals(),
            child_geometry.used_overflow,
        )
    }

    pub(crate) fn include_current_out_of_flow(
        &mut self,
        child_location: Point<S>,
        child_border_box: ScrollRectOf<S>,
        margin: Edges<S>,
        child_descendants: OptionalPhysicalContributionIntervalsOf<S>,
        child_used_overflow: UsedOverflow,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        self.include_child(
            child_location,
            child_border_box,
            margin,
            child_descendants,
            child_used_overflow,
        )
    }

    pub(crate) fn include_current_out_of_flow_geometry(
        &mut self,
        child_location: Point<S>,
        margin: Edges<S>,
        child_geometry: ScrollGeometryOf<S>,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        self.include_current_out_of_flow(
            child_location,
            child_geometry.border_box(),
            margin,
            child_geometry
                .source
                .contributions
                .propagatable_descendant_intervals(),
            child_geometry.used_overflow,
        )
    }

    fn include_child(
        &mut self,
        child_location: Point<S>,
        child_border_box: ScrollRectOf<S>,
        margin: Edges<S>,
        child_descendants: OptionalPhysicalContributionIntervalsOf<S>,
        child_used_overflow: UsedOverflow,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        let mut next = *self;
        let border_size = child_border_box.size();
        if border_size.width > S::ZERO && border_size.height > S::ZERO {
            let contribution =
                child_margin_contribution_rect(child_location, child_border_box, margin)?;
            next.include_descendant_rect(contribution);
        }

        for axis in [PhysicalAxis::Horizontal, PhysicalAxis::Vertical] {
            if matches!(
                used_overflow_at(child_used_overflow, axis).value(),
                Overflow::Visible
            ) && let Some(interval) = child_descendants.at(axis)
            {
                let offset = match axis {
                    PhysicalAxis::Horizontal => child_location.x,
                    PhysicalAxis::Vertical => child_location.y,
                };
                next.include_descendant_interval(axis, interval.translated(axis, offset)?);
            }
        }

        *self = next;
        Ok(())
    }

    fn include_descendant_rect(&mut self, rect: ScrollRectOf<S>) {
        for axis in [PhysicalAxis::Horizontal, PhysicalAxis::Vertical] {
            self.include_descendant_interval(
                axis,
                PhysicalContributionIntervalOf::from_rect(rect, axis),
            );
        }
    }

    fn include_descendant_interval(
        &mut self,
        axis: PhysicalAxis,
        interval: PhysicalContributionIntervalOf<S>,
    ) {
        self.propagatable_descendants.include(axis, interval);
    }

    pub(crate) fn replace_container_seed(&mut self, container_seed: ScrollRectOf<S>) {
        self.container_seed = PhysicalContributionBoundsOf::from_rect(container_seed);
    }

    pub(crate) fn exclude_reserved_gutter_from_range(&mut self) {
        self.container_range_basis = ContainerRangeBasis::Scrollport;
    }

    pub(crate) fn record_final_in_flow_end(
        &mut self,
        flow_axes: FlowAxes,
        axis: LogicalAxis,
        coordinate: S,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        if !coordinate.is_finite() {
            return Err(ScrollContributionErrorOf::NonFiniteFinalInFlowEnd {
                axis,
                value: coordinate,
            });
        }
        let side = match axis {
            LogicalAxis::Inline => flow_axes.inline_end(),
            LogicalAxis::Block => flow_axes.block_end(),
        };
        self.final_in_flow_ends.set(FinalInFlowEndOf {
            side,
            coordinate: canonical_scroll_zero(coordinate),
        });
        Ok(())
    }

    pub(crate) fn include_terminal_padding(
        &mut self,
        padding: Edges<S>,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        for (side, value) in [
            (PhysicalSide::Top, padding.top),
            (PhysicalSide::Right, padding.right),
            (PhysicalSide::Bottom, padding.bottom),
            (PhysicalSide::Left, padding.left),
        ] {
            if !value.is_finite() || value < S::ZERO {
                return Err(ScrollContributionErrorOf::InvalidTerminalPadding { side, value });
            }
        }

        let mut terminal_padding_overflow = OptionalPhysicalContributionIntervalsOf::NONE;
        for end in [self.final_in_flow_ends.x, self.final_in_flow_ends.y]
            .into_iter()
            .flatten()
        {
            let padding = physical_edge_value(padding, end.side);
            let coordinate = match end.side {
                PhysicalSide::Top | PhysicalSide::Left => end.coordinate - padding,
                PhysicalSide::Right | PhysicalSide::Bottom => end.coordinate + padding,
            };
            if !coordinate.is_finite() {
                return Err(ScrollContributionErrorOf::NonFiniteTerminalEnd {
                    side: end.side,
                    value: coordinate,
                });
            }
            terminal_padding_overflow.include(
                end.side.axis(),
                PhysicalContributionIntervalOf {
                    minimum: canonical_scroll_zero(coordinate),
                    maximum: canonical_scroll_zero(coordinate),
                },
            );
        }
        self.terminal_padding_overflow = terminal_padding_overflow;
        Ok(())
    }

    pub(crate) fn set_active_alignment_subject(
        &mut self,
        axis: PhysicalAxis,
        subject: ScrollRectOf<S>,
    ) {
        self.active_alignment_subjects.set(
            axis,
            PhysicalContributionIntervalOf::from_rect(subject, axis),
        );
    }

    #[must_use]
    fn complete_overflow(self) -> PhysicalContributionBoundsOf<S> {
        self.overflow_from_container_seed(self.container_seed)
    }

    #[must_use]
    fn range_overflow(self, scrollport: ScrollRectOf<S>) -> PhysicalContributionBoundsOf<S> {
        let container_seed = match self.container_range_basis {
            ContainerRangeBasis::PaddingBox => self.container_seed,
            ContainerRangeBasis::Scrollport => PhysicalContributionBoundsOf::from_rect(scrollport),
        };
        self.overflow_from_container_seed(container_seed)
    }

    fn overflow_from_container_seed(
        self,
        mut overflow: PhysicalContributionBoundsOf<S>,
    ) -> PhysicalContributionBoundsOf<S> {
        for axis in [PhysicalAxis::Horizontal, PhysicalAxis::Vertical] {
            if let Some(interval) = self.propagatable_descendants.at(axis) {
                overflow.include(axis, interval);
            }
            if let Some(interval) = self.terminal_padding_overflow.at(axis) {
                overflow.include(axis, interval);
            }
        }
        overflow
    }

    #[must_use]
    pub(crate) const fn propagatable_descendant_intervals(
        self,
    ) -> OptionalPhysicalContributionIntervalsOf<S> {
        self.propagatable_descendants
    }

    #[must_use]
    pub(crate) const fn active_alignment_subject_intervals(
        self,
    ) -> OptionalPhysicalContributionIntervalsOf<S> {
        self.active_alignment_subjects
    }

    pub(crate) fn content_size_from_anchor(
        self,
        anchor: Point<S>,
    ) -> Result<Size<S>, ScrollContributionErrorOf<S>> {
        let complete_overflow = self.complete_overflow();
        let x = complete_overflow.x();
        let y = complete_overflow.y();
        let minimum = Point::new(anchor.x.min(x.minimum()), anchor.y.min(y.minimum()));
        let maximum = Point::new(anchor.x.max(x.maximum()), anchor.y.max(y.maximum()));
        Ok(ScrollRectOf::try_new(
            minimum,
            Size::new(maximum.x - minimum.x, maximum.y - minimum.y),
        )?
        .size())
    }
}

fn child_margin_contribution_rect<S: LayoutScalar>(
    child_location: Point<S>,
    child_border_box: ScrollRectOf<S>,
    margin: Edges<S>,
) -> Result<ScrollRectOf<S>, ScrollContributionErrorOf<S>> {
    for (side, value) in [
        (PhysicalSide::Top, margin.top),
        (PhysicalSide::Right, margin.right),
        (PhysicalSide::Bottom, margin.bottom),
        (PhysicalSide::Left, margin.left),
    ] {
        if !value.is_finite() {
            return Err(ScrollContributionErrorOf::NonFiniteMargin { side, value });
        }
    }

    let top = margin.top.max(S::ZERO);
    let right = margin.right.max(S::ZERO);
    let bottom = margin.bottom.max(S::ZERO);
    let left = margin.left.max(S::ZERO);
    let border_origin = child_border_box.origin();
    let border_size = child_border_box.size();
    Ok(ScrollRectOf::try_new(
        Point::new(
            child_location.x + border_origin.x - left,
            child_location.y + border_origin.y - top,
        ),
        Size::new(
            border_size.width + left + right,
            border_size.height + top + bottom,
        ),
    )?)
}

fn physical_edge_value<S: LayoutScalar>(edges: Edges<S>, side: PhysicalSide) -> S {
    match side {
        PhysicalSide::Top => edges.top,
        PhysicalSide::Right => edges.right,
        PhysicalSide::Bottom => edges.bottom,
        PhysicalSide::Left => edges.left,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScrollOriginProgression {
    FlowEndward,
    FlowStartward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScrollOriginAxes {
    inline: ScrollOriginProgression,
    block: ScrollOriginProgression,
}

impl ScrollOriginAxes {
    pub(crate) const fn new(
        inline: ScrollOriginProgression,
        block: ScrollOriginProgression,
    ) -> Self {
        Self { inline, block }
    }

    const fn at(self, axis: LogicalAxis) -> ScrollOriginProgression {
        match axis {
            LogicalAxis::Inline => self.inline,
            LogicalAxis::Block => self.block,
        }
    }
}

fn derive_origin_aware_scroll_range<S: LayoutScalar>(
    flow_axes: FlowAxes,
    origin_axes: ScrollOriginAxes,
    used_overflow: UsedOverflow,
    scrollport: ScrollRectOf<S>,
    contributions: &ScrollContributionAccumulatorOf<S>,
) -> Result<PhysicalScrollRangeOf<S>, ScrollCoordinateErrorOf<S>> {
    let range_overflow = contributions.range_overflow(scrollport);
    let inline = derive_origin_aware_axis_range(
        flow_axes,
        origin_axes,
        LogicalAxis::Inline,
        used_overflow,
        scrollport,
        range_overflow,
        contributions.active_alignment_subject_intervals(),
    );
    let block = derive_origin_aware_axis_range(
        flow_axes,
        origin_axes,
        LogicalAxis::Block,
        used_overflow,
        scrollport,
        range_overflow,
        contributions.active_alignment_subject_intervals(),
    );
    let flow_relative = FlowRelativeScrollRangeOf::try_new(inline.0, inline.1, block.0, block.1)?;
    Ok(flow_axes.physical_scroll_range(flow_relative))
}

fn derive_origin_aware_axis_range<S: LayoutScalar>(
    flow_axes: FlowAxes,
    origin_axes: ScrollOriginAxes,
    logical_axis: LogicalAxis,
    used_overflow: UsedOverflow,
    scrollport: ScrollRectOf<S>,
    complete_overflow: PhysicalContributionBoundsOf<S>,
    active_subjects: OptionalPhysicalContributionIntervalsOf<S>,
) -> (S, S) {
    let physical_axis = match logical_axis {
        LogicalAxis::Inline => flow_axes.inline_axis(),
        LogicalAxis::Block => flow_axes.block_axis(),
    };
    if !used_overflow_at(used_overflow, physical_axis).exposes_scroll_range() {
        return (S::ZERO, S::ZERO);
    }

    let progression = origin_axes.at(logical_axis);
    let origin_decreasing = flow_axes
        .logical_axis_progression(logical_axis)
        .is_decreasing()
        ^ matches!(progression, ScrollOriginProgression::FlowStartward);
    let overflow = complete_overflow.at(physical_axis);
    let scrollport = PhysicalContributionIntervalOf::from_rect(scrollport, physical_axis);
    let active_subject = active_subjects.at(physical_axis);
    let (end_extent, start_extent) = if origin_decreasing {
        (
            (scrollport.minimum() - overflow.minimum()).max(S::ZERO),
            active_subject
                .map(|subject| (subject.maximum() - scrollport.maximum()).max(S::ZERO))
                .unwrap_or(S::ZERO),
        )
    } else {
        (
            (overflow.maximum() - scrollport.maximum()).max(S::ZERO),
            active_subject
                .map(|subject| (scrollport.minimum() - subject.minimum()).max(S::ZERO))
                .unwrap_or(S::ZERO),
        )
    };

    match progression {
        ScrollOriginProgression::FlowEndward => (-start_extent, end_extent),
        ScrollOriginProgression::FlowStartward => (-end_extent, start_extent),
    }
}

type DefaultContributionConstructor =
    fn(ScrollRectOf<DefaultScalar>) -> ScrollContributionAccumulatorOf<DefaultScalar>;
const _: DefaultContributionConstructor = ScrollContributionAccumulatorOf::new;
type DefaultDirectLineOperation =
    fn(&mut ScrollContributionAccumulatorOf<DefaultScalar>, ScrollRectOf<DefaultScalar>);
const _: DefaultDirectLineOperation = ScrollContributionAccumulatorOf::include_direct_line;
type DefaultChildContributionOperation = fn(
    &mut ScrollContributionAccumulatorOf<DefaultScalar>,
    Point<DefaultScalar>,
    ScrollRectOf<DefaultScalar>,
    Edges<DefaultScalar>,
    OptionalPhysicalContributionIntervalsOf<DefaultScalar>,
    UsedOverflow,
)
    -> Result<(), ScrollContributionErrorOf<DefaultScalar>>;
const _: DefaultChildContributionOperation = ScrollContributionAccumulatorOf::include_in_flow_child;
const _: DefaultChildContributionOperation =
    ScrollContributionAccumulatorOf::include_current_out_of_flow;
type DefaultFinalInFlowEndOperation = fn(
    &mut ScrollContributionAccumulatorOf<DefaultScalar>,
    FlowAxes,
    LogicalAxis,
    DefaultScalar,
) -> Result<(), ScrollContributionErrorOf<DefaultScalar>>;
const _: DefaultFinalInFlowEndOperation = ScrollContributionAccumulatorOf::record_final_in_flow_end;
type DefaultTerminalPaddingOperation = fn(
    &mut ScrollContributionAccumulatorOf<DefaultScalar>,
    Edges<DefaultScalar>,
) -> Result<(), ScrollContributionErrorOf<DefaultScalar>>;
const _: DefaultTerminalPaddingOperation =
    ScrollContributionAccumulatorOf::include_terminal_padding;
type DefaultAlignmentSubjectOperation = fn(
    &mut ScrollContributionAccumulatorOf<DefaultScalar>,
    PhysicalAxis,
    ScrollRectOf<DefaultScalar>,
);
const _: DefaultAlignmentSubjectOperation =
    ScrollContributionAccumulatorOf::set_active_alignment_subject;
type DefaultPropagatableIntervalsOperation =
    fn(
        ScrollContributionAccumulatorOf<DefaultScalar>,
    ) -> OptionalPhysicalContributionIntervalsOf<DefaultScalar>;
const _: DefaultPropagatableIntervalsOperation =
    ScrollContributionAccumulatorOf::propagatable_descendant_intervals;
type DefaultOriginAwareRangeDerivation =
    fn(
        FlowAxes,
        ScrollOriginAxes,
        UsedOverflow,
        ScrollRectOf<DefaultScalar>,
        &ScrollContributionAccumulatorOf<DefaultScalar>,
    ) -> Result<PhysicalScrollRangeOf<DefaultScalar>, ScrollCoordinateErrorOf<DefaultScalar>>;
const _: DefaultOriginAwareRangeDerivation = derive_origin_aware_scroll_range;
const _: ScrollOriginAxes = ScrollOriginAxes::new(
    ScrollOriginProgression::FlowEndward,
    ScrollOriginProgression::FlowStartward,
);

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalScrollGeometrySourceOf<S: LayoutScalar> {
    pub(crate) flow_axes: FlowAxes,
    pub(crate) computed_overflow: ComputedOverflow,
    pub(crate) item_is_replaced: bool,
    pub(crate) border_box_size: Size<S>,
    pub(crate) border: Edges<S>,
    pub(crate) padding: Edges<S>,
    pub(crate) scrollbar_gutter: ScrollbarGutter,
    pub(crate) scrollbar_width: ScrollbarWidthOf<S>,
    pub(crate) settled_auto_scrollbars: SettledAutoScrollbarState,
    pub(crate) clip_margin: ClipMarginSourceOf<S>,
    pub(crate) scroll_padding: OptimalRegionInsetsOf<S>,
    pub(crate) contributions: ScrollContributionAccumulatorOf<S>,
    pub(crate) origin_axes: ScrollOriginAxes,
    pub(crate) scroll_snap_type: ScrollSnapType,
    pub(crate) target_border_box: ScrollRectOf<S>,
    pub(crate) target_scroll_margin: ScrollMarginOf<S>,
    pub(crate) target_flow_axes: FlowAxes,
    pub(crate) target_snap_align: ScrollSnapAlign,
    pub(crate) target_snap_stop: ScrollSnapStop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasuredLeafScrollGeometrySourceOf<S: LayoutScalar> {
    pub(crate) flow_axes: FlowAxes,
    pub(crate) computed_overflow: ComputedOverflow,
    pub(crate) item_is_replaced: bool,
    pub(crate) border_box_size: Size<S>,
    pub(crate) border: Edges<S>,
    pub(crate) padding: Edges<S>,
    pub(crate) scrollbar_gutter: ScrollbarGutter,
    pub(crate) scrollbar_width: ScrollbarWidthOf<S>,
    pub(crate) settled_auto_scrollbars: SettledAutoScrollbarState,
    pub(crate) clip_margin: ClipMarginSourceOf<S>,
    pub(crate) scroll_padding: OptimalRegionInsetsOf<S>,
    pub(crate) measured_content_size: Size<S>,
    pub(crate) scroll_snap_type: ScrollSnapType,
    pub(crate) target_scroll_margin: ScrollMarginOf<S>,
    pub(crate) target_snap_align: ScrollSnapAlign,
    pub(crate) target_snap_stop: ScrollSnapStop,
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
    source: CanonicalScrollGeometrySourceOf<S>,
    flow_axes: FlowAxes,
    used_overflow: UsedOverflow,
    border_box: ScrollRectOf<S>,
    padding_box: ScrollRectOf<S>,
    content_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
    overflow_clip: OverflowClipOf<S>,
    scrollable_overflow: ScrollRectOf<S>,
    physical_range: PhysicalScrollRangeOf<S>,
    auto_scrollbar_observation: AutoScrollbarOverflowObservation,
    gutters: ScrollbarGutterRectsOf<S>,
    aggregate_reservation: Size<S>,
    resolved_scroll_padding: Edges<S>,
    optimal_viewing_region: ScrollRectOf<S>,
    scroll_snap_type: ScrollSnapType,
    target: ScrollTargetGeometryOf<S>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalScrollRectFact {
    BorderBox,
    TargetBorderBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CanonicalScrollGeometryErrorOf<S: LayoutScalar> {
    BoxClipGutter(ScrollBoxClipGutterErrorOf<S>),
    Contribution(ScrollContributionErrorOf<S>),
    ScrollableOverflow(ScrollRectErrorOf<S>),
    Range(ScrollCoordinateErrorOf<S>),
    RoundedRect {
        fact: CanonicalScrollRectFact,
        source: ScrollRectErrorOf<S>,
    },
    RoundedContribution(ScrollCoordinateErrorOf<S>),
    RoundedFinalInFlowEnd {
        side: PhysicalSide,
        value: S,
    },
    RoundedScrollbarWidth {
        value: S,
    },
    RoundedOptimalRegionInset {
        side: PhysicalSide,
        value: S,
    },
}

pub(crate) fn canonical_scroll_box_from_source<S: LayoutScalar>(
    source: CanonicalScrollBoxSourceOf<S>,
) -> Result<CanonicalScrollBoxOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let used_overflow =
        UsedOverflow::from_computed(source.computed_overflow, source.item_is_replaced);
    let boxes = derive_scroll_box_clip_gutter(ScrollBoxClipGutterSourceOf {
        flow_axes: source.flow_axes,
        used_overflow,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::default(),
        optimal_region_insets: OptimalRegionInsetsOf::default(),
    })
    .map_err(CanonicalScrollGeometryErrorOf::BoxClipGutter)?;

    Ok(CanonicalScrollBoxOf {
        border_box: boxes.border_box,
        padding_box: boxes.padding_box,
        effective_border: boxes.effective_border,
        effective_padding: boxes.effective_padding,
        effective_gutter: boxes.effective_reservation,
        scrollport: boxes.scrollport,
        content_box: boxes.content_box,
    })
}

pub(crate) fn settled_auto_scrollbars_change_available_geometry<S: LayoutScalar>(
    geometry: ScrollGeometryOf<S>,
    next_state: SettledAutoScrollbarState,
) -> Result<bool, CanonicalScrollGeometryErrorOf<S>> {
    let source = geometry.source;
    let box_source = |settled_auto_scrollbars| CanonicalScrollBoxSourceOf {
        flow_axes: source.flow_axes,
        computed_overflow: source.computed_overflow,
        item_is_replaced: source.item_is_replaced,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars,
    };
    let current = canonical_scroll_box_from_source(box_source(source.settled_auto_scrollbars))?;
    let prospective = canonical_scroll_box_from_source(box_source(next_state))?;

    Ok(current.effective_gutter() != prospective.effective_gutter()
        || current.content_box() != prospective.content_box())
}

pub(crate) fn rebuild_canonical_scroll_geometry_for_border_box<S: LayoutScalar>(
    geometry: ScrollGeometryOf<S>,
    border_box_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let mut source = geometry.source;
    let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
        flow_axes: source.flow_axes,
        computed_overflow: source.computed_overflow,
        item_is_replaced: source.item_is_replaced,
        border_box_size,
        border,
        padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
    })?;
    source.border_box_size = border_box_size;
    source.border = border;
    source.padding = padding;
    source
        .contributions
        .replace_container_seed(scroll_box.padding_box());
    source
        .contributions
        .include_terminal_padding(padding)
        .map_err(CanonicalScrollGeometryErrorOf::Contribution)?;
    source.target_border_box = scroll_box.border_box();
    canonical_scroll_geometry_from_source(source)
}

pub(crate) fn canonical_scroll_geometry_from_source<S: LayoutScalar>(
    source: CanonicalScrollGeometrySourceOf<S>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let used_overflow =
        UsedOverflow::from_computed(source.computed_overflow, source.item_is_replaced);
    let boxes = derive_scroll_box_clip_gutter(ScrollBoxClipGutterSourceOf {
        flow_axes: source.flow_axes,
        used_overflow,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
        clip_margin: source.clip_margin,
        optimal_region_insets: source.scroll_padding,
    })
    .map_err(CanonicalScrollGeometryErrorOf::BoxClipGutter)?;

    let complete = source.contributions.complete_overflow();
    let scrollable_overflow = ScrollRectOf::try_new(
        Point::new(complete.x().minimum(), complete.y().minimum()),
        Size::new(
            complete.x().maximum() - complete.x().minimum(),
            complete.y().maximum() - complete.y().minimum(),
        ),
    )
    .map_err(CanonicalScrollGeometryErrorOf::ScrollableOverflow)?;
    let physical_range = derive_origin_aware_scroll_range(
        source.flow_axes,
        source.origin_axes,
        used_overflow,
        boxes.scrollport,
        &source.contributions,
    )
    .map_err(CanonicalScrollGeometryErrorOf::Range)?;
    let mut auto_contributions = source.contributions;
    auto_contributions.replace_container_seed(boxes.scrollport);
    let auto_range = derive_origin_aware_scroll_range(
        source.flow_axes,
        source.origin_axes,
        used_overflow,
        boxes.scrollport,
        &auto_contributions,
    )
    .map_err(CanonicalScrollGeometryErrorOf::Range)?;
    let auto_scrollbar_observation = AutoScrollbarOverflowObservation::from_range(auto_range);
    let target = ScrollTargetGeometryOf {
        border_box: source.target_border_box,
        scroll_margin: source.target_scroll_margin,
        flow_axes: source.target_flow_axes,
        snap_align: source.target_snap_align,
        snap_stop: source.target_snap_stop,
    };

    Ok(ScrollGeometryOf {
        source,
        flow_axes: source.flow_axes,
        used_overflow,
        border_box: boxes.border_box,
        padding_box: boxes.padding_box,
        content_box: boxes.content_box,
        scrollport: boxes.scrollport,
        overflow_clip: boxes.overflow_clip,
        scrollable_overflow,
        physical_range,
        auto_scrollbar_observation,
        gutters: boxes.gutters,
        aggregate_reservation: boxes.aggregate_reservation,
        resolved_scroll_padding: boxes.resolved_scroll_padding,
        optimal_viewing_region: boxes.optimal_viewing_region,
        scroll_snap_type: source.scroll_snap_type,
        target,
    })
}

pub(crate) fn canonical_measured_leaf_scroll_geometry<S: LayoutScalar>(
    source: MeasuredLeafScrollGeometrySourceOf<S>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let used_overflow =
        UsedOverflow::from_computed(source.computed_overflow, source.item_is_replaced);
    let boxes = derive_scroll_box_clip_gutter(ScrollBoxClipGutterSourceOf {
        flow_axes: source.flow_axes,
        used_overflow,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
        clip_margin: source.clip_margin,
        optimal_region_insets: source.scroll_padding,
    })
    .map_err(CanonicalScrollGeometryErrorOf::BoxClipGutter)?;
    let measured_content = measured_leaf_content_rect(
        source.flow_axes,
        boxes.content_box,
        source.measured_content_size,
    )
    .map_err(CanonicalScrollGeometryErrorOf::ScrollableOverflow)?;
    let mut contributions = ScrollContributionAccumulatorOf::new(boxes.padding_box);
    contributions.include_direct_line(measured_content);
    for axis in [LogicalAxis::Inline, LogicalAxis::Block] {
        let side = match axis {
            LogicalAxis::Inline => source.flow_axes.inline_end(),
            LogicalAxis::Block => source.flow_axes.block_end(),
        };
        let coordinate = match side {
            PhysicalSide::Top => measured_content.origin().y,
            PhysicalSide::Right => measured_content.origin().x + measured_content.size().width,
            PhysicalSide::Bottom => measured_content.origin().y + measured_content.size().height,
            PhysicalSide::Left => measured_content.origin().x,
        };
        contributions
            .record_final_in_flow_end(source.flow_axes, axis, coordinate)
            .map_err(CanonicalScrollGeometryErrorOf::Contribution)?;
    }
    contributions
        .include_terminal_padding(source.padding)
        .map_err(CanonicalScrollGeometryErrorOf::Contribution)?;

    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes: source.flow_axes,
        computed_overflow: source.computed_overflow,
        item_is_replaced: source.item_is_replaced,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
        clip_margin: source.clip_margin,
        scroll_padding: source.scroll_padding,
        contributions,
        origin_axes: ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
        scroll_snap_type: source.scroll_snap_type,
        target_border_box: boxes.border_box,
        target_scroll_margin: source.target_scroll_margin,
        target_flow_axes: source.flow_axes,
        target_snap_align: source.target_snap_align,
        target_snap_stop: source.target_snap_stop,
    })
}

fn measured_leaf_content_rect<S: LayoutScalar>(
    flow_axes: FlowAxes,
    content_box: ScrollRectOf<S>,
    measured_content_size: Size<S>,
) -> Result<ScrollRectOf<S>, ScrollRectErrorOf<S>> {
    let origin = content_box.origin();
    let size = content_box.size();
    let x_start = if flow_axes.inline_axis() == PhysicalAxis::Horizontal {
        flow_axes.inline_start()
    } else {
        flow_axes.block_start()
    };
    let y_start = if flow_axes.inline_axis() == PhysicalAxis::Vertical {
        flow_axes.inline_start()
    } else {
        flow_axes.block_start()
    };
    ScrollRectOf::try_new(
        Point::new(
            match x_start {
                PhysicalSide::Left => origin.x,
                PhysicalSide::Right => origin.x + size.width - measured_content_size.width,
                PhysicalSide::Top | PhysicalSide::Bottom => {
                    unreachable!("physical x start side must be left or right")
                }
            },
            match y_start {
                PhysicalSide::Top => origin.y,
                PhysicalSide::Bottom => origin.y + size.height - measured_content_size.height,
                PhysicalSide::Right | PhysicalSide::Left => {
                    unreachable!("physical y start side must be top or bottom")
                }
            },
        ),
        measured_content_size,
    )
}

pub(crate) fn rebuild_rounded_canonical_scroll_geometry<S: LayoutScalar>(
    geometry: ScrollGeometryOf<S>,
    cumulative_origin: Point<S>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let source = geometry.source;
    let original_border_box =
        ScrollRectOf::try_new(Point::ZERO, source.border_box_size).map_err(|error| {
            CanonicalScrollGeometryErrorOf::RoundedRect {
                fact: CanonicalScrollRectFact::BorderBox,
                source: error,
            }
        })?;
    let rounded_border_box = round_canonical_source_rect(original_border_box, cumulative_origin)
        .map_err(|error| CanonicalScrollGeometryErrorOf::RoundedRect {
            fact: CanonicalScrollRectFact::BorderBox,
            source: error,
        })?;
    let rounded_scrollbar_width = round(source.scrollbar_width.get());
    let scrollbar_width = ScrollbarWidthOf::try_new(rounded_scrollbar_width).map_err(|_| {
        CanonicalScrollGeometryErrorOf::RoundedScrollbarWidth {
            value: rounded_scrollbar_width,
        }
    })?;
    let target_border_box =
        round_canonical_source_rect(source.target_border_box, cumulative_origin).map_err(
            |error| CanonicalScrollGeometryErrorOf::RoundedRect {
                fact: CanonicalScrollRectFact::TargetBorderBox,
                source: error,
            },
        )?;
    let scrollport_origin = geometry.scrollport.origin();
    let scroll_padding = round_canonical_scroll_padding(
        geometry.resolved_scroll_padding,
        geometry.scrollport.size(),
        Point::new(
            cumulative_origin.x + scrollport_origin.x,
            cumulative_origin.y + scrollport_origin.y,
        ),
    )?;
    let rounded_source = CanonicalScrollGeometrySourceOf {
        border_box_size: rounded_border_box.size(),
        border: round_canonical_source_edges(
            source.border,
            source.border_box_size,
            cumulative_origin,
        ),
        padding: round_canonical_source_edges(
            source.padding,
            geometry.scrollport.size(),
            Point::new(
                cumulative_origin.x + scrollport_origin.x,
                cumulative_origin.y + scrollport_origin.y,
            ),
        ),
        scrollbar_width,
        clip_margin: ClipMarginSourceOf::new(
            source.clip_margin.reference_box,
            round(source.clip_margin.margin),
        ),
        scroll_padding,
        contributions: round_canonical_contributions(source.contributions, cumulative_origin)?,
        target_border_box,
        ..source
    };

    canonical_scroll_geometry_from_source(rounded_source)
}

fn round_canonical_source_rect<S: LayoutScalar>(
    rect: ScrollRectOf<S>,
    cumulative_origin: Point<S>,
) -> Result<ScrollRectOf<S>, ScrollRectErrorOf<S>> {
    let origin = rect.origin();
    let size = rect.size();
    let rounded_origin = Point::new(
        round_canonical_source_coordinate(origin.x, cumulative_origin.x),
        round_canonical_source_coordinate(origin.y, cumulative_origin.y),
    );
    let rounded_end = Point::new(
        round_canonical_source_coordinate(origin.x + size.width, cumulative_origin.x),
        round_canonical_source_coordinate(origin.y + size.height, cumulative_origin.y),
    );
    ScrollRectOf::try_new(
        rounded_origin,
        Size::new(
            (rounded_end.x - rounded_origin.x).max(S::ZERO),
            (rounded_end.y - rounded_origin.y).max(S::ZERO),
        ),
    )
}

fn round_canonical_source_edges<S: LayoutScalar>(
    edges: Edges<S>,
    border_box_size: Size<S>,
    cumulative_origin: Point<S>,
) -> Edges<S> {
    Edges::new(
        round_canonical_source_coordinate(edges.top, cumulative_origin.y),
        canonical_scroll_zero(
            round(cumulative_origin.x + border_box_size.width)
                - round(cumulative_origin.x + border_box_size.width - edges.right),
        ),
        canonical_scroll_zero(
            round(cumulative_origin.y + border_box_size.height)
                - round(cumulative_origin.y + border_box_size.height - edges.bottom),
        ),
        round_canonical_source_coordinate(edges.left, cumulative_origin.x),
    )
}

fn round_canonical_scroll_padding<S: LayoutScalar>(
    resolved: Edges<S>,
    scrollport_size: Size<S>,
    cumulative_scrollport_origin: Point<S>,
) -> Result<OptimalRegionInsetsOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let rounded =
        round_canonical_source_edges(resolved, scrollport_size, cumulative_scrollport_origin);
    let value = |side, value| {
        LengthPercentageOf::px(value)
            .map(OptimalRegionInsetOf::Value)
            .map_err(|_| CanonicalScrollGeometryErrorOf::RoundedOptimalRegionInset { side, value })
    };
    Ok(OptimalRegionInsetsOf::new(
        value(PhysicalSide::Top, rounded.top)?,
        value(PhysicalSide::Right, rounded.right)?,
        value(PhysicalSide::Bottom, rounded.bottom)?,
        value(PhysicalSide::Left, rounded.left)?,
    ))
}

fn round_canonical_contributions<S: LayoutScalar>(
    contributions: ScrollContributionAccumulatorOf<S>,
    cumulative_origin: Point<S>,
) -> Result<ScrollContributionAccumulatorOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let container_seed = PhysicalContributionBoundsOf {
        x: round_canonical_interval(
            contributions.container_seed.x,
            PhysicalAxis::Horizontal,
            cumulative_origin,
        )?,
        y: round_canonical_interval(
            contributions.container_seed.y,
            PhysicalAxis::Vertical,
            cumulative_origin,
        )?,
    };
    let propagatable_descendants = round_canonical_optional_intervals(
        contributions.propagatable_descendants,
        cumulative_origin,
    )?;
    let active_alignment_subjects = round_canonical_optional_intervals(
        contributions.active_alignment_subjects,
        cumulative_origin,
    )?;
    let terminal_padding_overflow = round_canonical_optional_intervals(
        contributions.terminal_padding_overflow,
        cumulative_origin,
    )?;
    let final_in_flow_ends = PhysicalFinalInFlowEndsOf {
        x: round_canonical_final_in_flow_end(
            contributions.final_in_flow_ends.x,
            cumulative_origin,
        )?,
        y: round_canonical_final_in_flow_end(
            contributions.final_in_flow_ends.y,
            cumulative_origin,
        )?,
    };

    Ok(ScrollContributionAccumulatorOf {
        container_seed,
        container_range_basis: contributions.container_range_basis,
        propagatable_descendants,
        final_in_flow_ends,
        terminal_padding_overflow,
        active_alignment_subjects,
    })
}

fn round_canonical_optional_intervals<S: LayoutScalar>(
    intervals: OptionalPhysicalContributionIntervalsOf<S>,
    cumulative_origin: Point<S>,
) -> Result<OptionalPhysicalContributionIntervalsOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    Ok(OptionalPhysicalContributionIntervalsOf {
        x: intervals
            .x
            .map(|interval| {
                round_canonical_interval(interval, PhysicalAxis::Horizontal, cumulative_origin)
            })
            .transpose()?,
        y: intervals
            .y
            .map(|interval| {
                round_canonical_interval(interval, PhysicalAxis::Vertical, cumulative_origin)
            })
            .transpose()?,
    })
}

fn round_canonical_interval<S: LayoutScalar>(
    interval: PhysicalContributionIntervalOf<S>,
    axis: PhysicalAxis,
    cumulative_origin: Point<S>,
) -> Result<PhysicalContributionIntervalOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let cumulative = match axis {
        PhysicalAxis::Horizontal => cumulative_origin.x,
        PhysicalAxis::Vertical => cumulative_origin.y,
    };
    let minimum = round_canonical_source_coordinate(interval.minimum, cumulative);
    let maximum = round_canonical_source_coordinate(interval.maximum, cumulative);
    validate_physical_scroll_range(axis, minimum, maximum)
        .map_err(CanonicalScrollGeometryErrorOf::RoundedContribution)?;
    Ok(PhysicalContributionIntervalOf { minimum, maximum })
}

fn round_canonical_final_in_flow_end<S: LayoutScalar>(
    end: Option<FinalInFlowEndOf<S>>,
    cumulative_origin: Point<S>,
) -> Result<Option<FinalInFlowEndOf<S>>, CanonicalScrollGeometryErrorOf<S>> {
    let Some(end) = end else {
        return Ok(None);
    };
    let cumulative = match end.side.axis() {
        PhysicalAxis::Horizontal => cumulative_origin.x,
        PhysicalAxis::Vertical => cumulative_origin.y,
    };
    let coordinate = round_canonical_source_coordinate(end.coordinate, cumulative);
    if !coordinate.is_finite() {
        return Err(CanonicalScrollGeometryErrorOf::RoundedFinalInFlowEnd {
            side: end.side,
            value: coordinate,
        });
    }
    Ok(Some(FinalInFlowEndOf {
        side: end.side,
        coordinate,
    }))
}

fn round_canonical_source_coordinate<S: LayoutScalar>(value: S, cumulative: S) -> S {
    canonical_scroll_zero(round(cumulative + value) - round(cumulative))
}

type DefaultCanonicalScrollGeometryFactory =
    fn(
        CanonicalScrollGeometrySourceOf<DefaultScalar>,
    )
        -> Result<ScrollGeometryOf<DefaultScalar>, CanonicalScrollGeometryErrorOf<DefaultScalar>>;
const _: DefaultCanonicalScrollGeometryFactory =
    canonical_scroll_geometry_from_source::<DefaultScalar>;
type DefaultCanonicalScrollGeometryRounding =
    fn(
        ScrollGeometryOf<DefaultScalar>,
        Point<DefaultScalar>,
    )
        -> Result<ScrollGeometryOf<DefaultScalar>, CanonicalScrollGeometryErrorOf<DefaultScalar>>;
const _: DefaultCanonicalScrollGeometryRounding =
    rebuild_rounded_canonical_scroll_geometry::<DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollbarReservationOf<S: LayoutScalar> {
    #[cfg(test)]
    size: Size<S>,
    inset: Edges<S>,
}

impl<S: LayoutScalar> ScrollbarReservationOf<S> {
    #[must_use]
    pub(crate) fn from_overflow(
        overflow: ComputedOverflow,
        item_is_replaced: bool,
        scrollbar_width_value: S,
        direction: Direction,
    ) -> Self {
        Self::from_used_overflow(
            UsedOverflow::from_computed(overflow, item_is_replaced),
            scrollbar_width_value,
            direction,
        )
    }

    fn from_used_overflow(
        overflow: UsedOverflow,
        scrollbar_width_value: S,
        direction: Direction,
    ) -> Self {
        let size = scrollbar_size_from_used_overflow(overflow, scrollbar_width_value);
        Self {
            #[cfg(test)]
            size,
            inset: scrollbar_inset_from_size(size, direction),
        }
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) const fn size(self) -> Size<S> {
        self.size
    }

    #[must_use]
    pub(crate) const fn inset(self) -> Edges<S> {
        self.inset
    }
}

#[must_use]
pub(crate) fn scrollbar_size_from_overflow<S: LayoutScalar>(
    overflow: ComputedOverflow,
    item_is_replaced: bool,
    scrollbar_width_value: S,
) -> Size<S> {
    scrollbar_size_from_used_overflow(
        UsedOverflow::from_computed(overflow, item_is_replaced),
        scrollbar_width_value,
    )
}

fn scrollbar_size_from_used_overflow<S: LayoutScalar>(
    overflow: UsedOverflow,
    scrollbar_width_value: S,
) -> Size<S> {
    Size::new(
        if matches!(
            overflow.y().gutter_classification(),
            UsedOverflowGutter::Forced
        ) {
            scrollbar_width_value
        } else {
            S::ZERO
        },
        if matches!(
            overflow.x().gutter_classification(),
            UsedOverflowGutter::Forced
        ) {
            scrollbar_width_value
        } else {
            S::ZERO
        },
    )
}

#[must_use]
fn scrollbar_inset_from_size<S: LayoutScalar>(size: Size<S>, direction: Direction) -> Edges<S> {
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
pub(crate) fn content_box_inset_with_scrollbar<S: LayoutScalar>(
    padding: Edges<S>,
    border: Edges<S>,
    reservation: ScrollbarReservationOf<S>,
) -> Edges<S> {
    padding + border + reservation.inset()
}

fn round<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

#[cfg(test)]
mod fri05_c02_carrier_tests {
    use super::*;
    use crate::{Direction, ScrollSnapAlignValue, WritingMode};

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
        .expect("test clip source must be a finite ordered physical range");
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

#[cfg(test)]
mod fri05_c02_box_clip_gutter_tests {
    use super::*;
    use crate::{
        LengthPercentageOf, OverflowClipBox, PhysicalSide, ScrollbarGutter, ScrollbarWidthOf,
        WritingMode,
    };

    fn scalar<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }

    fn rect<S: LayoutScalar>(x: f64, y: f64, width: f64, height: f64) -> ScrollRectOf<S> {
        ScrollRectOf::try_new(
            Point::new(scalar(x), scalar(y)),
            Size::new(scalar(width), scalar(height)),
        )
        .unwrap()
    }

    fn flow_axes() -> [FlowAxes; 10] {
        [
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
            FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
            FlowAxes::new(WritingMode::SidewaysRl, Direction::Ltr),
            FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl),
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
        ]
    }

    fn settled_physical_bits(
        flow_axes: FlowAxes,
        inline: bool,
        block: bool,
    ) -> SettledAutoScrollbarState {
        match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => SettledAutoScrollbarState {
                x: inline,
                y: block,
            },
            PhysicalAxis::Vertical => SettledAutoScrollbarState {
                x: block,
                y: inline,
            },
        }
    }

    macro_rules! source_case {
        (
            $flow_axes:expr,
            $inline_overflow:expr,
            $block_overflow:expr,
            $item_is_replaced:expr,
            $border_box_size:expr,
            $border:expr,
            $padding:expr,
            $scrollbar_gutter:expr,
            $scrollbar_width:expr,
            $settled_auto_scrollbars:expr,
            $overflow_clip_margin:expr,
            $scroll_padding:expr $(,)?
        ) => {{
            let flow_axes = $flow_axes;
            let (x, y) = match flow_axes.inline_axis() {
                PhysicalAxis::Horizontal => ($inline_overflow, $block_overflow),
                PhysicalAxis::Vertical => ($block_overflow, $inline_overflow),
            };
            let computed = ComputedOverflow::try_new(x, y).unwrap();

            ScrollBoxClipGutterSourceOf {
                flow_axes,
                used_overflow: UsedOverflow::from_computed(computed, $item_is_replaced),
                border_box_size: $border_box_size,
                border: $border,
                padding: $padding,
                scrollbar_gutter: $scrollbar_gutter,
                scrollbar_width: ScrollbarWidthOf::try_new($scrollbar_width).unwrap(),
                settled_auto_scrollbars: $settled_auto_scrollbars,
                clip_margin: $overflow_clip_margin,
                optimal_region_insets: $scroll_padding,
            }
        }};
    }

    macro_rules! derive_case {
        ($($source:expr),+ $(,)?) => {{
            derive_scroll_box_clip_gutter(source_case!($($source),+)).unwrap()
        }};
    }

    fn gutter_at<S: LayoutScalar>(
        result: ScrollBoxClipGutterResultOf<S>,
        side: PhysicalSide,
    ) -> Option<ScrollRectOf<S>> {
        match side {
            PhysicalSide::Top => result.gutters.top,
            PhysicalSide::Right => result.gutters.right,
            PhysicalSide::Bottom => result.gutters.bottom,
            PhysicalSide::Left => result.gutters.left,
        }
    }

    fn assert_gutter_sides<S: LayoutScalar>(
        result: ScrollBoxClipGutterResultOf<S>,
        expected: &[PhysicalSide],
        thickness: S,
    ) {
        let mut expected_width = S::ZERO;
        let mut expected_height = S::ZERO;
        for side in [
            PhysicalSide::Top,
            PhysicalSide::Right,
            PhysicalSide::Bottom,
            PhysicalSide::Left,
        ] {
            let gutter = gutter_at(result, side);
            assert_eq!(gutter.is_some(), expected.contains(&side));
            let Some(gutter) = gutter else {
                continue;
            };

            match side {
                PhysicalSide::Top => {
                    assert_eq!(gutter.origin().y, result.padding_box.origin().y);
                    assert_eq!(gutter.size().height, thickness);
                    assert_eq!(gutter.origin().x, result.scrollport.origin().x);
                    assert_eq!(gutter.size().width, result.scrollport.size().width);
                    expected_height = expected_height + thickness;
                }
                PhysicalSide::Right => {
                    assert_eq!(
                        gutter.origin().x + gutter.size().width,
                        result.padding_box.origin().x + result.padding_box.size().width
                    );
                    assert_eq!(gutter.size().width, thickness);
                    assert_eq!(gutter.origin().y, result.scrollport.origin().y);
                    assert_eq!(gutter.size().height, result.scrollport.size().height);
                    expected_width = expected_width + thickness;
                }
                PhysicalSide::Bottom => {
                    assert_eq!(
                        gutter.origin().y + gutter.size().height,
                        result.padding_box.origin().y + result.padding_box.size().height
                    );
                    assert_eq!(gutter.size().height, thickness);
                    assert_eq!(gutter.origin().x, result.scrollport.origin().x);
                    assert_eq!(gutter.size().width, result.scrollport.size().width);
                    expected_height = expected_height + thickness;
                }
                PhysicalSide::Left => {
                    assert_eq!(gutter.origin().x, result.padding_box.origin().x);
                    assert_eq!(gutter.size().width, thickness);
                    assert_eq!(gutter.origin().y, result.scrollport.origin().y);
                    assert_eq!(gutter.size().height, result.scrollport.size().height);
                    expected_width = expected_width + thickness;
                }
            }
        }
        assert_eq!(
            result.aggregate_reservation,
            Size::new(expected_width, expected_height)
        );
    }

    fn assert_clip<S: LayoutScalar>(actual: Option<PhysicalClipAxisOf<S>>, minimum: S, maximum: S) {
        let actual = actual.expect("expected a finite clip interval");
        assert_eq!(actual.minimum(), minimum);
        assert_eq!(actual.maximum(), maximum);
    }

    fn assert_scalar_flow_matrix<S: LayoutScalar>() {
        let width = scalar::<S>(7.0);
        for flow_axes in flow_axes() {
            let none = SettledAutoScrollbarState { x: false, y: false };
            let common = (
                Size::new(scalar(100.0), scalar(80.0)),
                Edges::ZERO,
                Edges::ZERO,
                ClipMarginSourceOf::default(),
                OptimalRegionInsetsOf::default(),
            );

            let hidden_auto = derive_case!(
                flow_axes,
                Overflow::Hidden,
                Overflow::Hidden,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::Auto,
                width,
                none,
                common.3,
                common.4,
            );
            assert_gutter_sides(hidden_auto, &[], width);

            let hidden_stable = derive_case!(
                flow_axes,
                Overflow::Hidden,
                Overflow::Hidden,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::Stable,
                width,
                none,
                common.3,
                common.4,
            );
            assert_gutter_sides(hidden_stable, &[flow_axes.inline_end()], width);

            let hidden_both = derive_case!(
                flow_axes,
                Overflow::Hidden,
                Overflow::Hidden,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::StableBothEdges,
                width,
                none,
                common.3,
                common.4,
            );
            assert_gutter_sides(
                hidden_both,
                &[flow_axes.inline_start(), flow_axes.inline_end()],
                width,
            );

            let forced = derive_case!(
                flow_axes,
                Overflow::Hidden,
                Overflow::Scroll,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::Auto,
                width,
                none,
                common.3,
                common.4,
            );
            assert_gutter_sides(forced, &[flow_axes.inline_end()], width);

            let settled_block = settled_physical_bits(flow_axes, false, true);
            let conditional_block = derive_case!(
                flow_axes,
                Overflow::Auto,
                Overflow::Auto,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::Auto,
                width,
                settled_block,
                common.3,
                common.4,
            );
            assert_gutter_sides(conditional_block, &[flow_axes.inline_end()], width);

            let settled_inline = settled_physical_bits(flow_axes, true, false);
            let conditional_inline = derive_case!(
                flow_axes,
                Overflow::Auto,
                Overflow::Auto,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::Auto,
                width,
                settled_inline,
                common.3,
                common.4,
            );
            assert_gutter_sides(conditional_inline, &[flow_axes.block_end()], width);

            let auto_stable = derive_case!(
                flow_axes,
                Overflow::Auto,
                Overflow::Auto,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::Stable,
                width,
                none,
                common.3,
                common.4,
            );
            assert_gutter_sides(auto_stable, &[flow_axes.inline_end()], width);

            let auto_stable_with_inline = derive_case!(
                flow_axes,
                Overflow::Auto,
                Overflow::Auto,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::Stable,
                width,
                settled_inline,
                common.3,
                common.4,
            );
            assert_gutter_sides(
                auto_stable_with_inline,
                &[flow_axes.inline_end(), flow_axes.block_end()],
                width,
            );

            let auto_both = derive_case!(
                flow_axes,
                Overflow::Auto,
                Overflow::Auto,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::StableBothEdges,
                width,
                none,
                common.3,
                common.4,
            );
            assert_gutter_sides(
                auto_both,
                &[flow_axes.inline_start(), flow_axes.inline_end()],
                width,
            );

            let forced_both = derive_case!(
                flow_axes,
                Overflow::Scroll,
                Overflow::Scroll,
                false,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::StableBothEdges,
                width,
                none,
                common.3,
                common.4,
            );
            assert_gutter_sides(
                forced_both,
                &[
                    flow_axes.inline_start(),
                    flow_axes.inline_end(),
                    flow_axes.block_end(),
                ],
                width,
            );

            for gutter in [
                ScrollbarGutter::Auto,
                ScrollbarGutter::Stable,
                ScrollbarGutter::StableBothEdges,
            ] {
                let visible_clip = derive_case!(
                    flow_axes,
                    Overflow::Visible,
                    Overflow::Clip,
                    false,
                    common.0,
                    common.1,
                    common.2,
                    gutter,
                    width,
                    settled_physical_bits(flow_axes, true, true),
                    common.3,
                    common.4,
                );
                assert_gutter_sides(visible_clip, &[], width);
            }

            let replaced_hidden = derive_case!(
                flow_axes,
                Overflow::Hidden,
                Overflow::Hidden,
                true,
                common.0,
                common.1,
                common.2,
                ScrollbarGutter::StableBothEdges,
                width,
                settled_physical_bits(flow_axes, true, true),
                common.3,
                common.4,
            );
            assert_gutter_sides(replaced_hidden, &[], width);
        }
    }

    #[test]
    fn fri05_c02_box_clip_gutter_places_forced_stable_both_and_settled_auto_in_all_flows() {
        assert_scalar_flow_matrix::<f32>();
        assert_scalar_flow_matrix::<f64>();
    }

    fn assert_scalar_gutter_saturation<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let none = SettledAutoScrollbarState { x: false, y: false };
        let defaults = (
            Edges::ZERO,
            Edges::ZERO,
            ClipMarginSourceOf::default(),
            OptimalRegionInsetsOf::default(),
        );

        let one_sided = derive_case!(
            axes,
            Overflow::Hidden,
            Overflow::Scroll,
            false,
            Size::new(scalar(2.0), scalar(40.0)),
            defaults.0,
            defaults.1,
            ScrollbarGutter::Auto,
            scalar(15.0),
            none,
            defaults.2,
            defaults.3,
        );
        assert_eq!(one_sided.scrollport, rect(0.0, 0.0, 0.0, 40.0));
        assert_eq!(one_sided.content_box, one_sided.scrollport);
        assert_eq!(
            one_sided.aggregate_reservation,
            Size::new(scalar(2.0), S::ZERO)
        );
        assert_eq!(
            gutter_at(one_sided, PhysicalSide::Right),
            Some(rect(0.0, 0.0, 2.0, 40.0))
        );

        let symmetric = derive_case!(
            axes,
            Overflow::Hidden,
            Overflow::Hidden,
            false,
            Size::new(scalar(20.0), scalar(40.0)),
            defaults.0,
            defaults.1,
            ScrollbarGutter::StableBothEdges,
            scalar(15.0),
            none,
            defaults.2,
            defaults.3,
        );
        assert_eq!(
            symmetric.aggregate_reservation,
            Size::new(scalar(20.0), S::ZERO)
        );
        assert_eq!(symmetric.scrollport, rect(10.0, 0.0, 0.0, 40.0));
        assert_eq!(
            gutter_at(symmetric, PhysicalSide::Left),
            Some(rect(0.0, 0.0, 10.0, 40.0))
        );
        assert_eq!(
            gutter_at(symmetric, PhysicalSide::Right),
            Some(rect(10.0, 0.0, 10.0, 40.0))
        );

        let unsaturated = derive_case!(
            axes,
            Overflow::Hidden,
            Overflow::Hidden,
            false,
            Size::new(scalar(40.0), scalar(40.0)),
            defaults.0,
            defaults.1,
            ScrollbarGutter::StableBothEdges,
            scalar(15.0),
            none,
            defaults.2,
            defaults.3,
        );
        assert_eq!(
            unsaturated.aggregate_reservation,
            Size::new(scalar(30.0), S::ZERO)
        );
        assert_eq!(unsaturated.scrollport, rect(15.0, 0.0, 10.0, 40.0));

        let independent_axes = derive_case!(
            axes,
            Overflow::Scroll,
            Overflow::Scroll,
            false,
            Size::new(scalar(2.0), scalar(100.0)),
            defaults.0,
            defaults.1,
            ScrollbarGutter::Auto,
            scalar(15.0),
            none,
            defaults.2,
            defaults.3,
        );
        assert_eq!(
            independent_axes.aggregate_reservation,
            Size::new(scalar(2.0), scalar(15.0))
        );
        assert_eq!(independent_axes.scrollport, rect(0.0, 0.0, 0.0, 85.0));
        assert_eq!(
            gutter_at(independent_axes, PhysicalSide::Bottom),
            Some(rect(0.0, 85.0, 0.0, 15.0))
        );
    }

    #[test]
    fn fri05_c02_box_clip_gutter_proportionally_saturates_each_axis_and_preserves_requests() {
        assert_scalar_gutter_saturation::<f32>();
        assert_scalar_gutter_saturation::<f64>();
    }

    fn assert_scalar_zero_geometry<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let none = SettledAutoScrollbarState { x: false, y: false };
        let zero_thickness = derive_case!(
            axes,
            Overflow::Scroll,
            Overflow::Scroll,
            false,
            Size::new(scalar(30.0), scalar(20.0)),
            Edges::ZERO,
            Edges::ZERO,
            ScrollbarGutter::StableBothEdges,
            S::ZERO,
            none,
            ClipMarginSourceOf::default(),
            OptimalRegionInsetsOf::default(),
        );
        assert_gutter_sides(zero_thickness, &[], S::ZERO);
        assert_eq!(zero_thickness.scrollport, rect(0.0, 0.0, 30.0, 20.0));

        let zero_box = derive_case!(
            axes,
            Overflow::Scroll,
            Overflow::Scroll,
            false,
            Size::ZERO,
            Edges::all(scalar(5.0)),
            Edges::all(scalar(7.0)),
            ScrollbarGutter::StableBothEdges,
            scalar(15.0),
            none,
            ClipMarginSourceOf::default(),
            OptimalRegionInsetsOf::default(),
        );
        assert_gutter_sides(zero_box, &[], S::ZERO);
        assert_eq!(zero_box.border_box, rect(0.0, 0.0, 0.0, 0.0));
        assert_eq!(zero_box.padding_box, zero_box.border_box);
        assert_eq!(zero_box.content_box, zero_box.border_box);
        assert_eq!(zero_box.scrollport, zero_box.border_box);
        assert_eq!(zero_box.optimal_viewing_region, zero_box.border_box);
    }

    #[test]
    fn fri05_c02_box_clip_gutter_zero_thickness_and_zero_boxes_stay_empty_and_ordered() {
        assert_scalar_zero_geometry::<f32>();
        assert_scalar_zero_geometry::<f64>();
    }

    fn assert_scalar_box_insets<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let none = SettledAutoScrollbarState { x: false, y: false };
        let saturated_border = derive_case!(
            axes,
            Overflow::Visible,
            Overflow::Visible,
            false,
            Size::new(scalar(10.0), scalar(20.0)),
            Edges::new(scalar(2.0), scalar(8.0), scalar(2.0), scalar(8.0)),
            Edges::ZERO,
            ScrollbarGutter::Auto,
            S::ZERO,
            none,
            ClipMarginSourceOf::default(),
            OptimalRegionInsetsOf::default(),
        );
        assert_eq!(saturated_border.padding_box, rect(5.0, 2.0, 0.0, 16.0));

        let saturated_padding = derive_case!(
            axes,
            Overflow::Visible,
            Overflow::Visible,
            false,
            Size::new(scalar(10.0), scalar(20.0)),
            Edges::ZERO,
            Edges::new(scalar(15.0), scalar(8.0), scalar(15.0), scalar(8.0)),
            ScrollbarGutter::Auto,
            S::ZERO,
            none,
            ClipMarginSourceOf::default(),
            OptimalRegionInsetsOf::default(),
        );
        assert_eq!(saturated_padding.content_box, rect(5.0, 10.0, 0.0, 0.0));

        let nested: ScrollBoxClipGutterResultOf<S> = derive_case!(
            axes,
            Overflow::Scroll,
            Overflow::Scroll,
            false,
            Size::new(scalar(100.0), scalar(80.0)),
            Edges::all(scalar(1.0)),
            Edges::new(scalar(2.0), scalar(3.0), scalar(4.0), scalar(5.0)),
            ScrollbarGutter::Auto,
            scalar(10.0),
            none,
            ClipMarginSourceOf::default(),
            OptimalRegionInsetsOf::default(),
        );
        assert_eq!(nested.border_box, rect(0.0, 0.0, 100.0, 80.0));
        assert_eq!(nested.padding_box, rect(1.0, 1.0, 98.0, 78.0));
        assert_eq!(nested.scrollport, rect(1.0, 1.0, 88.0, 68.0));
        assert_eq!(nested.content_box, rect(6.0, 3.0, 80.0, 62.0));
    }

    #[test]
    fn fri05_c02_box_clip_gutter_border_padding_and_gutters_form_one_saturated_nesting() {
        assert_scalar_box_insets::<f32>();
        assert_scalar_box_insets::<f64>();
    }

    fn assert_scalar_clip_reference_boxes<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let none = SettledAutoScrollbarState { x: false, y: false };
        let size = Size::new(scalar(100.0), scalar(80.0));
        let border = Edges::all(scalar(2.0));
        let padding = Edges::all(scalar(5.0));
        let cases: [(OverflowClipBox, S, S); 3] = [
            (OverflowClipBox::ContentBox, scalar(4.0), scalar(96.0)),
            (
                OverflowClipBox::PaddingBox,
                -scalar::<S>(1.0),
                scalar(101.0),
            ),
            (OverflowClipBox::BorderBox, -scalar::<S>(3.0), scalar(103.0)),
        ];

        for (clip_box, minimum, maximum) in cases {
            let result = derive_case!(
                axes,
                Overflow::Clip,
                Overflow::Visible,
                false,
                size,
                border,
                padding,
                ScrollbarGutter::StableBothEdges,
                scalar(9.0),
                none,
                ClipMarginSourceOf::new(clip_box, scalar(3.0)),
                OptimalRegionInsetsOf::default(),
            );
            assert_clip(result.overflow_clip.x(), minimum, maximum);
            assert_eq!(result.overflow_clip.y(), None);
        }

        let y_only = derive_case!(
            axes,
            Overflow::Visible,
            Overflow::Clip,
            false,
            size,
            border,
            padding,
            ScrollbarGutter::StableBothEdges,
            scalar(9.0),
            none,
            ClipMarginSourceOf::new(OverflowClipBox::BorderBox, scalar(3.0)),
            OptimalRegionInsetsOf::default(),
        );
        assert_eq!(y_only.overflow_clip.x(), None);
        assert_clip(y_only.overflow_clip.y(), -scalar::<S>(3.0), scalar(83.0));
    }

    #[test]
    fn fri05_c02_box_clip_gutter_clip_expands_only_its_axis_from_each_reference_box() {
        assert_scalar_clip_reference_boxes::<f32>();
        assert_scalar_clip_reference_boxes::<f64>();
    }

    fn assert_scalar_scrollport_clips<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let none = SettledAutoScrollbarState { x: false, y: false };
        let clip_margin = ClipMarginSourceOf::new(OverflowClipBox::BorderBox, scalar(20.0));
        for overflow in [Overflow::Hidden, Overflow::Scroll, Overflow::Auto] {
            let result = derive_case!(
                axes,
                overflow,
                overflow,
                false,
                Size::new(scalar(100.0), scalar(80.0)),
                Edges::all(scalar(2.0)),
                Edges::all(scalar(5.0)),
                ScrollbarGutter::Auto,
                S::ZERO,
                none,
                clip_margin,
                OptimalRegionInsetsOf::default(),
            );
            assert_clip(result.overflow_clip.x(), scalar(2.0), scalar(98.0));
            assert_clip(result.overflow_clip.y(), scalar(2.0), scalar(78.0));
        }

        let visible = derive_case!(
            axes,
            Overflow::Visible,
            Overflow::Visible,
            false,
            Size::new(scalar(100.0), scalar(80.0)),
            Edges::all(scalar(2.0)),
            Edges::all(scalar(5.0)),
            ScrollbarGutter::StableBothEdges,
            scalar(10.0),
            settled_physical_bits(axes, true, true),
            clip_margin,
            OptimalRegionInsetsOf::default(),
        );
        assert_eq!(visible.overflow_clip.x(), None);
        assert_eq!(visible.overflow_clip.y(), None);

        let replaced_hidden = derive_case!(
            axes,
            Overflow::Hidden,
            Overflow::Hidden,
            true,
            Size::new(scalar(100.0), scalar(80.0)),
            Edges::all(scalar(2.0)),
            Edges::all(scalar(5.0)),
            ScrollbarGutter::StableBothEdges,
            scalar(10.0),
            settled_physical_bits(axes, true, true),
            clip_margin,
            OptimalRegionInsetsOf::default(),
        );
        assert_clip(
            replaced_hidden.overflow_clip.x(),
            -scalar::<S>(20.0),
            scalar(120.0),
        );
        assert_clip(
            replaced_hidden.overflow_clip.y(),
            -scalar::<S>(20.0),
            scalar(100.0),
        );
        assert_eq!(replaced_hidden.aggregate_reservation, Size::ZERO);
    }

    #[test]
    fn fri05_c02_box_clip_gutter_hidden_scroll_auto_and_replaced_hidden_use_their_own_clips() {
        assert_scalar_scrollport_clips::<f32>();
        assert_scalar_scrollport_clips::<f64>();
    }

    fn assert_scalar_scroll_padding<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let none = SettledAutoScrollbarState { x: false, y: false };
        let scroll_padding = OptimalRegionInsetsOf::new(
            OptimalRegionInsetOf::Value(
                LengthPercentageOf::from_percent_fraction(scalar(0.25)).unwrap(),
            ),
            OptimalRegionInsetOf::Value(
                LengthPercentageOf::from_percent_fraction(scalar(0.25)).unwrap(),
            ),
            OptimalRegionInsetOf::Value(
                LengthPercentageOf::from_coefficients(-scalar::<S>(20.0), scalar(0.25)).unwrap(),
            ),
            OptimalRegionInsetOf::Value(LengthPercentageOf::px(scalar(10.0)).unwrap()),
        );
        let result = derive_case!(
            axes,
            Overflow::Scroll,
            Overflow::Scroll,
            false,
            Size::new(scalar(100.0), scalar(60.0)),
            Edges::all(scalar(5.0)),
            Edges::ZERO,
            ScrollbarGutter::Auto,
            scalar(10.0),
            none,
            ClipMarginSourceOf::default(),
            scroll_padding,
        );
        assert_eq!(result.scrollport, rect(5.0, 5.0, 80.0, 40.0));
        assert_eq!(
            result.resolved_scroll_padding,
            Edges::new(scalar(10.0), scalar(20.0), S::ZERO, scalar(10.0))
        );
        assert_eq!(result.optimal_viewing_region, rect(15.0, 15.0, 50.0, 30.0));

        let automatic: ScrollBoxClipGutterResultOf<S> = derive_case!(
            axes,
            Overflow::Scroll,
            Overflow::Scroll,
            false,
            Size::new(scalar(100.0), scalar(60.0)),
            Edges::all(scalar(5.0)),
            Edges::ZERO,
            ScrollbarGutter::Auto,
            scalar(10.0),
            none,
            ClipMarginSourceOf::default(),
            OptimalRegionInsetsOf::default(),
        );
        assert_eq!(automatic.resolved_scroll_padding, Edges::ZERO);
        assert_eq!(automatic.optimal_viewing_region, automatic.scrollport);

        let oversized = OptimalRegionInsetsOf::<S>::new(
            OptimalRegionInsetOf::Value(LengthPercentageOf::px(scalar(5.0)).unwrap()),
            OptimalRegionInsetOf::Value(LengthPercentageOf::px(scalar(100.0)).unwrap()),
            OptimalRegionInsetOf::Value(LengthPercentageOf::px(scalar(5.0)).unwrap()),
            OptimalRegionInsetOf::Value(LengthPercentageOf::px(scalar(100.0)).unwrap()),
        );
        let oversized = derive_case!(
            axes,
            Overflow::Scroll,
            Overflow::Scroll,
            false,
            Size::new(scalar(100.0), scalar(60.0)),
            Edges::all(scalar(5.0)),
            Edges::ZERO,
            ScrollbarGutter::Auto,
            scalar(10.0),
            none,
            ClipMarginSourceOf::default(),
            oversized,
        );
        assert_eq!(
            oversized.resolved_scroll_padding,
            Edges::new(scalar(5.0), scalar(100.0), scalar(5.0), scalar(100.0))
        );
        assert_eq!(
            oversized.optimal_viewing_region,
            rect(45.0, 10.0, 0.0, 30.0)
        );
    }

    #[test]
    fn fri05_c02_box_clip_gutter_resolves_padding_after_scrollport_and_saturates_optimal_region() {
        assert_scalar_scroll_padding::<f32>();
        assert_scalar_scroll_padding::<f64>();
    }

    fn assert_scalar_non_finite_derivation_fails<S: LayoutScalar>(large: S) {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let none = SettledAutoScrollbarState { x: false, y: false };
        let clip_overflow = source_case!(
            axes,
            Overflow::Clip,
            Overflow::Visible,
            false,
            Size::new(large / scalar(2.0), scalar(10.0)),
            Edges::ZERO,
            Edges::ZERO,
            ScrollbarGutter::Auto,
            S::ZERO,
            none,
            ClipMarginSourceOf::new(OverflowClipBox::BorderBox, large),
            OptimalRegionInsetsOf::default(),
        );
        assert!(derive_scroll_box_clip_gutter(clip_overflow).is_err());

        let overflowing_padding = OptimalRegionInsetsOf::new(
            OptimalRegionInsetOf::Auto,
            OptimalRegionInsetOf::Value(LengthPercentageOf::from_percent_fraction(large).unwrap()),
            OptimalRegionInsetOf::Auto,
            OptimalRegionInsetOf::Auto,
        );
        let padding_overflow = source_case!(
            axes,
            Overflow::Visible,
            Overflow::Visible,
            false,
            Size::new(scalar(10.0), scalar(10.0)),
            Edges::ZERO,
            Edges::ZERO,
            ScrollbarGutter::Auto,
            S::ZERO,
            none,
            ClipMarginSourceOf::default(),
            overflowing_padding,
        );
        assert!(derive_scroll_box_clip_gutter(padding_overflow).is_err());
    }

    #[test]
    fn fri05_c02_box_clip_gutter_non_finite_derived_edges_fail_atomically() {
        assert_scalar_non_finite_derivation_fails::<f32>(f32::MAX);
        assert_scalar_non_finite_derivation_fails::<f64>(f64::MAX);
    }
}

#[cfg(test)]
mod fri05_c02_contribution_range_tests {
    use super::*;
    use crate::{Direction, WritingMode};

    fn scalar<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }

    fn rect<S: LayoutScalar>(x: f64, y: f64, width: f64, height: f64) -> ScrollRectOf<S> {
        ScrollRectOf::try_new(
            Point::new(scalar(x), scalar(y)),
            Size::new(scalar(width), scalar(height)),
        )
        .expect("test rectangle must be finite and non-negative")
    }

    fn used_overflow(x: Overflow, y: Overflow) -> UsedOverflow {
        UsedOverflow {
            x: UsedOverflowAxis { value: x },
            y: UsedOverflowAxis { value: y },
        }
    }

    fn all_flow_axes() -> [FlowAxes; 10] {
        [
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
            FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
            FlowAxes::new(WritingMode::SidewaysRl, Direction::Ltr),
            FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl),
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
        ]
    }

    fn assert_interval<S: LayoutScalar>(
        actual: Option<PhysicalContributionIntervalOf<S>>,
        minimum: f64,
        maximum: f64,
    ) {
        let actual = actual.expect("expected a retained physical interval");
        assert_eq!(actual.minimum(), scalar(minimum));
        assert_eq!(actual.maximum(), scalar(maximum));
    }

    fn assert_bounds<S: LayoutScalar>(
        actual: PhysicalContributionBoundsOf<S>,
        x_minimum: f64,
        x_maximum: f64,
        y_minimum: f64,
        y_maximum: f64,
    ) {
        assert_interval(Some(actual.x()), x_minimum, x_maximum);
        assert_interval(Some(actual.y()), y_minimum, y_maximum);
    }

    fn empty_descendants<S: LayoutScalar>() -> OptionalPhysicalContributionIntervalsOf<S> {
        ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 0.0, 0.0))
            .propagatable_descendant_intervals()
    }

    fn assert_scalar_padding_and_direct_line<S: LayoutScalar>() {
        let mut accumulator = ScrollContributionAccumulatorOf::<S>::new(rect(-5.0, -7.0, 0.0, 0.0));

        assert_bounds(accumulator.complete_overflow(), -5.0, -5.0, -7.0, -7.0);
        assert_eq!(
            accumulator.propagatable_descendant_intervals(),
            OptionalPhysicalContributionIntervalsOf::NONE
        );

        accumulator.include_direct_line(rect(-20.0, 3.0, 30.0, 4.0));

        assert_bounds(accumulator.complete_overflow(), -20.0, 10.0, -7.0, 7.0);
        let descendants = accumulator.propagatable_descendant_intervals();
        assert_interval(descendants.x(), -20.0, 10.0);
        assert_interval(descendants.y(), 3.0, 7.0);
    }

    #[test]
    fn fri05_c02_accumulator_padding_seed_and_direct_line_retain_zero_and_negative_origins() {
        assert_scalar_padding_and_direct_line::<f32>();
        assert_scalar_padding_and_direct_line::<f64>();
    }

    fn assert_scalar_flex_range_basis<S: LayoutScalar>() {
        let scrollport = rect(0.0, 0.0, 93.0, 80.0);
        let mut accumulator =
            ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 100.0, 80.0));
        accumulator.exclude_reserved_gutter_from_range();

        assert_bounds(accumulator.complete_overflow(), 0.0, 100.0, 0.0, 80.0);
        assert_bounds(accumulator.range_overflow(scrollport), 0.0, 93.0, 0.0, 80.0);

        accumulator.include_direct_line(rect(-5.0, 0.0, 115.0, 80.0));
        assert_bounds(accumulator.complete_overflow(), -5.0, 110.0, 0.0, 80.0);
        assert_bounds(
            accumulator.range_overflow(scrollport),
            -5.0,
            110.0,
            0.0,
            80.0,
        );

        let rounded = round_canonical_contributions(accumulator, Point::ZERO).unwrap();
        assert_eq!(
            rounded.container_range_basis,
            ContainerRangeBasis::Scrollport
        );
        assert_bounds(rounded.complete_overflow(), -5.0, 110.0, 0.0, 80.0);
        assert_bounds(rounded.range_overflow(scrollport), -5.0, 110.0, 0.0, 80.0);
    }

    #[test]
    fn fri05_c04_flex_geometry_range_basis_retains_padding_seed_and_descendant_overflow() {
        assert_scalar_flex_range_basis::<f32>();
        assert_scalar_flex_range_basis::<f64>();
    }

    fn assert_scalar_border_margin_and_absolute<S: LayoutScalar>() {
        let mut accumulator = ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 10.0, 10.0));
        let none = empty_descendants::<S>();

        accumulator
            .include_in_flow_child(
                Point::new(scalar(20.0), scalar(30.0)),
                rect(1.0, 2.0, 5.0, 4.0),
                Edges::new(scalar(-100.0), scalar(3.0), scalar(-4.0), scalar(2.0)),
                none,
                used_overflow(Overflow::Visible, Overflow::Visible),
            )
            .unwrap();
        accumulator
            .include_in_flow_child(
                Point::new(scalar(40.0), scalar(40.0)),
                rect(0.0, 0.0, 2.0, 3.0),
                Edges::all(scalar(-50.0)),
                none,
                used_overflow(Overflow::Visible, Overflow::Visible),
            )
            .unwrap();

        for zero_border in [rect(0.0, 0.0, 0.0, 10.0), rect(0.0, 0.0, 10.0, 0.0)] {
            accumulator
                .include_in_flow_child(
                    Point::new(scalar(-100.0), scalar(-100.0)),
                    zero_border,
                    Edges::all(scalar(100.0)),
                    none,
                    used_overflow(Overflow::Visible, Overflow::Visible),
                )
                .unwrap();
        }

        accumulator
            .include_current_out_of_flow(
                Point::new(scalar(-10.0), scalar(-20.0)),
                rect(0.0, 0.0, 4.0, 5.0),
                Edges::new(scalar(1.0), scalar(1.0), scalar(2.0), scalar(1.0)),
                none,
                used_overflow(Overflow::Visible, Overflow::Visible),
            )
            .unwrap();

        assert_bounds(accumulator.complete_overflow(), -11.0, 42.0, -21.0, 43.0);
        let descendants = accumulator.propagatable_descendant_intervals();
        assert_interval(descendants.x(), -11.0, 42.0);
        assert_interval(descendants.y(), -21.0, 43.0);
    }

    #[test]
    fn fri05_c02_accumulator_positive_area_margin_outsets_and_current_absolute_are_exact() {
        assert_scalar_border_margin_and_absolute::<f32>();
        assert_scalar_border_margin_and_absolute::<f64>();
    }

    fn descendant_source<S: LayoutScalar>() -> OptionalPhysicalContributionIntervalsOf<S> {
        let mut source = ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 0.0, 0.0));
        source.include_direct_line(rect(-4.0, 5.0, 12.0, 20.0));
        source.propagatable_descendant_intervals()
    }

    fn assert_scalar_transitive_and_trapped<S: LayoutScalar>() {
        let source = descendant_source::<S>();
        for trapped in [
            Overflow::Clip,
            Overflow::Hidden,
            Overflow::Scroll,
            Overflow::Auto,
        ] {
            let mut x_visible =
                ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 10.0, 10.0));
            x_visible
                .include_in_flow_child(
                    Point::new(scalar(20.0), scalar(30.0)),
                    rect(0.0, 0.0, 0.0, 8.0),
                    Edges::all(scalar(100.0)),
                    source,
                    used_overflow(Overflow::Visible, trapped),
                )
                .unwrap();
            assert_bounds(x_visible.complete_overflow(), 0.0, 28.0, 0.0, 10.0);
            let descendants = x_visible.propagatable_descendant_intervals();
            assert_interval(descendants.x(), 16.0, 28.0);
            assert_eq!(descendants.y(), None);

            let mut y_visible =
                ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 10.0, 10.0));
            y_visible
                .include_in_flow_child(
                    Point::new(scalar(20.0), scalar(30.0)),
                    rect(0.0, 0.0, 8.0, 0.0),
                    Edges::all(scalar(100.0)),
                    source,
                    used_overflow(trapped, Overflow::Visible),
                )
                .unwrap();
            assert_bounds(y_visible.complete_overflow(), 0.0, 10.0, 0.0, 55.0);
            let descendants = y_visible.propagatable_descendant_intervals();
            assert_eq!(descendants.x(), None);
            assert_interval(descendants.y(), 35.0, 55.0);
        }

        let mut middle = ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 0.0, 0.0));
        middle
            .include_in_flow_child(
                Point::new(scalar(10.0), scalar(20.0)),
                rect(0.0, 0.0, 0.0, 0.0),
                Edges::all(scalar(50.0)),
                source,
                used_overflow(Overflow::Visible, Overflow::Visible),
            )
            .unwrap();
        let middle_descendants = middle.propagatable_descendant_intervals();
        assert_interval(middle_descendants.x(), 6.0, 18.0);
        assert_interval(middle_descendants.y(), 25.0, 45.0);

        let mut parent = ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 1.0, 1.0));
        parent
            .include_in_flow_child(
                Point::new(scalar(100.0), scalar(-50.0)),
                rect(0.0, 0.0, 0.0, 0.0),
                Edges::all(scalar(80.0)),
                middle_descendants,
                used_overflow(Overflow::Visible, Overflow::Visible),
            )
            .unwrap();
        assert_bounds(parent.complete_overflow(), 0.0, 118.0, -25.0, 1.0);
        let descendants = parent.propagatable_descendant_intervals();
        assert_interval(descendants.x(), 106.0, 118.0);
        assert_interval(descendants.y(), -25.0, -5.0);
    }

    #[test]
    fn fri05_c02_accumulator_translates_visible_transitive_intervals_and_traps_each_axis() {
        assert_scalar_transitive_and_trapped::<f32>();
        assert_scalar_transitive_and_trapped::<f64>();
    }

    fn assert_scalar_terminal_padding_and_subject<S: LayoutScalar>() {
        let mut ordinary = ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 10.0, 10.0));
        ordinary.include_direct_line(rect(-40.0, 0.0, 140.0, 20.0));
        let horizontal = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        ordinary
            .record_final_in_flow_end(horizontal, LogicalAxis::Inline, scalar(40.0))
            .unwrap();
        ordinary
            .record_final_in_flow_end(horizontal, LogicalAxis::Block, scalar(30.0))
            .unwrap();
        ordinary
            .include_terminal_padding(Edges::new(S::ZERO, scalar(8.0), scalar(7.0), S::ZERO))
            .unwrap();
        assert_bounds(ordinary.complete_overflow(), -40.0, 100.0, 0.0, 37.0);

        let before_subject = ordinary.complete_overflow();
        ordinary
            .set_active_alignment_subject(PhysicalAxis::Horizontal, rect(-30.0, 5.0, 40.0, 2.0));
        ordinary.set_active_alignment_subject(PhysicalAxis::Vertical, rect(-30.0, 5.0, 40.0, 2.0));
        assert_eq!(ordinary.complete_overflow(), before_subject);
        let subjects = ordinary.active_alignment_subject_intervals();
        assert_interval(subjects.x(), -30.0, 10.0);
        assert_interval(subjects.y(), 5.0, 7.0);

        let mut reversed = ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 10.0, 10.0));
        let vertical_rl = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
        reversed
            .record_final_in_flow_end(vertical_rl, LogicalAxis::Inline, scalar(20.0))
            .unwrap();
        reversed
            .record_final_in_flow_end(vertical_rl, LogicalAxis::Block, scalar(-10.0))
            .unwrap();
        reversed
            .include_terminal_padding(Edges::new(S::ZERO, S::ZERO, scalar(5.0), scalar(4.0)))
            .unwrap();
        assert_bounds(reversed.complete_overflow(), -14.0, 10.0, 0.0, 25.0);
    }

    #[test]
    fn fri05_c02_accumulator_terminal_padding_and_active_subject_remain_separate() {
        assert_scalar_terminal_padding_and_subject::<f32>();
        assert_scalar_terminal_padding_and_subject::<f64>();
    }

    #[derive(Clone, Copy)]
    struct AxisRangeSource<S: LayoutScalar> {
        overflow_minimum: S,
        overflow_maximum: S,
        subject_minimum: S,
        subject_maximum: S,
    }

    fn axis_range_source<S: LayoutScalar>(
        scrollport_minimum: S,
        scrollport_maximum: S,
        end_extent: S,
        start_extent: S,
        origin_decreasing: bool,
    ) -> AxisRangeSource<S> {
        if origin_decreasing {
            AxisRangeSource {
                overflow_minimum: scrollport_minimum - end_extent,
                overflow_maximum: scrollport_maximum + scalar(70.0),
                subject_minimum: scrollport_minimum + scalar(1.0),
                subject_maximum: scrollport_maximum + start_extent,
            }
        } else {
            AxisRangeSource {
                overflow_minimum: scrollport_minimum - scalar(70.0),
                overflow_maximum: scrollport_maximum + end_extent,
                subject_minimum: scrollport_minimum - start_extent,
                subject_maximum: scrollport_maximum - scalar(1.0),
            }
        }
    }

    fn origin_is_decreasing(
        flow_axes: FlowAxes,
        logical_axis: LogicalAxis,
        progression: ScrollOriginProgression,
    ) -> bool {
        flow_axes
            .logical_axis_progression(logical_axis)
            .is_decreasing()
            ^ matches!(progression, ScrollOriginProgression::FlowStartward)
    }

    fn expected_flow_bounds<S: LayoutScalar>(
        progression: ScrollOriginProgression,
        end_extent: S,
        start_extent: S,
    ) -> (S, S) {
        match progression {
            ScrollOriginProgression::FlowEndward => (-start_extent, end_extent),
            ScrollOriginProgression::FlowStartward => (-end_extent, start_extent),
        }
    }

    fn expected_physical_bounds<S: LayoutScalar>(
        flow_decreasing: bool,
        flow_bounds: (S, S),
    ) -> (S, S) {
        if flow_decreasing {
            (-flow_bounds.1, -flow_bounds.0)
        } else {
            flow_bounds
        }
    }

    fn assert_scalar_all_origin_and_flow_mappings<S: LayoutScalar>() {
        let scrollport: ScrollRectOf<S> = rect(100.0, 300.0, 100.0, 100.0);
        let inline_end: S = scalar(31.0);
        let inline_start: S = scalar(11.0);
        let block_end: S = scalar(47.0);
        let block_start: S = scalar(13.0);

        for flow_axes in all_flow_axes() {
            for inline_progression in [
                ScrollOriginProgression::FlowEndward,
                ScrollOriginProgression::FlowStartward,
            ] {
                for block_progression in [
                    ScrollOriginProgression::FlowEndward,
                    ScrollOriginProgression::FlowStartward,
                ] {
                    let inline_source = axis_range_source(
                        if matches!(flow_axes.inline_axis(), PhysicalAxis::Horizontal) {
                            scalar(100.0)
                        } else {
                            scalar(300.0)
                        },
                        if matches!(flow_axes.inline_axis(), PhysicalAxis::Horizontal) {
                            scalar(200.0)
                        } else {
                            scalar(400.0)
                        },
                        inline_end,
                        inline_start,
                        origin_is_decreasing(flow_axes, LogicalAxis::Inline, inline_progression),
                    );
                    let block_source = axis_range_source(
                        if matches!(flow_axes.block_axis(), PhysicalAxis::Horizontal) {
                            scalar(100.0)
                        } else {
                            scalar(300.0)
                        },
                        if matches!(flow_axes.block_axis(), PhysicalAxis::Horizontal) {
                            scalar(200.0)
                        } else {
                            scalar(400.0)
                        },
                        block_end,
                        block_start,
                        origin_is_decreasing(flow_axes, LogicalAxis::Block, block_progression),
                    );
                    let (x_source, y_source) = match flow_axes.inline_axis() {
                        PhysicalAxis::Horizontal => (inline_source, block_source),
                        PhysicalAxis::Vertical => (block_source, inline_source),
                    };
                    let overflow = ScrollRectOf::try_new(
                        Point::new(x_source.overflow_minimum, y_source.overflow_minimum),
                        Size::new(
                            x_source.overflow_maximum - x_source.overflow_minimum,
                            y_source.overflow_maximum - y_source.overflow_minimum,
                        ),
                    )
                    .unwrap();
                    let subject = ScrollRectOf::try_new(
                        Point::new(x_source.subject_minimum, y_source.subject_minimum),
                        Size::new(
                            x_source.subject_maximum - x_source.subject_minimum,
                            y_source.subject_maximum - y_source.subject_minimum,
                        ),
                    )
                    .unwrap();
                    let mut accumulator = ScrollContributionAccumulatorOf::new(overflow);
                    accumulator.set_active_alignment_subject(PhysicalAxis::Horizontal, subject);
                    accumulator.set_active_alignment_subject(PhysicalAxis::Vertical, subject);
                    let complete_before = accumulator.complete_overflow();
                    let origin_axes = ScrollOriginAxes::new(inline_progression, block_progression);

                    let physical = derive_origin_aware_scroll_range(
                        flow_axes,
                        origin_axes,
                        used_overflow(Overflow::Scroll, Overflow::Scroll),
                        scrollport,
                        &accumulator,
                    )
                    .unwrap();
                    assert_eq!(accumulator.complete_overflow(), complete_before);

                    let flow_relative = flow_axes.flow_relative_scroll_range(physical);
                    let inline_expected =
                        expected_flow_bounds(inline_progression, inline_end, inline_start);
                    let block_expected =
                        expected_flow_bounds(block_progression, block_end, block_start);
                    assert_eq!(flow_relative.inline().minimum(), inline_expected.0);
                    assert_eq!(flow_relative.inline().maximum(), inline_expected.1);
                    assert_eq!(flow_relative.block().minimum(), block_expected.0);
                    assert_eq!(flow_relative.block().maximum(), block_expected.1);

                    let inline_physical = expected_physical_bounds(
                        flow_axes
                            .logical_axis_progression(LogicalAxis::Inline)
                            .is_decreasing(),
                        inline_expected,
                    );
                    let block_physical = expected_physical_bounds(
                        flow_axes
                            .logical_axis_progression(LogicalAxis::Block)
                            .is_decreasing(),
                        block_expected,
                    );
                    let (x_expected, y_expected) = match flow_axes.inline_axis() {
                        PhysicalAxis::Horizontal => (inline_physical, block_physical),
                        PhysicalAxis::Vertical => (block_physical, inline_physical),
                    };
                    assert_eq!(physical.x().minimum(), x_expected.0);
                    assert_eq!(physical.x().maximum(), x_expected.1);
                    assert_eq!(physical.y().minimum(), y_expected.0);
                    assert_eq!(physical.y().maximum(), y_expected.1);
                }
            }
        }
    }

    #[test]
    fn fri05_c02_range_all_origin_progressions_and_flow_mappings_project_exact_bounds() {
        assert_scalar_all_origin_and_flow_mappings::<f32>();
        assert_scalar_all_origin_and_flow_mappings::<f64>();
    }

    fn horizontal_range<S: LayoutScalar>(
        overflow: ScrollRectOf<S>,
        subject: Option<ScrollRectOf<S>>,
        x_overflow: Overflow,
        y_overflow: Overflow,
        origins: ScrollOriginAxes,
    ) -> (ScrollContributionAccumulatorOf<S>, PhysicalScrollRangeOf<S>) {
        let mut accumulator = ScrollContributionAccumulatorOf::new(overflow);
        if let Some(subject) = subject {
            accumulator.set_active_alignment_subject(PhysicalAxis::Horizontal, subject);
        }
        let range = derive_origin_aware_scroll_range(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            origins,
            used_overflow(x_overflow, y_overflow),
            rect(0.0, 0.0, 100.0, 100.0),
            &accumulator,
        )
        .unwrap();
        (accumulator, range)
    }

    fn assert_scalar_subject_bounds<S: LayoutScalar>() {
        let endward = ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        );
        let overflow: ScrollRectOf<S> = rect(-200.0, 0.0, 360.0, 100.0);

        let (_, ordinary) =
            horizontal_range(overflow, None, Overflow::Hidden, Overflow::Visible, endward);
        assert_eq!(ordinary.x().minimum(), S::ZERO);
        assert_eq!(ordinary.x().maximum(), scalar(60.0));

        for (subject, expected_minimum) in [
            (rect(0.0, 0.0, 160.0, 100.0), 0.0),
            (rect(-60.0, 0.0, 160.0, 100.0), -60.0),
            (rect(-30.0, 0.0, 160.0, 100.0), -30.0),
            (rect(0.0, 0.0, 100.0, 100.0), 0.0),
        ] {
            let (accumulator, range) = horizontal_range(
                overflow,
                Some(subject),
                Overflow::Hidden,
                Overflow::Visible,
                endward,
            );
            assert_eq!(range.x().minimum(), scalar(expected_minimum));
            assert_eq!(range.x().maximum(), scalar(60.0));
            assert_bounds(accumulator.complete_overflow(), -200.0, 160.0, 0.0, 100.0);
        }
    }

    #[test]
    fn fri05_c02_range_start_end_center_and_safe_subjects_bound_start_reachability() {
        assert_scalar_subject_bounds::<f32>();
        assert_scalar_subject_bounds::<f64>();
    }

    fn assert_scalar_exposure_and_axis_independence<S: LayoutScalar>() {
        let origins = ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        );
        let overflow: ScrollRectOf<S> = rect(-50.0, -70.0, 200.0, 220.0);
        let subject: ScrollRectOf<S> = rect(-20.0, -30.0, 140.0, 160.0);

        for exposure in [Overflow::Visible, Overflow::Clip] {
            let (_, range) = horizontal_range(
                overflow,
                Some(subject),
                exposure,
                Overflow::Visible,
                origins,
            );
            assert_eq!(
                (range.x().minimum(), range.x().maximum()),
                (S::ZERO, S::ZERO)
            );
        }
        for exposure in [Overflow::Hidden, Overflow::Scroll, Overflow::Auto] {
            let (_, range) = horizontal_range(
                overflow,
                Some(subject),
                exposure,
                Overflow::Visible,
                origins,
            );
            assert_eq!(
                (range.x().minimum(), range.x().maximum()),
                (scalar(-20.0), scalar(50.0))
            );
        }

        let mut accumulator = ScrollContributionAccumulatorOf::new(overflow);
        accumulator.set_active_alignment_subject(PhysicalAxis::Horizontal, subject);
        accumulator.set_active_alignment_subject(PhysicalAxis::Vertical, subject);
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let x_trapped = derive_origin_aware_scroll_range(
            axes,
            origins,
            used_overflow(Overflow::Clip, Overflow::Scroll),
            rect(0.0, 0.0, 100.0, 100.0),
            &accumulator,
        )
        .unwrap();
        assert_eq!(
            (x_trapped.x().minimum(), x_trapped.x().maximum()),
            (S::ZERO, S::ZERO)
        );
        assert_eq!(
            (x_trapped.y().minimum(), x_trapped.y().maximum()),
            (scalar(-30.0), scalar(50.0))
        );
        let y_trapped = derive_origin_aware_scroll_range(
            axes,
            origins,
            used_overflow(Overflow::Auto, Overflow::Visible),
            rect(0.0, 0.0, 100.0, 100.0),
            &accumulator,
        )
        .unwrap();
        assert_eq!(
            (y_trapped.x().minimum(), y_trapped.x().maximum()),
            (scalar(-20.0), scalar(50.0))
        );
        assert_eq!(
            (y_trapped.y().minimum(), y_trapped.y().maximum()),
            (S::ZERO, S::ZERO)
        );
    }

    #[test]
    fn fri05_c02_range_used_exposure_and_partial_axes_are_independent() {
        assert_scalar_exposure_and_axis_independence::<f32>();
        assert_scalar_exposure_and_axis_independence::<f64>();
    }

    fn assert_scalar_reversed_and_zero<S: LayoutScalar>() {
        let reversed = ScrollOriginAxes::new(
            ScrollOriginProgression::FlowStartward,
            ScrollOriginProgression::FlowEndward,
        );
        let (_, range) = horizontal_range::<S>(
            rect(-50.0, 0.0, 350.0, 100.0),
            Some(rect(0.0, 0.0, 120.0, 100.0)),
            Overflow::Scroll,
            Overflow::Visible,
            reversed,
        );
        assert_eq!(
            (range.x().minimum(), range.x().maximum()),
            (scalar(-50.0), scalar(20.0))
        );

        let (_, zero) = horizontal_range::<S>(
            rect(0.0, 0.0, 100.0, 100.0),
            Some(rect(0.0, 0.0, 100.0, 100.0)),
            Overflow::Scroll,
            Overflow::Scroll,
            reversed,
        );
        for value in [
            zero.x().minimum(),
            zero.x().maximum(),
            zero.y().minimum(),
            zero.y().maximum(),
        ] {
            assert_eq!(value, S::ZERO);
            assert!(!value.to_f64().is_sign_negative());
        }
    }

    #[test]
    fn fri05_c02_range_reversed_signs_and_safe_zero_keep_the_initial_anchor() {
        assert_scalar_reversed_and_zero::<f32>();
        assert_scalar_reversed_and_zero::<f64>();
    }
}

#[cfg(test)]
mod fri05_c02_factory_rounding_tests {
    use super::*;
    use crate::{
        Direction, ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStrictness, ScrollSnapType,
        WritingMode,
    };

    const FLOW_MAPPINGS: [(WritingMode, Direction); 10] = [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::HorizontalTb, Direction::Rtl),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Rtl),
        (WritingMode::VerticalLr, Direction::Ltr),
        (WritingMode::VerticalLr, Direction::Rtl),
        (WritingMode::SidewaysRl, Direction::Ltr),
        (WritingMode::SidewaysRl, Direction::Rtl),
        (WritingMode::SidewaysLr, Direction::Ltr),
        (WritingMode::SidewaysLr, Direction::Rtl),
    ];

    fn scalar<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }

    fn rect<S: LayoutScalar>(x: f64, y: f64, width: f64, height: f64) -> ScrollRectOf<S> {
        ScrollRectOf::try_new(
            Point::new(scalar(x), scalar(y)),
            Size::new(scalar(width), scalar(height)),
        )
        .unwrap()
    }

    fn px<S: LayoutScalar>(value: f64) -> OptimalRegionInsetOf<S> {
        OptimalRegionInsetOf::Value(LengthPercentageOf::px(scalar(value)).unwrap())
    }

    fn percent<S: LayoutScalar>(value: f64) -> OptimalRegionInsetOf<S> {
        OptimalRegionInsetOf::Value(
            LengthPercentageOf::from_percent_fraction(scalar(value)).unwrap(),
        )
    }

    fn factory_source<S: LayoutScalar>(flow_axes: FlowAxes) -> CanonicalScrollGeometrySourceOf<S> {
        let padding_box = rect(4.0, 1.0, 34.0, 26.0);
        let mut contributions = ScrollContributionAccumulatorOf::new(padding_box);
        contributions.include_direct_line(rect(-5.0, -7.0, 60.0, 50.0));
        contributions
            .record_final_in_flow_end(flow_axes, LogicalAxis::Inline, scalar(31.0))
            .unwrap();
        contributions
            .record_final_in_flow_end(flow_axes, LogicalAxis::Block, scalar(19.0))
            .unwrap();
        contributions
            .include_terminal_padding(Edges::new(
                scalar(2.0),
                scalar(3.0),
                scalar(4.0),
                scalar(5.0),
            ))
            .unwrap();
        contributions
            .set_active_alignment_subject(PhysicalAxis::Horizontal, rect(-2.0, 0.0, 10.0, 10.0));
        contributions
            .set_active_alignment_subject(PhysicalAxis::Vertical, rect(0.0, -4.0, 10.0, 10.0));

        CanonicalScrollGeometrySourceOf {
            flow_axes,
            computed_overflow: ComputedOverflow::try_new(Overflow::Scroll, Overflow::Auto).unwrap(),
            item_is_replaced: false,
            border_box_size: Size::new(scalar(40.0), scalar(30.0)),
            border: Edges::new(scalar(1.0), scalar(2.0), scalar(3.0), scalar(4.0)),
            padding: Edges::new(scalar(2.0), scalar(3.0), scalar(4.0), scalar(5.0)),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidthOf::try_new(scalar(6.0)).unwrap(),
            settled_auto_scrollbars: SettledAutoScrollbarState { x: true, y: true },
            clip_margin: ClipMarginSourceOf::new(OverflowClipBox::BorderBox, scalar(2.0)),
            scroll_padding: OptimalRegionInsetsOf::new(px(2.0), percent(0.25), px(30.0), px(3.0)),
            contributions,
            origin_axes: ScrollOriginAxes::new(
                ScrollOriginProgression::FlowEndward,
                ScrollOriginProgression::FlowEndward,
            ),
            scroll_snap_type: ScrollSnapType::Enabled {
                axis: ScrollSnapAxis::Both,
                strictness: ScrollSnapStrictness::Mandatory,
            },
            target_border_box: rect(-1.25, 2.5, 8.5, 7.25),
            target_scroll_margin: ScrollMarginOf::try_new(
                scalar(-1.0),
                scalar(2.0),
                scalar(3.0),
                scalar(-4.0),
            )
            .unwrap(),
            target_flow_axes: FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
            target_snap_align: ScrollSnapAlign::new(
                ScrollSnapAlignValue::End,
                ScrollSnapAlignValue::Center,
            ),
            target_snap_stop: ScrollSnapStop::Always,
        }
    }

    fn assert_finite_rect<S: LayoutScalar>(rect: ScrollRectOf<S>) {
        let origin = rect.origin();
        let size = rect.size();
        assert!(origin.x.is_finite());
        assert!(origin.y.is_finite());
        assert!(size.width.is_finite() && size.width >= S::ZERO);
        assert!(size.height.is_finite() && size.height >= S::ZERO);
        assert!((origin.x + size.width).is_finite());
        assert!((origin.y + size.height).is_finite());
    }

    fn assert_rect_contains<S: LayoutScalar>(outer: ScrollRectOf<S>, inner: ScrollRectOf<S>) {
        let outer_origin = outer.origin();
        let outer_size = outer.size();
        let inner_origin = inner.origin();
        let inner_size = inner.size();
        assert!(inner_origin.x >= outer_origin.x);
        assert!(inner_origin.y >= outer_origin.y);
        assert!(inner_origin.x + inner_size.width <= outer_origin.x + outer_size.width);
        assert!(inner_origin.y + inner_size.height <= outer_origin.y + outer_size.height);
    }

    fn assert_canonical_coherence<S: LayoutScalar>(geometry: ScrollGeometryOf<S>) {
        for rect in [
            geometry.border_box,
            geometry.padding_box,
            geometry.content_box,
            geometry.scrollport,
            geometry.scrollable_overflow,
            geometry.optimal_viewing_region,
            geometry.target.border_box(),
        ] {
            assert_finite_rect(rect);
        }
        assert_rect_contains(geometry.border_box, geometry.padding_box);
        assert_rect_contains(geometry.padding_box, geometry.scrollport);
        assert_rect_contains(geometry.scrollport, geometry.content_box);
        assert_rect_contains(geometry.scrollport, geometry.optimal_viewing_region);

        let padding_origin = geometry.padding_box.origin();
        let padding_size = geometry.padding_box.size();
        let scrollport_origin = geometry.scrollport.origin();
        let scrollport_size = geometry.scrollport.size();
        if let Some(top) = geometry.gutters.top {
            assert_eq!(top.origin().y, padding_origin.y);
            assert_eq!(top.origin().x, scrollport_origin.x);
            assert_eq!(top.size().width, scrollport_size.width);
            assert_eq!(top.origin().y + top.size().height, scrollport_origin.y);
        }
        if let Some(right) = geometry.gutters.right {
            assert_eq!(
                right.origin().x,
                scrollport_origin.x + scrollport_size.width
            );
            assert_eq!(right.origin().y, scrollport_origin.y);
            assert_eq!(right.size().height, scrollport_size.height);
            assert_eq!(
                right.origin().x + right.size().width,
                padding_origin.x + padding_size.width
            );
        }
        if let Some(bottom) = geometry.gutters.bottom {
            assert_eq!(bottom.origin().x, scrollport_origin.x);
            assert_eq!(
                bottom.origin().y,
                scrollport_origin.y + scrollport_size.height
            );
            assert_eq!(bottom.size().width, scrollport_size.width);
            assert_eq!(
                bottom.origin().y + bottom.size().height,
                padding_origin.y + padding_size.height
            );
        }
        if let Some(left) = geometry.gutters.left {
            assert_eq!(left.origin().x, padding_origin.x);
            assert_eq!(left.origin().y, scrollport_origin.y);
            assert_eq!(left.size().height, scrollport_size.height);
            assert_eq!(left.origin().x + left.size().width, scrollport_origin.x);
        }
        assert!(geometry.aggregate_reservation.width <= padding_size.width);
        assert!(geometry.aggregate_reservation.height <= padding_size.height);

        for clip in [geometry.overflow_clip.x(), geometry.overflow_clip.y()]
            .into_iter()
            .flatten()
        {
            assert!(clip.minimum().is_finite());
            assert!(clip.maximum().is_finite());
            assert!(clip.minimum() <= clip.maximum());
        }
        for (used, clip, range) in [
            (
                geometry.used_overflow.x(),
                geometry.overflow_clip.x(),
                geometry.physical_range.x(),
            ),
            (
                geometry.used_overflow.y(),
                geometry.overflow_clip.y(),
                geometry.physical_range.y(),
            ),
        ] {
            assert!(range.minimum().is_finite());
            assert!(range.maximum().is_finite());
            assert!(range.minimum() <= S::ZERO && range.maximum() >= S::ZERO);
            match used.value() {
                Overflow::Visible => {
                    assert_eq!(clip, None);
                    assert_eq!((range.minimum(), range.maximum()), (S::ZERO, S::ZERO));
                }
                Overflow::Clip => {
                    assert!(clip.is_some());
                    assert_eq!((range.minimum(), range.maximum()), (S::ZERO, S::ZERO));
                }
                Overflow::Hidden | Overflow::Scroll | Overflow::Auto => {
                    assert!(clip.is_some());
                }
            }
        }

        assert_eq!(geometry.flow_axes, geometry.source.flow_axes);
        assert_eq!(
            geometry.used_overflow,
            UsedOverflow::from_computed(
                geometry.source.computed_overflow,
                geometry.source.item_is_replaced,
            )
        );
        assert_eq!(
            geometry.physical_range,
            derive_origin_aware_scroll_range(
                geometry.flow_axes,
                geometry.source.origin_axes,
                geometry.used_overflow,
                geometry.scrollport,
                &geometry.source.contributions,
            )
            .unwrap()
        );
        let complete = geometry.source.contributions.complete_overflow();
        assert_eq!(
            geometry.scrollable_overflow.origin().x,
            complete.x().minimum()
        );
        assert_eq!(
            geometry.scrollable_overflow.origin().y,
            complete.y().minimum()
        );
        assert_eq!(
            geometry.scrollable_overflow.origin().x + geometry.scrollable_overflow.size().width,
            complete.x().maximum()
        );
        assert_eq!(
            geometry.scrollable_overflow.origin().y + geometry.scrollable_overflow.size().height,
            complete.y().maximum()
        );
        assert_eq!(geometry.scroll_snap_type, geometry.source.scroll_snap_type);
        assert_eq!(
            geometry.target.scroll_margin(),
            geometry.source.target_scroll_margin
        );
        assert_eq!(
            geometry.target.flow_axes(),
            geometry.source.target_flow_axes
        );
        assert_eq!(
            geometry.target.snap_align(),
            geometry.source.target_snap_align
        );
        assert_eq!(
            geometry.target.snap_stop(),
            geometry.source.target_snap_stop
        );
    }

    fn assert_factory_contract<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let source = factory_source(flow_axes);
        let geometry = canonical_scroll_geometry_from_source(source).unwrap();
        assert_canonical_coherence(geometry);

        assert_eq!(geometry.border_box, rect(0.0, 0.0, 40.0, 30.0));
        assert_eq!(geometry.padding_box, rect(4.0, 1.0, 34.0, 26.0));
        assert_eq!(geometry.scrollport, rect(10.0, 1.0, 22.0, 20.0));
        assert_eq!(geometry.content_box, rect(15.0, 3.0, 14.0, 14.0));
        assert_eq!(geometry.gutters.top, None);
        assert_eq!(geometry.gutters.left, Some(rect(4.0, 1.0, 6.0, 20.0)));
        assert_eq!(geometry.gutters.right, Some(rect(32.0, 1.0, 6.0, 20.0)));
        assert_eq!(geometry.gutters.bottom, Some(rect(10.0, 21.0, 22.0, 6.0)));
        assert_eq!(
            geometry.aggregate_reservation,
            Size::new(scalar(12.0), scalar(6.0))
        );
        assert_eq!(geometry.overflow_clip.x().unwrap().minimum(), scalar(10.0));
        assert_eq!(geometry.overflow_clip.x().unwrap().maximum(), scalar(32.0));
        assert_eq!(geometry.overflow_clip.y().unwrap().minimum(), scalar(1.0));
        assert_eq!(geometry.overflow_clip.y().unwrap().maximum(), scalar(21.0));
        assert_eq!(
            geometry.resolved_scroll_padding,
            Edges::new(scalar(2.0), scalar(5.5), scalar(30.0), scalar(3.0))
        );
        assert_eq!(geometry.scrollable_overflow, rect(-5.0, -7.0, 60.0, 50.0));
        assert_eq!(
            (
                geometry.physical_range.x().minimum(),
                geometry.physical_range.x().maximum(),
                geometry.physical_range.y().minimum(),
                geometry.physical_range.y().maximum(),
            ),
            (scalar(-12.0), scalar(23.0), scalar(-5.0), scalar(22.0))
        );
        assert_eq!(geometry.target.border_box(), source.target_border_box);

        let mut partial_source = source;
        partial_source.computed_overflow =
            ComputedOverflow::try_new(Overflow::Visible, Overflow::Clip).unwrap();
        partial_source.scrollbar_gutter = ScrollbarGutter::Auto;
        partial_source.settled_auto_scrollbars = SettledAutoScrollbarState { x: false, y: false };
        let partial = canonical_scroll_geometry_from_source(partial_source).unwrap();
        assert_canonical_coherence(partial);
        assert_eq!(partial.used_overflow.x().value(), Overflow::Visible);
        assert_eq!(partial.used_overflow.y().value(), Overflow::Clip);
        assert_eq!(partial.overflow_clip.x(), None);
        assert_eq!(partial.overflow_clip.y().unwrap().minimum(), scalar(-2.0));
        assert_eq!(partial.overflow_clip.y().unwrap().maximum(), scalar(32.0));

        let mut replaced_source = source;
        replaced_source.computed_overflow =
            ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap();
        replaced_source.item_is_replaced = true;
        let replaced = canonical_scroll_geometry_from_source(replaced_source).unwrap();
        assert_canonical_coherence(replaced);
        assert_eq!(replaced.used_overflow.x().value(), Overflow::Clip);
        assert_eq!(replaced.used_overflow.y().value(), Overflow::Clip);
        assert_eq!(replaced.aggregate_reservation, Size::ZERO);
        assert_eq!(
            (
                replaced.physical_range.x().minimum(),
                replaced.physical_range.x().maximum(),
                replaced.physical_range.y().minimum(),
                replaced.physical_range.y().maximum(),
            ),
            (S::ZERO, S::ZERO, S::ZERO, S::ZERO)
        );

        let mut saturated_source = source;
        saturated_source.computed_overflow =
            ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll).unwrap();
        saturated_source.border_box_size = Size::new(scalar(2.0), scalar(2.0));
        saturated_source.border = Edges::ZERO;
        saturated_source.padding = Edges::ZERO;
        saturated_source.scrollbar_width = ScrollbarWidthOf::try_new(scalar(15.0)).unwrap();
        saturated_source.scroll_padding = OptimalRegionInsetsOf::default();
        saturated_source.contributions =
            ScrollContributionAccumulatorOf::new(rect(0.0, 0.0, 2.0, 2.0));
        saturated_source.target_border_box = rect(0.0, 0.0, 2.0, 2.0);
        let saturated = canonical_scroll_geometry_from_source(saturated_source).unwrap();
        assert_canonical_coherence(saturated);
        assert_eq!(
            saturated.aggregate_reservation,
            Size::new(scalar(2.0), scalar(2.0))
        );
        assert_eq!(saturated.scrollport, rect(1.0, 0.0, 0.0, 0.0));
        assert_eq!(saturated.gutters.left, Some(rect(0.0, 0.0, 1.0, 0.0)));
        assert_eq!(saturated.gutters.right, Some(rect(1.0, 0.0, 1.0, 0.0)));
        assert_eq!(saturated.gutters.bottom, Some(rect(1.0, 0.0, 0.0, 2.0)));
    }

    #[test]
    fn fri05_c02_factory_composes_source_only_geometry_and_all_coherence_invariants() {
        assert_factory_contract::<f32>();
        assert_factory_contract::<f64>();
    }

    fn assert_factory_failure_contract<S>(largest: S)
    where
        S: LayoutScalar + std::panic::UnwindSafe,
    {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let mut non_finite = factory_source(flow_axes);
        non_finite.border_box_size.width = S::INFINITY;
        let outcome =
            std::panic::catch_unwind(move || canonical_scroll_geometry_from_source(non_finite));
        assert!(outcome.is_ok());
        assert!(matches!(
            outcome.unwrap(),
            Err(CanonicalScrollGeometryErrorOf::BoxClipGutter(
                ScrollBoxClipGutterErrorOf::Rect(ScrollRectErrorOf::NonFiniteSize {
                    axis: PhysicalAxis::Horizontal,
                    ..
                })
            ))
        ));

        let mut impossible = factory_source(flow_axes);
        impossible.contributions.container_seed.x = PhysicalContributionIntervalOf {
            minimum: -largest,
            maximum: largest,
        };
        let outcome =
            std::panic::catch_unwind(move || canonical_scroll_geometry_from_source(impossible));
        assert!(outcome.is_ok());
        assert!(matches!(
            outcome.unwrap(),
            Err(CanonicalScrollGeometryErrorOf::ScrollableOverflow(
                ScrollRectErrorOf::NonFiniteSize {
                    axis: PhysicalAxis::Horizontal,
                    ..
                }
            ))
        ));
    }

    #[test]
    fn fri05_c02_factory_returns_typed_finite_failures_without_panic_or_unsupported() {
        assert_factory_failure_contract::<f32>(f32::MAX);
        assert_factory_failure_contract::<f64>(f64::MAX);
    }

    fn expected_round_value<S: LayoutScalar>(value: S) -> S {
        (value + scalar(0.5)).floor()
    }

    fn expected_round_coordinate<S: LayoutScalar>(value: S, cumulative: S) -> S {
        let rounded = expected_round_value(cumulative + value) - expected_round_value(cumulative);
        canonical_scroll_zero(rounded)
    }

    fn expected_round_rect<S: LayoutScalar>(
        rect: ScrollRectOf<S>,
        cumulative_origin: Point<S>,
    ) -> ScrollRectOf<S> {
        let origin = rect.origin();
        let size = rect.size();
        let rounded_origin = Point::new(
            expected_round_coordinate(origin.x, cumulative_origin.x),
            expected_round_coordinate(origin.y, cumulative_origin.y),
        );
        let rounded_end = Point::new(
            expected_round_coordinate(origin.x + size.width, cumulative_origin.x),
            expected_round_coordinate(origin.y + size.height, cumulative_origin.y),
        );
        ScrollRectOf::try_new(
            rounded_origin,
            Size::new(
                (rounded_end.x - rounded_origin.x).max(S::ZERO),
                (rounded_end.y - rounded_origin.y).max(S::ZERO),
            ),
        )
        .unwrap()
    }

    fn expected_round_edges<S: LayoutScalar>(
        edges: Edges<S>,
        border_box_size: Size<S>,
        cumulative_origin: Point<S>,
    ) -> Edges<S> {
        Edges::new(
            expected_round_coordinate(edges.top, cumulative_origin.y),
            canonical_scroll_zero(
                expected_round_value(cumulative_origin.x + border_box_size.width)
                    - expected_round_value(
                        cumulative_origin.x + border_box_size.width - edges.right,
                    ),
            ),
            canonical_scroll_zero(
                expected_round_value(cumulative_origin.y + border_box_size.height)
                    - expected_round_value(
                        cumulative_origin.y + border_box_size.height - edges.bottom,
                    ),
            ),
            expected_round_coordinate(edges.left, cumulative_origin.x),
        )
    }

    fn expected_round_interval<S: LayoutScalar>(
        interval: PhysicalContributionIntervalOf<S>,
        axis: PhysicalAxis,
        cumulative_origin: Point<S>,
    ) -> PhysicalContributionIntervalOf<S> {
        let cumulative = match axis {
            PhysicalAxis::Horizontal => cumulative_origin.x,
            PhysicalAxis::Vertical => cumulative_origin.y,
        };
        PhysicalContributionIntervalOf {
            minimum: expected_round_coordinate(interval.minimum, cumulative),
            maximum: expected_round_coordinate(interval.maximum, cumulative),
        }
    }

    fn expected_round_optional_intervals<S: LayoutScalar>(
        intervals: OptionalPhysicalContributionIntervalsOf<S>,
        cumulative_origin: Point<S>,
    ) -> OptionalPhysicalContributionIntervalsOf<S> {
        OptionalPhysicalContributionIntervalsOf {
            x: intervals.x.map(|interval| {
                expected_round_interval(interval, PhysicalAxis::Horizontal, cumulative_origin)
            }),
            y: intervals.y.map(|interval| {
                expected_round_interval(interval, PhysicalAxis::Vertical, cumulative_origin)
            }),
        }
    }

    fn expected_round_contributions<S: LayoutScalar>(
        contributions: ScrollContributionAccumulatorOf<S>,
        cumulative_origin: Point<S>,
    ) -> ScrollContributionAccumulatorOf<S> {
        let round_end = |end: FinalInFlowEndOf<S>| {
            let cumulative = match end.side.axis() {
                PhysicalAxis::Horizontal => cumulative_origin.x,
                PhysicalAxis::Vertical => cumulative_origin.y,
            };
            FinalInFlowEndOf {
                side: end.side,
                coordinate: expected_round_coordinate(end.coordinate, cumulative),
            }
        };
        ScrollContributionAccumulatorOf {
            container_seed: PhysicalContributionBoundsOf {
                x: expected_round_interval(
                    contributions.container_seed.x,
                    PhysicalAxis::Horizontal,
                    cumulative_origin,
                ),
                y: expected_round_interval(
                    contributions.container_seed.y,
                    PhysicalAxis::Vertical,
                    cumulative_origin,
                ),
            },
            container_range_basis: contributions.container_range_basis,
            propagatable_descendants: expected_round_optional_intervals(
                contributions.propagatable_descendants,
                cumulative_origin,
            ),
            final_in_flow_ends: PhysicalFinalInFlowEndsOf {
                x: contributions.final_in_flow_ends.x.map(round_end),
                y: contributions.final_in_flow_ends.y.map(round_end),
            },
            terminal_padding_overflow: expected_round_optional_intervals(
                contributions.terminal_padding_overflow,
                cumulative_origin,
            ),
            active_alignment_subjects: expected_round_optional_intervals(
                contributions.active_alignment_subjects,
                cumulative_origin,
            ),
        }
    }

    fn fractional_source<S: LayoutScalar>(
        flow_axes: FlowAxes,
        index: usize,
    ) -> CanonicalScrollGeometrySourceOf<S> {
        let mut source = factory_source(flow_axes);
        source.border_box_size = Size::new(scalar(40.4), scalar(30.6));
        source.border = Edges::new(scalar(1.2), scalar(2.3), scalar(3.4), scalar(4.1));
        source.padding = Edges::new(scalar(2.2), scalar(3.3), scalar(4.4), scalar(5.1));
        source.scrollbar_width = ScrollbarWidthOf::try_new(scalar(3.6)).unwrap();
        source.clip_margin = ClipMarginSourceOf::new(OverflowClipBox::ContentBox, scalar(1.6));
        source.scroll_padding =
            OptimalRegionInsetsOf::new(px(1.3), percent(0.2), px(2.7), percent(0.1));
        let padding_box = ScrollRectOf::try_new(
            Point::new(scalar(4.1), scalar(1.2)),
            Size::new(scalar(34.0), scalar(26.0)),
        )
        .unwrap();
        let mut contributions = ScrollContributionAccumulatorOf::new(padding_box);
        contributions.include_direct_line(rect(-5.4, -7.2, 60.8, 50.6));
        contributions
            .record_final_in_flow_end(flow_axes, LogicalAxis::Inline, scalar(31.3))
            .unwrap();
        contributions
            .record_final_in_flow_end(flow_axes, LogicalAxis::Block, scalar(19.7))
            .unwrap();
        contributions
            .include_terminal_padding(source.padding)
            .unwrap();
        contributions
            .set_active_alignment_subject(PhysicalAxis::Horizontal, rect(-2.4, 0.0, 10.2, 10.0));
        contributions
            .set_active_alignment_subject(PhysicalAxis::Vertical, rect(0.0, -3.6, 10.0, 11.8));
        source.contributions = contributions;
        source.origin_axes = ScrollOriginAxes::new(
            if index.is_multiple_of(2) {
                ScrollOriginProgression::FlowEndward
            } else {
                ScrollOriginProgression::FlowStartward
            },
            if index.is_multiple_of(3) {
                ScrollOriginProgression::FlowStartward
            } else {
                ScrollOriginProgression::FlowEndward
            },
        );
        source.target_border_box = rect(-1.4, 2.6, 8.5, 7.25);
        source
    }

    fn expected_rounded_source<S: LayoutScalar>(
        source: CanonicalScrollGeometrySourceOf<S>,
        geometry: ScrollGeometryOf<S>,
        cumulative_origin: Point<S>,
    ) -> CanonicalScrollGeometrySourceOf<S> {
        let original_size = source.border_box_size;
        let rounded_border_box = expected_round_rect(
            ScrollRectOf::try_new(Point::ZERO, original_size).unwrap(),
            cumulative_origin,
        );
        let scrollport_origin = geometry.scrollport.origin();
        let rounded_scroll_padding = expected_round_edges(
            geometry.resolved_scroll_padding,
            geometry.scrollport.size(),
            Point::new(
                cumulative_origin.x + scrollport_origin.x,
                cumulative_origin.y + scrollport_origin.y,
            ),
        );
        let rounded_padding_value =
            |value| OptimalRegionInsetOf::Value(LengthPercentageOf::px(value).unwrap());
        CanonicalScrollGeometrySourceOf {
            border_box_size: rounded_border_box.size(),
            border: expected_round_edges(source.border, original_size, cumulative_origin),
            padding: expected_round_edges(
                source.padding,
                geometry.scrollport.size(),
                Point::new(
                    cumulative_origin.x + scrollport_origin.x,
                    cumulative_origin.y + scrollport_origin.y,
                ),
            ),
            scrollbar_width: ScrollbarWidthOf::try_new(expected_round_value(
                source.scrollbar_width.get(),
            ))
            .unwrap(),
            clip_margin: ClipMarginSourceOf::new(
                source.clip_margin.reference_box,
                expected_round_value(source.clip_margin.margin),
            ),
            scroll_padding: OptimalRegionInsetsOf::new(
                rounded_padding_value(rounded_scroll_padding.top),
                rounded_padding_value(rounded_scroll_padding.right),
                rounded_padding_value(rounded_scroll_padding.bottom),
                rounded_padding_value(rounded_scroll_padding.left),
            ),
            contributions: expected_round_contributions(source.contributions, cumulative_origin),
            target_border_box: expected_round_rect(source.target_border_box, cumulative_origin),
            ..source
        }
    }

    fn assert_rounding_contract<S: LayoutScalar>() {
        for (index, (writing_mode, direction)) in FLOW_MAPPINGS.into_iter().enumerate() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let source: CanonicalScrollGeometrySourceOf<S> = fractional_source(flow_axes, index);
            let unrounded = canonical_scroll_geometry_from_source(source).unwrap();
            let cumulative_origin = Point::new(
                scalar(10.25 + index as f64 * 0.13),
                scalar(-20.35 + index as f64 * 0.17),
            );
            let expected_source = expected_rounded_source(source, unrounded, cumulative_origin);
            let expected = canonical_scroll_geometry_from_source(expected_source).unwrap();
            let actual =
                rebuild_rounded_canonical_scroll_geometry(unrounded, cumulative_origin).unwrap();

            for geometry in [unrounded, actual] {
                let output = crate::NodeOutputOf::<S> {
                    size: geometry.border_box().size(),
                    ..crate::NodeOutputOf::new()
                }
                .with_scroll_geometry(Some(geometry));
                assert_eq!(output.content_box_size(), geometry.content_box().size());
                assert_eq!(output.scrollbar_size(), geometry.scrollbar_size());
            }

            assert_eq!(actual, expected, "{writing_mode:?}/{direction:?}");
            assert_canonical_coherence(actual);
            assert_eq!(actual.source.computed_overflow, source.computed_overflow);
            assert_eq!(actual.source.item_is_replaced, source.item_is_replaced);
            assert_eq!(actual.source.flow_axes, source.flow_axes);
            assert_eq!(actual.source.origin_axes, source.origin_axes);
            assert_eq!(actual.source.scroll_padding, expected_source.scroll_padding);
            assert_ne!(actual.source.scroll_padding, source.scroll_padding);
            for value in [
                actual.resolved_scroll_padding.top,
                actual.resolved_scroll_padding.right,
                actual.resolved_scroll_padding.bottom,
                actual.resolved_scroll_padding.left,
            ] {
                assert_eq!(value, expected_round_value(value));
            }
            assert_eq!(actual.scroll_snap_type, source.scroll_snap_type);
            assert_eq!(actual.target.scroll_margin(), source.target_scroll_margin);
            assert_eq!(actual.target.flow_axes(), source.target_flow_axes);
            assert_eq!(actual.target.snap_align(), source.target_snap_align);
            assert_eq!(actual.target.snap_stop(), source.target_snap_stop);
            assert_eq!(
                actual.target.border_box(),
                expected_source.target_border_box
            );
            assert_eq!(
                actual
                    .source
                    .contributions
                    .propagatable_descendant_intervals(),
                expected_source
                    .contributions
                    .propagatable_descendant_intervals()
            );
        }
    }

    #[test]
    fn fri05_c02_rounding_rebuilds_from_expected_sources_in_all_flows_and_scalar_lanes() {
        assert_rounding_contract::<f32>();
        assert_rounding_contract::<f64>();
    }

    #[test]
    fn fri05_c03_round_cache_ranges_and_output_helpers_agree_after_source_rounding() {
        assert_rounding_contract::<f32>();
        assert_rounding_contract::<f64>();
    }

    fn assert_mismatched_border_box_rebuild_retains_terminal_padding<S: LayoutScalar>() {
        for (flow_axes, padding, final_ends, overflow, range) in [
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                [0.0, 3.0, 4.0, 0.0],
                [30.0, 20.0],
                [0.0, 0.0, 33.0, 24.0],
                [0.0, 23.0, 0.0, 14.0],
            ),
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
                [0.0, 0.0, 4.0, 3.0],
                [0.0, 20.0],
                [-3.0, 0.0, 33.0, 24.0],
                [-3.0, 0.0, 0.0, 14.0],
            ),
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                [0.0, 0.0, 3.0, 4.0],
                [20.0, 0.0],
                [-4.0, 0.0, 34.0, 23.0],
                [-4.0, 0.0, 0.0, 13.0],
            ),
        ] {
            let padding = Edges::new(
                scalar(padding[0]),
                scalar(padding[1]),
                scalar(padding[2]),
                scalar(padding[3]),
            );
            let mut contributions = ScrollContributionAccumulatorOf::new(rect(0.0, 0.0, 8.0, 8.0));
            contributions.include_direct_line(rect(0.0, 0.0, 30.0, 20.0));
            for (axis, coordinate) in [LogicalAxis::Inline, LogicalAxis::Block]
                .into_iter()
                .zip(final_ends)
            {
                contributions
                    .record_final_in_flow_end(flow_axes, axis, scalar(coordinate))
                    .unwrap();
            }
            contributions.include_terminal_padding(padding).unwrap();

            let source = CanonicalScrollGeometrySourceOf {
                flow_axes,
                computed_overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden)
                    .unwrap(),
                border_box_size: Size::splat(scalar(8.0)),
                border: Edges::ZERO,
                padding,
                scrollbar_gutter: ScrollbarGutter::Auto,
                scrollbar_width: ScrollbarWidthOf::try_new(S::ZERO).unwrap(),
                settled_auto_scrollbars: SettledAutoScrollbarState::INITIAL,
                clip_margin: ClipMarginSourceOf::default(),
                scroll_padding: OptimalRegionInsetsOf::default(),
                contributions,
                origin_axes: ScrollOriginAxes::new(
                    ScrollOriginProgression::FlowEndward,
                    ScrollOriginProgression::FlowEndward,
                ),
                scroll_snap_type: ScrollSnapType::default(),
                target_border_box: rect(0.0, 0.0, 8.0, 8.0),
                target_flow_axes: flow_axes,
                ..factory_source(flow_axes)
            };
            let original = canonical_scroll_geometry_from_source(source).unwrap();
            let original_target = original.target();
            let rebuilt_size = Size::splat(scalar(10.0));
            let rebuilt = rebuild_canonical_scroll_geometry_for_border_box(
                original,
                rebuilt_size,
                Edges::ZERO,
                padding,
            )
            .unwrap();
            let expected_overflow = rect(overflow[0], overflow[1], overflow[2], overflow[3]);

            assert_eq!(
                rebuilt.scrollable_overflow(),
                expected_overflow,
                "{flow_axes:?}"
            );
            assert_eq!(
                (
                    rebuilt.physical_range().x().minimum(),
                    rebuilt.physical_range().x().maximum(),
                    rebuilt.physical_range().y().minimum(),
                    rebuilt.physical_range().y().maximum(),
                ),
                range.map(scalar::<S>).into(),
                "{flow_axes:?}"
            );
            assert_eq!(
                rebuilt
                    .source
                    .contributions
                    .content_size_from_anchor(rebuilt.content_box().origin())
                    .unwrap(),
                expected_overflow.size(),
                "{flow_axes:?}"
            );
            assert_eq!(
                rebuilt.source.contributions.propagatable_descendants,
                OptionalPhysicalContributionIntervalsOf {
                    x: Some(PhysicalContributionIntervalOf {
                        minimum: S::ZERO,
                        maximum: scalar(30.0),
                    }),
                    y: Some(PhysicalContributionIntervalOf {
                        minimum: S::ZERO,
                        maximum: scalar(20.0),
                    }),
                },
                "direct content remains one interval per axis for {flow_axes:?}"
            );
            assert_canonical_coherence(rebuilt);

            let output = crate::NodeOutputOf::<S>::new().with_scroll_geometry(Some(rebuilt));
            assert_eq!(
                output.content_box_size(),
                rebuilt.content_box().size(),
                "{flow_axes:?}"
            );
            assert_eq!(
                output.scrollbar_size(),
                rebuilt.scrollbar_size(),
                "{flow_axes:?}"
            );
            assert_eq!(
                rebuilt.target().border_box(),
                rebuilt.border_box(),
                "{flow_axes:?}"
            );
            assert_eq!(
                rebuilt.target().scroll_margin(),
                original_target.scroll_margin()
            );
            assert_eq!(rebuilt.target().flow_axes(), original_target.flow_axes());
            assert_eq!(rebuilt.target().snap_align(), original_target.snap_align());
            assert_eq!(rebuilt.target().snap_stop(), original_target.snap_stop());
        }
    }

    #[test]
    fn fri05_c03_round_cache_mismatched_border_box_reapplies_terminal_padding_in_both_scalar_lanes()
    {
        assert_mismatched_border_box_rebuild_retains_terminal_padding::<f32>();
        assert_mismatched_border_box_rebuild_retains_terminal_padding::<f64>();
    }

    fn assert_nested_padding_rounding_uses_absolute_boundaries<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let cumulative_origin = Point::new(scalar(0.25), scalar(0.25));

        let mut no_gutter = factory_source(flow_axes);
        no_gutter.computed_overflow =
            ComputedOverflow::try_new(Overflow::Clip, Overflow::Clip).unwrap();
        no_gutter.border_box_size = Size::new(scalar(10.0), scalar(10.0));
        no_gutter.border = Edges::new(scalar(0.40), scalar(0.40), scalar(0.40), scalar(0.40));
        no_gutter.padding = Edges::new(scalar(0.40), scalar(0.40), scalar(0.40), scalar(0.40));
        no_gutter.scrollbar_gutter = ScrollbarGutter::Auto;
        no_gutter.scrollbar_width = ScrollbarWidthOf::try_new(scalar(0.60)).unwrap();
        no_gutter.clip_margin = ClipMarginSourceOf::new(OverflowClipBox::ContentBox, S::ZERO);
        no_gutter.scroll_padding = OptimalRegionInsetsOf::default();
        no_gutter.contributions =
            ScrollContributionAccumulatorOf::new(rect(0.40, 0.40, 9.20, 9.20));
        no_gutter.target_border_box = rect(0.0, 0.0, 10.0, 10.0);

        let no_gutter = canonical_scroll_geometry_from_source(no_gutter).unwrap();
        let rounded_no_gutter =
            rebuild_rounded_canonical_scroll_geometry(no_gutter, cumulative_origin).unwrap();

        // Independent absolute-boundary oracle: 0.25 + 0.40 + 0.40 rounds to
        // 1 on each start side, while 0.25 + 10.0 - 0.40 - 0.40 rounds to 9
        // on each end side. These constants do not use either edge-rounding helper.
        assert_eq!(rounded_no_gutter.padding_box, rect(1.0, 1.0, 9.0, 9.0));
        assert_eq!(rounded_no_gutter.content_box, rect(1.0, 1.0, 8.0, 8.0));
        let x_clip = rounded_no_gutter.overflow_clip.x().unwrap();
        let y_clip = rounded_no_gutter.overflow_clip.y().unwrap();
        assert_eq!(
            (x_clip.minimum(), x_clip.maximum()),
            (scalar(1.0), scalar(9.0))
        );
        assert_eq!(
            (y_clip.minimum(), y_clip.maximum()),
            (scalar(1.0), scalar(9.0))
        );

        let mut guttered = no_gutter.source;
        guttered.computed_overflow =
            ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll).unwrap();
        guttered.border = Edges::new(scalar(0.10), scalar(0.30), scalar(0.30), scalar(0.10));
        guttered.padding = Edges::new(scalar(0.40), scalar(0.80), scalar(0.80), scalar(0.40));
        guttered.scrollbar_gutter = ScrollbarGutter::StableBothEdges;
        guttered.contributions = ScrollContributionAccumulatorOf::new(rect(0.10, 0.10, 9.60, 9.60));

        let guttered = canonical_scroll_geometry_from_source(guttered).unwrap();
        let rounded_guttered =
            rebuild_rounded_canonical_scroll_geometry(guttered, cumulative_origin).unwrap();

        // The x boundaries 0.25 + 0.10 + 0.60 + 0.40 and
        // 0.25 + 10.0 - 0.30 - 0.60 - 0.80 round to 1 and 9. The
        // corresponding y content boundaries also round to 1 and 9.
        assert_eq!(rounded_guttered.scrollport, rect(1.0, 0.0, 8.0, 9.0));
        assert_eq!(rounded_guttered.content_box, rect(1.0, 1.0, 8.0, 8.0));

        let mut orthogonal_guttered = guttered.source;
        orthogonal_guttered.flow_axes = FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);
        let orthogonal_guttered =
            canonical_scroll_geometry_from_source(orthogonal_guttered).unwrap();
        let rounded_orthogonal =
            rebuild_rounded_canonical_scroll_geometry(orthogonal_guttered, cumulative_origin)
                .unwrap();

        assert_eq!(rounded_orthogonal.scrollport, rect(0.0, 1.0, 9.0, 8.0));
        assert_eq!(rounded_orthogonal.content_box, rect(1.0, 1.0, 8.0, 8.0));
    }

    #[test]
    fn fri05_c02_rounding_nested_padding_uses_absolute_boundaries_in_both_scalar_lanes() {
        assert_nested_padding_rounding_uses_absolute_boundaries::<f32>();
        assert_nested_padding_rounding_uses_absolute_boundaries::<f64>();
    }

    fn assert_rounding_failure<S>(largest: S)
    where
        S: LayoutScalar + std::panic::UnwindSafe,
    {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let mut source = factory_source(flow_axes);
        source.target_border_box =
            ScrollRectOf::try_new(Point::new(largest / scalar(2.0), S::ZERO), Size::ZERO).unwrap();
        let geometry = canonical_scroll_geometry_from_source(source).unwrap();
        let outcome = std::panic::catch_unwind(move || {
            rebuild_rounded_canonical_scroll_geometry(geometry, Point::new(largest, S::ZERO))
        });
        assert!(outcome.is_ok());
        assert!(matches!(
            outcome.unwrap(),
            Err(CanonicalScrollGeometryErrorOf::RoundedRect {
                fact: CanonicalScrollRectFact::TargetBorderBox,
                ..
            })
        ));
    }

    #[test]
    fn fri05_c02_rounding_reports_finite_coordinate_overflow_without_panic() {
        assert_rounding_failure::<f32>(f32::MAX);
        assert_rounding_failure::<f64>(f64::MAX);
    }

    #[test]
    fn fri05_c03_root_block_legacy_absence_factory_has_no_migration_or_rounding_adapter() {
        let source = include_str!("scroll.rs");
        let production = source
            .split("#[cfg(test)]\nmod fri05_c02_carrier_tests")
            .next()
            .unwrap();
        assert_eq!(
            production
                .matches("fn canonical_scroll_geometry_from_source<")
                .count(),
            1
        );
        assert_eq!(
            production
                .matches("canonical_scroll_geometry_from_source(")
                .count(),
            3,
            "measured leaf, retained-source rebuild, and rounding are the production callers"
        );
        assert_eq!(
            production
                .matches("fn rebuild_rounded_canonical_scroll_geometry<")
                .count(),
            1
        );
        assert_eq!(
            production
                .matches("rebuild_rounded_canonical_scroll_geometry(")
                .count(),
            0,
            "callers invoke canonical rounding directly without a compatibility wrapper"
        );
        for forbidden in [
            "pub struct CanonicalScrollGeometrySourceOf",
            "pub fn canonical_scroll_geometry_from_source",
            "impl<S: LayoutScalar> CanonicalScrollGeometrySourceOf",
            "impl<S: LayoutScalar> Default for CanonicalScrollGeometrySourceOf",
        ] {
            assert!(
                !production.contains(forbidden),
                "unexpected surface: {forbidden}"
            );
        }
        for removed in [
            "ScrollUnsupportedFeature",
            "scroll_geometry_from_layout",
            "round_scroll_geometry",
            "ScrollGeometryOf::new",
            "MeasuredLeafProvenance",
        ] {
            assert!(!production.contains(removed), "retained adapter: {removed}");
        }
        let public_front_door = include_str!("lib.rs");
        assert!(!public_front_door.contains("CanonicalScrollGeometry"));
    }
}
