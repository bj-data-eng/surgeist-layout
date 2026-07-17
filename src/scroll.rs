use super::{
    ComputedOverflow, DefaultScalar, Direction, Edges, FlowAxes, LayoutScalar, LengthPercentageOf,
    LogicalAxis, NumericResolutionOf, Overflow, OverflowClipBox, PercentageBasisOf, PhysicalAxis,
    PhysicalSide, Point, ScrollMarginOf, ScrollSnapAlign, ScrollSnapStop, ScrollbarGutter,
    ScrollbarWidthOf, Size,
};
use crate::geometry::LogicalEdgesOf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollUnsupportedFeature {
    InvalidScrollRect,
    InvalidScrollGeometry,
}

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
    pub fn new(origin: Point<S>, size: Size<S>) -> Result<Self, ScrollUnsupportedFeature> {
        Self::try_new(origin, size).map_err(|_| ScrollUnsupportedFeature::InvalidScrollRect)
    }

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

/// Immutable layout-produced geometry and metadata for one scroll target.
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
struct SettledAutoScrollbarState {
    x: bool,
    y: bool,
}

impl SettledAutoScrollbarState {
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
struct PhysicalGutterRectsOf<S: LayoutScalar> {
    top: Option<ScrollRectOf<S>>,
    right: Option<ScrollRectOf<S>>,
    bottom: Option<ScrollRectOf<S>>,
    left: Option<ScrollRectOf<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ClipMarginSourceOf<S: LayoutScalar> {
    reference_box: OverflowClipBox,
    margin: S,
}

impl<S: LayoutScalar> ClipMarginSourceOf<S> {
    fn new(reference_box: OverflowClipBox, margin: S) -> Self {
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
enum OptimalRegionInsetOf<S: LayoutScalar> {
    #[default]
    Auto,
    Value(LengthPercentageOf<S>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OptimalRegionInsetsOf<S: LayoutScalar> {
    top: OptimalRegionInsetOf<S>,
    right: OptimalRegionInsetOf<S>,
    bottom: OptimalRegionInsetOf<S>,
    left: OptimalRegionInsetOf<S>,
}

impl<S: LayoutScalar> OptimalRegionInsetsOf<S> {
    fn new(
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
struct ScrollBoxClipGutterResultOf<S: LayoutScalar> {
    border_box: ScrollRectOf<S>,
    padding_box: ScrollRectOf<S>,
    content_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
    gutters: PhysicalGutterRectsOf<S>,
    aggregate_reservation: Size<S>,
    overflow_clip: OverflowClipOf<S>,
    resolved_scroll_padding: Edges<S>,
    optimal_viewing_region: ScrollRectOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ScrollBoxClipGutterErrorOf<S: LayoutScalar> {
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
    let (_, padding_box) = inset_scroll_rect_saturated(border_box, source.border)?;
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
    let (_, content_box) = inset_scroll_rect_saturated(scrollport, source.padding)?;
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
) -> Result<PhysicalGutterRectsOf<S>, ScrollBoxClipGutterErrorOf<S>> {
    Ok(PhysicalGutterRectsOf {
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

    const fn from_used_overflow(overflow: UsedOverflowAxis) -> Self {
        Self {
            exposure: if overflow.exposes_scroll_range() {
                ScrollOverflowExposure::ScrollableClip
            } else if overflow.clips_contents() {
                ScrollOverflowExposure::ClipOnly
            } else {
                ScrollOverflowExposure::Visible
            },
        }
    }

    pub const fn from_overflow(overflow: Overflow) -> Result<Self, ScrollUnsupportedFeature> {
        Ok(Self::from_used_overflow(UsedOverflowAxis {
            value: overflow,
        }))
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
    overflow: ComputedOverflow,
    item_is_replaced: bool,
) -> Result<ScrollContainerFacts, ScrollUnsupportedFeature> {
    let overflow = UsedOverflow::from_computed(overflow, item_is_replaced);
    Ok(ScrollContainerFacts::new(
        ScrollContainerAxis::from_used_overflow(overflow.x()),
        ScrollContainerAxis::from_used_overflow(overflow.y()),
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
pub(crate) fn scrollable_overflow_from_layout_content_size<S: LayoutScalar>(
    direction: Direction,
    overflow: UsedOverflow,
    border_box_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    scrollbar_width_value: S,
    content_size: Size<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let reservation =
        ScrollbarReservationOf::from_used_overflow(overflow, scrollbar_width_value, direction);
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
    overflow: ComputedOverflow,
    item_is_replaced: bool,
    border_box_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    scrollbar_width_value: S,
    scrollable_overflow: ScrollRectOf<S>,
) -> Result<ScrollGeometryOf<S>, ScrollUnsupportedFeature> {
    let container = scroll_container_facts_from_overflow(overflow, item_is_replaced)?;
    let reservation = ScrollbarReservationOf::from_overflow(
        overflow,
        item_is_replaced,
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
