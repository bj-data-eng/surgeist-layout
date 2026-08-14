use super::fixtures::{
    FlexTree, assert_fri07_c02_composition_finite_output, computed_overflow,
    fri07_c01_composition_output, fri07_c02_collapse_round_item, fri07_c02_collapse_round_output,
    fri07_c02_collapse_round_request,
};
use super::*;

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
