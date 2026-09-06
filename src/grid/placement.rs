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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PlacedGridArea {
    pub(super) column_start: usize,
    pub(super) row_start: usize,
    pub(super) column_end: usize,
    pub(super) row_end: usize,
}

impl PlacedGridArea {
    pub(super) const fn new(
        column_start: usize,
        row_start: usize,
        column_end: usize,
        row_end: usize,
    ) -> Self {
        Self {
            column_start,
            row_start,
            column_end,
            row_end,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GridPlacementDemandError {
    AxisCapacity {
        axis: GridAxisKind,
        requested_tracks: usize,
    },
    OccupancyCapacity {
        columns: usize,
        rows: usize,
    },
}

#[derive(Clone, Debug)]
struct PlacementDemand {
    // Hypothetical search extent may exceed inherited tracks; publication uses
    // the fixed inherited extent through `published_axis_track_count`.
    column_count: usize,
    row_count: usize,
    explicit_columns: usize,
    explicit_rows: usize,
    column_explicit_start: usize,
    row_explicit_start: usize,
    inherited_columns: bool,
    inherited_rows: bool,
    placed_areas: Vec<Option<PlacedGridArea>>,
}

impl PlacementDemand {
    fn new<S: LayoutScalar>(
        topology: &ExpandedGridTopology<S>,
        item_count: usize,
    ) -> Result<Self, GridPlacementDemandError> {
        let mut placed_areas = Vec::new();
        placed_areas.try_reserve_exact(item_count).map_err(|_| {
            GridPlacementDemandError::OccupancyCapacity {
                columns: topology.column_tracks.len(),
                rows: topology.row_tracks.len(),
            }
        })?;
        placed_areas.resize(item_count, None);
        Ok(Self {
            column_count: topology.column_tracks.len(),
            row_count: topology.row_tracks.len(),
            explicit_columns: topology.explicit_columns,
            explicit_rows: topology.explicit_rows,
            column_explicit_start: 0,
            row_explicit_start: 0,
            inherited_columns: topology.axis_is_inherited(GridAxisKind::Column),
            inherited_rows: topology.axis_is_inherited(GridAxisKind::Row),
            placed_areas,
        })
    }

    fn axis_track_count(&self, axis: GridAxisKind) -> usize {
        match axis {
            GridAxisKind::Column => self.column_count,
            GridAxisKind::Row => self.row_count,
        }
    }

    fn published_axis_track_count(&self, axis: GridAxisKind) -> usize {
        if self.axis_is_inherited(axis) {
            self.axis_explicit_count(axis)
        } else {
            self.axis_track_count(axis)
        }
    }

    fn automatic_minor_span(&self, placement: GridPlacement, axis: GridAxisKind) -> usize {
        let span = automatic_axis_span(placement);
        if self.axis_is_inherited(axis) {
            span.min(self.axis_explicit_count(axis))
        } else {
            span
        }
    }

    fn set_axis_track_count(&mut self, axis: GridAxisKind, count: usize) {
        match axis {
            GridAxisKind::Column => self.column_count = count,
            GridAxisKind::Row => self.row_count = count,
        }
    }

    fn axis_explicit_count(&self, axis: GridAxisKind) -> usize {
        match axis {
            GridAxisKind::Column => self.explicit_columns,
            GridAxisKind::Row => self.explicit_rows,
        }
    }

    fn axis_explicit_start(&self, axis: GridAxisKind) -> usize {
        match axis {
            GridAxisKind::Column => self.column_explicit_start,
            GridAxisKind::Row => self.row_explicit_start,
        }
    }

    fn set_axis_explicit_start(&mut self, axis: GridAxisKind, start: usize) {
        match axis {
            GridAxisKind::Column => self.column_explicit_start = start,
            GridAxisKind::Row => self.row_explicit_start = start,
        }
    }

    fn axis_is_inherited(&self, axis: GridAxisKind) -> bool {
        match axis {
            GridAxisKind::Column => self.inherited_columns,
            GridAxisKind::Row => self.inherited_rows,
        }
    }

    fn capacity_error(&self) -> GridPlacementDemandError {
        GridPlacementDemandError::OccupancyCapacity {
            columns: self.column_count,
            rows: self.row_count,
        }
    }
}

/// Sparse progress for the phase that places items locked to a major track.
/// Fully definite items contribute occupancy, but do not advance these frontiers.
struct LockedMajorFrontier {
    minor_ends: Vec<usize>,
}

impl LockedMajorFrontier {
    fn new(
        demand: &PlacementDemand,
        major: GridAxisKind,
    ) -> Result<Self, GridPlacementDemandError> {
        let mut minor_ends = Vec::new();
        minor_ends
            .try_reserve_exact(demand.axis_track_count(major))
            .map_err(|_| demand.capacity_error())?;
        minor_ends.resize(demand.axis_track_count(major), 0);
        Ok(Self { minor_ends })
    }

    fn start(&self, major_start: usize, major_end: usize) -> usize {
        self.minor_ends[major_start..major_end]
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
    }

    fn advance(&mut self, major_start: usize, major_end: usize, minor_end: usize) {
        for frontier in &mut self.minor_ends[major_start..major_end] {
            *frontier = (*frontier).max(minor_end);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PlacementCursor {
    major: usize,
    minor: usize,
}

#[derive(Clone, Copy, Debug)]
struct PlacementAxes {
    major: GridAxisKind,
    minor: GridAxisKind,
    column_flow: bool,
}

pub(super) fn derive_grid_placement_demand<Node, S: LayoutScalar>(
    topology: &mut ExpandedGridTopology<S>,
    placements: &mut GridPlacementContext<Node, S>,
    flow: GridAutoFlow,
) -> Result<(), GridPlacementDemandError> {
    let mut demand = PlacementDemand::new(topology, placements.items.len())?;
    derive_grid_placement_demand_inner(&mut demand, placements, flow)?;
    topology.apply_placement_demand(
        demand.column_explicit_start,
        demand.row_explicit_start,
        demand.published_axis_track_count(GridAxisKind::Column),
        demand.published_axis_track_count(GridAxisKind::Row),
    )?;
    placements.settled_areas = Some(demand.placed_areas);
    Ok(())
}

fn derive_grid_placement_demand_inner<Node, S: LayoutScalar>(
    demand: &mut PlacementDemand,
    placements: &GridPlacementContext<Node, S>,
    flow: GridAutoFlow,
) -> Result<(), GridPlacementDemandError> {
    if !placements.items.iter().any(|item| item.in_flow) {
        return Ok(());
    }

    grow_for_definite_placements(demand, placements)?;
    let column_flow = flow.is_column();
    let (major_axis, minor_axis) = if column_flow {
        (GridAxisKind::Column, GridAxisKind::Row)
    } else {
        (GridAxisKind::Row, GridAxisKind::Column)
    };
    let largest_unresolved_minor_span = placements
        .items
        .iter()
        .filter(|item| item.in_flow)
        .filter_map(|item| {
            let placement = item_axis_placement(item, minor_axis);
            (!has_definite_line(placement))
                .then_some(demand.automatic_minor_span(placement, minor_axis))
        })
        .max()
        .unwrap_or(1);
    ensure_axis_track_count(demand, minor_axis, largest_unresolved_minor_span)?;
    ensure_axis_track_count(demand, major_axis, 1)?;

    let mut occupancy = super::topology::GridOccupancy::new(demand.column_count, demand.row_count)?;

    for (index, placement) in placements.items.iter().enumerate() {
        if !placement.in_flow
            || !has_definite_line(placement.column)
            || !has_definite_line(placement.row)
        {
            continue;
        }
        let area = definite_integer_area(demand, placement)?;
        occupancy.occupy(area);
        demand.placed_areas[index] = Some(area);
    }

    let mut frontier = LockedMajorFrontier::new(demand, major_axis)?;
    for source_index in &placements.order_modified_indexes {
        let index = source_index.get();
        let placement = placements
            .items
            .get(index)
            .ok_or_else(|| demand.capacity_error())?;
        if !placement.in_flow || demand.placed_areas[index].is_some() {
            continue;
        }
        let major = item_axis_placement(placement, major_axis);
        if !has_definite_line(major) {
            continue;
        }
        let area = place_definite_major_item(
            demand,
            &mut occupancy,
            placement,
            PlacementAxes {
                major: major_axis,
                minor: minor_axis,
                column_flow,
            },
            flow.is_dense(),
            &mut frontier,
        )?;
        occupancy.occupy(area);
        demand.placed_areas[index] = Some(area);
    }

    let mut cursor = PlacementCursor::default();
    for source_index in &placements.order_modified_indexes {
        let index = source_index.get();
        let placement = placements
            .items
            .get(index)
            .ok_or_else(|| demand.capacity_error())?;
        if !placement.in_flow || demand.placed_areas[index].is_some() {
            continue;
        }
        if has_definite_line(item_axis_placement(placement, major_axis)) {
            return Err(demand.capacity_error());
        }
        let area = place_automatic_item(
            demand,
            &mut occupancy,
            placement,
            PlacementAxes {
                major: major_axis,
                minor: minor_axis,
                column_flow,
            },
            flow.is_dense(),
            &mut cursor,
        )?;
        occupancy.occupy(area);
        demand.placed_areas[index] = Some(area);
    }

    // Search retains hypothetical overflow cells. Clamp only settled areas, so
    // inherited bounds may produce overlapping results without restarting search.
    let inherited_columns = demand.inherited_columns;
    let inherited_rows = demand.inherited_rows;
    let columns = demand.published_axis_track_count(GridAxisKind::Column);
    let rows = demand.published_axis_track_count(GridAxisKind::Row);
    for area in demand.placed_areas.iter_mut().flatten() {
        if inherited_columns {
            (area.column_start, area.column_end) =
                clamp_subgrid_axis_range(area.column_start, area.column_end, columns);
        }
        if inherited_rows {
            (area.row_start, area.row_end) =
                clamp_subgrid_axis_range(area.row_start, area.row_end, rows);
        }
    }

    if demand
        .placed_areas
        .iter()
        .zip(&placements.items)
        .any(|(area, placement)| {
            placement.in_flow
                && !matches!(
                    area,
                    Some(area)
                        if area.column_start < area.column_end
                            && area.row_start < area.row_end
                            && area.column_end <= columns
                            && area.row_end <= rows
                )
        })
    {
        return Err(demand.capacity_error());
    }
    Ok(())
}

fn grow_for_definite_placements<Node, S: LayoutScalar>(
    demand: &mut PlacementDemand,
    placements: &GridPlacementContext<Node, S>,
) -> Result<(), GridPlacementDemandError> {
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        if demand.axis_is_inherited(axis) {
            continue;
        }
        let explicit_count = demand.axis_explicit_count(axis);
        let explicit_end = isize::try_from(explicit_count).map_err(|_| {
            GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: explicit_count,
            }
        })?;
        let mut minimum = 0_isize;
        let mut maximum = explicit_end;
        for item in placements.items.iter().filter(|item| item.in_flow) {
            if let Some((start, end)) =
                signed_definite_axis_range(item_axis_placement(item, axis), explicit_count, axis)?
            {
                minimum = minimum.min(start);
                maximum = maximum.max(end);
            }
        }
        let leading = minimum
            .checked_neg()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: usize::MAX,
            })?;
        let trailing = maximum
            .checked_sub(explicit_end)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: usize::MAX,
            })?;
        let requested_tracks = leading
            .checked_add(explicit_count)
            .and_then(|count| count.checked_add(trailing))
            .ok_or(GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: usize::MAX,
            })?;
        demand.set_axis_explicit_start(axis, leading);
        demand.set_axis_track_count(axis, requested_tracks.max(demand.axis_track_count(axis)));
    }
    Ok(())
}

fn signed_definite_axis_range(
    placement: GridPlacement,
    explicit_count: usize,
    axis: GridAxisKind,
) -> Result<Option<(isize, isize)>, GridPlacementDemandError> {
    if !has_definite_line(placement) {
        return Ok(None);
    }
    let start = placement
        .start()
        .map(|line| signed_grid_line(line.get(), explicit_count, axis))
        .transpose()?;
    let end = placement
        .end()
        .map(|line| signed_grid_line(line.get(), explicit_count, axis))
        .transpose()?;
    let span = placement
        .span()
        .map(|span| {
            isize::try_from(span.get()).map_err(|_| GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: span.get(),
            })
        })
        .transpose()?;
    let overflow = || GridPlacementDemandError::AxisCapacity {
        axis,
        requested_tracks: placement.span().map_or(usize::MAX, |span| span.get()),
    };
    let range = match (start, end, span) {
        (Some(start), Some(end), _) if start == end => {
            (start, start.checked_add(1).ok_or_else(overflow)?)
        }
        (Some(start), Some(end), _) => (start.min(end), start.max(end)),
        (Some(start), None, Some(span)) => (start, start.checked_add(span).ok_or_else(overflow)?),
        (Some(start), None, None) => (start, start.checked_add(1).ok_or_else(overflow)?),
        (None, Some(end), Some(span)) => (end.checked_sub(span).ok_or_else(overflow)?, end),
        (None, Some(end), None) => (end.checked_sub(1).ok_or_else(overflow)?, end),
        (None, None, _) => return Ok(None),
    };
    Ok(Some(range))
}

fn signed_grid_line(
    line: isize,
    explicit_count: usize,
    axis: GridAxisKind,
) -> Result<isize, GridPlacementDemandError> {
    if line > 0 {
        return Ok(line - 1);
    }
    isize::try_from(explicit_count)
        .ok()
        .and_then(|count| count.checked_add(line))
        .and_then(|line| line.checked_add(1))
        .ok_or(GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks: explicit_count,
        })
}

fn definite_integer_axis_range(
    demand: &PlacementDemand,
    placement: GridPlacement,
    axis: GridAxisKind,
) -> Result<(usize, usize), GridPlacementDemandError> {
    let (start, end) =
        signed_definite_axis_range(placement, demand.axis_explicit_count(axis), axis)?
            .ok_or_else(|| demand.capacity_error())?;
    if demand.axis_is_inherited(axis) {
        let range = clamp_subgrid_axis_range(
            start.max(0) as usize,
            end.max(0) as usize,
            demand.axis_explicit_count(axis),
        );
        if range.0 >= range.1 {
            return Err(demand.capacity_error());
        }
        return Ok(range);
    }
    let explicit_start = isize::try_from(demand.axis_explicit_start(axis)).map_err(|_| {
        GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks: demand.axis_track_count(axis),
        }
    })?;
    let start = start
        .checked_add(explicit_start)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks: demand.axis_track_count(axis),
        })?;
    let end = end
        .checked_add(explicit_start)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks: demand.axis_track_count(axis),
        })?;
    if start >= end || end > demand.axis_track_count(axis) {
        return Err(GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks: end,
        });
    }
    Ok((start, end))
}

/// Intersect a settled range with inherited tracks, retaining the nearest track
/// when the entire range falls outside. Callers resolve signed lines first.
pub(super) fn clamp_subgrid_axis_range(start: usize, end: usize, count: usize) -> (usize, usize) {
    let start = start.min(count);
    let end = end.min(count);
    if start < end || count == 0 {
        (start, end)
    } else if start == count {
        (count - 1, count)
    } else {
        (start, start + 1)
    }
}

fn definite_integer_area(
    demand: &PlacementDemand,
    placement: &ResolvedGridItemPlacement,
) -> Result<PlacedGridArea, GridPlacementDemandError> {
    let (column_start, column_end) =
        definite_integer_axis_range(demand, placement.column, GridAxisKind::Column)?;
    let (row_start, row_end) =
        definite_integer_axis_range(demand, placement.row, GridAxisKind::Row)?;
    Ok(PlacedGridArea::new(
        column_start,
        row_start,
        column_end,
        row_end,
    ))
}

fn item_axis_placement(placement: &ResolvedGridItemPlacement, axis: GridAxisKind) -> GridPlacement {
    match axis {
        GridAxisKind::Column => placement.column,
        GridAxisKind::Row => placement.row,
    }
}

fn automatic_axis_span(placement: GridPlacement) -> usize {
    placement.span().map_or(1, |span| span.get())
}

fn ensure_axis_track_count(
    demand: &mut PlacementDemand,
    axis: GridAxisKind,
    required_count: usize,
) -> Result<(), GridPlacementDemandError> {
    if isize::try_from(required_count).is_err() {
        return Err(GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks: required_count,
        });
    }
    if required_count <= demand.axis_track_count(axis) {
        return Ok(());
    }
    demand.set_axis_track_count(axis, required_count);
    Ok(())
}

fn grow_occupancy_axis(
    demand: &mut PlacementDemand,
    occupancy: &mut super::topology::GridOccupancy,
    axis: GridAxisKind,
    required_count: usize,
) -> Result<(), GridPlacementDemandError> {
    ensure_axis_track_count(demand, axis, required_count)?;
    occupancy.grow_to(demand.column_count, demand.row_count)
}

fn place_definite_major_item(
    demand: &mut PlacementDemand,
    occupancy: &mut super::topology::GridOccupancy,
    placement: &ResolvedGridItemPlacement,
    axes: PlacementAxes,
    dense: bool,
    frontier: &mut LockedMajorFrontier,
) -> Result<PlacedGridArea, GridPlacementDemandError> {
    let PlacementAxes {
        major: major_axis,
        minor: minor_axis,
        column_flow,
    } = axes;
    let (major_start, major_end) = definite_integer_axis_range(
        demand,
        item_axis_placement(placement, major_axis),
        major_axis,
    )?;
    let minor_span =
        demand.automatic_minor_span(item_axis_placement(placement, minor_axis), minor_axis);
    let minimum_minor_start = if dense {
        0
    } else {
        frontier.start(major_start, major_end)
    };
    let minimum_minor_end = minimum_minor_start.checked_add(minor_span).ok_or(
        GridPlacementDemandError::AxisCapacity {
            axis: minor_axis,
            requested_tracks: usize::MAX,
        },
    )?;
    ensure_axis_track_count(demand, minor_axis, minimum_minor_end)?;
    occupancy.grow_to(demand.column_count, demand.row_count)?;
    loop {
        let minor_count = demand.axis_track_count(minor_axis);
        let last_start =
            minor_count
                .checked_sub(minor_span)
                .ok_or(GridPlacementDemandError::AxisCapacity {
                    axis: minor_axis,
                    requested_tracks: minor_span,
                })?;
        for minor_start in minimum_minor_start..=last_start {
            let minor_end = minor_start.checked_add(minor_span).ok_or(
                GridPlacementDemandError::AxisCapacity {
                    axis: minor_axis,
                    requested_tracks: minor_span,
                },
            )?;
            let area = oriented_area(column_flow, major_start, major_end, minor_start, minor_end);
            if occupancy.is_free(area) {
                frontier.advance(major_start, major_end, minor_end);
                return Ok(area);
            }
        }
        let required =
            minor_count
                .checked_add(1)
                .ok_or(GridPlacementDemandError::AxisCapacity {
                    axis: minor_axis,
                    requested_tracks: usize::MAX,
                })?;
        grow_occupancy_axis(demand, occupancy, minor_axis, required)?;
    }
}

fn place_automatic_item(
    demand: &mut PlacementDemand,
    occupancy: &mut super::topology::GridOccupancy,
    placement: &ResolvedGridItemPlacement,
    axes: PlacementAxes,
    dense: bool,
    cursor: &mut PlacementCursor,
) -> Result<PlacedGridArea, GridPlacementDemandError> {
    let major_span = automatic_axis_span(item_axis_placement(placement, axes.major));
    let minor = item_axis_placement(placement, axes.minor);
    let mut search = if dense {
        PlacementCursor::default()
    } else {
        *cursor
    };

    if has_definite_line(minor) {
        let (minor_start, minor_end) = definite_integer_axis_range(demand, minor, axes.minor)?;
        if !dense && minor_start < search.minor {
            search.major =
                search
                    .major
                    .checked_add(1)
                    .ok_or(GridPlacementDemandError::AxisCapacity {
                        axis: axes.major,
                        requested_tracks: usize::MAX,
                    })?;
        }
        search.minor = minor_start;
        loop {
            let major_end = search.major.checked_add(major_span).ok_or(
                GridPlacementDemandError::AxisCapacity {
                    axis: axes.major,
                    requested_tracks: major_span,
                },
            )?;
            grow_occupancy_axis(demand, occupancy, axes.major, major_end)?;
            let area = oriented_area(
                axes.column_flow,
                search.major,
                major_end,
                minor_start,
                minor_end,
            );
            if occupancy.is_free(area) {
                if !dense {
                    *cursor = search;
                }
                return Ok(area);
            }
            search.major =
                search
                    .major
                    .checked_add(1)
                    .ok_or(GridPlacementDemandError::AxisCapacity {
                        axis: axes.major,
                        requested_tracks: usize::MAX,
                    })?;
        }
    }

    let minor_span = demand.automatic_minor_span(minor, axes.minor);
    ensure_axis_track_count(demand, axes.minor, minor_span)?;
    loop {
        let minor_count = demand.published_axis_track_count(axes.minor);
        let minor_end =
            search
                .minor
                .checked_add(minor_span)
                .ok_or(GridPlacementDemandError::AxisCapacity {
                    axis: axes.minor,
                    requested_tracks: minor_span,
                })?;
        if minor_end > minor_count {
            search.major =
                search
                    .major
                    .checked_add(1)
                    .ok_or(GridPlacementDemandError::AxisCapacity {
                        axis: axes.major,
                        requested_tracks: usize::MAX,
                    })?;
            search.minor = 0;
            continue;
        }
        let major_end =
            search
                .major
                .checked_add(major_span)
                .ok_or(GridPlacementDemandError::AxisCapacity {
                    axis: axes.major,
                    requested_tracks: major_span,
                })?;
        grow_occupancy_axis(demand, occupancy, axes.major, major_end)?;
        let area = oriented_area(
            axes.column_flow,
            search.major,
            major_end,
            search.minor,
            minor_end,
        );
        if occupancy.is_free(area) {
            if !dense {
                *cursor = search;
            }
            return Ok(area);
        }
        search.minor =
            search
                .minor
                .checked_add(1)
                .ok_or(GridPlacementDemandError::AxisCapacity {
                    axis: axes.minor,
                    requested_tracks: usize::MAX,
                })?;
    }
}

fn oriented_area(
    column_flow: bool,
    major_start: usize,
    major_end: usize,
    minor_start: usize,
    minor_end: usize,
) -> PlacedGridArea {
    if column_flow {
        PlacedGridArea::new(major_start, minor_start, major_end, minor_end)
    } else {
        PlacedGridArea::new(minor_start, major_start, minor_end, major_end)
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

pub(super) fn is_in_flow_grid_child<S: LayoutScalar>(style: &GridItemProjection<S>) -> bool {
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

pub(super) fn absolute_grid_area<S: LayoutScalar>(
    input: AbsoluteGridAreaInput<'_, S>,
) -> LogicalAbsoluteGridArea<S> {
    let AbsoluteGridAreaInput {
        column,
        row,
        columns,
        rows,
        gap,
        column_geometry,
        row_geometry,
        column_offsets,
        row_offsets,
        constants,
        lines,
    } = input;
    let fallback_column_geometry;
    let column_geometry = if let Some(geometry) = column_geometry {
        geometry
    } else {
        fallback_column_geometry =
            UsedGridAxisGeometryOf::new(columns.to_vec(), vec![false; columns.len()], gap.inline);
        &fallback_column_geometry
    };
    let fallback_row_geometry;
    let row_geometry = if let Some(geometry) = row_geometry {
        geometry
    } else {
        fallback_row_geometry =
            UsedGridAxisGeometryOf::new(rows.to_vec(), vec![false; rows.len()], gap.block);
        &fallback_row_geometry
    };
    let flow_axes = constants.flow_axes;
    let content_size =
        LogicalSizeOf::new(column_geometry.total_extent(), row_geometry.total_extent());
    let padding = flow_axes.logical_edges(constants.padding);
    let border = flow_axes.logical_edges(constants.border);
    let padding_size = LogicalSizeOf::new(padding.inline_sum(), padding.block_sum());
    let border_size = LogicalSizeOf::new(border.inline_sum(), border.block_sum());
    let logical_inner_size = flow_axes.logical_size(constants.node_inner_size);
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
    let logical_outer_size = flow_axes.logical_size(constants.node_outer_size);
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
        geometry: column_geometry,
        padding_box_location: border.inline_start,
        padding_box_size: padding_box_size.inline,
        is_reverse: false,
        explicit_start: lines.column_explicit_start,
        explicit_count: lines.column_explicit_count,
    });
    let block = absolute_grid_axis_area(AbsoluteGridAxisInput {
        placement: row,
        tracks: rows,
        offsets: row_offsets,
        geometry: row_geometry,
        padding_box_location: border.block_start,
        padding_box_size: padding_box_size.block,
        is_reverse: false,
        explicit_start: lines.row_explicit_start,
        explicit_count: lines.row_explicit_count,
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
        geometry,
        padding_box_location,
        padding_box_size,
        is_reverse,
        explicit_start,
        explicit_count,
    } = input;
    let padding_box_end = padding_box_location + padding_box_size;
    if let (Some(start), None, None) = (placement.start(), placement.end(), placement.span())
        && let Some(line) = used_grid_line_offset(
            start.get(),
            geometry,
            is_reverse,
            explicit_start,
            explicit_count,
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
        && let Some(line) = used_grid_line_offset(
            end.get(),
            geometry,
            is_reverse,
            explicit_start,
            explicit_count,
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
            used_grid_line_offset(
                start_line.get(),
                geometry,
                is_reverse,
                explicit_start,
                explicit_count,
            ),
            used_grid_line_offset(
                end_line.get(),
                geometry,
                is_reverse,
                explicit_start,
                explicit_count,
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
        size: geometry.span_extent(start, end),
    }
}

fn used_grid_line_offset<S: LayoutScalar>(
    line: isize,
    geometry: &UsedGridAxisGeometryOf<S>,
    is_reverse: bool,
    explicit_start: usize,
    explicit_count: usize,
) -> Option<S> {
    let index = grid_line_to_index(line, geometry.sizes().len(), explicit_start, explicit_count)?;
    let offset = geometry.line_offset(index)?;
    if is_reverse {
        Some(geometry.line_offset(geometry.sizes().len())? - offset)
    } else {
        Some(offset)
    }
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
    resolve_grid_child_areas_with_geometry(input, None, None)
}

pub(super) fn resolve_grid_child_areas_with_geometry<Node, S: LayoutScalar>(
    input: ResolveGridChildAreasInput<'_, Node, S>,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
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
    if let Some(settled_areas) = &placements.settled_areas {
        debug_assert_eq!(settled_areas.len(), placements.items.len());
        return settled_areas
            .iter()
            .map(|area| {
                area.map(|area| GridArea {
                    column: area.column_start,
                    row: area.row_start,
                    column_end: area.column_end,
                    row_end: area.row_end,
                    size: LogicalSizeOf::new(
                        column_geometry.map_or_else(
                            || {
                                track_span_sum(
                                    columns,
                                    area.column_start,
                                    area.column_end,
                                    gap.inline,
                                )
                            },
                            |geometry| geometry.span_extent(area.column_start, area.column_end),
                        ),
                        row_geometry.map_or_else(
                            || track_span_sum(rows, area.row_start, area.row_end, gap.block),
                            |geometry| geometry.span_extent(area.row_start, area.row_end),
                        ),
                    ),
                })
            })
            .collect();
    }
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
    let source_order;
    let traversal = if style.display.establishes_grid_lanes_formatting_context() {
        source_order = placements
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| item.in_flow.then_some(crate::SourceIndex::new(index)))
            .collect::<Vec<_>>();
        &source_order
    } else {
        &placements.order_modified_indexes
    };
    place_grid_child_area_phase(
        placements,
        traversal,
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
        traversal,
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
    pub(super) placements: &'a GridPlacementContext<Node, S>,
    pub(super) style: &'a GridContainerProjection<'a, S>,
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
    placements: &GridPlacementContext<Node, S>,
    traversal: &[crate::SourceIndex],
    areas: &mut [Option<GridArea<S>>],
    occupancy: &mut [bool],
    phase: PlacementPhase,
    grid: PlacementContext<'_, S>,
) {
    debug_assert_eq!(areas.len(), placements.items.len());
    for source_index in traversal {
        let index = source_index.get();
        let placement = &placements.items[index];
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
