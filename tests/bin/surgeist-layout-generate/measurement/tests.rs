use super::*;
use serde_json::json;

fn box_fixture() -> serde_json::Value {
    json!({"layoutInput":"box","tagName":"div","useRounding":false,"viewport":{"rootContext":"root","width":{"unit":"max-content"},"height":{"unit":"max-content"}},"style":{},"unroundedLayout":{"x":0,"y":0,"width":20,"height":10},"children":[]})
}
fn decode_fixture(value: &serde_json::Value) -> Result<DecodedMeasurement, MeasurementError> {
    decode(&value.to_string(), "fixture/case", "border-box-ltr")
}
fn supported(value: &serde_json::Value) -> ValidatedMeasurement {
    match decode_fixture(value).expect("valid fixture") {
        DecodedMeasurement::Supported(value) => value,
        DecodedMeasurement::Unsupported { reason } => panic!("unexpected unsupported {reason}"),
    }
}
#[test]
fn malformed_geometry_returns_location_and_json_source() {
    let mut value = box_fixture();
    value["unroundedLayout"]["width"] = json!("invalid");
    let error = decode_fixture(&value).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::Decode);
    assert_eq!(error.case_id, "fixture/case");
    assert_eq!(error.variant, "border-box-ltr");
    assert_eq!(error.field_path, "unroundedLayout.width");
    assert!(std::error::Error::source(&error).is_some());
}
#[test]
fn unknown_and_duplicate_fields_are_rejected_before_unsupported_classification() {
    for raw in [
        r#"{"layoutInput":"unsupported","unsupportedReason":"known","unknown":0}"#,
        r#"{"layoutInput":"unsupported","unsupportedReason":"first","unsupportedReason":"second"}"#,
    ] {
        assert_eq!(
            decode(raw, "case", "variant").unwrap_err().kind,
            MeasurementErrorKind::Decode
        );
    }
    let raw = r#"{"layoutInput":"box","unsupportedReason":"known","style":{"size":{"width":{"unit":"px","unit":"auto","value":2}}}}"#;
    assert_eq!(
        decode(raw, "case", "variant").unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
}
#[test]
fn unsupported_reason_precedence_does_not_require_supported_geometry() {
    let value = json!({"layoutInput":"box","unsupportedReason":"root","children":[{"layoutInput":"unsupported","unsupportedReason":"child"}]});
    assert!(
        matches!(decode_fixture(&value),Ok(DecodedMeasurement::Unsupported{reason}) if reason=="root")
    );
    let value = json!({"layoutInput":"box","children":[{"layoutInput":"box","children":[{"layoutInput":"unsupported","unsupportedReason":"first"}]},{"layoutInput":"unsupported","unsupportedReason":"second"}]});
    assert!(
        matches!(decode_fixture(&value),Ok(DecodedMeasurement::Unsupported{reason}) if reason=="first")
    );
}
#[test]
fn required_supported_fields_never_default() {
    for field in ["useRounding", "viewport", "children", "unroundedLayout"] {
        let mut value = box_fixture();
        value.as_object_mut().unwrap().remove(field);
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::MissingField);
        assert_eq!(error.field_path, field);
    }
}
#[test]
fn geometry_extents_and_f32_conversion_are_validated_before_formatting() {
    for (value, kind) in [
        (-1.0, MeasurementErrorKind::InvalidValue),
        (f64::from(f32::MAX) * 2.0, MeasurementErrorKind::OutOfRange),
    ] {
        let mut fixture = box_fixture();
        fixture["unroundedLayout"]["width"] = json!(value);
        assert_eq!(decode_fixture(&fixture).unwrap_err().kind, kind);
    }
    let mut fixture = box_fixture();
    fixture["unroundedLayout"]["width"] = json!(f64::from(f32::MAX));
    let measurement = supported(&fixture);
    assert!(
        crate::xml::generate_xml("representable", &measurement)
            .contains(&format!("width=\"{:.0}\"", f32::MAX))
    );
}
#[test]
fn dimension_units_values_and_track_arity_are_closed() {
    for dimension in [
        json!({"unit":"px"}),
        json!({"unit":"px","value":"3"}),
        json!({"unit":"auto","value":0}),
        json!({"unit":"em","value":2}),
        json!({"unit":"percent","value":f64::MAX}),
    ] {
        let mut value = box_fixture();
        value["style"]["size"] = json!({"width":dimension});
        assert!(decode_fixture(&value).is_err());
    }
    for arguments in [
        json!([]),
        json!([{"kind":"scalar","unit":"px","value":1}]),
        json!([{"kind":"scalar","unit":"px","value":1},{"kind":"scalar","unit":"px","value":2},{"kind":"scalar","unit":"px","value":3}]),
    ] {
        let mut value = box_fixture();
        value["style"]["gridTemplateColumns"] =
            json!([{"kind":"function","name":"minmax","arguments":arguments}]);
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.field_path, "style.gridTemplateColumns[0].arguments");
    }
}
#[test]
fn inline_metrics_are_coupled_and_zero_height_remains_valid() {
    let mut fixture = box_fixture();
    fixture["style"]["inlineMetrics"] = json!({"baseline":0,"lineHeight":0});
    assert!(decode_fixture(&fixture).is_ok());
    fixture["style"]["inlineMetrics"] = json!({"baseline":3,"lineHeight":2});
    let error = decode_fixture(&fixture).unwrap_err();
    assert_eq!(error.field_path, "style.inlineMetrics.lineHeight");
    fixture["style"]["inlineMetrics"] = json!({"baseline":2});
    assert_eq!(
        decode_fixture(&fixture).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
}
#[test]
fn old_metric_strings_and_flat_shape_endpoints_are_not_v1() {
    let mut value = box_fixture();
    value["style"]["inlineBaseline"] = json!("8px");
    assert_eq!(
        decode_fixture(&value).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
    let mut value = box_fixture();
    value["shapeBands"] =
        json!([{"bandMinimum":0,"bandMaximum":10,"intervalMinimum":0,"intervalMaximum":20}]);
    assert_eq!(
        decode_fixture(&value).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
}
#[test]
fn optional_shape_interval_and_explicit_empty_observations_are_distinct() {
    let mut value = box_fixture();
    value["shapeBands"] = json!([{"bandMinimum":0,"bandMaximum":10},{"bandMinimum":10,"bandMaximum":20,"interval":{"minimum":2,"maximum":8}}]);
    value["fragments"] = json!([]);
    let xml = crate::xml::generate_xml("shape", &supported(&value));
    assert!(xml.contains("<fragments/>"));
    assert!(xml.contains("<shape-band band-minimum=\"0\" band-maximum=\"10\"/>"));
    assert!(xml.contains("interval-minimum=\"2\" interval-maximum=\"8\""));
    value.as_object_mut().unwrap().remove("fragments");
    assert!(!crate::xml::generate_xml("absent", &supported(&value)).contains("<fragments"));
    value["fragments"] = serde_json::Value::Null;
    assert_eq!(
        decode_fixture(&value).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
}
#[test]
fn range_observations_cannot_mix_with_box_geometry() {
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"inline-text","inlineSegments":[{"id":u64::MAX,"inlineExtent":0,"inlineBaseline":0,"inlineLineHeight":0,"bidiLevel":125,"whitespaceEdge":"preserve","followingBreak":"prohibited"}],"rangeInks":[],"children":[]}]);
    let xml = crate::xml::generate_xml("empty-range", &supported(&root));
    assert!(xml.contains("<range-inks/>"));
    assert!(xml.contains(&u64::MAX.to_string()));
    root["children"][0]["unroundedLayout"] = json!({"x":0,"y":0,"width":0,"height":0});
    let error = decode_fixture(&root).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
    assert_eq!(error.node_path, "root.children[0]");
    assert_eq!(error.field_path, "rangeInks");
}
#[test]
fn boundary_metrics_and_end_payload_are_checked() {
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"inline-boundary","inlineBoundary":{"kind":"start","baseline":0,"lineHeight":0},"children":[]}]);
    assert_eq!(
        decode_fixture(&root).unwrap_err().field_path,
        "inlineBoundary.lineHeight"
    );
    root["children"][0]["inlineBoundary"] = json!({"kind":"start","baseline":2,"lineHeight":3});
    assert!(decode_fixture(&root).is_ok());
    root["children"][0]["inlineBoundary"]["kind"] = json!("end");
    assert_eq!(
        decode_fixture(&root).unwrap_err().kind,
        MeasurementErrorKind::ContradictoryFields
    );
}

#[test]
fn null_text_and_foreign_role_payload_are_rejected() {
    let mut root = box_fixture();
    root["textContent"] = serde_json::Value::Null;
    assert_eq!(
        decode_fixture(&root).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
    let reason = json!({"layoutInput":"unsupported","unsupportedReason":"known","style":{}});
    assert_eq!(
        decode_fixture(&reason).unwrap_err().kind,
        MeasurementErrorKind::ContradictoryFields
    );
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"inline-boundary","inlineBoundary":{"kind":"start"},"style":{},"children":[]}]);
    assert_eq!(
        decode_fixture(&root).unwrap_err().kind,
        MeasurementErrorKind::ContradictoryFields
    );
}

#[test]
fn unsupported_measurements_skip_metric_semantics_after_closed_wire_validation() {
    let fixture = json!({"layoutInput":"box","unsupportedReason":"outside layout","style":{"inlineMetrics":{"baseline":3,"lineHeight":2}},"unroundedLayout":{"width":-1}});
    assert!(
        matches!(decode_fixture(&fixture),Ok(DecodedMeasurement::Unsupported{reason}) if reason=="outside layout")
    );
}
#[test]
fn duplicate_shape_queries_are_rejected() {
    let mut fixture = box_fixture();
    fixture["shapeBands"] = json!([{"bandMinimum":0,"bandMaximum":10},{"bandMinimum":0,"bandMaximum":10,"interval":{"minimum":0,"maximum":5}}]);
    assert_eq!(
        decode_fixture(&fixture).unwrap_err().field_path,
        "shapeBands[1]"
    );
}

#[test]
fn unsupported_roles_require_reasons_and_cannot_hide_foreign_inline_payloads() {
    let mut fixture = box_fixture();
    fixture["layoutInput"] = json!("unsupported");
    let error = decode_fixture(&fixture).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::MissingField);
    assert_eq!(error.field_path, "unsupportedReason");
    for role in ["inline-text", "inline-boundary"] {
        let value = json!({"layoutInput":role,"unsupportedReason":"hidden"});
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
        assert_eq!(error.field_path, "unsupportedReason");
    }
    let fixture = json!({"layoutInput":"box","unsupportedReason":"hidden","inlineSegments":[]});
    let error = decode_fixture(&fixture).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
    assert_eq!(error.field_path, "layoutInput");
}

#[test]
fn unsupported_ancestors_do_not_hide_conflicting_inline_observation_forms() {
    let value = json!({"layoutInput":"box","unsupportedReason":"outside layout","children":[{"layoutInput":"inline-text","children":[],"rangeInks":[],"unroundedLayout":{"width":-1}}]});
    let error = decode_fixture(&value).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
    assert_eq!(error.node_path, "root.children[0]");
    assert_eq!(error.field_path, "rangeInks");
}

#[test]
fn root_atomic_participation_requires_a_parent_input_slot() {
    let mut value = box_fixture();
    value["atomicInlineParticipation"] = json!({"bidiLevel":0,"followingBreak":"prohibited"});
    let error = decode_fixture(&value).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
    assert_eq!(error.node_path, "root");
    assert_eq!(error.field_path, "atomicInlineParticipation");
}

#[test]
fn scrolling_uses_the_root_selected_geometry_observation() {
    let mut value = box_fixture();
    value["useRounding"] = json!(true);
    value["style"]["overflowX"] = json!("auto");
    value["unroundedLayout"] =
        json!({"x":0,"y":0,"width":20,"height":10,"scrollWidth":30,"scrollHeight":10});
    value["smartRoundedLayout"] =
        json!({"x":0,"y":0,"width":20,"height":10,"scrollWidth":50,"scrollHeight":10});
    value["naivelyRoundedLayout"] = json!({"clientWidth":20,"clientHeight":10});
    assert!(
        crate::xml::generate_xml("rounded", &supported(&value)).contains("scroll_width=\"30\"")
    );
    value["useRounding"] = json!(false);
    assert!(
        crate::xml::generate_xml("unrounded", &supported(&value)).contains("scroll_width=\"10\"")
    );
}

#[test]
fn unsupported_classification_does_not_bypass_structural_alternatives() {
    let cases = [
        (
            json!({"layoutInput":"box","unsupportedReason":"known","children":[{"layoutInput":"inline-boundary","inlineBoundary":{"kind":"end","baseline":0,"lineHeight":1},"children":[]}]}),
            "inlineBoundary.kind",
        ),
        (
            json!({"layoutInput":"box","unsupportedReason":"known","viewport":{"rootContext":"root","width":{"unit":"max-content"},"height":{"unit":"max-content"},"hostInlineSize":20}}),
            "viewport.rootContext",
        ),
        (
            json!({"layoutInput":"box","unsupportedReason":"known","children":[{"layoutInput":"inline-text","inlineSegments":[{"id":0,"inlineExtent":1,"inlineBaseline":0,"inlineLineHeight":1,"bidiLevel":0,"whitespaceEdge":"preserve","followingBreak":"prohibited","replacementInlineExtent":1}],"rangeInks":[],"children":[]}]}),
            "inlineSegments[0].replacementInlineExtent",
        ),
    ];
    for (value, field) in cases {
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
        assert_eq!(error.field_path, field);
    }
    let mut root = box_fixture();
    root["unsupportedReason"] = json!("known");
    root["viewport"] = json!({"rootContext":"flex-item","width":{"unit":"max-content"},"height":{"unit":"max-content"},"parentWritingMode":"horizontal-tb","parentDirection":"bogus","hostInlineSize":-1});
    let error = decode_fixture(&root).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
    assert_eq!(error.field_path, "viewport.parentDirection");
}

#[test]
fn block_br_used_extents_keep_width_height_order_before_insets() {
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"box","tagName":"br","style":{"display":"block","size":{"height":{"unit":"px","value":1}},"inset":{"left":{"unit":"px","value":3}}},"unroundedLayout":{"x":0,"y":0,"width":20,"height":10},"children":[]}]);
    let xml = crate::xml::generate_xml("block-br", &supported(&root));
    assert!(
        xml.contains(
            "<div source-tag=\"br\" display=\"block\" width=\"20px\" height=\"10px\" left=\"3px\"/>"
        ),
        "{xml}"
    );
}

#[test]
fn invalid_xml_characters_are_rejected_before_serialization() {
    for field in ["textContent", "tagName"] {
        let mut value = box_fixture();
        value[field] = json!("invalid\u{0001}text");
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.field_path, field);
    }
}
#[test]
fn cdata_terminators_in_text_are_escaped_without_changing_text() {
    let mut value = box_fixture();
    value["textContent"] = json!("before ]]> after");
    let xml = crate::xml::generate_xml("text", &supported(&value));
    let document = roxmltree::Document::parse(&xml).expect("generated XML must be well formed");
    let text = document
        .descendants()
        .find(|node| node.has_tag_name("text"))
        .unwrap()
        .text()
        .unwrap()
        .trim();
    assert_eq!(text, "before ]]> after");
}

#[test]
fn viewport_constraints_use_available_space_alternatives() {
    for dimension in [
        json!({"unit":"auto"}),
        json!({"unit":"percent","value":0.5}),
        json!({"unit":"fraction","value":1}),
    ] {
        let mut value = box_fixture();
        value["viewport"]["width"] = dimension;
        value["unsupportedReason"] = json!("known");
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.field_path, "viewport.width");
    }
    let mut value = box_fixture();
    value["viewport"]["width"] = json!({"unit":"px","value":-1});
    assert_eq!(
        decode_fixture(&value).unwrap_err().field_path,
        "viewport.width"
    );
    value["unsupportedReason"] = json!("known");
    assert!(matches!(
        decode_fixture(&value),
        Ok(DecodedMeasurement::Unsupported { .. })
    ));
}
