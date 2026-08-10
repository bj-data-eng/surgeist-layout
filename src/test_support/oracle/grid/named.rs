use super::placement::{AxisPlacement, GridArea, GridAxis, PlacementError};
use super::subgrid::TrackSpan;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedGridLines {
    pub axis: GridAxis,
    pub explicit_track_count: usize,
    pub line_names: Vec<Vec<LineNameEntry>>,
}

impl NamedGridLines {
    pub fn new<N, L>(
        axis: GridAxis,
        explicit_track_count: usize,
        line_names: Vec<L>,
    ) -> Result<Self, NamedGridError>
    where
        N: Into<String>,
        L: IntoIterator<Item = N>,
    {
        let line_count = line_names.len();
        if line_count != explicit_track_count + 1 {
            return Err(NamedGridError::LineNameCountMismatch {
                axis,
                explicit_track_count,
                line_count,
            });
        }

        let mut entries = Vec::with_capacity(line_count);
        for line in line_names {
            let mut line_entries = Vec::new();
            for name in line {
                let name = name.into();
                if is_reserved_line_name(&name) {
                    return Err(NamedGridError::ReservedLineName { name });
                }
                line_entries.push(LineNameEntry {
                    name,
                    origin: LineNameOrigin::Explicit,
                });
            }
            entries.push(line_entries);
        }

        Ok(Self {
            axis,
            explicit_track_count,
            line_names: entries,
        })
    }

    #[must_use]
    pub fn empty(axis: GridAxis, explicit_track_count: usize) -> Self {
        Self {
            axis,
            explicit_track_count,
            line_names: vec![Vec::new(); explicit_track_count + 1],
        }
    }

    #[must_use]
    pub fn named_occurrences(&self, name: &str) -> Vec<usize> {
        self.line_names
            .iter()
            .enumerate()
            .flat_map(|(index, entries)| {
                entries
                    .iter()
                    .filter(move |entry| entry.name == name)
                    .map(move |_| index + 1)
            })
            .collect()
    }

    #[must_use]
    pub fn line_names(&self, line: usize) -> Vec<&str> {
        if line == 0 {
            return Vec::new();
        }

        self.line_names
            .get(line - 1)
            .into_iter()
            .flatten()
            .map(|entry| entry.name.as_str())
            .collect()
    }
}

pub fn resolve_named_line(
    lines: &NamedGridLines,
    name: &str,
    occurrence: isize,
) -> Result<NamedLookupReport, NamedGridError> {
    if is_reserved_line_name(name) {
        return Err(NamedGridError::ReservedLineName {
            name: name.to_owned(),
        });
    }

    if occurrence == 0 {
        return Err(NamedGridError::ZeroLine);
    }

    let explicit_matches = explicit_matches(lines, name);
    let (resolved_line, implicit_lines_assumed_named) = if occurrence > 0 {
        resolve_forward_occurrence(lines.explicit_track_count, &explicit_matches, occurrence)
    } else {
        resolve_backward_occurrence(&explicit_matches, occurrence)
    };

    Ok(NamedLookupReport {
        axis: lines.axis,
        name: name.to_owned(),
        requested_occurrence: occurrence,
        resolved_line,
        explicit_matches,
        implicit_lines_assumed_named,
    })
}

pub fn resolve_numeric_line(
    lines: &NamedGridLines,
    raw_line: isize,
) -> Result<isize, NamedGridError> {
    match raw_line {
        0 => Err(NamedGridError::ZeroLine),
        line if line > 0 => Ok(line),
        line => Ok(lines.explicit_track_count as isize + 2 + line),
    }
}

pub fn resolve_named_span_from_start(
    lines: &NamedGridLines,
    start_line: isize,
    name: &str,
    count: usize,
) -> Result<NamedLookupReport, NamedGridError> {
    resolve_named_span(lines, start_line, name, count, SpanSearchDirection::Forward)
}

pub fn resolve_named_span_from_end(
    lines: &NamedGridLines,
    end_line: isize,
    name: &str,
    count: usize,
) -> Result<NamedLookupReport, NamedGridError> {
    resolve_named_span(lines, end_line, name, count, SpanSearchDirection::Backward)
}

pub fn resolve_anonymous_span_from_start(
    start_line: isize,
    count: usize,
) -> Result<isize, NamedGridError> {
    if count == 0 {
        return Err(NamedGridError::ZeroSpan);
    }

    Ok(start_line + count as isize)
}

pub fn resolve_anonymous_span_from_end(
    end_line: isize,
    count: usize,
) -> Result<isize, NamedGridError> {
    if count == 0 {
        return Err(NamedGridError::ZeroSpan);
    }

    Ok(end_line - count as isize)
}

pub fn expand_named_fixed_repeat<I>(
    axis: GridAxis,
    repeat_count: usize,
    components: I,
) -> Result<NamedGridLines, NamedGridError>
where
    I: IntoIterator<Item = NamedTrackComponent>,
{
    if repeat_count == 0 {
        return Err(NamedGridError::ZeroRepeat);
    }

    let components: Vec<NamedTrackComponent> = components.into_iter().collect();
    let explicit_track_count = components
        .iter()
        .filter(|component| matches!(component, NamedTrackComponent::Track))
        .count()
        * repeat_count;
    let mut line_names = vec![Vec::new()];
    let mut current_line = 0;

    for _ in 0..repeat_count {
        for component in &components {
            match component {
                NamedTrackComponent::LineNames(names) => {
                    for name in names {
                        if is_reserved_line_name(name) {
                            return Err(NamedGridError::ReservedLineName { name: name.clone() });
                        }
                        line_names[current_line].push(LineNameEntry {
                            name: name.clone(),
                            origin: LineNameOrigin::Explicit,
                        });
                    }
                }
                NamedTrackComponent::Track => {
                    current_line += 1;
                    if line_names.len() <= current_line {
                        line_names.push(Vec::new());
                    }
                }
            }
        }
    }

    line_names.resize_with(explicit_track_count + 1, Vec::new);

    Ok(NamedGridLines {
        axis,
        explicit_track_count,
        line_names,
    })
}

pub fn area_generated_lines(
    axis: GridAxis,
    areas: &TemplateAreas,
    base: NamedGridLines,
) -> Result<NamedGridLines, NamedGridError> {
    if base.axis != axis {
        return Err(NamedGridError::LineNameCountMismatch {
            axis,
            explicit_track_count: base.explicit_track_count,
            line_count: base.line_names.len(),
        });
    }

    let explicit_track_count = base.explicit_track_count.max(areas.axis_track_count(axis));
    let mut line_names = base.line_names;
    line_names.resize_with(explicit_track_count + 1, Vec::new);

    for area in &areas.area_order {
        let rect = areas
            .area_rectangles
            .get(area)
            .expect("area order only contains known rectangles");
        let (start_line, end_line) = match axis {
            GridAxis::Column => (rect.column_start, rect.column_end),
            GridAxis::Row => (rect.row_start, rect.row_end),
        };
        line_names[start_line - 1].push(LineNameEntry {
            name: format!("{area}-start"),
            origin: LineNameOrigin::AreaGenerated,
        });
        line_names[end_line - 1].push(LineNameEntry {
            name: format!("{area}-end"),
            origin: LineNameOrigin::AreaGenerated,
        });
    }

    Ok(NamedGridLines {
        axis,
        explicit_track_count,
        line_names,
    })
}

pub fn area_generated_facts(
    areas: &TemplateAreas,
    base_columns: NamedGridLines,
    base_rows: NamedGridLines,
) -> Result<AreaGeneratedFacts, NamedGridError> {
    let columns = area_generated_lines(GridAxis::Column, areas, base_columns)?;
    let rows = area_generated_lines(GridAxis::Row, areas, base_rows)?;

    Ok(AreaGeneratedFacts {
        areas: areas.clone(),
        columns,
        rows,
    })
}

pub fn resolve_named_area(
    areas: &TemplateAreas,
    area_name: &str,
) -> Result<NamedGridAreaPlacement, NamedGridError> {
    if !areas.contains_area(area_name) {
        return Err(NamedGridError::AreaNotFound {
            area: area_name.to_owned(),
        });
    }

    Ok(NamedGridAreaPlacement {
        row: NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: format!("{area_name}-start"),
                occurrence: 1,
            },
            end: NamedGridLine::Named {
                name: format!("{area_name}-end"),
                occurrence: 1,
            },
        },
        column: NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: format!("{area_name}-start"),
                occurrence: 1,
            },
            end: NamedGridLine::Named {
                name: format!("{area_name}-end"),
                occurrence: 1,
            },
        },
    })
}

pub fn resolve_named_grid_area_report(
    columns: &NamedGridLines,
    rows: &NamedGridLines,
    area_name: &str,
) -> Result<NamedGridAreaResolutionReport, NamedGridError> {
    let column = resolve_named_axis_placement(
        columns,
        NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: format!("{area_name}-start"),
                occurrence: 1,
            },
            end: NamedGridLine::Named {
                name: format!("{area_name}-end"),
                occurrence: 1,
            },
        },
        None,
    )?;
    let row = resolve_named_axis_placement(
        rows,
        NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: format!("{area_name}-start"),
                occurrence: 1,
            },
            end: NamedGridLine::Named {
                name: format!("{area_name}-end"),
                occurrence: 1,
            },
        },
        None,
    )?;

    let area = GridArea::new(
        column.resolved.start_line as usize,
        row.resolved.start_line as usize,
        column.resolved.span,
        row.resolved.span,
    );

    Ok(NamedGridAreaResolutionReport { area, row, column })
}

#[must_use]
pub fn expand_axis_shorthand(
    first: NamedGridLine,
    second: Option<NamedGridLine>,
) -> NamedAxisPlacement {
    let end = second.unwrap_or_else(|| {
        if matches!(first, NamedGridLine::BareIdent(_)) {
            first.clone()
        } else {
            NamedGridLine::Auto
        }
    });

    NamedAxisPlacement { start: first, end }
}

pub fn expand_grid_area_shorthand(
    values: Vec<NamedGridLine>,
) -> Result<NamedGridAreaPlacement, NamedGridError> {
    let expanded = match values.as_slice() {
        [row_start] => {
            let row_end = omitted_grid_area_side(row_start, row_start);
            let column_start = omitted_grid_area_side(row_start, row_start);
            let column_end = omitted_grid_area_side(row_start, row_start);
            (row_start.clone(), column_start, row_end, column_end)
        }
        [row_start, column_start] => {
            let row_end = omitted_grid_area_side(row_start, row_start);
            let column_end = omitted_grid_area_side(column_start, column_start);
            (row_start.clone(), column_start.clone(), row_end, column_end)
        }
        [row_start, column_start, row_end] => {
            let column_end = omitted_grid_area_side(column_start, column_start);
            (
                row_start.clone(),
                column_start.clone(),
                row_end.clone(),
                column_end,
            )
        }
        [row_start, column_start, row_end, column_end] => (
            row_start.clone(),
            column_start.clone(),
            row_end.clone(),
            column_end.clone(),
        ),
        _ => return Err(NamedGridError::InvalidGridAreaShorthandArity),
    };

    Ok(NamedGridAreaPlacement {
        row: NamedAxisPlacement {
            start: expanded.0,
            end: expanded.2,
        },
        column: NamedAxisPlacement {
            start: expanded.1,
            end: expanded.3,
        },
    })
}

pub fn expand_subgrid_name_list<I>(
    axis: GridAxis,
    used_track_count: usize,
    components: I,
) -> Result<SubgridNameExpansionReport, NamedGridError>
where
    I: IntoIterator<Item = SubgridNameComponent>,
{
    let components: Vec<SubgridNameComponent> = components.into_iter().collect();
    let slot_count = used_track_count + 1;
    let mut auto_fill_count = 0;

    for component in &components {
        match component {
            SubgridNameComponent::LineNames(names) => {
                validate_line_names(names)?;
            }
            SubgridNameComponent::Repeat {
                count: SubgridNameRepeatCount::Number(count),
                line_name_sets,
            } => {
                if *count == 0 {
                    return Err(NamedGridError::ZeroRepeat);
                }
                validate_line_name_sets(line_name_sets)?;
            }
            SubgridNameComponent::Repeat {
                count: SubgridNameRepeatCount::AutoFill,
                line_name_sets,
            } => {
                auto_fill_count += 1;
                if auto_fill_count > 1 {
                    return Err(NamedGridError::MultipleAutoFillRepeats);
                }
                validate_line_name_sets(line_name_sets)?;
            }
        }
    }

    let mut local_line_names = Vec::with_capacity(slot_count);
    for (index, component) in components.iter().enumerate() {
        match component {
            SubgridNameComponent::LineNames(names) => {
                push_line_name_slot(&mut local_line_names, slot_count, names.iter().cloned())
            }
            SubgridNameComponent::Repeat {
                count: SubgridNameRepeatCount::Number(count),
                line_name_sets,
            } => {
                for _ in 0..*count {
                    for names in line_name_sets {
                        push_line_name_slot(
                            &mut local_line_names,
                            slot_count,
                            names.iter().cloned(),
                        );
                    }
                }
            }
            SubgridNameComponent::Repeat {
                count: SubgridNameRepeatCount::AutoFill,
                line_name_sets,
            } => {
                let trailing_fixed_slots = fixed_slots_after(&components[index + 1..]);
                while local_line_names.len() + trailing_fixed_slots < slot_count {
                    for names in line_name_sets {
                        if local_line_names.len() + trailing_fixed_slots >= slot_count {
                            break;
                        }
                        push_line_name_slot(
                            &mut local_line_names,
                            slot_count,
                            names.iter().cloned(),
                        );
                    }
                    if line_name_sets.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    local_line_names.resize_with(slot_count, Vec::new);

    Ok(SubgridNameExpansionReport {
        axis,
        used_track_count,
        local_line_names,
    })
}

pub fn inherit_named_subgrid_lines(
    parent: &NamedGridLines,
    parent_span: TrackSpan,
    reversed: bool,
    local_line_names: Vec<Vec<String>>,
    parent_area_facts: Option<&AreaGeneratedFacts>,
) -> Result<SubgridLineNameInheritanceReport, NamedGridError> {
    let span_len = validate_parent_span(parent, parent_span)?;
    if local_line_names.len() != span_len + 1 {
        return Err(NamedGridError::LineNameCountMismatch {
            axis: parent.axis,
            explicit_track_count: span_len,
            line_count: local_line_names.len(),
        });
    }
    for names in &local_line_names {
        validate_line_names(names)?;
    }

    let inherited_line_names = inherited_parent_line_names(parent, parent_span, reversed);
    let local_grid = local_subgrid_line_names(parent.axis, span_len, local_line_names);
    let mut merged_line_names = inherited_line_names.clone();
    let clipped_area_sources = if let Some(facts) = parent_area_facts {
        let area_lines = clipped_area_line_names(parent.axis, parent_span, reversed, facts)?;
        for (line_index, names) in area_lines.line_names.into_iter().enumerate() {
            merged_line_names.line_names[line_index].extend(names);
        }
        clipped_area_sources(parent.axis, parent_span, facts)
    } else {
        BTreeMap::new()
    };
    for (line_index, names) in local_grid.line_names.iter().enumerate() {
        merged_line_names.line_names[line_index].extend(names.iter().cloned());
    }

    Ok(SubgridLineNameInheritanceReport {
        axis: parent.axis,
        parent_span,
        inherited_line_names,
        local_line_names: local_grid,
        lines: merged_line_names.clone(),
        merged_line_names,
        clipped_area_sources,
    })
}

pub fn resolve_named_subgrid_axis_placement(
    lines: &NamedGridLines,
    placement: NamedAxisPlacement,
    auto_cursor_line: Option<isize>,
) -> Result<SubgridAxisPlacementReport, NamedGridError> {
    let (clamped, unclamped_start_line, unclamped_end_line) =
        resolve_named_axis_placement_inner(lines, placement, auto_cursor_line, true)?;

    Ok(SubgridAxisPlacementReport {
        unclamped_start_line,
        unclamped_end_line,
        clamped,
    })
}

pub fn resolve_named_axis_placement(
    lines: &NamedGridLines,
    placement: NamedAxisPlacement,
    auto_cursor_line: Option<isize>,
) -> Result<NamedPlacementReport, NamedGridError> {
    resolve_named_axis_placement_inner(lines, placement, auto_cursor_line, false)
        .map(|(report, _, _)| report)
}

fn resolve_named_axis_placement_inner(
    lines: &NamedGridLines,
    placement: NamedAxisPlacement,
    auto_cursor_line: Option<isize>,
    clamp_to_explicit: bool,
) -> Result<(NamedPlacementReport, isize, isize), NamedGridError> {
    let original_start = placement.start;
    let original_end = placement.end;
    let mut normalized_start = original_start.clone();
    let mut normalized_end = original_end.clone();
    let mut conflict_resolution = None;
    let mut conflict_resolutions = Vec::new();

    if matches!(normalized_start, NamedGridLine::Span { .. })
        && matches!(normalized_end, NamedGridLine::Span { .. })
    {
        normalized_end = NamedGridLine::Auto;
        record_conflict(
            &mut conflict_resolution,
            &mut conflict_resolutions,
            NamedPlacementConflictResolution::DroppedEndSpan,
        );
    }

    if matches!(normalized_end, NamedGridLine::Auto) {
        default_lone_named_span_to_one(
            &mut normalized_start,
            &mut conflict_resolution,
            &mut conflict_resolutions,
        );
    }
    if matches!(normalized_start, NamedGridLine::Auto) {
        default_lone_named_span_to_one(
            &mut normalized_end,
            &mut conflict_resolution,
            &mut conflict_resolutions,
        );
    }

    let mut start_lookup = None;
    let mut end_lookup = None;
    let (mut start_line, mut end_line) = match (&normalized_start, &normalized_end) {
        (NamedGridLine::Auto, NamedGridLine::Auto) => {
            let start_line = auto_cursor_line.ok_or(NamedGridError::AutoWithoutCursor)?;
            (start_line, start_line + 1)
        }
        (NamedGridLine::Auto, NamedGridLine::Span { .. }) => {
            let start_line = auto_cursor_line.ok_or(NamedGridError::AutoWithoutCursor)?;
            let (end_line, lookup) = resolve_span_from_start(lines, start_line, &normalized_end)?;
            end_lookup = lookup;
            (start_line, end_line)
        }
        (NamedGridLine::Span { .. }, NamedGridLine::Auto) => {
            let start_line = auto_cursor_line.ok_or(NamedGridError::AutoWithoutCursor)?;
            let (end_line, lookup) = resolve_span_from_start(lines, start_line, &normalized_start)?;
            start_lookup = lookup;
            (start_line, end_line)
        }
        (NamedGridLine::Auto, end) if is_definite_line(end) => {
            let (end_line, lookup) = resolve_line(lines, end, PlacementSide::End)?;
            end_lookup = lookup;
            (end_line - 1, end_line)
        }
        (start, NamedGridLine::Auto) if is_definite_line(start) => {
            let (start_line, lookup) = resolve_line(lines, start, PlacementSide::Start)?;
            start_lookup = lookup;
            (start_line, start_line + 1)
        }
        (start, NamedGridLine::Span { .. }) if is_definite_line(start) => {
            let (start_line, lookup) = resolve_line(lines, start, PlacementSide::Start)?;
            start_lookup = lookup;
            let (end_line, lookup) = resolve_span_from_start(lines, start_line, &normalized_end)?;
            end_lookup = lookup;
            (start_line, end_line)
        }
        (NamedGridLine::Span { .. }, end) if is_definite_line(end) => {
            let (end_line, lookup) = resolve_line(lines, end, PlacementSide::End)?;
            end_lookup = lookup;
            let (start_line, lookup) = resolve_span_from_end(lines, end_line, &normalized_start)?;
            start_lookup = lookup;
            (start_line, end_line)
        }
        (start, end) if is_definite_line(start) && is_definite_line(end) => {
            let (start_line, lookup) = resolve_line(lines, start, PlacementSide::Start)?;
            start_lookup = lookup;
            let (end_line, lookup) = resolve_line(lines, end, PlacementSide::End)?;
            end_lookup = lookup;
            (start_line, end_line)
        }
        (NamedGridLine::Auto, _) | (_, NamedGridLine::Auto) => {
            return Err(NamedGridError::AutoWithoutCursor);
        }
        (NamedGridLine::Span { .. }, NamedGridLine::Span { .. }) => {
            unreachable!("span/span is normalized before resolution")
        }
        (NamedGridLine::Span { .. }, _) | (_, NamedGridLine::Span { .. }) => {
            unreachable!("span placement is handled when the opposite edge is resolvable")
        }
        (_, _) => unreachable!("all normalized named placement pairs are handled"),
    };

    if start_line > end_line {
        std::mem::swap(&mut start_line, &mut end_line);
        record_conflict(
            &mut conflict_resolution,
            &mut conflict_resolutions,
            NamedPlacementConflictResolution::SwappedResolvedLines,
        );
    } else if start_line == end_line {
        normalized_end = NamedGridLine::Span {
            name: None,
            count: 1,
        };
        end_lookup = None;
        end_line = start_line + 1;
        record_conflict(
            &mut conflict_resolution,
            &mut conflict_resolutions,
            NamedPlacementConflictResolution::DroppedEqualEndLine,
        );
    }

    let unclamped_start_line = start_line;
    let unclamped_end_line = end_line;
    if clamp_to_explicit {
        (start_line, end_line) =
            clamp_subgrid_resolved_lines(start_line, end_line, lines.explicit_track_count);
    }

    let resolved = AxisPlacement::try_new(start_line, end_line)
        .map_err(|err| map_placement_error(lines.axis, start_line, end_line, err))?;

    Ok((
        NamedPlacementReport {
            axis: lines.axis,
            original_start,
            original_end,
            normalized_start,
            normalized_end,
            conflict_resolution,
            conflict_resolutions,
            start_lookup,
            end_lookup,
            resolved,
        },
        unclamped_start_line,
        unclamped_end_line,
    ))
}

fn record_conflict(
    conflict_resolution: &mut Option<NamedPlacementConflictResolution>,
    conflict_resolutions: &mut Vec<NamedPlacementConflictResolution>,
    resolution: NamedPlacementConflictResolution,
) {
    if conflict_resolution.is_none() {
        *conflict_resolution = Some(resolution);
    }
    conflict_resolutions.push(resolution);
}

fn default_lone_named_span_to_one(
    line: &mut NamedGridLine,
    conflict_resolution: &mut Option<NamedPlacementConflictResolution>,
    conflict_resolutions: &mut Vec<NamedPlacementConflictResolution>,
) {
    if matches!(line, NamedGridLine::Span { name: Some(_), .. }) {
        *line = NamedGridLine::Span {
            name: None,
            count: 1,
        };
        record_conflict(
            conflict_resolution,
            conflict_resolutions,
            NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne,
        );
    }
}

fn is_definite_line(line: &NamedGridLine) -> bool {
    matches!(
        line,
        NamedGridLine::Number(_) | NamedGridLine::Named { .. } | NamedGridLine::BareIdent(_)
    )
}

fn resolve_line(
    lines: &NamedGridLines,
    line: &NamedGridLine,
    side: PlacementSide,
) -> Result<(isize, Option<NamedLookupReport>), NamedGridError> {
    match line {
        NamedGridLine::Number(raw_line) => Ok((resolve_numeric_line(lines, *raw_line)?, None)),
        NamedGridLine::Named { name, occurrence } => {
            let report = resolve_named_line(lines, name, *occurrence)?;
            Ok((report.resolved_line, Some(report)))
        }
        NamedGridLine::BareIdent(name) => {
            let report = resolve_bare_ident(lines, name, side)?;
            Ok((report.resolved_line, Some(report)))
        }
        NamedGridLine::Auto | NamedGridLine::Span { .. } => unreachable!("not a definite line"),
    }
}

fn resolve_bare_ident(
    lines: &NamedGridLines,
    name: &str,
    side: PlacementSide,
) -> Result<NamedLookupReport, NamedGridError> {
    if is_reserved_line_name(name) {
        return Err(NamedGridError::ReservedLineName {
            name: name.to_owned(),
        });
    }

    let preferred_name = match side {
        PlacementSide::Start => format!("{name}-start"),
        PlacementSide::End => format!("{name}-end"),
    };

    if lines.named_occurrences(&preferred_name).is_empty() {
        resolve_named_line(lines, name, 1)
    } else {
        resolve_named_line(lines, &preferred_name, 1)
    }
}

fn resolve_span_from_start(
    lines: &NamedGridLines,
    start_line: isize,
    span: &NamedGridLine,
) -> Result<(isize, Option<NamedLookupReport>), NamedGridError> {
    match span {
        NamedGridLine::Span {
            name: Some(name),
            count,
        } => {
            let report = resolve_named_span_from_start(lines, start_line, name, *count)?;
            Ok((report.resolved_line, Some(report)))
        }
        NamedGridLine::Span { name: None, count } => {
            Ok((resolve_anonymous_span_from_start(start_line, *count)?, None))
        }
        _ => unreachable!("not a span"),
    }
}

fn resolve_span_from_end(
    lines: &NamedGridLines,
    end_line: isize,
    span: &NamedGridLine,
) -> Result<(isize, Option<NamedLookupReport>), NamedGridError> {
    match span {
        NamedGridLine::Span {
            name: Some(name),
            count,
        } => {
            let report = resolve_named_span_from_end(lines, end_line, name, *count)?;
            Ok((report.resolved_line, Some(report)))
        }
        NamedGridLine::Span { name: None, count } => {
            Ok((resolve_anonymous_span_from_end(end_line, *count)?, None))
        }
        _ => unreachable!("not a span"),
    }
}

fn map_placement_error(
    axis: GridAxis,
    start_line: isize,
    end_line: isize,
    err: PlacementError,
) -> NamedGridError {
    match err {
        PlacementError::LineBeforeFirst => NamedGridError::LineBeforeFirst {
            axis,
            start_line,
            end_line,
        },
        PlacementError::EndBeforeStart => NamedGridError::LineBeforeFirst {
            axis,
            start_line,
            end_line,
        },
        PlacementError::ZeroSpan
        | PlacementError::NoExplicitTracks(_)
        | PlacementError::SpanExceedsExplicitTracks { .. } => {
            unreachable!("axis placement validation only checks resolved line bounds and order")
        }
    }
}

fn validate_line_names(names: &[String]) -> Result<(), NamedGridError> {
    for name in names {
        if is_reserved_line_name(name) {
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

fn push_line_name_slot<I>(line_names: &mut Vec<Vec<String>>, slot_count: usize, names: I)
where
    I: IntoIterator<Item = String>,
{
    if line_names.len() < slot_count {
        line_names.push(names.into_iter().collect());
    }
}

fn fixed_slots_after(components: &[SubgridNameComponent]) -> usize {
    components
        .iter()
        .map(|component| match component {
            SubgridNameComponent::LineNames(_) => 1,
            SubgridNameComponent::Repeat {
                count: SubgridNameRepeatCount::Number(count),
                line_name_sets,
            } => count * line_name_sets.len(),
            SubgridNameComponent::Repeat {
                count: SubgridNameRepeatCount::AutoFill,
                ..
            } => 0,
        })
        .sum()
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

fn omitted_grid_area_side(source: &NamedGridLine, opposite: &NamedGridLine) -> NamedGridLine {
    if matches!(source, NamedGridLine::BareIdent(_)) {
        opposite.clone()
    } else {
        NamedGridLine::Auto
    }
}

fn validate_parent_span(
    parent: &NamedGridLines,
    parent_span: TrackSpan,
) -> Result<usize, NamedGridError> {
    if parent_span.start == 0
        || parent_span.end <= parent_span.start
        || parent_span.end > parent.explicit_track_count + 1
    {
        return Err(NamedGridError::SubgridSpanOutOfRange { axis: parent.axis });
    }
    Ok(parent_span.end - parent_span.start)
}

fn inherited_parent_line_names(
    parent: &NamedGridLines,
    parent_span: TrackSpan,
    reversed: bool,
) -> NamedGridLines {
    let span_len = parent_span.end - parent_span.start;
    let mut line_names = Vec::with_capacity(span_len + 1);
    for local_line in 0..=span_len {
        let parent_line = if reversed {
            parent_span.end - local_line
        } else {
            parent_span.start + local_line
        };
        let names = parent.line_names[parent_line - 1]
            .iter()
            .filter(|entry| entry.origin == LineNameOrigin::Explicit)
            .cloned()
            .collect();
        line_names.push(names);
    }

    NamedGridLines {
        axis: parent.axis,
        explicit_track_count: span_len,
        line_names,
    }
}

fn local_subgrid_line_names(
    axis: GridAxis,
    explicit_track_count: usize,
    local_line_names: Vec<Vec<String>>,
) -> NamedGridLines {
    let line_names = local_line_names
        .into_iter()
        .map(|names| {
            names
                .into_iter()
                .map(|name| LineNameEntry {
                    name,
                    origin: LineNameOrigin::LocalSubgrid,
                })
                .collect()
        })
        .collect();

    NamedGridLines {
        axis,
        explicit_track_count,
        line_names,
    }
}

fn clipped_area_line_names(
    axis: GridAxis,
    parent_span: TrackSpan,
    reversed: bool,
    facts: &AreaGeneratedFacts,
) -> Result<NamedGridLines, NamedGridError> {
    let span_len = parent_span.end - parent_span.start;
    let mut line_names = vec![Vec::new(); span_len + 1];

    for area in &facts.areas.area_order {
        let Some(source) = clipped_area_source(axis, area, parent_span, facts) else {
            continue;
        };
        let start_index = local_subgrid_line_index(parent_span, source.parent_span.start, reversed);
        let end_index = local_subgrid_line_index(parent_span, source.parent_span.end, reversed);
        line_names[start_index].push(LineNameEntry {
            name: format!("{area}-start"),
            origin: LineNameOrigin::AreaGenerated,
        });
        line_names[end_index].push(LineNameEntry {
            name: format!("{area}-end"),
            origin: LineNameOrigin::AreaGenerated,
        });
    }

    Ok(NamedGridLines {
        axis,
        explicit_track_count: span_len,
        line_names,
    })
}

fn clipped_area_sources(
    axis: GridAxis,
    parent_span: TrackSpan,
    facts: &AreaGeneratedFacts,
) -> BTreeMap<String, ClippedAreaSource> {
    let mut sources = BTreeMap::new();
    for area in &facts.areas.area_order {
        if let Some(source) = clipped_area_source(axis, area, parent_span, facts) {
            sources.insert(area.clone(), source);
        }
    }
    sources
}

fn clipped_area_source(
    axis: GridAxis,
    area: &str,
    parent_span: TrackSpan,
    facts: &AreaGeneratedFacts,
) -> Option<ClippedAreaSource> {
    let rect = facts.areas.area_rectangle(area)?;
    let area_span = match axis {
        GridAxis::Column => TrackSpan::new(rect.column_start, rect.column_end),
        GridAxis::Row => TrackSpan::new(rect.row_start, rect.row_end),
    };
    let start = area_span.start.max(parent_span.start);
    let end = area_span.end.min(parent_span.end);
    if end <= start {
        return None;
    }

    Some(ClippedAreaSource {
        area: area.to_owned(),
        original_parent_span: area_span,
        parent_span: TrackSpan::new(start, end),
    })
}

fn local_subgrid_line_index(parent_span: TrackSpan, parent_line: usize, reversed: bool) -> usize {
    if reversed {
        parent_span.end - parent_line
    } else {
        parent_line - parent_span.start
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpanSearchDirection {
    Forward,
    Backward,
}

fn resolve_named_span(
    lines: &NamedGridLines,
    edge_line: isize,
    name: &str,
    count: usize,
    direction: SpanSearchDirection,
) -> Result<NamedLookupReport, NamedGridError> {
    if is_reserved_line_name(name) {
        return Err(NamedGridError::ReservedLineName {
            name: name.to_owned(),
        });
    }

    if count == 0 {
        return Err(NamedGridError::ZeroSpan);
    }

    let explicit_matches = explicit_matches(lines, name);
    let explicit_candidates: Vec<isize> = match direction {
        SpanSearchDirection::Forward => explicit_matches
            .iter()
            .copied()
            .filter(|line| *line > edge_line)
            .collect(),
        SpanSearchDirection::Backward => explicit_matches
            .iter()
            .rev()
            .copied()
            .filter(|line| *line < edge_line)
            .collect(),
    };
    let count = count as isize;
    let (resolved_line, implicit_lines_assumed_named) = match direction {
        SpanSearchDirection::Forward => resolve_forward_span(
            lines.explicit_track_count,
            edge_line,
            &explicit_candidates,
            count,
        ),
        SpanSearchDirection::Backward => {
            resolve_backward_span(edge_line, &explicit_candidates, count)
        }
    };

    Ok(NamedLookupReport {
        axis: lines.axis,
        name: name.to_owned(),
        requested_occurrence: count,
        resolved_line,
        explicit_matches,
        implicit_lines_assumed_named,
    })
}

fn explicit_matches(lines: &NamedGridLines, name: &str) -> Vec<isize> {
    lines
        .named_occurrences(name)
        .into_iter()
        .map(|line| line as isize)
        .collect()
}

fn resolve_forward_occurrence(
    explicit_track_count: usize,
    explicit_matches: &[isize],
    occurrence: isize,
) -> (isize, Vec<isize>) {
    let match_index = occurrence as usize - 1;
    if let Some(line) = explicit_matches.get(match_index) {
        return (*line, Vec::new());
    }

    let missing_count = occurrence - explicit_matches.len() as isize;
    let first_implicit_line = explicit_track_count as isize + 2;
    let implicit_lines_assumed_named = consecutive_lines(first_implicit_line, 1, missing_count);
    let resolved_line = *implicit_lines_assumed_named
        .last()
        .expect("positive missing occurrence produces implicit lines");

    (resolved_line, implicit_lines_assumed_named)
}

fn resolve_backward_occurrence(
    explicit_matches: &[isize],
    occurrence: isize,
) -> (isize, Vec<isize>) {
    let requested_count = -occurrence;
    if requested_count <= explicit_matches.len() as isize {
        let match_index = explicit_matches.len() - requested_count as usize;
        return (explicit_matches[match_index], Vec::new());
    }

    let missing_count = requested_count - explicit_matches.len() as isize;
    let implicit_lines_assumed_named = consecutive_lines(0, -1, missing_count);
    let resolved_line = *implicit_lines_assumed_named
        .last()
        .expect("negative missing occurrence produces implicit lines");

    (resolved_line, implicit_lines_assumed_named)
}

fn resolve_forward_span(
    explicit_track_count: usize,
    start_line: isize,
    explicit_candidates: &[isize],
    count: isize,
) -> (isize, Vec<isize>) {
    let match_index = count as usize - 1;
    if let Some(line) = explicit_candidates.get(match_index) {
        return (*line, Vec::new());
    }

    let missing_count = count - explicit_candidates.len() as isize;
    let first_implicit_line = (explicit_track_count as isize + 2).max(start_line + 1);
    let implicit_lines_assumed_named = consecutive_lines(first_implicit_line, 1, missing_count);
    let resolved_line = *implicit_lines_assumed_named
        .last()
        .expect("positive named span extension produces implicit lines");

    (resolved_line, implicit_lines_assumed_named)
}

fn resolve_backward_span(
    end_line: isize,
    explicit_candidates: &[isize],
    count: isize,
) -> (isize, Vec<isize>) {
    let match_index = count as usize - 1;
    if let Some(line) = explicit_candidates.get(match_index) {
        return (*line, Vec::new());
    }

    let missing_count = count - explicit_candidates.len() as isize;
    let first_implicit_line = (end_line - 1).min(0);
    let implicit_lines_assumed_named = consecutive_lines(first_implicit_line, -1, missing_count);
    let resolved_line = *implicit_lines_assumed_named
        .last()
        .expect("negative named span extension produces implicit lines");

    (resolved_line, implicit_lines_assumed_named)
}

fn consecutive_lines(start: isize, step: isize, count: isize) -> Vec<isize> {
    (0..count).map(|index| start + index * step).collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineNameEntry {
    pub name: String,
    pub origin: LineNameOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineNameOrigin {
    Explicit,
    AreaGenerated,
    LocalSubgrid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AreaGeneratedFacts {
    pub areas: TemplateAreas,
    pub columns: NamedGridLines,
    pub rows: NamedGridLines,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateAreas {
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub column_count: usize,
    pub area_rectangles: BTreeMap<String, AreaRectangle>,
    pub area_order: Vec<String>,
}

impl TemplateAreas {
    pub fn new<R, C, S>(rows: R) -> Result<Self, NamedGridError>
    where
        R: IntoIterator<Item = C>,
        C: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|row| row.into_iter().map(Into::into).collect())
            .collect();
        let row_count = rows.len();
        if row_count == 0 {
            return Err(NamedGridError::EmptyTemplateAreas);
        }

        let column_count = rows[0].len();
        if column_count == 0 {
            return Err(NamedGridError::TemplateAreaRowLengthMismatch {
                expected: 1,
                actual: 0,
                row: 1,
            });
        }

        for (index, row) in rows.iter().enumerate() {
            if row.len() != column_count {
                return Err(NamedGridError::TemplateAreaRowLengthMismatch {
                    expected: column_count,
                    actual: row.len(),
                    row: index + 1,
                });
            }
        }

        let mut area_order = Vec::new();
        let mut area_rectangles: BTreeMap<String, AreaRectangle> = BTreeMap::new();
        for (row_index, row) in rows.iter().enumerate() {
            for (column_index, token) in row.iter().enumerate() {
                if is_null_area_token(token) {
                    continue;
                }

                let entry = area_rectangles.entry(token.clone()).or_insert_with(|| {
                    area_order.push(token.clone());
                    AreaRectangle {
                        row_start: row_index + 1,
                        row_end: row_index + 2,
                        column_start: column_index + 1,
                        column_end: column_index + 2,
                    }
                });
                entry.row_start = entry.row_start.min(row_index + 1);
                entry.row_end = entry.row_end.max(row_index + 2);
                entry.column_start = entry.column_start.min(column_index + 1);
                entry.column_end = entry.column_end.max(column_index + 2);
            }
        }

        for (area, rect) in &area_rectangles {
            for row in rows.iter().take(rect.row_end - 1).skip(rect.row_start - 1) {
                for cell in row
                    .iter()
                    .take(rect.column_end - 1)
                    .skip(rect.column_start - 1)
                {
                    if cell != area {
                        return Err(NamedGridError::AreaNotRectangular { area: area.clone() });
                    }
                }
            }
        }

        Ok(Self {
            rows,
            row_count,
            column_count,
            area_rectangles,
            area_order,
        })
    }

    #[must_use]
    pub fn contains_area(&self, area: &str) -> bool {
        self.area_rectangles.contains_key(area)
    }

    #[must_use]
    pub fn area_rectangle(&self, area: &str) -> Option<&AreaRectangle> {
        self.area_rectangles.get(area)
    }

    #[must_use]
    pub fn axis_track_count(&self, axis: GridAxis) -> usize {
        match axis {
            GridAxis::Column => self.column_count,
            GridAxis::Row => self.row_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AreaRectangle {
    pub row_start: usize,
    pub row_end: usize,
    pub column_start: usize,
    pub column_end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedGridLine {
    Auto,
    Number(isize),
    BareIdent(String),
    Named { name: String, occurrence: isize },
    Span { name: Option<String>, count: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementSide {
    Start,
    End,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedAxisPlacement {
    pub start: NamedGridLine,
    pub end: NamedGridLine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedLookupReport {
    pub axis: GridAxis,
    pub name: String,
    pub requested_occurrence: isize,
    pub resolved_line: isize,
    pub explicit_matches: Vec<isize>,
    pub implicit_lines_assumed_named: Vec<isize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedLineOccurrence {
    pub line: isize,
    pub origin: LineNameOrigin,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedPlacementReport {
    pub axis: GridAxis,
    pub original_start: NamedGridLine,
    pub original_end: NamedGridLine,
    pub normalized_start: NamedGridLine,
    pub normalized_end: NamedGridLine,
    pub conflict_resolution: Option<NamedPlacementConflictResolution>,
    pub conflict_resolutions: Vec<NamedPlacementConflictResolution>,
    pub start_lookup: Option<NamedLookupReport>,
    pub end_lookup: Option<NamedLookupReport>,
    pub resolved: AxisPlacement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamedPlacementConflictResolution {
    DroppedEndSpan,
    SwappedResolvedLines,
    DroppedEqualEndLine,
    DefaultedLoneNamedSpanToOne,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedGridAreaPlacement {
    pub row: NamedAxisPlacement,
    pub column: NamedAxisPlacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedGridAreaResolutionReport {
    pub area: GridArea,
    pub row: NamedPlacementReport,
    pub column: NamedPlacementReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedTrackComponent {
    LineNames(Vec<String>),
    Track,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubgridNameComponent {
    LineNames(Vec<String>),
    Repeat {
        count: SubgridNameRepeatCount,
        line_name_sets: Vec<Vec<String>>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridNameRepeatCount {
    Number(usize),
    AutoFill,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubgridNameExpansionReport {
    pub axis: GridAxis,
    pub used_track_count: usize,
    pub local_line_names: Vec<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridLineNameInheritanceReport {
    pub axis: GridAxis,
    pub parent_span: TrackSpan,
    pub inherited_line_names: NamedGridLines,
    pub local_line_names: NamedGridLines,
    pub lines: NamedGridLines,
    pub merged_line_names: NamedGridLines,
    pub clipped_area_sources: BTreeMap<String, ClippedAreaSource>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClippedAreaSource {
    pub area: String,
    pub original_parent_span: TrackSpan,
    pub parent_span: TrackSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubgridAxisPlacementReport {
    pub unclamped_start_line: isize,
    pub unclamped_end_line: isize,
    pub clamped: NamedPlacementReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedGridError {
    LineNameCountMismatch {
        axis: GridAxis,
        explicit_track_count: usize,
        line_count: usize,
    },
    ReservedLineName {
        name: String,
    },
    ZeroLine,
    ZeroSpan,
    AutoWithoutCursor,
    LineBeforeFirst {
        axis: GridAxis,
        start_line: isize,
        end_line: isize,
    },
    EmptyTemplateAreas,
    TemplateAreaRowLengthMismatch {
        expected: usize,
        actual: usize,
        row: usize,
    },
    AreaNotRectangular {
        area: String,
    },
    AreaNotFound {
        area: String,
    },
    ZeroRepeat,
    MultipleAutoFillRepeats,
    InvalidGridAreaShorthandArity,
    SubgridSpanOutOfRange {
        axis: GridAxis,
    },
}

fn is_reserved_line_name(name: &str) -> bool {
    matches!(name, "auto" | "span")
}

fn is_null_area_token(token: &str) -> bool {
    !token.is_empty() && token.bytes().all(|byte| byte == b'.')
}
