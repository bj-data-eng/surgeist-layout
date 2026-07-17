use crate::scroll::{
    CanonicalScrollGeometrySourceOf, ClipMarginSourceOf, FlowRelativeScrollOffsetOf,
    FlowRelativeScrollRangeOf, OptimalRegionInsetsOf, OverflowClipOf, PhysicalClipAxisOf,
    PhysicalScrollOffsetOf, PhysicalScrollRangeOf, ScrollContributionAccumulatorOf,
    ScrollCoordinateErrorOf, ScrollOriginAxes, ScrollOriginProgression, ScrollRectErrorOf,
    ScrollRectOf, ScrollTargetGeometryOf, ScrollbarReservationOf, SettledAutoScrollbarState,
    UsedOverflow, UsedOverflowGutter, canonical_scroll_geometry_from_source,
    rebuild_rounded_canonical_scroll_geometry,
};
use crate::{
    ComputedOverflow, Direction, Edges, FlowAxes, LayoutScalar, LogicalAxis, Overflow,
    PhysicalAxis, Point, ScrollGeometryOf, ScrollMarginOf, ScrollRect, ScrollSnapAlign,
    ScrollSnapStop, ScrollSnapType, ScrollbarGutter, ScrollbarWidthOf, Size, WritingMode,
};

fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

struct CanonicalTestGeometryFactsOf<S: LayoutScalar> {
    flow_axes: FlowAxes,
    overflow: ComputedOverflow,
    item_is_replaced: bool,
    border_box_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    scrollbar_width_value: S,
    scrollable_overflow: ScrollRectOf<S>,
}

fn canonical_test_geometry<S: LayoutScalar>(
    facts: CanonicalTestGeometryFactsOf<S>,
) -> ScrollGeometryOf<S> {
    let CanonicalTestGeometryFactsOf {
        flow_axes,
        overflow,
        item_is_replaced,
        border_box_size,
        padding,
        border,
        scrollbar_width_value,
        scrollable_overflow,
    } = facts;
    let mut contributions = ScrollContributionAccumulatorOf::new(scrollable_overflow);
    contributions.include_direct_line(scrollable_overflow);
    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes,
        computed_overflow: overflow,
        item_is_replaced,
        border_box_size,
        border,
        padding,
        scrollbar_gutter: ScrollbarGutter::Auto,
        scrollbar_width: ScrollbarWidthOf::try_new(scrollbar_width_value).unwrap(),
        settled_auto_scrollbars: SettledAutoScrollbarState::INITIAL,
        clip_margin: ClipMarginSourceOf::default(),
        scroll_padding: OptimalRegionInsetsOf::default(),
        contributions,
        origin_axes: ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
        scroll_snap_type: ScrollSnapType::default(),
        target_border_box: ScrollRectOf::try_new(Point::ZERO, border_box_size).unwrap(),
        target_scroll_margin: ScrollMarginOf::default(),
        target_flow_axes: flow_axes,
        target_snap_align: ScrollSnapAlign::default(),
        target_snap_stop: ScrollSnapStop::default(),
    })
    .expect("canonical test source facts produce geometry")
}

#[test]
fn fri05_c01_node_input_private_used_overflow_phase_is_exact_through_scroll_consumers() {
    for (computed, clips, exposes_range, gutter, replaced) in [
        (
            Overflow::Visible,
            false,
            false,
            UsedOverflowGutter::None,
            Overflow::Visible,
        ),
        (
            Overflow::Clip,
            true,
            false,
            UsedOverflowGutter::None,
            Overflow::Clip,
        ),
        (
            Overflow::Hidden,
            true,
            true,
            UsedOverflowGutter::StableOnly,
            Overflow::Clip,
        ),
        (
            Overflow::Scroll,
            true,
            true,
            UsedOverflowGutter::Forced,
            Overflow::Scroll,
        ),
        (
            Overflow::Auto,
            true,
            true,
            UsedOverflowGutter::Conditional,
            Overflow::Auto,
        ),
    ] {
        let computed_pair = ComputedOverflow::try_new(computed, computed)
            .expect("same-group computed pair is canonical");
        let ordinary = UsedOverflow::from_computed(computed_pair, false);
        let replaced_pair = UsedOverflow::from_computed(computed_pair, true);

        for axis in [ordinary.x(), ordinary.y()] {
            assert_eq!(axis.value(), computed);
            assert_eq!(axis.clips_contents(), clips);
            assert_eq!(axis.exposes_scroll_range(), exposes_range);
            assert_eq!(axis.gutter_classification(), gutter);
        }
        for axis in [replaced_pair.x(), replaced_pair.y()] {
            assert_eq!(axis.value(), replaced);
        }

        let rect = ScrollRect::try_new(Point::ZERO, Size::splat(10.0)).unwrap();
        let ordinary_geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
            flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            overflow: computed_pair,
            item_is_replaced: false,
            border_box_size: Size::splat(10.0),
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_width_value: 0.0,
            scrollable_overflow: rect,
        });
        assert_eq!(ordinary_geometry.used_overflow_x(), computed);
        assert_eq!(ordinary_geometry.used_overflow_y(), computed);

        let replaced_geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
            flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            overflow: computed_pair,
            item_is_replaced: true,
            border_box_size: Size::splat(10.0),
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_width_value: 0.0,
            scrollable_overflow: rect,
        });
        assert_eq!(replaced_geometry.used_overflow_x(), replaced);
        assert_eq!(replaced_geometry.used_overflow_y(), replaced);
    }
}

fn assert_physical_range_maximum(range: PhysicalScrollRangeOf, maximum: Size) {
    assert_eq!(range.x().minimum(), 0.0);
    assert_eq!(range.x().maximum(), maximum.width);
    assert_eq!(range.y().minimum(), 0.0);
    assert_eq!(range.y().maximum(), maximum.height);
}

type ScrollProjectionExpectation = (WritingMode, Direction, (f64, f64), (f64, f64, f64, f64));

const ALL_SCROLL_PROJECTION_WRITING_MODES: [WritingMode; 5] = [
    WritingMode::HorizontalTb,
    WritingMode::VerticalRl,
    WritingMode::VerticalLr,
    WritingMode::SidewaysRl,
    WritingMode::SidewaysLr,
];

const ALL_SCROLL_PROJECTION_DIRECTIONS: [Direction; 2] = [Direction::Ltr, Direction::Rtl];

const SCROLL_PROJECTION_EXPECTATIONS: [ScrollProjectionExpectation; 10] = [
    (
        WritingMode::HorizontalTb,
        Direction::Ltr,
        (3.0, 7.0),
        (-3.0, 11.0, -5.0, 13.0),
    ),
    (
        WritingMode::HorizontalTb,
        Direction::Rtl,
        (-3.0, 7.0),
        (-11.0, 3.0, -5.0, 13.0),
    ),
    (
        WritingMode::VerticalRl,
        Direction::Ltr,
        (-7.0, 3.0),
        (-13.0, 5.0, -3.0, 11.0),
    ),
    (
        WritingMode::VerticalRl,
        Direction::Rtl,
        (-7.0, -3.0),
        (-13.0, 5.0, -11.0, 3.0),
    ),
    (
        WritingMode::VerticalLr,
        Direction::Ltr,
        (7.0, 3.0),
        (-5.0, 13.0, -3.0, 11.0),
    ),
    (
        WritingMode::VerticalLr,
        Direction::Rtl,
        (7.0, -3.0),
        (-5.0, 13.0, -11.0, 3.0),
    ),
    (
        WritingMode::SidewaysRl,
        Direction::Ltr,
        (-7.0, 3.0),
        (-13.0, 5.0, -3.0, 11.0),
    ),
    (
        WritingMode::SidewaysRl,
        Direction::Rtl,
        (-7.0, -3.0),
        (-13.0, 5.0, -11.0, 3.0),
    ),
    (
        WritingMode::SidewaysLr,
        Direction::Ltr,
        (7.0, -3.0),
        (-5.0, 13.0, -11.0, 3.0),
    ),
    (
        WritingMode::SidewaysLr,
        Direction::Rtl,
        (7.0, 3.0),
        (-5.0, 13.0, -3.0, 11.0),
    ),
];

fn scroll_projection_expectations_are_complete_and_unique(
    expectations: &[ScrollProjectionExpectation],
) -> bool {
    expectations.len()
        == ALL_SCROLL_PROJECTION_WRITING_MODES.len() * ALL_SCROLL_PROJECTION_DIRECTIONS.len()
        && ALL_SCROLL_PROJECTION_WRITING_MODES
            .into_iter()
            .all(|writing_mode| {
                ALL_SCROLL_PROJECTION_DIRECTIONS
                    .into_iter()
                    .all(|direction| {
                        expectations
                            .iter()
                            .filter(|(candidate_mode, candidate_direction, _, _)| {
                                *candidate_mode == writing_mode && *candidate_direction == direction
                            })
                            .count()
                            == 1
                    })
            })
}

#[test]
fn scroll_projection_expectations_reject_duplicates_and_cover_all_flow_mappings() {
    assert!(scroll_projection_expectations_are_complete_and_unique(
        &SCROLL_PROJECTION_EXPECTATIONS
    ));

    let mut duplicate_pair = SCROLL_PROJECTION_EXPECTATIONS;
    duplicate_pair[duplicate_pair.len() - 1] = duplicate_pair[0];
    assert!(!scroll_projection_expectations_are_complete_and_unique(
        &duplicate_pair
    ));
}

fn assert_scroll_projection_for_all_mappings<S: crate::LayoutScalar>() {
    let flow_offset =
        FlowRelativeScrollOffsetOf::try_new(S::from_f64(3.0), S::from_f64(7.0)).unwrap();
    let flow_range = FlowRelativeScrollRangeOf::try_new(
        S::from_f64(-3.0),
        S::from_f64(11.0),
        S::from_f64(-5.0),
        S::from_f64(13.0),
    )
    .unwrap();

    for (writing_mode, direction, offset, range) in SCROLL_PROJECTION_EXPECTATIONS {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical_offset = flow_axes.physical_scroll_offset(flow_offset);
        let physical_range = flow_axes.physical_scroll_range(flow_range);

        assert_eq!(
            physical_offset,
            PhysicalScrollOffsetOf::try_new(S::from_f64(offset.0), S::from_f64(offset.1)).unwrap()
        );
        assert_eq!(
            physical_range,
            PhysicalScrollRangeOf::try_new(
                S::from_f64(range.0),
                S::from_f64(range.1),
                S::from_f64(range.2),
                S::from_f64(range.3),
            )
            .unwrap()
        );
        assert_eq!(
            flow_axes.flow_relative_scroll_offset(physical_offset),
            flow_offset
        );
        assert_eq!(
            flow_axes.flow_relative_scroll_range(physical_range),
            flow_range
        );
        assert_eq!(
            flow_axes
                .physical_scroll_offset(flow_axes.flow_relative_scroll_offset(physical_offset)),
            physical_offset
        );
        assert_eq!(
            flow_axes.physical_scroll_range(flow_axes.flow_relative_scroll_range(physical_range)),
            physical_range
        );
    }
}

#[test]
fn scroll_projection_maps_all_ten_flow_mappings_in_both_scalar_lanes() {
    assert_scroll_projection_for_all_mappings::<f32>();
    assert_scroll_projection_for_all_mappings::<f64>();
}

#[test]
fn scroll_projection_canonicalizes_signed_zero_for_all_mappings_in_both_scalar_lanes() {
    for (writing_mode, direction, _, _) in SCROLL_PROJECTION_EXPECTATIONS {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical = flow_axes.physical_scroll_offset(
            FlowRelativeScrollOffsetOf::<f32>::try_new(-0.0, -0.0).unwrap(),
        );
        let flow = flow_axes.flow_relative_scroll_offset(
            PhysicalScrollOffsetOf::<f32>::try_new(-0.0, -0.0).unwrap(),
        );

        assert_eq!(physical.x().to_bits(), 0.0_f32.to_bits());
        assert_eq!(physical.y().to_bits(), 0.0_f32.to_bits());
        assert_eq!(flow.inline().to_bits(), 0.0_f32.to_bits());
        assert_eq!(flow.block().to_bits(), 0.0_f32.to_bits());

        let physical = flow_axes.physical_scroll_offset(
            FlowRelativeScrollOffsetOf::<f64>::try_new(-0.0, -0.0).unwrap(),
        );
        let flow = flow_axes.flow_relative_scroll_offset(
            PhysicalScrollOffsetOf::<f64>::try_new(-0.0, -0.0).unwrap(),
        );

        assert_eq!(physical.x().to_bits(), 0.0_f64.to_bits());
        assert_eq!(physical.y().to_bits(), 0.0_f64.to_bits());
        assert_eq!(flow.inline().to_bits(), 0.0_f64.to_bits());
        assert_eq!(flow.block().to_bits(), 0.0_f64.to_bits());
    }
}

macro_rules! assert_reversed_range_projection_canonicalizes_signed_zero {
    ($scalar:ty) => {{
        let mut reverses_inline = false;
        let mut reverses_block = false;
        let mut reverses_horizontal = false;
        let mut reverses_vertical = false;

        for (writing_mode, direction, _, _) in SCROLL_PROJECTION_EXPECTATIONS {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let inline_reverses = flow_axes
                .logical_axis_progression(LogicalAxis::Inline)
                .is_decreasing();
            let block_reverses = flow_axes
                .logical_axis_progression(LogicalAxis::Block)
                .is_decreasing();

            if !inline_reverses && !block_reverses {
                continue;
            }

            reverses_inline |= inline_reverses;
            reverses_block |= block_reverses;
            reverses_horizontal |= match flow_axes.inline_axis() {
                PhysicalAxis::Horizontal => inline_reverses,
                PhysicalAxis::Vertical => block_reverses,
            };
            reverses_vertical |= match flow_axes.inline_axis() {
                PhysicalAxis::Horizontal => block_reverses,
                PhysicalAxis::Vertical => inline_reverses,
            };

            let flow_range = FlowRelativeScrollRangeOf::try_new(
                0.0 as $scalar,
                0.0 as $scalar,
                0.0 as $scalar,
                0.0 as $scalar,
            )
            .unwrap();
            let physical_range = flow_axes.physical_scroll_range(flow_range);
            assert_eq!(
                physical_range.x().minimum().to_bits(),
                (0.0 as $scalar).to_bits()
            );
            assert_eq!(
                physical_range.x().maximum().to_bits(),
                (0.0 as $scalar).to_bits()
            );
            assert_eq!(
                physical_range.y().minimum().to_bits(),
                (0.0 as $scalar).to_bits()
            );
            assert_eq!(
                physical_range.y().maximum().to_bits(),
                (0.0 as $scalar).to_bits()
            );

            let physical_range = PhysicalScrollRangeOf::try_new(
                0.0 as $scalar,
                0.0 as $scalar,
                0.0 as $scalar,
                0.0 as $scalar,
            )
            .unwrap();
            let flow_range = flow_axes.flow_relative_scroll_range(physical_range);
            assert_eq!(
                flow_range.inline().minimum().to_bits(),
                (0.0 as $scalar).to_bits()
            );
            assert_eq!(
                flow_range.inline().maximum().to_bits(),
                (0.0 as $scalar).to_bits()
            );
            assert_eq!(
                flow_range.block().minimum().to_bits(),
                (0.0 as $scalar).to_bits()
            );
            assert_eq!(
                flow_range.block().maximum().to_bits(),
                (0.0 as $scalar).to_bits()
            );
        }

        assert!(reverses_inline);
        assert!(reverses_block);
        assert!(reverses_horizontal);
        assert!(reverses_vertical);
    }};
}

#[test]
fn scroll_projection_canonicalizes_range_signed_zero_after_reversal_in_both_scalar_lanes() {
    assert_reversed_range_projection_canonicalizes_signed_zero!(f32);
    assert_reversed_range_projection_canonicalizes_signed_zero!(f64);
}

fn assert_scroll_conversion_clamp_laws_for_all_mappings<S: crate::LayoutScalar>() {
    let flow_range = FlowRelativeScrollRangeOf::try_new(
        S::from_f64(-10.0),
        S::from_f64(20.0),
        S::from_f64(-30.0),
        S::from_f64(40.0),
    )
    .unwrap();

    for (writing_mode, direction, _, _) in SCROLL_PROJECTION_EXPECTATIONS {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical_range = flow_axes.physical_scroll_range(flow_range);

        for (inline, block) in [
            (-11.0, -31.0),
            (-10.0, -30.0),
            (2.0, 3.0),
            (20.0, 40.0),
            (21.0, 41.0),
        ] {
            let flow_offset =
                FlowRelativeScrollOffsetOf::try_new(S::from_f64(inline), S::from_f64(block))
                    .unwrap();
            let physical_offset = flow_axes.physical_scroll_offset(flow_offset);
            let flow_clamped = flow_range.clamp(flow_offset);
            let physical_clamped = physical_range.clamp(physical_offset);

            assert_eq!(
                physical_clamped,
                flow_axes.physical_scroll_offset(flow_clamped)
            );
            assert_eq!(
                flow_axes.flow_relative_scroll_offset(physical_clamped),
                flow_clamped
            );
            assert!(physical_clamped.x() >= physical_range.x().minimum());
            assert!(physical_clamped.x() <= physical_range.x().maximum());
            assert!(physical_clamped.y() >= physical_range.y().minimum());
            assert!(physical_clamped.y() <= physical_range.y().maximum());
            assert_eq!(physical_range.clamp(physical_clamped), physical_clamped);
        }
    }
}

#[test]
fn scroll_conversion_clamp_commutes_is_contained_and_idempotent_in_both_scalar_lanes() {
    assert_scroll_conversion_clamp_laws_for_all_mappings::<f32>();
    assert_scroll_conversion_clamp_laws_for_all_mappings::<f64>();
}

#[test]
fn scroll_coordinate_constructors_report_exact_semantic_errors() {
    assert_eq!(
        PhysicalScrollOffsetOf::try_new(f32::INFINITY, 1.0),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
            axis: PhysicalAxis::Horizontal,
            value: f32::INFINITY,
        })
    );
    assert_eq!(
        PhysicalScrollOffsetOf::try_new(1.0, f32::NEG_INFINITY),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
            axis: PhysicalAxis::Vertical,
            value: f32::NEG_INFINITY,
        })
    );
    assert_eq!(
        FlowRelativeScrollOffsetOf::try_new(f32::INFINITY, 1.0),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeOffset {
            axis: LogicalAxis::Inline,
            value: f32::INFINITY,
        })
    );
    assert_eq!(
        FlowRelativeScrollOffsetOf::try_new(1.0, f32::NEG_INFINITY),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeOffset {
            axis: LogicalAxis::Block,
            value: f32::NEG_INFINITY,
        })
    );

    assert_eq!(
        PhysicalScrollRangeOf::try_new(f32::INFINITY, 1.0, 0.0, 1.0),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMinimum {
            axis: PhysicalAxis::Horizontal,
            value: f32::INFINITY,
        })
    );
    assert_eq!(
        PhysicalScrollRangeOf::try_new(0.0, f32::INFINITY, 0.0, 1.0),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMaximum {
            axis: PhysicalAxis::Horizontal,
            value: f32::INFINITY,
        })
    );
    assert_eq!(
        PhysicalScrollRangeOf::try_new(0.0, 1.0, f32::NEG_INFINITY, 1.0),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMinimum {
            axis: PhysicalAxis::Vertical,
            value: f32::NEG_INFINITY,
        })
    );
    assert_eq!(
        PhysicalScrollRangeOf::try_new(0.0, 1.0, 0.0, f32::INFINITY),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMaximum {
            axis: PhysicalAxis::Vertical,
            value: f32::INFINITY,
        })
    );
    assert_eq!(
        PhysicalScrollRangeOf::try_new(3.0, 2.0, 0.0, 1.0),
        Err(ScrollCoordinateErrorOf::InvertedPhysicalRange {
            axis: PhysicalAxis::Horizontal,
            minimum: 3.0,
            maximum: 2.0,
        })
    );
    assert_eq!(
        PhysicalScrollRangeOf::try_new(0.0, 1.0, 3.0, 2.0),
        Err(ScrollCoordinateErrorOf::InvertedPhysicalRange {
            axis: PhysicalAxis::Vertical,
            minimum: 3.0,
            maximum: 2.0,
        })
    );

    assert_eq!(
        FlowRelativeScrollRangeOf::try_new(f32::INFINITY, 1.0, 0.0, 1.0),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeRangeMinimum {
            axis: LogicalAxis::Inline,
            value: f32::INFINITY,
        })
    );
    assert_eq!(
        FlowRelativeScrollRangeOf::try_new(0.0, f32::INFINITY, 0.0, 1.0),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeRangeMaximum {
            axis: LogicalAxis::Inline,
            value: f32::INFINITY,
        })
    );
    assert_eq!(
        FlowRelativeScrollRangeOf::try_new(0.0, 1.0, f32::NEG_INFINITY, 1.0),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeRangeMinimum {
            axis: LogicalAxis::Block,
            value: f32::NEG_INFINITY,
        })
    );
    assert_eq!(
        FlowRelativeScrollRangeOf::try_new(0.0, 1.0, 0.0, f32::INFINITY),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeRangeMaximum {
            axis: LogicalAxis::Block,
            value: f32::INFINITY,
        })
    );
    assert_eq!(
        FlowRelativeScrollRangeOf::try_new(3.0, 2.0, 0.0, 1.0),
        Err(ScrollCoordinateErrorOf::InvertedFlowRelativeRange {
            axis: LogicalAxis::Inline,
            minimum: 3.0,
            maximum: 2.0,
        })
    );
    assert_eq!(
        FlowRelativeScrollRangeOf::try_new(0.0, 1.0, 3.0, 2.0),
        Err(ScrollCoordinateErrorOf::InvertedFlowRelativeRange {
            axis: LogicalAxis::Block,
            minimum: 3.0,
            maximum: 2.0,
        })
    );
}

#[test]
fn scroll_coordinate_constructors_reject_f32_nan_with_typed_errors() {
    let nan = f32::from_bits(0x7fc0_0042);

    assert!(matches!(
        PhysicalScrollOffsetOf::<f32>::try_new(nan, 1.0),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
            axis: PhysicalAxis::Horizontal,
            value,
        }) if value.to_bits() == nan.to_bits()
    ));
    assert!(matches!(
        FlowRelativeScrollOffsetOf::<f32>::try_new(1.0, nan),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeOffset {
            axis: LogicalAxis::Block,
            value,
        }) if value.to_bits() == nan.to_bits()
    ));
    assert!(matches!(
        PhysicalScrollRangeOf::<f32>::try_new(nan, 1.0, 0.0, 1.0),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMinimum {
            axis: PhysicalAxis::Horizontal,
            value,
        }) if value.to_bits() == nan.to_bits()
    ));
    assert!(matches!(
        FlowRelativeScrollRangeOf::<f32>::try_new(0.0, 1.0, 0.0, nan),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeRangeMaximum {
            axis: LogicalAxis::Block,
            value,
        }) if value.to_bits() == nan.to_bits()
    ));
}

#[test]
fn scroll_coordinate_constructors_reject_f64_nan_with_typed_errors() {
    let nan = f64::from_bits(0x7ff8_0000_0000_0042);

    assert!(matches!(
        PhysicalScrollOffsetOf::<f64>::try_new(1.0, nan),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
            axis: PhysicalAxis::Vertical,
            value,
        }) if value.to_bits() == nan.to_bits()
    ));
    assert!(matches!(
        FlowRelativeScrollOffsetOf::<f64>::try_new(nan, 1.0),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeOffset {
            axis: LogicalAxis::Inline,
            value,
        }) if value.to_bits() == nan.to_bits()
    ));
    assert!(matches!(
        PhysicalScrollRangeOf::<f64>::try_new(0.0, 1.0, 0.0, nan),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMaximum {
            axis: PhysicalAxis::Vertical,
            value,
        }) if value.to_bits() == nan.to_bits()
    ));
    assert!(matches!(
        FlowRelativeScrollRangeOf::<f64>::try_new(nan, 1.0, 0.0, 1.0),
        Err(ScrollCoordinateErrorOf::NonFiniteFlowRelativeRangeMinimum {
            axis: LogicalAxis::Inline,
            value,
        }) if value.to_bits() == nan.to_bits()
    ));
}

#[test]
fn scroll_coordinate_preserves_signed_values_and_canonicalizes_zero_in_both_scalar_lanes() {
    let physical = PhysicalScrollOffsetOf::try_new(-3.5_f32, -0.0).unwrap();
    assert_eq!(physical.x(), -3.5);
    assert_eq!(physical.y().to_bits(), 0.0_f32.to_bits());

    let flow = FlowRelativeScrollOffsetOf::<f64>::try_new(-0.0, -9.25).unwrap();
    assert_eq!(flow.inline().to_bits(), 0.0_f64.to_bits());
    assert_eq!(flow.block(), -9.25);

    let physical_range =
        PhysicalScrollRangeOf::<f64>::try_new(-0.0, 16_777_217.0, -8.0, -0.0).unwrap();
    assert_eq!(physical_range.x().minimum().to_bits(), 0.0_f64.to_bits());
    assert_eq!(physical_range.x().maximum(), 16_777_217.0);
    assert_eq!(physical_range.y().minimum(), -8.0);
    assert_eq!(physical_range.y().maximum().to_bits(), 0.0_f64.to_bits());

    let flow_range = FlowRelativeScrollRangeOf::try_new(-4.0_f32, -0.0, -0.0, 7.0).unwrap();
    assert_eq!(flow_range.inline().minimum(), -4.0);
    assert_eq!(flow_range.inline().maximum().to_bits(), 0.0_f32.to_bits());
    assert_eq!(flow_range.block().minimum().to_bits(), 0.0_f32.to_bits());
    assert_eq!(flow_range.block().maximum(), 7.0);
}

#[test]
fn scroll_clamp_is_component_wise_contained_and_idempotent_in_both_spaces() {
    let physical_range = PhysicalScrollRangeOf::try_new(-10.0_f32, 20.0, -30.0, 40.0).unwrap();
    for (input, expected) in [
        ((-11.0, -31.0), (-10.0, -30.0)),
        ((-10.0, -30.0), (-10.0, -30.0)),
        ((2.0, 3.0), (2.0, 3.0)),
        ((20.0, 40.0), (20.0, 40.0)),
        ((21.0, 41.0), (20.0, 40.0)),
    ] {
        let clamped =
            physical_range.clamp(PhysicalScrollOffsetOf::try_new(input.0, input.1).unwrap());
        assert_eq!(
            clamped,
            PhysicalScrollOffsetOf::try_new(expected.0, expected.1).unwrap()
        );
        assert!(clamped.x() >= physical_range.x().minimum());
        assert!(clamped.x() <= physical_range.x().maximum());
        assert!(clamped.y() >= physical_range.y().minimum());
        assert!(clamped.y() <= physical_range.y().maximum());
        assert_eq!(physical_range.clamp(clamped), clamped);
    }

    let flow_range = FlowRelativeScrollRangeOf::<f64>::try_new(-10.0, 20.0, -30.0, 40.0).unwrap();
    for (input, expected) in [
        ((-11.0, -31.0), (-10.0, -30.0)),
        ((-10.0, -30.0), (-10.0, -30.0)),
        ((2.0, 3.0), (2.0, 3.0)),
        ((20.0, 40.0), (20.0, 40.0)),
        ((21.0, 41.0), (20.0, 40.0)),
    ] {
        let clamped =
            flow_range.clamp(FlowRelativeScrollOffsetOf::try_new(input.0, input.1).unwrap());
        assert_eq!(
            clamped,
            FlowRelativeScrollOffsetOf::try_new(expected.0, expected.1).unwrap()
        );
        assert!(clamped.inline() >= flow_range.inline().minimum());
        assert!(clamped.inline() <= flow_range.inline().maximum());
        assert!(clamped.block() >= flow_range.block().minimum());
        assert!(clamped.block() <= flow_range.block().maximum());
        assert_eq!(flow_range.clamp(clamped), clamped);
    }
}

#[test]
fn physical_scroll_range_clamps_signed_offsets() {
    let range = PhysicalScrollRangeOf::try_new(-120.0, 40.0, -30.0, 50.0).unwrap();

    assert_eq!(
        range.clamp(PhysicalScrollOffsetOf::try_new(-200.0, 99.0).unwrap()),
        PhysicalScrollOffsetOf::try_new(-120.0, 50.0).unwrap()
    );
}

#[test]
fn fri05_c02_rect_reports_every_axis_error_and_legacy_mapping_in_both_scalar_lanes() {
    fn assert_scalar<S: LayoutScalar>(maximum: S) {
        let zero = S::ZERO;
        let one = S::ONE;
        let invalid = [
            (
                Point::new(S::INFINITY, zero),
                Size::new(one, one),
                ScrollRectErrorOf::NonFiniteOrigin {
                    axis: PhysicalAxis::Horizontal,
                    value: S::INFINITY,
                },
            ),
            (
                Point::new(zero, -S::INFINITY),
                Size::new(one, one),
                ScrollRectErrorOf::NonFiniteOrigin {
                    axis: PhysicalAxis::Vertical,
                    value: -S::INFINITY,
                },
            ),
            (
                Point::new(zero, zero),
                Size::new(S::INFINITY, one),
                ScrollRectErrorOf::NonFiniteSize {
                    axis: PhysicalAxis::Horizontal,
                    value: S::INFINITY,
                },
            ),
            (
                Point::new(zero, zero),
                Size::new(one, -S::INFINITY),
                ScrollRectErrorOf::NonFiniteSize {
                    axis: PhysicalAxis::Vertical,
                    value: -S::INFINITY,
                },
            ),
            (
                Point::new(zero, zero),
                Size::new(-one, one),
                ScrollRectErrorOf::NegativeSize {
                    axis: PhysicalAxis::Horizontal,
                    value: -one,
                },
            ),
            (
                Point::new(zero, zero),
                Size::new(one, -one),
                ScrollRectErrorOf::NegativeSize {
                    axis: PhysicalAxis::Vertical,
                    value: -one,
                },
            ),
            (
                Point::new(maximum, zero),
                Size::new(maximum, one),
                ScrollRectErrorOf::NonFiniteEnd {
                    axis: PhysicalAxis::Horizontal,
                    value: S::INFINITY,
                    origin: maximum,
                    size: maximum,
                },
            ),
            (
                Point::new(zero, maximum),
                Size::new(one, maximum),
                ScrollRectErrorOf::NonFiniteEnd {
                    axis: PhysicalAxis::Vertical,
                    value: S::INFINITY,
                    origin: maximum,
                    size: maximum,
                },
            ),
        ];

        for (origin, size, expected) in invalid {
            assert_eq!(ScrollRectOf::try_new(origin, size), Err(expected));
        }
    }

    assert_scalar::<f32>(f32::MAX);
    assert_scalar::<f64>(f64::MAX);
}

#[test]
fn fri05_c02_rect_validation_precedence_is_atomic_and_axis_ordered() {
    let cases = [
        (
            Point::new(f32::INFINITY, f32::NEG_INFINITY),
            Size::new(f32::INFINITY, -1.0),
            ScrollRectErrorOf::NonFiniteOrigin {
                axis: PhysicalAxis::Horizontal,
                value: f32::INFINITY,
            },
        ),
        (
            Point::new(0.0, f32::INFINITY),
            Size::new(f32::INFINITY, -1.0),
            ScrollRectErrorOf::NonFiniteOrigin {
                axis: PhysicalAxis::Vertical,
                value: f32::INFINITY,
            },
        ),
        (
            Point::ZERO,
            Size::new(f32::INFINITY, f32::NEG_INFINITY),
            ScrollRectErrorOf::NonFiniteSize {
                axis: PhysicalAxis::Horizontal,
                value: f32::INFINITY,
            },
        ),
        (
            Point::ZERO,
            Size::new(-1.0, f32::INFINITY),
            ScrollRectErrorOf::NonFiniteSize {
                axis: PhysicalAxis::Vertical,
                value: f32::INFINITY,
            },
        ),
        (
            Point::ZERO,
            Size::new(-1.0, -2.0),
            ScrollRectErrorOf::NegativeSize {
                axis: PhysicalAxis::Horizontal,
                value: -1.0,
            },
        ),
        (
            Point::new(f32::MAX, f32::MAX),
            Size::new(f32::MAX, f32::MAX),
            ScrollRectErrorOf::NonFiniteEnd {
                axis: PhysicalAxis::Horizontal,
                value: f32::INFINITY,
                origin: f32::MAX,
                size: f32::MAX,
            },
        ),
    ];

    for (origin, size, expected) in cases {
        assert_eq!(ScrollRect::try_new(origin, size), Err(expected));
    }
}

#[test]
fn fri05_c02_rect_canonicalizes_every_signed_zero_in_both_scalar_lanes() {
    let f32_rect =
        ScrollRectOf::<f32>::try_new(Point::new(-0.0, 0.0), Size::new(-0.0, 0.0)).unwrap();
    assert_eq!(f32_rect.origin().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(f32_rect.origin().y.to_bits(), 0.0_f32.to_bits());
    assert_eq!(f32_rect.size().width.to_bits(), 0.0_f32.to_bits());
    assert_eq!(f32_rect.size().height.to_bits(), 0.0_f32.to_bits());

    let f64_rect =
        ScrollRectOf::<f64>::try_new(Point::new(0.0, -0.0), Size::new(0.0, -0.0)).unwrap();
    assert_eq!(f64_rect.origin().x.to_bits(), 0.0_f64.to_bits());
    assert_eq!(f64_rect.origin().y.to_bits(), 0.0_f64.to_bits());
    assert_eq!(f64_rect.size().width.to_bits(), 0.0_f64.to_bits());
    assert_eq!(f64_rect.size().height.to_bits(), 0.0_f64.to_bits());
}

#[test]
fn fri05_c02_rect_allows_zero_axes_and_zero_area_without_losing_precision() {
    fn assert_zero_rects<S: LayoutScalar>() {
        let seven = S::from_f64(7.0);
        for size in [
            Size::new(S::ZERO, seven),
            Size::new(seven, S::ZERO),
            Size::ZERO,
        ] {
            assert_eq!(
                ScrollRectOf::<S>::try_new(Point::ZERO, size)
                    .expect("zero axes and zero area are valid")
                    .size(),
                size
            );
        }
    }

    assert_zero_rects::<f32>();
    assert_zero_rects::<f64>();

    let f32_rect =
        ScrollRectOf::<f32>::try_new(Point::new(-1.25, 2.5), Size::new(3.75, 4.125)).unwrap();
    assert_eq!(f32_rect.origin(), Point::new(-1.25, 2.5));
    assert_eq!(f32_rect.size(), Size::new(3.75, 4.125));

    let f64_rect = ScrollRectOf::<f64>::try_new(
        Point::new(16_777_217.0, -0.125),
        Size::new(0.5, 8_388_609.0),
    )
    .unwrap();
    assert_eq!(f64_rect.origin(), Point::new(16_777_217.0, -0.125));
    assert_eq!(f64_rect.size(), Size::new(0.5, 8_388_609.0));
}

#[test]
fn fri05_c02_rect_error_traits_display_and_source_contract_are_exact() {
    fn assert_error<T: Clone + Copy + core::fmt::Debug + PartialEq + std::error::Error>() {}

    assert_error::<ScrollRectErrorOf<f32>>();
    assert_error::<ScrollRectErrorOf<f64>>();

    let errors = [
        (
            ScrollRectErrorOf::NonFiniteOrigin {
                axis: PhysicalAxis::Horizontal,
                value: f32::INFINITY,
            },
            "scroll rectangle horizontal origin must be finite",
        ),
        (
            ScrollRectErrorOf::NonFiniteSize {
                axis: PhysicalAxis::Vertical,
                value: f32::INFINITY,
            },
            "scroll rectangle vertical size must be finite",
        ),
        (
            ScrollRectErrorOf::NegativeSize {
                axis: PhysicalAxis::Horizontal,
                value: -1.0,
            },
            "scroll rectangle horizontal size must be non-negative",
        ),
        (
            ScrollRectErrorOf::NonFiniteEnd {
                axis: PhysicalAxis::Vertical,
                value: f32::INFINITY,
                origin: f32::MAX,
                size: f32::MAX,
            },
            "scroll rectangle vertical end must be finite",
        ),
    ];

    for (error, expected) in errors {
        assert_eq!(error.to_string(), expected);
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn fri05_c02_carrier_copy_clone_debug_partial_eq_contract_is_scalar_generic() {
    fn assert_traits<T: Clone + Copy + core::fmt::Debug + PartialEq>() {}

    assert_traits::<PhysicalClipAxisOf<f32>>();
    assert_traits::<PhysicalClipAxisOf<f64>>();
    assert_traits::<OverflowClipOf<f32>>();
    assert_traits::<OverflowClipOf<f64>>();
    assert_traits::<ScrollTargetGeometryOf<f32>>();
    assert_traits::<ScrollTargetGeometryOf<f64>>();
}

#[test]
fn scroll_geometry_supports_f64() {
    let range = PhysicalScrollRangeOf::<f64>::try_new(0.0, 1_000_000_000_000.0, 0.0, 0.5).unwrap();

    assert_eq!(
        range.clamp(PhysicalScrollOffsetOf::<f64>::try_new(2_000_000_000_000.0, 1.0).unwrap()),
        PhysicalScrollOffsetOf::<f64>::try_new(1_000_000_000_000.0, 0.5).unwrap()
    );
}

#[test]
fn scrollbar_size_uses_scroll_overflow_on_opposite_physical_axis() {
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            computed_overflow(Overflow::Auto, Overflow::Scroll),
            false,
            15.0,
        ),
        Size::new(15.0, 0.0)
    );
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            computed_overflow(Overflow::Scroll, Overflow::Auto),
            false,
            15.0,
        ),
        Size::new(0.0, 15.0)
    );
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            computed_overflow(Overflow::Scroll, Overflow::Scroll),
            false,
            15.0,
        ),
        Size::new(15.0, 15.0)
    );
}

#[test]
fn scrollbar_reservation_places_inline_gutter_by_direction() {
    let ltr = ScrollbarReservationOf::from_overflow(
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        false,
        12.0,
        Direction::Ltr,
    );
    let rtl = ScrollbarReservationOf::from_overflow(
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        false,
        12.0,
        Direction::Rtl,
    );

    assert_eq!(ltr.size(), Size::new(12.0, 0.0));
    assert_eq!(ltr.inset(), Edges::new(0.0, 12.0, 0.0, 0.0));
    assert_eq!(rtl.size(), Size::new(12.0, 0.0));
    assert_eq!(rtl.inset(), Edges::new(0.0, 0.0, 0.0, 12.0));
}

#[test]
fn content_box_inset_includes_padding_border_and_scrollbar_reservation() {
    let padding = Edges::new(1.0, 2.0, 3.0, 4.0);
    let border = Edges::new(5.0, 6.0, 7.0, 8.0);
    let reservation = ScrollbarReservationOf::from_overflow(
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        false,
        9.0,
        Direction::Ltr,
    );

    assert_eq!(
        crate::scroll::content_box_inset_with_scrollbar(padding, border, reservation),
        Edges::new(6.0, 17.0, 10.0, 12.0)
    );
}

#[test]
fn canonical_geometry_exposes_hidden_range_and_clip() {
    let scrollable_overflow = ScrollRect::try_new(Point::ZERO, Size::new(140.0, 70.0)).unwrap();
    let geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
        item_is_replaced: false,
        border_box_size: Size::new(100.0, 40.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width_value: 0.0,
        scrollable_overflow,
    });

    assert_eq!(
        geometry.scrollport(),
        ScrollRect::try_new(Point::ZERO, Size::new(100.0, 40.0)).unwrap()
    );
    let clip = geometry.overflow_clip();
    assert_eq!(
        clip.x().unwrap().minimum(),
        geometry.scrollport().origin().x
    );
    assert_eq!(clip.x().unwrap().maximum(), 100.0);
    assert_eq!(
        clip.y().unwrap().minimum(),
        geometry.scrollport().origin().y
    );
    assert_eq!(clip.y().unwrap().maximum(), 40.0);
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(140.0, 70.0)).unwrap()
    );
    assert_physical_range_maximum(geometry.physical_range(), Size::new(40.0, 30.0));
}

#[test]
fn canonical_geometry_keeps_clip_range_zero() {
    let scrollable_overflow = ScrollRect::try_new(Point::ZERO, Size::new(140.0, 70.0)).unwrap();
    let geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow: computed_overflow(Overflow::Clip, Overflow::Clip),
        item_is_replaced: false,
        border_box_size: Size::new(100.0, 40.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width_value: 0.0,
        scrollable_overflow,
    });

    assert!(geometry.overflow_clip().x().is_some());
    assert!(geometry.overflow_clip().y().is_some());
    assert_physical_range_maximum(geometry.physical_range(), Size::ZERO);
}

#[test]
fn canonical_geometry_keeps_visible_range_zero_with_visible_overflow() {
    let scrollable_overflow = ScrollRect::try_new(Point::ZERO, Size::new(140.0, 70.0)).unwrap();
    let geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
        item_is_replaced: false,
        border_box_size: Size::new(100.0, 40.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width_value: 0.0,
        scrollable_overflow,
    });

    assert_eq!(geometry.overflow_clip().x(), None);
    assert_eq!(geometry.overflow_clip().y(), None);
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(140.0, 70.0)).unwrap()
    );
    assert_physical_range_maximum(geometry.physical_range(), Size::ZERO);
}

#[test]
fn canonical_geometry_accounts_for_scrollbar_gutter() {
    let scrollable_overflow =
        ScrollRect::try_new(Point::new(-20.0, 0.0), Size::new(120.0, 40.0)).unwrap();
    let geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        item_is_replaced: false,
        border_box_size: Size::new(100.0, 40.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width_value: 10.0,
        scrollable_overflow,
    });

    assert_eq!(
        geometry.scrollport(),
        ScrollRect::try_new(Point::new(10.0, 0.0), Size::new(90.0, 40.0)).unwrap()
    );
    assert_eq!(
        geometry.gutters().left(),
        Some(ScrollRect::try_new(Point::ZERO, Size::new(10.0, 40.0)).unwrap())
    );
    assert_eq!(geometry.physical_range().x().minimum(), -30.0);
    assert_eq!(geometry.physical_range().x().maximum(), 0.0);
    assert_eq!(geometry.physical_range().y().minimum(), 0.0);
    assert_eq!(geometry.physical_range().y().maximum(), 0.0);
}

#[test]
fn canonical_geometry_replaced_hidden_axis_has_no_range_when_other_axis_scrolls() {
    let scrollable_overflow = ScrollRect::try_new(Point::ZERO, Size::new(140.0, 70.0)).unwrap();
    let geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        item_is_replaced: true,
        border_box_size: Size::new(100.0, 40.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width_value: 0.0,
        scrollable_overflow,
    });

    assert!(geometry.overflow_clip().x().is_some());
    assert!(geometry.overflow_clip().y().is_some());
    assert_physical_range_maximum(geometry.physical_range(), Size::new(0.0, 30.0));
}

#[test]
fn source_rounding_rounds_rects_with_cumulative_origin() {
    let scrollable_overflow =
        ScrollRect::try_new(Point::new(0.25, 0.25), Size::new(10.5, 20.5)).unwrap();
    let geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
        item_is_replaced: false,
        border_box_size: Size::new(5.5, 6.5),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width_value: 0.0,
        scrollable_overflow,
    });

    let rounded =
        rebuild_rounded_canonical_scroll_geometry(geometry, Point::new(10.25, 20.25)).unwrap();

    assert_eq!(
        rounded.scrollport(),
        ScrollRect::try_new(Point::ZERO, Size::new(6.0, 7.0)).unwrap()
    );
    assert_eq!(rounded.overflow_clip().x().unwrap().minimum(), 0.0);
    assert_eq!(rounded.overflow_clip().x().unwrap().maximum(), 6.0);
    assert_eq!(rounded.overflow_clip().y().unwrap().minimum(), 0.0);
    assert_eq!(rounded.overflow_clip().y().unwrap().maximum(), 7.0);
    assert_eq!(
        rounded.scrollable_overflow(),
        ScrollRect::try_new(Point::new(1.0, 1.0), Size::new(10.0, 20.0)).unwrap()
    );
    assert_eq!(rounded.gutters().top(), None);
    assert_eq!(rounded.gutters().right(), None);
    assert_eq!(rounded.gutters().bottom(), None);
    assert_eq!(rounded.gutters().left(), None);
    assert_physical_range_maximum(rounded.physical_range(), Size::new(5.0, 14.0));
}

#[test]
fn scroll_geometry_projects_signed_ranges_for_all_flow_mappings_before_and_after_rounding() {
    fn assert_scalar<S: crate::LayoutScalar>() {
        let cases = [
            (
                WritingMode::HorizontalTb,
                Direction::Ltr,
                (0.0, 40.0, 0.0, 30.0),
            ),
            (
                WritingMode::HorizontalTb,
                Direction::Rtl,
                (-40.0, 0.0, 0.0, 30.0),
            ),
            (
                WritingMode::VerticalRl,
                Direction::Ltr,
                (-40.0, 0.0, 0.0, 30.0),
            ),
            (
                WritingMode::VerticalRl,
                Direction::Rtl,
                (-40.0, 0.0, -30.0, 0.0),
            ),
            (
                WritingMode::VerticalLr,
                Direction::Ltr,
                (0.0, 40.0, 0.0, 30.0),
            ),
            (
                WritingMode::VerticalLr,
                Direction::Rtl,
                (0.0, 40.0, -30.0, 0.0),
            ),
            (
                WritingMode::SidewaysRl,
                Direction::Ltr,
                (-40.0, 0.0, 0.0, 30.0),
            ),
            (
                WritingMode::SidewaysRl,
                Direction::Rtl,
                (-40.0, 0.0, -30.0, 0.0),
            ),
            (
                WritingMode::SidewaysLr,
                Direction::Ltr,
                (0.0, 40.0, -30.0, 0.0),
            ),
            (
                WritingMode::SidewaysLr,
                Direction::Rtl,
                (0.0, 40.0, 0.0, 30.0),
            ),
        ];

        for (writing_mode, direction, (x_minimum, x_maximum, y_minimum, y_maximum)) in cases {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let overflow_origin = Point::new(
                if flow_axes
                    .physical_axis_progression(PhysicalAxis::Horizontal)
                    .is_decreasing()
                {
                    S::from_f64(-40.0)
                } else {
                    S::ZERO
                },
                if flow_axes
                    .physical_axis_progression(PhysicalAxis::Vertical)
                    .is_decreasing()
                {
                    S::from_f64(-30.0)
                } else {
                    S::ZERO
                },
            );
            let geometry = canonical_test_geometry(CanonicalTestGeometryFactsOf {
                flow_axes,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                item_is_replaced: false,
                border_box_size: Size::new(S::from_f64(100.0), S::from_f64(40.0)),
                padding: Edges::ZERO,
                border: Edges::ZERO,
                scrollbar_width_value: S::ZERO,
                scrollable_overflow: ScrollRectOf::try_new(
                    overflow_origin,
                    Size::new(S::from_f64(140.0), S::from_f64(70.0)),
                )
                .expect("finite overflow rectangle is valid"),
            });

            for geometry in [
                geometry,
                rebuild_rounded_canonical_scroll_geometry(
                    geometry,
                    Point::new(S::from_f64(0.25), S::from_f64(0.25)),
                )
                .expect("finite rounded geometry is valid"),
            ] {
                let range = geometry.physical_range();
                assert_eq!(range.x().minimum(), S::from_f64(x_minimum));
                assert_eq!(range.x().maximum(), S::from_f64(x_maximum));
                assert_eq!(range.y().minimum(), S::from_f64(y_minimum));
                assert_eq!(range.y().maximum(), S::from_f64(y_maximum));
            }
        }
    }

    assert_scalar::<f32>();
    assert_scalar::<f64>();
}
