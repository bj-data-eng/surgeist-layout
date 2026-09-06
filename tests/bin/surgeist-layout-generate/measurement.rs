//! Closed browser wire data and its validated layout-owned representation.

#[path = "measurement/style.rs"]
mod style;

use serde::Deserialize;
use std::fmt;
use style::{Dimension, Style};

pub(super) type Attributes = Vec<(&'static str, String)>;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum MeasurementErrorKind {
    Decode,
    MissingField,
    InvalidValue,
    ContradictoryFields,
    OutOfRange,
}

#[derive(Debug)]
pub(super) struct MeasurementError {
    pub case_id: String,
    pub variant: String,
    pub node_path: String,
    pub field_path: String,
    pub kind: MeasurementErrorKind,
    detail: Box<ErrorDetail>,
}
#[derive(Debug)]
struct ErrorDetail {
    message: String,
    source: Option<serde_json::Error>,
}
impl fmt::Display for MeasurementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) {}.{} [{:?}]: {}",
            self.case_id,
            self.variant,
            self.node_path,
            self.field_path,
            self.kind,
            self.detail.message
        )
    }
}
impl std::error::Error for MeasurementError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.detail.source.as_ref().map(|source| source as _)
    }
}

struct Context<'a> {
    case_id: &'a str,
    variant: &'a str,
    node_path: String,
}
impl Context<'_> {
    fn error(
        &self,
        field: impl Into<String>,
        kind: MeasurementErrorKind,
        detail: impl Into<String>,
    ) -> MeasurementError {
        MeasurementError {
            case_id: self.case_id.into(),
            variant: self.variant.into(),
            node_path: self.node_path.clone(),
            field_path: field.into(),
            kind,
            detail: Box::new(ErrorDetail {
                message: detail.into(),
                source: None,
            }),
        }
    }
    fn required<T>(&self, value: Option<T>, field: &str) -> Result<T, MeasurementError> {
        value.ok_or_else(|| {
            self.error(
                field,
                MeasurementErrorKind::MissingField,
                "required measurement field is absent",
            )
        })
    }
    fn ensure(&self, valid: bool, field: &str, detail: &str) -> Result<(), MeasurementError> {
        if valid {
            Ok(())
        } else {
            Err(self.error(field, MeasurementErrorKind::InvalidValue, detail))
        }
    }
    fn compatible(&self, valid: bool, field: &str, detail: &str) -> Result<(), MeasurementError> {
        if valid {
            Ok(())
        } else {
            Err(self.error(field, MeasurementErrorKind::ContradictoryFields, detail))
        }
    }
    fn child(&self, index: usize) -> Context<'_> {
        Context {
            case_id: self.case_id,
            variant: self.variant,
            node_path: format!("{}.children[{index}]", self.node_path),
        }
    }
}

/// Optional fields reject an explicit null unless their wire contract says otherwise.
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireNode {
    #[serde(default, deserialize_with = "present")]
    tag_name: Option<String>,
    layout_input: LayoutInput,
    #[serde(default, deserialize_with = "present")]
    unsupported_reason: Option<String>,
    #[serde(default, deserialize_with = "present")]
    style: Option<Style>,
    #[serde(default, deserialize_with = "present")]
    use_rounding: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    viewport: Option<WireViewport>,
    #[serde(default, deserialize_with = "present")]
    layout_ready_inline_root: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    layout_ready_anonymous_grid_text_wrapper: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    text_content: Option<String>,
    #[serde(default, deserialize_with = "present")]
    unrounded_layout: Option<WireGeometry>,
    #[serde(default, deserialize_with = "present")]
    smart_rounded_layout: Option<WireGeometry>,
    #[serde(default, deserialize_with = "present")]
    naively_rounded_layout: Option<WireGeometry>,
    #[serde(default, deserialize_with = "present")]
    inline_segments: Option<Vec<WireSegment>>,
    #[serde(default, deserialize_with = "present")]
    inline_boundary: Option<WireBoundary>,
    #[serde(default, deserialize_with = "present")]
    line_control_participation: Option<WireControl>,
    #[serde(default, deserialize_with = "present")]
    atomic_inline_participation: Option<WireAtomic>,
    #[serde(default, deserialize_with = "present")]
    shape_bands: Option<Vec<WireBand>>,
    #[serde(default, deserialize_with = "present")]
    fragments: Option<Vec<WireFragment>>,
    #[serde(default, deserialize_with = "present")]
    range_inks: Option<Vec<WireRange>>,
    #[serde(default, deserialize_with = "present")]
    children: Option<Vec<WireNode>>,
}
#[derive(Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum LayoutInput {
    Box,
    InlineText,
    InlineBoundary,
    Unsupported,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireViewport {
    width: Dimension,
    height: Dimension,
    root_context: RootContext,
    #[serde(default, deserialize_with = "present")]
    parent_writing_mode: Option<String>,
    #[serde(default, deserialize_with = "present")]
    parent_direction: Option<String>,
    #[serde(default, deserialize_with = "present")]
    host_inline_size: Option<f64>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum RootContext {
    Root,
    FlexItem,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireGeometry {
    #[serde(default, deserialize_with = "present")]
    x: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    y: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    width: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    height: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    scroll_width: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    scroll_height: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    client_width: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    client_height: Option<f64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireBoundary {
    kind: BoundaryKind,
    #[serde(default, deserialize_with = "present")]
    baseline: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    line_height: Option<f64>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum BoundaryKind {
    Start,
    End,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireControl {
    kind: ControlKind,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ControlKind {
    ForcedBreak,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireSegment {
    id: u64,
    inline_extent: f64,
    inline_baseline: f64,
    inline_line_height: f64,
    bidi_level: u64,
    whitespace_edge: WhitespaceEdge,
    following_break: BreakKind,
    #[serde(default, deserialize_with = "present")]
    replacement_inline_extent: Option<f64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireAtomic {
    bidi_level: u64,
    following_break: BreakKind,
    #[serde(default, deserialize_with = "present")]
    replacement_inline_extent: Option<f64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum WhitespaceEdge {
    Preserve,
    DiscardAtStart,
    DiscardAtEnd,
    DiscardAtBoth,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum BreakKind {
    Prohibited,
    Allowed,
    Mandatory,
    AllowedWithReplacement,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireBand {
    band_minimum: f64,
    band_maximum: f64,
    #[serde(default, deserialize_with = "present")]
    interval: Option<WireInterval>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireInterval {
    minimum: f64,
    maximum: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireFragment {
    source_segment_id: u64,
    line_index: u64,
    visual_index: u64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    baseline_x: f64,
    baseline_y: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireRange {
    source_segment_id: u64,
    line_index: u64,
    physical_start_edge: PhysicalEdge,
    start: f64,
    advance: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PhysicalEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug)]
pub(super) enum DecodedMeasurement {
    Unsupported { reason: String },
    Supported(ValidatedMeasurement),
}
#[derive(Debug)]
pub(super) struct ValidatedMeasurement {
    use_rounding: bool,
    viewport: Attributes,
    root: Box<Node>,
}
#[derive(Debug)]
pub(super) struct Node {
    kind: NodeKind,
    attrs: Attributes,
    children: Vec<Node>,
    text: Option<String>,
    observation: Observation,
    atomic: Option<Attributes>,
    bands: Option<Vec<Attributes>>,
    control_expectation: Option<Attributes>,
}
#[derive(Debug)]
pub(super) enum NodeKind {
    Box,
    Text,
    InlineText(Vec<Attributes>),
    Boundary(Attributes),
    Control,
}
#[derive(Debug)]
pub(super) enum Observation {
    Geometry {
        unrounded: Geometry,
        selected: Geometry,
        scroll: Option<(f64, f64)>,
        fragments: Option<Vec<Attributes>>,
    },
    RangeInks(Vec<Attributes>),
    Boundary,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Geometry {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub(super) fn decode(
    raw: &str,
    case_id: &str,
    variant: &str,
) -> Result<DecodedMeasurement, MeasurementError> {
    let mut deserializer = serde_json::Deserializer::from_str(raw);
    let wire: WireNode = serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
        let (node_path, field_path) = split_decode_path(&error.path().to_string());
        let source = error.into_inner();
        MeasurementError {
            case_id: case_id.into(),
            variant: variant.into(),
            node_path,
            field_path,
            kind: MeasurementErrorKind::Decode,
            detail: Box::new(ErrorDetail {
                message: source.to_string(),
                source: Some(source),
            }),
        }
    })?;
    deserializer.end().map_err(|source| MeasurementError {
        case_id: case_id.into(),
        variant: variant.into(),
        node_path: "root".into(),
        field_path: String::new(),
        kind: MeasurementErrorKind::Decode,
        detail: Box::new(ErrorDetail {
            message: source.to_string(),
            source: Some(source),
        }),
    })?;
    let context = Context {
        case_id,
        variant,
        node_path: "root".into(),
    };
    wire.validate_wire(&context)?;
    if let Some(reason) = wire.unsupported_reason() {
        return Ok(DecodedMeasurement::Unsupported {
            reason: reason.into(),
        });
    }
    let use_rounding = context.required(wire.use_rounding, "useRounding")?;
    let viewport = wire
        .viewport
        .as_ref()
        .ok_or_else(|| {
            context.error(
                "viewport",
                MeasurementErrorKind::MissingField,
                "root requires viewport",
            )
        })?
        .validate(&context)?;
    let root = wire.validate(&context, "horizontal-tb", use_rounding, true)?;
    Ok(DecodedMeasurement::Supported(ValidatedMeasurement {
        use_rounding,
        viewport,
        root: Box::new(root),
    }))
}

pub(super) fn number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
fn finite(value: f64, context: &Context<'_>, field: &str) -> Result<f64, MeasurementError> {
    context.ensure(value.is_finite(), field, "number must be finite")?;
    Ok(value)
}
fn nonnegative(value: f64, context: &Context<'_>, field: &str) -> Result<f64, MeasurementError> {
    finite(value, context, field)?;
    context.ensure(value >= 0.0, field, "extent must be nonnegative")?;
    Ok(value)
}
fn layout_number(value: f64, context: &Context<'_>, field: &str) -> Result<f64, MeasurementError> {
    finite(value, context, field)?;
    if (value as f32).is_finite() {
        Ok(value)
    } else {
        Err(context.error(
            field,
            MeasurementErrorKind::OutOfRange,
            "geometry must be representable as f32",
        ))
    }
}
fn bidi(value: u64, context: &Context<'_>, field: &str) -> Result<String, MeasurementError> {
    if value <= 125 {
        Ok(value.to_string())
    } else {
        Err(context.error(
            field,
            MeasurementErrorKind::OutOfRange,
            "bidi level exceeds 125",
        ))
    }
}

impl WireNode {
    fn unsupported_reason(&self) -> Option<&str> {
        self.unsupported_reason.as_deref().or_else(|| {
            self.children
                .as_deref()
                .unwrap_or_default()
                .iter()
                .find_map(Self::unsupported_reason)
        })
    }
    fn validate_wire(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        if let Some(reason) = &self.unsupported_reason {
            context.compatible(
                matches!(
                    self.layout_input,
                    LayoutInput::Box | LayoutInput::Unsupported
                ),
                "unsupportedReason",
                "only measured boxes or unsupported descriptors carry reasons",
            )?;
            context.ensure(
                !reason.trim().is_empty(),
                "unsupportedReason",
                "unsupported reason must be nonempty",
            )?;
        }
        if self.layout_input == LayoutInput::Unsupported {
            context.required(self.unsupported_reason.as_ref(), "unsupportedReason")?;
            context.compatible(
                self.tag_name.is_none()
                    && self.style.is_none()
                    && self.use_rounding.is_none()
                    && self.viewport.is_none()
                    && self.layout_ready_inline_root.is_none()
                    && self.layout_ready_anonymous_grid_text_wrapper.is_none()
                    && self.text_content.is_none()
                    && self.unrounded_layout.is_none()
                    && self.smart_rounded_layout.is_none()
                    && self.naively_rounded_layout.is_none()
                    && self.inline_segments.is_none()
                    && self.inline_boundary.is_none()
                    && self.line_control_participation.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.shape_bands.is_none()
                    && self.fragments.is_none()
                    && self.range_inks.is_none()
                    && self.children.is_none(),
                "layoutInput",
                "unsupported descriptors carry only a reason",
            )?;
        }
        if self.layout_input == LayoutInput::Box {
            context.compatible(
                self.inline_segments.is_none()
                    && self.inline_boundary.is_none()
                    && self.range_inks.is_none(),
                "layoutInput",
                "box roles cannot carry inline text or boundary payload",
            )?;
        }
        if self.layout_input == LayoutInput::InlineText {
            if self.range_inks.is_some() {
                context.compatible(self.fragments.is_none() && self.unrounded_layout.is_none() && self.smart_rounded_layout.is_none() && self.naively_rounded_layout.is_none() && self.children.as_ref().is_none_or(Vec::is_empty) && self.style.as_ref().is_none_or(|style| !style.has_scroll_fields()), "rangeInks", "Range observations cannot carry geometry, fragments, children, or scroll state")?;
            }
            context.compatible(
                self.line_control_participation.is_none(),
                "lineControlParticipation",
                "inline text cannot carry line-control participation",
            )?;
            context.compatible(
                self.inline_boundary.is_none()
                    && self.tag_name.is_none()
                    && self.style.is_none()
                    && self.text_content.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.shape_bands.is_none()
                    && self.layout_ready_inline_root.is_none()
                    && self.layout_ready_anonymous_grid_text_wrapper.is_none()
                    && self.children.as_ref().is_none_or(Vec::is_empty),
                "layoutInput",
                "inline text cannot carry box or boundary payload",
            )?;
        }
        if self.layout_input == LayoutInput::InlineBoundary {
            context.compatible(
                self.tag_name.is_none()
                    && self.style.is_none()
                    && self.use_rounding.is_none()
                    && self.viewport.is_none()
                    && self.layout_ready_inline_root.is_none()
                    && self.layout_ready_anonymous_grid_text_wrapper.is_none()
                    && self.text_content.is_none()
                    && self.unrounded_layout.is_none()
                    && self.smart_rounded_layout.is_none()
                    && self.naively_rounded_layout.is_none()
                    && self.inline_segments.is_none()
                    && self.line_control_participation.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.shape_bands.is_none()
                    && self.fragments.is_none()
                    && self.range_inks.is_none()
                    && self.children.as_ref().is_none_or(Vec::is_empty),
                "inlineBoundary",
                "inline boundaries cannot carry box, text, or observation payload",
            )?;
        }
        if let Some(style) = &self.style {
            style.validate_wire(context)?;
        }
        if let Some(viewport) = &self.viewport {
            viewport
                .width
                .validate_viewport_wire(context, "viewport.width")?;
            viewport
                .height
                .validate_viewport_wire(context, "viewport.height")?;
            viewport.validate_alternative(context)?;
        }
        if let Some(boundary) = &self.inline_boundary {
            boundary.validate_alternative(context)?;
        }
        for (index, segment) in self
            .inline_segments
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            validate_break_alternative(
                &segment.following_break,
                segment.replacement_inline_extent,
                context,
                &format!("inlineSegments[{index}]"),
            )?;
        }
        if let Some(atomic) = &self.atomic_inline_participation {
            atomic.validate_alternative(context)?;
        }
        for (index, child) in self
            .children
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            child.validate_wire(&context.child(index))?;
        }
        Ok(())
    }
    fn validate(
        self,
        context: &Context<'_>,
        parent_writing_mode: &str,
        use_rounding: bool,
        is_root: bool,
    ) -> Result<Node, MeasurementError> {
        let mut attrs = Vec::new();
        if let Some(tag) = &self.tag_name {
            validate_xml_text(tag, context, "tagName")?;
        }
        if let Some(text) = &self.text_content {
            validate_xml_text(text, context, "textContent")?;
        }
        let children = context.required(self.children, "children")?;
        let style = self.style.unwrap_or_default();
        let writing_mode = style.writing_mode().to_string();
        if self.layout_input == LayoutInput::InlineBoundary {
            let boundary = context.required(self.inline_boundary, "inlineBoundary")?;
            return Ok(Node {
                kind: NodeKind::Boundary(boundary.validate(context)?),
                attrs,
                children: Vec::new(),
                text: None,
                observation: Observation::Boundary,
                atomic: None,
                bands: None,
                control_expectation: None,
            });
        }
        context.compatible(
            self.inline_boundary.is_none(),
            "inlineBoundary",
            "boundary descriptor requires inline-boundary role",
        )?;
        let inline_text = self.layout_input == LayoutInput::InlineText;
        context.compatible(
            inline_text || (self.inline_segments.is_none() && self.range_inks.is_none()),
            "layoutInput",
            "text segments and Range observations require inline-text role",
        )?;
        let control = self.line_control_participation.is_some();
        let display = style.display.as_deref();
        let is_br = self.tag_name.as_deref() == Some("br");
        let br_control = is_br && matches!(display, Some("inline" | "none"));
        let br_box = is_br && display == Some("block");
        if let Some(WireControl {
            kind: ControlKind::ForcedBreak,
        }) = &self.line_control_participation
        {
            context.compatible(
                !inline_text && is_br && display == Some("inline") && children.is_empty(),
                "lineControlParticipation",
                "forced break requires an inline BR without children",
            )?;
        }
        context.compatible(
            !br_box || self.atomic_inline_participation.is_none(),
            "atomicInlineParticipation",
            "block BR cannot participate as an atomic inline",
        )?;
        context.compatible(
            !inline_text
                || (self.tag_name.is_none()
                    && children.is_empty()
                    && self.text_content.is_none()
                    && self.line_control_participation.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.shape_bands.is_none()
                    && self.layout_ready_inline_root.is_none()
                    && self.layout_ready_anonymous_grid_text_wrapper.is_none()),
            "layoutInput",
            "inline text cannot carry box participation or children",
        )?;
        if (!is_br || br_control || br_box)
            && let Some(tag) = self.tag_name
        {
            attrs.push(("source-tag", tag));
        }
        if let Some(marker) = self.layout_ready_inline_root {
            context.ensure(
                marker,
                "layoutReadyInlineRoot",
                "marker must be true when present",
            )?;
            attrs.push(("layout-ready-inline-root", "true".into()));
        }
        if let Some(marker) = self.layout_ready_anonymous_grid_text_wrapper {
            context.ensure(
                marker,
                "layoutReadyAnonymousGridTextWrapper",
                "marker must be true when present",
            )?;
            context.compatible(
                style.is_grid()
                    && !children.is_empty()
                    && children.iter().all(|child| {
                        child.layout_input == LayoutInput::InlineText
                            && child.children.as_ref().is_none_or(Vec::is_empty)
                    })
                    && self.text_content.is_none(),
                "layoutReadyAnonymousGridTextWrapper",
                "anonymous grid text wrapper requires only direct typed text",
            )?;
            attrs.push(("layout-ready-anonymous-grid-text-wrapper", "true".into()));
        }
        if control {
            attrs.push(("line-control", "forced-break".into()));
        }
        let shape = self
            .shape_bands
            .map(|bands| {
                context.ensure(
                    !bands.is_empty(),
                    "shapeBands",
                    "shape bands must be nonempty",
                )?;
                let mut queries = Vec::new();
                bands
                    .into_iter()
                    .enumerate()
                    .map(|(index, band)| {
                        let query = (band.band_minimum, band.band_maximum);
                        context.ensure(
                            !queries.contains(&query),
                            &format!("shapeBands[{index}]"),
                            "shape query endpoints must be unique",
                        )?;
                        queries.push(query);
                        band.validate(context, &format!("shapeBands[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        attrs.extend(style.attrs(
            context,
            parent_writing_mode,
            shape.is_some(),
            !is_br || br_control,
        )?);
        context.compatible(
            !is_root || self.atomic_inline_participation.is_none(),
            "atomicInlineParticipation",
            "atomic participation requires a parent input slot",
        )?;
        let atomic = self
            .atomic_inline_participation
            .map(|value| value.validate(context))
            .transpose()?;
        let kind = if inline_text {
            let segments = context.required(self.inline_segments, "inlineSegments")?;
            context.ensure(
                !segments.is_empty(),
                "inlineSegments",
                "inline text requires at least one segment",
            )?;
            let mut seen = std::collections::BTreeSet::new();
            let segments = segments
                .into_iter()
                .enumerate()
                .map(|(index, segment)| {
                    context.ensure(
                        seen.insert(segment.id),
                        &format!("inlineSegments[{index}].id"),
                        "segment identifier must be unique within its text node",
                    )?;
                    segment.validate(context, &format!("inlineSegments[{index}]"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            NodeKind::InlineText(segments)
        } else if control {
            NodeKind::Control
        } else if self.text_content.is_some()
            && !style.is_grid()
            && !children
                .iter()
                .any(|child| child.layout_input == LayoutInput::InlineText)
        {
            NodeKind::Text
        } else {
            NodeKind::Box
        };
        let observation = if let Some(ranges) = self.range_inks {
            context.compatible(
                self.fragments.is_none()
                    && self.unrounded_layout.is_none()
                    && self.smart_rounded_layout.is_none()
                    && self.naively_rounded_layout.is_none()
                    && !style.has_scroll_fields(),
                "rangeInks",
                "Range observations cannot carry geometry, fragments, or scroll state",
            )?;
            Observation::RangeInks(
                ranges
                    .into_iter()
                    .enumerate()
                    .map(|(index, range)| range.validate(context, &format!("rangeInks[{index}]")))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        } else {
            let unrounded = context.required(self.unrounded_layout, "unroundedLayout")?;
            let geometry = unrounded.validate(context, "unroundedLayout")?;
            let selected_scroll = if use_rounding {
                self.smart_rounded_layout
                    .as_ref()
                    .map(|value| (value.scroll_width, value.scroll_height))
            } else {
                Some((unrounded.scroll_width, unrounded.scroll_height))
            };
            let rounded = self
                .smart_rounded_layout
                .as_ref()
                .map(|value| value.validate(context, "smartRoundedLayout"))
                .transpose()?;
            let selected = if use_rounding {
                context.required(rounded, "smartRoundedLayout")?
            } else {
                geometry
            };
            if br_box {
                attrs.retain(|(key, _)| !matches!(*key, "width" | "height"));
                let index = attrs
                    .iter()
                    .position(|(key, _)| {
                        matches!(
                            *key,
                            "min-width"
                                | "min-height"
                                | "max-width"
                                | "max-height"
                                | "aspect-ratio"
                                | "row-gap"
                                | "column-gap"
                                | "top"
                                | "right"
                                | "bottom"
                                | "left"
                        ) || key.starts_with("margin-")
                            || key.starts_with("padding-")
                            || key.starts_with("border-")
                            || key.starts_with("grid-")
                    })
                    .unwrap_or(attrs.len());
                attrs.splice(
                    index..index,
                    [
                        ("width", format!("{}px", number(geometry.width))),
                        ("height", format!("{}px", number(geometry.height))),
                    ],
                );
            }
            let scroll = if style.has_scroll_overflow() {
                let naive =
                    context.required(self.naively_rounded_layout, "naivelyRoundedLayout")?;
                naive.validate_supplied(context, "naivelyRoundedLayout")?;
                let (scroll_width, scroll_height) = selected_scroll.ok_or_else(|| {
                    context.error(
                        "smartRoundedLayout",
                        MeasurementErrorKind::MissingField,
                        "rounded scroll observations are absent",
                    )
                })?;
                let sw = context.required(
                    scroll_width,
                    if use_rounding {
                        "smartRoundedLayout.scrollWidth"
                    } else {
                        "unroundedLayout.scrollWidth"
                    },
                )?;
                let sh = context.required(
                    scroll_height,
                    if use_rounding {
                        "smartRoundedLayout.scrollHeight"
                    } else {
                        "unroundedLayout.scrollHeight"
                    },
                )?;
                let cw =
                    context.required(naive.client_width, "naivelyRoundedLayout.clientWidth")?;
                let ch =
                    context.required(naive.client_height, "naivelyRoundedLayout.clientHeight")?;
                for (field, value) in [
                    ("unroundedLayout.scrollWidth", sw),
                    ("unroundedLayout.scrollHeight", sh),
                    ("naivelyRoundedLayout.clientWidth", cw),
                    ("naivelyRoundedLayout.clientHeight", ch),
                ] {
                    nonnegative(value, context, field)?;
                }
                Some((
                    layout_number((sw - cw).max(0.0), context, "scrollWidth")?,
                    layout_number((sh - ch).max(0.0), context, "scrollHeight")?,
                ))
            } else {
                if let Some(naive) = self.naively_rounded_layout {
                    naive.validate_supplied(context, "naivelyRoundedLayout")?;
                }
                None
            };
            let fragments = self
                .fragments
                .map(|fragments| {
                    fragments
                        .into_iter()
                        .enumerate()
                        .map(|(index, fragment)| {
                            fragment.validate(context, &format!("fragments[{index}]"))
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            Observation::Geometry {
                unrounded: geometry,
                selected,
                scroll,
                fragments,
            }
        };
        let has_typed_text = children
            .iter()
            .any(|child| child.layout_input == LayoutInput::InlineText);
        let mut children = children
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                child.validate(&context.child(index), &writing_mode, use_rounding, false)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !is_root && let Some(viewport) = self.viewport {
            viewport.validate(context)?;
        }
        prepare_control_expectations(&writing_mode, &mut children, context)?;
        Ok(Node {
            kind,
            attrs,
            children,
            text: if has_typed_text {
                None
            } else {
                self.text_content
            },
            observation,
            atomic,
            bands: shape,
            control_expectation: None,
        })
    }
}
impl WireGeometry {
    fn validate(&self, context: &Context<'_>, prefix: &str) -> Result<Geometry, MeasurementError> {
        self.validate_supplied(context, prefix)?;
        let field = |name: &str| format!("{prefix}.{name}");
        Ok(Geometry {
            x: layout_number(context.required(self.x, &field("x"))?, context, &field("x"))?,
            y: layout_number(context.required(self.y, &field("y"))?, context, &field("y"))?,
            width: layout_number(
                nonnegative(
                    context.required(self.width, &field("width"))?,
                    context,
                    &field("width"),
                )?,
                context,
                &field("width"),
            )?,
            height: layout_number(
                nonnegative(
                    context.required(self.height, &field("height"))?,
                    context,
                    &field("height"),
                )?,
                context,
                &field("height"),
            )?,
        })
    }
    fn validate_supplied(
        &self,
        context: &Context<'_>,
        prefix: &str,
    ) -> Result<(), MeasurementError> {
        for (name, value) in [
            ("x", self.x),
            ("y", self.y),
            ("width", self.width),
            ("height", self.height),
            ("scrollWidth", self.scroll_width),
            ("scrollHeight", self.scroll_height),
            ("clientWidth", self.client_width),
            ("clientHeight", self.client_height),
        ] {
            if let Some(value) = value {
                finite(value, context, &format!("{prefix}.{name}"))?;
                if !matches!(name, "x" | "y") {
                    nonnegative(value, context, &format!("{prefix}.{name}"))?;
                }
            }
        }
        Ok(())
    }
}
impl WireViewport {
    fn validate_alternative(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        if self.root_context == RootContext::FlexItem {
            let mode = context.required(
                self.parent_writing_mode.as_deref(),
                "viewport.parentWritingMode",
            )?;
            style::validate_writing_mode(mode, context, "viewport.parentWritingMode")?;
            let direction =
                context.required(self.parent_direction.as_deref(), "viewport.parentDirection")?;
            context.ensure(
                matches!(direction, "ltr" | "rtl"),
                "viewport.parentDirection",
                "unknown direction",
            )?;
            context.required(self.host_inline_size, "viewport.hostInlineSize")?;
        } else {
            context.compatible(
                self.parent_writing_mode.is_none()
                    && self.parent_direction.is_none()
                    && self.host_inline_size.is_none(),
                "viewport.rootContext",
                "root context cannot carry flex host metadata",
            )?;
        }
        Ok(())
    }
    fn validate(&self, context: &Context<'_>) -> Result<Attributes, MeasurementError> {
        self.width
            .validate_viewport_semantic(context, "viewport.width")?;
        self.height
            .validate_viewport_semantic(context, "viewport.height")?;
        let mut attrs = vec![
            ("width", self.width.validate(context, "viewport.width")?),
            ("height", self.height.validate(context, "viewport.height")?),
        ];
        if self.root_context == RootContext::FlexItem {
            let mode = context.required(
                self.parent_writing_mode.as_deref(),
                "viewport.parentWritingMode",
            )?;
            style::validate_writing_mode(mode, context, "viewport.parentWritingMode")?;
            let direction =
                context.required(self.parent_direction.as_deref(), "viewport.parentDirection")?;
            context.ensure(
                matches!(direction, "ltr" | "rtl"),
                "viewport.parentDirection",
                "unknown direction",
            )?;
            let host = nonnegative(
                context.required(self.host_inline_size, "viewport.hostInlineSize")?,
                context,
                "viewport.hostInlineSize",
            )?;
            attrs.extend([
                ("root-context", "flex-item".into()),
                ("parent-writing-mode", mode.into()),
                ("parent-direction", direction.into()),
                ("host-inline-size", format!("{}px", number(host))),
            ]);
        } else {
            context.compatible(
                self.parent_writing_mode.is_none()
                    && self.parent_direction.is_none()
                    && self.host_inline_size.is_none(),
                "viewport.rootContext",
                "root context cannot carry flex host metadata",
            )?;
        }
        Ok(attrs)
    }
}
impl WireBoundary {
    fn validate_alternative(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        context.compatible(
            self.baseline.is_some() == self.line_height.is_some(),
            "inlineBoundary",
            "boundary metrics must be paired",
        )?;
        context.compatible(
            self.kind == BoundaryKind::Start || self.baseline.is_none(),
            "inlineBoundary.kind",
            "only boundary starts carry metrics",
        )
    }
    fn validate(self, context: &Context<'_>) -> Result<Attributes, MeasurementError> {
        let kind = match self.kind {
            BoundaryKind::Start => "start",
            BoundaryKind::End => "end",
        };
        let mut attrs = vec![("kind", kind.into())];
        match (self.baseline, self.line_height) {
            (None, None) => {}
            (Some(baseline), Some(height)) => {
                context.compatible(
                    self.kind == BoundaryKind::Start,
                    "inlineBoundary.kind",
                    "only boundary starts carry metrics",
                )?;
                nonnegative(baseline, context, "inlineBoundary.baseline")?;
                nonnegative(height, context, "inlineBoundary.lineHeight")?;
                context.ensure(
                    height > 0.0 && baseline <= height,
                    "inlineBoundary.lineHeight",
                    "boundary strut must be positive and cover its baseline",
                )?;
                attrs.extend([
                    ("inline-baseline", number(baseline)),
                    ("inline-line-height", number(height)),
                ]);
            }
            _ => {
                return Err(context.error(
                    "inlineBoundary",
                    MeasurementErrorKind::ContradictoryFields,
                    "boundary metrics must be paired",
                ));
            }
        }
        Ok(attrs)
    }
}
fn break_attrs(
    kind: BreakKind,
    replacement: Option<f64>,
    context: &Context<'_>,
    prefix: &str,
) -> Result<Attributes, MeasurementError> {
    validate_break_alternative(&kind, replacement, context, prefix)?;
    let name = match kind {
        BreakKind::Prohibited => "prohibited",
        BreakKind::Allowed => "allowed",
        BreakKind::Mandatory => "mandatory",
        BreakKind::AllowedWithReplacement => "allowed-with-replacement",
    };
    let mut attrs = vec![("following-break", name.into())];
    if let Some(value) = replacement {
        attrs.push((
            "replacement-inline-extent",
            number(nonnegative(
                value,
                context,
                &format!("{prefix}.replacementInlineExtent"),
            )?),
        ));
    }
    Ok(attrs)
}
impl WireSegment {
    fn validate(self, context: &Context<'_>, prefix: &str) -> Result<Attributes, MeasurementError> {
        let field = |name: &str| format!("{prefix}.{name}");
        let mut attrs = vec![
            ("id", self.id.to_string()),
            (
                "inline-extent",
                number(nonnegative(
                    self.inline_extent,
                    context,
                    &field("inlineExtent"),
                )?),
            ),
            (
                "inline-baseline",
                number(nonnegative(
                    self.inline_baseline,
                    context,
                    &field("inlineBaseline"),
                )?),
            ),
            (
                "inline-line-height",
                number(nonnegative(
                    self.inline_line_height,
                    context,
                    &field("inlineLineHeight"),
                )?),
            ),
            (
                "bidi-level",
                bidi(self.bidi_level, context, &field("bidiLevel"))?,
            ),
            (
                "whitespace-edge",
                match self.whitespace_edge {
                    WhitespaceEdge::Preserve => "preserve",
                    WhitespaceEdge::DiscardAtStart => "discard-at-start",
                    WhitespaceEdge::DiscardAtEnd => "discard-at-end",
                    WhitespaceEdge::DiscardAtBoth => "discard-at-both",
                }
                .into(),
            ),
        ];
        context.ensure(
            self.inline_baseline <= self.inline_line_height,
            &field("inlineLineHeight"),
            "line height must cover the baseline",
        )?;
        attrs.extend(break_attrs(
            self.following_break,
            self.replacement_inline_extent,
            context,
            prefix,
        )?);
        Ok(attrs)
    }
}
impl WireAtomic {
    fn validate_alternative(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        context.compatible(
            self.following_break != BreakKind::AllowedWithReplacement
                && self.replacement_inline_extent.is_none(),
            "atomicInlineParticipation",
            "atomic breaks cannot carry text replacements",
        )
    }
    fn validate(self, context: &Context<'_>) -> Result<Attributes, MeasurementError> {
        context.compatible(
            self.following_break != BreakKind::AllowedWithReplacement
                && self.replacement_inline_extent.is_none(),
            "atomicInlineParticipation",
            "atomic breaks cannot carry text replacements",
        )?;
        let mut attrs = vec![(
            "bidi-level",
            bidi(
                self.bidi_level,
                context,
                "atomicInlineParticipation.bidiLevel",
            )?,
        )];
        attrs.extend(break_attrs(
            self.following_break,
            self.replacement_inline_extent,
            context,
            "atomicInlineParticipation",
        )?);
        Ok(attrs)
    }
}
impl WireBand {
    fn validate(self, context: &Context<'_>, prefix: &str) -> Result<Attributes, MeasurementError> {
        let mut attrs = vec![
            (
                "band-minimum",
                number(finite(
                    self.band_minimum,
                    context,
                    &format!("{prefix}.bandMinimum"),
                )?),
            ),
            (
                "band-maximum",
                number(finite(
                    self.band_maximum,
                    context,
                    &format!("{prefix}.bandMaximum"),
                )?),
            ),
        ];
        context.ensure(
            self.band_minimum <= self.band_maximum,
            prefix,
            "band interval must not decrease",
        )?;
        match self.interval {
            None => {}
            Some(WireInterval { minimum, maximum }) => {
                finite(minimum, context, &format!("{prefix}.interval.minimum"))?;
                finite(maximum, context, &format!("{prefix}.interval.maximum"))?;
                context.ensure(
                    minimum <= maximum,
                    prefix,
                    "shape interval must not decrease",
                )?;
                attrs.extend([
                    ("interval-minimum", number(minimum)),
                    ("interval-maximum", number(maximum)),
                ]);
            }
        }
        Ok(attrs)
    }
}
impl WireFragment {
    fn validate(self, context: &Context<'_>, prefix: &str) -> Result<Attributes, MeasurementError> {
        let field = |name: &str| format!("{prefix}.{name}");
        Ok(vec![
            ("source_segment_id", self.source_segment_id.to_string()),
            (
                "line_index",
                bounded_index(self.line_index, context, &field("lineIndex"))?,
            ),
            (
                "visual_index",
                bounded_index(self.visual_index, context, &field("visualIndex"))?,
            ),
            ("x", number(finite(self.x, context, &field("x"))?)),
            ("y", number(finite(self.y, context, &field("y"))?)),
            (
                "width",
                number(nonnegative(self.width, context, &field("width"))?),
            ),
            (
                "height",
                number(nonnegative(self.height, context, &field("height"))?),
            ),
            (
                "baseline_x",
                number(finite(self.baseline_x, context, &field("baselineX"))?),
            ),
            (
                "baseline_y",
                number(finite(self.baseline_y, context, &field("baselineY"))?),
            ),
        ])
    }
}
impl WireRange {
    fn validate(self, context: &Context<'_>, prefix: &str) -> Result<Attributes, MeasurementError> {
        let field = |name: &str| format!("{prefix}.{name}");
        Ok(vec![
            ("source_segment_id", self.source_segment_id.to_string()),
            (
                "line_index",
                bounded_index(self.line_index, context, &field("lineIndex"))?,
            ),
            (
                "physical_start_edge",
                match self.physical_start_edge {
                    PhysicalEdge::Left => "left",
                    PhysicalEdge::Right => "right",
                    PhysicalEdge::Top => "top",
                    PhysicalEdge::Bottom => "bottom",
                }
                .into(),
            ),
            (
                "start",
                number(finite(self.start, context, &field("start"))?),
            ),
            (
                "advance",
                number(nonnegative(self.advance, context, &field("advance"))?),
            ),
        ])
    }
}

impl ValidatedMeasurement {
    pub fn use_rounding(&self) -> bool {
        self.use_rounding
    }
    pub fn viewport(&self) -> &Attributes {
        &self.viewport
    }
    pub fn root(&self) -> &Node {
        &self.root
    }
}
impl Node {
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }
    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }
    pub fn children(&self) -> &[Node] {
        &self.children
    }
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }
    pub fn observation(&self) -> &Observation {
        &self.observation
    }
    pub fn atomic(&self) -> Option<&Attributes> {
        self.atomic.as_ref()
    }
    pub fn control_expectation(&self) -> Option<&Attributes> {
        self.control_expectation.as_ref()
    }
    pub fn bands(&self) -> Option<&[Attributes]> {
        self.bands.as_deref()
    }
}
impl Geometry {
    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn height(&self) -> f64 {
        self.height
    }
}

#[cfg(test)]
pub(super) fn style_attributes(raw: &str) -> Result<Attributes, MeasurementError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct StyleFixture {
        #[serde(default, deserialize_with = "present")]
        tag_name: Option<String>,
        style: Style,
    }
    let fixture: StyleFixture = serde_json::from_str(raw).map_err(test_decode_error)?;
    let context = Context {
        case_id: "style-test",
        variant: "test",
        node_path: "root".into(),
    };
    fixture.style.validate_wire(&context)?;
    let mut attrs = fixture
        .tag_name
        .map(|value| vec![("source-tag", value)])
        .unwrap_or_default();
    attrs.extend(
        fixture
            .style
            .attrs(&context, "horizontal-tb", false, true)?,
    );
    Ok(attrs)
}
#[cfg(test)]
fn test_decode_error(source: serde_json::Error) -> MeasurementError {
    MeasurementError {
        case_id: "style-test".into(),
        variant: "test".into(),
        node_path: "root".into(),
        field_path: String::new(),
        kind: MeasurementErrorKind::Decode,
        detail: Box::new(ErrorDetail {
            message: source.to_string(),
            source: Some(source),
        }),
    }
}
#[cfg(test)]
pub(super) fn dimension_attribute(raw: &str) -> Result<String, MeasurementError> {
    let value: Dimension = serde_json::from_str(raw).map_err(test_decode_error)?;
    value.validate(
        &Context {
            case_id: "style-test",
            variant: "test",
            node_path: "root".into(),
        },
        "dimension",
    )
}
#[cfg(test)]
pub(super) fn track_attribute(raw: &str) -> Result<String, MeasurementError> {
    style::test_track(raw)
}
#[cfg(test)]
pub(super) fn position_attribute(raw: &str) -> Result<Option<String>, MeasurementError> {
    style::test_position(raw)
}

#[cfg(test)]
#[path = "measurement/tests.rs"]
mod tests;

/// Project the browser-only line-control facts before the infallible XML phase.
fn prepare_control_expectations(
    writing_mode: &str,
    children: &mut [Node],
    context: &Context<'_>,
) -> Result<(), MeasurementError> {
    let indices = children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| (!matches!(node.kind, NodeKind::Boundary(_))).then_some(index))
        .collect::<Vec<_>>();
    for (source_index, &index) in indices.iter().enumerate() {
        if !matches!(children[index].kind, NodeKind::Control) {
            continue;
        }
        let control = block_interval(writing_mode, &children[index]).ok_or_else(|| {
            context.child(index).error(
                "unroundedLayout",
                MeasurementErrorKind::MissingField,
                "browser controls require geometry",
            )
        })?;
        let candidates = indices[..source_index]
            .iter()
            .rev()
            .take_while(|&&index| !matches!(children[index].kind, NodeKind::Control))
            .map(|&index| &children[index])
            .collect::<Vec<_>>();
        let terminal = candidates
            .iter()
            .all(|node| block_interval(writing_mode, node).is_some())
            .then(|| {
                candidates
                    .iter()
                    .filter(|node| {
                        block_interval(writing_mode, node).is_some_and(|interval| {
                            block_relation(writing_mode, control, interval) == "same"
                        })
                    })
                    .count()
            });
        let previous = source_index
            .checked_sub(1)
            .map(|index| block_interval(writing_mode, &children[indices[index]]))
            .map_or("absent", |interval| {
                interval.map_or("unobserved", |interval| {
                    block_relation(writing_mode, control, interval)
                })
            });
        let next = indices
            .get(source_index + 1)
            .map(|&index| block_interval(writing_mode, &children[index]))
            .map_or("absent", |interval| {
                interval.map_or("unobserved", |interval| {
                    block_relation(writing_mode, control, interval)
                })
            });
        children[index].control_expectation = Some(vec![
            ("source_index", source_index.to_string()),
            (
                "terminal_visual_slot",
                terminal.map_or_else(|| "unobserved".into(), |value| value.to_string()),
            ),
            ("previous_line", previous.into()),
            ("next_line", next.into()),
        ]);
    }
    Ok(())
}
#[derive(Clone, Copy)]
struct BlockInterval {
    minimum: f64,
    maximum: f64,
}
fn block_interval(writing_mode: &str, node: &Node) -> Option<BlockInterval> {
    let Observation::Geometry { unrounded, .. } = node.observation else {
        return None;
    };
    let (minimum, extent) = if writing_mode == "horizontal-tb" {
        (unrounded.y, unrounded.height)
    } else {
        (unrounded.x, unrounded.width)
    };
    // Both operands have already been bounded to finite f32 geometry. Their
    // sum and the center computation below are therefore representable in f64.
    Some(BlockInterval {
        minimum,
        maximum: minimum + extent,
    })
}
fn block_relation(
    writing_mode: &str,
    control: BlockInterval,
    neighbor: BlockInterval,
) -> &'static str {
    if neighbor.maximum >= control.minimum && control.maximum >= neighbor.minimum {
        return "same";
    }
    let control_center = control.minimum + (control.maximum - control.minimum) / 2.0;
    let neighbor_center = neighbor.minimum + (neighbor.maximum - neighbor.minimum) / 2.0;
    let earlier = if matches!(writing_mode, "vertical-rl" | "sideways-rl") {
        neighbor_center > control_center
    } else {
        neighbor_center < control_center
    };
    if earlier { "earlier" } else { "later" }
}

fn split_decode_path(path: &str) -> (String, String) {
    let mut node = "root".to_string();
    let mut remainder = path;
    while let Some(child) = remainder.strip_prefix("children[") {
        let Some(end) = child.find(']') else {
            break;
        };
        node.push_str(".children[");
        node.push_str(&child[..=end]);
        remainder = child[end + 1..]
            .strip_prefix('.')
            .unwrap_or(&child[end + 1..]);
    }
    (node, remainder.to_string())
}

fn bounded_index(
    value: u64,
    context: &Context<'_>,
    field: &str,
) -> Result<String, MeasurementError> {
    usize::try_from(value)
        .map(|index| index.to_string())
        .map_err(|_| {
            context.error(
                field,
                MeasurementErrorKind::OutOfRange,
                "index exceeds the fixture consumer capacity",
            )
        })
}

fn validate_break_alternative(
    kind: &BreakKind,
    replacement: Option<f64>,
    context: &Context<'_>,
    prefix: &str,
) -> Result<(), MeasurementError> {
    context.compatible(
        (*kind == BreakKind::AllowedWithReplacement) == replacement.is_some(),
        &format!("{prefix}.replacementInlineExtent"),
        "replacement extent belongs only to allowed-with-replacement breaks",
    )
}

fn validate_xml_text(
    value: &str,
    context: &Context<'_>,
    field: &str,
) -> Result<(), MeasurementError> {
    context.ensure(value.chars().all(|value|matches!(value,'\u{9}'|'\u{A}'|'\u{D}'|'\u{20}'..='\u{D7FF}'|'\u{E000}'..='\u{FFFD}'|'\u{10000}'..='\u{10FFFF}')),field,"text must contain only XML 1.0 characters")
}
