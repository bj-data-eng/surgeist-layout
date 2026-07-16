use super::{
    AspectRatioOf, DefaultScalar, Edges, FiniteScalarErrorOf, FlexBasisOf, GridLine, GridSpan,
    GridTemplateAreas, LayoutScalar, LengthAutoOf, LengthOf, LengthPercentageOf, MaxSizeOf,
    MinSizeOf, NonNegativeFiniteScalarErrorOf, NumericResolutionOf, PercentageBasisOf,
    PhysicalSide, PreferredSizeOf, Size, TrackComponentOf,
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
    Auto,
}

impl Overflow {
    #[must_use]
    pub const fn is_scrollable(self) -> bool {
        matches!(self, Self::Hidden | Self::Scroll | Self::Auto)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComputedOverflow {
    x: Overflow,
    y: Overflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComputedOverflowError {
    NonCanonicalPair { x: Overflow, y: Overflow },
}

impl core::fmt::Display for ComputedOverflowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonCanonicalPair { .. } => f.write_str(
                "computed overflow axes must both be visible/clip or both be hidden/scroll/auto",
            ),
        }
    }
}

impl std::error::Error for ComputedOverflowError {}

impl ComputedOverflow {
    pub const VISIBLE: Self = Self {
        x: Overflow::Visible,
        y: Overflow::Visible,
    };

    pub fn try_new(x: Overflow, y: Overflow) -> Result<Self, ComputedOverflowError> {
        if x.is_scrollable() == y.is_scrollable() {
            Ok(Self { x, y })
        } else {
            Err(ComputedOverflowError::NonCanonicalPair { x, y })
        }
    }

    #[must_use]
    pub const fn x(self) -> Overflow {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> Overflow {
        self.y
    }

    #[must_use]
    pub const fn establishes_independent_formatting_context(self) -> bool {
        self.x.is_scrollable() && self.y.is_scrollable()
    }
}

impl Default for ComputedOverflow {
    fn default() -> Self {
        Self::VISIBLE
    }
}

/// Reference box used by the overflow clip edge.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverflowClipBox {
    ContentBox,
    #[default]
    PaddingBox,
    BorderBox,
}

/// A validated overflow clip reference box and absolute margin.
///
/// The margin is finite and non-negative. Signed zero is normalized to the
/// scalar lane's canonical zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverflowClipMarginOf<S: LayoutScalar = DefaultScalar> {
    clip_box: OverflowClipBox,
    margin: S,
}

/// Default-scalar overflow clip margin.
pub type OverflowClipMargin = OverflowClipMarginOf<DefaultScalar>;

impl<S: LayoutScalar> OverflowClipMarginOf<S> {
    /// Constructs an overflow clip margin atomically.
    pub fn try_new(
        clip_box: OverflowClipBox,
        margin: S,
    ) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self {
            clip_box,
            margin: validate_numeric_property(margin)?,
        })
    }

    /// Returns the overflow clip reference box.
    #[must_use]
    pub const fn clip_box(self) -> OverflowClipBox {
        self.clip_box
    }

    /// Returns the finite non-negative absolute margin.
    #[must_use]
    pub const fn margin(self) -> S {
        self.margin
    }
}

impl<S: LayoutScalar> Default for OverflowClipMarginOf<S> {
    fn default() -> Self {
        Self {
            clip_box: OverflowClipBox::PaddingBox,
            margin: S::ZERO,
        }
    }
}

/// Classic scrollbar gutter reservation policy supplied to layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarGutter {
    #[default]
    Auto,
    Stable,
    StableBothEdges,
}

/// One physical scroll-padding edge in normalized layout input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollPaddingValueOf<S: LayoutScalar = DefaultScalar> {
    Value(LengthPercentageOf<S>),
    Auto,
}

/// Default-scalar scroll-padding edge value.
pub type ScrollPaddingValue = ScrollPaddingValueOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollPaddingValueOf<S> {
    /// The CSS initial `auto` value.
    pub const AUTO: Self = Self::Auto;

    /// Wraps an intrinsically validated length-percentage value.
    #[must_use]
    pub const fn value(value: LengthPercentageOf<S>) -> Self {
        Self::Value(value)
    }

    /// Constructs the CSS initial `auto` value.
    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    /// Returns whether this edge retains the `auto` value.
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Resolves this edge against its corresponding physical dimension.
    ///
    /// Callers pass scrollport width for left/right and scrollport height for
    /// top/bottom. Product `auto` resolves to zero. A negative numeric result
    /// is clamped to canonical zero, while missing-basis and invalid-numeric
    /// outcomes retain the underlying length-percentage diagnostic.
    #[must_use]
    pub fn resolve_against(self, basis: PercentageBasisOf<S>) -> NumericResolutionOf<S> {
        let resolution = match self {
            Self::Value(value) => value.resolve_against(basis),
            Self::Auto => NumericResolutionOf::Resolved(S::ZERO),
        };

        match resolution {
            NumericResolutionOf::Resolved(value) if value < S::ZERO => {
                NumericResolutionOf::Resolved(S::ZERO)
            }
            NumericResolutionOf::Resolved(value) if value == S::ZERO => {
                NumericResolutionOf::Resolved(S::ZERO)
            }
            resolution => resolution,
        }
    }
}

impl<S: LayoutScalar> Default for ScrollPaddingValueOf<S> {
    fn default() -> Self {
        Self::AUTO
    }
}

/// Four normalized physical scroll-padding edges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollPaddingOf<S: LayoutScalar = DefaultScalar> {
    top: ScrollPaddingValueOf<S>,
    right: ScrollPaddingValueOf<S>,
    bottom: ScrollPaddingValueOf<S>,
    left: ScrollPaddingValueOf<S>,
}

/// Default-scalar physical scroll padding.
pub type ScrollPadding = ScrollPaddingOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollPaddingOf<S> {
    /// Constructs four physical edges in top/right/bottom/left order.
    #[must_use]
    pub const fn new(
        top: ScrollPaddingValueOf<S>,
        right: ScrollPaddingValueOf<S>,
        bottom: ScrollPaddingValueOf<S>,
        left: ScrollPaddingValueOf<S>,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub const fn top(self) -> ScrollPaddingValueOf<S> {
        self.top
    }

    #[must_use]
    pub const fn right(self) -> ScrollPaddingValueOf<S> {
        self.right
    }

    #[must_use]
    pub const fn bottom(self) -> ScrollPaddingValueOf<S> {
        self.bottom
    }

    #[must_use]
    pub const fn left(self) -> ScrollPaddingValueOf<S> {
        self.left
    }
}

impl<S: LayoutScalar> Default for ScrollPaddingOf<S> {
    fn default() -> Self {
        Self::new(
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::AUTO,
        )
    }
}

/// Atomic scroll-margin construction failure for one physical edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollMarginErrorOf<S: LayoutScalar = DefaultScalar> {
    edge: PhysicalSide,
    source: FiniteScalarErrorOf<S>,
}

/// Default-scalar scroll-margin construction error.
pub type ScrollMarginError = ScrollMarginErrorOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollMarginErrorOf<S> {
    /// Returns the exact rejected physical edge.
    #[must_use]
    pub const fn edge(&self) -> PhysicalSide {
        self.edge
    }

    /// Returns the preserved scalar validation error.
    #[must_use]
    pub const fn error(&self) -> FiniteScalarErrorOf<S> {
        self.source
    }
}

impl<S: LayoutScalar> core::fmt::Display for ScrollMarginErrorOf<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let edge = match self.edge {
            PhysicalSide::Top => "top",
            PhysicalSide::Right => "right",
            PhysicalSide::Bottom => "bottom",
            PhysicalSide::Left => "left",
        };
        write!(f, "scroll margin {edge} edge must be finite")
    }
}

impl<S: LayoutScalar> std::error::Error for ScrollMarginErrorOf<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Four finite signed absolute physical scroll-margin outsets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollMarginOf<S: LayoutScalar = DefaultScalar> {
    top: S,
    right: S,
    bottom: S,
    left: S,
}

/// Default-scalar physical scroll margin.
pub type ScrollMargin = ScrollMarginOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollMarginOf<S> {
    /// Constructs all four physical edges atomically.
    pub fn try_new(top: S, right: S, bottom: S, left: S) -> Result<Self, ScrollMarginErrorOf<S>> {
        Ok(Self {
            top: validate_scroll_margin_edge(PhysicalSide::Top, top)?,
            right: validate_scroll_margin_edge(PhysicalSide::Right, right)?,
            bottom: validate_scroll_margin_edge(PhysicalSide::Bottom, bottom)?,
            left: validate_scroll_margin_edge(PhysicalSide::Left, left)?,
        })
    }

    #[must_use]
    pub const fn top(self) -> S {
        self.top
    }

    #[must_use]
    pub const fn right(self) -> S {
        self.right
    }

    #[must_use]
    pub const fn bottom(self) -> S {
        self.bottom
    }

    #[must_use]
    pub const fn left(self) -> S {
        self.left
    }
}

impl<S: LayoutScalar> Default for ScrollMarginOf<S> {
    fn default() -> Self {
        Self {
            top: S::ZERO,
            right: S::ZERO,
            bottom: S::ZERO,
            left: S::ZERO,
        }
    }
}

fn validate_scroll_margin_edge<S: LayoutScalar>(
    edge: PhysicalSide,
    value: S,
) -> Result<S, ScrollMarginErrorOf<S>> {
    if !value.is_finite() {
        return Err(ScrollMarginErrorOf {
            edge,
            source: FiniteScalarErrorOf::NonFinite { value },
        });
    }

    Ok(if value == S::ZERO { S::ZERO } else { value })
}

/// Axis selected by an enabled scroll snap type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollSnapAxis {
    X,
    Y,
    Block,
    Inline,
    Both,
}

/// Strictness selected by an enabled scroll snap type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollSnapStrictness {
    Proximity,
    Mandatory,
}

/// Normalized scroll snap behavior for a scroll container.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollSnapType {
    #[default]
    None,
    Enabled {
        axis: ScrollSnapAxis,
        strictness: ScrollSnapStrictness,
    },
}

/// One semantic block- or inline-axis snap alignment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollSnapAlignValue {
    #[default]
    None,
    Start,
    End,
    Center,
}

/// Explicit semantic block and inline snap alignments for one target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollSnapAlign {
    block: ScrollSnapAlignValue,
    inline: ScrollSnapAlignValue,
}

impl ScrollSnapAlign {
    /// Constructs semantic block and inline values without physical mapping.
    #[must_use]
    pub const fn new(block: ScrollSnapAlignValue, inline: ScrollSnapAlignValue) -> Self {
        Self { block, inline }
    }

    #[must_use]
    pub const fn block(self) -> ScrollSnapAlignValue {
        self.block
    }

    #[must_use]
    pub const fn inline(self) -> ScrollSnapAlignValue {
        self.inline
    }
}

impl Default for ScrollSnapAlign {
    fn default() -> Self {
        Self::new(ScrollSnapAlignValue::None, ScrollSnapAlignValue::None)
    }
}

/// Whether a scroll snap target may be skipped during an active snap operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollSnapStop {
    #[default]
    Normal,
    Always,
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
    pub overflow: ComputedOverflow,
    pub overflow_clip_margin: OverflowClipMarginOf<S>,
    pub scrollbar_gutter: ScrollbarGutter,
    pub scrollbar_width: self::ScrollbarWidthOf<S>,
    pub scroll_padding: ScrollPaddingOf<S>,
    pub scroll_margin: ScrollMarginOf<S>,
    pub scroll_snap_type: ScrollSnapType,
    pub scroll_snap_align: ScrollSnapAlign,
    pub scroll_snap_stop: ScrollSnapStop,
    pub position: Position,
    pub float: Float,
    pub clear: Clear,
    pub inset: Edges<LengthAutoOf<S>>,
    pub size: Size<PreferredSizeOf<S>>,
    pub min_size: Size<MinSizeOf<S>>,
    pub max_size: Size<MaxSizeOf<S>>,
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
    pub flex_basis: FlexBasisOf<S>,
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

/// Property sizing fields use distinct public domains.
///
/// ```compile_fail
/// use surgeist_layout::{MaxSize, NodeInput, PreferredSize, Size};
/// let _ = NodeInput {
///     size: Size::splat(MaxSize::NONE),
///     ..NodeInput::DEFAULT
/// };
/// let _: PreferredSize = MaxSize::NONE;
/// ```
///
/// The removed broad sizing family has no compatibility reexport.
///
/// ```compile_fail
/// use surgeist_layout::Dimension;
/// let _ = Dimension::AUTO;
/// ```
///
/// ```compile_fail
/// use surgeist_layout::DimensionOf;
/// type Legacy = DimensionOf<f64>;
/// let _: Legacy = Legacy::AUTO;
/// ```
const _: () = ();

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
        overflow: ComputedOverflow::VISIBLE,
        overflow_clip_margin: OverflowClipMarginOf {
            clip_box: OverflowClipBox::PaddingBox,
            margin: 0.0,
        },
        scrollbar_gutter: ScrollbarGutter::Auto,
        scrollbar_width: self::ScrollbarWidthOf::ZERO,
        scroll_padding: ScrollPaddingOf {
            top: ScrollPaddingValueOf::AUTO,
            right: ScrollPaddingValueOf::AUTO,
            bottom: ScrollPaddingValueOf::AUTO,
            left: ScrollPaddingValueOf::AUTO,
        },
        scroll_margin: ScrollMarginOf {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        },
        scroll_snap_type: ScrollSnapType::None,
        scroll_snap_align: ScrollSnapAlign {
            block: ScrollSnapAlignValue::None,
            inline: ScrollSnapAlignValue::None,
        },
        scroll_snap_stop: ScrollSnapStop::Normal,
        position: Position::Relative,
        float: Float::None,
        clear: Clear::None,
        inset: Edges::all(LengthAutoOf::AUTO),
        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO),
        max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE),
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
        flex_basis: FlexBasisOf::AUTO,
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
            overflow: ComputedOverflow::VISIBLE,
            overflow_clip_margin: OverflowClipMarginOf::default(),
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: self::ScrollbarWidthOf::ZERO,
            scroll_padding: ScrollPaddingOf::default(),
            scroll_margin: ScrollMarginOf::default(),
            scroll_snap_type: ScrollSnapType::None,
            scroll_snap_align: ScrollSnapAlign::default(),
            scroll_snap_stop: ScrollSnapStop::Normal,
            position: Position::Relative,
            float: Float::None,
            clear: Clear::None,
            inset: Edges::all(LengthAutoOf::AUTO),
            size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO),
            max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE),
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
            flex_basis: FlexBasisOf::AUTO,
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

#[cfg(test)]
mod property_field_migration_tests {
    use super::*;
    use crate::{
        CalcSizeCalculation, FlexBasisOf, LengthPercentageOf, LengthResolutionStatus, MaxSizeOf,
        MinSizeOf, PreferredSizeCalcBasis, PreferredSizeOf, SizingCalculation,
    };

    fn assert_field_types<S: LayoutScalar>(input: &NodeInputOf<S>) {
        let _: &Size<PreferredSizeOf<S>> = &input.size;
        let _: &Size<MinSizeOf<S>> = &input.min_size;
        let _: &Size<MaxSizeOf<S>> = &input.max_size;
        let _: &FlexBasisOf<S> = &input.flex_basis;
    }

    fn assert_generic_defaults<S: LayoutScalar>() {
        let input = NodeInputOf::<S>::default();
        assert_field_types(&input);
        assert_eq!(
            input.size,
            Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
        );
        assert_eq!(input.min_size, Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO));
        assert_eq!(input.max_size, Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE));
        assert_eq!(input.flex_basis, FlexBasisOf::AUTO);
    }

    #[test]
    fn property_field_migration_default_scalar_uses_exact_domains_and_initial_values() {
        let input = &NodeInput::DEFAULT;
        assert_field_types(input);
        assert_eq!(
            input.size,
            Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
        );
        assert_eq!(input.min_size, Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO));
        assert_eq!(input.max_size, Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE));
        assert_eq!(input.flex_basis, FlexBasisOf::AUTO);
    }

    #[test]
    fn property_field_migration_generic_scalar_uses_exact_domains_and_initial_values() {
        assert_generic_defaults::<f32>();
        assert_generic_defaults::<f64>();
    }

    #[test]
    fn property_field_migration_numeric_calculations_resolve_while_later_states_stay_unsupported() {
        let nested =
            SizingCalculation::min(vec![SizingCalculation::value(LengthPercentageOf::ZERO)])
                .expect("nonempty sizing calculation");
        let preferred = PreferredSizeOf::calculation(nested);
        let resolution = preferred
            .resolve_simple_with_status(None)
            .expect("valid numeric calculation resolves");
        assert_eq!(resolution.status(), LengthResolutionStatus::Resolved);
        assert_eq!(resolution.value, Some(0.0));

        let calc_size =
            PreferredSizeOf::calc_size(PreferredSizeCalcBasis::Auto, CalcSizeCalculation::size())
                .expect("valid preferred calc-size");
        assert_eq!(
            calc_size.resolve_simple_with_status(None),
            Err(LengthResolutionStatus::NonNumeric),
        );
        assert_eq!(
            FlexBasisOf::<f32>::CONTENT.resolve_simple_with_status(None),
            Err(LengthResolutionStatus::NonNumeric),
        );
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

    fn assert_fri05_c01_node_input_fields_and_defaults<S: LayoutScalar>(input: &NodeInputOf<S>) {
        let _: &ComputedOverflow = &input.overflow;
        let _: &OverflowClipMarginOf<S> = &input.overflow_clip_margin;
        let _: &ScrollbarGutter = &input.scrollbar_gutter;
        let _: &ScrollbarWidthOf<S> = &input.scrollbar_width;
        let _: &ScrollPaddingOf<S> = &input.scroll_padding;
        let _: &ScrollMarginOf<S> = &input.scroll_margin;
        let _: &ScrollSnapType = &input.scroll_snap_type;
        let _: &ScrollSnapAlign = &input.scroll_snap_align;
        let _: &ScrollSnapStop = &input.scroll_snap_stop;

        assert_eq!(input.overflow, ComputedOverflow::VISIBLE);
        assert_eq!(
            input.overflow_clip_margin,
            OverflowClipMarginOf::<S>::default()
        );
        assert_eq!(input.scrollbar_gutter, ScrollbarGutter::Auto);
        assert_eq!(input.scrollbar_width, ScrollbarWidthOf::<S>::ZERO);
        assert_eq!(input.scroll_padding, ScrollPaddingOf::<S>::default());
        assert_eq!(input.scroll_margin, ScrollMarginOf::<S>::default());
        assert_eq!(input.scroll_snap_type, ScrollSnapType::None);
        assert_eq!(input.scroll_snap_align, ScrollSnapAlign::default());
        assert_eq!(input.scroll_snap_stop, ScrollSnapStop::Normal);
    }

    #[test]
    fn fri05_c01_node_input_default_and_generic_fields_have_exact_domains_and_initial_values() {
        assert_fri05_c01_node_input_fields_and_defaults(&NodeInput::DEFAULT);
        assert_fri05_c01_node_input_fields_and_defaults(&NodeInputOf::<f32>::default());
        assert_fri05_c01_node_input_fields_and_defaults(&NodeInputOf::<f64>::default());
    }

    fn assert_canonical_zero<S: LayoutScalar>(value: S) {
        assert_eq!(value, S::ZERO);
        assert_eq!(value.to_f64().to_bits(), 0.0f64.to_bits());
    }

    fn assert_scroll_input_scalar_traits<S: LayoutScalar>() {
        fn assert_value<T: Clone + Copy + core::fmt::Debug + PartialEq>() {}
        fn assert_error<T: Clone + Copy + core::fmt::Debug + PartialEq + std::error::Error>() {}

        assert_value::<OverflowClipMarginOf<S>>();
        assert_value::<ScrollPaddingValueOf<S>>();
        assert_value::<ScrollPaddingOf<S>>();
        assert_value::<ScrollMarginOf<S>>();
        assert_error::<ScrollMarginErrorOf<S>>();
    }

    fn assert_clip_margin_contract<S: LayoutScalar>() {
        let default = OverflowClipMarginOf::<S>::default();
        assert_eq!(default.clip_box(), OverflowClipBox::PaddingBox);
        assert_canonical_zero(default.margin());

        for clip_box in [
            OverflowClipBox::ContentBox,
            OverflowClipBox::PaddingBox,
            OverflowClipBox::BorderBox,
        ] {
            let clip_margin = OverflowClipMarginOf::try_new(clip_box, S::from_f64(4.5))
                .expect("finite non-negative clip margin");
            assert_eq!(clip_margin.clip_box(), clip_box);
            assert_eq!(clip_margin.margin(), S::from_f64(4.5));
        }

        let signed_zero = OverflowClipMarginOf::try_new(OverflowClipBox::BorderBox, -S::ZERO)
            .expect("signed zero clip margin is valid");
        assert_canonical_zero(signed_zero.margin());

        assert_eq!(
            OverflowClipMarginOf::try_new(OverflowClipBox::ContentBox, S::from_f64(-1.0)),
            Err(NonNegativeFiniteScalarErrorOf::Negative {
                value: S::from_f64(-1.0),
            })
        );
        for value in [S::NAN, S::INFINITY, -S::INFINITY] {
            assert!(matches!(
                OverflowClipMarginOf::try_new(OverflowClipBox::PaddingBox, value),
                Err(NonNegativeFiniteScalarErrorOf::NonFinite { value: rejected })
                    if !rejected.is_finite()
            ));
        }
    }

    #[test]
    fn fri05_c01_scroll_input_clip_margin_is_validated_in_both_scalar_lanes() {
        assert_clip_margin_contract::<f32>();
        assert_clip_margin_contract::<f64>();
    }

    fn assert_padding_contract<S: LayoutScalar>(largest_finite: S) {
        let auto = ScrollPaddingValueOf::<S>::AUTO;
        assert_eq!(ScrollPaddingValueOf::<S>::default(), auto);
        assert_eq!(ScrollPaddingValueOf::<S>::auto(), auto);
        assert!(auto.is_auto());
        assert_eq!(
            auto.resolve_against(PercentageBasisOf::MISSING),
            NumericResolutionOf::Resolved(S::ZERO)
        );

        let quarter = LengthPercentageOf::from_percent_fraction(S::from_f64(0.25))
            .expect("finite percentage");
        let value = ScrollPaddingValueOf::value(quarter);
        assert!(!value.is_auto());
        assert_eq!(
            value.resolve_against(
                PercentageBasisOf::definite(S::from_f64(200.0)).expect("finite width")
            ),
            NumericResolutionOf::Resolved(S::from_f64(50.0))
        );
        assert_eq!(
            value.resolve_against(
                PercentageBasisOf::definite(S::from_f64(80.0)).expect("finite height")
            ),
            NumericResolutionOf::Resolved(S::from_f64(20.0))
        );
        assert_eq!(
            value.resolve_against(PercentageBasisOf::MISSING),
            NumericResolutionOf::MissingBasis { value: quarter }
        );

        let negative = LengthPercentageOf::from_coefficients(S::from_f64(-30.0), S::from_f64(0.1))
            .expect("finite negative calculation");
        let NumericResolutionOf::Resolved(clamped) = ScrollPaddingValueOf::value(negative)
            .resolve_against(
                PercentageBasisOf::definite(S::from_f64(100.0)).expect("finite basis"),
            )
        else {
            panic!("negative used scroll padding must resolve");
        };
        assert_canonical_zero(clamped);

        let overflowing = LengthPercentageOf::from_percent_fraction(largest_finite)
            .expect("largest finite coefficient");
        let basis = PercentageBasisOf::definite(S::from_f64(2.0)).expect("finite basis");
        let NumericResolutionOf::InvalidNumeric {
            value: invalid_value,
            basis: invalid_basis,
            resolved,
        } = ScrollPaddingValueOf::value(overflowing).resolve_against(basis)
        else {
            panic!("non-finite evaluation must remain invalid");
        };
        assert_eq!(invalid_value, overflowing);
        assert_eq!(invalid_basis, basis);
        assert!(!resolved.is_finite());

        let padding = ScrollPaddingOf::new(
            auto,
            value,
            ScrollPaddingValueOf::value(negative),
            ScrollPaddingValueOf::value(LengthPercentageOf::ZERO),
        );
        assert_eq!(padding.top(), auto);
        assert_eq!(padding.right(), value);
        assert_eq!(padding.bottom(), ScrollPaddingValueOf::value(negative));
        assert_eq!(
            padding.left(),
            ScrollPaddingValueOf::value(LengthPercentageOf::ZERO)
        );

        let default = ScrollPaddingOf::<S>::default();
        assert_eq!(default.top(), auto);
        assert_eq!(default.right(), auto);
        assert_eq!(default.bottom(), auto);
        assert_eq!(default.left(), auto);
    }

    #[test]
    fn fri05_c01_scroll_input_padding_resolves_physical_bases_in_both_scalar_lanes() {
        assert_padding_contract::<f32>(f32::MAX);
        assert_padding_contract::<f64>(f64::MAX);
    }

    fn assert_same_non_finite<S: LayoutScalar>(actual: S, expected: S) {
        if expected.to_f64().is_nan() {
            assert!(actual.to_f64().is_nan());
        } else {
            assert_eq!(actual.to_f64(), expected.to_f64());
        }
    }

    fn assert_scroll_margin_contract<S: LayoutScalar>() {
        let margin = ScrollMarginOf::try_new(
            S::from_f64(-4.0),
            S::from_f64(2.0),
            -S::ZERO,
            S::from_f64(6.0),
        )
        .expect("finite signed scroll margins");
        assert_eq!(margin.top(), S::from_f64(-4.0));
        assert_eq!(margin.right(), S::from_f64(2.0));
        assert_canonical_zero(margin.bottom());
        assert_eq!(margin.left(), S::from_f64(6.0));

        let default = ScrollMarginOf::<S>::default();
        assert_canonical_zero(default.top());
        assert_canonical_zero(default.right());
        assert_canonical_zero(default.bottom());
        assert_canonical_zero(default.left());

        for (edge, values, rejected, edge_name) in [
            (
                PhysicalSide::Top,
                [S::NAN, S::ZERO, S::ZERO, S::ZERO],
                S::NAN,
                "top",
            ),
            (
                PhysicalSide::Right,
                [S::ZERO, S::INFINITY, S::ZERO, S::ZERO],
                S::INFINITY,
                "right",
            ),
            (
                PhysicalSide::Bottom,
                [S::ZERO, S::ZERO, -S::INFINITY, S::ZERO],
                -S::INFINITY,
                "bottom",
            ),
            (
                PhysicalSide::Left,
                [S::ZERO, S::ZERO, S::ZERO, S::NAN],
                S::NAN,
                "left",
            ),
        ] {
            let error = ScrollMarginOf::try_new(values[0], values[1], values[2], values[3])
                .expect_err("non-finite aggregate edge must fail atomically");
            assert_eq!(error.edge(), edge);
            let FiniteScalarErrorOf::NonFinite { value } = error.error();
            assert_same_non_finite(value, rejected);
            assert_eq!(
                error.to_string(),
                format!("scroll margin {edge_name} edge must be finite")
            );

            let source = std::error::Error::source(&error)
                .expect("scroll margin diagnostic preserves its scalar source")
                .downcast_ref::<FiniteScalarErrorOf<S>>()
                .expect("source has the exact finite-scalar type");
            let FiniteScalarErrorOf::NonFinite { value } = *source;
            assert_same_non_finite(value, rejected);
        }
    }

    #[test]
    fn fri05_c01_scroll_input_signed_margin_is_atomic_in_both_scalar_lanes() {
        assert_scroll_margin_contract::<f32>();
        assert_scroll_margin_contract::<f64>();
    }

    #[test]
    fn fri05_c01_scroll_input_closed_enums_cover_states_defaults_and_traits() {
        fn assert_closed<T: Clone + Copy + core::fmt::Debug + Eq + PartialEq>() {}

        assert_closed::<OverflowClipBox>();
        assert_closed::<ScrollbarGutter>();
        assert_closed::<ScrollSnapAxis>();
        assert_closed::<ScrollSnapStrictness>();
        assert_closed::<ScrollSnapType>();
        assert_closed::<ScrollSnapAlignValue>();
        assert_closed::<ScrollSnapAlign>();
        assert_closed::<ScrollSnapStop>();
        assert_scroll_input_scalar_traits::<f32>();
        assert_scroll_input_scalar_traits::<f64>();

        assert_eq!(OverflowClipBox::default(), OverflowClipBox::PaddingBox);
        assert_eq!(ScrollbarGutter::default(), ScrollbarGutter::Auto);
        assert_eq!(ScrollSnapAlignValue::default(), ScrollSnapAlignValue::None);
        assert_eq!(ScrollSnapType::default(), ScrollSnapType::None);
        assert_eq!(ScrollSnapStop::default(), ScrollSnapStop::Normal);

        let clip_boxes = [
            OverflowClipBox::ContentBox,
            OverflowClipBox::PaddingBox,
            OverflowClipBox::BorderBox,
        ];
        let gutters = [
            ScrollbarGutter::Auto,
            ScrollbarGutter::Stable,
            ScrollbarGutter::StableBothEdges,
        ];
        let axes = [
            ScrollSnapAxis::X,
            ScrollSnapAxis::Y,
            ScrollSnapAxis::Block,
            ScrollSnapAxis::Inline,
            ScrollSnapAxis::Both,
        ];
        let strictnesses = [
            ScrollSnapStrictness::Proximity,
            ScrollSnapStrictness::Mandatory,
        ];
        let alignments = [
            ScrollSnapAlignValue::None,
            ScrollSnapAlignValue::Start,
            ScrollSnapAlignValue::End,
            ScrollSnapAlignValue::Center,
        ];
        let stops = [ScrollSnapStop::Normal, ScrollSnapStop::Always];

        assert_eq!(clip_boxes.len(), 3);
        assert_eq!(gutters.len(), 3);
        assert_eq!(alignments.len(), 4);
        assert_eq!(stops.len(), 2);
        for axis in axes {
            for strictness in strictnesses {
                assert_eq!(
                    ScrollSnapType::Enabled { axis, strictness },
                    ScrollSnapType::Enabled { axis, strictness }
                );
            }
        }
    }

    #[test]
    fn fri05_c01_scroll_input_snap_alignment_keeps_block_and_inline_roles() {
        let alignment =
            ScrollSnapAlign::new(ScrollSnapAlignValue::Start, ScrollSnapAlignValue::End);
        assert_eq!(alignment.block(), ScrollSnapAlignValue::Start);
        assert_eq!(alignment.inline(), ScrollSnapAlignValue::End);

        let default = ScrollSnapAlign::default();
        assert_eq!(default.block(), ScrollSnapAlignValue::None);
        assert_eq!(default.inline(), ScrollSnapAlignValue::None);
    }

    #[test]
    fn fri05_c01_computed_overflow_accepts_exact_canonical_pair_table() {
        let values = [
            Overflow::Visible,
            Overflow::Clip,
            Overflow::Hidden,
            Overflow::Scroll,
            Overflow::Auto,
        ];
        let accepted = [
            [true, true, false, false, false],
            [true, true, false, false, false],
            [false, false, true, true, true],
            [false, false, true, true, true],
            [false, false, true, true, true],
        ];
        let mut accepted_count = 0;
        let mut rejected_count = 0;

        for (x_index, x) in values.into_iter().enumerate() {
            for (y_index, y) in values.into_iter().enumerate() {
                let result = ComputedOverflow::try_new(x, y);
                if accepted[x_index][y_index] {
                    let pair = result.expect("canonical pair must be accepted");
                    assert_eq!((pair.x(), pair.y()), (x, y));
                    accepted_count += 1;
                } else {
                    assert_eq!(
                        result,
                        Err(ComputedOverflowError::NonCanonicalPair { x, y })
                    );
                    rejected_count += 1;
                }
            }
        }

        assert_eq!(accepted_count, 13);
        assert_eq!(rejected_count, 12);
    }

    #[test]
    fn fri05_c01_computed_overflow_visible_default_traits_and_diagnostics_are_exact() {
        fn assert_value_traits<T: Clone + Copy + core::fmt::Debug + Eq + PartialEq>() {}
        fn assert_error_traits<
            T: Clone + Copy + core::fmt::Debug + Eq + PartialEq + std::error::Error,
        >() {
        }

        const VISIBLE_X: Overflow = ComputedOverflow::VISIBLE.x();
        const VISIBLE_Y: Overflow = ComputedOverflow::VISIBLE.y();

        assert_value_traits::<ComputedOverflow>();
        assert_error_traits::<ComputedOverflowError>();
        assert_eq!(ComputedOverflow::default(), ComputedOverflow::VISIBLE);
        assert_eq!(
            (VISIBLE_X, VISIBLE_Y),
            (Overflow::Visible, Overflow::Visible)
        );

        let pair = ComputedOverflow::try_new(Overflow::Clip, Overflow::Visible)
            .expect("visible/clip pair is canonical");
        assert_eq!(
            format!("{pair:?}"),
            "ComputedOverflow { x: Clip, y: Visible }"
        );

        let error = ComputedOverflowError::NonCanonicalPair {
            x: Overflow::Visible,
            y: Overflow::Auto,
        };
        assert_eq!(
            error.to_string(),
            "computed overflow axes must both be visible/clip or both be hidden/scroll/auto"
        );
        assert_eq!(
            format!("{error:?}"),
            "NonCanonicalPair { x: Visible, y: Auto }"
        );
    }

    #[test]
    fn fri05_c01_computed_overflow_scrollability_and_block_pair_predicate_are_exact() {
        for (overflow, expected) in [
            (Overflow::Visible, false),
            (Overflow::Clip, false),
            (Overflow::Hidden, true),
            (Overflow::Scroll, true),
            (Overflow::Auto, true),
        ] {
            assert_eq!(overflow.is_scrollable(), expected);
        }

        for x in [Overflow::Visible, Overflow::Clip] {
            for y in [Overflow::Visible, Overflow::Clip] {
                let pair = ComputedOverflow::try_new(x, y).expect("pair is canonical");
                assert!(!pair.establishes_independent_formatting_context());
            }
        }

        for x in [Overflow::Hidden, Overflow::Scroll, Overflow::Auto] {
            for y in [Overflow::Hidden, Overflow::Scroll, Overflow::Auto] {
                let pair = ComputedOverflow::try_new(x, y).expect("pair is canonical");
                assert!(pair.establishes_independent_formatting_context());
            }
        }
    }

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
