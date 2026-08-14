use super::fixtures::{
    Fri08C02StretchTreeInput, Fri08C02TrackAxis, Fri08C03NestedAtomicTree, Fri08C03NestedFlowCase,
    Fri08C03NestedMeasureError, Fri08C03NestedMeasureMode, Fri08C04BaselineFlowCase,
    Fri08C04BaselineMeasureError, Fri08C04BaselineMeasureMode, Fri08C04BaselineParentAxis,
    Fri08C06RAtomicTree, Fri08C06RInheritedAxes, SubgridChildParentContextInput,
    SubgridEligibilityInput, assert_fri06_mr02_geometry_error_grid_own,
    assert_fri08_c02_stretch_intrinsic_minimums,
    assert_fri08_c03_containing_block_percentage_children,
    assert_fri08_c03_containing_block_percentage_controls,
    assert_fri08_c03_nested_candidate_bounds_edges_and_reversal,
    assert_fri08_c04_baseline_area_topology_controls, assert_fri08_c04_standalone_nested_flows,
    baseline_measure, computed_overflow, default_grid_item_projection, empty_subgrid_track,
    fri04_c04_grid_dispatch_assert_error, fri05_c05_grid_sizing_input, fri08_c01_placement_output,
    fri08_c01_placement_request, fri08_c02_auto_fit_repeat, fri08_c02_fit_content_track,
    fri08_c02_flex_track, fri08_c02_stretch_track, fri08_c02_stretch_tree,
    fri08_c02_track_mix_tree, fri08_c02_track_sizes, fri08_c03_nested_projection_tree,
    fri08_c04_baseline_area_implicit_tree, fri08_c04_baseline_world_coordinate,
    fri08_c04_standalone_nested_tree, fri08_c06r_assert_cold_warm, invalid_numeric_lp,
    subgrid_axis_report, subgrid_child_parent_context_with_geometry, subgrid_eligibility,
    subgrid_track, subgrid_track_of, tagged_group, traversal_leaf,
};
use super::*;

fn fri08_c06r_inherited_placement_flow_tree<S: LayoutScalar>(
    inherited_axes: Fri08C06RInheritedAxes,
    flow: GridAutoFlow,
) -> PublicLayoutTreeOf<S> {
    let scalar = S::from_f64;
    let inherited_columns = matches!(
        inherited_axes,
        Fri08C06RInheritedAxes::Columns | Fri08C06RInheritedAxes::Both
    );
    let inherited_rows = matches!(
        inherited_axes,
        Fri08C06RInheritedAxes::Rows | Fri08C06RInheritedAxes::Both
    );
    let root_columns = if inherited_columns {
        vec![TrackComponentOf::px(scalar(20.0)); 4]
    } else {
        vec![TrackComponentOf::px(scalar(100.0))]
    };
    let root_rows = if inherited_rows {
        vec![TrackComponentOf::px(scalar(20.0)); 4]
    } else {
        vec![TrackComponentOf::px(scalar(100.0))]
    };
    let subgrid_columns = if inherited_columns {
        subgrid_track_of()
    } else {
        vec![TrackComponentOf::px(scalar(20.0))]
    };
    let subgrid_rows = if inherited_rows {
        subgrid_track_of()
    } else {
        vec![TrackComponentOf::px(scalar(20.0))]
    };
    let (occupied_column, occupied_row, span_column, span_row) = if flow.is_column() {
        (
            GridPlacement::try_line(1).expect("first column"),
            GridPlacement::try_line(2).expect("second row"),
            GridPlacement::AUTO,
            GridPlacement::try_span(2).expect("two-row automatic span"),
        )
    } else {
        (
            GridPlacement::try_line(2).expect("second column"),
            GridPlacement::try_line(1).expect("first row"),
            GridPlacement::try_span(2).expect("two-column automatic span"),
            GridPlacement::AUTO,
        )
    };
    let definite = NodeInputOf {
        grid_column: occupied_column,
        grid_row: occupied_row,
        ..NodeInputOf::default()
    };

    PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3, 4, 5, 6])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(if inherited_columns {
                        scalar(80.0)
                    } else {
                        scalar(100.0)
                    }),
                    PreferredSizeOf::px(if inherited_rows {
                        scalar(80.0)
                    } else {
                        scalar(100.0)
                    }),
                ),
                grid_template_columns: root_columns,
                grid_template_rows: root_rows,
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: subgrid_columns,
                grid_template_rows: subgrid_rows,
                grid_auto_columns: vec![TrackComponentOf::px(scalar(20.0))],
                grid_auto_rows: vec![TrackComponentOf::px(scalar(20.0))],
                grid_column: if inherited_columns {
                    GridPlacement::try_line_span(1, 4).expect("four inherited columns")
                } else {
                    GridPlacement::try_line(1).expect("standalone column")
                },
                grid_row: if inherited_rows {
                    GridPlacement::try_line_span(1, 4).expect("four inherited rows")
                } else {
                    GridPlacement::try_line(1).expect("standalone row")
                },
                grid_auto_flow: flow,
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(3, definite.clone())
        .style(4, definite)
        .style(
            5,
            NodeInputOf {
                grid_column: span_column,
                grid_row: span_row,
                ..NodeInputOf::default()
            },
        )
        .style(6, NodeInputOf::default())
}

fn assert_fri08_c06r_inherited_placement_flow_matrix<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for (inherited_axes, flows) in [
        (
            Fri08C06RInheritedAxes::Columns,
            [GridAutoFlow::Row, GridAutoFlow::RowDense].as_slice(),
        ),
        (
            Fri08C06RInheritedAxes::Rows,
            [GridAutoFlow::Column, GridAutoFlow::ColumnDense].as_slice(),
        ),
        (
            Fri08C06RInheritedAxes::Both,
            [
                GridAutoFlow::Row,
                GridAutoFlow::RowDense,
                GridAutoFlow::Column,
                GridAutoFlow::ColumnDense,
            ]
            .as_slice(),
        ),
    ] {
        for &flow in flows {
            let tree = fri08_c06r_inherited_placement_flow_tree(inherited_axes, flow);
            fri08_c06r_assert_cold_warm(tree, &[1, 2, 3, 4, 5, 6], |batch| {
                let root = fri08_c01_placement_output(batch, 1);
                let subgrid = fri08_c01_placement_output(batch, 2);
                let expected_size = match inherited_axes {
                    Fri08C06RInheritedAxes::Columns => Size::new(scalar(80.0), scalar(100.0)),
                    Fri08C06RInheritedAxes::Rows => Size::new(scalar(100.0), scalar(80.0)),
                    Fri08C06RInheritedAxes::Both => Size::splat(scalar(80.0)),
                };
                assert_eq!(root.size, expected_size, "{inherited_axes:?} {flow:?}");
                assert_eq!(subgrid.location, Point::ZERO, "{inherited_axes:?} {flow:?}");
                assert_eq!(subgrid.size, expected_size, "{inherited_axes:?} {flow:?}");

                let overlap = if flow.is_column() {
                    Point::new(S::ZERO, scalar(20.0))
                } else {
                    Point::new(scalar(20.0), S::ZERO)
                };
                assert_eq!(
                    fri08_c01_placement_output(batch, 3).location,
                    overlap,
                    "{inherited_axes:?} {flow:?} first definite overlap"
                );
                assert_eq!(
                    fri08_c01_placement_output(batch, 4).location,
                    overlap,
                    "{inherited_axes:?} {flow:?} second definite overlap"
                );
                let spanning = fri08_c01_placement_output(batch, 5);
                let expected_span_location = if flow.is_column() {
                    Point::new(S::ZERO, scalar(40.0))
                } else {
                    Point::new(scalar(40.0), S::ZERO)
                };
                let expected_span_size = if flow.is_column() {
                    Size::new(scalar(20.0), scalar(40.0))
                } else {
                    Size::new(scalar(40.0), scalar(20.0))
                };
                assert_eq!(
                    spanning.location, expected_span_location,
                    "{inherited_axes:?} {flow:?}"
                );
                assert_eq!(
                    spanning.size, expected_span_size,
                    "{inherited_axes:?} {flow:?}"
                );

                let final_location = if flow.is_dense() {
                    Point::ZERO
                } else if flow.is_column() {
                    Point::new(scalar(20.0), S::ZERO)
                } else {
                    Point::new(S::ZERO, scalar(20.0))
                };
                assert_eq!(
                    fri08_c01_placement_output(batch, 6).location,
                    final_location,
                    "{inherited_axes:?} {flow:?} sparse cursor versus dense hole"
                );
                for node in 3..=6 {
                    assert_ne!(
                        fri08_c01_placement_output(batch, node).size,
                        Size::ZERO,
                        "{inherited_axes:?} {flow:?} node {node} has settled geometry"
                    );
                }
            });
        }
    }
}

#[test]
fn fri08_c06r_inherited_placement_sparse_dense_row_column_flows_preserve_geometry_and_cache() {
    assert_fri08_c06r_inherited_placement_flow_matrix::<f32>();
    assert_fri08_c06r_inherited_placement_flow_matrix::<f64>();
}

fn fri08_c06r_inherited_placement_overflow_tree<S: LayoutScalar>(
    inherited_axes: Fri08C06RInheritedAxes,
    overflow_axis: GridAxisKind,
    span: usize,
) -> PublicLayoutTreeOf<S> {
    let scalar = S::from_f64;
    let inherited_columns = matches!(
        inherited_axes,
        Fri08C06RInheritedAxes::Columns | Fri08C06RInheritedAxes::Both
    );
    let inherited_rows = matches!(
        inherited_axes,
        Fri08C06RInheritedAxes::Rows | Fri08C06RInheritedAxes::Both
    );
    let child_column = if overflow_axis == GridAxisKind::Column {
        GridPlacement::try_span(span).expect("nonzero inherited column span")
    } else {
        GridPlacement::AUTO
    };
    let child_row = if overflow_axis == GridAxisKind::Row {
        GridPlacement::try_span(span).expect("nonzero inherited row span")
    } else {
        GridPlacement::AUTO
    };

    PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(if inherited_columns {
                        scalar(80.0)
                    } else {
                        scalar(20.0)
                    }),
                    PreferredSizeOf::px(if inherited_rows {
                        scalar(80.0)
                    } else {
                        scalar(20.0)
                    }),
                ),
                grid_template_columns: if inherited_columns {
                    vec![TrackComponentOf::px(scalar(20.0)); 4]
                } else {
                    vec![TrackComponentOf::px(scalar(20.0))]
                },
                grid_template_rows: if inherited_rows {
                    vec![TrackComponentOf::px(scalar(20.0)); 4]
                } else {
                    vec![TrackComponentOf::px(scalar(20.0))]
                },
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: if inherited_columns {
                    subgrid_track_of()
                } else {
                    vec![TrackComponentOf::px(scalar(20.0))]
                },
                grid_template_rows: if inherited_rows {
                    subgrid_track_of()
                } else {
                    vec![TrackComponentOf::px(scalar(20.0))]
                },
                grid_column: if inherited_columns {
                    GridPlacement::try_line_span(1, 4).expect("four inherited columns")
                } else {
                    GridPlacement::try_line(1).expect("standalone column")
                },
                grid_row: if inherited_rows {
                    GridPlacement::try_line_span(1, 4).expect("four inherited rows")
                } else {
                    GridPlacement::try_line(1).expect("standalone row")
                },
                grid_auto_flow: if overflow_axis == GridAxisKind::Column {
                    GridAutoFlow::Row
                } else {
                    GridAutoFlow::Column
                },
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: child_column,
                grid_row: child_row,
                ..NodeInputOf::default()
            },
        )
}

fn assert_fri08_c06r_inherited_placement_capacity_overflow_is_atomic<S: LayoutScalar>() {
    for (inherited_axes, overflow_axis) in [
        (Fri08C06RInheritedAxes::Columns, GridAxisKind::Column),
        (Fri08C06RInheritedAxes::Rows, GridAxisKind::Row),
        (Fri08C06RInheritedAxes::Both, GridAxisKind::Column),
        (Fri08C06RInheritedAxes::Both, GridAxisKind::Row),
    ] {
        let accepted =
            fri08_c06r_inherited_placement_overflow_tree(inherited_axes, overflow_axis, 4);
        let mut tree = Fri08C06RAtomicTree::new(accepted);
        let request = Fri08C06RAtomicTree::<S>::request();
        let baseline = compute_layout(&tree, 1, request)
            .expect("an automatic span equal to inherited capacity succeeds");
        assert_ne!(
            fri08_c01_placement_output(&baseline, 3).size,
            Size::ZERO,
            "the within-capacity baseline publishes settled geometry"
        );
        baseline
            .apply_to(&mut tree)
            .expect("the accepted baseline commits atomically");
        assert!(
            !tree.retained.caches.is_empty(),
            "the baseline commits cache state"
        );

        let overflowing =
            fri08_c06r_inherited_placement_overflow_tree(inherited_axes, overflow_axis, 5);
        tree.tree.insert_input(3, overflowing.layout_input(3));
        let retained_before_failure = tree.retained.clone();

        for attempt_index in 0..2 {
            let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                compute_layout_invalidated(&tree, 1, request, &[3])
            }));
            let error = match attempt {
                Ok(Err(error)) => error,
                Ok(Ok(batch)) => panic!(
                    "attempt {attempt_index}: inherited {overflow_axis:?} capacity overflow must not return a completed batch; child output was {:?}",
                    fri08_c01_placement_output(&batch, 3)
                ),
                Err(_) => panic!(
                    "attempt {attempt_index}: inherited {overflow_axis:?} capacity overflow must return the typed error instead of panicking"
                ),
            };
            assert_eq!(
                error.site(),
                LayoutErrorSiteOf::Node(2),
                "{inherited_axes:?} {overflow_axis:?} attempt {attempt_index}"
            );
            assert_eq!(error.operation(), LayoutOperation::ChildLayout);
            assert_eq!(
                error.kind(),
                &LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::InvalidBlockScrollGeometry,
                )
            );
            assert_eq!(
                tree.retained, retained_before_failure,
                "{inherited_axes:?} {overflow_axis:?} attempt {attempt_index}: failure publishes no outputs and mutates no committed cache"
            );
        }
    }
}

#[test]
fn fri08_c06r_inherited_placement_capacity_overflow_is_typed_atomic_and_retryable() {
    assert_fri08_c06r_inherited_placement_capacity_overflow_is_atomic::<f32>();
    assert_fri08_c06r_inherited_placement_capacity_overflow_is_atomic::<f64>();
}

fn fri08_c05_composition_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("composition output for source node {node}"))
        .output()
}

fn fri08_c05_composition_grid001_placement_topology_order<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4, 5])
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
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                grid_column: GridPlacement::try_line(2).expect("middle explicit column"),
                grid_row: GridPlacement::try_line(1).expect("first explicit row"),
                item_order: ItemOrder::new(9),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: GridPlacement::try_span(2).expect("two-column automatic span"),
                item_order: ItemOrder::new(-9),
                size: Size::new(
                    PreferredSizeOf::px(scalar(80.0)),
                    PreferredSizeOf::px(scalar(20.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                display: Display::None,
                item_order: ItemOrder::new(-20),
                grid_column: GridPlacement::try_span(8).expect("excluded large span"),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                position: Position::Absolute,
                item_order: ItemOrder::new(-30),
                size: Size::new(
                    PreferredSizeOf::px(scalar(7.0)),
                    PreferredSizeOf::px(scalar(5.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(240.0))))
            .expect("finite composition viewport"),
    )
    .expect("GRID-001 public composition succeeds");

    assert_eq!(
        fri08_c05_composition_output(&batch, 1).size,
        Size::new(scalar(120.0), scalar(40.0))
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 2).location,
        Point::new(scalar(40.0), S::ZERO)
    );
    let spanning = fri08_c05_composition_output(&batch, 3);
    assert_eq!(spanning.source_index, SourceIndex::new(1));
    assert_eq!(spanning.location, Point::new(S::ZERO, scalar(20.0)));
    assert_eq!(spanning.size, Size::new(scalar(80.0), scalar(20.0)));
    assert_eq!(spanning.content_size, Size::new(scalar(80.0), scalar(20.0)));
    assert_eq!(fri08_c05_composition_output(&batch, 4).size, Size::ZERO);
    assert_eq!(
        fri08_c05_composition_output(&batch, 5).size,
        Size::new(scalar(7.0), scalar(5.0))
    );
}

#[test]
fn fri08_c05_composition_grid001_span_after_occupied_keeps_exact_demand_and_source_identity() {
    fri08_c05_composition_grid001_placement_topology_order::<f32>();
    fri08_c05_composition_grid001_placement_topology_order::<f64>();
}

#[test]
fn fri08_c05_composition_grid002_lanes_percentages_use_hybrid_boxes_in_all_flows() {
    assert_fri08_c03_containing_block_percentage_children::<f32>();
    assert_fri08_c03_containing_block_percentage_children::<f64>();
    assert_fri08_c03_containing_block_percentage_controls::<f32>();
    assert_fri08_c03_containing_block_percentage_controls::<f64>();
}

#[test]
fn fri08_c05_composition_grid003_fit_content_flex_and_stretch_share_one_sizing_result() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
            fri08_c05_composition_grid003_fit_flex_stretch::<f32>(axis, writing_mode);
            fri08_c05_composition_grid003_fit_flex_stretch::<f64>(axis, writing_mode);
        }
    }
}

fn fri08_c05_composition_grid003_fit_flex_stretch<S: LayoutScalar>(
    axis: Fri08C02TrackAxis,
    writing_mode: WritingMode,
) {
    let (tree, flow_axes, viewport) = fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
        display: Display::Grid,
        axis,
        writing_mode,
        definite_axis_size: Some(200.0),
        viewport_axis_size: 200.0,
        gap: 0.0,
        alignment: Some(AlignContent::Stretch),
        tracks: vec![
            fri08_c02_fit_content_track(50.0, 0.0),
            fri08_c02_flex_track(0.5),
            fri08_c02_stretch_track(MinTrackSizingOf::px(S::ZERO)),
        ],
        measurements: &[20.0, 0.0, 0.0],
    });

    assert_eq!(
        fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 3),
        [S::from_f64(20.0), S::from_f64(90.0), S::from_f64(90.0)]
    );
}

fn fri08_c05_composition_grid005_area_only_topology<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_auto_columns: vec![
                    TrackComponentOf::px(scalar(40.0)),
                    TrackComponentOf::px(scalar(20.0)),
                ],
                grid_auto_rows: vec![TrackComponentOf::px(scalar(15.0))],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            Some("main".to_string()),
                            Some("main".to_string()),
                            Some("main".to_string()),
                        ],
                    }],
                },
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                raw_grid_row: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("area-only intrinsic viewport"),
    )
    .expect("GRID-005 public composition succeeds");
    assert_eq!(
        fri08_c05_composition_output(&batch, 1).size,
        Size::new(scalar(100.0), scalar(15.0))
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 2).location,
        Point::ZERO
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 2).size,
        Size::new(scalar(100.0), scalar(15.0))
    );
}

#[test]
fn fri08_c05_composition_grid005_area_only_topology_uses_auto_pattern_and_area_edges() {
    fri08_c05_composition_grid005_area_only_topology::<f32>();
    fri08_c05_composition_grid005_area_only_topology::<f64>();
}

fn fri08_c05_composition_grid006_auto_fit_overlap_span_hole<S: LayoutScalar>(
    display: Display,
) -> [NodeOutputOf<S>; 4] {
    let scalar = S::from_f64;
    let repeat = TrackComponentOf::Repeat(
        TrackRepetitionOf::auto_fit_components(vec![
            TrackComponentOf::line_names(["slot"]),
            TrackComponentOf::px(scalar(40.0)),
        ])
        .expect("finite named auto-fit repeat"),
    );
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4, 5])
        .style(
            1,
            NodeInputOf {
                display,
                size: Size::new(
                    PreferredSizeOf::px(scalar(240.0)),
                    PreferredSizeOf::px(scalar(40.0)),
                ),
                grid_template_columns: vec![repeat],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                gap: Size::new(LengthOf::px(scalar(10.0)), LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                grid_column: GridPlacement::try_line(3).expect("third repeated track"),
                grid_row: GridPlacement::try_line(1).expect("shared explicit row"),
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: GridPlacement::try_line(3).expect("overlapping repeated track"),
                grid_row: GridPlacement::try_line(1).expect("shared explicit row"),
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                grid_column: GridPlacement::try_line(1).expect("first repeated track"),
                grid_row: GridPlacement::try_line(1).expect("shared explicit row"),
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                grid_column: GridPlacement::try_span(2).expect("two-track automatic span"),
                grid_row: GridPlacement::try_line(1).expect("shared explicit row"),
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(240.0)),
            AvailableOf::definite(scalar(40.0)),
        ))
        .expect("finite auto-fit viewport"),
    )
    .expect("GRID-006 public composition succeeds");
    [2, 3, 4, 5].map(|node| fri08_c05_composition_output(&batch, node))
}

#[test]
fn fri08_c05_composition_grid006_ordinary_and_lanes_auto_fit_keep_distinct_occupancy_policies() {
    fri08_c05_composition_grid006_distinct_policies::<f32>();
    fri08_c05_composition_grid006_distinct_policies::<f64>();
}

fn fri08_c05_composition_grid006_distinct_policies<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let ordinary = fri08_c05_composition_grid006_auto_fit_overlap_span_hole::<S>(Display::Grid);
    let lanes = fri08_c05_composition_grid006_auto_fit_overlap_span_hole::<S>(Display::GridLanes);

    assert_eq!(ordinary[0].location.x, scalar(50.0));
    assert_eq!(ordinary[1].location.x, scalar(50.0));
    assert_eq!(ordinary[2].location.x, scalar(0.0));
    assert_eq!(ordinary[3].location.x, scalar(100.0));
    assert_eq!(ordinary[3].size.width, scalar(90.0));

    assert_eq!(lanes[0].location.x, scalar(100.0));
    assert_eq!(lanes[1].location.x, scalar(100.0));
    assert_eq!(lanes[2].location.x, scalar(0.0));
    assert_eq!(lanes[3].size.width, scalar(90.0));
}

#[test]
fn fri08_c05_composition_grid007_auto_max_stretch_preserves_floors_in_all_axes() {
    assert_fri08_c02_stretch_intrinsic_minimums::<f32>();
    assert_fri08_c02_stretch_intrinsic_minimums::<f64>();
}

fn fri08_c05_composition_grid008_named_area_collision<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(scalar(80.0)),
                    PreferredSizeOf::px(scalar(20.0)),
                ),
                grid_template_columns: vec![
                    TrackComponentOf::line_names(["zone-start", "zone-start"]),
                    TrackComponentOf::px(scalar(40.0)),
                    TrackComponentOf::line_names(["zone-start"]),
                    TrackComponentOf::px(scalar(40.0)),
                ],
                grid_template_rows: vec![TrackComponentOf::px(scalar(20.0))],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![Some("zone".to_string()), Some("zone".to_string())],
                    }],
                },
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "zone-start".to_string(),
                        index: 2,
                    },
                    RawGridLine::Auto,
                ),
                raw_grid_row: RawGridPlacement::new(
                    RawGridLine::BareIdent("zone".to_string()),
                    RawGridLine::BareIdent("zone".to_string()),
                ),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(80.0)),
            AvailableOf::definite(scalar(20.0)),
        ))
        .expect("finite named-area viewport"),
    )
    .expect("GRID-008 public composition succeeds");
    assert_eq!(
        fri08_c05_composition_output(&batch, 2).location,
        Point::new(scalar(40.0), S::ZERO)
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 2).size,
        Size::new(scalar(40.0), scalar(20.0))
    );
}

#[test]
fn fri08_c05_composition_grid008_duplicate_names_and_area_origins_count_one_line() {
    fri08_c05_composition_grid008_named_area_collision::<f32>();
    fri08_c05_composition_grid008_named_area_collision::<f64>();
}

#[test]
fn fri08_c05_composition_grid010_standalone_and_inherited_boundaries_keep_baseline_roles() {
    assert_fri08_c04_standalone_nested_flows::<f32>();
    assert_fri08_c04_standalone_nested_flows::<f64>();
    fri08_c05_composition_grid010_nested_indefinite_lanes_output::<f32>();
    fri08_c05_composition_grid010_nested_indefinite_lanes_output::<f64>();
    assert_fri08_c04_baseline_area_topology_controls::<f32>();
    assert_fri08_c04_baseline_area_topology_controls::<f64>();
}

fn fri08_c05_composition_grid010_nested_indefinite_lanes_output<S: LayoutScalar>() {
    let tree = fri08_c03_nested_projection_tree(
        Fri08C03NestedFlowCase {
            root_direction: Direction::Ltr,
            first_wrapper_mode: WritingMode::HorizontalTb,
            first_wrapper_direction: Direction::Ltr,
            second_wrapper_mode: WritingMode::HorizontalTb,
            second_wrapper_direction: Direction::Ltr,
            inherited_axis: GridAxisKind::Column,
        },
        GridFlowToleranceOf::Length(LengthOf::ZERO),
        false,
    );
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("nested indefinite lanes viewport"),
    )
    .expect("nested indefinite lanes subgrid publishes a completed batch");

    assert_eq!(
        batch
            .final_entries()
            .iter()
            .map(LayoutOutputEntryOf::node)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 1).size,
        Size::new(S::from_f64(60.0), S::from_f64(10.0))
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 4).size,
        Size::new(S::from_f64(20.0), S::from_f64(10.0))
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 5).size,
        Size::new(S::from_f64(40.0), S::from_f64(10.0))
    );
    assert_eq!(
        [6, 7, 8].map(|node| fri08_c05_composition_output(&batch, node).size.width),
        [S::from_f64(20.0), S::ZERO, S::from_f64(40.0)]
    );
}

fn fri08_c05_composition_fractional_rounding<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::splat_clone(PreferredSizeOf::px(scalar(100.5))),
                grid_template_columns: vec![TrackComponentOf::px(scalar(100.5))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(100.5))],
                justify_items: Some(AlignItems::End),
                align_items: Some(AlignItems::End),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(scalar(10.25)),
                    PreferredSizeOf::px(scalar(20.25)),
                ),
                ..NodeInputOf::default()
            },
        );
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.5))))
        .expect("finite fractional viewport");
    let first = compute_layout(&tree, 1, request).expect("first fractional grid layout");
    let second = compute_layout(&tree, 1, request).expect("second fractional grid layout");
    assert_eq!(first.final_entries(), second.final_entries());
    assert_eq!(
        first
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == 2)
            .expect("unrounded fractional child")
            .output()
            .location,
        Point::new(scalar(90.25), scalar(80.25))
    );
    assert_eq!(
        fri08_c05_composition_output(&first, 2).location,
        Point::new(scalar(90.0), scalar(80.0))
    );
    assert_eq!(
        fri08_c05_composition_output(&first, 2).size,
        Size::new(scalar(11.0), scalar(21.0))
    );
}

#[test]
fn fri08_c05_composition_reliability_cache_errors_transactions_and_rounding_are_stable() {
    assert_fri08_c04_standalone_cache_and_failures_are_atomic::<f32>();
    assert_fri08_c04_standalone_cache_and_failures_are_atomic::<f64>();
    assert_fri08_c04_overflow_atomic_failures::<f32>();
    assert_fri08_c04_overflow_atomic_failures::<f64>();
    fri08_c05_composition_fractional_rounding::<f32>();
    fri08_c05_composition_fractional_rounding::<f64>();
}

#[test]
fn fri08_c05_composition_overflow_scrollbar_settles_once_in_both_scalar_lanes() {
    assert_fri08_c04_overflow_inline_scroll_range::<f32>();
    assert_fri08_c04_overflow_inline_scroll_range::<f64>();
}

#[test]
fn fri08_c05_composition_inherited_baseline_role_regression_preserves_fri06_contract() {
    assert_fri08_c04_baseline_area_topology_controls::<f32>();
    assert_fri08_c04_baseline_area_topology_controls::<f64>();
}

fn fri08_c05_composition_absolute_no_demand_control<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponentOf::px(scalar(40.0))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(20.0))],
                grid_auto_rows: vec![TrackComponentOf::px(scalar(20.0))],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                position: Position::Absolute,
                grid_column: GridPlacement::try_span(4).expect("out-of-flow span control"),
                size: Size::new(
                    PreferredSizeOf::px(scalar(7.0)),
                    PreferredSizeOf::px(scalar(5.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(3, NodeInputOf::default());
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("absolute-control viewport"),
    )
    .expect("FRI-10 negative control remains outside implicit demand");
    assert_eq!(
        fri08_c05_composition_output(&batch, 1).size,
        Size::new(scalar(40.0), scalar(20.0))
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 2).size,
        Size::new(scalar(7.0), scalar(5.0))
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 3).location,
        Point::ZERO
    );
}

#[test]
fn fri08_c05_composition_absolute_item_preserves_source_without_grid_demand() {
    fri08_c05_composition_absolute_no_demand_control::<f32>();
    fri08_c05_composition_absolute_no_demand_control::<f64>();
}

fn fri08_c05_composition_non_grid_block_control<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(scalar(60.0)), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(scalar(20.0)),
                    PreferredSizeOf::px(scalar(12.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("non-grid control viewport"),
    )
    .expect("default block composition succeeds");
    assert_eq!(
        fri08_c05_composition_output(&batch, 1).size,
        Size::new(scalar(60.0), scalar(12.0))
    );
    assert_eq!(
        fri08_c05_composition_output(&batch, 2).size,
        Size::new(scalar(20.0), scalar(12.0))
    );
}

#[test]
fn fri08_c05_composition_non_grid_default_block_behavior_is_unchanged() {
    fri08_c05_composition_non_grid_block_control::<f32>();
    fri08_c05_composition_non_grid_block_control::<f64>();
}

#[test]
fn fri08_c03_containing_block_rtl_overflow_keeps_negative_physical_range() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                grid_template_rows: vec![TrackComponent::px(40.0)],
                overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto)
                    .expect("auto overflow pair is canonical"),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::percent(1.2), PreferredSize::px(40.0)),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, Size::ZERO);
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequest::viewport(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT))
            .expect("RTL overflow containing-block viewport"),
    )
    .expect("RTL overflow lanes layout succeeds");
    let container = fri08_c01_placement_output(&batch, 1);
    let child = fri08_c01_placement_output(&batch, 2);
    assert_eq!(child.location, Point::new(-20.0, 0.0));
    assert_eq!(child.size, Size::new(120.0, 40.0));
    let range = container
        .scroll_geometry
        .expect("grid-lanes container publishes scroll geometry")
        .physical_range()
        .x();
    assert_eq!((range.minimum(), range.maximum()), (-20.0, 0.0));
}

fn assert_fri08_c03_nested_cache_and_failures_are_atomic<S: LayoutScalar>() {
    let mut tree = Fri08C03NestedAtomicTree::<S>::new();
    let request = Fri08C03NestedAtomicTree::<S>::request();
    let cold = compute_layout(&tree, 1, request).expect("cold recursive lanes layout succeeds");
    assert_eq!(
        fri08_c01_placement_output(&cold, 1).size.width,
        S::from_f64(60.0)
    );
    assert_eq!(
        cold.final_entries()
            .iter()
            .map(LayoutOutputEntryOf::node)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7, 8],
        "recursive collection and item order preserve source-associated publication"
    );
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();
    cold.apply_to(&mut tree)
        .expect("recursive lanes batch commit is infallible");

    tree.cache_queries.borrow_mut().clear();
    tree.measurement_requests.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm recursive lanes layout succeeds");
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert!(
        tree.cache_queries
            .borrow()
            .iter()
            .any(|(node, hit)| matches!(node, 4 | 5) && *hit),
        "warm recursive layout must reuse a committed descendant cache entry"
    );

    for mode in [
        Fri08C03NestedMeasureMode::ProviderError,
        Fri08C03NestedMeasureMode::NonFinite,
    ] {
        tree.measure_mode.set(mode);
        tree.measurement_requests.borrow_mut().clear();
        let retained_before_failure = tree.retained.clone();
        let error = compute_layout_invalidated(&tree, 1, request, &[4])
            .expect_err("recursive descendant failure returns no partial batch");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(4));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        match mode {
            Fri08C03NestedMeasureMode::ProviderError => assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::Measurement(Fri08C03NestedMeasureError::Provider)
            )),
            Fri08C03NestedMeasureMode::NonFinite => {
                let LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(
                    invalid,
                )) = error.kind()
                else {
                    panic!("expected invalid recursive measurement output, got {error:?}");
                };
                assert_eq!(invalid.axis(), PhysicalAxis::Horizontal);
            }
            Fri08C03NestedMeasureMode::Values => unreachable!("failure loop excludes values"),
        }
        assert_eq!(tree.retained, retained_before_failure);
        assert!(
            tree.measurement_requests
                .borrow()
                .iter()
                .any(|(node, _)| *node == 4),
            "the invalidated recursive descendant reached its failing provider"
        );
    }

    tree.measure_mode.set(Fri08C03NestedMeasureMode::Values);
    let recovered = compute_layout_invalidated(&tree, 1, request, &[4])
        .expect("recursive layout remains retryable after both failures");
    assert_eq!(recovered.final_entries(), cold_final);
    recovered
        .apply_to(&mut tree)
        .expect("recovered recursive lanes batch commits atomically");
    assert_eq!(tree.retained.final_outputs.len(), 8);
}

#[test]
fn fri08_c03_nested_cache_cold_warm_provider_non_finite_and_rollback_are_scalar_stable() {
    assert_fri08_c03_nested_cache_and_failures_are_atomic::<f32>();
    assert_fri08_c03_nested_cache_and_failures_are_atomic::<f64>();
}

fn assert_fri08_c04_standalone_cache_and_failures_are_atomic<S: LayoutScalar>() {
    let mut tree = Fri08C03NestedAtomicTree::with_tree(fri08_c04_standalone_nested_tree(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Direction::Rtl,
        MinSizeOf::MIN_CONTENT,
        true,
    ));
    let request = Fri08C03NestedAtomicTree::<S>::request();
    let cold = compute_layout(&tree, 1, request).expect("cold standalone layout succeeds");
    assert_eq!(
        fri08_c01_placement_output(&cold, 1).size,
        Size::new(S::from_f64(40.0), S::from_f64(20.0))
    );
    assert_eq!(
        cold.final_entries()
            .iter()
            .map(LayoutOutputEntryOf::node)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5]
    );
    let cold_final = cold.final_entries().to_vec();
    cold.apply_to(&mut tree)
        .expect("standalone cold batch commits atomically");

    tree.cache_queries.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm standalone layout succeeds");
    assert_eq!(warm.final_entries(), cold_final);
    assert!(
        tree.cache_queries
            .borrow()
            .iter()
            .any(|(node, hit)| matches!(node, 4 | 5) && *hit),
        "warm standalone layout reuses the ordinary descendant cache"
    );

    for mode in [
        Fri08C03NestedMeasureMode::ProviderError,
        Fri08C03NestedMeasureMode::NonFinite,
    ] {
        tree.measure_mode.set(mode);
        tree.measurement_requests.borrow_mut().clear();
        let retained_before_failure = tree.retained.clone();
        let error = compute_layout_invalidated(&tree, 1, request, &[4])
            .expect_err("standalone descendant failure publishes no partial batch");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(4));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        match mode {
            Fri08C03NestedMeasureMode::ProviderError => assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::Measurement(Fri08C03NestedMeasureError::Provider)
            )),
            Fri08C03NestedMeasureMode::NonFinite => assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(_))
            )),
            Fri08C03NestedMeasureMode::Values => unreachable!("failure cases are explicit"),
        }
        assert_eq!(tree.retained, retained_before_failure);
        assert!(
            tree.measurement_requests
                .borrow()
                .iter()
                .any(|(node, _)| *node == 4)
        );
    }

    tree.measure_mode.set(Fri08C03NestedMeasureMode::Values);
    let retry = compute_layout_invalidated(&tree, 1, request, &[4])
        .expect("standalone layout retries after provider failures");
    assert_eq!(retry.final_entries(), cold_final);
}

#[test]
fn fri08_c04_standalone_cache_retry_order_provider_nonfinite_and_rollback_are_scalar_stable() {
    assert_fri08_c04_standalone_cache_and_failures_are_atomic::<f32>();
    assert_fri08_c04_standalone_cache_and_failures_are_atomic::<f64>();
}

fn assert_fri08_c04_baseline_cache_and_failures_are_atomic<S: LayoutScalar>() {
    let case = Fri08C04BaselineFlowCase {
        parent_axis: Fri08C04BaselineParentAxis::Row,
        root_writing_mode: WritingMode::HorizontalTb,
        root_direction: Direction::Ltr,
        child_writing_mode: WritingMode::HorizontalTb,
        child_direction: Direction::Rtl,
    };
    let mut tree = fri08_c04_baseline_area_implicit_tree::<S>(case, AlignItems::Baseline);
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
        .expect("baseline atomic viewport");
    let cold = compute_layout(&tree, 1, request).expect("cold baseline composition succeeds");
    let cold_entries = cold.final_entries().to_vec();
    let cold_coordinates = fri08_c04_baseline_world_coordinate(&cold, case, AlignItems::Baseline);
    cold.apply_to(&mut tree)
        .expect("baseline batch commit is infallible");

    tree.cache_queries.borrow_mut().clear();
    tree.measurement_requests.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm baseline composition succeeds");
    assert_eq!(warm.final_entries(), cold_entries);
    assert_eq!(
        fri08_c04_baseline_world_coordinate(&warm, case, AlignItems::Baseline),
        cold_coordinates
    );
    assert!(
        tree.cache_queries
            .borrow()
            .iter()
            .any(|(node, hit)| matches!(node, 6 | 7) && *hit),
        "warm composition reuses a committed baseline member cache"
    );

    for mode in [
        Fri08C04BaselineMeasureMode::ProviderError,
        Fri08C04BaselineMeasureMode::NonFinite,
    ] {
        tree.measure_mode.set(mode);
        tree.measurement_requests.borrow_mut().clear();
        let retained_before = tree.retained.clone();
        let error = compute_layout_invalidated(&tree, 1, request, &[7])
            .expect_err("baseline member failure returns no completed batch");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(7));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        match mode {
            Fri08C04BaselineMeasureMode::ProviderError => assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::Measurement(Fri08C04BaselineMeasureError::Provider)
            )),
            Fri08C04BaselineMeasureMode::NonFinite => assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(_))
            )),
            Fri08C04BaselineMeasureMode::Values => unreachable!("failure modes are explicit"),
        }
        assert_eq!(tree.retained, retained_before);
        assert!(tree.measurement_requests.borrow().contains(&7));
    }

    tree.measure_mode.set(Fri08C04BaselineMeasureMode::Values);
    let retry = compute_layout_invalidated(&tree, 1, request, &[7])
        .expect("baseline composition retries after provider failures");
    assert_eq!(retry.final_entries(), cold_entries);
    assert_eq!(
        fri08_c04_baseline_world_coordinate(&retry, case, AlignItems::Baseline),
        cold_coordinates
    );
}

#[test]
fn fri08_c04_baseline_cache_error_nonfinite_retry_and_rollback_are_scalar_stable() {
    assert_fri08_c04_baseline_cache_and_failures_are_atomic::<f32>();
    assert_fri08_c04_baseline_cache_and_failures_are_atomic::<f64>();
}

fn fri08_c04_overflow_inline_scroll_output<S: LayoutScalar>() -> NodeOutputOf<S> {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(scalar(50.0)),
                    PreferredSizeOf::px(scalar(50.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Flex,
                overflow: ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll)
                    .expect("scroll pair is normalized"),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(15.0))
                    .expect("finite scrollbar width"),
                ..NodeInputOf::default()
            },
        )
        .measure(2, Size::new(scalar(100.0), scalar(10.0)));
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("max-content viewport is valid"),
    )
    .expect("inline scroll grid layout succeeds");
    fri08_c01_placement_output(&batch, 2)
}

fn assert_fri08_c04_overflow_inline_scroll_range<S: LayoutScalar>() {
    let output = fri08_c04_overflow_inline_scroll_output::<S>();
    let geometry = output
        .scroll_geometry
        .expect("performed grid child retains scroll geometry");
    let range = geometry.physical_range();
    assert_eq!(output.size, Size::splat(S::from_f64(50.0)));
    assert_eq!(range.x().maximum() - range.x().minimum(), S::from_f64(65.0));
    assert_eq!(
        range.y().maximum() - range.y().minimum(),
        S::ZERO,
        "a horizontal overflow with forced gutters must not invent block-axis scroll range"
    );
}

#[test]
fn fri08_c04_overflow_inline_scrollbar_settles_without_block_range_in_both_scalars() {
    assert_fri08_c04_overflow_inline_scroll_range::<f32>();
    assert_fri08_c04_overflow_inline_scroll_range::<f64>();
}

fn assert_fri08_c04_overflow_non_grid_leaf_range<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(50.0)),
                    PreferredSizeOf::px(scalar(50.0)),
                ),
                overflow: ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll)
                    .expect("scroll pair is normalized"),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(15.0))
                    .expect("finite scrollbar width"),
                ..NodeInputOf::default()
            },
        )
        .measure(1, Size::new(scalar(100.0), scalar(10.0)));
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("max-content viewport is valid"),
    )
    .expect("non-grid measured leaf layout succeeds");
    let geometry = fri08_c01_placement_output(&batch, 1)
        .scroll_geometry
        .expect("performed measured leaf retains geometry");
    let range = geometry.physical_range();
    assert_eq!(
        range.y().maximum() - range.y().minimum(),
        S::ZERO,
        "forced horizontal gutter is not block-axis scrollable overflow"
    );
}

#[test]
fn fri08_c04_overflow_canonical_measured_leaf_defect_is_not_grid_specific() {
    assert_fri08_c04_overflow_non_grid_leaf_range::<f32>();
    assert_fri08_c04_overflow_non_grid_leaf_range::<f64>();
}

fn assert_fri08_c04_overflow_non_grid_container_child_range<S: LayoutScalar>(display: Display) {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display,
                size: Size::new(
                    PreferredSizeOf::px(scalar(50.0)),
                    PreferredSizeOf::px(scalar(50.0)),
                ),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(scalar(50.0)),
                    PreferredSizeOf::px(scalar(50.0)),
                ),
                overflow: ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll)
                    .expect("scroll pair is normalized"),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(15.0))
                    .expect("finite scrollbar width"),
                ..NodeInputOf::default()
            },
        )
        .measure(2, Size::new(scalar(100.0), scalar(10.0)));
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("max-content viewport is valid"),
    )
    .unwrap_or_else(|error| panic!("{display:?} child layout succeeds: {error:?}"));
    let geometry = fri08_c01_placement_output(&batch, 2)
        .scroll_geometry
        .expect("performed non-grid child retains geometry");
    let range = geometry.physical_range();
    assert_eq!(
        range.y().maximum() - range.y().minimum(),
        S::ZERO,
        "{display:?} consumes the canonical measured-leaf range"
    );
}

#[test]
fn fri08_c04_overflow_block_and_flex_consume_canonical_measured_leaf_range() {
    for display in [Display::Block, Display::Flex] {
        assert_fri08_c04_overflow_non_grid_container_child_range::<f32>(display);
        assert_fri08_c04_overflow_non_grid_container_child_range::<f64>(display);
    }
}

fn fri08_c04_overflow_ordinary_intrinsic_inline_size<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    overflow: Overflow,
    available_inline: AvailableOf<S>,
) -> S {
    let scalar = S::from_f64;
    let flow_axes = FlowAxes::new(writing_mode, direction);
    let measured = flow_axes.physical_size(LogicalSizeOf::new(scalar(50.0), scalar(10.0)));
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::InlineGrid,
                writing_mode,
                direction,
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode,
                direction,
                overflow: ComputedOverflow::try_new(overflow, overflow)
                    .expect("overflow pair is normalized"),
                ..NodeInputOf::default()
            },
        )
        .measure(2, measured);
    let available = flow_axes.physical_size(LogicalSizeOf::new(
        available_inline,
        AvailableOf::MAX_CONTENT,
    ));
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(available).expect("intrinsic viewport is valid"),
    )
    .expect("ordinary intrinsic grid layout succeeds");
    flow_axes
        .logical_size(fri08_c01_placement_output(&batch, 1).size)
        .inline
}

fn assert_fri08_c04_overflow_ordinary_intrinsic_phases<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalRl] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for overflow in [
                Overflow::Visible,
                Overflow::Clip,
                Overflow::Hidden,
                Overflow::Scroll,
                Overflow::Auto,
            ] {
                let expected_maximum = if matches!(overflow, Overflow::Visible | Overflow::Clip) {
                    scalar(50.0)
                } else {
                    S::ZERO
                };
                assert_eq!(
                    fri08_c04_overflow_ordinary_intrinsic_inline_size::<S>(
                        writing_mode,
                        direction,
                        overflow,
                        AvailableOf::MAX_CONTENT,
                    ),
                    expected_maximum,
                    "{writing_mode:?} {direction:?} {overflow:?} ordinary automatic minimum"
                );
                let expected_minimum = if overflow == Overflow::Visible {
                    scalar(50.0)
                } else {
                    S::ZERO
                };
                assert_eq!(
                    fri08_c04_overflow_ordinary_intrinsic_inline_size::<S>(
                        writing_mode,
                        direction,
                        overflow,
                        AvailableOf::MIN_CONTENT,
                    ),
                    expected_minimum,
                    "{writing_mode:?} {direction:?} {overflow:?} automatic minimum"
                );
            }
        }
    }
}

#[test]
fn fri08_c04_overflow_ordinary_auto_minimum_and_max_content_eligibility_are_distinct() {
    assert_fri08_c04_overflow_ordinary_intrinsic_phases::<f32>();
    assert_fri08_c04_overflow_ordinary_intrinsic_phases::<f64>();
}

fn fri08_c04_overflow_hidden_subgrid_batch<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
) -> CompletedLayoutBatchOf<u32, S> {
    let scalar = S::from_f64;
    let flow_axes = FlowAxes::new(writing_mode, direction);
    let fixed = flow_axes
        .physical_size(LogicalSizeOf::new(scalar(50.0), scalar(50.0)))
        .map(PreferredSizeOf::px);
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInputOf {
                display: Display::InlineGrid,
                writing_mode,
                direction,
                grid_template_columns: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode,
                direction,
                size: fixed.clone(),
                grid_column: GridPlacement::try_line(1).expect("first column"),
                grid_row: GridPlacement::try_line(1).expect("first row"),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Grid,
                writing_mode,
                direction,
                overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden)
                    .expect("hidden pair is normalized"),
                grid_template_rows: subgrid_track_of(),
                grid_column: GridPlacement::try_line(2).expect("second column"),
                grid_row: GridPlacement::try_line_span(1, 2).expect("two inherited rows"),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                writing_mode,
                direction,
                size: fixed,
                grid_row: GridPlacement::try_line(2).expect("second inherited row"),
                ..NodeInputOf::default()
            },
        );
    compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("max-content viewport is valid"),
    )
    .expect("hidden inherited subgrid layout succeeds")
}

fn assert_fri08_c04_overflow_hidden_subgrid_intrinsic_size<S: LayoutScalar>() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let batch = fri08_c04_overflow_hidden_subgrid_batch::<S>(writing_mode, direction);
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let root = fri08_c01_placement_output(&batch, 1);
            let subgrid = fri08_c01_placement_output(&batch, 3);
            assert_eq!(
                flow_axes.logical_size(root.size),
                LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(100.0)),
                "hidden overflow preserves the standalone-axis max-content contribution"
            );
            assert_eq!(
                flow_axes.logical_size(subgrid.size),
                LogicalSizeOf::new(S::from_f64(50.0), S::from_f64(100.0))
            );
        }
    }
}

#[test]
fn fri08_c04_overflow_hidden_subgrid_preserves_standalone_intrinsic_size_all_flows_and_scalars() {
    assert_fri08_c04_overflow_hidden_subgrid_intrinsic_size::<f32>();
    assert_fri08_c04_overflow_hidden_subgrid_intrinsic_size::<f64>();
}

fn assert_fri08_c04_overflow_atomic_failures<S: LayoutScalar>() {
    let mut tree = Fri08C03NestedAtomicTree::<S>::new();
    for node in [2, 3, 4, 5] {
        let mut style = tree.tree.node_input(node).clone();
        style.overflow = match node {
            2 => ComputedOverflow::try_new(Overflow::Visible, Overflow::Clip),
            3 => ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden),
            4 => ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll),
            5 => ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto),
            _ => unreachable!("the overflow matrix has four selected nodes"),
        }
        .expect("overflow pair is normalized");
        style.item_order = ItemOrder::new(if node % 2 == 0 { 7 } else { -7 });
        tree.tree
            .insert_input(node, LayoutInputOf::box_input(style));
    }
    let request = Fri08C03NestedAtomicTree::<S>::request();
    let cold = compute_layout(&tree, 1, request).expect("cold overflow composition succeeds");
    let cold_final = cold.final_entries().to_vec();
    cold.apply_to(&mut tree)
        .expect("cold overflow batch commits atomically");

    tree.cache_queries.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm overflow composition succeeds");
    assert_eq!(warm.final_entries(), cold_final);
    assert!(tree.cache_queries.borrow().iter().any(|(_, hit)| *hit));

    for mode in [
        Fri08C03NestedMeasureMode::ProviderError,
        Fri08C03NestedMeasureMode::NonFinite,
    ] {
        tree.measure_mode.set(mode);
        let retained = tree.retained.clone();
        let error = compute_layout_invalidated(&tree, 1, request, &[4])
            .expect_err("descendant failure returns no completed overflow batch");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(4));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        assert_eq!(tree.retained, retained, "failure rolls back retained state");
    }
}

#[test]
fn fri08_c04_overflow_order_cache_provider_nonfinite_and_rollback_are_scalar_stable() {
    assert_fri08_c04_overflow_atomic_failures::<f32>();
    assert_fri08_c04_overflow_atomic_failures::<f64>();
}

fn assert_fri08_c02r_lanes_cold_warm_cache<S: LayoutScalar>() {
    let (tree, flow_axes, _) = fri08_c02_track_mix_tree(
        Display::GridLanes,
        Fri08C02TrackAxis::Columns,
        WritingMode::HorizontalTb,
        (50.0, 0.0),
        Some(200.0),
        vec![fri08_c02_flex_track::<S>(1.0)],
        &[20.0, 0.0],
    );
    let mut tree = Fri08C06RAtomicTree::new(tree);
    let request =
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(200.0))))
            .expect("finite lanes track-phase viewport");
    let cold = compute_layout(&tree, 1, request).expect("cold lanes sizing completes");
    assert_eq!(
        (0..2)
            .map(|index| {
                flow_axes
                    .logical_size(fri08_c01_placement_output(&cold, index + 2).size)
                    .inline
            })
            .collect::<Vec<_>>(),
        [S::from_f64(20.0), S::from_f64(180.0)]
    );
    assert_eq!(cold.final_entries().len(), 3);
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();
    cold.apply_to(&mut tree)
        .expect("cold lanes sizing batch commits atomically");

    tree.cache_queries.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm lanes sizing completes");
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert!(
        tree.cache_queries.borrow().iter().any(|(_, hit)| *hit),
        "warm lanes sizing must reuse committed cache state"
    );
}

#[test]
fn fri08_c02r_lanes_track_phase_completed_batches_are_cold_warm_cache_equivalent() {
    assert_fri08_c02r_lanes_cold_warm_cache::<f32>();
    assert_fri08_c02r_lanes_cold_warm_cache::<f64>();
}

fn fri08_c02_auto_fit_inherited_context(reversed: bool) -> GridParentContext<f32, u32> {
    let parent_style = NodeInput {
        display: Display::Grid,
        ..NodeInput::DEFAULT
    };
    let child_style = NodeInput {
        display: Display::Grid,
        direction: if reversed {
            Direction::Rtl
        } else {
            Direction::Ltr
        },
        overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
        grid_template_columns: vec![empty_subgrid_track()],
        grid_template_rows: vec![TrackComponent::px(20.0)],
        align_items: Some(AlignItems::Baseline),
        ..NodeInput::DEFAULT
    };
    let item = SubgridItemReport {
        node: 1_u32,
        column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
        row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
    };
    assert_eq!(item.column.mapping.reversed, reversed);
    let parent_geometry = UsedGridAxisGeometryOf::new(
        vec![40.0, 0.0, 40.0, 40.0],
        vec![false, true, false, false],
        10.0,
    );
    subgrid_child_parent_context_with_geometry(
        SubgridChildParentContextInput {
            item,
            child_style: &child_style,
            area: GridArea {
                column: 0,
                row: 0,
                column_end: 4,
                row_end: 1,
                size: LogicalSizeOf::new(140.0, 20.0),
            },
            content_box_size: Size::new(140.0, 20.0),
            columns: parent_geometry.sizes(),
            rows: &[20.0],
            gap: LogicalSizeOf::new(10.0, 0.0),
            parent_named_columns: &NamedGridLines::new(GridAxisKind::Column, 4),
            parent_named_rows: &NamedGridLines::new(GridAxisKind::Row, 1),
            parent_area_facts: None,
            parent_baseline_groups: &GridBaselineGroups {
                columns: vec![
                    tagged_group(PhysicalAxis::Horizontal, Some(7.0), Some(3.0)),
                    TrackBaselineGroup::default(),
                    tagged_group(PhysicalAxis::Horizontal, Some(11.0), Some(5.0)),
                    tagged_group(PhysicalAxis::Horizontal, Some(13.0), Some(6.0)),
                ],
                rows: vec![TrackBaselineGroup::default()],
            },
            margin: Edges::all(Some(0.0)),
            border: Edges::ZERO,
            padding: Edges::ZERO,
        },
        Some(&parent_geometry),
        None,
    )
    .expect("collapsed used geometry remains inheritable")
}

#[test]
fn fri08_c02_auto_fit_inherited_subgrid_slices_and_reverses_geometry_with_baseline_and_scroll_overflow()
 {
    for reversed in [false, true] {
        let context = fri08_c02_auto_fit_inherited_context(reversed);
        let columns = context.columns.as_ref().expect("column subgrid context");
        assert_eq!(columns.geometry.total_extent(), 140.0);
        assert_eq!(columns.geometry.span_extent(0, 4), 140.0);
        assert_eq!(columns.geometry.active_gap_total(), 20.0);
        assert_eq!(
            columns.geometry.collapsed(),
            if reversed {
                &[false, false, true, false]
            } else {
                &[false, true, false, false]
            }
        );
        let inherited_baselines = columns
            .major_baselines
            .iter()
            .filter_map(|baseline| baseline.map(|baseline| baseline.coordinate()))
            .collect::<Vec<_>>();
        assert_eq!(
            inherited_baselines,
            if reversed {
                vec![13.0, 11.0, 7.0]
            } else {
                vec![7.0, 11.0, 13.0]
            }
        );

        let mut tree = OracleTree::new()
            .children(1, [2])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    direction: if reversed {
                        Direction::Rtl
                    } else {
                        Direction::Ltr
                    },
                    overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
                    grid_template_columns: vec![empty_subgrid_track()],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    align_items: Some(AlignItems::Baseline),
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    grid_column: GridPlacement::try_lines(1, 5).expect("full inherited span"),
                    overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
                    ..NodeInput::DEFAULT
                },
            )
            .measure(2, baseline_measure(140.0, 20.0, Some(7.0), None));
        let output = compute_grid_with_context(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::new(Some(140.0), Some(20.0)),
                Size::new(Some(140.0), Some(20.0)),
                ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::Grid,
                ),
                Size::new(Available::Definite(140.0), Available::Definite(20.0)),
            ),
            context,
        )
        .expect("inherited collapsed geometry layout");
        assert_eq!(
            tree.layout(2).expect("subgrid child layout").size.width,
            140.0
        );
        assert!(output.scroll_geometry.is_some());
        assert_eq!(output.first_baselines.y, Some(7.0));
    }
}

#[test]
fn fri08_c02_auto_fit_public_parent_projects_reversed_subgrid_baseline_and_overflow_past_collapsed_lines()
 {
    let layout = |writing_mode| {
        let tree = PublicLayoutTreeOf::<f32>::new()
            .children(1, [2])
            .children(2, [3])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(190.0), PreferredSize::px(40.0)),
                    grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                    grid_template_rows: vec![TrackComponent::px(40.0)],
                    gap: Size::new(Length::px(10.0), Length::ZERO),
                    justify_content: Some(AlignContent::Center),
                    align_items: Some(AlignItems::Baseline),
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    display: Display::Grid,
                    writing_mode,
                    grid_column: GridPlacement::try_line(3).expect("retained third line"),
                    grid_row: GridPlacement::try_line(1).expect("single parent row"),
                    grid_template_columns: vec![TrackComponent::px(40.0)],
                    grid_template_rows: vec![empty_subgrid_track()],
                    align_items: Some(AlignItems::Baseline),
                    overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                3,
                NodeInput {
                    align_self: Some(AlignItems::Baseline),
                    overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                    ..NodeInput::DEFAULT
                },
            )
            .measure(3, Size::new(60.0, 20.0));
        let batch = compute_layout(
            &tree,
            1,
            LayoutRootRequest::viewport(Size::new(
                Available::Definite(190.0),
                Available::Definite(40.0),
            ))
            .expect("finite public auto-fit viewport"),
        )
        .expect("public auto-fit subgrid layout");
        (
            fri08_c01_placement_output(&batch, 2),
            fri08_c01_placement_output(&batch, 3),
        )
    };

    let (forward, forward_child) = layout(WritingMode::VerticalLr);
    let (reversed, reversed_child) = layout(WritingMode::VerticalRl);
    assert_eq!((forward.location.x, forward.size.width), (75.0, 40.0));
    assert_eq!((reversed.location.x, reversed.size.width), (75.0, 40.0));
    let forward_scroll = forward.scroll_geometry.expect("forward subgrid overflow");
    let reversed_scroll = reversed.scroll_geometry.expect("reversed subgrid overflow");
    assert_eq!(
        (
            forward_scroll.physical_range().x().minimum(),
            forward_scroll.physical_range().x().maximum(),
            reversed_scroll.physical_range().x().minimum(),
            reversed_scroll.physical_range().x().maximum(),
        ),
        (0.0, 20.0, -20.0, 0.0),
    );
    assert_eq!(forward_child.size, reversed_child.size);
    assert_eq!(
        (forward_child.location.x, reversed_child.location.x),
        (0.0, -20.0),
        "baseline-aligned descendant mirrors across the inherited reversed axis",
    );
}

#[test]
fn fri08_c02_auto_fit_public_decreasing_subgrid_preserves_baseline_location_and_overflow_geometry()
{
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .children(2, [3, 4, 5])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(190.0), PreferredSize::px(40.0)),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Center),
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_column: GridPlacement::try_line(3).expect("retained third line"),
                grid_row: GridPlacement::try_line(1).expect("single parent row"),
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                gap: Size::new(Length::px(20.0), Length::ZERO),
                align_items: Some(AlignItems::Baseline),
                overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(17.0), PreferredSize::px(10.0)),
                grid_column: GridPlacement::try_line(1).expect("single subgrid column"),
                grid_row: GridPlacement::try_line(1).expect("single inherited row"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                grid_column: GridPlacement::try_line(1).expect("single subgrid column"),
                grid_row: GridPlacement::try_line(1).expect("single inherited row"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            5,
            NodeInput {
                align_self: Some(AlignItems::Start),
                grid_column: GridPlacement::try_line(1).expect("single subgrid column"),
                grid_row: GridPlacement::try_line(1).expect("single inherited row"),
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                ..NodeInput::DEFAULT
            },
        )
        .measure(5, Size::new(60.0, 17.0));

    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequest::viewport(Size::new(
            Available::Definite(190.0),
            Available::Definite(40.0),
        ))
        .expect("finite public auto-fit viewport"),
    )
    .expect("public decreasing auto-fit subgrid layout");
    let subgrid = fri08_c01_placement_output(&batch, 2);
    let baseline_target = fri08_c01_placement_output(&batch, 3);
    let baseline_peer = fri08_c01_placement_output(&batch, 4);
    let descendant = fri08_c01_placement_output(&batch, 5);
    let overflow = subgrid
        .scroll_geometry
        .expect("decreasing subgrid publishes scroll overflow")
        .physical_range()
        .x();

    assert_eq!((subgrid.location.x, subgrid.size.width), (75.0, 40.0));
    assert_eq!(baseline_peer.location.x - baseline_target.location.x, 7.0);
    assert_eq!((overflow.minimum(), overflow.maximum()), (-20.0, 0.0));
    assert_eq!(descendant.location.x, -20.0);
}

#[test]
fn fri08_c01_placement_definite_line_plus_span_capacity_is_typed_and_retryable() {
    let overflowing = PublicLayoutTreeOf::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::line_span(
                    GridLine::new(isize::MAX).expect("largest positive grid line"),
                    GridSpan::new(usize::MAX).expect("largest grid span"),
                ),
                grid_row: GridPlacement::try_line(1).expect("first row"),
                ..NodeInput::DEFAULT
            },
        );

    let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compute_layout(&overflowing, 1, fri08_c01_placement_request())
    }));
    let error = match attempt {
        Ok(Err(error)) => error,
        Ok(Ok(_)) => panic!("capacity boundary must not publish a completed batch"),
        Err(_) => panic!("capacity boundary must return the typed public error instead of panic"),
    };
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    assert_eq!(
        error.kind(),
        &LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidBlockScrollGeometry,)
    );
    let retry_error = compute_layout(&overflowing, 1, fri08_c01_placement_request())
        .expect_err("the same immutable tree remains retryable after capacity failure");
    assert_eq!(retry_error.site(), LayoutErrorSiteOf::Node(1));
    assert_eq!(retry_error.operation(), LayoutOperation::ChildLayout);

    let retry = PublicLayoutTreeOf::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT);
    assert!(compute_layout(&retry, 1, fri08_c01_placement_request()).is_ok());
}

fn fri06_mr02_scroll_padding_cases<S: LayoutScalar>() -> [(ScrollPaddingOf<S>, Edges<S>); 2] {
    let value = |value| {
        ScrollPaddingValueOf::value(
            LengthPercentageOf::px(S::from_f64(value)).expect("test scroll padding is finite"),
        )
    };

    [
        (
            ScrollPaddingOf::new(
                value(11.0),
                ScrollPaddingValueOf::AUTO,
                value(33.0),
                ScrollPaddingValueOf::AUTO,
            ),
            Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO),
        ),
        (
            ScrollPaddingOf::new(
                ScrollPaddingValueOf::AUTO,
                value(22.0),
                ScrollPaddingValueOf::AUTO,
                value(44.0),
            ),
            Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
        ),
    ]
}

fn fri06_mr02_scroll_padding_input<S: LayoutScalar>(size: Size<S>) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        size.map(Some),
        size.map(Some),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        size.map(AvailableOf::definite),
    )
}

fn assert_fri06_mr02_scroll_padding_grid<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    for (scroll_padding, expected) in fri06_mr02_scroll_padding_cases() {
        let style = NodeInputOf::<S> {
            display: Display::Grid,
            size: Size::new(
                PreferredSizeOf::px(size.width),
                PreferredSizeOf::px(size.height),
            ),
            scroll_padding,
            ..NodeInputOf::default()
        };
        let mut tree = OracleTreeOf::<S>::new().children(0, []).style(0, style);
        let output = compute_grid(&mut tree, 0, fri06_mr02_scroll_padding_input(size))
            .expect("grid scroll-padding characterization succeeds");
        let geometry = output
            .scroll_geometry
            .expect("performed grid layout emits geometry");

        assert_eq!(geometry.resolved_scroll_padding(), expected);
    }
}

fn assert_fri06_mr02_scroll_padding_grid_child<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    for (scroll_padding, expected) in fri06_mr02_scroll_padding_cases() {
        let style = NodeInputOf::<S> {
            scroll_padding,
            ..NodeInputOf::default()
        };
        let geometry = super::child::retained_grid_child_scroll_geometry(
            &grid_item_projection!(&style),
            size,
            size,
            Edges::ZERO,
            Edges::ZERO,
            None,
        )
        .expect("grid-child scroll-padding characterization succeeds");

        assert_eq!(geometry.resolved_scroll_padding(), expected);
    }
}

#[test]
fn fri06_mr02_scroll_padding_grid_preserves_auto_and_value_on_each_physical_edge() {
    assert_fri06_mr02_scroll_padding_grid::<f32>();
    assert_fri06_mr02_scroll_padding_grid::<f64>();
}

#[test]
fn fri06_mr02_scroll_padding_grid_child_preserves_auto_and_value_on_each_physical_edge() {
    assert_fri06_mr02_scroll_padding_grid_child::<f32>();
    assert_fri06_mr02_scroll_padding_grid_child::<f64>();
}

fn fri05_c05_grid_sizing_tree(
    display: Display,
    writing_mode: WritingMode,
    direction: Direction,
    overflow: ComputedOverflow,
    item_is_replaced: bool,
    item_size: Size<PreferredSize>,
    grid_auto_flow: GridAutoFlow,
) -> OracleTree {
    OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display,
                writing_mode,
                direction,
                grid_auto_flow,
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                overflow,
                item_is_replaced,
                size: item_size,
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(20.0, 30.0), Size::new(80.0, 90.0)),
        )
}

#[test]
fn fri05_c05_subgrid_geometry_settles_local_auto_and_preserves_parent_local_output() {
    let mut tree = OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(7.0).unwrap(),
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::px(100.0)],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                grid_template_columns: vec![TrackComponent::Subgrid(SubgridTrack {
                    name_components: Vec::new(),
                })],
                grid_template_rows: vec![TrackComponent::Subgrid(SubgridTrack {
                    name_components: Vec::new(),
                })],
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                position: Position::Relative,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                inset: Edges::new(
                    LengthAuto::px(95.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(95.0),
                ),
                ..NodeInput::default()
            },
        );

    compute_grid(
        &mut tree,
        0,
        fri05_c05_grid_sizing_input(Size::splat(Some(100.0))),
    )
    .expect("nested inherited subgrid computes");

    let subgrid = tree.layout(1).expect("subgrid output is staged");
    let geometry = subgrid
        .scroll_geometry
        .expect("performed inherited subgrid publishes canonical geometry");
    assert_eq!(geometry.border_box().size(), subgrid.size);
    assert_eq!(geometry.target().border_box(), geometry.border_box());
    assert!(geometry.gutters().right().is_some());
    assert!(geometry.gutters().bottom().is_some());
    assert_eq!(geometry.content_box().size(), Size::splat(90.0));

    let performed = tree
        .inputs(2)
        .iter()
        .filter(|input| input.run_mode() == RunMode::PerformLayout)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        performed.len(),
        4,
        "initial layout and baseline refresh each settle the subgrid locally"
    );
    assert_eq!(
        performed
            .iter()
            .map(ComputeInputOf::containing_auto_scrollbar_pass)
            .collect::<Vec<_>>(),
        vec![
            crate::scroll::SettledAutoScrollbarState::INITIAL,
            crate::scroll::SettledAutoScrollbarState::new(true, true),
            crate::scroll::SettledAutoScrollbarState::INITIAL,
            crate::scroll::SettledAutoScrollbarState::new(true, true),
        ]
    );
    assert!(performed.iter().all(|input| {
        input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
    }));
}

fn assert_fri05_c05_grid_round_cache_for_display<S: LayoutScalar>(display: Display)
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S> + Round,
{
    let scalar = S::from_f64;
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let scroll_margin =
        ScrollMarginOf::try_new(scalar(1.25), scalar(2.25), scalar(3.25), scalar(4.25)).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let mut tree = OracleTreeOf::<S>::new().children(0, []).style(
        0,
        NodeInputOf {
            display,
            size: Size::new(
                PreferredSizeOf::px(scalar(100.4)),
                PreferredSizeOf::px(scalar(80.4)),
            ),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: ScrollbarWidthOf::try_new(scalar(7.4)).unwrap(),
            grid_template_columns: vec![TrackComponentOf::px(scalar(130.6))],
            grid_template_rows: vec![TrackComponentOf::px(scalar(110.6))],
            justify_content: Some(AlignContent::End),
            align_content: Some(AlignContent::Center),
            scroll_margin,
            scroll_snap_align: snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInputOf::default()
        },
    );
    let input = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(scalar(100.4)), Some(scalar(80.4))),
        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
        Size::new(
            AvailableOf::Definite(scalar(100.4)),
            AvailableOf::Definite(scalar(80.4)),
        ),
    );
    let cold = compute_grid(&mut tree, 0, input).expect("scalar grid geometry computes");
    let geometry = cold.scroll_geometry.expect("cold geometry is present");
    let format = match display {
        Display::Grid => "ordinary grid",
        Display::GridLanes => "grid lanes",
        _ => unreachable!("focused round/cache evidence accepts only grid-family displays"),
    };

    let assert_geometry = |phase: &str,
                           output: ComputeOutputOf<S>,
                           expected_border: Size<S>,
                           expected_bar: S,
                           expected_content_size: Size<S>,
                           expected_range: (S, S, S, S)| {
        let geometry = output
            .scroll_geometry
            .unwrap_or_else(|| panic!("{format} {phase} geometry is present"));
        let expected_scrollport = Size::new(
            expected_border.width - expected_bar,
            expected_border.height - expected_bar,
        );
        let expected_right = ScrollRectOf::try_new(
            Point::new(expected_scrollport.width, S::ZERO),
            Size::new(expected_bar, expected_scrollport.height),
        )
        .unwrap();
        let expected_bottom = ScrollRectOf::try_new(
            Point::new(S::ZERO, expected_scrollport.height),
            Size::new(expected_scrollport.width, expected_bar),
        )
        .unwrap();
        let expected_overflow = ScrollRectOf::try_new(Point::ZERO, expected_border).unwrap();
        let range = geometry.physical_range();

        assert_eq!(geometry.flow_axes(), flow_axes, "{format} {phase} axes");
        assert_eq!(
            geometry.used_overflow_x(),
            Overflow::Scroll,
            "{format} {phase} x"
        );
        assert_eq!(
            geometry.used_overflow_y(),
            Overflow::Scroll,
            "{format} {phase} y"
        );
        assert_eq!(
            geometry.border_box(),
            ScrollRectOf::try_new(Point::ZERO, expected_border).unwrap(),
            "{format} {phase} border"
        );
        assert_eq!(
            geometry.padding_box(),
            geometry.border_box(),
            "{format} {phase} padding box"
        );
        assert_eq!(
            geometry.content_box(),
            ScrollRectOf::try_new(Point::ZERO, expected_scrollport).unwrap(),
            "{format} {phase} content box"
        );
        assert_eq!(
            geometry.scrollport(),
            ScrollRectOf::try_new(Point::ZERO, expected_scrollport).unwrap(),
            "{format} {phase} scrollport"
        );
        assert_eq!(
            geometry.gutters().top(),
            None,
            "{format} {phase} top gutter"
        );
        assert_eq!(
            geometry.gutters().right(),
            Some(expected_right),
            "{format} {phase} right gutter"
        );
        assert_eq!(
            geometry.gutters().bottom(),
            Some(expected_bottom),
            "{format} {phase} bottom gutter"
        );
        assert_eq!(
            geometry.gutters().left(),
            None,
            "{format} {phase} left gutter"
        );
        assert_eq!(
            geometry.scrollbar_size(),
            Size::splat(expected_bar),
            "{format} {phase} scrollbar size"
        );
        assert_eq!(
            (
                range.x().minimum(),
                range.x().maximum(),
                range.y().minimum(),
                range.y().maximum()
            ),
            expected_range,
            "{format} {phase} alignment-bounded physical range"
        );
        assert_eq!(
            geometry.scrollable_overflow(),
            expected_overflow,
            "{format} {phase} complete overflow"
        );
        assert_eq!(
            geometry.canonical_content_size().unwrap(),
            expected_overflow.size(),
            "{format} {phase} canonical content extent"
        );
        assert_eq!(
            output.content_size, expected_content_size,
            "{format} {phase} output content size"
        );
        assert_eq!(
            geometry.target().border_box(),
            ScrollRectOf::try_new(Point::ZERO, expected_border).unwrap(),
            "{format} {phase} target border"
        );
        assert_eq!(
            geometry.target().flow_axes(),
            flow_axes,
            "{format} {phase} target axes"
        );
        assert_eq!(
            geometry.target().scroll_margin(),
            scroll_margin,
            "{format} {phase} target margin"
        );
        assert_eq!(
            geometry.target().snap_align(),
            snap_align,
            "{format} {phase} target alignment"
        );
        assert_eq!(
            geometry.target().snap_stop(),
            ScrollSnapStop::Always,
            "{format} {phase} target stop"
        );
    };

    assert_geometry(
        "cold",
        cold,
        Size::new(scalar(100.4), scalar(80.4)),
        scalar(7.4),
        Size::new(scalar(130.6), scalar(110.6)),
        (
            S::ZERO - (scalar(130.6) - (scalar(100.4) - scalar(7.4))),
            S::ZERO,
            S::ZERO - (scalar(110.6) - (scalar(80.4) - scalar(7.4))) / scalar(2.0),
            S::ZERO,
        ),
    );
    let mut cache = CacheOf::<S>::new();
    cache.store_with_context(&input, CacheKeyContext::new(), cold);
    let warm = cache
        .get_with_context(&input, CacheKeyContext::new())
        .expect("warm cache returns grid output");
    assert_eq!(warm, cold);
    assert_eq!(warm.scroll_geometry, Some(geometry));
    assert_geometry(
        "warm",
        warm,
        Size::new(scalar(100.4), scalar(80.4)),
        scalar(7.4),
        Size::new(scalar(130.6), scalar(110.6)),
        (
            S::ZERO - (scalar(130.6) - (scalar(100.4) - scalar(7.4))),
            S::ZERO,
            S::ZERO - (scalar(110.6) - (scalar(80.4) - scalar(7.4))) / scalar(2.0),
            S::ZERO,
        ),
    );

    Compute::set_unrounded(
        &mut tree,
        0,
        NodeOutputOf {
            source_index: SourceIndex::ZERO,
            location: Point::ZERO,
            size: cold.size,
            content_size: cold.content_size,
            scroll_geometry: Some(geometry),
            border: Edges::ZERO,
            padding: Edges::ZERO,
            margin: Edges::ZERO,
        },
    );
    round_layout(&mut tree, 0).expect("grid geometry rounds from canonical source");
    let rounded = tree.final_layout(0).expect("rounded output is staged");
    let rounded_geometry = rounded
        .scroll_geometry
        .expect("rounding retains canonical grid geometry");
    assert_geometry(
        "rounded",
        ComputeOutputOf {
            size: rounded.size,
            content_size: rounded.content_size,
            scroll_geometry: Some(rounded_geometry),
            ..cold
        },
        Size::new(scalar(100.0), scalar(80.0)),
        scalar(7.0),
        Size::new(scalar(131.0), scalar(111.0)),
        (scalar(-38.0), S::ZERO, scalar(-19.0), S::ZERO),
    );
    assert_eq!(
        rounded.content_box_size(),
        Size::new(scalar(93.0), scalar(73.0)),
        "{format} rounded output content box accessor"
    );
    assert_eq!(
        rounded.scrollbar_size(),
        Size::splat(scalar(7.0)),
        "{format} rounded output scrollbar accessor"
    );

    let measurement_input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(scalar(100.4)), Some(scalar(80.4))),
        Size::new(Some(scalar(100.4)), Some(scalar(80.4))),
        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
        Size::new(
            AvailableOf::Definite(scalar(100.4)),
            AvailableOf::Definite(scalar(80.4)),
        ),
    );
    let measurement = compute_grid(&mut tree, 0, measurement_input)
        .expect("grid-family measurement control computes");
    assert!(
        measurement.scroll_geometry.is_none(),
        "{format} measurement must not publish geometry"
    );
}

#[test]
fn fri05_c05_grid_round_cache_ordinary_and_lanes_match_in_both_scalar_lanes() {
    for display in [Display::Grid, Display::GridLanes] {
        assert_fri05_c05_grid_round_cache_for_display::<f32>(display);
        assert_fri05_c05_grid_round_cache_for_display::<f64>(display);
    }
}

fn assert_fri05_c05_grid_round_cache_subgrid<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S> + Round,
{
    let scalar = S::from_f64;
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let scroll_margin =
        ScrollMarginOf::try_new(scalar(1.25), scalar(2.25), scalar(3.25), scalar(4.25)).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::Start, ScrollSnapAlignValue::End);
    let subgrid_track = || {
        TrackComponentOf::Subgrid(SubgridTrack {
            name_components: Vec::new(),
        })
    };
    let mut tree = OracleTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.4)),
                    PreferredSizeOf::px(scalar(80.4)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(scalar(100.4))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(80.4))],
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(7.4)).unwrap(),
                grid_template_columns: vec![subgrid_track()],
                grid_template_rows: vec![subgrid_track()],
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                scroll_margin,
                scroll_snap_align: snap_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInputOf::default()
            },
        );
    let input = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(scalar(100.4)), Some(scalar(80.4))),
        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
        Size::new(
            AvailableOf::Definite(scalar(100.4)),
            AvailableOf::Definite(scalar(80.4)),
        ),
    );

    let assert_subgrid = |phase: &str,
                          output: NodeOutputOf<S>,
                          expected_border: Size<S>,
                          expected_bar: S,
                          expected_content_size: Size<S>,
                          expected_range: (S, S, S, S)| {
        let geometry = output
            .scroll_geometry
            .unwrap_or_else(|| panic!("nested subgrid {phase} geometry is present"));
        let expected_scrollport = Size::new(
            expected_border.width - expected_bar,
            expected_border.height - expected_bar,
        );
        let border_rect = ScrollRectOf::try_new(Point::ZERO, expected_border).unwrap();
        let scrollport_rect = ScrollRectOf::try_new(Point::ZERO, expected_scrollport).unwrap();
        let right_gutter = ScrollRectOf::try_new(
            Point::new(expected_scrollport.width, S::ZERO),
            Size::new(expected_bar, expected_scrollport.height),
        )
        .unwrap();
        let bottom_gutter = ScrollRectOf::try_new(
            Point::new(S::ZERO, expected_scrollport.height),
            Size::new(expected_scrollport.width, expected_bar),
        )
        .unwrap();
        let range = geometry.physical_range();

        assert_eq!(
            geometry.flow_axes(),
            flow_axes,
            "nested subgrid {phase} axes"
        );
        assert_eq!(
            geometry.used_overflow_x(),
            Overflow::Scroll,
            "nested subgrid {phase} x"
        );
        assert_eq!(
            geometry.used_overflow_y(),
            Overflow::Scroll,
            "nested subgrid {phase} y"
        );
        assert_eq!(
            geometry.border_box(),
            border_rect,
            "nested subgrid {phase} border"
        );
        assert_eq!(
            geometry.padding_box(),
            border_rect,
            "nested subgrid {phase} padding box"
        );
        assert_eq!(
            geometry.content_box(),
            scrollport_rect,
            "nested subgrid {phase} content box"
        );
        assert_eq!(
            geometry.scrollport(),
            scrollport_rect,
            "nested subgrid {phase} scrollport"
        );
        assert_eq!(
            geometry.gutters().top(),
            None,
            "nested subgrid {phase} top gutter"
        );
        assert_eq!(
            geometry.gutters().right(),
            Some(right_gutter),
            "nested subgrid {phase} right gutter"
        );
        assert_eq!(
            geometry.gutters().bottom(),
            Some(bottom_gutter),
            "nested subgrid {phase} bottom gutter"
        );
        assert_eq!(
            geometry.gutters().left(),
            None,
            "nested subgrid {phase} left gutter"
        );
        assert_eq!(
            geometry.scrollbar_size(),
            Size::splat(expected_bar),
            "nested subgrid {phase} scrollbar size"
        );
        assert_eq!(
            (
                range.x().minimum(),
                range.x().maximum(),
                range.y().minimum(),
                range.y().maximum()
            ),
            expected_range,
            "nested subgrid {phase} active-subject physical range"
        );
        assert_eq!(
            geometry.scrollable_overflow(),
            border_rect,
            "nested subgrid {phase} complete overflow"
        );
        assert_eq!(
            geometry.canonical_content_size().unwrap(),
            expected_border,
            "nested subgrid {phase} canonical content extent"
        );
        assert_eq!(
            output.content_size, expected_content_size,
            "nested subgrid {phase} output content size"
        );
        assert_eq!(
            output.content_box_size(),
            expected_scrollport,
            "nested subgrid {phase} output content box accessor"
        );
        assert_eq!(
            output.scrollbar_size(),
            Size::splat(expected_bar),
            "nested subgrid {phase} output scrollbar accessor"
        );
        assert_eq!(
            geometry.target().border_box(),
            border_rect,
            "nested subgrid {phase} target border"
        );
        assert_eq!(
            geometry.target().flow_axes(),
            flow_axes,
            "nested subgrid {phase} target axes"
        );
        assert_eq!(
            geometry.target().scroll_margin(),
            scroll_margin,
            "nested subgrid {phase} target margin"
        );
        assert_eq!(
            geometry.target().snap_align(),
            snap_align,
            "nested subgrid {phase} target alignment"
        );
        assert_eq!(
            geometry.target().snap_stop(),
            ScrollSnapStop::Always,
            "nested subgrid {phase} target stop"
        );
    };

    let cold = compute_grid(&mut tree, 0, input).expect("cold subgrid computes");
    let cold_subgrid = tree.layout(1).expect("cold subgrid output is staged");
    assert_subgrid(
        "cold",
        cold_subgrid,
        Size::new(scalar(100.4), scalar(80.4)),
        scalar(7.4),
        Size::new(scalar(100.4), scalar(80.4)),
        (S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );
    let warm = compute_grid(&mut tree, 0, input).expect("warm subgrid computes");
    let warm_subgrid = tree.layout(1).expect("warm subgrid output is staged");
    assert_eq!(warm, cold);
    assert_eq!(warm_subgrid, cold_subgrid);
    assert_subgrid(
        "warm",
        warm_subgrid,
        Size::new(scalar(100.4), scalar(80.4)),
        scalar(7.4),
        Size::new(scalar(100.4), scalar(80.4)),
        (S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );

    Compute::set_unrounded(
        &mut tree,
        0,
        NodeOutputOf {
            source_index: SourceIndex::ZERO,
            location: Point::ZERO,
            size: cold.size,
            content_size: cold.content_size,
            scroll_geometry: cold.scroll_geometry,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            margin: Edges::ZERO,
        },
    );
    round_layout(&mut tree, 0).expect("subgrid geometry rounds from canonical source");
    let rounded = tree
        .final_layout(1)
        .expect("rounded subgrid output is staged");
    let rounded_geometry = rounded
        .scroll_geometry
        .expect("rounded subgrid retains canonical geometry");
    assert_subgrid(
        "rounded",
        NodeOutputOf {
            scroll_geometry: Some(rounded_geometry),
            ..rounded
        },
        Size::new(scalar(100.0), scalar(80.0)),
        scalar(7.0),
        Size::new(scalar(100.0), scalar(80.0)),
        (S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );
}

#[test]
fn fri05_c05_grid_round_cache_subgrid_matches_in_both_scalar_lanes() {
    assert_fri05_c05_grid_round_cache_subgrid::<f32>();
    assert_fri05_c05_grid_round_cache_subgrid::<f64>();
}

#[test]
fn fri05_c05_grid_auto_settles_cross_axis_induction_and_partitions_nested_pass_state() {
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
                grid_template_columns: vec![TrackComponent::px(95.0)],
                grid_template_rows: vec![TrackComponent::px(120.0)],
                ..NodeInput::default()
            },
        )
        .style(1, NodeInput::default());

    let output = compute_grid(
        &mut tree,
        0,
        fri05_c05_grid_sizing_input(Size::splat(Some(100.0))),
    )
    .expect("ordinary grid auto coupling computes");
    let geometry = output
        .scroll_geometry
        .expect("stable pass publishes geometry");
    assert!(geometry.gutters().right().is_some());
    assert!(geometry.gutters().bottom().is_some());
    assert_eq!(geometry.content_box().size(), Size::splat(90.0));

    let performed = tree
        .inputs(1)
        .iter()
        .filter(|input| input.run_mode() == RunMode::PerformLayout)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        performed.len(),
        3,
        "initial, y-induced, then x-induced passes"
    );
    assert_eq!(
        performed
            .iter()
            .map(ComputeInputOf::containing_auto_scrollbar_pass)
            .collect::<Vec<_>>(),
        vec![
            crate::scroll::SettledAutoScrollbarState::INITIAL,
            crate::scroll::SettledAutoScrollbarState::new(false, true),
            crate::scroll::SettledAutoScrollbarState::new(true, true),
        ]
    );
    assert!(performed.iter().all(|input| {
        input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
    }));
}

fn assert_fri08_c07_t04_grid_settlement_root_and_context_state<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let caller_state = crate::scroll::SettledAutoScrollbarState::new(false, true);
    let mut tree = OracleTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(10.0)).unwrap(),
                grid_template_columns: vec![TrackComponentOf::px(scalar(95.0))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(120.0))],
                ..NodeInputOf::default()
            },
        )
        .style(1, NodeInputOf::default());
    let input = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::splat(Some(scalar(100.0))),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::splat(AvailableOf::definite(scalar(100.0))),
    )
    .with_settled_auto_scrollbars(caller_state);

    let output = compute_grid(&mut tree, 0, input).unwrap();
    let geometry = output.scroll_geometry.unwrap();
    assert!(geometry.gutters().right().is_some());
    assert!(geometry.gutters().bottom().is_some());
    assert_eq!(geometry.content_box().size(), Size::splat(scalar(90.0)));

    let performed = tree
        .inputs(1)
        .iter()
        .filter(|input| input.run_mode() == RunMode::PerformLayout)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(performed.len(), 2, "caller-settled y then induced x pass");
    assert_eq!(
        performed
            .iter()
            .map(ComputeInputOf::containing_auto_scrollbar_pass)
            .collect::<Vec<_>>(),
        vec![
            caller_state,
            crate::scroll::SettledAutoScrollbarState::new(true, true),
        ]
    );
    assert!(performed.iter().all(|input| {
        input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
    }));

    let mut measurement_tree = OracleTreeOf::<S>::new()
        .children(0, [])
        .style(0, tree.node_input(0).clone());
    let measurement = compute_grid(
        &mut measurement_tree,
        0,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(scalar(100.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::definite(scalar(100.0))),
        )
        .with_settled_auto_scrollbars(caller_state),
    )
    .unwrap();
    assert!(measurement.scroll_geometry.is_none());
}

#[test]
fn fri08_c07_t04_grid_settlement_preserves_root_context_state_and_termination() {
    assert_fri08_c07_t04_grid_settlement_root_and_context_state::<f32>();
    assert_fri08_c07_t04_grid_settlement_root_and_context_state::<f64>();
}

#[test]
fn fri08_c07_t04_grid_settlement_preserves_geometry_cache_and_inherited_contexts() {
    assert_fri05_c05_grid_round_cache_for_display::<f32>(Display::Grid);
    assert_fri05_c05_grid_round_cache_for_display::<f64>(Display::Grid);
    assert_fri08_c06r_inherited_placement_flow_matrix::<f32>();
    assert_fri08_c06r_inherited_placement_flow_matrix::<f64>();
}

#[test]
fn fri08_c07_t04_grid_settlement_preserves_exact_error_mapping() {
    assert_fri06_mr02_geometry_error_grid_own::<f32>();
    assert_fri06_mr02_geometry_error_grid_own::<f64>();
}

#[test]
fn fri08_c08_t05_impossible_state_standalone_nested_lanes_intrinsic_scroll_and_outputs_stay_composed()
 {
    assert_fri08_c04_standalone_nested_flows::<f32>();
    assert_fri08_c04_standalone_nested_flows::<f64>();
    fri08_c05_composition_grid010_nested_indefinite_lanes_output::<f32>();
    fri08_c05_composition_grid010_nested_indefinite_lanes_output::<f64>();
    assert_fri08_c03_nested_candidate_bounds_edges_and_reversal::<f32>();
    assert_fri08_c03_nested_candidate_bounds_edges_and_reversal::<f64>();
    assert_fri08_c04_overflow_hidden_subgrid_intrinsic_size::<f32>();
    assert_fri08_c04_overflow_hidden_subgrid_intrinsic_size::<f64>();

    let traversal = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1_u32,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Standalone,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 2),
            available_inline_size: None,
            available_inline_size_is_known: false,
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: vec![traversal_leaf(2, 1, 2)],
        })],
    })
    .unwrap();
    assert_eq!(traversal.leaves.len(), 1);
    assert_eq!(traversal.leaves[0].node, 1);
    assert_eq!(traversal.leaves[0].ancestor_span, GridTrackSpan::new(1, 2));
}

#[test]
fn fri08_c08_t05_impossible_state_remaining_errors_failure_atomicity_and_cache_are_exact() {
    assert_eq!(
        inherit_subgrid_tracks(SubgridTrackInheritanceInput {
            parent_tracks: &[],
            parent_span: GridTrackSpan::new(1, 2),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: 0.0,
            subgrid_gap: ResolvedSubgridGap::Normal,
        })
        .unwrap_err(),
        SubgridTrackInheritanceError::EmptyTrackList
    );
    assert_eq!(
        inherit_subgrid_tracks(SubgridTrackInheritanceInput {
            parent_tracks: &[10.0, 20.0],
            parent_span: GridTrackSpan::new(1, 4),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: 0.0,
            subgrid_gap: ResolvedSubgridGap::Normal,
        })
        .unwrap_err(),
        SubgridTrackInheritanceError::SpanOutOfRange
    );
    assert_eq!(
        traverse_subgrid_intrinsic(SubgridTraversalInput::<u32> {
            ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Unknown,
            root_children: Vec::new(),
        })
        .unwrap_err(),
        SubgridTraversalError::MissingIntrinsicMinTrackFacts
    );
    assert_eq!(
        traverse_subgrid_intrinsic(SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
            root_children: vec![traversal_leaf(1, 1, 1)],
        })
        .unwrap_err(),
        SubgridTraversalError::SpanOutOfRange
    );

    assert_fri08_c04_standalone_cache_and_failures_are_atomic::<f32>();
    assert_fri08_c04_standalone_cache_and_failures_are_atomic::<f64>();
    assert_fri08_c04_overflow_atomic_failures::<f32>();
    assert_fri08_c04_overflow_atomic_failures::<f64>();
}

#[test]
fn fri05_c05_grid_auto_minimum_computed_scrollability_reaches_grid_and_lanes_front_doors() {
    for display in [Display::Grid, Display::GridLanes] {
        for writing_mode in [
            WritingMode::HorizontalTb,
            WritingMode::VerticalRl,
            WritingMode::VerticalLr,
            WritingMode::SidewaysRl,
            WritingMode::SidewaysLr,
        ] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                let auto_flows: &[GridAutoFlow] = if display == Display::GridLanes {
                    &[GridAutoFlow::Row, GridAutoFlow::Column]
                } else {
                    &[GridAutoFlow::Row]
                };
                for &grid_auto_flow in auto_flows {
                    for overflow in [
                        Overflow::Visible,
                        Overflow::Clip,
                        Overflow::Hidden,
                        Overflow::Scroll,
                        Overflow::Auto,
                    ] {
                        let mut tree = fri05_c05_grid_sizing_tree(
                            display,
                            writing_mode,
                            direction,
                            computed_overflow(overflow, overflow),
                            false,
                            Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
                            grid_auto_flow,
                        );

                        compute_grid(
                            &mut tree,
                            0,
                            fri05_c05_grid_sizing_input(Size::splat(Some(40.0))),
                        )
                        .expect("grid automatic-minimum case computes");

                        let layout_input = tree
                            .inputs(1)
                            .iter()
                            .find(|input| input.run_mode() == RunMode::PerformLayout)
                            .expect("grid item receives final layout input");
                        let item_size = Size::new(Some(20.0), Some(30.0));
                        let content_size = Size::new(Some(80.0), Some(90.0));
                        let zero = Size::new(Some(0.0), Some(0.0));
                        let expected_known = if display == Display::Grid {
                            match overflow {
                                Overflow::Visible => content_size,
                                Overflow::Clip => item_size,
                                Overflow::Hidden | Overflow::Scroll | Overflow::Auto => zero,
                            }
                        } else {
                            let flow_axes = FlowAxes::new(writing_mode, direction);
                            let grid_axis = grid_axis_for_lanes(grid_auto_flow);
                            let physical_axis = match grid_axis.logical_axis() {
                                LogicalAxis::Inline => flow_axes.inline_axis(),
                                LogicalAxis::Block => flow_axes.block_axis(),
                            };
                            let selected = match (physical_axis, overflow) {
                                (PhysicalAxis::Horizontal, Overflow::Visible) => content_size.width,
                                (PhysicalAxis::Vertical, Overflow::Visible) => content_size.height,
                                (PhysicalAxis::Horizontal, Overflow::Clip) => item_size.width,
                                (PhysicalAxis::Vertical, Overflow::Clip) => item_size.height,
                                (_, Overflow::Hidden | Overflow::Scroll | Overflow::Auto) => {
                                    Some(0.0)
                                }
                            };
                            match physical_axis {
                                PhysicalAxis::Horizontal => Size::new(selected, item_size.height),
                                PhysicalAxis::Vertical => Size::new(item_size.width, selected),
                            }
                        };
                        assert_eq!(
                            layout_input.known(),
                            expected_known,
                            "{display:?} {writing_mode:?} {direction:?} {grid_auto_flow:?} {overflow:?} automatic minimum"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn fri05_c05_grid_intrinsic_overflow_projects_used_axes_and_traps_descendants() {
    let writing_modes = [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ];
    for display in [Display::Grid, Display::GridLanes] {
        for writing_mode in writing_modes {
            for direction in [Direction::Ltr, Direction::Rtl] {
                for (overflow, replaced, expected) in [
                    (
                        computed_overflow(Overflow::Visible, Overflow::Visible),
                        false,
                        Size::new(80.0, 90.0),
                    ),
                    (
                        computed_overflow(Overflow::Clip, Overflow::Clip),
                        false,
                        Size::new(20.0, 30.0),
                    ),
                    (
                        computed_overflow(Overflow::Hidden, Overflow::Hidden),
                        false,
                        Size::new(20.0, 30.0),
                    ),
                    (
                        computed_overflow(Overflow::Hidden, Overflow::Hidden),
                        true,
                        Size::new(20.0, 30.0),
                    ),
                    (
                        computed_overflow(Overflow::Scroll, Overflow::Scroll),
                        false,
                        Size::new(20.0, 30.0),
                    ),
                    (
                        computed_overflow(Overflow::Auto, Overflow::Auto),
                        false,
                        Size::new(20.0, 30.0),
                    ),
                    (
                        computed_overflow(Overflow::Visible, Overflow::Clip),
                        false,
                        Size::new(80.0, 30.0),
                    ),
                    (
                        computed_overflow(Overflow::Clip, Overflow::Visible),
                        false,
                        Size::new(20.0, 90.0),
                    ),
                ] {
                    let mut tree = fri05_c05_grid_sizing_tree(
                        display,
                        writing_mode,
                        direction,
                        overflow,
                        replaced,
                        Size::new(PreferredSize::px(20.0), PreferredSize::px(30.0)),
                        GridAutoFlow::Row,
                    );

                    let output =
                        compute_grid(&mut tree, 0, fri05_c05_grid_sizing_input(Size::NONE))
                            .expect("grid intrinsic overflow case computes");

                    assert_eq!(
                        output.size, expected,
                        "{display:?} {writing_mode:?} {direction:?} {overflow:?} replaced={replaced}"
                    );
                }
            }
        }
    }
}

#[test]
fn fri05_c05_grid_intrinsic_overflow_subgrid_uses_the_parent_flow_projection() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for (overflow, replaced, expected) in [
                (
                    computed_overflow(Overflow::Visible, Overflow::Visible),
                    false,
                    Size::new(80.0, 90.0),
                ),
                (
                    computed_overflow(Overflow::Clip, Overflow::Clip),
                    false,
                    Size::new(20.0, 30.0),
                ),
                (
                    computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    false,
                    Size::new(20.0, 30.0),
                ),
                (
                    computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    true,
                    Size::new(20.0, 30.0),
                ),
                (
                    computed_overflow(Overflow::Scroll, Overflow::Scroll),
                    false,
                    Size::new(20.0, 30.0),
                ),
                (
                    computed_overflow(Overflow::Auto, Overflow::Auto),
                    false,
                    Size::new(20.0, 30.0),
                ),
                (
                    computed_overflow(Overflow::Visible, Overflow::Clip),
                    false,
                    Size::new(80.0, 30.0),
                ),
                (
                    computed_overflow(Overflow::Clip, Overflow::Visible),
                    false,
                    Size::new(20.0, 90.0),
                ),
            ] {
                let mut tree = OracleTree::new()
                    .children(0, [1])
                    .children(1, [2])
                    .children(2, [])
                    .style(
                        0,
                        NodeInput {
                            display: Display::Grid,
                            writing_mode,
                            direction,
                            grid_template_columns: vec![TrackComponent::AUTO],
                            grid_template_rows: vec![TrackComponent::AUTO],
                            ..NodeInput::default()
                        },
                    )
                    .style(
                        1,
                        NodeInput {
                            display: Display::Grid,
                            writing_mode,
                            direction,
                            grid_template_columns: vec![empty_subgrid_track()],
                            grid_template_rows: vec![empty_subgrid_track()],
                            grid_column: GridPlacement::try_lines(1, -1)
                                .expect("valid subgrid column span"),
                            grid_row: GridPlacement::try_lines(1, -1)
                                .expect("valid subgrid row span"),
                            ..NodeInput::default()
                        },
                    )
                    .style(
                        2,
                        NodeInput {
                            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(30.0)),
                            overflow,
                            item_is_replaced: replaced,
                            ..NodeInput::default()
                        },
                    )
                    .measure(
                        2,
                        ComputeOutput::from_sizes(Size::new(20.0, 30.0), Size::new(80.0, 90.0)),
                    );

                let output = compute_grid(&mut tree, 0, fri05_c05_grid_sizing_input(Size::NONE))
                    .expect("intrinsic subgrid overflow case computes");

                assert_eq!(
                    output.size, expected,
                    "{writing_mode:?} {direction:?} {overflow:?} replaced={replaced}"
                );
            }
        }
    }
}

#[test]
fn fri05_c05_grid_intrinsic_overflow_percentage_tracks_keep_item_priority_when_trapped() {
    for (overflow, expected) in [
        (Overflow::Visible, Size::new(80.0, 90.0)),
        (Overflow::Clip, Size::new(20.0, 30.0)),
        (Overflow::Hidden, Size::new(20.0, 30.0)),
        (Overflow::Scroll, Size::new(20.0, 30.0)),
        (Overflow::Auto, Size::new(20.0, 30.0)),
    ] {
        let mut tree = OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInput {
                    display: Display::Grid,
                    grid_template_columns: vec![TrackComponent::percent(1.0)],
                    grid_template_rows: vec![TrackComponent::percent(1.0)],
                    ..NodeInput::default()
                },
            )
            .style(
                1,
                NodeInput {
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(30.0)),
                    overflow: computed_overflow(overflow, overflow),
                    ..NodeInput::default()
                },
            )
            .measure(
                1,
                ComputeOutput::from_sizes(Size::new(20.0, 30.0), Size::new(80.0, 90.0)),
            );

        let output = compute_grid(&mut tree, 0, fri05_c05_grid_sizing_input(Size::NONE))
            .expect("percentage-track intrinsic overflow case computes");

        assert_eq!(output.size, expected, "{overflow:?}");
    }
}

#[test]
fn fri04_c04_grid_dispatch_scrollable_auto_minimum_lane_reports_exact_grid_lanes_payload() {
    fri04_c04_grid_dispatch_assert_error(
        Display::GridLanes,
        NodeInput {
            display: Display::Flex,
            min_size: Size::new(MinSize::AUTO, MinSize::STRETCH),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            ..NodeInput::default()
        },
        SizingProperty::Minimum,
        SizingBehavior::Stretch,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Vertical,
        1,
    );
}

#[test]
fn lanes_reject_overflowed_affine_tolerance_resolution() {
    let err = place_lanes(LanePlacementInput::<&str> {
        grid_axis_tracks: 2,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: GridFlowTolerance::Length(Length::value(invalid_numeric_lp())),
        tolerance_basis: f32::MAX,
        items: Vec::new(),
    })
    .expect_err("overflowed affine tolerance should return a typed error");

    assert_eq!(err, LanePlacementError::InvalidGridFlowToleranceResolution);
}

#[test]
fn grid_lanes_layout_rejects_overflowed_affine_tolerance_resolution() {
    let style = NodeInput {
        display: Display::GridLanes,
        grid_auto_flow: GridAutoFlow::Row,
        grid_flow_tolerance: GridFlowTolerance::Length(Length::value(invalid_numeric_lp())),
        ..NodeInput::default()
    };
    let constants = Constants {
        flow_axes: crate::geometry::FlowAxes::new(
            crate::WritingMode::HorizontalTb,
            crate::Direction::Ltr,
        ),
        explicit_definite_content_size: Size::splat(Some(10.0)),
        node_outer_size: Size::splat(Some(10.0)),
        node_inner_size: Size::splat(Some(10.0)),
        node_min_size: Size::NONE,
        node_max_size: Size::NONE,
        available_inner_size: Size::splat(Some(10.0)),
        content_box_inset: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let lines = GridLines {
        column_explicit_start: 0,
        column_explicit_count: 1,
        row_explicit_start: 0,
        row_explicit_count: 1,
    };
    let context = GridContainerContext {
        topology: topology::ExpandedGridTopology::from_test_parts(
            vec![TrackSizingOf::AUTO],
            vec![TrackSizingOf::AUTO],
            named::NamedGridLines::new(GridAxisKind::Column, 1),
            named::NamedGridLines::new(GridAxisKind::Row, 1),
            None,
        ),
        gap: LogicalSizeOf::new(0.0, 0.0),
        column_gutters: OrdinaryGridAxisGuttersOf::new(1, &[], 0.0),
        row_gutters: OrdinaryGridAxisGuttersOf::new(1, &[], 0.0),
        percent_basis: LogicalSizeOf::new(Some(f32::MAX), Some(f32::MAX)),
        leading_columns: 0,
        leading_rows: 0,
        lines,
        inherited_column_offset: None,
        inherited_row_offset: None,
    };
    let placements = GridPlacementContext::new(Vec::<u32>::new(), Vec::new());
    let mut tree = OracleTree::new().children(1, []).style(1, style.clone());

    let err = resolve_grid_lanes_placement_with_resolved_tracks(
        &mut tree,
        1,
        &grid_container_projection!(&style),
        &constants,
        context,
        &[10.0],
        &[10.0],
        &placements,
        0.0,
        LogicalSizeOf::new(Some(10.0), Some(10.0)),
    )
    .expect("layout resolution should not fail")
    .expect_err("invalid layout tolerance should not produce a placement report");

    assert_eq!(err, LanePlacementError::InvalidGridFlowToleranceResolution);
}

#[test]
fn owner_to_current_placement_map_rejects_out_of_range_composition() {
    let identity = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        2,
    );
    let result = identity.compose(owner_placement_boundary!(
        1,
        2,
        GridTrackSpan::new(1, 3),
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
        Err(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange),
    );
}

#[test]
fn grid_absolute_child_layout_records_scrollbar_size_for_scroll_overflow() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(12.0).unwrap(),
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
        tree.layout(2)
            .expect("node layout is staged")
            .scrollbar_size(),
        Size::new(12.0, 10.0)
    );
}

#[test]
fn grid_content_box_compute_size_does_not_add_scrollbar_to_authored_size() {
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
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(15.0).unwrap(),
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

    assert_eq!(output.size, Size::new(42.0, 32.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn grid_scrollbar_gutter_does_not_force_outer_size_past_authored_size() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(2.0), PreferredSize::px(4.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(15.0).unwrap(),
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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(2.0, 4.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn grid_child_layout_records_scrollbar_size_for_scroll_overflow() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(11.0).unwrap(),
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
        tree.layout(2)
            .expect("node layout is staged")
            .scrollbar_size(),
        Size::new(11.0, 10.0)
    );
}

#[test]
fn grid_safe_align_content_falls_back_to_start_when_tracks_overflow() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
            ],
            align_content: Some(AlignContent::SafeCenter),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());

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

    let expected = align_tracks_report(
        100.0,
        vec![40.0, 40.0, 40.0],
        0.0,
        TrackAlignment::Center,
        AlignmentSafety::Safe,
    );

    assert!(expected.safe_fallback_used);
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, expected.offsets[0])
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(40.0, 40.0)
    );
}

#[test]
fn grid_justify_content_space_around_and_evenly_distribute_free_width() {
    fn run(alignment: AlignContent) -> (Point, Point) {
        let mut tree = OracleTree::new();
        tree.insert_children(1, vec![2, 3]);
        tree.insert_children(2, vec![]);
        tree.insert_children(3, vec![]);
        tree.insert_style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![TrackComponent::px(50.0), TrackComponent::px(50.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(alignment),
                ..NodeInput::default()
            },
        );
        tree.insert_style(2, NodeInput::default());
        tree.insert_style(3, NodeInput::default());

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

        (
            tree.layout(2).expect("node layout is staged").location,
            tree.layout(3).expect("node layout is staged").location,
        )
    }

    assert_eq!(
        run(AlignContent::SpaceAround),
        (Point::new(22.5, 0.0), Point::new(127.5, 0.0))
    );
    assert_eq!(
        run(AlignContent::SpaceEvenly),
        (Point::new(30.0, 0.0), Point::new(120.0, 0.0))
    );
}

#[test]
fn grid_content_size_includes_visible_child_overflow_content() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
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
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(40.0, 10.0)
    );
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn grid_safe_justify_self_falls_back_to_start_when_item_overflows() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(100.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(150.0), PreferredSize::px(50.0)),
            justify_self: Some(AlignItems::SafeCenter),
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
        Size::new(150.0, 50.0)
    );
}

#[test]
fn subgrid_eligibility_allows_clipped_overflow() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Auto),
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(report.eligible);
    assert_eq!(report.reason, None);
}
