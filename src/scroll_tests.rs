use crate::scroll::{
    FlowRelativeScrollOffsetOf, FlowRelativeScrollRangeOf, PhysicalScrollOffsetOf,
    PhysicalScrollRangeOf, ScrollBoxRects, ScrollCoordinateErrorOf, ScrollRectOf,
    ScrollbarReservation, UsedOverflow, UsedOverflowGutter,
};
use crate::{
    ComputedOverflow, Direction, Edges, FlowAxes, LogicalAxis, Overflow, PhysicalAxis, Point,
    ScrollContainerAxis, ScrollContainerFacts, ScrollGeometry, ScrollOverflowExposure, ScrollRect,
    ScrollUnsupportedFeature, ScrollbarGutterRects, Size, WritingMode,
};

fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
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

        let ordinary_facts =
            crate::scroll::scroll_container_facts_from_overflow(computed_pair, false)
                .expect("computed pair lowers to used scroll facts");
        assert_eq!(ordinary_facts.x().clips_overflow(), clips);
        assert_eq!(ordinary_facts.y().clips_overflow(), clips);
        assert_eq!(ordinary_facts.x().exposes_scroll_range(), exposes_range);
        assert_eq!(ordinary_facts.y().exposes_scroll_range(), exposes_range);

        let replaced_facts =
            crate::scroll::scroll_container_facts_from_overflow(computed_pair, true)
                .expect("replaced pair lowers to used scroll facts");
        let replaced_clips = replaced != Overflow::Visible;
        let replaced_range = matches!(
            replaced,
            Overflow::Hidden | Overflow::Scroll | Overflow::Auto
        );
        assert_eq!(replaced_facts.x().clips_overflow(), replaced_clips);
        assert_eq!(replaced_facts.y().clips_overflow(), replaced_clips);
        assert_eq!(replaced_facts.x().exposes_scroll_range(), replaced_range);
        assert_eq!(replaced_facts.y().exposes_scroll_range(), replaced_range);
    }
}

fn physical_range(x_maximum: f32, y_maximum: f32) -> PhysicalScrollRangeOf {
    PhysicalScrollRangeOf::try_new(0.0, x_maximum, 0.0, y_maximum)
        .expect("finite physical range is valid")
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
fn scroll_rect_rejects_negative_or_non_finite_size() {
    assert_eq!(
        ScrollRect::new(Point::ZERO, Size::new(10.0, -1.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRect
    );
    assert_eq!(
        ScrollRect::new(Point::new(f32::NAN, 0.0), Size::new(10.0, 1.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRect
    );
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
fn scroll_container_facts_distinguish_hidden_clip_and_scroll() {
    let hidden = ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap();
    let clip = ScrollContainerAxis::from_overflow(Overflow::Clip).unwrap();
    let scroll = ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap();
    let visible = ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap();

    assert_eq!(hidden.exposure(), ScrollOverflowExposure::ScrollableClip);
    assert!(hidden.exposes_scroll_range());
    assert!(hidden.clips_overflow());
    assert_eq!(clip.exposure(), ScrollOverflowExposure::ClipOnly);
    assert!(!clip.exposes_scroll_range());
    assert!(clip.clips_overflow());
    assert_eq!(scroll.exposure(), ScrollOverflowExposure::ScrollableClip);
    assert!(scroll.exposes_scroll_range());
    assert!(scroll.clips_overflow());
    assert_eq!(visible.exposure(), ScrollOverflowExposure::Visible);
    assert!(!visible.exposes_scroll_range());
    assert!(!visible.clips_overflow());
    assert!(ScrollContainerFacts::new(hidden, visible).requires_overflow_clip());
    assert!(!ScrollContainerFacts::new(visible, visible).requires_overflow_clip());
}

#[test]
fn scroll_geometry_front_door_preserves_physical_rects_and_flow_metadata() {
    let scrollport = ScrollRect::new(Point::new(1.0, 2.0), Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let clip = ScrollRect::new(Point::new(1.0, 2.0), Size::new(80.0, 40.0)).unwrap();
    let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let range = PhysicalScrollRangeOf::try_new(-50.0, 0.0, -40.0, 0.0).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);
    let geometry = ScrollGeometry::new(
        flow_axes,
        ScrollContainerFacts::new(
            ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap(),
            ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
        ),
        scrollport,
        Some(clip),
        overflow,
        range,
        gutters,
    )
    .unwrap();

    assert_eq!(geometry.flow_axes(), flow_axes);
    assert_eq!(geometry.scrollport(), scrollport);
    assert_eq!(geometry.overflow_clip(), Some(clip));
    assert_eq!(geometry.scrollable_overflow(), overflow);
    assert_eq!(geometry.physical_range(), range);
}

#[test]
fn scroll_geometry_rejects_clipping_axis_without_clip_rect() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = physical_range(0.0, 0.0);
    let gutters = ScrollbarGutterRects::new(None, None);

    for overflow_x in [Overflow::Hidden, Overflow::Clip, Overflow::Scroll] {
        assert_eq!(
            ScrollGeometry::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ScrollContainerFacts::new(
                    ScrollContainerAxis::from_overflow(overflow_x).unwrap(),
                    ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
                ),
                scrollport,
                None,
                overflow,
                range,
                gutters,
            )
            .unwrap_err(),
            ScrollUnsupportedFeature::InvalidScrollGeometry
        );
    }

    assert_eq!(
        ScrollGeometry::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
            ),
            scrollport,
            None,
            overflow,
            range,
            gutters,
        )
        .unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollGeometry
    );
}

#[test]
fn scroll_geometry_allows_visible_axes_without_clip_rect() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = physical_range(0.0, 0.0);
    let gutters = ScrollbarGutterRects::new(None, None);

    let geometry = ScrollGeometry::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ScrollContainerFacts::new(
            ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
            ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
        ),
        scrollport,
        None,
        overflow,
        range,
        gutters,
    )
    .unwrap();

    assert_eq!(geometry.overflow_clip(), None);
}

#[test]
fn scroll_geometry_rejects_clip_only_axis_with_non_zero_range() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = physical_range(40.0, 0.0);
    let gutters = ScrollbarGutterRects::new(None, None);

    assert_eq!(
        ScrollGeometry::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Clip).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
            ),
            scrollport,
            Some(scrollport),
            overflow,
            range,
            gutters,
        )
        .unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollGeometry
    );
}

#[test]
fn scroll_geometry_rejects_visible_axis_with_non_zero_range() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = physical_range(0.0, 50.0);
    let gutters = ScrollbarGutterRects::new(None, None);

    assert_eq!(
        ScrollGeometry::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
            ),
            scrollport,
            None,
            overflow,
            range,
            gutters,
        )
        .unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollGeometry
    );
}

#[test]
fn scroll_geometry_error_maps_nonfinite_layout_range_to_invalid_geometry() {
    let error = crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        false,
        Size::new(f32::MAX, 1.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        ScrollRect::new(Point::new(f32::MAX, 0.0), Size::new(f32::MAX, 1.0))
            .expect("finite overflow rectangle is valid"),
    )
    .expect_err("non-finite layout-produced range must be invalid geometry");

    assert_eq!(error, ScrollUnsupportedFeature::InvalidScrollGeometry);
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
    let ltr = ScrollbarReservation::from_overflow(
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        false,
        12.0,
        Direction::Ltr,
    );
    let rtl = ScrollbarReservation::from_overflow(
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
    let reservation = ScrollbarReservation::from_overflow(
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
fn scrollbar_box_rects_derive_ltr_scrollport_and_gutter_rects() {
    let rects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::new(10.0, 20.0), Size::new(100.0, 80.0)).unwrap(),
        Edges::new(2.0, 3.0, 4.0, 5.0),
        Edges::all(1.0),
        ScrollbarReservation::from_overflow(
            computed_overflow(Overflow::Scroll, Overflow::Scroll),
            false,
            10.0,
            Direction::Ltr,
        ),
    )
    .unwrap();

    assert_eq!(
        rects.border_box(),
        ScrollRect::new(Point::new(10.0, 20.0), Size::new(100.0, 80.0)).unwrap()
    );
    assert_eq!(
        rects.padding_box(),
        ScrollRect::new(Point::new(11.0, 21.0), Size::new(98.0, 78.0)).unwrap()
    );
    assert_eq!(
        rects.content_box(),
        ScrollRect::new(Point::new(16.0, 23.0), Size::new(80.0, 62.0)).unwrap()
    );
    assert_eq!(
        rects.scrollport(),
        ScrollRect::new(Point::new(11.0, 21.0), Size::new(88.0, 68.0)).unwrap()
    );
    assert_eq!(
        rects.gutters().vertical(),
        Some(ScrollRect::new(Point::new(99.0, 21.0), Size::new(10.0, 68.0)).unwrap())
    );
    assert_eq!(
        rects.gutters().horizontal(),
        Some(ScrollRect::new(Point::new(11.0, 89.0), Size::new(88.0, 10.0)).unwrap())
    );
}

#[test]
fn scrollbar_box_rects_shift_rtl_scrollport_after_left_gutter() {
    let rects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Edges::ZERO,
        Edges::ZERO,
        ScrollbarReservation::from_overflow(
            computed_overflow(Overflow::Auto, Overflow::Scroll),
            false,
            12.0,
            Direction::Rtl,
        ),
    )
    .unwrap();

    assert_eq!(
        rects.scrollport(),
        ScrollRect::new(Point::new(12.0, 0.0), Size::new(88.0, 40.0)).unwrap()
    );
    assert_eq!(
        rects.gutters().vertical(),
        Some(ScrollRect::new(Point::ZERO, Size::new(12.0, 40.0)).unwrap())
    );
    assert_eq!(rects.gutters().horizontal(), None);
}

#[test]
fn scrollbar_box_rects_clamp_overlarge_insets_to_empty_rects() {
    let rects: ScrollBoxRects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::ZERO, Size::new(10.0, 10.0)).unwrap(),
        Edges::all(20.0),
        Edges::all(20.0),
        ScrollbarReservation::from_overflow(
            computed_overflow(Overflow::Scroll, Overflow::Scroll),
            false,
            20.0,
            Direction::Ltr,
        ),
    )
    .unwrap();

    assert_eq!(rects.content_box().size(), Size::ZERO);
    assert_eq!(rects.scrollport().size(), Size::ZERO);
}

#[test]
fn scroll_geometry_from_layout_exposes_hidden_range_and_clip() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Size::new(140.0, 70.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        false,
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(
        geometry.scrollport(),
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap()
    );
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(140.0, 70.0)).unwrap()
    );
    assert_physical_range_maximum(geometry.physical_range(), Size::new(40.0, 30.0));
}

#[test]
fn scroll_geometry_from_layout_keeps_clip_range_zero() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Size::new(140.0, 70.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        computed_overflow(Overflow::Clip, Overflow::Clip),
        false,
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_physical_range_maximum(geometry.physical_range(), Size::ZERO);
}

#[test]
fn scroll_geometry_from_layout_keeps_visible_range_zero_with_visible_overflow() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Size::new(140.0, 70.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        computed_overflow(Overflow::Visible, Overflow::Visible),
        false,
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(geometry.overflow_clip(), None);
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(140.0, 70.0)).unwrap()
    );
    assert_physical_range_maximum(geometry.physical_range(), Size::ZERO);
}

#[test]
fn scroll_geometry_from_layout_accounts_for_scrollbar_gutter() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::new(10.0, 0.0), Size::new(90.0, 40.0)).unwrap(),
        Size::new(120.0, 40.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        computed_overflow(Overflow::Hidden, Overflow::Scroll),
        false,
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        10.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(
        geometry.scrollport(),
        ScrollRect::new(Point::new(10.0, 0.0), Size::new(90.0, 40.0)).unwrap()
    );
    assert_eq!(
        geometry.gutters().vertical(),
        Some(ScrollRect::new(Point::ZERO, Size::new(10.0, 40.0)).unwrap())
    );
    assert_eq!(geometry.physical_range().x().minimum(), -30.0);
    assert_eq!(geometry.physical_range().x().maximum(), 0.0);
    assert_eq!(geometry.physical_range().y().minimum(), 0.0);
    assert_eq!(geometry.physical_range().y().maximum(), 0.0);
}

#[test]
fn scroll_geometry_from_layout_replaced_hidden_axis_has_no_range_when_other_axis_scrolls() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Size::new(140.0, 70.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        computed_overflow(Overflow::Hidden, Overflow::Scroll),
        true,
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_physical_range_maximum(geometry.physical_range(), Size::new(0.0, 30.0));
}

#[test]
fn round_scroll_geometry_rounds_rects_with_cumulative_origin() {
    let scrollable_overflow =
        ScrollRect::new(Point::new(0.25, 0.25), Size::new(10.5, 20.5)).unwrap();
    let geometry = ScrollGeometry::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ScrollContainerFacts::new(
            ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
            ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
        ),
        ScrollRect::new(Point::new(0.25, 0.25), Size::new(5.5, 6.5)).unwrap(),
        Some(ScrollRect::new(Point::new(0.25, 0.25), Size::new(5.5, 6.5)).unwrap()),
        scrollable_overflow,
        physical_range(5.0, 14.0),
        ScrollbarGutterRects::new(
            None,
            Some(ScrollRect::new(Point::new(5.75, 0.25), Size::new(1.0, 6.5)).unwrap()),
        ),
    )
    .unwrap();

    let rounded = crate::scroll::round_scroll_geometry(geometry, Point::new(10.25, 20.25)).unwrap();

    assert_eq!(
        rounded.scrollport(),
        ScrollRect::new(Point::new(1.0, 1.0), Size::new(5.0, 6.0)).unwrap()
    );
    assert_eq!(rounded.overflow_clip(), Some(rounded.scrollport()));
    assert_eq!(
        rounded.scrollable_overflow(),
        ScrollRect::new(Point::new(1.0, 1.0), Size::new(10.0, 20.0)).unwrap()
    );
    assert_eq!(
        rounded.gutters().vertical(),
        Some(ScrollRect::new(Point::new(6.0, 1.0), Size::new(1.0, 6.0)).unwrap())
    );
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
            let geometry = crate::scroll::scroll_geometry_from_layout(
                FlowAxes::new(writing_mode, direction),
                computed_overflow(Overflow::Hidden, Overflow::Hidden),
                false,
                Size::new(S::from_f64(100.0), S::from_f64(40.0)),
                Edges::ZERO,
                Edges::ZERO,
                S::ZERO,
                ScrollRectOf::new(
                    Point::ZERO,
                    Size::new(S::from_f64(140.0), S::from_f64(70.0)),
                )
                .expect("finite overflow rectangle is valid"),
            )
            .expect("finite layout geometry is valid");

            for geometry in [
                geometry,
                crate::scroll::round_scroll_geometry(
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
