use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use crate::geometry::LogicalSizeOf;
use crate::test_support::layout_tree::OracleTreeOf;
use crate::*;

fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

fn fri06_atomic_participation<S: LayoutScalar>() -> AtomicInlineParticipationOf<S> {
    AtomicInlineParticipationOf::try_new(
        BidiLevel::try_new(0).unwrap(),
        InlineBreakOpportunityOf::prohibited(),
    )
    .unwrap()
}

struct RootTestScrollGeometryFacts<S: LayoutScalar> {
    flow_axes: FlowAxes,
    overflow: ComputedOverflow,
    item_is_replaced: bool,
    border_box_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    scrollbar_width: S,
    scrollable_overflow: ScrollRectOf<S>,
}

fn root_test_scroll_geometry<S: LayoutScalar>(
    facts: RootTestScrollGeometryFacts<S>,
) -> ScrollGeometryOf<S> {
    let mut contributions =
        crate::scroll::ScrollContributionAccumulatorOf::new(facts.scrollable_overflow);
    contributions.include_direct_line(facts.scrollable_overflow);
    crate::scroll::canonical_scroll_geometry_from_source(
        crate::scroll::CanonicalScrollGeometrySourceOf {
            flow_axes: facts.flow_axes,
            computed_overflow: facts.overflow,
            item_is_replaced: facts.item_is_replaced,
            border_box_size: facts.border_box_size,
            border: facts.border,
            padding: facts.padding,
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: ScrollbarWidthOf::try_new(facts.scrollbar_width).unwrap(),
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            clip_margin: crate::scroll::ClipMarginSourceOf::default(),
            scroll_padding: crate::scroll::OptimalRegionInsetsOf::default(),
            contributions,
            origin_axes: crate::scroll::ScrollOriginAxes::new(
                crate::scroll::ScrollOriginProgression::FlowEndward,
                crate::scroll::ScrollOriginProgression::FlowEndward,
            ),
            scroll_snap_type: ScrollSnapType::default(),
            target_border_box: ScrollRectOf::try_new(Point::ZERO, facts.border_box_size).unwrap(),
            target_scroll_margin: ScrollMarginOf::default(),
            target_flow_axes: facts.flow_axes,
            target_snap_align: ScrollSnapAlign::default(),
            target_snap_stop: ScrollSnapStop::default(),
        },
    )
    .expect("canonical root-test source facts produce geometry")
}

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

#[derive(Clone)]
struct Fri06C02TextTree<S: LayoutScalar> {
    inputs: HashMap<u32, LayoutInputOf<S>>,
    node_inputs: HashMap<u32, NodeInputOf<S>>,
    children: HashMap<u32, Vec<u32>>,
}

impl<S: LayoutScalar> Traverse for Fri06C02TextTree<S> {
    type Node = u32;
    type Scalar = S;
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
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<S: LayoutScalar> LayoutTree for Fri06C02TextTree<S> {
    type MeasureError = ();

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.node_inputs[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.inputs[&node].clone()
    }
}

fn fri06_c02_segment<S: LayoutScalar>(
    id: u64,
    extent: f64,
    whitespace: InlineWhitespaceEdge,
    following_break: InlineBreakOpportunityOf<S>,
) -> ShapedInlineSegmentOf<S> {
    fri06_c02_segment_with_level(id, extent, 0, whitespace, following_break)
}

fn fri06_c02_segment_with_level<S: LayoutScalar>(
    id: u64,
    extent: f64,
    bidi_level: u8,
    whitespace: InlineWhitespaceEdge,
    following_break: InlineBreakOpportunityOf<S>,
) -> ShapedInlineSegmentOf<S> {
    ShapedInlineSegmentOf::try_new(
        InlineSegmentId::new(id),
        S::from_f64(extent),
        InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0)).unwrap(),
        BidiLevel::try_new(bidi_level).unwrap(),
        whitespace,
        following_break,
    )
    .unwrap()
}

fn fri06_c02_segment_with_metrics<S: LayoutScalar>(
    id: u64,
    extent: f64,
    ascent: f64,
    descent: f64,
) -> ShapedInlineSegmentOf<S> {
    ShapedInlineSegmentOf::try_new(
        InlineSegmentId::new(id),
        S::from_f64(extent),
        InlineMetricsOf::from_ascent_descent(S::from_f64(ascent), S::from_f64(descent)).unwrap(),
        BidiLevel::try_new(0).unwrap(),
        InlineWhitespaceEdge::Preserve,
        InlineBreakOpportunityOf::prohibited(),
    )
    .unwrap()
}

fn fri06_c02_text_batch<S: LayoutScalar>(
    segments: Vec<ShapedInlineSegmentOf<S>>,
    available_inline: AvailableOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    fri06_c02_text_batch_with_flow(
        segments,
        available_inline,
        WritingMode::HorizontalTb,
        Direction::Ltr,
        TextAlign::Auto,
    )
}

fn fri06_c02_text_batch_with_flow<S: LayoutScalar>(
    segments: Vec<ShapedInlineSegmentOf<S>>,
    available_inline: AvailableOf<S>,
    writing_mode: WritingMode,
    direction: Direction,
    text_align: TextAlign,
) -> CompletedLayoutBatchOf<u32, S> {
    let root_input = NodeInputOf {
        display: Display::Block,
        writing_mode,
        direction,
        text_align,
        ..NodeInputOf::default()
    };
    let mut inputs = HashMap::new();
    inputs.insert(0, LayoutInputOf::box_input(root_input.clone()));
    inputs.insert(
        1,
        LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap()),
    );
    let tree = Fri06C02TextTree {
        inputs,
        node_inputs: HashMap::from([(0, root_input), (1, NodeInputOf::non_box())]),
        children: HashMap::from([(0, vec![1]), (1, Vec::new())]),
    };

    let viewport = FlowAxes::new(writing_mode, direction).physical_size(LogicalSizeOf::new(
        available_inline,
        AvailableOf::MAX_CONTENT,
    ));
    compute_layout(&tree, 0, LayoutRootRequestOf::viewport(viewport).unwrap()).unwrap()
}

fn fri06_c02_text_nodes_batch<S: LayoutScalar>(
    text_nodes: Vec<(u32, Vec<ShapedInlineSegmentOf<S>>)>,
    root_input: NodeInputOf<S>,
    available: Size<AvailableOf<S>>,
) -> CompletedLayoutBatchOf<u32, S> {
    let children = text_nodes.iter().map(|(node, _)| *node).collect::<Vec<_>>();
    let mut inputs = HashMap::from([(0, LayoutInputOf::box_input(root_input.clone()))]);
    let mut node_inputs = HashMap::from([(0, root_input)]);
    let mut tree_children = HashMap::from([(0, children)]);
    for (node, segments) in text_nodes {
        inputs.insert(
            node,
            LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap()),
        );
        node_inputs.insert(node, NodeInputOf::non_box());
        tree_children.insert(node, Vec::new());
    }
    let tree = Fri06C02TextTree {
        inputs,
        node_inputs,
        children: tree_children,
    };

    compute_layout(&tree, 0, LayoutRootRequestOf::viewport(available).unwrap()).unwrap()
}

fn fri06_c02_final_node<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

fn fri06_c03_atomic_participation<S: LayoutScalar>(
    level: u8,
    following_break: InlineBreakOpportunityOf<S>,
) -> AtomicInlineParticipationOf<S> {
    AtomicInlineParticipationOf::try_new(BidiLevel::try_new(level).unwrap(), following_break)
        .unwrap()
}

fn fri06_c03_atomic_style<S: LayoutScalar>(
    inline_extent: f64,
    block_extent: f64,
    inline_margin_start: f64,
    inline_margin_end: f64,
    level: u8,
    following_break: InlineBreakOpportunityOf<S>,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::InlineBlock,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(inline_extent)),
            PreferredSizeOf::px(S::from_f64(block_extent)),
        ),
        margin: Edges {
            right: LengthAutoOf::px(S::from_f64(inline_margin_end)),
            left: LengthAutoOf::px(S::from_f64(inline_margin_start)),
            ..Edges::all(LengthAutoOf::ZERO)
        },
        atomic_inline_participation: Some(fri06_c03_atomic_participation(level, following_break)),
        ..NodeInputOf::default()
    }
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

fn fri06_c03_mixed_batch_with_root<S: LayoutScalar>(
    children: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    available_inline: AvailableOf<S>,
    root_input: NodeInputOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let root_input = NodeInputOf {
        display: Display::Block,
        ..root_input
    };
    let child_nodes = children
        .iter()
        .map(|(node, _, _)| *node)
        .collect::<Vec<_>>();
    let mut inputs = HashMap::from([(0, LayoutInputOf::box_input(root_input.clone()))]);
    let mut node_inputs = HashMap::from([(0, root_input)]);
    let mut tree_children = HashMap::from([(0, child_nodes)]);
    for (node, layout_input, node_input) in children {
        inputs.insert(node, layout_input);
        node_inputs.insert(node, node_input);
        tree_children.insert(node, Vec::new());
    }
    let tree = Fri06C02TextTree {
        inputs,
        node_inputs,
        children: tree_children,
    };

    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(available_inline, AvailableOf::MAX_CONTENT))
            .unwrap(),
    )
    .unwrap()
}

fn fri06_c03_text_input<S: LayoutScalar>(
    segments: Vec<ShapedInlineSegmentOf<S>>,
) -> LayoutInputOf<S> {
    LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap())
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
    let tree = Fri06C02TextTree {
        inputs: HashMap::from([
            (0, LayoutInputOf::box_input(root_style.clone())),
            (1, LayoutInputOf::inline_text(parent_text)),
            (2, LayoutInputOf::box_input(atomic_style.clone())),
            (3, LayoutInputOf::inline_text(first_inner_text)),
            (4, LayoutInputOf::line_break(inner_break)),
            (5, LayoutInputOf::inline_text(last_inner_text)),
        ]),
        node_inputs: HashMap::from([
            (0, root_style),
            (1, NodeInputOf::non_box()),
            (2, atomic_style),
            (3, NodeInputOf::non_box()),
            (4, NodeInputOf::non_box()),
            (5, NodeInputOf::non_box()),
        ]),
        children: HashMap::from([
            (0, vec![1, 2]),
            (1, Vec::new()),
            (2, vec![3, 4, 5]),
            (3, Vec::new()),
            (4, Vec::new()),
            (5, Vec::new()),
        ]),
    };
    let viewport = parent_flow.physical_size(LogicalSizeOf::new(
        AvailableOf::definite(S::from_f64(100.0)),
        AvailableOf::MAX_CONTENT,
    ));

    compute_layout(&tree, 0, LayoutRootRequestOf::viewport(viewport).unwrap()).unwrap()
}

#[derive(Clone)]
struct Fri06C03CachedAtomicTree<S: LayoutScalar> {
    tree: Fri06C02TextTree<S>,
    atomic_output: ComputeOutputOf<S>,
}

impl<S: LayoutScalar> Traverse for Fri06C03CachedAtomicTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.tree.children(node)
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
            tree: Fri06C02TextTree {
                inputs: HashMap::from([
                    (0, LayoutInputOf::box_input(root_style.clone())),
                    (
                        1,
                        fri06_c03_text_input(vec![fri06_c02_segment_with_metrics(
                            703, 10.0, 8.0, 2.0,
                        )]),
                    ),
                    (2, LayoutInputOf::box_input(atomic_style.clone())),
                ]),
                node_inputs: HashMap::from([
                    (0, root_style),
                    (1, NodeInputOf::non_box()),
                    (2, atomic_style),
                ]),
                children: HashMap::from([(0, vec![1, 2]), (1, Vec::new()), (2, Vec::new())]),
            },
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
        let tree = Fri06C02TextTree {
            inputs: HashMap::from([
                (0, LayoutInputOf::box_input(root_style.clone())),
                (1, LayoutInputOf::box_input(atomic_style.clone())),
            ]),
            node_inputs: HashMap::from([(0, root_style), (1, atomic_style)]),
            children: HashMap::from([(0, vec![1]), (1, Vec::new())]),
        };
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
            Point::new(S::ZERO, S::from_f64(10.0))
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

fn fri06_c02_expected_physical_rect<S: LayoutScalar>(
    flow: (WritingMode, Direction),
    logical_rect: (f64, f64, f64, f64),
    containing: (f64, f64),
) -> (Point<S>, Size<S>) {
    let (writing_mode, direction) = flow;
    let (inline_start, block_start, inline_extent, block_extent) = logical_rect;
    let (containing_inline, containing_block) = containing;
    let (x, y, width, height) = match writing_mode {
        WritingMode::HorizontalTb => {
            let x = if direction == Direction::Rtl {
                containing_inline - inline_start - inline_extent
            } else {
                inline_start
            };
            (x, block_start, inline_extent, block_extent)
        }
        WritingMode::VerticalRl | WritingMode::SidewaysRl => {
            let y = if direction == Direction::Rtl {
                containing_inline - inline_start - inline_extent
            } else {
                inline_start
            };
            (
                containing_block - block_start - block_extent,
                y,
                block_extent,
                inline_extent,
            )
        }
        WritingMode::VerticalLr => {
            let y = if direction == Direction::Rtl {
                containing_inline - inline_start - inline_extent
            } else {
                inline_start
            };
            (block_start, y, block_extent, inline_extent)
        }
        WritingMode::SidewaysLr => {
            let y = if direction == Direction::Ltr {
                containing_inline - inline_start - inline_extent
            } else {
                inline_start
            };
            (block_start, y, block_extent, inline_extent)
        }
    };
    (
        Point::new(S::from_f64(x), S::from_f64(y)),
        Size::new(S::from_f64(width), S::from_f64(height)),
    )
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
            for (fragment, (inline_start, inline_extent, block_start, baseline_block)) in
                fragments.iter().zip([
                    (50.0, 10.0, 0.0, 8.0),
                    (40.0, 10.0, 0.0, 8.0),
                    (51.0, 4.0, 10.0, 18.0),
                    (45.0, 6.0, 10.0, 18.0),
                ])
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
        let tree = Fri06C02TextTree {
            inputs: HashMap::from([
                (0, LayoutInputOf::box_input(root.clone())),
                (1, LayoutInputOf::box_input(item.clone())),
                (2, LayoutInputOf::box_input(item.clone())),
                (3, LayoutInputOf::inline_text(text_one)),
                (4, LayoutInputOf::inline_text(text_two)),
            ]),
            node_inputs: HashMap::from([
                (0, root),
                (1, item.clone()),
                (2, item),
                (3, NodeInputOf::non_box()),
                (4, NodeInputOf::non_box()),
            ]),
            children: HashMap::from([
                (0, vec![1, 2]),
                (1, vec![3]),
                (2, vec![4]),
                (3, Vec::new()),
                (4, Vec::new()),
            ]),
        };
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

#[test]
fn fri06_c02_fragment_publication_is_per_node_source_ordered_and_retains_empty_anchors_both_scalars()
 {
    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c02_text_nodes_batch(
            vec![
                (
                    1,
                    vec![
                        fri06_c02_segment(
                            11,
                            8.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::mandatory(),
                        ),
                        fri06_c02_segment(
                            12,
                            6.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::prohibited(),
                        ),
                    ],
                ),
                (
                    2,
                    vec![fri06_c02_segment(
                        21,
                        7.0,
                        InlineWhitespaceEdge::DiscardAtBoth,
                        InlineBreakOpportunityOf::mandatory(),
                    )],
                ),
            ],
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
            Size::new(
                AvailableOf::definite(S::from_f64(30.0)),
                AvailableOf::MAX_CONTENT,
            ),
        );
        assert_eq!(
            batch
                .final_inline_fragments()
                .iter()
                .map(|entry| (entry.node(), entry.fragment().segment_id().get()))
                .collect::<Vec<_>>(),
            vec![(1, 11), (1, 12)]
        );
        let first = fri06_c02_final_node(&batch, 1);
        assert_eq!(first.location, Point::ZERO);
        assert_eq!(first.size, Size::new(S::from_f64(8.0), S::from_f64(20.0)));
        assert_eq!(first.content_size, first.size);
        assert_eq!(first.border, Edges::ZERO);
        assert_eq!(first.padding, Edges::ZERO);
        assert_eq!(first.margin, Edges::ZERO);
        assert!(first.scroll_geometry.is_none());
        let empty = fri06_c02_final_node(&batch, 2);
        assert_eq!(
            empty.location,
            Point::new(S::from_f64(6.0), S::from_f64(10.0))
        );
        assert_eq!(empty.size, Size::ZERO);
        assert_eq!(empty.content_size, Size::ZERO);
        assert!(empty.scroll_geometry.is_none());

        let hidden = fri06_c02_text_nodes_batch(
            vec![(
                1,
                vec![fri06_c02_segment(
                    31,
                    10.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                )],
            )],
            NodeInputOf {
                display: Display::None,
                ..NodeInputOf::default()
            },
            Size::splat(AvailableOf::definite(S::from_f64(30.0))),
        );
        assert!(hidden.final_inline_fragments().is_empty());
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

#[derive(Clone, Debug, Default, PartialEq)]
struct Fri06C02RetainedTextState<S: LayoutScalar> {
    unrounded_nodes: HashMap<u32, NodeOutputOf<S>>,
    final_nodes: HashMap<u32, NodeOutputOf<S>>,
    unrounded_fragments: HashMap<u32, Vec<InlineFragmentOutputOf<S>>>,
    final_fragments: HashMap<u32, Vec<InlineFragmentOutputOf<S>>>,
    caches: HashMap<u32, CacheOf<S>>,
    dirty: Vec<u32>,
}

#[derive(Clone, Debug)]
struct Fri06C02StatefulTextTree<S: LayoutScalar> {
    inputs: HashMap<u32, LayoutInputOf<S>>,
    node_inputs: HashMap<u32, NodeInputOf<S>>,
    children: HashMap<u32, Vec<u32>>,
    retained: Fri06C02RetainedTextState<S>,
    fragment_readbacks: Cell<usize>,
    reject_preparation: bool,
}

impl<S: LayoutScalar> Fri06C02StatefulTextTree<S> {
    fn new(segments: Vec<ShapedInlineSegmentOf<S>>) -> Self {
        let root_input = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        Self {
            inputs: HashMap::from([
                (0, LayoutInputOf::box_input(root_input.clone())),
                (
                    1,
                    LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap()),
                ),
            ]),
            node_inputs: HashMap::from([(0, root_input), (1, NodeInputOf::non_box())]),
            children: HashMap::from([(0, vec![1]), (1, Vec::new())]),
            retained: Fri06C02RetainedTextState::default(),
            fragment_readbacks: Cell::new(0),
            reject_preparation: false,
        }
    }

    fn replace_text(&mut self, segments: Vec<ShapedInlineSegmentOf<S>>) {
        self.inputs.insert(
            1,
            LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap()),
        );
    }

    fn add_failing_noncanonical_control(&mut self) {
        self.inputs
            .insert(2, LayoutInputOf::line_break(LineBreakInputOf::new()));
        self.node_inputs.insert(2, NodeInputOf::default());
        self.children.insert(0, vec![1, 2]);
        self.children.insert(2, Vec::new());
    }

    fn remove_failing_noncanonical_control(&mut self) {
        self.inputs.remove(&2);
        self.node_inputs.remove(&2);
        self.children.insert(0, vec![1]);
        self.children.remove(&2);
    }
}

impl<S: LayoutScalar> Traverse for Fri06C02StatefulTextTree<S> {
    type Node = u32;
    type Scalar = S;
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
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<S: LayoutScalar> LayoutTree for Fri06C02StatefulTextTree<S> {
    type MeasureError = ();

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.node_inputs[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.inputs[&node].clone()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        self.retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context))
    }

    fn unrounded_inline_fragments(&self, node: Self::Node) -> Option<&[InlineFragmentOutputOf<S>]> {
        self.fragment_readbacks
            .set(self.fragment_readbacks.get() + 1);
        self.retained
            .unrounded_fragments
            .get(&node)
            .map(Vec::as_slice)
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri06C02StatefulTextTree<S> {
    type Error = &'static str;
    type Prepared = Fri06C02RetainedTextState<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        if self.reject_preparation {
            return Err("C02 retained-state preparation rejected");
        }

        let mut prepared = self.retained.clone();
        for node in batch.invalidated_nodes() {
            prepared.unrounded_nodes.remove(node);
            prepared.final_nodes.remove(node);
            prepared.unrounded_fragments.remove(node);
            prepared.final_fragments.remove(node);
            prepared.caches.remove(node);
        }
        for entry in batch.unrounded_entries() {
            let node = entry.node();
            prepared.unrounded_nodes.insert(node, entry.output());
            if matches!(self.inputs.get(&node), Some(LayoutInputOf::InlineText(_))) {
                prepared.unrounded_fragments.insert(node, Vec::new());
            }
        }
        for entry in batch.final_entries() {
            let node = entry.node();
            prepared.final_nodes.insert(node, entry.output());
            if matches!(self.inputs.get(&node), Some(LayoutInputOf::InlineText(_))) {
                prepared.final_fragments.insert(node, Vec::new());
            }
        }
        for entry in batch.unrounded_inline_fragments() {
            prepared
                .unrounded_fragments
                .entry(entry.node())
                .or_default()
                .push(entry.fragment());
        }
        for entry in batch.final_inline_fragments() {
            prepared
                .final_fragments
                .entry(entry.node())
                .or_default()
                .push(entry.fragment());
        }
        for entry in batch.cache_clear_entries() {
            prepared.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            prepared
                .caches
                .entry(entry.node())
                .or_default()
                .store_with_context(entry.input(), entry.context(), entry.output());
        }
        prepared.dirty.clear();
        Ok(prepared)
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

fn fri06_c02_stateful_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(S::from_f64(20.0)),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}

#[test]
fn fri06_c02_cache_cold_warm_and_dirty_replacement_use_committed_state_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new(vec![fri06_c02_segment(
            61,
            9.25,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);

        let cold = compute_layout(&tree, 0, request).expect("cold text layout succeeds");
        let cold_unrounded = cold.unrounded_entries().to_vec();
        let cold_final = cold.final_entries().to_vec();
        let cold_unrounded_fragments = cold.unrounded_inline_fragments().to_vec();
        let cold_final_fragments = cold.final_inline_fragments().to_vec();
        assert_eq!(tree.fragment_readbacks.get(), 0);
        cold.apply_to(&mut tree).expect("cold batch commits");
        assert_eq!(tree.retained.dirty, []);
        assert_eq!(tree.retained.unrounded_fragments[&1].len(), 1);
        assert!(!tree.retained.caches.is_empty());

        let warm = compute_layout(&tree, 0, request).expect("warm text layout restores fragments");
        assert_eq!(warm.unrounded_entries(), cold_unrounded);
        assert_eq!(warm.final_entries(), cold_final);
        assert_eq!(warm.unrounded_inline_fragments(), cold_unrounded_fragments);
        assert_eq!(warm.final_inline_fragments(), cold_final_fragments);
        assert_eq!(tree.fragment_readbacks.get(), 1);
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 1),
            "the committed text cache must serve the warm text node"
        );
        warm.apply_to(&mut tree).expect("warm batch recommits");

        let stale_text_output = tree.retained.unrounded_nodes[&1];
        let stale_fragments = tree.retained.unrounded_fragments[&1].clone();
        let stale_caches = tree.retained.caches.clone();
        tree.replace_text(vec![fri06_c02_segment(
            62,
            13.75,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);
        tree.retained.dirty = vec![1, 1];

        let invalidated = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("dirty text layout succeeds");
        assert_eq!(invalidated.invalidated_nodes(), &[0, 1]);
        assert_eq!(tree.retained.dirty, [1, 1]);
        invalidated
            .apply_to(&mut tree)
            .expect("dirty replacement batch commits");

        assert!(tree.retained.dirty.is_empty());
        assert_ne!(tree.retained.unrounded_nodes[&1], stale_text_output);
        assert_ne!(tree.retained.unrounded_fragments[&1], stale_fragments);
        assert_ne!(tree.retained.caches, stale_caches);
        assert_eq!(
            tree.retained.unrounded_fragments[&1][0].segment_id(),
            InlineSegmentId::new(62)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_cache_committed_empty_fragment_state_replays_warm_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new(vec![fri06_c02_segment(
            63,
            6.5,
            InlineWhitespaceEdge::DiscardAtBoth,
            InlineBreakOpportunityOf::prohibited(),
        )]);

        let cold = compute_layout(&tree, 0, request).expect("cold discarded text layout succeeds");
        assert!(cold.unrounded_inline_fragments().is_empty());
        cold.apply_to(&mut tree)
            .expect("committed empty fragment state is retained");
        assert_eq!(tree.unrounded_inline_fragments(1), Some([].as_slice()));
        tree.fragment_readbacks.set(0);
        assert!(
            tree.retained.caches.contains_key(&1),
            "the committed empty fragment state must pair with a warm text cache"
        );

        let warm = compute_layout(&tree, 0, request)
            .expect("Some(&[]) is valid warm committed fragment state");
        assert!(warm.unrounded_inline_fragments().is_empty());
        assert!(warm.final_inline_fragments().is_empty());
        assert_eq!(tree.fragment_readbacks.get(), 1);
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 1)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_rounding_fractional_fragments_round_once_without_identity_drift_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let replacement =
            InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(0.75)).unwrap();
        let batch = fri06_c02_text_batch(
            vec![
                fri06_c02_segment(71, 8.25, InlineWhitespaceEdge::Preserve, replacement),
                fri06_c02_segment(
                    72,
                    8.25,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ],
            AvailableOf::definite(S::from_f64(10.5)),
        );
        let unrounded = batch.unrounded_inline_fragments();
        let final_fragments = batch.final_inline_fragments();
        assert_eq!(unrounded.len(), 2);
        assert_eq!(final_fragments.len(), 2);
        assert_eq!(
            unrounded
                .iter()
                .map(|entry| {
                    let fragment = entry.fragment();
                    (
                        entry.node(),
                        fragment.segment_id(),
                        fragment.line_index(),
                        fragment.visual_index(),
                        fragment.replacement_inline_extent(),
                    )
                })
                .collect::<Vec<_>>(),
            final_fragments
                .iter()
                .map(|entry| {
                    let fragment = entry.fragment();
                    (
                        entry.node(),
                        fragment.segment_id(),
                        fragment.line_index(),
                        fragment.visual_index(),
                        fragment.replacement_inline_extent(),
                    )
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(
            unrounded[0].fragment().rect().size().width,
            S::from_f64(8.25)
        );
        assert_eq!(
            final_fragments[0].fragment().rect().size().width,
            S::from_f64(8.0)
        );
        assert_eq!(unrounded[0].fragment().baseline().y, S::from_f64(8.0));
        assert_eq!(final_fragments[0].fragment().baseline().y, S::from_f64(8.0));
        assert_eq!(
            unrounded[0].fragment().replacement_inline_extent(),
            Some(S::from_f64(0.75))
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_transaction_layout_and_preparation_failures_preserve_retained_state_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new(vec![fri06_c02_segment(
            81,
            7.5,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);
        let cold = compute_layout(&tree, 0, request).expect("cold text layout succeeds");
        cold.apply_to(&mut tree).expect("cold batch commits");
        assert!(
            tree.retained.caches.contains_key(&1),
            "transaction proof requires committed text cache state"
        );

        tree.replace_text(vec![fri06_c02_segment(
            82,
            8.5,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);
        tree.retained.dirty = vec![1];
        tree.add_failing_noncanonical_control();
        let before_layout_failure = tree.retained.clone();

        let error = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect_err("noncanonical control pairing fails before retained-state mutation");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::NonBoxNodeRole {
                reason: NonBoxNodeRoleError::NonCanonicalNodeInput,
            })
        );
        assert_eq!(tree.retained, before_layout_failure);

        tree.remove_failing_noncanonical_control();
        let replacement = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("dirty text replacement stages successfully");
        tree.reject_preparation = true;
        let before_preparation_failure = tree.retained.clone();
        assert_eq!(
            replacement.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, before_preparation_failure);
        assert_eq!(tree.retained.dirty, [1]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn assert_positive_physical_range<S: LayoutScalar>(
    range: PhysicalScrollRangeOf<S>,
    maximum: Size<S>,
) {
    assert_eq!(range.x().minimum(), S::ZERO);
    assert_eq!(range.x().maximum(), maximum.width);
    assert_eq!(range.y().minimum(), S::ZERO);
    assert_eq!(range.y().maximum(), maximum.height);
}

#[derive(Clone, Debug, Default)]
struct RootSessionTree<M = &'static str> {
    children: HashMap<u32, Vec<u32>>,
    inputs: HashMap<u32, LayoutInput>,
    measurements: HashMap<u32, Result<Size, M>>,
    leaf_nodes: HashSet<u32>,
    measured_nodes: RefCell<Vec<u32>>,
    caches: RefCell<HashMap<u32, Cache>>,
}

impl<M> RootSessionTree<M> {
    fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    fn style(mut self, node: u32, style: NodeInput) -> Self {
        self.inputs.insert(node, LayoutInput::box_input(style));
        self
    }

    fn measure(mut self, node: u32, output: Result<Size, M>) -> Self {
        self.leaf_nodes.insert(node);
        self.measurements.insert(node, output);
        self
    }

    fn leaf_without_provider(mut self, node: u32) -> Self {
        self.leaf_nodes.insert(node);
        self
    }

    fn measured_nodes(&self) -> Vec<u32> {
        self.measured_nodes.borrow().clone()
    }
}

impl<M> Traverse for RootSessionTree<M> {
    type Node = u32;
    type Scalar = Scalar;
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

impl<M: Clone> LayoutTree for RootSessionTree<M> {
    type MeasureError = M;

    fn node_input(&self, node: Self::Node) -> &NodeInput {
        self.inputs[&node]
            .as_box()
            .expect("test root session node is a box")
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        self.inputs[&node].clone()
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.leaf_nodes.contains(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        _input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.measured_nodes.borrow_mut().push(node);
        self.measurements.get(&node).cloned()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>> {
        self.caches
            .borrow()
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context))
    }
}

#[derive(Clone, Debug, Default)]
struct PublicFlowTree<S: LayoutScalar> {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInputOf<S>>,
    caches: RefCell<HashMap<u32, CacheOf<S>>>,
    cache_inputs: RefCell<Vec<(u32, ComputeInputOf<S>)>>,
}

impl<S: LayoutScalar> PublicFlowTree<S> {
    fn with_children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    fn with_style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.styles.insert(node, style);
        self
    }

    fn apply_cache_entries(&self, entries: &[LayoutCacheStoreEntryOf<u32, S>]) {
        let mut caches = self.caches.borrow_mut();
        for entry in entries {
            caches.entry(entry.node()).or_default().store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
    }

    fn cache_inputs(&self, node: u32) -> Vec<ComputeInputOf<S>> {
        self.cache_inputs
            .borrow()
            .iter()
            .filter_map(|(recorded_node, input)| (*recorded_node == node).then_some(*input))
            .collect()
    }

    fn clear_cache_inputs(&self) {
        self.cache_inputs.borrow_mut().clear();
    }
}

impl<S: LayoutScalar> Traverse for PublicFlowTree<S> {
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

impl<S: LayoutScalar> LayoutTree for PublicFlowTree<S> {
    type MeasureError = ();

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
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        self.cache_inputs.borrow_mut().push((node, *input));
        self.caches
            .borrow()
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context))
    }
}

#[test]
fn flex_item_root_uses_explicit_parent_axes_for_percentage_and_cache_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let parent_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let item_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
        let viewport = Size::new(
            AvailableOf::definite(scalar::<S>(200.0)),
            AvailableOf::definite(scalar::<S>(80.0)),
        );
        let available = Size::new(
            AvailableOf::definite(scalar::<S>(140.0)),
            AvailableOf::definite(scalar::<S>(300.0)),
        );
        let root_style = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            direction: Direction::Rtl,
            padding: Edges::new(
                LengthOf::percent(scalar::<S>(0.0625)),
                LengthOf::percent(scalar::<S>(0.125)),
                LengthOf::percent(scalar::<S>(0.25)),
                LengthOf::percent(scalar::<S>(0.5)),
            ),
            ..NodeInputOf::<S>::default()
        };
        let child_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(scalar::<S>(10.0)),
                PreferredSizeOf::px(scalar::<S>(20.0)),
            ),
            ..NodeInputOf::<S>::default()
        };
        let tree = PublicFlowTree::default()
            .with_children(0, [])
            .with_style(0, root_style.clone());
        let request = LayoutRootRequestOf::flex_item_under_viewport(
            available,
            FlexItemRootContextOf::under_viewport(viewport, parent_axes)
                .expect("finite flex-item root context"),
        )
        .expect("finite flex-item root request");

        let cold = compute_layout(&tree, 0, request).expect("cold flex-item root layout");
        let root_entry = cold
            .cache_store_entries()
            .iter()
            .find(|entry| {
                entry.node() == 0 && entry.input().run_mode() == RunMode::PerformRootLayout
            })
            .expect("cold root compute is cached");
        let root_input = *root_entry.input();
        let root_output = root_entry.output();
        let descendant_tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, root_style)
            .with_style(1, child_style);
        let descendant_batch =
            compute_layout(&descendant_tree, 0, request).expect("root with descendant layout");
        let child_input = descendant_batch
            .cache_store_entries()
            .iter()
            .find(|entry| entry.node() == 1 && entry.input().run_mode() == RunMode::PerformLayout)
            .expect("child layout compute is cached")
            .input();

        assert_eq!(
            root_input.containing_layout_context(),
            ContainingLayoutContext::new(parent_axes, ParentFormattingContext::Flex)
        );
        assert_eq!(
            root_input.known(),
            Size::new(Some(scalar::<S>(140.0)), None)
        );
        assert_eq!(
            child_input.containing_layout_context(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::BlockFlow)
        );

        let root = public_flow_output(cold.unrounded_entries(), 0);
        let expected_padding = Edges::new(
            scalar::<S>(12.5),
            scalar::<S>(25.0),
            scalar::<S>(50.0),
            scalar::<S>(100.0),
        );
        assert_eq!(root.padding, expected_padding);
        let logical_padding = parent_axes.logical_edges(root.padding);
        assert_eq!(logical_padding.inline_start, scalar::<S>(100.0));
        assert_eq!(logical_padding.inline_end, scalar::<S>(25.0));
        assert_eq!(logical_padding.block_start, scalar::<S>(12.5));
        assert_eq!(logical_padding.block_end, scalar::<S>(50.0));

        let cache_context = CacheKeyContext::new();
        let mut cache = CacheOf::<S>::new();
        cache.store_with_context(&root_input, cache_context, root_output);
        assert_eq!(
            cache.get_with_context(&root_input, cache_context),
            Some(root_output)
        );
        let role_only = ComputeInputOf::flex_item_root(
            root_input.known(),
            root_input.parent(),
            ContainingLayoutContext::new(parent_axes, ParentFormattingContext::NoParent),
            root_input.available(),
        );
        assert_eq!(cache.get_with_context(&role_only, cache_context), None);
        let axes_only = ComputeInputOf::flex_item_root(
            root_input.known(),
            root_input.parent(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::Flex),
            root_input.available(),
        );
        assert_eq!(cache.get_with_context(&axes_only, cache_context), None);

        tree.apply_cache_entries(cold.cache_store_entries());
        tree.clear_cache_inputs();
        let warm = compute_layout(&tree, 0, request).expect("warm flex-item root layout");
        assert_eq!(
            public_flow_output(warm.unrounded_entries(), 0),
            public_flow_output(cold.unrounded_entries(), 0)
        );
        assert_eq!(
            public_flow_output(warm.final_entries(), 0),
            public_flow_output(cold.final_entries(), 0)
        );
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 0),
            "the identical root context should hit the applied cold cache"
        );

        let viewport_tree = PublicFlowTree::default().with_children(0, []).with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                ..NodeInputOf::<S>::default()
            },
        );
        let viewport_batch = compute_layout(
            &viewport_tree,
            0,
            LayoutRootRequestOf::viewport(available).expect("finite viewport request"),
        )
        .expect("viewport layout");
        let viewport_input = viewport_batch
            .cache_store_entries()
            .iter()
            .find(|entry| {
                entry.node() == 0 && entry.input().run_mode() == RunMode::PerformRootLayout
            })
            .expect("viewport root compute is cached")
            .input();
        assert_eq!(
            viewport_input.containing_layout_context(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::NoParent)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

struct FlowRootLeafTree<S: LayoutScalar> {
    style: NodeInputOf<S>,
    natural_size: Size<S>,
    measurement: RefCell<Option<LeafMeasureInputOf<S>>>,
}

impl<S: LayoutScalar> FlowRootLeafTree<S> {
    fn new(style: NodeInputOf<S>) -> Self {
        Self {
            style,
            natural_size: Size::ZERO,
            measurement: RefCell::new(None),
        }
    }

    fn with_natural_size(mut self, natural_size: Size<S>) -> Self {
        self.natural_size = natural_size;
        self
    }
}

impl<S: LayoutScalar> Traverse for FlowRootLeafTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Empty<u32>
    where
        Self: 'a;

    fn children(&self, _node: Self::Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Self::Node) -> usize {
        0
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        unreachable!("flow-root leaf test tree has no children")
    }
}

impl<S: LayoutScalar> LayoutTree for FlowRootLeafTree<S> {
    type MeasureError = ();

    fn node_input(&self, _node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        &self.style
    }

    fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.style.clone())
    }

    fn has_leaf_measurement(&self, _node: Self::Node) -> bool {
        true
    }

    fn measure_leaf(
        &self,
        _node: Self::Node,
        input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.measurement.replace(Some(input));
        Some(Ok(self.natural_size))
    }
}

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn single_final_output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .first()
        .expect("single root must produce one final output")
        .output()
}

fn public_flow_output<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .expect("public layout batch contains the requested node")
        .output()
}

#[test]
fn replaced_viewport_and_flex_item_roots_keep_measured_auto_inline_size_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar::<S>;
        let available = Size::new(
            AvailableOf::definite(scalar(200.0)),
            AvailableOf::MAX_CONTENT,
        );
        let parent_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);

        for replaced in [true, false] {
            let style = NodeInputOf {
                display: Display::Block,
                item_is_replaced: replaced,
                ..NodeInputOf::default()
            };
            let viewport_tree = FlowRootLeafTree::new(style.clone())
                .with_natural_size(Size::new(scalar(50.0), scalar(10.0)));
            let viewport = compute_layout(
                &viewport_tree,
                0,
                LayoutRootRequestOf::viewport(available).expect("finite viewport request"),
            )
            .expect("measured viewport root lays out");
            assert_eq!(
                single_final_output(&viewport).size.width,
                scalar(if replaced { 50.0 } else { 200.0 })
            );

            let flex_tree = FlowRootLeafTree::new(style)
                .with_natural_size(Size::new(scalar(50.0), scalar(10.0)));
            let flex = compute_layout(
                &flex_tree,
                0,
                LayoutRootRequestOf::flex_item_under_viewport(
                    available,
                    FlexItemRootContextOf::under_viewport(available, parent_axes)
                        .expect("finite flex-item context"),
                )
                .expect("finite flex-item request"),
            )
            .expect("measured flex-item root lays out");
            assert_eq!(
                single_final_output(&flex).size.width,
                scalar(if replaced { 50.0 } else { 200.0 })
            );
            assert!(flex.cache_store_entries().iter().any(|entry| {
                entry.input().containing_layout_context()
                    == ContainingLayoutContext::new(parent_axes, ParentFormattingContext::Flex)
            }));
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn source_index_identity_survives_root_hidden_rounding_and_batch() {
    let tree = PublicFlowTree::default()
        .with_children(10, [20, 30])
        .with_children(20, [])
        .with_children(30, [40])
        .with_children(40, [])
        .with_style(
            10,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(100.25), PreferredSizeOf::px(50.25)),
                ..NodeInput::default()
            },
        )
        .with_style(
            20,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(25.25), PreferredSizeOf::px(10.25)),
                ..NodeInput::default()
            },
        )
        .with_style(
            30,
            NodeInput {
                display: Display::None,
                ..NodeInput::default()
            },
        )
        .with_style(40, NodeInput::default());

    let batch = compute_layout(
        &tree,
        10,
        LayoutRootRequest::viewport(Size::splat(AvailableOf::definite(200.0)))
            .expect("valid viewport request"),
    )
    .expect("root layout with a hidden subtree succeeds");

    let expected_identity = [
        (10, SourceIndex::ZERO),
        (20, SourceIndex::new(0)),
        (30, SourceIndex::new(1)),
        (40, SourceIndex::new(0)),
    ];
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        for (node, source_index) in expected_identity {
            assert_eq!(public_flow_output(entries, node).source_index, source_index);
        }
    }

    let unrounded_nodes = batch
        .unrounded_entries()
        .iter()
        .map(LayoutOutputEntry::node)
        .collect::<Vec<_>>();
    let final_nodes = batch
        .final_entries()
        .iter()
        .map(LayoutOutputEntry::node)
        .collect::<Vec<_>>();
    assert_ne!(unrounded_nodes, final_nodes);
    assert_eq!(final_nodes, vec![10, 20, 30, 40]);
}

fn logical_flex_leaf<S: LayoutScalar>(width: f64, height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(scalar::<S>(width)),
            PreferredSizeOf::px(scalar::<S>(height)),
        ),
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink factor"),
        ..NodeInputOf::default()
    }
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

fn assert_logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical<
    S: LayoutScalar,
>() {
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
                overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                position: Position::Relative,
                inset: Edges {
                    top: LengthAutoOf::px(scalar(95.5)),
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
    .expect("visible overflow and scroll projection succeed");
    assert_eq!(
        public_flow_output(batch.final_entries(), 1).location,
        Point::new(scalar(0.0), scalar(96.0))
    );
    let root = public_flow_output(batch.final_entries(), 0);
    assert_eq!(root.content_size.height, scalar(116.0));
    assert!(root.scroll_geometry.is_some());
}

#[test]
fn logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical_for_f32() {
    assert_logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical::<f32>();
}

#[test]
fn logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical_for_f64() {
    assert_logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical::<f64>();
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

fn root_writing_mode_directions() -> [(WritingMode, Direction); 10] {
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

fn fractional_child_rect<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
) -> (Point<S>, Size<S>, Point<S>, Size<S>) {
    let scalar = scalar::<S>;
    match (writing_mode, direction) {
        (WritingMode::HorizontalTb, Direction::Ltr) => (
            Point::ZERO,
            Size::new(scalar(10.25), scalar(20.25)),
            Point::ZERO,
            Size::new(scalar(10.0), scalar(20.0)),
        ),
        (WritingMode::HorizontalTb, Direction::Rtl) => (
            Point::new(scalar(90.25), S::ZERO),
            Size::new(scalar(10.25), scalar(20.25)),
            Point::new(scalar(90.0), S::ZERO),
            Size::new(scalar(11.0), scalar(20.0)),
        ),
        (WritingMode::VerticalRl, Direction::Ltr) | (WritingMode::SidewaysRl, Direction::Ltr) => (
            Point::new(scalar(80.25), S::ZERO),
            Size::new(scalar(20.25), scalar(10.25)),
            Point::new(scalar(80.0), S::ZERO),
            Size::new(scalar(21.0), scalar(10.0)),
        ),
        (WritingMode::VerticalRl, Direction::Rtl) => (
            Point::new(scalar(80.25), scalar(90.25)),
            Size::new(scalar(20.25), scalar(10.25)),
            Point::new(scalar(80.0), scalar(90.0)),
            Size::new(scalar(21.0), scalar(11.0)),
        ),
        (WritingMode::VerticalLr, Direction::Ltr) | (WritingMode::SidewaysLr, Direction::Rtl) => (
            Point::ZERO,
            Size::new(scalar(20.25), scalar(10.25)),
            Point::ZERO,
            Size::new(scalar(20.0), scalar(10.0)),
        ),
        (WritingMode::VerticalLr, Direction::Rtl) | (WritingMode::SidewaysLr, Direction::Ltr) => (
            Point::new(S::ZERO, scalar(90.25)),
            Size::new(scalar(20.25), scalar(10.25)),
            Point::new(S::ZERO, scalar(90.0)),
            Size::new(scalar(20.0), scalar(11.0)),
        ),
        (WritingMode::SidewaysRl, Direction::Rtl) => (
            Point::new(scalar(80.25), scalar(90.25)),
            Size::new(scalar(20.25), scalar(10.25)),
            Point::new(scalar(80.0), scalar(90.0)),
            Size::new(scalar(21.0), scalar(11.0)),
        ),
    }
}

fn assert_ordinary_block_root_contexts_round_fractional_physical_edges<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let root_size = Size::splat(scalar(100.5));
    let viewport = root_size.map(AvailableOf::definite);

    for (writing_mode, direction) in root_writing_mode_directions() {
        let (
            expected_unrounded_location,
            expected_unrounded_size,
            expected_final_location,
            expected_final_size,
        ) = fractional_child_rect(writing_mode, direction);
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: root_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: expected_unrounded_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
        )
        .expect("fractional root layout succeeds");
        let unrounded_child = public_flow_output(batch.unrounded_entries(), 1);
        let final_root = public_flow_output(batch.final_entries(), 0);
        let final_child = public_flow_output(batch.final_entries(), 1);

        assert_eq!(unrounded_child.location, expected_unrounded_location);
        assert_eq!(unrounded_child.size, expected_unrounded_size);
        assert_eq!(final_child.location, expected_final_location);
        assert_eq!(final_child.size, expected_final_size);
        assert_eq!(final_root.size, Size::splat(scalar(101.0)));
    }
}

#[test]
fn ordinary_block_root_contexts_round_fractional_physical_edges_for_all_flows_f32() {
    assert_ordinary_block_root_contexts_round_fractional_physical_edges::<f32>();
}

#[test]
fn ordinary_block_root_contexts_round_fractional_physical_edges_for_all_flows_f64() {
    assert_ordinary_block_root_contexts_round_fractional_physical_edges::<f64>();
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

fn root_cache_input(available: Size<Available>) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformRootLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        available.map(Available::into_option),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        available,
    )
}

fn assert_public_scroll_geometry_error_without_batch(
    tree: &RootSessionTree,
    available: Size<Available>,
    expected_site: LayoutErrorSite<u32>,
    expected_operation: LayoutOperation,
    expected_invariant: LayoutInternalInvariant,
) {
    let request =
        LayoutRootRequest::viewport(available).expect("finite root availability is valid");
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compute_layout(tree, 0, request)
    }));
    let error = match outcome {
        Ok(Err(error)) => error,
        Ok(Ok(_)) => panic!("scroll-geometry overflow must not return a completed layout batch"),
        Err(_) => panic!("scroll-geometry overflow must not unwind from compute_layout"),
    };

    let expected_kind = LayoutErrorKind::InternalInvariant(expected_invariant);
    assert_eq!(
        (error.site(), error.operation(), error.kind()),
        (expected_site, expected_operation, &expected_kind)
    );
}

fn overflowing_scroll_edges() -> Edges<Length> {
    Edges {
        left: Length::px(f32::MAX),
        ..Edges::all(Length::ZERO)
    }
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
fn scroll_geometry_error_maps_rounding_overflow_through_the_public_front_door() {
    let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let available = Size::splat(Available::definite(f32::MAX));
    let style = NodeInput {
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(PreferredSize::px(1.0), PreferredSize::px(1.0)),
        ..NodeInput::default()
    };
    let mut output = ComputeOutput::from_outer_size(Size::new(1.0, 1.0));
    output.scroll_geometry = Some(root_test_scroll_geometry(RootTestScrollGeometryFacts {
        flow_axes,
        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
        item_is_replaced: false,
        border_box_size: Size::new(1.0, 1.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width: 0.0,
        scrollable_overflow: ScrollRect::try_new(Point::ZERO, Size::new(f32::MAX, 1.0)).unwrap(),
    }));
    let mut cache = Cache::new();
    cache.store_with_context(
        &ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            available.map(Available::into_option),
            crate::ContainingLayoutContext::new(
                flow_axes,
                crate::ParentFormattingContext::NoParent,
            ),
            available,
        ),
        CacheKeyContext::new(),
        output,
    );
    let tree: RootSessionTree = RootSessionTree::default().style(0, style);
    tree.caches.borrow_mut().insert(0, cache);

    assert_public_scroll_geometry_error_without_batch(
        &tree,
        available,
        LayoutErrorSite::Node(0),
        LayoutOperation::RoundingFinalization,
        LayoutInternalInvariant::InvalidRoundedScrollGeometry,
    );
}

struct ConstraintOverflowTree<S: LayoutScalar> {
    style: NodeInputOf<S>,
    measure_calls: Cell<usize>,
}

impl<S: LayoutScalar> Traverse for ConstraintOverflowTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Empty<u32>
    where
        Self: 'a;

    fn children(&self, _node: Self::Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Self::Node) -> usize {
        0
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        unreachable!("constraint overflow test tree has no children")
    }
}

impl<S: LayoutScalar> LayoutTree for ConstraintOverflowTree<S> {
    type MeasureError = ();

    fn node_input(&self, _node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        &self.style
    }

    fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.style.clone())
    }

    fn has_leaf_measurement(&self, _node: Self::Node) -> bool {
        true
    }

    fn measure_leaf(
        &self,
        _node: Self::Node,
        _input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.measure_calls.set(self.measure_calls.get() + 1);
        Some(Ok(Size::ZERO))
    }
}

fn assert_tree_leaf_constraint_overflow<S: LayoutScalar>(largest_finite: S) {
    let tree = ConstraintOverflowTree {
        style: NodeInputOf {
            padding: Edges::all(LengthOf::px(largest_finite)),
            ..NodeInputOf::default()
        },
        measure_calls: Cell::new(0),
    };
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(largest_finite)))
        .expect("largest finite root availability is valid");

    let error = compute_layout(&tree, 0, request)
        .expect_err("overflowing content-space arithmetic must return no completed batch");

    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
            if *value == -S::INFINITY
    ));
    assert_eq!(tree.measure_calls.get(), 0);
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
fn root_request_preserves_distinct_validated_contexts_and_rounding_policy() {
    let available = Size::new(Available::definite(640.0), Available::definite(480.0));
    let viewport = LayoutRootRequest::viewport(available).unwrap();
    let parent_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let flex_context = FlexItemRootContext::under_viewport(available, parent_axes).unwrap();
    let flex_item = LayoutRootRequest::flex_item_under_viewport(available, flex_context).unwrap();

    assert_eq!(viewport.available(), available);
    assert_eq!(
        viewport.rounding_mode(),
        LayoutRoundingMode::NearestCssPixel
    );
    assert_eq!(viewport.context(), LayoutRootContext::Viewport);
    assert_eq!(
        flex_item.context(),
        LayoutRootContext::FlexItemUnderViewport(flex_context)
    );
    assert_eq!(flex_context.viewport_available(), available);
    assert_eq!(flex_context.parent_flow_axes(), parent_axes);
}

#[test]
fn compute_layout_success_returns_completed_batch_without_tree_mutation() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.25), PreferredSize::px(20.5)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("root layout succeeds");

    assert_eq!(batch.unrounded_entries().len(), 1);
    assert_eq!(batch.unrounded_entries()[0].node(), 0);
    assert_eq!(
        batch.unrounded_entries()[0].output().size,
        Size::new(10.25, 20.5)
    );
    assert_eq!(batch.final_entries().len(), 1);
    assert_eq!(batch.final_entries()[0].node(), 0);
    assert_eq!(
        batch.final_entries()[0].output().size,
        Size::new(10.0, 21.0)
    );
}

#[test]
fn compute_layout_stages_cache_store_with_the_cold_root_output() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request).expect("cold root layout succeeds");

    assert_eq!(batch.cache_store_entries().len(), 1);
    let entry = &batch.cache_store_entries()[0];
    assert_eq!(entry.node(), 0);
    assert_eq!(entry.output().size, Size::new(10.0, 20.0));
    let mut applied_cache = Cache::new();
    applied_cache.store_with_context(entry.input(), entry.context(), entry.output());
    assert_eq!(
        applied_cache.get_with_context(entry.input(), entry.context()),
        Some(entry.output())
    );
}

#[test]
fn fri05_c05_grid_geometry_root_front_door_stages_only_canonical_ordinary_output() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(90.0)],
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("ordinary grid root layout succeeds");
    let unrounded = batch.unrounded_entries()[0].output();
    let geometry = unrounded
        .scroll_geometry
        .expect("ordinary grid root publishes canonical geometry");
    assert_eq!(geometry.used_overflow_x(), Overflow::Scroll);
    assert_eq!(geometry.used_overflow_y(), Overflow::Scroll);
    assert_eq!(geometry.target().border_box(), geometry.border_box());
    assert_eq!(unrounded.content_box_size(), geometry.content_box().size());
    assert_eq!(batch.cache_store_entries().len(), 1);
    assert_eq!(
        batch.cache_store_entries()[0]
            .output()
            .scroll_geometry
            .expect("stable root cache entry retains geometry"),
        geometry
    );
    assert!(batch.final_entries()[0].output().scroll_geometry.is_some());
}

#[test]
fn compute_layout_uses_a_matching_root_cache_hit_without_staging_a_store() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let cached = ComputeOutput::from_outer_size(Size::new(33.0, 44.0));
    let mut cache = Cache::new();
    cache.store_with_context(&input, CacheKeyContext::new(), cached);
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request).expect("cached root layout succeeds");

    assert_eq!(
        batch.unrounded_entries()[0].output().size,
        Size::new(33.0, 44.0)
    );
    assert!(batch.cache_store_entries().is_empty());
}

#[test]
fn compute_layout_root_diagnostics_reject_invalid_cached_scroll_geometry_without_batch() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(f32::NAN, 20.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let error = compute_layout(&tree, 0, request)
        .expect_err("invalid cached root output must not complete a layout batch");

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry)
    );
}

#[test]
fn compute_layout_ignores_cached_container_output_until_the_subtree_is_complete() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Ok(Size::new(12.0, 8.0)));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let cached = ComputeOutput::from_outer_size(Size::new(33.0, 44.0));
    let mut cache = Cache::new();
    cache.store_with_context(&input, CacheKeyContext::new(), cached);
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request)
        .expect("a cached container request must return a complete layout batch");

    for node in [0, 1] {
        assert!(
            batch
                .unrounded_entries()
                .iter()
                .any(|entry| entry.node() == node)
        );
        assert!(
            batch
                .final_entries()
                .iter()
                .any(|entry| entry.node() == node)
        );
    }
    assert_ne!(
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .expect("root output must be staged")
            .output()
            .size,
        cached.size
    );
    let measured_nodes = tree.measured_nodes();
    assert!(!measured_nodes.is_empty());
    assert!(measured_nodes.iter().all(|node| *node == 1));
}

#[test]
fn compute_layout_cached_container_failure_returns_no_batch() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Err("measure failed"));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(33.0, 44.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(available).unwrap();

    let error = compute_layout(&tree, 0, request)
        .expect_err("a cached container must not hide a descendant provider failure");

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::Measurement("measure failed")
    );
    assert_eq!(tree.measured_nodes(), vec![1]);
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn f32_tree_leaf_constraint_overflow_returns_typed_error_before_measurement() {
    assert_tree_leaf_constraint_overflow(f32::MAX);
}

#[test]
fn f64_tree_leaf_constraint_overflow_returns_typed_error_before_measurement() {
    assert_tree_leaf_constraint_overflow(f64::MAX);
}

#[test]
fn compute_layout_stages_hidden_root_cache_clear_without_a_store() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("hidden root layout succeeds");

    assert!(batch.cache_store_entries().is_empty());
    assert_eq!(batch.cache_clear_entries().len(), 1);
    assert_eq!(batch.cache_clear_entries()[0].node(), 0);
}

#[test]
fn compute_layout_failure_drops_staged_cache_effects_without_mutating_tree_cache() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Err("measure failed"));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(7.0, 9.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(available).unwrap();

    let result = compute_layout(&tree, 0, request);

    assert!(result.is_err());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_provider_error_returns_no_completed_batch() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Err("measure failed"));
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
        &LayoutErrorKind::Measurement("measure failed")
    );
}

#[test]
fn compute_layout_rejects_claimed_leaf_without_provider() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .leaf_without_provider(1);
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
        &LayoutErrorKind::InternalInvariant(
            LayoutInternalInvariant::MissingLeafMeasurementProvider
        )
    );
    assert_eq!(tree.measured_nodes(), vec![1]);
}

#[test]
fn compute_layout_rejects_invalid_provider_output_without_batch() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Ok(Size::new(f32::NAN, 10.0)));
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let result = compute_layout(&tree, 0, request);
    let error = match result {
        Ok(_) => panic!("invalid provider output must not complete a layout batch"),
        Err(error) => error,
    };

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(output))
            if output.axis() == PhysicalAxis::Horizontal
    ));
    let LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(output)) = error.kind()
    else {
        panic!("invalid provider output must retain its measurement diagnostic");
    };
    let NonNegativeFiniteScalarErrorOf::NonFinite { value } = output.error() else {
        panic!("invalid provider output must retain the rejected scalar");
    };
    assert!(value.is_nan());
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
fn compute_layout_rejects_measured_child_invalid_affine_width_without_batch() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::value(overflowing), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .measure(1, Ok(Size::new(12.0, 8.0)));
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        })
    );
    assert!(tree.measured_nodes().is_empty());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_rejects_measured_child_invalid_affine_padding_without_batch() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                padding: Edges::all(Length::value(overflowing)),
                ..NodeInput::default()
            },
        )
        .measure(1, Ok(Size::new(12.0, 8.0)));
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        })
    );
    assert!(tree.measured_nodes().is_empty());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_rejects_root_measured_leaf_invalid_affine_width_without_batch() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [])
        .style(
            0,
            NodeInput {
                size: Size::new(PreferredSize::value(overflowing), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .measure(0, Ok(Size::new(12.0, 8.0)));
    let before = tree.caches.borrow().clone();
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
            value: f32::INFINITY,
        })
    );
    assert!(tree.measured_nodes().is_empty());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_rejects_root_measured_leaf_invalid_affine_padding_without_batch() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [])
        .style(
            0,
            NodeInput {
                padding: Edges::all(Length::value(overflowing)),
                ..NodeInput::default()
            },
        )
        .measure(0, Ok(Size::new(12.0, 8.0)));
    let before = tree.caches.borrow().clone();
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
            value: f32::INFINITY,
        })
    );
    assert!(tree.measured_nodes().is_empty());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn fri04_c03_leaf_root_root_front_door_consumes_leaf_and_inner_display_calculations() {
    fn calculation(value: f32) -> SizingCalculation {
        SizingCalculation::value(LengthPercentageOf::px(value).expect("finite sizing value"))
    }

    fn style(display: Display) -> NodeInput {
        NodeInput {
            display,
            size: Size::new(
                PreferredSize::calculation(
                    SizingCalculation::max(vec![calculation(60.0), calculation(45.0)])
                        .expect("nonempty maximum"),
                ),
                PreferredSize::calculation(SizingCalculation::clamp(
                    Some(calculation(20.0)),
                    calculation(40.0),
                    Some(calculation(70.0)),
                )),
            ),
            min_size: Size::new(
                MinSize::calculation(
                    SizingCalculation::min(vec![calculation(-8.0), calculation(-3.0)])
                        .expect("nonempty minimum"),
                ),
                MinSize::calculation(
                    SizingCalculation::max(vec![calculation(10.0), calculation(15.0)])
                        .expect("nonempty maximum"),
                ),
            ),
            max_size: Size::new(
                MaxSize::calculation(SizingCalculation::clamp(
                    None,
                    calculation(55.0),
                    Some(calculation(90.0)),
                )),
                MaxSize::calculation(
                    SizingCalculation::max(vec![calculation(45.0), calculation(35.0)])
                        .expect("nonempty maximum"),
                ),
            ),
            ..NodeInput::default()
        }
    }

    let request = || {
        LayoutRootRequest::viewport(Size::new(
            Available::definite(100.0),
            Available::definite(80.0),
        ))
        .expect("valid root request")
    };
    let leaf: RootSessionTree = RootSessionTree::default()
        .children(0, [])
        .style(0, style(Display::Block))
        .measure(0, Ok(Size::new(1.0, 1.0)));
    let leaf_batch = compute_layout(&leaf, 0, request()).expect("root leaf layout succeeds");
    assert_eq!(
        leaf_batch.unrounded_entries()[0].output().size,
        Size::new(55.0, 40.0)
    );

    let inner: RootSessionTree = RootSessionTree::default()
        .children(0, [])
        .style(0, style(Display::Block));
    let inner_batch = compute_layout(&inner, 0, request()).expect("root block layout succeeds");
    assert_eq!(
        inner_batch.unrounded_entries()[0].output().size,
        Size::new(55.0, 40.0)
    );
}

#[test]
fn fri04_c04_leaf_block_positioned_root_reports_actual_leaf_or_block_algorithm() {
    let style = || NodeInput {
        display: Display::Block,
        size: Size::new(PreferredSize::AUTO, PreferredSize::STRETCH),
        ..NodeInput::default()
    };
    let request = || {
        LayoutRootRequest::viewport(Size::splat(Available::definite(100.0)))
            .expect("valid viewport")
    };
    let cases: [(RootSessionTree<&'static str>, SizingAlgorithm); 2] = [
        (
            RootSessionTree::default()
                .children(0, [])
                .style(0, style())
                .measure(0, Ok(Size::new(10.0, 10.0))),
            SizingAlgorithm::Leaf,
        ),
        (
            RootSessionTree::default().children(0, []).style(0, style()),
            SizingAlgorithm::Block,
        ),
    ];

    for (tree, expected_algorithm) in cases {
        let error = compute_layout(&tree, 0, request())
            .expect_err("later-owned root sizing must be rejected");
        assert_eq!(error.site(), LayoutErrorSite::Node(0));
        let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
            unsupported,
        )) = error.kind()
        else {
            panic!("expected exact root sizing capability");
        };
        assert_eq!(unsupported.property(), SizingProperty::Preferred);
        assert_eq!(unsupported.behavior(), SizingBehavior::Stretch);
        assert_eq!(unsupported.algorithm(), expected_algorithm);
        assert_eq!(unsupported.axis(), PhysicalAxis::Vertical);
    }
}

#[test]
fn fri04_c04_leaf_block_positioned_root_leaf_and_block_supported_geometry() {
    let calc_size = || {
        Size::new(
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::Any,
                CalcSizeCalculation::from_coefficients(20.0, 0.5, 0.0)
                    .expect("finite Any calculation"),
            )
            .expect("valid Any calc-size"),
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::FullPercentage,
                CalcSizeCalculation::from_coefficients(10.0, 0.0, 0.5)
                    .expect("finite FullPercentage calculation"),
            )
            .expect("valid FullPercentage calc-size"),
        )
    };
    let request = || {
        LayoutRootRequest::viewport(Size::new(
            Available::definite(200.0),
            Available::definite(160.0),
        ))
        .expect("valid viewport")
    };
    let leaf: RootSessionTree<&'static str> = RootSessionTree::default()
        .children(0, [])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: calc_size(),
                ..NodeInput::default()
            },
        )
        .measure(0, Ok(Size::new(1.0, 1.0)));
    let leaf_batch = compute_layout(&leaf, 0, request()).expect("root leaf calc-size resolves");
    assert_eq!(
        leaf_batch.unrounded_entries()[0].output().size,
        Size::new(120.0, 90.0)
    );

    for intrinsic in [PreferredSize::MIN_CONTENT, PreferredSize::MAX_CONTENT] {
        let block: RootSessionTree<&'static str> =
            RootSessionTree::default().children(0, []).style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(intrinsic.clone(), intrinsic),
                    ..NodeInput::default()
                },
            );
        let batch = compute_layout(&block, 0, request())
            .expect("root block preferred intrinsic sizing resolves");
        assert_eq!(batch.unrounded_entries()[0].output().size, Size::ZERO);
    }
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
fn compute_layout_preserves_nested_subgrid_resolution_failure() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::from(
                    LengthPercentageOf::px(20.0).expect("finite track"),
                )],
                grid_template_rows: vec![TrackComponent::from(
                    LengthPercentageOf::px(20.0).expect("finite track"),
                )],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponent::from(overflowing)],
                size: Size::new(PreferredSize::AUTO, PreferredSize::px(f32::MAX)),
                ..NodeInput::default()
            },
        );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(20.0),
        Available::definite(20.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
    ));
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

struct SingleRootTree {
    style: NodeInput,
    output: ComputeOutput,
    layouts: HashMap<u32, NodeOutput>,
    input: Option<ComputeInput>,
}

impl SingleRootTree {
    fn new(style: NodeInput) -> Self {
        Self {
            style,
            output: ComputeOutput::from_outer_size(Size::ZERO),
            layouts: HashMap::new(),
            input: None,
        }
    }
}

impl Traverse for SingleRootTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Empty<u32>;

    fn children(&self, _node: Self::Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Self::Node) -> usize {
        0
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        unreachable!("root test tree has no children")
    }
}

impl Compute for SingleRootTree {
    fn node_input(&self, _node: Self::Node) -> &NodeInput {
        &self.style
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
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        Ok({
            self.input = Some(input);
            self.output
        })
    }
}

#[test]
fn root_layout_emits_scroll_geometry_for_scroll_overflow() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
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
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), None);
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(130.0, 70.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn root_layout_emits_clip_geometry_without_range() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: computed_overflow(Overflow::Clip, Overflow::Clip),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn root_scroll_geometry_range_accounts_for_padding_border_and_gutter() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
        padding: Edges::all(Length::px(2.0)),
        border: Edges::all(Length::px(3.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
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
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
        ..NodeInput::default()
    });
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
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));
    tree.output.scroll_geometry = Some(child_geometry);

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
    assert_positive_physical_range(geometry.physical_range(), Size::new(48.0, 30.0));
}

#[test]
fn f64_round_layout_preserves_large_coordinates() {
    let large = 16_777_217.25_f64;
    let mut tree = OracleTreeOf::<f64>::new()
        .style(0, NodeInputOf::<f64>::default())
        .unrounded(
            0,
            NodeOutputOf::<f64> {
                location: Point::new(large, large + 0.5),
                size: Size::new(10.5, 20.25),
                ..NodeOutputOf::<f64>::default()
            },
        );

    round_layout(&mut tree, 0).unwrap();

    let final_layout = tree
        .output(0)
        .expect("rounding must stage final output for the root node");
    assert_eq!(final_layout.location.x, large.round());
    assert_eq!(final_layout.location.y, (large + 0.5).round());
}

#[test]
fn round_layout_rounds_scroll_geometry_with_node_output() {
    let mut tree = OracleTreeOf::<f64>::new().unrounded(
        0,
        NodeOutputOf::<f64> {
            location: Point::new(10.25, 20.25),
            size: Size::new(100.5, 40.5),
            content_size: Size::new(120.5, 70.5),
            scroll_geometry: Some(root_test_scroll_geometry(RootTestScrollGeometryFacts {
                flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                item_is_replaced: false,
                border_box_size: Size::new(100.5, 40.5),
                padding: Edges::ZERO,
                border: Edges::all(0.25),
                scrollbar_width: 0.0,
                scrollable_overflow: ScrollRectOf::try_new(
                    Point::new(0.25, 0.25),
                    Size::new(120.5, 70.5),
                )
                .unwrap(),
            })),
            ..NodeOutputOf::<f64>::default()
        },
    );

    round_layout(&mut tree, 0).unwrap();

    let geometry = tree
        .output(0)
        .expect("rounding must stage final output for the root node")
        .scroll_geometry
        .unwrap();
    assert_eq!(geometry.scrollport().origin(), Point::new(1.0, 1.0));
    assert_eq!(geometry.scrollport().size(), Size::new(100.0, 40.0));
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(1.0, 1.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(120.0, 70.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(20.0, 30.0));
}

#[test]
fn round_layout_diagnostics_rejects_invalid_rounded_scroll_geometry() {
    let scrollable_overflow = ScrollRect::try_new(Point::new(f32::MAX, 0.0), Size::ZERO).unwrap();
    let scroll_geometry = root_test_scroll_geometry(RootTestScrollGeometryFacts {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
        item_is_replaced: false,
        border_box_size: Size::new(1.0, 1.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width: 0.0,
        scrollable_overflow,
    });
    let mut tree = OracleTreeOf::<f32>::new().unrounded(
        0,
        NodeOutput {
            location: Point::new(f32::MAX, 0.0),
            scroll_geometry: Some(scroll_geometry),
            ..NodeOutput::new()
        },
    );

    let error = round_layout(&mut tree, 0)
        .expect_err("invalid rounded scroll geometry must not stage final output");

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InternalInvariant(LayoutInternalInvariant::InvalidRoundedScrollGeometry)
    );
    assert_eq!(tree.final_layout(0), None);
}

#[test]
fn root_layout_stores_child_output_as_root_layout() {
    #[derive(Default)]
    struct RootTree {
        style: NodeInput,
        layout: Option<NodeOutput>,
        input: Option<ComputeInput>,
    }

    impl Traverse for RootTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("root has no children in this test")
        }
    }

    impl Compute for RootTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, layout: NodeOutput) {
            self.layout = Some(layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.input = Some(input);
                ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0))
            })
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
            direction: Direction::Rtl,
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(13.0).unwrap(),
            ..NodeInput::default()
        },
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    )
    .unwrap();

    assert_eq!(
        tree.input,
        Some(ComputeInput::for_child(
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
        ))
    );
    let layout = tree.layout.expect("root layout should be stored");
    assert_eq!(layout.location, crate::Point::new(120.0, 0.0));
    assert_eq!(layout.size, Size::new(80.0, 20.0));
    assert_eq!(layout.content_size, Size::new(80.0, 20.0));
    assert_eq!(layout.scrollbar_size(), Size::new(13.0, 13.0));
}

#[test]
fn inline_level_root_keeps_intrinsic_width_under_definite_viewport() {
    #[derive(Default)]
    struct RootTree {
        style: NodeInput,
        layout: Option<NodeOutput>,
        input: Option<ComputeInput>,
    }

    impl Traverse for RootTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("root has no children in this test")
        }
    }

    impl Compute for RootTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, layout: NodeOutput) {
            self.layout = Some(layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.input = Some(input);
                ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0))
            })
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
            display: Display::InlineGrid,
            ..NodeInput::default()
        },
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    )
    .unwrap();

    assert_eq!(
        tree.input.expect("root should be computed").known(),
        Size::NONE
    );
    assert_eq!(
        tree.layout.expect("root layout should be stored").size,
        Size::new(80.0, 20.0)
    );
}

#[test]
fn max_width_root_uses_clamped_available_width_under_definite_viewport() {
    #[derive(Default)]
    struct RootTree {
        style: NodeInput,
        layout: Option<NodeOutput>,
        input: Option<ComputeInput>,
    }

    impl Traverse for RootTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("root has no children in this test")
        }
    }

    impl Compute for RootTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, layout: NodeOutput) {
            self.layout = Some(layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.input = Some(input);
                let width = input.known().width.unwrap_or(272.0);
                ComputeOutput::from_sizes(Size::new(width, 72.0), Size::new(width, 72.0))
            })
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
            display: Display::Grid,
            max_size: Size::new(MaxSize::px(260.0), MaxSize::NONE),
            ..NodeInput::default()
        },
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(800.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.input.expect("root should be computed").known(),
        Size::new(Some(260.0), None)
    );
    assert_eq!(
        tree.layout.expect("root layout should be stored").size,
        Size::new(260.0, 72.0)
    );
}

#[test]
fn block_root_with_max_width_uses_clamped_available_outer_width() {
    #[derive(Default)]
    struct RootTree {
        style: NodeInput,
        layout: Option<NodeOutput>,
        input: Option<ComputeInput>,
    }

    impl Traverse for RootTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("root has no children in this test")
        }
    }

    impl Compute for RootTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, layout: NodeOutput) {
            self.layout = Some(layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.input = Some(input);
                ComputeOutput::from_sizes(
                    Size::new(input.known().width.unwrap_or(112.0), 20.0),
                    Size::new(input.known().width.unwrap_or(112.0), 20.0),
                )
            })
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
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
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(800.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.input.expect("root should be computed").known().width,
        Some(272.0)
    );
    assert_eq!(
        tree.layout
            .expect("root layout should be stored")
            .size
            .width,
        272.0
    );
}

#[test]
fn round_layout_uses_cumulative_viewport_edges() {
    #[derive(Default)]
    struct RoundTree {
        children: HashMap<u32, Vec<u32>>,
        unrounded: HashMap<u32, NodeOutput>,
        final_layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for RoundTree {
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

    impl Round for RoundTree {
        fn unrounded(
            &self,
            node: Self::Node,
        ) -> crate::LayoutResultOf<Self::Node, NodeOutput, Self::Scalar> {
            Ok(self.unrounded[&node])
        }

        fn set_final(&mut self, node: Self::Node, layout: NodeOutput) {
            self.final_layouts.insert(node, layout);
        }
    }

    let mut tree = RoundTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.unrounded.insert(
        1,
        NodeOutput {
            location: Point::new(0.2, 0.0),
            size: Size::new(10.4, 10.0),
            content_size: Size::new(10.4, 10.0),
            border: Edges::all(0.4),
            padding: Edges::all(0.6),
            ..NodeOutput::new()
        },
    );
    tree.unrounded.insert(
        2,
        NodeOutput {
            location: Point::new(-0.5, 0.0),
            size: Size::new(10.0, 10.0),
            content_size: Size::new(10.0, 10.0),
            border: Edges::all(0.6),
            padding: Edges::all(0.4),
            ..NodeOutput::new()
        },
    );

    round_layout(&mut tree, 1).unwrap();

    assert_eq!(tree.final_layouts[&1].location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layouts[&1].size.width, 11.0);
    assert_eq!(tree.final_layouts[&1].content_size.width, 11.0);
    assert_eq!(tree.final_layouts[&1].border.left, 1.0);
    assert_eq!(tree.final_layouts[&1].border.right, 1.0);
    assert_eq!(tree.final_layouts[&1].padding.left, 1.0);
    assert_eq!(tree.final_layouts[&1].padding.right, 1.0);

    assert_eq!(tree.final_layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layouts[&2].size.width, 10.0);
    assert_eq!(tree.final_layouts[&2].content_size.width, 10.0);
    assert_eq!(tree.final_layouts[&2].scrollbar_size(), Size::ZERO);
    assert_eq!(tree.final_layouts[&2].border.left, 0.0);
    assert_eq!(tree.final_layouts[&2].border.right, 1.0);
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

struct Fri05C03MeasuredLeafTree<S: LayoutScalar = f32> {
    style: NodeInputOf<S>,
    measured: Size<S>,
    measurement_inputs: RefCell<Vec<LeafMeasureInputOf<S>>>,
}

impl<S: LayoutScalar> Traverse for Fri05C03MeasuredLeafTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a> = std::iter::Empty<u32>;

    fn children(&self, _node: Self::Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Self::Node) -> usize {
        0
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        unreachable!("FRI-05 measured-leaf tree has no children")
    }
}

impl<S: LayoutScalar> LayoutTree for Fri05C03MeasuredLeafTree<S> {
    type MeasureError = ();

    fn node_input(&self, _node: Self::Node) -> &NodeInputOf<S> {
        &self.style
    }

    fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.style.clone())
    }

    fn has_leaf_measurement(&self, _node: Self::Node) -> bool {
        true
    }

    fn measure_leaf(
        &self,
        _node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.measurement_inputs.borrow_mut().push(input);
        Some(Ok(self.measured))
    }
}

fn fri05_c03_root_measurement_size(input: LeafMeasureInput) -> Size<f32> {
    assert_eq!(input.known_content_size(), Size::NONE);
    input.available_content_size().map(|available| {
        available
            .definite_value()
            .expect("FRI-05 root measurement availability is definite")
            .get()
    })
}

fn fri05_c03_tree_leaf_layout(
    style: NodeInput,
    measured: Size<f32>,
    available: Size<f32>,
) -> (NodeOutput, Vec<Size<f32>>) {
    let tree = Fri05C03MeasuredLeafTree {
        style,
        measured,
        measurement_inputs: RefCell::new(Vec::new()),
    };
    let request = LayoutRootRequest::viewport(available.map(Available::definite))
        .expect("FRI-05 root request is valid");
    let batch = compute_layout(&tree, 0, request).expect("tree-backed measured leaf succeeds");
    let output = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .expect("tree-backed leaf stages its unrounded root output")
        .output();
    let inputs = tree
        .measurement_inputs
        .borrow()
        .iter()
        .copied()
        .map(fri05_c03_root_measurement_size)
        .collect();
    (output, inputs)
}

fn fri05_c03_root_gutter_at(
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

fn fri05_c03_root_all_flow_axes() -> [FlowAxes; 10] {
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

#[test]
fn fri05_c03_round_cache_flex_item_root_keeps_cached_geometry_through_rounded_publication() {
    let own_flow_axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl);
    let parent_flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let scroll_margin = ScrollMargin::try_new(-1.0, 2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::End);
    let tree = Fri05C03MeasuredLeafTree {
        style: NodeInput {
            display: Display::Block,
            writing_mode: own_flow_axes.writing_mode(),
            direction: own_flow_axes.direction(),
            overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(3.6).unwrap(),
            size: Size::new(PreferredSize::px(100.4), PreferredSize::px(80.6)),
            scroll_margin,
            scroll_snap_type: ScrollSnapType::Enabled {
                axis: ScrollSnapAxis::Inline,
                strictness: ScrollSnapStrictness::Proximity,
            },
            scroll_snap_align: snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInput::default()
        },
        measured: Size::new(120.25, 90.75),
        measurement_inputs: RefCell::new(Vec::new()),
    };
    let available = Size::new(Available::definite(140.0), Available::definite(120.0));
    let viewport = Size::new(Available::definite(800.0), Available::definite(600.0));
    let context = FlexItemRootContext::under_viewport(viewport, parent_flow_axes).unwrap();
    let request = LayoutRootRequest::flex_item_under_viewport(available, context).unwrap();
    let batch = compute_layout(&tree, 0, request)
        .expect("flex-item measured root publishes cached and rounded geometry");

    assert_eq!(batch.cache_store_entries().len(), 1);
    let cached = batch.cache_store_entries()[0]
        .output()
        .scroll_geometry
        .expect("the stable cached result retains geometry");
    assert_eq!(cached.flow_axes(), own_flow_axes);
    assert_eq!(cached.used_overflow_x(), Overflow::Hidden);
    assert_eq!(cached.used_overflow_y(), Overflow::Scroll);
    assert_eq!(cached.target().scroll_margin(), scroll_margin);
    assert_eq!(cached.target().snap_align(), snap_align);
    assert_eq!(cached.target().snap_stop(), ScrollSnapStop::Always);

    let unrounded = batch.unrounded_entries()[0].output();
    let unrounded_geometry = unrounded
        .scroll_geometry
        .expect("flex-item root publication preserves cached geometry");
    assert_eq!(unrounded_geometry, cached);
    assert_eq!(unrounded.scrollbar_size(), cached.scrollbar_size());

    let rounded = batch.final_entries()[0].output();
    let rounded_geometry = rounded
        .scroll_geometry
        .expect("rounding preserves present flex-item root geometry");
    assert_eq!(rounded_geometry.flow_axes(), own_flow_axes);
    assert_eq!(rounded_geometry.used_overflow_x(), Overflow::Hidden);
    assert_eq!(rounded_geometry.used_overflow_y(), Overflow::Scroll);
    assert_eq!(rounded_geometry.target().scroll_margin(), scroll_margin);
    assert_eq!(rounded_geometry.target().flow_axes(), own_flow_axes);
    assert_eq!(rounded_geometry.target().snap_align(), snap_align);
    assert_eq!(
        rounded_geometry.target().snap_stop(),
        ScrollSnapStop::Always
    );
    assert_eq!(
        rounded_geometry.target().border_box(),
        rounded_geometry.border_box()
    );
    assert_eq!(
        rounded.content_box_size(),
        rounded_geometry.content_box().size()
    );
    assert_eq!(rounded.scrollbar_size(), rounded_geometry.scrollbar_size());
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

fn fri05_c03_tree_leaf_auto_case(
    style: NodeInput,
    measured: Size<f32>,
    expected_inputs: &[Size<f32>],
    expected_content_box: Size<f32>,
    expected_scrollbar_size: Size<f32>,
) {
    let (output, inputs) = fri05_c03_tree_leaf_layout(style, measured, Size::new(100.0, 100.0));
    assert_eq!(inputs, expected_inputs);
    let geometry = output
        .scroll_geometry
        .expect("tree-backed leaf publishes stable geometry");
    assert_eq!(geometry.content_box().size(), expected_content_box);
    assert_eq!(geometry.scrollbar_size(), expected_scrollbar_size);
    assert_eq!(output.scrollbar_size(), expected_scrollbar_size);
}

#[test]
fn fri05_c03_leaf_auto_tree_backed_runs_exact_monotone_passes() {
    let automatic = NodeInput {
        overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
        scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        ..NodeInput::default()
    };
    fri05_c03_tree_leaf_auto_case(
        automatic.clone(),
        Size::new(120.0, 100.0),
        &[
            Size::new(100.0, 100.0),
            Size::new(100.0, 85.0),
            Size::new(85.0, 85.0),
        ],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_tree_leaf_auto_case(
        automatic.clone(),
        Size::new(100.0, 120.0),
        &[
            Size::new(100.0, 100.0),
            Size::new(85.0, 100.0),
            Size::new(85.0, 85.0),
        ],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_tree_leaf_auto_case(
        automatic.clone(),
        Size::new(80.0, 80.0),
        &[Size::new(100.0, 100.0)],
        Size::new(100.0, 100.0),
        Size::ZERO,
    );
    fri05_c03_tree_leaf_auto_case(
        automatic.clone(),
        Size::new(120.0, 80.0),
        &[Size::new(100.0, 100.0), Size::new(100.0, 85.0)],
        Size::new(100.0, 85.0),
        Size::new(0.0, 15.0),
    );
    fri05_c03_tree_leaf_auto_case(
        automatic,
        Size::new(80.0, 120.0),
        &[Size::new(100.0, 100.0), Size::new(85.0, 100.0)],
        Size::new(85.0, 100.0),
        Size::new(15.0, 0.0),
    );
    fri05_c03_tree_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(80.0, 80.0),
        &[Size::new(85.0, 85.0)],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_tree_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::Stable,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(80.0, 80.0),
        &[Size::new(85.0, 100.0)],
        Size::new(85.0, 100.0),
        Size::new(15.0, 0.0),
    );
    fri05_c03_tree_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(60.0, 80.0),
        &[Size::new(70.0, 100.0)],
        Size::new(70.0, 100.0),
        Size::new(30.0, 0.0),
    );
    fri05_c03_tree_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::ZERO,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(120.0, 80.0),
        &[Size::new(100.0, 100.0)],
        Size::new(100.0, 100.0),
        Size::ZERO,
    );
}

fn fri05_c04_flex_geometry_overflow_at_flow_axes(
    flow_axes: FlowAxes,
    inline: Overflow,
    block: Overflow,
) -> ComputedOverflow {
    match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => computed_overflow(inline, block),
        PhysicalAxis::Vertical => computed_overflow(block, inline),
    }
}

fn fri05_c04_flex_geometry_assert_zero_range(geometry: ScrollGeometry, context: &str) {
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
fn fri05_c04_flex_geometry_rounded_publication_excludes_reserved_gutters_all_flows() {
    let regular_size = Size::new(100.0, 80.0);

    for flow_axes in fri05_c03_root_all_flow_axes() {
        let assert_case = |case: &str,
                           size: Size<f32>,
                           overflow: ComputedOverflow,
                           gutter: ScrollbarGutter,
                           scrollbar_width: f32,
                           expected_sides: &[PhysicalSide],
                           expected_thickness: f32| {
            let tree = PublicFlowTree::default().with_children(0, []).with_style(
                0,
                NodeInput {
                    display: Display::Flex,
                    writing_mode: flow_axes.writing_mode(),
                    direction: flow_axes.direction(),
                    overflow,
                    scrollbar_gutter: gutter,
                    scrollbar_width: ScrollbarWidth::try_new(scrollbar_width).unwrap(),
                    size: Size::new(
                        PreferredSize::px(size.width),
                        PreferredSize::px(size.height),
                    ),
                    ..NodeInput::default()
                },
            );
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequest::viewport(size.map(Available::definite)).unwrap(),
            )
            .unwrap_or_else(|error| {
                panic!("{case}/{flow_axes:?} public flex layout succeeds: {error:?}")
            });
            assert_eq!(batch.unrounded_entries().len(), 1, "{case}/{flow_axes:?}");
            assert_eq!(batch.final_entries().len(), 1, "{case}/{flow_axes:?}");

            let unrounded = batch.unrounded_entries()[0]
                .output()
                .scroll_geometry
                .expect("unrounded flex root retains canonical geometry");
            fri05_c04_flex_geometry_assert_zero_range(
                unrounded,
                &format!("unrounded {case}/{flow_axes:?}"),
            );

            let output = batch.final_entries()[0].output();
            let geometry = output
                .scroll_geometry
                .expect("rounded flex root retains canonical geometry");
            let expected_padding_box = ScrollRect::try_new(Point::ZERO, size).unwrap();
            let thickness = |side| {
                if expected_sides.contains(&side) {
                    expected_thickness
                } else {
                    0.0
                }
            };
            let top = thickness(PhysicalSide::Top);
            let right = thickness(PhysicalSide::Right);
            let bottom = thickness(PhysicalSide::Bottom);
            let left = thickness(PhysicalSide::Left);
            let expected_scrollport = ScrollRect::try_new(
                Point::new(left, top),
                Size::new(size.width - left - right, size.height - top - bottom),
            )
            .unwrap();

            assert_eq!(output.size, size, "{case}/{flow_axes:?}");
            assert_eq!(geometry.flow_axes(), flow_axes, "{case}/{flow_axes:?}");
            assert_eq!(
                geometry.border_box(),
                expected_padding_box,
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                geometry.padding_box(),
                expected_padding_box,
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                geometry.scrollable_overflow(),
                expected_padding_box,
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                geometry.scrollport(),
                expected_scrollport,
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                geometry.content_box(),
                expected_scrollport,
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                output.content_box_size(),
                expected_scrollport.size(),
                "{case}/{flow_axes:?}"
            );

            let expected_gutter = |side| {
                let side_thickness = thickness(side);
                if side_thickness == 0.0 {
                    return None;
                }
                let origin = expected_padding_box.origin();
                let padding_size = expected_padding_box.size();
                let scrollport_origin = expected_scrollport.origin();
                let scrollport_size = expected_scrollport.size();
                let (origin, gutter_size) = match side {
                    PhysicalSide::Top => (
                        Point::new(scrollport_origin.x, origin.y),
                        Size::new(scrollport_size.width, side_thickness),
                    ),
                    PhysicalSide::Right => (
                        Point::new(
                            origin.x + padding_size.width - side_thickness,
                            scrollport_origin.y,
                        ),
                        Size::new(side_thickness, scrollport_size.height),
                    ),
                    PhysicalSide::Bottom => (
                        Point::new(
                            scrollport_origin.x,
                            origin.y + padding_size.height - side_thickness,
                        ),
                        Size::new(scrollport_size.width, side_thickness),
                    ),
                    PhysicalSide::Left => (
                        Point::new(origin.x, scrollport_origin.y),
                        Size::new(side_thickness, scrollport_size.height),
                    ),
                };
                Some(ScrollRect::try_new(origin, gutter_size).unwrap())
            };
            for side in [
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
            ] {
                assert_eq!(
                    fri05_c03_root_gutter_at(geometry.gutters(), side),
                    expected_gutter(side),
                    "{case}/{flow_axes:?}/{side:?}"
                );
            }

            let expected_scrollbar_size = Size::new(left + right, top + bottom);
            assert_eq!(
                geometry.scrollbar_size(),
                expected_scrollbar_size,
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                output.scrollbar_size(),
                expected_scrollbar_size,
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                geometry.target().border_box(),
                geometry.border_box(),
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                geometry.target().flow_axes(),
                flow_axes,
                "{case}/{flow_axes:?}"
            );

            let x_clip = geometry.overflow_clip().x().expect("x clip is present");
            let y_clip = geometry.overflow_clip().y().expect("y clip is present");
            assert_eq!(
                (x_clip.minimum(), x_clip.maximum()),
                (
                    expected_scrollport.origin().x,
                    expected_scrollport.origin().x + expected_scrollport.size().width,
                ),
                "{case}/{flow_axes:?}"
            );
            assert_eq!(
                (y_clip.minimum(), y_clip.maximum()),
                (
                    expected_scrollport.origin().y,
                    expected_scrollport.origin().y + expected_scrollport.size().height,
                ),
                "{case}/{flow_axes:?}"
            );
            fri05_c04_flex_geometry_assert_zero_range(
                geometry,
                &format!("rounded {case}/{flow_axes:?}"),
            );
        };

        let one_edge = [flow_axes.inline_end()];
        let both_edges = [flow_axes.inline_start(), flow_axes.inline_end()];
        assert_case(
            "forced",
            regular_size,
            fri05_c04_flex_geometry_overflow_at_flow_axes(
                flow_axes,
                Overflow::Hidden,
                Overflow::Scroll,
            ),
            ScrollbarGutter::Auto,
            7.0,
            &one_edge,
            7.0,
        );
        assert_case(
            "stable",
            regular_size,
            fri05_c04_flex_geometry_overflow_at_flow_axes(
                flow_axes,
                Overflow::Hidden,
                Overflow::Hidden,
            ),
            ScrollbarGutter::Stable,
            7.0,
            &one_edge,
            7.0,
        );
        assert_case(
            "stable-both-edges",
            regular_size,
            fri05_c04_flex_geometry_overflow_at_flow_axes(
                flow_axes,
                Overflow::Hidden,
                Overflow::Hidden,
            ),
            ScrollbarGutter::StableBothEdges,
            7.0,
            &both_edges,
            7.0,
        );

        let tiny_size = Size::new(5.0, 3.0);
        let saturated_thickness = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => tiny_size.width / 2.0,
            PhysicalAxis::Vertical => tiny_size.height / 2.0,
        };
        assert_case(
            "saturated-tiny",
            tiny_size,
            fri05_c04_flex_geometry_overflow_at_flow_axes(
                flow_axes,
                Overflow::Hidden,
                Overflow::Hidden,
            ),
            ScrollbarGutter::StableBothEdges,
            10.0,
            &both_edges,
            saturated_thickness,
        );
    }
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

#[test]
fn fri05_c04_flex_child_geometry_public_auto_max_tiny_gutter_rounds_absolute_all_flows() {
    let available_size = Size::new(100.0, 100.0);

    for flow_axes in fri05_c03_root_all_flow_axes() {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
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
            )
            .with_style(
                1,
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
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequest::viewport(available_size.map(Available::definite)).unwrap(),
        )
        .unwrap_or_else(|error| {
            panic!("public tiny absolute flex succeeds for {flow_axes:?}: {error:?}")
        });

        for (phase, entries) in [
            ("unrounded", batch.unrounded_entries()),
            ("rounded", batch.final_entries()),
        ] {
            let root = public_flow_output(entries, 0);
            let root_geometry = root
                .scroll_geometry
                .unwrap_or_else(|| panic!("{phase} root retains geometry for {flow_axes:?}"));
            let absolute = public_flow_output(entries, 1);
            let absolute_geometry = absolute.scroll_geometry.unwrap_or_else(|| {
                panic!("{phase} absolute child retains geometry for {flow_axes:?}")
            });

            assert_eq!(root.size, Size::new(100.0, 5.0), "{phase}/{flow_axes:?}");
            assert_eq!(
                root_geometry.scrollport().size(),
                Size::new(90.0, 0.0),
                "{phase}/{flow_axes:?}"
            );
            assert_eq!(absolute.size, Size::ZERO, "{phase}/{flow_axes:?}");
            assert_eq!(absolute_geometry.border_box().size(), Size::ZERO);
            assert_eq!(
                absolute_geometry.target().border_box(),
                absolute_geometry.border_box()
            );
            assert_eq!(
                absolute.location.y,
                root_geometry.scrollport().origin().y + root_geometry.scrollport().size().height,
                "{phase} bottom: 0 uses the saturated scrollport for {flow_axes:?}"
            );
        }
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

fn fri05_c03_block_root_state(input: &ComputeInput) -> (bool, bool) {
    let state = input.containing_auto_scrollbar_pass();
    (
        state.at(PhysicalAxis::Horizontal),
        state.at(PhysicalAxis::Vertical),
    )
}

fn fri05_c04_local_auto_state(input: &ComputeInput) -> (bool, bool) {
    let state = input.settled_auto_scrollbars();
    (
        state.at(PhysicalAxis::Horizontal),
        state.at(PhysicalAxis::Vertical),
    )
}

fn fri05_c04_flex_auto_public_tree(nested: bool, child_size: Size<f32>) -> PublicFlowTree<f32> {
    let container = NodeInput {
        display: Display::Flex,
        overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
        scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        align_items: Some(AlignItems::FlexStart),
        ..NodeInput::default()
    };
    let absolute = NodeInput {
        display: Display::Block,
        position: Position::Absolute,
        size: child_size.map(PreferredSize::px),
        inset: Edges::new(
            LengthAuto::px(0.0),
            LengthAuto::AUTO,
            LengthAuto::AUTO,
            LengthAuto::px(0.0),
        ),
        ..NodeInput::default()
    };

    if nested {
        PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [2])
            .with_children(2, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(1, container)
            .with_style(2, absolute)
    } else {
        PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, container)
            .with_style(1, absolute)
    }
}

fn fri05_c04_nested_flex_auto_tree(inner_overflows: bool) -> PublicFlowTree<f32> {
    let outer_children = if inner_overflows { vec![1] } else { vec![1, 3] };
    let inner_child = if inner_overflows {
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(60.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            inset: Edges::new(
                LengthAuto::px(0.0),
                LengthAuto::AUTO,
                LengthAuto::AUTO,
                LengthAuto::px(0.0),
            ),
            ..NodeInput::default()
        }
    } else {
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            ..NodeInput::default()
        }
    };

    let mut tree = PublicFlowTree::default()
        .with_children(0, outer_children)
        .with_children(1, [2])
        .with_children(2, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Flex,
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
                display: Display::Flex,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                size: Size::new(PreferredSize::px(40.0), PreferredSize::AUTO),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .with_style(2, inner_child);

    if !inner_overflows {
        tree = tree.with_children(3, []).with_style(
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
    }
    tree
}

fn fri05_c04_assert_initial_local_auto_state(inputs: &[ComputeInput]) {
    assert!(
        inputs
            .iter()
            .all(|input| fri05_c04_local_auto_state(input) == (false, false)),
        "every recursively dispatched node starts local auto settlement at INITIAL: {inputs:#?}"
    );
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
        let (batch, hidden_requests) =
            crate::compute::trace_hidden_compute_session_requests(|| {
                compute_layout(&tree, 0, request)
                    .expect("auto container with hidden subtree lays out")
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
fn fri05_c04_flex_auto_nested_nonoverflow_keeps_local_initial_and_outer_cache_passes() {
    let tree = fri05_c04_nested_flex_auto_tree(false);
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    let cold = compute_layout(&tree, 0, request).expect("cold nested flex layout succeeds");
    let outer = public_flow_output(cold.unrounded_entries(), 0);
    let inner = public_flow_output(cold.unrounded_entries(), 1);
    assert_eq!(
        outer.scroll_geometry.unwrap().scrollbar_size(),
        Size::new(0.0, 15.0)
    );
    assert_eq!(inner.scroll_geometry.unwrap().scrollbar_size(), Size::ZERO);

    let grandchild_requests = tree.cache_inputs(2);
    fri05_c04_assert_initial_local_auto_state(&grandchild_requests);
    assert!(
        grandchild_requests
            .iter()
            .all(|input| fri05_c03_block_root_state(input) == (false, false))
    );
    for node in [0, 1, 2, 3] {
        assert_eq!(
            cold.unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only stable cold output is published for node {node}"
        );
    }

    tree.apply_cache_entries(cold.cache_store_entries());
    tree.clear_cache_inputs();
    let warm = compute_layout(&tree, 0, request).expect("warm nested flex layout succeeds");
    for node in [0, 1, 2, 3] {
        assert_eq!(
            public_flow_output(warm.unrounded_entries(), node),
            public_flow_output(cold.unrounded_entries(), node),
            "warm output remains the stable cold output for node {node}"
        );
        assert_eq!(
            warm.unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only stable warm output is published for node {node}"
        );
    }
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
fn fri05_c04_flex_auto_root_and_nested_publish_stable_output_with_exact_pass_cache_bits() {
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    for nested in [false, true] {
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
            let tree = fri05_c04_flex_auto_public_tree(nested, child_size);
            let container = u32::from(nested);
            let absolute = container + 1;
            let cold = compute_layout(&tree, 0, request).expect("cold flex auto layout succeeds");
            let output = public_flow_output(cold.unrounded_entries(), container);
            assert_eq!(
                output.scroll_geometry.unwrap().scrollbar_size(),
                expected_scrollbars,
                "nested={nested}, child={child_size:?}"
            );
            for node in [container, absolute] {
                assert_eq!(
                    cold.unrounded_entries()
                        .iter()
                        .filter(|entry| entry.node() == node)
                        .count(),
                    1,
                    "only stable node {node} output is published for nested={nested}"
                );
            }

            let cache_inputs = tree.cache_inputs(absolute);
            assert!(
                cache_inputs
                    .iter()
                    .all(|input| fri05_c04_local_auto_state(input) == (false, false)),
                "nested={nested}, child={child_size:?}: child-local state must start at INITIAL"
            );
            assert_eq!(
                cache_inputs
                    .iter()
                    .map(fri05_c03_block_root_state)
                    .collect::<Vec<_>>(),
                expected_states,
                "nested={nested}, child={child_size:?}: {cache_inputs:#?}"
            );
            assert_eq!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| {
                        entry.node() == absolute
                            && entry.input().run_mode() == RunMode::PerformLayout
                    })
                    .map(|entry| fri05_c03_block_root_state(entry.input()))
                    .collect::<Vec<_>>(),
                expected_states,
                "nested={nested}, child={child_size:?}"
            );
            assert!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| entry.node() == absolute)
                    .all(|entry| fri05_c04_local_auto_state(entry.input()) == (false, false)),
                "nested={nested}, child={child_size:?}: cached child-local state stays INITIAL"
            );
            let child_cache_inputs = cold
                .cache_store_entries()
                .iter()
                .filter(|entry| {
                    entry.node() == absolute && entry.input().run_mode() == RunMode::PerformLayout
                })
                .map(LayoutCacheStoreEntryOf::input)
                .collect::<Vec<_>>();
            assert!(
                child_cache_inputs
                    .iter()
                    .all(|input| input.known() == child_cache_inputs[0].known()),
                "distinct containing passes stage separate entries for identical known child geometry"
            );
            assert!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| entry.node() == container)
                    .all(|entry| fri05_c03_block_root_state(entry.input()) == (false, false)),
                "no speculative container pass is cached under an ordinary request"
            );

            tree.apply_cache_entries(cold.cache_store_entries());
            tree.clear_cache_inputs();
            let warm = compute_layout(&tree, 0, request).expect("warm flex auto layout succeeds");
            assert_eq!(
                public_flow_output(warm.unrounded_entries(), container),
                public_flow_output(cold.unrounded_entries(), container),
                "nested={nested}, child={child_size:?}"
            );
            assert_eq!(
                public_flow_output(warm.final_entries(), container),
                public_flow_output(cold.final_entries(), container),
                "nested={nested}, child={child_size:?}"
            );
            for node in [container, absolute] {
                assert_eq!(
                    warm.unrounded_entries()
                        .iter()
                        .filter(|entry| entry.node() == node)
                        .count(),
                    1,
                    "only stable warm output is published for node {node}"
                );
            }
        }
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

#[test]
fn fri05_c03_integration_padding_seed_root_rounding_and_cache_preserve_gutter_area_in_both_scalar_lanes()
 {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar::<S>;
        let flow_axes = FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl);
        assert_eq!(flow_axes.inline_end(), PhysicalSide::Top);
        let size = Size::new(scalar(100.4), scalar(80.4));
        let style = NodeInputOf::<S> {
            display: Display::Block,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow: match flow_axes.inline_axis() {
                PhysicalAxis::Horizontal => computed_overflow(Overflow::Hidden, Overflow::Scroll),
                PhysicalAxis::Vertical => computed_overflow(Overflow::Scroll, Overflow::Hidden),
            },
            scrollbar_width: ScrollbarWidthOf::try_new(scalar(6.6)).unwrap(),
            size: Size::new(
                PreferredSizeOf::px(size.width),
                PreferredSizeOf::px(size.height),
            ),
            padding: Edges::all(LengthOf::px(scalar(0.4))),
            border: Edges::all(LengthOf::px(scalar(0.3))),
            ..NodeInputOf::default()
        };
        let tree = PublicFlowTree::default()
            .with_children(0, [])
            .with_style(0, style);
        let request = LayoutRootRequestOf::viewport(size.map(AvailableOf::definite))
            .expect("fractional viewport request is valid");
        let cold = compute_layout(&tree, 0, request).expect("cold guttered block root lays out");

        for (phase, output) in [
            ("unrounded", public_flow_output(cold.unrounded_entries(), 0)),
            ("rounded", public_flow_output(cold.final_entries(), 0)),
        ] {
            let geometry = output
                .scroll_geometry
                .expect("performed block root emits geometry");
            assert_ne!(geometry.padding_box(), geometry.scrollport(), "{phase}");
            assert_eq!(
                geometry.scrollable_overflow(),
                geometry.padding_box(),
                "root publication must retain the canonical own padding seed after {phase} publication"
            );
            let range = geometry.physical_range();
            assert_eq!(
                (range.x().minimum(), range.x().maximum()),
                (S::ZERO, S::ZERO)
            );
            assert_eq!(
                (range.y().minimum(), range.y().maximum()),
                (S::ZERO, S::ZERO)
            );
            assert_eq!(output.content_box_size(), geometry.content_box().size());
            assert_eq!(output.scrollbar_size(), geometry.scrollbar_size());
            assert_eq!(geometry.target().border_box(), geometry.border_box());
        }

        tree.apply_cache_entries(cold.cache_store_entries());
        tree.clear_cache_inputs();
        let warm = compute_layout(&tree, 0, request).expect("warm guttered block root lays out");
        assert_eq!(
            public_flow_output(warm.unrounded_entries(), 0),
            public_flow_output(cold.unrounded_entries(), 0)
        );
        assert_eq!(
            public_flow_output(warm.final_entries(), 0),
            public_flow_output(cold.final_entries(), 0)
        );
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 0),
            "the stable ordinary root request must reuse its cached geometry"
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri05_c04_flex_round_cache_root_preserves_source_geometry_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar::<S>;
        let flow_axes = FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl);
        let size = Size::new(scalar(100.4), scalar(80.4));
        let scroll_margin =
            ScrollMarginOf::try_new(scalar(-1.2), scalar(2.3), scalar(3.4), scalar(-4.5))
                .expect("finite flex target margin");
        let snap_align =
            ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::End);
        let style = NodeInputOf::<S> {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow: match flow_axes.inline_axis() {
                PhysicalAxis::Horizontal => computed_overflow(Overflow::Hidden, Overflow::Scroll),
                PhysicalAxis::Vertical => computed_overflow(Overflow::Scroll, Overflow::Hidden),
            },
            scrollbar_width: ScrollbarWidthOf::try_new(scalar(6.6)).unwrap(),
            size: Size::new(
                PreferredSizeOf::px(size.width),
                PreferredSizeOf::px(size.height),
            ),
            padding: Edges::all(LengthOf::px(scalar(0.4))),
            border: Edges::all(LengthOf::px(scalar(0.3))),
            scroll_margin,
            scroll_snap_align: snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInputOf::default()
        };
        let tree = PublicFlowTree::default()
            .with_children(0, [])
            .with_style(0, style.clone());
        let request = LayoutRootRequestOf::viewport(size.map(AvailableOf::definite))
            .expect("fractional flex viewport request is valid");
        let cold = compute_layout(&tree, 0, request).expect("cold guttered flex root lays out");

        let cached = cold
            .cache_store_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .expect("ordinary flex root output is cached")
            .output();
        let unrounded = public_flow_output(cold.unrounded_entries(), 0);
        assert_eq!(cached.scroll_geometry, unrounded.scroll_geometry);
        assert_eq!(cached.content_size, unrounded.content_size);

        for (phase, output) in [
            ("unrounded", unrounded),
            ("rounded", public_flow_output(cold.final_entries(), 0)),
        ] {
            let geometry = output
                .scroll_geometry
                .expect("performed flex root emits geometry");
            assert_eq!(geometry.flow_axes(), flow_axes, "{phase}");
            assert_eq!(geometry.used_overflow_x(), style.overflow.x(), "{phase}");
            assert_eq!(geometry.used_overflow_y(), style.overflow.y(), "{phase}");
            assert_ne!(geometry.padding_box(), geometry.scrollport(), "{phase}");
            assert_eq!(
                geometry.scrollable_overflow(),
                geometry.padding_box(),
                "canonical source retains the flex padding seed after {phase} publication"
            );
            let range = geometry.physical_range();
            assert_eq!(
                (range.x().minimum(), range.x().maximum()),
                (S::ZERO, S::ZERO),
                "{phase}"
            );
            assert!(range.y().minimum() <= range.y().maximum(), "{phase}");
            assert_eq!(output.content_box_size(), geometry.content_box().size());
            assert_eq!(output.scrollbar_size(), geometry.scrollbar_size());
            assert_eq!(geometry.target().border_box(), geometry.border_box());
            assert_eq!(geometry.target().scroll_margin(), scroll_margin);
            assert_eq!(geometry.target().flow_axes(), flow_axes);
            assert_eq!(geometry.target().snap_align(), snap_align);
            assert_eq!(geometry.target().snap_stop(), ScrollSnapStop::Always);
        }

        tree.apply_cache_entries(cold.cache_store_entries());
        tree.clear_cache_inputs();
        let warm = compute_layout(&tree, 0, request).expect("warm guttered flex root lays out");
        assert_eq!(
            public_flow_output(warm.unrounded_entries(), 0),
            public_flow_output(cold.unrounded_entries(), 0)
        );
        assert_eq!(
            public_flow_output(warm.final_entries(), 0),
            public_flow_output(cold.final_entries(), 0)
        );
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 0),
            "the stable ordinary flex root request reuses its cached canonical geometry"
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri05_c04_flex_round_cache_nested_flex_reuses_identical_canonical_output() {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar::<S>;
        let flow_axes = FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl);
        let root_size = Size::new(scalar(100.4), scalar(80.6));
        let child_size = Size::new(scalar(120.25), scalar(90.75));
        let overflow = computed_overflow(Overflow::Hidden, Overflow::Scroll);
        let root = NodeInputOf::<S> {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidthOf::try_new(scalar(3.6)).unwrap(),
            size: root_size.map(PreferredSizeOf::px),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInputOf::default()
        };
        let child = NodeInputOf::<S> {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_gutter: ScrollbarGutter::Stable,
            scrollbar_width: ScrollbarWidthOf::try_new(scalar(2.6)).unwrap(),
            size: child_size.map(PreferredSizeOf::px),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
            ..NodeInputOf::default()
        };
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, root)
            .with_style(1, child);
        let request = LayoutRootRequestOf::viewport(root_size.map(AvailableOf::definite)).unwrap();
        let cold = compute_layout(&tree, 0, request).expect("cold nested flex layout");
        let nested_unrounded = public_flow_output(cold.unrounded_entries(), 1);
        let nested_cached = cold
            .cache_store_entries()
            .iter()
            .find(|entry| entry.node() == 1 && entry.input().run_mode() == RunMode::PerformLayout)
            .expect("nested performed flex output is cached")
            .output();
        assert_eq!(
            nested_cached.scroll_geometry,
            nested_unrounded.scroll_geometry
        );
        assert_eq!(nested_cached.content_size, nested_unrounded.content_size);

        tree.apply_cache_entries(cold.cache_store_entries());
        tree.clear_cache_inputs();
        let warm = compute_layout(&tree, 0, request).expect("warm nested flex layout");
        for node in [0, 1] {
            assert_eq!(
                public_flow_output(warm.unrounded_entries(), node),
                public_flow_output(cold.unrounded_entries(), node),
                "unrounded node {node}"
            );
            assert_eq!(
                public_flow_output(warm.final_entries(), node),
                public_flow_output(cold.final_entries(), node),
                "rounded node {node}"
            );
        }
        assert!(
            warm.cache_store_entries().iter().all(|entry| {
                entry.node() != 1 || entry.input().run_mode() != RunMode::PerformLayout
            }),
            "the nested ordinary flex request must hit its warm cache entry"
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri05_c03_integration_padding_seed_fractional_terminal_auto_probe_survives_rounding_and_cache_in_both_scalar_lanes()
 {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar::<S>;
        let border_size = Size::new(scalar(10.1), scalar(10.0));
        let measured_content_size = Size::new(scalar(10.4), scalar(1.0));
        let terminal_padding = scalar(0.4);
        let tree = Fri05C03MeasuredLeafTree {
            style: NodeInputOf::<S> {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidthOf::try_new(S::ZERO).unwrap(),
                size: Size::new(
                    PreferredSizeOf::px(border_size.width),
                    PreferredSizeOf::px(border_size.height),
                ),
                padding: Edges {
                    right: LengthOf::px(terminal_padding),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
            measured: measured_content_size,
            measurement_inputs: RefCell::new(Vec::new()),
        };
        let request = LayoutRootRequestOf::viewport(border_size.map(AvailableOf::definite))
            .expect("fractional terminal viewport request is valid");
        let cold =
            compute_layout(&tree, 0, request).expect("fractional terminal measured root lays out");

        let unrounded = public_flow_output(cold.unrounded_entries(), 0);
        let unrounded_geometry = unrounded
            .scroll_geometry
            .expect("unrounded root publication retains geometry");
        let exact_terminal_end = measured_content_size.width + terminal_padding;
        assert_eq!(
            unrounded_geometry.scrollport().size().width,
            border_size.width
        );
        assert_eq!(
            unrounded_geometry.scrollable_overflow().size().width,
            exact_terminal_end,
            "the public complete overflow retains the exact non-seed terminal point"
        );

        let cached = cold
            .cache_store_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .expect("the ordinary root request stages a cache copy")
            .output()
            .scroll_geometry
            .expect("the cached root output retains geometry");
        assert_eq!(cached, unrounded_geometry);

        let rounded = public_flow_output(cold.final_entries(), 0);
        let rounded_geometry = rounded
            .scroll_geometry
            .expect("rounded root publication retains geometry");
        assert_eq!(rounded_geometry.scrollport().size().width, scalar(10.0));
        assert_eq!(
            rounded_geometry.scrollable_overflow().size().width,
            scalar(11.0)
        );
        assert_eq!(
            (
                rounded_geometry.physical_range().x().minimum(),
                rounded_geometry.physical_range().x().maximum(),
            ),
            (S::ZERO, scalar(1.0)),
            "canonical rounded geometry exposes the retained terminal overflow"
        );
        let observed =
            crate::scroll::SettledAutoScrollbarState::INITIAL.transition(rounded_geometry);
        assert!(
            observed.at(PhysicalAxis::Horizontal),
            "the conditional auto observation must retain the same exact terminal overflow"
        );
        assert!(!observed.at(PhysicalAxis::Vertical));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
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
