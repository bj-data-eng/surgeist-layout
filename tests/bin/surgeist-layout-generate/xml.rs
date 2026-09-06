//! Formatting of validated layout measurements into independent fixture XML.

use crate::measurement::{Attributes, Node, NodeKind, Observation, ValidatedMeasurement};

pub(super) fn generate_xml(name: &str, measurement: &ValidatedMeasurement) -> String {
    let mut lines = vec![format!(
        "<test name=\"{}\" use-rounding=\"{}\">",
        escape_attr(name),
        measurement.use_rounding()
    )];
    lines.push(format!(
        "  <viewport{}/>",
        attr_text(measurement.viewport())
    ));
    lines.push("  <input>".into());
    write_input(&mut lines, measurement.root(), 4);
    lines.push("  </input>".into());
    lines.push("  <expectations>".into());
    write_expectation(&mut lines, measurement.root(), 4, true);
    lines.push("  </expectations>".into());
    lines.push("</test>".into());
    format!("{}\n", lines.join("\n"))
}
fn write_input(lines: &mut Vec<String>, node: &Node, indent: usize) {
    let pad = " ".repeat(indent);
    let tag = match node.kind() {
        NodeKind::Boundary(attrs) => {
            lines.push(format!("{pad}<inline-boundary{}/>", attr_text(attrs)));
            return;
        }
        NodeKind::InlineText(segments) => {
            lines.push(format!("{pad}<text layout-input=\"inline-text\">"));
            for attrs in segments {
                lines.push(format!(
                    "{}<segment{}/>",
                    " ".repeat(indent + 2),
                    attr_text(attrs)
                ));
            }
            lines.push(format!("{pad}</text>"));
            return;
        }
        NodeKind::Text => "text",
        NodeKind::Box | NodeKind::Control => "div",
    };
    if node.children().is_empty() && node.text().is_none() && node.bands().is_none() {
        lines.push(format!("{pad}<{tag}{}/>", attr_text(node.attrs())));
        return;
    }
    lines.push(format!("{pad}<{tag}{}>", attr_text(node.attrs())));
    for child in node.children() {
        write_input(lines, child, indent + 2);
    }
    for (index, child) in node.children().iter().enumerate() {
        if let Some(attrs) = child.atomic() {
            let mut indexed = vec![("child-index", index.to_string())];
            indexed.extend_from_slice(attrs);
            lines.push(format!(
                "{}<atomic-placeholder{}/>",
                " ".repeat(indent + 2),
                attr_text(&indexed)
            ));
        }
    }
    if let Some(bands) = node.bands() {
        write_collection(lines, "shape-bands", "shape-band", bands, indent + 2);
    }
    if let Some(text) = node.text() {
        lines.push(format!(
            "{}{}",
            " ".repeat(indent + 2),
            escape_text(text.trim())
        ));
    }
    lines.push(format!("{pad}</{tag}>"));
}
fn write_collection(
    lines: &mut Vec<String>,
    container: &str,
    item: &str,
    rows: &[Attributes],
    indent: usize,
) {
    let pad = " ".repeat(indent);
    if rows.is_empty() {
        lines.push(format!("{pad}<{container}/>"));
        return;
    }
    lines.push(format!("{pad}<{container}>"));
    for attrs in rows {
        lines.push(format!(
            "{}<{item}{}/>",
            " ".repeat(indent + 2),
            attr_text(attrs)
        ));
    }
    lines.push(format!("{pad}</{container}>"));
}
fn write_expectation(lines: &mut Vec<String>, node: &Node, indent: usize, is_root: bool) {
    let pad = " ".repeat(indent);
    let (geometry, scroll, fragments) = match node.observation() {
        Observation::Boundary => return,
        Observation::RangeInks(ranges) => {
            lines.push(format!("{pad}<node>"));
            write_collection(lines, "range-inks", "range-ink", ranges, indent + 2);
            lines.push(format!("{pad}</node>"));
            return;
        }
        Observation::Geometry {
            selected,
            scroll,
            fragments,
            ..
        } => (selected, scroll, fragments),
    };
    let mut attrs = vec![
        (
            "x",
            if is_root {
                "0".into()
            } else {
                layout_number(geometry.x())
            },
        ),
        (
            "y",
            if is_root {
                "0".into()
            } else {
                layout_number(geometry.y())
            },
        ),
        ("width", layout_number(geometry.width())),
        ("height", layout_number(geometry.height())),
    ];
    if let Some((width, height)) = scroll {
        attrs.extend([
            ("scroll_width", layout_number(*width)),
            ("scroll_height", layout_number(*height)),
        ]);
    }
    let children = node
        .children()
        .iter()
        .filter(|child| !matches!(child.kind(), NodeKind::Boundary(_)))
        .collect::<Vec<_>>();
    if children.is_empty() && fragments.is_none() {
        lines.push(format!("{pad}<node{}/>", attr_text(&attrs)));
        return;
    }
    lines.push(format!("{pad}<node{}>", attr_text(&attrs)));
    if let Some(fragments) = fragments {
        write_collection(lines, "fragments", "fragment", fragments, indent + 2);
    }
    for child in children {
        if let Some(attrs) = child.control_expectation() {
            lines.push(format!("{}<node>", " ".repeat(indent + 2)));
            lines.push(format!(
                "{}<browser-control{}/>",
                " ".repeat(indent + 4),
                attr_text(attrs)
            ));
            lines.push(format!("{}</node>", " ".repeat(indent + 2)));
        } else {
            write_expectation(lines, child, indent + 2, false);
        }
    }
    lines.push(format!("{pad}</node>"));
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
fn layout_number(value: f64) -> String {
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
    value
        .as_ref()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace("]]>", "]]&gt;")
}

#[cfg(test)]
#[path = "../../layout/browser_parity/support.rs"]
mod browser_parity_support;
#[cfg(test)]
#[path = "semantic_tests.rs"]
mod tests;
