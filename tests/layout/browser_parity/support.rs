use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use surgeist_layout as layout;
use surgeist_layout::{ComputeInput, ComputeOutput, GridSpan, LayoutTree as _};

type Scalar = layout::Scalar;

#[derive(Clone, Copy, Debug)]
struct ComparisonTolerance {
    value: Scalar,
}

impl ComparisonTolerance {
    const fn browser_parity() -> Self {
        Self { value: 0.1 }
    }

    fn contains(self, delta: Scalar) -> bool {
        delta.abs() <= self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Golden {
    pub name: String,
    pub use_rounding: bool,
    pub viewport: Viewport,
    pub root: Node,
    pub expectations: Expectation,
}

impl Golden {
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let raw = std::fs::read_to_string(path.as_ref())
            .map_err(|source| Error::new(source.to_string()))?;
        Self::parse(&raw)
    }

    pub fn parse(raw: &str) -> Result<Self, Error> {
        let document = roxmltree::Document::parse(raw).map_err(|source| Error {
            message: source.to_string(),
        })?;
        let test = document.root_element();
        expect_tag(test, "test")?;

        let viewport = one_child(test, "viewport")?;
        let input = one_child(test, "input")?;
        let expectations = one_child(test, "expectations")?;

        Ok(Self {
            name: required_attr(test, "name")?.to_string(),
            use_rounding: parse_bool(test.attribute("use-rounding").unwrap_or("true"))?,
            viewport: Viewport {
                width: parse_available(required_attr(viewport, "width")?)?,
                height: parse_available(required_attr(viewport, "height")?)?,
                root_context: parse_root_context(viewport)?,
            },
            root: parse_node(one_element_child(input)?)?,
            expectations: parse_expectation(one_element_child(expectations)?)?,
        })
    }
}

pub fn fixture_files(relative_dir: &str) -> Result<Vec<PathBuf>, Error> {
    let root = fixture_root(relative_dir)?;
    fixture_files_in(&root, "xml")
}

pub fn fixture_files_in(root: &Path, extension: &str) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    collect_files_with_extension(root, extension, &mut files)?;
    files.sort();
    Ok(files)
}

pub const fn fixture_skip_policy_mentions_x_prefix() -> bool {
    false
}

pub const fn fixture_skip_policy_filters_unsupported_constructs() -> bool {
    false
}

fn fixture_root(relative_dir: &str) -> Result<PathBuf, Error> {
    let root = relative_crate_root()
        .ok_or_else(|| Error::new("failed to locate surgeist-layout crate from relative paths"))?;
    Ok(root.join("tests/layout/browser_parity").join(relative_dir))
}

fn relative_crate_root() -> Option<PathBuf> {
    [Path::new("."), Path::new("crates/surgeist-layout")]
        .into_iter()
        .find(|root| root.join("tests/layout/browser_parity/xml").is_dir())
        .map(Path::to_path_buf)
}

pub fn assert_surgeist_matches(golden: &Golden) -> Result<(), Error> {
    let mut tree = TestTree::from_golden(&golden.root)?;
    let available = layout::Size::new(
        to_layout_available(golden.viewport.width),
        to_layout_available(golden.viewport.height),
    );

    let request = root_request(available, golden.viewport.root_context)?;
    let batch = layout::compute_layout(&tree, 0, request)
        .map_err(|error| Error::new(format!("{}: layout failed: {error:?}", golden.name)))?;
    tree.apply_completed_batch(&batch);

    compare_expectation(
        &tree,
        0,
        &golden.expectations,
        &golden.name,
        golden.use_rounding,
        batch.final_inline_fragments(),
    )
}

fn root_request(
    available: layout::Size<layout::Available>,
    root_context: RootContext,
) -> Result<layout::LayoutRootRequest, Error> {
    match root_context {
        RootContext::Root => layout::LayoutRootRequest::viewport(available),
        RootContext::FlexItem {
            parent_axes,
            host_inline_size,
        } => {
            let context = layout::FlexItemRootContext::under_viewport(available, parent_axes)
                .map_err(|error| Error::new(format!("invalid flex viewport: {error:?}")))?;
            let host_available = match parent_axes.inline_axis() {
                layout::PhysicalAxis::Horizontal => layout::Size::new(
                    layout::Available::Definite(host_inline_size),
                    layout::Available::MaxContent,
                ),
                layout::PhysicalAxis::Vertical => layout::Size::new(
                    layout::Available::MaxContent,
                    layout::Available::Definite(host_inline_size),
                ),
            };
            layout::LayoutRootRequest::flex_item_under_viewport(host_available, context)
        }
    }
    .map_err(|error| Error::new(format!("invalid layout root request: {error:?}")))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Available {
    Definite(Scalar),
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: Available,
    pub height: Available,
    pub root_context: RootContext,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RootContext {
    Root,
    FlexItem {
        parent_axes: layout::FlowAxes,
        host_inline_size: Scalar,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub style: StyleAttrs,
    pub text: Option<String>,
    pub children: Vec<Node>,
    inline_text: Option<layout::InlineTextInput>,
    atomic_inline_participation: Option<layout::AtomicInlineParticipation>,
    shape_bands: Option<Vec<FixtureShapeBand>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FixtureShapeBand {
    band_minimum: Scalar,
    band_maximum: Scalar,
    response: FixtureShapeResponse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FixtureShapeResponse {
    Empty,
    Interval {
        minimum: Scalar,
        maximum: Scalar,
        originating_band: Option<(Scalar, Scalar)>,
    },
    Failure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeKind {
    Div,
    Text,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleAttrs {
    attrs: BTreeMap<String, String>,
}

impl StyleAttrs {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attrs.get(name).map(String::as_str)
    }

    pub fn display(&self) -> Option<String> {
        self.get("display").map(ToOwned::to_owned)
    }

    pub fn width(&self) -> Option<String> {
        self.get("width").map(ToOwned::to_owned)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expectation {
    pub x: Option<Scalar>,
    pub y: Option<Scalar>,
    pub width: Option<Scalar>,
    pub height: Option<Scalar>,
    pub scroll_size: Option<Size>,
    pub fragments: Option<Vec<InlineFragmentExpectation>>,
    pub range_inks: Option<Vec<InlineRangeInkExpectation>>,
    pub children: Vec<Expectation>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineFragmentExpectation {
    pub source_segment_id: u64,
    pub line_index: usize,
    pub visual_index: usize,
    pub x: Scalar,
    pub y: Scalar,
    pub width: Scalar,
    pub height: Scalar,
    pub baseline_x: Scalar,
    pub baseline_y: Scalar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalStartEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineRangeInkExpectation {
    pub source_segment_id: u64,
    pub line_index: usize,
    pub visual_index: usize,
    pub physical_start_edge: PhysicalStartEdge,
    pub start: Scalar,
    pub advance: Scalar,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: Scalar,
    pub height: Scalar,
}

impl Size {
    pub const fn new(width: Scalar, height: Scalar) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

fn parse_node(xml: roxmltree::Node<'_, '_>) -> Result<Node, Error> {
    let kind = match xml.tag_name().name() {
        "div" => NodeKind::Div,
        "text" => NodeKind::Text,
        tag => return Err(Error::new(format!("unsupported input node `<{tag}>`"))),
    };

    if let Some(layout_input) = xml.attribute("layout-input") {
        if kind != NodeKind::Text || layout_input != "inline-text" {
            return Err(Error::new(format!(
                "invalid `layout-input` on `<{}>`: `{layout_input}`",
                xml.tag_name().name()
            )));
        }
        return parse_inline_text_node(xml);
    }

    let mut attrs = BTreeMap::new();
    for attr in xml.attributes() {
        attrs.insert(attr.name().to_string(), attr.value().to_string());
    }

    let mut shape_bands = None;
    for table in xml
        .children()
        .filter(roxmltree::Node::is_element)
        .filter(|child| child.has_tag_name("shape-bands"))
    {
        if shape_bands.is_some() {
            return Err(Error::new(
                "expected at most one `<shape-bands>` child on an input node",
            ));
        }
        shape_bands = Some(parse_shape_bands(table)?);
    }

    let text = xml
        .text()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned);
    let mut children = xml
        .children()
        .filter(roxmltree::Node::is_element)
        .filter(|child| {
            !child.has_tag_name("atomic-placeholder") && !child.has_tag_name("shape-bands")
        })
        .map(parse_node)
        .collect::<Result<Vec<_>, _>>()?;
    let mut atomic_indices = std::collections::BTreeSet::new();
    for placeholder in xml
        .children()
        .filter(roxmltree::Node::is_element)
        .filter(|child| child.has_tag_name("atomic-placeholder"))
    {
        let (child_index, participation) = parse_atomic_placeholder(placeholder)?;
        if !atomic_indices.insert(child_index) {
            return Err(Error::new(format!(
                "duplicate atomic child index `{child_index}`"
            )));
        }
        let child = children
            .get_mut(child_index)
            .ok_or_else(|| Error::new(format!("unmatched atomic child index `{child_index}`")))?;
        if child.inline_text.is_some() || child.atomic_inline_participation.is_some() {
            return Err(Error::new(format!(
                "unmatched atomic child index `{child_index}`"
            )));
        }
        child.atomic_inline_participation = Some(participation);
    }

    Ok(Node {
        kind,
        style: StyleAttrs { attrs },
        text,
        children,
        inline_text: None,
        atomic_inline_participation: None,
        shape_bands,
    })
}

fn parse_inline_text_node(xml: roxmltree::Node<'_, '_>) -> Result<Node, Error> {
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| attribute.name() != "layout-input")
    {
        if attribute.name() == "display" {
            return Err(Error::new(
                "inline text must not specify box attribute `display`",
            ));
        }
        return Err(Error::new(format!(
            "unsupported inline text attribute `{}`",
            attribute.name()
        )));
    }
    for child in xml.children() {
        if child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()) {
            return Err(Error::new("unsupported non-whitespace text in inline text"));
        }
        if child.is_element() && !child.has_tag_name("segment") {
            let tag = child.tag_name().name();
            if matches!(tag, "div" | "text") {
                return Err(Error::new(format!(
                    "inline text must not contain layout child `<{tag}>`"
                )));
            }
            return Err(Error::new(format!(
                "unsupported inline text child `<{tag}>`"
            )));
        }
    }
    let segments = xml
        .children()
        .filter(roxmltree::Node::is_element)
        .map(parse_inline_segment)
        .collect::<Result<Vec<_>, _>>()?;
    if segments.is_empty() {
        return Err(Error::new("inline text requires at least one `<segment>`"));
    }
    let inline_text = layout::InlineTextInput::try_new(segments).map_err(|error| match error {
        layout::InlineTextInputError::DuplicateSegmentId { segment_id } => {
            Error::new(format!("duplicate segment id `{}`", segment_id.get()))
        }
        _ => Error::new(format!("invalid inline text input: {error:?}")),
    })?;
    Ok(Node {
        kind: NodeKind::Text,
        style: StyleAttrs::default(),
        text: None,
        children: Vec::new(),
        inline_text: Some(inline_text),
        atomic_inline_participation: None,
        shape_bands: None,
    })
}

fn parse_inline_segment(
    xml: roxmltree::Node<'_, '_>,
) -> Result<layout::ShapedInlineSegment, Error> {
    expect_tag(xml, "segment")?;
    const ATTRIBUTES: &[&str] = &[
        "id",
        "inline-extent",
        "inline-baseline",
        "inline-line-height",
        "bidi-level",
        "whitespace-edge",
        "following-break",
        "replacement-inline-extent",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<segment>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_inline_payload(xml)?;
    let segment_id = layout::InlineSegmentId::new(parse_inline_integer(xml, "id")?);
    let inline_extent = parse_inline_nonnegative_number(xml, "inline-extent")?;
    let baseline = parse_inline_nonnegative_number(xml, "inline-baseline")?;
    let line_height = parse_inline_nonnegative_number(xml, "inline-line-height")?;
    let metrics = layout::InlineMetrics::try_new(baseline, line_height).map_err(|_| {
        Error::new(format!(
            "invalid inline metrics on `<segment>`: baseline `{baseline}`, line height `{line_height}`"
        ))
    })?;
    let bidi_level = parse_bidi_level(xml, "segment")?;
    let whitespace_edge = match required_attr(xml, "whitespace-edge")? {
        "preserve" => layout::InlineWhitespaceEdge::Preserve,
        "discard-at-line-start" => layout::InlineWhitespaceEdge::DiscardAtLineStart,
        "discard-at-line-end" => layout::InlineWhitespaceEdge::DiscardAtLineEnd,
        "discard-at-both" => layout::InlineWhitespaceEdge::DiscardAtBoth,
        raw => {
            return Err(Error::new(format!(
                "invalid `whitespace-edge` on `<segment>`: `{raw}`"
            )));
        }
    };
    let following_break = parse_inline_break(xml, "segment")?;
    layout::ShapedInlineSegment::try_new(
        segment_id,
        inline_extent,
        metrics,
        bidi_level,
        whitespace_edge,
        following_break,
    )
    .map_err(|error| Error::new(format!("invalid segment replacement: {error:?}")))
}

fn parse_atomic_placeholder(
    xml: roxmltree::Node<'_, '_>,
) -> Result<(usize, layout::AtomicInlineParticipation), Error> {
    const ATTRIBUTES: &[&str] = &[
        "child-index",
        "bidi-level",
        "following-break",
        "replacement-inline-extent",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<atomic-placeholder>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_inline_payload(xml)?;
    let child_index = parse_inline_integer(xml, "child-index")?;
    let bidi_level = parse_bidi_level(xml, "atomic-placeholder")?;
    let following_break = parse_inline_break(xml, "atomic-placeholder")?;
    let participation = layout::AtomicInlineParticipation::try_new(bidi_level, following_break)
        .map_err(|_| Error::new("atomic placeholder break replacement is not allowed"))?;
    Ok((child_index, participation))
}

fn parse_inline_break(
    xml: roxmltree::Node<'_, '_>,
    tag: &str,
) -> Result<layout::InlineBreakOpportunity, Error> {
    let raw = required_attr(xml, "following-break")?;
    let replacement = xml.attribute("replacement-inline-extent");
    match (raw, replacement) {
        ("prohibited", None) => Ok(layout::InlineBreakOpportunity::prohibited()),
        ("allowed", None) => Ok(layout::InlineBreakOpportunity::allowed()),
        ("mandatory", None) => Ok(layout::InlineBreakOpportunity::mandatory()),
        ("allowed-with-replacement", Some(_)) => {
            let extent = parse_inline_nonnegative_number(xml, "replacement-inline-extent")?;
            layout::InlineBreakOpportunity::try_allowed_with_replacement(extent)
                .map_err(|_| Error::new("invalid `replacement-inline-extent`"))
        }
        ("allowed-with-replacement", None) => Err(Error::new(
            "missing `replacement-inline-extent` for replacement break",
        )),
        ("prohibited" | "allowed" | "mandatory", Some(_)) => Err(Error::new(
            "replacement-inline-extent requires `allowed-with-replacement`",
        )),
        _ => Err(Error::new(format!(
            "invalid `following-break` on `<{tag}>`: `{raw}`"
        ))),
    }
}

fn parse_bidi_level(xml: roxmltree::Node<'_, '_>, tag: &str) -> Result<layout::BidiLevel, Error> {
    let raw = required_attr(xml, "bidi-level")?;
    let level = parse_ascii_integer::<u8>(raw)
        .and_then(|level| layout::BidiLevel::try_new(level).ok())
        .ok_or_else(|| Error::new(format!("invalid `bidi-level` on `<{tag}>`: `{raw}`")))?;
    Ok(level)
}

fn parse_inline_integer<T>(xml: roxmltree::Node<'_, '_>, name: &str) -> Result<T, Error>
where
    T: std::str::FromStr,
{
    let raw = required_attr(xml, name)?;
    parse_ascii_integer(raw).ok_or_else(|| {
        Error::new(format!(
            "invalid `{name}` on `<{}>`: `{raw}`",
            xml.tag_name().name()
        ))
    })
}

fn parse_ascii_integer<T: std::str::FromStr>(raw: &str) -> Option<T> {
    (!raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| raw.parse().ok())
        .flatten()
}

fn parse_inline_nonnegative_number(
    xml: roxmltree::Node<'_, '_>,
    name: &str,
) -> Result<Scalar, Error> {
    let raw = required_attr(xml, name)?;
    let value = parse_number(raw)?;
    if !value.is_finite() || value < 0.0 {
        return Err(Error::new(format!(
            "invalid `{name}` on `<{}>`: `{raw}`",
            xml.tag_name().name()
        )));
    }
    Ok(value)
}

fn validate_inline_payload(xml: roxmltree::Node<'_, '_>) -> Result<(), Error> {
    let tag = xml.tag_name().name();
    for child in xml.children() {
        if child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()) {
            return Err(Error::new(format!(
                "unsupported non-whitespace text in `<{tag}>`"
            )));
        }
        if child.is_element() {
            return Err(Error::new(format!(
                "unsupported `<{tag}>` child `<{}>`",
                child.tag_name().name()
            )));
        }
    }
    Ok(())
}

fn parse_shape_bands(xml: roxmltree::Node<'_, '_>) -> Result<Vec<FixtureShapeBand>, Error> {
    expect_tag(xml, "shape-bands")?;
    if let Some(attribute) = xml.attributes().next() {
        return Err(Error::new(format!(
            "unsupported `<shape-bands>` attribute `{}`",
            attribute.name()
        )));
    }
    for child in xml.children() {
        if child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()) {
            return Err(Error::new(
                "unsupported non-whitespace text in `<shape-bands>`",
            ));
        }
        if child.is_element() && !child.has_tag_name("shape-band") {
            return Err(Error::new(format!(
                "unsupported `<shape-bands>` child `<{}>`",
                child.tag_name().name()
            )));
        }
    }

    let mut bands = Vec::new();
    for band in xml.children().filter(roxmltree::Node::is_element) {
        let band = parse_shape_band(band)?;
        if bands.iter().any(|existing: &FixtureShapeBand| {
            existing.band_minimum == band.band_minimum && existing.band_maximum == band.band_maximum
        }) {
            return Err(Error::new(format!(
                "duplicate shape query band `{}..{}`",
                band.band_minimum, band.band_maximum
            )));
        }
        bands.push(band);
    }
    Ok(bands)
}

fn parse_shape_band(xml: roxmltree::Node<'_, '_>) -> Result<FixtureShapeBand, Error> {
    expect_tag(xml, "shape-band")?;
    const ATTRIBUTES: &[&str] = &[
        "band-minimum",
        "band-maximum",
        "interval-minimum",
        "interval-maximum",
        "origin-band-minimum",
        "origin-band-maximum",
        "provider-result",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<shape-band>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_inline_payload(xml)?;

    let band_minimum = parse_number(required_attr(xml, "band-minimum")?)?;
    let band_maximum = parse_number(required_attr(xml, "band-maximum")?)?;
    let validation_margin_box =
        layout::ScrollRect::try_new(layout::Point::ZERO, layout::Size::ZERO)
            .expect("zero validation margin box is valid");
    let query = layout::FloatExclusionQuery::try_new(
        validation_margin_box,
        layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr),
        band_minimum,
        band_maximum,
    )
    .map_err(|error| Error::new(format!("invalid shape band query: {error:?}")))?;

    let interval = match (
        xml.attribute("interval-minimum"),
        xml.attribute("interval-maximum"),
    ) {
        (Some(minimum), Some(maximum)) => Some((parse_number(minimum)?, parse_number(maximum)?)),
        (None, None) => None,
        _ => {
            return Err(Error::new("shape interval endpoints must appear together"));
        }
    };
    let originating_band = match (
        xml.attribute("origin-band-minimum"),
        xml.attribute("origin-band-maximum"),
    ) {
        (Some(minimum), Some(maximum)) => {
            let minimum = parse_number(minimum)?;
            let maximum = parse_number(maximum)?;
            layout::FloatExclusionQuery::try_new(
                validation_margin_box,
                query.flow_axes(),
                minimum,
                maximum,
            )
            .map_err(|error| {
                Error::new(format!("invalid originating shape band query: {error:?}"))
            })?;
            Some((minimum, maximum))
        }
        (None, None) => None,
        _ => {
            return Err(Error::new(
                "originating shape band endpoints must appear together",
            ));
        }
    };

    let response = match xml.attribute("provider-result") {
        Some("failure") => {
            if interval.is_some() || originating_band.is_some() {
                return Err(Error::new(
                    "provider failure must not include an exclusion interval",
                ));
            }
            FixtureShapeResponse::Failure
        }
        Some(value) => {
            return Err(Error::new(format!(
                "unsupported shape provider result `{value}`"
            )));
        }
        None => match interval {
            Some((minimum, maximum)) => {
                layout::FloatExclusionInterval::try_new(query, minimum, maximum).map_err(
                    |error| Error::new(format!("invalid shape exclusion interval: {error:?}")),
                )?;
                FixtureShapeResponse::Interval {
                    minimum,
                    maximum,
                    originating_band,
                }
            }
            None if originating_band.is_some() => {
                return Err(Error::new(
                    "originating shape band requires an exclusion interval",
                ));
            }
            None => FixtureShapeResponse::Empty,
        },
    };

    Ok(FixtureShapeBand {
        band_minimum: query.band_minimum(),
        band_maximum: query.band_maximum(),
        response,
    })
}

fn parse_expectation(xml: roxmltree::Node<'_, '_>) -> Result<Expectation, Error> {
    expect_tag(xml, "node")?;
    let scroll_size = match (
        xml.attribute("scroll_width"),
        xml.attribute("scroll_height"),
    ) {
        (Some(width), Some(height)) => Some(Size::new(parse_number(width)?, parse_number(height)?)),
        (None, None) => None,
        _ => {
            return Err(Error::new(
                "expected scroll_width and scroll_height to be present together",
            ));
        }
    };

    let mut fragments = None;
    let mut range_inks = None;
    let mut children = Vec::new();
    for child in xml.children().filter(roxmltree::Node::is_element) {
        match child.tag_name().name() {
            "node" => children.push(parse_expectation(child)?),
            "fragments" if fragments.is_none() => {
                fragments = Some(parse_fragment_expectations(child)?);
            }
            "fragments" => {
                return Err(Error::new(
                    "expected at most one `<fragments>` child on `<node>`",
                ));
            }
            "range-inks" if range_inks.is_none() => {
                range_inks = Some(parse_range_ink_expectations(child)?);
            }
            "range-inks" => {
                return Err(Error::new(
                    "expected at most one `<range-inks>` child on `<node>`",
                ));
            }
            tag => {
                return Err(Error::new(format!(
                    "unsupported expectation child `<{tag}>`"
                )));
            }
        }
    }
    if fragments.is_some() && range_inks.is_some() {
        return Err(Error::new(
            "model fragments and Range ink are distinct expectation categories",
        ));
    }

    Ok(Expectation {
        x: optional_number_attr(xml, "x")?,
        y: optional_number_attr(xml, "y")?,
        width: optional_number_attr(xml, "width")?,
        height: optional_number_attr(xml, "height")?,
        scroll_size,
        fragments,
        range_inks,
        children,
    })
}

fn parse_range_ink_expectations(
    xml: roxmltree::Node<'_, '_>,
) -> Result<Vec<InlineRangeInkExpectation>, Error> {
    expect_tag(xml, "range-inks")?;
    if let Some(attribute) = xml.attributes().next() {
        return Err(Error::new(format!(
            "unsupported `<range-inks>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, Some("range-ink"))?;
    let range_inks = xml
        .children()
        .filter(roxmltree::Node::is_element)
        .map(parse_range_ink_expectation)
        .collect::<Result<Vec<_>, _>>()?;
    if range_inks.is_empty() {
        return Err(Error::new(
            "expected at least one `<range-ink>` child on `<range-inks>`",
        ));
    }
    Ok(range_inks)
}

fn parse_range_ink_expectation(
    xml: roxmltree::Node<'_, '_>,
) -> Result<InlineRangeInkExpectation, Error> {
    expect_tag(xml, "range-ink")?;
    let physical_start_edge = match required_attr(xml, "physical_start_edge")? {
        "left" => PhysicalStartEdge::Left,
        "right" => PhysicalStartEdge::Right,
        "top" => PhysicalStartEdge::Top,
        "bottom" => PhysicalStartEdge::Bottom,
        raw => {
            return Err(Error::new(format!(
                "invalid `physical_start_edge` on `<range-ink>`: `{raw}`"
            )));
        }
    };
    let expectation = InlineRangeInkExpectation {
        source_segment_id: parse_range_ink_integer(xml, "source_segment_id")?,
        line_index: parse_range_ink_integer(xml, "line_index")?,
        visual_index: parse_range_ink_integer(xml, "visual_index")?,
        physical_start_edge,
        start: parse_range_ink_number(xml, "start", false)?,
        advance: parse_range_ink_number(xml, "advance", true)?,
    };
    const ATTRIBUTES: &[&str] = &[
        "source_segment_id",
        "line_index",
        "visual_index",
        "physical_start_edge",
        "start",
        "advance",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<range-ink>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, None)?;
    Ok(expectation)
}

fn parse_range_ink_integer<T>(xml: roxmltree::Node<'_, '_>, name: &str) -> Result<T, Error>
where
    T: std::str::FromStr,
{
    let raw = required_attr(xml, name)?;
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<range-ink>`: `{raw}`"
        )));
    }
    raw.parse()
        .map_err(|_| Error::new(format!("invalid `{name}` on `<range-ink>`: `{raw}`")))
}

fn parse_range_ink_number(
    xml: roxmltree::Node<'_, '_>,
    name: &str,
    nonnegative: bool,
) -> Result<Scalar, Error> {
    let raw = required_attr(xml, name)?;
    let value = parse_number(raw)?;
    if !value.is_finite() || (nonnegative && value < 0.0) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<range-ink>`: `{raw}`"
        )));
    }
    Ok(value)
}

fn parse_fragment_expectations(
    xml: roxmltree::Node<'_, '_>,
) -> Result<Vec<InlineFragmentExpectation>, Error> {
    expect_tag(xml, "fragments")?;
    if let Some(attribute) = xml.attributes().next() {
        return Err(Error::new(format!(
            "unsupported `<fragments>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, Some("fragment"))?;
    xml.children()
        .filter(roxmltree::Node::is_element)
        .map(parse_fragment_expectation)
        .collect()
}

fn parse_fragment_expectation(
    xml: roxmltree::Node<'_, '_>,
) -> Result<InlineFragmentExpectation, Error> {
    expect_tag(xml, "fragment")?;
    let expectation = InlineFragmentExpectation {
        source_segment_id: parse_fragment_integer(xml, "source_segment_id")?,
        line_index: parse_fragment_integer(xml, "line_index")?,
        visual_index: parse_fragment_integer(xml, "visual_index")?,
        x: parse_fragment_number(xml, "x", false)?,
        y: parse_fragment_number(xml, "y", false)?,
        width: parse_fragment_number(xml, "width", true)?,
        height: parse_fragment_number(xml, "height", true)?,
        baseline_x: parse_fragment_number(xml, "baseline_x", false)?,
        baseline_y: parse_fragment_number(xml, "baseline_y", false)?,
    };
    const ATTRIBUTES: &[&str] = &[
        "source_segment_id",
        "line_index",
        "visual_index",
        "x",
        "y",
        "width",
        "height",
        "baseline_x",
        "baseline_y",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<fragment>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, None)?;
    Ok(expectation)
}

fn validate_fragment_payload(
    xml: roxmltree::Node<'_, '_>,
    allowed_child: Option<&str>,
) -> Result<(), Error> {
    let tag = xml.tag_name().name();
    for child in xml.children() {
        if child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()) {
            return Err(Error::new(format!(
                "unsupported non-whitespace text in `<{tag}>`"
            )));
        }
        if child.is_element() && Some(child.tag_name().name()) != allowed_child {
            return Err(Error::new(format!(
                "unsupported `<{tag}>` child `<{}>`",
                child.tag_name().name()
            )));
        }
    }
    Ok(())
}

fn parse_fragment_integer<T>(xml: roxmltree::Node<'_, '_>, name: &str) -> Result<T, Error>
where
    T: std::str::FromStr,
{
    let raw = required_attr(xml, name)?;
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<fragment>`: `{raw}`"
        )));
    }
    raw.parse()
        .map_err(|_| Error::new(format!("invalid `{name}` on `<fragment>`: `{raw}`")))
}

fn parse_fragment_number(
    xml: roxmltree::Node<'_, '_>,
    name: &str,
    nonnegative: bool,
) -> Result<Scalar, Error> {
    let raw = required_attr(xml, name)?;
    let value = parse_number(raw)?;
    if !value.is_finite() || (nonnegative && value < 0.0) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<fragment>`: `{raw}`"
        )));
    }
    Ok(value)
}

fn parse_available(raw: &str) -> Result<Available, Error> {
    match raw {
        "min-content" => Ok(Available::MinContent),
        "max-content" => Ok(Available::MaxContent),
        value => {
            let value = value.strip_suffix("px").unwrap_or(value);
            Ok(Available::Definite(parse_number(value)?))
        }
    }
}

fn parse_root_context(viewport: roxmltree::Node<'_, '_>) -> Result<RootContext, Error> {
    let raw = viewport.attribute("root-context").unwrap_or("root");
    let parent_writing_mode = viewport.attribute("parent-writing-mode");
    let parent_direction = viewport.attribute("parent-direction");
    let host_inline_size = viewport.attribute("host-inline-size");

    match raw {
        "root" => {
            if parent_writing_mode.is_some()
                || parent_direction.is_some()
                || host_inline_size.is_some()
            {
                return Err(Error::new(
                    "root viewport must not specify flex-item metadata",
                ));
            }
            Ok(RootContext::Root)
        }
        "flex-item" => {
            let writing_mode =
                parse_writing_mode(Some(required_attr(viewport, "parent-writing-mode")?))?;
            let direction = parse_direction(required_attr(viewport, "parent-direction")?)?;
            let host_inline_size =
                parse_host_inline_size(required_attr(viewport, "host-inline-size")?)?;
            Ok(RootContext::FlexItem {
                parent_axes: layout::FlowAxes::new(writing_mode, direction),
                host_inline_size,
            })
        }
        _ => Err(Error::new(format!("unsupported root context `{raw}`"))),
    }
}

fn parse_host_inline_size(raw: &str) -> Result<Scalar, Error> {
    let value = raw
        .strip_suffix("px")
        .ok_or_else(|| Error::new(format!("invalid host inline size `{raw}`")))
        .and_then(parse_number)?;
    if !value.is_finite() || value < 0.0 {
        return Err(Error::new(format!("invalid host inline size `{raw}`")));
    }
    Ok(value)
}

fn parse_item_order(raw: &str) -> Result<layout::ItemOrder, Error> {
    let bytes = raw.as_bytes();
    let digits = match bytes {
        [b'0'] => &bytes[1..],
        [b'-', first, rest @ ..] if first.is_ascii_digit() && *first != b'0' => rest,
        [first, rest @ ..] if first.is_ascii_digit() && *first != b'0' => rest,
        _ => return Err(Error::new(format!("invalid item order `{raw}`"))),
    };
    if !digits.iter().all(u8::is_ascii_digit) {
        return Err(Error::new(format!("invalid item order `{raw}`")));
    }

    raw.parse::<i32>()
        .map(layout::ItemOrder::new)
        .map_err(|_| Error::new(format!("invalid item order `{raw}`")))
}

fn parse_bool(raw: &str) -> Result<bool, Error> {
    match raw {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Error::new(format!("invalid boolean `{raw}`"))),
    }
}

fn parse_number(raw: &str) -> Result<Scalar, Error> {
    // Browser parity XML is a default-precision fixture boundary. The layout
    // engine has generic scalar APIs, but checked-in browser fixtures parse
    // through layout::Scalar/layout::DefaultScalar so legacy XML remains stable.
    raw.parse()
        .map_err(|_| Error::new(format!("invalid number `{raw}`")))
}

fn optional_number_attr(xml: roxmltree::Node<'_, '_>, name: &str) -> Result<Option<Scalar>, Error> {
    xml.attribute(name).map(parse_number).transpose()
}

fn required_attr<'a>(node: roxmltree::Node<'a, '_>, name: &str) -> Result<&'a str, Error> {
    node.attribute(name).ok_or_else(|| {
        Error::new(format!(
            "missing `{name}` on `<{}>`",
            node.tag_name().name()
        ))
    })
}

fn one_child<'a>(
    node: roxmltree::Node<'a, '_>,
    tag: &str,
) -> Result<roxmltree::Node<'a, 'a>, Error> {
    let mut matches = node
        .children()
        .filter(roxmltree::Node::is_element)
        .filter(|child| child.has_tag_name(tag));
    let child = matches
        .next()
        .ok_or_else(|| Error::new(format!("missing `<{tag}>` child")))?;
    if matches.next().is_some() {
        return Err(Error::new(format!("expected exactly one `<{tag}>` child")));
    }
    Ok(child)
}

fn one_element_child<'a>(node: roxmltree::Node<'a, '_>) -> Result<roxmltree::Node<'a, 'a>, Error> {
    let mut children = node.children().filter(roxmltree::Node::is_element);
    let child = children
        .next()
        .ok_or_else(|| Error::new(format!("missing child for `<{}>`", node.tag_name().name())))?;
    if children.next().is_some() {
        return Err(Error::new(format!(
            "expected exactly one child for `<{}>`",
            node.tag_name().name()
        )));
    }
    Ok(child)
}

fn expect_tag(node: roxmltree::Node<'_, '_>, tag: &str) -> Result<(), Error> {
    if node.has_tag_name(tag) {
        Ok(())
    } else {
        Err(Error::new(format!(
            "expected `<{tag}>`, found `<{}>`",
            node.tag_name().name()
        )))
    }
}

#[derive(Clone, Debug)]
struct TestNode {
    node_input: layout::NodeInput,
    layout_input: layout::LayoutInput,
    font_family: FontFamily,
    font_size: Scalar,
    line_height: Scalar,
    text: Option<String>,
    children: Vec<usize>,
    synthetic: bool,
    preserve_fractional_min_content: bool,
    use_tighter_monospace_wrap: bool,
    cache: layout::Cache,
    unrounded: layout::NodeOutput,
    unrounded_present: bool,
    final_layout: layout::NodeOutput,
    final_layout_present: bool,
    unrounded_inline_fragments: Option<Vec<layout::InlineFragmentOutput>>,
    final_inline_fragments: Option<Vec<layout::InlineFragmentOutput>>,
    shape_bands: Option<Vec<FixtureShapeBand>>,
}

#[derive(Clone, Debug, Default)]
struct TestTree {
    nodes: Vec<TestNode>,
}

#[derive(Clone, Copy, Debug)]
struct InheritedTextContext {
    font_family: FontFamily,
    font_size: Scalar,
    line_height: LineHeightState,
    grid_lanes_text: bool,
    inline_level_text: bool,
    containing_flow: Option<layout::FlowAxes>,
}

impl TestTree {
    fn from_golden(root: &Node) -> Result<Self, Error> {
        let mut tree = Self::default();
        tree.push_node(
            root,
            InheritedTextContext {
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: LineHeightState::Normal,
                grid_lanes_text: false,
                inline_level_text: false,
                containing_flow: None,
            },
        )?;
        Ok(tree)
    }

    fn push_node(&mut self, node: &Node, inherited: InheritedTextContext) -> Result<usize, Error> {
        let id = self.nodes.len();
        let font_family = font_family(&node.style)?.unwrap_or(inherited.font_family);
        let font_size = font_size(&node.style)?.unwrap_or(inherited.font_size);
        let line_height = match line_height(&node.style)? {
            Some(value) => LineHeightState::Px(value),
            None => inherited.line_height,
        };
        let resolved_line_height = line_height.resolve(font_size);
        let mut layout_input = match &node.inline_text {
            Some(input) => layout::LayoutInput::inline_text(input.clone()),
            None => to_layout_input_in_flow(&node.style, inherited.containing_flow)?,
        };
        if let Some(participation) = node.atomic_inline_participation {
            let Some(input) = layout_input.as_box() else {
                return Err(Error::new("atomic placeholder must reference a box child"));
            };
            let mut input = input.clone();
            input.atomic_inline_participation = Some(participation);
            layout_input = layout::LayoutInput::box_input(input);
        }
        if node.shape_bands.is_some()
            && !layout_input.as_box().is_some_and(|input| {
                input.float_exclusion == layout::FloatExclusion::Shape
                    && input.display != layout::Display::None
                    && input.position != layout::Position::Absolute
                    && matches!(input.float, layout::Float::Left | layout::Float::Right)
            })
        {
            return Err(Error::new(
                "shape band table requires a visible in-flow left/right shape float",
            ));
        }
        let box_display = layout_input.as_box().map(|input| input.display);
        let containing_flow = layout_input
            .as_box()
            .map(|input| layout::FlowAxes::new(input.writing_mode, input.direction));
        let grid_lanes_text = inherited.grid_lanes_text
            || box_display.is_some_and(layout::Display::establishes_grid_lanes_formatting_context);
        let inline_level_text = inherited.inline_level_text
            || box_display.is_some_and(layout::Display::is_inline_level);
        self.nodes.push(TestNode {
            node_input: layout_input
                .as_box()
                .cloned()
                .unwrap_or_else(layout::NodeInput::non_box),
            layout_input,
            font_family,
            font_size,
            line_height: resolved_line_height,
            text: node.text.clone(),
            children: Vec::new(),
            synthetic: false,
            preserve_fractional_min_content: inherited.grid_lanes_text,
            use_tighter_monospace_wrap: !inherited.inline_level_text,
            cache: layout::Cache::new(),
            unrounded: layout::NodeOutput::new(),
            unrounded_present: false,
            final_layout: layout::NodeOutput::new(),
            final_layout_present: false,
            unrounded_inline_fragments: None,
            final_inline_fragments: None,
            shape_bands: node.shape_bands.clone(),
        });

        let mut children = node
            .children
            .iter()
            .map(|child| {
                self.push_node(
                    child,
                    InheritedTextContext {
                        font_family,
                        font_size,
                        line_height,
                        grid_lanes_text,
                        inline_level_text,
                        containing_flow,
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(text) = &node.text
            && box_display.is_some_and(grid_text_container_needs_anonymous_child)
        {
            children.push(self.push_synthetic_text(
                text,
                font_family,
                font_size,
                resolved_line_height,
                grid_lanes_text,
                inline_level_text,
            )?);
            self.nodes[id].text = None;
        }
        self.nodes[id].children = children;
        Ok(id)
    }

    fn push_synthetic_text(
        &mut self,
        text: &str,
        font_family: FontFamily,
        font_size: Scalar,
        line_height: Scalar,
        preserve_fractional_min_content: bool,
        inherited_inline_level_text: bool,
    ) -> Result<usize, Error> {
        let id = self.nodes.len();
        self.nodes.push(TestNode {
            node_input: layout::NodeInput::default(),
            layout_input: layout::LayoutInput::box_input(layout::NodeInput::default()),
            font_family,
            font_size,
            line_height,
            text: Some(text.to_string()),
            children: Vec::new(),
            synthetic: true,
            preserve_fractional_min_content,
            use_tighter_monospace_wrap: !inherited_inline_level_text,
            cache: layout::Cache::new(),
            unrounded: layout::NodeOutput::new(),
            unrounded_present: false,
            final_layout: layout::NodeOutput::new(),
            final_layout_present: false,
            unrounded_inline_fragments: None,
            final_inline_fragments: None,
            shape_bands: None,
        });
        Ok(id)
    }

    fn measure(&self, node: usize, input: layout::LeafMeasureInput) -> layout::Size {
        let known = input.known_content_size();
        let available = input
            .available_content_size()
            .map(layout::MeasurementAvailable::into_available);
        if let Some(text) = &self.nodes[node].text {
            let text = TextMeasure::new(
                text,
                self.nodes[node].font_family,
                self.nodes[node].font_size,
                self.nodes[node].line_height,
                self.nodes[node].preserve_fractional_min_content,
                self.nodes[node].use_tighter_monospace_wrap,
            );
            if self.box_node_input(node).writing_mode.is_vertical() {
                let height = known.height.unwrap_or_else(|| match available.height {
                    layout::Available::Definite(height) => height,
                    layout::Available::MinContent => text.min_content_width(),
                    layout::Available::MaxContent => text.max_content_width(),
                });
                let width = known.width.unwrap_or_else(|| text.width_for_height(height));
                return layout::Size::new(width, height);
            }

            let width = known.width.unwrap_or_else(|| match available.width {
                layout::Available::Definite(width) => text
                    .max_content_width()
                    .min(width)
                    .max(text.min_content_width()),
                layout::Available::MinContent => text.min_content_width(),
                layout::Available::MaxContent => text.max_content_width(),
            });
            let height = known.height.unwrap_or_else(|| text.height_for_width(width));
            return layout::Size::new(width, height);
        }

        layout::Size::new(known.width.unwrap_or(0.0), known.height.unwrap_or(0.0))
    }

    fn apply_completed_batch(&mut self, batch: &layout::CompletedLayoutBatch<usize>) {
        batch.apply_to(self).unwrap_or_else(|error| match error {});
    }

    fn box_node_input(&self, node: usize) -> &layout::NodeInput {
        &self.nodes[node].node_input
    }
}

fn can_use_leaf_measurement(display: layout::Display, child_count: usize, has_text: bool) -> bool {
    child_count == 0 && (has_text || !display.establishes_grid_formatting_context())
}

fn grid_text_container_needs_anonymous_child(display: layout::Display) -> bool {
    matches!(
        display,
        layout::Display::Grid
            | layout::Display::InlineGrid
            | layout::Display::GridLanes
            | layout::Display::InlineGridLanes
    )
}

#[derive(Clone, Copy)]
struct TextMeasure<'a> {
    text: &'a str,
    font_family: FontFamily,
    font_size: Scalar,
    line_height: Scalar,
    preserve_fractional_min_content: bool,
    use_tighter_monospace_wrap: bool,
}

impl<'a> TextMeasure<'a> {
    const LINE_HEIGHT: Scalar = 10.0;

    fn new(
        text: &'a str,
        font_family: FontFamily,
        font_size: Scalar,
        line_height: Scalar,
        preserve_fractional_min_content: bool,
        use_tighter_monospace_wrap: bool,
    ) -> Self {
        Self {
            text,
            font_family,
            font_size,
            line_height,
            preserve_fractional_min_content,
            use_tighter_monospace_wrap,
        }
    }

    fn max_content_width(self) -> Scalar {
        self.char_count() as Scalar * self.advance()
    }

    fn min_content_width(self) -> Scalar {
        let width = self
            .wrap_words()
            .into_iter()
            .map(|word| word.chars)
            .max()
            .unwrap_or(0) as Scalar
            * self.advance();
        if self.preserve_fractional_min_content {
            width
        } else {
            width.floor()
        }
    }

    fn height_for_width(self, width: Scalar) -> Scalar {
        let advance = self.wrap_advance();
        let width = width.max(advance);
        let max_chars_per_line = (width / advance).floor().max(1.0) as usize;
        let mut line_count = 1;
        let mut line_chars = 0;
        for word in self.wrap_words() {
            let separator = usize::from(word.space_before && line_chars > 0);
            if line_chars == 0 {
                line_chars = word.chars;
            } else if line_chars + separator + word.chars <= max_chars_per_line {
                line_chars += separator + word.chars;
            } else {
                line_count += 1;
                line_chars = word.chars;
            }
        }
        line_count as Scalar * self.line_height()
    }

    fn width_for_height(self, height: Scalar) -> Scalar {
        let line_height = self.line_height();
        let height = height.max(line_height);
        let max_chars_per_column = (height / line_height).floor().max(1.0) as usize;
        let column_count = self.char_count().div_ceil(max_chars_per_column);
        column_count as Scalar * self.advance()
    }

    fn char_count(self) -> usize {
        self.text.chars().filter(|ch| *ch != '\u{200b}').count()
    }

    fn advance(self) -> Scalar {
        self.font_size * self.font_family.advance_factor()
    }

    fn wrap_advance(self) -> Scalar {
        if self.use_tighter_monospace_wrap {
            self.font_size * self.font_family.wrap_advance_factor()
        } else {
            self.advance()
        }
    }

    fn line_height(self) -> Scalar {
        self.line_height
    }

    fn wrap_words(self) -> Vec<WrapWord> {
        let mut words = Vec::new();
        let mut chars = 0;
        let mut space_before_next = false;
        let mut current_space_before = false;
        for ch in self.text.chars() {
            if ch == '\u{200b}' || ch.is_whitespace() {
                if chars > 0 {
                    words.push(WrapWord {
                        chars,
                        space_before: current_space_before,
                    });
                    chars = 0;
                }
                current_space_before = ch.is_whitespace() || space_before_next;
                space_before_next = current_space_before;
            } else {
                chars += 1;
            }
        }
        if chars > 0 {
            words.push(WrapWord {
                chars,
                space_before: current_space_before,
            });
        }
        words
    }
}

#[derive(Clone, Copy)]
struct WrapWord {
    chars: usize,
    space_before: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FontFamily {
    Ahem,
    Monospace,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LineHeightState {
    Normal,
    Px(Scalar),
}

impl LineHeightState {
    fn resolve(self, font_size: Scalar) -> Scalar {
        match self {
            Self::Normal => font_size,
            Self::Px(value) => value,
        }
    }
}

impl FontFamily {
    fn advance_factor(self) -> Scalar {
        match self {
            Self::Ahem => 1.0,
            // Chromium's default monospace font in these fixtures is narrower
            // than Ahem's square glyphs. Keep this deliberately small and local
            // to parity XML text measurement until a real font backend exists.
            Self::Monospace => 0.602_083_3,
        }
    }

    fn wrap_advance_factor(self) -> Scalar {
        match self {
            Self::Ahem => self.advance_factor(),
            Self::Monospace => 0.593_75,
        }
    }
}

impl layout::Traverse for TestTree {
    type Node = usize;

    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.nodes[node].children.iter().copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.nodes[node].children.len()
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.nodes[node].children[index]
    }
}

impl layout::LayoutTree for TestTree {
    type MeasureError = Error;

    fn node_input(&self, node: Self::Node) -> &layout::NodeInput {
        self.box_node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> layout::LayoutInput {
        self.nodes[node].layout_input.clone()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInput,
        context: layout::CacheKeyContext,
    ) -> Option<ComputeOutput> {
        self.nodes[node].cache.get_with_context(input, context)
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<layout::NodeOutput> {
        self.nodes[node]
            .unrounded_present
            .then_some(self.nodes[node].unrounded)
    }

    fn float_exclusion_interval(
        &self,
        node: Self::Node,
        query: layout::FloatExclusionQuery,
    ) -> Option<Result<Option<layout::FloatExclusionInterval>, Self::MeasureError>> {
        let table = self.nodes[node].shape_bands.as_ref()?;
        let Some(band) = table.iter().find(|band| {
            band.band_minimum == query.band_minimum() && band.band_maximum == query.band_maximum()
        }) else {
            return Some(Err(Error::new(format!(
                "missing fixture shape response for query band `{}..{}`",
                query.band_minimum(),
                query.band_maximum()
            ))));
        };

        match band.response {
            FixtureShapeResponse::Empty => Some(Ok(None)),
            FixtureShapeResponse::Failure => Some(Err(Error::new(format!(
                "fixture shape provider failure for query band `{}..{}`",
                query.band_minimum(),
                query.band_maximum()
            )))),
            FixtureShapeResponse::Interval {
                minimum,
                maximum,
                originating_band,
            } => {
                let originating_query = match originating_band {
                    Some((band_minimum, band_maximum)) => {
                        match layout::FloatExclusionQuery::try_new(
                            query.margin_box(),
                            query.flow_axes(),
                            band_minimum,
                            band_maximum,
                        ) {
                            Ok(query) => query,
                            Err(error) => {
                                return Some(Err(Error::new(format!(
                                    "invalid fixture originating shape query: {error:?}"
                                ))));
                            }
                        }
                    }
                    None => query,
                };
                Some(
                    layout::FloatExclusionInterval::try_new(originating_query, minimum, maximum)
                        .map_err(|error| {
                            Error::new(format!("invalid fixture shape response: {error:?}"))
                        }),
                )
            }
        }
    }

    fn unrounded_inline_fragments(
        &self,
        node: Self::Node,
    ) -> Option<&[layout::InlineFragmentOutput]> {
        self.nodes[node].unrounded_inline_fragments.as_deref()
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        if self.nodes[node].layout_input.as_box().is_none() {
            return false;
        }
        can_use_leaf_measurement(
            self.box_node_input(node).display,
            self.nodes[node].children.len(),
            self.nodes[node].text.is_some(),
        )
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: layout::LeafMeasureInput,
    ) -> Option<Result<layout::Size, Self::MeasureError>> {
        self.nodes[node].layout_input.as_box()?;
        Some(Ok(self.measure(node, input)))
    }
}

impl layout::LayoutBatchSink<usize, Scalar> for TestTree {
    type Error = std::convert::Infallible;
    type Prepared = Vec<TestNode>;

    fn prepare_layout_batch(
        &self,
        batch: &layout::CompletedLayoutBatch<usize>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut prepared = self.nodes.clone();
        for node in batch.invalidated_nodes() {
            let state = &mut prepared[*node];
            state.cache.clear();
            state.unrounded_present = false;
            state.final_layout_present = false;
            state.unrounded_inline_fragments = None;
            state.final_inline_fragments = None;
        }
        for entry in batch.cache_clear_entries() {
            prepared[entry.node()].cache.clear();
        }
        for entry in batch.unrounded_entries() {
            let state = &mut prepared[entry.node()];
            state.unrounded = entry.output();
            state.unrounded_present = true;
            if matches!(state.layout_input, layout::LayoutInput::InlineText(_)) {
                state.unrounded_inline_fragments = Some(Vec::new());
            }
        }
        for entry in batch.final_entries() {
            let state = &mut prepared[entry.node()];
            state.final_layout = entry.output();
            state.final_layout_present = true;
            if matches!(state.layout_input, layout::LayoutInput::InlineText(_)) {
                state.final_inline_fragments = Some(Vec::new());
            }
        }
        for entry in batch.unrounded_inline_fragments() {
            prepared[entry.node()]
                .unrounded_inline_fragments
                .get_or_insert_with(Vec::new)
                .push(entry.fragment());
        }
        for entry in batch.final_inline_fragments() {
            prepared[entry.node()]
                .final_inline_fragments
                .get_or_insert_with(Vec::new)
                .push(entry.fragment());
        }
        for entry in batch.cache_store_entries() {
            prepared[entry.node()].cache.store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
        Ok(prepared)
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.nodes = prepared;
    }
}

fn compare_expectation(
    tree: &TestTree,
    node: usize,
    expected: &Expectation,
    path: &str,
    use_rounding: bool,
    final_inline_fragments: &[layout::InlineFragmentOutputEntry<usize>],
) -> Result<(), Error> {
    let mut fragment_cursor = FinalFragmentCursor {
        entries: final_inline_fragments,
        next: 0,
    };
    compare_expectation_in_source_order(
        tree,
        node,
        expected,
        path,
        use_rounding,
        &mut fragment_cursor,
    )?;
    if let Some(entry) = fragment_cursor.entries.get(fragment_cursor.next) {
        return Err(Error::new(format!(
            "{path}: unexpected fragment source association at node {}",
            entry.node()
        )));
    }
    Ok(())
}

struct FinalFragmentCursor<'a> {
    entries: &'a [layout::InlineFragmentOutputEntry<usize>],
    next: usize,
}

impl<'a> FinalFragmentCursor<'a> {
    fn take_for_node(&mut self, node: usize) -> &'a [layout::InlineFragmentOutputEntry<usize>] {
        let start = self.next;
        while self
            .entries
            .get(self.next)
            .is_some_and(|entry| entry.node() == node)
        {
            self.next += 1;
        }
        &self.entries[start..self.next]
    }
}

fn compare_expectation_in_source_order(
    tree: &TestTree,
    node: usize,
    expected: &Expectation,
    path: &str,
    use_rounding: bool,
    fragment_cursor: &mut FinalFragmentCursor<'_>,
) -> Result<(), Error> {
    let selected_output_is_present = if use_rounding {
        tree.nodes[node].final_layout_present
    } else {
        tree.nodes[node].unrounded_present
    };
    if !selected_output_is_present
        && matches!(
            tree.nodes[node].layout_input,
            layout::LayoutInput::LineBreak(_) | layout::LayoutInput::InlineBoundary(_)
        )
    {
        let phase = if use_rounding { "final" } else { "unrounded" };
        return Err(Error::new(format!(
            "{path}: control geometry mismatch, expected {phase} output"
        )));
    }

    let actual = if use_rounding {
        tree.nodes[node].final_layout
    } else {
        tree.nodes[node].unrounded
    };
    compare_optional_number(path, "x", actual.location.x, expected.x)?;
    compare_optional_number(path, "y", actual.location.y, expected.y)?;
    compare_optional_number(path, "width", actual.size.width, expected.width)?;
    compare_optional_number(path, "height", actual.size.height, expected.height)?;

    if let Some(expected_scroll_size) = expected.scroll_size {
        let scroll_geometry = actual.scroll_geometry.ok_or_else(|| {
            Error::new(format!(
                "{path}: scroll geometry mismatch, expected canonical geometry"
            ))
        })?;
        let range = scroll_geometry.physical_range();
        let x_span = range.x().maximum() - range.x().minimum();
        let y_span = range.y().maximum() - range.y().minimum();
        compare_number(path, "scroll width", x_span, expected_scroll_size.width)?;
        compare_number(path, "scroll height", y_span, expected_scroll_size.height)?;
    }

    let actual_fragments = fragment_cursor.take_for_node(node);
    if let Some(expected_fragments) = &expected.fragments {
        compare_fragment_expectations(path, actual_fragments, expected_fragments)?;
    }
    if let Some(expected_range_inks) = &expected.range_inks {
        compare_range_ink_expectations(path, actual_fragments, expected_range_inks)?;
    }

    let children = tree.nodes[node]
        .children
        .iter()
        .copied()
        .filter(|child| !tree.nodes[*child].synthetic)
        .collect::<Vec<_>>();
    if children.len() != expected.children.len() {
        return Err(Error::new(format!(
            "{path}: expected {} children, got {}",
            expected.children.len(),
            children.len()
        )));
    }

    for (index, (child, expected_child)) in children
        .into_iter()
        .zip(expected.children.iter())
        .enumerate()
    {
        compare_expectation_in_source_order(
            tree,
            child,
            expected_child,
            &format!("{path}/{index}"),
            use_rounding,
            fragment_cursor,
        )?;
    }

    Ok(())
}

fn compare_fragment_expectations(
    path: &str,
    actual: &[layout::InlineFragmentOutputEntry<usize>],
    expected: &[InlineFragmentExpectation],
) -> Result<(), Error> {
    if actual.len() != expected.len() {
        return Err(Error::new(format!(
            "{path}: fragment count mismatch, expected {}, got {}",
            expected.len(),
            actual.len()
        )));
    }

    for (index, (entry, expected)) in actual.iter().zip(expected).enumerate() {
        let fragment = entry.fragment();
        compare_fragment_identity(
            path,
            index,
            "source segment id",
            fragment.segment_id().get(),
            expected.source_segment_id,
        )?;
        compare_fragment_identity(
            path,
            index,
            "line index",
            fragment.line_index(),
            expected.line_index,
        )?;
        compare_fragment_identity(
            path,
            index,
            "visual index",
            fragment.visual_index(),
            expected.visual_index,
        )?;

        let rect = fragment.rect();
        compare_number(
            path,
            &format!("fragment[{index}] rect x"),
            rect.origin().x,
            expected.x,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] rect y"),
            rect.origin().y,
            expected.y,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] rect width"),
            rect.size().width,
            expected.width,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] rect height"),
            rect.size().height,
            expected.height,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] baseline x"),
            fragment.baseline().x,
            expected.baseline_x,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] baseline y"),
            fragment.baseline().y,
            expected.baseline_y,
        )?;
    }

    Ok(())
}

fn compare_range_ink_expectations(
    path: &str,
    actual: &[layout::InlineFragmentOutputEntry<usize>],
    expected: &[InlineRangeInkExpectation],
) -> Result<(), Error> {
    if actual.len() != expected.len() {
        return Err(Error::new(format!(
            "{path}: Range ink count mismatch, expected {}, got {}",
            expected.len(),
            actual.len()
        )));
    }

    for (index, (entry, expected)) in actual.iter().zip(expected).enumerate() {
        let fragment = entry.fragment();
        compare_range_ink_identity(
            path,
            index,
            "source segment id",
            fragment.segment_id().get(),
            expected.source_segment_id,
        )?;
        compare_range_ink_identity(
            path,
            index,
            "line index",
            fragment.line_index(),
            expected.line_index,
        )?;
        compare_range_ink_identity(
            path,
            index,
            "visual index",
            fragment.visual_index(),
            expected.visual_index,
        )?;

        let rect = fragment.rect();
        let (start, advance) = match expected.physical_start_edge {
            PhysicalStartEdge::Left => (rect.origin().x, rect.size().width),
            PhysicalStartEdge::Right => (rect.origin().x + rect.size().width, rect.size().width),
            PhysicalStartEdge::Top => (rect.origin().y, rect.size().height),
            PhysicalStartEdge::Bottom => (rect.origin().y + rect.size().height, rect.size().height),
        };
        compare_number(
            path,
            &format!("Range ink[{index}] physical flow-inline start"),
            start,
            expected.start,
        )?;
        compare_number(
            path,
            &format!("Range ink[{index}] advance"),
            advance,
            expected.advance,
        )?;
    }

    Ok(())
}

fn compare_range_ink_identity<T>(
    path: &str,
    index: usize,
    field: &str,
    actual: T,
    expected: T,
) -> Result<(), Error>
where
    T: Copy + std::fmt::Display + Eq,
{
    if actual == expected {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{path}: Range ink[{index}] {field} mismatch, expected {expected}, got {actual}"
        )))
    }
}

fn compare_fragment_identity<T>(
    path: &str,
    index: usize,
    field: &str,
    actual: T,
    expected: T,
) -> Result<(), Error>
where
    T: Copy + std::fmt::Display + Eq,
{
    if actual == expected {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{path}: fragment[{index}] {field} mismatch, expected {expected}, got {actual}"
        )))
    }
}

fn compare_number(path: &str, field: &str, actual: Scalar, expected: Scalar) -> Result<(), Error> {
    if ComparisonTolerance::browser_parity().contains(actual - expected) {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{path}: {field} mismatch, expected {expected}, got {actual}"
        )))
    }
}

fn compare_optional_number(
    path: &str,
    field: &str,
    actual: Scalar,
    expected: Option<Scalar>,
) -> Result<(), Error> {
    match expected {
        Some(expected) => compare_number(path, field, actual, expected),
        None => Ok(()),
    }
}

fn to_layout_input(attrs: &StyleAttrs) -> Result<layout::LayoutInput, Error> {
    to_layout_input_in_flow(attrs, None)
}

fn to_layout_input_in_flow(
    attrs: &StyleAttrs,
    containing_flow: Option<layout::FlowAxes>,
) -> Result<layout::LayoutInput, Error> {
    let input = to_node_input(attrs)?;
    if attrs.get("source-tag") == Some("br") {
        let flow = containing_flow
            .unwrap_or_else(|| layout::FlowAxes::new(input.writing_mode, input.direction));
        let mut br = layout::LineBreakInput::new()
            .with_direction(flow.direction())
            .with_writing_mode(flow.writing_mode())
            .with_vertical_align(input.vertical_align)
            .with_clear(input.clear);
        if input.display == layout::Display::None {
            br = br.hidden();
        }
        if let Some(metrics) = inline_metrics(attrs)? {
            br = br.with_metrics(metrics);
        }
        Ok(layout::LayoutInput::line_break(br))
    } else {
        Ok(layout::LayoutInput::box_input(input))
    }
}

fn to_node_input(attrs: &StyleAttrs) -> Result<layout::NodeInput, Error> {
    for name in ["overflow", "scroll-padding", "scroll-margin", "transform"] {
        if attrs.get(name).is_some() {
            return Err(Error::new(format!(
                "unsupported authored fixture attribute `{name}`"
            )));
        }
    }
    let mut input = layout::NodeInput {
        overflow: parse_computed_overflow(attrs)?,
        ..layout::NodeInput::default()
    };
    let source_tag = attrs.get("source-tag");
    input.display = match attrs.get("display") {
        Some("inline") if source_tag == Some("br") => layout::Display::Block,
        Some(value) => parse_display(value)?,
        None => match source_tag {
            Some("div") => layout::Display::Block,
            _ => input.display,
        },
    };
    if let Some(value) = attrs.get("order") {
        input.item_order = parse_item_order(value)?;
    }
    if let Some(value) = attrs.get("box-sizing") {
        input.box_sizing = parse_box_sizing(value)?;
    }
    if let Some(value) = attrs.get("direction") {
        input.direction = parse_direction(value)?;
    }
    if let Some(value) = attrs.get("position") {
        input.position = parse_position(value)?;
    }
    if let Some(value) = attrs.get("float") {
        input.float = parse_float(value)?;
    }
    if let Some(value) = attrs.get("float-exclusion") {
        input.float_exclusion = match value {
            "shape" => layout::FloatExclusion::Shape,
            _ => {
                return Err(Error::new(format!(
                    "unsupported fixture float exclusion `{value}`"
                )));
            }
        };
    }
    if let Some(value) = attrs.get("clear") {
        input.clear = parse_clear(value)?;
    }
    if let Some(value) = attrs.get("scrollbar-width") {
        input.scrollbar_width = layout::ScrollbarWidth::try_new(parse_number(value)?)
            .map_err(|source| Error::new(source.to_string()))?;
    }
    if let Some(value) = attrs.get("overflow-clip-margin") {
        input.overflow_clip_margin = parse_overflow_clip_margin(value)?;
    }
    if let Some(value) = attrs.get("scrollbar-gutter") {
        input.scrollbar_gutter = parse_scrollbar_gutter(value)?;
    }
    input.scroll_padding = layout::ScrollPadding::new(
        parse_scroll_padding(attrs.get("scroll-padding-top"))?,
        parse_scroll_padding(attrs.get("scroll-padding-right"))?,
        parse_scroll_padding(attrs.get("scroll-padding-bottom"))?,
        parse_scroll_padding(attrs.get("scroll-padding-left"))?,
    );
    input.scroll_margin = layout::ScrollMargin::try_new(
        parse_scroll_margin(attrs.get("scroll-margin-top"))?,
        parse_scroll_margin(attrs.get("scroll-margin-right"))?,
        parse_scroll_margin(attrs.get("scroll-margin-bottom"))?,
        parse_scroll_margin(attrs.get("scroll-margin-left"))?,
    )
    .map_err(|source| Error::new(source.to_string()))?;
    if let Some(value) = attrs.get("scroll-snap-type") {
        input.scroll_snap_type = parse_scroll_snap_type(value)?;
    }
    if let Some(value) = attrs.get("scroll-snap-align") {
        input.scroll_snap_align = parse_scroll_snap_align(value)?;
    }
    if let Some(value) = attrs.get("scroll-snap-stop") {
        input.scroll_snap_stop = parse_scroll_snap_stop(value)?;
    }
    if let Some(value) = attrs.get("text-align") {
        input.text_align = parse_text_align(value)?;
    }
    if let Some(value) = attrs.get("vertical-align") {
        input.vertical_align = parse_vertical_align(value)?;
    }
    input.writing_mode = parse_writing_mode(attrs.get("writing-mode"))?;
    if let Some(value) = attrs.get("flex-direction") {
        input.flex_direction = parse_flex_direction(value)?;
    }
    if let Some(value) = attrs.get("flex-wrap") {
        input.flex_wrap = parse_flex_wrap(value)?;
    }
    if let Some(value) = attrs.get("flex-grow") {
        input.flex_grow = layout::FlexGrow::try_new(parse_number(value)?)
            .map_err(|source| Error::new(source.to_string()))?;
    }
    if let Some(value) = attrs.get("flex-shrink") {
        input.flex_shrink = layout::FlexShrink::try_new(parse_number(value)?)
            .map_err(|source| Error::new(source.to_string()))?;
    }
    if let Some(value) = attrs.get("flex-basis") {
        input.flex_basis = parse_flex_basis(value)?;
    }
    if let Some(value) = attrs.get("width") {
        input.size.width = parse_preferred_size(value)?;
    }
    if let Some(value) = attrs.get("height") {
        input.size.height = parse_preferred_size(value)?;
    }
    if let Some(value) = attrs.get("min-width") {
        input.min_size.width = parse_min_size(value)?;
    }
    if let Some(value) = attrs.get("min-height") {
        input.min_size.height = parse_min_size(value)?;
    }
    if let Some(value) = attrs.get("max-width") {
        input.max_size.width = parse_max_size(value)?;
    }
    if let Some(value) = attrs.get("max-height") {
        input.max_size.height = parse_max_size(value)?;
    }
    if let Some(value) = attrs.get("aspect-ratio") {
        let value = parse_number(value)?;
        input.aspect_ratio = layout::AspectRatio::new(value);
        if input.aspect_ratio.is_none() {
            return Err(Error::new(format!("invalid aspect-ratio `{value}`")));
        }
    }
    let flow_axes = layout::FlowAxes::new(input.writing_mode, input.direction);
    let (default_inline_gap, default_block_gap) = match flow_axes.inline_axis() {
        layout::PhysicalAxis::Horizontal => (input.gap.width, input.gap.height),
        layout::PhysicalAxis::Vertical => (input.gap.height, input.gap.width),
    };
    let inline_gap = match attrs.get("column-gap") {
        Some(value) => parse_length_with_calc(value)?,
        None => default_inline_gap,
    };
    let block_gap = match attrs.get("row-gap") {
        Some(value) => parse_length_with_calc(value)?,
        None => default_block_gap,
    };
    input.gap = match flow_axes.inline_axis() {
        layout::PhysicalAxis::Horizontal => layout::Size::new(inline_gap, block_gap),
        layout::PhysicalAxis::Vertical => layout::Size::new(block_gap, inline_gap),
    };

    apply_edges_auto(
        &mut input.margin,
        attrs,
        [
            ("margin-top", 0),
            ("margin-right", 1),
            ("margin-bottom", 2),
            ("margin-left", 3),
        ],
        layout::LengthAuto::ZERO,
    )?;
    apply_edges(
        &mut input.padding,
        attrs,
        [
            ("padding-top", 0),
            ("padding-right", 1),
            ("padding-bottom", 2),
            ("padding-left", 3),
        ],
        layout::Length::ZERO,
    )?;
    apply_edges(
        &mut input.border,
        attrs,
        [
            ("border-top", 0),
            ("border-right", 1),
            ("border-bottom", 2),
            ("border-left", 3),
        ],
        layout::Length::ZERO,
    )?;
    apply_edges_auto(
        &mut input.inset,
        attrs,
        [("top", 0), ("right", 1), ("bottom", 2), ("left", 3)],
        layout::LengthAuto::AUTO,
    )?;

    if let Some(value) = attrs.get("align-items") {
        input.align_items = Some(parse_align_items(value)?);
    }
    if let Some(value) = attrs.get("align-self") {
        input.align_self = Some(parse_align_items(value)?);
    }
    if let Some(value) = attrs.get("justify-items") {
        input.justify_items = Some(parse_align_items(value)?);
    }
    if let Some(value) = attrs.get("justify-self") {
        input.justify_self = Some(parse_align_items(value)?);
    }
    if let Some(value) = attrs.get("align-content") {
        input.align_content = Some(parse_align_content(value)?);
    }
    if let Some(value) = attrs.get("justify-content") {
        input.justify_content = Some(parse_align_content(value)?);
    }
    if let Some(value) = attrs.get("grid-auto-flow") {
        input.grid_auto_flow = parse_grid_auto_flow(value)?;
    }
    if let Some(value) = attrs.get("grid-template-columns") {
        input.grid_template_columns = parse_track_component_list_with_calc(value)?;
    }
    if let Some(value) = attrs.get("grid-template-rows") {
        input.grid_template_rows = parse_track_component_list_with_calc(value)?;
    }
    if let Some(value) = attrs.get("grid-template-areas") {
        input.grid_template_areas = parse_grid_template_areas(value)?;
    }
    if let Some(value) = attrs.get("grid-auto-columns") {
        input.grid_auto_columns = parse_track_component_list_with_calc(value)?;
    }
    if let Some(value) = attrs.get("grid-auto-rows") {
        input.grid_auto_rows = parse_track_component_list_with_calc(value)?;
    }
    let (grid_column, raw_grid_column) =
        parse_grid_placement(attrs.get("grid-column-start"), attrs.get("grid-column-end"))?;
    input.grid_column = grid_column;
    input.raw_grid_column = raw_grid_column;
    let (grid_row, raw_grid_row) =
        parse_grid_placement(attrs.get("grid-row-start"), attrs.get("grid-row-end"))?;
    input.grid_row = grid_row;
    input.raw_grid_row = raw_grid_row;

    Ok(input)
}

fn apply_edges(
    edges: &mut layout::Edges<layout::Length>,
    attrs: &StyleAttrs,
    names: [(&str, usize); 4],
    default: layout::Length,
) -> Result<(), Error> {
    for (name, side) in names {
        if let Some(value) = attrs.get(name) {
            set_edge(edges, side, parse_length_with_calc(value)?);
        } else {
            set_edge(edges, side, default);
        }
    }
    Ok(())
}

fn apply_edges_auto(
    edges: &mut layout::Edges<layout::LengthAuto>,
    attrs: &StyleAttrs,
    names: [(&str, usize); 4],
    default: layout::LengthAuto,
) -> Result<(), Error> {
    for (name, side) in names {
        if let Some(value) = attrs.get(name) {
            set_edge(edges, side, parse_length_auto_with_calc(value)?);
        } else {
            set_edge(edges, side, default);
        }
    }
    Ok(())
}

fn set_edge<T>(edges: &mut layout::Edges<T>, side: usize, value: T) {
    match side {
        0 => edges.top = value,
        1 => edges.right = value,
        2 => edges.bottom = value,
        3 => edges.left = value,
        _ => unreachable!("edge side index is fixed by caller"),
    }
}

fn font_size(attrs: &StyleAttrs) -> Result<Option<Scalar>, Error> {
    match attrs.get("font-size") {
        Some(value) => Ok(Some(parse_px_dimension(value, "font-size")?)),
        None => Ok(None),
    }
}

fn line_height(attrs: &StyleAttrs) -> Result<Option<Scalar>, Error> {
    match attrs.get("line-height") {
        Some(value) => Ok(Some(parse_px_dimension(value, "line-height")?)),
        None => Ok(None),
    }
}

fn inline_metrics(attrs: &StyleAttrs) -> Result<Option<layout::InlineMetrics>, Error> {
    match (
        attrs.get("inline-baseline"),
        attrs.get("inline-line-height"),
    ) {
        (None, None) => Ok(None),
        (Some(baseline), Some(line_height)) => {
            layout::InlineMetrics::from_line_height_and_baseline(
                parse_px_dimension(line_height, "inline-line-height")?,
                parse_px_dimension(baseline, "inline-baseline")?,
            )
            .map(Some)
            .map_err(|error| Error::new(format!("{error:?}")))
        }
        _ => Err(Error::new(
            "inline metrics require inline-baseline and inline-line-height",
        )),
    }
}

fn font_family(attrs: &StyleAttrs) -> Result<Option<FontFamily>, Error> {
    match attrs.get("font-family") {
        None => Ok(None),
        Some("ahem") => Ok(Some(FontFamily::Ahem)),
        Some("monospace") => Ok(Some(FontFamily::Monospace)),
        Some(value) => Err(Error::new(format!(
            "unsupported parity fixture font-family `{value}`"
        ))),
    }
}

fn parse_px_dimension(raw: &str, name: &str) -> Result<Scalar, Error> {
    let value = raw
        .strip_suffix("px")
        .ok_or_else(|| Error::new(format!("{name} must use px units, got `{raw}`")))?;
    parse_number(value)
}

fn parse_raw_grid_line(raw: &str) -> Result<layout::RawGridLine, Error> {
    let tokens = split_top_level_whitespace(raw);
    match tokens.as_slice() {
        [token] if token == "auto" => Ok(layout::RawGridLine::Auto),
        [token] if token == "span" => Err(Error::new("invalid grid span `span`")),
        [token] => match parse_style_line_index(token) {
            Ok(0) => Err(Error::new(format!("grid line cannot be zero in `{raw}`"))),
            Ok(line) => Ok(layout::RawGridLine::Line(line.into())),
            Err(_) => Ok(layout::RawGridLine::BareIdent(
                parse_custom_ident(token)?.to_owned(),
            )),
        },
        [span, token] if span == "span" => {
            if let Ok(index) = parse_style_span_index(token) {
                Ok(layout::RawGridLine::Span(index.into()))
            } else {
                Ok(layout::RawGridLine::NamedSpan {
                    name: parse_custom_ident(token)?.to_owned(),
                    index: 1,
                })
            }
        }
        [name, index] if parse_style_line_index(index).is_ok() => named_raw_line(name, index, raw),
        [index, name] if parse_style_line_index(index).is_ok() => named_raw_line(name, index, raw),
        [span, first, second] if span == "span" => {
            if let Ok(index) = parse_style_span_index(first) {
                Ok(layout::RawGridLine::NamedSpan {
                    name: parse_custom_ident(second)?.to_owned(),
                    index: index.into(),
                })
            } else {
                Ok(layout::RawGridLine::NamedSpan {
                    name: parse_custom_ident(first)?.to_owned(),
                    index: parse_style_span_index(second)?.into(),
                })
            }
        }
        _ => Err(Error::new(format!("unsupported grid line `{raw}`"))),
    }
}

fn parse_grid_placement(
    start: Option<&str>,
    end: Option<&str>,
) -> Result<(layout::GridPlacement, layout::RawGridPlacement), Error> {
    let start = start.unwrap_or("auto");
    let end = end.unwrap_or("auto");
    let mut span = None;
    let legacy_start = parse_grid_line_or_span(start, &mut span).unwrap_or(None);
    let legacy_end = parse_grid_line_or_span(end, &mut span).unwrap_or(None);
    let legacy = match (legacy_start, legacy_end, span) {
        (Some(start), Some(end), None) => layout::GridPlacement::try_lines(start, end),
        (Some(start), None, Some(span)) => layout::GridPlacement::try_line_span(start, span),
        (None, Some(end), Some(span)) => layout::GridPlacement::try_span_line(span, end),
        (Some(start), None, None) => layout::GridPlacement::try_line(start),
        (None, Some(end), None) => layout::GridPlacement::try_end_line(end),
        (None, None, Some(span)) => layout::GridPlacement::try_span(span),
        (None, None, None) => Some(layout::GridPlacement::AUTO),
        (Some(_), Some(_), Some(_)) => None,
    }
    .unwrap_or(layout::GridPlacement::AUTO);
    let raw = layout::RawGridPlacement::new(parse_raw_grid_line(start)?, parse_raw_grid_line(end)?);
    Ok((legacy, raw))
}

fn parse_grid_template_areas(raw: &str) -> Result<layout::GridTemplateAreas, Error> {
    let rows = raw
        .split('/')
        .map(str::trim)
        .filter(|row| !row.is_empty())
        .map(parse_grid_template_area_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(layout::GridTemplateAreas { rows })
}

fn parse_grid_template_area_row(raw: &str) -> Result<layout::GridTemplateAreaRow, Error> {
    let cells = split_top_level_whitespace(raw)
        .into_iter()
        .map(|cell| {
            if is_grid_template_area_null_cell(&cell) {
                Ok(None)
            } else {
                parse_custom_ident(&cell).map(|name| Some(name.to_owned()))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(layout::GridTemplateAreaRow { cells })
}

fn is_grid_template_area_null_cell(cell: &str) -> bool {
    !cell.is_empty() && cell.bytes().all(|byte| byte == b'.')
}

fn named_raw_line(name: &str, index: &str, raw: &str) -> Result<layout::RawGridLine, Error> {
    let index = parse_style_line_index(index)?;
    if index == 0 {
        return Err(Error::new(format!(
            "named grid line occurrence cannot be zero in `{raw}`"
        )));
    }
    Ok(layout::RawGridLine::NamedLine {
        name: parse_custom_ident(name)?.to_owned(),
        index: index.into(),
    })
}

fn parse_style_line_index(raw: &str) -> Result<i16, Error> {
    raw.parse()
        .map_err(|_| Error::new(format!("invalid grid line `{raw}`")))
}

fn parse_style_span_index(raw: &str) -> Result<u16, Error> {
    let index: u16 = raw
        .parse()
        .map_err(|_| Error::new(format!("invalid grid span `{raw}`")))?;
    if index == 0 {
        return Err(Error::new("grid span cannot be zero"));
    }
    Ok(index)
}

fn parse_custom_ident(raw: &str) -> Result<&str, Error> {
    if matches!(raw, "auto" | "span") || raw.parse::<i64>().is_ok() {
        return Err(Error::new(format!("invalid grid custom-ident `{raw}`")));
    }
    Ok(raw)
}

fn to_layout_available(value: Available) -> layout::Available {
    match value {
        Available::Definite(value) => layout::Available::definite(value),
        Available::MinContent => layout::Available::MIN_CONTENT,
        Available::MaxContent => layout::Available::MAX_CONTENT,
    }
}

fn parse_display(raw: &str) -> Result<layout::Display, Error> {
    match raw {
        "block" => Ok(layout::Display::Block),
        "inline-block" => Ok(layout::Display::InlineBlock),
        "flex" => Ok(layout::Display::Flex),
        "grid" => Ok(layout::Display::Grid),
        "inline-grid" => Ok(layout::Display::InlineGrid),
        "grid-lanes" => Ok(layout::Display::GridLanes),
        "inline-grid-lanes" => Ok(layout::Display::InlineGridLanes),
        "none" => Ok(layout::Display::None),
        _ => Err(Error::new(format!("unsupported display `{raw}`"))),
    }
}

fn parse_box_sizing(raw: &str) -> Result<layout::BoxSizing, Error> {
    match raw {
        "border-box" => Ok(layout::BoxSizing::BorderBox),
        "content-box" => Ok(layout::BoxSizing::ContentBox),
        _ => Err(Error::new(format!("unsupported box-sizing `{raw}`"))),
    }
}

fn parse_direction(raw: &str) -> Result<layout::Direction, Error> {
    match raw {
        "ltr" => Ok(layout::Direction::Ltr),
        "rtl" => Ok(layout::Direction::Rtl),
        _ => Err(Error::new(format!("unsupported direction `{raw}`"))),
    }
}

fn parse_position(raw: &str) -> Result<layout::Position, Error> {
    match raw {
        "relative" => Ok(layout::Position::Relative),
        "absolute" => Ok(layout::Position::Absolute),
        _ => Err(Error::new(format!("unsupported position `{raw}`"))),
    }
}

fn parse_float(raw: &str) -> Result<layout::Float, Error> {
    match raw {
        "none" => Ok(layout::Float::None),
        "left" => Ok(layout::Float::Left),
        "right" => Ok(layout::Float::Right),
        _ => Err(Error::new(format!("unsupported float `{raw}`"))),
    }
}

fn parse_clear(raw: &str) -> Result<layout::Clear, Error> {
    match raw {
        "none" => Ok(layout::Clear::None),
        "left" => Ok(layout::Clear::Left),
        "right" => Ok(layout::Clear::Right),
        "both" => Ok(layout::Clear::Both),
        _ => Err(Error::new(format!("unsupported clear `{raw}`"))),
    }
}

fn parse_overflow(raw: &str) -> Result<layout::Overflow, Error> {
    match raw {
        "visible" => Ok(layout::Overflow::Visible),
        "clip" => Ok(layout::Overflow::Clip),
        "hidden" => Ok(layout::Overflow::Hidden),
        "scroll" => Ok(layout::Overflow::Scroll),
        "auto" => Ok(layout::Overflow::Auto),
        _ => Err(Error::new(format!("unsupported overflow `{raw}`"))),
    }
}

fn parse_computed_overflow(attrs: &StyleAttrs) -> Result<layout::ComputedOverflow, Error> {
    let x = attrs
        .get("overflow-x")
        .map(parse_overflow)
        .transpose()?
        .unwrap_or(layout::Overflow::Visible);
    let y = attrs
        .get("overflow-y")
        .map(parse_overflow)
        .transpose()?
        .unwrap_or(layout::Overflow::Visible);

    layout::ComputedOverflow::try_new(x, y)
        .map_err(|error| Error::new(format!("invalid computed overflow pair: {error}")))
}

fn parse_overflow_clip_margin(raw: &str) -> Result<layout::OverflowClipMargin, Error> {
    let parts = raw.split_whitespace().collect::<Vec<_>>();
    let (clip_box, length) = match parts.as_slice() {
        [length] => (layout::OverflowClipBox::PaddingBox, *length),
        [clip_box, length] => {
            let clip_box = match *clip_box {
                "content-box" => layout::OverflowClipBox::ContentBox,
                "padding-box" => layout::OverflowClipBox::PaddingBox,
                "border-box" => layout::OverflowClipBox::BorderBox,
                _ => {
                    return Err(Error::new(format!(
                        "unsupported overflow clip box `{clip_box}`"
                    )));
                }
            };
            (clip_box, *length)
        }
        _ => {
            return Err(Error::new(format!(
                "unsupported overflow clip margin `{raw}`"
            )));
        }
    };
    let margin = parse_px(length, "overflow clip margin")?;
    layout::OverflowClipMargin::try_new(clip_box, margin)
        .map_err(|source| Error::new(source.to_string()))
}

fn parse_scrollbar_gutter(raw: &str) -> Result<layout::ScrollbarGutter, Error> {
    match raw {
        "auto" => Ok(layout::ScrollbarGutter::Auto),
        "stable" => Ok(layout::ScrollbarGutter::Stable),
        "stable both-edges" => Ok(layout::ScrollbarGutter::StableBothEdges),
        _ => Err(Error::new(format!("unsupported scrollbar gutter `{raw}`"))),
    }
}

fn parse_scroll_padding(raw: Option<&str>) -> Result<layout::ScrollPaddingValue, Error> {
    let Some(raw) = raw else {
        return Ok(layout::ScrollPaddingValue::AUTO);
    };
    if raw == "auto" {
        return Ok(layout::ScrollPaddingValue::AUTO);
    }
    let value = if raw.starts_with("calc(") {
        parse_calc_expression(raw)?
    } else if let Some(px) = raw.strip_suffix("px") {
        length_percentage_px(parse_number(px)?, raw)?
    } else if let Some(percent) = raw.strip_suffix('%') {
        length_percentage_percent(parse_number(percent)? / 100.0, raw)?
    } else {
        return Err(Error::new(format!("unsupported scroll padding `{raw}`")));
    };
    Ok(layout::ScrollPaddingValue::value(value))
}

fn parse_scroll_margin(raw: Option<&str>) -> Result<Scalar, Error> {
    raw.map_or(Ok(0.0), |raw| parse_px(raw, "scroll margin"))
}

fn parse_px(raw: &str, property: &str) -> Result<Scalar, Error> {
    raw.strip_suffix("px")
        .ok_or_else(|| Error::new(format!("unsupported {property} `{raw}`")))
        .and_then(parse_number)
}

fn parse_scroll_snap_type(raw: &str) -> Result<layout::ScrollSnapType, Error> {
    if raw == "none" {
        return Ok(layout::ScrollSnapType::None);
    }
    let parts = raw.split_whitespace().collect::<Vec<_>>();
    let [axis, strictness] = parts.as_slice() else {
        return Err(Error::new(format!("unsupported scroll snap type `{raw}`")));
    };
    let axis = match *axis {
        "x" => layout::ScrollSnapAxis::X,
        "y" => layout::ScrollSnapAxis::Y,
        "block" => layout::ScrollSnapAxis::Block,
        "inline" => layout::ScrollSnapAxis::Inline,
        "both" => layout::ScrollSnapAxis::Both,
        _ => return Err(Error::new(format!("unsupported scroll snap axis `{axis}`"))),
    };
    let strictness = match *strictness {
        "proximity" => layout::ScrollSnapStrictness::Proximity,
        "mandatory" => layout::ScrollSnapStrictness::Mandatory,
        _ => {
            return Err(Error::new(format!(
                "unsupported scroll snap strictness `{strictness}`"
            )));
        }
    };
    Ok(layout::ScrollSnapType::Enabled { axis, strictness })
}

fn parse_scroll_snap_align(raw: &str) -> Result<layout::ScrollSnapAlign, Error> {
    let parts = raw.split_whitespace().collect::<Vec<_>>();
    let [block, inline] = parts.as_slice() else {
        return Err(Error::new(format!("unsupported scroll snap align `{raw}`")));
    };
    let parse_value = |value| match value {
        "none" => Ok(layout::ScrollSnapAlignValue::None),
        "start" => Ok(layout::ScrollSnapAlignValue::Start),
        "end" => Ok(layout::ScrollSnapAlignValue::End),
        "center" => Ok(layout::ScrollSnapAlignValue::Center),
        _ => Err(Error::new(format!(
            "unsupported scroll snap alignment `{value}`"
        ))),
    };
    Ok(layout::ScrollSnapAlign::new(
        parse_value(block)?,
        parse_value(inline)?,
    ))
}

fn parse_scroll_snap_stop(raw: &str) -> Result<layout::ScrollSnapStop, Error> {
    match raw {
        "normal" => Ok(layout::ScrollSnapStop::Normal),
        "always" => Ok(layout::ScrollSnapStop::Always),
        _ => Err(Error::new(format!("unsupported scroll snap stop `{raw}`"))),
    }
}

fn parse_text_align(raw: &str) -> Result<layout::TextAlign, Error> {
    match raw {
        "left" | "-webkit-left" => Ok(layout::TextAlign::LegacyLeft),
        "right" | "-webkit-right" => Ok(layout::TextAlign::LegacyRight),
        "center" | "-webkit-center" => Ok(layout::TextAlign::LegacyCenter),
        _ => Err(Error::new(format!("unsupported text-align `{raw}`"))),
    }
}

fn parse_vertical_align(raw: &str) -> Result<layout::VerticalAlign, Error> {
    match raw {
        "baseline" => Ok(layout::VerticalAlign::Baseline),
        "top" => Ok(layout::VerticalAlign::Top),
        "bottom" => Ok(layout::VerticalAlign::Bottom),
        _ => Err(Error::new(format!(
            "unsupported parity fixture vertical-align `{raw}`"
        ))),
    }
}

fn parse_writing_mode(raw: Option<&str>) -> Result<layout::WritingMode, Error> {
    match raw {
        None | Some("horizontal-tb") => Ok(layout::WritingMode::HorizontalTb),
        Some("vertical-lr") => Ok(layout::WritingMode::VerticalLr),
        Some("vertical-rl") => Ok(layout::WritingMode::VerticalRl),
        Some("sideways-rl") => Ok(layout::WritingMode::SidewaysRl),
        Some("sideways-lr") => Ok(layout::WritingMode::SidewaysLr),
        Some(value) => Err(Error::new(format!("unsupported writing-mode `{value}`"))),
    }
}

fn parse_flex_direction(raw: &str) -> Result<layout::FlexDirection, Error> {
    match raw {
        "row" => Ok(layout::FlexDirection::Row),
        "column" => Ok(layout::FlexDirection::Column),
        "row-reverse" => Ok(layout::FlexDirection::RowReverse),
        "column-reverse" => Ok(layout::FlexDirection::ColumnReverse),
        _ => Err(Error::new(format!("unsupported flex-direction `{raw}`"))),
    }
}

fn parse_flex_wrap(raw: &str) -> Result<layout::FlexWrap, Error> {
    match raw {
        "nowrap" => Ok(layout::FlexWrap::NoWrap),
        "wrap" => Ok(layout::FlexWrap::Wrap),
        "wrap-reverse" => Ok(layout::FlexWrap::WrapReverse),
        _ => Err(Error::new(format!("unsupported flex-wrap `{raw}`"))),
    }
}

fn parse_align_items(raw: &str) -> Result<layout::AlignItems, Error> {
    let safe = raw.starts_with("safe ");
    let has_overflow_prefix = safe || raw.starts_with("unsafe ");
    let raw = alignment_keyword(raw);
    match (safe, has_overflow_prefix, raw) {
        (true, _, "end") => Ok(layout::AlignItems::SafeEnd),
        (true, _, "flex-end") => Ok(layout::AlignItems::SafeFlexEnd),
        (true, _, "center") => Ok(layout::AlignItems::SafeCenter),
        (_, _, "start") => Ok(layout::AlignItems::Start),
        (_, _, "end") => Ok(layout::AlignItems::End),
        (_, _, "flex-start") => Ok(layout::AlignItems::FlexStart),
        (_, _, "flex-end") => Ok(layout::AlignItems::FlexEnd),
        (_, _, "center") => Ok(layout::AlignItems::Center),
        (_, false, "baseline") => Ok(layout::AlignItems::Baseline),
        (_, false, "first baseline") => Ok(layout::AlignItems::Baseline),
        (_, false, "last baseline") => Ok(layout::AlignItems::LastBaseline),
        (_, _, "stretch") => Ok(layout::AlignItems::Stretch),
        _ => Err(Error::new(format!("unsupported alignment `{raw}`"))),
    }
}

fn parse_align_content(raw: &str) -> Result<layout::AlignContent, Error> {
    let safe = raw.starts_with("safe ");
    let raw = alignment_keyword(raw);
    match (safe, raw) {
        (true, "end") => Ok(layout::AlignContent::SafeEnd),
        (true, "flex-end") => Ok(layout::AlignContent::SafeFlexEnd),
        (true, "center") => Ok(layout::AlignContent::SafeCenter),
        (_, "start") => Ok(layout::AlignContent::Start),
        (_, "end") => Ok(layout::AlignContent::End),
        (_, "flex-start") => Ok(layout::AlignContent::FlexStart),
        (_, "flex-end") => Ok(layout::AlignContent::FlexEnd),
        (_, "center") => Ok(layout::AlignContent::Center),
        (_, "stretch") => Ok(layout::AlignContent::Stretch),
        (_, "space-between") => Ok(layout::AlignContent::SpaceBetween),
        (_, "space-evenly") => Ok(layout::AlignContent::SpaceEvenly),
        (_, "space-around") => Ok(layout::AlignContent::SpaceAround),
        _ => Err(Error::new(format!("unsupported content alignment `{raw}`"))),
    }
}

fn alignment_keyword(raw: &str) -> &str {
    raw.strip_prefix("safe ")
        .or_else(|| raw.strip_prefix("unsafe "))
        .unwrap_or(raw)
}

fn parse_grid_auto_flow(raw: &str) -> Result<layout::GridAutoFlow, Error> {
    match raw {
        "row" => Ok(layout::GridAutoFlow::Row),
        "column" => Ok(layout::GridAutoFlow::Column),
        "row dense" => Ok(layout::GridAutoFlow::RowDense),
        "column dense" => Ok(layout::GridAutoFlow::ColumnDense),
        _ => Err(Error::new(format!("unsupported grid-auto-flow `{raw}`"))),
    }
}

fn parse_calc_expression(raw: &str) -> Result<layout::LengthPercentageOf, Error> {
    let body = raw
        .strip_prefix("calc(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| Error::new(format!("unsupported calc expression `{raw}`")))?;
    parse_calc_sum(body.trim(), raw)
}

fn parse_calc_sum(body: &str, raw: &str) -> Result<layout::LengthPercentageOf, Error> {
    let parts = body.split_whitespace().collect::<Vec<_>>();
    let [first, operator, second] = parts.as_slice() else {
        return Err(Error::new(format!("unsupported calc expression `{raw}`")));
    };

    let left = parse_calc_term(first, 1.0)?;
    let right = match *operator {
        "+" => parse_calc_term(second, 1.0)?,
        "-" => parse_calc_term(second, -1.0)?,
        _ => return Err(Error::new(format!("unsupported calc expression `{raw}`"))),
    };

    layout::LengthPercentageOf::from_coefficients(
        left.absolute_px() + right.absolute_px(),
        left.percent_fraction() + right.percent_fraction(),
    )
    .map_err(|error| Error::new(format!("invalid calc expression `{raw}`: {error}")))
}

fn parse_calc_term(raw: &str, sign: Scalar) -> Result<layout::LengthPercentageOf, Error> {
    if let Some(px) = raw.strip_suffix("px") {
        return length_percentage_px(parse_number(px)? * sign, raw);
    }
    if let Some(percent) = raw.strip_suffix('%') {
        return length_percentage_percent(parse_number(percent)? / 100.0 * sign, raw);
    }
    Err(Error::new(format!(
        "unsupported calc expression term `{raw}`"
    )))
}

fn parse_length_with_calc(raw: &str) -> Result<layout::Length, Error> {
    if raw.trim_start().starts_with("calc(") {
        return Ok(layout::Length::value(parse_calc_expression(raw)?));
    }
    parse_length(raw)
}

fn parse_length_auto_with_calc(raw: &str) -> Result<layout::LengthAuto, Error> {
    if raw == "auto" {
        return Ok(layout::LengthAuto::AUTO);
    }
    Ok(parse_length_with_calc(raw)?.into())
}

fn parse_preferred_size(raw: &str) -> Result<layout::PreferredSize, Error> {
    let raw = checked_sizing_fixture(raw)?;
    match raw {
        "auto" => Ok(layout::PreferredSize::AUTO),
        "min-content" => Ok(layout::PreferredSize::MIN_CONTENT),
        "max-content" => Ok(layout::PreferredSize::MAX_CONTENT),
        "stretch" => Ok(layout::PreferredSize::STRETCH),
        "fit-content" => Ok(layout::PreferredSize::FIT_CONTENT),
        "contain" => Ok(layout::PreferredSize::CONTAIN),
        _ => match parse_sizing_function(raw)? {
            Some(("fit-content", body)) => Ok(layout::PreferredSize::fit_content_function(
                parse_fit_content_argument(body, raw)?,
            )),
            Some(("calc-size", body)) => {
                let (basis, calculation) = parse_calc_size_arguments(body, raw)?;
                let basis = parse_preferred_calc_size_basis(basis, raw)?;
                layout::PreferredSize::calc_size(basis, calculation).map_err(|error| {
                    Error::new(format!("invalid preferred-size fixture `{raw}`: {error}"))
                })
            }
            _ => Ok(layout::PreferredSize::calculation(
                parse_sizing_calculation_inner(raw)?,
            )),
        },
    }
}

fn parse_min_size(raw: &str) -> Result<layout::MinSize, Error> {
    let raw = checked_sizing_fixture(raw)?;
    match raw {
        "auto" => Ok(layout::MinSize::AUTO),
        "min-content" => Ok(layout::MinSize::MIN_CONTENT),
        "max-content" => Ok(layout::MinSize::MAX_CONTENT),
        "stretch" => Ok(layout::MinSize::STRETCH),
        "fit-content" => Ok(layout::MinSize::FIT_CONTENT),
        "contain" => Ok(layout::MinSize::CONTAIN),
        _ => match parse_sizing_function(raw)? {
            Some(("fit-content", body)) => Ok(layout::MinSize::fit_content_function(
                parse_fit_content_argument(body, raw)?,
            )),
            Some(("calc-size", body)) => {
                let (basis, calculation) = parse_calc_size_arguments(body, raw)?;
                let basis = parse_min_calc_size_basis(basis, raw)?;
                layout::MinSize::calc_size(basis, calculation).map_err(|error| {
                    Error::new(format!("invalid minimum-size fixture `{raw}`: {error}"))
                })
            }
            _ => Ok(layout::MinSize::calculation(
                parse_sizing_calculation_inner(raw)?,
            )),
        },
    }
}

fn parse_max_size(raw: &str) -> Result<layout::MaxSize, Error> {
    let raw = checked_sizing_fixture(raw)?;
    match raw {
        "none" => Ok(layout::MaxSize::NONE),
        "min-content" => Ok(layout::MaxSize::MIN_CONTENT),
        "max-content" => Ok(layout::MaxSize::MAX_CONTENT),
        "stretch" => Ok(layout::MaxSize::STRETCH),
        "fit-content" => Ok(layout::MaxSize::FIT_CONTENT),
        "contain" => Ok(layout::MaxSize::CONTAIN),
        _ => match parse_sizing_function(raw)? {
            Some(("fit-content", body)) => Ok(layout::MaxSize::fit_content_function(
                parse_fit_content_argument(body, raw)?,
            )),
            Some(("calc-size", body)) => {
                let (basis, calculation) = parse_calc_size_arguments(body, raw)?;
                let basis = parse_max_calc_size_basis(basis, raw)?;
                layout::MaxSize::calc_size(basis, calculation).map_err(|error| {
                    Error::new(format!("invalid maximum-size fixture `{raw}`: {error}"))
                })
            }
            _ => Ok(layout::MaxSize::calculation(
                parse_sizing_calculation_inner(raw)?,
            )),
        },
    }
}

fn parse_flex_basis(raw: &str) -> Result<layout::FlexBasis, Error> {
    let raw = checked_sizing_fixture(raw)?;
    match raw {
        "auto" => Ok(layout::FlexBasis::AUTO),
        "content" => Ok(layout::FlexBasis::CONTENT),
        "min-content" => Ok(layout::FlexBasis::MIN_CONTENT),
        "max-content" => Ok(layout::FlexBasis::MAX_CONTENT),
        "stretch" => Ok(layout::FlexBasis::STRETCH),
        "fit-content" => Ok(layout::FlexBasis::FIT_CONTENT),
        "contain" => Ok(layout::FlexBasis::CONTAIN),
        _ => match parse_sizing_function(raw)? {
            Some(("fit-content", body)) => Ok(layout::FlexBasis::fit_content_function(
                parse_fit_content_argument(body, raw)?,
            )),
            Some(("calc-size", body)) => {
                let (basis, calculation) = parse_calc_size_arguments(body, raw)?;
                let basis = parse_flex_calc_size_basis(basis, raw)?;
                layout::FlexBasis::calc_size(basis, calculation).map_err(|error| {
                    Error::new(format!("invalid flex-basis fixture `{raw}`: {error}"))
                })
            }
            _ => Ok(layout::FlexBasis::calculation(
                parse_sizing_calculation_inner(raw)?,
            )),
        },
    }
}

fn parse_length(raw: &str) -> Result<layout::Length, Error> {
    if let Some(px) = raw.strip_suffix("px") {
        return length_px(parse_number(px)?, raw);
    }
    if let Some(percent) = raw.strip_suffix('%') {
        return length_percent(parse_number(percent)? / 100.0, raw);
    }
    // Browser parity XML is a typed fixture format. Unitless fixture numbers
    // represent layout lengths; app-facing CSS parsing stays outside layout math.
    if let Ok(value) = parse_number(raw) {
        return length_px(value, raw);
    }
    Err(Error::new(format!("unsupported length `{raw}`")))
}

const MAX_SIZING_FUNCTION_DEPTH: usize = 64;

#[derive(Clone, Copy, Debug)]
struct FixtureSizingCoefficients {
    absolute_px: Scalar,
    percent_fraction: Scalar,
    size_fraction: Scalar,
}

fn checked_sizing_fixture(raw: &str) -> Result<&str, Error> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(Error::new("empty sizing fixture value"));
    }

    let mut depth = 0usize;
    for ch in raw.chars() {
        match ch {
            '(' => {
                depth += 1;
                if depth > MAX_SIZING_FUNCTION_DEPTH {
                    return Err(Error::new(format!(
                        "sizing function nesting depth {depth} exceeds {MAX_SIZING_FUNCTION_DEPTH} in `{raw}`"
                    )));
                }
            }
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return Err(Error::new(format!(
                        "unbalanced sizing fixture delimiters in `{raw}`"
                    )));
                };
                depth = next_depth;
            }
            '[' | ']' => {
                return Err(Error::new(format!(
                    "unsupported sizing fixture delimiter in `{raw}`"
                )));
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(Error::new(format!(
            "unbalanced sizing fixture delimiters in `{raw}`"
        )));
    }
    Ok(raw)
}

fn parse_sizing_function(raw: &str) -> Result<Option<(&str, &str)>, Error> {
    let Some(open_index) = raw.find('(') else {
        return Ok(None);
    };
    let name = &raw[..open_index];
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
    {
        return Err(Error::new(format!(
            "malformed sizing fixture function `{raw}`"
        )));
    }

    let mut depth = 0usize;
    let mut close_index = None;
    for (offset, ch) in raw[open_index..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return Err(Error::new(format!(
                        "unbalanced sizing fixture delimiters in `{raw}`"
                    )));
                };
                depth = next_depth;
                if depth == 0 {
                    close_index = Some(open_index + offset);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(close_index) = close_index else {
        return Err(Error::new(format!(
            "unbalanced sizing fixture delimiters in `{raw}`"
        )));
    };
    if close_index + 1 != raw.len() {
        return Err(Error::new(format!(
            "trailing input after sizing fixture function in `{raw}`"
        )));
    }
    Ok(Some((name, &raw[open_index + 1..close_index])))
}

fn split_sizing_arguments<'a>(body: &'a str, raw: &str) -> Result<Vec<&'a str>, Error> {
    let mut arguments = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in body.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return Err(Error::new(format!(
                        "unbalanced sizing fixture arguments in `{raw}`"
                    )));
                };
                depth = next_depth;
            }
            ',' if depth == 0 => {
                arguments.push(body[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(Error::new(format!(
            "unbalanced sizing fixture arguments in `{raw}`"
        )));
    }
    arguments.push(body[start..].trim());
    if arguments.iter().any(|argument| argument.is_empty()) {
        return Err(Error::new(format!(
            "empty sizing fixture function argument in `{raw}`"
        )));
    }
    Ok(arguments)
}

fn parse_fit_content_argument(body: &str, raw: &str) -> Result<layout::SizingCalculation, Error> {
    let arguments = split_sizing_arguments(body, raw)?;
    let [argument] = arguments.as_slice() else {
        return Err(Error::new(format!(
            "fit-content() requires exactly one argument in `{raw}`"
        )));
    };
    parse_sizing_calculation_inner(argument)
}

fn parse_calc_size_arguments<'a>(
    body: &'a str,
    raw: &str,
) -> Result<(&'a str, layout::CalcSizeCalculation), Error> {
    let arguments = split_sizing_arguments(body, raw)?;
    let [basis, calculation] = arguments.as_slice() else {
        return Err(Error::new(format!(
            "calc-size() requires exactly two arguments in `{raw}`"
        )));
    };
    Ok((basis, parse_calc_size_calculation_inner(calculation)?))
}

fn parse_preferred_calc_size_basis(
    basis: &str,
    raw: &str,
) -> Result<layout::PreferredSizeCalcBasis, Error> {
    match basis {
        "any" => Ok(layout::PreferredSizeCalcBasis::Any),
        "100%" => Ok(layout::PreferredSizeCalcBasis::FullPercentage),
        "auto" => Ok(layout::PreferredSizeCalcBasis::Auto),
        "min-content" => Ok(layout::PreferredSizeCalcBasis::MinContent),
        "max-content" => Ok(layout::PreferredSizeCalcBasis::MaxContent),
        "stretch" => Ok(layout::PreferredSizeCalcBasis::Stretch),
        "fit-content" => Ok(layout::PreferredSizeCalcBasis::FitContent),
        "contain" => Ok(layout::PreferredSizeCalcBasis::Contain),
        _ => Err(Error::new(format!(
            "invalid preferred-size calc-size basis `{basis}` in `{raw}`"
        ))),
    }
}

fn parse_min_calc_size_basis(basis: &str, raw: &str) -> Result<layout::MinSizeCalcBasis, Error> {
    match basis {
        "any" => Ok(layout::MinSizeCalcBasis::Any),
        "100%" => Ok(layout::MinSizeCalcBasis::FullPercentage),
        "auto" => Ok(layout::MinSizeCalcBasis::Auto),
        "min-content" => Ok(layout::MinSizeCalcBasis::MinContent),
        "max-content" => Ok(layout::MinSizeCalcBasis::MaxContent),
        "stretch" => Ok(layout::MinSizeCalcBasis::Stretch),
        "fit-content" => Ok(layout::MinSizeCalcBasis::FitContent),
        "contain" => Ok(layout::MinSizeCalcBasis::Contain),
        _ => Err(Error::new(format!(
            "invalid minimum-size calc-size basis `{basis}` in `{raw}`"
        ))),
    }
}

fn parse_max_calc_size_basis(basis: &str, raw: &str) -> Result<layout::MaxSizeCalcBasis, Error> {
    match basis {
        "any" => Ok(layout::MaxSizeCalcBasis::Any),
        "100%" => Ok(layout::MaxSizeCalcBasis::FullPercentage),
        "none" => Ok(layout::MaxSizeCalcBasis::None),
        "min-content" => Ok(layout::MaxSizeCalcBasis::MinContent),
        "max-content" => Ok(layout::MaxSizeCalcBasis::MaxContent),
        "stretch" => Ok(layout::MaxSizeCalcBasis::Stretch),
        "fit-content" => Ok(layout::MaxSizeCalcBasis::FitContent),
        "contain" => Ok(layout::MaxSizeCalcBasis::Contain),
        _ => Err(Error::new(format!(
            "invalid maximum-size calc-size basis `{basis}` in `{raw}`"
        ))),
    }
}

fn parse_flex_calc_size_basis(basis: &str, raw: &str) -> Result<layout::FlexBasisCalcBasis, Error> {
    match basis {
        "any" => Ok(layout::FlexBasisCalcBasis::Any),
        "100%" => Ok(layout::FlexBasisCalcBasis::FullPercentage),
        "auto" => Ok(layout::FlexBasisCalcBasis::Auto),
        "content" => Ok(layout::FlexBasisCalcBasis::Content),
        "min-content" => Ok(layout::FlexBasisCalcBasis::MinContent),
        "max-content" => Ok(layout::FlexBasisCalcBasis::MaxContent),
        "stretch" => Ok(layout::FlexBasisCalcBasis::Stretch),
        "fit-content" => Ok(layout::FlexBasisCalcBasis::FitContent),
        "contain" => Ok(layout::FlexBasisCalcBasis::Contain),
        _ => Err(Error::new(format!(
            "invalid flex-basis calc-size basis `{basis}` in `{raw}`"
        ))),
    }
}

fn parse_sizing_calculation_inner(raw: &str) -> Result<layout::SizingCalculation, Error> {
    let Some((name, body)) = parse_sizing_function(raw)? else {
        return Ok(layout::SizingCalculation::value(parse_sizing_leaf_value(
            raw,
        )?));
    };

    match name {
        "calc" => Ok(layout::SizingCalculation::value(parse_sizing_affine_value(
            body, false,
        )?)),
        "min" | "max" => {
            let arguments = split_sizing_arguments(body, raw)?;
            let calculations = arguments
                .into_iter()
                .map(parse_sizing_calculation_inner)
                .collect::<Result<Vec<_>, _>>()?;
            let calculation = if name == "min" {
                layout::SizingCalculation::min(calculations)
            } else {
                layout::SizingCalculation::max(calculations)
            };
            calculation.map_err(|error| {
                Error::new(format!("invalid sizing fixture function `{raw}`: {error}"))
            })
        }
        "clamp" => {
            let arguments = split_sizing_arguments(body, raw)?;
            let [minimum, preferred, maximum] = arguments.as_slice() else {
                return Err(Error::new(format!(
                    "clamp() requires exactly three arguments in `{raw}`"
                )));
            };
            if *preferred == "none" {
                return Err(Error::new(format!(
                    "clamp() preferred argument cannot be omitted in `{raw}`"
                )));
            }
            let minimum = (*minimum != "none")
                .then(|| parse_sizing_calculation_inner(minimum))
                .transpose()?;
            let preferred = parse_sizing_calculation_inner(preferred)?;
            let maximum = (*maximum != "none")
                .then(|| parse_sizing_calculation_inner(maximum))
                .transpose()?;
            Ok(layout::SizingCalculation::clamp(
                minimum, preferred, maximum,
            ))
        }
        _ => Err(Error::new(format!(
            "unsupported sizing fixture function `{name}` in `{raw}`"
        ))),
    }
}

fn parse_calc_size_calculation_inner(raw: &str) -> Result<layout::CalcSizeCalculation, Error> {
    let Some((name, body)) = parse_sizing_function(raw)? else {
        return calc_size_calculation_from_coefficients(
            parse_fixture_affine_coefficients(raw, true, true)?,
            raw,
        );
    };

    match name {
        "calc" => calc_size_calculation_from_coefficients(
            parse_fixture_affine_coefficients(body, true, false)?,
            raw,
        ),
        "min" | "max" => {
            let arguments = split_sizing_arguments(body, raw)?;
            let calculations = arguments
                .into_iter()
                .map(parse_calc_size_calculation_inner)
                .collect::<Result<Vec<_>, _>>()?;
            let calculation = if name == "min" {
                layout::CalcSizeCalculation::min(calculations)
            } else {
                layout::CalcSizeCalculation::max(calculations)
            };
            calculation.map_err(|error| {
                Error::new(format!(
                    "invalid calc-size fixture function `{raw}`: {error}"
                ))
            })
        }
        "clamp" => {
            let arguments = split_sizing_arguments(body, raw)?;
            let [minimum, preferred, maximum] = arguments.as_slice() else {
                return Err(Error::new(format!(
                    "clamp() requires exactly three arguments in `{raw}`"
                )));
            };
            if *preferred == "none" {
                return Err(Error::new(format!(
                    "clamp() preferred argument cannot be omitted in `{raw}`"
                )));
            }
            let minimum = (*minimum != "none")
                .then(|| parse_calc_size_calculation_inner(minimum))
                .transpose()?;
            let preferred = parse_calc_size_calculation_inner(preferred)?;
            let maximum = (*maximum != "none")
                .then(|| parse_calc_size_calculation_inner(maximum))
                .transpose()?;
            Ok(layout::CalcSizeCalculation::clamp(
                minimum, preferred, maximum,
            ))
        }
        _ => Err(Error::new(format!(
            "unsupported calc-size fixture function `{name}` in `{raw}`"
        ))),
    }
}

fn parse_sizing_leaf_value(raw: &str) -> Result<layout::LengthPercentageOf, Error> {
    let tokens = raw.split_whitespace().collect::<Vec<_>>();
    let [atom] = tokens.as_slice() else {
        return Err(Error::new(format!(
            "unsupported sizing fixture leaf `{raw}`"
        )));
    };
    let coefficients = parse_fixture_affine_atom(atom, false, true, raw)?;
    layout::LengthPercentageOf::from_coefficients(
        coefficients.absolute_px,
        coefficients.percent_fraction,
    )
    .map_err(|error| Error::new(format!("invalid sizing fixture `{raw}`: {error}")))
}

fn parse_sizing_affine_value(
    raw: &str,
    allow_unitless: bool,
) -> Result<layout::LengthPercentageOf, Error> {
    let coefficients = parse_fixture_affine_coefficients(raw, false, allow_unitless)?;
    layout::LengthPercentageOf::from_coefficients(
        coefficients.absolute_px,
        coefficients.percent_fraction,
    )
    .map_err(|error| Error::new(format!("invalid sizing fixture `{raw}`: {error}")))
}

fn calc_size_calculation_from_coefficients(
    coefficients: FixtureSizingCoefficients,
    raw: &str,
) -> Result<layout::CalcSizeCalculation, Error> {
    layout::CalcSizeCalculation::from_coefficients(
        coefficients.absolute_px,
        coefficients.percent_fraction,
        coefficients.size_fraction,
    )
    .map_err(|error| Error::new(format!("invalid calc-size fixture `{raw}`: {error}")))
}

fn parse_fixture_affine_coefficients(
    raw: &str,
    allow_size: bool,
    allow_unitless: bool,
) -> Result<FixtureSizingCoefficients, Error> {
    let tokens = raw.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return Err(Error::new(format!("empty affine sizing fixture `{raw}`")));
    }

    let mut coefficients = FixtureSizingCoefficients {
        absolute_px: 0.0,
        percent_fraction: 0.0,
        size_fraction: 0.0,
    };
    let mut start = 0usize;
    let mut sign = 1.0;
    loop {
        let end = tokens[start..]
            .iter()
            .position(|token| *token == "+" || *token == "-")
            .map_or(tokens.len(), |offset| start + offset);
        if end == start {
            return Err(Error::new(format!("missing affine sizing term in `{raw}`")));
        }
        let term = parse_fixture_affine_term(&tokens[start..end], allow_size, allow_unitless, raw)?;
        coefficients.absolute_px += term.absolute_px * sign;
        coefficients.percent_fraction += term.percent_fraction * sign;
        coefficients.size_fraction += term.size_fraction * sign;

        if end == tokens.len() {
            break;
        }
        sign = if tokens[end] == "+" { 1.0 } else { -1.0 };
        start = end + 1;
        if start == tokens.len() {
            return Err(Error::new(format!("missing affine sizing term in `{raw}`")));
        }
    }
    Ok(coefficients)
}

fn parse_fixture_affine_term(
    tokens: &[&str],
    allow_size: bool,
    allow_unitless: bool,
    raw: &str,
) -> Result<FixtureSizingCoefficients, Error> {
    match tokens {
        [atom] => parse_fixture_affine_atom(atom, allow_size, allow_unitless, raw),
        [left, "*", right] if allow_size => {
            let size_factor = match (*left, *right) {
                ("size", factor) => parse_number(factor)?,
                (factor, "size") => parse_number(factor)?,
                _ => {
                    return Err(Error::new(format!(
                        "unsupported affine sizing product in `{raw}`"
                    )));
                }
            };
            Ok(FixtureSizingCoefficients {
                absolute_px: 0.0,
                percent_fraction: 0.0,
                size_fraction: size_factor,
            })
        }
        _ => Err(Error::new(format!(
            "unsupported affine sizing term in `{raw}`"
        ))),
    }
}

fn parse_fixture_affine_atom(
    atom: &str,
    allow_size: bool,
    allow_unitless: bool,
    raw: &str,
) -> Result<FixtureSizingCoefficients, Error> {
    let mut coefficients = FixtureSizingCoefficients {
        absolute_px: 0.0,
        percent_fraction: 0.0,
        size_fraction: 0.0,
    };
    if let Some(px) = atom.strip_suffix("px") {
        coefficients.absolute_px = parse_number(px)?;
        return Ok(coefficients);
    }
    if let Some(percent) = atom.strip_suffix('%') {
        coefficients.percent_fraction = parse_number(percent)? / 100.0;
        return Ok(coefficients);
    }
    if allow_size {
        if atom == "size" {
            coefficients.size_fraction = 1.0;
            return Ok(coefficients);
        }
        if let Some(factor) = atom.strip_suffix("*size") {
            coefficients.size_fraction = parse_number(factor)?;
            return Ok(coefficients);
        }
        if let Some(factor) = atom.strip_prefix("size*") {
            coefficients.size_fraction = parse_number(factor)?;
            return Ok(coefficients);
        }
    }
    if allow_unitless {
        coefficients.absolute_px = parse_number(atom)?;
        return Ok(coefficients);
    }
    Err(Error::new(format!(
        "unsupported affine sizing term `{atom}` in `{raw}`"
    )))
}

fn length_percentage_px(value: Scalar, raw: &str) -> Result<layout::LengthPercentageOf, Error> {
    layout::LengthPercentageOf::px(value)
        .map_err(|error| Error::new(format!("invalid length `{raw}`: {error}")))
}

fn length_percentage_percent(
    value: Scalar,
    raw: &str,
) -> Result<layout::LengthPercentageOf, Error> {
    layout::LengthPercentageOf::from_percent_fraction(value)
        .map_err(|error| Error::new(format!("invalid length `{raw}`: {error}")))
}

fn length_px(value: Scalar, raw: &str) -> Result<layout::Length, Error> {
    Ok(layout::Length::value(length_percentage_px(value, raw)?))
}

fn length_percent(value: Scalar, raw: &str) -> Result<layout::Length, Error> {
    Ok(layout::Length::value(length_percentage_percent(
        value, raw,
    )?))
}

#[cfg(test)]
fn preferred_size_px(value: Scalar) -> layout::PreferredSize {
    layout::PreferredSize::value(
        layout::LengthPercentageOf::px(value).expect("finite test dimension px"),
    )
}

#[cfg(test)]
fn min_track_px(value: Scalar) -> layout::MinTrackSizing {
    layout::LengthPercentageOf::px(value)
        .expect("finite test min track px")
        .into()
}

#[cfg(test)]
fn max_track_px(value: Scalar) -> layout::MaxTrackSizing {
    layout::LengthPercentageOf::px(value)
        .expect("finite test max track px")
        .into()
}

#[cfg(test)]
fn track_px(value: Scalar) -> layout::TrackSizing {
    layout::LengthPercentageOf::px(value)
        .expect("finite test track px")
        .into()
}

#[cfg(test)]
fn track_component_px(value: Scalar) -> layout::TrackComponent {
    layout::TrackComponent::Track(track_px(value))
}

fn parse_track_component_list(raw: &str) -> Result<Vec<layout::TrackComponent>, Error> {
    parse_track_component_list_with_calc(raw)
}

fn parse_track_component_list_with_calc(raw: &str) -> Result<Vec<layout::TrackComponent>, Error> {
    if raw.trim_start().starts_with("subgrid") {
        return Ok(vec![parse_subgrid_track_component(raw)?]);
    }
    split_top_level_whitespace(raw)
        .into_iter()
        .map(|part| parse_track_component_with_calc(&part))
        .collect()
}

fn parse_track_component(raw: &str) -> Result<layout::TrackComponent, Error> {
    parse_track_component_with_calc(raw)
}

fn parse_track_component_with_calc(raw: &str) -> Result<layout::TrackComponent, Error> {
    if let Some(body) = function_body(raw, "repeat") {
        let (count, tracks) = split_once_top_level_comma(body)?;
        let repeat = match count.trim() {
            "auto-fill" => layout::TrackRepetition::auto_fill_components(
                parse_track_component_list_with_calc(tracks)?,
            ),
            "auto-fit" => layout::TrackRepetition::auto_fit_components(
                parse_track_component_list_with_calc(tracks)?,
            ),
            raw => layout::TrackRepetition::count_components(
                raw.parse()
                    .map_err(|_| Error::new(format!("invalid repeat count `{raw}`")))?,
                parse_track_component_list_with_calc(tracks)?,
            ),
        };
        let repeat =
            repeat.map_err(|error| Error::new(format!("invalid track repeat: {error}")))?;
        return Ok(layout::TrackComponent::Repeat(repeat));
    }
    if raw.starts_with('[') {
        return Ok(layout::TrackComponent::LineNames(parse_subgrid_line_names(
            raw,
        )?));
    }
    Ok(layout::TrackComponent::Track(parse_track_sizing_with_calc(
        raw,
    )?))
}

fn parse_subgrid_track_component(raw: &str) -> Result<layout::TrackComponent, Error> {
    let mut parts = split_top_level_whitespace(raw);
    if parts.first().map(String::as_str) != Some("subgrid") {
        return Err(Error::new(format!("invalid subgrid track list `{raw}`")));
    }
    let name_components = parts
        .drain(1..)
        .map(|part| parse_subgrid_line_name_component(&part))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(layout::TrackComponent::Subgrid(layout::SubgridTrack {
        name_components,
    }))
}

fn parse_subgrid_line_name_component(raw: &str) -> Result<layout::SubgridLineNameComponent, Error> {
    if let Some(body) = function_body(raw, "repeat") {
        let (count, names) = split_once_top_level_comma(body)?;
        let line_name_sets = split_top_level_whitespace(names)
            .into_iter()
            .map(|part| parse_subgrid_line_names(&part))
            .collect::<Result<Vec<_>, _>>()?;
        let count = match count.trim() {
            "auto-fill" => layout::SubgridLineNameRepeatCount::AutoFill,
            raw => layout::SubgridLineNameRepeatCount::Count(
                raw.parse()
                    .map_err(|_| Error::new(format!("invalid repeat count `{raw}`")))?,
            ),
        };
        return Ok(layout::SubgridLineNameComponent::Repeat {
            count,
            line_name_sets,
        });
    }
    Ok(layout::SubgridLineNameComponent::LineNames(
        parse_subgrid_line_names(raw)?,
    ))
}

fn parse_subgrid_line_names(raw: &str) -> Result<Vec<String>, Error> {
    let body = raw
        .strip_prefix('[')
        .and_then(|raw| raw.strip_suffix(']'))
        .ok_or_else(|| Error::new(format!("invalid subgrid line-name list `{raw}`")))?;
    body.split_whitespace()
        .map(|name| parse_custom_ident(name).map(ToOwned::to_owned))
        .collect::<Result<Vec<_>, _>>()
}

fn parse_track_sizing_with_calc(raw: &str) -> Result<layout::TrackSizing, Error> {
    let raw = checked_sizing_fixture(raw)?;
    if let Some(("minmax", body)) = parse_sizing_function(raw)? {
        let arguments = split_sizing_arguments(body, raw)?;
        let [min, max] = arguments.as_slice() else {
            return Err(Error::new(format!(
                "minmax() requires exactly two arguments in `{raw}`"
            )));
        };
        return Ok(layout::TrackSizing::minmax(
            parse_min_track_sizing_inner(min)?,
            parse_max_track_sizing_inner(max)?,
        ));
    }
    if let Some(("fit-content", body)) = parse_sizing_function(raw)? {
        return Ok(layout::TrackSizing::fit_content(
            parse_fit_content_argument(body, raw)?,
        ));
    }
    match raw {
        "auto" => Ok(layout::TrackSizing::AUTO),
        "min-content" => Ok(layout::TrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::TrackSizing::MAX_CONTENT),
        _ if raw.ends_with("fr") => Ok(layout::TrackSizing::flex(parse_track_flex(raw)?)),
        _ => Ok(layout::TrackSizing::calculation(
            parse_sizing_calculation_inner(raw)?,
        )),
    }
}

fn parse_min_track_sizing_with_calc(raw: &str) -> Result<layout::MinTrackSizing, Error> {
    let raw = checked_sizing_fixture(raw)?;
    parse_min_track_sizing_inner(raw)
}

fn parse_min_track_sizing_inner(raw: &str) -> Result<layout::MinTrackSizing, Error> {
    match raw {
        "auto" => Ok(layout::MinTrackSizing::AUTO),
        "min-content" => Ok(layout::MinTrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::MinTrackSizing::MAX_CONTENT),
        _ => Ok(layout::MinTrackSizing::Calculation(
            parse_sizing_calculation_inner(raw)?,
        )),
    }
}

fn parse_max_track_sizing_with_calc(raw: &str) -> Result<layout::MaxTrackSizing, Error> {
    let raw = checked_sizing_fixture(raw)?;
    parse_max_track_sizing_inner(raw)
}

fn parse_max_track_sizing_inner(raw: &str) -> Result<layout::MaxTrackSizing, Error> {
    if let Some(("fit-content", body)) = parse_sizing_function(raw)? {
        return Ok(layout::MaxTrackSizing::fit_content(
            parse_fit_content_argument(body, raw)?,
        ));
    }
    match raw {
        "auto" => Ok(layout::MaxTrackSizing::AUTO),
        "min-content" => Ok(layout::MaxTrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::MaxTrackSizing::MAX_CONTENT),
        _ if raw.ends_with("fr") => Ok(layout::MaxTrackSizing::flex(parse_track_flex(raw)?)),
        _ => Ok(layout::MaxTrackSizing::Calculation(
            parse_sizing_calculation_inner(raw)?,
        )),
    }
}

fn parse_track_flex(raw: &str) -> Result<layout::TrackFlexFactor, Error> {
    let value = raw
        .strip_suffix("fr")
        .ok_or_else(|| Error::new(format!("invalid track flex `{raw}`")))?;
    layout::TrackFlexFactor::try_new(parse_number(value)?)
        .map_err(|error| Error::new(format!("invalid track flex `{raw}`: {error}")))
}

fn function_body<'a>(raw: &'a str, name: &str) -> Option<&'a str> {
    raw.strip_prefix(name)
        .and_then(|raw| raw.strip_prefix('('))
        .and_then(|raw| raw.strip_suffix(')'))
}

fn split_once_top_level_comma(raw: &str) -> Result<(&str, &str), Error> {
    let mut depth = 0usize;
    for (index, ch) in raw.char_indices() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Ok((&raw[..index], &raw[index + 1..])),
            _ => {}
        }
    }
    Err(Error::new(format!("expected top-level comma in `{raw}`")))
}

fn split_top_level_whitespace(raw: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, ch) in raw.char_indices() {
        match ch {
            '(' | '[' => {
                depth += 1;
                start.get_or_insert(index);
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                start.get_or_insert(index);
            }
            ch if ch.is_whitespace() && depth == 0 => {
                if let Some(start_index) = start.take() {
                    parts.push(raw[start_index..index].to_string());
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }
    if let Some(start_index) = start {
        parts.push(raw[start_index..].to_string());
    }
    parts
}

fn parse_grid_line(raw: &str) -> Result<Option<isize>, Error> {
    if raw == "auto" || raw.starts_with("span ") {
        return Ok(None);
    }
    parse_grid_line_number(raw).map(Some)
}

fn parse_grid_line_or_span(raw: &str, span: &mut Option<usize>) -> Result<Option<isize>, Error> {
    if raw == "auto" {
        return Ok(None);
    }
    if let Some(raw_span) = raw.strip_prefix("span ") {
        *span = Some(parse_nonnegative_line(raw_span)?);
        return Ok(None);
    }
    parse_grid_line_number(raw).map(Some)
}

fn parse_grid_line_number(raw: &str) -> Result<isize, Error> {
    raw.parse()
        .map_err(|_| Error::new(format!("invalid grid line `{raw}`")))
}

fn parse_nonnegative_line(raw: &str) -> Result<usize, Error> {
    let value: i64 = raw
        .parse()
        .map_err(|_| Error::new(format!("invalid grid line `{raw}`")))?;
    if value < 0 {
        return Err(Error::new(format!(
            "negative grid line `{raw}` is not supported by Surgeist layout yet"
        )));
    }
    usize::try_from(value).map_err(|_| Error::new(format!("grid line `{raw}` is too large")))
}

fn collect_files_with_extension(
    dir: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    for entry in std::fs::read_dir(dir).map_err(|source| Error::new(source.to_string()))? {
        let entry = entry.map_err(|source| Error::new(source.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, files)?;
        } else if path
            .extension()
            .and_then(|actual| actual.to_str())
            .is_some_and(|actual| actual == extension)
        {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_node_input(attrs: StyleAttrs) -> Result<layout::NodeInput, Error> {
        to_node_input(&attrs)
    }

    fn overflow_attrs(x: Option<&str>, y: Option<&str>) -> StyleAttrs {
        let mut attrs = BTreeMap::new();
        if let Some(x) = x {
            attrs.insert("overflow-x".to_string(), x.to_string());
        }
        if let Some(y) = y {
            attrs.insert("overflow-y".to_string(), y.to_string());
        }
        StyleAttrs { attrs }
    }

    fn observed_overflow_axes(input: &layout::NodeInput) -> (layout::Overflow, layout::Overflow) {
        (input.overflow.x(), input.overflow.y())
    }

    fn fri05_c06_attrs(values: &[(&str, &str)]) -> StyleAttrs {
        StyleAttrs {
            attrs: values
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
        }
    }

    #[test]
    fn fri05_c06_parser_lowers_finite_scroll_fields_through_production_types() {
        let input = test_node_input(fri05_c06_attrs(&[
            ("overflow-x", "hidden"),
            ("overflow-y", "auto"),
            ("overflow-clip-margin", "content-box 3.5px"),
            ("scrollbar-gutter", "stable both-edges"),
            ("scroll-padding-top", "auto"),
            ("scroll-padding-right", "12px"),
            ("scroll-padding-bottom", "25%"),
            ("scroll-padding-left", "calc(4px + 10%)"),
            ("scroll-margin-top", "-1px"),
            ("scroll-margin-right", "2px"),
            ("scroll-margin-bottom", "3.5px"),
            ("scroll-margin-left", "-4px"),
            ("scroll-snap-type", "inline mandatory"),
            ("scroll-snap-align", "start center"),
            ("scroll-snap-stop", "always"),
        ]))
        .expect("finite computed scroll fixture fields should lower");

        assert_eq!(
            observed_overflow_axes(&input),
            (layout::Overflow::Hidden, layout::Overflow::Auto)
        );
        assert_eq!(
            input.overflow_clip_margin.clip_box(),
            layout::OverflowClipBox::ContentBox
        );
        assert_eq!(input.overflow_clip_margin.margin(), 3.5);
        assert_eq!(
            input.scrollbar_gutter,
            layout::ScrollbarGutter::StableBothEdges
        );
        assert!(input.scroll_padding.top().is_auto());
        for (actual, px, percent) in [
            (input.scroll_padding.right(), 12.0, 0.0),
            (input.scroll_padding.bottom(), 0.0, 0.25),
            (input.scroll_padding.left(), 4.0, 0.1),
        ] {
            let layout::ScrollPaddingValue::Value(actual) = actual else {
                panic!("expected numeric scroll padding, got {actual:?}");
            };
            assert_eq!(
                (actual.absolute_px(), actual.percent_fraction()),
                (px, percent)
            );
        }
        assert_eq!(
            (
                input.scroll_margin.top(),
                input.scroll_margin.right(),
                input.scroll_margin.bottom(),
                input.scroll_margin.left(),
            ),
            (-1.0, 2.0, 3.5, -4.0)
        );
        assert_eq!(
            input.scroll_snap_type,
            layout::ScrollSnapType::Enabled {
                axis: layout::ScrollSnapAxis::Inline,
                strictness: layout::ScrollSnapStrictness::Mandatory,
            }
        );
        assert_eq!(
            input.scroll_snap_align,
            layout::ScrollSnapAlign::new(
                layout::ScrollSnapAlignValue::Start,
                layout::ScrollSnapAlignValue::Center,
            )
        );
        assert_eq!(input.scroll_snap_stop, layout::ScrollSnapStop::Always);
    }

    #[test]
    fn fri05_c06_parser_accepts_exact_keyword_domains_and_initials() {
        for overflow in ["visible", "clip", "hidden", "scroll", "auto"] {
            assert!(
                parse_overflow(overflow).is_ok(),
                "rejected overflow {overflow}"
            );
        }
        for (name, value) in [
            ("overflow-clip-margin", "border-box 0px"),
            ("scrollbar-gutter", "stable"),
            ("scroll-snap-type", "both proximity"),
            ("scroll-snap-align", "none end"),
            ("scroll-snap-stop", "normal"),
        ] {
            test_node_input(fri05_c06_attrs(&[(name, value)]))
                .unwrap_or_else(|error| panic!("rejected {name}={value:?}: {error}"));
        }
    }

    #[test]
    fn fri05_c06_parser_rejects_ambiguous_or_non_computed_scroll_syntax() {
        for (name, value) in [
            ("overflow", "hidden"),
            ("scroll-padding", "1px 2px"),
            ("scroll-margin", "1px"),
            ("transform", "translateX(1px)"),
            ("overflow-clip-margin", "inherit"),
            ("overflow-clip-margin", "padding-box -1px"),
            ("overflow-clip-margin", "padding-box 1em"),
            ("overflow-clip-margin", "padding-box var(--clip)"),
            ("scrollbar-gutter", "stable force"),
            ("scroll-padding-top", "initial"),
            ("scroll-padding-top", "1em"),
            ("scroll-padding-top", "var(--padding)"),
            ("scroll-padding-top", "NaNpx"),
            ("scroll-margin-left", "10%"),
            ("scroll-margin-left", "infpx"),
            ("scroll-snap-type", "x"),
            ("scroll-snap-type", "x mandatory extra"),
            ("scroll-snap-align", "start"),
            ("scroll-snap-align", "start center end"),
            ("scroll-snap-stop", "inherit"),
        ] {
            assert!(
                test_node_input(fri05_c06_attrs(&[(name, value)])).is_err(),
                "accepted {name}={value:?}"
            );
        }
    }

    #[test]
    fn fri05_c06_parser_rejects_noncanonical_computed_overflow_constructor_pairs() {
        assert!(
            layout::ComputedOverflow::try_new(layout::Overflow::Visible, layout::Overflow::Auto)
                .is_err()
        );
        assert!(
            layout::ComputedOverflow::try_new(layout::Overflow::Clip, layout::Overflow::Scroll)
                .is_err()
        );
    }

    #[test]
    fn fri05_c06_computed_overflow_transition_accepts_direct_valid_computed_pairs() {
        use layout::Overflow::{Auto, Clip, Hidden, Scroll, Visible};

        for (x_token, y_token, expected) in [
            ("visible", "visible", (Visible, Visible)),
            ("visible", "clip", (Visible, Clip)),
            ("clip", "visible", (Clip, Visible)),
            ("clip", "clip", (Clip, Clip)),
            ("hidden", "hidden", (Hidden, Hidden)),
            ("hidden", "scroll", (Hidden, Scroll)),
            ("hidden", "auto", (Hidden, Auto)),
            ("scroll", "hidden", (Scroll, Hidden)),
            ("scroll", "scroll", (Scroll, Scroll)),
            ("scroll", "auto", (Scroll, Auto)),
            ("auto", "hidden", (Auto, Hidden)),
            ("auto", "scroll", (Auto, Scroll)),
            ("auto", "auto", (Auto, Auto)),
        ] {
            let input = test_node_input(overflow_attrs(Some(x_token), Some(y_token)))
                .unwrap_or_else(|error| {
                    panic!("rejected valid pair ({x_token}, {y_token}): {error}")
                });
            assert_eq!(observed_overflow_axes(&input), expected);
        }
    }

    #[test]
    fn fri05_c06_computed_overflow_transition_rejects_direct_invalid_computed_pairs() {
        for (x, y) in [
            ("visible", "hidden"),
            ("visible", "scroll"),
            ("visible", "auto"),
            ("clip", "hidden"),
            ("clip", "scroll"),
            ("clip", "auto"),
            ("hidden", "visible"),
            ("hidden", "clip"),
            ("scroll", "visible"),
            ("scroll", "clip"),
            ("auto", "visible"),
            ("auto", "clip"),
        ] {
            assert!(
                test_node_input(overflow_attrs(Some(x), Some(y))).is_err(),
                "accepted invalid computed pair ({x}, {y})"
            );
        }
    }

    fn fri05_c06_scroll_expectation(width: Scalar, height: Scalar) -> Golden {
        let mut golden = Golden::parse(
            r#"
            <test name="fri05-c06-nonzero" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="flex" overflow-x="scroll" overflow-y="scroll"
                         scrollbar-width="15" align-items="start" justify-content="end"
                         width="50px" height="50px">
                        <div display="flex" flex-shrink="0" width="100px" height="100px" />
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="50" height="50"
                          scroll_width="65" scroll_height="65">
                        <node x="-65" y="0" width="100" height="100" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("scroll expectation fixture should parse");
        golden.expectations.scroll_size = Some(Size::new(width, height));
        golden
    }

    #[test]
    fn fri05_c06_comparator_signed_range_uses_span_instead_of_maximum_endpoint() {
        assert_surgeist_matches(&fri05_c06_scroll_expectation(65.0, 65.0))
            .expect("canonical signed range span should match");
    }

    #[test]
    fn fri05_c06_comparator_explicit_zero_range_spans_pass() {
        let golden = Golden::parse(
            r#"
            <test name="fri05-c06-zero" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div overflow-x="hidden" overflow-y="hidden" width="50px" height="50px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="50" height="50"
                          scroll_width="0" scroll_height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("zero scroll expectation fixture should parse");

        assert_surgeist_matches(&golden).expect("explicit zero range spans should match");
    }

    #[test]
    fn fri05_c06_comparator_wrong_x_range_span_names_scroll_width_mismatch() {
        let error = assert_surgeist_matches(&fri05_c06_scroll_expectation(64.0, 65.0))
            .expect_err("wrong x range span should fail");

        assert_eq!(
            error.to_string(),
            "fri05-c06-nonzero: scroll width mismatch, expected 64, got 65"
        );
    }

    #[test]
    fn fri05_c06_comparator_wrong_y_range_span_names_scroll_height_mismatch() {
        let error = assert_surgeist_matches(&fri05_c06_scroll_expectation(65.0, 64.0))
            .expect_err("wrong y range span should fail");

        assert_eq!(
            error.to_string(),
            "fri05-c06-nonzero: scroll height mismatch, expected 64, got 65"
        );
    }

    #[test]
    fn fri05_c06_comparator_wrong_scroll_span_precedes_invalid_child_mismatch() {
        let mut golden = fri05_c06_scroll_expectation(64.0, 65.0);
        golden.expectations.children[0].x = Some(-64.0);

        let error = assert_surgeist_matches(&golden)
            .expect_err("wrong root scroll span should fail before the invalid child");

        assert_eq!(
            error.to_string(),
            "fri05-c06-nonzero: scroll width mismatch, expected 64, got 65"
        );
    }

    #[test]
    fn fri05_c06_comparator_absent_geometry_names_scroll_geometry_mismatch() {
        let golden = Golden::parse(
            r#"
            <test name="fri05-c06-missing-geometry" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="none" />
                </input>
                <expectations>
                    <node scroll_width="0" scroll_height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("missing geometry fixture should parse");
        let error = assert_surgeist_matches(&golden)
            .expect_err("missing canonical scroll geometry should fail");

        assert_eq!(
            error.to_string(),
            "fri05-c06-missing-geometry: scroll geometry mismatch, expected canonical geometry"
        );
    }

    fn line_break_tree(input: layout::LineBreakInput) -> TestTree {
        TestTree {
            nodes: vec![TestNode {
                node_input: layout::NodeInput::non_box(),
                layout_input: layout::LayoutInput::LineBreak(input),
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: TextMeasure::LINE_HEIGHT,
                text: None,
                children: Vec::new(),
                synthetic: false,
                preserve_fractional_min_content: false,
                use_tighter_monospace_wrap: false,
                cache: layout::Cache::new(),
                unrounded: layout::NodeOutput::new(),
                unrounded_present: true,
                final_layout: layout::NodeOutput::new(),
                final_layout_present: true,
                unrounded_inline_fragments: None,
                final_inline_fragments: None,
                shape_bands: None,
            }],
        }
    }

    #[test]
    fn fri06_c06_comparator_wrong_control_x_names_x_mismatch() {
        let tree = line_break_tree(layout::LineBreakInput::new());
        let expected = Expectation {
            x: Some(1.0),
            y: Some(0.0),
            width: Some(0.0),
            height: Some(0.0),
            scroll_size: None,
            fragments: None,
            range_inks: None,
            children: vec![Expectation {
                x: Some(99.0),
                y: None,
                width: None,
                height: None,
                scroll_size: None,
                fragments: None,
                range_inks: None,
                children: Vec::new(),
            }],
        };

        let error = compare_expectation(&tree, 0, &expected, "fri06-c06-control", true, &[])
            .expect_err("wrong control x should fail before unrelated child comparison");

        assert_eq!(
            error.to_string(),
            "fri06-c06-control: x mismatch, expected 1, got 0"
        );
    }

    #[test]
    fn fri06_c06_comparator_zero_control_geometry_passes_and_missing_output_is_named() {
        let expected = Expectation {
            x: Some(0.0),
            y: Some(0.0),
            width: Some(0.0),
            height: Some(0.0),
            scroll_size: None,
            fragments: None,
            range_inks: None,
            children: Vec::new(),
        };
        let mut tree = line_break_tree(layout::LineBreakInput::new());
        compare_expectation(&tree, 0, &expected, "fri06-c06-control", true, &[])
            .expect("published zero control geometry should compare");

        tree.nodes[0].final_layout_present = false;
        let error = compare_expectation(&tree, 0, &expected, "fri06-c06-control", true, &[])
            .expect_err("missing control output should fail");
        assert_eq!(
            error.to_string(),
            "fri06-c06-control: control geometry mismatch, expected final output"
        );
    }

    fn fri06_c06_fragment_xml(fragment_body: &str) -> String {
        format!(
            r#"
            <test name="fri06-c06-fragment-parser" use-rounding="true">
                <viewport width="100" height="max-content" />
                <input><div display="block" /></input>
                <expectations>
                    <node>{fragment_body}</node>
                </expectations>
            </test>
            "#
        )
    }

    fn fri06_c08_range_ink_golden() -> Golden {
        Golden::parse(
            r#"
            <test name="fri06-c08-range-ink" use-rounding="false">
                <viewport width="100px" height="max-content" />
                <input>
                    <div display="block" width="100px">
                        <text layout-input="inline-text">
                            <segment id="11" inline-extent="10" inline-baseline="8" inline-line-height="10" bidi-level="0" whitespace-edge="preserve" following-break="prohibited" />
                        </text>
                    </div>
                </input>
                <expectations>
                    <node>
                        <node>
                            <range-inks>
                                <range-ink source_segment_id="11" line_index="0" visual_index="0" physical_start_edge="left" start="0" advance="10" />
                            </range-inks>
                        </node>
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("Range ink should parse as a distinct finite observation category")
    }

    #[test]
    fn fri06_c08_range_ink_parser_and_comparator_ignore_browser_block_ink_geometry() {
        let golden = fri06_c08_range_ink_golden();

        assert_surgeist_matches(&golden).expect(
            "Range ink should compare source/line/visual and flow-inline facts without browser block ink geometry",
        );
    }

    #[test]
    fn fri06_c08_range_ink_wrong_identity_or_inline_interval_still_fails() {
        for (field, expected_diagnostic) in [
            (
                "source",
                "Range ink[0] source segment id mismatch, expected 12, got 11",
            ),
            (
                "line",
                "Range ink[0] line index mismatch, expected 1, got 0",
            ),
            (
                "visual",
                "Range ink[0] visual index mismatch, expected 1, got 0",
            ),
            (
                "start",
                "Range ink[0] physical flow-inline start mismatch, expected 1, got 0",
            ),
            (
                "advance",
                "Range ink[0] advance mismatch, expected 11, got 10",
            ),
        ] {
            let mut golden = fri06_c08_range_ink_golden();
            let range_ink = &mut golden.expectations.children[0].range_inks.as_mut().unwrap()[0];
            match field {
                "source" => range_ink.source_segment_id += 1,
                "line" => range_ink.line_index += 1,
                "visual" => range_ink.visual_index += 1,
                "start" => range_ink.start += 1.0,
                "advance" => range_ink.advance += 1.0,
                _ => unreachable!(),
            }

            let error = assert_surgeist_matches(&golden)
                .expect_err("wrong Range-ink identity or inline interval should fail");
            assert!(
                error.to_string().contains(expected_diagnostic),
                "unexpected {field} diagnostic: {error}"
            );
        }
    }

    #[test]
    fn fri06_c08_range_ink_parser_is_finite_complete_and_category_exclusive() {
        let complete = r#"<range-ink source_segment_id="11" line_index="0" visual_index="0" physical_start_edge="left" start="0" advance="10" />"#;
        for (body, diagnostic) in [
            (
                "<range-inks />".to_string(),
                "expected at least one `<range-ink>` child on `<range-inks>`",
            ),
            (
                r#"<range-inks><range-ink source_segment_id="11" line_index="0" visual_index="0" physical_start_edge="left" start="0" /></range-inks>"#.to_string(),
                "missing `advance` on `<range-ink>`",
            ),
            (
                r#"<range-inks><range-ink source_segment_id="11" line_index="0" visual_index="0" physical_start_edge="inline" start="0" advance="10" /></range-inks>"#.to_string(),
                "invalid `physical_start_edge` on `<range-ink>`: `inline`",
            ),
            (
                r#"<range-inks><range-ink source_segment_id="11" line_index="0" visual_index="0" physical_start_edge="left" start="NaN" advance="10" /></range-inks>"#.to_string(),
                "invalid `start` on `<range-ink>`: `NaN`",
            ),
            (
                r#"<range-inks><range-ink source_segment_id="11" line_index="0" visual_index="0" physical_start_edge="left" start="0" advance="-1" /></range-inks>"#.to_string(),
                "invalid `advance` on `<range-ink>`: `-1`",
            ),
            (
                format!(r#"<range-inks>{complete}</range-inks><fragments>{}</fragments>"#, fri06_c06_complete_fragment("")),
                "model fragments and Range ink are distinct expectation categories",
            ),
        ] {
            let error = Golden::parse(&fri06_c06_fragment_xml(&body))
                .expect_err("invalid or mixed Range-ink category should fail closed");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    #[test]
    fn fri06_c06_comparator_parser_distinguishes_absent_and_explicit_empty_fragments() {
        let absent = Golden::parse(&fri06_c06_fragment_xml(""))
            .expect("legacy expectation without fragments should parse");
        let empty = Golden::parse(&fri06_c06_fragment_xml("<fragments />"))
            .expect("explicit empty fragment state should parse");

        assert_eq!(absent.expectations.fragments, None);
        assert_eq!(empty.expectations.fragments, Some(Vec::new()));
    }

    #[test]
    fn fri06_c06_comparator_parser_requires_every_fragment_field() {
        let fields = [
            ("source_segment_id", "11"),
            ("line_index", "0"),
            ("visual_index", "2"),
            ("x", "1.25"),
            ("y", "2.5"),
            ("width", "10.25"),
            ("height", "10"),
            ("baseline_x", "1.25"),
            ("baseline_y", "10.5"),
        ];

        for (missing, _) in fields {
            let attrs = fields
                .iter()
                .filter(|(name, _)| *name != missing)
                .map(|(name, value)| format!(r#"{name}="{value}""#))
                .collect::<Vec<_>>()
                .join(" ");
            let xml =
                fri06_c06_fragment_xml(&format!("<fragments><fragment {attrs} /></fragments>"));
            let error = Golden::parse(&xml).expect_err("missing fragment field should fail");

            assert_eq!(
                error.to_string(),
                format!("missing `{missing}` on `<fragment>`")
            );
        }
    }

    #[test]
    fn fri06_c06_comparator_parser_rejects_nonfinite_negative_and_unknown_fragment_facts() {
        for (fragment, diagnostic) in [
            (
                r#"<fragment source_segment_id="11" line_index="0" visual_index="2" x="NaN" y="2.5" width="10.25" height="10" baseline_x="1.25" baseline_y="10.5" />"#,
                "invalid `x` on `<fragment>`: `NaN`",
            ),
            (
                r#"<fragment source_segment_id="11" line_index="0" visual_index="2" x="1.25" y="2.5" width="-1" height="10" baseline_x="1.25" baseline_y="10.5" />"#,
                "invalid `width` on `<fragment>`: `-1`",
            ),
            (
                r#"<fragment source_segment_id="11" line_index="0" visual_index="2" x="1.25" y="2.5" width="10.25" height="10" baseline_x="1.25" baseline_y="10.5" fallback="true" />"#,
                "unsupported `<fragment>` attribute `fallback`",
            ),
        ] {
            let xml = fri06_c06_fragment_xml(&format!("<fragments>{fragment}</fragments>"));
            let error = Golden::parse(&xml).expect_err("invalid fragment fact should fail");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    fn fri06_c06_complete_fragment(payload: &str) -> String {
        format!(
            r#"<fragment source_segment_id="11" line_index="0" visual_index="2" x="1.25" y="2.5" width="10.25" height="10" baseline_x="1.25" baseline_y="10.5">{payload}</fragment>"#
        )
    }

    #[test]
    fn fri06_c06_comparator_fragments_reject_non_whitespace_text() {
        let xml = fri06_c06_fragment_xml("<fragments>payload</fragments>");
        let error = Golden::parse(&xml).expect_err("non-whitespace text in fragments should fail");
        assert_eq!(
            error.to_string(),
            "unsupported non-whitespace text in `<fragments>`"
        );
    }

    #[test]
    fn fri06_c06_comparator_fragments_reject_nested_element() {
        let xml = fri06_c06_fragment_xml("<fragments><nested /></fragments>");
        let error = Golden::parse(&xml).expect_err("unknown fragments child should fail");
        assert_eq!(
            error.to_string(),
            "unsupported `<fragments>` child `<nested>`"
        );
    }

    #[test]
    fn fri06_c06_comparator_fragment_rejects_non_whitespace_text() {
        let fragment = fri06_c06_complete_fragment("payload");
        let xml = fri06_c06_fragment_xml(&format!("<fragments>{fragment}</fragments>"));
        let error = Golden::parse(&xml).expect_err("non-whitespace text in fragment should fail");
        assert_eq!(
            error.to_string(),
            "unsupported non-whitespace text in `<fragment>`"
        );
    }

    #[test]
    fn fri06_c06_comparator_fragment_rejects_nested_element() {
        let fragment = fri06_c06_complete_fragment("<nested />");
        let xml = fri06_c06_fragment_xml(&format!("<fragments>{fragment}</fragments>"));
        let error = Golden::parse(&xml).expect_err("unknown fragment child should fail");
        assert_eq!(
            error.to_string(),
            "unsupported `<fragment>` child `<nested>`"
        );
    }

    struct Fri06C06FragmentTree {
        layout_inputs: Vec<layout::LayoutInput>,
        node_inputs: Vec<layout::NodeInput>,
        children: Vec<Vec<usize>>,
    }

    impl layout::Traverse for Fri06C06FragmentTree {
        type Node = usize;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[node][index]
        }
    }

    impl layout::LayoutTree for Fri06C06FragmentTree {
        type MeasureError = std::convert::Infallible;

        fn node_input(&self, node: Self::Node) -> &layout::NodeInput {
            &self.node_inputs[node]
        }

        fn layout_input(&self, node: Self::Node) -> layout::LayoutInput {
            self.layout_inputs[node].clone()
        }
    }

    fn fri06_c06_segment(id: u64, extent: Scalar) -> layout::ShapedInlineSegment {
        layout::ShapedInlineSegment::try_new(
            layout::InlineSegmentId::new(id),
            extent,
            layout::InlineMetrics::from_ascent_descent(8.0, 2.0).unwrap(),
            layout::BidiLevel::try_new(0).unwrap(),
            layout::InlineWhitespaceEdge::Preserve,
            layout::InlineBreakOpportunity::prohibited(),
        )
        .unwrap()
    }

    fn fri06_c06_fragment_observation()
    -> (TestTree, layout::CompletedLayoutBatch<usize>, Expectation) {
        let root_input = layout::NodeInput {
            display: layout::Display::Block,
            ..layout::NodeInput::default()
        };
        let text = |id, extent| {
            layout::LayoutInput::inline_text(
                layout::InlineTextInput::try_new(vec![fri06_c06_segment(id, extent)]).unwrap(),
            )
        };
        let metrics = layout::InlineMetrics::from_ascent_descent(8.0, 2.0).unwrap();
        let source = Fri06C06FragmentTree {
            layout_inputs: vec![
                layout::LayoutInput::box_input(root_input.clone()),
                text(11, 10.25),
                layout::LayoutInput::inline_boundary(layout::InlineBoundaryInput::new(
                    layout::InlineBoundaryKind::Start,
                    metrics,
                )),
                text(22, 5.0),
            ],
            node_inputs: vec![
                root_input,
                layout::NodeInput::non_box(),
                layout::NodeInput::non_box(),
                layout::NodeInput::non_box(),
            ],
            children: vec![vec![1, 2, 3], Vec::new(), Vec::new(), Vec::new()],
        };
        let request = layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::definite(100.0),
            layout::Available::MaxContent,
        ))
        .unwrap();
        let batch = layout::compute_layout(&source, 0, request).unwrap();
        let nodes = source
            .layout_inputs
            .iter()
            .enumerate()
            .map(|(node, layout_input)| TestNode {
                node_input: source.node_inputs[node].clone(),
                layout_input: layout_input.clone(),
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: TextMeasure::LINE_HEIGHT,
                text: None,
                children: source.children[node].clone(),
                synthetic: false,
                preserve_fractional_min_content: false,
                use_tighter_monospace_wrap: false,
                cache: layout::Cache::new(),
                unrounded: batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .map_or_else(layout::NodeOutput::new, |entry| entry.output()),
                unrounded_present: batch
                    .unrounded_entries()
                    .iter()
                    .any(|entry| entry.node() == node),
                final_layout: batch
                    .final_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .map_or_else(layout::NodeOutput::new, |entry| entry.output()),
                final_layout_present: batch
                    .final_entries()
                    .iter()
                    .any(|entry| entry.node() == node),
                unrounded_inline_fragments: matches!(
                    layout_input,
                    layout::LayoutInput::InlineText(_)
                )
                .then(|| {
                    batch
                        .unrounded_inline_fragments()
                        .iter()
                        .filter(|entry| entry.node() == node)
                        .map(|entry| entry.fragment())
                        .collect()
                }),
                final_inline_fragments: matches!(layout_input, layout::LayoutInput::InlineText(_))
                    .then(|| {
                        batch
                            .final_inline_fragments()
                            .iter()
                            .filter(|entry| entry.node() == node)
                            .map(|entry| entry.fragment())
                            .collect()
                    }),
                shape_bands: None,
            })
            .collect();
        let tree = TestTree { nodes };

        fn expected_for(
            tree: &TestTree,
            batch: &layout::CompletedLayoutBatch<usize>,
            node: usize,
        ) -> Expectation {
            let fragments = batch
                .final_inline_fragments()
                .iter()
                .filter(|entry| entry.node() == node)
                .map(|entry| {
                    let fragment = entry.fragment();
                    let rect = fragment.rect();
                    InlineFragmentExpectation {
                        source_segment_id: fragment.segment_id().get(),
                        line_index: fragment.line_index(),
                        visual_index: fragment.visual_index(),
                        x: rect.origin().x,
                        y: rect.origin().y,
                        width: rect.size().width,
                        height: rect.size().height,
                        baseline_x: fragment.baseline().x,
                        baseline_y: fragment.baseline().y,
                    }
                })
                .collect();
            Expectation {
                x: None,
                y: None,
                width: None,
                height: None,
                scroll_size: None,
                fragments: Some(fragments),
                range_inks: None,
                children: tree.nodes[node]
                    .children
                    .iter()
                    .copied()
                    .map(|child| expected_for(tree, batch, child))
                    .collect(),
            }
        }

        let expected = expected_for(&tree, &batch, 0);
        (tree, batch, expected)
    }

    fn fri06_c06_compare_fragments(
        tree: &TestTree,
        batch: &layout::CompletedLayoutBatch<usize>,
        expected: &Expectation,
    ) -> Result<(), Error> {
        compare_expectation(
            tree,
            0,
            expected,
            "fri06-c06-fragments",
            false,
            batch.final_inline_fragments(),
        )
    }

    #[test]
    fn fri06_c06_comparator_uses_final_source_order_and_preserves_visual_slots() {
        let (tree, batch, expected) = fri06_c06_fragment_observation();

        assert_eq!(
            batch
                .final_inline_fragments()
                .iter()
                .map(|entry| (
                    entry.node(),
                    entry.fragment().segment_id().get(),
                    entry.fragment().visual_index(),
                ))
                .collect::<Vec<_>>(),
            [(1, 11, 0), (3, 22, 2)]
        );
        assert_ne!(
            batch.unrounded_inline_fragments()[0].fragment().rect(),
            batch.final_inline_fragments()[0].fragment().rect()
        );
        fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect("final source-associated fragments should match");
    }

    #[test]
    fn fri06_c06_comparator_fragment_fields_have_stable_diagnostics() {
        let (tree, batch, expected) = fri06_c06_fragment_observation();
        let actual = batch.final_inline_fragments()[0].fragment();
        let rect = actual.rect();
        let baseline = actual.baseline();

        let mut cases = Vec::new();
        let mut changed = expected.clone();
        changed.children[0].fragments.as_mut().unwrap()[0].source_segment_id = 12;
        cases.push((
            changed,
            "fri06-c06-fragments/0: fragment[0] source segment id mismatch, expected 12, got 11"
                .to_string(),
        ));
        let mut changed = expected.clone();
        changed.children[0].fragments.as_mut().unwrap()[0].line_index += 1;
        cases.push((
            changed,
            format!(
                "fri06-c06-fragments/0: fragment[0] line index mismatch, expected {}, got {}",
                actual.line_index() + 1,
                actual.line_index()
            ),
        ));
        let mut changed = expected.clone();
        changed.children[0].fragments.as_mut().unwrap()[0].visual_index += 1;
        cases.push((
            changed,
            format!(
                "fri06-c06-fragments/0: fragment[0] visual index mismatch, expected {}, got {}",
                actual.visual_index() + 1,
                actual.visual_index()
            ),
        ));

        for (field, actual_value) in [
            ("rect x", rect.origin().x),
            ("rect y", rect.origin().y),
            ("rect width", rect.size().width),
            ("rect height", rect.size().height),
            ("baseline x", baseline.x),
            ("baseline y", baseline.y),
        ] {
            let mut changed = expected.clone();
            let fragment = &mut changed.children[0].fragments.as_mut().unwrap()[0];
            match field {
                "rect x" => fragment.x += 1.0,
                "rect y" => fragment.y += 1.0,
                "rect width" => fragment.width += 1.0,
                "rect height" => fragment.height += 1.0,
                "baseline x" => fragment.baseline_x += 1.0,
                "baseline y" => fragment.baseline_y += 1.0,
                _ => unreachable!(),
            }
            cases.push((
                changed,
                format!(
                    "fri06-c06-fragments/0: fragment[0] {field} mismatch, expected {}, got {actual_value}",
                    actual_value + 1.0
                ),
            ));
        }

        for (changed, diagnostic) in cases {
            let error = fri06_c06_compare_fragments(&tree, &batch, &changed)
                .expect_err("wrong fragment field should fail");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    #[test]
    fn fri06_c08_range_ink_does_not_relax_explicit_model_line_block_or_baseline() {
        let (tree, batch, expected) = fri06_c06_fragment_observation();
        for (field, diagnostic) in [
            ("block", "fragment[0] rect y mismatch"),
            ("baseline", "fragment[0] baseline y mismatch"),
        ] {
            let mut changed = expected.clone();
            let fragment = &mut changed.children[0].fragments.as_mut().unwrap()[0];
            match field {
                "block" => fragment.y += 1.0,
                "baseline" => fragment.baseline_y += 1.0,
                _ => unreachable!(),
            }
            let error = fri06_c06_compare_fragments(&tree, &batch, &changed)
                .expect_err("wrong explicit model-line block geometry should fail");
            assert!(
                error.to_string().contains(diagnostic),
                "unexpected {field} diagnostic: {error}"
            );
        }
    }

    #[test]
    fn fri06_c06_comparator_fragment_numeric_tolerance_does_not_relax_identity() {
        let (tree, batch, mut expected) = fri06_c06_fragment_observation();
        let fragment = &mut expected.children[0].fragments.as_mut().unwrap()[0];
        fragment.x += 0.05;
        fragment.baseline_y += 0.05;
        fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect("fragment geometry and baseline should use browser tolerance");

        expected.children[0].fragments.as_mut().unwrap()[0].visual_index += 1;
        let error = fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect_err("visual identity remains exact");
        assert!(
            error
                .to_string()
                .contains("fragment[0] visual index mismatch")
        );
    }

    #[test]
    fn fri06_c06_comparator_explicit_empty_checks_while_absent_skips_fragments() {
        let (tree, batch, mut expected) = fri06_c06_fragment_observation();
        expected.children[0].fragments = None;
        fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect("absent legacy fragment expectation should preserve current meaning");

        expected.children[0].fragments = Some(Vec::new());
        let error = fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect_err("explicit empty fragment state should compare exactly");
        assert_eq!(
            error.to_string(),
            "fri06-c06-fragments/0: fragment count mismatch, expected 0, got 1"
        );
    }

    #[test]
    fn fri06_c06_comparator_missing_fragment_output_is_named() {
        let (_tree, _batch, expected) = fri06_c06_fragment_observation();
        let expected_fragments = expected.children[0].fragments.as_ref().unwrap();
        let error = compare_fragment_expectations("fri06-c06-fragments/0", &[], expected_fragments)
            .expect_err("missing final fragment output should fail");
        assert_eq!(
            error.to_string(),
            "fri06-c06-fragments/0: fragment count mismatch, expected 1, got 0"
        );
    }

    #[test]
    fn fri06_c06_comparator_old_xml_without_fragments_keeps_legacy_semantics() {
        let golden = Golden::parse(
            r#"
            <test name="fri06-c06-old-xml" use-rounding="true">
                <viewport width="100px" height="80px" />
                <input><div width="10px" height="20px" /></input>
                <expectations><node x="0" y="0" width="10" height="20" /></expectations>
            </test>
            "#,
        )
        .expect("legacy XML should parse");

        assert_surgeist_matches(&golden).expect("legacy XML meaning should remain unchanged");
    }

    #[test]
    fn layout_input_returns_browser_parity_line_break_node() {
        let input = layout::LineBreakInput::new().hidden();
        let tree = line_break_tree(input);

        assert_eq!(tree.layout_input(0), layout::LayoutInput::LineBreak(input));
        assert_eq!(tree.layout_input(0).as_line_break(), Some(input));
    }

    #[test]
    fn node_input_returns_canonical_non_box_for_browser_parity_line_break_node() {
        let tree = line_break_tree(layout::LineBreakInput::new());

        assert_eq!(tree.node_input(0), &layout::NodeInput::non_box());
    }

    #[test]
    fn parse_available_accepts_css_pixel_viewport_values() {
        assert_eq!(
            parse_available("400px").expect("px viewport should parse"),
            Available::Definite(400.0)
        );
        assert_eq!(
            parse_available("max-content").expect("keyword viewport should parse"),
            Available::MaxContent
        );
    }

    #[test]
    fn browser_parity_xml_numbers_use_default_layout_scalar() {
        fn require_default_scalar(value: layout::DefaultScalar) -> layout::DefaultScalar {
            value
        }

        let parsed: Scalar =
            parse_number("137.203125").expect("fixture number should parse as default scalar");
        // Exact default-scalar value for 137.203125, written without a
        // decimal literal that trips clippy's excessive-precision lint.
        let expected = layout::DefaultScalar::from_bits(0x4309_3400);

        assert_eq!(require_default_scalar(parsed), expected);
    }

    #[test]
    fn item_order_parser_is_canonical_and_scalar_independent() {
        fn parse_fixture_order(raw: Option<&str>) -> Result<layout::ItemOrder, Error> {
            let order = raw
                .map(|value| format!(r#" order="{value}""#))
                .unwrap_or_default();
            let golden = Golden::parse(&format!(
                r#"
                <test name="item-order" use-rounding="true">
                    <viewport width="100px" height="100px" />
                    <input>
                        <div{order} />
                    </input>
                    <expectations>
                        <node x="0" y="0" width="0" height="0" />
                    </expectations>
                </test>
                "#
            ))?;
            let tree = TestTree::from_golden(&golden.root)?;
            Ok(tree.box_node_input(0).item_order)
        }

        assert_eq!(
            parse_fixture_order(None).expect("missing order should default"),
            layout::ItemOrder::ZERO
        );
        for (raw, expected) in [
            ("-2147483648", i32::MIN),
            ("0", 0),
            ("2147483647", i32::MAX),
        ] {
            assert_eq!(
                parse_fixture_order(Some(raw)).expect("canonical i32 order should parse"),
                layout::ItemOrder::new(expected),
                "{raw}"
            );
        }

        for raw in [
            "+1",
            "01",
            "-0",
            "1.0",
            "1e0",
            "order",
            " 1",
            "1 ",
            "2147483648",
            "-2147483649",
        ] {
            assert!(
                parse_fixture_order(Some(raw)).is_err(),
                "noncanonical order `{raw}` should fail"
            );
        }
    }

    #[test]
    fn viewport_parent_axes_schema_is_strict() -> Result<(), Error> {
        fn parse_viewport(attrs: &str, root_attrs: &str) -> Result<RootContext, Error> {
            Golden::parse(&format!(
                r#"
                <test name="viewport-parent-axes" use-rounding="true">
                    <viewport width="100px" height="100px" {attrs} />
                    <input>
                        <div {root_attrs} />
                    </input>
                    <expectations>
                        <node x="0" y="0" width="0" height="0" />
                    </expectations>
                </test>
                "#
            ))
            .map(|golden| golden.viewport.root_context)
        }

        assert_eq!(parse_viewport("", "")?, RootContext::Root);
        for attrs in [
            r#"host-inline-size="100px""#,
            r#"parent-writing-mode="horizontal-tb""#,
            r#"parent-direction="ltr""#,
            r#"parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="100px""#,
        ] {
            assert!(
                parse_viewport(attrs, "").is_err(),
                "root viewport metadata `{attrs}` should be rejected"
            );
        }

        let vertical_rtl = parse_viewport(
            r#"root-context="flex-item" parent-writing-mode="vertical-rl" parent-direction="rtl" host-inline-size="37.5px""#,
            r#"writing-mode="horizontal-tb" direction="ltr""#,
        )?;
        let horizontal_ltr = parse_viewport(
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="0px""#,
            r#"writing-mode="vertical-lr" direction="rtl""#,
        )?;
        assert_ne!(
            vertical_rtl, horizontal_ltr,
            "fixture root context should retain the parsed parent axes"
        );

        for attrs in [
            r#"root-context="flex-item""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb""#,
            r#"root-context="flex-item" parent-direction="ltr""#,
            r#"root-context="flex-item" host-inline-size="100px""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" host-inline-size="100px""#,
            r#"root-context="flex-item" parent-direction="ltr" host-inline-size="100px""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal" parent-direction="ltr""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="left-to-right""#,
            r#"root-context="flex-item" parent-writing-mode=" horizontal-tb" parent-direction="ltr""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr ""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="100""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="max-content""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="NaNpx""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="infpx""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="-0.5px""#,
        ] {
            assert!(
                parse_viewport(attrs, r#"writing-mode="vertical-lr" direction="rtl""#).is_err(),
                "invalid flex-item viewport metadata `{attrs}` should be rejected"
            );
        }

        Ok::<(), Error>(())
    }

    #[test]
    fn flex_item_root_separates_host_inline_allocation_from_viewport_context() -> Result<(), Error>
    {
        fn request(attrs: &str) -> Result<layout::LayoutRootRequest, Error> {
            let golden = Golden::parse(&format!(
                r#"
                <test name="flex-host-allocation" use-rounding="true">
                    <viewport width="400px" height="60px" {attrs} />
                    <input><div display="grid" /></input>
                    <expectations><node x="0" y="0" width="0" height="0" /></expectations>
                </test>
                "#
            ))?;
            let viewport_available = layout::Size::new(
                to_layout_available(golden.viewport.width),
                to_layout_available(golden.viewport.height),
            );
            root_request(viewport_available, golden.viewport.root_context)
        }

        for (attrs, expected_available) in [
            (
                r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="160px""#,
                layout::Size::new(
                    layout::Available::Definite(160.0),
                    layout::Available::MaxContent,
                ),
            ),
            (
                r#"root-context="flex-item" parent-writing-mode="vertical-rl" parent-direction="rtl" host-inline-size="80.5px""#,
                layout::Size::new(
                    layout::Available::MaxContent,
                    layout::Available::Definite(80.5),
                ),
            ),
        ] {
            let request = request(attrs)?;
            assert_eq!(request.available(), expected_available);
            let layout::LayoutRootContext::FlexItemUnderViewport(context) = request.context()
            else {
                panic!("expected flex-item root context");
            };
            assert_eq!(
                context.viewport_available(),
                layout::Size::new(
                    layout::Available::Definite(400.0),
                    layout::Available::Definite(60.0),
                )
            );
        }

        Ok(())
    }

    #[test]
    fn parses_viewport_root_context_metadata() {
        let golden = Golden::parse(
            r#"
            <test name="viewport-flex-item" use-rounding="true">
                <viewport width="400px" height="max-content" root-context="flex-item" parent-writing-mode="vertical-rl" parent-direction="rtl" host-inline-size="20px" />
                <input>
                    <div display="grid" />
                </input>
                <expectations>
                    <node x="0" y="0" width="0" height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("viewport root context should parse");

        assert_eq!(
            golden.viewport.root_context,
            RootContext::FlexItem {
                parent_axes: layout::FlowAxes::new(
                    layout::WritingMode::VerticalRl,
                    layout::Direction::Rtl,
                ),
                host_inline_size: 20.0,
            }
        );
    }

    #[test]
    fn flex_item_root_uses_the_public_compute_request() {
        let golden = Golden::parse(
            r#"
            <test name="viewport-flex-item" use-rounding="true">
                <viewport width="400px" height="80px" root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="400px" />
                <input>
                    <div display="flex" width="50%" height="20px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="200" height="20" />
                </expectations>
            </test>
            "#,
        )
        .expect("flex-item root fixture should parse");

        assert_eq!(
            golden.viewport.root_context,
            RootContext::FlexItem {
                parent_axes: layout::FlowAxes::new(
                    layout::WritingMode::HorizontalTb,
                    layout::Direction::Ltr,
                ),
                host_inline_size: 400.0,
            }
        );

        assert_surgeist_matches(&golden)
            .expect("flex-item root should be computed through the public request");
    }

    #[test]
    fn completed_batch_application_stages_outputs_and_cache_changes() {
        let golden = Golden::parse(
            r#"
            <test name="batch-application" use-rounding="false">
                <viewport width="100px" height="80px" />
                <input>
                    <div width="10.25px" height="20.5px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="10.25" height="20.5" />
                </expectations>
            </test>
            "#,
        )
        .expect("batch fixture should parse");
        let mut tree = TestTree::from_golden(&golden.root).expect("test tree should build");
        let available = layout::Size::new(
            to_layout_available(golden.viewport.width),
            to_layout_available(golden.viewport.height),
        );
        let request = root_request(available, golden.viewport.root_context)
            .expect("root request should be valid");
        let batch = layout::compute_layout(&tree, 0, request)
            .expect("layout should produce a completed batch");

        assert_eq!(batch.unrounded_entries().len(), 1);
        assert_eq!(batch.final_entries().len(), 1);
        assert_eq!(batch.cache_store_entries().len(), 1);
        tree.apply_completed_batch(&batch);
        assert_eq!(tree.nodes[0].unrounded.size, layout::Size::new(10.25, 20.5));
        assert_eq!(
            tree.nodes[0].final_layout.size,
            layout::Size::new(10.0, 21.0)
        );
        assert!(!tree.nodes[0].cache.is_empty());

        let cached_batch = layout::compute_layout(&tree, 0, request)
            .expect("identical layout should produce a completed batch");

        assert_eq!(cached_batch.unrounded_entries().len(), 1);
        assert_eq!(cached_batch.final_entries().len(), 1);
        assert!(cached_batch.cache_store_entries().is_empty());
        assert_surgeist_matches(&golden).expect("unrounded expectations should remain observable");

        let hidden_input = layout::NodeInput {
            display: layout::Display::None,
            ..layout::NodeInput::default()
        };
        tree.nodes[0].node_input = hidden_input.clone();
        tree.nodes[0].layout_input = layout::LayoutInput::box_input(hidden_input);
        let hidden_request = root_request(available, RootContext::Root)
            .expect("hidden root request should be valid");
        let hidden_batch = layout::compute_layout(&tree, 0, hidden_request)
            .expect("hidden root should produce a completed batch");

        assert_eq!(hidden_batch.cache_clear_entries().len(), 1);
        tree.apply_completed_batch(&hidden_batch);
        assert!(tree.nodes[0].cache.is_empty());
    }

    #[test]
    fn parses_partial_geometry_expectations() {
        let golden = Golden::parse(
            r#"
            <test name="partial-geometry" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div width="10px" height="20px" />
                </input>
                <expectations>
                    <node height="20" />
                </expectations>
            </test>
            "#,
        )
        .expect("partial geometry should parse");

        assert_eq!(golden.expectations.height, Some(20.0));
        assert_eq!(golden.expectations.width, None);
        assert_eq!(golden.expectations.x, None);
        assert_eq!(golden.expectations.y, None);
    }

    #[test]
    fn parse_text_align_accepts_chromium_webkit_aliases() {
        assert_eq!(
            parse_text_align("-webkit-left").expect("webkit left should parse"),
            layout::TextAlign::LegacyLeft
        );
        assert_eq!(
            parse_text_align("-webkit-right").expect("webkit right should parse"),
            layout::TextAlign::LegacyRight
        );
        assert_eq!(
            parse_text_align("-webkit-center").expect("webkit center should parse"),
            layout::TextAlign::LegacyCenter
        );
    }

    #[test]
    fn property_field_migration_preferred_parser_accepts_browser_fixture_unitless_lengths() {
        assert_eq!(
            parse_preferred_size("40").expect("unitless fixture length should parse"),
            preferred_size_px(40.0)
        );
        assert_eq!(
            parse_preferred_size("0").expect("unitless zero fixture length should parse"),
            preferred_size_px(0.0)
        );
    }

    #[test]
    fn parse_length_rejects_non_fixture_css_units() {
        assert!(parse_length("1em").is_err());
        assert!(parse_length("calc(100% - 1px)").is_err());
    }

    #[test]
    fn parse_length_rejects_non_finite_fixture_numbers_without_panicking() {
        let px_error = parse_length("NaNpx").expect_err("NaN px should be rejected");
        assert!(
            px_error.to_string().contains("scalar must be finite"),
            "unexpected error: {px_error}"
        );

        let percent_error = parse_length("inf%").expect_err("infinite percent should be rejected");
        assert!(
            percent_error.to_string().contains("scalar must be finite"),
            "unexpected error: {percent_error}"
        );

        let calc_error =
            parse_length_with_calc("calc(infpx + 10%)").expect_err("infinite calc should fail");
        assert!(
            calc_error.to_string().contains("scalar must be finite"),
            "unexpected error: {calc_error}"
        );
    }

    #[test]
    fn parse_length_accepts_fixture_calc_px_plus_percent() {
        let length = parse_length_with_calc("calc(12px + 25%)").expect("fixture calc should parse");
        let layout::Length::Value(value) = length else {
            panic!("expected affine calc length, got {length:?}");
        };
        assert_eq!(value.absolute_px(), 12.0);
        assert_eq!(value.percent_fraction(), 0.25);
        assert_eq!(length.resolve_optional(Some(200.0)), Some(62.0));
    }

    #[test]
    fn property_field_migration_preferred_parser_accepts_fixture_calc_percent_minus_px() {
        let preferred = parse_preferred_size("calc(50% - 8px)")
            .expect("fixture calc preferred size should parse");
        let expected = layout::PreferredSize::value(
            layout::LengthPercentageOf::from_coefficients(-8.0, 0.5)
                .expect("finite expected fixture calculation"),
        );
        assert_eq!(preferred, expected);
    }

    #[test]
    fn property_field_migration_parser_routes_simple_values_by_destination_property() {
        let value = layout::LengthPercentageOf::px(12.0).expect("finite fixture value");
        assert_eq!(
            parse_preferred_size("12px").expect("preferred size should parse"),
            layout::PreferredSize::value(value),
        );
        assert_eq!(
            parse_min_size("12px").expect("minimum size should parse"),
            layout::MinSize::value(value),
        );
        assert_eq!(
            parse_max_size("none").expect("maximum initial value should parse"),
            layout::MaxSize::NONE,
        );
        assert_eq!(
            parse_flex_basis("12px").expect("flex basis should parse"),
            layout::FlexBasis::value(value),
        );
        assert!(parse_max_size("auto").is_err());
        assert!(parse_preferred_size("none").is_err());
        assert!(parse_min_size("1fr").is_err());
        assert!(parse_flex_basis("1fr").is_err());
    }

    #[test]
    fn fri04_c05_parser_accepts_property_keywords_nested_calculations_and_fit_content() {
        for (raw, expected) in [
            ("auto", layout::PreferredSize::AUTO),
            ("min-content", layout::PreferredSize::MIN_CONTENT),
            ("max-content", layout::PreferredSize::MAX_CONTENT),
            ("stretch", layout::PreferredSize::STRETCH),
            ("fit-content", layout::PreferredSize::FIT_CONTENT),
            ("contain", layout::PreferredSize::CONTAIN),
        ] {
            assert_eq!(
                parse_preferred_size(raw).expect("preferred keyword should parse"),
                expected,
                "preferred keyword {raw}"
            );
        }
        for (raw, expected) in [
            ("auto", layout::MinSize::AUTO),
            ("min-content", layout::MinSize::MIN_CONTENT),
            ("max-content", layout::MinSize::MAX_CONTENT),
            ("stretch", layout::MinSize::STRETCH),
            ("fit-content", layout::MinSize::FIT_CONTENT),
            ("contain", layout::MinSize::CONTAIN),
        ] {
            assert_eq!(
                parse_min_size(raw).expect("minimum keyword should parse"),
                expected,
                "minimum keyword {raw}"
            );
        }
        for (raw, expected) in [
            ("none", layout::MaxSize::NONE),
            ("min-content", layout::MaxSize::MIN_CONTENT),
            ("max-content", layout::MaxSize::MAX_CONTENT),
            ("stretch", layout::MaxSize::STRETCH),
            ("fit-content", layout::MaxSize::FIT_CONTENT),
            ("contain", layout::MaxSize::CONTAIN),
        ] {
            assert_eq!(
                parse_max_size(raw).expect("maximum keyword should parse"),
                expected,
                "maximum keyword {raw}"
            );
        }
        for (raw, expected) in [
            ("auto", layout::FlexBasis::AUTO),
            ("content", layout::FlexBasis::CONTENT),
            ("min-content", layout::FlexBasis::MIN_CONTENT),
            ("max-content", layout::FlexBasis::MAX_CONTENT),
            ("stretch", layout::FlexBasis::STRETCH),
            ("fit-content", layout::FlexBasis::FIT_CONTENT),
            ("contain", layout::FlexBasis::CONTAIN),
        ] {
            assert_eq!(
                parse_flex_basis(raw).expect("flex keyword should parse"),
                expected,
                "flex keyword {raw}"
            );
        }

        let px = |value| {
            layout::SizingCalculation::value(
                layout::LengthPercentageOf::px(value).expect("finite expected px"),
            )
        };
        let percent = |value| {
            layout::SizingCalculation::value(
                layout::LengthPercentageOf::from_percent_fraction(value)
                    .expect("finite expected percentage"),
            )
        };
        let affine = layout::SizingCalculation::value(
            layout::LengthPercentageOf::from_coefficients(5.0, 0.1)
                .expect("finite expected affine value"),
        );
        let nested_min = layout::SizingCalculation::min(vec![percent(0.25), affine])
            .expect("nonempty expected minimum");
        let nested_max = layout::SizingCalculation::max(vec![px(10.0), nested_min])
            .expect("nonempty expected maximum");
        let expected = layout::PreferredSize::calculation(layout::SizingCalculation::clamp(
            None,
            nested_max,
            Some(px(90.0)),
        ));
        assert_eq!(
            parse_preferred_size("clamp(none, max(10px, min(25%, calc(5px + 10%))), 90px)")
                .expect("nested preferred calculation should parse"),
            expected
        );
        assert_eq!(
            parse_min_size("clamp(none, 20px, none)")
                .expect("omitted clamp endpoints should parse"),
            layout::MinSize::calculation(layout::SizingCalculation::clamp(None, px(20.0), None))
        );
        assert!(
            parse_flex_basis("fit-content(max(10px, 25%))")
                .expect("flex fit-content function should parse")
                .is_fit_content_function()
        );
        assert!(matches!(
            parse_max_track_sizing_with_calc("fit-content(min(40px, 50%))")
                .expect("maximum track fit-content should parse"),
            layout::MaxTrackSizing::FitContent(_)
        ));
    }

    #[test]
    fn fri04_c05_parser_accepts_canonical_calc_size_bases_and_affine_programs() {
        let size = layout::CalcSizeCalculation::size();
        for (raw, basis) in [
            ("100%", layout::PreferredSizeCalcBasis::FullPercentage),
            ("auto", layout::PreferredSizeCalcBasis::Auto),
            ("min-content", layout::PreferredSizeCalcBasis::MinContent),
            ("max-content", layout::PreferredSizeCalcBasis::MaxContent),
            ("stretch", layout::PreferredSizeCalcBasis::Stretch),
            ("fit-content", layout::PreferredSizeCalcBasis::FitContent),
            ("contain", layout::PreferredSizeCalcBasis::Contain),
        ] {
            assert_eq!(
                parse_preferred_size(&format!("calc-size({raw}, size)"))
                    .expect("preferred calc-size basis should parse"),
                layout::PreferredSize::calc_size(basis, size.clone())
                    .expect("expected preferred calc-size should construct")
            );
        }
        for (raw, basis) in [
            ("100%", layout::MinSizeCalcBasis::FullPercentage),
            ("auto", layout::MinSizeCalcBasis::Auto),
            ("min-content", layout::MinSizeCalcBasis::MinContent),
            ("max-content", layout::MinSizeCalcBasis::MaxContent),
            ("stretch", layout::MinSizeCalcBasis::Stretch),
            ("fit-content", layout::MinSizeCalcBasis::FitContent),
            ("contain", layout::MinSizeCalcBasis::Contain),
        ] {
            assert_eq!(
                parse_min_size(&format!("calc-size({raw}, size)"))
                    .expect("minimum calc-size basis should parse"),
                layout::MinSize::calc_size(basis, size.clone())
                    .expect("expected minimum calc-size should construct")
            );
        }
        for (raw, basis) in [
            ("100%", layout::MaxSizeCalcBasis::FullPercentage),
            ("none", layout::MaxSizeCalcBasis::None),
            ("min-content", layout::MaxSizeCalcBasis::MinContent),
            ("max-content", layout::MaxSizeCalcBasis::MaxContent),
            ("stretch", layout::MaxSizeCalcBasis::Stretch),
            ("fit-content", layout::MaxSizeCalcBasis::FitContent),
            ("contain", layout::MaxSizeCalcBasis::Contain),
        ] {
            assert_eq!(
                parse_max_size(&format!("calc-size({raw}, size)"))
                    .expect("maximum calc-size basis should parse"),
                layout::MaxSize::calc_size(basis, size.clone())
                    .expect("expected maximum calc-size should construct")
            );
        }
        for (raw, basis) in [
            ("100%", layout::FlexBasisCalcBasis::FullPercentage),
            ("auto", layout::FlexBasisCalcBasis::Auto),
            ("content", layout::FlexBasisCalcBasis::Content),
            ("min-content", layout::FlexBasisCalcBasis::MinContent),
            ("max-content", layout::FlexBasisCalcBasis::MaxContent),
            ("stretch", layout::FlexBasisCalcBasis::Stretch),
            ("fit-content", layout::FlexBasisCalcBasis::FitContent),
            ("contain", layout::FlexBasisCalcBasis::Contain),
        ] {
            assert_eq!(
                parse_flex_basis(&format!("calc-size({raw}, size)"))
                    .expect("flex calc-size basis should parse"),
                layout::FlexBasis::calc_size(basis, size.clone())
                    .expect("expected flex calc-size should construct")
            );
        }

        let independent = layout::CalcSizeCalculation::from_coefficients(10.0, 0.25, 0.0)
            .expect("finite independent calc-size coefficients");
        assert_eq!(
            parse_preferred_size("calc-size(any, 10px + 25%)")
                .expect("independent Any calc-size should parse"),
            layout::PreferredSize::calc_size(
                layout::PreferredSizeCalcBasis::Any,
                independent.clone()
            )
            .expect("independent Any calc-size should construct")
        );
        assert_eq!(
            parse_min_size("calc-size(any, 10px + 25%)")
                .expect("minimum Any calc-size should parse"),
            layout::MinSize::calc_size(layout::MinSizeCalcBasis::Any, independent.clone())
                .expect("minimum Any calc-size should construct")
        );
        assert_eq!(
            parse_max_size("calc-size(any, 10px + 25%)")
                .expect("maximum Any calc-size should parse"),
            layout::MaxSize::calc_size(layout::MaxSizeCalcBasis::Any, independent.clone())
                .expect("maximum Any calc-size should construct")
        );
        assert_eq!(
            parse_flex_basis("calc-size(any, 10px + 25%)")
                .expect("flex Any calc-size should parse"),
            layout::FlexBasis::calc_size(layout::FlexBasisCalcBasis::Any, independent)
                .expect("flex Any calc-size should construct")
        );

        let nested_min = layout::CalcSizeCalculation::min(vec![
            layout::CalcSizeCalculation::from_coefficients(0.0, 0.0, 0.5)
                .expect("finite size coefficient"),
            layout::CalcSizeCalculation::from_coefficients(0.0, 0.8, 0.0)
                .expect("finite percentage coefficient"),
        ])
        .expect("nonempty calc-size minimum");
        let nested_max = layout::CalcSizeCalculation::max(vec![
            layout::CalcSizeCalculation::from_coefficients(10.0, 0.25, 0.0)
                .expect("finite affine coefficient"),
            nested_min,
        ])
        .expect("nonempty calc-size maximum");
        let nested = layout::CalcSizeCalculation::clamp(
            None,
            nested_max,
            Some(
                layout::CalcSizeCalculation::from_coefficients(100.0, 0.0, 0.0)
                    .expect("finite maximum coefficient"),
            ),
        );
        assert_eq!(
            parse_preferred_size(
                "calc-size(auto, clamp(none, max(10px + 25%, min(size * 0.5, 80%)), 100px))"
            )
            .expect("nested calc-size program should parse"),
            layout::PreferredSize::calc_size(layout::PreferredSizeCalcBasis::Auto, nested)
                .expect("nested calc-size program should construct")
        );
    }

    #[test]
    fn fri04_c05_parser_accepts_track_only_flex_and_rejects_flex_in_minimum_track() {
        let factor = layout::TrackFlexFactor::try_new(2.0).expect("finite track flex");
        assert_eq!(
            parse_max_track_sizing_with_calc("2fr").expect("maximum track flex should parse"),
            layout::MaxTrackSizing::flex(factor)
        );
        assert_eq!(
            parse_track_sizing_with_calc("2fr").expect("complete track flex should parse"),
            layout::TrackSizing::flex(factor)
        );
        assert!(parse_min_track_sizing_with_calc("2fr").is_err());
        assert!(parse_track_sizing_with_calc("minmax(2fr, 20px)").is_err());
        assert!(parse_preferred_size("2fr").is_err());
        assert!(parse_min_size("2fr").is_err());
        assert!(parse_max_size("2fr").is_err());
        assert!(parse_flex_basis("2fr").is_err());

        assert!(parse_min_track_sizing_with_calc("min(10px, max(20%, 30px))").is_ok());
        assert!(parse_max_track_sizing_with_calc("max(10px, min(20%, 30px))").is_ok());
        assert!(
            parse_track_sizing_with_calc("minmax(min(10px, 20%), fit-content(max(30px, 40%)))")
                .is_ok()
        );
    }

    #[test]
    fn fri04_c05_parser_rejects_malformed_arity_nonfinite_and_cross_property_values() {
        for raw in [
            "min()",
            "max()",
            "min(10px,)",
            "clamp(10px, 20px)",
            "clamp(10px, 20px, 30px, 40px)",
            "fit-content()",
            "fit-content(10px, 20px)",
            "calc-size(any)",
            "calc-size(any, 10px, 20px)",
            "min(10px, max(20px, 30px)",
            "min(10px)), 20px)",
            "min(10px) trailing",
            "10px + 20%",
            "min(10px + 20%, 30px)",
            "NaNpx",
            "inf%",
            "calc(1e38px + 3e38px)",
            "size",
            "calc(size + 1px)",
            "min(size, 1px)",
            "calc-size(any, size)",
            "calc-size(any, max(1px, size))",
        ] {
            assert!(
                parse_preferred_size(raw).is_err(),
                "invalid preferred fixture unexpectedly parsed: {raw}"
            );
        }

        for raw in ["none", "content", "1fr"] {
            assert!(parse_preferred_size(raw).is_err(), "preferred {raw}");
            assert!(parse_min_size(raw).is_err(), "minimum {raw}");
        }
        for raw in ["auto", "content", "1fr"] {
            assert!(parse_max_size(raw).is_err(), "maximum {raw}");
        }
        for raw in ["none", "1fr"] {
            assert!(parse_flex_basis(raw).is_err(), "flex basis {raw}");
        }
        assert!(parse_min_track_sizing_with_calc("content").is_err());
        assert!(parse_min_track_sizing_with_calc("fit-content(10px)").is_err());
        assert!(parse_max_track_sizing_with_calc("content").is_err());
        assert!(parse_max_track_sizing_with_calc("none").is_err());
        assert!(parse_max_track_sizing_with_calc("-1fr").is_err());
        assert!(parse_max_track_sizing_with_calc("NaNfr").is_err());
        assert!(parse_max_track_sizing_with_calc("inffr").is_err());
    }

    #[test]
    fn fri04_c05_parser_accepts_depth_64_and_rejects_depth_65_before_descent() {
        let nested_min = |depth: usize| {
            let mut raw = "min(".repeat(depth);
            raw.push_str("10px");
            raw.push_str(&")".repeat(depth));
            raw
        };

        assert!(
            parse_preferred_size(&nested_min(64)).is_ok(),
            "fixture nesting at the documented limit should parse"
        );
        let error = parse_preferred_size(&nested_min(65))
            .expect_err("fixture nesting beyond the documented limit should fail");
        assert!(
            error.to_string().contains("exceeds 64"),
            "unexpected excessive-depth error: {error}"
        );
    }

    #[test]
    fn fri04_c05_parser_preserves_existing_unitless_fixture_lengths() {
        assert_eq!(
            parse_preferred_size("40").expect("unitless preferred length should parse"),
            preferred_size_px(40.0)
        );
        assert_eq!(
            parse_min_size("0").expect("unitless minimum length should parse"),
            layout::MinSize::ZERO
        );
        assert_eq!(
            parse_max_track_sizing_with_calc("12")
                .expect("unitless maximum track length should parse"),
            max_track_px(12.0)
        );
    }

    #[test]
    fn parse_length_rejects_unsupported_calc_fixture_syntax() {
        let error =
            parse_length_with_calc("calc(100% / 2)").expect_err("division is not supported yet");
        assert!(
            error.to_string().contains("unsupported calc expression"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn to_node_input_lowers_calc_margin_as_affine_value() {
        let golden = Golden::parse(
            r#"
            <test name="calc-margin" use-rounding="true">
                <viewport width="200px" height="max-content" />
                <input>
                    <div display="block" margin-left="calc(10% - 4px)" />
                </input>
                <expectations>
                    <node x="0" y="0" width="0" height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("calc margin XML should parse");

        let tree = TestTree::from_golden(&golden.root).expect("calc margin should lower");

        let layout::LengthAuto::Value(value) = tree.box_node_input(0).margin.left else {
            panic!(
                "expected affine calc margin-left, got {:?}",
                tree.box_node_input(0).margin.left
            );
        };
        assert_eq!(value.absolute_px(), -4.0);
        assert_eq!(value.percent_fraction(), 0.1);
        assert_eq!(
            tree.box_node_input(0)
                .margin
                .left
                .resolve_optional(Some(200.0)),
            Some(16.0)
        );
    }

    #[test]
    fn to_node_input_lowers_calc_grid_track_as_affine_value() {
        let golden = Golden::parse(
            r#"
            <test name="calc-grid-track" use-rounding="true">
                <viewport width="240px" height="max-content" />
                <input>
                    <div display="grid" grid-template-columns="calc(25% + 20px)" />
                </input>
                <expectations>
                    <node x="0" y="0" width="0" height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("calc grid track XML should parse");

        let tree = TestTree::from_golden(&golden.root).expect("calc grid track should lower");

        let [layout::TrackComponent::Track(track)] =
            tree.box_node_input(0).grid_template_columns.as_slice()
        else {
            panic!(
                "expected one grid track, got {:?}",
                tree.box_node_input(0).grid_template_columns
            );
        };
        let layout::MinTrackSizing::Calculation(min) = &track.min else {
            panic!("expected affine calc min track, got {:?}", track.min);
        };
        let layout::MaxTrackSizing::Calculation(max) = &track.max else {
            panic!("expected affine calc max track, got {:?}", track.max);
        };
        assert_eq!(min, max);
        assert_eq!(
            min.resolve_against(layout::PercentageBasisOf::definite(240.0).unwrap())
                .value,
            Some(80.0)
        );
    }

    #[test]
    fn parse_track_component_list_accepts_rich_grid_tracks() {
        let tracks = parse_track_component_list(
            "40px minmax(20px,40px) fit-content(50%) repeat(2, 1fr auto)",
        )
        .expect("rich grid track list should parse");

        assert_eq!(tracks.len(), 4);
        assert_eq!(tracks[0], track_component_px(40.0));
        assert_eq!(
            tracks[1],
            layout::TrackComponent::minmax(min_track_px(20.0), max_track_px(40.0))
        );
        assert_eq!(
            tracks[2],
            layout::TrackComponent::fit_content(layout::SizingCalculation::value(
                layout::LengthPercentageOf::from_percent_fraction(0.5).unwrap()
            ))
        );
        assert_eq!(
            tracks[3],
            layout::TrackComponent::Repeat(
                layout::TrackRepetition::count(
                    2,
                    vec![
                        layout::TrackSizing::flex(layout::TrackFlexFactor::try_new(1.0).unwrap()),
                        layout::TrackSizing::AUTO
                    ]
                )
                .expect("valid track repetition")
            )
        );
    }

    #[test]
    fn track_sizing_parser_accepts_role_valid_values_and_rejects_invalid_flex() {
        assert!(parse_track_sizing_with_calc("calc(20px + 25%)").is_ok());
        assert!(parse_track_sizing_with_calc("fit-content(50%)").is_ok());
        assert!(parse_track_sizing_with_calc("1fr").is_ok());
        assert!(parse_track_sizing_with_calc("minmax(10px,2fr)").is_ok());

        assert!(parse_track_sizing_with_calc("minmax(1fr,20px)").is_err());
        assert!(parse_track_sizing_with_calc("-1fr").is_err());
        assert!(parse_track_sizing_with_calc("NaNfr").is_err());
        assert!(parse_track_sizing_with_calc("inffr").is_err());
    }

    #[test]
    fn parse_track_component_list_accepts_auto_repeat() {
        assert_eq!(
            parse_track_component("repeat(auto-fill, minmax(150px,1fr))")
                .expect("auto-fill should parse"),
            layout::TrackComponent::Repeat(
                layout::TrackRepetition::auto_fill(vec![layout::TrackSizing::minmax(
                    min_track_px(150.0),
                    layout::MaxTrackSizing::Flex(layout::TrackFlexFactor::try_new(1.0).unwrap())
                )])
                .expect("valid track repetition")
            )
        );
        assert_eq!(
            parse_track_component("repeat(auto-fit, 40px)").expect("auto-fit should parse"),
            layout::TrackComponent::Repeat(
                layout::TrackRepetition::auto_fit(vec![track_px(40.0)])
                    .expect("valid track repetition")
            )
        );
    }

    #[test]
    fn parse_track_component_list_accepts_explicit_line_names() {
        let parsed =
            parse_track_component_list("[a] 10px [b c] 20px [d]").expect("line names should parse");

        assert_eq!(
            parsed,
            vec![
                layout::TrackComponent::LineNames(vec!["a".to_string()]),
                layout::TrackComponent::Track(track_px(10.0)),
                layout::TrackComponent::LineNames(vec!["b".to_string(), "c".to_string()]),
                layout::TrackComponent::Track(track_px(20.0)),
                layout::TrackComponent::LineNames(vec!["d".to_string()]),
            ]
        );
    }

    #[test]
    fn parse_track_component_list_rejects_reserved_line_names() {
        assert!(parse_track_component_list("[auto] 10px").is_err());
        assert!(parse_track_component_list("[span] 10px").is_err());
    }

    #[test]
    fn to_node_input_preserves_named_grid_syntax() {
        let input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([
                (
                    "grid-template-columns".to_string(),
                    "[a] 10px [b]".to_string(),
                ),
                ("grid-column-start".to_string(), "a 2".to_string()),
            ]),
        })
        .expect("named grid syntax should parse to layout input");

        assert_eq!(
            input.raw_grid_column,
            layout::RawGridPlacement::new(
                layout::RawGridLine::NamedLine {
                    name: "a".to_string(),
                    index: 2,
                },
                layout::RawGridLine::Auto,
            )
        );
        assert_eq!(
            input.grid_template_columns,
            vec![
                layout::TrackComponent::LineNames(vec!["a".to_string()]),
                layout::TrackComponent::Track(track_px(10.0)),
                layout::TrackComponent::LineNames(vec!["b".to_string()]),
            ]
        );
    }

    #[test]
    fn to_node_input_leaves_untagged_default_display_to_layout_default() {
        let input = test_node_input(StyleAttrs {
            attrs: BTreeMap::new(),
        })
        .expect("empty attrs should parse");

        assert_eq!(input.display, layout::NodeInput::default().display);
    }

    #[test]
    fn to_node_input_applies_html_source_tag_display_defaults() {
        let div = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("source-tag".to_string(), "div".to_string())]),
        })
        .expect("source-tag div should parse");

        assert_eq!(div.display, layout::Display::Block);
    }

    #[test]
    fn source_tag_br_lowers_to_line_break_input() {
        let input = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("direction".to_string(), "rtl".to_string()),
                ("writing-mode".to_string(), "vertical-rl".to_string()),
                ("vertical-align".to_string(), "top".to_string()),
                ("clear".to_string(), "both".to_string()),
            ]),
        })
        .expect("source-tag br should lower");

        let layout::LayoutInput::LineBreak(input) = input else {
            panic!("br should lower to line break");
        };
        assert_eq!(input.direction(), layout::Direction::Rtl);
        assert_eq!(input.writing_mode(), layout::WritingMode::VerticalRl);
        assert_eq!(input.vertical_align(), layout::VerticalAlign::Top);
        assert_eq!(input.clear(), layout::Clear::Both);
    }

    #[test]
    fn source_tag_br_uses_containing_inline_flow_when_descendant_style_differs() {
        let golden = Golden::parse(
            r#"
            <test name="br-containing-flow" use-rounding="true">
                <viewport width="100px" height="100px" />
                <input>
                    <div direction="rtl" writing-mode="vertical-rl">
                        <div source-tag="br" direction="ltr" writing-mode="horizontal-tb" />
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
        .expect("fixture should parse");

        let tree = TestTree::from_golden(&golden.root).expect("test tree should build");
        let layout::LayoutInput::LineBreak(input) = tree.nodes[1].layout_input else {
            panic!("br should lower to line break");
        };

        assert_eq!(input.direction(), layout::Direction::Rtl);
        assert_eq!(input.writing_mode(), layout::WritingMode::VerticalRl);
        assert_eq!(golden.root.children[0].style.get("direction"), Some("ltr"));
        assert_eq!(
            golden.root.children[0].style.get("writing-mode"),
            Some("horizontal-tb")
        );
    }

    #[test]
    fn source_tag_br_display_inline_lowers_to_visible_line_break() {
        let input = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "inline".to_string()),
                ("direction".to_string(), "rtl".to_string()),
            ]),
        })
        .expect("display inline br should lower");

        let layout::LayoutInput::LineBreak(input) = input else {
            panic!("br should lower to line break");
        };
        assert_eq!(input.display(), layout::LineBreakDisplay::Break);
        assert_eq!(input.direction(), layout::Direction::Rtl);
    }

    #[test]
    fn source_tag_br_display_none_lowers_to_hidden_line_break() {
        let input = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "none".to_string()),
            ]),
        })
        .expect("display none br should lower");

        let layout::LayoutInput::LineBreak(input) = input else {
            panic!("br should lower to line break");
        };
        assert_eq!(input.display(), layout::LineBreakDisplay::None);
    }

    #[test]
    fn source_tag_br_lowers_explicit_inline_metrics() {
        let input = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("inline-baseline".to_string(), "15px".to_string()),
                ("inline-line-height".to_string(), "20px".to_string()),
            ]),
        })
        .expect("source-tag br with inline metrics should lower");

        let layout::LayoutInput::LineBreak(input) = input else {
            panic!("br should lower to line break");
        };
        assert_eq!(input.metrics().baseline(), 15.0);
        assert_eq!(input.metrics().line_extent(), 20.0);
        assert_eq!(input.metrics().after_baseline(), 5.0);
    }

    #[test]
    fn sideways_writing_mode_parses_without_normalization() {
        for (raw, expected) in [
            ("horizontal-tb", layout::WritingMode::HorizontalTb),
            ("vertical-rl", layout::WritingMode::VerticalRl),
            ("vertical-lr", layout::WritingMode::VerticalLr),
            ("sideways-rl", layout::WritingMode::SidewaysRl),
            ("sideways-lr", layout::WritingMode::SidewaysLr),
        ] {
            assert_eq!(
                parse_writing_mode(Some(raw)).expect("known writing mode should parse"),
                expected,
                "{raw} must preserve its exact layout writing mode"
            );
        }

        assert!(parse_writing_mode(Some("sideways")).is_err());
    }

    #[test]
    fn source_tag_br_rejects_partial_inline_metrics() {
        let error = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("inline-baseline".to_string(), "15px".to_string()),
            ]),
        })
        .expect_err("partial inline metrics should be rejected");

        assert!(
            error.to_string().contains("inline metrics require"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn generation_report_uses_explicit_br_unsupported_buckets() {
        let report = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/layout/browser_parity/xml/generation-reports/all.json");
        let raw = std::fs::read_to_string(&report)
            .unwrap_or_else(|error| panic!("{} should read: {error}", report.display()));
        let report_json: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|error| panic!("{} should parse as JSON: {error}", report.display()));
        let unsupported = report_json["unsupported"]
            .as_array()
            .expect("unsupported report entries should be an array");
        let reasons = unsupported
            .iter()
            .map(|entry| {
                entry
                    .get("reason")
                    .or_else(|| entry.get("kind"))
                    .or_else(|| entry.get("error"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown")
            })
            .collect::<Vec<_>>();

        assert!(
            !reasons.contains(&"Unsupported <br> line-break semantics"),
            "BR fixtures must not remain in the stale generic unsupported bucket"
        );
        assert!(
            reasons.contains(&"Unsupported vertical <br> line-break semantics"),
            "vertical <br> fixtures should remain explicitly unsupported"
        );
        assert!(
            reasons.contains(&"Unsupported <br> outside block inline-run semantics"),
            "outside-block <br> fixtures should remain explicitly unsupported"
        );
        let unsupported_sources = unsupported
            .iter()
            .filter_map(|entry| entry.get("source").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>();
        for source in [
            "html/block/block_br_vertical_rl_inline_block_metrics.html",
            "html/block/block_br_vertical_lr_inline_block_metrics.html",
            "html/block/block_br_vertical_rl_empty_lines_metrics.html",
            "html/block/block_br_vertical_rl_rtl_inline_block_metrics.html",
        ] {
            assert!(
                !unsupported_sources.contains(&source),
                "{source} should generate rather than remain unsupported"
            );
        }
    }

    #[test]
    fn to_node_input_preserves_grid_template_areas() {
        let input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([(
                "grid-template-areas".to_string(),
                "head head / nav main".to_string(),
            )]),
        })
        .expect("grid template areas should parse to layout input");

        assert_eq!(
            input.grid_template_areas,
            layout::GridTemplateAreas {
                rows: vec![
                    layout::GridTemplateAreaRow {
                        cells: vec![Some("head".to_string()), Some("head".to_string())],
                    },
                    layout::GridTemplateAreaRow {
                        cells: vec![Some("nav".to_string()), Some("main".to_string())],
                    },
                ],
            }
        );
    }

    #[test]
    fn to_node_input_treats_grid_template_area_dot_runs_as_null_cells() {
        let input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([(
                "grid-template-areas".to_string(),
                "... main / footer ...".to_string(),
            )]),
        })
        .expect("grid template areas should parse to layout input");

        assert_eq!(
            input.grid_template_areas,
            layout::GridTemplateAreas {
                rows: vec![
                    layout::GridTemplateAreaRow {
                        cells: vec![None, Some("main".to_string())],
                    },
                    layout::GridTemplateAreaRow {
                        cells: vec![Some("footer".to_string()), None],
                    },
                ],
            }
        );
    }

    #[test]
    fn parse_track_component_list_accepts_subgrid() {
        assert_eq!(
            parse_track_component_list("subgrid [] [start end]").expect("subgrid should parse"),
            vec![layout::TrackComponent::Subgrid(layout::SubgridTrack {
                name_components: vec![
                    layout::SubgridLineNameComponent::LineNames(Vec::new()),
                    layout::SubgridLineNameComponent::LineNames(vec![
                        "start".to_string(),
                        "end".to_string(),
                    ]),
                ],
            })]
        );
    }

    #[test]
    fn parse_display_preserves_inline_grid_variants() {
        assert_eq!(
            parse_display("inline-grid").expect("inline-grid should parse"),
            layout::Display::InlineGrid
        );
        assert_eq!(
            parse_display("inline-grid-lanes").expect("inline-grid-lanes should parse"),
            layout::Display::InlineGridLanes
        );
    }

    #[test]
    fn parse_display_preserves_inline_block() {
        assert_eq!(
            parse_display("inline-block").expect("inline-block should parse"),
            layout::Display::InlineBlock
        );
    }

    #[test]
    fn empty_inline_grid_display_uses_grid_tracks_instead_of_leaf_measurement() {
        let node_input = layout::NodeInput {
            display: layout::Display::InlineGrid,
            grid_template_columns: vec![track_component_px(40.0)],
            grid_template_rows: vec![track_component_px(20.0)],
            ..layout::NodeInput::default()
        };
        let mut tree = TestTree {
            nodes: vec![TestNode {
                node_input: node_input.clone(),
                layout_input: layout::LayoutInput::box_input(node_input),
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: TextMeasure::LINE_HEIGHT,
                text: None,
                children: Vec::new(),
                synthetic: false,
                preserve_fractional_min_content: false,
                use_tighter_monospace_wrap: false,
                cache: layout::Cache::new(),
                unrounded: layout::NodeOutput::new(),
                unrounded_present: false,
                final_layout: layout::NodeOutput::new(),
                final_layout_present: false,
                unrounded_inline_fragments: None,
                final_inline_fragments: None,
                shape_bands: None,
            }],
        };

        let request = root_request(
            layout::Size::splat(layout::Available::MaxContent),
            RootContext::Root,
        )
        .expect("root request should be valid");
        let batch =
            layout::compute_layout(&tree, 0, request).expect("inline grid layout should complete");
        tree.apply_completed_batch(&batch);

        assert_eq!(
            tree.nodes[0].final_layout.size,
            layout::Size::new(40.0, 20.0)
        );
    }

    #[test]
    fn empty_inline_grid_xml_uses_grid_tracks_instead_of_leaf_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="empty-inline-grid" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-grid" grid-template-columns="40px" grid-template-rows="20px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="40" height="20" />
                </expectations>
            </test>
            "#,
        )
        .expect("inline-grid parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("empty inline-grid should size from its grid tracks");
    }

    #[test]
    fn grid_text_container_uses_anonymous_text_child_for_layout() {
        let golden = Golden::parse(
            r#"
            <test name="grid-text-container" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-grid" grid-template-columns="50px">
                        <div display="grid">hello</div>
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="50" height="10">
                        <node x="0" y="0" width="50" height="10" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("grid text fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("grid container text should contribute through anonymous text child");
    }

    #[test]
    fn text_inline_grid_xml_uses_text_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="text-inline-grid" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <text display="inline-grid" font-size="15px">X</text>
                </input>
                <expectations>
                    <node x="0" y="0" width="15" height="15" />
                </expectations>
            </test>
            "#,
        )
        .expect("inline-grid text parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("text inline-grid should use fixture text measurement");
    }

    #[test]
    fn font_size_attr_scales_fixture_text_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="font-size-text" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <text font-size="12px">x</text>
                </input>
                <expectations>
                    <node x="0" y="0" width="12" height="12" />
                </expectations>
            </test>
            "#,
        )
        .expect("font-size parity fixture should parse");

        assert_surgeist_matches(&golden).expect("font-size should scale fixture text measurement");
    }

    #[test]
    fn monospace_text_measurement_wraps_at_spaces() {
        let text = TextMeasure::new(
            "The cat can not be separated from milk",
            FontFamily::Monospace,
            16.0,
            16.0,
            false,
            true,
        );

        assert_eq!(text.height_for_width(80.0), 96.0);
    }

    #[test]
    fn monospace_text_measurement_matches_chromium_subgrid_wrap_width() {
        let text = TextMeasure::new(
            "The cat can not be separated from milk",
            FontFamily::Monospace,
            16.0,
            16.0,
            false,
            true,
        );

        assert_eq!(text.height_for_width(86.0), 80.0);
    }

    #[test]
    fn monospace_min_content_keeps_browser_fractional_width() {
        let text = TextMeasure::new("Number 1", FontFamily::Monospace, 15.0, 15.0, true, true);

        assert_eq!(text.min_content_width(), 54.1875);
    }

    #[test]
    fn font_size_attr_inherits_to_fixture_text_descendants() {
        let golden = Golden::parse(
            r#"
            <test name="inherited-font-size-text" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div font-size="12px">
                        <text>x</text>
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="12" height="12">
                        <node x="0" y="0" width="12" height="12" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("inherited font-size parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("font-size should inherit to fixture text descendants");
    }

    #[test]
    fn line_height_attr_controls_fixture_text_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="line-height-text" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-block" line-height="0px">
                        x
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="10" height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("line-height parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("line-height should affect fixture text measurement");
    }

    #[test]
    fn font_family_attr_uses_monospace_fixture_text_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="monospace-font-family-text" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div font-family="monospace" font-size="24px">
                        <text>x</text>
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="14" height="24">
                        <node x="0" y="0" width="14" height="24" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("monospace font-family parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("font-family should select fixture text measurement");
    }

    #[test]
    fn orthogonal_nested_subgrid_text_leaf_preserves_lowered_physical_size() {
        let golden = Golden::parse(
            r#"
            <test name="orthogonal-nested-subgrid-text-sized" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-grid" box-sizing="content-box" direction="ltr" font-family="monospace" font-size="24px" line-height="24px" row-gap="20px" column-gap="20px" border-top="3px" border-left="3px" border-bottom="3px" border-right="3px" grid-template-rows="100px auto" grid-template-columns="100px auto">
                        <div display="grid" box-sizing="content-box" direction="ltr" writing-mode="vertical-rl" font-family="monospace" font-size="24px" line-height="24px" row-gap="100px" column-gap="100px" grid-template-rows="subgrid" grid-template-columns="subgrid" grid-row-start="span 2" grid-column-start="span 2">
                            <div display="grid" box-sizing="content-box" direction="ltr" writing-mode="horizontal-tb" font-family="monospace" font-size="24px" line-height="24px" row-gap="100px" column-gap="100px" grid-template-rows="subgrid" grid-template-columns="100px" grid-column-start="span 2">
                                <text display="block" box-sizing="content-box" direction="ltr" font-family="monospace" font-size="24px" line-height="24px" width="24px" height="24px">a</text>
                                <text display="block" box-sizing="content-box" direction="ltr" font-family="monospace" font-size="24px" line-height="24px" width="24px" height="24px" grid-row-start="2">b</text>
                            </div>
                            <text display="block" box-sizing="content-box" direction="ltr" writing-mode="vertical-rl" font-family="monospace" font-size="24px" line-height="24px" width="72px" height="24px" grid-row-start="1" grid-column-start="2">ccc</text>
                        </div>
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="238" height="190">
                        <node>
                            <node>
                                <node />
                                <node />
                            </node>
                            <node width="72" height="24" />
                        </node>
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("orthogonal nested subgrid text fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("text leaf should preserve its lowered physical size and writing mode");
    }

    #[test]
    fn empty_inline_grid_lanes_fixture_uses_grid_lanes_tracks_instead_of_leaf_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="empty-inline-grid-lanes" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-grid-lanes" grid-template-columns="40px" grid-template-rows="20px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="40" height="20" />
                </expectations>
            </test>
            "#,
        )
        .expect("inline-grid-lanes parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("empty inline-grid-lanes should size from its grid-lanes tracks");
    }

    #[test]
    fn parse_alignment_accepts_safe_and_unsafe_prefixes() {
        assert_eq!(
            parse_align_items("safe center").expect("safe item alignment should parse"),
            layout::AlignItems::SafeCenter
        );
        assert_eq!(
            parse_align_items("baseline").expect("baseline alignment should parse"),
            layout::AlignItems::Baseline
        );
        assert_eq!(
            parse_align_items("first baseline").expect("first baseline alignment should parse"),
            layout::AlignItems::Baseline
        );
        assert_eq!(
            parse_align_items("last baseline").expect("last baseline alignment should parse"),
            layout::AlignItems::LastBaseline
        );
        assert_eq!(
            parse_align_content("unsafe end").expect("unsafe content alignment should parse"),
            layout::AlignContent::End
        );
    }

    #[test]
    fn parse_alignment_rejects_prefixed_baselines() {
        assert!(parse_align_items("safe baseline").is_err());
        assert!(parse_align_items("unsafe first baseline").is_err());
        assert!(parse_align_items("safe last baseline").is_err());
    }

    #[test]
    fn to_node_input_lowers_baseline_and_subgrid_parity_attrs() {
        let node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([
                ("display".to_string(), "inline-grid".to_string()),
                ("align-items".to_string(), "last baseline".to_string()),
                ("align-self".to_string(), "first baseline".to_string()),
                (
                    "grid-template-columns".to_string(),
                    "subgrid [a] [b]".to_string(),
                ),
            ]),
        })
        .expect("baseline subgrid attrs should parse");

        assert_eq!(node_input.display, layout::Display::InlineGrid);
        assert_eq!(
            node_input.align_items,
            Some(layout::AlignItems::LastBaseline)
        );
        assert_eq!(node_input.align_self, Some(layout::AlignItems::Baseline));
        assert_eq!(
            node_input.grid_template_columns,
            vec![layout::TrackComponent::Subgrid(layout::SubgridTrack {
                name_components: vec![
                    layout::SubgridLineNameComponent::LineNames(vec!["a".to_string()]),
                    layout::SubgridLineNameComponent::LineNames(vec!["b".to_string()]),
                ],
            })]
        );
    }

    #[test]
    fn to_node_input_preserves_end_only_grid_placement_for_abspos() {
        let node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("grid-column-end".to_string(), "1".to_string())]),
        })
        .expect("end-only grid placement should parse");

        assert_eq!(
            node_input.grid_column,
            layout::GridPlacement::try_end_line(1).expect("valid grid line")
        );
        assert_eq!(
            node_input.raw_grid_column,
            layout::RawGridPlacement::new(layout::RawGridLine::Auto, layout::RawGridLine::Line(1))
        );
    }

    #[test]
    fn to_node_input_preserves_physical_auto_inline_margins() {
        let node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([
                ("margin-left".to_string(), "auto".to_string()),
                ("margin-right".to_string(), "12px".to_string()),
            ]),
        })
        .expect("physical margin attrs should parse");

        assert_eq!(node_input.margin.left, layout::LengthAuto::AUTO);
        assert_eq!(
            node_input.margin.right,
            length_px(12.0, "12px").unwrap().into()
        );
    }

    #[test]
    fn fixture_gaps_project_logical_css_axes() {
        let expected_physical_gap = layout::Size::new(
            length_px(7.0, "7px").expect("valid row gap"),
            length_px(11.0, "11px").expect("valid column gap"),
        );

        for (writing_mode, direction) in [
            ("vertical-rl", "ltr"),
            ("vertical-lr", "rtl"),
            ("sideways-rl", "rtl"),
            ("sideways-lr", "ltr"),
        ] {
            let node_input = test_node_input(StyleAttrs {
                attrs: BTreeMap::from([
                    ("writing-mode".to_string(), writing_mode.to_string()),
                    ("direction".to_string(), direction.to_string()),
                    ("column-gap".to_string(), "11px".to_string()),
                    ("row-gap".to_string(), "7px".to_string()),
                ]),
            })
            .expect("logical CSS gaps should parse");

            assert_eq!(
                node_input.gap, expected_physical_gap,
                "{writing_mode} {direction} must store column-gap as inline and row-gap as block"
            );
        }
    }

    #[test]
    fn parse_grid_line_accepts_negative_lines() {
        assert_eq!(
            parse_grid_line("-1").expect("negative line should parse"),
            Some(-1)
        );
        let mut span = None;
        assert_eq!(
            parse_grid_line_or_span("-3", &mut span).expect("negative end line should parse"),
            Some(-3)
        );
        assert_eq!(span, None);
    }

    #[test]
    fn parse_grid_line_accepts_named_line() {
        assert_eq!(
            parse_raw_grid_line("a").unwrap(),
            layout::RawGridLine::BareIdent("a".to_string())
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_line_with_occurrence() {
        assert_eq!(
            parse_raw_grid_line("a 8").unwrap(),
            layout::RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 8
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_integer_before_named_line() {
        assert_eq!(
            parse_raw_grid_line("2 a").unwrap(),
            layout::RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 2
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_negative_named_line_occurrence() {
        assert_eq!(
            parse_raw_grid_line("b -1").unwrap(),
            layout::RawGridLine::NamedLine {
                name: "b".to_string(),
                index: -1
            }
        );
    }

    #[test]
    fn parse_grid_line_rejects_zero_numeric_line() {
        assert!(parse_raw_grid_line("0").is_err());
    }

    #[test]
    fn parse_grid_line_accepts_named_span() {
        assert_eq!(
            parse_raw_grid_line("span a").unwrap(),
            layout::RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 1
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_span_with_count() {
        assert_eq!(
            parse_raw_grid_line("span 2 a").unwrap(),
            layout::RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_span_with_reversed_count_order() {
        assert_eq!(
            parse_raw_grid_line("span a 2").unwrap(),
            layout::RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2
            }
        );
    }

    #[test]
    fn parse_grid_line_rejects_zero_named_line_occurrence() {
        assert!(parse_raw_grid_line("a 0").is_err());
    }

    #[test]
    fn parse_grid_line_rejects_zero_named_span_count() {
        assert!(parse_raw_grid_line("span 0 a").is_err());
        assert!(parse_raw_grid_line("span a 0").is_err());
    }

    #[test]
    fn parse_grid_line_rejects_reserved_named_custom_ident() {
        assert!(parse_raw_grid_line("auto 1").is_err());
        assert!(parse_raw_grid_line("1 auto").is_err());
        assert!(parse_raw_grid_line("span auto").is_err());
        assert!(parse_raw_grid_line("span 1 auto").is_err());
    }

    #[test]
    fn comparison_tolerance_is_named_policy() {
        let tolerance = ComparisonTolerance::browser_parity();

        assert!(tolerance.contains(0.05));
        assert!(!tolerance.contains(0.2));
    }

    #[test]
    fn parse_grid_start_line_accepts_span() {
        let mut node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("grid-column-start".to_string(), "span 2".to_string())]),
        })
        .expect("span start should parse");

        assert_eq!(node_input.grid_column.start(), None);
        assert_eq!(node_input.grid_column.span(), GridSpan::new(2));

        node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("grid-row-start".to_string(), "span 3".to_string())]),
        })
        .expect("row span start should parse");

        assert_eq!(node_input.grid_row.start(), None);
        assert_eq!(node_input.grid_row.span(), GridSpan::new(3));
    }

    fn fri06_c06_inline_input_xml(input: &str) -> String {
        format!(
            r#"
            <test name="fri06-c06-inline-input" use-rounding="true">
                <viewport width="100" height="max-content" />
                <input>{input}</input>
                <expectations><node /></expectations>
            </test>
            "#
        )
    }

    fn fri06_c06_segment_xml(
        id: &str,
        extent: &str,
        metrics: (&str, &str),
        bidi: &str,
        whitespace: &str,
        following_break: &str,
        replacement: Option<&str>,
    ) -> String {
        let (baseline, line_height) = metrics;
        let replacement = replacement
            .map(|value| format!(r#" replacement-inline-extent="{value}""#))
            .unwrap_or_default();
        format!(
            r#"<segment id="{id}" inline-extent="{extent}" inline-baseline="{baseline}" inline-line-height="{line_height}" bidi-level="{bidi}" whitespace-edge="{whitespace}" following-break="{following_break}"{replacement} />"#
        )
    }

    fn fri06_c06_inline_text(segments: &str) -> String {
        format!(r#"<text layout-input="inline-text">{segments}</text>"#)
    }

    fn fri06_c06_lower(input: &str) -> Result<TestTree, Error> {
        let golden = Golden::parse(&fri06_c06_inline_input_xml(input))?;
        TestTree::from_golden(&golden.root)
    }

    fn fri06_c06_valid_segment() -> String {
        fri06_c06_segment_xml(
            "11",
            "10.25",
            ("8", "10"),
            "1",
            "preserve",
            "allowed-with-replacement",
            Some("1.5"),
        )
    }

    #[test]
    fn fri06_c06_inline_input_valid_shaped_text_uses_production_model_and_non_box_pairing() {
        let segments = [
            fri06_c06_segment_xml(
                "11",
                "10.25",
                ("8", "10"),
                "0",
                "preserve",
                "prohibited",
                None,
            ),
            fri06_c06_segment_xml(
                "22",
                "5",
                ("7", "9"),
                "1",
                "discard-at-line-start",
                "allowed",
                None,
            ),
            fri06_c06_segment_xml(
                "33",
                "4",
                ("6", "8"),
                "2",
                "discard-at-line-end",
                "mandatory",
                None,
            ),
            fri06_c06_segment_xml(
                "44",
                "3",
                ("5", "7"),
                "3",
                "preserve",
                "allowed-with-replacement",
                Some("1.5"),
            ),
        ]
        .join("");
        let tree = fri06_c06_lower(&format!(
            r#"<div display="block">{}</div>"#,
            fri06_c06_inline_text(&segments)
        ))
        .expect("reviewed shaped input should lower");

        let layout::LayoutInput::InlineText(input) = tree.layout_input(1) else {
            panic!("valid shaped text must lower as LayoutInput::InlineText");
        };
        assert_eq!(tree.node_input(1), &layout::NodeInput::non_box());
        assert!(tree.nodes[1].children.is_empty());
        assert!(!tree.has_leaf_measurement(1));
        assert_eq!(tree.nodes[1].text, None);
        assert!(!tree.nodes[1].synthetic);

        let segments = input.segments();
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].segment_id().get(), 11);
        assert_eq!(segments[0].inline_extent(), 10.25);
        assert_eq!(segments[0].metrics().baseline(), 8.0);
        assert_eq!(segments[0].metrics().line_extent(), 10.0);
        assert_eq!(segments[1].bidi_level().get(), 1);
        assert_eq!(
            segments[1].whitespace_edge(),
            layout::InlineWhitespaceEdge::DiscardAtLineStart
        );
        assert_eq!(
            segments[2].following_break().kind(),
            layout::InlineBreakKind::Mandatory
        );
        assert_eq!(
            segments[3].following_break().replacement_inline_extent(),
            Some(1.5)
        );
    }

    #[test]
    fn fri06_c06_inline_input_segment_validation_rejects_empty_and_duplicate_ids() {
        let empty = fri06_c06_valid_segment().replace("id=\"11\"", "id=\"\"");
        let error = fri06_c06_lower(&fri06_c06_inline_text(&empty))
            .expect_err("an empty segment ID must be rejected");
        assert!(error.to_string().contains("invalid `id` on `<segment>`"));

        let duplicate = format!("{}{}", fri06_c06_valid_segment(), fri06_c06_valid_segment());
        let error = fri06_c06_lower(&fri06_c06_inline_text(&duplicate))
            .expect_err("duplicate caller-local segment IDs must be rejected");
        assert!(error.to_string().contains("duplicate segment id `11`"));

        let error = fri06_c06_lower(&fri06_c06_inline_text(""))
            .expect_err("an empty shaped text input must be rejected");
        assert!(
            error
                .to_string()
                .contains("inline text requires at least one `<segment>`")
        );
    }

    #[test]
    fn fri06_c06_inline_input_segment_validation_rejects_partial_and_nonfinite_tuples() {
        for missing in [
            "id",
            "inline-extent",
            "inline-baseline",
            "inline-line-height",
            "bidi-level",
            "whitespace-edge",
            "following-break",
        ] {
            let needle = format!(r#" {missing}=""#);
            let segment = fri06_c06_valid_segment();
            let start = segment.find(&needle).expect("field exists");
            let value_start = start + needle.len();
            let value_end = value_start
                + segment[value_start..]
                    .find('"')
                    .expect("field value terminates");
            let mut partial = segment;
            partial.replace_range(start..=value_end, "");
            let error = fri06_c06_lower(&fri06_c06_inline_text(&partial))
                .expect_err("a partial shaped tuple must be rejected");
            assert_eq!(
                error.to_string(),
                format!("missing `{missing}` on `<segment>`")
            );
        }

        for (field, invalid) in [
            ("inline-extent", "NaN"),
            ("inline-baseline", "inf"),
            ("inline-line-height", "NaN"),
            ("replacement-inline-extent", "-inf"),
        ] {
            let mut segment = fri06_c06_valid_segment();
            let field_prefix = format!(r#"{field}=""#);
            let start = segment.find(&field_prefix).expect("field exists") + field_prefix.len();
            let end = start + segment[start..].find('"').expect("field value terminates");
            segment.replace_range(start..end, invalid);
            let error = fri06_c06_lower(&fri06_c06_inline_text(&segment))
                .expect_err("non-finite shaped metrics must be rejected");
            assert!(error.to_string().contains(&format!("invalid `{field}`")));
        }
    }

    #[test]
    fn fri06_c06_inline_input_segment_validation_rejects_out_of_domain_choices() {
        for (field, valid, invalid) in [
            ("bidi-level", "1", "126"),
            ("whitespace-edge", "preserve", "collapse"),
            ("following-break", "allowed-with-replacement", "fallback"),
        ] {
            let segment = fri06_c06_valid_segment().replace(
                &format!(r#"{field}="{valid}""#),
                &format!(r#"{field}="{invalid}""#),
            );
            let error = fri06_c06_lower(&fri06_c06_inline_text(&segment))
                .expect_err("out-of-domain inline fact must be rejected");
            assert!(error.to_string().contains(&format!("invalid `{field}`")));
        }

        for segment in [
            fri06_c06_valid_segment().replace(" replacement-inline-extent=\"1.5\"", ""),
            fri06_c06_valid_segment().replace(
                "following-break=\"allowed-with-replacement\"",
                "following-break=\"allowed\"",
            ),
            fri06_c06_valid_segment().replace(
                "whitespace-edge=\"preserve\"",
                "whitespace-edge=\"discard-at-both\"",
            ),
        ] {
            let error = fri06_c06_lower(&fri06_c06_inline_text(&segment))
                .expect_err("contradictory break tuple must be rejected");
            assert!(error.to_string().contains("replacement"));
        }
    }

    #[test]
    fn fri06_c06_inline_input_schema_validation_rejects_unknown_attributes_and_payload() {
        for (shaped_text, diagnostic) in [
            (
                fri06_c06_inline_text(
                    &fri06_c06_valid_segment().replace(" />", " glyphs=\"x\" />"),
                ),
                "unsupported `<segment>` attribute `glyphs`",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text" font-family="Ahem">{}</text>"#,
                    fri06_c06_valid_segment()
                ),
                "unsupported inline text attribute `font-family`",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text">{}authored text</text>"#,
                    fri06_c06_valid_segment()
                ),
                "unsupported non-whitespace text in inline text",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text">{}<unknown /></text>"#,
                    fri06_c06_valid_segment()
                ),
                "unsupported inline text child `<unknown>`",
            ),
            (
                r#"<text layout-input="inline-text"><segment id="11" inline-extent="1" inline-baseline="1" inline-line-height="1" bidi-level="0" whitespace-edge="preserve" following-break="prohibited">payload</segment></text>"#.to_string(),
                "unsupported non-whitespace text in `<segment>`",
            ),
        ] {
            let error = fri06_c06_lower(&shaped_text)
                .expect_err("unknown shaped attributes or payload must fail closed");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    #[test]
    fn fri06_c06_inline_input_schema_validation_rejects_box_children_and_measurement() {
        for (shaped_text, diagnostic) in [
            (
                format!(
                    r#"<text layout-input="inline-text" display="block">{}</text>"#,
                    fri06_c06_valid_segment()
                ),
                "inline text must not specify box attribute `display`",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text">{}<div /></text>"#,
                    fri06_c06_valid_segment()
                ),
                "inline text must not contain layout child `<div>`",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text">{}measured leaf text</text>"#,
                    fri06_c06_valid_segment()
                ),
                "unsupported non-whitespace text in inline text",
            ),
        ] {
            let error = fri06_c06_lower(&shaped_text)
                .expect_err("inline text must reject contradictory box or measurement state");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    fn fri06_c06_atomic_fixture(atomic_facts: &str) -> String {
        format!(
            r#"
            <div display="block">
                <div display="inline-block" width="4" height="6" />
                {}
                {atomic_facts}
            </div>
            "#,
            fri06_c06_inline_text(&fri06_c06_segment_xml(
                "22",
                "5",
                ("4", "6"),
                "0",
                "preserve",
                "prohibited",
                None,
            ))
        )
    }

    #[test]
    fn fri06_c06_inline_input_atomic_binding_uses_exact_child_index_and_preserves_order() {
        let tree = fri06_c06_lower(&fri06_c06_atomic_fixture(
            r#"<atomic-placeholder child-index="0" bidi-level="3" following-break="mandatory" />"#,
        ))
        .expect("reviewed atomic placeholder should bind");

        assert_eq!(tree.nodes[0].children, vec![1, 2]);
        let participation = tree
            .node_input(1)
            .atomic_inline_participation
            .expect("the referenced atomic child receives participation");
        assert_eq!(participation.bidi_level().get(), 3);
        assert_eq!(
            participation.following_break().kind(),
            layout::InlineBreakKind::Mandatory
        );
        assert_eq!(tree.node_input(2), &layout::NodeInput::non_box());
    }

    #[test]
    fn fri06_c06_inline_input_atomic_binding_rejects_unmatched_duplicate_and_invalid_facts() {
        for (atomic_facts, diagnostic) in [
            (
                r#"<atomic-placeholder child-index="2" bidi-level="0" following-break="allowed" />"#,
                "unmatched atomic child index `2`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="allowed" /><atomic-placeholder child-index="0" bidi-level="1" following-break="mandatory" />"#,
                "duplicate atomic child index `0`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="126" following-break="allowed" />"#,
                "invalid `bidi-level` on `<atomic-placeholder>`: `126`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="fallback" />"#,
                "invalid `following-break` on `<atomic-placeholder>`: `fallback`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="allowed-with-replacement" replacement-inline-extent="1" />"#,
                "atomic placeholder break replacement is not allowed",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" />"#,
                "missing `following-break` on `<atomic-placeholder>`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="allowed" glyph="x" />"#,
                "unsupported `<atomic-placeholder>` attribute `glyph`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="allowed">payload</atomic-placeholder>"#,
                "unsupported non-whitespace text in `<atomic-placeholder>`",
            ),
        ] {
            let error = fri06_c06_lower(&fri06_c06_atomic_fixture(atomic_facts))
                .expect_err("invalid or unmatched atomic facts must fail closed");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    fn fri06_c06_inline_request() -> layout::LayoutRootRequest {
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::definite(100.0),
            layout::Available::MaxContent,
        ))
        .expect("valid inline fixture request")
    }

    #[test]
    fn fri06_c06_inline_input_fragment_cache_commits_and_restores_nonempty_slices() {
        let mut tree = fri06_c06_lower(&format!(
            r#"<div display="block">{}</div>"#,
            fri06_c06_inline_text(&fri06_c06_segment_xml(
                "11",
                "10.25",
                ("8", "10"),
                "0",
                "preserve",
                "prohibited",
                None,
            ))
        ))
        .expect("valid shaped fixture should lower");
        let request = fri06_c06_inline_request();
        let cold = layout::compute_layout(&tree, 0, request).expect("cold inline layout succeeds");
        assert_eq!(cold.unrounded_inline_fragments().len(), 1);
        assert_eq!(cold.final_inline_fragments().len(), 1);
        assert_ne!(
            cold.unrounded_inline_fragments()[0].fragment().rect(),
            cold.final_inline_fragments()[0].fragment().rect()
        );

        tree.apply_completed_batch(&cold);
        assert_eq!(
            tree.unrounded_inline_fragments(1),
            Some([cold.unrounded_inline_fragments()[0].fragment()].as_slice())
        );

        let warm = layout::compute_layout(&tree, 0, request).expect("warm inline layout succeeds");
        assert_eq!(
            warm.unrounded_inline_fragments(),
            cold.unrounded_inline_fragments()
        );
        assert_eq!(warm.final_inline_fragments(), cold.final_inline_fragments());
    }

    #[test]
    fn fri06_c06_inline_input_fragment_cache_distinguishes_committed_empty_from_absent() {
        let mut tree = fri06_c06_lower(&format!(
            r#"<div display="block">{}</div>"#,
            fri06_c06_inline_text(&fri06_c06_segment_xml(
                "11",
                "3",
                ("0", "0"),
                "0",
                "discard-at-both",
                "prohibited",
                None,
            ))
        ))
        .expect("valid discardable shaped fixture should lower");
        assert_eq!(tree.unrounded_inline_fragments(1), None);

        let request = fri06_c06_inline_request();
        let cold = layout::compute_layout(&tree, 0, request).expect("cold empty layout succeeds");
        assert!(cold.unrounded_inline_fragments().is_empty());
        tree.apply_completed_batch(&cold);
        assert_eq!(tree.unrounded_inline_fragments(1), Some([].as_slice()));

        let warm = layout::compute_layout(&tree, 0, request).expect("warm empty layout succeeds");
        assert!(warm.unrounded_inline_fragments().is_empty());
        assert!(warm.final_inline_fragments().is_empty());
    }

    fn fri06_c06_shape_input_xml(float_attrs: &str, provider: &str) -> String {
        format!(
            r#"
            <test name="fri06-c06-shape-input" use-rounding="true">
                <viewport width="100" height="max-content" />
                <input>
                    <div display="block" width="100">
                        <div width="80" height="20" {float_attrs}>
                            {provider}
                        </div>
                        <div display="block" width="30" height="10" float="left" />
                    </div>
                </input>
                <expectations><node /></expectations>
            </test>
            "#
        )
    }

    fn fri06_c06_shape_input_lower(float_attrs: &str, provider: &str) -> Result<TestTree, Error> {
        let golden = Golden::parse(&fri06_c06_shape_input_xml(float_attrs, provider))?;
        TestTree::from_golden(&golden.root)
    }

    fn fri06_c06_shape_input_query(
        band_minimum: Scalar,
        band_maximum: Scalar,
    ) -> layout::FloatExclusionQuery {
        layout::FloatExclusionQuery::try_new(
            layout::ScrollRect::try_new(layout::Point::ZERO, layout::Size::new(80.0, 20.0))
                .expect("finite test margin box"),
            layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr),
            band_minimum,
            band_maximum,
        )
        .expect("finite test query")
    }

    fn fri06_c06_shape_input_request() -> layout::LayoutRootRequest {
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::definite(100.0),
            layout::Available::MaxContent,
        ))
        .expect("finite shape fixture request")
    }

    #[test]
    fn fri06_c06_shape_input_bottom_alignment_lowers_exactly_without_widening() {
        let bottom = fri06_c06_shape_input_lower(r#"float="left" vertical-align="bottom""#, "")
            .expect("bottom is the reviewed finite alignment addition");
        assert_eq!(
            bottom.node_input(1).vertical_align,
            layout::VerticalAlign::Bottom
        );

        let error = fri06_c06_shape_input_lower(r#"float="left" vertical-align="middle""#, "")
            .expect_err("later-owned vertical alignment must remain rejected");
        assert_eq!(
            error.to_string(),
            "unsupported parity fixture vertical-align `middle`"
        );
    }

    #[test]
    fn fri06_c06_shape_input_valid_table_returns_empty_partial_and_full_intervals() {
        let provider = r#"
            <shape-bands>
                <shape-band band-minimum="0" band-maximum="10" />
                <shape-band band-minimum="10" band-maximum="20"
                    interval-minimum="0" interval-maximum="40" />
                <shape-band band-minimum="20" band-maximum="30"
                    interval-minimum="0" interval-maximum="80" />
            </shape-bands>
        "#;
        let tree = fri06_c06_shape_input_lower(r#"float="left" float-exclusion="shape""#, provider)
            .expect("finite shape table lowers through production constructors");
        assert_eq!(
            tree.node_input(1).float_exclusion,
            layout::FloatExclusion::Shape
        );

        let empty = tree
            .float_exclusion_interval(1, fri06_c06_shape_input_query(0.0, 10.0))
            .expect("shape table is a provider")
            .expect("empty intersection is a successful response");
        assert_eq!(empty, None);

        for (band, expected) in [((10.0, 20.0), (0.0, 40.0)), ((20.0, 30.0), (0.0, 80.0))] {
            let interval = tree
                .float_exclusion_interval(1, fri06_c06_shape_input_query(band.0, band.1))
                .expect("shape table is a provider")
                .expect("finite interval is a successful response")
                .expect("partial and full intervals remain nonempty");
            assert_eq!((interval.minimum(), interval.maximum()), expected);
        }
    }

    #[test]
    fn fri06_c06_shape_input_schema_rejects_partial_nonfinite_duplicate_and_unknown_facts() {
        let cases = [
            (
                r#"<shape-bands><shape-band band-minimum="0" /></shape-bands>"#,
                "missing `band-maximum` on `<shape-band>`",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="NaN" band-maximum="10" /></shape-bands>"#,
                "invalid shape band query",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="10" band-maximum="0" /></shape-bands>"#,
                "invalid shape band query",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" interval-minimum="2" /></shape-bands>"#,
                "shape interval endpoints must appear together",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" interval-minimum="8" interval-maximum="2" /></shape-bands>"#,
                "invalid shape exclusion interval",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" origin-band-minimum="1" interval-minimum="0" interval-maximum="5" /></shape-bands>"#,
                "originating shape band endpoints must appear together",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" /><shape-band band-minimum="0" band-maximum="10" /></shape-bands>"#,
                "duplicate shape query band `0..10`",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" path="M0 0" /></shape-bands>"#,
                "unsupported `<shape-band>` attribute `path`",
            ),
            (
                r#"<shape-bands geometry="path"><shape-band band-minimum="0" band-maximum="10" /></shape-bands>"#,
                "unsupported `<shape-bands>` attribute `geometry`",
            ),
            (
                r#"<shape-bands><path /></shape-bands>"#,
                "unsupported `<shape-bands>` child `<path>`",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" provider-result="failure" interval-minimum="0" interval-maximum="5" /></shape-bands>"#,
                "provider failure must not include an exclusion interval",
            ),
        ];

        for (provider, diagnostic) in cases {
            let error =
                fri06_c06_shape_input_lower(r#"float="left" float-exclusion="shape""#, provider)
                    .expect_err("strict finite shape schema must fail closed");
            assert!(
                error.to_string().contains(diagnostic),
                "expected `{diagnostic}`, got `{error}`"
            );
        }
    }

    #[test]
    fn fri06_c06_shape_input_table_binds_only_visible_in_flow_left_or_right_shape_float() {
        let provider =
            r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" /></shape-bands>"#;
        for attrs in [
            r#"float="none" float-exclusion="shape""#,
            r#"display="none" float="left" float-exclusion="shape""#,
            r#"position="absolute" float="right" float-exclusion="shape""#,
            r#"float="left""#,
        ] {
            let error = fri06_c06_shape_input_lower(attrs, provider)
                .expect_err("shape table must bind only to the reviewed shape-float role");
            assert_eq!(
                error.to_string(),
                "shape band table requires a visible in-flow left/right shape float"
            );
        }

        for side in ["left", "right"] {
            fri06_c06_shape_input_lower(
                &format!(r#"float="{side}" float-exclusion="shape""#),
                provider,
            )
            .expect("both visible in-flow shape-float sides accept a table");
        }
    }

    #[test]
    fn fri06_c06_shape_input_compute_consumes_partial_table_through_production_provider() {
        let tree = fri06_c06_shape_input_lower(
            r#"float="left" float-exclusion="shape""#,
            r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" interval-minimum="0" interval-maximum="40" /></shape-bands>"#,
        )
        .expect("valid partial fixture response lowers");
        let batch = layout::compute_layout(&tree, 0, fri06_c06_shape_input_request())
            .expect("production compute consumes the finite provider response");
        let second_float = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 2)
            .expect("second float publishes output")
            .output();
        assert_eq!(second_float.location, layout::Point::new(40.0, 0.0));
    }

    #[test]
    fn fri06_c06_shape_input_compute_reports_missing_mismatch_and_provider_failure() {
        let missing = fri06_c06_shape_input_lower(r#"float="left" float-exclusion="shape""#, "")
            .expect("shape without a table represents a missing fixture provider");
        let error = layout::compute_layout(&missing, 0, fri06_c06_shape_input_request())
            .expect_err("requested missing provider must fail");
        assert_eq!(
            error.site(),
            layout::LayoutErrorSite::ContainerSubject {
                container: 0,
                subject: 1,
            }
        );
        assert_eq!(
            error.operation(),
            layout::LayoutOperation::FloatExclusionQuery
        );
        assert!(matches!(
            error.kind(),
            layout::LayoutErrorKind::MissingContext(
                layout::LayoutMissingContext::FloatExclusionProvider
            )
        ));

        let mismatch = fri06_c06_shape_input_lower(
            r#"float="left" float-exclusion="shape""#,
            r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" origin-band-minimum="10" origin-band-maximum="20" interval-minimum="0" interval-maximum="40" /></shape-bands>"#,
        )
        .expect("representable mismatched origin lowers");
        let error = layout::compute_layout(&mismatch, 0, fri06_c06_shape_input_request())
            .expect_err("production compute must reject a mismatched originating query");
        let layout::LayoutErrorKind::InvalidInput(
            layout::LayoutInvalidInput::FloatExclusionProviderOutput {
                error: layout::FloatExclusionIntervalError::QueryMismatch { expected, actual },
            },
        ) = error.kind()
        else {
            panic!("unexpected mismatch diagnostic: {error:?}");
        };
        assert_eq!(
            (expected.band_minimum(), expected.band_maximum()),
            (0.0, 10.0)
        );
        assert_eq!((actual.band_minimum(), actual.band_maximum()), (10.0, 20.0));

        let failure = fri06_c06_shape_input_lower(
            r#"float="left" float-exclusion="shape""#,
            r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" provider-result="failure" /></shape-bands>"#,
        )
        .expect("representable provider failure lowers");
        let error = layout::compute_layout(&failure, 0, fri06_c06_shape_input_request())
            .expect_err("production compute must preserve fixture provider failure");
        assert_eq!(
            error.site(),
            layout::LayoutErrorSite::ContainerSubject {
                container: 0,
                subject: 1,
            }
        );
        assert_eq!(
            error.operation(),
            layout::LayoutOperation::FloatExclusionQuery
        );
        let layout::LayoutErrorKind::Measurement(provider_error) = error.kind() else {
            panic!("unexpected provider failure diagnostic: {error:?}");
        };
        assert_eq!(
            provider_error.to_string(),
            "fixture shape provider failure for query band `0..10`"
        );
    }
}
