//! Layout fixture semantics supplied to the generic browser engine.

use crate::xml::{escape_attr, generate_xml};
use serde_json::Value;

#[cfg(test)]
pub(super) const TEST_HELPER_SOURCE: &str =
    include_str!("../../layout/browser_parity/scripts/gentest/test_helper.js");
#[cfg(test)]
pub(super) const TEST_BASE_STYLE_SOURCE: &str =
    include_str!("../../layout/browser_parity/scripts/gentest/test_base_style.css");
pub(super) const GRID_TEMPLATE_AREA_CAPTURE_SCRIPT: &str = r#"(() => {
  if (window.__surgeistGridTemplateAreaCaptureInstalled) return true;

  function parseSurgeistGridTemplateAreas(input) {
    if (!input || input === "none") return undefined;
    const rows = Array.from(input.matchAll(/"([^"]*)"/g), match => match[1].trim());
    if (rows.length === 0) return undefined;
    return rows.map(row => row.split(/\s+/).map(cell => /^\.+$/.test(cell) ? null : cell));
  }

  function authoredSurgeistGridTemplateAreas(element) {
    const computedStyle = getComputedStyle(element);
    if (element.style.gridTemplateAreas) return element.style.gridTemplateAreas;
    if (typeof authoredStyleValue === "function") {
      const authored = authoredStyleValue(element, "gridTemplateAreas", computedStyle);
      if (authored) return authored;
      if (computedStyle.gridTemplateAreas && computedStyle.gridTemplateAreas !== "none") {
        return computedStyle.gridTemplateAreas;
      }
    }
    return "";
  }

  const originalDescribeElement = describeElement;
  describeElement = function(element, expectedElement = null) {
    const data = originalDescribeElement(element, expectedElement);
    if (data && data.style) {
      data.style.gridTemplateAreas = parseSurgeistGridTemplateAreas(
        authoredSurgeistGridTemplateAreas(element)
      );
    }
    return data;
  };

  window.__surgeistGridTemplateAreaCaptureInstalled = true;
  return true;
})()"#;

#[cfg(test)]
pub(super) fn browser_fixture_document(raw: &str, base_url: &str) -> Result<String, String> {
    fixture_document(raw, base_url, TEST_BASE_STYLE_SOURCE)
}

fn fixture_document(raw: &str, base_url: &str, base_style: &str) -> Result<String, String> {
    let mut head_injection = format!("<base href=\"{}\">", escape_attr(base_url));
    if raw.contains("test_base_style.css") {
        head_injection.push_str("<style>");
        head_injection.push_str(base_style);
        head_injection.push_str("</style>");
    }

    let lower = raw.to_ascii_lowercase();
    if let Some(index) = lower.find("<head>") {
        let insert_at = index + "<head>".len();
        let mut html = String::with_capacity(raw.len() + head_injection.len());
        html.push_str(&raw[..insert_at]);
        html.push_str(&head_injection);
        html.push_str(&raw[insert_at..]);
        Ok(html)
    } else if let Some(index) = lower.find("<html") {
        let html_tag_end = raw[index..]
            .find('>')
            .ok_or_else(|| "fixture html tag is missing closing `>`".to_string())?
            + index
            + 1;
        Ok(format!(
            "{}<head>{head_injection}</head>{}",
            &raw[..html_tag_end],
            &raw[html_tag_end..]
        ))
    } else if lower.starts_with("<!doctype") {
        let doctype_end = raw
            .find('>')
            .ok_or_else(|| "fixture doctype is missing closing `>`".to_string())?
            + 1;
        Ok(format!(
            "{}<html><head>{head_injection}</head><body>{}</body></html>",
            &raw[..doctype_end],
            &raw[doctype_end..]
        ))
    } else {
        Ok(format!(
            "<html><head>{head_injection}</head><body>{raw}</body></html>"
        ))
    }
}

#[cfg(test)]
pub(super) fn browser_document_write_script(html: &str) -> String {
    let html = serde_json::to_string(html).expect("serializing HTML should not fail");
    format!(
        "(() => {{ document.open(); document.write({html}); document.close(); return true; }})()"
    )
}

pub(super) fn fixture_cases() -> [(&'static str, &'static str); 4] {
    [
        ("border_box_ltr", "borderBoxLtrData"),
        ("content_box_ltr", "contentBoxLtrData"),
        ("border_box_rtl", "borderBoxRtlData"),
        ("content_box_rtl", "contentBoxRtlData"),
    ]
}

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Component, Path};

use surgeist_generator::RelativePath;
use surgeist_generator::browser::{
    BrowserCorpusAdapter, CaseOutcome, FixtureInput, FixtureSpec, PreparedFixture,
    ResourceDependencies,
};

pub(super) struct LayoutAdapter;

#[derive(Debug)]
pub(super) enum AdapterError {
    Preparation(String),
    Transport {
        fixture: String,
    },
    Envelope {
        fixture: String,
        source: crate::envelope::EnvelopeError,
    },
    Declaration {
        fixture: String,
        detail: String,
    },
    Measurement(Box<crate::measurement::MeasurementError>),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preparation(message) => formatter.write_str(message),
            Self::Transport { fixture } => write!(
                formatter,
                "{fixture}: measurement helper must return a JSON string"
            ),
            Self::Envelope { fixture, source } => write!(formatter, "{fixture}: {source}"),
            Self::Declaration { fixture, detail } => {
                write!(formatter, "{fixture}: invalid measurement cases: {detail}")
            }
            Self::Measurement(source) => source.fmt(formatter),
        }
    }
}

impl std::error::Error for AdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Envelope { source, .. } => Some(source),
            Self::Measurement(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl BrowserCorpusAdapter for LayoutAdapter {
    type Error = AdapterError;

    fn prepare(&self, input: FixtureInput<'_>) -> Result<PreparedFixture, Self::Error> {
        let source = std::str::from_utf8(input.source_bytes()).map_err(|error| {
            AdapterError::Preparation(format!("fixture source is not UTF-8: {error}"))
        })?;
        let helper_path = RelativePath::new("scripts/gentest/test_helper.js")
            .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        let style_path = RelativePath::new("scripts/gentest/test_base_style.css")
            .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        let helper = input.input_bytes(&helper_path).ok_or_else(|| {
            AdapterError::Preparation("captured measurement helper is missing".to_string())
        })?;
        let style = input.input_bytes(&style_path).ok_or_else(|| {
            AdapterError::Preparation("captured base stylesheet is missing".to_string())
        })?;
        let helper = std::str::from_utf8(helper).map_err(|error| {
            AdapterError::Preparation(format!("measurement helper is not UTF-8: {error}"))
        })?;
        let style = std::str::from_utf8(style).map_err(|error| {
            AdapterError::Preparation(format!("base stylesheet is not UTF-8: {error}"))
        })?;
        let document =
            fixture_document(source, input.base_url(), style).map_err(AdapterError::Preparation)?;
        let resources = match linked_resources(input.fixture().source(), source) {
            Ok(paths) => ResourceDependencies::Paths(paths),
            Err(reason) => ResourceDependencies::Invalid { reason },
        };
        let setup = helper_setup_expression(helper)?;
        PreparedFixture::new(
            document,
            "document.readyState !== 'loading' && !!document.body".to_string(),
            setup,
            "getTestData()".to_string(),
            resources,
        )
        .map_err(|error| AdapterError::Preparation(error.to_string()))
    }

    fn lower(
        &self,
        fixture: &FixtureSpec,
        measurement: Value,
    ) -> Result<Vec<CaseOutcome>, Self::Error> {
        let raw = measurement
            .as_str()
            .ok_or_else(|| AdapterError::Transport {
                fixture: fixture.name().to_string(),
            })?;
        let measurements =
            crate::envelope::decode(raw).map_err(|source| AdapterError::Envelope {
                fixture: fixture.name().to_string(),
                source,
            })?;
        if fixture.cases().len() != fixture_cases().len() {
            return Err(AdapterError::Declaration {
                fixture: fixture.name().to_string(),
                detail: format!("expected four variants, received {}", fixture.cases().len()),
            });
        }
        fixture_cases()
            .into_iter()
            .zip(fixture.cases())
            .zip(measurements)
            .enumerate()
            .map(|(index, (((variant, _), case), raw))| {
                if case.variant() != variant {
                    return Err(AdapterError::Declaration {
                        fixture: fixture.name().to_string(),
                        detail: format!(
                            "cases[{index}].variant must be {variant}, received {}",
                            case.variant()
                        ),
                    });
                }
                let measurement = crate::measurement::decode(raw.get(), case.id(), variant)
                    .map_err(|source| AdapterError::Measurement(Box::new(source)))?;
                Ok(match measurement {
                    crate::measurement::DecodedMeasurement::Unsupported { reason } => {
                        CaseOutcome::Unsupported {
                            case_id: case.id().to_string(),
                            reason,
                        }
                    }
                    crate::measurement::DecodedMeasurement::Supported(measurement) => {
                        CaseOutcome::Generated {
                            case_id: case.id().to_string(),
                            bytes: generate_xml(case.id(), &measurement).into_bytes(),
                        }
                    }
                })
            })
            .collect()
    }
}

fn linked_resources(source: &RelativePath, raw: &str) -> Result<Vec<RelativePath>, String> {
    let mut references = BTreeSet::new();
    for marker in ["src=\"", "src='", "href=\"", "href='"] {
        let quote = marker
            .chars()
            .last()
            .expect("resource marker contains a quote");
        for (start, _) in raw.match_indices(marker) {
            let value = &raw[start + marker.len()..];
            let end = value.find(quote).ok_or_else(|| {
                format!(
                    "{} has an unterminated linked-resource attribute",
                    source.as_str()
                )
            })?;
            let reference = value[..end].split(['?', '#']).next().unwrap_or_default();
            if reference.is_empty()
                || reference.starts_with('/')
                || reference.contains("://")
                || reference.starts_with("data:")
            {
                continue;
            }
            let path = resolve_reference(source, reference)?;
            if matches!(
                path.as_str(),
                "scripts/gentest/test_helper.js" | "scripts/gentest/test_base_style.css"
            ) {
                continue;
            }
            references.insert(path);
        }
    }
    Ok(references.into_iter().collect())
}

fn resolve_reference(source: &RelativePath, reference: &str) -> Result<RelativePath, String> {
    let mut resolved = Path::new(source.as_str())
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components()
        .filter_map(|component| match component {
            Component::Normal(segment) => Some(segment.to_os_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for component in Path::new(reference).components() {
        match component {
            Component::Normal(segment) => resolved.push(segment.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                if resolved.pop().is_none() {
                    return Err(format!(
                        "linked resource {reference:?} escapes the corpus root"
                    ));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "linked resource {reference:?} must be corpus-relative"
                ));
            }
        }
    }
    let path = resolved.into_iter().collect::<std::path::PathBuf>();
    RelativePath::new(path.to_string_lossy().replace('\\', "/")).map_err(|error| error.to_string())
}

fn helper_setup_expression(helper: &str) -> Result<String, AdapterError> {
    let helper = serde_json::to_string(helper).map_err(|error| {
        AdapterError::Preparation(format!("failed to encode helper script: {error}"))
    })?;
    Ok(format!(
        "if (typeof getTestData !== 'function') {{ (0, eval)({helper}); }}\n{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_generator::browser::{CaseSpec, FixtureStatus};

    fn fixture() -> FixtureSpec {
        FixtureSpec::new(
            "example".to_string(),
            RelativePath::new("html/grid/example.html").unwrap(),
            fixture_cases()
                .into_iter()
                .map(|(variant, _)| {
                    CaseSpec::new(
                        format!("example__{variant}"),
                        variant.to_string(),
                        RelativePath::new(format!("xml/grid/example__{variant}.xml")).unwrap(),
                    )
                    .unwrap()
                })
                .collect(),
            FixtureStatus::Active,
        )
        .unwrap()
    }

    fn unsupported(reason: &str) -> Value {
        serde_json::json!({"layoutInput": "unsupported", "unsupportedReason": reason})
    }

    fn supported_box() -> Value {
        let geometry = serde_json::json!({
            "x": 0, "y": 0, "width": 10, "height": 20,
            "scrollWidth": 10, "scrollHeight": 20,
            "clientWidth": 10, "clientHeight": 20,
        });
        serde_json::json!({
            "layoutInput": "box", "tagName": "div", "useRounding": true,
            "viewport": {"width": {"unit": "px", "value": 100}, "height": {"unit": "px", "value": 100}, "rootContext": "root"},
            "style": {"display": "block", "boxSizing": "border-box", "writingMode": "horizontal-tb", "direction": "ltr"},
            "unroundedLayout": geometry.clone(),
            "smartRoundedLayout": geometry.clone(),
            "naivelyRoundedLayout": geometry,
            "children": [],
        })
    }

    fn envelope(nodes: [Value; 4]) -> Value {
        let mut fields = serde_json::Map::new();
        fields.insert("schemaVersion".to_string(), Value::from(1));
        for ((_, key), node) in fixture_cases().into_iter().zip(nodes) {
            fields.insert(key.to_string(), node);
        }
        Value::String(Value::Object(fields).to_string())
    }

    #[test]
    fn mixed_measurements_keep_declared_case_order_and_observed_geometry() {
        let outcomes = LayoutAdapter
            .lower(
                &fixture(),
                envelope([
                    supported_box(),
                    unsupported("second"),
                    supported_box(),
                    unsupported("fourth"),
                ]),
            )
            .unwrap();
        for (index, outcome) in outcomes.into_iter().enumerate() {
            match outcome {
                CaseOutcome::Generated { case_id, bytes } => {
                    assert!(matches!(index, 0 | 2));
                    assert_eq!(case_id, fixture().cases()[index].id());
                    let xml = std::str::from_utf8(&bytes).unwrap();
                    let document = roxmltree::Document::parse(xml).unwrap();
                    let expected = document
                        .descendants()
                        .find(|node| node.has_tag_name("expectations"))
                        .unwrap();
                    let node = expected.first_element_child().unwrap();
                    assert_eq!(node.attribute("width"), Some("10"));
                    assert_eq!(node.attribute("height"), Some("20"));
                }
                CaseOutcome::Unsupported { case_id, reason } => {
                    assert!(matches!(index, 1 | 3));
                    assert_eq!(case_id, fixture().cases()[index].id());
                    assert_eq!(reason, if index == 1 { "second" } else { "fourth" });
                }
            }
        }
    }

    #[test]
    fn malformed_measurement_keeps_case_and_field_context_in_adapter_error() {
        let mut malformed = supported_box();
        malformed["smartRoundedLayout"]["width"] = Value::String("not a number".to_string());
        let error = LayoutAdapter
            .lower(
                &fixture(),
                envelope([
                    unsupported("first"),
                    malformed,
                    supported_box(),
                    unsupported("last"),
                ]),
            )
            .unwrap_err();
        let AdapterError::Measurement(source) = &error else {
            panic!("expected measurement error: {error}");
        };
        assert_eq!(source.case_id, "example__content_box_ltr");
        assert_eq!(source.variant, "content_box_ltr");
        assert!(source.field_path.contains("smartRoundedLayout.width"));
        assert_eq!(
            source.kind,
            crate::measurement::MeasurementErrorKind::Decode
        );
        assert!(
            std::error::Error::source(&error)
                .and_then(std::error::Error::source)
                .is_some()
        );
    }

    #[test]
    fn nested_duplicate_fields_are_rejected_before_unsupported_classification() {
        let payload = envelope(std::array::from_fn(|_| unsupported("first")));
        let raw = payload.as_str().unwrap().replacen(
            "\"unsupportedReason\":\"first\"",
            "\"unsupportedReason\":\"first\",\"unsupportedReason\":\"second\"",
            1,
        );
        let error = LayoutAdapter
            .lower(&fixture(), Value::String(raw))
            .unwrap_err();
        let AdapterError::Measurement(source) = &error else {
            panic!("expected measurement error: {error}");
        };
        assert_eq!(
            source.kind,
            crate::measurement::MeasurementErrorKind::Decode
        );
        assert!(std::error::Error::source(source.as_ref()).is_some());
    }

    #[test]
    fn preloaded_unversioned_helper_is_preserved_and_rejected_at_ingress() {
        let Value::String(versioned) = envelope(std::array::from_fn(|_| unsupported("old helper")))
        else {
            unreachable!()
        };
        let mut legacy: Value = serde_json::from_str(&versioned).unwrap();
        legacy.as_object_mut().unwrap().remove("schemaVersion");
        let original = format!(
            "function getTestData() {{ return {}; }} function describeElement() {{ return {{ style: {{}} }}; }}",
            serde_json::to_string(&legacy.to_string()).unwrap(),
        );
        let original = helper_setup_expression(&original).unwrap();
        let replacement = helper_setup_expression(TEST_HELPER_SOURCE).unwrap();
        let script = format!(
            "global.window={{}}; const vm=require('node:vm'); vm.runInThisContext({}); const before=getTestData; vm.runInThisContext({}); if(getTestData!==before) throw Error('preloaded helper replaced'); process.stdout.write(getTestData());",
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&replacement).unwrap(),
        );
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(script)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let error = LayoutAdapter
            .lower(
                &fixture(),
                Value::String(String::from_utf8(output.stdout).unwrap()),
            )
            .unwrap_err();
        let AdapterError::Envelope { source, .. } = error else {
            panic!("expected envelope error");
        };
        assert_eq!(
            source.kind(),
            crate::envelope::EnvelopeErrorKind::MissingVersion
        );
    }

    #[test]
    fn adapter_requires_a_closed_version_one_measurement_envelope() {
        let variants = fixture_cases()
            .into_iter()
            .map(|(_, key)| format!("\"{key}\":{{\"layoutInput\":\"unsupported\",\"unsupportedReason\":\"known unsupported\"}}"))
            .collect::<Vec<_>>()
            .join(",");
        for extra in [
            "",
            "\"schemaVersion\":2,",
            "\"schemaVersion\":1.0,",
            "\"schemaVersion\":null,",
            "\"schemaVersion\":1,\"unexpected\":true,",
            "\"schemaVersion\":1,\"schemaVersion\":1,",
        ] {
            let payload = format!("{{{extra}{variants}}}");
            assert!(
                LayoutAdapter
                    .lower(&fixture(), Value::String(payload))
                    .is_err(),
                "invalid envelope prefix was accepted: {extra:?}"
            );
        }
    }

    #[test]
    fn measurement_protocol_rejects_an_object_in_place_of_the_helpers_json_string() {
        let error = LayoutAdapter
            .lower(&fixture(), serde_json::json!({}))
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "example: measurement helper must return a JSON string"
        );
    }

    #[test]
    fn missing_variant_is_a_fixture_error_before_any_outcomes_are_published() {
        let error = LayoutAdapter
            .lower(
                &fixture(),
                Value::String("{\"schemaVersion\":1}".to_string()),
            )
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "example: measurement envelope borderBoxLtrData: declared variant is required"
        );
    }

    #[test]
    fn unsupported_variants_retain_order_and_first_descendant_reason() {
        let mut description = fixture_cases()
            .into_iter()
            .map(|(_, key)| {
                (
                    key.to_string(),
                    serde_json::json!({"layoutInput": "box", "children": [
                        {"layoutInput": "box", "children": [{"layoutInput": "unsupported", "unsupportedReason": "first"}]},
                        {"layoutInput": "unsupported", "unsupportedReason": "second"}
                    ]}),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        description.insert("schemaVersion".to_string(), Value::from(1));
        let outcomes = LayoutAdapter
            .lower(
                &fixture(),
                Value::String(Value::Object(description).to_string()),
            )
            .unwrap();
        let identities = outcomes
            .iter()
            .map(|outcome| match outcome {
                CaseOutcome::Unsupported { case_id, reason } => {
                    assert_eq!(reason, "first");
                    case_id.as_str()
                }
                CaseOutcome::Generated { .. } => {
                    panic!("unsupported measurement produced an artifact")
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            identities,
            [
                "example__border_box_ltr",
                "example__content_box_ltr",
                "example__border_box_rtl",
                "example__content_box_rtl"
            ]
        );
    }

    #[test]
    fn linked_resource_discovery_retains_literal_reference_rules() {
        let paths = linked_resources(
            &RelativePath::new("html/grid/example.html").unwrap(),
            r#"
<link href="../../assets/a.css?version=1#fragment"><img src='../../assets/a.css'>
<script src="../../scripts/gentest/test_helper.js"></script>
<link href="../../scripts/gentest/test_base_style.css"><img src="/absolute.png">
<img src="https://example.invalid/external.png"><img src="data:image/png;base64,abc">
<img src="../shared/z.png">
"#,
        )
        .unwrap();
        assert_eq!(
            paths.iter().map(RelativePath::as_str).collect::<Vec<_>>(),
            ["assets/a.css", "html/shared/z.png"]
        );
        assert!(
            linked_resources(
                &RelativePath::new("html/example.html").unwrap(),
                "<img src='../../../outside'>"
            )
            .is_err()
        );
        assert!(
            linked_resources(
                &RelativePath::new("html/example.html").unwrap(),
                "<img src='unterminated>"
            )
            .is_err()
        );
    }

    #[test]
    fn setup_injects_a_missing_helper_and_preserves_an_existing_helper() {
        let first = helper_setup_expression("function getTestData() { return 'original'; } function describeElement() { return { style: {} }; }").unwrap();
        let second =
            helper_setup_expression("function getTestData() { return 'replacement'; }").unwrap();
        let script = format!(
            "global.window = {{}}; const vm = require('node:vm'); vm.runInThisContext({}); if (getTestData() !== 'original') throw Error('missing helper was not loaded'); const original = describeElement; vm.runInThisContext({}); if (getTestData() !== 'original' || describeElement !== original) throw Error('existing helper was replaced');",
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap(),
        );
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(script)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
