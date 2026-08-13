use super::*;

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

pub(super) fn round_canonical_contributions<S: LayoutScalar>(
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

type DefaultCanonicalScrollGeometryRounding =
    fn(
        ScrollGeometryOf<DefaultScalar>,
        Point<DefaultScalar>,
    )
        -> Result<ScrollGeometryOf<DefaultScalar>, CanonicalScrollGeometryErrorOf<DefaultScalar>>;
const _: DefaultCanonicalScrollGeometryRounding =
    rebuild_rounded_canonical_scroll_geometry::<DefaultScalar>;

#[cfg(test)]
mod fri05_c02_factory_rounding_tests {
    use super::construction::fri05_c02_factory_tests::{
        FLOW_MAPPINGS, assert_canonical_coherence, factory_source, percent, px, rect, scalar,
    };
    use super::*;
    use crate::{Direction, ScrollSnapType, WritingMode};

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
}
