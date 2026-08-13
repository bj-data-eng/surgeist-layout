use super::*;

mod flexible;
mod intrinsic;
mod ordinary;
mod subgrid_intrinsic;
mod validation;

pub(super) use flexible::*;
pub(super) use intrinsic::*;
pub(super) use ordinary::{
    OrdinaryIntrinsicContributionInput, apply_ordinary_intrinsic_contribution,
    resolve_inline_tracks, resolve_tracks_with_gutters, track_base_size,
    track_resolution_intrinsic_sizes,
};
#[cfg(test)]
pub(super) use ordinary::{
    distribute_tracks_between_bounds, resolve_tracks, track_base_size_for_intrinsics,
    track_growth_limit, track_min_size_for_intrinsics,
};
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
