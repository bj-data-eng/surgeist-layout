//! Preserved layout helper and XML contract tests.

use super::*;
use crate::adapter::{
    GRID_TEMPLATE_AREA_CAPTURE_SCRIPT, TEST_HELPER_SOURCE, browser_document_write_script,
    browser_fixture_document, fixture_cases,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

fn fri08_c05_inputs_synthetic_grid_node() -> Value {
    json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "grid",
            "gridTemplateRows": [
                {"kind": "scalar", "unit": "px", "value": 20},
                {"kind": "scalar", "unit": "px", "value": 20}
            ],
            "gridTemplateColumns": [
                {"kind": "scalar", "unit": "px", "value": 30},
                {"kind": "scalar", "unit": "px", "value": 50}
            ],
            "gridTemplateAreas": [["head", "head"], ["nav", "main"]]
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 80, "height": 40, "scrollWidth": 80, "scrollHeight": 40},
        "unroundedLayout": {"x": 0, "y": 0, "width": 80, "height": 40, "scrollWidth": 80, "scrollHeight": 40},
        "naivelyRoundedLayout": {"clientWidth": 80, "clientHeight": 40},
        "children": [
            {
                "style": {
                    "display": "block",
                    "gridColumnStart": {"kind": "named-line", "name": "head-start", "value": 0},
                    "gridColumnEnd": {"kind": "named-line", "name": "head-end", "value": 0},
                    "gridRowStart": {"kind": "named-line", "name": "head-start", "value": 0},
                    "gridRowEnd": {"kind": "named-line", "name": "head-end", "value": 0}
                },
                "smartRoundedLayout": {"x": 0, "y": 0, "width": 80, "height": 20, "scrollWidth": 80, "scrollHeight": 20},
                "unroundedLayout": {"x": 0, "y": 0, "width": 80, "height": 20, "scrollWidth": 80, "scrollHeight": 20},
                "naivelyRoundedLayout": {"clientWidth": 80, "clientHeight": 20},
                "children": []
            },
            {
                "style": {
                    "display": "block",
                    "gridColumnStart": {"kind": "named-line", "name": "main-start", "value": 0},
                    "gridColumnEnd": {"kind": "named-line", "name": "main-end", "value": 0},
                    "gridRowStart": {"kind": "named-line", "name": "main-start", "value": 0},
                    "gridRowEnd": {"kind": "named-line", "name": "main-end", "value": 0}
                },
                "smartRoundedLayout": {"x": 30, "y": 20, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
                "unroundedLayout": {"x": 30, "y": 20, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
                "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 20},
                "children": []
            }
        ]
    })
}

#[test]
fn fri08_c05_inputs_synthetic_generator_values_reach_public_layout_independent_of_identity() {
    let node = fri08_c05_inputs_synthetic_grid_node();
    let original_xml = generate_xml("original/source__border_box_ltr", &node);
    let renamed_xml = generate_xml("renamed/source__content_box_rtl", &node);
    let original = browser_parity_support::Golden::parse(&original_xml)
        .expect("synthetic finite generator output should parse");
    let renamed = browser_parity_support::Golden::parse(&renamed_xml)
        .expect("renamed synthetic finite generator output should parse");

    assert_eq!(original.root, renamed.root);
    assert_eq!(
        original.root.style.get("grid-template-areas"),
        Some("head head / nav main")
    );
    browser_parity_support::assert_surgeist_matches(&original)
        .expect("synthetic generator values should reach finite public layout");
    browser_parity_support::assert_surgeist_matches(&renamed)
        .expect("fixture identity must not affect finite public layout");
}

#[test]
fn fri05_c06_helper_captures_computed_overflow_axes() {
    assert!(TEST_HELPER_SOURCE.contains("overflowX: parseEnum(computedStyle.overflowX)"));
    assert!(TEST_HELPER_SOURCE.contains("overflowY: parseEnum(computedStyle.overflowY)"));
    assert!(!TEST_HELPER_SOURCE.contains("overflowX: parseEnum(styleValue(\"overflowX\"))"));
    assert!(!TEST_HELPER_SOURCE.contains("overflowY: parseEnum(styleValue(\"overflowY\"))"));
}

#[test]
fn fri05_c06_helper_captures_exact_computed_scroll_fields() {
    for field in [
        "overflowClipMargin",
        "scrollbarGutter",
        "scrollPaddingTop",
        "scrollPaddingRight",
        "scrollPaddingBottom",
        "scrollPaddingLeft",
        "scrollMarginTop",
        "scrollMarginRight",
        "scrollMarginBottom",
        "scrollMarginLeft",
        "scrollSnapType",
        "scrollSnapAlign",
        "scrollSnapStop",
    ] {
        assert!(
            TEST_HELPER_SOURCE.contains(&format!("{field}: computedStyle.{field}")),
            "helper does not capture computed {field}"
        );
    }
}

#[test]
fn fri04_c05_helper_serializer_canonical_percent_object_round_trips_and_serializes() {
    assert_eq!(
        dimension(&json!({"unit": "percent", "value": 0.1})),
        Some("10%".to_string())
    );

    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const document = { styleSheets: [] };
"#,
            TEST_HELPER_SOURCE,
            r#"
const canonical = parseSizingDimension("10%");
const roundTripped = parseSizingDimension(canonical);
if (JSON.stringify(roundTripped) !== JSON.stringify(canonical)) {
  throw new Error(`canonical percentage changed from ${JSON.stringify(canonical)} to ${JSON.stringify(roundTripped)}`);
}

class RawTypedOmPercent {
  constructor(value) {
    this.unit = "percent";
    this.value = value;
  }

  toString() {
    return `${this.value}%`;
  }
}

const rawTypedOm = parseSizingDimension(new RawTypedOmPercent(10));
if (JSON.stringify(rawTypedOm) !== JSON.stringify(canonical)) {
  throw new Error(`raw Typed OM percentage produced ${JSON.stringify(rawTypedOm)}`);
}
"#,
        ]
        .concat();

    run_bundled_helper_script("fri04-c05-canonical-percent", script);
}

#[test]
fn fri04_c05_helper_serializer_accepts_fixture_affine_calc_size_forms() {
    let script = [
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const document = { styleSheets: [] };
"#,
        TEST_HELPER_SOURCE,
        r#"
for (const raw of [
  "calc-size(auto, 10px + 20%)",
  "calc-size(auto, size * 0.5)",
  "calc-size(auto, 0.5 * size)",
  "calc-size(auto, size*0.5)",
  "calc-size(auto, 0.5*size)",
  "calc-size(auto, -0.5 * size)",
  "calc-size(auto, calc(10px + 20%))",
  "calc-size(auto, 0.5)",
]) {
  const actual = parseSizingDimension(raw);
  const expected = { unit: "sizing", value: raw };
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`fixture-supported calc-size ${raw} produced ${JSON.stringify(actual)}`);
  }
}
"#,
    ]
    .concat();

    run_bundled_helper_script("fri04-c05-accepted-affine", script);
}

#[test]
fn fri04_c05_helper_serializer_rejects_non_fixture_affine_calc_size_forms() {
    let script = [
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const document = { styleSheets: [] };
"#,
        TEST_HELPER_SOURCE,
        r#"
for (const raw of [
  "calc-size(auto, -size)",
  "calc-size(auto, +size)",
  "calc-size(auto, 10px+20%)",
  "calc-size(auto, 10px +20%)",
  "calc-size(auto, 10px+ 20%)",
  "calc-size(auto, size *0.5)",
  "calc-size(auto, size* 0.5)",
  "calc-size(auto, 0.5 *size)",
  "calc-size(auto, 0.5* size)",
  "calc-size(auto, 1e39px)",
  "calc-size(auto, 1e999px)",
  "calc-size(auto, 1e38px + 3e38px)",
  "calc-size(auto, calc(1e38px + 3e38px))",
  "calc-size(auto, size * 1e999)",
  "calc-size(auto, 1e999 * size)",
]) {
  const actual = parseSizingDimension(raw);
  if (actual !== undefined) {
    throw new Error(`fixture-rejected calc-size ${raw} produced ${JSON.stringify(actual)}`);
  }
}
"#,
    ]
    .concat();

    run_bundled_helper_script("fri04-c05-rejected-affine", script);
}

#[test]
fn fri04_c05_helper_serializer_helper_preserves_finite_sizing_tokens_exactly() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const document = { styleSheets: [] };
"#,
            TEST_HELPER_SOURCE,
            r#"
const parseOwnedSizing = typeof parseSizingDimension === "function"
  ? parseSizingDimension
  : parseDimension;

function assertEqual(name, actual, expected) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${name} expected ${expectedJson} but got ${actualJson}`);
  }
}

for (const [raw, expected, allowFrUnits] of [
  ["12px", { unit: "px", value: 12 }, false],
  ["25%", { unit: "percent", value: 0.25 }, false],
  ["calc(5px + 10%)", { unit: "calc", value: "calc(5px + 10%)" }, false],
  ["min(10px, max(20%, calc(5px + 10%)))", { unit: "sizing", value: "min(10px, max(20%, calc(5px + 10%)))" }, false],
  ["max(10px, min(20%, 30px))", { unit: "sizing", value: "max(10px, min(20%, 30px))" }, false],
  ["clamp(none, max(10px, 25%), 90px)", { unit: "sizing", value: "clamp(none, max(10px, 25%), 90px)" }, false],
  ["fit-content(max(10px, 25%))", { unit: "sizing", value: "fit-content(max(10px, 25%))" }, false],
  ["calc-size(auto, clamp(none, max(10px + 25%, size * 0.5), 100px))", { unit: "sizing", value: "calc-size(auto, clamp(none, max(10px + 25%, size * 0.5), 100px))" }, false],
  ["auto", { unit: "auto" }, false],
  ["none", { unit: "none" }, false],
  ["content", { unit: "content" }, false],
  ["min-content", { unit: "min-content" }, false],
  ["max-content", { unit: "max-content" }, false],
  ["stretch", { unit: "stretch" }, false],
  ["fit-content", { unit: "fit-content" }, false],
  ["contain", { unit: "contain" }, false],
  ["2fr", { unit: "fraction", value: 2 }, true],
]) {
  assertEqual(raw, parseOwnedSizing(raw, { allowFrUnits }), expected);
}

assertEqual(
  "calculated grid tracks",
  parseGridTrackDefinitions("minmax(calc(25% + 10px), calc(35% + 16px)) fit-content(calc(25% + 15px))"),
  [
    {
      kind: "function",
      name: "minmax",
      arguments: [
        { kind: "scalar", unit: "calc", value: "calc(25% + 10px)" },
        { kind: "scalar", unit: "calc", value: "calc(35% + 16px)" },
      ],
    },
    {
      kind: "function",
      name: "fit-content",
      arguments: [
        { kind: "scalar", unit: "calc", value: "calc(25% + 15px)" },
      ],
    },
  ]
);
"#,
        ]
        .concat();

    run_bundled_helper_script("fri04-c05-owned-sizing", script);
}

#[test]
fn fri04_c05_helper_serializer_helper_rejects_unsupported_syntax_and_box_fr() {
    let script = [
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const document = { styleSheets: [] };
"#,
        TEST_HELPER_SOURCE,
        r#"
const parseOwnedSizing = typeof parseSizingDimension === "function"
  ? parseSizingDimension
  : parseDimension;

for (const raw of [
  "calc(100% / 2)",
  "min()",
  "max(10px,)",
  "clamp(10px, 20px)",
  "fit-content(10px, 20px)",
  "calc-size(any, size)",
  "var(--size)",
  "min(10px, max(20px, 30px)",
  "min(10px) trailing",
  "1fr",
]) {
  const actual = parseOwnedSizing(raw, { allowFrUnits: false });
  if (actual !== undefined) {
    throw new Error(`unsupported box sizing token ${raw} produced ${JSON.stringify(actual)}`);
  }
}

for (const raw of ["-1fr", "NaNfr", "Infinityfr"]) {
  const actual = parseOwnedSizing(raw, { allowFrUnits: true });
  if (actual !== undefined) {
    throw new Error(`invalid track flex ${raw} produced ${JSON.stringify(actual)}`);
  }
}
"#,
    ]
    .concat();

    run_bundled_helper_script("fri04-c05-rejected-sizing", script);
}

#[test]
fn fri04_c05_helper_serializer_emits_exact_box_flex_and_track_attributes() {
    let node = json!({
        "style": {
            "flexBasis": {"unit": "content"},
            "size": {
                "width": {"unit": "sizing", "value": "min(calc(70% - 8px), 160px)"},
                "height": {"unit": "calc", "value": "calc(50% + 4px)"}
            },
            "minSize": {
                "width": {"unit": "sizing", "value": "max(calc(20% + 12px), 72px)"},
                "height": {"unit": "px", "value": 18}
            },
            "maxSize": {
                "width": {"unit": "sizing", "value": "clamp(90px, calc(50% + 4px), 166px)"},
                "height": {"unit": "sizing", "value": "fit-content(max(24px, 30%))"}
            },
            "gridTemplateColumns": [
                {
                    "kind": "function",
                    "name": "minmax",
                    "arguments": [
                        {"kind": "scalar", "unit": "sizing", "value": "calc(25% + 10px)"},
                        {"kind": "scalar", "unit": "sizing", "value": "calc(35% + 16px)"}
                    ]
                },
                {
                    "kind": "function",
                    "name": "fit-content",
                    "arguments": [
                        {"kind": "scalar", "unit": "sizing", "value": "calc(25% + 15px)"}
                    ]
                }
            ],
            "gridTemplateRows": [
                {"kind": "scalar", "unit": "sizing", "value": "max(20px, 10%)"}
            ]
        }
    });
    let attrs = input_attrs(&node).into_iter().collect::<BTreeMap<_, _>>();

    assert_eq!(attrs.get("flex-basis").map(String::as_str), Some("content"));
    assert_eq!(
        attrs.get("width").map(String::as_str),
        Some("min(calc(70% - 8px), 160px)")
    );
    assert_eq!(
        attrs.get("height").map(String::as_str),
        Some("calc(50% + 4px)")
    );
    assert_eq!(
        attrs.get("min-width").map(String::as_str),
        Some("max(calc(20% + 12px), 72px)")
    );
    assert_eq!(attrs.get("min-height").map(String::as_str), Some("18px"));
    assert_eq!(
        attrs.get("max-width").map(String::as_str),
        Some("clamp(90px, calc(50% + 4px), 166px)")
    );
    assert_eq!(
        attrs.get("max-height").map(String::as_str),
        Some("fit-content(max(24px, 30%))")
    );
    assert_eq!(
        attrs.get("grid-template-columns").map(String::as_str),
        Some("minmax(calc(25% + 10px),calc(35% + 16px)) fit-content(calc(25% + 15px))")
    );
    assert_eq!(
        attrs.get("grid-template-rows").map(String::as_str),
        Some("max(20px, 10%)")
    );
}

fn run_bundled_helper_script(name: &str, script: String) {
    let root = std::env::temp_dir().join(format!("surgeist-layout-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join(format!("{name}.js"));
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run bundled helper smoke test");

    assert!(
        output.status.success(),
        "node bundled helper smoke test failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

fn run_bundled_helper_json(name: &str, script: String) -> Value {
    let root = std::env::temp_dir().join(format!("surgeist-layout-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join(format!("{name}.js"));
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run bundled helper JSON test");
    let cleanup = fs::remove_dir_all(&root);

    assert!(
        output.status.success(),
        "node bundled helper JSON test failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    cleanup.expect("helper JSON test temp dir cleanup");
    serde_json::from_slice(&output.stdout).expect("helper test must emit one JSON value")
}

#[test]
fn fri07_c04_collapse_helper_lowers_only_computed_in_flow_flex_item_collapse() {
    let script = format!(
        r#"
const window = {{}};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

const parent = {{}};
const element = {{
  id: "original-name",
  parentElement: parent,
  getBoundingClientRect() {{ return {{ x: 1, y: 2, width: 3, height: 4 }}; }},
}};
let parentDisplay = "flex";
function getComputedStyle(target) {{
  if (target !== parent) throw new Error("collapse lowering inspected an unexpected element");
  return {{ display: parentDisplay }};
}}

const cases = [
  ["collapsed flex item", {{ visibility: "collapse", position: "static", display: "block" }}, "flex", "collapsed"],
  ["normal flex item", {{ visibility: "visible", position: "static", display: "block" }}, "flex", undefined],
  ["hidden flex item", {{ visibility: "hidden", position: "static", display: "block" }}, "flex", undefined],
  ["absolute flex child", {{ visibility: "collapse", position: "absolute", display: "block" }}, "flex", undefined],
  ["display-none flex child", {{ visibility: "collapse", position: "static", display: "none" }}, "flex", undefined],
  ["non-flex child", {{ visibility: "collapse", position: "static", display: "block" }}, "block", undefined],
];
for (const [label, style, display, expected] of cases) {{
  parentDisplay = display;
  const actual = normalizedFlexItemCollapse(element, style);
  if (actual !== expected) {{
    throw new Error(`${{label}}: expected ${{String(expected)}}, got ${{String(actual)}}`);
  }}
}}

parentDisplay = "flex";
const collapsedStyle = {{ visibility: "collapse", position: "relative", display: "block" }};
const before = normalizedFlexItemCollapse(element, collapsedStyle);
element.id = "renamed-fixture";
element.getBoundingClientRect = () => ({{ x: 90, y: 80, width: 70, height: 60 }});
const after = normalizedFlexItemCollapse(element, collapsedStyle);
if (before !== "collapsed" || after !== before) {{
  throw new Error(`fixture name or geometry changed normalized collapse: ${{before}} -> ${{after}}`);
}}
"#
    );

    run_bundled_helper_script("fri07-c04-collapse-helper", script);
}

#[test]
fn fri07_c04_collapse_serializer_emits_exact_attribute_only_for_collapsed() {
    let collapsed = input_attrs(&json!({
        "tagName": "div",
        "style": {"flexItemCollapse": "collapsed"}
    }));
    assert_eq!(
        collapsed
            .iter()
            .filter(|(name, _)| *name == "flex-item-collapse")
            .collect::<Vec<_>>(),
        vec![&("flex-item-collapse", "collapsed".to_string())]
    );

    let normal = input_attrs(&json!({"tagName": "div", "style": {}}));
    assert!(normal.iter().all(|(name, _)| *name != "flex-item-collapse"));
}

#[test]
fn fri07_c04_collapse_serializer_rejects_every_noncollapsed_explicit_state() {
    for value in [
        json!("normal"),
        json!("visible"),
        json!("hidden"),
        json!("inherit"),
        json!(""),
        json!(true),
    ] {
        let node = json!({
            "tagName": "div",
            "style": {"flexItemCollapse": value}
        });
        assert!(
            std::panic::catch_unwind(|| input_attrs(&node)).is_err(),
            "serializer accepted explicit collapse state {value}"
        );
    }
}

fn keep_imported_browser_parity_support_reachable(golden: &browser_parity_support::Golden) {
    let parse_file = |path: &Path| browser_parity_support::Golden::parse_file(path);
    let _ = parse_file;
    let _ = browser_parity_support::fixture_files;
    let _ = browser_parity_support::fixture_files_in;
    let _ = browser_parity_support::fixture_skip_policy_mentions_x_prefix;
    let _ = browser_parity_support::fixture_skip_policy_filters_unsupported_constructs;
    let _ = golden.root.style.display();
    let _ = golden.root.style.width();
}

fn br_helper_smoke_script(
    parent_display: &str,
    writing_mode: &str,
    layout_ready_vertical_br: bool,
    expected_reason: Option<&str>,
) -> String {
    let expected_reason =
        expected_reason.map_or_else(|| "undefined".to_string(), |reason| format!("{reason:?}"));
    let vertical_br_attr = if layout_ready_vertical_br {
        r#"name === "data-surgeist-layout-ready-vertical-br" ? "true" : null"#
    } else {
        "null"
    };
    format!(
        r#"
const window = {{ innerWidth: 800 }};
const Node = {{ ELEMENT_NODE: 1, TEXT_NODE: 3 }};
let activeProbe;
const document = {{
  styleSheets: [],
  createElement() {{
    return {{
      style: {{}},
      children: [],
      append(...children) {{ this.children.push(...children); }},
      getBoundingClientRect() {{
        const lineOver = this.style.verticalAlign === "top";
        const writingMode = activeProbe?.style.writingMode;
        if (writingMode === "horizontal-tb") {{
          const y = lineOver ? 0 : 8;
          return {{ x: 0, y, left: 0, right: 0, top: y, bottom: y, width: 0, height: 0 }};
        }}
        const advancesLeft = writingMode === "vertical-rl" || writingMode === "sideways-rl";
        const x = lineOver ? 0 : (advancesLeft ? -8 : 8);
        return {{ x, y: 0, left: x, right: x, top: 0, bottom: 0, width: 0, height: 0 }};
      }},
      offsetWidth: 0,
      clientWidth: 0,
      remove() {{}},
    }};
  }},
  body: {{ appendChild(probe) {{ activeProbe = probe; }} }},
}};

{TEST_HELPER_SOURCE}

const parent = {{
  tagName: "DIV",
  classList: {{ contains() {{ return false; }} }},
  getBoundingClientRect() {{ return {{ x: 0, y: 0, width: 100, height: 20, right: 100, left: 0, bottom: 20, top: 0 }}; }},
  clientLeft: 0,
  clientTop: 0,
  getAttribute(name) {{ return {vertical_br_attr}; }},
}};

const element = {{
  tagName: "BR",
  classList: {{ contains() {{ return false; }} }},
  style: {{
    gridTemplateRows: "",
    gridTemplateColumns: "",
    gridAutoRows: "",
    gridAutoColumns: "",
    gridRowStart: "auto",
    gridRowEnd: "auto",
    gridColumnStart: "auto",
    gridColumnEnd: "auto",
  }},
  parentNode: parent,
  parentElement: parent,
  childNodes: [],
  childElementCount: 0,
  textContent: "",
  getBoundingClientRect() {{ return {{ x: 0, y: 0, width: 0, height: 0, right: 0, left: 0, bottom: 0, top: 0 }}; }},
  scrollWidth: 0,
  scrollHeight: 0,
  clientWidth: 0,
  clientHeight: 0,
  offsetWidth: 0,
  offsetHeight: 0,
  offsetLeft: 0,
  offsetTop: 0,
  getAttribute() {{ return null; }},
}};

function getComputedStyle(target) {{
  return {{
    display: target === parent ? "{parent_display}" : "inline",
    boxSizing: "content-box",
    direction: "ltr",
    writingMode: target === element ? "{writing_mode}" : "horizontal-tb",
    font: "10px ahem",
    fontFamily: "ahem",
    fontSize: "10px",
    lineHeight: "10px",
    width: "0px",
    height: "0px",
    minWidth: "0px",
    minHeight: "0px",
    maxWidth: "none",
    maxHeight: "none",
    marginLeft: "0px",
    marginRight: "0px",
    marginTop: "0px",
    marginBottom: "0px",
  }};
}}

const data = describeElement(element);
const expectedReason = {expected_reason};
if (data.unsupportedReason !== expectedReason) {{
  throw new Error(`expected unsupportedReason ${{expectedReason}}, got ${{data.unsupportedReason}}`);
}}
if (data.tagName !== "br") {{
  throw new Error(`expected tagName br, got ${{data.tagName}}`);
}}
if (expectedReason === undefined) {{
  if (data.style.inlineBaseline !== "8px") {{
    throw new Error(`expected inlineBaseline 8px, got ${{data.style.inlineBaseline}}`);
  }}
  if (data.style.inlineLineHeight !== "10px") {{
    throw new Error(`expected inlineLineHeight 10px, got ${{data.style.inlineLineHeight}}`);
  }}
}}
"#
    )
}

#[test]
fn fixture_cases_match_browser_measurement_keys() {
    assert_eq!(
        fixture_cases(),
        [
            ("border_box_ltr", "borderBoxLtrData"),
            ("content_box_ltr", "contentBoxLtrData"),
            ("border_box_rtl", "borderBoxRtlData"),
            ("content_box_rtl", "contentBoxRtlData"),
        ]
    );
}

#[derive(Debug, PartialEq, Eq)]
enum Fri06C08DirectRootChild {
    Text(String),
    Element(String),
}

fn fri06_c08_direct_test_root(
    raw: &str,
    relative: &str,
) -> (Vec<Fri06C08DirectRootChild>, Option<Value>) {
    let root_marker = r#"<div id="test-root""#;
    assert_eq!(
        raw.matches(root_marker).count(),
        1,
        "{relative} must contain one exact #test-root element"
    );
    let root_start = raw.find(root_marker).expect("checked one root marker");
    let root_end = raw[root_start..]
        .rfind("</div>")
        .map(|offset| root_start + offset + "</div>".len())
        .unwrap_or_else(|| panic!("{relative} must close #test-root"));
    let document = roxmltree::Document::parse(&raw[root_start..root_end])
        .unwrap_or_else(|error| panic!("{relative} #test-root must parse: {error}"));
    let root = document.root_element();
    assert_eq!(root.attribute("id"), Some("test-root"), "{relative}");

    let children = root
        .children()
        .map(|child| match child.node_type() {
            roxmltree::NodeType::Text => Fri06C08DirectRootChild::Text(
                child
                    .text()
                    .expect("text node must contain text")
                    .to_string(),
            ),
            roxmltree::NodeType::Element => {
                Fri06C08DirectRootChild::Element(child.tag_name().name().to_string())
            }
            other => panic!("{relative} has unexpected direct #test-root child {other:?}"),
        })
        .collect();
    let authored_breaks = root.attribute("data-surgeist-inline-breaks").map(|raw| {
        serde_json::from_str(raw)
            .unwrap_or_else(|error| panic!("{relative} authored breaks must parse: {error}"))
    });
    (children, authored_breaks)
}

#[test]
fn fri06_c08_t2_word_only_segments_preserve_direct_root_sequence_and_break_indices() {
    use Fri06C08DirectRootChild::{Element, Text};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity/html");
    let cases = [
        (
            "block/fri06_bidi_mixed_inline.html",
            vec![
                Text("\n  ".to_string()),
                Element("bdo".to_string()),
                Text(" ".to_string()),
                Element("bdo".to_string()),
                Text("delta".to_string()),
            ],
            4,
            "delta",
            None,
        ),
        (
            "block/fri06_inline_mixed_text_atomic_wrap.html",
            vec![
                Text("text".to_string()),
                Element("span".to_string()),
                Element("span".to_string()),
                Element("span".to_string()),
            ],
            0,
            "text",
            Some(json!([
                {"sourceIndex": 0, "followingBreak": "allowed"},
                {"sourceIndex": 1, "followingBreak": "allowed"}
            ])),
        ),
        (
            "float/fri06_float_line_exclusion.html",
            vec![
                Text("\n  ".to_string()),
                Element("span".to_string()),
                Text("\n  ".to_string()),
                Element("span".to_string()),
                Text("line".to_string()),
                Element("span".to_string()),
                Element("span".to_string()),
                Element("span".to_string()),
                Element("span".to_string()),
                Text("\n".to_string()),
            ],
            4,
            "line",
            Some(json!([
                {"sourceIndex": 4, "followingBreak": "allowed"},
                {"sourceIndex": 5, "followingBreak": "allowed"},
                {"sourceIndex": 6, "followingBreak": "allowed"},
                {"sourceIndex": 7, "followingBreak": "allowed"}
            ])),
        ),
        (
            "float/fri06_float_shape_exclusion.html",
            vec![
                Text("\n  ".to_string()),
                Element("span".to_string()),
                Text("bands".to_string()),
                Element("span".to_string()),
                Element("span".to_string()),
                Element("span".to_string()),
                Element("span".to_string()),
                Text("\n".to_string()),
            ],
            2,
            "bands",
            Some(json!([
                {"sourceIndex": 4, "followingBreak": "allowed"}
            ])),
        ),
    ];

    for (relative, expected_children, source_index, word, expected_breaks) in cases {
        let raw = fs::read_to_string(root.join(relative)).expect(relative);
        let (children, authored_breaks) = fri06_c08_direct_test_root(&raw, relative);
        assert_eq!(
            children, expected_children,
            "{relative} direct #test-root sequence must preserve source indices and element adjacency"
        );
        assert_eq!(
            children.get(source_index),
            Some(&Text(word.to_string())),
            "{relative} source segment {source_index} must contain only {word:?} with no boundary whitespace"
        );
        assert_eq!(
            authored_breaks, expected_breaks,
            "{relative} authored break indices must remain exact"
        );
    }
}

#[test]
fn fri06_c08_new_helper_and_serializer_require_one_finite_shape_bands_field() {
    let script = [
            r#"
const window = {};
const document = { styleSheets: [] };
const CSSRule = { STYLE_RULE: 1 };
const element = {
  getAttribute(name) {
    if (name !== "data-surgeist-shape-bands") return null;
    return '[{"bandMinimum":0,"bandMaximum":20,"intervalMinimum":0,"intervalMaximum":44},{"bandMinimum":20,"bandMaximum":40,"intervalMinimum":0,"intervalMaximum":28},{"bandMinimum":40,"bandMaximum":60}]';
  },
};
"#,
            TEST_HELPER_SOURCE,
            r#"
const bands = layoutReadyShapeBands(element);
if (JSON.stringify(bands) !== JSON.stringify([
  {bandMinimum: 0, bandMaximum: 20, intervalMinimum: 0, intervalMaximum: 44},
  {bandMinimum: 20, bandMaximum: 40, intervalMinimum: 0, intervalMaximum: 28},
  {bandMinimum: 40, bandMaximum: 60},
])) {
  throw new Error(`shapeBands must preserve the finite physical table, got ${JSON.stringify(bands)}`);
}
"#,
        ]
        .concat();
    run_bundled_helper_script("fri06-c08-new-shape-bands", script);

    let node = json!({
        "tagName": "div",
        "useRounding": false,
        "viewport": {"width": {"unit": "px", "value": 100}, "height": {"unit": "max-content"}},
        "style": {"display": "block"},
        "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 60},
        "children": [{
            "tagName": "div",
            "style": {"cssFloat": "left"},
            "shapeBands": [
                {"bandMinimum": 0, "bandMaximum": 20, "intervalMinimum": 0, "intervalMaximum": 44},
                {"bandMinimum": 20, "bandMaximum": 40, "intervalMinimum": 0, "intervalMaximum": 28},
                {"bandMinimum": 40, "bandMaximum": 60},
            ],
            "unroundedLayout": {"x": 0, "y": 0, "width": 44, "height": 60},
            "children": [],
        }],
    });
    let xml = generate_xml("fri06_c08_new_shape_bands", &node);
    for expected in [
        r#"float="left" float-exclusion="shape""#,
        r#"<shape-bands>"#,
        r#"<shape-band band-minimum="0" band-maximum="20" interval-minimum="0" interval-maximum="44"/>"#,
        r#"<shape-band band-minimum="20" band-maximum="40" interval-minimum="0" interval-maximum="28"/>"#,
        r#"<shape-band band-minimum="40" band-maximum="60"/>"#,
    ] {
        assert!(xml.contains(expected), "missing {expected:?} in\n{xml}");
    }
}

#[test]
fn fri06_c08_new_authored_break_opportunity_serializes_and_wraps_through_public_layout() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const parentRect = { x: 0, y: 0, left: 0, top: 0, right: 72, bottom: 38, width: 72, height: 38 };
const textRect = { x: 0, y: 0, left: 0, top: 0, right: 40, bottom: 20, width: 40, height: 20 };
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return textRect; },
  getClientRects() { return [textRect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const text = { nodeType: Node.TEXT_NODE, textContent: "text " };
const atomics = [
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "inline-block" }, width: 18, x: 0, y: 20 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "inline-block" }, width: 24, x: 18, y: 20 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "inline-block" }, width: 30, x: 42, y: 20 },
];
for (const atomic of atomics) {
  atomic.getBoundingClientRect = () => ({
    x: atomic.x,
    y: atomic.y,
    left: atomic.x,
    top: atomic.y,
    right: atomic.x + atomic.width,
    bottom: atomic.y + 18,
    width: atomic.width,
    height: 18,
  });
}
const parent = {
  childNodes: [text, ...atomics],
  getBoundingClientRect() { return parentRect; },
  getAttribute(name) {
    if (name === "data-surgeist-layout-ready-inline") return "true";
    if (name === "data-surgeist-inline-breaks") {
      return '[{"sourceIndex":0,"followingBreak":"allowed"}]';
    }
    return null;
  },
};
function getComputedStyle(element) {
  if (element === parent) {
    return { direction: "ltr", writingMode: "horizontal-tb", fontSize: "16px", lineHeight: "20px", display: "block" };
  }
  return { direction: "ltr", writingMode: "horizontal-tb", display: "inline-block" };
}
"#,
            TEST_HELPER_SOURCE,
            r#"
describeElement = function(element) {
  return {
    tagName: "span",
    style: {
      display: "inline-block",
      size: {
        width: { unit: "px", value: element.width },
        height: { unit: "px", value: 18 },
      },
    },
    unroundedLayout: { x: element.x, y: element.y, width: element.width, height: 18 },
    children: [],
  };
};
const children = describeChildNodes(parent);
console.log(JSON.stringify({
  tagName: "div",
  layoutReadyInlineRoot: true,
  useRounding: false,
  viewport: { width: { unit: "px", value: 72 }, height: { unit: "max-content" } },
  style: { display: "block", size: { width: { unit: "px", value: 72 } } },
  unroundedLayout: { x: 0, y: 0, width: 72, height: 38 },
  children,
}));
"#,
        ]
        .concat();
    let node = run_bundled_helper_json("fri06-c08-new-authored-break-wrap", script);
    let xml = generate_xml("fri06_c08_new_authored_break_wrap", &node);
    let golden = browser_parity_support::Golden::parse(&xml).expect("serialized fixture");
    keep_imported_browser_parity_support_reachable(&golden);
    let layout = browser_parity_support::assert_surgeist_matches(&golden);

    assert!(
        layout.is_ok(),
        "layout-ready input must serialize the authored allowed boundary and wrap through compute_layout; result={layout:?}\n{xml}"
    );
    assert!(
            xml.contains(
                r#"<segment id="0" inline-extent="40" inline-baseline="14.8" inline-line-height="20" bidi-level="0" whitespace-edge="preserve" following-break="allowed"/>"#
            ),
            "text source/segment identity and exact allowed boundary must be serialized\n{xml}"
        );
}

#[test]
fn fri06_c08_new_float_line_breaks_advance_inside_reduced_band_through_public_layout() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const parentRect = { x: 0, y: 0, left: 0, top: 0, right: 180, bottom: 63, width: 180, height: 63 };
const textRect = { x: 42, y: 0, left: 42, top: 0, right: 80.53125, bottom: 20, width: 38.53125, height: 20 };
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return textRect; },
  getClientRects() { return [textRect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const ignored = { nodeType: 8 };
const floating = [
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "block" }, display: "block", cssFloat: "left", width: 42, height: 42, x: 0, y: 0 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "block" }, display: "block", cssFloat: "right", width: 50, height: 62, x: 130, y: 0 },
];
const text = { nodeType: Node.TEXT_NODE, textContent: "line " };
const atomics = [
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "inline-block" }, width: 28, x: 81, y: 0 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "inline-block" }, width: 32, x: 42, y: 21 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "inline-block" }, width: 36, x: 74, y: 21 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", style: { display: "inline-block" }, width: 40, x: 0, y: 42 },
];
for (const atomic of atomics) {
  atomic.getBoundingClientRect = () => ({
    x: atomic.x,
    y: atomic.y,
    left: atomic.x,
    top: atomic.y,
    right: atomic.x + atomic.width,
    bottom: atomic.y + 16,
    width: atomic.width,
    height: 16,
  });
}
const parent = {
  childNodes: [ignored, floating[0], ignored, floating[1], text, ...atomics],
  getBoundingClientRect() { return parentRect; },
  getAttribute(name) {
    if (name === "data-surgeist-layout-ready-inline") return "true";
    if (name === "data-surgeist-inline-breaks") {
      return '[{"sourceIndex":4,"followingBreak":"allowed"},{"sourceIndex":5,"followingBreak":"allowed"},{"sourceIndex":6,"followingBreak":"allowed"},{"sourceIndex":7,"followingBreak":"allowed"}]';
    }
    if (name === "data-surgeist-inline-struts") {
      return '[{"beforeSourceIndex":5,"baseline":12,"lineHeight":20}]';
    }
    return null;
  },
};
function getComputedStyle(element) {
  if (element === parent) {
    return { direction: "ltr", writingMode: "horizontal-tb", fontSize: "16px", lineHeight: "20px", display: "block" };
  }
  return { direction: "ltr", writingMode: "horizontal-tb", display: element.style.display };
}
"#,
            TEST_HELPER_SOURCE,
            r#"
describeElement = function(element) {
  return {
    tagName: "span",
    style: {
      display: element.style.display,
      cssFloat: element.cssFloat || "none",
      size: {
        width: { unit: "px", value: element.width },
        height: { unit: "px", value: element.height || 16 },
      },
    },
    unroundedLayout: {
      x: element.x,
      y: element.y,
      width: element.width,
      height: element.height || 16,
    },
    smartRoundedLayout: {
      x: element.x,
      y: element.y,
      width: element.width,
      height: element.height || 16,
    },
    children: [],
  };
};
const children = describeChildNodes(parent);
console.log(JSON.stringify({
  tagName: "div",
  layoutReadyInlineRoot: true,
  useRounding: true,
  viewport: { width: { unit: "px", value: 180 }, height: { unit: "max-content" } },
  style: { display: "block", size: { width: { unit: "px", value: 180 } } },
  unroundedLayout: { x: 0, y: 0, width: 180, height: 63 },
  smartRoundedLayout: { x: 0, y: 0, width: 180, height: 63 },
  children,
}));
"#,
        ]
        .concat();
    let node = run_bundled_helper_json("fri06-c08-new-float-line-breaks", script);
    let xml = generate_xml("fri06_c08_new_float_line_breaks", &node);
    let golden = browser_parity_support::Golden::parse(&xml).expect("serialized fixture");
    let layout = browser_parity_support::assert_surgeist_matches(&golden);

    assert!(
        layout.is_ok(),
        "allowed source boundaries must wrap and advance inside the 88px opposing-float band through compute_layout; result={layout:?}\n{xml}"
    );
    for expected in [
        r#"<segment id="4" inline-extent="38.53125" inline-baseline="14.8" inline-line-height="20" bidi-level="0" whitespace-edge="preserve" following-break="allowed"/>"#,
        r#"<atomic-placeholder child-index="4" bidi-level="0" following-break="allowed"/>"#,
        r#"<atomic-placeholder child-index="5" bidi-level="0" following-break="allowed"/>"#,
        r#"<atomic-placeholder child-index="6" bidi-level="0" following-break="allowed"/>"#,
        r#"<atomic-placeholder child-index="7" bidi-level="0" following-break="prohibited"/>"#,
    ] {
        assert!(xml.contains(expected), "missing {expected:?} in\n{xml}");
    }
}

#[test]
fn fri06_c08_existing_blockified_authored_inline_is_not_atomic_or_breakable() {
    let script = [
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const document = { styleSheets: [] };
const blockified = {
  nodeType: Node.ELEMENT_NODE,
  tagName: "SPAN",
  style: { display: "inline-block" },
};
const br = {
  nodeType: Node.ELEMENT_NODE,
  tagName: "BR",
  style: { display: "inline" },
};
let authoredBreaks = null;
const parent = {
  childNodes: [blockified, br],
  parentElement: null,
  getAttribute(name) {
    if (name === "data-surgeist-layout-ready-inline") return "true";
    if (name === "data-surgeist-inline-breaks") return authoredBreaks;
    return null;
  },
};
function getComputedStyle(element) {
  return {
    display: element === blockified ? "block" : "inline",
    direction: "ltr",
  };
}
"#,
        TEST_HELPER_SOURCE,
        r#"
describeElement = function(element) {
  return { tagName: element.tagName.toLowerCase(), children: [] };
};

const children = describeChildNodes(parent);
if (Object.prototype.hasOwnProperty.call(children[0], "atomicInlineParticipation")) {
  throw new Error("authored-inline/computed-block child received an atomic placeholder");
}

authoredBreaks = '[{"sourceIndex":0,"followingBreak":"allowed"}]';
let rejected = false;
try {
  describeChildNodes(parent);
} catch (error) {
  rejected = String(error).includes("must target shaped text or an atomic inline");
}
if (!rejected) {
  throw new Error("authored-inline/computed-block child accepted an authored atomic break fact");
}
"#,
    ]
    .concat();

    run_bundled_helper_script("fri06-c08-blockified-non-atomic", script);
}

#[test]
fn fri06_c08_existing_range_ink_omits_model_visual_identity() {
    let script = [
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const parentRect = { x: 0, y: 0, left: 0, top: 0, right: 100, bottom: 20, width: 100, height: 20 };
const textRect = { x: 10, y: 0, left: 10, top: 0, right: 35, bottom: 20, width: 25, height: 20 };
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return textRect; },
  getClientRects() { return [textRect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const parent = {
  parentElement: null,
  getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
  getBoundingClientRect() { return parentRect; },
};
const text = { nodeType: Node.TEXT_NODE, textContent: "X", parentElement: parent };
function getComputedStyle(element) {
  return {
    direction: "rtl",
    writingMode: "horizontal-tb",
    fontSize: "25px",
    lineHeight: "25px",
  };
}
"#,
        TEST_HELPER_SOURCE,
        r#"
const shaped = layoutReadyTextNodeData(text, parent, 7);
const rangeInk = shaped.rangeInks[0];
const keys = Object.keys(rangeInk).sort();
const expectedKeys = ["advance", "lineIndex", "physicalStartEdge", "sourceSegmentId", "start"];
if (JSON.stringify(keys) !== JSON.stringify(expectedKeys)) {
  throw new Error(`Range ink retained a model-only field: ${JSON.stringify(rangeInk)}`);
}
if (rangeInk.sourceSegmentId !== 7 || rangeInk.lineIndex !== 0 ||
    rangeInk.physicalStartEdge !== "right" || rangeInk.start !== 35 || rangeInk.advance !== 25) {
  throw new Error(`Range source/line/flow-inline facts changed: ${JSON.stringify(rangeInk)}`);
}
"#,
    ]
    .concat();

    run_bundled_helper_script("fri06-c08-range-ink-no-model-visual", script);
}

#[test]
fn fri06_c08_existing_shaped_text_source_retains_typed_word_segments() {
    let relative = "subgrid/subgrid_auto_track_sizing_min_content_text_runs.html";
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity/html");
    let raw = fs::read_to_string(root.join(relative)).expect(relative);
    assert!(
        !raw.contains("<span"),
        "{relative} must retain shaped text identity rather than replacement boxes"
    );

    let inline_marker =
        r#"<div data-surgeist-anonymous-grid-text-wrapper="true" data-surgeist-inline-breaks="#;
    let inline_start = raw.find(inline_marker).expect("typed text parent marker");
    let inline_end = raw[inline_start..]
        .find("</div>")
        .map(|offset| inline_start + offset + "</div>".len())
        .expect("typed text parent close");
    let document = roxmltree::Document::parse(&raw[inline_start..inline_end])
        .expect("typed text parent must parse");
    let inline_parent = document
        .descendants()
        .find(|node| {
            node.is_element()
                && node
                    .attribute("style")
                    .is_some_and(|style| style.contains("grid-template-rows: subgrid"))
        })
        .expect("typed text parent");
    let children = inline_parent.children().collect::<Vec<_>>();
    let expected_segments = [(0, "X"), (4, "XXXX"), (8, "XX"), (12, "XXX")];
    for (source_index, expected_text) in expected_segments {
        assert_eq!(
            children[source_index].text(),
            Some(expected_text),
            "source segment {source_index}"
        );
    }
    let breaks: Value = serde_json::from_str(
        inline_parent
            .attribute("data-surgeist-inline-breaks")
            .expect("typed text breaks"),
    )
    .expect("typed text breaks must parse");
    assert_eq!(
        breaks,
        json!([
            {"sourceIndex": 0, "followingBreak": "allowed"},
            {"sourceIndex": 4, "followingBreak": "allowed"},
            {"sourceIndex": 8, "followingBreak": "allowed"}
        ])
    );
}

#[test]
fn fri06_c08_existing_grid_indentation_is_ignored_without_suppressing_unmarked_inline_space() {
    let script = [
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const document = { styleSheets: [] };
"#,
        TEST_HELPER_SOURCE,
        r#"
const inline = {
  nodeType: Node.ELEMENT_NODE,
  style: { display: "inline-grid" },
};
const whitespace = { nodeType: Node.TEXT_NODE, textContent: "\n  " };
const childNodes = [inline, whitespace, inline];
function parent(display, marked) {
  return {
    childNodes,
    getAttribute(name) {
      return marked && name === "data-surgeist-layout-ready-inline" ? "true" : null;
    },
  };
}
function getComputedStyle(element) {
  return { display: element.style?.display || element.display || "block" };
}

const grid = parent("grid", false);
grid.display = "grid";
if (unsupportedChildNodesReason(grid) !== undefined) {
  throw new Error("grid-parent indentation must not be classified as mixed inline content");
}
const block = parent("block", false);
block.display = "block";
if (unsupportedChildNodesReason(block) !== "Unsupported mixed text/element content") {
  throw new Error("unmarked significant inline whitespace must stay unsupported");
}
const marked = parent("block", true);
marked.display = "block";
if (unsupportedChildNodesReason(marked) !== undefined) {
  throw new Error("explicit layout-ready inline fixtures must leave unsupported accounting");
}
"#,
    ]
    .concat();

    run_bundled_helper_script("fri06-c08-grid-indentation", script);
}

#[test]
fn fri06_c08_existing_helper_emits_complete_finite_text_and_control_facts() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const rootRect = { x: 10, y: 20, left: 10, top: 20, right: 210, bottom: 60, width: 200, height: 40 };
const textRect = { x: 30, y: 24, left: 30, top: 24, right: 34, bottom: 34, width: 4, height: 10 };
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return textRect; },
  getClientRects() { return [textRect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const parent = {
  parentElement: null,
  getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
  getBoundingClientRect() { return rootRect; },
};
const text = { nodeType: Node.TEXT_NODE, textContent: " ", parentElement: parent };
function getComputedStyle() {
  return {
    direction: "ltr",
    writingMode: "horizontal-tb",
    fontSize: "10px",
    lineHeight: "10px",
  };
}
"#,
            TEST_HELPER_SOURCE,
            r#"
const shaped = layoutReadyTextNodeData(text, parent, 7);
const segment = shaped.inlineSegments[0];
const rangeInk = shaped.rangeInks[0];
for (const [name, value] of Object.entries({
  inlineExtent: segment.inlineExtent,
  inlineBaseline: segment.inlineBaseline,
  inlineLineHeight: segment.inlineLineHeight,
  start: rangeInk.start,
  advance: rangeInk.advance,
})) {
  if (!Number.isFinite(value)) throw new Error(`${name} must be finite, got ${value}`);
}
if (segment.id !== 7 || rangeInk.sourceSegmentId !== 7) {
  throw new Error("text source and Range-ink segment identity must remain stable");
}
if (rangeInk.lineIndex !== 0 || Object.prototype.hasOwnProperty.call(rangeInk, "visualIndex")) {
  throw new Error("Range ink must retain line identity without model visual identity");
}

const metrics = brInlineMetricsForElement({ tagName: "BR" }, {
  fontSize: "10px",
  lineHeight: "0px",
});
if (metrics.baseline !== "0px" || metrics.lineHeight !== "0px") {
  throw new Error(`zero-height control metrics must remain valid, got ${JSON.stringify(metrics)}`);
}
"#,
        ]
        .concat();

    run_bundled_helper_script("fri06-c08-finite-inline-facts", script);
}

#[test]
fn fri06_c08_t1_range_start_uses_nearest_explicit_inline_root() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const rootRect = { x: 10, y: 20, left: 10, top: 20, right: 210, bottom: 120, width: 200, height: 100 };
const parentRect = { x: 25, y: 35, left: 25, top: 35, right: 125, bottom: 85, width: 100, height: 50 };
const textRect = { x: 40, y: 41, left: 40, top: 41, right: 44, bottom: 51, width: 4, height: 10 };
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return textRect; },
  getClientRects() { return [textRect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const root = {
  parentElement: null,
  getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
  getBoundingClientRect() { return rootRect; },
};
const parent = {
  parentElement: root,
  getAttribute() { return null; },
  getBoundingClientRect() { return parentRect; },
};
const text = { nodeType: Node.TEXT_NODE, textContent: "x", parentElement: parent };
let flow = { direction: "ltr", writingMode: "horizontal-tb" };
function getComputedStyle() {
  return {
    direction: flow.direction,
    writingMode: flow.writingMode,
    fontSize: "10px",
    lineHeight: "10px",
  };
}
"#,
            TEST_HELPER_SOURCE,
            r#"
for (const [direction, writingMode, physicalStartEdge, start, advance] of [
  ["ltr", "horizontal-tb", "left", 30, 4],
  ["rtl", "horizontal-tb", "right", 34, 4],
  ["ltr", "vertical-rl", "top", 21, 10],
  ["rtl", "vertical-rl", "bottom", 31, 10],
]) {
  flow = { direction, writingMode };
  const rangeInk = layoutReadyTextNodeData(text, parent, 7).rangeInks[0];
  if (rangeInk.physicalStartEdge !== physicalStartEdge ||
      rangeInk.start !== start || rangeInk.advance !== advance) {
    throw new Error(`Range start must be local to the explicit inline root, got ${JSON.stringify(rangeInk)}`);
  }
}
"#,
        ]
        .concat();

    run_bundled_helper_script("fri06-c08-t1-root-local-range", script);
}

#[test]
fn fri06_c08_t1_helper_emits_control_fact_only_for_lowered_inline_br() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const document = { styleSheets: [] };
const root = {
  parentElement: null,
  getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
};
const parent = { parentElement: root, getAttribute() { return null; } };
"#,
            TEST_HELPER_SOURCE,
            r#"
const inlineBr = { tagName: "BR", parentElement: parent };
const blockifiedBr = { tagName: "BR", parentElement: parent };
const unactivatedBr = { tagName: "BR", parentElement: { parentElement: null, getAttribute() { return null; } } };
const activatedSpan = { tagName: "SPAN", parentElement: parent };

const valid = layoutReadyLineControlParticipation(inlineBr, { display: "inline" });
if (JSON.stringify(valid) !== JSON.stringify({ kind: "forced-break" })) {
  throw new Error(`computed inline BR must emit an explicit control fact, got ${JSON.stringify(valid)}`);
}
for (const [name, element, style] of [
  ["blockified BR", blockifiedBr, { display: "block" }],
  ["source tag alone", unactivatedBr, { display: "inline" }],
  ["activation ancestor alone", activatedSpan, { display: "inline" }],
]) {
  const actual = layoutReadyLineControlParticipation(element, style);
  if (actual !== undefined) {
    throw new Error(`${name} must not emit model control participation, got ${JSON.stringify(actual)}`);
  }
}
"#,
        ]
        .concat();

    run_bundled_helper_script("fri06-c08-t1-explicit-control-role", script);
}

#[test]
fn fri06_c08_t1_serializer_gates_control_on_explicit_fact() {
    let root = |child: Value| {
        json!({
            "tagName": "div",
            "useRounding": false,
            "viewport": {
                "width": {"unit": "px", "value": 100},
                "height": {"unit": "max-content"},
            },
            "style": {"display": "block", "direction": "ltr", "writingMode": "horizontal-tb"},
            "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
            "children": [child],
        })
    };
    let br = |participation: Option<Value>, display: &str| {
        let mut node = json!({
            "tagName": "br",
            "style": {
                "display": display,
                "inlineBaseline": "8px",
                "inlineLineHeight": "10px",
            },
            "unroundedLayout": {"x": 10, "y": 0, "width": 0, "height": 10},
            "children": [],
        });
        if let Some(participation) = participation {
            node["lineControlParticipation"] = participation;
        }
        node
    };

    let ordinary = generate_xml(
        "fri06_c08_t1_source_tag_negative",
        &root(br(None, "inline")),
    );
    assert!(
        ordinary.contains(r#"source-tag="br""#),
        "legacy BR input changed\n{ordinary}"
    );
    assert!(
        !ordinary.contains("line-control="),
        "source tag alone promoted a control\n{ordinary}"
    );
    assert!(
        !ordinary.contains("<browser-control"),
        "source tag alone promoted a browser control\n{ordinary}"
    );

    let explicit = generate_xml(
        "fri06_c08_t1_explicit_control",
        &root(br(Some(json!({"kind": "forced-break"})), "inline")),
    );
    assert!(
        explicit.contains(r#"line-control="forced-break""#),
        "missing explicit line control\n{explicit}"
    );
    assert!(
        explicit.contains("<browser-control"),
        "missing explicit browser control observation\n{explicit}"
    );

    let blockified = generate_xml("fri06_c08_t1_blockified_br", &root(br(None, "block")));
    assert!(
        blockified.contains(r#"<div source-tag="br" display="block" width="0px" height="10px"/>"#),
        "blockified BR lost its ordinary box\n{blockified}"
    );
    assert!(
        !blockified.contains("line-control="),
        "blockified BR gained line-break lowering data\n{blockified}"
    );
    assert!(
        !blockified.contains("inline-baseline="),
        "blockified BR retained control metrics\n{blockified}"
    );
    assert!(
        !blockified.contains("<browser-control"),
        "blockified BR retained a control observation\n{blockified}"
    );
}

#[test]
fn fri06_c08_t1_serializer_rejects_malformed_and_non_br_control_facts() {
    let root = |child: Value| {
        json!({
            "tagName": "div",
            "useRounding": false,
            "viewport": {
                "width": {"unit": "px", "value": 100},
                "height": {"unit": "max-content"},
            },
            "style": {"display": "block", "direction": "ltr", "writingMode": "horizontal-tb"},
            "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
            "children": [child],
        })
    };
    let br = |participation: Value, display: &str| {
        json!({
            "tagName": "br",
            "lineControlParticipation": participation,
            "style": {
                "display": display,
                "inlineBaseline": "8px",
                "inlineLineHeight": "10px",
            },
            "unroundedLayout": {"x": 10, "y": 0, "width": 0, "height": 10},
            "children": [],
        })
    };

    for (label, child) in [
        ("non-object", br(json!(true), "inline")),
        ("wrong kind", br(json!({"kind": "boundary"}), "inline")),
        (
            "extra field",
            br(json!({"kind": "forced-break", "sourceTag": "br"}), "inline"),
        ),
        ("blockified", br(json!({"kind": "forced-break"}), "block")),
        (
            "non-BR",
            json!({
                "tagName": "span",
                "lineControlParticipation": {"kind": "forced-break"},
                "style": {"display": "inline"},
                "unroundedLayout": {"x": 10, "y": 0, "width": 10, "height": 10},
                "children": [],
            }),
        ),
        (
            "inline text",
            json!({
                "layoutInput": "inline-text",
                "lineControlParticipation": {"kind": "forced-break"},
                "inlineSegments": [{
                    "id": 0,
                    "inlineExtent": 10,
                    "inlineBaseline": 8,
                    "inlineLineHeight": 10,
                    "bidiLevel": 0,
                    "whitespaceEdge": "preserve",
                    "followingBreak": "prohibited",
                }],
                "children": [],
            }),
        ),
    ] {
        let result = std::panic::catch_unwind(|| {
            generate_xml("fri06_c08_t1_malformed_control", &root(child))
        });
        assert!(
            result.is_err(),
            "serializer accepted malformed control state {label}"
        );
    }
}

#[test]
fn fri06_c08_recovery_inputs_block_br_used_size_round_trips_as_an_ordinary_box() {
    let cases = [
        ("horizontal", "horizontal-tb", 0.0, 10.0),
        ("vertical", "vertical-rl", 10.0, 0.0),
        ("unequal-flex", "horizontal-tb", 0.0, 19.0),
    ];

    for (label, writing_mode, width, height) in cases {
        let root_display = if label == "unequal-flex" {
            "flex"
        } else {
            "block"
        };
        let node = json!({
            "tagName": "div",
            "useRounding": false,
            "viewport": {
                "width": {"unit": "px", "value": 100},
                "height": {"unit": "px", "value": 100},
            },
            "style": {
                "display": root_display,
                "direction": "ltr",
                "writingMode": writing_mode,
                "size": {
                    "width": {"unit": "px", "value": 100},
                    "height": {"unit": "px", "value": 100},
                },
            },
            "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 100},
            "children": [{
                "tagName": "br",
                "style": {
                    "display": "block",
                    "direction": "ltr",
                    "writingMode": writing_mode,
                    "inlineBaseline": "8px",
                    "inlineLineHeight": "10px",
                },
                "unroundedLayout": {
                    "x": 0,
                    "y": 0,
                    "width": width,
                    "height": height,
                },
                "children": [],
            }],
        });

        let xml = generate_xml(&format!("fri06_c08_recovery_inputs_br_{label}"), &node);
        let golden = browser_parity_support::Golden::parse(&xml).unwrap_or_else(|error| {
            panic!("{label} serialized fixture must parse: {error}\n{xml}")
        });
        keep_imported_browser_parity_support_reachable(&golden);
        let child = &golden.root.children[0];
        assert_eq!(child.style.get("source-tag"), Some("br"), "{label}");
        assert_eq!(
            child.style.get("display"),
            Some("block"),
            "{label} computed role"
        );
        assert_eq!(
            child.style.get("width"),
            Some(format!("{width}px").as_str()),
            "{label} used width"
        );
        assert_eq!(
            child.style.get("height"),
            Some(format!("{height}px").as_str()),
            "{label} used height"
        );

        let input = xml
            .split_once("<input>")
            .and_then(|(_, rest)| rest.split_once("</input>"))
            .map(|(input, _)| input)
            .expect("serialized input section");
        for prohibited in [
            "line-control=",
            "inline-baseline=",
            "inline-line-height=",
            "<browser-control",
            "<atomic-placeholder",
        ] {
            assert!(
                !input.contains(prohibited),
                "{label} block BR emitted prohibited {prohibited:?}\n{xml}"
            );
        }
    }

    let node = json!({
        "tagName": "div",
        "useRounding": false,
        "viewport": {
            "width": {"unit": "px", "value": 20},
            "height": {"unit": "px", "value": 10},
        },
        "style": {
            "display": "block",
            "size": {
                "width": {"unit": "px", "value": 20},
                "height": {"unit": "px", "value": 10},
            },
        },
        "unroundedLayout": {"x": 0, "y": 0, "width": 20, "height": 10},
        "children": [{
            "tagName": "br",
            "style": {"display": "block"},
            "unroundedLayout": {"x": 0, "y": 0, "width": 0, "height": 10},
            "children": [],
        }],
    });
    let xml = generate_xml("fri06_c08_recovery_inputs_zero_width_br", &node);
    let golden = browser_parity_support::Golden::parse(&xml).expect("serialized block BR");
    browser_parity_support::assert_surgeist_matches(&golden).unwrap_or_else(|error| {
        panic!("used 0x10 block BR must survive the real parser/layout path: {error}\n{xml}")
    });
}

#[test]
fn fri06_c08_recovery_inputs_block_br_validation_and_non_br_controls_stay_narrow() {
    let root = |child: Value| {
        json!({
            "tagName": "div",
            "useRounding": false,
            "viewport": {
                "width": {"unit": "px", "value": 100},
                "height": {"unit": "max-content"},
            },
            "style": {"display": "block"},
            "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
            "children": [child],
        })
    };
    let block_br = |layout: Value| {
        json!({
            "tagName": "br",
            "style": {
                "display": "block",
                "inlineBaseline": "8px",
                "inlineLineHeight": "10px",
            },
            "unroundedLayout": layout,
            "children": [],
        })
    };

    for (label, layout) in [
        ("missing width", json!({"x": 0, "y": 0, "height": 10})),
        (
            "negative height",
            json!({"x": 0, "y": 0, "width": 0, "height": -1}),
        ),
    ] {
        let result = std::panic::catch_unwind(|| {
            generate_xml(
                "fri06_c08_recovery_inputs_invalid_block_br",
                &root(block_br(layout)),
            )
        });
        assert!(result.is_err(), "block BR accepted {label}");
    }

    let ordinary = json!({
        "tagName": "span",
        "style": {"display": "block"},
        "unroundedLayout": {"x": 0, "y": 0, "width": 0, "height": 19},
        "children": [],
    });
    let xml = generate_xml(
        "fri06_c08_recovery_inputs_ordinary_block_control",
        &root(ordinary),
    );
    let golden = browser_parity_support::Golden::parse(&xml).expect("ordinary block fixture");
    assert_eq!(golden.root.children[0].style.get("width"), None);
    assert_eq!(golden.root.children[0].style.get("height"), None);

    let inline_br = json!({
        "tagName": "br",
        "lineControlParticipation": {"kind": "forced-break"},
        "style": {
            "display": "inline",
            "inlineBaseline": "8px",
            "inlineLineHeight": "10px",
        },
        "unroundedLayout": {"x": 0, "y": 0, "width": 0, "height": 10},
        "children": [],
    });
    let xml = generate_xml(
        "fri06_c08_recovery_inputs_inline_br_control",
        &root(inline_br),
    );
    assert!(xml.contains(r#"source-tag="br" line-control="forced-break" display="inline""#));
    assert!(xml.contains(r#"inline-baseline="8px" inline-line-height="10px""#));
}

#[test]
fn fri06_c08_recovery_inputs_range_lines_round_trip_in_root_local_source_order() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
let selected;
const range = {
  selectNodeContents(node) { selected = node; },
  getBoundingClientRect() { return selected.rect; },
  getClientRects() { return [selected.rect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const ignored = { nodeType: 8 };
const texts = [
  { nodeType: Node.TEXT_NODE, textContent: "X", rect: { x: 0, y: 0, left: 0, top: 0, right: 10, bottom: 10, width: 10, height: 10 } },
  { nodeType: Node.TEXT_NODE, textContent: "XXXX", rect: { x: 0, y: 20, left: 0, top: 20, right: 40, bottom: 30, width: 40, height: 10 } },
  { nodeType: Node.TEXT_NODE, textContent: "XX", rect: { x: 0, y: 40, left: 0, top: 40, right: 20, bottom: 50, width: 20, height: 10 } },
  { nodeType: Node.TEXT_NODE, textContent: "XXX", rect: { x: 0, y: 60, left: 0, top: 60, right: 30, bottom: 70, width: 30, height: 10 } },
];
const rootRect = { x: 0, y: 0, left: 0, top: 0, right: 100, bottom: 100, width: 100, height: 100 };
const root = {
  parentElement: null,
  childNodes: [texts[0], ignored, ignored, ignored, texts[1], ignored, ignored, ignored, texts[2], ignored, ignored, ignored, texts[3]],
  getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
  getBoundingClientRect() { return rootRect; },
};
for (const text of texts) text.parentElement = root;
function getComputedStyle() {
  return { direction: "ltr", writingMode: "horizontal-tb", fontSize: "10px", lineHeight: "10px", display: "block" };
}
"#,
            TEST_HELPER_SOURCE,
            r#"
const children = describeChildNodes(root);
console.log(JSON.stringify({
  tagName: "div",
  layoutReadyInlineRoot: true,
  useRounding: false,
  viewport: { width: { unit: "px", value: 100 }, height: { unit: "px", value: 100 } },
  style: {
    display: "block",
    size: { width: { unit: "px", value: 100 }, height: { unit: "px", value: 100 } },
  },
  unroundedLayout: { x: 0, y: 0, width: 100, height: 100 },
  children,
}));
"#,
        ]
        .concat();

    let node = run_bundled_helper_json("fri06-c08-recovery-root-lines", script);
    let xml = generate_xml("fri06_c08_recovery_inputs_root_lines", &node);
    let golden = browser_parity_support::Golden::parse(&xml).expect("serialized Range lines");
    keep_imported_browser_parity_support_reachable(&golden);
    assert_eq!(
        golden.root.style.get("layout-ready-inline-root"),
        Some("true"),
        "the exact explicit-root marker must survive serialization and parsing\n{xml}"
    );
    let lines = golden
        .expectations
        .children
        .iter()
        .map(|child| {
            child
                .range_inks
                .as_ref()
                .and_then(|ranges| ranges.first())
                .expect("one Range observation")
                .line_index
        })
        .collect::<Vec<_>>();
    assert_eq!(
        lines,
        [0, 1, 2, 3],
        "four source runs must retain distinct browser lines\n{xml}"
    );
    for (source_id, child) in [0, 4, 8, 12].into_iter().zip(&golden.expectations.children) {
        assert_eq!(
            child.range_inks.as_ref().unwrap()[0].source_segment_id,
            source_id
        );
    }
}

#[test]
fn fri06_c08_recovery_inputs_range_registry_reuses_resets_and_rejects_invalid_identity() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const document = { styleSheets: [] };
function root(writingMode = "horizontal-tb") {
  const value = {
    parentElement: null,
    writingMode,
    getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
    getBoundingClientRect() { return { left: 0, top: 0, right: 100 }; },
  };
  return value;
}
function fragment(left, top, right) { return { left, top, right, width: right - left, height: 10 }; }
function getComputedStyle(element) { return { writingMode: element.writingMode }; }
"#,
            TEST_HELPER_SOURCE,
            r#"
const same = root();
resetLayoutReadyRangeLineRegistry(same);
if (layoutReadyRangeLineIndex(same, fragment(0, 10, 10)) !== 0 ||
    layoutReadyRangeLineIndex(same, fragment(10, 10.05, 20)) !== 0) {
  throw new Error("same-line anchors within 0.1px must reuse one line index");
}

const outer = root();
const nested = root();
resetLayoutReadyRangeLineRegistry(outer);
resetLayoutReadyRangeLineRegistry(nested);
if (layoutReadyRangeLineIndex(outer, fragment(0, 10, 10)) !== 0 ||
    layoutReadyRangeLineIndex(outer, fragment(0, 20, 10)) !== 1 ||
    layoutReadyRangeLineIndex(nested, fragment(0, 20, 10)) !== 0) {
  throw new Error("nested explicit roots must reset Range line identity");
}

for (const [writingMode, fragmentRect] of [
  ["horizontal-tb", fragment(0, 10, 10)],
  ["vertical-rl", fragment(0, 0, 90)],
  ["sideways-rl", fragment(0, 0, 90)],
  ["vertical-lr", fragment(10, 0, 20)],
  ["sideways-lr", fragment(10, 0, 20)],
]) {
  const flowRoot = root(writingMode);
  resetLayoutReadyRangeLineRegistry(flowRoot);
  if (layoutReadyRangeLineIndex(flowRoot, fragmentRect) !== 0) {
    throw new Error(`${writingMode} failed to allocate its first physical block anchor`);
  }
}

function mustReject(label, action, expected) {
  let error;
  try { action(); } catch (caught) { error = String(caught); }
  if (!error || !error.includes(expected)) {
    throw new Error(`${label} did not reject with ${expected}: ${error}`);
  }
}
mustReject("unknown writing mode", () => {
  const bad = root("diagonal");
  resetLayoutReadyRangeLineRegistry(bad);
  layoutReadyRangeLineIndex(bad, fragment(0, 0, 10));
}, "unknown writing mode");
mustReject("nonfinite coordinate", () => {
  const bad = root();
  resetLayoutReadyRangeLineRegistry(bad);
  layoutReadyRangeLineIndex(bad, fragment(0, Number.NaN, 10));
}, "finite block-progress coordinate");
mustReject("ambiguous identity", () => {
  const bad = root();
  resetLayoutReadyRangeLineRegistry(bad);
  layoutReadyRangeLineIndex(bad, fragment(0, 0, 10));
  layoutReadyRangeLineIndex(bad, fragment(0, 0.15, 10));
  layoutReadyRangeLineIndex(bad, fragment(0, 0.075, 10));
}, "ambiguous Range line identity");

let selectedFragments = [];
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return { x: 0, y: 0, left: 0, top: 0, right: 10, bottom: 10, width: 10, height: 10 }; },
  getClientRects() { return selectedFragments; },
  detach() {},
};
document.createRange = () => range;
const marked = root();
const parent = { parentElement: marked };
const text = { nodeType: Node.TEXT_NODE, textContent: "x", parentElement: parent };
marked.getBoundingClientRect = () => ({ x: 0, y: 0, left: 0, top: 0, right: 100, bottom: 100, width: 100, height: 100 });
parent.getBoundingClientRect = marked.getBoundingClientRect;
getComputedStyle = () => ({ direction: "ltr", writingMode: "horizontal-tb", fontSize: "10px", lineHeight: "10px" });
mustReject("zero fragments", () => layoutReadyTextNodeData(text, parent, 0), "exactly one fragment");
selectedFragments = [
  { x: 0, y: 0, left: 0, top: 0, right: 5, bottom: 10, width: 5, height: 10 },
  { x: 5, y: 0, left: 5, top: 0, right: 10, bottom: 10, width: 5, height: 10 },
];
mustReject("multiple fragments", () => layoutReadyTextNodeData(text, parent, 0), "exactly one fragment");
selectedFragments = [{ x: 0, y: 0, left: 0, top: 0, right: 10, bottom: 10, width: 10, height: 10 }];
const unmarked = { parentElement: null, getBoundingClientRect: marked.getBoundingClientRect };
mustReject(
  "missing explicit root",
  () => layoutReadyTextNodeData({ ...text, parentElement: unmarked }, unmarked, 0),
  "explicit layout-ready inline root",
);
"#,
        ]
        .concat();

    run_bundled_helper_script("fri06-c08-recovery-range-controls", script);
}

#[test]
fn fri06_c08_recovery_inputs_shape_break_round_trips_before_42px_atomic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let relative = "html/float/fri06_float_shape_exclusion.html";
    let raw = fs::read_to_string(root.join(relative)).expect(relative);
    let (_, authored_breaks) = fri06_c08_direct_test_root(&raw, relative);
    let authored_breaks = serde_json::to_string(&authored_breaks.unwrap_or(Value::Null))
        .expect("authored shape breaks JSON");

    let mut script = String::from(
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
let selected;
const range = {
  selectNodeContents(node) { selected = node; },
  getBoundingClientRect() { return selected.rect; },
  getClientRects() { return [selected.rect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const whitespace = { nodeType: Node.TEXT_NODE, textContent: "\n" };
const floating = { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", display: "block", cssFloat: "left", width: 44, height: 60, x: 0, y: 0 };
const text = { nodeType: Node.TEXT_NODE, textContent: "bands", rect: { x: 44, y: 1.2, left: 44, top: 1.2, right: 92.1640625, bottom: 21.2, width: 48.1640625, height: 20 } };
const atomics = [
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", display: "inline-block", width: 34, height: 16, x: 92.1640625, y: 0 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", display: "inline-block", width: 38, height: 16, x: 126.1640625, y: 0 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", display: "inline-block", width: 42, height: 16, x: 44, y: 21.2 },
  { nodeType: Node.ELEMENT_NODE, tagName: "SPAN", display: "inline-block", width: 46, height: 16, x: 86, y: 21.2 },
];
"#,
    );
    script.push_str(&format!("const authoredBreaks = {authored_breaks};\n"));
    script.push_str(
            r#"
const parentRect = { x: 0, y: 0, left: 0, top: 0, right: 180, bottom: 60, width: 180, height: 60 };
const parent = {
  parentElement: null,
  childNodes: [whitespace, floating, text, ...atomics, whitespace],
  getAttribute(name) {
    if (name === "data-surgeist-layout-ready-inline") return "true";
    if (name === "data-surgeist-inline-breaks") return authoredBreaks === null ? null : JSON.stringify(authoredBreaks);
    return null;
  },
  getBoundingClientRect() { return parentRect; },
};
for (const child of parent.childNodes) child.parentElement = parent;
function getComputedStyle(element) {
  if (element === parent) {
    return { direction: "ltr", writingMode: "horizontal-tb", fontSize: "16px", lineHeight: "20px", display: "block" };
  }
  return { direction: "ltr", writingMode: "horizontal-tb", display: element.display };
}
"#,
        );
    script.push_str(TEST_HELPER_SOURCE);
    script.push_str(
        r#"
describeElement = function(element) {
  const described = {
    tagName: "span",
    style: {
      display: element.display,
      cssFloat: element.cssFloat || "none",
      size: {
        width: { unit: "px", value: element.width },
        height: { unit: "px", value: element.height },
      },
    },
    unroundedLayout: { x: element.x, y: element.y, width: element.width, height: element.height },
    children: [],
  };
  if (element === floating) {
    described.shapeBands = [
      { bandMinimum: 0, bandMaximum: 21.2, intervalMinimum: 0, intervalMaximum: 44 },
      { bandMinimum: 21.2, bandMaximum: 37.2, intervalMinimum: 0, intervalMaximum: 44 },
    ];
  }
  return described;
};
const children = describeChildNodes(parent);
console.log(JSON.stringify({
  tagName: "div",
  layoutReadyInlineRoot: true,
  useRounding: false,
  viewport: { width: { unit: "px", value: 180 }, height: { unit: "max-content" } },
  style: {
    display: "block",
    direction: "ltr",
    writingMode: "horizontal-tb",
    fontFamily: "monospace",
    fontSize: { unit: "px", value: 16 },
    lineHeight: { unit: "px", value: 20 },
    size: { width: { unit: "px", value: 180 }, height: { unit: "auto" } },
  },
  unroundedLayout: { x: 0, y: 0, width: 180, height: 60 },
  children,
}));
"#,
    );

    let node = run_bundled_helper_json("fri06-c08-recovery-shape-break", script);
    let xml = generate_xml("fri06_float_shape_exclusion__border_box_ltr", &node);
    let golden = browser_parity_support::Golden::parse(&xml).expect("serialized shape fixture");
    let layout = browser_parity_support::assert_surgeist_matches(&golden);
    assert!(
        layout.is_ok(),
        "the 38px atomic must carry the allowed break before 42px through helper, serializer, parser, and public layout; result={layout:?}\n{xml}"
    );
    for expected in [
        r#"<atomic-placeholder child-index="2" bidi-level="0" following-break="prohibited"/>"#,
        r#"<atomic-placeholder child-index="3" bidi-level="0" following-break="allowed"/>"#,
        r#"<atomic-placeholder child-index="4" bidi-level="0" following-break="prohibited"/>"#,
        r#"<atomic-placeholder child-index="5" bidi-level="0" following-break="prohibited"/>"#,
    ] {
        assert!(xml.contains(expected), "missing {expected:?}\n{xml}");
    }
    assert_eq!(
        serde_json::from_str::<Value>(&authored_breaks).expect("authored break value"),
        json!([{"sourceIndex": 4, "followingBreak": "allowed"}])
    );
}

#[test]
fn fri06_c08_recovery_inputs_shape_source_rejects_wrong_duplicate_range_br_and_float_targets() {
    fn validate(raw: &str) -> Result<(), String> {
        let root_start = raw
            .find(r#"<div id="test-root""#)
            .ok_or_else(|| "missing test root".to_string())?;
        let root_end = raw[root_start..]
            .rfind("</div>")
            .map(|offset| root_start + offset + "</div>".len())
            .ok_or_else(|| "unclosed test root".to_string())?;
        let document = roxmltree::Document::parse(&raw[root_start..root_end])
            .map_err(|error| error.to_string())?;
        let root = document.root_element();
        let breaks = root
            .attribute("data-surgeist-inline-breaks")
            .ok_or_else(|| "missing shape break table".to_string())?;
        let breaks: Value = serde_json::from_str(breaks).map_err(|error| error.to_string())?;
        if breaks != json!([{"sourceIndex": 4, "followingBreak": "allowed"}]) {
            return Err("shape break must target only sourceIndex 4".to_string());
        }
        let target = root
            .children()
            .nth(4)
            .ok_or_else(|| "shape break target is out of range".to_string())?;
        if !target.has_tag_name("span") {
            return Err("shape break target must be the 38px atomic".to_string());
        }
        let style = target.attribute("style").unwrap_or_default();
        if !style.contains("display: inline-block")
            || !style.contains("width: 38px")
            || style.contains("float:")
        {
            return Err("shape break target must be the non-floating 38px atomic".to_string());
        }
        Ok(())
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity/html/float/fri06_float_shape_exclusion.html");
    let raw = fs::read_to_string(&path).expect("shape source");
    validate(&raw).expect("reviewed shape source contract");

    let exact = r#"data-surgeist-inline-breaks='[{"sourceIndex":4,"followingBreak":"allowed"}]'"#;
    for (label, mutated) in [
            ("wrong index 5", raw.replace(exact, &exact.replace(":4", ":5"))),
            (
                "duplicate",
                raw.replace(
                    exact,
                    r#"data-surgeist-inline-breaks='[{"sourceIndex":4,"followingBreak":"allowed"},{"sourceIndex":4,"followingBreak":"allowed"}]'"#,
                ),
            ),
            ("out of range", raw.replace(exact, &exact.replace(":4", ":99"))),
            (
                "BR target",
                raw.replacen(
                    r#"<span style="display: inline-block; width: 38px; height: 16px;"></span>"#,
                    r#"<br style="display: inline; width: 38px; height: 16px;"/>"#,
                    1,
                ),
            ),
            (
                "float target",
                raw.replacen(
                    r#"display: inline-block; width: 38px; height: 16px;"#,
                    r#"display: inline-block; float: left; width: 38px; height: 16px;"#,
                    1,
                ),
            ),
        ] {
            assert!(
                validate(&mutated).is_err(),
                "shape source contract accepted {label}"
            );
        }
}

#[test]
fn fri06_c08_t1_typed_inline_children_replace_legacy_raw_fallback() {
    let typed_parent = json!({
        "tagName": "div",
        "useRounding": false,
        "viewport": {"width": {"unit": "px", "value": 100}, "height": {"unit": "max-content"}},
        "style": {"display": "block"},
        "textContent": "duplicate raw fallback",
        "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
        "children": [{
            "layoutInput": "inline-text",
            "inlineSegments": [{
                "id": 0,
                "inlineExtent": 10,
                "inlineBaseline": 8,
                "inlineLineHeight": 10,
                "bidiLevel": 0,
                "whitespaceEdge": "preserve",
                "followingBreak": "prohibited",
            }],
            "children": [],
        }],
    });
    let typed_xml = generate_xml("fri06_c08_t1_typed_replacement", &typed_parent);
    assert!(typed_xml.contains(r#"<text layout-input="inline-text">"#));
    assert!(
        !typed_xml.contains("duplicate raw fallback"),
        "typed text retained duplicate raw fallback\n{typed_xml}"
    );
}

#[test]
fn fri06_c08_t1_spill_matrix_preserves_64_legacy_variants() {
    let families = [
        "block_basic_with_br",
        "block_border_fixed_size_with_br",
        "block_br_empty_lines_metrics",
        "block_br_inline_block_metrics",
        "block_br_vertical_lr_inline_block_metrics",
        "block_br_vertical_rl_empty_lines_metrics",
        "block_br_vertical_rl_inline_block_metrics",
        "block_br_vertical_rl_rtl_inline_block_metrics",
        "block_direction_rtl_with_br",
        "block_margin_x_fixed_auto_left_and_right_with_br",
        "block_margin_x_fixed_auto_left_with_br",
        "block_margin_y_collapse_through_blocked_by_padding_bottom_with_br",
        "block_margin_y_collapse_through_positive_with_br",
        "block_margin_y_simple_positive_with_br",
        "block_padding_border_fixed_size_with_br",
        "block_padding_fixed_size_with_br",
    ];
    let variants = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];
    let mut cases = 0;
    for family in families {
        for variant in variants {
            let node = json!({
                "tagName": "div",
                "useRounding": false,
                "viewport": {"width": {"unit": "px", "value": 100}, "height": {"unit": "max-content"}},
                "style": {"display": "block", "direction": "ltr", "writingMode": "horizontal-tb"},
                "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
                "children": [{
                    "tagName": "br",
                    "style": {
                        "display": "inline",
                        "inlineBaseline": "0px",
                        "inlineLineHeight": "0px",
                    },
                    "unroundedLayout": {"x": 0, "y": 10, "width": 0, "height": 0},
                    "children": [],
                }],
            });
            let xml = generate_xml(&format!("{family}__{variant}"), &node);
            assert!(
                xml.contains(r#"source-tag="br""#),
                "legacy BR input changed for {family}__{variant}\n{xml}"
            );
            assert!(
                !xml.contains("line-control="),
                "legacy source gained model control fact for {family}__{variant}\n{xml}"
            );
            assert!(
                !xml.contains("<browser-control"),
                "legacy source gained browser control observation for {family}__{variant}\n{xml}"
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 64);
}

#[test]
fn fri06_c08_t2_mixed_wrap_allows_break_after_first_atomic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let relative = "html/block/fri06_inline_mixed_text_atomic_wrap.html";
    let raw = fs::read_to_string(root.join(relative)).expect(relative);
    let (_, authored_breaks) = fri06_c08_direct_test_root(&raw, relative);
    assert_eq!(
        authored_breaks,
        Some(json!([
            {"sourceIndex": 0, "followingBreak": "allowed"},
            {"sourceIndex": 1, "followingBreak": "allowed"}
        ])),
        "the first 18px atomic must own the later allowed break"
    );
}

#[test]
fn fri06_c08_t2_bfc_avoidance_removes_label_caused_scroll_source_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let relative = "html/float/fri06_float_bfc_avoidance.html";
    let raw = fs::read_to_string(root.join(relative)).expect(relative);

    for oracle in [
        "display: block; overflow: auto; width: 180px",
        "float: left; width: 54px; height: 64px",
        "display: block; overflow: auto; width: auto; height: 24px",
        "display: flex; width: auto; height: 24px",
        "display: block; width: auto; height: 24px",
    ] {
        assert!(
            raw.contains(oracle),
            "missing BFC geometry oracle {oracle:?}"
        );
    }
    let retained_labels = ["overflow BFC", "flex BFC", "ordinary block"]
        .into_iter()
        .filter(|label| raw.contains(label))
        .collect::<Vec<_>>();
    assert!(
        retained_labels.is_empty(),
        "label-caused nonzero scroll source remains: {retained_labels:?}"
    );
    assert_eq!(raw.matches("<div style=").count(), 4);
    assert!(
        !raw.contains("<span"),
        "the BFC oracle must remain box-only"
    );
}

#[test]
fn fri06_c08_t2_shape_query_recorder_uses_two_observed_finite_bands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let relative = "html/float/fri06_float_shape_exclusion.html";
    let raw = fs::read_to_string(root.join(relative)).expect(relative);
    let root_start = raw.find(r#"<div id="test-root""#).expect("one test root");
    let root_end = raw[root_start..]
        .rfind("</div>")
        .map(|offset| root_start + offset + "</div>".len())
        .expect("closed test root");
    let document = roxmltree::Document::parse(&raw[root_start..root_end])
        .expect("shape fixture root must parse");
    let encoded = document
        .descendants()
        .find_map(|node| node.attribute("data-surgeist-shape-bands"))
        .expect("one finite shape query recorder");
    let recorded: Value = serde_json::from_str(encoded).expect("finite shape query table");

    assert_eq!(
        recorded,
        json!([
            {
                "bandMinimum": 0,
                "bandMaximum": 21.2,
                "intervalMinimum": 0,
                "intervalMaximum": 44
            },
            {
                "bandMinimum": 21.2,
                "bandMaximum": 37.2,
                "intervalMinimum": 0,
                "intervalMaximum": 44
            }
        ]),
        "query recorder must match the two exact observed browser bands"
    );
    assert!(!raw.contains("shape-outside"));
    assert!(!raw.contains("path="));
    assert!(!raw.contains("geometry="));
}

#[test]
fn fri06_c08_range_ink_helper_keeps_browser_block_ink_out_of_metric_geometry() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const parentRect = { x: 10, y: 20, left: 10, top: 20, right: 110, bottom: 80, width: 100, height: 60 };
const rangeRect = { x: 25, y: 33, left: 25, top: 33, right: 34, bottom: 40, width: 9, height: 7 };
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return rangeRect; },
  getClientRects() { return [rangeRect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const parent = {
  parentElement: null,
  getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
  getBoundingClientRect() { return parentRect; },
};
const text = { nodeType: Node.TEXT_NODE, textContent: "ink", parentElement: parent };
let flow = { direction: "ltr", writingMode: "horizontal-tb" };
function getComputedStyle() {
  return {
    direction: flow.direction,
    writingMode: flow.writingMode,
    fontSize: "16px",
    lineHeight: "20px",
  };
}
"#,
            TEST_HELPER_SOURCE,
            r#"
for (const [direction, writingMode, physicalStartEdge, start, advance] of [
  ["ltr", "horizontal-tb", "left", 15, 9],
  ["rtl", "horizontal-tb", "right", 24, 9],
  ["ltr", "vertical-rl", "top", 13, 7],
  ["rtl", "vertical-rl", "bottom", 20, 7],
]) {
  flow = { direction, writingMode };
  resetLayoutReadyRangeLineRegistry(parent);
  const shaped = layoutReadyTextNodeData(text, parent, 7);
  if (Object.prototype.hasOwnProperty.call(shaped, "fragments")) {
    throw new Error("Range y/height/baseline must not be emitted as model fragment geometry");
  }
  if (Object.prototype.hasOwnProperty.call(shaped, "unroundedLayout") ||
      Object.prototype.hasOwnProperty.call(shaped, "smartRoundedLayout")) {
    throw new Error("Range geometry must not be emitted as text-node metric union geometry");
  }
  if (JSON.stringify(shaped.rangeInks) !== JSON.stringify([{
    sourceSegmentId: 7,
    lineIndex: 0,
    physicalStartEdge,
    start,
    advance,
  }])) {
    throw new Error(`Range ink must retain only source/line and flow-inline facts, got ${JSON.stringify(shaped.rangeInks)}`);
  }
  if (shaped.inlineSegments[0].inlineBaseline !== 14.8 ||
      shaped.inlineSegments[0].inlineLineHeight !== 20) {
    throw new Error("supplied model line metrics must remain independent from 7px Range ink height");
  }
}
"#,
        ]
        .concat();

    run_bundled_helper_script("fri06-c08-range-ink-category", script);
}

#[test]
fn fri06_c08_range_ink_serializer_emits_only_physical_inline_observations() {
    let node = json!({
        "tagName": "div",
        "useRounding": false,
        "viewport": {
            "width": {"unit": "px", "value": 100},
            "height": {"unit": "max-content"},
        },
        "style": {"display": "block"},
        "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
        "children": [{
            "layoutInput": "inline-text",
            "inlineSegments": [{
                "id": 7,
                "inlineExtent": 9,
                "inlineBaseline": 14.8,
                "inlineLineHeight": 20,
                "bidiLevel": 0,
                "whitespaceEdge": "preserve",
                "followingBreak": "prohibited",
            }],
            "rangeInks": [{
                "sourceSegmentId": 7,
                "lineIndex": 0,
                "physicalStartEdge": "left",
                "start": 15,
                "advance": 9,
            }],
            "children": [],
        }],
    });

    let xml = generate_xml("fri06_c08_range_ink_serializer", &node);
    assert!(
            xml.contains(
                r#"<range-ink source_segment_id="7" line_index="0" physical_start_edge="left" start="15" advance="9"/>"#
            ),
            "missing explicit Range-ink category in\n{xml}"
        );
    assert!(
        !xml.contains("<fragment "),
        "Range ink must not serialize as a model fragment\n{xml}"
    );
    assert!(
        xml.contains("<node><range-inks>") || xml.contains("<node>\n        <range-inks>"),
        "Range-backed text-node expectation must not serialize metric union attributes\n{xml}"
    );
}

#[test]
fn fri06_c08_browser_control_serializer_emits_only_source_slot_and_neighbor_lines() {
    let node = json!({
        "tagName": "div",
        "useRounding": true,
        "viewport": {
            "width": {"unit": "px", "value": 140},
            "height": {"unit": "max-content"},
        },
        "style": {
            "display": "block",
            "direction": "ltr",
            "writingMode": "horizontal-tb",
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 140, "height": 72},
        "unroundedLayout": {"x": 0, "y": 0, "width": 140, "height": 72},
        "children": [
            {
                "tagName": "span",
                "style": {"display": "inline-block"},
                "smartRoundedLayout": {"x": 0, "y": 5, "width": 32, "height": 12},
                "unroundedLayout": {"x": 0, "y": 5, "width": 32, "height": 12},
                "children": [],
            },
            {
                "tagName": "br",
                "lineControlParticipation": {"kind": "forced-break"},
                "style": {"display": "inline"},
                "smartRoundedLayout": {"x": 32, "y": 2, "width": 0, "height": 19},
                "unroundedLayout": {"x": 32, "y": 2, "width": 0, "height": 19},
                "children": [],
            },
            {
                "tagName": "br",
                "lineControlParticipation": {"kind": "forced-break"},
                "style": {"display": "inline"},
                "smartRoundedLayout": {"x": 0, "y": 26, "width": 0, "height": 19},
                "unroundedLayout": {"x": 0, "y": 26, "width": 0, "height": 19},
                "children": [],
            },
            {
                "tagName": "span",
                "style": {"display": "inline-block"},
                "smartRoundedLayout": {"x": 0, "y": 53, "width": 48, "height": 12},
                "unroundedLayout": {"x": 0, "y": 53, "width": 48, "height": 12},
                "children": [],
            },
        ],
    });

    let xml = generate_xml("fri06_c08_browser_control_serializer", &node);
    for expected in [
        r#"<browser-control source_index="1" terminal_visual_slot="1" previous_line="same" next_line="later"/>"#,
        r#"<browser-control source_index="2" terminal_visual_slot="0" previous_line="earlier" next_line="later"/>"#,
    ] {
        assert!(xml.contains(expected), "missing {expected:?} in\n{xml}");
    }
    assert!(
        !xml.contains(r#"<node x="32" y="2" width="0" height="19""#)
            && !xml.contains(r#"<node x="0" y="26" width="0" height="19""#),
        "browser BR ink rectangles must not serialize as model control geometry\n{xml}"
    );

    let unobserved = json!({
        "tagName": "div",
        "useRounding": false,
        "viewport": {
            "width": {"unit": "px", "value": 100},
            "height": {"unit": "max-content"},
        },
        "style": {
            "display": "block",
            "direction": "ltr",
            "writingMode": "horizontal-tb",
        },
        "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
        "children": [
            {
                "layoutInput": "inline-text",
                "inlineSegments": [{
                    "id": 0,
                    "inlineExtent": 10,
                    "inlineBaseline": 8,
                    "inlineLineHeight": 10,
                    "bidiLevel": 0,
                    "whitespaceEdge": "preserve",
                    "followingBreak": "prohibited",
                }],
                "rangeInks": [{
                    "sourceSegmentId": 0,
                    "lineIndex": 0,
                    "physicalStartEdge": "left",
                    "start": 0,
                    "advance": 10,
                }],
                "children": [],
            },
            {
                "tagName": "br",
                "lineControlParticipation": {"kind": "forced-break"},
                "style": {"display": "inline"},
                "unroundedLayout": {"x": 10, "y": 0, "width": 0, "height": 10},
                "children": [],
            },
        ],
    });
    let xml = generate_xml("fri06_c08_unobserved_control_neighbor", &unobserved);
    assert!(
            xml.contains(
                r#"<browser-control source_index="1" terminal_visual_slot="unobserved" previous_line="unobserved" next_line="absent"/>"#
            ),
            "missing finite unobserved control category in\n{xml}"
        );
}

#[test]
fn fri06_c08_range_ink_serializer_rejects_every_ordinary_metric_and_scroll_state() {
    let incompatible_states = [
        (
            "unroundedLayout",
            json!({
                "unroundedLayout": {
                    "x": 0,
                    "y": 0,
                    "width": 9,
                    "height": 20,
                    "scrollWidth": 9,
                    "scrollHeight": 20,
                },
            }),
        ),
        (
            "smartRoundedLayout",
            json!({
                "smartRoundedLayout": {
                    "x": 0,
                    "y": 0,
                    "width": 9,
                    "height": 20,
                    "scrollWidth": 9,
                    "scrollHeight": 20,
                },
            }),
        ),
        (
            "naivelyRoundedLayout",
            json!({
                "naivelyRoundedLayout": {
                    "x": 0,
                    "y": 0,
                    "width": 9,
                    "height": 20,
                    "scrollWidth": 9,
                    "scrollHeight": 20,
                    "clientWidth": 9,
                    "clientHeight": 20,
                },
            }),
        ),
        (
            "style.overflowX",
            json!({"style": {"overflowX": "visible"}}),
        ),
        ("style.overflowY", json!({"style": {"overflowY": "clip"}})),
        (
            "style.overflowClipMargin",
            json!({"style": {"overflowClipMargin": "0px"}}),
        ),
        (
            "style.scrollbarWidth",
            json!({"style": {"scrollbarWidth": 0}}),
        ),
        (
            "style.scrollbarGutter",
            json!({"style": {"scrollbarGutter": "auto"}}),
        ),
        (
            "style.scrollPaddingTop",
            json!({"style": {"scrollPaddingTop": "auto"}}),
        ),
        (
            "style.scrollPaddingRight",
            json!({"style": {"scrollPaddingRight": "auto"}}),
        ),
        (
            "style.scrollPaddingBottom",
            json!({"style": {"scrollPaddingBottom": "auto"}}),
        ),
        (
            "style.scrollPaddingLeft",
            json!({"style": {"scrollPaddingLeft": "auto"}}),
        ),
        (
            "style.scrollMarginTop",
            json!({"style": {"scrollMarginTop": "0px"}}),
        ),
        (
            "style.scrollMarginRight",
            json!({"style": {"scrollMarginRight": "0px"}}),
        ),
        (
            "style.scrollMarginBottom",
            json!({"style": {"scrollMarginBottom": "0px"}}),
        ),
        (
            "style.scrollMarginLeft",
            json!({"style": {"scrollMarginLeft": "0px"}}),
        ),
        (
            "style.scrollSnapType",
            json!({"style": {"scrollSnapType": "none"}}),
        ),
        (
            "style.scrollSnapAlign",
            json!({"style": {"scrollSnapAlign": "none"}}),
        ),
        (
            "style.scrollSnapStop",
            json!({"style": {"scrollSnapStop": "normal"}}),
        ),
    ];

    for (label, incompatible_state) in incompatible_states {
        let mut node = json!({
            "tagName": "div",
            "useRounding": false,
            "viewport": {
                "width": {"unit": "px", "value": 100},
                "height": {"unit": "max-content"},
            },
            "style": {"display": "block"},
            "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
            "children": [{
                "layoutInput": "inline-text",
                "inlineSegments": [{
                    "id": 7,
                    "inlineExtent": 9,
                    "inlineBaseline": 14.8,
                    "inlineLineHeight": 20,
                    "bidiLevel": 0,
                    "whitespaceEdge": "preserve",
                    "followingBreak": "prohibited",
                }],
                "rangeInks": [{
                    "sourceSegmentId": 7,
                    "lineIndex": 0,
                    "physicalStartEdge": "left",
                    "start": 15,
                    "advance": 9,
                }],
                "children": [],
            }],
        });
        node["children"][0]
            .as_object_mut()
            .expect("Range-backed text node should be an object")
            .extend(
                incompatible_state
                    .as_object()
                    .expect("incompatible state should be an object")
                    .clone(),
            );

        let result = std::panic::catch_unwind(|| {
            generate_xml("fri06_c08_range_ink_serializer_rejection", &node)
        });
        assert!(
            result.is_err(),
            "Range ink serializer accepted incompatible state {label}"
        );
    }
}

#[test]
fn fri06_c08_existing_helper_emits_explicit_root_local_range_inline_coordinates_only() {
    let script = [
            r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const rootRect = { x: 10, y: 20, left: 10, top: 20, right: 210, bottom: 120, width: 200, height: 100 };
const parentRect = { x: 25, y: 35, left: 25, top: 35, right: 125, bottom: 85, width: 100, height: 50 };
const textRect = { x: 40, y: 41, left: 40, top: 41, right: 44, bottom: 51, width: 4, height: 10 };
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return textRect; },
  getClientRects() { return [textRect]; },
  detach() {},
};
const root = {
  parentElement: null,
  getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
  getBoundingClientRect() { return rootRect; },
};
const document = {
  styleSheets: [],
  createRange() { return range; },
  getElementById(id) { return id === "test-root" ? root : null; },
};
const parent = { parentElement: root, getBoundingClientRect() { return parentRect; } };
const text = { nodeType: Node.TEXT_NODE, textContent: "x", parentElement: parent };
function getComputedStyle() {
  return {
    direction: "ltr",
    writingMode: "horizontal-tb",
    fontSize: "10px",
    lineHeight: "10px",
  };
}
"#,
            TEST_HELPER_SOURCE,
            r#"
const shaped = layoutReadyTextNodeData(text, parent, 7);
const rangeInk = shaped.rangeInks[0];
if (rangeInk.physicalStartEdge !== "left" || rangeInk.start !== 30 || rangeInk.advance !== 4) {
  throw new Error(`Range ink must retain explicit-root-local flow-inline geometry, got ${JSON.stringify(rangeInk)}`);
}
if (rangeInk.sourceSegmentId !== 7 || rangeInk.lineIndex !== 0 ||
    Object.prototype.hasOwnProperty.call(rangeInk, "visualIndex")) {
  throw new Error(`Range-ink identity must remain stable, got ${JSON.stringify(rangeInk)}`);
}
if ("y" in rangeInk || "height" in rangeInk || "baselineX" in rangeInk || "baselineY" in rangeInk) {
  throw new Error(`Range ink must not retain block-axis or baseline geometry, got ${JSON.stringify(rangeInk)}`);
}
"#,
        ]
        .concat();

    run_bundled_helper_script("fri06-c08-parent-local-range-ink", script);
}

#[test]
fn fri06_c08_existing_serializer_emits_c06_shaped_atomic_control_and_fragment_schema() {
    let node = json!({
        "tagName": "div",
        "useRounding": false,
        "viewport": {
            "width": {"unit": "px", "value": 100},
            "height": {"unit": "max-content"},
        },
        "style": {"display": "block"},
        "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
        "children": [
            {
                "layoutInput": "inline-text",
                "inlineSegments": [{
                    "id": 7,
                    "inlineExtent": 4.5,
                    "inlineBaseline": 8,
                    "inlineLineHeight": 10,
                    "bidiLevel": 0,
                    "whitespaceEdge": "discard-at-both",
                    "followingBreak": "allowed",
                }],
                "unroundedLayout": {"x": 20, "y": 4, "width": 4.5, "height": 10},
                "fragments": [{
                    "sourceSegmentId": 7,
                    "lineIndex": 0,
                    "visualIndex": 1,
                    "x": 20,
                    "y": 4,
                    "width": 4.5,
                    "height": 10,
                    "baselineX": 20,
                    "baselineY": 12,
                }],
                "children": [],
            },
            {
                "tagName": "span",
                "style": {"display": "inline-block"},
                "atomicInlineParticipation": {
                    "bidiLevel": 1,
                    "followingBreak": "prohibited",
                },
                "unroundedLayout": {"x": 24.5, "y": 0, "width": 10, "height": 10},
                "children": [],
            },
            {
                "tagName": "br",
                "lineControlParticipation": {"kind": "forced-break"},
                "style": {
                    "display": "inline",
                    "inlineBaseline": "0px",
                    "inlineLineHeight": "0px",
                },
                "unroundedLayout": {"x": 34.5, "y": 0, "width": 0, "height": 0},
                "children": [],
            },
        ],
    });

    let xml = generate_xml("fri06_c08_existing_schema", &node);
    for expected in [
        r#"<text layout-input="inline-text">"#,
        r#"<segment id="7" inline-extent="4.5" inline-baseline="8" inline-line-height="10" bidi-level="0" whitespace-edge="discard-at-both" following-break="allowed"/>"#,
        r#"<atomic-placeholder child-index="1" bidi-level="1" following-break="prohibited"/>"#,
        r#"<div source-tag="br" line-control="forced-break" display="inline" inline-baseline="0px" inline-line-height="0px"/>"#,
        r#"<fragment source_segment_id="7" line_index="0" visual_index="1" x="20" y="4" width="4.5" height="10" baseline_x="20" baseline_y="12"/>"#,
    ] {
        assert!(xml.contains(expected), "missing {expected:?} in\n{xml}");
    }
}

#[test]
fn xml_generation_preserves_browser_parity_fixture_shape() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {"display": "block", "direction": "ltr", "size": {"width": {"unit": "px", "value": 50}}},
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
        "unroundedLayout": {"x": 0, "y": 0, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
        "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 20},
        "children": [
            {
                "useRounding": true,
                "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                "style": {"direction": "ltr", "size": {"height": {"unit": "px", "value": 10}}},
                "smartRoundedLayout": {"x": 0, "y": 0, "width": 50, "height": 10, "scrollWidth": 50, "scrollHeight": 10},
                "unroundedLayout": {"x": 0, "y": 0, "width": 50, "height": 10, "scrollWidth": 50, "scrollHeight": 10},
                "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 10},
                "children": []
            }
        ]
    });

    let xml = generate_xml("block_basic__border_box_ltr", &node);

    assert!(xml.contains("<test name=\"block_basic__border_box_ltr\" use-rounding=\"true\">"));
    assert!(xml.contains("  <viewport width=\"max-content\" height=\"max-content\"/>"));
    assert!(xml.contains("    <div display=\"block\" direction=\"ltr\" width=\"50px\">"));
    assert!(xml.contains("      <div direction=\"ltr\" height=\"10px\"/>"));
    assert!(xml.contains("    <node x=\"0\" y=\"0\" width=\"50\" height=\"20\">"));
    assert!(xml.contains("      <node x=\"0\" y=\"0\" width=\"50\" height=\"10\"/>"));
}

#[test]
fn xml_generation_normalizes_root_expectation_to_origin() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {"display": "inline-grid"},
        "smartRoundedLayout": {"x": 7, "y": 4, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
        "unroundedLayout": {"x": 7, "y": 4, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
        "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 20},
        "children": [
            {
                "useRounding": true,
                "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                "style": {"display": "block"},
                "smartRoundedLayout": {"x": 1, "y": 2, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
                "unroundedLayout": {"x": 1, "y": 2, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
                "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 10},
                "children": []
            }
        ]
    });

    let xml = generate_xml("inline_grid__border_box_ltr", &node);

    assert!(xml.contains("    <node x=\"0\" y=\"0\" width=\"50\" height=\"20\">"));
    assert!(xml.contains("      <node x=\"1\" y=\"2\" width=\"10\" height=\"10\"/>"));
}

#[test]
fn xml_generation_marks_viewport_flex_item_root_context() {
    let node = json!({
        "useRounding": true,
        "viewport": {
            "width": {"unit": "px", "value": 400},
            "height": {"unit": "max-content"},
            "rootContext": "flex-item",
            "parentWritingMode": "horizontal-tb",
            "parentDirection": "ltr",
            "hostInlineSize": 160
        },
        "style": {"display": "grid"},
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 160, "height": 20, "scrollWidth": 160, "scrollHeight": 20},
        "unroundedLayout": {"x": 0, "y": 0, "width": 160, "height": 20, "scrollWidth": 160, "scrollHeight": 20},
        "naivelyRoundedLayout": {"clientWidth": 160, "clientHeight": 20},
        "children": []
    });

    let xml = generate_xml("grid_flex_item__border_box_ltr", &node);

    assert!(xml.contains(concat!(
        "  <viewport width=\"400px\" height=\"max-content\" ",
        "root-context=\"flex-item\" parent-writing-mode=\"horizontal-tb\" ",
        "parent-direction=\"ltr\" host-inline-size=\"160px\"/>"
    )));
}

#[test]
fn xml_generation_serializes_exact_order_and_parent_axes() {
    let flex_item = json!({
        "useRounding": true,
        "viewport": {
            "width": {"unit": "px", "value": 400},
            "height": {"unit": "max-content"},
            "rootContext": "flex-item",
            "parentWritingMode": "vertical-rl",
            "parentDirection": "rtl",
            "hostInlineSize": 37.5
        },
        "style": {"display": "grid", "order": "0", "writingMode": "horizontal-tb"},
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 160, "height": 20, "scrollWidth": 160, "scrollHeight": 20},
        "unroundedLayout": {"x": 0, "y": 0, "width": 160, "height": 20, "scrollWidth": 160, "scrollHeight": 20},
        "naivelyRoundedLayout": {"clientWidth": 160, "clientHeight": 20},
        "children": [
            {
                "style": {"order": "-2147483648"},
                "smartRoundedLayout": {"x": 0, "y": 0, "width": 80, "height": 20, "scrollWidth": 80, "scrollHeight": 20},
                "unroundedLayout": {"x": 0, "y": 0, "width": 80, "height": 20, "scrollWidth": 80, "scrollHeight": 20},
                "naivelyRoundedLayout": {"clientWidth": 80, "clientHeight": 20},
                "children": []
            },
            {
                "style": {"order": "2147483647"},
                "smartRoundedLayout": {"x": 80, "y": 0, "width": 80, "height": 20, "scrollWidth": 80, "scrollHeight": 20},
                "unroundedLayout": {"x": 80, "y": 0, "width": 80, "height": 20, "scrollWidth": 80, "scrollHeight": 20},
                "naivelyRoundedLayout": {"clientWidth": 80, "clientHeight": 20},
                "children": []
            }
        ]
    });
    let root = json!({
        "useRounding": true,
        "viewport": {
            "width": {"unit": "max-content"},
            "height": {"unit": "max-content"},
            "rootContext": "root",
            "parentWritingMode": "sideways-lr",
            "parentDirection": "rtl"
        },
        "style": {"display": "block", "order": "0"},
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 0, "height": 0, "scrollWidth": 0, "scrollHeight": 0},
        "unroundedLayout": {"x": 0, "y": 0, "width": 0, "height": 0, "scrollWidth": 0, "scrollHeight": 0},
        "naivelyRoundedLayout": {"clientWidth": 0, "clientHeight": 0},
        "children": []
    });

    let flex_xml = generate_xml("exact_flex_item_metadata", &flex_item);
    assert!(flex_xml.contains(concat!(
        "<viewport width=\"400px\" height=\"max-content\" ",
        "root-context=\"flex-item\" parent-writing-mode=\"vertical-rl\" ",
        "parent-direction=\"rtl\" host-inline-size=\"37.5px\"/>"
    )));
    assert!(!flex_xml.contains("order=\"0\""));
    assert!(flex_xml.contains("order=\"-2147483648\""));
    assert!(flex_xml.contains("order=\"2147483647\""));

    let mut zero_host = flex_item.clone();
    zero_host["viewport"]["hostInlineSize"] = json!(0);
    let zero_host_xml = generate_xml("zero_flex_host_allocation", &zero_host);
    assert!(zero_host_xml.contains("host-inline-size=\"0px\""));

    let root_xml = generate_xml("root_omits_parent_metadata", &root);
    assert!(!root_xml.contains("order=\"0\""));
    assert!(!root_xml.contains("parent-writing-mode="));
    assert!(!root_xml.contains("parent-direction="));
}

#[test]
fn xml_generation_keeps_grid_text_elements_as_containers() {
    for display in ["grid", "inline-grid", "grid-lanes", "inline-grid-lanes"] {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {"display": display, "direction": "ltr"},
            "textContent": "hello",
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 50, "height": 10, "scrollWidth": 50, "scrollHeight": 10},
            "unroundedLayout": {"x": 0, "y": 0, "width": 50, "height": 10, "scrollWidth": 50, "scrollHeight": 10},
            "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 10},
            "children": []
        });

        let xml = generate_xml("grid_text__border_box_ltr", &node);

        assert!(xml.contains(&format!(
            "    <div display=\"{display}\" direction=\"ltr\">"
        )));
        assert!(xml.contains("      hello"));
        assert!(!xml.contains(&format!("<text display=\"{display}\"")));
    }
}

#[test]
fn xml_generation_preserves_explicit_grid_line_names() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "grid-lanes",
            "gridTemplateColumns": [
                {"kind": "line-names", "names": ["lane"]},
                {"kind": "scalar", "unit": "px", "value": 20},
                {"kind": "line-names", "names": ["lane"]},
                {"kind": "scalar", "unit": "px", "value": 30},
                {"kind": "line-names", "names": ["lane"]},
                {"kind": "scalar", "unit": "px", "value": 40},
                {"kind": "line-names", "names": ["lane"]}
            ]
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 90, "height": 0, "scrollWidth": 90, "scrollHeight": 0},
        "unroundedLayout": {"x": 0, "y": 0, "width": 90, "height": 0, "scrollWidth": 90, "scrollHeight": 0},
        "naivelyRoundedLayout": {"clientWidth": 90, "clientHeight": 0},
        "children": []
    });

    let xml = generate_xml("grid_lanes_named_placement__border_box_ltr", &node);

    assert!(xml.contains("grid-template-columns=\"[lane] 20px [lane] 30px [lane] 40px [lane]\""));
}

#[test]
fn xml_generation_preserves_calc_lengths() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "px", "value": 200}, "height": {"unit": "max-content"}},
        "style": {
            "display": "block",
            "size": {"width": {"unit": "calc", "value": "calc(50% + 20px)"}},
            "margin": {"left": {"unit": "calc", "value": "calc(10% - 4px)"}}
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 120, "height": 10, "scrollWidth": 120, "scrollHeight": 10},
        "unroundedLayout": {"x": 0, "y": 0, "width": 120, "height": 10, "scrollWidth": 120, "scrollHeight": 10},
        "naivelyRoundedLayout": {"clientWidth": 120, "clientHeight": 10},
        "children": []
    });

    let xml = generate_xml("calc_lengths__border_box_ltr", &node);

    assert!(xml.contains(r#"width="calc(50% + 20px)""#));
    assert!(xml.contains(r#"margin-left="calc(10% - 4px)""#));
}

#[test]
fn xml_generation_preserves_calc_grid_tracks() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "px", "value": 240}, "height": {"unit": "max-content"}},
        "style": {
            "display": "grid",
            "gridTemplateColumns": [
                {"kind": "scalar", "unit": "calc", "value": "calc(25% + 20px)"},
                {"kind": "scalar", "unit": "px", "value": 80}
            ]
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 240, "height": 10, "scrollWidth": 240, "scrollHeight": 10},
        "unroundedLayout": {"x": 0, "y": 0, "width": 240, "height": 10, "scrollWidth": 240, "scrollHeight": 10},
        "naivelyRoundedLayout": {"clientWidth": 240, "clientHeight": 10},
        "children": []
    });

    let xml = generate_xml("calc_grid_tracks__border_box_ltr", &node);

    assert!(xml.contains(r#"grid-template-columns="calc(25% + 20px) 80px""#));
}

#[test]
fn xml_generation_preserves_grid_template_areas() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "grid",
            "gridTemplateRows": [
                {"kind": "scalar", "unit": "px", "value": 20},
                {"kind": "scalar", "unit": "px", "value": 40}
            ],
            "gridTemplateColumns": [
                {"kind": "scalar", "unit": "px", "value": 30},
                {"kind": "scalar", "unit": "px", "value": 50}
            ],
            "gridTemplateAreas": [
                ["head", "head"],
                ["nav", "main"]
            ]
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 80, "height": 60, "scrollWidth": 80, "scrollHeight": 60},
        "unroundedLayout": {"x": 0, "y": 0, "width": 80, "height": 60, "scrollWidth": 80, "scrollHeight": 60},
        "naivelyRoundedLayout": {"clientWidth": 80, "clientHeight": 60},
        "children": []
    });

    let xml = generate_xml(
        "grid_named_template_area_generated_names__border_box_ltr",
        &node,
    );

    assert!(xml.contains("grid-template-areas=\"head head / nav main\""));
}

#[test]
fn xml_generation_preserves_non_default_font_size() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "block",
            "fontSize": {"unit": "px", "value": 12},
        },
        "textContent": "x",
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 12, "height": 12, "scrollWidth": 12, "scrollHeight": 12},
        "unroundedLayout": {"x": 0, "y": 0, "width": 12, "height": 12, "scrollWidth": 12, "scrollHeight": 12},
        "naivelyRoundedLayout": {"clientWidth": 12, "clientHeight": 12},
        "children": []
    });

    let xml = generate_xml("font_size__border_box_ltr", &node);

    assert!(xml.contains("    <text display=\"block\" font-size=\"12px\">"));
}

#[test]
fn xml_generation_preserves_non_default_font_family() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "block",
            "fontFamily": "monospace",
        },
        "textContent": "x",
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
        "unroundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
        "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 10},
        "children": []
    });

    let xml = generate_xml("font_family__border_box_ltr", &node);

    assert!(xml.contains("    <text display=\"block\" font-family=\"monospace\">"));
}

#[test]
fn xml_generation_elides_default_ahem_font_size() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "block",
            "fontSize": {"unit": "px", "value": 10},
        },
        "textContent": "x",
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
        "unroundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
        "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 10},
        "children": []
    });

    let xml = generate_xml("font_size__border_box_ltr", &node);

    assert!(!xml.contains("font-size"));
}

#[test]
fn xml_generation_preserves_non_default_line_height() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "block",
            "lineHeight": {"unit": "px", "value": 0},
        },
        "textContent": "x",
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 10, "height": 0, "scrollWidth": 10, "scrollHeight": 0},
        "unroundedLayout": {"x": 0, "y": 0, "width": 10, "height": 0, "scrollWidth": 10, "scrollHeight": 0},
        "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 0},
        "children": []
    });

    let xml = generate_xml("line_height__border_box_ltr", &node);

    assert!(xml.contains("    <text display=\"block\" line-height=\"0px\">"));
}

#[test]
fn br_inline_metrics_xml_generation_serializes_complete_br_metrics() {
    let node = json!({
        "tagName": "br",
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "inline",
            "inlineBaseline": "21px",
            "inlineLineHeight": "30px",
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 0, "height": 0, "scrollWidth": 0, "scrollHeight": 0},
        "unroundedLayout": {"x": 0, "y": 0, "width": 0, "height": 0, "scrollWidth": 0, "scrollHeight": 0},
        "naivelyRoundedLayout": {"clientWidth": 0, "clientHeight": 0},
        "children": []
    });

    let xml = generate_xml("br_inline_metrics__border_box_ltr", &node);

    assert!(xml.contains(
            "    <div source-tag=\"br\" display=\"inline\" inline-baseline=\"21px\" inline-line-height=\"30px\"/>"
        ));
}

#[test]
fn br_inline_metrics_xml_generation_does_not_infer_metrics_from_text_styles() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "block",
            "fontSize": {"unit": "px", "value": 20},
            "lineHeight": {"unit": "px", "value": 30},
            "inlineBaseline": "",
            "inlineLineHeight": "",
        },
        "textContent": "x",
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 20, "height": 30, "scrollWidth": 20, "scrollHeight": 30},
        "unroundedLayout": {"x": 0, "y": 0, "width": 20, "height": 30, "scrollWidth": 20, "scrollHeight": 30},
        "naivelyRoundedLayout": {"clientWidth": 20, "clientHeight": 30},
        "children": []
    });

    let xml = generate_xml("non_br_text_styles__border_box_ltr", &node);

    assert!(xml.contains("font-size=\"20px\""));
    assert!(xml.contains("line-height=\"30px\""));
    assert!(!xml.contains("inline-baseline"));
    assert!(!xml.contains("inline-line-height"));
}

#[test]
fn xml_generation_preserves_vertical_align_top() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "inline-grid-lanes",
            "verticalAlign": "top",
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
        "unroundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
        "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 10},
        "children": []
    });

    let xml = generate_xml("vertical_align__border_box_ltr", &node);

    assert!(xml.contains("vertical-align=\"top\""));
}

#[test]
fn bundled_helper_falls_back_to_computed_min_size_units() {
    assert!(TEST_HELPER_SOURCE.contains("parseResolvedDimension(lengthStyleValue(\"minWidth\")"));
    assert!(TEST_HELPER_SOURCE.contains("parseResolvedDimension(lengthStyleValue(\"minHeight\")"));
}

#[test]
fn bundled_helper_preserves_resolved_percentage_min_max_size_values() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-resolved-percent-min-max-size-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("resolved-percent-min-max-size.js");
    let script = format!(
        r#"
const window = {{}};
{TEST_HELPER_SOURCE}

const resolved = parseResolvedDimension("10%", "10%");
const actual = JSON.stringify(parseSize({{ width: resolved, height: resolved }}));
const expected = JSON.stringify({{
  width: {{ unit: "percent", value: 0.1 }},
  height: {{ unit: "percent", value: 0.1 }},
}});
if (actual !== expected) {{
  throw new Error(`resolved percentage min/max size should not be reparsed; got ${{actual}}`);
}}
"#
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run resolved percentage min/max size smoke test");

    assert!(
        output.status.success(),
        "node resolved percentage min/max size smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_uses_computed_font_size() {
    assert!(TEST_HELPER_SOURCE.contains("fontSize: parseDimension(computedStyle.fontSize)"));
}

#[test]
fn bundled_helper_uses_computed_display() {
    assert!(TEST_HELPER_SOURCE.contains("display: parseEnum(computedStyle.display)"));
}

#[test]
fn bundled_helper_uses_computed_writing_mode() {
    assert!(TEST_HELPER_SOURCE.contains("writingMode: parseEnum(computedStyle.writingMode)"));
}

#[test]
fn bundled_helper_lowers_authored_logical_size_to_physical_size() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-logical-size-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("logical-size-capture.js");
    let script = format!(
        r#"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

function capture(style, writingMode) {{
  return parseElementSize((property) => style[property] || "", {{ writingMode }});
}}
function assertEqual(name, actual, expected) {{
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {{
    throw new Error(`${{name}} expected ${{expectedJson}} but got ${{actualJson}}`);
  }}
}}

assertEqual("horizontal", capture({{ inlineSize: "24px", blockSize: "72px" }}, "horizontal-tb"), {{
  width: {{ unit: "px", value: 24 }},
  height: {{ unit: "px", value: 72 }},
}});
assertEqual("vertical", capture({{ inlineSize: "24px", blockSize: "72px" }}, "vertical-rl"), {{
  width: {{ unit: "px", value: 72 }},
  height: {{ unit: "px", value: 24 }},
}});
assertEqual("physical wins", capture({{
  width: "10px",
  height: "11px",
  inlineSize: "24px",
  blockSize: "72px",
}}, "vertical-rl"), {{
  width: {{ unit: "px", value: 10 }},
  height: {{ unit: "px", value: 11 }},
}});
"#
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run logical size capture smoke test");

    assert!(
        output.status.success(),
        "node logical size capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_records_vertical_align() {
    assert!(TEST_HELPER_SOURCE.contains("verticalAlign: parseEnum(styleValue(\"verticalAlign\"))"));
}

#[test]
fn bundled_helper_captures_stylesheet_logical_margins_as_effective_physical_edges() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-effective-margin-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("effective-margin-capture.js");
    let script = format!(
        r#"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{
  styleSheets: [
    {{
      cssRules: [
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".logical",
          style: {{
            0: "margin-inline-start",
            length: 1,
            getPropertyValue(property) {{
              return property === "margin-inline-start" ? "13px" : "";
            }},
          }},
        }},
      ],
    }},
  ],
}};
function getComputedStyle() {{
  throw new Error("unexpected getComputedStyle call");
}}

{TEST_HELPER_SOURCE}

const element = {{
  classList: {{ contains() {{ return false; }} }},
  matches(selector) {{ return selector === ".logical"; }},
  style: {{
    length: 0,
    getPropertyValue() {{ return ""; }},
  }},
}};
const margin = parseEffectiveMargin(element, {{
  marginLeft: "13px",
  marginRight: "0px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}});
const actual = JSON.stringify(margin);
const expected = JSON.stringify({{ left: {{ unit: "px", value: 13 }} }});
if (actual !== expected) {{
  throw new Error(`unexpected effective margin ${{actual}}`);
}}
"#
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run effective margin capture smoke test");

    assert!(
        output.status.success(),
        "node effective margin capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_preserves_authored_percentage_margin_values() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-percentage-margin-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("percentage-margin-capture.js");
    let script = format!(
        r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
function styleDeclaration(entries) {{
  const declaration = {{
    ...Object.fromEntries(entries.flatMap(([property, value]) => [
      [property, value],
      [property.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), value],
    ])),
    length: entries.length,
    getPropertyValue(property) {{
      const entry = entries.find(([name]) => name === property);
      return entry ? entry[1] : "";
    }},
  }};
  entries.forEach(([property], index) => {{
    declaration[index] = property;
  }});
  return declaration;
}}
const document = {{
  styleSheets: [
    {{
      cssRules: [
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".shorthand",
          style: styleDeclaration([["margin", "5% 10% 15% 20%"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".logical",
          style: styleDeclaration([["margin-inline", "20% 10%"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".inline",
          style: styleDeclaration([["margin-left", "10px"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: "#specific",
          style: styleDeclaration([["margin-left", "10px"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".specific",
          style: styleDeclaration([["margin-left", "20%"]]),
        }},
      ],
    }},
  ],
}};
function getComputedStyle() {{
  throw new Error("unexpected getComputedStyle call");
}}

{TEST_HELPER_SOURCE}

function assertMargin(name, element, computedStyle, expected) {{
  const actual = JSON.stringify(parseEffectiveMargin(element, computedStyle));
  const expectedJson = JSON.stringify(expected);
  if (actual !== expectedJson) {{
    throw new Error(`${{name}} expected ${{expectedJson}} but got ${{actual}}`);
  }}
}}
function unit(value, unit) {{
  return {{ value, unit }};
}}
function elementFor(selectors, style, typedOmValues) {{
  const selectorSet = new Set(Array.isArray(selectors) ? selectors : [selectors]);
  return {{
    classList: {{ contains() {{ return false; }} }},
    matches(candidate) {{ return selectorSet.has(candidate); }},
    computedStyleMap() {{
      return {{
        get(property) {{
          return typedOmValues[property] || unit(0, "px");
        }},
      }};
    }},
    style,
  }};
}}

assertMargin("inline longhands", elementFor("", styleDeclaration([
  ["margin-left", "20%"],
  ["margin-right", "10%"],
]), {{
  "margin-left": unit(20, "percent"),
  "margin-right": unit(10, "percent"),
}}), {{
  marginLeft: "20px",
  marginRight: "10px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}}, {{
  left: {{ unit: "percent", value: 0.2 }},
  right: {{ unit: "percent", value: 0.1 }},
}});

assertMargin("stylesheet shorthand", elementFor(".shorthand", styleDeclaration([]), {{
  "margin-left": unit(20, "percent"),
  "margin-right": unit(10, "percent"),
  "margin-top": unit(5, "percent"),
  "margin-bottom": unit(15, "percent"),
}}), {{
  marginLeft: "20px",
  marginRight: "10px",
  marginTop: "5px",
  marginBottom: "15px",
  direction: "ltr",
}}, {{
  left: {{ unit: "percent", value: 0.2 }},
  right: {{ unit: "percent", value: 0.1 }},
  top: {{ unit: "percent", value: 0.05 }},
  bottom: {{ unit: "percent", value: 0.15 }},
}});

assertMargin("stylesheet margin-inline", elementFor(".logical", styleDeclaration([]), {{
  "margin-left": unit(20, "percent"),
  "margin-right": unit(10, "percent"),
}}), {{
  marginLeft: "20px",
  marginRight: "10px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}}, {{
  left: {{ unit: "percent", value: 0.2 }},
  right: {{ unit: "percent", value: 0.1 }},
}});

assertMargin("inline beats stylesheet", elementFor(".inline", styleDeclaration([
  ["margin-left", "20%"],
]), {{
  "margin-left": unit(20, "percent"),
}}), {{
  marginLeft: "20px",
  marginRight: "0px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}}, {{
  left: {{ unit: "percent", value: 0.2 }},
}});

assertMargin("defeated stylesheet percent falls back to computed", elementFor(["#specific", ".specific"], styleDeclaration([]), {{
  "margin-left": unit(10, "px"),
}}), {{
  marginLeft: "10px",
  marginRight: "0px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}}, {{
  left: {{ unit: "px", value: 10 }},
}});
"##
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run percentage margin capture smoke test");

    assert!(
        output.status.success(),
        "node percentage margin capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_preserves_calc_dimension_values() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-calc-dimension-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("calc-dimension-capture.js");
    let script = format!(
        r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};
function getComputedStyle() {{
  throw new Error("unexpected getComputedStyle call");
}}

{TEST_HELPER_SOURCE}

function assertEqual(name, actual, expected) {{
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {{
    throw new Error(`${{name}} expected ${{expectedJson}} but got ${{actualJson}}`);
  }}
}}

// Live Chrome 149 probe: computedStyleMap().get("width") for
// width: calc(50% + 20px) returned CSSMathSum, operator "sum",
// values [CSSUnitValue percent 50, CSSUnitValue px 20], and toString()
// reconstructed "calc(50% + 20px)". margin-left subtraction also returned
// CSSMathSum with the negative px term represented by CSSMathNegate.
assertEqual("typed om calc", parseDimension({{
  toString() {{ return "calc(50% + 20px)"; }},
}}), {{ unit: "calc", value: "calc(50% + 20px)" }});

assertEqual(
  "inline authored calc fallback",
  parseDimension("calc(10% - 4px)"),
  {{ unit: "calc", value: "calc(10% - 4px)" }}
);
"##
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run calc dimension capture smoke test");

    assert!(
        output.status.success(),
        "node calc dimension capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_preserves_calc_grid_tracks() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-calc-grid-track-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("calc-grid-track-capture.js");
    let script = format!(
        r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

const actual = JSON.stringify(parseGridTrackDefinitions("calc(25% + 20px) 80px"));
const expected = JSON.stringify([
  {{ kind: "scalar", unit: "calc", value: "calc(25% + 20px)" }},
  {{ kind: "scalar", unit: "px", value: 80 }},
]);
if (actual !== expected) {{
  throw new Error(`unexpected calc grid tracks ${{actual}}`);
}}
"##
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run calc grid track capture smoke test");

    assert!(
        output.status.success(),
        "node calc grid track capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_preserves_single_calc_gap_shorthand() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-calc-gap-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("calc-gap-capture.js");
    let script = format!(
        r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

const actual = JSON.stringify(parseGaps((property) => {{
  if (property === "gap") return {{ unit: "calc", value: "calc(5% + 2px)" }};
  return "";
}}));
const expected = JSON.stringify({{
  row: {{ unit: "calc", value: "calc(5% + 2px)" }},
  column: {{ unit: "calc", value: "calc(5% + 2px)" }},
}});
if (actual !== expected) {{
  throw new Error(`unexpected calc gap ${{actual}}`);
}}
"##
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run calc gap capture smoke test");

    assert!(
        output.status.success(),
        "node calc gap capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_preserves_inline_shorthand_calc_margin_with_typed_om() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-inline-calc-margin-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("inline-calc-margin-capture.js");
    let script = format!(
        r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};
function styleDeclaration(entries) {{
  const declaration = {{
    ...Object.fromEntries(entries.flatMap(([property, value]) => [
      [property, value],
      [property.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), value],
    ])),
    length: entries.length,
    getPropertyValue(property) {{
      const entry = entries.find(([name]) => name === property);
      return entry ? entry[1] : "";
    }},
  }};
  entries.forEach(([property], index) => {{
    declaration[index] = property;
  }});
  return declaration;
}}

{TEST_HELPER_SOURCE}

const element = {{
  classList: {{ contains() {{ return false; }} }},
  matches() {{ return false; }},
  style: styleDeclaration([["margin", "calc(10% - 4px) 0px"]]),
  computedStyleMap() {{
    return {{
      get(property) {{
        if (property === "margin-top") return {{ toString() {{ return "calc(10% - 4px)"; }} }};
        return {{ unit: "px", value: 0 }};
      }},
    }};
  }},
}};
const margin = parseEffectiveMargin(element, {{
  marginLeft: "0px",
  marginRight: "0px",
  marginTop: "16px",
  marginBottom: "0px",
  direction: "ltr",
}});
const actual = JSON.stringify(margin);
const expected = JSON.stringify({{
  top: {{ unit: "calc", value: "calc(10% - 4px)" }},
}});
if (actual !== expected) {{
  throw new Error(`unexpected inline shorthand calc margin ${{actual}}`);
}}
"##
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run inline calc margin capture smoke test");

    assert!(
        output.status.success(),
        "node inline calc margin capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_does_not_preserve_stylesheet_scanned_calc_lengths() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-stylesheet-calc-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("stylesheet-calc-capture.js");
    let script = format!(
        r##"
const window = {{ innerWidth: 800 }};
const Node = {{ ELEMENT_NODE: 1, TEXT_NODE: 3 }};
const CSSRule = {{ STYLE_RULE: 1 }};
function styleDeclaration(entries) {{
  const declaration = {{
    ...Object.fromEntries(entries.flatMap(([property, value]) => [
      [property, value],
      [property.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), value],
    ])),
    length: entries.length,
    getPropertyValue(property) {{
      const entry = entries.find(([name]) => name === property);
      return entry ? entry[1] : "";
    }},
  }};
  entries.forEach(([property], index) => {{
    declaration[index] = property;
  }});
  return declaration;
}}
const document = {{
  styleSheets: [
    {{
      cssRules: [
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".stylesheet-calc",
          style: styleDeclaration([
            ["width", "calc(50% + 20px)"],
            ["height", "10px"],
            ["grid-template-columns", "calc(25% + 20px) 80px"],
          ]),
        }},
      ],
    }},
  ],
  createElement() {{
    return {{
      style: {{}},
      offsetWidth: 0,
      clientWidth: 0,
      remove() {{}},
    }};
  }},
  body: {{ appendChild() {{}} }},
}};

{TEST_HELPER_SOURCE}

const parent = {{
  getBoundingClientRect() {{ return {{ x: 0, y: 0, width: 200, height: 10, right: 200, left: 0, bottom: 10, top: 0 }}; }},
  classList: {{ contains() {{ return false; }} }},
  clientLeft: 0,
  clientTop: 0,
}};
const element = {{
  tagName: "DIV",
  classList: {{ contains() {{ return false; }} }},
  matches(selector) {{ return selector === ".stylesheet-calc"; }},
  style: styleDeclaration([]),
  computedStyleMap() {{
    return {{
      get(property) {{
        if (property === "width") return {{ toString() {{ return "calc(50% + 20px)"; }} }};
        if (property === "height") return {{ unit: "px", value: 10 }};
        if (property === "grid-template-columns") return {{
          toString() {{ return "calc(25% + 20px) 80px"; }},
        }};
        return undefined;
      }},
    }};
  }},
  parentNode: parent,
  childNodes: [],
  childElementCount: 0,
  textContent: "",
  getBoundingClientRect() {{ return {{ x: 0, y: 0, width: 120, height: 10, right: 120, left: 0, bottom: 10, top: 0 }}; }},
  scrollWidth: 120,
  scrollHeight: 10,
  clientWidth: 120,
  clientHeight: 10,
  offsetWidth: 120,
  offsetHeight: 10,
  offsetLeft: 0,
  offsetTop: 0,
  getAttribute() {{ return null; }},
}};
function getComputedStyle() {{
  return {{
    display: "block",
    boxSizing: "border-box",
    direction: "ltr",
    writingMode: "horizontal-tb",
    fontFamily: "ahem",
    fontSize: "10px",
    lineHeight: "10px",
    width: "120px",
    height: "10px",
    minWidth: "0px",
    minHeight: "0px",
    maxWidth: "none",
    maxHeight: "none",
    marginLeft: "0px",
    marginRight: "0px",
    marginTop: "0px",
    marginBottom: "0px",
  }};
}}

const data = describeElement(element, {{}});
const actual = JSON.stringify(data.style.size);
const expected = JSON.stringify({{ height: {{ unit: "px", value: 10 }} }});
if (actual !== expected) {{
  throw new Error(`stylesheet calc should not be preserved; got ${{actual}}`);
}}
if (data.style.gridTemplateColumns !== undefined) {{
  throw new Error(`stylesheet compound calc should not be preserved; got ${{JSON.stringify(data.style.gridTemplateColumns)}}`);
}}
"##
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run stylesheet calc capture smoke test");

    assert!(
        output.status.success(),
        "node stylesheet calc capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn bundled_helper_does_not_guess_stylesheet_auto_margin_when_specificity_defeats_it() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-defeated-auto-margin-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("defeated-auto-margin-capture.js");
    let script = format!(
        r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
function styleDeclaration(entries) {{
  const declaration = {{
    ...Object.fromEntries(entries.map(([property, value]) => [property.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), value])),
    ...Object.fromEntries(entries.map(([property, value]) => [property, value])),
    length: entries.length,
    getPropertyValue(property) {{
      const entry = entries.find(([name]) => name === property);
      return entry ? entry[1] : "";
    }},
  }};
  entries.forEach(([property], index) => {{
    declaration[index] = property;
  }});
  return declaration;
}}
const document = {{
  styleSheets: [
    {{
      cssRules: [
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: "#item",
          style: styleDeclaration([["margin-left", "10px"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".item",
          style: styleDeclaration([["margin-left", "auto"]]),
        }},
      ],
    }},
  ],
}};
function getComputedStyle() {{
  throw new Error("unexpected getComputedStyle call");
}}

{TEST_HELPER_SOURCE}

const element = {{
  classList: {{ contains() {{ return false; }} }},
  matches(selector) {{ return selector === "#item" || selector === ".item"; }},
  style: {{
    length: 0,
    getPropertyValue() {{ return ""; }},
  }},
}};
const margin = parseEffectiveMargin(element, {{
  marginLeft: "10px",
  marginRight: "0px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}});
const actual = JSON.stringify(margin);
const expected = JSON.stringify({{ left: {{ unit: "px", value: 10 }} }});
if (actual !== expected) {{
  throw new Error(`unexpected effective margin ${{actual}}`);
}}
"##
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run defeated auto margin capture smoke test");

    assert!(
        output.status.success(),
        "node defeated auto margin capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn generator_injected_capture_records_grid_template_areas() {
    assert!(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT.contains("gridTemplateAreas"));
    assert!(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT.contains("originalDescribeElement"));
    assert!(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT.contains("authoredStyleValue"));
    assert!(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT.contains("computedStyle.gridTemplateAreas"));
}

#[test]
fn generator_injected_capture_reads_local_authored_grid_template_areas() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-grid-template-area-capture-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp dir");
    let script_path = root.join("grid-template-area-capture.js");
    let script = format!(
        r#"
const window = {{}};
const element = {{ style: {{ gridTemplateAreas: "" }} }};
function getComputedStyle() {{
  return {{ gridTemplateAreas: "none" }};
}}
function authoredStyleValue(element, property) {{
  if (property !== "gridTemplateAreas") {{
    throw new Error(`unexpected property ${{property}}`);
  }}
  return '"head head" "nav main"';
}}
function describeElement() {{
  return {{ style: {{ display: "grid" }} }};
}}

{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT}

const data = describeElement(element);
const actual = JSON.stringify(data.style.gridTemplateAreas);
const expected = JSON.stringify([["head", "head"], ["nav", "main"]]);
if (actual !== expected) {{
  throw new Error(`unexpected gridTemplateAreas ${{actual}}`);
}}
"#
    );
    fs::write(&script_path, script).expect("script");

    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("node should run grid template area capture smoke test");

    assert!(
        output.status.success(),
        "node grid template area capture smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn fri08_c05_adapter_missing_helper_runtime_injection_keeps_finite_area_lowering_active() {
    let script = format!(
        r#"
const window = {{ innerWidth: 800 }};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

const helperDescribeElement = describeElement;
const helperClaimedGridTemplateAreaCapture =
  window.__surgeistGridTemplateAreaCaptureInstalled === true;

{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT}

if (
  typeof getTestData !== "function" ||
  !helperClaimedGridTemplateAreaCapture ||
  describeElement !== helperDescribeElement
) {{
  throw new Error(
    `runtime injection did not keep current finite helper lowering: loaded=${{typeof getTestData === "function"}} claimed=${{helperClaimedGridTemplateAreaCapture}} wrapped=${{describeElement !== helperDescribeElement}}`
  );
}}

describeChildNodes = () => [];
brInlineMetricsForElement = () => undefined;
layoutReadyLineControlParticipation = () => undefined;
layoutReadyAnonymousGridTextWrapper = () => undefined;
unsupportedElementReason = () => undefined;
unsupportedChildNodesReason = () => undefined;
getScrollBarWidth = () => 0;
parseEffectiveMargin = () => ({{}});
parseViewportConstraint = () => ({{ width: {{ unit: "max-content" }}, height: {{ unit: "max-content" }}, rootContext: "root" }});
layoutReadyShapeBands = () => undefined;
normalizedFlexItemCollapse = () => undefined;

const parent = {{
  clientLeft: 0,
  clientTop: 0,
  classList: {{ contains() {{ return false; }} }},
  getBoundingClientRect() {{ return {{ x: 0, y: 0, left: 0, right: 80, top: 0, bottom: 40, width: 80, height: 40 }}; }},
}};
const inlineStyle = new Proxy({{
  gridTemplateAreas: '"head head" "nav main"',
}}, {{ get(target, property) {{ return target[property] ?? ""; }} }});
const computedStyle = new Proxy({{
  display: "grid",
  boxSizing: "border-box",
  direction: "ltr",
  writingMode: "horizontal-tb",
  fontFamily: "monospace",
  fontSize: "10px",
  lineHeight: "10px",
  minWidth: "0px",
  minHeight: "0px",
  maxWidth: "none",
  maxHeight: "none",
  gridTemplateAreas: '"computed computed"',
}}, {{ get(target, property) {{ return target[property] ?? ""; }} }});
function getComputedStyle() {{ return computedStyle; }}
const element = {{
  id: "original-source__border_box_ltr",
  tagName: "DIV",
  style: inlineStyle,
  parentNode: parent,
  parentElement: parent,
  classList: {{ contains() {{ return false; }} }},
  childNodes: [],
  childElementCount: 0,
  textContent: "",
  scrollWidth: 80,
  scrollHeight: 40,
  clientWidth: 80,
  clientHeight: 40,
  offsetWidth: 80,
  offsetHeight: 40,
  offsetLeft: 0,
  offsetTop: 0,
  getAttribute() {{ return null; }},
  getBoundingClientRect() {{ return {{ x: 0, y: 0, left: 0, right: 80, top: 0, bottom: 40, width: 80, height: 40 }}; }},
}};

const authored = describeElement(element).style.gridTemplateAreas;
const expectedAuthored = [["head", "head"], ["nav", "main"]];
if (JSON.stringify(authored) !== JSON.stringify(expectedAuthored)) {{
  throw new Error(`missing authored gridTemplateAreas serialization: ${{JSON.stringify(authored)}}`);
}}

inlineStyle.gridTemplateAreas = "";
const computed = describeElement(element).style.gridTemplateAreas;
if (JSON.stringify(computed) !== JSON.stringify([["computed", "computed"]])) {{
  throw new Error(`missing computed gridTemplateAreas serialization: ${{JSON.stringify(computed)}}`);
}}

element.id = "renamed-source__content_box_rtl";
element.getBoundingClientRect = () => ({{ x: 99, y: 88, left: 99, right: 876, top: 88, bottom: 754, width: 777, height: 666 }});
const mutated = describeElement(element).style.gridTemplateAreas;
if (JSON.stringify(mutated) !== JSON.stringify(computed)) {{
  throw new Error(`source, variant, or geometry changed serialized input: ${{JSON.stringify(mutated)}}`);
}}
computedStyle.gridTemplateAreas = "none";
for (const [kind, value] of [
  ["malformed", 'head head'],
  ["unequal-row", '"head head" "main"'],
  ["invalid-ident", '"head @" "nav main"'],
  ["non-rectangular", '"head head" "head main"'],
]) {{
  inlineStyle.gridTemplateAreas = value;
  let rejected = false;
  try {{ describeElement(element); }} catch (_) {{ rejected = true; }}
  if (!rejected) {{
    throw new Error(`active runtime describeElement accepted ${{kind}} grid-template-areas: ${{value}}`);
  }}
}}
console.log(JSON.stringify(mutated));
"#
    );

    let helper_areas = run_bundled_helper_json("fri08-c05-grid-template-areas", script);
    assert_eq!(helper_areas, json!([["computed", "computed"]]));

    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {
            "display": "grid",
            "gridTemplateRows": [
                {"kind": "scalar", "unit": "px", "value": 20}
            ],
            "gridTemplateColumns": [
                {"kind": "scalar", "unit": "px", "value": 30},
                {"kind": "scalar", "unit": "px", "value": 50}
            ],
            "gridTemplateAreas": helper_areas
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 80, "height": 20, "scrollWidth": 80, "scrollHeight": 20},
        "unroundedLayout": {"x": 0, "y": 0, "width": 80, "height": 20, "scrollWidth": 80, "scrollHeight": 20},
        "naivelyRoundedLayout": {"clientWidth": 80, "clientHeight": 20},
        "children": []
    });
    let xml = generate_xml("renamed_source__content_box_rtl", &node);
    assert!(xml.contains("grid-template-areas=\"computed computed\""));
    let golden = browser_parity_support::Golden::parse(&xml)
        .expect("helper-generated template areas should survive XML parsing");
    assert_eq!(
        golden.root.style.get("grid-template-areas"),
        Some("computed computed")
    );
    browser_parity_support::assert_surgeist_matches(&golden)
        .expect("helper-generated XML should reach public layout");
}

#[test]
fn fri08_c05_adapter_runtime_injection_rejects_non_finite_grid_template_areas() {
    let script = format!(
        r#"
const window = {{}};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT}

if (typeof parseGridTemplateAreas !== "function") {{
  throw new Error("bundled helper is missing parseGridTemplateAreas");
}}
for (const [kind, value] of [
  ["malformed", 'head head'],
  ["unequal-row", '"head head" "main"'],
  ["invalid-ident", '"head @" "nav main"'],
  ["non-rectangular", '"head head" "head main"'],
]) {{
  let rejected = false;
  try {{ parseGridTemplateAreas(value); }} catch (_) {{ rejected = true; }}
  if (!rejected) throw new Error(`${{kind}} grid-template-areas accepted: ${{value}}`);
}}
"#
    );

    run_bundled_helper_script("fri08-c05-invalid-grid-template-areas", script);
}

#[test]
fn bundled_helper_records_default_root_viewport_as_max_content() {
    assert!(TEST_HELPER_SOURCE.contains("width: rootFillsBrowserViewport"));
    assert!(TEST_HELPER_SOURCE.contains(": { unit: 'max-content' }"));
}

#[test]
fn bundled_helper_records_viewport_width_when_root_fills_browser_viewport() {
    assert!(TEST_HELPER_SOURCE.contains("rootFillsBrowserViewport"));
    assert!(TEST_HELPER_SOURCE.contains("px(window.innerWidth)"));
    assert!(TEST_HELPER_SOURCE.contains("Math.round(boundingRect.width) === window.innerWidth"));
}

#[test]
fn bundled_helper_captures_exact_order_and_flex_parent_axes() {
    let script = format!(
        r#"
const window = {{ innerWidth: 800 }};
const document = {{
  styleSheets: [],
  createElement() {{
    return {{ style: {{}}, offsetWidth: 0, clientWidth: 0, remove() {{}} }};
  }},
  body: {{ appendChild() {{}} }},
}};

{TEST_HELPER_SOURCE}

let parentWritingMode = "horizontal-tb";
let parentDirection = "ltr";
let rootWidth = 160;
let rootHeight = 20;
const parent = {{
  classList: {{ contains(name) {{ return name === "viewport"; }} }},
  style: {{ width: "400px", height: "" }},
  getBoundingClientRect() {{ return {{ x: 0, y: 0, width: 400, height: 20, right: 400, left: 0, bottom: 20, top: 0 }}; }},
  clientLeft: 0,
  clientTop: 0,
}};
const element = {{
  tagName: "DIV",
  classList: {{ contains() {{ return false; }} }},
  style: {{
    gridTemplateRows: "",
    gridTemplateColumns: "",
    gridAutoRows: "",
    gridAutoColumns: "",
    gridRowStart: "auto",
    gridRowEnd: "auto",
    gridColumnStart: "auto",
    gridColumnEnd: "auto",
  }},
  parentNode: parent,
  parentElement: parent,
  childNodes: [],
  childElementCount: 0,
  textContent: "",
  getBoundingClientRect() {{ return {{ x: 0, y: 0, width: rootWidth, height: rootHeight, right: rootWidth, left: 0, bottom: rootHeight, top: 0 }}; }},
  scrollWidth: 160,
  scrollHeight: 20,
  clientWidth: 160,
  clientHeight: 20,
  offsetWidth: 160,
  offsetHeight: 20,
  offsetLeft: 0,
  offsetTop: 0,
  getAttribute() {{ return null; }},
}};
let order = "0";
function getComputedStyle(target) {{
  if (target === parent) {{
    return {{ writingMode: parentWritingMode, direction: parentDirection }};
  }}
  return {{
    display: "grid",
    boxSizing: "border-box",
    direction: "ltr",
    writingMode: "horizontal-tb",
    order,
    fontFamily: "ahem",
    fontSize: "10px",
    lineHeight: "10px",
    width: "160px",
    height: "20px",
    minWidth: "0px",
    minHeight: "0px",
    maxWidth: "none",
    maxHeight: "none",
    marginLeft: "0px",
    marginRight: "0px",
    marginTop: "0px",
    marginBottom: "0px",
  }};
}}

for (const testCase of [
  {{ order: "0", writingMode: "horizontal-tb", direction: "ltr", width: 160, height: 20, hostInlineSize: 160 }},
  {{ order: "-2147483648", writingMode: "vertical-rl", direction: "rtl", width: 160, height: 37.5, hostInlineSize: 37.5 }},
  {{ order: "2147483647", writingMode: "horizontal-tb", direction: "rtl", width: 80, height: 60, hostInlineSize: 80 }},
]) {{
  order = testCase.order;
  parentWritingMode = testCase.writingMode;
  parentDirection = testCase.direction;
  rootWidth = testCase.width;
  rootHeight = testCase.height;
  const data = describeElement(element);
  if (data.style.order !== testCase.order) {{
    throw new Error(`expected exact order ${{testCase.order}}, got ${{data.style.order}}`);
  }}
  if (data.viewport.rootContext !== "flex-item") {{
    throw new Error(`expected flex-item root, got ${{data.viewport.rootContext}}`);
  }}
  if (data.viewport.parentWritingMode !== testCase.writingMode || data.viewport.parentDirection !== testCase.direction) {{
    throw new Error(`expected parent axes, got ${{JSON.stringify(data.viewport)}}`);
  }}
  if (data.viewport.hostInlineSize !== testCase.hostInlineSize) {{
    throw new Error(`expected host inline size ${{testCase.hostInlineSize}}, got ${{data.viewport.hostInlineSize}}`);
  }}
}}
"#
    );

    run_bundled_helper_script("exact-order-and-flex-parent-axes", script);
}

#[test]
fn br_inline_metrics_bundled_helper_describes_br_with_layout_ready_metrics() {
    assert!(TEST_HELPER_SOURCE.contains("tagName: e.tagName.toLowerCase()"));
    assert!(TEST_HELPER_SOURCE.contains("unsupportedElementReason"));
    assert!(!TEST_HELPER_SOURCE.contains("Unsupported <br> line-break semantics"));

    run_bundled_helper_script(
        "br-supported-block-parent",
        br_helper_smoke_script("block", "horizontal-tb", false, None),
    );

    let node = json!({
        "tagName": "br",
        "style": {
            "display": "inline",
            "inlineBaseline": "8px",
            "inlineLineHeight": "10px",
        },
    });
    assert_eq!(
        input_attrs(&node),
        vec![
            ("source-tag", "br".to_string()),
            ("display", "inline".to_string()),
            ("inline-baseline", "8px".to_string()),
            ("inline-line-height", "10px".to_string())
        ]
    );
}

#[test]
fn bundled_helper_keeps_vertical_br_explicitly_unsupported() {
    assert!(!TEST_HELPER_SOURCE.contains("Unsupported <br> line-break semantics"));
    assert!(TEST_HELPER_SOURCE.contains("Unsupported vertical <br> line-break semantics"));

    run_bundled_helper_script(
        "br-vertical-unsupported",
        br_helper_smoke_script(
            "block",
            "vertical-rl",
            false,
            Some("Unsupported vertical <br> line-break semantics"),
        ),
    );
    run_bundled_helper_script(
        "br-vertical-layout-ready",
        br_helper_smoke_script("block", "vertical-rl", true, None),
    );
}

#[test]
fn bundled_helper_keeps_unmodeled_br_parent_contexts_unsupported() {
    assert!(!TEST_HELPER_SOURCE.contains("Unsupported <br> line-break semantics"));
    assert!(TEST_HELPER_SOURCE.contains("Unsupported <br> outside block inline-run semantics"));

    run_bundled_helper_script(
        "br-inline-parent-unsupported",
        br_helper_smoke_script(
            "inline",
            "horizontal-tb",
            false,
            Some("Unsupported <br> outside block inline-run semantics"),
        ),
    );
}

#[test]
fn bundled_helper_rejects_mixed_text_content_instead_of_synthesizing_margins() {
    assert!(TEST_HELPER_SOURCE.contains("unsupportedChildNodesReason"));
    assert!(TEST_HELPER_SOURCE.contains("Unsupported mixed text/element content"));
    assert!(TEST_HELPER_SOURCE.contains("isSignificantInlineWhitespace"));
    assert!(!TEST_HELPER_SOURCE.contains("addLeadingInlineWhitespaceMargin"));
    assert!(!TEST_HELPER_SOURCE.contains("style.margin.left = { unit: 'px', value: width }"));
}

#[test]
fn bundled_helper_reports_missing_test_root_as_unsupported() {
    assert!(TEST_HELPER_SOURCE.contains("unsupportedTestData"));
    assert!(TEST_HELPER_SOURCE.contains("Unsupported missing #test-root fixture root"));
}

#[test]
fn xml_generation_applies_root_rounding_policy_to_descendants() {
    let node = json!({
        "useRounding": false,
        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
        "style": {"direction": "ltr"},
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 200, "height": 42, "scrollWidth": 200, "scrollHeight": 42},
        "unroundedLayout": {"x": 0, "y": 0, "width": 200, "height": 42.15625, "scrollWidth": 200, "scrollHeight": 42},
        "naivelyRoundedLayout": {"clientWidth": 200, "clientHeight": 42},
        "children": [
            {
                "useRounding": true,
                "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                "style": {"direction": "ltr"},
                "smartRoundedLayout": {"x": 8, "y": 8, "width": 97, "height": 26, "scrollWidth": 97, "scrollHeight": 26},
                "unroundedLayout": {"x": 8, "y": 8, "width": 97, "height": 26.15625, "scrollWidth": 97, "scrollHeight": 26},
                "naivelyRoundedLayout": {"clientWidth": 97, "clientHeight": 26},
                "children": [
                    {
                        "useRounding": true,
                        "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                        "style": {"direction": "ltr"},
                        "smartRoundedLayout": {"x": 10, "y": 10, "width": 38, "height": 6, "scrollWidth": 38, "scrollHeight": 6},
                        "unroundedLayout": {"x": 10.078125, "y": 10.078125, "width": 38.40625, "height": 6, "scrollWidth": 38, "scrollHeight": 6},
                        "naivelyRoundedLayout": {"clientWidth": 38, "clientHeight": 6},
                        "children": []
                    }
                ]
            }
        ]
    });

    let xml = generate_xml("percentage_moderate_complexity__border_box_ltr", &node);

    assert!(xml.contains(
        "<test name=\"percentage_moderate_complexity__border_box_ltr\" use-rounding=\"false\">"
    ));
    assert!(xml.contains("    <node x=\"0\" y=\"0\" width=\"200\" height=\"42.15625\">"));
    assert!(xml.contains("      <node x=\"8\" y=\"8\" width=\"97\" height=\"26.15625\">"));
    assert!(xml.contains(
        "        <node x=\"10.078125\" y=\"10.078125\" width=\"38.40625\" height=\"6\"/>"
    ));
}

#[test]
fn layout_xml_attrs_use_f32_compatible_browser_fixture_boundary() {
    assert_eq!(layout_number_attr_value(137.203125), "137.20313");
    assert_eq!(layout_number_attr_value(42.15625), "42.15625");
    assert_eq!(layout_number_attr_value(10.0), "10");
    assert_eq!(number_attr_value(55.00000000000001), "55.00000000000001");
}

#[test]
fn browser_document_script_writes_json_escaped_html() {
    let script = browser_document_write_script("<!doctype html><p>quote\"</p>");

    assert!(script.contains("document.write"));
    assert!(script.contains(r#"quote\""#));
    assert!(!script.contains("Page.navigate"));
}

#[test]
fn browser_fixture_document_inlines_base_style_only_when_authored_fixture_references_it() {
    let raw_without_style =
        "<!doctype html><html><head></head><body><div id=\"test-root\"></div></body></html>";
    let raw_with_style = r#"<!doctype html><html><head><link rel="stylesheet" href="../../scripts/gentest/test_base_style.css"></head><body><div></div></body></html>"#;
    let with_style = browser_fixture_document(raw_with_style, "file:///tmp/fixtures/flex/")
        .expect("fixture document");
    let without_style = browser_fixture_document(raw_without_style, "file:///tmp/fixtures/html/")
        .expect("fixture document");

    assert!(with_style.contains("<base href=\"file:///tmp/fixtures/flex/\">"));
    assert!(with_style.contains("#test-root"));
    assert!(without_style.contains("<base href=\"file:///tmp/fixtures/html/\">"));
    assert!(!without_style.contains("#test-root"));
}

#[test]
fn browser_fixture_document_preserves_doctype_when_synthesizing_head() {
    let raw = "<!doctype html><div id=\"test-root\"></div>";
    let document =
        browser_fixture_document(raw, "file:///tmp/fixtures/html/").expect("fixture document");

    assert!(document.starts_with("<!doctype html><html><head>"));
    assert!(document.contains("<base href=\"file:///tmp/fixtures/html/\">"));
    assert!(document.contains("</head><body><div id=\"test-root\"></div></body></html>"));
}

#[test]
fn browser_fixture_document_inserts_head_inside_no_head_html_document() {
    let raw = "<!doctype html><html><body><div id=\"test-root\"></div></body></html>";
    let document =
        browser_fixture_document(raw, "file:///tmp/fixtures/html/").expect("fixture document");

    assert!(document.starts_with("<!doctype html><html><head>"));
    assert!(document.contains("</head><body><div id=\"test-root\"></div></body></html>"));
}

#[test]
fn xml_generation_is_comment_free() {
    let node = serde_json::json!({
        "useRounding": true,
        "viewport": { "width": "max-content", "height": "max-content" },
        "style": { "display": "block" },
        "children": [],
        "smartRoundedLayout": { "x": 0, "y": 0, "width": 10, "height": 20 },
        "unroundedLayout": { "x": 0, "y": 0, "width": 10, "height": 20 },
        "naivelyRoundedLayout": { "clientWidth": 10, "clientHeight": 20 }
    });
    let xml = generate_xml("basic__border_box_ltr", &node);

    assert!(xml.starts_with("<test "));
    assert!(!xml.contains("generated-by: surgeist-layout-generate"));
}

#[test]
fn grid_position_serializes_named_lines_and_spans() {
    assert_eq!(
        grid_position(&serde_json::json!({
            "kind": "named-line",
            "name": "a",
            "value": 8
        })),
        Some("a 8".to_string())
    );
    assert_eq!(
        grid_position(&serde_json::json!({
            "kind": "named-span",
            "name": "a",
            "value": 0
        })),
        Some("span a".to_string())
    );
    assert_eq!(
        grid_position(&serde_json::json!({
            "kind": "named-span",
            "name": "a",
            "value": 2
        })),
        Some("span 2 a".to_string())
    );
}

#[test]
fn track_definition_serializes_subgrid_line_names() {
    assert_eq!(
        track_definition(&serde_json::json!({
            "kind": "subgrid",
            "lineNames": [
                [],
                [],
                [],
                ["b"]
            ]
        })),
        Some("subgrid [] [] [] [b]".to_string())
    );
}

#[test]
fn fri06_c08r_zero_width_whitespace_helper_preserves_discard_anchor() {
    let script = [
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
let rect = { x: 100, y: 50, left: 100, top: 50, right: 100, bottom: 50, width: 0, height: 0 };
let fragments = [];
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return rect; },
  getClientRects() { return fragments; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const root = {
  parentElement: null,
  getAttribute(name) { return name === "data-surgeist-layout-ready-inline" ? "true" : null; },
  getBoundingClientRect() {
    return { x: 0, y: 0, left: 0, top: 0, right: 100, bottom: 100, width: 100, height: 100 };
  },
};
const first = { nodeType: Node.ELEMENT_NODE, style: { display: "inline-block" } };
const second = { nodeType: Node.ELEMENT_NODE, style: { display: "inline-block" } };
const whitespace = { nodeType: Node.TEXT_NODE, textContent: "\n      ", parentElement: null };
const parent = {
  parentElement: root,
  childNodes: [{ nodeType: Node.TEXT_NODE, textContent: "\n      " }, first, whitespace, second],
};
whitespace.parentElement = parent;
function getComputedStyle(element) {
  return {
    display: element === parent ? "block" : element.style?.display || "block",
    direction: "ltr", writingMode: "horizontal-tb", fontSize: "0px", lineHeight: "0px",
  };
}
"#,
        TEST_HELPER_SOURCE,
        r#"
function mustReject(label, callback) {
  try { callback(); } catch (_) { return; }
  throw new Error(`${label} did not reject`);
}
const shaped = layoutReadyTextNodeData(whitespace, parent, 2);
const segment = shaped.inlineSegments[0];
if (JSON.stringify({
  id: segment.id,
  inlineExtent: segment.inlineExtent,
  inlineBaseline: segment.inlineBaseline,
  inlineLineHeight: segment.inlineLineHeight,
  whitespaceEdge: segment.whitespaceEdge,
  followingBreak: segment.followingBreak,
  rangeInks: shaped.rangeInks,
}) !== JSON.stringify({
  id: 2, inlineExtent: 0, inlineBaseline: 0, inlineLineHeight: 0,
  whitespaceEdge: "discard-at-both", followingBreak: "allowed", rangeInks: [],
})) {
  throw new Error(`zero-width discard anchor changed: ${JSON.stringify(shaped)}`);
}
mustReject("non-whitespace zero fragment", () =>
  layoutReadyTextNodeData({ ...whitespace, textContent: "x" }, parent, 2));
rect = { ...rect, right: 101, width: 1 };
mustReject("nonzero bounding tuple", () => layoutReadyTextNodeData(whitespace, parent, 2));
rect = { x: 100, y: 50, left: 100, top: 50, right: undefined, bottom: 50, width: 0, height: 0 };
mustReject("incomplete bounding tuple", () => layoutReadyTextNodeData(whitespace, parent, 2));
rect = { x: 100, y: 50, left: 100, top: 50, right: 100, bottom: 50, width: 0, height: 0 };
fragments = [rect, rect];
mustReject("multiple fragments", () => layoutReadyTextNodeData(whitespace, parent, 2));
"#,
    ]
    .concat();
    run_bundled_helper_script("fri06-c08r-zero-width-whitespace", script);
}

#[test]
fn fri06_c08r_empty_range_serializer_preserves_explicit_zero_observations() {
    let mut text = fri06_c08r_fixture_input_text(7);
    text["rangeInks"] = json!([]);
    let node = fri06_c08r_fixture_input_root("block", vec![text]);

    let xml = generate_xml("fri06_c08r_empty_range", &node);
    assert!(xml.contains("<range-inks/>"), "{xml}");
}

#[test]
fn fri06_c08r_fixture_input_serializer_emits_closed_explicit_forms() {
    let node = json!({
        "tagName": "div",
        "layoutReadyInlineRoot": true,
        "useRounding": false,
        "viewport": {"width": {"unit": "px", "value": 100}, "height": {"unit": "max-content"}},
        "style": {"display": "block", "size": {"width": {"unit": "px", "value": 100}}},
        "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 10},
        "children": [
            {"layoutInput": "inline-boundary", "inlineBoundary": {"kind": "start"}, "children": []},
            {
                "layoutInput": "inline-text",
                "inlineSegments": [{
                    "id": 7, "inlineExtent": 10, "inlineBaseline": 8, "inlineLineHeight": 10,
                    "bidiLevel": 0, "whitespaceEdge": "preserve", "followingBreak": "prohibited"
                }],
                "rangeInks": [{"sourceSegmentId": 7, "lineIndex": 0, "physicalStartEdge": "left", "start": 0, "advance": 10}],
                "children": []
            },
            {"layoutInput": "inline-boundary", "inlineBoundary": {"kind": "end"}, "children": []}
        ]
    });
    let xml = generate_xml("renamed_explicit_fixture", &node);
    assert!(xml.contains(r#"<inline-boundary kind="start"/>"#), "{xml}");
    assert!(xml.contains(r#"<inline-boundary kind="end"/>"#), "{xml}");
    let (_, expectations) = xml
        .split_once("  <expectations>\n")
        .expect("independent expectation section");
    assert!(
        !expectations.contains("inline-boundary"),
        "transparent input boundaries must have no expectation nodes\n{xml}"
    );
    browser_parity_support::Golden::parse(&xml).expect("serialized explicit fixture must parse");
}

fn fri06_c08r_fixture_input_text(id: u64) -> Value {
    json!({
        "layoutInput": "inline-text",
        "inlineSegments": [{
            "id": id, "inlineExtent": 10, "inlineBaseline": 8, "inlineLineHeight": 10,
            "bidiLevel": 0, "whitespaceEdge": "preserve", "followingBreak": "prohibited"
        }],
        "rangeInks": [{
            "sourceSegmentId": id, "lineIndex": 0, "physicalStartEdge": "left",
            "start": id * 10, "advance": 10
        }],
        "children": []
    })
}

fn fri06_c08r_fixture_input_box(display: &str, children: Vec<Value>) -> Value {
    json!({
        "tagName": "div",
        "style": {"display": display},
        "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
        "children": children
    })
}

fn fri06_c08r_fixture_input_atomic(width: u64) -> Value {
    json!({
        "tagName": "span",
        "style": {
            "display": "inline-block",
            "size": {"width": {"unit": "px", "value": width}, "height": {"unit": "px", "value": 10}}
        },
        "atomicInlineParticipation": {"bidiLevel": 0, "followingBreak": "prohibited"},
        "unroundedLayout": {"x": 0, "y": 0, "width": width, "height": 10},
        "children": []
    })
}

fn fri06_c08r_fixture_input_root(display: &str, children: Vec<Value>) -> Value {
    json!({
        "tagName": "div",
        "layoutReadyInlineRoot": true,
        "useRounding": false,
        "viewport": {"width": {"unit": "px", "value": 100}, "height": {"unit": "max-content"}},
        "style": {"display": display, "size": {"width": {"unit": "px", "value": 100}}},
        "unroundedLayout": {"x": 0, "y": 0, "width": 100, "height": 20},
        "children": children
    })
}

fn fri06_c08r_fixture_input_boundary(kind: &str, metrics: Option<(f64, f64)>) -> Value {
    let inline_boundary = metrics.map_or_else(
            || json!({"kind": kind}),
            |(baseline, line_height)| {
                json!({"kind": kind, "baseline": baseline, "lineHeight": line_height})
            },
        );
    json!({
        "layoutInput": "inline-boundary",
        "inlineBoundary": inline_boundary,
        "children": []
    })
}

#[test]
fn fri06_c08r_fixture_input_all_five_adapter_families_serialize_then_parse() {
    let mut baseline_item =
        fri06_c08r_fixture_input_box("inline-grid", vec![fri06_c08r_fixture_input_text(0)]);
    baseline_item["layoutReadyAnonymousGridTextWrapper"] = Value::Bool(true);
    let baseline = fri06_c08r_fixture_input_root(
        "grid",
        vec![fri06_c08r_fixture_input_box(
            "grid",
            vec![baseline_item.clone(), baseline_item],
        )],
    );

    let mut four_run =
        fri06_c08r_fixture_input_box("grid", (0..4).map(fri06_c08r_fixture_input_text).collect());
    four_run["layoutReadyAnonymousGridTextWrapper"] = Value::Bool(true);
    let four_run = fri06_c08r_fixture_input_root(
        "block",
        vec![fri06_c08r_fixture_input_box("grid", vec![four_run])],
    );

    let bidi = fri06_c08r_fixture_input_root(
        "block",
        vec![
            fri06_c08r_fixture_input_boundary("start", None),
            fri06_c08r_fixture_input_text(0),
            fri06_c08r_fixture_input_boundary("end", None),
            fri06_c08r_fixture_input_text(1),
            fri06_c08r_fixture_input_boundary("start", None),
            fri06_c08r_fixture_input_text(2),
            fri06_c08r_fixture_input_boundary("end", None),
            fri06_c08r_fixture_input_text(3),
        ],
    );
    let mixed = fri06_c08r_fixture_input_root(
        "block",
        vec![
            fri06_c08r_fixture_input_text(0),
            fri06_c08r_fixture_input_atomic(18),
            fri06_c08r_fixture_input_boundary("start", Some((14.8, 20.0))),
            fri06_c08r_fixture_input_atomic(24),
        ],
    );
    let float = fri06_c08r_fixture_input_root(
        "block",
        vec![
            fri06_c08r_fixture_input_box("block", vec![]),
            fri06_c08r_fixture_input_box("block", vec![]),
            fri06_c08r_fixture_input_text(4),
            fri06_c08r_fixture_input_boundary("start", Some((12.0, 20.0))),
            fri06_c08r_fixture_input_atomic(28),
        ],
    );

    for (name, node, expected) in [
        (
            "subgrid_baseline_auto_columns_first_item__border_box_ltr",
            baseline,
            "layout-ready-anonymous-grid-text-wrapper=\"true\"",
        ),
        (
            "subgrid_auto_track_sizing_min_content_text_runs__border_box_ltr",
            four_run,
            "layout-ready-anonymous-grid-text-wrapper=\"true\"",
        ),
        (
            "fri06_bidi_mixed_inline__border_box_ltr",
            bidi,
            "<inline-boundary kind=\"end\"/>",
        ),
        (
            "fri06_inline_mixed_text_atomic_wrap__border_box_ltr",
            mixed,
            "inline-baseline=\"14.8\" inline-line-height=\"20\"",
        ),
        (
            "fri06_float_line_exclusion__border_box_ltr",
            float,
            "inline-baseline=\"12\" inline-line-height=\"20\"",
        ),
    ] {
        let xml = generate_xml(name, &node);
        assert!(xml.contains(expected), "{name} lacks {expected:?}\n{xml}");
        browser_parity_support::Golden::parse(&xml)
            .unwrap_or_else(|error| panic!("{name} explicit XML must parse: {error}\n{xml}"));
    }
}

#[test]
fn fri06_c08r_fixture_input_helper_projects_only_closed_marker_facts() {
    let script = [
            r#"
const window = {};
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const document = { styleSheets: [] };
function getComputedStyle(element) { return element.computedStyle; }
"#,
            TEST_HELPER_SOURCE,
            r#"
layoutReadyTextNodeData = function() {
  return { layoutInput: 'inline-text', inlineSegments: [{ id: 0 }], children: [] };
};
const text = { nodeType: Node.TEXT_NODE, textContent: 'alpha' };
const bdo = {
  tagName: 'BDO', childNodes: [text], computedStyle: { display: 'inline' },
  getAttribute(name) { return name === 'data-surgeist-transparent-inline-container' ? 'true' : null; },
};
const projection = layoutReadyTransparentInlineProjection(bdo);
if (projection.length !== 3 || projection[0].inlineBoundary.kind !== 'start' ||
    projection[1].layoutInput !== 'inline-text' || projection[2].inlineBoundary.kind !== 'end') {
  throw new Error(`invalid transparent projection ${JSON.stringify(projection)}`);
}
const atomic = { nodeType: Node.ELEMENT_NODE, tagName: 'SPAN', computedStyle: { display: 'inline-block' } };
const root = {
  getAttribute(name) {
    if (name === 'data-surgeist-layout-ready-inline') return 'true';
    if (name === 'data-surgeist-inline-struts') return '[{"beforeSourceIndex":0,"baseline":12,"lineHeight":20}]';
    return null;
  },
};
const strut = layoutReadyInlineStruts(root, [atomic]).get(0).inlineBoundary;
if (strut.kind !== 'start' || strut.baseline !== 12 || strut.lineHeight !== 20) {
  throw new Error(`invalid strut projection ${JSON.stringify(strut)}`);
}
const grid = {
  childNodes: [text],
  getAttribute(name) { return name === 'data-surgeist-anonymous-grid-text-wrapper' ? 'true' : null; },
};
if (layoutReadyAnonymousGridTextWrapper(
      grid,
      { display: 'grid' },
      [{ layoutInput: 'inline-text', inlineSegments: [{ id: 0 }], children: [] }]
    ) !== true) {
  throw new Error('anonymous wrapper marker did not project');
}
"#,
        ]
        .concat();
    run_bundled_helper_script("fri06-c08r-closed-marker-projection", script);
}

#[test]
fn fri06_c08r_fixture_input_helper_rejects_invalid_marker_roles_metrics_and_topology() {
    let script = [
            r#"
const window = {};
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const document = { styleSheets: [] };
function getComputedStyle(element) { return element.computedStyle; }
"#,
            TEST_HELPER_SOURCE,
            r#"
function mustThrow(label, callback) {
  try { callback(); } catch (_) { return; }
  throw new Error(`${label} did not fail`);
}
const text = { nodeType: Node.TEXT_NODE, textContent: 'alpha' };
mustThrow('transparent value', () => layoutReadyTransparentInlineProjection({
  tagName: 'BDO', childNodes: [text], computedStyle: { display: 'inline' },
  getAttribute() { return 'false'; },
}));
mustThrow('transparent topology', () => layoutReadyTransparentInlineProjection({
  tagName: 'BDO', childNodes: [text, text], computedStyle: { display: 'inline' },
  getAttribute() { return 'true'; },
}));
mustThrow('anonymous role', () => layoutReadyAnonymousGridTextWrapper(
  { childNodes: [text], getAttribute() { return 'true'; } },
  { display: 'block' },
  [{ layoutInput: 'inline-text', inlineSegments: [{ id: 0 }], children: [] }]
));
const atomic = { nodeType: Node.ELEMENT_NODE, tagName: 'SPAN', computedStyle: { display: 'inline-block' } };
function strutRoot(value) {
  return {
    getAttribute(name) {
      if (name === 'data-surgeist-layout-ready-inline') return 'true';
      if (name === 'data-surgeist-inline-struts') return value;
      return null;
    },
  };
}
mustThrow('strut extra field', () => layoutReadyInlineStruts(
  strutRoot('[{"beforeSourceIndex":0,"baseline":12,"lineHeight":20,"extra":1}]'), [atomic]
));
mustThrow('strut partial metrics', () => layoutReadyInlineStruts(
  strutRoot('[{"beforeSourceIndex":0,"baseline":12}]'), [atomic]
));
mustThrow('strut invalid metrics', () => layoutReadyInlineStruts(
  strutRoot('[{"beforeSourceIndex":0,"baseline":21,"lineHeight":20}]'), [atomic]
));
mustThrow('strut non-atomic target', () => layoutReadyInlineStruts(
  strutRoot('[{"beforeSourceIndex":0,"baseline":12,"lineHeight":20}]'), [text]
));
mustThrow('strut duplicate target', () => layoutReadyInlineStruts(
  strutRoot('[{"beforeSourceIndex":0,"baseline":12,"lineHeight":20},{"beforeSourceIndex":0,"baseline":12,"lineHeight":20}]'), [atomic]
));
"#,
        ]
        .concat();
    run_bundled_helper_script("fri06-c08r-invalid-marker-facts", script);
}

#[test]
fn fri06_c12_t07_direction_only_changes_do_not_choose_bidi_levels() {
    let script = [
        r#"
const window = {};
const CSSRule = { STYLE_RULE: 1 };
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const rootRect = { x: 0, y: 0, left: 0, top: 0, right: 100, bottom: 20, width: 100, height: 20 };
const textRect = { x: 0, y: 0, left: 0, top: 0, right: 10, bottom: 10, width: 10, height: 10 };
const range = {
  selectNodeContents() {},
  getBoundingClientRect() { return textRect; },
  getClientRects() { return [textRect]; },
  detach() {},
};
const document = { styleSheets: [], createRange() { return range; } };
const root = {
  parentElement: null,
  getAttribute(name) { return name === 'data-surgeist-layout-ready-inline' ? 'true' : null; },
  getBoundingClientRect() { return rootRect; },
};
const parent = { parentElement: root, childNodes: [], getAttribute() { return null; } };
const text = { nodeType: Node.TEXT_NODE, textContent: 'alpha', parentElement: parent };
parent.childNodes = [text];
let direction = 'ltr';
function getComputedStyle(element) {
  return {
    direction,
    writingMode: 'horizontal-tb',
    fontSize: '10px',
    lineHeight: '10px',
    display: element === root ? 'block' : 'inline',
  };
}
"#,
        TEST_HELPER_SOURCE,
        r#"
const levels = [];
for (direction of ['ltr', 'rtl']) {
  resetLayoutReadyRangeLineRegistry(root);
  levels.push(layoutReadyTextNodeData(text, parent, 0).inlineSegments[0].bidiLevel);
}
if (JSON.stringify(levels) !== JSON.stringify([0, 0])) {
  throw new Error(`direction-only mutation chose bidi input ${JSON.stringify(levels)}`);
}
"#,
    ]
    .concat();
    run_bundled_helper_script("fri06-c12-t07-direction-neutral-bidi", script);
}

#[test]
fn fri06_c12_t07_br_inline_metrics_use_browser_measured_baseline() {
    let script = [
            r#"
const window = {};
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
let baselineDistance = 15;
let appended = 0;
let removed = 0;
let lastProbe;
const verticalLrMarkerX = { lineOver: 30, baseline: 15, lineUnder: 0 };

function zeroRect(x, y) {
  return { x, y, left: x, right: x, top: y, bottom: y, width: 0, height: 0 };
}

function createElement(tagName) {
  return {
    tagName: tagName.toUpperCase(),
    style: {},
    children: [],
    append(...children) { this.children.push(...children); },
    getBoundingClientRect() {
      const lineOver = this.style.verticalAlign === 'top';
      const writingMode = lastProbe.style.writingMode;
      if (writingMode === 'horizontal-tb') {
        return zeroRect(100, lineOver ? 100 : 100 + baselineDistance);
      }
      if (writingMode === 'vertical-rl' || writingMode === 'sideways-rl') {
        return zeroRect(lineOver ? 100 : 100 - baselineDistance, 100);
      }
      if (writingMode === 'vertical-lr') {
        return zeroRect(lineOver ? verticalLrMarkerX.lineOver : verticalLrMarkerX.baseline, 100);
      }
      if (writingMode === 'sideways-lr') {
        return zeroRect(lineOver ? 100 : 100 + baselineDistance, 100);
      }
      throw new Error(`unexpected writing mode ${writingMode}`);
    },
    remove() {
      if (!this.removed) {
        this.removed = true;
        removed += 1;
      }
    },
  };
}

const document = {
  styleSheets: [],
  createElement,
  body: {
    appendChild(probe) {
      appended += 1;
      lastProbe = probe;
    },
  },
};
"#,
            TEST_HELPER_SOURCE,
            r#"
function metrics(writingMode, direction = 'ltr', lineHeight = '20px') {
  return brInlineMetricsForElement({ tagName: 'BR' }, {
    font: '16px "Measured Family"',
    fontSize: '16px',
    lineHeight,
    writingMode,
    direction,
  });
}

const horizontal = metrics('horizontal-tb');
if (horizontal.baseline !== '15px' || horizontal.lineHeight !== '20px') {
  throw new Error(`expected browser-measured 15/20 BR metrics, got ${JSON.stringify(horizontal)}`);
}
if (JSON.stringify(Object.keys(horizontal)) !== JSON.stringify(['baseline', 'lineHeight'])) {
  throw new Error(`BR helper fields changed: ${JSON.stringify(horizontal)}`);
}
if (lastProbe.style.font !== '16px "Measured Family"' ||
    lastProbe.style.lineHeight !== '20px' ||
    lastProbe.style.writingMode !== 'horizontal-tb' ||
    lastProbe.style.direction !== 'ltr') {
  throw new Error(`probe did not inherit the complete computed line context: ${JSON.stringify(lastProbe.style)}`);
}

for (const [writingMode, direction] of [
  ['vertical-rl', 'ltr'],
  ['sideways-rl', 'rtl'],
  ['sideways-lr', 'ltr'],
]) {
  const measured = metrics(writingMode, direction);
  if (measured.baseline !== '15px' || measured.lineHeight !== '20px') {
    throw new Error(`${writingMode}/${direction} did not use logical block distance: ${JSON.stringify(measured)}`);
  }
}

if (!(verticalLrMarkerX.lineOver > verticalLrMarkerX.baseline &&
      verticalLrMarkerX.baseline > verticalLrMarkerX.lineUnder)) {
  throw new Error(`vertical-lr fake does not reproduce Chrome marker orientation: ${JSON.stringify(verticalLrMarkerX)}`);
}
let verticalLr;
try {
  verticalLr = metrics('vertical-lr', 'ltr', '30px');
} catch (error) {
  throw new Error(`vertical-lr helper selected baseline - line-over = 15 - 30 = -15 and rejected it: ${String(error)}`);
}
if (verticalLr.baseline !== '15px' || verticalLr.lineHeight !== '30px') {
  throw new Error(`vertical-lr Chrome marker orientation did not produce 15/30 metrics: ${JSON.stringify(verticalLr)}`);
}

baselineDistance = 25;
const clamped = metrics('horizontal-tb');
if (clamped.baseline !== '20px') {
  throw new Error(`BR baseline was not clamped to finite line height: ${JSON.stringify(clamped)}`);
}

const beforeZero = appended;
const zero = metrics('horizontal-tb', 'ltr', '0px');
if (zero.baseline !== '0px' || zero.lineHeight !== '0px' || appended !== beforeZero) {
  throw new Error(`zero line height must remain exact without a probe: ${JSON.stringify(zero)}`);
}

function mustReject(label, callback) {
  let rejected = false;
  try { callback(); } catch (_) { rejected = true; }
  if (!rejected) throw new Error(`${label} measurement was not rejected`);
}

baselineDistance = 15;
mustReject('normal line height instead of synthetic 19.2px', () => metrics('horizontal-tb', 'ltr', 'normal'));
baselineDistance = Number.NaN;
mustReject('nonfinite marker', () => metrics('horizontal-tb'));
baselineDistance = -1;
mustReject('negative logical marker distance', () => metrics('horizontal-tb'));
mustReject('nonfinite line height', () => metrics('horizontal-tb', 'ltr', 'Infinitypx'));
if (appended !== removed) {
  throw new Error(`BR probes leaked: appended ${appended}, removed ${removed}`);
}
"#,
        ]
        .concat();

    run_bundled_helper_script("fri06-c12-t07-browser-measured-br-baseline", script);
}

#[test]
fn fri06_c12_t07_bidi_marker_is_closed_targeted_unique_and_consumed() {
    let script = [
            r#"
const window = {};
const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
const document = { styleSheets: [] };
function getComputedStyle(element) { return element.computedStyle || { display: 'block' }; }
const text = { nodeType: Node.TEXT_NODE, textContent: 'alpha' };
const atomic = {
  nodeType: Node.ELEMENT_NODE,
  tagName: 'SPAN',
  computedStyle: { display: 'inline-block' },
};
const br = { nodeType: Node.ELEMENT_NODE, tagName: 'BR', computedStyle: { display: 'inline' } };
function parent(value, children = [text]) {
  return {
    childNodes: children,
    computedStyle: { display: 'block', direction: 'ltr' },
    getAttribute(name) { return name === 'data-surgeist-inline-bidi-levels' ? value : null; },
  };
}
"#,
            TEST_HELPER_SOURCE,
            r#"
function mustReject(label, callback, expected) {
  let error;
  try { callback(); } catch (caught) { error = String(caught); }
  if (!error || !error.includes(expected)) {
    throw new Error(`${label} did not reject with ${expected}: ${error}`);
  }
}

const valid = layoutReadyInlineBidiLevels(
  parent('[{"sourceIndex":0,"bidiLevel":1}]'), [text]
);
if (consumeLayoutReadyInlineBidiLevel(valid, 0) !== 1 || valid.size !== 0) {
  throw new Error('valid source-indexed bidi marker was not consumed exactly once');
}
rejectUnusedLayoutReadyInlineBidiLevels(valid);

const matchingParent = parent('[{"sourceIndex":0,"bidiLevel":1,"whenDirection":"rtl"}]');
matchingParent.computedStyle.direction = 'rtl';
const matching = layoutReadyInlineBidiLevels(matchingParent, [text]);
if (consumeLayoutReadyInlineBidiLevel(matching, 0) !== 1 || matching.size !== 0) {
  throw new Error('matching scoped bidi marker did not supply its authored level exactly once');
}
rejectUnusedLayoutReadyInlineBidiLevels(matching);

const inactiveParent = parent('[{"sourceIndex":0,"bidiLevel":1,"whenDirection":"rtl"}]');
const inactive = layoutReadyInlineBidiLevels(inactiveParent, [text]);
if (inactive.size !== 1 || consumeLayoutReadyInlineBidiLevel(inactive, 0) !== 0 || inactive.size !== 0) {
  throw new Error('inactive scoped bidi marker was not accounted without supplying a level');
}
rejectUnusedLayoutReadyInlineBidiLevels(inactive);

for (const [label, value, children, expected] of [
  ['syntax', '{', [text], 'valid JSON'],
  ['empty', '[]', [text], 'nonempty finite table'],
  ['extra field', '[{"sourceIndex":0,"bidiLevel":1,"extra":true}]', [text], 'closed fields'],
  ['missing field', '[{"sourceIndex":0}]', [text], 'closed fields'],
  ['missing level with direction', '[{"sourceIndex":0,"whenDirection":"rtl"}]', [text], 'closed fields'],
  ['invalid direction', '[{"sourceIndex":0,"bidiLevel":1,"whenDirection":"RTL"}]', [text], 'direction ltr or rtl'],
  ['non-string direction', '[{"sourceIndex":0,"bidiLevel":1,"whenDirection":1}]', [text], 'direction ltr or rtl'],
  ['negative source', '[{"sourceIndex":-1,"bidiLevel":1}]', [text], 'existing non-negative sourceIndex'],
  ['missing target', '[{"sourceIndex":1,"bidiLevel":1}]', [text], 'existing non-negative sourceIndex'],
  ['duplicate target', '[{"sourceIndex":0,"bidiLevel":1},{"sourceIndex":0,"bidiLevel":2}]', [text], 'duplicates sourceIndex'],
  ['duplicate scoped target', '[{"sourceIndex":0,"bidiLevel":1,"whenDirection":"ltr"},{"sourceIndex":0,"bidiLevel":2,"whenDirection":"rtl"}]', [text], 'duplicates sourceIndex'],
  ['zero level', '[{"sourceIndex":0,"bidiLevel":0}]', [text], 'integer in 1..=125'],
  ['high level', '[{"sourceIndex":0,"bidiLevel":126}]', [text], 'integer in 1..=125'],
  ['fractional level', '[{"sourceIndex":0,"bidiLevel":1.5}]', [text], 'integer in 1..=125'],
  ['BR target', '[{"sourceIndex":0,"bidiLevel":1}]', [br], 'shaped text or an atomic inline'],
  ['inactive BR target', '[{"sourceIndex":0,"bidiLevel":1,"whenDirection":"rtl"}]', [br], 'shaped text or an atomic inline'],
  ['ordinary box target', '[{"sourceIndex":0,"bidiLevel":1}]', [{...atomic, computedStyle: {display: 'block'}}], 'shaped text or an atomic inline'],
]) {
  mustReject(label, () => layoutReadyInlineBidiLevels(parent(value, children), children), expected);
}

const unused = layoutReadyInlineBidiLevels(
  parent('[{"sourceIndex":0,"bidiLevel":1}]', [atomic]), [atomic]
);
mustReject(
  'unused record',
  () => rejectUnusedLayoutReadyInlineBidiLevels(unused),
  'unused sourceIndex 0'
);

const inactiveUnusedParent = parent(
  '[{"sourceIndex":0,"bidiLevel":1,"whenDirection":"rtl"}]', [atomic]
);
const inactiveUnused = layoutReadyInlineBidiLevels(inactiveUnusedParent, [atomic]);
rejectUnusedLayoutReadyInlineBidiLevels(inactiveUnused);

const atomicChildren = [text, atomic, { ...text }];
const atomicMarker = '[{"sourceIndex":0,"bidiLevel":1,"whenDirection":"rtl"},{"sourceIndex":1,"bidiLevel":1,"whenDirection":"rtl"},{"sourceIndex":2,"bidiLevel":1,"whenDirection":"rtl"}]';
const ltrAtomicParent = parent(atomicMarker, atomicChildren);
const ltrAtomic = layoutReadyInlineBidiLevels(ltrAtomicParent, atomicChildren);
const ltrLevels = atomicChildren.map((_, sourceIndex) =>
  consumeLayoutReadyInlineBidiLevel(ltrAtomic, sourceIndex)
);
if (JSON.stringify(ltrLevels) !== JSON.stringify([0, 0, 0]) || ltrAtomic.size !== 0) {
  throw new Error(`inactive atomic RTL records produced ${JSON.stringify(ltrLevels)}`);
}
rejectUnusedLayoutReadyInlineBidiLevels(ltrAtomic);

const rtlAtomicParent = parent(atomicMarker, atomicChildren);
rtlAtomicParent.computedStyle.direction = 'rtl';
const rtlAtomic = layoutReadyInlineBidiLevels(rtlAtomicParent, atomicChildren);
const rtlLevels = atomicChildren.map((_, sourceIndex) =>
  consumeLayoutReadyInlineBidiLevel(rtlAtomic, sourceIndex)
);
if (JSON.stringify(rtlLevels) !== JSON.stringify([1, 1, 1]) || rtlAtomic.size !== 0) {
  throw new Error(`active atomic RTL records produced ${JSON.stringify(rtlLevels)}`);
}
rejectUnusedLayoutReadyInlineBidiLevels(rtlAtomic);
"#,
        ]
        .concat();
    run_bundled_helper_script("fri06-c12-t07-bidi-marker-validation", script);

    let bidi = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/layout/browser_parity/html/block/fri06_bidi_mixed_inline.html"),
    )
    .expect("bidi source");
    assert_eq!(bidi.matches("data-surgeist-inline-bidi-levels=").count(), 1);
    assert!(
        bidi.contains(r#"data-surgeist-inline-bidi-levels='[{"sourceIndex":0,"bidiLevel":1}]'"#)
    );
}
