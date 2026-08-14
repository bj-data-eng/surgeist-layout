use super::fixtures::{
    Fri05C03MeasuredLeafTree, PublicFlowTree, RootSessionTree, RootTestScrollGeometryFacts,
    assert_fri06_c08_float_line_final_height, assert_positive_physical_range,
    assert_public_scroll_geometry_error_without_batch, computed_overflow,
    fri05_c03_root_all_flow_axes, fri05_c03_root_gutter_at, fri06_c02_final_node,
    fri06_c02_segment, fri06_c02_text_batch, fri06_c04_front_door_batch,
    fri06_c12_t08_forced_break_fallback_batch, logical_flex_leaf, public_flow_output,
    root_test_scroll_geometry, root_writing_mode_directions, scalar,
};
use super::*;

#[test]
fn fri06_c04_float_scroll_rounding_preserves_mapped_side_band_and_position_identity_all_flows_both_scalars()
 {
    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in root_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            for side in [Float::Left, Float::Right] {
                let root_style = NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: flow_axes.physical_size(LogicalSizeOf::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::px(S::from_f64(40.0)),
                    )),
                    ..NodeInputOf::default()
                };
                let float = NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    float: side,
                    size: flow_axes
                        .physical_size(LogicalSizeOf::new(S::from_f64(20.25), S::from_f64(10.25)))
                        .map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                };
                let text = InlineTextInputOf::try_new(vec![fri06_c02_segment(
                    951,
                    10.25,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                )])
                .unwrap();
                let batch = fri06_c04_front_door_batch(
                    root_style,
                    LogicalSizeOf::new(
                        AvailableOf::definite(S::from_f64(100.0)),
                        AvailableOf::definite(S::from_f64(40.0)),
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
                let container_size = flow_axes
                    .physical_size(LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(40.0)));
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    let floated = public_flow_output(entries, 1);
                    let line = public_flow_output(entries, 2);
                    assert_eq!(floated.source_index, SourceIndex::new(0));
                    assert_eq!(line.source_index, SourceIndex::new(1));
                    let float_origin =
                        flow_axes.logical_point(floated.location, floated.size, container_size);
                    let line_origin =
                        flow_axes.logical_point(line.location, line.size, container_size);
                    let float_inline = flow_axes.logical_size(floated.size).inline;
                    match side {
                        Float::Left => {
                            assert!(line_origin.inline >= float_origin.inline + float_inline)
                        }
                        Float::Right => assert!(line_origin.inline <= float_origin.inline),
                        Float::None => unreachable!(),
                    }
                    assert_eq!(line_origin.block, S::ZERO);
                }
                let unrounded_fragment = batch.unrounded_inline_fragments()[0].fragment();
                let rounded_fragment = batch.final_inline_fragments()[0].fragment();
                assert_eq!(
                    (
                        unrounded_fragment.segment_id(),
                        unrounded_fragment.line_index(),
                        unrounded_fragment.visual_index(),
                    ),
                    (
                        rounded_fragment.segment_id(),
                        rounded_fragment.line_index(),
                        rounded_fragment.visual_index(),
                    ),
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_border_box_ltr_rounds_final_height_to_63() {
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Ltr, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Ltr, BoxSizing::BorderBox);
}

#[test]
fn fri06_c08_float_line_border_box_rtl_rounds_final_height_to_63() {
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Rtl, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Rtl, BoxSizing::BorderBox);
}

#[test]
fn fri06_c08_float_line_content_box_ltr_rounds_final_height_to_63() {
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Ltr, BoxSizing::ContentBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Ltr, BoxSizing::ContentBox);
}

#[test]
fn fri06_c08_float_line_content_box_rtl_rounds_final_height_to_63() {
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Rtl, BoxSizing::ContentBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Rtl, BoxSizing::ContentBox);
}

#[test]
fn fri06_c12_t08_fractional_forced_break_fallback_preserves_unrounded_envelope() {
    fn assert_close<S: LayoutScalar>(actual: S, expected: f64, label: &str) {
        let expected = S::from_f64(expected);
        let tolerance = S::EPSILON * expected.abs().max(S::ONE) * S::from_f64(8.0);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{label}: expected {expected:?} within {tolerance:?}, got {actual:?}",
        );
    }

    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c12_t08_forced_break_fallback_batch::<S>(14.8);

        for (index, (parent_node, first_atomic, second_atomic)) in
            [(1, 2, 4), (5, 6, 8), (9, 10, 12)].into_iter().enumerate()
        {
            let parent = public_flow_output(batch.unrounded_entries(), parent_node);
            assert_close(
                parent.size.height,
                42.4,
                "two unrounded 21.2px line envelopes publish once per parent",
            );
            assert_close(
                parent.location.y,
                42.4 * index as f64,
                "parent block progression consumes the unrounded envelope once",
            );
            assert_close(
                public_flow_output(batch.unrounded_entries(), first_atomic)
                    .location
                    .y,
                0.0,
                "first atomic starts at the first unrounded line envelope",
            );
            assert_close(
                public_flow_output(batch.unrounded_entries(), second_atomic)
                    .location
                    .y,
                21.2,
                "second atomic starts after one unrounded line envelope",
            );
        }
        assert_close(
            public_flow_output(batch.unrounded_entries(), 0).size.height,
            127.2,
            "root publishes three unrounded parent envelopes exactly once",
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 0).size.height,
            S::from_f64(127.0),
            "ordinary final-layout rounding remains the only rounding phase",
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
    assert_eq!(unrounded_nodes, final_nodes);
    assert_eq!(final_nodes, vec![10, 20, 30, 40]);
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
fn round_layout_uses_cumulative_viewport_edges() {
    let mut tree = OracleTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .unrounded(
            1,
            NodeOutput {
                location: Point::new(0.2, 0.0),
                size: Size::new(10.4, 10.0),
                content_size: Size::new(10.4, 10.0),
                border: Edges::all(0.4),
                padding: Edges::all(0.6),
                ..NodeOutput::new()
            },
        )
        .unrounded(
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

    let root = tree.final_layout(1).expect("root final layout is staged");
    assert_eq!(root.location, Point::new(0.0, 0.0));
    assert_eq!(root.size.width, 11.0);
    assert_eq!(root.content_size.width, 11.0);
    assert_eq!(root.border.left, 1.0);
    assert_eq!(root.border.right, 1.0);
    assert_eq!(root.padding.left, 1.0);
    assert_eq!(root.padding.right, 1.0);

    let child = tree.final_layout(2).expect("child final layout is staged");
    assert_eq!(child.location, Point::new(0.0, 0.0));
    assert_eq!(child.size.width, 10.0);
    assert_eq!(child.content_size.width, 10.0);
    assert_eq!(child.scrollbar_size(), Size::ZERO);
    assert_eq!(child.border.left, 0.0);
    assert_eq!(child.border.right, 1.0);
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
