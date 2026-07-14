use super::*;
use crate::geometry::{LogicalPointOf, LogicalSizeOf};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridArea<S: LayoutScalar = Scalar> {
    pub(super) column: usize,
    pub(super) row: usize,
    pub(super) column_end: usize,
    pub(super) row_end: usize,
    pub(super) size: LogicalSizeOf<S>,
}

impl<S: LayoutScalar> GridArea<S> {
    fn single(column: usize, row: usize, width: S, height: S) -> Self {
        Self {
            column,
            row,
            column_end: column + 1,
            row_end: row + 1,
            size: LogicalSizeOf::new(width, height),
        }
    }
}

pub(super) fn grid_track_requirement_from_placements(
    placements: &[ResolvedGridItemPlacement],
) -> LogicalSizeOf<usize> {
    placements.iter().filter(|item| item.in_flow).fold(
        LogicalSizeOf::new(1, 1),
        |requirement, item| {
            LogicalSizeOf::new(
                requirement
                    .inline
                    .max(placement_track_requirement(item.column)),
                requirement.block.max(placement_track_requirement(item.row)),
            )
        },
    )
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
    [placement.start(), placement.end()]
        .into_iter()
        .flatten()
        .map(|line| line.get())
        .filter(|line| *line < 0)
        .filter_map(|line| {
            let index = explicit_count as isize + line + 1;
            (index < 0).then_some((-index) as usize)
        })
        .max()
}

pub(super) fn is_in_flow_grid_child<S: LayoutScalar>(style: &NodeInputOf<S>) -> bool {
    style.display != super::Display::None && style.position != Position::Absolute
}

pub(super) fn placement_track_requirement(placement: super::GridPlacement) -> usize {
    let start = placement
        .start()
        .map(|line| line.get())
        .filter(|line| *line > 0)
        .map(|line| (line - 1) as usize);
    let end = placement
        .end()
        .map(|line| line.get())
        .filter(|line| *line > 0)
        .map(|line| (line - 1) as usize);
    let span = placement.span().map(|span| span.get());
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
    if let Some(span) = placement.span() {
        return span.get();
    }

    match (placement.start(), placement.end()) {
        (Some(start), Some(end)) if start == end => 1,
        (Some(start), Some(end)) => {
            let start = explicit_grid_line_to_absolute(start.get(), explicit_track_count);
            let end = explicit_grid_line_to_absolute(end.get(), explicit_track_count);
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

pub(super) fn mark_occupied<S: LayoutScalar>(
    occupancy: &mut [bool],
    column_count: usize,
    area: GridArea<S>,
) {
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

pub(super) fn fully_definite_area<S: LayoutScalar>(
    column: super::GridPlacement,
    row: super::GridPlacement,
    columns: &[S],
    rows: &[S],
    gap: LogicalSizeOf<S>,
    lines: GridLines,
) -> Option<GridArea<S>> {
    if !has_definite_line(column) || !has_definite_line(row) {
        return None;
    }

    definite_area(column, row, columns, rows, gap, lines)
}

pub(super) fn absolute_grid_area<S: LayoutScalar, F: super::child::AbsoluteGridPlacementFrame>(
    frame: F,
    input: AbsoluteGridAreaInput<'_, S>,
) -> LogicalAbsoluteGridArea<S> {
    let AbsoluteGridAreaInput {
        column,
        row,
        columns,
        rows,
        column_offsets,
        row_offsets,
        gap,
        constants,
        lines,
        column_line_offset_adjustment,
    } = input;
    let content_size =
        LogicalSizeOf::new(track_sum(columns, gap.inline), track_sum(rows, gap.block));
    let padding = frame.placement_edges(constants.padding);
    let border = frame.placement_edges(constants.border);
    let padding_size = LogicalSizeOf::new(padding.inline_sum(), padding.block_sum());
    let border_size = LogicalSizeOf::new(border.inline_sum(), border.block_sum());
    let logical_inner_size = frame.placement_size(constants.node_inner_size);
    let padding_box_size = LogicalSizeOf::new(
        logical_inner_size
            .inline
            .map(|size| size + padding_size.inline)
            .unwrap_or(content_size.inline + padding_size.inline),
        logical_inner_size
            .block
            .map(|size| size + padding_size.block)
            .unwrap_or(content_size.block + padding_size.block),
    );
    let logical_outer_size = frame.placement_size(constants.node_outer_size);
    let static_padding_box_size = LogicalSizeOf::new(
        logical_outer_size
            .inline
            .map(|size| size - border_size.inline)
            .unwrap_or(padding_box_size.inline),
        logical_outer_size
            .block
            .map(|size| size - border_size.block)
            .unwrap_or(padding_box_size.block),
    );
    let inline = absolute_grid_axis_area(AbsoluteGridAxisInput {
        placement: column,
        tracks: columns,
        offsets: column_offsets,
        gap: gap.inline,
        padding_box_location: border.inline_start,
        padding_box_size: padding_box_size.inline,
        is_reverse: frame.column_is_reverse(),
        explicit_start: lines.column_explicit_start,
        explicit_count: lines.column_explicit_count,
        positive_line_offset_adjustment: column_line_offset_adjustment,
    });
    let block = absolute_grid_axis_area(AbsoluteGridAxisInput {
        placement: row,
        tracks: rows,
        offsets: row_offsets,
        gap: gap.block,
        padding_box_location: border.block_start,
        padding_box_size: padding_box_size.block,
        is_reverse: false,
        explicit_start: lines.row_explicit_start,
        explicit_count: lines.row_explicit_count,
        positive_line_offset_adjustment: S::ZERO,
    });

    let column_is_definite = has_definite_line(column);
    let row_is_definite = has_definite_line(row);
    LogicalAbsoluteGridArea {
        location: LogicalPointOf::new(inline.location, block.location),
        static_location: LogicalPointOf::new(
            if column_is_definite {
                inline.location
            } else {
                border.inline_start
            },
            if row_is_definite {
                block.location
            } else {
                border.block_start
            },
        ),
        size: LogicalSizeOf::new(inline.size, block.size),
        static_size: LogicalSizeOf::new(
            if column_is_definite {
                inline.size
            } else {
                static_padding_box_size.inline
            },
            if row_is_definite {
                block.size
            } else {
                static_padding_box_size.block
            },
        ),
    }
}

pub(super) fn absolute_grid_axis_area<S: LayoutScalar>(
    input: AbsoluteGridAxisInput<'_, S>,
) -> AbsoluteGridAxisArea<S> {
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
        positive_line_offset_adjustment,
    } = input;
    let padding_box_end = padding_box_location + padding_box_size;
    if let (Some(start), None, None) = (placement.start(), placement.end(), placement.span())
        && let Some(line) = grid_line_offset(
            start.get(),
            tracks,
            offsets,
            is_reverse,
            explicit_start,
            explicit_count,
            positive_line_offset_adjustment,
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
            size: (end - location).max(S::ZERO),
        };
    }

    if let (None, Some(end), None) = (placement.start(), placement.end(), placement.span())
        && let Some(line) = grid_line_offset(
            end.get(),
            tracks,
            offsets,
            is_reverse,
            explicit_start,
            explicit_count,
            positive_line_offset_adjustment,
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
            size: (end - location).max(S::ZERO),
        };
    }

    if let (Some(start_line), Some(end_line), None) =
        (placement.start(), placement.end(), placement.span())
        && let (Some(start), Some(end)) = (
            grid_line_offset(
                start_line.get(),
                tracks,
                offsets,
                is_reverse,
                explicit_start,
                explicit_count,
                positive_line_offset_adjustment,
            ),
            grid_line_offset(
                end_line.get(),
                tracks,
                offsets,
                is_reverse,
                explicit_start,
                explicit_count,
                positive_line_offset_adjustment,
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
            .reduce(S::min)
            .unwrap_or(offsets[start])
    } else {
        offsets[start]
    };

    AbsoluteGridAxisArea {
        location,
        size: track_span_sum(tracks, start, end, gap),
    }
}

pub(super) fn grid_line_offset<S: LayoutScalar>(
    line: isize,
    tracks: &[S],
    offsets: &[S],
    is_reverse: bool,
    explicit_start: usize,
    explicit_count: usize,
    positive_line_offset_adjustment: S,
) -> Option<S> {
    let index = grid_line_to_index(line, tracks.len(), explicit_start, explicit_count)?;
    let adjustment = if line > 0 && index > 0 {
        positive_line_offset_adjustment
    } else {
        S::ZERO
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

pub(super) fn definite_area<S: LayoutScalar>(
    column: super::GridPlacement,
    row: super::GridPlacement,
    columns: &[S],
    rows: &[S],
    gap: LogicalSizeOf<S>,
    lines: GridLines,
) -> Option<GridArea<S>> {
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
        size: LogicalSizeOf::new(
            track_span_sum(columns, column_start, column_end, gap.inline),
            track_span_sum(rows, row_start, row_end, gap.block),
        ),
    })
}

pub(super) fn has_definite_line(placement: super::GridPlacement) -> bool {
    placement.start().is_some() || placement.end().is_some()
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
    let start = placement.start().and_then(|line| {
        grid_line_to_index(line.get(), track_count, explicit_start, explicit_count)
    });
    let end = placement.end().and_then(|line| {
        grid_line_to_index(line.get(), track_count, explicit_start, explicit_count)
    });
    let span = placement.span().map(|span| span.get());
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

pub(super) fn track_span_sum<S: LayoutScalar>(sizes: &[S], start: usize, end: usize, gap: S) -> S {
    let end = end.clamp(start + 1, sizes.len());
    let tracks = &sizes[start..end];
    tracks.iter().copied().fold(S::ZERO, |sum, size| sum + size)
        + gap * S::from_usize(tracks.len().saturating_sub(1))
}

pub(super) fn next_auto_area<S: LayoutScalar>(
    placement_index: &mut usize,
    occupancy: &[bool],
    columns: &[S],
    rows: &[S],
    gap: LogicalSizeOf<S>,
    span: LogicalSizeOf<usize>,
    column_flow: bool,
) -> GridArea<S> {
    let column_span = span.inline.max(1);
    let row_span = span.block.max(1);
    loop {
        let index = *placement_index;
        *placement_index += 1;
        let (column, row) = if column_flow {
            (index / rows.len(), index % rows.len())
        } else {
            (index % columns.len(), index / columns.len())
        };
        if row >= rows.len() || column >= columns.len() {
            return GridArea::single(column, row, S::ZERO, S::ZERO);
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
                size: LogicalSizeOf::new(
                    track_span_sum(columns, column, column_end, gap.inline),
                    track_span_sum(rows, row, row_end, gap.block),
                ),
            };
        }
    }
}

pub(super) fn resolve_grid_child_areas<Node, S: LayoutScalar>(
    input: ResolveGridChildAreasInput<'_, Node, S>,
) -> Vec<Option<GridArea<S>>> {
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

pub(super) struct ResolveGridChildAreasInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) children: &'a [Node],
    pub(super) placements: &'a GridPlacementContext<Node>,
    pub(super) style: &'a NodeInputOf<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) lines: GridLines,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum PlacementPhase {
    DefiniteMajor,
    Auto,
}

pub(super) fn place_grid_child_area_phase<Node, S: LayoutScalar>(
    placements: &GridPlacementContext<Node>,
    areas: &mut [Option<GridArea<S>>],
    occupancy: &mut [bool],
    phase: PlacementPhase,
    grid: PlacementContext<'_, S>,
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

pub(super) struct PlacementContext<'a, S: LayoutScalar = Scalar> {
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
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

pub(super) struct PlacementGrid<'a, S: LayoutScalar = Scalar> {
    pub(super) occupancy: &'a [bool],
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) lines: GridLines,
    pub(super) column_flow: bool,
    pub(super) dense_flow: bool,
    pub(super) placement_index: &'a mut usize,
}

pub(super) fn resolve_grid_area<S: LayoutScalar>(
    column: super::GridPlacement,
    row: super::GridPlacement,
    grid: PlacementGrid<'_, S>,
) -> GridArea<S> {
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
            next_area_with_fixed_column(search_index, &grid, column, placement_span_or_one(row)),
            false,
        )
    } else if has_definite_line(row) && !has_definite_line(column) {
        (
            next_area_with_fixed_row(search_index, &grid, row, placement_span_or_one(column)),
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
                LogicalSizeOf::new(
                    placement_span_or_one(column).get(),
                    placement_span_or_one(row).get(),
                ),
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

fn placement_span_or_one(placement: super::GridPlacement) -> crate::GridSpan {
    placement
        .span()
        .unwrap_or_else(|| crate::GridSpan::new(1).expect("one is a valid grid span"))
}

pub(super) fn next_area_with_fixed_column<S: LayoutScalar>(
    search_index: usize,
    grid: &PlacementGrid<'_, S>,
    column: super::GridPlacement,
    row_span: crate::GridSpan,
) -> GridArea<S> {
    let Some((column_start, column_end)) = placement_range(
        column,
        grid.columns.len(),
        grid.lines.column_explicit_start,
        grid.lines.column_explicit_count,
    ) else {
        return GridArea::single(grid.columns.len(), grid.rows.len(), S::ZERO, S::ZERO);
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
    let row_span = row_span.get();
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
                size: LogicalSizeOf::new(
                    track_span_sum(grid.columns, column_start, column_end, grid.gap.inline),
                    track_span_sum(grid.rows, row, row_end, grid.gap.block),
                ),
            };
        }
        row += 1;
    }
    GridArea::single(column_start, grid.rows.len(), S::ZERO, S::ZERO)
}

pub(super) fn next_area_with_fixed_row<S: LayoutScalar>(
    search_index: usize,
    grid: &PlacementGrid<'_, S>,
    row: super::GridPlacement,
    column_span: crate::GridSpan,
) -> GridArea<S> {
    let Some((row_start, row_end)) = placement_range(
        row,
        grid.rows.len(),
        grid.lines.row_explicit_start,
        grid.lines.row_explicit_count,
    ) else {
        return GridArea::single(grid.columns.len(), grid.rows.len(), S::ZERO, S::ZERO);
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
    let column_span = column_span.get();
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
                size: LogicalSizeOf::new(
                    track_span_sum(grid.columns, column, column_end, grid.gap.inline),
                    track_span_sum(grid.rows, row_start, row_end, grid.gap.block),
                ),
            };
        }
        column += 1;
    }
    GridArea::single(grid.columns.len(), row_start, S::ZERO, S::ZERO)
}
