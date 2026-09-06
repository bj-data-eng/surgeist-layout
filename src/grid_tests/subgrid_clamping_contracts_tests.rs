use super::fixtures::{fri08_c01_placement_compute, fri08_c01_placement_output, subgrid_track_of};
use super::*;

fn named_tracks<S: LayoutScalar>(size: S) -> Vec<TrackComponentOf<S>> {
    let mut tracks = Vec::new();
    for _ in 0..3 {
        tracks.push(TrackComponentOf::line_names(["slot"]));
        tracks.push(TrackComponentOf::px(size));
    }
    tracks.push(TrackComponentOf::line_names(["slot"]));
    tracks
}

fn named_line(index: isize) -> RawGridLine {
    RawGridLine::NamedLine {
        name: "slot".to_string(),
        index,
    }
}

fn clamping_tree<S: LayoutScalar>(
    row_axis: bool,
    numeric: GridPlacement,
    raw: RawGridPlacement,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: named_tracks(S::from_f64(20.0)),
                grid_template_rows: named_tracks(S::from_f64(30.0)),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: subgrid_track_of(),
                grid_template_rows: subgrid_track_of(),
                grid_column: GridPlacement::try_lines(1, 4).unwrap(),
                grid_row: GridPlacement::try_lines(1, 4).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: if row_axis {
                    GridPlacement::try_line(1).unwrap()
                } else {
                    numeric
                },
                grid_row: if row_axis {
                    numeric
                } else {
                    GridPlacement::try_line(1).unwrap()
                },
                raw_grid_column: if row_axis {
                    RawGridPlacement::AUTO
                } else {
                    raw.clone()
                },
                raw_grid_row: if row_axis {
                    raw
                } else {
                    RawGridPlacement::AUTO
                },
                ..NodeInputOf::default()
            },
        )
}

fn assert_subgrid_placement_representations<S: LayoutScalar>() {
    // Each axis inherits exactly three tracks. Positive, negative and named lines
    // all resolve to these same ranges before contextual intersection.
    for row_axis in [false, true] {
        for (start, end, expected_start, expected_span) in [
            (2, 5, 1, 2),
            (5, 7, 2, 1),
            (-7, -5, 0, 1),
            (-5, 3, 0, 2),
            (2, -1, 1, 2),
            (3, 1, 0, 2),
            (4, 4, 2, 1),
        ] {
            let numeric = GridPlacement::try_lines(start, end).unwrap();
            for (representation, numeric, raw) in [
                ("numeric", numeric, RawGridPlacement::AUTO),
                (
                    "raw numeric",
                    GridPlacement::AUTO,
                    RawGridPlacement::new(RawGridLine::Line(start), RawGridLine::Line(end)),
                ),
                (
                    "named",
                    GridPlacement::AUTO,
                    RawGridPlacement::new(named_line(start), named_line(end)),
                ),
            ] {
                let batch =
                    fri08_c01_placement_compute(&clamping_tree::<S>(row_axis, numeric, raw));
                let output = fri08_c01_placement_output(&batch, 3);
                let track_size = if row_axis { 30.0 } else { 20.0 };
                let (location, size) = if row_axis {
                    (output.location.y, output.size.height)
                } else {
                    (output.location.x, output.size.width)
                };
                assert_eq!(
                    location,
                    S::from_f64(track_size * expected_start as f64),
                    "{representation} {start}/{end}, row={row_axis}"
                );
                assert_eq!(
                    size,
                    S::from_f64(track_size * expected_span as f64),
                    "{representation} {start}/{end}, row={row_axis}"
                );
                assert_eq!(
                    fri08_c01_placement_output(&batch, 2).size,
                    Size::new(S::from_f64(60.0), S::from_f64(90.0))
                );
            }
        }
        for (numeric, raw, expected_start, expected_span) in [
            (
                GridPlacement::try_line_span(2, 6).unwrap(),
                RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(6)),
                1,
                2,
            ),
            (
                GridPlacement::try_span_line(6, 3).unwrap(),
                RawGridPlacement::new(RawGridLine::Span(6), RawGridLine::Line(3)),
                0,
                2,
            ),
        ] {
            for (numeric, raw) in [
                (numeric, RawGridPlacement::AUTO),
                (GridPlacement::AUTO, raw),
            ] {
                let batch =
                    fri08_c01_placement_compute(&clamping_tree::<S>(row_axis, numeric, raw));
                let output = fri08_c01_placement_output(&batch, 3);
                let track_size = if row_axis { 30.0 } else { 20.0 };
                let (location, size) = if row_axis {
                    (output.location.y, output.size.height)
                } else {
                    (output.location.x, output.size.width)
                };
                assert_eq!(location, S::from_f64(track_size * expected_start as f64));
                assert_eq!(size, S::from_f64(track_size * expected_span as f64));
            }
        }
    }
}

#[test]
fn subgrid_placement_representations_clamp_equally_f32() {
    assert_subgrid_placement_representations::<f32>();
}

#[test]
fn subgrid_placement_representations_clamp_equally_f64() {
    assert_subgrid_placement_representations::<f64>();
}

fn assert_mixed_subgrid_axes<S: LayoutScalar>() {
    for inherited_rows in [false, true] {
        let tracks = subgrid_track_of();
        let standalone = vec![TrackComponentOf::px(S::from_f64(10.0))];
        let tree = clamping_tree::<S>(inherited_rows, GridPlacement::AUTO, RawGridPlacement::AUTO)
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: if inherited_rows {
                        standalone.clone()
                    } else {
                        tracks.clone()
                    },
                    grid_template_rows: if inherited_rows { tracks } else { standalone },
                    grid_auto_columns: vec![TrackComponentOf::px(S::from_f64(10.0))],
                    grid_auto_rows: vec![TrackComponentOf::px(S::from_f64(10.0))],
                    grid_column: GridPlacement::try_lines(1, 4).unwrap(),
                    grid_row: GridPlacement::try_lines(1, 4).unwrap(),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_column: GridPlacement::try_line(5).unwrap(),
                    grid_row: GridPlacement::try_line(5).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch = fri08_c01_placement_compute(&tree);
        let output = fri08_c01_placement_output(&batch, 3);
        let (location, size) = if inherited_rows {
            (
                Point::new(S::from_f64(40.0), S::from_f64(60.0)),
                Size::new(S::from_f64(10.0), S::from_f64(30.0)),
            )
        } else {
            (
                Point::new(S::from_f64(40.0), S::from_f64(40.0)),
                Size::new(S::from_f64(20.0), S::from_f64(10.0)),
            )
        };
        assert_eq!(output.location, location, "inherited rows={inherited_rows}");
        assert_eq!(output.size, size, "inherited rows={inherited_rows}");
    }
}

#[test]
fn mixed_subgrid_axes_only_materialize_standalone_tracks_f32() {
    assert_mixed_subgrid_axes::<f32>();
}

#[test]
fn mixed_subgrid_axes_only_materialize_standalone_tracks_f64() {
    assert_mixed_subgrid_axes::<f64>();
}

fn assert_nested_subgrid_clamping<S: LayoutScalar>() {
    let tree = clamping_tree::<S>(false, GridPlacement::AUTO, RawGridPlacement::AUTO)
        .children(3, [4, 5])
        .style(
            3,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: subgrid_track_of(),
                grid_template_rows: subgrid_track_of(),
                grid_column: GridPlacement::try_lines(2, 5).unwrap(),
                grid_row: GridPlacement::try_lines(2, 5).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                grid_column: GridPlacement::try_line(5).unwrap(),
                grid_row: GridPlacement::try_line(5).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                raw_grid_column: RawGridPlacement::new(named_line(5), RawGridLine::Auto),
                raw_grid_row: RawGridPlacement::new(named_line(5), RawGridLine::Auto),
                ..NodeInputOf::default()
            },
        );
    let batch = fri08_c01_placement_compute(&tree);
    let nested = fri08_c01_placement_output(&batch, 3);
    assert_eq!(
        nested.location,
        Point::new(S::from_f64(20.0), S::from_f64(30.0))
    );
    assert_eq!(nested.size, Size::new(S::from_f64(40.0), S::from_f64(60.0)));
    for node in [4, 5] {
        let output = fri08_c01_placement_output(&batch, node);
        assert_eq!(
            output.location,
            Point::new(S::from_f64(20.0), S::from_f64(30.0))
        );
        assert_eq!(output.size, Size::new(S::from_f64(20.0), S::from_f64(30.0)));
    }
}

#[test]
fn nested_subgrid_clamping_uses_each_inherited_extent_f32() {
    assert_nested_subgrid_clamping::<f32>();
}

#[test]
fn nested_subgrid_clamping_uses_each_inherited_extent_f64() {
    assert_nested_subgrid_clamping::<f64>();
}
