use super::fixtures::{
    Fri04C04GridSizingValue, Fri08C02StretchTreeInput, Fri08C02TrackAxis, Fri08C06RInheritedAxes,
    SubgridChildParentContextInput, SubgridEligibilityInput,
    assert_fri08_c02_fit_content_flex_composes, baseline_measure, compute_oracle_grid,
    computed_overflow, default_grid_item_projection, empty_subgrid_track, entry_names,
    fri04_c03_grid_track_nested, fri04_c03_grid_track_percentage_nested,
    fri04_c03_grid_track_value, fri04_c04_grid_dispatch_assert_error,
    fri04_c04_grid_dispatch_style, fri05_c05_grid_sizing_input, fri06_c07_height_output,
    fri06_c12_t08_inherited_baseline_gap_position, fri08_c01_placement_output,
    fri08_c01_topology_for_style, fri08_c02_auto_fit_output, fri08_c02_auto_fit_repeat,
    fri08_c02_flex_track, fri08_c02_stretch_track, fri08_c02_stretch_tree,
    fri08_c02_track_mix_tree, fri08_c02_track_sizes, fri08_c03_auto_fit_batch,
    fri08_c03_auto_fit_named_repeat, fri08_c04_standalone_intrinsic_minimum_width,
    fri08_c06r_assert_cold_warm, inherited_placement_group, inherited_placement_member,
    inherited_placement_witness, invalid_numeric_lp, local_line_names, lp,
    single_grid_placement_context, subgrid_axis_report, subgrid_child_parent_context,
    subgrid_child_parent_context_with_geometry, subgrid_eligibility, subgrid_track,
    subgrid_track_of, tagged_baseline, traversal_leaf, vertical_baseline_measure,
    with_projected_subgrid_child_input,
};
use super::*;

fn child_subgrid_gap<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axis: GridAxisKind,
    area_size: Size<S>,
) -> Result<ResolvedSubgridGap<S>, crate::LengthResolutionStatus<S>> {
    super::child::child_subgrid_gap(&grid_container_projection!(style), axis, area_size)
}

fn needs_intrinsic_subgrid_context<Node: Copy, S: LayoutScalar>(
    style: &NodeInputOf<S>,
    item: SubgridItemReport<Node>,
    area: GridArea<S>,
) -> bool {
    super::tracks::needs_intrinsic_subgrid_context(
        &grid_container_projection!(style),
        &grid_item_projection!(style),
        item,
        area,
    )
}

fn subgrid_child_parent_context_from_ancestor_groups<Node: Copy + PartialEq, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'_, Node, S>,
    ancestor_baseline_groups: &FinalAncestorBaselineGroups<Node, S>,
    parent_grid: Node,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>> {
    with_projected_subgrid_child_input(input, |input| {
        super::child::subgrid_child_parent_context_from_ancestor_groups(
            input,
            ancestor_baseline_groups,
            parent_grid,
        )
    })
}

fn fri08_c06r_inherited_placement_leading_trailing_tree<S: LayoutScalar>(
    inherited_axes: Fri08C06RInheritedAxes,
) -> PublicLayoutTreeOf<S> {
    let scalar = S::from_f64;
    let inherited_columns = inherited_axes == Fri08C06RInheritedAxes::Columns;
    let root_columns = if inherited_columns {
        vec![TrackComponentOf::px(scalar(20.0)); 4]
    } else {
        vec![TrackComponentOf::px(scalar(100.0))]
    };
    let root_rows = if inherited_columns {
        vec![TrackComponentOf::px(scalar(100.0))]
    } else {
        vec![TrackComponentOf::px(scalar(20.0)); 4]
    };

    PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3, 4])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: if inherited_columns {
                    Size::new(
                        PreferredSizeOf::px(scalar(80.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    )
                } else {
                    Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(80.0)),
                    )
                },
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
                grid_template_columns: if inherited_columns {
                    subgrid_track_of()
                } else {
                    vec![TrackComponentOf::px(scalar(40.0))]
                },
                grid_template_rows: if inherited_columns {
                    vec![TrackComponentOf::px(scalar(40.0))]
                } else {
                    subgrid_track_of()
                },
                grid_auto_columns: vec![
                    TrackComponentOf::px(scalar(10.0)),
                    TrackComponentOf::px(scalar(20.0)),
                ],
                grid_auto_rows: vec![
                    TrackComponentOf::px(scalar(10.0)),
                    TrackComponentOf::px(scalar(20.0)),
                ],
                grid_column: if inherited_columns {
                    GridPlacement::try_line_span(1, 4).expect("four inherited columns")
                } else {
                    GridPlacement::try_line(1).expect("standalone column")
                },
                grid_row: if inherited_columns {
                    GridPlacement::try_line(1).expect("standalone row")
                } else {
                    GridPlacement::try_line_span(1, 4).expect("four inherited rows")
                },
                grid_auto_flow: if inherited_columns {
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
                grid_column: if inherited_columns {
                    GridPlacement::try_line(1).expect("first inherited column")
                } else {
                    GridPlacement::try_line(-3).expect("leading implicit column")
                },
                grid_row: if inherited_columns {
                    GridPlacement::try_line(-3).expect("leading implicit row")
                } else {
                    GridPlacement::try_line(1).expect("first inherited row")
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                grid_column: if inherited_columns {
                    GridPlacement::try_line(2).expect("second inherited column")
                } else {
                    GridPlacement::try_line(3).expect("trailing implicit column")
                },
                grid_row: if inherited_columns {
                    GridPlacement::try_line(3).expect("trailing implicit row")
                } else {
                    GridPlacement::try_line(2).expect("second inherited row")
                },
                ..NodeInputOf::default()
            },
        )
}

fn assert_fri08_c06r_inherited_placement_leading_trailing<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for inherited_axes in [
        Fri08C06RInheritedAxes::Columns,
        Fri08C06RInheritedAxes::Rows,
    ] {
        let tree = fri08_c06r_inherited_placement_leading_trailing_tree(inherited_axes);
        fri08_c06r_assert_cold_warm(tree, &[1, 2, 3, 4], |batch| {
            let leading = fri08_c01_placement_output(batch, 3);
            let trailing = fri08_c01_placement_output(batch, 4);
            if inherited_axes == Fri08C06RInheritedAxes::Columns {
                assert_eq!(leading.location, Point::ZERO);
                assert_eq!(leading.size, Size::new(scalar(20.0), scalar(20.0)));
                assert_eq!(trailing.location, Point::new(scalar(20.0), scalar(70.0)));
                assert_eq!(trailing.size, Size::new(scalar(20.0), scalar(20.0)));
            } else {
                assert_eq!(leading.location, Point::ZERO);
                assert_eq!(leading.size, Size::new(scalar(20.0), scalar(20.0)));
                assert_eq!(trailing.location, Point::new(scalar(70.0), scalar(20.0)));
                assert_eq!(trailing.size, Size::new(scalar(20.0), scalar(20.0)));
            }
        });
    }
}

#[test]
fn fri08_c06r_inherited_placement_standalone_axes_preserve_leading_trailing_pattern_phase() {
    assert_fri08_c06r_inherited_placement_leading_trailing::<f32>();
    assert_fri08_c06r_inherited_placement_leading_trailing::<f64>();
}

#[test]
fn fri08_c06r_inherited_placement_architecture_has_no_residual_ordinary_estimator() {
    let orchestration = include_str!("../grid/mod.rs");
    for residual in [
        "visible_cell_count",
        "placement_cell_span",
        "auto_fit_limit",
        ".div_ceil(",
    ] {
        assert!(
            !orchestration.contains(residual),
            "FRI-08.14 forbids residual ordinary-grid demand estimator `{residual}`"
        );
    }
    assert!(
        !include_str!("../grid/placement.rs").contains("fn placement_cell_span"),
        "the estimator-only placement span helper must be absent"
    );
}

#[test]
fn fri08_c03_auto_fit_ordinary_grid_and_lanes_auto_fill_remain_separate_controls() {
    for (display, repeat, expected_x) in [
        (Display::Grid, TrackRepeat::AutoFit, 40.0),
        (Display::GridLanes, TrackRepeat::AutoFill, 0.0),
    ] {
        let tree = PublicLayoutTreeOf::<f32>::new().children(1, [2, 3]).style(
            1,
            NodeInput {
                display,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c03_auto_fit_named_repeat(repeat)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        );
        let tree = [2, 3].into_iter().fold(tree, |tree, node| {
            tree.style(
                node,
                NodeInput {
                    grid_column: GridPlacement::try_line(1).expect("first repeated track"),
                    grid_row: GridPlacement::try_line(1).expect("single row"),
                    ..NodeInput::DEFAULT
                },
            )
        });
        let batch = fri08_c03_auto_fit_batch(&tree, Size::new(120.0, 20.0));
        let output = fri08_c01_placement_output(&batch, 2);
        assert_eq!((output.location.x, output.size.width), (expected_x, 40.0));
    }
}

#[test]
fn fri08_c03_auto_fit_lanes_flexible_tracks_and_stretched_items_ignore_collapsed_gutters_on_both_axes()
 {
    let mut track_geometries = Vec::new();
    for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
        let repeat = TrackComponent::Repeat(
            TrackRepetition::auto_fit_components(vec![TrackComponent::minmax(
                MinTrackSizing::px(40.0),
                MaxTrackSizing::flex(
                    TrackFlexFactor::try_new(1.0).expect("finite auto-fit flex factor"),
                ),
            )])
            .expect("valid flexible auto-fit repetition"),
        );
        let (columns, rows, auto_flow, size, gap, child_placement) = match axis {
            Fri08C02TrackAxis::Columns => (
                vec![repeat],
                vec![TrackComponent::px(20.0)],
                GridAutoFlow::Row,
                Size::new(140.0, 20.0),
                Size::new(Length::px(10.0), Length::ZERO),
                (
                    GridPlacement::try_line(1).expect("first repeated column"),
                    GridPlacement::try_line(1).expect("single row"),
                ),
            ),
            Fri08C02TrackAxis::Rows => (
                vec![TrackComponent::px(20.0)],
                vec![repeat],
                GridAutoFlow::Column,
                Size::new(20.0, 140.0),
                Size::new(Length::ZERO, Length::px(10.0)),
                (
                    GridPlacement::try_line(1).expect("single column"),
                    GridPlacement::try_line(1).expect("first repeated row"),
                ),
            ),
        };
        let tree = PublicLayoutTreeOf::<f32>::new()
            .children(1, [2])
            .style(
                1,
                NodeInput {
                    display: Display::GridLanes,
                    size: size.map(PreferredSize::px),
                    grid_template_columns: columns,
                    grid_template_rows: rows,
                    grid_auto_flow: auto_flow,
                    gap,
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    grid_column: child_placement.0,
                    grid_row: child_placement.1,
                    ..NodeInput::DEFAULT
                },
            );

        let batch = fri08_c03_auto_fit_batch(&tree, size);
        let child = fri08_c01_placement_output(&batch, 2);
        let track_geometry = match axis {
            Fri08C02TrackAxis::Columns => (child.location.x, child.size.width),
            Fri08C02TrackAxis::Rows => (child.location.y, child.size.height),
        };
        track_geometries.push(track_geometry);
    }
    assert_eq!(track_geometries, [(0.0, 140.0), (0.0, 140.0)]);
}

#[test]
fn fri08_c04_standalone_ordinary_one_axis_subgrid_keeps_automatic_minimum() {
    assert_eq!(
        fri08_c04_standalone_intrinsic_minimum_width(MinSize::AUTO),
        20.0
    );
}

#[test]
fn fri08_c02_stretch_lanes_policy_accepts_c03_auto_fit_control() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
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
                grid_column: GridPlacement::try_line(1).expect("first lane track"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("overlapping lane track"),
                ..NodeInput::DEFAULT
            },
        );
    let output = fri08_c02_auto_fit_output(&tree, Size::new(120.0, 20.0), 2);
    assert_eq!((output.location.x, output.size.width), (40.0, 40.0));
}

#[test]
fn fri08_c02_fit_content_vertical_sideways_and_scalar_lanes_project_the_same_sizes() {
    for writing_mode in [
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        assert_fri08_c02_fit_content_flex_composes::<f32>(Fri08C02TrackAxis::Columns, writing_mode);
        assert_fri08_c02_fit_content_flex_composes::<f32>(Fri08C02TrackAxis::Rows, writing_mode);
        assert_fri08_c02_fit_content_flex_composes::<f64>(Fri08C02TrackAxis::Columns, writing_mode);
        assert_fri08_c02_fit_content_flex_composes::<f64>(Fri08C02TrackAxis::Rows, writing_mode);
    }
}

fn assert_fri08_c02r_lanes_fit_content_flex<S: LayoutScalar>() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::SidewaysLr,
    ] {
        for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
            for (fit_limit, basis, intrinsic_bases, expected) in [
                (50.0, 200.0, [20.0, 0.0], [20.0, 180.0]),
                (20.0, 100.0, [30.0, 0.0], [30.0, 70.0]),
            ] {
                let (tree, flow_axes, viewport) = fri08_c02_track_mix_tree(
                    Display::GridLanes,
                    axis,
                    writing_mode,
                    (fit_limit, 0.0),
                    Some(basis),
                    vec![fri08_c02_flex_track::<S>(1.0)],
                    &intrinsic_bases,
                );
                let sizes = fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 2);
                assert_eq!(
                    sizes,
                    expected.map(S::from_f64),
                    "{writing_mode:?} {axis:?} fit-content and flex must execute one phase pipeline"
                );
            }
        }
    }
}

#[test]
fn fri08_c02r_lanes_track_phase_fit_content_and_flex_compose_across_axes_flows_and_scalars() {
    assert_fri08_c02r_lanes_fit_content_flex::<f32>();
    assert_fri08_c02r_lanes_fit_content_flex::<f64>();
}

fn assert_fri08_c02r_lanes_auto_max_stretch<S: LayoutScalar>() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::SidewaysLr,
    ] {
        for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
            for alignment in [None, Some(AlignContent::Stretch)] {
                let (tree, flow_axes, viewport) =
                    fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                        display: Display::GridLanes,
                        axis,
                        writing_mode,
                        definite_axis_size: Some(100.0),
                        viewport_axis_size: 100.0,
                        gap: 0.0,
                        alignment,
                        tracks: vec![fri08_c02_stretch_track(MinTrackSizingOf::px(S::ZERO))],
                        measurements: &[0.0],
                    });
                assert_eq!(
                    fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 1),
                    [S::from_f64(100.0)],
                    "{writing_mode:?} {axis:?} {alignment:?} minmax(0,auto) stretches"
                );
            }

            let (tree, flow_axes, viewport) = fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                display: Display::GridLanes,
                axis,
                writing_mode,
                definite_axis_size: Some(100.0),
                viewport_axis_size: 100.0,
                gap: 0.0,
                alignment: Some(AlignContent::Stretch),
                tracks: vec![
                    fri08_c02_stretch_track(MinTrackSizingOf::<S>::MIN_CONTENT),
                    fri08_c02_stretch_track(MinTrackSizingOf::<S>::MAX_CONTENT),
                ],
                measurements: &[20.0, 30.0],
            });
            assert_eq!(
                fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 2),
                [S::from_f64(45.0), S::from_f64(55.0)],
                "{writing_mode:?} {axis:?} intrinsic floors receive the same remainder"
            );
        }
    }
}

#[test]
fn fri08_c02r_lanes_track_phase_auto_max_stretch_preserves_floors_across_axes_flows_and_scalars() {
    assert_fri08_c02r_lanes_auto_max_stretch::<f32>();
    assert_fri08_c02r_lanes_auto_max_stretch::<f64>();
}

fn assert_fri08_c02r_lanes_stretch_exclusions<S: LayoutScalar>() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::SidewaysLr,
    ] {
        for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
            let resolve = |track, definite_axis_size, alignment, measurement| {
                let (tree, flow_axes, viewport) =
                    fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                        display: Display::GridLanes,
                        axis,
                        writing_mode,
                        definite_axis_size,
                        viewport_axis_size: 100.0,
                        gap: 0.0,
                        alignment,
                        tracks: vec![track],
                        measurements: &[measurement],
                    });
                fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 1)[0]
            };

            assert_eq!(
                resolve(
                    TrackComponentOf::minmax(
                        MinTrackSizingOf::px(S::ZERO),
                        MaxTrackSizingOf::MAX_CONTENT,
                    ),
                    Some(100.0),
                    Some(AlignContent::Stretch),
                    20.0,
                ),
                S::from_f64(20.0),
                "{writing_mode:?} {axis:?} a non-auto maximum is ineligible"
            );
            assert_eq!(
                resolve(
                    fri08_c02_stretch_track(MinTrackSizingOf::MIN_CONTENT),
                    Some(100.0),
                    Some(AlignContent::Start),
                    20.0,
                ),
                S::from_f64(20.0),
                "{writing_mode:?} {axis:?} start alignment does not stretch"
            );

            let (indefinite_tree, indefinite_flow_axes, _) =
                fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                    display: Display::GridLanes,
                    axis,
                    writing_mode,
                    definite_axis_size: None,
                    viewport_axis_size: 100.0,
                    gap: 0.0,
                    alignment: Some(AlignContent::Stretch),
                    tracks: vec![fri08_c02_stretch_track(MinTrackSizingOf::<S>::MIN_CONTENT)],
                    measurements: &[20.0],
                });
            let indefinite_batch = compute_layout(
                &indefinite_tree,
                1,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
                    .expect("indefinite lanes track-phase viewport"),
            )
            .expect("indefinite lanes track sizing completes");
            let indefinite_size = indefinite_flow_axes
                .logical_size(fri08_c01_placement_output(&indefinite_batch, 2).size);
            assert_eq!(
                match axis {
                    Fri08C02TrackAxis::Columns => indefinite_size.inline,
                    Fri08C02TrackAxis::Rows => indefinite_size.block,
                },
                S::from_f64(20.0),
                "{writing_mode:?} {axis:?} indefinite space does not stretch"
            );
            assert_eq!(
                resolve(
                    fri08_c02_stretch_track(MinTrackSizingOf::MIN_CONTENT),
                    Some(10.0),
                    Some(AlignContent::Stretch),
                    20.0,
                ),
                S::from_f64(20.0),
                "{writing_mode:?} {axis:?} a non-positive remainder preserves the floor"
            );

            let (collapsed_tree, collapsed_flow_axes, collapsed_viewport) =
                fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                    display: Display::GridLanes,
                    axis,
                    writing_mode,
                    definite_axis_size: Some(120.0),
                    viewport_axis_size: 120.0,
                    gap: 0.0,
                    alignment: Some(AlignContent::Stretch),
                    tracks: vec![fri08_c02_auto_fit_repeat::<S>()],
                    measurements: &[0.0],
                });
            assert_eq!(
                fri08_c02_track_sizes(
                    &collapsed_tree,
                    collapsed_flow_axes,
                    collapsed_viewport,
                    axis,
                    1,
                ),
                [S::from_f64(40.0)],
                "{writing_mode:?} {axis:?} collapsed auto-fit tracks stay zero under stretch"
            );
        }
    }
}

#[test]
fn fri08_c02r_lanes_track_phase_stretch_excludes_non_auto_collapsed_start_indefinite_and_non_positive()
 {
    assert_fri08_c02r_lanes_stretch_exclusions::<f32>();
    assert_fri08_c02r_lanes_stretch_exclusions::<f64>();
}

#[test]
fn fri08_c02r_lanes_track_phase_architecture_has_no_collection_fit_content_shortcut() {
    let tracks = [
        include_str!("../grid/tracks/flexible.rs"),
        include_str!("../grid/tracks/intrinsic.rs"),
        include_str!("../grid/tracks/mod.rs"),
        include_str!("../grid/tracks/ordinary.rs"),
        include_str!("../grid/tracks/subgrid_intrinsic.rs"),
        include_str!("../grid/tracks/validation.rs"),
    ]
    .concat();
    assert!(
        !tracks.contains("fn resolve_lanes_inline_tracks"),
        "FRI-08.14(5) forbids the lanes collection-wide inline-track shortcut"
    );
    assert!(
        !tracks.contains("fn resolve_fit_content_tracks"),
        "FRI-08.14(5) forbids an orphan collection-wide fit-content resolver"
    );
}

#[test]
fn fri08_c02r_lanes_track_phase_architecture_has_one_auto_maximum_predicate() {
    let tracks = [
        include_str!("../grid/tracks/flexible.rs"),
        include_str!("../grid/tracks/intrinsic.rs"),
        include_str!("../grid/tracks/mod.rs"),
        include_str!("../grid/tracks/ordinary.rs"),
        include_str!("../grid/tracks/subgrid_intrinsic.rs"),
        include_str!("../grid/tracks/validation.rs"),
    ]
    .concat();
    assert!(
        !tracks.contains("fn resolve_lanes_tracks_with_intrinsics")
            && !tracks.contains("fn resolve_lanes_tracks_with_gutters"),
        "FRI-08.14(6) requires lanes stretch to use the canonical maximum-is-Auto predicate"
    );
    assert_eq!(
        tracks.matches("track_has_auto_maximum(").count(),
        1,
        "the maximum-is-Auto predicate has one canonical state-owner use"
    );
    assert!(tracks.contains("fn track_has_auto_maximum"));
}

#[test]
fn fri08_c02r_lanes_track_phase_architecture_has_one_policy_free_final_owner() {
    let orchestration = include_str!("../grid/mod.rs");
    assert!(
        !orchestration.contains("GridTrackSizingPolicy"),
        "FRI-08.14(14) forbids an ordinary-versus-lanes final sizing discriminator"
    );
    assert_eq!(
        orchestration
            .matches("resolve_inline_tracks(input)")
            .count(),
        1
    );
    assert_eq!(
        orchestration
            .matches("resolve_tracks_with_gutters(")
            .count(),
        1
    );
}

#[test]
fn fri08_c02_lanes_negative_auto_fit_accepts_c03_preplacement_collapse() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
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
                grid_column: GridPlacement::try_line(1).expect("first lane track"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("overlapping lane track"),
                ..NodeInput::DEFAULT
            },
        );
    let output = fri08_c02_auto_fit_output(&tree, Size::new(120.0, 20.0), 2);
    assert_eq!((output.location.x, output.size.width), (40.0, 40.0));
}

fn assert_fri08_c06_collapsed_gutter_grid_lanes_interior_collapse<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::GridLanes,
                size: Size::new(
                    PreferredSizeOf::<S>::px(scalar(190.0)),
                    PreferredSizeOf::<S>::px(scalar(20.0)),
                ),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponentOf::<S>::px(scalar(20.0))],
                gap: Size::new(LengthOf::<S>::px(scalar(10.0)), LengthOf::<S>::ZERO),
                justify_content: Some(AlignContent::Start),
                ..NodeInputOf::<S>::default()
            },
        )
        .style(2, NodeInputOf::<S>::default())
        .style(
            3,
            NodeInputOf::<S> {
                grid_column: GridPlacement::try_line(4).expect("fourth lane track"),
                ..NodeInputOf::<S>::default()
            },
        );

    let automatic = fri08_c02_auto_fit_output(&tree, Size::new(scalar(190.0), scalar(20.0)), 2);
    let definite = fri08_c02_auto_fit_output(&tree, Size::new(scalar(190.0), scalar(20.0)), 3);
    assert_eq!(
        (automatic.location.x, automatic.size.width),
        (S::ZERO, scalar(40.0))
    );
    assert_eq!(
        (definite.location.x, definite.size.width),
        (scalar(40.0), scalar(40.0))
    );
    assert_eq!(
        definite.location.x - automatic.location.x - automatic.size.width,
        S::ZERO,
        "grid-lanes retains zero active-gap total when the two interior tracks collapse",
    );
}

#[test]
fn fri08_c06_collapsed_gutter_grid_lanes_interior_collapse_keeps_zero_active_gap_total() {
    assert_fri08_c06_collapsed_gutter_grid_lanes_interior_collapse::<f32>();
    assert_fri08_c06_collapsed_gutter_grid_lanes_interior_collapse::<f64>();
}

fn assert_fri08_c06_collapsed_gutter_inherited_grid_lanes_space_between<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    for reversed in [false, true] {
        let parent_style = NodeInputOf::<S> {
            display: Display::Grid,
            ..NodeInputOf::<S>::default()
        };
        let child_style = NodeInputOf::<S> {
            display: Display::GridLanes,
            direction: if reversed {
                Direction::Rtl
            } else {
                Direction::Ltr
            },
            size: Size::new(
                PreferredSizeOf::px(scalar(190.0)),
                PreferredSizeOf::px(scalar(20.0)),
            ),
            grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack {
                name_components: Vec::new(),
            })],
            grid_template_rows: vec![TrackComponentOf::px(scalar(20.0))],
            gap: Size::new(LengthOf::px(scalar(10.0)), LengthOf::ZERO),
            justify_content: Some(AlignContent::SpaceBetween),
            ..NodeInputOf::<S>::default()
        };
        let parent_gutters = OrdinaryGridAxisGuttersOf::new_zero_adjacent_to_collapsed_tracks(
            4,
            &[false, true, true, false],
            scalar(10.0),
        );
        let parent_geometry = UsedGridAxisGeometryOf::from_active_boundary_gutters(
            vec![scalar(40.0), S::ZERO, S::ZERO, scalar(40.0)],
            parent_gutters.collapsed().to_vec(),
            parent_gutters.active_boundary_after().to_vec(),
            parent_gutters.gutter_after().to_vec(),
        );
        let item = SubgridItemReport {
            node: 1_u32,
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        };
        let context = subgrid_child_parent_context_with_geometry(
            SubgridChildParentContextInput {
                item,
                child_style: &child_style,
                area: GridArea {
                    column: 0,
                    row: 0,
                    column_end: 4,
                    row_end: 1,
                    size: LogicalSizeOf::new(scalar(190.0), scalar(20.0)),
                },
                content_box_size: Size::new(scalar(190.0), scalar(20.0)),
                columns: parent_geometry.sizes(),
                rows: &[scalar(20.0)],
                gap: LogicalSizeOf::new(scalar(10.0), S::ZERO),
                parent_named_columns: &NamedGridLines::new(GridAxisKind::Column, 4),
                parent_named_rows: &NamedGridLines::new(GridAxisKind::Row, 1),
                parent_area_facts: None,
                parent_baseline_groups: &GridBaselineGroups {
                    columns: vec![TrackBaselineGroup::default(); 4],
                    rows: vec![TrackBaselineGroup::default()],
                },
                margin: Edges::all(Some(S::ZERO)),
                border: Edges::all(S::ZERO),
                padding: Edges::all(S::ZERO),
            },
            Some(&parent_geometry),
            None,
        )
        .expect("collapsed grid-lanes gutter policy remains inheritable");

        let mut tree = OracleTreeOf::<S>::new()
            .children(1, [2, 3])
            .style(1, child_style)
            .style(
                2,
                NodeInputOf::<S> {
                    size: Size::new(
                        PreferredSizeOf::px(scalar(40.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    grid_column: GridPlacement::try_line(1).expect("first inherited track"),
                    ..NodeInputOf::<S>::default()
                },
            )
            .style(
                3,
                NodeInputOf::<S> {
                    size: Size::new(
                        PreferredSizeOf::px(scalar(40.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    grid_column: GridPlacement::try_line(4).expect("last inherited track"),
                    ..NodeInputOf::<S>::default()
                },
            );
        compute_grid_with_context(
            &mut tree,
            1,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::new(Some(scalar(190.0)), Some(scalar(20.0))),
                Size::new(Some(scalar(190.0)), Some(scalar(20.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::Grid,
                ),
                Size::new(
                    AvailableOf::Definite(scalar(190.0)),
                    AvailableOf::Definite(scalar(20.0)),
                ),
            ),
            context,
        )
        .expect("inherited grid-lanes layout");

        let first = tree.layout(2).expect("first inherited-track child");
        let last = tree.layout(3).expect("last inherited-track child");
        let recreated_gap = if reversed {
            first.location.x - last.location.x - last.size.width
        } else {
            last.location.x - first.location.x - first.size.width
        };
        assert_eq!(
            recreated_gap,
            S::ZERO,
            "{reversed:?} inherited grid-lanes SpaceBetween must not reactivate a boundary adjacent to collapsed tracks",
        );
    }
}

#[test]
fn fri08_c06_collapsed_gutter_inherited_grid_lanes_space_between_keeps_zero_gap() {
    assert_fri08_c06_collapsed_gutter_inherited_grid_lanes_space_between::<f32>();
    assert_fri08_c06_collapsed_gutter_inherited_grid_lanes_space_between::<f64>();
}

#[test]
fn fri08_c02_lanes_negative_stretch_retains_pre_c02_auto_track_geometry() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Stretch),
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT);
    let output = fri08_c02_auto_fit_output(&tree, Size::new(100.0, 20.0), 2);
    assert_eq!((output.location.x, output.size.width), (0.0, 100.0));
}

fn assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track<S: LayoutScalar>(
    axis: Fri08C02TrackAxis,
) {
    let scalar = S::from_f64;
    let (size, columns, rows) = match axis {
        Fri08C02TrackAxis::Columns => (
            Size::new(
                PreferredSizeOf::px(scalar(120.0)),
                PreferredSizeOf::px(scalar(20.0)),
            ),
            vec![fri08_c02_auto_fit_repeat()],
            vec![TrackComponentOf::px(scalar(20.0))],
        ),
        Fri08C02TrackAxis::Rows => (
            Size::new(
                PreferredSizeOf::px(scalar(20.0)),
                PreferredSizeOf::px(scalar(120.0)),
            ),
            vec![TrackComponentOf::px(scalar(20.0))],
            vec![fri08_c02_auto_fit_repeat()],
        ),
    };
    let first_line = GridPlacement::try_line(1).expect("first repeated track");
    let mut tree = PublicLayoutTreeOf::new().children(1, [2, 3]).style(
        1,
        NodeInputOf {
            display: Display::Grid,
            size,
            grid_template_columns: columns,
            grid_template_rows: rows,
            justify_content: Some(AlignContent::Center),
            align_content: Some(AlignContent::Center),
            ..NodeInputOf::default()
        },
    );
    for node in [2, 3] {
        let (grid_column, grid_row) = match axis {
            Fri08C02TrackAxis::Columns => {
                (first_line, GridPlacement::try_line(1).expect("single row"))
            }
            Fri08C02TrackAxis::Rows => (
                GridPlacement::try_line(1).expect("single column"),
                first_line,
            ),
        };
        tree = tree.style(
            node,
            NodeInputOf {
                grid_column,
                grid_row,
                ..NodeInputOf::default()
            },
        );
    }
    let viewport = match axis {
        Fri08C02TrackAxis::Columns => Size::new(scalar(120.0), scalar(20.0)),
        Fri08C02TrackAxis::Rows => Size::new(scalar(20.0), scalar(120.0)),
    };
    let output = fri08_c02_auto_fit_output(&tree, viewport, 2);
    match axis {
        Fri08C02TrackAxis::Columns => {
            assert_eq!(output.location.x, scalar(40.0));
            assert_eq!(output.size.width, scalar(40.0));
        }
        Fri08C02TrackAxis::Rows => {
            assert_eq!(output.location.y, scalar(40.0));
            assert_eq!(output.size.height, scalar(40.0));
        }
    }
}

#[test]
fn fri08_c02_auto_fit_overlap_collapses_columns_and_rows_in_both_scalar_lanes() {
    assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track::<f32>(
        Fri08C02TrackAxis::Columns,
    );
    assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track::<f32>(Fri08C02TrackAxis::Rows);
    assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track::<f64>(
        Fri08C02TrackAxis::Columns,
    );
    assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track::<f64>(Fri08C02TrackAxis::Rows);
}

#[test]
fn fri08_c02_auto_fit_public_active_subgrid_preserves_space_between_boundary_geometry() {
    let repeat = TrackComponent::Repeat(
        TrackRepetition::auto_fit_components(vec![TrackComponent::px(100.0)])
            .expect("valid fixed auto-fit repetition"),
    );
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .children(2, [3, 4])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(500.0), PreferredSize::px(40.0)),
                grid_template_columns: vec![repeat],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                justify_content: Some(AlignContent::SpaceBetween),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 4).expect("all active parent tracks"),
                grid_row: GridPlacement::try_line(1).expect("single parent row"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                gap: Size::new(Length::px(250.0), Length::ZERO),
                overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                grid_column: GridPlacement::try_line(2).expect("second inherited track"),
                grid_row: GridPlacement::try_line(1).expect("single inherited row"),
                justify_self: Some(AlignItems::Start),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(10.0)),
                grid_column: GridPlacement::try_line(3).expect("third inherited track"),
                grid_row: GridPlacement::try_line(1).expect("single inherited row"),
                justify_self: Some(AlignItems::Start),
                ..NodeInput::DEFAULT
            },
        );

    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequest::viewport(Size::new(
            Available::Definite(500.0),
            Available::Definite(40.0),
        ))
        .expect("finite active inherited-axis viewport"),
    )
    .expect("active inherited-axis layout");
    let subgrid = fri08_c01_placement_output(&batch, 2);
    let second_track_child = fri08_c01_placement_output(&batch, 3);
    let overflowing_child = fri08_c01_placement_output(&batch, 4);
    let overflow = subgrid
        .scroll_geometry
        .expect("active inherited-axis scroll overflow")
        .physical_range()
        .x();

    assert_eq!((subgrid.location.x, subgrid.size.width), (0.0, 500.0));
    assert_eq!(second_track_child.location.x, 275.0);
    assert_eq!(overflowing_child.location.x, 475.0);
    assert_eq!((overflow.minimum(), overflow.maximum()), (0.0, 75.0));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Fri06C07SubgridRtlRow {
    source: &'static str,
    variant: &'static str,
    self_alignment: AlignItems,
    item_alignment: AlignItems,
    box_sizing: BoxSizing,
}

const FRI06_C07_SUBGRID_RTL_ROWS: [Fri06C07SubgridRtlRow; 32] = [
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_baseline_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_baseline_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_center_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_center_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_end_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_end_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_start_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_start_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_baseline_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_baseline_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_center_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_center_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_end_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_end_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_start_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_start_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_baseline_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_baseline_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_center_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_center_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_end_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_end_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_start_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_start_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_baseline_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_baseline_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_center_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_center_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_end_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_end_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_start_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_start_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::ContentBox,
    },
];

#[derive(Clone, Debug)]
struct Fri06C07SubgridRtlCase<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    target_inline_grid: u32,
    target_subgrid: u32,
    target_subject: u32,
}

fn fri06_c07_subgrid_rtl_alignment_index(alignment: AlignItems) -> usize {
    match alignment {
        AlignItems::Start => 0,
        AlignItems::End => 1,
        AlignItems::Center => 2,
        AlignItems::Baseline => 3,
        _ => unreachable!("the finite C07-T3 table contains only the four reviewed alignments"),
    }
}

fn fri06_c07_subgrid_rtl_expected_origin<S: LayoutScalar>(alignment: AlignItems) -> S {
    let value = match alignment {
        AlignItems::Start => 30.0,
        AlignItems::End => 130.0,
        AlignItems::Center => 230.0,
        AlignItems::Baseline => 330.0,
        _ => unreachable!("the finite C07-T3 table contains only the four reviewed alignments"),
    };
    S::from_f64(value)
}

fn fri06_c07_subgrid_rtl_tree<S: LayoutScalar>(
    row: Fri06C07SubgridRtlRow,
) -> Fri06C07SubgridRtlCase<S> {
    let scalar = S::from_f64;
    let root = NodeInputOf {
        display: Display::Block,
        box_sizing: row.box_sizing,
        direction: Direction::Rtl,
        size: Size::new(
            PreferredSizeOf::px(scalar(400.0)),
            PreferredSizeOf::px(scalar(400.0)),
        ),
        ..NodeInputOf::default()
    };
    let alignments = [
        AlignItems::Start,
        AlignItems::End,
        AlignItems::Center,
        AlignItems::Baseline,
    ];
    let mut children = HashMap::new();
    let mut inputs = HashMap::from([(0, root)]);
    let inline_grid_nodes = [1, 2, 3, 4];
    children.insert(0, inline_grid_nodes.to_vec());
    for (self_index, self_alignment) in alignments.into_iter().enumerate() {
        let inline_grid_node = inline_grid_nodes[self_index];
        let first_subgrid_node = 10 + u32::try_from(self_index * 4).expect("finite matrix index");
        let subgrid_nodes = (first_subgrid_node..first_subgrid_node + 4).collect::<Vec<_>>();
        let inline_grid = NodeInputOf {
            display: Display::InlineGrid,
            box_sizing: row.box_sizing,
            direction: Direction::Rtl,
            grid_template_columns: vec![TrackComponentOf::px(scalar(100.0))],
            grid_auto_rows: vec![TrackComponentOf::px(scalar(100.0))],
            atomic_inline_participation: Some(
                AtomicInlineParticipationOf::try_new(
                    BidiLevel::try_new(1).expect("RTL bidi level is valid"),
                    InlineBreakOpportunityOf::prohibited(),
                )
                .expect("atomic inline participation is valid"),
            ),
            ..NodeInputOf::default()
        };
        inputs.insert(inline_grid_node, inline_grid);
        children.insert(inline_grid_node, subgrid_nodes.clone());

        for (item_index, item_alignment) in alignments.into_iter().enumerate() {
            let subgrid_node = subgrid_nodes[item_index];
            let subject_node =
                100 + u32::try_from(self_index * 4 + item_index).expect("finite matrix index");
            let subgrid = NodeInputOf {
                display: Display::Grid,
                box_sizing: row.box_sizing,
                direction: Direction::Rtl,
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                align_self: Some(self_alignment),
                justify_self: Some(item_alignment),
                margin: Edges::all(LengthAutoOf::px(scalar(10.0))),
                border: Edges::all(LengthOf::px(scalar(10.0))),
                padding: Edges::all(LengthOf::px(scalar(10.0))),
                ..NodeInputOf::default()
            };
            let subject = NodeInputOf {
                display: Display::Block,
                box_sizing: row.box_sizing,
                direction: Direction::Rtl,
                size: Size::new(
                    PreferredSizeOf::px(scalar(40.0)),
                    PreferredSizeOf::px(scalar(40.0)),
                ),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            };
            inputs.insert(subgrid_node, subgrid);
            inputs.insert(subject_node, subject);
            children.insert(subgrid_node, vec![subject_node]);
            children.insert(subject_node, vec![]);
        }
    }

    let target_self_index = fri06_c07_subgrid_rtl_alignment_index(row.self_alignment);
    let target_item_index = fri06_c07_subgrid_rtl_alignment_index(row.item_alignment);
    let target_subgrid =
        10 + u32::try_from(target_self_index * 4 + target_item_index).expect("finite matrix index");
    let target_subject = 100
        + u32::try_from(target_self_index * 4 + target_item_index).expect("finite matrix index");

    let mut tree = PublicLayoutTreeOf::new();
    for (node, node_children) in children {
        tree.insert_children(node, node_children);
    }
    for (node, input) in inputs {
        tree.insert_input(node, LayoutInputOf::box_input(input));
    }

    Fri06C07SubgridRtlCase {
        tree,
        target_inline_grid: inline_grid_nodes[target_self_index],
        target_subgrid,
        target_subject,
    }
}

fn fri06_c07_subgrid_rtl_mismatches<S: LayoutScalar>() -> Vec<String> {
    assert_eq!(FRI06_C07_SUBGRID_RTL_ROWS.len(), 32);
    let unique_rows = FRI06_C07_SUBGRID_RTL_ROWS
        .iter()
        .map(|row| (row.source, row.variant))
        .collect::<HashSet<_>>();
    assert_eq!(unique_rows.len(), 32);

    let mut physical_inline_mismatches = Vec::new();
    for row in FRI06_C07_SUBGRID_RTL_ROWS {
        let case = fri06_c07_subgrid_rtl_tree::<S>(row);
        let batch = compute_layout(
            &case.tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(400.0))))
                .expect("RTL subgrid viewport is valid"),
        )
        .expect("RTL subgrid alignment computes through the public front door");
        let inline_grid =
            fri06_c07_height_output(batch.unrounded_entries(), case.target_inline_grid);
        let subgrid = fri06_c07_height_output(batch.unrounded_entries(), case.target_subgrid);
        let subject = fri06_c07_height_output(batch.unrounded_entries(), case.target_subject);

        assert_eq!(subject.size, Size::splat(S::from_f64(40.0)), "{row:?}");
        let root_relative_origin = Point::new(
            inline_grid.location.x + subgrid.location.x + subject.location.x,
            inline_grid.location.y + subgrid.location.y + subject.location.y,
        );
        assert_eq!(
            root_relative_origin.y,
            S::from_f64(match row.item_alignment {
                AlignItems::Start => 30.0,
                AlignItems::End => 130.0,
                AlignItems::Center => 230.0,
                AlignItems::Baseline => 330.0,
                _ => unreachable!("finite table alignment"),
            }),
            "{row:?} must preserve the logical block origin"
        );
        let expected_x = fri06_c07_subgrid_rtl_expected_origin(row.self_alignment);
        if root_relative_origin.x != expected_x {
            physical_inline_mismatches.push(format!(
                "{row:?}: inline-grid x={:?}, subgrid x={:?}, subject x={:?}, root-relative x={:?}, expected x={expected_x:?}",
                inline_grid.location.x,
                subgrid.location.x,
                subject.location.x,
                root_relative_origin.x,
            ));
        }
    }
    physical_inline_mismatches
}

#[test]
fn fri06_c07_subgrid_rtl_exact_thirty_two_rows_mirror_only_physical_inline_origin() {
    let mut mismatches = fri06_c07_subgrid_rtl_mismatches::<f32>();
    mismatches.extend(fri06_c07_subgrid_rtl_mismatches::<f64>());
    assert!(
        mismatches.is_empty(),
        "every row in both scalar lanes must mirror the logical inline origin exactly once: {mismatches:#?}"
    );
}

#[test]
fn fri05_c05_grid_geometry_measurement_remains_absent_while_grid_lanes_publish() {
    let style = NodeInput {
        display: Display::Grid,
        size: Size::new(PreferredSize::px(40.0), PreferredSize::px(30.0)),
        overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
        grid_template_columns: vec![TrackComponent::px(40.0)],
        grid_template_rows: vec![TrackComponent::px(30.0)],
        ..NodeInput::default()
    };
    let mut measurement = OracleTree::new().children(0, []).style(0, style.clone());
    let measurement_output = compute_grid(
        &mut measurement,
        0,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(40.0), Some(30.0)),
            Size::new(Some(40.0), Some(30.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(40.0), Available::definite(30.0)),
        ),
    )
    .unwrap();
    assert!(measurement_output.scroll_geometry.is_none());

    let mut lanes = OracleTree::new().children(0, []).style(
        0,
        NodeInput {
            display: Display::GridLanes,
            ..style
        },
    );
    let lanes_output = compute_grid(
        &mut lanes,
        0,
        fri05_c05_grid_sizing_input(Size::new(Some(40.0), Some(30.0))),
    )
    .unwrap();
    assert!(lanes_output.scroll_geometry.is_some());
}

#[test]
fn fri05_c05_grid_lanes_geometry_publishes_reservation_range_and_target_in_all_flows() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
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
                        display: Display::GridLanes,
                        writing_mode,
                        direction,
                        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                        overflow: computed_overflow(overflow, overflow),
                        item_is_replaced: replaced,
                        scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                        scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                        justify_content: Some(AlignContent::End),
                        align_content: Some(AlignContent::Center),
                        grid_template_columns: vec![TrackComponent::px(160.0)],
                        grid_template_rows: vec![TrackComponent::px(140.0)],
                        scroll_margin: ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap(),
                        scroll_snap_align: ScrollSnapAlign::new(
                            ScrollSnapAlignValue::End,
                            ScrollSnapAlignValue::Center,
                        ),
                        scroll_snap_stop: ScrollSnapStop::Always,
                        ..NodeInput::default()
                    },
                );

                let output = compute_grid(
                    &mut tree,
                    0,
                    fri05_c05_grid_sizing_input(Size::new(Some(100.0), Some(80.0))),
                )
                .expect("grid-lanes geometry computes");
                let geometry = output
                    .scroll_geometry
                    .expect("performed grid-lanes publishes canonical geometry");
                assert_eq!(geometry.used_overflow_x(), expected_used);
                assert_eq!(geometry.used_overflow_y(), expected_used);
                assert_eq!(geometry.target().border_box(), geometry.border_box());
                assert_eq!(
                    geometry.target().scroll_margin(),
                    ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap()
                );
                assert_eq!(geometry.target().snap_stop(), ScrollSnapStop::Always);
                assert!(
                    output.content_size.width >= geometry.canonical_content_size().unwrap().width
                        && output.content_size.height
                            >= geometry.canonical_content_size().unwrap().height
                );

                if matches!(overflow, Overflow::Scroll) && !replaced {
                    let flow_range = geometry
                        .flow_axes()
                        .flow_relative_scroll_range(geometry.physical_range());
                    assert!(flow_range.inline().minimum() < 0.0);
                    assert_eq!(flow_range.inline().maximum(), 0.0);
                    assert!(flow_range.block().minimum() < 0.0);
                    assert_eq!(flow_range.block().maximum(), 0.0);
                }
            }
        }
    }
}

#[test]
fn fri05_c05_grid_lanes_geometry_reserves_forced_stable_both_zero_tiny_and_auto() {
    for (overflow, gutter, width, size, expected_edges) in [
        (
            computed_overflow(Overflow::Scroll, Overflow::Scroll),
            ScrollbarGutter::Auto,
            10.0,
            Size::new(100.0, 80.0),
            (false, true, true),
        ),
        (
            computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::Stable,
            10.0,
            Size::new(100.0, 80.0),
            (false, true, false),
        ),
        (
            computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::StableBothEdges,
            10.0,
            Size::new(100.0, 80.0),
            (true, true, false),
        ),
        (
            computed_overflow(Overflow::Scroll, Overflow::Scroll),
            ScrollbarGutter::Auto,
            0.0,
            Size::new(100.0, 80.0),
            (false, false, false),
        ),
        (
            computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::StableBothEdges,
            8.0,
            Size::new(10.0, 6.0),
            (true, true, false),
        ),
    ] {
        let mut tree = OracleTree::new().children(0, []).style(
            0,
            NodeInput {
                display: Display::GridLanes,
                size: size.map(PreferredSize::px),
                overflow,
                scrollbar_gutter: gutter,
                scrollbar_width: ScrollbarWidth::try_new(width).unwrap(),
                grid_template_columns: vec![TrackComponent::px(1.0)],
                grid_template_rows: vec![TrackComponent::px(1.0)],
                ..NodeInput::default()
            },
        );
        let geometry = compute_grid(&mut tree, 0, fri05_c05_grid_sizing_input(size.map(Some)))
            .unwrap()
            .scroll_geometry
            .unwrap();
        assert_eq!(geometry.gutters().left().is_some(), expected_edges.0);
        assert_eq!(geometry.gutters().right().is_some(), expected_edges.1);
        assert_eq!(geometry.gutters().bottom().is_some(), expected_edges.2);
        if size == Size::new(10.0, 6.0) {
            assert_eq!(geometry.content_box().size(), Size::new(0.0, 6.0));
        }
    }

    let mut auto = OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                grid_template_columns: vec![TrackComponent::px(95.0)],
                grid_template_rows: vec![TrackComponent::px(120.0)],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
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
    let geometry = compute_grid(
        &mut auto,
        0,
        fri05_c05_grid_sizing_input(Size::splat(Some(100.0))),
    )
    .unwrap()
    .scroll_geometry
    .unwrap();
    assert!(geometry.gutters().right().is_some());
    assert!(geometry.gutters().bottom().is_some());
    assert!(auto.inputs(1).iter().all(|input| {
        input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
    }));
}

#[test]
fn fri04_c04_grid_dispatch_grid_and_lanes_cover_all_direct_and_keyword_basis_payloads() {
    let sizing =
        || SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite calculation"));
    let calc = || CalcSizeCalculation::value(LengthPercentageOf::ZERO);

    for (display, algorithm) in [
        (Display::Grid, SizingAlgorithm::Grid),
        (Display::InlineGrid, SizingAlgorithm::Grid),
        (Display::GridLanes, SizingAlgorithm::GridLanes),
        (Display::InlineGridLanes, SizingAlgorithm::GridLanes),
    ] {
        for (value, behavior) in [
            (PreferredSize::STRETCH, SizingBehavior::Stretch),
            (PreferredSize::FIT_CONTENT, SizingBehavior::FitContent),
            (PreferredSize::CONTAIN, SizingBehavior::Contain),
            (
                PreferredSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Preferred(value),
                    PhysicalAxis::Horizontal,
                ),
                SizingProperty::Preferred,
                behavior,
                algorithm,
                PhysicalAxis::Horizontal,
                0,
            );
        }
        for (value, behavior) in [
            (MinSize::MIN_CONTENT, SizingBehavior::MinContent),
            (MinSize::MAX_CONTENT, SizingBehavior::MaxContent),
            (MinSize::STRETCH, SizingBehavior::Stretch),
            (MinSize::FIT_CONTENT, SizingBehavior::FitContent),
            (MinSize::CONTAIN, SizingBehavior::Contain),
            (
                MinSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Minimum(value),
                    PhysicalAxis::Vertical,
                ),
                SizingProperty::Minimum,
                behavior,
                algorithm,
                PhysicalAxis::Vertical,
                0,
            );
        }
        for (value, behavior) in [
            (MaxSize::MIN_CONTENT, SizingBehavior::MinContent),
            (MaxSize::MAX_CONTENT, SizingBehavior::MaxContent),
            (MaxSize::STRETCH, SizingBehavior::Stretch),
            (MaxSize::FIT_CONTENT, SizingBehavior::FitContent),
            (MaxSize::CONTAIN, SizingBehavior::Contain),
            (
                MaxSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Maximum(value),
                    PhysicalAxis::Horizontal,
                ),
                SizingProperty::Maximum,
                behavior,
                algorithm,
                PhysicalAxis::Horizontal,
                0,
            );
        }

        for (basis, behavior) in [
            (PreferredSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
            (
                PreferredSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                PreferredSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (
                PreferredSizeCalcBasis::Stretch,
                CalcSizeBehaviorBasis::Stretch,
            ),
            (
                PreferredSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (
                PreferredSizeCalcBasis::Contain,
                CalcSizeBehaviorBasis::Contain,
            ),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Preferred(
                        PreferredSize::calc_size(basis, calc()).expect("valid calc-size"),
                    ),
                    PhysicalAxis::Vertical,
                ),
                SizingProperty::Preferred,
                SizingBehavior::CalcSize(behavior),
                algorithm,
                PhysicalAxis::Vertical,
                0,
            );
        }
        for (basis, behavior) in [
            (MinSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
            (
                MinSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                MinSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (MinSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
            (
                MinSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (MinSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Minimum(
                        MinSize::calc_size(basis, calc()).expect("valid calc-size"),
                    ),
                    PhysicalAxis::Horizontal,
                ),
                SizingProperty::Minimum,
                SizingBehavior::CalcSize(behavior),
                algorithm,
                PhysicalAxis::Horizontal,
                0,
            );
        }
        for (basis, behavior) in [
            (MaxSizeCalcBasis::None, CalcSizeBehaviorBasis::None),
            (
                MaxSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                MaxSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (MaxSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
            (
                MaxSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (MaxSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Maximum(
                        MaxSize::calc_size(basis, calc()).expect("valid calc-size"),
                    ),
                    PhysicalAxis::Vertical,
                ),
                SizingProperty::Maximum,
                SizingBehavior::CalcSize(behavior),
                algorithm,
                PhysicalAxis::Vertical,
                0,
            );
        }
    }
}

#[test]
fn grid_lanes_order_modified_sequence_drives_running_offsets_and_intrinsic_contributions_in_both_scalar_lanes()
 {
    assert_grid_lanes_order_modified_sequence::<f32>();
    assert_grid_lanes_order_modified_sequence::<f64>();
}

fn assert_grid_lanes_order_modified_sequence<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let item_style = |order, width, height| NodeInputOf::<S> {
        item_order: ItemOrder::new(order),
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        grid_column: GridPlacement::try_line(1).expect("one is a valid grid line"),
        ..NodeInputOf::default()
    };
    let mut tree = OracleTreeOf::<S>::new()
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
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::px(S::ZERO)],
                grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                grid_column: GridPlacement::try_line_span(1, 2)
                    .expect("line one with span two is valid"),
                ..item_style(2, 100.0, 10.0)
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::None,
                item_order: ItemOrder::new(i32::MIN),
                ..item_style(0, 500.0, 500.0)
            },
        )
        .style(3, item_style(-1, 80.0, 20.0))
        .style(
            4,
            NodeInputOf {
                position: Position::Absolute,
                item_order: ItemOrder::new(i32::MIN),
                ..item_style(0, 500.0, 500.0)
            },
        )
        .style(5, item_style(0, 0.0, 30.0))
        .style(6, item_style(2, 0.0, 40.0));

    let placement = |column, in_flow| ResolvedGridItemPlacement {
        column,
        row: GridPlacement::try_line(1).expect("one is a valid grid line"),
        absolute_column: column,
        absolute_row: GridPlacement::try_line(1).expect("one is a valid grid line"),
        in_flow,
    };
    let item_inputs = (1..=6)
        .map(|child| grid_child_input!(tree.node_input(child)))
        .collect();
    let placements = GridPlacementContext::new_with_child_inputs(
        vec![1, 2, 3, 4, 5, 6],
        vec![
            placement(
                GridPlacement::try_line_span(1, 2).expect("line-span placement is valid"),
                true,
            ),
            placement(GridPlacement::AUTO, false),
            placement(
                GridPlacement::try_line(1).expect("one is a valid grid line"),
                true,
            ),
            placement(GridPlacement::AUTO, false),
            placement(
                GridPlacement::try_line(1).expect("one is a valid grid line"),
                true,
            ),
            placement(
                GridPlacement::try_line(1).expect("one is a valid grid line"),
                true,
            ),
        ],
        item_inputs,
    )
    .with_order_modified_indexes(vec![
        SourceIndex::new(2),
        SourceIndex::new(4),
        SourceIndex::new(0),
        SourceIndex::new(5),
    ]);
    let constants = Constants {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        explicit_definite_content_size: Size::NONE,
        node_outer_size: Size::NONE,
        node_inner_size: Size::NONE,
        node_min_size: Size::NONE,
        node_max_size: Size::NONE,
        available_inner_size: Size::NONE,
        content_box_inset: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let tracks = [TrackSizingOf::AUTO, TrackSizingOf::AUTO];
    let container_input = tree.node_input(0).clone();
    let container_style = grid_container_projection!(&container_input);
    let subgrid_report = collect_subgrid_report(&container_style, &placements);
    let intrinsic_sizes = lane_intrinsic_track_sizes(
        &mut tree,
        0,
        LaneIntrinsicTrackSizeInput {
            container_style: &container_style,
            constants: &constants,
            axis: GridAxisKind::Column,
            tracks: &tracks,
            gap: S::ZERO,
            available: AvailableOf::MAX_CONTENT,
            available_basis: None,
            gutters: None,
            lines: GridLines {
                column_explicit_start: 0,
                column_explicit_count: 2,
                row_explicit_start: 0,
                row_explicit_count: 1,
            },
            placements: &placements,
            subgrid_report: &subgrid_report,
        },
    )
    .expect("intrinsic collection succeeds")
    .expect("intrinsic placement is valid");
    assert_eq!(
        intrinsic_sizes,
        vec![S::from_f64(80.0), S::from_f64(20.0)],
        "overlapping definite spans use the ordinary contribution leveling phase in order-modified sequence"
    );

    let output = compute_grid(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
        ),
    )
    .expect("order-modified grid-lanes layout succeeds");

    assert_eq!(output.size.width, S::from_f64(130.0));
    for (node, expected_y) in [(3, 0.0), (5, 20.0), (1, 50.0), (6, 60.0)] {
        let layout = tree.layout(node).expect("in-flow lane child is staged");
        assert_eq!(layout.source_index, SourceIndex::new((node - 1) as usize));
        assert_eq!(layout.location.y, S::from_f64(expected_y));
    }
    assert_eq!(
        Traverse::children(&tree, 0).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6],
        "final traversal stays in source order"
    );
    assert!(tree.inputs(2).iter().any(|input| {
        input.run_mode() == RunMode::PerformHiddenLayout
            && input.parent_formatting_context() == ParentFormattingContext::Grid
    }));
    assert_eq!(
        tree.layout(4)
            .expect("absolute lane child is staged")
            .source_index,
        SourceIndex::new(3)
    );
}

#[test]
fn grid_lanes_replaced_normal_preplacement_starts_while_explicit_stretch_remains_in_both_scalar_lanes()
 {
    assert_grid_lanes_replaced_normal_preplacement::<f32>();
    assert_grid_lanes_replaced_normal_preplacement::<f64>();
}

fn assert_grid_lanes_replaced_normal_preplacement<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    for grid_axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (label, replaced, item_alignment, container_alignment, stretches) in [
            ("replaced normal", true, None, None, false),
            ("non-replaced normal", false, None, None, true),
            (
                "explicit item stretch",
                true,
                Some(AlignItems::Stretch),
                None,
                true,
            ),
            (
                "explicit container stretch",
                true,
                None,
                Some(AlignItems::Stretch),
                true,
            ),
        ] {
            let grid_axis_size = match grid_axis {
                GridAxisKind::Column => S::from_f64(100.0),
                GridAxisKind::Row => S::from_f64(80.0),
            };
            let measured =
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(30.0), S::from_f64(20.0)));
            let container_style = NodeInputOf {
                justify_items: container_alignment,
                align_items: container_alignment,
                ..NodeInputOf::default()
            };
            let child_style = NodeInputOf {
                item_is_replaced: replaced,
                justify_self: item_alignment,
                align_self: item_alignment,
                ..NodeInputOf::default()
            };
            let mut tree = OracleTreeOf::<S>::new()
                .children(1, [])
                .style(1, child_style.clone())
                .measure(1, measured);
            let container_projection = grid_container_projection!(&container_style);
            let child_projection = grid_item_projection!(&child_style);
            let constants = Constants {
                flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                explicit_definite_content_size: Size::NONE,
                node_outer_size: Size::NONE,
                node_inner_size: Size::NONE,
                node_min_size: Size::NONE,
                node_max_size: Size::NONE,
                available_inner_size: Size::NONE,
                content_box_inset: Edges::ZERO,
                padding: Edges::ZERO,
                border: Edges::ZERO,
            };
            measure_lane_axis_margin_box_with_grid_axis(
                &mut tree,
                1,
                LaneAxisMarginBoxMeasureInput {
                    child_style: &child_projection,
                    container_style: &container_projection,
                    constants: &constants,
                    lane_axis: match grid_axis {
                        GridAxisKind::Column => GridAxisKind::Row,
                        GridAxisKind::Row => GridAxisKind::Column,
                    },
                    containing_block: GridLanesItemContainingBlockOf::new(
                        constants.flow_axes,
                        grid_axis,
                        grid_axis_size,
                        LogicalSizeOf::new(None, None),
                    ),
                },
            )
            .expect("grid-lanes pre-placement measurement succeeds");

            let input = tree
                .inputs(1)
                .last()
                .expect("measurement input is recorded");
            let expected = stretches.then_some(grid_axis_size);
            assert_eq!(
                match grid_axis {
                    GridAxisKind::Column => input.known().width,
                    GridAxisKind::Row => input.known().height,
                },
                expected,
                "{label} must resolve pre-placement {grid_axis:?} alignment"
            );
        }
    }
}

#[test]
fn ordinary_grid_order_modified_placement_precedes_mixed_phases_and_preserves_source_identity_in_both_scalar_lanes()
 {
    assert_ordinary_grid_order_modified_placement::<f32>();
    assert_ordinary_grid_order_modified_placement::<f64>();
}

fn assert_ordinary_grid_order_modified_placement<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    for auto_flow in [
        GridAutoFlow::Row,
        GridAutoFlow::RowDense,
        GridAutoFlow::Column,
        GridAutoFlow::ColumnDense,
    ] {
        let column_flow = auto_flow.is_column();
        let item_style = |order| NodeInputOf::<S> {
            item_order: ItemOrder::new(order),
            size: Size::splat_clone(PreferredSizeOf::px(S::from_f64(10.0))),
            ..NodeInputOf::default()
        };
        let definite_major = |order| NodeInputOf::<S> {
            grid_column: if column_flow {
                GridPlacement::try_line(1).expect("one is a valid grid line")
            } else {
                GridPlacement::AUTO
            },
            grid_row: if column_flow {
                GridPlacement::AUTO
            } else {
                GridPlacement::try_line(1).expect("one is a valid grid line")
            },
            ..item_style(order)
        };
        let mut tree = OracleTreeOf::<S>::new()
            .children(0, [1, 2, 3, 4, 5, 6, 7, 8])
            .children(1, [])
            .children(2, [])
            .children(3, [])
            .children(4, [])
            .children(5, [])
            .children(6, [])
            .children(7, [])
            .children(8, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::splat_clone(PreferredSizeOf::px(S::from_f64(80.0))),
                    grid_template_columns: vec![TrackComponentOf::px(S::from_f64(20.0)); 4],
                    grid_template_rows: vec![TrackComponentOf::px(S::from_f64(20.0)); 4],
                    grid_auto_flow: auto_flow,
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
            .style(
                3,
                NodeInputOf {
                    grid_column: GridPlacement::try_line(2).expect("two is a valid grid line"),
                    grid_row: GridPlacement::try_line(2).expect("two is a valid grid line"),
                    item_order: ItemOrder::new(i32::MAX),
                    ..item_style(0)
                },
            )
            .style(4, definite_major(2))
            .style(
                5,
                NodeInputOf {
                    position: Position::Absolute,
                    item_order: ItemOrder::new(i32::MIN),
                    ..item_style(0)
                },
            )
            .style(6, definite_major(-1))
            .style(7, item_style(0))
            .style(8, item_style(2));

        let computation = compute_grid_with_report(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(S::from_f64(80.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::splat(AvailableOf::definite(S::from_f64(80.0))),
            ),
        )
        .expect("order-modified ordinary-grid layout succeeds");
        assert!(computation.report().is_empty());

        let expected_cells = if column_flow {
            [
                (6, 0, 0),
                (3, 1, 1),
                (4, 0, 1),
                (7, 0, 2),
                (1, 0, 3),
                (8, 1, 0),
            ]
        } else {
            [
                (6, 0, 0),
                (3, 1, 1),
                (4, 1, 0),
                (7, 2, 0),
                (1, 3, 0),
                (8, 0, 1),
            ]
        };
        for (node, column, row) in expected_cells {
            let layout = tree.layout(node).expect("in-flow child layout is staged");
            assert_eq!(layout.source_index, SourceIndex::new((node - 1) as usize));
            assert_eq!(
                layout.location,
                Point::new(S::from_usize(column * 20), S::from_usize(row * 20)),
                "{auto_flow:?} node {node} must use order-modified placement"
            );
        }
        assert!(
            tree.inputs(2)
                .iter()
                .any(|input| input.run_mode() == RunMode::PerformHiddenLayout),
            "hidden children remain outside the ordinary-grid permutation"
        );
        assert_eq!(
            tree.layout(5)
                .expect("absolute child layout is staged")
                .source_index,
            SourceIndex::new(4)
        );
    }
}

#[test]
fn fri04_c03_grid_track_grid_and_lanes_consume_nested_properties_on_both_axes_and_clamp_results() {
    for display in [Display::Grid, Display::GridLanes] {
        let mut tree = OracleTree::new().children(1, []).style(
            1,
            NodeInput {
                display,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_grid_track_nested(20.0, 80.0, 120.0)),
                    PreferredSize::calculation(fri04_c03_grid_track_nested(20.0, 70.0, 120.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_grid_track_nested(40.0, 90.0, 110.0)),
                    MinSize::calculation(fri04_c03_grid_track_nested(30.0, 60.0, 90.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_grid_track_nested(30.0, 85.0, 100.0)),
                    MaxSize::calculation(fri04_c03_grid_track_nested(30.0, 65.0, 100.0)),
                ),
                ..NodeInput::default()
            },
        );

        let output = compute_grid(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(200.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::definite(200.0)),
            ),
        )
        .expect("grid property calculations resolve");

        assert_eq!(output.size, Size::new(85.0, 65.0), "{display:?}");

        let mut negative_tree = OracleTree::new().children(1, []).style(
            1,
            NodeInput {
                display,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_grid_track_nested(-40.0, -20.0, -10.0)),
                    PreferredSize::calculation(fri04_c03_grid_track_nested(-30.0, -15.0, -5.0)),
                ),
                ..NodeInput::default()
            },
        );
        let negative = compute_grid(
            &mut negative_tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(200.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::definite(200.0)),
            ),
        )
        .expect("negative complete calculations clamp after evaluation");
        assert_eq!(negative.size, Size::ZERO, "{display:?}");
    }
}

#[test]
fn fri04_c03_grid_track_grid_and_lanes_preserve_missing_and_definite_property_bases() {
    for display in [Display::Grid, Display::GridLanes] {
        let style = NodeInput {
            display,
            size: Size::new(
                PreferredSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.5, 80.0)),
                PreferredSize::calculation(fri04_c03_grid_track_percentage_nested(
                    10.0, 0.25, 80.0,
                )),
            ),
            min_size: Size::new(
                MinSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.6, 80.0)),
                MinSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.3, 80.0)),
            ),
            max_size: Size::new(
                MaxSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.625, 80.0)),
                MaxSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.325, 80.0)),
            ),
            ..NodeInput::default()
        };
        let mut definite_tree = OracleTree::new().children(1, []).style(1, style.clone());
        let definite = compute_grid(
            &mut definite_tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(120.0), Some(200.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(Available::definite(120.0), Available::definite(200.0)),
            ),
        )
        .expect("definite property bases resolve");
        assert_eq!(definite.size.width, 72.0, "{display:?} width");
        assert!(
            (definite.size.height - 60.0).abs() <= 0.000_01,
            "{display:?} height: {}",
            definite.size.height
        );

        let mut missing_tree = OracleTree::new().children(1, []).style(1, style);
        let missing = compute_grid(
            &mut missing_tree,
            1,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .expect("missing property bases retain intrinsic sizing");
        assert_eq!(missing.size, Size::ZERO, "{display:?}");
    }
}

#[test]
fn fri04_c03_grid_track_grid_and_lanes_propagate_invalid_property_numeric() {
    let invalid = SizingCalculation::max(vec![
        fri04_c03_grid_track_value(10.0),
        SizingCalculation::value(invalid_numeric_lp()),
    ])
    .expect("nested maximum is nonempty");
    for display in [Display::Grid, Display::GridLanes] {
        for property in ["preferred", "minimum", "maximum"] {
            for axis in [PhysicalAxis::Horizontal, PhysicalAxis::Vertical] {
                let mut style = NodeInput {
                    display,
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                    ..NodeInput::default()
                };
                match (property, axis) {
                    ("preferred", PhysicalAxis::Horizontal) => {
                        style.size.width = PreferredSize::calculation(invalid.clone())
                    }
                    ("preferred", PhysicalAxis::Vertical) => {
                        style.size.height = PreferredSize::calculation(invalid.clone())
                    }
                    ("minimum", PhysicalAxis::Horizontal) => {
                        style.min_size.width = MinSize::calculation(invalid.clone())
                    }
                    ("minimum", PhysicalAxis::Vertical) => {
                        style.min_size.height = MinSize::calculation(invalid.clone())
                    }
                    ("maximum", PhysicalAxis::Horizontal) => {
                        style.max_size.width = MaxSize::calculation(invalid.clone())
                    }
                    ("maximum", PhysicalAxis::Vertical) => {
                        style.max_size.height = MaxSize::calculation(invalid.clone())
                    }
                    _ => unreachable!("property and physical axis matrix is exhaustive"),
                }
                let mut tree = OracleTree::new().children(1, []).style(1, style);
                let error = compute_grid(
                    &mut tree,
                    1,
                    ComputeInput::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        Size::splat(Some(f32::MAX)),
                        ContainingLayoutContext::new(
                            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                            ParentFormattingContext::NoParent,
                        ),
                        Size::splat(Available::definite(f32::MAX)),
                    ),
                )
                .expect_err("invalid nested grid property numeric must fail");
                assert!(
                    matches!(
                        error.kind(),
                        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { value })
                            if value.is_infinite()
                    ),
                    "{display:?} {property} {axis:?}"
                );
            }
        }
    }
}

#[test]
fn fri04_c03_grid_track_lanes_resolve_nested_track_bases_and_propagate_invalid_numeric() {
    let report = lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: None,
        gap: 0.0,
        tracks: vec![TrackSizing::new(
            MinTrackSizing::Calculation(fri04_c03_grid_track_nested(20.0, 40.0, 60.0)),
            MaxTrackSizing::Auto,
        )],
        content_sized_tracks: vec![0],
        items: Vec::new(),
    })
    .expect("nested lane track sizing resolves")
    .expect("nested lane placement is valid");
    assert_eq!(report.final_track_sizes, [40.0]);

    let invalid = SizingCalculation::max(vec![
        fri04_c03_grid_track_value(10.0),
        SizingCalculation::value(invalid_numeric_lp()),
    ])
    .expect("nested maximum is nonempty");
    let error = lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxisKind::Row,
        available: Some(f32::MAX),
        gap: 0.0,
        tracks: vec![TrackSizing::new(
            MinTrackSizing::Calculation(invalid),
            MaxTrackSizing::Auto,
        )],
        content_sized_tracks: vec![0],
        items: Vec::new(),
    })
    .expect_err("invalid nested lane track numeric must fail");
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { value })
            if value.is_infinite()
    ));
}

#[test]
fn lane_intrinsic_item_exposes_exactly_one_kind() {
    let contribution = LaneContributionFacts {
        min_content: 1.0,
        max_content: 2.0,
        min_size: 0.0,
        automatic_minimum_applies: true,
    };
    let span = LaneTrackSpanLength::new(2).expect("span should be nonzero");
    let item = LaneIntrinsicItem::indefinite("item", span, contribution);

    assert!(matches!(
        item.kind(),
        LaneIntrinsicItemKind::Indefinite { span } if span.get() == 2
    ));
}

#[test]
fn lane_intrinsic_item_rejects_malformed_definite_span_without_track_context() {
    let contribution = LaneContributionFacts {
        min_content: 1.0,
        max_content: 2.0,
        min_size: 0.0,
        automatic_minimum_applies: true,
    };
    let span = LaneTrackSpan::new(0, 1);
    let err = LaneIntrinsicItem::definite("item", span, contribution)
        .expect_err("malformed definite span should be rejected at construction");

    assert_eq!(err, LanePlacementError::InvalidDefiniteLaneSpan { span });
}

#[test]
fn lane_track_span_length_rejects_zero() {
    assert!(LaneTrackSpanLength::new(0).is_none());
}

#[test]
fn lane_errors_carry_context() {
    let err = place_lanes(LanePlacementInput {
        grid_axis_tracks: 2,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: GridFlowTolerance::Infinite,
        tolerance_basis: 0.0,
        items: vec![LaneItem {
            item: "item",
            grid_axis_span: 1,
            definite_grid_axis_start: Some(0),
            lane_axis_margin_box: 1.0,
        }],
    })
    .expect_err("zero grid axis start should be rejected with context");

    assert_eq!(err, LanePlacementError::InvalidGridAxisStart { start: 0 });
}

#[test]
fn lanes_reject_invalid_raw_tolerance_basis() {
    let err = place_lanes(LanePlacementInput::<&str> {
        grid_axis_tracks: 2,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: GridFlowTolerance::Percent(0.25),
        tolerance_basis: f32::NAN,
        items: Vec::new(),
    })
    .expect_err("invalid raw tolerance basis should return a typed error");

    assert_eq!(err, LanePlacementError::InvalidGridFlowToleranceBasis);
}

#[test]
fn grid_lanes_content_size_uses_measured_lane_margin_boxes() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(
            &mut self,
            node: u32,
            input: ComputeInput,
        ) -> LayoutResultOf<u32, ComputeOutput, Scalar> {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return Ok(self.outputs[&node]);
            }

            match node_input.display.inner_display() {
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
                Display::Flex => compute_flex(self, node, input),
                Display::None => Ok(ComputeOutput::HIDDEN),
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
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

    impl Compute for RecursiveTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
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
            self.compute_node(node, input)
        }
    }

    let mut tree = RecursiveTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::GridLanes,
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![TrackComponent::px(0.0)],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        );
    }
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));
    tree.outputs
        .insert(3, ComputeOutput::from_outer_size(Size::new(20.0, 15.0)));

    let output = tree
        .compute_node(
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

    assert_eq!(output.content_size, Size::new(20.0, 30.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 15.0));
}

#[test]
fn grid_lanes_content_size_preserves_resolved_track_sum() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(80.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

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

    assert_eq!(output.content_size, Size::new(80.0, 20.0));
}

#[test]
fn named_grid_lanes_use_resolved_raw_grid_axis_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(child.location, Point::new(40.0, 0.0));
    assert_eq!(child.size, Size::new(40.0, 20.0));
}

#[test]
fn named_grid_lanes_intrinsic_sizing_uses_resolved_raw_grid_axis_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![
                    TrackComponent::AUTO,
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::AUTO,
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(10.0, 20.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(50.0, 20.0)));

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
    let named = tree.layout(3).expect("named lane child should be laid out");

    assert_eq!(output.content_size.width, 60.0);
    assert_eq!(named.location.x, 10.0);
    assert_eq!(named.size.width, 50.0);
}

#[test]
fn named_grid_lanes_resolve_repeated_named_start_and_end_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(20.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(30.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["lane"]),
                ],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 2,
                    },
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 4,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(70.0, 0.0)));

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
    let child = tree.layout(2).expect("named lane child should be laid out");

    assert_eq!(output.size, Size::new(90.0, 0.0));
    assert_eq!(child.location.x, 20.0);
    assert_eq!(child.size.width, 70.0);
}

#[test]
fn grid_lanes_with_rows_template_uses_columns_as_lane_axis_for_intrinsic_width() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3, 4, 5])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_rows: vec![
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                ],
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .style(4, NodeInput::default())
        .measure(2, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(4, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(5, ComputeOutput::from_outer_size(Size::new(73.0, 30.0)));

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

    assert_eq!(output.size, Size::new(145.0, 45.0));
    assert_eq!(
        tree.layout(5)
            .expect("spanning item should be laid out")
            .location
            .x,
        72.0
    );
}

#[test]
fn grid_lanes_lane_measurement_honors_min_content_width() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_rows: vec![TrackComponent::AUTO, TrackComponent::AUTO],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::MAX_CONTENT, PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .available(Size::new(
                    Available::definite(54.0),
                    Available::definite(15.0),
                )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(Some(72.0), None))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .available(Size::new(Available::definite(72.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .available(Size::new(Available::definite(72.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(None, Some(30.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::definite(30.0))),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::definite(30.0))),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(72.0),
                    Available::definite(30.0),
                )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(54.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(54.0),
                    Available::definite(30.0),
                )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(Some(54.0), Some(30.0)))
                .parent(Size::new(Some(54.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(54.0),
                    Available::definite(30.0),
                )),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .available(Size::new(
                    Available::definite(72.0),
                    Available::definite(15.0),
                )),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(Some(72.0), None))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .available(Size::new(Available::definite(72.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .available(Size::new(Available::definite(72.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(None, Some(30.0)))
                .available(Size::new(Available::MAX_CONTENT, Available::definite(30.0))),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .available(Size::new(Available::MAX_CONTENT, Available::definite(30.0))),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(72.0),
                    Available::definite(30.0),
                )),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(Some(72.0), Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(72.0),
                    Available::definite(30.0),
                )),
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

    assert_eq!(output.size, Size::new(72.0, 60.0));
    assert!(tree.inputs(2).iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize
            && input.available() == Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT)
    }));
}

#[test]
fn named_grid_lanes_place_item_between_named_ordinary_grid_lines() {
    let oracle_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
        3,
        vec![
            Vec::<&str>::new(),
            vec!["slot-start"],
            vec![],
            vec!["slot-end"],
        ],
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_axis_placement(
        &oracle_lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "slot-start".to_string(),
                occurrence: 1,
            },
            end: lts::oracle::grid::NamedGridLine::Named {
                name: "slot-end".to_string(),
                occurrence: 1,
            },
        },
        None,
    )
    .unwrap()
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["slot-start"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["slot-end"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "slot-start".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "slot-end".to_string(),
                        index: 1,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

#[test]
fn named_grid_lanes_span_named_implicit_fallback_line() {
    let oracle_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
        1,
        vec![vec!["a"], vec!["a"]],
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_axis_placement(
        &oracle_lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "a".to_string(),
                occurrence: 2,
            },
            end: lts::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_string()),
                count: 2,
            },
        },
        None,
    )
    .unwrap()
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_columns: vec![TrackComponent::px(40.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
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
                        index: 2,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

#[test]
fn named_grid_lanes_subgrid_axis_uses_inherited_line_names() {
    let parent_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
        4,
        vec![
            vec!["a"],
            vec!["b"],
            Vec::<&str>::new(),
            vec!["c"],
            vec!["d"],
        ],
    )
    .unwrap();
    let subgrid = lts::oracle::grid::inherit_named_subgrid_lines(
        &parent_lines,
        lts::oracle::grid::TrackSpan::new(2, 5),
        false,
        vec![Vec::<String>::new(), Vec::new(), Vec::new(), Vec::new()],
        None,
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_subgrid_axis_placement(
        &subgrid.lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "b".to_string(),
                occurrence: 1,
            },
            end: lts::oracle::grid::NamedGridLine::Named {
                name: "c".to_string(),
                occurrence: 1,
            },
        },
        None,
    )
    .unwrap()
    .clamped
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(160.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["b"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["c"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["d"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::GridLanes,
                grid_column: GridPlacement::try_lines(2, 5).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "b".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "c".to_string(),
                        index: 1,
                    },
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("subgridded lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

fn assert_logical_inherited_grid_axis_contexts<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    let parent_columns = [scalar(30.0), scalar(40.0)];
    let parent_rows = [scalar(50.0), scalar(60.0)];
    let parent_gap = LogicalSizeOf::new(scalar(7.0), scalar(11.0));
    let parent_area_size = LogicalSizeOf::new(scalar(77.0), scalar(121.0));
    let parent_named_columns = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let parent_named_rows = named::NamedGridLines::new(GridAxisKind::Row, 2);
    let parent_baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); 2],
        columns: vec![TrackBaselineGroup::default(); 2],
    };

    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let parent_direction = match direction {
                Direction::Ltr => Direction::Rtl,
                Direction::Rtl => Direction::Ltr,
            };
            let parent_style = NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: parent_direction,
                ..NodeInputOf::default()
            };
            let child_style = NodeInputOf {
                display: Display::Grid,
                writing_mode,
                direction,
                grid_template_columns: subgrid_track_of(),
                grid_template_rows: subgrid_track_of(),
                ..NodeInputOf::default()
            };
            let parent_flow_axes =
                crate::geometry::FlowAxes::new(parent_style.writing_mode, parent_style.direction);
            let child_flow_axes =
                crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction);
            let parent_physical_size = parent_flow_axes.physical_size(parent_area_size);
            let item = SubgridItemReport {
                node: 1,
                column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
                row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
            };
            let column_mapping = item.column.mapping;
            let row_mapping = item.row.mapping;
            assert_eq!(column_mapping.queried_axis, GridAxisKind::Column);
            assert_eq!(column_mapping.child_axis, GridAxisKind::Column);
            assert_eq!(row_mapping.queried_axis, GridAxisKind::Row);
            assert_eq!(row_mapping.child_axis, GridAxisKind::Row);

            let mut context = subgrid_child_parent_context(SubgridChildParentContextInput {
                item,
                child_style: &child_style,
                area: GridArea {
                    row: 0,
                    column: 0,
                    row_end: 2,
                    column_end: 2,
                    size: parent_area_size,
                },
                content_box_size: parent_physical_size,
                columns: &parent_columns,
                rows: &parent_rows,
                gap: parent_gap,
                parent_named_columns: &parent_named_columns,
                parent_named_rows: &parent_named_rows,
                parent_area_facts: None,
                parent_baseline_groups: &parent_baseline_groups,
                margin: Edges::all(Some(S::ZERO)),
                border: Edges::ZERO,
                padding: Edges::ZERO,
            })
            .expect("subgrid inherited context must resolve");
            let columns = context
                .columns
                .as_mut()
                .expect("columns subgrid axis must inherit");
            columns.offset = scalar(13.0);
            let rows = context
                .rows
                .as_mut()
                .expect("rows subgrid axis must inherit");
            rows.offset = scalar(17.0);

            let columns = context
                .columns
                .as_ref()
                .expect("columns subgrid axis must remain inherited");
            let rows = context
                .rows
                .as_ref()
                .expect("rows subgrid axis must remain inherited");
            let inherited_logical_size = LogicalSizeOf::new(
                track_sum(&columns.tracks, columns.gap),
                track_sum(&rows.tracks, rows.gap),
            );
            assert_eq!(
                child_flow_axes.physical_size(inherited_logical_size),
                parent_physical_size,
                "{writing_mode:?} {direction:?} must retain parent physical extent through child logical axes"
            );
            assert_eq!(
                grid_axis_logical_offsets(
                    &columns.tracks,
                    Some(columns.offset),
                    S::ZERO,
                    GridAlignment {
                        start: S::ZERO,
                        gap: columns.gap,
                    },
                )[0],
                scalar(13.0),
                "{writing_mode:?} {direction:?} must retain the inherited inline offset logically"
            );
            assert_eq!(
                grid_axis_logical_offsets(
                    &rows.tracks,
                    Some(rows.offset),
                    S::ZERO,
                    GridAlignment {
                        start: S::ZERO,
                        gap: rows.gap,
                    },
                )[0],
                scalar(17.0),
                "{writing_mode:?} {direction:?} must retain the inherited block offset logically"
            );

            let mut tree = OracleTreeOf::<S>::new()
                .children(1, [])
                .style(1, child_style);
            let output = compute_grid_with_context(
                &mut tree,
                1,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    parent_physical_size.map(Some),
                    crate::ContainingLayoutContext::new(
                        parent_flow_axes,
                        crate::ParentFormattingContext::NoParent,
                    ),
                    parent_physical_size.map(AvailableOf::definite),
                ),
                context,
            )
            .expect("inherited grid sizing must complete");

            assert_eq!(
                output.size, parent_physical_size,
                "{writing_mode:?} {direction:?} must project inherited track totals to physical output"
            );
            assert_eq!(
                output.content_size, parent_physical_size,
                "{writing_mode:?} {direction:?} must retain inherited content geometry physically"
            );
        }
    }
}

#[test]
fn logical_inherited_grid_axis_contexts_f32() {
    assert_logical_inherited_grid_axis_contexts::<f32>();
}

#[test]
fn logical_inherited_grid_axis_contexts_f64() {
    assert_logical_inherited_grid_axis_contexts::<f64>();
}

#[test]
fn subgrid_template_resolves_to_empty_explicit_tracks_and_grows_implicit_tracks() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4, 5, 6]);
    for child in 2..=6 {
        tree.insert_children(child, vec![]);
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::Subgrid(crate::SubgridTrack {
                    name_components: vec![crate::SubgridLineNameComponent::LineNames(vec![
                        "main".to_string(),
                    ])],
                }),
                TrackComponent::Repeat(
                    crate::TrackRepetition::auto_fit(vec![crate::TrackSizing::px(10.0)])
                        .expect("valid track repetition"),
                ),
            ],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            grid_auto_columns: vec![TrackComponent::px(10.0)],
            grid_auto_rows: vec![TrackComponent::px(10.0)],
            ..NodeInput::default()
        },
    );
    for child in 2..=6 {
        tree.insert_style(child, NodeInput::default());
        tree.insert_measure(child, ComputeOutput::from_outer_size(Size::new(10.0, 10.0)));
    }

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

    assert_eq!(output.content_size, Size::new(10.0, 50.0));
    assert_eq!(
        tree.layout(6).expect("node layout is staged").location.y,
        40.0
    );
}

#[test]
fn row_subgrid_intrinsic_width_uses_inherited_rows_for_column_auto_flow() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
                grid_auto_flow: GridAutoFlow::Column,
                grid_template_rows: vec![empty_subgrid_track()],
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
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

    assert_eq!(output.content_size, Size::new(100.0, 100.0));
    assert_eq!(tree.layout(2).unwrap().size, Size::new(100.0, 100.0));
    assert_eq!(tree.layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.layout(4).unwrap().location, Point::new(0.0, 50.0));
}

#[test]
fn row_subgrid_constrained_sizing_keeps_fixed_descendants_when_sibling_uses_percent() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_row: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(30.0)),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::percent(0.5)),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
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

    assert_eq!(output.content_size, Size::new(100.0, 30.0));
    assert_eq!(tree.layout(2).unwrap().size.height, 30.0);
}

fn row_subgrid_auto_track_sizing_tree(
    columns: Vec<TrackComponent>,
    subgrid_column: GridPlacement,
) -> OracleTree {
    OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: columns,
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: subgrid_column,
                margin: Edges {
                    top: LengthAuto::px(7.0),
                    right: LengthAuto::px(11.0),
                    bottom: LengthAuto::px(3.0),
                    left: LengthAuto::px(5.0),
                },
                padding: Edges {
                    top: Length::px(3.0),
                    right: Length::px(5.0),
                    bottom: Length::px(7.0),
                    left: Length::px(11.0),
                },
                border: Edges {
                    top: Length::px(5.0),
                    right: Length::px(7.0),
                    bottom: Length::px(11.0),
                    left: Length::px(3.0),
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::default())
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(58.0, 84.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::NONE)
                .available(Size::new(Available::Definite(58.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(58.0, 84.0)))
                .run_mode(RunMode::PerformLayout)
                .available(Size::new(
                    Available::Definite(58.0),
                    Available::Definite(84.0),
                )),
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(58.0, 116.0)))
}

#[test]
fn row_subgrid_auto_track_sizing_fixed_then_auto_uses_descendant_contribution_once() {
    let mut tree = row_subgrid_auto_track_sizing_tree(
        vec![TrackComponent::px(100.0), TrackComponent::AUTO],
        GridPlacement::try_line(1).expect("valid grid line"),
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

    assert_eq!(output.size.height, 120.0);
    assert_eq!(tree.layout(2).unwrap().size.height, 110.0);
}

#[test]
fn row_subgrid_auto_track_sizing_auto_then_fixed_uses_descendant_contribution_once() {
    let mut tree = row_subgrid_auto_track_sizing_tree(
        vec![TrackComponent::AUTO, TrackComponent::px(100.0)],
        GridPlacement::try_line(2).expect("valid grid line"),
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

    assert_eq!(output.size.height, 120.0);
    assert_eq!(tree.layout(2).unwrap().size.height, 110.0);
}

#[test]
fn row_subgrid_intrinsic_width_accumulates_standalone_percent_columns() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [5])
        .children(4, [6])
        .children(5, [])
        .children(6, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_rows: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
                grid_template_columns: vec![
                    TrackComponent::percent(0.2),
                    TrackComponent::percent(0.3),
                ],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            5,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::default())
        .style(4, NodeInput::default());

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

    assert_eq!(output.content_size.width, 200.0);
    assert_eq!(tree.layout(2).unwrap().size.width, 100.0);
}

#[test]
fn vertical_intrinsic_subgrid_final_sizing_keeps_definite_physical_height() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::AUTO, PreferredSize::px(100.0)),
                grid_template_columns: vec![
                    TrackComponent::percent(0.2),
                    TrackComponent::percent(0.3),
                ],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, ComputeOutput::from_outer_size(Size::new(100.0, 10.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(100.0, 10.0)));

    let parent_context = GridParentContext {
        columns: None,
        rows: Some(InheritedGridAxis {
            offset: 0.0,
            gap: 0.0,
            tracks: vec![100.0],
            geometry: UsedGridAxisGeometryOf::new(vec![100.0], vec![false], 0.0),
            named_lines: named::NamedGridLines::new(GridAxisKind::Row, 1),
            area_facts: None,
            template_area_expanded: false,
            major_baselines: vec![None],
            minor_baselines: vec![None],
            owner_baseline_targets: None,
            parent_start: 0,
            parent_end: 1,
            reversed: false,
        }),
    };

    let output = compute_grid_with_context(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(None, Some(100.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::MAX_CONTENT, Available::Definite(100.0)),
        ),
        parent_context,
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 100.0));
}

#[test]
fn subgrid_line_names_merge_local_names_at_corresponding_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
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
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack::new(
                    vec![
                        vec!["local-start".to_string()],
                        vec![],
                        vec!["middle".to_string()],
                        vec!["local-end".to_string()],
                    ],
                ))],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "local-start".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "middle".to_string(),
                        index: 1,
                    },
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("local-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_clip_parent_area_generated_names_to_span() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(160.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            None,
                            Some("main".to_string()),
                            Some("main".to_string()),
                            None,
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(2, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("area-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_nested_subgrid_inherits_area_generated_names() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4, 5, 6, 7])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(160.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            None,
                            Some("main".to_string()),
                            Some("main".to_string()),
                            None,
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(2, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(5, NodeInput::default())
        .style(6, NodeInput::default())
        .style(7, NodeInput::default());

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(4)
        .expect("nested area-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_named_placement_beyond_span_clamps_to_edge_track() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
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
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 2).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(3)),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("clamped child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 40.0);
}

#[test]
fn grid_subgrid_declaration_without_parent_grid_keeps_ordinary_grid_fallback() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![empty_subgrid_track()],
            grid_template_rows: vec![TrackComponent::AUTO],
            grid_auto_columns: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

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

    assert_eq!(output.content_size, Size::new(20.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn nested_inherited_grid_axis_preserves_owner_targets_without_envelope_rewrite_through_production_context_builder()
 {
    let owner_member =
        inherited_placement_member(91, GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let owner_group = AncestorBaselineGroup::reduce(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        4,
        [owner_member],
    );
    let owner_direct_member =
        inherited_placement_member(92, GridAxisKind::Row, AncestorBaselineRole::First, 1, 10.0);
    assert_eq!(
        owner_group.placement_offset(owner_direct_member, 100.0, 20.0, 0.0),
        Some(7.0),
        "the owner-direct item consumes the immutable owner target",
    );
    let ancestor_groups = final_ancestor_baseline_groups_for_transport_test(
        owner_group.clone(),
        AncestorBaselineGroup::reduce(
            1_u32,
            GridAxisKind::Column,
            PhysicalAxis::Horizontal,
            1,
            Vec::<AncestorBaselineMember<u32>>::new(),
        ),
    );
    let parent_style = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        grid_template_rows: subgrid_track(),
        gap: Size::new(Length::ZERO, Length::px(26.0)),
        ..NodeInput::default()
    };
    let context = subgrid_child_parent_context_from_ancestor_groups(
        SubgridChildParentContextInput {
            item: SubgridItemReport {
                node: 7_u32,
                column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
                row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
            },
            child_style: &child_style,
            area: GridArea {
                row: 0,
                column: 0,
                row_end: 4,
                column_end: 1,
                size: LogicalSizeOf::new(25.0, 178.0),
            },
            content_box_size: Size::new(25.0, 178.0),
            columns: &[25.0],
            rows: &[25.0; 4],
            gap: LogicalSizeOf::new(0.0, 10.0),
            parent_named_columns: &NamedGridLines::new(GridAxisKind::Column, 1),
            parent_named_rows: &NamedGridLines::new(GridAxisKind::Row, 4),
            parent_area_facts: None,
            parent_baseline_groups: &GridBaselineGroups {
                rows: vec![TrackBaselineGroup::default(); 4],
                columns: vec![TrackBaselineGroup::default()],
            },
            margin: Edges::all(Some(0.0)),
            border: Edges::ZERO,
            padding: Edges::ZERO,
        },
        &ancestor_groups,
        1_u32,
    )
    .unwrap();

    let row = context.rows.as_ref().unwrap();
    assert_eq!(
        row.major_baselines[1].unwrap().coordinate(),
        9.0,
        "the child-internal envelope retains its reviewed geometry",
    );
    let transported = row.owner_baseline_targets.as_ref().unwrap();
    assert_eq!(
        transported
            .group
            .target_record(AncestorBaselineRole::First, 1)
            .unwrap()
            .finite_owner_logical_target(),
        17.0,
        "production context transport must not rewrite the owner target from the envelope",
    );
    let placement = InheritedCurrentGridBaselinePlacement::try_derive(
        &transported.group,
        InheritedCurrentGridBaselinePlacementInput {
            axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            mapping: transported.mapping.clone(),
            direct_witness: inherited_placement_witness(
                GridAxisKind::Row,
                AncestorBaselineRole::First,
                1,
            ),
            current_grid: 7,
            item: 11,
        },
    )
    .unwrap();
    assert_eq!(placement.translated_target(), 25.0);

    let current_rows = row.tracks.clone();
    let current_groups = final_ancestor_baseline_groups_with_parent_context_for_transport_test(
        final_ancestor_baseline_groups_for_transport_test(
            inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0),
            AncestorBaselineGroup::reduce(
                7_u32,
                GridAxisKind::Column,
                PhysicalAxis::Horizontal,
                1,
                Vec::<AncestorBaselineMember<u32>>::new(),
            ),
        ),
        &context,
    );
    let nested_style = NodeInput {
        display: Display::Grid,
        grid_template_rows: subgrid_track(),
        gap: Size::new(Length::ZERO, Length::px(34.0)),
        ..NodeInput::default()
    };
    let nested_context = subgrid_child_parent_context_from_ancestor_groups(
        SubgridChildParentContextInput {
            item: SubgridItemReport {
                node: 11_u32,
                column: subgrid_axis_report(&child_style, &nested_style, GridAxisKind::Column),
                row: subgrid_axis_report(&child_style, &nested_style, GridAxisKind::Row),
            },
            child_style: &nested_style,
            area: GridArea {
                row: 0,
                column: 0,
                row_end: 4,
                column_end: 1,
                size: LogicalSizeOf::new(25.0, 202.0),
            },
            content_box_size: Size::new(25.0, 202.0),
            columns: &[25.0],
            rows: &current_rows,
            gap: LogicalSizeOf::new(0.0, 26.0),
            parent_named_columns: &NamedGridLines::new(GridAxisKind::Column, 1),
            parent_named_rows: &NamedGridLines::new(GridAxisKind::Row, 4),
            parent_area_facts: None,
            parent_baseline_groups: &GridBaselineGroups {
                rows: vec![TrackBaselineGroup::default(); 4],
                columns: vec![TrackBaselineGroup::default()],
            },
            margin: Edges::all(Some(0.0)),
            border: Edges::ZERO,
            padding: Edges::ZERO,
        },
        &current_groups,
        7_u32,
    )
    .unwrap();
    let nested_row = nested_context.rows.as_ref().unwrap();
    assert_eq!(nested_row.major_baselines[1].unwrap().coordinate(), 5.0);
    let nested_transport = nested_row.owner_baseline_targets.as_ref().unwrap();
    assert_eq!(
        nested_transport
            .group
            .target_record(AncestorBaselineRole::First, 1)
            .unwrap()
            .finite_owner_logical_target(),
        17.0,
    );
    let nested_placement = InheritedCurrentGridBaselinePlacement::try_derive(
        &nested_transport.group,
        InheritedCurrentGridBaselinePlacementInput {
            axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            mapping: nested_transport.mapping.clone(),
            direct_witness: CurrentGridDirectWitness::new(
                11,
                12,
                GridAxisKind::Row,
                GridTrackSpan::new(1, 2),
                AncestorBaselineRole::First,
            ),
            current_grid: 11,
            item: 12,
        },
    )
    .unwrap();
    assert_eq!(nested_placement.translated_target(), 21.0);
    assert_eq!(
        owner_group
            .target_record(AncestorBaselineRole::First, 1)
            .unwrap()
            .finite_owner_logical_target(),
        17.0,
    );
}

#[test]
fn fri06_c12_t08_refreshed_subgrid_offsets_remain_logical_until_projection() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::px(100.0)],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                direction: Direction::Rtl,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    left: LengthAuto::px(24.0),
                    right: LengthAuto::px(10.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(3, NodeInput::default())
        .measure(3, baseline_measure(20.0, 20.0, Some(14.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(
        tree.layout(2).expect("subgrid is laid out").location.x,
        24.0
    );
}

#[test]
fn fri06_c12_t08_smaller_inherited_gap_applies_one_signed_track_transform() {
    assert_eq!(
        fri06_c12_t08_inherited_baseline_gap_position(20.0, 10.0),
        82.0
    );
}

fn fri06_c12_t08_nested_inherited_row_baseline_delta(
    parent_gap: f32,
    child_gap: f32,
    reversed: bool,
    alignment: AlignItems,
) -> f32 {
    let child_writing_mode = if reversed {
        WritingMode::VerticalLr
    } else {
        WritingMode::VerticalRl
    };
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [5])
        .children(5, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(80.0)),
                grid_template_rows: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::px(parent_gap), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                writing_mode: child_writing_mode,
                grid_row: GridPlacement::try_lines(1, 3).expect("valid inherited row span"),
                grid_column: GridPlacement::try_lines(1, 3).expect("valid column span"),
                grid_template_rows: vec![empty_subgrid_track()],
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::px(child_gap), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                writing_mode: child_writing_mode,
                grid_row: GridPlacement::try_line(2).expect("valid direct row"),
                grid_column: GridPlacement::try_line(1).expect("valid direct column"),
                align_self: Some(alignment),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Grid,
                writing_mode: child_writing_mode,
                grid_row: GridPlacement::try_lines(1, 3).expect("valid nested row span"),
                grid_column: GridPlacement::try_line(2).expect("valid nested column"),
                grid_template_rows: vec![empty_subgrid_track()],
                grid_template_columns: vec![TrackComponent::px(40.0)],
                gap: Size::new(Length::px(child_gap), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                writing_mode: child_writing_mode,
                grid_row: GridPlacement::try_line(2).expect("valid flattened row"),
                align_self: Some(alignment),
                ..NodeInput::default()
            },
        );
    let (direct, nested) = match alignment {
        AlignItems::Baseline => (
            vertical_baseline_measure(30.0, 20.0, Some(20.0), None),
            vertical_baseline_measure(30.0, 20.0, Some(5.0), None),
        ),
        AlignItems::LastBaseline => (
            vertical_baseline_measure(30.0, 20.0, None, Some(5.0)),
            vertical_baseline_measure(30.0, 20.0, None, Some(20.0)),
        ),
        _ => unreachable!("the inherited-row control uses baseline alignment"),
    };
    tree = tree.measure(3, direct).measure(5, nested);

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(140.0), Available::Definite(80.0)),
    )
    .expect("nested inherited-row layout computes");
    round_layout(&mut tree, 1).expect("nested inherited-row layout rounds");

    let direct_layout = tree.final_layout(3).expect("direct member is laid out");
    let nested_grid = tree.final_layout(4).expect("nested grid is laid out");
    let nested_layout = tree.final_layout(5).expect("flattened member is laid out");
    let (direct_baseline, nested_baseline) = match alignment {
        AlignItems::Baseline => (20.0, 5.0),
        AlignItems::LastBaseline => (25.0, 10.0),
        _ => unreachable!("the inherited-row control uses baseline alignment"),
    };
    direct_layout.location.x + direct_baseline
        - nested_grid.location.x
        - nested_layout.location.x
        - nested_baseline
}

#[test]
fn fri06_c12_t08_inherited_row_gap_adjustment_stays_in_member_and_view_mapping() {
    let cases = [
        (10.0, 20.0, false, AlignItems::Baseline),
        (10.0, 20.0, false, AlignItems::LastBaseline),
        (20.0, 10.0, false, AlignItems::Baseline),
        (20.0, 10.0, false, AlignItems::LastBaseline),
        (10.0, 20.0, true, AlignItems::Baseline),
        (10.0, 20.0, true, AlignItems::LastBaseline),
        (20.0, 10.0, true, AlignItems::Baseline),
        (20.0, 10.0, true, AlignItems::LastBaseline),
    ];
    let deltas = cases.map(|(parent_gap, child_gap, reversed, alignment)| {
        fri06_c12_t08_nested_inherited_row_baseline_delta(
            parent_gap, child_gap, reversed, alignment,
        )
    });
    assert_eq!(deltas, [0.0; 8]);
}

#[test]
fn fri06_c12_t08_inherited_gap_transform_uses_local_first_and_last_edges_after_reversal() {
    let parent_major = [
        Some(tagged_baseline(PhysicalAxis::Vertical, 14.0)),
        Some(tagged_baseline(PhysicalAxis::Vertical, 30.0)),
    ];
    let parent_minor = [
        Some(tagged_baseline(PhysicalAxis::Vertical, 6.0)),
        Some(tagged_baseline(PhysicalAxis::Vertical, 12.0)),
    ];

    let forward = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &parent_major,
        parent_minor: &parent_minor,
        physical_axis: PhysicalAxis::Vertical,
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 2.0,
        end_mbp: 3.0,
        parent_gap: 10.0,
        subgrid_gap: 20.0,
    })
    .unwrap();
    assert_eq!(
        forward.final_major,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 12.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 25.0)),
        ],
        "first baselines rebase only across their local start edge",
    );
    assert_eq!(
        forward.final_minor,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 1.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 9.0)),
        ],
        "last baselines rebase only across their local end edge",
    );

    let reversed = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &parent_major,
        parent_minor: &parent_minor,
        physical_axis: PhysicalAxis::Vertical,
        parent_span: GridTrackSpan::new(1, 3),
        reversed: true,
        start_mbp: 2.0,
        end_mbp: 3.0,
        parent_gap: 10.0,
        subgrid_gap: 20.0,
    })
    .unwrap();
    assert_eq!(
        reversed.final_major,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 28.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 9.0)),
        ],
    );
    assert_eq!(
        reversed.final_minor,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 7.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 3.0)),
        ],
    );
}

#[test]
fn nested_subgrid_percent_columns_rerun_rows_after_inherited_width_and_margin() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    left: LengthAuto::px(10.0),
                    right: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::percent(1.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    right: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(4, NodeInput::default())
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(100.0, 64.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(None, None))
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::MAX_CONTENT,
                )),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 96.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(Some(80.0), None))
                .available(Size::new(Available::Definite(80.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 96.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, None))
                .available(Size::new(
                    Available::Definite(80.0),
                    Available::Definite(96.0),
                )),
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(100.0, 64.0)));

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
    tree.set_unrounded(
        1,
        NodeOutput {
            size: output.size,
            content_size: output.content_size,
            ..NodeOutput::new()
        },
    );
    round_layout(&mut tree, 1).unwrap();

    assert_eq!(output.size.height, 96.0);
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(85.0, 96.0));
    assert_eq!(tree.final_layout(3).unwrap().size, Size::new(80.0, 96.0));
}

#[test]
fn row_subgrid_percent_column_leaf_uses_spanned_inline_size_for_row_contribution() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![
                    TrackComponent::percent(0.5),
                    TrackComponent::percent(0.5),
                ],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(50.0, 90.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(Some(50.0), None))
                .available(Size::new(Available::Definite(50.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(50.0, 90.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, None))
                .available(Size::new(
                    Available::Definite(50.0),
                    Available::Definite(90.0),
                )),
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(100.0, 40.0)));

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
    tree.set_unrounded(
        1,
        NodeOutput {
            size: output.size,
            content_size: output.content_size,
            ..NodeOutput::new()
        },
    );
    round_layout(&mut tree, 1).unwrap();

    assert_eq!(output.size, Size::new(100.0, 90.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(100.0, 90.0));
    assert!(
        tree.inputs(3)
            .iter()
            .any(|input| input.run_mode() == RunMode::ComputeSize
                && input.known().width == Some(50.0)
                && input.available().width == Available::Definite(50.0)),
        "row contribution should measure the leaf against its 50px column span"
    );
}

#[test]
fn orthogonal_nested_subgrid_width_includes_full_horizontal_leaf_contribution() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 6])
        .children(3, [4, 5])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                gap: Size::new(Length::px(20.0), Length::px(20.0)),
                border: Edges::all(Length::px(3.0)),
                grid_template_columns: vec![TrackComponent::px(100.0), TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::px(100.0), TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                gap: Size::new(Length::px(100.0), Length::px(100.0)),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_span(2).expect("valid grid span"),
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                gap: Size::new(Length::px(100.0), Length::px(100.0)),
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(4, NodeInput::DEFAULT)
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(24.0, 24.0)))
        .measure(5, ComputeOutput::from_outer_size(Size::new(24.0, 24.0)))
        .measure(6, ComputeOutput::from_outer_size(Size::new(72.0, 24.0)));

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

    assert_eq!(output.size.width, 238.0);
}

#[test]
fn grid_auto_size_ignores_ineligible_row_subgrid_when_resolving_percent_columns() {
    let mut tree = OracleTree::new()
        .measure(2, ComputeOutput::from_outer_size(Size::new(100.0, 100.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(100.0, 100.0)));
    tree.insert_children(1, vec![2, 3]);
    for node in 2..=3 {
        tree.insert_children(node, vec![]);
        tree.insert_style(node, NodeInput::default());
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::percent(0.5), TrackComponent::percent(0.5)],
            grid_template_rows: vec![empty_subgrid_track()],
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

    assert_eq!(output.size.width, 100.0);
}

#[test]
fn subgrid_intrinsic_row_sizing_uses_subgrid_content_not_parent_height() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4, 5, 6, 7])
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::AUTO; 4],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                grid_row: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(25.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            5,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            7,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(75.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 1).unwrap();
    let child = tree.final_layout(2).expect("child grid should be laid out");
    let subgrid = tree.final_layout(3).expect("subgrid should be laid out");

    assert_eq!(child.size, Size::new(100.0, 100.0));
    assert_eq!(subgrid.size, Size::new(100.0, 100.0));
}

#[test]
fn lane_axis_margin_box_measurement_resolves_affine_margins_against_grid_axis() {
    let margin = lp(4.0, 0.10);
    let child_style = NodeInput {
        margin: Edges {
            left: LengthAuto::value(margin),
            right: LengthAuto::px(6.0),
            top: LengthAuto::ZERO,
            bottom: LengthAuto::ZERO,
        },
        ..NodeInput::default()
    };
    let container_style = NodeInput::default();
    let constants = Constants {
        flow_axes: crate::geometry::FlowAxes::new(
            container_style.writing_mode,
            container_style.direction,
        ),
        explicit_definite_content_size: Size::new(Some(200.0), Some(80.0)),
        node_outer_size: Size::new(Some(200.0), Some(80.0)),
        node_inner_size: Size::new(Some(200.0), Some(80.0)),
        node_min_size: Size::NONE,
        node_max_size: Size::NONE,
        available_inner_size: Size::new(Some(200.0), Some(80.0)),
        content_box_inset: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let mut tree = LaneMarginMeasureTree {
        child_style: child_style.clone(),
        child_output: ComputeOutput::from_sizes_and_baselines(
            Size::new(50.0, 12.0),
            Size::new(50.0, 12.0),
            Baselines::NONE,
        ),
        last_input: None,
    };
    let child_projection = grid_item_projection!(&child_style);
    let container_projection = grid_container_projection!(&container_style);

    let measured = measure_lane_axis_margin_box_with_grid_axis(
        &mut tree,
        LaneMarginMeasureTree::CHILD,
        LaneAxisMarginBoxMeasureInput {
            child_style: &child_projection,
            container_style: &container_projection,
            constants: &constants,
            lane_axis: GridAxisKind::Column,
            containing_block: GridLanesItemContainingBlockOf::new(
                constants.flow_axes,
                GridAxisKind::Column,
                200.0,
                LogicalSizeOf::new(Some(200.0), Some(80.0)),
            ),
        },
    );

    assert_eq!(measured, Ok(80.0));
    let input = tree
        .last_input
        .expect("measurement should compute the child");
    assert_eq!(input.known().width, Some(170.0));
    assert_eq!(input.parent().width, Some(200.0));
    assert_eq!(input.available().width, Available::Definite(170.0));
}

struct LaneMarginMeasureTree {
    child_style: NodeInput,
    child_output: ComputeOutput,
    last_input: Option<ComputeInput>,
}

impl LaneMarginMeasureTree {
    const ROOT: usize = 0;
    const CHILD: usize = 1;
}

impl Traverse for LaneMarginMeasureTree {
    type Node = usize;
    type Scalar = Scalar;
    type Children<'a> = std::vec::IntoIter<Self::Node>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        match node {
            Self::ROOT => vec![Self::CHILD].into_iter(),
            _ => Vec::new().into_iter(),
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == Self::ROOT)
    }

    fn child(&self, _node: Self::Node, index: usize) -> Self::Node {
        assert_eq!(index, 0);
        Self::CHILD
    }
}

impl Compute for LaneMarginMeasureTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        assert_eq!(node, Self::CHILD);
        &self.child_style
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {
        unreachable!("lane margin measurement should not write layout output");
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        Ok({
            assert_eq!(node, Self::CHILD);
            self.last_input = Some(input);
            self.child_output
        })
    }
}

#[test]
fn grid_lane_track_base_rejects_positive_invalid_affine_numeric_result() {
    let outcome = lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: Some(2.0),
        gap: 0.0,
        tracks: vec![TrackSizing::new(
            MinTrackSizing::Calculation(SizingCalculation::value(invalid_numeric_lp())),
            MaxTrackSizing::Auto,
        )],
        content_sized_tracks: vec![0],
        items: Vec::new(),
    });

    let error = outcome.expect_err("invalid lane track sizing must not produce output");

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        })
    );
}

#[test]
fn grid_lane_track_base_rejects_signed_invalid_affine_numeric_result() {
    let outcome = lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: Some(f32::MAX),
        gap: 0.0,
        tracks: vec![TrackSizing::new(
            MinTrackSizing::Calculation(SizingCalculation::value(lp(-f32::MAX, -1.0))),
            MaxTrackSizing::Auto,
        )],
        content_sized_tracks: vec![0],
        items: Vec::new(),
    });

    let error = outcome.expect_err("invalid lane track sizing must not produce output");

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::NEG_INFINITY,
        })
    );
}

#[test]
fn grid_lane_track_base_rejects_positive_and_signed_invalid_f64_affine_numeric_results() {
    for (label, absolute, percent, expected) in [
        ("positive", f64::MAX, 1.0_f64, f64::INFINITY),
        ("signed", -f64::MAX, -1.0_f64, f64::NEG_INFINITY),
    ] {
        let outcome = lane_intrinsic_sizing(LaneIntrinsicSizingInputOf::<f64> {
            axis: GridAxisKind::Column,
            available: Some(f64::MAX),
            gap: 0.0,
            tracks: vec![TrackSizingOf::new(
                MinTrackSizingOf::Calculation(SizingCalculationOf::value(
                    LengthPercentageOf::from_coefficients(absolute, percent)
                        .expect("test coefficients are finite"),
                )),
                MaxTrackSizingOf::Auto,
            )],
            content_sized_tracks: vec![0],
            items: Vec::new(),
        });

        let error = outcome.expect_err("invalid lane track sizing must not produce output");

        assert_eq!(error.site(), LayoutErrorSite::Standalone, "{label} site");
        assert_eq!(
            error.operation(),
            LayoutOperation::ValueResolution,
            "{label} operation"
        );
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                value: expected,
            }),
            "{label} numeric detail"
        );
    }
}

#[test]
fn lane_intrinsic_public_inputs_accept_non_default_scalar() {
    let facts = LaneContributionFactsOf::<f64> {
        min_content: 1.25_f64,
        max_content: 2.5_f64,
        min_size: 0.75_f64,
        automatic_minimum_applies: true,
    };
    let item = LaneIntrinsicItemOf::<f64>::indefinite(
        "wide",
        LaneTrackSpanLength::new(2).expect("span should be nonzero"),
        facts,
    );
    let input = LaneIntrinsicSizingInputOf::<f64> {
        axis: GridAxisKind::Column,
        available: Some(10.5_f64),
        gap: 1.5_f64,
        tracks: vec![TrackSizingOf::<f64>::AUTO],
        content_sized_tracks: vec![0],
        items: vec![item],
    };

    assert_eq!(input.gap, 1.5_f64);
    assert_eq!(input.items[0].contribution().max_content, 2.5_f64);

    let placement_input = LanePlacementInputOf::<_, f64> {
        grid_axis_tracks: 1,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 1.5_f64,
        tolerance: GridFlowToleranceOf::Percent(0.25_f64),
        tolerance_basis: 10.5_f64,
        items: Vec::<LaneItemOf<&str, f64>>::new(),
    };

    assert_eq!(
        placement_input.tolerance,
        GridFlowToleranceOf::Percent(0.25_f64)
    );
}

#[test]
fn lane_public_helpers_compute_with_non_default_scalar() {
    let placement = place_lanes(LanePlacementInputOf::<_, f64> {
        grid_axis_tracks: 2,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 0.5,
        tolerance: GridFlowToleranceOf::Normal { font_size: 0.0 },
        tolerance_basis: 0.0,
        items: vec![
            LaneItemOf {
                item: "a",
                grid_axis_span: 1,
                definite_grid_axis_start: None,
                lane_axis_margin_box: 10.25,
            },
            LaneItemOf {
                item: "b",
                grid_axis_span: 1,
                definite_grid_axis_start: None,
                lane_axis_margin_box: 12.5,
            },
        ],
    })
    .expect("f64 lane placement should compute");

    assert_eq!(placement.content_size, 12.5);
    assert_eq!(placement.item_offsets[1].offset, 0.0);

    let intrinsic = lane_intrinsic_sizing(LaneIntrinsicSizingInputOf::<f64> {
        axis: GridAxisKind::Column,
        available: Some(80.0),
        gap: 1.25,
        tracks: vec![TrackSizingOf::<f64>::AUTO],
        content_sized_tracks: vec![0],
        items: vec![
            LaneIntrinsicItemOf::<f64>::definite(
                "definite",
                LaneTrackSpan::new(1, 2),
                LaneContributionFactsOf {
                    min_content: 9.5,
                    max_content: 14.25,
                    min_size: 7.0,
                    automatic_minimum_applies: true,
                },
            )
            .expect("span is valid"),
        ],
    })
    .expect("f64 lane intrinsic sizing should not fail")
    .expect("f64 lane intrinsic sizing should produce a report");

    assert_eq!(intrinsic.final_track_sizes, vec![9.5]);
}

#[test]
fn grid_lanes_compute_result_accepts_non_default_scalar() {
    #[derive(Clone)]
    struct F64GridTree {
        styles: Vec<NodeInputOf<f64>>,
        children: Vec<Vec<usize>>,
        layouts: Vec<NodeOutputOf<f64>>,
    }

    impl Traverse for F64GridTree {
        type Node = usize;
        type Scalar = f64;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[node][index]
        }
    }

    impl Compute for F64GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInputOf<f64> {
            &self.styles[node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<f64>) {
            self.layouts[node] = layout;
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInputOf<f64>,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                let style = &self.styles[node];
                let size = input.known().unwrap_or(Size::new(
                    style
                        .size
                        .width
                        .resolve_simple_with_status(input.parent().width)
                        .expect("affine preferred width is supported")
                        .value
                        .or_else(|| input.available().width.into_option())
                        .unwrap_or(0.0),
                    style
                        .size
                        .height
                        .resolve_simple_with_status(input.parent().height)
                        .expect("affine preferred height is supported")
                        .value
                        .or_else(|| input.available().height.into_option())
                        .unwrap_or(0.0),
                ));
                ComputeOutputOf::from_sizes(size, size)
            })
        }
    }

    let root_style = NodeInputOf::<f64> {
        display: Display::GridLanes,
        size: Size::new(PreferredSizeOf::px(120.0), PreferredSizeOf::px(90.0)),
        grid_template_columns: vec![TrackSizingOf::px(60.0).into()],
        grid_auto_rows: vec![TrackSizingOf::px(40.0).into()],
        ..NodeInputOf::default()
    };
    let child_style = NodeInputOf::<f64> {
        size: Size::new(PreferredSizeOf::px(30.0), PreferredSizeOf::px(20.0)),
        ..NodeInputOf::default()
    };
    let mut tree = F64GridTree {
        styles: vec![root_style, child_style],
        children: vec![vec![1], Vec::new()],
        layouts: vec![NodeOutputOf::new(), NodeOutputOf::new()],
    };

    let computation = compute_grid_with_report(
        &mut tree,
        0,
        ComputeInputOf::for_child(
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
            Size::splat(AvailableOf::MAX_CONTENT),
        ),
    )
    .unwrap();
    let (output, report) = computation.into_parts();

    assert!(report.is_empty());
    assert_eq!(output.size, Size::new(120.0, 90.0));
    assert_eq!(tree.layouts[1].size, Size::new(30.0, 20.0));
}

fn assert_fri08_c01_area_only_pattern_topology<S: LayoutScalar>() {
    let px = |value| TrackComponentOf::Track(TrackSizingOf::px(S::from_f64(value)));
    let style = NodeInputOf::<S> {
        grid_auto_columns: vec![px(40.0), px(20.0)],
        grid_auto_rows: vec![px(5.0), px(7.0)],
        grid_template_areas: GridTemplateAreas {
            rows: vec![
                GridTemplateAreaRow {
                    cells: vec![
                        Some("main".to_string()),
                        Some("main".to_string()),
                        Some("main".to_string()),
                    ],
                },
                GridTemplateAreaRow {
                    cells: vec![
                        Some("main".to_string()),
                        Some("main".to_string()),
                        Some("main".to_string()),
                    ],
                },
                GridTemplateAreaRow {
                    cells: vec![
                        Some("main".to_string()),
                        Some("main".to_string()),
                        Some("main".to_string()),
                    ],
                },
            ],
        },
        ..NodeInputOf::default()
    };

    let topology = fri08_c01_topology_for_style(&style, None, None);

    assert_eq!(topology.explicit_columns, 3);
    assert_eq!(topology.explicit_rows, 3);
    assert_eq!(
        topology.column_tracks,
        vec![
            TrackSizingOf::px(S::from_f64(40.0)),
            TrackSizingOf::px(S::from_f64(20.0)),
            TrackSizingOf::px(S::from_f64(40.0)),
        ]
    );
    assert_eq!(
        topology.row_tracks,
        vec![
            TrackSizingOf::px(S::from_f64(5.0)),
            TrackSizingOf::px(S::from_f64(7.0)),
            TrackSizingOf::px(S::from_f64(5.0)),
        ]
    );
    assert_eq!(
        topology
            .column_origins
            .iter()
            .map(|origin| origin.sizing)
            .collect::<Vec<_>>(),
        vec![
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 0 },
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 1 },
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 0 },
        ]
    );
    assert_eq!(
        topology
            .row_origins
            .iter()
            .map(|origin| origin.sizing)
            .collect::<Vec<_>>(),
        vec![
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 0 },
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 1 },
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 0 },
        ]
    );
    assert_eq!(
        topology.named_columns.named_occurrences("main-start"),
        vec![1]
    );
    assert_eq!(
        topology.named_columns.named_occurrences("main-end"),
        vec![4]
    );
    assert_eq!(topology.named_rows.named_occurrences("main-start"), vec![1]);
    assert_eq!(topology.named_rows.named_occurrences("main-end"), vec![4]);
    assert!(topology.has_complete_origin_evidence());
}

#[test]
fn fri08_c01_topology_area_only_pattern_phase_is_axis_symmetric_in_both_scalar_lanes() {
    assert_fri08_c01_area_only_pattern_topology::<f32>();
    assert_fri08_c01_area_only_pattern_topology::<f64>();
}

#[test]
fn fri08_c01_topology_inherited_and_local_names_collide_as_one_membership() {
    let parent = named_parent_lines(1, &[&["a"], &[]]);
    let local = local_subgrid_entries(&[&["a"], &[]]);

    let lines = named::inherit_subgrid_named_lines(&parent, 1, 2, false, &local, None)
        .expect("valid inherited named span");

    assert_eq!(lines.named_occurrences("a"), vec![1]);
    assert_eq!(
        lines
            .entries_on_line(1)
            .iter()
            .map(|entry| entry.origin)
            .collect::<Vec<_>>(),
        vec![
            named::LineNameOrigin::Inherited,
            named::LineNameOrigin::LocalSubgrid,
        ]
    );
}

#[test]
fn named_lines_return_empty_local_map_for_subgrid() {
    let lines =
        named::named_lines_from_track_components(GridAxisKind::Row, &subgrid_track(), 2).unwrap();

    assert_eq!(lines.axis, GridAxisKind::Row);
    assert_eq!(lines.explicit_track_count, 2);
    assert!(lines.named_occurrences("anything").is_empty());
}

#[test]
fn subgrid_axis_placement_reports_one_authored_fallback_once() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let (placement, absolute, report) = resolve_grid_item_axis_placements_with_report(
        &lines,
        &RawGridPlacement::new(RawGridLine::Line(0), RawGridLine::Auto),
        GridPlacement::AUTO,
        true,
    );

    assert_eq!(placement, GridPlacement::AUTO);
    assert_eq!(absolute, GridPlacement::AUTO);
    assert_eq!(
        report
            .errors()
            .iter()
            .filter(|error| **error == NamedGridErrorReport::ZeroLine)
            .count(),
        1
    );
}

#[test]
fn subgrid_line_names_expand_auto_fill_and_fixed_slots() {
    let names = named::expand_subgrid_local_line_names(
        GridAxisKind::Column,
        4,
        &[
            SubgridLineNameComponent::LineNames(vec!["start".to_string()]),
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                line_name_sets: vec![vec!["fill".to_string()]],
            },
            SubgridLineNameComponent::LineNames(vec!["end".to_string()]),
        ],
    )
    .unwrap();

    assert_eq!(names.len(), 5);
    assert_eq!(
        local_line_names(&names),
        vec![
            vec!["start"],
            vec!["fill"],
            vec!["fill"],
            vec!["fill"],
            vec!["end"],
        ]
    );
}

#[test]
fn subgrid_line_names_inherit_parent_explicit_and_local_names() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let local = local_subgrid_entries(&[&["local-start"], &[], &["middle"], &["local-end"]]);

    let lines = named::inherit_subgrid_named_lines(&parent, 2, 5, false, &local, None).unwrap();

    assert_eq!(
        entry_names(lines.entries_on_line(1)),
        vec!["b", "local-start"]
    );
    assert_eq!(entry_names(lines.entries_on_line(3)), vec!["c", "middle"]);
    assert_eq!(
        entry_names(lines.entries_on_line(4)),
        vec!["d", "local-end"]
    );
    assert_eq!(
        lines.entries_on_line(1)[1].origin,
        named::LineNameOrigin::LocalSubgrid
    );
}

#[test]
fn subgrid_line_names_reinherit_local_parent_names() {
    let parent = named_parent_lines(2, &[&["outer"], &[], &["outer-end"]]);
    let outer_local = local_subgrid_entries(&[&["local-start"], &[], &["local-end"]]);
    let outer =
        named::inherit_subgrid_named_lines(&parent, 1, 3, false, &outer_local, None).unwrap();
    let nested_local = local_subgrid_entries(&[&[], &[], &[]]);

    let nested =
        named::inherit_subgrid_named_lines(&outer, 1, 3, false, &nested_local, None).unwrap();

    assert_eq!(
        entry_names(nested.entries_on_line(1)),
        vec!["outer", "local-start"]
    );
    assert_eq!(
        entry_names(nested.entries_on_line(3)),
        vec!["outer-end", "local-end"]
    );
}

#[test]
fn subgrid_line_names_reverse_parent_line_order() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let local = local_subgrid_entries(&[&[], &[], &[], &[]]);

    let lines = named::inherit_subgrid_named_lines(&parent, 2, 5, true, &local, None).unwrap();

    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["d"]);
    assert_eq!(entry_names(lines.entries_on_line(2)), vec!["c"]);
    assert_eq!(entry_names(lines.entries_on_line(4)), vec!["b"]);
}

#[test]
fn subgrid_intrinsic_parent_context_uses_actual_span_and_reversal() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let report = SubgridAxisReport {
        mapping: GridAxisMappingReport {
            queried_axis: GridAxisKind::Column,
            parent_axis: GridAxisKind::Column,
            child_axis: GridAxisKind::Column,
            reversed: true,
        },
        eligibility: SubgridEligibility {
            eligible: true,
            reason: None,
        },
    };

    let axis: InheritedGridAxis<Scalar> = intrinsic_subgrid_axis_parent_context(
        report,
        GridArea {
            row: 0,
            column: 1,
            row_end: 1,
            column_end: 4,
            size: LogicalSizeOf::new(Scalar::ZERO, Scalar::ZERO),
        },
        Size::<Scalar>::ZERO,
        (None, None),
        &parent,
        &parent,
        None,
    )
    .unwrap();
    let local = local_subgrid_entries(&[&[], &[], &[], &[]]);
    let lines = named::inherit_subgrid_named_lines(
        &axis.named_lines,
        axis.parent_start + 1,
        axis.parent_end + 1,
        axis.reversed,
        &local,
        axis.area_facts.as_ref(),
    )
    .unwrap();

    assert_eq!(axis.parent_start, 1);
    assert_eq!(axis.parent_end, 4);
    assert!(axis.reversed);
    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["d"]);
    assert_eq!(entry_names(lines.entries_on_line(4)), vec!["b"]);
}

#[test]
fn subgrid_line_names_recompute_area_generated_names_clipped_to_span() {
    let areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![
                Some("a".to_string()),
                Some("a".to_string()),
                Some("a".to_string()),
                Some("a".to_string()),
            ],
        }],
    };
    let parent = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &areas,
    )
    .unwrap();
    let local = local_subgrid_entries(&[&[], &[], &[]]);

    let lines =
        named::inherit_subgrid_named_lines(&parent, 2, 4, false, &local, Some(&parent.area_facts))
            .unwrap();

    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["a-start"]);
    assert_eq!(entry_names(lines.entries_on_line(3)), vec!["a-end"]);
}

#[test]
fn subgrid_area_facts_preserve_reversed_orientation_and_axis_validity() {
    let areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![
                None,
                Some("main".to_string()),
                Some("main".to_string()),
                None,
            ],
        }],
    };
    let parent_lines = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &areas,
    )
    .unwrap();
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            parent_lines.clone(),
            Some(parent_lines.area_facts.clone()),
            1,
            3,
            true,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &grid_container_projection!(&NodeInput {
            grid_template_columns: subgrid_track(),
            ..NodeInput::DEFAULT
        }),
        2,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.named_occurrences("main-start"), vec![3]);
    assert_eq!(context.columns.named_occurrences("main-end"), vec![1]);
    assert!(facts.columns_valid);
    assert!(!facts.rows_valid);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 3);
    assert_eq!(rectangle.column_start_name, 3);
    assert_eq!(rectangle.column_end_name, 1);
}

#[test]
fn subgrid_local_area_facts_clamp_to_inherited_span() {
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4)
                .unwrap(),
            None,
            0,
            2,
            false,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &grid_container_projection!(&NodeInput {
            grid_template_columns: subgrid_track(),
            grid_template_areas: crate::GridTemplateAreas {
                rows: vec![crate::GridTemplateAreaRow {
                    cells: vec![
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                    ],
                }],
            },
            ..NodeInput::DEFAULT
        }),
        2,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.explicit_track_count, 2);
    assert_eq!(context.columns.named_occurrences("wide-start"), vec![1]);
    assert_eq!(context.columns.named_occurrences("wide-end"), vec![3]);
    assert_eq!(facts.column_count, 2);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 3);
}

#[test]
fn subgrid_duplicate_area_facts_merge_with_parent_clipped_boundaries() {
    let parent_areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![Some("same".to_string()), None, None, None],
        }],
    };
    let parent_lines = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &parent_areas,
    )
    .unwrap();
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            parent_lines.clone(),
            Some(parent_lines.area_facts.clone()),
            0,
            3,
            false,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &grid_container_projection!(&NodeInput {
            grid_template_columns: subgrid_track(),
            grid_template_areas: crate::GridTemplateAreas {
                rows: vec![crate::GridTemplateAreaRow {
                    cells: vec![None, Some("same".to_string()), None],
                }],
            },
            ..NodeInput::DEFAULT
        }),
        3,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.named_occurrences("same-start"), vec![1]);
    assert_eq!(context.columns.named_occurrences("same-end"), vec![2]);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 2);
}

#[test]
fn subgrid_named_placement_clamps_beyond_explicit_span() {
    let lines = named_parent_lines(2, &[&["a"], &[], &["a"]]);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: -3,
            },
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 4,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(1, 3).expect("valid grid lines")
    );
}

#[test]
fn subgrid_named_placement_resolves_wpt_line_names_before_clamping_to_span() {
    let parent = named_parent_lines(6, &[&["a"], &[], &[], &[], &["b"], &[], &["a", "b"]]);
    let local = local_subgrid_entries(&[&["x"], &["b"], &[], &[], &["b"]]);
    let lines = named::inherit_subgrid_named_lines(&parent, 2, 6, false, &local, None).unwrap();

    assert_eq!(lines.named_occurrences("b"), vec![2, 4, 5]);

    let cases = [
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(4, 5).expect("valid grid lines"),
        ),
    ];

    for (raw, expected) in cases {
        let placement = named::resolve_subgrid_placement(&lines, &raw, None).unwrap();
        assert_eq!(placement, expected, "raw placement {raw:?}");
    }
}

#[test]
fn subgrid_named_placement_resolves_wpt_named_spans_before_clamping_to_span() {
    let parent = named_parent_lines(6, &[&["a"], &[], &[], &[], &["b"], &[], &["a", "b"]]);
    let local = local_subgrid_entries(&[&["x"], &["b"], &[], &[], &["b"]]);
    let lines = named::inherit_subgrid_named_lines(&parent, 2, 6, false, &local, None).unwrap();

    let cases = [
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 2,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(1, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
    ];

    for (raw, expected) in cases {
        let placement = named::resolve_subgrid_placement(&lines, &raw, None).unwrap();
        assert_eq!(placement, expected, "raw placement {raw:?}");
    }
}

#[test]
fn subgrid_named_placement_expands_collapsed_clamp_to_edge_track() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 1);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(3)),
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(1, 2).expect("valid grid lines")
    );
}

#[test]
fn subgrid_named_span_counts_implicit_names_beyond_end_before_clamping() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 10);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 1,
            },
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 8,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(10, 11).expect("valid grid lines")
    );
}

fn named_parent_lines(
    explicit_track_count: usize,
    line_names: &[&[&str]],
) -> named::NamedGridLines {
    let mut lines = named::NamedGridLines::new(GridAxisKind::Column, explicit_track_count);
    for (line_index, names) in line_names.iter().enumerate() {
        lines.line_names[line_index] = names
            .iter()
            .map(|name| named::LineNameEntry {
                name: (*name).to_string(),
                origin: named::LineNameOrigin::Explicit,
            })
            .collect();
    }
    lines
}

fn test_inherited_axis(
    named_lines: named::NamedGridLines,
    area_facts: Option<named::GridAreaNameFacts>,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
) -> InheritedGridAxis {
    let track_count = parent_end - parent_start;
    InheritedGridAxis {
        offset: 0.0,
        gap: 0.0,
        tracks: vec![0.0; track_count],
        geometry: UsedGridAxisGeometryOf::new(
            vec![0.0; track_count],
            vec![false; track_count],
            0.0,
        ),
        named_lines,
        area_facts,
        template_area_expanded: false,
        major_baselines: vec![None; track_count],
        minor_baselines: vec![None; track_count],
        owner_baseline_targets: None,
        parent_start,
        parent_end,
        reversed,
    }
}

fn local_subgrid_entries(line_names: &[&[&str]]) -> Vec<Vec<named::LineNameEntry>> {
    line_names
        .iter()
        .map(|names| {
            names
                .iter()
                .map(|name| named::LineNameEntry {
                    name: (*name).to_string(),
                    origin: named::LineNameOrigin::LocalSubgrid,
                })
                .collect()
        })
        .collect()
}

#[test]
fn vertical_subgrid_percentage_gap_uses_flow_relative_axis_basis() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalLr,
        gap: Size::new(Length::percent(0.10), Length::percent(0.10)),
        ..NodeInput::default()
    };
    let area_size = Size::new(300.0, 500.0);

    assert_eq!(
        child_subgrid_gap(&style, GridAxisKind::Column, area_size),
        Ok(ResolvedSubgridGap::Length(50.0))
    );
    assert_eq!(
        child_subgrid_gap(&style, GridAxisKind::Row, area_size),
        Ok(ResolvedSubgridGap::Length(30.0))
    );
}

#[test]
fn vertical_subgrid_percentage_edges_use_physical_area_basis() {
    let parent_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        grid_template_columns: subgrid_track(),
        padding: Edges::all(Length::percent(0.1)),
        ..NodeInput::default()
    };
    let tree = OracleTree::new()
        .children(2, [3])
        .children(3, [])
        .style(2, child_style.clone())
        .style(3, NodeInput::default());
    let area = GridArea {
        column: 0,
        column_end: 1,
        row: 0,
        row_end: 1,
        size: LogicalSizeOf::new(200.0, 100.0),
    };
    let named_columns = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let named_rows = named::NamedGridLines::new(GridAxisKind::Row, 2);

    let children = [2];
    let placed_areas = [Some(area)];
    let placements = single_grid_placement_context(2, &child_style);
    let subgrid_report = GridSubgridReport {
        items: vec![SubgridItemReport {
            node: 2,
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        }],
    };
    let report = collect_grid_subgrid_intrinsic_traversal::<OracleTree, core::convert::Infallible>(
        &tree,
        GridSubgridIntrinsicTraversalInput {
            axis: GridAxisKind::Column,
            containing_flow_axes: crate::geometry::FlowAxes::new(
                parent_style.writing_mode,
                parent_style.direction,
            ),
            children: &children,
            placed_areas: &placed_areas,
            placements: &placements,
            subgrid_report: &subgrid_report,
            named_columns: &named_columns,
            named_rows: &named_rows,
            area_facts: None,
            parent_gap: Size::ZERO,
            column_gutters: None,
            row_gutters: None,
            column_sizes: &[200.0, 1.0],
            row_sizes: &[100.0, 1.0],
            container_size: Size::new(Some(100.0), Some(200.0)),
            intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(&[true, false]),
        },
    )
    .unwrap()
    .expect("eligible subgrid traversal must produce a report");

    assert_eq!(report.edge_lower_bounds, vec![40.0, 0.0]);
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![40.0, 0.0]
    );
}

#[test]
fn orthogonal_subgrid_percentage_edges_use_containing_physical_area_basis() {
    let parent_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        grid_template_rows: subgrid_track(),
        padding: Edges::all(Length::percent(0.1)),
        ..NodeInput::default()
    };
    let tree = OracleTree::new()
        .children(2, [3])
        .children(3, [])
        .style(2, child_style.clone())
        .style(3, NodeInput::default());
    let area = GridArea {
        column: 0,
        column_end: 1,
        row: 0,
        row_end: 1,
        size: LogicalSizeOf::new(200.0, 100.0),
    };
    let named_columns = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let named_rows = named::NamedGridLines::new(GridAxisKind::Row, 2);

    let children = [2];
    let placed_areas = [Some(area)];
    let placements = single_grid_placement_context(2, &child_style);
    let subgrid_report = GridSubgridReport {
        items: vec![SubgridItemReport {
            node: 2,
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        }],
    };
    let report = collect_grid_subgrid_intrinsic_traversal::<OracleTree, core::convert::Infallible>(
        &tree,
        GridSubgridIntrinsicTraversalInput {
            axis: GridAxisKind::Column,
            containing_flow_axes: crate::geometry::FlowAxes::new(
                parent_style.writing_mode,
                parent_style.direction,
            ),
            children: &children,
            placed_areas: &placed_areas,
            placements: &placements,
            subgrid_report: &subgrid_report,
            named_columns: &named_columns,
            named_rows: &named_rows,
            area_facts: None,
            parent_gap: Size::ZERO,
            column_gutters: None,
            row_gutters: None,
            column_sizes: &[200.0, 1.0],
            row_sizes: &[100.0, 1.0],
            container_size: Size::new(Some(100.0), Some(200.0)),
            intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(&[true, false]),
        },
    )
    .unwrap()
    .expect("eligible subgrid traversal must produce a report");

    assert_eq!(report.edge_lower_bounds, vec![40.0, 0.0]);
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![40.0, 0.0]
    );
}

#[test]
fn nested_subgrid_same_flow_projects_physical_edge_sums_before_local_track_sizing() {
    let parent_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let outer_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        grid_template_rows: subgrid_track(),
        grid_template_columns: vec![TrackComponent::percent(1.0)],
        margin: Edges::new(
            LengthAuto::px(3.0),
            LengthAuto::px(5.0),
            LengthAuto::px(7.0),
            LengthAuto::px(11.0),
        ),
        border: Edges::new(
            Length::px(13.0),
            Length::px(17.0),
            Length::px(19.0),
            Length::px(23.0),
        ),
        padding: Edges::new(
            Length::px(29.0),
            Length::px(31.0),
            Length::px(37.0),
            Length::px(41.0),
        ),
        ..NodeInput::default()
    };
    let inner_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        grid_template_rows: subgrid_track(),
        grid_template_columns: vec![TrackComponent::percent(1.0)],
        padding: Edges::new(
            Length::percent(0.1),
            Length::percent(0.2),
            Length::percent(0.3),
            Length::percent(0.4),
        ),
        ..NodeInput::default()
    };
    let tree = OracleTree::new()
        .children(2, [3])
        .children(3, [4])
        .children(4, [])
        .style(2, outer_style.clone())
        .style(3, inner_style)
        .style(4, NodeInput::default());
    let area = GridArea {
        column: 0,
        column_end: 1,
        row: 0,
        row_end: 1,
        size: LogicalSizeOf::new(200.0, 100.0),
    };
    let children = [2];
    let placed_areas = [Some(area)];
    let placements = single_grid_placement_context(2, &outer_style);
    let subgrid_report = GridSubgridReport {
        items: vec![SubgridItemReport {
            node: 2,
            column: subgrid_axis_report(&parent_style, &outer_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &outer_style, GridAxisKind::Row),
        }],
    };
    let report = collect_grid_subgrid_intrinsic_traversal::<OracleTree, core::convert::Infallible>(
        &tree,
        GridSubgridIntrinsicTraversalInput {
            axis: GridAxisKind::Row,
            containing_flow_axes: crate::geometry::FlowAxes::new(
                parent_style.writing_mode,
                parent_style.direction,
            ),
            children: &children,
            placed_areas: &placed_areas,
            placements: &placements,
            subgrid_report: &subgrid_report,
            named_columns: &named::NamedGridLines::new(GridAxisKind::Column, 2),
            named_rows: &named::NamedGridLines::new(GridAxisKind::Row, 2),
            area_facts: None,
            parent_gap: Size::ZERO,
            column_gutters: None,
            row_gutters: None,
            column_sizes: &[200.0, 1.0],
            row_sizes: &[100.0, 1.0],
            container_size: Size::new(Some(100.0), Some(200.0)),
            intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(&[true, false]),
        },
    )
    .unwrap()
    .expect("eligible nested same-flow traversal must produce a report");

    assert_eq!(
        report.leaves[0].available_inline_size,
        Some(55.2),
        "physical vertical edge sums must reduce the local inline track before nesting"
    );
}

#[test]
fn vertical_grid_axis_offsets_add_local_inset_to_inherited_offsets() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalLr,
        ..NodeInput::default()
    };
    let tracks = [20.0, 30.0];
    let alignment = GridAlignment {
        start: 7.0,
        gap: 5.0,
    };
    let content_box_inset = Edges {
        left: 11.0,
        right: 0.0,
        top: 13.0,
        bottom: 0.0,
    };
    let geometry = UsedGridAxisGeometryOf::new(tracks.to_vec(), vec![false; tracks.len()], 5.0);
    let style_projection = grid_container_projection!(&style);

    let column_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style: &style_projection,
        axis: GridAxisKind::Column,
        tracks: &tracks,
        geometry: &geometry,
        inherited_offset: Some(100.0),
        content_box_left: 0.0,
        content_box_size: Size::new(300.0, 400.0),
        content_box_inset,
        alignment,
    });
    let row_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style: &style_projection,
        axis: GridAxisKind::Row,
        tracks: &tracks,
        geometry: &geometry,
        inherited_offset: Some(200.0),
        content_box_left: 0.0,
        content_box_size: Size::new(300.0, 400.0),
        content_box_inset,
        alignment,
    });

    assert_eq!(column_offsets, vec![120.0, 145.0]);
    assert_eq!(row_offsets, vec![218.0, 243.0]);
}

#[test]
fn subgrid_eligibility_reports_first_blocking_reason() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: false,
        child_style: &NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(report.reason, Some(SubgridIneligibleReason::NoParentGrid));
}

#[test]
fn subgrid_eligibility_rejects_non_grid_container_display() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Block,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report.reason,
        Some(SubgridIneligibleReason::UnsupportedDisplay)
    );
}

#[test]
fn subgrid_axis_report_allows_supported_vertical_parent_mapping_to_inherit() {
    let report = subgrid_axis_report(
        &NodeInput {
            display: Display::Grid,
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
        &NodeInput {
            display: Display::Grid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
        GridAxisKind::Column,
    );

    assert!(report.eligibility.eligible);
    assert_eq!(
        report.mapping,
        GridAxisMappingReport {
            queried_axis: GridAxisKind::Column,
            parent_axis: GridAxisKind::Row,
            child_axis: GridAxisKind::Column,
            reversed: true,
        }
    );
    assert!(report.can_inherit());
}

fn subgrid_item_report(parent: &NodeInput, child: &NodeInput) -> SubgridItemReport<()> {
    SubgridItemReport {
        node: (),
        column: subgrid_axis_report(parent, child, GridAxisKind::Column),
        row: subgrid_axis_report(parent, child, GridAxisKind::Row),
    }
}

fn grid_area(column: usize, column_end: usize, row: usize, row_end: usize) -> GridArea {
    GridArea {
        column,
        column_end,
        row,
        row_end,
        size: LogicalSizeOf::new(0.0, 0.0),
    }
}

#[test]
fn intrinsic_subgrid_context_is_needed_for_both_axis_subgrids() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 3, 0, 2),
    ));
}

#[test]
fn intrinsic_subgrid_context_is_not_needed_for_single_column_both_axis_subgrid() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(!needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
    ));
}

#[test]
fn intrinsic_subgrid_context_is_needed_for_row_subgrid_with_percent_columns() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: vec![TrackComponent::percent(0.5)],
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
    ));
}

#[test]
fn intrinsic_subgrid_context_uses_mapped_parent_axis_for_orthogonal_subgrid() {
    let parent = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Column,
        grid_template_columns: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
    ));
}

#[test]
fn subgrid_eligibility_rejects_grid_lanes_parent_in_lane_axis() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            display: Display::GridLanes,
            grid_auto_flow: GridAutoFlow::Row,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            grid_template_rows: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report.reason,
        Some(SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    );
}

#[test]
fn subgrid_eligibility_allows_grid_lanes_parent_in_grid_axis() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::GridLanes,
            grid_auto_flow: GridAutoFlow::Row,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(report.eligible);
    assert_eq!(report.reason, None);
}

#[test]
fn subgrid_eligibility_treats_inline_grid_lanes_parent_as_lanes() {
    let rejected = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            display: Display::InlineGridLanes,
            grid_auto_flow: GridAutoFlow::Row,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::InlineGrid,
            grid_template_rows: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        rejected.reason,
        Some(SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    );

    let allowed = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::InlineGridLanes,
            grid_auto_flow: GridAutoFlow::Row,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::InlineGrid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(allowed.eligible);
    assert_eq!(allowed.reason, None);
}

#[test]
fn subgrid_eligibility_allows_ordinary_grid_parent_in_both_axes() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let report = subgrid_eligibility(SubgridEligibilityInput {
            axis,
            parent_style: &parent,
            has_parent_grid: true,
            child_style: &child,
        });

        assert!(report.eligible, "{axis:?} subgrid should be eligible");
        assert_eq!(report.reason, None);
    }
}

#[test]
fn subgrid_track_inheritance_copies_parent_columns_for_span() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 60.0, 90.0],
        parent_span: GridTrackSpan::new(2, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.copied_parent_tracks, vec![60.0, 90.0]);
    assert_eq!(report.final_tracks, vec![60.0, 90.0]);
}

#[test]
fn subgrid_track_inheritance_reverses_copied_columns_before_mbp_consumption() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 60.0, 90.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: true,
        start_mbp: 10.0,
        end_mbp: 20.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.after_reversal, vec![90.0, 60.0, 40.0]);
    assert_eq!(report.final_tracks, vec![80.0, 60.0, 20.0]);
}

#[test]
fn subgrid_track_inheritance_consumes_start_and_end_mbp_across_tracks() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[5.0, 20.0, 10.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 12.0,
        end_mbp: 25.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(0.0),
    })
    .unwrap();

    assert_eq!(report.start_mbp_removed, vec![0.0, 13.0, 10.0]);
    assert_eq!(report.end_mbp_removed, vec![0.0, 0.0, 0.0]);
    assert_eq!(report.final_tracks, vec![0.0, 0.0, 0.0]);
}

#[test]
fn subgrid_track_inheritance_resolves_normal_gap_to_parent_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[50.0, 50.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Normal,
    })
    .unwrap();

    assert_eq!(report.resolved_subgrid_gap, 20.0);
    assert_eq!(report.gap_difference, 0.0);
    assert_eq!(report.final_tracks, vec![50.0, 50.0]);
}

#[test]
fn subgrid_track_inheritance_applies_explicit_gap_difference_to_internal_edges() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[50.0, 50.0, 50.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(20.0),
    })
    .unwrap();

    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.final_tracks, vec![45.0, 40.0, 45.0]);
}

#[test]
fn column_subgrid_layout_tracks_expand_collapsed_tracks_into_shifted_lines() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(150.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Column, &report);

    assert_eq!(report.final_tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(tracks, vec![175.0, 100.0, 25.0]);
    assert_eq!(gap, 0.0);
}

#[test]
fn row_subgrid_layout_tracks_keep_collapsed_tracks_with_resolved_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(150.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Row, &report);

    assert_eq!(report.final_tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(gap, 150.0);
}

#[test]
fn subgrid_layout_tracks_keep_non_collapsed_gap_sizing() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Length(100.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Column, &report);

    assert_eq!(report.final_tracks, vec![60.0, 60.0]);
    assert_eq!(tracks, vec![60.0, 60.0]);
    assert_eq!(gap, 100.0);
}

#[test]
fn subgrid_track_inheritance_expands_tracks_for_smaller_subgrid_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 40.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.gap_difference, -5.0);
    assert_eq!(report.final_tracks, vec![45.0, 45.0]);
}

#[test]
fn subgrid_track_inheritance_rejects_empty_parent_tracks() {
    let err = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[],
        parent_span: GridTrackSpan::new(1, 2),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Normal,
    })
    .unwrap_err();

    assert_eq!(err, SubgridTrackInheritanceError::EmptyTrackList);
}

#[test]
fn subgrid_track_inheritance_rejects_invalid_parent_spans() {
    for span in [
        GridTrackSpan::new(0, 2),
        GridTrackSpan::new(2, 2),
        GridTrackSpan::new(3, 2),
        GridTrackSpan::new(1, 4),
    ] {
        let err = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
            parent_tracks: &[10.0, 20.0],
            parent_span: span,
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: 0.0,
            subgrid_gap: ResolvedSubgridGap::Normal,
        })
        .unwrap_err();

        assert_eq!(err, SubgridTrackInheritanceError::SpanOutOfRange);
    }
}

fn traversal_subgrid(
    node: u32,
    start: usize,
    end: usize,
    children: Vec<SubgridTraversalChild<u32>>,
) -> SubgridTraversalChild<u32> {
    SubgridTraversalChild::Subgrid(SubgridTraversalNode {
        node,
        style: default_grid_item_projection(),
        axis: SubgridTraversalAxis::Inherited,
        reversed: false,
        span_in_parent: GridTrackSpan::new(start, end),
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
        children,
    })
}

#[test]
fn subgrid_traversal_keeps_edge_lower_bounds_off_non_intrinsic_tracks() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[false, false]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 3),
            available_inline_size: None,
            available_inline_size_is_known: false,
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges {
                start: 10.0,
                end: 12.0,
            },
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: Vec::new(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 0.0]);
}

#[test]
fn subgrid_traversal_places_edge_lower_bounds_in_ancestor_track_space() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(2, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges {
                start: 20.0,
                end: 30.0,
            },
            parent_gap: 20.0,
            subgrid_gap: 10.0,
            children: vec![traversal_leaf(2, 1, 2)],
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 20.0, 0.0, 30.0]);
}

#[test]
fn subgrid_traversal_reports_missing_intrinsic_min_facts() {
    let err = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Unknown,
        root_children: vec![traversal_subgrid(1, 1, 2, Vec::new())],
    })
    .unwrap_err();

    assert_eq!(err, SubgridTraversalError::MissingIntrinsicMinTrackFacts);
}

#[test]
fn subgrid_traversal_accumulates_edge_adjustment_in_nested_translated_span() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges {
                start: 2.0,
                end: 4.0,
            },
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
                node: 2,
                style: default_grid_item_projection(),
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(2, 3),
                available_inline_size: None,
                available_inline_size_is_known: false,
                align_self: AlignItems::Stretch,
                standalone_parent_context: None,
                queried_axis_fully_inherited: true,
                margins: SubgridAxisEdges {
                    start: 3.0,
                    end: 5.0,
                },
                border: SubgridAxisEdges::default(),
                padding: SubgridAxisEdges::default(),
                parent_gap: 0.0,
                subgrid_gap: 0.0,
                children: vec![traversal_leaf(3, 1, 2)],
            })],
        })],
    })
    .unwrap();

    assert_eq!(report.leaves[0].ancestor_span, GridTrackSpan::new(2, 3));
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![2.0, 8.0, 4.0]
    );
}

#[test]
fn subgrid_traversal_accumulates_gap_adjustment_through_nested_subgrids() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
                node: 2,
                style: default_grid_item_projection(),
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(2, 3),
                available_inline_size: None,
                available_inline_size_is_known: false,
                align_self: AlignItems::Stretch,
                standalone_parent_context: None,
                queried_axis_fully_inherited: true,
                margins: SubgridAxisEdges::default(),
                border: SubgridAxisEdges::default(),
                padding: SubgridAxisEdges::default(),
                parent_gap: 20.0,
                subgrid_gap: 28.0,
                children: vec![traversal_leaf(3, 1, 2)],
            })],
        })],
    })
    .unwrap();

    assert_eq!(
        report.leaves[0].accumulated_gap_adjustment,
        vec![5.0, 10.0, 5.0]
    );
}

#[test]
fn subgrid_traversal_applies_gap_adjustment_to_internal_edges() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: vec![traversal_leaf(2, 2, 3)],
        })],
    })
    .unwrap();

    assert_eq!(report.leaves[0].ancestor_span, GridTrackSpan::new(2, 3));
    assert_eq!(
        report.leaves[0].accumulated_gap_adjustment,
        vec![5.0, 10.0, 5.0]
    );
}

#[test]
fn subgrid_traversal_uses_positive_gap_adjustments_as_empty_track_lower_bounds() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 10.0,
            children: Vec::new(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![5.0, 10.0, 10.0, 5.0]);
}

#[test]
fn subgrid_traversal_combines_empty_edge_and_gap_lower_bounds() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges {
                start: 21.0,
                end: 9.0,
            },
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: Vec::new(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![26.0, 10.0, 10.0, 14.0]);
}

#[test]
fn subgrid_traversal_ignores_gap_adjustment_for_single_track_subgrid() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            style: default_grid_item_projection(),
            axis: SubgridTraversalAxis::Inherited,
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
            parent_gap: 10.0,
            subgrid_gap: 30.0,
            children: vec![traversal_leaf(2, 1, 2)],
        })],
    })
    .unwrap();

    assert_eq!(report.leaves[0].accumulated_gap_adjustment, vec![0.0]);
}

#[test]
fn subgrid_traversal_keeps_standalone_as_one_boundary_leaf() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
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

    assert_eq!(report.leaves.len(), 1);
    assert_eq!(report.leaves[0].node, 1);
    assert_eq!(report.leaves[0].ancestor_span, GridTrackSpan::new(1, 2));
    assert!(report.leaves[0].standalone_parent_context.is_none());
}
