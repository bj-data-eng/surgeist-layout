use super::*;

mod intrinsic;
mod subgrid_intrinsic;
mod validation;

pub(super) use intrinsic::*;
#[cfg(test)]
pub(super) use subgrid_intrinsic::needs_intrinsic_subgrid_context;
pub(super) use subgrid_intrinsic::{
    PercentTrackContent, cyclic_percent_track_content_size, item_inherits_parent_axis,
    track_components_have_percent_sizing,
};
pub(super) use validation::*;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct OrdinaryGridAxisGuttersOf<S: LayoutScalar = Scalar> {
    collapsed: Vec<bool>,
    active_boundary_after: Vec<bool>,
    gutter_after: Vec<S>,
}

impl<S: LayoutScalar> OrdinaryGridAxisGuttersOf<S> {
    pub(super) fn new(track_count: usize, collapsed: &[bool], gap: S) -> Self {
        let mut collapsed = collapsed.to_vec();
        collapsed.resize(track_count, false);
        let active_boundary_after = Self::derive_active_boundary_after(&collapsed);
        let gutter_after = active_boundary_after
            .iter()
            .map(|active| if *active { gap } else { S::ZERO })
            .collect();
        Self {
            collapsed,
            active_boundary_after,
            gutter_after,
        }
    }

    pub(super) fn new_zero_adjacent_to_collapsed_tracks(
        track_count: usize,
        collapsed: &[bool],
        gap: S,
    ) -> Self {
        let mut collapsed = collapsed.to_vec();
        collapsed.resize(track_count, false);
        let active_boundary_after = collapsed
            .windows(2)
            .map(|pair| !pair[0] && !pair[1])
            .collect::<Vec<_>>();
        let gutter_after = active_boundary_after
            .iter()
            .map(|active| if *active { gap } else { S::ZERO })
            .collect();
        Self {
            collapsed,
            active_boundary_after,
            gutter_after,
        }
    }

    pub(super) fn from_active_boundary_gutters(
        track_count: usize,
        collapsed: &[bool],
        active_boundary_after: &[bool],
        gutter_after: &[S],
    ) -> Self {
        let mut collapsed = collapsed.to_vec();
        collapsed.resize(track_count, false);
        let mut active_boundary_after = active_boundary_after.to_vec();
        active_boundary_after.resize(track_count.saturating_sub(1), false);
        let mut gutter_after = gutter_after.to_vec();
        gutter_after.resize(track_count.saturating_sub(1), S::ZERO);
        Self {
            collapsed,
            active_boundary_after,
            gutter_after,
        }
    }

    fn derive_active_boundary_after(collapsed: &[bool]) -> Vec<bool> {
        let mut has_active_track_after = false;
        let mut active_boundary_after = vec![false; collapsed.len().saturating_sub(1)];
        for index in (0..collapsed.len()).rev() {
            if !collapsed[index] {
                if has_active_track_after && index < active_boundary_after.len() {
                    active_boundary_after[index] = true;
                }
                has_active_track_after = true;
            }
        }
        active_boundary_after
    }

    pub(super) fn collapsed(&self) -> &[bool] {
        &self.collapsed
    }

    pub(super) fn gutter_after(&self) -> &[S] {
        &self.gutter_after
    }

    pub(super) fn active_boundary_after(&self) -> &[bool] {
        &self.active_boundary_after
    }

    pub(super) fn reversed(&self) -> Self {
        let uses_coincident_interior_policy =
            self.active_boundary_after == Self::derive_active_boundary_after(&self.collapsed);
        let mut collapsed = self.collapsed.clone();
        collapsed.reverse();
        let (active_boundary_after, gutter_after) = if uses_coincident_interior_policy {
            let active_boundary_after = Self::derive_active_boundary_after(&collapsed);
            let mut active_gutters = self
                .gutter_after
                .iter()
                .copied()
                .zip(&self.active_boundary_after)
                .filter_map(|(gutter, active)| active.then_some(gutter))
                .collect::<Vec<_>>();
            active_gutters.reverse();
            let mut active_gutters = active_gutters.into_iter();
            let gutter_after = active_boundary_after
                .iter()
                .map(|active| {
                    if *active {
                        active_gutters.next().unwrap_or(S::ZERO)
                    } else {
                        S::ZERO
                    }
                })
                .collect();
            (active_boundary_after, gutter_after)
        } else {
            let mut active_boundary_after = self.active_boundary_after.clone();
            active_boundary_after.reverse();
            let mut gutter_after = self.gutter_after.clone();
            gutter_after.reverse();
            (active_boundary_after, gutter_after)
        };
        Self {
            collapsed,
            active_boundary_after,
            gutter_after,
        }
    }

    pub(super) fn active_gap_total(&self) -> S {
        self.gutter_after
            .iter()
            .copied()
            .fold(S::ZERO, |sum, gutter| sum + gutter)
    }

    pub(super) fn span_gap_total(&self, start: usize, end: usize) -> S {
        if start >= end || end > self.collapsed.len() {
            return S::ZERO;
        }
        self.gutter_after[start..end.saturating_sub(1)]
            .iter()
            .copied()
            .fold(S::ZERO, |sum, gutter| sum + gutter)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UsedGridAxisGeometryOf<S: LayoutScalar = Scalar> {
    sizes: Vec<S>,
    collapsed: Vec<bool>,
    active_boundary_after: Vec<bool>,
    gutter_after: Vec<S>,
    line_offsets: Vec<S>,
}

impl<S: LayoutScalar> UsedGridAxisGeometryOf<S> {
    pub(super) fn new(sizes: Vec<S>, collapsed: Vec<bool>, gap: S) -> Self {
        let gutters = OrdinaryGridAxisGuttersOf::new(sizes.len(), &collapsed, gap);
        Self::from_sizing_gutters(sizes, &gutters)
    }

    pub(super) fn from_sizing_gutters(
        sizes: Vec<S>,
        gutters: &OrdinaryGridAxisGuttersOf<S>,
    ) -> Self {
        Self::from_active_boundary_gutters(
            sizes,
            gutters.collapsed().to_vec(),
            gutters.active_boundary_after().to_vec(),
            gutters.gutter_after().to_vec(),
        )
    }

    pub(super) fn from_active_boundary_gutters(
        sizes: Vec<S>,
        collapsed: Vec<bool>,
        active_boundary_after: Vec<bool>,
        gutter_after: Vec<S>,
    ) -> Self {
        let mut collapsed = collapsed;
        collapsed.resize(sizes.len(), false);
        let mut active_boundary_after = active_boundary_after;
        active_boundary_after.resize(sizes.len().saturating_sub(1), false);
        let mut gutter_after = gutter_after;
        gutter_after.resize(sizes.len().saturating_sub(1), S::ZERO);
        let mut line_offsets = Vec::with_capacity(sizes.len() + 1);
        let mut cursor = S::ZERO;
        line_offsets.push(cursor);
        for (index, size) in sizes.iter().copied().enumerate() {
            cursor = cursor + size;
            if let Some(gutter) = gutter_after.get(index) {
                cursor = cursor + *gutter;
            }
            line_offsets.push(cursor);
        }
        Self {
            sizes,
            collapsed,
            active_boundary_after,
            gutter_after,
            line_offsets,
        }
    }

    pub(super) fn sizes(&self) -> &[S] {
        &self.sizes
    }

    pub(super) fn collapsed(&self) -> &[bool] {
        &self.collapsed
    }

    pub(super) fn gutter_after(&self) -> &[S] {
        &self.gutter_after
    }

    pub(super) fn active_boundary_after(&self) -> &[bool] {
        &self.active_boundary_after
    }

    pub(super) fn sizing_gutters(&self) -> OrdinaryGridAxisGuttersOf<S> {
        OrdinaryGridAxisGuttersOf::from_active_boundary_gutters(
            self.sizes.len(),
            &self.collapsed,
            &self.active_boundary_after,
            &self.gutter_after,
        )
    }

    pub(super) fn line_offsets(&self) -> &[S] {
        &self.line_offsets
    }

    pub(super) fn active_gap_total(&self) -> S {
        self.gutter_after
            .iter()
            .copied()
            .fold(S::ZERO, |sum, gutter| sum + gutter)
    }

    pub(super) fn total_extent(&self) -> S {
        self.sizes
            .iter()
            .copied()
            .fold(S::ZERO, |sum, size| sum + size)
            + self.active_gap_total()
    }

    pub(super) fn span_extent(&self, start: usize, end: usize) -> S {
        if start >= end || end > self.sizes.len() {
            return S::ZERO;
        }
        self.sizes[start..end]
            .iter()
            .copied()
            .fold(S::ZERO, |sum, size| sum + size)
            + self.gutter_after[start..end.saturating_sub(1)]
                .iter()
                .copied()
                .fold(S::ZERO, |sum, gutter| sum + gutter)
    }

    pub(super) fn line_offset(&self, line: usize) -> Option<S> {
        self.line_offsets.get(line).copied()
    }

    pub(super) fn translated(mut self, offset: S) -> Self {
        for line_offset in &mut self.line_offsets {
            *line_offset = *line_offset + offset;
        }
        self
    }
}
use crate::geometry::{FlowAxes, LogicalSizeOf};
use crate::scroll::{UsedOverflow, UsedOverflowAxis};
use crate::{
    LengthResolutionOf, LengthResolutionStatus, MaxTrackSizingOf, MinTrackSizingOf,
    PercentageBasisOf, SizingCalculationOf,
};

pub(super) fn resolve_track_calculation<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> LengthResolutionOf<S> {
    let basis = match basis {
        Some(value) => match PercentageBasisOf::definite(value) {
            Ok(basis) => basis,
            Err(_) => {
                return LengthResolutionOf::invalid_numeric(value, calculation.depends_on_basis());
            }
        },
        None => PercentageBasisOf::MISSING,
    };
    let resolution = calculation.resolve_against(basis);
    match resolution.status() {
        LengthResolutionStatus::Resolved => LengthResolutionOf::definite(
            resolution
                .value
                .expect("resolved sizing calculation must carry a value")
                .max(S::ZERO),
            calculation.depends_on_basis(),
        ),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            resolution
        }
        LengthResolutionStatus::NonNumeric => {
            unreachable!("a sizing calculation always has numeric program semantics")
        }
    }
}

fn resolve_track_calculation_optional<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> Option<S> {
    resolution_optional(resolve_track_calculation(calculation, basis))
}
fn axis_margin_sum<S: LayoutScalar>(margin: Edges<S>, axis: GridAxisKind) -> S {
    match axis {
        GridAxisKind::Column => margin.horizontal_sum(),
        GridAxisKind::Row => margin.vertical_sum(),
    }
}

fn axis_available<S: LayoutScalar>(
    available: Size<AvailableOf<S>>,
    axis: GridAxisKind,
) -> AvailableOf<S> {
    match axis {
        GridAxisKind::Column => available.width,
        GridAxisKind::Row => available.height,
    }
}

pub(super) fn grid_axis_used_overflow<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> UsedOverflowAxis {
    let overflow = UsedOverflow::from_computed(style.overflow, style.item_is_replaced);
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => overflow.x(),
        crate::geometry::PhysicalAxis::Vertical => overflow.y(),
    }
}

pub(super) fn grid_axis_computed_overflow<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> Overflow {
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => style.overflow.x(),
        crate::geometry::PhysicalAxis::Vertical => style.overflow.y(),
    }
}

fn grid_axis_physical_axis(
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> crate::geometry::PhysicalAxis {
    match axis.logical_axis() {
        crate::LogicalAxis::Inline => flow_axes.inline_axis(),
        crate::LogicalAxis::Block => flow_axes.block_axis(),
    }
}

pub(super) fn grid_axis_size<T>(flow_axes: FlowAxes, size: Size<T>, axis: GridAxisKind) -> T {
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => size.width,
        crate::geometry::PhysicalAxis::Vertical => size.height,
    }
}

pub(super) fn track_has_percent_sizing<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    track.depends_on_basis()
}

pub(super) fn resolve_flex_fraction<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    base_sizes: &[S],
    space_to_fill: Option<S>,
) -> S {
    if !tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizingOf::Flex(_)))
    {
        return S::ZERO;
    }

    if let Some(space_to_fill) = space_to_fill {
        return find_size_of_fr(tracks, base_sizes, space_to_fill.max(S::ZERO));
    }

    let flex_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            track_flex_factor(track).map(|factor| {
                if factor > S::ONE {
                    base_sizes[index] / factor
                } else {
                    base_sizes[index]
                }
            })
        })
        .fold(S::ZERO, S::max);
    let occupied_sub_one_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            let factor = track_flex_factor(track)?;
            (base_sizes.get(index).copied().unwrap_or(S::ZERO) > S::ZERO && factor < S::ONE)
                .then_some(factor)
        })
        .fold(S::ZERO, |sum, value| sum + value);

    if occupied_sub_one_fraction > S::ZERO && occupied_sub_one_fraction < S::ONE {
        flex_fraction * occupied_sub_one_fraction
    } else {
        flex_fraction
    }
}

pub(super) fn find_size_of_fr<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    base_sizes: &[S],
    space_to_fill: S,
) -> S {
    if space_to_fill <= S::ZERO {
        return S::ZERO;
    }

    let mut hypothetical = S::INFINITY;
    loop {
        let previous = hypothetical;
        let mut used_space = S::ZERO;
        let mut flex_sum = S::ZERO;
        for (index, track) in tracks.iter().enumerate() {
            if let Some(factor) = track_flex_factor(track)
                && factor * hypothetical >= base_sizes[index]
            {
                flex_sum = flex_sum + factor;
            } else {
                used_space = used_space + base_sizes[index];
            }
        }

        hypothetical = (space_to_fill - used_space) / flex_sum.max(S::ONE);
        let valid = tracks.iter().enumerate().all(|(index, track)| {
            if let Some(factor) = track_flex_factor(track) {
                factor * hypothetical >= base_sizes[index] || factor * previous < base_sizes[index]
            } else {
                true
            }
        });
        if valid {
            return hypothetical.max(S::ZERO);
        }
    }
}

pub(super) fn track_flex_factor<S: LayoutScalar>(track: &TrackSizingOf<S>) -> Option<S> {
    if let MaxTrackSizingOf::Flex(value) = &track.max {
        Some(value.get())
    } else {
        None
    }
}

fn track_has_auto_maximum<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    matches!(track.max, MaxTrackSizingOf::Auto)
}

fn stretch_empty_auto_track_basis<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    available_size: Option<S>,
    alignment: AlignContent,
    enabled: bool,
    max_intrinsic_sizes: &[S],
) -> Option<S> {
    if !enabled || alignment != AlignContent::Stretch {
        return None;
    }

    let has_empty_auto_track = tracks.iter().enumerate().any(|(index, track)| {
        matches!(
            track,
            TrackSizingOf {
                min: MinTrackSizingOf::Auto,
                max: MaxTrackSizingOf::Auto
            }
        ) && intrinsic_at(max_intrinsic_sizes, index) == S::ZERO
    });
    let has_non_auto_track = tracks.iter().any(|track| {
        !matches!(
            track,
            TrackSizingOf {
                min: MinTrackSizingOf::Auto,
                max: MaxTrackSizingOf::Auto
            }
        )
    });

    (has_empty_auto_track && has_non_auto_track)
        .then_some(available_size)
        .flatten()
}

pub(super) fn resolve_track_min_bounds<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
) -> Vec<S> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let intrinsic = match track.min {
                MinTrackSizingOf::MaxContent => intrinsic_at(max_intrinsic_sizes, index),
                _ => intrinsic_at(min_intrinsic_sizes, index),
            };
            track_min_size(&track.min, basis, intrinsic)
        })
        .collect()
}

#[derive(Clone)]
struct OrdinaryTrackState<'a, S: LayoutScalar> {
    sizing_functions: &'a TrackSizingOf<S>,
    base_size: S,
    growth_limit: Option<S>,
    fit_content_limit: Option<S>,
    flex_factor: Option<S>,
    auto_max_stretch_eligible: bool,
    collapsed: bool,
}

impl<'a, S: LayoutScalar> OrdinaryTrackState<'a, S> {
    fn new(sizing_functions: &'a TrackSizingOf<S>, collapsed: bool) -> Self {
        Self {
            sizing_functions,
            base_size: S::ZERO,
            growth_limit: None,
            fit_content_limit: None,
            flex_factor: track_flex_factor(sizing_functions),
            auto_max_stretch_eligible: track_has_auto_maximum(sizing_functions),
            collapsed,
        }
    }

    fn apply_intrinsic_contributions(
        &mut self,
        basis: Option<S>,
        min_intrinsic: S,
        max_intrinsic: S,
    ) {
        if self.collapsed {
            self.base_size = S::ZERO;
            self.growth_limit = Some(S::ZERO);
            self.fit_content_limit = None;
            self.flex_factor = None;
            self.auto_max_stretch_eligible = false;
            return;
        }

        self.fit_content_limit = match &self.sizing_functions.max {
            MaxTrackSizingOf::FitContent(limit) => Some(resolution_or_fallback(
                resolve_track_calculation(limit, basis),
                max_intrinsic,
            )),
            _ => None,
        };
        self.base_size = match self.fit_content_limit {
            Some(limit) => max_intrinsic.min(min_intrinsic.max(limit)),
            None => match self.sizing_functions.max {
                MaxTrackSizingOf::Flex(_) => max_intrinsic.max(track_min_size_for_intrinsics(
                    &self.sizing_functions.min,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                )),
                _ => track_base_size_for_intrinsics(
                    self.sizing_functions,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                ),
            },
        };
        self.growth_limit = self
            .fit_content_limit
            .map(|limit| max_intrinsic.min(min_intrinsic.max(limit)))
            .or_else(|| {
                track_growth_limit_for_intrinsics(
                    self.sizing_functions,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                )
            });
        let floor = track_growth_floor_for_intrinsics(
            self.sizing_functions,
            basis,
            min_intrinsic,
            max_intrinsic,
        );
        if let Some(growth_limit) = self.growth_limit {
            self.base_size = self.base_size.min(growth_limit.max(floor));
        }
    }
}

fn ordinary_track_states<'a, S: LayoutScalar>(
    tracks: &'a [TrackSizingOf<S>],
    basis: Option<S>,
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Vec<OrdinaryTrackState<'a, S>> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let collapsed = gutters
                .and_then(|gutters| gutters.collapsed().get(index))
                .copied()
                .unwrap_or(false);
            let mut state = OrdinaryTrackState::new(track, collapsed);
            state.apply_intrinsic_contributions(
                basis,
                intrinsic_at(min_intrinsic_sizes, index),
                intrinsic_at(max_intrinsic_sizes, index),
            );
            state
        })
        .collect()
}

fn resolve_ordinary_track_phases<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Vec<S> {
    let gap_total = gutters.map_or_else(
        || gap * S::from_usize(tracks.len().saturating_sub(1)),
        OrdinaryGridAxisGuttersOf::active_gap_total,
    );
    let mut states = ordinary_track_states(
        tracks,
        basis,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    );
    let base_sizes = states
        .iter()
        .map(|state| state.base_size)
        .collect::<Vec<_>>();
    let fr_size = resolve_flex_fraction(tracks, &base_sizes, basis.map(|size| size - gap_total));
    for state in &mut states {
        if let Some(flex_factor) = state.flex_factor {
            state.base_size = state.base_size.max(flex_factor * fr_size);
        }
    }

    let flex_used = states
        .iter()
        .filter(|state| state.flex_factor.is_some())
        .map(|state| state.base_size)
        .fold(S::ZERO, |sum, value| sum + value);
    let fixed_sum = states
        .iter()
        .filter(|state| state.flex_factor.is_none())
        .map(|state| state.base_size)
        .fold(S::ZERO, |sum, value| sum + value);
    let auto_count = states
        .iter()
        .filter(|state| state.auto_max_stretch_eligible && !state.collapsed)
        .count();
    let auto_size = if alignment == AlignContent::Stretch && auto_count > 0 {
        basis
            .map(|size| {
                ((size - gap_total - fixed_sum - flex_used).max(S::ZERO))
                    / S::from_usize(auto_count)
            })
            .unwrap_or(S::ZERO)
    } else {
        S::ZERO
    };
    states
        .into_iter()
        .map(|state| {
            if state.auto_max_stretch_eligible {
                state.base_size + auto_size
            } else {
                state.base_size
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
pub(super) struct OrdinaryIntrinsicContributionInput<'a, S: LayoutScalar = Scalar> {
    pub(super) tracks: &'a [TrackSizingOf<S>],
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) kind: IntrinsicSpanContribution,
    pub(super) percent_basis: Option<S>,
    pub(super) contribution: S,
    pub(super) gap: S,
    pub(super) gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
}

pub(super) fn apply_ordinary_intrinsic_contribution<S: LayoutScalar>(
    sizes: &mut [S],
    input: OrdinaryIntrinsicContributionInput<'_, S>,
) {
    let OrdinaryIntrinsicContributionInput {
        tracks,
        start,
        end,
        kind,
        percent_basis,
        contribution,
        gap,
        gutters,
    } = input;
    let Some(span_tracks) = tracks.get(start..end) else {
        return;
    };
    let Some(span_sizes) = sizes.get_mut(start..end) else {
        return;
    };
    if span_tracks.is_empty() || !span_tracks.iter().any(track_accepts_intrinsic_contribution) {
        return;
    }
    if span_tracks.len() == 1 {
        span_sizes[0] = span_sizes[0].max(contribution);
        return;
    }
    let mut target = span_contribution_with_gutters(contribution, start, end, gap, gutters);
    if matches!(kind, IntrinsicSpanContribution::MinContent { .. }) {
        target = (target - intrinsic_span_minimum_floor_space(span_tracks)).max(S::ZERO);
    }
    distribute_intrinsic_span(span_sizes, span_tracks, kind, percent_basis, target);
}

#[cfg(test)]
pub(super) fn resolve_tracks<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    intrinsic_sizes: &[S],
) -> Vec<S> {
    resolve_tracks_with_gutters(tracks, basis, gap, alignment, intrinsic_sizes, None)
}

pub(super) fn resolve_tracks_with_gutters<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    intrinsic_sizes: &[S],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Vec<S> {
    resolve_ordinary_track_phases(
        tracks,
        basis,
        gap,
        alignment,
        intrinsic_sizes,
        intrinsic_sizes,
        gutters,
    )
}

pub(super) fn resolve_inline_tracks<S: LayoutScalar>(input: InlineTrackInput<'_, S>) -> Vec<S> {
    let InlineTrackInput {
        tracks,
        basis,
        definite_size,
        available_size,
        gap,
        alignment,
        stretch_empty_auto_to_available,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    } = input;

    let max_tracks = resolve_ordinary_track_phases(
        tracks,
        basis,
        gap,
        AlignContent::Start,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    );
    let mut min_tracks =
        resolve_track_min_bounds(tracks, basis, min_intrinsic_sizes, max_intrinsic_sizes);
    if let Some(gutters) = gutters {
        for (size, collapsed) in min_tracks.iter_mut().zip(gutters.collapsed()) {
            if *collapsed {
                *size = S::ZERO;
            }
        }
    }
    let max_content = track_sum_with_gutters(&max_tracks, gap, gutters);
    let min_content = track_sum_with_gutters(&min_tracks, gap, gutters);
    if let Some(available_size) = definite_size.or(available_size)
        && max_content > S::ZERO
        && available_size < max_content
    {
        let target = available_size.max(min_content).min(max_content);
        return distribute_tracks_between_bounds_with_gutters(
            &min_tracks,
            &max_tracks,
            gap,
            gutters,
            target,
        );
    }

    let phase_basis = basis.or_else(|| {
        stretch_empty_auto_track_basis(
            tracks,
            available_size,
            alignment,
            stretch_empty_auto_to_available,
            max_intrinsic_sizes,
        )
    });
    resolve_ordinary_track_phases(
        tracks,
        phase_basis,
        gap,
        alignment,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    )
}

pub(super) fn track_base_size_for_intrinsics<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    min_intrinsic: S,
    max_intrinsic: S,
) -> S {
    let min = track_min_size_for_intrinsics(&track.min, basis, min_intrinsic, max_intrinsic);
    let max_base = match &track.max {
        MaxTrackSizingOf::Calculation(calculation) => {
            resolution_or_else(resolve_track_calculation(calculation, basis), || {
                if calculation.depends_on_basis() {
                    max_intrinsic
                } else {
                    resolution_or_zero(resolve_track_calculation(calculation, None))
                }
            })
        }
        MaxTrackSizingOf::Flex(_) => S::ZERO,
        MaxTrackSizingOf::Auto | MaxTrackSizingOf::MaxContent => max_intrinsic,
        MaxTrackSizingOf::MinContent => min_intrinsic,
        MaxTrackSizingOf::FitContent(limit) => {
            let limit =
                resolution_or_fallback(resolve_track_calculation(limit, basis), max_intrinsic);
            max_intrinsic.min(limit)
        }
    };
    min.max(max_base)
}

pub(super) fn track_min_size_for_intrinsics<S: LayoutScalar>(
    min: &MinTrackSizingOf<S>,
    basis: Option<S>,
    min_intrinsic: S,
    max_intrinsic: S,
) -> S {
    match min {
        MinTrackSizingOf::Calculation(calculation) => {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MaxContent => max_intrinsic,
        MinTrackSizingOf::MinContent => min_intrinsic,
    }
}

pub(super) fn track_growth_floor_for_intrinsics<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    min_intrinsic: S,
    max_intrinsic: S,
) -> S {
    match &track.min {
        MinTrackSizingOf::Auto => S::ZERO,
        min => track_min_size_for_intrinsics(min, basis, min_intrinsic, max_intrinsic),
    }
}

pub(super) fn track_growth_limit_for_intrinsics<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    min_intrinsic: S,
    max_intrinsic: S,
) -> Option<S> {
    match &track.max {
        MaxTrackSizingOf::Calculation(calculation) | MaxTrackSizingOf::FitContent(calculation) => {
            resolution_optional(resolve_track_calculation(calculation, basis))
                .or_else(|| calculation.depends_on_basis().then_some(max_intrinsic))
        }
        MaxTrackSizingOf::MinContent => Some(min_intrinsic),
        MaxTrackSizingOf::MaxContent | MaxTrackSizingOf::Auto => Some(max_intrinsic),
        MaxTrackSizingOf::Flex(_) => None,
    }
}

#[cfg(test)]
pub(super) fn distribute_tracks_between_bounds<S: LayoutScalar>(
    min_tracks: &[S],
    max_tracks: &[S],
    gap: S,
    target: S,
) -> Vec<S> {
    distribute_tracks_between_bounds_with_gutters(min_tracks, max_tracks, gap, None, target)
}

fn distribute_tracks_between_bounds_with_gutters<S: LayoutScalar>(
    min_tracks: &[S],
    max_tracks: &[S],
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
    target: S,
) -> Vec<S> {
    let min_sum = track_sum_with_gutters(min_tracks, gap, gutters);
    let max_sum = track_sum_with_gutters(max_tracks, gap, gutters);
    if target <= min_sum {
        return min_tracks.to_vec();
    }
    if target >= max_sum {
        return max_tracks.to_vec();
    }

    let mut resolved = max_tracks.to_vec();
    let shrink = (max_sum - target).max(S::ZERO);
    let shrink_capacity = max_tracks
        .iter()
        .zip(min_tracks)
        .map(|(max, min)| (*max - *min).max(S::ZERO))
        .fold(S::ZERO, |sum, value| sum + value);
    if shrink_capacity == S::ZERO {
        return resolved;
    }

    let ratio = (shrink / shrink_capacity).min(S::ONE);
    for (index, resolved) in resolved.iter_mut().enumerate() {
        let capacity = (max_tracks[index] - min_tracks[index]).max(S::ZERO);
        *resolved = *resolved - capacity * ratio;
    }
    resolved
}

pub(super) fn extend_auto_tracks<S: LayoutScalar>(
    tracks: &mut Vec<TrackSizingOf<S>>,
    auto_tracks: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    required_count: usize,
) -> Result<(), LengthResolutionStatus<S>> {
    let auto_tracks = expand_track_components(auto_tracks, basis, gap, None)?;
    let mut index = 0;
    while tracks.len() < required_count {
        let track = if auto_tracks.is_empty() {
            TrackSizingOf::AUTO
        } else {
            auto_tracks[index].clone()
        };
        tracks.push(track);
        if !auto_tracks.is_empty() {
            index = (index + 1) % auto_tracks.len();
        }
    }
    Ok(())
}

pub(super) fn prepend_auto_tracks<S: LayoutScalar>(
    tracks: &mut Vec<TrackSizingOf<S>>,
    auto_tracks: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    required_count: usize,
    auto_fit_limit: Option<usize>,
) -> Result<(), LengthResolutionStatus<S>> {
    if required_count == 0 {
        return Ok(());
    }

    let auto_tracks = expand_track_components(auto_tracks, basis, gap, auto_fit_limit)?;
    let generated = if auto_tracks.is_empty() {
        vec![TrackSizingOf::AUTO; required_count]
    } else {
        (0..required_count)
            .map(|index| {
                let phase = (auto_tracks.len() + index + auto_tracks.len()
                    - required_count % auto_tracks.len())
                    % auto_tracks.len();
                auto_tracks[phase].clone()
            })
            .collect::<Vec<_>>()
    };
    tracks.splice(0..0, generated);
    Ok(())
}

pub(super) fn intrinsic_at<S: LayoutScalar>(intrinsic_sizes: &[S], index: usize) -> S {
    intrinsic_sizes.get(index).copied().unwrap_or(S::ZERO)
}

pub(super) fn track_resolution_intrinsic_sizes<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
) -> Vec<S> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            if track.min == MinTrackSizingOf::MaxContent
                || match &track.max {
                    MaxTrackSizingOf::Auto
                    | MaxTrackSizingOf::Flex(_)
                    | MaxTrackSizingOf::MaxContent => true,
                    MaxTrackSizingOf::Calculation(calculation) => calculation.depends_on_basis(),
                    MaxTrackSizingOf::FitContent(_) | MaxTrackSizingOf::MinContent => false,
                }
            {
                intrinsic_at(max_intrinsic_sizes, index)
            } else if track.min == MinTrackSizingOf::MinContent
                || track.max == MaxTrackSizingOf::MinContent
            {
                intrinsic_at(min_intrinsic_sizes, index)
            } else {
                intrinsic_at(max_intrinsic_sizes, index)
            }
        })
        .collect()
}

pub(super) fn track_base_size<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    intrinsic: S,
) -> S {
    let min = track_min_size(&track.min, basis, intrinsic);
    let max_base = match &track.max {
        MaxTrackSizingOf::Calculation(calculation) => {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MaxTrackSizingOf::Flex(_) => S::ZERO,
        MaxTrackSizingOf::Auto | MaxTrackSizingOf::MinContent | MaxTrackSizingOf::MaxContent => {
            intrinsic
        }
        MaxTrackSizingOf::FitContent(limit) => {
            let limit = resolution_or_fallback(resolve_track_calculation(limit, basis), intrinsic);
            intrinsic.min(limit)
        }
    };
    min.max(max_base)
}

pub(super) fn track_min_size<S: LayoutScalar>(
    min: &MinTrackSizingOf<S>,
    basis: Option<S>,
    intrinsic: S,
) -> S {
    match min {
        MinTrackSizingOf::Calculation(calculation) => {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => {
            intrinsic
        }
    }
}

#[cfg(test)]
pub(super) fn track_growth_limit<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    intrinsic: S,
) -> Option<S> {
    match &track.max {
        MaxTrackSizingOf::Calculation(calculation) => {
            resolution_optional(resolve_track_calculation(calculation, basis))
        }
        MaxTrackSizingOf::FitContent(limit) => {
            let min = track_min_size(&track.min, basis, intrinsic);
            Some(intrinsic.max(min).min(resolution_or_fallback(
                resolve_track_calculation(limit, basis),
                intrinsic,
            )))
        }
        MaxTrackSizingOf::Flex(_)
        | MaxTrackSizingOf::Auto
        | MaxTrackSizingOf::MinContent
        | MaxTrackSizingOf::MaxContent => None,
    }
}

fn resolution_or_zero<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> S {
    resolution_or_fallback(resolution, S::ZERO)
}

fn resolution_or_fallback<S: LayoutScalar>(resolution: LengthResolutionOf<S>, fallback: S) -> S {
    resolution_or_else(resolution, || fallback)
}

fn resolution_or_else<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
    fallback: impl FnOnce() -> S,
) -> S {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution
            .value
            .expect("resolved length resolution must carry a value"),
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => fallback(),
    }
}

fn resolution_optional<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> Option<S> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution.value,
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => None,
    }
}

#[cfg(test)]
pub(super) fn track_sum<S: LayoutScalar>(sizes: &[S], gap: S) -> S {
    sizes
        .iter()
        .copied()
        .fold(S::ZERO, |sum, value| sum + value)
        + gap * S::from_usize(sizes.len().saturating_sub(1))
}

pub(super) fn track_sum_with_gutters<S: LayoutScalar>(
    sizes: &[S],
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    sizes
        .iter()
        .copied()
        .fold(S::ZERO, |sum, value| sum + value)
        + gutters.map_or_else(
            || gap * S::from_usize(sizes.len().saturating_sub(1)),
            OrdinaryGridAxisGuttersOf::active_gap_total,
        )
}

pub(super) fn track_span_sum_with_gutters<S: LayoutScalar>(
    sizes: &[S],
    start: usize,
    end: usize,
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    if start >= end || end > sizes.len() {
        return S::ZERO;
    }
    sizes[start..end]
        .iter()
        .copied()
        .fold(S::ZERO, |sum, size| sum + size)
        + gutters.map_or_else(
            || gap * S::from_usize(end.saturating_sub(start + 1)),
            |gutters| gutters.span_gap_total(start, end),
        )
}

fn span_contribution_with_gutters<S: LayoutScalar>(
    contribution: S,
    start: usize,
    end: usize,
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    let gutter_total = gutters.map_or_else(
        || gap * S::from_usize(end.saturating_sub(start + 1)),
        |gutters| gutters.span_gap_total(start, end),
    );
    (contribution - gutter_total).max(S::ZERO)
}

#[cfg(test)]
pub(super) fn track_content_sum<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    sizes: &[S],
    gap: S,
) -> S {
    track_sum(sizes, gap) + sub_one_flex_unfilled_space(tracks, sizes)
}

pub(super) fn track_content_sum_with_gutters<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    sizes: &[S],
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    track_sum_with_gutters(sizes, gap, gutters) + sub_one_flex_unfilled_space(tracks, sizes)
}

fn sub_one_flex_unfilled_space<S: LayoutScalar>(tracks: &[TrackSizingOf<S>], sizes: &[S]) -> S {
    let flex_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            let factor =
                track_flex_factor(track).filter(|factor| *factor > S::ZERO && *factor < S::ONE)?;
            let size = sizes.get(index).copied().unwrap_or(S::ZERO);
            (size > S::ZERO).then_some(size / factor)
        })
        .min_by(|left, right| scalar_total_cmp(*left, *right));

    let Some(flex_fraction) = flex_fraction else {
        return S::ZERO;
    };

    let mut occupied_fraction = S::ZERO;
    for (index, track) in tracks.iter().enumerate() {
        let Some(factor) =
            track_flex_factor(track).filter(|factor| *factor > S::ZERO && *factor < S::ONE)
        else {
            continue;
        };
        let size = sizes.get(index).copied().unwrap_or(S::ZERO);
        if size > factor * flex_fraction + S::from_f64(0.001) {
            occupied_fraction = occupied_fraction + factor;
        }
    }

    if occupied_fraction > S::ZERO && occupied_fraction < S::ONE {
        flex_fraction * (S::ONE - occupied_fraction)
    } else {
        S::ZERO
    }
}

pub(super) fn offsets<S: LayoutScalar>(sizes: &[S], start: S, gap: S) -> Vec<S> {
    let mut cursor = start;
    sizes
        .iter()
        .map(|size| {
            let offset = cursor;
            cursor = cursor + *size + gap;
            offset
        })
        .collect()
}

#[cfg(test)]
pub(super) fn rtl_offsets<S: LayoutScalar>(
    sizes: &[S],
    content_box_left: S,
    content_box_width: S,
    start: S,
    gap: S,
) -> Vec<S> {
    if content_box_width <= S::ZERO {
        return vec![content_box_left; sizes.len()];
    }

    let mut cursor = content_box_left + content_box_width - start;
    sizes
        .iter()
        .map(|size| {
            cursor = cursor - *size;
            let offset = cursor;
            cursor = cursor - gap;
            offset
        })
        .collect()
}
