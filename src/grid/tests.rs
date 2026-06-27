use super::*;
use crate::{
    Baselines, CalcExpression, CalcTerm, GridFlowToleranceOf, GridLine, GridSpan, LayoutCalcStore,
    NoCalcResolver, RawGridLine, RawGridPlacement, SubgridLineNameComponent,
    SubgridLineNameRepeatCount, SubgridTrack, TrackRepetition, TrackSizingOf, WritingMode,
};

fn subgrid_track() -> Vec<TrackComponent> {
    vec![TrackComponent::Subgrid(SubgridTrack {
        name_components: Vec::new(),
    })]
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
fn named_lines_preserve_duplicate_source_order_names() {
    let lines = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["a", "b", "a"]),
            TrackComponent::px(20.0),
        ],
        1,
    )
    .unwrap();

    assert_eq!(lines.named_occurrences("a"), vec![1, 1]);
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
fn named_lines_reject_reserved_explicit_line_names() {
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
fn named_lines_return_empty_local_map_for_subgrid() {
    let lines =
        named::named_lines_from_track_components(GridAxisKind::Row, &subgrid_track(), 2).unwrap();

    assert_eq!(lines.axis, GridAxisKind::Row);
    assert_eq!(lines.explicit_track_count, 2);
    assert!(lines.named_occurrences("anything").is_empty());
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
    let base = named::named_lines_from_track_components(GridAxisKind::Row, &[], 0).unwrap();
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
fn named_grid_placement_context_ignores_non_in_flow_track_requirements() {
    let placements = vec![
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(100).expect("valid grid line"),
            row: GridPlacement::try_line(100).expect("valid grid line"),
            absolute_column: GridPlacement::try_line(100).expect("valid grid line"),
            absolute_row: GridPlacement::try_line(100).expect("valid grid line"),
            in_flow: false,
        },
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(-10).expect("valid grid line"),
            row: GridPlacement::AUTO,
            absolute_column: GridPlacement::try_line(-10).expect("valid grid line"),
            absolute_row: GridPlacement::AUTO,
            in_flow: false,
        },
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(2).expect("valid grid line"),
            row: GridPlacement::try_line(3).expect("valid grid line"),
            absolute_column: GridPlacement::try_line(2).expect("valid grid line"),
            absolute_row: GridPlacement::try_line(3).expect("valid grid line"),
            in_flow: true,
        },
    ];

    assert_eq!(
        grid_track_requirement_from_placements(&placements),
        Size::new(2, 3)
    );
    assert_eq!(
        leading_implicit_tracks_from_placements(&placements, GridAxisKind::Column, 2),
        0
    );
}

#[test]
fn grid_axis_placement_preserves_out_of_range_numeric_lines() {
    let lines = named::named_lines_from_track_components(GridAxisKind::Column, &[], 2).unwrap();

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
    let base = named::named_lines_from_track_components(GridAxisKind::Column, &[], 1).unwrap();
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
        mapping: Ok(GridAxisMappingReport {
            queried_axis: GridAxisKind::Column,
            parent_axis: GridAxisKind::Column,
            child_axis: GridAxisKind::Column,
            reversed: true,
        }),
        eligibility: SubgridEligibility {
            eligible: true,
            reason: None,
        },
    };

    let axis = intrinsic_subgrid_axis_parent_context(
        report,
        GridArea {
            row: 0,
            column: 1,
            row_end: 1,
            column_end: 4,
            size: Size::ZERO,
        },
        Size::ZERO,
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
        named::named_lines_from_track_components(GridAxisKind::Column, &[], 4).unwrap(),
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
        named::named_lines_from_track_components(GridAxisKind::Column, &[], 4).unwrap(),
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
        &NodeInput {
            grid_template_columns: subgrid_track(),
            ..NodeInput::DEFAULT
        },
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
            named::named_lines_from_track_components(GridAxisKind::Column, &[], 4).unwrap(),
            None,
            0,
            2,
            false,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &NodeInput {
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
        },
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
        named::named_lines_from_track_components(GridAxisKind::Column, &[], 4).unwrap(),
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
        &NodeInput {
            grid_template_columns: subgrid_track(),
            grid_template_areas: crate::GridTemplateAreas {
                rows: vec![crate::GridTemplateAreaRow {
                    cells: vec![None, Some("same".to_string()), None],
                }],
            },
            ..NodeInput::DEFAULT
        },
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
        order: 0,
        area: GridArea {
            row,
            column,
            row_end: row + row_span,
            column_end: column + 1,
            size: Size::new(40.0, height),
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
        relative_offset: Point::ZERO,
        first_baseline: first,
        last_baseline: last,
        published_row_baselines: None,
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
        scrollbar_size: Size::ZERO,
        border: Edges::ZERO,
        padding: Edges::ZERO,
        overflow: Point::new(Overflow::Visible, Overflow::Visible),
    }
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
        named_lines,
        area_facts,
        major_baselines: vec![None; track_count],
        minor_baselines: vec![None; track_count],
        parent_start,
        parent_end,
        reversed,
        start_mbp: 0.0,
        end_mbp: 0.0,
        gap_difference: 0.0,
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

fn local_line_names(line_names: &[Vec<named::LineNameEntry>]) -> Vec<Vec<&str>> {
    line_names
        .iter()
        .map(|entries| entry_names(entries))
        .collect()
}

fn entry_names(entries: &[named::LineNameEntry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.name.as_str()).collect()
}

#[test]
fn row_baselines_choose_first_baseline_for_first_group() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 22.0, 30.0),
        baseline_test_item(0, 1, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 2);

    assert_eq!(groups.rows[0].first, Some(14.0));
}

#[test]
fn row_baselines_choose_last_baseline_for_last_group() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::LastBaseline, 8.0, 22.0, 30.0),
        baseline_test_item(0, 1, 2, AlignItems::LastBaseline, 8.0, 18.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 2);

    assert_eq!(groups.rows[1].last, Some(12.0));
}

#[test]
fn row_baselines_keep_first_groups_per_start_row() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 22.0, 30.0),
        baseline_test_item(1, 0, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 1);

    assert_eq!(groups.rows[0].first, Some(8.0));
    assert_eq!(groups.rows[1].first, Some(14.0));
}

#[test]
fn row_baselines_keep_last_groups_per_end_row() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 22.0, 30.0),
        baseline_test_item(1, 0, 1, AlignItems::LastBaseline, 8.0, 18.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 1);

    assert_eq!(groups.rows[0].last, Some(8.0));
    assert_eq!(groups.rows[1].last, Some(12.0));
}

#[test]
fn published_row_baselines_map_reversed_subgrid_back_to_parent_rows() {
    let axis = InheritedGridAxis {
        offset: 0.0,
        gap: 10.0,
        tracks: vec![40.0, 40.0, 40.0],
        named_lines: named::NamedGridLines::new(GridAxisKind::Row, 3),
        area_facts: None,
        major_baselines: vec![None, None, None],
        minor_baselines: vec![None, None, None],
        parent_start: 1,
        parent_end: 4,
        reversed: true,
        start_mbp: 3.0,
        end_mbp: 7.0,
        gap_difference: -2.0,
    };
    let published = publish_row_baseline_groups(
        &[
            TrackBaselineGroup {
                first: Some(10.0),
                last: None,
            },
            TrackBaselineGroup {
                first: Some(20.0),
                last: Some(6.0),
            },
            TrackBaselineGroup {
                first: None,
                last: Some(12.0),
            },
        ],
        &axis,
    );

    assert_eq!(
        published,
        vec![
            PublishedTrackBaselineGroup {
                parent_index: 3,
                group: TrackBaselineGroup {
                    first: Some(11.0),
                    last: None,
                },
            },
            PublishedTrackBaselineGroup {
                parent_index: 2,
                group: TrackBaselineGroup {
                    first: Some(16.0),
                    last: Some(2.0),
                },
            },
            PublishedTrackBaselineGroup {
                parent_index: 1,
                group: TrackBaselineGroup {
                    first: None,
                    last: Some(17.0),
                },
            },
        ]
    );
}

#[test]
fn empty_published_row_baselines_do_not_suppress_item_fallback() {
    let mut item = baseline_test_item(0, 0, 1, AlignItems::Baseline, 9.0, 11.0, 20.0);
    item.published_row_baselines = Some(Vec::new());

    let groups = baseline_groups(&[item], 1, 1);

    assert_eq!(groups.rows[0].first, Some(9.0));
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

    let groups = baseline_groups(&items, 1, 3);

    assert_eq!(groups.columns, vec![TrackBaselineGroup::default(); 3],);
}

#[test]
fn spanned_track_size_counts_tracks_and_internal_gaps() {
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 0, 1, 7.0), 20.0);
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 0, 2, 7.0), 67.0);
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 1, 3, 7.0), 57.0);
}

#[test]
fn baseline_offset_major_uses_margin_box_baseline() {
    let offset = baseline_offset(
        BaselineGroupKind::Major,
        20.0,
        BaselineGeometry {
            available_span_size: 70.0,
            margin_box_size: 40.0,
            major_baseline: 14.0,
            minor_baseline: 12.0,
        },
    );

    assert_eq!(offset, 6.0);
}

#[test]
fn baseline_offset_minor_uses_alignment_context_end() {
    let offset = baseline_offset(
        BaselineGroupKind::Minor,
        18.0,
        BaselineGeometry {
            available_span_size: 70.0,
            margin_box_size: 40.0,
            major_baseline: 14.0,
            minor_baseline: 12.0,
        },
    );

    assert_eq!(offset, 24.0);
}

#[test]
fn baseline_offset_major_allows_row_spanning_gap_area() {
    let offset = baseline_offset(
        BaselineGroupKind::Major,
        14.0,
        BaselineGeometry {
            available_span_size: 90.0,
            margin_box_size: 30.0,
            major_baseline: 8.0,
            minor_baseline: 10.0,
        },
    );

    assert_eq!(offset, 6.0);
}

#[test]
fn baseline_offset_minor_allows_row_spanning_gap_area() {
    let offset = baseline_offset(
        BaselineGroupKind::Minor,
        14.0,
        BaselineGeometry {
            available_span_size: 90.0,
            margin_box_size: 30.0,
            major_baseline: 8.0,
            minor_baseline: 10.0,
        },
    );

    assert_eq!(offset, 56.0);
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
        BaselineGeometry {
            available_span_size: 40.0,
            margin_box_size: 30.0,
            major_baseline: 6.0,
            minor_baseline: 8.0,
        },
        TrackBaselineGroup {
            first: Some(18.0),
            last: Some(12.0),
        },
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
        BaselineGeometry {
            available_span_size: 40.0,
            margin_box_size: 30.0,
            major_baseline: 6.0,
            minor_baseline: 2.0,
        },
        TrackBaselineGroup {
            first: Some(18.0),
            last: Some(12.0),
        },
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
        BaselineGeometry {
            available_span_size: 40.0,
            margin_box_size: 30.0,
            major_baseline: 6.0,
            minor_baseline: 2.0,
        },
        TrackBaselineGroup {
            first: Some(18.0),
            last: Some(12.0),
        },
    );

    assert_eq!(shim, BaselineShim::default());
}

#[test]
fn baseline_shim_for_intrinsic_contribution_synthesized_baseline_participates() {
    let participation = baseline_participation(AlignItems::Baseline, false, false, Baselines::NONE);

    let shim = baseline_shim_for_intrinsic_contribution(
        participation,
        BaselineGeometry {
            available_span_size: 40.0,
            margin_box_size: 30.0,
            major_baseline: 6.0,
            minor_baseline: 2.0,
        },
        TrackBaselineGroup {
            first: Some(18.0),
            last: Some(12.0),
        },
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
    let groups = baseline_groups(&items, 1, 2);

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0], 0.0),
        Some(6.0)
    );
    assert_eq!(
        baseline_aligned_block_offset(&items[1], &groups, &[40.0], 0.0),
        Some(0.0)
    );
}

#[test]
fn baseline_aligned_block_offset_first_spanning_item() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::Baseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 2, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 2, 2);

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0, 40.0], 7.0),
        Some(6.0)
    );
}

#[test]
fn baseline_aligned_block_offset_last_single_row_item() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 1, 2);

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0], 0.0),
        Some(14.0)
    );
    assert_eq!(
        baseline_aligned_block_offset(&items[1], &groups, &[40.0], 0.0),
        Some(10.0)
    );
}

#[test]
fn baseline_aligned_block_offset_last_spanning_item() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 2, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 2, 2);

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0, 40.0], 7.0),
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
    let first_groups = baseline_groups(&first_items, 1, 2);

    assert_eq!(
        baseline_aligned_block_offset(&first_items[0], &first_groups, &[40.0], 0.0),
        Some(6.0)
    );

    let mut last_items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    last_items[0].vertical_axis.margin_start = 3.0;
    last_items[0].vertical_axis.margin_end = 5.0;
    let last_groups = baseline_groups(&last_items, 1, 2);

    assert_eq!(
        baseline_aligned_block_offset(&last_items[0], &last_groups, &[40.0], 0.0),
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
        baseline_aligned_block_offset(&items[0], &groups, &[40.0], 0.0),
        None
    );
}

#[test]
fn baseline_participation_rejects_block_auto_margins() {
    let participation = baseline_participation(AlignItems::Baseline, true, false, Baselines::NONE);

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
    let participation =
        baseline_participation(AlignItems::LastBaseline, false, false, Baselines::NONE);

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

#[test]
fn grid_axis_mapping_supports_horizontal_rtl_reversal() {
    let report = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            direction: Direction::Rtl,
            ..NodeInput::default()
        },
        child_style: &NodeInput::default(),
    })
    .unwrap();

    assert_eq!(report.parent_axis, GridAxisKind::Column);
    assert_eq!(report.child_axis, GridAxisKind::Column);
    assert!(report.reversed);
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
    })
    .unwrap();
    let row = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Row,
        parent_style: &NodeInput::default(),
        child_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
    })
    .unwrap();

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
    })
    .unwrap();
    let row = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
        child_style: &NodeInput::default(),
    })
    .unwrap();

    assert_eq!(column.parent_axis, GridAxisKind::Row);
    assert_eq!(column.child_axis, GridAxisKind::Column);
    assert!(column.reversed);
    assert_eq!(row.parent_axis, GridAxisKind::Column);
    assert_eq!(row.child_axis, GridAxisKind::Row);
    assert!(!row.reversed);
}

#[test]
fn vertical_subgrid_percentage_gap_uses_flow_relative_axis_basis() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalLr,
        gap: Size::new(Length::Percent(0.10), Length::Percent(0.10)),
        ..NodeInput::default()
    };
    let area_size = Size::new(300.0, 500.0);

    assert_eq!(
        child_subgrid_gap(&style, GridAxisKind::Column, area_size, &NoCalcResolver),
        ResolvedSubgridGap::Length(50.0)
    );
    assert_eq!(
        child_subgrid_gap(&style, GridAxisKind::Row, area_size, &NoCalcResolver),
        ResolvedSubgridGap::Length(30.0)
    );
}

#[test]
fn grid_area_physical_origin_maps_vertical_grid_tracks_without_collapsing_rows() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let column_offsets = [0.0, 30.0];
    let row_offsets = [60.0, 0.0];

    assert_eq!(
        grid_area_physical_origin(
            &style,
            &column_offsets,
            &row_offsets,
            GridArea {
                column: 0,
                column_end: 1,
                row: 0,
                row_end: 1,
                size: Size::new(30.0, 50.0),
            },
        ),
        Point::new(60.0, 0.0)
    );
    assert_eq!(
        grid_area_physical_origin(
            &style,
            &column_offsets,
            &row_offsets,
            GridArea {
                column: 1,
                column_end: 2,
                row: 1,
                row_end: 2,
                size: Size::new(40.0, 60.0),
            },
        ),
        Point::new(0.0, 30.0)
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

    let column_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style: &style,
        axis: GridAxisKind::Column,
        tracks: &tracks,
        inherited_offset: Some(100.0),
        content_box_left: 0.0,
        content_box_size: Size::new(300.0, 400.0),
        content_box_inset,
        alignment,
    });
    let row_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style: &style,
        axis: GridAxisKind::Row,
        tracks: &tracks,
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

    let area = absolute_grid_axis_area(AbsoluteGridAxisInput {
        placement: GridPlacement::try_lines(3, 5).expect("valid grid lines"),
        tracks: &tracks,
        offsets: &offsets,
        gap: 0.0,
        padding_box_location: 0.0,
        padding_box_size: 240.0,
        is_reverse: true,
        explicit_start: 0,
        explicit_count: 8,
        reverse_positive_line_offset_adjustment: 0.0,
    });

    assert_eq!(area.location, 120.0);
    assert_eq!(area.size, 60.0);
}

#[test]
fn grid_item_sizing_transfers_min_block_through_aspect_ratio_to_inline_size() {
    let child_style = NodeInput {
        min_size: Size::new(Dimension::AUTO, Dimension::px(50.0)),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let sizing = grid_item_sizing(
        &child_style,
        &NodeInput::default(),
        Size::new(100.0, 100.0),
        Size::splat(Some(100.0)),
        &NoCalcResolver,
    );

    assert_eq!(sizing.known, Size::new(Some(200.0), Some(100.0)));
}

#[test]
fn grid_item_sizing_keeps_inline_stretch_when_min_inline_defines_aspect_ratio() {
    let child_style = NodeInput {
        min_size: Size::new(Dimension::px(50.0), Dimension::AUTO),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let sizing = grid_item_sizing(
        &child_style,
        &NodeInput::default(),
        Size::new(100.0, 100.0),
        Size::splat(Some(100.0)),
        &NoCalcResolver,
    );

    assert_eq!(sizing.known, Size::new(Some(100.0), Some(50.0)));
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
            overflow: Point::new(Overflow::Hidden, Overflow::Visible),
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(report.eligible);
    assert_eq!(report.reason, None);
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
        Ok(GridAxisMappingReport {
            queried_axis: GridAxisKind::Column,
            parent_axis: GridAxisKind::Row,
            child_axis: GridAxisKind::Column,
            reversed: true,
        })
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
        size: Size::ZERO,
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
        &NoCalcResolver,
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
        &NoCalcResolver,
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
        &NoCalcResolver,
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
        &NoCalcResolver,
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
fn subgrid_baselines_apply_negative_gap_difference_to_internal_edges() {
    let report = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &[Some(13.0), Some(20.0)],
        parent_minor: &[Some(5.0), Some(20.0)],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: 10.0,
    })
    .unwrap();

    assert_eq!(report.gap_difference, -5.0);
    assert_eq!(report.final_major, vec![Some(18.0), Some(25.0)]);
    assert_eq!(report.final_minor, vec![Some(10.0), Some(25.0)]);
}

#[test]
fn subgrid_baselines_reverse_and_adjust_edges() {
    let report = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &[Some(6.0), None, Some(14.0)],
        parent_minor: &[Some(3.0), Some(8.0), None],
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
        vec![Some(14.0), None, Some(6.0)]
    );
    assert_eq!(report.final_major, vec![Some(12.0), None, Some(6.0)]);
    assert_eq!(report.final_minor, vec![None, Some(8.0), Some(-2.0)]);
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
            TrackBaselineGroup {
                first: Some(8.0),
                last: Some(3.0),
            },
            TrackBaselineGroup {
                first: Some(14.0),
                last: Some(5.0),
            },
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
            size: Size::new(80.0, 20.0),
        },
        content_box_size: Size::new(80.0, 20.0),
        columns: &[40.0, 40.0],
        rows: &[20.0],
        gap: Size::ZERO,
        parent_named_columns: &parent_named_columns,
        parent_named_rows: &parent_named_rows,
        parent_area_facts: None,
        parent_baseline_groups: &parent_baseline_groups,
        margin: Edges::all(Some(0.0)),
        border: Edges::ZERO,
        padding: Edges::ZERO,
        resolver: &NoCalcResolver,
    });

    let columns = context.columns.expect("column subgrid should inherit");
    assert_eq!(columns.major_baselines, vec![Some(8.0), Some(14.0)]);
    assert_eq!(columns.minor_baselines, vec![Some(3.0), Some(5.0)]);
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

fn traversal_leaf(node: u32, start: usize, end: usize) -> SubgridTraversalChild<u32> {
    SubgridTraversalChild::Leaf(SubgridTraversalLeaf {
        node,
        span_in_parent: GridTrackSpan::new(start, end),
        available_inline_size: None,
        available_inline_size_is_known: false,
    })
}

fn traversal_subgrid(
    node: u32,
    start: usize,
    end: usize,
    children: Vec<SubgridTraversalChild<u32>>,
) -> SubgridTraversalChild<u32> {
    SubgridTraversalChild::Subgrid(SubgridTraversalNode {
        node,
        axis: SubgridTraversalAxis::Inherited,
        reversed: false,
        span_in_parent: GridTrackSpan::new(start, end),
        available_inline_size: None,
        available_inline_size_is_known: false,
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
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 3),
            available_inline_size: None,
            available_inline_size_is_known: false,
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
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(2, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
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
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
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
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(2, 3),
                available_inline_size: None,
                available_inline_size_is_known: false,
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
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
                node: 2,
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(2, 3),
                available_inline_size: None,
                available_inline_size_is_known: false,
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
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
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
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
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
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
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
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 2),
            available_inline_size: None,
            available_inline_size_is_known: false,
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
fn subgrid_traversal_rejects_standalone_subgrid_explicitly() {
    let err = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Standalone,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 2),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: vec![traversal_leaf(2, 1, 2)],
        })],
    })
    .unwrap_err();

    assert_eq!(
        err,
        SubgridTraversalError::StandaloneSubgridTraversalUnsupported
    );
}

#[test]
fn fr_span_contribution_distributes_by_flex_factor() {
    let tracks = [TrackSizing::fr(1.0), TrackSizing::fr(2.0)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [20.0, 40.0]);
}

#[test]
fn fr_span_contribution_subtracts_non_flex_base_tracks() {
    let tracks = [TrackSizing::MIN_CONTENT, TrackSizing::fr(1.0)];
    let mut sizes = [10.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        40.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [10.0, 30.0]);
}

#[test]
fn fr_span_contribution_normalizes_sub_one_factors() {
    let tracks = [TrackSizing::fr(0.2), TrackSizing::fr(0.3)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [24.0, 36.0]);
}

#[test]
fn fr_span_contribution_normalizes_sub_one_factors_after_non_flex_tracks() {
    let tracks = [
        TrackSizing::px(9.0),
        TrackSizing::fr(0.5),
        TrackSizing::fr(0.5),
    ];
    let mut sizes = [0.0, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        18.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [0.0, 4.5, 4.5]);
}

#[test]
fn fr_span_contribution_splits_zero_factors_evenly() {
    let tracks = [TrackSizing::fr(0.0), TrackSizing::fr(0.0)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [30.0, 30.0]);
}

#[test]
fn fr_span_contribution_keeps_indefinite_percent_tracks_for_initial_sizing() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::fit_content(Length::px(20.0)),
        TrackSizing::AUTO,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
        TrackSizing::fr(1.0),
        TrackSizing::fr(2.0),
    ];
    let mut sizes = [0.0; 8];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        160.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 50.0, 100.0]);
}

#[test]
fn fr_span_contribution_reserves_resolved_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::fit_content(Length::px(20.0)),
        TrackSizing::AUTO,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
        TrackSizing::fr(1.0),
        TrackSizing::fr(2.0),
    ];
    let mut sizes = [0.0; 8];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        Some(160.0),
        160.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 39.333332, 78.666664]);
}

#[test]
fn max_content_span_prefers_max_content_track() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
    ];
    let mut sizes = [80.0, 80.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        320.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [80.0, 230.0, 0.0]);
}

#[test]
fn max_content_span_prefers_max_content_track_over_min_content_maximum() {
    let tracks = [
        TrackSizing::MAX_CONTENT,
        TrackSizing::minmax(MinTrackSizing::MAX_CONTENT, MaxTrackSizing::MIN_CONTENT),
    ];
    let mut sizes = [40.0, 20.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        80.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [60.0, 20.0]);
}

#[test]
fn min_content_span_counts_indefinite_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [0.0, 0.0, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MinContent {
            prioritize_min_tracks: false,
        },
        None,
        160.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [42.666668, 42.666668, 0.0, 0.0]);
}

#[test]
fn max_content_span_keeps_indefinite_percent_tracks_for_initial_sizing() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [42.666668, 42.666668, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        320.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [42.666668, 267.3333, 0.0, 0.0]);
}

#[test]
fn max_content_span_reserves_resolved_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [42.666668, 42.666668, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        Some(320.0),
        320.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [42.666668, 203.33333, 0.0, 0.0]);
}

#[test]
fn indefinite_flex_tracks_keep_span_resolved_bases() {
    let tracks = [TrackSizing::fr(1.0), TrackSizing::fr(2.0)];
    let sizes = resolve_tracks(
        &tracks,
        None,
        0.0,
        AlignContent::Start,
        &[20.0, 40.0],
        &NoCalcResolver,
    );

    assert_eq!(sizes, [20.0, 40.0]);
}

#[test]
fn inline_sub_one_flex_tracks_keep_non_spanned_track_proportional_to_used_fraction() {
    let tracks = [
        TrackSizing::fr(0.2),
        TrackSizing::fr(0.3),
        TrackSizing::fr(0.5),
    ];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: None,
        definite_size: None,
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[24.0, 36.0, 0.0],
        max_intrinsic_sizes: &[24.0, 36.0, 0.0],
    });

    assert_eq!(sizes, [24.0, 36.0, 9.0]);
}

#[test]
fn sub_one_flex_track_content_sum_includes_unfilled_fraction() {
    let tracks = [
        TrackSizing::fr(0.2),
        TrackSizing::fr(0.3),
        TrackSizing::fr(0.5),
    ];

    assert_eq!(track_content_sum(&tracks, &[24.0, 36.0, 9.0], 0.0), 78.0);
}

#[test]
fn tracks_shrink_between_min_and_max_bounds() {
    let sizes =
        distribute_tracks_between_bounds(&[40.0, 20.0, 40.0], &[40.0, 40.0, 40.0], 0.0, 110.0);

    assert_eq!(sizes, [40.0, 30.0, 40.0]);
}

#[test]
fn tracks_stop_shrinking_at_minimum_bounds() {
    let sizes =
        distribute_tracks_between_bounds(&[40.0, 20.0, 40.0], &[40.0, 40.0, 40.0], 0.0, 90.0);

    assert_eq!(sizes, [40.0, 20.0, 40.0]);
}

#[test]
fn inline_minmax_tracks_shrink_to_minimum_bounds() {
    let tracks = [
        TrackSizing::px(40.0),
        TrackSizing::minmax(MinTrackSizing::px(20.0), MaxTrackSizing::px(40.0)),
        TrackSizing::px(40.0),
    ];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: Some(90.0),
        definite_size: Some(90.0),
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[0.0, 0.0, 0.0],
        max_intrinsic_sizes: &[0.0, 0.0, 0.0],
    });

    assert_eq!(sizes, [40.0, 20.0, 40.0]);
}

#[test]
fn inline_minmax_tracks_interpolate_inside_bounds() {
    let tracks = [
        TrackSizing::px(40.0),
        TrackSizing::minmax(MinTrackSizing::px(20.0), MaxTrackSizing::px(40.0)),
        TrackSizing::px(40.0),
    ];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: Some(110.0),
        definite_size: Some(110.0),
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[0.0, 0.0, 0.0],
        max_intrinsic_sizes: &[0.0, 0.0, 0.0],
    });

    assert_eq!(sizes, [40.0, 30.0, 40.0]);
}

#[test]
fn inline_minmax_max_content_minimum_overrides_fixed_maximum() {
    let tracks = [TrackSizing::minmax(
        MinTrackSizing::MAX_CONTENT,
        MaxTrackSizing::px(10.0),
    )];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: None,
        definite_size: None,
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[20.0],
        max_intrinsic_sizes: &[40.0],
    });

    assert_eq!(sizes, [40.0]);
}

#[test]
fn inline_minmax_auto_minimum_allows_fixed_maximum() {
    let tracks = [TrackSizing::minmax(
        MinTrackSizing::AUTO,
        MaxTrackSizing::px(10.0),
    )];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: None,
        definite_size: None,
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[20.0],
        max_intrinsic_sizes: &[40.0],
    });

    assert_eq!(sizes, [10.0]);
}

#[test]
fn definite_flex_tracks_respect_larger_base_tracks() {
    let tracks = [
        TrackSizing::px(40.0),
        TrackSizing::fr(1.0),
        TrackSizing::fr(1.0),
    ];
    let sizes = resolve_tracks(
        &tracks,
        Some(200.0),
        0.0,
        AlignContent::Start,
        &[0.0, 100.0, 0.0],
        &NoCalcResolver,
    );

    assert_eq!(sizes, [40.0, 100.0, 60.0]);
}

#[test]
fn grid_calc_percent_track_needs_layout_resolution() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(20.0),
        CalcTerm::percent(0.10),
    ]));
    let track = TrackSizing::new(
        MinTrackSizing::Length(Length::calc(id)),
        MaxTrackSizing::Length(Length::px(100.0)),
    );

    assert!(track.depends_on_basis_with(&store));
    assert_eq!(track.percent_fraction_with(&store), 0.10);
}
