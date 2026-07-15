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
    pub children: Vec<Expectation>,
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

    let mut attrs = BTreeMap::new();
    for attr in xml.attributes() {
        attrs.insert(attr.name().to_string(), attr.value().to_string());
    }

    let text = xml
        .text()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned);
    let children = xml
        .children()
        .filter(roxmltree::Node::is_element)
        .map(parse_node)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Node {
        kind,
        style: StyleAttrs { attrs },
        text,
        children,
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

    Ok(Expectation {
        x: optional_number_attr(xml, "x")?,
        y: optional_number_attr(xml, "y")?,
        width: optional_number_attr(xml, "width")?,
        height: optional_number_attr(xml, "height")?,
        scroll_size,
        children: xml
            .children()
            .filter(roxmltree::Node::is_element)
            .map(parse_expectation)
            .collect::<Result<Vec<_>, _>>()?,
    })
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
    final_layout: layout::NodeOutput,
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
        let layout_input = to_layout_input(&node.style)?;
        let box_display = layout_input.as_box().map(|input| input.display);
        let grid_lanes_text = inherited.grid_lanes_text
            || box_display.is_some_and(layout::Display::establishes_grid_lanes_formatting_context);
        let inline_level_text = inherited.inline_level_text
            || box_display.is_some_and(layout::Display::is_inline_level);
        self.nodes.push(TestNode {
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
            final_layout: layout::NodeOutput::new(),
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
            final_layout: layout::NodeOutput::new(),
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
        for entry in batch.unrounded_entries() {
            self.nodes[entry.node()].unrounded = entry.output();
        }
        for entry in batch.final_entries() {
            self.nodes[entry.node()].final_layout = entry.output();
        }
        for entry in batch.cache_store_entries() {
            self.nodes[entry.node()].cache.store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
        for entry in batch.cache_clear_entries() {
            self.nodes[entry.node()].cache.clear();
        }
    }

    fn box_node_input(&self, node: usize) -> &layout::NodeInput {
        self.nodes[node]
            .layout_input
            .as_box()
            .unwrap_or_else(|| panic!("line break node has no box NodeInput"))
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
    type MeasureError = std::convert::Infallible;

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

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
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
        Some(Ok(self.measure(node, input)))
    }
}

fn compare_expectation(
    tree: &TestTree,
    node: usize,
    expected: &Expectation,
    path: &str,
    use_rounding: bool,
) -> Result<(), Error> {
    // The browser reports a rect for `<br>`, while layout models it as a zero-size
    // inline control carrying flow and metrics data rather than box geometry.
    if matches!(
        tree.nodes[node].layout_input,
        layout::LayoutInput::LineBreak(_)
    ) {
        return Ok(());
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
        compare_expectation(
            tree,
            child,
            expected_child,
            &format!("{path}/{index}"),
            use_rounding,
        )?;
    }

    Ok(())
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
    let input = to_node_input(attrs)?;
    if attrs.get("source-tag") == Some("br") {
        let mut br = layout::LineBreakInput::new()
            .with_direction(input.direction)
            .with_writing_mode(input.writing_mode)
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
    let mut input = layout::NodeInput::default();
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
    if let Some(value) = attrs.get("clear") {
        input.clear = parse_clear(value)?;
    }
    if let Some(value) = attrs.get("overflow-x") {
        input.overflow.x = parse_overflow(value)?;
    }
    if let Some(value) = attrs.get("overflow-y") {
        input.overflow.y = parse_overflow(value)?;
    }
    if let Some(value) = attrs.get("scrollbar-width") {
        input.scrollbar_width = layout::ScrollbarWidth::try_new(parse_number(value)?)
            .map_err(|source| Error::new(source.to_string()))?;
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
        input.flex_basis = parse_dimension_with_calc(value)?;
    }
    if let Some(value) = attrs.get("width") {
        input.size.width = parse_dimension_with_calc(value)?;
    }
    if let Some(value) = attrs.get("height") {
        input.size.height = parse_dimension_with_calc(value)?;
    }
    if let Some(value) = attrs.get("min-width") {
        input.min_size.width = parse_dimension_with_calc(value)?;
    }
    if let Some(value) = attrs.get("min-height") {
        input.min_size.height = parse_dimension_with_calc(value)?;
    }
    if let Some(value) = attrs.get("max-width") {
        input.max_size.width = parse_dimension_with_calc(value)?;
    }
    if let Some(value) = attrs.get("max-height") {
        input.max_size.height = parse_dimension_with_calc(value)?;
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
        "scroll" | "auto" => Ok(layout::Overflow::Scroll),
        _ => Err(Error::new(format!("unsupported overflow `{raw}`"))),
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

fn parse_dimension_with_calc(raw: &str) -> Result<layout::Dimension, Error> {
    match raw {
        "auto" => Ok(layout::Dimension::AUTO),
        "min-content" => Ok(layout::Dimension::MIN_CONTENT),
        "max-content" => Ok(layout::Dimension::MAX_CONTENT),
        _ => {
            if raw.trim_start().starts_with("calc(") {
                return Ok(layout::Dimension::value(parse_calc_expression(raw)?));
            }
            parse_dimension(raw)
        }
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

fn parse_dimension(raw: &str) -> Result<layout::Dimension, Error> {
    match raw {
        "auto" => Ok(layout::Dimension::AUTO),
        "min-content" => Ok(layout::Dimension::MIN_CONTENT),
        "max-content" => Ok(layout::Dimension::MAX_CONTENT),
        _ => {
            if let Some(fr) = raw.strip_suffix("fr") {
                return Ok(layout::Dimension::fr(parse_number(fr)?));
            }
            Ok(parse_length(raw)?.into())
        }
    }
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
fn dimension_px(value: Scalar) -> layout::Dimension {
    layout::Dimension::value(
        layout::LengthPercentageOf::px(value).expect("finite test dimension px"),
    )
}

#[cfg(test)]
fn min_track_px(value: Scalar) -> layout::MinTrackSizing {
    layout::Length::value(layout::LengthPercentageOf::px(value).expect("finite test min track px"))
        .into()
}

#[cfg(test)]
fn max_track_px(value: Scalar) -> layout::MaxTrackSizing {
    layout::Length::value(layout::LengthPercentageOf::px(value).expect("finite test max track px"))
        .into()
}

#[cfg(test)]
fn track_px(value: Scalar) -> layout::TrackSizing {
    layout::Length::value(layout::LengthPercentageOf::px(value).expect("finite test track px"))
        .into()
}

#[cfg(test)]
fn track_component_px(value: Scalar) -> layout::TrackComponent {
    layout::TrackComponent::Track(track_px(value))
}

#[cfg(test)]
fn length_percent_for_test(value: Scalar) -> layout::Length {
    layout::Length::value(
        layout::LengthPercentageOf::from_percent_fraction(value).expect("finite test percent"),
    )
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
    if let Some(body) = function_body(raw, "minmax") {
        let (min, max) = split_once_top_level_comma(body)?;
        return Ok(layout::TrackSizing::minmax(
            parse_min_track_sizing_with_calc(min.trim())?,
            parse_max_track_sizing_with_calc(max.trim())?,
        ));
    }
    if let Some(body) = function_body(raw, "fit-content") {
        return Ok(layout::TrackSizing::fit_content(parse_length_with_calc(
            body.trim(),
        )?));
    }
    Ok(parse_dimension_with_calc(raw)?.into())
}

fn parse_min_track_sizing_with_calc(raw: &str) -> Result<layout::MinTrackSizing, Error> {
    match raw {
        "auto" => Ok(layout::MinTrackSizing::AUTO),
        "min-content" => Ok(layout::MinTrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::MinTrackSizing::MAX_CONTENT),
        _ => Ok(parse_length_with_calc(raw)?.into()),
    }
}

fn parse_max_track_sizing_with_calc(raw: &str) -> Result<layout::MaxTrackSizing, Error> {
    match raw {
        "auto" => Ok(layout::MaxTrackSizing::AUTO),
        "min-content" => Ok(layout::MaxTrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::MaxTrackSizing::MAX_CONTENT),
        _ if raw.ends_with("fr") => {
            let value = raw.trim_end_matches("fr");
            Ok(layout::MaxTrackSizing::fr(parse_number(value)?))
        }
        _ => Ok(parse_length_with_calc(raw)?.into()),
    }
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

    fn line_break_tree(input: layout::LineBreakInput) -> TestTree {
        TestTree {
            nodes: vec![TestNode {
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
                final_layout: layout::NodeOutput::new(),
            }],
        }
    }

    #[test]
    fn layout_input_returns_browser_parity_line_break_node() {
        let input = layout::LineBreakInput::new().hidden();
        let tree = line_break_tree(input);

        assert_eq!(tree.layout_input(0), layout::LayoutInput::LineBreak(input));
        assert_eq!(tree.layout_input(0).as_line_break(), Some(input));
    }

    #[test]
    #[should_panic(expected = "line break node has no box NodeInput")]
    fn node_input_panics_for_browser_parity_line_break_node() {
        let tree = line_break_tree(layout::LineBreakInput::new());

        let _ = tree.node_input(0);
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

        tree.nodes[0].layout_input = layout::LayoutInput::box_input(layout::NodeInput {
            display: layout::Display::None,
            ..layout::NodeInput::default()
        });
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
    fn parse_dimension_accepts_browser_fixture_unitless_lengths() {
        assert_eq!(
            parse_dimension("40").expect("unitless fixture length should parse"),
            dimension_px(40.0)
        );
        assert_eq!(
            parse_dimension("0").expect("unitless zero fixture length should parse"),
            dimension_px(0.0)
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
    fn parse_dimension_accepts_fixture_calc_percent_minus_px() {
        let dimension = parse_dimension_with_calc("calc(50% - 8px)")
            .expect("fixture calc dimension should parse");
        let layout::Dimension::Value(value) = dimension else {
            panic!("expected affine calc dimension, got {dimension:?}");
        };
        assert_eq!(value.absolute_px(), -8.0);
        assert_eq!(value.percent_fraction(), 0.5);
        assert_eq!(dimension.resolve_optional(Some(240.0)), Some(112.0));
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
        let layout::MinTrackSizing::Length(layout::Length::Value(min)) = track.min else {
            panic!("expected affine calc min track, got {:?}", track.min);
        };
        let layout::MaxTrackSizing::Length(layout::Length::Value(max)) = track.max else {
            panic!("expected affine calc max track, got {:?}", track.max);
        };
        assert_eq!(min, max);
        assert_eq!(min.absolute_px(), 20.0);
        assert_eq!(min.percent_fraction(), 0.25);
        assert_eq!(track.min.definite(Some(240.0)), Some(80.0));
        assert_eq!(track.max.definite(Some(240.0)), Some(80.0));
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
            layout::TrackComponent::fit_content(length_percent_for_test(0.5))
        );
        assert_eq!(
            tracks[3],
            layout::TrackComponent::Repeat(
                layout::TrackRepetition::count(
                    2,
                    vec![layout::TrackSizing::fr(1.0), layout::TrackSizing::AUTO]
                )
                .expect("valid track repetition")
            )
        );
    }

    #[test]
    fn parse_track_component_list_accepts_auto_repeat() {
        assert_eq!(
            parse_track_component("repeat(auto-fill, minmax(150px,1fr))")
                .expect("auto-fill should parse"),
            layout::TrackComponent::Repeat(
                layout::TrackRepetition::auto_fill(vec![layout::TrackSizing::minmax(
                    min_track_px(150.0),
                    layout::MaxTrackSizing::fr(1.0)
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
        let mut tree = TestTree {
            nodes: vec![TestNode {
                layout_input: layout::LayoutInput::box_input(layout::NodeInput {
                    display: layout::Display::InlineGrid,
                    grid_template_columns: vec![track_component_px(40.0)],
                    grid_template_rows: vec![track_component_px(20.0)],
                    ..layout::NodeInput::default()
                }),
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
                final_layout: layout::NodeOutput::new(),
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
}
