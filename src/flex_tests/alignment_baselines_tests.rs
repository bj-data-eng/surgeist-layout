use super::fixtures::{
    FlexTree, computed_overflow, fri05_c04_assert_flow_range, fri05_c04_flex_all_flow_axes,
    fri05_c04_flex_input, fri05_c04_flex_overflow_at_flow_axes, fri07_c02_collapse_round_item,
    fri07_c02_collapse_round_output, fri07_c02_collapse_round_request,
};
use super::*;

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
