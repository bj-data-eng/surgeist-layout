use crate::{
    GridPlacement, GridTemplateAreas, LayoutScalar, RawGridLine, RawGridPlacement,
    SubgridLineNameComponent, SubgridLineNameRepeatCount, TrackComponentOf, TrackRepeat,
};

use super::{GridAxisKind, GridContainerProjection, GridParentContext, InheritedGridAxis};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NamedGridLines {
    pub(super) axis: GridAxisKind,
    pub(super) explicit_track_count: usize,
    pub(super) line_names: Vec<Vec<LineNameEntry>>,
    pub(super) area_facts: GridAreaNameFacts,
}

impl NamedGridLines {
    pub(super) fn new(axis: GridAxisKind, explicit_track_count: usize) -> Self {
        Self {
            axis,
            explicit_track_count,
            line_names: vec![Vec::new(); explicit_track_count + 1],
            area_facts: GridAreaNameFacts::default(),
        }
    }

    fn ensure_track_count(&mut self, explicit_track_count: usize) {
        self.explicit_track_count = self.explicit_track_count.max(explicit_track_count);
        self.line_names
            .resize_with(self.explicit_track_count + 1, Vec::new);
    }

    fn add_line_names(&mut self, line_index: usize, names: &[String], origin: LineNameOrigin) {
        self.ensure_track_count(line_index);
        self.line_names[line_index].extend(
            names
                .iter()
                .cloned()
                .map(|name| LineNameEntry { name, origin }),
        );
    }

    fn add_area_name(&mut self, line_index: usize, name: String) {
        self.ensure_track_count(line_index);
        self.line_names[line_index].push(LineNameEntry {
            name,
            origin: LineNameOrigin::AreaGenerated,
        });
    }

    #[cfg(test)]
    pub(super) fn named_occurrences(&self, name: &str) -> Vec<usize> {
        self.line_names
            .iter()
            .enumerate()
            .filter_map(|(line_index, entries)| {
                entries
                    .iter()
                    .any(|entry| entry.name == name)
                    .then_some(line_index + 1)
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn entries_on_line(&self, line: usize) -> &[LineNameEntry] {
        if line == 0 {
            return &[];
        }

        self.line_names
            .get(line - 1)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LineNameEntry {
    pub(super) name: String,
    pub(super) origin: LineNameOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LineNameOrigin {
    Explicit,
    Inherited,
    AreaGenerated,
    LocalSubgrid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NamedGridError {
    ReservedLineName {
        name: String,
    },
    UnresolvedAutoRepeatNames {
        axis: GridAxisKind,
    },
    EmptyTemplateAreas,
    TemplateAreaRowLengthMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    NonRectangularTemplateArea {
        name: String,
    },
    ZeroRepeat {
        axis: GridAxisKind,
    },
    MultipleAutoFillRepeats {
        axis: GridAxisKind,
    },
    ZeroLine,
    ZeroSpan,
    AutoWithoutCursor,
    LineBeforeFirst {
        axis: GridAxisKind,
        line: isize,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NamedGridReport {
    errors: Vec<NamedGridErrorReport>,
}

impl NamedGridReport {
    pub fn errors(&self) -> &[NamedGridErrorReport] {
        &self.errors
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub(super) fn from_error(error: NamedGridError) -> Self {
        let mut report = Self::default();
        report.push_error(error);
        report
    }

    pub(super) fn push_error(&mut self, error: NamedGridError) {
        self.errors.push(error.into());
    }

    pub(super) fn extend(&mut self, other: NamedGridReport) {
        self.errors.extend(other.errors);
    }

    pub(super) fn extend_unique(&mut self, other: NamedGridReport) {
        for error in other.errors {
            if !self.errors.contains(&error) {
                self.errors.push(error);
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedGridErrorReport {
    ReservedLineName {
        name: String,
    },
    UnresolvedAutoRepeatNames {
        axis: GridAxisKind,
    },
    EmptyTemplateAreas,
    TemplateAreaRowLengthMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    NonRectangularTemplateArea {
        name: String,
    },
    ZeroRepeat {
        axis: GridAxisKind,
    },
    MultipleAutoFillRepeats {
        axis: GridAxisKind,
    },
    ZeroLine,
    ZeroSpan,
    AutoWithoutCursor,
    LineBeforeFirst {
        axis: GridAxisKind,
        line: isize,
    },
}

impl From<NamedGridError> for NamedGridErrorReport {
    fn from(error: NamedGridError) -> Self {
        match error {
            NamedGridError::ReservedLineName { name } => Self::ReservedLineName { name },
            NamedGridError::UnresolvedAutoRepeatNames { axis } => {
                Self::UnresolvedAutoRepeatNames { axis }
            }
            NamedGridError::EmptyTemplateAreas => Self::EmptyTemplateAreas,
            NamedGridError::TemplateAreaRowLengthMismatch {
                row,
                expected,
                actual,
            } => Self::TemplateAreaRowLengthMismatch {
                row,
                expected,
                actual,
            },
            NamedGridError::NonRectangularTemplateArea { name } => {
                Self::NonRectangularTemplateArea { name }
            }
            NamedGridError::ZeroRepeat { axis } => Self::ZeroRepeat { axis },
            NamedGridError::MultipleAutoFillRepeats { axis } => {
                Self::MultipleAutoFillRepeats { axis }
            }
            NamedGridError::ZeroLine => Self::ZeroLine,
            NamedGridError::ZeroSpan => Self::ZeroSpan,
            NamedGridError::AutoWithoutCursor => Self::AutoWithoutCursor,
            NamedGridError::LineBeforeFirst { axis, line } => Self::LineBeforeFirst { axis, line },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlacementSide {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpanSearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResolvedLine {
    css_line: isize,
    absolute_line: isize,
}

pub(super) fn resolve_grid_placement(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> Result<GridPlacement, NamedGridError> {
    let mut start = placement.start.clone();
    let mut end = placement.end.clone();

    if matches!(start, RawGridLine::NamedSpan { .. }) && matches!(end, RawGridLine::Auto) {
        start = RawGridLine::Span(1);
    }
    if matches!(end, RawGridLine::NamedSpan { .. }) && matches!(start, RawGridLine::Auto) {
        end = RawGridLine::Span(1);
    }
    if is_span(&start) && is_span(&end) {
        end = RawGridLine::Auto;
    }
    if matches!(start, RawGridLine::NamedSpan { .. }) && matches!(end, RawGridLine::Auto) {
        start = RawGridLine::Span(1);
    }

    match (&start, &end) {
        (RawGridLine::Auto, RawGridLine::Auto) => Ok(GridPlacement::AUTO),
        (RawGridLine::Auto, span) if is_span(span) => {
            let span = resolve_span_count(span)?;
            if let Some(cursor) = auto_cursor_line {
                Ok(GridPlacement::try_line_span(cursor, span)
                    .expect("resolved cursor line/span must be valid"))
            } else {
                Ok(GridPlacement::try_span(span).expect("resolved grid span must be valid"))
            }
        }
        (span, RawGridLine::Auto) if is_span(span) => {
            let span = resolve_span_count(span)?;
            if let Some(cursor) = auto_cursor_line {
                Ok(GridPlacement::try_line_span(cursor, span)
                    .expect("resolved cursor line/span must be valid"))
            } else {
                Ok(GridPlacement::try_span(span).expect("resolved grid span must be valid"))
            }
        }
        (RawGridLine::Auto, end) if is_definite_line(end) => {
            let end = resolve_line(lines, end, PlacementSide::End)?;
            validate_resolved_range(lines, end.absolute_line - 1, end.absolute_line)?;
            Ok(GridPlacement::try_end_line(end.css_line)
                .expect("resolved grid end line must be valid"))
        }
        (start, RawGridLine::Auto) if is_definite_line(start) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            validate_resolved_range(lines, start.absolute_line, start.absolute_line + 1)?;
            Ok(GridPlacement::try_line(start.css_line)
                .expect("resolved grid start line must be valid"))
        }
        (start, span) if is_definite_line(start) && is_span(span) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            match span {
                RawGridLine::NamedSpan { name, index } => {
                    let end =
                        resolve_named_span_from_start(lines, start.absolute_line, name, *index)?;
                    normalize_resolved_lines(lines, start, end)
                }
                _ => {
                    let span = resolve_span_count(span)?;
                    validate_resolved_range(
                        lines,
                        start.absolute_line,
                        start.absolute_line + span as isize,
                    )?;
                    Ok(GridPlacement::try_line_span(start.css_line, span)
                        .expect("resolved grid line/span must be valid"))
                }
            }
        }
        (span, end) if is_span(span) && is_definite_line(end) => {
            let end = resolve_line(lines, end, PlacementSide::End)?;
            match span {
                RawGridLine::NamedSpan { name, index } => {
                    let start =
                        resolve_named_span_from_end(lines, end.absolute_line, name, *index)?;
                    normalize_resolved_lines(lines, start, end)
                }
                _ => {
                    let span = resolve_span_count(span)?;
                    validate_resolved_range(
                        lines,
                        end.absolute_line - span as isize,
                        end.absolute_line,
                    )?;
                    Ok(GridPlacement::try_span_line(span, end.css_line)
                        .expect("resolved grid span/end line must be valid"))
                }
            }
        }
        (start, end) if is_definite_line(start) && is_definite_line(end) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            let end = resolve_line(lines, end, PlacementSide::End)?;
            normalize_resolved_lines(lines, start, end)
        }
        (RawGridLine::Auto, _) | (_, RawGridLine::Auto) => Err(NamedGridError::AutoWithoutCursor),
        _ => Err(NamedGridError::AutoWithoutCursor),
    }
}

#[cfg(test)]
pub(super) fn resolve_grid_placement_or_auto(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> GridPlacement {
    resolve_grid_placement_or_auto_with_report(lines, placement, auto_cursor_line).0
}

pub(super) fn resolve_grid_placement_or_auto_with_report(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> (GridPlacement, NamedGridReport) {
    match resolve_grid_placement(lines, placement, auto_cursor_line) {
        Ok(placement) => (placement, NamedGridReport::default()),
        Err(error) => (GridPlacement::AUTO, NamedGridReport::from_error(error)),
    }
}

pub(super) fn resolve_subgrid_placement(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> Result<GridPlacement, NamedGridError> {
    let Some((start_line, end_line)) =
        resolve_subgrid_placement_lines(lines, placement, auto_cursor_line)?
    else {
        return resolve_grid_placement(lines, placement, auto_cursor_line);
    };

    let (start_line, end_line) =
        clamp_subgrid_resolved_lines(start_line, end_line, lines.explicit_track_count);
    Ok(GridPlacement::try_lines(start_line, end_line)
        .expect("clamped subgrid placement lines must be valid"))
}

fn resolve_subgrid_placement_lines(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> Result<Option<(isize, isize)>, NamedGridError> {
    let mut start = placement.start.clone();
    let mut end = placement.end.clone();

    if matches!(start, RawGridLine::NamedSpan { .. }) && matches!(end, RawGridLine::Auto) {
        start = RawGridLine::Span(1);
    }
    if matches!(end, RawGridLine::NamedSpan { .. }) && matches!(start, RawGridLine::Auto) {
        end = RawGridLine::Span(1);
    }
    if is_span(&start) && is_span(&end) {
        end = RawGridLine::Auto;
    }
    if matches!(start, RawGridLine::NamedSpan { .. }) && matches!(end, RawGridLine::Auto) {
        start = RawGridLine::Span(1);
    }

    let (mut start_line, mut end_line) = match (&start, &end) {
        (RawGridLine::Auto, RawGridLine::Auto) => {
            let Some(cursor) = auto_cursor_line else {
                return Ok(None);
            };
            (cursor, cursor + 1)
        }
        (RawGridLine::Auto, span) if is_span(span) => {
            let span = resolve_span_count(span)?;
            let Some(cursor) = auto_cursor_line else {
                return Ok(None);
            };
            (cursor, cursor + span as isize)
        }
        (span, RawGridLine::Auto) if is_span(span) => {
            let span = resolve_span_count(span)?;
            let Some(cursor) = auto_cursor_line else {
                return Ok(None);
            };
            (cursor, cursor + span as isize)
        }
        (RawGridLine::Auto, end) if is_definite_line(end) => {
            let end = resolve_line(lines, end, PlacementSide::End)?;
            (end.absolute_line - 1, end.absolute_line)
        }
        (start, RawGridLine::Auto) if is_definite_line(start) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            (start.absolute_line, start.absolute_line + 1)
        }
        (start, span) if is_definite_line(start) && is_span(span) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            let end_line = match span {
                RawGridLine::NamedSpan { name, index } => {
                    resolve_named_span_from_start(lines, start.absolute_line, name, *index)?
                        .absolute_line
                }
                _ => start.absolute_line + resolve_span_count(span)? as isize,
            };
            (start.absolute_line, end_line)
        }
        (span, end) if is_span(span) && is_definite_line(end) => {
            let end = resolve_line(lines, end, PlacementSide::End)?;
            let start_line = match span {
                RawGridLine::NamedSpan { name, index } => {
                    resolve_subgrid_named_span_from_end(lines, end.absolute_line, name, *index)?
                        .absolute_line
                }
                _ => end.absolute_line - resolve_span_count(span)? as isize,
            };
            (start_line, end.absolute_line)
        }
        (start, end) if is_definite_line(start) && is_definite_line(end) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            let end = resolve_line(lines, end, PlacementSide::End)?;
            (start.absolute_line, end.absolute_line)
        }
        (RawGridLine::Auto, _) | (_, RawGridLine::Auto) => {
            return Err(NamedGridError::AutoWithoutCursor);
        }
        _ => return Err(NamedGridError::AutoWithoutCursor),
    };

    if start_line > end_line {
        std::mem::swap(&mut start_line, &mut end_line);
    } else if start_line == end_line {
        end_line = start_line + 1;
    }

    Ok(Some((start_line, end_line)))
}

fn resolve_subgrid_named_span_from_end(
    lines: &NamedGridLines,
    end_line: isize,
    name: &str,
    count: usize,
) -> Result<ResolvedLine, NamedGridError> {
    validate_line_name(name)?;
    if count == 0 {
        return Err(NamedGridError::ZeroSpan);
    }

    let explicit_end_line = lines.explicit_track_count as isize + 1;
    if end_line <= explicit_end_line + 1 {
        return resolve_named_span_from_end(lines, end_line, name, count);
    }

    let first_after_explicit = lines.explicit_track_count as isize + 2;
    let implicit_after_count = (end_line - first_after_explicit).max(0) as usize;
    if count <= implicit_after_count {
        let absolute_line = end_line - count as isize;
        return Ok(ResolvedLine {
            css_line: css_line_for_absolute_line(lines, absolute_line),
            absolute_line,
        });
    }

    let remaining_count = count - implicit_after_count;
    let explicit_matches = explicit_matches(lines, name);
    let explicit_candidates = explicit_matches
        .iter()
        .rev()
        .copied()
        .filter(|line| *line < end_line)
        .collect::<Vec<_>>();
    let absolute_line =
        resolve_backward_span(end_line, &explicit_candidates, remaining_count as isize);
    Ok(ResolvedLine {
        css_line: css_line_for_absolute_line(lines, absolute_line),
        absolute_line,
    })
}

fn clamp_subgrid_resolved_lines(
    start_line: isize,
    end_line: isize,
    explicit_track_count: usize,
) -> (isize, isize) {
    let explicit_end_line = explicit_track_count as isize + 1;
    let mut start_line = start_line.clamp(1, explicit_end_line);
    let mut end_line = end_line.clamp(1, explicit_end_line);

    if start_line == end_line && explicit_track_count > 0 {
        if start_line == explicit_end_line {
            start_line -= 1;
        } else {
            end_line += 1;
        }
    }

    (start_line, end_line)
}

fn normalize_resolved_lines(
    lines: &NamedGridLines,
    mut start: ResolvedLine,
    mut end: ResolvedLine,
) -> Result<GridPlacement, NamedGridError> {
    if start.absolute_line > end.absolute_line {
        std::mem::swap(&mut start, &mut end);
    }
    if start.absolute_line == end.absolute_line {
        validate_resolved_range(lines, start.absolute_line, start.absolute_line + 1)?;
        return Ok(GridPlacement::try_line_span(start.css_line, 1)
            .expect("normalized equal grid lines must produce a valid span"));
    }

    validate_resolved_range(lines, start.absolute_line, end.absolute_line)?;
    Ok(GridPlacement::try_lines(start.css_line, end.css_line)
        .expect("normalized grid lines must be valid"))
}

fn validate_resolved_range(
    lines: &NamedGridLines,
    start_line: isize,
    end_line: isize,
) -> Result<(), NamedGridError> {
    if start_line < 1 || end_line < 1 {
        return Err(NamedGridError::LineBeforeFirst {
            axis: lines.axis,
            line: start_line.min(end_line),
        });
    }
    Ok(())
}

fn resolve_line(
    lines: &NamedGridLines,
    line: &RawGridLine,
    side: PlacementSide,
) -> Result<ResolvedLine, NamedGridError> {
    match line {
        RawGridLine::Line(raw_line) => resolve_numeric_line(lines, *raw_line),
        RawGridLine::NamedLine { name, index } => resolve_named_line(lines, name, *index),
        RawGridLine::BareIdent(name) => resolve_bare_ident(lines, name, side),
        RawGridLine::Auto | RawGridLine::Span(_) | RawGridLine::NamedSpan { .. } => {
            Err(NamedGridError::AutoWithoutCursor)
        }
    }
}

fn resolve_numeric_line(
    lines: &NamedGridLines,
    raw_line: isize,
) -> Result<ResolvedLine, NamedGridError> {
    if raw_line == 0 {
        return Err(NamedGridError::ZeroLine);
    }
    let absolute_line = if raw_line > 0 {
        raw_line
    } else {
        lines.explicit_track_count as isize + 2 + raw_line
    };
    Ok(ResolvedLine {
        css_line: raw_line,
        absolute_line,
    })
}

fn resolve_named_line(
    lines: &NamedGridLines,
    name: &str,
    occurrence: isize,
) -> Result<ResolvedLine, NamedGridError> {
    validate_line_name(name)?;
    if occurrence == 0 {
        return Err(NamedGridError::ZeroLine);
    }

    let explicit_matches = explicit_matches(lines, name);
    let absolute_line = if occurrence > 0 {
        resolve_forward_occurrence(lines.explicit_track_count, &explicit_matches, occurrence)
    } else {
        resolve_backward_occurrence(&explicit_matches, occurrence)
    };

    Ok(ResolvedLine {
        css_line: css_line_for_absolute_line(lines, absolute_line),
        absolute_line,
    })
}

fn resolve_bare_ident(
    lines: &NamedGridLines,
    name: &str,
    side: PlacementSide,
) -> Result<ResolvedLine, NamedGridError> {
    validate_line_name(name)?;
    let side_name = match side {
        PlacementSide::Start => format!("{name}-start"),
        PlacementSide::End => format!("{name}-end"),
    };
    if !explicit_matches(lines, &side_name).is_empty() {
        return resolve_named_line(lines, &side_name, 1);
    }
    resolve_named_line(lines, name, 1)
}

fn resolve_named_span_from_start(
    lines: &NamedGridLines,
    start_line: isize,
    name: &str,
    count: usize,
) -> Result<ResolvedLine, NamedGridError> {
    resolve_named_span(lines, start_line, name, count, SpanSearchDirection::Forward)
}

fn resolve_named_span_from_end(
    lines: &NamedGridLines,
    end_line: isize,
    name: &str,
    count: usize,
) -> Result<ResolvedLine, NamedGridError> {
    resolve_named_span(lines, end_line, name, count, SpanSearchDirection::Backward)
}

fn resolve_named_span(
    lines: &NamedGridLines,
    edge_line: isize,
    name: &str,
    count: usize,
    direction: SpanSearchDirection,
) -> Result<ResolvedLine, NamedGridError> {
    validate_line_name(name)?;
    if count == 0 {
        return Err(NamedGridError::ZeroSpan);
    }

    let explicit_matches = explicit_matches(lines, name);
    let explicit_candidates = match direction {
        SpanSearchDirection::Forward => explicit_matches
            .iter()
            .copied()
            .filter(|line| *line > edge_line)
            .collect::<Vec<_>>(),
        SpanSearchDirection::Backward => explicit_matches
            .iter()
            .rev()
            .copied()
            .filter(|line| *line < edge_line)
            .collect::<Vec<_>>(),
    };
    let absolute_line = match direction {
        SpanSearchDirection::Forward => resolve_forward_span(
            lines.explicit_track_count,
            edge_line,
            &explicit_candidates,
            count as isize,
        ),
        SpanSearchDirection::Backward => {
            resolve_backward_span(edge_line, &explicit_candidates, count as isize)
        }
    };
    Ok(ResolvedLine {
        css_line: css_line_for_absolute_line(lines, absolute_line),
        absolute_line,
    })
}

fn resolve_span_count(line: &RawGridLine) -> Result<usize, NamedGridError> {
    let count = match line {
        RawGridLine::Span(count) | RawGridLine::NamedSpan { index: count, .. } => *count,
        _ => return Err(NamedGridError::AutoWithoutCursor),
    };
    if count == 0 {
        return Err(NamedGridError::ZeroSpan);
    }
    Ok(count)
}

fn is_definite_line(line: &RawGridLine) -> bool {
    matches!(
        line,
        RawGridLine::Line(_) | RawGridLine::BareIdent(_) | RawGridLine::NamedLine { .. }
    )
}

fn is_span(line: &RawGridLine) -> bool {
    matches!(line, RawGridLine::Span(_) | RawGridLine::NamedSpan { .. })
}

fn validate_line_name(name: &str) -> Result<(), NamedGridError> {
    if matches!(name, "auto" | "span") {
        return Err(NamedGridError::ReservedLineName {
            name: name.to_string(),
        });
    }
    Ok(())
}

fn explicit_matches(lines: &NamedGridLines, name: &str) -> Vec<isize> {
    lines
        .line_names
        .iter()
        .enumerate()
        .filter_map(|(line_index, entries)| {
            entries
                .iter()
                .any(|entry| entry.name == name)
                .then_some(line_index as isize + 1)
        })
        .collect()
}

fn resolve_forward_occurrence(
    explicit_track_count: usize,
    explicit_matches: &[isize],
    occurrence: isize,
) -> isize {
    let match_index = occurrence as usize - 1;
    if let Some(line) = explicit_matches.get(match_index) {
        return *line;
    }

    let missing_count = occurrence - explicit_matches.len() as isize;
    let first_implicit_line = explicit_track_count as isize + 2;
    first_implicit_line + missing_count - 1
}

fn resolve_backward_occurrence(explicit_matches: &[isize], occurrence: isize) -> isize {
    let requested_count = -occurrence;
    if requested_count <= explicit_matches.len() as isize {
        let match_index = explicit_matches.len() - requested_count as usize;
        return explicit_matches[match_index];
    }

    let missing_count = requested_count - explicit_matches.len() as isize;
    1 - missing_count
}

fn resolve_forward_span(
    explicit_track_count: usize,
    start_line: isize,
    explicit_candidates: &[isize],
    count: isize,
) -> isize {
    let match_index = count as usize - 1;
    if let Some(line) = explicit_candidates.get(match_index) {
        return *line;
    }

    let missing_count = count - explicit_candidates.len() as isize;
    let first_implicit_line = (explicit_track_count as isize + 2).max(start_line + 1);
    first_implicit_line + missing_count - 1
}

fn resolve_backward_span(end_line: isize, explicit_candidates: &[isize], count: isize) -> isize {
    let match_index = count as usize - 1;
    if let Some(line) = explicit_candidates.get(match_index) {
        return *line;
    }

    let missing_count = count - explicit_candidates.len() as isize;
    let first_implicit_line = (end_line - 1).min(0);
    first_implicit_line - missing_count + 1
}

fn css_line_for_absolute_line(lines: &NamedGridLines, absolute_line: isize) -> isize {
    if absolute_line > 0 {
        absolute_line
    } else {
        absolute_line - lines.explicit_track_count as isize - 2
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct GridAreaNameFacts {
    pub(super) row_count: usize,
    pub(super) column_count: usize,
    pub(super) rows_valid: bool,
    pub(super) columns_valid: bool,
    pub(super) area_order: Vec<String>,
    pub(super) area_rectangles: Vec<GridAreaNameRectangle>,
}

impl GridAreaNameFacts {
    pub(super) fn is_valid_for_axis(&self, axis: GridAxisKind) -> bool {
        match axis {
            GridAxisKind::Column => self.columns_valid,
            GridAxisKind::Row => self.rows_valid,
        }
    }

    pub(super) fn from_specified_areas(areas: &GridTemplateAreas) -> Result<Self, NamedGridError> {
        if areas.rows.is_empty() {
            return Err(NamedGridError::EmptyTemplateAreas);
        }

        let column_count = areas.rows[0].cells.len();
        if column_count == 0 {
            return Err(NamedGridError::TemplateAreaRowLengthMismatch {
                row: 1,
                expected: 1,
                actual: 0,
            });
        }

        let mut accumulators = Vec::<AreaAccumulator>::new();
        for (row_index, row) in areas.rows.iter().enumerate() {
            if row.cells.is_empty() {
                return Err(NamedGridError::TemplateAreaRowLengthMismatch {
                    row: row_index + 1,
                    expected: column_count.max(1),
                    actual: 0,
                });
            }
            if row.cells.len() != column_count {
                return Err(NamedGridError::TemplateAreaRowLengthMismatch {
                    row: row_index + 1,
                    expected: column_count,
                    actual: row.cells.len(),
                });
            }

            for (column_index, cell) in row.cells.iter().enumerate() {
                let Some(name) = cell else {
                    continue;
                };

                if let Some(accumulator) = accumulators
                    .iter_mut()
                    .find(|area| area.rectangle.name == *name)
                {
                    accumulator.rectangle.row_start =
                        accumulator.rectangle.row_start.min(row_index + 1);
                    accumulator.rectangle.row_end =
                        accumulator.rectangle.row_end.max(row_index + 2);
                    accumulator.rectangle.column_start =
                        accumulator.rectangle.column_start.min(column_index + 1);
                    accumulator.rectangle.column_end =
                        accumulator.rectangle.column_end.max(column_index + 2);
                    accumulator.cell_count += 1;
                } else {
                    accumulators.push(AreaAccumulator {
                        rectangle: GridAreaNameRectangle {
                            name: name.clone(),
                            row_start: row_index + 1,
                            row_end: row_index + 2,
                            column_start: column_index + 1,
                            column_end: column_index + 2,
                            row_start_name: row_index + 1,
                            row_end_name: row_index + 2,
                            column_start_name: column_index + 1,
                            column_end_name: column_index + 2,
                        },
                        cell_count: 1,
                    });
                }
            }
        }

        let mut area_order = Vec::with_capacity(accumulators.len());
        let mut area_rectangles = Vec::with_capacity(accumulators.len());
        for accumulator in accumulators {
            let mut rectangle = accumulator.rectangle;
            let expected_cells = (rectangle.row_end - rectangle.row_start)
                * (rectangle.column_end - rectangle.column_start);
            if accumulator.cell_count != expected_cells {
                return Err(NamedGridError::NonRectangularTemplateArea {
                    name: rectangle.name,
                });
            }
            rectangle.row_start_name = rectangle.row_start;
            rectangle.row_end_name = rectangle.row_end;
            rectangle.column_start_name = rectangle.column_start;
            rectangle.column_end_name = rectangle.column_end;
            area_order.push(rectangle.name.clone());
            area_rectangles.push(rectangle);
        }

        Ok(Self {
            row_count: areas.rows.len(),
            column_count,
            rows_valid: true,
            columns_valid: true,
            area_order,
            area_rectangles,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GridAreaNameRectangle {
    pub(super) name: String,
    pub(super) row_start: usize,
    pub(super) row_end: usize,
    pub(super) column_start: usize,
    pub(super) column_end: usize,
    pub(super) row_start_name: usize,
    pub(super) row_end_name: usize,
    pub(super) column_start_name: usize,
    pub(super) column_end_name: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AreaAccumulator {
    rectangle: GridAreaNameRectangle,
    cell_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GridNamedContext {
    pub(super) columns: NamedGridLines,
    pub(super) rows: NamedGridLines,
    pub(super) area_facts: Option<GridAreaNameFacts>,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "retained unit-test entry point for named-grid report parity scaffolding"
    )
)]
pub(super) fn build_grid_named_context<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    explicit_columns: usize,
    explicit_rows: usize,
    parent_context: &GridParentContext<S>,
) -> Result<GridNamedContext, NamedGridError> {
    build_grid_named_context_with_report(style, explicit_columns, explicit_rows, parent_context)
        .map(|(context, _report)| context)
}

pub(super) fn build_grid_named_context_with_report<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    explicit_columns: usize,
    explicit_rows: usize,
    parent_context: &GridParentContext<S>,
) -> Result<(GridNamedContext, NamedGridReport), NamedGridError> {
    let mut report = NamedGridReport::default();
    let style_area_facts = if style.grid_template_areas.rows.is_empty() {
        None
    } else {
        match GridAreaNameFacts::from_specified_areas(style.grid_template_areas) {
            Ok(facts) => Some(facts),
            Err(error) => {
                report.push_error(error);
                None
            }
        }
    };

    let inherited_area_facts = inherited_subgrid_area_facts(parent_context);
    let style_area_facts = clamp_style_area_facts_to_subgrid_axes(style_area_facts, parent_context);
    let merged_area_facts = merge_subgrid_area_facts(
        style_area_facts.clone(),
        inherited_area_facts,
        parent_context,
    );

    let columns = if let Some(parent_axis) = &parent_context.columns {
        let mut columns = inherited_subgrid_axis_named_lines(
            GridAxisKind::Column,
            parent_axis,
            style.grid_template_columns,
            None,
        )?;
        if let Some(facts) = &merged_area_facts {
            columns =
                add_area_generated_lines_from_facts(GridAxisKind::Column, columns, facts.clone());
        }
        columns
    } else {
        let mut columns = named_lines_from_track_components(
            GridAxisKind::Column,
            style.grid_template_columns,
            explicit_columns,
        )?;
        if let Some(facts) = &style_area_facts {
            columns =
                add_area_generated_lines_from_facts(GridAxisKind::Column, columns, facts.clone());
        }
        columns
    };
    let rows = if let Some(parent_axis) = &parent_context.rows {
        let mut rows = inherited_subgrid_axis_named_lines(
            GridAxisKind::Row,
            parent_axis,
            style.grid_template_rows,
            None,
        )?;
        if let Some(facts) = &merged_area_facts {
            rows = add_area_generated_lines_from_facts(GridAxisKind::Row, rows, facts.clone());
        }
        rows
    } else {
        let mut rows = named_lines_from_track_components(
            GridAxisKind::Row,
            style.grid_template_rows,
            explicit_rows,
        )?;
        if let Some(facts) = &style_area_facts {
            rows = add_area_generated_lines_from_facts(GridAxisKind::Row, rows, facts.clone());
        }
        rows
    };

    Ok((
        GridNamedContext {
            columns,
            rows,
            area_facts: merged_area_facts,
        },
        report,
    ))
}

pub(super) fn empty_grid_named_context(
    explicit_columns: usize,
    explicit_rows: usize,
) -> GridNamedContext {
    GridNamedContext {
        columns: NamedGridLines::new(GridAxisKind::Column, explicit_columns),
        rows: NamedGridLines::new(GridAxisKind::Row, explicit_rows),
        area_facts: None,
    }
}

fn inherited_subgrid_axis_named_lines<S: LayoutScalar>(
    axis: GridAxisKind,
    parent_axis: &InheritedGridAxis<S>,
    components: &[TrackComponentOf<S>],
    parent_area_facts: Option<&GridAreaNameFacts>,
) -> Result<NamedGridLines, NamedGridError> {
    let used_track_count = parent_axis.tracks.len();
    let local_line_names = expand_subgrid_local_line_names(
        axis,
        used_track_count,
        subgrid_line_name_components(components).unwrap_or(&[]),
    )?;
    inherit_subgrid_named_lines(
        &parent_axis.named_lines,
        parent_axis.parent_start + 1,
        parent_axis.parent_end + 1,
        parent_axis.reversed,
        &local_line_names,
        parent_area_facts,
    )
}

fn subgrid_line_name_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Option<&[SubgridLineNameComponent]> {
    components.iter().find_map(|component| match component {
        TrackComponentOf::Subgrid(subgrid) => Some(subgrid.name_components.as_slice()),
        TrackComponentOf::LineNames(_)
        | TrackComponentOf::Track(_)
        | TrackComponentOf::Repeat(_) => None,
    })
}

pub(super) fn named_lines_from_track_components<S: LayoutScalar>(
    axis: GridAxisKind,
    components: &[TrackComponentOf<S>],
    explicit_track_count: usize,
) -> Result<NamedGridLines, NamedGridError> {
    validate_track_component_line_names(components)?;

    if components
        .iter()
        .any(|component| matches!(component, TrackComponentOf::Subgrid(_)))
    {
        return Ok(NamedGridLines::new(axis, explicit_track_count));
    }

    let mut lines = NamedGridLines::new(axis, explicit_track_count);
    let mut current_line = 0;
    append_track_component_names(
        &mut lines,
        components,
        explicit_track_count,
        &mut current_line,
    )?;
    lines.ensure_track_count(current_line);
    Ok(lines)
}

pub(super) fn expand_subgrid_local_line_names(
    axis: GridAxisKind,
    used_track_count: usize,
    components: &[SubgridLineNameComponent],
) -> Result<Vec<Vec<LineNameEntry>>, NamedGridError> {
    let slot_count = used_track_count + 1;
    let mut auto_fill_count = 0;

    for component in components {
        match component {
            SubgridLineNameComponent::LineNames(names) => validate_line_names(names)?,
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::Count(count),
                line_name_sets,
            } => {
                if *count == 0 {
                    return Err(NamedGridError::ZeroRepeat { axis });
                }
                validate_line_name_sets(line_name_sets)?;
            }
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                line_name_sets,
            } => {
                auto_fill_count += 1;
                if auto_fill_count > 1 {
                    return Err(NamedGridError::MultipleAutoFillRepeats { axis });
                }
                validate_line_name_sets(line_name_sets)?;
            }
        }
    }

    let mut local_line_names = Vec::with_capacity(slot_count);
    for (index, component) in components.iter().enumerate() {
        match component {
            SubgridLineNameComponent::LineNames(names) => {
                push_subgrid_line_name_slot(&mut local_line_names, slot_count, names);
            }
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::Count(count),
                line_name_sets,
            } => {
                for _ in 0..*count {
                    for names in line_name_sets {
                        push_subgrid_line_name_slot(&mut local_line_names, slot_count, names);
                    }
                }
            }
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                line_name_sets,
            } => {
                let trailing_fixed_slots = fixed_subgrid_slots_after(&components[index + 1..]);
                while local_line_names.len() + trailing_fixed_slots < slot_count {
                    for names in line_name_sets {
                        if local_line_names.len() + trailing_fixed_slots >= slot_count {
                            break;
                        }
                        push_subgrid_line_name_slot(&mut local_line_names, slot_count, names);
                    }
                    if line_name_sets.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    local_line_names.resize_with(slot_count, Vec::new);
    Ok(local_line_names)
}

pub(super) fn inherit_subgrid_named_lines(
    parent: &NamedGridLines,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
    local_line_names: &[Vec<LineNameEntry>],
    parent_area_facts: Option<&GridAreaNameFacts>,
) -> Result<NamedGridLines, NamedGridError> {
    let span_len = validate_subgrid_parent_span(parent, parent_start, parent_end)?;
    if local_line_names.len() != span_len + 1 {
        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis: parent.axis });
    }
    for entries in local_line_names {
        for entry in entries {
            validate_line_name(&entry.name)?;
        }
    }

    let mut lines = NamedGridLines::new(parent.axis, span_len);
    for local_line in 0..=span_len {
        let parent_line = subgrid_parent_line(parent_start, parent_end, local_line, reversed);
        lines.line_names[local_line].extend(
            parent.line_names[parent_line - 1]
                .iter()
                .filter(|entry| entry.origin != LineNameOrigin::AreaGenerated)
                .map(|entry| LineNameEntry {
                    name: entry.name.clone(),
                    origin: LineNameOrigin::Inherited,
                }),
        );
    }

    if let Some(facts) = parent_area_facts {
        add_clipped_subgrid_area_names(&mut lines, parent_start, parent_end, reversed, facts);
    }

    for (line_index, entries) in local_line_names.iter().enumerate() {
        lines.line_names[line_index].extend(entries.iter().cloned());
    }

    Ok(lines)
}

fn inherited_subgrid_area_facts<S: LayoutScalar>(
    parent_context: &GridParentContext<S>,
) -> Option<GridAreaNameFacts> {
    let source = parent_context
        .columns
        .as_ref()
        .and_then(|axis| axis.area_facts.as_ref())
        .or_else(|| {
            parent_context
                .rows
                .as_ref()
                .and_then(|axis| axis.area_facts.as_ref())
        })?;
    let facts = clip_subgrid_area_facts(
        source,
        parent_context.columns.as_ref(),
        parent_context.rows.as_ref(),
    );
    (facts.columns_valid || facts.rows_valid).then_some(facts)
}

fn merge_subgrid_area_facts<S: LayoutScalar>(
    local: Option<GridAreaNameFacts>,
    inherited: Option<GridAreaNameFacts>,
    parent_context: &GridParentContext<S>,
) -> Option<GridAreaNameFacts> {
    match (local, inherited) {
        (None, None) => None,
        (Some(local), None) => Some(local),
        (None, Some(inherited)) => Some(inherited),
        (Some(mut local), Some(inherited)) => {
            let mut inherited_rectangles = inherited.area_rectangles;
            for name in inherited.area_order {
                if let Some(local_index) = local
                    .area_order
                    .iter()
                    .position(|local_name| local_name == &name)
                {
                    let Some(inherited_index) = inherited_rectangles
                        .iter()
                        .position(|rectangle| rectangle.name == name)
                    else {
                        continue;
                    };
                    merge_duplicate_subgrid_area_rectangle(
                        &mut local.area_rectangles[local_index],
                        &inherited_rectangles[inherited_index],
                        parent_context,
                    );
                    continue;
                }
                let Some(rectangle_index) = inherited_rectangles
                    .iter()
                    .position(|rectangle| rectangle.name == name)
                else {
                    continue;
                };
                local.area_order.push(name);
                local
                    .area_rectangles
                    .push(inherited_rectangles.remove(rectangle_index));
            }

            local.row_count = local.row_count.max(inherited.row_count);
            local.column_count = local.column_count.max(inherited.column_count);
            local.rows_valid |= inherited.rows_valid;
            local.columns_valid |= inherited.columns_valid;
            Some(local)
        }
    }
}

fn clamp_style_area_facts_to_subgrid_axes<S: LayoutScalar>(
    facts: Option<GridAreaNameFacts>,
    parent_context: &GridParentContext<S>,
) -> Option<GridAreaNameFacts> {
    let mut facts = facts?;

    let column_boundary = parent_context
        .columns
        .as_ref()
        .map(|axis| axis.tracks.len() + 1);
    let row_boundary = parent_context
        .rows
        .as_ref()
        .map(|axis| axis.tracks.len() + 1);

    if column_boundary.is_none() && row_boundary.is_none() {
        return Some(facts);
    }

    for rectangle in &mut facts.area_rectangles {
        if let Some(boundary) = column_boundary {
            clamp_area_axis_to_local_boundary(
                &mut rectangle.column_start,
                &mut rectangle.column_end,
                &mut rectangle.column_start_name,
                &mut rectangle.column_end_name,
                boundary,
            );
        }
        if let Some(boundary) = row_boundary {
            clamp_area_axis_to_local_boundary(
                &mut rectangle.row_start,
                &mut rectangle.row_end,
                &mut rectangle.row_start_name,
                &mut rectangle.row_end_name,
                boundary,
            );
        }
    }

    if let Some(boundary) = column_boundary {
        facts.column_count = facts.column_count.min(boundary.saturating_sub(1));
    }
    if let Some(boundary) = row_boundary {
        facts.row_count = facts.row_count.min(boundary.saturating_sub(1));
    }

    Some(facts)
}

fn clamp_area_axis_to_local_boundary(
    start: &mut usize,
    end: &mut usize,
    start_name: &mut usize,
    end_name: &mut usize,
    boundary: usize,
) {
    *start = (*start).clamp(1, boundary);
    *end = (*end).clamp(1, boundary);
    *start_name = (*start_name).clamp(1, boundary);
    *end_name = (*end_name).clamp(1, boundary);
}

fn merge_duplicate_subgrid_area_rectangle<S: LayoutScalar>(
    local: &mut GridAreaNameRectangle,
    inherited: &GridAreaNameRectangle,
    parent_context: &GridParentContext<S>,
) {
    if parent_context.columns.is_some() {
        local.column_start = local.column_start.min(inherited.column_start);
        local.column_end = local.column_end.min(inherited.column_end);
        local.column_start_name = local.column_start_name.min(inherited.column_start_name);
        local.column_end_name = local.column_end_name.min(inherited.column_end_name);
    }
    if parent_context.rows.is_some() {
        local.row_start = local.row_start.min(inherited.row_start);
        local.row_end = local.row_end.min(inherited.row_end);
        local.row_start_name = local.row_start_name.min(inherited.row_start_name);
        local.row_end_name = local.row_end_name.min(inherited.row_end_name);
    }
}

fn clip_subgrid_area_facts<S: LayoutScalar>(
    facts: &GridAreaNameFacts,
    columns: Option<&InheritedGridAxis<S>>,
    rows: Option<&InheritedGridAxis<S>>,
) -> GridAreaNameFacts {
    let columns_valid = columns.is_some() && facts.columns_valid;
    let rows_valid = rows.is_some() && facts.rows_valid;
    let column_count = columns
        .map(|axis| axis.parent_end.saturating_sub(axis.parent_start))
        .unwrap_or(0);
    let row_count = rows
        .map(|axis| axis.parent_end.saturating_sub(axis.parent_start))
        .unwrap_or(0);
    let mut area_order = Vec::new();
    let mut area_rectangles = Vec::new();

    for name in &facts.area_order {
        let Some(rectangle) = facts
            .area_rectangles
            .iter()
            .find(|rectangle| rectangle.name == *name)
        else {
            continue;
        };
        let Some(column) = clip_area_axis_lines(
            rectangle.column_start,
            rectangle.column_end,
            columns_valid,
            columns.map(|axis| (axis.parent_start + 1, axis.parent_end + 1, axis.reversed)),
        ) else {
            continue;
        };
        let Some(row) = clip_area_axis_lines(
            rectangle.row_start,
            rectangle.row_end,
            rows_valid,
            rows.map(|axis| (axis.parent_start + 1, axis.parent_end + 1, axis.reversed)),
        ) else {
            continue;
        };
        area_order.push(name.clone());
        area_rectangles.push(GridAreaNameRectangle {
            name: name.clone(),
            row_start: row.start,
            row_end: row.end,
            column_start: column.start,
            column_end: column.end,
            row_start_name: row.start_name,
            row_end_name: row.end_name,
            column_start_name: column.start_name,
            column_end_name: column.end_name,
        });
    }

    GridAreaNameFacts {
        row_count,
        column_count,
        rows_valid,
        columns_valid,
        area_order,
        area_rectangles,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClippedAreaAxisLines {
    start: usize,
    end: usize,
    start_name: usize,
    end_name: usize,
}

fn clip_area_axis_lines(
    start: usize,
    end: usize,
    axis_valid: bool,
    clip: Option<(usize, usize, bool)>,
) -> Option<ClippedAreaAxisLines> {
    if !axis_valid {
        return Some(ClippedAreaAxisLines {
            start: 1,
            end: 1,
            start_name: 1,
            end_name: 1,
        });
    }
    let Some((boundary_start, boundary_end, reversed)) = clip else {
        return Some(ClippedAreaAxisLines {
            start,
            end,
            start_name: start,
            end_name: end,
        });
    };
    let clipped_start = start.max(boundary_start);
    let clipped_end = end.min(boundary_end);
    if clipped_end <= clipped_start {
        return None;
    }
    if reversed {
        Some(ClippedAreaAxisLines {
            start: boundary_end - clipped_end + 1,
            end: boundary_end - clipped_start + 1,
            start_name: boundary_end - clipped_start + 1,
            end_name: boundary_end - clipped_end + 1,
        })
    } else {
        Some(ClippedAreaAxisLines {
            start: clipped_start - boundary_start + 1,
            end: clipped_end - boundary_start + 1,
            start_name: clipped_start - boundary_start + 1,
            end_name: clipped_end - boundary_start + 1,
        })
    }
}

#[cfg(test)]
pub(super) fn add_area_generated_lines(
    axis: GridAxisKind,
    base: NamedGridLines,
    areas: &GridTemplateAreas,
) -> Result<NamedGridLines, NamedGridError> {
    if areas.rows.is_empty() {
        return Ok(base);
    }

    let facts = GridAreaNameFacts::from_specified_areas(areas)?;
    Ok(add_area_generated_lines_from_facts(axis, base, facts))
}

fn add_area_generated_lines_from_facts(
    axis: GridAxisKind,
    mut base: NamedGridLines,
    facts: GridAreaNameFacts,
) -> NamedGridLines {
    if !facts.is_valid_for_axis(axis) {
        base.area_facts = facts;
        return base;
    }
    let area_track_count = match axis {
        GridAxisKind::Column => facts.column_count,
        GridAxisKind::Row => facts.row_count,
    };
    base.ensure_track_count(area_track_count);

    for rectangle in &facts.area_rectangles {
        let (start_line, end_line) = match axis {
            GridAxisKind::Column => (rectangle.column_start_name, rectangle.column_end_name),
            GridAxisKind::Row => (rectangle.row_start_name, rectangle.row_end_name),
        };
        base.add_area_name(start_line - 1, format!("{}-start", rectangle.name));
        base.add_area_name(end_line - 1, format!("{}-end", rectangle.name));
    }

    base.area_facts = facts;
    base
}

fn append_track_component_names<S: LayoutScalar>(
    lines: &mut NamedGridLines,
    components: &[TrackComponentOf<S>],
    explicit_track_count: usize,
    current_line: &mut usize,
) -> Result<(), NamedGridError> {
    let auto_repeat_count =
        auto_repeat_expansion_count(lines.axis, components, explicit_track_count)?;

    for component in components {
        match component {
            TrackComponentOf::LineNames(names) => {
                lines.add_line_names(*current_line, names, LineNameOrigin::Explicit);
            }
            TrackComponentOf::Track(_) => {
                *current_line += 1;
                lines.ensure_track_count(*current_line);
            }
            TrackComponentOf::Repeat(repetition) => {
                let repeated_track_count = fixed_track_count(lines.axis, repetition.components())?;
                let count = match repetition.repeat() {
                    TrackRepeat::Count(count) => count.get(),
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit => auto_repeat_count.unwrap_or(0),
                };
                for _ in 0..count {
                    append_track_component_names(
                        lines,
                        repetition.components(),
                        repeated_track_count,
                        current_line,
                    )?;
                }
            }
            TrackComponentOf::Subgrid(_) => {}
        }
    }

    Ok(())
}

fn validate_track_component_line_names<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Result<(), NamedGridError> {
    for component in components {
        match component {
            TrackComponentOf::LineNames(names) => validate_line_names(names)?,
            TrackComponentOf::Repeat(repetition) => {
                validate_track_component_line_names(repetition.components())?;
            }
            TrackComponentOf::Track(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    Ok(())
}

fn validate_line_names(names: &[String]) -> Result<(), NamedGridError> {
    for name in names {
        if matches!(name.as_str(), "auto" | "span") {
            return Err(NamedGridError::ReservedLineName { name: name.clone() });
        }
    }
    Ok(())
}

fn validate_line_name_sets(line_name_sets: &[Vec<String>]) -> Result<(), NamedGridError> {
    for names in line_name_sets {
        validate_line_names(names)?;
    }
    Ok(())
}

fn push_subgrid_line_name_slot(
    line_names: &mut Vec<Vec<LineNameEntry>>,
    slot_count: usize,
    names: &[String],
) {
    if line_names.len() < slot_count {
        line_names.push(
            names
                .iter()
                .cloned()
                .map(|name| LineNameEntry {
                    name,
                    origin: LineNameOrigin::LocalSubgrid,
                })
                .collect(),
        );
    }
}

fn fixed_subgrid_slots_after(components: &[SubgridLineNameComponent]) -> usize {
    components
        .iter()
        .map(|component| match component {
            SubgridLineNameComponent::LineNames(_) => 1,
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::Count(count),
                line_name_sets,
            } => count * line_name_sets.len(),
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                ..
            } => 0,
        })
        .sum()
}

fn validate_subgrid_parent_span(
    parent: &NamedGridLines,
    parent_start: usize,
    parent_end: usize,
) -> Result<usize, NamedGridError> {
    if parent_start == 0
        || parent_end <= parent_start
        || parent_end > parent.explicit_track_count + 1
    {
        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis: parent.axis });
    }
    Ok(parent_end - parent_start)
}

fn subgrid_parent_line(
    parent_start: usize,
    parent_end: usize,
    local_line: usize,
    reversed: bool,
) -> usize {
    if reversed {
        parent_end - local_line
    } else {
        parent_start + local_line
    }
}

fn add_clipped_subgrid_area_names(
    lines: &mut NamedGridLines,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
    facts: &GridAreaNameFacts,
) {
    if !facts.is_valid_for_axis(lines.axis) {
        return;
    }
    for area in &facts.area_order {
        let Some(rectangle) = facts
            .area_rectangles
            .iter()
            .find(|rectangle| rectangle.name == *area)
        else {
            continue;
        };
        let (area_start, area_end, area_start_name, area_end_name) = match lines.axis {
            GridAxisKind::Column => (
                rectangle.column_start,
                rectangle.column_end,
                rectangle.column_start_name,
                rectangle.column_end_name,
            ),
            GridAxisKind::Row => (
                rectangle.row_start,
                rectangle.row_end,
                rectangle.row_start_name,
                rectangle.row_end_name,
            ),
        };
        let clipped_start = area_start.max(parent_start);
        let clipped_end = area_end.min(parent_end);
        if clipped_end <= clipped_start {
            continue;
        }

        let start_edge = clipped_semantic_area_edge(
            clipped_start,
            clipped_end,
            area_start_name,
            area_end_name,
            true,
        );
        let end_edge = clipped_semantic_area_edge(
            clipped_start,
            clipped_end,
            area_start_name,
            area_end_name,
            false,
        );
        let start_index = subgrid_local_line_index(parent_start, parent_end, start_edge, reversed);
        let end_index = subgrid_local_line_index(parent_start, parent_end, end_edge, reversed);
        lines.line_names[start_index].push(LineNameEntry {
            name: format!("{area}-start"),
            origin: LineNameOrigin::AreaGenerated,
        });
        lines.line_names[end_index].push(LineNameEntry {
            name: format!("{area}-end"),
            origin: LineNameOrigin::AreaGenerated,
        });
    }
}

fn clipped_semantic_area_edge(
    clipped_start: usize,
    clipped_end: usize,
    area_start_name: usize,
    area_end_name: usize,
    start_edge: bool,
) -> usize {
    let forward = area_start_name <= area_end_name;
    match (forward, start_edge) {
        (true, true) | (false, false) => clipped_start,
        (true, false) | (false, true) => clipped_end,
    }
}

fn subgrid_local_line_index(
    parent_start: usize,
    parent_end: usize,
    parent_line: usize,
    reversed: bool,
) -> usize {
    if reversed {
        parent_end - parent_line
    } else {
        parent_line - parent_start
    }
}

fn auto_repeat_expansion_count<S: LayoutScalar>(
    axis: GridAxisKind,
    components: &[TrackComponentOf<S>],
    explicit_track_count: usize,
) -> Result<Option<usize>, NamedGridError> {
    let mut fixed_tracks = 0;
    let mut auto_repeated_tracks = None;

    for component in components {
        match component {
            TrackComponentOf::Track(_) => fixed_tracks += 1,
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
            TrackComponentOf::Repeat(repetition) => match repetition.repeat() {
                TrackRepeat::Count(count) => {
                    fixed_tracks += fixed_track_count(axis, repetition.components())? * count.get();
                }
                TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                    if auto_repeated_tracks.is_some() {
                        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis });
                    }
                    auto_repeated_tracks = Some(fixed_track_count(axis, repetition.components())?);
                }
            },
        }
    }

    let Some(auto_repeated_tracks) = auto_repeated_tracks else {
        return Ok(None);
    };
    if auto_repeated_tracks == 0 || explicit_track_count < fixed_tracks {
        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis });
    }

    let remaining_tracks = explicit_track_count - fixed_tracks;
    if !remaining_tracks.is_multiple_of(auto_repeated_tracks) {
        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis });
    }

    Ok(Some(remaining_tracks / auto_repeated_tracks))
}

fn fixed_track_count<S: LayoutScalar>(
    axis: GridAxisKind,
    components: &[TrackComponentOf<S>],
) -> Result<usize, NamedGridError> {
    let mut count = 0;
    for component in components {
        match component {
            TrackComponentOf::Track(_) => count += 1,
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
            TrackComponentOf::Repeat(repetition) => match repetition.repeat() {
                TrackRepeat::Count(repeat_count) => {
                    count += fixed_track_count(axis, repetition.components())? * repeat_count.get();
                }
                TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                    return Err(NamedGridError::UnresolvedAutoRepeatNames { axis });
                }
            },
        }
    }
    Ok(count)
}
