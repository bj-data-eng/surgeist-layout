use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) struct AutoRepeatTrackOrigin {
    pub(in crate::grid) kind: TrackRepeat,
    pub(in crate::grid) repeat_group: usize,
    pub(in crate::grid) repetition_index: usize,
    pub(in crate::grid) track_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct ExpandedTrackOf<S: LayoutScalar = Scalar> {
    pub(in crate::grid) sizing: TrackSizingOf<S>,
    pub(in crate::grid) auto_repeat: Option<AutoRepeatTrackOrigin>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct TrackExpansionOf<S: LayoutScalar = Scalar> {
    pub(in crate::grid) tracks: Vec<ExpandedTrackOf<S>>,
}

impl<S: LayoutScalar> TrackExpansionOf<S> {
    pub(in crate::grid) fn inherited(tracks: Vec<TrackSizingOf<S>>) -> Self {
        Self {
            tracks: tracks
                .into_iter()
                .map(|sizing| ExpandedTrackOf {
                    sizing,
                    auto_repeat: None,
                })
                .collect(),
        }
    }
}

pub(in crate::grid) fn expand_track_components_with_origins<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    auto_fit_limit: Option<usize>,
) -> Result<TrackExpansionOf<S>, LengthResolutionStatus<S>> {
    if subgrid_components(components) {
        return Ok(TrackExpansionOf { tracks: Vec::new() });
    }

    validate_track_components(components, basis)?;
    let mut tracks = Vec::new();
    let mut auto_repeat_group = 0;
    let reserved = reserved_track_space(components, basis, gap);
    for component in components {
        match component {
            TrackComponentOf::Track(sizing) => tracks.push(ExpandedTrackOf {
                sizing: sizing.clone(),
                auto_repeat: None,
            }),
            TrackComponentOf::Repeat(repetition) => {
                let repeated_tracks = repetition.sizing_tracks();
                let count = match repetition.repeat() {
                    TrackRepeat::Count(count) => count.get(),
                    TrackRepeat::AutoFill => {
                        auto_repeat_count(&repeated_tracks, basis, gap, reserved)
                    }
                    TrackRepeat::AutoFit => {
                        auto_repeat_count(&repeated_tracks, basis, gap, reserved)
                            .min(auto_fit_limit.unwrap_or(usize::MAX))
                            .max(1)
                    }
                };
                let repeat_group = auto_repeat_group;
                let auto_repeat_kind = match repetition.repeat() {
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                        auto_repeat_group += 1;
                        Some(repetition.repeat())
                    }
                    TrackRepeat::Count(_) => None,
                };
                for repetition_index in 0..count {
                    tracks.extend(repeated_tracks.iter().cloned().enumerate().map(
                        |(track_index, sizing)| ExpandedTrackOf {
                            sizing,
                            auto_repeat: auto_repeat_kind.map(|kind| AutoRepeatTrackOrigin {
                                kind,
                                repeat_group,
                                repetition_index,
                                track_index,
                            }),
                        },
                    ));
                }
            }
            TrackComponentOf::LineNames(_) => {}
            TrackComponentOf::Subgrid(_) => {
                unreachable!("subgrid templates return before expansion")
            }
        }
    }
    Ok(TrackExpansionOf { tracks })
}

pub(in crate::grid) fn expand_track_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    auto_fit_limit: Option<usize>,
) -> Result<Vec<TrackSizingOf<S>>, LengthResolutionStatus<S>> {
    Ok(
        expand_track_components_with_origins(components, basis, gap, auto_fit_limit)?
            .tracks
            .into_iter()
            .map(|track| track.sizing)
            .collect(),
    )
}

fn validate_track_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    for component in components {
        match component {
            TrackComponentOf::Track(track) => validate_track_sizing(track, basis)?,
            TrackComponentOf::Repeat(repetition) => {
                validate_track_components(repetition.components(), basis)?;
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    Ok(())
}

fn validate_track_sizing<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    if let MinTrackSizingOf::Calculation(calculation) = &track.min {
        validate_track_calculation(calculation, basis)?;
    }
    match &track.max {
        MaxTrackSizingOf::Calculation(calculation) | MaxTrackSizingOf::FitContent(calculation) => {
            validate_track_calculation(calculation, basis)?;
        }
        MaxTrackSizingOf::Flex(_)
        | MaxTrackSizingOf::Auto
        | MaxTrackSizingOf::MinContent
        | MaxTrackSizingOf::MaxContent => {}
    }
    Ok(())
}

fn validate_track_calculation<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    let resolution = resolve_track_calculation(calculation, basis);
    match resolution.status() {
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::Resolved | LengthResolutionStatus::MissingBasis => Ok(()),
        LengthResolutionStatus::NonNumeric => Err(LengthResolutionStatus::NonNumeric),
    }
}

pub(in crate::grid) fn track_expansion_basis<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    node_basis: Option<S>,
    available_basis: Option<S>,
) -> Option<S> {
    if subgrid_components(components) {
        return None;
    }

    node_basis.or_else(|| {
        auto_repeat_components(components)
            .then_some(available_basis)
            .flatten()
    })
}

pub(in crate::grid) fn subgrid_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> bool {
    components
        .iter()
        .any(|component| matches!(component, TrackComponentOf::Subgrid(_)))
}

pub(in crate::grid) fn auto_repeat_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> bool {
    components.iter().any(|component| {
        matches!(
            component,
            TrackComponentOf::Repeat(repetition)
                if matches!(repetition.repeat(), TrackRepeat::AutoFill | TrackRepeat::AutoFit)
        )
    })
}

pub(in crate::grid) fn tracks_need_available_basis<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
) -> bool {
    tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizingOf::Flex(_)))
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct ReservedTrackSpace<S: LayoutScalar = Scalar> {
    pub(in crate::grid) count: usize,
    pub(in crate::grid) size: S,
}

pub(in crate::grid) fn reserved_track_space<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
) -> ReservedTrackSpace<S> {
    let mut count = 0;
    let mut size = S::ZERO;
    for component in components {
        match component {
            TrackComponentOf::Track(track) => {
                count += 1;
                size = size + track_base_size(track, basis, S::ZERO);
            }
            TrackComponentOf::Repeat(repetition) => {
                if let TrackRepeat::Count(repeat_count) = repetition.repeat() {
                    let repeated_tracks = repetition.sizing_tracks();
                    count += repeat_count.get() * repeated_tracks.len();
                    size = size
                        + S::from_usize(repeat_count.get())
                            * repeated_tracks
                                .iter()
                                .map(|track| track_base_size(track, basis, S::ZERO))
                                .fold(S::ZERO, |sum, value| sum + value);
                }
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }

    if count > 1 {
        size = size + gap * S::from_usize(count - 1);
    }

    ReservedTrackSpace { count, size }
}

pub(in crate::grid) fn auto_repeat_count<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    reserved: ReservedTrackSpace<S>,
) -> usize {
    let Some(basis) = basis else {
        return 1;
    };
    if tracks.is_empty() {
        return 0;
    }
    let track_sum = tracks
        .iter()
        .map(|track| track_base_size(track, Some(basis), S::ZERO).max(S::ONE))
        .fold(S::ZERO, |sum, value| sum + value);
    let repeat_size = track_sum + gap * S::from_usize(tracks.len());
    if repeat_size <= S::ZERO {
        1
    } else {
        let available = if reserved.count == 0 {
            basis + gap
        } else {
            basis - reserved.size
        };
        (available / repeat_size)
            .floor()
            .max(S::ONE)
            .floor_to_usize_saturating()
    }
}
