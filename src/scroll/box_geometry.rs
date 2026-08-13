use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UsedOverflowGutter {
    None,
    StableOnly,
    Conditional,
    Forced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UsedOverflowAxis {
    pub(super) value: Overflow,
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
    pub(super) x: UsedOverflowAxis,
    pub(super) y: UsedOverflowAxis,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SettledAutoScrollbarState {
    pub(super) x: bool,
    pub(super) y: bool,
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
pub(super) struct AutoScrollbarOverflowObservation {
    x: bool,
    y: bool,
}

impl AutoScrollbarOverflowObservation {
    pub(super) fn from_range<S: LayoutScalar>(range: PhysicalScrollRangeOf<S>) -> Self {
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

pub(super) fn used_overflow_at(overflow: UsedOverflow, axis: PhysicalAxis) -> UsedOverflowAxis {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ClipMarginSourceOf<S: LayoutScalar> {
    pub(super) reference_box: OverflowClipBox,
    pub(super) margin: S,
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

    pub(crate) fn from_scroll_padding(scroll_padding: crate::ScrollPaddingOf<S>) -> Self {
        fn inset<S: LayoutScalar>(
            value: crate::ScrollPaddingValueOf<S>,
        ) -> OptimalRegionInsetOf<S> {
            match value {
                crate::ScrollPaddingValueOf::Auto => OptimalRegionInsetOf::Auto,
                crate::ScrollPaddingValueOf::Value(value) => OptimalRegionInsetOf::Value(value),
            }
        }

        Self::new(
            inset(scroll_padding.top()),
            inset(scroll_padding.right()),
            inset(scroll_padding.bottom()),
            inset(scroll_padding.left()),
        )
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
pub(super) struct ScrollBoxClipGutterSourceOf<S: LayoutScalar> {
    pub(super) flow_axes: FlowAxes,
    pub(super) used_overflow: UsedOverflow,
    pub(super) border_box_size: Size<S>,
    pub(super) border: Edges<S>,
    pub(super) padding: Edges<S>,
    pub(super) scrollbar_gutter: ScrollbarGutter,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
    pub(super) settled_auto_scrollbars: SettledAutoScrollbarState,
    pub(super) clip_margin: ClipMarginSourceOf<S>,
    pub(super) optimal_region_insets: OptimalRegionInsetsOf<S>,
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
    pub(super) border_box: ScrollRectOf<S>,
    pub(super) padding_box: ScrollRectOf<S>,
    pub(super) effective_border: Edges<S>,
    pub(super) effective_padding: Edges<S>,
    pub(super) effective_gutter: Edges<S>,
    pub(super) scrollport: ScrollRectOf<S>,
    pub(super) content_box: ScrollRectOf<S>,
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
pub(super) struct ScrollBoxClipGutterResultOf<S: LayoutScalar> {
    pub(super) border_box: ScrollRectOf<S>,
    pub(super) padding_box: ScrollRectOf<S>,
    pub(super) content_box: ScrollRectOf<S>,
    pub(super) scrollport: ScrollRectOf<S>,
    pub(super) effective_border: Edges<S>,
    pub(super) effective_padding: Edges<S>,
    pub(super) effective_reservation: Edges<S>,
    pub(super) gutters: ScrollbarGutterRectsOf<S>,
    pub(super) aggregate_reservation: Size<S>,
    pub(super) overflow_clip: OverflowClipOf<S>,
    pub(super) resolved_scroll_padding: Edges<S>,
    pub(super) optimal_viewing_region: ScrollRectOf<S>,
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

pub(super) fn derive_scroll_box_clip_gutter<S: LayoutScalar>(
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
        return (canonical_zero(start), canonical_zero(end));
    }

    let largest = start.max(end);
    let start_share = start / largest;
    let end_share = end / largest;
    let effective_start = dimension * (start_share / (start_share + end_share));
    let effective_end = (dimension - effective_start).max(S::ZERO);
    (
        canonical_zero(effective_start),
        canonical_zero(effective_end),
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
        range: PhysicalScrollAxisRangeOf::new(canonical_zero(minimum), canonical_zero(maximum)),
    }))
}

pub(super) fn scroll_rect_axis_interval<S: LayoutScalar>(
    rect: ScrollRectOf<S>,
    axis: PhysicalAxis,
) -> (S, S) {
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
            Ok(canonical_zero(value.max(S::ZERO)))
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
