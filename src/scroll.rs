use super::{
    ComputedOverflow, DefaultScalar, Direction, Edges, FlowAxes, LayoutScalar, LengthPercentageOf,
    LogicalAxis, NodeInputOf, NumericResolutionOf, Overflow, OverflowClipBox, PercentageBasisOf,
    PhysicalAxis, PhysicalSide, Point, ScrollMarginOf, ScrollSnapAlign, ScrollSnapStop,
    ScrollSnapType, ScrollbarGutter, ScrollbarWidthOf, Size,
    scalar::{canonical_zero, round_layout_coordinate},
};
use crate::geometry::LogicalEdgesOf;

mod box_geometry;
mod model;

#[cfg(test)]
pub(crate) use box_geometry::UsedOverflowGutter;
use box_geometry::{
    AutoScrollbarOverflowObservation, ScrollBoxClipGutterSourceOf, derive_scroll_box_clip_gutter,
    scroll_rect_axis_interval, used_overflow_at,
};
pub(crate) use box_geometry::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, ClipMarginSourceOf,
    MeasuredLeafContentBoxInsetSourceOf, OptimalRegionInsetOf, OptimalRegionInsetsOf,
    ScrollBoxClipGutterErrorOf, ScrollbarReservationOf, SettledAutoScrollbarState, UsedOverflow,
    UsedOverflowAxis, content_box_inset_with_scrollbar, measured_leaf_content_box_inset,
    scrollbar_size_from_overflow,
};

use model::validate_physical_scroll_range;
pub use model::{
    FlowRelativeScrollAxisRange, FlowRelativeScrollAxisRangeOf, FlowRelativeScrollOffset,
    FlowRelativeScrollOffsetOf, FlowRelativeScrollRange, FlowRelativeScrollRangeOf, OverflowClip,
    OverflowClipOf, PhysicalClipAxis, PhysicalClipAxisOf, PhysicalScrollAxisRange,
    PhysicalScrollAxisRangeOf, PhysicalScrollOffset, PhysicalScrollOffsetOf, PhysicalScrollRange,
    PhysicalScrollRangeOf, ScrollCoordinateError, ScrollCoordinateErrorOf, ScrollGeometry,
    ScrollGeometryOf, ScrollRect, ScrollRectError, ScrollRectErrorOf, ScrollRectOf,
    ScrollTargetGeometry, ScrollTargetGeometryOf, ScrollbarGutterRects, ScrollbarGutterRectsOf,
};

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
            minimum: canonical_zero(minimum),
            maximum: canonical_zero(maximum),
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
    ScrollContainerAxes(UsedOverflow),
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

    fn exclude_reserved_gutter_from_scroll_container_axes(&mut self, overflow: UsedOverflow) {
        self.container_range_basis = ContainerRangeBasis::ScrollContainerAxes(overflow);
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
            coordinate: canonical_zero(coordinate),
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
            let padding = padding.at_physical_side(end.side);
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
                    minimum: canonical_zero(coordinate),
                    maximum: canonical_zero(coordinate),
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
        let mut container_seed = match self.container_range_basis {
            ContainerRangeBasis::PaddingBox => self.container_seed,
            ContainerRangeBasis::Scrollport => PhysicalContributionBoundsOf::from_rect(scrollport),
            ContainerRangeBasis::ScrollContainerAxes(_) => self.container_seed,
        };
        if let ContainerRangeBasis::ScrollContainerAxes(overflow) = self.container_range_basis {
            let scrollport = PhysicalContributionBoundsOf::from_rect(scrollport);
            if matches!(overflow.x().value(), Overflow::Scroll | Overflow::Auto) {
                container_seed.x = scrollport.x;
            }
            if matches!(overflow.y().value(), Overflow::Scroll | Overflow::Auto) {
                container_seed.y = scrollport.y;
            }
        }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalScrollRangeSeedPolicy {
    IncludeReservedGutter,
    ExcludeReservedGutter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CanonicalRetainedScrollSourceOf<'a, S: LayoutScalar> {
    Existing(&'a ScrollGeometryOf<S>),
    Reconstruct { content_size: Size<S> },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalScrollSourceBuilderOf<S: LayoutScalar> {
    flow_axes: FlowAxes,
    computed_overflow: ComputedOverflow,
    item_is_replaced: bool,
    border_box_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
    scrollbar_gutter: ScrollbarGutter,
    scrollbar_width: ScrollbarWidthOf<S>,
    settled_auto_scrollbars: SettledAutoScrollbarState,
    clip_margin: ClipMarginSourceOf<S>,
    scroll_padding: OptimalRegionInsetsOf<S>,
    origin_axes: ScrollOriginAxes,
    scroll_snap_type: ScrollSnapType,
    target_scroll_margin: ScrollMarginOf<S>,
    target_snap_align: ScrollSnapAlign,
    target_snap_stop: ScrollSnapStop,
}

impl<S: LayoutScalar> CanonicalScrollSourceBuilderOf<S> {
    #[must_use]
    pub(crate) fn for_node(
        style: &NodeInputOf<S>,
        flow_axes: FlowAxes,
        border_box_size: Size<S>,
        border: Edges<S>,
        padding: Edges<S>,
        settled_auto_scrollbars: SettledAutoScrollbarState,
        origin_axes: ScrollOriginAxes,
    ) -> Self {
        Self {
            flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            border_box_size,
            border,
            padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars,
            clip_margin: ClipMarginSourceOf::new(
                style.overflow_clip_margin.clip_box(),
                style.overflow_clip_margin.margin(),
            ),
            scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
            origin_axes,
            scroll_snap_type: style.scroll_snap_type,
            target_scroll_margin: style.scroll_margin,
            target_snap_align: style.scroll_snap_align,
            target_snap_stop: style.scroll_snap_stop,
        }
    }

    pub(crate) fn geometry_from_contributions(
        self,
        contributions: ScrollContributionAccumulatorOf<S>,
        target_border_box: ScrollRectOf<S>,
    ) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
        canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
            flow_axes: self.flow_axes,
            computed_overflow: self.computed_overflow,
            item_is_replaced: self.item_is_replaced,
            border_box_size: self.border_box_size,
            border: self.border,
            padding: self.padding,
            scrollbar_gutter: self.scrollbar_gutter,
            scrollbar_width: self.scrollbar_width,
            settled_auto_scrollbars: self.settled_auto_scrollbars,
            clip_margin: self.clip_margin,
            scroll_padding: self.scroll_padding,
            contributions,
            origin_axes: self.origin_axes,
            scroll_snap_type: self.scroll_snap_type,
            target_border_box,
            target_scroll_margin: self.target_scroll_margin,
            target_flow_axes: self.flow_axes,
            target_snap_align: self.target_snap_align,
            target_snap_stop: self.target_snap_stop,
        })
    }

    pub(crate) fn geometry_from_retained_source(
        self,
        source: CanonicalRetainedScrollSourceOf<'_, S>,
        range_seed_policy: CanonicalScrollRangeSeedPolicy,
    ) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
        let content_size = match source {
            CanonicalRetainedScrollSourceOf::Existing(geometry) => {
                if geometry.border_box().origin() == Point::ZERO
                    && geometry.border_box().size() == self.border_box_size
                {
                    return Ok(*geometry);
                }
                return rebuild_canonical_scroll_geometry_for_border_box(
                    *geometry,
                    self.border_box_size,
                    self.border,
                    self.padding,
                );
            }
            CanonicalRetainedScrollSourceOf::Reconstruct { content_size } => content_size,
        };

        let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes: self.flow_axes,
            computed_overflow: self.computed_overflow,
            item_is_replaced: self.item_is_replaced,
            border_box_size: self.border_box_size,
            border: self.border,
            padding: self.padding,
            scrollbar_gutter: self.scrollbar_gutter,
            scrollbar_width: self.scrollbar_width,
            settled_auto_scrollbars: self.settled_auto_scrollbars,
        })?;
        let content_box = scroll_box.content_box();
        let direct_content = ScrollRectOf::try_new(
            content_box.origin(),
            Size::new(
                content_box.size().width.max(content_size.width),
                content_box.size().height.max(content_size.height),
            ),
        )
        .map_err(CanonicalScrollGeometryErrorOf::ScrollableOverflow)?;
        let mut contributions = ScrollContributionAccumulatorOf::new(scroll_box.padding_box());
        match range_seed_policy {
            CanonicalScrollRangeSeedPolicy::IncludeReservedGutter => {}
            CanonicalScrollRangeSeedPolicy::ExcludeReservedGutter => {
                contributions.exclude_reserved_gutter_from_range();
            }
        }
        contributions.include_direct_line(direct_content);
        self.geometry_from_contributions(contributions, scroll_box.border_box())
    }
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
    contributions.exclude_reserved_gutter_from_scroll_container_axes(used_overflow);
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
    let rounded_scrollbar_width = round_layout_coordinate(source.scrollbar_width.get());
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
            round_layout_coordinate(source.clip_margin.margin),
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
        canonical_zero(
            round_layout_coordinate(cumulative_origin.x + border_box_size.width)
                - round_layout_coordinate(
                    cumulative_origin.x + border_box_size.width - edges.right,
                ),
        ),
        canonical_zero(
            round_layout_coordinate(cumulative_origin.y + border_box_size.height)
                - round_layout_coordinate(
                    cumulative_origin.y + border_box_size.height - edges.bottom,
                ),
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
    canonical_zero(
        round_layout_coordinate(cumulative + value) - round_layout_coordinate(cumulative),
    )
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

    fn assert_fri06_mr02_physical_edge_scroll_terminal_padding<S: LayoutScalar>() {
        let padding = Edges::new(scalar(11.0), scalar(22.0), scalar(33.0), scalar(44.0));
        let cases = [
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
                LogicalAxis::Inline,
                -100.0,
                [0.0, 0.0, -111.0, 0.0],
            ),
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                LogicalAxis::Inline,
                100.0,
                [0.0, 122.0, 0.0, 0.0],
            ),
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                LogicalAxis::Block,
                100.0,
                [0.0, 0.0, 0.0, 133.0],
            ),
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                LogicalAxis::Block,
                -100.0,
                [-144.0, 0.0, 0.0, 0.0],
            ),
        ];

        for (flow_axes, logical_axis, coordinate, expected) in cases {
            let mut accumulator =
                ScrollContributionAccumulatorOf::<S>::new(rect(0.0, 0.0, 0.0, 0.0));
            accumulator
                .record_final_in_flow_end(flow_axes, logical_axis, scalar(coordinate))
                .unwrap();
            accumulator.include_terminal_padding(padding).unwrap();
            assert_bounds(
                accumulator.complete_overflow(),
                expected[0],
                expected[1],
                expected[2],
                expected[3],
            );
        }
    }

    #[test]
    fn fri06_mr02_physical_edge_scroll_terminal_padding_selects_four_distinct_sentinels() {
        assert_fri06_mr02_physical_edge_scroll_terminal_padding::<f32>();
        assert_fri06_mr02_physical_edge_scroll_terminal_padding::<f64>();
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
        canonical_zero(rounded)
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
            canonical_zero(
                expected_round_value(cumulative_origin.x + border_box_size.width)
                    - expected_round_value(
                        cumulative_origin.x + border_box_size.width - edges.right,
                    ),
            ),
            canonical_zero(
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

    fn assert_fri06_mr02_layout_round_scroll_publication<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let source = fractional_source(flow_axes, 0);
        let unrounded = canonical_scroll_geometry_from_source(source).unwrap();
        let cumulative_origin = Point::new(scalar(10.25), scalar(-20.35));
        let expected_source = expected_rounded_source(source, unrounded, cumulative_origin);
        let expected = canonical_scroll_geometry_from_source(expected_source).unwrap();

        let actual =
            rebuild_rounded_canonical_scroll_geometry(unrounded, cumulative_origin).unwrap();
        let output = crate::NodeOutputOf::<S> {
            size: actual.border_box().size(),
            ..crate::NodeOutputOf::new()
        }
        .with_scroll_geometry(Some(actual));

        assert_eq!(actual.physical_range(), expected.physical_range());
        assert_eq!(actual.scrollable_overflow(), expected.scrollable_overflow());
        assert_eq!(output.scroll_geometry, Some(actual));
        assert_eq!(output.content_box_size(), actual.content_box().size());
        assert_eq!(output.scrollbar_size(), actual.scrollbar_size());
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

    #[test]
    fn fri06_mr02_layout_round_scroll_ranges_and_publication_preserve_cumulative_source_rounding() {
        assert_fri06_mr02_layout_round_scroll_publication::<f32>();
        assert_fri06_mr02_layout_round_scroll_publication::<f64>();
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
    fn fri06_mr02_layout_round_scroll_overflow_preserves_typed_error_without_panic() {
        assert_rounding_failure::<f32>(f32::MAX);
        assert_rounding_failure::<f64>(f64::MAX);
    }

    #[test]
    fn fri05_c03_root_block_legacy_absence_factory_has_no_migration_or_rounding_adapter() {
        let source = include_str!("scroll.rs");
        let production = source
            .split("#[cfg(test)]\nmod fri05_c02_contribution_range_tests")
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
            4,
            "the canonical source builder, measured leaf, retained-source rebuild, and rounding are the production callers"
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
