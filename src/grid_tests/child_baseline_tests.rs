use super::fixtures::{
    Fri04C04GridSizingValue, Fri08C04BaselineFlowCase, Fri08C04BaselineMeasureMode,
    Fri08C04BaselineParentAxis, Fri08C04BaselineTree, GridAxisMappingInput, GridTestRetainedState,
    SubgridChildParentContextInput, SubgridEligibilityInput,
    assert_fri06_mr02_geometry_error_grid_own,
    assert_fri08_c03_containing_block_percentage_children,
    assert_fri08_c04_baseline_area_topology_controls, baseline_measure, compute_oracle_grid,
    compute_oracle_grid_output, computed_overflow, default_grid_item_projection,
    derive_inherited_placement, empty_subgrid_track, final_y, fri04_c04_grid_dispatch_assert_error,
    fri04_c04_grid_dispatch_style, fri05_c05_grid_sizing_input,
    fri06_c12_t08_inherited_baseline_gap_position, fri06_mr02_geometry_error_assert,
    fri08_c01_placement_compute, fri08_c01_placement_output, fri08_c02_auto_fit_output,
    fri08_c02_auto_fit_repeat, fri08_c03_intrinsic_facts, fri08_c03_intrinsic_projected_item,
    fri08_c04_baseline_area_implicit_tree, fri08_c04_baseline_physical_edge,
    fri08_c04_baseline_world_coordinate, inherited_placement_group, inherited_placement_mapping,
    inherited_placement_member, inherited_placement_witness, lp, map_grid_axis,
    single_grid_placement_context, subgrid_axis_report, subgrid_child_parent_context,
    subgrid_eligibility, subgrid_track, subgrid_track_of, tagged_baseline, tagged_group,
    vertical_baseline_measure, with_projected_subgrid_child_input,
};
use super::*;

fn subgrid_child_parent_context_from_ancestor_groups_with_geometry<
    Node: Copy + PartialEq,
    S: LayoutScalar,
>(
    input: SubgridChildParentContextInput<'_, Node, S>,
    ancestor_baseline_groups: &FinalAncestorBaselineGroups<Node, S>,
    parent_template_area_expanded_axes: TemplateAreaExpandedAxes,
    parent_grid: Node,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>> {
    with_projected_subgrid_child_input(input, |input| {
        super::child::subgrid_child_parent_context_from_ancestor_groups_with_geometry(
            input,
            ancestor_baseline_groups,
            parent_template_area_expanded_axes,
            parent_grid,
            column_geometry,
            row_geometry,
        )
    })
}

#[test]
fn fri08_c03_containing_block_percentage_children_f32() {
    assert_fri08_c03_containing_block_percentage_children::<f32>();
}

#[test]
fn fri08_c03_containing_block_percentage_children_f64() {
    assert_fri08_c03_containing_block_percentage_children::<f64>();
}

#[test]
fn fri08_c03_containing_block_subgrid_child_receives_hybrid_context() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                grid_template_rows: vec![TrackComponent::px(40.0)],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::AUTO),
                grid_template_columns: vec![TrackComponent::percent(1.0)],
                grid_template_rows: vec![TrackComponent::Subgrid(SubgridTrack::new(vec![]))],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(10.0)),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInput::DEFAULT
            },
        )
        .measure(3, Size::ZERO);
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequest::viewport(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT))
            .expect("subgrid containing-block viewport"),
    )
    .expect("grid-lanes subgrid layout succeeds");

    let subgrid = fri08_c01_placement_output(&batch, 2);
    let descendant = fri08_c01_placement_output(&batch, 3);
    assert_eq!(subgrid.size, Size::new(100.0, 40.0));
    assert_eq!(descendant.size, Size::new(100.0, 10.0));
    assert_eq!(descendant.location, Point::ZERO);
}

#[test]
fn fri08_c03_containing_block_baseline_synthesis_survives_final_layout() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(100.0, 20.0),
        Size::new(100.0, 20.0),
        Baselines {
            first: Point::new(None, Some(7.0)),
            last: Point::new(None, Some(15.0)),
        },
    );
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                grid_template_rows: vec![TrackComponent::px(40.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, child_output);
    let output = crate::compute_grid(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(40.0)),
            ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .expect("baseline lanes layout succeeds");

    assert_eq!(
        tree.layout(2).expect("baseline child layout").size.width,
        100.0
    );
    assert_eq!(output.first_baselines.y, Some(7.0));
    assert_eq!(output.last_baselines.y, Some(15.0));
}

#[test]
fn fri08_c03_intrinsic_equivalence_requires_candidates_baseline_role_and_edges() {
    let input = LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: None,
        gap: 0.0,
        tracks: vec![TrackSizing::AUTO; 3],
        content_sized_tracks: vec![0, 1, 2],
        items: Vec::new(),
    };
    let zero_edges = LaneIntrinsicEdgeFactsOf::default();
    let items = vec![
        fri08_c03_intrinsic_projected_item(
            "equivalent-a",
            1,
            None,
            LaneIntrinsicBaselineRole::None,
            zero_edges,
            fri08_c03_intrinsic_facts(5.0, 30.0, 40.0),
        ),
        fri08_c03_intrinsic_projected_item(
            "equivalent-b",
            1,
            None,
            LaneIntrinsicBaselineRole::None,
            zero_edges,
            fri08_c03_intrinsic_facts(20.0, 10.0, 50.0),
        ),
        fri08_c03_intrinsic_projected_item(
            "equivalent-candidate-set",
            1,
            Some(vec![2, 1, 0, 1]),
            LaneIntrinsicBaselineRole::None,
            zero_edges,
            fri08_c03_intrinsic_facts(1.0, 2.0, 3.0),
        ),
        fri08_c03_intrinsic_projected_item(
            "different-candidates",
            1,
            Some(vec![0]),
            LaneIntrinsicBaselineRole::None,
            zero_edges,
            fri08_c03_intrinsic_facts(7.0, 8.0, 9.0),
        ),
        fri08_c03_intrinsic_projected_item(
            "different-baseline",
            1,
            None,
            LaneIntrinsicBaselineRole::First,
            zero_edges,
            fri08_c03_intrinsic_facts(7.0, 8.0, 9.0),
        ),
        fri08_c03_intrinsic_projected_item(
            "different-edges",
            1,
            None,
            LaneIntrinsicBaselineRole::None,
            LaneIntrinsicEdgeFactsOf {
                start_mbp: 1.0,
                ..zero_edges
            },
            fri08_c03_intrinsic_facts(7.0, 8.0, 9.0),
        ),
    ];
    let report = lane_intrinsic_sizing_projected_with::<(), _, core::convert::Infallible>(
        &input,
        &items,
        None,
        LayoutErrorSite::Standalone,
    )
    .expect("projection values are finite")
    .expect("projection spans are valid");

    assert_eq!(report.indefinite_groups.len(), 4);
    let equivalent = report
        .indefinite_groups
        .iter()
        .find(|group| {
            group.item_ids == ["equivalent-a", "equivalent-b", "equivalent-candidate-set"]
        })
        .expect("equal keys share one componentwise maximum group");
    assert_eq!(equivalent.max_min_size, 20.0);
    assert_eq!(equivalent.max_min_content, 30.0);
    assert_eq!(equivalent.max_max_content, 50.0);
}

fn assert_fri08_c04_baseline_area_implicit_composition<S: LayoutScalar>() {
    let cases = [
        Fri08C04BaselineFlowCase {
            parent_axis: Fri08C04BaselineParentAxis::Row,
            root_writing_mode: WritingMode::HorizontalTb,
            root_direction: Direction::Ltr,
            child_writing_mode: WritingMode::HorizontalTb,
            child_direction: Direction::Rtl,
        },
        Fri08C04BaselineFlowCase {
            parent_axis: Fri08C04BaselineParentAxis::Row,
            root_writing_mode: WritingMode::VerticalRl,
            root_direction: Direction::Ltr,
            child_writing_mode: WritingMode::VerticalLr,
            child_direction: Direction::Rtl,
        },
        Fri08C04BaselineFlowCase {
            parent_axis: Fri08C04BaselineParentAxis::Row,
            root_writing_mode: WritingMode::SidewaysRl,
            root_direction: Direction::Rtl,
            child_writing_mode: WritingMode::SidewaysLr,
            child_direction: Direction::Ltr,
        },
        Fri08C04BaselineFlowCase {
            parent_axis: Fri08C04BaselineParentAxis::Column,
            root_writing_mode: WritingMode::VerticalLr,
            root_direction: Direction::Ltr,
            child_writing_mode: WritingMode::HorizontalTb,
            child_direction: Direction::Ltr,
        },
    ];
    for (case_index, case) in cases.into_iter().enumerate() {
        let mut role_coordinates = Vec::new();
        for alignment in [AlignItems::Baseline, AlignItems::LastBaseline] {
            let tree = fri08_c04_baseline_area_implicit_tree::<S>(case, alignment);
            let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
                .expect("baseline composition viewport");
            let batch = compute_layout(&tree, 1, request)
                .expect("area/implicit inherited baseline layout succeeds");
            let (direct, inherited) = fri08_c04_baseline_world_coordinate(&batch, case, alignment);
            if matches!(case_index, 0 | 3) {
                assert!(
                    (direct - inherited).abs() <= S::from_f64(0.001),
                    "{case:?} {alignment:?} direct/current targets agree: direct={direct:?}, inherited={inherited:?}"
                );
            } else {
                let expected = match alignment {
                    AlignItems::Baseline => (70.0, 137.0),
                    AlignItems::LastBaseline => (90.0, 100.0),
                    _ => unreachable!("the role loop contains only first and last baseline"),
                };
                assert!(
                    (direct - S::from_f64(expected.0)).abs() <= S::from_f64(0.001)
                        && (inherited - S::from_f64(expected.1)).abs() <= S::from_f64(0.001),
                    "{case:?} {alignment:?} preserves reversed physical projection: direct={direct:?}, inherited={inherited:?}"
                );
            }
            assert_eq!(
                batch
                    .final_entries()
                    .iter()
                    .map(LayoutOutputEntryOf::node)
                    .collect::<Vec<_>>(),
                [1, 2, 4, 6, 3, 5, 7],
                "item order changes placement traversal without changing source publication"
            );
            let implicit = fri08_c01_placement_output(&batch, 2);
            let nested = fri08_c01_placement_output(&batch, 3);
            assert_ne!(
                implicit.location, nested.location,
                "the implicit and area-created tracks remain distinct"
            );
            role_coordinates.push((direct, inherited));
        }
        assert_eq!(
            role_coordinates.len(),
            2,
            "first and last roles both execute"
        );
    }
}

#[test]
fn fri08_c04_baseline_area_created_implicit_standalone_roles_map_both_axes_and_scalars() {
    assert_fri08_c04_baseline_area_implicit_composition::<f32>();
    assert_fri08_c04_baseline_area_implicit_composition::<f64>();
}

#[test]
fn fri08_c04_baseline_area_topology_controls_preserve_first_and_last_roles() {
    assert_fri08_c04_baseline_area_topology_controls::<f32>();
    assert_fri08_c04_baseline_area_topology_controls::<f64>();
}

fn fri08_c04_baseline_lanes_auto_fit_tree<S: LayoutScalar>(
    alignment: AlignItems,
) -> Fri08C04BaselineTree<S> {
    let scalar = S::from_f64;
    let child_axes = FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);
    let repeat = TrackRepetitionOf::auto_fit_components(vec![TrackComponentOf::px(scalar(40.0))])
        .expect("finite lanes auto-fit component");
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 4, 3])
        .children(2, [])
        .children(3, [5])
        .children(4, [6])
        .children(5, [7])
        .children(6, [])
        .children(7, [8])
        .children(8, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalLr,
                direction: Direction::Ltr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(160.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(scalar(30.0))],
                grid_template_rows: vec![TrackComponentOf::Repeat(repeat)],
                grid_auto_flow: GridAutoFlow::Column,
                gap: Size::new(LengthOf::px(scalar(10.0)), LengthOf::px(scalar(7.0))),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                direction: Direction::Ltr,
                item_order: ItemOrder::new(0),
                size: Size::new(
                    PreferredSizeOf::px(scalar(8.0)),
                    PreferredSizeOf::px(scalar(8.0)),
                ),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalLr,
                direction: Direction::Ltr,
                grid_row: GridPlacement::try_line(2).expect("direct auto-fit row"),
                item_order: ItemOrder::new(8),
                justify_self: Some(AlignItems::Start),
                align_self: Some(alignment),
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO],
                justify_items: Some(AlignItems::Start),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalLr,
                direction: Direction::Ltr,
                grid_row: GridPlacement::try_line_span(1, 2)
                    .expect("lanes subgrid spans auto-fit rows"),
                item_order: ItemOrder::new(-8),
                grid_template_columns: vec![TrackComponentOf::px(scalar(30.0))],
                grid_template_rows: subgrid_track_of(),
                grid_auto_flow: GridAutoFlow::Column,
                gap: child_axes
                    .physical_size(LogicalSizeOf::new(scalar(3.0), scalar(10.0)))
                    .map(LengthOf::px),
                margin: Edges::all(LengthAutoOf::ZERO),
                border: Edges::all(LengthOf::ZERO),
                padding: Edges::all(LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                display: Display::GridLanes,
                writing_mode: WritingMode::VerticalLr,
                direction: Direction::Ltr,
                grid_column: GridPlacement::try_line(1).expect("standalone lanes column"),
                grid_row: GridPlacement::try_line(2).expect("inherited auto-fit track"),
                item_order: ItemOrder::new(-9),
                justify_self: Some(AlignItems::Start),
                align_self: Some(alignment),
                grid_template_columns: subgrid_track_of(),
                grid_template_rows: vec![TrackComponentOf::AUTO],
                justify_items: Some(AlignItems::Start),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            6,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                direction: Direction::Ltr,
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            7,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalLr,
                direction: Direction::Ltr,
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO],
                justify_items: Some(AlignItems::Start),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            8,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                direction: Direction::Ltr,
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        );
    Fri08C04BaselineTree {
        tree,
        measurements: HashMap::from([
            (
                6,
                child_axes.physical_size(LogicalSizeOf::new(scalar(18.0), scalar(28.0))),
            ),
            (
                8,
                child_axes.physical_size(LogicalSizeOf::new(scalar(16.0), scalar(12.0))),
            ),
        ]),
        failing_node: 8,
        measure_mode: std::cell::Cell::new(Fri08C04BaselineMeasureMode::Values),
        measurement_requests: std::cell::RefCell::new(Vec::new()),
        cache_queries: std::cell::RefCell::new(Vec::new()),
        retained: GridTestRetainedState::default(),
    }
}

fn assert_fri08_c04_baseline_lanes_auto_fit<S: LayoutScalar>() {
    let flow_axes = FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);
    for alignment in [AlignItems::Baseline, AlignItems::LastBaseline] {
        let tree = fri08_c04_baseline_lanes_auto_fit_tree::<S>(alignment);
        let batch = compute_layout(
            &tree,
            1,
            LayoutRootRequestOf::viewport(root_axes_size::<S>(160.0, 100.0))
                .expect("lanes auto-fit viewport"),
        )
        .expect("lanes auto-fit subgrid baseline layout succeeds");
        let direct = fri08_c01_placement_output(&batch, 4).location.x
            + fri08_c04_baseline_physical_edge(
                fri08_c01_placement_output(&batch, 6),
                flow_axes,
                alignment,
            );
        let inherited = fri08_c01_placement_output(&batch, 3).location.x
            + fri08_c01_placement_output(&batch, 5).location.x
            + fri08_c01_placement_output(&batch, 7).location.x
            + fri08_c04_baseline_physical_edge(
                fri08_c01_placement_output(&batch, 8),
                flow_axes,
                alignment,
            );
        let expected = match alignment {
            AlignItems::Baseline => (78.0, 62.0),
            AlignItems::LastBaseline => (44.0, 60.0),
            _ => unreachable!("the role loop contains only first and last baseline"),
        };
        assert!(
            (direct - S::from_f64(expected.0)).abs() <= S::from_f64(0.001)
                && (inherited - S::from_f64(expected.1)).abs() <= S::from_f64(0.001),
            "lanes {alignment:?} retains its published grid-axis projection: direct={direct:?}, inherited={inherited:?}"
        );
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .map(LayoutOutputEntryOf::node)
                .collect::<Vec<_>>(),
            [1, 2, 4, 6, 3, 5, 7, 8]
        );
    }
}

fn root_axes_size<S: LayoutScalar>(inline: f64, block: f64) -> Size<AvailableOf<S>> {
    Size::new(
        AvailableOf::definite(S::from_f64(inline)),
        AvailableOf::definite(S::from_f64(block)),
    )
}

#[test]
fn fri08_c04_baseline_lanes_auto_fit_consumes_inherited_first_and_last_targets() {
    assert_fri08_c04_baseline_lanes_auto_fit::<f32>();
    assert_fri08_c04_baseline_lanes_auto_fit::<f64>();
}

#[test]
fn fri08_c06_collapsed_gutter_alignment_distributes_between_nearest_active_tracks() {
    let gutters = OrdinaryGridAxisGuttersOf::new(3, &[false, true, false], 10.0_f64);
    for (alignment, free_space, expected_start, expected_gutters) in [
        (AlignContent::Start, 20.0, 0.0, vec![10.0, 0.0]),
        (AlignContent::Center, 20.0, 10.0, vec![10.0, 0.0]),
        (AlignContent::SpaceBetween, 20.0, 0.0, vec![30.0, 0.0]),
        (AlignContent::SpaceAround, 20.0, 5.0, vec![20.0, 0.0]),
        (AlignContent::SpaceEvenly, 30.0, 10.0, vec![20.0, 0.0]),
    ] {
        let actual = ordinary_grid_axis_alignment(free_space, &gutters, alignment);
        assert_eq!(actual.start, expected_start, "{alignment:?} start");
        assert_eq!(
            actual.gutter_after, expected_gutters,
            "{alignment:?} coincident boundary gutters"
        );
    }
}

#[test]
fn fri08_c02_auto_fit_absolute_span_crosses_collapsed_lines_with_canonical_extent() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(3).expect("only occupied repetition"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_lines(1, 4).expect("cross collapsed lines"),
                grid_row: GridPlacement::try_lines(1, 2).expect("single row"),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );

    let in_flow = fri08_c02_auto_fit_output(&tree, Size::new(140.0, 20.0), 2);
    let absolute = fri08_c02_auto_fit_output(&tree, Size::new(140.0, 20.0), 3);
    assert_eq!((in_flow.location.x, in_flow.size.width), (50.0, 40.0));
    assert_eq!((absolute.location.x, absolute.size.width), (50.0, 40.0));
}

#[test]
fn fri08_c02_auto_fit_inherited_baseline_crosses_collapsed_boundary_without_uniform_gap_delta() {
    for reversed in [false, true] {
        let role = if reversed {
            AncestorBaselineRole::Last
        } else {
            AncestorBaselineRole::First
        };
        let owner_member = inherited_placement_member(91, GridAxisKind::Column, role, 2, 17.0);
        let owner_group = AncestorBaselineGroup::reduce(
            1_u32,
            GridAxisKind::Column,
            PhysicalAxis::Horizontal,
            4,
            [owner_member],
        );
        let ancestor_groups = final_ancestor_baseline_groups_for_transport_test(
            AncestorBaselineGroup::reduce(
                1_u32,
                GridAxisKind::Row,
                PhysicalAxis::Vertical,
                1,
                Vec::<AncestorBaselineMember<u32>>::new(),
            ),
            owner_group.clone(),
        );
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
            grid_template_columns: vec![empty_subgrid_track()],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(20.0), Length::ZERO),
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            ..NodeInput::DEFAULT
        };
        let parent_geometry = UsedGridAxisGeometryOf::new(
            vec![40.0, 0.0, 40.0, 40.0],
            vec![false, true, false, false],
            10.0,
        );
        let context = subgrid_child_parent_context_from_ancestor_groups_with_geometry(
            SubgridChildParentContextInput {
                item: SubgridItemReport {
                    node: 7_u32,
                    column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
                    row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
                },
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
                    columns: vec![TrackBaselineGroup::default(); 4],
                    rows: vec![TrackBaselineGroup::default()],
                },
                margin: Edges::all(Some(0.0)),
                border: Edges::ZERO,
                padding: Edges::ZERO,
            },
            &ancestor_groups,
            TemplateAreaExpandedAxes::default(),
            1_u32,
            Some(&parent_geometry),
            None,
        )
        .expect("collapsed ordinary geometry remains inheritable");
        let inherited = context.columns.as_ref().expect("column subgrid context");
        let transported = inherited
            .owner_baseline_targets
            .as_ref()
            .expect("owner baseline target remains transportable");
        let local_span = if reversed {
            GridTrackSpan::new(1, 2)
        } else {
            GridTrackSpan::new(2, 3)
        };
        let placement = InheritedCurrentGridBaselinePlacement::try_derive(
            &transported.group,
            InheritedCurrentGridBaselinePlacementInput {
                axis: GridAxisKind::Column,
                physical_axis: PhysicalAxis::Horizontal,
                mapping: transported.mapping.clone(),
                direct_witness: CurrentGridDirectWitness::new(
                    7,
                    11,
                    GridAxisKind::Column,
                    local_span,
                    role,
                ),
                current_grid: 7,
                item: 11,
            },
        )
        .expect("collapsed-boundary baseline placement");

        assert_eq!(
            placement.translated_target(),
            if reversed { 83.0 } else { 7.0 },
            "{reversed:?} inherited baseline must observe the coincident interior gutter",
        );
        assert_eq!(inherited.geometry.total_extent(), 145.0);
    }
}

#[test]
fn fri08_c02_auto_fit_public_parent_decreasing_baseline_uses_collapsed_boundary_gutter() {
    let owner_group = AncestorBaselineGroup::reduce(
        1_u32,
        GridAxisKind::Column,
        PhysicalAxis::Horizontal,
        4,
        [inherited_placement_member(
            91,
            GridAxisKind::Column,
            AncestorBaselineRole::First,
            2,
            17.0,
        )],
    );
    let ancestor_groups = final_ancestor_baseline_groups_for_transport_test(
        AncestorBaselineGroup::reduce(
            1_u32,
            GridAxisKind::Row,
            PhysicalAxis::Vertical,
            1,
            Vec::<AncestorBaselineMember<u32>>::new(),
        ),
        owner_group,
    );
    let parent_style = NodeInput {
        display: Display::Grid,
        direction: Direction::Rtl,
        grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
        gap: Size::new(Length::px(10.0), Length::ZERO),
        ..NodeInput::DEFAULT
    };
    let child_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        grid_template_columns: vec![TrackComponent::px(40.0)],
        grid_template_rows: vec![empty_subgrid_track()],
        gap: Size::new(Length::px(20.0), Length::ZERO),
        ..NodeInput::DEFAULT
    };
    let parent_geometry = UsedGridAxisGeometryOf::new(
        vec![40.0, 0.0, 40.0, 40.0],
        vec![false, true, false, false],
        10.0,
    );
    let context = subgrid_child_parent_context_from_ancestor_groups_with_geometry(
        SubgridChildParentContextInput {
            item: SubgridItemReport {
                node: 7_u32,
                column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
                row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
            },
            child_style: &child_style,
            area: GridArea {
                column: 2,
                row: 0,
                column_end: 3,
                row_end: 1,
                size: LogicalSizeOf::new(40.0, 40.0),
            },
            content_box_size: Size::new(40.0, 40.0),
            columns: parent_geometry.sizes(),
            rows: &[40.0],
            gap: LogicalSizeOf::new(10.0, 0.0),
            parent_named_columns: &NamedGridLines::new(GridAxisKind::Column, 4),
            parent_named_rows: &NamedGridLines::new(GridAxisKind::Row, 1),
            parent_area_facts: None,
            parent_baseline_groups: &GridBaselineGroups {
                columns: vec![TrackBaselineGroup::default(); 4],
                rows: vec![TrackBaselineGroup::default()],
            },
            margin: Edges::all(Some(0.0)),
            border: Edges::ZERO,
            padding: Edges::ZERO,
        },
        &ancestor_groups,
        TemplateAreaExpandedAxes::default(),
        1_u32,
        Some(&parent_geometry),
        None,
    )
    .expect("ordinary auto-fit geometry remains inheritable");

    assert_eq!(
        context
            .rows
            .expect("inherited decreasing row")
            .major_baselines[0]
            .expect("transported first baseline")
            .coordinate(),
        17.0,
        "the adjacent collapsed gutter has no scalar subgrid-gap translation",
    );
}

#[test]
fn fri08_c01_placement_absolute_and_display_none_children_create_no_demand() {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                grid_auto_columns: vec![TrackComponent::px(50.0)],
                grid_auto_rows: vec![TrackComponent::px(50.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_line(100).expect("absolute control line"),
                grid_row: GridPlacement::try_line(100).expect("absolute control line"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::None,
                grid_column: GridPlacement::try_line(-100).expect("hidden control line"),
                grid_row: GridPlacement::try_line(-100).expect("hidden control line"),
                ..NodeInput::DEFAULT
            },
        )
        .style(4, NodeInput::DEFAULT);

    let batch = fri08_c01_placement_compute(&tree);

    assert_eq!(fri08_c01_placement_output(&batch, 1).size.height, 10.0);
    assert_eq!(
        fri08_c01_placement_output(&batch, 4).size,
        Size::new(10.0, 10.0)
    );
}

fn assert_fri06_mr02_geometry_error_grid_child<S: LayoutScalar>(display: Display) {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(7, [11])
        .children(11, [])
        .style(
            7,
            NodeInputOf {
                display,
                size: size.map(PreferredSizeOf::px),
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO],
                ..NodeInputOf::default()
            },
        )
        .style(11, NodeInputOf::default())
        .measure_when(
            11,
            crate::test_support::layout_tree::OracleMeasurementOf::new(
                ComputeOutputOf::from_sizes(
                    Size::new(S::from_f64(10.0), S::from_f64(10.0)),
                    Size::splat(S::INFINITY),
                ),
            )
            .run_mode(RunMode::PerformLayout),
        )
        .measure(
            11,
            ComputeOutputOf::from_outer_size(Size::new(S::from_f64(10.0), S::from_f64(10.0))),
        );
    let error = compute_grid(
        &mut tree,
        7,
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
        ),
    )
    .expect_err("invalid retained grid child geometry must fail");

    fri06_mr02_geometry_error_assert(
        error,
        LayoutErrorSiteOf::ContainerSubject {
            container: 7,
            subject: 11,
        },
        LayoutOperation::ChildLayout,
        LayoutInternalInvariant::InvalidBlockScrollGeometry,
    );
}

#[test]
fn fri06_mr02_geometry_error_grid_own_preserves_root_and_child_mapping_both_scalars() {
    assert_fri06_mr02_geometry_error_grid_own::<f32>();
    assert_fri06_mr02_geometry_error_grid_own::<f64>();
}

#[test]
fn fri06_mr02_geometry_error_grid_child_preserves_container_subject_both_scalars() {
    for display in [Display::Grid, Display::GridLanes] {
        assert_fri06_mr02_geometry_error_grid_child::<f32>(display);
        assert_fri06_mr02_geometry_error_grid_child::<f64>(display);
    }
}

fn used_overflow(x: Overflow, y: Overflow) -> crate::scroll::UsedOverflow {
    crate::scroll::UsedOverflow::from_computed(computed_overflow(x, y), false)
}

#[test]
fn fri05_c05_grid_alignment_uses_final_track_subject_in_all_flow_mappings() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let mut tree = OracleTree::new()
                .children(0, [1])
                .children(1, [])
                .style(
                    0,
                    NodeInput {
                        display: Display::Grid,
                        writing_mode,
                        direction,
                        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                        overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                        scrollbar_width: ScrollbarWidth::ZERO,
                        justify_content: Some(AlignContent::End),
                        align_content: Some(AlignContent::Center),
                        grid_template_columns: vec![TrackComponent::px(160.0)],
                        grid_template_rows: vec![TrackComponent::px(140.0)],
                        ..NodeInput::default()
                    },
                )
                .style(1, NodeInput::default());

            let output = compute_grid(
                &mut tree,
                0,
                fri05_c05_grid_sizing_input(Size::new(Some(100.0), Some(80.0))),
            )
            .expect("ordinary grid alignment geometry computes");
            let geometry = output
                .scroll_geometry
                .expect("alignment geometry is present");
            let flow_range = geometry
                .flow_axes()
                .flow_relative_scroll_range(geometry.physical_range());
            let logical_container = geometry.flow_axes().logical_size(Size::new(100.0, 80.0));
            let inline_start_extent = 160.0 - logical_container.inline;
            let block_extent = (140.0 - logical_container.block) / 2.0;
            assert_eq!(
                (flow_range.inline().minimum(), flow_range.inline().maximum()),
                (-inline_start_extent, 0.0),
                "{writing_mode:?} {direction:?} inline end subject"
            );
            assert_eq!(
                (flow_range.block().minimum(), flow_range.block().maximum()),
                (-block_extent, block_extent),
                "{writing_mode:?} {direction:?} block center subject"
            );
        }
    }
}

#[test]
fn fri05_c05_grid_alignment_start_center_end_safe_distributed_and_out_of_flow_are_bounded() {
    for (alignment, expected) in [
        (AlignContent::Start, (0.0, 60.0)),
        (AlignContent::End, (-60.0, 0.0)),
        (AlignContent::Center, (-30.0, 30.0)),
        (AlignContent::SafeEnd, (0.0, 60.0)),
        (AlignContent::SpaceBetween, (0.0, 60.0)),
    ] {
        let mut tree = OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                    overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                    scrollbar_width: ScrollbarWidth::ZERO,
                    justify_content: Some(alignment),
                    align_content: Some(alignment),
                    grid_template_columns: vec![TrackComponent::px(160.0)],
                    grid_template_rows: vec![TrackComponent::px(140.0)],
                    ..NodeInput::default()
                },
            )
            .style(1, NodeInput::default());
        let geometry = compute_grid(
            &mut tree,
            0,
            fri05_c05_grid_sizing_input(Size::new(Some(100.0), Some(80.0))),
        )
        .unwrap()
        .scroll_geometry
        .unwrap();
        let range = geometry
            .flow_axes()
            .flow_relative_scroll_range(geometry.physical_range());
        assert_eq!(
            (range.inline().minimum(), range.inline().maximum()),
            expected
        );
    }

    let mut out_of_flow = OracleTree::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                scrollbar_width: ScrollbarWidth::ZERO,
                justify_content: Some(AlignContent::End),
                grid_template_columns: vec![TrackComponent::px(160.0)],
                grid_template_rows: vec![TrackComponent::px(80.0)],
                ..NodeInput::default()
            },
        )
        .style(1, NodeInput::default())
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                inset: Edges {
                    left: LengthAuto::px(-100.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::default()
            },
        );
    let geometry = compute_grid(
        &mut out_of_flow,
        0,
        fri05_c05_grid_sizing_input(Size::new(Some(100.0), Some(80.0))),
    )
    .unwrap()
    .scroll_geometry
    .unwrap();
    let range = geometry
        .flow_axes()
        .flow_relative_scroll_range(geometry.physical_range());
    assert_eq!(range.inline().minimum(), -60.0);
}

#[test]
fn fri05_c05_grid_child_geometry_retains_in_flow_and_absolute_target_metadata() {
    for display in [Display::Grid, Display::GridLanes] {
        let size = Size::new(40.0, 30.0);
        let in_flow_margin = ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap();
        let absolute_margin = ScrollMargin::try_new(-5.0, 6.0, 7.0, -8.0).unwrap();
        let mut tree = OracleTree::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInput {
                    display,
                    size: size.map(PreferredSize::px),
                    grid_template_columns: vec![TrackComponent::px(40.0)],
                    grid_template_rows: vec![TrackComponent::px(30.0)],
                    ..NodeInput::default()
                },
            )
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(12.0), PreferredSize::px(9.0)),
                    grid_template_columns: vec![TrackComponent::Subgrid(SubgridTrack {
                        name_components: Vec::new(),
                    })],
                    grid_template_rows: vec![TrackComponent::px(9.0)],
                    justify_self: Some(AlignItems::Start),
                    align_self: Some(AlignItems::Start),
                    scroll_margin: in_flow_margin,
                    scroll_snap_align: ScrollSnapAlign::new(
                        ScrollSnapAlignValue::Start,
                        ScrollSnapAlignValue::End,
                    ),
                    scroll_snap_stop: ScrollSnapStop::Always,
                    ..NodeInput::default()
                },
            )
            .style(
                2,
                NodeInput {
                    display: Display::Block,
                    position: Position::Absolute,
                    size: Size::new(PreferredSize::px(8.0), PreferredSize::px(6.0)),
                    inset: Edges::new(
                        LengthAuto::px(3.0),
                        LengthAuto::AUTO,
                        LengthAuto::AUTO,
                        LengthAuto::px(5.0),
                    ),
                    scroll_margin: absolute_margin,
                    scroll_snap_align: ScrollSnapAlign::new(
                        ScrollSnapAlignValue::Center,
                        ScrollSnapAlignValue::Start,
                    ),
                    scroll_snap_stop: ScrollSnapStop::Always,
                    ..NodeInput::default()
                },
            );

        compute_grid(&mut tree, 0, fri05_c05_grid_sizing_input(size.map(Some)))
            .expect("grid child geometry case computes");

        for (node, source_index, expected_margin, expected_align) in [
            (
                1,
                0,
                in_flow_margin,
                ScrollSnapAlign::new(ScrollSnapAlignValue::Start, ScrollSnapAlignValue::End),
            ),
            (
                2,
                1,
                absolute_margin,
                ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::Start),
            ),
        ] {
            let child = tree.layout(node).expect("grid child output is staged");
            assert_eq!(child.source_index, SourceIndex::new(source_index));
            let geometry = child
                .scroll_geometry
                .expect("grid child retains canonical geometry");
            assert_eq!(geometry.border_box().size(), child.size);
            assert_eq!(geometry.target().border_box(), geometry.border_box());
            assert_eq!(geometry.target().scroll_margin(), expected_margin);
            assert_eq!(geometry.target().snap_align(), expected_align);
            assert_eq!(geometry.target().snap_stop(), ScrollSnapStop::Always);
        }
    }
}

fn fri05_c05_grid_positive_margin_bounds(output: NodeOutput) -> (Point<f32>, Point<f32>) {
    let minimum = Point::new(
        output.location.x - output.margin.left.max(0.0),
        output.location.y - output.margin.top.max(0.0),
    );
    let maximum = Point::new(
        output.location.x + output.size.width + output.margin.right.max(0.0),
        output.location.y + output.size.height + output.margin.bottom.max(0.0),
    );
    (minimum, maximum)
}

#[test]
fn fri05_c05_grid_contribution_container_origins_margins_terminal_padding_and_absolute_are_exact() {
    let padding = Edges::new(
        Length::px(7.0),
        Length::px(4.0),
        Length::px(3.0),
        Length::px(10.0),
    );
    let mut tree = OracleTree::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInput {
                display: Display::Grid,
                size: Size::ZERO.map(PreferredSize::px),
                padding,
                grid_template_columns: vec![TrackComponent::px(0.0)],
                grid_template_rows: vec![TrackComponent::px(0.0)],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(5.0), PreferredSize::px(6.0)),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                margin: Edges::new(
                    LengthAuto::px(2.0),
                    LengthAuto::px(4.0),
                    LengthAuto::px(3.0),
                    LengthAuto::px(1.0),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                position: Position::Relative,
                size: Size::new(PreferredSize::px(3.0), PreferredSize::px(4.0)),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                inset: Edges::new(
                    LengthAuto::px(-12.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(-15.0),
                ),
                margin: Edges::all(LengthAuto::px(-8.0)),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(PreferredSize::px(5.0), PreferredSize::px(5.0)),
                inset: Edges::new(
                    LengthAuto::px(2.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(30.0),
                ),
                margin: Edges {
                    right: LengthAuto::px(7.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        );

    let output = compute_grid(
        &mut tree,
        0,
        fri05_c05_grid_sizing_input(Size::splat(Some(0.0))),
    )
    .expect("grid origin and contribution case computes");
    let first = tree.layout(1).expect("first in-flow output");
    let second = tree.layout(2).expect("second in-flow output");
    let absolute = tree.layout(3).expect("absolute output");
    assert_eq!(first.source_index, SourceIndex::new(0));
    assert_eq!(second.source_index, SourceIndex::new(1));
    assert_eq!(absolute.source_index, SourceIndex::new(2));

    let bounds = [first, second, absolute].map(fri05_c05_grid_positive_margin_bounds);
    let minimum = bounds
        .iter()
        .fold(Point::<f32>::ZERO, |minimum, (origin, _)| {
            Point::new(minimum.x.min(origin.x), minimum.y.min(origin.y))
        });
    let mut maximum = bounds.iter().fold(Point::<f32>::ZERO, |maximum, (_, end)| {
        Point::new(maximum.x.max(end.x), maximum.y.max(end.y))
    });
    let in_flow_end = [first, second]
        .map(fri05_c05_grid_positive_margin_bounds)
        .iter()
        .fold(Point::<f32>::ZERO, |maximum, (_, end)| {
            Point::new(maximum.x.max(end.x), maximum.y.max(end.y))
        });
    maximum.x = maximum.x.max(in_flow_end.x + 4.0);
    maximum.y = maximum.y.max(in_flow_end.y + 3.0);

    assert!(first.location.x > 0.0 && first.location.y > 0.0);
    assert!(second.location.x < 0.0 && second.location.y < 0.0);
    assert_eq!(
        output.content_size,
        Size::new(maximum.x - minimum.x, maximum.y - minimum.y),
        "container-local locations, positive outsets, negative starts, terminal padding, and the current absolute child contribute without area-relative subtraction"
    );
}

#[test]
fn fri04_c04_grid_dispatch_nested_items_and_absolute_children_keep_actual_algorithm_and_site() {
    for (display, algorithm) in [
        (Display::Grid, SizingAlgorithm::Grid),
        (Display::InlineGrid, SizingAlgorithm::Grid),
        (Display::GridLanes, SizingAlgorithm::GridLanes),
        (Display::InlineGridLanes, SizingAlgorithm::GridLanes),
    ] {
        fri04_c04_grid_dispatch_assert_error(
            display,
            fri04_c04_grid_dispatch_style(
                Fri04C04GridSizingValue::Minimum(MinSize::STRETCH),
                PhysicalAxis::Horizontal,
            ),
            SizingProperty::Minimum,
            SizingBehavior::Stretch,
            algorithm,
            PhysicalAxis::Horizontal,
            1,
        );
        fri04_c04_grid_dispatch_assert_error(
            display,
            NodeInput {
                position: Position::Absolute,
                size: Size::new(PreferredSize::AUTO, PreferredSize::MAX_CONTENT),
                ..NodeInput::default()
            },
            SizingProperty::Preferred,
            SizingBehavior::MaxContent,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            1,
        );
    }
}

#[test]
fn ordinary_grid_replaced_normal_alignment_starts_while_explicit_stretch_remains_in_both_scalar_lanes()
 {
    assert_ordinary_grid_replaced_normal_alignment::<f32>();
    assert_ordinary_grid_replaced_normal_alignment::<f64>();
}

fn assert_ordinary_grid_replaced_normal_alignment<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let measured =
        ComputeOutputOf::from_outer_size(Size::new(S::from_f64(30.0), S::from_f64(20.0)));
    for (label, replaced, item_alignment, container_alignment, expected_known) in [
        ("replaced normal", true, None, None, Size::NONE),
        (
            "non-replaced normal",
            false,
            None,
            None,
            Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(80.0))),
        ),
        (
            "explicit item stretch",
            true,
            Some(AlignItems::Stretch),
            None,
            Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(80.0))),
        ),
        (
            "explicit container stretch",
            true,
            None,
            Some(AlignItems::Stretch),
            Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(80.0))),
        ),
    ] {
        let mut tree = OracleTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::px(S::from_f64(80.0)),
                    ),
                    grid_template_columns: vec![TrackComponentOf::px(S::from_f64(100.0))],
                    grid_template_rows: vec![TrackComponentOf::px(S::from_f64(80.0))],
                    justify_items: container_alignment,
                    align_items: container_alignment,
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    item_is_replaced: replaced,
                    justify_self: item_alignment,
                    align_self: item_alignment,
                    ..NodeInputOf::default()
                },
            )
            .measure(1, measured);

        compute_grid(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(80.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(
                    AvailableOf::definite(S::from_f64(100.0)),
                    AvailableOf::definite(S::from_f64(80.0)),
                ),
            ),
        )
        .expect("replaced ordinary-grid alignment succeeds");

        let layout = tree.layout(1).expect("grid child layout is staged");
        assert_eq!(layout.location, Point::ZERO, "{label} starts in its area");
        assert_eq!(layout.size, measured.size, "{label} keeps measured output");
        let layout_input = tree
            .inputs(1)
            .iter()
            .find(|input| input.run_mode() == RunMode::PerformLayout)
            .expect("grid child receives a perform-layout request");
        assert_eq!(
            layout_input.known(),
            expected_known,
            "{label} resolves normal alignment on both axes"
        );
    }
}

#[test]
fn grid_and_lanes_child_context_is_complete_for_layout_sizing_and_absolute_paths() {
    assert_grid_child_context_is_complete::<f32>();
    assert_grid_child_context_is_complete::<f64>();
}

fn assert_grid_child_context_is_complete<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let flow_axes = FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl);
    let expected =
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::Grid);

    for display in [Display::Grid, Display::GridLanes] {
        for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
            let mut tree = OracleTreeOf::<S>::new()
                .children(0, [1, 2])
                .children(1, [])
                .children(2, [])
                .style(
                    0,
                    NodeInputOf {
                        display,
                        writing_mode: WritingMode::SidewaysRl,
                        direction: Direction::Rtl,
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                        grid_template_columns: vec![TrackComponentOf::px(S::from_f64(160.0))],
                        grid_template_rows: vec![TrackComponentOf::px(S::from_f64(120.0))],
                        ..NodeInputOf::default()
                    },
                )
                .style(1, NodeInputOf::default())
                .style(
                    2,
                    NodeInputOf {
                        position: Position::Absolute,
                        size: Size::new(
                            PreferredSizeOf::px(S::from_f64(30.0)),
                            PreferredSizeOf::px(S::from_f64(12.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .measure(
                    1,
                    ComputeOutputOf::from_outer_size(Size::new(
                        S::from_f64(40.0),
                        S::from_f64(20.0),
                    )),
                )
                .measure(
                    2,
                    ComputeOutputOf::from_outer_size(Size::new(
                        S::from_f64(30.0),
                        S::from_f64(12.0),
                    )),
                );

            crate::compute_grid(
                &mut tree,
                0,
                ComputeInputOf::for_child(
                    run_mode,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    Size::new(Some(S::from_f64(300.0)), Some(S::from_f64(240.0))),
                    crate::ContainingLayoutContext::new(
                        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                        crate::ParentFormattingContext::NoParent,
                    ),
                    Size::splat(AvailableOf::definite(S::from_f64(300.0))),
                ),
            )
            .expect("grid context capture layout succeeds");

            let normal_inputs = tree.inputs(1);
            assert!(
                !normal_inputs.is_empty(),
                "{display:?} must request its in-flow child"
            );
            assert!(
                normal_inputs
                    .iter()
                    .all(|input| input.containing_layout_context() == expected),
                "every {display:?} in-flow request must use the parent axes and Grid role: {normal_inputs:#?}"
            );

            if run_mode == RunMode::ComputeSize {
                assert!(
                    normal_inputs
                        .iter()
                        .any(|input| input.run_mode() == RunMode::ComputeSize),
                    "{display:?} intrinsic sizing must request the child through the complete context"
                );
            } else {
                assert!(
                    normal_inputs
                        .iter()
                        .any(|input| input.run_mode() == RunMode::PerformLayout),
                    "{display:?} normal layout must request the child through the complete context"
                );
                let absolute_inputs = tree.inputs(2);
                assert!(
                    absolute_inputs
                        .iter()
                        .any(|input| input.run_mode() == RunMode::PerformLayout),
                    "{display:?} absolute scheduling must request the child"
                );
                assert!(
                    absolute_inputs
                        .iter()
                        .all(|input| input.containing_layout_context() == expected),
                    "every {display:?} absolute request must use the parent axes and Grid role: {absolute_inputs:#?}"
                );
            }
        }
    }
}
fn final_height(tree: &OracleTree, node: u32) -> Scalar {
    tree.final_layout(node)
        .expect("node should have a final layout")
        .size
        .height
}

#[test]
fn grid_lanes_display_uses_separate_placement_path_before_child_layout() {
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
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::GridLanes,
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

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

    assert_eq!(output.content_size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].scrollbar_size(), Size::new(10.0, 10.0));
}

#[test]
fn grid_lanes_reports_synthesized_container_baselines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(20.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 20.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(0.0));
}

fn assert_physical_baseline_grid_and_lanes_preserve_an_orthogonal_child_x<S: LayoutScalar>()
where
    lts::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    for display in [Display::Grid, Display::GridLanes] {
        let child_baselines = ComputeOutputOf::from_sizes_and_baselines(
            Size::new(S::from_f64(70.0), S::from_f64(20.0)),
            Size::new(S::from_f64(70.0), S::from_f64(20.0)),
            BaselinesOf {
                first: Point::new(Some(S::from_f64(7.0)), None),
                last: Point::new(Some(S::from_f64(11.0)), None),
            },
        );
        let mut tree = lts::layout_tree::OracleTreeOf::<S>::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInputOf {
                    display,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(120.0)),
                        PreferredSizeOf::px(S::from_f64(80.0)),
                    ),
                    grid_template_columns: vec![TrackComponentOf::px(S::from_f64(120.0))],
                    grid_template_rows: vec![TrackComponentOf::px(S::from_f64(80.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    writing_mode: WritingMode::VerticalRl,
                    align_self: Some(AlignItems::Start),
                    margin: Edges::new(
                        LengthAutoOf::px(S::from_f64(17.0)),
                        LengthAutoOf::px(S::from_f64(5.0)),
                        LengthAutoOf::px(S::from_f64(13.0)),
                        LengthAutoOf::px(S::from_f64(11.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .measure(2, child_baselines);

        let output = compute_grid(
            &mut tree,
            1,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(120.0)), Some(S::from_f64(80.0))),
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::new(
                    AvailableOf::definite(S::from_f64(120.0)),
                    AvailableOf::definite(S::from_f64(80.0)),
                ),
            ),
        )
        .expect("grid layout succeeds");
        let child = tree.layout(2).expect("grid child layout is staged");

        assert_eq!(
            child.location,
            Point::new(S::from_f64(11.0), S::from_f64(17.0))
        );
        assert_eq!(
            output.first_baselines,
            Point::new(Some(child.location.x + S::from_f64(7.0)), None)
        );
        assert_eq!(
            output.last_baselines,
            Point::new(Some(child.location.x + S::from_f64(11.0)), None)
        );
    }
}

#[test]
fn physical_baseline_grid_and_lanes_preserve_an_orthogonal_child_x_for_f32() {
    assert_physical_baseline_grid_and_lanes_preserve_an_orthogonal_child_x::<f32>();
}

#[test]
fn physical_baseline_grid_and_lanes_preserve_an_orthogonal_child_x_for_f64() {
    assert_physical_baseline_grid_and_lanes_preserve_an_orthogonal_child_x::<f64>();
}

fn physical_baseline_from_logical_block<S: LayoutScalar>(
    flow_axes: crate::geometry::FlowAxes,
    logical_coordinate: S,
    logical_size: LogicalSizeOf<S>,
) -> Point<Option<S>> {
    let coordinate = if flow_axes
        .logical_axis_progression(LogicalAxis::Block)
        .is_decreasing()
    {
        logical_size.block - logical_coordinate
    } else {
        logical_coordinate
    };
    match flow_axes.block_axis() {
        PhysicalAxis::Horizontal => Point::new(Some(coordinate), None),
        PhysicalAxis::Vertical => Point::new(None, Some(coordinate)),
    }
}

fn assert_logical_grid_lanes_axes_baselines<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    let logical_container_size = LogicalSizeOf::new(scalar(70.0), scalar(110.0));
    let logical_child_size = LogicalSizeOf::new(scalar(10.0), scalar(20.0));

    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
            let physical_container_size = flow_axes.physical_size(logical_container_size);
            let physical_child_size = flow_axes.physical_size(logical_child_size);
            let first_child = ComputeOutputOf::from_sizes_and_baselines(
                physical_child_size,
                physical_child_size,
                BaselinesOf {
                    first: physical_baseline_from_logical_block(
                        flow_axes,
                        scalar(13.0),
                        logical_child_size,
                    ),
                    last: physical_baseline_from_logical_block(
                        flow_axes,
                        scalar(7.0),
                        logical_child_size,
                    ),
                },
            );
            let last_child = ComputeOutputOf::from_sizes_and_baselines(
                physical_child_size,
                physical_child_size,
                BaselinesOf {
                    first: physical_baseline_from_logical_block(
                        flow_axes,
                        scalar(12.0),
                        logical_child_size,
                    ),
                    last: physical_baseline_from_logical_block(
                        flow_axes,
                        scalar(8.0),
                        logical_child_size,
                    ),
                },
            );

            for (grid_auto_flow, expected_last_block) in [
                (GridAutoFlow::Row, scalar(8.0)),
                (GridAutoFlow::Column, scalar(58.0)),
            ] {
                let mut tree = OracleTreeOf::<S>::new()
                    .children(1, [2, 3])
                    .children(2, [])
                    .children(3, [])
                    .style(
                        1,
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
                            grid_column: if grid_auto_flow == GridAutoFlow::Row {
                                GridPlacement::try_line(1).expect("test grid column is valid")
                            } else {
                                GridPlacement::AUTO
                            },
                            grid_row: if grid_auto_flow == GridAutoFlow::Column {
                                GridPlacement::try_line(1).expect("test grid row is valid")
                            } else {
                                GridPlacement::AUTO
                            },
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        3,
                        NodeInputOf {
                            writing_mode,
                            direction,
                            grid_column: if grid_auto_flow == GridAutoFlow::Row {
                                GridPlacement::try_line(2).expect("test grid column is valid")
                            } else {
                                GridPlacement::AUTO
                            },
                            grid_row: if grid_auto_flow == GridAutoFlow::Column {
                                GridPlacement::try_line(2).expect("test grid row is valid")
                            } else {
                                GridPlacement::AUTO
                            },
                            ..NodeInputOf::default()
                        },
                    )
                    .measure(2, first_child)
                    .measure(3, last_child);

                let output = crate::compute_grid(
                    &mut tree,
                    1,
                    ComputeInputOf::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        physical_container_size.map(Some),
                        crate::ContainingLayoutContext::new(
                            crate::geometry::FlowAxes::new(
                                WritingMode::HorizontalTb,
                                Direction::Ltr,
                            ),
                            crate::ParentFormattingContext::NoParent,
                        ),
                        physical_container_size.map(AvailableOf::definite),
                    ),
                )
                .expect("logical grid-lanes baseline layout succeeds");

                assert_eq!(
                    output.first_baselines,
                    physical_baseline_from_logical_block(
                        flow_axes,
                        scalar(13.0),
                        logical_container_size,
                    ),
                    "{writing_mode:?} {direction:?} {grid_auto_flow:?} must project the first lane baseline on the container block axis"
                );
                assert_eq!(
                    output.last_baselines,
                    physical_baseline_from_logical_block(
                        flow_axes,
                        expected_last_block,
                        logical_container_size,
                    ),
                    "{writing_mode:?} {direction:?} {grid_auto_flow:?} must project the last lane baseline on the container block axis"
                );
            }
        }
    }
}

#[test]
fn logical_grid_lanes_axes_baselines_f32() {
    assert_logical_grid_lanes_axes_baselines::<f32>();
}

#[test]
fn logical_grid_lanes_axes_baselines_f64() {
    assert_logical_grid_lanes_axes_baselines::<f64>();
}

fn assert_logical_ordinary_grid_in_flow_placement_baselines_and_extents<S: LayoutScalar>()
where
    OracleTreeOf<S>: Compute<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    let logical_container_size = LogicalSizeOf::new(scalar(70.0), scalar(110.0));
    let logical_child_size = LogicalSizeOf::new(scalar(10.0), scalar(20.0));

    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
            let physical_container_size = flow_axes.physical_size(logical_container_size);
            let physical_child_size = flow_axes.physical_size(logical_child_size);

            for (alignment, child_locations, shared_baseline) in [
                (AlignItems::Baseline, [scalar(6.0), S::ZERO], scalar(14.0)),
                (
                    AlignItems::LastBaseline,
                    [scalar(24.0), scalar(30.0)],
                    scalar(40.0),
                ),
            ] {
                let (first_baselines, last_baselines) = match alignment {
                    AlignItems::Baseline => ([scalar(8.0), scalar(14.0)], [None, None]),
                    AlignItems::LastBaseline => {
                        ([S::ZERO, S::ZERO], [Some(scalar(16.0)), Some(scalar(10.0))])
                    }
                    _ => unreachable!("the test only uses baseline item alignment"),
                };
                let child_outputs = [
                    ComputeOutputOf::from_sizes_and_baselines(
                        physical_child_size,
                        flow_axes.physical_size(LogicalSizeOf::new(scalar(10.0), scalar(20.0))),
                        BaselinesOf {
                            first: physical_baseline_from_logical_block(
                                flow_axes,
                                first_baselines[0],
                                logical_child_size,
                            ),
                            last: last_baselines[0].map_or(Point::NONE, |last| {
                                physical_baseline_from_logical_block(
                                    flow_axes,
                                    last,
                                    logical_child_size,
                                )
                            }),
                        },
                    ),
                    ComputeOutputOf::from_sizes_and_baselines(
                        physical_child_size,
                        flow_axes.physical_size(LogicalSizeOf::new(scalar(80.0), scalar(120.0))),
                        BaselinesOf {
                            first: physical_baseline_from_logical_block(
                                flow_axes,
                                first_baselines[1],
                                logical_child_size,
                            ),
                            last: last_baselines[1].map_or(Point::NONE, |last| {
                                physical_baseline_from_logical_block(
                                    flow_axes,
                                    last,
                                    logical_child_size,
                                )
                            }),
                        },
                    ),
                ];
                let mut tree = OracleTreeOf::<S>::new()
                    .children(1, [2, 3])
                    .children(2, [])
                    .children(3, [])
                    .style(
                        1,
                        NodeInputOf {
                            display: Display::Grid,
                            writing_mode,
                            direction,
                            size: physical_container_size.map(PreferredSizeOf::px),
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
                            align_items: Some(alignment),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        2,
                        NodeInputOf {
                            writing_mode,
                            direction,
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        3,
                        NodeInputOf {
                            writing_mode,
                            direction,
                            grid_column: GridPlacement::try_line(2)
                                .expect("test grid line is valid"),
                            ..NodeInputOf::default()
                        },
                    )
                    .measure(2, child_outputs[0])
                    .measure(3, child_outputs[1]);

                let output = crate::compute_grid(
                    &mut tree,
                    1,
                    ComputeInputOf::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        physical_container_size.map(Some),
                        crate::ContainingLayoutContext::new(
                            crate::geometry::FlowAxes::new(
                                WritingMode::HorizontalTb,
                                Direction::Ltr,
                            ),
                            crate::ParentFormattingContext::NoParent,
                        ),
                        physical_container_size.map(AvailableOf::definite),
                    ),
                )
                .expect("logical ordinary-grid baseline layout succeeds");

                for (index, (inline, block)) in [
                    (S::ZERO, child_locations[0]),
                    (scalar(30.0), child_locations[1]),
                ]
                .into_iter()
                .enumerate()
                {
                    assert_eq!(
                        tree.layout(index as u32 + 2)
                            .expect("grid child layout is staged")
                            .location,
                        flow_axes.physical_point(
                            crate::geometry::LogicalPointOf::new(inline, block),
                            logical_child_size,
                            physical_container_size,
                        ),
                        "{writing_mode:?} {direction:?} {alignment:?} child {} must align on the container block axis",
                        index + 1
                    );
                }
                let (minimum, maximum) = [2, 3].into_iter().fold(
                    (Point::<S>::ZERO, Point::<S>::ZERO),
                    |(mut minimum, mut maximum): (Point<S>, Point<S>), node| {
                        let child = tree.layout(node).expect("grid child layout is staged");
                        let geometry = child
                            .scroll_geometry
                            .expect("grid child retains canonical geometry");
                        let border = geometry.border_box();
                        let border_origin = Point::new(
                            child.location.x + border.origin().x,
                            child.location.y + border.origin().y,
                        );
                        minimum.x = minimum.x.min(border_origin.x);
                        minimum.y = minimum.y.min(border_origin.y);
                        maximum.x = maximum.x.max(border_origin.x + border.size().width);
                        maximum.y = maximum.y.max(border_origin.y + border.size().height);
                        let descendants = geometry.propagatable_descendant_intervals();
                        if let Some(interval) = descendants.at(PhysicalAxis::Horizontal) {
                            minimum.x = minimum.x.min(child.location.x + interval.minimum());
                            maximum.x = maximum.x.max(child.location.x + interval.maximum());
                        }
                        if let Some(interval) = descendants.at(PhysicalAxis::Vertical) {
                            minimum.y = minimum.y.min(child.location.y + interval.minimum());
                            maximum.y = maximum.y.max(child.location.y + interval.maximum());
                        }
                        (minimum, maximum)
                    },
                );
                let legacy_visible_extent = Size::new(
                    physical_container_size.width.max(maximum.x - minimum.x),
                    physical_container_size.height.max(maximum.y - minimum.y),
                );
                let canonical_visible_extent = output
                    .scroll_geometry
                    .expect("performed ordinary grid publishes geometry")
                    .canonical_content_size()
                    .expect("ordinary grid canonical content extent is valid");
                let expected_visible_extent = legacy_visible_extent.max(canonical_visible_extent);
                let expected_baseline = physical_baseline_from_logical_block(
                    flow_axes,
                    shared_baseline,
                    logical_container_size,
                );
                let actual_baseline = match alignment {
                    AlignItems::Baseline => output.first_baselines,
                    AlignItems::LastBaseline => output.last_baselines,
                    _ => unreachable!("the test only uses baseline item alignment"),
                };

                assert_eq!(
                    actual_baseline, expected_baseline,
                    "{writing_mode:?} {direction:?} {alignment:?} must publish its baseline on the container block axis"
                );
                assert_eq!(
                    output.content_size, expected_visible_extent,
                    "{writing_mode:?} {direction:?} must retain visible content extents physically after projection"
                );
            }
        }
    }
}

#[test]
fn logical_ordinary_grid_in_flow_placement_baselines_and_extents_f32() {
    assert_logical_ordinary_grid_in_flow_placement_baselines_and_extents::<f32>();
}

#[test]
fn logical_ordinary_grid_in_flow_placement_baselines_and_extents_f64() {
    assert_logical_ordinary_grid_in_flow_placement_baselines_and_extents::<f64>();
}

#[test]
fn grid_lanes_does_not_apply_lane_axis_baseline_offsets() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(20.0)],
                grid_template_rows: vec![TrackComponent::px(0.0)],
                gap: Size::new(Length::ZERO, Length::px(5.0)),
                align_items: Some(AlignItems::Baseline),
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(20.0, 10.0, Some(2.0), None))
        .measure(3, baseline_measure(20.0, 15.0, Some(12.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 0.0);
    assert_eq!(final_y(&tree, 3), 15.0);
}

#[test]
fn grid_lanes_reports_last_baseline_from_spanning_item_end_edge() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(6.0)))
        .measure(3, baseline_measure(30.0, 80.0, None, Some(8.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.last_baselines.y, Some(72.0));
}

#[test]
fn both_axis_subgrid_zero_gap_auto_placement_advances_fully_auto_children() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::ZERO, Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                grid_row: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::default())
        .style(4, NodeInput::default());

    compute_root(
        &mut tree,
        1,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 1).unwrap();

    assert_eq!(
        tree.final_layout(3)
            .expect("first subgrid child should be laid out")
            .location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(4)
            .expect("second subgrid child should be laid out")
            .location,
        Point::new(40.0, 0.0)
    );
}

#[test]
fn subgrid_line_names_place_child_with_inherited_parent_names() {
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
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(2, 5).expect("valid grid lines"),
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
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("subgrid child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn grid_block_child_with_subgrid_tracks_lays_out_as_block_child() {
    assert_non_grid_child_with_subgrid_tracks_lays_out_as_ordinary_child(Display::Block);
}

#[test]
fn grid_flex_child_with_subgrid_tracks_lays_out_as_flex_child() {
    assert_non_grid_child_with_subgrid_tracks_lays_out_as_ordinary_child(Display::Flex);
}

fn assert_non_grid_child_with_subgrid_tracks_lays_out_as_ordinary_child(display: Display) {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(
            &mut self,
            node: u32,
            input: ComputeInput,
        ) -> LayoutResultOf<u32, ComputeOutput, Scalar> {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return Ok(ComputeOutput::from_outer_size(Size::new(30.0, 12.0)));
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
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![3]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display,
            grid_template_columns: vec![empty_subgrid_track()],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(3, NodeInput::default());

    crate::compute_grid(
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

    assert_eq!(tree.layouts[&2].size.width, 40.0);
    assert_eq!(tree.layouts[&3].size, Size::new(30.0, 12.0));
}

#[test]
fn grid_absolute_child_with_subgrid_tracks_does_not_participate_as_subgrid() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Grid,
            position: Position::Absolute,
            grid_template_columns: vec![empty_subgrid_track()],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(10.0, 10.0)));

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

    assert_eq!(output.content_size, Size::new(40.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(10.0, 10.0)
    );
}

#[test]
fn row_subgrid_child_inherits_parent_baseline_group() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4, 5, 6, 7])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(
            5,
            NodeInput {
                display: Display::None,
                ..NodeInput::default()
            },
        )
        .style(
            6,
            NodeInput {
                display: Display::None,
                ..NodeInput::default()
            },
        )
        .style(
            7,
            NodeInput {
                display: Display::None,
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(14.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(8.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 4), 6.0);
}

fn assert_real_row_subgrid_baseline_projection<S: LayoutScalar>(writing_mode: WritingMode)
where
    OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let flow_axes = FlowAxes::new(writing_mode, Direction::Ltr);
    let root_size =
        flow_axes.physical_size(LogicalSizeOf::new(S::from_f64(120.0), S::from_f64(80.0)));
    let physical_baseline = |coordinate: S| match flow_axes.block_axis() {
        PhysicalAxis::Horizontal => Point::new(Some(coordinate), None),
        PhysicalAxis::Vertical => Point::new(None, Some(coordinate)),
    };
    let baseline_output = |coordinate: S| {
        ComputeOutputOf::from_sizes_and_baselines(
            flow_axes.physical_size(LogicalSizeOf::new(S::from_f64(30.0), S::from_f64(20.0))),
            flow_axes.physical_size(LogicalSizeOf::new(S::from_f64(30.0), S::from_f64(20.0))),
            BaselinesOf {
                first: physical_baseline(coordinate),
                last: physical_baseline(coordinate),
            },
        )
    };

    for alignment in [AlignItems::Baseline, AlignItems::LastBaseline] {
        let mut tree = OracleTreeOf::<S>::new()
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [4])
            .children(4, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    size: root_size.map(PreferredSizeOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(S::from_f64(60.0)),
                        TrackComponentOf::px(S::from_f64(60.0)),
                    ],
                    grid_template_rows: vec![TrackComponentOf::px(S::from_f64(80.0))],
                    align_items: Some(alignment),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    writing_mode,
                    align_self: Some(alignment),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                    grid_template_columns: vec![TrackComponentOf::px(S::from_f64(60.0))],
                    grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack {
                        name_components: Vec::new(),
                    })],
                    ..NodeInputOf::default()
                },
            )
            .style(
                4,
                NodeInputOf {
                    writing_mode,
                    align_self: Some(alignment),
                    ..NodeInputOf::default()
                },
            )
            .measure(2, baseline_output(S::from_f64(14.0)))
            .measure(4, baseline_output(S::from_f64(8.0)));

        let output = compute_grid(
            &mut tree,
            1,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                root_size.map(Some),
                crate::ContainingLayoutContext::new(
                    flow_axes,
                    crate::ParentFormattingContext::NoParent,
                ),
                root_size.map(AvailableOf::definite),
            ),
        )
        .expect("nested row subgrid baseline layout succeeds");
        let root_baseline = match alignment {
            AlignItems::Baseline => output.first_baselines,
            AlignItems::LastBaseline => output.last_baselines,
            _ => unreachable!("the test only uses baseline alignments"),
        };
        let root_baseline_coordinate = if writing_mode == WritingMode::VerticalRl {
            if alignment == AlignItems::Baseline {
                S::from_f64(68.0)
            } else {
                S::from_f64(14.0)
            }
        } else if alignment == AlignItems::Baseline {
            S::from_f64(14.0)
        } else {
            S::from_f64(68.0)
        };
        assert_eq!(
            root_baseline,
            physical_baseline(root_baseline_coordinate),
            "{writing_mode:?} {alignment:?}"
        );
        let descendant = tree.layout(4).expect("nested descendant was laid out");
        let sibling = tree.layout(2).expect("baseline sibling was laid out");
        let (descendant_block_offset, sibling_block_offset) =
            if writing_mode == WritingMode::VerticalRl {
                if alignment == AlignItems::Baseline {
                    (S::from_f64(60.0), S::from_f64(54.0))
                } else {
                    (S::from_f64(6.0), S::ZERO)
                }
            } else if alignment == AlignItems::Baseline {
                (S::from_f64(6.0), S::ZERO)
            } else {
                (S::from_f64(60.0), S::from_f64(54.0))
            };
        match flow_axes.block_axis() {
            PhysicalAxis::Horizontal => {
                assert_eq!(
                    descendant.location.x, descendant_block_offset,
                    "descendant={descendant:?} sibling={sibling:?} root={root_baseline:?}"
                );
                assert_eq!(sibling.location.x, sibling_block_offset);
            }
            PhysicalAxis::Vertical => {
                assert_eq!(
                    descendant.location.y, descendant_block_offset,
                    "descendant={descendant:?} sibling={sibling:?} root={root_baseline:?}"
                );
                assert_eq!(sibling.location.y, sibling_block_offset);
            }
        }
    }
}

#[test]
fn real_row_subgrid_baseline_projection_f32() {
    assert_real_row_subgrid_baseline_projection::<f32>(WritingMode::HorizontalTb);
    assert_real_row_subgrid_baseline_projection::<f32>(WritingMode::VerticalRl);
}

#[test]
fn real_row_subgrid_baseline_projection_f64() {
    assert_real_row_subgrid_baseline_projection::<f64>(WritingMode::HorizontalTb);
    assert_real_row_subgrid_baseline_projection::<f64>(WritingMode::VerticalRl);
}

#[test]
fn orthogonal_baseline_subgrid_does_not_group_incompatible_physical_axes() {
    let vertical_child_baselines = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        Baselines {
            first: Point::new(Some(15.0), None),
            last: Point::new(Some(21.0), None),
        },
    );
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(80.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(45.0), None))
        .measure(4, vertical_child_baselines);

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn row_subgrid_inherited_baseline_accounts_for_margin_border_padding() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                border: Edges {
                    top: Length::px(2.0),
                    ..Edges::all(Length::ZERO)
                },
                padding: Edges {
                    top: Length::px(5.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(14.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(8.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 4), 7.0);
}

#[test]
fn row_subgrid_publishes_descendant_baseline_to_parent_row() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(17.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 49.0);
}

#[test]
fn row_subgrid_without_descendant_baseline_leaves_ancestor_group_unchanged() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(4, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(14.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(20.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 0.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn sibling_row_subgrids_revisit_inherited_published_baselines() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [4])
        .children(3, [5])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(4, baseline_measure(30.0, 20.0, Some(30.0), None))
        .measure(5, baseline_measure(30.0, 20.0, Some(8.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 5), 62.0);
}

#[test]
fn fri06_c12_t08_fully_inherited_baseline_root_stays_out_of_ancestor_group() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(14.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(8.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!((final_y(&tree, 2), final_y(&tree, 4)), (0.0, 40.0));
}

#[test]
fn fri06_c12_t08_second_inherited_row_keeps_direct_descendant_in_ancestor_baseline() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4, 5])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("second inherited row placement"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(9.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(30.0), None))
        .measure(5, baseline_measure(30.0, 20.0, Some(50.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(
        (
            final_y(&tree, 2),
            final_y(&tree, 3),
            final_y(&tree, 4),
            final_y(&tree, 5)
        ),
        (21.0, 0.0, 0.0, 40.0),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_maps_row_and_column_half_gaps() {
    for axis in [GridAxisKind::Row, GridAxisKind::Column] {
        let first_group = inherited_placement_group(axis, AncestorBaselineRole::First, 1, 17.0);
        let first = derive_inherited_placement(
            &first_group,
            axis,
            AncestorBaselineRole::First,
            1,
            false,
            10.0,
            20.0,
        )
        .unwrap();
        assert_eq!(
            (
                first.selected_local_track(),
                first.frame_translation(),
                first.accumulated_gutter_translation(),
                first.translated_target(),
            ),
            (1, 0.0, 5.0, 22.0),
        );

        let last_group = inherited_placement_group(axis, AncestorBaselineRole::Last, 2, 17.0);
        let last = derive_inherited_placement(
            &last_group,
            axis,
            AncestorBaselineRole::Last,
            2,
            false,
            10.0,
            20.0,
        )
        .unwrap();
        assert_eq!(
            (
                last.frame_translation(),
                last.accumulated_gutter_translation(),
                last.translated_target()
            ),
            (0.0, -5.0, 12.0),
        );
    }
}

#[test]
fn inherited_current_grid_baseline_placement_maps_first_and_last_edges_through_reversal() {
    let first_group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 2, 17.0);
    let first = derive_inherited_placement(
        &first_group,
        GridAxisKind::Row,
        AncestorBaselineRole::First,
        1,
        true,
        10.0,
        20.0,
    )
    .unwrap();
    assert_eq!(
        (
            first.frame_translation(),
            first.accumulated_gutter_translation(),
            first.translated_target()
        ),
        (0.0, -5.0, 12.0),
    );

    let last_group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::Last, 1, 17.0);
    let last = derive_inherited_placement(
        &last_group,
        GridAxisKind::Row,
        AncestorBaselineRole::Last,
        2,
        true,
        10.0,
        20.0,
    )
    .unwrap();
    assert_eq!(
        (
            last.frame_translation(),
            last.accumulated_gutter_translation(),
            last.translated_target()
        ),
        (0.0, 5.0, 22.0),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_is_zero_at_role_terminal_edges() {
    let first_group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 0, 17.0);
    let first = derive_inherited_placement(
        &first_group,
        GridAxisKind::Row,
        AncestorBaselineRole::First,
        0,
        false,
        10.0,
        20.0,
    )
    .unwrap();
    assert_eq!(first.accumulated_gutter_translation(), 0.0);

    let last_group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::Last, 3, 17.0);
    let last = derive_inherited_placement(
        &last_group,
        GridAxisKind::Row,
        AncestorBaselineRole::Last,
        3,
        false,
        10.0,
        20.0,
    )
    .unwrap();
    assert_eq!(last.accumulated_gutter_translation(), 0.0);
}

#[test]
fn inherited_current_grid_baseline_placement_uses_typed_owner_and_current_identity() {
    let group = inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let equal = derive_inherited_placement(
        &group,
        GridAxisKind::Row,
        AncestorBaselineRole::First,
        1,
        false,
        10.0,
        10.0,
    )
    .unwrap();
    assert_eq!(
        (
            equal.frame_translation(),
            equal.accumulated_gutter_translation(),
            equal.translated_target(),
        ),
        (0.0, 0.0, 17.0)
    );

    let owner_direct = InheritedCurrentGridBaselinePlacement::try_derive(
        &group,
        InheritedCurrentGridBaselinePlacementInput {
            axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            mapping: CheckedOwnerToCurrentPlacementMap::identity(
                1_u32,
                GridAxisKind::Row,
                PhysicalAxis::Vertical,
                PhysicalProgression::Increasing,
                4,
            ),
            direct_witness: CurrentGridDirectWitness::new(
                1,
                11,
                GridAxisKind::Row,
                GridTrackSpan::new(1, 2),
                AncestorBaselineRole::First,
            ),
            current_grid: 1,
            item: 11,
        },
    )
    .unwrap();
    assert_eq!(
        (
            owner_direct.frame_translation(),
            owner_direct.accumulated_gutter_translation(),
            owner_direct.translated_target()
        ),
        (0.0, 0.0, 17.0),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_keeps_mbp_in_base_mapping() {
    let group = inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let mapping = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        4,
    )
    .compose(owner_placement_boundary!(
        1,
        7,
        GridTrackSpan::new(0, 4),
        false,
        PhysicalProgression::Increasing,
        PhysicalProgression::Increasing,
        &[0.0; 4],
        &[0.0; 4],
        &[-3.0; 4],
        &[4.0; 4],
        10.0,
        20.0,
        3.0,
        4.0,
    ))
    .unwrap();
    let placement = InheritedCurrentGridBaselinePlacement::try_derive(
        &group,
        InheritedCurrentGridBaselinePlacementInput {
            axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            mapping,
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
    assert_eq!(
        (
            placement.immutable_owner_target(),
            placement.frame_translation(),
            placement.accumulated_gutter_translation(),
            placement.translated_target(),
        ),
        (17.0, -3.0, 5.0, 19.0),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_repeat_is_identical_and_mutates_no_input() {
    let group =
        inherited_placement_group(GridAxisKind::Column, AncestorBaselineRole::First, 1, 17.0);
    let mapping = inherited_placement_mapping(
        GridAxisKind::Column,
        false,
        GridTrackSpan::new(0, 4),
        10.0,
        20.0,
    );
    let witness = inherited_placement_witness(GridAxisKind::Column, AncestorBaselineRole::First, 1);
    let group_before = group.clone();
    let mapping_before = mapping.clone();
    let witness_before = witness;
    let input = || InheritedCurrentGridBaselinePlacementInput {
        axis: GridAxisKind::Column,
        physical_axis: PhysicalAxis::Horizontal,
        mapping: mapping.clone(),
        direct_witness: witness,
        current_grid: 7,
        item: 11,
    };
    let first = InheritedCurrentGridBaselinePlacement::try_derive(&group, input()).unwrap();
    let second = InheritedCurrentGridBaselinePlacement::try_derive(&group, input()).unwrap();
    assert_eq!(first, second);
    assert_eq!(group, group_before);
    assert_eq!(mapping, mapping_before);
    assert_eq!(witness, witness_before);
}

fn inherited_placement_input(
    axis: GridAxisKind,
    role: AncestorBaselineRole,
) -> InheritedCurrentGridBaselinePlacementInput<u32, f32> {
    InheritedCurrentGridBaselinePlacementInput {
        axis,
        physical_axis: match axis {
            GridAxisKind::Column => PhysicalAxis::Horizontal,
            GridAxisKind::Row => PhysicalAxis::Vertical,
        },
        mapping: inherited_placement_mapping(axis, false, GridTrackSpan::new(0, 4), 10.0, 20.0),
        direct_witness: inherited_placement_witness(axis, role, 1),
        current_grid: 7,
        item: 11,
    }
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_axis_mismatch_first() {
    let group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, f32::NAN);
    let input = inherited_placement_input(GridAxisKind::Column, AncestorBaselineRole::First);
    assert_eq!(
        InheritedCurrentGridBaselinePlacement::try_derive(&group, input),
        Err(InheritedCurrentGridBaselinePlacementError::AxisMismatch),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_physical_axis_mismatch() {
    let group = inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let mut input = inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::First);
    input.physical_axis = PhysicalAxis::Horizontal;
    assert_eq!(
        InheritedCurrentGridBaselinePlacement::try_derive(&group, input),
        Err(InheritedCurrentGridBaselinePlacementError::PhysicalAxisMismatch),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_span_out_of_range() {
    let group = inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let mut input = inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::First);
    input.mapping = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        5,
    );
    assert_eq!(
        InheritedCurrentGridBaselinePlacement::try_derive(&group, input),
        Err(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_selected_track_out_of_range() {
    let group = inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let mut input = inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::First);
    input
        .direct_witness
        .set_local_span_for_test(GridTrackSpan::new(4, 5));
    assert_eq!(
        InheritedCurrentGridBaselinePlacement::try_derive(&group, input),
        Err(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_role_target_mismatch() {
    let group = inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let input = inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::Last);
    assert_eq!(
        InheritedCurrentGridBaselinePlacement::try_derive(&group, input),
        Err(InheritedCurrentGridBaselinePlacementError::RoleTargetMismatch),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_ownership_mismatch() {
    let group = inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let mut input = inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::First);
    input.item = 12;
    assert_eq!(
        InheritedCurrentGridBaselinePlacement::try_derive(&group, input),
        Err(InheritedCurrentGridBaselinePlacementError::OwnershipMismatch),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_unusable_inherited_mapping() {
    let group = inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let mut input = inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::First);
    input.mapping = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Column,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        4,
    );
    input.direct_witness = CurrentGridDirectWitness::new(
        1,
        11,
        GridAxisKind::Row,
        GridTrackSpan::new(1, 2),
        AncestorBaselineRole::First,
    );
    input.current_grid = 1;
    assert_eq!(
        InheritedCurrentGridBaselinePlacement::try_derive(&group, input),
        Err(InheritedCurrentGridBaselinePlacementError::UnusableInheritedMapping),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_non_finite_last() {
    let group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::Last, 1, f32::NAN);
    let input = inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::Last);
    assert_eq!(
        InheritedCurrentGridBaselinePlacement::try_derive(&group, input),
        Err(InheritedCurrentGridBaselinePlacementError::NonFinite),
    );
}

#[test]
fn subgrid_baseline_placement_error_propagates_with_node_site() {
    let error = subgrid_child_context_error::<_, f32, ()>(
        11_u32,
        SubgridChildContextError::BaselineInheritance(SubgridBaselineInheritanceError::Placement(
            InheritedCurrentGridBaselinePlacementError::RoleTargetMismatch,
        )),
    );
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(11));
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    assert_eq!(
        error.kind(),
        &LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridBaselineInheritance),
    );
}

#[test]
fn subgrid_baseline_placement_error_propagates_with_container_subject_site() {
    let error = subgrid_child_context_container_error::<_, f32, ()>(
        7_u32,
        11_u32,
        SubgridChildContextError::BaselineInheritance(SubgridBaselineInheritanceError::Placement(
            InheritedCurrentGridBaselinePlacementError::RoleTargetMismatch,
        )),
    );
    assert_eq!(
        error.site(),
        LayoutErrorSiteOf::ContainerSubject {
            container: 7,
            subject: 11,
        },
    );
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    assert_eq!(
        error.kind(),
        &LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridBaselineInheritance),
    );
}

#[test]
fn late_subgrid_baseline_placement_error_after_prior_item_preparation_mutates_no_item_output_or_batch()
 {
    let first_group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let missing_last_group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let prepared = prepare_inherited_current_grid_baseline_placements([
        (
            &first_group,
            inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::First),
        ),
        (
            &missing_last_group,
            inherited_placement_input(GridAxisKind::Row, AncestorBaselineRole::Last),
        ),
    ]);
    assert_eq!(
        prepared,
        Err(InheritedCurrentGridBaselinePlacementError::RoleTargetMismatch),
    );
}

#[test]
fn inherited_current_grid_baseline_placement_rejects_non_finite_gaps_before_arithmetic_and_preserves_precedence()
 {
    for (parent_gap, current_gap) in [(f32::NAN, 20.0), (10.0, f32::INFINITY)] {
        let identity = CheckedOwnerToCurrentPlacementMap::identity(
            1_u32,
            GridAxisKind::Row,
            PhysicalAxis::Vertical,
            PhysicalProgression::Increasing,
            4,
        );
        let input = owner_placement_boundary!(
            1,
            7,
            GridTrackSpan::new(0, 4),
            false,
            PhysicalProgression::Increasing,
            PhysicalProgression::Increasing,
            &[0.0; 4],
            &[0.0; 4],
            &[0.0; 4],
            &[0.0; 4],
            parent_gap,
            current_gap,
            0.0,
            0.0,
        );
        assert_eq!(
            identity.compose(input),
            Err(InheritedCurrentGridBaselinePlacementError::NonFinite),
        );
    }

    let identity = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        4,
    );
    let ownership_first = owner_placement_boundary!(
        9,
        7,
        GridTrackSpan::new(0, 4),
        false,
        PhysicalProgression::Increasing,
        PhysicalProgression::Increasing,
        &[0.0; 4],
        &[0.0; 4],
        &[0.0; 4],
        &[0.0; 4],
        f32::NAN,
        f32::INFINITY,
        0.0,
        0.0,
    );
    assert_eq!(
        identity.compose(ownership_first),
        Err(InheritedCurrentGridBaselinePlacementError::OwnershipMismatch),
    );

    let mut mapping_first = owner_placement_boundary!(
        1,
        7,
        GridTrackSpan::new(0, 4),
        false,
        PhysicalProgression::Increasing,
        PhysicalProgression::Increasing,
        &[0.0; 4],
        &[0.0; 4],
        &[0.0; 4],
        &[0.0; 4],
        f32::NAN,
        f32::INFINITY,
        0.0,
        0.0,
    );
    mapping_first.inherited = false;
    assert_eq!(
        identity.compose(mapping_first),
        Err(InheritedCurrentGridBaselinePlacementError::UnusableInheritedMapping),
    );
}

fn fri06_c12_t08_reversed_inherited_column_baselines(alignment: AlignItems) -> [(f32, f32); 3] {
    let mut tree = OracleTree::new()
        .children(1, [2, 5])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(100.0), TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::px(80.0)],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                direction: Direction::Rtl,
                grid_column: GridPlacement::try_lines(1, 3).expect("valid inherited column span"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_column: GridPlacement::try_line(1).expect("valid first local column"),
                grid_row: GridPlacement::try_line(1).expect("valid first local row"),
                justify_self: Some(alignment),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_column: GridPlacement::try_line(1).expect("valid first local column"),
                grid_row: GridPlacement::try_line(2).expect("valid second local row"),
                justify_self: Some(alignment),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_column: GridPlacement::try_line(2).expect("valid ancestor column"),
                justify_self: Some(alignment),
                ..NodeInput::default()
            },
        );
    let (first, second, ancestor) = match alignment {
        AlignItems::Baseline => (
            vertical_baseline_measure(30.0, 20.0, Some(24.0), None),
            vertical_baseline_measure(50.0, 20.0, Some(9.0), None),
            vertical_baseline_measure(100.0, 20.0, Some(45.0), None),
        ),
        AlignItems::LastBaseline => (
            vertical_baseline_measure(30.0, 20.0, None, Some(7.0)),
            vertical_baseline_measure(50.0, 20.0, None, Some(18.0)),
            vertical_baseline_measure(100.0, 20.0, None, Some(40.0)),
        ),
        _ => unreachable!("the reversed-column control uses baseline alignment"),
    };
    tree = tree
        .measure(3, first)
        .measure(4, second)
        .measure(5, ancestor);

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(200.0), Available::Definite(80.0)),
    )
    .expect("reversed inherited-column layout computes");
    round_layout(&mut tree, 1).expect("reversed inherited-column layout rounds");

    let subgrid_x = tree
        .final_layout(2)
        .expect("reversed inherited-column grid is laid out")
        .location
        .x;
    [3_u32, 4, 5].map(|node| {
        let layout = tree
            .final_layout(node)
            .expect("reversed-column member is laid out");
        let baseline = match (alignment, node) {
            (AlignItems::Baseline, 3) => 24.0,
            (AlignItems::Baseline, 4) => 9.0,
            (AlignItems::Baseline, 5) => 45.0,
            (AlignItems::LastBaseline, 3) => 23.0,
            (AlignItems::LastBaseline, 4) => 32.0,
            (AlignItems::LastBaseline, 5) => 60.0,
            _ => unreachable!("the reversed-column control uses baseline alignment"),
        };
        let parent_x = if node == 5 { 0.0 } else { subgrid_x };
        (
            parent_x + layout.location.x,
            parent_x + layout.location.x + baseline,
        )
    })
}

#[test]
fn fri06_c12_t08_reversed_inherited_columns_preserve_first_baseline_target() {
    let members = fri06_c12_t08_reversed_inherited_column_baselines(AlignItems::Baseline);
    assert_eq!(members.map(|member| member.1), [145.0; 3], "{members:?}");
}

#[test]
fn fri06_c12_t08_reversed_inherited_columns_preserve_last_baseline_target() {
    let members = fri06_c12_t08_reversed_inherited_column_baselines(AlignItems::LastBaseline);
    assert_eq!(members.map(|member| member.1), [160.0; 3], "{members:?}");
}

fn fri06_c12_t08_parent_baseline_row(
    direct_baseline: f32,
    nested_baseline: f32,
    row_track: TrackComponent,
    alignment: AlignItems,
) -> f32 {
    let baseline_measurement = |baseline| match alignment {
        AlignItems::Baseline => baseline_measure(30.0, 20.0, Some(baseline), None),
        AlignItems::LastBaseline => baseline_measure(30.0, 20.0, None, Some(baseline)),
        _ => unreachable!("the intrinsic baseline helper uses a baseline alignment"),
    };
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![row_track],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                align_self: Some(alignment),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                align_self: Some(alignment),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measurement(direct_baseline))
        .measure(4, baseline_measurement(nested_baseline));

    let output = compute_grid(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .expect("intrinsic baseline subgrid layout succeeds");

    output.size.height
}

#[test]
fn fri06_c12_t08_parent_baseline_envelope_grows_an_intrinsic_row() {
    assert_eq!(
        fri06_c12_t08_parent_baseline_row(14.0, 8.0, TrackComponent::AUTO, AlignItems::Baseline,),
        26.0,
    );
}

#[test]
fn fri06_c12_t08_equal_parent_and_nested_baselines_need_no_intrinsic_shim() {
    assert_eq!(
        fri06_c12_t08_parent_baseline_row(14.0, 14.0, TrackComponent::AUTO, AlignItems::Baseline,),
        20.0,
    );
}

#[test]
fn fri06_c12_t08_fixed_parent_row_ignores_intrinsic_baseline_shim() {
    assert_eq!(
        fri06_c12_t08_parent_baseline_row(
            14.0,
            8.0,
            TrackComponent::px(40.0),
            AlignItems::Baseline,
        ),
        40.0,
    );
}

#[test]
fn fri06_c12_t08_parent_last_baseline_envelope_grows_the_same_intrinsic_row() {
    assert_eq!(
        fri06_c12_t08_parent_baseline_row(
            14.0,
            8.0,
            TrackComponent::AUTO,
            AlignItems::LastBaseline,
        ),
        26.0,
    );
}

#[test]
fn fri06_c12_t08_equal_parent_and_nested_last_baselines_need_no_intrinsic_shim() {
    assert_eq!(
        fri06_c12_t08_parent_baseline_row(
            14.0,
            14.0,
            TrackComponent::AUTO,
            AlignItems::LastBaseline,
        ),
        20.0,
    );
}

#[test]
fn fri06_c12_t08_synthesized_intrinsic_cycle_is_disqualified_from_the_baseline_group() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let participation = baseline_participation_for_container(
        AlignItems::Baseline,
        false,
        true,
        Baselines::NONE,
        flow_axes,
        flow_axes,
    );

    assert!(!participation.participates);
    assert_eq!(participation.group, None);
    assert!(participation.synthesized);
    assert_eq!(participation.fallback_alignment, Some(AlignItems::Start));
}

#[test]
fn fri06_c12_t08_inherited_baseline_gap_adjustment_is_applied_once() {
    assert_eq!(
        fri06_c12_t08_inherited_baseline_gap_position(10.0, 20.0),
        72.0
    );
}

#[test]
fn fri06_c12_t08_equal_inherited_gap_keeps_the_parent_baseline_coordinate() {
    assert_eq!(
        fri06_c12_t08_inherited_baseline_gap_position(0.0, 0.0),
        62.0
    );
}

fn assert_published_baseline_group_order_keeps_compatible_axis<S: LayoutScalar>(
    incompatible_first: bool,
) where
    lts::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let root_children = if incompatible_first { [2, 3] } else { [3, 2] };
    let incompatible_baselines = ComputeOutputOf::from_sizes_and_baselines(
        Size::new(S::from_f64(30.0), S::from_f64(20.0)),
        Size::new(S::from_f64(30.0), S::from_f64(20.0)),
        BaselinesOf {
            first: Point::new(Some(S::from_f64(15.0)), None),
            last: Point::NONE,
        },
    );
    let compatible_baselines = ComputeOutputOf::from_sizes_and_baselines(
        Size::new(S::from_f64(30.0), S::from_f64(20.0)),
        Size::new(S::from_f64(30.0), S::from_f64(20.0)),
        BaselinesOf {
            first: Point::new(None, Some(S::from_f64(14.0))),
            last: Point::NONE,
        },
    );
    let mut tree = lts::layout_tree::OracleTreeOf::<S>::new()
        .children(1, root_children)
        .children(2, [4])
        .children(3, [5])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(120.0)),
                    PreferredSizeOf::px(S::from_f64(80.0)),
                ),
                grid_template_columns: vec![
                    TrackComponentOf::px(S::from_f64(60.0)),
                    TrackComponentOf::px(S::from_f64(60.0)),
                ],
                grid_template_rows: vec![TrackComponentOf::px(S::from_f64(80.0))],
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(1).expect("first grid row"),
                grid_template_columns: vec![TrackComponentOf::px(S::from_f64(60.0))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack {
                    name_components: Vec::new(),
                })],
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("first grid row"),
                grid_template_columns: vec![TrackComponentOf::px(S::from_f64(60.0))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack {
                    name_components: Vec::new(),
                })],
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                writing_mode: WritingMode::VerticalRl,
                align_self: Some(AlignItems::Baseline),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                align_self: Some(AlignItems::Baseline),
                ..NodeInputOf::default()
            },
        )
        .measure(4, incompatible_baselines)
        .measure(5, compatible_baselines);

    let output = compute_grid(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(120.0)), Some(S::from_f64(80.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(120.0)),
                AvailableOf::definite(S::from_f64(80.0)),
            ),
        ),
    )
    .expect("sibling subgrid layout succeeds");

    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(15.0)), Some(S::from_f64(14.0)))
    );
    assert_eq!(output.last_baselines, Point::new(None, Some(S::ZERO)));
}

#[test]
fn baseline_group_order_rejects_incompatible_published_baselines_for_f32() {
    assert_published_baseline_group_order_keeps_compatible_axis::<f32>(true);
    assert_published_baseline_group_order_keeps_compatible_axis::<f32>(false);
}

#[test]
fn baseline_group_order_rejects_incompatible_published_baselines_for_f64() {
    assert_published_baseline_group_order_keeps_compatible_axis::<f64>(true);
    assert_published_baseline_group_order_keeps_compatible_axis::<f64>(false);
}

#[test]
fn column_subgrid_baseline_alignment_does_not_grow_auto_parent_row_twice() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![empty_subgrid_track()],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Grid,
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(3, baseline_measure(15.0, 15.0, Some(12.0), None))
        .measure(4, baseline_measure(30.0, 30.0, Some(24.0), None));

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

    assert_eq!(output.size, Size::new(45.0, 30.0));
    assert_eq!(final_height(&tree, 2), 30.0);
    assert_eq!(final_y(&tree, 3), 12.0);
    assert_eq!(final_y(&tree, 4), 0.0);
}

#[test]
fn grid_auto_places_children_into_declared_column_tracks() {
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
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());

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

    assert_eq!(output.size, Size::new(200.0, 40.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 40.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(120.0, 40.0)
    );
}

#[test]
fn grid_display_none_child_does_not_consume_auto_placement_cell() {
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
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
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

    assert_eq!(
        tree.inputs(2)
            .iter()
            .filter(|input| input.run_mode() == RunMode::PerformHiddenLayout)
            .count(),
        1
    );
    assert!(
        tree.inputs(3)
            .iter()
            .all(|input| input.run_mode() != RunMode::PerformHiddenLayout)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::ZERO
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(80.0, 40.0)
    );
}

#[test]
fn grid_absolute_child_does_not_consume_auto_placement_cell() {
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
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(12.0)),
            ..NodeInput::default()
        },
    );
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 12.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(80.0, 40.0)
    );
    let absolute_layout_input = tree
        .inputs(2)
        .iter()
        .find(|input| input.run_mode() == RunMode::PerformLayout)
        .expect("absolute grid child should be laid out");
    let normal_layout_input = tree
        .inputs(3)
        .iter()
        .find(|input| input.run_mode() == RunMode::PerformLayout)
        .expect("normal grid child should be laid out");
    assert_eq!(
        absolute_layout_input.known(),
        Size::new(Some(30.0), Some(12.0))
    );
    assert_eq!(
        normal_layout_input.known(),
        Size::new(Some(80.0), Some(40.0))
    );
}

#[test]
fn named_grid_absolute_child_uses_resolved_raw_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["b"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["c"]),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
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
                raw_grid_row: RawGridPlacement::lines(1, 2),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(2)
        .expect("absolute child should be laid out");

    assert_eq!(child.location, Point::new(40.0, 0.0));
    assert_eq!(child.size, Size::new(40.0, 20.0));
}

#[test]
fn vertical_grid_absolute_child_maps_rows_to_physical_x_and_columns_to_y() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [])
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
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                inset: Edges::all(LengthAuto::ZERO),
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
    let child = tree.layout(2).expect("absolute child should be laid out");

    assert_eq!(output.size, Size::new(110.0, 70.0));
    assert_eq!(child.location, Point::new(0.0, 30.0));
    assert_eq!(child.size, Size::new(60.0, 40.0));
}

#[test]
fn grid_absolute_child_without_explicit_size_uses_measured_size() {
    let mut tree =
        OracleTree::new().measure(2, ComputeOutput::from_outer_size(Size::new(36.0, 14.0)));
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(60.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(60.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
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
        Size::new(36.0, 14.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::NONE);
    assert_eq!(
        tree.inputs(2)[0].available(),
        Size::new(Available::definite(120.0), Available::definite(60.0))
    );
}

#[test]
fn grid_absolute_child_resolves_size_from_opposing_insets() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(60.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(60.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(8.0),
                right: LengthAuto::px(12.0),
                top: LengthAuto::px(6.0),
                bottom: LengthAuto::px(10.0),
            },
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
        Point::new(8.0, 6.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(100.0, 44.0)
    );
    assert_eq!(
        tree.inputs(2)[0].known(),
        Size::new(Some(100.0), Some(44.0))
    );
}

#[test]
fn grid_absolute_child_without_horizontal_insets_uses_rtl_start_alignment() {
    let mut tree =
        OracleTree::new().measure(2, ComputeOutput::from_outer_size(Size::new(30.0, 12.0)));
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            justify_items: Some(AlignItems::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(12.0)),
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
        Point::new(90.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 12.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::new(None, Some(12.0)));
}

#[test]
fn grid_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(8.0),
                right: LengthAuto::px(12.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(12.0)),
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
        Point::new(78.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 12.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(30.0), Some(12.0)));
}

#[test]
fn grid_absolute_child_expands_horizontal_auto_margins() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(12.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                ..Edges::all(LengthAuto::ZERO)
            },
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
        Point::new(45.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.left,
        45.0
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.right,
        45.0
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 12.0)
    );
}

#[test]
fn grid_absolute_child_expands_vertical_auto_margins() {
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
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
            margin: Edges {
                top: LengthAuto::AUTO,
                bottom: LengthAuto::AUTO,
                ..Edges::all(LengthAuto::ZERO)
            },
            inset: Edges {
                top: LengthAuto::px(10.0),
                bottom: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::AUTO)
            },
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
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.top,
        40.0
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.bottom,
        40.0
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(20.0, 20.0)
    );
}

#[test]
fn grid_absolute_child_percent_size_resolves_against_grid_area() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0), TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            size: Size::new(PreferredSize::percent(0.5), PreferredSize::percent(0.5)),
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
        Point::new(120.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(40.0, 20.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(40.0), Some(20.0)));
}

#[test]
fn grid_absolute_child_percent_padding_resolves_against_grid_area() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0), TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(12.0)),
            padding: Edges::all(Length::percent(0.1)),
            border: Edges::all(Length::percent(0.05)),
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
        Point::new(120.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").padding,
        Edges::all(8.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").border,
        Edges::all(4.0)
    );
}

#[test]
fn grid_absolute_child_applies_aspect_ratio_to_authored_size() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(30.0), PreferredSize::AUTO),
            aspect_ratio: AspectRatio::new(2.0),
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
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 15.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(30.0), Some(15.0)));
}

#[test]
fn grid_absolute_child_clamps_authored_size_to_min_and_max() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::AUTO, MinSize::px(30.0)),
            max_size: Size::new(MaxSize::px(50.0), MaxSize::NONE),
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
        tree.layout(2).expect("node layout is staged").size,
        Size::new(50.0, 30.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(50.0), Some(30.0)));
}

#[test]
fn grid_absolute_child_content_box_size_includes_padding_and_border() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
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
        tree.layout(2).expect("node layout is staged").size,
        Size::new(42.0, 32.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(42.0), Some(32.0)));
}

#[test]
fn grid_absolute_child_size_cannot_shrink_below_padding_and_border() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(4.0), PreferredSize::px(4.0)),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
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
        tree.layout(2).expect("node layout is staged").size,
        Size::new(12.0, 12.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(12.0), Some(12.0)));
}

#[test]
fn grid_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(10.0),
                right: LengthAuto::px(20.0),
                top: LengthAuto::AUTO,
                bottom: LengthAuto::AUTO,
            },
            aspect_ratio: AspectRatio::new(2.0),
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
        tree.layout(2).expect("node layout is staged").size,
        Size::new(90.0, 45.0)
    );
    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(90.0), Some(45.0)));
}

#[test]
fn grid_absolute_child_available_space_excludes_non_auto_margins() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            margin: Edges {
                left: LengthAuto::px(10.0),
                right: LengthAuto::px(20.0),
                top: LengthAuto::px(3.0),
                bottom: LengthAuto::px(7.0),
            },
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
        tree.inputs(2)[0].available(),
        Size::new(Available::definite(90.0), Available::definite(70.0))
    );
    assert_eq!(
        tree.inputs(2)[0].parent(),
        Size::new(Some(120.0), Some(80.0))
    );
}

#[test]
fn grid_item_with_aspect_ratio_stretches_width_and_keeps_start_aligned_height() {
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
            aspect_ratio: AspectRatio::new(2.0),
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

    let layout_input = tree
        .inputs(2)
        .iter()
        .find(|input| input.run_mode() == RunMode::PerformLayout)
        .expect("grid item should be laid out");
    assert_eq!(layout_input.known(), Size::new(Some(100.0), Some(50.0)));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(100.0, 50.0)
    );
}

#[test]
fn grid_item_expands_inline_auto_margins_after_child_layout() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            margin: Edges {
                top: LengthAuto::ZERO,
                left: LengthAuto::Auto,
                right: LengthAuto::Auto,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 40.0)));

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

    let layout_input = tree
        .inputs(2)
        .iter()
        .find(|input| input.run_mode() == RunMode::PerformLayout)
        .expect("grid item should be laid out");
    assert_eq!(layout_input.known().width, None);
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(40.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.left,
        40.0
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.right,
        40.0
    );
}

#[test]
fn grid_auto_flow_column_places_children_down_rows_then_across_columns() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4]);
    for node in 2..=4 {
        tree.insert_children(node, vec![]);
        tree.insert_style(node, NodeInput::default());
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(50.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(30.0)],
            grid_auto_columns: vec![TrackComponent::px(40.0)],
            grid_auto_flow: GridAutoFlow::Column,
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

    assert_eq!(output.content_size, Size::new(120.0, 50.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(80.0, 30.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(40.0, 20.0)
    );
}

#[test]
fn grid_align_content_center_offsets_tracks_inside_inner_height() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            align_content: Some(AlignContent::Center),
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 30.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 40.0)
    );
}

#[test]
fn grid_align_items_center_offsets_smaller_child_within_grid_area() {
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
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
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
        Point::new(0.0, 15.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 10.0)
    );
}

#[test]
fn grid_align_self_overrides_parent_align_items() {
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
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            align_self: Some(AlignItems::End),
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
        Point::new(0.0, 30.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 10.0)
    );
}

#[test]
fn grid_aligns_items_to_shared_first_baseline() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(14.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 6.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_aligns_items_to_shared_last_baseline() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, None, Some(4.0)))
        .measure(3, baseline_measure(30.0, 30.0, None, Some(10.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 14.0);
    assert_eq!(final_y(&tree, 3), 10.0);
}

#[test]
fn grid_reports_first_baseline_from_first_row_grid_order() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(120.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(7.0), None))
        .measure(3, baseline_measure(30.0, 20.0, Some(9.0), None));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.first_baselines.y, Some(7.0));
}

#[test]
fn grid_reports_last_baseline_from_last_row_grid_order() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(120.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, None, Some(6.0)))
        .measure(3, baseline_measure(30.0, 30.0, None, Some(8.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.last_baselines.y, Some(62.0));
}

#[test]
fn grid_reports_first_baseline_from_shared_major_group_before_fallback_item() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(3, baseline_measure(30.0, 20.0, Some(14.0), None));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.first_baselines.y, Some(14.0));
}

#[test]
fn grid_reports_last_baseline_from_shared_minor_group_before_fallback_item() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(30.0)],
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(6.0)))
        .measure(3, baseline_measure(30.0, 20.0, None, Some(2.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.last_baselines.y, Some(64.0));
}

#[test]
fn grid_reports_last_baseline_from_spanning_item_that_occupies_last_row() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(6.0)))
        .measure(3, baseline_measure(30.0, 80.0, None, Some(8.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.last_baselines.y, Some(72.0));
}

#[test]
fn grid_aligns_first_baseline_with_block_margins() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    bottom: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(14.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 6.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_aligns_last_baseline_with_block_margins() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    bottom: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, None, Some(4.0)))
        .measure(3, baseline_measure(30.0, 30.0, None, Some(10.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 14.0);
    assert_eq!(final_y(&tree, 3), 10.0);
}

#[test]
fn grid_aligns_first_baseline_for_item_spanning_rows_with_gap() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(120.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::ZERO, Length::px(7.0)),
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(14.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 6.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_aligns_last_baseline_for_item_spanning_rows_with_gap() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(120.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::ZERO, Length::px(7.0)),
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(4.0)))
        .measure(3, baseline_measure(30.0, 30.0, None, Some(10.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 61.0);
    assert_eq!(final_y(&tree, 3), 57.0);
}

#[test]
fn grid_baseline_increases_auto_row_size() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(6.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 42.0);
    assert_eq!(final_y(&tree, 3), 12.0);
}

#[test]
fn grid_last_baseline_increases_auto_row_size() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, None, Some(2.0)))
        .measure(3, baseline_measure(30.0, 25.0, None, Some(12.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 30.0);
    assert_eq!(final_y(&tree, 3), 5.0);
}

#[test]
fn grid_absolute_baseline_child_does_not_affect_row_baseline_shim() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(6.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 20.0);
}

#[test]
fn grid_auto_block_margin_baseline_child_does_not_affect_row_baseline_shim() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                margin: Edges {
                    top: LengthAuto::Auto,
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(6.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 30.0);
}

#[test]
fn grid_baseline_less_child_spanning_intrinsic_row_uses_fallback_without_shim() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, ComputeOutput::from_outer_size(Size::new(30.0, 10.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 20.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_fixed_row_baseline_seeds_spanning_auto_row_shim() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(6.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 42.0);
    assert_eq!(final_y(&tree, 3), 12.0);
}

#[test]
fn grid_constrained_row_baseline_sizing_uses_layout_mode() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure_when(
            3,
            OracleMeasurement::new(baseline_measure(30.0, 30.0, Some(6.0), None))
                .run_mode(RunMode::PerformLayout),
        );

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 42.0);
    assert_eq!(final_y(&tree, 3), 12.0);
}

#[test]
fn grid_baseline_less_child_in_fixed_row_does_not_grow_intrinsic_sizing() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, ComputeOutput::from_outer_size(Size::new(30.0, 30.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 20.0);
    assert_eq!(final_y(&tree, 2), 12.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_justify_items_center_offsets_smaller_child_within_grid_area() {
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
    tree.insert_style(2, NodeInput::default());
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
        Point::new(25.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 10.0)
    );
}

#[test]
fn grid_child_affine_size_and_margin_resolve_against_grid_area() {
    let mut tree = OracleTree::new();
    let width = lp(10.0, 0.5);
    let margin = lp(5.0, 0.1);
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::value(width), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::value(margin),
                right: LengthAuto::ZERO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
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
            Size::new(Some(100.0), Some(40.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::Definite(100.0), Available::Definite(40.0)),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(2).last().map(|input| input.known()),
        Some(Size::new(Some(60.0), Some(10.0)))
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(15.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(60.0, 10.0)
    );
}

#[test]
fn grid_child_pure_helpers_accept_non_default_scalar() {
    let geometry = tagged_geometry(PhysicalAxis::Vertical, 80.0, 30.0, 12.5, 7.25);
    let shared = tagged_group(PhysicalAxis::Vertical, Some(20.0), Some(10.0));

    assert_eq!(
        baseline_shim_for_intrinsic_contribution(
            BaselineParticipation {
                participates: true,
                group: Some(BaselineGroupKind::Major),
                synthesized: false,
                fallback_alignment: None,
            },
            geometry,
            shared,
            PhysicalAxis::Vertical,
        ),
        BaselineShim::<f64> {
            before: 7.5,
            after: 0.0,
        }
    );
    assert_eq!(
        baseline_offset(
            BaselineGroupKind::Minor,
            tagged_baseline(PhysicalAxis::Vertical, 10.0_f64),
            geometry,
            PhysicalAxis::Vertical,
        ),
        Some(47.25)
    );
    assert_eq!(spanned_track_size(&[10.0_f64, 20.0, 30.0], 0, 3, 2.5), 65.0);

    assert_eq!(
        logical_grid_item_axis::<f64>(100.0, 20.0, None, None, AlignItems::Center,),
        ResolvedGridItemAxis::<f64> {
            offset: 40.0,
            margin_start: 40.0,
            margin_end: 40.0,
        }
    );

    assert_eq!(
        absolute_grid_axis(AbsoluteGridAxis::<f64> {
            area_location: 5.0,
            static_area_location: 10.0,
            area_size: 100.0,
            static_area_size: 80.0,
            size: 20.0,
            margin_start: Some(2.5),
            margin_end: Some(7.5),
            inset_start: None,
            inset_end: None,
            alignment: AlignItems::End,
            progression: crate::geometry::PhysicalProgression::Increasing,
        }),
        ResolvedAbsoluteGridAxis::<f64> {
            location: 62.5,
            margin_start: 2.5,
            margin_end: 7.5,
        }
    );
}

#[test]
fn grid_child_pending_and_subgrid_inheritance_helpers_accept_non_default_scalar() {
    let area = GridArea::<f64> {
        column: 0,
        row: 0,
        column_end: 1,
        row_end: 2,
        size: LogicalSizeOf::new(40.0, 90.0),
    };
    let item = PendingGridItem::<_, f64> {
        node: "child",
        style: grid_item_projection!(&NodeInputOf::default()),
        nested_container: None,
        source_index: 0,
        area,
        output: ComputeOutputOf::<f64>::from_sizes_and_baselines(
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
            BaselinesOf {
                first: Point::new(None, Some(8.0)),
                last: Point::new(None, Some(22.0)),
            },
        ),
        horizontal_axis: ResolvedGridItemAxis::<f64> {
            offset: 0.0,
            margin_start: 0.0,
            margin_end: 0.0,
        },
        vertical_axis: ResolvedGridItemAxis::<f64> {
            offset: 0.0,
            margin_start: 3.0,
            margin_end: 5.0,
        },
        child_flow_axes: crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        logical_relative_offset: LogicalPointOf::new(0.0, 0.0),
        first_baseline: BaselinesOf {
            first: Point::new(None, Some(8.0)),
            last: Point::NONE,
        }
        .first_block_baseline(crate::geometry::FlowAxes::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
        ))
        .expect("the test baseline is present"),
        last_baseline: BaselinesOf {
            first: Point::NONE,
            last: Point::new(None, Some(22.0)),
        }
        .last_block_baseline(crate::geometry::FlowAxes::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
        ))
        .expect("the test baseline is present"),
        location: Point::ZERO,
        block_offset: 0.0,
        block_auto_margins: false,
        baseline_participation: BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Major),
            synthesized: false,
            fallback_alignment: None,
        },
        margin: Edges::new(3.0, 0.0, 5.0, 0.0),
        border: Edges::ZERO,
        padding: Edges::ZERO,
        overflow: used_overflow(Overflow::Visible, Overflow::Visible),
    };

    let groups = baseline_groups(
        std::slice::from_ref(&item),
        2,
        1,
        horizontal_baseline_flow_axes(),
    );
    assert_eq!(
        groups.rows[0].first,
        Some(tagged_baseline(PhysicalAxis::Vertical, 11.0))
    );
    assert_eq!(
        baseline_aligned_block_offset(
            &item,
            &groups,
            &[40.0_f64, 40.0],
            10.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(3.0)
    );

    let inherited = inherit_subgrid_tracks(SubgridTrackInheritanceInput::<f64> {
        parent_tracks: &[20.0, 30.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 2.0,
        end_mbp: 4.0,
        parent_gap: 6.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();
    assert_eq!(inherited.gap_difference, 2.0);
    assert_eq!(inherited.final_tracks, vec![16.0, 24.0]);

    let inherited_baselines = inherit_subgrid_baselines(SubgridBaselineInheritanceInput::<f64> {
        parent_major: &[
            Some(tagged_baseline(PhysicalAxis::Vertical, 9.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 17.0)),
        ],
        parent_minor: &[
            Some(tagged_baseline(PhysicalAxis::Vertical, 4.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 6.0)),
        ],
        physical_axis: PhysicalAxis::Vertical,
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 2.0,
        end_mbp: 4.0,
        parent_gap: 6.0,
        subgrid_gap: inherited.resolved_subgrid_gap,
    })
    .unwrap();
    assert_eq!(inherited_baselines.gap_difference, 2.0);
    assert_eq!(
        inherited_baselines.final_major,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 7.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 15.0)),
        ]
    );
    assert_eq!(
        inherited_baselines.final_minor,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 2.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 2.0)),
        ]
    );

    let (layout_tracks, layout_gap) =
        inherited_subgrid_layout_tracks(GridAxisKind::Row, &inherited);
    assert_eq!(layout_tracks, vec![16.0, 24.0]);
    assert_eq!(layout_gap, 10.0);

    let offset_style = NodeInputOf::<f64>::default();
    let offset_style_projection = grid_container_projection!(&offset_style);
    let offset_geometry = UsedGridAxisGeometryOf::new(vec![12.5, 17.5], vec![false, false], 3.25);
    let offsets = grid_axis_offsets(GridAxisOffsetsInput::<f64> {
        style: &offset_style_projection,
        axis: GridAxisKind::Column,
        tracks: &[12.5, 17.5],
        geometry: &offset_geometry,
        inherited_offset: Some(1.25),
        content_box_left: 0.0,
        content_box_size: Size::new(60.0, 20.0),
        content_box_inset: Edges::new(0.0, 0.0, 0.0, 2.0),
        alignment: GridAlignment {
            start: 0.5,
            gap: 3.25,
        },
    });
    assert_eq!(offsets, vec![3.75, 19.5]);

    let child_style = NodeInputOf::<f64> {
        display: Display::Grid,
        grid_template_rows: subgrid_track_of(),
        ..NodeInputOf::default()
    };
    let parent_context = subgrid_child_parent_context(SubgridChildParentContextInput::<_, f64> {
        item: SubgridItemReport {
            node: "child",
            column: SubgridAxisReport {
                mapping: GridAxisMappingReport {
                    queried_axis: GridAxisKind::Column,
                    parent_axis: GridAxisKind::Column,
                    child_axis: GridAxisKind::Column,
                    reversed: false,
                },
                eligibility: SubgridEligibility {
                    eligible: false,
                    reason: Some(SubgridIneligibleReason::NotRequested),
                },
            },
            row: SubgridAxisReport {
                mapping: GridAxisMappingReport {
                    queried_axis: GridAxisKind::Row,
                    parent_axis: GridAxisKind::Row,
                    child_axis: GridAxisKind::Row,
                    reversed: false,
                },
                eligibility: SubgridEligibility {
                    eligible: true,
                    reason: None,
                },
            },
        },
        child_style: &child_style,
        area,
        content_box_size: Size::new(40.0, 90.0),
        columns: &[40.0],
        rows: &[20.0, 30.0],
        gap: LogicalSizeOf::new(0.0, 6.0),
        parent_named_columns: &named::NamedGridLines::new(GridAxisKind::Column, 1),
        parent_named_rows: &named::NamedGridLines::new(GridAxisKind::Row, 2),
        parent_area_facts: None,
        parent_baseline_groups: &GridBaselineGroups::<f64> {
            rows: groups.rows,
            columns: vec![TrackBaselineGroup::default()],
        },
        margin: Edges::ZERO.map(Some),
        border: Edges::ZERO,
        padding: Edges::ZERO,
    })
    .unwrap();

    let rows = parent_context.rows.expect("row subgrid should inherit");
    assert_eq!(rows.tracks, vec![20.0, 30.0]);
    assert_eq!(rows.gap, 6.0);
}

#[test]
fn grid_alignment_accepts_f64_and_preserves_fractional_distribution() {
    let alignment = grid_alignment::<f64>(9_000_000.75_f64, 3, 0.25_f64, AlignContent::SpaceAround);

    assert_eq!(alignment.start, 1_500_000.125_f64);
    assert_eq!(alignment.gap, 3_000_000.5_f64);
}

fn horizontal_baseline_flow_axes() -> crate::geometry::FlowAxes {
    crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
}

fn baseline_test_item(
    row: usize,
    column: usize,
    row_span: usize,
    align_self: AlignItems,
    first: Scalar,
    last: Scalar,
    height: Scalar,
) -> PendingGridItem<()> {
    PendingGridItem {
        node: (),
        style: grid_item_projection!(&NodeInput {
            align_self: Some(align_self),
            ..NodeInput::default()
        }),
        nested_container: None,
        source_index: 0,
        area: GridArea {
            row,
            column,
            row_end: row + row_span,
            column_end: column + 1,
            size: LogicalSizeOf::new(40.0, height),
        },
        output: ComputeOutput::from_sizes_and_baselines(
            Size::new(40.0, height),
            Size::ZERO,
            Baselines {
                first: Point::new(None, Some(first)),
                last: Point::new(None, Some(last)),
            },
        ),
        horizontal_axis: ResolvedGridItemAxis {
            offset: 0.0,
            margin_start: 0.0,
            margin_end: 0.0,
        },
        vertical_axis: ResolvedGridItemAxis {
            offset: 0.0,
            margin_start: 0.0,
            margin_end: 0.0,
        },
        child_flow_axes: crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        logical_relative_offset: LogicalPointOf::new(0.0, 0.0),
        first_baseline: Baselines {
            first: Point::new(None, Some(first)),
            last: Point::NONE,
        }
        .first_block_baseline(crate::geometry::FlowAxes::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
        ))
        .expect("the test baseline is present"),
        last_baseline: Baselines {
            first: Point::NONE,
            last: Point::new(None, Some(last)),
        }
        .last_block_baseline(crate::geometry::FlowAxes::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
        ))
        .expect("the test baseline is present"),
        location: Point::ZERO,
        block_offset: 0.0,
        block_auto_margins: false,
        baseline_participation: BaselineParticipation {
            participates: matches!(align_self, AlignItems::Baseline | AlignItems::LastBaseline),
            group: match align_self {
                AlignItems::Baseline => Some(BaselineGroupKind::Major),
                AlignItems::LastBaseline => Some(BaselineGroupKind::Minor),
                _ => None,
            },
            synthesized: false,
            fallback_alignment: None,
        },
        margin: Edges::ZERO,
        border: Edges::ZERO,
        padding: Edges::ZERO,
        overflow: used_overflow(Overflow::Visible, Overflow::Visible),
    }
}

#[test]
fn row_baselines_choose_first_baseline_for_first_group() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 22.0, 30.0),
        baseline_test_item(0, 1, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 2, horizontal_baseline_flow_axes());

    assert_eq!(
        groups.rows[0].first,
        Some(tagged_baseline(PhysicalAxis::Vertical, 14.0))
    );
}

#[test]
fn row_baselines_choose_last_baseline_for_last_group() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::LastBaseline, 8.0, 22.0, 30.0),
        baseline_test_item(0, 1, 2, AlignItems::LastBaseline, 8.0, 18.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 2, horizontal_baseline_flow_axes());

    assert_eq!(
        groups.rows[1].last,
        Some(tagged_baseline(PhysicalAxis::Vertical, 12.0))
    );
}

#[test]
fn row_baselines_keep_first_groups_per_start_row() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 22.0, 30.0),
        baseline_test_item(1, 0, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 1, horizontal_baseline_flow_axes());

    assert_eq!(
        groups.rows[0].first,
        Some(tagged_baseline(PhysicalAxis::Vertical, 8.0))
    );
    assert_eq!(
        groups.rows[1].first,
        Some(tagged_baseline(PhysicalAxis::Vertical, 14.0))
    );
}

#[test]
fn row_baselines_keep_last_groups_per_end_row() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 22.0, 30.0),
        baseline_test_item(1, 0, 1, AlignItems::LastBaseline, 8.0, 18.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 1, horizontal_baseline_flow_axes());

    assert_eq!(
        groups.rows[0].last,
        Some(tagged_baseline(PhysicalAxis::Vertical, 8.0))
    );
    assert_eq!(
        groups.rows[1].last,
        Some(tagged_baseline(PhysicalAxis::Vertical, 12.0))
    );
}

#[test]
fn baseline_groups_columns_are_default_filled_to_grid_width() {
    let items = vec![baseline_test_item(
        0,
        0,
        1,
        AlignItems::Baseline,
        8.0,
        22.0,
        30.0,
    )];

    let groups = baseline_groups(&items, 1, 3, horizontal_baseline_flow_axes());

    assert_eq!(groups.columns, vec![TrackBaselineGroup::default(); 3],);
}

#[test]
fn baseline_offset_major_uses_margin_box_baseline() {
    let offset = baseline_offset(
        BaselineGroupKind::Major,
        tagged_baseline(PhysicalAxis::Vertical, 20.0),
        tagged_geometry(PhysicalAxis::Vertical, 70.0, 40.0, 14.0, 12.0),
        PhysicalAxis::Vertical,
    );

    assert_eq!(offset, Some(6.0));
}

#[test]
fn baseline_offset_minor_uses_alignment_context_end() {
    let offset = baseline_offset(
        BaselineGroupKind::Minor,
        tagged_baseline(PhysicalAxis::Vertical, 18.0),
        tagged_geometry(PhysicalAxis::Vertical, 70.0, 40.0, 14.0, 12.0),
        PhysicalAxis::Vertical,
    );

    assert_eq!(offset, Some(24.0));
}

#[test]
fn baseline_offset_major_allows_row_spanning_gap_area() {
    let offset = baseline_offset(
        BaselineGroupKind::Major,
        tagged_baseline(PhysicalAxis::Vertical, 14.0),
        tagged_geometry(PhysicalAxis::Vertical, 90.0, 30.0, 8.0, 10.0),
        PhysicalAxis::Vertical,
    );

    assert_eq!(offset, Some(6.0));
}

#[test]
fn baseline_offset_minor_allows_row_spanning_gap_area() {
    let offset = baseline_offset(
        BaselineGroupKind::Minor,
        tagged_baseline(PhysicalAxis::Vertical, 14.0),
        tagged_geometry(PhysicalAxis::Vertical, 90.0, 30.0, 8.0, 10.0),
        PhysicalAxis::Vertical,
    );

    assert_eq!(offset, Some(56.0));
}

#[test]
fn baseline_shim_for_intrinsic_contribution_first_grows_before_item() {
    let shim = baseline_shim_for_intrinsic_contribution(
        BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Major),
            synthesized: false,
            fallback_alignment: Some(AlignItems::Start),
        },
        tagged_geometry(PhysicalAxis::Vertical, 40.0, 30.0, 6.0, 8.0),
        tagged_group(PhysicalAxis::Vertical, Some(18.0), Some(12.0)),
        PhysicalAxis::Vertical,
    );

    assert_eq!(
        shim,
        BaselineShim {
            before: 12.0,
            after: 0.0,
        }
    );
}

#[test]
fn baseline_shim_for_intrinsic_contribution_last_grows_after_item() {
    let shim = baseline_shim_for_intrinsic_contribution(
        BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Minor),
            synthesized: false,
            fallback_alignment: Some(AlignItems::End),
        },
        tagged_geometry(PhysicalAxis::Vertical, 40.0, 30.0, 6.0, 2.0),
        tagged_group(PhysicalAxis::Vertical, Some(18.0), Some(12.0)),
        PhysicalAxis::Vertical,
    );

    assert_eq!(
        shim,
        BaselineShim {
            before: 0.0,
            after: 10.0,
        }
    );
}

#[test]
fn baseline_shim_for_intrinsic_contribution_nonparticipant_is_zero() {
    let shim = baseline_shim_for_intrinsic_contribution(
        BaselineParticipation {
            participates: false,
            group: None,
            synthesized: false,
            fallback_alignment: None,
        },
        tagged_geometry(PhysicalAxis::Vertical, 40.0, 30.0, 6.0, 2.0),
        tagged_group(PhysicalAxis::Vertical, Some(18.0), Some(12.0)),
        PhysicalAxis::Vertical,
    );

    assert_eq!(shim, BaselineShim::default());
}

#[test]
fn baseline_shim_for_intrinsic_contribution_synthesized_baseline_participates() {
    let participation = baseline_participation(
        AlignItems::Baseline,
        false,
        false,
        Baselines::NONE,
        crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
    );

    let shim = baseline_shim_for_intrinsic_contribution(
        participation,
        tagged_geometry(PhysicalAxis::Vertical, 40.0, 30.0, 6.0, 2.0),
        tagged_group(PhysicalAxis::Vertical, Some(18.0), Some(12.0)),
        PhysicalAxis::Vertical,
    );

    assert_eq!(
        shim,
        BaselineShim {
            before: 12.0,
            after: 0.0
        }
    );
}

#[test]
fn baseline_aligned_block_offset_first_single_row_item() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 1, 2, horizontal_baseline_flow_axes());

    assert_eq!(
        baseline_aligned_block_offset(
            &items[0],
            &groups,
            &[40.0],
            0.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(6.0)
    );
    assert_eq!(
        baseline_aligned_block_offset(
            &items[1],
            &groups,
            &[40.0],
            0.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(0.0)
    );
}

#[test]
fn baseline_aligned_block_offset_first_spanning_item() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::Baseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 2, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 2, 2, horizontal_baseline_flow_axes());

    assert_eq!(
        baseline_aligned_block_offset(
            &items[0],
            &groups,
            &[40.0, 40.0],
            7.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(6.0)
    );
}

#[test]
fn baseline_aligned_block_offset_last_single_row_item() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 1, 2, horizontal_baseline_flow_axes());

    assert_eq!(
        baseline_aligned_block_offset(
            &items[0],
            &groups,
            &[40.0],
            0.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(14.0)
    );
    assert_eq!(
        baseline_aligned_block_offset(
            &items[1],
            &groups,
            &[40.0],
            0.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(10.0)
    );
}

#[test]
fn baseline_aligned_block_offset_last_spanning_item() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 2, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 2, 2, horizontal_baseline_flow_axes());

    assert_eq!(
        baseline_aligned_block_offset(
            &items[0],
            &groups,
            &[40.0, 40.0],
            7.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(61.0)
    );
}

#[test]
fn baseline_aligned_block_offset_first_and_last_include_margins() {
    let mut first_items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];
    first_items[0].vertical_axis.margin_start = 3.0;
    first_items[0].vertical_axis.margin_end = 5.0;
    let first_groups = baseline_groups(&first_items, 1, 2, horizontal_baseline_flow_axes());

    assert_eq!(
        baseline_aligned_block_offset(
            &first_items[0],
            &first_groups,
            &[40.0],
            0.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(6.0)
    );

    let mut last_items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    last_items[0].vertical_axis.margin_start = 3.0;
    last_items[0].vertical_axis.margin_end = 5.0;
    let last_groups = baseline_groups(&last_items, 1, 2, horizontal_baseline_flow_axes());

    assert_eq!(
        baseline_aligned_block_offset(
            &last_items[0],
            &last_groups,
            &[40.0],
            0.0,
            horizontal_baseline_flow_axes(),
        ),
        Some(14.0)
    );
}

#[test]
fn baseline_aligned_block_offset_returns_none_without_group_baseline() {
    let items = [baseline_test_item(
        0,
        0,
        1,
        AlignItems::Baseline,
        8.0,
        16.0,
        20.0,
    )];
    let groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default()],
        columns: vec![TrackBaselineGroup::default()],
    };

    assert_eq!(
        baseline_aligned_block_offset(
            &items[0],
            &groups,
            &[40.0],
            0.0,
            horizontal_baseline_flow_axes(),
        ),
        None
    );
}

#[test]
fn baseline_participation_rejects_block_auto_margins() {
    let participation = baseline_participation(
        AlignItems::Baseline,
        true,
        false,
        Baselines::NONE,
        crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
    );

    assert_eq!(
        participation,
        BaselineParticipation {
            participates: false,
            group: Some(BaselineGroupKind::Major),
            synthesized: true,
            fallback_alignment: Some(AlignItems::Start),
        }
    );
}

#[test]
fn baseline_participation_accepts_synthesized_baselines() {
    let participation = baseline_participation(
        AlignItems::LastBaseline,
        false,
        false,
        Baselines::NONE,
        crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
    );

    assert_eq!(
        participation,
        BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Minor),
            synthesized: true,
            fallback_alignment: Some(AlignItems::End),
        }
    );
}

fn tagged_geometry<S: LayoutScalar>(
    axis: PhysicalAxis,
    available_span_size: S,
    margin_box_size: S,
    major_baseline: S,
    minor_baseline: S,
) -> BaselineGeometry<S> {
    BaselineGeometry {
        available_span_size,
        margin_box_size,
        major_baseline: tagged_baseline(axis, major_baseline),
        minor_baseline: tagged_baseline(axis, minor_baseline),
    }
}

fn axis_baseline_item<S: LayoutScalar>() -> PendingGridItem<(), S> {
    let child_flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    PendingGridItem {
        node: (),
        style: default_grid_item_projection(),
        nested_container: None,
        source_index: 0,
        area: GridArea {
            column: 0,
            row: 0,
            column_end: 1,
            row_end: 1,
            size: LogicalSizeOf::new(S::from_f64(70.0), S::from_f64(80.0)),
        },
        output: ComputeOutputOf::from_sizes_and_baselines(
            Size::new(S::from_f64(30.0), S::from_f64(20.0)),
            Size::new(S::from_f64(30.0), S::from_f64(20.0)),
            BaselinesOf {
                first: Point::new(Some(S::from_f64(7.0)), None),
                last: Point::new(Some(S::from_f64(11.0)), None),
            },
        ),
        horizontal_axis: ResolvedGridItemAxis {
            offset: S::ZERO,
            margin_start: S::ZERO,
            margin_end: S::ZERO,
        },
        vertical_axis: ResolvedGridItemAxis {
            offset: S::ZERO,
            margin_start: S::ZERO,
            margin_end: S::ZERO,
        },
        child_flow_axes,
        logical_relative_offset: LogicalPointOf::new(S::ZERO, S::ZERO),
        first_baseline: tagged_baseline(PhysicalAxis::Horizontal, S::from_f64(7.0)),
        last_baseline: tagged_baseline(PhysicalAxis::Horizontal, S::from_f64(11.0)),
        location: Point::new(S::from_f64(17.0), S::from_f64(19.0)),
        block_offset: S::ZERO,
        block_auto_margins: false,
        baseline_participation: BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Major),
            synthesized: false,
            fallback_alignment: Some(AlignItems::Start),
        },
        margin: Edges::ZERO,
        border: Edges::ZERO,
        padding: Edges::ZERO,
        overflow: used_overflow(Overflow::Visible, Overflow::Visible),
    }
}

fn assert_baseline_group_axis_rejects_incompatible_application<S: LayoutScalar>() {
    let item = axis_baseline_item::<S>();
    let groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup {
            first: Some(tagged_baseline(PhysicalAxis::Vertical, S::from_f64(45.0))),
            last: None,
        }],
        columns: vec![TrackBaselineGroup::default()],
    };

    assert_eq!(
        baseline_aligned_block_offset(
            &item,
            &groups,
            &[S::from_f64(80.0)],
            S::ZERO,
            crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        ),
        None
    );
    assert_eq!(
        item.location,
        Point::new(S::from_f64(17.0), S::from_f64(19.0))
    );

    let baselines = grid_container_baselines(
        std::slice::from_ref(&item),
        &groups,
        &[S::ZERO],
        &[S::from_f64(80.0)],
        crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
    )
    .baselines;
    assert_eq!(baselines.first, Point::new(Some(S::from_f64(24.0)), None));
    assert_eq!(baselines.last, Point::new(Some(S::from_f64(28.0)), None));
}

fn assert_baseline_group_axis_preserves_compatible_application<S: LayoutScalar>() {
    let item = axis_baseline_item::<S>();
    let groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup {
            first: Some(tagged_baseline(PhysicalAxis::Horizontal, S::from_f64(45.0))),
            last: None,
        }],
        columns: vec![TrackBaselineGroup::default()],
    };

    assert_eq!(
        baseline_aligned_block_offset(
            &item,
            &groups,
            &[S::from_f64(80.0)],
            S::ZERO,
            crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        ),
        Some(S::from_f64(22.0))
    );
    let baselines = grid_container_baselines(
        std::slice::from_ref(&item),
        &groups,
        &[S::ZERO],
        &[S::from_f64(80.0)],
        crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
    )
    .baselines;
    assert_eq!(baselines.first, Point::new(Some(S::from_f64(45.0)), None));
    assert_eq!(baselines.last, Point::new(Some(S::from_f64(28.0)), None));
}

fn assert_baseline_group_axis_rejects_incompatible_intrinsic_shim<S: LayoutScalar>() {
    let geometry = BaselineGeometry {
        available_span_size: S::from_f64(80.0),
        margin_box_size: S::from_f64(30.0),
        major_baseline: tagged_baseline(PhysicalAxis::Horizontal, S::from_f64(7.0)),
        minor_baseline: tagged_baseline(PhysicalAxis::Horizontal, S::from_f64(19.0)),
    };
    let shared = TrackBaselineGroup {
        first: Some(tagged_baseline(PhysicalAxis::Vertical, S::from_f64(45.0))),
        last: Some(tagged_baseline(PhysicalAxis::Vertical, S::from_f64(61.0))),
    };
    let participation = BaselineParticipation {
        participates: true,
        group: Some(BaselineGroupKind::Major),
        synthesized: false,
        fallback_alignment: Some(AlignItems::Start),
    };

    assert_eq!(
        baseline_shim_for_intrinsic_contribution(
            participation,
            geometry,
            shared,
            PhysicalAxis::Horizontal,
        ),
        BaselineShim::default()
    );
}

#[test]
fn baseline_group_axis_rejects_incompatible_subgrid_application_for_f32() {
    assert_baseline_group_axis_rejects_incompatible_application::<f32>();
    assert_baseline_group_axis_preserves_compatible_application::<f32>();
    assert_baseline_group_axis_rejects_incompatible_intrinsic_shim::<f32>();
}

#[test]
fn baseline_group_axis_rejects_incompatible_subgrid_application_for_f64() {
    assert_baseline_group_axis_rejects_incompatible_application::<f64>();
    assert_baseline_group_axis_preserves_compatible_application::<f64>();
    assert_baseline_group_axis_rejects_incompatible_intrinsic_shim::<f64>();
}

fn assert_orthogonal_baseline_subgrid_rejects_inherited_physical_y<S: LayoutScalar>()
where
    lts::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let child_baselines = ComputeOutputOf::from_sizes_and_baselines(
        Size::new(S::from_f64(30.0), S::from_f64(20.0)),
        Size::new(S::from_f64(30.0), S::from_f64(20.0)),
        BaselinesOf {
            first: Point::new(Some(S::from_f64(7.0)), None),
            last: Point::new(Some(S::from_f64(11.0)), None),
        },
    );
    let mut tree = lts::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(80.0)),
                    PreferredSizeOf::px(S::from_f64(60.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(S::from_f64(60.0))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack {
                    name_components: Vec::new(),
                })],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode: WritingMode::VerticalRl,
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .measure(2, child_baselines);
    let parent_context = GridParentContext {
        columns: None,
        rows: Some(InheritedGridAxis {
            offset: S::ZERO,
            gap: S::ZERO,
            tracks: vec![S::from_f64(80.0)],
            geometry: UsedGridAxisGeometryOf::new(vec![S::from_f64(80.0)], vec![false], S::ZERO),
            named_lines: named::NamedGridLines::new(GridAxisKind::Row, 1),
            area_facts: None,
            template_area_expanded: false,
            major_baselines: vec![Some(tagged_baseline(
                PhysicalAxis::Vertical,
                S::from_f64(45.0),
            ))],
            minor_baselines: vec![Some(tagged_baseline(
                PhysicalAxis::Vertical,
                S::from_f64(61.0),
            ))],
            owner_baseline_targets: None,
            parent_start: 0,
            parent_end: 1,
            reversed: false,
        }),
    };

    let output = compute_grid_with_context(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(80.0)), Some(S::from_f64(60.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(80.0)),
                AvailableOf::definite(S::from_f64(60.0)),
            ),
        ),
        parent_context,
    )
    .expect("orthogonal subgrid layout succeeds");
    let child = tree.layout(2).expect("subgrid child layout is staged");

    assert_eq!(child.location, Point::new(S::from_f64(50.0), S::ZERO));
    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(57.0)), None)
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(S::from_f64(61.0)), None)
    );
}

#[test]
fn orthogonal_baseline_subgrid_rejects_inherited_physical_y_for_f32() {
    assert_orthogonal_baseline_subgrid_rejects_inherited_physical_y::<f32>();
}

#[test]
fn orthogonal_baseline_subgrid_rejects_inherited_physical_y_for_f64() {
    assert_orthogonal_baseline_subgrid_rejects_inherited_physical_y::<f64>();
}

fn assert_inherited_baseline_group_applies_on_the_same_physical_axis<S: LayoutScalar>()
where
    lts::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = lts::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(70.0)),
                    PreferredSizeOf::px(S::from_f64(80.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(S::from_f64(70.0))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack {
                    name_components: Vec::new(),
                })],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                align_self: Some(AlignItems::Baseline),
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(30.0), S::from_f64(20.0)),
                Size::new(S::from_f64(30.0), S::from_f64(20.0)),
                BaselinesOf {
                    first: Point::new(None, Some(S::from_f64(7.0))),
                    last: Point::new(None, Some(S::from_f64(11.0))),
                },
            ),
        );
    let parent_context = GridParentContext {
        columns: None,
        rows: Some(InheritedGridAxis {
            offset: S::ZERO,
            gap: S::ZERO,
            tracks: vec![S::from_f64(80.0)],
            geometry: UsedGridAxisGeometryOf::new(vec![S::from_f64(80.0)], vec![false], S::ZERO),
            named_lines: named::NamedGridLines::new(GridAxisKind::Row, 1),
            area_facts: None,
            template_area_expanded: false,
            major_baselines: vec![Some(tagged_baseline(
                PhysicalAxis::Vertical,
                S::from_f64(45.0),
            ))],
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
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(70.0)), Some(S::from_f64(80.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(70.0)),
                AvailableOf::definite(S::from_f64(80.0)),
            ),
        ),
        parent_context,
    )
    .expect("parallel subgrid layout succeeds");
    let child = tree.layout(2).expect("subgrid child layout is staged");

    assert_eq!(child.location, Point::new(S::ZERO, S::from_f64(38.0)));
    assert_eq!(
        output.first_baselines,
        Point::new(None, Some(S::from_f64(45.0)))
    );
    assert_eq!(
        output.last_baselines,
        Point::new(None, Some(S::from_f64(49.0)))
    );
}

#[test]
fn baseline_group_axis_applies_compatible_inherited_group_for_f32() {
    assert_inherited_baseline_group_applies_on_the_same_physical_axis::<f32>();
}

#[test]
fn baseline_group_axis_applies_compatible_inherited_group_for_f64() {
    assert_inherited_baseline_group_applies_on_the_same_physical_axis::<f64>();
}

fn assert_intrinsic_baseline_geometry_uses_child_flow_axes<S: LayoutScalar>(
    writing_mode: WritingMode,
    available_span_size: S,
    margin_box_size: S,
    major_baseline: S,
    minor_baseline: S,
) {
    let output = ComputeOutputOf::from_sizes_and_baselines(
        Size::new(S::from_f64(70.0), S::from_f64(110.0)),
        Size::new(S::from_f64(70.0), S::from_f64(110.0)),
        match writing_mode {
            WritingMode::HorizontalTb => BaselinesOf {
                first: Point::new(None, Some(S::from_f64(23.0))),
                last: Point::new(None, Some(S::from_f64(31.0))),
            },
            WritingMode::VerticalRl | WritingMode::SidewaysLr => BaselinesOf {
                first: Point::new(Some(S::from_f64(17.0)), None),
                last: Point::new(Some(S::from_f64(29.0)), None),
            },
            WritingMode::VerticalLr | WritingMode::SidewaysRl => {
                unreachable!("the regression covers one child flow per physical block axis")
            }
        },
    );
    let margin = Edges::new(
        S::from_f64(3.0),
        S::from_f64(7.0),
        S::from_f64(13.0),
        S::from_f64(19.0),
    );

    let flow_axes = crate::geometry::FlowAxes::new(writing_mode, Direction::Ltr);
    assert_eq!(
        baseline_geometry_for_intrinsic_contribution(output, margin, flow_axes,),
        tagged_geometry(
            flow_axes.block_axis(),
            available_span_size,
            margin_box_size,
            major_baseline,
            minor_baseline,
        )
    );
}

#[test]
fn orthogonal_baseline_intrinsic_geometry_uses_child_block_extent_and_line_margins_for_f32() {
    assert_intrinsic_baseline_geometry_uses_child_flow_axes::<f32>(
        WritingMode::HorizontalTb,
        0.0,
        126.0,
        26.0,
        92.0,
    );
    assert_intrinsic_baseline_geometry_uses_child_flow_axes::<f32>(
        WritingMode::VerticalRl,
        0.0,
        96.0,
        24.0,
        60.0,
    );
}

#[test]
fn orthogonal_baseline_intrinsic_geometry_uses_child_block_extent_and_line_margins_for_f64() {
    assert_intrinsic_baseline_geometry_uses_child_flow_axes::<f64>(
        WritingMode::HorizontalTb,
        0.0,
        126.0,
        26.0,
        92.0,
    );
    assert_intrinsic_baseline_geometry_uses_child_flow_axes::<f64>(
        WritingMode::SidewaysLr,
        0.0,
        96.0,
        36.0,
        48.0,
    );
}

#[test]
fn grid_item_axis_uses_physical_progression_for_reversed_start_alignment() {
    let resolved = physical_grid_item_axis(PhysicalGridItemAxis {
        area_size: 100.0,
        size: 20.0,
        margin_start: Some(5.0),
        margin_end: Some(7.0),
        alignment: AlignItems::Start,
        progression: crate::geometry::PhysicalProgression::Decreasing,
    });

    assert_eq!(
        resolved,
        ResolvedGridItemAxis {
            offset: 73.0,
            margin_start: 5.0,
            margin_end: 7.0,
        }
    );
}

#[test]
fn grid_axis_mapping_maps_child_vertical_axes_to_parent_physical_axes() {
    let column = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Column,
        parent_style: &NodeInput::default(),
        child_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
    });
    let row = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Row,
        parent_style: &NodeInput::default(),
        child_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
    });

    assert_eq!(column.parent_axis, GridAxisKind::Row);
    assert_eq!(column.child_axis, GridAxisKind::Column);
    assert_eq!(row.parent_axis, GridAxisKind::Column);
    assert_eq!(row.child_axis, GridAxisKind::Row);
}

#[test]
fn grid_axis_mapping_maps_vertical_parent_axes_to_horizontal_child_physical_axes() {
    let column = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
        child_style: &NodeInput::default(),
    });
    let row = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
        child_style: &NodeInput::default(),
    });

    assert_eq!(column.parent_axis, GridAxisKind::Row);
    assert_eq!(column.child_axis, GridAxisKind::Column);
    assert!(column.reversed);
    assert_eq!(row.parent_axis, GridAxisKind::Column);
    assert_eq!(row.child_axis, GridAxisKind::Row);
    assert!(!row.reversed);
}

#[test]
fn vertical_grid_child_percentage_padding_uses_unequal_physical_area_height_basis() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                grid_template_columns: vec![TrackComponent::from(lp(200.0, 0.0))],
                grid_template_rows: vec![TrackComponent::from(lp(100.0, 0.0))],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(PreferredSize::px(1.0), PreferredSize::px(1.0)),
                padding: Edges::all(Length::percent(0.1)),
                ..NodeInput::default()
            },
        );

    compute_grid(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::definite(200.0)),
        ),
    )
    .unwrap();

    let child = tree.layout(2).expect("grid child layout must be recorded");
    assert_eq!(child.padding, Edges::all(20.0));
    assert_eq!(child.size, Size::new(40.0, 40.0));
}

#[test]
fn orthogonal_subgrid_grandchild_percentage_edges_use_immediate_containing_flow() {
    let parent_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let outer_style = NodeInput {
        display: Display::Grid,
        grid_template_columns: subgrid_track(),
        grid_template_rows: vec![TrackComponent::percent(1.0)],
        ..NodeInput::default()
    };
    let grandchild_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        grid_template_columns: vec![TrackComponent::percent(1.0)],
        grid_template_rows: subgrid_track(),
        margin: Edges::new(
            LengthAuto::percent(0.01),
            LengthAuto::percent(0.02),
            LengthAuto::percent(0.03),
            LengthAuto::percent(0.04),
        ),
        border: Edges::new(
            Length::percent(0.05),
            Length::percent(0.06),
            Length::percent(0.07),
            Length::percent(0.08),
        ),
        padding: Edges::new(
            Length::percent(0.09),
            Length::percent(0.10),
            Length::percent(0.11),
            Length::percent(0.12),
        ),
        ..NodeInput::default()
    };
    let tree = OracleTree::new()
        .children(2, [3])
        .children(3, [4])
        .children(4, [])
        .style(2, outer_style.clone())
        .style(3, grandchild_style)
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
    .expect("eligible orthogonal traversal must produce a report");

    assert_eq!(
        report.leaves[0].available_inline_size,
        Some(164.0),
        "grandchild percentage edges must use the immediate horizontal subgrid flow"
    );
}

#[test]
fn absolute_grid_item_axis_placement_preserves_end_only_first_line() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 3);

    let placement = resolve_absolute_grid_item_axis_placement(
        &lines,
        &RawGridPlacement::new(RawGridLine::Auto, RawGridLine::Line(1)),
        GridPlacement::try_end_line(1).expect("valid grid line"),
    );

    assert_eq!(
        placement,
        GridPlacement::try_end_line(1).expect("valid grid line")
    );
}

#[test]
fn absolute_grid_axis_area_uses_left_edge_for_definite_rtl_range() {
    let tracks = vec![30.0; 8];
    let offsets = rtl_offsets(&tracks, 0.0, 240.0, 0.0, 0.0);
    let geometry = UsedGridAxisGeometryOf::new(tracks.clone(), vec![false; tracks.len()], 0.0);

    let area = absolute_grid_axis_area(AbsoluteGridAxisInput {
        placement: GridPlacement::try_lines(3, 5).expect("valid grid lines"),
        tracks: &tracks,
        offsets: &offsets,
        geometry: &geometry,
        padding_box_location: 0.0,
        padding_box_size: 240.0,
        is_reverse: true,
        explicit_start: 0,
        explicit_count: 8,
    });

    assert_eq!(area.location, 120.0);
    assert_eq!(area.size, 60.0);
}

#[test]
fn subgrid_eligibility_rejects_excluded_children() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            position: Position::Absolute,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report.reason,
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    );
}

#[test]
fn subgrid_eligibility_rejects_display_none_children() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::None,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report.reason,
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    );
}

#[test]
fn subgrid_eligibility_allows_grid_lanes_child_display() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::GridLanes,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(report.eligible);
    assert_eq!(report.reason, None);
}

#[test]
fn subgrid_eligibility_allows_inline_grid_child_display() {
    for display in [Display::InlineGrid, Display::InlineGridLanes] {
        let report = subgrid_eligibility(SubgridEligibilityInput {
            axis: GridAxisKind::Column,
            parent_style: &NodeInput {
                display: Display::Grid,
                ..NodeInput::default()
            },
            has_parent_grid: true,
            child_style: &NodeInput {
                display,
                grid_template_columns: subgrid_track(),
                ..NodeInput::default()
            },
        });

        assert!(report.eligible, "{display:?} should be eligible");
        assert_eq!(report.reason, None);
    }
}

#[test]
fn subgrid_baselines_apply_negative_gap_difference_to_local_baseline_edges() {
    let report = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &[
            Some(tagged_baseline(PhysicalAxis::Vertical, 13.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 20.0)),
        ],
        parent_minor: &[
            Some(tagged_baseline(PhysicalAxis::Vertical, 5.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 20.0)),
        ],
        physical_axis: PhysicalAxis::Vertical,
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: 10.0,
    })
    .unwrap();

    assert_eq!(report.gap_difference, -5.0);
    assert_eq!(
        report.final_major,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 13.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 25.0)),
        ]
    );
    assert_eq!(
        report.final_minor,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 10.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 20.0)),
        ]
    );
}

#[test]
fn subgrid_baselines_reverse_and_adjust_edges() {
    let report = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &[
            Some(tagged_baseline(PhysicalAxis::Vertical, 6.0)),
            None,
            Some(tagged_baseline(PhysicalAxis::Vertical, 14.0)),
        ],
        parent_minor: &[
            Some(tagged_baseline(PhysicalAxis::Vertical, 3.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, 8.0)),
            None,
        ],
        physical_axis: PhysicalAxis::Vertical,
        parent_span: GridTrackSpan::new(1, 4),
        reversed: true,
        start_mbp: 2.0,
        end_mbp: 5.0,
        parent_gap: 12.0,
        subgrid_gap: 12.0,
    })
    .unwrap();

    assert_eq!(
        report.after_reversal_major,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 14.0)),
            None,
            Some(tagged_baseline(PhysicalAxis::Vertical, 6.0)),
        ]
    );
    assert_eq!(
        report.final_major,
        vec![
            Some(tagged_baseline(PhysicalAxis::Vertical, 12.0)),
            None,
            Some(tagged_baseline(PhysicalAxis::Vertical, 6.0)),
        ]
    );
    assert_eq!(
        report.final_minor,
        vec![
            None,
            Some(tagged_baseline(PhysicalAxis::Vertical, 8.0)),
            Some(tagged_baseline(PhysicalAxis::Vertical, -2.0)),
        ]
    );
}

#[test]
fn column_subgrid_context_preserves_inherited_baseline_groups() {
    let parent_style = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        grid_template_columns: subgrid_track(),
        grid_template_rows: vec![TrackComponent::px(20.0)],
        ..NodeInput::default()
    };
    let parent_baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default()],
        columns: vec![
            tagged_group(PhysicalAxis::Horizontal, Some(8.0), Some(3.0)),
            tagged_group(PhysicalAxis::Horizontal, Some(14.0), Some(5.0)),
        ],
    };
    let parent_named_columns = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let parent_named_rows = named::NamedGridLines::new(GridAxisKind::Row, 1);

    let context = subgrid_child_parent_context(SubgridChildParentContextInput {
        item: SubgridItemReport {
            node: (),
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        },
        child_style: &child_style,
        area: GridArea {
            row: 0,
            column: 0,
            row_end: 1,
            column_end: 2,
            size: LogicalSizeOf::new(80.0, 20.0),
        },
        content_box_size: Size::new(80.0, 20.0),
        columns: &[40.0, 40.0],
        rows: &[20.0],
        gap: LogicalSizeOf::new(0.0, 0.0),
        parent_named_columns: &parent_named_columns,
        parent_named_rows: &parent_named_rows,
        parent_area_facts: None,
        parent_baseline_groups: &parent_baseline_groups,
        margin: Edges::all(Some(0.0)),
        border: Edges::ZERO,
        padding: Edges::ZERO,
    })
    .unwrap();

    let columns = context.columns.expect("column subgrid should inherit");
    assert_eq!(
        columns.major_baselines,
        vec![
            Some(tagged_baseline(PhysicalAxis::Horizontal, 8.0)),
            Some(tagged_baseline(PhysicalAxis::Horizontal, 14.0)),
        ]
    );
    assert_eq!(
        columns.minor_baselines,
        vec![
            Some(tagged_baseline(PhysicalAxis::Horizontal, 3.0)),
            Some(tagged_baseline(PhysicalAxis::Horizontal, 5.0)),
        ]
    );
}

#[test]
fn subgrid_child_context_rejects_inheritable_axis_without_parent_tracks() {
    let parent_style = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        grid_template_columns: subgrid_track(),
        ..NodeInput::default()
    };
    let parent_baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default()],
        columns: Vec::new(),
    };
    let parent_named_columns = named::NamedGridLines::new(GridAxisKind::Column, 0);
    let parent_named_rows = named::NamedGridLines::new(GridAxisKind::Row, 1);

    let result = subgrid_child_parent_context(SubgridChildParentContextInput {
        item: SubgridItemReport {
            node: (),
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        },
        child_style: &child_style,
        area: GridArea {
            row: 0,
            column: 0,
            row_end: 1,
            column_end: 1,
            size: LogicalSizeOf::new(0.0, 20.0),
        },
        content_box_size: Size::new(0.0, 20.0),
        columns: &[],
        rows: &[20.0],
        gap: LogicalSizeOf::new(0.0, 0.0),
        parent_named_columns: &parent_named_columns,
        parent_named_rows: &parent_named_rows,
        parent_area_facts: None,
        parent_baseline_groups: &parent_baseline_groups,
        margin: Edges::all(Some(0.0)),
        border: Edges::ZERO,
        padding: Edges::ZERO,
    });

    assert!(matches!(
        result,
        Err(SubgridChildContextError::TrackInheritance(
            SubgridTrackInheritanceError::EmptyTrackList
        ))
    ));

    let error: LayoutError<u32> = subgrid_child_context_container_error(
        10,
        20,
        SubgridChildContextError::TrackInheritance(SubgridTrackInheritanceError::EmptyTrackList),
    );
    assert_eq!(
        error.site(),
        LayoutErrorSite::ContainerSubject {
            container: 10,
            subject: 20,
        }
    );
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InternalInvariant(LayoutInternalInvariant::SubgridTrackInheritance)
    );
}
