use super::*;

fn resolve_track_min_bounds<S: LayoutScalar>(
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
pub(super) struct OrdinaryTrackState<'a, S: LayoutScalar> {
    sizing_functions: &'a TrackSizingOf<S>,
    pub(super) base_size: S,
    growth_limit: Option<S>,
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

pub(in crate::grid) fn resolve_tracks_with_gutters<S: LayoutScalar>(
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

pub(in crate::grid) fn resolve_inline_tracks<S: LayoutScalar>(
    input: InlineTrackInput<'_, S>,
) -> Vec<S> {
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

#[cfg(test)]
pub(in crate::grid) fn distribute_tracks_between_bounds<S: LayoutScalar>(
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
