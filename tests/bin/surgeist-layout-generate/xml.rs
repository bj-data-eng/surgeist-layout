//! Layout-owned browser measurement to fixture XML lowering.

use serde_json::Value;

pub(super) fn generate_xml(name: &str, node: &Value) -> String {
    let mut lines = Vec::new();
    let use_rounding = bool_field(node, "useRounding");
    lines.push(format!(
        "<test name=\"{}\" use-rounding=\"{}\">",
        escape_attr(name),
        use_rounding
    ));
    let viewport = &node["viewport"];
    let root_context = viewport["rootContext"].as_str().unwrap_or("root");
    let root_context_attr = if root_context == "root" {
        String::new()
    } else {
        let host_inline_size = viewport["hostInlineSize"]
            .as_f64()
            .filter(|value| value.is_finite() && *value >= 0.0)
            .expect("flex-item viewport host inline size must be finite and non-negative");
        format!(
            " root-context=\"{}\" parent-writing-mode=\"{}\" parent-direction=\"{}\" host-inline-size=\"{}px\"",
            escape_attr(root_context),
            escape_attr(viewport["parentWritingMode"].as_str().unwrap_or_default()),
            escape_attr(viewport["parentDirection"].as_str().unwrap_or_default()),
            number_attr_value(host_inline_size),
        )
    };
    lines.push(format!(
        "  <viewport width=\"{}\" height=\"{}\"{}/>",
        dimension(&viewport["width"]).unwrap_or_default(),
        dimension(&viewport["height"]).unwrap_or_default(),
        root_context_attr
    ));
    lines.push("  <input>".to_string());
    write_input(&mut lines, node, 4, "horizontal-tb");
    lines.push("  </input>".to_string());
    lines.push("  <expectations>".to_string());
    write_expectation(
        &mut lines,
        node,
        ExpectationWriteContext {
            use_rounding,
            indent: 4,
            is_root: true,
            parent_abs_x: 0.0,
            parent_abs_y: 0.0,
        },
    );
    lines.push("  </expectations>".to_string());
    lines.push("</test>".to_string());
    format!("{}\n", lines.join("\n"))
}

fn write_input(lines: &mut Vec<String>, node: &Value, indent: usize, parent_writing_mode: &str) {
    line_control_kind(node);
    if node["layoutInput"].as_str() == Some("inline-boundary") {
        write_inline_boundary_input(lines, node, indent);
        return;
    }
    if node["layoutInput"].as_str() == Some("inline-text") {
        write_inline_text_input(lines, node, indent);
        return;
    }

    let children = node["children"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let has_typed_inline_text = children
        .iter()
        .any(|child| child["layoutInput"].as_str() == Some("inline-text"));
    let text_content = (!has_typed_inline_text)
        .then(|| node.get("textContent"))
        .flatten();
    let attrs = input_attrs_with_parent_writing_mode(node, parent_writing_mode);
    let writing_mode =
        string(&node["style"], "writingMode").unwrap_or_else(|| "horizontal-tb".to_string());
    let tag = if text_content.is_some() && !direct_text_requires_container(node) {
        "text"
    } else {
        "div"
    };
    let pad = " ".repeat(indent);
    let has_atomic_placeholders = children
        .iter()
        .any(|child| !child["atomicInlineParticipation"].is_null());
    let has_shape_bands = node["shapeBands"].as_array().is_some();

    if children.is_empty() && text_content.is_none() && !has_atomic_placeholders && !has_shape_bands
    {
        lines.push(format!("{pad}<{tag}{}/>", attr_text(&attrs)));
        return;
    }

    lines.push(format!("{pad}<{tag}{}>", attr_text(&attrs)));
    for child in children {
        write_input(lines, child, indent + 2, &writing_mode);
    }
    for (child_index, child) in children.iter().enumerate() {
        write_atomic_placeholder(lines, child, child_index, indent + 2);
    }
    if let Some(bands) = node["shapeBands"].as_array() {
        write_shape_bands(lines, bands, indent + 2);
    }
    if let Some(text) = text_content.and_then(Value::as_str) {
        lines.push(format!(
            "{}{}",
            " ".repeat(indent + 2),
            escape_text(text.trim())
        ));
    }
    lines.push(format!("{pad}</{tag}>"));
}

fn write_inline_boundary_input(lines: &mut Vec<String>, node: &Value, indent: usize) {
    let object = node
        .as_object()
        .expect("layout-ready inline boundary must be an object");
    assert!(
        object
            .keys()
            .all(|key| matches!(key.as_str(), "layoutInput" | "inlineBoundary" | "children")),
        "layout-ready inline boundary contains an unsupported field"
    );
    assert!(
        node["children"].as_array().is_none_or(Vec::is_empty),
        "layout-ready inline boundary must not contain payload"
    );
    let boundary = node["inlineBoundary"]
        .as_object()
        .expect("layout-ready inline boundary requires a closed descriptor");
    let kind = boundary
        .get("kind")
        .and_then(Value::as_str)
        .filter(|kind| matches!(*kind, "start" | "end"))
        .expect("layout-ready inline boundary kind must be start or end");
    assert!(
        boundary
            .keys()
            .all(|key| { matches!(key.as_str(), "kind" | "baseline" | "lineHeight") }),
        "layout-ready inline boundary descriptor contains an unsupported field"
    );
    let baseline = boundary.get("baseline");
    let line_height = boundary.get("lineHeight");
    let mut attrs = vec![("kind", kind.to_string())];
    match (baseline, line_height) {
        (None, None) => {}
        (Some(baseline), Some(line_height)) => {
            assert_eq!(
                kind, "start",
                "only a start inline boundary may carry metrics"
            );
            let baseline = baseline
                .as_f64()
                .filter(|value| value.is_finite() && *value >= 0.0)
                .expect("inline boundary baseline must be finite and non-negative");
            let line_height = line_height
                .as_f64()
                .filter(|value| value.is_finite() && *value > 0.0 && *value >= baseline)
                .expect("inline boundary line height must be finite, positive, and cover baseline");
            attrs.push(("inline-baseline", number_attr_value(baseline)));
            attrs.push(("inline-line-height", number_attr_value(line_height)));
        }
        _ => panic!("layout-ready inline boundary metrics must be complete"),
    }
    lines.push(format!(
        "{}<inline-boundary{}/>",
        " ".repeat(indent),
        attr_text(&attrs)
    ));
}

fn write_inline_text_input(lines: &mut Vec<String>, node: &Value, indent: usize) {
    let segments = node["inlineSegments"]
        .as_array()
        .filter(|segments| !segments.is_empty())
        .expect("layout-ready inline text requires at least one complete segment");
    let pad = " ".repeat(indent);
    lines.push(format!(r#"{pad}<text layout-input="inline-text">"#));
    for segment in segments {
        let mut attrs = vec![
            ("id", required_integer_attr(segment, "id")),
            (
                "inline-extent",
                required_nonnegative_number_attr(segment, "inlineExtent"),
            ),
            (
                "inline-baseline",
                required_nonnegative_number_attr(segment, "inlineBaseline"),
            ),
            (
                "inline-line-height",
                required_nonnegative_number_attr(segment, "inlineLineHeight"),
            ),
            ("bidi-level", required_integer_attr(segment, "bidiLevel")),
            (
                "whitespace-edge",
                required_string_attr(segment, "whitespaceEdge"),
            ),
            (
                "following-break",
                required_string_attr(segment, "followingBreak"),
            ),
        ];
        maybe_break_replacement_attr(&mut attrs, segment);
        lines.push(format!(
            "{}<segment{}/>",
            " ".repeat(indent + 2),
            attr_text(&attrs)
        ));
    }
    lines.push(format!("{pad}</text>"));
}

fn write_atomic_placeholder(
    lines: &mut Vec<String>,
    child: &Value,
    child_index: usize,
    indent: usize,
) {
    let participation = &child["atomicInlineParticipation"];
    if participation.is_null() {
        return;
    }
    let mut attrs = vec![
        ("child-index", child_index.to_string()),
        (
            "bidi-level",
            required_integer_attr(participation, "bidiLevel"),
        ),
        (
            "following-break",
            required_string_attr(participation, "followingBreak"),
        ),
    ];
    maybe_break_replacement_attr(&mut attrs, participation);
    lines.push(format!(
        "{}<atomic-placeholder{}/>",
        " ".repeat(indent),
        attr_text(&attrs)
    ));
}

fn write_shape_bands(lines: &mut Vec<String>, bands: &[Value], indent: usize) {
    assert!(
        !bands.is_empty(),
        "layout-ready fixture field `shapeBands` must be nonempty"
    );
    let pad = " ".repeat(indent);
    lines.push(format!("{pad}<shape-bands>"));
    for band in bands {
        let mut attrs = vec![
            (
                "band-minimum",
                required_finite_number_attr(band, "bandMinimum"),
            ),
            (
                "band-maximum",
                required_finite_number_attr(band, "bandMaximum"),
            ),
        ];
        match (
            band.get("intervalMinimum").filter(|value| !value.is_null()),
            band.get("intervalMaximum").filter(|value| !value.is_null()),
        ) {
            (Some(_), Some(_)) => {
                attrs.push((
                    "interval-minimum",
                    required_finite_number_attr(band, "intervalMinimum"),
                ));
                attrs.push((
                    "interval-maximum",
                    required_finite_number_attr(band, "intervalMaximum"),
                ));
            }
            (None, None) => {}
            _ => panic!("layout-ready fixture field `shapeBands` requires both interval endpoints"),
        }
        lines.push(format!(
            "{}<shape-band{}/>",
            " ".repeat(indent + 2),
            attr_text(&attrs)
        ));
    }
    lines.push(format!("{pad}</shape-bands>"));
}

fn maybe_break_replacement_attr(attrs: &mut Vec<(&'static str, String)>, value: &Value) {
    if !value["replacementInlineExtent"].is_null() {
        attrs.push((
            "replacement-inline-extent",
            required_nonnegative_number_attr(value, "replacementInlineExtent"),
        ));
    }
}

fn required_string_attr(value: &Value, field: &str) -> String {
    value[field]
        .as_str()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| panic!("layout-ready fixture field `{field}` must be a nonempty string"))
        .to_string()
}

fn required_integer_attr(value: &Value, field: &str) -> String {
    value[field]
        .as_u64()
        .unwrap_or_else(|| panic!("layout-ready fixture field `{field}` must be an integer"))
        .to_string()
}

fn required_nonnegative_number_attr(value: &Value, field: &str) -> String {
    let number = value[field]
        .as_f64()
        .filter(|number| number.is_finite() && *number >= 0.0)
        .unwrap_or_else(|| {
            panic!("layout-ready fixture field `{field}` must be finite and non-negative")
        });
    number_attr_value(number)
}

fn direct_text_requires_container(node: &Value) -> bool {
    matches!(
        node["style"]["display"].as_str(),
        Some("grid" | "inline-grid" | "grid-lanes" | "inline-grid-lanes")
    )
}

#[derive(Clone, Copy, Debug)]
struct ExpectationWriteContext {
    use_rounding: bool,
    indent: usize,
    is_root: bool,
    parent_abs_x: f64,
    parent_abs_y: f64,
}

impl ExpectationWriteContext {
    fn child(self, abs_x: f64, abs_y: f64) -> Self {
        Self {
            indent: self.indent + 2,
            is_root: false,
            parent_abs_x: abs_x,
            parent_abs_y: abs_y,
            ..self
        }
    }
}

fn write_expectation(lines: &mut Vec<String>, node: &Value, context: ExpectationWriteContext) {
    line_control_kind(node);
    if let Some(range_inks) = node["rangeInks"].as_array() {
        assert_eq!(
            node["layoutInput"].as_str(),
            Some("inline-text"),
            "layout-ready fixture field `rangeInks` is valid only on inline text"
        );
        assert!(
            node["fragments"].is_null(),
            "Range ink and explicit model fragments are distinct expectation categories"
        );
        assert!(
            node["children"].as_array().is_none_or(Vec::is_empty),
            "layout-ready Range ink text must not contain child expectations"
        );
        for field in [
            "unroundedLayout",
            "smartRoundedLayout",
            "naivelyRoundedLayout",
        ] {
            assert!(
                node.get(field).is_none(),
                "layout-ready Range ink text must not contain ordinary node metric or scroll state `{field}`"
            );
        }
        if let Some(style) = node["style"].as_object() {
            for field in [
                "overflowX",
                "overflowY",
                "overflowClipMargin",
                "scrollbarWidth",
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
                    !style.contains_key(field),
                    "layout-ready Range ink text must not contain ordinary overflow or scroll input `style.{field}`"
                );
            }
        }
        let pad = " ".repeat(context.indent);
        lines.push(format!("{pad}<node>"));
        write_range_ink_expectations(lines, range_inks, context.indent + 2);
        lines.push(format!("{pad}</node>"));
        return;
    }

    let layout = if context.use_rounding {
        &node["smartRoundedLayout"]
    } else {
        &node["unroundedLayout"]
    };
    let abs_x = if context.is_root {
        0.0
    } else {
        context.parent_abs_x + number(&layout["x"])
    };
    let abs_y = if context.is_root {
        0.0
    } else {
        context.parent_abs_y + number(&layout["y"])
    };
    let mut attrs = vec![
        (
            "x",
            if context.is_root {
                "0".to_string()
            } else {
                layout_number_attr(&layout["x"])
            },
        ),
        (
            "y",
            if context.is_root {
                "0".to_string()
            } else {
                layout_number_attr(&layout["y"])
            },
        ),
        ("width", layout_number_attr(&layout["width"])),
        ("height", layout_number_attr(&layout["height"])),
    ];

    let overflow_x = node["style"]["overflowX"].as_str().unwrap_or_default();
    let overflow_y = node["style"]["overflowY"].as_str().unwrap_or_default();
    if ["hidden", "scroll", "auto"].contains(&overflow_x)
        || ["hidden", "scroll", "auto"].contains(&overflow_y)
    {
        let client = &node["naivelyRoundedLayout"];
        attrs.push((
            "scroll_width",
            layout_number_attr_value(
                (number(&layout["scrollWidth"]) - number(&client["clientWidth"])).max(0.0),
            ),
        ));
        attrs.push((
            "scroll_height",
            layout_number_attr_value(
                (number(&layout["scrollHeight"]) - number(&client["clientHeight"])).max(0.0),
            ),
        ));
    }

    let pad = " ".repeat(context.indent);
    let children = node["children"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let expectation_children = children
        .iter()
        .filter(|child| child["layoutInput"].as_str() != Some("inline-boundary"))
        .cloned()
        .collect::<Vec<_>>();
    let fragments = node["fragments"].as_array();
    if expectation_children.is_empty() && fragments.is_none() {
        lines.push(format!("{pad}<node{}/>", attr_text(&attrs)));
        return;
    }

    lines.push(format!("{pad}<node{}>", attr_text(&attrs)));
    if let Some(fragments) = fragments {
        write_fragment_expectations(lines, fragments, context.indent + 2);
    }
    for (source_index, child) in expectation_children.iter().enumerate() {
        if line_control_kind(child).is_some() {
            write_browser_control_expectation(
                lines,
                node,
                &expectation_children,
                source_index,
                context.indent + 2,
            );
        } else {
            write_expectation(lines, child, context.child(abs_x, abs_y));
        }
    }
    lines.push(format!("{pad}</node>"));
}

#[derive(Clone, Copy)]
struct BrowserBlockInterval {
    minimum: f64,
    maximum: f64,
}

fn write_browser_control_expectation(
    lines: &mut Vec<String>,
    parent: &Value,
    siblings: &[Value],
    source_index: usize,
    indent: usize,
) {
    let control = &siblings[source_index];
    assert_eq!(
        line_control_kind(control),
        Some("forced-break"),
        "browser control observation requires explicit line-control participation"
    );
    assert_eq!(
        control["tagName"].as_str(),
        Some("br"),
        "browser control observation requires a BR source"
    );
    assert!(
        control["children"].as_array().is_none_or(Vec::is_empty),
        "browser control observation must not contain child expectations"
    );
    let control_interval = browser_block_interval(parent, control);
    let current_line_candidates = siblings[..source_index]
        .iter()
        .rev()
        .take_while(|sibling| line_control_kind(sibling).is_none())
        .collect::<Vec<_>>();
    let terminal_visual_slot = current_line_candidates
        .iter()
        .all(|sibling| browser_block_interval_if_present(parent, sibling).is_some())
        .then(|| {
            current_line_candidates
                .iter()
                .filter(|sibling| {
                    browser_block_interval_if_present(parent, sibling).is_some_and(|interval| {
                        browser_block_relation(parent, control_interval, interval) == "same"
                    })
                })
                .count()
        });
    let previous_line = source_index
        .checked_sub(1)
        .map(|index| browser_block_interval_if_present(parent, &siblings[index]))
        .map_or("absent", |interval| {
            interval.map_or("unobserved", |interval| {
                browser_block_relation(parent, control_interval, interval)
            })
        });
    let next_line = siblings
        .get(source_index + 1)
        .map(|sibling| browser_block_interval_if_present(parent, sibling))
        .map_or("absent", |interval| {
            interval.map_or("unobserved", |interval| {
                browser_block_relation(parent, control_interval, interval)
            })
        });

    let pad = " ".repeat(indent);
    lines.push(format!("{pad}<node>"));
    lines.push(format!(
        "{}<browser-control{}/>",
        " ".repeat(indent + 2),
        attr_text(&[
            ("source_index", source_index.to_string()),
            (
                "terminal_visual_slot",
                terminal_visual_slot
                    .map_or_else(|| "unobserved".to_string(), |slot| slot.to_string(),),
            ),
            ("previous_line", previous_line.to_string()),
            ("next_line", next_line.to_string()),
        ])
    ));
    lines.push(format!("{pad}</node>"));
}

fn browser_block_interval(parent: &Value, node: &Value) -> BrowserBlockInterval {
    browser_block_interval_if_present(parent, node)
        .expect("browser control observations require unrounded sibling geometry")
}

fn browser_block_interval_if_present(parent: &Value, node: &Value) -> Option<BrowserBlockInterval> {
    let layout = node.get("unroundedLayout")?;
    let vertical = parent["style"]["writingMode"]
        .as_str()
        .is_some_and(|writing_mode| writing_mode != "horizontal-tb");
    let (minimum, extent) = if vertical {
        (layout["x"].as_f64()?, layout["width"].as_f64()?)
    } else {
        (layout["y"].as_f64()?, layout["height"].as_f64()?)
    };
    assert!(
        minimum.is_finite() && extent.is_finite() && extent >= 0.0,
        "browser control observations require finite non-negative block geometry"
    );
    Some(BrowserBlockInterval {
        minimum,
        maximum: minimum + extent,
    })
}

fn browser_block_relation(
    parent: &Value,
    control: BrowserBlockInterval,
    neighbor: BrowserBlockInterval,
) -> &'static str {
    if neighbor.maximum >= control.minimum && control.maximum >= neighbor.minimum {
        return "same";
    }
    let control_center = control.minimum + (control.maximum - control.minimum) / 2.0;
    let neighbor_center = neighbor.minimum + (neighbor.maximum - neighbor.minimum) / 2.0;
    let block_decreases = matches!(
        parent["style"]["writingMode"].as_str(),
        Some("vertical-rl" | "sideways-rl")
    );
    let neighbor_is_earlier = if block_decreases {
        neighbor_center > control_center
    } else {
        neighbor_center < control_center
    };
    if neighbor_is_earlier {
        "earlier"
    } else {
        "later"
    }
}

fn write_fragment_expectations(lines: &mut Vec<String>, fragments: &[Value], indent: usize) {
    let pad = " ".repeat(indent);
    if fragments.is_empty() {
        lines.push(format!("{pad}<fragments/>"));
        return;
    }
    lines.push(format!("{pad}<fragments>"));
    for fragment in fragments {
        let attrs = [
            (
                "source_segment_id",
                required_integer_attr(fragment, "sourceSegmentId"),
            ),
            ("line_index", required_integer_attr(fragment, "lineIndex")),
            (
                "visual_index",
                required_integer_attr(fragment, "visualIndex"),
            ),
            ("x", required_finite_number_attr(fragment, "x")),
            ("y", required_finite_number_attr(fragment, "y")),
            ("width", required_nonnegative_number_attr(fragment, "width")),
            (
                "height",
                required_nonnegative_number_attr(fragment, "height"),
            ),
            (
                "baseline_x",
                required_finite_number_attr(fragment, "baselineX"),
            ),
            (
                "baseline_y",
                required_finite_number_attr(fragment, "baselineY"),
            ),
        ];
        lines.push(format!(
            "{}<fragment{}/>",
            " ".repeat(indent + 2),
            attr_text(&attrs)
        ));
    }
    lines.push(format!("{pad}</fragments>"));
}

fn write_range_ink_expectations(lines: &mut Vec<String>, range_inks: &[Value], indent: usize) {
    let pad = " ".repeat(indent);
    if range_inks.is_empty() {
        lines.push(format!("{pad}<range-inks/>"));
        return;
    }
    lines.push(format!("{pad}<range-inks>"));
    for range_ink in range_inks {
        let physical_start_edge = required_string_attr(range_ink, "physicalStartEdge");
        assert!(
            matches!(
                physical_start_edge.as_str(),
                "left" | "right" | "top" | "bottom"
            ),
            "layout-ready fixture field `physicalStartEdge` must name a physical edge"
        );
        let attrs = [
            (
                "source_segment_id",
                required_integer_attr(range_ink, "sourceSegmentId"),
            ),
            ("line_index", required_integer_attr(range_ink, "lineIndex")),
            ("physical_start_edge", physical_start_edge),
            ("start", required_finite_number_attr(range_ink, "start")),
            (
                "advance",
                required_nonnegative_number_attr(range_ink, "advance"),
            ),
        ];
        lines.push(format!(
            "{}<range-ink{}/>",
            " ".repeat(indent + 2),
            attr_text(&attrs)
        ));
    }
    lines.push(format!("{pad}</range-inks>"));
}

fn required_finite_number_attr(value: &Value, field: &str) -> String {
    let number = value[field]
        .as_f64()
        .filter(|number| number.is_finite())
        .unwrap_or_else(|| panic!("layout-ready fixture field `{field}` must be finite"));
    number_attr_value(number)
}

#[cfg(test)]
fn input_attrs(node: &Value) -> Vec<(&'static str, String)> {
    input_attrs_with_parent_writing_mode(node, "horizontal-tb")
}

fn input_attrs_with_parent_writing_mode(
    node: &Value,
    parent_writing_mode: &str,
) -> Vec<(&'static str, String)> {
    let style = &node["style"];
    let mut attrs = Vec::new();
    let source_tag = string(node, "tagName");
    let is_br = source_tag.as_deref() == Some("br");
    let br_serializes_as_control =
        is_br && matches!(style["display"].as_str(), Some("inline" | "none"));
    let br_serializes_as_box = is_br && style["display"].as_str() == Some("block");
    if !is_br || br_serializes_as_control || br_serializes_as_box {
        maybe(&mut attrs, "source-tag", source_tag, None);
    }
    if let Some(marker) = node.get("layoutReadyInlineRoot") {
        assert_eq!(
            marker.as_bool(),
            Some(true),
            "layout-ready fixture field `layoutReadyInlineRoot` must be true when present"
        );
        attrs.push(("layout-ready-inline-root", "true".to_string()));
    }
    if let Some(marker) = node.get("layoutReadyAnonymousGridTextWrapper") {
        assert_eq!(
            marker.as_bool(),
            Some(true),
            "layout-ready fixture field `layoutReadyAnonymousGridTextWrapper` must be true when present"
        );
        assert!(
            matches!(
                style["display"].as_str(),
                Some("grid" | "inline-grid" | "grid-lanes" | "inline-grid-lanes")
            ),
            "anonymous grid text wrapper marker requires a grid formatting role"
        );
        let children = node["children"]
            .as_array()
            .filter(|children| !children.is_empty())
            .expect("anonymous grid text wrapper marker requires direct typed text");
        assert!(
            children.iter().all(|child| {
                child["layoutInput"].as_str() == Some("inline-text")
                    && child["children"].as_array().is_none_or(Vec::is_empty)
            }) && node.get("textContent").is_none_or(Value::is_null),
            "anonymous grid text wrapper marker rejects mixed fallback content"
        );
        attrs.push((
            "layout-ready-anonymous-grid-text-wrapper",
            "true".to_string(),
        ));
    }
    if br_serializes_as_box {
        assert!(
            node["atomicInlineParticipation"].is_null(),
            "computed block BR must not carry atomic inline participation"
        );
    }
    if let Some(kind) = line_control_kind(node) {
        attrs.push(("line-control", kind.to_string()));
    }
    maybe(&mut attrs, "display", string(style, "display"), None);
    maybe(
        &mut attrs,
        "box-sizing",
        string(style, "boxSizing"),
        Some("border-box"),
    );
    maybe(&mut attrs, "direction", string(style, "direction"), None);
    maybe(&mut attrs, "order", string(style, "order"), Some("0"));
    match style.get("flexItemCollapse") {
        None => {}
        Some(Value::String(value)) if value == "collapsed" => {
            attrs.push(("flex-item-collapse", value.clone()));
        }
        Some(value) => {
            panic!(
                "layout-ready fixture field `flexItemCollapse` must be exactly `collapsed` when present, got {value}"
            )
        }
    }
    if let Some(writing_mode) = writing_mode_attr(style, parent_writing_mode) {
        attrs.push(("writing-mode", writing_mode));
    }
    maybe(
        &mut attrs,
        "position",
        string(style, "position"),
        Some("relative"),
    );
    maybe(&mut attrs, "float", string(style, "cssFloat"), None);
    if node["shapeBands"].as_array().is_some() {
        attrs.push(("float-exclusion", "shape".to_string()));
    }
    maybe(&mut attrs, "clear", string(style, "clear"), None);
    maybe(
        &mut attrs,
        "flex-direction",
        string(style, "flexDirection"),
        Some("row"),
    );
    maybe(
        &mut attrs,
        "flex-wrap",
        string(style, "flexWrap"),
        Some("nowrap"),
    );
    let non_default_overflow =
        non_default_overflow(style, "overflowX") || non_default_overflow(style, "overflowY");
    if non_default_overflow {
        attrs.push((
            "overflow-x",
            string(style, "overflowX").unwrap_or_else(|| "visible".to_string()),
        ));
        attrs.push((
            "overflow-y",
            string(style, "overflowY").unwrap_or_else(|| "visible".to_string()),
        ));
        maybe(
            &mut attrs,
            "scrollbar-width",
            number_string(style, "scrollbarWidth"),
            None,
        );
    }
    maybe(
        &mut attrs,
        "overflow-clip-margin",
        string(style, "overflowClipMargin"),
        Some("0px"),
    );
    maybe(
        &mut attrs,
        "scrollbar-gutter",
        string(style, "scrollbarGutter"),
        Some("auto"),
    );
    for (attr, field, initial) in [
        ("scroll-padding-top", "scrollPaddingTop", "auto"),
        ("scroll-padding-right", "scrollPaddingRight", "auto"),
        ("scroll-padding-bottom", "scrollPaddingBottom", "auto"),
        ("scroll-padding-left", "scrollPaddingLeft", "auto"),
        ("scroll-margin-top", "scrollMarginTop", "0px"),
        ("scroll-margin-right", "scrollMarginRight", "0px"),
        ("scroll-margin-bottom", "scrollMarginBottom", "0px"),
        ("scroll-margin-left", "scrollMarginLeft", "0px"),
    ] {
        maybe(&mut attrs, attr, string(style, field), Some(initial));
    }
    maybe(
        &mut attrs,
        "scroll-snap-type",
        string(style, "scrollSnapType"),
        Some("none"),
    );
    maybe(
        &mut attrs,
        "scroll-snap-align",
        string(style, "scrollSnapAlign"),
        Some("none"),
    );
    maybe(
        &mut attrs,
        "scroll-snap-stop",
        string(style, "scrollSnapStop"),
        Some("normal"),
    );
    maybe(&mut attrs, "text-align", string(style, "textAlign"), None);
    maybe(
        &mut attrs,
        "vertical-align",
        string(style, "verticalAlign"),
        Some("baseline"),
    );
    maybe(&mut attrs, "font-family", font_family(style), Some("ahem"));
    maybe(
        &mut attrs,
        "font-size",
        dimension(&style["fontSize"]),
        Some("10px"),
    );
    maybe(
        &mut attrs,
        "line-height",
        dimension(&style["lineHeight"]),
        Some("10px"),
    );
    if !is_br || br_serializes_as_control {
        maybe(
            &mut attrs,
            "inline-baseline",
            dimension_or_non_empty_string(&style["inlineBaseline"]),
            None,
        );
        maybe(
            &mut attrs,
            "inline-line-height",
            dimension_or_non_empty_string(&style["inlineLineHeight"]),
            None,
        );
    }
    maybe(&mut attrs, "align-items", string(style, "alignItems"), None);
    maybe(&mut attrs, "align-self", string(style, "alignSelf"), None);
    maybe(
        &mut attrs,
        "justify-items",
        string(style, "justifyItems"),
        None,
    );
    maybe(
        &mut attrs,
        "justify-self",
        string(style, "justifySelf"),
        None,
    );
    maybe(
        &mut attrs,
        "align-content",
        string(style, "alignContent"),
        None,
    );
    maybe(
        &mut attrs,
        "justify-content",
        string(style, "justifyContent"),
        None,
    );
    maybe(
        &mut attrs,
        "flex-grow",
        number_string(style, "flexGrow"),
        Some("0"),
    );
    maybe(
        &mut attrs,
        "flex-shrink",
        number_string(style, "flexShrink"),
        Some("1"),
    );
    maybe(
        &mut attrs,
        "flex-basis",
        dimension(&style["flexBasis"]),
        Some("auto"),
    );
    if br_serializes_as_box {
        attrs.push((
            "width",
            format!(
                "{}px",
                required_nonnegative_number_attr(&node["unroundedLayout"], "width")
            ),
        ));
        attrs.push((
            "height",
            format!(
                "{}px",
                required_nonnegative_number_attr(&node["unroundedLayout"], "height")
            ),
        ));
    } else {
        maybe(
            &mut attrs,
            "width",
            dimension(&style["size"]["width"]),
            Some("auto"),
        );
        maybe(
            &mut attrs,
            "height",
            dimension(&style["size"]["height"]),
            Some("auto"),
        );
    }
    maybe(
        &mut attrs,
        "min-width",
        dimension(&style["minSize"]["width"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "min-height",
        dimension(&style["minSize"]["height"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "max-width",
        dimension(&style["maxSize"]["width"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "max-height",
        dimension(&style["maxSize"]["height"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "aspect-ratio",
        number_string(style, "aspectRatio"),
        None,
    );
    maybe(&mut attrs, "row-gap", dimension(&style["gap"]["row"]), None);
    maybe(
        &mut attrs,
        "column-gap",
        dimension(&style["gap"]["column"]),
        None,
    );
    edge_attrs(
        &mut attrs,
        "margin",
        &style["margin"],
        ["top", "left", "bottom", "right"],
    );
    logical_inline_margin_attrs(&mut attrs, style);
    edge_attrs(
        &mut attrs,
        "padding",
        &style["padding"],
        ["top", "left", "bottom", "right"],
    );
    edge_attrs(
        &mut attrs,
        "border",
        &style["border"],
        ["top", "left", "bottom", "right"],
    );
    edge_attrs(
        &mut attrs,
        "",
        &style["inset"],
        ["top", "left", "bottom", "right"],
    );
    maybe(
        &mut attrs,
        "grid-auto-flow",
        grid_auto_flow(&style["gridAutoFlow"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-template-rows",
        dimension_list(&style["gridTemplateRows"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-template-columns",
        dimension_list(&style["gridTemplateColumns"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-template-areas",
        grid_template_areas(&style["gridTemplateAreas"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-auto-rows",
        dimension_list(&style["gridAutoRows"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-auto-columns",
        dimension_list(&style["gridAutoColumns"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-row-start",
        grid_position(&style["gridRowStart"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-row-end",
        grid_position(&style["gridRowEnd"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-column-start",
        grid_position(&style["gridColumnStart"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-column-end",
        grid_position(&style["gridColumnEnd"]),
        None,
    );
    attrs
}

fn line_control_kind(node: &Value) -> Option<&str> {
    let participation = node.get("lineControlParticipation")?;
    let participation = participation
        .as_object()
        .expect("layout-ready fixture field `lineControlParticipation` must be an object");
    assert_eq!(
        participation.len(),
        1,
        "layout-ready fixture field `lineControlParticipation` has unsupported fields"
    );
    assert_eq!(
        node["tagName"].as_str(),
        Some("br"),
        "line-control participation requires a BR source"
    );
    assert_eq!(
        node["style"]["display"].as_str(),
        Some("inline"),
        "line-control participation requires a computed inline BR role"
    );
    let kind = participation
        .get("kind")
        .and_then(Value::as_str)
        .expect("layout-ready fixture field `lineControlParticipation.kind` must be a string");
    assert_eq!(
        kind, "forced-break",
        "unsupported layout-ready line-control participation kind"
    );
    Some(kind)
}

fn writing_mode_attr(style: &Value, parent_writing_mode: &str) -> Option<String> {
    let writing_mode = string(style, "writingMode").unwrap_or_else(|| "horizontal-tb".to_string());
    if writing_mode.starts_with("vertical-")
        || writing_mode.starts_with("sideways-")
        || (writing_mode == "horizontal-tb"
            && (parent_writing_mode.starts_with("vertical-")
                || parent_writing_mode.starts_with("sideways-")))
    {
        Some(writing_mode)
    } else {
        None
    }
}

fn maybe(
    attrs: &mut Vec<(&'static str, String)>,
    key: &'static str,
    value: Option<String>,
    elide: Option<&str>,
) {
    if let Some(value) = value
        && elide != Some(value.as_str())
    {
        attrs.push((key, value));
    }
}

fn edge_attrs(
    attrs: &mut Vec<(&'static str, String)>,
    prefix: &'static str,
    edges: &Value,
    names: [&'static str; 4],
) {
    for name in names {
        let key = match (prefix, name) {
            ("margin", "top") => "margin-top",
            ("margin", "right") => "margin-right",
            ("margin", "bottom") => "margin-bottom",
            ("margin", "left") => "margin-left",
            ("padding", "top") => "padding-top",
            ("padding", "right") => "padding-right",
            ("padding", "bottom") => "padding-bottom",
            ("padding", "left") => "padding-left",
            ("border", "top") => "border-top",
            ("border", "right") => "border-right",
            ("border", "bottom") => "border-bottom",
            ("border", "left") => "border-left",
            ("", "top") => "top",
            ("", "right") => "right",
            ("", "bottom") => "bottom",
            ("", "left") => "left",
            _ => continue,
        };
        maybe(attrs, key, dimension(&edges[name]), None);
    }
}

fn logical_inline_margin_attrs(attrs: &mut Vec<(&'static str, String)>, style: &Value) {
    let logical = &style["logicalMargin"];
    if logical.is_null() {
        return;
    }

    let writing_mode = string(style, "writingMode").unwrap_or_else(|| "horizontal-tb".to_string());
    let (start_attr, end_attr) = logical_inline_margin_edges(
        &writing_mode,
        string(style, "direction").as_deref() == Some("rtl"),
    );
    maybe_edge_attr(attrs, start_attr, dimension(&logical["inlineStart"]));
    maybe_edge_attr(attrs, end_attr, dimension(&logical["inlineEnd"]));
}

fn logical_inline_margin_edges(writing_mode: &str, rtl: bool) -> (&'static str, &'static str) {
    match (writing_mode, rtl) {
        ("vertical-rl" | "vertical-lr" | "sideways-rl", false) => ("margin-top", "margin-bottom"),
        ("vertical-rl" | "vertical-lr" | "sideways-rl", true) => ("margin-bottom", "margin-top"),
        ("sideways-lr", false) => ("margin-bottom", "margin-top"),
        ("sideways-lr", true) => ("margin-top", "margin-bottom"),
        (_, false) => ("margin-left", "margin-right"),
        (_, true) => ("margin-right", "margin-left"),
    }
}

fn maybe_edge_attr(
    attrs: &mut Vec<(&'static str, String)>,
    key: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value {
        if let Some((_, existing_value)) = attrs.iter_mut().find(|(existing, _)| *existing == key) {
            *existing_value = value;
        } else {
            attrs.push((key, value));
        }
    }
}

fn attr_text(attrs: &[(&str, String)]) -> String {
    if attrs.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            attrs
                .iter()
                .map(|(key, value)| format!("{key}=\"{}\"", escape_attr(value)))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

fn dimension(value: &Value) -> Option<String> {
    let unit = value.get("unit").and_then(Value::as_str)?;
    match unit {
        "auto" | "none" | "content" | "max-content" | "min-content" | "stretch" | "fit-content"
        | "contain" => Some(unit.to_string()),
        "px" => Some(format!("{}px", number_attr(&value["value"]))),
        "percent" => Some(format!(
            "{}%",
            number_attr_value(number(&value["value"]) * 100.0)
        )),
        "fraction" => Some(format!("{}fr", number_attr(&value["value"]))),
        "calc" | "sizing" => value
            .get("value")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn dimension_or_non_empty_string(value: &Value) -> Option<String> {
    dimension(value).or_else(|| {
        let value = value.as_str()?;
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn dimension_list(values: &Value) -> Option<String> {
    let values = values.as_array()?;
    let serialized = values
        .iter()
        .filter_map(track_definition)
        .collect::<Vec<_>>();
    (!serialized.is_empty()).then(|| serialized.join(" "))
}

fn grid_template_areas(value: &Value) -> Option<String> {
    let rows = value.as_array()?;
    let serialized = rows
        .iter()
        .map(grid_template_area_row)
        .collect::<Option<Vec<_>>>()?;
    (!serialized.is_empty()).then(|| serialized.join(" / "))
}

fn grid_template_area_row(value: &Value) -> Option<String> {
    let cells = value.as_array()?;
    let serialized = cells
        .iter()
        .map(|cell| {
            if cell.is_null() {
                Some(".")
            } else {
                cell.as_str()
            }
        })
        .collect::<Option<Vec<_>>>()?;
    (!serialized.is_empty()).then(|| serialized.join(" "))
}

fn track_definition(value: &Value) -> Option<String> {
    match value.get("kind").and_then(Value::as_str) {
        Some("scalar") | None => dimension(value),
        Some("line-names") => line_names_track_definition(value),
        Some("subgrid") => Some(subgrid_track_definition(value)),
        Some("function") => {
            let name = value["name"].as_str()?;
            let arguments = value["arguments"].as_array()?;
            match name {
                "fit-content" => {
                    let limit = dimension(arguments.first()?)?;
                    Some(format!("fit-content({limit})"))
                }
                "minmax" => {
                    let min = dimension(arguments.first()?)?;
                    let max = dimension(arguments.get(1)?)?;
                    Some(format!("minmax({min},{max})"))
                }
                "repeat" => {
                    let repetition = repetition(arguments.first()?)?;
                    let tracks = arguments
                        .iter()
                        .skip(1)
                        .map(track_definition)
                        .collect::<Option<Vec<_>>>()?
                        .join(" ");
                    Some(format!("repeat({repetition}, {tracks})"))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn line_names_track_definition(value: &Value) -> Option<String> {
    let names = value.get("names")?.as_array()?;
    let names = names
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()?;
    Some(format!("[{}]", names.join(" ")))
}

fn subgrid_track_definition(value: &Value) -> String {
    let mut parts = vec!["subgrid".to_string()];
    if let Some(line_names) = value.get("lineNames").and_then(Value::as_array) {
        parts.extend(line_names.iter().filter_map(|names| {
            let names = names.as_array()?;
            let names = names
                .iter()
                .map(Value::as_str)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("[{}]", names.join(" ")))
        }));
    }
    parts.join(" ")
}

fn repetition(value: &Value) -> Option<String> {
    match value["unit"].as_str()? {
        "auto-fill" => Some("auto-fill".to_string()),
        "auto-fit" => Some("auto-fit".to_string()),
        "integer" => Some(number_attr(&value["value"])),
        _ => None,
    }
}

fn grid_auto_flow(value: &Value) -> Option<String> {
    let direction = value["direction"].as_str()?;
    match value["algorithm"].as_str() {
        Some("dense") => Some(format!("{direction} dense")),
        _ => Some(direction.to_string()),
    }
}

fn grid_position(value: &Value) -> Option<String> {
    match value["kind"].as_str()? {
        "auto" => None,
        "span" => Some(format!("span {}", number_attr(&value["value"]))),
        "line" => Some(number_attr(&value["value"])),
        "named-line" => {
            let name = value["name"].as_str()?;
            let index = value["value"].as_f64()?;
            if index == 0.0 {
                Some(name.to_string())
            } else {
                Some(format!("{name} {}", number_attr(&value["value"])))
            }
        }
        "named-span" => {
            let name = value["name"].as_str()?;
            let span = value["value"].as_f64()?;
            if span == 0.0 {
                Some(format!("span {name}"))
            } else {
                Some(format!("span {} {name}", number_attr(&value["value"])))
            }
        }
        _ => None,
    }
}

fn string(value: &Value, key: &str) -> Option<String> {
    value[key].as_str().map(ToString::to_string)
}

fn font_family(value: &Value) -> Option<String> {
    let family = string(value, "fontFamily")?.replace('"', "");
    let primary = family.split(',').next()?.trim().to_ascii_lowercase();
    match primary.as_str() {
        "ahem" | "monospace" => Some(primary),
        _ => None,
    }
}

fn number_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|value| {
        if value.is_null() {
            None
        } else {
            Some(number_attr(value))
        }
    })
}

fn non_default_overflow(style: &Value, key: &str) -> bool {
    string(style, key).is_some_and(|value| value != "visible")
}

fn bool_field(value: &Value, key: &str) -> bool {
    value[key].as_bool().unwrap_or(false)
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or(0.0)
}

fn number_attr(value: &Value) -> String {
    number_attr_value(number(value))
}

fn number_attr_value(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn layout_number_attr(value: &Value) -> String {
    layout_number_attr_value(number(value))
}

fn layout_number_attr_value(value: f64) -> String {
    // Browser parity layout geometry is serialized through an f32-compatible
    // boundary on purpose. Layout can run f64 lanes, but these generated XML
    // fixtures target the default layout::Scalar precision.
    let value = value as f32;
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

pub(super) fn escape_attr(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn escape_text(value: impl AsRef<str>) -> String {
    value.as_ref().replace('&', "&amp;").replace('<', "&lt;")
}

#[cfg(test)]
#[path = "../../layout/browser_parity/support.rs"]
mod browser_parity_support;

#[cfg(test)]
#[path = "semantic_tests.rs"]
mod tests;
