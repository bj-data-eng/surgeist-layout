use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PhysicalContributionIntervalOf<S: LayoutScalar> {
    pub(super) minimum: S,
    pub(super) maximum: S,
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
pub(super) struct PhysicalContributionBoundsOf<S: LayoutScalar> {
    pub(super) x: PhysicalContributionIntervalOf<S>,
    pub(super) y: PhysicalContributionIntervalOf<S>,
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
    pub(super) const fn x(self) -> PhysicalContributionIntervalOf<S> {
        self.x
    }

    #[must_use]
    pub(super) const fn y(self) -> PhysicalContributionIntervalOf<S> {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OptionalPhysicalContributionIntervalsOf<S: LayoutScalar> {
    pub(super) x: Option<PhysicalContributionIntervalOf<S>>,
    pub(super) y: Option<PhysicalContributionIntervalOf<S>>,
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
pub(super) struct FinalInFlowEndOf<S: LayoutScalar> {
    pub(super) side: PhysicalSide,
    pub(super) coordinate: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PhysicalFinalInFlowEndsOf<S: LayoutScalar> {
    pub(super) x: Option<FinalInFlowEndOf<S>>,
    pub(super) y: Option<FinalInFlowEndOf<S>>,
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
pub(super) enum ContainerRangeBasis {
    PaddingBox,
    Scrollport,
    ScrollContainerAxes(UsedOverflow),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollContributionAccumulatorOf<S: LayoutScalar> {
    pub(super) container_seed: PhysicalContributionBoundsOf<S>,
    pub(super) container_range_basis: ContainerRangeBasis,
    pub(super) propagatable_descendants: OptionalPhysicalContributionIntervalsOf<S>,
    pub(super) final_in_flow_ends: PhysicalFinalInFlowEndsOf<S>,
    pub(super) terminal_padding_overflow: OptionalPhysicalContributionIntervalsOf<S>,
    pub(super) active_alignment_subjects: OptionalPhysicalContributionIntervalsOf<S>,
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

    pub(super) fn exclude_reserved_gutter_from_scroll_container_axes(
        &mut self,
        overflow: UsedOverflow,
    ) {
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
    pub(super) fn complete_overflow(self) -> PhysicalContributionBoundsOf<S> {
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

pub(super) fn derive_origin_aware_scroll_range<S: LayoutScalar>(
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
