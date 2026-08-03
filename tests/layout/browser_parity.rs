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

#[test]
fn runs_browser_parity_smoke_fixture_against_surgeist_layout() {
    let golden = support::Golden::parse(include_str!(
        "browser_parity/xml/block/block_basic__border_box_ltr.xml"
    ))
    .expect("browser parity fixture should parse");

    support::assert_surgeist_matches(&golden).expect("surgeist layout should match fixture");
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
fn fri_03_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_outputs() {
    let expected = fri_03_expected_paths();

    assert!(fri_03_fixture_paths(expected.iter().skip(1).cloned()).is_err());

    let mut duplicate = expected.clone();
    duplicate.push(expected[0].clone());
    assert!(fri_03_fixture_paths(duplicate).is_err());

    let mut misplaced = expected.clone();
    let file = misplaced[0]
        .file_name()
        .expect("FRI-03 path should have a filename")
        .to_owned();
    misplaced[0] = PathBuf::from("xml/other").join(file);
    assert!(fri_03_fixture_paths(misplaced).is_err());

    let mut extra = expected;
    extra.push(PathBuf::from(
        "xml/block/block_align_baseline_child_margin_percent__extra.xml",
    ));
    assert!(fri_03_fixture_paths(extra).is_err());

    let paths = fri_03_fixture_paths(browser_parity_fixture_paths())
        .unwrap_or_else(|error| panic!("FRI-03 fixture matrix is incomplete: {error}"));
    assert_eq!(paths.len(), 32);
    assert_eq!(
        paths
            .iter()
            .filter(|path| fri_03_family(path) == Some(Fri03Family::BlockMargin))
            .count(),
        4
    );
    assert_eq!(
        paths
            .iter()
            .filter(|path| fri_03_family(path) == Some(Fri03Family::Order))
            .count(),
        12
    );
    assert_eq!(
        paths
            .iter()
            .filter(|path| fri_03_family(path) == Some(Fri03Family::FlexItemRoot))
            .count(),
        16
    );

    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    for relative in paths {
        let fixture = corpus_root.join(&relative);
        let golden = support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
        assert_fri_03_fixture_topology(&golden, &relative)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.display()));
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
fn fri04_c05_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_paths() {
    let expected = fri04_c05_expected_paths();

    assert!(fri04_c05_fixture_paths(expected.iter().skip(1).cloned()).is_err());

    let mut duplicate = expected.clone();
    duplicate.push(expected[0].clone());
    assert!(fri04_c05_fixture_paths(duplicate).is_err());

    let mut misplaced = expected.clone();
    let file = misplaced[0]
        .file_name()
        .expect("FRI-04 path should have a filename")
        .to_owned();
    misplaced[0] = PathBuf::from("xml/other").join(file);
    assert!(fri04_c05_fixture_paths(misplaced).is_err());

    let mut extra = expected;
    extra.push(PathBuf::from(
        "xml/block/fri04_sizing_math_functions__extra.xml",
    ));
    assert!(fri04_c05_fixture_paths(extra).is_err());

    let paths = fri04_c05_fixture_paths(browser_parity_fixture_paths())
        .unwrap_or_else(|error| panic!("FRI-04 C05 fixture matrix is incomplete: {error}"));
    assert_eq!(paths.len(), 12);
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

#[test]
fn fri04_c05_fixture_inventory_manifest_and_report_are_final() {
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let html_root = corpus_root.join("html");
    let html = support::fixture_files_in(&html_root, "html")
        .expect("HTML parity fixtures should be readable");
    let sources = [
        "block/fri04_sizing_math_functions.html",
        "flex/fri04_flex_basis_content.html",
        "grid/fri04_track_math_functions.html",
    ];

    assert_eq!(html.len(), 1432);
    for source in sources {
        assert!(
            html.contains(&html_root.join(source)),
            "missing FRI-04 source {source}"
        );
    }

    let xml = support::fixture_files_in(&corpus_root.join("xml"), "xml")
        .expect("XML parity fixtures should be readable");
    assert_eq!(xml.len(), 5712);

    let manifest_path = corpus_root.join("corpus.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("{} should read: {error}", manifest_path.display()));
    assert!(manifest.contains("generated = 5712"));
    for source in sources {
        let case = format!(
            "id = \"{}\"\nsource_root = \"surgeist\"\nsource = \"{source}\"\ngenerator = \"constrained-html\"\nstatus = \"active\"",
            source.trim_end_matches(".html")
        );
        assert_eq!(
            manifest.matches(&case).count(),
            1,
            "manifest should contain exactly one active case for {source}"
        );
    }

    let report_root = corpus_root.join("xml/generation-reports");
    let reports = support::fixture_files_in(&report_root, "json")
        .expect("generation reports should be readable");
    assert_eq!(
        reports
            .iter()
            .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["all.json"])
    );
    let report_path = report_root.join("all.json");
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&report_path)
            .unwrap_or_else(|error| panic!("{} should read: {error}", report_path.display())),
    )
    .unwrap_or_else(|error| panic!("{} should parse: {error}", report_path.display()));
    assert_eq!(report["summary"]["generated"], 5712);
    assert_eq!(report["summary"]["unsupported"], 16);
    for bucket in ["expected_fail", "quarantined", "failed_to_generate"] {
        assert_eq!(report["summary"][bucket], 0, "nonzero {bucket} summary");
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
fn fri05_c06_computed_overflow_corpus_has_exact_44_output_inventory() {
    let expected = fri05_c06_computed_overflow_paths();
    assert_eq!(expected.len(), 44);
    assert!(expected.iter().all(|path| path.is_file()));

    let actual = support::fixture_files("xml")
        .expect("XML corpus should be readable")
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("fri05_"))
        })
        .map(|path| {
            path.canonicalize()
                .unwrap_or_else(|error| panic!("{} should canonicalize: {error}", path.display()))
        })
        .collect::<BTreeSet<_>>();
    let expected = expected
        .into_iter()
        .map(|path| {
            path.canonicalize()
                .unwrap_or_else(|error| panic!("{} should canonicalize: {error}", path.display()))
        })
        .collect();
    assert_eq!(actual, expected);
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
fn fri05_c06_computed_overflow_corpus_outputs_have_current_provenance() {
    let report_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity/xml/generation-reports/all.json");
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&report_path)
            .unwrap_or_else(|error| panic!("{} should read: {error}", report_path.display())),
    )
    .unwrap_or_else(|error| panic!("{} should parse: {error}", report_path.display()));
    let helper_sha = report["metadata"]["helper_sha256"]
        .as_str()
        .expect("report should name helper provenance");
    for path in fri05_c06_computed_overflow_paths() {
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should read: {error}", path.display()));
        assert!(
            raw.trim_start()
                .starts_with("<!-- generated-by: surgeist-layout-generate "),
            "{} lacks generator provenance",
            path.display()
        );
        assert!(
            raw.contains(&format!("helper-sha256=\"{helper_sha}\"")),
            "{} has stale helper provenance",
            path.display()
        );
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
    let paths = block_axis_fixture_paths(fixtures.iter().map(|(relative, _)| relative.clone()))
        .unwrap_or_else(|error| panic!("{error}"));

    for (relative, fixture) in fixtures {
        if !paths.contains(&relative) {
            continue;
        }
        let golden = support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
        assert_block_axis_fixture_topology(&golden, &relative)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.display()));
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} failed layout comparison: {error}", fixture.display())
        });
    }
}

#[test]
fn runs_fri_02_flex_axis_families_against_surgeist_layout() {
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
    let paths = flex_axis_fixture_paths(fixtures.iter().map(|(relative, _)| relative.clone()))
        .unwrap_or_else(|error| panic!("{error}"));

    for (relative, fixture) in fixtures {
        if !paths.contains(&relative) {
            continue;
        }
        let golden = support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
        assert_flex_axis_fixture_topology(&golden, &relative)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.display()));
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} failed layout comparison: {error}", fixture.display())
        });
    }
}

#[test]
fn runs_fri_02_grid_axis_families_against_surgeist_layout() {
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
    let paths = grid_axis_fixture_paths(fixtures.iter().map(|(relative, _)| relative.clone()))
        .unwrap_or_else(|error| panic!("{error}"));

    for (relative, fixture) in fixtures {
        if !paths.contains(&relative) {
            continue;
        }
        let golden = support::Golden::parse_file(&fixture)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", fixture.display()));
        assert_grid_axis_fixture_topology(&golden, &relative)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.display()));
        support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
            panic!("{} failed layout comparison: {error}", fixture.display())
        });
    }
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

#[test]
fn grid_axes_fixture_matrix_is_generated() {
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
        });

    grid_axis_fixture_paths(fixtures)
        .unwrap_or_else(|error| panic!("grid_axes fixture matrix is incomplete: {error}"));
}

#[test]
fn grid_lanes_axis_fixture_matrix() {
    let fixtures = browser_parity_fixture_paths();
    grid_lanes_axis_fixture_paths(fixtures)
        .unwrap_or_else(|error| panic!("grid-lanes axis fixture matrix is incomplete: {error}"));
}

#[test]
fn subgrid_axis_fixture_matrix() {
    let fixtures = browser_parity_fixture_paths();
    subgrid_axis_fixture_paths(fixtures)
        .unwrap_or_else(|error| panic!("subgrid axis fixture matrix is incomplete: {error}"));
}

#[test]
fn flex_axis_fixture_matrix_rejects_missing_duplicate_misplaced_and_leaf_lowered_topology_paths() {
    let expected = flex_axis_expected_paths();

    assert!(flex_axis_fixture_paths(expected.iter().skip(1).cloned()).is_err());

    let mut duplicate = expected.clone();
    duplicate.push(expected[0].clone());
    assert!(flex_axis_fixture_paths(duplicate).is_err());

    let mut misplaced = expected.clone();
    misplaced[0] = PathBuf::from("xml/other/flex_axes_horizontal_tb_row__border_box_ltr.xml");
    assert!(flex_axis_fixture_paths(misplaced).is_err());

    let leaf_lowered = support::Golden::parse(
        r#"
        <test name="flex_axes_horizontal_tb_row__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input>
            <div display="flex" writing-mode="horizontal-tb" flex-direction="row">
              <text display="block" width="20px" height="10px">a</text>
              <div display="block" width="10px" height="20px" />
            </div>
          </input>
          <expectations>
            <node x="0" y="0" width="30" height="20">
              <node x="0" y="0" width="20" height="10" />
              <node x="20" y="0" width="10" height="20" />
            </node>
          </expectations>
        </test>
        "#,
    )
    .expect("leaf-lowered fixture should parse");
    assert!(
        assert_flex_axis_fixture_topology(
            &leaf_lowered,
            Path::new("xml/flex/flex_axes_horizontal_tb_row__border_box_ltr.xml"),
        )
        .is_err()
    );

    let bypassed = support::Golden::parse(
        r#"
        <test name="flex_axes_horizontal_tb_row__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input>
            <div display="block" writing-mode="horizontal-tb" flex-direction="row">
              <div display="block" width="20px" height="10px" />
              <div display="block" width="10px" height="20px" />
            </div>
          </input>
          <expectations>
            <node x="0" y="0" width="20" height="30">
              <node x="0" y="0" width="20" height="10" />
              <node x="0" y="10" width="10" height="20" />
            </node>
          </expectations>
        </test>
        "#,
    )
    .expect("bypassed fixture should parse");
    assert!(
        assert_flex_axis_fixture_topology(
            &bypassed,
            Path::new("xml/flex/flex_axes_horizontal_tb_row__border_box_ltr.xml"),
        )
        .is_err()
    );

    let one_child = support::Golden::parse(
        r#"
        <test name="flex_axes_horizontal_tb_row__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input>
            <div display="flex" writing-mode="horizontal-tb" flex-direction="row">
              <div display="block" width="20px" height="10px" />
            </div>
          </input>
          <expectations>
            <node x="0" y="0" width="20" height="10">
              <node x="0" y="0" width="20" height="10" />
            </node>
          </expectations>
        </test>
        "#,
    )
    .expect("one-child fixture should parse");
    assert!(
        assert_flex_axis_fixture_topology(
            &one_child,
            Path::new("xml/flex/flex_axes_horizontal_tb_row__border_box_ltr.xml"),
        )
        .is_err()
    );
}

#[test]
fn grid_axis_fixture_matrix_rejects_invalid_paths_and_topology() {
    let expected = grid_axis_expected_paths();

    assert!(grid_axis_fixture_paths(expected.iter().skip(1).cloned()).is_err());

    let mut duplicate = expected.clone();
    duplicate.push(expected[0].clone());
    assert!(grid_axis_fixture_paths(duplicate).is_err());

    let mut misplaced = expected.clone();
    misplaced[0] = PathBuf::from("xml/other/grid_axes_horizontal_tb_parallel__border_box_ltr.xml");
    assert!(grid_axis_fixture_paths(misplaced).is_err());

    let mut extra = expected.clone();
    extra.push(PathBuf::from(
        "xml/grid/grid_axes_extra__border_box_ltr.xml",
    ));
    assert!(grid_axis_fixture_paths(extra).is_err());

    for (description, root_style, first_child, second_child) in [
        (
            "non-grid root",
            "display=\"block\" writing-mode=\"horizontal-tb\" grid-template-columns=\"30px 40px\" grid-template-rows=\"50px 60px\"",
            "display=\"block\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
            "display=\"block\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        ),
        (
            "subgrid root",
            "display=\"grid\" writing-mode=\"horizontal-tb\" grid-template-columns=\"subgrid\" grid-template-rows=\"50px 60px\"",
            "display=\"block\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
            "display=\"block\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        ),
        (
            "grid-lanes root",
            "display=\"grid\" writing-mode=\"horizontal-tb\" grid-template-columns=\"lanes 30px 40px\" grid-template-rows=\"50px 60px\"",
            "display=\"block\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
            "display=\"block\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        ),
        (
            "absolute topology",
            "display=\"grid\" writing-mode=\"horizontal-tb\" grid-template-columns=\"30px 40px\" grid-template-rows=\"50px 60px\"",
            "display=\"block\" position=\"absolute\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
            "display=\"block\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        ),
        (
            "hidden-only topology",
            "display=\"grid\" writing-mode=\"horizontal-tb\" grid-template-columns=\"30px 40px\" grid-template-rows=\"50px 60px\"",
            "display=\"none\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
            "display=\"block\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        ),
        (
            "equal totals",
            "display=\"grid\" writing-mode=\"horizontal-tb\" grid-template-columns=\"30px 40px\" grid-template-rows=\"30px 40px\"",
            "display=\"block\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
            "display=\"block\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        ),
        (
            "indefinite placement",
            "display=\"grid\" writing-mode=\"horizontal-tb\" grid-template-columns=\"30px auto\" grid-template-rows=\"50px 60px\"",
            "display=\"block\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
            "display=\"block\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        ),
        (
            "overlapping placement",
            "display=\"grid\" writing-mode=\"horizontal-tb\" grid-template-columns=\"30px 40px\" grid-template-rows=\"50px 60px\"",
            "display=\"block\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
            "display=\"block\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
        ),
    ] {
        let golden = grid_axis_test_golden(root_style, first_child, second_child, false);
        assert!(
            assert_grid_axis_fixture_topology(
                &golden,
                Path::new("xml/grid/grid_axes_horizontal_tb_parallel__border_box_ltr.xml"),
            )
            .is_err(),
            "{description} must be rejected"
        );
    }

    let text_only = grid_axis_test_golden(
        "display=\"grid\" writing-mode=\"horizontal-tb\" grid-template-columns=\"30px 40px\" grid-template-rows=\"50px 60px\"",
        "display=\"block\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
        "display=\"block\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        true,
    );
    assert!(
        assert_grid_axis_fixture_topology(
            &text_only,
            Path::new("xml/grid/grid_axes_horizontal_tb_parallel__border_box_ltr.xml"),
        )
        .is_err(),
        "text-only topology must be rejected"
    );

    let wrong_flow = grid_axis_test_golden(
        "display=\"grid\" writing-mode=\"vertical-rl\" grid-template-columns=\"30px 40px\" grid-template-rows=\"50px 60px\"",
        "display=\"block\" writing-mode=\"vertical-rl\" grid-column-start=\"1\" grid-column-end=\"2\" grid-row-start=\"1\" grid-row-end=\"2\"",
        "display=\"block\" writing-mode=\"vertical-rl\" grid-column-start=\"2\" grid-column-end=\"3\" grid-row-start=\"2\" grid-row-end=\"3\"",
        false,
    );
    assert!(
        assert_grid_axis_fixture_topology(
            &wrong_flow,
            Path::new("xml/grid/grid_axes_vertical_opposing__border_box_ltr.xml"),
        )
        .is_err(),
        "wrong named parent/child flow relationship must be rejected"
    );
}

#[test]
fn grid_lanes_axis_fixture_matrix_rejects_invalid_paths_and_topology() {
    assert_axis_fixture_paths_reject_invalid_inventory(grid_lanes_axis_expected_paths());

    for (description, golden) in [
        (
            "non-grid lanes case",
            grid_lanes_axis_test_golden(
                "block",
                "block",
                "grid-lanes",
                "50px 60px",
                "horizontal-tb",
                "horizontal-tb",
            ),
        ),
        (
            "wrong grid root",
            grid_lanes_axis_test_golden(
                "grid",
                "grid-lanes",
                "grid-lanes",
                "50px 60px",
                "horizontal-tb",
                "horizontal-tb",
            ),
        ),
        (
            "wrong child flow",
            grid_lanes_axis_test_golden(
                "block",
                "grid-lanes",
                "grid-lanes",
                "50px 60px",
                "vertical-rl",
                "horizontal-tb",
            ),
        ),
        ("text child", grid_lanes_axis_text_child_golden()),
        (
            "absolute child",
            grid_lanes_axis_item_golden(
                "<div position=\"absolute\" grid-row-start=\"1\" grid-row-end=\"2\" />",
                "<div grid-row-start=\"2\" grid-row-end=\"3\" />",
            ),
        ),
        (
            "hidden-only children",
            grid_lanes_axis_item_golden(
                "<div display=\"none\" grid-row-start=\"1\" grid-row-end=\"2\" />",
                "<div display=\"none\" grid-row-start=\"2\" grid-row-end=\"3\" />",
            ),
        ),
        (
            "equal totals",
            grid_lanes_axis_test_golden(
                "block",
                "grid-lanes",
                "grid-lanes",
                "30px 40px",
                "horizontal-tb",
                "horizontal-tb",
            ),
        ),
        (
            "indefinite tracks",
            grid_lanes_axis_test_golden(
                "block",
                "grid-lanes",
                "grid-lanes",
                "auto 60px",
                "horizontal-tb",
                "horizontal-tb",
            ),
        ),
    ] {
        assert!(
            assert_grid_lanes_axis_fixture_topology(
                &golden,
                Path::new(
                    "xml/grid-lanes/grid_lanes_axes_horizontal_tb_parallel__border_box_ltr.xml"
                ),
            )
            .is_err(),
            "{description} must be rejected"
        );
    }

    let mut overlapping = support::Golden::parse(include_str!(
        "browser_parity/xml/grid-lanes/grid_lanes_axes_horizontal_tb_parallel__border_box_ltr.xml"
    ))
    .expect("grid-lanes overlap golden should parse");
    overlapping.expectations.children[1].y = Some(0.0);
    assert!(
        assert_grid_lanes_axis_fixture_topology(
            &overlapping,
            Path::new("xml/grid-lanes/grid_lanes_axes_horizontal_tb_parallel__border_box_ltr.xml"),
        )
        .is_err(),
        "overlapping top-level case expectations must be rejected"
    );
}

#[test]
fn subgrid_axis_fixture_matrix_rejects_invalid_paths_and_topology() {
    assert_axis_fixture_paths_reject_invalid_inventory(subgrid_axis_expected_paths());

    for (description, golden) in [
        (
            "non-grid parent",
            subgrid_axis_test_golden(
                "block",
                "block",
                "grid",
                "30px 40px",
                "50px 60px",
                "horizontal-tb",
                "horizontal-tb",
            ),
        ),
        (
            "wrong grid root",
            subgrid_axis_test_golden(
                "grid",
                "grid",
                "grid",
                "30px 40px",
                "50px 60px",
                "horizontal-tb",
                "horizontal-tb",
            ),
        ),
        (
            "wrong child flow",
            subgrid_axis_test_golden(
                "block",
                "grid",
                "grid",
                "30px 40px",
                "50px 60px",
                "vertical-rl",
                "horizontal-tb",
            ),
        ),
        ("text item", subgrid_axis_text_item_golden()),
        ("absolute item", subgrid_axis_absolute_item_golden()),
        (
            "hidden-only items",
            subgrid_axis_item_golden(
                "<div display=\"none\" grid-column-start=\"1\" grid-column-end=\"2\" />",
                "<div display=\"none\" grid-column-start=\"2\" grid-column-end=\"3\" />",
            ),
        ),
        (
            "equal totals",
            subgrid_axis_test_golden(
                "block",
                "grid",
                "grid",
                "30px 40px",
                "30px 40px",
                "horizontal-tb",
                "horizontal-tb",
            ),
        ),
        (
            "indefinite tracks",
            subgrid_axis_test_golden(
                "block",
                "grid",
                "grid",
                "auto 40px",
                "50px 60px",
                "horizontal-tb",
                "horizontal-tb",
            ),
        ),
    ] {
        assert!(
            assert_subgrid_axis_fixture_topology(
                &golden,
                Path::new("xml/subgrid/subgrid_axes_horizontal_tb_parallel__border_box_ltr.xml"),
            )
            .is_err(),
            "{description} must be rejected"
        );
    }

    let mut overlapping = support::Golden::parse(include_str!(
        "browser_parity/xml/subgrid/subgrid_axes_horizontal_tb_parallel__border_box_ltr.xml"
    ))
    .expect("subgrid overlap golden should parse");
    overlapping.expectations.children[1].y = Some(0.0);
    assert!(
        assert_subgrid_axis_fixture_topology(
            &overlapping,
            Path::new("xml/subgrid/subgrid_axes_horizontal_tb_parallel__border_box_ltr.xml"),
        )
        .is_err(),
        "overlapping top-level case expectations must be rejected"
    );
}

#[test]
fn block_axis_fixture_matrix_rejects_missing_duplicate_misplaced_and_topology_bypassed_paths() {
    let expected = block_axis_expected_paths();

    assert!(block_axis_fixture_paths(expected.iter().skip(1).cloned()).is_err());

    let mut duplicate = expected.clone();
    duplicate.push(expected[0].clone());
    assert!(block_axis_fixture_paths(duplicate).is_err());

    let mut misplaced = expected.clone();
    misplaced[0] = PathBuf::from("xml/other/block_axes_horizontal_tb__border_box_ltr.xml");
    assert!(block_axis_fixture_paths(misplaced).is_err());

    let topology_bypass = support::Golden::parse(
        r#"
        <test name="block_axes_horizontal_tb__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input>
            <div display="block" writing-mode="horizontal-tb">
              <div display="block" width="20px" height="10px" margin-left="13px" />
            </div>
          </input>
          <expectations>
            <node x="0" y="0" width="20" height="10">
              <node x="13" y="0" width="20" height="10" />
            </node>
          </expectations>
        </test>
        "#,
    )
    .expect("topology bypass fixture should parse");
    assert!(
        assert_block_axis_fixture_topology(
            &topology_bypass,
            Path::new("xml/block/block_axes_horizontal_tb__border_box_ltr.xml"),
        )
        .is_err()
    );
}

#[test]
fn calc_fixture_family_rejects_misplaced_duplicate_variant() {
    let family = "block/block_calc_width_margin";
    let candidates = [
        "xml/block/block_calc_width_margin__border_box_ltr.xml",
        "xml/block/block_calc_width_margin__border_box_rtl.xml",
        "xml/block/block_calc_width_margin__content_box_ltr.xml",
        "xml/block/block_calc_width_margin__content_box_rtl.xml",
        "xml/other/block_calc_width_margin__border_box_ltr.xml",
    ]
    .map(PathBuf::from);

    assert!(
        calc_fixture_family_paths(family, candidates).is_err(),
        "a misplaced duplicate variant must be rejected"
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

fn assert_fri_03_fixture_topology(golden: &support::Golden, path: &Path) -> Result<(), String> {
    match fri_03_family(path) {
        Some(Fri03Family::BlockMargin) => assert_fri_03_block_margin_topology(golden),
        Some(Fri03Family::Order) => assert_fri_03_order_topology(golden, path),
        Some(Fri03Family::FlexItemRoot) => assert_fri_03_flex_item_root_topology(golden, path),
        None => Err(format!("{} is not an owned FRI-03 fixture", path.display())),
    }
}

fn assert_fri_03_block_margin_topology(golden: &support::Golden) -> Result<(), String> {
    if golden.root.kind != support::NodeKind::Div
        || golden.root.style.get("display") != Some("flex")
        || golden.root.children.len() != 2
        || golden.root.children[1].children.len() != 1
        || golden.expectations.children.len() != 2
        || golden.expectations.children[1].children.len() != 1
        || golden.expectations.children[1].children[0].y != Some(1.0)
    {
        return Err(
            "block-margin fixture must retain the flex boundary and nested-child y=1 topology"
                .to_string(),
        );
    }
    Ok(())
}

fn assert_fri_03_order_topology(golden: &support::Golden, path: &Path) -> Result<(), String> {
    let source = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.split_once("__"))
        .map(|(source, _)| source)
        .ok_or_else(|| format!("{} must have a source and variant", path.display()))?;
    let expected_display = match source {
        "fri03_order_modified_flex" => "flex",
        "fri03_order_modified_grid" => "grid",
        "fri03_order_modified_lanes" => "grid-lanes",
        _ => return Err(format!("{source} is not an FRI-03 order source")),
    };

    if golden.root.kind != support::NodeKind::Div
        || golden.root.style.get("display") != Some(expected_display)
        || golden.root.children.len() != 4
        || golden.expectations.children.len() != 4
    {
        return Err("order fixture must retain its four-item source topology".to_string());
    }
    if expected_display != "flex"
        && (golden.root.style.get("grid-template-columns") != Some("20px 20px 20px 20px")
            || golden.root.style.get("grid-template-rows") != Some("20px"))
    {
        return Err("grid order fixture must retain its four definite columns".to_string());
    }
    if golden
        .root
        .children
        .iter()
        .map(|child| child.style.get("order").unwrap_or("0"))
        .collect::<Vec<_>>()
        != ["2", "-1", "2", "0"]
        || golden.root.children.iter().any(|child| {
            child.kind != support::NodeKind::Div
                || child.style.get("display") != Some("flex")
                || child.style.width() != Some("20px".to_string())
                || child.style.get("height") != Some("20px")
                || !child.children.is_empty()
        })
    {
        return Err(
            "order fixture must retain signed order on four source-ordered items".to_string(),
        );
    }

    let expected_x = if path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.ends_with("_rtl"))
    {
        [20.0, 60.0, 0.0, 40.0]
    } else {
        [40.0, 0.0, 60.0, 20.0]
    };
    let actual_x = golden
        .expectations
        .children
        .iter()
        .map(|child| child.x)
        .collect::<Vec<_>>();
    if actual_x != expected_x.map(Some)
        || golden.expectations.children.iter().any(|child| {
            child.y != Some(0.0)
                || child.width != Some(20.0)
                || child.height != Some(20.0)
                || !child.children.is_empty()
        })
    {
        return Err(
            "order fixture expectations must stay source-indexed while geometry follows order"
                .to_string(),
        );
    }
    Ok(())
}

fn assert_fri_03_flex_item_root_topology(
    golden: &support::Golden,
    path: &Path,
) -> Result<(), String> {
    let source = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.split_once("__"))
        .map(|(source, _)| source)
        .ok_or_else(|| format!("{} must have a source and variant", path.display()))?;
    let (expected_width, expected_host) = match source {
        "grid_available_space_greater_than_max_content" => {
            (support::Available::Definite(400.0), 160.0)
        }
        "grid_available_space_smaller_than_max_content" => {
            (support::Available::Definite(80.0), 80.0)
        }
        "grid_available_space_smaller_than_min_content" => {
            (support::Available::Definite(60.0), 80.0)
        }
        "chrome_issue_325928327" => (support::Available::MaxContent, 40.0),
        _ => return Err(format!("{source} is not an FRI-03 flex-item-root source")),
    };
    let expected_direction = if path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.ends_with("_rtl"))
    {
        surgeist_layout::Direction::Rtl
    } else {
        surgeist_layout::Direction::Ltr
    };

    if golden.viewport.width != expected_width
        || golden.viewport.height != support::Available::MaxContent
        || golden.root.kind != support::NodeKind::Div
        || golden.root.style.get("display") != Some("grid")
        || golden.expectations.width != Some(expected_host)
    {
        return Err(
            "flex-item-root fixture must retain separate viewport and host geometry".to_string(),
        );
    }
    match golden.viewport.root_context {
        support::RootContext::FlexItem {
            parent_axes,
            host_inline_size,
        } if parent_axes.writing_mode() == surgeist_layout::WritingMode::HorizontalTb
            && parent_axes.direction() == expected_direction
            && parent_axes.inline_axis() == surgeist_layout::PhysicalAxis::Horizontal
            && host_inline_size == expected_host => {}
        context => {
            return Err(format!(
                "flex-item-root fixture must retain explicit horizontal parent context and host allocation, got {context:?}"
            ));
        }
    }
    Ok(())
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

fn assert_axis_fixture_paths_reject_invalid_inventory(expected: Vec<PathBuf>) {
    let checker = if expected[0]
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("grid_lanes_axes_"))
    {
        grid_lanes_axis_fixture_paths as fn(Vec<PathBuf>) -> Result<BTreeSet<PathBuf>, String>
    } else {
        subgrid_axis_fixture_paths as fn(Vec<PathBuf>) -> Result<BTreeSet<PathBuf>, String>
    };

    assert!(checker(expected.iter().skip(1).cloned().collect()).is_err());
    let mut duplicate = expected.clone();
    duplicate.push(expected[0].clone());
    assert!(checker(duplicate).is_err());
    let mut misplaced = expected.clone();
    let file = misplaced[0].file_name().unwrap().to_owned();
    misplaced[0] = PathBuf::from("xml/other").join(file);
    assert!(checker(misplaced).is_err());
    let mut extra = expected;
    let parent = extra[0].parent().unwrap().to_owned();
    let prefix = if extra[0]
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("grid_lanes_axes_"))
    {
        "grid_lanes_axes_"
    } else {
        "subgrid_axes_"
    };
    extra.push(parent.join(format!("{prefix}extra__border_box_ltr.xml")));
    assert!(checker(extra).is_err());
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

fn grid_axis_test_golden(
    root_style: &str,
    first_child_style: &str,
    second_child_style: &str,
    first_child_is_text: bool,
) -> support::Golden {
    let first_child = if first_child_is_text {
        format!("<text {first_child_style}>text</text>")
    } else {
        format!("<div {first_child_style}/>")
    };
    support::Golden::parse(&format!(
        r#"
        <test name="grid_axes_horizontal_tb_parallel__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input>
            <div {root_style}>
              {first_child}
              <div {second_child_style}/>
            </div>
          </input>
          <expectations>
            <node x="0" y="0" width="70" height="110">
              <node x="0" y="0" width="30" height="50" />
              <node x="30" y="50" width="40" height="60" />
            </node>
          </expectations>
        </test>
        "#,
    ))
    .expect("grid-axis test golden should parse")
}

fn grid_lanes_axis_test_golden(
    root_display: &str,
    columns_display: &str,
    rows_display: &str,
    columns_rows_tracks: &str,
    columns_child_mode: &str,
    rows_child_mode: &str,
) -> support::Golden {
    support::Golden::parse(&format!(
        r#"
        <test name="grid_lanes_axes_horizontal_tb_parallel__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input>
            <div display="{root_display}">
              <div display="{columns_display}" writing-mode="horizontal-tb" grid-auto-flow="column" grid-template-rows="{columns_rows_tracks}" width="70px" height="110px">
                <div writing-mode="{columns_child_mode}" grid-row-start="1" grid-row-end="2" />
                <div writing-mode="{columns_child_mode}" grid-row-start="2" grid-row-end="3" />
              </div>
              <div display="{rows_display}" writing-mode="horizontal-tb" grid-auto-flow="row" grid-template-columns="30px 40px" width="70px" height="110px">
                <div writing-mode="{rows_child_mode}" grid-column-start="1" grid-column-end="2" />
                <div writing-mode="{rows_child_mode}" grid-column-start="2" grid-column-end="3" />
              </div>
            </div>
          </input>
          <expectations>
            <node x="0" y="0" width="70" height="220">
              <node x="0" y="0" width="70" height="110"><node x="0" y="0" width="30" height="50" /><node x="0" y="50" width="40" height="60" /></node>
              <node x="0" y="110" width="70" height="110"><node x="0" y="0" width="30" height="50" /><node x="30" y="50" width="40" height="60" /></node>
            </node>
          </expectations>
        </test>
        "#,
    ))
    .expect("grid-lanes axis test golden should parse")
}

fn grid_lanes_axis_text_child_golden() -> support::Golden {
    grid_lanes_axis_item_golden(
        "<text grid-row-start=\"1\" grid-row-end=\"2\">text</text>",
        "<div grid-row-start=\"2\" grid-row-end=\"3\" />",
    )
}

fn grid_lanes_axis_item_golden(
    columns_first_item: &str,
    columns_second_item: &str,
) -> support::Golden {
    support::Golden::parse(&format!(
        r#"
        <test name="grid_lanes_axes_horizontal_tb_parallel__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input><div display="block">
            <div display="grid-lanes" writing-mode="horizontal-tb" grid-auto-flow="column" grid-template-rows="50px 60px" width="70px" height="110px">{columns_first_item}{columns_second_item}</div>
            <div display="grid-lanes" writing-mode="horizontal-tb" grid-auto-flow="row" grid-template-columns="30px 40px" width="70px" height="110px"><div grid-column-start="1" grid-column-end="2" /><div grid-column-start="2" grid-column-end="3" /></div>
          </div></input>
          <expectations><node x="0" y="0" width="70" height="220"><node x="0" y="0" width="70" height="110"><node x="0" y="0" width="30" height="50" /><node x="0" y="50" width="40" height="60" /></node><node x="0" y="110" width="70" height="110"><node x="0" y="0" width="30" height="50" /><node x="30" y="50" width="40" height="60" /></node></node></expectations>
        </test>
        "#,
    ))
    .expect("grid-lanes item golden should parse")
}

fn subgrid_axis_test_golden(
    root_display: &str,
    parent_display: &str,
    subgrid_display: &str,
    parent_columns: &str,
    parent_rows: &str,
    columns_child_mode: &str,
    rows_child_mode: &str,
) -> support::Golden {
    support::Golden::parse(&format!(
        r#"
        <test name="subgrid_axes_horizontal_tb_parallel__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input><div display="{root_display}">
            <div display="{parent_display}" writing-mode="horizontal-tb" grid-template-columns="{parent_columns}" grid-template-rows="{parent_rows}"><div display="{subgrid_display}" writing-mode="{columns_child_mode}" grid-template-columns="subgrid" grid-column-start="1" grid-column-end="3"><div grid-column-start="1" grid-column-end="2" /><div grid-column-start="2" grid-column-end="3" /></div></div>
            <div display="{parent_display}" writing-mode="horizontal-tb" grid-template-columns="{parent_columns}" grid-template-rows="{parent_rows}"><div display="{subgrid_display}" writing-mode="{rows_child_mode}" grid-template-rows="subgrid" grid-row-start="1" grid-row-end="3"><div grid-row-start="1" grid-row-end="2" /><div grid-row-start="2" grid-row-end="3" /></div></div>
          </div></input>
          <expectations><node x="0" y="0" width="70" height="220"><node x="0" y="0" width="70" height="110"><node x="0" y="0" width="70" height="110"><node x="0" y="0" width="30" height="50" /><node x="30" y="0" width="40" height="50" /></node></node><node x="0" y="110" width="70" height="110"><node x="0" y="0" width="70" height="110"><node x="0" y="0" width="30" height="50" /><node x="0" y="50" width="30" height="60" /></node></node></node></expectations>
        </test>
        "#,
    ))
    .expect("subgrid axis test golden should parse")
}

fn subgrid_axis_absolute_item_golden() -> support::Golden {
    subgrid_axis_item_golden(
        "<div position=\"absolute\" grid-column-start=\"1\" grid-column-end=\"2\" />",
        "<div grid-column-start=\"2\" grid-column-end=\"3\" />",
    )
}

fn subgrid_axis_text_item_golden() -> support::Golden {
    subgrid_axis_item_golden(
        "<text grid-column-start=\"1\" grid-column-end=\"2\">text</text>",
        "<div grid-column-start=\"2\" grid-column-end=\"3\" />",
    )
}

fn subgrid_axis_item_golden(
    columns_first_item: &str,
    columns_second_item: &str,
) -> support::Golden {
    support::Golden::parse(&format!(
        r#"
        <test name="subgrid_axes_horizontal_tb_parallel__border_box_ltr" use-rounding="true">
          <viewport width="max-content" height="max-content" />
          <input><div display="block">
            <div display="grid" writing-mode="horizontal-tb" grid-template-columns="30px 40px" grid-template-rows="50px 60px"><div display="grid" writing-mode="horizontal-tb" grid-template-columns="subgrid" grid-column-start="1" grid-column-end="3">{columns_first_item}{columns_second_item}</div></div>
            <div display="grid" writing-mode="horizontal-tb" grid-template-columns="30px 40px" grid-template-rows="50px 60px"><div display="grid" writing-mode="horizontal-tb" grid-template-rows="subgrid" grid-row-start="1" grid-row-end="3"><div grid-row-start="1" grid-row-end="2" /><div grid-row-start="2" grid-row-end="3" /></div></div>
          </div></input>
          <expectations><node x="0" y="0" width="70" height="220"><node x="0" y="0" width="70" height="110"><node x="0" y="0" width="70" height="110"><node x="0" y="0" width="30" height="50" /><node x="30" y="0" width="40" height="50" /></node></node><node x="0" y="110" width="70" height="110"><node x="0" y="0" width="70" height="110"><node x="0" y="0" width="30" height="50" /><node x="0" y="50" width="30" height="60" /></node></node></node></expectations>
        </test>
        "#,
    ))
    .expect("subgrid item golden should parse")
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
fn all_checked_in_browser_parity_xml_has_generator_provenance() {
    let fixtures = support::fixture_files("xml").expect("fixtures should load");
    assert!(
        !fixtures.is_empty(),
        "expected at least one browser parity XML fixture"
    );

    for fixture in fixtures {
        let raw = std::fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("{} should read: {error}", fixture.display()));
        assert!(
            raw.trim_start()
                .starts_with("<!-- generated-by: surgeist-layout-generate "),
            "{} is missing surgeist-layout-generate provenance",
            fixture.display()
        );
    }
}

#[test]
fn browser_parity_corpus_manifest_exists() {
    let manifest =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity/corpus.toml");
    let raw = std::fs::read_to_string(&manifest).unwrap_or_else(|error| {
        panic!(
            "expected browser parity corpus manifest at {}: {error}",
            manifest.display()
        )
    });

    assert!(raw.contains("schema_version = 2"));
    assert!(raw.contains("[browser]"));
    assert!(raw.contains("[browser.launch]"));
    assert!(raw.contains("[generation_reports.full]"));
    assert!(raw.contains("[source_roots.taffy]"));
    assert!(raw.contains("[source_roots.surgeist]"));
}

#[test]
fn browser_parity_html_corpus_inventory_is_documented() {
    let html_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity/html");
    let fixtures = support::fixture_files_in(&html_root, "html")
        .expect("HTML parity fixtures should be readable");

    let taffy_plus_local_count = fixtures
        .iter()
        .filter(|fixture| !is_under_suite(fixture, "subgrid"))
        .filter(|fixture| !is_under_suite(fixture, "grid-lanes"))
        .count();
    let subgrid_count = fixtures
        .iter()
        .filter(|fixture| is_under_suite(fixture, "subgrid"))
        .count();
    let grid_lanes_count = fixtures
        .iter()
        .filter(|fixture| is_under_suite(fixture, "grid-lanes"))
        .count();

    assert_eq!(taffy_plus_local_count, 1186);
    assert_eq!(subgrid_count, 219);
    assert_eq!(grid_lanes_count, 27);
    assert_eq!(fixtures.len(), 1432);

    for source in [
        "flex/fri03_order_modified_flex.html",
        "grid/fri03_order_modified_grid.html",
        "grid-lanes/fri03_order_modified_lanes.html",
    ] {
        assert!(
            fixtures.contains(&html_root.join(source)),
            "missing FRI-03 source {source}"
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
fn browser_parity_generation_report_counts_full_scope() {
    let report = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity/xml/generation-reports/all.json");
    let raw = std::fs::read_to_string(&report)
        .unwrap_or_else(|error| panic!("{} should read: {error}", report.display()));

    let report_json: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("{} should parse as JSON: {error}", report.display()));

    assert_eq!(report_json["filter"], serde_json::Value::Null);
    assert_eq!(report_json["summary"]["generated"], 5712);
    assert_eq!(report_json["summary"]["unsupported"], 16);
    assert_eq!(report_json["summary"]["expected_fail"], 0);
    assert_eq!(report_json["summary"]["quarantined"], 0);
    assert_eq!(report_json["summary"]["failed_to_generate"], 0);
    assert!(
        !raw.contains("skipped"),
        "generation report must use explicit buckets, not a generic skipped bucket"
    );
    assert_eq!(
        report_bucket_len(&report_json, "generated"),
        5712,
        "generated bucket length must match its summary"
    );
    assert_eq!(
        report_bucket_len(&report_json, "unsupported"),
        16,
        "unsupported bucket length must match its summary"
    );
    assert_eq!(
        report_bucket_len(&report_json, "expected_fail"),
        0,
        "expected_fail bucket length must match its summary"
    );
    assert_eq!(
        report_bucket_len(&report_json, "quarantined"),
        0,
        "quarantined bucket length must match its summary"
    );
    assert_eq!(
        report_bucket_len(&report_json, "failed_to_generate"),
        0,
        "failed_to_generate bucket length must match its summary"
    );

    let reported_outputs = report_outputs(&report_json);
    let unique_reported_outputs = reported_outputs.iter().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        reported_outputs.len(),
        unique_reported_outputs.len(),
        "generated report outputs must be unique"
    );

    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let actual_outputs = support::fixture_files_in(&corpus_root.join("xml"), "xml")
        .expect("XML fixtures should be readable")
        .into_iter()
        .map(|fixture| {
            fixture
                .strip_prefix(&corpus_root)
                .unwrap_or_else(|error| {
                    panic!("{} should be under corpus root: {error}", fixture.display())
                })
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/")
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        unique_reported_outputs, actual_outputs,
        "generated full report outputs must match checked-in XML exactly"
    );

    for output in [
        "xml/flex/fri03_order_modified_flex__border_box_ltr.xml",
        "xml/flex/fri03_order_modified_flex__border_box_rtl.xml",
        "xml/flex/fri03_order_modified_flex__content_box_ltr.xml",
        "xml/flex/fri03_order_modified_flex__content_box_rtl.xml",
        "xml/grid/fri03_order_modified_grid__border_box_ltr.xml",
        "xml/grid/fri03_order_modified_grid__border_box_rtl.xml",
        "xml/grid/fri03_order_modified_grid__content_box_ltr.xml",
        "xml/grid/fri03_order_modified_grid__content_box_rtl.xml",
        "xml/grid-lanes/fri03_order_modified_lanes__border_box_ltr.xml",
        "xml/grid-lanes/fri03_order_modified_lanes__border_box_rtl.xml",
        "xml/grid-lanes/fri03_order_modified_lanes__content_box_ltr.xml",
        "xml/grid-lanes/fri03_order_modified_lanes__content_box_rtl.xml",
    ] {
        assert!(
            unique_reported_outputs.contains(output),
            "missing FRI-03 generated output {output}"
        );
    }
}

#[test]
fn browser_parity_generation_report_inventory_is_full_only() {
    let report_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity/xml/generation-reports");
    let reports = support::fixture_files_in(&report_root, "json")
        .expect("generation reports should be readable");
    let report_basenames = reports
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| panic!("{} should have a UTF-8 basename", path.display()))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(report_basenames, BTreeSet::from(["all.json"]));
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

fn report_bucket_len(report_json: &serde_json::Value, bucket: &str) -> usize {
    let bucket_len = report_json[bucket]
        .as_array()
        .unwrap_or_else(|| panic!("{bucket} report entries should be an array"))
        .len();
    let summary_len = report_json["summary"][bucket]
        .as_u64()
        .unwrap_or_else(|| panic!("{bucket} summary count should be a number"))
        as usize;
    assert_eq!(
        bucket_len, summary_len,
        "{bucket} bucket length must match its summary"
    );
    bucket_len
}

fn report_outputs(report_json: &serde_json::Value) -> Vec<String> {
    report_json["generated"]
        .as_array()
        .expect("generated report entries should be an array")
        .iter()
        .map(|entry| {
            entry["output"]
                .as_str()
                .expect("generated report entries should include output paths")
                .to_string()
        })
        .collect()
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

fn is_under_suite(path: &Path, suite: &str) -> bool {
    path.components()
        .any(|component| component.as_os_str().to_string_lossy() == suite)
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

#[test]
fn fri06_c08_r0_control_probe_matrix_is_exact_72_plus_24_rows() {
    const INLINE_COLUMN_SOURCES: [&str; 12] = [
        "subgrid_baseline_inline_column_inner_col1_first",
        "subgrid_baseline_inline_column_inner_col1_last",
        "subgrid_baseline_inline_column_inner_col2_first",
        "subgrid_baseline_inline_column_inner_col2_last",
        "subgrid_baseline_inline_column_outer_col3_first",
        "subgrid_baseline_inline_column_outer_col3_last",
        "subgrid_baseline_inline_column_parent_col1_first",
        "subgrid_baseline_inline_column_parent_col1_last",
        "subgrid_baseline_inline_column_parent_col2_first",
        "subgrid_baseline_inline_column_parent_col2_last",
        "subgrid_baseline_inline_column_parent_col3_first",
        "subgrid_baseline_inline_column_parent_col3_last",
    ];
    const NESTED_BLOCK_SOURCES: [&str; 12] = [
        "subgrid_baseline_nested_block_inner_row1_first",
        "subgrid_baseline_nested_block_inner_row1_last",
        "subgrid_baseline_nested_block_inner_row2_first",
        "subgrid_baseline_nested_block_inner_row2_last",
        "subgrid_baseline_nested_block_outer_row3_first",
        "subgrid_baseline_nested_block_outer_row3_last",
        "subgrid_baseline_nested_block_parent_row1_first",
        "subgrid_baseline_nested_block_parent_row1_last",
        "subgrid_baseline_nested_block_parent_row2_first",
        "subgrid_baseline_nested_block_parent_row2_last",
        "subgrid_baseline_nested_block_parent_row3_first",
        "subgrid_baseline_nested_block_parent_row3_last",
    ];
    const LTR_VARIANTS: [&str; 2] = ["border_box_ltr", "content_box_ltr"];
    const RTL_VARIANTS: [&str; 2] = ["border_box_rtl", "content_box_rtl"];
    const ALL_VARIANTS: [&str; 4] = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];

    let rows = include_str!("../../plans/P01-layout/P01-I06-S01-C10-public-comparison-census.tsv")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .skip(1)
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .map(|fields| format!("{}\t{}", fields[1], fields[2]))
        .collect::<BTreeSet<_>>();
    let expected_rows = |sources: &[&str], variants: &[&str]| {
        sources
            .iter()
            .flat_map(|source| {
                variants
                    .iter()
                    .map(move |variant| format!("html/subgrid/{source}.html\t{variant}"))
            })
            .collect::<BTreeSet<_>>()
    };

    let control = expected_rows(&INLINE_COLUMN_SOURCES, &LTR_VARIANTS)
        .into_iter()
        .chain(expected_rows(&NESTED_BLOCK_SOURCES, &ALL_VARIANTS))
        .collect::<BTreeSet<_>>();
    let masked_rtl = expected_rows(&INLINE_COLUMN_SOURCES, &RTL_VARIANTS);
    let selected = rows
        .iter()
        .filter(|row| {
            row.contains("/subgrid_baseline_inline_column_")
                || row.contains("/subgrid_baseline_nested_block_")
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let complete_probe = control
        .iter()
        .chain(masked_rtl.iter())
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(control.len(), 72);
    assert_eq!(masked_rtl.len(), 24);
    assert!(control.is_disjoint(&masked_rtl));
    assert_eq!(complete_probe.len(), 96);
    assert_eq!(selected, complete_probe);
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
        "ltr" => (0, "left", 42, [81, 42, 74, 90]),
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
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="28px" height="16px"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="32px" height="16px"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="36px" height="16px"/>
      <div source-tag="span" display="inline-block"{box_attr} direction="{direction}" font-family="monospace" font-size="16px" line-height="20px" width="40px" height="16px"/>
      <atomic-placeholder child-index="3" bidi-level="{bidi_level}" following-break="allowed"/>
      <atomic-placeholder child-index="4" bidi-level="{bidi_level}" following-break="allowed"/>
      <atomic-placeholder child-index="5" bidi-level="{bidi_level}" following-break="allowed"/>
      <atomic-placeholder child-index="6" bidi-level="{bidi_level}" following-break="prohibited"/>
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
      <node x="90" y="42" width="40" height="16"/>
    </node>
  </expectations>
</test>"#,
        atomic_x[0], atomic_x[1], atomic_x[2]
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
    r#"<test name="fri06_float_shape_exclusion__border_box_ltr" use-rounding="false">
  <viewport width="180px" height="max-content"/>
  <input>
    <div source-tag="div" layout-ready-inline-root="true" display="block" direction="ltr" font-family="monospace" font-size="16px" line-height="20px" width="180px">
      <div source-tag="span" display="block" float="left" float-exclusion="shape" width="44px" height="60px">
        <shape-bands>
          <shape-band band-minimum="0" band-maximum="21.2" interval-minimum="0" interval-maximum="44"/>
          <shape-band band-minimum="21.2" band-maximum="37.2" interval-minimum="0" interval-maximum="44"/>
        </shape-bands>
      </div>
      <text layout-input="inline-text">
        <segment id="2" inline-extent="48.1640625" inline-baseline="14.8" inline-line-height="20" bidi-level="0" whitespace-edge="preserve" following-break="prohibited"/>
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
    <node x="0" y="0" width="180" height="60.5">
      <node x="0" y="0" width="44" height="60"/>
      <node>
        <range-inks>
          <range-ink source_segment_id="2" line_index="0" physical_start_edge="left" start="44" advance="48.1640625"/>
        </range-inks>
      </node>
      <node x="92.16406" y="0" width="34" height="16"/>
      <node x="126.16406" y="0" width="38" height="16"/>
      <node x="44" y="21.2" width="42" height="16"/>
      <node x="86" y="21.2" width="46" height="16"/>
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
fn fri06_c08r_lineage_support_has_no_name_or_expectation_compatibility_path() {
    let source = include_str!("browser_parity/support.rs");
    for identifier in [
        "apply_fri06_c08_finite_adapter",
        "fri06_c08_fixture_name",
        "mark_fri06_c08_grid_inline_text_wrappers",
        "mark_fri06_c08_four_run_grid_wrapper",
        "lower_fri06_c08_bidi_inline_boundaries",
        "insert_fri06_c08_atomic_line_strut",
        "insert_fri06_c08_float_line_strut",
        "fri06_c08_synthetic_boundary",
        "fixture_synthetic",
    ] {
        assert!(
            !source.contains(identifier),
            "removed compatibility identifier remains: {identifier}"
        );
    }
    let parser = source
        .split_once("impl Golden {")
        .and_then(|(_, rest)| rest.split_once("pub fn fixture_files("))
        .map(|(parser, _)| parser)
        .expect("Golden parser source");
    assert!(parser.contains("let root = parse_node(one_element_child(input)?)?;"));
    assert!(parser.contains("validate_fri06_c08r_explicit_input(&root, true)?;"));
    assert!(
        parser.contains("let expectations = parse_expectation(one_element_child(expectations)?)?;")
    );
    assert!(!parser.contains("&name, &mut root"));
    assert!(!parser.contains("&mut expectations"));
}

#[test]
fn fri06_c08r_final_activation_union_browser_passes_without_substitutes() {
    const KNOWN_CHROME_FAILURE_SUBSTITUTES: [&str; 0] = [];
    let census =
        include_str!("../../plans/P01-layout/P01-I06-S01-C10-public-comparison-census.tsv");
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut outputs = BTreeSet::new();
    let mut failures = Vec::new();
    for line in census.lines().filter(|line| !line.starts_with('#')).skip(1) {
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 6, "activation row must retain six fields");
        let source = fields[1]
            .strip_prefix("html/")
            .and_then(|source| source.strip_suffix(".html"))
            .expect("normalized activation source");
        let output = format!(
            "tests/layout/browser_parity/xml/{source}__{}.xml",
            fields[2]
        );
        assert!(outputs.insert(output.clone()), "duplicate activation row");
        match support::Golden::parse_file(repository.join(&output)) {
            Ok(golden) => {
                if let Err(error) = support::assert_surgeist_matches(&golden) {
                    failures.push(format!("{output}: {error}"));
                }
            }
            Err(error) => failures.push(format!("{output}: {error}")),
        }
    }
    assert_eq!(outputs.len(), 388);
    assert!(KNOWN_CHROME_FAILURE_SUBSTITUTES.is_empty());
    assert!(
        failures.is_empty(),
        "activation rows without a reviewed substitute failed:\n{}",
        failures.join("\n")
    );
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
                [15.0, 60.0]
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
                [15.0, 45.0]
            },
        )
    };
    let edge = if direction == "ltr" { "left" } else { "right" };
    format!(
        r#"<test name="{source}__{variant}" use-rounding="true">
  <viewport width="max-content" height="max-content"/>
  <input>
    <div layout-ready-inline-root="true" display="grid" box-sizing="{box_sizing}" direction="{direction}" align-items="baseline"{root_tracks} font-family="ahem" font-size="15px" line-height="15px">
      <div display="grid" align-items="baseline"{subgrid_tracks}>
        <div display="grid" align-items="baseline"{item_tracks}>
          <text layout-input="inline-text">
            <segment id="0" inline-extent="15" inline-baseline="12" inline-line-height="15" bidi-level="{bidi_level}" whitespace-edge="preserve" following-break="prohibited"/>
          </text>
        </div>
        <div display="grid" align-items="baseline"{item_tracks} font-size="30px" line-height="30px">
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
              <range-ink source_segment_id="0" line_index="0" physical_start_edge="{edge}" start="{}" advance="30"/>
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
      <div display="inline-block" box-sizing="{box_sizing}" direction="{direction}" width="24px" height="18px"/>
      <div display="inline-block" box-sizing="{box_sizing}" direction="{direction}" width="30px" height="18px"/>
      <atomic-placeholder child-index="1" bidi-level="{bidi_level}" following-break="allowed"/>
      <atomic-placeholder child-index="2" bidi-level="{bidi_level}" following-break="prohibited"/>
      <atomic-placeholder child-index="3" bidi-level="{bidi_level}" following-break="prohibited"/>
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
        .expect_err("an altered fixture name must not activate the finite strut");

    let altered_topology = exact.replacen(
        r#"child-index="1" bidi-level="0" following-break="allowed""#,
        r#"child-index="1" bidi-level="0" following-break="prohibited""#,
        1,
    );
    assert!(
        support::Golden::parse(&altered_topology).is_err(),
        "an altered break topology must fail before the finite strut activates"
    );
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
