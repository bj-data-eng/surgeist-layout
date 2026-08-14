use super::fixtures::{
    FlowRootLeafTree, PublicFlowTree, RootSessionTree, assert_fri06_c08_float_line_final_height,
    assert_fri06_c08_mixed_inline_atomic_x, assert_fri06_c08_r1_mixed_unit_traversal,
    computed_overflow, fri05_c03_root_all_flow_axes, fri05_c03_root_gutter_at,
    fri05_c03_tree_leaf_layout, fri06_atomic_participation, fri06_c02_expected_physical_rect,
    fri06_c02_final_node, fri06_c02_segment, fri06_c02_segment_with_level,
    fri06_c02_segment_with_metrics, fri06_c02_text_batch_with_flow, fri06_c03_atomic_participation,
    fri06_c03_mixed_batch_with_root, fri06_c03_text_input, fri06_c04_bfc_batch,
    fri06_c04_front_door_batch, fri06_c04_line_batch, fri06_c04_line_box, fri06_c04_logical_origin,
    logical_flex_leaf, public_flow_output, public_layout_tree, root_writing_mode_directions,
    scalar, single_final_output,
};
use super::*;

#[test]
fn root_and_hidden_contexts_are_explicit_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
        let incoming =
            crate::ContainingLayoutContext::new(axes, crate::ParentFormattingContext::Grid);
        let hidden = ComputeInputOf::<S>::hidden(incoming);
        assert_eq!(hidden.containing_layout_context(), incoming);
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
    assert_logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow::<f32>();
    assert_logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow::<f64>();
}

fn fri06_c03_projection_batch<S: LayoutScalar>(
    flow: FlowAxes,
    text_align: TextAlign,
    logical_atomics: &[(f64, f64, InlineBreakKind)],
    available_inline: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let root_input = NodeInputOf {
        display: Display::Block,
        writing_mode: flow.writing_mode(),
        direction: flow.direction(),
        text_align,
        ..NodeInputOf::default()
    };
    let mut inputs = HashMap::from([(0, LayoutInputOf::box_input(root_input.clone()))]);
    let mut children = HashMap::new();
    let mut root_children = Vec::new();
    for (offset, (inline, block, following_break)) in logical_atomics.iter().copied().enumerate() {
        let node = u32::try_from(offset + 1).unwrap();
        let following_break = match following_break {
            InlineBreakKind::Prohibited => InlineBreakOpportunityOf::prohibited(),
            InlineBreakKind::Allowed => InlineBreakOpportunityOf::allowed(),
            InlineBreakKind::Mandatory => InlineBreakOpportunityOf::mandatory(),
            InlineBreakKind::AllowedWithReplacement => {
                panic!("atomic projection fixtures never carry replacement breaks")
            }
        };
        let physical_size =
            flow.physical_size(LogicalSizeOf::new(S::from_f64(inline), S::from_f64(block)));
        let style = NodeInputOf {
            display: Display::InlineBlock,
            writing_mode: flow.writing_mode(),
            direction: flow.direction(),
            size: physical_size.map(PreferredSizeOf::px),
            atomic_inline_participation: Some(fri06_c03_atomic_participation(0, following_break)),
            ..NodeInputOf::default()
        };
        inputs.insert(node, LayoutInputOf::box_input(style.clone()));
        children.insert(node, Vec::new());
        root_children.push(node);
    }
    children.insert(0, root_children);
    let tree = public_layout_tree(inputs, children);
    let available = flow.physical_size(LogicalSizeOf::new(
        AvailableOf::definite(S::from_f64(available_inline)),
        AvailableOf::MAX_CONTENT,
    ));
    compute_layout(&tree, 0, LayoutRootRequestOf::viewport(available).unwrap()).unwrap()
}

#[test]
fn fri06_c03_projection_soft_and_forced_unequal_atomic_lines_align_per_line_in_all_flows_both_scalars()
 {
    fn expected_offset(
        align: TextAlign,
        decreases: bool,
        containing_inline: f64,
        used_inline: f64,
    ) -> f64 {
        let free = containing_inline - used_inline;
        match align {
            TextAlign::LegacyLeft if decreases => free,
            TextAlign::LegacyRight if !decreases => free,
            TextAlign::LegacyCenter => free / 2.0,
            TextAlign::Auto | TextAlign::LegacyLeft | TextAlign::LegacyRight => 0.0,
        }
    }

    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in fri06_c02_flow_mappings() {
            let flow = FlowAxes::new(writing_mode, direction);
            let decreases = fri06_c02_inline_decreases(writing_mode, direction);
            for align in [
                TextAlign::LegacyLeft,
                TextAlign::LegacyRight,
                TextAlign::LegacyCenter,
            ] {
                let soft = fri06_c03_projection_batch::<S>(
                    flow,
                    align,
                    &[
                        (30.0, 10.0, InlineBreakKind::Allowed),
                        (30.0, 10.0, InlineBreakKind::Allowed),
                        (10.0, 10.0, InlineBreakKind::Prohibited),
                    ],
                    40.0,
                );
                let soft_root = fri06_c02_final_node(&soft, 0);
                for (node, logical_rect) in [
                    (
                        1,
                        (
                            expected_offset(align, decreases, 40.0, 30.0),
                            0.0,
                            30.0,
                            10.0,
                        ),
                    ),
                    (
                        2,
                        (
                            expected_offset(align, decreases, 40.0, 40.0),
                            10.0,
                            30.0,
                            10.0,
                        ),
                    ),
                    (
                        3,
                        (
                            expected_offset(align, decreases, 40.0, 40.0) + 30.0,
                            10.0,
                            10.0,
                            10.0,
                        ),
                    ),
                ] {
                    let expected = fri06_c02_expected_physical_rect(
                        (writing_mode, direction),
                        logical_rect,
                        (40.0, 20.0),
                    );
                    let output = fri06_c02_final_node(&soft, node);
                    assert_eq!(
                        (output.location, output.size),
                        expected,
                        "soft {writing_mode:?} {direction:?} {align:?} node {node}; root={soft_root:?}"
                    );
                }

                let forced = fri06_c03_projection_batch::<S>(
                    flow,
                    align,
                    &[
                        (30.0, 10.0, InlineBreakKind::Mandatory),
                        (10.0, 10.0, InlineBreakKind::Prohibited),
                    ],
                    100.0,
                );
                for (node, logical_rect) in [
                    (
                        1,
                        (
                            expected_offset(align, decreases, 100.0, 30.0),
                            0.0,
                            30.0,
                            10.0,
                        ),
                    ),
                    (
                        2,
                        (
                            expected_offset(align, decreases, 100.0, 10.0),
                            10.0,
                            10.0,
                            10.0,
                        ),
                    ),
                ] {
                    let expected = fri06_c02_expected_physical_rect(
                        (writing_mode, direction),
                        logical_rect,
                        (100.0, 20.0),
                    );
                    let output = fri06_c02_final_node(&forced, node);
                    assert_eq!(
                        (output.location, output.size),
                        expected,
                        "forced {writing_mode:?} {direction:?} {align:?} node {node}"
                    );
                }
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_percentage_definite_physical_and_logical_block_basis_without_indefinite_substitute() {
    fn assert_case<S: LayoutScalar>(
        flow_axes: FlowAxes,
        containing_block: Option<f64>,
        fraction: f64,
        expected_block: f64,
    ) {
        let root_logical_size = LogicalSizeOf::new(
            PreferredSizeOf::AUTO,
            containing_block.map_or(PreferredSizeOf::AUTO, |extent| {
                PreferredSizeOf::px(S::from_f64(extent))
            }),
        );
        let root_style = NodeInputOf {
            display: Display::Block,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            size: flow_axes.physical_size(root_logical_size),
            ..NodeInputOf::default()
        };
        let atomic_logical_size = LogicalSizeOf::new(
            PreferredSizeOf::px(S::from_f64(10.0)),
            PreferredSizeOf::percent(S::from_f64(fraction)),
        );
        let atomic_style = NodeInputOf {
            display: Display::InlineBlock,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            size: flow_axes.physical_size(atomic_logical_size),
            atomic_inline_participation: Some(fri06_c03_atomic_participation(
                0,
                InlineBreakOpportunityOf::prohibited(),
            )),
            ..NodeInputOf::default()
        };
        let tree = public_layout_tree(
            HashMap::from([
                (0, LayoutInputOf::box_input(root_style.clone())),
                (1, LayoutInputOf::box_input(atomic_style.clone())),
            ]),
            HashMap::from([(0, vec![1]), (1, Vec::new())]),
        );
        let viewport = flow_axes.physical_size(LogicalSizeOf::new(
            AvailableOf::definite(S::from_f64(80.0)),
            AvailableOf::MAX_CONTENT,
        ));
        let batch =
            compute_layout(&tree, 0, LayoutRootRequestOf::viewport(viewport).unwrap()).unwrap();
        let atomic = fri06_c02_final_node(&batch, 1);
        assert_eq!(
            flow_axes.logical_size(atomic.size).block,
            S::from_f64(expected_block),
            "percentage block basis for {flow_axes:?} and {containing_block:?}"
        );
    }

    fn assert_lane<S: LayoutScalar>() {
        assert_case::<S>(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Some(100.0),
            0.5,
            50.0,
        );
        assert_case::<S>(
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
            Some(120.0),
            0.25,
            30.0,
        );
        assert_case::<S>(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            None,
            0.5,
            0.0,
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_clear_all_values_accept_all_containing_flows_without_exclusions_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let writing_modes = [
            WritingMode::HorizontalTb,
            WritingMode::VerticalRl,
            WritingMode::VerticalLr,
            WritingMode::SidewaysRl,
            WritingMode::SidewaysLr,
        ];
        let directions = [Direction::Ltr, Direction::Rtl];
        let clears = [Clear::None, Clear::Left, Clear::Right, Clear::Both];
        for writing_mode in writing_modes {
            for direction in directions {
                for clear in clears {
                    let metrics = InlineMetricsOf::from_line_height_and_baseline(
                        S::from_f64(10.0),
                        S::from_f64(7.0),
                    )
                    .unwrap();
                    let batch = fri06_c03_mixed_batch_with_root(
                        vec![(
                            1,
                            LayoutInputOf::line_break(
                                LineBreakInputOf::new()
                                    .with_writing_mode(writing_mode)
                                    .with_direction(direction)
                                    .with_clear(clear)
                                    .with_metrics(metrics),
                            ),
                            NodeInputOf::non_box(),
                        )],
                        AvailableOf::definite(S::from_f64(80.0)),
                        NodeInputOf {
                            writing_mode,
                            direction,
                            ..NodeInputOf::default()
                        },
                    );

                    assert_eq!(fri06_c02_final_node(&batch, 1).size, Size::ZERO);
                }
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

struct Fri06C07DirectTree<S: LayoutScalar>(PublicLayoutTreeOf<S>);

impl<S: LayoutScalar> Traverse for Fri06C07DirectTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.0, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.0.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.0.child(node, index)
    }
}

impl<S: LayoutScalar> Compute<()> for Fri06C07DirectTree<S> {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.0.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.0.layout_input(node)
    }

    fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutputOf<S>) {}

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S, ()> {
        match self.0.layout_input(node) {
            LayoutInputOf::Box(style) => {
                assert_eq!(style.display.inner_display(), Display::Block);
                compute_block(self, node, input)
            }
            LayoutInputOf::InlineText(_) => {
                let size = Size::new(
                    input.known().width.unwrap_or(S::ZERO),
                    input.known().height.unwrap_or(S::ZERO),
                );
                Ok(ComputeOutputOf::from_sizes(size, size))
            }
            LayoutInputOf::LineBreak(_) | LayoutInputOf::InlineBoundary(_) => {
                unreachable!("block layout consumes controls without child computation")
            }
        }
    }
}

#[test]
fn fri06_c07_logical_clear_vertical_control_and_float_projection_both_scalars() {
    #[derive(Clone, Copy)]
    enum Family {
        VerticalBreak,
        LogicalFloat,
    }

    #[derive(Clone, Copy)]
    struct Row {
        source: &'static str,
        variant: &'static str,
        family: Family,
        direction: Direction,
        box_sizing: BoxSizing,
    }

    const ROWS: [Row; 8] = [
        Row {
            source: "html/block/fri06_vertical_break_clear.html",
            variant: "border_box_ltr",
            family: Family::VerticalBreak,
            direction: Direction::Ltr,
            box_sizing: BoxSizing::BorderBox,
        },
        Row {
            source: "html/block/fri06_vertical_break_clear.html",
            variant: "content_box_ltr",
            family: Family::VerticalBreak,
            direction: Direction::Ltr,
            box_sizing: BoxSizing::ContentBox,
        },
        Row {
            source: "html/block/fri06_vertical_break_clear.html",
            variant: "border_box_rtl",
            family: Family::VerticalBreak,
            direction: Direction::Rtl,
            box_sizing: BoxSizing::BorderBox,
        },
        Row {
            source: "html/block/fri06_vertical_break_clear.html",
            variant: "content_box_rtl",
            family: Family::VerticalBreak,
            direction: Direction::Rtl,
            box_sizing: BoxSizing::ContentBox,
        },
        Row {
            source: "html/float/fri06_float_logical_clear.html",
            variant: "border_box_ltr",
            family: Family::LogicalFloat,
            direction: Direction::Ltr,
            box_sizing: BoxSizing::BorderBox,
        },
        Row {
            source: "html/float/fri06_float_logical_clear.html",
            variant: "content_box_ltr",
            family: Family::LogicalFloat,
            direction: Direction::Ltr,
            box_sizing: BoxSizing::ContentBox,
        },
        Row {
            source: "html/float/fri06_float_logical_clear.html",
            variant: "border_box_rtl",
            family: Family::LogicalFloat,
            direction: Direction::Rtl,
            box_sizing: BoxSizing::BorderBox,
        },
        Row {
            source: "html/float/fri06_float_logical_clear.html",
            variant: "content_box_rtl",
            family: Family::LogicalFloat,
            direction: Direction::Rtl,
            box_sizing: BoxSizing::ContentBox,
        },
    ];

    fn vertical_break_batch<S: LayoutScalar>(
        row: Row,
    ) -> (CompletedLayoutBatchOf<u32, S>, ComputeOutputOf<S>) {
        let flow_axes = FlowAxes::new(WritingMode::VerticalRl, row.direction);
        let root_style = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            direction: row.direction,
            box_sizing: row.box_sizing,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(40.0)),
                PreferredSizeOf::px(S::from_f64(40.0)),
            ),
            ..NodeInputOf::default()
        };
        let float_style = fri06_c04_line_box(
            flow_axes,
            LogicalSizeOf::new(S::ZERO, S::from_f64(20.0)),
            Float::Left,
            None,
        );
        let text = fri06_c03_text_input(vec![fri06_c02_segment(
            470,
            10.0,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);
        let metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();

        let break_input = LineBreakInputOf::new()
            .with_writing_mode(WritingMode::VerticalRl)
            .with_direction(row.direction)
            .with_metrics(metrics)
            .with_clear(Clear::Left);
        let tree = public_layout_tree(
            HashMap::from([
                (0, LayoutInputOf::box_input(root_style.clone())),
                (1, LayoutInputOf::box_input(float_style.clone())),
                (2, text),
                (3, LayoutInputOf::line_break(break_input)),
            ]),
            HashMap::from([
                (0, vec![1, 2, 3]),
                (1, Vec::new()),
                (2, Vec::new()),
                (3, Vec::new()),
            ]),
        );
        let available = flow_axes.physical_size(LogicalSizeOf::new(
            AvailableOf::definite(S::from_f64(40.0)),
            AvailableOf::definite(S::from_f64(40.0)),
        ));
        let batch =
            compute_layout(&tree, 0, LayoutRootRequestOf::viewport(available).unwrap()).unwrap();
        let mut direct_tree = Fri06C07DirectTree(tree);
        let root_output = compute_block(
            &mut direct_tree,
            0,
            ComputeInputOf::root_layout(
                Size::NONE,
                available.map(AvailableOf::into_option),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                available,
            ),
        )
        .unwrap();

        (batch, root_output)
    }

    fn logical_float_batch<S: LayoutScalar>(
        row: Row,
        float_side: Float,
        clear: Clear,
        float_logical_size: LogicalSizeOf<S>,
    ) -> CompletedLayoutBatchOf<u32, S> {
        let flow_axes = FlowAxes::new(WritingMode::VerticalRl, row.direction);
        let root_style = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            direction: row.direction,
            box_sizing: row.box_sizing,
            size: flow_axes
                .physical_size(LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(160.0)))
                .map(PreferredSizeOf::px),
            ..NodeInputOf::default()
        };
        let float_style = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            direction: row.direction,
            float: float_side,
            size: flow_axes
                .physical_size(float_logical_size)
                .map(PreferredSizeOf::px),
            ..NodeInputOf::default()
        };
        let cleared_style = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            direction: row.direction,
            clear,
            size: flow_axes
                .physical_size(LogicalSizeOf::new(S::from_f64(50.0), S::from_f64(10.0)))
                .map(PreferredSizeOf::px),
            ..NodeInputOf::default()
        };

        fri06_c04_front_door_batch(
            root_style,
            LogicalSizeOf::new(
                AvailableOf::definite(S::from_f64(100.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
            vec![1, 2],
            vec![
                (
                    1,
                    LayoutInputOf::box_input(float_style.clone()),
                    float_style,
                    Vec::new(),
                ),
                (
                    2,
                    LayoutInputOf::box_input(cleared_style.clone()),
                    cleared_style,
                    Vec::new(),
                ),
            ],
        )
    }

    fn assert_lane<S: LayoutScalar>() {
        let unique = ROWS
            .iter()
            .map(|row| (row.source, row.variant))
            .collect::<HashSet<_>>();
        assert_eq!(ROWS.len(), 8);
        assert_eq!(unique.len(), 8);

        for row in ROWS {
            match row.family {
                Family::VerticalBreak => {
                    let (batch, root_output) = vertical_break_batch::<S>(row);
                    let break_output = fri06_c02_final_node(&batch, 3);
                    let expected_break = match row.direction {
                        Direction::Ltr => Point::new(S::from_f64(25.0), S::from_f64(10.0)),
                        Direction::Rtl => Point::new(S::from_f64(25.0), S::from_f64(30.0)),
                    };
                    assert_eq!(
                        break_output.location, expected_break,
                        "{} {}",
                        row.source, row.variant
                    );
                    assert_eq!(break_output.size, Size::ZERO);
                    let expected_exclusion = match row.direction {
                        Direction::Ltr => Point::new(S::from_f64(20.0), S::ZERO),
                        Direction::Rtl => Point::new(S::from_f64(20.0), S::from_f64(40.0)),
                    };
                    assert_eq!(fri06_c02_final_node(&batch, 1).location, expected_exclusion);
                    assert_eq!(
                        fri06_c02_final_node(&batch, 1).size,
                        Size::new(S::from_f64(20.0), S::ZERO)
                    );
                    let root_node_output = fri06_c02_final_node(&batch, 0);
                    assert_eq!(root_node_output.size, Size::splat(S::from_f64(40.0)));
                    assert_eq!(root_output.size, Size::splat(S::from_f64(40.0)));
                    let baselines = root_output.baselines();
                    assert_eq!(
                        baselines,
                        BaselinesOf {
                            first: Point::new(Some(S::from_f64(25.0)), None),
                            last: Point::new(Some(S::from_f64(5.0)), None),
                        },
                        "{} {} root baselines",
                        row.source,
                        row.variant,
                    );
                    assert_eq!(
                        root_output.size.width - baselines.first.x.unwrap(),
                        S::from_f64(15.0),
                        "{} {} break logical block baseline",
                        row.source,
                        row.variant,
                    );
                    assert_eq!(
                        root_output.size.width - baselines.last.x.unwrap(),
                        S::from_f64(35.0),
                        "{} {} following-strut logical block baseline",
                        row.source,
                        row.variant,
                    );
                }
                Family::LogicalFloat => {
                    let line_start = logical_float_batch::<S>(
                        row,
                        Float::Left,
                        Clear::Left,
                        LogicalSizeOf::new(S::from_f64(20.0), S::from_f64(20.0)),
                    );
                    let line_end = logical_float_batch::<S>(
                        row,
                        Float::Right,
                        Clear::Right,
                        LogicalSizeOf::new(S::from_f64(30.0), S::from_f64(40.0)),
                    );
                    assert_eq!(
                        fri06_c02_final_node(&line_start, 0).size,
                        Size::new(S::from_f64(160.0), S::from_f64(100.0)),
                    );
                    assert_eq!(
                        fri06_c02_final_node(&line_end, 0).size,
                        Size::new(S::from_f64(160.0), S::from_f64(100.0)),
                    );
                    let expected_start_float = match row.direction {
                        Direction::Ltr => Point::new(S::from_f64(140.0), S::ZERO),
                        Direction::Rtl => Point::new(S::from_f64(140.0), S::from_f64(80.0)),
                    };
                    let expected_end_float = match row.direction {
                        Direction::Ltr => Point::new(S::from_f64(120.0), S::from_f64(70.0)),
                        Direction::Rtl => Point::new(S::from_f64(120.0), S::ZERO),
                    };
                    assert_eq!(
                        fri06_c02_final_node(&line_start, 1).location,
                        expected_start_float,
                        "{} {} line-start float",
                        row.source,
                        row.variant,
                    );
                    assert_eq!(
                        fri06_c02_final_node(&line_end, 1).location,
                        expected_end_float,
                        "{} {} line-end float",
                        row.source,
                        row.variant,
                    );
                    assert_eq!(
                        fri06_c02_final_node(&line_start, 1).size,
                        Size::splat(S::from_f64(20.0))
                    );
                    assert_eq!(
                        fri06_c02_final_node(&line_end, 1).size,
                        Size::new(S::from_f64(40.0), S::from_f64(30.0))
                    );
                    let expected_start = match row.direction {
                        Direction::Ltr => Point::new(S::from_f64(130.0), S::ZERO),
                        Direction::Rtl => Point::new(S::from_f64(130.0), S::from_f64(50.0)),
                    };
                    let expected_end = match row.direction {
                        Direction::Ltr => Point::new(S::from_f64(110.0), S::ZERO),
                        Direction::Rtl => Point::new(S::from_f64(110.0), S::from_f64(50.0)),
                    };
                    assert_eq!(
                        fri06_c02_final_node(&line_start, 2).location,
                        expected_start,
                        "{} {} line-start clear",
                        row.source,
                        row.variant
                    );
                    assert_eq!(
                        fri06_c02_final_node(&line_end, 2).location,
                        expected_end,
                        "{} {} line-end clear",
                        row.source,
                        row.variant
                    );
                    assert_eq!(
                        fri06_c02_final_node(&line_start, 2).size,
                        Size::new(S::from_f64(10.0), S::from_f64(50.0))
                    );
                    assert_eq!(
                        fri06_c02_final_node(&line_end, 2).size,
                        Size::new(S::from_f64(10.0), S::from_f64(50.0))
                    );
                }
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_clear_all_flows_values_and_matching_sides_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
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
            for float_side in [Float::Left, Float::Right] {
                for clear in [Clear::None, Clear::Left, Clear::Right, Clear::Both] {
                    let float_style = fri06_c04_line_box(
                        flow_axes,
                        LogicalSizeOf::new(S::from_f64(30.0), S::from_f64(30.0)),
                        float_side,
                        None,
                    );
                    let atomic_style = |following_break| {
                        fri06_c04_line_box(
                            flow_axes,
                            LogicalSizeOf::new(S::from_f64(10.0), S::from_f64(10.0)),
                            Float::None,
                            Some(fri06_c03_atomic_participation(0, following_break)),
                        )
                    };
                    let first = atomic_style(InlineBreakOpportunityOf::prohibited());
                    let second = atomic_style(InlineBreakOpportunityOf::prohibited());
                    let metrics = InlineMetricsOf::from_line_height_and_baseline(
                        S::from_f64(10.0),
                        S::from_f64(8.0),
                    )
                    .unwrap();
                    let batch = fri06_c04_line_batch(
                        flow_axes,
                        TextAlign::Auto,
                        vec![
                            (
                                1,
                                LayoutInputOf::box_input(float_style.clone()),
                                float_style,
                            ),
                            (2, LayoutInputOf::box_input(first.clone()), first),
                            (
                                3,
                                LayoutInputOf::line_break(
                                    LineBreakInputOf::new()
                                        .with_writing_mode(writing_mode)
                                        .with_direction(direction)
                                        .with_metrics(metrics)
                                        .with_clear(clear),
                                ),
                                NodeInputOf::non_box(),
                            ),
                            (4, LayoutInputOf::box_input(second.clone()), second),
                        ],
                    );
                    let matching = clear == Clear::Both
                        || clear == Clear::Left && float_side == Float::Left
                        || clear == Clear::Right && float_side == Float::Right;
                    let second_origin =
                        fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&batch, 4));
                    assert_eq!(
                        second_origin.block,
                        if matching {
                            S::from_f64(30.0)
                        } else {
                            S::from_f64(12.0)
                        },
                        "clear mismatch for {writing_mode:?} {direction:?} {float_side:?} {clear:?}",
                    );
                    assert_eq!(fri06_c02_final_node(&batch, 3).size, Size::ZERO);
                }
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_alignment_legacy_values_use_each_final_band_all_flows_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
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
            let inline_decreases = flow_axes
                .logical_axis_progression(LogicalAxis::Inline)
                .is_decreasing();
            for align in [
                TextAlign::LegacyLeft,
                TextAlign::LegacyRight,
                TextAlign::LegacyCenter,
            ] {
                let float_style = fri06_c04_line_box(
                    flow_axes,
                    LogicalSizeOf::new(S::from_f64(30.0), S::from_f64(30.0)),
                    Float::Left,
                    None,
                );
                let atomic = |inline| {
                    fri06_c04_line_box(
                        flow_axes,
                        LogicalSizeOf::new(S::from_f64(inline), S::from_f64(10.0)),
                        Float::None,
                        Some(fri06_c03_atomic_participation(
                            0,
                            InlineBreakOpportunityOf::prohibited(),
                        )),
                    )
                };
                let first = atomic(20.0);
                let second = atomic(10.0);
                let batch = fri06_c04_line_batch(
                    flow_axes,
                    align,
                    vec![
                        (
                            1,
                            LayoutInputOf::box_input(float_style.clone()),
                            float_style,
                        ),
                        (2, LayoutInputOf::box_input(first.clone()), first),
                        (
                            3,
                            LayoutInputOf::line_break(
                                LineBreakInputOf::new()
                                    .with_writing_mode(writing_mode)
                                    .with_direction(direction)
                                    .with_metrics(
                                        InlineMetricsOf::from_line_height_and_baseline(
                                            S::from_f64(10.0),
                                            S::from_f64(8.0),
                                        )
                                        .unwrap(),
                                    ),
                            ),
                            NodeInputOf::non_box(),
                        ),
                        (4, LayoutInputOf::box_input(second.clone()), second),
                    ],
                );
                let expected = |used: f64| {
                    let free = 70.0 - used;
                    let offset = match align {
                        TextAlign::LegacyCenter => free / 2.0,
                        TextAlign::LegacyLeft if inline_decreases => free,
                        TextAlign::LegacyRight if !inline_decreases => free,
                        TextAlign::LegacyLeft | TextAlign::LegacyRight | TextAlign::Auto => 0.0,
                    };
                    S::from_f64(30.0 + offset)
                };
                assert_eq!(
                    fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&batch, 2)).inline,
                    expected(20.0),
                );
                assert_eq!(
                    fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&batch, 4)).inline,
                    expected(10.0),
                );
                assert_eq!(fri06_c02_final_node(&batch, 0).location, Point::ZERO);
                assert_eq!(
                    flow_axes
                        .logical_size(fri06_c02_final_node(&batch, 0).size)
                        .inline,
                    S::from_f64(100.0),
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_bfc_nested_bfc_floating_and_atomic_contexts_trap_internal_floats_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let fixed = |display, inline, block| NodeInputOf {
            display,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            size: flow_axes.physical_size(LogicalSizeOf::new(
                PreferredSizeOf::px(S::from_f64(inline)),
                PreferredSizeOf::px(S::from_f64(block)),
            )),
            ..NodeInputOf::default()
        };
        let floated = |inline, block| NodeInputOf {
            float: Float::Left,
            ..fixed(Display::Block, inline, block)
        };
        let bfc = |inline, block| NodeInputOf {
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ..fixed(Display::Block, inline, block)
        };

        let nested_bfc = fri06_c04_bfc_batch(
            flow_axes,
            vec![1, 2, 4],
            vec![
                (1, floated(30.0, 20.0), Vec::new()),
                (2, bfc(70.0, 20.0), vec![3]),
                (3, floated(20.0, 50.0), Vec::new()),
                (4, bfc(100.0, 10.0), Vec::new()),
            ],
        );
        assert_eq!(
            fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&nested_bfc, 2)),
            LogicalPointOf::new(S::from_f64(30.0), S::ZERO),
        );
        assert_eq!(
            fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&nested_bfc, 3)),
            LogicalPointOf::new(S::ZERO, S::ZERO),
        );
        assert_eq!(
            fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&nested_bfc, 4)),
            LogicalPointOf::new(S::ZERO, S::from_f64(20.0)),
        );

        let floating_context = fri06_c04_bfc_batch(
            flow_axes,
            vec![1, 3],
            vec![
                (1, floated(30.0, 20.0), vec![2]),
                (2, floated(10.0, 50.0), Vec::new()),
                (3, bfc(70.0, 10.0), Vec::new()),
            ],
        );
        assert_eq!(
            fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&floating_context, 1)),
            LogicalPointOf::new(S::ZERO, S::ZERO),
        );
        assert_eq!(
            fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&floating_context, 3)),
            LogicalPointOf::new(S::from_f64(30.0), S::ZERO),
        );

        let atomic = NodeInputOf {
            atomic_inline_participation: Some(fri06_c03_atomic_participation(
                0,
                InlineBreakOpportunityOf::prohibited(),
            )),
            ..fixed(Display::InlineBlock, 30.0, 20.0)
        };
        let atomic_context = fri06_c04_bfc_batch(
            flow_axes,
            vec![1, 2, 4],
            vec![
                (1, floated(30.0, 20.0), Vec::new()),
                (2, atomic, vec![3]),
                (3, floated(10.0, 50.0), Vec::new()),
                (4, bfc(100.0, 10.0), Vec::new()),
            ],
        );
        assert_eq!(
            fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&atomic_context, 2)),
            LogicalPointOf::new(S::from_f64(30.0), S::ZERO),
        );
        assert_eq!(
            fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&atomic_context, 4)),
            LogicalPointOf::new(S::ZERO, S::from_f64(20.0)),
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_size_intrinsics_keep_logical_clear_and_overwide_contributions_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for flow_axes in [
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        ] {
            let root_style = NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                ..NodeInputOf::default()
            };
            let make_float = |side, clear, inline| {
                let style = NodeInputOf {
                    display: Display::Block,
                    writing_mode: flow_axes.writing_mode(),
                    direction: flow_axes.direction(),
                    float: side,
                    clear,
                    size: flow_axes
                        .physical_size(LogicalSizeOf::new(S::from_f64(inline), S::from_f64(5.0)))
                        .map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                };
                (LayoutInputOf::box_input(style.clone()), style)
            };
            let (first_input, first) = make_float(Float::Left, Clear::None, 30.0);
            let (second_input, second) = make_float(Float::Right, Clear::None, 40.0);
            let (overwide_input, overwide) = make_float(Float::Left, Clear::Both, 80.0);
            let nodes = vec![
                (1, first_input, first, Vec::new()),
                (2, second_input, second, Vec::new()),
                (3, overwide_input, overwide, Vec::new()),
            ];
            for (available, expected) in [
                (AvailableOf::MIN_CONTENT, S::from_f64(80.0)),
                (AvailableOf::MAX_CONTENT, S::from_f64(80.0)),
            ] {
                let batch = fri06_c04_front_door_batch(
                    root_style.clone(),
                    LogicalSizeOf::new(available, AvailableOf::MAX_CONTENT),
                    vec![1, 2, 3],
                    nodes.clone(),
                );
                assert_eq!(
                    flow_axes
                        .logical_size(fri06_c02_final_node(&batch, 0).size)
                        .inline,
                    expected,
                    "intrinsic floats remain logical and clear starts a new contribution band for {flow_axes:?}",
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_r1_mixed_units_project_complete_traversal_in_all_flow_mappings() {
    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in fri06_c02_flow_mappings() {
            assert_fri06_c08_r1_mixed_unit_traversal::<S>(
                FlowAxes::new(writing_mode, direction),
                BoxSizing::ContentBox,
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_mixed_inline_ltr_border_box_preserves_logical_placement() {
    assert_fri06_c08_mixed_inline_atomic_x::<f32>(Direction::Ltr, BoxSizing::BorderBox, 99.0);
    assert_fri06_c08_mixed_inline_atomic_x::<f64>(Direction::Ltr, BoxSizing::BorderBox, 99.0);
}

#[test]
fn fri06_c08_mixed_inline_ltr_content_box_preserves_logical_placement() {
    assert_fri06_c08_mixed_inline_atomic_x::<f32>(Direction::Ltr, BoxSizing::ContentBox, 99.0);
    assert_fri06_c08_mixed_inline_atomic_x::<f64>(Direction::Ltr, BoxSizing::ContentBox, 99.0);
}

#[test]
fn fri06_c12_t08_rtl_float_line_slots_project_logical_progression_once() {
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Rtl, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Rtl, BoxSizing::BorderBox);
}

#[test]
fn fri06_c12_t08_ltr_float_line_slots_preserve_logical_progression() {
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Ltr, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Ltr, BoxSizing::BorderBox);
}

fn fri06_c02_flow_mappings() -> [(WritingMode, Direction); 10] {
    [
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
    ]
}

fn fri06_c02_inline_decreases(writing_mode: WritingMode, direction: Direction) -> bool {
    match writing_mode {
        WritingMode::HorizontalTb
        | WritingMode::VerticalRl
        | WritingMode::VerticalLr
        | WritingMode::SidewaysRl => direction == Direction::Rtl,
        WritingMode::SidewaysLr => direction == Direction::Ltr,
    }
}

#[test]
fn fri06_c02_alignment_uses_each_unequal_line_extent_and_clamps_overflow_in_all_flows_both_scalars()
{
    fn expected_offset(
        align: TextAlign,
        decreases: bool,
        containing_inline: f64,
        used_inline: f64,
    ) -> f64 {
        let free = (containing_inline - used_inline).max(0.0);
        match align {
            TextAlign::Auto => 0.0,
            TextAlign::LegacyLeft if decreases => free,
            TextAlign::LegacyRight if !decreases => free,
            TextAlign::LegacyCenter => free / 2.0,
            TextAlign::LegacyLeft | TextAlign::LegacyRight => 0.0,
        }
    }

    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in fri06_c02_flow_mappings() {
            let decreases = fri06_c02_inline_decreases(writing_mode, direction);
            for align in [
                TextAlign::Auto,
                TextAlign::LegacyLeft,
                TextAlign::LegacyRight,
                TextAlign::LegacyCenter,
            ] {
                let batch = fri06_c02_text_batch_with_flow(
                    vec![
                        fri06_c02_segment_with_level(
                            1,
                            30.0,
                            0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::mandatory(),
                        ),
                        fri06_c02_segment_with_level(
                            2,
                            10.0,
                            0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::prohibited(),
                        ),
                    ],
                    AvailableOf::definite(S::from_f64(100.0)),
                    writing_mode,
                    direction,
                    align,
                );
                let fragments = batch.final_inline_fragments();
                for (fragment, (used_inline, block_start)) in
                    fragments.iter().zip([(30.0, 0.0), (10.0, 10.0)])
                {
                    let expected = fri06_c02_expected_physical_rect(
                        (writing_mode, direction),
                        (
                            expected_offset(align, decreases, 100.0, used_inline),
                            block_start,
                            used_inline,
                            10.0,
                        ),
                        (100.0, 20.0),
                    );
                    assert_eq!(
                        (
                            fragment.fragment().rect().origin(),
                            fragment.fragment().rect().size()
                        ),
                        expected,
                        "{writing_mode:?} {direction:?} {align:?}"
                    );
                }
            }

            let overflow = fri06_c02_text_batch_with_flow(
                vec![
                    fri06_c02_segment_with_level(
                        3,
                        120.0,
                        0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::mandatory(),
                    ),
                    fri06_c02_segment_with_level(
                        4,
                        10.0,
                        0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    ),
                ],
                AvailableOf::definite(S::from_f64(100.0)),
                writing_mode,
                direction,
                TextAlign::LegacyCenter,
            );
            for (fragment, (inline_start, inline_extent, block_start)) in overflow
                .final_inline_fragments()
                .iter()
                .zip([(0.0, 120.0, 0.0), (45.0, 10.0, 10.0)])
            {
                let expected = fri06_c02_expected_physical_rect(
                    (writing_mode, direction),
                    (inline_start, block_start, inline_extent, 10.0),
                    (100.0, 20.0),
                );
                assert_eq!(
                    (
                        fragment.fragment().rect().origin(),
                        fragment.fragment().rect().size()
                    ),
                    expected,
                    "overflow {writing_mode:?} {direction:?}"
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_flow_projects_rect_baseline_anchor_and_run_extents_in_all_mappings_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in fri06_c02_flow_mappings() {
            let batch = fri06_c02_text_batch_with_flow(
                vec![
                    fri06_c02_segment_with_level(
                        1,
                        10.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    ),
                    fri06_c02_segment_with_level(
                        2,
                        10.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::mandatory(),
                    ),
                    fri06_c02_segment_with_level(
                        3,
                        4.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    ),
                    fri06_c02_segment_with_level(
                        4,
                        6.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    ),
                ],
                AvailableOf::definite(S::from_f64(100.0)),
                writing_mode,
                direction,
                TextAlign::LegacyCenter,
            );
            let fragments = batch.final_inline_fragments();
            assert_eq!(
                fragments
                    .iter()
                    .map(|entry| entry.fragment().visual_index())
                    .collect::<Vec<_>>(),
                vec![1, 0, 1, 0]
            );
            let logical_fragments = [
                (50.0, 10.0, 0.0, 8.0),
                (40.0, 10.0, 0.0, 8.0),
                (51.0, 4.0, 10.0, 18.0),
                (45.0, 6.0, 10.0, 18.0),
            ];
            for (fragment, (inline_start, inline_extent, block_start, baseline_block)) in
                fragments.iter().zip(logical_fragments)
            {
                let expected_rect = fri06_c02_expected_physical_rect(
                    (writing_mode, direction),
                    (inline_start, block_start, inline_extent, 10.0),
                    (100.0, 20.0),
                );
                let expected_baseline = fri06_c02_expected_physical_rect(
                    (writing_mode, direction),
                    (inline_start, baseline_block, 0.0, 0.0),
                    (100.0, 20.0),
                )
                .0;
                assert_eq!(
                    (
                        fragment.fragment().rect().origin(),
                        fragment.fragment().rect().size()
                    ),
                    expected_rect,
                    "rect {writing_mode:?} {direction:?}"
                );
                assert_eq!(
                    fragment.fragment().baseline(),
                    expected_baseline,
                    "baseline {writing_mode:?} {direction:?}"
                );
            }

            let minimum = fragments
                .iter()
                .fold(None, |minimum, entry| {
                    let origin = entry.fragment().rect().origin();
                    Some(minimum.map_or(origin, |current: Point<S>| {
                        Point::new(current.x.min(origin.x), current.y.min(origin.y))
                    }))
                })
                .unwrap();
            let maximum = fragments
                .iter()
                .fold(None, |maximum, entry| {
                    let rect = entry.fragment().rect();
                    let point = Point::new(
                        rect.origin().x + rect.size().width,
                        rect.origin().y + rect.size().height,
                    );
                    Some(maximum.map_or(point, |current: Point<S>| {
                        Point::new(current.x.max(point.x), current.y.max(point.y))
                    }))
                })
                .unwrap();
            let text = fri06_c02_final_node(&batch, 1);
            assert_eq!(text.location, minimum);
            assert_eq!(
                text.size,
                Size::new(maximum.x - minimum.x, maximum.y - minimum.y)
            );
            assert_eq!(
                fri06_c02_final_node(&batch, 0).size,
                FlowAxes::new(writing_mode, direction)
                    .physical_size(LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(20.0)))
            );

            let anchor_batch = fri06_c02_text_batch_with_flow(
                vec![fri06_c02_segment_with_level(
                    9,
                    5.0,
                    0,
                    InlineWhitespaceEdge::DiscardAtBoth,
                    InlineBreakOpportunityOf::mandatory(),
                )],
                AvailableOf::definite(S::from_f64(100.0)),
                writing_mode,
                direction,
                TextAlign::Auto,
            );
            let expected_anchor = fri06_c02_expected_physical_rect(
                (writing_mode, direction),
                (0.0, 0.0, 0.0, 0.0),
                (100.0, 10.0),
            )
            .0;
            let anchor = fri06_c02_final_node(&anchor_batch, 1);
            assert_eq!(anchor.location, expected_anchor);
            assert_eq!(anchor.size, Size::ZERO);
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_block_text_containing_baselines_align_flex_items_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let root = NodeInputOf {
            display: Display::Flex,
            align_items: Some(AlignItems::Baseline),
            ..NodeInputOf::default()
        };
        let item = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let text_one =
            InlineTextInputOf::try_new(vec![fri06_c02_segment_with_metrics(51, 10.0, 8.0, 2.0)])
                .unwrap();
        let text_two =
            InlineTextInputOf::try_new(vec![fri06_c02_segment_with_metrics(52, 10.0, 4.0, 6.0)])
                .unwrap();
        let tree = public_layout_tree(
            HashMap::from([
                (0, LayoutInputOf::box_input(root.clone())),
                (1, LayoutInputOf::box_input(item.clone())),
                (2, LayoutInputOf::box_input(item.clone())),
                (3, LayoutInputOf::inline_text(text_one)),
                (4, LayoutInputOf::inline_text(text_two)),
            ]),
            HashMap::from([
                (0, vec![1, 2]),
                (1, vec![3]),
                (2, vec![4]),
                (3, Vec::new()),
                (4, Vec::new()),
            ]),
        );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(100.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .unwrap();
        let first_item = fri06_c02_final_node(&batch, 1);
        let second_item = fri06_c02_final_node(&batch, 2);
        let first_fragment = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| entry.node() == 3)
            .unwrap()
            .fragment();
        let second_fragment = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| entry.node() == 4)
            .unwrap()
            .fragment();
        assert_eq!(
            first_item.location.y + first_fragment.baseline().y,
            second_item.location.y + second_fragment.baseline().y
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[derive(Clone, Copy, Debug)]
struct LogicalFlexChildFlow {
    writing_mode: WritingMode,
    direction: Direction,
}

fn logical_flex_opposing_flow(flow: LogicalFlexChildFlow) -> LogicalFlexChildFlow {
    LogicalFlexChildFlow {
        writing_mode: match flow.writing_mode {
            WritingMode::HorizontalTb => WritingMode::HorizontalTb,
            WritingMode::VerticalRl => WritingMode::VerticalLr,
            WritingMode::VerticalLr => WritingMode::VerticalRl,
            WritingMode::SidewaysRl => WritingMode::SidewaysLr,
            WritingMode::SidewaysLr => WritingMode::SidewaysRl,
        },
        direction: match flow.writing_mode {
            WritingMode::HorizontalTb => match flow.direction {
                Direction::Ltr => Direction::Rtl,
                Direction::Rtl => Direction::Ltr,
            },
            WritingMode::VerticalRl
            | WritingMode::VerticalLr
            | WritingMode::SidewaysRl
            | WritingMode::SidewaysLr => flow.direction,
        },
    }
}

fn logical_flex_orthogonal_flow(flow: LogicalFlexChildFlow) -> LogicalFlexChildFlow {
    LogicalFlexChildFlow {
        writing_mode: match flow.writing_mode {
            WritingMode::HorizontalTb => WritingMode::VerticalLr,
            WritingMode::VerticalRl
            | WritingMode::VerticalLr
            | WritingMode::SidewaysRl
            | WritingMode::SidewaysLr => WritingMode::HorizontalTb,
        },
        direction: flow.direction,
    }
}

fn logical_flex_all_flow_expected(
    writing_mode: WritingMode,
    direction: Direction,
    flex_direction: FlexDirection,
) -> [(f64, f64); 3] {
    match (writing_mode, direction, flex_direction) {
        (WritingMode::HorizontalTb, Direction::Ltr, FlexDirection::Row)
        | (WritingMode::HorizontalTb, Direction::Rtl, FlexDirection::RowReverse) => {
            [(0.0, 0.0), (10.0, 0.0), (30.0, 0.0)]
        }
        (WritingMode::HorizontalTb, Direction::Ltr, FlexDirection::RowReverse)
        | (WritingMode::HorizontalTb, Direction::Rtl, FlexDirection::Row) => {
            [(90.0, 0.0), (70.0, 0.0), (40.0, 0.0)]
        }
        (WritingMode::HorizontalTb, Direction::Ltr, FlexDirection::Column) => {
            [(0.0, 0.0), (0.0, 10.0), (0.0, 30.0)]
        }
        (WritingMode::HorizontalTb, Direction::Ltr, FlexDirection::ColumnReverse) => {
            [(0.0, 90.0), (0.0, 70.0), (0.0, 40.0)]
        }
        (WritingMode::HorizontalTb, Direction::Rtl, FlexDirection::Column) => {
            [(90.0, 0.0), (80.0, 10.0), (70.0, 30.0)]
        }
        (WritingMode::HorizontalTb, Direction::Rtl, FlexDirection::ColumnReverse) => {
            [(90.0, 90.0), (80.0, 70.0), (70.0, 40.0)]
        }
        (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Ltr, FlexDirection::Row)
        | (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::RowReverse,
        ) => [(90.0, 0.0), (80.0, 10.0), (70.0, 30.0)],
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::RowReverse,
        )
        | (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Rtl, FlexDirection::Row) => {
            [(90.0, 90.0), (80.0, 70.0), (70.0, 40.0)]
        }
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::Column,
        ) => [(90.0, 0.0), (70.0, 0.0), (40.0, 0.0)],
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
        ) => [(0.0, 0.0), (10.0, 0.0), (30.0, 0.0)],
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::Column,
        ) => [(90.0, 90.0), (70.0, 80.0), (40.0, 70.0)],
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
        ) => [(0.0, 90.0), (10.0, 80.0), (30.0, 70.0)],
        (WritingMode::VerticalLr, Direction::Ltr, FlexDirection::Row)
        | (WritingMode::VerticalLr, Direction::Rtl, FlexDirection::RowReverse)
        | (WritingMode::SidewaysLr, Direction::Rtl, FlexDirection::Row)
        | (WritingMode::SidewaysLr, Direction::Ltr, FlexDirection::RowReverse) => {
            [(0.0, 0.0), (0.0, 10.0), (0.0, 30.0)]
        }
        (WritingMode::VerticalLr, Direction::Ltr, FlexDirection::RowReverse)
        | (WritingMode::VerticalLr, Direction::Rtl, FlexDirection::Row)
        | (WritingMode::SidewaysLr, Direction::Rtl, FlexDirection::RowReverse)
        | (WritingMode::SidewaysLr, Direction::Ltr, FlexDirection::Row) => {
            [(0.0, 90.0), (0.0, 70.0), (0.0, 40.0)]
        }
        (WritingMode::VerticalLr, Direction::Ltr, FlexDirection::Column)
        | (WritingMode::SidewaysLr, Direction::Rtl, FlexDirection::Column) => {
            [(0.0, 0.0), (10.0, 0.0), (30.0, 0.0)]
        }
        (WritingMode::VerticalLr, Direction::Ltr, FlexDirection::ColumnReverse)
        | (WritingMode::SidewaysLr, Direction::Rtl, FlexDirection::ColumnReverse) => {
            [(90.0, 0.0), (70.0, 0.0), (40.0, 0.0)]
        }
        (WritingMode::VerticalLr, Direction::Rtl, FlexDirection::Column)
        | (WritingMode::SidewaysLr, Direction::Ltr, FlexDirection::Column) => {
            [(0.0, 90.0), (10.0, 80.0), (30.0, 70.0)]
        }
        (WritingMode::VerticalLr, Direction::Rtl, FlexDirection::ColumnReverse)
        | (WritingMode::SidewaysLr, Direction::Ltr, FlexDirection::ColumnReverse) => {
            [(90.0, 90.0), (70.0, 80.0), (40.0, 70.0)]
        }
    }
}

fn assert_logical_flex_placement_vertical_lr_row_projects_inline_main<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.0, 20.0))
        .with_style(2, logical_flex_leaf(10.0, 20.0));
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 1).location,
        Point::new(scalar(0.0), scalar(0.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).location,
        Point::new(scalar(0.0), scalar(20.0))
    );
}

#[test]
fn logical_flex_placement_vertical_lr_row_projects_inline_main_for_f32() {
    assert_logical_flex_placement_vertical_lr_row_projects_inline_main::<f32>();
}

#[test]
fn logical_flex_placement_vertical_lr_row_projects_inline_main_for_f64() {
    assert_logical_flex_placement_vertical_lr_row_projects_inline_main::<f64>();
}

fn assert_logical_flex_boundaries_reverse_and_wrap_reverse_project_once<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let reversed = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::RowReverse,
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.0, 20.0))
        .with_style(2, logical_flex_leaf(10.0, 20.0));
    let reversed_batch = compute_layout(
        &reversed,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("reversed non-leaf flex root layout succeeds");
    assert_eq!(
        public_flow_output(reversed_batch.final_entries(), 1).location,
        Point::new(scalar(0.0), scalar(80.0))
    );
    assert_eq!(
        public_flow_output(reversed_batch.final_entries(), 2).location,
        Point::new(scalar(0.0), scalar(60.0))
    );

    let wrapped = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::WrapReverse,
                align_content: Some(AlignContent::FlexStart),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.0, 60.0))
        .with_style(2, logical_flex_leaf(10.0, 60.0));
    let wrapped_batch = compute_layout(
        &wrapped,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("wrapped non-leaf flex root layout succeeds");
    assert_eq!(
        public_flow_output(wrapped_batch.final_entries(), 1).location,
        Point::new(scalar(90.0), scalar(0.0))
    );
    assert_eq!(
        public_flow_output(wrapped_batch.final_entries(), 2).location,
        Point::new(scalar(80.0), scalar(0.0))
    );
}

#[test]
fn logical_flex_boundaries_reverse_and_wrap_reverse_project_once_for_f32() {
    assert_logical_flex_boundaries_reverse_and_wrap_reverse_project_once::<f32>();
}

#[test]
fn logical_flex_boundaries_reverse_and_wrap_reverse_project_once_for_f64() {
    assert_logical_flex_boundaries_reverse_and_wrap_reverse_project_once::<f64>();
}

fn assert_logical_flex_placement_wrap_reverse_keeps_logical_and_flex_alignment_distinct<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2, 3, 4])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_children(4, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::WrapReverse,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                align_self: Some(AlignItems::Start),
                ..logical_flex_leaf(10.0, 10.0)
            },
        )
        .with_style(
            2,
            NodeInputOf {
                align_self: Some(AlignItems::FlexStart),
                ..logical_flex_leaf(10.0, 10.0)
            },
        )
        .with_style(
            3,
            NodeInputOf {
                align_self: Some(AlignItems::End),
                ..logical_flex_leaf(10.0, 10.0)
            },
        )
        .with_style(
            4,
            NodeInputOf {
                align_self: Some(AlignItems::FlexEnd),
                ..logical_flex_leaf(10.0, 10.0)
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("wrap-reverse logical and flex alignment succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 1).location,
        Point::new(scalar(0.0), scalar(0.0)),
        "logical start remains tied to the container flow start"
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).location,
        Point::new(scalar(10.0), scalar(90.0)),
        "flex start follows the wrap-reversed cross axis"
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 3).location,
        Point::new(scalar(20.0), scalar(90.0)),
        "logical end remains tied to the container flow end"
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 4).location,
        Point::new(scalar(30.0), scalar(0.0)),
        "flex end follows the wrap-reversed cross axis"
    );
}

#[test]
fn logical_flex_placement_wrap_reverse_distinguishes_logical_and_flex_alignment_for_f32() {
    assert_logical_flex_placement_wrap_reverse_keeps_logical_and_flex_alignment_distinct::<f32>();
}

#[test]
fn logical_flex_placement_wrap_reverse_distinguishes_logical_and_flex_alignment_for_f64() {
    assert_logical_flex_placement_wrap_reverse_keeps_logical_and_flex_alignment_distinct::<f64>();
}

fn assert_logical_flex_placement_maps_auto_margins_and_relative_trailing_inset<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                position: Position::Relative,
                margin: Edges {
                    top: LengthAutoOf::AUTO,
                    left: LengthAutoOf::AUTO,
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                inset: Edges {
                    bottom: LengthAutoOf::px(scalar(5.0)),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                ..logical_flex_leaf(10.0, 20.0)
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("logical auto-margin layout succeeds");
    let output = public_flow_output(batch.final_entries(), 1);
    assert_eq!(output.margin.top, scalar(80.0));
    assert_eq!(output.margin.left, scalar(90.0));
    assert_eq!(output.location, Point::new(scalar(90.0), scalar(75.0)));
}

#[test]
fn logical_flex_placement_maps_auto_margins_and_relative_trailing_inset_for_f32() {
    assert_logical_flex_placement_maps_auto_margins_and_relative_trailing_inset::<f32>();
}

#[test]
fn logical_flex_placement_maps_auto_margins_and_relative_trailing_inset_for_f64() {
    assert_logical_flex_placement_maps_auto_margins_and_relative_trailing_inset::<f64>();
}

fn assert_logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    struct Case {
        name: &'static str,
        writing_mode: WritingMode,
        direction: Direction,
        flex_direction: FlexDirection,
        flex_wrap: FlexWrap,
        relative_location: Point<f64>,
        absolute_location: Point<f64>,
    }

    for case in [
        Case {
            name: "horizontal LTR row reverse",
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            flex_direction: FlexDirection::RowReverse,
            flex_wrap: FlexWrap::NoWrap,
            relative_location: Point::new(100.0, 20.0),
            absolute_location: Point::new(10.0, 20.0),
        },
        Case {
            name: "horizontal LTR row wrap reverse",
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::WrapReverse,
            relative_location: Point::new(10.0, 110.0),
            absolute_location: Point::new(10.0, 20.0),
        },
        Case {
            name: "vertical RL RTL row reverse",
            writing_mode: WritingMode::VerticalRl,
            direction: Direction::Rtl,
            flex_direction: FlexDirection::RowReverse,
            flex_wrap: FlexWrap::NoWrap,
            relative_location: Point::new(60.0, -40.0),
            absolute_location: Point::new(60.0, 50.0),
        },
        Case {
            name: "sideways LR RTL row wrap reverse",
            writing_mode: WritingMode::SidewaysLr,
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::WrapReverse,
            relative_location: Point::new(100.0, 20.0),
            absolute_location: Point::new(10.0, 20.0),
        },
        Case {
            name: "horizontal RTL column reverse",
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Rtl,
            flex_direction: FlexDirection::ColumnReverse,
            flex_wrap: FlexWrap::NoWrap,
            relative_location: Point::new(60.0, 110.0),
            absolute_location: Point::new(60.0, 20.0),
        },
    ] {
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                    writing_mode: case.writing_mode,
                    direction: case.direction,
                    flex_direction: case.flex_direction,
                    flex_wrap: case.flex_wrap,
                    align_content: Some(AlignContent::FlexStart),
                    align_items: Some(AlignItems::FlexStart),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    position: Position::Relative,
                    inset: Edges {
                        top: LengthAutoOf::px(scalar(20.0)),
                        right: LengthAutoOf::px(scalar(30.0)),
                        bottom: LengthAutoOf::px(scalar(40.0)),
                        left: LengthAutoOf::px(scalar(10.0)),
                    },
                    ..logical_flex_leaf(10.0, 10.0)
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    position: Position::Absolute,
                    inset: Edges {
                        top: LengthAutoOf::px(scalar(20.0)),
                        right: LengthAutoOf::px(scalar(30.0)),
                        bottom: LengthAutoOf::px(scalar(40.0)),
                        left: LengthAutoOf::px(scalar(10.0)),
                    },
                    ..logical_flex_leaf(10.0, 10.0)
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("valid viewport request"),
        )
        .expect("positioned inset precedence layout succeeds");

        assert_eq!(
            public_flow_output(batch.final_entries(), 1).location,
            Point::new(
                scalar(case.relative_location.x),
                scalar(case.relative_location.y),
            ),
            "{} relative positioning keeps normal-flow authored-edge precedence",
            case.name
        );
        assert_eq!(
            public_flow_output(batch.final_entries(), 2).location,
            Point::new(
                scalar(case.absolute_location.x),
                scalar(case.absolute_location.y),
            ),
            "{} absolute positioning keeps normal-flow authored-edge precedence",
            case.name
        );
    }
}

#[test]
fn logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence_for_f32() {
    assert_logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence::<f32>();
}

#[test]
fn logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence_for_f64() {
    assert_logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence::<f64>();
}

fn assert_logical_flex_boundaries_absolute_static_alignment_and_all_flows<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let absolute = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                position: Position::Absolute,
                align_self: Some(AlignItems::FlexEnd),
                ..logical_flex_leaf(10.0, 20.0)
            },
        );
    let batch = compute_layout(
        &absolute,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("logical absolute static alignment succeeds");
    assert_eq!(
        public_flow_output(batch.final_entries(), 1).location,
        Point::new(scalar(90.0), scalar(0.0))
    );

    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for flex_direction in [
                FlexDirection::Row,
                FlexDirection::RowReverse,
                FlexDirection::Column,
                FlexDirection::ColumnReverse,
            ] {
                let parallel_flow = LogicalFlexChildFlow {
                    writing_mode,
                    direction,
                };
                let opposing_flow = logical_flex_opposing_flow(parallel_flow);
                let orthogonal_flow = logical_flex_orthogonal_flow(parallel_flow);
                let tree = PublicFlowTree::default()
                    .with_children(0, [1, 2, 3])
                    .with_children(1, [4])
                    .with_children(2, [5])
                    .with_children(3, [6])
                    .with_children(4, [])
                    .with_children(5, [])
                    .with_children(6, [])
                    .with_style(
                        0,
                        NodeInputOf {
                            display: Display::Flex,
                            writing_mode,
                            direction,
                            size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                            flex_direction,
                            justify_content: Some(AlignContent::FlexStart),
                            align_items: Some(AlignItems::Start),
                            ..NodeInputOf::default()
                        },
                    )
                    .with_style(
                        1,
                        NodeInputOf {
                            writing_mode: parallel_flow.writing_mode,
                            direction: parallel_flow.direction,
                            ..logical_flex_leaf(10.0, 10.0)
                        },
                    )
                    .with_style(
                        2,
                        NodeInputOf {
                            writing_mode: opposing_flow.writing_mode,
                            direction: opposing_flow.direction,
                            ..logical_flex_leaf(20.0, 20.0)
                        },
                    )
                    .with_style(
                        3,
                        NodeInputOf {
                            writing_mode: orthogonal_flow.writing_mode,
                            direction: orthogonal_flow.direction,
                            ..logical_flex_leaf(30.0, 30.0)
                        },
                    )
                    .with_style(4, logical_flex_leaf(4.0, 5.0))
                    .with_style(5, logical_flex_leaf(6.0, 7.0))
                    .with_style(6, logical_flex_leaf(8.0, 9.0));
                let batch = compute_layout(
                    &tree,
                    0,
                    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(
                        100.0,
                    ))))
                    .expect("valid viewport request"),
                )
                .expect("all logical flex directions complete without fallback");
                assert_eq!(batch.final_entries().len(), 7);
                for (node, (x, y)) in [1_u32, 2, 3]
                    .into_iter()
                    .zip(logical_flex_all_flow_expected(
                        writing_mode,
                        direction,
                        flex_direction,
                    ))
                {
                    assert_eq!(
                        public_flow_output(batch.final_entries(), node).location,
                        Point::new(scalar(x), scalar(y)),
                        "{writing_mode:?} {direction:?} {flex_direction:?} must project child {node} through its physical axis and progression"
                    );
                }
                for (node, child_flow) in [
                    (4_u32, parallel_flow),
                    (5, opposing_flow),
                    (6, orthogonal_flow),
                ] {
                    let (descendant_x, descendant_y) =
                        logical_flex_descendant_expected(node, child_flow);
                    assert_eq!(
                        public_flow_output(batch.final_entries(), node).location,
                        Point::new(scalar(descendant_x), scalar(descendant_y)),
                        "{writing_mode:?} {direction:?} {flex_direction:?} must retain {child_flow:?} for descendant {node}"
                    );
                }
            }
        }
    }
}

fn logical_flex_descendant_expected(node: u32, child_flow: LogicalFlexChildFlow) -> (f64, f64) {
    match (child_flow.writing_mode, child_flow.direction) {
        (WritingMode::HorizontalTb, Direction::Ltr) => (0.0, 0.0),
        (WritingMode::HorizontalTb, Direction::Rtl) => match node {
            4 => (6.0, 0.0),
            5 => (14.0, 0.0),
            6 => (22.0, 0.0),
            _ => unreachable!("all-flow descendant fixture has nodes 4 through 6"),
        },
        (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Ltr) => match node {
            4 => (6.0, 0.0),
            5 => (14.0, 0.0),
            6 => (22.0, 0.0),
            _ => unreachable!("all-flow descendant fixture has nodes 4 through 6"),
        },
        (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Rtl) => match node {
            4 => (6.0, 5.0),
            5 => (14.0, 13.0),
            6 => (22.0, 21.0),
            _ => unreachable!("all-flow descendant fixture has nodes 4 through 6"),
        },
        (WritingMode::VerticalLr, Direction::Ltr) => (0.0, 0.0),
        (WritingMode::VerticalLr, Direction::Rtl) | (WritingMode::SidewaysLr, Direction::Ltr) => {
            match node {
                4 => (0.0, 5.0),
                5 => (0.0, 13.0),
                6 => (0.0, 21.0),
                _ => unreachable!("all-flow descendant fixture has nodes 4 through 6"),
            }
        }
        (WritingMode::SidewaysLr, Direction::Rtl) => (0.0, 0.0),
    }
}

fn assert_logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    for (flex_direction, start, flex_start, end, flex_end) in [
        (
            FlexDirection::RowReverse,
            (0.0, 0.0),
            (90.0, 0.0),
            (90.0, 0.0),
            (0.0, 0.0),
        ),
        (
            FlexDirection::ColumnReverse,
            (0.0, 0.0),
            (0.0, 90.0),
            (0.0, 90.0),
            (0.0, 0.0),
        ),
    ] {
        for (alignment, expected) in [
            (AlignContent::Start, start),
            (AlignContent::FlexStart, flex_start),
            (AlignContent::End, end),
            (AlignContent::FlexEnd, flex_end),
        ] {
            let tree = PublicFlowTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Flex,
                        size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                        flex_direction,
                        justify_content: Some(alignment),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(1, logical_flex_leaf(10.0, 10.0));
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                    .expect("valid viewport request"),
            )
            .expect("reversed main alignment layout succeeds");
            assert_eq!(
                public_flow_output(batch.final_entries(), 1).location,
                Point::new(scalar(expected.0), scalar(expected.1)),
                "{flex_direction:?} {alignment:?} keeps logical and flex-relative main alignment distinct"
            );
        }
    }
}

fn assert_logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    for (alignment, expected_y) in [
        (AlignContent::Start, 10.0),
        (AlignContent::FlexStart, 90.0),
        (AlignContent::End, 90.0),
        (AlignContent::FlexEnd, 10.0),
    ] {
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                    flex_wrap: FlexWrap::WrapReverse,
                    align_content: Some(alignment),
                    ..NodeInputOf::default()
                },
            )
            .with_style(1, logical_flex_leaf(60.0, 10.0))
            .with_style(2, logical_flex_leaf(60.0, 10.0));
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("valid viewport request"),
        )
        .expect("wrap-reversed line alignment layout succeeds");
        assert_eq!(
            public_flow_output(batch.final_entries(), 1).location,
            Point::new(scalar(0.0), scalar(expected_y)),
            "wrap-reverse {alignment:?} keeps logical and flex-relative line alignment distinct"
        );
    }
}

fn assert_logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    for (flex_direction, start, flex_start, end, flex_end) in [
        (
            FlexDirection::RowReverse,
            (0.0, 0.0),
            (90.0, 0.0),
            (90.0, 0.0),
            (0.0, 0.0),
        ),
        (
            FlexDirection::ColumnReverse,
            (0.0, 0.0),
            (0.0, 90.0),
            (0.0, 90.0),
            (0.0, 0.0),
        ),
    ] {
        for (alignment, expected) in [
            (AlignContent::Start, start),
            (AlignContent::FlexStart, flex_start),
            (AlignContent::End, end),
            (AlignContent::FlexEnd, flex_end),
        ] {
            let tree = PublicFlowTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Flex,
                        size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                        flex_direction,
                        justify_content: Some(alignment),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        position: Position::Absolute,
                        ..logical_flex_leaf(10.0, 10.0)
                    },
                );
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                    .expect("valid viewport request"),
            )
            .expect("absolute reversed main alignment layout succeeds");
            assert_eq!(
                public_flow_output(batch.final_entries(), 1).location,
                Point::new(scalar(expected.0), scalar(expected.1)),
                "absolute {flex_direction:?} {alignment:?} keeps logical and flex-relative main alignment distinct"
            );
        }
    }
}

fn assert_logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2, 3, 4])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_children(4, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::WrapReverse,
                ..NodeInputOf::default()
            },
        );
    let tree = [
        (1, AlignItems::Start),
        (2, AlignItems::FlexStart),
        (3, AlignItems::End),
        (4, AlignItems::FlexEnd),
    ]
    .into_iter()
    .fold(tree, |tree, (node, align_self)| {
        tree.with_style(
            node,
            NodeInputOf {
                position: Position::Absolute,
                align_self: Some(align_self),
                ..logical_flex_leaf(10.0, 10.0)
            },
        )
    });
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("absolute wrap-reverse logical and flex alignment succeeds");

    for (node, expected_y) in [(1, 0.0), (2, 90.0), (3, 90.0), (4, 0.0)] {
        assert_eq!(
            public_flow_output(batch.final_entries(), node).location,
            Point::new(scalar(0.0), scalar(expected_y)),
            "absolute item {node} keeps its logical or flex-relative cross alignment"
        );
    }
}

#[test]
fn logical_flex_boundaries_absolute_static_alignment_and_all_flows_for_f32() {
    assert_logical_flex_boundaries_absolute_static_alignment_and_all_flows::<f32>();
}

#[test]
fn logical_flex_boundaries_absolute_static_alignment_and_all_flows_for_f64() {
    assert_logical_flex_boundaries_absolute_static_alignment_and_all_flows::<f64>();
}

#[test]
fn logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords_for_f32() {
    assert_logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords::<f32>(
    );
}

#[test]
fn logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords_for_f64() {
    assert_logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords::<f64>(
    );
}

#[test]
fn logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords_for_f32()
 {
    assert_logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords::<f32>();
}

#[test]
fn logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords_for_f64()
 {
    assert_logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords::<f64>();
}

#[test]
fn logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords_for_f32()
 {
    assert_logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords::<f32>();
}

#[test]
fn logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords_for_f64()
 {
    assert_logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords::<f64>();
}

#[test]
fn logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment_for_f32()
{
    assert_logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment::<
        f32,
    >();
}

#[test]
fn logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment_for_f64()
{
    assert_logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment::<
        f64,
    >();
}

fn assert_logical_flex_sizing_vertical_lr_row_uses_container_inline_axis<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
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
                writing_mode: WritingMode::SidewaysLr,
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
            .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 1).size,
        Size::new(scalar(10.0), scalar(20.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).size,
        Size::new(scalar(10.0), scalar(20.0))
    );
}

#[test]
fn logical_flex_sizing_vertical_lr_row_uses_container_inline_axis_for_f32() {
    assert_logical_flex_sizing_vertical_lr_row_uses_container_inline_axis::<f32>();
}

#[test]
fn logical_flex_sizing_vertical_lr_row_uses_container_inline_axis_for_f64() {
    assert_logical_flex_sizing_vertical_lr_row_uses_container_inline_axis::<f64>();
}

fn assert_logical_ordinary_grid_container_sizing<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_outer_size = crate::geometry::LogicalSizeOf::new(scalar(70.0), scalar(110.0));
    let logical_style_size = crate::geometry::LogicalSizeOf::new(scalar(80.0), scalar(120.0));
    let logical_min_size = crate::geometry::LogicalSizeOf::new(scalar(60.0), scalar(100.0));
    let logical_gap = crate::geometry::LogicalSizeOf::new(
        LengthOf::percent(scalar(0.1)),
        LengthOf::percent(scalar(0.2)),
    );

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    direction,
                    size: flow_axes
                        .physical_size(logical_style_size)
                        .map(PreferredSizeOf::px),
                    min_size: flow_axes.physical_size(logical_min_size).map(MinSizeOf::px),
                    max_size: flow_axes
                        .physical_size(logical_outer_size)
                        .map(MaxSizeOf::px),
                    gap: flow_axes.physical_size(logical_gap),
                    grid_template_columns: vec![TrackComponentOf::px(scalar(30.0))],
                    grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                    grid_auto_columns: vec![TrackComponentOf::px(scalar(33.0))],
                    grid_auto_rows: vec![TrackComponentOf::px(scalar(48.0))],
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                    grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("ordinary grid root layout succeeds");
        let expected = flow_axes.physical_size(logical_outer_size);
        let output = public_flow_output(batch.unrounded_entries(), 0);

        assert_eq!(output.size, expected, "{writing_mode:?} {direction:?}");
        assert_eq!(
            output.content_size, expected,
            "{writing_mode:?} {direction:?}"
        );
    }
}

#[test]
fn logical_ordinary_grid_container_sizing_f32() {
    assert_logical_ordinary_grid_container_sizing::<f32>();
}

#[test]
fn logical_ordinary_grid_container_sizing_f64() {
    assert_logical_ordinary_grid_container_sizing::<f64>();
}

fn assert_logical_ordinary_grid_intrinsic_reruns_public_leaves<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let physical_leaf_size = Size::new(
        PreferredSizeOf::px(scalar(17.0)),
        PreferredSizeOf::px(scalar(31.0)),
    );
    let expected_size = Size::new(scalar(17.0), scalar(31.0));
    let relationships = [
        (
            WritingMode::HorizontalTb,
            Direction::Ltr,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "parallel",
        ),
        (
            WritingMode::HorizontalTb,
            Direction::Rtl,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "opposing",
        ),
        (
            WritingMode::HorizontalTb,
            Direction::Ltr,
            WritingMode::VerticalRl,
            Direction::Ltr,
            "parent-horizontal-child-vertical",
        ),
        (
            WritingMode::SidewaysLr,
            Direction::Rtl,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "parent-sideways-child-horizontal",
        ),
    ];

    for (parent_writing_mode, parent_direction, child_writing_mode, child_direction, label) in
        relationships
    {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::InlineGrid,
                    writing_mode: parent_writing_mode,
                    direction: parent_direction,
                    grid_template_columns: vec![TrackComponentOf::AUTO],
                    grid_template_rows: vec![TrackComponentOf::AUTO],
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: child_writing_mode,
                    direction: child_direction,
                    size: physical_leaf_size.clone(),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("ordinary grid public leaf layout succeeds");

        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 0).size,
            expected_size,
            "{label}"
        );
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 1).size,
            expected_size,
            "{label}"
        );
    }
}

#[test]
fn logical_ordinary_grid_intrinsic_reruns_public_leaves_f32() {
    assert_logical_ordinary_grid_intrinsic_reruns_public_leaves::<f32>();
}

#[test]
fn logical_ordinary_grid_intrinsic_reruns_public_leaves_f64() {
    assert_logical_ordinary_grid_intrinsic_reruns_public_leaves::<f64>();
}

#[derive(Clone, Copy, Debug)]
struct LogicalGridChildFlow {
    writing_mode: WritingMode,
    direction: Direction,
}

fn logical_grid_opposing_flow(flow: LogicalGridChildFlow) -> LogicalGridChildFlow {
    LogicalGridChildFlow {
        writing_mode: match flow.writing_mode {
            WritingMode::HorizontalTb => WritingMode::HorizontalTb,
            WritingMode::VerticalRl => WritingMode::VerticalLr,
            WritingMode::VerticalLr => WritingMode::VerticalRl,
            WritingMode::SidewaysRl => WritingMode::SidewaysLr,
            WritingMode::SidewaysLr => WritingMode::SidewaysRl,
        },
        direction: match flow.writing_mode {
            WritingMode::HorizontalTb => match flow.direction {
                Direction::Ltr => Direction::Rtl,
                Direction::Rtl => Direction::Ltr,
            },
            WritingMode::VerticalRl
            | WritingMode::VerticalLr
            | WritingMode::SidewaysRl
            | WritingMode::SidewaysLr => flow.direction,
        },
    }
}

fn logical_grid_orthogonal_flow(flow: LogicalGridChildFlow) -> LogicalGridChildFlow {
    LogicalGridChildFlow {
        writing_mode: match flow.writing_mode {
            WritingMode::HorizontalTb => WritingMode::VerticalLr,
            WritingMode::VerticalRl
            | WritingMode::VerticalLr
            | WritingMode::SidewaysRl
            | WritingMode::SidewaysLr => WritingMode::HorizontalTb,
        },
        direction: flow.direction,
    }
}

fn nearest_css_pixel<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

fn assert_logical_ordinary_grid_absolute_static<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_container_size = crate::geometry::LogicalSizeOf::new(scalar(70.5), scalar(110.25));
    let logical_child_size = crate::geometry::LogicalSizeOf::new(scalar(11.25), scalar(13.5));
    let explicit_margin =
        crate::geometry::LogicalEdgesOf::new(scalar(1.25), scalar(2.5), scalar(3.75), scalar(4.25));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical_container_size = flow_axes.physical_size(logical_container_size);
        let physical_child_size = flow_axes.physical_size(logical_child_size);
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2, 3, 4])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_children(4, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    direction,
                    size: physical_container_size.map(PreferredSizeOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(scalar(30.25)),
                        TrackComponentOf::px(scalar(40.25)),
                    ],
                    grid_template_rows: vec![
                        TrackComponentOf::px(scalar(50.25)),
                        TrackComponentOf::px(scalar(60.0)),
                    ],
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(PreferredSizeOf::px),
                    position: Position::Absolute,
                    grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                    grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                    margin: flow_axes.physical_edges(explicit_margin.map(LengthAutoOf::px)),
                    inset: flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                        LengthAutoOf::px(scalar(2.25)),
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::px(scalar(3.5)),
                    )),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(PreferredSizeOf::px),
                    position: Position::Absolute,
                    grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                    grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                    margin: flow_axes.physical_edges(explicit_margin.map(LengthAutoOf::px)),
                    justify_self: Some(AlignItems::End),
                    align_self: Some(AlignItems::Center),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(PreferredSizeOf::px),
                    position: Position::Absolute,
                    grid_row: GridPlacement::try_line(2).expect("valid grid row"),
                    margin: flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                    )),
                    justify_self: Some(AlignItems::End),
                    align_self: Some(AlignItems::End),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                4,
                NodeInputOf {
                    display: Display::None,
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("logical ordinary-grid absolute layout succeeds");

        let explicit_inset_location = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(scalar(33.75), scalar(89.0)),
            logical_child_size,
            physical_container_size,
        );
        let aligned_inline = scalar(40.25) - logical_child_size.inline - explicit_margin.inline_end;
        let aligned_block = (scalar(60.0) - logical_child_size.block + explicit_margin.block_start
            - explicit_margin.block_end)
            / scalar(2.0);
        let aligned_location = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(
                scalar(30.25) + aligned_inline,
                scalar(50.25) + aligned_block,
            ),
            logical_child_size,
            physical_container_size,
        );
        let static_location = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(
                (logical_container_size.inline - logical_child_size.inline) / scalar(2.0),
                scalar(50.25) + (scalar(60.0) - logical_child_size.block) / scalar(2.0),
            ),
            logical_child_size,
            physical_container_size,
        );

        for (node, expected_location) in [
            (1, explicit_inset_location),
            (2, aligned_location),
            (3, static_location),
        ] {
            let unrounded = public_flow_output(batch.unrounded_entries(), node);
            let rounded = public_flow_output(batch.final_entries(), node);
            assert_eq!(
                unrounded.location, expected_location,
                "{writing_mode:?} {direction:?} absolute child {node} must project its logical area once"
            );
            assert_eq!(unrounded.size, physical_child_size);
            assert_eq!(
                rounded.location,
                Point::new(
                    nearest_css_pixel(unrounded.location.x),
                    nearest_css_pixel(unrounded.location.y),
                )
            );
        }
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 4),
            NodeOutputOf::with_source_index(crate::SourceIndex::new(3))
        );
    }
}

#[test]
fn logical_ordinary_grid_absolute_static_f32() {
    assert_logical_ordinary_grid_absolute_static::<f32>();
}

#[test]
fn logical_ordinary_grid_absolute_static_f64() {
    assert_logical_ordinary_grid_absolute_static::<f64>();
}

fn grid_lanes_absolute_expected_location<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    node: u32,
) -> Point<S> {
    let scalar = scalar::<S>;
    let logical_origin = match node {
        1 => crate::geometry::LogicalPointOf::new(scalar(39.25), scalar(96.75)),
        2 => crate::geometry::LogicalPointOf::new(scalar(62.25), scalar(81.0)),
        3 => crate::geometry::LogicalPointOf::new(scalar(32.375), scalar(81.25)),
        _ => unreachable!("grid-lanes fixture has nodes 1 through 3"),
    };
    let flow_axes = FlowAxes::new(writing_mode, direction);
    flow_axes.physical_point(
        logical_origin,
        crate::geometry::LogicalSizeOf::new(scalar(11.25), scalar(13.5)),
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            scalar(76.0),
            scalar(118.0),
        )),
    )
}

fn grid_lanes_nearest_css_pixel<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

fn assert_logical_grid_lanes_absolute_static<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_container_size = crate::geometry::LogicalSizeOf::new(scalar(76.0), scalar(118.0));
    let logical_child_size = crate::geometry::LogicalSizeOf::new(scalar(11.25), scalar(13.5));
    let explicit_margin =
        crate::geometry::LogicalEdgesOf::new(scalar(1.25), scalar(2.5), scalar(3.75), scalar(4.25));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical_container_size = flow_axes.physical_size(logical_container_size);
        let physical_child_size = flow_axes.physical_size(logical_child_size);
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::GridLanes,
                    writing_mode,
                    direction,
                    size: physical_container_size.map(PreferredSizeOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(scalar(30.25)),
                        TrackComponentOf::px(scalar(40.25)),
                    ],
                    grid_template_rows: vec![
                        TrackComponentOf::px(scalar(50.25)),
                        TrackComponentOf::px(scalar(60.0)),
                    ],
                    gap: flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
                        LengthOf::px(scalar(5.5)),
                        LengthOf::px(scalar(7.75)),
                    )),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(PreferredSizeOf::px),
                    position: Position::Absolute,
                    grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                    grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                    margin: flow_axes.physical_edges(explicit_margin.map(LengthAutoOf::px)),
                    inset: flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                        LengthAutoOf::px(scalar(2.25)),
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::px(scalar(3.5)),
                    )),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(PreferredSizeOf::px),
                    position: Position::Absolute,
                    grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                    grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                    margin: flow_axes.physical_edges(explicit_margin.map(LengthAutoOf::px)),
                    justify_self: Some(AlignItems::End),
                    align_self: Some(AlignItems::Center),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(PreferredSizeOf::px),
                    position: Position::Absolute,
                    grid_row: GridPlacement::try_line(2).expect("valid grid row"),
                    margin: flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                    )),
                    justify_self: Some(AlignItems::End),
                    align_self: Some(AlignItems::End),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("grid-lanes absolute layout succeeds");

        for node in [1, 2, 3] {
            let expected_location =
                grid_lanes_absolute_expected_location(writing_mode, direction, node);
            let unrounded = public_flow_output(batch.unrounded_entries(), node);
            let rounded = public_flow_output(batch.final_entries(), node);
            assert_eq!(
                unrounded.location, expected_location,
                "{writing_mode:?} {direction:?} grid-lanes absolute child {node} must preserve its C07 projection"
            );
            assert_eq!(unrounded.size, physical_child_size);
            assert_eq!(
                rounded.location,
                Point::new(
                    grid_lanes_nearest_css_pixel(unrounded.location.x),
                    grid_lanes_nearest_css_pixel(unrounded.location.y),
                )
            );
            assert_eq!(
                rounded.size,
                Size::new(
                    grid_lanes_nearest_css_pixel(unrounded.location.x + unrounded.size.width)
                        - rounded.location.x,
                    grid_lanes_nearest_css_pixel(unrounded.location.y + unrounded.size.height)
                        - rounded.location.y,
                )
            );
        }
    }
}

#[test]
fn logical_grid_lanes_absolute_static_f32() {
    assert_logical_grid_lanes_absolute_static::<f32>();
}

#[test]
fn logical_grid_lanes_absolute_static_f64() {
    assert_logical_grid_lanes_absolute_static::<f64>();
}

fn logical_axis_value<S: LayoutScalar>(
    size: crate::geometry::LogicalSizeOf<S>,
    axis: LogicalAxis,
) -> S {
    match axis {
        LogicalAxis::Inline => size.inline,
        LogicalAxis::Block => size.block,
    }
}

fn logical_axis_start<S: LayoutScalar>(
    edges: crate::geometry::LogicalEdgesOf<S>,
    axis: LogicalAxis,
) -> S {
    match axis {
        LogicalAxis::Inline => edges.inline_start,
        LogicalAxis::Block => edges.block_start,
    }
}

fn logical_axis_margin_sum<S: LayoutScalar>(
    edges: crate::geometry::LogicalEdgesOf<S>,
    axis: LogicalAxis,
) -> S {
    match axis {
        LogicalAxis::Inline => edges.inline_sum(),
        LogicalAxis::Block => edges.block_sum(),
    }
}

#[test]
fn orthogonal_grid_lanes_selected_rows_use_column_lane_offsets() {
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::GridLanes,
                size: Size::new(
                    PreferredSizeOf::px(30.0 + 40.0),
                    PreferredSizeOf::px(50.0 + 60.0),
                ),
                grid_auto_flow: GridAutoFlow::Column,
                grid_template_columns: vec![TrackComponentOf::px(30.0), TrackComponentOf::px(40.0)],
                grid_template_rows: vec![TrackComponentOf::px(50.0), TrackComponentOf::px(60.0)],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSizeOf::px(30.0), PreferredSizeOf::px(50.0)),
                grid_row: GridPlacement::try_lines(1, 2).expect("valid first grid row"),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSizeOf::px(40.0), PreferredSizeOf::px(60.0)),
                grid_row: GridPlacement::try_lines(2, 3).expect("valid second grid row"),
                ..NodeInputOf::default()
            },
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(200.0)))
            .expect("valid viewport request"),
    )
    .expect("orthogonal grid-lanes layout succeeds");

    assert_eq!(
        public_flow_output(batch.unrounded_entries(), 2).location,
        Point::new(30.0, 0.0),
        "the selected second row must own the second logical column lane offset"
    );
}

fn assert_logical_grid_lanes_axes<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_track_totals = crate::geometry::LogicalSizeOf::new(scalar(70.0), scalar(110.0));
    let logical_gap = crate::geometry::LogicalSizeOf::new(scalar(7.0), scalar(11.0));
    let logical_container_size = logical_track_totals + logical_gap;
    let child_logical_sizes = [
        crate::geometry::LogicalSizeOf::new(scalar(10.0), scalar(13.0)),
        crate::geometry::LogicalSizeOf::new(scalar(12.0), scalar(17.0)),
        crate::geometry::LogicalSizeOf::new(scalar(11.0), scalar(19.0)),
    ];
    let child_logical_margins = [
        crate::geometry::LogicalEdgesOf::new(scalar(1.0), scalar(2.0), scalar(3.0), scalar(4.0)),
        crate::geometry::LogicalEdgesOf::new(scalar(2.0), scalar(1.0), scalar(4.0), scalar(3.0)),
        crate::geometry::LogicalEdgesOf::new(scalar(3.0), scalar(2.0), scalar(1.0), scalar(5.0)),
    ];

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical_container_size = flow_axes.physical_size(logical_container_size);
        let parent_flow = LogicalFlexChildFlow {
            writing_mode,
            direction,
        };
        let child_flows = [
            parent_flow,
            logical_flex_opposing_flow(parent_flow),
            logical_flex_orthogonal_flow(parent_flow),
        ];

        for (grid_auto_flow, row_flow) in [(GridAutoFlow::Row, true), (GridAutoFlow::Column, false)]
        {
            let lane_axis = if row_flow {
                LogicalAxis::Block
            } else {
                LogicalAxis::Inline
            };
            let first_margin_box = logical_axis_value(
                flow_axes.logical_size(
                    FlowAxes::new(child_flows[0].writing_mode, child_flows[0].direction)
                        .physical_size(child_logical_sizes[0]),
                ),
                lane_axis,
            ) + logical_axis_margin_sum(
                flow_axes.logical_edges(
                    FlowAxes::new(child_flows[0].writing_mode, child_flows[0].direction)
                        .physical_edges(child_logical_margins[0]),
                ),
                lane_axis,
            );
            let expected_origins = if row_flow {
                [
                    crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                    crate::geometry::LogicalPointOf::new(scalar(37.0), S::ZERO),
                    crate::geometry::LogicalPointOf::new(
                        S::ZERO,
                        first_margin_box + logical_gap.block,
                    ),
                ]
            } else {
                [
                    crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                    crate::geometry::LogicalPointOf::new(scalar(37.0), S::ZERO),
                    crate::geometry::LogicalPointOf::new(
                        first_margin_box + logical_gap.inline,
                        S::ZERO,
                    ),
                ]
            };

            let mut tree = PublicFlowTree::default()
                .with_children(0, [1, 2, 3])
                .with_children(1, [])
                .with_children(2, [])
                .with_children(3, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::GridLanes,
                        writing_mode,
                        direction,
                        size: physical_container_size.map(PreferredSizeOf::px),
                        grid_auto_flow,
                        grid_template_columns: vec![
                            TrackComponentOf::px(scalar(30.0)),
                            TrackComponentOf::px(scalar(40.0)),
                        ],
                        grid_template_rows: vec![
                            TrackComponentOf::px(scalar(50.0)),
                            TrackComponentOf::px(scalar(60.0)),
                        ],
                        gap: flow_axes.physical_size(logical_gap.map(LengthOf::px)),
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        justify_items: Some(AlignItems::Start),
                        align_items: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                );

            for ((node, child_flow), (logical_size, logical_margin)) in [1, 2, 3]
                .into_iter()
                .zip(child_flows)
                .zip(child_logical_sizes.into_iter().zip(child_logical_margins))
            {
                let child_flow_axes = FlowAxes::new(child_flow.writing_mode, child_flow.direction);
                let mut child_style = NodeInputOf {
                    display: Display::Block,
                    writing_mode: child_flow.writing_mode,
                    direction: child_flow.direction,
                    size: child_flow_axes
                        .physical_size(logical_size)
                        .map(PreferredSizeOf::px),
                    margin: child_flow_axes.physical_edges(logical_margin.map(LengthAutoOf::px)),
                    ..NodeInputOf::default()
                };
                if row_flow {
                    child_style.grid_column =
                        GridPlacement::try_line(if node == 2 { 2 } else { 1 })
                            .expect("valid grid column");
                } else {
                    child_style.grid_row = GridPlacement::try_line(if node == 2 { 2 } else { 1 })
                        .expect("valid grid row");
                }
                tree = tree.with_style(node, child_style);
            }

            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                    .expect("valid viewport request"),
            )
            .expect("logical grid-lanes public layout succeeds");

            let container = public_flow_output(batch.unrounded_entries(), 0);
            assert_eq!(
                container.size, physical_container_size,
                "{writing_mode:?} {direction:?} {grid_auto_flow:?} container size must project logical tracks and gaps"
            );
            assert_eq!(
                container.content_size, physical_container_size,
                "{writing_mode:?} {direction:?} {grid_auto_flow:?} content extent must stay physical at the output boundary"
            );
            for ((node, child_flow), (logical_size, logical_margin)) in [1, 2, 3]
                .into_iter()
                .zip(child_flows)
                .zip(child_logical_sizes.into_iter().zip(child_logical_margins))
            {
                let child_flow_axes = FlowAxes::new(child_flow.writing_mode, child_flow.direction);
                let physical_size = child_flow_axes.physical_size(logical_size);
                let parent_logical_size = flow_axes.logical_size(physical_size);
                let parent_logical_margin =
                    flow_axes.logical_edges(child_flow_axes.physical_edges(logical_margin));
                let expected_logical_origin = expected_origins[(node - 1) as usize]
                    + crate::geometry::LogicalPointOf::new(
                        logical_axis_start(parent_logical_margin, LogicalAxis::Inline),
                        logical_axis_start(parent_logical_margin, LogicalAxis::Block),
                    );
                let expected_location = flow_axes.physical_point(
                    expected_logical_origin,
                    parent_logical_size,
                    physical_container_size,
                );
                let output = public_flow_output(batch.unrounded_entries(), node);
                assert_eq!(
                    output.size, physical_size,
                    "{writing_mode:?} {direction:?} {grid_auto_flow:?} child {node} must retain physical output geometry"
                );
                assert_eq!(
                    output.location, expected_location,
                    "{writing_mode:?} {direction:?} {grid_auto_flow:?} child {node} must place from logical lanes"
                );
            }

            let intrinsic_child_flow = child_flows[2];
            let intrinsic_child_flow_axes = FlowAxes::new(
                intrinsic_child_flow.writing_mode,
                intrinsic_child_flow.direction,
            );
            let intrinsic_parent_logical_size = if row_flow {
                crate::geometry::LogicalSizeOf::new(scalar(30.0), scalar(20.0))
            } else {
                crate::geometry::LogicalSizeOf::new(scalar(20.0), scalar(50.0))
            };
            let intrinsic_physical_size = flow_axes.physical_size(intrinsic_parent_logical_size);
            let intrinsic_tree = PublicFlowTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::InlineGridLanes,
                        writing_mode,
                        direction,
                        grid_auto_flow,
                        grid_template_columns: if row_flow {
                            vec![TrackComponentOf::AUTO, TrackComponentOf::px(scalar(40.0))]
                        } else {
                            vec![
                                TrackComponentOf::px(scalar(30.0)),
                                TrackComponentOf::px(scalar(40.0)),
                            ]
                        },
                        grid_template_rows: if row_flow {
                            vec![
                                TrackComponentOf::px(scalar(50.0)),
                                TrackComponentOf::px(scalar(60.0)),
                            ]
                        } else {
                            vec![TrackComponentOf::AUTO, TrackComponentOf::px(scalar(60.0))]
                        },
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: intrinsic_child_flow.writing_mode,
                        direction: intrinsic_child_flow.direction,
                        size: intrinsic_child_flow_axes
                            .physical_size(
                                intrinsic_child_flow_axes.logical_size(intrinsic_physical_size),
                            )
                            .map(PreferredSizeOf::px),
                        grid_column: if row_flow {
                            GridPlacement::try_line(1).expect("valid intrinsic grid column")
                        } else {
                            GridPlacement::AUTO
                        },
                        grid_row: if row_flow {
                            GridPlacement::AUTO
                        } else {
                            GridPlacement::try_line(1).expect("valid intrinsic grid row")
                        },
                        ..NodeInputOf::default()
                    },
                );
            let intrinsic_batch = compute_layout(
                &intrinsic_tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                    .expect("valid intrinsic viewport request"),
            )
            .expect("logical intrinsic grid-lanes public layout succeeds");
            assert_eq!(
                public_flow_output(intrinsic_batch.unrounded_entries(), 0).size,
                flow_axes.physical_size(logical_track_totals),
                "{writing_mode:?} {direction:?} {grid_auto_flow:?} intrinsic lanes must size on their logical grid axis"
            );
        }
    }

    assert_logical_grid_lanes_absolute_static::<S>();
}

#[test]
fn logical_grid_lanes_axes_f32() {
    assert_logical_grid_lanes_axes::<f32>();
}

#[test]
fn logical_grid_lanes_axes_f64() {
    assert_logical_grid_lanes_axes::<f64>();
}

fn assert_logical_inherited_grid_axis_contexts_public<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let parent_flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    let logical_parent_size = crate::geometry::LogicalSizeOf::new(scalar(77.0), scalar(121.0));
    let parent_size = parent_flow.physical_size(logical_parent_size);

    for (writing_mode, direction) in root_writing_mode_directions() {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode: parent_flow.writing_mode(),
                    direction: parent_flow.direction(),
                    size: parent_size.map(PreferredSizeOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(scalar(30.0)),
                        TrackComponentOf::px(scalar(40.0)),
                    ],
                    grid_template_rows: vec![
                        TrackComponentOf::px(scalar(50.0)),
                        TrackComponentOf::px(scalar(60.0)),
                    ],
                    gap: parent_flow.physical_size(crate::geometry::LogicalSizeOf::new(
                        LengthOf::px(scalar(7.0)),
                        LengthOf::px(scalar(11.0)),
                    )),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    direction,
                    grid_column: GridPlacement::try_lines(1, -1).expect("valid subgrid columns"),
                    grid_row: GridPlacement::try_lines(1, -1).expect("valid subgrid rows"),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        vec![],
                    ))],
                    grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid inherited grid viewport request"),
        )
        .expect("public inherited grid layout succeeds");
        let child = public_flow_output(batch.unrounded_entries(), 1);
        assert_eq!(
            child.size, parent_size,
            "{writing_mode:?} {direction:?} must preserve inherited physical extent"
        );
        assert_eq!(
            child.content_size, parent_size,
            "{writing_mode:?} {direction:?} must preserve inherited physical content extent"
        );
    }
}

#[test]
fn logical_inherited_grid_axis_contexts_public_f32() {
    assert_logical_inherited_grid_axis_contexts_public::<f32>();
}

#[test]
fn logical_inherited_grid_axis_contexts_public_f64() {
    assert_logical_inherited_grid_axis_contexts_public::<f64>();
}

fn assert_logical_subgrid_axes<S: LayoutScalar>() {
    #[derive(Clone, Copy)]
    struct ExpectedTopology {
        inherited_physical_axis: PhysicalAxis,
        parent_axis: GridAxisKind,
        reversed: bool,
    }

    fn expected_axis(
        writing_mode: WritingMode,
        direction: Direction,
        axis: GridAxisKind,
    ) -> (PhysicalAxis, bool) {
        match (writing_mode, direction, axis) {
            (WritingMode::HorizontalTb, Direction::Ltr, GridAxisKind::Column) => {
                (PhysicalAxis::Horizontal, true)
            }
            (WritingMode::HorizontalTb, Direction::Rtl, GridAxisKind::Column) => {
                (PhysicalAxis::Horizontal, false)
            }
            (WritingMode::HorizontalTb, _, GridAxisKind::Row) => (PhysicalAxis::Vertical, true),
            (
                WritingMode::VerticalRl | WritingMode::SidewaysRl,
                Direction::Ltr,
                GridAxisKind::Column,
            ) => (PhysicalAxis::Vertical, true),
            (
                WritingMode::VerticalRl | WritingMode::SidewaysRl,
                Direction::Rtl,
                GridAxisKind::Column,
            ) => (PhysicalAxis::Vertical, false),
            (WritingMode::VerticalRl | WritingMode::SidewaysRl, _, GridAxisKind::Row) => {
                (PhysicalAxis::Horizontal, false)
            }
            (WritingMode::VerticalLr, Direction::Ltr, GridAxisKind::Column) => {
                (PhysicalAxis::Vertical, true)
            }
            (WritingMode::VerticalLr, Direction::Rtl, GridAxisKind::Column) => {
                (PhysicalAxis::Vertical, false)
            }
            (WritingMode::VerticalLr, _, GridAxisKind::Row) => (PhysicalAxis::Horizontal, true),
            (WritingMode::SidewaysLr, Direction::Ltr, GridAxisKind::Column) => {
                (PhysicalAxis::Vertical, false)
            }
            (WritingMode::SidewaysLr, Direction::Rtl, GridAxisKind::Column) => {
                (PhysicalAxis::Vertical, true)
            }
            (WritingMode::SidewaysLr, _, GridAxisKind::Row) => (PhysicalAxis::Horizontal, true),
        }
    }

    fn expected_topology(
        parent_writing_mode: WritingMode,
        parent_direction: Direction,
        child_flow: LogicalFlexChildFlow,
        child_axis: GridAxisKind,
    ) -> ExpectedTopology {
        let (inherited_physical_axis, child_increases) =
            expected_axis(child_flow.writing_mode, child_flow.direction, child_axis);
        let (parent_inline_axis, parent_inline_increases) =
            expected_axis(parent_writing_mode, parent_direction, GridAxisKind::Column);
        let (parent_block_axis, parent_block_increases) =
            expected_axis(parent_writing_mode, parent_direction, GridAxisKind::Row);
        let (parent_axis, parent_increases) = if parent_inline_axis == inherited_physical_axis {
            (GridAxisKind::Column, parent_inline_increases)
        } else {
            debug_assert_eq!(parent_block_axis, inherited_physical_axis);
            (GridAxisKind::Row, parent_block_increases)
        };
        ExpectedTopology {
            inherited_physical_axis,
            parent_axis,
            reversed: parent_increases != child_increases,
        }
    }

    let scalar = scalar::<S>;
    let logical_parent_size = crate::geometry::LogicalSizeOf::new(scalar(77.0), scalar(121.0));
    let logical_gap = crate::geometry::LogicalSizeOf::new(scalar(7.0), scalar(11.0));

    for (parent_writing_mode, parent_direction) in root_writing_mode_directions() {
        let parent_flow = LogicalFlexChildFlow {
            writing_mode: parent_writing_mode,
            direction: parent_direction,
        };
        let parent_flow_axes = FlowAxes::new(parent_writing_mode, parent_direction);
        let parent_size = parent_flow_axes.physical_size(logical_parent_size);
        for child_flow in [
            parent_flow,
            logical_flex_opposing_flow(parent_flow),
            logical_flex_orthogonal_flow(parent_flow),
        ] {
            let child_flow_axes = FlowAxes::new(child_flow.writing_mode, child_flow.direction);
            for axis in [GridAxisKind::Column, GridAxisKind::Row] {
                let topology =
                    expected_topology(parent_writing_mode, parent_direction, child_flow, axis);
                let inherited_physical_axis = topology.inherited_physical_axis;
                let parent_axis = topology.parent_axis;
                let (cross_first_track, cross_second_track, cross_gap) = match parent_axis {
                    GridAxisKind::Column => (scalar(50.0), scalar(60.0), scalar(11.0)),
                    GridAxisKind::Row => (scalar(30.0), scalar(40.0), scalar(7.0)),
                };
                let mut child_style = NodeInputOf {
                    display: Display::Grid,
                    writing_mode: child_flow.writing_mode,
                    direction: child_flow.direction,
                    grid_column: GridPlacement::try_lines(1, -1)
                        .expect("valid subgrid column span"),
                    grid_row: GridPlacement::try_lines(1, -1).expect("valid subgrid row span"),
                    ..NodeInputOf::default()
                };
                match axis {
                    GridAxisKind::Column => {
                        child_style.grid_template_columns =
                            vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))];
                        child_style.grid_template_rows = vec![
                            TrackComponentOf::px(cross_first_track),
                            TrackComponentOf::px(cross_second_track),
                        ];
                    }
                    GridAxisKind::Row => {
                        child_style.grid_template_rows =
                            vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))];
                        child_style.grid_template_columns = vec![
                            TrackComponentOf::px(cross_first_track),
                            TrackComponentOf::px(cross_second_track),
                        ];
                    }
                }
                child_style.gap =
                    child_flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
                        if axis == GridAxisKind::Column {
                            LengthOf::Normal
                        } else {
                            LengthOf::px(cross_gap)
                        },
                        if axis == GridAxisKind::Row {
                            LengthOf::Normal
                        } else {
                            LengthOf::px(cross_gap)
                        },
                    ));
                let tree = PublicFlowTree::default()
                    .with_children(0, [1])
                    .with_children(1, [2])
                    .with_children(2, [])
                    .with_style(
                        0,
                        NodeInputOf {
                            display: Display::Grid,
                            writing_mode: parent_writing_mode,
                            direction: parent_direction,
                            size: parent_size.map(PreferredSizeOf::px),
                            grid_template_columns: vec![
                                TrackComponentOf::px(scalar(30.0)),
                                TrackComponentOf::px(scalar(40.0)),
                            ],
                            grid_template_rows: vec![
                                TrackComponentOf::px(scalar(50.0)),
                                TrackComponentOf::px(scalar(60.0)),
                            ],
                            gap: parent_flow_axes.physical_size(logical_gap.map(LengthOf::px)),
                            ..NodeInputOf::default()
                        },
                    )
                    .with_style(1, child_style)
                    .with_style(
                        2,
                        NodeInputOf {
                            display: Display::Block,
                            writing_mode: child_flow.writing_mode,
                            direction: child_flow.direction,
                            grid_column: if axis == GridAxisKind::Column {
                                GridPlacement::try_lines(2, 3)
                                    .expect("valid inherited column placement")
                            } else {
                                GridPlacement::try_lines(1, 2)
                                    .expect("valid cross-axis column placement")
                            },
                            grid_row: if axis == GridAxisKind::Row {
                                GridPlacement::try_lines(2, 3)
                                    .expect("valid inherited row placement")
                            } else {
                                GridPlacement::try_lines(1, 2)
                                    .expect("valid cross-axis row placement")
                            },
                            ..NodeInputOf::default()
                        },
                    );

                let batch = compute_layout(
                    &tree,
                    0,
                    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(
                        200.0,
                    ))))
                    .expect("valid subgrid viewport request"),
                )
                .expect("logical subgrid public layout succeeds");
                let child = public_flow_output(batch.unrounded_entries(), 1);
                let inherited_extent = match inherited_physical_axis {
                    PhysicalAxis::Horizontal => child.size.width,
                    PhysicalAxis::Vertical => child.size.height,
                };
                let expected_extent = match inherited_physical_axis {
                    PhysicalAxis::Horizontal => parent_size.width,
                    PhysicalAxis::Vertical => parent_size.height,
                };
                assert_eq!(
                    inherited_extent, expected_extent,
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must preserve the inherited physical extent"
                );
                let child_inputs = tree.cache_inputs(1);
                assert!(
                    child_inputs
                        .iter()
                        .any(|input| input.containing_flow_axes() == parent_flow_axes),
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must compute the subgrid through its parent flow: {child_inputs:?}"
                );
                assert!(
                    child_inputs.iter().any(|input| {
                        let inherited_extent = match topology.inherited_physical_axis {
                            PhysicalAxis::Horizontal => parent_size.width,
                            PhysicalAxis::Vertical => parent_size.height,
                        };
                        let (known, available) = match topology.inherited_physical_axis {
                            PhysicalAxis::Horizontal => {
                                (input.known().width, input.available().width)
                            }
                            PhysicalAxis::Vertical => {
                                (input.known().height, input.available().height)
                            }
                        };
                        input.containing_flow_axes() == parent_flow_axes
                            && known == Some(inherited_extent)
                            && available == AvailableOf::definite(inherited_extent)
                    }),
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must project the inherited physical size and available area through the child flow: {child_inputs:?}"
                );

                let descendant = public_flow_output(batch.unrounded_entries(), 2);
                let descendant_origin = parent_flow_axes.logical_point(
                    descendant.location,
                    descendant.size,
                    parent_size,
                );
                let descendant_size = parent_flow_axes.logical_size(descendant.size);
                let (first_track, second_track, gap) = match parent_axis {
                    GridAxisKind::Column => (scalar(30.0), scalar(40.0), scalar(7.0)),
                    GridAxisKind::Row => (scalar(50.0), scalar(60.0), scalar(11.0)),
                };
                let expected_offset = if topology.reversed {
                    S::ZERO
                } else {
                    first_track + gap
                };
                let (actual_offset, actual_extent) = match parent_axis {
                    GridAxisKind::Column => (descendant_origin.inline, descendant_size.inline),
                    GridAxisKind::Row => (descendant_origin.block, descendant_size.block),
                };
                assert_eq!(
                    actual_offset, expected_offset,
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must place the descendant on the mapped inherited track"
                );
                assert_eq!(
                    actual_extent,
                    if topology.reversed {
                        first_track
                    } else {
                        second_track
                    },
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must preserve the mapped inherited track extent"
                );
            }
        }
    }
}

#[test]
fn logical_subgrid_axes_f32() {
    assert_logical_subgrid_axes::<f32>();
}

#[test]
fn logical_subgrid_axes_f64() {
    assert_logical_subgrid_axes::<f64>();
}

fn assert_nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let parent_flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [2])
        .with_children(2, [3])
        .with_children(3, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::MAX_CONTENT,
                    PreferredSizeOf::px(scalar(40.0)),
                ),
                grid_template_columns: vec![
                    TrackComponentOf::AUTO,
                    TrackComponentOf::AUTO,
                    TrackComponentOf::AUTO,
                ],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalLr,
                grid_column: GridPlacement::try_lines(1, 3)
                    .expect("outer subgrid spans two of three parent columns"),
                grid_template_columns: vec![TrackComponentOf::px(scalar(40.0))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(1)
                    .expect("inner subgrid spans one of two inherited tracks"),
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            3,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(20.0)),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );

    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
            .expect("valid nested provisional subgrid viewport request"),
    )
    .expect("nested orthogonal partial subgrid layout succeeds");

    let inputs = tree.cache_inputs(1);
    assert!(
        inputs.iter().any(|input| {
            input.run_mode() == RunMode::ComputeSize
                && input.known() == Size::new(Some(scalar(20.0)), None)
                && input.parent() == Size::new(Some(scalar(20.0)), Some(S::ZERO))
                && input.containing_flow_axes() == parent_flow_axes
                && input.available()
                    == Size::new(
                        AvailableOf::definite(scalar(20.0)),
                        AvailableOf::MAX_CONTENT,
                    )
        }),
        "nested partial subgrid node 1 must retain a resolved cross-axis span and provisional other axis: {inputs:?}"
    );
}

#[test]
fn nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis_f32()
{
    assert_nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis::<
        f32,
    >();
}

#[test]
fn nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis_f64()
{
    assert_nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis::<
        f64,
    >();
}

fn assert_subgrid_mbp_preserves_area_basis_and_content_capacity<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let assert_approximately = |actual: S, expected: S, label: &str| {
        assert!(
            (actual - expected).abs() <= S::from_f64(0.000_1),
            "{label}: expected {expected:?}, got {actual:?}"
        );
    };
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [2])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(40.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(scalar(100.0))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(8.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(5.0)),
                ),
                border: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::px(scalar(9.0)),
                    LengthOf::ZERO,
                    LengthOf::px(scalar(6.0)),
                ),
                padding: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::percent(scalar(0.10)),
                    LengthOf::ZERO,
                    LengthOf::percent(scalar(0.07)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::percent(scalar(1.0)),
                    PreferredSizeOf::px(scalar(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
            .expect("valid asymmetric subgrid MBP viewport request"),
    )
    .expect("asymmetric subgrid MBP layout succeeds");
    let subgrid = public_flow_output(batch.unrounded_entries(), 1);
    let descendant = public_flow_output(batch.unrounded_entries(), 2);

    assert_eq!(subgrid.location.x, scalar(5.0));
    assert_eq!(subgrid.size.width, scalar(87.0));
    assert_eq!(subgrid.margin.left, scalar(5.0));
    assert_eq!(subgrid.margin.right, scalar(8.0));
    assert_eq!(subgrid.border.left, scalar(6.0));
    assert_eq!(subgrid.border.right, scalar(9.0));
    assert_approximately(
        subgrid.padding.left,
        scalar(7.0),
        "left padding resolves against the raw 100px grid area",
    );
    assert_approximately(
        subgrid.padding.right,
        scalar(10.0),
        "right padding resolves against the raw 100px grid area",
    );
    assert_approximately(
        descendant.location.x,
        scalar(13.0),
        "descendant local x is the subgrid border and padding inset",
    );
    assert_approximately(
        descendant.size.width,
        scalar(55.0),
        "descendant width is the subgrid content capacity",
    );
    assert_approximately(
        subgrid.location.x + descendant.location.x,
        scalar(18.0),
        "subgrid and descendant local coordinates compose to the root-space x",
    );
}

#[test]
fn subgrid_mbp_preserves_area_basis_and_content_capacity_f32() {
    assert_subgrid_mbp_preserves_area_basis_and_content_capacity::<f32>();
}

#[test]
fn subgrid_mbp_preserves_area_basis_and_content_capacity_f64() {
    assert_subgrid_mbp_preserves_area_basis_and_content_capacity::<f64>();
}

fn assert_logical_ordinary_grid_public_contexts<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let viewport = Size::splat(AvailableOf::definite(scalar(200.0)));
    let containing_flow = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let grid_tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [3])
        .with_children(3, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                size: Size::new(
                    PreferredSizeOf::px(scalar(110.0)),
                    PreferredSizeOf::px(scalar(70.0)),
                ),
                grid_template_columns: vec![
                    TrackComponentOf::px(scalar(30.0)),
                    TrackComponentOf::px(scalar(40.0)),
                ],
                grid_template_rows: vec![
                    TrackComponentOf::px(scalar(50.0)),
                    TrackComponentOf::px(scalar(60.0)),
                ],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                position: Position::Absolute,
                size: Size::new(
                    PreferredSizeOf::px(scalar(10.25)),
                    PreferredSizeOf::px(scalar(20.25)),
                ),
                grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::None,
                ..NodeInputOf::default()
            },
        )
        .with_style(3, NodeInputOf::default());

    let viewport_batch = compute_layout(
        &grid_tree,
        0,
        LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
    )
    .expect("viewport ordinary-grid context succeeds");
    let child_entry = viewport_batch
        .cache_store_entries()
        .iter()
        .find(|entry| entry.node() == 1 && entry.input().run_mode() == RunMode::PerformLayout)
        .expect("absolute grid child stores a layout cache entry");
    assert_eq!(child_entry.input().containing_flow_axes(), containing_flow);
    assert_eq!(
        child_entry.output().size,
        Size::new(scalar(10.25), scalar(20.25))
    );
    assert_eq!(
        public_flow_output(viewport_batch.final_entries(), 0).size,
        Size::new(scalar(110.0), scalar(70.0))
    );
    assert_eq!(
        public_flow_output(viewport_batch.unrounded_entries(), 2),
        NodeOutputOf::with_source_index(crate::SourceIndex::new(1))
    );
    assert_eq!(
        public_flow_output(viewport_batch.unrounded_entries(), 3),
        NodeOutputOf::with_source_index(crate::SourceIndex::new(0))
    );

    grid_tree.apply_cache_entries(viewport_batch.cache_store_entries());
    grid_tree.clear_cache_inputs();
    let warm_batch = compute_layout(
        &grid_tree,
        0,
        LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
    )
    .expect("warm viewport ordinary-grid context succeeds");
    assert!(
        grid_tree
            .cache_inputs(1)
            .iter()
            .any(|input| *input == *child_entry.input())
    );
    assert!(
        warm_batch.cache_store_entries().iter().all(|entry| {
            entry.node() != 1 || entry.input().run_mode() != RunMode::PerformLayout
        })
    );

    let flex_batch = compute_layout(
        &grid_tree,
        0,
        LayoutRootRequestOf::flex_item_under_viewport(
            viewport,
            FlexItemRootContextOf::under_viewport(viewport, containing_flow)
                .expect("valid flex item root context"),
        )
        .expect("valid flex item root request"),
    )
    .expect("flex-item ordinary-grid context succeeds");
    assert_eq!(
        public_flow_output(flex_batch.final_entries(), 1),
        public_flow_output(warm_batch.final_entries(), 1)
    );
}

#[test]
fn logical_ordinary_grid_public_contexts_f32() {
    assert_logical_ordinary_grid_public_contexts::<f32>();
}

#[test]
fn logical_ordinary_grid_public_contexts_f64() {
    assert_logical_ordinary_grid_public_contexts::<f64>();
}

fn assert_logical_ordinary_grid_in_flow_placement_public_output<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_container_size = crate::geometry::LogicalSizeOf::new(scalar(70.0), scalar(110.0));
    let child_size = Size::new(scalar(11.25), scalar(13.5));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let parallel_flow = LogicalGridChildFlow {
            writing_mode,
            direction,
        };
        let opposing_flow = logical_grid_opposing_flow(parallel_flow);
        let orthogonal_flow = logical_grid_orthogonal_flow(parallel_flow);
        let child_flows = [
            parallel_flow,
            opposing_flow,
            orthogonal_flow,
            logical_grid_opposing_flow(orthogonal_flow),
        ];
        let area_origins = [
            (scalar(0.0), scalar(0.0), scalar(30.0), scalar(50.0)),
            (scalar(30.0), scalar(0.0), scalar(40.0), scalar(50.0)),
            (scalar(0.0), scalar(50.0), scalar(30.0), scalar(60.0)),
            (scalar(30.0), scalar(50.0), scalar(40.0), scalar(60.0)),
        ];
        let alignments = [
            (AlignItems::End, AlignItems::Center),
            (AlignItems::Center, AlignItems::Start),
            (AlignItems::Start, AlignItems::End),
            (AlignItems::End, AlignItems::End),
        ];
        let logical_margins = [
            crate::geometry::LogicalEdgesOf::new(
                scalar(1.25),
                scalar(2.5),
                scalar(3.75),
                scalar(4.25),
            ),
            crate::geometry::LogicalEdgesOf::new(
                scalar(2.25),
                scalar(1.5),
                scalar(4.5),
                scalar(3.25),
            ),
            crate::geometry::LogicalEdgesOf::new(
                scalar(3.5),
                scalar(2.0),
                scalar(1.25),
                scalar(5.0),
            ),
            crate::geometry::LogicalEdgesOf::new(
                scalar(1.5),
                scalar(3.75),
                scalar(2.25),
                scalar(4.5),
            ),
        ];
        let relative_offsets = [
            crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
            crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
            crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
            crate::geometry::LogicalPointOf::new(scalar(2.5), -scalar(1.25)),
        ];

        let mut tree = PublicFlowTree::default()
            .with_children(0, [1, 2, 3, 4])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_children(4, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    direction,
                    size: flow_axes
                        .physical_size(logical_container_size)
                        .map(PreferredSizeOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(scalar(30.0)),
                        TrackComponentOf::px(scalar(40.0)),
                    ],
                    grid_template_rows: vec![
                        TrackComponentOf::px(scalar(50.0)),
                        TrackComponentOf::px(scalar(60.0)),
                    ],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            );

        for (index, ((child_flow, (justify_self, align_self)), logical_margin)) in child_flows
            .into_iter()
            .zip(alignments)
            .zip(logical_margins)
            .enumerate()
        {
            let logical_inset = crate::geometry::LogicalEdgesOf::new(
                LengthAutoOf::px(relative_offsets[index].inline),
                LengthAutoOf::AUTO,
                LengthAutoOf::px(relative_offsets[index].block),
                LengthAutoOf::AUTO,
            );
            tree = tree.with_style(
                index as u32 + 1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: child_flow.writing_mode,
                    direction: child_flow.direction,
                    size: child_size.map(PreferredSizeOf::px),
                    margin: flow_axes.physical_edges(logical_margin.map(LengthAutoOf::px)),
                    inset: flow_axes.physical_edges(logical_inset),
                    position: Position::Relative,
                    justify_self: Some(justify_self),
                    align_self: Some(align_self),
                    grid_column: GridPlacement::try_line(index as isize % 2 + 1)
                        .expect("test grid column is valid"),
                    grid_row: GridPlacement::try_line(index as isize / 2 + 1)
                        .expect("test grid row is valid"),
                    ..NodeInputOf::default()
                },
            );
        }

        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("logical ordinary-grid in-flow placement succeeds");
        let root_unrounded = public_flow_output(batch.unrounded_entries(), 0);

        for (
            index,
            (
                (
                    (inline_origin, block_origin, inline_size, block_size),
                    (justify_self, align_self),
                ),
                logical_margin,
            ),
        ) in area_origins
            .into_iter()
            .zip(alignments)
            .zip(logical_margins)
            .enumerate()
        {
            let logical_child_size = flow_axes.logical_size(child_size);
            let inline_offset = match justify_self {
                AlignItems::Start => logical_margin.inline_start,
                AlignItems::End => {
                    inline_size - logical_child_size.inline - logical_margin.inline_end
                }
                AlignItems::Center => {
                    (inline_size - logical_child_size.inline + logical_margin.inline_start
                        - logical_margin.inline_end)
                        / scalar(2.0)
                }
                _ => unreachable!("the test only uses resolved item alignments"),
            };
            let block_offset = match align_self {
                AlignItems::Start => logical_margin.block_start,
                AlignItems::End => block_size - logical_child_size.block - logical_margin.block_end,
                AlignItems::Center => {
                    (block_size - logical_child_size.block + logical_margin.block_start
                        - logical_margin.block_end)
                        / scalar(2.0)
                }
                _ => unreachable!("the test only uses resolved item alignments"),
            };
            let logical_location = crate::geometry::LogicalPointOf::new(
                inline_origin + inline_offset + relative_offsets[index].inline,
                block_origin + block_offset + relative_offsets[index].block,
            );
            let expected_location = flow_axes.physical_point(
                logical_location,
                logical_child_size,
                flow_axes.physical_size(logical_container_size),
            );
            let unrounded = public_flow_output(batch.unrounded_entries(), index as u32 + 1);
            let rounded = public_flow_output(batch.final_entries(), index as u32 + 1);
            let physical_margin = flow_axes.physical_edges(logical_margin);
            let cumulative_x = root_unrounded.location.x + unrounded.location.x;
            let cumulative_y = root_unrounded.location.y + unrounded.location.y;

            assert_eq!(
                unrounded.location,
                expected_location,
                "{writing_mode:?} {direction:?} child {} must project its logical grid area once",
                index + 1
            );
            assert_eq!(unrounded.size, child_size);
            assert_eq!(unrounded.margin, physical_margin);
            assert_eq!(
                rounded.location,
                Point::new(
                    nearest_css_pixel(unrounded.location.x),
                    nearest_css_pixel(unrounded.location.y),
                )
            );
            assert_eq!(
                rounded.size,
                Size::new(
                    nearest_css_pixel(cumulative_x + unrounded.size.width)
                        - nearest_css_pixel(cumulative_x),
                    nearest_css_pixel(cumulative_y + unrounded.size.height)
                        - nearest_css_pixel(cumulative_y),
                )
            );
        }
    }
}

#[test]
fn logical_ordinary_grid_in_flow_placement_public_output_f32() {
    assert_logical_ordinary_grid_in_flow_placement_public_output::<f32>();
}

#[test]
fn logical_ordinary_grid_in_flow_placement_public_output_f64() {
    assert_logical_ordinary_grid_in_flow_placement_public_output::<f64>();
}

fn assert_logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [3])
        .with_children(2, [4])
        .with_children(3, [])
        .with_children(4, [])
        .with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(30.0)),
                    PreferredSizeOf::px(scalar(60.0)),
                ),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            3,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(20.0)),
                    PreferredSizeOf::px(scalar(30.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            4,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(20.0)),
                    PreferredSizeOf::px(scalar(70.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(30.0)),
            AvailableOf::definite(scalar(60.0)),
        ))
        .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 1).size,
        Size::new(scalar(30.0), scalar(30.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).size,
        Size::new(scalar(30.0), scalar(70.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 1).content_size,
        Size::new(scalar(30.0), scalar(30.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).content_size,
        Size::new(scalar(30.0), scalar(70.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 0)
            .content_size
            .height,
        scalar(100.0)
    );
}

#[test]
fn logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions_for_f32() {
    assert_logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions::<f32>();
}

#[test]
fn logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions_for_f64() {
    assert_logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions::<f64>();
}

fn flex_item_style<S: LayoutScalar>(flex_basis: S) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::splat_clone(PreferredSizeOf::px(scalar(10.0))),
        flex_basis: FlexBasisOf::px(flex_basis),
        flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow factor"),
        ..NodeInputOf::default()
    }
}

fn assert_logical_flex_sizing_wrap_thresholds_select_container_axes<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    for direction in [
        FlexDirection::Row,
        FlexDirection::RowReverse,
        FlexDirection::Column,
        FlexDirection::ColumnReverse,
    ] {
        let (container_size, bases, expected_sizes) = if direction.is_row() {
            (
                Size::new(
                    PreferredSizeOf::px(scalar(80.0)),
                    PreferredSizeOf::px(scalar(50.0)),
                ),
                [scalar(30.0), scalar(30.0), scalar(20.0)],
                [
                    Size::new(scalar(10.0), scalar(50.0)),
                    Size::new(scalar(10.0), scalar(30.0)),
                    Size::new(scalar(10.0), scalar(20.0)),
                ],
            )
        } else {
            (
                Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(80.0)),
                ),
                [scalar(60.0), scalar(60.0), scalar(40.0)],
                [
                    Size::new(scalar(100.0), scalar(10.0)),
                    Size::new(scalar(60.0), scalar(10.0)),
                    Size::new(scalar(40.0), scalar(10.0)),
                ],
            )
        };
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInputOf {
                    writing_mode: WritingMode::VerticalLr,
                    size: container_size,
                    flex_direction: direction,
                    flex_wrap: FlexWrap::Wrap,
                    ..NodeInputOf::default()
                },
            )
            .with_style(1, flex_item_style(bases[0]))
            .with_style(2, flex_item_style(bases[1]))
            .with_style(3, flex_item_style(bases[2]));
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(scalar(100.0)),
                AvailableOf::definite(scalar(100.0)),
            ))
            .expect("valid viewport request"),
        )
        .expect("non-leaf flex root layout succeeds");

        for (node, expected_size) in [1_u32, 2, 3].into_iter().zip(expected_sizes) {
            assert_eq!(
                public_flow_output(batch.final_entries(), node).size,
                expected_size
            );
        }
    }
}

#[test]
fn logical_flex_sizing_wrap_thresholds_select_container_axes_for_f32() {
    assert_logical_flex_sizing_wrap_thresholds_select_container_axes::<f32>();
}

#[test]
fn logical_flex_sizing_wrap_thresholds_select_container_axes_for_f64() {
    assert_logical_flex_sizing_wrap_thresholds_select_container_axes::<f64>();
}

fn assert_logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let item = NodeInputOf {
        display: Display::Block,
        size: Size::splat_clone(PreferredSizeOf::px(scalar(10.0))),
        flex_basis: FlexBasisOf::px(scalar(45.0)),
        flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow factor"),
        margin: Edges::all(LengthAutoOf::percent(scalar(0.1))),
        ..NodeInputOf::default()
    };
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2, 3])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(200.0)),
                ),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                gap: Size::new(
                    LengthOf::percent(scalar(0.1)),
                    LengthOf::percent(scalar(0.1)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, item.clone())
        .with_style(2, item.clone())
        .with_style(3, item);
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(100.0)),
            AvailableOf::definite(scalar(200.0)),
        ))
        .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    for node in [1_u32, 2, 3] {
        let output = public_flow_output(batch.final_entries(), node);
        assert_eq!(output.margin, Edges::all(scalar(20.0)));
    }
    assert_eq!(
        public_flow_output(batch.final_entries(), 1).size.height,
        scalar(50.0)
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).size.height,
        scalar(50.0)
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 3).size.height,
        scalar(160.0)
    );
}

#[test]
fn logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes_for_f32() {
    assert_logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes::<f32>();
}

#[test]
fn logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes_for_f64() {
    assert_logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes::<f64>();
}

fn assert_logical_flex_sizing_preserves_horizontal_and_child_flow_ownership<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2, 3])
        .with_children(1, [4])
        .with_children(2, [5])
        .with_children(3, [6])
        .with_children(4, [])
        .with_children(5, [])
        .with_children(6, [])
        .with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(120.0)),
                ),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(30.0)),
                    PreferredSizeOf::px(scalar(40.0)),
                ),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(scalar(30.0)),
                    PreferredSizeOf::px(scalar(40.0)),
                ),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            3,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::HorizontalTb,
                size: Size::new(
                    PreferredSizeOf::px(scalar(30.0)),
                    PreferredSizeOf::px(scalar(40.0)),
                ),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            4,
            NodeInputOf {
                display: Display::Block,
                flex_basis: FlexBasisOf::percent(scalar(0.5)),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            5,
            NodeInputOf {
                display: Display::Block,
                flex_basis: FlexBasisOf::percent(scalar(0.5)),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            6,
            NodeInputOf {
                display: Display::Block,
                flex_basis: FlexBasisOf::percent(scalar(0.5)),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(100.0)),
            AvailableOf::definite(scalar(120.0)),
        ))
        .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 4).size,
        Size::new(scalar(30.0), scalar(20.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 5).size,
        Size::new(scalar(30.0), scalar(20.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 6).size,
        Size::new(scalar(15.0), scalar(40.0))
    );

    let horizontal = PublicFlowTree::default()
        .with_children(0, [1, 2, 3])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_style(
            0,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(80.0)),
                ),
                flex_wrap: FlexWrap::Wrap,
                gap: Size::new(LengthOf::ZERO, LengthOf::percent(scalar(0.1))),
                align_content: Some(AlignContent::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, flex_item_style(scalar(60.0)))
        .with_style(2, flex_item_style(scalar(60.0)))
        .with_style(3, flex_item_style(scalar(40.0)));
    let horizontal_batch = compute_layout(
        &horizontal,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(100.0)),
            AvailableOf::definite(scalar(80.0)),
        ))
        .expect("valid viewport request"),
    )
    .expect("horizontal non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(horizontal_batch.final_entries(), 1).size,
        Size::new(scalar(100.0), scalar(10.0))
    );
    assert_eq!(
        public_flow_output(horizontal_batch.final_entries(), 2).size,
        Size::new(scalar(60.0), scalar(10.0))
    );
    assert_eq!(
        public_flow_output(horizontal_batch.final_entries(), 3).size,
        Size::new(scalar(40.0), scalar(10.0))
    );
    assert_eq!(
        public_flow_output(horizontal_batch.final_entries(), 0)
            .content_size
            .height,
        scalar(80.0)
    );
}

#[test]
fn logical_flex_sizing_preserves_horizontal_and_child_flow_ownership_for_f32() {
    assert_logical_flex_sizing_preserves_horizontal_and_child_flow_ownership::<f32>();
}

#[test]
fn logical_flex_sizing_preserves_horizontal_and_child_flow_ownership_for_f64() {
    assert_logical_flex_sizing_preserves_horizontal_and_child_flow_ownership::<f64>();
}

fn assert_logical_flex_public_contexts<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let vertical_containing_flow = FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl);
    let horizontal_containing_flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);

    // A non-leaf flex root passes its own vertical containing flow to children
    // whose own flows differ, so percentage sizing remains owned by the parent.
    assert_logical_flex_sizing_preserves_horizontal_and_child_flow_ownership::<S>();

    let flex_root = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.0, 20.0))
        .with_style(2, logical_flex_leaf(10.0, 20.0));
    let viewport = Size::splat(AvailableOf::definite(scalar(100.0)));
    let flex_root_batch = compute_layout(
        &flex_root,
        0,
        LayoutRootRequestOf::flex_item_under_viewport(
            viewport,
            FlexItemRootContextOf::under_viewport(viewport, vertical_containing_flow)
                .expect("valid flex item root viewport context"),
        )
        .expect("valid flex item root request"),
    )
    .expect("public flex item root layout succeeds");
    assert_eq!(
        public_flow_output(flex_root_batch.final_entries(), 0).location,
        Point::ZERO
    );
    assert_eq!(
        public_flow_output(flex_root_batch.final_entries(), 0).size,
        Size::splat(scalar(100.0))
    );
    assert_eq!(
        public_flow_output(flex_root_batch.final_entries(), 1).location,
        Point::new(S::ZERO, S::ZERO)
    );
    assert_eq!(
        public_flow_output(flex_root_batch.final_entries(), 2).location,
        Point::new(S::ZERO, scalar(20.0))
    );

    let cache_tree = |writing_mode, direction| {
        PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    writing_mode,
                    direction,
                    size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                    flex_direction: FlexDirection::Row,
                    ..NodeInputOf::default()
                },
            )
            .with_style(1, logical_flex_leaf(10.25, 20.25))
    };
    let cache_request =
        LayoutRootRequestOf::viewport(viewport).expect("valid cache viewport request");
    let vertical_cache_tree = cache_tree(WritingMode::VerticalLr, Direction::Rtl);
    let cold_cache_batch = compute_layout(&vertical_cache_tree, 0, cache_request)
        .expect("cold non-leaf flex cache traversal succeeds");
    let cold_child_entry = cold_cache_batch
        .cache_store_entries()
        .iter()
        .find(|entry| entry.node() == 1 && entry.input().run_mode() == RunMode::PerformLayout)
        .expect("cold flex traversal stages the child final-layout cache output");
    assert_eq!(
        cold_child_entry.input().containing_flow_axes(),
        vertical_containing_flow
    );
    assert_eq!(
        cold_child_entry.output().size,
        Size::new(scalar(10.25), scalar(20.25))
    );
    assert_eq!(
        cold_child_entry.output().content_size,
        Size::new(scalar(10.25), scalar(20.25))
    );
    let cold_child = public_flow_output(cold_cache_batch.final_entries(), 1);
    assert_eq!(cold_child.source_index, crate::SourceIndex::new(0));
    assert_eq!(cold_child.location, Point::new(S::ZERO, scalar(80.0)));
    assert_eq!(cold_child.size, Size::new(scalar(10.0), scalar(20.0)));
    assert_eq!(
        cold_child.content_size,
        Size::new(scalar(10.0), scalar(20.0))
    );
    assert_eq!(cold_child.border, Edges::ZERO);
    assert_eq!(cold_child.padding, Edges::ZERO);
    assert_eq!(cold_child.margin, Edges::ZERO);
    let cold_child_geometry = cold_child
        .scroll_geometry
        .expect("performed flex child retains canonical geometry");
    assert_eq!(cold_child_geometry.border_box().size(), cold_child.size);
    assert_eq!(
        cold_child_geometry.target().border_box(),
        cold_child_geometry.border_box()
    );
    assert_eq!(
        cold_child.scrollbar_size(),
        cold_child_geometry.scrollbar_size()
    );

    vertical_cache_tree.apply_cache_entries(cold_cache_batch.cache_store_entries());
    vertical_cache_tree.clear_cache_inputs();
    let warm_cache_batch = compute_layout(&vertical_cache_tree, 0, cache_request)
        .expect("matching public flex cache traversal succeeds");
    assert!(
        vertical_cache_tree
            .cache_inputs(1)
            .iter()
            .any(|input| *input == *cold_child_entry.input())
    );
    assert!(
        warm_cache_batch.cache_store_entries().iter().all(|entry| {
            entry.node() != 1 || entry.input().run_mode() != RunMode::PerformLayout
        })
    );
    assert_eq!(
        public_flow_output(warm_cache_batch.final_entries(), 1),
        public_flow_output(cold_cache_batch.final_entries(), 1)
    );

    let horizontal_cache_tree = cache_tree(WritingMode::HorizontalTb, Direction::Ltr);
    horizontal_cache_tree.apply_cache_entries(&[*cold_child_entry]);
    let distinct_flow_batch = compute_layout(&horizontal_cache_tree, 0, cache_request)
        .expect("distinct-flow public flex cache traversal succeeds");
    assert!(
        horizontal_cache_tree
            .cache_inputs(1)
            .iter()
            .any(|input| input.containing_flow_axes() == horizontal_containing_flow)
    );
    assert!(
        distinct_flow_batch
            .cache_store_entries()
            .iter()
            .any(|entry| {
                entry.node() == 1
                    && entry.input().run_mode() == RunMode::PerformLayout
                    && entry.input().containing_flow_axes() == horizontal_containing_flow
            })
    );

    let hidden = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [2])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::None,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                ..NodeInputOf::default()
            },
        )
        .with_style(2, logical_flex_leaf(20.0, 10.0));
    let hidden_batch = compute_layout(
        &hidden,
        0,
        LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
    )
    .expect("hidden flex descendant layout succeeds");
    assert_eq!(
        hidden_batch
            .cache_clear_entries()
            .iter()
            .map(|entry| entry.node())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    for node in [1, 2] {
        assert_eq!(
            public_flow_output(hidden_batch.unrounded_entries(), node),
            NodeOutputOf::with_source_index(crate::SourceIndex::new(0))
        );
        assert_eq!(
            public_flow_output(hidden_batch.final_entries(), node),
            NodeOutputOf::with_source_index(crate::SourceIndex::new(0))
        );
    }

    let fractional = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.5))),
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::FlexEnd),
                justify_content: Some(AlignContent::FlexEnd),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.25, 20.25));
    let fractional_batch = compute_layout(
        &fractional,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.5))))
            .expect("valid fractional viewport request"),
    )
    .expect("fractional non-horizontal flex layout succeeds");
    assert_eq!(
        public_flow_output(fractional_batch.unrounded_entries(), 1).location,
        Point::new(scalar(90.25), scalar(80.25))
    );
    assert_eq!(
        public_flow_output(fractional_batch.unrounded_entries(), 1).size,
        Size::new(scalar(10.25), scalar(20.25))
    );
    assert_eq!(
        public_flow_output(fractional_batch.final_entries(), 1).location,
        Point::new(scalar(90.0), scalar(80.0))
    );
    assert_eq!(
        public_flow_output(fractional_batch.final_entries(), 1).size,
        Size::new(scalar(11.0), scalar(21.0))
    );
}

#[test]
fn logical_flex_public_contexts_preserve_flow_and_physical_output_for_f32() {
    assert_logical_flex_public_contexts::<f32>();
}

#[test]
fn logical_flex_public_contexts_preserve_flow_and_physical_output_for_f64() {
    assert_logical_flex_public_contexts::<f64>();
}

fn assert_viewport_root_logical_inline_auto_fill<S: LayoutScalar>(
    writing_mode: WritingMode,
    expected_location: Point<S>,
) {
    let tree = FlowRootLeafTree::new(NodeInputOf::<S> {
        writing_mode,
        size: Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO),
        ..NodeInputOf::default()
    });
    let viewport = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );
    let request = LayoutRootRequestOf::viewport(viewport).expect("valid viewport request");

    let batch = compute_layout(&tree, 0, request).expect("root layout succeeds");
    let output = single_final_output(&batch);

    assert_eq!(output.size, Size::new(scalar(20.0), scalar(110.0)));
    assert_eq!(output.location, expected_location);
}

fn assert_horizontal_viewport_root_logical_inline_auto_fill<S: LayoutScalar>() {
    let tree = FlowRootLeafTree::new(NodeInputOf::<S> {
        writing_mode: WritingMode::HorizontalTb,
        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(30.0))),
        ..NodeInputOf::default()
    });
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    ))
    .expect("valid viewport request");

    let batch = compute_layout(&tree, 0, request).expect("root layout succeeds");
    let output = single_final_output(&batch);

    assert_eq!(output.size, Size::new(scalar(70.0), scalar(30.0)));
    assert_eq!(output.location, Point::ZERO);
}

#[test]
fn root_flow_logical_inline_auto_fill_and_start_placement_work_for_f32() {
    assert_horizontal_viewport_root_logical_inline_auto_fill::<f32>();
    assert_viewport_root_logical_inline_auto_fill::<f32>(
        WritingMode::VerticalRl,
        Point::new(50.0, 0.0),
    );
    assert_viewport_root_logical_inline_auto_fill::<f32>(
        WritingMode::SidewaysLr,
        Point::new(0.0, 0.0),
    );
}

#[test]
fn root_flow_logical_inline_auto_fill_and_start_placement_work_for_f64() {
    assert_horizontal_viewport_root_logical_inline_auto_fill::<f64>();
    assert_viewport_root_logical_inline_auto_fill::<f64>(
        WritingMode::VerticalRl,
        Point::new(50.0, 0.0),
    );
    assert_viewport_root_logical_inline_auto_fill::<f64>(
        WritingMode::SidewaysLr,
        Point::new(0.0, 0.0),
    );
}

fn assert_ordinary_block_root_contexts<S: LayoutScalar>() {
    let viewport = Size::new(
        AvailableOf::definite(scalar::<S>(100.0)),
        AvailableOf::definite(scalar::<S>(100.0)),
    );
    let logical_size = crate::geometry::LogicalSizeOf::new(scalar::<S>(20.0), scalar::<S>(10.0));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let size = flow_axes.physical_size(logical_size);
        let style = NodeInputOf::<S> {
            display: Display::Block,
            writing_mode,
            direction,
            size: size.map(PreferredSizeOf::px),
            ..NodeInputOf::default()
        };

        let viewport_tree = FlowRootLeafTree::new(style.clone());
        let viewport_batch = compute_layout(
            &viewport_tree,
            0,
            LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
        )
        .expect("viewport root layout succeeds");
        let viewport_output = single_final_output(&viewport_batch);
        assert_eq!(viewport_output.size, size);
        assert_eq!(
            viewport_output.location,
            flow_axes.physical_point(
                crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                logical_size,
                Size::new(scalar::<S>(100.0), scalar::<S>(100.0)),
            )
        );

        let flex_tree = FlowRootLeafTree::new(style);
        let flex_batch = compute_layout(
            &flex_tree,
            0,
            LayoutRootRequestOf::flex_item_under_viewport(
                viewport,
                FlexItemRootContextOf::under_viewport(viewport, flow_axes)
                    .expect("valid flex root viewport context"),
            )
            .expect("valid flex root request"),
        )
        .expect("flex root layout succeeds");
        assert_eq!(single_final_output(&flex_batch).size, size);
    }
}

#[test]
fn ordinary_block_root_contexts_preserve_all_flow_mappings_for_f32() {
    assert_ordinary_block_root_contexts::<f32>();
}

#[test]
fn ordinary_block_root_contexts_preserve_all_flow_mappings_for_f64() {
    assert_ordinary_block_root_contexts::<f64>();
}

fn assert_ordinary_block_root_contexts_clear_hidden_descendants<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let viewport = Size::splat(AvailableOf::definite(scalar(100.0)));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [2])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::splat_clone(PreferredSizeOf::px(scalar(100.0))),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::None,
                    writing_mode: WritingMode::HorizontalTb,
                    direction: Direction::Ltr,
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::splat_clone(PreferredSizeOf::px(scalar(20.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
        )
        .expect("hidden descendant layout succeeds");

        assert_eq!(
            batch
                .cache_clear_entries()
                .iter()
                .map(|entry| entry.node())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        for node in [1, 2] {
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), node),
                NodeOutputOf::with_source_index(crate::SourceIndex::new(0))
            );
            assert_eq!(
                public_flow_output(batch.final_entries(), node),
                NodeOutputOf::with_source_index(crate::SourceIndex::new(0))
            );
        }
    }
}

#[test]
fn ordinary_block_root_contexts_clear_hidden_descendants_for_all_flows_f32() {
    assert_ordinary_block_root_contexts_clear_hidden_descendants::<f32>();
}

#[test]
fn ordinary_block_root_contexts_clear_hidden_descendants_for_all_flows_f64() {
    assert_ordinary_block_root_contexts_clear_hidden_descendants::<f64>();
}

fn assert_root_flow_opposite_edge_uses_only_definite_extent<S: LayoutScalar>() {
    let style = NodeInputOf::<S> {
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(
            PreferredSizeOf::px(scalar(20.0)),
            PreferredSizeOf::px(scalar(30.0)),
        ),
        ..NodeInputOf::default()
    };
    let definite_tree = FlowRootLeafTree::new(style.clone());
    let definite_request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    ))
    .expect("valid definite viewport request");
    let definite =
        compute_layout(&definite_tree, 0, definite_request).expect("definite root layout succeeds");
    assert_eq!(
        single_final_output(&definite).location,
        Point::new(scalar(50.0), S::ZERO)
    );

    let intrinsic_tree = FlowRootLeafTree::new(style);
    let intrinsic_request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::MAX_CONTENT,
        AvailableOf::definite(scalar(110.0)),
    ))
    .expect("valid intrinsic viewport request");
    let intrinsic = compute_layout(&intrinsic_tree, 0, intrinsic_request)
        .expect("intrinsic root layout succeeds");
    assert_eq!(single_final_output(&intrinsic).location, Point::ZERO);

    let sideways_style = NodeInputOf::<S> {
        writing_mode: WritingMode::SidewaysLr,
        size: Size::new(
            PreferredSizeOf::px(scalar(20.0)),
            PreferredSizeOf::px(scalar(30.0)),
        ),
        ..NodeInputOf::default()
    };
    let sideways_definite_tree = FlowRootLeafTree::new(sideways_style.clone());
    let sideways_definite_request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    ))
    .expect("valid definite sideways viewport request");
    let sideways_definite = compute_layout(&sideways_definite_tree, 0, sideways_definite_request)
        .expect("definite sideways root layout succeeds");
    assert_eq!(
        single_final_output(&sideways_definite).location,
        Point::new(S::ZERO, scalar(80.0))
    );

    let sideways_intrinsic_tree = FlowRootLeafTree::new(sideways_style);
    let sideways_intrinsic_request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::MAX_CONTENT,
    ))
    .expect("valid intrinsic sideways viewport request");
    let sideways_intrinsic =
        compute_layout(&sideways_intrinsic_tree, 0, sideways_intrinsic_request)
            .expect("intrinsic sideways root layout succeeds");
    assert_eq!(
        single_final_output(&sideways_intrinsic).location,
        Point::ZERO
    );
}

#[test]
fn root_flow_opposite_edge_uses_only_definite_extent_for_f32() {
    assert_root_flow_opposite_edge_uses_only_definite_extent::<f32>();
}

#[test]
fn root_flow_opposite_edge_uses_only_definite_extent_for_f64() {
    assert_root_flow_opposite_edge_uses_only_definite_extent::<f64>();
}

fn assert_root_and_flex_root_percentage_edges_use_logical_inline_basis<S: LayoutScalar>() {
    let style = NodeInputOf::<S> {
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(
            PreferredSizeOf::px(scalar(20.0)),
            PreferredSizeOf::px(scalar(30.0)),
        ),
        margin: Edges::all(LengthAutoOf::percent(scalar(0.3))),
        padding: Edges::all(LengthOf::percent(scalar(0.1))),
        border: Edges::all(LengthOf::percent(scalar(0.2))),
        ..NodeInputOf::default()
    };
    let viewport = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );

    let viewport_tree = FlowRootLeafTree::new(style.clone());
    let viewport_batch = compute_layout(
        &viewport_tree,
        0,
        LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
    )
    .expect("viewport root layout succeeds");
    let viewport_output = single_final_output(&viewport_batch);
    assert_eq!(viewport_output.margin, Edges::all(scalar(33.0)));
    assert_eq!(viewport_output.padding, Edges::all(scalar(11.0)));
    assert_eq!(viewport_output.border, Edges::all(scalar(22.0)));

    let flex_tree = FlowRootLeafTree::new(style);
    let flex_batch = compute_layout(
        &flex_tree,
        0,
        LayoutRootRequestOf::flex_item_under_viewport(
            Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
            FlexItemRootContextOf::under_viewport(
                viewport,
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
            )
            .expect("valid flex root viewport context"),
        )
        .expect("valid flex root request"),
    )
    .expect("flex root layout succeeds");
    let flex_output = single_final_output(&flex_batch);
    assert_eq!(flex_output.location, Point::ZERO);
    assert_eq!(flex_output.margin, Edges::all(scalar(33.0)));
    assert_eq!(flex_output.padding, Edges::all(scalar(11.0)));
    assert_eq!(flex_output.border, Edges::all(scalar(22.0)));
}

#[test]
fn root_flow_percentage_edges_use_vertical_inline_extent_for_f32() {
    assert_root_and_flex_root_percentage_edges_use_logical_inline_basis::<f32>();
}

#[test]
fn root_flow_percentage_edges_use_vertical_inline_extent_for_f64() {
    assert_root_and_flex_root_percentage_edges_use_logical_inline_basis::<f64>();
}

fn assert_flex_root_flow_known_inline_uses_host_availability<S: LayoutScalar>() {
    let host = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );
    let viewport = Size::new(
        AvailableOf::definite(scalar(130.0)),
        AvailableOf::definite(scalar(210.0)),
    );

    for writing_mode in [WritingMode::VerticalRl, WritingMode::SidewaysLr] {
        let style = NodeInputOf::<S> {
            writing_mode,
            size: Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO),
            ..NodeInputOf::default()
        };
        let tree = FlowRootLeafTree::new(style.clone());
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::flex_item_under_viewport(
                host,
                FlexItemRootContextOf::under_viewport(
                    viewport,
                    FlowAxes::new(writing_mode, Direction::Ltr),
                )
                .expect("valid flex root viewport context"),
            )
            .expect("valid flex root request"),
        )
        .expect("flex root layout succeeds");
        let output = single_final_output(&batch);

        assert_eq!(output.size, Size::new(scalar(20.0), scalar(110.0)));
        assert_eq!(output.location, Point::ZERO);

        let unavailable_tree = FlowRootLeafTree::new(style);
        let unavailable = compute_layout(
            &unavailable_tree,
            0,
            LayoutRootRequestOf::flex_item_under_viewport(
                Size::new(
                    AvailableOf::definite(scalar(70.0)),
                    AvailableOf::MAX_CONTENT,
                ),
                FlexItemRootContextOf::under_viewport(
                    viewport,
                    FlowAxes::new(writing_mode, Direction::Ltr),
                )
                .expect("valid flex root viewport context"),
            )
            .expect("valid intrinsic flex root request"),
        )
        .expect("intrinsic flex root layout succeeds");
        let unavailable_output = single_final_output(&unavailable);

        assert_eq!(unavailable_output.size, Size::new(scalar(20.0), S::ZERO));
        assert_eq!(unavailable_output.location, Point::ZERO);
    }
}

#[test]
fn flex_root_flow_known_inline_uses_host_availability_for_f32() {
    assert_flex_root_flow_known_inline_uses_host_availability::<f32>();
}

#[test]
fn flex_root_flow_known_inline_uses_host_availability_for_f64() {
    assert_flex_root_flow_known_inline_uses_host_availability::<f64>();
}

#[test]
fn compute_layout_uses_flex_root_viewport_context_as_parent_basis() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(PreferredSize::percent(0.5), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    let viewport = Size::new(Available::definite(200.0), Available::definite(80.0));
    let request = LayoutRootRequest::flex_item_under_viewport(
        Size::splat(Available::MAX_CONTENT),
        FlexItemRootContext::under_viewport(
            viewport,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        )
        .unwrap(),
    )
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("flex-item root layout succeeds");

    assert_eq!(
        batch.unrounded_entries()[0].output().size,
        Size::new(100.0, 20.0)
    );
    assert_eq!(batch.unrounded_entries()[0].output().padding, Edges::ZERO);
    assert_eq!(batch.unrounded_entries()[0].output().border, Edges::ZERO);
}

fn assert_logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow<
    S: LayoutScalar,
>() {
    #[derive(Default)]
    struct HiddenTree<S: LayoutScalar> {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutputOf<S>>,
        caches: HashMap<u32, CacheOf<S>>,
        styles: HashMap<u32, NodeInputOf<S>>,
        calls: Vec<(u32, ComputeInputOf<S>)>,
        cache_get_calls: Cell<usize>,
        cache_store_calls: usize,
    }

    impl<S: LayoutScalar> Traverse for HiddenTree<S> {
        type Node = u32;
        type Scalar = S;
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

    impl<S: LayoutScalar> Compute for HiddenTree<S> {
        fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInputOf<S>,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            let expected_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
            assert_eq!(
                input,
                ComputeInputOf::hidden(crate::ContainingLayoutContext::new(
                    expected_axes,
                    crate::ParentFormattingContext::NoParent
                ))
            );
            self.calls.push((node, input));
            compute_hidden(
                self,
                node,
                SourceIndex::ZERO,
                input.containing_layout_context(),
                input.containing_auto_scrollbar_pass(),
            )
        }
    }

    impl<S: LayoutScalar> CacheAccess for HiddenTree<S> {
        type Node = u32;
        type Scalar = S;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInputOf<S>,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutputOf<S>> {
            self.cache_get_calls.set(self.cache_get_calls.get() + 1);
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInputOf<S>,
            context: crate::CacheKeyContext,
            output: ComputeOutputOf<S>,
        ) {
            self.cache_store_calls += 1;
            self.caches
                .get_mut(&node)
                .expect("test hidden node cache exists")
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![3]);
    tree.children.insert(3, vec![]);
    for node in [1, 2, 3] {
        tree.styles.insert(node, NodeInputOf::default());
        tree.caches.insert(node, CacheOf::new());
        tree.caches.get_mut(&node).unwrap().store_with_context(
            &ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::splat(Some(scalar::<S>(1.0))),
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(AvailableOf::MAX_CONTENT),
            ),
            CacheKeyContext::new(),
            ComputeOutputOf::from_outer_size(Size::splat(scalar::<S>(1.0))),
        );
    }

    let expected_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let expected_input = ComputeInputOf::hidden(crate::ContainingLayoutContext::new(
        expected_axes,
        crate::ParentFormattingContext::NoParent,
    ));
    assert_eq!(
        compute_hidden(
            &mut tree,
            1,
            SourceIndex::ZERO,
            crate::ContainingLayoutContext::new(
                expected_axes,
                crate::ParentFormattingContext::Grid,
            ),
            crate::scroll::SettledAutoScrollbarState::INITIAL,
        )
        .unwrap(),
        ComputeOutputOf::HIDDEN
    );
    assert_eq!(tree.calls, vec![(2, expected_input), (3, expected_input)]);
    for node in [1, 2, 3] {
        assert_eq!(
            tree.layouts[&node],
            NodeOutputOf::with_source_index(crate::SourceIndex::new(0))
        );
        assert!(tree.caches[&node].is_empty());
    }
    assert_eq!(tree.cache_get_calls.get(), 0);
    assert_eq!(tree.cache_store_calls, 0);
}

#[test]
fn logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow_for_f32() {
    assert_logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow::<f32>();
}

#[test]
fn logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow_for_f64() {
    assert_logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow::<f64>();
}

fn assert_subgrid_orthogonal_local_cross_flow_does_not_expand_parent_intrinsic_axis<
    S: LayoutScalar,
>() {
    let scalar = S::from_f64;
    let outer_grid = NodeInputOf {
        display: Display::Grid,
        grid_template_columns: vec![
            TrackComponentOf::px(scalar(30.0)),
            TrackComponentOf::px(scalar(40.0)),
        ],
        grid_template_rows: vec![
            TrackComponentOf::px(scalar(50.0)),
            TrackComponentOf::px(scalar(60.0)),
        ],
        gap: Size::new(LengthOf::px(scalar(11.0)), LengthOf::px(scalar(7.0))),
        ..NodeInputOf::default()
    };
    let vertical_item = |column, row| NodeInputOf {
        display: Display::Flex,
        writing_mode: WritingMode::VerticalRl,
        grid_column: GridPlacement::try_lines(column, column + 1)
            .expect("valid orthogonal subgrid item column placement"),
        grid_row: GridPlacement::try_lines(row, row + 1)
            .expect("valid orthogonal subgrid item row placement"),
        ..NodeInputOf::default()
    };
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 4])
        .with_children(1, [2])
        .with_children(2, [3, 8])
        .with_children(3, [])
        .with_children(8, [])
        .with_children(4, [5])
        .with_children(5, [6, 7])
        .with_children(6, [])
        .with_children(7, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .with_style(1, outer_grid.clone())
        .with_style(
            2,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![
                    TrackComponentOf::px(scalar(50.0)),
                    TrackComponentOf::px(scalar(60.0)),
                ],
                gap: Size::new(LengthOf::px(scalar(7.0)), LengthOf::px(scalar(11.0))),
                grid_column: GridPlacement::try_lines(1, 3)
                    .expect("valid columns-subgrid column placement"),
                grid_row: GridPlacement::try_lines(1, 3)
                    .expect("valid columns-subgrid row placement"),
                ..NodeInputOf::default()
            },
        )
        .with_style(3, vertical_item(1, 1))
        .with_style(8, vertical_item(2, 2))
        .with_style(4, outer_grid)
        .with_style(
            5,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_template_columns: vec![
                    TrackComponentOf::px(scalar(30.0)),
                    TrackComponentOf::px(scalar(40.0)),
                ],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                gap: Size::new(LengthOf::px(scalar(7.0)), LengthOf::px(scalar(11.0))),
                grid_column: GridPlacement::try_lines(1, 3)
                    .expect("valid rows-subgrid column placement"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid rows-subgrid row placement"),
                ..NodeInputOf::default()
            },
        )
        .with_style(6, vertical_item(1, 1))
        .with_style(7, vertical_item(2, 2));

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("valid auto-sized root request"),
    )
    .expect("orthogonal subgrid layout succeeds");

    let root = public_flow_output(batch.unrounded_entries(), 0);
    let columns_outer = public_flow_output(batch.unrounded_entries(), 1);
    let columns_subgrid = public_flow_output(batch.unrounded_entries(), 2);
    let rows_outer = public_flow_output(batch.unrounded_entries(), 4);
    let rows_subgrid = public_flow_output(batch.unrounded_entries(), 5);

    assert_eq!(root.size, Size::new(scalar(81.0), scalar(234.0)));
    for output in [columns_outer, columns_subgrid, rows_outer, rows_subgrid] {
        assert_eq!(output.size, Size::new(scalar(81.0), scalar(117.0)));
    }
    assert_eq!(columns_outer.location, Point::new(S::ZERO, S::ZERO));
    assert_eq!(rows_outer.location, Point::new(S::ZERO, scalar(117.0)));

    for (node, location, size) in [
        (
            3,
            Point::new(scalar(31.0), S::ZERO),
            Size::new(scalar(50.0), scalar(48.0)),
        ),
        (
            8,
            Point::new(scalar(-36.0), scalar(59.0)),
            Size::new(scalar(60.0), scalar(58.0)),
        ),
        (
            6,
            Point::new(scalar(39.0), S::ZERO),
            Size::new(scalar(42.0), scalar(30.0)),
        ),
        (
            7,
            Point::new(S::ZERO, scalar(41.0)),
            Size::new(scalar(32.0), scalar(40.0)),
        ),
    ] {
        let output = public_flow_output(batch.unrounded_entries(), node);
        assert_eq!(output.location, location, "node {node} location");
        assert_eq!(output.size, size, "node {node} size");
    }
}

#[test]
fn subgrid_orthogonal_local_cross_flow_does_not_expand_parent_intrinsic_axis_f32() {
    assert_subgrid_orthogonal_local_cross_flow_does_not_expand_parent_intrinsic_axis::<f32>();
}

#[test]
fn subgrid_orthogonal_local_cross_flow_does_not_expand_parent_intrinsic_axis_f64() {
    assert_subgrid_orthogonal_local_cross_flow_does_not_expand_parent_intrinsic_axis::<f64>();
}

fn orthogonal_auto_child_grid<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
) -> NodeInputOf<S> {
    let scalar = scalar::<S>;
    let flow_axes = FlowAxes::new(writing_mode, direction);
    NodeInputOf {
        display: Display::Grid,
        writing_mode,
        direction,
        grid_template_columns: vec![
            TrackComponentOf::px(scalar(30.0)),
            TrackComponentOf::px(scalar(40.0)),
        ],
        grid_template_rows: vec![
            TrackComponentOf::px(scalar(50.0)),
            TrackComponentOf::px(scalar(60.0)),
        ],
        gap: flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            LengthOf::px(scalar(11.0)),
            LengthOf::px(scalar(7.0)),
        )),
        ..NodeInputOf::default()
    }
}

fn orthogonal_auto_child_subgrid<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Grid,
        writing_mode,
        direction,
        grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
        grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
        grid_column: GridPlacement::try_lines(1, -1).expect("valid full subgrid column span"),
        grid_row: GridPlacement::try_lines(1, -1).expect("valid full subgrid row span"),
        ..NodeInputOf::default()
    }
}

fn orthogonal_auto_child_subgrid_descendant<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        writing_mode,
        direction,
        grid_column: GridPlacement::try_lines(2, 3).expect("valid second subgrid column"),
        grid_row: GridPlacement::try_lines(2, 3).expect("valid second subgrid row"),
        ..NodeInputOf::default()
    }
}

fn orthogonal_auto_child_tree<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    root_height: PreferredSizeOf<S>,
) -> PublicFlowTree<S> {
    let outer_grid = orthogonal_auto_child_grid(writing_mode, direction);
    let subgrid = orthogonal_auto_child_subgrid(writing_mode, direction);
    let descendant = orthogonal_auto_child_subgrid_descendant(writing_mode, direction);

    PublicFlowTree::default()
        .with_children(0, [1, 4])
        .with_children(1, [2])
        .with_children(2, [3])
        .with_children(3, [])
        .with_children(4, [5])
        .with_children(5, [6])
        .with_children(6, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::AUTO, root_height),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, outer_grid.clone())
        .with_style(2, subgrid.clone())
        .with_style(3, descendant.clone())
        .with_style(4, outer_grid)
        .with_style(5, subgrid)
        .with_style(6, descendant)
}

fn assert_orthogonal_auto_child_inline_size_remains_indefinite<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_outer_size = crate::geometry::LogicalSizeOf::new(scalar(81.0), scalar(117.0));
    let logical_descendant_origin =
        crate::geometry::LogicalPointOf::new(scalar(41.0), scalar(57.0));
    let logical_descendant_size = crate::geometry::LogicalSizeOf::new(scalar(40.0), scalar(60.0));

    for writing_mode in [
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let outer_size = flow_axes.physical_size(logical_outer_size);
            let descendant_size = flow_axes.physical_size(logical_descendant_size);
            let descendant_location = flow_axes.physical_point(
                logical_descendant_origin,
                logical_descendant_size,
                outer_size,
            );
            let tree =
                orthogonal_auto_child_tree::<S>(writing_mode, direction, PreferredSizeOf::AUTO);
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
                    .expect("valid auto-sized root request"),
            )
            .expect("orthogonal auto child layout succeeds");

            let root = public_flow_output(batch.unrounded_entries(), 0);
            assert_eq!(root.size, Size::new(scalar(117.0), scalar(162.0)));

            for node in [1, 2, 4, 5] {
                assert_eq!(
                    public_flow_output(batch.unrounded_entries(), node).size,
                    outer_size,
                    "{writing_mode:?} {direction:?} node {node} must retain its intrinsic physical grid/subgrid size"
                );
            }
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), 1).location,
                Point::ZERO
            );
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), 4).location,
                Point::new(S::ZERO, scalar(81.0))
            );

            for node in [3, 6] {
                let descendant = public_flow_output(batch.unrounded_entries(), node);
                assert_eq!(
                    descendant.location, descendant_location,
                    "{writing_mode:?} {direction:?} node {node} must use the inherited subgrid area"
                );
                assert_eq!(
                    descendant.size, descendant_size,
                    "{writing_mode:?} {direction:?} node {node} must use the inherited subgrid track size"
                );
            }
        }
    }
}

#[test]
fn orthogonal_auto_child_inline_size_remains_indefinite_f32() {
    assert_orthogonal_auto_child_inline_size_remains_indefinite::<f32>();
}

#[test]
fn orthogonal_auto_child_inline_size_remains_indefinite_f64() {
    assert_orthogonal_auto_child_inline_size_remains_indefinite::<f64>();
}

fn assert_orthogonal_child_fixed_parent_height_remains_definite<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    for writing_mode in [
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let tree = orthogonal_auto_child_tree::<S>(
                writing_mode,
                direction,
                PreferredSizeOf::px(scalar(162.0)),
            );
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
                    .expect("valid fixed-height root request"),
            )
            .expect("fixed-height orthogonal child layout succeeds");

            assert_eq!(
                public_flow_output(batch.unrounded_entries(), 0).size,
                Size::new(scalar(117.0), scalar(162.0))
            );
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), 1).size,
                Size::new(scalar(117.0), scalar(162.0)),
                "{writing_mode:?} {direction:?} must retain the fixed parent height"
            );
        }
    }
}

#[test]
fn orthogonal_child_fixed_parent_height_remains_definite_f32() {
    assert_orthogonal_child_fixed_parent_height_remains_definite::<f32>();
}

#[test]
fn orthogonal_child_fixed_parent_height_remains_definite_f64() {
    assert_orthogonal_child_fixed_parent_height_remains_definite::<f64>();
}

#[test]
fn fri05_c03_leaf_geometry_tree_backed_emits_flow_clip_and_target_geometry() {
    for flow_axes in fri05_c03_root_all_flow_axes() {
        let overflow = match flow_axes.block_axis() {
            PhysicalAxis::Horizontal => computed_overflow(Overflow::Scroll, Overflow::Hidden),
            PhysicalAxis::Vertical => computed_overflow(Overflow::Hidden, Overflow::Scroll),
        };
        let style = NodeInput {
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_width: ScrollbarWidth::try_new(7.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        };
        let expected_content_size = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(93.0, 80.0),
            PhysicalAxis::Vertical => Size::new(100.0, 73.0),
        };
        let (output, inputs) =
            fri05_c03_tree_leaf_layout(style, Size::new(20.0, 10.0), Size::new(100.0, 80.0));
        assert_eq!(inputs, [expected_content_size], "{flow_axes:?}");
        let geometry = output
            .scroll_geometry
            .expect("tree-backed performed leaf emits geometry");
        assert_eq!(geometry.flow_axes(), flow_axes);
        assert_eq!(geometry.content_box().size(), expected_content_size);
        assert_eq!(output.content_size, Size::new(100.0, 80.0), "{flow_axes:?}");
        assert_eq!(
            output.content_size,
            geometry.scrollable_overflow().size(),
            "{flow_axes:?}"
        );
        assert!(
            fri05_c03_root_gutter_at(geometry.gutters(), flow_axes.inline_end()).is_some(),
            "missing inline-end gutter for {flow_axes:?}"
        );
    }

    let scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let (output, inputs) = fri05_c03_tree_leaf_layout(
        NodeInput {
            writing_mode: WritingMode::VerticalRl,
            direction: Direction::Rtl,
            overflow: computed_overflow(Overflow::Visible, Overflow::Clip),
            overflow_clip_margin: OverflowClipMargin::try_new(OverflowClipBox::BorderBox, 3.0)
                .unwrap(),
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(30.0)),
            scroll_margin,
            scroll_snap_type: ScrollSnapType::Enabled {
                axis: ScrollSnapAxis::Both,
                strictness: ScrollSnapStrictness::Mandatory,
            },
            scroll_snap_align: snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInput::default()
        },
        Size::new(60.0, 50.0),
        Size::new(40.0, 30.0),
    );
    assert_eq!(inputs, [Size::new(40.0, 30.0)]);
    let geometry = output
        .scroll_geometry
        .expect("tree leaf geometry is present");
    assert_eq!(geometry.overflow_clip().x(), None);
    let y_clip = geometry.overflow_clip().y().expect("y clip is present");
    assert_eq!((y_clip.minimum(), y_clip.maximum()), (-3.0, 33.0));
    let target = geometry.target();
    assert_eq!(target.scroll_margin(), scroll_margin);
    assert_eq!(target.snap_align(), snap_align);
    assert_eq!(target.snap_stop(), ScrollSnapStop::Always);
    assert_eq!(
        target.flow_axes(),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl)
    );
}

#[test]
fn fri05_c04_flex_child_geometry_tree_retains_in_flow_and_absolute_targets() {
    let parent_size = Size::new(140.0, 90.0);
    let in_flow_axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl);
    let absolute_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let in_flow_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let absolute_margin = ScrollMargin::try_new(-5.0, 6.0, -7.0, 8.0).unwrap();
    let in_flow_align =
        ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::End);
    let absolute_align =
        ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
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
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                writing_mode: in_flow_axes.writing_mode(),
                direction: in_flow_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
                scrollbar_gutter: ScrollbarGutter::Stable,
                scrollbar_width: ScrollbarWidth::try_new(4.0).unwrap(),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(22.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                scroll_margin: in_flow_margin,
                scroll_snap_align: in_flow_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInput::default()
            },
        )
        .with_style(
            2,
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                writing_mode: absolute_axes.writing_mode(),
                direction: absolute_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
                scrollbar_width: ScrollbarWidth::try_new(3.0).unwrap(),
                size: Size::new(PreferredSize::px(28.0), PreferredSize::px(18.0)),
                inset: Edges::new(
                    LengthAuto::px(4.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(6.0),
                ),
                scroll_margin: absolute_margin,
                scroll_snap_align: absolute_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(parent_size.map(Available::definite)).unwrap(),
    )
    .expect("tree-backed flex child geometry layout succeeds");

    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("rounded", batch.final_entries()),
    ] {
        for (node, expected_axes, expected_margin, expected_align) in [
            (1, in_flow_axes, in_flow_margin, in_flow_align),
            (2, absolute_axes, absolute_margin, absolute_align),
        ] {
            let output = public_flow_output(entries, node);
            let geometry = output
                .scroll_geometry
                .unwrap_or_else(|| panic!("{phase} flex child {node} retains canonical geometry"));
            assert_eq!(geometry.border_box().size(), output.size, "{phase}/{node}");
            assert_eq!(
                geometry.target().border_box(),
                geometry.border_box(),
                "{phase}/{node}"
            );
            assert_eq!(
                geometry.target().flow_axes(),
                expected_axes,
                "{phase}/{node}"
            );
            assert_eq!(
                geometry.target().scroll_margin(),
                expected_margin,
                "{phase}/{node}"
            );
            assert_eq!(
                geometry.target().snap_align(),
                expected_align,
                "{phase}/{node}"
            );
            assert_eq!(
                geometry.target().snap_stop(),
                ScrollSnapStop::Always,
                "{phase}/{node}"
            );
            assert_eq!(output.scrollbar_size(), geometry.scrollbar_size());
        }
    }
}

fn fri05_c04_hidden_auto_tree(display: Display) -> PublicFlowTree<f32> {
    PublicFlowTree::default()
        .with_children(0, [1, 4])
        .with_children(1, [2])
        .with_children(2, [3])
        .with_children(3, [])
        .with_children(4, [])
        .with_style(
            0,
            NodeInput {
                display,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                size: Size::splat_clone(PreferredSize::px(100.0)),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::None,
                ..NodeInput::default()
            },
        )
        .with_style(2, NodeInput::default())
        .with_style(3, NodeInput::default())
        .with_style(
            4,
            NodeInput {
                position: Position::Absolute,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
                inset: Edges::new(
                    LengthAuto::px(0.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(0.0),
                ),
                ..NodeInput::default()
            },
        )
}

#[test]
fn fri05_c04_flex_auto_hidden_subtrees_retain_immediate_containing_pass() {
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();

    for display in [Display::Flex, Display::Block] {
        let tree = fri05_c04_hidden_auto_tree(display);
        let (batch, hidden_requests) = crate::engine::trace_hidden_compute_session_requests(|| {
            compute_layout(&tree, 0, request).expect("auto container with hidden subtree lays out")
        });
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 0)
                .scroll_geometry
                .unwrap()
                .scrollbar_size(),
            Size::new(0.0, 15.0),
            "{display:?} must transition its horizontal auto pass"
        );
        assert_eq!(
            hidden_requests.len(),
            6,
            "{display:?} visits all three hidden nodes in both containing passes"
        );
        assert!(
            hidden_requests
                .iter()
                .all(|(local, _)| *local == crate::scroll::SettledAutoScrollbarState::INITIAL),
            "{display:?} hidden nodes keep child-local settlement INITIAL: {hidden_requests:#?}"
        );
        let containing_states = hidden_requests
            .iter()
            .map(|(_, state)| {
                (
                    state.at(PhysicalAxis::Horizontal),
                    state.at(PhysicalAxis::Vertical),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            containing_states,
            [
                (false, false),
                (false, false),
                (false, false),
                (true, false),
                (true, false),
                (true, false),
            ],
            "{display:?} direct and recursive hidden nodes retain each immediate containing pass"
        );
    }
}

#[test]
fn fri05_c03_block_nested_partial_axes_and_trapped_values_preserve_independent_intervals() {
    for (overflow, nested_size, expected) in [
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
            computed_overflow(Overflow::Hidden, Overflow::Hidden),
            Size::ZERO,
            Size::ZERO,
        ),
        (
            computed_overflow(Overflow::Scroll, Overflow::Scroll),
            Size::ZERO,
            Size::ZERO,
        ),
        (
            computed_overflow(Overflow::Auto, Overflow::Auto),
            Size::ZERO,
            Size::ZERO,
        ),
    ] {
        let tree = RootSessionTree::<&'static str>::default()
            .children(0, [1])
            .children(1, [2])
            .children(2, [])
            .style(
                0,
                NodeInput {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(PreferredSize::px(0.0), PreferredSize::px(0.0)),
                    ..NodeInput::default()
                },
            )
            .style(
                1,
                NodeInput {
                    display: Display::Block,
                    position: Position::Absolute,
                    overflow,
                    size: nested_size.map(PreferredSize::px),
                    ..NodeInput::default()
                },
            )
            .style(
                2,
                NodeInput {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    ..NodeInput::default()
                },
            )
            .measure(2, Ok(Size::new(20.0, 30.0)));
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap(),
        )
        .expect("nested block contribution layout succeeds");
        let output = |node| {
            batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .expect("nested block output is staged")
                .output()
        };

        let nested = output(1);
        assert_eq!(nested.content_size, Size::new(20.0, 30.0));
        assert_eq!(
            nested.scroll_geometry.unwrap().scrollable_overflow().size(),
            Size::new(20.0, 30.0)
        );

        let root = output(0);
        let geometry = root
            .scroll_geometry
            .expect("root block geometry is present");
        assert_eq!(geometry.scrollable_overflow().origin(), Point::ZERO);
        assert_eq!(geometry.scrollable_overflow().size(), expected);
        assert_eq!(root.content_size, expected);
    }
}
