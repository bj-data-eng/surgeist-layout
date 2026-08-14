use super::super::{DefaultScalar, GridLine, GridSpan, LayoutScalar, LengthOf};

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
