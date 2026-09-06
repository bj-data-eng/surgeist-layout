use super::fixtures::{
    Fri08C03NestedAtomicTree, Fri08C03NestedFlowCase, Fri08C03NestedMeasureError,
    Fri08C03NestedMeasureMode, Fri08C06RAtomicTree, GridAxisMappingInput, SubgridEligibilityInput,
    assert_fri08_c03_nested_candidate_bounds_edges_and_reversal,
    assert_fri08_c04_standalone_nested_flows, baseline_measure, compute_oracle_grid,
    compute_oracle_grid_output, computed_overflow, empty_subgrid_track,
    fri05_c05_grid_sizing_input, fri08_c01_placement_compute, fri08_c01_placement_output,
    fri08_c01_topology_for_style, fri08_c02_auto_fit_output, fri08_c02_auto_fit_repeat,
    fri08_c03_auto_fit_batch, fri08_c03_auto_fit_named_repeat, fri08_c03_nested_projection_tree,
    lp, map_grid_axis, subgrid_axis_report, subgrid_eligibility, subgrid_track, subgrid_track_of,
    tagged_baseline, vertical_baseline_measure,
};
use super::*;

fn assert_locked_major_sparse_frontiers<S: LayoutScalar>() {
    for flow in [
        GridAutoFlow::Row,
        GridAutoFlow::RowDense,
        GridAutoFlow::Column,
        GridAutoFlow::ColumnDense,
    ] {
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, [2, 8, 4, 3, 5, 6, 7])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_auto_flow: flow,
                    grid_template_columns: vec![TrackComponentOf::px(S::from_f64(10.0)); 3],
                    grid_template_rows: vec![TrackComponentOf::px(S::from_f64(10.0)); 3],
                    grid_auto_columns: vec![TrackComponentOf::px(S::from_f64(10.0))],
                    grid_auto_rows: vec![TrackComponentOf::px(S::from_f64(10.0))],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            );
        for (node, major, minor) in [
            (
                2,
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(2).unwrap(),
            ),
            (
                8,
                GridPlacement::try_line(2).unwrap(),
                GridPlacement::try_line(6).unwrap(),
            ),
            (
                3,
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_span(2).unwrap(),
            ),
            (4, GridPlacement::try_line(1).unwrap(), GridPlacement::AUTO),
            (5, GridPlacement::try_line(2).unwrap(), GridPlacement::AUTO),
            (
                6,
                GridPlacement::try_lines(1, 3).unwrap(),
                GridPlacement::AUTO,
            ),
            (7, GridPlacement::try_line(2).unwrap(), GridPlacement::AUTO),
        ] {
            tree = tree.style(
                node,
                NodeInputOf {
                    item_order: if node == 3 {
                        ItemOrder::new(-1)
                    } else {
                        ItemOrder::ZERO
                    },
                    grid_column: if flow.is_column() { major } else { minor },
                    grid_row: if flow.is_column() { minor } else { major },
                    ..NodeInputOf::default()
                },
            );
        }
        let batch = fri08_c01_placement_compute(&tree);
        assert_eq!(
            fri08_c01_placement_output(&batch, 3).source_index,
            SourceIndex::new(3)
        );
        assert_eq!(
            fri08_c01_placement_output(&batch, 4).source_index,
            SourceIndex::new(2)
        );
        for (node, major, minor) in [
            (3, 0.0, 20.0),
            (4, 0.0, if flow.is_dense() { 0.0 } else { 40.0 }),
            (5, 10.0, 0.0),
            (6, 0.0, if flow.is_dense() { 40.0 } else { 60.0 }),
            (7, 10.0, if flow.is_dense() { 10.0 } else { 70.0 }),
        ] {
            let expected = if flow.is_column() {
                Point::new(S::from_f64(major), S::from_f64(minor))
            } else {
                Point::new(S::from_f64(minor), S::from_f64(major))
            };
            assert_eq!(
                fri08_c01_placement_output(&batch, node).location,
                expected,
                "{flow:?}, node {node}"
            );
        }
    }
}

#[test]
fn locked_major_sparse_frontiers_both_scalars() {
    assert_locked_major_sparse_frontiers::<f32>();
    assert_locked_major_sparse_frontiers::<f64>();
}

fn assert_overfull_inherited_subgrid<S: LayoutScalar>() {
    for flow in [
        GridAutoFlow::Row,
        GridAutoFlow::Column,
        GridAutoFlow::RowDense,
        GridAutoFlow::ColumnDense,
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3, 4, 5])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: vec![TrackComponentOf::px(S::from_f64(20.0))],
                    grid_template_rows: vec![TrackComponentOf::px(S::from_f64(20.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: subgrid_track_of(),
                    grid_template_rows: subgrid_track_of(),
                    grid_auto_flow: flow,
                    ..NodeInputOf::default()
                },
            )
            .style(3, NodeInputOf::default())
            .style(4, NodeInputOf::default())
            .style(
                5,
                NodeInputOf {
                    grid_column: GridPlacement::try_span(5).unwrap(),
                    grid_row: GridPlacement::try_span(5).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch = fri08_c01_placement_compute(&tree);
        for node in [2, 3, 4, 5] {
            let output = fri08_c01_placement_output(&batch, node);
            assert_eq!(output.location, Point::ZERO, "{flow:?} node {node}");
            assert_eq!(
                output.size,
                Size::splat(S::from_f64(20.0)),
                "{flow:?} node {node}"
            );
        }
    }
}

#[test]
fn overfull_inherited_subgrid_clamps_both_scalars() {
    assert_overfull_inherited_subgrid::<f32>();
    assert_overfull_inherited_subgrid::<f64>();
}

fn fri08_c08_t03_cache_input<S: LayoutScalar>(
    run_mode: RunMode,
    available: f64,
) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        run_mode,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::splat(AvailableOf::definite(S::from_f64(available))),
    )
}

fn assert_fri08_c08_t03_retained_state_prepares_and_commits<S: LayoutScalar>() {
    let mut tree = Fri08C06RAtomicTree::new(PublicLayoutTreeOf::new());
    assert!(tree.retained.unrounded.is_empty());
    assert!(tree.retained.final_outputs.is_empty());
    assert!(tree.retained.caches.is_empty());

    let invalidated_input = fri08_c08_t03_cache_input::<S>(RunMode::PerformRootLayout, 90.0);
    let cleared_input = fri08_c08_t03_cache_input::<S>(RunMode::PerformRootLayout, 100.0);
    let replacement_input = fri08_c08_t03_cache_input::<S>(RunMode::ComputeSize, 110.0);
    let replacement_miss = fri08_c08_t03_cache_input::<S>(RunMode::ComputeSize, 111.0);
    let invalidated_cache =
        ComputeOutputOf::from_outer_size(Size::new(S::from_f64(9.0), S::from_f64(19.0)));
    let cleared_cache =
        ComputeOutputOf::from_outer_size(Size::new(S::from_f64(10.0), S::from_f64(20.0)));
    let replacement_cache =
        ComputeOutputOf::from_outer_size(Size::new(S::from_f64(11.0), S::from_f64(21.0)));
    let seed_unrounded = NodeOutputOf {
        size: Size::new(S::from_f64(9.25), S::from_f64(19.5)),
        ..NodeOutputOf::default()
    };
    let seed_final = NodeOutputOf {
        size: Size::new(S::from_f64(9.0), S::from_f64(20.0)),
        ..NodeOutputOf::default()
    };
    let seed = CompletedLayoutBatchOf::from_entries(
        vec![LayoutOutputEntryOf::new(9, seed_unrounded)],
        vec![LayoutOutputEntryOf::new(9, seed_final)],
        Vec::new(),
        Vec::new(),
        vec![
            LayoutCacheStoreEntryOf::new(
                9,
                invalidated_input,
                CacheKeyContext::new(),
                invalidated_cache,
            ),
            LayoutCacheStoreEntryOf::new(10, cleared_input, CacheKeyContext::new(), cleared_cache),
        ],
        Vec::new(),
        Vec::new(),
    );
    seed.apply_to(&mut tree).unwrap();

    let replacement_unrounded = NodeOutputOf {
        size: Size::new(S::from_f64(12.25), S::from_f64(22.5)),
        ..NodeOutputOf::default()
    };
    let replacement_final = NodeOutputOf {
        size: Size::new(S::from_f64(12.0), S::from_f64(23.0)),
        ..NodeOutputOf::default()
    };
    let replacement = CompletedLayoutBatchOf::from_entries(
        vec![LayoutOutputEntryOf::new(12, replacement_unrounded)],
        vec![LayoutOutputEntryOf::new(12, replacement_final)],
        Vec::new(),
        Vec::new(),
        vec![LayoutCacheStoreEntryOf::new(
            10,
            replacement_input,
            CacheKeyContext::new(),
            replacement_cache,
        )],
        vec![LayoutCacheClearEntry::new(10)],
        vec![9],
    );
    let retained_before_prepare = tree.retained.clone();
    let prepared = tree.prepare_layout_batch(&replacement).unwrap();

    assert_eq!(tree.retained, retained_before_prepare);
    assert!(!prepared.unrounded.contains_key(&9));
    assert!(!prepared.final_outputs.contains_key(&9));
    assert!(!prepared.caches.contains_key(&9));
    assert_eq!(prepared.unrounded.get(&12), Some(&replacement_unrounded));
    assert_eq!(prepared.final_outputs.get(&12), Some(&replacement_final));
    assert_eq!(
        prepared.caches[&10].get_with_context(&cleared_input, CacheKeyContext::new()),
        None,
        "cache clearing removes the prior run-mode entry before replacement storage"
    );
    assert_eq!(
        prepared.caches[&10].get_with_context(&replacement_input, CacheKeyContext::new()),
        Some(replacement_cache)
    );
    assert_eq!(
        prepared.caches[&10].get_with_context(&replacement_miss, CacheKeyContext::new()),
        None,
        "cache reuse remains keyed by the complete compute input"
    );

    tree.commit_layout_batch(prepared);
    assert_eq!(
        tree.retained.unrounded.get(&12),
        Some(&replacement_unrounded)
    );
    assert_eq!(
        tree.retained.final_outputs.get(&12),
        Some(&replacement_final)
    );
    tree.cache_queries.borrow_mut().clear();
    assert_eq!(
        tree.cache_get(10, &replacement_input, CacheKeyContext::new()),
        Some(replacement_cache)
    );
    assert_eq!(tree.cache_queries.borrow().as_slice(), &[(10, true)]);
}

#[test]
fn fri08_c08_t03_retained_state_prepares_invalidates_and_commits_for_both_scalars() {
    assert_fri08_c08_t03_retained_state_prepares_and_commits::<f32>();
    assert_fri08_c08_t03_retained_state_prepares_and_commits::<f64>();
}

fn assert_fri08_c08_t03_retained_state_failed_layout_rolls_back<S: LayoutScalar>() {
    let mut tree = Fri08C03NestedAtomicTree::<S>::new();
    let request = Fri08C03NestedAtomicTree::<S>::request();
    let cold = compute_layout(&tree, 1, request).unwrap();
    cold.apply_to(&mut tree).unwrap();
    let retained_before_failure = tree.retained.clone();

    tree.measure_mode
        .set(Fri08C03NestedMeasureMode::ProviderError);
    let error = compute_layout_invalidated(&tree, 1, request, &[4])
        .expect_err("provider failure returns no completed batch");

    assert_eq!(error.site(), LayoutErrorSiteOf::Node(4));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(Fri08C03NestedMeasureError::Provider)
    ));
    assert_eq!(tree.retained, retained_before_failure);
}

#[test]
fn fri08_c08_t03_retained_state_failed_layout_has_no_partial_publication_for_both_scalars() {
    assert_fri08_c08_t03_retained_state_failed_layout_rolls_back::<f32>();
    assert_fri08_c08_t03_retained_state_failed_layout_rolls_back::<f64>();
}

#[test]
fn fri08_c03_auto_fit_automatic_span_sum_not_item_count_or_cell_area_sets_demand() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(40.0)),
                grid_template_columns: vec![fri08_c03_auto_fit_named_repeat(TrackRepeat::AutoFit)],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Center),
                align_content: Some(AlignContent::Start),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_span(2).expect("two-track automatic span"),
                grid_row: GridPlacement::try_span(2).expect("two-cell lane-axis span control"),
                ..NodeInput::DEFAULT
            },
        );
    let batch = fri08_c03_auto_fit_batch(&tree, Size::new(140.0, 40.0));
    let automatic = fri08_c01_placement_output(&batch, 2);

    assert_eq!((automatic.location.x, automatic.size.width), (25.0, 90.0));
}

fn assert_fri08_c03_auto_fit_candidates_across_flows<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
            let logical_container = LogicalSizeOf::new(scalar(120.0), scalar(120.0));
            let physical_container = flow_axes.physical_size(logical_container);
            for auto_flow in [
                GridAutoFlow::Row,
                GridAutoFlow::RowDense,
                GridAutoFlow::Column,
                GridAutoFlow::ColumnDense,
            ] {
                for tolerance in [
                    GridFlowToleranceOf::Normal {
                        font_size: scalar(16.0),
                    },
                    GridFlowToleranceOf::Infinite,
                ] {
                    let grid_axis = grid_axis_for_lanes(auto_flow);
                    let repeated = fri08_c03_auto_fit_named_repeat(TrackRepeat::AutoFit);
                    let (columns, rows) = match grid_axis {
                        GridAxisKind::Column => {
                            (vec![repeated], vec![TrackComponentOf::px(scalar(40.0))])
                        }
                        GridAxisKind::Row => {
                            (vec![TrackComponentOf::px(scalar(40.0))], vec![repeated])
                        }
                    };
                    let (explicit_style, automatic_style) = match grid_axis {
                        GridAxisKind::Column => (
                            NodeInputOf {
                                grid_column: GridPlacement::try_line(2)
                                    .expect("second retained identity"),
                                grid_row: GridPlacement::try_line(1).expect("single lane row"),
                                ..NodeInputOf::default()
                            },
                            NodeInputOf {
                                grid_row: GridPlacement::try_line(1).expect("single lane row"),
                                ..NodeInputOf::default()
                            },
                        ),
                        GridAxisKind::Row => (
                            NodeInputOf {
                                grid_column: GridPlacement::try_line(1)
                                    .expect("single lane column"),
                                grid_row: GridPlacement::try_line(2)
                                    .expect("second retained identity"),
                                ..NodeInputOf::default()
                            },
                            NodeInputOf {
                                grid_column: GridPlacement::try_line(1)
                                    .expect("single lane column"),
                                ..NodeInputOf::default()
                            },
                        ),
                    };
                    let tree = PublicLayoutTreeOf::new()
                        .children(1, [2, 3])
                        .style(
                            1,
                            NodeInputOf {
                                display: Display::GridLanes,
                                writing_mode,
                                direction,
                                size: physical_container.map(PreferredSizeOf::px),
                                grid_template_columns: columns,
                                grid_template_rows: rows,
                                grid_auto_flow: auto_flow,
                                grid_flow_tolerance: tolerance,
                                justify_content: Some(AlignContent::Center),
                                align_content: Some(AlignContent::Center),
                                ..NodeInputOf::default()
                            },
                        )
                        .style(2, explicit_style)
                        .style(3, automatic_style);
                    let batch = fri08_c03_auto_fit_batch(&tree, physical_container);
                    let explicit = fri08_c01_placement_output(&batch, 2);
                    let automatic = fri08_c01_placement_output(&batch, 3);
                    let explicit_logical = flow_axes.logical_point(
                        explicit.location,
                        explicit.size,
                        physical_container,
                    );
                    let automatic_logical = flow_axes.logical_point(
                        automatic.location,
                        automatic.size,
                        physical_container,
                    );
                    let automatic_size = flow_axes.logical_size(automatic.size);
                    match grid_axis {
                        GridAxisKind::Column => {
                            assert_eq!(explicit_logical.inline, scalar(60.0));
                            assert_eq!(automatic_logical.inline, scalar(20.0));
                            assert_eq!(automatic_size.inline, scalar(40.0));
                        }
                        GridAxisKind::Row => {
                            assert_eq!(flow_axes.logical_size(explicit.size).block, scalar(40.0));
                            assert_eq!(automatic_size.block, scalar(40.0));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn fri08_c03_auto_fit_automatic_placement_skips_collapsed_candidates_in_all_controls() {
    assert_fri08_c03_auto_fit_candidates_across_flows::<f32>();
    assert_fri08_c03_auto_fit_candidates_across_flows::<f64>();
}

#[test]
fn fri08_c04_standalone_nested_one_and_both_axis_inheritance_maps_all_flows_and_scalars() {
    assert_fri08_c04_standalone_nested_flows::<f32>();
    assert_fri08_c04_standalone_nested_flows::<f64>();
}

#[test]
fn fri08_c03_nested_candidate_bounds_edges_and_reversal_are_scalar_stable() {
    assert_fri08_c03_nested_candidate_bounds_edges_and_reversal::<f32>();
    assert_fri08_c03_nested_candidate_bounds_edges_and_reversal::<f64>();
}

fn fri08_c03_nested_projection_outputs<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
) -> [NodeOutputOf<S>; 4] {
    let batch = compute_layout(
        tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("nested max-content viewport"),
    )
    .expect("nested production projection succeeds");
    [
        fri08_c01_placement_output(&batch, 1),
        fri08_c01_placement_output(&batch, 6),
        fri08_c01_placement_output(&batch, 7),
        fri08_c01_placement_output(&batch, 8),
    ]
}

fn assert_fri08_c03_nested_recursive_edges_gaps_reversal<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for tolerance in [
        GridFlowToleranceOf::Length(LengthOf::<S>::ZERO),
        GridFlowToleranceOf::<S>::Infinite,
    ] {
        let ltr = fri08_c03_nested_projection_outputs(&fri08_c03_nested_projection_tree(
            Fri08C03NestedFlowCase {
                root_direction: Direction::Ltr,
                first_wrapper_mode: WritingMode::HorizontalTb,
                first_wrapper_direction: Direction::Ltr,
                second_wrapper_mode: WritingMode::HorizontalTb,
                second_wrapper_direction: Direction::Ltr,
                inherited_axis: GridAxisKind::Column,
            },
            tolerance,
            true,
        ));
        assert_eq!(
            [
                ltr[0].size.width,
                ltr[1].size.width,
                ltr[2].size.width,
                ltr[3].size.width,
            ],
            [scalar(156.0), scalar(56.0), S::ZERO, scalar(80.0)]
        );
        assert_eq!(ltr[1].location.x, S::ZERO);
        assert_eq!(ltr[3].location.x, scalar(76.0));

        let rtl = fri08_c03_nested_projection_outputs(&fri08_c03_nested_projection_tree(
            Fri08C03NestedFlowCase {
                root_direction: Direction::Rtl,
                first_wrapper_mode: WritingMode::HorizontalTb,
                first_wrapper_direction: Direction::Ltr,
                second_wrapper_mode: WritingMode::HorizontalTb,
                second_wrapper_direction: Direction::Ltr,
                inherited_axis: GridAxisKind::Column,
            },
            tolerance,
            true,
        ));
        assert_eq!(rtl[0].size.width, scalar(156.0));
        assert_eq!(rtl[1].size.width, scalar(80.0));
        assert_eq!(rtl[2].size.width, S::ZERO);
        assert_eq!(rtl[3].size.width, scalar(56.0));
        assert_eq!(rtl[1].location.x, scalar(76.0));
        assert_eq!(rtl[3].location.x, S::ZERO);
    }
}

#[test]
fn fri08_c03_nested_recursive_edges_gaps_reversal_and_order_are_scalar_stable() {
    assert_fri08_c03_nested_recursive_edges_gaps_reversal::<f32>();
    assert_fri08_c03_nested_recursive_edges_gaps_reversal::<f64>();
}

fn assert_fri08_c03_nested_vertical_sideways_row_axis_mapping<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let vertical = fri08_c03_nested_projection_outputs(&fri08_c03_nested_projection_tree(
        Fri08C03NestedFlowCase {
            root_direction: Direction::Ltr,
            first_wrapper_mode: WritingMode::VerticalRl,
            first_wrapper_direction: Direction::Ltr,
            second_wrapper_mode: WritingMode::VerticalRl,
            second_wrapper_direction: Direction::Ltr,
            inherited_axis: GridAxisKind::Row,
        },
        GridFlowToleranceOf::Length(LengthOf::<S>::ZERO),
        false,
    ));
    assert_eq!(
        [
            vertical[0].size.width,
            vertical[1].size.width,
            vertical[2].size.width,
            vertical[3].size.width,
        ],
        [scalar(60.0), scalar(40.0), S::ZERO, scalar(20.0)]
    );

    let sideways = fri08_c03_nested_projection_outputs(&fri08_c03_nested_projection_tree(
        Fri08C03NestedFlowCase {
            root_direction: Direction::Ltr,
            first_wrapper_mode: WritingMode::VerticalRl,
            first_wrapper_direction: Direction::Ltr,
            second_wrapper_mode: WritingMode::SidewaysLr,
            second_wrapper_direction: Direction::Ltr,
            inherited_axis: GridAxisKind::Row,
        },
        GridFlowToleranceOf::<S>::Infinite,
        false,
    ));
    assert_eq!(sideways[1].size.width, scalar(20.0));
    assert_eq!(sideways[2].size.width, S::ZERO);
    assert_eq!(sideways[3].size.width, scalar(40.0));
}

#[test]
fn fri08_c03_nested_vertical_and_sideways_flows_map_descendants_through_row_axis() {
    assert_fri08_c03_nested_vertical_sideways_row_axis_mapping::<f32>();
    assert_fri08_c03_nested_vertical_sideways_row_axis_mapping::<f64>();
}

fn fri08_c02_auto_fill_repeat<S: LayoutScalar>() -> TrackComponentOf<S> {
    TrackComponentOf::Repeat(
        TrackRepetitionOf::auto_fill_components(vec![TrackComponentOf::px(S::from_f64(40.0))])
            .expect("valid fixed auto-fill repetition"),
    )
}

#[test]
fn fri08_c02_auto_fit_spanning_item_keeps_every_covered_repetition_open() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_lines(2, 4).expect("second through third tracks"),
                grid_row: GridPlacement::try_line(1).expect("single row"),
                ..NodeInput::DEFAULT
            },
        );

    let output = fri08_c02_auto_fit_output(&tree, Size::new(120.0, 20.0), 2);
    assert_eq!(output.location.x, 20.0);
    assert_eq!(output.size.width, 80.0);
}

#[test]
fn fri08_c02_auto_fit_named_positive_and_negative_lines_keep_collapsed_identity() {
    let repeated = TrackComponent::Repeat(
        TrackRepetition::auto_fit_components(vec![
            TrackComponent::line_names(["slot"]),
            TrackComponent::px(40.0),
        ])
        .expect("valid named auto-fit repetition"),
    );
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![repeated],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "slot".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(-2).expect("third retained track"),
                ..NodeInput::DEFAULT
            },
        );

    let named = fri08_c02_auto_fit_output(&tree, Size::new(120.0, 20.0), 2);
    let negative = fri08_c02_auto_fit_output(&tree, Size::new(120.0, 20.0), 3);
    assert_eq!((named.location.x, named.size.width), (20.0, 40.0));
    assert_eq!((negative.location.x, negative.size.width), (60.0, 40.0));
}

#[test]
fn fri08_c02_auto_fit_auto_fill_remains_expanded_and_does_not_collapse() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c02_auto_fill_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("first repetition"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("overlap first repetition"),
                ..NodeInput::DEFAULT
            },
        );

    let output = fri08_c02_auto_fit_output(&tree, Size::new(120.0, 20.0), 2);
    assert_eq!((output.location.x, output.size.width), (0.0, 40.0));
}

fn assert_fri08_c01_placement_span_after_occupied_cell_adds_one_exact_row<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(PreferredSizeOf::px(scalar(120.0)), PreferredSizeOf::AUTO),
                grid_template_columns: vec![
                    TrackComponentOf::px(scalar(40.0)),
                    TrackComponentOf::px(scalar(40.0)),
                    TrackComponentOf::px(scalar(40.0)),
                ],
                grid_template_rows: vec![TrackComponentOf::px(scalar(20.0))],
                grid_auto_rows: vec![TrackComponentOf::px(scalar(20.0))],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                grid_column: GridPlacement::try_line(2).expect("middle column"),
                grid_row: GridPlacement::try_line(1).expect("first row"),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: GridPlacement::try_span(2).expect("two-column span"),
                ..NodeInputOf::default()
            },
        );

    let batch = fri08_c01_placement_compute(&tree);

    assert_eq!(
        fri08_c01_placement_output(&batch, 1).size,
        Size::new(scalar(120.0), scalar(40.0))
    );
    assert_eq!(
        fri08_c01_placement_output(&batch, 3).location,
        Point::new(scalar(0.0), scalar(20.0))
    );
    assert_eq!(
        fri08_c01_placement_output(&batch, 3).size,
        Size::new(scalar(80.0), scalar(20.0))
    );
}

#[test]
fn fri08_c01_placement_span_after_occupied_cell_adds_one_exact_row() {
    assert_fri08_c01_placement_span_after_occupied_cell_adds_one_exact_row::<f32>();
}

#[test]
fn fri08_c01_placement_definite_overlap_adds_no_implicit_row() {
    let definite = NodeInput {
        grid_column: GridPlacement::try_line(1).expect("first column"),
        grid_row: GridPlacement::try_line(1).expect("first row"),
        ..NodeInput::DEFAULT
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                grid_auto_rows: vec![TrackComponent::px(10.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(2, definite.clone())
        .style(3, definite);

    let batch = fri08_c01_placement_compute(&tree);

    assert_eq!(fri08_c01_placement_output(&batch, 1).size.height, 10.0);
    assert_eq!(fri08_c01_placement_output(&batch, 2).location, Point::ZERO);
    assert_eq!(fri08_c01_placement_output(&batch, 3).location, Point::ZERO);
}

#[test]
fn fri08_c01_placement_automatic_span_reserves_its_full_extent() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(90.0), PreferredSize::AUTO),
                grid_template_columns: vec![TrackComponent::px(30.0), TrackComponent::px(30.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                grid_auto_columns: vec![TrackComponent::px(30.0)],
                grid_auto_rows: vec![TrackComponent::px(10.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_span(3).expect("three-column span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::DEFAULT);

    let batch = fri08_c01_placement_compute(&tree);

    assert_eq!(
        fri08_c01_placement_output(&batch, 2).size,
        Size::new(90.0, 10.0)
    );
    assert_eq!(
        fri08_c01_placement_output(&batch, 3).location,
        Point::new(0.0, 10.0)
    );
}

fn fri08_c01_placement_dense_tree<S: LayoutScalar>(flow: GridAutoFlow) -> PublicLayoutTreeOf<S> {
    let scalar = S::from_f64;
    let (columns, rows, occupied_column, occupied_row, span_column, span_row) = if flow.is_column()
    {
        (
            vec![TrackComponentOf::px(scalar(10.0))],
            vec![
                TrackComponentOf::px(scalar(10.0)),
                TrackComponentOf::px(scalar(10.0)),
                TrackComponentOf::px(scalar(10.0)),
            ],
            GridPlacement::try_line(1).expect("first column"),
            GridPlacement::try_line(2).expect("middle row"),
            GridPlacement::AUTO,
            GridPlacement::try_span(2).expect("two-row span"),
        )
    } else {
        (
            vec![
                TrackComponentOf::px(scalar(10.0)),
                TrackComponentOf::px(scalar(10.0)),
                TrackComponentOf::px(scalar(10.0)),
            ],
            vec![TrackComponentOf::px(scalar(10.0))],
            GridPlacement::try_line(2).expect("middle column"),
            GridPlacement::try_line(1).expect("first row"),
            GridPlacement::try_span(2).expect("two-column span"),
            GridPlacement::AUTO,
        )
    };
    PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_auto_flow: flow,
                grid_template_columns: columns,
                grid_template_rows: rows,
                grid_auto_columns: vec![TrackComponentOf::px(scalar(10.0))],
                grid_auto_rows: vec![TrackComponentOf::px(scalar(10.0))],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                grid_column: occupied_column,
                grid_row: occupied_row,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: span_column,
                grid_row: span_row,
                ..NodeInputOf::default()
            },
        )
        .style(4, NodeInputOf::default())
}

fn assert_fri08_c01_placement_dense_backfills_but_sparse_cursors_remain_monotonic<
    S: LayoutScalar,
>() {
    let scalar = S::from_f64;
    for (sparse_flow, dense_flow, sparse_location) in [
        (
            GridAutoFlow::Row,
            GridAutoFlow::RowDense,
            Point::new(scalar(20.0), scalar(10.0)),
        ),
        (
            GridAutoFlow::Column,
            GridAutoFlow::ColumnDense,
            Point::new(scalar(10.0), scalar(20.0)),
        ),
    ] {
        let sparse = fri08_c01_placement_compute(&fri08_c01_placement_dense_tree::<S>(sparse_flow));
        let dense = fri08_c01_placement_compute(&fri08_c01_placement_dense_tree::<S>(dense_flow));
        assert_eq!(
            fri08_c01_placement_output(&sparse, 4).location,
            sparse_location
        );
        assert_eq!(
            fri08_c01_placement_output(&dense, 4).location,
            Point::new(scalar(0.0), scalar(0.0))
        );
    }
}

#[test]
fn fri08_c01_placement_dense_backfills_but_sparse_cursors_remain_monotonic() {
    assert_fri08_c01_placement_dense_backfills_but_sparse_cursors_remain_monotonic::<f32>();
}

#[test]
fn fri08_c01_placement_f64_covers_implicit_growth_and_sparse_dense_behavior() {
    assert_fri08_c01_placement_span_after_occupied_cell_adds_one_exact_row::<f64>();
    assert_fri08_c01_placement_dense_backfills_but_sparse_cursors_remain_monotonic::<f64>();
}

#[test]
fn fri08_c01_placement_row_sparse_advances_after_definite_column() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![
                    TrackComponent::px(10.0),
                    TrackComponent::px(10.0),
                    TrackComponent::px(10.0),
                ],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                grid_auto_rows: vec![TrackComponent::px(10.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT)
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(3).expect("third column"),
                ..NodeInput::DEFAULT
            },
        )
        .style(4, NodeInput::DEFAULT);

    let batch = fri08_c01_placement_compute(&tree);

    assert_eq!(
        fri08_c01_placement_output(&batch, 4).location,
        Point::new(0.0, 10.0)
    );
}

#[test]
fn fri08_c01_placement_column_sparse_advances_after_definite_row() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_auto_flow: GridAutoFlow::Column,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![
                    TrackComponent::px(10.0),
                    TrackComponent::px(10.0),
                    TrackComponent::px(10.0),
                ],
                grid_auto_columns: vec![TrackComponent::px(10.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT)
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_line(3).expect("third row"),
                ..NodeInput::DEFAULT
            },
        )
        .style(4, NodeInput::DEFAULT);

    let batch = fri08_c01_placement_compute(&tree);

    assert_eq!(
        fri08_c01_placement_output(&batch, 4).location,
        Point::new(10.0, 0.0)
    );
}

#[test]
fn fri08_c01_placement_leading_growth_preserves_line_translation_and_auto_pattern_phase() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                grid_auto_columns: vec![TrackComponent::px(10.0), TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(-3).expect("leading implicit line"),
                grid_row: GridPlacement::try_line(1).expect("first row"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("explicit first line"),
                grid_row: GridPlacement::try_line(1).expect("first row"),
                ..NodeInput::DEFAULT
            },
        );

    let batch = fri08_c01_placement_compute(&tree);

    assert_eq!(fri08_c01_placement_output(&batch, 2).size.width, 20.0);
    assert_eq!(fri08_c01_placement_output(&batch, 3).location.x, 20.0);
}

#[test]
fn fri08_c01_placement_automatic_span_capacity_is_axis_typed_without_allocation() {
    let mut topology = ExpandedGridTopology::from_test_parts(
        vec![TrackSizing::AUTO],
        vec![TrackSizing::AUTO],
        named::NamedGridLines::new(GridAxisKind::Column, 1),
        named::NamedGridLines::new(GridAxisKind::Row, 1),
        None,
    );
    let mut placements = GridPlacementContext::new(
        vec![2_u32],
        vec![ResolvedGridItemPlacement {
            column: GridPlacement::try_span(usize::MAX).expect("nonzero maximum span"),
            row: GridPlacement::AUTO,
            absolute_column: GridPlacement::AUTO,
            absolute_row: GridPlacement::AUTO,
            in_flow: true,
        }],
    );

    assert_eq!(
        derive_grid_placement_demand(&mut topology, &mut placements, GridAutoFlow::Row),
        Err(GridPlacementDemandError::AxisCapacity {
            axis: GridAxisKind::Column,
            requested_tracks: usize::MAX,
        })
    );
    assert_eq!(topology.column_tracks.len(), 1);
    assert!(placements.settled_areas.is_none());
}

#[test]
fn fri08_c01_placement_occupancy_product_capacity_is_typed_without_allocation() {
    assert_eq!(
        topology::GridOccupancy::new(usize::MAX, 2),
        Err(GridPlacementDemandError::OccupancyCapacity {
            columns: usize::MAX,
            rows: 2,
        })
    );
}

#[test]
fn fri05_c05_grid_geometry_publishes_canonical_ordinary_container_output() {
    for (overflow, replaced, expected_used) in [
        (Overflow::Visible, false, Overflow::Visible),
        (Overflow::Clip, false, Overflow::Clip),
        (Overflow::Hidden, false, Overflow::Hidden),
        (Overflow::Scroll, false, Overflow::Scroll),
        (Overflow::Auto, false, Overflow::Auto),
        (Overflow::Hidden, true, Overflow::Clip),
    ] {
        let mut tree = OracleTree::new().children(0, []).style(
            0,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                padding: Edges::all(Length::px(5.0)),
                border: Edges::all(Length::px(2.0)),
                overflow: computed_overflow(overflow, overflow),
                item_is_replaced: replaced,
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                scroll_margin: ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap(),
                scroll_snap_align: ScrollSnapAlign::new(
                    ScrollSnapAlignValue::End,
                    ScrollSnapAlignValue::Center,
                ),
                scroll_snap_stop: ScrollSnapStop::Always,
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::default()
            },
        );

        let output = compute_grid(
            &mut tree,
            0,
            fri05_c05_grid_sizing_input(Size::new(Some(100.0), Some(80.0))),
        )
        .expect("ordinary grid geometry computes");
        let geometry = output
            .scroll_geometry
            .expect("performed ordinary grid publishes canonical geometry");

        assert_eq!(geometry.border_box().size(), Size::new(100.0, 80.0));
        assert_eq!(geometry.used_overflow_x(), expected_used);
        assert_eq!(geometry.used_overflow_y(), expected_used);
        assert_eq!(geometry.target().border_box(), geometry.border_box());
        assert_eq!(geometry.target().flow_axes(), geometry.flow_axes());
        assert_eq!(
            geometry.target().scroll_margin(),
            ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap()
        );
        assert_eq!(
            geometry.target().snap_align(),
            ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center)
        );
        assert_eq!(geometry.target().snap_stop(), ScrollSnapStop::Always);
        assert_eq!(
            output.content_size,
            geometry.canonical_content_size().unwrap()
        );
        assert_eq!(
            geometry.content_box().size(),
            output.size - geometry.scrollbar_size() - Size::splat(14.0)
        );
    }
}

#[test]
fn fri05_c05_grid_geometry_reservations_are_effective_per_axis_and_saturate_tiny_boxes() {
    for (overflow, gutter, width, expected) in [
        (
            (Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::Auto,
            10.0,
            (false, false, false),
        ),
        (
            (Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::Stable,
            10.0,
            (false, true, false),
        ),
        (
            (Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::StableBothEdges,
            10.0,
            (true, true, false),
        ),
        (
            (Overflow::Scroll, Overflow::Hidden),
            ScrollbarGutter::Auto,
            10.0,
            (false, false, true),
        ),
        (
            (Overflow::Hidden, Overflow::Scroll),
            ScrollbarGutter::Auto,
            10.0,
            (false, true, false),
        ),
        (
            (Overflow::Scroll, Overflow::Scroll),
            ScrollbarGutter::Auto,
            0.0,
            (false, false, false),
        ),
    ] {
        let mut tree = OracleTree::new().children(0, []).style(
            0,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                overflow: computed_overflow(overflow.0, overflow.1),
                scrollbar_gutter: gutter,
                scrollbar_width: ScrollbarWidth::try_new(width).unwrap(),
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                ..NodeInput::default()
            },
        );
        let geometry = compute_grid(
            &mut tree,
            0,
            fri05_c05_grid_sizing_input(Size::new(Some(100.0), Some(80.0))),
        )
        .unwrap()
        .scroll_geometry
        .unwrap();
        assert_eq!(geometry.gutters().left().is_some(), expected.0);
        assert_eq!(geometry.gutters().right().is_some(), expected.1);
        assert_eq!(geometry.gutters().bottom().is_some(), expected.2);
    }

    let mut tiny = OracleTree::new().children(0, []).style(
        0,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(6.0)),
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(8.0).unwrap(),
            ..NodeInput::default()
        },
    );
    let geometry = compute_grid(
        &mut tiny,
        0,
        fri05_c05_grid_sizing_input(Size::new(Some(10.0), Some(6.0))),
    )
    .unwrap()
    .scroll_geometry
    .unwrap();
    assert_eq!(geometry.content_box().size(), Size::new(0.0, 6.0));
    assert_eq!(geometry.scrollbar_size(), Size::new(10.0, 0.0));
}

#[derive(Clone, Copy)]
struct Fri08C08T05FlowFacts {
    inline_axis: PhysicalAxis,
    inline_progression: PhysicalProgression,
    block_progression: PhysicalProgression,
}

fn fri08_c08_t05_flow_facts(
    writing_mode: WritingMode,
    direction: Direction,
) -> Fri08C08T05FlowFacts {
    use PhysicalAxis::{Horizontal, Vertical};
    use PhysicalProgression::{Decreasing, Increasing};

    match (writing_mode, direction) {
        (WritingMode::HorizontalTb, Direction::Ltr) => Fri08C08T05FlowFacts {
            inline_axis: Horizontal,
            inline_progression: Increasing,
            block_progression: Increasing,
        },
        (WritingMode::HorizontalTb, Direction::Rtl) => Fri08C08T05FlowFacts {
            inline_axis: Horizontal,
            inline_progression: Decreasing,
            block_progression: Increasing,
        },
        (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Ltr) => {
            Fri08C08T05FlowFacts {
                inline_axis: Vertical,
                inline_progression: Increasing,
                block_progression: Decreasing,
            }
        }
        (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Rtl) => {
            Fri08C08T05FlowFacts {
                inline_axis: Vertical,
                inline_progression: Decreasing,
                block_progression: Decreasing,
            }
        }
        (WritingMode::VerticalLr, Direction::Ltr) => Fri08C08T05FlowFacts {
            inline_axis: Vertical,
            inline_progression: Increasing,
            block_progression: Increasing,
        },
        (WritingMode::VerticalLr, Direction::Rtl) => Fri08C08T05FlowFacts {
            inline_axis: Vertical,
            inline_progression: Decreasing,
            block_progression: Increasing,
        },
        (WritingMode::SidewaysLr, Direction::Ltr) => Fri08C08T05FlowFacts {
            inline_axis: Vertical,
            inline_progression: Decreasing,
            block_progression: Increasing,
        },
        (WritingMode::SidewaysLr, Direction::Rtl) => Fri08C08T05FlowFacts {
            inline_axis: Vertical,
            inline_progression: Increasing,
            block_progression: Increasing,
        },
    }
}

fn assert_fri08_c08_t05_total_axis_mapping<S: LayoutScalar>() {
    let writing_modes = [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ];
    for parent_writing_mode in writing_modes {
        for parent_direction in [Direction::Ltr, Direction::Rtl] {
            let parent_facts = fri08_c08_t05_flow_facts(parent_writing_mode, parent_direction);
            let parent_style = NodeInputOf::<S> {
                display: Display::Grid,
                writing_mode: parent_writing_mode,
                direction: parent_direction,
                ..NodeInputOf::default()
            };
            for child_writing_mode in writing_modes {
                for child_direction in [Direction::Ltr, Direction::Rtl] {
                    let child_facts = fri08_c08_t05_flow_facts(child_writing_mode, child_direction);
                    let child_style = NodeInputOf::<S> {
                        display: Display::Grid,
                        writing_mode: child_writing_mode,
                        direction: child_direction,
                        grid_template_columns: subgrid_track_of(),
                        grid_template_rows: subgrid_track_of(),
                        ..NodeInputOf::default()
                    };
                    for queried_axis in [GridAxisKind::Column, GridAxisKind::Row] {
                        let child_physical_axis = match queried_axis {
                            GridAxisKind::Column => child_facts.inline_axis,
                            GridAxisKind::Row => child_facts.inline_axis.other(),
                        };
                        let parent_axis = if parent_facts.inline_axis == child_physical_axis {
                            GridAxisKind::Column
                        } else {
                            GridAxisKind::Row
                        };
                        let parent_progression = match parent_axis {
                            GridAxisKind::Column => parent_facts.inline_progression,
                            GridAxisKind::Row => parent_facts.block_progression,
                        };
                        let child_progression = match queried_axis {
                            GridAxisKind::Column => child_facts.inline_progression,
                            GridAxisKind::Row => child_facts.block_progression,
                        };
                        let report = map_grid_axis(GridAxisMappingInput {
                            queried_axis,
                            parent_style: &parent_style,
                            child_style: &child_style,
                        });
                        let axis_report =
                            subgrid_axis_report(&parent_style, &child_style, queried_axis);

                        assert_eq!(
                            report,
                            GridAxisMappingReport {
                                queried_axis,
                                parent_axis,
                                child_axis: queried_axis,
                                reversed: parent_progression != child_progression,
                            },
                            "{parent_writing_mode:?} {parent_direction:?} to {child_writing_mode:?} {child_direction:?} {queried_axis:?}"
                        );
                        assert_eq!(axis_report.mapping, report);
                        assert!(axis_report.can_inherit());
                        assert_eq!(
                            inherited_subgrid_physical_axis(
                                axis_report,
                                FlowAxes::new(parent_writing_mode, parent_direction),
                                FlowAxes::new(child_writing_mode, child_direction),
                            ),
                            Some(child_physical_axis)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn fri08_c08_t05_impossible_state_all_flow_axis_mappings_and_scalar_callers_are_total() {
    assert_fri08_c08_t05_total_axis_mapping::<f32>();
    assert_fri08_c08_t05_total_axis_mapping::<f64>();
}

fn fri08_c08_t05_subgrid_reason(
    parent_style: &NodeInput,
    has_parent_grid: bool,
    child_style: &NodeInput,
    axis: GridAxisKind,
) -> Option<SubgridIneligibleReason> {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis,
        parent_style,
        has_parent_grid,
        child_style,
    });
    assert_eq!(report.eligible, report.reason.is_none());
    report.reason
}

#[test]
fn fri08_c08_t05_impossible_state_reachable_eligibility_combinations_keep_exact_reasons() {
    let ordinary_parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let lanes_parent = NodeInput {
        display: Display::GridLanes,
        grid_auto_flow: GridAutoFlow::Row,
        ..NodeInput::default()
    };
    let requested = NodeInput {
        display: Display::Grid,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };
    let not_requested = NodeInput {
        display: Display::Block,
        position: Position::Absolute,
        ..NodeInput::default()
    };
    assert_eq!(
        fri08_c08_t05_subgrid_reason(
            &ordinary_parent,
            false,
            &not_requested,
            GridAxisKind::Column,
        ),
        Some(SubgridIneligibleReason::NotRequested)
    );

    let no_parent = NodeInput {
        position: Position::Absolute,
        ..requested.clone()
    };
    assert_eq!(
        fri08_c08_t05_subgrid_reason(&ordinary_parent, false, &no_parent, GridAxisKind::Column,),
        Some(SubgridIneligibleReason::NoParentGrid)
    );

    let excluded = NodeInput {
        display: Display::Block,
        position: Position::Absolute,
        ..requested.clone()
    };
    assert_eq!(
        fri08_c08_t05_subgrid_reason(&lanes_parent, true, &excluded, GridAxisKind::Column),
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    );

    let unsupported = NodeInput {
        display: Display::Block,
        ..requested.clone()
    };
    assert_eq!(
        fri08_c08_t05_subgrid_reason(&lanes_parent, true, &unsupported, GridAxisKind::Column),
        Some(SubgridIneligibleReason::UnsupportedDisplay)
    );
    assert_eq!(
        fri08_c08_t05_subgrid_reason(&lanes_parent, true, &requested, GridAxisKind::Row),
        Some(SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    );
    assert_eq!(
        fri08_c08_t05_subgrid_reason(&ordinary_parent, true, &requested, GridAxisKind::Column,),
        None
    );
    assert_eq!(
        fri08_c08_t05_subgrid_reason(&lanes_parent, true, &requested, GridAxisKind::Column),
        None
    );

    let scroll_container = NodeInput {
        overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Auto).unwrap(),
        ..requested
    };
    assert_eq!(
        fri08_c08_t05_subgrid_reason(
            &ordinary_parent,
            true,
            &scroll_container,
            GridAxisKind::Column,
        ),
        None
    );
}

#[test]
fn fri05_c05_grid_auto_covers_none_single_axis_and_both_induction_orders() {
    for (tracks, expected_gutters, expected_passes) in [
        (Size::new(80.0, 80.0), (false, false), 1),
        (Size::new(120.0, 80.0), (true, false), 2),
        (Size::new(80.0, 120.0), (false, true), 2),
        (Size::new(120.0, 95.0), (true, true), 3),
        (Size::new(95.0, 120.0), (true, true), 3),
    ] {
        let mut tree = OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                    grid_template_columns: vec![TrackComponent::px(tracks.width)],
                    grid_template_rows: vec![TrackComponent::px(tracks.height)],
                    ..NodeInput::default()
                },
            )
            .style(1, NodeInput::default());
        let geometry = compute_grid(
            &mut tree,
            0,
            fri05_c05_grid_sizing_input(Size::splat(Some(100.0))),
        )
        .unwrap()
        .scroll_geometry
        .unwrap();
        assert_eq!(geometry.gutters().bottom().is_some(), expected_gutters.0);
        assert_eq!(geometry.gutters().right().is_some(), expected_gutters.1);
        assert_eq!(
            tree.inputs(1)
                .iter()
                .filter(|input| input.run_mode() == RunMode::PerformLayout)
                .count(),
            expected_passes
        );
    }
}

#[test]
fn fri04_c04_grid_dispatch_container_reports_exact_grid_capability() {
    let mut tree = OracleTree::new().children(0, []).style(
        0,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::STRETCH, PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );

    let error = compute_grid(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(100.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::definite(100.0)),
        ),
    )
    .expect_err("later-owned grid sizing must be rejected");

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected exact sizing capability, got {:?}", error.kind());
    };
    assert_eq!(unsupported.property(), SizingProperty::Preferred);
    assert_eq!(unsupported.behavior(), SizingBehavior::Stretch);
    assert_eq!(unsupported.algorithm(), SizingAlgorithm::Grid);
    assert_eq!(unsupported.axis(), PhysicalAxis::Horizontal);
}

fn assert_grid_authoritative_known_width_ignores_min_max<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let known_width = S::from_f64(20.0);
    let containing_width = S::from_f64(100.0);
    let known_size = Size::splat(Some(known_width));

    for (min_width, max_width) in [
        (Some(S::from_f64(40.0)), None),
        (None, Some(S::from_f64(10.0))),
    ] {
        let mut tree = OracleTreeOf::<S>::new().children(1, []).style(
            1,
            NodeInputOf {
                display: Display::Grid,
                min_size: Size::new(
                    min_width.map(MinSizeOf::px).unwrap_or(MinSizeOf::AUTO),
                    MinSizeOf::AUTO,
                ),
                max_size: Size::new(
                    max_width.map(MaxSizeOf::px).unwrap_or(MaxSizeOf::NONE),
                    MaxSizeOf::NONE,
                ),
                ..NodeInputOf::default()
            },
        );

        let output = compute_grid(
            &mut tree,
            1,
            ComputeInputOf::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                known_size,
                Size::splat(Some(containing_width)),
                crate::ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::new(
                    AvailableOf::Definite(containing_width),
                    AvailableOf::Definite(containing_width),
                ),
            ),
        )
        .expect("known grid size layout succeeds");

        assert_eq!(output.size.width, known_width);
    }
}

#[test]
fn grid_authoritative_known_width_ignores_min_max_f32() {
    assert_grid_authoritative_known_width_ignores_min_max::<f32>();
}

#[test]
fn grid_authoritative_known_width_ignores_min_max_f64() {
    assert_grid_authoritative_known_width_ignores_min_max::<f64>();
}

#[test]
fn fri06_c12_t08_row_and_column_groups_choose_the_lastmost_member() {
    let member = |source, axis, last_baseline| {
        let (child_flow_axes, containing_flow_axes) = match axis {
            GridAxisKind::Column => (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ),
            GridAxisKind::Row => (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ),
        };
        ancestor_baseline_member(AncestorBaselineMemberInput {
            source,
            axis,
            ancestor_span: GridTrackSpan::new(1, 2),
            alignment: AlignItems::LastBaseline,
            block_auto_margins: false,
            synthesized_baseline_cycle: false,
            output: ComputeOutput::from_sizes_and_baselines(
                match axis {
                    GridAxisKind::Column => Size::new(50.0, 30.0),
                    GridAxisKind::Row => Size::new(30.0, 50.0),
                },
                match axis {
                    GridAxisKind::Column => Size::new(50.0, 30.0),
                    GridAxisKind::Row => Size::new(30.0, 50.0),
                },
                Baselines {
                    first: Point::NONE,
                    last: match axis {
                        GridAxisKind::Column => Point::new(Some(last_baseline), None),
                        GridAxisKind::Row => Point::new(None, Some(last_baseline)),
                    },
                },
            ),
            margin: Edges::all(0.0),
            child_flow_axes,
            containing_flow_axes,
            start_adjustment: 0.0,
            end_adjustment: 0.0,
        })
        .expect("last-baseline item participates")
    };
    for (axis, physical_axis) in [
        (GridAxisKind::Row, PhysicalAxis::Vertical),
        (GridAxisKind::Column, PhysicalAxis::Horizontal),
    ] {
        let direct = member(1_u32, axis, 25.0);
        let nested = member(2_u32, axis, 10.0);
        let group = AncestorBaselineGroup::reduce(1_u32, axis, physical_axis, 1, [direct, nested]);
        assert_eq!(group.target_for(nested), Some(40.0));
        assert_eq!(group.intrinsic_shim(direct).after, 15.0);
    }
}

#[test]
fn owner_to_current_placement_map_identity_has_zero_translation() {
    let map = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        3,
    );

    assert_eq!(map.owner(), 1);
    assert_eq!(map.current_grid(), 1);
    assert_eq!(map.boundary_count(), 0);
    assert_eq!(map.track_count(), 3);
    for local in 0..3 {
        assert_eq!(map.owner_track_for_local(local), Some(local));
        assert_eq!(
            map.translations_for(local, AncestorBaselineRole::First),
            Some((0.0, 0.0)),
        );
        assert_eq!(
            map.translations_for(local, AncestorBaselineRole::Last),
            Some((0.0, 0.0)),
        );
    }
}

#[test]
fn owner_to_current_placement_map_composes_reversal_and_physical_progression() {
    let identity = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        3,
    );
    let map = identity
        .compose(owner_placement_boundary!(
            1,
            2,
            GridTrackSpan::new(0, 3),
            true,
            PhysicalProgression::Increasing,
            PhysicalProgression::Decreasing,
            &[0.0, 40.0, 90.0],
            &[30.0, 80.0, 150.0],
            &[5.0, 45.0, 95.0],
            &[35.0, 85.0, 155.0],
            10.0,
            20.0,
            0.0,
            0.0,
        ))
        .unwrap();

    assert_eq!(map.owner_track_for_local(0), Some(2));
    assert_eq!(map.owner_track_for_local(2), Some(0));
    assert_eq!(map.current_progression(), PhysicalProgression::Decreasing);
    assert_eq!(
        map.translations_for(1, AncestorBaselineRole::First),
        Some((5.0, -5.0)),
    );
    assert_eq!(
        map.translations_for(1, AncestorBaselineRole::Last),
        Some((5.0, 5.0)),
    );
}

#[test]
fn owner_to_current_placement_map_accumulates_positive_zero_and_negative_half_gaps() {
    let expected = [(20.0, 5.0), (10.0, 0.0), (0.0, -5.0)];
    for (current_gap, half_gap) in expected {
        let identity = CheckedOwnerToCurrentPlacementMap::identity(
            1_u32,
            GridAxisKind::Row,
            PhysicalAxis::Vertical,
            PhysicalProgression::Increasing,
            3,
        );
        let map = identity
            .compose(owner_placement_boundary!(
                1,
                2,
                GridTrackSpan::new(0, 3),
                false,
                PhysicalProgression::Increasing,
                PhysicalProgression::Increasing,
                &[0.0, 40.0, 90.0],
                &[30.0, 80.0, 150.0],
                &[0.0, 40.0, 90.0],
                &[30.0, 80.0, 150.0],
                10.0,
                current_gap,
                0.0,
                0.0,
            ))
            .unwrap();
        assert_eq!(
            map.translations_for(1, AncestorBaselineRole::First),
            Some((0.0, half_gap)),
        );
        assert_eq!(
            map.translations_for(1, AncestorBaselineRole::Last),
            Some((0.0, -half_gap)),
        );
    }
}

#[test]
fn owner_to_current_placement_map_rejects_identity_discontinuity() {
    let identity = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        2,
    );
    let result = identity.compose(owner_placement_boundary!(
        9,
        2,
        GridTrackSpan::new(0, 2),
        false,
        PhysicalProgression::Increasing,
        PhysicalProgression::Increasing,
        &[0.0, 40.0],
        &[30.0, 80.0],
        &[0.0, 40.0],
        &[30.0, 80.0],
        10.0,
        10.0,
        0.0,
        0.0,
    ));
    assert_eq!(
        result,
        Err(InheritedCurrentGridBaselinePlacementError::OwnershipMismatch),
    );
}

#[test]
fn fri06_c12_t08_inline_column_direct_members_consume_column_group() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(1).expect("valid first row"),
                justify_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(2).expect("valid second row"),
                justify_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, vertical_baseline_measure(30.0, 20.0, Some(25.0), None))
        .measure(3, vertical_baseline_measure(50.0, 20.0, Some(10.0), None));

    compute_oracle_grid(&mut tree);

    let first = tree.final_layout(2).expect("first direct item is laid out");
    let second = tree
        .final_layout(3)
        .expect("second direct item is laid out");
    assert_eq!(second.location.x, 15.0);
    assert_eq!(first.location.x + 25.0, second.location.x + 10.0);
}

#[test]
fn fri06_c12_t08_vertical_nested_direct_members_use_ancestor_half_gap() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(130.0), PreferredSize::px(80.0)),
                grid_template_rows: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(2).expect("valid second row"),
                grid_column: GridPlacement::try_line(2).expect("valid second column"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_lines(1, 3).expect("valid inherited row span"),
                grid_column: GridPlacement::try_line(1).expect("valid first column"),
                grid_template_rows: vec![empty_subgrid_track()],
                grid_template_columns: vec![TrackComponent::px(40.0)],
                gap: Size::new(Length::px(20.0), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(2).expect("valid inherited second row"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, vertical_baseline_measure(30.0, 20.0, Some(20.0), None))
        .measure(4, vertical_baseline_measure(30.0, 20.0, Some(5.0), None));

    compute_oracle_grid(&mut tree);

    let direct = tree.final_layout(2).expect("direct member is laid out");
    let nested = tree.final_layout(4).expect("flattened member is laid out");
    assert_eq!((direct.location.x, nested.location.x), (5.0, 25.0));
}

fn fri06_c12_t08_refreshed_cross_flow_tree(
    direction: Direction,
    child_writing_mode: WritingMode,
    container_height: f32,
    descendant_height: f32,
) -> OracleTree {
    let mut tree = OracleTree::new()
        .children(1, [2, 4])
        .children(2, [3])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                direction,
                size: Size::new(
                    PreferredSize::px(200.0),
                    PreferredSize::px(container_height),
                ),
                grid_template_columns: vec![TrackComponent::px(100.0), TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::px(container_height)],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                writing_mode: child_writing_mode,
                direction,
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Stretch),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .measure(
            3,
            baseline_measure(20.0, descendant_height, Some(7.0), None),
        )
        .measure(4, baseline_measure(20.0, 10.0, Some(7.0), None));

    compute_oracle_grid(&mut tree);
    tree
}

#[test]
fn fri06_c12_t08_refreshed_vertical_rl_item_uses_horizontal_rtl_grid_coordinates_once() {
    let tree = fri06_c12_t08_refreshed_cross_flow_tree(
        Direction::Rtl,
        WritingMode::VerticalRl,
        80.0,
        10.0,
    );

    assert_eq!(
        tree.final_layout(2)
            .expect("refreshed item is laid out")
            .size,
        Size::new(20.0, 80.0),
        "the containing grid axes own refreshed area sizing",
    );
    assert_eq!(
        tree.final_layout(2)
            .expect("refreshed item is laid out")
            .location,
        Point::new(180.0, 0.0),
        "horizontal RTL projects the refreshed logical item once",
    );
    assert_eq!(
        tree.final_layout(4)
            .expect("ordinary sibling is laid out")
            .location,
        Point::new(80.0, 0.0),
        "the ordinary non-refreshed sibling remains in its RTL grid area",
    );
    assert_eq!(
        tree.final_layout(3)
            .expect("nested descendant is laid out")
            .location,
        Point::new(0.0, 70.0),
        "vertical-rl child-internal projection remains child-local after refresh",
    );
    assert_eq!(
        tree.final_layout(1).expect("root is laid out").size,
        Size::new(200.0, 80.0),
        "root accumulation retains the containing-grid dimensions",
    );
}

#[test]
fn fri06_c12_t08_refreshed_same_flow_ltr_and_ordinary_item_remain_unchanged() {
    let tree = fri06_c12_t08_refreshed_cross_flow_tree(
        Direction::Ltr,
        WritingMode::HorizontalTb,
        80.0,
        10.0,
    );

    assert_eq!(
        tree.final_layout(2)
            .expect("refreshed item is laid out")
            .size,
        Size::new(100.0, 80.0),
    );
    assert_eq!(
        tree.final_layout(2)
            .expect("refreshed item is laid out")
            .location,
        Point::new(0.0, 0.0),
    );
    assert_eq!(
        tree.final_layout(3)
            .expect("nested descendant is laid out")
            .location,
        Point::new(0.0, 0.0),
    );
    assert_eq!(
        tree.final_layout(4)
            .expect("ordinary sibling is laid out")
            .location,
        Point::new(100.0, 0.0),
    );
}

#[test]
fn fri06_c12_t08_refreshed_vertical_cross_writing_offset_is_projected_once() {
    let tree = fri06_c12_t08_refreshed_cross_flow_tree(
        Direction::Rtl,
        WritingMode::VerticalRl,
        120.0,
        24.0,
    );

    assert_eq!(
        tree.final_layout(2)
            .expect("refreshed cross-writing item is laid out")
            .size,
        Size::new(20.0, 120.0),
    );
    assert_eq!(
        tree.final_layout(3)
            .expect("cross-writing descendant is laid out")
            .location,
        Point::new(0.0, 96.0),
        "child-internal vertical RTL projection consumes the refreshed area once",
    );
}

#[test]
fn grid_auto_placement_continues_into_declared_rows_with_gap() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(205.0), PreferredSize::px(75.0)),
            grid_template_columns: vec![TrackComponent::px(100.0), TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(30.0), TrackComponent::px(40.0)],
            gap: Size::new(Length::px(5.0), Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());
    tree.insert_style(4, NodeInput::default());

    let output = crate::compute_grid(
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

    assert_eq!(output.content_size, Size::new(205.0, 75.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(100.0, 30.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(105.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(100.0, 30.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(0.0, 35.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(100.0, 40.0)
    );
}

#[test]
fn named_grid_in_flow_item_occupies_cell_before_auto_sibling() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["taken"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "taken".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::default());

    compute_oracle_grid(&mut tree);
    let named = tree
        .final_layout(2)
        .expect("named child should be laid out");
    let auto = tree.final_layout(3).expect("auto child should be laid out");

    assert_eq!(named.location, Point::new(0.0, 0.0));
    assert_eq!(named.size, Size::new(40.0, 20.0));
    assert_eq!(auto.location, Point::new(40.0, 0.0));
    assert_eq!(auto.size, Size::new(40.0, 20.0));
}

#[test]
fn grid_auto_placement_creates_implicit_rows_from_auto_rows() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::px(80.0), TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(30.0)],
            grid_auto_rows: vec![TrackComponent::px(40.0)],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());
    tree.insert_style(4, NodeInput::default());

    let output = crate::compute_grid(
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

    assert_eq!(output.size, Size::new(200.0, 75.0));
    assert_eq!(output.content_size, Size::new(200.0, 75.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 30.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(120.0, 30.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(0.0, 35.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(80.0, 40.0)
    );
}

#[test]
fn grid_auto_rows_repeat_for_multiple_implicit_rows() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4, 5]);
    for node in 2..=5 {
        tree.insert_children(node, vec![]);
        tree.insert_style(node, NodeInput::default());
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(50.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::px(50.0)],
            grid_auto_rows: vec![TrackComponent::px(10.0), TrackComponent::px(20.0)],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_grid(
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

    assert_eq!(output.content_size, Size::new(50.0, 75.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(50.0, 10.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 15.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(50.0, 20.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(0.0, 40.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(50.0, 10.0)
    );
    assert_eq!(
        tree.layout(5).expect("node layout is staged").location,
        Point::new(0.0, 55.0)
    );
    assert_eq!(
        tree.layout(5).expect("node layout is staged").size,
        Size::new(50.0, 20.0)
    );
}

#[test]
fn grid_compute_size_applies_aspect_ratio_to_max_size() {
    #[derive(Default)]
    struct NoChildMeasurementTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for NoChildMeasurementTree {
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

    impl Compute for NoChildMeasurementTree {
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
            panic!("definite grid compute-size should not measure children")
        }
    }

    let mut tree = NoChildMeasurementTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            max_size: Size::new(MaxSize::px(50.0), MaxSize::NONE),
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_grid(
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

    assert_eq!(output.size, Size::new(50.0, 25.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn grid_content_size_mode_ignores_authored_size() {
    #[derive(Default)]
    struct NoChildMeasurementTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for NoChildMeasurementTree {
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

    impl Compute for NoChildMeasurementTree {
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
            panic!("empty grid content-size should not measure children")
        }
    }

    let mut tree = NoChildMeasurementTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(30.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );

    let output = crate::compute_grid(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::ContentSize,
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

    assert_eq!(output.size, Size::new(30.0, 20.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn grid_dense_auto_flow_backfills_earlier_free_cells() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(90.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(30.0),
                TrackComponent::px(30.0),
                TrackComponent::px(30.0),
            ],
            grid_template_rows: vec![TrackComponent::px(10.0), TrackComponent::px(10.0)],
            grid_auto_flow: GridAutoFlow::RowDense,
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            grid_column: GridPlacement::try_span(2).expect("valid grid span"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(4, NodeInput::default());

    crate::compute_grid(
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

    let columns = DefiniteTracks::new(90.0, 0.0)
        .track(Track::px(30.0))
        .track(Track::px(30.0))
        .track(Track::px(30.0))
        .solve();
    let rows = DefiniteTracks::new(20.0, 0.0)
        .track(Track::px(10.0))
        .track(Track::px(10.0))
        .solve();
    let mut placement = AutoPlacer::try_new(3, 2, Flow::RowDense)
        .unwrap()
        .occupied(OracleGridArea::new(2, 1, 1, 1));
    let third_area = placement.place(2, 1).unwrap();
    let fourth_area = placement.place(1, 1).unwrap();
    let second_columns = columns.area(2, 3);
    let third_columns = columns.area(
        third_area.column_start,
        third_area.column_start + third_area.column_span,
    );
    let third_rows = rows.area(
        third_area.row_start,
        third_area.row_start + third_area.row_span,
    );
    let fourth_columns = columns.area(
        fourth_area.column_start,
        fourth_area.column_start + fourth_area.column_span,
    );
    let fourth_rows = rows.area(
        fourth_area.row_start,
        fourth_area.row_start + fourth_area.row_span,
    );

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(second_columns.start, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(third_columns.start, third_rows.start)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(third_columns.size, third_rows.size)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(fourth_columns.start, fourth_rows.start)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(fourth_columns.size, fourth_rows.size)
    );
}

#[test]
fn grid_dense_row_flow_places_definite_row_items_before_auto_items() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, 2..=10);
    for node in 2..=10 {
        tree.insert_children(node, vec![]);
        tree.insert_style(node, NodeInput::default());
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(120.0)),
            grid_auto_flow: GridAutoFlow::RowDense,
            grid_template_columns: vec![
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
            ],
            grid_template_rows: vec![
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
            ],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        4,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            size: Size::new(PreferredSize::px(35.0), PreferredSize::px(35.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        7,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        9,
        NodeInput {
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(120.0), Some(120.0)),
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
        tree.layout(2).expect("node layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 40.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(0.0, 80.0)
    );
    assert_eq!(
        tree.layout(7).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(9).expect("node layout is staged").location,
        Point::new(40.0, 0.0)
    );
}

#[test]
fn grid_definite_column_auto_row_stays_in_auto_placement_order() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(
        3,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 20.0)
    );
}

#[test]
fn grid_definite_column_line_span_resolves_from_start_line() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(150.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(30.0),
                TrackComponent::px(40.0),
                TrackComponent::px(50.0),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(5.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line_span(2, 2).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
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

    let columns = DefiniteTracks::new(150.0, 5.0)
        .track(Track::px(30.0))
        .track(Track::px(40.0))
        .track(Track::px(50.0))
        .solve();
    let column_area = LinePlacement::LineSpan { start: 2, span: 2 }
        .resolve_axis(1)
        .unwrap();
    let expected_column_area = columns.area(
        column_area.start_line as usize,
        column_area.end_line as usize,
    );

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(expected_column_area.start, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(expected_column_area.size, 20.0)
    );
}

#[test]
fn grid_definite_column_span_line_resolves_to_end_line() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(150.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(30.0),
                TrackComponent::px(40.0),
                TrackComponent::px(50.0),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(5.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_span_line(2, 4).expect("valid grid span line"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
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

    let columns = DefiniteTracks::new(150.0, 5.0)
        .track(Track::px(30.0))
        .track(Track::px(40.0))
        .track(Track::px(50.0))
        .solve();
    let column_area = LinePlacement::SpanLine { span: 2, end: 4 }
        .resolve_axis(1)
        .unwrap();
    let expected_column_area = columns.area(
        column_area.start_line as usize,
        column_area.end_line as usize,
    );

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(expected_column_area.start, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(expected_column_area.size, 20.0)
    );
}

#[test]
fn grid_row_span_auto_placement_creates_enough_implicit_rows() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(50.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::px(50.0)],
            grid_auto_rows: vec![
                TrackComponent::px(10.0),
                TrackComponent::px(20.0),
                TrackComponent::px(30.0),
            ],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_row: GridPlacement::try_span(3).expect("valid grid span"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(50.0, 70.0)
    );
}

#[test]
fn grid_definite_column_line_creates_required_implicit_columns() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(10.0)),
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            grid_auto_columns: vec![TrackComponent::px(30.0), TrackComponent::px(40.0)],
            gap: Size::new(Length::px(5.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_lines(3, 4).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(60.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(40.0, 10.0)
    );
}

#[test]
fn vertical_rl_grid_places_distinct_rows_on_physical_x_axis() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_template_columns: vec![TrackComponent::px(30.0), TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(50.0), TrackComponent::px(60.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT)
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        );

    let output = crate::compute_grid(
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

    assert_eq!(output.size, Size::new(110.0, 70.0));
    assert_eq!(tree.layout(2).unwrap().location, Point::new(60.0, 0.0));
    assert_eq!(tree.layout(3).unwrap().location, Point::new(0.0, 30.0));
}

#[test]
fn grid_recomputes_min_content_columns_from_resolved_row_height() {
    let mut tree = OracleTree::new()
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(40.0, 40.0)))
                .known(Size::new(None, Some(40.0))),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(40.0, 40.0)))
                .known(Size::new(Some(40.0), Some(40.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 40.0)))
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(40.0, 20.0)))
                .known(Size::new(Some(40.0), Some(20.0))),
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(20.0, 20.0)));
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::MIN_CONTENT],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            writing_mode: WritingMode::VerticalLr,
            ..NodeInput::default()
        },
    );
    tree.insert_style(3, NodeInput::default());

    let output = crate::compute_grid(
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

    assert_eq!(output.size, Size::new(40.0, 60.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(40.0, 40.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 40.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(40.0, 20.0)
    );
}

#[test]
fn grid_spanning_item_redistributes_beyond_fit_content_limit() {
    let mut tree = OracleTree::new()
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(40.0, 40.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT)),
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(80.0, 40.0)));
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::Track(crate::TrackSizing {
                    min: MinTrackSizing::Auto,
                    max: MaxTrackSizing::MaxContent,
                }),
                TrackComponent::Track(crate::TrackSizing {
                    min: MinTrackSizing::Auto,
                    max: MaxTrackSizing::FitContent(SizingCalculation::value(lp(10.0, 0.0))),
                }),
            ],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        4,
        NodeInput {
            grid_column: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_grid(
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

    assert_eq!(output.size, Size::new(80.0, 40.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(60.0, 40.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(60.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(20.0, 40.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(80.0, 40.0)
    );
}

#[test]
fn grid_content_size_for_later_column_uses_final_container_local_item_end() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::px(50.0), TrackComponent::px(50.0)],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 10.0), Size::new(80.0, 10.0)),
    );

    let output = crate::compute_grid(
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(50.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(50.0, 10.0)
    );
    assert_eq!(output.content_size, Size::new(130.0, 10.0));
}

#[test]
fn grid_justify_self_overrides_parent_justify_items() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            justify_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            justify_self: Some(AlignItems::End),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(30.0, 10.0)));

    crate::compute_grid(
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(50.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 10.0)
    );
}

#[test]
fn named_grid_column_places_item_between_repeated_named_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "a".to_string(),
                        index: 2,
                    },
                    RawGridLine::NamedSpan {
                        name: "a".to_string(),
                        index: 1,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 40.0);
    assert_eq!(child.size.width, 40.0);
}

#[test]
fn named_grid_template_area_bare_name_uses_generated_start_and_end_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            Some("foo".to_string()),
                            Some("foo".to_string()),
                            Some("bar".to_string()),
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("foo".to_string()),
                    RawGridLine::BareIdent("foo".to_string()),
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn fri08_c01_topology_populated_area_only_grid_uses_the_same_explicit_edges() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_auto_columns: vec![TrackComponent::px(40.0)],
                grid_auto_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            Some("main".to_string()),
                            Some("main".to_string()),
                            Some("main".to_string()),
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                raw_grid_row: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(120.0), Available::Definite(20.0)),
    )
    .unwrap();
    round_layout(&mut tree, 1).unwrap();

    let root = tree.final_layout(1).expect("area-only root layout");
    let child = tree.final_layout(2).expect("area-named child layout");
    assert_eq!(root.size, Size::new(120.0, 20.0));
    assert_eq!(child.location, Point::ZERO);
    assert_eq!(child.size, Size::new(120.0, 20.0));
}

#[test]
fn fri08_c01_topology_uses_larger_area_or_sized_list_dimension_without_losing_names() {
    let style = NodeInput {
        display: Display::Grid,
        grid_template_columns: vec![
            TrackComponent::line_names(["column-authored-start"]),
            TrackComponent::px(30.0),
        ],
        grid_template_rows: vec![
            TrackComponent::line_names(["row-authored-start"]),
            TrackComponent::px(5.0),
            TrackComponent::px(7.0),
            TrackComponent::px(9.0),
            TrackComponent::line_names(["row-authored-end"]),
        ],
        grid_auto_columns: vec![TrackComponent::px(10.0), TrackComponent::px(20.0)],
        grid_auto_rows: vec![TrackComponent::px(11.0)],
        grid_template_areas: GridTemplateAreas {
            rows: vec![GridTemplateAreaRow {
                cells: vec![
                    Some("main".to_string()),
                    Some("main".to_string()),
                    Some("main".to_string()),
                ],
            }],
        },
        ..NodeInput::DEFAULT
    };
    let named = named::build_grid_named_context(
        &grid_container_projection!(&style),
        1,
        3,
        &GridParentContext::none(),
    )
    .expect("valid mixed topology inputs");
    assert_eq!(named.columns.explicit_track_count, 3);
    assert_eq!(named.rows.explicit_track_count, 3);
    assert_eq!(
        named.columns.named_occurrences("column-authored-start"),
        vec![1]
    );
    assert_eq!(named.columns.named_occurrences("main-end"), vec![4]);
    assert_eq!(named.rows.named_occurrences("row-authored-start"), vec![1]);
    assert_eq!(named.rows.named_occurrences("row-authored-end"), vec![4]);
    assert_eq!(named.rows.named_occurrences("main-end"), vec![2]);

    let mut tree = OracleTree::new().children(1, []).style(1, style);
    let output = compute_oracle_grid_output(&mut tree);
    assert_eq!(output.size, Size::new(60.0, 21.0));
}

#[test]
fn named_grid_invalid_template_areas_keep_explicit_line_names() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["foo"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![
                        GridTemplateAreaRow {
                            cells: vec![Some("bad".to_string()), Some("bad".to_string())],
                        },
                        GridTemplateAreaRow {
                            cells: vec![Some("bad".to_string()), None],
                        },
                    ],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "foo".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 40.0);
    assert_eq!(child.size.width, 40.0);
}

#[test]
fn invalid_named_grid_context_is_reported() {
    let mut tree = OracleTree::new().children(1, []).style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
            grid_template_areas: GridTemplateAreas {
                rows: vec![
                    GridTemplateAreaRow {
                        cells: vec![Some("bad".to_string()), Some("bad".to_string())],
                    },
                    GridTemplateAreaRow {
                        cells: vec![Some("bad".to_string())],
                    },
                ],
            },
            ..NodeInput::DEFAULT
        },
    );

    let result = crate::compute_grid_with_report(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(120.0), Some(20.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::Definite(120.0), Available::Definite(20.0)),
        ),
    )
    .unwrap();

    assert!(result.report().named_grid_errors().contains(
        &NamedGridErrorReport::TemplateAreaRowLengthMismatch {
            row: 2,
            expected: 2,
            actual: 1,
        },
    ));
}

#[test]
fn invalid_named_grid_context_fallback_is_reported() {
    let mut tree = OracleTree::new().children(1, []).style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::line_names(["auto"]),
                TrackComponent::px(40.0),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::DEFAULT
        },
    );

    let result = crate::compute_grid_with_report(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(40.0), Some(20.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::Definite(40.0), Available::Definite(20.0)),
        ),
    )
    .unwrap();

    assert!(result.report().named_grid_errors().contains(
        &NamedGridErrorReport::ReservedLineName {
            name: "auto".to_string(),
        },
    ));
}

#[test]
fn invalid_grid_item_placement_reports_one_authored_fallback_once() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(RawGridLine::Line(0), RawGridLine::Auto),
                ..NodeInput::DEFAULT
            },
        );

    let result = crate::compute_grid_with_report(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(40.0), Some(20.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::Definite(40.0), Available::Definite(20.0)),
        ),
    )
    .unwrap();

    let zero_line_count = result
        .report()
        .named_grid_errors()
        .iter()
        .filter(|error| **error == NamedGridErrorReport::ZeroLine)
        .count();

    assert_eq!(zero_line_count, 1);
}

#[test]
fn named_grid_bare_ident_is_distinct_from_explicit_named_line() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["foo-start"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["foo", "foo-end"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("foo".to_string()),
                    RawGridLine::BareIdent("foo".to_string()),
                ),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "foo".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let bare = tree.final_layout(2).expect("bare child should be laid out");
    let explicit = tree
        .final_layout(3)
        .expect("explicit child should be laid out");

    assert_eq!(bare.location.x, 0.0);
    assert_eq!(bare.size.width, 40.0);
    assert_eq!(explicit.location.x, 40.0);
    assert_eq!(explicit.size.width, 40.0);
}

#[test]
fn named_grid_start_after_end_and_equal_lines_normalize() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::lines(3, 1),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::lines(2, 2),
                raw_grid_row: RawGridPlacement::line(2),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let swapped = tree
        .final_layout(2)
        .expect("swapped child should be laid out");
    let equal = tree
        .final_layout(3)
        .expect("equal child should be laid out");

    assert_eq!(swapped.location.x, 0.0);
    assert_eq!(swapped.size.width, 80.0);
    assert_eq!(equal.location.x, 40.0);
    assert_eq!(equal.size.width, 40.0);
}

#[test]
fn auto_repeat_count_uses_f64_saturating_floor() {
    let tracks = [TrackSizingOf::<f64>::px(10.0)];
    let reserved = ReservedTrackSpace::<f64> {
        count: 1,
        size: 10.25,
    };

    let count = auto_repeat_count(&tracks, Some(43.0_f64), 0.25_f64, reserved);

    assert_eq!(count, 3);
}

#[test]
fn shared_grid_contexts_accept_non_default_scalar() {
    let named_lines = named::NamedGridLines::new(GridAxisKind::Column, 1);
    let inherited_axis = InheritedGridAxis::<f64, usize> {
        offset: 0.25,
        gap: 1.5,
        tracks: vec![10.0, 20.0],
        geometry: UsedGridAxisGeometryOf::new(vec![10.0, 20.0], vec![false, false], 1.5),
        named_lines: named_lines.clone(),
        area_facts: None,
        template_area_expanded: false,
        major_baselines: vec![Some(tagged_baseline(PhysicalAxis::Horizontal, 2.0))],
        minor_baselines: vec![None],
        owner_baseline_targets: None,
        parent_start: 0,
        parent_end: 2,
        reversed: false,
    };
    let parent_context = GridParentContext::<f64, usize> {
        columns: Some(inherited_axis),
        rows: None,
    };
    assert!(parent_context.has_inherited_axis());

    let lines = GridLines {
        column_explicit_start: 0,
        column_explicit_count: 1,
        row_explicit_start: 0,
        row_explicit_count: 1,
    };
    let container_context = GridContainerContext::<f64> {
        topology: topology::ExpandedGridTopology::from_test_parts(
            vec![TrackSizingOf::AUTO],
            vec![TrackSizingOf::AUTO],
            named_lines.clone(),
            named::NamedGridLines::new(GridAxisKind::Row, 1),
            None,
        ),
        gap: LogicalSizeOf::new(1.0, 2.0),
        column_gutters: OrdinaryGridAxisGuttersOf::new(1, &[], 1.0),
        row_gutters: OrdinaryGridAxisGuttersOf::new(1, &[], 2.0),
        percent_basis: LogicalSizeOf::new(Some(100.0), None),
        leading_columns: 0,
        leading_rows: 0,
        lines,
        inherited_column_offset: Some(0.25),
        inherited_row_offset: None,
    };
    let constants = Constants::<f64> {
        flow_axes: crate::geometry::FlowAxes::new(
            crate::WritingMode::HorizontalTb,
            crate::Direction::Ltr,
        ),
        explicit_definite_content_size: Size::splat(Some(100.0)),
        node_outer_size: Size::splat(Some(120.0)),
        node_inner_size: Size::splat(Some(100.0)),
        node_min_size: Size::NONE,
        node_max_size: Size::NONE,
        available_inner_size: Size::splat(Some(100.0)),
        content_box_inset: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let style = NodeInputOf::<f64>::default();
    let style_projection = grid_container_projection!(&style);
    let tracks = vec![TrackSizingOf::<f64>::AUTO];
    let placements = GridPlacementContext::new(Vec::<usize>::new(), Vec::new());
    let subgrid_report = GridSubgridReport { items: Vec::new() };
    let sizing_phases = GridTrackSizingPhases;
    let column_gutters = OrdinaryGridAxisGuttersOf::new(1, &[], 1.0);
    let row_gutters = OrdinaryGridAxisGuttersOf::new(1, &[], 2.0);

    let _initialized = InitializedGridTracks::<usize, f64> {
        column_tracks: tracks.clone(),
        row_tracks: tracks.clone(),
        context: container_context.clone(),
        placements: GridPlacementContext::new(Vec::new(), Vec::new()),
        subgrid_report: GridSubgridReport { items: Vec::new() },
        report: GridComputationReport::default(),
    };
    let _track_input = GridTrackResolutionInput::<usize, f64> {
        style: &style_projection,
        constants: &constants,
        column_tracks: &tracks,
        row_tracks: &tracks,
        context: container_context.clone(),
        subgrid_report: &subgrid_report,
        sizing_flow_axes: crate::geometry::FlowAxes::new(
            crate::WritingMode::HorizontalTb,
            crate::Direction::Ltr,
        ),
        available: LogicalSizeOf::new(
            AvailableOf::<f64>::MAX_CONTENT,
            AvailableOf::<f64>::MAX_CONTENT,
        ),
        intrinsic_max_available: LogicalSizeOf::new(false, false),
        placements: &placements,
    };
    let _track_resolution = GridTrackResolution::<f64> {
        sizing_phases,
        columns: vec![10.0],
        rows: vec![20.0],
        column_min_intrinsic_sizes: vec![1.0],
        column_max_intrinsic_sizes: vec![2.0],
        row_intrinsic_sizes: vec![3.0],
    };
    let _child_input = GridChildLayoutInput::<usize, f64> {
        sizing_phases,
        style: &style_projection,
        constants: &constants,
        column_tracks: &tracks,
        row_tracks: &tracks,
        context: container_context.clone(),
        columns: &[10.0],
        rows: &[20.0],
        column_min_intrinsic_sizes: &[1.0],
        column_max_intrinsic_sizes: &[2.0],
        row_intrinsic_sizes: &[3.0],
        output_size: Size::new(100.0, 100.0),
        subgrid_report: &subgrid_report,
        parent_context: &parent_context,
        placements: &placements,
        containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState::INITIAL,
    };
    let _layout_context = GridLayoutContext::<usize, f64> {
        style: &style_projection,
        constants: &constants,
        container_content_size: Size::new(100.0, 100.0),
        columns: &[10.0],
        rows: &[20.0],
        collapsed_columns: &[false],
        collapsed_rows: &[false],
        row_tracks: &tracks,
        gap: LogicalSizeOf::new(1.0, 2.0),
        column_gutters: &column_gutters,
        row_gutters: &row_gutters,
        lines,
        named_columns: named_lines,
        named_rows: named::NamedGridLines::new(GridAxisKind::Row, 1),
        area_facts: None,
        template_area_expanded_axes: TemplateAreaExpandedAxes::default(),
        inherited_column_offset: Some(0.25),
        inherited_row_offset: None,
        subgrid_report: &subgrid_report,
        parent_context: &parent_context,
        placements: &placements,
        containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState::INITIAL,
    };
}

#[test]
fn public_grid_placement_rejects_zero_line_and_span() {
    assert_eq!(GridLine::new(0), None);
    assert_eq!(GridSpan::new(0), None);
    assert!(GridLine::new(1).is_some());
    assert!(GridSpan::new(1).is_some());
    assert_eq!(GridPlacement::try_line(0), None);
    assert_eq!(GridPlacement::try_lines(0, 1), None);
    assert_eq!(GridPlacement::try_lines(1, 0), None);
    assert_eq!(GridPlacement::try_line_span(0, 1), None);
    assert_eq!(GridPlacement::try_line_span(1, 0), None);
    assert_eq!(GridPlacement::try_span_line(0, 1), None);
    assert_eq!(GridPlacement::try_span_line(1, 0), None);
    assert_eq!(GridPlacement::try_span(0), None);
}

#[test]
fn grid_placement_fields_are_constructed_through_validated_values() {
    let placement = GridPlacement::line_span(
        GridLine::new(2).expect("valid line"),
        GridSpan::new(3).expect("valid span"),
    );

    assert_eq!(placement.start(), Some(GridLine::new(2).unwrap()));
    assert_eq!(placement.span(), Some(GridSpan::new(3).unwrap()));
}

#[test]
fn named_lines_preserve_explicit_names_and_fixed_repeats() {
    let lines = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["a"]),
            TrackComponent::px(20.0),
            TrackComponent::line_names(["b", "a"]),
            TrackComponent::Repeat(
                TrackRepetition::count_components(
                    2,
                    vec![
                        TrackComponent::line_names(["c"]),
                        TrackComponent::px(10.0),
                        TrackComponent::line_names(["d"]),
                    ],
                )
                .expect("valid track repetition"),
            ),
        ],
        3,
    )
    .unwrap();

    assert_eq!(lines.named_occurrences("a"), vec![1, 2]);
    assert_eq!(lines.named_occurrences("b"), vec![2]);
    assert_eq!(lines.named_occurrences("c"), vec![2, 3]);
    assert_eq!(lines.named_occurrences("d"), vec![3, 4]);
}

#[test]
fn fri08_c01_topology_duplicate_tokens_preserve_origins_but_count_one_physical_line() {
    let lines = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["a", "b", "a"]),
            TrackComponent::px(20.0),
        ],
        1,
    )
    .unwrap();

    assert_eq!(lines.named_occurrences("a"), vec![1]);
    assert_eq!(
        lines
            .entries_on_line(1)
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b", "a"]
    );
}

#[test]
fn fri08_c01_topology_invalid_names_retain_typed_diagnostics() {
    let error = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["auto"]),
            TrackComponent::px(20.0),
        ],
        1,
    )
    .unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::ReservedLineName {
            name: "auto".to_string(),
        }
    );

    let repeat_error = named::named_lines_from_track_components(
        GridAxisKind::Row,
        &[TrackComponent::Repeat(
            TrackRepetition::count_components(
                2,
                vec![
                    TrackComponent::line_names(["span"]),
                    TrackComponent::px(10.0),
                ],
            )
            .expect("valid track repetition"),
        )],
        2,
    )
    .unwrap_err();

    assert_eq!(
        repeat_error,
        named::NamedGridError::ReservedLineName {
            name: "span".to_string(),
        }
    );
}

#[test]
fn fri08_c01_topology_second_named_occurrence_uses_the_second_physical_line() {
    let lines = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["a", "a"]),
            TrackComponent::px(40.0),
            TrackComponent::line_names(["a"]),
            TrackComponent::px(40.0),
        ],
        2,
    )
    .unwrap();

    let placement = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::Auto,
        ),
        None,
    )
    .unwrap();

    assert_eq!(lines.named_occurrences("a"), vec![1, 2]);
    assert_eq!(
        placement,
        GridPlacement::try_line(2).expect("second physical named line is valid")
    );
}

#[test]
fn fri08_c01_topology_authored_and_area_name_collision_is_one_lookup_occurrence() {
    let style = NodeInput {
        grid_template_columns: vec![
            TrackComponent::line_names(["zone-start"]),
            TrackComponent::px(40.0),
            TrackComponent::line_names(["zone-start"]),
            TrackComponent::px(40.0),
        ],
        grid_template_rows: vec![TrackComponent::px(20.0)],
        grid_template_areas: GridTemplateAreas {
            rows: vec![GridTemplateAreaRow {
                cells: vec![Some("zone".to_string()), Some("zone".to_string())],
            }],
        },
        ..NodeInput::DEFAULT
    };
    let context = named::build_grid_named_context(
        &grid_container_projection!(&style),
        2,
        1,
        &GridParentContext::none(),
    )
    .expect("valid authored names and rectangular areas");

    let positive = named::resolve_grid_placement(
        &context.columns,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "zone-start".to_string(),
                index: 2,
            },
            RawGridLine::Auto,
        ),
        None,
    )
    .unwrap();
    let negative = named::resolve_grid_placement(
        &context.columns,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "zone-start".to_string(),
                index: -1,
            },
            RawGridLine::Auto,
        ),
        None,
    )
    .unwrap();
    let span = named::resolve_grid_placement(
        &context.columns,
        &RawGridPlacement::new(
            RawGridLine::Line(1),
            RawGridLine::NamedSpan {
                name: "zone-start".to_string(),
                index: 1,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(context.columns.named_occurrences("zone-start"), vec![1, 2]);
    assert_eq!(positive, GridPlacement::try_line(2).unwrap());
    assert_eq!(negative, GridPlacement::try_line(2).unwrap());
    assert_eq!(span, GridPlacement::try_lines(1, 2).unwrap());
    assert_eq!(
        context
            .columns
            .entries_on_line(1)
            .iter()
            .filter(|entry| entry.name == "zone-start")
            .map(|entry| entry.origin)
            .collect::<Vec<_>>(),
        vec![
            named::LineNameOrigin::Explicit,
            named::LineNameOrigin::AreaGenerated,
        ]
    );
}

#[test]
fn fri08_c01_topology_retains_auto_repeat_identity_and_boundary_names() {
    let style = NodeInput {
        grid_template_columns: vec![
            TrackComponent::line_names(["leading"]),
            TrackComponent::Repeat(
                TrackRepetition::auto_fit_components(vec![
                    TrackComponent::line_names(["repeat-start"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["repeat-end"]),
                ])
                .expect("valid auto-fit repetition"),
            ),
            TrackComponent::line_names(["trailing"]),
        ],
        grid_template_rows: vec![TrackComponent::px(20.0)],
        ..NodeInput::DEFAULT
    };

    let topology = fri08_c01_topology_for_style(&style, Some(120.0), Some(20.0));

    assert_eq!(topology.explicit_columns, 3);
    assert_eq!(topology.named_columns.named_occurrences("leading"), vec![1]);
    assert_eq!(
        topology.named_columns.named_occurrences("trailing"),
        vec![4]
    );
    assert_eq!(
        topology
            .column_origins
            .iter()
            .map(|origin| origin.auto_repeat)
            .collect::<Vec<_>>(),
        vec![
            Some(AutoRepeatTrackOrigin {
                kind: TrackRepeat::AutoFit,
                repeat_group: 0,
                repetition_index: 0,
                track_index: 0,
            }),
            Some(AutoRepeatTrackOrigin {
                kind: TrackRepeat::AutoFit,
                repeat_group: 0,
                repetition_index: 1,
                track_index: 0,
            }),
            Some(AutoRepeatTrackOrigin {
                kind: TrackRepeat::AutoFit,
                repeat_group: 0,
                repetition_index: 2,
                track_index: 0,
            }),
        ]
    );
}

#[test]
fn named_lines_classify_unresolved_auto_repeat_names() {
    let error = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["before"]),
            TrackComponent::Repeat(
                TrackRepetition::auto_fit_components(vec![
                    TrackComponent::line_names(["inside"]),
                    TrackComponent::px(10.0),
                    TrackComponent::px(10.0),
                ])
                .expect("valid track repetition"),
            ),
            TrackComponent::line_names(["after"]),
        ],
        3,
    )
    .unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::UnresolvedAutoRepeatNames {
            axis: GridAxisKind::Column
        }
    );
}

#[test]
fn named_lines_validate_auto_repeat_names_before_unresolved_classification() {
    let error = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[TrackComponent::Repeat(
            TrackRepetition::auto_fit_components(vec![
                TrackComponent::line_names(["auto"]),
                TrackComponent::px(10.0),
                TrackComponent::px(10.0),
            ])
            .expect("valid track repetition"),
        )],
        3,
    )
    .unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::ReservedLineName {
            name: "auto".to_string(),
        }
    );
}

#[test]
fn named_lines_add_template_area_generated_names_and_facts() {
    let base = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[TrackComponent::line_names(["explicit"])],
        0,
    )
    .unwrap();
    let areas = crate::GridTemplateAreas {
        rows: vec![
            crate::GridTemplateAreaRow {
                cells: vec![Some("head".to_string()), Some("head".to_string())],
            },
            crate::GridTemplateAreaRow {
                cells: vec![Some("nav".to_string()), Some("main".to_string())],
            },
        ],
    };

    let lines = named::add_area_generated_lines(GridAxisKind::Column, base, &areas).unwrap();

    assert_eq!(lines.explicit_track_count, 2);
    assert_eq!(lines.named_occurrences("explicit"), vec![1]);
    assert_eq!(lines.named_occurrences("head-start"), vec![1]);
    assert_eq!(lines.named_occurrences("head-end"), vec![3]);
    assert_eq!(lines.named_occurrences("nav-start"), vec![1]);
    assert_eq!(lines.named_occurrences("main-start"), vec![2]);
    assert_eq!(lines.area_facts.area_order, vec!["head", "nav", "main"]);
    assert_eq!(lines.area_facts.row_count, 2);
    assert_eq!(lines.area_facts.column_count, 2);
    assert!(lines.area_facts.rows_valid);
    assert!(lines.area_facts.columns_valid);
    assert_eq!(
        lines.area_facts.area_rectangles,
        vec![
            named::GridAreaNameRectangle {
                name: "head".to_string(),
                row_start: 1,
                row_end: 2,
                column_start: 1,
                column_end: 3,
                row_start_name: 1,
                row_end_name: 2,
                column_start_name: 1,
                column_end_name: 3,
            },
            named::GridAreaNameRectangle {
                name: "nav".to_string(),
                row_start: 2,
                row_end: 3,
                column_start: 1,
                column_end: 2,
                row_start_name: 2,
                row_end_name: 3,
                column_start_name: 1,
                column_end_name: 2,
            },
            named::GridAreaNameRectangle {
                name: "main".to_string(),
                row_start: 2,
                row_end: 3,
                column_start: 2,
                column_end: 3,
                row_start_name: 2,
                row_end_name: 3,
                column_start_name: 2,
                column_end_name: 3,
            },
        ]
    );
}

#[test]
fn named_lines_ignore_template_area_null_cells() {
    let base =
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Row, &[], 0).unwrap();
    let areas = crate::GridTemplateAreas {
        rows: vec![
            crate::GridTemplateAreaRow {
                cells: vec![None, Some("main".to_string())],
            },
            crate::GridTemplateAreaRow {
                cells: vec![None, Some("main".to_string())],
            },
        ],
    };

    let lines = named::add_area_generated_lines(GridAxisKind::Row, base, &areas).unwrap();

    assert_eq!(lines.named_occurrences("main-start"), vec![1]);
    assert_eq!(lines.named_occurrences("main-end"), vec![3]);
    assert!(lines.named_occurrences(".-start").is_empty());
}

#[test]
fn named_lines_reject_invalid_template_area_row_widths() {
    let areas = crate::GridTemplateAreas {
        rows: vec![
            crate::GridTemplateAreaRow {
                cells: vec![Some("a".to_string()), Some("a".to_string())],
            },
            crate::GridTemplateAreaRow {
                cells: vec![Some("a".to_string())],
            },
        ],
    };

    let error = named::GridAreaNameFacts::from_specified_areas(&areas).unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::TemplateAreaRowLengthMismatch {
            row: 2,
            expected: 2,
            actual: 1,
        }
    );
}

fn named_grid_lines() -> named::NamedGridLines {
    named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["a", "foo-start"]),
            TrackComponent::px(20.0),
            TrackComponent::line_names(["a", "foo", "foo-end"]),
            TrackComponent::px(20.0),
            TrackComponent::line_names(["a"]),
        ],
        2,
    )
    .unwrap()
}

#[test]
fn named_grid_resolver_places_between_repeated_line_and_named_span() {
    let placement = named::resolve_grid_placement(
        &named_grid_lines(),
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 1,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(2, 3).expect("valid grid lines")
    );
}

#[test]
fn named_grid_resolver_uses_side_aware_bare_ident_before_plain_name() {
    let lines = named_grid_lines();

    let bare = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::BareIdent("foo".to_string()),
            RawGridLine::BareIdent("foo".to_string()),
        ),
        None,
    )
    .unwrap();
    let explicit = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "foo".to_string(),
                index: 1,
            },
            RawGridLine::NamedLine {
                name: "foo".to_string(),
                index: 1,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        bare,
        GridPlacement::try_lines(1, 2).expect("valid grid lines")
    );
    assert_eq!(
        explicit,
        GridPlacement::try_line_span(2, 1).expect("valid grid line span")
    );
}

#[test]
fn named_grid_resolver_handles_negative_and_missing_occurrences() {
    let lines = named_grid_lines();

    let negative = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: -1,
            },
            RawGridLine::Auto,
        ),
        None,
    )
    .unwrap();
    let missing_after = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 4,
            },
            RawGridLine::Auto,
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        negative,
        GridPlacement::try_line(3).expect("valid grid line")
    );
    assert_eq!(
        missing_after,
        GridPlacement::try_line(4).expect("valid grid line")
    );
}

#[test]
fn named_grid_resolver_normalizes_spans_and_conflicts() {
    let lines = named_grid_lines();

    let lone_named_span = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::Auto,
        ),
        Some(2),
    )
    .unwrap();
    let both_spans = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(RawGridLine::Span(2), RawGridLine::Span(3)),
        Some(1),
    )
    .unwrap();
    let mixed_named_span = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::Span(3),
        ),
        Some(1),
    )
    .unwrap();
    let mixed_anonymous_span_first = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::Span(3),
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
        ),
        Some(1),
    )
    .unwrap();
    let start_after_end =
        named::resolve_grid_placement(&lines, &RawGridPlacement::lines(3, 1), None).unwrap();
    let equal_lines =
        named::resolve_grid_placement(&lines, &RawGridPlacement::lines(2, 2), None).unwrap();

    assert_eq!(
        lone_named_span,
        GridPlacement::try_line_span(2, 1).expect("valid grid line span")
    );
    assert_eq!(
        both_spans,
        GridPlacement::try_line_span(1, 2).expect("valid grid line span")
    );
    assert_eq!(
        mixed_named_span,
        GridPlacement::try_line_span(1, 1).expect("valid grid line span")
    );
    assert_eq!(
        mixed_anonymous_span_first,
        GridPlacement::try_line_span(1, 3).expect("valid grid line span")
    );
    assert_eq!(
        start_after_end,
        GridPlacement::try_lines(1, 3).expect("valid grid lines")
    );
    assert_eq!(
        equal_lines,
        GridPlacement::try_line_span(2, 1).expect("valid grid line span")
    );
}

#[test]
fn grid_axis_placement_preserves_out_of_range_numeric_lines() {
    let lines =
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 2).unwrap();

    assert_eq!(
        resolve_grid_item_axis_placement(
            &lines,
            &RawGridPlacement::line(-5),
            GridPlacement::try_line(-5).expect("valid grid line"),
        ),
        GridPlacement::try_line(-5).expect("valid grid line")
    );
    assert_eq!(
        resolve_grid_item_axis_placement(
            &lines,
            &RawGridPlacement::line(5),
            GridPlacement::try_line(5).expect("valid grid line"),
        ),
        GridPlacement::try_line(5).expect("valid grid line")
    );
}

#[test]
fn named_grid_invalid_raw_placement_falls_back_to_auto() {
    let lines = named_grid_lines();

    assert_eq!(
        named::resolve_grid_placement_or_auto(
            &lines,
            &RawGridPlacement::new(RawGridLine::Line(0), RawGridLine::Auto),
            None,
        ),
        GridPlacement::AUTO
    );
    assert_eq!(
        named::resolve_grid_placement_or_auto(
            &lines,
            &RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "auto".to_string(),
                    index: 1,
                },
                RawGridLine::Auto,
            ),
            None,
        ),
        GridPlacement::AUTO
    );
    assert_eq!(
        named::resolve_grid_placement_or_auto(
            &lines,
            &RawGridPlacement::new(RawGridLine::Span(0), RawGridLine::Auto),
            Some(1),
        ),
        GridPlacement::AUTO
    );
    assert_eq!(
        named::resolve_grid_placement_or_auto(
            &lines,
            &RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "missing".to_string(),
                    index: -4,
                },
                RawGridLine::Auto,
            ),
            None,
        ),
        GridPlacement::AUTO
    );
}

#[test]
fn named_grid_placement_fallback_is_reported() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let (placement, report) = named::resolve_grid_placement_or_auto_with_report(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 0,
            },
            RawGridLine::Auto,
        ),
        None,
    );

    assert_eq!(placement, GridPlacement::AUTO);
    assert!(report.errors().contains(&NamedGridErrorReport::ZeroLine));
}

#[test]
fn named_grid_implicit_named_line_is_not_reported_as_fallback() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let (placement, report) = named::resolve_grid_placement_or_auto_with_report(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "implicit".to_string(),
                index: 1,
            },
            RawGridLine::Auto,
        ),
        None,
    );

    assert_eq!(
        placement,
        GridPlacement::try_line(4).expect("valid implicit grid line")
    );
    assert!(report.is_empty());
}

#[test]
fn named_lines_reject_non_rectangular_template_areas() {
    let areas = crate::GridTemplateAreas {
        rows: vec![
            crate::GridTemplateAreaRow {
                cells: vec![Some("a".to_string()), Some("a".to_string())],
            },
            crate::GridTemplateAreaRow {
                cells: vec![Some("a".to_string()), None],
            },
        ],
    };

    let error = named::GridAreaNameFacts::from_specified_areas(&areas).unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::NonRectangularTemplateArea {
            name: "a".to_string(),
        }
    );
}

#[test]
fn named_lines_treat_default_template_areas_as_noop() {
    let base =
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 1).unwrap();
    let lines = named::add_area_generated_lines(
        GridAxisKind::Column,
        base,
        &crate::GridTemplateAreas::default(),
    )
    .unwrap();

    assert_eq!(lines.explicit_track_count, 1);
    assert_eq!(lines.line_names.len(), 2);
    assert!(lines.area_facts.area_order.is_empty());
}

#[test]
fn grid_axis_mapping_supports_horizontal_rtl_reversal() {
    let report = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            direction: Direction::Rtl,
            ..NodeInput::default()
        },
        child_style: &NodeInput::default(),
    });

    assert_eq!(report.parent_axis, GridAxisKind::Column);
    assert_eq!(report.child_axis, GridAxisKind::Column);
    assert!(report.reversed);
}

#[test]
fn grid_axis_mapping_supports_sideways_lr_used_direction_inversion() {
    let report = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            writing_mode: WritingMode::SidewaysLr,
            direction: Direction::Ltr,
            ..NodeInput::default()
        },
        child_style: &NodeInput {
            writing_mode: WritingMode::SidewaysLr,
            direction: Direction::Rtl,
            ..NodeInput::default()
        },
    });

    assert_eq!(report.parent_axis, GridAxisKind::Column);
    assert_eq!(report.child_axis, GridAxisKind::Column);
    assert!(report.reversed);
}
