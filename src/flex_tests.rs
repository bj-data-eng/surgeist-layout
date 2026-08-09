use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use proptest::prelude::*;

use crate::flex::FlexAxes;
use crate::geometry::PhysicalProgression;
use crate::test_support::layout_tree::{
    OracleMeasurementOf, OracleTree, OracleTreeOf, PublicLayoutTreeOf,
};
use crate::test_support::scroll_geometry::{
    assert_geometry_error as fri06_mr02_geometry_error_assert, assert_scroll_padding_inputs_exact,
    geometry_error_input as fri06_mr02_geometry_error_input,
    geometry_error_largest_finite as fri06_mr02_geometry_error_largest_finite,
    scroll_padding_inputs,
};
use crate::*;

type FlexTree<S = Scalar> = OracleTreeOf<S>;
type RecursiveTree = OracleTree;

fn fri07_c01_absolute_auto_margin_layout<S: LayoutScalar>(
    flow_axes: FlowAxes,
    mut container: NodeInputOf<S>,
    child: NodeInputOf<S>,
) -> NodeOutputOf<S> {
    container.display = Display::Flex;
    container.writing_mode = flow_axes.writing_mode();
    container.direction = flow_axes.direction();
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(1, container)
        .style(2, child);
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(300.0))))
            .expect("absolute auto-margin viewport is finite"),
    )
    .expect("absolute auto-margin layout succeeds");

    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 2)
        .expect("absolute flex child is published")
        .output()
}

fn assert_fri07_c01_absolute_auto_margin_auto_inset_zeroes_axis<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    let output = fri07_c01_absolute_auto_margin_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        NodeInputOf {
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(100.0)),
                PreferredSizeOf::px(S::from_f64(40.0)),
            ),
            ..NodeInputOf::default()
        },
        NodeInputOf {
            position: Position::Absolute,
            inset: Edges {
                left: px(0.0),
                top: px(0.0),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(20.0)),
                PreferredSizeOf::px(S::from_f64(10.0)),
            ),
            margin: Edges {
                top: LengthAutoOf::AUTO,
                right: LengthAutoOf::AUTO,
                bottom: LengthAutoOf::AUTO,
                left: LengthAutoOf::AUTO,
            },
            ..NodeInputOf::default()
        },
    );

    assert_eq!(output.margin, Edges::ZERO);
    assert_eq!(output.location, Point::ZERO);
}

#[test]
fn fri07_c01_absolute_auto_margin_auto_inset_zeroes_used_margins_in_both_scalar_lanes() {
    assert_fri07_c01_absolute_auto_margin_auto_inset_zeroes_axis::<f32>();
    assert_fri07_c01_absolute_auto_margin_auto_inset_zeroes_axis::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_start_auto_inset_matrix<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    let preferred_px = |value| PreferredSizeOf::px(S::from_f64(value));
    let container = || NodeInputOf {
        size: Size::new(preferred_px(100.0), preferred_px(40.0)),
        ..NodeInputOf::default()
    };

    for (name, inset, margin, expected_margin, expected_location) in [
        (
            "horizontal start auto",
            Edges {
                top: px(0.0),
                right: px(11.0),
                bottom: px(0.0),
                left: LengthAutoOf::AUTO,
            },
            Edges {
                top: px(0.0),
                right: px(7.0),
                bottom: px(0.0),
                left: LengthAutoOf::AUTO,
            },
            Edges::new(S::ZERO, S::from_f64(7.0), S::ZERO, S::ZERO),
            Point::new(S::from_f64(62.0), S::ZERO),
        ),
        (
            "vertical start auto",
            Edges {
                top: LengthAutoOf::AUTO,
                right: px(0.0),
                bottom: px(5.0),
                left: px(0.0),
            },
            Edges {
                top: LengthAutoOf::AUTO,
                right: px(0.0),
                bottom: px(9.0),
                left: px(0.0),
            },
            Edges::new(S::ZERO, S::ZERO, S::from_f64(9.0), S::ZERO),
            Point::new(S::ZERO, S::from_f64(16.0)),
        ),
    ] {
        let output = fri07_c01_absolute_auto_margin_layout(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            container(),
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: Size::new(preferred_px(20.0), preferred_px(10.0)),
                margin,
                ..NodeInputOf::default()
            },
        );

        assert_eq!(output.margin, expected_margin, "{name} used margins");
        assert_eq!(output.location, expected_location, "{name} placement");
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_start_auto_insets_zero_only_auto_margins() {
    assert_fri07_c01_absolute_auto_margin_start_auto_inset_matrix::<f32>();
    assert_fri07_c01_absolute_auto_margin_start_auto_inset_matrix::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_definite_inset_matrix<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    let preferred_px = |value| PreferredSizeOf::px(S::from_f64(value));
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let container = || NodeInputOf {
        size: Size::new(preferred_px(100.0), preferred_px(40.0)),
        ..NodeInputOf::default()
    };
    let inset = Edges {
        top: px(0.0),
        right: px(20.0),
        bottom: px(0.0),
        left: px(10.0),
    };

    for (name, width, left, right, expected_left, expected_right, expected_x) in [
        ("no auto", 20.0, px(3.0), px(5.0), 3.0, 5.0, 13.0),
        (
            "one auto positive",
            20.0,
            LengthAutoOf::AUTO,
            px(5.0),
            45.0,
            5.0,
            55.0,
        ),
        (
            "one auto negative",
            80.0,
            LengthAutoOf::AUTO,
            px(5.0),
            -15.0,
            5.0,
            -5.0,
        ),
        (
            "two auto positive",
            20.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            25.0,
            25.0,
            35.0,
        ),
        (
            "two auto zero",
            70.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            0.0,
            0.0,
            10.0,
        ),
        (
            "two auto negative inline",
            100.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            0.0,
            -30.0,
            10.0,
        ),
    ] {
        let output = fri07_c01_absolute_auto_margin_layout(
            flow_axes,
            container(),
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: Size::new(preferred_px(width), preferred_px(10.0)),
                margin: Edges {
                    top: px(0.0),
                    right,
                    bottom: px(0.0),
                    left,
                },
                ..NodeInputOf::default()
            },
        );

        assert_eq!(
            output.margin.left,
            S::from_f64(expected_left),
            "{name} left"
        );
        assert_eq!(
            output.margin.right,
            S::from_f64(expected_right),
            "{name} right"
        );
        assert_eq!(output.location.x, S::from_f64(expected_x), "{name} x");
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_definite_insets_use_signed_inset_modified_space() {
    assert_fri07_c01_absolute_auto_margin_definite_inset_matrix::<f32>();
    assert_fri07_c01_absolute_auto_margin_definite_inset_matrix::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_flow_mapping<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let container_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            S::from_f64(100.0),
            S::from_f64(60.0),
        ));
        let child_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            S::from_f64(120.0),
            S::from_f64(20.0),
        ));
        let inset = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            px(0.0),
            px(0.0),
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
        ));
        let margin = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            px(0.0),
            px(0.0),
        ));
        let output = fri07_c01_absolute_auto_margin_layout(
            flow_axes,
            NodeInputOf {
                size: container_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: child_size.map(PreferredSizeOf::px),
                margin,
                ..NodeInputOf::default()
            },
        );
        let logical_margin = flow_axes.logical_edges(output.margin);

        assert_eq!(
            logical_margin.inline_start,
            S::ZERO,
            "{flow_axes:?} inline start"
        );
        assert_eq!(
            logical_margin.inline_end,
            S::from_f64(-20.0),
            "{flow_axes:?} inline end"
        );
        match flow_axes.inline_start() {
            PhysicalSide::Left => assert_eq!(output.location.x, S::ZERO, "{flow_axes:?} x"),
            PhysicalSide::Right => {
                assert_eq!(output.location.x, S::from_f64(-20.0), "{flow_axes:?} x")
            }
            PhysicalSide::Top => assert_eq!(output.location.y, S::ZERO, "{flow_axes:?} y"),
            PhysicalSide::Bottom => {
                assert_eq!(output.location.y, S::from_f64(-20.0), "{flow_axes:?} y")
            }
        }
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_negative_inline_space_uses_containing_flow_start() {
    assert_fri07_c01_absolute_auto_margin_flow_mapping::<f32>();
    assert_fri07_c01_absolute_auto_margin_flow_mapping::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_negative_block_space_divides<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let container_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            S::from_f64(60.0),
            S::from_f64(100.0),
        ));
        let child_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            S::from_f64(20.0),
            S::from_f64(120.0),
        ));
        let inset = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            px(0.0),
            px(0.0),
        ));
        let margin = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            px(0.0),
            px(0.0),
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
        ));
        let output = fri07_c01_absolute_auto_margin_layout(
            flow_axes,
            NodeInputOf {
                size: container_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: child_size.map(PreferredSizeOf::px),
                margin,
                ..NodeInputOf::default()
            },
        );
        let logical_margin = flow_axes.logical_edges(output.margin);

        assert_eq!(
            logical_margin.block_start,
            S::from_f64(-10.0),
            "{flow_axes:?} block start"
        );
        assert_eq!(
            logical_margin.block_end,
            S::from_f64(-10.0),
            "{flow_axes:?} block end"
        );
        match flow_axes.block_axis() {
            PhysicalAxis::Horizontal => {
                assert_eq!(output.location.x, S::from_f64(-10.0), "{flow_axes:?} x")
            }
            PhysicalAxis::Vertical => {
                assert_eq!(output.location.y, S::from_f64(-10.0), "{flow_axes:?} y")
            }
        }
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_negative_block_space_divides_normally() {
    assert_fri07_c01_absolute_auto_margin_negative_block_space_divides::<f32>();
    assert_fri07_c01_absolute_auto_margin_negative_block_space_divides::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_padding_border_box_sizing<S: LayoutScalar>() {
    let length = |value| LengthOf::px(S::from_f64(value));
    let auto_length = |value| LengthAutoOf::px(S::from_f64(value));
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let container = || NodeInputOf {
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(preferred(120.0), preferred(80.0)),
        padding: Edges::new(length(10.0), length(10.0), length(10.0), length(10.0)),
        border: Edges::new(length(5.0), length(5.0), length(5.0), length(5.0)),
        ..NodeInputOf::default()
    };
    let child_edges = Edges::new(length(1.0), length(4.0), length(1.0), length(2.0));
    let child_padding = Edges::new(length(1.0), length(7.0), length(1.0), length(3.0));
    let inset = Edges {
        top: auto_length(0.0),
        right: auto_length(20.0),
        bottom: auto_length(0.0),
        left: auto_length(10.0),
    };
    let auto_inline_margin = Edges {
        top: LengthAutoOf::ZERO,
        right: LengthAutoOf::AUTO,
        bottom: LengthAutoOf::ZERO,
        left: LengthAutoOf::AUTO,
    };

    for (box_sizing, expected_size, expected_margin, expected_x) in [
        (BoxSizing::ContentBox, 36.0, 22.0, 37.0),
        (BoxSizing::BorderBox, 20.0, 30.0, 45.0),
    ] {
        let output = fri07_c01_absolute_auto_margin_layout(
            flow_axes,
            container(),
            NodeInputOf {
                position: Position::Absolute,
                box_sizing,
                inset,
                size: Size::new(preferred(20.0), preferred(10.0)),
                padding: child_padding,
                border: child_edges,
                margin: auto_inline_margin,
                ..NodeInputOf::default()
            },
        );

        assert_eq!(
            output.size.width,
            S::from_f64(expected_size),
            "{box_sizing:?} width"
        );
        assert_eq!(
            output.margin.left,
            S::from_f64(expected_margin),
            "{box_sizing:?} left"
        );
        assert_eq!(
            output.margin.right,
            S::from_f64(expected_margin),
            "{box_sizing:?} right"
        );
        assert_eq!(
            output.location.x,
            S::from_f64(expected_x),
            "{box_sizing:?} x"
        );
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_uses_containing_padding_box_and_used_border_box() {
    assert_fri07_c01_absolute_auto_margin_padding_border_box_sizing::<f32>();
    assert_fri07_c01_absolute_auto_margin_padding_border_box_sizing::<f64>();
}

fn fri07_c01_cross_auto_margin_output<S: LayoutScalar>(
    block_start: LengthAutoOf<S>,
    block_end: LengthAutoOf<S>,
) -> NodeOutputOf<S> {
    fri07_c01_cross_auto_margin_case(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        FlexDirection::Row,
        FlexWrap::NoWrap,
        40.0,
        60.0,
        (block_start, block_end),
        AlignItems::FlexStart,
    )
}

fn fri07_c01_cross_auto_margin_case<S: LayoutScalar>(
    flow_axes: FlowAxes,
    flex_direction: FlexDirection,
    flex_wrap: FlexWrap,
    line_cross: f64,
    item_cross: f64,
    logical_cross_margin: (LengthAutoOf<S>, LengthAutoOf<S>),
    align_items: AlignItems,
) -> NodeOutputOf<S> {
    let axes = FlexAxes::new(flow_axes, flex_direction, flex_wrap);
    let (logical_cross_start, logical_cross_end) = logical_cross_margin;
    let zero = LengthAutoOf::ZERO;
    let margin = if flex_direction.is_row() {
        flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            zero,
            zero,
            logical_cross_start,
            logical_cross_end,
        ))
    } else {
        flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            logical_cross_start,
            logical_cross_end,
            zero,
            zero,
        ))
    };
    let container_size = axes.size_from_main_cross(S::from_f64(100.0), S::from_f64(line_cross));
    let item_size = axes.size_from_main_cross(S::from_f64(20.0), S::from_f64(item_cross));
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                flex_direction,
                flex_wrap,
                size: container_size.map(PreferredSizeOf::px),
                align_items: Some(align_items),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: item_size.map(PreferredSizeOf::px),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                margin,
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(200.0))))
            .expect("cross auto-margin viewport is finite"),
    )
    .expect("cross auto-margin layout succeeds");

    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 2)
        .expect("cross auto-margin layout publishes the flex item")
        .output()
}

#[test]
fn fri07_c01_cross_auto_margin_both_auto_overflow_anchors_normal_logical_start() {
    let output = fri07_c01_cross_auto_margin_output::<f32>(LengthAutoOf::AUTO, LengthAutoOf::AUTO);

    assert_eq!(output.margin.top, 0.0);
    assert_eq!(output.margin.bottom, -20.0);
    assert_eq!(output.location, Point::new(0.0, 0.0));
}

#[test]
fn fri07_c01_cross_auto_margin_auto_start_overflow_replaces_fixed_opposite() {
    let output =
        fri07_c01_cross_auto_margin_output::<f32>(LengthAutoOf::AUTO, LengthAutoOf::px(5.0));

    assert_eq!(output.margin.top, 0.0);
    assert_eq!(output.margin.bottom, -20.0);
    assert_eq!(output.location, Point::new(0.0, 0.0));
}

fn assert_fri07_c01_cross_auto_margin_signed_matrix<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    for (name, line, item, start, end, expected_start, expected_end) in [
        ("positive neither", 40.0, 20.0, px(3.0), px(5.0), 3.0, 5.0),
        (
            "positive start",
            40.0,
            20.0,
            LengthAutoOf::AUTO,
            px(5.0),
            15.0,
            5.0,
        ),
        (
            "positive end",
            40.0,
            20.0,
            px(5.0),
            LengthAutoOf::AUTO,
            5.0,
            15.0,
        ),
        (
            "positive both",
            40.0,
            20.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            10.0,
            10.0,
        ),
        ("zero neither", 40.0, 32.0, px(3.0), px(5.0), 3.0, 5.0),
        (
            "zero start",
            40.0,
            35.0,
            LengthAutoOf::AUTO,
            px(5.0),
            0.0,
            5.0,
        ),
        (
            "zero end",
            40.0,
            35.0,
            px(5.0),
            LengthAutoOf::AUTO,
            5.0,
            0.0,
        ),
        (
            "zero both",
            40.0,
            40.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            0.0,
            0.0,
        ),
        ("negative neither", 40.0, 60.0, px(3.0), px(5.0), 3.0, 5.0),
        (
            "negative start",
            40.0,
            60.0,
            LengthAutoOf::AUTO,
            px(5.0),
            0.0,
            -20.0,
        ),
        (
            "negative end",
            40.0,
            60.0,
            px(5.0),
            LengthAutoOf::AUTO,
            5.0,
            -25.0,
        ),
        (
            "negative both",
            40.0,
            60.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            0.0,
            -20.0,
        ),
    ] {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let axes = FlexAxes::new(flow_axes, FlexDirection::Row, FlexWrap::NoWrap);
        let output = fri07_c01_cross_auto_margin_case(
            flow_axes,
            FlexDirection::Row,
            FlexWrap::NoWrap,
            line,
            item,
            (start, end),
            AlignItems::FlexStart,
        );

        assert_eq!(
            axes.normal_cross_start_edge(output.margin),
            S::from_f64(expected_start),
            "{name} logical start"
        );
        assert_eq!(
            axes.normal_cross_end_edge(output.margin),
            S::from_f64(expected_end),
            "{name} logical end"
        );
    }
}

#[test]
fn fri07_c01_cross_auto_margin_signed_space_covers_every_auto_edge_pattern() {
    assert_fri07_c01_cross_auto_margin_signed_matrix::<f32>();
    assert_fri07_c01_cross_auto_margin_signed_matrix::<f64>();
}

fn fri07_c01_cross_auto_margin_origin_from_side<S: LayoutScalar>(
    side: PhysicalSide,
    container_extent: S,
    item_extent: S,
) -> S {
    match side {
        PhysicalSide::Top | PhysicalSide::Left => S::ZERO,
        PhysicalSide::Right | PhysicalSide::Bottom => container_extent - item_extent,
    }
}

fn assert_fri07_c01_cross_auto_margin_axis_mapping<S: LayoutScalar>() {
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        for flex_direction in [
            FlexDirection::Row,
            FlexDirection::RowReverse,
            FlexDirection::Column,
            FlexDirection::ColumnReverse,
        ] {
            let axes = FlexAxes::new(flow_axes, flex_direction, FlexWrap::NoWrap);
            let output = fri07_c01_cross_auto_margin_case::<S>(
                flow_axes,
                flex_direction,
                FlexWrap::NoWrap,
                40.0,
                60.0,
                (LengthAutoOf::AUTO, LengthAutoOf::AUTO),
                AlignItems::FlexStart,
            );
            let normal_cross_start = if flex_direction.is_row() {
                flow_axes.block_start()
            } else {
                flow_axes.inline_start()
            };

            assert_eq!(
                axes.normal_cross_start_edge(output.margin),
                S::ZERO,
                "{flow_axes:?} {flex_direction:?} logical start"
            );
            assert_eq!(
                axes.normal_cross_end_edge(output.margin),
                S::from_f64(-20.0),
                "{flow_axes:?} {flex_direction:?} logical end"
            );
            assert_eq!(
                axes.cross_point(output.location),
                fri07_c01_cross_auto_margin_origin_from_side(
                    normal_cross_start,
                    S::from_f64(40.0),
                    S::from_f64(60.0),
                ),
                "{flow_axes:?} {flex_direction:?} cross geometry"
            );
            assert_eq!(
                axes.main_point(output.location),
                fri07_c01_cross_auto_margin_origin_from_side(
                    axes.main_start_side(),
                    S::from_f64(100.0),
                    S::from_f64(20.0),
                ),
                "{flow_axes:?} {flex_direction:?} main geometry"
            );
            assert_eq!(
                output.size,
                axes.size_from_main_cross(S::from_f64(20.0), S::from_f64(60.0)),
                "{flow_axes:?} {flex_direction:?} physical size"
            );
        }
    }
}

#[test]
fn fri07_c01_cross_auto_margin_maps_all_flows_axes_and_main_reversals() {
    assert_fri07_c01_cross_auto_margin_axis_mapping::<f32>();
    assert_fri07_c01_cross_auto_margin_axis_mapping::<f64>();
}

fn assert_fri07_c01_cross_auto_margin_wrap_progression<S: LayoutScalar>() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (flex_wrap, expected_cross) in [(FlexWrap::Wrap, 0.0), (FlexWrap::WrapReverse, -20.0)] {
        let axes = FlexAxes::new(flow_axes, FlexDirection::Row, flex_wrap);
        let output = fri07_c01_cross_auto_margin_case::<S>(
            flow_axes,
            FlexDirection::Row,
            flex_wrap,
            40.0,
            60.0,
            (LengthAutoOf::AUTO, LengthAutoOf::AUTO),
            AlignItems::FlexStart,
        );

        assert_eq!(axes.normal_cross_start_edge(output.margin), S::ZERO);
        assert_eq!(axes.normal_cross_end_edge(output.margin), S::ZERO);
        assert_eq!(
            axes.cross_point(output.location),
            S::from_f64(expected_cross),
            "{flex_wrap:?} keeps its wrap-aware physical progression"
        );
    }
}

#[test]
fn fri07_c01_cross_auto_margin_wrap_reversal_only_reverses_line_progression() {
    assert_fri07_c01_cross_auto_margin_wrap_progression::<f32>();
    assert_fri07_c01_cross_auto_margin_wrap_progression::<f64>();
}

fn assert_fri07_c01_cross_auto_margin_controls<S: LayoutScalar>() {
    let viewport = || {
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(200.0))))
            .expect("cross auto-margin control viewport is finite")
    };
    let main_auto_tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::px(S::from_f64(40.0)),
                ),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::splat_clone(PreferredSizeOf::px(S::from_f64(20.0))),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::ZERO,
                    LengthAutoOf::ZERO,
                    LengthAutoOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        );
    let main_auto_batch = compute_layout(&main_auto_tree, 1, viewport())
        .expect("ordinary main-axis auto margin remains supported");
    let main_auto = main_auto_batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 2)
        .expect("main-axis control publishes the item")
        .output();
    assert_eq!(main_auto.margin.left, S::from_f64(80.0));
    assert_eq!(main_auto.location, Point::new(S::from_f64(80.0), S::ZERO));

    let centered = fri07_c01_cross_auto_margin_case::<S>(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        FlexDirection::Row,
        FlexWrap::NoWrap,
        40.0,
        20.0,
        (LengthAutoOf::ZERO, LengthAutoOf::ZERO),
        AlignItems::Center,
    );
    assert_eq!(centered.margin, Edges::ZERO);
    assert_eq!(centered.location, Point::new(S::ZERO, S::from_f64(10.0)));
}

#[test]
fn fri07_c01_cross_auto_margin_preserves_main_auto_and_non_auto_cross_alignment() {
    assert_fri07_c01_cross_auto_margin_controls::<f32>();
    assert_fri07_c01_cross_auto_margin_controls::<f64>();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C01IntrinsicMeasureError {
    ProviderFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C01IntrinsicMeasureMode {
    Values,
    ProviderFailure,
    NonFinite,
}

#[derive(Clone, Debug)]
struct Fri07C01IntrinsicTree<S: LayoutScalar> {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInputOf<S>>,
    measured_nodes: Vec<u32>,
    leaf_requests: RefCell<HashMap<u32, Vec<LeafMeasureInputOf<S>>>>,
    mode: Fri07C01IntrinsicMeasureMode,
}

impl<S: LayoutScalar> Fri07C01IntrinsicTree<S> {
    fn new(mode: Fri07C01IntrinsicMeasureMode) -> Self {
        Self {
            children: HashMap::new(),
            styles: HashMap::new(),
            measured_nodes: Vec::new(),
            leaf_requests: RefCell::new(HashMap::new()),
            mode,
        }
    }

    fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    fn style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.styles.insert(node, style);
        self
    }

    fn measured(mut self, node: u32) -> Self {
        self.measured_nodes.push(node);
        self
    }

    fn leaf_requests(&self, node: u32) -> Vec<LeafMeasureInputOf<S>> {
        self.leaf_requests
            .borrow()
            .get(&node)
            .cloned()
            .unwrap_or_default()
    }
}

impl<S: LayoutScalar> Traverse for Fri07C01IntrinsicTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C01IntrinsicTree<S> {
    type MeasureError = Fri07C01IntrinsicMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        self.styles
            .get(&node)
            .unwrap_or_else(|| panic!("intrinsic test node {node} must have style"))
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.measured_nodes.contains(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.leaf_requests
            .borrow_mut()
            .entry(node)
            .or_default()
            .push(input);
        match self.mode {
            Fri07C01IntrinsicMeasureMode::ProviderFailure => {
                Some(Err(Fri07C01IntrinsicMeasureError::ProviderFailure))
            }
            Fri07C01IntrinsicMeasureMode::NonFinite => {
                Some(Ok(Size::new(Self::Scalar::INFINITY, Self::Scalar::ONE)))
            }
            Fri07C01IntrinsicMeasureMode::Values => {
                let available = input.available_content_size();
                let intrinsic = |available| match available {
                    MeasurementAvailableOf::MinContent => Some(Self::Scalar::from_f64(20.0)),
                    MeasurementAvailableOf::MaxContent => Some(Self::Scalar::from_f64(100.0)),
                    MeasurementAvailableOf::Definite(_) => None,
                };
                Some(Ok(Size::new(
                    intrinsic(available.width).unwrap_or(Self::Scalar::from_f64(10.0)),
                    intrinsic(available.height).unwrap_or(Self::Scalar::from_f64(10.0)),
                )))
            }
        }
    }
}

fn fri07_c01_intrinsic_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .expect("intrinsic public layout publishes the requested node")
        .output()
}

fn fri07_c01_intrinsic_leaf_tree<S: LayoutScalar>(
    direction: FlexDirection,
    child_writing_mode: WritingMode,
    mode: Fri07C01IntrinsicMeasureMode,
) -> Fri07C01IntrinsicTree<S> {
    let container_size = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(200.0)),
            PreferredSizeOf::px(S::from_f64(40.0)),
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(40.0)),
            PreferredSizeOf::px(S::from_f64(200.0)),
        ),
    };
    let preferred = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(77.0)),
            PreferredSizeOf::AUTO,
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSizeOf::AUTO,
            PreferredSizeOf::px(S::from_f64(77.0)),
        ),
    };

    Fri07C01IntrinsicTree::new(mode)
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                flex_direction: direction,
                size: container_size,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode: child_writing_mode,
                size: preferred.clone(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::MIN_CONTENT,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                writing_mode: child_writing_mode,
                size: preferred,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::MAX_CONTENT,
                ..NodeInputOf::default()
            },
        )
        .measured(2)
        .measured(3)
}

fn fri07_c01_intrinsic_child_container_tree<S: LayoutScalar>(
    direction: FlexDirection,
    child_writing_mode: WritingMode,
) -> Fri07C01IntrinsicTree<S> {
    let container_size = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(200.0)),
            PreferredSizeOf::px(S::from_f64(40.0)),
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(40.0)),
            PreferredSizeOf::px(S::from_f64(200.0)),
        ),
    };
    let preferred = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(77.0)),
            PreferredSizeOf::AUTO,
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSizeOf::AUTO,
            PreferredSizeOf::px(S::from_f64(77.0)),
        ),
    };

    Fri07C01IntrinsicTree::new(Fri07C01IntrinsicMeasureMode::Values)
        .children(1, [2, 3])
        .children(2, [4])
        .children(3, [5])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                flex_direction: direction,
                size: container_size,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: child_writing_mode,
                size: preferred.clone(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::MIN_CONTENT,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Block,
                writing_mode: child_writing_mode,
                size: preferred,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::MAX_CONTENT,
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                writing_mode: child_writing_mode,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                writing_mode: child_writing_mode,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .measured(4)
        .measured(5)
}

fn fri07_c01_intrinsic_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(300.0))))
        .expect("intrinsic test viewport is finite")
}

fn fri07_c01_intrinsic_recomputation_tree<S: LayoutScalar>(
    flex_basis: FlexBasisOf<S>,
) -> Fri07C01IntrinsicTree<S> {
    Fri07C01IntrinsicTree::new(Fri07C01IntrinsicMeasureMode::Values)
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(200.0)),
                    PreferredSizeOf::px(S::from_f64(40.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(77.0)),
                    PreferredSizeOf::AUTO,
                ),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis,
                flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow"),
                ..NodeInputOf::default()
            },
        )
        .measured(2)
}

fn assert_fri07_c01_intrinsic_provider_constraint_survives_recomputation<S: LayoutScalar>() {
    let scenarios = [
        (
            FlexBasisOf::<S>::MIN_CONTENT,
            MeasurementAvailableOf::MIN_CONTENT,
        ),
        (
            FlexBasisOf::<S>::MAX_CONTENT,
            MeasurementAvailableOf::MAX_CONTENT,
        ),
    ];
    let observed = scenarios
        .iter()
        .map(|(flex_basis, _)| {
            let tree = fri07_c01_intrinsic_recomputation_tree(flex_basis.clone());
            let batch = compute_layout(&tree, 1, fri07_c01_intrinsic_request())
                .expect("intrinsic flex basis remains supported through final layout");
            let requests = tree
                .leaf_requests(2)
                .into_iter()
                .map(|input| {
                    (
                        input.known_content_size().width,
                        input.available_content_size().width,
                    )
                })
                .collect::<Vec<_>>();
            (requests, fri07_c01_intrinsic_output(&batch, 2).size.width)
        })
        .collect::<Vec<_>>();
    let expected = scenarios
        .into_iter()
        .map(|(_, expected)| {
            (
                vec![
                    (None, expected),
                    (Some(S::from_f64(200.0)), expected),
                    (None, expected),
                ],
                S::from_f64(200.0),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        observed, expected,
        "the provider must receive each selected intrinsic main-axis constraint for initial, cross-recomputation, and final requests while grown final geometry remains 200px",
    );
}

#[test]
fn fri07_c01_intrinsic_public_layout_retains_provider_constraint_through_final_layout() {
    assert_fri07_c01_intrinsic_provider_constraint_survives_recomputation::<f32>();
    assert_fri07_c01_intrinsic_provider_constraint_survives_recomputation::<f64>();
}

fn assert_fri07_c01_intrinsic_leaf_geometry<S: LayoutScalar>() {
    for (direction, child_writing_mode, axis) in [
        (
            FlexDirection::Row,
            WritingMode::HorizontalTb,
            PhysicalAxis::Horizontal,
        ),
        (
            FlexDirection::Column,
            WritingMode::HorizontalTb,
            PhysicalAxis::Vertical,
        ),
        (
            FlexDirection::Column,
            WritingMode::VerticalRl,
            PhysicalAxis::Vertical,
        ),
    ] {
        let tree = fri07_c01_intrinsic_leaf_tree::<S>(
            direction,
            child_writing_mode,
            Fri07C01IntrinsicMeasureMode::Values,
        );
        let batch = compute_layout(&tree, 1, fri07_c01_intrinsic_request())
            .expect("direct intrinsic flex bases are supported");
        let min = fri07_c01_intrinsic_output(&batch, 2).size;
        let max = fri07_c01_intrinsic_output(&batch, 3).size;
        let main = |size: Size<S>| match axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        };
        assert_eq!(main(min), S::from_f64(20.0));
        assert_eq!(main(max), S::from_f64(100.0));
    }
}

#[test]
fn fri07_c01_intrinsic_public_layout_preserves_distinct_leaf_geometry_in_both_scalar_lanes() {
    assert_fri07_c01_intrinsic_leaf_geometry::<f32>();
    assert_fri07_c01_intrinsic_leaf_geometry::<f64>();
}

fn assert_fri07_c01_intrinsic_child_container_geometry<S: LayoutScalar>() {
    for (direction, child_writing_mode, axis) in [
        (
            FlexDirection::Row,
            WritingMode::HorizontalTb,
            PhysicalAxis::Horizontal,
        ),
        (
            FlexDirection::Column,
            WritingMode::VerticalRl,
            PhysicalAxis::Vertical,
        ),
    ] {
        let tree = fri07_c01_intrinsic_child_container_tree::<S>(direction, child_writing_mode);
        let batch = compute_layout(&tree, 1, fri07_c01_intrinsic_request())
            .expect("intrinsic child-container flex bases are supported");
        let min = fri07_c01_intrinsic_output(&batch, 2).size;
        let max = fri07_c01_intrinsic_output(&batch, 3).size;
        let main = |size: Size<S>| match axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        };
        assert_eq!(main(min), S::from_f64(20.0));
        assert_eq!(main(max), S::from_f64(100.0));
    }
}

#[test]
fn fri07_c01_intrinsic_public_layout_preserves_child_container_geometry_in_both_scalar_lanes() {
    assert_fri07_c01_intrinsic_child_container_geometry::<f32>();
    assert_fri07_c01_intrinsic_child_container_geometry::<f64>();
}

fn assert_fri07_c01_intrinsic_measurement_errors<S: LayoutScalar>() {
    let provider_tree = fri07_c01_intrinsic_leaf_tree::<S>(
        FlexDirection::Row,
        WritingMode::HorizontalTb,
        Fri07C01IntrinsicMeasureMode::ProviderFailure,
    );
    let provider_error = compute_layout(&provider_tree, 1, fri07_c01_intrinsic_request())
        .expect_err("intrinsic provider failure must remain typed");
    assert_eq!(provider_error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(provider_error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        provider_error.kind(),
        LayoutErrorKindOf::Measurement(Fri07C01IntrinsicMeasureError::ProviderFailure)
    ));

    let non_finite_tree = fri07_c01_intrinsic_leaf_tree::<S>(
        FlexDirection::Row,
        WritingMode::HorizontalTb,
        Fri07C01IntrinsicMeasureMode::NonFinite,
    );
    let non_finite_error = compute_layout(&non_finite_tree, 1, fri07_c01_intrinsic_request())
        .expect_err("intrinsic non-finite provider output must remain typed");
    assert_eq!(non_finite_error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(
        non_finite_error.operation(),
        LayoutOperation::LeafMeasurement
    );
    let LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(invalid)) =
        non_finite_error.kind()
    else {
        panic!("expected invalid measurement output, got {non_finite_error:?}");
    };
    assert_eq!(invalid.axis(), PhysicalAxis::Horizontal);
}

#[test]
fn fri07_c01_intrinsic_provider_failure_and_non_finite_output_remain_exact_in_both_scalar_lanes() {
    assert_fri07_c01_intrinsic_measurement_errors::<f32>();
    assert_fri07_c01_intrinsic_measurement_errors::<f64>();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C01CompositionMeasureMode {
    Values,
    Failure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C01CompositionMeasureError {
    ProviderFailure,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Fri07C01CompositionRetained<S: LayoutScalar> {
    unrounded: HashMap<u32, NodeOutputOf<S>>,
    final_outputs: HashMap<u32, NodeOutputOf<S>>,
    caches: HashMap<u32, CacheOf<S>>,
}

#[derive(Clone, Debug)]
struct Fri07C01CompositionTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    measure_mode: Cell<Fri07C01CompositionMeasureMode>,
    measurement_requests: RefCell<Vec<(u32, LeafMeasureInputOf<S>)>>,
    cache_queries: RefCell<Vec<(u32, bool)>>,
    retained: Fri07C01CompositionRetained<S>,
}

impl<S: LayoutScalar> Fri07C01CompositionTree<S> {
    fn new() -> Self {
        let px = |value| PreferredSizeOf::px(S::from_f64(value));
        let auto_px = |value| LengthAutoOf::px(S::from_f64(value));
        let intrinsic_margin = Edges {
            top: LengthAutoOf::AUTO,
            right: LengthAutoOf::ZERO,
            bottom: LengthAutoOf::AUTO,
            left: LengthAutoOf::ZERO,
        };
        let intrinsic_item = |basis, order, replaced| NodeInputOf {
            item_is_replaced: replaced,
            item_order: ItemOrder::new(order),
            size: Size::new(PreferredSizeOf::AUTO, px(50.4)),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            flex_basis: basis,
            flex_grow: FlexGrowOf::try_new(S::ZERO).expect("zero is a valid flex grow"),
            flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
            margin: intrinsic_margin,
            ..NodeInputOf::default()
        };
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2, 3, 4])
            .children(2, [])
            .children(3, [])
            .children(4, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(px(130.0), px(40.0)),
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(5.0))
                        .expect("composition scrollbar width is finite"),
                    align_items: Some(AlignItems::FlexStart),
                    ..NodeInputOf::default()
                },
            )
            .style(2, intrinsic_item(FlexBasisOf::MIN_CONTENT, 2, false))
            .style(
                3,
                NodeInputOf {
                    position: Position::Absolute,
                    item_order: ItemOrder::new(-100),
                    inset: Edges::new(auto_px(0.0), auto_px(20.0), auto_px(0.0), auto_px(10.0)),
                    size: Size::new(px(20.0), px(10.0)),
                    margin: Edges::all(LengthAutoOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .style(4, intrinsic_item(FlexBasisOf::MAX_CONTENT, -2, true));

        Self {
            tree,
            measure_mode: Cell::new(Fri07C01CompositionMeasureMode::Values),
            measurement_requests: RefCell::new(Vec::new()),
            cache_queries: RefCell::new(Vec::new()),
            retained: Fri07C01CompositionRetained::default(),
        }
    }

    fn request() -> LayoutRootRequestOf<S> {
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(200.0))))
            .expect("composition viewport is finite")
    }

    fn apply_cache_entry(
        retained: &mut Fri07C01CompositionRetained<S>,
        entry: &LayoutCacheStoreEntryOf<u32, S>,
    ) {
        retained
            .caches
            .entry(entry.node())
            .or_default()
            .store_with_context(entry.input(), entry.context(), entry.output());
    }
}

impl<S: LayoutScalar> Traverse for Fri07C01CompositionTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C01CompositionTree<S> {
    type MeasureError = Fri07C01CompositionMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        matches!(node, 2 | 4)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.measurement_requests.borrow_mut().push((node, input));
        if self.measure_mode.get() == Fri07C01CompositionMeasureMode::Failure && node == 4 {
            return Some(Err(Fri07C01CompositionMeasureError::ProviderFailure));
        }

        let width = match input.available_content_size().width {
            MeasurementAvailableOf::MinContent => S::from_f64(40.4),
            MeasurementAvailableOf::MaxContent => S::from_f64(100.4),
            MeasurementAvailableOf::Definite(width) => width.get(),
        };
        Some(Ok(Size::new(width, S::from_f64(50.4))))
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<NodeOutputOf<S>> {
        self.retained.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri07C01CompositionTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = Fri07C01CompositionRetained<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut prepared = self.retained.clone();
        for node in batch.invalidated_nodes() {
            prepared.unrounded.remove(node);
            prepared.final_outputs.remove(node);
            prepared.caches.remove(node);
        }
        for entry in batch.unrounded_entries() {
            prepared.unrounded.insert(entry.node(), entry.output());
        }
        for entry in batch.final_entries() {
            prepared.final_outputs.insert(entry.node(), entry.output());
        }
        for entry in batch.cache_clear_entries() {
            prepared.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            Self::apply_cache_entry(&mut prepared, entry);
        }
        Ok(prepared)
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

fn fri07_c01_composition_output<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("composition layout must publish node {node}"))
        .output()
}

fn fri07_c01_composition_assert_near<S: LayoutScalar>(actual: S, expected: f64, context: &str) {
    let difference = (actual.to_f64() - expected).abs();
    assert!(
        difference <= 0.000_02,
        "{context}: expected {expected}, got {}",
        actual.to_f64()
    );
}

fn fri07_c01_composition_geometry<S: LayoutScalar>() -> Vec<f64> {
    let tree = Fri07C01CompositionTree::<S>::new();
    let batch = compute_layout(&tree, 1, Fri07C01CompositionTree::<S>::request())
        .expect("composed intrinsic and margin layout succeeds");
    let root = fri07_c01_composition_output(batch.unrounded_entries(), 1);
    let min = fri07_c01_composition_output(batch.unrounded_entries(), 2);
    let absolute = fri07_c01_composition_output(batch.unrounded_entries(), 3);
    let max = fri07_c01_composition_output(batch.unrounded_entries(), 4);
    let rounded_min = fri07_c01_composition_output(batch.final_entries(), 2);
    let rounded_max = fri07_c01_composition_output(batch.final_entries(), 4);

    assert_eq!(min.source_index, SourceIndex::new(0));
    assert_eq!(absolute.source_index, SourceIndex::new(1));
    assert_eq!(max.source_index, SourceIndex::new(2));
    fri07_c01_composition_assert_near(max.location.x, 0.0, "order-modified max x");
    fri07_c01_composition_assert_near(max.size.width, 100.4, "replaced max-content width");
    fri07_c01_composition_assert_near(min.location.x, 100.4, "order-modified min x");
    fri07_c01_composition_assert_near(min.size.width, 40.4, "non-replaced min-content width");
    for (name, output) in [("min", min), ("max", max)] {
        fri07_c01_composition_assert_near(output.location.y, 0.0, &format!("{name} y"));
        fri07_c01_composition_assert_near(output.size.height, 50.4, &format!("{name} height"));
        fri07_c01_composition_assert_near(output.margin.top, 0.0, &format!("{name} top"));
        fri07_c01_composition_assert_near(output.margin.bottom, -15.4, &format!("{name} bottom"));
    }
    fri07_c01_composition_assert_near(absolute.margin.left, 37.5, "absolute left margin");
    fri07_c01_composition_assert_near(absolute.margin.right, 37.5, "absolute right margin");
    fri07_c01_composition_assert_near(absolute.margin.top, 12.5, "absolute top margin");
    fri07_c01_composition_assert_near(absolute.margin.bottom, 12.5, "absolute bottom margin");
    fri07_c01_composition_assert_near(absolute.location.x, 47.5, "absolute x");
    fri07_c01_composition_assert_near(absolute.location.y, 12.5, "absolute y");

    let scroll = root
        .scroll_geometry
        .expect("composed auto overflow publishes scroll geometry");
    assert_eq!(scroll.used_overflow_x(), Overflow::Auto);
    assert_eq!(scroll.used_overflow_y(), Overflow::Auto);
    assert_eq!(scroll.scrollbar_size(), Size::splat(S::from_f64(5.0)));
    assert_eq!(
        scroll.scrollport().size(),
        Size::new(S::from_f64(125.0), S::from_f64(35.0))
    );
    fri07_c01_composition_assert_near(
        scroll.physical_range().x().maximum(),
        15.8,
        "settled horizontal scroll range",
    );
    fri07_c01_composition_assert_near(
        scroll.physical_range().y().maximum(),
        15.4,
        "settled vertical scroll range",
    );
    assert_eq!(rounded_max.location.x, S::ZERO);
    assert_eq!(
        rounded_max.size,
        Size::new(S::from_f64(100.0), S::from_f64(50.0))
    );
    assert_eq!(rounded_min.location.x, S::from_f64(100.0));
    assert_eq!(
        rounded_min.size,
        Size::new(S::from_f64(41.0), S::from_f64(50.0))
    );

    vec![
        min.location.x.to_f64(),
        min.size.width.to_f64(),
        min.margin.bottom.to_f64(),
        max.location.x.to_f64(),
        max.size.width.to_f64(),
        max.margin.bottom.to_f64(),
        absolute.location.x.to_f64(),
        absolute.location.y.to_f64(),
        scroll.physical_range().x().maximum().to_f64(),
        scroll.physical_range().y().maximum().to_f64(),
    ]
}

#[test]
fn fri07_c01_composition_order_replaced_overflow_absolute_rounding_and_scalars_agree() {
    let f32_geometry = fri07_c01_composition_geometry::<f32>();
    let f64_geometry = fri07_c01_composition_geometry::<f64>();

    assert_eq!(f32_geometry.len(), f64_geometry.len());
    for (index, (f32_value, f64_value)) in f32_geometry.into_iter().zip(f64_geometry).enumerate() {
        assert!(
            (f32_value - f64_value).abs() <= 0.000_02,
            "composition scalar lane mismatch at field {index}: {f32_value} versus {f64_value}"
        );
    }
}

fn assert_fri07_c01_composition_replaced_intrinsic_sizing<S: LayoutScalar>() {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let auto_px = |value| LengthAutoOf::px(S::from_f64(value));
    for (replaced, expected_width) in [(true, 50.0), (false, 60.0)] {
        let tree = Fri07C01IntrinsicTree::new(Fri07C01IntrinsicMeasureMode::Values)
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(px(50.0), px(20.0)),
                    align_items: Some(AlignItems::Stretch),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    item_is_replaced: replaced,
                    aspect_ratio: AspectRatioOf::new(S::from_f64(3.0)),
                    flex_basis: FlexBasisOf::MAX_CONTENT,
                    flex_grow: FlexGrowOf::try_new(S::ZERO).expect("zero is a valid flex grow"),
                    flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    position: Position::Absolute,
                    inset: Edges {
                        top: auto_px(0.0),
                        left: auto_px(0.0),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    size: Size::new(px(10.0), px(5.0)),
                    margin: Edges::all(LengthAutoOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .measured(2);
        let batch = compute_layout(&tree, 1, Fri07C01CompositionTree::<S>::request())
            .expect("replaced intrinsic composition layout succeeds");
        let intrinsic = fri07_c01_composition_output(batch.unrounded_entries(), 2);
        let absolute = fri07_c01_composition_output(batch.unrounded_entries(), 3);

        fri07_c01_composition_assert_near(
            intrinsic.size.width,
            expected_width,
            if replaced {
                "replaced intrinsic automatic minimum"
            } else {
                "non-replaced intrinsic automatic minimum"
            },
        );
        fri07_c01_composition_assert_near(intrinsic.size.height, 20.0, "intrinsic cross stretch");
        assert_eq!(intrinsic.source_index, SourceIndex::new(0));
        assert_eq!(absolute.source_index, SourceIndex::new(1));
        assert_eq!(absolute.margin, Edges::ZERO);
        assert_eq!(absolute.location, Point::ZERO);
        assert!(
            tree.leaf_requests(2).iter().any(|input| {
                input.available_content_size().width == MeasurementAvailableOf::MAX_CONTENT
            }),
            "max-content basis must reach the provider for replaced={replaced}"
        );
    }
}

#[test]
fn fri07_c01_composition_intrinsic_replaced_and_non_replaced_sizing_remain_distinct() {
    assert_fri07_c01_composition_replaced_intrinsic_sizing::<f32>();
    assert_fri07_c01_composition_replaced_intrinsic_sizing::<f64>();
}

fn assert_fri07_c01_composition_cache_and_atomicity<S: LayoutScalar>() {
    let mut tree = Fri07C01CompositionTree::<S>::new();
    let request = Fri07C01CompositionTree::<S>::request();
    let cold = compute_layout(&tree, 1, request).expect("cold composition layout succeeds");
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();
    let cold_measurements = tree.measurement_requests.borrow().len();
    assert!(
        cold_measurements > 0,
        "cold layout must invoke intrinsic measurement"
    );
    cold.apply_to(&mut tree)
        .expect("infallible composition batch commit succeeds");

    tree.cache_queries.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm composition layout succeeds");
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert!(
        tree.cache_queries.borrow().iter().any(|(_, hit)| *hit),
        "warm composition layout must reuse a committed cache entry"
    );
    assert!(
        tree.measurement_requests.borrow()[cold_measurements..]
            .iter()
            .all(|(node, input)| match node {
                2 => input.available_content_size().width == MeasurementAvailableOf::MIN_CONTENT,
                4 => matches!(
                    input.available_content_size().width,
                    MeasurementAvailableOf::MinContent | MeasurementAvailableOf::MaxContent
                ),
                _ => false,
            }),
        "warm recomputation must preserve intrinsic measurement constraints"
    );

    tree.measure_mode
        .set(Fri07C01CompositionMeasureMode::Failure);
    let retained_before_failure = tree.retained.clone();
    let error = compute_layout_invalidated(&tree, 1, request, &[4])
        .expect_err("invalidated intrinsic provider failure returns no batch");
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(4));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(Fri07C01CompositionMeasureError::ProviderFailure)
    ));
    assert_eq!(tree.retained, retained_before_failure);
}

#[test]
fn fri07_c01_composition_cache_cold_warm_and_failed_measurement_are_atomic() {
    assert_fri07_c01_composition_cache_and_atomicity::<f32>();
    assert_fri07_c01_composition_cache_and_atomicity::<f64>();
}

fn fri07_c02_collapse_round_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("collapsed-flex public layout must publish node {node}"))
        .output()
}

fn fri07_c02_collapse_round_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(300.0))))
        .expect("collapsed-flex test viewport is finite")
}

fn fri07_c02_collapse_round_item<S: LayoutScalar>(
    main: f64,
    cross: f64,
    collapse: FlexItemCollapse,
) -> NodeInputOf<S> {
    NodeInputOf {
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(main)),
            PreferredSizeOf::px(S::from_f64(cross)),
        ),
        flex_item_collapse: collapse,
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
        ..NodeInputOf::default()
    }
}

fn assert_fri07_c02_collapse_round_single_line_and_gap<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                gap: Size::new(LengthOf::px(S::from_f64(9.0)), LengthOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            fri07_c02_collapse_round_item(40.0, 30.0, FlexItemCollapse::Collapsed),
        )
        .style(
            3,
            fri07_c02_collapse_round_item(20.0, 10.0, FlexItemCollapse::Normal),
        );

    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("single-line collapsed flex layout succeeds");
    let container = fri07_c02_collapse_round_output(&batch, 1);
    let collapsed = fri07_c02_collapse_round_output(&batch, 2);
    let normal = fri07_c02_collapse_round_output(&batch, 3);

    assert_eq!(
        container.size,
        Size::new(S::from_f64(100.0), S::from_f64(30.0))
    );
    assert_eq!(
        collapsed,
        NodeOutputOf::with_source_index(SourceIndex::new(0))
    );
    assert_eq!(
        normal.location,
        Point::ZERO,
        "no committed gap precedes the normal item"
    );
    assert_eq!(normal.size, Size::new(S::from_f64(20.0), S::from_f64(10.0)));

    let zero_main_tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                gap: Size::new(LengthOf::px(S::from_f64(9.0)), LengthOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            fri07_c02_collapse_round_item(0.0, 25.0, FlexItemCollapse::Collapsed),
        )
        .style(
            3,
            NodeInputOf {
                margin: Edges {
                    left: LengthAutoOf::AUTO,
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..fri07_c02_collapse_round_item(20.0, 10.0, FlexItemCollapse::Normal)
            },
        );
    let zero_main_batch = compute_layout(&zero_main_tree, 1, fri07_c02_collapse_round_request())
        .expect("zero-main collapsed flex layout succeeds");
    let auto_margin_normal = fri07_c02_collapse_round_output(&zero_main_batch, 3);
    assert_eq!(
        fri07_c02_collapse_round_output(&zero_main_batch, 1)
            .size
            .height,
        S::from_f64(25.0)
    );
    assert_eq!(auto_margin_normal.location.x, S::from_f64(80.0));
    assert_eq!(auto_margin_normal.margin.left, S::from_f64(80.0));
}

#[test]
fn fri07_c02_collapse_round_single_line_keeps_strut_and_suppresses_committed_gap() {
    assert_fri07_c02_collapse_round_single_line_and_gap::<f32>();
    assert_fri07_c02_collapse_round_single_line_and_gap::<f64>();
}

fn assert_fri07_c02_collapse_round_rewraps_by_zero_main_size_and_identity<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                flex_wrap: FlexWrap::Wrap,
                gap: Size::new(
                    LengthOf::px(S::from_f64(10.0)),
                    LengthOf::px(S::from_f64(4.0)),
                ),
                align_content: Some(AlignContent::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            fri07_c02_collapse_round_item(60.0, 10.0, FlexItemCollapse::Normal),
        )
        .style(
            3,
            fri07_c02_collapse_round_item(50.0, 30.0, FlexItemCollapse::Collapsed),
        )
        .style(
            4,
            fri07_c02_collapse_round_item(30.0, 10.0, FlexItemCollapse::Normal),
        );

    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("wrapped collapsed flex layout succeeds");
    let container = fri07_c02_collapse_round_output(&batch, 1);
    let first = fri07_c02_collapse_round_output(&batch, 2);
    let second = fri07_c02_collapse_round_output(&batch, 4);

    assert_eq!(container.size.height, S::from_f64(44.0));
    assert_eq!(first.location, Point::ZERO);
    assert_eq!(second.location, Point::new(S::ZERO, S::from_f64(34.0)));
    assert_eq!(
        second.source_index,
        SourceIndex::new(2),
        "rewrapping retains raw source association"
    );
}

#[test]
fn fri07_c02_collapse_round_zero_main_rewrap_keeps_collection_gaps_and_identity_strut() {
    assert_fri07_c02_collapse_round_rewraps_by_zero_main_size_and_identity::<f32>();
    assert_fri07_c02_collapse_round_rewraps_by_zero_main_size_and_identity::<f64>();
}

fn fri07_c02_collapse_round_margin_case<S: LayoutScalar>(
    collapse_main_margins: (LengthAutoOf<S>, LengthAutoOf<S>),
) -> NodeOutputOf<S> {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                flex_wrap: FlexWrap::Wrap,
                gap: Size::new(LengthOf::px(S::from_f64(5.0)), LengthOf::ZERO),
                align_content: Some(AlignContent::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            fri07_c02_collapse_round_item(60.0, 10.0, FlexItemCollapse::Normal),
        )
        .style(
            3,
            NodeInputOf {
                margin: Edges {
                    left: collapse_main_margins.0,
                    right: collapse_main_margins.1,
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..fri07_c02_collapse_round_item(70.0, 30.0, FlexItemCollapse::Collapsed)
            },
        )
        .style(
            4,
            fri07_c02_collapse_round_item(20.0, 10.0, FlexItemCollapse::Normal),
        );
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("collapsed main-margin layout succeeds");
    fri07_c02_collapse_round_output(&batch, 4)
}

fn assert_fri07_c02_collapse_round_fixed_and_auto_main_margins<S: LayoutScalar>() {
    let fixed = fri07_c02_collapse_round_margin_case((
        LengthAutoOf::px(S::from_f64(15.0)),
        LengthAutoOf::px(S::from_f64(15.0)),
    ));
    let automatic =
        fri07_c02_collapse_round_margin_case::<S>((LengthAutoOf::AUTO, LengthAutoOf::AUTO));

    assert_eq!(fixed.location.y, S::from_f64(30.0));
    assert_eq!(automatic.location.y, S::ZERO);
    assert_eq!(automatic.location.x, S::from_f64(65.0));
}

#[test]
fn fri07_c02_collapse_round_retains_fixed_and_zeroes_auto_main_margins_for_collection() {
    assert_fri07_c02_collapse_round_fixed_and_auto_main_margins::<f32>();
    assert_fri07_c02_collapse_round_fixed_and_auto_main_margins::<f64>();
}

fn assert_fri07_c02_collapse_round_largest_strut_and_all_collapsed<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                flex_wrap: FlexWrap::Wrap,
                align_content: Some(AlignContent::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            fri07_c02_collapse_round_item(70.0, 20.0, FlexItemCollapse::Collapsed),
        )
        .style(
            3,
            fri07_c02_collapse_round_item(70.0, 30.0, FlexItemCollapse::Collapsed),
        );
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("all-collapsed flex layout succeeds");

    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 1).size,
        Size::new(S::from_f64(100.0), S::from_f64(30.0)),
        "two identity struts rewrapped onto one all-collapsed line use their maximum"
    );
    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 2),
        NodeOutputOf::with_source_index(SourceIndex::new(0))
    );
    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 3),
        NodeOutputOf::with_source_index(SourceIndex::new(1))
    );
}

#[test]
fn fri07_c02_collapse_round_multiple_all_collapsed_struts_take_largest_not_sum() {
    assert_fri07_c02_collapse_round_largest_strut_and_all_collapsed::<f32>();
    assert_fri07_c02_collapse_round_largest_strut_and_all_collapsed::<f64>();
}

fn assert_fri07_c02_collapse_round_align_content_stretch_is_captured<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::px(S::from_f64(130.0)),
                ),
                flex_wrap: FlexWrap::Wrap,
                align_content: Some(AlignContent::Stretch),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            fri07_c02_collapse_round_item(80.0, 30.0, FlexItemCollapse::Normal),
        )
        .style(
            3,
            fri07_c02_collapse_round_item(70.0, 40.0, FlexItemCollapse::Collapsed),
        )
        .style(
            4,
            fri07_c02_collapse_round_item(80.0, 30.0, FlexItemCollapse::Normal),
        );
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("align-content stretch collapse layout succeeds");

    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 4).location.y,
        S::from_f64(75.0),
        "the collapsed identity carries its first-round 50px stretched line into the rewrapped first line"
    );
}

#[test]
fn fri07_c02_collapse_round_captures_align_content_stretch_before_rewrap() {
    assert_fri07_c02_collapse_round_align_content_stretch_is_captured::<f32>();
    assert_fri07_c02_collapse_round_align_content_stretch_is_captured::<f64>();
}

fn assert_fri07_c02_collapse_round_flow_matrix<S: LayoutScalar>() {
    let flow_axes = [
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
    ];
    for flow in flow_axes {
        for direction in [
            FlexDirection::Row,
            FlexDirection::RowReverse,
            FlexDirection::Column,
            FlexDirection::ColumnReverse,
        ] {
            for wrap in [FlexWrap::Wrap, FlexWrap::WrapReverse] {
                let axes = FlexAxes::new(flow, direction, wrap);
                let container_size =
                    axes.size_from_main_cross(S::from_f64(100.0), S::from_f64(30.0));
                let collapsed_size = axes.size_from_main_cross(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::px(S::from_f64(30.0)),
                );
                let normal_size = axes.size_from_main_cross(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                );
                let tree = PublicLayoutTreeOf::new()
                    .children(1, [2, 3])
                    .children(2, [])
                    .children(3, [])
                    .style(
                        1,
                        NodeInputOf {
                            display: Display::Flex,
                            writing_mode: flow.writing_mode(),
                            direction: flow.direction(),
                            flex_direction: direction,
                            flex_wrap: wrap,
                            size: container_size.map(PreferredSizeOf::px),
                            align_content: Some(AlignContent::FlexStart),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        2,
                        NodeInputOf {
                            size: collapsed_size,
                            flex_item_collapse: FlexItemCollapse::Collapsed,
                            flex_grow: FlexGrowOf::ZERO,
                            flex_shrink: FlexShrinkOf::try_new(S::ZERO)
                                .expect("zero is a valid flex shrink"),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        3,
                        NodeInputOf {
                            size: normal_size,
                            flex_grow: FlexGrowOf::ZERO,
                            flex_shrink: FlexShrinkOf::try_new(S::ZERO)
                                .expect("zero is a valid flex shrink"),
                            ..NodeInputOf::default()
                        },
                    );
                let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
                    .expect("flow-mapped collapse layout succeeds");
                let normal = fri07_c02_collapse_round_output(&batch, 3);

                assert_eq!(
                    fri07_c02_collapse_round_output(&batch, 1).size,
                    container_size
                );
                assert_eq!(
                    normal.size,
                    axes.size_from_main_cross(S::from_f64(20.0), S::from_f64(10.0))
                );
                assert_eq!(
                    axes.main_point(normal.location),
                    axes.main_position_from_start(
                        container_size,
                        S::ZERO,
                        S::ZERO,
                        S::from_f64(20.0),
                        S::ZERO,
                    ),
                );
                assert_eq!(
                    axes.cross_point(normal.location),
                    axes.cross_position_from_start(
                        container_size,
                        S::ZERO,
                        S::ZERO,
                        S::from_f64(10.0),
                        S::ZERO,
                    ),
                );
            }
        }
    }
}

#[test]
fn fri07_c02_collapse_round_all_flows_directions_reversals_use_existing_flex_axes() {
    assert_fri07_c02_collapse_round_flow_matrix::<f32>();
    assert_fri07_c02_collapse_round_flow_matrix::<f64>();
}

fn assert_fri07_c02_collapse_round_intrinsic_replaced_controls<S: LayoutScalar>() {
    for item_is_replaced in [false, true] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    item_is_replaced,
                    flex_item_collapse: FlexItemCollapse::Collapsed,
                    flex_basis: FlexBasisOf::MIN_CONTENT,
                    min_size: Size::new(
                        MinSizeOf::px(S::from_f64(70.0)),
                        MinSizeOf::px(S::from_f64(24.0)),
                    ),
                    max_size: Size::new(
                        MaxSizeOf::px(S::from_f64(80.0)),
                        MaxSizeOf::px(S::from_f64(24.0)),
                    ),
                    flex_grow: FlexGrowOf::ZERO,
                    flex_shrink: FlexShrinkOf::try_new(S::ZERO)
                        .expect("zero is a valid flex shrink"),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                fri07_c02_collapse_round_item(20.0, 10.0, FlexItemCollapse::Normal),
            )
            .measure(2, Size::new(S::from_f64(75.0), S::from_f64(24.0)));
        let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
            .expect("intrinsic replaced collapse layout succeeds");

        assert_eq!(
            fri07_c02_collapse_round_output(&batch, 1).size.height,
            S::from_f64(24.0)
        );
        assert_eq!(
            fri07_c02_collapse_round_output(&batch, 3).location,
            Point::ZERO
        );
    }
}

#[test]
fn fri07_c02_collapse_round_intrinsic_min_max_and_replacedness_preserve_strut_only() {
    assert_fri07_c02_collapse_round_intrinsic_replaced_controls::<f32>();
    assert_fri07_c02_collapse_round_intrinsic_replaced_controls::<f64>();
}

#[derive(Clone, Debug)]
struct Fri07C02CollapseRoundBaselineTree<S: LayoutScalar> {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInputOf<S>>,
}

impl<S: LayoutScalar> Traverse for Fri07C02CollapseRoundBaselineTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C02CollapseRoundBaselineTree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.styles[&node].clone())
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        _context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let _ = input;
        (node == 2).then(|| {
            ComputeOutputOf::from_sizes_and_first_baselines(
                Size::new(S::from_f64(20.0), S::from_f64(10.0)),
                Size::ZERO,
                Point::new(None, Some(S::from_f64(30.0))),
            )
        })
    }
}

fn assert_fri07_c02_collapse_round_baseline_line_size_is_strut<S: LayoutScalar>() {
    let tree = Fri07C02CollapseRoundBaselineTree {
        children: HashMap::from([(1, vec![2, 3]), (2, vec![]), (3, vec![])]),
        styles: HashMap::from([
            (
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    align_items: Some(AlignItems::Baseline),
                    ..NodeInputOf::default()
                },
            ),
            (
                2,
                fri07_c02_collapse_round_item(20.0, 10.0, FlexItemCollapse::Collapsed),
            ),
            (
                3,
                fri07_c02_collapse_round_item(20.0, 20.0, FlexItemCollapse::Normal),
            ),
        ]),
    };
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("baseline collapse layout succeeds");

    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 1).size.height,
        S::from_f64(30.0),
        "the strut is the baseline-expanded line size, not the collapsed item's 10px size"
    );
    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 3).location.y,
        S::ZERO
    );
}

#[test]
fn fri07_c02_collapse_round_baseline_expansion_is_captured_as_used_line_size() {
    assert_fri07_c02_collapse_round_baseline_line_size_is_strut::<f32>();
    assert_fri07_c02_collapse_round_baseline_line_size_is_strut::<f64>();
}

#[derive(Clone, Debug)]
struct Fri07C02CollapseRoundLedgerTree<S: LayoutScalar> {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInputOf<S>>,
    requests: RefCell<Vec<(u32, LeafMeasureInputOf<S>)>>,
}

impl<S: LayoutScalar> Traverse for Fri07C02CollapseRoundLedgerTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C02CollapseRoundLedgerTree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.styles[&node].clone())
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        node == 3
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        (node == 3).then(|| {
            self.requests.borrow_mut().push((node, input));
            Ok(Size::new(S::from_f64(20.0), S::from_f64(17.0)))
        })
    }
}

fn assert_fri07_c02_collapse_round_ledger_is_finite<S: LayoutScalar>() {
    let tree = Fri07C02CollapseRoundLedgerTree {
        children: HashMap::from([(1, vec![2, 3]), (2, vec![]), (3, vec![])]),
        styles: HashMap::from([
            (
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            ),
            (
                2,
                fri07_c02_collapse_round_item(40.0, 30.0, FlexItemCollapse::Collapsed),
            ),
            (
                3,
                NodeInputOf {
                    flex_basis: FlexBasisOf::px(S::from_f64(20.0)),
                    flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow"),
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    ..NodeInputOf::default()
                },
            ),
        ]),
        requests: RefCell::new(Vec::new()),
    };
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("measurement-ledger collapse layout succeeds");
    let requests = tree.requests.borrow();

    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 1).size.height,
        S::from_f64(30.0)
    );
    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 3).size.width,
        S::from_f64(100.0),
        "the normal item alone receives the existing second-round flex growth"
    );
    let unresolved_cross_requests = requests
        .iter()
        .filter_map(|(_, input)| {
            let known = input.known_content_size();
            known.height.is_none().then_some(known.width).flatten()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        unresolved_cross_requests,
        [
            S::from_f64(60.0),
            S::from_f64(60.0),
            S::from_f64(100.0),
            S::from_f64(100.0),
        ],
        "the ordinary cross-size resolver's two measurements occur at the 60px first-round target and 100px second-round target only, with no third group"
    );
}

#[test]
fn fri07_c02_collapse_round_measurement_ledger_proves_two_rounds_and_no_third() {
    assert_fri07_c02_collapse_round_ledger_is_finite::<f32>();
    assert_fri07_c02_collapse_round_ledger_is_finite::<f64>();
}

fn fri07_c02_collapsed_output_subtree<S: LayoutScalar>(
    with_collapsed_descendants: bool,
) -> PublicLayoutTreeOf<S> {
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let length = |value| LengthOf::px(S::from_f64(value));
    let auto_length = |value| LengthAutoOf::px(S::from_f64(value));
    let collapsed = NodeInputOf {
        flex_item_collapse: FlexItemCollapse::Collapsed,
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(preferred(30.0), preferred(25.0)),
        margin: Edges::new(
            auto_length(13.0),
            auto_length(17.0),
            auto_length(19.0),
            auto_length(23.0),
        ),
        padding: Edges::all(length(3.0)),
        border: Edges::all(length(2.0)),
        overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
        scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(4.0))
            .expect("collapsed scrollbar width is finite"),
        scroll_margin: ScrollMarginOf::try_new(
            S::from_f64(5.0),
            S::from_f64(6.0),
            S::from_f64(7.0),
            S::from_f64(8.0),
        )
        .expect("collapsed scroll margin is finite"),
        scroll_snap_align: ScrollSnapAlign::new(
            ScrollSnapAlignValue::Center,
            ScrollSnapAlignValue::End,
        ),
        scroll_snap_stop: ScrollSnapStop::Always,
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
        ..NodeInputOf::default()
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 6, 7])
        .children(3, [])
        .children(6, [])
        .children(7, [8])
        .children(8, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(preferred(120.0), preferred(60.0)),
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(3.0))
                    .expect("container scrollbar width is finite"),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .style(2, collapsed)
        .style(
            3,
            fri07_c02_collapse_round_item(20.0, 10.0, FlexItemCollapse::Normal),
        )
        .style(
            6,
            NodeInputOf {
                position: Position::Absolute,
                flex_item_collapse: FlexItemCollapse::Collapsed,
                inset: Edges {
                    top: auto_length(3.0),
                    left: auto_length(4.0),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                size: Size::new(preferred(10.0), preferred(8.0)),
                ..NodeInputOf::default()
            },
        )
        .style(
            7,
            NodeInputOf {
                display: Display::None,
                flex_item_collapse: FlexItemCollapse::Collapsed,
                size: Size::new(preferred(90.0), preferred(80.0)),
                ..NodeInputOf::default()
            },
        )
        .style(
            8,
            fri07_c02_collapse_round_item(70.0, 60.0, FlexItemCollapse::Normal),
        );

    if with_collapsed_descendants {
        tree = tree
            .children(2, [4, 5])
            .children(4, [])
            .children(5, [])
            .style(
                4,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                    overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                    scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(9.0))
                        .expect("nested scrollbar width is finite"),
                    scroll_margin: ScrollMarginOf::try_new(
                        S::from_f64(31.0),
                        S::from_f64(32.0),
                        S::from_f64(33.0),
                        S::from_f64(34.0),
                    )
                    .expect("nested scroll margin is finite"),
                    scroll_snap_align: ScrollSnapAlign::new(
                        ScrollSnapAlignValue::End,
                        ScrollSnapAlignValue::Center,
                    ),
                    scroll_snap_stop: ScrollSnapStop::Always,
                    ..NodeInputOf::default()
                },
            )
            .style(
                5,
                NodeInputOf {
                    position: Position::Absolute,
                    inset: Edges {
                        right: auto_length(-200.0),
                        bottom: auto_length(-150.0),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    size: Size::new(preferred(300.0), preferred(250.0)),
                    overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                    ..NodeInputOf::default()
                },
            )
            .measure(4, Size::new(S::from_f64(400.0), S::from_f64(350.0)));
    } else {
        tree = tree.children(2, []);
    }

    tree
}

fn assert_fri07_c02_collapsed_output_zero_and_hidden_subtree<S: LayoutScalar>() {
    let hostile = compute_layout(
        &fri07_c02_collapsed_output_subtree::<S>(true),
        1,
        fri07_c02_collapse_round_request(),
    )
    .expect("collapsed subtree layout succeeds");
    let leaf = compute_layout(
        &fri07_c02_collapsed_output_subtree::<S>(false),
        1,
        fri07_c02_collapse_round_request(),
    )
    .expect("collapsed leaf control layout succeeds");

    for entries in [hostile.unrounded_entries(), hostile.final_entries()] {
        for (node, source_index) in [(2, 0), (4, 0), (5, 1)] {
            assert_eq!(
                fri07_c01_composition_output(entries, node),
                NodeOutputOf::with_source_index(SourceIndex::new(source_index)),
                "collapsed node and descendants publish exact hidden output"
            );
        }
    }

    for node in [1, 3, 6, 7, 8] {
        assert_eq!(
            fri07_c02_collapse_round_output(&hostile, node),
            fri07_c02_collapse_round_output(&leaf, node),
            "collapsed nested content cannot change retained output for node {node}"
        );
    }

    let collapsed = fri07_c02_collapse_round_output(&hostile, 2);
    assert_eq!(collapsed.source_index, SourceIndex::new(0));
    assert_eq!(collapsed.location, Point::ZERO);
    assert_eq!(collapsed.size, Size::ZERO);
    assert_eq!(collapsed.content_size, Size::ZERO);
    assert_eq!(collapsed.border, Edges::ZERO);
    assert_eq!(collapsed.padding, Edges::ZERO);
    assert_eq!(collapsed.margin, Edges::ZERO);
    assert_eq!(collapsed.scroll_geometry, None);

    let normal = fri07_c02_collapse_round_output(&hostile, 3);
    assert_eq!(normal.source_index, SourceIndex::new(1));
    assert_eq!(normal.location, Point::ZERO);
    assert_eq!(normal.size, Size::new(S::from_f64(20.0), S::from_f64(10.0)));

    let absolute = fri07_c02_collapse_round_output(&hostile, 6);
    assert_eq!(absolute.source_index, SourceIndex::new(2));
    assert_eq!(
        absolute.location,
        Point::new(S::from_f64(4.0), S::from_f64(3.0))
    );
    assert_eq!(
        absolute.size,
        Size::new(S::from_f64(10.0), S::from_f64(8.0))
    );
    assert!(absolute.scroll_geometry.is_some());
    assert_eq!(
        fri07_c02_collapse_round_output(&hostile, 7),
        NodeOutputOf::with_source_index(SourceIndex::new(3)),
        "display-none keeps its existing source-indexed hidden owner"
    );
}

#[test]
fn fri07_c02_collapsed_output_is_exact_zero_and_hides_normal_and_absolute_descendants() {
    assert_fri07_c02_collapsed_output_zero_and_hidden_subtree::<f32>();
    assert_fri07_c02_collapsed_output_zero_and_hidden_subtree::<f64>();
}

fn assert_fri07_c02_collapsed_output_baseline_is_private<S: LayoutScalar>() {
    let tree = Fri07C02CollapseRoundBaselineTree {
        children: HashMap::from([(1, vec![2, 3]), (2, vec![]), (3, vec![])]),
        styles: HashMap::from([
            (
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::px(S::from_f64(40.0)),
                    ),
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    align_items: Some(AlignItems::Baseline),
                    ..NodeInputOf::default()
                },
            ),
            (
                2,
                NodeInputOf {
                    margin: Edges::all(LengthAutoOf::px(S::from_f64(12.0))),
                    overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                    scroll_snap_align: ScrollSnapAlign::new(
                        ScrollSnapAlignValue::Center,
                        ScrollSnapAlignValue::End,
                    ),
                    scroll_snap_stop: ScrollSnapStop::Always,
                    ..fri07_c02_collapse_round_item(20.0, 10.0, FlexItemCollapse::Collapsed)
                },
            ),
            (
                3,
                fri07_c02_collapse_round_item(20.0, 20.0, FlexItemCollapse::Normal),
            ),
        ]),
    };
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("baseline-bearing collapsed layout succeeds");

    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 2),
        NodeOutputOf::with_source_index(SourceIndex::new(0)),
        "the first-round baseline and margins remain private strut inputs"
    );
    assert_eq!(
        fri07_c02_collapse_round_output(&batch, 3).location,
        Point::ZERO,
        "the collapsed baseline cannot become a final alignment subject"
    );
}

#[test]
fn fri07_c02_collapsed_output_carries_no_baseline_margin_overflow_or_scroll_target() {
    assert_fri07_c02_collapsed_output_baseline_is_private::<f32>();
    assert_fri07_c02_collapsed_output_baseline_is_private::<f64>();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C02CollapsedOutputMeasureMode {
    Values,
    FailFirstRound,
    FailSecondRound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C02CollapsedOutputMeasureError {
    FirstRound,
    SecondRound,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Fri07C02CollapsedOutputRetained<S: LayoutScalar> {
    unrounded: HashMap<u32, NodeOutputOf<S>>,
    final_outputs: HashMap<u32, NodeOutputOf<S>>,
    caches: HashMap<u32, CacheOf<S>>,
}

#[derive(Clone, Debug)]
struct Fri07C02CollapsedOutputTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    measure_mode: Cell<Fri07C02CollapsedOutputMeasureMode>,
    measurement_requests: RefCell<Vec<LeafMeasureInputOf<S>>>,
    cache_queries: RefCell<Vec<(u32, bool)>>,
    retained: Fri07C02CollapsedOutputRetained<S>,
}

impl<S: LayoutScalar> Fri07C02CollapsedOutputTree<S> {
    fn new() -> Self {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                fri07_c02_collapse_round_item(40.0, 30.0, FlexItemCollapse::Collapsed),
            )
            .style(
                3,
                NodeInputOf {
                    flex_basis: FlexBasisOf::px(S::from_f64(20.0)),
                    flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow"),
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    ..NodeInputOf::default()
                },
            );
        Self {
            tree,
            measure_mode: Cell::new(Fri07C02CollapsedOutputMeasureMode::Values),
            measurement_requests: RefCell::new(Vec::new()),
            cache_queries: RefCell::new(Vec::new()),
            retained: Fri07C02CollapsedOutputRetained::default(),
        }
    }

    fn apply_cache_entry(
        retained: &mut Fri07C02CollapsedOutputRetained<S>,
        entry: &LayoutCacheStoreEntryOf<u32, S>,
    ) {
        retained
            .caches
            .entry(entry.node())
            .or_default()
            .store_with_context(entry.input(), entry.context(), entry.output());
    }
}

impl<S: LayoutScalar> Traverse for Fri07C02CollapsedOutputTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C02CollapsedOutputTree<S> {
    type MeasureError = Fri07C02CollapsedOutputMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        node == 3
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if node != 3 {
            return None;
        }
        self.measurement_requests.borrow_mut().push(input);
        let known = input.known_content_size();
        let first_round = known.height.is_none() && known.width == Some(S::from_f64(60.0));
        let second_round = known.height.is_none() && known.width == Some(S::from_f64(100.0));
        match self.measure_mode.get() {
            Fri07C02CollapsedOutputMeasureMode::FailFirstRound if first_round => {
                Some(Err(Fri07C02CollapsedOutputMeasureError::FirstRound))
            }
            Fri07C02CollapsedOutputMeasureMode::FailSecondRound if second_round => {
                Some(Err(Fri07C02CollapsedOutputMeasureError::SecondRound))
            }
            _ => Some(Ok(Size::new(S::from_f64(20.0), S::from_f64(17.0)))),
        }
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<NodeOutputOf<S>> {
        self.retained.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri07C02CollapsedOutputTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = Fri07C02CollapsedOutputRetained<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut prepared = self.retained.clone();
        for node in batch.invalidated_nodes() {
            prepared.unrounded.remove(node);
            prepared.final_outputs.remove(node);
            prepared.caches.remove(node);
        }
        for entry in batch.unrounded_entries() {
            prepared.unrounded.insert(entry.node(), entry.output());
        }
        for entry in batch.final_entries() {
            prepared.final_outputs.insert(entry.node(), entry.output());
        }
        for entry in batch.cache_clear_entries() {
            prepared.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            Self::apply_cache_entry(&mut prepared, entry);
        }
        Ok(prepared)
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

fn assert_fri07_c02_collapsed_output_cache_and_failures_are_atomic<S: LayoutScalar>() {
    let mut tree = Fri07C02CollapsedOutputTree::<S>::new();
    let request = fri07_c02_collapse_round_request();
    let cold = compute_layout(&tree, 1, request).expect("cold collapsed layout succeeds");
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();
    assert_eq!(
        fri07_c02_collapse_round_output(&cold, 2),
        NodeOutputOf::with_source_index(SourceIndex::new(0))
    );
    cold.apply_to(&mut tree)
        .expect("infallible collapsed batch commit succeeds");

    tree.cache_queries.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm collapsed layout succeeds");
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert!(
        tree.cache_queries
            .borrow()
            .iter()
            .any(|(node, hit)| *node == 3 && *hit),
        "warm collapsed layout reuses a committed normal-item measurement"
    );

    for (mode, expected_error, expected_failed_width) in [
        (
            Fri07C02CollapsedOutputMeasureMode::FailFirstRound,
            Fri07C02CollapsedOutputMeasureError::FirstRound,
            S::from_f64(60.0),
        ),
        (
            Fri07C02CollapsedOutputMeasureMode::FailSecondRound,
            Fri07C02CollapsedOutputMeasureError::SecondRound,
            S::from_f64(100.0),
        ),
    ] {
        tree.measure_mode.set(mode);
        tree.measurement_requests.borrow_mut().clear();
        let retained_before_failure = tree.retained.clone();
        let error = compute_layout_invalidated(&tree, 1, request, &[3])
            .expect_err("failed collapse round returns no partial batch");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(3));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement(error) if *error == expected_error
        ));
        assert_eq!(tree.retained, retained_before_failure);
        assert!(
            tree.measurement_requests.borrow().iter().any(|input| {
                let known = input.known_content_size();
                known.height.is_none() && known.width == Some(expected_failed_width)
            }),
            "the requested collapse round reached its failing measurement"
        );
    }
}

#[test]
fn fri07_c02_collapsed_output_cache_cold_warm_and_both_round_failures_are_atomic() {
    assert_fri07_c02_collapsed_output_cache_and_failures_are_atomic::<f32>();
    assert_fri07_c02_collapsed_output_cache_and_failures_are_atomic::<f64>();
}

fn fri07_c02_composition_atomic_tree<S: LayoutScalar>() -> Fri07C02CollapsedOutputTree<S> {
    let mut composition = Fri07C02CollapsedOutputTree::<S>::new();
    let mut collapsed = composition.tree.node_input(2).clone();
    collapsed.item_order = ItemOrder::new(7);
    collapsed.item_is_replaced = true;
    collapsed.overflow = computed_overflow(Overflow::Scroll, Overflow::Scroll);
    collapsed.scrollbar_width =
        ScrollbarWidthOf::try_new(S::from_f64(3.0)).expect("composition scrollbar width is finite");
    let mut normal = composition.tree.node_input(3).clone();
    normal.item_order = ItemOrder::new(-4);
    normal.margin = Edges {
        top: LengthAutoOf::AUTO,
        bottom: LengthAutoOf::AUTO,
        ..Edges::all(LengthAutoOf::ZERO)
    };
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let auto_px = |value| LengthAutoOf::px(S::from_f64(value));

    composition.tree = core::mem::take(&mut composition.tree)
        .children(1, [2, 3, 4, 5])
        .children(4, [])
        .children(5, [])
        .style(2, collapsed)
        .style(3, normal)
        .style(
            4,
            NodeInputOf {
                position: Position::Absolute,
                flex_item_collapse: FlexItemCollapse::Collapsed,
                inset: Edges {
                    top: auto_px(2.5),
                    left: auto_px(4.5),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                size: Size::new(px(11.5), px(7.5)),
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                display: Display::None,
                flex_item_collapse: FlexItemCollapse::Collapsed,
                size: Size::new(px(90.0), px(80.0)),
                ..NodeInputOf::default()
            },
        );
    composition
}

fn assert_fri07_c02_composition_cache_failures_and_siblings<S: LayoutScalar>() {
    let mut tree = fri07_c02_composition_atomic_tree::<S>();
    let request = fri07_c02_collapse_round_request();
    let cold = compute_layout(&tree, 1, request).expect("cold collapse composition succeeds");
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();

    assert_eq!(
        fri07_c02_collapse_round_output(&cold, 2),
        NodeOutputOf::with_source_index(SourceIndex::new(0)),
        "order-modified collapsed output retains raw source identity"
    );
    assert_eq!(
        fri07_c02_collapse_round_output(&cold, 3).source_index,
        SourceIndex::new(1)
    );
    assert_eq!(
        fri07_c02_collapse_round_output(&cold, 4).source_index,
        SourceIndex::new(2),
        "absolute sibling keeps its source association and independent owner"
    );
    assert_eq!(
        fri07_c02_collapse_round_output(&cold, 5),
        NodeOutputOf::with_source_index(SourceIndex::new(3)),
        "display-none sibling keeps its existing hidden owner"
    );
    cold.apply_to(&mut tree)
        .expect("infallible composed collapse batch commit succeeds");

    tree.cache_queries.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm collapse composition succeeds");
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert!(
        tree.cache_queries.borrow().iter().any(|(_, hit)| *hit),
        "warm collapse composition reuses committed cache state"
    );

    for (mode, expected_error, expected_failed_width) in [
        (
            Fri07C02CollapsedOutputMeasureMode::FailFirstRound,
            Fri07C02CollapsedOutputMeasureError::FirstRound,
            S::from_f64(60.0),
        ),
        (
            Fri07C02CollapsedOutputMeasureMode::FailSecondRound,
            Fri07C02CollapsedOutputMeasureError::SecondRound,
            S::from_f64(100.0),
        ),
    ] {
        tree.measure_mode.set(mode);
        tree.measurement_requests.borrow_mut().clear();
        let retained_before_failure = tree.retained.clone();
        let error = compute_layout_invalidated(&tree, 1, request, &[3])
            .expect_err("failed composed collapse round returns no partial batch");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(3));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement(error) if *error == expected_error
        ));
        assert_eq!(tree.retained, retained_before_failure);
        assert!(
            tree.measurement_requests.borrow().iter().any(|input| {
                let known = input.known_content_size();
                known.height.is_none() && known.width == Some(expected_failed_width)
            }),
            "the requested composed collapse round reached its failing provider"
        );
    }
}

#[test]
fn fri07_c02_composition_order_cache_siblings_and_both_round_failures_are_atomic() {
    assert_fri07_c02_composition_cache_failures_and_siblings::<f32>();
    assert_fri07_c02_composition_cache_failures_and_siblings::<f64>();
}

#[derive(Clone, Copy, Debug)]
struct Fri07C02CompositionCase {
    flow: FlowAxes,
    direction: FlexDirection,
    wrap: FlexWrap,
    collapse: FlexItemCollapse,
    order: ItemOrder,
    max_content_basis: bool,
    auto_margin_pattern: usize,
    replaced: bool,
    overflow: ComputedOverflow,
}

#[derive(Clone, Debug)]
struct Fri07C02CompositionMatrixTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    axes: FlexAxes,
    requests: RefCell<Vec<(u32, LeafMeasureInputOf<S>)>>,
}

impl<S: LayoutScalar> Traverse for Fri07C02CompositionMatrixTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C02CompositionMatrixTree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        matches!(node, 2 | 3)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if !matches!(node, 2 | 3) {
            return None;
        }
        self.requests.borrow_mut().push((node, input));
        let available_main = self.axes.main_size(input.available_content_size());
        let intrinsic_main = match available_main {
            MeasurementAvailableOf::MinContent => S::from_f64(24.25),
            MeasurementAvailableOf::MaxContent => S::from_f64(58.75),
            MeasurementAvailableOf::Definite(value) => value.get().min(S::from_f64(72.5)),
        };
        let (main, cross) = if node == 2 {
            (intrinsic_main, S::from_f64(31.25))
        } else {
            (S::from_f64(35.125), S::from_f64(18.75))
        };
        Some(Ok(self.axes.size_from_main_cross(main, cross)))
    }
}

fn fri07_c02_composition_cases() -> Vec<Fri07C02CompositionCase> {
    let flows = [
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
    ];
    let directions = [
        FlexDirection::Row,
        FlexDirection::RowReverse,
        FlexDirection::Column,
        FlexDirection::ColumnReverse,
    ];
    let wraps = [FlexWrap::NoWrap, FlexWrap::Wrap, FlexWrap::WrapReverse];
    let orders = [ItemOrder::new(-3), ItemOrder::new(0), ItemOrder::new(4)];
    let overflows = [
        computed_overflow(Overflow::Visible, Overflow::Clip),
        computed_overflow(Overflow::Hidden, Overflow::Auto),
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        computed_overflow(Overflow::Scroll, Overflow::Hidden),
    ];

    (0..80)
        .map(|index| Fri07C02CompositionCase {
            flow: flows[(index / 8) % flows.len()],
            direction: directions[(index / 2) % directions.len()],
            wrap: wraps[index % wraps.len()],
            collapse: if index % 2 == 0 {
                FlexItemCollapse::Normal
            } else {
                FlexItemCollapse::Collapsed
            },
            order: orders[(index / 3) % orders.len()],
            max_content_basis: (index / 2) % 2 == 1,
            auto_margin_pattern: (index / 3) % 4,
            replaced: (index / 5) % 2 == 1,
            overflow: overflows[(index / 7) % overflows.len()],
        })
        .collect()
}

fn fri07_c02_composition_matrix_tree<S: LayoutScalar>(
    case: Fri07C02CompositionCase,
) -> Fri07C02CompositionMatrixTree<S> {
    let axes = FlexAxes::new(case.flow, case.direction, case.wrap);
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let length = |value| LengthOf::px(S::from_f64(value));
    let auto_length = |value| LengthAutoOf::px(S::from_f64(value));
    let container_size = axes.size_from_main_cross(preferred(101.5), preferred(63.75));
    let gap = axes.size_from_main_cross(length(4.25), length(3.5));
    let mut target_margin = Edges::all(LengthAutoOf::ZERO);
    if matches!(case.auto_margin_pattern, 1 | 3) {
        axes.set_main_start_edge(&mut target_margin, LengthAutoOf::AUTO);
    }
    if matches!(case.auto_margin_pattern, 2 | 3) {
        axes.set_main_end_edge(&mut target_margin, LengthAutoOf::AUTO);
    }
    let target_min = axes.size_from_main_cross(MinSizeOf::ZERO, MinSizeOf::ZERO);
    let target_max = axes.size_from_main_cross(
        MaxSizeOf::px(S::from_f64(70.25)),
        MaxSizeOf::px(S::from_f64(39.75)),
    );
    let peer_size = axes.size_from_main_cross(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
    let absolute_size = axes.size_from_main_cross(preferred(112.5), preferred(76.5));
    let target_basis = if case.max_content_basis {
        FlexBasisOf::MAX_CONTENT
    } else {
        FlexBasisOf::MIN_CONTENT
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4, 5])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: case.flow.writing_mode(),
                direction: case.flow.direction(),
                flex_direction: case.direction,
                flex_wrap: case.wrap,
                size: container_size,
                gap,
                align_content: Some(AlignContent::FlexStart),
                align_items: Some(AlignItems::FlexStart),
                overflow: case.overflow,
                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(3.25))
                    .expect("matrix scrollbar width is finite"),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                item_order: case.order,
                item_is_replaced: case.replaced,
                flex_item_collapse: case.collapse,
                size: peer_size.clone(),
                min_size: target_min,
                max_size: target_max,
                flex_basis: target_basis,
                flex_grow: FlexGrowOf::ZERO,
                flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
                margin: target_margin,
                overflow: case.overflow,
                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(2.25))
                    .expect("item scrollbar width is finite"),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                item_order: ItemOrder::new(-case.order.get()),
                size: peer_size,
                min_size: axes.size_from_main_cross(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::px(S::from_f64(35.125)),
                flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow"),
                flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                position: Position::Absolute,
                flex_item_collapse: FlexItemCollapse::Collapsed,
                inset: Edges {
                    top: auto_length(2.5),
                    left: auto_length(4.5),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                size: absolute_size,
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                display: Display::None,
                flex_item_collapse: FlexItemCollapse::Collapsed,
                size: axes.size_from_main_cross(preferred(90.0), preferred(80.0)),
                ..NodeInputOf::default()
            },
        );

    Fri07C02CompositionMatrixTree {
        tree,
        axes,
        requests: RefCell::new(Vec::new()),
    }
}

fn fri07_c02_composition_without_collapsed_scroll_source<S: LayoutScalar>(
    case: Fri07C02CompositionCase,
) -> Fri07C02CompositionMatrixTree<S> {
    let mut control = fri07_c02_composition_matrix_tree(case);
    let mut target = control.tree.node_input(2).clone();
    target.overflow = computed_overflow(Overflow::Clip, Overflow::Clip);
    target.scrollbar_width = ScrollbarWidthOf::ZERO;
    control.tree = core::mem::take(&mut control.tree).style(2, target);
    control
}

fn assert_fri07_c02_composition_finite_output<S: LayoutScalar>(
    output: NodeOutputOf<S>,
    context: &str,
) {
    let values = [
        output.location.x,
        output.location.y,
        output.size.width,
        output.size.height,
        output.content_size.width,
        output.content_size.height,
        output.border.top,
        output.border.right,
        output.border.bottom,
        output.border.left,
        output.padding.top,
        output.padding.right,
        output.padding.bottom,
        output.padding.left,
        output.margin.top,
        output.margin.right,
        output.margin.bottom,
        output.margin.left,
    ];
    assert!(
        values.into_iter().all(LayoutScalar::is_finite),
        "{context}: every published scalar is finite"
    );
    assert!(
        output.size.width >= S::ZERO
            && output.size.height >= S::ZERO
            && output.content_size.width >= S::ZERO
            && output.content_size.height >= S::ZERO,
        "{context}: published box sizes are non-negative"
    );
    if let Some(geometry) = output.scroll_geometry {
        for (name, rect) in [
            ("border", geometry.border_box()),
            ("padding", geometry.padding_box()),
            ("content", geometry.content_box()),
            ("scrollport", geometry.scrollport()),
            ("overflow", geometry.scrollable_overflow()),
        ] {
            assert!(
                rect.origin().x.is_finite()
                    && rect.origin().y.is_finite()
                    && rect.size().width.is_finite()
                    && rect.size().height.is_finite()
                    && rect.size().width >= S::ZERO
                    && rect.size().height >= S::ZERO,
                "{context}: {name} scroll box is finite and non-negative"
            );
        }
        let range = geometry.physical_range();
        assert!(
            range.x().minimum().is_finite()
                && range.x().maximum().is_finite()
                && range.y().minimum().is_finite()
                && range.y().maximum().is_finite(),
            "{context}: signed scroll range is finite"
        );
    }
}

fn fri07_c02_composition_case_geometry<S: LayoutScalar>(
    case: Fri07C02CompositionCase,
    case_index: usize,
) -> Vec<f64> {
    let tree = fri07_c02_composition_matrix_tree::<S>(case);
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .unwrap_or_else(|error| panic!("bounded composition case {case_index} failed: {error:?}"));
    let target = fri07_c02_collapse_round_output(&batch, 2);
    let peer = fri07_c02_collapse_round_output(&batch, 3);
    let absolute = fri07_c02_collapse_round_output(&batch, 4);
    let hidden = fri07_c02_collapse_round_output(&batch, 5);

    assert_eq!(target.source_index, SourceIndex::new(0));
    assert_eq!(peer.source_index, SourceIndex::new(1));
    assert_eq!(absolute.source_index, SourceIndex::new(2));
    assert_eq!(hidden, NodeOutputOf::with_source_index(SourceIndex::new(3)));
    if case.collapse == FlexItemCollapse::Collapsed {
        assert_eq!(
            target,
            NodeOutputOf::with_source_index(SourceIndex::new(0)),
            "bounded case {case_index}: collapsed output has no scroll contribution"
        );
    }

    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        for node in 1..=5 {
            assert_fri07_c02_composition_finite_output(
                fri07_c01_composition_output(entries, node),
                &format!("bounded case {case_index} node {node}"),
            );
        }
    }

    let expected_intrinsic = if case.max_content_basis {
        MeasurementAvailableOf::MAX_CONTENT
    } else {
        MeasurementAvailableOf::MIN_CONTENT
    };
    assert!(
        tree.requests.borrow().iter().any(|(node, input)| {
            *node == 2 && tree.axes.main_size(input.available_content_size()) == expected_intrinsic
        }),
        "bounded case {case_index}: selected intrinsic basis reaches the provider"
    );

    let root = fri07_c02_collapse_round_output(&batch, 1);
    let root_scroll = root
        .scroll_geometry
        .expect("performed composition root publishes scroll geometry");
    assert_eq!(root_scroll.used_overflow_x(), case.overflow.x());
    assert_eq!(root_scroll.used_overflow_y(), case.overflow.y());
    let scrollbar_size = root_scroll.scrollbar_size();
    if case.overflow.x() == Overflow::Scroll {
        assert!(
            scrollbar_size.height > S::ZERO,
            "bounded case {case_index}: horizontal overflow settles a scrollbar"
        );
    }
    if case.overflow.y() == Overflow::Scroll {
        assert!(
            scrollbar_size.width > S::ZERO,
            "bounded case {case_index}: vertical overflow settles a scrollbar"
        );
    }
    if case.collapse == FlexItemCollapse::Collapsed {
        let control = fri07_c02_composition_without_collapsed_scroll_source::<S>(case);
        let control_batch = compute_layout(&control, 1, fri07_c02_collapse_round_request())
            .unwrap_or_else(|error| {
                panic!("collapsed scroll control case {case_index} failed: {error:?}")
            });
        let control_scroll = fri07_c02_collapse_round_output(&control_batch, 1)
            .scroll_geometry
            .expect("collapsed scroll control root publishes scroll geometry");
        assert_eq!(
            root_scroll.scrollable_overflow(),
            control_scroll.scrollable_overflow(),
            "bounded case {case_index}: collapsed item overflow cannot enlarge root scroll geometry"
        );
        assert_eq!(
            root_scroll.physical_range(),
            control_scroll.physical_range(),
            "bounded case {case_index}: collapsed item overflow cannot change root scroll ranges"
        );
        assert_eq!(
            root_scroll.scrollbar_size(),
            control_scroll.scrollbar_size(),
            "bounded case {case_index}: collapsed item overflow cannot change settled scrollbar state"
        );
    }
    let rounded_target = fri07_c01_composition_output(batch.final_entries(), 2);
    let rounded_peer = fri07_c01_composition_output(batch.final_entries(), 3);
    vec![
        target.location.x.to_f64(),
        target.location.y.to_f64(),
        target.size.width.to_f64(),
        target.size.height.to_f64(),
        peer.location.x.to_f64(),
        peer.location.y.to_f64(),
        peer.size.width.to_f64(),
        peer.size.height.to_f64(),
        absolute.location.x.to_f64(),
        absolute.location.y.to_f64(),
        root_scroll.scrollbar_size().width.to_f64(),
        root_scroll.scrollbar_size().height.to_f64(),
        root_scroll.physical_range().x().minimum().to_f64(),
        root_scroll.physical_range().x().maximum().to_f64(),
        root_scroll.physical_range().y().minimum().to_f64(),
        root_scroll.physical_range().y().maximum().to_f64(),
        rounded_target.location.x.to_f64(),
        rounded_target.location.y.to_f64(),
        rounded_peer.location.x.to_f64(),
        rounded_peer.location.y.to_f64(),
    ]
}

#[test]
fn fri07_c02_composition_bounded_matrix_is_finite_source_stable_and_scalar_equivalent() {
    let cases = fri07_c02_composition_cases();
    assert_eq!(
        cases.len(),
        80,
        "the property matrix stays explicitly bounded"
    );
    let mut observed_auto_settlement = false;
    for (case_index, case) in cases.into_iter().enumerate() {
        let f32_geometry = fri07_c02_composition_case_geometry::<f32>(case, case_index);
        let f64_geometry = fri07_c02_composition_case_geometry::<f64>(case, case_index);
        assert_eq!(f32_geometry.len(), f64_geometry.len());
        observed_auto_settlement |= (case.overflow.x() == Overflow::Auto && f32_geometry[11] > 0.0)
            || (case.overflow.y() == Overflow::Auto && f32_geometry[10] > 0.0);
        for (field, (f32_value, f64_value)) in
            f32_geometry.into_iter().zip(f64_geometry).enumerate()
        {
            assert!(
                (f32_value - f64_value).abs() <= 0.000_02,
                "bounded case {case_index} scalar mismatch at field {field}: {f32_value} versus {f64_value}; case={case:?}"
            );
        }
    }
    assert!(
        observed_auto_settlement,
        "the bounded overflow pairs exercise automatic scrollbar settlement"
    );
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Fri07C02CompositionMeasurement<S: LayoutScalar> {
    node: u32,
    known_main: Option<S>,
    known_cross: Option<S>,
    available_main: MeasurementAvailableOf<S>,
    available_cross: MeasurementAvailableOf<S>,
}

fn fri07_c02_composition_measurement_trace<S: LayoutScalar>(
    case: Fri07C02CompositionCase,
) -> Vec<Fri07C02CompositionMeasurement<S>> {
    let tree = fri07_c02_composition_matrix_tree::<S>(case);
    compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("composed measurement trace layout succeeds");
    tree.requests
        .borrow()
        .iter()
        .map(|(node, input)| Fri07C02CompositionMeasurement {
            node: *node,
            known_main: tree.axes.main_size(input.known_content_size()),
            known_cross: tree.axes.cross_size(input.known_content_size()),
            available_main: tree.axes.main_size(input.available_content_size()),
            available_cross: tree.axes.cross_size(input.available_content_size()),
        })
        .collect()
}

fn fri07_c02_composition_definite<S: LayoutScalar>(value: f64) -> MeasurementAvailableOf<S> {
    MeasurementAvailableOf::definite(S::from_f64(value))
        .expect("composed measurement target is finite and non-negative")
}

fn assert_fri07_c02_composition_exact_round_sequences<S: LayoutScalar>() {
    let measurement =
        |node: u32,
         known_main: Option<f64>,
         available_main: MeasurementAvailableOf<S>,
         available_cross: MeasurementAvailableOf<S>| Fri07C02CompositionMeasurement {
            node,
            known_main: known_main.map(S::from_f64),
            known_cross: None,
            available_main,
            available_cross,
        };
    let definite = fri07_c02_composition_definite::<S>;
    let no_auto = Fri07C02CompositionCase {
        flow: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        direction: FlexDirection::Row,
        wrap: FlexWrap::Wrap,
        collapse: FlexItemCollapse::Collapsed,
        order: ItemOrder::new(-3),
        max_content_basis: false,
        auto_margin_pattern: 0,
        replaced: false,
        overflow: computed_overflow(Overflow::Visible, Overflow::Clip),
    };
    assert_eq!(
        fri07_c02_composition_measurement_trace(no_auto),
        [
            measurement(
                2,
                None,
                MeasurementAvailableOf::MIN_CONTENT,
                definite(39.75),
            ),
            measurement(3, None, definite(35.125), definite(63.75)),
            measurement(3, Some(73.0), definite(73.0), definite(63.75)),
            measurement(3, Some(101.5), definite(101.5), definite(63.75)),
            measurement(3, None, definite(101.5), definite(18.75)),
        ],
        "the composed non-auto case performs one intrinsic measurement, then the exact first-round, second-round, and final normal-item sequence"
    );

    let auto = Fri07C02CompositionCase {
        flow: FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        direction: FlexDirection::RowReverse,
        wrap: FlexWrap::Wrap,
        collapse: FlexItemCollapse::Collapsed,
        order: ItemOrder::new(4),
        max_content_basis: true,
        auto_margin_pattern: 2,
        replaced: false,
        overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
    };
    assert_eq!(
        fri07_c02_composition_measurement_trace(auto),
        [
            measurement(2, None, MeasurementAvailableOf::MAX_CONTENT, definite(37.5),),
            measurement(3, None, definite(35.125), definite(60.5)),
            measurement(3, Some(38.5), definite(38.5), definite(60.5)),
            measurement(3, Some(101.5), definite(101.5), definite(60.5)),
            measurement(3, None, definite(101.5), definite(18.75)),
            measurement(2, None, MeasurementAvailableOf::MAX_CONTENT, definite(37.5),),
            measurement(3, None, definite(35.125), definite(60.5)),
            measurement(3, Some(35.25), definite(35.25), definite(60.5)),
            measurement(3, Some(98.25), definite(98.25), definite(60.5)),
            measurement(3, None, definite(98.25), definite(18.75)),
        ],
        "the composed auto-overflow case performs exactly one bounded five-measurement sequence before and after the 3.25px settled scrollbar changes the main target"
    );
}

#[test]
fn fri07_c02_composition_round_sequence_is_exact_and_bounded_with_auto_overflow() {
    assert_fri07_c02_composition_exact_round_sequences::<f32>();
    assert_fri07_c02_composition_exact_round_sequences::<f64>();
}

fn fri07_c02_composition_dimension_outputs<S: LayoutScalar>(
    flow: FlowAxes,
    direction: FlexDirection,
    wrap: FlexWrap,
    container_main: f64,
    target_main: f64,
    orders: (i32, i32),
    auto_margin_pattern: usize,
) -> (NodeOutputOf<S>, NodeOutputOf<S>) {
    let axes = FlexAxes::new(flow, direction, wrap);
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let length = |value| LengthOf::px(S::from_f64(value));
    let mut target_margin = Edges::all(LengthAutoOf::ZERO);
    if matches!(auto_margin_pattern, 1 | 3) {
        axes.set_main_start_edge(&mut target_margin, LengthAutoOf::AUTO);
    }
    if matches!(auto_margin_pattern, 2 | 3) {
        axes.set_main_end_edge(&mut target_margin, LengthAutoOf::AUTO);
    }
    let item = |main, cross, order, margin| NodeInputOf {
        item_order: ItemOrder::new(order),
        size: axes.size_from_main_cross(preferred(main), preferred(cross)),
        margin,
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
        ..NodeInputOf::default()
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: flow.writing_mode(),
                direction: flow.direction(),
                flex_direction: direction,
                flex_wrap: wrap,
                size: axes.size_from_main_cross(preferred(container_main), preferred(40.0)),
                gap: axes.size_from_main_cross(length(5.0), length(4.0)),
                align_content: Some(AlignContent::FlexStart),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .style(2, item(target_main, 10.0, orders.0, target_margin))
        .style(
            3,
            item(30.0, 20.0, orders.1, Edges::all(LengthAutoOf::ZERO)),
        );
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("composition dimension control layout succeeds");
    (
        fri07_c02_collapse_round_output(&batch, 2),
        fri07_c02_collapse_round_output(&batch, 3),
    )
}

fn assert_fri07_c02_composition_rotated_dimensions_are_observable<S: LayoutScalar>() {
    let horizontal_ltr = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let (target_first, peer_second) = fri07_c02_composition_dimension_outputs::<S>(
        horizontal_ltr,
        FlexDirection::Row,
        FlexWrap::NoWrap,
        100.0,
        20.0,
        (-1, 1),
        0,
    );
    let (target_second, peer_first) = fri07_c02_composition_dimension_outputs::<S>(
        horizontal_ltr,
        FlexDirection::Row,
        FlexWrap::NoWrap,
        100.0,
        20.0,
        (1, -1),
        0,
    );
    assert_eq!(target_first.location, Point::ZERO);
    assert_eq!(peer_second.location.x, S::from_f64(25.0));
    assert_eq!(peer_first.location, Point::ZERO);
    assert_eq!(target_second.location.x, S::from_f64(35.0));
    assert_eq!(target_first.source_index, target_second.source_index);

    for (flow, direction, expected_target) in [
        (horizontal_ltr, FlexDirection::Row, Point::new(0.0, 0.0)),
        (
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
            FlexDirection::Row,
            Point::new(80.0, 0.0),
        ),
        (
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
            FlexDirection::RowReverse,
            Point::new(0.0, 0.0),
        ),
        (
            FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
            FlexDirection::Row,
            Point::new(30.0, 0.0),
        ),
        (
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
            FlexDirection::Row,
            Point::new(30.0, 80.0),
        ),
    ] {
        let (target, _) = fri07_c02_composition_dimension_outputs::<S>(
            flow,
            direction,
            FlexWrap::NoWrap,
            100.0,
            20.0,
            (-1, 1),
            0,
        );
        assert_eq!(
            target.location,
            expected_target.map(S::from_f64),
            "flow={flow:?} direction={direction:?}"
        );
    }

    let (_, no_wrap_peer) = fri07_c02_composition_dimension_outputs::<S>(
        horizontal_ltr,
        FlexDirection::Row,
        FlexWrap::NoWrap,
        70.0,
        60.0,
        (-1, 1),
        0,
    );
    let (_, wrap_peer) = fri07_c02_composition_dimension_outputs::<S>(
        horizontal_ltr,
        FlexDirection::Row,
        FlexWrap::Wrap,
        70.0,
        60.0,
        (-1, 1),
        0,
    );
    let (wrap_reverse_target, wrap_reverse_peer) = fri07_c02_composition_dimension_outputs::<S>(
        horizontal_ltr,
        FlexDirection::Row,
        FlexWrap::WrapReverse,
        70.0,
        60.0,
        (-1, 1),
        0,
    );
    assert_eq!(
        no_wrap_peer.location,
        Point::new(S::from_f64(65.0), S::ZERO)
    );
    assert_eq!(wrap_peer.location, Point::new(S::ZERO, S::from_f64(14.0)));
    assert_eq!(wrap_reverse_target.location.y, S::from_f64(30.0));
    assert_eq!(wrap_reverse_peer.location.y, S::from_f64(6.0));

    for (pattern, expected_start, expected_end, expected_x, expected_peer_x) in [
        (0, 0.0, 0.0, 0.0, 25.0),
        (1, 45.0, 0.0, 45.0, 70.0),
        (2, 0.0, 45.0, 0.0, 70.0),
        (3, 22.5, 22.5, 22.5, 70.0),
    ] {
        let (target, peer) = fri07_c02_composition_dimension_outputs::<S>(
            horizontal_ltr,
            FlexDirection::Row,
            FlexWrap::NoWrap,
            100.0,
            20.0,
            (-1, 1),
            pattern,
        );
        assert_eq!(target.margin.left, S::from_f64(expected_start));
        assert_eq!(target.margin.right, S::from_f64(expected_end));
        assert_eq!(target.location.x, S::from_f64(expected_x));
        assert_eq!(peer.location.x, S::from_f64(expected_peer_x));
    }
}

fn assert_fri07_c02_composition_replacedness_is_observable<S: LayoutScalar>() {
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let mut widths = Vec::new();
    for replaced in [true, false] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(preferred(60.0), preferred(20.0)),
                    align_items: Some(AlignItems::Stretch),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    item_is_replaced: replaced,
                    aspect_ratio: AspectRatioOf::new(S::from_f64(2.0)),
                    flex_basis: FlexBasisOf::px(S::from_f64(90.0)),
                    flex_grow: FlexGrowOf::ZERO,
                    flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                fri07_c02_collapse_round_item(0.0, 15.0, FlexItemCollapse::Collapsed),
            )
            .measure(2, Size::new(S::from_f64(90.0), S::from_f64(10.0)));
        let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
            .expect("replacedness composition control succeeds");
        let target = fri07_c02_collapse_round_output(&batch, 2);
        assert_eq!(target.size.height, S::from_f64(20.0));
        assert_eq!(target.source_index, SourceIndex::new(0));
        assert_eq!(
            fri07_c02_collapse_round_output(&batch, 3),
            NodeOutputOf::with_source_index(SourceIndex::new(1))
        );
        widths.push(target.size.width);
    }
    assert_eq!(widths, [S::from_f64(60.0), S::from_f64(90.0)]);
}

#[test]
fn fri07_c02_composition_rotated_dimensions_have_independent_layout_effects() {
    assert_fri07_c02_composition_rotated_dimensions_are_observable::<f32>();
    assert_fri07_c02_composition_rotated_dimensions_are_observable::<f64>();
    assert_fri07_c02_composition_replacedness_is_observable::<f32>();
    assert_fri07_c02_composition_replacedness_is_observable::<f64>();
}

const FRI07_C03_COMPOSED_SCALAR_TOLERANCE: f64 = 0.000_02;

#[derive(Clone, Copy, Debug)]
struct Fri07C03ComposedCase {
    swap_intrinsic_bases: bool,
    collapse_max_item: bool,
    reverse_order: bool,
    reverse_source: bool,
    flow: FlowAxes,
    direction: FlexDirection,
    wrap: FlexWrap,
    replaced: bool,
    cross_auto_margin_pattern: usize,
    absolute_pattern: usize,
    overflow: ComputedOverflow,
    container_main: f64,
}

impl Fri07C03ComposedCase {
    fn deterministic() -> Self {
        Self {
            swap_intrinsic_bases: false,
            collapse_max_item: false,
            reverse_order: false,
            reverse_source: false,
            flow: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            replaced: false,
            cross_auto_margin_pattern: 3,
            absolute_pattern: 0,
            overflow: computed_overflow(Overflow::Visible, Overflow::Clip),
            container_main: 120.0,
        }
    }

    fn axes(self) -> FlexAxes {
        FlexAxes::new(self.flow, self.direction, self.wrap)
    }

    fn children(self) -> [u32; 4] {
        if self.reverse_source {
            [3, 4, 2, 5]
        } else {
            [2, 3, 4, 5]
        }
    }

    fn source_index(self, node: u32) -> SourceIndex {
        let index = self
            .children()
            .iter()
            .position(|child| *child == node)
            .expect("every composed child has a source position");
        SourceIndex::new(index)
    }
}

#[derive(Clone, Debug)]
struct Fri07C03ComposedTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    axes: FlexAxes,
    measure_mode: Cell<Fri07C03ComposedMeasureMode>,
    requests: RefCell<Vec<(u32, LeafMeasureInputOf<S>)>>,
    cache_queries: RefCell<Vec<(u32, bool)>>,
    retained: Fri07C01CompositionRetained<S>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C03ComposedMeasureMode {
    Values,
    FailIntrinsic,
    FailSecondRound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C03ComposedMeasureError {
    Intrinsic,
    SecondRound,
}

impl<S: LayoutScalar> Traverse for Fri07C03ComposedTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C03ComposedTree<S> {
    type MeasureError = Fri07C03ComposedMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        matches!(node, 2 | 3)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if !matches!(node, 2 | 3) {
            return None;
        }
        let node_request_index = self
            .requests
            .borrow()
            .iter()
            .filter(|(request_node, _)| *request_node == node)
            .count();
        self.requests.borrow_mut().push((node, input));
        match (self.measure_mode.get(), node, node_request_index) {
            (Fri07C03ComposedMeasureMode::FailIntrinsic, 2, 0) => {
                return Some(Err(Fri07C03ComposedMeasureError::Intrinsic));
            }
            (Fri07C03ComposedMeasureMode::FailSecondRound, 2, 1) => {
                return Some(Err(Fri07C03ComposedMeasureError::SecondRound));
            }
            _ => {}
        }
        let main = match (node, self.axes.main_size(input.available_content_size())) {
            (2, MeasurementAvailableOf::MinContent) => S::from_f64(20.0),
            (2, MeasurementAvailableOf::MaxContent) => S::from_f64(45.0),
            (3, MeasurementAvailableOf::MinContent) => S::from_f64(25.0),
            (3, MeasurementAvailableOf::MaxContent) => S::from_f64(60.0),
            (_, MeasurementAvailableOf::Definite(value)) => value.get(),
            _ => unreachable!("only composed intrinsic leaves are measured"),
        };
        let cross = if node == 2 { 20.0 } else { 30.0 };
        Some(Ok(self.axes.size_from_main_cross(main, S::from_f64(cross))))
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<NodeOutputOf<S>> {
        self.retained.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri07C03ComposedTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = Fri07C01CompositionRetained<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut prepared = self.retained.clone();
        for node in batch.invalidated_nodes() {
            prepared.unrounded.remove(node);
            prepared.final_outputs.remove(node);
            prepared.caches.remove(node);
        }
        for entry in batch.unrounded_entries() {
            prepared.unrounded.insert(entry.node(), entry.output());
        }
        for entry in batch.final_entries() {
            prepared.final_outputs.insert(entry.node(), entry.output());
        }
        for entry in batch.cache_clear_entries() {
            prepared.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            Fri07C01CompositionTree::apply_cache_entry(&mut prepared, entry);
        }
        Ok(prepared)
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

fn fri07_c03_composed_layout_tree<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
    collapsed_main: f64,
) -> Fri07C03ComposedTree<S> {
    let axes = case.axes();
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let length = |value| LengthOf::px(S::from_f64(value));
    let auto_length = |value| LengthAutoOf::px(S::from_f64(value));
    let mut cross_margin = Edges::all(LengthAutoOf::ZERO);
    if matches!(case.cross_auto_margin_pattern, 1 | 3) {
        axes.set_normal_cross_start_edge(&mut cross_margin, LengthAutoOf::AUTO);
    }
    if matches!(case.cross_auto_margin_pattern, 2 | 3) {
        axes.set_normal_cross_end_edge(&mut cross_margin, LengthAutoOf::AUTO);
    }
    let (min_basis, max_basis) = if case.swap_intrinsic_bases {
        (FlexBasisOf::MAX_CONTENT, FlexBasisOf::MIN_CONTENT)
    } else {
        (FlexBasisOf::MIN_CONTENT, FlexBasisOf::MAX_CONTENT)
    };
    let (min_order, max_order) = if case.reverse_order {
        (ItemOrder::new(3), ItemOrder::new(-3))
    } else {
        (ItemOrder::new(-3), ItemOrder::new(3))
    };
    let (inset, absolute_margin) = match case.absolute_pattern {
        0 => (
            Edges::new(
                auto_length(5.0),
                auto_length(20.0),
                auto_length(15.0),
                auto_length(10.0),
            ),
            Edges::all(LengthAutoOf::AUTO),
        ),
        1 => (
            Edges {
                top: auto_length(5.0),
                left: auto_length(10.0),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            Edges::all(LengthAutoOf::AUTO),
        ),
        2 => (
            Edges::all(auto_length(40.0)),
            Edges::all(LengthAutoOf::AUTO),
        ),
        _ => unreachable!("the absolute pattern strategy is bounded"),
    };
    let intrinsic_item = |basis, order, collapse, margin, replaced| NodeInputOf {
        item_order: order,
        item_is_replaced: replaced,
        flex_item_collapse: collapse,
        size: axes.size_from_main_cross(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
        min_size: axes.size_from_main_cross(MinSizeOf::ZERO, MinSizeOf::ZERO),
        flex_basis: basis,
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
        margin,
        ..NodeInputOf::default()
    };
    let collapsed = NodeInputOf {
        item_order: ItemOrder::new(0),
        flex_item_collapse: FlexItemCollapse::Collapsed,
        size: axes.size_from_main_cross(preferred(collapsed_main), preferred(50.0)),
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
        overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
        scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(3.0))
            .expect("collapsed scrollbar width is finite"),
        ..NodeInputOf::default()
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, case.children())
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: case.flow.writing_mode(),
                direction: case.flow.direction(),
                flex_direction: case.direction,
                flex_wrap: case.wrap,
                size: axes
                    .size_from_main_cross(preferred(case.container_main), PreferredSizeOf::AUTO),
                gap: axes.size_from_main_cross(length(5.0), length(4.0)),
                align_content: Some(AlignContent::FlexStart),
                align_items: Some(AlignItems::FlexStart),
                overflow: case.overflow,
                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(3.0))
                    .expect("composed scrollbar width is finite"),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            intrinsic_item(
                min_basis,
                min_order,
                FlexItemCollapse::Normal,
                cross_margin,
                case.replaced,
            ),
        )
        .style(
            3,
            intrinsic_item(
                max_basis,
                max_order,
                if case.collapse_max_item {
                    FlexItemCollapse::Collapsed
                } else {
                    FlexItemCollapse::Normal
                },
                Edges::all(LengthAutoOf::ZERO),
                false,
            ),
        )
        .style(4, collapsed)
        .style(
            5,
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: Size::new(preferred(20.0), preferred(10.0)),
                margin: absolute_margin,
                ..NodeInputOf::default()
            },
        );

    Fri07C03ComposedTree {
        tree,
        axes,
        measure_mode: Cell::new(Fri07C03ComposedMeasureMode::Values),
        requests: RefCell::new(Vec::new()),
        cache_queries: RefCell::new(Vec::new()),
        retained: Fri07C01CompositionRetained::default(),
    }
}

#[derive(Clone, Debug)]
struct Fri07C03ComposedSnapshot<S: LayoutScalar> {
    outputs: [NodeOutputOf<S>; 5],
    requests: Vec<(u32, LeafMeasureInputOf<S>)>,
}

impl<S: LayoutScalar> Fri07C03ComposedSnapshot<S> {
    fn output(&self, node: u32) -> NodeOutputOf<S> {
        self.outputs[(node - 1) as usize]
    }

    fn geometry(&self) -> Vec<f64> {
        let mut geometry = Vec::new();
        for output in self.outputs {
            geometry.extend([
                output.location.x.to_f64(),
                output.location.y.to_f64(),
                output.size.width.to_f64(),
                output.size.height.to_f64(),
                output.margin.top.to_f64(),
                output.margin.right.to_f64(),
                output.margin.bottom.to_f64(),
                output.margin.left.to_f64(),
            ]);
        }
        let scroll = self
            .output(1)
            .scroll_geometry
            .expect("composed root publishes scroll geometry");
        geometry.extend([
            scroll.physical_range().x().minimum().to_f64(),
            scroll.physical_range().x().maximum().to_f64(),
            scroll.physical_range().y().minimum().to_f64(),
            scroll.physical_range().y().maximum().to_f64(),
            scroll.scrollbar_size().width.to_f64(),
            scroll.scrollbar_size().height.to_f64(),
        ]);
        geometry
    }
}

fn fri07_c03_composed_layout_snapshot<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
    collapsed_main: f64,
) -> Fri07C03ComposedSnapshot<S> {
    let tree = fri07_c03_composed_layout_tree::<S>(case, collapsed_main);
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("all four completed flex capabilities compose");
    let outputs = core::array::from_fn(|index| {
        fri07_c01_composition_output(batch.unrounded_entries(), (index + 1) as u32)
    });
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        for node in 1..=5 {
            assert_fri07_c02_composition_finite_output(
                fri07_c01_composition_output(entries, node),
                &format!("C03 composed node {node}"),
            );
        }
    }
    Fri07C03ComposedSnapshot {
        outputs,
        requests: tree.requests.into_inner(),
    }
}

fn fri07_c03_expected_intrinsic<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
    node: u32,
) -> MeasurementAvailableOf<S> {
    match (case.swap_intrinsic_bases, node) {
        (false, 2) | (true, 3) => MeasurementAvailableOf::MIN_CONTENT,
        (false, 3) | (true, 2) => MeasurementAvailableOf::MAX_CONTENT,
        _ => unreachable!("only the two intrinsic items have basis expectations"),
    }
}

fn assert_fri07_c03_composed_layout_case<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
) -> Fri07C03ComposedSnapshot<S> {
    let axes = case.axes();
    let snapshot = fri07_c03_composed_layout_snapshot::<S>(case, 70.0);
    for node in 2..=5 {
        assert_eq!(
            snapshot.output(node).source_index,
            case.source_index(node),
            "node {node} remains associated with its raw source position for {case:?}"
        );
    }
    assert_eq!(
        snapshot.output(4),
        NodeOutputOf::with_source_index(case.source_index(4)),
        "the strut item publishes a zero box"
    );
    if case.collapse_max_item {
        assert_eq!(
            snapshot.output(3),
            NodeOutputOf::with_source_index(case.source_index(3)),
            "the rotated collapsed intrinsic item publishes no geometry"
        );
    }

    for node in [2, 3] {
        let expected = fri07_c03_expected_intrinsic::<S>(case, node);
        let intrinsic_count = snapshot
            .requests
            .iter()
            .filter(|(request_node, input)| {
                *request_node == node && axes.main_size(input.available_content_size()) == expected
            })
            .count();
        assert!(
            intrinsic_count >= 1,
            "node {node} must retain its selected intrinsic constraint for {case:?}; requests={:?}",
            snapshot.requests
        );
        let collapse_round_markers = snapshot
            .requests
            .iter()
            .filter(|(request_node, input)| {
                *request_node == node
                    && axes.main_size(input.available_content_size()) == expected
                    && axes.main_size(input.known_content_size()).is_none()
                    && match axes.cross_size(input.available_content_size()) {
                        MeasurementAvailableOf::Definite(value) => value.get() > S::from_f64(50.0),
                        MeasurementAvailableOf::MinContent | MeasurementAvailableOf::MaxContent => {
                            false
                        }
                    }
            })
            .count();
        assert!(
            (1..=2).contains(&collapse_round_markers),
            "node {node} observes no more than two complete collapsed-layout settlements for {case:?}; requests={:?}",
            snapshot.requests
        );
    }

    assert!(
        axes.cross_size(snapshot.output(1).size) >= S::from_f64(50.0),
        "the collapsed item's first-round 50px used cross size remains a line strut"
    );
    let min = snapshot.output(2);
    let cross_start = axes.normal_cross_start_edge(min.margin);
    let cross_end = axes.normal_cross_end_edge(min.margin);
    match case.cross_auto_margin_pattern {
        0 => assert_eq!((cross_start, cross_end), (S::ZERO, S::ZERO)),
        1 => {
            assert!(cross_start >= S::ZERO);
            assert_eq!(cross_end, S::ZERO);
        }
        2 => {
            assert_eq!(cross_start, S::ZERO);
            assert!(cross_end >= S::ZERO);
        }
        3 => {
            assert!(cross_start >= S::ZERO);
            fri07_c01_composition_assert_near(
                cross_start - cross_end,
                0.0,
                "paired ordinary cross auto margins",
            );
        }
        _ => unreachable!("the cross auto-margin strategy is bounded"),
    }

    let absolute = snapshot.output(5);
    let containing_scrollport = snapshot
        .output(1)
        .scroll_geometry
        .expect("composed root publishes its inset containing geometry")
        .scrollport();
    let containing_size = containing_scrollport.size();
    let containing_origin = containing_scrollport.origin();
    match case.absolute_pattern {
        0 => {
            fri07_c01_composition_assert_near(
                absolute.margin.left + absolute.margin.right,
                containing_size.width.to_f64() - 50.0,
                "definite horizontal inset-modified margin sum",
            );
            fri07_c01_composition_assert_near(
                absolute.margin.top + absolute.margin.bottom,
                containing_size.height.to_f64() - 30.0,
                "definite vertical inset-modified margin sum",
            );
            fri07_c01_composition_assert_near(
                absolute.location.x - absolute.margin.left,
                containing_origin.x.to_f64() + 10.0,
                "absolute definite left inset",
            );
            fri07_c01_composition_assert_near(
                absolute.location.y - absolute.margin.top,
                containing_origin.y.to_f64() + 5.0,
                "absolute definite top inset",
            );
        }
        1 => {
            assert_eq!(absolute.margin, Edges::ZERO);
            assert_eq!(
                absolute.location,
                Point::new(
                    containing_origin.x + S::from_f64(10.0),
                    containing_origin.y + S::from_f64(5.0),
                )
            );
        }
        2 => {
            fri07_c01_composition_assert_near(
                absolute.margin.left + absolute.margin.right,
                containing_size.width.to_f64() - 100.0,
                "negative horizontal inset-modified margin sum",
            );
            fri07_c01_composition_assert_near(
                absolute.margin.top + absolute.margin.bottom,
                containing_size.height.to_f64() - 90.0,
                "negative vertical inset-modified margin sum",
            );
        }
        _ => unreachable!("the absolute strategy is bounded"),
    }

    let root_scroll = snapshot
        .output(1)
        .scroll_geometry
        .expect("composed root publishes settled scroll geometry");
    assert_eq!(root_scroll.used_overflow_x(), case.overflow.x());
    assert_eq!(root_scroll.used_overflow_y(), case.overflow.y());

    let payload_control = fri07_c03_composed_layout_snapshot::<S>(case, 370.0);
    for node in [1, 2, 3, 5] {
        assert_eq!(
            snapshot.output(node),
            payload_control.output(node),
            "the collapsed item's first-round main size and scroll state cannot contribute to committed node {node}"
        );
    }
    snapshot
}

#[test]
fn fri07_c03_composed_layout_exact_geometry_margins_strut_absolute_and_scroll() {
    let case = Fri07C03ComposedCase::deterministic();
    let snapshot = assert_fri07_c03_composed_layout_case::<f64>(case);
    let root = snapshot.output(1);
    let min = snapshot.output(2);
    let max = snapshot.output(3);
    let absolute = snapshot.output(5);

    assert_eq!(root.size, Size::new(120.0, 50.0));
    assert_eq!(root.content_size, Size::new(120.0, 50.0));
    assert_eq!(min.location, Point::new(0.0, 15.0));
    assert_eq!(min.size, Size::new(20.0, 20.0));
    assert_eq!(min.margin, Edges::new(15.0, 0.0, 15.0, 0.0));
    assert_eq!(max.location, Point::new(25.0, 0.0));
    assert_eq!(max.size, Size::new(60.0, 30.0));
    assert_eq!(absolute.location, Point::new(45.0, 15.0));
    assert_eq!(absolute.margin, Edges::new(10.0, 35.0, 10.0, 35.0));
    let scroll = root
        .scroll_geometry
        .expect("deterministic root publishes scroll geometry");
    assert_eq!(scroll.used_overflow_x(), Overflow::Visible);
    assert_eq!(scroll.used_overflow_y(), Overflow::Clip);
    assert_eq!(scroll.scrollbar_size(), Size::ZERO);
    assert_eq!(scroll.physical_range().x().minimum(), 0.0);
    assert_eq!(scroll.physical_range().x().maximum(), 0.0);
    assert_eq!(scroll.physical_range().y().minimum(), 0.0);
    assert_eq!(scroll.physical_range().y().maximum(), 0.0);
}

fn fri07_c03_composed_layout_cases() -> Vec<Fri07C03ComposedCase> {
    let mut cases = Vec::new();
    let base = Fri07C03ComposedCase::deterministic();
    for swap_intrinsic_bases in [false, true] {
        cases.push(Fri07C03ComposedCase {
            swap_intrinsic_bases,
            ..base
        });
    }
    for collapse_max_item in [false, true] {
        cases.push(Fri07C03ComposedCase {
            collapse_max_item,
            ..base
        });
    }
    for (reverse_order, reverse_source) in [(false, false), (true, false), (false, true)] {
        cases.push(Fri07C03ComposedCase {
            reverse_order,
            reverse_source,
            ..base
        });
    }
    for flow in [
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
    ] {
        cases.push(Fri07C03ComposedCase { flow, ..base });
    }
    for direction in [
        FlexDirection::Row,
        FlexDirection::RowReverse,
        FlexDirection::Column,
        FlexDirection::ColumnReverse,
    ] {
        cases.push(Fri07C03ComposedCase { direction, ..base });
    }
    for wrap in [FlexWrap::NoWrap, FlexWrap::Wrap, FlexWrap::WrapReverse] {
        cases.push(Fri07C03ComposedCase {
            wrap,
            container_main: if wrap == FlexWrap::NoWrap {
                120.0
            } else {
                70.0
            },
            ..base
        });
    }
    for replaced in [false, true] {
        cases.push(Fri07C03ComposedCase { replaced, ..base });
    }
    for cross_auto_margin_pattern in 0..4 {
        cases.push(Fri07C03ComposedCase {
            cross_auto_margin_pattern,
            ..base
        });
    }
    for absolute_pattern in 0..3 {
        cases.push(Fri07C03ComposedCase {
            absolute_pattern,
            ..base
        });
    }
    for overflow in [
        computed_overflow(Overflow::Visible, Overflow::Clip),
        computed_overflow(Overflow::Hidden, Overflow::Auto),
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        computed_overflow(Overflow::Scroll, Overflow::Hidden),
    ] {
        cases.push(Fri07C03ComposedCase { overflow, ..base });
    }
    cases
}

#[test]
fn fri07_c03_composed_layout_paired_controls_rotate_every_owned_dimension() {
    let cases = fri07_c03_composed_layout_cases();
    assert_eq!(
        cases.len(),
        30,
        "the deterministic control set stays bounded"
    );
    for (index, case) in cases.into_iter().enumerate() {
        let f32_snapshot = assert_fri07_c03_composed_layout_case::<f32>(case);
        let f64_snapshot = assert_fri07_c03_composed_layout_case::<f64>(case);
        let f32_geometry = f32_snapshot.geometry();
        let f64_geometry = f64_snapshot.geometry();
        assert_eq!(f32_geometry.len(), f64_geometry.len());
        for (field, (f32_value, f64_value)) in
            f32_geometry.into_iter().zip(f64_geometry).enumerate()
        {
            assert!(
                (f32_value - f64_value).abs() <= FRI07_C03_COMPOSED_SCALAR_TOLERANCE,
                "deterministic control {index} field {field} differs across scalar lanes: {f32_value} versus {f64_value}; case={case:?}"
            );
        }
    }

    let normal = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        collapse_max_item: false,
        ..Fri07C03ComposedCase::deterministic()
    });
    let collapsed = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        collapse_max_item: true,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(normal.output(3).size, Size::ZERO);
    assert_eq!(collapsed.output(3).size, Size::ZERO);

    let min_basis = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        swap_intrinsic_bases: false,
        ..Fri07C03ComposedCase::deterministic()
    });
    let max_basis = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        swap_intrinsic_bases: true,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_eq!(min_basis.output(2).size.width, 20.0);
    assert_eq!(max_basis.output(2).size.width, 45.0);

    let source_forward =
        assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase::deterministic());
    let source_reverse = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        reverse_source: true,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_eq!(
        source_forward.output(2).location,
        source_reverse.output(2).location
    );
    assert_ne!(
        source_forward.output(2).source_index,
        source_reverse.output(2).source_index
    );

    let order_reverse = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        reverse_order: true,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(
        source_forward.output(2).location,
        order_reverse.output(2).location
    );
    assert_eq!(source_forward.output(2).size, order_reverse.output(2).size);

    let vertical_flow = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        flow: FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_eq!(source_forward.output(1).size, Size::new(120.0, 50.0));
    assert_eq!(vertical_flow.output(1).size, Size::new(50.0, 120.0));

    let row_reverse = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        direction: FlexDirection::RowReverse,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(
        source_forward.output(2).location,
        row_reverse.output(2).location
    );

    let narrow_nowrap = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        container_main: 70.0,
        ..Fri07C03ComposedCase::deterministic()
    });
    let narrow_wrap = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        wrap: FlexWrap::Wrap,
        container_main: 70.0,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(narrow_nowrap.output(1).size, narrow_wrap.output(1).size);

    let non_replaced = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        swap_intrinsic_bases: true,
        replaced: false,
        container_main: 40.0,
        ..Fri07C03ComposedCase::deterministic()
    });
    let replaced = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        swap_intrinsic_bases: true,
        replaced: true,
        container_main: 40.0,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_eq!(
        non_replaced.output(2).size,
        replaced.output(2).size,
        "direct intrinsic-basis geometry remains selected by the provider while replacedness rotates independently"
    );

    let no_cross_auto = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        cross_auto_margin_pattern: 0,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(
        no_cross_auto.output(2).margin,
        source_forward.output(2).margin
    );

    let auto_inset = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        absolute_pattern: 1,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(source_forward.output(5).margin, auto_inset.output(5).margin);

    let forced_scroll = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        overflow: computed_overflow(Overflow::Scroll, Overflow::Hidden),
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(
        source_forward
            .output(1)
            .scroll_geometry
            .expect("visible control has scroll geometry")
            .scrollbar_size(),
        forced_scroll
            .output(1)
            .scroll_geometry
            .expect("forced-scroll control has scroll geometry")
            .scrollbar_size()
    );
}

fn fri07_c03_flow(selector: usize) -> FlowAxes {
    [
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysRl, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
    ][selector]
}

fn fri07_c03_direction(selector: usize) -> FlexDirection {
    [
        FlexDirection::Row,
        FlexDirection::RowReverse,
        FlexDirection::Column,
        FlexDirection::ColumnReverse,
    ][selector]
}

fn fri07_c03_wrap(selector: usize) -> FlexWrap {
    [FlexWrap::NoWrap, FlexWrap::Wrap, FlexWrap::WrapReverse][selector]
}

fn fri07_c03_overflow(selector: usize) -> ComputedOverflow {
    [
        computed_overflow(Overflow::Visible, Overflow::Clip),
        computed_overflow(Overflow::Hidden, Overflow::Auto),
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        computed_overflow(Overflow::Scroll, Overflow::Hidden),
    ][selector]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn fri07_c03_composed_layout_bounded_property_preserves_invariants(
        swap_intrinsic_bases in any::<bool>(),
        collapse_max_item in any::<bool>(),
        reverse_order in any::<bool>(),
        reverse_source in any::<bool>(),
        flow_selector in 0usize..6,
        direction_selector in 0usize..4,
        wrap_selector in 0usize..3,
        replaced in any::<bool>(),
        cross_auto_margin_pattern in 0usize..4,
        absolute_pattern in 0usize..3,
        overflow_selector in 0usize..4,
        container_main in 70u16..151,
    ) {
        let case = Fri07C03ComposedCase {
            swap_intrinsic_bases,
            collapse_max_item,
            reverse_order,
            reverse_source,
            flow: fri07_c03_flow(flow_selector),
            direction: fri07_c03_direction(direction_selector),
            wrap: fri07_c03_wrap(wrap_selector),
            replaced,
            cross_auto_margin_pattern,
            absolute_pattern,
            overflow: fri07_c03_overflow(overflow_selector),
            container_main: f64::from(container_main),
        };
        let f32_snapshot = assert_fri07_c03_composed_layout_case::<f32>(case);
        let f64_snapshot = assert_fri07_c03_composed_layout_case::<f64>(case);
        let f32_geometry = f32_snapshot.geometry();
        let f64_geometry = f64_snapshot.geometry();
        prop_assert_eq!(f32_geometry.len(), f64_geometry.len());
        for (field, (f32_value, f64_value)) in
            f32_geometry.into_iter().zip(f64_geometry).enumerate()
        {
            prop_assert!(
                (f32_value - f64_value).abs() <= FRI07_C03_COMPOSED_SCALAR_TOLERANCE,
                "property field {} differs across scalar lanes: {} versus {}; case={:?}",
                field,
                f32_value,
                f64_value,
                case,
            );
        }

        let basis_control = assert_fri07_c03_composed_layout_case::<f64>(
            Fri07C03ComposedCase {
                swap_intrinsic_bases: !case.swap_intrinsic_bases,
                ..case
            },
        );
        prop_assert_ne!(
            f64_snapshot.output(2).size,
            basis_control.output(2).size,
            "the paired basis control must change only the selected intrinsic geometry"
        );

        let source_control = assert_fri07_c03_composed_layout_case::<f64>(
            Fri07C03ComposedCase {
                reverse_source: !case.reverse_source,
                ..case
            },
        );
        prop_assert_eq!(
            f64_snapshot.output(2).location,
            source_control.output(2).location,
            "source rotation cannot change order-modified physical geometry"
        );
        prop_assert_ne!(
            f64_snapshot.output(2).source_index,
            source_control.output(2).source_index,
            "source rotation remains observable in stable source association"
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Fri07C03ComposedStateMeasurement<S: LayoutScalar> {
    node: u32,
    known_main: Option<S>,
    known_cross: Option<S>,
    available_main: MeasurementAvailableOf<S>,
    available_cross: MeasurementAvailableOf<S>,
}

fn fri07_c03_composed_state_measurements<S: LayoutScalar>(
    tree: &Fri07C03ComposedTree<S>,
) -> Vec<Fri07C03ComposedStateMeasurement<S>> {
    tree.requests
        .borrow()
        .iter()
        .map(|(node, input)| Fri07C03ComposedStateMeasurement {
            node: *node,
            known_main: tree.axes.main_size(input.known_content_size()),
            known_cross: tree.axes.cross_size(input.known_content_size()),
            available_main: tree.axes.main_size(input.available_content_size()),
            available_cross: tree.axes.cross_size(input.available_content_size()),
        })
        .collect()
}

fn fri07_c03_composed_state_definite<S: LayoutScalar>(value: f64) -> MeasurementAvailableOf<S> {
    MeasurementAvailableOf::definite(S::from_f64(value))
        .expect("composed state measurement target is finite and non-negative")
}

fn assert_fri07_c03_composed_state_exact_round_bounds<S: LayoutScalar>() {
    let tree = fri07_c03_composed_layout_tree::<S>(Fri07C03ComposedCase::deterministic(), 70.0);
    compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("bounded composed state layout succeeds");
    let measurement = |node, available_main, available_cross| Fri07C03ComposedStateMeasurement {
        node,
        known_main: None,
        known_cross: None,
        available_main,
        available_cross,
    };
    assert_eq!(
        fri07_c03_composed_state_measurements(&tree),
        [
            measurement(
                2,
                MeasurementAvailableOf::MIN_CONTENT,
                fri07_c03_composed_state_definite(300.0),
            ),
            measurement(
                3,
                MeasurementAvailableOf::MAX_CONTENT,
                fri07_c03_composed_state_definite(300.0),
            ),
            measurement(
                2,
                MeasurementAvailableOf::MIN_CONTENT,
                fri07_c03_composed_state_definite(20.0),
            ),
            measurement(
                3,
                MeasurementAvailableOf::MAX_CONTENT,
                fri07_c03_composed_state_definite(30.0),
            ),
        ],
        "the complete composed tree performs only the existing intrinsic pass and one finite collapsed replay"
    );
}

#[test]
fn fri07_c03_composed_state_measurement_trace_has_exact_finite_round_bound() {
    assert_fri07_c03_composed_state_exact_round_bounds::<f32>();
    assert_fri07_c03_composed_state_exact_round_bounds::<f64>();
}

fn fri07_c03_composed_state_geometry<S: LayoutScalar>(
    unrounded: &[LayoutOutputEntryOf<u32, S>],
    final_outputs: &[LayoutOutputEntryOf<u32, S>],
) -> Vec<f64> {
    let mut geometry = Vec::new();
    for entries in [unrounded, final_outputs] {
        for node in 1..=5 {
            let output = fri07_c01_composition_output(entries, node);
            geometry.extend([
                output.location.x.to_f64(),
                output.location.y.to_f64(),
                output.size.width.to_f64(),
                output.size.height.to_f64(),
                output.margin.top.to_f64(),
                output.margin.right.to_f64(),
                output.margin.bottom.to_f64(),
                output.margin.left.to_f64(),
            ]);
        }
    }
    geometry
}

fn assert_fri07_c03_composed_state_cache_rounding_and_scalar<S: LayoutScalar>() -> Vec<f64> {
    let case = Fri07C03ComposedCase {
        container_main: 120.5,
        ..Fri07C03ComposedCase::deterministic()
    };
    let mut tree = fri07_c03_composed_layout_tree::<S>(case, 70.25);
    let request = fri07_c02_collapse_round_request();
    let cold = compute_layout(&tree, 1, request).expect("cold composed state layout succeeds");
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();
    let cold_unrounded_fragments = cold.unrounded_inline_fragments().to_vec();
    let cold_final_fragments = cold.final_inline_fragments().to_vec();
    let cold_measurements = fri07_c03_composed_state_measurements(&tree);
    assert_eq!(cold_measurements.len(), 4);

    for node in 1..=5 {
        let unrounded = fri07_c01_composition_output(&cold_unrounded, node);
        let rounded = fri07_c01_composition_output(&cold_final, node);
        assert_eq!(unrounded.source_index, rounded.source_index);
        for (unrounded_start, unrounded_size, rounded_start, rounded_size) in [
            (
                unrounded.location.x,
                unrounded.size.width,
                rounded.location.x,
                rounded.size.width,
            ),
            (
                unrounded.location.y,
                unrounded.size.height,
                rounded.location.y,
                rounded.size.height,
            ),
        ] {
            fri07_c01_composition_assert_near(
                rounded_start,
                unrounded_start.to_f64().round(),
                "rounded composed source start",
            );
            fri07_c01_composition_assert_near(
                rounded_start + rounded_size,
                (unrounded_start + unrounded_size).to_f64().round(),
                "rounded composed source end",
            );
        }
    }
    let unrounded_absolute = fri07_c01_composition_output(&cold_unrounded, 5);
    let rounded_absolute = fri07_c01_composition_output(&cold_final, 5);
    assert_ne!(unrounded_absolute.location.x, rounded_absolute.location.x);
    assert_eq!(
        fri07_c01_composition_output(&cold_unrounded, 4),
        NodeOutputOf::with_source_index(case.source_index(4))
    );
    assert_eq!(
        fri07_c01_composition_output(&cold_final, 4),
        NodeOutputOf::with_source_index(case.source_index(4))
    );

    cold.apply_to(&mut tree)
        .expect("cold composed state batch commit succeeds");
    let cold_retained = tree.retained.clone();
    assert!(!cold_retained.caches.is_empty());

    tree.cache_queries.borrow_mut().clear();
    tree.requests.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm composed state layout succeeds");
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert_eq!(warm.unrounded_inline_fragments(), cold_unrounded_fragments);
    assert_eq!(warm.final_inline_fragments(), cold_final_fragments);
    assert!(
        tree.cache_queries.borrow().iter().any(|(_, hit)| *hit),
        "warm composed state layout reuses committed cache facts"
    );
    assert!(
        fri07_c03_composed_state_measurements(&tree).len() <= cold_measurements.len(),
        "warm cache use cannot introduce another flex or collapse round"
    );
    warm.apply_to(&mut tree)
        .expect("warm composed state batch commit succeeds");
    assert_eq!(tree.retained.unrounded, cold_retained.unrounded);
    assert_eq!(tree.retained.final_outputs, cold_retained.final_outputs);
    for entry in warm.cache_store_entries().iter().rev() {
        let committed =
            tree.retained.caches[&entry.node()].get_with_context(entry.input(), entry.context());
        assert_eq!(
            committed,
            Some(entry.output()),
            "every warm staged cache fact is committed through the existing cache owner"
        );
    }

    fri07_c03_composed_state_geometry(&cold_unrounded, &cold_final)
}

#[test]
fn fri07_c03_composed_state_cold_warm_rounding_and_scalar_lanes_agree() {
    let f32_geometry = assert_fri07_c03_composed_state_cache_rounding_and_scalar::<f32>();
    let f64_geometry = assert_fri07_c03_composed_state_cache_rounding_and_scalar::<f64>();
    assert_eq!(f32_geometry.len(), f64_geometry.len());
    for (field, (f32_value, f64_value)) in f32_geometry.into_iter().zip(f64_geometry).enumerate() {
        assert!(
            (f32_value - f64_value).abs() <= FRI07_C03_COMPOSED_SCALAR_TOLERANCE,
            "composed state field {field} differs across scalar lanes: {f32_value} versus {f64_value}"
        );
    }
}

fn assert_fri07_c03_composed_state_failure_is_atomic<S: LayoutScalar>(
    mode: Fri07C03ComposedMeasureMode,
    expected_error: Fri07C03ComposedMeasureError,
    expected_requests: usize,
) {
    let case = Fri07C03ComposedCase::deterministic();
    let request = fri07_c02_collapse_round_request();
    let mut tree = fri07_c03_composed_layout_tree::<S>(case, 70.0);
    let initial =
        compute_layout(&tree, 1, request).expect("initial composed state layout succeeds");
    initial
        .apply_to(&mut tree)
        .expect("initial composed state batch commit succeeds");

    tree.requests.borrow_mut().clear();
    tree.measure_mode.set(mode);
    let retained_before_failure = tree.retained.clone();
    let error = compute_layout_invalidated(&tree, 1, request, &[1, 2])
        .expect_err("composed provider failure returns no partial batch");
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(error) if *error == expected_error
    ));
    assert_eq!(
        tree.retained, retained_before_failure,
        "failed composed layout commits neither partial output nor cache"
    );
    assert_eq!(
        fri07_c03_composed_state_measurements(&tree).len(),
        expected_requests,
        "failure occurs at its exact bounded measurement phase"
    );

    tree.requests.borrow_mut().clear();
    tree.measure_mode.set(Fri07C03ComposedMeasureMode::Values);
    let recovery = compute_layout_invalidated(&tree, 1, request, &[1, 2])
        .expect("composed state recovers after provider failure");
    let mut fresh_tree = fri07_c03_composed_layout_tree::<S>(case, 70.0);
    let fresh = compute_layout_invalidated(&fresh_tree, 1, request, &[1, 2])
        .expect("fresh composed state layout succeeds");
    assert_eq!(recovery.unrounded_entries(), fresh.unrounded_entries());
    assert_eq!(recovery.final_entries(), fresh.final_entries());
    assert_eq!(
        recovery.unrounded_inline_fragments(),
        fresh.unrounded_inline_fragments()
    );
    assert_eq!(
        recovery.final_inline_fragments(),
        fresh.final_inline_fragments()
    );
    recovery
        .apply_to(&mut tree)
        .expect("recovery batch commit succeeds");
    fresh
        .apply_to(&mut fresh_tree)
        .expect("fresh batch commit succeeds");
    assert_eq!(tree.retained.unrounded, fresh_tree.retained.unrounded);
    assert_eq!(
        tree.retained.final_outputs,
        fresh_tree.retained.final_outputs
    );
    let recovered_warm = compute_layout(&tree, 1, request)
        .expect("recovered composed cache serves a complete warm layout");
    let fresh_warm = compute_layout(&fresh_tree, 1, request)
        .expect("fresh composed cache serves a complete warm layout");
    assert_eq!(
        recovered_warm.unrounded_entries(),
        fresh_warm.unrounded_entries(),
        "recovery cache behavior matches a fresh tree"
    );
    assert_eq!(recovered_warm.final_entries(), fresh_warm.final_entries());
}

#[test]
fn fri07_c03_composed_state_intrinsic_and_second_round_failures_are_atomic_and_recoverable() {
    for (mode, expected_error, expected_requests) in [
        (
            Fri07C03ComposedMeasureMode::FailIntrinsic,
            Fri07C03ComposedMeasureError::Intrinsic,
            1,
        ),
        (
            Fri07C03ComposedMeasureMode::FailSecondRound,
            Fri07C03ComposedMeasureError::SecondRound,
            3,
        ),
    ] {
        assert_fri07_c03_composed_state_failure_is_atomic::<f32>(
            mode,
            expected_error,
            expected_requests,
        );
        assert_fri07_c03_composed_state_failure_is_atomic::<f64>(
            mode,
            expected_error,
            expected_requests,
        );
    }
}

fn fri07_c03_composed_state_batch<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
    collapsed_main: f64,
    absolute_collapse: FlexItemCollapse,
) -> CompletedLayoutBatchOf<u32, S> {
    let mut tree = fri07_c03_composed_layout_tree::<S>(case, collapsed_main);
    let mut absolute = tree.tree.node_input(5).clone();
    absolute.flex_item_collapse = absolute_collapse;
    tree.tree = core::mem::take(&mut tree.tree).style(5, absolute);
    compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("composed state control layout succeeds")
}

fn assert_fri07_c03_composed_state_settlement_and_inert_absolute<S: LayoutScalar>() {
    let case = Fri07C03ComposedCase {
        overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
        container_main: 120.5,
        ..Fri07C03ComposedCase::deterministic()
    };
    let baseline = fri07_c03_composed_state_batch::<S>(case, 70.25, FlexItemCollapse::Normal);
    let hostile = fri07_c03_composed_state_batch::<S>(case, 370.25, FlexItemCollapse::Normal);
    assert_eq!(baseline.unrounded_entries(), hostile.unrounded_entries());
    assert_eq!(baseline.final_entries(), hostile.final_entries());
    let root = fri07_c01_composition_output(baseline.unrounded_entries(), 1);
    let scroll = root
        .scroll_geometry
        .expect("composed overflow control publishes scroll geometry");
    assert_eq!(scroll.used_overflow_x(), Overflow::Auto);
    assert_eq!(scroll.used_overflow_y(), Overflow::Scroll);
    assert_eq!(scroll.scrollbar_size().width, S::from_f64(3.0));
    assert_eq!(
        fri07_c01_composition_output(baseline.unrounded_entries(), 4),
        NodeOutputOf::with_source_index(case.source_index(4))
    );
    assert_eq!(
        fri07_c01_composition_output(baseline.final_entries(), 4),
        NodeOutputOf::with_source_index(case.source_index(4))
    );

    let collapsed_absolute =
        fri07_c03_composed_state_batch::<S>(case, 70.25, FlexItemCollapse::Collapsed);
    assert_eq!(
        baseline.unrounded_entries(),
        collapsed_absolute.unrounded_entries(),
        "collapse remains inert on the composed absolute child"
    );
    assert_eq!(baseline.final_entries(), collapsed_absolute.final_entries());
}

#[test]
fn fri07_c03_composed_state_settlement_excludes_collapsed_facts_and_absolute_is_inert() {
    assert_fri07_c03_composed_state_settlement_and_inert_absolute::<f32>();
    assert_fri07_c03_composed_state_settlement_and_inert_absolute::<f64>();
}

fn fri07_c03_composed_state_assert_unsupported_basis(
    flex_basis: FlexBasisOf<f64>,
    behavior: SizingBehavior,
) {
    let case = Fri07C03ComposedCase::deterministic();
    let mut tree = fri07_c03_composed_layout_tree::<f64>(case, 70.0);
    let mut intrinsic = tree.tree.node_input(2).clone();
    intrinsic.flex_basis = flex_basis;
    tree.tree = core::mem::take(&mut tree.tree).style(2, intrinsic);
    let error = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect_err("later-owned flex basis remains unsupported in the composed tree");
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    let LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected exact sizing capability payload, got {error:?}");
    };
    assert_eq!(unsupported.property(), SizingProperty::FlexBasis);
    assert_eq!(unsupported.behavior(), behavior);
    assert_eq!(unsupported.algorithm(), SizingAlgorithm::Flex);
    assert_eq!(unsupported.axis(), PhysicalAxis::Horizontal);
}

#[test]
fn fri07_c03_composed_state_later_owned_flex_basis_payloads_remain_exact() {
    let sizing = || {
        SizingCalculationOf::value(
            LengthPercentageOf::px(10.0).expect("finite composed sizing calculation"),
        )
    };
    for (flex_basis, behavior) in [
        (FlexBasisOf::STRETCH, SizingBehavior::Stretch),
        (FlexBasisOf::FIT_CONTENT, SizingBehavior::FitContent),
        (FlexBasisOf::CONTAIN, SizingBehavior::Contain),
        (
            FlexBasisOf::fit_content_function(sizing()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri07_c03_composed_state_assert_unsupported_basis(flex_basis, behavior);
    }

    let calc = CalcSizeCalculationOf::value(LengthPercentageOf::ZERO);
    for (basis, payload) in [
        (FlexBasisCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
        (FlexBasisCalcBasis::Content, CalcSizeBehaviorBasis::Content),
        (
            FlexBasisCalcBasis::MinContent,
            CalcSizeBehaviorBasis::MinContent,
        ),
        (
            FlexBasisCalcBasis::MaxContent,
            CalcSizeBehaviorBasis::MaxContent,
        ),
        (FlexBasisCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
        (
            FlexBasisCalcBasis::FitContent,
            CalcSizeBehaviorBasis::FitContent,
        ),
        (FlexBasisCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
    ] {
        fri07_c03_composed_state_assert_unsupported_basis(
            FlexBasisOf::calc_size(basis, calc.clone()).expect("valid composed calc-size"),
            SizingBehavior::CalcSize(payload),
        );
    }
}

fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

fn assert_fri06_mr02_geometry_error_flex_own<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite();
    let style = NodeInputOf {
        display: Display::Flex,
        size: Size::new(PreferredSizeOf::px(largest), PreferredSizeOf::px(S::ONE)),
        padding: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        border: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        ..NodeInputOf::default()
    };

    for (run_mode, operation, invariant) in [
        (
            RunMode::PerformRootLayout,
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        (
            RunMode::PerformLayout,
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        ),
    ] {
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(7, [])
            .style(7, style.clone());
        let error = compute_flex(&mut tree, 7, fri06_mr02_geometry_error_input(run_mode))
            .expect_err("overflowing flex geometry must fail");

        fri06_mr02_geometry_error_assert(error, LayoutErrorSiteOf::Node(7), operation, invariant);
    }
}

fn assert_fri06_mr02_geometry_error_flex_child<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(7, [11])
        .children(11, [])
        .style(
            7,
            NodeInputOf {
                display: Display::Flex,
                size: size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .style(11, NodeInputOf::default())
        .measure(
            11,
            ComputeOutputOf::from_sizes(
                Size::new(S::from_f64(10.0), S::from_f64(10.0)),
                Size::splat(S::INFINITY),
            ),
        );
    let error = compute_flex(
        &mut tree,
        7,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            size.map(Some),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            size.map(AvailableOf::definite),
        ),
    )
    .expect_err("invalid retained flex child geometry must fail");

    fri06_mr02_geometry_error_assert(
        error,
        LayoutErrorSiteOf::ContainerSubject {
            container: 7,
            subject: 11,
        },
        LayoutOperation::ChildLayout,
        LayoutInternalInvariant::InvalidBlockScrollGeometry,
    );
}

fn assert_fri08_c07_t02_scroll_source_flex_paths<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let size = Size::new(scalar(100.0), scalar(80.0));
    let flow_axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl);
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let scroll_margin =
        ScrollMarginOf::try_new(scalar(-1.0), scalar(2.0), scalar(-3.0), scalar(4.0)).unwrap();
    let mut tree = OracleTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                flex_direction: FlexDirection::RowReverse,
                flex_wrap: FlexWrap::WrapReverse,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                overflow_clip_margin: OverflowClipMarginOf::try_new(
                    OverflowClipBox::ContentBox,
                    scalar(2.0),
                )
                .unwrap(),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(7.0)).unwrap(),
                size: size.map(PreferredSizeOf::px),
                padding: Edges::all(LengthOf::px(scalar(3.0))),
                scroll_padding: ScrollPaddingOf::new(
                    ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(1.0)).unwrap()),
                    ScrollPaddingValueOf::AUTO,
                    ScrollPaddingValueOf::AUTO,
                    ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(4.0)).unwrap()),
                ),
                scroll_snap_type: ScrollSnapType::Enabled {
                    axis: ScrollSnapAxis::Inline,
                    strictness: ScrollSnapStrictness::Proximity,
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: Size::new(
                    PreferredSizeOf::px(scalar(140.0)),
                    PreferredSizeOf::px(scalar(90.0)),
                ),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
                scroll_margin,
                scroll_snap_align: snap_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                position: Position::Absolute,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: Size::new(
                    PreferredSizeOf::px(scalar(30.0)),
                    PreferredSizeOf::px(scalar(20.0)),
                ),
                inset: Edges::new(
                    LengthAutoOf::px(scalar(5.0)),
                    LengthAutoOf::AUTO,
                    LengthAutoOf::AUTO,
                    LengthAutoOf::px(scalar(6.0)),
                ),
                scroll_margin,
                scroll_snap_align: snap_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes(
                Size::new(scalar(30.0), scalar(20.0)),
                Size::new(scalar(42.0), scalar(31.0)),
            ),
        );

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            size.map(Some),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            size.map(AvailableOf::definite),
        ),
    )
    .unwrap();

    let container = output.scroll_geometry.unwrap();
    assert_eq!(container.flow_axes(), flow_axes);
    assert!(container.overflow_clip().x().is_some());
    assert!(container.overflow_clip().y().is_some());
    assert_ne!(container.scrollbar_size(), Size::ZERO);
    assert_eq!(container.resolved_scroll_padding().top, scalar(1.0));
    assert_eq!(container.resolved_scroll_padding().left, scalar(4.0));
    assert_eq!(
        container.scroll_snap_type(),
        ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Inline,
            strictness: ScrollSnapStrictness::Proximity,
        }
    );
    let container_range = container.physical_range();
    assert!(container_range.x().minimum() <= S::ZERO);
    assert!(container_range.y().minimum() <= S::ZERO);

    let existing = tree.layout(1).unwrap();
    let existing_geometry = existing.scroll_geometry.unwrap();
    assert_eq!(existing_geometry.border_box().size(), existing.size);
    assert_eq!(existing_geometry.target().flow_axes(), flow_axes);
    assert_eq!(existing_geometry.target().scroll_margin(), scroll_margin);
    assert_eq!(existing_geometry.target().snap_align(), snap_align);

    let reconstructed = tree.layout(2).unwrap();
    let reconstructed_geometry = reconstructed.scroll_geometry.unwrap();
    assert_eq!(reconstructed_geometry.flow_axes(), flow_axes);
    assert_eq!(
        reconstructed_geometry.target().border_box().size(),
        reconstructed.size
    );
    assert_eq!(
        reconstructed_geometry.target().scroll_margin(),
        scroll_margin
    );
    assert_eq!(reconstructed_geometry.target().snap_align(), snap_align);
    assert_eq!(
        reconstructed_geometry.target().snap_stop(),
        ScrollSnapStop::Always
    );
    assert_eq!(
        reconstructed_geometry.scrollable_overflow().size(),
        Size::new(scalar(42.0), scalar(31.0))
    );
}

#[test]
fn fri08_c07_t02_scroll_source_flex_preserves_existing_reconstruction_and_origins() {
    assert_fri08_c07_t02_scroll_source_flex_paths::<f32>();
    assert_fri08_c07_t02_scroll_source_flex_paths::<f64>();
}

#[test]
fn fri08_c07_t02_scroll_source_flex_preserves_caller_local_errors() {
    assert_fri06_mr02_geometry_error_flex_own::<f32>();
    assert_fri06_mr02_geometry_error_flex_own::<f64>();
    assert_fri06_mr02_geometry_error_flex_child::<f32>();
    assert_fri06_mr02_geometry_error_flex_child::<f64>();
}

#[test]
fn fri06_mr02_geometry_error_flex_own_preserves_root_and_child_mapping_both_scalars() {
    assert_fri06_mr02_geometry_error_flex_own::<f32>();
    assert_fri06_mr02_geometry_error_flex_own::<f64>();
}

#[test]
fn fri06_mr02_geometry_error_flex_child_preserves_container_subject_both_scalars() {
    assert_fri06_mr02_geometry_error_flex_child::<f32>();
    assert_fri06_mr02_geometry_error_flex_child::<f64>();
}

#[test]
fn fri08_c07_t05_scroll_fixture_flex_assertion_preserves_error_identity() {
    assert_fri06_mr02_geometry_error_flex_own::<f32>();
    assert_fri06_mr02_geometry_error_flex_own::<f64>();
    assert_fri06_mr02_geometry_error_flex_child::<f32>();
    assert_fri06_mr02_geometry_error_flex_child::<f64>();
}

fn fri06_mr02_scroll_padding_cases<S: LayoutScalar>() -> [(ScrollPaddingOf<S>, Edges<S>); 2] {
    let [first, second] = scroll_padding_inputs();

    [
        (
            first,
            Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO),
        ),
        (
            second,
            Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
        ),
    ]
}

fn assert_fri06_mr02_scroll_padding_flex<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    for (scroll_padding, expected) in fri06_mr02_scroll_padding_cases() {
        let style = NodeInputOf::<S> {
            display: Display::Flex,
            size: Size::new(
                PreferredSizeOf::px(size.width),
                PreferredSizeOf::px(size.height),
            ),
            scroll_padding,
            ..NodeInputOf::default()
        };
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [])
            .style(0, style);
        let output = compute_flex(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                size.map(Some),
                size.map(Some),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                size.map(AvailableOf::definite),
            ),
        )
        .expect("flex scroll-padding characterization succeeds");
        let geometry = output
            .scroll_geometry
            .expect("performed flex layout emits geometry");

        assert_eq!(geometry.resolved_scroll_padding(), expected);
    }
}

#[test]
fn fri06_mr02_scroll_padding_flex_preserves_auto_and_value_on_each_physical_edge() {
    assert_fri06_mr02_scroll_padding_flex::<f32>();
    assert_fri06_mr02_scroll_padding_flex::<f64>();
}

#[test]
fn fri08_c07_t05_scroll_fixture_flex_rows_preserve_exact_auto_and_value_edges() {
    fn assert_rows<S: LayoutScalar>() {
        assert_scroll_padding_inputs_exact::<S>();
        assert_eq!(
            fri06_mr02_scroll_padding_cases::<S>().map(|(_, expected)| expected),
            [
                Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO,),
                Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
            ]
        );
    }

    assert_rows::<f32>();
    assert_rows::<f64>();
}

fn fri04_c03_flex_value(value: f32) -> SizingCalculation {
    SizingCalculation::value(LengthPercentageOf::px(value).expect("test sizing value is finite"))
}

fn fri04_c03_flex_nested(minimum: f32, preferred: f32, maximum: f32) -> SizingCalculation {
    let preferred = SizingCalculation::max(vec![
        fri04_c03_flex_value(preferred),
        SizingCalculation::min(vec![
            fri04_c03_flex_value(preferred - 5.0),
            fri04_c03_flex_value(preferred + 5.0),
        ])
        .expect("nested minimum is nonempty"),
    ])
    .expect("nested maximum is nonempty");
    SizingCalculation::clamp(
        Some(fri04_c03_flex_value(minimum)),
        preferred,
        Some(fri04_c03_flex_value(maximum)),
    )
}

fn fri04_c03_flex_percentage_nested(
    minimum: f32,
    percentage: f32,
    maximum: f32,
) -> SizingCalculation {
    let preferred = SizingCalculation::max(vec![
        SizingCalculation::value(
            LengthPercentageOf::from_percent_fraction(percentage)
                .expect("test percentage is finite"),
        ),
        SizingCalculation::min(vec![
            fri04_c03_flex_value(minimum + 5.0),
            fri04_c03_flex_value(maximum - 5.0),
        ])
        .expect("nested minimum is nonempty"),
    ])
    .expect("nested maximum is nonempty");
    SizingCalculation::clamp(
        Some(fri04_c03_flex_value(minimum)),
        preferred,
        Some(fri04_c03_flex_value(maximum)),
    )
}

#[test]
fn fri04_c04_flex_dispatch_auto_uses_preferred_main_size_but_content_bypasses_it() {
    fn first_known_main_size(
        preferred_main_size: PreferredSize,
        flex_basis: FlexBasis,
    ) -> Option<f32> {
        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    display: Display::Flex,
                    size: Size::new(PreferredSize::px(200.0), PreferredSize::px(40.0)),
                    ..NodeInput::default()
                },
            )
            .style(
                2,
                NodeInput {
                    size: Size::new(preferred_main_size, PreferredSize::px(20.0)),
                    min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                    flex_basis,
                    ..NodeInput::default()
                },
            )
            .measure(2, ComputeOutput::from_outer_size(Size::new(25.0, 20.0)));

        compute_flex(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(300.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::definite(300.0)),
            ),
        )
        .expect("supported flex basis resolves");

        tree.inputs(2)
            .first()
            .expect("flex item is measured")
            .known()
            .width
    }

    assert_eq!(
        first_known_main_size(PreferredSize::px(80.0), FlexBasis::AUTO),
        Some(80.0)
    );
    assert_eq!(
        first_known_main_size(PreferredSize::AUTO, FlexBasis::AUTO),
        None
    );
    assert_eq!(
        first_known_main_size(PreferredSize::px(80.0), FlexBasis::CONTENT),
        None
    );
}

fn fri04_c04_flex_dispatch_first_item_input(
    direction: FlexDirection,
    container_main: Option<f32>,
    child: NodeInput,
) -> ComputeInput {
    let container_size = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            container_main.map_or(PreferredSize::AUTO, PreferredSize::px),
            PreferredSize::px(100.0),
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSize::px(100.0),
            container_main.map_or(PreferredSize::AUTO, PreferredSize::px),
        ),
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                size: container_size,
                flex_direction: direction,
                ..NodeInput::default()
            },
        )
        .style(2, child)
        .measure(2, ComputeOutput::from_outer_size(Size::new(25.0, 20.0)));

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(300.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .expect("supported flex dispatch resolves");

    *tree
        .inputs(2)
        .first()
        .expect("flex item receives a sizing input")
}

#[test]
fn fri04_c04_flex_dispatch_numeric_and_calc_size_bases_use_each_physical_main_axis() {
    let ordinary = || {
        FlexBasis::calculation(SizingCalculation::value(
            LengthPercentageOf::from_percent_fraction(0.5).expect("finite percentage"),
        ))
    };
    let any = || {
        FlexBasis::calc_size(
            FlexBasisCalcBasis::Any,
            CalcSizeCalculation::from_coefficients(10.0, 0.5, 0.0).expect("finite Any calculation"),
        )
        .expect("Any calculation does not reference size")
    };
    let full = || {
        FlexBasis::calc_size(
            FlexBasisCalcBasis::FullPercentage,
            CalcSizeCalculation::from_coefficients(10.0, 0.1, 0.5)
                .expect("finite FullPercentage calculation"),
        )
        .expect("valid FullPercentage calculation")
    };

    for (direction, axis) in [
        (FlexDirection::Row, PhysicalAxis::Horizontal),
        (FlexDirection::Column, PhysicalAxis::Vertical),
    ] {
        for (basis, expected) in [(ordinary(), 100.0), (any(), 110.0), (full(), 130.0)] {
            let input = fri04_c04_flex_dispatch_first_item_input(
                direction,
                Some(200.0),
                NodeInput {
                    min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                    flex_basis: basis,
                    ..NodeInput::default()
                },
            );
            assert_eq!(
                match axis {
                    PhysicalAxis::Horizontal => input.known().width,
                    PhysicalAxis::Vertical => input.known().height,
                },
                Some(expected)
            );
        }

        let any_missing = fri04_c04_flex_dispatch_first_item_input(
            direction,
            None,
            NodeInput {
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: any(),
                ..NodeInput::default()
            },
        );
        let full_missing = fri04_c04_flex_dispatch_first_item_input(
            direction,
            None,
            NodeInput {
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: full(),
                ..NodeInput::default()
            },
        );
        let ordinary_missing = fri04_c04_flex_dispatch_first_item_input(
            direction,
            None,
            NodeInput {
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: ordinary(),
                ..NodeInput::default()
            },
        );
        let main = |input: ComputeInput| match axis {
            PhysicalAxis::Horizontal => input.known().width,
            PhysicalAxis::Vertical => input.known().height,
        };
        assert_eq!(main(any_missing), Some(10.0));
        assert_eq!(main(full_missing), None);
        assert_eq!(main(ordinary_missing), None);
    }
}

fn fri04_c04_flex_dispatch_assert_error(
    container: NodeInput,
    child: NodeInput,
    property: SizingProperty,
    behavior: SizingBehavior,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
) {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(1, container)
        .style(2, child)
        .measure(2, ComputeOutput::from_outer_size(Size::splat(10.0)));
    let error = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(200.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::definite(200.0)),
        ),
    )
    .expect_err("later-owned flex sizing must be rejected");
    assert_eq!(error.site(), LayoutErrorSite::Node(2));
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected exact sizing capability, got {:?}", error.kind());
    };
    assert_eq!(
        (
            unsupported.property(),
            unsupported.behavior(),
            unsupported.algorithm(),
            unsupported.axis(),
        ),
        (property, behavior, algorithm, axis)
    );
}

#[test]
fn fri04_c04_flex_dispatch_direct_and_keyword_bases_return_exact_payloads() {
    let sizing =
        || SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite calculation"));
    let calc = || CalcSizeCalculation::value(LengthPercentageOf::ZERO);
    let container = || NodeInput {
        display: Display::Flex,
        size: Size::new(PreferredSize::px(200.0), PreferredSize::px(100.0)),
        ..NodeInput::default()
    };

    for (value, behavior) in [
        (PreferredSize::MIN_CONTENT, SizingBehavior::MinContent),
        (PreferredSize::MAX_CONTENT, SizingBehavior::MaxContent),
        (PreferredSize::STRETCH, SizingBehavior::Stretch),
        (PreferredSize::FIT_CONTENT, SizingBehavior::FitContent),
        (PreferredSize::CONTAIN, SizingBehavior::Contain),
        (
            PreferredSize::fit_content_function(sizing()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                size: Size::new(value, PreferredSize::AUTO),
                ..NodeInput::default()
            },
            SizingProperty::Preferred,
            behavior,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
        );
    }
    for (value, behavior) in [
        (MinSize::MIN_CONTENT, SizingBehavior::MinContent),
        (MinSize::MAX_CONTENT, SizingBehavior::MaxContent),
        (MinSize::STRETCH, SizingBehavior::Stretch),
        (MinSize::FIT_CONTENT, SizingBehavior::FitContent),
        (MinSize::CONTAIN, SizingBehavior::Contain),
        (
            MinSize::fit_content_function(sizing()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                min_size: Size::new(MinSize::AUTO, value),
                ..NodeInput::default()
            },
            SizingProperty::Minimum,
            behavior,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
        );
    }
    for (value, behavior) in [
        (MaxSize::MIN_CONTENT, SizingBehavior::MinContent),
        (MaxSize::MAX_CONTENT, SizingBehavior::MaxContent),
        (MaxSize::STRETCH, SizingBehavior::Stretch),
        (MaxSize::FIT_CONTENT, SizingBehavior::FitContent),
        (MaxSize::CONTAIN, SizingBehavior::Contain),
        (
            MaxSize::fit_content_function(sizing()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                max_size: Size::new(value, MaxSize::NONE),
                ..NodeInput::default()
            },
            SizingProperty::Maximum,
            behavior,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
        );
    }
    for (value, behavior) in [
        (FlexBasis::STRETCH, SizingBehavior::Stretch),
        (FlexBasis::FIT_CONTENT, SizingBehavior::FitContent),
        (FlexBasis::CONTAIN, SizingBehavior::Contain),
        (
            FlexBasis::fit_content_function(sizing()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            NodeInput {
                flex_direction: FlexDirection::Column,
                ..container()
            },
            NodeInput {
                flex_basis: value,
                ..NodeInput::default()
            },
            SizingProperty::FlexBasis,
            behavior,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
        );
    }

    for (basis, expected) in [
        (PreferredSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
        (
            PreferredSizeCalcBasis::MinContent,
            CalcSizeBehaviorBasis::MinContent,
        ),
        (
            PreferredSizeCalcBasis::MaxContent,
            CalcSizeBehaviorBasis::MaxContent,
        ),
        (
            PreferredSizeCalcBasis::Stretch,
            CalcSizeBehaviorBasis::Stretch,
        ),
        (
            PreferredSizeCalcBasis::FitContent,
            CalcSizeBehaviorBasis::FitContent,
        ),
        (
            PreferredSizeCalcBasis::Contain,
            CalcSizeBehaviorBasis::Contain,
        ),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                size: Size::new(
                    PreferredSize::calc_size(basis, calc()).expect("valid calc-size"),
                    PreferredSize::AUTO,
                ),
                ..NodeInput::default()
            },
            SizingProperty::Preferred,
            SizingBehavior::CalcSize(expected),
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
        );
    }
    for (basis, expected) in [
        (MinSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
        (
            MinSizeCalcBasis::MinContent,
            CalcSizeBehaviorBasis::MinContent,
        ),
        (
            MinSizeCalcBasis::MaxContent,
            CalcSizeBehaviorBasis::MaxContent,
        ),
        (MinSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
        (
            MinSizeCalcBasis::FitContent,
            CalcSizeBehaviorBasis::FitContent,
        ),
        (MinSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                min_size: Size::new(
                    MinSize::AUTO,
                    MinSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                ..NodeInput::default()
            },
            SizingProperty::Minimum,
            SizingBehavior::CalcSize(expected),
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
        );
    }
    for (basis, expected) in [
        (MaxSizeCalcBasis::None, CalcSizeBehaviorBasis::None),
        (
            MaxSizeCalcBasis::MinContent,
            CalcSizeBehaviorBasis::MinContent,
        ),
        (
            MaxSizeCalcBasis::MaxContent,
            CalcSizeBehaviorBasis::MaxContent,
        ),
        (MaxSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
        (
            MaxSizeCalcBasis::FitContent,
            CalcSizeBehaviorBasis::FitContent,
        ),
        (MaxSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                max_size: Size::new(
                    MaxSize::calc_size(basis, calc()).expect("valid calc-size"),
                    MaxSize::NONE,
                ),
                ..NodeInput::default()
            },
            SizingProperty::Maximum,
            SizingBehavior::CalcSize(expected),
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
        );
    }
    for (basis, expected) in [
        (FlexBasisCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
        (FlexBasisCalcBasis::Content, CalcSizeBehaviorBasis::Content),
        (
            FlexBasisCalcBasis::MinContent,
            CalcSizeBehaviorBasis::MinContent,
        ),
        (
            FlexBasisCalcBasis::MaxContent,
            CalcSizeBehaviorBasis::MaxContent,
        ),
        (FlexBasisCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
        (
            FlexBasisCalcBasis::FitContent,
            CalcSizeBehaviorBasis::FitContent,
        ),
        (FlexBasisCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            NodeInput {
                flex_direction: FlexDirection::Column,
                ..container()
            },
            NodeInput {
                flex_basis: FlexBasis::calc_size(basis, calc()).expect("valid calc-size"),
                ..NodeInput::default()
            },
            SizingProperty::FlexBasis,
            SizingBehavior::CalcSize(expected),
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
        );
    }
}

#[test]
fn fri04_c04_flex_dispatch_container_item_root_and_absolute_report_consuming_algorithm() {
    let container_style = || NodeInput {
        display: Display::Flex,
        size: Size::new(PreferredSize::px(200.0), PreferredSize::px(100.0)),
        ..NodeInput::default()
    };

    fri04_c04_flex_dispatch_assert_error(
        container_style(),
        NodeInput {
            size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
            ..NodeInput::default()
        },
        SizingProperty::Preferred,
        SizingBehavior::MinContent,
        SizingAlgorithm::Flex,
        PhysicalAxis::Horizontal,
    );
    fri04_c04_flex_dispatch_assert_error(
        container_style(),
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::AUTO, PreferredSize::MAX_CONTENT),
            ..NodeInput::default()
        },
        SizingProperty::Preferred,
        SizingBehavior::MaxContent,
        SizingAlgorithm::Positioned,
        PhysicalAxis::Vertical,
    );

    let mut container_tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                size: Size::new(PreferredSize::STRETCH, PreferredSize::px(100.0)),
                ..NodeInput::default()
            },
        );
    let container_error = compute_flex(
        &mut container_tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(200.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::definite(200.0)),
        ),
    )
    .expect_err("flex container stretch is later-owned");
    assert_eq!(container_error.site(), LayoutErrorSite::Node(1));
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        container_unsupported,
    )) = container_error.kind()
    else {
        panic!("expected flex container sizing capability");
    };
    assert_eq!(container_unsupported.algorithm(), SizingAlgorithm::Flex);

    let root = PublicLayoutTreeOf::new().style(
        0,
        NodeInput {
            display: Display::Flex,
            min_size: Size::new(MinSize::AUTO, MinSize::STRETCH),
            ..NodeInput::default()
        },
    );
    let root_error = compute_layout(
        &root,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(100.0)))
            .expect("valid root request"),
    )
    .expect_err("flex root minimum stretch is later-owned");
    assert_eq!(root_error.site(), LayoutErrorSite::Node(0));
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        root_unsupported,
    )) = root_error.kind()
    else {
        panic!("expected flex root sizing capability");
    };
    assert_eq!(root_unsupported.algorithm(), SizingAlgorithm::Flex);
    assert_eq!(root_unsupported.property(), SizingProperty::Minimum);
    assert_eq!(root_unsupported.axis(), PhysicalAxis::Vertical);
}

#[test]
fn fri04_c04_flex_dispatch_invalid_numeric_preserves_item_node_site() {
    let invalid = || {
        SizingCalculation::value(
            LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX)
                .expect("finite sizing coefficients"),
        )
    };
    let styles = [
        NodeInput {
            size: Size::new(PreferredSize::calculation(invalid()), PreferredSize::AUTO),
            ..NodeInput::default()
        },
        NodeInput {
            min_size: Size::new(MinSize::calculation(invalid()), MinSize::AUTO),
            ..NodeInput::default()
        },
        NodeInput {
            max_size: Size::new(MaxSize::calculation(invalid()), MaxSize::NONE),
            ..NodeInput::default()
        },
        NodeInput {
            flex_basis: FlexBasis::calculation(invalid()),
            ..NodeInput::default()
        },
    ];

    for style in styles {
        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    display: Display::Flex,
                    size: Size::new(PreferredSize::px(200.0), PreferredSize::px(200.0)),
                    ..NodeInput::default()
                },
            )
            .style(2, style);
        let error = compute_flex(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(200.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::definite(200.0)),
            ),
        )
        .expect_err("overflowing flex sizing calculation must fail");
        assert_eq!(error.site(), LayoutErrorSite::Node(2));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert!(matches!(
            error.kind(),
            LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { value })
                if *value == f32::INFINITY
        ));
    }
}

#[test]
fn fri04_c03_flex_row_layout_consumes_nested_container_item_and_absolute_properties() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2, 3, 4, 5, 6])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(180.0, 200.0, 220.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(100.0, 120.0, 140.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(150.0, 170.0, 190.0)),
                    MinSize::calculation(fri04_c03_flex_nested(80.0, 90.0, 110.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(200.0, 230.0, 250.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(120.0, 150.0, 170.0)),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(60.0, 80.0, 100.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(40.0, 60.0, 80.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(20.0, 40.0, 60.0)),
                    MinSize::calculation(fri04_c03_flex_nested(30.0, 50.0, 70.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(80.0, 100.0, 120.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(45.0, 55.0, 65.0)),
                ),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_percentage_nested(
                    60.0, 0.35, 80.0,
                )),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(-40.0, -20.0, -10.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(-30.0, -15.0, -5.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(-30.0, -20.0, -10.0)),
                    MinSize::calculation(fri04_c03_flex_nested(-20.0, -10.0, -5.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(-30.0, -20.0, -10.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(-20.0, -10.0, -5.0)),
                ),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_nested(-30.0, -10.0, -5.0)),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                position: Position::Absolute,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(70.0, 90.0, 110.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(35.0, 45.0, 55.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(50.0, 60.0, 70.0)),
                    MinSize::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(90.0, 100.0, 120.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(40.0, 45.0, 50.0)),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(10.0, 20.0, 30.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(30.0, 40.0, 50.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            6,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(10.0, 20.0, 30.0)),
                    PreferredSize::calculation(fri04_c03_flex_percentage_nested(20.0, 0.2, 30.0)),
                ),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_nested(10.0, 20.0, 30.0)),
                ..NodeInput::default()
            },
        );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(240.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::definite(240.0)),
        ),
    )
    .expect("row flex calculations resolve");

    assert_eq!(output.size, Size::new(200.0, 120.0));
    assert_eq!(
        tree.output(2).expect("normal child is laid out").size,
        Size::new(70.0, 55.0)
    );
    assert_eq!(
        tree.output(3).expect("negative child is laid out").size,
        Size::ZERO
    );
    assert_eq!(
        tree.output(4).expect("absolute child is laid out").size,
        Size::new(90.0, 45.0)
    );
    assert_eq!(
        tree.output(5)
            .expect("automatic-minimum child is laid out")
            .size,
        Size::new(30.0, 20.0)
    );
    assert_eq!(
        tree.output(6)
            .expect("basis-dependent final-known child is laid out")
            .size,
        Size::new(20.0, 25.0)
    );
}

#[test]
fn fri04_c03_flex_column_layout_maps_nested_main_and_cross_calculations_to_physical_axes() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(180.0)),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(35.0, 45.0, 55.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(70.0, 90.0, 110.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(30.0, 40.0, 50.0)),
                    MinSize::calculation(fri04_c03_flex_nested(40.0, 50.0, 60.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(38.0, 42.0, 48.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(90.0, 100.0, 120.0)),
                ),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_percentage_nested(
                    60.0,
                    75.0 / 180.0,
                    90.0,
                )),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_percentage_nested(20.0, 0.2, 40.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                ),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                ..NodeInput::default()
            },
        );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(140.0), Some(180.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(140.0), Available::definite(180.0)),
        ),
    )
    .expect("column flex calculations resolve");

    assert_eq!(
        tree.output(2).expect("column child is laid out").size,
        Size::new(42.0, 75.0)
    );
    assert_eq!(
        tree.inputs(2)
            .last()
            .expect("final child request is recorded")
            .known(),
        Size::new(Some(42.0), Some(75.0))
    );
    assert_eq!(
        tree.output(3)
            .expect("column final-known child is laid out")
            .size,
        Size::new(28.0, 30.0)
    );
}

#[test]
fn fri04_c03_flex_compute_size_missing_numeric_basis_uses_content_not_authored_main_size() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(PreferredSize::px(90.0), PreferredSize::px(10.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_percentage_nested(
                    10.0, 0.5, 80.0,
                )),
                ..NodeInput::default()
            },
        )
        .measure(
            2,
            ComputeOutput::from_sizes(Size::new(35.0, 10.0), Size::new(35.0, 10.0)),
        );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .expect("missing flex-basis percentage context uses content sizing");

    assert_eq!(output.size, Size::new(35.0, 10.0));
    assert!(tree.inputs(2).iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize
            && input.sizing_mode() == SizingMode::ContentSize
            && input.parent().width.is_none()
    }));
}

#[test]
fn fri04_c03_flex_invalid_numeric_propagates_for_every_numeric_property_role() {
    let invalid = || {
        SizingCalculation::min(vec![
            SizingCalculation::value(
                LengthPercentageOf::from_coefficients(f32::MAX, 1.0)
                    .expect("finite overflowing coefficients"),
            ),
            fri04_c03_flex_value(10.0),
        ])
        .expect("nested minimum is nonempty")
    };

    for role in ["preferred", "minimum", "maximum", "flex-basis"] {
        let mut child = NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            ..NodeInput::default()
        };
        match role {
            "preferred" => child.size.width = PreferredSize::calculation(invalid()),
            "minimum" => child.min_size.width = MinSize::calculation(invalid()),
            "maximum" => child.max_size.width = MaxSize::calculation(invalid()),
            "flex-basis" => child.flex_basis = FlexBasis::calculation(invalid()),
            _ => unreachable!(),
        }

        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    display: Display::Flex,
                    size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(40.0)),
                    ..NodeInput::default()
                },
            )
            .style(2, child)
            .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

        let error = compute_flex(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(f32::MAX), Some(40.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(Available::definite(f32::MAX), Available::definite(40.0)),
            ),
        )
        .expect_err("invalid numeric flex property must fail");

        assert_eq!(error.site(), LayoutErrorSite::Node(2), "role: {role}");
        assert_eq!(
            error.operation(),
            LayoutOperation::ValueResolution,
            "role: {role}"
        );
        assert_eq!(
            error.kind(),
            &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
                value: f32::INFINITY,
            }),
            "role: {role}"
        );
    }
}

#[test]
fn flex_child_context_is_complete_for_layout_sizing_and_absolute_paths() {
    assert_flex_child_context_is_complete::<f32>();
    assert_flex_child_context_is_complete::<f64>();
}

fn assert_flex_child_context_is_complete<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let flow_axes = FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);
    let expected =
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::Flex);

    for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    writing_mode: WritingMode::VerticalLr,
                    direction: Direction::Ltr,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .style(1, NodeInputOf::default())
            .style(
                2,
                NodeInputOf {
                    position: Position::Absolute,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(30.0)),
                        PreferredSizeOf::px(S::from_f64(12.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .measure(
                1,
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(40.0), S::from_f64(20.0))),
            )
            .measure(
                2,
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(30.0), S::from_f64(12.0))),
            );

        crate::compute_flex(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                run_mode,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(300.0)), Some(S::from_f64(240.0))),
                crate::ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(AvailableOf::definite(S::from_f64(300.0))),
            ),
        )
        .expect("flex context capture layout succeeds");

        let normal_inputs = tree.inputs(1);
        assert!(
            !normal_inputs.is_empty(),
            "flex must request its in-flow child"
        );
        assert!(
            normal_inputs
                .iter()
                .all(|input| input.containing_layout_context() == expected),
            "every flex in-flow request must use the parent axes and Flex role: {normal_inputs:#?}"
        );

        if run_mode == RunMode::ComputeSize {
            assert!(
                normal_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::ComputeSize),
                "flex intrinsic sizing must request the child through the complete context"
            );
        } else {
            assert!(
                normal_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::PerformLayout),
                "flex normal layout must request the child through the complete context"
            );
            let absolute_inputs = tree.inputs(2);
            assert!(
                absolute_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::PerformLayout),
                "flex absolute scheduling must request the child"
            );
            assert!(
                absolute_inputs
                    .iter()
                    .all(|input| input.containing_layout_context() == expected),
                "every flex absolute request must use the parent axes and Flex role: {absolute_inputs:#?}"
            );
        }
    }
}

#[derive(Clone, Copy)]
struct FlexAxesExpectation {
    main_logical_axis: LogicalAxis,
    cross_logical_axis: LogicalAxis,
    main_physical_axis: PhysicalAxis,
    cross_physical_axis: PhysicalAxis,
    main_start_side: PhysicalSide,
    main_end_side: PhysicalSide,
    cross_start_side: PhysicalSide,
    cross_end_side: PhysicalSide,
    main_reversed: bool,
    cross_reversed: bool,
    main_progression: PhysicalProgression,
    cross_progression: PhysicalProgression,
}

#[derive(Clone, Copy)]
struct FlexAxesCase {
    writing_mode: WritingMode,
    direction: Direction,
    flex_direction: FlexDirection,
    normal: FlexAxesExpectation,
    wrap_reverse: FlexAxesExpectation,
}

fn assert_flex_axes_expectation(axes: FlexAxes, expectation: FlexAxesExpectation) {
    assert_eq!(axes.main_logical_axis(), expectation.main_logical_axis);
    assert_eq!(axes.cross_logical_axis(), expectation.cross_logical_axis);
    assert_eq!(axes.main_physical_axis(), expectation.main_physical_axis);
    assert_eq!(axes.cross_physical_axis(), expectation.cross_physical_axis);
    assert_eq!(axes.main_start_side(), expectation.main_start_side);
    assert_eq!(axes.main_end_side(), expectation.main_end_side);
    assert_eq!(axes.cross_start_side(), expectation.cross_start_side);
    assert_eq!(axes.cross_end_side(), expectation.cross_end_side);
    assert_eq!(axes.main_is_reversed(), expectation.main_reversed);
    assert_eq!(axes.cross_is_reversed(), expectation.cross_reversed);
    assert_eq!(axes.main_progression(), expectation.main_progression);
    assert_eq!(axes.cross_progression(), expectation.cross_progression);
}

#[test]
fn flex_axes_matrix_covers_all_flows_directions_and_flex_directions() {
    use LogicalAxis::{Block, Inline};
    use PhysicalAxis::{Horizontal, Vertical};
    use PhysicalProgression::{Decreasing, Increasing};
    use PhysicalSide::{Bottom, Left, Right, Top};

    macro_rules! expectation {
        (
            $main_logical_axis:ident,
            $cross_logical_axis:ident,
            $main_physical_axis:ident,
            $cross_physical_axis:ident,
            $main_start_side:ident,
            $main_end_side:ident,
            $cross_start_side:ident,
            $cross_end_side:ident,
            $main_reversed:expr,
            $cross_reversed:expr,
            $main_progression:ident,
            $cross_progression:ident
        ) => {
            FlexAxesExpectation {
                main_logical_axis: $main_logical_axis,
                cross_logical_axis: $cross_logical_axis,
                main_physical_axis: $main_physical_axis,
                cross_physical_axis: $cross_physical_axis,
                main_start_side: $main_start_side,
                main_end_side: $main_end_side,
                cross_start_side: $cross_start_side,
                cross_end_side: $cross_end_side,
                main_reversed: $main_reversed,
                cross_reversed: $cross_reversed,
                main_progression: $main_progression,
                cross_progression: $cross_progression,
            }
        };
    }

    macro_rules! case {
        ($writing_mode:expr, $direction:expr, $flex_direction:expr, $normal:expr, $wrap_reverse:expr) => {
            FlexAxesCase {
                writing_mode: $writing_mode,
                direction: $direction,
                flex_direction: $flex_direction,
                normal: $normal,
                wrap_reverse: $wrap_reverse,
            }
        };
    }

    let cases = [
        case!(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Horizontal, Vertical, Left, Right, Top, Bottom, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Horizontal, Vertical, Left, Right, Bottom, Top, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Horizontal, Vertical, Right, Left, Top, Bottom, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Horizontal, Vertical, Right, Left, Bottom, Top, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Vertical, Horizontal, Top, Bottom, Left, Right, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Vertical, Horizontal, Top, Bottom, Right, Left, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Vertical, Horizontal, Bottom, Top, Left, Right, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Vertical, Horizontal, Bottom, Top, Right, Left, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Horizontal, Vertical, Right, Left, Top, Bottom, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Horizontal, Vertical, Right, Left, Bottom, Top, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Horizontal, Vertical, Left, Right, Top, Bottom, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Horizontal, Vertical, Left, Right, Bottom, Top, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Vertical, Horizontal, Top, Bottom, Right, Left, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Vertical, Horizontal, Top, Bottom, Left, Right, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Vertical, Horizontal, Bottom, Top, Right, Left, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Vertical, Horizontal, Bottom, Top, Left, Right, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, false, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, false, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, true, false,
                Increasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, true, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, false, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, false, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, true, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, true, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, false, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, false, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, true, false,
                Increasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, true, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, false, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, false, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, true, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, true, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, true, true,
                Decreasing, Decreasing
            )
        ),
    ];

    assert_eq!(cases.len(), 40);
    for case in cases {
        let flow_axes = FlowAxes::new(case.writing_mode, case.direction);
        let normal = FlexAxes::new(flow_axes, case.flex_direction, FlexWrap::Wrap);
        let wrap_reverse = FlexAxes::new(flow_axes, case.flex_direction, FlexWrap::WrapReverse);

        assert_eq!(normal.flow_direction(), case.direction);
        assert_eq!(wrap_reverse.flow_direction(), case.direction);
        assert_flex_axes_expectation(normal, case.normal);
        assert_flex_axes_expectation(wrap_reverse, case.wrap_reverse);

        assert_eq!(normal.main_logical_axis(), wrap_reverse.main_logical_axis());
        assert_eq!(
            normal.cross_logical_axis(),
            wrap_reverse.cross_logical_axis()
        );
        assert_eq!(
            normal.main_physical_axis(),
            wrap_reverse.main_physical_axis()
        );
        assert_eq!(
            normal.cross_physical_axis(),
            wrap_reverse.cross_physical_axis()
        );
        assert_eq!(normal.main_start_side(), wrap_reverse.main_start_side());
        assert_eq!(normal.main_end_side(), wrap_reverse.main_end_side());
        assert_eq!(normal.main_is_reversed(), wrap_reverse.main_is_reversed());
        assert_eq!(normal.main_progression(), wrap_reverse.main_progression());
        assert_ne!(normal.cross_start_side(), wrap_reverse.cross_start_side());
        assert_ne!(normal.cross_end_side(), wrap_reverse.cross_end_side());
        assert_ne!(normal.cross_progression(), wrap_reverse.cross_progression());
    }
}

#[test]
fn flex_axes_selectors_and_mutators_follow_the_resolved_mapping() {
    let axes = FlexAxes::new(
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        FlexDirection::ColumnReverse,
        FlexWrap::WrapReverse,
    );
    let size = Size::new(3.0, 5.0);
    let point = Point::new(7.0, 11.0);
    let mut edges = Edges::new(2.0, 3.0, 5.0, 7.0);

    assert_eq!(
        axes.flow_axes(),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr)
    );
    assert_eq!(axes.main_size(size), 3.0);
    assert_eq!(axes.cross_size(size), 5.0);
    assert_eq!(axes.size_from_main_cross(13.0, 17.0), Size::new(13.0, 17.0));
    assert_eq!(axes.with_main_size(size, 19.0), Size::new(19.0, 5.0));
    assert_eq!(axes.with_cross_size(size, 23.0), Size::new(3.0, 23.0));
    assert_eq!(axes.main_point(point), 7.0);
    assert_eq!(axes.cross_point(point), 11.0);
    assert_eq!(
        axes.point_from_main_cross(29.0, 31.0),
        Point::new(29.0, 31.0)
    );

    assert_eq!(axes.main_start_edge(edges), 3.0);
    assert_eq!(axes.main_end_edge(edges), 7.0);
    assert_eq!(axes.cross_start_edge(edges), 2.0);
    assert_eq!(axes.cross_end_edge(edges), 5.0);
    assert_eq!(axes.main_edge_sum(edges), 10.0);
    assert_eq!(axes.cross_edge_sum(edges), 7.0);
    axes.set_main_start_edge(&mut edges, 37.0);
    axes.set_main_end_edge(&mut edges, 41.0);
    axes.set_cross_start_edge(&mut edges, 43.0);
    axes.set_cross_end_edge(&mut edges, 47.0);
    assert_eq!(edges, Edges::new(43.0, 37.0, 47.0, 41.0));

    assert_eq!(axes.main_requested_axis(), crate::RequestedAxis::Horizontal);
    assert_eq!(axes.cross_requested_axis(), crate::RequestedAxis::Vertical);
    assert_eq!(
        axes.main_size_from_cross_aspect(
            11.0,
            AspectRatio::new(2.0).expect("finite positive aspect ratio"),
        ),
        22.0
    );

    let vertical_main = FlexAxes::new(
        FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        FlexDirection::Row,
        FlexWrap::NoWrap,
    );
    assert_eq!(
        vertical_main.main_size_from_cross_aspect(
            22.0,
            AspectRatio::new(2.0).expect("finite positive aspect ratio"),
        ),
        11.0
    );
    assert_eq!(vertical_main.main_size(size), 5.0);
    assert_eq!(vertical_main.cross_size(size), 3.0);
    assert_eq!(
        vertical_main.size_from_main_cross(13.0, 17.0),
        Size::new(17.0, 13.0)
    );
    assert_eq!(
        vertical_main.with_main_size(size, 19.0),
        Size::new(3.0, 19.0)
    );
    assert_eq!(
        vertical_main.with_cross_size(size, 23.0),
        Size::new(23.0, 5.0)
    );
    assert_eq!(vertical_main.main_point(point), 11.0);
    assert_eq!(vertical_main.cross_point(point), 7.0);
    assert_eq!(vertical_main.main_requested_axis(), RequestedAxis::Vertical);
    assert_eq!(
        vertical_main.cross_requested_axis(),
        RequestedAxis::Horizontal
    );
    assert_eq!(
        vertical_main.point_from_main_cross(29.0, 31.0),
        Point::new(31.0, 29.0)
    );

    let mut vertical_edges = Edges::new(2.0, 3.0, 5.0, 7.0);
    assert_eq!(vertical_main.main_start_edge(vertical_edges), 5.0);
    assert_eq!(vertical_main.main_end_edge(vertical_edges), 2.0);
    assert_eq!(vertical_main.cross_start_edge(vertical_edges), 7.0);
    assert_eq!(vertical_main.cross_end_edge(vertical_edges), 3.0);
    assert_eq!(vertical_main.main_edge_sum(vertical_edges), 7.0);
    assert_eq!(vertical_main.cross_edge_sum(vertical_edges), 10.0);
    vertical_main.set_main_start_edge(&mut vertical_edges, 37.0);
    vertical_main.set_main_end_edge(&mut vertical_edges, 41.0);
    vertical_main.set_cross_start_edge(&mut vertical_edges, 43.0);
    vertical_main.set_cross_end_edge(&mut vertical_edges, 47.0);
    assert_eq!(vertical_edges, Edges::new(41.0, 47.0, 37.0, 43.0));
    assert_eq!(
        FlexAxes::new(
            FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
            FlexDirection::Row,
            FlexWrap::Wrap,
        ),
        vertical_main
    );
}

fn fri06_mr02_physical_edge_flex_value<T: Copy>(edges: Edges<T>, side: PhysicalSide) -> T {
    match side {
        PhysicalSide::Top => edges.top,
        PhysicalSide::Right => edges.right,
        PhysicalSide::Bottom => edges.bottom,
        PhysicalSide::Left => edges.left,
    }
}

fn assert_fri06_mr02_physical_edge_flex_carrier<T>(edges: Edges<T>)
where
    T: Copy + core::fmt::Debug + PartialEq,
{
    for (writing_mode, direction) in [
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
    ] {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        for flex_direction in [
            FlexDirection::Row,
            FlexDirection::RowReverse,
            FlexDirection::Column,
            FlexDirection::ColumnReverse,
        ] {
            for wrap in [FlexWrap::Wrap, FlexWrap::WrapReverse] {
                let axes = FlexAxes::new(flow_axes, flex_direction, wrap);
                let normal_start = |axis| match axis {
                    LogicalAxis::Inline => flow_axes.inline_start(),
                    LogicalAxis::Block => flow_axes.block_start(),
                };

                assert_eq!(
                    axes.main_start_edge(edges),
                    fri06_mr02_physical_edge_flex_value(edges, axes.main_start_side())
                );
                assert_eq!(
                    axes.main_end_edge(edges),
                    fri06_mr02_physical_edge_flex_value(edges, axes.main_end_side())
                );
                assert_eq!(
                    axes.cross_start_edge(edges),
                    fri06_mr02_physical_edge_flex_value(edges, axes.cross_start_side())
                );
                assert_eq!(
                    axes.cross_end_edge(edges),
                    fri06_mr02_physical_edge_flex_value(edges, axes.cross_end_side())
                );
                assert_eq!(
                    axes.normal_main_start_edge(edges),
                    fri06_mr02_physical_edge_flex_value(
                        edges,
                        normal_start(axes.main_logical_axis()),
                    )
                );
                assert_eq!(
                    axes.normal_main_end_edge(edges),
                    fri06_mr02_physical_edge_flex_value(
                        edges,
                        normal_start(axes.main_logical_axis()).opposite(),
                    )
                );
                assert_eq!(
                    axes.normal_cross_start_edge(edges),
                    fri06_mr02_physical_edge_flex_value(
                        edges,
                        normal_start(axes.cross_logical_axis()),
                    )
                );
                assert_eq!(
                    axes.normal_cross_end_edge(edges),
                    fri06_mr02_physical_edge_flex_value(
                        edges,
                        normal_start(axes.cross_logical_axis()).opposite(),
                    )
                );
            }
        }
    }
}

fn assert_fri06_mr02_physical_edge_flex_scalar_carriers<S: LayoutScalar>() {
    assert_fri06_mr02_physical_edge_flex_carrier(Edges::new(
        S::from_f64(11.0),
        S::from_f64(22.0),
        S::from_f64(33.0),
        S::from_f64(44.0),
    ));
    assert_fri06_mr02_physical_edge_flex_carrier(Edges::new(
        Some(S::from_f64(11.0)),
        Some(S::from_f64(22.0)),
        Some(S::from_f64(33.0)),
        Some(S::from_f64(44.0)),
    ));
}

#[test]
fn fri06_mr02_physical_edge_flex_selectors_cover_scalar_optional_and_boolean_carriers() {
    assert_fri06_mr02_physical_edge_flex_scalar_carriers::<f32>();
    assert_fri06_mr02_physical_edge_flex_scalar_carriers::<f64>();
    assert_fri06_mr02_physical_edge_flex_carrier(Edges::new(false, false, true, true));
    assert_fri06_mr02_physical_edge_flex_carrier(Edges::new(false, true, false, true));
}

#[test]
fn flex_direction_retains_row_column_and_reverse_classification() {
    assert!(FlexDirection::Row.is_row());
    assert!(FlexDirection::RowReverse.is_row());
    assert!(FlexDirection::Column.is_column());
    assert!(FlexDirection::ColumnReverse.is_column());
    assert!(!FlexDirection::Row.is_reverse());
    assert!(FlexDirection::RowReverse.is_reverse());
}

#[test]
fn flex_row_lays_out_fixed_children_with_gap_and_container_insets() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(30.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 20.0), Size::new(40.0, 20.0)),
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(30.0, 30.0), Size::new(30.0, 30.0)),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(200.0, 42.0));
    assert_eq!(output.content_size, Size::new(198.0, 40.0));

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(6.0, 6.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(40.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(56.0, 6.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(30.0, 30.0)
    );

    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(40.0), Some(20.0)));
    assert_eq!(tree.inputs(3)[0].known(), Size::new(Some(30.0), Some(30.0)));
}

#[test]
fn f64_flex_layout_preserves_fractional_growth() {
    let container_width = 16_777_217.75;
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::px(container_width), PreferredSizeOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
                size: Size::new(PreferredSizeOf::px(20.125), PreferredSizeOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::Block,
                flex_grow: FlexGrowOf::try_new(3.0).unwrap(),
                size: Size::new(PreferredSizeOf::px(20.125), PreferredSizeOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        );

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInputOf::<f64>::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(container_width), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(container_width),
                AvailableOf::MAX_CONTENT,
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(container_width, 10.0));
    assert_eq!(
        tree.output(1)
            .expect("flex layout must stage output for the first child")
            .size
            .width,
        4_194_314.5
    );
    assert_eq!(
        tree.output(2)
            .expect("flex layout must stage output for the second child")
            .size
            .width,
        12_582_903.25
    );
    assert_eq!(
        tree.output(2)
            .expect("flex layout must stage output for the second child")
            .location
            .x,
        4_194_314.5
    );
}

#[test]
fn flex_margin_resolution_handles_invalid_affine_numeric_result_without_panicking() {
    let invalid_margin =
        LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX).expect("finite coefficients");
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                margin: Edges {
                    left: LengthAuto::value(invalid_margin),
                    ..Edges::all(LengthAuto::ZERO)
                },
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 20.0)));

    let error = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(120.0), Some(40.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(120.0), Available::definite(40.0)),
        ),
    )
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(2));
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
    ));
}

#[test]
fn flex_content_size_includes_visible_child_overflow_content() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(40.0, 10.0)
    );
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn flex_final_content_size_uses_rerun_output() {
    let mut tree = FlexTree::default();
    tree.insert_children(0, vec![1]);
    tree.insert_children(1, vec![]);
    tree.insert_style(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        1,
        NodeInput {
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            1,
            OracleMeasurementOf::new(ComputeOutput::from_sizes(
                Size::new(80.0, 40.0),
                Size::new(80.0, 40.0),
            ))
            .run_mode(RunMode::PerformLayout)
            .known(Size::new(Some(80.0), Some(10.0))),
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
        );

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(80.0), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(80.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert!(tree.inputs(1).iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize && input.known().width == Some(80.0)
    }));
    assert!(tree.inputs(1).iter().any(|input| {
        input.run_mode() == RunMode::PerformLayout && input.known().width == Some(80.0)
    }));
    assert_eq!(output.content_size.height, 40.0);
}

#[test]
fn flex_relative_child_inset_offsets_final_layout_location() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(7.0, 3.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn flex_relative_child_trailing_inset_offsets_negative() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            inset: Edges {
                right: LengthAuto::px(5.0),
                bottom: LengthAuto::px(2.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(-5.0, -2.0)
    );
}

#[test]
fn flex_compute_size_short_circuits_when_container_size_is_definite() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for FlexTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("definite compute-size should not measure children")
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn flex_compute_size_measures_children_without_perform_layout() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(20.0, 10.0));
    assert_eq!(tree.inputs(2)[0].run_mode(), RunMode::ComputeSize);
}

#[test]
fn flex_row_auto_main_item_uses_content_sizing_for_base_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(50.0), Some(10.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(50.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    let base_input = tree.inputs(2)[0];
    assert_eq!(base_input.sizing_mode(), SizingMode::ContentSize);
    assert_eq!(base_input.known().width, None);
    assert_eq!(base_input.known().height, Some(10.0));
    assert_eq!(base_input.available().width, Available::MAX_CONTENT);
    assert_eq!(base_input.available().height, Available::definite(10.0));
}

#[test]
fn flex_row_hidden_overflow_item_has_zero_automatic_minimum() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutput::from_outer_size(Size::new(0.0, 50.0)))
                .known(Size::new(Some(0.0), Some(50.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 50.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(40.0, 50.0)));

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(20.0), Some(50.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(0.0, 50.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(40.0, 50.0)
    );
}

#[test]
fn flex_column_hidden_overflow_aspect_item_has_zero_automatic_minimum() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            flex_direction: FlexDirection::Column,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Hidden),
            flex_basis: FlexBasis::px(0.0),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            aspect_ratio: AspectRatio::new(1.0),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutput::from_outer_size(Size::new(100.0, 0.0)))
                .known(Size::new(Some(100.0), Some(0.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 50.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(20.0, 50.0)));

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(20.0), Some(50.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(20.0, 50.0)
    );
}

#[test]
fn flex_column_cross_axis_hidden_overflow_aspect_item_has_zero_automatic_minimum() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            flex_direction: FlexDirection::Column,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            flex_basis: FlexBasis::px(0.0),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            aspect_ratio: AspectRatio::new(1.0),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutput::from_outer_size(Size::new(100.0, 0.0)))
                .known(Size::new(Some(100.0), Some(0.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 50.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(20.0, 50.0)));

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(20.0), Some(50.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(20.0, 50.0)
    );
}

#[test]
fn flex_compute_size_uses_definite_min_max_without_measuring_children() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for FlexTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("definite min/max compute-size should not measure children")
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            min_size: Size::new(MinSize::px(100.0), MinSize::px(40.0)),
            max_size: Size::new(MaxSize::px(100.0), MaxSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
}

#[test]
fn flex_display_none_child_gets_zero_layout_and_hidden_input() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::None,
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree = tree.measure_when(
        3,
        OracleMeasurementOf::new(ComputeOutput::HIDDEN).run_mode(RunMode::PerformHiddenLayout),
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged"),
        NodeOutput::with_source_index(crate::SourceIndex::new(1))
    );
    assert_eq!(
        tree.inputs(3),
        vec![ComputeInput::hidden(crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr,),
            crate::ParentFormattingContext::Flex
        ))]
    );
}

#[test]
fn flex_container_reserves_scrollbar_gutter_from_inner_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(0.0), PreferredSize::px(10.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::new(100.0, 40.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(90.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::ZERO
    );
}

#[test]
fn flex_scrollbar_gutter_uses_left_inset_for_rtl_containers() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn flex_child_layout_records_scrollbar_size_for_scroll_overflow() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(7.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2)
            .expect("child layout is staged")
            .scrollbar_size(),
        Size::new(7.0, 7.0)
    );
}

#[test]
fn flex_absolute_child_uses_insets_without_affecting_flow() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(25.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(12.0)),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 12.0), Size::new(80.0, 32.0)),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::new(100.0, 41.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(25.0, 10.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(7.0, 9.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(20.0, 12.0)
    );
    assert_eq!(tree.inputs(3)[0].known(), Size::new(Some(20.0), Some(12.0)));
}

#[test]
fn flex_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(400.0), PreferredSize::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges::all(LengthAuto::percent(0.05)),
            aspect_ratio: AspectRatio::new(3.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(2)[0].known(),
        Size::new(Some(360.0), Some(120.0))
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(20.0, 15.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(360.0, 120.0)
    );
}

#[test]
fn flex_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(400.0), PreferredSize::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::percent(0.1),
                right: LengthAuto::percent(0.1),
                top: LengthAuto::percent(0.05),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(PreferredSize::percent(0.4), PreferredSize::AUTO),
            aspect_ratio: AspectRatio::new(3.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(2)[0].known(),
        Size::new(Some(160.0), Some(160.0 / 3.0))
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(200.0, 15.0)
    );
}

#[test]
fn flex_absolute_child_max_height_shrinks_flex_grandchild() {
    let mut tree = RecursiveTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![3]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
            flex_direction: FlexDirection::Column,
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            flex_direction: FlexDirection::Column,
            inset: Edges {
                bottom: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            max_size: Size::new(MaxSize::NONE, MaxSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            flex_basis: FlexBasis::px(150.0),
            flex_shrink: FlexShrinkOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 80.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 100.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(100.0, 100.0)
    );
}

#[test]
fn flex_absolute_child_cross_alignment_honors_wrap_reverse() {
    fn layout_child(
        align_self: AlignItems,
        flex_direction: FlexDirection,
        layout_direction: Direction,
    ) -> NodeOutput {
        let mut tree = FlexTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    direction: layout_direction,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    flex_direction,
                    flex_wrap: FlexWrap::WrapReverse,
                    ..NodeInput::default()
                },
            )
            .style(
                2,
                NodeInput {
                    direction: layout_direction,
                    position: Position::Absolute,
                    align_self: Some(align_self),
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                    ..NodeInput::default()
                },
            )
            .measure(2, ComputeOutput::from_outer_size(Size::splat(20.0)));

        compute_flex(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(100.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(Available::definite(100.0), Available::MAX_CONTENT),
            ),
        )
        .unwrap();

        tree.layout(2).expect("absolute child layout is staged")
    }

    let default_layout = layout_child(AlignItems::Stretch, FlexDirection::Row, Direction::Ltr);
    assert_eq!(default_layout.location, Point::new(0.0, 80.0));
    assert_eq!(default_layout.size, Size::new(20.0, 20.0));

    let flex_end_layout = layout_child(AlignItems::FlexEnd, FlexDirection::Row, Direction::Ltr);
    assert_eq!(flex_end_layout.location, Point::new(0.0, 0.0));
    assert_eq!(flex_end_layout.size, Size::new(20.0, 20.0));

    let column_rtl_layout =
        layout_child(AlignItems::Stretch, FlexDirection::Column, Direction::Rtl);
    assert_eq!(column_rtl_layout.location, Point::new(0.0, 0.0));
    assert_eq!(column_rtl_layout.size, Size::new(20.0, 20.0));

    let column_rtl_flex_end_layout =
        layout_child(AlignItems::FlexEnd, FlexDirection::Column, Direction::Rtl);
    assert_eq!(column_rtl_flex_end_layout.location, Point::new(80.0, 0.0));
    assert_eq!(column_rtl_flex_end_layout.size, Size::new(20.0, 20.0));
}

#[test]
fn flex_absolute_child_cross_start_margin_uses_physical_edge_in_rtl_column() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            justify_content: Some(AlignContent::FlexEnd),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            direction: Direction::Rtl,
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::px(10.0),
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(100.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(90.0, 80.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(10.0, 10.0)
    );
}

#[test]
fn flex_absolute_child_uses_min_size_when_min_exceeds_max_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                right: LengthAuto::px(10.0),
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            min_size: Size::new(MinSize::px(50.0), MinSize::px(60.0)),
            max_size: Size::new(MaxSize::px(40.0), MaxSize::px(30.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(100.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(40.0, 30.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(50.0, 60.0)
    );
}

#[test]
fn flex_absolute_child_size_cannot_shrink_below_padding_and_border() {
    fn tree_with_child(child_style: NodeInput) -> FlexTree {
        let mut tree = FlexTree::default();
        tree.insert_children(1, vec![2]);
        tree.insert_children(2, vec![]);
        tree.insert_style(1, NodeInput::default());
        tree.insert_style(2, child_style);
        tree
    }

    fn run(tree: &mut FlexTree) {
        compute_flex(
            tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            ),
        )
        .unwrap();
    }

    let padding = Edges {
        top: Length::px(2.0),
        right: Length::px(4.0),
        bottom: Length::px(6.0),
        left: Length::px(8.0),
    };
    let border = Edges {
        top: Length::px(1.0),
        right: Length::px(3.0),
        bottom: Length::px(5.0),
        left: Length::px(7.0),
    };

    let mut authored_size = tree_with_child(NodeInput {
        position: Position::Absolute,
        size: Size::new(PreferredSize::px(12.0), PreferredSize::px(12.0)),
        padding,
        border,
        ..NodeInput::default()
    });
    run(&mut authored_size);
    assert_eq!(
        authored_size.inputs(2)[0].known(),
        Size::new(Some(22.0), Some(14.0))
    );
    assert_eq!(
        authored_size
            .layout(2)
            .expect("child layout is staged")
            .size,
        Size::new(22.0, 14.0)
    );

    let mut max_size = tree_with_child(NodeInput {
        position: Position::Absolute,
        max_size: Size::new(MaxSize::px(12.0), MaxSize::px(12.0)),
        padding,
        border,
        ..NodeInput::default()
    });
    run(&mut max_size);
    assert_eq!(
        max_size.layout(2).expect("child layout is staged").size,
        Size::new(22.0, 14.0)
    );
}

#[test]
fn flex_absolute_child_layout_records_scrollbar_size_for_scroll_overflow() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(8.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2)
            .expect("child layout is staged")
            .scrollbar_size(),
        Size::new(8.0, 8.0)
    );
}

#[test]
fn flex_absolute_child_can_resolve_from_trailing_insets() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                right: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(72.0, 34.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn fri07_c01_absolute_auto_margin_original_auto_end_inset_zeroes_inline_margins() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        0.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        0.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
}

#[test]
fn flex_absolute_child_without_insets_uses_flex_alignment() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            justify_content: Some(AlignContent::Center),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(40.0, 15.0)
    );
}

#[test]
fn flex_row_distributes_positive_free_space_with_flex_grow() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(200.0, 20.0));
    assert_eq!(output.content_size, Size::new(200.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(105.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(105.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(95.0, 20.0)
    );

    assert_eq!(
        tree.inputs(2).last().unwrap().known(),
        Size::new(Some(105.0), Some(20.0))
    );
    assert_eq!(
        tree.inputs(3).last().unwrap().known(),
        Size::new(Some(95.0), Some(20.0))
    );
}

#[test]
fn flex_row_with_grow_sum_below_one_uses_that_fraction_of_free_space() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            flex_grow: FlexGrowOf::try_new(0.5).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::ZERO
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(60.0, 10.0)
    );
}

#[test]
fn flex_row_distributes_negative_free_space_with_flex_shrink() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(70.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert!((tree.layout(2).expect("child layout is staged").size.width - 53.333).abs() < 0.01);
    assert!((tree.layout(3).expect("child layout is staged").location.x - 53.333).abs() < 0.01);
    assert!((tree.layout(3).expect("child layout is staged").size.width - 46.667).abs() < 0.01);
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size.height,
        20.0
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size.height,
        20.0
    );
}

#[test]
fn flex_row_relayouts_content_box_percentage_item_at_shrunk_target() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(730.0), PreferredSize::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(100.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            padding: Edges::all(Length::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(730.0), Some(300.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size.width,
        730.0
    );
    assert_eq!(
        tree.inputs(2)
            .last()
            .expect("child should be laid out")
            .known()
            .width,
        Some(730.0)
    );
}

#[test]
fn flex_row_visible_item_does_not_shrink_below_automatic_min_content_width() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(20.0)),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutput::from_outer_size(Size::new(90.0, 20.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutput::from_outer_size(Size::new(90.0, 20.0)))
                .known(Size::new(Some(90.0), Some(20.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(160.0, 20.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(40.0, 20.0)));

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert!(
        tree.inputs(2).iter().any(|input| {
            input.run_mode() == RunMode::ComputeSize
                && input.available().width == Available::MIN_CONTENT
        }),
        "visible flex item should be measured with min-content for its automatic minimum"
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size.width,
        90.0
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location.x,
        90.0
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size.width,
        40.0
    );
}

#[test]
fn flex_row_with_shrink_sum_below_one_uses_that_fraction_of_negative_free_space() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(10.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            flex_shrink: FlexShrinkOf::try_new(0.5).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(80.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::ZERO
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(90.0, 10.0)
    );
}

#[test]
fn flex_row_wraps_items_into_multiple_lines() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            flex_wrap: FlexWrap::Wrap,
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3, 4] {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 38.0));
    assert_eq!(output.content_size, Size::new(100.0, 38.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 14.0)
    );
    assert_eq!(
        tree.layout(4).expect("child layout is staged").location,
        Point::new(0.0, 28.0)
    );
}

#[test]
fn flex_row_auto_width_wraps_against_definite_available_width() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            flex_wrap: FlexWrap::Wrap,
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3, 4] {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 38.0));
    assert_eq!(output.content_size, Size::new(100.0, 38.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 14.0)
    );
    assert_eq!(
        tree.layout(4).expect("child layout is staged").location,
        Point::new(0.0, 28.0)
    );
}

#[test]
fn flex_order_modified_sequence_precedes_wrapping_and_preserves_source_identity_in_both_scalar_lanes()
 {
    assert_flex_order_modified_sequence_precedes_wrapping::<f32>();
    assert_flex_order_modified_sequence_precedes_wrapping::<f64>();
}

fn assert_flex_order_modified_sequence_precedes_wrapping<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    for (direction, expected_x) in [
        (FlexDirection::Row, [0.0, 30.0, 0.0, 30.0]),
        (FlexDirection::RowReverse, [30.0, 0.0, 30.0, 0.0]),
    ] {
        let item_style = |order| NodeInputOf::<S> {
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.0)),
                PreferredSizeOf::px(S::from_f64(10.0)),
            ),
            item_order: ItemOrder::new(order),
            flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
            ..NodeInputOf::default()
        };
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2, 3, 4, 5, 6])
            .children(1, [])
            .children(2, [])
            .children(3, [])
            .children(4, [])
            .children(5, [])
            .children(6, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    flex_direction: direction,
                    flex_wrap: FlexWrap::Wrap,
                    align_content: Some(AlignContent::FlexStart),
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(60.0)),
                        PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(1, item_style(2))
            .style(
                2,
                NodeInputOf {
                    display: Display::None,
                    item_order: ItemOrder::new(i32::MIN),
                    ..item_style(0)
                },
            )
            .style(3, item_style(-1))
            .style(
                4,
                NodeInputOf {
                    position: Position::Absolute,
                    item_order: ItemOrder::new(i32::MIN),
                    ..item_style(0)
                },
            )
            .style(5, item_style(2))
            .style(6, item_style(0));

        compute_flex(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(60.0)), Some(S::from_f64(20.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(
                    AvailableOf::definite(S::from_f64(60.0)),
                    AvailableOf::definite(S::from_f64(20.0)),
                ),
            ),
        )
        .expect("order-modified wrapped flex layout succeeds");

        for (node, expected_source, x, y) in [
            (3, 2, expected_x[0], 0.0),
            (6, 5, expected_x[1], 0.0),
            (1, 0, expected_x[2], 10.0),
            (5, 4, expected_x[3], 10.0),
        ] {
            let layout = tree.layout(node).expect("in-flow child layout is staged");
            assert_eq!(layout.source_index, SourceIndex::new(expected_source));
            assert_eq!(layout.location, Point::new(S::from_f64(x), S::from_f64(y)));
        }
        assert!(
            tree.inputs(2)
                .iter()
                .any(|input| input.run_mode() == RunMode::PerformHiddenLayout),
            "hidden child scheduling remains outside the in-flow permutation"
        );
        assert_eq!(
            tree.layout(4)
                .expect("absolute child layout is staged")
                .source_index,
            SourceIndex::new(3)
        );
    }
}

#[test]
fn flex_row_justifies_items_on_the_main_axis() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(25.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(25.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn flex_row_aligns_items_on_the_cross_axis() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 15.0)
    );
}

#[test]
fn flex_row_reports_first_child_baseline() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes_and_first_baselines(
            Size::new(20.0, 10.0),
            Size::ZERO,
            Point::new(None, Some(7.0)),
        ),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.first_baselines.y, Some(7.0));
    assert_eq!(output.last_baselines.y, Some(7.0));
}

fn assert_flex_uses_the_orthogonal_child_line_over_margin_for_baselines<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [2])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::Baseline),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(200.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(70.0)),
                    PreferredSizeOf::px(S::from_f64(110.0)),
                ),
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(3.0)),
                    LengthAutoOf::px(S::from_f64(7.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(19.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(110.0)),
                Size::new(S::from_f64(70.0), S::from_f64(110.0)),
                BaselinesOf {
                    first: Point::new(Some(S::from_f64(17.0)), None),
                    last: Point::new(Some(S::from_f64(29.0)), None),
                },
            ),
        );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(200.0)), Some(S::from_f64(160.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(200.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("flex layout succeeds");

    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(36.0)), None)
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(S::from_f64(36.0)), None)
    );
}

#[test]
fn orthogonal_baseline_flex_uses_line_over_margin_for_f32() {
    assert_flex_uses_the_orthogonal_child_line_over_margin_for_baselines::<f32>();
}

#[test]
fn orthogonal_baseline_flex_uses_line_over_margin_for_f64() {
    assert_flex_uses_the_orthogonal_child_line_over_margin_for_baselines::<f64>();
}

fn assert_flex_translates_orthogonal_child_baselines_on_the_child_block_axis<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [2])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::Baseline),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(200.0)),
                    PreferredSizeOf::px(S::from_f64(160.0)),
                ),
                padding: Edges {
                    top: LengthOf::px(S::from_f64(5.0)),
                    left: LengthOf::px(S::from_f64(3.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(70.0)),
                    PreferredSizeOf::px(S::from_f64(110.0)),
                ),
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(17.0)),
                    LengthAutoOf::px(S::from_f64(7.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(110.0)),
                Size::new(S::from_f64(70.0), S::from_f64(110.0)),
                BaselinesOf {
                    first: Point::new(Some(S::from_f64(17.0)), None),
                    last: Point::new(Some(S::from_f64(29.0)), None),
                },
            ),
        );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(200.0)), Some(S::from_f64(160.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(200.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("flex layout succeeds");

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(S::from_f64(14.0), S::from_f64(22.0))
    );
    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(31.0)), None)
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(S::from_f64(31.0)), None)
    );
}

#[test]
fn orthogonal_baseline_flex_translation_uses_physical_x_for_f32() {
    assert_flex_translates_orthogonal_child_baselines_on_the_child_block_axis::<f32>();
}

#[test]
fn orthogonal_baseline_flex_translation_uses_physical_x_for_f64() {
    assert_flex_translates_orthogonal_child_baselines_on_the_child_block_axis::<f64>();
}

fn assert_logical_flex_sizing_orthogonal_refreshes_mapped_main<S: LayoutScalar>(
    container_main: f64,
    child_main: f64,
    expected_child_size: Size<S>,
) {
    let baseline_output = |size| {
        ComputeOutputOf::from_sizes_and_baselines(
            size,
            size,
            BaselinesOf {
                first: Point::new(Some(size.width), None),
                last: Point::new(Some(size.width), None),
            },
        )
    };
    let initial_main = S::from_f64(child_main);
    let initial_size = Size::new(initial_main / S::from_f64(2.0), initial_main);
    let mut tree = OracleTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalRl,
                flex_direction: FlexDirection::Row,
                size: Size::new(
                    PreferredSizeOf::AUTO,
                    PreferredSizeOf::px(S::from_f64(container_main)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(initial_main)),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow"),
                flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                ..NodeInputOf::default()
            },
        )
        .measure_when(
            2,
            OracleMeasurementOf::new(baseline_output(expected_child_size)).known(Size::new(
                Some(expected_child_size.width),
                Some(expected_child_size.height),
            )),
        )
        .measure_when(
            2,
            OracleMeasurementOf::new(baseline_output(expected_child_size))
                .known(Size::new(None, Some(expected_child_size.height))),
        )
        .measure(2, baseline_output(initial_size));

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(None, Some(S::from_f64(container_main))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::MAX_CONTENT,
                AvailableOf::definite(S::from_f64(container_main)),
            ),
        ),
    )
    .expect("flex layout succeeds");

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        expected_child_size
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(S::ZERO, S::ZERO),
        "the corrected mapped main size reaches final physical placement"
    );
    assert_eq!(
        output.first_baselines,
        Point::new(Some(expected_child_size.width), None),
        "the refreshed vertical child synthesizes a size-dependent physical-x baseline"
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(expected_child_size.width), None),
        "the refreshed vertical child retains the selected physical-x baseline"
    );
}

fn assert_logical_flex_sizing_orthogonal_refresh_grow<S: LayoutScalar>() {
    assert_logical_flex_sizing_orthogonal_refreshes_mapped_main::<S>(
        160.0,
        40.0,
        Size::new(S::from_f64(80.0), S::from_f64(160.0)),
    );
}

fn assert_logical_flex_sizing_orthogonal_refresh_shrink<S: LayoutScalar>() {
    assert_logical_flex_sizing_orthogonal_refreshes_mapped_main::<S>(
        100.0,
        160.0,
        Size::new(S::from_f64(50.0), S::from_f64(100.0)),
    );
}

#[test]
fn logical_flex_sizing_orthogonal_refresh_grow_for_f32() {
    assert_logical_flex_sizing_orthogonal_refresh_grow::<f32>();
}

#[test]
fn logical_flex_sizing_orthogonal_refresh_grow_for_f64() {
    assert_logical_flex_sizing_orthogonal_refresh_grow::<f64>();
}

#[test]
fn logical_flex_sizing_orthogonal_refresh_shrink_for_f32() {
    assert_logical_flex_sizing_orthogonal_refresh_shrink::<f32>();
}

#[test]
fn logical_flex_sizing_orthogonal_refresh_shrink_for_f64() {
    assert_logical_flex_sizing_orthogonal_refresh_shrink::<f64>();
}

#[test]
fn logical_flex_placement_baseline_refresh_grow_projects_physical_x_for_f32() {
    assert_logical_flex_sizing_orthogonal_refresh_grow::<f32>();
}

#[test]
fn logical_flex_placement_baseline_refresh_grow_projects_physical_x_for_f64() {
    assert_logical_flex_sizing_orthogonal_refresh_grow::<f64>();
}

#[test]
fn logical_flex_placement_baseline_refresh_shrink_projects_physical_x_for_f32() {
    assert_logical_flex_sizing_orthogonal_refresh_shrink::<f32>();
}

#[test]
fn logical_flex_placement_baseline_refresh_shrink_projects_physical_x_for_f64() {
    assert_logical_flex_sizing_orthogonal_refresh_shrink::<f64>();
}

fn assert_logical_flex_final_size_selector_uses_vertical_row_main_axis<S: LayoutScalar>(
    writing_mode: WritingMode,
) {
    let initial_size = Size::new(S::from_f64(75.0), S::from_f64(20.0));
    let final_size = Size::new(S::from_f64(50.0), S::from_f64(20.0));
    let mut tree = OracleTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Flex,
                writing_mode,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(200.0)),
                    PreferredSizeOf::px(S::from_f64(100.0)),
                ),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::percent(S::from_f64(0.25)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                min_size: Size::new(MinSizeOf::px(S::from_f64(75.0)), MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutputOf::from_sizes(final_size, final_size))
                .run_mode(RunMode::PerformLayout),
        )
        .measure(2, ComputeOutputOf::from_sizes(initial_size, initial_size));

    compute_flex(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(200.0)), Some(S::from_f64(100.0))),
            crate::ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(200.0)),
                AvailableOf::definite(S::from_f64(100.0)),
            ),
        ),
    )
    .expect("vertical or sideways flex row layout succeeds");

    assert_eq!(
        tree.inputs(2)
            .iter()
            .rev()
            .find(|input| input.run_mode() == RunMode::PerformLayout)
            .expect("final layout request is recorded")
            .known()
            .width,
        Some(S::from_f64(50.0)),
        "the percentage-dependent physical width is refined after a vertical main-axis row"
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(S::from_f64(50.0), S::from_f64(20.0)),
        "the corrected final known width reaches child output"
    );
}

#[test]
fn logical_flex_placement_final_size_selector_maps_vertical_row_for_f32() {
    assert_logical_flex_final_size_selector_uses_vertical_row_main_axis::<f32>(
        WritingMode::VerticalLr,
    );
}

#[test]
fn logical_flex_placement_final_size_selector_maps_vertical_row_for_f64() {
    assert_logical_flex_final_size_selector_uses_vertical_row_main_axis::<f64>(
        WritingMode::VerticalLr,
    );
}

#[test]
fn logical_flex_placement_final_size_selector_maps_sideways_row_for_f32() {
    assert_logical_flex_final_size_selector_uses_vertical_row_main_axis::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn logical_flex_placement_final_size_selector_maps_sideways_row_for_f64() {
    assert_logical_flex_final_size_selector_uses_vertical_row_main_axis::<f64>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn flex_row_aligns_baseline_items_by_child_baselines() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            align_items: Some(AlignItems::Baseline),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes_and_first_baselines(
            Size::new(20.0, 20.0),
            Size::ZERO,
            Point::new(None, Some(15.0)),
        ),
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes_and_first_baselines(
            Size::new(20.0, 10.0),
            Size::ZERO,
            Point::new(None, Some(5.0)),
        ),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location.y,
        0.0
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location.y,
        10.0
    );
    assert_eq!(output.first_baselines.y, Some(15.0));
}

#[test]
fn flex_row_stretches_auto_cross_size_items() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 40.0)
    );
    assert_eq!(
        tree.inputs(2).last().unwrap().known(),
        Size::new(Some(20.0), Some(40.0))
    );
}

#[test]
fn flex_row_stretch_transfers_cross_size_through_aspect_ratio() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(50.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            aspect_ratio: AspectRatio::new(2.0),
            flex_grow: FlexGrowOf::try_new(0.0).unwrap(),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 50.0)
    );
    assert_eq!(
        tree.inputs(2).last().unwrap().known(),
        Size::new(Some(100.0), Some(50.0))
    );
}

#[test]
fn flex_row_stretched_aspect_ratio_item_does_not_shrink_below_transferred_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            min_size: Size::new(MinSize::AUTO, MinSize::px(40.0)),
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(200.0, 100.0)
    );
}

#[test]
fn flex_replaced_automatic_minimum_selects_smaller_suggestion_and_preserves_cross_stretch_in_both_scalar_lanes()
 {
    assert_flex_replaced_automatic_minimum_selects_smaller_suggestion::<f32>();
    assert_flex_replaced_automatic_minimum_selects_smaller_suggestion::<f64>();
}

fn assert_flex_replaced_automatic_minimum_selects_smaller_suggestion<S: LayoutScalar>() {
    let mut widths = Vec::new();
    let mut heights = Vec::new();
    for item_is_replaced in [true, false] {
        let mut tree = FlexTree::default();
        tree.insert_children(1, [2]);
        tree.insert_children(2, []);
        tree.insert_style(
            1,
            NodeInputOf {
                display: Display::Flex,
                align_items: Some(AlignItems::Stretch),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(60.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );
        tree.insert_style(
            2,
            NodeInputOf {
                item_is_replaced,
                aspect_ratio: AspectRatioOf::new(S::from_f64(2.0)),
                flex_basis: FlexBasisOf::px(S::from_f64(90.0)),
                flex_grow: FlexGrowOf::try_new(S::ZERO).expect("zero is a valid flex grow"),
                flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                ..NodeInputOf::default()
            },
        );
        let expected_width = if item_is_replaced {
            S::from_f64(60.0)
        } else {
            S::from_f64(90.0)
        };
        tree = tree
            .measure_when(
                2,
                OracleMeasurementOf::new(ComputeOutputOf::from_outer_size(Size::new(
                    expected_width,
                    S::from_f64(20.0),
                )))
                .known(Size::new(Some(expected_width), Some(S::from_f64(20.0)))),
            )
            .measure(
                2,
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(90.0), S::from_f64(10.0))),
            );

        compute_flex(
            &mut tree,
            1,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(60.0)), Some(S::from_f64(20.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(
                    AvailableOf::definite(S::from_f64(60.0)),
                    AvailableOf::definite(S::from_f64(20.0)),
                ),
            ),
        )
        .expect("replaced automatic-minimum flex layout succeeds");

        let layout = tree.layout(2).expect("child layout is staged");
        widths.push(layout.size.width);
        heights.push(layout.size.height);
    }

    assert_eq!(widths, [S::from_f64(60.0), S::from_f64(90.0)]);
    assert_eq!(heights, [S::from_f64(20.0), S::from_f64(20.0)]);
}

#[test]
fn flex_row_aspect_ratio_auto_min_respects_authored_width_cap() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(300.0), PreferredSize::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(100.0)),
            aspect_ratio: AspectRatio::new(2.0),
            flex_grow: FlexGrowOf::try_new(0.0).unwrap(),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::definite(100.0)),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(50.0, 100.0)
    );
}

#[test]
fn flex_row_aligns_wrapped_lines_with_align_content() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Center),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 60.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 18.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 32.0)
    );
}

#[test]
fn flex_column_wrap_with_one_line_honors_align_content_end() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3, 4, 5, 6]);
    for node in 2..=6 {
        tree.insert_children(node, vec![]);
    }
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::End),
            ..NodeInput::default()
        },
    );
    for child in 2..=6 {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 100.0));
    for child in 2..=6 {
        assert_eq!(
            tree.layout(child)
                .expect("child layout is staged")
                .location
                .x,
            50.0
        );
    }
}

#[test]
fn flex_row_stretches_wrapped_lines_with_align_content_stretch() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Stretch),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 60.0));
    assert_eq!(output.content_size, Size::new(100.0, 60.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 32.0)
    );
}

#[test]
fn flex_row_stretched_wrapped_line_stretches_auto_cross_size_item() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Stretch),
            align_items: Some(AlignItems::Stretch),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(80.0, 28.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(80.0, 28.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 32.0)
    );
    assert_eq!(
        tree.inputs(3).last().unwrap().known(),
        Size::new(Some(80.0), Some(28.0))
    );
}

#[test]
fn flex_row_wrap_reverse_places_lines_from_the_reversed_cross_axis() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_content: Some(AlignContent::FlexStart),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 36.0)
    );
}

#[test]
fn flex_row_wrap_reverse_flips_flex_start_item_alignment() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn flex_row_wrap_reverse_respects_reversed_align_content() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_content: Some(AlignContent::FlexEnd),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.insert_style(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 14.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
}

#[test]
fn flex_row_growth_respects_max_main_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            max_size: Size::new(MaxSize::px(60.0), MaxSize::NONE),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(60.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(60.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(140.0, 20.0)
    );
}

#[test]
fn flex_row_distributes_positive_space_to_main_axis_auto_margins() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            margin: Edges::new(
                LengthAuto::ZERO,
                LengthAuto::ZERO,
                LengthAuto::ZERO,
                LengthAuto::AUTO,
            ),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        80.0
    );
}

#[test]
fn flex_row_distributes_cross_axis_auto_margins() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            margin: Edges::new(
                LengthAuto::AUTO,
                LengthAuto::ZERO,
                LengthAuto::AUTO,
                LengthAuto::ZERO,
            ),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 15.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.top,
        15.0
    );
    assert_eq!(
        tree.layout(2)
            .expect("child layout is staged")
            .margin
            .bottom,
        15.0
    );
}

#[test]
fn flex_row_reverse_places_items_from_the_reversed_main_axis() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            flex_direction: FlexDirection::RowReverse,
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn flex_row_rtl_places_items_from_the_right_edge() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn flex_row_rtl_relative_insets_follow_rtl_main_axis() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            inset: Edges {
                left: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            inset: Edges {
                right: LengthAuto::px(7.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(85.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(53.0, 0.0)
    );
}

#[test]
fn flex_column_rtl_aligns_cross_start_to_the_right_edge() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::FlexStart),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            padding: Edges {
                left: Length::px(4.0),
                right: Length::px(6.0),
                top: Length::ZERO,
                bottom: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(74.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn flex_column_rtl_cross_axis_auto_margin_uses_rtl_edges() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::px(3.0),
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        77.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        3.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(3.0, 0.0)
    );
}

#[test]
fn flex_column_reverse_places_items_from_the_reversed_main_axis() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(100.0)),
            flex_direction: FlexDirection::ColumnReverse,
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 80.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 50.0)
    );
}

#[test]
fn flex_row_uses_flex_basis_as_the_main_base_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(10.0)),
            flex_basis: FlexBasis::px(30.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(30.0, 10.0)
    );
    assert_eq!(
        tree.inputs(2).last().unwrap().known(),
        Size::new(Some(30.0), Some(10.0))
    );
}

#[test]
fn flex_row_flex_basis_zero_preserves_padding_border_without_authored_content_width() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(12.0), PreferredSize::px(12.0)),
            flex_basis: FlexBasis::px(0.0),
            padding: Edges {
                left: Length::px(8.0),
                top: Length::px(2.0),
                right: Length::px(4.0),
                bottom: Length::px(6.0),
            },
            border: Edges {
                left: Length::px(7.0),
                top: Length::px(1.0),
                right: Length::px(3.0),
                bottom: Length::px(5.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(12.0), PreferredSize::px(12.0)),
            flex_basis: FlexBasis::px(0.0),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(22.0, 14.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(22.0, 14.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(22.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(0.0, 12.0)
    );
    assert_eq!(
        tree.inputs(2).last().unwrap().known(),
        Size::new(Some(22.0), Some(14.0))
    );
}

#[test]
fn flex_row_flex_basis_padding_floor_preserves_leaf_content_intrinsic_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            flex_basis: FlexBasis::px(0.0),
            padding: Edges {
                left: Length::px(10.0),
                right: Length::px(10.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(0.0, 10.0), Size::new(120.0, 10.0)),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size.width, 120.0);
    assert_eq!(output.content_size.width, 120.0);
    assert_eq!(
        tree.layout(2)
            .expect("child layout is staged")
            .content_size
            .width,
        120.0
    );
}

use crate::{LengthPercentageOf, NodeInput, PreferredSize};

#[test]
fn flex_percent_dependent_affine_size_requests_definite_cross_rerun() {
    let height = LengthPercentageOf::from_coefficients(10.0, 0.50).expect("finite coefficients");
    let mut child = NodeInput::default();
    child.size.height = PreferredSize::value(height);

    assert!(child.size.height.depends_on_basis());
}

fn fri05_c04_flex_all_flow_axes() -> [FlowAxes; 10] {
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

fn fri05_c04_flex_overflow_at_flow_axes(
    flow_axes: FlowAxes,
    inline: Overflow,
    block: Overflow,
) -> ComputedOverflow {
    match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => computed_overflow(inline, block),
        PhysicalAxis::Vertical => computed_overflow(block, inline),
    }
}

fn fri05_c04_flex_input(size: Size<f32>, flow_axes: FlowAxes) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        size.map(Some),
        size.map(Some),
        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
        size.map(Available::definite),
    )
}

fn fri05_c04_empty_flex_output(style: NodeInput, size: Size<f32>) -> ComputeOutput {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [])
        .style(0, style);
    compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
        .expect("FRI-05 empty flex layout succeeds")
}

fn fri05_c04_flex_gutter_at(
    gutters: ScrollbarGutterRects,
    side: PhysicalSide,
) -> Option<ScrollRect> {
    match side {
        PhysicalSide::Top => gutters.top(),
        PhysicalSide::Right => gutters.right(),
        PhysicalSide::Bottom => gutters.bottom(),
        PhysicalSide::Left => gutters.left(),
    }
}

fn fri05_c04_assert_zero_range(geometry: ScrollGeometry, context: &str) {
    let range = geometry.physical_range();
    assert_eq!(
        (
            range.x().minimum(),
            range.x().maximum(),
            range.y().minimum(),
            range.y().maximum(),
        ),
        (0.0, 0.0, 0.0, 0.0),
        "{context}"
    );
}

#[test]
fn fri05_c04_flex_geometry_empty_and_simple_nonoverflowing_publish_canonical_boxes_all_flows() {
    let size = Size::new(100.0, 80.0);
    let border = Edges::all(Length::px(2.0));
    let padding = Edges::all(Length::px(3.0));
    let scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let expected_border_box = ScrollRect::try_new(Point::ZERO, size).unwrap();
    let expected_padding_box =
        ScrollRect::try_new(Point::new(2.0, 2.0), Size::new(96.0, 76.0)).unwrap();
    let expected_content_box =
        ScrollRect::try_new(Point::new(5.0, 5.0), Size::new(90.0, 70.0)).unwrap();

    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let style = NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            border,
            padding,
            scroll_margin,
            scroll_snap_align: snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInput::default()
        };
        let output = fri05_c04_empty_flex_output(style.clone(), size);
        let geometry = output
            .scroll_geometry
            .expect("performed empty flex emits canonical geometry");

        assert_eq!(geometry.flow_axes(), flow_axes);
        assert_eq!(geometry.used_overflow_x(), Overflow::Visible);
        assert_eq!(geometry.used_overflow_y(), Overflow::Visible);
        assert_eq!(geometry.border_box(), expected_border_box);
        assert_eq!(geometry.padding_box(), expected_padding_box);
        assert_eq!(geometry.content_box(), expected_content_box);
        assert_eq!(geometry.scrollport(), expected_padding_box);
        assert_eq!(geometry.scrollable_overflow(), expected_padding_box);
        assert_eq!(geometry.overflow_clip().x(), None);
        assert_eq!(geometry.overflow_clip().y(), None);
        assert_eq!(geometry.scrollbar_size(), Size::ZERO);
        assert_eq!(geometry.target().border_box(), expected_border_box);
        assert_eq!(geometry.target().flow_axes(), flow_axes);
        assert_eq!(geometry.target().scroll_margin(), scroll_margin);
        assert_eq!(geometry.target().snap_align(), snap_align);
        assert_eq!(geometry.target().snap_stop(), ScrollSnapStop::Always);
        fri05_c04_assert_zero_range(geometry, &format!("empty {flow_axes:?}"));

        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(0, style)
            .style(
                1,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(8.0)),
                    min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                    ..NodeInput::default()
                },
            );
        let simple = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
            .expect("FRI-05 simple flex layout succeeds");
        let simple_geometry = simple
            .scroll_geometry
            .expect("performed simple flex emits canonical geometry");
        assert_eq!(simple_geometry.border_box(), expected_border_box);
        assert_eq!(simple_geometry.padding_box(), expected_padding_box);
        assert_eq!(simple_geometry.content_box(), expected_content_box);
        assert_eq!(simple_geometry.scrollport(), expected_padding_box);
        assert_eq!(simple_geometry.scrollable_overflow(), expected_padding_box);
        assert_eq!(simple_geometry.target().border_box(), expected_border_box);
        fri05_c04_assert_zero_range(simple_geometry, &format!("simple {flow_axes:?}"));
    }
}

#[test]
fn fri05_c04_flex_geometry_forced_stable_both_zero_and_tiny_saturate_all_flows() {
    let size = Size::new(100.0, 80.0);
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let style = |overflow, gutter, width| NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_gutter: gutter,
            scrollbar_width: ScrollbarWidth::try_new(width).unwrap(),
            size: Size::new(
                PreferredSize::px(size.width),
                PreferredSize::px(size.height),
            ),
            ..NodeInput::default()
        };
        let forced = fri05_c04_empty_flex_output(
            style(
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Scroll),
                ScrollbarGutter::Auto,
                7.0,
            ),
            size,
        )
        .scroll_geometry
        .expect("forced-scroll flex emits geometry");
        let stable = fri05_c04_empty_flex_output(
            style(
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
                ScrollbarGutter::Stable,
                7.0,
            ),
            size,
        )
        .scroll_geometry
        .expect("stable-gutter flex emits geometry");
        let both = fri05_c04_empty_flex_output(
            style(
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
                ScrollbarGutter::StableBothEdges,
                7.0,
            ),
            size,
        )
        .scroll_geometry
        .expect("both-edge flex emits geometry");

        for (case, geometry, expected_sides) in [
            ("forced", forced, vec![flow_axes.inline_end()]),
            ("stable", stable, vec![flow_axes.inline_end()]),
            (
                "both",
                both,
                vec![flow_axes.inline_start(), flow_axes.inline_end()],
            ),
        ] {
            assert_eq!(geometry.flow_axes(), flow_axes, "{case}/{flow_axes:?}");
            assert_eq!(geometry.border_box(), geometry.padding_box());
            assert_eq!(geometry.content_box(), geometry.scrollport());
            let scrollport = geometry.scrollport();
            let x_clip = geometry.overflow_clip().x().expect("x clip is present");
            let y_clip = geometry.overflow_clip().y().expect("y clip is present");
            assert_eq!(
                (x_clip.minimum(), x_clip.maximum()),
                (
                    scrollport.origin().x,
                    scrollport.origin().x + scrollport.size().width,
                )
            );
            assert_eq!(
                (y_clip.minimum(), y_clip.maximum()),
                (
                    scrollport.origin().y,
                    scrollport.origin().y + scrollport.size().height,
                )
            );
            assert_eq!(geometry.target().border_box(), geometry.border_box());
            assert_eq!(geometry.target().flow_axes(), flow_axes);
            for side in [
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
            ] {
                assert_eq!(
                    fri05_c04_flex_gutter_at(geometry.gutters(), side).is_some(),
                    expected_sides.contains(&side),
                    "unexpected {side:?} gutter for {case}/{flow_axes:?}: {geometry:#?}"
                );
            }
            fri05_c04_assert_zero_range(geometry, &format!("{case} {flow_axes:?}"));
        }

        let expected_one_edge = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(7.0, 0.0),
            PhysicalAxis::Vertical => Size::new(0.0, 7.0),
        };
        assert_eq!(forced.scrollbar_size(), expected_one_edge, "{flow_axes:?}");
        assert_eq!(stable.scrollbar_size(), expected_one_edge, "{flow_axes:?}");
        assert_eq!(both.scrollbar_size(), expected_one_edge + expected_one_edge);

        let zero_width = fri05_c04_empty_flex_output(
            style(
                computed_overflow(Overflow::Scroll, Overflow::Scroll),
                ScrollbarGutter::StableBothEdges,
                0.0,
            ),
            size,
        )
        .scroll_geometry
        .expect("zero-width scrollbar flex emits geometry");
        assert_eq!(zero_width.scrollbar_size(), Size::ZERO);
        assert_eq!(zero_width.scrollport(), zero_width.padding_box());
        assert_eq!(zero_width.gutters().top(), None);
        assert_eq!(zero_width.gutters().right(), None);
        assert_eq!(zero_width.gutters().bottom(), None);
        assert_eq!(zero_width.gutters().left(), None);
        fri05_c04_assert_zero_range(zero_width, &format!("zero width {flow_axes:?}"));

        let tiny_size = Size::new(5.0, 3.0);
        let tiny = fri05_c04_empty_flex_output(
            NodeInput {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                size: Size::new(
                    PreferredSize::px(tiny_size.width),
                    PreferredSize::px(tiny_size.height),
                ),
                ..NodeInput::default()
            },
            tiny_size,
        )
        .scroll_geometry
        .expect("tiny both-edge flex emits geometry");
        let expected_tiny_reservation = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(tiny_size.width, 0.0),
            PhysicalAxis::Vertical => Size::new(0.0, tiny_size.height),
        };
        assert_eq!(tiny.scrollbar_size(), expected_tiny_reservation);
        assert_eq!(
            match flow_axes.inline_axis() {
                PhysicalAxis::Horizontal => tiny.scrollport().size().width,
                PhysicalAxis::Vertical => tiny.scrollport().size().height,
            },
            0.0,
            "tiny inline scrollport saturates for {flow_axes:?}"
        );
        assert!(fri05_c04_flex_gutter_at(tiny.gutters(), flow_axes.inline_start()).is_some());
        assert!(fri05_c04_flex_gutter_at(tiny.gutters(), flow_axes.inline_end()).is_some());
        fri05_c04_assert_zero_range(tiny, &format!("tiny {flow_axes:?}"));

        let zero_size = Size::ZERO;
        let zero = fri05_c04_empty_flex_output(
            NodeInput {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(0.0)),
                ..NodeInput::default()
            },
            zero_size,
        )
        .scroll_geometry
        .expect("zero-size flex emits ordered geometry");
        assert_eq!(zero.border_box().size(), Size::ZERO);
        assert_eq!(zero.padding_box().size(), Size::ZERO);
        assert_eq!(zero.content_box().size(), Size::ZERO);
        assert_eq!(zero.scrollport().size(), Size::ZERO);
        assert_eq!(zero.scrollbar_size(), Size::ZERO);
        assert_eq!(zero.gutters().top(), None);
        assert_eq!(zero.gutters().right(), None);
        assert_eq!(zero.gutters().bottom(), None);
        assert_eq!(zero.gutters().left(), None);
        fri05_c04_assert_zero_range(zero, &format!("zero box {flow_axes:?}"));
    }
}

fn fri05_c04_child_geometry_source(style: NodeInput, size: Size<f32>) -> ComputeOutput {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(9, [])
        .style(9, style);
    crate::compute_block(&mut tree, 9, fri05_c04_flex_input(size, flow_axes))
        .expect("child geometry source block lays out")
}

#[test]
fn fri05_c04_flex_child_geometry_direct_retains_in_flow_and_rebuilds_absolute_target() {
    let parent_size = Size::new(120.0, 80.0);
    let child_flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let in_flow_scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let in_flow_snap_align =
        ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::End);
    let in_flow_style = NodeInput {
        display: Display::Block,
        writing_mode: child_flow_axes.writing_mode(),
        direction: child_flow_axes.direction(),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        scrollbar_gutter: ScrollbarGutter::StableBothEdges,
        scrollbar_width: ScrollbarWidth::try_new(4.0).unwrap(),
        size: Size::new(PreferredSize::px(24.0), PreferredSize::px(18.0)),
        min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
        scroll_margin: in_flow_scroll_margin,
        scroll_snap_align: in_flow_snap_align,
        scroll_snap_stop: ScrollSnapStop::Always,
        ..NodeInput::default()
    };
    let absolute_size = Size::new(30.0, 20.0);
    let current_absolute_scroll_margin = ScrollMargin::try_new(8.0, 7.0, 6.0, 5.0).unwrap();
    let absolute_style = NodeInput {
        position: Position::Absolute,
        size: Size::new(
            PreferredSize::px(absolute_size.width),
            PreferredSize::px(absolute_size.height),
        ),
        inset: Edges::new(
            LengthAuto::px(3.0),
            LengthAuto::AUTO,
            LengthAuto::AUTO,
            LengthAuto::px(5.0),
        ),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        scrollbar_width: ScrollbarWidth::try_new(3.0).unwrap(),
        scroll_margin: current_absolute_scroll_margin,
        ..NodeInput::default()
    };
    let retained_absolute_scroll_margin = ScrollMargin::try_new(-5.0, 4.0, -3.0, 2.0).unwrap();
    let retained_absolute_snap_align =
        ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let stale_absolute = fri05_c04_child_geometry_source(
        NodeInput {
            position: Position::Relative,
            scroll_margin: retained_absolute_scroll_margin,
            scroll_snap_align: retained_absolute_snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..absolute_style.clone()
        },
        Size::new(10.0, 8.0),
    );
    let stale_border_box = stale_absolute
        .scroll_geometry
        .expect("source output has geometry")
        .border_box();

    let parent_flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: Size::new(
                    PreferredSize::px(parent_size.width),
                    PreferredSize::px(parent_size.height),
                ),
                ..NodeInput::default()
            },
        )
        .style(1, in_flow_style)
        .style(2, absolute_style)
        .measure(2, stale_absolute);
    compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(parent_size, parent_flow_axes),
    )
    .expect("flex child geometry layout succeeds");

    let in_flow = tree.layout(1).expect("in-flow child is staged");
    let in_flow_geometry = in_flow
        .scroll_geometry
        .expect("in-flow child retains canonical geometry");
    assert_eq!(in_flow_geometry.border_box().size(), in_flow.size);
    assert_eq!(
        in_flow_geometry.target().border_box(),
        in_flow_geometry.border_box()
    );
    assert_eq!(in_flow_geometry.target().flow_axes(), child_flow_axes);
    assert_eq!(
        in_flow_geometry.target().scroll_margin(),
        in_flow_scroll_margin
    );
    assert_eq!(in_flow_geometry.target().snap_align(), in_flow_snap_align);
    assert_eq!(
        in_flow_geometry.target().snap_stop(),
        ScrollSnapStop::Always
    );
    assert_eq!(in_flow.scrollbar_size(), in_flow_geometry.scrollbar_size());

    let absolute = tree.layout(2).expect("absolute child is staged");
    let absolute_geometry = absolute
        .scroll_geometry
        .expect("absolute child retains canonical geometry");
    assert_ne!(absolute_geometry.border_box(), stale_border_box);
    assert_eq!(absolute.size, absolute_size);
    assert_eq!(absolute_geometry.border_box().size(), absolute_size);
    assert_eq!(
        absolute_geometry.target().border_box(),
        absolute_geometry.border_box()
    );
    assert_eq!(
        absolute_geometry.target().scroll_margin(),
        retained_absolute_scroll_margin
    );
    assert_ne!(
        absolute_geometry.target().scroll_margin(),
        current_absolute_scroll_margin
    );
    assert_eq!(
        absolute_geometry.target().snap_align(),
        retained_absolute_snap_align
    );
    assert_eq!(
        absolute_geometry.target().snap_stop(),
        ScrollSnapStop::Always
    );
    assert_eq!(
        absolute.scrollbar_size(),
        absolute_geometry.scrollbar_size()
    );
}

fn fri05_c04_flex_child_geometry_tiny_absolute_styles(
    flow_axes: FlowAxes,
) -> (NodeInput, NodeInput) {
    (
        NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            flex_direction: FlexDirection::Column,
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            max_size: Size::new(MaxSize::NONE, MaxSize::px(5.0)),
            ..NodeInput::default()
        },
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(0.0), PreferredSize::px(0.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            inset: Edges::new(
                LengthAuto::AUTO,
                LengthAuto::AUTO,
                LengthAuto::px(0.0),
                LengthAuto::AUTO,
            ),
            ..NodeInput::default()
        },
    )
}

#[test]
fn fri05_c04_flex_child_geometry_direct_auto_max_tiny_gutter_keeps_absolute_inputs_non_negative_all_flows()
 {
    let available_size = Size::new(100.0, 100.0);

    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let (root_style, absolute_style) =
            fri05_c04_flex_child_geometry_tiny_absolute_styles(flow_axes);
        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(0, root_style)
            .style(1, absolute_style);
        let output = compute_flex(
            &mut tree,
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                available_size.map(Some),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                available_size.map(Available::definite),
            ),
        )
        .unwrap_or_else(|error| panic!("tiny absolute flex succeeds for {flow_axes:?}: {error:?}"));

        assert_eq!(output.size, Size::new(100.0, 5.0), "{flow_axes:?}");
        let root_geometry = output
            .scroll_geometry
            .expect("performed flex retains final canonical geometry");
        assert_eq!(
            root_geometry.scrollport().size(),
            Size::new(90.0, 0.0),
            "{flow_axes:?}"
        );

        let absolute = tree
            .layout(1)
            .expect("tiny absolute child is staged without a negative basis");
        let absolute_geometry = absolute
            .scroll_geometry
            .expect("tiny absolute child retains canonical geometry");
        assert_eq!(absolute.size, Size::ZERO, "{flow_axes:?}");
        assert_eq!(absolute_geometry.border_box().size(), Size::ZERO);
        assert_eq!(
            absolute_geometry.target().border_box(),
            absolute_geometry.border_box()
        );
        assert_eq!(
            absolute.location.y,
            root_geometry.scrollport().origin().y + root_geometry.scrollport().size().height,
            "bottom: 0 uses the final saturated scrollport for {flow_axes:?}"
        );

        let child_input = tree
            .inputs(1)
            .iter()
            .find(|input| input.run_mode() == RunMode::PerformLayout)
            .expect("absolute child receives a perform-layout request");
        assert_eq!(
            child_input.parent(),
            root_geometry.content_box().size().map(Some),
            "final canonical content-box basis for {flow_axes:?}"
        );
        assert_eq!(
            child_input.available(),
            root_geometry.scrollport().size().map(Available::definite),
            "final canonical available space for {flow_axes:?}"
        );

        let mut ordinary_root = fri05_c04_flex_child_geometry_tiny_absolute_styles(flow_axes).0;
        ordinary_root.size.height = PreferredSize::px(80.0);
        ordinary_root.max_size.height = MaxSize::NONE;
        let ordinary_absolute = fri05_c04_flex_child_geometry_tiny_absolute_styles(flow_axes).1;
        let mut ordinary_tree = crate::test_support::layout_tree::OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(0, ordinary_root)
            .style(1, ordinary_absolute);
        let ordinary = compute_flex(
            &mut ordinary_tree,
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                available_size.map(Some),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                available_size.map(Available::definite),
            ),
        )
        .unwrap_or_else(|error| {
            panic!("ordinary absolute flex succeeds for {flow_axes:?}: {error:?}")
        });
        let ordinary_geometry = ordinary
            .scroll_geometry
            .expect("ordinary flex retains canonical geometry");
        let ordinary_child = ordinary_tree
            .layout(1)
            .expect("ordinary absolute child remains staged");
        assert_eq!(ordinary.size, Size::new(100.0, 80.0), "{flow_axes:?}");
        assert_eq!(
            ordinary_child.location.y,
            ordinary_geometry.scrollport().origin().y
                + ordinary_geometry.scrollport().size().height,
            "ordinary bottom placement remains on the settled scrollport for {flow_axes:?}"
        );
    }
}

fn fri05_c04_positive_margin_rect(output: NodeOutput) -> ScrollRect {
    let top = output.margin.top.max(0.0);
    let right = output.margin.right.max(0.0);
    let bottom = output.margin.bottom.max(0.0);
    let left = output.margin.left.max(0.0);
    ScrollRect::try_new(
        Point::new(output.location.x - left, output.location.y - top),
        Size::new(
            output.size.width + left + right,
            output.size.height + top + bottom,
        ),
    )
    .unwrap()
}

fn fri05_c04_union_rects(rects: impl IntoIterator<Item = ScrollRect>) -> ScrollRect {
    let mut rects = rects.into_iter();
    let first = rects.next().expect("the test union is nonempty");
    let mut minimum = first.origin();
    let mut maximum = Point::new(
        first.origin().x + first.size().width,
        first.origin().y + first.size().height,
    );
    for rect in rects {
        minimum.x = minimum.x.min(rect.origin().x);
        minimum.y = minimum.y.min(rect.origin().y);
        maximum.x = maximum.x.max(rect.origin().x + rect.size().width);
        maximum.y = maximum.y.max(rect.origin().y + rect.size().height);
    }
    ScrollRect::try_new(
        minimum,
        Size::new(maximum.x - minimum.x, maximum.y - minimum.y),
    )
    .unwrap()
}

#[test]
fn fri05_c04_flex_contribution_positive_outsets_negative_margins_and_source_order_are_exact() {
    let size = Size::new(10.0, 10.0);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: size.map(PreferredSize::px),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                item_order: ItemOrder::new(10),
                size: Size::new(PreferredSize::px(7.0), PreferredSize::px(4.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                margin: Edges::new(
                    LengthAuto::px(3.0),
                    LengthAuto::px(5.0),
                    LengthAuto::px(2.0),
                    LengthAuto::px(4.0),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                item_order: ItemOrder::new(-10),
                size: Size::new(PreferredSize::px(6.0), PreferredSize::px(3.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                margin: Edges::new(
                    LengthAuto::px(-7.0),
                    LengthAuto::px(-11.0),
                    LengthAuto::px(-5.0),
                    LengthAuto::px(-13.0),
                ),
                ..NodeInput::default()
            },
        );

    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            size,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("flex contribution layout succeeds");
    let first = tree.layout(1).expect("first source output is retained");
    let second = tree.layout(2).expect("second source output is retained");
    assert_eq!(first.source_index, SourceIndex::new(0));
    assert_eq!(second.source_index, SourceIndex::new(1));

    let expected = fri05_c04_union_rects([
        ScrollRect::try_new(Point::ZERO, size).unwrap(),
        fri05_c04_positive_margin_rect(first),
        fri05_c04_positive_margin_rect(second),
    ]);
    let geometry = output.scroll_geometry.expect("flex geometry is present");
    assert_eq!(geometry.scrollable_overflow(), expected);
    let expected_maximum = Point::new(
        expected.origin().x + expected.size().width,
        expected.origin().y + expected.size().height,
    );
    assert_eq!(
        output.content_size,
        Size::new(
            expected_maximum.x.max(0.0) - expected.origin().x.min(0.0),
            expected_maximum.y.max(0.0) - expected.origin().y.min(0.0),
        ),
        "negative starts and positive ends remain independent"
    );
}

#[test]
fn fri05_c04_flex_contribution_terminal_padding_extends_only_the_final_in_flow_ends() {
    let size = Size::new(10.0, 8.0);
    let padding = Edges {
        right: Length::px(4.0),
        bottom: Length::px(3.0),
        ..Edges::all(Length::ZERO)
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: size.map(PreferredSize::px),
                padding,
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(12.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            size,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("terminal-padding flex layout succeeds");
    let child = tree.layout(1).unwrap();
    let overflow = output.scroll_geometry.unwrap().scrollable_overflow();

    assert_eq!(overflow.origin(), Point::ZERO);
    assert_eq!(
        overflow.size().width,
        child.location.x + child.size.width + 4.0
    );
    assert_eq!(
        overflow.size().height,
        child.location.y + child.size.height + 3.0
    );
}

fn fri05_c04_flex_nested_output(
    overflow: ComputedOverflow,
    child_size: Size<f32>,
) -> ComputeOutput {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: Size::ZERO.map(PreferredSize::px),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow,
                size: child_size.map(PreferredSize::px),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                align_self: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(30.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                ..NodeInput::default()
            },
        );
    compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            Size::ZERO,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("nested flex contribution layout succeeds")
}

#[test]
fn fri05_c04_flex_nested_visible_and_trapped_axes_preserve_zero_area_intervals_independently() {
    for (overflow, child_size, expected) in [
        (
            computed_overflow(Overflow::Visible, Overflow::Clip),
            Size::new(0.0, 5.0),
            Size::new(20.0, 0.0),
        ),
        (
            computed_overflow(Overflow::Clip, Overflow::Visible),
            Size::new(5.0, 0.0),
            Size::new(0.0, 30.0),
        ),
        (
            computed_overflow(Overflow::Clip, Overflow::Clip),
            Size::new(0.0, 5.0),
            Size::ZERO,
        ),
        (
            computed_overflow(Overflow::Hidden, Overflow::Scroll),
            Size::new(0.0, 5.0),
            Size::ZERO,
        ),
        (
            computed_overflow(Overflow::Scroll, Overflow::Auto),
            Size::new(5.0, 0.0),
            Size::ZERO,
        ),
        (
            computed_overflow(Overflow::Auto, Overflow::Hidden),
            Size::new(5.0, 0.0),
            Size::ZERO,
        ),
    ] {
        let output = fri05_c04_flex_nested_output(overflow, child_size);
        let geometry = output
            .scroll_geometry
            .expect("nested flex geometry is present");
        assert_eq!(geometry.scrollable_overflow().origin(), Point::ZERO);
        assert_eq!(
            geometry.scrollable_overflow().size(),
            expected,
            "{overflow:?}"
        );
        assert_eq!(output.content_size, expected, "{overflow:?}");
    }
}

#[test]
fn fri05_c04_flex_absolute_margin_and_visible_descendant_contribute_once_without_terminal_padding()
{
    let size = Size::ZERO;
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: size.map(PreferredSize::px),
                padding: Edges {
                    right: Length::px(4.0),
                    bottom: Length::px(3.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                overflow: ComputedOverflow::VISIBLE,
                size: Size::new(PreferredSize::px(5.0), PreferredSize::px(5.0)),
                inset: Edges::new(
                    LengthAuto::px(0.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(10.0),
                ),
                margin: Edges {
                    right: LengthAuto::px(7.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(9.0), PreferredSize::px(12.0)),
                ..NodeInput::default()
            },
        );
    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            size,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("absolute flex contribution layout succeeds");
    let absolute = tree.layout(1).expect("absolute output is retained");
    let own_margin = fri05_c04_positive_margin_rect(absolute);
    let own_max_x = own_margin.origin().x + own_margin.size().width;
    let geometry = output
        .scroll_geometry
        .expect("absolute flex geometry is present");

    assert_eq!(geometry.scrollable_overflow().origin(), Point::ZERO);
    assert_eq!(geometry.scrollable_overflow().size().width, own_max_x);
    assert_eq!(geometry.scrollable_overflow().size().height, 12.0);
    assert_eq!(output.content_size, geometry.scrollable_overflow().size());
    assert_ne!(geometry.scrollable_overflow().size().width, own_max_x + 4.0);
}

fn fri05_c04_flex_origin_output(
    flow_axes: FlowAxes,
    direction: FlexDirection,
    wrap: FlexWrap,
) -> (ScrollGeometry, ScrollGeometry) {
    let axes = FlexAxes::new(flow_axes, direction, wrap);
    let size = axes.size_from_main_cross(100.0, 80.0);
    let child_size = axes.size_from_main_cross(140.0, 60.0);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: fri05_c04_flex_overflow_at_flow_axes(
                    flow_axes,
                    Overflow::Scroll,
                    Overflow::Scroll,
                ),
                size: size.map(PreferredSize::px),
                flex_direction: direction,
                flex_wrap: wrap,
                align_content: Some(AlignContent::FlexStart),
                align_items: Some(AlignItems::FlexStart),
                justify_content: Some(AlignContent::FlexStart),
                ..NodeInput::default()
            },
        );
    for child in [1, 2] {
        tree = tree.style(
            child,
            NodeInput {
                size: child_size.map(PreferredSize::px),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
        .expect("origin-aware flex layout succeeds");
    let unrounded = output
        .scroll_geometry
        .expect("performed flex layout has geometry");
    tree.set_unrounded(
        0,
        NodeOutput {
            size: output.size,
            content_size: output.content_size,
            ..NodeOutput::new()
        }
        .with_scroll_geometry(Some(unrounded)),
    );
    crate::round_layout(&mut tree, 0).expect("canonical flex geometry rounds");
    let rounded = tree
        .final_layout(0)
        .and_then(|output| output.scroll_geometry)
        .expect("rounded flex geometry is retained");
    (unrounded, rounded)
}

fn fri05_c04_assert_flow_range(
    geometry: ScrollGeometry,
    flow_axes: FlowAxes,
    inline: (f32, f32),
    block: (f32, f32),
    context: &str,
) {
    let expected = FlowRelativeScrollRange::try_new(inline.0, inline.1, block.0, block.1)
        .expect("expected flow range is ordered");
    assert_eq!(
        geometry.physical_range(),
        flow_axes.physical_scroll_range(expected),
        "{context}"
    );
    assert_eq!(
        flow_axes.flow_relative_scroll_range(geometry.physical_range()),
        expected,
        "{context}"
    );
}

#[test]
fn fri05_c04_flex_origin_main_cross_progressions_project_all_flows_before_and_after_rounding() {
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        for direction in [
            FlexDirection::Row,
            FlexDirection::RowReverse,
            FlexDirection::Column,
            FlexDirection::ColumnReverse,
        ] {
            for wrap in [FlexWrap::Wrap, FlexWrap::WrapReverse] {
                let main = if direction.is_reverse() {
                    (-40.0, 0.0)
                } else {
                    (0.0, 40.0)
                };
                let cross = if wrap == FlexWrap::WrapReverse {
                    (-40.0, 0.0)
                } else {
                    (0.0, 40.0)
                };
                let (inline, block) = if direction.is_row() {
                    (main, cross)
                } else {
                    (cross, main)
                };
                let context = format!("{flow_axes:?} {direction:?} {wrap:?}");
                let (unrounded, rounded) = fri05_c04_flex_origin_output(flow_axes, direction, wrap);
                fri05_c04_assert_flow_range(unrounded, flow_axes, inline, block, &context);
                fri05_c04_assert_flow_range(rounded, flow_axes, inline, block, &context);
            }
        }
    }
}

fn fri05_c04_flex_alignment_output(
    justify_content: Option<AlignContent>,
    align_content: Option<AlignContent>,
    wrap: FlexWrap,
    child_sizes: &[Size<f32>],
) -> (ComputeOutput, crate::test_support::layout_tree::OracleTree) {
    let size = Size::new(100.0, 80.0);
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let children = (1..=u32::try_from(child_sizes.len()).unwrap()).collect::<Vec<_>>();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, children.iter().copied())
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                size: size.map(PreferredSize::px),
                flex_wrap: wrap,
                align_content,
                align_items: Some(AlignItems::FlexStart),
                justify_content,
                ..NodeInput::default()
            },
        );
    for (child, child_size) in children.into_iter().zip(child_sizes.iter().copied()) {
        tree = tree.children(child, []).style(
            child,
            NodeInput {
                size: child_size.map(PreferredSize::px),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }
    let output = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
        .expect("alignment-aware flex layout succeeds");
    (output, tree)
}

fn fri05_c04_assert_physical_range(output: ComputeOutput, expected: (f32, f32, f32, f32)) {
    let range = output.scroll_geometry.unwrap().physical_range();
    assert_eq!(
        (
            range.x().minimum(),
            range.x().maximum(),
            range.y().minimum(),
            range.y().maximum(),
        ),
        expected
    );
}

#[test]
fn fri05_c04_flex_alignment_justify_subjects_cover_start_end_center_space_none_and_safe_fallback() {
    for (alignment, expected) in [
        (Some(AlignContent::Start), (0.0, 40.0, 0.0, 0.0)),
        (Some(AlignContent::End), (-40.0, 0.0, 0.0, 0.0)),
        (Some(AlignContent::Center), (-20.0, 20.0, 0.0, 0.0)),
        (None, (0.0, 40.0, 0.0, 0.0)),
        (Some(AlignContent::SafeEnd), (0.0, 40.0, 0.0, 0.0)),
    ] {
        let (output, _) = fri05_c04_flex_alignment_output(
            alignment,
            None,
            FlexWrap::NoWrap,
            &[Size::new(140.0, 20.0)],
        );
        fri05_c04_assert_physical_range(output, expected);
    }

    let (distributed, tree) = fri05_c04_flex_alignment_output(
        Some(AlignContent::SpaceBetween),
        None,
        FlexWrap::NoWrap,
        &[Size::new(20.0, 20.0), Size::new(20.0, 20.0)],
    );
    fri05_c04_assert_physical_range(distributed, (0.0, 0.0, 0.0, 0.0));
    assert_eq!(tree.layout(1).unwrap().location.x, 0.0);
    assert_eq!(tree.layout(2).unwrap().location.x, 80.0);
}

#[test]
fn fri05_c04_flex_alignment_main_subject_includes_positive_margins_and_gaps_once() {
    let size = Size::new(100.0, 80.0);
    let layout = |justify_content, gap, children: &[(f32, Edges<LengthAuto>)]| {
        let child_ids = (1..=u32::try_from(children.len()).unwrap()).collect::<Vec<_>>();
        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(0, child_ids.iter().copied())
            .style(
                0,
                NodeInput {
                    display: Display::Flex,
                    overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                    size: size.map(PreferredSize::px),
                    gap: Size::new(Length::px(gap), Length::ZERO),
                    align_items: Some(AlignItems::FlexStart),
                    justify_content: Some(justify_content),
                    ..NodeInput::default()
                },
            );
        for (child, (width, margin)) in child_ids.into_iter().zip(children.iter().copied()) {
            tree = tree.children(child, []).style(
                child,
                NodeInput {
                    size: Size::new(PreferredSize::px(width), PreferredSize::px(20.0)),
                    min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                    flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                    margin,
                    ..NodeInput::default()
                },
            );
        }
        compute_flex(
            &mut tree,
            0,
            fri05_c04_flex_input(
                size,
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ),
        )
        .expect("margin-aware alignment layout succeeds")
    };

    let start_margin = layout(
        AlignContent::End,
        0.0,
        &[(
            120.0,
            Edges {
                left: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::ZERO)
            },
        )],
    );
    fri05_c04_assert_physical_range(start_margin, (-40.0, 0.0, 0.0, 0.0));

    let end_margin = layout(
        AlignContent::Start,
        0.0,
        &[(
            120.0,
            Edges {
                right: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::ZERO)
            },
        )],
    );
    fri05_c04_assert_physical_range(end_margin, (0.0, 40.0, 0.0, 0.0));

    let gap = layout(
        AlignContent::End,
        20.0,
        &[
            (
                40.0,
                Edges {
                    left: LengthAuto::px(10.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
            ),
            (
                40.0,
                Edges {
                    right: LengthAuto::px(10.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
            ),
        ],
    );
    fri05_c04_assert_physical_range(gap, (-20.0, 0.0, 0.0, 0.0));
}

#[test]
fn fri05_c04_flex_alignment_align_content_records_only_applicable_multiline_line_subject() {
    let (inapplicable, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::End),
        FlexWrap::NoWrap,
        &[Size::new(20.0, 120.0)],
    );
    fri05_c04_assert_physical_range(inapplicable, (0.0, 0.0, 0.0, 40.0));

    let (wrapped_single_line, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::End),
        FlexWrap::Wrap,
        &[Size::new(20.0, 20.0), Size::new(20.0, 20.0)],
    );
    fri05_c04_assert_physical_range(wrapped_single_line, (0.0, 0.0, 0.0, 0.0));

    let (empty_wrapped, _) =
        fri05_c04_flex_alignment_output(None, Some(AlignContent::End), FlexWrap::Wrap, &[]);
    fri05_c04_assert_physical_range(empty_wrapped, (0.0, 0.0, 0.0, 0.0));

    let (oversized_single_line, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::End),
        FlexWrap::Wrap,
        &[Size::new(20.0, 120.0)],
    );
    fri05_c04_assert_physical_range(oversized_single_line, (0.0, 0.0, 0.0, 0.0));

    let multiline_sizes = [Size::new(60.0, 60.0), Size::new(60.0, 60.0)];
    let (applicable, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::End),
        FlexWrap::Wrap,
        &multiline_sizes,
    );
    fri05_c04_assert_physical_range(applicable, (0.0, 0.0, -40.0, 0.0));

    let (safe, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::SafeEnd),
        FlexWrap::Wrap,
        &multiline_sizes,
    );
    fri05_c04_assert_physical_range(safe, (0.0, 0.0, 0.0, 40.0));
}

#[test]
fn fri05_c04_flex_alignment_main_subject_projects_all_flows_and_orientations() {
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        for direction in [
            FlexDirection::Row,
            FlexDirection::RowReverse,
            FlexDirection::Column,
            FlexDirection::ColumnReverse,
        ] {
            let axes = FlexAxes::new(flow_axes, direction, FlexWrap::NoWrap);
            let size = axes.size_from_main_cross(100.0, 80.0);
            let child_size = axes.size_from_main_cross(140.0, 20.0);
            let mut tree = crate::test_support::layout_tree::OracleTree::new()
                .children(0, [1])
                .children(1, [])
                .style(
                    0,
                    NodeInput {
                        display: Display::Flex,
                        writing_mode: flow_axes.writing_mode(),
                        direction: flow_axes.direction(),
                        overflow: fri05_c04_flex_overflow_at_flow_axes(
                            flow_axes,
                            Overflow::Scroll,
                            Overflow::Scroll,
                        ),
                        size: size.map(PreferredSize::px),
                        flex_direction: direction,
                        align_items: Some(AlignItems::FlexStart),
                        justify_content: Some(AlignContent::Center),
                        ..NodeInput::default()
                    },
                )
                .style(
                    1,
                    NodeInput {
                        size: child_size.map(PreferredSize::px),
                        min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                        flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                        ..NodeInput::default()
                    },
                );
            let output = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
                .expect("mapped alignment flex layout succeeds");
            let main = (-20.0, 20.0);
            let (inline, block) = if direction.is_row() {
                (main, (0.0, 0.0))
            } else {
                ((0.0, 0.0), main)
            };
            fri05_c04_assert_flow_range(
                output.scroll_geometry.unwrap(),
                flow_axes,
                inline,
                block,
                &format!("{flow_axes:?} {direction:?}"),
            );
        }
    }
}

#[test]
fn fri05_c04_flex_alignment_subject_bounds_farther_absolute_and_nested_start_overflow() {
    let size = Size::new(100.0, 80.0);
    let absolute = |left| NodeInput {
        display: Display::Block,
        position: Position::Absolute,
        size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
        inset: Edges::new(
            LengthAuto::px(0.0),
            LengthAuto::AUTO,
            LengthAuto::AUTO,
            LengthAuto::px(left),
        ),
        ..NodeInput::default()
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 3, 4])
        .children(1, [2])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                size: size.map(PreferredSize::px),
                align_items: Some(AlignItems::FlexStart),
                justify_content: Some(AlignContent::Center),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: ComputedOverflow::VISIBLE,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(20.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        )
        .style(2, absolute(-100.0))
        .style(3, absolute(-100.0))
        .style(4, absolute(160.0));

    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            size,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("bounded alignment overflow layout succeeds");
    let geometry = output.scroll_geometry.unwrap();
    let overflow = geometry.scrollable_overflow();
    assert!(
        overflow.origin().x < -100.0,
        "nested start overflow is retained"
    );
    assert_eq!(overflow.origin().x + overflow.size().width, 170.0);
    fri05_c04_assert_physical_range(output, (-20.0, 70.0, 0.0, 0.0));
}

#[derive(Default)]
struct Fri05C04FlexAutoPassTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    child_output: Option<ComputeOutput>,
    child_outputs: HashMap<u32, ComputeOutput>,
    child_inputs: Vec<ComputeInput>,
    child_requests: Vec<(u32, ComputeInput)>,
    layouts: Vec<(u32, NodeOutput)>,
}

impl Traverse for Fri05C04FlexAutoPassTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl Compute for Fri05C04FlexAutoPassTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInput {
        LayoutInput::box_input(self.styles[&node].clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.push((node, layout));
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, ComputeOutput, Self::Scalar> {
        self.child_inputs.push(input);
        self.child_requests.push((node, input));
        if self.styles[&node].display == Display::Flex && self.child_count(node) != 0 {
            return compute_flex(
                self,
                node,
                input.with_settled_auto_scrollbars(
                    crate::scroll::SettledAutoScrollbarState::INITIAL,
                ),
            );
        }
        Ok(self.child_outputs.get(&node).copied().unwrap_or_else(|| {
            self.child_output
                .expect("FRI-05 flex auto child output is configured")
        }))
    }
}

fn fri05_c04_flex_auto_states(inputs: &[ComputeInput]) -> Vec<(bool, bool)> {
    assert!(
        inputs.iter().all(|input| {
            input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
        }),
        "each direct child request must begin node-local auto settlement at INITIAL: {inputs:#?}"
    );
    let mut states = inputs
        .iter()
        .map(|input| {
            let state = input.containing_auto_scrollbar_pass();
            (
                state.at(PhysicalAxis::Horizontal),
                state.at(PhysicalAxis::Vertical),
            )
        })
        .collect::<Vec<_>>();
    states.dedup();
    states
}

fn fri05_c04_flex_auto_absolute_case(
    flow_axes: FlowAxes,
    container_size: Size<f32>,
    child_size: Size<f32>,
    overflow: ComputedOverflow,
    gutter: ScrollbarGutter,
    scrollbar_width: f32,
) -> (ComputeOutput, Fri05C04FlexAutoPassTree) {
    let (left, right) = match flow_axes.physical_axis_progression(PhysicalAxis::Horizontal) {
        PhysicalProgression::Increasing => (LengthAuto::px(0.0), LengthAuto::AUTO),
        PhysicalProgression::Decreasing => (LengthAuto::AUTO, LengthAuto::px(0.0)),
    };
    let (top, bottom) = match flow_axes.physical_axis_progression(PhysicalAxis::Vertical) {
        PhysicalProgression::Increasing => (LengthAuto::px(0.0), LengthAuto::AUTO),
        PhysicalProgression::Decreasing => (LengthAuto::AUTO, LengthAuto::px(0.0)),
    };
    let mut tree = Fri05C04FlexAutoPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_gutter: gutter,
            scrollbar_width: ScrollbarWidth::try_new(scrollbar_width).unwrap(),
            size: container_size.map(PreferredSize::px),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            size: child_size.map(PreferredSize::px),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            inset: Edges::new(top, right, bottom, left),
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_sizes(child_size, child_size));

    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(container_size, flow_axes),
    )
    .expect("monotone flex auto layout succeeds");
    (output, tree)
}

type Fri05C04AutoStateBits = (bool, bool);
type Fri05C04AutoRequestStates = (Vec<Fri05C04AutoStateBits>, Vec<Fri05C04AutoStateBits>);

fn fri05_c04_flex_auto_request_states(
    requests: &[(u32, ComputeInput)],
    node: u32,
) -> Fri05C04AutoRequestStates {
    let matching = requests
        .iter()
        .filter_map(|(requested, input)| (*requested == node).then_some(*input))
        .collect::<Vec<_>>();
    let local = matching
        .iter()
        .map(|input| {
            let state = input.settled_auto_scrollbars();
            (
                state.at(PhysicalAxis::Horizontal),
                state.at(PhysicalAxis::Vertical),
            )
        })
        .collect::<Vec<_>>();
    let mut containing = Vec::new();
    for state in matching.iter().map(|input| {
        let state = input.containing_auto_scrollbar_pass();
        (
            state.at(PhysicalAxis::Horizontal),
            state.at(PhysicalAxis::Vertical),
        )
    }) {
        if !containing.contains(&state) {
            containing.push(state);
        }
    }
    (local, containing)
}

fn fri05_c04_flex_under_flex_case(
    inner_overflows: bool,
) -> (ComputeOutput, Fri05C04FlexAutoPassTree) {
    let mut tree = Fri05C04FlexAutoPassTree::default();
    tree.children
        .insert(0, if inner_overflows { vec![1] } else { vec![1, 3] });
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::splat_clone(PreferredSize::px(100.0)),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Flex,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::splat_clone(PreferredSize::px(40.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            flex_shrink: FlexShrink::try_new(0.0).unwrap(),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: if inner_overflows {
                Position::Absolute
            } else {
                Position::Relative
            },
            size: if inner_overflows {
                Size::new(PreferredSize::px(60.0), PreferredSize::px(20.0))
            } else {
                Size::splat_clone(PreferredSize::px(20.0))
            },
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            inset: Edges::new(
                LengthAuto::px(0.0),
                LengthAuto::AUTO,
                LengthAuto::AUTO,
                LengthAuto::px(0.0),
            ),
            ..NodeInput::default()
        },
    );
    tree.child_outputs.insert(
        2,
        ComputeOutput::from_sizes(
            if inner_overflows {
                Size::new(60.0, 20.0)
            } else {
                Size::splat(20.0)
            },
            if inner_overflows {
                Size::new(60.0, 20.0)
            } else {
                Size::splat(20.0)
            },
        ),
    );
    if !inner_overflows {
        tree.children.insert(3, vec![]);
        tree.styles.insert(
            3,
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                inset: Edges::new(
                    LengthAuto::px(0.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(0.0),
                ),
                ..NodeInput::default()
            },
        );
        tree.child_outputs.insert(
            3,
            ComputeOutput::from_sizes(Size::new(120.0, 80.0), Size::new(120.0, 80.0)),
        );
    }

    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            Size::splat(100.0),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("real flex-under-flex auto layout succeeds");
    (output, tree)
}

#[test]
fn fri05_c04_flex_auto_direct_nested_passes_separate_local_and_containing_state() {
    let (outer, tree) = fri05_c04_flex_under_flex_case(false);
    assert_eq!(
        outer.scroll_geometry.unwrap().scrollbar_size(),
        Size::new(0.0, 15.0)
    );
    let inner = tree
        .layouts
        .iter()
        .rev()
        .find_map(|(node, output)| (*node == 1).then_some(*output))
        .expect("outer retains the stable inner flex output");
    assert_eq!(inner.scroll_geometry.unwrap().scrollbar_size(), Size::ZERO);

    let (inner_local, inner_containing) =
        fri05_c04_flex_auto_request_states(&tree.child_requests, 1);
    assert!(inner_local.iter().all(|state| *state == (false, false)));
    assert_eq!(inner_containing, [(false, false), (true, false)]);
    let (grandchild_local, grandchild_containing) =
        fri05_c04_flex_auto_request_states(&tree.child_requests, 2);
    assert!(
        grandchild_local
            .iter()
            .all(|state| *state == (false, false))
    );
    assert!(
        grandchild_containing
            .iter()
            .all(|state| *state == (false, false))
    );
}

#[test]
fn fri05_c04_flex_auto_direct_inner_settlement_becomes_grandchild_containing_pass() {
    let (outer, tree) = fri05_c04_flex_under_flex_case(true);
    assert_eq!(outer.scroll_geometry.unwrap().scrollbar_size(), Size::ZERO);
    let inner = tree
        .layouts
        .iter()
        .rev()
        .find_map(|(node, output)| (*node == 1).then_some(*output))
        .expect("outer retains the independently settled inner flex output");
    assert_eq!(
        inner.scroll_geometry.unwrap().scrollbar_size(),
        Size::new(0.0, 15.0)
    );

    let (inner_local, inner_containing) =
        fri05_c04_flex_auto_request_states(&tree.child_requests, 1);
    assert!(inner_local.iter().all(|state| *state == (false, false)));
    assert!(
        inner_containing
            .iter()
            .all(|state| *state == (false, false))
    );
    let (grandchild_local, grandchild_containing) =
        fri05_c04_flex_auto_request_states(&tree.child_requests, 2);
    assert!(
        grandchild_local
            .iter()
            .all(|state| *state == (false, false))
    );
    assert_eq!(grandchild_containing, [(false, false), (true, false)]);
}

#[test]
fn fri05_c04_flex_auto_root_axis_cases_settle_monotonically_from_actual_pass_geometry() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let size = Size::splat(100.0);
    for (child_size, expected_states, expected_scrollbars) in [
        (Size::new(80.0, 80.0), vec![(false, false)], Size::ZERO),
        (
            Size::new(120.0, 80.0),
            vec![(false, false), (true, false)],
            Size::new(0.0, 15.0),
        ),
        (
            Size::new(80.0, 120.0),
            vec![(false, false), (false, true)],
            Size::new(15.0, 0.0),
        ),
        (
            Size::new(120.0, 100.0),
            vec![(false, false), (true, false), (true, true)],
            Size::splat(15.0),
        ),
        (
            Size::new(100.0, 120.0),
            vec![(false, false), (false, true), (true, true)],
            Size::splat(15.0),
        ),
    ] {
        let (output, tree) = fri05_c04_flex_auto_absolute_case(
            flow_axes,
            size,
            child_size,
            computed_overflow(Overflow::Auto, Overflow::Auto),
            ScrollbarGutter::Auto,
            15.0,
        );
        let states = fri05_c04_flex_auto_states(&tree.child_inputs);
        assert_eq!(states, expected_states, "child size {child_size:?}");
        assert!(states.len() <= 3);
        assert_eq!(
            output.scroll_geometry.unwrap().scrollbar_size(),
            expected_scrollbars,
            "child size {child_size:?}"
        );
    }
}

#[test]
fn fri05_c04_flex_auto_alignment_subject_start_overflow_can_induce_other_axis() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let size = Size::splat(100.0);
    let child_size = Size::new(120.0, 100.0);
    let mut tree = Fri05C04FlexAutoPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: size.map(PreferredSize::px),
            align_items: Some(AlignItems::FlexStart),
            justify_content: Some(AlignContent::End),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            size: child_size.map(PreferredSize::px),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            flex_shrink: FlexShrink::try_new(0.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_sizes(child_size, child_size));

    let output = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
        .expect("alignment-subject auto layout succeeds");
    assert_eq!(
        fri05_c04_flex_auto_states(&tree.child_inputs),
        [(false, false), (true, false), (true, true)]
    );
    let range = output.scroll_geometry.unwrap().physical_range();
    assert!(
        range.x().minimum() < 0.0,
        "the actual start subject is observed"
    );
    assert_eq!(range.x().maximum(), 0.0);
}

#[test]
fn fri05_c04_flex_reservation_forced_stable_both_zero_and_auto_map_all_flows() {
    let size = Size::new(100.0, 80.0);
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        for (overflow, gutter, width, inline_start, inline_end) in [
            (
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Scroll),
                ScrollbarGutter::Auto,
                15.0,
                false,
                true,
            ),
            (
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
                ScrollbarGutter::Stable,
                15.0,
                false,
                true,
            ),
            (
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
                ScrollbarGutter::StableBothEdges,
                15.0,
                true,
                true,
            ),
            (
                computed_overflow(Overflow::Scroll, Overflow::Scroll),
                ScrollbarGutter::StableBothEdges,
                0.0,
                false,
                false,
            ),
        ] {
            let (output, _) = fri05_c04_flex_auto_absolute_case(
                flow_axes,
                size,
                Size::new(20.0, 20.0),
                overflow,
                gutter,
                width,
            );
            let gutters = output.scroll_geometry.unwrap().gutters();
            for side in [
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
            ] {
                let expected = (side == flow_axes.inline_start() && inline_start)
                    || (side == flow_axes.inline_end() && inline_end);
                assert_eq!(
                    fri05_c04_flex_gutter_at(gutters, side).is_some(),
                    expected,
                    "{flow_axes:?} {side:?}"
                );
            }
        }

        let (auto_output, auto) = fri05_c04_flex_auto_absolute_case(
            flow_axes,
            size,
            Size::new(120.0, 20.0),
            computed_overflow(Overflow::Auto, Overflow::Auto),
            ScrollbarGutter::Auto,
            15.0,
        );
        assert_eq!(
            fri05_c04_flex_auto_states(&auto.child_inputs),
            [(false, false), (true, false)],
            "{flow_axes:?}"
        );
        let expected_auto_side = if flow_axes.inline_axis() == PhysicalAxis::Horizontal {
            flow_axes.block_end()
        } else {
            flow_axes.inline_end()
        };
        let auto_gutters = auto_output.scroll_geometry.unwrap().gutters();
        for side in [
            PhysicalSide::Top,
            PhysicalSide::Right,
            PhysicalSide::Bottom,
            PhysicalSide::Left,
        ] {
            assert_eq!(
                fri05_c04_flex_gutter_at(auto_gutters, side).is_some(),
                side == expected_auto_side,
                "auto {flow_axes:?} {side:?}"
            );
        }
    }
}

#[test]
fn fri05_c04_flex_tiny_induced_reservations_saturate_without_extra_evaluations() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let (output, tree) = fri05_c04_flex_auto_absolute_case(
        flow_axes,
        Size::splat(2.0),
        Size::new(3.0, 2.0),
        computed_overflow(Overflow::Auto, Overflow::Auto),
        ScrollbarGutter::Auto,
        15.0,
    );
    assert_eq!(
        fri05_c04_flex_auto_states(&tree.child_inputs),
        [(false, false), (true, false), (true, true)]
    );
    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.content_box().size(), Size::ZERO);
    assert_eq!(geometry.scrollport().size(), Size::ZERO);
    assert_eq!(geometry.scrollbar_size(), Size::splat(2.0));
    assert!(
        [
            geometry.border_box(),
            geometry.padding_box(),
            geometry.content_box(),
            geometry.scrollport(),
        ]
        .into_iter()
        .all(|rect| rect.size().width >= 0.0 && rect.size().height >= 0.0)
    );

    let mut measurement_tree = Fri05C04FlexAutoPassTree::default();
    measurement_tree.children.insert(0, vec![1]);
    measurement_tree.children.insert(1, vec![]);
    measurement_tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(PreferredSize::px(2.0), PreferredSize::px(2.0)),
            ..NodeInput::default()
        },
    );
    measurement_tree.styles.insert(1, NodeInput::default());
    measurement_tree.child_output = Some(ComputeOutput::from_outer_size(Size::splat(1.0)));
    let measurement = compute_flex(
        &mut measurement_tree,
        0,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::splat(Some(2.0)),
            Size::splat(Some(2.0)),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            Size::splat(Available::definite(2.0)),
        ),
    )
    .expect("fixed flex measurement remains geometry-free");
    assert!(measurement.scroll_geometry.is_none());
    assert!(measurement_tree.child_inputs.is_empty());
}

fn assert_fri08_c07_t03_optional_math_flex_results<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    let style = NodeInputOf::<S> {
        display: Display::Flex,
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(PreferredSizeOf::px(scalar(4.0)), PreferredSizeOf::AUTO),
        padding: Edges::new(
            LengthOf::px(scalar(7.0)),
            LengthOf::px(scalar(5.0)),
            LengthOf::px(scalar(4.0)),
            LengthOf::px(scalar(3.0)),
        ),
        ..NodeInputOf::default()
    };
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [])
        .style(0, style);
    let output = crate::compute_flex(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(scalar(100.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        ),
    )
    .unwrap_or_else(|_| panic!("finite flex sizing must succeed"));

    assert_eq!(output.size, Size::new(scalar(8.0), scalar(11.0)));

    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let overflowing = LengthPercentageOf::from_coefficients(largest, S::ONE)
        .unwrap_or_else(|_| panic!("finite coefficients must be accepted"));
    let mut failing_tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::value(overflowing), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        );
    let error = crate::compute_flex(
        &mut failing_tree,
        0,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(largest), Some(scalar(100.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(AvailableOf::definite(largest), AvailableOf::MAX_CONTENT),
        ),
    )
    .expect_err("non-finite flex sizing must preserve its error");

    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
            value: S::INFINITY,
        })
    );
}

#[test]
fn fri08_c07_t03_optional_math_flex_results_preserve_both_scalar_lanes() {
    assert_fri08_c07_t03_optional_math_flex_results::<f32>();
    assert_fri08_c07_t03_optional_math_flex_results::<f64>();
}
