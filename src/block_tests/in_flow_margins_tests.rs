use super::fixtures::{
    BlockTree, CalcBlockTree, PublicBlockTree, all_writing_mode_directions, computed_overflow,
    fri05_c03_block_union_content_size, fri06_atomic_participation, lp, public_final_output,
    scalar_value,
};
use super::*;

fn scalar_percentage<S: LayoutScalar>(
    absolute_px: f64,
    percent_fraction: f64,
) -> LengthPercentageOf<S> {
    LengthPercentageOf::from_coefficients(scalar_value(absolute_px), scalar_value(percent_fraction))
        .expect("test coefficients are finite")
}

#[test]
fn parent_context_gates_only_block_boundary_collapse_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>()
    where
        crate::test_support::layout_tree::OracleTreeOf<S>:
            Compute + Traverse<Node = u32, Scalar = S>,
    {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        for (parent_context, expected_collapse) in [
            (ParentFormattingContext::BlockFlow, true),
            (ParentFormattingContext::Flex, false),
            (ParentFormattingContext::Grid, false),
            (ParentFormattingContext::NoParent, false),
        ] {
            let mut child_output =
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(40.0), S::ZERO));
            child_output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
                flow_axes,
                CollapsibleMarginOf::from_margin(S::from_f64(3.0)),
                CollapsibleMarginOf::from_margin(S::from_f64(5.0)),
                true,
            );
            let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        size: Size::new(
                            PreferredSizeOf::px(S::from_f64(40.0)),
                            PreferredSizeOf::AUTO,
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        margin: Edges::new(
                            LengthAutoOf::px(S::from_f64(3.0)),
                            LengthAutoOf::ZERO,
                            LengthAutoOf::px(S::from_f64(5.0)),
                            LengthAutoOf::ZERO,
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .measure(1, child_output);
            let output = crate::compute_block(
                &mut tree,
                0,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(100.0))),
                    ContainingLayoutContext::new(flow_axes, parent_context),
                    Size::new(
                        AvailableOf::definite(S::from_f64(100.0)),
                        AvailableOf::MAX_CONTENT,
                    ),
                ),
            )
            .expect("block layout succeeds");

            let collapse = output.block_margin_collapse;
            assert_eq!(
                collapse.at(flow_axes.block_start()).resolve(),
                if expected_collapse {
                    S::from_f64(3.0)
                } else {
                    S::ZERO
                },
                "unexpected block-start collapse for {parent_context:?}"
            );
            assert_eq!(
                collapse.at(flow_axes.block_end()).resolve(),
                if expected_collapse {
                    S::from_f64(5.0)
                } else {
                    S::ZERO
                },
                "unexpected block-end collapse for {parent_context:?}"
            );
            assert_eq!(
                collapse.can_collapse_through(flow_axes),
                expected_collapse,
                "unexpected boundary collapse for {parent_context:?}"
            );
        }

        let mut root_tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [])
            .style(0, NodeInputOf::default());
        let root_output = crate::compute_block(
            &mut root_tree,
            0,
            ComputeInputOf::root_layout(
                Size::NONE,
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(100.0))),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::BlockFlow),
                Size::splat(AvailableOf::definite(S::from_f64(100.0))),
            ),
        )
        .expect("root-mode block layout succeeds");
        assert!(
            !root_output
                .block_margin_collapse
                .can_collapse_through(flow_axes)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn fri06_mr02_physical_edge_value<S: LayoutScalar>(edges: Edges<S>, side: PhysicalSide) -> S {
    match side {
        PhysicalSide::Top => edges.top,
        PhysicalSide::Right => edges.right,
        PhysicalSide::Bottom => edges.bottom,
        PhysicalSide::Left => edges.left,
    }
}

fn assert_fri06_mr02_physical_edge_block_margin_selection<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let sentinels = Edges::new(
        S::from_f64(11.0),
        S::from_f64(22.0),
        S::from_f64(33.0),
        S::from_f64(44.0),
    );

    for (writing_mode, direction) in [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::SidewaysRl, Direction::Rtl),
    ] {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let mut child_output = ComputeOutputOf::from_outer_size(Size::ZERO);
        child_output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
            flow_axes,
            CollapsibleMarginOf::ZERO,
            CollapsibleMarginOf::ZERO,
            true,
        );
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::AUTO,
                    )),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    margin: sentinels.map(LengthAutoOf::px),
                    ..NodeInputOf::default()
                },
            )
            .measure(1, child_output);

        let output = crate::compute_block(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(S::from_f64(100.0))),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::BlockFlow),
                Size::splat(AvailableOf::definite(S::from_f64(100.0))),
            ),
        )
        .expect("block layout with physical margin sentinels succeeds");

        assert_eq!(
            output
                .block_margin_collapse
                .at(flow_axes.block_start())
                .resolve(),
            fri06_mr02_physical_edge_value(sentinels, flow_axes.block_start()),
            "block-start margin selection changed for {writing_mode:?} {direction:?}"
        );
        assert_eq!(
            output
                .block_margin_collapse
                .at(flow_axes.block_end())
                .resolve(),
            fri06_mr02_physical_edge_value(sentinels, flow_axes.block_end()),
            "block-end margin selection changed for {writing_mode:?} {direction:?}"
        );
    }
}

fn assert_fri06_mr02_physical_edge_leaf_collapse_selection<S: LayoutScalar>() {
    let sentinels = [
        (PhysicalSide::Top, S::from_f64(11.0)),
        (PhysicalSide::Right, S::from_f64(22.0)),
        (PhysicalSide::Bottom, S::from_f64(33.0)),
        (PhysicalSide::Left, S::from_f64(44.0)),
    ];

    for (writing_mode, direction) in [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::VerticalLr, Direction::Rtl),
        (WritingMode::SidewaysRl, Direction::Rtl),
        (WritingMode::SidewaysLr, Direction::Ltr),
    ] {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        for (side, sentinel) in sentinels {
            for use_border in [false, true] {
                let mut edge = Edges::all(LengthOf::ZERO);
                match side {
                    PhysicalSide::Top => edge.top = LengthOf::px(sentinel),
                    PhysicalSide::Right => edge.right = LengthOf::px(sentinel),
                    PhysicalSide::Bottom => edge.bottom = LengthOf::px(sentinel),
                    PhysicalSide::Left => edge.left = LengthOf::px(sentinel),
                }
                let mut style = NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    ..NodeInputOf::default()
                };
                if use_border {
                    style.border = edge;
                } else {
                    style.padding = edge;
                }
                let input = ComputeInputOf::leaf_layout(
                    Size::NONE,
                    Size::splat(Some(S::from_f64(100.0))),
                    ContainingLayoutContext::new(flow_axes, ParentFormattingContext::BlockFlow),
                    Size::splat(AvailableOf::definite(S::from_f64(100.0))),
                )
                .expect("physical-edge leaf input is valid");
                let output = compute_leaf(input, &style, |_| Ok::<_, ()>(Size::ZERO))
                    .expect("physical-edge leaf layout succeeds");
                let selected_for_block_axis =
                    side == flow_axes.block_start() || side == flow_axes.block_end();

                assert_eq!(
                    output.block_margin_collapse.can_collapse_through(flow_axes),
                    !selected_for_block_axis,
                    "leaf {} selection changed for {side:?} in {writing_mode:?} {direction:?}",
                    if use_border { "border" } else { "padding" }
                );
            }
        }
    }
}

#[test]
fn fri06_mr02_physical_edge_block_and_leaf_callers_select_physical_sentinels_both_scalars() {
    assert_fri06_mr02_physical_edge_block_margin_selection::<f32>();
    assert_fri06_mr02_physical_edge_block_margin_selection::<f64>();
    assert_fri06_mr02_physical_edge_leaf_collapse_selection::<f32>();
    assert_fri06_mr02_physical_edge_leaf_collapse_selection::<f64>();
}

#[test]
fn replaced_block_child_keeps_measured_auto_inline_size_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar_value::<S>;
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(scalar(200.0)), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    item_is_replaced: true,
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .with_measurement(1, Size::new(scalar(50.0), scalar(10.0)))
            .with_measurement(2, Size::new(scalar(50.0), scalar(10.0)));
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(scalar(200.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .expect("finite viewport request"),
        )
        .expect("measured block children lay out");

        assert_eq!(public_final_output(&batch, 1).size.width, scalar(50.0));
        assert_eq!(public_final_output(&batch, 2).size.width, scalar(200.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn block_layout_ignores_item_order_for_geometry() {
    let layout = |item_orders: [ItemOrder; 3]| {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::splat_clone(PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[0],
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                2,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[1],
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                3,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[2],
                    size: Size::new(PreferredSize::px(30.0), PreferredSize::px(30.0)),
                    ..NodeInput::default()
                },
            );
        let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0)))
            .expect("finite viewport is valid");
        let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

        [
            public_final_output(&batch, 1),
            public_final_output(&batch, 2),
            public_final_output(&batch, 3),
        ]
    };

    let source_order = layout([ItemOrder::ZERO; 3]);
    let non_default_order = layout([ItemOrder::new(7), ItemOrder::new(-3), ItemOrder::new(2)]);

    assert_eq!(non_default_order, source_order);
    assert_eq!(
        non_default_order.map(|output| (output.source_index, output.location)),
        [
            (SourceIndex::new(0), Point::new(0.0, 0.0)),
            (SourceIndex::new(1), Point::new(0.0, 10.0)),
            (SourceIndex::new(2), Point::new(0.0, 30.0)),
        ]
    );
}

fn assert_ordinary_block_flow<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    expected_first: Point<S>,
    expected_second: Point<S>,
) {
    let scalar = scalar_value::<S>;
    let child_style = NodeInputOf {
        display: Display::Block,
        writing_mode,
        direction,
        size: Size::new(
            PreferredSizeOf::px(scalar(20.0)),
            PreferredSizeOf::px(scalar(10.0)),
        ),
        ..NodeInputOf::default()
    };
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, child_style.clone())
        .with_style(2, child_style);
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
        .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

    assert_eq!(public_final_output(&batch, 1).location, expected_first);
    assert_eq!(public_final_output(&batch, 2).location, expected_second);
}

#[test]
fn ordinary_block_flow_uses_logical_block_progression_for_f32() {
    assert_ordinary_block_flow::<f32>(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::HorizontalTb,
        Direction::Rtl,
        Point::new(80.0, 0.0),
        Point::new(80.0, 10.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalLr,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalLr,
        Direction::Rtl,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysLr,
        Direction::Ltr,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysLr,
        Direction::Rtl,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
}

#[test]
fn ordinary_block_flow_uses_logical_block_progression_for_f64() {
    assert_ordinary_block_flow::<f64>(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::HorizontalTb,
        Direction::Rtl,
        Point::new(80.0, 0.0),
        Point::new(80.0, 10.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalLr,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalLr,
        Direction::Rtl,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysLr,
        Direction::Ltr,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysLr,
        Direction::Rtl,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
}

fn assert_ordinary_block_boundaries<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let container_size = Size::new(scalar(100.0), scalar(100.0));
    let child_logical_size = crate::geometry::LogicalSizeOf::new(scalar(20.0), scalar(10.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let child_size = flow_axes.physical_size(child_logical_size);
        let relative_inset = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            LengthAutoOf::px(scalar(3.0)),
            LengthAutoOf::AUTO,
            LengthAutoOf::px(scalar(5.0)),
            LengthAutoOf::AUTO,
        ));
        let relative_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(scalar(3.0), scalar(5.0)),
            child_logical_size,
            container_size,
        );
        let relative_tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    position: Position::Relative,
                    size: child_size.map(PreferredSizeOf::px),
                    inset: relative_inset,
                    ..NodeInputOf::default()
                },
            );
        let request =
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid");
        let relative =
            compute_layout(&relative_tree, 0, request).expect("relative block layout succeeds");

        assert_eq!(
            public_final_output(&relative, 1).location,
            relative_expected
        );

        let inline_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(S::ZERO, scalar(10.0)),
            child_logical_size,
            container_size,
        );
        let inline_tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let request =
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid");
        let inline =
            compute_layout(&inline_tree, 0, request).expect("inline block layout succeeds");

        assert_eq!(public_final_output(&inline, 2).location, inline_expected);

        let static_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(S::ZERO, scalar(10.0)),
            child_logical_size,
            container_size,
        );
        let static_tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    position: Position::Absolute,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let request =
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid");
        let static_position =
            compute_layout(&static_tree, 0, request).expect("static fallback layout succeeds");

        assert_eq!(
            public_final_output(&static_position, 2).location,
            static_expected
        );
    }
}

#[test]
fn ordinary_block_boundaries_project_through_containing_flow_for_f32() {
    assert_ordinary_block_boundaries::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_through_containing_flow_for_f64() {
    assert_ordinary_block_boundaries::<f64>();
}

fn assert_ordinary_block_boundary_baselines<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let container_size = Size::new(S::from_f64(100.0), S::from_f64(100.0));
    let logical_size = crate::geometry::LogicalSizeOf::new(S::from_f64(20.0), S::from_f64(10.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let child_size = flow_axes.physical_size(logical_size);
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::px(S::from_f64(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let output = crate::compute_block(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                container_size.map(Some),
                crate::ContainingLayoutContext::new(
                    flow_axes,
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(AvailableOf::definite(S::from_f64(100.0))),
            ),
        )
        .expect("block layout succeeds");

        let (expected_first, expected_last) =
            if flow_axes.block_axis() == crate::PhysicalAxis::Horizontal {
                let location = flow_axes.physical_point(
                    crate::geometry::LogicalPointOf::new(S::ZERO, S::from_f64(20.0)),
                    crate::geometry::LogicalSizeOf::new(S::ZERO, S::ZERO),
                    container_size,
                );
                let baseline = Point::new(Some(location.x), None);
                (baseline, baseline)
            } else {
                let baseline = Some(S::from_f64(20.0));
                (Point::new(None, baseline), Point::new(None, baseline))
            };
        assert_eq!(output.first_baselines, expected_first);
        assert_eq!(output.last_baselines, expected_last);
    }
}

#[test]
fn ordinary_block_boundaries_project_inline_baselines_for_f32() {
    assert_ordinary_block_boundary_baselines::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_inline_baselines_for_f64() {
    assert_ordinary_block_boundary_baselines::<f64>();
}

fn assert_ordinary_block_boundary_inline_report_overflow<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let root_size = Size::new(scalar(40.0), scalar(100.0));

    for (writing_mode, direction) in all_writing_mode_directions()
        .into_iter()
        .filter(|(writing_mode, _)| *writing_mode != WritingMode::HorizontalTb)
    {
        let expected_scrollable_overflow =
            ScrollRectOf::try_new(Point::ZERO, Size::new(scalar(40.0), scalar(100.0)))
                .expect("finite expected overflow rectangle");
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    text_align: TextAlign::LegacyCenter,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: root_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    writing_mode,
                    direction,
                    size: Size::splat_clone(PreferredSizeOf::px(scalar(20.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid"),
        )
        .expect("inline run layout succeeds");
        let root = public_final_output(&batch, 0);

        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
        assert_eq!(
            root.scroll_geometry
                .expect("root always has scroll geometry")
                .scrollable_overflow(),
            expected_scrollable_overflow,
        );
    }
}

#[test]
fn ordinary_block_boundaries_project_vertical_and_sideways_inline_report_overflow_for_f32() {
    assert_ordinary_block_boundary_inline_report_overflow::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_vertical_and_sideways_inline_report_overflow_for_f64() {
    assert_ordinary_block_boundary_inline_report_overflow::<f64>();
}

fn assert_ordinary_block_boundaries_keep_inline_content_coordinates<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let root_size = Size::new(scalar(50.0), scalar(50.0));
    let padding = Edges::new(
        LengthOf::px(scalar(2.0)),
        LengthOf::px(scalar(3.0)),
        LengthOf::px(scalar(5.0)),
        LengthOf::px(scalar(7.0)),
    );
    let border = Edges::new(
        LengthOf::px(scalar(1.0)),
        LengthOf::px(scalar(2.0)),
        LengthOf::px(scalar(3.0)),
        LengthOf::px(scalar(4.0)),
    );
    let expected_content_size = Size::new(scalar(40.0), scalar(45.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: root_size.map(PreferredSizeOf::px),
                    padding,
                    border,
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    writing_mode,
                    direction,
                    size: expected_content_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(50.0))))
                .expect("finite viewport is valid"),
        )
        .expect("padded inline block layout succeeds");
        let root = public_final_output(&batch, 0);
        let expected_scrollable_overflow = match (writing_mode, direction) {
            (WritingMode::HorizontalTb, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::HorizontalTb, Direction::Rtl)
            | (WritingMode::VerticalRl, Direction::Ltr)
            | (WritingMode::SidewaysRl, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(-2.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalRl, Direction::Rtl)
            | (WritingMode::SidewaysRl, Direction::Rtl) => ScrollRectOf::try_new(
                Point::new(scalar(-2.0), scalar(-5.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalLr, Direction::Ltr)
            | (WritingMode::SidewaysLr, Direction::Rtl) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalLr, Direction::Rtl)
            | (WritingMode::SidewaysLr, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(-5.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
        }
        .expect("finite expected scrollable overflow");

        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
        assert_eq!(
            root.scroll_geometry
                .expect("root always has scroll geometry")
                .scrollable_overflow(),
            expected_scrollable_overflow,
        );
    }
}

#[test]
fn ordinary_block_boundaries_keep_padded_inline_content_coordinates_for_f32() {
    assert_ordinary_block_boundaries_keep_inline_content_coordinates::<f32>();
}

#[test]
fn ordinary_block_boundaries_keep_padded_inline_content_coordinates_for_f64() {
    assert_ordinary_block_boundaries_keep_inline_content_coordinates::<f64>();
}

fn assert_ordinary_block_boundaries_use_logical_float_bfc_cursor<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;

    for writing_mode in [WritingMode::VerticalRl, WritingMode::VerticalLr] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    clear: Clear::Left,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid"),
        )
        .expect("vertical float and BFC layout succeeds");

        let flow_axes = FlowAxes::new(writing_mode, Direction::Ltr);
        let containing_size = Size::splat(scalar(100.0));
        for (node, expected_block) in [(2, 10.0), (3, 20.0)] {
            let output = public_final_output(&batch, node);
            assert_eq!(
                flow_axes
                    .logical_point(output.location, output.size, containing_size)
                    .block,
                scalar(expected_block),
            );
        }
    }
}

#[test]
fn ordinary_block_boundaries_use_vertical_logical_float_bfc_cursor_for_f32() {
    assert_ordinary_block_boundaries_use_logical_float_bfc_cursor::<f32>();
}

#[test]
fn ordinary_block_boundaries_use_vertical_logical_float_bfc_cursor_for_f64() {
    assert_ordinary_block_boundaries_use_logical_float_bfc_cursor::<f64>();
}

fn assert_ordinary_block_logical_sizing<S: LayoutScalar>(writing_mode: WritingMode) {
    let scalar = scalar_value::<S>;
    let percentage_thirty = LengthOf::value(scalar_percentage::<S>(0.0, 0.3));
    let percentage_sixty = LengthOf::value(scalar_percentage::<S>(0.0, 0.6));
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(100.0))),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                size: Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO),
                padding: Edges::new(
                    percentage_thirty,
                    LengthOf::ZERO,
                    percentage_sixty,
                    LengthOf::ZERO,
                ),
                ..NodeInputOf::default()
            },
        );
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
        .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");
    let root = public_final_output(&batch, 0);
    let child = public_final_output(&batch, 1);

    assert_eq!(root.size, Size::new(scalar(20.0), scalar(100.0)));
    assert_eq!(child.size, Size::new(scalar(20.0), scalar(100.0)));
    assert_eq!(child.padding.top, scalar(30.0));
    assert_eq!(child.padding.bottom, scalar(60.0));
}

#[test]
fn ordinary_block_logical_sizing_uses_vertical_and_sideways_inline_bases_for_f32() {
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::VerticalRl);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::VerticalLr);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::SidewaysRl);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::SidewaysLr);
}

#[test]
fn ordinary_block_logical_sizing_uses_vertical_and_sideways_inline_bases_for_f64() {
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::VerticalRl);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::VerticalLr);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::SidewaysRl);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::SidewaysLr);
}

fn assert_ordinary_block_collapse_relationship<S: LayoutScalar>(
    child_writing_mode: WritingMode,
    child_direction: Direction,
    measured_leaf: bool,
    expected_second_block_offset: S,
) {
    let scalar = scalar_value::<S>;
    let child_size = if child_writing_mode == WritingMode::HorizontalTb {
        Size::new(
            PreferredSizeOf::px(scalar(10.0)),
            PreferredSizeOf::px(S::ZERO),
        )
    } else {
        Size::new(
            PreferredSizeOf::px(S::ZERO),
            PreferredSizeOf::px(scalar(10.0)),
        )
    };
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: child_writing_mode,
                direction: child_direction,
                size: child_size,
                margin: Edges::new(
                    LengthAutoOf::px(scalar(30.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(10.0)),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        let measured = if child_writing_mode == WritingMode::HorizontalTb {
            Size::new(scalar(10.0), S::ZERO)
        } else {
            Size::new(S::ZERO, scalar(10.0))
        };
        tree = tree.with_measurement(1, measured);
    }
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
        .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

    assert_eq!(
        public_final_output(&batch, 2).location,
        Point::new(S::ZERO, expected_second_block_offset)
    );
}

fn assert_ordinary_block_relationship_matrix<S: LayoutScalar>() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            measured_leaf,
            scalar_value(60.0),
        );
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            measured_leaf,
            scalar_value(60.0),
        );
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::VerticalRl,
            Direction::Ltr,
            measured_leaf,
            scalar_value(100.0),
        );
    }

    for measured_leaf in [false, true] {
        let scalar = scalar_value::<S>;
        let mut tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalLr,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(200.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::HorizontalTb,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                    ..NodeInputOf::default()
                },
            );
        if measured_leaf {
            tree = tree.with_measurement(1, Size::new(scalar(5.0), scalar(10.0)));
        }
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(100.0)),
            AvailableOf::definite(scalar(200.0)),
        ))
        .expect("finite viewport is valid");

        let batch = compute_layout(&tree, 0, request).expect("orthogonal layout succeeds");
        assert_eq!(
            public_final_output(&batch, 1).size,
            Size::new(scalar(100.0), scalar(10.0))
        );
    }

    for child_direction in [Direction::Ltr, Direction::Rtl] {
        for measured_leaf in [false, true] {
            let scalar = scalar_value::<S>;
            let mut tree = PublicBlockTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: WritingMode::VerticalLr,
                        size: Size::new(
                            PreferredSizeOf::px(scalar(100.0)),
                            PreferredSizeOf::px(scalar(200.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: WritingMode::VerticalLr,
                        direction: child_direction,
                        size: Size::new(PreferredSizeOf::px(scalar(10.0)), PreferredSizeOf::AUTO),
                        ..NodeInputOf::default()
                    },
                );
            if measured_leaf {
                tree = tree.with_measurement(1, Size::new(scalar(10.0), scalar(5.0)));
            }
            let request = LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(scalar(100.0)),
                AvailableOf::definite(scalar(200.0)),
            ))
            .expect("finite viewport is valid");

            let batch = compute_layout(&tree, 0, request).expect("parallel layout succeeds");
            assert_eq!(
                public_final_output(&batch, 1).size,
                Size::new(scalar(10.0), scalar(200.0))
            );
        }
    }
}

fn assert_ordinary_block_opposing_flow_collapse<S: LayoutScalar>(measured_leaf: bool) {
    let scalar = scalar_value::<S>;
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::ZERO),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(30.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(10.0)),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        tree = tree.with_measurement(1, Size::new(S::ZERO, scalar(10.0)));
    }
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
        .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("opposing block layout succeeds");

    assert_eq!(
        public_final_output(&batch, 1).location,
        Point::new(scalar(30.0), S::ZERO)
    );
    assert_eq!(
        public_final_output(&batch, 2).location,
        Point::new(scalar(60.0), S::ZERO)
    );
}

fn assert_ordinary_block_opposing_flow_collapse_for_scalar<S: LayoutScalar>() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_opposing_flow_collapse::<S>(measured_leaf);
    }
}

fn assert_ordinary_block_orthogonal_inline_margin_subtraction<S: LayoutScalar>(
    measured_leaf: bool,
) {
    let scalar = scalar_value::<S>;
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(200.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(30.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        tree = tree.with_measurement(1, Size::new(scalar(5.0), scalar(10.0)));
    }
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(100.0)),
        AvailableOf::definite(scalar(200.0)),
    ))
    .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("orthogonal layout succeeds");

    assert_eq!(
        public_final_output(&batch, 1).size,
        Size::new(scalar(10.0), scalar(10.0))
    );
}

#[test]
fn ordinary_block_orthogonal_preserves_parallel_opposing_and_measured_leaf_relationships_for_f32() {
    assert_ordinary_block_relationship_matrix::<f32>();
}

#[test]
fn ordinary_block_orthogonal_preserves_parallel_opposing_and_measured_leaf_relationships_for_f64() {
    assert_ordinary_block_relationship_matrix::<f64>();
}

#[test]
fn ordinary_block_opposing_flow_collapse_preserves_real_and_measured_leaves_for_f32() {
    assert_ordinary_block_opposing_flow_collapse_for_scalar::<f32>();
}

#[test]
fn ordinary_block_opposing_flow_collapse_preserves_real_and_measured_leaves_for_f64() {
    assert_ordinary_block_opposing_flow_collapse_for_scalar::<f64>();
}

#[test]
fn ordinary_block_orthogonal_subtracts_physical_child_inline_margins_for_f32() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_orthogonal_inline_margin_subtraction::<f32>(measured_leaf);
    }
}

#[test]
fn ordinary_block_orthogonal_subtracts_physical_child_inline_margins_for_f64() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_orthogonal_inline_margin_subtraction::<f64>(measured_leaf);
    }
}

#[test]
fn block_layout_stacks_in_flow_children_vertically() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                right: Length::px(5.0),
                bottom: Length::px(7.0),
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(30.0, 12.0), Size::new(30.0, 12.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformRootLayout,
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

    assert_eq!(output.size, Size::new(100.0, 41.0));
    assert_eq!(output.content_size, Size::new(98.0, 39.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(18.0, 6.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        6.0
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(12.0, 21.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(30.0, 12.0)
    );
    assert_eq!(tree.inputs(2)[0].parent(), Size::new(Some(82.0), None));
    assert_eq!(tree.inputs(3)[0].parent(), Size::new(Some(82.0), None));
}

#[test]
fn block_in_flow_affine_margin_resolves_against_containing_block_width() {
    let mut tree = CalcBlockTree::default();
    let margin_left = lp(-4.0, 0.1);
    let width = lp(20.0, 0.5);
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::value(width), PreferredSize::AUTO),
            margin: Edges {
                left: LengthAuto::value(margin_left),
                right: LengthAuto::ZERO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(120.0, 10.0)));

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(200.0), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::Definite(200.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(120.0), None));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(16.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        16.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(120.0, 10.0)
    );
}

#[test]
fn block_container_affine_padding_uses_parent_basis() {
    let mut tree = CalcBlockTree::default();
    let padding = lp(2.0, 0.1);
    tree.insert_children(0, vec![1]);
    tree.insert_children(1, vec![]);
    tree.insert_style(
        0,
        NodeInput {
            padding: Edges::all(Length::value(padding)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(1, NodeInput::default());

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
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

    assert_eq!(output.content_size.width, 100.0);
}

#[test]
fn block_auto_width_includes_in_flow_child_horizontal_margins() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            item_is_table: true,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformRootLayout,
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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(3.0, 0.0)
    );
    assert_eq!(output.size, Size::new(32.0, 10.0));
    assert_eq!(output.content_size, Size::new(32.0, 10.0));
}

#[test]
fn block_layout_collapses_adjacent_in_flow_vertical_margins() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = crate::compute_block(
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
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 20.0)
    );
    assert_eq!(output.size, Size::new(100.0, 30.0));
    assert_eq!(output.content_size, Size::new(100.0, 30.0));
}

#[test]
fn block_layout_collapses_first_child_top_margin_through_parent() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
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
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        10.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        0.0
    );
}

#[test]
fn block_scroll_container_keeps_first_child_top_margin_inside() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
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
        Point::new(0.0, 10.0)
    );
    assert_eq!(output.size, Size::new(100.0, 15.0));
    assert_eq!(output.content_size, Size::new(100.0, 15.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        0.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        0.0
    );
    assert!(
        !output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

#[test]
fn block_rtl_scrollbar_gutter_uses_left_inset() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(17.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
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
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(83.0, 10.0)));

    crate::compute_block(
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
        Point::new(17.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(83.0, 10.0)
    );
}

#[test]
fn block_layout_collapses_last_child_bottom_margin_through_parent() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
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
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        0.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        10.0
    );
}

#[test]
fn block_layout_keeps_grid_child_margins_inside_parent_flow() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(50.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Grid,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(50.0, 20.0)));

    let output = crate::compute_block(
        &mut tree,
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

    assert_eq!(output.size, Size::new(50.0, 30.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.top,
        10.0
    );
}

#[test]
fn block_layout_collapses_margins_through_empty_in_flow_child() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            border: Edges {
                top: Length::px(1.0),
                right: Length::ZERO,
                bottom: Length::px(1.0),
                left: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                bottom: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(7.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    let mut empty_output = ComputeOutput::from_sizes(Size::new(100.0, 0.0), Size::new(100.0, 0.0));
    empty_output.block_margin_collapse = PhysicalBlockMarginCollapse::from_block_flow(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        CollapsibleMargin::ZERO,
        CollapsibleMargin::ZERO,
        true,
    );
    tree.insert_measure(2, empty_output);
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = crate::compute_block(
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
        Point::new(0.0, 11.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 11.0)
    );
    assert_eq!(output.size, Size::new(100.0, 22.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
}

#[test]
fn block_empty_auto_height_can_collapse_through() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
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
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 0.0));
    assert!(
        output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

#[test]
fn block_with_padding_reports_own_margins_when_child_collapse_is_blocked() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            margin: Edges {
                top: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            padding: Edges {
                top: Length::px(1.0),
                bottom: Length::px(1.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 2.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        8.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        6.0
    );
    assert!(
        !output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

fn assert_collapsible_percentage_margins_use_containing_inline_extent<S: LayoutScalar>(
    writing_mode: WritingMode,
) where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let top_margin = LengthPercentageOf::<S>::from_coefficients(S::ZERO, S::from_f64(0.25))
        .expect("test coefficients are finite");
    let bottom_margin = LengthPercentageOf::<S>::from_coefficients(S::ZERO, S::from_f64(0.5))
        .expect("test coefficients are finite");
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                margin: Edges {
                    top: LengthAutoOf::value(top_margin),
                    bottom: LengthAutoOf::value(bottom_margin),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                padding: Edges {
                    top: LengthOf::px(S::from_f64(1.0)),
                    bottom: LengthOf::px(S::from_f64(1.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(40.0)), Some(S::from_f64(120.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(writing_mode, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(40.0)),
                AvailableOf::definite(S::from_f64(120.0)),
            ),
        ),
    )
    .expect("block layout succeeds");

    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        S::from_f64(30.0)
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        S::from_f64(60.0)
    );
}

#[test]
fn collapsible_percentage_margins_use_non_horizontal_containing_inline_extent_for_f32() {
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f32>(
        WritingMode::VerticalRl,
    );
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn collapsible_percentage_margins_use_non_horizontal_containing_inline_extent_for_f64() {
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f64>(
        WritingMode::VerticalRl,
    );
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f64>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn block_in_flow_invalid_numeric_horizontal_margin_uses_zero_fallback() {
    let invalid_margin = LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX)
        .expect("test coefficients are finite");
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::AUTO),
                margin: Edges {
                    left: LengthAuto::value(invalid_margin),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(f32::MAX), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(f32::MAX), Available::MAX_CONTENT),
        ),
    )
    .expect("the in-flow invalid-numeric margin falls back to zero");

    assert_eq!(
        tree.output(2)
            .expect("child block receives an in-flow layout")
            .margin
            .left,
        0.0
    );
}

#[test]
fn block_layout_positions_in_flow_children_from_right_edge_in_rtl() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::ZERO,
                right: Length::px(5.0),
                bottom: Length::ZERO,
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(7.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(67.0, 1.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        7.0
    );
}

#[test]
fn block_layout_expands_horizontal_auto_margins_for_in_flow_children() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::AUTO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::AUTO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(40.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        40.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        40.0
    );
}

#[test]
fn block_content_size_includes_visible_child_overflow_content() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
    );

    let output = crate::compute_block(
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
fn block_relative_child_inset_offsets_final_layout_location() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
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
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(13.0, 3.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn block_layout_stretches_auto_width_in_flow_children() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::ZERO,
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                left: LengthAuto::px(3.0),
                right: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(76.0, 10.0)));

    let output = crate::compute_block(
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

    assert_eq!(tree.inputs(2)[0].known().width, Some(76.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(76.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(8.0, 0.0)
    );
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
    assert_eq!(output.size, Size::new(100.0, 10.0));
}

#[test]
fn block_compute_size_uses_in_flow_children_for_auto_height() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::px(7.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(76.0, 10.0)));

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
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

    assert_eq!(tree.inputs(2)[0].run_mode(), RunMode::ComputeSize);
    assert_eq!(tree.inputs(2)[0].known().width, Some(76.0));
    assert_eq!(output.size, Size::new(100.0, 26.0));
    assert_eq!(output.content_size, Size::ZERO);
    assert!(tree.layout(2).is_none());
}

#[test]
fn block_compute_size_uses_definite_min_max_without_measuring_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            min_size: Size::new(MinSize::px(100.0), MinSize::px(40.0)),
            max_size: Size::new(MaxSize::px(100.0), MaxSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = crate::compute_block(
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
fn block_definite_compute_size_keeps_grid_children_on_fast_path_until_grid_baselines() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            panic!("definite grid compute-size should stay on the fast path")
        }
    }

    for display in [Display::Grid, Display::GridLanes] {
        let mut tree = BlockTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![3]);
        tree.children.insert(3, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                min_size: Size::new(MinSize::px(100.0), MinSize::px(40.0)),
                max_size: Size::new(MaxSize::px(100.0), MaxSize::px(40.0)),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInput {
                display,
                ..NodeInput::default()
            },
        );
        tree.styles.insert(3, NodeInput::default());

        let output = crate::compute_block(
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
}

#[test]
fn block_auto_height_clamps_to_max_size() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            max_size: Size::new(MaxSize::NONE, MaxSize::px(12.0)),
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
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 20.0), Size::new(100.0, 20.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 20.0)
    );
}

#[test]
fn block_auto_size_applies_aspect_ratio_to_max_size() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            aspect_ratio: AspectRatio::new(2.0),
            max_size: Size::new(MaxSize::px(50.0), MaxSize::NONE),
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
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(50.0, 25.0));
}

#[test]
fn block_legacy_text_align_offsets_table_child_in_free_inline_space() {
    fn run(text_align: TextAlign, direction: Direction) -> NodeOutput {
        let mut tree = BlockTree::default();
        tree.insert_children(1, vec![2]);
        tree.insert_children(2, vec![]);
        tree.insert_style(
            1,
            NodeInput {
                display: Display::Block,
                direction,
                text_align,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        );
        tree.insert_style(
            2,
            NodeInput {
                display: Display::Block,
                item_is_table: true,
                ..NodeInput::default()
            },
        );
        tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(60.0, 10.0)));

        crate::compute_block(
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

        tree.layout(2).expect("child layout is staged")
    }

    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Ltr).location.x,
        70.0
    );
    assert_eq!(
        run(TextAlign::LegacyRight, Direction::Ltr).location.x,
        140.0
    );
    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Rtl).location.x,
        70.0
    );
    assert_eq!(run(TextAlign::LegacyLeft, Direction::Rtl).location.x, 0.0);
}
