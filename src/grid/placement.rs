use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridArea {
    pub(super) column: usize,
    pub(super) row: usize,
    pub(super) column_end: usize,
    pub(super) row_end: usize,
    pub(super) size: Size,
}

impl GridArea {
    fn single(column: usize, row: usize, width: Scalar, height: Scalar) -> Self {
        Self {
            column,
            row,
            column_end: column + 1,
            row_end: row + 1,
            size: Size::new(width, height),
        }
    }
}

pub(super) fn grid_track_requirement_from_placements(
    placements: &[ResolvedGridItemPlacement],
) -> Size<usize> {
    placements
        .iter()
        .filter(|item| item.in_flow)
        .fold(Size::new(1, 1), |requirement, item| {
            Size::new(
                requirement
                    .width
                    .max(placement_track_requirement(item.column)),
                requirement
                    .height
                    .max(placement_track_requirement(item.row)),
            )
        })
}

pub(super) fn leading_implicit_tracks_from_placements(
    placements: &[ResolvedGridItemPlacement],
    axis: GridAxisKind,
    explicit_count: usize,
) -> usize {
    placements
        .iter()
        .filter(|item| item.in_flow)
        .filter_map(|item| {
            let placement = match axis {
                GridAxisKind::Column => item.column,
                GridAxisKind::Row => item.row,
            };
            leading_implicit_tracks_for_placement(placement, explicit_count)
        })
        .max()
        .unwrap_or(0)
}

pub(super) fn leading_implicit_tracks_for_placement(
    placement: super::GridPlacement,
    explicit_count: usize,
) -> Option<usize> {
    [placement.start, placement.end]
        .into_iter()
        .flatten()
        .filter(|line| *line < 0)
        .filter_map(|line| {
            let index = explicit_count as isize + line + 1;
            (index < 0).then_some((-index) as usize)
        })
        .max()
}

pub(super) fn is_in_flow_grid_child(style: &NodeInput) -> bool {
    style.display != super::Display::None && style.position != Position::Absolute
}

pub(super) fn placement_track_requirement(placement: super::GridPlacement) -> usize {
    let start = placement
        .start
        .filter(|line| *line > 0)
        .map(|line| (line - 1) as usize);
    let end = placement
        .end
        .filter(|line| *line > 0)
        .map(|line| (line - 1) as usize);
    let span = placement.span.map(|span| span.max(1));
    match (start, end, span) {
        (Some(start), Some(end), _) if start == end => start + 1,
        (Some(start), Some(end), _) => start.max(end),
        (Some(start), None, Some(span)) => start + span,
        (Some(start), None, None) => start + 1,
        (None, Some(end), _) => end,
        (None, None, Some(span)) => span,
        (None, None, None) => 1,
    }
}

pub(super) fn placement_cell_span(
    placement: super::GridPlacement,
    explicit_track_count: usize,
) -> usize {
    if let Some(span) = placement.span {
        return span.max(1);
    }

    match (placement.start, placement.end) {
        (Some(start), Some(end)) if start == end => 1,
        (Some(start), Some(end)) => {
            let start = explicit_grid_line_to_absolute(start, explicit_track_count);
            let end = explicit_grid_line_to_absolute(end, explicit_track_count);
            start.abs_diff(end).max(1)
        }
        _ => 1,
    }
}

fn explicit_grid_line_to_absolute(line: isize, explicit_track_count: usize) -> isize {
    if line > 0 {
        line
    } else {
        explicit_track_count as isize + 2 + line
    }
}

pub(super) fn mark_occupied(occupancy: &mut [bool], column_count: usize, area: GridArea) {
    for row in area.row..area.row_end {
        for column in area.column..area.column_end {
            occupancy[row * column_count + column] = true;
        }
    }
}

pub(super) fn area_is_free(
    occupancy: &[bool],
    column_count: usize,
    row_count: usize,
    column: usize,
    row: usize,
    column_span: usize,
    row_span: usize,
) -> bool {
    if column + column_span > column_count || row + row_span > row_count {
        return false;
    }

    (row..row + row_span).all(|row| {
        (column..column + column_span).all(|column| !occupancy[row * column_count + column])
    })
}

pub(super) fn fully_definite_area(
    column: super::GridPlacement,
    row: super::GridPlacement,
    columns: &[Scalar],
    rows: &[Scalar],
    gap: Size,
    lines: GridLines,
) -> Option<GridArea> {
    if !has_definite_line(column) || !has_definite_line(row) {
        return None;
    }

    definite_area(column, row, columns, rows, gap, lines)
}

pub(super) fn absolute_grid_area(input: AbsoluteGridAreaInput<'_>) -> AbsoluteGridArea {
    let AbsoluteGridAreaInput {
        column,
        row,
        columns,
        rows,
        column_offsets,
        row_offsets,
        gap,
        constants,
        columns_are_rtl,
        lines,
        column_line_offset_adjustment,
    } = input;
    let content_size = Size::new(track_sum(columns, gap.width), track_sum(rows, gap.height));
    let padding_box_size = constants
        .node_inner_size
        .add_optional(constants.padding.sum_axes())
        .unwrap_or(content_size + constants.padding.sum_axes());
    let static_padding_box_size = constants
        .node_outer_size
        .sub_optional(constants.border.sum_axes())
        .unwrap_or(padding_box_size);
    let horizontal = absolute_grid_axis_area(AbsoluteGridAxisInput {
        placement: column,
        tracks: columns,
        offsets: column_offsets,
        gap: gap.width,
        padding_box_location: constants.content_box_inset.left - constants.padding.left,
        padding_box_size: padding_box_size.width,
        is_reverse: columns_are_rtl,
        explicit_start: lines.column_explicit_start,
        explicit_count: lines.column_explicit_count,
        reverse_positive_line_offset_adjustment: column_line_offset_adjustment,
    });
    let vertical = absolute_grid_axis_area(AbsoluteGridAxisInput {
        placement: row,
        tracks: rows,
        offsets: row_offsets,
        gap: gap.height,
        padding_box_location: constants.border.top,
        padding_box_size: padding_box_size.height,
        is_reverse: false,
        explicit_start: lines.row_explicit_start,
        explicit_count: lines.row_explicit_count,
        reverse_positive_line_offset_adjustment: 0.0,
    });

    let column_is_definite = has_definite_line(column);
    let row_is_definite = has_definite_line(row);
    AbsoluteGridArea {
        location: Point::new(horizontal.location, vertical.location),
        static_location: Point::new(
            if column_is_definite {
                horizontal.location
            } else {
                constants.border.left
            },
            if row_is_definite {
                vertical.location
            } else {
                constants.border.top
            },
        ),
        size: Size::new(horizontal.size, vertical.size),
        static_size: Size::new(
            if column_is_definite {
                horizontal.size
            } else {
                static_padding_box_size.width
            },
            if row_is_definite {
                vertical.size
            } else {
                static_padding_box_size.height
            },
        ),
    }
}

pub(super) fn absolute_grid_axis_area(input: AbsoluteGridAxisInput<'_>) -> AbsoluteGridAxisArea {
    let AbsoluteGridAxisInput {
        placement,
        tracks,
        offsets,
        gap,
        padding_box_location,
        padding_box_size,
        is_reverse,
        explicit_start,
        explicit_count,
        reverse_positive_line_offset_adjustment,
    } = input;
    let padding_box_end = padding_box_location + padding_box_size;
    if let (Some(start), None, None) = (placement.start, placement.end, placement.span)
        && let Some(line) = grid_line_offset(
            start,
            tracks,
            offsets,
            is_reverse,
            explicit_start,
            explicit_count,
            reverse_positive_line_offset_adjustment,
        )
    {
        let location = if is_reverse {
            padding_box_location
        } else {
            line
        };
        let end = if is_reverse { line } else { padding_box_end };
        return AbsoluteGridAxisArea {
            location,
            size: (end - location).max(0.0),
        };
    }

    if let (None, Some(end), None) = (placement.start, placement.end, placement.span)
        && let Some(line) = grid_line_offset(
            end,
            tracks,
            offsets,
            is_reverse,
            explicit_start,
            explicit_count,
            reverse_positive_line_offset_adjustment,
        )
    {
        let location = if is_reverse {
            line
        } else {
            padding_box_location
        };
        let end = if is_reverse { padding_box_end } else { line };
        return AbsoluteGridAxisArea {
            location,
            size: (end - location).max(0.0),
        };
    }

    if let (Some(start_line), Some(end_line), None) =
        (placement.start, placement.end, placement.span)
        && let (Some(start), Some(end)) = (
            grid_line_offset(
                start_line,
                tracks,
                offsets,
                is_reverse,
                explicit_start,
                explicit_count,
                reverse_positive_line_offset_adjustment,
            ),
            grid_line_offset(
                end_line,
                tracks,
                offsets,
                is_reverse,
                explicit_start,
                explicit_count,
                reverse_positive_line_offset_adjustment,
            ),
        )
    {
        return AbsoluteGridAxisArea {
            location: start.min(end),
            size: (start - end).abs(),
        };
    }

    let Some((start, end)) =
        placement_range(placement, tracks.len(), explicit_start, explicit_count)
            .filter(|_| has_definite_line(placement))
    else {
        return AbsoluteGridAxisArea {
            location: padding_box_location,
            size: padding_box_size,
        };
    };

    let location = if is_reverse {
        offsets[start..end]
            .iter()
            .copied()
            .reduce(Scalar::min)
            .unwrap_or(offsets[start])
    } else {
        offsets[start]
    };

    AbsoluteGridAxisArea {
        location,
        size: track_span_sum(tracks, start, end, gap),
    }
}

pub(super) fn grid_line_offset(
    line: isize,
    tracks: &[Scalar],
    offsets: &[Scalar],
    is_reverse: bool,
    explicit_start: usize,
    explicit_count: usize,
    reverse_positive_line_offset_adjustment: Scalar,
) -> Option<Scalar> {
    let index = grid_line_to_index(line, tracks.len(), explicit_start, explicit_count)?;
    let adjustment = if is_reverse && line > 0 && index > 0 {
        reverse_positive_line_offset_adjustment
    } else {
        0.0
    };
    if is_reverse {
        if index == 0 && !tracks.is_empty() {
            return Some(offsets[0] + tracks[0] + adjustment);
        }
        if index > 0 && index <= tracks.len() {
            return Some(offsets[index - 1] + adjustment);
        }

        return None;
    }

    if index < offsets.len() {
        return Some(offsets[index]);
    }
    if index == tracks.len() && !tracks.is_empty() {
        return Some(offsets[tracks.len() - 1] + tracks[tracks.len() - 1]);
    }

    None
}

pub(super) fn definite_area(
    column: super::GridPlacement,
    row: super::GridPlacement,
    columns: &[Scalar],
    rows: &[Scalar],
    gap: Size,
    lines: GridLines,
) -> Option<GridArea> {
    let (column_start, column_end) = placement_range(
        column,
        columns.len(),
        lines.column_explicit_start,
        lines.column_explicit_count,
    )?;
    let (row_start, row_end) = placement_range(
        row,
        rows.len(),
        lines.row_explicit_start,
        lines.row_explicit_count,
    )?;
    Some(GridArea {
        column: column_start,
        row: row_start,
        column_end,
        row_end,
        size: Size::new(
            track_span_sum(columns, column_start, column_end, gap.width),
            track_span_sum(rows, row_start, row_end, gap.height),
        ),
    })
}

pub(super) fn has_definite_line(placement: super::GridPlacement) -> bool {
    placement.start.is_some() || placement.end.is_some()
}

pub(super) fn definite_axis_start_and_span(
    placement: super::GridPlacement,
    track_count: usize,
    explicit_start: usize,
    explicit_count: usize,
) -> Option<(usize, usize)> {
    if !has_definite_line(placement) {
        return None;
    }

    placement_range(placement, track_count, explicit_start, explicit_count)
        .map(|(start, end)| (start, end - start))
}

pub(super) fn placement_range(
    placement: super::GridPlacement,
    track_count: usize,
    explicit_start: usize,
    explicit_count: usize,
) -> Option<(usize, usize)> {
    let start = placement
        .start
        .and_then(|line| grid_line_to_index(line, track_count, explicit_start, explicit_count));
    let end = placement
        .end
        .and_then(|line| grid_line_to_index(line, track_count, explicit_start, explicit_count));
    let span = placement.span.map(|span| span.max(1));
    let (start, end) = match (start, end, span) {
        (Some(start), Some(end), _) if start == end => (start, start + 1),
        (Some(start), Some(end), _) => (start.min(end), start.max(end)),
        (Some(start), None, Some(span)) => (start, start + span),
        (Some(start), None, None) => (start, start + 1),
        (None, Some(end), Some(span)) => (end.saturating_sub(span), end),
        (None, Some(end), None) => (end.saturating_sub(1), end),
        (None, None, Some(span)) => (0, span.min(track_count)),
        (None, None, None) => return Some((0, 1.min(track_count))),
    };

    (start < track_count && end > start && end <= track_count).then_some((start, end))
}

pub(super) fn grid_line_to_index(
    line: isize,
    track_count: usize,
    explicit_start: usize,
    explicit_count: usize,
) -> Option<usize> {
    if line == 0 {
        return None;
    }
    if line > 0 {
        return usize::try_from(line - 1)
            .ok()
            .map(|index| explicit_start + index)
            .filter(|index| *index <= track_count);
    }
    let index = explicit_start as isize + explicit_count as isize + line + 1;
    (index >= 0 && index <= track_count as isize).then_some(index as usize)
}

pub(super) fn track_span_sum(sizes: &[Scalar], start: usize, end: usize, gap: Scalar) -> Scalar {
    let end = end.clamp(start + 1, sizes.len());
    track_sum(&sizes[start..end], gap)
}

pub(super) fn next_auto_area(
    placement_index: &mut usize,
    occupancy: &[bool],
    columns: &[Scalar],
    rows: &[Scalar],
    gap: Size,
    span: Size<usize>,
    column_flow: bool,
) -> GridArea {
    let column_span = span.width.max(1);
    let row_span = span.height.max(1);
    loop {
        let index = *placement_index;
        *placement_index += 1;
        let (column, row) = if column_flow {
            (index / rows.len(), index % rows.len())
        } else {
            (index % columns.len(), index / columns.len())
        };
        if row >= rows.len() || column >= columns.len() {
            return GridArea::single(column, row, 0.0, 0.0);
        }
        if area_is_free(
            occupancy,
            columns.len(),
            rows.len(),
            column,
            row,
            column_span,
            row_span,
        ) {
            let column_end = column + column_span;
            let row_end = row + row_span;
            return GridArea {
                column,
                row,
                column_end,
                row_end,
                size: Size::new(
                    track_span_sum(columns, column, column_end, gap.width),
                    track_span_sum(rows, row, row_end, gap.height),
                ),
            };
        }
    }
}

pub(super) fn resolve_grid_child_areas<Node>(
    input: ResolveGridChildAreasInput<'_, Node>,
) -> Vec<Option<GridArea>> {
    let ResolveGridChildAreasInput {
        children,
        placements,
        style,
        columns,
        rows,
        gap,
        lines,
    } = input;
    debug_assert_eq!(children.len(), placements.children.len());
    debug_assert_eq!(children.len(), placements.items.len());
    let mut areas = vec![None; children.len()];
    let mut occupancy = vec![false; columns.len() * rows.len()];
    for (index, placement) in placements.items.iter().enumerate() {
        if placement.in_flow
            && let Some(area) =
                fully_definite_area(placement.column, placement.row, columns, rows, gap, lines)
        {
            mark_occupied(&mut occupancy, columns.len(), area);
            areas[index] = Some(area);
        }
    }

    let mut placement_index = 0;
    let column_flow = if style.display.establishes_grid_lanes_formatting_context() {
        column_flow_for_grid_lanes(style)
    } else {
        style.grid_auto_flow.is_column()
    };
    let dense_flow = style.grid_auto_flow.is_dense();
    place_grid_child_area_phase(
        placements,
        &mut areas,
        &mut occupancy,
        PlacementPhase::DefiniteMajor,
        PlacementContext {
            columns,
            rows,
            gap,
            lines,
            column_flow,
            dense_flow,
            placement_index: &mut placement_index,
        },
    );
    place_grid_child_area_phase(
        placements,
        &mut areas,
        &mut occupancy,
        PlacementPhase::Auto,
        PlacementContext {
            columns,
            rows,
            gap,
            lines,
            column_flow,
            dense_flow,
            placement_index: &mut placement_index,
        },
    );

    areas
}

pub(super) struct ResolveGridChildAreasInput<'a, Node> {
    pub(super) children: &'a [Node],
    pub(super) placements: &'a GridPlacementContext<Node>,
    pub(super) style: &'a NodeInput,
    pub(super) columns: &'a [Scalar],
    pub(super) rows: &'a [Scalar],
    pub(super) gap: Size,
    pub(super) lines: GridLines,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum PlacementPhase {
    DefiniteMajor,
    Auto,
}

pub(super) fn place_grid_child_area_phase<Node>(
    placements: &GridPlacementContext<Node>,
    areas: &mut [Option<GridArea>],
    occupancy: &mut [bool],
    phase: PlacementPhase,
    grid: PlacementContext<'_>,
) {
    debug_assert_eq!(areas.len(), placements.items.len());
    for (index, placement) in placements.items.iter().enumerate() {
        if areas[index].is_some() {
            continue;
        }
        if !placement.in_flow {
            continue;
        }
        if placement_phase(placement.column, placement.row, grid.column_flow) != phase {
            continue;
        }

        let area = resolve_grid_area(
            placement.column,
            placement.row,
            PlacementGrid {
                occupancy: &*occupancy,
                columns: grid.columns,
                rows: grid.rows,
                gap: grid.gap,
                lines: grid.lines,
                column_flow: grid.column_flow,
                dense_flow: grid.dense_flow,
                placement_index: grid.placement_index,
            },
        );
        if area.row < grid.rows.len() && area.column < grid.columns.len() {
            mark_occupied(occupancy, grid.columns.len(), area);
        }
        areas[index] = Some(area);
    }
}

pub(super) struct PlacementContext<'a> {
    pub(super) columns: &'a [Scalar],
    pub(super) rows: &'a [Scalar],
    pub(super) gap: Size,
    pub(super) lines: GridLines,
    pub(super) column_flow: bool,
    pub(super) dense_flow: bool,
    pub(super) placement_index: &'a mut usize,
}

pub(super) fn placement_phase(
    column: super::GridPlacement,
    row: super::GridPlacement,
    column_flow: bool,
) -> PlacementPhase {
    let has_major_line = if column_flow {
        has_definite_line(column)
    } else {
        has_definite_line(row)
    };
    if has_major_line {
        PlacementPhase::DefiniteMajor
    } else {
        PlacementPhase::Auto
    }
}

pub(super) struct PlacementGrid<'a> {
    pub(super) occupancy: &'a [bool],
    pub(super) columns: &'a [Scalar],
    pub(super) rows: &'a [Scalar],
    pub(super) gap: Size,
    pub(super) lines: GridLines,
    pub(super) column_flow: bool,
    pub(super) dense_flow: bool,
    pub(super) placement_index: &'a mut usize,
}

pub(super) fn resolve_grid_area(
    column: super::GridPlacement,
    row: super::GridPlacement,
    grid: PlacementGrid<'_>,
) -> GridArea {
    if let Some(area) =
        fully_definite_area(column, row, grid.columns, grid.rows, grid.gap, grid.lines)
    {
        return area;
    }

    let mut search_index = if grid.dense_flow {
        0
    } else {
        *grid.placement_index
    };
    let (area, advance_cursor) = if has_definite_line(column) && !has_definite_line(row) {
        (
            next_area_with_fixed_column(search_index, &grid, column, row.span.unwrap_or(1)),
            false,
        )
    } else if has_definite_line(row) && !has_definite_line(column) {
        (
            next_area_with_fixed_row(search_index, &grid, row, column.span.unwrap_or(1)),
            false,
        )
    } else {
        (
            next_auto_area(
                &mut search_index,
                grid.occupancy,
                grid.columns,
                grid.rows,
                grid.gap,
                Size::new(column.span.unwrap_or(1), row.span.unwrap_or(1)),
                grid.column_flow,
            ),
            true,
        )
    };

    if advance_cursor && !grid.dense_flow {
        *grid.placement_index = if grid.column_flow {
            area.column * grid.rows.len() + area.row_end
        } else {
            area.row * grid.columns.len() + area.column_end
        };
    }
    area
}

pub(super) fn next_area_with_fixed_column(
    search_index: usize,
    grid: &PlacementGrid<'_>,
    column: super::GridPlacement,
    row_span: usize,
) -> GridArea {
    let Some((column_start, column_end)) = placement_range(
        column,
        grid.columns.len(),
        grid.lines.column_explicit_start,
        grid.lines.column_explicit_count,
    ) else {
        return GridArea::single(grid.columns.len(), grid.rows.len(), 0.0, 0.0);
    };
    let column_span = column_end - column_start;
    let (current_column, mut row) = if grid.column_flow {
        (
            search_index / grid.rows.len(),
            search_index % grid.rows.len(),
        )
    } else {
        (
            search_index % grid.columns.len(),
            search_index / grid.columns.len(),
        )
    };
    if column_start < current_column {
        row += 1;
    }
    let row_span = row_span.max(1);
    while row < grid.rows.len() {
        if area_is_free(
            grid.occupancy,
            grid.columns.len(),
            grid.rows.len(),
            column_start,
            row,
            column_span,
            row_span,
        ) {
            let row_end = row + row_span;
            return GridArea {
                column: column_start,
                row,
                column_end,
                row_end,
                size: Size::new(
                    track_span_sum(grid.columns, column_start, column_end, grid.gap.width),
                    track_span_sum(grid.rows, row, row_end, grid.gap.height),
                ),
            };
        }
        row += 1;
    }
    GridArea::single(column_start, grid.rows.len(), 0.0, 0.0)
}

pub(super) fn next_area_with_fixed_row(
    search_index: usize,
    grid: &PlacementGrid<'_>,
    row: super::GridPlacement,
    column_span: usize,
) -> GridArea {
    let Some((row_start, row_end)) = placement_range(
        row,
        grid.rows.len(),
        grid.lines.row_explicit_start,
        grid.lines.row_explicit_count,
    ) else {
        return GridArea::single(grid.columns.len(), grid.rows.len(), 0.0, 0.0);
    };
    let row_span = row_end - row_start;
    let (current_row, mut column) = if grid.column_flow {
        (
            search_index % grid.rows.len(),
            search_index / grid.rows.len(),
        )
    } else {
        (
            search_index / grid.columns.len(),
            search_index % grid.columns.len(),
        )
    };
    if row_start < current_row {
        column += 1;
    }
    let column_span = column_span.max(1);
    while column < grid.columns.len() {
        if area_is_free(
            grid.occupancy,
            grid.columns.len(),
            grid.rows.len(),
            column,
            row_start,
            column_span,
            row_span,
        ) {
            let column_end = column + column_span;
            return GridArea {
                column,
                row: row_start,
                column_end,
                row_end,
                size: Size::new(
                    track_span_sum(grid.columns, column, column_end, grid.gap.width),
                    track_span_sum(grid.rows, row_start, row_end, grid.gap.height),
                ),
            };
        }
        column += 1;
    }
    GridArea::single(grid.columns.len(), row_start, 0.0, 0.0)
}
