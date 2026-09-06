//! Execute the bundled producer while isolating DOM geometry measurement.

use crate::adapter::TEST_HELPER_SOURCE;
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

fn helper_response(has_root: bool) -> Value {
    // `describeElement` owns geometry and style capture. This recorder leaves
    // the real `getTestData` in charge of root discovery, body mutation, variant
    // order, and JSON transport, which are the protocol contracts under test.
    let script = r#"
const vm = require('node:vm');
const fs = require('node:fs');
const input = JSON.parse(fs.readFileSync(0, 'utf8'));
const root = {};
const measurements = [];
const bodyClasses = [];
const rootQueries = [];
let className = '';
const context = vm.createContext({
  window: {},
  document: {
    getElementById(id) {
      rootQueries.push(id);
      return input.hasRoot ? root : null;
    },
    body: {
      get className() { return className; },
      set className(value) { className = value; bodyClasses.push(value); },
    },
  },
});
vm.runInContext(input.helperSource, context);
context.describeElement = element => {
  if (element !== root) throw new Error('measured a different root');
  measurements.push(className);
  return { observedClass: className };
};
const response = vm.runInContext('getTestData()', context);
process.stdout.write(JSON.stringify({ response, measurements, bodyClasses, rootQueries }));
"#;
    run_node(
        script,
        json!({ "helperSource": TEST_HELPER_SOURCE, "hasRoot": has_root }),
    )
}

fn run_node(script: &str, input: Value) -> Value {
    let mut child = Command::new("node")
        .args(["-e", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node must execute the bundled measurement helper");
    child
        .stdin
        .take()
        .expect("piped Node input")
        .write_all(
            serde_json::to_string(&input)
                .expect("helper input is JSON")
                .as_bytes(),
        )
        .expect("write the captured helper source to Node");
    let output = child
        .wait_with_output()
        .expect("wait for the helper result");
    assert!(
        output.status.success(),
        "helper failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    serde_json::from_slice(&output.stdout).expect("helper harness returns JSON")
}

fn decode_envelope(result: &Value) -> Value {
    let response = result["response"]
        .as_str()
        .expect("getTestData retains JSON-string transport");
    let envelope: Value =
        serde_json::from_str(response).expect("measurement envelope is valid JSON");
    assert_eq!(envelope["schemaVersion"].as_u64(), Some(1));
    assert_eq!(
        envelope
            .as_object()
            .expect("measurement envelope is an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "borderBoxLtrData",
            "borderBoxRtlData",
            "contentBoxLtrData",
            "contentBoxRtlData",
            "schemaVersion",
        ],
    );
    envelope
}

#[test]
fn present_root_returns_version_one_with_four_ordered_measurements() {
    let result = helper_response(true);
    let envelope = decode_envelope(&result);
    let expected_classes = json!([
        "border-box ltr",
        "content-box ltr",
        "border-box rtl",
        "content-box rtl",
    ]);
    assert_eq!(result["rootQueries"], json!(["test-root"]));
    assert_eq!(result["bodyClasses"], expected_classes);
    assert_eq!(result["measurements"], expected_classes);
    for (key, class_name) in [
        ("borderBoxLtrData", "border-box ltr"),
        ("contentBoxLtrData", "content-box ltr"),
        ("borderBoxRtlData", "border-box rtl"),
        ("contentBoxRtlData", "content-box rtl"),
    ] {
        assert_eq!(envelope[key], json!({ "observedClass": class_name }));
    }
}

#[test]
fn missing_root_returns_version_one_with_four_unsupported_variants() {
    let result = helper_response(false);
    let envelope = decode_envelope(&result);
    assert_eq!(result["rootQueries"], json!(["test-root"]));
    assert_eq!(result["bodyClasses"], json!([]));
    assert_eq!(result["measurements"], json!([]));
    for key in [
        "borderBoxLtrData",
        "contentBoxLtrData",
        "borderBoxRtlData",
        "contentBoxRtlData",
    ] {
        assert_eq!(
            envelope[key],
            json!({ "layoutInput": "unsupported", "unsupportedReason": "Unsupported missing #test-root fixture root" }),
        );
    }
}

fn helper_probe(probe: &str) -> Value {
    run_node(
        r#"
const vm = require('node:vm');
const fs = require('node:fs');
const input = JSON.parse(fs.readFileSync(0, 'utf8'));
const context = vm.createContext({ window: {} });
vm.runInContext(input.helperSource, context);
process.stdout.write(JSON.stringify(vm.runInContext(input.probe, context)));
"#,
        json!({ "helperSource": TEST_HELPER_SOURCE, "probe": probe }),
    )
}

#[test]
fn br_metrics_are_numeric_and_zero_height_is_preserved() {
    let result = helper_probe("brInlineMetricsForElement({tagName: 'BR'}, {lineHeight: '0px'})");
    assert_eq!(result, json!({ "baseline": 0, "lineHeight": 0 }));
}

#[test]
fn shape_measurements_use_an_optional_interval_object() {
    let result = helper_probe(
        r#"layoutReadyShapeBands({ getAttribute: () => JSON.stringify([
          {bandMinimum: 0, bandMaximum: 10, intervalMinimum: 2, intervalMaximum: 8},
          {bandMinimum: 10, bandMaximum: 20}
        ]) })"#,
    );
    assert_eq!(
        result,
        json!([
            { "bandMinimum": 0, "bandMaximum": 10, "interval": { "minimum": 2, "maximum": 8 } },
            { "bandMinimum": 10, "bandMaximum": 20 },
        ])
    );
}

#[test]
fn named_grid_placements_omit_absent_occurrences() {
    let result = helper_probe(
        "['header', 'header -2', 'span header', 'span 3 header'].map(parseGridPosition)",
    );
    assert_eq!(
        result,
        json!([
            { "kind": "named-line", "name": "header" },
            { "kind": "named-line", "name": "header", "occurrence": -2 },
            { "kind": "named-span", "name": "header" },
            { "kind": "named-span", "name": "header", "occurrence": 3 },
        ])
    );
}
