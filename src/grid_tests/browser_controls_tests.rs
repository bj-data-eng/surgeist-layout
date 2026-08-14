use super::fixtures::{
    derive_inherited_placement, empty_subgrid_track, fri08_c02_auto_fit_output,
    fri08_c02_auto_fit_repeat, inherited_placement_group, vertical_baseline_measure,
};
use super::*;

mod fri06_c12_t08_browser_front_door {
    use crate as surgeist_layout;

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/layout/browser_parity/support.rs"
    ));
}

fn fri08_c05_later_owned_browser_parity_disposition(relative_path: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let golden = fri06_c12_t08_browser_front_door::Golden::parse_file(path)
        .expect("later-owned browser control parses");
    fri06_c12_t08_browser_front_door::assert_surgeist_matches(&golden)
        .expect_err("later-owned behavior must remain visibly unresolved during FRI-08")
        .to_string()
}

#[test]
fn fri09_c05_control_records_ordinary_grid_baseline_distribution_mismatch() {
    assert_eq!(
        fri08_c05_later_owned_browser_parity_disposition(
            "tests/layout/browser_parity/xml/grid/grid_align_items_baseline_child_multiline_no_override_on_secondline__content_box_ltr.xml",
        ),
        "grid_align_items_baseline_child_multiline_no_override_on_secondline__content_box_ltr/1: y mismatch, expected 68, got 60"
    );
}

#[test]
fn fri10_c05_control_records_grid_absolute_percentage_and_static_position_mismatches() {
    assert_eq!(
        fri08_c05_later_owned_browser_parity_disposition(
            "tests/layout/browser_parity/xml/grid/absolute_correct_cross_child_size_with_percentage__content_box_ltr.xml",
        ),
        "absolute_correct_cross_child_size_with_percentage__content_box_ltr/0: x mismatch, expected 50, got 0"
    );
    assert_eq!(
        fri08_c05_later_owned_browser_parity_disposition(
            "tests/layout/browser_parity/xml/grid/grid_absolute_layout_within_border_static__content_box_ltr.xml",
        ),
        "grid_absolute_layout_within_border_static__content_box_ltr/0: x mismatch, expected 10, got 0"
    );
}

#[test]
fn fri08_c03_intrinsic_checked_in_min_max_container_variants_use_candidate_projection() {
    for family in [
        "grid_lanes_min_content_container_sizing",
        "grid_lanes_max_content_container_sizing",
    ] {
        for variant in [
            "border_box_ltr",
            "border_box_rtl",
            "content_box_ltr",
            "content_box_rtl",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/layout/browser_parity/xml/grid-lanes")
                .join(format!("{family}__{variant}.xml"));
            let golden = fri06_c12_t08_browser_front_door::Golden::parse_file(&path)
                .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
            fri06_c12_t08_browser_front_door::assert_surgeist_matches(&golden)
                .unwrap_or_else(|error| panic!("{} must match: {error}", path.display()));
        }
    }
}

#[test]
fn fri08_c04_standalone_column_autoflow_measures_the_local_grid_container() {
    for variant in [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/layout/browser_parity/xml/subgrid")
            .join(format!(
                "subgrid_standalone_axis_column_autoflow__{variant}.xml"
            ));
        let golden = fri06_c12_t08_browser_front_door::Golden::parse_file(&path)
            .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
        fri06_c12_t08_browser_front_door::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} must match: {error}", path.display()));
    }
}

#[test]
fn fri08_c04_overflow_frozen_grid_and_subgrid_controls_match_public_layout() {
    for (suite, source) in [
        ("grid", "grid_overflow_inline_axis_scroll"),
        ("subgrid", "subgrid_overflow_hidden_does_not_prohibit"),
        (
            "subgrid",
            "subgrid_sibling_overflow_footer_second_matches_first",
        ),
        (
            "subgrid",
            "subgrid_sibling_overflow_footer_third_matches_first",
        ),
    ] {
        for variant in [
            "border_box_ltr",
            "border_box_rtl",
            "content_box_ltr",
            "content_box_rtl",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/layout/browser_parity/xml")
                .join(suite)
                .join(format!("{source}__{variant}.xml"));
            let golden = fri06_c12_t08_browser_front_door::Golden::parse_file(&path)
                .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
            fri06_c12_t08_browser_front_door::assert_surgeist_matches(&golden)
                .unwrap_or_else(|error| panic!("{} must match: {error}", path.display()));
        }
    }
}

#[test]
fn fri08_c06_collapsed_gutter_automatic_and_absolute_spans_use_the_same_line_offsets() {
    let golden = fri06_c12_t08_browser_front_door::Golden::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/layout/browser_parity/xml/grid/fri08_auto_fit_occupied_track_collapse__border_box_ltr.xml"
    )))
    .expect("authoritative collapsed-gutter fixture parses");
    fri06_c12_t08_browser_front_door::assert_surgeist_matches(&golden)
        .expect("automatic span uses the coincident-gutter line offsets");

    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3, 4])
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
                grid_column: GridPlacement::try_line(1).expect("first repeated track"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(3).expect("third repeated track"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_lines(1, 4)
                    .expect("absolute span through the interior collapsed run"),
                grid_row: GridPlacement::try_lines(1, 2).expect("single row"),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );
    let absolute = fri08_c02_auto_fit_output(&tree, Size::new(140.0, 20.0), 4);
    assert_eq!((absolute.location.x, absolute.size.width), (25.0, 90.0));
}

fn assert_fri06_c12_t08_ordinary_fixture_geometry(relative_path: &str) {
    fn clear_browser_control_observations(
        expectation: &mut fri06_c12_t08_browser_front_door::Expectation,
    ) {
        expectation.browser_control = None;
        for child in &mut expectation.children {
            clear_browser_control_observations(child);
        }
    }

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let mut golden = fri06_c12_t08_browser_front_door::Golden::parse_file(path)
        .expect("reviewed T08 fixture parses");
    let _ = (
        fri06_c12_t08_browser_front_door::fixture_files,
        fri06_c12_t08_browser_front_door::fixture_files_in,
        fri06_c12_t08_browser_front_door::fixture_skip_policy_mentions_x_prefix(),
        fri06_c12_t08_browser_front_door::fixture_skip_policy_filters_unsupported_constructs(),
        golden.root.style.display(),
        golden.root.style.width(),
    );
    clear_browser_control_observations(&mut golden.expectations);
    fri06_c12_t08_browser_front_door::assert_surgeist_matches(&golden)
        .unwrap_or_else(|error| panic!("{} ordinary geometry: {error}", golden.name));
}

#[test]
fn horizontal_auto_rows_composed_target_places_later_item_at_y128_without_moving_first_row() {
    assert_fri06_c12_t08_ordinary_fixture_geometry(
        "tests/layout/browser_parity/xml/subgrid/subgrid_baseline_auto_rows_inner_row1_first__border_box_ltr.xml",
    );
}

#[test]
fn rtl_inline_column_composed_target_places_first_item_at_x265() {
    assert_fri06_c12_t08_ordinary_fixture_geometry(
        "tests/layout/browser_parity/xml/subgrid/subgrid_baseline_inline_column_inner_col1_first__border_box_rtl.xml",
    );
}

#[test]
fn ltr_inline_column_composed_target_remains_x265() {
    assert_fri06_c12_t08_ordinary_fixture_geometry(
        "tests/layout/browser_parity/xml/subgrid/subgrid_baseline_inline_column_inner_col1_first__border_box_ltr.xml",
    );
}

#[test]
fn vertical_auto_rows_composed_target_remains_x308() {
    assert_fri06_c12_t08_ordinary_fixture_geometry(
        "tests/layout/browser_parity/xml/subgrid/subgrid_baseline_vertical_auto_rows_inner_row1_first__border_box_ltr.xml",
    );
}

#[test]
fn vertical_auto_rows_current_grid_first_moves_x126_to_x121_while_last_stays_x30() {
    let first_group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::First, 3, 28.0);
    let first = derive_inherited_placement(
        &first_group,
        GridAxisKind::Row,
        AncestorBaselineRole::First,
        3,
        false,
        0.0,
        10.0,
    )
    .unwrap();
    let last_group =
        inherited_placement_group(GridAxisKind::Row, AncestorBaselineRole::Last, 3, 10.0);
    let last = derive_inherited_placement(
        &last_group,
        GridAxisKind::Row,
        AncestorBaselineRole::Last,
        3,
        false,
        0.0,
        10.0,
    )
    .unwrap();
    assert_eq!(
        (
            first.translated_target(),
            126.0 - first.accumulated_gutter_translation(),
            last.translated_target(),
            30.0 - last.accumulated_gutter_translation(),
        ),
        (33.0, 121.0, 10.0, 30.0),
    );
}

#[test]
fn fri06_c12_t08_vertical_auto_rows_preserve_full_fixture_targets() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3, 5, 6, 7])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .children(7, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(600.0), PreferredSize::px(400.0)),
                grid_template_rows: vec![
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                ],
                grid_template_columns: vec![
                    TrackComponent::px(100.0),
                    TrackComponent::px(100.0),
                    TrackComponent::px(100.0),
                    TrackComponent::px(100.0),
                ],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(1).expect("valid first row"),
                grid_column: GridPlacement::try_line(1).expect("valid first column"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(1).expect("valid first row"),
                grid_column: GridPlacement::try_line(2).expect("valid second column"),
                grid_template_rows: vec![empty_subgrid_track()],
                grid_template_columns: vec![TrackComponent::px(100.0)],
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
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(1).expect("valid first row"),
                grid_column: GridPlacement::try_line(3).expect("valid third column"),
                ..NodeInput::default()
            },
        )
        .style(
            6,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid second row"),
                grid_column: GridPlacement::try_line(3).expect("valid third column"),
                ..NodeInput::default()
            },
        )
        .style(
            7,
            NodeInput {
                grid_row: GridPlacement::try_line(3).expect("valid third row"),
                grid_column: GridPlacement::try_line(3).expect("valid third column"),
                ..NodeInput::default()
            },
        )
        .measure(2, vertical_baseline_measure(10.0, 20.0, Some(9.0), None))
        .measure(4, vertical_baseline_measure(10.0, 20.0, Some(1.0), None));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(600.0), Available::Definite(400.0)),
    )
    .expect("vertical auto-row fixture computes");
    round_layout(&mut tree, 1).expect("vertical auto-row fixture rounds");

    assert_eq!(
        [5_u32, 6, 7].map(|node| {
            tree.final_layout(node)
                .expect("auto-row probe is laid out")
                .size
                .width
        }),
        [212.0, 194.0, 194.0],
    );
}
