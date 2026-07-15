use super::{
    AspectRatioOf, DefaultScalar, DimensionOf, Edges, GridLine, GridSpan, GridTemplateAreas,
    LayoutScalar, LengthAutoOf, LengthOf, NonNegativeFiniteScalarErrorOf, Point, Size,
    TrackComponentOf,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Display {
    Block,
    #[default]
    Flex,
    Grid,
    GridLanes,
    InlineBlock,
    InlineGrid,
    InlineGridLanes,
    None,
}

impl Display {
    #[must_use]
    pub const fn is_inline_level(self) -> bool {
        matches!(
            self,
            Self::InlineBlock | Self::InlineGrid | Self::InlineGridLanes
        )
    }

    /// Returns the display used to dispatch this box's own layout algorithm.
    ///
    /// This does not describe parent-flow participation. For example,
    /// `InlineBlock` participates in its parent as an atomic inline-level box,
    /// while its contents are laid out by the block formatting context.
    #[must_use]
    pub const fn inner_display(self) -> Self {
        match self {
            Self::InlineBlock => Self::Block,
            Self::InlineGrid => Self::Grid,
            Self::InlineGridLanes => Self::GridLanes,
            display => display,
        }
    }

    #[must_use]
    pub const fn establishes_grid_formatting_context(self) -> bool {
        matches!(self.inner_display(), Self::Grid | Self::GridLanes)
    }

    #[must_use]
    pub const fn establishes_grid_lanes_formatting_context(self) -> bool {
        matches!(self.inner_display(), Self::GridLanes)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineBreakDisplay {
    #[default]
    Break,
    None,
}

impl LineBreakDisplay {
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BoxSizing {
    ContentBox,
    #[default]
    BorderBox,
}

/// The already-resolved used inline direction for layout input.
///
/// This is not an authored CSS `direction` token. Root style and text
/// integration resolve authored direction effects before constructing a layout
/// input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

impl Direction {
    #[must_use]
    pub const fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
    Hidden,
    Scroll,
}

impl Overflow {
    #[must_use]
    pub const fn clips_contents(self) -> bool {
        matches!(self, Self::Clip | Self::Hidden | Self::Scroll)
    }

    #[must_use]
    pub const fn is_scrollable(self) -> bool {
        matches!(self, Self::Scroll)
    }

    #[must_use]
    pub const fn blocks_margin_collapse(self) -> bool {
        matches!(self, Self::Hidden | Self::Scroll)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Position {
    #[default]
    Relative,
    Absolute,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Float {
    #[default]
    None,
    Left,
    Right,
}

impl Float {
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Clear {
    #[default]
    None,
    Left,
    Right,
    Both,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAlign {
    #[default]
    Auto,
    LegacyLeft,
    LegacyRight,
    LegacyCenter,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Top,
}

/// The writing-mode state supplied to layout.
///
/// The five supported values are `HorizontalTb`, `VerticalRl`, `VerticalLr`,
/// `SidewaysRl`, and `SidewaysLr`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}

impl WritingMode {
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        !matches!(self, Self::HorizontalTb)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineMetricsOf<S: LayoutScalar = DefaultScalar> {
    baseline: S,
    line_extent: S,
}

pub type InlineMetrics = InlineMetricsOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InlineMetricsError<S: LayoutScalar = DefaultScalar> {
    NonFinite { value: S },
    Negative { value: S },
    BaselineExceedsLineExtent { baseline: S, line_extent: S },
    BaselineExceedsLineHeight { baseline: S, line_height: S },
}

impl<S: LayoutScalar> InlineMetricsOf<S> {
    pub fn try_new(baseline: S, line_extent: S) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(baseline)?;
        validate_non_negative_finite(line_extent)?;

        if baseline > line_extent {
            return Err(InlineMetricsError::BaselineExceedsLineExtent {
                baseline,
                line_extent,
            });
        }

        Ok(Self {
            baseline,
            line_extent,
        })
    }

    pub fn from_ascent_descent(ascent: S, descent: S) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(ascent)?;
        validate_non_negative_finite(descent)?;
        Self::try_new(ascent, ascent + descent)
    }

    pub fn from_line_height_and_baseline(
        line_height: S,
        baseline: S,
    ) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(line_height)?;
        validate_non_negative_finite(baseline)?;

        if baseline > line_height {
            return Err(InlineMetricsError::BaselineExceedsLineHeight {
                baseline,
                line_height,
            });
        }

        Ok(Self {
            baseline,
            line_extent: line_height,
        })
    }

    #[must_use]
    pub const fn baseline(self) -> S {
        self.baseline
    }

    #[must_use]
    pub const fn line_extent(self) -> S {
        self.line_extent
    }

    #[must_use]
    pub fn after_baseline(self) -> S {
        self.line_extent - self.baseline
    }
}

impl<S: LayoutScalar> Default for InlineMetricsOf<S> {
    fn default() -> Self {
        Self::from_line_height_and_baseline(S::from_f64(16.0), S::from_f64(12.0))
            .expect("default inline metrics are valid")
    }
}

fn validate_non_negative_finite<S: LayoutScalar>(value: S) -> Result<(), InlineMetricsError<S>> {
    if !value.is_finite() {
        return Err(InlineMetricsError::NonFinite { value });
    }
    if value < S::ZERO {
        return Err(InlineMetricsError::Negative { value });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineBreakInputOf<S: LayoutScalar = DefaultScalar> {
    display: LineBreakDisplay,
    direction: Direction,
    writing_mode: WritingMode,
    vertical_align: VerticalAlign,
    clear: Clear,
    metrics: InlineMetricsOf<S>,
}

pub type LineBreakInput = LineBreakInputOf<DefaultScalar>;

impl<S: LayoutScalar> LineBreakInputOf<S> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_metrics(mut self, metrics: InlineMetricsOf<S>) -> Self {
        self.metrics = metrics;
        self
    }

    #[must_use]
    pub const fn hidden(mut self) -> Self {
        self.display = LineBreakDisplay::None;
        self
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn with_clear(mut self, clear: Clear) -> Self {
        self.clear = clear;
        self
    }

    #[must_use]
    pub const fn display(self) -> LineBreakDisplay {
        self.display
    }

    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn clear(self) -> Clear {
        self.clear
    }

    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}

impl<S: LayoutScalar> Default for LineBreakInputOf<S> {
    fn default() -> Self {
        Self {
            display: LineBreakDisplay::Break,
            direction: Direction::Ltr,
            writing_mode: WritingMode::HorizontalTb,
            vertical_align: VerticalAlign::Baseline,
            clear: Clear::None,
            metrics: InlineMetricsOf::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBoundaryKind {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoundaryInputOf<S: LayoutScalar = DefaultScalar> {
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    vertical_align: VerticalAlign,
    metrics: InlineMetricsOf<S>,
}

pub type InlineBoundaryInput = InlineBoundaryInputOf<DefaultScalar>;

impl<S: LayoutScalar> InlineBoundaryInputOf<S> {
    #[must_use]
    pub const fn new(kind: InlineBoundaryKind, metrics: InlineMetricsOf<S>) -> Self {
        Self {
            kind,
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            vertical_align: VerticalAlign::Baseline,
            metrics,
        }
    }

    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoundaryKind {
        self.kind
    }

    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SafeEnd,
    SafeFlexEnd,
    SafeCenter,
    Baseline,
    LastBaseline,
    Stretch,
}

impl AlignItems {
    #[must_use]
    pub const fn unsafe_position(self) -> Self {
        match self {
            Self::SafeEnd => Self::End,
            Self::SafeFlexEnd => Self::FlexEnd,
            Self::SafeCenter => Self::Center,
            Self::Baseline | Self::LastBaseline => self,
            position => position,
        }
    }

    /// Applies CSS safe alignment fallback for any layout scalar lane.
    ///
    /// This is intentionally non-const because generic scalar comparison is
    /// provided through the `LayoutScalar` contract.
    #[must_use]
    pub fn safe_fallback<S: LayoutScalar>(self, free_space: S) -> Self {
        if free_space < S::ZERO {
            match self {
                Self::SafeEnd | Self::SafeFlexEnd | Self::SafeCenter => Self::Start,
                position => position.unsafe_position(),
            }
        } else {
            self.unsafe_position()
        }
    }
}

pub type AlignSelf = AlignItems;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SafeEnd,
    SafeFlexEnd,
    SafeCenter,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

impl AlignContent {
    #[must_use]
    pub const fn reversed(self) -> Self {
        match self {
            Self::Start => Self::End,
            Self::End => Self::Start,
            Self::FlexStart => Self::FlexEnd,
            Self::FlexEnd => Self::FlexStart,
            Self::SafeEnd => Self::Start,
            Self::SafeFlexEnd => Self::FlexStart,
            Self::Stretch => Self::End,
            style => style,
        }
    }

    #[must_use]
    pub const fn unsafe_position(self) -> Self {
        match self {
            Self::SafeEnd => Self::End,
            Self::SafeFlexEnd => Self::FlexEnd,
            Self::SafeCenter => Self::Center,
            position => position,
        }
    }

    /// Applies CSS safe alignment fallback for any layout scalar lane.
    ///
    /// This is intentionally non-const because generic scalar comparison is
    /// provided through the `LayoutScalar` contract.
    #[must_use]
    pub fn safe_fallback<S: LayoutScalar>(self, free_space: S) -> Self {
        if free_space < S::ZERO {
            match self {
                Self::SafeEnd | Self::SafeFlexEnd | Self::SafeCenter => Self::Start,
                position => position.unsafe_position(),
            }
        } else {
            self.unsafe_position()
        }
    }
}

pub type JustifyContent = AlignContent;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Flex direction selects a container-local logical main axis.
///
/// Physical-axis selection requires the container's resolved flow and is owned
/// by the crate-private flex algorithm.
///
/// ```compile_fail
/// use surgeist_layout::{FlexDirection, PhysicalAxis};
/// let _: PhysicalAxis = FlexDirection::Row.main_axis();
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{FlexDirection, PhysicalAxis};
/// let _: PhysicalAxis = FlexDirection::Column.cross_axis();
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl FlexDirection {
    #[must_use]
    pub const fn is_row(self) -> bool {
        matches!(self, Self::Row | Self::RowReverse)
    }

    #[must_use]
    pub const fn is_column(self) -> bool {
        matches!(self, Self::Column | Self::ColumnReverse)
    }

    #[must_use]
    pub const fn is_reverse(self) -> bool {
        matches!(self, Self::RowReverse | Self::ColumnReverse)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GridAutoFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridFlowToleranceOf<S: LayoutScalar = DefaultScalar> {
    Normal { font_size: S },
    Length(LengthOf<S>),
    Percent(S),
    Infinite,
}

pub type GridFlowTolerance = GridFlowToleranceOf<DefaultScalar>;

impl<S: LayoutScalar> Default for GridFlowToleranceOf<S> {
    fn default() -> Self {
        Self::Normal {
            font_size: S::from_usize(16),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GridPlacement {
    start: Option<GridLine>,
    end: Option<GridLine>,
    span: Option<GridSpan>,
}

impl GridPlacement {
    pub const AUTO: Self = Self {
        start: None,
        end: None,
        span: None,
    };

    #[must_use]
    pub const fn line(line: GridLine) -> Self {
        Self {
            start: Some(line),
            end: None,
            span: None,
        }
    }

    #[must_use]
    pub const fn try_line(line: isize) -> Option<Self> {
        match GridLine::new(line) {
            Some(line) => Some(Self::line(line)),
            None => None,
        }
    }

    #[must_use]
    pub const fn lines(start: GridLine, end: GridLine) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
            span: None,
        }
    }

    #[must_use]
    pub const fn try_lines(start: isize, end: isize) -> Option<Self> {
        match (GridLine::new(start), GridLine::new(end)) {
            (Some(start), Some(end)) => Some(Self::lines(start, end)),
            _ => None,
        }
    }

    #[must_use]
    pub const fn end_line(line: GridLine) -> Self {
        Self {
            start: None,
            end: Some(line),
            span: None,
        }
    }

    #[must_use]
    pub const fn try_end_line(line: isize) -> Option<Self> {
        match GridLine::new(line) {
            Some(line) => Some(Self::end_line(line)),
            None => None,
        }
    }

    #[must_use]
    pub const fn line_span(line: GridLine, span: GridSpan) -> Self {
        Self {
            start: Some(line),
            end: None,
            span: Some(span),
        }
    }

    #[must_use]
    pub const fn try_line_span(line: isize, span: usize) -> Option<Self> {
        match (GridLine::new(line), GridSpan::new(span)) {
            (Some(line), Some(span)) => Some(Self::line_span(line, span)),
            _ => None,
        }
    }

    #[must_use]
    pub const fn span_line(span: GridSpan, line: GridLine) -> Self {
        Self {
            start: None,
            end: Some(line),
            span: Some(span),
        }
    }

    #[must_use]
    pub const fn try_span_line(span: usize, line: isize) -> Option<Self> {
        match (GridSpan::new(span), GridLine::new(line)) {
            (Some(span), Some(line)) => Some(Self::span_line(span, line)),
            _ => None,
        }
    }

    #[must_use]
    pub const fn try_span(span: usize) -> Option<Self> {
        match GridSpan::new(span) {
            Some(span) => Some(Self {
                start: None,
                end: None,
                span: Some(span),
            }),
            None => None,
        }
    }

    #[must_use]
    pub const fn is_auto(self) -> bool {
        self.start.is_none() && self.end.is_none() && self.span.is_none()
    }

    #[must_use]
    pub const fn start(self) -> Option<GridLine> {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> Option<GridLine> {
        self.end
    }

    #[must_use]
    pub const fn span(self) -> Option<GridSpan> {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RawGridLine {
    Auto,
    Line(isize),
    Span(usize),
    BareIdent(String),
    NamedLine { name: String, index: isize },
    NamedSpan { name: String, index: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawGridPlacement {
    pub start: RawGridLine,
    pub end: RawGridLine,
}

impl RawGridPlacement {
    pub const AUTO: Self = Self {
        start: RawGridLine::Auto,
        end: RawGridLine::Auto,
    };

    #[must_use]
    pub const fn new(start: RawGridLine, end: RawGridLine) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn line(line: isize) -> Self {
        Self::new(RawGridLine::Line(line), RawGridLine::Auto)
    }

    #[must_use]
    pub const fn lines(start: isize, end: isize) -> Self {
        Self::new(RawGridLine::Line(start), RawGridLine::Line(end))
    }

    #[must_use]
    pub const fn span(span: usize) -> Self {
        Self::new(RawGridLine::Auto, RawGridLine::Span(span))
    }
}

impl Default for RawGridPlacement {
    fn default() -> Self {
        Self::AUTO
    }
}

impl GridAutoFlow {
    #[must_use]
    pub const fn is_column(self) -> bool {
        matches!(self, Self::Column | Self::ColumnDense)
    }

    #[must_use]
    pub const fn is_dense(self) -> bool {
        matches!(self, Self::RowDense | Self::ColumnDense)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarWidthOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

pub type ScrollbarWidth = ScrollbarWidthOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollbarWidthOf<S> {
    pub const ZERO: Self = Self { value: S::ZERO };

    pub fn try_new(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self {
            value: validate_numeric_property(value)?,
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }
}

impl<S: LayoutScalar> Default for ScrollbarWidthOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexGrowOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

pub type FlexGrow = FlexGrowOf<DefaultScalar>;

impl<S: LayoutScalar> FlexGrowOf<S> {
    pub const ZERO: Self = Self { value: S::ZERO };

    pub fn try_new(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self {
            value: validate_numeric_property(value)?,
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }
}

impl<S: LayoutScalar> Default for FlexGrowOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexShrinkOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

pub type FlexShrink = FlexShrinkOf<DefaultScalar>;

impl<S: LayoutScalar> FlexShrinkOf<S> {
    pub const ONE: Self = Self { value: S::ONE };

    pub fn try_new(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self {
            value: validate_numeric_property(value)?,
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }
}

impl<S: LayoutScalar> Default for FlexShrinkOf<S> {
    fn default() -> Self {
        Self::ONE
    }
}

fn validate_numeric_property<S: LayoutScalar>(
    value: S,
) -> Result<S, NonNegativeFiniteScalarErrorOf<S>> {
    if !value.is_finite() {
        return Err(NonNegativeFiniteScalarErrorOf::NonFinite { value });
    }

    if value < S::ZERO {
        return Err(NonNegativeFiniteScalarErrorOf::Negative { value });
    }

    Ok(if value == S::ZERO { S::ZERO } else { value })
}

/// A layout item's signed order value, independent of its source identity.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemOrder(i32);

impl ItemOrder {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

pub(crate) fn item_order_permutation(
    items: &[(ItemOrder, crate::SourceIndex)],
) -> Vec<crate::SourceIndex> {
    let mut ordered = items.to_vec();
    ordered.sort_by_key(|&(item_order, source_index)| (item_order, source_index));
    ordered
        .into_iter()
        .map(|(_, source_index)| source_index)
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeInputOf<S: LayoutScalar = DefaultScalar> {
    pub display: Display,
    pub item_is_table: bool,
    pub item_is_replaced: bool,
    pub item_order: ItemOrder,
    pub box_sizing: BoxSizing,
    pub direction: Direction,
    pub text_align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub writing_mode: WritingMode,
    pub overflow: Point<Overflow>,
    pub scrollbar_width: self::ScrollbarWidthOf<S>,
    pub position: Position,
    pub float: Float,
    pub clear: Clear,
    pub inset: Edges<LengthAutoOf<S>>,
    pub size: Size<DimensionOf<S>>,
    pub min_size: Size<DimensionOf<S>>,
    pub max_size: Size<DimensionOf<S>>,
    pub aspect_ratio: Option<AspectRatioOf<S>>,
    pub margin: Edges<LengthAutoOf<S>>,
    pub padding: Edges<LengthOf<S>>,
    pub border: Edges<LengthOf<S>>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub justify_items: Option<AlignItems>,
    pub justify_self: Option<AlignSelf>,
    pub align_content: Option<AlignContent>,
    pub justify_content: Option<JustifyContent>,
    pub gap: Size<LengthOf<S>>,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_basis: DimensionOf<S>,
    pub flex_grow: FlexGrowOf<S>,
    pub flex_shrink: FlexShrinkOf<S>,
    pub grid_template_columns: Vec<TrackComponentOf<S>>,
    pub grid_template_rows: Vec<TrackComponentOf<S>>,
    pub grid_template_areas: GridTemplateAreas,
    pub grid_auto_columns: Vec<TrackComponentOf<S>>,
    pub grid_auto_rows: Vec<TrackComponentOf<S>>,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_flow_tolerance: GridFlowToleranceOf<S>,
    pub grid_column: GridPlacement,
    pub grid_row: GridPlacement,
    pub raw_grid_column: RawGridPlacement,
    pub raw_grid_row: RawGridPlacement,
}

pub type NodeInput = NodeInputOf<DefaultScalar>;

impl NodeInputOf<DefaultScalar> {
    pub const DEFAULT: Self = Self {
        display: Display::Flex,
        item_is_table: false,
        item_is_replaced: false,
        item_order: ItemOrder::ZERO,
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Ltr,
        text_align: TextAlign::Auto,
        vertical_align: VerticalAlign::Baseline,
        writing_mode: WritingMode::HorizontalTb,
        overflow: Point {
            x: Overflow::Visible,
            y: Overflow::Visible,
        },
        scrollbar_width: self::ScrollbarWidthOf::ZERO,
        position: Position::Relative,
        float: Float::None,
        clear: Clear::None,
        inset: Edges::all(LengthAutoOf::AUTO),
        size: Size::new(DimensionOf::AUTO, DimensionOf::AUTO),
        min_size: Size::new(DimensionOf::AUTO, DimensionOf::AUTO),
        max_size: Size::new(DimensionOf::AUTO, DimensionOf::AUTO),
        aspect_ratio: None,
        margin: Edges::all(LengthAutoOf::ZERO),
        padding: Edges::all(LengthOf::ZERO),
        border: Edges::all(LengthOf::ZERO),
        align_items: None,
        align_self: None,
        justify_items: None,
        justify_self: None,
        align_content: None,
        justify_content: None,
        gap: Size::new(LengthOf::NORMAL, LengthOf::NORMAL),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        flex_basis: DimensionOf::AUTO,
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::ONE,
        grid_template_columns: Vec::new(),
        grid_template_rows: Vec::new(),
        grid_template_areas: GridTemplateAreas { rows: Vec::new() },
        grid_auto_columns: Vec::new(),
        grid_auto_rows: Vec::new(),
        grid_auto_flow: GridAutoFlow::Row,
        grid_flow_tolerance: GridFlowToleranceOf::Normal { font_size: 16.0 },
        grid_column: GridPlacement::AUTO,
        grid_row: GridPlacement::AUTO,
        raw_grid_column: RawGridPlacement::AUTO,
        raw_grid_row: RawGridPlacement::AUTO,
    };
}

impl<S: LayoutScalar> Default for NodeInputOf<S> {
    fn default() -> Self {
        Self {
            display: Display::Flex,
            item_is_table: false,
            item_is_replaced: false,
            item_order: ItemOrder::ZERO,
            box_sizing: BoxSizing::BorderBox,
            direction: Direction::Ltr,
            text_align: TextAlign::Auto,
            vertical_align: VerticalAlign::Baseline,
            writing_mode: WritingMode::HorizontalTb,
            overflow: Point {
                x: Overflow::Visible,
                y: Overflow::Visible,
            },
            scrollbar_width: self::ScrollbarWidthOf::ZERO,
            position: Position::Relative,
            float: Float::None,
            clear: Clear::None,
            inset: Edges::all(LengthAutoOf::AUTO),
            size: Size::new(DimensionOf::AUTO, DimensionOf::AUTO),
            min_size: Size::new(DimensionOf::AUTO, DimensionOf::AUTO),
            max_size: Size::new(DimensionOf::AUTO, DimensionOf::AUTO),
            aspect_ratio: None,
            margin: Edges::all(LengthAutoOf::ZERO),
            padding: Edges::all(LengthOf::ZERO),
            border: Edges::all(LengthOf::ZERO),
            align_items: None,
            align_self: None,
            justify_items: None,
            justify_self: None,
            align_content: None,
            justify_content: None,
            gap: Size::new(LengthOf::NORMAL, LengthOf::NORMAL),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            flex_basis: DimensionOf::AUTO,
            flex_grow: FlexGrowOf::ZERO,
            flex_shrink: FlexShrinkOf::ONE,
            grid_template_columns: Vec::new(),
            grid_template_rows: Vec::new(),
            grid_template_areas: GridTemplateAreas { rows: Vec::new() },
            grid_auto_columns: Vec::new(),
            grid_auto_rows: Vec::new(),
            grid_auto_flow: GridAutoFlow::Row,
            grid_flow_tolerance: GridFlowToleranceOf::default(),
            grid_column: GridPlacement::AUTO,
            grid_row: GridPlacement::AUTO,
            raw_grid_column: RawGridPlacement::AUTO,
            raw_grid_row: RawGridPlacement::AUTO,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LayoutInputOf<S: LayoutScalar = DefaultScalar> {
    Box(std::boxed::Box<NodeInputOf<S>>),
    LineBreak(LineBreakInputOf<S>),
    InlineBoundary(InlineBoundaryInputOf<S>),
}

pub type LayoutInput = LayoutInputOf<DefaultScalar>;

impl<S: LayoutScalar> LayoutInputOf<S> {
    #[must_use]
    pub fn box_input(input: NodeInputOf<S>) -> Self {
        Self::Box(std::boxed::Box::new(input))
    }

    #[must_use]
    pub const fn line_break(input: LineBreakInputOf<S>) -> Self {
        Self::LineBreak(input)
    }

    #[must_use]
    pub const fn inline_boundary(input: InlineBoundaryInputOf<S>) -> Self {
        Self::InlineBoundary(input)
    }

    #[must_use]
    pub fn as_box(&self) -> Option<&NodeInputOf<S>> {
        match self {
            Self::Box(input) => Some(input.as_ref()),
            Self::LineBreak(_) | Self::InlineBoundary(_) => None,
        }
    }

    #[must_use]
    pub const fn as_line_break(&self) -> Option<LineBreakInputOf<S>> {
        match self {
            Self::Box(_) | Self::InlineBoundary(_) => None,
            Self::LineBreak(input) => Some(*input),
        }
    }

    #[must_use]
    pub const fn as_inline_boundary(&self) -> Option<InlineBoundaryInputOf<S>> {
        match self {
            Self::Box(_) | Self::LineBreak(_) => None,
            Self::InlineBoundary(input) => Some(*input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceIndex;

    #[test]
    fn item_order_permutation_is_signed_total_and_stable() {
        let items = [
            (ItemOrder::ZERO, SourceIndex::new(4)),
            (ItemOrder::new(-1), SourceIndex::new(3)),
            (ItemOrder::new(1), SourceIndex::new(2)),
            (ItemOrder::new(-1), SourceIndex::new(1)),
            (ItemOrder::ZERO, SourceIndex::new(0)),
            (ItemOrder::new(i32::MIN), SourceIndex::new(6)),
            (ItemOrder::new(i32::MAX), SourceIndex::new(5)),
        ];
        assert_eq!(
            item_order_permutation(&items),
            [6, 1, 3, 0, 4, 2, 5].map(SourceIndex::new)
        );

        let all_zero = [
            (ItemOrder::ZERO, SourceIndex::new(2)),
            (ItemOrder::default(), SourceIndex::new(0)),
            (ItemOrder::ZERO, SourceIndex::new(1)),
        ];
        assert_eq!(
            item_order_permutation(&all_zero),
            [0, 1, 2].map(SourceIndex::new)
        );
        assert_eq!(item_order_permutation(&[]), Vec::new());
    }
}
