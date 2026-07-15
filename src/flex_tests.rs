use std::collections::HashMap;

use crate::flex::FlexAxes;
use crate::geometry::PhysicalProgression;
use crate::*;

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
                    size: Size::new(DimensionOf::AUTO, DimensionOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .style(1, NodeInputOf::default())
            .style(
                2,
                NodeInputOf {
                    position: Position::Absolute,
                    size: Size::new(
                        DimensionOf::px(S::from_f64(30.0)),
                        DimensionOf::px(S::from_f64(12.0)),
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

fn output_from_known_or(input: ComputeInput, fallback: Size) -> ComputeOutput {
    let size = Size::new(
        input.known().width.unwrap_or(fallback.width),
        input.known().height.unwrap_or(fallback.height),
    );
    ComputeOutput::from_sizes(size, size)
}

fn fake_leaf_error(
    node: u32,
    error: LayoutError<(), core::convert::Infallible>,
) -> LayoutError<u32> {
    LayoutError::new(
        LayoutErrorSite::Node(node),
        error.operation(),
        error.kind().clone(),
    )
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
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                self.outputs[&node]
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(30.0), Dimension::px(30.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 20.0), Size::new(40.0, 20.0)),
    );
    tree.outputs.insert(
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
    assert_eq!(output.content_size, Size::new(80.0, 30.0));

    assert_eq!(tree.layouts[&2].location, Point::new(6.0, 6.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(56.0, 6.0));
    assert_eq!(tree.layouts[&3].size, Size::new(30.0, 30.0));

    assert_eq!(
        tree.inputs[&2][0].known(),
        Size::new(Some(40.0), Some(20.0))
    );
    assert_eq!(
        tree.inputs[&3][0].known(),
        Size::new(Some(30.0), Some(30.0))
    );
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
                size: Size::new(DimensionOf::px(container_width), DimensionOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
                size: Size::new(DimensionOf::px(20.125), DimensionOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::Block,
                flex_grow: FlexGrowOf::try_new(3.0).unwrap(),
                size: Size::new(DimensionOf::px(20.125), DimensionOf::px(10.0)),
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
                size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
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
                size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
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
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 10.0));
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn flex_final_content_size_uses_rerun_output() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                let size = if input.run_mode() == RunMode::PerformLayout
                    && input.known().width == Some(80.0)
                {
                    Size::new(80.0, 40.0)
                } else {
                    Size::new(20.0, 10.0)
                };
                ComputeOutput::from_sizes(size, size)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(Dimension::px(80.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
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

    assert!(tree.inputs[&1].iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize && input.known().width == Some(80.0)
    }));
    assert!(tree.inputs[&1].iter().any(|input| {
        input.run_mode() == RunMode::PerformLayout && input.known().width == Some(80.0)
    }));
    assert_eq!(output.content_size.height, 40.0);
}

#[test]
fn flex_relative_child_inset_offsets_final_layout_location() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(7.0, 3.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_relative_child_trailing_inset_offsets_negative() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            inset: Edges {
                right: LengthAuto::px(5.0),
                bottom: LengthAuto::px(2.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(-5.0, -2.0));
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
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
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
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {
            panic!("compute-size must not write child layouts")
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_outer_size(Size::new(20.0, 10.0))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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
    assert_eq!(tree.inputs[&2][0].run_mode(), RunMode::ComputeSize);
}

#[test]
fn flex_row_auto_main_item_uses_content_sizing_for_base_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_outer_size(Size::new(0.0, input.known().height.unwrap_or(10.0)))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(50.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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

    let base_input = tree.inputs[&2][0];
    assert_eq!(base_input.sizing_mode(), SizingMode::ContentSize);
    assert_eq!(base_input.known().width, None);
    assert_eq!(base_input.known().height, Some(10.0));
    assert_eq!(base_input.available().width, Available::MAX_CONTENT);
    assert_eq!(base_input.available().height, Available::definite(10.0));
}

#[test]
fn flex_row_hidden_overflow_item_has_zero_automatic_minimum() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_outer_size(Size::new(
                    input.known().width.unwrap_or(40.0),
                    input.known().height.unwrap_or(50.0),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
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

    assert_eq!(tree.layouts[&2].size, Size::new(0.0, 50.0));
    assert_eq!(tree.layouts[&3].size, Size::new(40.0, 50.0));
}

#[test]
fn flex_column_hidden_overflow_aspect_item_has_zero_automatic_minimum() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_outer_size(Size::new(
                    input.known().width.unwrap_or(40.0),
                    input.known().height.unwrap_or(50.0),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            flex_direction: FlexDirection::Column,
            size: Size::new(Dimension::px(20.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Hidden),
            flex_basis: Dimension::px(0.0),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            aspect_ratio: AspectRatio::new(1.0),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
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

    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 50.0));
}

#[test]
fn flex_column_cross_axis_hidden_overflow_aspect_item_has_zero_automatic_minimum() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_outer_size(Size::new(
                    input.known().width.unwrap_or(40.0),
                    input.known().height.unwrap_or(50.0),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            flex_direction: FlexDirection::Column,
            size: Size::new(Dimension::px(20.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Clip),
            flex_basis: Dimension::px(0.0),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            aspect_ratio: AspectRatio::new(1.0),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
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

    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 50.0));
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
            min_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            max_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
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
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                if input.run_mode() == RunMode::PerformLayout {
                    ComputeOutput::from_sizes(
                        Size::new(input.known().width.unwrap(), input.known().height.unwrap()),
                        Size::ZERO,
                    )
                } else {
                    ComputeOutput::HIDDEN
                }
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::None,
            size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
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

    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(
        tree.layouts[&3],
        NodeOutput::with_source_index(crate::SourceIndex::new(1))
    );
    assert_eq!(
        tree.inputs[&3],
        vec![ComputeInput::hidden(crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr,),
            crate::ParentFormattingContext::Flex
        ))]
    );
}

#[test]
fn flex_container_reserves_scrollbar_gutter_from_inner_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(0.0), Dimension::px(10.0)),
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
    assert_eq!(output.content_size, Size::new(90.0, 40.0));
    assert_eq!(tree.layouts[&2].size, Size::new(90.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::ZERO);
}

#[test]
fn flex_scrollbar_gutter_uses_left_inset_for_rtl_containers() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_child_layout_records_scrollbar_size_for_scroll_overflow() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
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

    assert_eq!(tree.layouts[&2].scrollbar_size, Size::new(7.0, 7.0));
}

#[test]
fn flex_absolute_child_uses_insets_without_affecting_flow() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                if node == 3 {
                    return Ok(ComputeOutput::from_sizes(
                        Size::new(input.known().width.unwrap(), input.known().height.unwrap()),
                        Size::new(80.0, 32.0),
                    ));
                }
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(25.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(12.0)),
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
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
    assert_eq!(output.content_size, Size::new(87.0, 41.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(25.0, 10.0));
    assert_eq!(tree.layouts[&3].location, Point::new(7.0, 9.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 12.0));
    assert_eq!(
        tree.inputs[&3][0].known(),
        Size::new(Some(20.0), Some(12.0))
    );
}

#[test]
fn flex_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(400.0), Dimension::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
        tree.inputs[&2][0].known(),
        Size::new(Some(360.0), Some(120.0))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(20.0, 15.0));
    assert_eq!(tree.layouts[&2].size, Size::new(360.0, 120.0));
}

#[test]
fn flex_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(400.0), Dimension::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::percent(0.1),
                right: LengthAuto::percent(0.1),
                top: LengthAuto::percent(0.05),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(Dimension::percent(0.4), Dimension::AUTO),
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
        tree.inputs[&2][0].known(),
        Size::new(Some(160.0), Some(160.0 / 3.0))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(200.0, 15.0));
}

#[test]
fn flex_absolute_child_max_height_shrinks_flex_grandchild() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(
            &mut self,
            node: u32,
            input: ComputeInput,
        ) -> LayoutResultOf<u32, ComputeOutput, Scalar> {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return compute_leaf(input, &node_input, |measure_input| {
                    let known = measure_input.known_content_size();
                    Ok::<_, core::convert::Infallible>(Size::new(
                        known.width.unwrap_or(0.0),
                        known.height.unwrap_or(0.0),
                    ))
                })
                .map_err(|error| fake_leaf_error(node, error));
            }

            match node_input.display.inner_display() {
                Display::Flex => compute_flex(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::None => Ok(ComputeOutput::HIDDEN),
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
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

    impl Compute for RecursiveTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            self.compute_node(node, input)
        }
    }

    let mut tree = RecursiveTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![3]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
            flex_direction: FlexDirection::Column,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            flex_direction: FlexDirection::Column,
            inset: Edges {
                bottom: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            max_size: Size::new(Dimension::AUTO, Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            flex_basis: Dimension::px(150.0),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 80.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 100.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(100.0, 100.0));
}

#[test]
fn flex_absolute_child_cross_alignment_honors_wrap_reverse() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs_for(node, input))
        }
    }

    impl FlexTree {
        fn new(
            align_self: AlignItems,
            flex_direction: FlexDirection,
            layout_direction: Direction,
        ) -> Self {
            let mut tree = Self::default();
            tree.children.insert(1, vec![2]);
            tree.children.insert(2, vec![]);
            tree.styles.insert(
                1,
                NodeInput {
                    direction: layout_direction,
                    size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                    flex_direction,
                    flex_wrap: FlexWrap::WrapReverse,
                    ..NodeInput::default()
                },
            );
            tree.styles.insert(
                2,
                NodeInput {
                    direction: layout_direction,
                    position: Position::Absolute,
                    align_self: Some(align_self),
                    size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
                    ..NodeInput::default()
                },
            );
            tree
        }

        fn outputs_for(&self, _node: u32, input: ComputeInput) -> ComputeOutput {
            output_from_known_or(input, Size::ZERO)
        }

        fn layout_child(&mut self) -> NodeOutput {
            compute_flex(
                self,
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
            self.layouts[&2]
        }
    }

    let default_layout =
        FlexTree::new(AlignItems::Stretch, FlexDirection::Row, Direction::Ltr).layout_child();
    assert_eq!(default_layout.location, Point::new(0.0, 80.0));
    assert_eq!(default_layout.size, Size::new(20.0, 20.0));

    let flex_end_layout =
        FlexTree::new(AlignItems::FlexEnd, FlexDirection::Row, Direction::Ltr).layout_child();
    assert_eq!(flex_end_layout.location, Point::new(0.0, 0.0));
    assert_eq!(flex_end_layout.size, Size::new(20.0, 20.0));

    let column_rtl_layout =
        FlexTree::new(AlignItems::Stretch, FlexDirection::Column, Direction::Rtl).layout_child();
    assert_eq!(column_rtl_layout.location, Point::new(0.0, 0.0));
    assert_eq!(column_rtl_layout.size, Size::new(20.0, 20.0));

    let column_rtl_flex_end_layout =
        FlexTree::new(AlignItems::FlexEnd, FlexDirection::Column, Direction::Rtl).layout_child();
    assert_eq!(column_rtl_flex_end_layout.location, Point::new(80.0, 0.0));
    assert_eq!(column_rtl_flex_end_layout.size, Size::new(20.0, 20.0));
}

#[test]
fn flex_absolute_child_cross_start_margin_uses_physical_edge_in_rtl_column() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            justify_content: Some(AlignContent::FlexEnd),
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            direction: Direction::Rtl,
            position: Position::Absolute,
            size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(90.0, 80.0));
    assert_eq!(tree.layouts[&2].size, Size::new(10.0, 10.0));
}

#[test]
fn flex_absolute_child_uses_min_size_when_min_exceeds_max_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_sizes(
                    Size::new(
                        input.known().width.unwrap_or(0.0),
                        input.known().height.unwrap_or(0.0),
                    ),
                    Size::new(
                        input.known().width.unwrap_or(0.0),
                        input.known().height.unwrap_or(0.0),
                    ),
                )
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                right: LengthAuto::px(10.0),
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            min_size: Size::new(Dimension::px(50.0), Dimension::px(60.0)),
            max_size: Size::new(Dimension::px(40.0), Dimension::px(30.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 30.0));
    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 60.0));
}

#[test]
fn flex_absolute_child_size_cannot_shrink_below_padding_and_border() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_sizes(
                    Size::new(
                        input.known().width.unwrap_or(0.0),
                        input.known().height.unwrap_or(0.0),
                    ),
                    Size::new(
                        input.known().width.unwrap_or(0.0),
                        input.known().height.unwrap_or(0.0),
                    ),
                )
            })
        }
    }

    fn tree_with_child(child_style: NodeInput) -> FlexTree {
        let mut tree = FlexTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![]);
        tree.styles.insert(1, NodeInput::default());
        tree.styles.insert(2, child_style);
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
        size: Size::new(Dimension::px(12.0), Dimension::px(12.0)),
        padding,
        border,
        ..NodeInput::default()
    });
    run(&mut authored_size);
    assert_eq!(
        authored_size.inputs[&2][0].known(),
        Size::new(Some(22.0), Some(14.0))
    );
    assert_eq!(authored_size.layouts[&2].size, Size::new(22.0, 14.0));

    let mut max_size = tree_with_child(NodeInput {
        position: Position::Absolute,
        max_size: Size::new(Dimension::px(12.0), Dimension::px(12.0)),
        padding,
        border,
        ..NodeInput::default()
    });
    run(&mut max_size);
    assert_eq!(max_size.layouts[&2].size, Size::new(22.0, 14.0));
}

#[test]
fn flex_absolute_child_layout_records_scrollbar_size_for_scroll_overflow() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
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

    assert_eq!(tree.layouts[&2].scrollbar_size, Size::new(8.0, 8.0));
}

#[test]
fn flex_absolute_child_can_resolve_from_trailing_insets() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                right: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(72.0, 34.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_absolute_child_expands_auto_margins() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
}

#[test]
fn flex_absolute_child_without_insets_uses_flex_alignment() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            justify_content: Some(AlignContent::Center),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 15.0));
}

#[test]
fn flex_row_distributes_positive_free_space_with_flex_grow() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
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
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(105.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(105.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(95.0, 20.0));

    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
        Size::new(Some(105.0), Some(20.0))
    );
    assert_eq!(
        tree.inputs[&3].last().unwrap().known(),
        Size::new(Some(95.0), Some(20.0))
    );
}

#[test]
fn flex_row_with_grow_sum_below_one_uses_that_fraction_of_free_space() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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
    assert_eq!(tree.layouts[&2].location, Point::ZERO);
    assert_eq!(tree.layouts[&2].size, Size::new(60.0, 10.0));
}

#[test]
fn flex_row_distributes_negative_free_space_with_flex_shrink() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(80.0), Dimension::px(20.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(70.0), Dimension::px(20.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
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
    assert!((tree.layouts[&2].size.width - 53.333).abs() < 0.01);
    assert!((tree.layouts[&3].location.x - 53.333).abs() < 0.01);
    assert!((tree.layouts[&3].size.width - 46.667).abs() < 0.01);
    assert_eq!(tree.layouts[&2].size.height, 20.0);
    assert_eq!(tree.layouts[&3].size.height, 20.0);
}

#[test]
fn flex_row_relayouts_content_box_percentage_item_at_shrunk_target() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_outer_size(Size::new(
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(730.0), Dimension::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(Dimension::percent(1.0), Dimension::px(100.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
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

    assert_eq!(tree.layouts[&2].size.width, 730.0);
    assert_eq!(
        tree.inputs[&2]
            .last()
            .expect("child should be laid out")
            .known()
            .width,
        Some(730.0)
    );
}

#[test]
fn flex_row_visible_item_does_not_shrink_below_automatic_min_content_width() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                if node == 2
                    && input.run_mode() == RunMode::ComputeSize
                    && input.available().width == Available::MIN_CONTENT
                {
                    return Ok(ComputeOutput::from_outer_size(Size::new(90.0, 20.0)));
                }

                let fallback = if node == 2 {
                    Size::new(160.0, 20.0)
                } else {
                    Size::new(40.0, 20.0)
                };
                ComputeOutput::from_outer_size(Size::new(
                    input.known().width.unwrap_or(fallback.width),
                    input.known().height.unwrap_or(fallback.height),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::px(20.0)),
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
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
    assert!(
        tree.inputs[&2].iter().any(|input| {
            input.run_mode() == RunMode::ComputeSize
                && input.available().width == Available::MIN_CONTENT
        }),
        "visible flex item should be measured with min-content for its automatic minimum"
    );
    assert_eq!(tree.layouts[&2].size.width, 90.0);
    assert_eq!(tree.layouts[&3].location.x, 90.0);
    assert_eq!(tree.layouts[&3].size.width, 40.0);
}

#[test]
fn flex_row_with_shrink_sum_below_one_uses_that_fraction_of_negative_free_space() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(80.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(10.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
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
    assert_eq!(tree.layouts[&2].location, Point::ZERO);
    assert_eq!(tree.layouts[&2].size, Size::new(90.0, 10.0));
}

#[test]
fn flex_row_wraps_items_into_multiple_lines() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            flex_wrap: FlexWrap::Wrap,
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3, 4] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(60.0), Dimension::px(10.0)),
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
    assert_eq!(output.content_size, Size::new(60.0, 38.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 14.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 28.0));
}

#[test]
fn flex_row_auto_width_wraps_against_definite_available_width() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            flex_wrap: FlexWrap::Wrap,
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3, 4] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(60.0), Dimension::px(10.0)),
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
    assert_eq!(output.content_size, Size::new(60.0, 38.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 14.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 28.0));
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
                DimensionOf::px(S::from_f64(30.0)),
                DimensionOf::px(S::from_f64(10.0)),
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
                        DimensionOf::px(S::from_f64(60.0)),
                        DimensionOf::px(S::from_f64(20.0)),
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
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(25.0), Dimension::px(10.0)),
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
    assert_eq!(tree.layouts[&2].location, Point::new(25.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(50.0, 0.0));
}

#[test]
fn flex_row_aligns_items_on_the_cross_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 15.0));
}

#[test]
fn flex_row_reports_first_child_baseline() {
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
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                let size = Size::new(
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(0.0),
                );
                ComputeOutput::from_sizes_and_first_baselines(
                    size,
                    Size::ZERO,
                    Point::new(None, Some(7.0)),
                )
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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
                size: Size::new(DimensionOf::px(S::from_f64(200.0)), DimensionOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    DimensionOf::px(S::from_f64(70.0)),
                    DimensionOf::px(S::from_f64(110.0)),
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
                    DimensionOf::px(S::from_f64(200.0)),
                    DimensionOf::px(S::from_f64(160.0)),
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
                    DimensionOf::px(S::from_f64(70.0)),
                    DimensionOf::px(S::from_f64(110.0)),
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

struct BaselineRefreshTree<S: LayoutScalar> {
    styles: HashMap<u32, NodeInputOf<S>>,
    layouts: HashMap<u32, NodeOutputOf<S>>,
    initial_child_main: S,
}

impl<S: LayoutScalar> Traverse for BaselineRefreshTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        match node {
            1 => [2].iter().copied(),
            _ => [].iter().copied(),
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == 1)
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        2
    }
}

impl<S: LayoutScalar> Compute for BaselineRefreshTree<S> {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.styles[&node].clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S> {
        assert_eq!(node, 2, "the focused flex tree exposes one measured child");
        let main = input.known().height.unwrap_or(self.initial_child_main);
        let size = Size::new(main / S::from_f64(2.0), main);
        Ok(ComputeOutputOf::from_sizes_and_baselines(
            size,
            size,
            BaselinesOf {
                first: Point::new(Some(size.width), None),
                last: Point::new(Some(size.width), None),
            },
        ))
    }
}

fn assert_logical_flex_sizing_orthogonal_refreshes_mapped_main<S: LayoutScalar>(
    container_main: f64,
    child_main: f64,
    expected_child_size: Size<S>,
) {
    let mut tree = BaselineRefreshTree {
        styles: HashMap::from([
            (
                1,
                NodeInputOf::<S> {
                    display: Display::Flex,
                    writing_mode: WritingMode::VerticalRl,
                    flex_direction: FlexDirection::Row,
                    size: Size::new(
                        DimensionOf::AUTO,
                        DimensionOf::px(S::from_f64(container_main)),
                    ),
                    ..NodeInputOf::default()
                },
            ),
            (
                2,
                NodeInputOf::<S> {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalRl,
                    size: Size::new(DimensionOf::AUTO, DimensionOf::px(S::from_f64(child_main))),
                    min_size: Size::new(DimensionOf::ZERO, DimensionOf::ZERO),
                    flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow"),
                    flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                    ..NodeInputOf::default()
                },
            ),
        ]),
        layouts: HashMap::new(),
        initial_child_main: S::from_f64(child_main),
    };

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

    assert_eq!(tree.layouts[&2].size, expected_child_size);
    assert_eq!(
        tree.layouts[&2].location,
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

struct FinalSizeSelectorTree<S: LayoutScalar> {
    styles: HashMap<u32, NodeInputOf<S>>,
    layouts: HashMap<u32, NodeOutputOf<S>>,
    final_known: Option<Size<Option<S>>>,
}

impl<S: LayoutScalar> Traverse for FinalSizeSelectorTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        match node {
            1 => [2].iter().copied(),
            _ => [].iter().copied(),
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == 1)
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        2
    }
}

impl<S: LayoutScalar> Compute for FinalSizeSelectorTree<S> {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.styles[&node].clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S> {
        assert_eq!(node, 2, "the focused flex tree exposes one child");
        if input.run_mode() == RunMode::PerformLayout {
            self.final_known = Some(input.known());
            let size = Size::new(
                input.known().width.unwrap_or(S::from_f64(75.0)),
                input.known().height.unwrap_or(S::from_f64(20.0)),
            );
            return Ok(ComputeOutputOf::from_sizes(size, size));
        }

        Ok(ComputeOutputOf::from_sizes(
            Size::new(S::from_f64(75.0), S::from_f64(20.0)),
            Size::new(S::from_f64(75.0), S::from_f64(20.0)),
        ))
    }
}

fn assert_logical_flex_final_size_selector_uses_vertical_row_main_axis<S: LayoutScalar>(
    writing_mode: WritingMode,
) {
    let mut tree = FinalSizeSelectorTree {
        styles: HashMap::from([
            (
                1,
                NodeInputOf::<S> {
                    display: Display::Flex,
                    writing_mode,
                    size: Size::new(
                        DimensionOf::px(S::from_f64(200.0)),
                        DimensionOf::px(S::from_f64(100.0)),
                    ),
                    flex_direction: FlexDirection::Row,
                    ..NodeInputOf::default()
                },
            ),
            (
                2,
                NodeInputOf::<S> {
                    display: Display::Block,
                    size: Size::new(
                        DimensionOf::percent(S::from_f64(0.25)),
                        DimensionOf::px(S::from_f64(20.0)),
                    ),
                    min_size: Size::new(DimensionOf::px(S::from_f64(75.0)), DimensionOf::ZERO),
                    ..NodeInputOf::default()
                },
            ),
        ]),
        layouts: HashMap::new(),
        final_known: None,
    };

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
        tree.final_known
            .expect("final layout request is recorded")
            .width,
        Some(S::from_f64(50.0)),
        "the percentage-dependent physical width is refined after a vertical main-axis row"
    );
    assert_eq!(
        tree.layouts[&2].size,
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
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                let baseline = match node {
                    2 => 15.0,
                    3 => 5.0,
                    _ => 0.0,
                };
                let size = Size::new(
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(0.0),
                );
                ComputeOutput::from_sizes_and_first_baselines(
                    size,
                    Size::ZERO,
                    Point::new(None, Some(baseline)),
                )
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            align_items: Some(AlignItems::Baseline),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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
    assert_eq!(tree.layouts[&2].location.y, 0.0);
    assert_eq!(tree.layouts[&3].location.y, 10.0);
    assert_eq!(output.first_baselines.y, Some(15.0));
}

#[test]
fn flex_row_stretches_auto_cross_size_items() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                let size = Size::new(
                    input.known().width.unwrap_or(20.0),
                    input.known().height.unwrap_or(10.0),
                );
                ComputeOutput::from_sizes(size, size)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::AUTO),
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
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 40.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
        Size::new(Some(20.0), Some(40.0))
    );
}

#[test]
fn flex_row_stretch_transfers_cross_size_through_aspect_ratio() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::new(20.0, 10.0))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(200.0), Dimension::px(50.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 50.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
        Size::new(Some(100.0), Some(50.0))
    );
}

#[test]
fn flex_row_stretched_aspect_ratio_item_does_not_shrink_below_transferred_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::new(0.0, 0.0)))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            min_size: Size::new(Dimension::AUTO, Dimension::px(40.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(200.0, 100.0));
}

#[test]
fn flex_replaced_automatic_minimum_selects_smaller_suggestion_and_preserves_cross_stretch_in_both_scalar_lanes()
 {
    assert_flex_replaced_automatic_minimum_selects_smaller_suggestion::<f32>();
    assert_flex_replaced_automatic_minimum_selects_smaller_suggestion::<f64>();
}

fn assert_flex_replaced_automatic_minimum_selects_smaller_suggestion<S: LayoutScalar>() {
    #[derive(Default)]
    struct FlexTree<S: LayoutScalar> {
        styles: HashMap<u32, NodeInputOf<S>>,
        layouts: HashMap<u32, NodeOutputOf<S>>,
    }

    impl<S: LayoutScalar> Traverse for FlexTree<S> {
        type Node = u32;
        type Scalar = S;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            match node {
                1 => [2].iter().copied(),
                _ => [].iter().copied(),
            }
        }

        fn child_count(&self, node: Self::Node) -> usize {
            usize::from(node == 1)
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            2
        }
    }

    impl<S: LayoutScalar> Compute for FlexTree<S> {
        fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInputOf<S>,
        ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S> {
            let size = Size::new(
                input.known().width.unwrap_or(S::from_f64(90.0)),
                input.known().height.unwrap_or(S::from_f64(10.0)),
            );
            Ok(ComputeOutputOf::from_sizes(size, size))
        }
    }

    let mut widths = Vec::new();
    let mut heights = Vec::new();
    for item_is_replaced in [true, false] {
        let mut tree = FlexTree::default();
        tree.styles.insert(
            1,
            NodeInputOf {
                display: Display::Flex,
                align_items: Some(AlignItems::Stretch),
                size: Size::new(
                    DimensionOf::px(S::from_f64(60.0)),
                    DimensionOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInputOf {
                item_is_replaced,
                aspect_ratio: AspectRatioOf::new(S::from_f64(2.0)),
                flex_basis: DimensionOf::px(S::from_f64(90.0)),
                flex_grow: FlexGrowOf::try_new(S::ZERO).expect("zero is a valid flex grow"),
                flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                ..NodeInputOf::default()
            },
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

        let layout = tree.layouts[&2];
        widths.push(layout.size.width);
        heights.push(layout.size.height);
    }

    assert_eq!(widths, [S::from_f64(60.0), S::from_f64(90.0)]);
    assert_eq!(heights, [S::from_f64(20.0), S::from_f64(20.0)]);
}

#[test]
fn flex_row_aspect_ratio_auto_min_respects_authored_width_cap() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::new(20.0, 10.0)))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(300.0), Dimension::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(50.0), Dimension::px(100.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 100.0));
}

#[test]
fn flex_row_aligns_wrapped_lines_with_align_content() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Center),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::px(10.0)),
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
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 18.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 32.0));
}

#[test]
fn flex_column_wrap_with_one_line_honors_align_content_end() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3, 4, 5, 6]);
    for node in 2..=6 {
        tree.children.insert(node, vec![]);
    }
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::End),
            ..NodeInput::default()
        },
    );
    for child in 2..=6 {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(50.0), Dimension::px(10.0)),
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
        assert_eq!(tree.layouts[&child].location.x, 50.0);
    }
}

#[test]
fn flex_row_stretches_wrapped_lines_with_align_content_stretch() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Stretch),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::px(10.0)),
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
    assert_eq!(output.content_size, Size::new(80.0, 60.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 32.0));
}

#[test]
fn flex_row_stretched_wrapped_line_stretches_auto_cross_size_item() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                let size = Size::new(
                    input.known().width.unwrap_or(80.0),
                    input.known().height.unwrap_or(10.0),
                );
                ComputeOutput::from_sizes(size, size)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Stretch),
            align_items: Some(AlignItems::Stretch),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
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

    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 28.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 28.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 32.0));
    assert_eq!(
        tree.inputs[&3].last().unwrap().known(),
        Size::new(Some(80.0), Some(28.0))
    );
}

#[test]
fn flex_row_wrap_reverse_places_lines_from_the_reversed_cross_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_content: Some(AlignContent::FlexStart),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 50.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 36.0));
}

#[test]
fn flex_row_wrap_reverse_flips_flex_start_item_alignment() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 50.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_row_wrap_reverse_respects_reversed_align_content() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_content: Some(AlignContent::FlexEnd),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 14.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 0.0));
}

#[test]
fn flex_row_growth_respects_max_main_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            max_size: Size::new(Dimension::px(60.0), Dimension::AUTO),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
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

    assert_eq!(tree.layouts[&2].size, Size::new(60.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(60.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(140.0, 20.0));
}

#[test]
fn flex_row_distributes_positive_space_to_main_axis_auto_margins() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&2].margin.left, 80.0);
}

#[test]
fn flex_row_distributes_cross_axis_auto_margins() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 15.0));
    assert_eq!(tree.layouts[&2].margin.top, 15.0);
    assert_eq!(tree.layouts[&2].margin.bottom, 15.0);
}

#[test]
fn flex_row_reverse_places_items_from_the_reversed_main_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            flex_direction: FlexDirection::RowReverse,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(50.0, 0.0));
}

#[test]
fn flex_row_rtl_places_items_from_the_right_edge() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(50.0, 0.0));
}

#[test]
fn flex_row_rtl_relative_insets_follow_rtl_main_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            inset: Edges {
                left: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            inset: Edges {
                right: LengthAuto::px(7.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(85.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(53.0, 0.0));
}

#[test]
fn flex_column_rtl_aligns_cross_start_to_the_right_edge() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::FlexStart),
            size: Size::new(Dimension::px(100.0), Dimension::px(80.0)),
            padding: Edges {
                left: Length::px(4.0),
                right: Length::px(6.0),
                top: Length::ZERO,
                bottom: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(74.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_column_rtl_cross_axis_auto_margin_uses_rtl_edges() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            size: Size::new(Dimension::px(100.0), Dimension::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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

    assert_eq!(tree.layouts[&2].margin.right, 77.0);
    assert_eq!(tree.layouts[&2].margin.left, 3.0);
    assert_eq!(tree.layouts[&2].location, Point::new(3.0, 0.0));
}

#[test]
fn flex_column_reverse_places_items_from_the_reversed_main_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(100.0)),
            flex_direction: FlexDirection::ColumnReverse,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 80.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 50.0));
}

#[test]
fn flex_row_uses_flex_basis_as_the_main_base_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                let size = Size::new(
                    input.known().width.unwrap_or(10.0),
                    input.known().height.unwrap_or(10.0),
                );
                ComputeOutput::from_sizes(size, size)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::px(10.0)),
            flex_basis: Dimension::px(30.0),
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

    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 10.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
        Size::new(Some(30.0), Some(10.0))
    );
}

#[test]
fn flex_row_flex_basis_zero_preserves_padding_border_without_authored_content_width() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::new(34.0, 10.0))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(12.0), Dimension::px(12.0)),
            flex_basis: Dimension::px(0.0),
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
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(12.0), Dimension::px(12.0)),
            flex_basis: Dimension::px(0.0),
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
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(22.0, 14.0));
    assert_eq!(tree.layouts[&3].location, Point::new(22.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(0.0, 12.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
        Size::new(Some(22.0), Some(14.0))
    );
}

#[test]
fn flex_row_flex_basis_padding_floor_preserves_leaf_content_intrinsic_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(ComputeOutput::from_sizes(
                Size::new(0.0, 10.0),
                Size::new(120.0, 10.0),
            ))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            flex_basis: Dimension::px(0.0),
            padding: Edges {
                left: Length::px(10.0),
                right: Length::px(10.0),
                ..Edges::all(Length::ZERO)
            },
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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size.width, 120.0);
    assert_eq!(output.content_size.width, 120.0);
    assert_eq!(tree.layouts[&2].content_size.width, 120.0);
}

use crate::{Dimension, LengthPercentageOf, NodeInput};

#[test]
fn flex_percent_dependent_affine_size_requests_definite_cross_rerun() {
    let height = LengthPercentageOf::from_coefficients(10.0, 0.50).expect("finite coefficients");
    let mut child = NodeInput::default();
    child.size.height = Dimension::value(height);

    assert!(child.size.height.depends_on_basis());
}
