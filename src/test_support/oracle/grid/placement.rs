#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridAxis {
    Column,
    Row,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridArea {
    pub column_start: usize,
    pub row_start: usize,
    pub column_span: usize,
    pub row_span: usize,
}

impl GridArea {
    #[must_use]
    pub const fn new(
        column_start: usize,
        row_start: usize,
        column_span: usize,
        row_span: usize,
    ) -> Self {
        Self {
            column_start,
            row_start,
            column_span,
            row_span,
        }
    }

    #[must_use]
    pub const fn start(self, axis: GridAxis) -> usize {
        match axis {
            GridAxis::Column => self.column_start,
            GridAxis::Row => self.row_start,
        }
    }

    #[must_use]
    pub const fn span(self, axis: GridAxis) -> usize {
        match axis {
            GridAxis::Column => self.column_span,
            GridAxis::Row => self.row_span,
        }
    }
}

/// Positive numeric grid-line placement for the base oracle.
///
/// Negative and named line resolution are intentionally outside this slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinePlacement {
    Auto,
    Line(isize),
    Span(usize),
    LineSpan { start: isize, span: usize },
    SpanLine { span: usize, end: isize },
    Lines { start: isize, end: isize },
}

impl LinePlacement {
    pub fn resolve_axis(self, auto_start_line: isize) -> Result<AxisPlacement, PlacementError> {
        match self {
            Self::Auto => AxisPlacement::new(auto_start_line, auto_start_line + 1),
            Self::Line(start) => AxisPlacement::new(start, start + 1),
            Self::Span(span) => {
                validate_span(span)?;
                AxisPlacement::new(auto_start_line, auto_start_line + span as isize)
            }
            Self::LineSpan { start, span } => {
                validate_span(span)?;
                AxisPlacement::new(start, start + span as isize)
            }
            Self::SpanLine { span, end } => {
                validate_span(span)?;
                AxisPlacement::new(end - span as isize, end)
            }
            Self::Lines { start, end } => AxisPlacement::new(start, end),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemPlacement {
    pub column: LinePlacement,
    pub row: LinePlacement,
}

impl ItemPlacement {
    pub fn resolve(
        self,
        auto_column_start_line: isize,
        auto_row_start_line: isize,
    ) -> Result<ResolvedItemPlacement, PlacementError> {
        Ok(ResolvedItemPlacement {
            column: self.column.resolve_axis(auto_column_start_line)?,
            row: self.row.resolve_axis(auto_row_start_line)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisPlacement {
    pub start_line: isize,
    pub end_line: isize,
    pub span: usize,
}

impl AxisPlacement {
    fn new(start_line: isize, end_line: isize) -> Result<Self, PlacementError> {
        if start_line < 1 || end_line < 1 {
            return Err(PlacementError::LineBeforeFirst);
        }
        if end_line <= start_line {
            return Err(PlacementError::EndBeforeStart);
        }

        Ok(Self {
            start_line,
            end_line,
            span: (end_line - start_line) as usize,
        })
    }

    pub fn try_new(start_line: isize, end_line: isize) -> Result<Self, PlacementError> {
        Self::new(start_line, end_line)
    }

    #[must_use]
    pub fn implicit_after(self, explicit_tracks: usize) -> usize {
        let explicit_end_line = explicit_tracks as isize + 1;
        self.end_line
            .saturating_sub(explicit_end_line)
            .try_into()
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedItemPlacement {
    pub column: AxisPlacement,
    pub row: AxisPlacement,
}

impl ResolvedItemPlacement {
    #[must_use]
    pub fn area(self) -> GridArea {
        GridArea::new(
            self.column.start_line as usize,
            self.row.start_line as usize,
            self.column.span,
            self.row.span,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementError {
    ZeroSpan,
    EndBeforeStart,
    LineBeforeFirst,
    UnresolvedAuto,
    NamedLinesUnsupported,
    NoExplicitTracks(GridAxis),
    SpanExceedsExplicitTracks {
        axis: GridAxis,
        span: usize,
        explicit_tracks: usize,
    },
}

fn validate_span(span: usize) -> Result<(), PlacementError> {
    if span == 0 {
        Err(PlacementError::ZeroSpan)
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AutoPlacer {
    flow: Flow,
    columns: usize,
    rows: usize,
    placed: Vec<GridArea>,
    occupied: Vec<GridArea>,
    cursor_column: usize,
    cursor_row: usize,
}

impl AutoPlacer {
    pub fn try_new(columns: usize, rows: usize, flow: Flow) -> Result<Self, PlacementError> {
        if columns == 0 {
            return Err(PlacementError::NoExplicitTracks(GridAxis::Column));
        }
        if rows == 0 {
            return Err(PlacementError::NoExplicitTracks(GridAxis::Row));
        }
        Ok(Self {
            flow,
            columns,
            rows,
            placed: Vec::new(),
            occupied: Vec::new(),
            cursor_column: 1,
            cursor_row: 1,
        })
    }

    #[must_use]
    pub fn occupied(mut self, area: GridArea) -> Self {
        self.occupied.push(area);
        self
    }

    pub fn place(
        &mut self,
        column_span: usize,
        row_span: usize,
    ) -> Result<GridArea, PlacementError> {
        if column_span == 0 || row_span == 0 {
            return Err(PlacementError::ZeroSpan);
        }
        match self.flow {
            Flow::Row | Flow::RowDense if column_span > self.columns => {
                return Err(PlacementError::SpanExceedsExplicitTracks {
                    axis: GridAxis::Column,
                    span: column_span,
                    explicit_tracks: self.columns,
                });
            }
            Flow::Column | Flow::ColumnDense if row_span > self.rows => {
                return Err(PlacementError::SpanExceedsExplicitTracks {
                    axis: GridAxis::Row,
                    span: row_span,
                    explicit_tracks: self.rows,
                });
            }
            Flow::Row | Flow::RowDense | Flow::Column | Flow::ColumnDense => {}
        }

        let dense = matches!(self.flow, Flow::RowDense | Flow::ColumnDense);
        let start = if dense {
            (1, 1)
        } else {
            (self.cursor_column, self.cursor_row)
        };
        let area = match self.flow {
            Flow::Row | Flow::RowDense => self.place_row_major(start, column_span, row_span),
            Flow::Column | Flow::ColumnDense => {
                self.place_column_major(start, column_span, row_span)
            }
        };

        self.placed.push(area);
        self.occupied.push(area);
        if !dense {
            match self.flow {
                Flow::Row | Flow::RowDense => {
                    self.cursor_column = area.column_start + area.column_span;
                    self.cursor_row = area.row_start;
                    if self.cursor_column > self.columns {
                        self.cursor_column = 1;
                        self.cursor_row += 1;
                    }
                }
                Flow::Column | Flow::ColumnDense => {
                    self.cursor_column = area.column_start;
                    self.cursor_row = area.row_start + area.row_span;
                    if self.cursor_row > self.rows {
                        self.cursor_row = 1;
                        self.cursor_column += 1;
                    }
                }
            }
        }
        Ok(area)
    }

    #[must_use]
    pub fn report(&self) -> PlacementReport {
        PlacementReport {
            areas: self.placed.clone(),
            occupied: self.occupied.clone(),
            implicit_columns_before: 0,
            implicit_columns_after: implicit_after(&self.occupied, self.columns, GridAxis::Column),
            implicit_rows_before: 0,
            implicit_rows_after: implicit_after(&self.occupied, self.rows, GridAxis::Row),
            cursor: PlacementCursor {
                column: self.cursor_column,
                row: self.cursor_row,
            },
        }
    }

    fn place_row_major(
        &self,
        start: (usize, usize),
        column_span: usize,
        row_span: usize,
    ) -> GridArea {
        let (mut column, mut row) = start;
        loop {
            if column + column_span - 1 <= self.columns {
                let area = GridArea::new(column, row, column_span, row_span);
                if self.fits(area) {
                    return area;
                }
            }

            column += 1;
            if column > self.columns {
                column = 1;
                row += 1;
            }
        }
    }

    fn place_column_major(
        &self,
        start: (usize, usize),
        column_span: usize,
        row_span: usize,
    ) -> GridArea {
        let (mut column, mut row) = start;
        loop {
            if row + row_span - 1 <= self.rows {
                let area = GridArea::new(column, row, column_span, row_span);
                if self.fits(area) {
                    return area;
                }
            }

            row += 1;
            if row > self.rows {
                row = 1;
                column += 1;
            }
        }
    }

    fn fits(&self, candidate: GridArea) -> bool {
        !self
            .occupied
            .iter()
            .any(|occupied| areas_overlap(candidate, *occupied))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementCursor {
    pub column: usize,
    pub row: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementReport {
    pub areas: Vec<GridArea>,
    pub occupied: Vec<GridArea>,
    pub implicit_columns_before: usize,
    pub implicit_columns_after: usize,
    pub implicit_rows_before: usize,
    pub implicit_rows_after: usize,
    pub cursor: PlacementCursor,
}

fn areas_overlap(a: GridArea, b: GridArea) -> bool {
    let a_column_end = a.column_start + a.column_span;
    let b_column_end = b.column_start + b.column_span;
    let a_row_end = a.row_start + a.row_span;
    let b_row_end = b.row_start + b.row_span;

    a.column_start < b_column_end
        && b.column_start < a_column_end
        && a.row_start < b_row_end
        && b.row_start < a_row_end
}

fn implicit_after(areas: &[GridArea], explicit_tracks: usize, axis: GridAxis) -> usize {
    let max_track = areas
        .iter()
        .map(|area| match axis {
            GridAxis::Column => area.column_start + area.column_span - 1,
            GridAxis::Row => area.row_start + area.row_span - 1,
        })
        .max()
        .unwrap_or(0);

    max_track.saturating_sub(explicit_tracks)
}
