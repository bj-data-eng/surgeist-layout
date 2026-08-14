use super::*;

pub(super) fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

pub(super) fn public_layout_tree<S: LayoutScalar>(
    inputs: HashMap<u32, LayoutInputOf<S>>,
    children: HashMap<u32, Vec<u32>>,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new();
    for (node, input) in inputs {
        tree.insert_input(node, input);
    }
    for (node, children) in children {
        tree.insert_children(node, children);
    }
    tree
}

pub(super) fn fri06_atomic_participation<S: LayoutScalar>() -> AtomicInlineParticipationOf<S> {
    AtomicInlineParticipationOf::try_new(
        BidiLevel::try_new(0).unwrap(),
        InlineBreakOpportunityOf::prohibited(),
    )
    .unwrap()
}

pub(super) struct RootTestScrollGeometryFacts<S: LayoutScalar> {
    pub(super) flow_axes: FlowAxes,
    pub(super) overflow: ComputedOverflow,
    pub(super) item_is_replaced: bool,
    pub(super) border_box_size: Size<S>,
    pub(super) padding: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) scrollbar_width: S,
    pub(super) scrollable_overflow: ScrollRectOf<S>,
}

pub(super) fn root_test_scroll_geometry<S: LayoutScalar>(
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

pub(super) fn fri06_c02_segment<S: LayoutScalar>(
    id: u64,
    extent: f64,
    whitespace: InlineWhitespaceEdge,
    following_break: InlineBreakOpportunityOf<S>,
) -> ShapedInlineSegmentOf<S> {
    fri06_c02_segment_with_level(id, extent, 0, whitespace, following_break)
}

pub(super) fn fri06_c02_segment_with_level<S: LayoutScalar>(
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

pub(super) fn fri06_c02_segment_with_metrics<S: LayoutScalar>(
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

pub(super) fn fri06_c02_text_batch<S: LayoutScalar>(
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

pub(super) fn fri06_c02_text_batch_with_flow<S: LayoutScalar>(
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
    let tree = public_layout_tree(inputs, HashMap::from([(0, vec![1]), (1, Vec::new())]));

    let viewport = FlowAxes::new(writing_mode, direction).physical_size(LogicalSizeOf::new(
        available_inline,
        AvailableOf::MAX_CONTENT,
    ));
    compute_layout(&tree, 0, LayoutRootRequestOf::viewport(viewport).unwrap()).unwrap()
}

pub(super) fn fri06_c02_text_nodes_batch<S: LayoutScalar>(
    text_nodes: Vec<(u32, Vec<ShapedInlineSegmentOf<S>>)>,
    root_input: NodeInputOf<S>,
    available: Size<AvailableOf<S>>,
) -> CompletedLayoutBatchOf<u32, S> {
    let children = text_nodes.iter().map(|(node, _)| *node).collect::<Vec<_>>();
    let mut inputs = HashMap::from([(0, LayoutInputOf::box_input(root_input.clone()))]);
    let mut tree_children = HashMap::from([(0, children)]);
    for (node, segments) in text_nodes {
        inputs.insert(
            node,
            LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap()),
        );
        tree_children.insert(node, Vec::new());
    }
    let tree = public_layout_tree(inputs, tree_children);

    compute_layout(&tree, 0, LayoutRootRequestOf::viewport(available).unwrap()).unwrap()
}

pub(super) fn fri06_c02_final_node<S: LayoutScalar>(
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

pub(super) fn fri06_c03_atomic_participation<S: LayoutScalar>(
    level: u8,
    following_break: InlineBreakOpportunityOf<S>,
) -> AtomicInlineParticipationOf<S> {
    AtomicInlineParticipationOf::try_new(BidiLevel::try_new(level).unwrap(), following_break)
        .unwrap()
}

pub(super) fn fri06_c03_atomic_style<S: LayoutScalar>(
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

pub(super) fn fri06_c03_mixed_batch_with_root<S: LayoutScalar>(
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
    let mut tree_children = HashMap::from([(0, child_nodes)]);
    for (node, layout_input, _node_input) in children {
        inputs.insert(node, layout_input);
        tree_children.insert(node, Vec::new());
    }
    let tree = public_layout_tree(inputs, tree_children);

    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(available_inline, AvailableOf::MAX_CONTENT))
            .unwrap(),
    )
    .unwrap()
}

pub(super) fn fri06_c03_text_input<S: LayoutScalar>(
    segments: Vec<ShapedInlineSegmentOf<S>>,
) -> LayoutInputOf<S> {
    LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap())
}

pub(super) fn fri06_c04_line_box<S: LayoutScalar>(
    flow_axes: FlowAxes,
    logical_size: LogicalSizeOf<S>,
    float: Float,
    participation: Option<AtomicInlineParticipationOf<S>>,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: if float.is_none() {
            Display::InlineBlock
        } else {
            Display::Block
        },
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        float,
        size: flow_axes
            .physical_size(logical_size)
            .map(PreferredSizeOf::px),
        atomic_inline_participation: participation,
        ..NodeInputOf::default()
    }
}

pub(super) fn fri06_c04_line_batch<S: LayoutScalar>(
    flow_axes: FlowAxes,
    text_align: TextAlign,
    children: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
) -> CompletedLayoutBatchOf<u32, S> {
    let logical_root_size = LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(160.0));
    let root_size = flow_axes.physical_size(logical_root_size);
    let root_input = NodeInputOf {
        display: Display::Block,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        text_align,
        size: root_size.map(PreferredSizeOf::px),
        ..NodeInputOf::default()
    };
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
        LayoutRootRequestOf::viewport(root_size.map(AvailableOf::definite)).unwrap(),
    )
    .unwrap()
}

pub(super) fn fri06_c04_bfc_batch<S: LayoutScalar>(
    flow_axes: FlowAxes,
    root_children: Vec<u32>,
    nodes: Vec<(u32, NodeInputOf<S>, Vec<u32>)>,
) -> CompletedLayoutBatchOf<u32, S> {
    let logical_root_size = LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(160.0));
    let root_size = flow_axes.physical_size(logical_root_size);
    let root_style = NodeInputOf {
        display: Display::Block,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        size: root_size.map(PreferredSizeOf::px),
        ..NodeInputOf::default()
    };
    let mut inputs = HashMap::from([(0, LayoutInputOf::box_input(root_style.clone()))]);
    let mut children = HashMap::from([(0, root_children)]);
    for (node, style, node_children) in nodes {
        inputs.insert(node, LayoutInputOf::box_input(style.clone()));
        children.insert(node, node_children);
    }
    let tree = public_layout_tree(inputs, children);

    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(root_size.map(AvailableOf::definite)).unwrap(),
    )
    .unwrap()
}

pub(super) type Fri06C04FrontDoorNode<S> = (u32, LayoutInputOf<S>, NodeInputOf<S>, Vec<u32>);

pub(super) fn fri06_c04_front_door_batch<S: LayoutScalar>(
    root_style: NodeInputOf<S>,
    logical_available: LogicalSizeOf<AvailableOf<S>>,
    root_children: Vec<u32>,
    nodes: Vec<Fri06C04FrontDoorNode<S>>,
) -> CompletedLayoutBatchOf<u32, S> {
    let flow_axes = FlowAxes::new(root_style.writing_mode, root_style.direction);
    let mut inputs = HashMap::from([(0, LayoutInputOf::box_input(root_style.clone()))]);
    let mut children = HashMap::from([(0, root_children)]);
    for (node, layout_input, _node_input, node_children) in nodes {
        inputs.insert(node, layout_input);
        children.insert(node, node_children);
    }
    let tree = public_layout_tree(inputs, children);

    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(flow_axes.physical_size(logical_available)).unwrap(),
    )
    .unwrap()
}

pub(super) fn fri06_c04_logical_origin<S: LayoutScalar>(
    flow_axes: FlowAxes,
    output: NodeOutputOf<S>,
) -> LogicalPointOf<S> {
    flow_axes.logical_point(
        output.location,
        output.size,
        flow_axes.physical_size(LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(160.0))),
    )
}

pub(super) fn assert_fri06_c08_r1_mixed_unit_traversal<S: LayoutScalar>(
    flow_axes: FlowAxes,
    box_sizing: BoxSizing,
) {
    let logical_root_size = LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(160.0));
    let root_size = flow_axes.physical_size(logical_root_size);
    let root_input = NodeInputOf {
        display: Display::Block,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        box_sizing,
        size: root_size.map(PreferredSizeOf::px),
        ..NodeInputOf::default()
    };
    let boundary_metrics =
        InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0)).unwrap();
    let boundary = |kind| {
        InlineBoundaryInputOf::new(kind, boundary_metrics)
            .with_writing_mode(flow_axes.writing_mode())
            .with_direction(flow_axes.direction())
    };
    let atomic = fri06_c04_line_box(
        flow_axes,
        LogicalSizeOf::new(S::from_f64(5.0), S::from_f64(12.0)),
        Float::None,
        Some(fri06_c03_atomic_participation(
            2,
            InlineBreakOpportunityOf::prohibited(),
        )),
    );
    let children = vec![
        (
            1,
            fri06_c03_text_input(vec![fri06_c02_segment_with_level(
                901,
                10.0,
                1,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            )]),
            NodeInputOf::non_box(),
        ),
        (
            2,
            LayoutInputOf::inline_boundary(boundary(InlineBoundaryKind::Start)),
            NodeInputOf::non_box(),
        ),
        (3, LayoutInputOf::box_input(atomic.clone()), atomic),
        (
            4,
            LayoutInputOf::inline_boundary(boundary(InlineBoundaryKind::End)),
            NodeInputOf::non_box(),
        ),
        (
            5,
            fri06_c03_text_input(vec![fri06_c02_segment_with_level(
                902,
                7.0,
                1,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            )]),
            NodeInputOf::non_box(),
        ),
    ];
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
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(root_size.map(AvailableOf::definite)).unwrap(),
    )
    .unwrap();

    let source_starts = if flow_axes.direction() == Direction::Rtl {
        [12.0, 12.0, 7.0, 7.0, 0.0]
    } else {
        [0.0, 10.0, 10.0, 15.0, 15.0]
    };
    let logical_rects = [
        (source_starts[0], 4.0, 10.0, 10.0),
        (source_starts[1], 12.0, 0.0, 0.0),
        (source_starts[2], 0.0, 5.0, 12.0),
        (source_starts[3], 12.0, 0.0, 0.0),
        (source_starts[4], 4.0, 7.0, 10.0),
    ];
    for (index, logical_rect) in logical_rects.into_iter().enumerate() {
        let node = u32::try_from(index + 1).unwrap();
        let output = fri06_c02_final_node(&batch, node);
        let (expected_origin, expected_size) = fri06_c02_expected_physical_rect(
            (flow_axes.writing_mode(), flow_axes.direction()),
            logical_rect,
            (100.0, 160.0),
        );
        assert_eq!(
            (output.location, output.size, output.source_index.get()),
            (expected_origin, expected_size, index),
            "{flow_axes:?} {box_sizing:?} source unit {index}"
        );
    }

    assert_eq!(
        batch
            .final_inline_fragments()
            .iter()
            .map(|entry| (
                entry.node(),
                entry.fragment().segment_id(),
                entry.fragment().visual_index(),
            ))
            .collect::<Vec<_>>(),
        if flow_axes.direction() == Direction::Rtl {
            vec![
                (1, InlineSegmentId::new(901), 4),
                (5, InlineSegmentId::new(902), 0),
            ]
        } else {
            vec![
                (1, InlineSegmentId::new(901), 0),
                (5, InlineSegmentId::new(902), 4),
            ]
        },
        "{flow_axes:?} {box_sizing:?} stable fragment identities"
    );
}

pub(super) fn assert_fri06_c08_mixed_inline_atomic_x<S: LayoutScalar>(
    direction: Direction,
    box_sizing: BoxSizing,
    expected_x: f64,
) {
    let mut leading_atomic = fri06_c03_atomic_style(
        18.0,
        18.0,
        0.0,
        0.0,
        1,
        InlineBreakOpportunityOf::prohibited(),
    );
    leading_atomic.box_sizing = box_sizing;
    let mut atomic = fri06_c03_atomic_style(
        12.0,
        18.0,
        0.0,
        0.0,
        1,
        InlineBreakOpportunityOf::prohibited(),
    );
    atomic.box_sizing = box_sizing;
    atomic.margin = match direction {
        Direction::Ltr => Edges {
            left: LengthAutoOf::px(S::from_f64(6.0)),
            ..Edges::all(LengthAutoOf::ZERO)
        },
        Direction::Rtl => Edges {
            right: LengthAutoOf::px(S::from_f64(6.0)),
            ..Edges::all(LengthAutoOf::ZERO)
        },
    };
    let root = NodeInputOf {
        display: Display::Block,
        writing_mode: WritingMode::HorizontalTb,
        direction,
        box_sizing,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(210.0)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    };
    let batch = fri06_c03_mixed_batch_with_root(
        vec![
            (
                1,
                LayoutInputOf::box_input(leading_atomic.clone()),
                leading_atomic,
            ),
            (
                2,
                fri06_c03_text_input(vec![fri06_c02_segment_with_level(
                    801,
                    75.0,
                    0,
                    InlineWhitespaceEdge::DiscardAtBoth,
                    InlineBreakOpportunityOf::prohibited(),
                )]),
                NodeInputOf::non_box(),
            ),
            (3, LayoutInputOf::box_input(atomic.clone()), atomic),
        ],
        AvailableOf::definite(S::from_f64(210.0)),
        root,
    );

    assert_eq!(
        fri06_c02_final_node(&batch, 3).location.x,
        S::from_f64(expected_x),
        "{direction:?} {box_sizing:?} mixed-line atomic placement"
    );
}

pub(super) fn assert_fri06_c08_float_line_final_height<S: LayoutScalar>(
    direction: Direction,
    box_sizing: BoxSizing,
) {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
    let float = |side, inline, block| {
        let mut style = fri06_c04_line_box(
            flow_axes,
            LogicalSizeOf::new(S::from_f64(inline), S::from_f64(block)),
            side,
            None,
        );
        style.box_sizing = box_sizing;
        style
    };
    let atomic = |inline, following_break| {
        let mut style = fri06_c04_line_box(
            flow_axes,
            LogicalSizeOf::new(S::from_f64(inline), S::from_f64(16.0)),
            Float::None,
            Some(fri06_c03_atomic_participation(
                u8::from(direction == Direction::Rtl),
                following_break,
            )),
        );
        style.box_sizing = box_sizing;
        style
    };
    let (physical_left_side, physical_right_side) = match direction {
        Direction::Ltr => (Float::Left, Float::Right),
        Direction::Rtl => (Float::Right, Float::Left),
    };
    let left_float = float(physical_left_side, 42.0, 42.0);
    let right_float = float(physical_right_side, 50.0, 62.0);
    let first_atomic = atomic(28.0, InlineBreakOpportunityOf::allowed());
    let second_atomic = atomic(32.0, InlineBreakOpportunityOf::allowed());
    let third_atomic = atomic(36.0, InlineBreakOpportunityOf::allowed());
    let fourth_atomic = atomic(40.0, InlineBreakOpportunityOf::prohibited());
    let line_strut = InlineBoundaryInputOf::new(
        InlineBoundaryKind::Start,
        InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(12.0))
            .unwrap(),
    )
    .with_writing_mode(WritingMode::HorizontalTb)
    .with_direction(direction);
    let root = NodeInputOf {
        display: Display::Block,
        writing_mode: WritingMode::HorizontalTb,
        direction,
        box_sizing,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(180.0)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    };
    let batch = fri06_c03_mixed_batch_with_root(
        vec![
            (1, LayoutInputOf::box_input(left_float.clone()), left_float),
            (
                2,
                LayoutInputOf::box_input(right_float.clone()),
                right_float,
            ),
            (
                3,
                fri06_c03_text_input(vec![fri06_c02_segment_with_metrics(4, 40.0, 14.8, 5.2)]),
                NodeInputOf::non_box(),
            ),
            (
                8,
                LayoutInputOf::inline_boundary(line_strut),
                NodeInputOf::non_box(),
            ),
            (
                4,
                LayoutInputOf::box_input(first_atomic.clone()),
                first_atomic,
            ),
            (
                5,
                LayoutInputOf::box_input(second_atomic.clone()),
                second_atomic,
            ),
            (
                6,
                LayoutInputOf::box_input(third_atomic.clone()),
                third_atomic,
            ),
            (
                7,
                LayoutInputOf::box_input(fourth_atomic.clone()),
                fourth_atomic,
            ),
        ],
        AvailableOf::definite(S::from_f64(180.0)),
        root,
    );

    let (expected_left_float_x, expected_right_float_x) = (0.0, 130.0);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(
            public_flow_output(entries, 1).location,
            Point::new(S::from_f64(expected_left_float_x), S::ZERO),
            "{direction:?} {box_sizing:?} line-left float"
        );
        assert_eq!(
            public_flow_output(entries, 2).location,
            Point::new(S::from_f64(expected_right_float_x), S::ZERO),
            "{direction:?} {box_sizing:?} line-right float"
        );
    }
    assert_eq!(
        batch
            .final_inline_fragments()
            .iter()
            .map(|entry| entry.fragment().line_index())
            .collect::<Vec<_>>(),
        [0],
        "{direction:?} {box_sizing:?} shaped-text line"
    );
    assert_eq!(
        [4, 5, 6, 7].map(|node| public_flow_output(batch.final_entries(), node).size),
        [
            Size::new(S::from_f64(28.0), S::from_f64(16.0)),
            Size::new(S::from_f64(32.0), S::from_f64(16.0)),
            Size::new(S::from_f64(36.0), S::from_f64(16.0)),
            Size::new(S::from_f64(40.0), S::from_f64(16.0)),
        ],
        "{direction:?} {box_sizing:?} atomic sizes"
    );
    for (node, expected) in [4, 5, 6, 7].into_iter().zip([0.0, 21.2, 21.2, 42.0]) {
        let actual = public_flow_output(batch.unrounded_entries(), node)
            .location
            .y;
        assert!(
            (actual - S::from_f64(expected)).abs() <= S::from_f64(0.000_1),
            "{direction:?} {box_sizing:?} unrounded float-band line placement: \
             node {node} expected {expected}, got {actual:?}"
        );
    }
    let expected_second_line_atomic_x = match direction {
        Direction::Ltr => [42.0, 74.0],
        Direction::Rtl => [62.0, 94.0],
    };
    assert_eq!(
        [5, 6].map(|node| {
            public_flow_output(batch.unrounded_entries(), node)
                .location
                .x
        }),
        expected_second_line_atomic_x.map(S::from_f64),
        "bidi visual ordering assigns logical float-band slots once before physical projection",
    );
    let expected_terminal_x = match direction {
        Direction::Ltr => 0.0,
        Direction::Rtl => 90.0,
    };
    assert_eq!(
        public_flow_output(batch.unrounded_entries(), 7).location.x,
        S::from_f64(expected_terminal_x),
        "{direction:?} {box_sizing:?} terminal atomic physical placement"
    );
    assert_eq!(
        public_flow_output(batch.unrounded_entries(), 0).size,
        Size::new(S::from_f64(180.0), S::from_f64(62.5)),
        "{direction:?} {box_sizing:?} unrounded block geometry"
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 0).size,
        Size::new(S::from_f64(180.0), S::from_f64(63.0)),
        "{direction:?} {box_sizing:?} final block geometry"
    );
}

pub(super) fn fri06_c12_t08_forced_break_fallback_batch<S: LayoutScalar>(
    line_break_baseline: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let atomic = |node| {
        let style = fri06_c03_atomic_style(
            24.0,
            16.0,
            0.0,
            0.0,
            0,
            InlineBreakOpportunityOf::prohibited(),
        );
        (
            node,
            LayoutInputOf::box_input(style.clone()),
            style,
            Vec::new(),
        )
    };
    let line_break = LineBreakInputOf::new().with_metrics(
        InlineMetricsOf::from_line_height_and_baseline(
            S::from_f64(20.0),
            S::from_f64(line_break_baseline),
        )
        .unwrap(),
    );
    let parent = NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(160.0)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    };
    let mut nodes = Vec::new();
    for (parent_node, first_atomic, line_break_node, second_atomic) in
        [(1, 2, 3, 4), (5, 6, 7, 8), (9, 10, 11, 12)]
    {
        nodes.push((
            parent_node,
            LayoutInputOf::box_input(parent.clone()),
            parent.clone(),
            vec![first_atomic, line_break_node, second_atomic],
        ));
        nodes.push(atomic(first_atomic));
        nodes.push((
            line_break_node,
            LayoutInputOf::line_break(line_break),
            NodeInputOf::non_box(),
            Vec::new(),
        ));
        nodes.push(atomic(second_atomic));
    }
    fri06_c04_front_door_batch(
        NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(160.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        },
        LogicalSizeOf::new(
            AvailableOf::definite(S::from_f64(160.0)),
            AvailableOf::MAX_CONTENT,
        ),
        vec![1, 5, 9],
        nodes,
    )
}

pub(super) fn fri06_c02_expected_physical_rect<S: LayoutScalar>(
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

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Fri06C02RetainedTextState<S: LayoutScalar> {
    pub(super) unrounded_nodes: HashMap<u32, NodeOutputOf<S>>,
    pub(super) final_nodes: HashMap<u32, NodeOutputOf<S>>,
    pub(super) unrounded_fragments: HashMap<u32, Vec<InlineFragmentOutputOf<S>>>,
    pub(super) final_fragments: HashMap<u32, Vec<InlineFragmentOutputOf<S>>>,
    pub(super) caches: HashMap<u32, CacheOf<S>>,
    pub(super) dirty: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) enum Fri06C05ShapeProvider<S: LayoutScalar> {
    #[default]
    Disabled,
    Empty,
    Interval {
        minimum: S,
        maximum: S,
    },
    Failure,
}

#[derive(Clone, Debug)]
pub(super) struct Fri06C02StatefulTextTree<S: LayoutScalar> {
    pub(super) inputs: HashMap<u32, LayoutInputOf<S>>,
    pub(super) node_inputs: HashMap<u32, NodeInputOf<S>>,
    pub(super) children: HashMap<u32, Vec<u32>>,
    pub(super) retained: Fri06C02RetainedTextState<S>,
    pub(super) fragment_readbacks: Cell<usize>,
    pub(super) reject_preparation: bool,
    pub(super) shape_provider: Fri06C05ShapeProvider<S>,
    pub(super) shape_queries: Cell<usize>,
    pub(super) cache_queries: RefCell<Vec<(u32, bool)>>,
}

impl<S: LayoutScalar> Fri06C02StatefulTextTree<S> {
    pub(super) fn new(segments: Vec<ShapedInlineSegmentOf<S>>) -> Self {
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
            shape_provider: Fri06C05ShapeProvider::Disabled,
            shape_queries: Cell::new(0),
            cache_queries: RefCell::new(Vec::new()),
        }
    }

    pub(super) fn new_mixed() -> Self {
        let root_input = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(20.0)),
                PreferredSizeOf::px(S::from_f64(20.0)),
            ),
            overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
            ..NodeInputOf::default()
        };
        let text = InlineTextInputOf::try_new(vec![fri06_c02_segment(
            91,
            9.25,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::allowed(),
        )])
        .unwrap();
        let atomic = fri06_c03_atomic_style(
            25.25,
            10.5,
            0.0,
            0.0,
            0,
            InlineBreakOpportunityOf::prohibited(),
        );
        let boundary = InlineBoundaryInputOf::new(
            InlineBoundaryKind::Start,
            InlineMetricsOf::from_ascent_descent(S::from_f64(6.25), S::from_f64(3.75)).unwrap(),
        );
        let line_break = LineBreakInputOf::new().with_metrics(
            InlineMetricsOf::from_ascent_descent(S::from_f64(8.25), S::from_f64(1.75)).unwrap(),
        );
        Self {
            inputs: HashMap::from([
                (0, LayoutInputOf::box_input(root_input.clone())),
                (1, LayoutInputOf::inline_text(text)),
                (2, LayoutInputOf::box_input(atomic.clone())),
                (3, LayoutInputOf::inline_boundary(boundary)),
                (4, LayoutInputOf::line_break(line_break)),
            ]),
            node_inputs: HashMap::from([
                (0, root_input),
                (1, NodeInputOf::non_box()),
                (2, atomic),
                (3, NodeInputOf::non_box()),
                (4, NodeInputOf::non_box()),
            ]),
            children: HashMap::from([
                (0, vec![1, 2, 3, 4]),
                (1, Vec::new()),
                (2, Vec::new()),
                (3, Vec::new()),
                (4, Vec::new()),
            ]),
            retained: Fri06C02RetainedTextState::default(),
            fragment_readbacks: Cell::new(0),
            reject_preparation: false,
            shape_provider: Fri06C05ShapeProvider::Disabled,
            shape_queries: Cell::new(0),
            cache_queries: RefCell::new(Vec::new()),
        }
    }

    pub(super) fn new_float_mixed() -> Self {
        let root_input = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(40.0)),
                PreferredSizeOf::AUTO,
            ),
            overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
            ..NodeInputOf::default()
        };
        let float = NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(15.25)),
                PreferredSizeOf::px(S::from_f64(12.5)),
            ),
            ..NodeInputOf::default()
        };
        let text = InlineTextInputOf::try_new(vec![fri06_c02_segment(
            93,
            20.25,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )])
        .unwrap();
        Self {
            inputs: HashMap::from([
                (0, LayoutInputOf::box_input(root_input.clone())),
                (1, LayoutInputOf::box_input(float.clone())),
                (2, LayoutInputOf::inline_text(text)),
            ]),
            node_inputs: HashMap::from([(0, root_input), (1, float), (2, NodeInputOf::non_box())]),
            children: HashMap::from([(0, vec![1, 2]), (1, Vec::new()), (2, Vec::new())]),
            retained: Fri06C02RetainedTextState::default(),
            fragment_readbacks: Cell::new(0),
            reject_preparation: false,
            shape_provider: Fri06C05ShapeProvider::Disabled,
            shape_queries: Cell::new(0),
            cache_queries: RefCell::new(Vec::new()),
        }
    }

    pub(super) fn new_shape_provider(provider: Fri06C05ShapeProvider<S>) -> Self {
        let mut tree = Self::new_float_mixed();
        let mut float = tree.node_inputs[&1].clone();
        float.float_exclusion = FloatExclusion::Shape;
        tree.inputs
            .insert(1, LayoutInputOf::box_input(float.clone()));
        tree.node_inputs.insert(1, float);
        tree.shape_provider = provider;
        tree
    }

    pub(super) fn replace_float_inline_extent(&mut self, extent: f64) {
        let style = NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(extent)),
                PreferredSizeOf::px(S::from_f64(12.5)),
            ),
            ..NodeInputOf::default()
        };
        self.inputs
            .insert(1, LayoutInputOf::box_input(style.clone()));
        self.node_inputs.insert(1, style);
    }

    pub(super) fn add_failing_float_path_control(&mut self) {
        self.inputs
            .insert(9, LayoutInputOf::line_break(LineBreakInputOf::new()));
        self.node_inputs.insert(9, NodeInputOf::default());
        self.children.get_mut(&0).unwrap().push(9);
        self.children.insert(9, Vec::new());
    }

    pub(super) fn remove_failing_float_path_control(&mut self) {
        self.inputs.remove(&9);
        self.node_inputs.remove(&9);
        self.children.get_mut(&0).unwrap().retain(|node| *node != 9);
        self.children.remove(&9);
    }

    pub(super) fn replace_atomic_inline_extent(&mut self, extent: f64) {
        let style = fri06_c03_atomic_style(
            extent,
            10.5,
            0.0,
            0.0,
            0,
            InlineBreakOpportunityOf::prohibited(),
        );
        self.inputs
            .insert(2, LayoutInputOf::box_input(style.clone()));
        self.node_inputs.insert(2, style);
    }

    pub(super) fn replace_text(&mut self, segments: Vec<ShapedInlineSegmentOf<S>>) {
        self.inputs.insert(
            1,
            LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap()),
        );
    }

    pub(super) fn add_failing_noncanonical_control(&mut self) {
        self.inputs
            .insert(2, LayoutInputOf::line_break(LineBreakInputOf::new()));
        self.node_inputs.insert(2, NodeInputOf::default());
        self.children.insert(0, vec![1, 2]);
        self.children.insert(2, Vec::new());
    }

    pub(super) fn remove_failing_noncanonical_control(&mut self) {
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
        self.retained.unrounded_nodes.get(&node).copied()
    }

    fn unrounded_inline_fragments(&self, node: Self::Node) -> Option<&[InlineFragmentOutputOf<S>]> {
        self.fragment_readbacks
            .set(self.fragment_readbacks.get() + 1);
        self.retained
            .unrounded_fragments
            .get(&node)
            .map(Vec::as_slice)
    }

    fn float_exclusion_interval(
        &self,
        _node: Self::Node,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        self.shape_queries.set(self.shape_queries.get() + 1);
        match self.shape_provider {
            Fri06C05ShapeProvider::Disabled => None,
            Fri06C05ShapeProvider::Empty => Some(Ok(None)),
            Fri06C05ShapeProvider::Interval { minimum, maximum } => Some(Ok(
                FloatExclusionIntervalOf::try_new(query, minimum, maximum).unwrap(),
            )),
            Fri06C05ShapeProvider::Failure => Some(Err(())),
        }
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

pub(super) fn fri06_c02_stateful_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(S::from_f64(20.0)),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}

pub(super) fn assert_positive_physical_range<S: LayoutScalar>(
    range: PhysicalScrollRangeOf<S>,
    maximum: Size<S>,
) {
    assert_eq!(range.x().minimum(), S::ZERO);
    assert_eq!(range.x().maximum(), maximum.width);
    assert_eq!(range.y().minimum(), S::ZERO);
    assert_eq!(range.y().maximum(), maximum.height);
}

#[derive(Clone, Debug, Default)]
pub(super) struct RootSessionTree<M = &'static str> {
    pub(super) children: HashMap<u32, Vec<u32>>,
    pub(super) inputs: HashMap<u32, LayoutInput>,
    pub(super) measurements: HashMap<u32, Result<Size, M>>,
    pub(super) leaf_nodes: HashSet<u32>,
    pub(super) measured_nodes: RefCell<Vec<u32>>,
    pub(super) caches: RefCell<HashMap<u32, Cache>>,
}

impl<M> RootSessionTree<M> {
    pub(super) fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    pub(super) fn style(mut self, node: u32, style: NodeInput) -> Self {
        self.inputs.insert(node, LayoutInput::box_input(style));
        self
    }

    pub(super) fn measure(mut self, node: u32, output: Result<Size, M>) -> Self {
        self.leaf_nodes.insert(node);
        self.measurements.insert(node, output);
        self
    }

    pub(super) fn leaf_without_provider(mut self, node: u32) -> Self {
        self.leaf_nodes.insert(node);
        self
    }

    pub(super) fn measured_nodes(&self) -> Vec<u32> {
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
pub(super) struct PublicFlowTree<S: LayoutScalar> {
    pub(super) children: HashMap<u32, Vec<u32>>,
    pub(super) styles: HashMap<u32, NodeInputOf<S>>,
    pub(super) caches: RefCell<HashMap<u32, CacheOf<S>>>,
    pub(super) cache_inputs: RefCell<Vec<(u32, ComputeInputOf<S>)>>,
}

impl<S: LayoutScalar> PublicFlowTree<S> {
    pub(super) fn with_children(
        mut self,
        node: u32,
        children: impl IntoIterator<Item = u32>,
    ) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    pub(super) fn with_style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.styles.insert(node, style);
        self
    }

    pub(super) fn apply_cache_entries(&self, entries: &[LayoutCacheStoreEntryOf<u32, S>]) {
        let mut caches = self.caches.borrow_mut();
        for entry in entries {
            caches.entry(entry.node()).or_default().store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
    }

    pub(super) fn cache_inputs(&self, node: u32) -> Vec<ComputeInputOf<S>> {
        self.cache_inputs
            .borrow()
            .iter()
            .filter_map(|(recorded_node, input)| (*recorded_node == node).then_some(*input))
            .collect()
    }

    pub(super) fn clear_cache_inputs(&self) {
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

pub(super) struct FlowRootLeafTree<S: LayoutScalar> {
    pub(super) style: NodeInputOf<S>,
    pub(super) natural_size: Size<S>,
    pub(super) measurement: RefCell<Option<LeafMeasureInputOf<S>>>,
}

impl<S: LayoutScalar> FlowRootLeafTree<S> {
    pub(super) fn new(style: NodeInputOf<S>) -> Self {
        Self {
            style,
            natural_size: Size::ZERO,
            measurement: RefCell::new(None),
        }
    }

    pub(super) fn with_natural_size(mut self, natural_size: Size<S>) -> Self {
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

pub(super) fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

pub(super) fn single_final_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .first()
        .expect("single root must produce one final output")
        .output()
}

pub(super) fn public_flow_output<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .expect("public layout batch contains the requested node")
        .output()
}

pub(super) fn logical_flex_leaf<S: LayoutScalar>(width: f64, height: f64) -> NodeInputOf<S> {
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

pub(super) fn root_writing_mode_directions() -> [(WritingMode, Direction); 10] {
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

pub(super) fn assert_public_scroll_geometry_error_without_batch(
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

pub(super) fn fri06_mr02_geometry_error_largest_finite<S: LayoutScalar>() -> S {
    if core::mem::size_of::<S>() == core::mem::size_of::<f32>() {
        S::from_f64(f32::MAX.into())
    } else {
        S::from_f64(f64::MAX)
    }
}

pub(super) struct Fri05C03MeasuredLeafTree<S: LayoutScalar = f32> {
    pub(super) style: NodeInputOf<S>,
    pub(super) measured: Size<S>,
    pub(super) measurement_inputs: RefCell<Vec<LeafMeasureInputOf<S>>>,
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

pub(super) fn fri05_c03_root_measurement_size(input: LeafMeasureInput) -> Size<f32> {
    assert_eq!(input.known_content_size(), Size::NONE);
    input.available_content_size().map(|available| {
        available
            .definite_value()
            .expect("FRI-05 root measurement availability is definite")
            .get()
    })
}

pub(super) fn fri05_c03_tree_leaf_layout(
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

pub(super) fn fri05_c03_root_gutter_at(
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

pub(super) fn fri05_c03_root_all_flow_axes() -> [FlowAxes; 10] {
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

pub(super) fn fri05_c03_block_root_state(input: &ComputeInput) -> (bool, bool) {
    let state = input.containing_auto_scrollbar_pass();
    (
        state.at(PhysicalAxis::Horizontal),
        state.at(PhysicalAxis::Vertical),
    )
}

pub(super) fn fri05_c04_local_auto_state(input: &ComputeInput) -> (bool, bool) {
    let state = input.settled_auto_scrollbars();
    (
        state.at(PhysicalAxis::Horizontal),
        state.at(PhysicalAxis::Vertical),
    )
}

pub(super) fn fri05_c04_nested_flex_auto_tree(inner_overflows: bool) -> PublicFlowTree<f32> {
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

pub(super) fn fri05_c04_assert_initial_local_auto_state(inputs: &[ComputeInput]) {
    assert!(
        inputs
            .iter()
            .all(|input| fri05_c04_local_auto_state(input) == (false, false)),
        "every recursively dispatched node starts local auto settlement at INITIAL: {inputs:#?}"
    );
}
