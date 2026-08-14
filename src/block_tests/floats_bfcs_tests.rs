use super::fixtures::{
    BlockTree, PublicBlockTree, ShapeProviderBehavior, all_writing_mode_directions,
    computed_overflow, fri06_atomic_participation, public_final_output, scalar_value,
};
use super::*;

fn fri06_c04_float_style<S: LayoutScalar>(
    flow_axes: FlowAxes,
    logical_size: crate::geometry::LogicalSizeOf<S>,
    side: Float,
    clear: Clear,
    logical_margin: crate::geometry::LogicalEdgesOf<S>,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        float: side,
        clear,
        size: flow_axes
            .physical_size(logical_size)
            .map(PreferredSizeOf::px),
        margin: flow_axes
            .physical_edges(logical_margin)
            .map(LengthAutoOf::px),
        ..NodeInputOf::default()
    }
}

fn fri06_c04_float_batch<S: LayoutScalar>(
    flow_axes: FlowAxes,
    children: impl IntoIterator<Item = (u32, NodeInputOf<S>)>,
) -> CompletedLayoutBatchOf<u32, S> {
    let logical_root_size =
        crate::geometry::LogicalSizeOf::new(scalar_value(100.0), scalar_value(160.0));
    let root_size = flow_axes.physical_size(logical_root_size);
    let children = children.into_iter().collect::<Vec<_>>();
    let child_ids = children.iter().map(|(node, _)| *node).collect::<Vec<_>>();
    let mut tree = PublicBlockTree::default()
        .with_children(0, child_ids)
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    for (node, style) in children {
        tree = tree.with_children(node, []).with_style(node, style);
    }

    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(root_size.map(AvailableOf::definite))
            .expect("finite float viewport is valid"),
    )
    .expect("rectangular float layout succeeds")
}

fn fri06_c04_block_layout_without_shape_provider<S: LayoutScalar>(
    children: impl IntoIterator<Item = (u32, NodeInputOf<S>)>,
) -> crate::test_support::layout_tree::OracleTreeOf<S> {
    let root_size = Size::new(scalar_value(100.0), scalar_value(160.0));
    let children = children.into_iter().collect::<Vec<_>>();
    let child_ids = children.iter().map(|(node, _)| *node).collect::<Vec<_>>();
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, child_ids)
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    for (node, style) in children {
        tree = tree.children(node, []).style(node, style);
    }

    crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::root_layout(
            Size::NONE,
            root_size.map(Some),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            root_size.map(AvailableOf::definite),
        ),
    )
    .expect("C04 block layout succeeds without a shape provider");
    tree
}

fn fri06_c04_expected_float_location<S: LayoutScalar>(
    flow_axes: FlowAxes,
    logical_origin: crate::geometry::LogicalPointOf<S>,
    logical_size: crate::geometry::LogicalSizeOf<S>,
) -> Point<S> {
    flow_axes.physical_point(
        logical_origin,
        logical_size,
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            scalar_value(100.0),
            scalar_value(160.0),
        )),
    )
}

fn fri06_c05_provider_tree<S: LayoutScalar>(
    exclusion: FloatExclusion,
    provider: ShapeProviderBehavior<S>,
    spacer_block_extent: Option<S>,
) -> PublicBlockTree<S> {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let root_size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    let mut children = vec![1];
    let mut tree = PublicBlockTree::default()
        .with_shape_provider(provider)
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                float_exclusion: exclusion,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(30.0)),
                    PreferredSizeOf::px(S::from_f64(30.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if let Some(block_extent) = spacer_block_extent {
        children.push(2);
        tree = tree.with_children(2, []).with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::px(block_extent),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    children.push(3);
    tree.with_children(0, children)
        .with_children(3, [])
        .with_style(
            3,
            NodeInputOf {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                ..NodeInputOf::default()
            },
        )
}

fn fri06_c05_provider_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(S::from_f64(100.0)),
        AvailableOf::definite(S::from_f64(80.0)),
    ))
    .expect("finite provider test viewport is valid")
}

fn fri06_c05_expected_query<S: LayoutScalar>() -> FloatExclusionQueryOf<S> {
    FloatExclusionQueryOf::try_new(
        ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(30.0), S::from_f64(30.0)))
            .unwrap(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        S::ZERO,
        S::from_f64(10.0),
    )
    .unwrap()
}

#[test]
fn fri06_c05_provider_role_valid_shape_reaches_exact_overlapping_band_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let tree = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: S::from_f64(30.0),
            },
            None,
        );
        let batch = compute_layout(&tree, 0, fri06_c05_provider_request())
            .expect("a valid shape interval reaches the typed provider front door");

        assert_eq!(tree.shape_queries(), vec![(1, fri06_c05_expected_query())]);
        assert_eq!(
            public_final_output(&batch, 3).location,
            Point::new(S::from_f64(30.0), S::ZERO),
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_role_empty_shape_is_valid_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let tree =
            fri06_c05_provider_tree::<S>(FloatExclusion::Shape, ShapeProviderBehavior::Empty, None);
        let batch = compute_layout(&tree, 0, fri06_c05_provider_request())
            .expect("an empty shape interval is a valid provider result");

        assert_eq!(tree.shape_queries(), vec![(1, fri06_c05_expected_query())]);
        assert_eq!(public_final_output(&batch, 3).location, Point::ZERO);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_role_margin_box_and_nonoverlapping_shape_make_zero_calls_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let margin_box = fri06_c05_provider_tree::<S>(
            FloatExclusion::MarginBox,
            ShapeProviderBehavior::Failure,
            None,
        );
        compute_layout(&margin_box, 0, fri06_c05_provider_request())
            .expect("margin-box exclusion does not require the provider");
        assert!(margin_box.shape_queries().is_empty());

        let nonoverlapping = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Failure,
            Some(S::from_f64(40.0)),
        );
        compute_layout(&nonoverlapping, 0, fri06_c05_provider_request())
            .expect("a non-overlapping shape does not require the provider");
        assert!(nonoverlapping.shape_queries().is_empty());
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_error_missing_provider_has_exact_context_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let tree = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Missing,
            None,
        );
        let error = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap_err();

        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::MissingContext(LayoutMissingContext::FloatExclusionProvider),
        );
        assert_eq!(tree.shape_queries(), vec![(1, fri06_c05_expected_query())]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_error_failure_preserves_measurement_and_context_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let tree = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Failure,
            None,
        );
        let error = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap_err();

        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        assert_eq!(error.kind(), &LayoutErrorKindOf::Measurement(()));
        assert_eq!(tree.shape_queries(), vec![(1, fri06_c05_expected_query())]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_error_mismatched_query_preserves_expected_and_actual_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let expected = fri06_c05_expected_query();
        let actual = FloatExclusionQueryOf::try_new(
            expected.margin_box(),
            expected.flow_axes(),
            S::from_f64(12.0),
            S::from_f64(22.0),
        )
        .unwrap();
        let tree = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Mismatch {
                query: actual,
                minimum: S::ZERO,
                maximum: S::from_f64(30.0),
            },
            None,
        );
        let error = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap_err();

        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::FloatExclusionProviderOutput {
                error: FloatExclusionIntervalErrorOf::QueryMismatch { expected, actual },
            }),
        );
        assert_eq!(tree.shape_queries(), vec![(1, expected)]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn fri06_c05_shape_tree<S: LayoutScalar>(
    flow_axes: FlowAxes,
    provider: ShapeProviderBehavior<S>,
    children: impl IntoIterator<Item = (u32, NodeInputOf<S>)>,
) -> PublicBlockTree<S> {
    let logical_root_size =
        crate::geometry::LogicalSizeOf::new(scalar_value(100.0), scalar_value(160.0));
    let root_size = flow_axes.physical_size(logical_root_size);
    let children = children.into_iter().collect::<Vec<_>>();
    let child_ids = children.iter().map(|(node, _)| *node).collect::<Vec<_>>();
    let mut tree = PublicBlockTree::default()
        .with_shape_provider(provider)
        .with_children(0, child_ids)
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    for (node, style) in children {
        tree = tree.with_children(node, []).with_style(node, style);
    }
    tree
}

fn fri06_c05_shape_request<S: LayoutScalar>(flow_axes: FlowAxes) -> LayoutRootRequestOf<S> {
    let root_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        scalar_value(100.0),
        scalar_value(160.0),
    ));
    LayoutRootRequestOf::viewport(root_size.map(AvailableOf::definite))
        .expect("finite shape-band viewport is valid")
}

fn fri06_c05_shape_float_style<S: LayoutScalar>(
    flow_axes: FlowAxes,
    side: Float,
    inline: f64,
    block: f64,
) -> NodeInputOf<S> {
    let mut style = fri06_c04_float_style(
        flow_axes,
        crate::geometry::LogicalSizeOf::new(scalar_value(inline), scalar_value(block)),
        side,
        Clear::None,
        crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );
    style.float_exclusion = FloatExclusion::Shape;
    style
}

fn fri06_c05_margin_float_style<S: LayoutScalar>(
    flow_axes: FlowAxes,
    side: Float,
    inline: f64,
    block: f64,
) -> NodeInputOf<S> {
    fri06_c04_float_style(
        flow_axes,
        crate::geometry::LogicalSizeOf::new(scalar_value(inline), scalar_value(block)),
        side,
        Clear::None,
        crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    )
}

fn fri06_c05_atomic_style<S: LayoutScalar>(
    flow_axes: FlowAxes,
    inline: f64,
    block: f64,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::InlineBlock,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        atomic_inline_participation: Some(fri06_atomic_participation()),
        size: flow_axes
            .physical_size(crate::geometry::LogicalSizeOf::new(
                scalar_value(inline),
                scalar_value(block),
            ))
            .map(PreferredSizeOf::px),
        ..NodeInputOf::default()
    }
}

fn fri06_c05_physical_inline_interval<S: LayoutScalar>(
    flow_axes: FlowAxes,
    logical_minimum: f64,
    logical_maximum: f64,
) -> (S, S) {
    let containing_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        scalar_value(100.0),
        scalar_value(160.0),
    ));
    let logical_size = crate::geometry::LogicalSizeOf::new(
        scalar_value(logical_maximum - logical_minimum),
        S::ZERO,
    );
    let physical_origin = flow_axes.physical_point(
        crate::geometry::LogicalPointOf::new(scalar_value(logical_minimum), S::ZERO),
        logical_size,
        containing_size,
    );
    let physical_size = flow_axes.physical_size(logical_size);
    match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => (physical_origin.x, physical_origin.x + physical_size.width),
        PhysicalAxis::Vertical => (physical_origin.y, physical_origin.y + physical_size.height),
    }
}

fn fri06_c05_expected_shape_query<S: LayoutScalar>(
    flow_axes: FlowAxes,
    margin_box_origin: crate::geometry::LogicalPointOf<S>,
    margin_box_size: crate::geometry::LogicalSizeOf<S>,
    block_start: S,
    block_end: S,
) -> FloatExclusionQueryOf<S> {
    let containing_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        scalar_value(100.0),
        scalar_value(160.0),
    ));
    let physical_margin_box = ScrollRectOf::try_new(
        flow_axes.physical_point(margin_box_origin, margin_box_size, containing_size),
        flow_axes.physical_size(margin_box_size),
    )
    .unwrap();
    let physical_band_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        S::ZERO,
        block_end - block_start,
    ));
    let physical_band_origin = flow_axes.physical_point(
        crate::geometry::LogicalPointOf::new(S::ZERO, block_start),
        crate::geometry::LogicalSizeOf::new(S::ZERO, block_end - block_start),
        containing_size,
    );
    let (band_minimum, band_maximum) = match flow_axes.block_axis() {
        PhysicalAxis::Horizontal => (
            physical_band_origin.x,
            physical_band_origin.x + physical_band_size.width,
        ),
        PhysicalAxis::Vertical => (
            physical_band_origin.y,
            physical_band_origin.y + physical_band_size.height,
        ),
    };
    FloatExclusionQueryOf::try_new(physical_margin_box, flow_axes, band_minimum, band_maximum)
        .unwrap()
}

#[test]
fn fri06_c05_shape_band_empty_partial_full_clipped_zero_opposing_stacked_cleared_and_overwide_both_scalars()
 {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let cases: [(ShapeProviderBehavior<S>, (f64, f64)); 6] = [
            (ShapeProviderBehavior::Empty, (0.0, 0.0)),
            (
                ShapeProviderBehavior::Interval {
                    minimum: scalar_value(0.0),
                    maximum: scalar_value(40.0),
                },
                (40.0, 0.0),
            ),
            (
                ShapeProviderBehavior::Interval {
                    minimum: scalar_value(0.0),
                    maximum: scalar_value(80.0),
                },
                (0.0, 20.0),
            ),
            (
                ShapeProviderBehavior::Interval {
                    minimum: scalar_value(-20.0),
                    maximum: scalar_value(40.0),
                },
                (40.0, 0.0),
            ),
            (
                ShapeProviderBehavior::Interval {
                    minimum: scalar_value(90.0),
                    maximum: scalar_value(110.0),
                },
                (0.0, 0.0),
            ),
            (
                ShapeProviderBehavior::Interval {
                    minimum: scalar_value(40.0),
                    maximum: scalar_value(40.0),
                },
                (0.0, 0.0),
            ),
        ];
        for (provider, expected) in cases {
            let tree = fri06_c05_shape_tree(
                flow_axes,
                provider,
                [
                    (
                        1,
                        fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                    ),
                    (
                        2,
                        fri06_c05_margin_float_style(flow_axes, Float::Left, 30.0, 10.0),
                    ),
                ],
            );
            let batch = compute_layout(&tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
            assert_eq!(
                public_final_output(&batch, 2).location,
                Point::new(scalar_value(expected.0), scalar_value(expected.1)),
                "shape interval did not replace the rectangular float collision"
            );
        }

        let stacked = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(100.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 20.0, 20.0),
                ),
                (
                    2,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 20.0, 30.0),
                ),
                (3, fri06_c05_atomic_style(flow_axes, 10.0, 10.0)),
            ],
        );
        let stacked_batch =
            compute_layout(&stacked, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&stacked_batch, 3).location,
            Point::new(scalar_value(40.0), S::ZERO),
            "same-side shape intervals must choose the farthest inward edge"
        );

        let opposing = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(100.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 40.0, 20.0),
                ),
                (
                    2,
                    fri06_c05_shape_float_style(flow_axes, Float::Right, 60.0, 30.0),
                ),
                (3, fri06_c05_atomic_style(flow_axes, 10.0, 10.0)),
            ],
        );
        let opposing_batch =
            compute_layout(&opposing, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&opposing_batch, 3).location,
            Point::new(S::ZERO, scalar_value(20.0)),
            "opposing shapes must close the first band and advance to the finite transition"
        );

        let mut cleared = fri06_c05_margin_float_style(flow_axes, Float::Left, 10.0, 10.0);
        cleared.clear = Clear::Left;
        let cleared_tree = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Empty,
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                ),
                (2, cleared),
                (
                    3,
                    fri06_c05_margin_float_style(flow_axes, Float::Right, 120.0, 10.0),
                ),
            ],
        );
        let cleared_batch =
            compute_layout(&cleared_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&cleared_batch, 2).location,
            Point::new(S::ZERO, scalar_value(20.0)),
        );
        assert_eq!(
            public_final_output(&cleared_batch, 3).location,
            Point::new(scalar_value(-20.0), S::ZERO),
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_shape_flow_physical_intervals_preserve_both_logical_sides_all_flows_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in all_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);

            let (minimum, maximum) = fri06_c05_physical_inline_interval(flow_axes, 0.0, 25.0);
            let start_tree = fri06_c05_shape_tree(
                flow_axes,
                ShapeProviderBehavior::Interval { minimum, maximum },
                [
                    (
                        1,
                        fri06_c05_shape_float_style(flow_axes, Float::Left, 40.0, 20.0),
                    ),
                    (
                        2,
                        fri06_c05_margin_float_style(flow_axes, Float::Left, 10.0, 10.0),
                    ),
                    (3, fri06_c05_atomic_style(flow_axes, 10.0, 10.0)),
                ],
            );
            let start_batch =
                compute_layout(&start_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
            assert_eq!(
                public_final_output(&start_batch, 2).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(25.0), S::ZERO),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0)),
                ),
                "line-start shape side was lost for {writing_mode:?} {direction:?}",
            );
            assert_eq!(
                public_final_output(&start_batch, 3).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(35.0), S::ZERO),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0)),
                ),
                "line band did not reuse the mapped line-start identity for {writing_mode:?} {direction:?}",
            );
            let expected_start_query = fri06_c05_expected_shape_query(
                flow_axes,
                crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                crate::geometry::LogicalSizeOf::new(scalar_value(40.0), scalar_value(20.0)),
                S::ZERO,
                scalar_value(10.0),
            );
            assert_eq!(
                start_tree.shape_queries(),
                vec![
                    (1, expected_start_query),
                    (1, expected_start_query),
                    (1, expected_start_query),
                ],
                "float, line, and final float candidates must share the exact mapped query for {writing_mode:?} {direction:?}",
            );

            let (minimum, maximum) = fri06_c05_physical_inline_interval(flow_axes, 75.0, 100.0);
            let end_tree = fri06_c05_shape_tree(
                flow_axes,
                ShapeProviderBehavior::Interval { minimum, maximum },
                [
                    (
                        1,
                        fri06_c05_shape_float_style(flow_axes, Float::Right, 40.0, 20.0),
                    ),
                    (
                        2,
                        fri06_c05_margin_float_style(flow_axes, Float::Right, 10.0, 10.0),
                    ),
                ],
            );
            let end_batch =
                compute_layout(&end_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
            assert_eq!(
                public_final_output(&end_batch, 2).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(65.0), S::ZERO),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0)),
                ),
                "line-end shape side was lost for {writing_mode:?} {direction:?}",
            );
            let expected_end_query = fri06_c05_expected_shape_query(
                flow_axes,
                crate::geometry::LogicalPointOf::new(scalar_value(60.0), S::ZERO),
                crate::geometry::LogicalSizeOf::new(scalar_value(40.0), scalar_value(20.0)),
                S::ZERO,
                scalar_value(10.0),
            );
            assert_eq!(
                end_tree.shape_queries(),
                vec![(1, expected_end_query), (1, expected_end_query)],
                "line-end candidate passes must retain the final physical margin box and ordered band for {writing_mode:?} {direction:?}",
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_shape_query_float_line_and_bfc_consumers_record_exact_candidate_once_per_pass_both_scalars()
 {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let expected_margin_box = ScrollRectOf::try_new(
            Point::ZERO,
            Size::new(scalar_value(80.0), scalar_value(20.0)),
        )
        .unwrap();
        let expected = FloatExclusionQueryOf::try_new(
            expected_margin_box,
            flow_axes,
            S::ZERO,
            scalar_value(10.0),
        )
        .unwrap();

        let float_tree = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(40.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                ),
                (
                    2,
                    fri06_c05_margin_float_style(flow_axes, Float::Left, 30.0, 10.0),
                ),
            ],
        );
        compute_layout(&float_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            float_tree.shape_queries(),
            vec![(1, expected), (1, expected)]
        );

        let bfc_tree = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(40.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                ),
                (
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                        size: Size::new(
                            PreferredSizeOf::px(scalar_value(30.0)),
                            PreferredSizeOf::px(scalar_value(10.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                ),
            ],
        );
        let bfc_batch = compute_layout(&bfc_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&bfc_batch, 2).location,
            Point::new(scalar_value(40.0), S::ZERO),
        );
        assert_eq!(bfc_tree.shape_queries(), vec![(1, expected), (1, expected)]);

        let line_tree = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(40.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                ),
                (2, fri06_c05_atomic_style(flow_axes, 60.0, 10.0)),
            ],
        );
        compute_layout(&line_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(line_tree.shape_queries(), vec![(1, expected)]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_float_dominated_terminal_extent_stops_at_the_float_edge() {
    fn assert_lane<S: LayoutScalar>() {
        let root = NodeInputOf {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(180.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        };
        let mut floating = fri06_c05_shape_float_style(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Float::Left,
            44.0,
            60.0,
        );
        floating.float_exclusion = FloatExclusion::Shape;
        let segment = ShapedInlineSegmentOf::try_new(
            InlineSegmentId::new(1),
            S::from_f64(48.164_062_5),
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(14.8))
                .unwrap(),
            BidiLevel::try_new(0).unwrap(),
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )
        .unwrap();
        let text = LayoutInputOf::inline_text(InlineTextInputOf::try_new(vec![segment]).unwrap());
        let atomic = |inline_extent, following_break| NodeInputOf {
            display: Display::InlineBlock,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(inline_extent)),
                PreferredSizeOf::px(S::from_f64(16.0)),
            ),
            atomic_inline_participation: Some(
                AtomicInlineParticipationOf::try_new(
                    BidiLevel::try_new(0).unwrap(),
                    following_break,
                )
                .unwrap(),
            ),
            ..NodeInputOf::default()
        };
        let atomics = [
            atomic(34.0, InlineBreakOpportunityOf::prohibited()),
            atomic(38.0, InlineBreakOpportunityOf::prohibited()),
            atomic(42.0, InlineBreakOpportunityOf::allowed()),
            atomic(46.0, InlineBreakOpportunityOf::prohibited()),
        ];
        let mut tree = PublicBlockTree::default()
            .with_shape_provider(ShapeProviderBehavior::Bands(vec![
                (S::ZERO, S::from_f64(21.2), S::ZERO, S::from_f64(44.0)),
                (
                    S::from_f64(21.2),
                    S::from_f64(37.2),
                    S::ZERO,
                    S::from_f64(44.0),
                ),
            ]))
            .with_children(0, [1, 2, 3, 4, 5, 6])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(0, root)
            .with_style(1, floating)
            .with_style(2, NodeInputOf::non_box())
            .with_layout_input(2, text);
        for (offset, style) in atomics.into_iter().enumerate() {
            let node = u32::try_from(offset + 3).unwrap();
            tree = tree.with_children(node, []).with_style(node, style);
        }

        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(180.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .expect("shape-excluded inline layout succeeds");

        assert_eq!(
            batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output()
                .size
                .height,
            S::from_f64(60.0),
        );
        assert_eq!(
            public_final_output(&batch, 0).size.height,
            S::from_f64(60.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_float_shape_slots_follow_visual_order_before_physical_projection() {
    fn assert_lane<S: LayoutScalar>(direction: Direction) {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
        let root = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::HorizontalTb,
            direction,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(180.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        };
        let floating = fri06_c05_shape_float_style(flow_axes, Float::Right, 44.0, 60.0);
        let (shape_minimum, shape_maximum) = match direction {
            Direction::Ltr => (S::from_f64(136.0), S::from_f64(180.0)),
            Direction::Rtl => (S::ZERO, S::from_f64(44.0)),
        };
        let bidi_level = BidiLevel::try_new(u8::from(direction == Direction::Rtl)).unwrap();
        let segments = [(1, 66.0), (2, 114.0)]
            .into_iter()
            .map(|(id, inline_extent)| {
                ShapedInlineSegmentOf::try_new(
                    InlineSegmentId::new(id),
                    S::from_f64(inline_extent),
                    InlineMetricsOf::from_line_height_and_baseline(
                        S::from_f64(20.0),
                        S::from_f64(14.0),
                    )
                    .unwrap(),
                    bidi_level,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let text = LayoutInputOf::inline_text(InlineTextInputOf::try_new(segments).unwrap());
        let tree = PublicBlockTree::default()
            .with_shape_provider(ShapeProviderBehavior::Interval {
                minimum: shape_minimum,
                maximum: shape_maximum,
            })
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(0, root)
            .with_style(1, floating)
            .with_style(2, NodeInputOf::non_box())
            .with_layout_input(2, text);

        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(180.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .expect("shape-backed float slot layout succeeds");
        let flow_inline_starts = batch
            .unrounded_inline_fragments()
            .iter()
            .map(|entry| {
                let rect = entry.fragment().rect();
                match direction {
                    Direction::Ltr => rect.origin().x,
                    Direction::Rtl => rect.origin().x + rect.size().width,
                }
            })
            .collect::<Vec<_>>();
        let expected = match direction {
            Direction::Ltr => vec![S::ZERO, S::from_f64(66.0)],
            Direction::Rtl => vec![S::from_f64(66.0), S::from_f64(180.0)],
        };
        assert_eq!(
            flow_inline_starts, expected,
            "logical slots are LTR 0/66 and RTL 114/0 before FlowAxes projects them",
        );
    }

    assert_lane::<f32>(Direction::Ltr);
    assert_lane::<f64>(Direction::Ltr);
    assert_lane::<f32>(Direction::Rtl);
    assert_lane::<f64>(Direction::Rtl);
}

#[test]
fn fri06_c04_float_place_mapped_sides_and_clear_values_all_flows_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        for (writing_mode, direction) in all_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let batch = fri06_c04_float_batch(
                flow_axes,
                [
                    (
                        1,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(30.0),
                                scalar_value(20.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        2,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(25.0),
                                scalar_value(30.0),
                            ),
                            Float::Right,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        3,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(20.0),
                                scalar_value(10.0),
                            ),
                            Float::Left,
                            Clear::Left,
                            zero,
                        ),
                    ),
                    (
                        4,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(15.0),
                                scalar_value(10.0),
                            ),
                            Float::Right,
                            Clear::Right,
                            zero,
                        ),
                    ),
                    (
                        5,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(10.0),
                                scalar_value(10.0),
                            ),
                            Float::Left,
                            Clear::Both,
                            zero,
                        ),
                    ),
                ],
            );

            for (node, inline, block, inline_size, block_size) in [
                (1, 0.0, 0.0, 30.0, 20.0),
                (2, 75.0, 0.0, 25.0, 30.0),
                (3, 0.0, 20.0, 20.0, 10.0),
                (4, 85.0, 30.0, 15.0, 10.0),
                (5, 0.0, 40.0, 10.0, 10.0),
            ] {
                assert_eq!(
                    public_final_output(&batch, node).location,
                    fri06_c04_expected_float_location(
                        flow_axes,
                        crate::geometry::LogicalPointOf::new(
                            scalar_value(inline),
                            scalar_value(block),
                        ),
                        crate::geometry::LogicalSizeOf::new(
                            scalar_value(inline_size),
                            scalar_value(block_size),
                        ),
                    ),
                    "mapped float placement diverged for {writing_mode:?} {direction:?} node {node}",
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_place_margin_box_remains_physical_for_later_float_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        let margin_box_float = fri06_c04_float_style(
            flow_axes,
            crate::geometry::LogicalSizeOf::new(scalar_value(80.0), scalar_value(20.0)),
            Float::Left,
            Clear::None,
            zero,
        );
        let tree = fri06_c04_block_layout_without_shape_provider([
            (1, margin_box_float),
            (
                2,
                fri06_c04_float_style(
                    flow_axes,
                    crate::geometry::LogicalSizeOf::new(scalar_value(30.0), scalar_value(10.0)),
                    Float::Left,
                    Clear::None,
                    zero,
                ),
            ),
        ]);

        assert_eq!(
            tree.layout(2).expect("later float was laid out").location,
            Point::new(S::ZERO, scalar_value(20.0)),
            "later float must avoid the Shape float's physical margin box",
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_bfc_margin_box_remains_physical_without_provider_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        let margin_box_float = fri06_c04_float_style(
            flow_axes,
            crate::geometry::LogicalSizeOf::new(scalar_value(80.0), scalar_value(20.0)),
            Float::Left,
            Clear::None,
            zero,
        );
        let tree = fri06_c04_block_layout_without_shape_provider([
            (1, margin_box_float),
            (
                2,
                NodeInputOf {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(
                        PreferredSizeOf::px(scalar_value(30.0)),
                        PreferredSizeOf::px(scalar_value(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            ),
        ]);

        assert_eq!(
            tree.layout(2)
                .expect("qualifying BFC was laid out")
                .location,
            Point::new(S::ZERO, scalar_value(20.0)),
            "qualifying BFC must avoid the Shape float's physical margin box",
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_direct_compute_inherits_float_into_internal_line_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let root_size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
        let fixed_size = |width, height| {
            Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::px(S::from_f64(height)),
            )
        };
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2, 3])
            .children(1, [])
            .children(2, [])
            .children(3, [4, 5])
            .children(4, [])
            .children(5, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: root_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    size: fixed_size(0.0, 0.0),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: fixed_size(30.0, 30.0),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .style(
                4,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Right,
                    size: fixed_size(20.0, 20.0),
                    ..NodeInputOf::default()
                },
            )
            .style(
                5,
                NodeInputOf {
                    display: Display::InlineBlock,
                    size: fixed_size(40.0, 10.0),
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    ..NodeInputOf::default()
                },
            );

        crate::compute_block(
            &mut tree,
            0,
            ComputeInputOf::root_layout(
                Size::NONE,
                root_size.map(Some),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                root_size.map(AvailableOf::definite),
            ),
        )
        .expect("direct block compute succeeds");

        let parent_float = tree.layout(2).expect("parent float is laid out");
        assert_eq!(parent_float.source_index, SourceIndex::new(1));
        let ordinary = tree.layout(3).expect("ordinary child is laid out");
        assert_eq!(ordinary.source_index, SourceIndex::new(2));
        assert_eq!(ordinary.location, Point::ZERO);
        assert_eq!(ordinary.size.width, S::from_f64(100.0));

        let local_float = tree.layout(4).expect("child-local float is laid out");
        assert_eq!(local_float.source_index, SourceIndex::ZERO);
        assert_eq!(local_float.location, Point::new(S::from_f64(80.0), S::ZERO));
        let internal_line = tree.layout(5).expect("internal atomic line is laid out");
        assert_eq!(internal_line.source_index, SourceIndex::new(1));
        assert_eq!(
            internal_line.location,
            Point::new(S::from_f64(30.0), S::ZERO)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_ledger_full_span_asymmetric_opposing_and_source_order_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        for (writing_mode, direction) in all_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let candidate_margin = crate::geometry::LogicalEdgesOf::new(
                scalar_value(3.0),
                scalar_value(7.0),
                scalar_value(5.0),
                scalar_value(5.0),
            );
            let batch = fri06_c04_float_batch(
                flow_axes,
                [
                    (
                        1,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(40.0),
                                scalar_value(20.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        2,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(30.0),
                                scalar_value(60.0),
                            ),
                            Float::Right,
                            Clear::Left,
                            zero,
                        ),
                    ),
                    (
                        3,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(40.0),
                                scalar_value(30.0),
                            ),
                            Float::Left,
                            Clear::None,
                            candidate_margin,
                        ),
                    ),
                    (
                        4,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(20.0),
                                scalar_value(10.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                ],
            );

            assert_eq!(
                public_final_output(&batch, 3).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(3.0), scalar_value(25.0),),
                    crate::geometry::LogicalSizeOf::new(scalar_value(40.0), scalar_value(30.0),),
                ),
                "full-span collision was missed for {writing_mode:?} {direction:?}",
            );
            assert_eq!(
                public_final_output(&batch, 4).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(40.0), S::ZERO,),
                    crate::geometry::LogicalSizeOf::new(scalar_value(20.0), scalar_value(10.0),),
                ),
                "same-side source order diverged for {writing_mode:?} {direction:?}",
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_ledger_evaluates_each_float_span_pair_once_per_candidate_pass() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
        let logical_container =
            crate::geometry::LogicalSizeOf::new(scalar_value(100.0), scalar_value(120.0));
        let mut exclusions = FloatExclusions::new(
            flow_axes,
            flow_axes.physical_size(logical_container),
            scalar_value(100.0),
            crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
        );
        exclusions.record_test_float(
            FloatLedgerSide::LineStart,
            FloatExclusion::MarginBox,
            crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
            crate::geometry::LogicalSizeOf::new(scalar_value(40.0), scalar_value(20.0)),
        );
        exclusions.record_test_float(
            FloatLedgerSide::LineEnd,
            FloatExclusion::MarginBox,
            crate::geometry::LogicalPointOf::new(scalar_value(70.0), scalar_value(20.0)),
            crate::geometry::LogicalSizeOf::new(scalar_value(30.0), scalar_value(60.0)),
        );
        exclusions.record_test_float(
            FloatLedgerSide::LineStart,
            FloatExclusion::MarginBox,
            crate::geometry::LogicalPointOf::new(scalar_value(10.0), scalar_value(90.0)),
            crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0)),
        );
        exclusions.record_test_float(
            FloatLedgerSide::LineStart,
            FloatExclusion::MarginBox,
            crate::geometry::LogicalPointOf::new(scalar_value(20.0), S::ZERO),
            crate::geometry::LogicalSizeOf::new(scalar_value(30.0), scalar_value(40.0)),
        );

        let band = exclusions.query_rectangular_line_band(S::ZERO, scalar_value(40.0));
        assert_eq!(band.inline_start, scalar_value(50.0));
        assert_eq!(band.inline_end, scalar_value(70.0));
        assert_eq!(band.next_transition, Some(scalar_value(20.0)));
        assert_eq!(band.evaluated, 4);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_ledger_shape_is_not_approximated_by_rectangular_line_band() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut exclusions = FloatExclusions::new(
        flow_axes,
        Size::new(100.0, 100.0),
        100.0,
        crate::geometry::LogicalEdgesOf::new(0.0, 0.0, 0.0, 0.0),
    );
    exclusions.record_test_float(
        FloatLedgerSide::LineStart,
        FloatExclusion::Shape,
        crate::geometry::LogicalPointOf::new(0.0, 0.0),
        crate::geometry::LogicalSizeOf::new(80.0, 50.0),
    );

    let band = exclusions.query_rectangular_line_band(0.0, 20.0);
    assert_eq!(band.inline_start, 0.0);
    assert_eq!(band.inline_end, 100.0);
    assert_eq!(band.next_transition, None);
    assert_eq!(band.evaluated, 1);
}

#[test]
fn fri06_c04_float_progress_zero_band_exact_transition_and_overwide_side_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        for (writing_mode, direction) in all_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let batch = fri06_c04_float_batch(
                flow_axes,
                [
                    (
                        1,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(50.0),
                                scalar_value(20.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        2,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(50.0),
                                scalar_value(30.0),
                            ),
                            Float::Right,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        3,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(10.0),
                                scalar_value(15.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        4,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(120.0),
                                scalar_value(10.0),
                            ),
                            Float::Right,
                            Clear::None,
                            zero,
                        ),
                    ),
                ],
            );

            assert_eq!(
                public_final_output(&batch, 3).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(S::ZERO, scalar_value(20.0)),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(15.0),),
                ),
                "zero band did not advance to the exact transition for {writing_mode:?} {direction:?}",
            );
            assert_eq!(
                public_final_output(&batch, 4).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(-20.0), scalar_value(35.0),),
                    crate::geometry::LogicalSizeOf::new(scalar_value(120.0), scalar_value(10.0),),
                ),
                "overwide float did not terminate on line-end for {writing_mode:?} {direction:?}",
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn block_float_contributes_to_intrinsic_width_and_places_from_right_edge() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(80.0)),
            border: Edges::all(Length::px(2.0)),
            ..NodeInput::default()
        },
    );
    for node in [2, 3, 4] {
        tree.insert_style(
            node,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        );
        tree.insert_measure(
            node,
            ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
        );
    }

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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(154.0, 80.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(102.0, 2.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(52.0, 2.0)
    );
    assert_eq!(
        tree.layout(4).expect("child layout is staged").location,
        Point::new(2.0, 2.0)
    );
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(100.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats_above_full_width_float() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 200.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_overflow_clip_zero_width_child_ignores_float_exclusion_without_clear() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Clip, Overflow::Clip),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_right_alignment_without_float_exclusion() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(150.0, 0.0)
    );
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_center_alignment_without_float_exclusion() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(75.0, 0.0)
    );
}

#[test]
fn block_bfc_float_content_size_height_excludes_container_top_inset() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                border: Edges {
                    top: Length::px(5.0),
                    ..Edges::all(Length::ZERO)
                },
                padding: Edges {
                    top: Length::px(10.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 15.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().content_size.height, 40.0);
}

#[test]
fn block_bfc_clear_only_visible_child_keeps_normal_x_while_clearing_y() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(150.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Left,
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(150.0, 50.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 70.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_left_sits_below_left_float_row() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Left,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 100.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_right_sits_below_all_right_floats() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Right,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 200.0)
    );
}
