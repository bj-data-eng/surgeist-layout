use crate::{LayoutScalar, LengthResolutionStatus, Scalar, TrackComponentOf, TrackSizingOf};

use super::named::{GridAreaNameFacts, GridNamedContext, NamedGridLines};
use super::placement::{GridPlacementDemandError, PlacedGridArea};
use super::tracks::{AutoRepeatTrackOrigin, TrackExpansionOf, expand_track_components};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ImplicitTrackSide {
    Leading,
    Trailing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExplicitTrackSizingOrigin {
    AuthoredTemplate,
    Inherited,
    TemplateAreaAutoPattern {
        pattern_index: usize,
    },
    ImplicitAutoPattern {
        pattern_index: usize,
        side: ImplicitTrackSide,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ExplicitTrackOrigin {
    pub(super) sizing: ExplicitTrackSizingOrigin,
    pub(super) auto_repeat: Option<AutoRepeatTrackOrigin>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ExpandedGridTopology<S: LayoutScalar = Scalar> {
    pub(super) column_tracks: Vec<TrackSizingOf<S>>,
    pub(super) row_tracks: Vec<TrackSizingOf<S>>,
    pub(super) explicit_columns: usize,
    pub(super) explicit_rows: usize,
    pub(super) column_explicit_start: usize,
    pub(super) row_explicit_start: usize,
    pub(super) named_columns: NamedGridLines,
    pub(super) named_rows: NamedGridLines,
    pub(super) area_facts: Option<GridAreaNameFacts>,
    pub(super) column_origins: Vec<ExplicitTrackOrigin>,
    pub(super) row_origins: Vec<ExplicitTrackOrigin>,
    pub(super) collapsed_columns: Vec<bool>,
    pub(super) collapsed_rows: Vec<bool>,
    column_auto_pattern: Vec<TrackSizingOf<S>>,
    row_auto_pattern: Vec<TrackSizingOf<S>>,
    inherited_columns: bool,
    inherited_rows: bool,
}

pub(super) struct ExpandedGridTopologyInput<'a, S: LayoutScalar = Scalar> {
    pub(super) columns: TrackExpansionOf<S>,
    pub(super) rows: TrackExpansionOf<S>,
    pub(super) named: GridNamedContext,
    pub(super) auto_columns: &'a [TrackComponentOf<S>],
    pub(super) auto_rows: &'a [TrackComponentOf<S>],
    pub(super) column_basis: Option<S>,
    pub(super) row_basis: Option<S>,
    pub(super) column_gap: S,
    pub(super) row_gap: S,
    pub(super) inherited_columns: bool,
    pub(super) inherited_rows: bool,
}

struct ExpandedAxisTopology<S: LayoutScalar = Scalar> {
    tracks: Vec<TrackSizingOf<S>>,
    origins: Vec<ExplicitTrackOrigin>,
    named_lines: NamedGridLines,
}

impl<S: LayoutScalar> ExpandedGridTopology<S> {
    pub(super) fn new(
        input: ExpandedGridTopologyInput<'_, S>,
    ) -> Result<Self, LengthResolutionStatus<S>> {
        let GridNamedContext {
            columns: named_columns,
            rows: named_rows,
            area_facts,
        } = input.named;
        let columns = complete_explicit_axis(
            input.columns,
            named_columns,
            input.auto_columns,
            input.column_basis,
            input.column_gap,
            input.inherited_columns,
        )?;
        let rows = complete_explicit_axis(
            input.rows,
            named_rows,
            input.auto_rows,
            input.row_basis,
            input.row_gap,
            input.inherited_rows,
        )?;
        let column_auto_pattern = expand_track_components(
            input.auto_columns,
            input.column_basis,
            input.column_gap,
            None,
        )?;
        let row_auto_pattern =
            expand_track_components(input.auto_rows, input.row_basis, input.row_gap, None)?;
        let explicit_columns = columns.tracks.len();
        let explicit_rows = rows.tracks.len();
        let topology = Self {
            explicit_columns,
            explicit_rows,
            column_explicit_start: 0,
            row_explicit_start: 0,
            column_tracks: columns.tracks,
            row_tracks: rows.tracks,
            named_columns: columns.named_lines,
            named_rows: rows.named_lines,
            area_facts,
            column_origins: columns.origins,
            row_origins: rows.origins,
            collapsed_columns: vec![false; explicit_columns],
            collapsed_rows: vec![false; explicit_rows],
            column_auto_pattern,
            row_auto_pattern,
            inherited_columns: input.inherited_columns,
            inherited_rows: input.inherited_rows,
        };
        debug_assert!(topology.has_complete_origin_evidence());
        Ok(topology)
    }

    pub(super) fn has_complete_origin_evidence(&self) -> bool {
        axis_origin_evidence_is_complete(
            &self.column_tracks,
            self.column_explicit_start,
            self.explicit_columns,
            &self.named_columns,
            &self.column_origins,
        ) && axis_origin_evidence_is_complete(
            &self.row_tracks,
            self.row_explicit_start,
            self.explicit_rows,
            &self.named_rows,
            &self.row_origins,
        )
    }

    #[cfg(test)]
    pub(super) fn from_test_parts(
        column_tracks: Vec<TrackSizingOf<S>>,
        row_tracks: Vec<TrackSizingOf<S>>,
        named_columns: NamedGridLines,
        named_rows: NamedGridLines,
        area_facts: Option<GridAreaNameFacts>,
    ) -> Self {
        let explicit_columns = column_tracks.len();
        let explicit_rows = row_tracks.len();
        Self {
            column_tracks,
            row_tracks,
            explicit_columns,
            explicit_rows,
            column_explicit_start: 0,
            row_explicit_start: 0,
            named_columns,
            named_rows,
            area_facts,
            column_origins: vec![
                ExplicitTrackOrigin {
                    sizing: ExplicitTrackSizingOrigin::AuthoredTemplate,
                    auto_repeat: None,
                };
                explicit_columns
            ],
            row_origins: vec![
                ExplicitTrackOrigin {
                    sizing: ExplicitTrackSizingOrigin::AuthoredTemplate,
                    auto_repeat: None,
                };
                explicit_rows
            ],
            collapsed_columns: vec![false; explicit_columns],
            collapsed_rows: vec![false; explicit_rows],
            column_auto_pattern: vec![TrackSizingOf::AUTO],
            row_auto_pattern: vec![TrackSizingOf::AUTO],
            inherited_columns: false,
            inherited_rows: false,
        }
    }

    pub(super) fn axis_is_inherited(&self, axis: super::GridAxisKind) -> bool {
        match axis {
            super::GridAxisKind::Column => self.inherited_columns,
            super::GridAxisKind::Row => self.inherited_rows,
        }
    }

    pub(super) fn apply_placement_demand(
        &mut self,
        column_explicit_start: usize,
        row_explicit_start: usize,
        column_count: usize,
        row_count: usize,
    ) -> Result<(), GridPlacementDemandError> {
        let column_growth = implicit_axis_growth(
            super::GridAxisKind::Column,
            &self.column_auto_pattern,
            self.explicit_columns,
            column_explicit_start,
            column_count,
        )?;
        let row_growth = implicit_axis_growth(
            super::GridAxisKind::Row,
            &self.row_auto_pattern,
            self.explicit_rows,
            row_explicit_start,
            row_count,
        )?;

        reserve_axis_growth(
            super::GridAxisKind::Column,
            &mut self.column_tracks,
            &mut self.column_origins,
            column_growth
                .total_count()
                .ok_or(GridPlacementDemandError::AxisCapacity {
                    axis: super::GridAxisKind::Column,
                    requested_tracks: column_count,
                })?,
        )?;
        reserve_axis_growth(
            super::GridAxisKind::Row,
            &mut self.row_tracks,
            &mut self.row_origins,
            row_growth
                .total_count()
                .ok_or(GridPlacementDemandError::AxisCapacity {
                    axis: super::GridAxisKind::Row,
                    requested_tracks: row_count,
                })?,
        )?;

        apply_axis_growth(
            &mut self.column_tracks,
            &mut self.column_origins,
            column_growth,
        );
        apply_axis_growth(&mut self.row_tracks, &mut self.row_origins, row_growth);
        self.collapsed_columns
            .resize(self.column_tracks.len(), false);
        self.collapsed_rows.resize(self.row_tracks.len(), false);
        self.column_explicit_start = column_explicit_start;
        self.row_explicit_start = row_explicit_start;
        debug_assert!(self.has_complete_origin_evidence());
        Ok(())
    }

    pub(super) fn collapse_ordinary_auto_fit(&mut self, settled_areas: &[Option<PlacedGridArea>]) {
        collapse_auto_fit_axis(
            &self.column_origins,
            settled_areas,
            super::GridAxisKind::Column,
            &mut self.collapsed_columns,
        );
        collapse_auto_fit_axis(
            &self.row_origins,
            settled_areas,
            super::GridAxisKind::Row,
            &mut self.collapsed_rows,
        );
    }

    pub(super) fn apply_lanes_auto_fit_policy(
        &mut self,
        axis: super::GridAxisKind,
        track_count: usize,
        explicit_start: usize,
        placements: &[LanesAutoFitPlacement],
    ) -> Result<(), GridPlacementDemandError> {
        if self.axis_is_inherited(axis) {
            return Ok(());
        }
        let (origins, collapsed) = match axis {
            super::GridAxisKind::Column => (&self.column_origins, &mut self.collapsed_columns),
            super::GridAxisKind::Row => (&self.row_origins, &mut self.collapsed_rows),
        };
        collapse_lanes_auto_fit_axis(
            origins,
            track_count,
            explicit_start,
            placements,
            axis,
            collapsed,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LanesAutoFitPlacement {
    pub(super) definite_start: Option<usize>,
    pub(super) span: usize,
}

fn collapse_lanes_auto_fit_axis(
    origins: &[ExplicitTrackOrigin],
    track_count: usize,
    explicit_start: usize,
    placements: &[LanesAutoFitPlacement],
    axis: super::GridAxisKind,
    collapsed: &mut Vec<bool>,
) -> Result<(), GridPlacementDemandError> {
    collapsed.clear();
    collapsed.resize(track_count, false);
    let automatic_demand = placements
        .iter()
        .filter(|placement| placement.definite_start.is_none())
        .try_fold(0usize, |sum, placement| sum.checked_add(placement.span))
        .ok_or(GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks: usize::MAX,
        })?;
    let mut explicitly_occupied = vec![false; track_count];
    for placement in placements.iter().filter_map(|placement| {
        placement
            .definite_start
            .map(|start| (start, placement.span))
    }) {
        let end =
            placement
                .0
                .checked_add(placement.1)
                .ok_or(GridPlacementDemandError::AxisCapacity {
                    axis,
                    requested_tracks: usize::MAX,
                })?;
        for occupied in explicitly_occupied
            .get_mut(placement.0..end.min(track_count))
            .into_iter()
            .flatten()
        {
            *occupied = true;
        }
    }

    let mut remaining_demand = automatic_demand;
    for (origin_index, origin) in origins.iter().enumerate() {
        if !origin
            .auto_repeat
            .is_some_and(|origin| origin.kind == crate::TrackRepeat::AutoFit)
        {
            continue;
        }
        let Some(index) = explicit_start.checked_add(origin_index) else {
            return Err(GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: usize::MAX,
            });
        };
        let Some(collapsed) = collapsed.get_mut(index) else {
            continue;
        };
        if explicitly_occupied[index] {
            continue;
        }
        if remaining_demand > 0 {
            remaining_demand -= 1;
        } else {
            *collapsed = true;
        }
    }
    Ok(())
}

fn collapse_auto_fit_axis(
    origins: &[ExplicitTrackOrigin],
    settled_areas: &[Option<PlacedGridArea>],
    axis: super::GridAxisKind,
    collapsed: &mut Vec<bool>,
) {
    collapsed.clear();
    collapsed.resize(origins.len(), false);
    for (index, origin) in origins.iter().enumerate() {
        if !origin
            .auto_repeat
            .is_some_and(|origin| origin.kind == crate::TrackRepeat::AutoFit)
        {
            continue;
        }
        let occupied = settled_areas.iter().flatten().any(|area| match axis {
            super::GridAxisKind::Column => area.column_start <= index && index < area.column_end,
            super::GridAxisKind::Row => area.row_start <= index && index < area.row_end,
        });
        collapsed[index] = !occupied;
    }
}

struct ImplicitAxisGrowth<S: LayoutScalar = Scalar> {
    leading_tracks: Vec<TrackSizingOf<S>>,
    leading_origins: Vec<ExplicitTrackOrigin>,
    trailing_tracks: Vec<TrackSizingOf<S>>,
    trailing_origins: Vec<ExplicitTrackOrigin>,
}

impl<S: LayoutScalar> ImplicitAxisGrowth<S> {
    fn total_count(&self) -> Option<usize> {
        self.leading_tracks
            .len()
            .checked_add(self.trailing_tracks.len())
    }
}

fn implicit_axis_growth<S: LayoutScalar>(
    axis: super::GridAxisKind,
    pattern: &[TrackSizingOf<S>],
    explicit_count: usize,
    explicit_start: usize,
    total_count: usize,
) -> Result<ImplicitAxisGrowth<S>, GridPlacementDemandError> {
    let explicit_end = explicit_start.checked_add(explicit_count).ok_or(
        GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks: total_count,
        },
    )?;
    let trailing =
        total_count
            .checked_sub(explicit_end)
            .ok_or(GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: total_count,
            })?;
    let mut leading_tracks = Vec::new();
    let mut leading_origins = Vec::new();
    let mut trailing_tracks = Vec::new();
    let mut trailing_origins = Vec::new();
    for values in [
        (&mut leading_tracks, explicit_start),
        (&mut trailing_tracks, trailing),
    ] {
        values.0.try_reserve_exact(values.1).map_err(|_| {
            GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: total_count,
            }
        })?;
    }
    for values in [
        (&mut leading_origins, explicit_start),
        (&mut trailing_origins, trailing),
    ] {
        values.0.try_reserve_exact(values.1).map_err(|_| {
            GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: total_count,
            }
        })?;
    }

    for offset in 0..explicit_start {
        let distance_before_explicit = explicit_start - offset;
        let pattern_index = if pattern.is_empty() {
            0
        } else {
            (pattern.len() - distance_before_explicit % pattern.len()) % pattern.len()
        };
        leading_tracks.push(
            pattern
                .get(pattern_index)
                .cloned()
                .unwrap_or(TrackSizingOf::AUTO),
        );
        leading_origins.push(ExplicitTrackOrigin {
            sizing: ExplicitTrackSizingOrigin::ImplicitAutoPattern {
                pattern_index,
                side: ImplicitTrackSide::Leading,
            },
            auto_repeat: None,
        });
    }
    for offset in 0..trailing {
        let pattern_index = if pattern.is_empty() {
            0
        } else {
            offset % pattern.len()
        };
        trailing_tracks.push(
            pattern
                .get(pattern_index)
                .cloned()
                .unwrap_or(TrackSizingOf::AUTO),
        );
        trailing_origins.push(ExplicitTrackOrigin {
            sizing: ExplicitTrackSizingOrigin::ImplicitAutoPattern {
                pattern_index,
                side: ImplicitTrackSide::Trailing,
            },
            auto_repeat: None,
        });
    }
    Ok(ImplicitAxisGrowth {
        leading_tracks,
        leading_origins,
        trailing_tracks,
        trailing_origins,
    })
}

fn reserve_axis_growth<S: LayoutScalar>(
    axis: super::GridAxisKind,
    tracks: &mut Vec<TrackSizingOf<S>>,
    origins: &mut Vec<ExplicitTrackOrigin>,
    additional: usize,
) -> Result<(), GridPlacementDemandError> {
    let requested_tracks =
        tracks
            .len()
            .checked_add(additional)
            .ok_or(GridPlacementDemandError::AxisCapacity {
                axis,
                requested_tracks: usize::MAX,
            })?;
    tracks
        .try_reserve_exact(additional)
        .map_err(|_| GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks,
        })?;
    origins
        .try_reserve_exact(additional)
        .map_err(|_| GridPlacementDemandError::AxisCapacity {
            axis,
            requested_tracks,
        })?;
    Ok(())
}

fn apply_axis_growth<S: LayoutScalar>(
    tracks: &mut Vec<TrackSizingOf<S>>,
    origins: &mut Vec<ExplicitTrackOrigin>,
    growth: ImplicitAxisGrowth<S>,
) {
    tracks.splice(0..0, growth.leading_tracks);
    origins.splice(0..0, growth.leading_origins);
    tracks.extend(growth.trailing_tracks);
    origins.extend(growth.trailing_origins);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GridOccupancy {
    columns: usize,
    rows: usize,
    cells: Vec<bool>,
}

impl GridOccupancy {
    pub(super) fn new(columns: usize, rows: usize) -> Result<Self, GridPlacementDemandError> {
        let cell_count = columns
            .checked_mul(rows)
            .ok_or(GridPlacementDemandError::OccupancyCapacity { columns, rows })?;
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(cell_count)
            .map_err(|_| GridPlacementDemandError::OccupancyCapacity { columns, rows })?;
        cells.resize(cell_count, false);
        Ok(Self {
            columns,
            rows,
            cells,
        })
    }

    pub(super) fn grow_to(
        &mut self,
        columns: usize,
        rows: usize,
    ) -> Result<(), GridPlacementDemandError> {
        if columns <= self.columns && rows <= self.rows {
            return Ok(());
        }
        let columns = columns.max(self.columns);
        let rows = rows.max(self.rows);
        let cell_count = columns
            .checked_mul(rows)
            .ok_or(GridPlacementDemandError::OccupancyCapacity { columns, rows })?;
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(cell_count)
            .map_err(|_| GridPlacementDemandError::OccupancyCapacity { columns, rows })?;
        cells.resize(cell_count, false);
        for row in 0..self.rows {
            for column in 0..self.columns {
                cells[row * columns + column] = self.cells[row * self.columns + column];
            }
        }
        self.columns = columns;
        self.rows = rows;
        self.cells = cells;
        Ok(())
    }

    pub(super) fn is_free(&self, area: PlacedGridArea) -> bool {
        area.column_end <= self.columns
            && area.row_end <= self.rows
            && (area.row_start..area.row_end).all(|row| {
                (area.column_start..area.column_end)
                    .all(|column| !self.cells[row * self.columns + column])
            })
    }

    pub(super) fn occupy(&mut self, area: PlacedGridArea) {
        debug_assert!(area.column_start < area.column_end);
        debug_assert!(area.row_start < area.row_end);
        debug_assert!(area.column_end <= self.columns);
        debug_assert!(area.row_end <= self.rows);
        for row in area.row_start..area.row_end {
            for column in area.column_start..area.column_end {
                self.cells[row * self.columns + column] = true;
            }
        }
    }
}

fn complete_explicit_axis<S: LayoutScalar>(
    expansion: TrackExpansionOf<S>,
    named_lines: NamedGridLines,
    auto_components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    inherited: bool,
) -> Result<ExpandedAxisTopology<S>, LengthResolutionStatus<S>> {
    let mut tracks = Vec::with_capacity(named_lines.explicit_track_count);
    let mut origins = Vec::with_capacity(named_lines.explicit_track_count);
    for track in expansion.tracks {
        tracks.push(track.sizing);
        origins.push(ExplicitTrackOrigin {
            sizing: if inherited {
                ExplicitTrackSizingOrigin::Inherited
            } else {
                ExplicitTrackSizingOrigin::AuthoredTemplate
            },
            auto_repeat: track.auto_repeat,
        });
    }

    if !inherited && tracks.len() < named_lines.explicit_track_count {
        let auto_pattern = expand_track_components(auto_components, basis, gap, None)?;
        let missing = named_lines.explicit_track_count - tracks.len();
        for pattern_offset in 0..missing {
            let (sizing, pattern_index) = if auto_pattern.is_empty() {
                (TrackSizingOf::AUTO, 0)
            } else {
                let pattern_index = pattern_offset % auto_pattern.len();
                (auto_pattern[pattern_index].clone(), pattern_index)
            };
            tracks.push(sizing);
            origins.push(ExplicitTrackOrigin {
                sizing: ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index },
                auto_repeat: None,
            });
        }
    }

    debug_assert_eq!(tracks.len(), named_lines.explicit_track_count);
    Ok(ExpandedAxisTopology {
        tracks,
        origins,
        named_lines,
    })
}

fn axis_origin_evidence_is_complete<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    explicit_start: usize,
    explicit_count: usize,
    named_lines: &NamedGridLines,
    origins: &[ExplicitTrackOrigin],
) -> bool {
    tracks.len() == origins.len()
        && named_lines.explicit_track_count == explicit_count
        && explicit_start
            .checked_add(explicit_count)
            .is_some_and(|explicit_end| explicit_end <= tracks.len())
        && origins
            .iter()
            .enumerate()
            .all(|(index, origin)| match origin.sizing {
                ExplicitTrackSizingOrigin::AuthoredTemplate => true,
                ExplicitTrackSizingOrigin::Inherited => origin.auto_repeat.is_none(),
                ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index } => {
                    origin.auto_repeat.is_none() && pattern_index < explicit_count.max(1)
                }
                ExplicitTrackSizingOrigin::ImplicitAutoPattern {
                    pattern_index,
                    side,
                } => {
                    origin.auto_repeat.is_none()
                        && pattern_index < tracks.len().max(1)
                        && match side {
                            ImplicitTrackSide::Leading => index < explicit_start,
                            ImplicitTrackSide::Trailing => explicit_start
                                .checked_add(explicit_count)
                                .is_some_and(|explicit_end| index >= explicit_end),
                        }
                }
            })
}
