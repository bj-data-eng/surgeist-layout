use super::{
    AspectRatioOf, DefaultScalar, DimensionOf, Edges, GridLine, GridSpan, GridTemplateAreas,
    LayoutScalar, LengthAutoOf, LengthOf, Point, Size, TrackComponentOf,
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
pub enum BoxSizing {
    ContentBox,
    #[default]
    BorderBox,
}

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalLr,
    VerticalRl,
}

impl WritingMode {
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::VerticalLr | Self::VerticalRl)
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

    #[must_use]
    pub const fn main_axis(self) -> super::Axis {
        match self {
            Self::Row | Self::RowReverse => super::Axis::Horizontal,
            Self::Column | Self::ColumnReverse => super::Axis::Vertical,
        }
    }

    #[must_use]
    pub const fn cross_axis(self) -> super::Axis {
        match self {
            Self::Row | Self::RowReverse => super::Axis::Vertical,
            Self::Column | Self::ColumnReverse => super::Axis::Horizontal,
        }
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

#[derive(Clone, Debug, PartialEq)]
pub struct NodeInputOf<S: LayoutScalar = DefaultScalar> {
    pub display: Display,
    pub item_is_table: bool,
    pub item_is_replaced: bool,
    pub box_sizing: BoxSizing,
    pub direction: Direction,
    pub text_align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub writing_mode: WritingMode,
    pub overflow: Point<Overflow>,
    pub scrollbar_width: S,
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
    pub flex_grow: S,
    pub flex_shrink: S,
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
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Ltr,
        text_align: TextAlign::Auto,
        vertical_align: VerticalAlign::Baseline,
        writing_mode: WritingMode::HorizontalTb,
        overflow: Point {
            x: Overflow::Visible,
            y: Overflow::Visible,
        },
        scrollbar_width: DefaultScalar::ZERO,
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
        flex_grow: DefaultScalar::ZERO,
        flex_shrink: DefaultScalar::ONE,
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
            box_sizing: BoxSizing::BorderBox,
            direction: Direction::Ltr,
            text_align: TextAlign::Auto,
            vertical_align: VerticalAlign::Baseline,
            writing_mode: WritingMode::HorizontalTb,
            overflow: Point {
                x: Overflow::Visible,
                y: Overflow::Visible,
            },
            scrollbar_width: S::ZERO,
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
            flex_grow: S::ZERO,
            flex_shrink: S::ONE,
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
