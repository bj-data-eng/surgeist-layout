use super::ordinary::{OrdinaryTrackState, intrinsic_at};
use super::*;

use crate::geometry::FlowAxes;
use crate::grid::input::GridContainerProjection;
use crate::scroll::{UsedOverflow, UsedOverflowAxis};

pub(in crate::grid) fn grid_axis_used_overflow<'a, S: LayoutScalar>(
    style: impl Into<GridContainerProjection<'a, S>>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> UsedOverflowAxis {
    let style = style.into();
    let overflow =
        UsedOverflow::from_computed(style.common.overflow, style.common.item_is_replaced);
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => overflow.x(),
        crate::geometry::PhysicalAxis::Vertical => overflow.y(),
    }
}

pub(in crate::grid) fn grid_axis_computed_overflow<'a, S: LayoutScalar>(
    style: impl Into<GridContainerProjection<'a, S>>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> Overflow {
    let style = style.into();
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => style.common.overflow.x(),
        crate::geometry::PhysicalAxis::Vertical => style.common.overflow.y(),
    }
}

pub(in crate::grid) fn grid_axis_physical_axis(
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> crate::geometry::PhysicalAxis {
    match axis.logical_axis() {
        crate::LogicalAxis::Inline => flow_axes.inline_axis(),
        crate::LogicalAxis::Block => flow_axes.block_axis(),
    }
}

pub(in crate::grid) fn grid_axis_size<T>(
    flow_axes: FlowAxes,
    size: Size<T>,
    axis: GridAxisKind,
) -> T {
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => size.width,
        crate::geometry::PhysicalAxis::Vertical => size.height,
    }
}

pub(in crate::grid) fn resolve_flex_fraction<S: LayoutScalar>(
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

pub(in crate::grid) fn find_size_of_fr<S: LayoutScalar>(
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

pub(in crate::grid) fn track_flex_factor<S: LayoutScalar>(track: &TrackSizingOf<S>) -> Option<S> {
    if let MaxTrackSizingOf::Flex(value) = &track.max {
        Some(value.get())
    } else {
        None
    }
}

fn track_has_auto_maximum<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    matches!(track.max, MaxTrackSizingOf::Auto)
}

pub(super) fn stretch_empty_auto_track_basis<S: LayoutScalar>(
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
            let mut state =
                OrdinaryTrackState::new(track, track_has_auto_maximum(track), collapsed);
            state.apply_intrinsic_contributions(
                basis,
                intrinsic_at(min_intrinsic_sizes, index),
                intrinsic_at(max_intrinsic_sizes, index),
            );
            state
        })
        .collect()
}

pub(super) fn resolve_ordinary_track_phases<S: LayoutScalar>(
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

#[cfg(test)]
pub(in crate::grid) fn track_sum<S: LayoutScalar>(sizes: &[S], gap: S) -> S {
    sizes
        .iter()
        .copied()
        .fold(S::ZERO, |sum, value| sum + value)
        + gap * S::from_usize(sizes.len().saturating_sub(1))
}

pub(in crate::grid) fn track_sum_with_gutters<S: LayoutScalar>(
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

pub(in crate::grid) fn track_span_sum_with_gutters<S: LayoutScalar>(
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

pub(super) fn span_contribution_with_gutters<S: LayoutScalar>(
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
pub(in crate::grid) fn track_content_sum<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    sizes: &[S],
    gap: S,
) -> S {
    track_sum(sizes, gap) + sub_one_flex_unfilled_space(tracks, sizes)
}

pub(in crate::grid) fn track_content_sum_with_gutters<S: LayoutScalar>(
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

pub(in crate::grid) fn offsets<S: LayoutScalar>(sizes: &[S], start: S, gap: S) -> Vec<S> {
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
pub(in crate::grid) fn rtl_offsets<S: LayoutScalar>(
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
