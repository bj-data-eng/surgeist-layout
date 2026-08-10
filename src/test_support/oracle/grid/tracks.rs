use super::contributions::ItemContributionFacts;
use super::placement::GridAxis;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Track {
    Px(f32),
    Percent(f32),
    Fr(f32),
}

impl Track {
    #[must_use]
    pub const fn px(size: f32) -> Self {
        Self::Px(size)
    }

    #[must_use]
    pub const fn percent(factor: f32) -> Self {
        Self::Percent(factor)
    }

    #[must_use]
    pub const fn fr(factor: f32) -> Self {
        Self::Fr(factor)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefiniteTracks {
    container: f32,
    gap: f32,
    tracks: Vec<Track>,
}

impl DefiniteTracks {
    #[must_use]
    pub const fn new(container: f32, gap: f32) -> Self {
        Self {
            container,
            gap,
            tracks: Vec::new(),
        }
    }

    #[must_use]
    pub fn track(mut self, track: Track) -> Self {
        self.tracks.push(track);
        self
    }

    #[must_use]
    pub fn solve(self) -> SolvedTracks {
        let gap_total = self.gap * self.tracks.len().saturating_sub(1) as f32;
        let fixed_sum = self
            .tracks
            .iter()
            .map(|track| match track {
                Track::Px(size) => *size,
                Track::Percent(factor) => self.container * factor,
                Track::Fr(_) => 0.0,
            })
            .sum::<f32>();
        let fr_sum = self
            .tracks
            .iter()
            .map(|track| match track {
                Track::Px(_) | Track::Percent(_) => 0.0,
                Track::Fr(factor) => *factor,
            })
            .sum::<f32>();
        let leftover = (self.container - gap_total - fixed_sum).max(0.0);
        let fr_unit = if fr_sum > 0.0 {
            leftover / fr_sum.max(1.0)
        } else {
            0.0
        };
        let sizes = self
            .tracks
            .iter()
            .map(|track| match track {
                Track::Px(size) => *size,
                Track::Percent(factor) => self.container * factor,
                Track::Fr(factor) => fr_unit * factor,
            })
            .collect::<Vec<_>>();

        SolvedTracks::new(sizes, self.gap)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SolvedTracks {
    sizes: Vec<f32>,
    offsets: Vec<f32>,
}

impl SolvedTracks {
    #[must_use]
    pub(crate) fn new(sizes: Vec<f32>, gap: f32) -> Self {
        let mut cursor = 0.0;
        let mut offsets = Vec::with_capacity(sizes.len());
        for size in &sizes {
            offsets.push(cursor);
            cursor += size + gap;
        }
        Self { sizes, offsets }
    }

    pub(crate) fn offsets_mut(&mut self) -> &mut [f32] {
        &mut self.offsets
    }

    #[must_use]
    pub fn sizes(&self) -> &[f32] {
        &self.sizes
    }

    #[must_use]
    pub fn size(&self, index: usize) -> f32 {
        self.sizes[index]
    }

    #[must_use]
    pub fn offset(&self, index: usize) -> f32 {
        self.offsets[index]
    }

    #[must_use]
    pub fn area(&self, start_line: usize, end_line: usize) -> TrackArea {
        assert!(start_line > 0, "grid lines are 1-based");
        assert!(end_line > start_line, "end line must be after start line");
        assert!(
            end_line <= self.sizes.len() + 1,
            "end line must fit inside the solved track axis"
        );

        let start_index = start_line - 1;
        let end_index = end_line - 2;
        let start = self.offset(start_index);
        let end = self.offset(end_index) + self.size(end_index);
        TrackArea {
            start,
            size: end - start,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackArea {
    pub start: f32,
    pub size: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackMin {
    Fixed(f32),
    Percent(f32),
    Auto,
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackMax {
    Fixed(f32),
    Percent(f32),
    Flex(f32),
    Auto,
    MaxContent,
    FitContent(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridTrack {
    pub min: TrackMin,
    pub max: TrackMax,
}

impl GridTrack {
    #[must_use]
    pub const fn new(min: TrackMin, max: TrackMax) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub const fn fixed(size: f32) -> Self {
        Self::new(TrackMin::Fixed(size), TrackMax::Fixed(size))
    }

    #[must_use]
    pub const fn percent(factor: f32) -> Self {
        Self::new(TrackMin::Percent(factor), TrackMax::Percent(factor))
    }

    #[must_use]
    pub const fn flex(factor: f32) -> Self {
        Self::new(TrackMin::Auto, TrackMax::Flex(factor))
    }

    #[must_use]
    pub const fn auto() -> Self {
        Self::new(TrackMin::Auto, TrackMax::Auto)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackSizingSlice {
    axis: GridAxis,
    available: Option<f32>,
    gap: f32,
    tracks: Vec<GridTrack>,
    items: Vec<ItemContributionFacts>,
    stretch_auto_tracks: bool,
}

impl TrackSizingSlice {
    #[must_use]
    pub const fn definite_columns(available: f32, gap: f32) -> Self {
        Self {
            axis: GridAxis::Column,
            available: Some(available),
            gap,
            tracks: Vec::new(),
            items: Vec::new(),
            stretch_auto_tracks: false,
        }
    }

    #[must_use]
    pub const fn definite_rows(available: f32, gap: f32) -> Self {
        Self {
            axis: GridAxis::Row,
            available: Some(available),
            gap,
            tracks: Vec::new(),
            items: Vec::new(),
            stretch_auto_tracks: false,
        }
    }

    #[must_use]
    pub const fn indefinite_columns(gap: f32) -> Self {
        Self {
            axis: GridAxis::Column,
            available: None,
            gap,
            tracks: Vec::new(),
            items: Vec::new(),
            stretch_auto_tracks: false,
        }
    }

    #[must_use]
    pub const fn indefinite_rows(gap: f32) -> Self {
        Self {
            axis: GridAxis::Row,
            available: None,
            gap,
            tracks: Vec::new(),
            items: Vec::new(),
            stretch_auto_tracks: false,
        }
    }

    #[must_use]
    pub fn track(mut self, track: GridTrack) -> Self {
        self.tracks.push(track);
        self
    }

    #[must_use]
    pub fn item(mut self, item: ItemContributionFacts) -> Self {
        self.items.push(item);
        self
    }

    #[must_use]
    pub const fn stretch_auto_tracks(mut self) -> Self {
        self.stretch_auto_tracks = true;
        self
    }

    #[must_use]
    pub fn solve(self) -> TrackSizingReport {
        self.try_solve()
            .expect("track sizing input must be supported by this oracle slice")
    }

    pub fn try_solve(self) -> Result<TrackSizingReport, TrackSizingError> {
        let initialized = TrackState {
            tracks: self
                .tracks
                .iter()
                .map(|track| initialize_track(*track, self.available))
                .collect(),
        };

        let after_intrinsic_minimums = grow_single_span_items(
            &initialized,
            &self.items,
            ContributionPhase::Minimum,
            self.axis,
        );
        let after_content_based_minimums = grow_single_span_items(
            &after_intrinsic_minimums,
            &self.items,
            ContributionPhase::ContentMinimum,
            self.axis,
        );
        let after_spanning_items = grow_spanning_items(
            &after_content_based_minimums,
            &self.tracks,
            &self.items,
            ContributionPhase::ContentMinimum,
            self.axis,
            self.gap,
        )?;
        let after_maximize_tracks = maximize_tracks(
            &after_spanning_items,
            &self.tracks,
            self.available,
            self.gap,
        );
        let (flex_fraction, after_flexing) = flex_tracks(
            &after_maximize_tracks,
            &self.tracks,
            self.available,
            self.gap,
        );
        let after_stretch = stretch_tracks(
            &after_flexing,
            &self.tracks,
            self.available,
            self.gap,
            self.stretch_auto_tracks,
        );
        let final_tracks = solved_tracks(&after_stretch, self.gap);

        Ok(TrackSizingReport {
            initialized,
            after_intrinsic_minimums,
            after_content_based_minimums,
            after_spanning_items,
            after_maximize_tracks,
            flex_fraction,
            after_flexing,
            after_stretch,
            final_tracks,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackState {
    pub tracks: Vec<TrackSize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSize {
    pub base: f32,
    pub growth_limit: GrowthLimit,
}

impl TrackSize {
    #[must_use]
    pub const fn new(base: f32, growth_limit: GrowthLimit) -> Self {
        Self { base, growth_limit }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GrowthLimit {
    Definite(f32),
    Infinite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackSizingReport {
    pub initialized: TrackState,
    pub after_intrinsic_minimums: TrackState,
    pub after_content_based_minimums: TrackState,
    pub after_spanning_items: TrackState,
    pub after_maximize_tracks: TrackState,
    pub flex_fraction: Option<f32>,
    pub after_flexing: TrackState,
    pub after_stretch: TrackState,
    pub final_tracks: Vec<SolvedTrack>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackSizingError {
    UnsupportedSpanningTrackMix {
        axis: GridAxis,
        start: usize,
        span: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolvedTrack {
    pub size: f32,
    pub offset: f32,
}

fn initialize_track(track: GridTrack, available: Option<f32>) -> TrackSize {
    let base = match track.min {
        TrackMin::Fixed(size) => size,
        TrackMin::Percent(factor) => available.map_or(0.0, |available| available * factor),
        TrackMin::Auto | TrackMin::MinContent | TrackMin::MaxContent => 0.0,
    };
    let growth_limit = match track.max {
        TrackMax::Fixed(size) => GrowthLimit::Definite(size.max(base)),
        TrackMax::Percent(factor) => available
            .map(|available| GrowthLimit::Definite((available * factor).max(base)))
            .unwrap_or(GrowthLimit::Infinite),
        TrackMax::FitContent(limit) => GrowthLimit::Definite(limit.max(base)),
        TrackMax::Flex(_) | TrackMax::Auto | TrackMax::MaxContent => GrowthLimit::Infinite,
    };

    TrackSize::new(base, growth_limit)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContributionPhase {
    Minimum,
    ContentMinimum,
}

fn grow_single_span_items(
    state: &TrackState,
    items: &[ItemContributionFacts],
    phase: ContributionPhase,
    axis: GridAxis,
) -> TrackState {
    let mut state = state.clone();
    for item in items {
        if item.area.span(axis) != 1 {
            continue;
        }
        let index = item.area.start(axis) - 1;
        assert!(index < state.tracks.len(), "single-span item must fit");
        let contribution = item_contribution(*item, phase);
        grow_base_to(&mut state.tracks[index], contribution);
    }
    state
}

fn grow_spanning_items(
    state: &TrackState,
    tracks: &[GridTrack],
    items: &[ItemContributionFacts],
    phase: ContributionPhase,
    axis: GridAxis,
    gap: f32,
) -> Result<TrackState, TrackSizingError> {
    let mut state = state.clone();
    for item in items {
        let span = item.area.span(axis);
        if span <= 1 {
            continue;
        }
        let start = item.area.start(axis) - 1;
        let end = start + span;
        assert!(end <= state.tracks.len(), "spanning item must fit");
        if !spanning_tracks_are_supported(&tracks[start..end]) {
            return Err(TrackSizingError::UnsupportedSpanningTrackMix {
                axis,
                start: start + 1,
                span,
            });
        }

        let contribution = item_contribution(*item, phase);
        distribute_deficit(&mut state.tracks[start..end], contribution, gap);
    }
    Ok(state)
}

fn spanning_tracks_are_supported(tracks: &[GridTrack]) -> bool {
    tracks
        .first()
        .is_none_or(|first| tracks.iter().all(|track| track == first))
}

fn item_contribution(item: ItemContributionFacts, phase: ContributionPhase) -> f32 {
    let contributions = item.contributions();
    match phase {
        ContributionPhase::Minimum => contributions.minimum,
        ContributionPhase::ContentMinimum => contributions.limited_min_content,
    }
}

fn distribute_deficit(tracks: &mut [TrackSize], contribution: f32, gap: f32) {
    let gap_total = gap * tracks.len().saturating_sub(1) as f32;
    let current = tracks.iter().map(|track| track.base).sum::<f32>() + gap_total;
    let mut deficit = (contribution - current).max(0.0);

    while deficit > 0.000_001 {
        let growable = tracks
            .iter()
            .filter(|track| growth_capacity(**track) > 0.000_001)
            .count();
        if growable == 0 {
            break;
        }

        let share = deficit / growable as f32;
        let mut distributed = 0.0;
        for track in tracks.iter_mut() {
            let capacity = growth_capacity(*track);
            if capacity <= 0.000_001 {
                continue;
            }
            let growth = capacity.min(share);
            track.base += growth;
            distributed += growth;
        }

        if distributed <= 0.000_001 {
            break;
        }
        deficit -= distributed;
    }
}

fn grow_base_to(track: &mut TrackSize, target: f32) {
    let target = target.max(track.base);
    track.base = match track.growth_limit {
        GrowthLimit::Definite(limit) => target.min(limit),
        GrowthLimit::Infinite => target,
    };
}

fn growth_capacity(track: TrackSize) -> f32 {
    match track.growth_limit {
        GrowthLimit::Definite(limit) => (limit - track.base).max(0.0),
        GrowthLimit::Infinite => f32::INFINITY,
    }
}

fn maximize_tracks(
    state: &TrackState,
    tracks: &[GridTrack],
    available: Option<f32>,
    gap: f32,
) -> TrackState {
    let Some(available) = available else {
        return state.clone();
    };
    let gap_total = gap * state.tracks.len().saturating_sub(1) as f32;
    let current = state.tracks.iter().map(|track| track.base).sum::<f32>() + gap_total;
    let mut free = (available - current).max(0.0);
    let mut state = state.clone();

    while free > 0.000_001 {
        let growable = state
            .tracks
            .iter()
            .zip(tracks)
            .filter(|(size, track)| maximize_capacity(**size, **track) > 0.000_001)
            .count();
        if growable == 0 {
            break;
        }

        let share = free / growable as f32;
        let mut distributed = 0.0;
        for (size, track) in state.tracks.iter_mut().zip(tracks) {
            let capacity = maximize_capacity(*size, *track);
            if capacity <= 0.000_001 {
                continue;
            }
            let growth = capacity.min(share);
            size.base += growth;
            distributed += growth;
        }

        if distributed <= 0.000_001 {
            break;
        }
        free -= distributed;
    }

    state
}

fn maximize_capacity(size: TrackSize, track: GridTrack) -> f32 {
    if matches!(track.max, TrackMax::Flex(_) | TrackMax::Auto) {
        return 0.0;
    }
    match size.growth_limit {
        GrowthLimit::Definite(limit) => (limit - size.base).max(0.0),
        GrowthLimit::Infinite => 0.0,
    }
}

fn flex_tracks(
    state: &TrackState,
    tracks: &[GridTrack],
    available: Option<f32>,
    gap: f32,
) -> (Option<f32>, TrackState) {
    let flex_indices = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| match track.max {
            TrackMax::Flex(factor) if factor > 0.0 => Some((index, factor)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let Some(available) = available.filter(|_| !flex_indices.is_empty()) else {
        return (None, state.clone());
    };

    let gap_total = gap * state.tracks.len().saturating_sub(1) as f32;
    let non_flex_base_sum = state
        .tracks
        .iter()
        .zip(tracks)
        .map(|(size, track)| match track.max {
            TrackMax::Flex(_) => 0.0,
            _ => size.base,
        })
        .sum::<f32>();
    let mut frozen = vec![false; state.tracks.len()];
    let mut flex_fraction = 0.0;

    loop {
        let frozen_base_sum = flex_indices
            .iter()
            .filter(|(index, _)| frozen[*index])
            .map(|(index, _)| state.tracks[*index].base)
            .sum::<f32>();
        let unfrozen_factor_sum = flex_indices
            .iter()
            .filter(|(index, _)| !frozen[*index])
            .map(|(_, factor)| *factor)
            .sum::<f32>();
        if unfrozen_factor_sum == 0.0 {
            break;
        }

        flex_fraction = ((available - gap_total - non_flex_base_sum - frozen_base_sum).max(0.0))
            / unfrozen_factor_sum.max(1.0);
        let mut froze_track = false;
        for (index, factor) in &flex_indices {
            if !frozen[*index] && state.tracks[*index].base > flex_fraction * *factor {
                frozen[*index] = true;
                froze_track = true;
            }
        }
        if !froze_track {
            break;
        }
    }

    let tracks = state
        .tracks
        .iter()
        .enumerate()
        .map(|(index, size)| match tracks[index].max {
            TrackMax::Flex(factor) if !frozen[index] => {
                TrackSize::new(size.base.max(flex_fraction * factor), size.growth_limit)
            }
            _ => *size,
        })
        .collect();

    (Some(flex_fraction), TrackState { tracks })
}

fn stretch_tracks(
    state: &TrackState,
    tracks: &[GridTrack],
    available: Option<f32>,
    gap: f32,
    enabled: bool,
) -> TrackState {
    let Some(available) = available.filter(|_| enabled) else {
        return state.clone();
    };
    let gap_total = gap * state.tracks.len().saturating_sub(1) as f32;
    let current = state.tracks.iter().map(|track| track.base).sum::<f32>() + gap_total;
    let free = (available - current).max(0.0);
    let stretchable = tracks
        .iter()
        .filter(|track| matches!(track.max, TrackMax::Auto))
        .count();
    if stretchable == 0 || free == 0.0 {
        return state.clone();
    }

    let share = free / stretchable as f32;
    let tracks = state
        .tracks
        .iter()
        .zip(tracks)
        .map(|(size, track)| {
            if matches!(track.max, TrackMax::Auto) {
                TrackSize::new(size.base + share, size.growth_limit)
            } else {
                *size
            }
        })
        .collect();

    TrackState { tracks }
}

fn solved_tracks(state: &TrackState, gap: f32) -> Vec<SolvedTrack> {
    let mut offset = 0.0;
    state
        .tracks
        .iter()
        .map(|track| {
            let solved = SolvedTrack {
                size: track.base,
                offset,
            };
            offset += track.base + gap;
            solved
        })
        .collect()
}

/// Homogeneous intrinsic track slice that shares each spanning deficit equally.
///
/// This does not model the full grid intrinsic sizing algorithm: no growth
/// limits, fit-content caps, percent reservation, or track-category priority.
#[derive(Clone, Debug, PartialEq)]
pub struct EqualShareIntrinsicTracks {
    bases: Vec<f32>,
}

impl EqualShareIntrinsicTracks {
    #[must_use]
    pub fn new(count: usize) -> Self {
        Self {
            bases: vec![0.0; count],
        }
    }

    #[must_use]
    pub fn base(mut self, index: usize, base: f32) -> Self {
        self.bases[index] = base;
        self
    }

    #[must_use]
    pub fn item(mut self, start: usize, span: usize, contribution: f32) -> Self {
        assert!(span > 0, "span must be positive");
        let end = start + span;
        assert!(end <= self.bases.len(), "spanning item must fit");

        let current = self.bases[start..end].iter().sum::<f32>();
        let deficit = (contribution - current).max(0.0);
        let share = deficit / span as f32;
        for base in &mut self.bases[start..end] {
            *base += share;
        }
        self
    }

    #[must_use]
    pub fn solve(self, gap: f32) -> SolvedTracks {
        SolvedTracks::new(self.bases, gap)
    }
}
