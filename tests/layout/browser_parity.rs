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

    assert_eq!(
        taffy_plus_local_count, 1150,
        "expected the Taffy baseline plus thirty-one Surgeist constrained additions and sixteen BR coverage fixtures, including four layout-ready vertical BR fixtures"
    );
    assert_eq!(subgrid_count, 210);
    assert_eq!(grid_lanes_count, 16);
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
    assert_eq!(report_json["summary"]["generated"], 5148);
    assert_eq!(report_json["summary"]["unsupported"], 356);
    assert_eq!(report_json["summary"]["expected_fail"], 0);
    assert_eq!(report_json["summary"]["quarantined"], 0);
    assert_eq!(report_json["summary"]["failed_to_generate"], 0);
    assert!(
        !raw.contains("skipped"),
        "generation report must use explicit buckets, not a generic skipped bucket"
    );
    assert_eq!(
        report_bucket_len(&report_json, "generated"),
        5148,
        "generated bucket length must match its summary"
    );
    assert_eq!(
        report_bucket_len(&report_json, "unsupported"),
        356,
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
