use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use surgeist_layout as layout;
use surgeist_layout::{CacheAccess as _, Compute as _};
use surgeist_retained as retained;
use surgeist_style as s;

type Scalar = layout::Scalar;

#[derive(Default)]
struct StyleFixtureTree {
    state: retained::State,
}

impl s::Tree for StyleFixtureTree {
    type Id = usize;

    fn version_hint(&self) -> Option<u64> {
        None
    }

    fn node(&self, id: Self::Id) -> s::Result<s::Node<'_, Self::Id>> {
        Ok(s::Node {
            id,
            tag: None,
            key: None,
            classes: &[],
            attributes: &[],
            role: retained::Role::Generic,
            state: &self.state,
            text: false,
        })
    }

    fn parent(&self, _id: Self::Id, _traversal: s::Traversal) -> s::Result<Option<Self::Id>> {
        Ok(None)
    }

    fn children(
        &self,
        _id: Self::Id,
        _traversal: s::Traversal,
    ) -> s::Result<impl Iterator<Item = Self::Id> + '_> {
        Ok(std::iter::empty())
    }

    fn previous_sibling(
        &self,
        _id: Self::Id,
        _traversal: s::Traversal,
    ) -> s::Result<Option<Self::Id>> {
        Ok(None)
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
                root_context: parse_root_context(
                    viewport.attribute("root-context").unwrap_or("root"),
                )?,
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

    match golden.viewport.root_context {
        RootContext::Root => layout::compute_root(&mut tree, 0, available),
        RootContext::FlexItem => compute_viewport_flex_item_root(&mut tree, available),
    }
    if golden.use_rounding {
        layout::round_layout(&mut tree, 0);
    } else {
        tree.copy_unrounded_to_final();
    }

    compare_expectation(&tree, 0, &golden.expectations, &golden.name)
}

fn compute_viewport_flex_item_root(
    tree: &mut TestTree,
    available: layout::Size<layout::Available>,
) {
    let output = tree.compute_child(
        0,
        layout::ComputeInput {
            run_mode: layout::RunMode::PerformRootLayout,
            sizing_mode: layout::SizingMode::InherentSize,
            axis: layout::RequestedAxis::Both,
            known: layout::Size::NONE,
            parent: available.map(layout::Available::into_option),
            available,
        },
    );
    tree.set_unrounded(
        0,
        layout::NodeOutput {
            order: 0,
            location: layout::Point::ZERO,
            size: output.size,
            content_size: output.content_size,
            ..layout::NodeOutput::new()
        },
    );
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootContext {
    Root,
    FlexItem,
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

fn parse_root_context(raw: &str) -> Result<RootContext, Error> {
    match raw {
        "root" => Ok(RootContext::Root),
        "flex-item" => Ok(RootContext::FlexItem),
        _ => Err(Error::new(format!("unsupported root context `{raw}`"))),
    }
}

fn parse_bool(raw: &str) -> Result<bool, Error> {
    match raw {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Error::new(format!("invalid boolean `{raw}`"))),
    }
}

fn parse_number(raw: &str) -> Result<Scalar, Error> {
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
    calc_store: layout::LayoutCalcStore,
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
        let mut lowering = s::adapters::layout::LayoutLoweringSession::new();
        tree.push_node(
            root,
            InheritedTextContext {
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: LineHeightState::Normal,
                grid_lanes_text: false,
                inline_level_text: false,
            },
            &mut lowering,
        )?;
        tree.calc_store = lowering.finish();
        Ok(tree)
    }

    fn push_node(
        &mut self,
        node: &Node,
        inherited: InheritedTextContext,
        lowering: &mut s::adapters::layout::LayoutLoweringSession,
    ) -> Result<usize, Error> {
        let id = self.nodes.len();
        let font_family = font_family(&node.style)?.unwrap_or(inherited.font_family);
        let font_size = font_size(&node.style)?.unwrap_or(inherited.font_size);
        let line_height = match line_height(&node.style)? {
            Some(value) => LineHeightState::Px(value),
            None => inherited.line_height,
        };
        let resolved_line_height = line_height.resolve(font_size);
        let node_input = to_node_input(&node.style, lowering)?;
        let grid_lanes_text = inherited.grid_lanes_text
            || node_input
                .display
                .establishes_grid_lanes_formatting_context();
        let inline_level_text = inherited.inline_level_text || node_input.display.is_inline_level();
        self.nodes.push(TestNode {
            node_input,
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
                    lowering,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(text) = &node.text
            && grid_text_container_needs_anonymous_child(self.nodes[id].node_input.display)
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

    fn compute_uncached(
        &mut self,
        node: usize,
        input: layout::ComputeInput,
    ) -> layout::ComputeOutput {
        let node_input = self.nodes[node].node_input.clone();
        if node_input.display == layout::Display::None
            || input.run_mode == layout::RunMode::PerformHiddenLayout
        {
            return layout::compute_hidden(self, node);
        }

        if can_use_leaf_measurement(
            node_input.display,
            self.nodes[node].children.len(),
            self.nodes[node].text.is_some(),
        ) {
            let mut output = layout::compute_leaf(input, &node_input, |known, available| {
                self.measure(node, known, available)
            });
            if let Some(text) = &self.nodes[node].text {
                let baseline = TextMeasure::new(
                    text,
                    self.nodes[node].font_family,
                    self.nodes[node].font_size,
                    self.nodes[node].line_height,
                    self.nodes[node].preserve_fractional_min_content,
                    self.nodes[node].use_tighter_monospace_wrap,
                )
                .baseline()
                .min(output.size.height);
                output.first_baselines.y = Some(baseline);
                output.last_baselines.y = Some(baseline);
            }
            return output;
        }

        match node_input.display.inner_display() {
            layout::Display::Block => layout::compute_block(self, node, input),
            layout::Display::Flex => layout::compute_flex(self, node, input),
            layout::Display::Grid | layout::Display::GridLanes => {
                layout::compute_grid(self, node, input)
            }
            layout::Display::None => layout::compute_hidden(self, node),
            layout::Display::InlineBlock
            | layout::Display::InlineGrid
            | layout::Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }

    fn measure(
        &self,
        node: usize,
        known: layout::Size<Option<Scalar>>,
        available: layout::Size<layout::Available>,
    ) -> layout::Size {
        if let Some(text) = &self.nodes[node].text {
            let text = TextMeasure::new(
                text,
                self.nodes[node].font_family,
                self.nodes[node].font_size,
                self.nodes[node].line_height,
                self.nodes[node].preserve_fractional_min_content,
                self.nodes[node].use_tighter_monospace_wrap,
            );
            if self.nodes[node].node_input.writing_mode.is_vertical() {
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

    fn copy_unrounded_to_final(&mut self) {
        for node in &mut self.nodes {
            node.final_layout = node.unrounded;
        }
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

    fn baseline(self) -> Scalar {
        self.font_size * 0.8
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

impl layout::Compute for TestTree {
    fn node_input(&self, node: Self::Node) -> &layout::NodeInput {
        &self.nodes[node].node_input
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: layout::NodeOutput) {
        self.nodes[node].unrounded = layout;
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: layout::ComputeInput,
    ) -> layout::ComputeOutput {
        if let Some(output) = self.cache_get(node, &input) {
            return output;
        }
        let output = self.compute_uncached(node, input);
        self.cache_store(node, &input, output);
        output
    }

    fn calc_resolver(&self) -> &dyn layout::CalcResolver {
        self
    }
}

impl layout::CalcResolver for TestTree {
    fn resolve_calc(&self, id: layout::CalcId, basis: Option<Scalar>) -> layout::CalcResolution {
        self.calc_store.resolve_calc(id, basis)
    }

    fn calc_depends_on_basis(&self, id: layout::CalcId) -> bool {
        self.calc_store.calc_depends_on_basis(id)
    }

    fn calc_percent_fraction(&self, id: layout::CalcId) -> Option<Scalar> {
        self.calc_store.calc_percent_fraction(id)
    }
}

impl layout::Round for TestTree {
    fn unrounded(&self, node: Self::Node) -> layout::NodeOutput {
        self.nodes[node].unrounded
    }

    fn set_final(&mut self, node: Self::Node, layout: layout::NodeOutput) {
        self.nodes[node].final_layout = layout;
    }
}

impl layout::CacheAccess for TestTree {
    type Node = usize;

    fn cache_get(
        &self,
        node: Self::Node,
        input: &layout::ComputeInput,
    ) -> Option<layout::ComputeOutput> {
        self.nodes[node].cache.get(input)
    }

    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &layout::ComputeInput,
        output: layout::ComputeOutput,
    ) {
        self.nodes[node].cache.store(input, output);
    }

    fn cache_clear(&mut self, node: Self::Node) {
        self.nodes[node].cache.clear();
    }
}

fn compare_expectation(
    tree: &TestTree,
    node: usize,
    expected: &Expectation,
    path: &str,
) -> Result<(), Error> {
    let actual = tree.nodes[node].final_layout;
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
        compare_expectation(tree, child, expected_child, &format!("{path}/{index}"))?;
    }

    Ok(())
}

fn compare_number(path: &str, field: &str, actual: Scalar, expected: Scalar) -> Result<(), Error> {
    const TOLERANCE: Scalar = 0.1;
    if (actual - expected).abs() < TOLERANCE {
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

fn to_node_input(
    attrs: &StyleAttrs,
    lowering: &mut s::adapters::layout::LayoutLoweringSession,
) -> Result<layout::NodeInput, Error> {
    let declarations = to_declarations(attrs)?;
    let tree = StyleFixtureTree::default();
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&declarations))
        .map_err(|error| Error::new(error.to_string()))?;
    let mut input = lowering
        .lower_node(&resolved)
        .map_err(|error| Error::new(error.to_string()))?;
    if let Some(value) = attrs.get("vertical-align") {
        input.vertical_align = parse_vertical_align(value)?;
    }
    Ok(input)
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

fn to_declarations(attrs: &StyleAttrs) -> Result<s::Declarations, Error> {
    let mut declarations = s::Declarations::new();
    if attrs.get("source-tag") == Some("br") {
        return Err(Error::new(
            "unsupported source-tag `br`; line-break semantics are not represented",
        ));
    }
    let display = match attrs.get("display") {
        Some(value) => Some(parse_display(value)?),
        None => match attrs.get("source-tag") {
            Some("div") => Some(layout::Display::Block),
            _ => None,
        },
    };
    if let Some(display) = display {
        declarations.insert(
            s::Property::Display,
            s::Value::Display(to_style_display(display)),
        );
    }
    if let Some(value) = attrs.get("box-sizing") {
        declarations.insert(
            s::Property::BoxSizing,
            s::Value::BoxSizing(to_style_box_sizing(parse_box_sizing(value)?)),
        );
    }
    if let Some(value) = attrs.get("direction") {
        declarations.insert(
            s::Property::Direction,
            s::Value::Direction(to_style_direction(parse_direction(value)?)),
        );
    }
    if let Some(value) = attrs.get("position") {
        declarations.insert(
            s::Property::Position,
            s::Value::Position(to_style_position(parse_position(value)?)),
        );
    }
    if let Some(value) = attrs.get("float") {
        declarations.insert(
            s::Property::Float,
            s::Value::Float(to_style_float(parse_float(value)?)),
        );
    }
    if let Some(value) = attrs.get("clear") {
        declarations.insert(
            s::Property::Clear,
            s::Value::Clear(to_style_clear(parse_clear(value)?)),
        );
    }
    if let Some(value) = attrs.get("overflow-x") {
        declarations.insert(
            s::Property::OverflowX,
            s::Value::Overflow(to_style_overflow(parse_overflow(value)?)),
        );
    }
    if let Some(value) = attrs.get("overflow-y") {
        declarations.insert(
            s::Property::OverflowY,
            s::Value::Overflow(to_style_overflow(parse_overflow(value)?)),
        );
    }
    if let Some(value) = attrs.get("scrollbar-width") {
        declarations.insert(
            s::Property::ScrollbarWidth,
            s::Value::Number(parse_number(value)?),
        );
    }
    if let Some(value) = attrs.get("text-align") {
        declarations.insert(
            s::Property::TextAlign,
            s::Value::TextAlign(to_style_text_align(parse_text_align(value)?)),
        );
    }
    if let Some(value) = attrs.get("line-height") {
        declarations.insert(
            s::Property::LineHeight,
            s::Value::Length(to_style_dimension(parse_dimension(value)?)?),
        );
    }
    declarations.insert(
        s::Property::WritingMode,
        s::Value::WritingMode(to_style_writing_mode(parse_writing_mode(
            attrs.get("writing-mode"),
        )?)),
    );
    if let Some(value) = attrs.get("flex-direction") {
        declarations.insert(
            s::Property::FlexDirection,
            s::Value::FlexDirection(to_style_flex_direction(parse_flex_direction(value)?)),
        );
    }
    if let Some(value) = attrs.get("flex-wrap") {
        declarations.insert(
            s::Property::FlexWrap,
            s::Value::FlexWrap(to_style_flex_wrap(parse_flex_wrap(value)?)),
        );
    }
    if let Some(value) = attrs.get("flex-grow") {
        declarations.insert(
            s::Property::FlexGrow,
            s::Value::Number(parse_number(value)?),
        );
    }
    if let Some(value) = attrs.get("flex-shrink") {
        declarations.insert(
            s::Property::FlexShrink,
            s::Value::Number(parse_number(value)?),
        );
    }
    if let Some(value) = attrs.get("flex-basis") {
        declarations.insert(
            s::Property::FlexBasis,
            s::Value::Length(parse_style_dimension(value)?),
        );
    }
    if let Some(value) = attrs.get("width") {
        declarations.insert(
            s::Property::Width,
            s::Value::Length(parse_style_dimension(value)?),
        );
    }
    if let Some(value) = attrs.get("height") {
        declarations.insert(
            s::Property::Height,
            s::Value::Length(parse_style_dimension(value)?),
        );
    }
    if let Some(value) = attrs.get("min-width") {
        declarations.insert(
            s::Property::MinWidth,
            s::Value::Length(parse_style_dimension(value)?),
        );
    }
    if let Some(value) = attrs.get("min-height") {
        declarations.insert(
            s::Property::MinHeight,
            s::Value::Length(parse_style_dimension(value)?),
        );
    }
    if let Some(value) = attrs.get("max-width") {
        declarations.insert(
            s::Property::MaxWidth,
            s::Value::Length(parse_style_dimension(value)?),
        );
    }
    if let Some(value) = attrs.get("max-height") {
        declarations.insert(
            s::Property::MaxHeight,
            s::Value::Length(parse_style_dimension(value)?),
        );
    }
    if let Some(value) = attrs.get("aspect-ratio") {
        declarations.insert(
            s::Property::AspectRatio,
            s::Value::Number(parse_number(value)?),
        );
    }
    if let Some(value) = attrs.get("row-gap") {
        declarations.insert(
            s::Property::RowGap,
            s::Value::Length(parse_style_length(value)?),
        );
    }
    if let Some(value) = attrs.get("column-gap") {
        declarations.insert(
            s::Property::ColumnGap,
            s::Value::Length(parse_style_length(value)?),
        );
    }

    insert_edges_auto(
        &mut declarations,
        attrs,
        s::Property::Margin,
        s::Edges::default(),
        [
            ("margin-top", 0),
            ("margin-right", 1),
            ("margin-bottom", 2),
            ("margin-left", 3),
        ],
    )?;
    insert_edges(
        &mut declarations,
        attrs,
        s::Property::Padding,
        s::Edges::default(),
        [
            ("padding-top", 0),
            ("padding-right", 1),
            ("padding-bottom", 2),
            ("padding-left", 3),
        ],
    )?;
    insert_edges(
        &mut declarations,
        attrs,
        s::Property::BorderWidth,
        s::Edges::default(),
        [
            ("border-top", 0),
            ("border-right", 1),
            ("border-bottom", 2),
            ("border-left", 3),
        ],
    )?;
    insert_edges_auto(
        &mut declarations,
        attrs,
        s::Property::Inset,
        s::Edges::all(s::Length::Auto),
        [("top", 0), ("right", 1), ("bottom", 2), ("left", 3)],
    )?;

    if let Some(value) = attrs.get("align-items") {
        declarations.insert(
            s::Property::AlignItems,
            s::Value::AlignItems(to_style_align_items(parse_align_items(value)?)),
        );
    }
    if let Some(value) = attrs.get("align-self") {
        declarations.insert(
            s::Property::AlignSelf,
            s::Value::AlignItems(to_style_align_items(parse_align_items(value)?)),
        );
    }
    if let Some(value) = attrs.get("justify-items") {
        declarations.insert(
            s::Property::JustifyItems,
            s::Value::AlignItems(to_style_align_items(parse_align_items(value)?)),
        );
    }
    if let Some(value) = attrs.get("justify-self") {
        declarations.insert(
            s::Property::JustifySelf,
            s::Value::AlignItems(to_style_align_items(parse_align_items(value)?)),
        );
    }
    if let Some(value) = attrs.get("align-content") {
        declarations.insert(
            s::Property::AlignContent,
            s::Value::AlignContent(to_style_align_content(parse_align_content(value)?)),
        );
    }
    if let Some(value) = attrs.get("justify-content") {
        declarations.insert(
            s::Property::JustifyContent,
            s::Value::AlignContent(to_style_align_content(parse_align_content(value)?)),
        );
    }
    if let Some(value) = attrs.get("grid-auto-flow") {
        declarations.insert(
            s::Property::GridAutoFlow,
            s::Value::GridAutoFlow(to_style_grid_auto_flow(parse_grid_auto_flow(value)?)),
        );
    }
    if let Some(value) = attrs.get("grid-template-columns") {
        declarations.insert(
            s::Property::GridTemplateColumns,
            s::Value::GridTrackList(parse_style_track_component_list(value)?),
        );
    }
    if let Some(value) = attrs.get("grid-template-rows") {
        declarations.insert(
            s::Property::GridTemplateRows,
            s::Value::GridTrackList(parse_style_track_component_list(value)?),
        );
    }
    if let Some(value) = attrs.get("grid-template-areas") {
        declarations.insert(
            s::Property::GridTemplateAreas,
            s::Value::GridTemplateAreas(parse_grid_template_areas(value)?),
        );
    }
    if let Some(value) = attrs.get("grid-auto-columns") {
        declarations.insert(
            s::Property::GridAutoColumns,
            s::Value::GridTrackList(parse_style_track_component_list(value)?),
        );
    }
    if let Some(value) = attrs.get("grid-auto-rows") {
        declarations.insert(
            s::Property::GridAutoRows,
            s::Value::GridTrackList(parse_style_track_component_list(value)?),
        );
    }
    if let Some(value) = attrs.get("grid-column-start") {
        declarations.insert(
            s::Property::GridColumnStart,
            s::Value::GridLine(parse_style_grid_line(value)?),
        );
    }
    if let Some(value) = attrs.get("grid-column-end") {
        declarations.insert(
            s::Property::GridColumnEnd,
            s::Value::GridLine(parse_style_grid_line(value)?),
        );
    }
    if let Some(value) = attrs.get("grid-row-start") {
        declarations.insert(
            s::Property::GridRowStart,
            s::Value::GridLine(parse_style_grid_line(value)?),
        );
    }
    if let Some(value) = attrs.get("grid-row-end") {
        declarations.insert(
            s::Property::GridRowEnd,
            s::Value::GridLine(parse_style_grid_line(value)?),
        );
    }
    Ok(declarations)
}

fn insert_edges(
    declarations: &mut s::Declarations,
    attrs: &StyleAttrs,
    property: s::Property,
    mut edges: s::Edges,
    names: [(&str, usize); 4],
) -> Result<(), Error> {
    let mut present = false;
    for (name, side) in names {
        if let Some(value) = attrs.get(name) {
            present = true;
            set_edge(&mut edges, side, parse_style_length(value)?);
        }
    }
    if present {
        declarations.insert(property, s::Value::Edges(edges));
    }
    Ok(())
}

fn insert_edges_auto(
    declarations: &mut s::Declarations,
    attrs: &StyleAttrs,
    property: s::Property,
    mut edges: s::Edges,
    names: [(&str, usize); 4],
) -> Result<(), Error> {
    let mut present = false;
    for (name, side) in names {
        if let Some(value) = attrs.get(name) {
            present = true;
            set_edge(&mut edges, side, parse_style_length_auto(value)?);
        }
    }
    if present {
        declarations.insert(property, s::Value::Edges(edges));
    }
    Ok(())
}

fn set_edge(edges: &mut s::Edges, side: usize, value: s::Length) {
    match side {
        0 => edges.top = value,
        1 => edges.right = value,
        2 => edges.bottom = value,
        3 => edges.left = value,
        _ => unreachable!("edge side index is fixed by caller"),
    }
}

fn to_style_display(value: layout::Display) -> s::Display {
    match value {
        layout::Display::Block => s::Display::Block,
        layout::Display::Flex => s::Display::Flex,
        layout::Display::Grid => s::Display::Grid,
        layout::Display::GridLanes => s::Display::GridLanes,
        layout::Display::InlineBlock => s::Display::InlineBlock,
        layout::Display::InlineGrid => s::Display::InlineGrid,
        layout::Display::InlineGridLanes => s::Display::InlineGridLanes,
        layout::Display::None => s::Display::None,
    }
}

fn to_style_box_sizing(value: layout::BoxSizing) -> s::BoxSizing {
    match value {
        layout::BoxSizing::ContentBox => s::BoxSizing::ContentBox,
        layout::BoxSizing::BorderBox => s::BoxSizing::BorderBox,
    }
}

fn to_style_direction(value: layout::Direction) -> s::Direction {
    match value {
        layout::Direction::Ltr => s::Direction::Ltr,
        layout::Direction::Rtl => s::Direction::Rtl,
    }
}

fn to_style_position(value: layout::Position) -> s::LayoutPosition {
    match value {
        layout::Position::Relative => s::LayoutPosition::Relative,
        layout::Position::Absolute => s::LayoutPosition::Absolute,
    }
}

fn to_style_float(value: layout::Float) -> s::Float {
    match value {
        layout::Float::None => s::Float::None,
        layout::Float::Left => s::Float::Left,
        layout::Float::Right => s::Float::Right,
    }
}

fn to_style_clear(value: layout::Clear) -> s::Clear {
    match value {
        layout::Clear::None => s::Clear::None,
        layout::Clear::Left => s::Clear::Left,
        layout::Clear::Right => s::Clear::Right,
        layout::Clear::Both => s::Clear::Both,
    }
}

fn to_style_overflow(value: layout::Overflow) -> s::Overflow {
    match value {
        layout::Overflow::Visible => s::Overflow::Visible,
        layout::Overflow::Clip => s::Overflow::Clip,
        layout::Overflow::Hidden => s::Overflow::Hidden,
        layout::Overflow::Scroll => s::Overflow::Scroll,
    }
}

fn to_style_text_align(value: layout::TextAlign) -> s::StyleTextAlign {
    match value {
        layout::TextAlign::Auto => s::StyleTextAlign::Auto,
        layout::TextAlign::LegacyLeft => s::StyleTextAlign::LegacyLeft,
        layout::TextAlign::LegacyRight => s::StyleTextAlign::LegacyRight,
        layout::TextAlign::LegacyCenter => s::StyleTextAlign::LegacyCenter,
    }
}

fn to_style_writing_mode(value: layout::WritingMode) -> s::WritingMode {
    match value {
        layout::WritingMode::HorizontalTb => s::WritingMode::HorizontalTb,
        layout::WritingMode::VerticalLr => s::WritingMode::VerticalLr,
        layout::WritingMode::VerticalRl => s::WritingMode::VerticalRl,
    }
}

fn to_style_flex_direction(value: layout::FlexDirection) -> s::FlexDirection {
    match value {
        layout::FlexDirection::Row => s::FlexDirection::Row,
        layout::FlexDirection::Column => s::FlexDirection::Column,
        layout::FlexDirection::RowReverse => s::FlexDirection::RowReverse,
        layout::FlexDirection::ColumnReverse => s::FlexDirection::ColumnReverse,
    }
}

fn to_style_flex_wrap(value: layout::FlexWrap) -> s::FlexWrap {
    match value {
        layout::FlexWrap::NoWrap => s::FlexWrap::NoWrap,
        layout::FlexWrap::Wrap => s::FlexWrap::Wrap,
        layout::FlexWrap::WrapReverse => s::FlexWrap::WrapReverse,
    }
}

fn to_style_align_items(value: layout::AlignItems) -> s::AlignItems {
    match value {
        layout::AlignItems::Start => s::AlignItems::Start,
        layout::AlignItems::End => s::AlignItems::End,
        layout::AlignItems::FlexStart => s::AlignItems::FlexStart,
        layout::AlignItems::FlexEnd => s::AlignItems::FlexEnd,
        layout::AlignItems::Center => s::AlignItems::Center,
        layout::AlignItems::SafeEnd => s::AlignItems::SafeEnd,
        layout::AlignItems::SafeFlexEnd => s::AlignItems::SafeFlexEnd,
        layout::AlignItems::SafeCenter => s::AlignItems::SafeCenter,
        layout::AlignItems::Baseline => s::AlignItems::Baseline,
        layout::AlignItems::LastBaseline => s::AlignItems::LastBaseline,
        layout::AlignItems::Stretch => s::AlignItems::Stretch,
    }
}

fn to_style_align_content(value: layout::AlignContent) -> s::AlignContent {
    match value {
        layout::AlignContent::Start => s::AlignContent::Start,
        layout::AlignContent::End => s::AlignContent::End,
        layout::AlignContent::FlexStart => s::AlignContent::FlexStart,
        layout::AlignContent::FlexEnd => s::AlignContent::FlexEnd,
        layout::AlignContent::Center => s::AlignContent::Center,
        layout::AlignContent::SafeEnd => s::AlignContent::SafeEnd,
        layout::AlignContent::SafeFlexEnd => s::AlignContent::SafeFlexEnd,
        layout::AlignContent::SafeCenter => s::AlignContent::SafeCenter,
        layout::AlignContent::Stretch => s::AlignContent::Stretch,
        layout::AlignContent::SpaceBetween => s::AlignContent::SpaceBetween,
        layout::AlignContent::SpaceEvenly => s::AlignContent::SpaceEvenly,
        layout::AlignContent::SpaceAround => s::AlignContent::SpaceAround,
    }
}

fn to_style_grid_auto_flow(value: layout::GridAutoFlow) -> s::GridAutoFlow {
    match value {
        layout::GridAutoFlow::Row => s::GridAutoFlow::Row,
        layout::GridAutoFlow::Column => s::GridAutoFlow::Column,
        layout::GridAutoFlow::RowDense => s::GridAutoFlow::RowDense,
        layout::GridAutoFlow::ColumnDense => s::GridAutoFlow::ColumnDense,
    }
}

fn to_style_dimension(value: layout::Dimension) -> Result<s::Length, Error> {
    Ok(match value {
        layout::Dimension::Px(value) => s::Length::px(value),
        layout::Dimension::Percent(value) => s::Length::percent(value * 100.0),
        layout::Dimension::Auto => s::Length::Auto,
        layout::Dimension::MinContent => s::Length::MinContent,
        layout::Dimension::MaxContent => s::Length::MaxContent,
        layout::Dimension::Calc(id) => {
            return Err(Error::new(format!(
                "unsupported calc dimension handle `{}`",
                id.index()
            )));
        }
        layout::Dimension::Fr(value) if (value - 1.0).abs() < Scalar::EPSILON => s::Length::Fill,
        layout::Dimension::Fr(value) => {
            return Err(Error::new(format!(
                "unsupported non-unit flexible dimension `{value}fr`"
            )));
        }
    })
}

fn to_style_length(value: layout::Length) -> s::Length {
    match value {
        layout::Length::Normal => s::Length::NORMAL,
        layout::Length::Px(value) => s::Length::px(value),
        layout::Length::Percent(value) => s::Length::percent(value * 100.0),
        layout::Length::Calc(id) => {
            panic!("unsupported calc length handle `{}`", id.index());
        }
    }
}

fn to_style_track_component(
    component: layout::TrackComponent,
) -> Result<s::GridTrackComponent, Error> {
    Ok(match component {
        layout::TrackComponent::Track(track) => {
            s::GridTrackComponent::Track(to_style_track_sizing(track)?)
        }
        layout::TrackComponent::Repeat(repeat) => {
            s::GridTrackComponent::Repeat(to_style_track_repeat(repeat)?)
        }
        layout::TrackComponent::LineNames(names) => s::GridTrackComponent::LineNames(names),
        layout::TrackComponent::Subgrid(subgrid) => {
            s::GridTrackComponent::Subgrid(s::SubgridTrack::from_components(
                subgrid
                    .name_components
                    .into_iter()
                    .map(to_style_subgrid_line_name_component)
                    .collect(),
            ))
        }
    })
}

fn to_style_track_repeat(repeat: layout::TrackRepetition) -> Result<s::TrackRepeat, Error> {
    let components = repeat
        .components
        .into_iter()
        .map(to_style_track_component)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(match repeat.repeat {
        layout::TrackRepeat::Count(count) => s::TrackRepeat::count(
            count
                .try_into()
                .map_err(|_| Error::new("repeat count does not fit style track repeat"))?,
            components,
        ),
        layout::TrackRepeat::AutoFill => s::TrackRepeat::auto_fill(components),
        layout::TrackRepeat::AutoFit => s::TrackRepeat::auto_fit(components),
    })
}

fn to_style_subgrid_line_name_component(
    component: layout::SubgridLineNameComponent,
) -> s::SubgridLineNameComponent {
    match component {
        layout::SubgridLineNameComponent::LineNames(names) => {
            s::SubgridLineNameComponent::LineNames(names)
        }
        layout::SubgridLineNameComponent::Repeat {
            count,
            line_name_sets,
        } => s::SubgridLineNameComponent::Repeat {
            count: match count {
                layout::SubgridLineNameRepeatCount::Count(count) => {
                    s::SubgridLineNameRepeatCount::Count(count)
                }
                layout::SubgridLineNameRepeatCount::AutoFill => {
                    s::SubgridLineNameRepeatCount::AutoFill
                }
            },
            line_name_sets,
        },
    }
}

fn to_style_track_sizing(track: layout::TrackSizing) -> Result<s::TrackSizing, Error> {
    Ok(s::TrackSizing::minmax(
        to_style_min_track_sizing(track.min),
        to_style_max_track_sizing(track.max),
    ))
}

fn to_style_min_track_sizing(track: layout::MinTrackSizing) -> s::MinTrackSizing {
    match track {
        layout::MinTrackSizing::Length(length) => {
            s::MinTrackSizing::Length(to_style_length(length))
        }
        layout::MinTrackSizing::Auto => s::MinTrackSizing::Auto,
        layout::MinTrackSizing::MinContent => s::MinTrackSizing::MinContent,
        layout::MinTrackSizing::MaxContent => s::MinTrackSizing::MaxContent,
    }
}

fn to_style_max_track_sizing(track: layout::MaxTrackSizing) -> s::MaxTrackSizing {
    match track {
        layout::MaxTrackSizing::Length(length) => {
            s::MaxTrackSizing::Length(to_style_length(length))
        }
        layout::MaxTrackSizing::Flex(value) => s::MaxTrackSizing::Flex(value),
        layout::MaxTrackSizing::Auto => s::MaxTrackSizing::Auto,
        layout::MaxTrackSizing::MinContent => s::MaxTrackSizing::MinContent,
        layout::MaxTrackSizing::MaxContent => s::MaxTrackSizing::MaxContent,
        layout::MaxTrackSizing::FitContent(length) => {
            s::MaxTrackSizing::FitContent(to_style_length(length))
        }
    }
}

fn parse_style_grid_line(raw: &str) -> Result<s::GridLine, Error> {
    let tokens = split_top_level_whitespace(raw);
    match tokens.as_slice() {
        [token] if token == "auto" => Ok(s::GridLine::Auto),
        [token] if token == "span" => Err(Error::new("invalid grid span `span`")),
        [token] => match parse_style_line_index(token) {
            Ok(line) => Ok(s::GridLine::Line(line)),
            Err(_) => Ok(s::GridLine::BareIdent(
                parse_custom_ident(token)?.to_owned(),
            )),
        },
        [span, token] if span == "span" => {
            if let Ok(index) = parse_style_span_index(token) {
                Ok(s::GridLine::Span(index))
            } else {
                Ok(s::GridLine::NamedSpan {
                    name: parse_custom_ident(token)?.to_owned(),
                    index: 1,
                })
            }
        }
        [name, index] if parse_style_line_index(index).is_ok() => named_line(name, index, raw),
        [index, name] if parse_style_line_index(index).is_ok() => named_line(name, index, raw),
        [span, first, second] if span == "span" => {
            if let Ok(index) = parse_style_span_index(first) {
                Ok(s::GridLine::NamedSpan {
                    name: parse_custom_ident(second)?.to_owned(),
                    index,
                })
            } else {
                Ok(s::GridLine::NamedSpan {
                    name: parse_custom_ident(first)?.to_owned(),
                    index: parse_style_span_index(second)?,
                })
            }
        }
        _ => Err(Error::new(format!("unsupported grid line `{raw}`"))),
    }
}

fn parse_grid_template_areas(raw: &str) -> Result<s::GridTemplateAreas, Error> {
    let rows = raw
        .split('/')
        .map(str::trim)
        .filter(|row| !row.is_empty())
        .map(parse_grid_template_area_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(s::GridTemplateAreas::new(rows))
}

fn parse_grid_template_area_row(raw: &str) -> Result<s::GridTemplateAreaRow, Error> {
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
    Ok(s::GridTemplateAreaRow::new(cells))
}

fn is_grid_template_area_null_cell(cell: &str) -> bool {
    !cell.is_empty() && cell.bytes().all(|byte| byte == b'.')
}

fn named_line(name: &str, index: &str, raw: &str) -> Result<s::GridLine, Error> {
    let index = parse_style_line_index(index)?;
    if index == 0 {
        return Err(Error::new(format!(
            "named grid line occurrence cannot be zero in `{raw}`"
        )));
    }
    Ok(s::GridLine::NamedLine {
        name: parse_custom_ident(name)?.to_owned(),
        index,
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

fn parse_style_track_component_list(raw: &str) -> Result<s::GridTrackList, Error> {
    let components = if raw.trim_start().starts_with("subgrid") {
        vec![to_style_track_component(parse_subgrid_track_component(
            raw,
        )?)?]
    } else {
        split_top_level_whitespace(raw)
            .into_iter()
            .map(|part| parse_style_track_component(&part))
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(s::GridTrackList::new(components))
}

fn parse_style_track_component(raw: &str) -> Result<s::GridTrackComponent, Error> {
    if let Some(body) = function_body(raw, "repeat") {
        let (count, tracks) = split_once_top_level_comma(body)?;
        let components = parse_style_track_component_list(tracks)?.components;
        let repeat = match count.trim() {
            "auto-fill" => s::TrackRepeat::auto_fill(components),
            "auto-fit" => s::TrackRepeat::auto_fit(components),
            raw => s::TrackRepeat::count(
                raw.parse()
                    .map_err(|_| Error::new(format!("invalid repeat count `{raw}`")))?,
                components,
            ),
        };
        return Ok(s::GridTrackComponent::Repeat(repeat));
    }
    if raw.starts_with('[') {
        return Ok(s::GridTrackComponent::LineNames(parse_subgrid_line_names(
            raw,
        )?));
    }
    Ok(s::GridTrackComponent::Track(parse_style_track_sizing(raw)?))
}

fn parse_style_track_sizing(raw: &str) -> Result<s::TrackSizing, Error> {
    if let Some(body) = function_body(raw, "minmax") {
        let (min, max) = split_once_top_level_comma(body)?;
        return Ok(s::TrackSizing::minmax(
            parse_style_min_track_sizing(min.trim())?,
            parse_style_max_track_sizing(max.trim())?,
        ));
    }
    if let Some(body) = function_body(raw, "fit-content") {
        return Ok(s::TrackSizing::minmax(
            s::MinTrackSizing::Auto,
            s::MaxTrackSizing::FitContent(parse_style_length(body.trim())?),
        ));
    }
    if let Some(flex) = raw.strip_suffix("fr") {
        return Ok(s::TrackSizing::fr(parse_number(flex)?));
    }
    match parse_style_dimension(raw)? {
        s::Length::Auto => Ok(s::TrackSizing::AUTO),
        s::Length::MinContent => Ok(s::TrackSizing::minmax(
            s::MinTrackSizing::MinContent,
            s::MaxTrackSizing::MinContent,
        )),
        s::Length::MaxContent => Ok(s::TrackSizing::minmax(
            s::MinTrackSizing::MaxContent,
            s::MaxTrackSizing::MaxContent,
        )),
        s::Length::Fill => Ok(s::TrackSizing::fr(1.0)),
        length => Ok(s::TrackSizing::minmax(
            s::MinTrackSizing::Length(length.clone()),
            s::MaxTrackSizing::Length(length),
        )),
    }
}

fn parse_style_min_track_sizing(raw: &str) -> Result<s::MinTrackSizing, Error> {
    match raw {
        "auto" => Ok(s::MinTrackSizing::Auto),
        "min-content" => Ok(s::MinTrackSizing::MinContent),
        "max-content" => Ok(s::MinTrackSizing::MaxContent),
        _ => Ok(s::MinTrackSizing::Length(parse_style_length(raw)?)),
    }
}

fn parse_style_max_track_sizing(raw: &str) -> Result<s::MaxTrackSizing, Error> {
    match raw {
        "auto" => Ok(s::MaxTrackSizing::Auto),
        "min-content" => Ok(s::MaxTrackSizing::MinContent),
        "max-content" => Ok(s::MaxTrackSizing::MaxContent),
        _ if raw.ends_with("fr") => {
            let value = raw.trim_end_matches("fr");
            Ok(s::MaxTrackSizing::fr(parse_number(value)?))
        }
        _ => Ok(s::MaxTrackSizing::Length(parse_style_length(raw)?)),
    }
}

fn parse_style_calc_length(raw: &str) -> Result<s::CalcLength, Error> {
    let body = raw
        .strip_prefix("calc(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| Error::new(format!("unsupported calc expression `{raw}`")))?;
    parse_style_calc_sum(body.trim(), raw)
}

fn parse_style_calc_sum(body: &str, raw: &str) -> Result<s::CalcLength, Error> {
    let parts = body.split_whitespace().collect::<Vec<_>>();
    let [first, operator, second] = parts.as_slice() else {
        return Err(Error::new(format!("unsupported calc expression `{raw}`")));
    };

    let left = parse_style_calc_term(first)?;
    let right = parse_style_calc_term(second)?;
    let right = match *operator {
        "+" => s::CalcLengthTerm::add(right),
        "-" => s::CalcLengthTerm::sub(right),
        _ => return Err(Error::new(format!("unsupported calc expression `{raw}`"))),
    };

    Ok(s::CalcLength::sum([s::CalcLengthTerm::add(left), right]))
}

fn parse_style_calc_term(raw: &str) -> Result<s::CalcLength, Error> {
    if let Some(px) = raw.strip_suffix("px") {
        return Ok(s::CalcLength::px(parse_number(px)?));
    }
    if let Some(percent) = raw.strip_suffix('%') {
        return Ok(s::CalcLength::percent(parse_number(percent)?));
    }
    Err(Error::new(format!(
        "unsupported calc expression term `{raw}`"
    )))
}

fn parse_style_length(raw: &str) -> Result<s::Length, Error> {
    if raw.trim_start().starts_with("calc(") {
        return Ok(s::Length::Calc(parse_style_calc_length(raw)?));
    }
    Ok(to_style_length(parse_length(raw)?))
}

fn parse_style_length_auto(raw: &str) -> Result<s::Length, Error> {
    if raw == "auto" {
        return Ok(s::Length::Auto);
    }
    parse_style_length(raw)
}

fn parse_style_dimension(raw: &str) -> Result<s::Length, Error> {
    match raw {
        "auto" => Ok(s::Length::Auto),
        "min-content" => Ok(s::Length::MinContent),
        "max-content" => Ok(s::Length::MaxContent),
        _ => {
            if raw.trim_start().starts_with("calc(") {
                return Ok(s::Length::Calc(parse_style_calc_length(raw)?));
            }
            to_style_dimension(parse_dimension(raw)?)
        }
    }
}

fn parse_length(raw: &str) -> Result<layout::Length, Error> {
    if let Some(px) = raw.strip_suffix("px") {
        return Ok(layout::Length::px(parse_number(px)?));
    }
    if let Some(percent) = raw.strip_suffix('%') {
        return Ok(layout::Length::percent(parse_number(percent)? / 100.0));
    }
    // Browser parity XML is a typed fixture format. Unitless fixture numbers
    // represent layout lengths; app-facing CSS parsing stays outside layout math.
    if let Ok(value) = parse_number(raw) {
        return Ok(layout::Length::px(value));
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

fn parse_track_component_list(raw: &str) -> Result<Vec<layout::TrackComponent>, Error> {
    if raw.trim_start().starts_with("subgrid") {
        return Ok(vec![parse_subgrid_track_component(raw)?]);
    }
    split_top_level_whitespace(raw)
        .into_iter()
        .map(|part| parse_track_component(&part))
        .collect()
}

fn parse_track_component(raw: &str) -> Result<layout::TrackComponent, Error> {
    if let Some(body) = function_body(raw, "repeat") {
        let (count, tracks) = split_once_top_level_comma(body)?;
        let repeat = match count.trim() {
            "auto-fill" => {
                layout::TrackRepetition::auto_fill_components(parse_track_component_list(tracks)?)
            }
            "auto-fit" => {
                layout::TrackRepetition::auto_fit_components(parse_track_component_list(tracks)?)
            }
            raw => layout::TrackRepetition::count_components(
                raw.parse()
                    .map_err(|_| Error::new(format!("invalid repeat count `{raw}`")))?,
                parse_track_component_list(tracks)?,
            ),
        };
        return Ok(layout::TrackComponent::Repeat(repeat));
    }
    if raw.starts_with('[') {
        return Ok(layout::TrackComponent::LineNames(parse_subgrid_line_names(
            raw,
        )?));
    }
    Ok(layout::TrackComponent::Track(parse_track_sizing(raw)?))
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

fn parse_track_sizing(raw: &str) -> Result<layout::TrackSizing, Error> {
    if let Some(body) = function_body(raw, "minmax") {
        let (min, max) = split_once_top_level_comma(body)?;
        return Ok(layout::TrackSizing::minmax(
            parse_min_track_sizing(min.trim())?,
            parse_max_track_sizing(max.trim())?,
        ));
    }
    if let Some(body) = function_body(raw, "fit-content") {
        return Ok(layout::TrackSizing::fit_content(parse_length(body.trim())?));
    }
    Ok(parse_dimension(raw)?.into())
}

fn parse_min_track_sizing(raw: &str) -> Result<layout::MinTrackSizing, Error> {
    match raw {
        "auto" => Ok(layout::MinTrackSizing::AUTO),
        "min-content" => Ok(layout::MinTrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::MinTrackSizing::MAX_CONTENT),
        _ => Ok(parse_length(raw)?.into()),
    }
}

fn parse_max_track_sizing(raw: &str) -> Result<layout::MaxTrackSizing, Error> {
    match raw {
        "auto" => Ok(layout::MaxTrackSizing::AUTO),
        "min-content" => Ok(layout::MaxTrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::MaxTrackSizing::MAX_CONTENT),
        _ if raw.ends_with("fr") => {
            let value = raw.trim_end_matches("fr");
            Ok(layout::MaxTrackSizing::fr(parse_number(value)?))
        }
        _ => Ok(parse_length(raw)?.into()),
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
        let mut lowering = s::adapters::layout::LayoutLoweringSession::new();
        to_node_input(&attrs, &mut lowering)
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
    fn parses_viewport_root_context_metadata() {
        let golden = Golden::parse(
            r#"
            <test name="viewport-flex-item" use-rounding="true">
                <viewport width="400px" height="max-content" root-context="flex-item" />
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

        assert_eq!(golden.viewport.root_context, RootContext::FlexItem);
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
            layout::Dimension::px(40.0)
        );
        assert_eq!(
            parse_dimension("0").expect("unitless zero fixture length should parse"),
            layout::Dimension::px(0.0)
        );
    }

    #[test]
    fn parse_length_rejects_non_fixture_css_units() {
        assert!(parse_length("1em").is_err());
        assert!(parse_length("calc(100% - 1px)").is_err());
    }

    #[test]
    fn parse_style_length_accepts_fixture_calc_px_plus_percent() {
        let length = parse_style_length("calc(12px + 25%)").expect("fixture calc should parse");
        assert!(matches!(length, s::Length::Calc(_)));
    }

    #[test]
    fn parse_style_dimension_accepts_fixture_calc_percent_minus_px() {
        let dimension =
            parse_style_dimension("calc(50% - 8px)").expect("fixture calc dimension should parse");
        assert!(matches!(dimension, s::Length::Calc(_)));
    }

    #[test]
    fn parse_style_length_rejects_unsupported_calc_fixture_syntax() {
        let error =
            parse_style_length("calc(100% / 2)").expect_err("division is not supported yet");
        assert!(
            error.to_string().contains("unsupported calc expression"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn to_node_input_lowers_calc_margin_with_fixture_store() {
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

        let layout::LengthAuto::Calc(id) = tree.nodes[0].node_input.margin.left else {
            panic!(
                "expected calc margin-left, got {:?}",
                tree.nodes[0].node_input.margin.left
            );
        };
        assert_eq!(
            layout::CalcResolver::resolve_calc(&tree.calc_store, id, Some(200.0)).value,
            Some(16.0)
        );
    }

    #[test]
    fn to_node_input_lowers_calc_grid_track_with_fixture_store() {
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
            tree.nodes[0].node_input.grid_template_columns.as_slice()
        else {
            panic!(
                "expected one grid track, got {:?}",
                tree.nodes[0].node_input.grid_template_columns
            );
        };
        assert!(matches!(
            track.min,
            layout::MinTrackSizing::Length(layout::Length::Calc(_))
        ));
        assert!(matches!(
            track.max,
            layout::MaxTrackSizing::Length(layout::Length::Calc(_))
        ));
        assert_eq!(tree.calc_store.len(), 2);
    }

    #[test]
    fn parse_track_component_list_accepts_rich_grid_tracks() {
        let tracks = parse_track_component_list(
            "40px minmax(20px,40px) fit-content(50%) repeat(2, 1fr auto)",
        )
        .expect("rich grid track list should parse");

        assert_eq!(tracks.len(), 4);
        assert_eq!(tracks[0], layout::TrackComponent::px(40.0));
        assert_eq!(
            tracks[1],
            layout::TrackComponent::minmax(
                layout::MinTrackSizing::px(20.0),
                layout::MaxTrackSizing::px(40.0)
            )
        );
        assert_eq!(
            tracks[2],
            layout::TrackComponent::fit_content(layout::Length::percent(0.5))
        );
        assert_eq!(
            tracks[3],
            layout::TrackComponent::Repeat(layout::TrackRepetition::count(
                2,
                vec![layout::TrackSizing::fr(1.0), layout::TrackSizing::AUTO]
            ))
        );
    }

    #[test]
    fn parse_track_component_list_accepts_auto_repeat() {
        assert_eq!(
            parse_track_component("repeat(auto-fill, minmax(150px,1fr))")
                .expect("auto-fill should parse"),
            layout::TrackComponent::Repeat(layout::TrackRepetition::auto_fill(vec![
                layout::TrackSizing::minmax(
                    layout::MinTrackSizing::px(150.0),
                    layout::MaxTrackSizing::fr(1.0)
                )
            ]))
        );
        assert_eq!(
            parse_track_component("repeat(auto-fit, 40px)").expect("auto-fit should parse"),
            layout::TrackComponent::Repeat(layout::TrackRepetition::auto_fit(vec![
                layout::TrackSizing::px(40.0)
            ]))
        );
    }

    #[test]
    fn parse_track_component_list_accepts_explicit_line_names() {
        let parsed = parse_style_track_component_list("[a] 10px [b c] 20px [d]")
            .expect("line names should parse");

        assert_eq!(
            parsed.components,
            vec![
                s::GridTrackComponent::LineNames(vec!["a".to_owned()]),
                s::GridTrackComponent::Track(s::TrackSizing::px(10.0)),
                s::GridTrackComponent::LineNames(vec!["b".to_owned(), "c".to_owned()]),
                s::GridTrackComponent::Track(s::TrackSizing::px(20.0)),
                s::GridTrackComponent::LineNames(vec!["d".to_owned()]),
            ]
        );
    }

    #[test]
    fn parse_track_component_list_rejects_reserved_line_names() {
        assert!(parse_style_track_component_list("[auto] 10px").is_err());
        assert!(parse_style_track_component_list("[span] 10px").is_err());
    }

    #[test]
    fn to_declarations_preserves_named_grid_syntax() {
        let declarations = to_declarations(&StyleAttrs {
            attrs: BTreeMap::from([
                (
                    "grid-template-columns".to_string(),
                    "[a] 10px [b]".to_string(),
                ),
                ("grid-column-start".to_string(), "a 2".to_string()),
            ]),
        })
        .expect("named grid syntax should parse to style declarations");

        assert_eq!(
            declarations.get(s::Property::GridColumnStart),
            Some(&s::Value::GridLine(s::GridLine::NamedLine {
                name: "a".to_owned(),
                index: 2,
            }))
        );
        assert_eq!(
            declarations.get(s::Property::GridTemplateColumns),
            Some(&s::Value::GridTrackList(s::GridTrackList::new(vec![
                s::GridTrackComponent::LineNames(vec!["a".to_owned()]),
                s::GridTrackComponent::Track(s::TrackSizing::px(10.0)),
                s::GridTrackComponent::LineNames(vec!["b".to_owned()]),
            ])))
        );
    }

    #[test]
    fn to_declarations_leaves_untagged_default_display_to_toolkit_default() {
        let declarations = to_declarations(&StyleAttrs {
            attrs: BTreeMap::new(),
        })
        .expect("empty attrs should parse");

        assert_eq!(declarations.get(s::Property::Display), None);
    }

    #[test]
    fn to_declarations_applies_html_source_tag_display_defaults() {
        let div = to_declarations(&StyleAttrs {
            attrs: BTreeMap::from([("source-tag".to_string(), "div".to_string())]),
        })
        .expect("source-tag div should parse");

        assert_eq!(
            div.get(s::Property::Display),
            Some(&s::Value::Display(s::Display::Block))
        );
    }

    #[test]
    fn source_tag_br_is_rejected_until_line_break_semantics_are_modeled() {
        let error = to_declarations(&StyleAttrs {
            attrs: BTreeMap::from([("source-tag".to_string(), "br".to_string())]),
        })
        .expect_err("source-tag br must not lower to an ordinary sized node");

        assert_eq!(
            error.to_string(),
            "unsupported source-tag `br`; line-break semantics are not represented"
        );
    }

    #[test]
    fn checked_fixture_enumerator_quarantines_unsupported_br_xml() {
        let fixtures = fixture_files("xml").expect("checked XML fixtures should load");
        let stale_br_fixture = Path::new(
            "crates/surgeist-layout/tests/layout/browser_parity/xml/subgrid/subgrid_baseline_vertical_nested_parent_row1_first__content_box_ltr.xml",
        );

        assert!(
            !fixtures.is_empty(),
            "expected checked XML fixtures before quarantine filtering"
        );
        assert!(
            fixtures.iter().all(|fixture| {
                !std::fs::read_to_string(fixture)
                    .expect("fixture should be readable")
                    .contains("source-tag=\"br\"")
            }),
            "source-tag=\"br\" fixtures must stay out of checked parity until line-break semantics are modeled"
        );
        assert!(
            !fixtures
                .iter()
                .any(|fixture| fixture.ends_with(stale_br_fixture)),
            "stale XML generated from source <br> fixtures must stay out of checked parity"
        );
    }

    #[test]
    fn to_declarations_preserves_grid_template_areas() {
        let declarations = to_declarations(&StyleAttrs {
            attrs: BTreeMap::from([(
                "grid-template-areas".to_string(),
                "head head / nav main".to_string(),
            )]),
        })
        .expect("grid template areas should parse to style declarations");

        assert_eq!(
            declarations.get(s::Property::GridTemplateAreas),
            Some(&s::Value::GridTemplateAreas(s::GridTemplateAreas::new([
                s::GridTemplateAreaRow::named(["head", "head"]),
                s::GridTemplateAreaRow::named(["nav", "main"]),
            ])))
        );
    }

    #[test]
    fn to_declarations_treats_grid_template_area_dot_runs_as_null_cells() {
        let declarations = to_declarations(&StyleAttrs {
            attrs: BTreeMap::from([(
                "grid-template-areas".to_string(),
                "... main / footer ...".to_string(),
            )]),
        })
        .expect("grid template areas should parse to style declarations");

        assert_eq!(
            declarations.get(s::Property::GridTemplateAreas),
            Some(&s::Value::GridTemplateAreas(s::GridTemplateAreas::new([
                s::GridTemplateAreaRow::new([None, Some("main".to_string())]),
                s::GridTemplateAreaRow::new([Some("footer".to_string()), None]),
            ])))
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
                node_input: layout::NodeInput {
                    display: layout::Display::InlineGrid,
                    grid_template_columns: vec![layout::TrackComponent::px(40.0)],
                    grid_template_rows: vec![layout::TrackComponent::px(20.0)],
                    ..layout::NodeInput::default()
                },
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
            calc_store: layout::LayoutCalcStore::new(),
        };

        layout::compute_root(
            &mut tree,
            0,
            layout::Size::splat(layout::Available::MaxContent),
        );
        layout::round_layout(&mut tree, 0);

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

        assert_eq!(node_input.grid_column, layout::GridPlacement::end_line(1));
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
        assert_eq!(node_input.margin.right, layout::LengthAuto::px(12.0));
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
            parse_style_grid_line("a").unwrap(),
            s::GridLine::BareIdent("a".to_owned())
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_line_with_occurrence() {
        assert_eq!(
            parse_style_grid_line("a 8").unwrap(),
            s::GridLine::NamedLine {
                name: "a".to_owned(),
                index: 8,
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_integer_before_named_line() {
        assert_eq!(
            parse_style_grid_line("2 a").unwrap(),
            s::GridLine::NamedLine {
                name: "a".to_owned(),
                index: 2,
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_negative_named_line_occurrence() {
        assert_eq!(
            parse_style_grid_line("b -1").unwrap(),
            s::GridLine::NamedLine {
                name: "b".to_owned(),
                index: -1,
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_span() {
        assert_eq!(
            parse_style_grid_line("span a").unwrap(),
            s::GridLine::NamedSpan {
                name: "a".to_owned(),
                index: 1,
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_span_with_count() {
        assert_eq!(
            parse_style_grid_line("span 2 a").unwrap(),
            s::GridLine::NamedSpan {
                name: "a".to_owned(),
                index: 2,
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_span_with_reversed_count_order() {
        assert_eq!(
            parse_style_grid_line("span a 2").unwrap(),
            s::GridLine::NamedSpan {
                name: "a".to_owned(),
                index: 2,
            }
        );
    }

    #[test]
    fn parse_grid_line_rejects_zero_named_line_occurrence() {
        assert!(parse_style_grid_line("a 0").is_err());
    }

    #[test]
    fn parse_grid_line_rejects_zero_named_span_count() {
        assert!(parse_style_grid_line("span 0 a").is_err());
        assert!(parse_style_grid_line("span a 0").is_err());
    }

    #[test]
    fn parse_grid_line_rejects_reserved_named_custom_ident() {
        assert!(parse_style_grid_line("auto 1").is_err());
        assert!(parse_style_grid_line("1 auto").is_err());
        assert!(parse_style_grid_line("span auto").is_err());
        assert!(parse_style_grid_line("span 1 auto").is_err());
    }

    #[test]
    fn parse_grid_start_line_accepts_span() {
        let mut node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("grid-column-start".to_string(), "span 2".to_string())]),
        })
        .expect("span start should parse");

        assert_eq!(node_input.grid_column.start, None);
        assert_eq!(node_input.grid_column.span, Some(2));

        node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("grid-row-start".to_string(), "span 3".to_string())]),
        })
        .expect("row span start should parse");

        assert_eq!(node_input.grid_row.start, None);
        assert_eq!(node_input.grid_row.span, Some(3));
    }
}
