use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

#[path = "browser_parity/support.rs"]
mod support;

#[test]
fn parses_browser_parity_xml() {
    let golden = support::Golden::parse(include_str!(
        "browser_parity/xml/block/block_basic__border_box_ltr.xml"
    ))
    .expect("browser parity fixture should parse");

    assert_eq!(golden.name, "block_basic__border_box_ltr");
    assert!(golden.use_rounding);
    assert_eq!(golden.viewport.width, support::Available::MaxContent);
    assert_eq!(golden.viewport.height, support::Available::MaxContent);
    assert_eq!(golden.viewport.root_context, support::RootContext::Root);
    assert_eq!(golden.root.style.display(), Some("block".to_string()));
    assert_eq!(golden.root.style.width(), Some("50px".to_string()));
    assert_eq!(golden.root.children.len(), 2);
    assert_eq!(golden.expectations.width, Some(50.0));
    assert_eq!(golden.expectations.height, Some(20.0));
    assert_eq!(golden.expectations.children[1].x, Some(0.0));
    assert_eq!(golden.expectations.children[1].y, Some(10.0));
}

fn fri08_c05_adapter_template_areas_xml(name: &str, root_width: &str, head_width: &str) -> String {
    format!(
        concat!(
            "<test name=\"{}\">",
            "<viewport width=\"max-content\" height=\"max-content\"/>",
            "<input><div display=\"grid\" grid-template-areas=\"head head / nav main\" ",
            "grid-auto-columns=\"30px 50px\" grid-auto-rows=\"20px\">",
            "<div grid-column-start=\"head-start\" grid-column-end=\"head-end\" ",
            "grid-row-start=\"head-start\" grid-row-end=\"head-end\"/>",
            "<div grid-column-start=\"main-start\" grid-column-end=\"main-end\" ",
            "grid-row-start=\"main-start\" grid-row-end=\"main-end\"/>",
            "</div></input>",
            "<expectations><node width=\"{}\" height=\"40\">",
            "<node x=\"0\" y=\"0\" width=\"{}\" height=\"20\"/>",
            "<node x=\"30\" y=\"20\" width=\"50\" height=\"20\"/>",
            "</node></expectations>",
            "</test>"
        ),
        name, root_width, head_width
    )
}

#[test]
fn fri08_c05_adapter_template_areas_parse_is_name_variant_and_expectation_independent() {
    let original = support::Golden::parse(&fri08_c05_adapter_template_areas_xml(
        "grid/fri08_template_areas__border_box_ltr",
        "80",
        "80",
    ))
    .expect("finite serialized template areas should parse");
    let mutated = support::Golden::parse(&fri08_c05_adapter_template_areas_xml(
        "renamed/source__content_box_rtl",
        "999",
        "777",
    ))
    .expect("renamed fixture with changed variant and expectations should parse");

    assert_eq!(original.root, mutated.root);
    assert_eq!(
        original.root.style.get("grid-template-areas"),
        Some("head head / nav main")
    );
}

#[test]
fn fri08_c05_adapter_template_areas_reach_public_layout() {
    let golden = support::Golden::parse(&fri08_c05_adapter_template_areas_xml(
        "finite_template_areas_public_layout",
        "80",
        "80",
    ))
    .expect("finite serialized template areas should parse");

    support::assert_surgeist_matches(&golden)
        .expect("serialized template areas should drive area-generated public placement");
}

#[test]
fn fri08_c05_adapter_template_areas_reject_identifier_without_name_start() {
    let golden = support::Golden::parse(concat!(
        "<test name=\"invalid-template-area-ident\">",
        "<viewport width=\"max-content\" height=\"max-content\"/>",
        "<input><div display=\"grid\" grid-template-areas=\"--\"/></input>",
        "<expectations><node/></expectations>",
        "</test>"
    ))
    .expect("well-formed XML should reach the existing adapter consumer");

    assert!(
        support::assert_surgeist_matches(&golden).is_err(),
        "a double hyphen without a following name-start must fail closed"
    );
}

fn fri08_c05_adapter_template_area_ident_xml(value: &str) -> String {
    format!(
        concat!(
            "<test name=\"finite-template-area-ident\">",
            "<viewport width=\"max-content\" height=\"max-content\"/>",
            "<input><div display=\"grid\" grid-template-areas=\"{}\"/></input>",
            "<expectations><node/></expectations>",
            "</test>"
        ),
        value
    )
}

#[test]
fn fri08_c05_adapter_template_area_identifier_boundaries_match_finite_helper() {
    for (case, value) in [
        ("ordinary name", "head"),
        ("underscore name-start", "_"),
        ("one leading hyphen", "-head"),
        ("two leading hyphens with a name-start", "--head"),
        ("finite continuation characters", "head2-_"),
    ] {
        let golden = support::Golden::parse(&fri08_c05_adapter_template_area_ident_xml(value))
            .expect("well-formed XML should reach the existing adapter consumer");
        assert!(
            support::assert_surgeist_matches(&golden).is_ok(),
            "helper-valid {case} {value:?} must reach public layout"
        );
    }

    for (case, value) in [
        ("missing name-start after two hyphens", "--"),
        ("lone hyphen", "-"),
        ("leading digit", "2head"),
        ("reserved identifier", "auto"),
        ("reserved identifier", "default"),
        ("reserved identifier", "inherit"),
        ("reserved identifier", "initial"),
        ("reserved identifier", "none"),
        ("reserved identifier", "revert"),
        ("reserved identifier", "revert-layer"),
        ("reserved identifier", "span"),
        ("reserved identifier", "unset"),
        ("illegal punctuation", "head@"),
    ] {
        let golden = support::Golden::parse(&fri08_c05_adapter_template_area_ident_xml(value))
            .expect("well-formed XML should reach the existing adapter consumer");
        assert!(
            support::assert_surgeist_matches(&golden).is_err(),
            "helper-invalid {case} {value:?} must fail closed"
        );
    }
}

#[test]
fn fri08_c05_adapter_template_areas_reject_malformed_unknown_and_contradictory_values() {
    for value in [
        "",
        "none",
        "auto",
        "head head / main",
        "head head / head main",
        "head @ / nav main",
    ] {
        let xml = format!(
            concat!(
                "<test name=\"invalid-template-areas\">",
                "<viewport width=\"max-content\" height=\"max-content\"/>",
                "<input><div display=\"grid\" grid-template-areas=\"{}\"/></input>",
                "<expectations><node/></expectations>",
                "</test>"
            ),
            value
        );
        let golden = support::Golden::parse(&xml)
            .expect("well-formed XML should reach the existing adapter consumer");
        assert!(
            support::assert_surgeist_matches(&golden).is_err(),
            "explicit grid-template-areas value {value:?} must fail closed"
        );
    }
}

fn fri08_c05_inputs_new_sources() -> [&'static str; 10] {
    [
        "grid/fri08_auto_placement_span_after_occupied.html",
        "grid/fri08_explicit_overlap_no_implicit_growth.html",
        "grid/fri08_fit_content_flex_composition.html",
        "grid/fri08_template_areas_explicit_tracks.html",
        "grid/fri08_auto_fit_occupied_track_collapse.html",
        "grid/fri08_stretch_minmax_auto.html",
        "grid/fri08_duplicate_line_name_token.html",
        "grid/fri08_grid_composition.html",
        "grid-lanes/fri08_nested_indefinite_subgrid.html",
        "subgrid/fri08_standalone_intrinsic_composition.html",
    ]
}

fn fri08_c05_inputs_control_sources() -> [&'static str; 8] {
    [
        "grid/grid_overflow_inline_axis_scroll.html",
        "grid-lanes/grid_lanes_item_containing_block_content_width.html",
        "grid-lanes/grid_lanes_min_content_container_sizing.html",
        "grid-lanes/grid_lanes_max_content_container_sizing.html",
        "subgrid/subgrid_overflow_hidden_does_not_prohibit.html",
        "subgrid/subgrid_sibling_overflow_footer_second_matches_first.html",
        "subgrid/subgrid_sibling_overflow_footer_third_matches_first.html",
        "subgrid/subgrid_standalone_axis_column_autoflow.html",
    ]
}

fn fri08_c05_inputs_prospective_paths<'a>(
    corpus: &'a Path,
    sources: impl IntoIterator<Item = &'a str>,
) -> BTreeSet<PathBuf> {
    sources
        .into_iter()
        .flat_map(|source| {
            let id = source
                .strip_suffix(".html")
                .expect("FRI-08 source should have an HTML extension");
            [
                "border_box_ltr",
                "border_box_rtl",
                "content_box_ltr",
                "content_box_rtl",
            ]
            .into_iter()
            .map(move |variant| corpus.join(format!("xml/{id}__{variant}.xml")))
        })
        .collect()
}

fn manifest_active_output_paths() -> BTreeSet<PathBuf> {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let manifest = std::fs::read_to_string(corpus.join("corpus.toml"))
        .expect("browser parity corpus manifest should read");
    let selected_sources = fri08_c05_inputs_new_sources()
        .into_iter()
        .chain(fri08_c05_inputs_control_sources())
        .collect::<BTreeSet<_>>();
    let active_sources = manifest.split("[[cases]]").filter_map(|record| {
        let source = record.lines().find_map(|line| {
            line.strip_prefix("source = \"")
                .and_then(|value| value.strip_suffix('"'))
        })?;
        (record.lines().any(|line| line == "status = \"active\"")
            && selected_sources.contains(source))
        .then_some(source)
    });

    fri08_c05_inputs_prospective_paths(&corpus, active_sources)
}

#[test]
fn manifest_active_outputs_match_layout() {
    let mut paths = manifest_active_output_paths().into_iter();
    let first = paths
        .next()
        .expect("manifest should declare at least one selected active parity source");
    for path in std::iter::once(first).chain(paths) {
        let golden = support::Golden::parse_file(&path)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", path.display()));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} failed layout comparison: {error}", path.display()));
    }
}

fn fri07_c04_fixture_input_xml(name: &str, root_width: &str, first_width: &str) -> String {
    format!(
        concat!(
            "<test name=\"{}\">",
            "<viewport width=\"max-content\" height=\"max-content\"/>",
            "<input><div display=\"flex\" width=\"20px\" height=\"10px\">",
            "<div flex-item-collapse=\"collapsed\" width=\"10px\" height=\"10px\"/>",
            "<div width=\"20px\" height=\"10px\"/>",
            "</div></input>",
            "<expectations><node width=\"{}\" height=\"10\">",
            "<node x=\"0\" y=\"0\" width=\"{}\" height=\"0\"/>",
            "<node x=\"0\" y=\"0\" width=\"20\" height=\"10\"/>",
            "</node></expectations>",
            "</test>"
        ),
        name, root_width, first_width
    )
}

#[test]
fn fri07_c04_fixture_input_normalized_collapse_is_name_expectation_and_sibling_independent() {
    let original = support::Golden::parse(&fri07_c04_fixture_input_xml(
        "fri07_collapsed_source",
        "20",
        "0",
    ))
    .expect("normalized collapsed input should parse");
    let mutated = support::Golden::parse(&fri07_c04_fixture_input_xml(
        "unrelated_renamed_source",
        "999",
        "777",
    ))
    .expect("renamed fixture with changed expectations should parse");

    assert_eq!(original.root, mutated.root);
    assert_eq!(
        original.root.children[0].style.get("flex-item-collapse"),
        Some("collapsed")
    );
    assert_eq!(
        original.root.children[1].style.get("flex-item-collapse"),
        None
    );
}

#[test]
fn fri07_c04_fixture_input_normalized_collapse_reaches_public_layout_behavior() {
    let golden = support::Golden::parse(&fri07_c04_fixture_input_xml(
        "fixture_input_public_layout",
        "20",
        "0",
    ))
    .expect("normalized collapsed input should parse");

    support::assert_surgeist_matches(&golden)
        .expect("normalized collapsed input should drive collapsed public layout");
}

fn fri07_c04_fixture_sources() -> [&'static str; 6] {
    [
        "flex/fri07_cross_auto_margin_overflow.html",
        "flex/fri07_absolute_auto_margin_insets.html",
        "flex/fri07_intrinsic_flex_basis.html",
        "flex/fri07_collapsed_strut_single_line.html",
        "flex/fri07_collapsed_strut_wrapping.html",
        "flex/fri07_flex_composition.html",
    ]
}

fn fri07_c04_browser_output_paths() -> Vec<PathBuf> {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    fri07_c04_fixture_sources()
        .into_iter()
        .flat_map(|source| {
            let id = source
                .strip_suffix(".html")
                .expect("FRI-07 fixture source should have an HTML extension");
            [
                "border_box_ltr",
                "border_box_rtl",
                "content_box_ltr",
                "content_box_rtl",
            ]
            .into_iter()
            .map({
                let corpus = corpus.clone();
                move |variant| corpus.join(format!("xml/{id}__{variant}.xml"))
            })
        })
        .collect()
}

#[test]
fn fri07_c04_browser_parity_exact_twenty_four_outputs_parse_without_embedded_provenance() {
    let paths = fri07_c04_browser_output_paths();
    assert_eq!(paths.len(), 24);
    assert!(paths.iter().all(|path| path.is_file()));

    for path in paths {
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should read: {error}", path.display()));
        assert!(
            !raw.contains("<!-- generated-by: surgeist-layout-generate "),
            "{} contains embedded provenance",
            path.display()
        );
        let golden = support::Golden::parse(&raw)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", path.display()));
        assert_eq!(golden.name, path.file_stem().unwrap().to_string_lossy());
    }
}

#[test]
fn normalized_flex_collapse_ordinary_variants_match_layout() {
    let paths = fri07_c04_browser_output_paths()
        .into_iter()
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.contains("fri07_cross_auto_margin_overflow")
                || name.contains("fri07_absolute_auto_margin_insets")
                || name.contains("fri07_intrinsic_flex_basis")
        })
        .collect::<Vec<_>>();

    for path in paths {
        let golden = support::Golden::parse_file(&path)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", path.display()));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} failed layout comparison: {error}", path.display()));
    }
}

#[test]
fn normalized_flex_collapse_chrome_variants_match_reviewed_geometry() {
    let paths = fri07_c04_browser_output_paths()
        .into_iter()
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.contains("fri07_collapsed_strut_single_line")
                || name.contains("fri07_collapsed_strut_wrapping")
                || name.contains("fri07_flex_composition")
        })
        .collect::<Vec<_>>();

    for path in paths {
        let golden = support::Golden::parse_file(&path)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", path.display()));
        let expected = if golden
            .name
            .starts_with("fri07_collapsed_strut_single_line__")
        {
            format!("{}/0: y mismatch, expected 0, got 5", golden.name)
        } else if golden.name.starts_with("fri07_collapsed_strut_wrapping__") {
            format!("{}: height mismatch, expected 157, got 68", golden.name)
        } else if golden.name.ends_with("_ltr") {
            format!("{}/0: y mismatch, expected 6, got 27", golden.name)
        } else {
            format!("{}/0: y mismatch, expected 134, got 113", golden.name)
        };
        let actual = support::assert_surgeist_matches(&golden)
            .expect_err("qualified Chrome row must retain its reviewed mismatch")
            .to_string();
        assert_eq!(actual, expected, "{}", path.display());
    }
}

#[test]
fn runs_browser_parity_smoke_fixture_against_surgeist_layout() {
    let golden = support::Golden::parse(include_str!(
        "browser_parity/xml/block/block_basic__border_box_ltr.xml"
    ))
    .expect("browser parity fixture should parse");

    support::assert_surgeist_matches(&golden).expect("surgeist layout should match fixture");
}

fn clear_browser_control_observations(expectation: &mut support::Expectation) {
    expectation.browser_control = None;
    for child in &mut expectation.children {
        clear_browser_control_observations(child);
    }
}

#[test]
fn fri06_c12_t08_representative_xml_has_strict_ordinary_geometry() {
    let fixtures = [
        include_str!(
            "browser_parity/xml/subgrid/subgrid_baseline_inline_column_inner_col1_first__border_box_ltr.xml"
        ),
        include_str!(
            "browser_parity/xml/subgrid/subgrid_baseline_vertical_auto_rows_inner_row1_first__border_box_ltr.xml"
        ),
        include_str!(
            "browser_parity/xml/subgrid/subgrid_baseline_vertical_nested_inner_row1_first__border_box_ltr.xml"
        ),
    ];

    let mut mismatches = Vec::new();
    for fixture in fixtures {
        let golden = support::Golden::parse(fixture)
            .expect("preserved representative subgrid fixture should parse");
        let root_before = format!("{:?}", golden.root);
        let mut expectations = golden.expectations.clone();
        clear_browser_control_observations(&mut expectations);
        let ordinary_geometry = support::Golden {
            expectations,
            ..golden
        };
        assert_eq!(format!("{:?}", ordinary_geometry.root), root_before);

        if let Err(error) = support::assert_surgeist_matches(&ordinary_geometry) {
            mismatches.push(format!("{}: {error}", ordinary_geometry.name));
        }
    }

    assert!(
        mismatches.is_empty(),
        "representative ordinary geometry mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn block_item_boundary_margin_variants_match_browser() {
    let fixtures = [
        include_str!(
            "browser_parity/xml/block/block_align_baseline_child_margin_percent__border_box_ltr.xml"
        ),
        include_str!(
            "browser_parity/xml/block/block_align_baseline_child_margin_percent__border_box_rtl.xml"
        ),
        include_str!(
            "browser_parity/xml/block/block_align_baseline_child_margin_percent__content_box_ltr.xml"
        ),
        include_str!(
            "browser_parity/xml/block/block_align_baseline_child_margin_percent__content_box_rtl.xml"
        ),
    ];

    for fixture in fixtures {
        let golden = support::Golden::parse(fixture).expect("settled block fixture should parse");
        assert_eq!(golden.root.kind, support::NodeKind::Div);
        assert_eq!(golden.root.style.get("display"), Some("flex"));
        assert_eq!(golden.root.children.len(), 2);
        assert_eq!(golden.root.children[1].children.len(), 1);
        assert_eq!(golden.expectations.children.len(), 2);
        assert_eq!(golden.expectations.children[1].children.len(), 1);
        assert_eq!(golden.expectations.children[1].children[0].y, Some(1.0));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} failed layout comparison: {error}", golden.name));
    }
}

#[test]
fn flex_item_order_variants_match_browser() {
    let fixtures = [
        include_str!("browser_parity/xml/flex/fri03_order_modified_flex__border_box_ltr.xml"),
        include_str!("browser_parity/xml/flex/fri03_order_modified_flex__border_box_rtl.xml"),
        include_str!("browser_parity/xml/flex/fri03_order_modified_flex__content_box_ltr.xml"),
        include_str!("browser_parity/xml/flex/fri03_order_modified_flex__content_box_rtl.xml"),
    ];

    for fixture in fixtures {
        let golden = support::Golden::parse(fixture).expect("settled flex-order fixture parses");
        assert_eq!(golden.root.kind, support::NodeKind::Div);
        assert_eq!(golden.root.style.get("display"), Some("flex"));
        assert_eq!(golden.root.children.len(), 4);
        assert_eq!(golden.expectations.children.len(), 4);
        assert!(golden.root.children.iter().all(|child| {
            child.kind == support::NodeKind::Div
                && child.style.get("display") == Some("flex")
                && child.style.width() == Some("20px".to_string())
                && child.style.get("height") == Some("20px")
        }));
        assert_eq!(
            golden
                .root
                .children
                .iter()
                .map(|child| child.style.get("order").unwrap_or("0"))
                .collect::<Vec<_>>(),
            ["2", "-1", "2", "0"]
        );
        assert_eq!(
            (golden.expectations.width, golden.expectations.height),
            (Some(80.0), Some(20.0))
        );
        assert!(golden.expectations.children.iter().all(|child| {
            child.y == Some(0.0) && child.width == Some(20.0) && child.height == Some(20.0)
        }));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} failed layout comparison: {error}", golden.name));
    }
}

#[test]
fn grid_item_order_variants_match_browser() {
    let fixtures = [
        include_str!("browser_parity/xml/grid/fri03_order_modified_grid__border_box_ltr.xml"),
        include_str!("browser_parity/xml/grid/fri03_order_modified_grid__border_box_rtl.xml"),
        include_str!("browser_parity/xml/grid/fri03_order_modified_grid__content_box_ltr.xml"),
        include_str!("browser_parity/xml/grid/fri03_order_modified_grid__content_box_rtl.xml"),
    ];

    for fixture in fixtures {
        let golden = support::Golden::parse(fixture).expect("settled grid-order fixture parses");
        assert_eq!(golden.root.kind, support::NodeKind::Div);
        assert_eq!(golden.root.style.get("display"), Some("grid"));
        assert_eq!(
            golden.root.style.get("grid-template-columns"),
            Some("20px 20px 20px 20px")
        );
        assert_eq!(golden.root.style.get("grid-template-rows"), Some("20px"));
        assert_eq!(golden.root.children.len(), 4);
        assert_eq!(golden.expectations.children.len(), 4);
        assert!(golden.root.children.iter().all(|child| {
            child.kind == support::NodeKind::Div
                && child.style.get("display") == Some("flex")
                && child.style.width() == Some("20px".to_string())
                && child.style.get("height") == Some("20px")
                && child.children.is_empty()
        }));
        assert_eq!(
            golden
                .root
                .children
                .iter()
                .map(|child| child.style.get("order").unwrap_or("0"))
                .collect::<Vec<_>>(),
            ["2", "-1", "2", "0"]
        );
        assert_eq!(
            (golden.expectations.width, golden.expectations.height),
            (Some(80.0), Some(20.0))
        );
        assert!(golden.expectations.children.iter().all(|child| {
            child.children.is_empty()
                && child.y == Some(0.0)
                && child.width == Some(20.0)
                && child.height == Some(20.0)
        }));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} failed layout comparison: {error}", golden.name));
    }
}

#[test]
fn grid_lanes_item_order_variants_match_browser() {
    let fixtures = [
        include_str!(
            "browser_parity/xml/grid-lanes/fri03_order_modified_lanes__border_box_ltr.xml"
        ),
        include_str!(
            "browser_parity/xml/grid-lanes/fri03_order_modified_lanes__border_box_rtl.xml"
        ),
        include_str!(
            "browser_parity/xml/grid-lanes/fri03_order_modified_lanes__content_box_ltr.xml"
        ),
        include_str!(
            "browser_parity/xml/grid-lanes/fri03_order_modified_lanes__content_box_rtl.xml"
        ),
    ];

    for fixture in fixtures {
        let golden =
            support::Golden::parse(fixture).expect("settled grid-lanes order fixture parses");
        assert_eq!(golden.root.kind, support::NodeKind::Div);
        assert_eq!(golden.root.style.get("display"), Some("grid-lanes"));
        assert_eq!(
            golden.root.style.get("grid-template-columns"),
            Some("20px 20px 20px 20px")
        );
        assert_eq!(golden.root.style.get("grid-template-rows"), Some("20px"));
        assert_eq!(golden.root.children.len(), 4);
        assert_eq!(golden.expectations.children.len(), 4);
        assert!(golden.root.children.iter().all(|child| {
            child.kind == support::NodeKind::Div
                && child.style.get("display") == Some("flex")
                && child.style.width() == Some("20px".to_string())
                && child.style.get("height") == Some("20px")
                && child.children.is_empty()
        }));
        assert_eq!(
            golden
                .root
                .children
                .iter()
                .map(|child| child.style.get("order").unwrap_or("0"))
                .collect::<Vec<_>>(),
            ["2", "-1", "2", "0"]
        );
        assert_eq!(
            (golden.expectations.width, golden.expectations.height),
            (Some(80.0), Some(20.0))
        );
        assert!(golden.expectations.children.iter().all(|child| {
            child.children.is_empty() && child.width == Some(20.0) && child.height == Some(20.0)
        }));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} failed layout comparison: {error}", golden.name));
    }
}

#[test]
fn runs_fri_03_box_participation_against_surgeist_layout() {
    let paths = fri_03_fixture_paths(browser_parity_fixture_paths())
        .unwrap_or_else(|error| panic!("FRI-03 fixture matrix is incomplete: {error}"));
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");

    for relative in paths {
        let fixture = corpus_root.join(relative);
        let golden = support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} failed layout comparison: {error}", fixture.display())
        });
    }
}

#[test]
fn fri04_c05_fixture_outputs_parse_and_match_surgeist_layout() {
    let paths = fri04_c05_fixture_paths(browser_parity_fixture_paths())
        .unwrap_or_else(|error| panic!("FRI-04 C05 fixture matrix is incomplete: {error}"));
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");

    for relative in paths {
        let fixture = corpus_root.join(&relative);
        let golden = support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} failed layout comparison: {error}", fixture.display())
        });
    }
}

fn fri05_c06_computed_overflow_paths() -> Vec<PathBuf> {
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity/xml");
    let mut paths = Vec::new();
    for id in [
        "block/fri05_overflow_auto_cross_axis",
        "flex/fri05_overflow_auto_cross_axis",
        "grid/fri05_overflow_auto_cross_axis",
        "grid/fri05_hidden_auto_minimum",
        "grid-lanes/fri05_hidden_auto_minimum",
        "block/fri05_mixed_axis_clip_margin",
        "block/fri05_scrollbar_gutter_stable_both_edges",
        "flex/fri05_nested_zero_axis_overflow",
        "grid/fri05_nested_zero_axis_overflow",
        "grid/fri05_scroll_extent_area_origin",
        "block/fri05_scroll_target_geometry",
    ] {
        for variant in [
            "border_box_ltr",
            "border_box_rtl",
            "content_box_ltr",
            "content_box_rtl",
        ] {
            paths.push(corpus_root.join(format!("{id}__{variant}.xml")));
        }
    }
    paths
}

#[test]
fn fri05_c06_computed_overflow_corpus_outputs_parse() {
    for path in fri05_c06_computed_overflow_paths() {
        support::Golden::parse_file(&path)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", path.display()));
    }
}

#[test]
fn fri05_c06_computed_overflow_corpus_outputs_match_layout() {
    for path in fri05_c06_computed_overflow_paths() {
        let golden = support::Golden::parse_file(&path)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", path.display()));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} failed layout comparison: {error}", path.display()));
    }
}

#[test]
fn fri05_c06_computed_overflow_corpus_outputs_have_centralized_provenance() {
    let report_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity/xml/generation-reports/all.json");
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&report_path)
            .unwrap_or_else(|error| panic!("{} should read: {error}", report_path.display())),
    )
    .unwrap_or_else(|error| panic!("{} should parse: {error}", report_path.display()));
    assert_eq!(
        report["metadata"]["inputs"]["scripts/gentest/test_helper.js"]
            .as_str()
            .map(str::len),
        Some(64)
    );
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    for path in fri05_c06_computed_overflow_paths() {
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should read: {error}", path.display()));
        assert!(
            !raw.contains("generated-by: surgeist-layout-generate"),
            "{} contains embedded provenance",
            path.display()
        );
        let output = path
            .strip_prefix(&corpus)
            .expect("fixture should be under corpus root")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let entries = report["generated"]
            .as_array()
            .expect("generated report bucket")
            .iter()
            .filter(|entry| entry["output"].as_str() == Some(output.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1, "{output}");
        assert_eq!(entries[0]["output_sha256"].as_str().map(str::len), Some(64));
    }
}

#[test]
fn runs_subgrid_relative_rtl_abspos_fixture_against_surgeist_layout() {
    let golden = support::Golden::parse(include_str!(
        "browser_parity/xml/subgrid/subgrid_abspos_relative_rtl_column_3_to_5__border_box_ltr.xml"
    ))
    .expect("browser parity fixture should parse");

    support::assert_surgeist_matches(&golden).expect("surgeist layout should match fixture");
}

#[test]
fn runs_subgrid_alignment_item_fixture_against_surgeist_layout() {
    let golden = support::Golden::parse(include_str!(
        "browser_parity/xml/subgrid/subgrid_alignment_002_center_start_item__border_box_rtl.xml"
    ))
    .expect("browser parity fixture should parse");

    support::assert_surgeist_matches(&golden).expect("surgeist layout should match fixture");
}

#[test]
fn runs_orthogonal_subgrid_alignment_fixture_against_surgeist_layout() {
    let golden = support::Golden::parse(include_str!(
        "browser_parity/xml/subgrid/subgrid_alignment_002_center_start_orthogonal_item__border_box_ltr.xml"
    ))
    .expect("browser parity fixture should parse");

    support::assert_surgeist_matches(&golden).expect("surgeist layout should match fixture");
}

#[test]
fn runs_grid_multiline_baseline_fixture_against_surgeist_layout() {
    let golden = support::Golden::parse(include_str!(
        "browser_parity/xml/grid/grid_align_items_baseline_multiline__border_box_ltr.xml"
    ))
    .expect("browser parity fixture should parse");

    support::assert_surgeist_matches(&golden).expect("surgeist layout should match fixture");
}

#[test]
fn runs_block_calc_width_margin_fixture_family_against_surgeist_layout() {
    assert_calc_fixture_family_matches("block/block_calc_width_margin");
}

#[test]
fn runs_flex_calc_basis_margin_gap_fixture_family_against_surgeist_layout() {
    assert_calc_fixture_family_matches("flex/flex_calc_basis_margin_gap");
}

#[test]
fn runs_grid_calc_track_and_item_margin_fixture_family_against_surgeist_layout() {
    assert_calc_fixture_family_matches("grid/grid_calc_track_and_item_margin");
}

#[test]
fn runs_fri_02_block_axis_families_against_surgeist_layout() {
    assert_axis_fixture_family_matches(
        block_axis_fixture_paths,
        assert_block_axis_fixture_topology,
    );
}

#[test]
fn runs_fri_02_flex_axis_families_against_surgeist_layout() {
    assert_axis_fixture_family_matches(flex_axis_fixture_paths, assert_flex_axis_fixture_topology);
}

#[test]
fn runs_fri_02_grid_axis_families_against_surgeist_layout() {
    assert_axis_fixture_family_matches(grid_axis_fixture_paths, assert_grid_axis_fixture_topology);
}

#[test]
fn runs_fri_02_grid_lanes_axis_families_against_surgeist_layout() {
    assert_axis_fixture_family_matches(
        grid_lanes_axis_fixture_paths,
        assert_grid_lanes_axis_fixture_topology,
    );
}

#[test]
fn runs_fri_02_subgrid_axis_families_against_surgeist_layout() {
    assert_axis_fixture_family_matches(
        subgrid_axis_fixture_paths,
        assert_subgrid_axis_fixture_topology,
    );
}

fn assert_calc_fixture_family_matches(family: &str) {
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity")
        .canonicalize()
        .expect("browser parity fixture root should exist");
    let fixtures = support::fixture_files("xml")
        .expect("fixtures should load")
        .into_iter()
        .map(|fixture| {
            let fixture = fixture.canonicalize().unwrap_or_else(|error| {
                panic!("{} should canonicalize: {error}", fixture.display())
            });
            let relative = fixture.strip_prefix(&corpus_root).unwrap_or_else(|error| {
                panic!(
                    "{} should be under {}: {error}",
                    fixture.display(),
                    corpus_root.display()
                )
            });
            (relative.to_path_buf(), fixture)
        })
        .collect::<Vec<_>>();
    let paths = calc_fixture_family_paths(
        family,
        fixtures.iter().map(|(relative, _)| relative.clone()),
    )
    .unwrap_or_else(|error| panic!("{error}"));

    for fixture in fixtures
        .into_iter()
        .filter_map(|(relative, fixture)| paths.contains(&relative).then_some(fixture))
    {
        let golden = support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} failed layout comparison: {error}", fixture.display())
        });
    }
}

fn calc_fixture_family_paths(
    family: &str,
    candidate_paths: impl IntoIterator<Item = PathBuf>,
) -> Result<BTreeSet<PathBuf>, String> {
    const EXPECTED_VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    let family_basename = Path::new(family)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{family} must have a UTF-8 basename"))?;
    let fixture_prefix = format!("{family_basename}__");
    let fixtures = candidate_paths
        .into_iter()
        .filter(|candidate| {
            candidate
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("xml")
                && candidate
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem.starts_with(&fixture_prefix))
        })
        .collect::<Vec<_>>();
    let discovered = fixtures.iter().cloned().collect::<BTreeSet<_>>();
    let expected = EXPECTED_VARIANTS
        .into_iter()
        .map(|suffix| PathBuf::from("xml").join(format!("{family}__{suffix}.xml")))
        .collect::<BTreeSet<_>>();

    if fixtures.len() != discovered.len() {
        return Err(format!(
            "{family} fixture discovery must not contain duplicate relative paths: {discovered:#?}"
        ));
    }
    if discovered != expected {
        return Err(format!(
            "{family} fixture family must contain exactly the required relative variants: {discovered:#?}"
        ));
    }

    Ok(discovered)
}

fn fri04_c05_expected_paths() -> Vec<PathBuf> {
    const SOURCES: [&str; 3] = [
        "block/fri04_sizing_math_functions",
        "flex/fri04_flex_basis_content",
        "grid/fri04_track_math_functions",
    ];
    const VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    SOURCES
        .into_iter()
        .flat_map(|source| {
            VARIANTS
                .into_iter()
                .map(move |variant| PathBuf::from("xml").join(format!("{source}__{variant}.xml")))
        })
        .collect()
}

fn fri04_c05_fixture_paths(
    candidate_paths: impl IntoIterator<Item = PathBuf>,
) -> Result<BTreeSet<PathBuf>, String> {
    let expected = fri04_c05_expected_paths()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let fixtures = candidate_paths
        .into_iter()
        .filter(|candidate| fri04_c05_fixture_source(candidate).is_some())
        .collect::<Vec<_>>();
    let discovered = fixtures.iter().cloned().collect::<BTreeSet<_>>();

    if fixtures.len() != discovered.len() {
        return Err(format!(
            "FRI-04 C05 fixture discovery must not contain duplicate relative paths: {discovered:#?}"
        ));
    }
    if discovered != expected {
        return Err(format!(
            "FRI-04 C05 fixture matrix must contain exactly the required relative variants: {discovered:#?}"
        ));
    }

    Ok(discovered)
}

fn fri04_c05_fixture_source(path: &Path) -> Option<&str> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("xml") {
        return None;
    }
    let source = path
        .file_stem()
        .and_then(|stem| stem.to_str())?
        .split_once("__")?
        .0;
    [
        "fri04_sizing_math_functions",
        "fri04_flex_basis_content",
        "fri04_track_math_functions",
    ]
    .contains(&source)
    .then_some(source)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri03Family {
    BlockMargin,
    Order,
    FlexItemRoot,
}

fn fri_03_expected_paths() -> Vec<PathBuf> {
    const FAMILIES: [&str; 8] = [
        "block/block_align_baseline_child_margin_percent",
        "flex/fri03_order_modified_flex",
        "grid/fri03_order_modified_grid",
        "grid-lanes/fri03_order_modified_lanes",
        "grid/grid_available_space_greater_than_max_content",
        "grid/grid_available_space_smaller_than_max_content",
        "grid/grid_available_space_smaller_than_min_content",
        "grid/chrome_issue_325928327",
    ];
    const VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    FAMILIES
        .into_iter()
        .flat_map(|family| {
            VARIANTS
                .into_iter()
                .map(move |variant| PathBuf::from("xml").join(format!("{family}__{variant}.xml")))
        })
        .collect()
}

fn fri_03_fixture_paths(
    candidate_paths: impl IntoIterator<Item = PathBuf>,
) -> Result<BTreeSet<PathBuf>, String> {
    let expected = fri_03_expected_paths().into_iter().collect::<BTreeSet<_>>();
    let fixtures = candidate_paths
        .into_iter()
        .filter(|candidate| fri_03_family(candidate).is_some())
        .collect::<Vec<_>>();
    let discovered = fixtures.iter().cloned().collect::<BTreeSet<_>>();

    if fixtures.len() != discovered.len() {
        return Err(format!(
            "FRI-03 fixture discovery must not contain duplicate relative paths: {discovered:#?}"
        ));
    }
    if discovered != expected {
        return Err(format!(
            "FRI-03 fixture matrix must contain exactly the required relative variants: {discovered:#?}"
        ));
    }

    Ok(discovered)
}

fn fri_03_family(path: &Path) -> Option<Fri03Family> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("xml") {
        return None;
    }
    let source = path
        .file_stem()
        .and_then(|stem| stem.to_str())?
        .split_once("__")?
        .0;
    match source {
        "block_align_baseline_child_margin_percent" => Some(Fri03Family::BlockMargin),
        "fri03_order_modified_flex"
        | "fri03_order_modified_grid"
        | "fri03_order_modified_lanes" => Some(Fri03Family::Order),
        "grid_available_space_greater_than_max_content"
        | "grid_available_space_smaller_than_max_content"
        | "grid_available_space_smaller_than_min_content"
        | "chrome_issue_325928327" => Some(Fri03Family::FlexItemRoot),
        _ => None,
    }
}

fn block_axis_fixture_paths(
    candidate_paths: impl IntoIterator<Item = PathBuf>,
) -> Result<BTreeSet<PathBuf>, String> {
    let expected = block_axis_expected_paths()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let fixtures = candidate_paths
        .into_iter()
        .filter(|candidate| {
            candidate
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("xml")
                && candidate
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem.starts_with("block_axes_"))
        })
        .collect::<Vec<_>>();
    let discovered = fixtures.iter().cloned().collect::<BTreeSet<_>>();

    if fixtures.len() != discovered.len() {
        return Err(format!(
            "block-axis fixture discovery must not contain duplicate relative paths: {discovered:#?}"
        ));
    }
    if discovered != expected {
        return Err(format!(
            "block-axis fixture matrix must contain exactly the required relative variants: {discovered:#?}"
        ));
    }

    Ok(discovered)
}

fn flex_axis_fixture_paths(
    candidate_paths: impl IntoIterator<Item = PathBuf>,
) -> Result<BTreeSet<PathBuf>, String> {
    let expected = flex_axis_expected_paths()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let fixtures = candidate_paths
        .into_iter()
        .filter(|candidate| {
            candidate
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("xml")
                && candidate
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem.starts_with("flex_axes_"))
        })
        .collect::<Vec<_>>();
    let discovered = fixtures.iter().cloned().collect::<BTreeSet<_>>();

    if fixtures.len() != discovered.len() {
        return Err(format!(
            "flex-axis fixture discovery must not contain duplicate relative paths: {discovered:#?}"
        ));
    }
    if discovered != expected {
        return Err(format!(
            "flex-axis fixture matrix must contain exactly the required relative variants: {discovered:#?}"
        ));
    }

    Ok(discovered)
}

fn grid_axis_fixture_paths(
    candidate_paths: impl IntoIterator<Item = PathBuf>,
) -> Result<BTreeSet<PathBuf>, String> {
    let expected = grid_axis_expected_paths()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let fixtures = candidate_paths
        .into_iter()
        .filter(|candidate| {
            candidate
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("xml")
                && candidate
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem.starts_with("grid_axes_"))
        })
        .collect::<Vec<_>>();
    let discovered = fixtures.iter().cloned().collect::<BTreeSet<_>>();

    if fixtures.len() != discovered.len() {
        return Err(format!(
            "grid-axis fixture discovery must not contain duplicate relative paths: {discovered:#?}"
        ));
    }
    if discovered != expected {
        return Err(format!(
            "grid-axis fixture matrix must contain exactly the required relative variants: {discovered:#?}"
        ));
    }

    Ok(discovered)
}

fn browser_parity_fixture_paths() -> Vec<PathBuf> {
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity")
        .canonicalize()
        .expect("browser parity fixture root should exist");
    support::fixture_files("xml")
        .expect("fixtures should load")
        .into_iter()
        .map(|fixture| {
            let fixture = fixture.canonicalize().unwrap_or_else(|error| {
                panic!("{} should canonicalize: {error}", fixture.display())
            });
            fixture
                .strip_prefix(&corpus_root)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} should be under {}: {error}",
                        fixture.display(),
                        corpus_root.display()
                    )
                })
                .to_path_buf()
        })
        .collect()
}

fn assert_axis_fixture_family_matches(
    matrix: fn(Vec<PathBuf>) -> Result<BTreeSet<PathBuf>, String>,
    topology: fn(&support::Golden, &Path) -> Result<(), String>,
) {
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity")
        .canonicalize()
        .expect("browser parity fixture root should exist");
    let fixtures = support::fixture_files("xml")
        .expect("fixtures should load")
        .into_iter()
        .map(|fixture| {
            let fixture = fixture.canonicalize().unwrap_or_else(|error| {
                panic!("{} should canonicalize: {error}", fixture.display())
            });
            let relative = fixture.strip_prefix(&corpus_root).unwrap_or_else(|error| {
                panic!(
                    "{} should be under {}: {error}",
                    fixture.display(),
                    corpus_root.display()
                )
            });
            (relative.to_path_buf(), fixture)
        })
        .collect::<Vec<_>>();
    let paths = matrix(
        fixtures
            .iter()
            .map(|(relative, _)| relative.clone())
            .collect(),
    )
    .unwrap_or_else(|error| panic!("{error}"));

    for (relative, fixture) in fixtures {
        if !paths.contains(&relative) {
            continue;
        }
        let golden = support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
        topology(&golden, &relative)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.display()));
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} failed layout comparison: {error}", fixture.display())
        });
    }
}

fn grid_lanes_axis_fixture_paths(
    candidate_paths: Vec<PathBuf>,
) -> Result<BTreeSet<PathBuf>, String> {
    axis_fixture_paths(
        candidate_paths,
        "grid_lanes_axes_",
        grid_lanes_axis_expected_paths(),
        "grid-lanes axis",
    )
}

fn subgrid_axis_fixture_paths(candidate_paths: Vec<PathBuf>) -> Result<BTreeSet<PathBuf>, String> {
    axis_fixture_paths(
        candidate_paths,
        "subgrid_axes_",
        subgrid_axis_expected_paths(),
        "subgrid axis",
    )
}

fn axis_fixture_paths(
    candidate_paths: Vec<PathBuf>,
    prefix: &str,
    expected_paths: Vec<PathBuf>,
    label: &str,
) -> Result<BTreeSet<PathBuf>, String> {
    let expected = expected_paths.into_iter().collect::<BTreeSet<_>>();
    let fixtures = candidate_paths
        .into_iter()
        .filter(|candidate| {
            candidate
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("xml")
                && candidate
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem.starts_with(prefix))
        })
        .collect::<Vec<_>>();
    let discovered = fixtures.iter().cloned().collect::<BTreeSet<_>>();

    if fixtures.len() != discovered.len() {
        return Err(format!(
            "{label} fixture discovery must not contain duplicate relative paths: {discovered:#?}"
        ));
    }
    if discovered != expected {
        return Err(format!(
            "{label} fixture matrix must contain exactly the required relative variants: {discovered:#?}"
        ));
    }

    Ok(discovered)
}

fn flex_axis_expected_paths() -> Vec<PathBuf> {
    const MODES: [&str; 5] = [
        "horizontal_tb",
        "vertical_rl",
        "vertical_lr",
        "sideways_rl",
        "sideways_lr",
    ];
    const DIRECTIONS: [&str; 4] = ["row", "row_reverse", "column", "column_reverse"];
    const VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    MODES
        .into_iter()
        .flat_map(|mode| {
            DIRECTIONS.into_iter().flat_map(move |direction| {
                VARIANTS.into_iter().map(move |variant| {
                    PathBuf::from("xml/flex")
                        .join(format!("flex_axes_{mode}_{direction}__{variant}.xml"))
                })
            })
        })
        .collect()
}

fn grid_axis_expected_paths() -> Vec<PathBuf> {
    const FAMILIES: [&str; 9] = [
        "grid_axes_horizontal_tb_parallel",
        "grid_axes_vertical_rl_parallel",
        "grid_axes_vertical_lr_parallel",
        "grid_axes_sideways_rl_parallel",
        "grid_axes_sideways_lr_parallel",
        "grid_axes_vertical_opposing",
        "grid_axes_sideways_opposing",
        "grid_axes_horizontal_parent_orthogonal_child",
        "grid_axes_vertical_parent_orthogonal_child",
    ];
    const VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    FAMILIES
        .into_iter()
        .flat_map(|family| {
            VARIANTS.into_iter().map(move |variant| {
                PathBuf::from("xml/grid").join(format!("{family}__{variant}.xml"))
            })
        })
        .collect()
}

fn grid_lanes_axis_expected_paths() -> Vec<PathBuf> {
    axis_expected_paths("xml/grid-lanes", "grid_lanes_axes")
}

fn subgrid_axis_expected_paths() -> Vec<PathBuf> {
    axis_expected_paths("xml/subgrid", "subgrid_axes")
}

fn axis_expected_paths(directory: &str, prefix: &str) -> Vec<PathBuf> {
    const FAMILIES: [&str; 9] = [
        "horizontal_tb_parallel",
        "vertical_rl_parallel",
        "vertical_lr_parallel",
        "sideways_rl_parallel",
        "sideways_lr_parallel",
        "vertical_opposing",
        "sideways_opposing",
        "horizontal_parent_orthogonal_child",
        "vertical_parent_orthogonal_child",
    ];
    const VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    FAMILIES
        .into_iter()
        .flat_map(|family| {
            VARIANTS.into_iter().map(move |variant| {
                PathBuf::from(directory).join(format!("{prefix}_{family}__{variant}.xml"))
            })
        })
        .collect()
}

fn assert_flex_axis_fixture_topology(golden: &support::Golden, path: &Path) -> Result<(), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} must have a UTF-8 filename", path.display()))?;
    let family = file_name
        .strip_prefix("flex_axes_")
        .and_then(|name| name.split_once("__"))
        .map(|(family, _)| family)
        .ok_or_else(|| format!("{} must use a flex-axis family filename", path.display()))?;
    let (mode, direction) = [
        "horizontal_tb",
        "vertical_rl",
        "vertical_lr",
        "sideways_rl",
        "sideways_lr",
    ]
    .into_iter()
    .find_map(|mode| {
        family
            .strip_prefix(&format!("{mode}_"))
            .map(|direction| (mode.replace('_', "-"), direction.replace('_', "-")))
    })
    .ok_or_else(|| format!("{file_name} must name a supported writing mode"))?;

    if golden.root.kind != support::NodeKind::Div
        || golden.root.style.get("display") != Some("flex")
        || golden
            .root
            .style
            .get("writing-mode")
            .unwrap_or("horizontal-tb")
            != mode
        || golden.root.style.get("flex-direction").unwrap_or("row") != direction
    {
        return Err("target root must be flex in the named writing mode and direction".to_string());
    }
    if golden.root.children.len() < 2 || golden.expectations.children.len() < 2 {
        return Err("target root must have at least two element children".to_string());
    }
    if golden
        .root
        .children
        .iter()
        .any(|child| child.kind != support::NodeKind::Div)
    {
        return Err("target flex children must remain non-text element nodes".to_string());
    }

    let first = &golden.expectations.children[0];
    let second = &golden.expectations.children[1];
    if (first.width, first.height) == (second.width, second.height) {
        return Err("target children must have unequal physical sizes".to_string());
    }

    Ok(())
}

fn assert_grid_axis_fixture_topology(golden: &support::Golden, path: &Path) -> Result<(), String> {
    let (parent_mode, child_mode) = grid_axis_family_modes(path)?;
    if golden.root.kind != support::NodeKind::Div
        || golden.root.style.get("display") != Some("grid")
        || golden
            .root
            .style
            .get("writing-mode")
            .unwrap_or("horizontal-tb")
            != parent_mode
    {
        return Err("target root must be an ordinary grid in the named writing mode".to_string());
    }
    if golden.root.style.get("grid-template-columns") != Some("30px 40px")
        || golden.root.style.get("grid-template-rows") != Some("50px 60px")
    {
        return Err(
            "target root must use the exact unequal 30px 40px columns and 50px 60px rows"
                .to_string(),
        );
    }
    if golden.root.children.len() != 2 || golden.expectations.children.len() != 2 {
        return Err("target root must have exactly two in-flow element children".to_string());
    }

    for (index, child) in golden.root.children.iter().enumerate() {
        if child.kind != support::NodeKind::Div
            || child.style.get("display") == Some("none")
            || !matches!(child.style.get("position"), None | Some("static"))
            || child.style.get("writing-mode").unwrap_or("horizontal-tb") != child_mode
        {
            return Err(
                "target children must be visible in-flow elements in the named child flow"
                    .to_string(),
            );
        }
        let (column_start, column_end, row_start, row_end) = if index == 0 {
            ("1", "2", "1", "2")
        } else {
            ("2", "3", "2", "3")
        };
        if child.style.get("grid-column-start") != Some(column_start)
            || child.style.get("grid-column-end") != Some(column_end)
            || child.style.get("grid-row-start") != Some(row_start)
            || child.style.get("grid-row-end") != Some(row_end)
        {
            return Err(
                "target children must occupy definite non-overlapping diagonal grid cells"
                    .to_string(),
            );
        }
    }

    let first = &golden.expectations.children[0];
    let second = &golden.expectations.children[1];
    let (first_x, first_y, first_width, first_height) = expectation_rect(first)?;
    let (second_x, second_y, second_width, second_height) = expectation_rect(second)?;
    if first_width <= 0.0
        || first_height <= 0.0
        || second_width <= 0.0
        || second_height <= 0.0
        || first_x < second_x + second_width
            && second_x < first_x + first_width
            && first_y < second_y + second_height
            && second_y < first_y + first_height
    {
        return Err(
            "target child expectations must have positive non-overlapping physical boxes"
                .to_string(),
        );
    }

    Ok(())
}

fn grid_axis_family_modes(path: &Path) -> Result<(&'static str, &'static str), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} must have a UTF-8 filename", path.display()))?;
    let family = file_name
        .strip_prefix("grid_axes_")
        .and_then(|name| name.split_once("__"))
        .map(|(family, _)| family)
        .ok_or_else(|| format!("{} must use a grid-axis family filename", path.display()))?;
    match family {
        "horizontal_tb_parallel" => Ok(("horizontal-tb", "horizontal-tb")),
        "vertical_rl_parallel" => Ok(("vertical-rl", "vertical-rl")),
        "vertical_lr_parallel" => Ok(("vertical-lr", "vertical-lr")),
        "sideways_rl_parallel" => Ok(("sideways-rl", "sideways-rl")),
        "sideways_lr_parallel" => Ok(("sideways-lr", "sideways-lr")),
        "vertical_opposing" => Ok(("vertical-rl", "vertical-lr")),
        "sideways_opposing" => Ok(("sideways-rl", "sideways-lr")),
        "horizontal_parent_orthogonal_child" => Ok(("horizontal-tb", "vertical-rl")),
        "vertical_parent_orthogonal_child" => Ok(("vertical-rl", "horizontal-tb")),
        _ => Err(format!("{file_name} must name a required grid-axis family")),
    }
}

fn assert_grid_lanes_axis_fixture_topology(
    golden: &support::Golden,
    path: &Path,
) -> Result<(), String> {
    let (parent_mode, child_mode) = axis_family_modes(path, "grid_lanes_axes_")?;
    if golden.root.kind != support::NodeKind::Div
        || golden.root.style.get("display") != Some("block")
        || golden.root.children.len() != 2
        || golden.expectations.children.len() != 2
    {
        return Err(
            "target root must contain exactly the columns-lanes and rows-lanes cases".to_string(),
        );
    }

    let columns = &golden.root.children[0];
    let rows = &golden.root.children[1];
    assert_grid_lanes_case(
        columns,
        "50px 60px",
        "30px 40px",
        "column",
        parent_mode,
        child_mode,
        "rows",
    )?;
    assert_grid_lanes_case(
        rows,
        "30px 40px",
        "50px 60px",
        "row",
        parent_mode,
        child_mode,
        "columns",
    )?;
    assert_non_overlapping_positive_children(&golden.expectations.children)?;
    Ok(())
}

fn assert_grid_lanes_case(
    case: &support::Node,
    primary_tracks: &str,
    other_tracks: &str,
    auto_flow: &str,
    parent_mode: &str,
    child_mode: &str,
    primary_axis: &str,
) -> Result<(), String> {
    if case.kind != support::NodeKind::Div
        || case.style.get("display") != Some("grid-lanes")
        || case.style.get("writing-mode").unwrap_or("horizontal-tb") != parent_mode
        || case.style.get(primary_axis_name(primary_axis)) != Some(primary_tracks)
        || case.style.get(other_axis_name(primary_axis)) != Some(other_tracks)
        || case.style.get("grid-auto-flow") != Some(auto_flow)
        || case.children.len() != 2
    {
        return Err("each named lanes case must have definite unequal logical totals".to_string());
    }
    for (index, child) in case.children.iter().enumerate() {
        if child.kind != support::NodeKind::Div
            || child.style.get("display") == Some("none")
            || !matches!(child.style.get("position"), None | Some("static"))
            || child.style.get("writing-mode").unwrap_or("horizontal-tb") != child_mode
        {
            return Err(
                "lanes items must be visible in-flow elements in the named child flow".to_string(),
            );
        }
        let (start, end) = if index == 0 { ("1", "2") } else { ("2", "3") };
        let axis = primary_axis.trim_end_matches('s');
        if child.style.get(&format!("grid-{axis}-start")) != Some(start)
            || child.style.get(&format!("grid-{axis}-end")) != Some(end)
        {
            return Err(
                "lanes items must have definite non-overlapping primary-axis placement".to_string(),
            );
        }
    }
    Ok(())
}

fn primary_axis_name(axis: &str) -> &str {
    match axis {
        "rows" => "grid-template-rows",
        "columns" => "grid-template-columns",
        _ => unreachable!("only grid axes are used by fixture topology"),
    }
}

fn other_axis_name(axis: &str) -> &str {
    match axis {
        "rows" => "grid-template-columns",
        "columns" => "grid-template-rows",
        _ => unreachable!("only grid axes are used by fixture topology"),
    }
}

fn assert_subgrid_axis_fixture_topology(
    golden: &support::Golden,
    path: &Path,
) -> Result<(), String> {
    let (parent_mode, child_mode) = axis_family_modes(path, "subgrid_axes_")?;
    if golden.root.kind != support::NodeKind::Div
        || golden.root.style.get("display") != Some("block")
        || golden.root.children.len() != 2
        || golden.expectations.children.len() != 2
    {
        return Err(
            "target root must contain exactly the columns-subgrid and rows-subgrid cases"
                .to_string(),
        );
    }
    assert_subgrid_case(&golden.root.children[0], "columns", parent_mode, child_mode)?;
    assert_subgrid_case(&golden.root.children[1], "rows", parent_mode, child_mode)?;
    assert_non_overlapping_positive_children(&golden.expectations.children)?;
    Ok(())
}

fn assert_subgrid_case(
    parent: &support::Node,
    inherited_axis: &str,
    parent_mode: &str,
    child_mode: &str,
) -> Result<(), String> {
    if parent.kind != support::NodeKind::Div
        || parent.style.get("display") != Some("grid")
        || parent.style.get("writing-mode").unwrap_or("horizontal-tb") != parent_mode
        || parent.style.get("grid-template-columns") != Some("30px 40px")
        || parent.style.get("grid-template-rows") != Some("50px 60px")
        || parent.children.len() != 1
    {
        return Err("subgrid parent must keep the exact unequal inherited tracks".to_string());
    }
    let subgrid = &parent.children[0];
    if subgrid.kind != support::NodeKind::Div
        || subgrid.style.get("display") != Some("grid")
        || subgrid.style.get("writing-mode").unwrap_or("horizontal-tb") != child_mode
        || subgrid
            .style
            .get(&format!("grid-template-{inherited_axis}"))
            != Some("subgrid")
        || subgrid.children.len() != 2
    {
        return Err("case must expose the named inherited subgrid axis and child flow".to_string());
    }
    for (index, item) in subgrid.children.iter().enumerate() {
        if item.kind != support::NodeKind::Div
            || item.style.get("display") == Some("none")
            || !matches!(item.style.get("position"), None | Some("static"))
        {
            return Err("subgrid items must be visible in-flow elements".to_string());
        }
        let (start, end) = if index == 0 { ("1", "2") } else { ("2", "3") };
        let axis = inherited_axis.trim_end_matches('s');
        if item.style.get(&format!("grid-{axis}-start")) != Some(start)
            || item.style.get(&format!("grid-{axis}-end")) != Some(end)
        {
            return Err("subgrid items must expose inherited-track progression".to_string());
        }
    }
    Ok(())
}

fn assert_non_overlapping_positive_children(
    expectations: &[support::Expectation],
) -> Result<(), String> {
    let rectangles = expectations
        .iter()
        .map(expectation_rect)
        .collect::<Result<Vec<_>, _>>()?;

    for &(x, y, width, height) in &rectangles {
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
        {
            return Err("case expectations must have finite positive physical boxes".to_string());
        }
    }

    for (index, &(x, y, width, height)) in rectangles.iter().enumerate() {
        for &(other_x, other_y, other_width, other_height) in &rectangles[index + 1..] {
            if x < other_x + other_width
                && other_x < x + width
                && y < other_y + other_height
                && other_y < y + height
            {
                return Err(
                    "top-level case expectations must have non-overlapping physical boxes"
                        .to_string(),
                );
            }
        }
    }
    Ok(())
}

fn axis_family_modes(path: &Path, prefix: &str) -> Result<(&'static str, &'static str), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} must have a UTF-8 filename", path.display()))?;
    let family = file_name
        .strip_prefix(prefix)
        .and_then(|name| name.split_once("__"))
        .map(|(family, _)| family)
        .ok_or_else(|| format!("{file_name} must use an axis-family filename"))?;
    match family {
        "horizontal_tb_parallel" => Ok(("horizontal-tb", "horizontal-tb")),
        "vertical_rl_parallel" => Ok(("vertical-rl", "vertical-rl")),
        "vertical_lr_parallel" => Ok(("vertical-lr", "vertical-lr")),
        "sideways_rl_parallel" => Ok(("sideways-rl", "sideways-rl")),
        "sideways_lr_parallel" => Ok(("sideways-lr", "sideways-lr")),
        "vertical_opposing" => Ok(("vertical-rl", "vertical-lr")),
        "sideways_opposing" => Ok(("sideways-rl", "sideways-lr")),
        "horizontal_parent_orthogonal_child" => Ok(("horizontal-tb", "vertical-rl")),
        "vertical_parent_orthogonal_child" => Ok(("vertical-rl", "horizontal-tb")),
        _ => Err(format!("{file_name} must name a required axis family")),
    }
}

fn expectation_rect(expectation: &support::Expectation) -> Result<(f32, f32, f32, f32), String> {
    Ok((
        expectation
            .x
            .ok_or_else(|| "target expectation must have x".to_string())?,
        expectation
            .y
            .ok_or_else(|| "target expectation must have y".to_string())?,
        expectation
            .width
            .ok_or_else(|| "target expectation must have width".to_string())?,
        expectation
            .height
            .ok_or_else(|| "target expectation must have height".to_string())?,
    ))
}

fn block_axis_expected_paths() -> Vec<PathBuf> {
    const FAMILIES: [&str; 5] = [
        "block_axes_horizontal_tb",
        "block_axes_vertical_rl",
        "block_axes_vertical_lr",
        "block_axes_sideways_rl",
        "block_axes_sideways_lr",
    ];
    const VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    FAMILIES
        .into_iter()
        .flat_map(|family| {
            VARIANTS.into_iter().map(move |variant| {
                PathBuf::from("xml/block").join(format!("{family}__{variant}.xml"))
            })
        })
        .collect()
}

fn assert_block_axis_fixture_topology(golden: &support::Golden, path: &Path) -> Result<(), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} must have a UTF-8 filename", path.display()))?;
    let mode = file_name
        .strip_prefix("block_axes_")
        .and_then(|name| name.split_once("__"))
        .map(|(mode, _)| mode.replace('_', "-"))
        .ok_or_else(|| format!("{} must use a block-axis family filename", path.display()))?;
    if golden.root.kind != support::NodeKind::Div
        || golden.root.style.get("display") != Some("block")
        || golden
            .root
            .style
            .get("writing-mode")
            .unwrap_or("horizontal-tb")
            != mode
    {
        return Err("target root must be an ordinary block in the named writing mode".to_string());
    }
    if golden.root.children.len() != 2 || golden.expectations.children.len() != 2 {
        return Err("target root must have exactly two ordinary block children".to_string());
    }
    if golden.root.children.iter().any(|child| {
        child.kind != support::NodeKind::Div || child.style.get("display") != Some("block")
    }) {
        return Err("target children must stay on ordinary block topology".to_string());
    }

    let first = &golden.expectations.children[0];
    let second = &golden.expectations.children[1];
    if (first.width, first.height) == (second.width, second.height) {
        return Err("target children must have unequal physical sizes".to_string());
    }

    let direction = golden.root.style.get("direction").unwrap_or("ltr");
    let margin_attr = match (mode.as_str(), direction) {
        ("horizontal-tb", "ltr") => "margin-left",
        ("horizontal-tb", "rtl") => "margin-right",
        ("vertical-rl" | "vertical-lr" | "sideways-rl", "ltr") => "margin-top",
        ("vertical-rl" | "vertical-lr" | "sideways-rl", "rtl") => "margin-bottom",
        ("sideways-lr", "ltr") => "margin-bottom",
        ("sideways-lr", "rtl") => "margin-top",
        _ => return Err("target root must have an LTR or RTL direction".to_string()),
    };
    if golden.root.children[0].style.get(margin_attr) != Some("13px") {
        return Err(format!(
            "first child must retain its inline-start-sensitive {margin_attr} margin"
        ));
    }

    Ok(())
}

#[test]
fn parses_all_checked_in_browser_parity_xml() {
    let fixtures = support::fixture_files("xml").expect("fixtures should load");
    assert!(
        !fixtures.is_empty(),
        "expected at least one browser parity XML fixture"
    );

    for fixture in fixtures {
        support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
    }
}

#[test]
fn parses_generated_xml_with_provenance_comment() {
    let golden = support::Golden::parse(
        r#"
        <!-- generated-by: surgeist-layout-generate schema=1 source="html/block/basic.html" source-sha256="abc" helper-sha256="def" browser="Chrome/149" -->
        <test name="with-provenance" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input><div /></input>
            <expectations><node x="0" y="0" width="0" height="0" /></expectations>
        </test>
        "#,
    )
    .expect("provenance comments should not break parsing");

    assert_eq!(golden.name, "with-provenance");
}

#[test]
fn all_checked_in_browser_parity_xml_is_comment_free_with_centralized_provenance() {
    let fixtures = support::fixture_files("xml").expect("fixtures should load");
    assert!(
        !fixtures.is_empty(),
        "expected at least one browser parity XML fixture"
    );

    for fixture in fixtures {
        let raw = std::fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("{} should read: {error}", fixture.display()));
        assert!(
            !raw.contains("generated-by: surgeist-layout-generate"),
            "{} contains embedded generated provenance",
            fixture.display()
        );
    }
}

#[test]
fn browser_parity_html_provenance_does_not_reference_local_temp_checkouts() {
    let html_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity/html");
    let fixtures = support::fixture_files_in(&html_root, "html")
        .expect("HTML parity fixtures should be readable");
    let mut offenders = Vec::new();

    for fixture in fixtures {
        let raw = std::fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("{} should read: {error}", fixture.display()));
        for forbidden in ["tmp/servo/"] {
            if raw.contains(forbidden) {
                offenders.push(fixture.display().to_string());
                break;
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "HTML parity provenance must use stable upstream paths, not local temp checkout paths:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn browser_parity_generation_report_sources_are_existing_html() {
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let report = corpus_root.join("xml/generation-reports/all.json");
    let raw = std::fs::read_to_string(&report)
        .unwrap_or_else(|error| panic!("{} should read: {error}", report.display()));
    let report_json: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("{} should parse as JSON: {error}", report.display()));

    assert_report_sources_are_existing_html(&corpus_root, &report_json);
}

#[test]
fn report_source_validation_rejects_escaping_html_sources() {
    let corpus_root = std::env::temp_dir().join(format!(
        "surgeist-layout-report-source-escape-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(corpus_root.join("html")).expect("html dir");
    std::fs::write(corpus_root.join("html/declared.html"), "<!doctype html>")
        .expect("declared HTML fixture");
    let report_json = serde_json::json!({
        "generated": [
            {
                "name": "escape__border_box_ltr",
                "source": "html/../html/declared.html",
                "output": "xml/escape__border_box_ltr.xml",
                "variant": "border_box_ltr"
            }
        ],
        "unsupported": [],
        "expected_fail": [],
        "quarantined": [],
        "failed_to_generate": []
    });

    let error = std::panic::catch_unwind(|| {
        assert_report_sources_are_existing_html(&corpus_root, &report_json);
    })
    .expect_err("escaping HTML report source should fail validation");

    let message = panic_message(&error);
    assert!(
        message.contains("must be a local relative path"),
        "unexpected panic message: {message}"
    );
    std::fs::remove_dir_all(corpus_root).ok();
}

fn assert_report_sources_are_existing_html(corpus_root: &Path, report_json: &serde_json::Value) {
    for bucket in [
        "generated",
        "unsupported",
        "expected_fail",
        "quarantined",
        "failed_to_generate",
    ] {
        let entries = report_json[bucket]
            .as_array()
            .unwrap_or_else(|| panic!("{bucket} report entries should be an array"));
        for entry in entries {
            let source = entry["source"]
                .as_str()
                .unwrap_or_else(|| panic!("{bucket} report entry should include source"));
            if let Some(html_source) = source.strip_prefix("html/") {
                assert_local_relative_path(source, html_source);
                let html_path = corpus_root.join("html").join(html_source);
                assert!(
                    html_path.is_file(),
                    "{bucket} report source {source} is not an existing constrained HTML fixture"
                );
            } else {
                panic!("{bucket} report source {source} should start with html/");
            }
        }
    }
}

fn assert_local_relative_path(context: &str, path: &str) {
    let path = Path::new(path);
    assert!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir)),
        "{context} must be a local relative path"
    );
}

fn panic_message(error: &Box<dyn Any + Send>) -> String {
    if let Some(message) = error.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = error.downcast_ref::<&str>() {
        message.to_string()
    } else {
        "<non-string panic>".to_string()
    }
}

#[test]
fn fixture_discovery_does_not_hide_quarantine_by_filename() {
    assert!(
        !support::fixture_skip_policy_mentions_x_prefix(),
        "quarantine must be manifest-driven, not filename-prefix-driven"
    );
}

#[test]
fn fixture_discovery_does_not_silently_skip_unsupported_constructs() {
    assert!(
        !support::fixture_skip_policy_filters_unsupported_constructs(),
        "unsupported constructs must be reported as buckets, not removed from discovery"
    );
}

#[test]
fn parity_filter_must_match_at_least_one_fixture() {
    let fixtures = vec![
        PathBuf::from("xml/grid/example.xml"),
        PathBuf::from("xml/subgrid/example.xml"),
    ];

    let error = filtered_parity_fixtures(fixtures, Some("typo"))
        .expect_err("a stale or misspelled parity filter should fail visibly");

    assert!(error.contains("SURGEIST_PARITY_FILTER"));
    assert!(error.contains("typo"));
    assert!(error.contains("matched no browser parity XML fixtures"));
}

fn filtered_parity_fixtures(
    fixtures: Vec<PathBuf>,
    filter: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let filtered = fixtures
        .into_iter()
        .filter(|fixture| {
            filter
                .map(|filter| fixture.to_string_lossy().contains(filter))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    if let Some(filter) = filter
        && filtered.is_empty()
    {
        return Err(format!(
            "SURGEIST_PARITY_FILTER `{filter}` matched no browser parity XML fixtures"
        ));
    }

    Ok(filtered)
}

#[test]
#[ignore = "run when validating full Surgeist layout parity against checked-in XML"]
fn runs_all_checked_in_browser_parity_xml() {
    let fixtures = support::fixture_files("xml").expect("fixtures should load");
    assert!(
        !fixtures.is_empty(),
        "expected at least one browser parity XML fixture"
    );

    let mut failures = Vec::new();
    let mut by_suite = BTreeMap::<String, usize>::new();
    let mut by_kind = BTreeMap::<String, usize>::new();
    let filter = std::env::var("SURGEIST_PARITY_FILTER").ok();
    let fixtures = filtered_parity_fixtures(fixtures, filter.as_deref())
        .unwrap_or_else(|error| panic!("{error}"));
    for fixture in fixtures {
        let golden = support::Golden::parse_file(&fixture).expect("fixture should parse");
        if let Err(error) = support::assert_surgeist_matches(&golden) {
            let error = error.to_string();
            *by_suite.entry(suite_name(&fixture)).or_default() += 1;
            *by_kind
                .entry(classified_error_kind(&golden, &error))
                .or_default() += 1;
            failures.push(format!("{} failed: {error}", fixture.display()));
        }
    }

    if !failures.is_empty() {
        let shown = failures.iter().take(80).cloned().collect::<Vec<_>>();
        panic!(
            "{} browser parity fixtures failed:\n\nBy suite:\n{}\n\nBy kind:\n{}\n\nFirst failures:\n{}{}",
            failures.len(),
            count_lines(&by_suite),
            count_lines(&by_kind),
            shown.join("\n"),
            if failures.len() > shown.len() {
                format!("\n... {} more", failures.len() - shown.len())
            } else {
                String::new()
            }
        );
    }
}

fn suite_name(path: &Path) -> String {
    path.components()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|window| {
            (window[0].as_os_str() == "xml").then(|| window[1].as_os_str().to_string_lossy())
        })
        .map(|name| name.into_owned())
        .unwrap_or_else(|| "unknown".to_string())
}

fn error_kind(error: &str) -> String {
    if let Some((_, field)) = error.rsplit_once(": ")
        && let Some((field, _)) = field.split_once(" mismatch")
    {
        return format!("{field} mismatch");
    }
    if let Some((kind, _)) = error.split_once('`') {
        return kind.trim().to_string();
    }
    error.to_string()
}

fn classified_error_kind(_golden: &support::Golden, error: &str) -> String {
    error_kind(error)
}

fn count_lines(counts: &BTreeMap<String, usize>) -> String {
    counts
        .iter()
        .map(|(name, count)| format!("{name}: {count}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn atomic_inline_display_failures_are_not_bucketed_as_unsupported_inline_context() {
    for display in ["inline-block", "inline-grid", "inline-grid-lanes"] {
        let golden = support::Golden::parse(&format!(
            r#"
        <test name="{display}-bucket" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input>
                <div display="block">
                    <div display="{display}" />
                </div>
            </input>
            <expectations>
                <node x="0" y="0" width="0" height="0">
                    <node x="0" y="0" width="0" height="0" />
                </node>
            </expectations>
        </test>
        "#
        ))
        .expect("inline fixture should parse");

        assert_eq!(
            classified_error_kind(&golden, "root/0: width mismatch, expected 10, got 0"),
            "width mismatch",
            "{display} should report the actual mismatch kind"
        );
    }
}

#[test]
fn text_leaf_failures_are_not_bucketed_as_unsupported_inline_context() {
    let golden = support::Golden::parse(
        r#"
        <test name="text-bucket" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input>
                <text display="block">hello</text>
            </input>
            <expectations>
                <node x="0" y="0" width="0" height="0" />
            </expectations>
        </test>
        "#,
    )
    .expect("inline fixture should parse");

    assert_eq!(
        classified_error_kind(&golden, "root/0: width mismatch, expected 10, got 0"),
        "width mismatch"
    );
}

#[test]
fn named_grid_placement_failures_report_actual_kind() {
    let golden = support::Golden::parse(
        r#"
        <test name="named-grid-bucket" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input>
                <div display="grid">
                    <div grid-column-start="a" />
                </div>
            </input>
            <expectations>
                <node x="0" y="0" width="0" height="0">
                    <node x="0" y="0" width="0" height="0" />
                </node>
            </expectations>
        </test>
        "#,
    )
    .expect("named grid fixture should parse");

    assert_eq!(
        classified_error_kind(&golden, "root/0: x mismatch, expected 30, got 0"),
        "x mismatch"
    );
}

fn fri06_c08_recovery_characterization_box_attr(box_sizing: &str) -> &'static str {
    match box_sizing {
        "border_box" => "",
        "content_box" => " box-sizing=\"content-box\"",
        other => panic!("unexpected C08 characterization box sizing {other}"),
    }
}

fn fri06_c08_recovery_characterization_percentage_xml(
    box_sizing: &str,
    range_start: f32,
    atomic_x: u8,
    trailing_range_start: f32,
) -> String {
    let box_attr = fri06_c08_recovery_characterization_box_attr(box_sizing);
    format!(
        r#"<test name="fri06_atomic_inline_percentage_block_size__{box_sizing}_rtl" use-rounding="true">
  <viewport width="max-content" height="max-content"/>
  <input>
    <div source-tag="div" layout-ready-inline-root="true" display="block"{box_attr} direction="rtl" font-family="monospace" font-size="16px" line-height="20px" width="180px" height="80px">
      <text layout-input="inline-text">
        <segment id="0" inline-extent="57.796875" inline-baseline="14.8" inline-line-height="20" bidi-level="1" whitespace-edge="preserve" following-break="prohibited"/>
      </text>
      <div source-tag="span" display="inline-block"{box_attr} direction="rtl" font-family="monospace" font-size="16px" line-height="20px" width="20px" height="50%"/>
      <text layout-input="inline-text">
        <segment id="2" inline-extent="86.703125" inline-baseline="14.8" inline-line-height="20" bidi-level="1" whitespace-edge="preserve" following-break="prohibited"/>
      </text>
      <atomic-placeholder child-index="1" bidi-level="1" following-break="prohibited"/>
    </div>
  </input>
  <expectations>
    <node x="0" y="0" width="180" height="80">
      <node>
        <range-inks>
          <range-ink source_segment_id="0" line_index="0" physical_start_edge="right" start="{range_start}" advance="57.796875"/>
        </range-inks>
      </node>
      <node x="{atomic_x}" y="0" width="20" height="40"/>
      <node>
        <range-inks>
          <range-ink source_segment_id="2" line_index="0" physical_start_edge="right" start="{trailing_range_start}" advance="86.703125"/>
        </range-inks>
      </node>
    </node>
  </expectations>
</test>"#
    )
}

fn fri06_c08_recovery_characterization_percentage_browser_rows() -> [String; 2] {
    ["border_box", "content_box"].map(|box_sizing| {
        fri06_c08_recovery_characterization_percentage_xml(box_sizing, 73.296875, 73, 180.0)
    })
}

fn fri06_c08_recovery_characterization_vertical_xml(
    box_sizing: &str,
    direction: &str,
    first_atomic_x: u8,
    second_atomic_x: u8,
    clear_x: u8,
) -> String {
    let box_attr = fri06_c08_recovery_characterization_box_attr(box_sizing);
    let (float_y, first_y, second_y, clear_y, bidi_level) = match direction {
        "ltr" => (0, 36, 36, 0, 0),
        "rtl" => (104, 62, 74, 122, 1),
        other => panic!("unexpected C08 characterization direction {other}"),
    };
    format!(
        r#"<test name="fri06_vertical_break_clear__{box_sizing}_{direction}" use-rounding="true">
  <viewport width="max-content" height="max-content"/>
  <input>
    <div source-tag="div" display="block"{box_attr} direction="{direction}" writing-mode="vertical-rl" overflow-x="auto" overflow-y="auto" scrollbar-width="15" font-family="monospace" font-size="16px" line-height="24px" width="96px" height="140px">
      <div source-tag="span" display="block"{box_attr} direction="{direction}" writing-mode="vertical-rl" float="inline-start" font-family="monospace" font-size="16px" line-height="24px" width="28px" height="36px"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" writing-mode="vertical-rl" font-family="monospace" font-size="16px" line-height="24px" width="18px" height="42px"/>
      <div source-tag="br" line-control="forced-break" display="inline"{box_attr} direction="{direction}" writing-mode="vertical-rl" font-family="monospace" font-size="16px" line-height="24px" inline-baseline="16.8px" inline-line-height="24px"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" writing-mode="vertical-rl" font-family="monospace" font-size="16px" line-height="24px" width="18px" height="30px"/>
      <div source-tag="span" display="block"{box_attr} direction="{direction}" writing-mode="vertical-rl" clear="inline-start" font-family="monospace" font-size="16px" line-height="24px" width="18px" height="18px"/>
      <atomic-placeholder child-index="1" bidi-level="{bidi_level}" following-break="prohibited"/>
      <atomic-placeholder child-index="3" bidi-level="{bidi_level}" following-break="prohibited"/>
    </div>
  </input>
  <expectations>
    <node x="0" y="0" width="96" height="140" scroll_width="0" scroll_height="0">
      <node x="68" y="{float_y}" width="28" height="36"/>
      <node x="{first_atomic_x}" y="{first_y}" width="18" height="42"/>
      <node>
        <browser-control source_index="2" terminal_visual_slot="2" previous_line="same" next_line="later"/>
      </node>
      <node x="{second_atomic_x}" y="{second_y}" width="18" height="30"/>
      <node x="{clear_x}" y="{clear_y}" width="18" height="18"/>
    </node>
  </expectations>
</test>"#
    )
}

fn fri06_c08_recovery_characterization_vertical_browser_rows() -> Vec<String> {
    ["border_box", "content_box"]
        .into_iter()
        .flat_map(|box_sizing| {
            ["ltr", "rtl"].map(|direction| {
                fri06_c08_recovery_characterization_vertical_xml(box_sizing, direction, 75, 51, 30)
            })
        })
        .collect()
}

fn fri06_c08_recovery_characterization_float_xml(box_sizing: &str, direction: &str) -> String {
    let box_attr = fri06_c08_recovery_characterization_box_attr(box_sizing);
    let (bidi_level, edge, range_start, atomic_x) = match direction {
        "ltr" => (0, "left", 42, [81, 42, 74, 0]),
        "rtl" => (1, "right", 130, [63, 98, 62, 90]),
        other => panic!("unexpected C08 characterization direction {other}"),
    };
    format!(
        r#"<test name="fri06_float_line_exclusion__{box_sizing}_{direction}" use-rounding="true">
  <viewport width="max-content" height="max-content"/>
  <input>
    <div source-tag="div" layout-ready-inline-root="true" display="block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="180px">
      <div source-tag="span" display="block"{box_attr} direction="{direction}" float="left" font-family="monospace" font-size="16px" line-height="20px" width="42px" height="42px"/>
      <div source-tag="span" display="block"{box_attr} direction="{direction}" float="right" font-family="monospace" font-size="16px" line-height="20px" width="50px" height="62px"/>
      <text layout-input="inline-text">
        <segment id="4" inline-extent="38.53125" inline-baseline="14.8" inline-line-height="20" bidi-level="{bidi_level}" whitespace-edge="preserve" following-break="allowed"/>
      </text>
      <inline-boundary kind="start" inline-baseline="12" inline-line-height="20"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="28px" height="16px"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="32px" height="16px"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="36px" height="16px"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="40px" height="16px"/>
      <atomic-placeholder child-index="4" bidi-level="0" following-break="allowed"/>
      <atomic-placeholder child-index="5" bidi-level="0" following-break="allowed"/>
      <atomic-placeholder child-index="6" bidi-level="0" following-break="allowed"/>
      <atomic-placeholder child-index="7" bidi-level="0" following-break="prohibited"/>
    </div>
  </input>
  <expectations>
    <node x="0" y="0" width="180" height="63">
      <node x="0" y="0" width="42" height="42"/>
      <node x="130" y="0" width="50" height="62"/>
      <node>
        <range-inks>
          <range-ink source_segment_id="4" line_index="0" physical_start_edge="{edge}" start="{range_start}" advance="38.53125"/>
        </range-inks>
      </node>
      <node x="{}" y="0" width="28" height="16"/>
      <node x="{}" y="21" width="32" height="16"/>
      <node x="{}" y="21" width="36" height="16"/>
      <node x="{}" y="42" width="40" height="16"/>
    </node>
  </expectations>
</test>"#,
        atomic_x[0], atomic_x[1], atomic_x[2], atomic_x[3]
    )
}

fn fri06_c08_recovery_characterization_float_browser_rows() -> Vec<String> {
    ["border_box", "content_box"]
        .into_iter()
        .flat_map(|box_sizing| {
            ["ltr", "rtl"].map(|direction| {
                fri06_c08_recovery_characterization_float_xml(box_sizing, direction)
            })
        })
        .collect()
}

#[test]
fn fri06_c08_recovery_characterization_direct_rtl_browser_geometry_matches_both_box_models() {
    for xml in fri06_c08_recovery_characterization_percentage_browser_rows() {
        let golden =
            support::Golden::parse(&xml).expect("exact C08 direct RTL fixture should parse");
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} browser geometry mismatch: {error}\n{xml}", golden.name)
        });
    }
}

#[test]
fn fri06_c08_recovery_characterization_vertical_browser_geometry_matches_all_variants() {
    let rows = fri06_c08_recovery_characterization_vertical_browser_rows();
    assert_eq!(rows.len(), 4);
    for xml in rows {
        let golden = support::Golden::parse(&xml).expect("exact C08 vertical fixture should parse");
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} browser geometry mismatch: {error}\n{xml}", golden.name)
        });
    }
}

#[test]
fn fri06_c08_recovery_characterization_float_browser_geometry_matches_all_variants() {
    let rows = fri06_c08_recovery_characterization_float_browser_rows();
    assert_eq!(rows.len(), 4);
    let mut mismatches = Vec::new();
    for xml in rows {
        let golden = support::Golden::parse(&xml).expect("exact C08 float fixture should parse");
        if let Err(error) = support::assert_surgeist_matches(&golden) {
            mismatches.push(format!("{}: {error}", golden.name));
        }
    }
    assert!(
        mismatches.is_empty(),
        "C08R browser geometry mismatches:\n{}",
        mismatches.join("\n")
    );
}

fn fri06_c08_recovery_inputs_shape_xml() -> String {
    r#"<test name="fri06_float_shape_exclusion__border_box_ltr" use-rounding="true">
  <viewport width="max-content" height="max-content"/>
  <input>
    <div source-tag="div" layout-ready-inline-root="true" display="block" direction="ltr" overflow-x="auto" overflow-y="auto" scrollbar-width="15" font-family="monospace" font-size="16px" line-height="20px" width="180px">
      <div source-tag="span" display="block" float="left" float-exclusion="shape" width="44px" height="60px">
        <shape-bands>
          <shape-band band-minimum="0" band-maximum="21.2" interval-minimum="0" interval-maximum="44"/>
          <shape-band band-minimum="21.2" band-maximum="37.2" interval-minimum="0" interval-maximum="44"/>
        </shape-bands>
      </div>
      <text layout-input="inline-text">
        <segment id="2" inline-extent="48.171875" inline-baseline="14.8" inline-line-height="20" bidi-level="0" whitespace-edge="preserve" following-break="prohibited"/>
      </text>
      <div source-tag="span" display="inline-block" width="34px" height="16px"/>
      <div source-tag="span" display="inline-block" width="38px" height="16px"/>
      <div source-tag="span" display="inline-block" width="42px" height="16px"/>
      <div source-tag="span" display="inline-block" width="46px" height="16px"/>
      <atomic-placeholder child-index="2" bidi-level="0" following-break="prohibited"/>
      <atomic-placeholder child-index="3" bidi-level="0" following-break="allowed"/>
      <atomic-placeholder child-index="4" bidi-level="0" following-break="prohibited"/>
      <atomic-placeholder child-index="5" bidi-level="0" following-break="prohibited"/>
    </div>
  </input>
  <expectations>
    <node x="0" y="0" width="180" height="60" scroll_width="0" scroll_height="0">
      <node x="0" y="0" width="44" height="60"/>
      <node>
        <range-inks>
          <range-ink source_segment_id="2" line_index="0" physical_start_edge="left" start="44" advance="48.171875"/>
        </range-inks>
      </node>
      <node x="92" y="0" width="34" height="16"/>
      <node x="126" y="0" width="38" height="16"/>
      <node x="44" y="21" width="42" height="16"/>
      <node x="86" y="21" width="46" height="16"/>
    </node>
  </expectations>
</test>"#
        .to_string()
}

fn fri06_c08r_fixture_input_explicit_boundary_xml(name: &str, expectations: &str) -> String {
    format!(
        r#"<test name="{name}" use-rounding="false">
  <viewport width="100px" height="max-content" />
  <input>
    <div layout-ready-inline-root="true" display="block" width="100px">
      <inline-boundary kind="start" />
      <text layout-input="inline-text">
        <segment id="7" inline-extent="10" inline-baseline="8" inline-line-height="10" bidi-level="0" whitespace-edge="preserve" following-break="prohibited" />
      </text>
      <inline-boundary kind="end" />
      <inline-boundary kind="start" inline-baseline="8" inline-line-height="10" />
      <div display="inline-block" width="10px" height="10px" />
      <atomic-placeholder child-index="4" bidi-level="0" following-break="prohibited" />
    </div>
  </input>
  <expectations>{expectations}</expectations>
</test>"#
    )
}

#[test]
fn fri06_c12_t07_typed_inline_boundaries_are_expectation_transparent() {
    let xml = fri06_c08r_fixture_input_explicit_boundary_xml(
        "fri06_c12_t07_transparent_boundaries",
        r#"<node x="0" y="0" width="100" height="12"><node /><node /></node>"#,
    );
    let golden = support::Golden::parse(&xml).expect("typed boundary fixture must parse");
    support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
        panic!("typed input boundaries inflated expectations: {error}\n{xml}")
    });
}

fn fri06_c12_t07_wrapper_range_xml(name: &str) -> String {
    format!(
        r#"<test name="{name}" use-rounding="false">
  <viewport width="100px" height="max-content"/>
  <input>
    <div layout-ready-inline-root="true" display="block" width="100px">
      <div display="block">
        <text layout-input="inline-text">
          <segment id="0" inline-extent="10" inline-baseline="8" inline-line-height="10" bidi-level="0" whitespace-edge="preserve" following-break="prohibited"/>
        </text>
      </div>
      <div display="block">
        <text layout-input="inline-text">
          <segment id="0" inline-extent="10" inline-baseline="8" inline-line-height="10" bidi-level="0" whitespace-edge="preserve" following-break="prohibited"/>
        </text>
      </div>
    </div>
  </input>
  <expectations>
    <node x="0" y="0" width="100" height="20">
      <node x="0" y="0" width="100" height="10">
        <node>
          <range-inks>
            <range-ink source_segment_id="0" line_index="0" physical_start_edge="left" start="0" advance="10"/>
          </range-inks>
        </node>
      </node>
      <node x="0" y="10" width="100" height="10">
        <node>
          <range-inks>
            <range-ink source_segment_id="0" line_index="1" physical_start_edge="left" start="0" advance="10"/>
          </range-inks>
        </node>
      </node>
    </node>
  </expectations>
</test>"#
    )
}

#[test]
fn fri06_c12_t07_local_wrapper_range_lines_use_explicit_root_physical_identity() {
    let xml = fri06_c12_t07_wrapper_range_xml("fri06_c12_t07_root_range_lines");
    let golden = support::Golden::parse(&xml).expect("root-local Range fixture must parse");
    support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
        panic!("local wrapper Range identity was not root-normalized: {error}\n{xml}")
    });
}

#[test]
fn fri06_c12_t07_rename_and_expectation_mutation_preserve_normalized_input() {
    let original = fri06_c08r_fixture_input_explicit_boundary_xml(
        "fri06_bidi_mixed_inline__border_box_ltr",
        r#"<node width="100" height="12"><node /><node /></node>"#,
    );
    let mutated = fri06_c08r_fixture_input_explicit_boundary_xml(
        "renamed_without_fixture_dispatch",
        r#"<node width="999" height="777"><node x="40" /><node y="80"><node /></node></node>"#,
    );
    let original = support::Golden::parse(&original).expect("original fixture must parse");
    let mutated = support::Golden::parse(&mutated).expect("mutated fixture must parse");
    assert_eq!(original.root, mutated.root);
}

#[test]
fn fri06_c12_t07_exact_28_explicit_adapter_comparator_rows_pass() {
    const VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];
    const GRID_SOURCES: [&str; 4] = [
        "subgrid_baseline_auto_columns_first_item",
        "subgrid_baseline_auto_columns_second_item",
        "subgrid_baseline_standalone_axis_first_item",
        "subgrid_baseline_standalone_axis_second_item",
    ];

    let mut rows = Vec::new();
    for source in GRID_SOURCES {
        for variant in VARIANTS {
            rows.push(fri06_c12_t07_wrapper_range_xml(&format!(
                "{source}__{variant}"
            )));
        }
    }
    for variant in VARIANTS {
        for source in [
            "fri06_bidi_mixed_inline",
            "fri06_inline_mixed_text_atomic_wrap",
            "fri06_float_line_exclusion",
        ] {
            rows.push(fri06_c08r_fixture_input_explicit_boundary_xml(
                &format!("{source}__{variant}"),
                r#"<node x="0" y="0" width="100" height="12"><node /><node /></node>"#,
            ));
        }
    }
    assert_eq!(rows.len(), 28);

    let failures = rows
        .iter()
        .filter_map(|xml| {
            let golden = support::Golden::parse(xml)
                .unwrap_or_else(|error| panic!("explicit adapter row must parse: {error}\n{xml}"));
            support::assert_surgeist_matches(&golden)
                .err()
                .map(|error| format!("{}: {error}", golden.name))
        })
        .collect::<Vec<_>>();
    assert!(
        failures.is_empty(),
        "explicit adapter/comparator rows failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn fri06_c08r_fixture_input_closed_boundary_forms_parse_without_name_dispatch() {
    let xml = fri06_c08r_fixture_input_explicit_boundary_xml(
        "renamed_fixture_without_c08_identity",
        r#"<node x="0" y="0" width="100" height="10"><node /><node /></node>"#,
    );
    let golden = support::Golden::parse(&xml).expect("closed explicit boundary forms must parse");
    assert_eq!(golden.root.children.len(), 5);
}

#[test]
fn fri06_c08r_fixture_input_rename_and_expectation_only_mutation_preserve_input() {
    let first = fri06_c08r_fixture_input_explicit_boundary_xml(
        "fri06_bidi_mixed_inline__border_box_ltr",
        r#"<node x="0" y="0" width="100" height="10"><node /><node /></node>"#,
    );
    let second = fri06_c08r_fixture_input_explicit_boundary_xml(
        "arbitrary_renamed_fixture",
        r#"<node x="0" y="0" width="999" height="777"><node x="40" /><node y="80"><node /></node></node>"#,
    );
    let first = support::Golden::parse(&first).expect("first explicit fixture must parse");
    let second = support::Golden::parse(&second).expect("mutated explicit fixture must parse");
    assert_eq!(first.root, second.root);
}

#[test]
fn fri06_c08r_fixture_input_expectation_structure_cannot_control_input_lowering() {
    let ordinary = fri06_c08r_fixture_input_explicit_boundary_xml(
        "fri06_bidi_mixed_inline__border_box_ltr",
        "<node><node /><node /></node>",
    );
    let mutated = fri06_c08r_fixture_input_explicit_boundary_xml(
        "fri06_bidi_mixed_inline__border_box_ltr",
        "<node width=\"999\"><node><node /></node><node x=\"40\" /></node>",
    );
    let ordinary = support::Golden::parse(&ordinary).expect("ordinary expectations must parse");
    let mutated = support::Golden::parse(&mutated)
        .expect("valid expectation-only structure must not block input lowering");
    assert_eq!(ordinary.root, mutated.root);
}

#[test]
fn fri06_c08r_fixture_input_fixture_name_cannot_select_input_lowering() {
    let named = fri06_c08r_fixture_input_explicit_boundary_xml(
        "fri06_bidi_mixed_inline__border_box_ltr",
        "<node><node /><node /></node>",
    );
    let renamed = fri06_c08r_fixture_input_explicit_boundary_xml(
        "arbitrary_renamed_fixture",
        "<node><node /><node /></node>",
    );
    let named = support::Golden::parse(&named).expect("named fixture must parse");
    let renamed = support::Golden::parse(&renamed).expect("renamed fixture must parse");
    assert_eq!(named.root, renamed.root);
}

fn fri06_c08r_fixture_input_anonymous_wrapper_xml(
    marker: &str,
    display: &str,
    payload: &str,
) -> String {
    format!(
        r#"<test name="arbitrary_anonymous_wrapper" use-rounding="false">
  <viewport width="100px" height="max-content" />
  <input>
    <div display="grid">
      <div display="{display}" {marker}>{payload}</div>
    </div>
  </input>
  <expectations><node><node><node /></node></node></expectations>
</test>"#
    )
}

#[test]
fn fri06_c08r_fixture_input_parser_rejects_unknown_partial_malformed_and_payload_forms() {
    let valid = fri06_c08r_fixture_input_explicit_boundary_xml(
        "arbitrary_explicit_fixture",
        "<node><node /><node /></node>",
    );
    for (label, malformed, expected) in [
        (
            "unknown attribute",
            valid.replacen(
                "<inline-boundary kind=\"start\" />",
                "<inline-boundary kind=\"start\" extra=\"1\" />",
                1,
            ),
            "unsupported inline boundary attribute",
        ),
        (
            "invalid kind",
            valid.replacen("kind=\"start\"", "kind=\"middle\"", 1),
            "invalid inline boundary kind",
        ),
        (
            "partial metrics",
            valid.replacen(
                "<inline-boundary kind=\"start\" inline-baseline=\"8\" inline-line-height=\"10\" />",
                "<inline-boundary kind=\"start\" inline-baseline=\"8\" />",
                1,
            ),
            "metrics require both",
        ),
        (
            "end metrics",
            valid.replacen(
                "<inline-boundary kind=\"end\" />",
                "<inline-boundary kind=\"end\" inline-baseline=\"8\" inline-line-height=\"10\" />",
                1,
            ),
            "only a start inline boundary may carry metrics",
        ),
        (
            "invalid metrics",
            valid.replacen(
                "<inline-boundary kind=\"start\" inline-baseline=\"8\" inline-line-height=\"10\" />",
                "<inline-boundary kind=\"start\" inline-baseline=\"11\" inline-line-height=\"10\" />",
                1,
            ),
            "0 <= baseline",
        ),
        (
            "text payload",
            valid.replacen(
                "<inline-boundary kind=\"end\" />",
                "<inline-boundary kind=\"end\">payload</inline-boundary>",
                1,
            ),
            "unsupported non-whitespace text",
        ),
        (
            "element payload",
            valid.replacen(
                "<inline-boundary kind=\"end\" />",
                "<inline-boundary kind=\"end\"><div /></inline-boundary>",
                1,
            ),
            "unsupported `<inline-boundary>` child",
        ),
        (
            "metric boundary before text",
            valid.replacen(
                "<inline-boundary kind=\"start\" />",
                "<inline-boundary kind=\"start\" inline-baseline=\"8\" inline-line-height=\"10\" />",
                1,
            ),
            "must immediately precede one typed atomic child",
        ),
        (
            "misplaced end",
            valid.replacen(
                "<inline-boundary kind=\"start\" />",
                "<inline-boundary kind=\"end\" />",
                1,
            ),
            "misplaced end inline boundary",
        ),
        (
            "unknown layout-ready attribute",
            valid.replacen(
                "layout-ready-inline-root=\"true\"",
                "layout-ready-inline-root=\"true\" layout-ready-unknown=\"true\"",
                1,
            ),
            "unsupported layout-ready input attribute",
        ),
    ] {
        let error = match support::Golden::parse(&malformed) {
            Ok(_) => panic!("parser accepted {label}"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains(expected),
            "{label}: expected {expected:?}, got {error}"
        );
    }
}

#[test]
fn fri06_c08r_fixture_input_anonymous_wrapper_parser_is_closed_and_topology_checked() {
    let text = r#"<text layout-input="inline-text"><segment id="0" inline-extent="10" inline-baseline="8" inline-line-height="10" bidi-level="0" whitespace-edge="preserve" following-break="prohibited" /></text>"#;
    let valid = fri06_c08r_fixture_input_anonymous_wrapper_xml(
        "layout-ready-anonymous-grid-text-wrapper=\"true\"",
        "grid",
        text,
    );
    support::Golden::parse(&valid).expect("closed anonymous wrapper form must parse");

    for (label, xml, expected) in [
        (
            "invalid marker value",
            valid.replacen(
                "layout-ready-anonymous-grid-text-wrapper=\"true\"",
                "layout-ready-anonymous-grid-text-wrapper=\"false\"",
                1,
            ),
            "must be exactly `true`",
        ),
        (
            "invalid role",
            valid.replacen("display=\"grid\"", "display=\"block\"", 2),
            "requires only direct shaped-text children in a grid role",
        ),
        (
            "mixed raw fallback",
            valid.replacen("</div>", "raw fallback</div>", 1),
            "rejects raw text fallback",
        ),
        (
            "box child",
            fri06_c08r_fixture_input_anonymous_wrapper_xml(
                "layout-ready-anonymous-grid-text-wrapper=\"true\"",
                "grid",
                "<div display=\"block\" />",
            ),
            "requires only direct shaped-text children in a grid role",
        ),
    ] {
        let error = match support::Golden::parse(&xml) {
            Ok(_) => panic!("parser accepted {label}"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains(expected),
            "{label}: expected {expected:?}, got {error}"
        );
    }
}

#[test]
fn fri06_c08_recovery_inputs_shape_break_places_34_38_then_42_46_on_two_lines() {
    let xml = fri06_c08_recovery_inputs_shape_xml();
    let golden = support::Golden::parse(&xml).expect("exact shape-break fixture should parse");
    assert_eq!(
        golden.root.style.get("layout-ready-inline-root"),
        Some("true")
    );
    support::assert_surgeist_matches(&golden)
        .unwrap_or_else(|error| panic!("reviewed shape break placement changed: {error}\n{xml}"));

    let wrong_target = xml
        .replacen(
            r#"child-index="3" bidi-level="0" following-break="allowed""#,
            r#"child-index="3" bidi-level="0" following-break="prohibited""#,
            1,
        )
        .replacen(
            r#"child-index="4" bidi-level="0" following-break="prohibited""#,
            r#"child-index="4" bidi-level="0" following-break="allowed""#,
            1,
        );
    let golden = support::Golden::parse(&wrong_target).expect("wrong target remains schema-valid");
    support::assert_surgeist_matches(&golden)
        .expect_err("moving the allowed break from 38px to 42px must reject browser placement");

    for (label, malformed) in [
        (
            "duplicate",
            xml.replacen(
                r#"<atomic-placeholder child-index="3" bidi-level="0" following-break="allowed"/>"#,
                r#"<atomic-placeholder child-index="3" bidi-level="0" following-break="allowed"/>
      <atomic-placeholder child-index="3" bidi-level="0" following-break="allowed"/>"#,
                1,
            ),
        ),
        (
            "out of range",
            xml.replacen(r#"child-index="3""#, r#"child-index="99""#, 1),
        ),
    ] {
        assert!(
            support::Golden::parse(&malformed).is_err(),
            "shape atomic break parser accepted {label} target"
        );
    }
}

fn fri06_c08_recovery_adapter_variant(variant: &str) -> (&'static str, &'static str, u8) {
    match variant {
        "border_box_ltr" => ("border-box", "ltr", 0),
        "border_box_rtl" => ("border-box", "rtl", 1),
        "content_box_ltr" => ("content-box", "ltr", 0),
        "content_box_rtl" => ("content-box", "rtl", 1),
        other => panic!("unexpected C08 recovery-adapter variant {other}"),
    }
}

fn fri06_c08_recovery_adapter_grid_range_xml(source: &str, variant: &str) -> String {
    let (box_sizing, direction, bidi_level) = fri06_c08_recovery_adapter_variant(variant);
    let (root_tracks, subgrid_tracks, item_tracks, starts) = if source.contains("standalone_axis") {
        (
            r#" grid-template-rows="30px" grid-template-columns="60px""#,
            r#" grid-template-rows="subgrid" grid-template-columns="repeat(2, 30px)""#,
            r#" grid-template-rows="auto" grid-template-columns="subgrid""#,
            if direction == "ltr" {
                [0.0, 30.0]
            } else {
                [60.0, 30.0]
            },
        )
    } else {
        (
            r#" grid-template-columns="repeat(2, auto)""#,
            r#" grid-column-start="1" grid-column-end="-1" grid-template-columns="subgrid""#,
            "",
            if direction == "ltr" {
                [0.0, 15.0]
            } else {
                [45.0, 30.0]
            },
        )
    };
    let edge = if direction == "ltr" { "left" } else { "right" };
    format!(
        r#"<test name="{source}__{variant}" use-rounding="true">
  <viewport width="max-content" height="max-content"/>
  <input>
    <div layout-ready-inline-root="true" display="grid" box-sizing="{box_sizing}" direction="{direction}" align-items="baseline"{root_tracks} font-family="ahem" font-size="15px" line-height="15px">
      <div display="grid" direction="{direction}" align-items="baseline"{subgrid_tracks}>
        <div layout-ready-anonymous-grid-text-wrapper="true" display="grid" direction="{direction}" align-items="baseline"{item_tracks}>
          <text layout-input="inline-text">
            <segment id="0" inline-extent="15" inline-baseline="12" inline-line-height="15" bidi-level="{bidi_level}" whitespace-edge="preserve" following-break="prohibited"/>
          </text>
        </div>
        <div layout-ready-anonymous-grid-text-wrapper="true" display="grid" direction="{direction}" align-items="baseline"{item_tracks} font-size="30px" line-height="30px">
          <text layout-input="inline-text">
            <segment id="0" inline-extent="30" inline-baseline="24" inline-line-height="30" bidi-level="{bidi_level}" whitespace-edge="preserve" following-break="prohibited"/>
          </text>
        </div>
      </div>
    </div>
  </input>
  <expectations>
    <node>
      <node>
        <node>
          <node>
            <range-inks>
              <range-ink source_segment_id="0" line_index="0" physical_start_edge="{edge}" start="{}" advance="15"/>
            </range-inks>
          </node>
        </node>
        <node>
          <node>
            <range-inks>
              <range-ink source_segment_id="0" line_index="1" physical_start_edge="{edge}" start="{}" advance="30"/>
            </range-inks>
          </node>
        </node>
      </node>
    </node>
  </expectations>
</test>"#,
        starts[0], starts[1]
    )
}

#[test]
fn fri06_c08_recovery_adapter_all_16_grid_range_starts_are_explicit_root_relative() {
    const SOURCES: [&str; 4] = [
        "subgrid_baseline_auto_columns_first_item",
        "subgrid_baseline_auto_columns_second_item",
        "subgrid_baseline_standalone_axis_first_item",
        "subgrid_baseline_standalone_axis_second_item",
    ];
    const VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    let mut compared = 0;
    let mut failures = Vec::new();
    for source in SOURCES {
        for variant in VARIANTS {
            let xml = fri06_c08_recovery_adapter_grid_range_xml(source, variant);
            let golden = support::Golden::parse(&xml)
                .unwrap_or_else(|error| panic!("{source}__{variant} must parse: {error}\n{xml}"));
            compared += 1;
            if let Err(error) = support::assert_surgeist_matches(&golden) {
                failures.push(format!("{}: {error}", golden.name));
            }
        }
    }
    assert_eq!(compared, 16);
    assert!(
        failures.is_empty(),
        "all 16 generated grid shapes must compare:\n{}",
        failures.join("\n")
    );

    let applied_twice = fri06_c08_recovery_adapter_grid_range_xml(
        "subgrid_baseline_auto_columns_first_item",
        "border_box_ltr",
    )
    .replacen(
        r#"physical_start_edge="left" start="15" advance="30""#,
        r#"physical_start_edge="left" start="30" advance="30""#,
        1,
    );
    let applied_twice = support::Golden::parse(&applied_twice)
        .expect("double-translation negative control should parse");
    support::assert_surgeist_matches(&applied_twice)
        .expect_err("ordinary plus synthetic ancestry must not be added twice");
}

fn fri06_c08_recovery_adapter_nested_rtl_range_xml(variant: &str) -> String {
    let (box_sizing, direction, bidi_level) = fri06_c08_recovery_adapter_variant(variant);
    assert_eq!(direction, "rtl");
    format!(
        r#"<test name="fri06_atomic_inline_baseline__{variant}" use-rounding="true">
  <viewport width="max-content" height="max-content"/>
  <input>
    <div layout-ready-inline-root="true" display="block" box-sizing="{box_sizing}" direction="rtl" width="220px" font-family="monospace" font-size="16px" line-height="24px">
      <div display="inline-block" box-sizing="{box_sizing}" direction="rtl" width="28px" margin-bottom="6px">
        <div display="block" box-sizing="{box_sizing}" direction="rtl" height="12px">
          <text layout-input="inline-text">
            <segment id="0" inline-extent="9.640625" inline-baseline="14.8" inline-line-height="24" bidi-level="{bidi_level}" whitespace-edge="preserve" following-break="prohibited"/>
          </text>
        </div>
      </div>
      <atomic-placeholder child-index="0" bidi-level="{bidi_level}" following-break="prohibited"/>
    </div>
  </input>
  <expectations>
    <node>
      <node>
        <node>
          <node>
            <range-inks>
              <range-ink source_segment_id="0" line_index="0" physical_start_edge="right" start="220" advance="9.640625"/>
            </range-inks>
          </node>
        </node>
      </node>
    </node>
  </expectations>
</test>"#
    )
}

#[test]
fn fri06_c08_recovery_adapter_both_nested_rtl_range_starts_are_explicit_root_relative() {
    let mut failures = Vec::new();
    for variant in ["border_box_rtl", "content_box_rtl"] {
        let xml = fri06_c08_recovery_adapter_nested_rtl_range_xml(variant);
        let golden = support::Golden::parse(&xml).unwrap_or_else(|error| {
            panic!("{variant} nested RTL shape must parse: {error}\n{xml}")
        });
        if let Err(error) = support::assert_surgeist_matches(&golden) {
            failures.push(format!("{}: {error}", golden.name));
        }
    }
    assert!(
        failures.is_empty(),
        "both generated nested RTL shapes must compare:\n{}",
        failures.join("\n")
    );

    let missing_marker = fri06_c08_recovery_adapter_nested_rtl_range_xml("border_box_rtl")
        .replacen(r#" layout-ready-inline-root="true""#, "", 1);
    let missing_marker = support::Golden::parse(&missing_marker)
        .expect("a missing nested marker remains schema-valid input");
    assert!(
        support::assert_surgeist_matches(&missing_marker)
            .expect_err("nested Range ancestry without its explicit root must fail")
            .to_string()
            .contains("requires an explicit inline root marker")
    );
}

fn fri06_c08_recovery_adapter_mixed_wrap_xml(variant: &str) -> String {
    let (box_sizing, direction, bidi_level) = fri06_c08_recovery_adapter_variant(variant);
    format!(
        r#"<test name="fri06_inline_mixed_text_atomic_wrap__{variant}" use-rounding="true">
  <viewport width="max-content" height="max-content"/>
  <input>
    <div layout-ready-inline-root="true" display="block" box-sizing="{box_sizing}" direction="{direction}" width="72px" font-family="monospace" font-size="16px" line-height="20px">
      <text layout-input="inline-text">
        <segment id="0" inline-extent="38.53125" inline-baseline="14.8" inline-line-height="20" bidi-level="{bidi_level}" whitespace-edge="preserve" following-break="allowed"/>
      </text>
      <div display="inline-block" box-sizing="{box_sizing}" direction="{direction}" width="18px" height="18px"/>
      <inline-boundary kind="start" inline-baseline="14.8" inline-line-height="20"/>
      <div display="inline-block" box-sizing="{box_sizing}" direction="{direction}" width="24px" height="18px"/>
      <div display="inline-block" box-sizing="{box_sizing}" direction="{direction}" width="30px" height="18px"/>
      <atomic-placeholder child-index="1" bidi-level="{bidi_level}" following-break="allowed"/>
      <atomic-placeholder child-index="3" bidi-level="{bidi_level}" following-break="prohibited"/>
      <atomic-placeholder child-index="4" bidi-level="{bidi_level}" following-break="prohibited"/>
    </div>
  </input>
  <expectations>
    <node height="46">
      <node/>
      <node/>
      <node y="23"/>
      <node y="23"/>
    </node>
  </expectations>
</test>"#
    )
}

#[test]
fn fri06_c08_recovery_adapter_all_four_mixed_wrap_rows_use_root_metric_continuation_strut() {
    let mut failures = Vec::new();
    for variant in [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ] {
        let xml = fri06_c08_recovery_adapter_mixed_wrap_xml(variant);
        let golden = support::Golden::parse(&xml).unwrap_or_else(|error| {
            panic!("{variant} mixed-wrap shape must parse: {error}\n{xml}")
        });
        if let Err(error) = support::assert_surgeist_matches(&golden) {
            failures.push(format!("{}: {error}", golden.name));
        }
    }
    assert!(
        failures.is_empty(),
        "all four mixed-wrap rows must be 46px with y=23 continuation atomics:\n{}",
        failures.join("\n")
    );

    let exact = fri06_c08_recovery_adapter_mixed_wrap_xml("border_box_ltr");
    let altered_name = exact.replacen(
        "fri06_inline_mixed_text_atomic_wrap__border_box_ltr",
        "fri06_inline_mixed_text_atomic_wrap_control__border_box_ltr",
        1,
    );
    let altered_name = support::Golden::parse(&altered_name)
        .expect("an altered fixture name remains schema-valid");
    support::assert_surgeist_matches(&altered_name)
        .expect("the explicit continuation strut must not depend on fixture identity");

    let altered_topology = exact.replacen(
        r#"child-index="1" bidi-level="0" following-break="allowed""#,
        r#"child-index="1" bidi-level="0" following-break="prohibited""#,
        1,
    );
    let altered_topology = support::Golden::parse(&altered_topology)
        .expect("the alternate valid break topology should parse");
    support::assert_surgeist_matches(&altered_topology)
        .expect_err("the altered break topology must not match the final browser geometry");
}

fn fri06_c08_recovery_adapter_direct_ltr_range_xml() -> String {
    r#"<test name="fri06_c08_recovery_adapter_direct_ltr" use-rounding="false">
  <viewport width="100px" height="max-content"/>
  <input>
    <div layout-ready-inline-root="true" display="block" width="100px" font-size="10px" line-height="20px">
      <text layout-input="inline-text">
        <segment id="7" inline-extent="10" inline-baseline="8" inline-line-height="20" bidi-level="0" whitespace-edge="preserve" following-break="prohibited"/>
      </text>
    </div>
  </input>
  <expectations>
    <node>
      <node>
        <range-inks>
          <range-ink source_segment_id="7" line_index="0" physical_start_edge="left" start="0" advance="10"/>
        </range-inks>
      </node>
    </node>
  </expectations>
</test>"#
        .to_string()
}

#[test]
fn fri06_c08_recovery_adapter_direct_ltr_is_zero_and_identity_remains_strict() {
    let xml = fri06_c08_recovery_adapter_direct_ltr_range_xml();
    let golden = support::Golden::parse(&xml).expect("direct LTR shape should parse");
    support::assert_surgeist_matches(&golden)
        .expect("a direct LTR owner must receive zero ancestor translation");

    let missing_marker = xml.replacen(r#" layout-ready-inline-root="true""#, "", 1);
    let missing_marker = support::Golden::parse(&missing_marker)
        .expect("a missing direct marker remains schema-valid input");
    assert!(
        support::assert_surgeist_matches(&missing_marker)
            .expect_err("a direct Range owner without its explicit root must fail")
            .to_string()
            .contains("requires an explicit inline root marker")
    );

    for (label, changed) in [
        (
            "source",
            xml.replacen(r#"source_segment_id="7""#, r#"source_segment_id="8""#, 1),
        ),
        (
            "line",
            xml.replacen(r#"line_index="0""#, r#"line_index="1""#, 1),
        ),
        (
            "edge",
            xml.replacen(
                r#"physical_start_edge="left""#,
                r#"physical_start_edge="right""#,
                1,
            ),
        ),
        (
            "advance",
            xml.replacen(r#"advance="10""#, r#"advance="11""#, 1),
        ),
    ] {
        let changed = support::Golden::parse(&changed)
            .unwrap_or_else(|error| panic!("mutated {label} identity should parse: {error}"));
        assert!(
            support::assert_surgeist_matches(&changed).is_err(),
            "mutated Range {label} identity must fail"
        );
    }
}
