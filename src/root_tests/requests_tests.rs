use super::fixtures::{
    FlowRootLeafTree, PublicFlowTree, RootSessionTree, RootTestScrollGeometryFacts,
    assert_fri06_c08_float_line_final_height, assert_fri06_c08_mixed_inline_atomic_x,
    assert_fri06_c08_r1_mixed_unit_traversal, assert_positive_physical_range,
    assert_public_scroll_geometry_error_without_batch, computed_overflow,
    fri05_c03_block_root_state, fri05_c04_assert_initial_local_auto_state,
    fri05_c04_local_auto_state, fri05_c04_nested_flex_auto_tree, fri06_atomic_participation,
    fri06_c02_final_node, fri06_c02_segment, fri06_c02_segment_with_level,
    fri06_c02_segment_with_metrics, fri06_c02_text_batch, fri06_c02_text_nodes_batch,
    fri06_c03_atomic_participation, fri06_c03_atomic_style, fri06_c03_mixed_batch_with_root,
    fri06_c03_text_input, fri06_c04_bfc_batch, fri06_c04_front_door_batch, fri06_c04_line_batch,
    fri06_c04_line_box, fri06_c04_logical_origin, fri06_c12_t08_forced_break_fallback_batch,
    fri06_mr02_geometry_error_largest_finite, public_flow_output, public_layout_tree,
    root_test_scroll_geometry, root_writing_mode_directions, scalar, single_final_output,
};
use super::*;

fn assert_fri08_c07_t02_scroll_source_root_paths<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let size = Size::new(scalar(90.0), scalar(60.0));
    let flow_axes = FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl);
    let scroll_margin =
        ScrollMarginOf::try_new(scalar(1.0), scalar(-2.0), scalar(3.0), scalar(-4.0)).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::End);
    let style = NodeInputOf {
        display: Display::Block,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        overflow_clip_margin: OverflowClipMarginOf::try_new(
            OverflowClipBox::BorderBox,
            scalar(2.0),
        )
        .unwrap(),
        scrollbar_width: ScrollbarWidthOf::try_new(scalar(5.0)).unwrap(),
        size: size.map(PreferredSizeOf::px),
        border: Edges::all(LengthOf::px(scalar(2.0))),
        padding: Edges::all(LengthOf::px(scalar(3.0))),
        scroll_padding: ScrollPaddingOf::new(
            ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(1.0)).unwrap()),
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(4.0)).unwrap()),
            ScrollPaddingValueOf::AUTO,
        ),
        scroll_margin,
        scroll_snap_type: ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Block,
            strictness: ScrollSnapStrictness::Mandatory,
        },
        scroll_snap_align: snap_align,
        scroll_snap_stop: ScrollSnapStop::Always,
        ..NodeInputOf::default()
    };

    let mut reconstructed_tree = OracleTreeOf::<S>::new()
        .children(7, [])
        .style(7, style.clone())
        .measure(
            7,
            ComputeOutputOf::from_sizes(size, Size::new(scalar(120.0), scalar(95.0))),
        );
    compute_root(&mut reconstructed_tree, 7, size.map(AvailableOf::definite)).unwrap();
    let reconstructed = reconstructed_tree
        .output(7)
        .and_then(|output| output.scroll_geometry)
        .unwrap();
    assert_eq!(reconstructed.flow_axes(), flow_axes);
    assert_eq!(reconstructed.border_box().size(), size);
    assert_eq!(
        reconstructed.target().border_box(),
        reconstructed.border_box()
    );
    assert_eq!(reconstructed.target().scroll_margin(), scroll_margin);
    assert_eq!(reconstructed.target().flow_axes(), flow_axes);
    assert_eq!(reconstructed.target().snap_align(), snap_align);
    assert_eq!(reconstructed.target().snap_stop(), ScrollSnapStop::Always);
    assert!(reconstructed.overflow_clip().x().is_some());
    assert!(reconstructed.overflow_clip().y().is_some());
    assert_eq!(reconstructed.resolved_scroll_padding().top, scalar(1.0));
    assert_eq!(reconstructed.resolved_scroll_padding().bottom, scalar(4.0));
    assert_eq!(
        reconstructed.scroll_snap_type(),
        ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Block,
            strictness: ScrollSnapStrictness::Mandatory,
        }
    );

    let existing = root_test_scroll_geometry(RootTestScrollGeometryFacts {
        flow_axes,
        overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: size,
        padding: Edges::all(scalar(3.0)),
        border: Edges::all(scalar(2.0)),
        scrollbar_width: scalar(5.0),
        scrollable_overflow: ScrollRectOf::try_new(
            Point::new(scalar(-7.0), scalar(-3.0)),
            Size::new(scalar(130.0), scalar(100.0)),
        )
        .unwrap(),
    });
    let mut child_output =
        ComputeOutputOf::from_sizes(size, Size::new(scalar(120.0), scalar(95.0)));
    child_output.scroll_geometry = Some(existing);
    let mut existing_tree = OracleTreeOf::<S>::new()
        .children(7, [])
        .style(7, style)
        .measure(7, child_output);
    compute_root(&mut existing_tree, 7, size.map(AvailableOf::definite)).unwrap();
    assert_eq!(
        existing_tree
            .output(7)
            .and_then(|output| output.scroll_geometry),
        Some(existing),
        "matching existing geometry remains byte-for-byte canonical"
    );
}

#[test]
fn fri08_c07_t02_scroll_source_root_preserves_existing_reconstruction_and_metadata() {
    assert_fri08_c07_t02_scroll_source_root_paths::<f32>();
    assert_fri08_c07_t02_scroll_source_root_paths::<f64>();
}

fn fri06_c03_mixed_batch<S: LayoutScalar>(
    children: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    available_inline: AvailableOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    fri06_c03_mixed_batch_with_root(
        children,
        available_inline,
        NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        },
    )
}

fn fri06_c03_logical_block_start<S: LayoutScalar>(
    flow_axes: FlowAxes,
    output: NodeOutputOf<S>,
    container_size: Size<S>,
) -> S {
    flow_axes
        .logical_point(output.location, output.size, container_size)
        .block
}

fn fri06_c03_fragment<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> InlineFragmentOutputOf<S> {
    batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.node() == node)
        .expect("the requested shaped source publishes one fragment")
        .fragment()
}

fn fri06_c03_nested_atomic_baseline_batch<S: LayoutScalar>(
    parent_flow: FlowAxes,
    child_flow: FlowAxes,
    overflow: Overflow,
    item_is_replaced: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let root_style = NodeInputOf {
        display: Display::Block,
        writing_mode: parent_flow.writing_mode(),
        direction: parent_flow.direction(),
        ..NodeInputOf::default()
    };
    let atomic_size =
        parent_flow.physical_size(LogicalSizeOf::new(S::from_f64(10.0), S::from_f64(20.0)));
    let atomic_margin = parent_flow.physical_edges(
        crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::from_f64(1.0), S::from_f64(2.0))
            .map(LengthAutoOf::px),
    );
    let atomic_style = NodeInputOf {
        display: Display::InlineBlock,
        writing_mode: child_flow.writing_mode(),
        direction: child_flow.direction(),
        size: atomic_size.map(PreferredSizeOf::px),
        margin: atomic_margin,
        overflow: ComputedOverflow::try_new(overflow, overflow).unwrap(),
        item_is_replaced,
        atomic_inline_participation: Some(fri06_c03_atomic_participation(
            0,
            InlineBreakOpportunityOf::prohibited(),
        )),
        ..NodeInputOf::default()
    };
    let parent_text =
        InlineTextInputOf::try_new(vec![fri06_c02_segment_with_metrics(700, 10.0, 8.0, 2.0)])
            .unwrap();
    let first_inner_text =
        InlineTextInputOf::try_new(vec![fri06_c02_segment_with_metrics(701, 5.0, 4.0, 6.0)])
            .unwrap();
    let last_inner_text =
        InlineTextInputOf::try_new(vec![fri06_c02_segment_with_metrics(702, 5.0, 7.0, 3.0)])
            .unwrap();
    let zero_metrics = InlineMetricsOf::from_ascent_descent(S::ZERO, S::ZERO).unwrap();
    let inner_break = LineBreakInputOf::new()
        .with_writing_mode(child_flow.writing_mode())
        .with_direction(child_flow.direction())
        .with_metrics(zero_metrics);
    let tree = public_layout_tree(
        HashMap::from([
            (0, LayoutInputOf::box_input(root_style.clone())),
            (1, LayoutInputOf::inline_text(parent_text)),
            (2, LayoutInputOf::box_input(atomic_style.clone())),
            (3, LayoutInputOf::inline_text(first_inner_text)),
            (4, LayoutInputOf::line_break(inner_break)),
            (5, LayoutInputOf::inline_text(last_inner_text)),
        ]),
        HashMap::from([
            (0, vec![1, 2]),
            (1, Vec::new()),
            (2, vec![3, 4, 5]),
            (3, Vec::new()),
            (4, Vec::new()),
            (5, Vec::new()),
        ]),
    );
    let viewport = parent_flow.physical_size(LogicalSizeOf::new(
        AvailableOf::definite(S::from_f64(100.0)),
        AvailableOf::MAX_CONTENT,
    ));

    compute_layout(&tree, 0, LayoutRootRequestOf::viewport(viewport).unwrap()).unwrap()
}

#[derive(Clone)]
struct Fri06C03CachedAtomicTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    atomic_output: ComputeOutputOf<S>,
}

impl<S: LayoutScalar> Traverse for Fri06C03CachedAtomicTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

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

impl<S: LayoutScalar> LayoutTree for Fri06C03CachedAtomicTree<S> {
    type MeasureError = ();

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn cache_get(
        &self,
        node: Self::Node,
        _input: &ComputeInputOf<S>,
        _context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        (node == 2).then_some(self.atomic_output)
    }
}

#[test]
fn fri06_c03_atomic_baseline_visible_inner_and_non_visible_margin_edge_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let parallel = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let opposing_parent = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
        let opposing_child = FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);

        for (parent_flow, child_flow, atomic_block_start, baseline, block_extent) in [
            (parallel, parallel, 4.0, 8.0, 26.0),
            (opposing_parent, opposing_child, 1.0, 17.0, 23.0),
        ] {
            let batch = fri06_c03_nested_atomic_baseline_batch::<S>(
                parent_flow,
                child_flow,
                Overflow::Visible,
                false,
            );
            let root = fri06_c02_final_node(&batch, 0);
            let atomic = fri06_c02_final_node(&batch, 2);
            let parent_fragment = fri06_c03_fragment(&batch, 1);
            assert_eq!(
                fri06_c03_logical_block_start(parent_flow, atomic, root.size),
                S::from_f64(atomic_block_start),
                "visible inner first baseline in {parent_flow:?}/{child_flow:?}"
            );
            assert_eq!(
                parent_flow
                    .logical_point(parent_fragment.baseline(), Size::ZERO, root.size)
                    .block,
                S::from_f64(baseline),
                "container baseline in {parent_flow:?}/{child_flow:?}"
            );
            assert_eq!(
                parent_flow.logical_size(root.size).block,
                S::from_f64(block_extent),
                "visible inner descent in {parent_flow:?}/{child_flow:?}"
            );
        }

        for overflow in [
            Overflow::Clip,
            Overflow::Hidden,
            Overflow::Scroll,
            Overflow::Auto,
        ] {
            let batch =
                fri06_c03_nested_atomic_baseline_batch::<S>(parallel, parallel, overflow, false);
            let root = fri06_c02_final_node(&batch, 0);
            let atomic = fri06_c02_final_node(&batch, 2);
            let parent_fragment = fri06_c03_fragment(&batch, 1);
            assert_eq!(
                fri06_c03_logical_block_start(parallel, atomic, root.size),
                S::from_f64(1.0),
                "{overflow:?} falls back to the block-end margin edge"
            );
            assert_eq!(parent_fragment.baseline().y, S::from_f64(23.0));
            assert_eq!(root.size.height, S::from_f64(25.0));
        }

        let replaced_hidden =
            fri06_c03_nested_atomic_baseline_batch::<S>(parallel, parallel, Overflow::Hidden, true);
        let root = fri06_c02_final_node(&replaced_hidden, 0);
        assert_eq!(
            fri06_c03_fragment(&replaced_hidden, 1).baseline().y,
            S::from_f64(23.0)
        );
        assert_eq!(root.size.height, S::from_f64(25.0));

        let atomic_style = NodeInputOf {
            display: Display::InlineBlock,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(10.0)),
                PreferredSizeOf::px(S::from_f64(20.0)),
            ),
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(1.0)),
                bottom: LengthAutoOf::px(S::from_f64(2.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            atomic_inline_participation: Some(fri06_c03_atomic_participation(
                0,
                InlineBreakOpportunityOf::prohibited(),
            )),
            ..NodeInputOf::default()
        };
        let root_style = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let tree = Fri06C03CachedAtomicTree {
            tree: public_layout_tree(
                HashMap::from([
                    (0, LayoutInputOf::box_input(root_style.clone())),
                    (
                        1,
                        fri06_c03_text_input(vec![fri06_c02_segment_with_metrics(
                            703, 10.0, 8.0, 2.0,
                        )]),
                    ),
                    (2, LayoutInputOf::box_input(atomic_style.clone())),
                ]),
                HashMap::from([(0, vec![1, 2]), (1, Vec::new()), (2, Vec::new())]),
            ),
            atomic_output: ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(10.0), S::from_f64(20.0)),
                Size::new(S::from_f64(10.0), S::from_f64(20.0)),
                BaselinesOf::from_block_coordinates(parallel, None, Some(S::from_f64(6.0))),
            ),
        };
        let last_only = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(100.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .unwrap();
        let root = fri06_c02_final_node(&last_only, 0);
        assert_eq!(
            fri06_c02_final_node(&last_only, 2).location.y,
            S::from_f64(2.0)
        );
        assert_eq!(
            fri06_c03_fragment(&last_only, 1).baseline().y,
            S::from_f64(8.0)
        );
        assert_eq!(root.size.height, S::from_f64(24.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_atomic_baseline_absent_inner_uses_positive_and_negative_margin_edges_once() {
    fn assert_lane<S: LayoutScalar>() {
        let text = fri06_c03_text_input(vec![fri06_c02_segment_with_metrics(710, 10.0, 8.0, 2.0)]);
        let positive = NodeInputOf {
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(3.0)),
                bottom: LengthAutoOf::px(S::from_f64(5.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..fri06_c03_atomic_style(
                10.0,
                10.0,
                0.0,
                0.0,
                0,
                InlineBreakOpportunityOf::prohibited(),
            )
        };
        let negative = NodeInputOf {
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(-2.0)),
                bottom: LengthAutoOf::px(S::from_f64(-3.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..fri06_c03_atomic_style(
                10.0,
                10.0,
                0.0,
                0.0,
                0,
                InlineBreakOpportunityOf::prohibited(),
            )
        };
        let batch = fri06_c03_mixed_batch(
            vec![
                (1, text, NodeInputOf::non_box()),
                (2, LayoutInputOf::box_input(positive.clone()), positive),
                (3, LayoutInputOf::box_input(negative.clone()), negative),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
        );
        let root = fri06_c02_final_node(&batch, 0);
        assert_eq!(
            fri06_c03_fragment(&batch, 1).baseline().y,
            S::from_f64(18.0)
        );
        assert_eq!(fri06_c02_final_node(&batch, 2).location.y, S::from_f64(3.0));
        assert_eq!(
            fri06_c02_final_node(&batch, 3).location.y,
            S::from_f64(11.0)
        );
        assert_eq!(root.size.height, S::from_f64(20.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_top_bottom_mixed_atomics_and_metric_controls_expand_one_line_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let text = fri06_c03_text_input(vec![fri06_c02_segment_with_metrics(720, 10.0, 8.0, 2.0)]);
        let top_with_margins = NodeInputOf {
            vertical_align: VerticalAlign::Top,
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(2.0)),
                bottom: LengthAutoOf::px(S::from_f64(3.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..fri06_c03_atomic_style(
                10.0,
                20.0,
                0.0,
                0.0,
                0,
                InlineBreakOpportunityOf::prohibited(),
            )
        };
        let top = NodeInputOf {
            vertical_align: VerticalAlign::Top,
            ..fri06_c03_atomic_style(
                10.0,
                25.0,
                0.0,
                0.0,
                0,
                InlineBreakOpportunityOf::prohibited(),
            )
        };
        let bottom_with_margins = NodeInputOf {
            vertical_align: VerticalAlign::Bottom,
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(1.0)),
                bottom: LengthAutoOf::px(S::from_f64(4.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..fri06_c03_atomic_style(
                10.0,
                30.0,
                0.0,
                0.0,
                0,
                InlineBreakOpportunityOf::prohibited(),
            )
        };
        let bottom = NodeInputOf {
            vertical_align: VerticalAlign::Bottom,
            ..fri06_c03_atomic_style(
                10.0,
                35.0,
                0.0,
                0.0,
                0,
                InlineBreakOpportunityOf::prohibited(),
            )
        };
        let top_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(22.0), S::from_f64(6.0))
                .unwrap();
        let bottom_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(32.0), S::from_f64(20.0))
                .unwrap();
        let break_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(10.0), S::from_f64(8.0))
                .unwrap();
        let batch = fri06_c03_mixed_batch(
            vec![
                (1, text, NodeInputOf::non_box()),
                (
                    2,
                    LayoutInputOf::box_input(top_with_margins.clone()),
                    top_with_margins,
                ),
                (3, LayoutInputOf::box_input(top.clone()), top),
                (
                    4,
                    LayoutInputOf::box_input(bottom_with_margins.clone()),
                    bottom_with_margins,
                ),
                (5, LayoutInputOf::box_input(bottom.clone()), bottom),
                (
                    6,
                    LayoutInputOf::inline_boundary(
                        InlineBoundaryInputOf::new(InlineBoundaryKind::Start, top_metrics)
                            .with_vertical_align(VerticalAlign::Top),
                    ),
                    NodeInputOf::non_box(),
                ),
                (
                    7,
                    LayoutInputOf::inline_boundary(
                        InlineBoundaryInputOf::new(InlineBoundaryKind::End, bottom_metrics)
                            .with_vertical_align(VerticalAlign::Bottom),
                    ),
                    NodeInputOf::non_box(),
                ),
                (
                    8,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(break_metrics)),
                    NodeInputOf::non_box(),
                ),
                (
                    9,
                    fri06_c03_text_input(vec![fri06_c02_segment_with_metrics(721, 10.0, 4.0, 6.0)]),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
        );

        let root = fri06_c02_final_node(&batch, 0);
        assert_eq!(root.size.height, S::from_f64(49.0));
        assert_eq!(
            fri06_c03_fragment(&batch, 1).baseline().y,
            S::from_f64(18.0)
        );
        assert_eq!(
            fri06_c03_fragment(&batch, 9).baseline().y,
            S::from_f64(43.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 2).location.y,
            S::from_f64(-2.0)
        );
        assert_eq!(fri06_c02_final_node(&batch, 3).location.y, S::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 4).location.y, S::from_f64(1.0));
        assert_eq!(fri06_c02_final_node(&batch, 5).location.y, S::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 6).location.y, S::from_f64(6.0));
        assert_eq!(
            fri06_c02_final_node(&batch, 7).location.y,
            S::from_f64(23.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 8).location.y,
            S::from_f64(18.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_control_mixed_break_boundary_and_hidden_output_publish_from_one_source_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let boundary_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();
        let break_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(12.0), S::from_f64(9.0))
                .unwrap();
        let atomic_style = fri06_c03_atomic_style(
            10.0,
            10.0,
            0.0,
            0.0,
            2,
            InlineBreakOpportunityOf::prohibited(),
        );
        let batch = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    fri06_c03_text_input(vec![fri06_c02_segment_with_level(
                        401,
                        10.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
                (
                    2,
                    LayoutInputOf::box_input(atomic_style.clone()),
                    atomic_style,
                ),
                (
                    3,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::Start,
                        boundary_metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
                (
                    4,
                    fri06_c03_text_input(vec![fri06_c02_segment_with_level(
                        402,
                        10.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
                (
                    5,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(break_metrics)),
                    NodeInputOf::non_box(),
                ),
                (
                    6,
                    LayoutInputOf::line_break(LineBreakInputOf::new().hidden()),
                    NodeInputOf::non_box(),
                ),
                (
                    7,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        403,
                        5.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::definite(S::from_f64(40.0)),
        );

        assert_eq!(
            fri06_c02_final_node(&batch, 0).size.height,
            S::from_f64(32.0)
        );
        assert_eq!(fri06_c02_final_node(&batch, 3).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 5).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 6).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 6).location, Point::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 3).source_index.get(), 2);
        assert_eq!(fri06_c02_final_node(&batch, 5).source_index.get(), 4);
        assert_eq!(fri06_c02_final_node(&batch, 6).source_index.get(), 5);
        assert_eq!(batch.final_inline_fragments().len(), 3);
        assert_eq!(
            batch
                .final_inline_fragments()
                .iter()
                .map(|entry| (
                    entry.fragment().line_index(),
                    entry.fragment().visual_index()
                ))
                .collect::<Vec<_>>(),
            [(0, 1), (0, 3), (1, 0)]
        );
        assert_eq!(fri06_c02_final_node(&batch, 2).location.x, S::ZERO);
        assert_eq!(
            fri06_c02_final_node(&batch, 3).location.x,
            S::from_f64(20.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 5).location.x,
            S::from_f64(30.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_control_forced_break_splits_min_content_indivisible_groups_both_scalars() {
    fn atomic<S: LayoutScalar>(
        node: u32,
        extent: f64,
        following_break: InlineBreakOpportunityOf<S>,
    ) -> (u32, LayoutInputOf<S>, NodeInputOf<S>) {
        let style = fri06_c03_atomic_style(extent, 10.0, 0.0, 0.0, 0, following_break);
        (node, LayoutInputOf::box_input(style.clone()), style)
    }

    fn children<S: LayoutScalar>() -> Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)> {
        vec![
            atomic(1, 12.0, InlineBreakOpportunityOf::prohibited()),
            atomic(2, 18.0, InlineBreakOpportunityOf::prohibited()),
            (
                3,
                LayoutInputOf::line_break(
                    LineBreakInputOf::new().with_metrics(
                        InlineMetricsOf::from_line_height_and_baseline(
                            S::from_f64(10.0),
                            S::from_f64(8.0),
                        )
                        .unwrap(),
                    ),
                ),
                NodeInputOf::non_box(),
            ),
            atomic(4, 17.0, InlineBreakOpportunityOf::prohibited()),
            atomic(5, 23.0, InlineBreakOpportunityOf::allowed()),
            atomic(6, 25.0, InlineBreakOpportunityOf::prohibited()),
        ]
    }

    fn lane<S: LayoutScalar>() -> (S, S, S, S, S) {
        let min = fri06_c03_mixed_batch(children(), AvailableOf::MIN_CONTENT);
        let max = fri06_c03_mixed_batch(children(), AvailableOf::MAX_CONTENT);

        (
            fri06_c02_final_node(&min, 0).size.width,
            fri06_c02_final_node(&max, 0).size.width,
            fri06_c02_final_node(&min, 3).location.x,
            fri06_c02_final_node(&min, 4).location.x,
            fri06_c02_final_node(&min, 6).location.x,
        )
    }

    let f32_lane = lane::<f32>();
    let f64_lane = lane::<f64>();
    assert_eq!(
        [
            f64::from(f32_lane.0),
            f64::from(f32_lane.1),
            f64::from(f32_lane.2),
            f64::from(f32_lane.3),
            f64::from(f32_lane.4),
            f64_lane.0,
            f64_lane.1,
            f64_lane.2,
            f64_lane.3,
            f64_lane.4,
        ],
        [40.0, 65.0, 30.0, 0.0, 0.0, 40.0, 65.0, 30.0, 0.0, 0.0]
    );
}

#[test]
fn fri06_c03_strut_adjacent_final_breaks_preserve_empty_following_line_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();
        let batch = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
                (
                    2,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::MAX_CONTENT,
        );

        assert_eq!(
            fri06_c02_final_node(&batch, 0).size.height,
            S::from_f64(60.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 1).location.y,
            S::from_f64(15.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 2).location.y,
            S::from_f64(35.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_control_leading_trailing_only_child_and_boundary_edges_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();
        let text = || {
            (
                2,
                fri06_c03_text_input(vec![fri06_c02_segment(
                    451,
                    10.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                )]),
                NodeInputOf::non_box(),
            )
        };

        let leading = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
                text(),
            ],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&leading, 0).size.height,
            S::from_f64(40.0)
        );
        assert_eq!(
            fri06_c02_final_node(&leading, 1).location.y,
            S::from_f64(15.0)
        );

        let trailing = fri06_c03_mixed_batch(
            vec![
                text(),
                (
                    3,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&trailing, 0).size.height,
            S::from_f64(40.0)
        );
        assert_eq!(
            fri06_c02_final_node(&trailing, 3).location,
            Point::new(S::from_f64(10.0), S::from_f64(15.0))
        );

        let only_break = fri06_c03_mixed_batch(
            vec![(
                1,
                LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                NodeInputOf::non_box(),
            )],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&only_break, 0).size.height,
            S::from_f64(40.0)
        );

        let only_boundary = fri06_c03_mixed_batch(
            vec![(
                1,
                LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                    InlineBoundaryKind::Start,
                    metrics,
                )),
                NodeInputOf::non_box(),
            )],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&only_boundary, 0).size.height,
            S::from_f64(20.0)
        );
        assert_eq!(
            fri06_c02_final_node(&only_boundary, 1).location,
            Point::new(S::ZERO, S::from_f64(15.0))
        );

        let adjacent_boundaries = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::Start,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
                (
                    2,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&adjacent_boundaries, 1).location,
            Point::new(S::ZERO, S::from_f64(15.0))
        );
        assert_eq!(
            fri06_c02_final_node(&adjacent_boundaries, 2).location,
            Point::new(S::ZERO, S::from_f64(15.0))
        );

        let boundaries = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::Start,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
                text(),
                (
                    3,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&boundaries, 0).size.height,
            S::from_f64(20.0)
        );
        assert_eq!(
            fri06_c02_final_node(&boundaries, 1).location,
            Point::new(S::ZERO, S::from_f64(15.0))
        );
        assert_eq!(
            fri06_c02_final_node(&boundaries, 3).location,
            Point::new(S::from_f64(10.0), S::from_f64(15.0))
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_control_visible_break_uses_each_unequal_lines_alignment_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(12.0), S::from_f64(9.0))
                .unwrap();
        let batch = fri06_c03_mixed_batch_with_root(
            vec![
                (
                    1,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        461,
                        20.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
                (
                    2,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
                (
                    3,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        462,
                        6.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::definite(S::from_f64(40.0)),
            NodeInputOf {
                text_align: TextAlign::LegacyCenter,
                ..NodeInputOf::default()
            },
        );

        assert_eq!(
            fri06_c02_final_node(&batch, 1).location.x,
            S::from_f64(10.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 2).location.x,
            S::from_f64(30.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 3).location.x,
            S::from_f64(17.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_text_atomic_control_rewrap_transition_and_progress_both_scalars() {
    fn atomic_participation<S: LayoutScalar>(
        following_break: InlineBreakOpportunityOf<S>,
    ) -> AtomicInlineParticipationOf<S> {
        fri06_c03_atomic_participation(0, following_break)
    }

    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let float = |node, side, inline, block| {
            let style = fri06_c04_line_box(
                flow_axes,
                LogicalSizeOf::new(S::from_f64(inline), S::from_f64(block)),
                side,
                None,
            );
            (node, LayoutInputOf::box_input(style.clone()), style)
        };
        let atomic = |node, inline, block, following_break| {
            let style = fri06_c04_line_box(
                flow_axes,
                LogicalSizeOf::new(S::from_f64(inline), S::from_f64(block)),
                Float::None,
                Some(atomic_participation(following_break)),
            );
            (node, LayoutInputOf::box_input(style.clone()), style)
        };
        let metrics =
            InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0)).unwrap();

        let mixed = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Auto,
            vec![
                float(1, Float::Left, 30.0, 30.0),
                (
                    2,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        601,
                        20.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
                atomic(3, 10.0, 10.0, InlineBreakOpportunityOf::prohibited()),
                (
                    4,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
        );
        let fragment = fri06_c03_fragment(&mixed, 2);
        assert_eq!(fragment.line_index(), 0);
        assert_eq!(fragment.visual_index(), 0);
        assert_eq!(fragment.rect().origin().x, S::from_f64(30.0));
        assert_eq!(
            fri06_c02_final_node(&mixed, 3).location.x,
            S::from_f64(50.0)
        );
        assert_eq!(
            fri06_c02_final_node(&mixed, 4).location.x,
            S::from_f64(60.0)
        );
        assert_eq!(fri06_c02_final_node(&mixed, 4).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&mixed, 0).location, Point::ZERO);
        assert_eq!(
            fri06_c02_final_node(&mixed, 0).size.width,
            S::from_f64(100.0)
        );

        let right = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Auto,
            vec![
                float(1, Float::Right, 30.0, 20.0),
                (
                    2,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        602,
                        20.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
                atomic(3, 20.0, 10.0, InlineBreakOpportunityOf::prohibited()),
                (
                    4,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
        );
        assert_eq!(fri06_c03_fragment(&right, 2).rect().origin().x, S::ZERO);
        assert_eq!(
            fri06_c02_final_node(&right, 3).location.x,
            S::from_f64(20.0)
        );
        assert_eq!(
            fri06_c02_final_node(&right, 4).location.x,
            S::from_f64(40.0)
        );

        let opposing = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Auto,
            vec![
                float(1, Float::Left, 30.0, 20.0),
                float(2, Float::Right, 20.0, 20.0),
                (
                    3,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        603,
                        20.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
                atomic(4, 30.0, 10.0, InlineBreakOpportunityOf::prohibited()),
                (
                    5,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
        );
        assert_eq!(
            fri06_c03_fragment(&opposing, 3).rect().origin().x,
            S::from_f64(30.0)
        );
        assert_eq!(
            fri06_c02_final_node(&opposing, 4).location.x,
            S::from_f64(50.0)
        );
        assert_eq!(
            fri06_c02_final_node(&opposing, 5).location.x,
            S::from_f64(80.0)
        );

        let rewrapped = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Auto,
            vec![
                float(1, Float::Left, 60.0, 10.0),
                (
                    2,
                    fri06_c03_text_input(vec![
                        fri06_c02_segment(
                            611,
                            30.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::allowed(),
                        ),
                        fri06_c02_segment(
                            612,
                            30.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::allowed(),
                        ),
                        fri06_c02_segment(
                            613,
                            30.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::prohibited(),
                        ),
                    ]),
                    NodeInputOf::non_box(),
                ),
            ],
        );
        let fragments = rewrapped
            .final_inline_fragments()
            .iter()
            .filter(|entry| entry.node() == 2)
            .map(|entry| entry.fragment())
            .collect::<Vec<_>>();
        assert_eq!(fragments.len(), 3);
        assert_eq!(fragments[0].line_index(), 0);
        assert_eq!(
            fragments[0].rect().origin(),
            Point::new(S::from_f64(60.0), S::ZERO)
        );
        assert_eq!(fragments[1].line_index(), 1);
        assert_eq!(
            fragments[1].rect().origin(),
            Point::new(S::ZERO, S::from_f64(10.0))
        );
        assert_eq!(fragments[2].line_index(), 1);
        assert_eq!(
            fragments[2].rect().origin(),
            Point::new(S::from_f64(30.0), S::from_f64(10.0))
        );

        let forced = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Auto,
            vec![
                float(1, Float::Left, 30.0, 15.0),
                atomic(2, 20.0, 20.0, InlineBreakOpportunityOf::prohibited()),
                (
                    3,
                    LayoutInputOf::line_break(
                        LineBreakInputOf::new().with_metrics(
                            InlineMetricsOf::from_line_height_and_baseline(
                                S::from_f64(20.0),
                                S::from_f64(15.0),
                            )
                            .unwrap(),
                        ),
                    ),
                    NodeInputOf::non_box(),
                ),
                atomic(4, 10.0, 10.0, InlineBreakOpportunityOf::prohibited()),
            ],
        );
        assert_eq!(
            fri06_c02_final_node(&forced, 2).location.x,
            S::from_f64(30.0)
        );
        assert_eq!(
            fri06_c02_final_node(&forced, 4).location,
            Point::new(S::ZERO, S::from_f64(30.0))
        );

        let no_space = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Auto,
            vec![
                float(1, Float::Left, 50.0, 20.0),
                float(2, Float::Right, 50.0, 20.0),
                atomic(3, 120.0, 10.0, InlineBreakOpportunityOf::prohibited()),
            ],
        );
        assert_eq!(
            fri06_c02_final_node(&no_space, 3).location,
            Point::new(S::ZERO, S::from_f64(20.0))
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_ordinary_block_keeps_outer_edge_and_inherits_parent_float_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for child_direction in [Direction::Ltr, Direction::Rtl] {
            let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
            let root_style = NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::px(S::from_f64(80.0)),
                ),
                ..NodeInputOf::default()
            };
            let float_style = fri06_c04_line_box(
                flow_axes,
                LogicalSizeOf::new(S::from_f64(40.0), S::from_f64(20.0)),
                Float::Left,
                None,
            );
            let ordinary_style = NodeInputOf {
                display: Display::Block,
                direction: child_direction,
                text_align: TextAlign::LegacyLeft,
                margin: Edges {
                    right: LengthAutoOf::px(S::from_f64(5.0)),
                    left: LengthAutoOf::px(S::from_f64(5.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                border: Edges::all(LengthOf::px(S::from_f64(2.0))),
                padding: Edges::all(LengthOf::px(S::from_f64(3.0))),
                ..NodeInputOf::default()
            };
            let text = InlineTextInputOf::try_new(vec![
                fri06_c02_segment(
                    701,
                    30.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::allowed(),
                ),
                fri06_c02_segment(
                    702,
                    30.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::allowed(),
                ),
                fri06_c02_segment(
                    703,
                    30.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ])
            .unwrap();
            let tree = public_layout_tree(
                HashMap::from([
                    (0, LayoutInputOf::box_input(root_style.clone())),
                    (1, LayoutInputOf::box_input(float_style.clone())),
                    (2, LayoutInputOf::box_input(ordinary_style.clone())),
                    (3, LayoutInputOf::inline_text(text)),
                ]),
                HashMap::from([
                    (0, vec![1, 2]),
                    (1, Vec::new()),
                    (2, vec![3]),
                    (3, Vec::new()),
                ]),
            );

            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::new(
                    AvailableOf::definite(S::from_f64(100.0)),
                    AvailableOf::definite(S::from_f64(80.0)),
                ))
                .unwrap(),
            )
            .unwrap();

            let ordinary = fri06_c02_final_node(&batch, 2);
            assert_eq!(ordinary.source_index, SourceIndex::new(1));
            assert_eq!(ordinary.location, Point::new(S::from_f64(5.0), S::ZERO));
            assert_eq!(ordinary.size.width, S::from_f64(90.0));
            assert_eq!(
                fri06_c02_final_node(&batch, 3).source_index,
                SourceIndex::ZERO
            );

            let fragments = batch
                .final_inline_fragments()
                .iter()
                .filter(|entry| entry.node() == 3)
                .map(|entry| entry.fragment())
                .collect::<Vec<_>>();
            assert_eq!(fragments.len(), 3);
            for (index, (segment, line, x, baseline_y)) in [
                (701, 0, 35.0, 13.0),
                (702, 1, 35.0, 23.0),
                (703, 2, 5.0, 33.0),
            ]
            .into_iter()
            .enumerate()
            {
                assert_eq!(fragments[index].segment_id(), InlineSegmentId::new(segment));
                assert_eq!(fragments[index].line_index(), line);
                assert_eq!(fragments[index].visual_index(), 0);
                assert_eq!(fragments[index].rect().origin().x, S::from_f64(x));
                let baseline_x = if child_direction == Direction::Rtl {
                    x + 30.0
                } else {
                    x
                };
                assert_eq!(
                    fragments[index].baseline(),
                    Point::new(S::from_f64(baseline_x), S::from_f64(baseline_y))
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_nested_local_float_keeps_combined_ledger_order_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let root_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(100.0)),
                PreferredSizeOf::px(S::from_f64(80.0)),
            ),
            ..NodeInputOf::default()
        };
        let preceding_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::px(S::ZERO)),
            ..NodeInputOf::default()
        };
        let parent_float_style = fri06_c04_line_box(
            flow_axes,
            LogicalSizeOf::new(S::from_f64(30.0), S::from_f64(30.0)),
            Float::Left,
            None,
        );
        let ordinary_style = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let local_float_style = fri06_c04_line_box(
            flow_axes,
            LogicalSizeOf::new(S::from_f64(20.0), S::from_f64(20.0)),
            Float::Right,
            None,
        );
        let text = InlineTextInputOf::try_new(vec![fri06_c02_segment(
            704,
            40.0,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )])
        .unwrap();
        let tree = public_layout_tree(
            HashMap::from([
                (0, LayoutInputOf::box_input(root_style.clone())),
                (1, LayoutInputOf::box_input(preceding_style.clone())),
                (2, LayoutInputOf::box_input(parent_float_style.clone())),
                (3, LayoutInputOf::box_input(ordinary_style.clone())),
                (4, LayoutInputOf::box_input(local_float_style.clone())),
                (5, LayoutInputOf::inline_text(text)),
            ]),
            HashMap::from([
                (0, vec![1, 2, 3]),
                (1, Vec::new()),
                (2, Vec::new()),
                (3, vec![4, 5]),
                (4, Vec::new()),
                (5, Vec::new()),
            ]),
        );

        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(100.0)),
                AvailableOf::definite(S::from_f64(80.0)),
            ))
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            fri06_c02_final_node(&batch, 2).source_index,
            SourceIndex::new(1)
        );
        let ordinary = fri06_c02_final_node(&batch, 3);
        assert_eq!(ordinary.source_index, SourceIndex::new(2));
        assert_eq!(ordinary.location, Point::ZERO);
        assert_eq!(ordinary.size.width, S::from_f64(100.0));

        let local_float = fri06_c02_final_node(&batch, 4);
        assert_eq!(local_float.source_index, SourceIndex::ZERO);
        assert_eq!(local_float.location, Point::new(S::from_f64(80.0), S::ZERO));
        assert_eq!(
            fri06_c02_final_node(&batch, 5).source_index,
            SourceIndex::new(1)
        );
        let fragment = fri06_c03_fragment(&batch, 5);
        assert_eq!(fragment.line_index(), 0);
        assert_eq!(
            fragment.rect().origin(),
            Point::new(S::from_f64(30.0), S::ZERO)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_bfc_role_exact_positive_and_negative_matrix_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for flow_axes in [
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        ] {
            let sized = |display, overflow, item_is_replaced, position, float| {
                let mut style = NodeInputOf {
                    display,
                    writing_mode: flow_axes.writing_mode(),
                    direction: flow_axes.direction(),
                    overflow,
                    item_is_replaced,
                    position,
                    float,
                    size: flow_axes.physical_size(LogicalSizeOf::new(
                        PreferredSizeOf::px(S::from_f64(20.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    )),
                    ..NodeInputOf::default()
                };
                if display.is_inline_level() {
                    style.atomic_inline_participation = Some(fri06_c03_atomic_participation(
                        0,
                        InlineBreakOpportunityOf::prohibited(),
                    ));
                }
                style
            };
            let float_style = sized(
                Display::Block,
                ComputedOverflow::VISIBLE,
                false,
                Position::Relative,
                Float::Left,
            );
            let float_style = NodeInputOf {
                size: flow_axes.physical_size(LogicalSizeOf::new(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::px(S::from_f64(30.0)),
                )),
                ..float_style
            };
            let hidden = computed_overflow(Overflow::Hidden, Overflow::Hidden);
            let scroll = computed_overflow(Overflow::Scroll, Overflow::Scroll);
            let auto = computed_overflow(Overflow::Auto, Overflow::Auto);
            let clip = computed_overflow(Overflow::Clip, Overflow::Clip);

            let positive = [
                ("flex", Display::Flex, ComputedOverflow::VISIBLE, false),
                (
                    "replaced-flex",
                    Display::Flex,
                    ComputedOverflow::VISIBLE,
                    true,
                ),
                ("grid", Display::Grid, ComputedOverflow::VISIBLE, false),
                (
                    "replaced-grid",
                    Display::Grid,
                    ComputedOverflow::VISIBLE,
                    true,
                ),
                (
                    "grid-lanes",
                    Display::GridLanes,
                    ComputedOverflow::VISIBLE,
                    false,
                ),
                (
                    "replaced-grid-lanes",
                    Display::GridLanes,
                    ComputedOverflow::VISIBLE,
                    true,
                ),
                ("block-hidden", Display::Block, hidden, false),
                ("block-scroll", Display::Block, scroll, false),
                ("block-auto", Display::Block, auto, false),
            ];
            for (label, display, overflow, item_is_replaced) in positive {
                let subject = sized(
                    display,
                    overflow,
                    item_is_replaced,
                    Position::Relative,
                    Float::None,
                );
                let batch = fri06_c04_bfc_batch(
                    flow_axes,
                    vec![1, 2],
                    vec![
                        (1, float_style.clone(), Vec::new()),
                        (2, subject, Vec::new()),
                    ],
                );
                let output = fri06_c02_final_node(&batch, 2);
                let origin = fri06_c04_logical_origin(flow_axes, output);
                assert_eq!(
                    origin,
                    LogicalPointOf::new(S::from_f64(40.0), S::ZERO),
                    "positive BFC role {label} did not avoid the active float in {flow_axes:?}",
                );
                assert_eq!(
                    flow_axes.logical_size(output.size).inline,
                    S::from_f64(20.0),
                    "positive BFC role {label} changed its definite size in {flow_axes:?}",
                );
            }

            let ordinary_negative = [
                ("block-visible", ComputedOverflow::VISIBLE, false),
                ("block-clip", clip, false),
                ("replaced-block-hidden", hidden, true),
                ("replaced-block-scroll", scroll, true),
                ("replaced-block-auto", auto, true),
            ];
            for (label, overflow, item_is_replaced) in ordinary_negative {
                let subject = sized(
                    Display::Block,
                    overflow,
                    item_is_replaced,
                    Position::Relative,
                    Float::None,
                );
                let batch = fri06_c04_bfc_batch(
                    flow_axes,
                    vec![1, 2],
                    vec![
                        (1, float_style.clone(), Vec::new()),
                        (2, subject, Vec::new()),
                    ],
                );
                assert_eq!(
                    fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&batch, 2)),
                    LogicalPointOf::new(S::ZERO, S::ZERO),
                    "negative BFC role {label} moved its ordinary outer edge in {flow_axes:?}",
                );
            }

            for display in [
                Display::InlineBlock,
                Display::InlineGrid,
                Display::InlineGridLanes,
            ] {
                let subject = sized(display, hidden, false, Position::Relative, Float::None);
                let batch = fri06_c04_bfc_batch(
                    flow_axes,
                    vec![1, 2],
                    vec![
                        (1, float_style.clone(), Vec::new()),
                        (2, subject, Vec::new()),
                    ],
                );
                let output = fri06_c02_final_node(&batch, 2);
                assert_eq!(
                    fri06_c04_logical_origin(flow_axes, output),
                    LogicalPointOf::new(S::from_f64(40.0), S::ZERO),
                    "{display:?} must participate in the float-adjusted inline line",
                );
                assert_eq!(
                    flow_axes.logical_size(output.size).inline,
                    S::from_f64(20.0)
                );
            }

            let absolute = sized(
                Display::Flex,
                hidden,
                false,
                Position::Absolute,
                Float::None,
            );
            let floating = sized(
                Display::Flex,
                hidden,
                false,
                Position::Relative,
                Float::Left,
            );
            let none = sized(
                Display::None,
                hidden,
                false,
                Position::Relative,
                Float::None,
            );
            for (label, subject, expected_origin, expected_inline_size) in [
                ("absolute", absolute, S::ZERO, S::from_f64(20.0)),
                ("floating", floating, S::from_f64(40.0), S::from_f64(20.0)),
                ("display-none", none, S::ZERO, S::ZERO),
            ] {
                let batch = fri06_c04_bfc_batch(
                    flow_axes,
                    vec![1, 2],
                    vec![
                        (1, float_style.clone(), Vec::new()),
                        (2, subject, Vec::new()),
                    ],
                );
                let output = fri06_c02_final_node(&batch, 2);
                if label == "display-none" {
                    assert_eq!(output.location, Point::ZERO);
                } else {
                    assert_eq!(
                        fri06_c04_logical_origin(flow_axes, output).inline,
                        expected_origin,
                        "{label} entered block-child BFC avoidance in {flow_axes:?}",
                    );
                }
                assert_eq!(
                    flow_axes.logical_size(output.size).inline,
                    expected_inline_size,
                    "{label} changed size in {flow_axes:?}",
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_bfc_size_auto_definite_zero_overwide_margin_boxes_and_clear_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for flow_axes in [
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        ] {
            let box_style = |inline: PreferredSizeOf<S>, block: f64| NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: flow_axes.physical_size(LogicalSizeOf::new(
                    inline,
                    PreferredSizeOf::px(S::from_f64(block)),
                )),
                ..NodeInputOf::default()
            };
            let floated = |side, inline, block| NodeInputOf {
                float: side,
                overflow: ComputedOverflow::VISIBLE,
                ..box_style(PreferredSizeOf::px(S::from_f64(inline)), block)
            };
            let margin = |inline_start: f64, inline_end: f64| {
                flow_axes.physical_edges(
                    crate::geometry::LogicalEdgesOf::new(
                        S::from_f64(inline_start),
                        S::from_f64(inline_end),
                        S::ZERO,
                        S::ZERO,
                    )
                    .map(LengthAutoOf::px),
                )
            };
            let run = |subject: NodeInputOf<S>, floats: Vec<NodeInputOf<S>>| {
                let mut children = Vec::new();
                let mut nodes = Vec::new();
                for (index, float) in floats.into_iter().enumerate() {
                    let node = u32::try_from(index + 1).unwrap();
                    children.push(node);
                    nodes.push((node, float, Vec::new()));
                }
                let subject_node = u32::try_from(children.len() + 1).unwrap();
                children.push(subject_node);
                nodes.push((subject_node, subject, Vec::new()));
                let batch = fri06_c04_bfc_batch(flow_axes, children, nodes);
                fri06_c02_final_node(&batch, subject_node)
            };

            for (display, overflow) in [
                (
                    Display::Block,
                    computed_overflow(Overflow::Hidden, Overflow::Hidden),
                ),
                (Display::Flex, ComputedOverflow::VISIBLE),
                (Display::Grid, ComputedOverflow::VISIBLE),
                (Display::GridLanes, ComputedOverflow::VISIBLE),
            ] {
                let auto = run(
                    NodeInputOf {
                        display,
                        overflow,
                        margin: margin(10.0, 10.0),
                        ..box_style(PreferredSizeOf::AUTO, 10.0)
                    },
                    vec![floated(Float::Left, 40.0, 20.0)],
                );
                assert_eq!(
                    fri06_c04_logical_origin(flow_axes, auto),
                    LogicalPointOf::new(S::from_f64(50.0), S::ZERO),
                    "{display:?} auto inline placement did not use its selected band",
                );
                assert_eq!(
                    flow_axes.logical_size(auto.size).inline,
                    S::from_f64(40.0),
                    "{display:?} auto inline size was not saturated to its selected band",
                );
            }

            let definite = run(
                NodeInputOf {
                    margin: margin(5.0, 5.0),
                    ..box_style(PreferredSizeOf::px(S::from_f64(50.0)), 10.0)
                },
                vec![floated(Float::Left, 40.0, 20.0)],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, definite),
                LogicalPointOf::new(S::from_f64(45.0), S::ZERO),
            );

            let spanning = run(
                box_style(PreferredSizeOf::px(S::from_f64(70.0)), 20.0),
                vec![
                    floated(Float::Left, 20.0, 10.0),
                    floated(Float::Right, 90.0, 20.0),
                ],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, spanning),
                LogicalPointOf::new(S::ZERO, S::from_f64(30.0)),
                "complete-span collision must observe the later-starting float",
            );

            let zero = run(
                NodeInputOf {
                    margin: margin(35.0, 35.0),
                    ..box_style(PreferredSizeOf::px(S::ZERO), 10.0)
                },
                vec![floated(Float::Left, 40.0, 20.0)],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, zero),
                LogicalPointOf::new(S::from_f64(35.0), S::from_f64(20.0)),
            );
            assert_eq!(flow_axes.logical_size(zero.size).inline, S::ZERO);

            let overwide = run(
                NodeInputOf {
                    margin: margin(5.0, 5.0),
                    ..box_style(PreferredSizeOf::px(S::from_f64(120.0)), 10.0)
                },
                vec![floated(Float::Left, 40.0, 20.0)],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, overwide),
                LogicalPointOf::new(S::from_f64(5.0), S::from_f64(20.0)),
            );

            let cleared = run(
                NodeInputOf {
                    clear: Clear::Left,
                    margin: margin(10.0, 10.0),
                    ..box_style(PreferredSizeOf::AUTO, 10.0)
                },
                vec![
                    floated(Float::Left, 40.0, 20.0),
                    floated(Float::Right, 30.0, 40.0),
                ],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, cleared),
                LogicalPointOf::new(S::from_f64(10.0), S::from_f64(20.0)),
            );
            assert_eq!(
                flow_axes.logical_size(cleared.size).inline,
                S::from_f64(50.0),
            );

            let ordinary = run(
                NodeInputOf {
                    overflow: ComputedOverflow::VISIBLE,
                    margin: margin(10.0, 10.0),
                    ..box_style(PreferredSizeOf::AUTO, 10.0)
                },
                vec![floated(Float::Left, 40.0, 20.0)],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, ordinary),
                LogicalPointOf::new(S::from_f64(10.0), S::ZERO),
            );
            assert_eq!(
                flow_axes.logical_size(ordinary.size).inline,
                S::from_f64(80.0),
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_size_auto_block_encloses_inset_float_mixed_and_nested_both_scalars() {
    fn float_style<S: LayoutScalar>(
        flow_axes: FlowAxes,
        inline: f64,
        block: f64,
        clear: Clear,
    ) -> NodeInputOf<S> {
        NodeInputOf {
            display: Display::Block,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            float: Float::Left,
            clear,
            size: flow_axes
                .physical_size(LogicalSizeOf::new(S::from_f64(inline), S::from_f64(block)))
                .map(PreferredSizeOf::px),
            margin: flow_axes
                .physical_edges(LogicalEdgesOf::new(
                    S::from_f64(1.0),
                    S::from_f64(2.0),
                    S::from_f64(2.0),
                    S::from_f64(3.0),
                ))
                .map(LengthAutoOf::px),
            ..NodeInputOf::default()
        }
    }

    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in root_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let padding = LogicalEdgesOf::new(
                S::from_f64(1.0),
                S::from_f64(1.0),
                S::from_f64(3.0),
                S::from_f64(5.0),
            );
            let border = LogicalEdgesOf::new(
                S::from_f64(1.0),
                S::from_f64(1.0),
                S::from_f64(2.0),
                S::from_f64(4.0),
            );
            let root_style = NodeInputOf {
                display: Display::Block,
                writing_mode,
                direction,
                size: flow_axes.physical_size(LogicalSizeOf::new(
                    PreferredSizeOf::px(S::from_f64(80.0)),
                    PreferredSizeOf::AUTO,
                )),
                padding: flow_axes.physical_edges(padding).map(LengthOf::px),
                border: flow_axes.physical_edges(border).map(LengthOf::px),
                ..NodeInputOf::default()
            };
            let float = float_style(flow_axes, 20.0, 10.0, Clear::None);
            let float_only = fri06_c04_front_door_batch(
                root_style.clone(),
                LogicalSizeOf::new(
                    AvailableOf::definite(S::from_f64(80.0)),
                    AvailableOf::MAX_CONTENT,
                ),
                vec![1],
                vec![(
                    1,
                    LayoutInputOf::box_input(float.clone()),
                    float.clone(),
                    Vec::new(),
                )],
            );
            let root = fri06_c02_final_node(&float_only, 0);
            assert_eq!(
                flow_axes.logical_size(root.size).block,
                S::from_f64(29.0),
                "float-only auto block size must count the five-unit start inset once for {flow_axes:?}",
            );
            let floated = fri06_c02_final_node(&float_only, 1);
            let floated_origin = flow_axes.logical_point(floated.location, floated.size, root.size);
            assert_eq!(floated_origin.block, S::from_f64(7.0));

            let text = InlineTextInputOf::try_new(vec![fri06_c02_segment(
                940,
                20.0,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            )])
            .unwrap();
            let mixed = fri06_c04_front_door_batch(
                root_style,
                LogicalSizeOf::new(
                    AvailableOf::definite(S::from_f64(80.0)),
                    AvailableOf::MAX_CONTENT,
                ),
                vec![1, 2],
                vec![
                    (
                        1,
                        LayoutInputOf::box_input(float.clone()),
                        float,
                        Vec::new(),
                    ),
                    (
                        2,
                        LayoutInputOf::inline_text(text),
                        NodeInputOf::non_box(),
                        Vec::new(),
                    ),
                ],
            );
            assert_eq!(
                flow_axes
                    .logical_size(fri06_c02_final_node(&mixed, 0).size)
                    .block,
                S::from_f64(29.0),
                "mixed normal flow must not hide the taller owned float for {flow_axes:?}",
            );
        }

        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let root_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(80.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        };
        let nested_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(60.0)),
                PreferredSizeOf::AUTO,
            ),
            padding: Edges {
                top: LengthOf::px(S::from_f64(4.0)),
                bottom: LengthOf::px(S::from_f64(6.0)),
                ..Edges::all(LengthOf::ZERO)
            },
            overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap(),
            ..NodeInputOf::default()
        };
        let nested_float = NodeInputOf {
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(1.0)),
                bottom: LengthAutoOf::px(S::from_f64(2.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..float_style(flow_axes, 20.0, 8.0, Clear::Both)
        };
        let following_text = InlineTextInputOf::try_new(vec![fri06_c02_segment(
            941,
            10.0,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )])
        .unwrap();
        let nested = fri06_c04_front_door_batch(
            root_style,
            LogicalSizeOf::new(
                AvailableOf::definite(S::from_f64(80.0)),
                AvailableOf::MAX_CONTENT,
            ),
            vec![1, 3],
            vec![
                (
                    1,
                    LayoutInputOf::box_input(nested_style.clone()),
                    nested_style,
                    vec![2],
                ),
                (
                    2,
                    LayoutInputOf::box_input(nested_float.clone()),
                    nested_float,
                    Vec::new(),
                ),
                (
                    3,
                    LayoutInputOf::inline_text(following_text),
                    NodeInputOf::non_box(),
                    Vec::new(),
                ),
            ],
        );
        let root = fri06_c02_final_node(&nested, 0);
        let nested_output = fri06_c02_final_node(&nested, 1);
        assert_eq!(nested_output.size.height, S::from_f64(21.0));
        assert_eq!(root.size.height, S::from_f64(31.0));
        assert_eq!(
            fri06_c02_final_node(&nested, 3).location,
            Point::new(S::ZERO, S::from_f64(21.0))
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_scroll_signed_fractional_geometry_and_source_order_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let root_style = NodeInputOf {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(20.0)),
                PreferredSizeOf::px(S::from_f64(8.0)),
            ),
            overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
            ..NodeInputOf::default()
        };
        let float = NodeInputOf {
            display: Display::Block,
            direction: Direction::Rtl,
            float: Float::Left,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.5)),
                PreferredSizeOf::px(S::from_f64(8.25)),
            ),
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(0.25)),
                bottom: LengthAutoOf::px(S::from_f64(0.25)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..NodeInputOf::default()
        };
        let text = InlineTextInputOf::try_new(vec![fri06_c02_segment(
            950,
            5.25,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )])
        .unwrap();
        let batch = fri06_c04_front_door_batch(
            root_style,
            LogicalSizeOf::new(
                AvailableOf::definite(S::from_f64(20.0)),
                AvailableOf::definite(S::from_f64(8.0)),
            ),
            vec![1, 2],
            vec![
                (
                    1,
                    LayoutInputOf::box_input(float.clone()),
                    float,
                    Vec::new(),
                ),
                (
                    2,
                    LayoutInputOf::inline_text(text),
                    NodeInputOf::non_box(),
                    Vec::new(),
                ),
            ],
        );
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(
                entries
                    .iter()
                    .map(LayoutOutputEntryOf::node)
                    .collect::<Vec<_>>(),
                vec![0, 1, 2],
            );
            assert_eq!(
                public_flow_output(entries, 1).source_index,
                SourceIndex::new(0)
            );
            assert_eq!(
                public_flow_output(entries, 2).source_index,
                SourceIndex::new(1)
            );
        }
        let unrounded_float = public_flow_output(batch.unrounded_entries(), 1);
        let rounded_float = public_flow_output(batch.final_entries(), 1);
        assert_eq!(
            unrounded_float.location,
            Point::new(S::from_f64(-10.5), S::from_f64(0.25))
        );
        assert_eq!(
            rounded_float.location,
            Point::new(S::from_f64(-10.0), S::ZERO)
        );
        let unrounded_root = public_flow_output(batch.unrounded_entries(), 0);
        let unrounded_geometry = unrounded_root.scroll_geometry.unwrap();
        assert_eq!(
            unrounded_geometry.scrollable_overflow().origin().x,
            S::from_f64(-10.5)
        );
        assert_eq!(
            unrounded_geometry.scrollable_overflow().size().width,
            S::from_f64(30.5)
        );
        assert_eq!(
            unrounded_geometry.physical_range().x().minimum(),
            S::from_f64(-10.5)
        );
        assert_eq!(unrounded_geometry.physical_range().x().maximum(), S::ZERO);
        let fragment = batch.unrounded_inline_fragments()[0].fragment();
        assert_eq!(fragment.line_index(), 0);
        assert_eq!(fragment.visual_index(), 0);
        assert_eq!(fragment.rect().origin().y, S::from_f64(8.75));
        let rounded_fragment = batch.final_inline_fragments()[0].fragment();
        assert_eq!(rounded_fragment.line_index(), fragment.line_index());
        assert_eq!(rounded_fragment.visual_index(), fragment.visual_index());
        assert_eq!(rounded_fragment.rect().origin().y, S::from_f64(9.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_mixed_text_atomic_source_gaps_and_repeat_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let children = || {
            let atomic = fri06_c03_atomic_style(
                20.0,
                6.0,
                2.0,
                3.0,
                0,
                InlineBreakOpportunityOf::prohibited(),
            );
            let hidden = NodeInputOf {
                display: Display::None,
                ..NodeInputOf::default()
            };
            let absolute = NodeInputOf {
                position: Position::Absolute,
                ..fri06_c03_atomic_style(
                    7.0,
                    5.0,
                    0.0,
                    0.0,
                    0,
                    InlineBreakOpportunityOf::prohibited(),
                )
            };
            let floated = NodeInputOf {
                float: Float::Left,
                ..fri06_c03_atomic_style(
                    8.0,
                    4.0,
                    0.0,
                    0.0,
                    0,
                    InlineBreakOpportunityOf::prohibited(),
                )
            };
            vec![
                (
                    1,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        101,
                        10.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
                (2, LayoutInputOf::box_input(hidden.clone()), hidden),
                (3, LayoutInputOf::box_input(atomic.clone()), atomic),
                (4, LayoutInputOf::box_input(absolute.clone()), absolute),
                (
                    5,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        102,
                        10.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
                (6, LayoutInputOf::box_input(floated.clone()), floated),
                (
                    7,
                    fri06_c03_text_input(vec![fri06_c02_segment(
                        103,
                        5.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )]),
                    NodeInputOf::non_box(),
                ),
            ]
        };
        let first = fri06_c03_mixed_batch(children(), AvailableOf::definite(S::from_f64(100.0)));
        let second = fri06_c03_mixed_batch(children(), AvailableOf::definite(S::from_f64(100.0)));
        assert_eq!(first.final_entries(), second.final_entries());
        assert_eq!(
            first.final_inline_fragments(),
            second.final_inline_fragments()
        );

        let fragments = first.final_inline_fragments();
        assert_eq!(fragments.len(), 3);
        assert_eq!(fragments[0].node(), 1);
        assert_eq!(
            fragments[0].fragment().segment_id(),
            InlineSegmentId::new(101)
        );
        assert_eq!(fragments[0].fragment().rect().origin().x, S::ZERO);
        assert_eq!(fragments[1].node(), 5);
        assert_eq!(
            fragments[1].fragment().segment_id(),
            InlineSegmentId::new(102)
        );
        assert_eq!(fragments[1].fragment().rect().origin().x, S::from_f64(35.0));
        assert_eq!(fragments[2].node(), 7);
        assert_eq!(
            fragments[2].fragment().segment_id(),
            InlineSegmentId::new(103)
        );
        assert_eq!(
            fragments[2].fragment().rect().origin(),
            Point::new(S::from_f64(8.0), S::from_f64(10.0))
        );

        let hidden = fri06_c02_final_node(&first, 2);
        let atomic = fri06_c02_final_node(&first, 3);
        let trailing_text = fri06_c02_final_node(&first, 5);
        let float = fri06_c02_final_node(&first, 6);
        let after_float = fri06_c02_final_node(&first, 7);
        assert_eq!(hidden.source_index, SourceIndex::new(1));
        assert_eq!(hidden.size, Size::ZERO);
        assert_eq!(atomic.source_index, SourceIndex::new(2));
        assert_eq!(atomic.location.x, S::from_f64(12.0));
        assert_eq!(trailing_text.source_index, SourceIndex::new(4));
        assert_eq!(float.source_index, SourceIndex::new(5));
        assert_eq!(after_float.source_index, SourceIndex::new(6));
        assert_eq!(
            first
                .final_entries()
                .iter()
                .filter(|entry| entry.node() == 3)
                .count(),
            1,
            "one supplied atomic placeholder publishes exactly once"
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_atomic_break_opportunities_overwide_and_intrinsic_both_scalars() {
    fn text<S: LayoutScalar>(
        node: u32,
        id: u64,
        extent: f64,
        following_break: InlineBreakOpportunityOf<S>,
    ) -> (u32, LayoutInputOf<S>, NodeInputOf<S>) {
        (
            node,
            fri06_c03_text_input(vec![fri06_c02_segment(
                id,
                extent,
                InlineWhitespaceEdge::Preserve,
                following_break,
            )]),
            NodeInputOf::non_box(),
        )
    }

    fn atomic<S: LayoutScalar>(
        node: u32,
        extent: f64,
        margin_start: f64,
        margin_end: f64,
        following_break: InlineBreakOpportunityOf<S>,
    ) -> (u32, LayoutInputOf<S>, NodeInputOf<S>) {
        let style =
            fri06_c03_atomic_style(extent, 10.0, margin_start, margin_end, 0, following_break);
        (node, LayoutInputOf::box_input(style.clone()), style)
    }

    fn assert_lane<S: LayoutScalar>() {
        let allowed = fri06_c03_mixed_batch(
            vec![
                text(1, 201, 10.0, InlineBreakOpportunityOf::allowed()),
                atomic(2, 8.0, 1.0, 1.0, InlineBreakOpportunityOf::prohibited()),
                atomic(3, 8.0, 1.0, 1.0, InlineBreakOpportunityOf::allowed()),
                text(4, 202, 15.0, InlineBreakOpportunityOf::prohibited()),
            ],
            AvailableOf::definite(S::from_f64(30.0)),
        );
        let fragments = allowed.final_inline_fragments();
        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].fragment().line_index(), 0);
        assert_eq!(fragments[1].fragment().line_index(), 1);
        assert_eq!(fri06_c02_final_node(&allowed, 3).location.y, S::ZERO);
        assert_eq!(
            fragments[1].fragment().rect().origin(),
            Point::new(S::ZERO, S::from_f64(12.0))
        );

        let mandatory = fri06_c03_mixed_batch(
            vec![
                text(1, 211, 10.0, InlineBreakOpportunityOf::prohibited()),
                atomic(2, 10.0, 0.0, 0.0, InlineBreakOpportunityOf::mandatory()),
                text(3, 212, 10.0, InlineBreakOpportunityOf::prohibited()),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
        );
        assert_eq!(
            mandatory.final_inline_fragments()[0]
                .fragment()
                .line_index(),
            0
        );
        assert_eq!(
            mandatory.final_inline_fragments()[1]
                .fragment()
                .line_index(),
            1
        );

        let overwide = fri06_c03_mixed_batch(
            vec![
                atomic(1, 40.0, 0.0, 0.0, InlineBreakOpportunityOf::mandatory()),
                text(2, 221, 5.0, InlineBreakOpportunityOf::prohibited()),
            ],
            AvailableOf::definite(S::from_f64(20.0)),
        );
        assert_eq!(
            fri06_c02_final_node(&overwide, 1).size.width,
            S::from_f64(40.0)
        );
        assert_eq!(
            overwide.final_inline_fragments()[0].fragment().line_index(),
            1
        );

        let intrinsic_children = || {
            vec![
                text(1, 231, 10.0, InlineBreakOpportunityOf::<S>::allowed()),
                atomic(
                    2,
                    20.0,
                    5.0,
                    5.0,
                    InlineBreakOpportunityOf::<S>::mandatory(),
                ),
                text(3, 232, 15.0, InlineBreakOpportunityOf::<S>::prohibited()),
            ]
        };
        let min = fri06_c03_mixed_batch(intrinsic_children(), AvailableOf::MIN_CONTENT);
        let max = fri06_c03_mixed_batch(intrinsic_children(), AvailableOf::MAX_CONTENT);
        assert_eq!(fri06_c02_final_node(&min, 0).size.width, S::from_f64(30.0));
        assert_eq!(fri06_c02_final_node(&max, 0).size.width, S::from_f64(40.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_mixed_bidi_reorders_complete_units_per_line_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let text = |node, id, extent, level, whitespace, following_break| {
            (
                node,
                fri06_c03_text_input(vec![fri06_c02_segment_with_level(
                    id,
                    extent,
                    level,
                    whitespace,
                    following_break,
                )]),
                NodeInputOf::non_box(),
            )
        };
        let atomic = |node, level, following_break| {
            let style = fri06_c03_atomic_style(10.0, 10.0, 0.0, 0.0, level, following_break);
            (node, LayoutInputOf::box_input(style.clone()), style)
        };
        let batch = fri06_c03_mixed_batch(
            vec![
                text(
                    1,
                    301,
                    10.0,
                    1,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                atomic(2, 2, InlineBreakOpportunityOf::prohibited()),
                text(
                    3,
                    302,
                    5.0,
                    1,
                    InlineWhitespaceEdge::DiscardAtLineEnd,
                    InlineBreakOpportunityOf::allowed(),
                ),
                text(
                    4,
                    303,
                    5.0,
                    1,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                atomic(5, 2, InlineBreakOpportunityOf::prohibited()),
                text(
                    6,
                    304,
                    5.0,
                    1,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ],
            AvailableOf::definite(S::from_f64(25.0)),
        );

        let fragments = batch.final_inline_fragments();
        assert_eq!(
            fragments
                .iter()
                .map(|entry| entry.fragment().segment_id())
                .collect::<Vec<_>>(),
            [
                InlineSegmentId::new(301),
                InlineSegmentId::new(303),
                InlineSegmentId::new(304),
            ]
        );
        assert_eq!(
            fragments
                .iter()
                .map(|entry| (
                    entry.fragment().line_index(),
                    entry.fragment().visual_index()
                ))
                .collect::<Vec<_>>(),
            [(0, 2), (1, 2), (1, 0)]
        );
        assert_eq!(
            fragments[0].fragment().rect().origin(),
            Point::new(S::from_f64(10.0), S::from_f64(2.0))
        );
        assert_eq!(
            fragments[1].fragment().rect().origin(),
            Point::new(S::from_f64(15.0), S::from_f64(14.0))
        );
        assert_eq!(
            fragments[2].fragment().rect().origin(),
            Point::new(S::ZERO, S::from_f64(14.0))
        );
        assert_eq!(fri06_c02_final_node(&batch, 2).location, Point::ZERO);
        assert_eq!(
            fri06_c02_final_node(&batch, 5).location,
            Point::new(S::from_f64(5.0), S::from_f64(12.0))
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_r1_bidi_range_four_variants_preserve_complete_unit_geometry() {
    fn assert_lane<S: LayoutScalar>() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
                assert_fri06_c08_r1_mixed_unit_traversal::<S>(
                    FlowAxes::new(WritingMode::HorizontalTb, direction),
                    box_sizing,
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_r1_rtl_visual_placement_is_independent_of_surviving_unit_composition() {
    fn assert_lane<S: LayoutScalar>() {
        for writing_mode in [
            WritingMode::HorizontalTb,
            WritingMode::VerticalRl,
            WritingMode::VerticalLr,
            WritingMode::SidewaysRl,
            WritingMode::SidewaysLr,
        ] {
            let flow_axes = FlowAxes::new(writing_mode, Direction::Rtl);
            let placements = [
                None,
                Some(InlineWhitespaceEdge::DiscardAtBoth),
                Some(InlineWhitespaceEdge::Preserve),
            ]
            .map(|middle_whitespace| {
                let atomic = |node, inline_extent| {
                    let style = fri06_c04_line_box(
                        flow_axes,
                        LogicalSizeOf::new(S::from_f64(inline_extent), S::from_f64(12.0)),
                        Float::None,
                        Some(fri06_c03_atomic_participation(
                            1,
                            InlineBreakOpportunityOf::prohibited(),
                        )),
                    );
                    (node, LayoutInputOf::box_input(style.clone()), style)
                };
                let mut children = vec![atomic(1, 10.0)];
                children.push(match middle_whitespace {
                    Some(whitespace) => (
                        2,
                        fri06_c03_text_input(vec![fri06_c02_segment_with_level(
                            903,
                            15.0,
                            1,
                            whitespace,
                            InlineBreakOpportunityOf::prohibited(),
                        )]),
                        NodeInputOf::non_box(),
                    ),
                    None => atomic(2, 15.0),
                });
                children.push(atomic(3, 20.0));
                let batch = fri06_c04_line_batch(flow_axes, TextAlign::Auto, children);
                let root_size = flow_axes
                    .physical_size(LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(160.0)));
                let logical_inline_start = |node| {
                    let output = fri06_c02_final_node(&batch, node);
                    flow_axes
                        .logical_point(output.location, output.size, root_size)
                        .inline
                };

                if middle_whitespace.is_some() {
                    let fragment = batch.final_inline_fragments()[0].fragment();
                    assert_eq!(fragment.segment_id(), InlineSegmentId::new(903));
                    assert_eq!(fragment.visual_index(), 1);
                }

                [logical_inline_start(1), logical_inline_start(3)]
            });

            assert_eq!(
                placements,
                [
                    [S::from_f64(35.0), S::ZERO],
                    [S::from_f64(35.0), S::ZERO],
                    [S::from_f64(35.0), S::ZERO],
                ],
                "{writing_mode:?} RTL all-atomic, middle-discard, and middle-preserve slices"
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_mixed_inline_rtl_border_box_projects_reordered_slice_once() {
    assert_fri06_c08_mixed_inline_atomic_x::<f32>(Direction::Rtl, BoxSizing::BorderBox, 99.0);
    assert_fri06_c08_mixed_inline_atomic_x::<f64>(Direction::Rtl, BoxSizing::BorderBox, 99.0);
}

#[test]
fn fri06_c08_mixed_inline_rtl_content_box_projects_reordered_slice_once() {
    assert_fri06_c08_mixed_inline_atomic_x::<f32>(Direction::Rtl, BoxSizing::ContentBox, 99.0);
    assert_fri06_c08_mixed_inline_atomic_x::<f64>(Direction::Rtl, BoxSizing::ContentBox, 99.0);
}

fn fri06_c08_float_line_control_batch<S: LayoutScalar>(
    mut in_flow: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    right_float_block_extent: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let left_float = fri06_c04_line_box(
        flow_axes,
        LogicalSizeOf::new(S::from_f64(42.0), S::from_f64(42.0)),
        Float::Left,
        None,
    );
    let right_float = fri06_c04_line_box(
        flow_axes,
        LogicalSizeOf::new(S::from_f64(50.0), S::from_f64(right_float_block_extent)),
        Float::Right,
        None,
    );
    let mut children = vec![
        (1, LayoutInputOf::box_input(left_float.clone()), left_float),
        (
            2,
            LayoutInputOf::box_input(right_float.clone()),
            right_float,
        ),
    ];
    children.append(&mut in_flow);
    fri06_c03_mixed_batch_with_root(
        children,
        AvailableOf::definite(S::from_f64(180.0)),
        NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(180.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        },
    )
}

fn assert_fri06_c08_float_line_control_height<S: LayoutScalar>(
    in_flow: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
) {
    assert_eq!(
        public_flow_output(
            fri06_c08_float_line_control_batch(in_flow, 62.0).final_entries(),
            0,
        )
        .size,
        Size::new(S::from_f64(180.0), S::from_f64(62.0)),
    );
}

fn fri06_c08_float_line_continuation_input<S: LayoutScalar>(
    line_height: f64,
    levels: [u8; 4],
) -> Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)> {
    let atomic = |node, inline, level, following_break| {
        let style = fri06_c03_atomic_style(inline, 16.0, 0.0, 0.0, level, following_break);
        (node, LayoutInputOf::box_input(style.clone()), style)
    };
    let strut = InlineBoundaryInputOf::new(
        InlineBoundaryKind::Start,
        InlineMetricsOf::from_line_height_and_baseline(S::from_f64(line_height), S::from_f64(12.0))
            .unwrap(),
    );
    vec![
        (
            3,
            fri06_c03_text_input(vec![fri06_c02_segment_with_metrics(813, 40.0, 14.8, 5.2)]),
            NodeInputOf::non_box(),
        ),
        (
            8,
            LayoutInputOf::inline_boundary(strut),
            NodeInputOf::non_box(),
        ),
        atomic(4, 28.0, levels[0], InlineBreakOpportunityOf::allowed()),
        atomic(5, 32.0, levels[1], InlineBreakOpportunityOf::allowed()),
        atomic(6, 36.0, levels[2], InlineBreakOpportunityOf::allowed()),
        atomic(7, 40.0, levels[3], InlineBreakOpportunityOf::prohibited()),
    ]
}

#[test]
fn fri06_c08_float_line_mixed_bidi_continuation_uses_visual_placement() {
    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c08_float_line_control_batch(
            fri06_c08_float_line_continuation_input::<S>(20.0, [0, 1, 2, 1]),
            62.0,
        );

        assert_eq!(
            [5, 6].map(|node| public_flow_output(batch.unrounded_entries(), node)
                .location
                .x),
            [S::from_f64(78.0), S::from_f64(42.0)],
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_fractional_terminal_extent_uses_resolved_geometry() {
    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c08_float_line_control_batch(
            fri06_c08_float_line_continuation_input::<S>(20.25, [0; 4]),
            62.0,
        );

        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 0).size.height,
            S::from_f64(62.25),
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_pure_text_keeps_integral_float_height_at_62() {
    fn assert_lane<S: LayoutScalar>() {
        assert_fri06_c08_float_line_control_height::<S>(vec![(
            3,
            fri06_c03_text_input::<S>(vec![fri06_c02_segment_with_metrics::<S>(
                810, 40.0, 14.8, 5.3,
            )]),
            NodeInputOf::non_box(),
        )]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_all_atomic_keeps_integral_float_height_at_62() {
    fn assert_lane<S: LayoutScalar>() {
        let atomic: NodeInputOf<S> = fri06_c03_atomic_style(
            40.0,
            20.2,
            0.0,
            0.0,
            0,
            InlineBreakOpportunityOf::prohibited(),
        );
        assert_fri06_c08_float_line_control_height::<S>(vec![(
            3,
            LayoutInputOf::box_input(atomic.clone()),
            atomic,
        )]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_empty_inline_keeps_integral_float_height_at_62() {
    fn assert_lane<S: LayoutScalar>() {
        let metrics =
            InlineMetricsOf::from_ascent_descent(S::from_f64(7.8), S::from_f64(2.3)).unwrap();
        assert_fri06_c08_float_line_control_height::<S>(vec![
            (
                3,
                fri06_c03_text_input::<S>(vec![fri06_c02_segment_with_metrics::<S>(
                    812, 1.0, 8.0, 2.0,
                )]),
                NodeInputOf::non_box(),
            ),
            (
                4,
                LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                NodeInputOf::non_box(),
            ),
        ]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_nonterminal_mixed_inline_keeps_integral_float_height_at_62() {
    fn assert_lane<S: LayoutScalar>() {
        let atomic: NodeInputOf<S> = fri06_c03_atomic_style(
            28.0,
            16.0,
            0.0,
            0.0,
            0,
            InlineBreakOpportunityOf::prohibited(),
        );
        let terminal_block = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(10.0)),
                PreferredSizeOf::px(S::from_f64(1.0)),
            ),
            ..NodeInputOf::default()
        };
        assert_fri06_c08_float_line_control_height::<S>(vec![
            (
                3,
                fri06_c03_text_input::<S>(vec![fri06_c02_segment_with_metrics::<S>(
                    811, 40.0, 14.8, 5.2,
                )]),
                NodeInputOf::non_box(),
            ),
            (4, LayoutInputOf::box_input(atomic.clone()), atomic),
            (
                5,
                LayoutInputOf::box_input(terminal_block.clone()),
                terminal_block,
            ),
        ]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_horizontal_forced_break_fallback_expands_each_line_envelope() {
    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c12_t08_forced_break_fallback_batch::<S>(15.0);

        for (parent_node, first_atomic, second_atomic, block_start) in
            [(1, 2, 4, 0.0), (5, 6, 8, 42.0), (9, 10, 12, 84.0)]
        {
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), parent_node)
                    .size
                    .height,
                S::from_f64(42.0),
                "each completed parent publishes two exact 21px line envelopes",
            );
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), parent_node)
                    .location
                    .y,
                S::from_f64(block_start),
                "parent block progression consumes the exact completed envelope",
            );
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), first_atomic)
                    .location
                    .y,
                S::ZERO,
                "the first atomic starts at the actual completed line envelope",
            );
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), second_atomic)
                    .location
                    .y,
                S::from_f64(21.0),
                "the second fallback envelope starts after exactly one completed line",
            );
        }
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 0).size.height,
            S::from_f64(126.0),
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 0).size.height,
            S::from_f64(126.0),
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_ltr_post_exclusion_continuation_restarts_at_line_start() {
    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c08_float_line_control_batch(
            fri06_c08_float_line_continuation_input::<S>(20.0, [0; 4]),
            62.0,
        );

        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 7).location.x,
            S::ZERO,
        );
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 6).location.x,
            S::from_f64(74.0),
            "the nearest in-exclusion continuation remains unchanged",
        );
        assert_eq!(
            public_flow_output(
                fri06_c08_float_line_control_batch(
                    fri06_c08_float_line_continuation_input::<S>(20.25, [0; 4]),
                    62.0,
                )
                .unrounded_entries(),
                0,
            )
            .size
            .height,
            S::from_f64(62.25),
            "fractional terminal line geometry remains independent of continuation placement",
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_terminal_line_phase_survives_both_inline_progressions() {
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Ltr, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Ltr, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Rtl, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Rtl, BoxSizing::BorderBox);
}

fn fri06_c08_recovery_characterization_batch<S: LayoutScalar>(
    children: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    root_input: NodeInputOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let child_nodes = children
        .iter()
        .map(|(node, _, _)| *node)
        .collect::<Vec<_>>();
    let mut inputs = HashMap::from([(0, LayoutInputOf::box_input(root_input.clone()))]);
    let mut tree_children = HashMap::from([(0, child_nodes)]);
    for (node, layout_input, _node_input) in children {
        inputs.insert(node, layout_input);
        tree_children.insert(node, Vec::new());
    }
    let tree = public_layout_tree(inputs, tree_children);
    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn fri06_c08_recovery_characterization_segment<S: LayoutScalar>(
    id: u64,
    inline_extent: f64,
    baseline: f64,
    line_height: f64,
    bidi_level: u8,
    following_break: InlineBreakOpportunityOf<S>,
) -> ShapedInlineSegmentOf<S> {
    ShapedInlineSegmentOf::try_new(
        InlineSegmentId::new(id),
        S::from_f64(inline_extent),
        InlineMetricsOf::from_line_height_and_baseline(
            S::from_f64(line_height),
            S::from_f64(baseline),
        )
        .unwrap(),
        BidiLevel::try_new(bidi_level).unwrap(),
        InlineWhitespaceEdge::Preserve,
        following_break,
    )
    .unwrap()
}

#[test]
fn fri06_c08_recovery_characterization_exact_public_inputs_cover_both_scalar_lanes() {
    fn assert_percentage_lane<S: LayoutScalar>() {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            let text = |id, extent| {
                fri06_c08_recovery_characterization_segment(
                    id,
                    extent,
                    14.8,
                    20.0,
                    1,
                    InlineBreakOpportunityOf::prohibited(),
                )
            };
            let atomic = NodeInputOf {
                display: Display::InlineBlock,
                direction: Direction::Rtl,
                box_sizing,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::percent(S::from_f64(0.5)),
                ),
                atomic_inline_participation: Some(fri06_c03_atomic_participation(
                    1,
                    InlineBreakOpportunityOf::prohibited(),
                )),
                ..NodeInputOf::default()
            };
            let root = NodeInputOf {
                display: Display::Block,
                direction: Direction::Rtl,
                box_sizing,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(180.0)),
                    PreferredSizeOf::px(S::from_f64(80.0)),
                ),
                ..NodeInputOf::default()
            };
            let batch = fri06_c08_recovery_characterization_batch(
                vec![
                    (
                        1,
                        fri06_c03_text_input(vec![text(0, 57.796875)]),
                        NodeInputOf::non_box(),
                    ),
                    (2, LayoutInputOf::box_input(atomic.clone()), atomic),
                    (
                        3,
                        fri06_c03_text_input(vec![text(2, 86.703125)]),
                        NodeInputOf::non_box(),
                    ),
                ],
                root,
            );

            assert_eq!(
                fri06_c02_final_node(&batch, 0).size,
                Size::new(S::from_f64(180.0), S::from_f64(80.0))
            );
            let atomic = fri06_c02_final_node(&batch, 2);
            let fragments = batch.unrounded_inline_fragments();
            assert_eq!(fragments.len(), 2);
            assert_eq!(
                (
                    fragments[0].fragment().baseline().x,
                    atomic.location.x,
                    fragments[1].fragment().baseline().x,
                ),
                (
                    S::from_f64(73.296875),
                    S::from_f64(73.0),
                    S::from_f64(180.0),
                ),
                "{box_sizing:?} direct RTL visual starts"
            );
            assert_eq!(atomic.source_index, SourceIndex::new(1));
            assert_eq!(
                (atomic.location, atomic.size, atomic.content_size),
                (
                    Point::new(S::from_f64(73.0), S::ZERO),
                    Size::new(S::from_f64(20.0), S::from_f64(40.0)),
                    Size::new(S::from_f64(20.0), S::from_f64(40.0)),
                ),
                "{box_sizing:?} rounded percentage atomic geometry"
            );
            for (index, (node, id, x, width, baseline_x)) in [
                (1, 0, 15.5, 57.796875, 73.296875),
                (3, 2, 93.296875, 86.703125, 180.0),
            ]
            .into_iter()
            .enumerate()
            {
                let entry = fragments[index];
                let fragment = entry.fragment();
                assert_eq!(
                    (entry.node(), fragment.segment_id()),
                    (node, InlineSegmentId::new(id))
                );
                assert_eq!(fragment.line_index(), 0);
                assert_eq!(
                    fragment.rect(),
                    ScrollRectOf::try_new(
                        Point::new(S::from_f64(x), S::from_f64(25.2)),
                        Size::new(S::from_f64(width), S::from_f64(20.0)),
                    )
                    .unwrap()
                );
                assert_eq!(
                    fragment.baseline(),
                    Point::new(S::from_f64(baseline_x), S::from_f64(40.0))
                );
            }
        }
    }

    fn assert_vertical_lane<S: LayoutScalar>() {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                let axes = FlowAxes::new(WritingMode::VerticalRl, direction);
                let bidi_level = u8::from(direction == Direction::Rtl);
                let box_input = |display, width, height, float, clear, participation| NodeInputOf {
                    display,
                    writing_mode: WritingMode::VerticalRl,
                    direction,
                    box_sizing,
                    float,
                    clear,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(width)),
                        PreferredSizeOf::px(S::from_f64(height)),
                    ),
                    atomic_inline_participation: participation,
                    ..NodeInputOf::default()
                };
                let participation = || {
                    Some(fri06_c03_atomic_participation(
                        bidi_level,
                        InlineBreakOpportunityOf::prohibited(),
                    ))
                };
                let floating =
                    box_input(Display::Block, 28.0, 36.0, Float::Left, Clear::None, None);
                let first = box_input(
                    Display::InlineBlock,
                    18.0,
                    42.0,
                    Float::None,
                    Clear::None,
                    participation(),
                );
                let second = box_input(
                    Display::InlineBlock,
                    18.0,
                    30.0,
                    Float::None,
                    Clear::None,
                    participation(),
                );
                let cleared = box_input(Display::Block, 18.0, 18.0, Float::None, Clear::Left, None);
                let line_break = LineBreakInputOf::new()
                    .with_writing_mode(WritingMode::VerticalRl)
                    .with_direction(direction)
                    .with_metrics(
                        InlineMetricsOf::from_line_height_and_baseline(
                            S::from_f64(24.0),
                            S::from_f64(16.8),
                        )
                        .unwrap(),
                    );
                let root = NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalRl,
                    direction,
                    box_sizing,
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(15.0)).unwrap(),
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(96.0)),
                        PreferredSizeOf::px(S::from_f64(140.0)),
                    ),
                    ..NodeInputOf::default()
                };
                let batch = fri06_c08_recovery_characterization_batch(
                    vec![
                        (1, LayoutInputOf::box_input(floating.clone()), floating),
                        (2, LayoutInputOf::box_input(first.clone()), first),
                        (
                            3,
                            LayoutInputOf::line_break(line_break),
                            NodeInputOf::non_box(),
                        ),
                        (4, LayoutInputOf::box_input(second.clone()), second),
                        (5, LayoutInputOf::box_input(cleared.clone()), cleared),
                    ],
                    root,
                );
                let (float_y, first_y, second_y, clear_y) = match direction {
                    Direction::Ltr => (0.0, 36.0, 36.0, 0.0),
                    Direction::Rtl => (104.0, 62.0, 74.0, 122.0),
                };
                assert_eq!(
                    fri06_c02_final_node(&batch, 0).size,
                    Size::new(S::from_f64(96.0), S::from_f64(140.0))
                );
                let mut mismatches = Vec::new();
                for (node, x, y, width, height) in [
                    (1, 68.0, float_y, 28.0, 36.0),
                    (2, 75.0, first_y, 18.0, 42.0),
                    (4, 51.0, second_y, 18.0, 30.0),
                    (5, 30.0, clear_y, 18.0, 18.0),
                ] {
                    let output = fri06_c02_final_node(&batch, node);
                    let expected = (
                        Point::new(S::from_f64(x), S::from_f64(y)),
                        Size::new(S::from_f64(width), S::from_f64(height)),
                    );
                    if (output.location, output.size) != expected {
                        mismatches.push(format!(
                            "source {node}: expected {expected:?}, got {:?}",
                            (output.location, output.size)
                        ));
                    }
                }
                let control = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 3)
                    .expect("forced break publishes unrounded geometry")
                    .output();
                assert_eq!(control.size, Size::ZERO);
                let control_block = axes
                    .logical_point(
                        control.location,
                        control.size,
                        Size::new(S::from_f64(96.0), S::from_f64(140.0)),
                    )
                    .block;
                if (control_block - S::from_f64(16.8)).abs() > S::from_f64(0.000_1) {
                    mismatches.push(format!(
                        "forced-break metric baseline: expected 16.8, got {control_block:?}"
                    ));
                }
                let clear_block = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 5)
                    .map(|entry| {
                        let output = entry.output();
                        axes.logical_point(
                            output.location,
                            output.size,
                            Size::new(S::from_f64(96.0), S::from_f64(140.0)),
                        )
                        .block
                    })
                    .expect("cleared block publishes unrounded geometry");
                if clear_block != S::from_f64(48.0) {
                    mismatches.push(format!(
                        "two resolved 24px line bands: expected 48, got {clear_block:?}"
                    ));
                }
                assert!(
                    mismatches.is_empty(),
                    "{axes:?} {box_sizing:?} vertical line-band mismatches:\n{}",
                    mismatches.join("\n")
                );
            }
        }
    }

    fn assert_float_lane<S: LayoutScalar>() {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                let axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
                let bidi_level = u8::from(direction == Direction::Rtl);
                let floating = |physical_left: bool, width, height| {
                    let side = match (direction, physical_left) {
                        (Direction::Ltr, true) | (Direction::Rtl, false) => Float::Left,
                        (Direction::Ltr, false) | (Direction::Rtl, true) => Float::Right,
                    };
                    let mut input = fri06_c04_line_box(
                        axes,
                        LogicalSizeOf::new(S::from_f64(width), S::from_f64(height)),
                        side,
                        None,
                    );
                    input.box_sizing = box_sizing;
                    input
                };
                let atomic = |width, following_break| {
                    let mut input = fri06_c04_line_box(
                        axes,
                        LogicalSizeOf::new(S::from_f64(width), S::from_f64(16.0)),
                        Float::None,
                        Some(fri06_c03_atomic_participation(bidi_level, following_break)),
                    );
                    input.box_sizing = box_sizing;
                    input
                };
                let left = floating(true, 42.0, 42.0);
                let right = floating(false, 50.0, 62.0);
                let atomics = [
                    atomic(28.0, InlineBreakOpportunityOf::allowed()),
                    atomic(32.0, InlineBreakOpportunityOf::allowed()),
                    atomic(36.0, InlineBreakOpportunityOf::allowed()),
                    atomic(40.0, InlineBreakOpportunityOf::prohibited()),
                ];
                let segment = fri06_c08_recovery_characterization_segment(
                    4,
                    38.53125,
                    14.8,
                    20.0,
                    bidi_level,
                    InlineBreakOpportunityOf::allowed(),
                );
                let line_strut = InlineBoundaryInputOf::new(
                    InlineBoundaryKind::Start,
                    InlineMetricsOf::from_line_height_and_baseline(
                        S::from_f64(20.0),
                        S::from_f64(12.0),
                    )
                    .unwrap(),
                )
                .with_writing_mode(WritingMode::HorizontalTb)
                .with_direction(direction);
                let root = NodeInputOf {
                    display: Display::Block,
                    direction,
                    box_sizing,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(180.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                };
                let batch = fri06_c08_recovery_characterization_batch(
                    vec![
                        (1, LayoutInputOf::box_input(left.clone()), left),
                        (2, LayoutInputOf::box_input(right.clone()), right),
                        (
                            3,
                            fri06_c03_text_input(vec![segment]),
                            NodeInputOf::non_box(),
                        ),
                        (
                            8,
                            LayoutInputOf::inline_boundary(line_strut),
                            NodeInputOf::non_box(),
                        ),
                        (
                            4,
                            LayoutInputOf::box_input(atomics[0].clone()),
                            atomics[0].clone(),
                        ),
                        (
                            5,
                            LayoutInputOf::box_input(atomics[1].clone()),
                            atomics[1].clone(),
                        ),
                        (
                            6,
                            LayoutInputOf::box_input(atomics[2].clone()),
                            atomics[2].clone(),
                        ),
                        (
                            7,
                            LayoutInputOf::box_input(atomics[3].clone()),
                            atomics[3].clone(),
                        ),
                    ],
                    root,
                );
                assert_eq!(
                    public_flow_output(batch.unrounded_entries(), 0).size,
                    Size::new(S::from_f64(180.0), S::from_f64(62.5))
                );
                assert_eq!(
                    fri06_c02_final_node(&batch, 0).size,
                    Size::new(S::from_f64(180.0), S::from_f64(63.0))
                );
                for (node, x, y, width, height) in
                    [(1, 0.0, 0.0, 42.0, 42.0), (2, 130.0, 0.0, 50.0, 62.0)]
                {
                    let output = fri06_c02_final_node(&batch, node);
                    assert_eq!(
                        (output.location, output.size),
                        (
                            Point::new(S::from_f64(x), S::from_f64(y)),
                            Size::new(S::from_f64(width), S::from_f64(height))
                        )
                    );
                }
                let atomic_x = match direction {
                    Direction::Ltr => [81.0, 42.0, 74.0, 0.0],
                    Direction::Rtl => [102.0, 62.0, 94.0, 90.0],
                };
                for (index, (node, width, y)) in [
                    (4, 28.0, 0.0),
                    (5, 32.0, 21.0),
                    (6, 36.0, 21.0),
                    (7, 40.0, 42.0),
                ]
                .into_iter()
                .enumerate()
                {
                    let output = fri06_c02_final_node(&batch, node);
                    assert_eq!(
                        (output.location, output.size),
                        (
                            Point::new(S::from_f64(atomic_x[index]), S::from_f64(y)),
                            Size::new(S::from_f64(width), S::from_f64(16.0))
                        ),
                        "{axes:?} {box_sizing:?} atomic {node}"
                    );
                }
                for (node, expected) in [4, 5, 6, 7].into_iter().zip([0.0, 21.2, 21.2, 42.0]) {
                    let actual = public_flow_output(batch.unrounded_entries(), node)
                        .location
                        .y;
                    assert!(
                        (actual - S::from_f64(expected)).abs() <= S::from_f64(0.000_1),
                        "{axes:?} {box_sizing:?} unrounded continuation phase: \
                         node {node} expected {expected}, got {actual:?}"
                    );
                }
                let fragment = batch.unrounded_inline_fragments()[0].fragment();
                let (x, baseline_x) = match direction {
                    Direction::Ltr => (42.0, 42.0),
                    Direction::Rtl => (63.46875, 102.0),
                };
                assert_eq!(fragment.line_index(), 0);
                assert_eq!(fragment.rect().origin().x, S::from_f64(x));
                assert!(
                    (fragment.rect().origin().y - S::from_f64(1.2)).abs() <= S::from_f64(0.000_1)
                );
                assert_eq!(
                    fragment.rect().size(),
                    Size::new(S::from_f64(38.53125), S::from_f64(20.0))
                );
                assert_eq!(fragment.baseline().x, S::from_f64(baseline_x));
                assert!((fragment.baseline().y - S::from_f64(16.0)).abs() <= S::from_f64(0.000_1));
                if direction == Direction::Rtl {
                    let first_atomic = public_flow_output(batch.unrounded_entries(), 4);
                    assert_eq!(
                        [
                            fragment.rect().origin().x + fragment.rect().size().width,
                            first_atomic.location.x + first_atomic.size.width,
                        ],
                        [S::from_f64(102.0), S::from_f64(130.0)],
                        "logical float-band starts 78/50 project once to physical Range starts 102/130",
                    );
                }
            }
        }
    }

    assert_percentage_lane::<f32>();
    assert_percentage_lane::<f64>();
    assert_vertical_lane::<f32>();
    assert_vertical_lane::<f64>();
    assert_float_lane::<f32>();
    assert_float_lane::<f64>();
}

#[test]
fn fri06_c07_sideways_lr_rtl_all_atomic_odd_bidi_keeps_visual_placement_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl);
        let atomic = |node, inline_extent| {
            let style = fri06_c04_line_box(
                flow_axes,
                LogicalSizeOf::new(S::from_f64(inline_extent), S::from_f64(20.0)),
                Float::None,
                Some(fri06_c03_atomic_participation(
                    1,
                    InlineBreakOpportunityOf::prohibited(),
                )),
            );
            (node, LayoutInputOf::box_input(style.clone()), style)
        };
        let batch = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Auto,
            vec![atomic(1, 10.0), atomic(2, 20.0)],
        );
        let first_source = fri06_c02_final_node(&batch, 1);
        let second_source = fri06_c02_final_node(&batch, 2);

        assert_eq!(
            first_source.size,
            Size::new(S::from_f64(20.0), S::from_f64(10.0))
        );
        assert_eq!(
            second_source.size,
            Size::new(S::from_f64(20.0), S::from_f64(20.0))
        );
        assert_eq!(first_source.location.y, S::from_f64(20.0));
        assert_eq!(second_source.location.y, S::ZERO);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_bidi_nested_equal_levels_reset_per_line_and_keep_discarded_slot_gaps_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let nested = fri06_c02_text_batch(
            [0, 1, 2, 2, 1, 0]
                .into_iter()
                .enumerate()
                .map(|(index, level)| {
                    fri06_c02_segment_with_level(
                        u64::try_from(index + 1).unwrap(),
                        10.0,
                        level,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )
                })
                .collect(),
            AvailableOf::definite(S::from_f64(100.0)),
        );
        let nested_fragments = nested.final_inline_fragments();
        assert_eq!(
            nested_fragments
                .iter()
                .map(|entry| entry.fragment().segment_id().get())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6],
            "public fragment identity stays in source segment order"
        );
        assert_eq!(
            nested_fragments
                .iter()
                .map(|entry| entry.fragment().visual_index())
                .collect::<Vec<_>>(),
            vec![0, 4, 2, 3, 1, 5]
        );
        assert_eq!(
            nested_fragments
                .iter()
                .map(|entry| entry.fragment().rect().origin().x)
                .collect::<Vec<_>>(),
            [0.0, 40.0, 20.0, 30.0, 10.0, 50.0]
                .map(S::from_f64)
                .to_vec()
        );

        let wrapped = fri06_c02_text_batch(
            (0..4)
                .map(|index| {
                    fri06_c02_segment_with_level(
                        10 + index,
                        10.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        if index == 1 {
                            InlineBreakOpportunityOf::mandatory()
                        } else {
                            InlineBreakOpportunityOf::prohibited()
                        },
                    )
                })
                .collect(),
            AvailableOf::definite(S::from_f64(100.0)),
        );
        assert_eq!(
            wrapped
                .final_inline_fragments()
                .iter()
                .map(|entry| (
                    entry.fragment().line_index(),
                    entry.fragment().visual_index(),
                    entry.fragment().rect().origin(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, 1, Point::new(S::from_f64(10.0), S::ZERO)),
                (0, 0, Point::new(S::ZERO, S::ZERO)),
                (1, 1, Point::new(S::from_f64(10.0), S::from_f64(10.0))),
                (1, 0, Point::new(S::ZERO, S::from_f64(10.0))),
            ]
        );

        let discarded = fri06_c02_text_batch(
            vec![
                fri06_c02_segment_with_level(
                    20,
                    5.0,
                    0,
                    InlineWhitespaceEdge::DiscardAtLineStart,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                fri06_c02_segment_with_level(
                    21,
                    10.0,
                    0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                fri06_c02_segment_with_level(
                    22,
                    5.0,
                    0,
                    InlineWhitespaceEdge::DiscardAtLineEnd,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
        );
        let fragments = discarded.final_inline_fragments();
        assert_eq!(fragments.len(), 1);
        assert_eq!(
            fragments[0].fragment().segment_id(),
            InlineSegmentId::new(21)
        );
        assert_eq!(fragments[0].fragment().visual_index(), 1);
        assert_eq!(fragments[0].fragment().rect().origin(), Point::ZERO);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_break_latest_replacement_mandatory_final_and_overwide_progress_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let replacement =
            InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(5.0)).unwrap();
        let selected = vec![
            fri06_c02_segment(
                1,
                15.0,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::allowed(),
            ),
            fri06_c02_segment(2, 20.0, InlineWhitespaceEdge::Preserve, replacement),
            fri06_c02_segment(
                3,
                20.0,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            ),
        ];
        let first =
            fri06_c02_text_batch(selected.clone(), AvailableOf::definite(S::from_f64(40.0)));
        let second = fri06_c02_text_batch(selected, AvailableOf::definite(S::from_f64(40.0)));
        assert_eq!(first.final_entries(), second.final_entries());
        assert_eq!(
            first.final_inline_fragments(),
            second.final_inline_fragments()
        );
        let fragments = first.final_inline_fragments();
        assert_eq!(
            fragments
                .iter()
                .map(|entry| entry.fragment().line_index())
                .collect::<Vec<_>>(),
            vec![0, 0, 1]
        );
        assert_eq!(fragments[0].fragment().replacement_inline_extent(), None);
        assert_eq!(
            fragments[1].fragment().replacement_inline_extent(),
            Some(S::from_f64(5.0))
        );

        let unselected = fri06_c02_text_batch(
            vec![
                fri06_c02_segment(
                    1,
                    15.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::allowed(),
                ),
                fri06_c02_segment(2, 20.0, InlineWhitespaceEdge::Preserve, replacement),
                fri06_c02_segment(
                    3,
                    20.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
        );
        assert!(
            unselected
                .final_inline_fragments()
                .iter()
                .all(|entry| entry.fragment().replacement_inline_extent().is_none())
        );

        let mandatory = fri06_c02_text_batch(
            vec![
                fri06_c02_segment(
                    4,
                    10.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::mandatory(),
                ),
                fri06_c02_segment(
                    5,
                    12.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::mandatory(),
                ),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
        );
        assert_eq!(
            mandatory.final_inline_fragments()[0]
                .fragment()
                .line_index(),
            0
        );
        assert_eq!(
            mandatory.final_inline_fragments()[1]
                .fragment()
                .line_index(),
            1
        );
        assert_eq!(
            fri06_c02_final_node(&mandatory, 0).size.height,
            S::from_f64(30.0)
        );

        let overwide = fri06_c02_text_batch(
            vec![fri06_c02_segment(
                6,
                60.0,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            )],
            AvailableOf::definite(S::from_f64(40.0)),
        );
        assert_eq!(overwide.final_inline_fragments().len(), 1);
        assert_eq!(
            overwide.final_inline_fragments()[0]
                .fragment()
                .rect()
                .size()
                .width,
            S::from_f64(60.0)
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_whitespace_discards_every_edge_mode_and_retains_empty_source_anchor_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c02_text_batch(
            vec![
                fri06_c02_segment(
                    1,
                    5.0,
                    InlineWhitespaceEdge::DiscardAtLineStart,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                fri06_c02_segment(
                    2,
                    10.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                fri06_c02_segment(
                    3,
                    5.0,
                    InlineWhitespaceEdge::DiscardAtLineEnd,
                    InlineBreakOpportunityOf::allowed(),
                ),
                fri06_c02_segment(
                    4,
                    5.0,
                    InlineWhitespaceEdge::DiscardAtBoth,
                    InlineBreakOpportunityOf::allowed(),
                ),
                fri06_c02_segment(
                    5,
                    10.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ],
            AvailableOf::definite(S::from_f64(20.0)),
        );
        let fragments = batch.final_inline_fragments();
        assert_eq!(
            fragments
                .iter()
                .map(|entry| entry.fragment().segment_id().get())
                .collect::<Vec<_>>(),
            vec![2, 5]
        );
        assert_eq!(fragments[0].fragment().line_index(), 0);
        assert_eq!(fragments[1].fragment().line_index(), 1);

        let empty = fri06_c02_text_batch(
            vec![fri06_c02_segment(
                7,
                9.0,
                InlineWhitespaceEdge::DiscardAtBoth,
                InlineBreakOpportunityOf::mandatory(),
            )],
            AvailableOf::definite(S::from_f64(20.0)),
        );
        assert!(empty.final_inline_fragments().is_empty());
        let text = fri06_c02_final_node(&empty, 1);
        assert_eq!(text.source_index, SourceIndex::ZERO);
        assert_eq!(text.location, Point::ZERO);
        assert_eq!(text.size, Size::ZERO);
        assert_eq!(text.content_size, Size::ZERO);
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_intrinsic_reports_exact_min_and_max_content_in_both_scalar_lanes() {
    fn segments<S: LayoutScalar>() -> Vec<ShapedInlineSegmentOf<S>> {
        vec![
            fri06_c02_segment(
                1,
                12.0,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            ),
            fri06_c02_segment(
                2,
                8.0,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(4.0)).unwrap(),
            ),
            fri06_c02_segment(
                3,
                30.0,
                InlineWhitespaceEdge::DiscardAtBoth,
                InlineBreakOpportunityOf::allowed(),
            ),
            fri06_c02_segment(
                4,
                7.0,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::mandatory(),
            ),
            fri06_c02_segment(
                5,
                15.0,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            ),
        ]
    }
    fn assert_lane<S: LayoutScalar>() {
        let min = fri06_c02_text_batch(segments::<S>(), AvailableOf::MIN_CONTENT);
        let max = fri06_c02_text_batch(segments::<S>(), AvailableOf::MAX_CONTENT);
        assert_eq!(fri06_c02_final_node(&min, 0).size.width, S::from_f64(24.0));
        assert_eq!(fri06_c02_final_node(&max, 0).size.width, S::from_f64(57.0));
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_block_text_adjacent_nodes_share_soft_mandatory_and_bidi_lines_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let root = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let soft = fri06_c02_text_nodes_batch(
            vec![
                (
                    1,
                    vec![fri06_c02_segment_with_level(
                        1,
                        15.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::allowed(),
                    )],
                ),
                (
                    2,
                    vec![fri06_c02_segment_with_level(
                        2,
                        15.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )],
                ),
            ],
            root.clone(),
            Size::new(
                AvailableOf::definite(S::from_f64(20.0)),
                AvailableOf::MAX_CONTENT,
            ),
        );
        assert_eq!(
            soft.final_inline_fragments()
                .iter()
                .map(|entry| (entry.node(), entry.fragment().line_index()))
                .collect::<Vec<_>>(),
            vec![(1, 0), (2, 1)]
        );

        let shared_bidi = fri06_c02_text_nodes_batch(
            vec![
                (
                    1,
                    vec![fri06_c02_segment_with_level(
                        3,
                        10.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )],
                ),
                (
                    2,
                    vec![fri06_c02_segment_with_level(
                        4,
                        10.0,
                        1,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::mandatory(),
                    )],
                ),
            ],
            root,
            Size::new(
                AvailableOf::definite(S::from_f64(40.0)),
                AvailableOf::MAX_CONTENT,
            ),
        );
        assert_eq!(
            shared_bidi
                .final_inline_fragments()
                .iter()
                .map(|entry| (
                    entry.node(),
                    entry.fragment().line_index(),
                    entry.fragment().visual_index()
                ))
                .collect::<Vec<_>>(),
            vec![(1, 0, 1), (2, 0, 0)]
        );

        let mandatory = fri06_c02_text_nodes_batch(
            vec![
                (
                    1,
                    vec![fri06_c02_segment(
                        5,
                        9.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::mandatory(),
                    )],
                ),
                (
                    2,
                    vec![fri06_c02_segment(
                        6,
                        11.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )],
                ),
            ],
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
            Size::new(
                AvailableOf::definite(S::from_f64(40.0)),
                AvailableOf::MAX_CONTENT,
            ),
        );
        assert_eq!(
            mandatory
                .final_inline_fragments()
                .iter()
                .map(|entry| (entry.node(), entry.fragment().line_index()))
                .collect::<Vec<_>>(),
            vec![(1, 0), (2, 1)]
        );

        let intrinsic_nodes = || {
            vec![
                (
                    1,
                    vec![
                        fri06_c02_segment(
                            7,
                            12.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::prohibited(),
                        ),
                        fri06_c02_segment(
                            8,
                            8.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(
                                4.0,
                            ))
                            .unwrap(),
                        ),
                    ],
                ),
                (
                    2,
                    vec![
                        fri06_c02_segment(
                            9,
                            30.0,
                            InlineWhitespaceEdge::DiscardAtBoth,
                            InlineBreakOpportunityOf::allowed(),
                        ),
                        fri06_c02_segment(
                            10,
                            7.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::mandatory(),
                        ),
                    ],
                ),
                (
                    3,
                    vec![fri06_c02_segment(
                        11,
                        15.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )],
                ),
            ]
        };
        let intrinsic_root = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let min = fri06_c02_text_nodes_batch(
            intrinsic_nodes(),
            intrinsic_root.clone(),
            Size::new(AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT),
        );
        let max = fri06_c02_text_nodes_batch(
            intrinsic_nodes(),
            intrinsic_root,
            Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
        );
        assert_eq!(fri06_c02_final_node(&min, 0).size.width, S::from_f64(24.0));
        assert_eq!(fri06_c02_final_node(&max, 0).size.width, S::from_f64(57.0));
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_scroll_uses_fragment_rects_without_replacement_or_full_line_proxy_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let replacement =
            InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(5.0)).unwrap();
        let batch = fri06_c02_text_nodes_batch(
            vec![
                (
                    1,
                    vec![fri06_c02_segment(
                        41,
                        20.0,
                        InlineWhitespaceEdge::Preserve,
                        replacement,
                    )],
                ),
                (
                    2,
                    vec![fri06_c02_segment(
                        42,
                        10.0,
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )],
                ),
            ],
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(22.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
                ..NodeInputOf::default()
            },
            Size::new(
                AvailableOf::definite(S::from_f64(22.0)),
                AvailableOf::definite(S::from_f64(20.0)),
            ),
        );
        let root = fri06_c02_final_node(&batch, 0);
        let range = root.scroll_geometry.unwrap().physical_range();
        assert_eq!(range.x().minimum(), S::ZERO);
        assert_eq!(range.x().maximum(), S::ZERO);
        assert_eq!(range.y().minimum(), S::ZERO);
        assert_eq!(range.y().maximum(), S::ZERO);
        assert_eq!(batch.final_inline_fragments().len(), 2);
        assert_eq!(
            batch.final_inline_fragments()[0]
                .fragment()
                .replacement_inline_extent(),
            Some(S::from_f64(5.0))
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn assert_flex_root_percentage_parent_is_separate_from_host_fill<S: LayoutScalar>() {
    let host = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );

    for writing_mode in [WritingMode::VerticalRl, WritingMode::SidewaysLr] {
        let style = NodeInputOf::<S> {
            writing_mode,
            size: Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO),
            max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::percent(scalar(0.8))),
            padding: Edges::new(
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
            ),
            border: Edges::new(
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };

        for (viewport_height, expected_height, expected_edge) in
            [(210.0, 110.0, 8.0), (100.0, 80.0, 4.0)]
        {
            let viewport = Size::new(
                AvailableOf::definite(scalar(130.0)),
                AvailableOf::definite(scalar(viewport_height)),
            );
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

            assert_eq!(output.location, Point::ZERO);
            assert_eq!(
                output.size,
                Size::new(scalar(20.0), scalar(expected_height))
            );
            assert_eq!(
                output.padding,
                Edges::new(
                    scalar(expected_edge),
                    S::ZERO,
                    scalar(expected_edge),
                    S::ZERO,
                )
            );
            assert_eq!(
                output.border,
                Edges::new(
                    scalar(expected_edge),
                    S::ZERO,
                    scalar(expected_edge),
                    S::ZERO,
                )
            );
        }
    }
}

#[test]
fn flex_root_percentage_parent_separates_host_fill_for_f32() {
    assert_flex_root_percentage_parent_is_separate_from_host_fill::<f32>();
}

#[test]
fn flex_root_percentage_parent_separates_host_fill_for_f64() {
    assert_flex_root_percentage_parent_is_separate_from_host_fill::<f64>();
}

fn overflowing_scroll_edges() -> Edges<Length> {
    Edges {
        left: Length::px(f32::MAX),
        ..Edges::all(Length::ZERO)
    }
}

fn assert_fri06_mr02_geometry_error_leaf_standalone_mapping<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let size = Size::new(largest, S::ONE);
    let style = NodeInputOf {
        padding: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        ..NodeInputOf::default()
    };
    let input = ComputeInputOf::leaf_layout(
        size.map(Some),
        size.map(Some),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        size.map(AvailableOf::definite),
    )
    .unwrap();
    let error = compute_leaf(input, &style, |_measurement| {
        Ok::<_, ()>(Size::new(largest, S::ZERO))
    })
    .expect_err("overflowing standalone measured-content geometry must fail");

    assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry)
    ));
}

#[test]
fn fri06_mr02_geometry_error_leaf_standalone_mapping_remains_unchanged_both_scalars() {
    assert_fri06_mr02_geometry_error_leaf_standalone_mapping::<f32>();
    assert_fri06_mr02_geometry_error_leaf_standalone_mapping::<f64>();
}

#[test]
fn scroll_geometry_error_maps_root_block_overflow_through_the_public_front_door() {
    let tree: RootSessionTree = RootSessionTree::default().style(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            padding: overflowing_scroll_edges(),
            border: overflowing_scroll_edges(),
            ..NodeInput::default()
        },
    );

    assert_public_scroll_geometry_error_without_batch(
        &tree,
        Size::new(Available::definite(f32::MAX), Available::definite(1.0)),
        LayoutErrorSite::Node(0),
        LayoutOperation::RootLayout,
        LayoutInternalInvariant::InvalidRootScrollGeometry,
    );
}

#[test]
fn scroll_geometry_error_maps_non_root_block_overflow_through_the_public_front_door() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(1.0)),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
                padding: overflowing_scroll_edges(),
                border: overflowing_scroll_edges(),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                ..NodeInput::default()
            },
        );

    assert_public_scroll_geometry_error_without_batch(
        &tree,
        Size::new(Available::definite(100.0), Available::definite(1.0)),
        LayoutErrorSite::Node(1),
        LayoutOperation::ChildLayout,
        LayoutInternalInvariant::InvalidBlockScrollGeometry,
    );
}

#[test]
fn scroll_geometry_error_maps_block_inline_float_and_absolute_overflow_to_the_subject() {
    let available = Size::new(Available::definite(100.0), Available::definite(1.0));
    let variants = [
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(fri06_atomic_participation()),
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            float: Float::Left,
            display: Display::Block,
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            position: Position::Absolute,
            display: Display::Block,
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    ];

    for child_style in variants {
        let tree: RootSessionTree = RootSessionTree::default()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(1.0)),
                    ..NodeInput::default()
                },
            )
            .style(1, child_style)
            .measure(1, Ok(Size::new(f32::MAX, 1.0)));

        assert_public_scroll_geometry_error_without_batch(
            &tree,
            available,
            LayoutErrorSite::ContainerSubject {
                container: 0,
                subject: 1,
            },
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        );
    }
}

#[test]
fn root_request_rejects_invalid_definite_availability() {
    let cases = [
        (
            Size::new(Available::definite(-1.0), Available::MAX_CONTENT),
            PhysicalAxis::Horizontal,
            NonNegativeFiniteScalarErrorOf::Negative { value: -1.0 },
        ),
        (
            Size::new(Available::definite(f32::NAN), Available::MAX_CONTENT),
            PhysicalAxis::Horizontal,
            NonNegativeFiniteScalarErrorOf::NonFinite { value: f32::NAN },
        ),
        (
            Size::new(Available::MAX_CONTENT, Available::definite(f32::INFINITY)),
            PhysicalAxis::Vertical,
            NonNegativeFiniteScalarErrorOf::NonFinite {
                value: f32::INFINITY,
            },
        ),
    ];

    for (available, axis, scalar_error) in cases {
        let error = LayoutRootRequest::viewport(available).unwrap_err();

        assert_eq!(error.axis(), axis);
        match (error.scalar(), scalar_error) {
            (
                NonNegativeFiniteScalarErrorOf::Negative { value },
                NonNegativeFiniteScalarErrorOf::Negative { value: expected },
            ) => assert_eq!(value, expected),
            (
                NonNegativeFiniteScalarErrorOf::NonFinite { value },
                NonNegativeFiniteScalarErrorOf::NonFinite { value: expected },
            ) => {
                if expected.is_nan() {
                    assert!(value.is_nan());
                } else {
                    assert_eq!(value, expected);
                }
            }
            (actual, expected) => panic!("expected {expected:?}, got {actual:?}"),
        }
    }

    let valid_viewport = Size::new(Available::definite(100.0), Available::definite(80.0));
    let flex_context = FlexItemRootContext::under_viewport(
        valid_viewport,
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
    )
    .unwrap();
    let error = LayoutRootRequest::flex_item_under_viewport(
        Size::new(Available::definite(-2.0), Available::MAX_CONTENT),
        flex_context,
    )
    .unwrap_err();
    assert_eq!(error.axis(), PhysicalAxis::Horizontal);
    assert_eq!(
        error.scalar(),
        NonNegativeFiniteScalarErrorOf::Negative { value: -2.0 }
    );
}

#[test]
fn compute_layout_stops_after_first_recursive_child_error() {
    let tree = RootSessionTree::default()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .style(2, NodeInput::default())
        .measure(1, Err("first child failed"))
        .measure(2, Ok(Size::new(20.0, 10.0)));
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::Measurement("first child failed")
    );
    assert_eq!(tree.measured_nodes(), vec![1]);
}

#[test]
fn compute_layout_reports_consumed_invalid_numeric_resolution() {
    let invalid_padding =
        LengthPercentageOf::from_coefficients(-f32::MAX, -1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
            padding: Edges::new(
                Length::value(invalid_padding),
                Length::ZERO,
                Length::ZERO,
                Length::ZERO,
            ),
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::NEG_INFINITY,
        })
    );
}

#[test]
fn compute_layout_rejects_overflowing_affine_grid_auto_fit_track() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let track = TrackSizing::from(overflowing);
    let repeat = TrackRepetition::auto_fit(vec![track]).expect("nonempty repeated track list");
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::Repeat(repeat)],
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(20.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
    ));
}

#[test]
fn track_sizing_nested_calculation_produces_track_geometry() {
    let nested = SizingCalculation::min(vec![
        SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite track")),
        SizingCalculation::value(LengthPercentageOf::px(20.0).expect("finite track")),
    ])
    .expect("nonempty minimum");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackSizing::calculation(nested).into()],
                ..NodeInput::default()
            },
        )
        .style(1, NodeInput::default());
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(20.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("nested track calculation resolves");
    let child = public_flow_output(batch.unrounded_entries(), 1);

    assert_eq!(child.location, Point::ZERO);
    assert_eq!(child.size.width, 10.0);
}

#[test]
fn hidden_layout_writes_zero_line_break_output_without_box_compute() {
    #[derive(Default)]
    struct HiddenTree {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutput>,
        caches: HashMap<u32, Cache>,
        inputs: HashMap<u32, LayoutInput>,
        hidden_children: Vec<u32>,
    }

    impl Traverse for HiddenTree {
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

    impl Compute for HiddenTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            self.inputs[&node]
                .as_box()
                .unwrap_or_else(|| panic!("line break node {node} has no box NodeInput"))
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            self.inputs[&node].clone()
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
                assert_eq!(
                    input,
                    ComputeInput::hidden(crate::ContainingLayoutContext::new(
                        crate::geometry::FlowAxes::new(
                            crate::WritingMode::HorizontalTb,
                            crate::Direction::Ltr,
                        ),
                        crate::ParentFormattingContext::NoParent
                    ))
                );
                let _ = self.node_input(node);
                self.hidden_children.push(node);
                ComputeOutput::HIDDEN
            })
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutput> {
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
            output: ComputeOutput,
        ) {
            self.caches
                .get_mut(&node)
                .unwrap()
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.inputs
        .insert(1, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(2, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(3, LayoutInput::line_break(LineBreakInput::new()));
    tree.caches.insert(1, Cache::new());
    tree.caches.insert(2, Cache::new());
    tree.caches.insert(3, Cache::new());

    assert_eq!(
        compute_hidden(
            &mut tree,
            1,
            SourceIndex::ZERO,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            crate::scroll::SettledAutoScrollbarState::INITIAL,
        )
        .unwrap(),
        ComputeOutput::HIDDEN
    );
    assert_eq!(tree.hidden_children, vec![2]);
    assert_eq!(
        tree.layouts[&1],
        NodeOutput::with_source_index(crate::SourceIndex::new(0))
    );
    assert_eq!(
        tree.layouts[&3],
        NodeOutput::with_source_index(crate::SourceIndex::new(1))
    );
    assert!(tree.caches[&1].is_empty());
    assert!(tree.caches[&3].is_empty());
}

#[test]
fn hidden_compute_sets_inline_boundary_children_to_hidden_output() {
    #[derive(Default)]
    struct HiddenTree {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutput>,
        caches: HashMap<u32, Cache>,
        inputs: HashMap<u32, LayoutInput>,
        hidden_children: Vec<u32>,
    }

    impl Traverse for HiddenTree {
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

    impl Compute for HiddenTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            self.inputs[&node]
                .as_box()
                .unwrap_or_else(|| panic!("inline boundary node {node} has no box NodeInput"))
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            self.inputs[&node].clone()
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
                assert_eq!(
                    input,
                    ComputeInput::hidden(crate::ContainingLayoutContext::new(
                        crate::geometry::FlowAxes::new(
                            crate::WritingMode::HorizontalTb,
                            crate::Direction::Ltr,
                        ),
                        crate::ParentFormattingContext::NoParent
                    ))
                );
                let _ = self.node_input(node);
                self.hidden_children.push(node);
                ComputeOutput::HIDDEN
            })
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutput> {
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
            output: ComputeOutput,
        ) {
            self.caches
                .get_mut(&node)
                .unwrap()
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let metrics = InlineMetrics::from_line_height_and_baseline(16.0, 12.0).unwrap();
    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.inputs
        .insert(1, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(2, LayoutInput::box_input(NodeInput::default()));
    tree.inputs.insert(
        3,
        LayoutInput::inline_boundary(InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics)),
    );
    tree.caches.insert(1, Cache::new());
    tree.caches.insert(2, Cache::new());
    tree.caches.insert(3, Cache::new());

    assert_eq!(
        compute_hidden(
            &mut tree,
            1,
            SourceIndex::ZERO,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            crate::scroll::SettledAutoScrollbarState::INITIAL,
        )
        .unwrap(),
        ComputeOutput::HIDDEN
    );
    assert_eq!(tree.hidden_children, vec![2]);
    assert_eq!(
        tree.layouts[&1],
        NodeOutput::with_source_index(crate::SourceIndex::new(0))
    );
    assert_eq!(
        tree.layouts[&3],
        NodeOutput::with_source_index(crate::SourceIndex::new(1))
    );
    assert!(tree.caches[&1].is_empty());
    assert!(tree.caches[&3].is_empty());
}

#[test]
fn f64_tree_can_run_root_layout_smoke_test() {
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new().style(
        0,
        NodeInputOf::<f64> {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::px(50.0)),
            ..NodeInputOf::<f64>::default()
        },
    );

    compute_root(
        &mut tree,
        0,
        Size::new(AvailableOf::definite(100.0), AvailableOf::definite(50.0)),
    )
    .unwrap();

    assert_eq!(
        tree.output(0)
            .expect("root layout must stage output for the root node")
            .size,
        Size::new(100.0, 50.0)
    );
}

#[test]
fn root_layout_emits_scroll_geometry_for_scroll_overflow() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0)),
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollport(),
        ScrollRect::try_new(Point::ZERO, Size::new(90.0, 30.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(40.0, 40.0));
    assert_eq!(
        geometry
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(99.0, -5.0).unwrap()),
        PhysicalScrollOffset::try_new(40.0, 0.0).unwrap()
    );
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
}

#[test]
fn root_layout_emits_visible_scroll_geometry_without_range() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0)),
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), None);
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(130.0, 70.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn root_layout_emits_clip_geometry_without_range() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Clip, Overflow::Clip),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0)),
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn root_scroll_geometry_range_accounts_for_padding_border_and_gutter() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
                scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                padding: Edges::all(Length::px(2.0)),
                border: Edges::all(Length::px(3.0)),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0)),
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollport(),
        ScrollRect::try_new(Point::new(3.0, 3.0), Size::new(84.0, 34.0)).unwrap()
    );
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::new(3.0, 3.0), Size::new(132.0, 72.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(48.0, 38.0));
    assert_eq!(
        geometry
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(99.0, 99.0).unwrap()),
        PhysicalScrollOffset::try_new(48.0, 38.0).unwrap()
    );
}

#[test]
fn root_scroll_geometry_preserves_child_origin_bearing_scrollable_overflow() {
    let child_overflow =
        ScrollRect::try_new(Point::new(-12.0, -4.0), Size::new(160.0, 74.0)).unwrap();
    let child_geometry = root_test_scroll_geometry(RootTestScrollGeometryFacts {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
        item_is_replaced: false,
        border_box_size: Size::new(100.0, 40.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width: 0.0,
        scrollable_overflow: child_overflow,
    });
    let mut output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));
    output.scroll_geometry = Some(child_geometry);
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(1, output);

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
    assert_positive_physical_range(geometry.physical_range(), Size::new(48.0, 30.0));
}

#[test]
fn root_layout_stores_child_output_as_root_layout() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                direction: Direction::Rtl,
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                scrollbar_width: crate::ScrollbarWidthOf::try_new(13.0).unwrap(),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0)),
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(1),
        &[ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(200.0), None),
            Size::new(Some(200.0), Some(100.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Rtl
                ),
                crate::ParentFormattingContext::NoParent
            ),
            Size::new(Available::definite(200.0), Available::definite(100.0))
        )]
    );
    let layout = tree.layout(1).expect("root layout should be stored");
    assert_eq!(layout.location, crate::Point::new(120.0, 0.0));
    assert_eq!(layout.size, Size::new(80.0, 20.0));
    assert_eq!(layout.content_size, Size::new(80.0, 20.0));
    assert_eq!(layout.scrollbar_size(), Size::new(13.0, 13.0));
}

#[test]
fn inline_level_root_keeps_intrinsic_width_under_definite_viewport() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0)),
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(1)
            .first()
            .expect("root should be computed")
            .known(),
        Size::NONE
    );
    assert_eq!(
        tree.layout(1).expect("root layout should be stored").size,
        Size::new(80.0, 20.0)
    );
}

#[test]
fn max_width_root_uses_clamped_available_width_under_definite_viewport() {
    let expected_known = Size::new(Some(260.0), None);
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                max_size: Size::new(MaxSize::px(260.0), MaxSize::NONE),
                ..NodeInput::default()
            },
        )
        .measure_when(
            1,
            OracleMeasurementOf::new(ComputeOutput::from_sizes(
                Size::new(260.0, 72.0),
                Size::new(260.0, 72.0),
            ))
            .known(expected_known),
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(800.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(1)
            .first()
            .expect("root should be computed")
            .known(),
        expected_known
    );
    assert_eq!(
        tree.layout(1).expect("root layout should be stored").size,
        Size::new(260.0, 72.0)
    );
}

#[test]
fn block_root_with_max_width_uses_clamped_available_outer_width() {
    let expected_known = Size::new(Some(272.0), None);
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                box_sizing: BoxSizing::ContentBox,
                max_size: Size::new(MaxSize::px(260.0), MaxSize::NONE),
                padding: Edges::new(
                    Length::px(1.0),
                    Length::px(5.0),
                    Length::px(1.0),
                    Length::px(5.0),
                ),
                border: Edges::all(Length::px(1.0)),
                ..NodeInput::default()
            },
        )
        .measure_when(
            1,
            OracleMeasurementOf::new(ComputeOutput::from_sizes(
                Size::new(272.0, 20.0),
                Size::new(272.0, 20.0),
            ))
            .known(expected_known),
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(800.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(1)
            .first()
            .expect("root should be computed")
            .known()
            .width,
        Some(272.0)
    );
    assert_eq!(
        tree.layout(1)
            .expect("root layout should be stored")
            .size
            .width,
        272.0
    );
}

#[test]
fn fri05_c03_root_geometry_fallback_roots_seed_their_own_padding_box() {
    let style = NodeInput {
        display: Display::Flex,
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        padding: Edges::all(Length::px(10.0)),
        ..NodeInput::default()
    };
    let available = Size::splat(Available::definite(100.0));
    let parent_flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let flex_context = FlexItemRootContext::under_viewport(available, parent_flow_axes).unwrap();
    let requests = [
        (
            "viewport",
            LayoutRootRequest::viewport(available).expect("viewport request is valid"),
        ),
        (
            "flex-item",
            LayoutRootRequest::flex_item_under_viewport(available, flex_context)
                .expect("flex-item root request is valid"),
        ),
    ];

    for (root_kind, request) in requests {
        let tree = PublicFlowTree::default()
            .with_children(0, [])
            .with_style(0, style.clone());
        let batch = compute_layout(&tree, 0, request).expect("empty flex root lays out");
        assert_eq!(batch.unrounded_entries().len(), 1, "{root_kind}");
        assert_eq!(batch.final_entries().len(), 1, "{root_kind}");

        for (phase, output) in [
            ("unrounded", batch.unrounded_entries()[0].output()),
            ("rounded", batch.final_entries()[0].output()),
        ] {
            let geometry = output
                .scroll_geometry
                .expect("a performed fallback root has canonical geometry");
            let expected_padding_box =
                ScrollRect::try_new(Point::ZERO, Size::splat(100.0)).unwrap();
            let expected_content_box =
                ScrollRect::try_new(Point::new(10.0, 10.0), Size::splat(80.0)).unwrap();

            assert_eq!(output.size, Size::splat(100.0), "{root_kind}/{phase}");
            assert_eq!(
                geometry.padding_box(),
                expected_padding_box,
                "{root_kind}/{phase}"
            );
            assert_eq!(
                geometry.content_box(),
                expected_content_box,
                "{root_kind}/{phase}"
            );
            assert_eq!(
                geometry.scrollable_overflow(),
                expected_padding_box,
                "{root_kind}/{phase}"
            );
            assert_eq!(
                (
                    geometry.physical_range().x().minimum(),
                    geometry.physical_range().x().maximum(),
                    geometry.physical_range().y().minimum(),
                    geometry.physical_range().y().maximum(),
                ),
                (0.0, 0.0, 0.0, 0.0),
                "{root_kind}/{phase}"
            );
            assert_eq!(
                output.content_box_size(),
                expected_content_box.size(),
                "{root_kind}/{phase}"
            );
            assert_eq!(output.scrollbar_size(), Size::ZERO, "{root_kind}/{phase}");
            assert_eq!(
                geometry.target().border_box(),
                geometry.border_box(),
                "{root_kind}/{phase}"
            );
        }
    }
}

#[test]
fn fri05_c03_root_geometry_viewport_flex_rebuilds_complete_source_and_target() {
    let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let scroll_padding = ScrollPadding::new(
        ScrollPaddingValue::value(LengthPercentageOf::px(2.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(4.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(3.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(1.0).unwrap()),
    );
    let scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let tree = PublicFlowTree::default().with_children(0, []).with_style(
        0,
        NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
            overflow_clip_margin: OverflowClipMargin::try_new(OverflowClipBox::BorderBox, 5.0)
                .unwrap(),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(6.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            scroll_padding,
            scroll_margin,
            scroll_snap_type: ScrollSnapType::Enabled {
                axis: ScrollSnapAxis::Both,
                strictness: ScrollSnapStrictness::Mandatory,
            },
            scroll_snap_align: snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInput::default()
        },
    );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::new(
            Available::definite(100.0),
            Available::definite(80.0),
        ))
        .unwrap(),
    )
    .expect("viewport flex root publishes canonical geometry");

    for output in [
        batch.unrounded_entries()[0].output(),
        batch.final_entries()[0].output(),
    ] {
        let geometry = output
            .scroll_geometry
            .expect("a performed viewport flex root has geometry");
        assert_eq!(geometry.flow_axes(), flow_axes);
        assert_eq!(geometry.used_overflow_x(), Overflow::Hidden);
        assert_eq!(geometry.used_overflow_y(), Overflow::Scroll);
        assert_eq!(
            geometry.resolved_scroll_padding(),
            Edges::new(2.0, 4.0, 3.0, 1.0)
        );
        assert_eq!(
            geometry.scroll_snap_type(),
            ScrollSnapType::Enabled {
                axis: ScrollSnapAxis::Both,
                strictness: ScrollSnapStrictness::Mandatory,
            }
        );
        let target = geometry.target();
        assert_eq!(target.border_box(), geometry.border_box());
        assert_eq!(target.scroll_margin(), scroll_margin);
        assert_eq!(target.flow_axes(), flow_axes);
        assert_eq!(target.snap_align(), snap_align);
        assert_eq!(target.snap_stop(), ScrollSnapStop::Always);
        assert_eq!(output.content_box_size(), geometry.content_box().size());
        assert_eq!(output.scrollbar_size(), geometry.scrollbar_size());
    }
}

fn fri05_c03_block_root_output(batch: &CompletedLayoutBatch<u32>, node: u32) -> NodeOutput {
    batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("FRI-05 block node {node} has staged output"))
        .output()
}

#[test]
fn fri05_c04_flex_auto_independent_inner_settlement_keys_grandchildren_by_inner_pass() {
    let tree = fri05_c04_nested_flex_auto_tree(true);
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    let batch = compute_layout(&tree, 0, request).expect("independent inner auto layout succeeds");
    assert_eq!(
        public_flow_output(batch.unrounded_entries(), 0)
            .scroll_geometry
            .unwrap()
            .scrollbar_size(),
        Size::ZERO
    );
    assert_eq!(
        public_flow_output(batch.unrounded_entries(), 1)
            .scroll_geometry
            .unwrap()
            .scrollbar_size(),
        Size::new(15.0, 0.0)
    );

    let grandchild_requests = tree.cache_inputs(2);
    fri05_c04_assert_initial_local_auto_state(&grandchild_requests);
    assert_eq!(
        grandchild_requests
            .iter()
            .map(fri05_c03_block_root_state)
            .collect::<HashSet<_>>(),
        HashSet::from([(false, false), (true, true)])
    );
    for node in [0, 1, 2] {
        assert_eq!(
            batch
                .unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only stable independently settled output is published for node {node}"
        );
    }
}

#[test]
fn fri05_c03_block_reservation_root_preserves_hidden_stable_both_edges() {
    let tree = PublicFlowTree::default().with_children(0, []).with_style(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();
    let batch = compute_layout(&tree, 0, request).expect("stable block root succeeds");
    let geometry = fri05_c03_block_root_output(&batch, 0)
        .scroll_geometry
        .expect("block root emits geometry");

    assert_eq!(geometry.scrollbar_size(), Size::new(30.0, 0.0));
    assert!(geometry.gutters().left().is_some());
    assert!(geometry.gutters().right().is_some());
    assert_eq!(geometry.gutters().top(), None);
    assert_eq!(geometry.gutters().bottom(), None);
    assert_eq!(geometry.content_box().size(), Size::new(70.0, 80.0));
}

#[test]
fn fri05_c03_block_auto_root_keys_each_pass_and_publishes_only_stable_nodes() {
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(100.0)),
                ..NodeInput::default()
            },
        );
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    let batch = compute_layout(&tree, 0, request).expect("auto block root succeeds");

    let expected_states = [(false, false), (true, false), (true, true)];
    let cache_inputs = tree.cache_inputs(1);
    fri05_c04_assert_initial_local_auto_state(&cache_inputs);
    assert_eq!(
        cache_inputs
            .iter()
            .map(fri05_c03_block_root_state)
            .collect::<Vec<_>>(),
        expected_states,
        "recorded child cache inputs: {cache_inputs:#?}"
    );
    assert_eq!(
        batch
            .cache_store_entries()
            .iter()
            .filter(|entry| entry.node() == 1)
            .map(|entry| fri05_c03_block_root_state(entry.input()))
            .collect::<Vec<_>>(),
        expected_states
    );
    assert!(
        batch
            .cache_store_entries()
            .iter()
            .filter(|entry| entry.node() == 1)
            .all(|entry| fri05_c04_local_auto_state(entry.input()) == (false, false))
    );

    for node in [0, 1] {
        assert_eq!(
            batch
                .unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only the stable unrounded output for node {node} is published"
        );
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only the stable rounded output for node {node} is published"
        );
    }

    let root = fri05_c03_block_root_output(&batch, 0);
    let geometry = root
        .scroll_geometry
        .expect("stable block root output includes geometry");
    assert_eq!(geometry.content_box().size(), Size::new(85.0, 85.0));
    assert_eq!(geometry.scrollbar_size(), Size::new(15.0, 15.0));
    assert_eq!(root.scrollbar_size(), Size::new(15.0, 15.0));
}

#[test]
fn fri05_c03_block_auto_root_stable_reservations_stage_only_geometry_changes() {
    for (gutter, child_size, expected_states, expected_scrollbar_size) in [
        (
            ScrollbarGutter::Stable,
            Size::new(80.0, 120.0),
            vec![(false, false)],
            Size::new(15.0, 0.0),
        ),
        (
            ScrollbarGutter::StableBothEdges,
            Size::new(60.0, 120.0),
            vec![(false, false)],
            Size::new(30.0, 0.0),
        ),
        (
            ScrollbarGutter::Stable,
            Size::new(90.0, 120.0),
            vec![(false, false), (true, true)],
            Size::new(15.0, 15.0),
        ),
    ] {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    scrollbar_gutter: gutter,
                    scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSize::px(child_size.width),
                        PreferredSize::px(child_size.height),
                    ),
                    ..NodeInput::default()
                },
            );
        let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
        let batch = compute_layout(&tree, 0, request).expect("stable auto block root succeeds");

        let cache_inputs = tree.cache_inputs(1);
        fri05_c04_assert_initial_local_auto_state(&cache_inputs);
        assert_eq!(
            cache_inputs
                .iter()
                .map(fri05_c03_block_root_state)
                .collect::<Vec<_>>(),
            expected_states,
            "{gutter:?} child size {child_size:?}"
        );
        assert_eq!(
            batch
                .cache_store_entries()
                .iter()
                .filter(|entry| entry.node() == 1)
                .count(),
            expected_states.len(),
            "only geometry-changing child evaluations are staged"
        );
        for node in [0, 1] {
            assert_eq!(
                batch
                    .unrounded_entries()
                    .iter()
                    .filter(|entry| entry.node() == node)
                    .count(),
                1,
                "only one stable unrounded output is published for node {node}"
            );
        }
        assert_eq!(
            fri05_c03_block_root_output(&batch, 0)
                .scroll_geometry
                .expect("stable root geometry is present")
                .scrollbar_size(),
            expected_scrollbar_size
        );
    }
}

#[test]
fn fri05_c03_block_tiny_root_keeps_ordered_zero_scrollport_geometry() {
    let tree = PublicFlowTree::default().with_children(0, []).with_style(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(2.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(2.0),
        Available::definite(20.0),
    ))
    .unwrap();
    let batch = compute_layout(&tree, 0, request).expect("tiny block root remains supported");
    let geometry = fri05_c03_block_root_output(&batch, 0)
        .scroll_geometry
        .expect("tiny block root emits geometry");

    assert_eq!(geometry.border_box().size(), Size::new(2.0, 20.0));
    assert_eq!(geometry.content_box().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollport().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollbar_size(), Size::new(2.0, 0.0));
}

#[test]
fn fri05_c03_block_tiny_root_available_below_raw_edges_avoids_false_auto_settlement() {
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                border: Edges {
                    top: Length::px(15.0),
                    bottom: Length::px(15.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(0.0)),
                ..NodeInput::default()
            },
        );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(12.0),
    ))
    .unwrap();
    let batch = compute_layout(&tree, 0, request)
        .expect("root availability below raw edges remains supported");

    let cache_inputs = tree.cache_inputs(1);
    fri05_c04_assert_initial_local_auto_state(&cache_inputs);
    assert_eq!(cache_inputs.len(), 1);
    assert_eq!(fri05_c03_block_root_state(&cache_inputs[0]), (false, false));
    assert_eq!(
        cache_inputs[0].available().width,
        Available::definite(100.0)
    );
    assert_eq!(
        batch
            .cache_store_entries()
            .iter()
            .filter(|entry| entry.node() == 1)
            .count(),
        1
    );
    assert_eq!(
        fri05_c03_block_root_output(&batch, 1).location,
        Point::new(0.0, 15.0)
    );

    let root = fri05_c03_block_root_output(&batch, 0);
    assert_eq!(root.size, Size::new(100.0, 30.0));
    let geometry = root
        .scroll_geometry
        .expect("performed root emits canonical geometry");
    assert_eq!(geometry.border_box().size(), root.size);
    assert_eq!(geometry.padding_box().origin(), Point::new(0.0, 15.0));
    assert_eq!(geometry.padding_box().size(), Size::new(100.0, 0.0));
    assert_eq!(geometry.content_box(), geometry.padding_box());
    assert_eq!(geometry.scrollport(), geometry.padding_box());
    assert_eq!(geometry.physical_range().x().maximum(), 0.0);
    assert_eq!(geometry.physical_range().y().maximum(), 0.0);
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
}

#[test]
fn fri05_c04_flex_contribution_root_content_size_unions_anchor_and_overflow_per_axis() {
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Flex,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(8.0)),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(3.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                inset: Edges {
                    left: LengthAuto::px(-6.0),
                    top: LengthAuto::px(-4.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::new(
            Available::definite(10.0),
            Available::definite(8.0),
        ))
        .unwrap(),
    )
    .expect("root flex contribution layout succeeds");
    let root = public_flow_output(batch.final_entries(), 0);
    let geometry = root.scroll_geometry.expect("root flex geometry is present");
    let overflow = geometry.scrollable_overflow();
    let anchor = geometry.content_box().origin();
    let minimum = Point::new(
        anchor.x.min(overflow.origin().x),
        anchor.y.min(overflow.origin().y),
    );
    let maximum = Point::new(
        anchor.x.max(overflow.origin().x + overflow.size().width),
        anchor.y.max(overflow.origin().y + overflow.size().height),
    );

    assert!(overflow.origin().x < 0.0);
    assert!(overflow.origin().y < 0.0);
    assert_eq!(
        root.content_size,
        Size::new(maximum.x - minimum.x, maximum.y - minimum.y)
    );
}
