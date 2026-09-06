use super::*;

use super::flexible::{
    OrdinaryTrackSizingInput, resolve_ordinary_track_phases, span_contribution_with_gutters,
    stretch_empty_auto_track_basis,
};
use crate::LengthResolutionOf;

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

pub(super) fn resolution_optional<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> Option<S> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution.value,
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => None,
    }
}

#[derive(Clone)]
pub(super) struct OrdinaryTrackState<'a, S: LayoutScalar> {
    sizing_functions: &'a TrackSizingOf<S>,
    pub(super) base_size: S,
    pub(super) growth_limit: Option<S>,
    fit_content_limit: Option<S>,
    pub(super) flex_factor: Option<S>,
    pub(super) auto_max_stretch_eligible: bool,
    pub(super) collapsed: bool,
}

impl<'a, S: LayoutScalar> OrdinaryTrackState<'a, S> {
    pub(super) fn new(
        sizing_functions: &'a TrackSizingOf<S>,
        auto_max_stretch_eligible: bool,
        collapsed: bool,
    ) -> Self {
        Self {
            sizing_functions,
            base_size: S::ZERO,
            growth_limit: None,
            fit_content_limit: None,
            flex_factor: track_flex_factor(sizing_functions),
            auto_max_stretch_eligible,
            collapsed,
        }
    }

    pub(super) fn apply_intrinsic_contributions(
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
        // Intrinsic minima establish the base; fixed maxima only bound later growth.
        // Keep the already-resolved flexible contribution floor until the fr phase.
        self.base_size = if self.flex_factor.is_some() {
            max_intrinsic.max(track_min_size_for_intrinsics(
                &self.sizing_functions.min,
                basis,
                min_intrinsic,
                max_intrinsic,
            ))
        } else {
            let intrinsic = if self.sizing_functions.min == MinTrackSizingOf::MaxContent {
                max_intrinsic
            } else {
                min_intrinsic
            };
            track_min_size(&self.sizing_functions.min, basis, intrinsic)
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
            self.growth_limit = Some(growth_limit.max(self.base_size));
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct OrdinaryIntrinsicContributionInput<'a, S: LayoutScalar = Scalar> {
    pub(in crate::grid) tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) start: usize,
    pub(in crate::grid) end: usize,
    pub(in crate::grid) kind: IntrinsicSpanContribution,
    pub(in crate::grid) percent_basis: Option<S>,
    pub(in crate::grid) contribution: S,
    pub(in crate::grid) gap: S,
    pub(in crate::grid) gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
}

pub(in crate::grid) fn apply_ordinary_intrinsic_contribution<S: LayoutScalar>(
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
pub(in crate::grid) fn resolve_tracks<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    intrinsic_sizes: &[S],
) -> Vec<S> {
    resolve_tracks_with_gutters(tracks, basis, gap, alignment, intrinsic_sizes, None)
}

#[cfg(test)]
pub(in crate::grid) fn resolve_tracks_with_gutters<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    intrinsic_sizes: &[S],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Vec<S> {
    resolve_ordinary_track_phases(OrdinaryTrackSizingInput {
        tracks,
        percent_basis: basis,
        available: basis.map_or(AvailableOf::MAX_CONTENT, AvailableOf::Definite),
        stretch_size: basis,
        gap,
        alignment,
        min_intrinsic_sizes: intrinsic_sizes,
        max_intrinsic_sizes: intrinsic_sizes,
        gutters,
    })
}

pub(in crate::grid) fn resolve_axis_tracks<S: LayoutScalar>(
    input: AxisTrackInput<'_, S>,
) -> Vec<S> {
    let AxisTrackInput {
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
    let stretch_size = definite_size.or_else(|| {
        stretch_empty_auto_track_basis(
            tracks,
            available_size.into_option(),
            alignment,
            stretch_empty_auto_to_available,
            max_intrinsic_sizes,
        )
    });
    let phase_input = OrdinaryTrackSizingInput {
        tracks,
        percent_basis: basis,
        available: definite_size.map_or(available_size, AvailableOf::Definite),
        stretch_size,
        gap,
        alignment,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    };
    resolve_ordinary_track_phases(phase_input)
}

#[cfg(test)]
pub(in crate::grid) fn track_base_size_for_intrinsics<S: LayoutScalar>(
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

pub(in crate::grid) fn track_min_size_for_intrinsics<S: LayoutScalar>(
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

fn track_growth_floor_for_intrinsics<S: LayoutScalar>(
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

fn track_growth_limit_for_intrinsics<S: LayoutScalar>(
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

pub(super) fn intrinsic_at<S: LayoutScalar>(intrinsic_sizes: &[S], index: usize) -> S {
    intrinsic_sizes.get(index).copied().unwrap_or(S::ZERO)
}

pub(in crate::grid) fn track_resolution_intrinsic_sizes<S: LayoutScalar>(
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

pub(in crate::grid) fn track_base_size<S: LayoutScalar>(
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

fn track_min_size<S: LayoutScalar>(min: &MinTrackSizingOf<S>, basis: Option<S>, intrinsic: S) -> S {
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
pub(in crate::grid) fn track_growth_limit<S: LayoutScalar>(
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
