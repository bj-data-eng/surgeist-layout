use super::axis::opposite_axis;
use super::baseline::{
    BaselineAlignment, ContainerBaselineFallbackItem, ContainerBaselineReport, fallback_end_key,
    fallback_start_key,
};
use super::contributions::{ContributionSize, ItemContributionFacts};
use super::placement::{GridArea, GridAxis};
use super::subgrid::{OracleGridError, TrackSpan};
use super::tracks::{GridTrack, TrackMax, TrackMin, TrackSizingReport, TrackSizingSlice};
use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaneAutoFlow {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridLanesBaselineReason {
    WebKitMasonryFallback,
    NoItems,
    NoBaselineAlignmentRequested,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridLanesBaselineInput {
    pub auto_flow: LaneAutoFlow,
    pub queried_axis: GridAxis,
    pub requested_alignment: BaselineAlignment,
    pub has_items: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridLanesBaselinePolicyReport {
    pub applies_item_offsets: bool,
    pub reason: Option<GridLanesBaselineReason>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LaneFlowTolerance {
    Fixed(f32),
    Percent(f32),
    Infinite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementInput {
    pub grid_axis_tracks: usize,
    pub auto_flow: LaneAutoFlow,
    pub lane_gap: f32,
    pub tolerance: LaneFlowTolerance,
    pub tolerance_basis: f32,
    pub items: Vec<LaneItemInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemInput {
    pub id: &'static str,
    pub grid_axis_span: usize,
    pub definite_grid_axis_start: Option<usize>,
    pub lane_axis_margin_box: f32,
}

impl LaneItemInput {
    #[must_use]
    pub const fn definite(
        id: &'static str,
        grid_axis_start: usize,
        grid_axis_span: usize,
        lane_axis_margin_box: f32,
    ) -> Self {
        Self {
            id,
            grid_axis_span,
            definite_grid_axis_start: Some(grid_axis_start),
            lane_axis_margin_box,
        }
    }

    #[must_use]
    pub const fn auto(id: &'static str, grid_axis_span: usize, lane_axis_margin_box: f32) -> Self {
        Self {
            id,
            grid_axis_span,
            definite_grid_axis_start: None,
            lane_axis_margin_box,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemOffset {
    pub id: &'static str,
    pub grid_axis_start: usize,
    pub grid_axis_span: usize,
    pub offset: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementReport {
    pub lane_axis: GridAxis,
    pub grid_axis: GridAxis,
    pub item_offsets: Vec<LaneItemOffset>,
    pub content_size: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementTrace {
    pub report: LanePlacementReport,
    pub running_positions_after_each_item: Vec<Vec<f32>>,
    pub final_cursor: usize,
}

#[must_use]
pub const fn lane_axis(auto_flow: LaneAutoFlow) -> GridAxis {
    match auto_flow {
        LaneAutoFlow::Row => GridAxis::Row,
        LaneAutoFlow::Column => GridAxis::Column,
    }
}

#[must_use]
pub const fn grid_axis_for_lanes(auto_flow: LaneAutoFlow) -> GridAxis {
    opposite_axis(lane_axis(auto_flow))
}

#[must_use]
pub fn grid_lanes_baseline_policy(input: GridLanesBaselineInput) -> GridLanesBaselinePolicyReport {
    let reason = if !input.has_items {
        Some(GridLanesBaselineReason::NoItems)
    } else if input.requested_alignment == BaselineAlignment::None {
        Some(GridLanesBaselineReason::NoBaselineAlignmentRequested)
    } else {
        Some(GridLanesBaselineReason::WebKitMasonryFallback)
    };

    GridLanesBaselinePolicyReport {
        applies_item_offsets: false,
        reason,
    }
}

#[must_use]
pub fn grid_lanes_container_baselines(
    items: Vec<ContainerBaselineFallbackItem>,
) -> ContainerBaselineReport {
    let first = items
        .iter()
        .min_by_key(|item| fallback_start_key(item))
        .map(|item| item.block_offset + item.first_baseline);
    let last = items
        .iter()
        .max_by_key(|item| fallback_end_key(item))
        .map(|item| item.block_offset + item.last_baseline);

    ContainerBaselineReport { first, last }
}

pub fn place_lanes(input: LanePlacementInput) -> Result<LanePlacementReport, OracleGridError> {
    place_lanes_trace(input).map(|trace| trace.report)
}

pub fn place_lanes_trace(input: LanePlacementInput) -> Result<LanePlacementTrace, OracleGridError> {
    if input.grid_axis_tracks == 0 {
        return Err(OracleGridError::EmptyTrackList);
    }

    let mut running = vec![0.0; input.grid_axis_tracks];
    let mut item_offsets = Vec::new();
    let mut running_positions_after_each_item = Vec::new();
    let mut cursor = 0usize;
    let tolerance = resolve_tolerance(input.tolerance, input.tolerance_basis);
    let mut content_size: f32 = 0.0;

    for item in input.items {
        let (start_zero, span) = match item.definite_grid_axis_start {
            Some(start_line) => {
                if start_line == 0 || item.grid_axis_span == 0 {
                    return Err(OracleGridError::SpanOutOfRange);
                }
                let start_zero = start_line - 1;
                if start_zero + item.grid_axis_span > input.grid_axis_tracks {
                    return Err(OracleGridError::SpanOutOfRange);
                }
                (start_zero, item.grid_axis_span)
            }
            None => {
                let span = item.grid_axis_span.clamp(1, input.grid_axis_tracks);
                let start_zero = if matches!(input.tolerance, LaneFlowTolerance::Infinite) {
                    infinite_candidate_start(cursor, span, input.grid_axis_tracks)
                } else {
                    finite_candidate_start(&running, cursor, span, tolerance)
                };
                (start_zero, span)
            }
        };

        let previous = running[start_zero..start_zero + span]
            .iter()
            .copied()
            .fold(0.0, f32::max);
        let new_position = previous + item.lane_axis_margin_box + input.lane_gap;
        content_size = content_size.max(new_position - input.lane_gap);
        for position in &mut running[start_zero..start_zero + span] {
            *position = new_position;
        }

        item_offsets.push(LaneItemOffset {
            id: item.id,
            grid_axis_start: start_zero + 1,
            grid_axis_span: span,
            offset: previous,
        });
        running_positions_after_each_item.push(running.clone());
        cursor = (start_zero + span) % input.grid_axis_tracks;
    }

    Ok(LanePlacementTrace {
        report: LanePlacementReport {
            lane_axis: lane_axis(input.auto_flow),
            grid_axis: grid_axis_for_lanes(input.auto_flow),
            item_offsets,
            content_size,
        },
        running_positions_after_each_item,
        final_cursor: cursor,
    })
}

fn resolve_tolerance(tolerance: LaneFlowTolerance, basis: f32) -> f32 {
    match tolerance {
        LaneFlowTolerance::Fixed(value) => value,
        LaneFlowTolerance::Percent(factor) => factor * basis,
        LaneFlowTolerance::Infinite => f32::INFINITY,
    }
}

fn infinite_candidate_start(cursor: usize, span: usize, track_count: usize) -> usize {
    if cursor + span > track_count {
        0
    } else {
        cursor
    }
}

fn finite_candidate_start(running: &[f32], cursor: usize, span: usize, tolerance: f32) -> usize {
    let track_count = running.len();
    let max_start = track_count + 1 - span;
    let shifted_cursor = if cursor >= max_start { 0 } else { cursor };
    let absolute_shortest = (0..max_start)
        .map(|start| max_running_position(running, start, span))
        .fold(f32::INFINITY, f32::min);

    for offset in 0..max_start {
        let start = (shifted_cursor + offset) % max_start;
        if max_running_position(running, start, span) <= absolute_shortest + tolerance {
            return start;
        }
    }

    0
}

fn max_running_position(running: &[f32], start: usize, span: usize) -> f32 {
    running[start..start + span]
        .iter()
        .copied()
        .fold(0.0, f32::max)
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingInput {
    pub axis: GridAxis,
    pub available: Option<f32>,
    pub gap: f32,
    pub tracks: Vec<GridTrack>,
    pub content_sized_tracks: Vec<usize>,
    pub items: Vec<LaneIntrinsicItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicItem {
    id: &'static str,
    kind: LaneIntrinsicItemKind,
    contribution: ItemContributionFacts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaneTrackSpanLength(NonZeroUsize);

impl LaneTrackSpanLength {
    #[must_use]
    pub const fn new(span: usize) -> Option<Self> {
        match NonZeroUsize::new(span) {
            Some(span) => Some(Self(span)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaneIntrinsicItemKind {
    Definite { span: TrackSpan },
    Indefinite { span: LaneTrackSpanLength },
}

impl LaneIntrinsicItem {
    pub fn definite(
        id: &'static str,
        span: TrackSpan,
        contribution: ItemContributionFacts,
    ) -> Result<Self, OracleGridError> {
        if span.checked_len().is_err() {
            return Err(OracleGridError::SpanOutOfRange);
        }
        Ok(Self {
            id,
            kind: LaneIntrinsicItemKind::Definite { span },
            contribution,
        })
    }

    #[must_use]
    pub const fn indefinite(
        id: &'static str,
        span: LaneTrackSpanLength,
        contribution: ItemContributionFacts,
    ) -> Self {
        Self {
            id,
            kind: LaneIntrinsicItemKind::Indefinite { span },
            contribution,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &'static str {
        self.id
    }

    #[must_use]
    pub const fn kind(&self) -> LaneIntrinsicItemKind {
        self.kind
    }

    #[must_use]
    pub const fn contribution(&self) -> ItemContributionFacts {
        self.contribution
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefiniteLaneIntrinsicItem {
    pub id: &'static str,
    pub span: TrackSpan,
    pub contribution: ItemContributionFacts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndefiniteLaneContributionGroup {
    pub span: usize,
    pub max_min_content: f32,
    pub max_max_content: f32,
    pub max_min_size: f32,
    pub item_ids: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingReport {
    pub definite_items: Vec<DefiniteLaneIntrinsicItem>,
    pub indefinite_groups: Vec<IndefiniteLaneContributionGroup>,
    pub converted_indefinite_items: Vec<DefiniteLaneIntrinsicItem>,
    pub final_track_report: TrackSizingReport,
}

pub fn lane_intrinsic_sizing(
    input: LaneIntrinsicSizingInput,
) -> Result<LaneIntrinsicSizingReport, OracleGridError> {
    if input.content_sized_tracks.is_empty() || input.tracks.is_empty() {
        return Err(OracleGridError::EmptyTrackList);
    }
    if input
        .content_sized_tracks
        .iter()
        .any(|track_index| *track_index >= input.tracks.len())
    {
        return Err(OracleGridError::SpanOutOfRange);
    }

    let mut definite_items = Vec::new();
    let mut indefinite_groups: Vec<IndefiniteLaneContributionGroup> = Vec::new();

    for item in &input.items {
        match item.kind() {
            LaneIntrinsicItemKind::Definite { span } => {
                span.checked_len()?;
                if span.end > input.tracks.len() + 1 {
                    return Err(OracleGridError::SpanOutOfRange);
                }
                let contribution =
                    contribution_with_span_area(input.axis, span, item.contribution());
                definite_items.push(DefiniteLaneIntrinsicItem {
                    id: item.id(),
                    span,
                    contribution,
                });
            }
            LaneIntrinsicItemKind::Indefinite { span } => {
                let span = span.get().min(input.tracks.len());
                let contributions = item.contribution().contributions();
                if let Some(group) = indefinite_groups
                    .iter_mut()
                    .find(|group| group.span == span)
                {
                    group.max_min_content = group.max_min_content.max(contributions.min_content);
                    group.max_max_content = group.max_max_content.max(contributions.max_content);
                    group.max_min_size = group.max_min_size.max(contributions.minimum);
                    group.item_ids.push(item.id());
                } else {
                    indefinite_groups.push(IndefiniteLaneContributionGroup {
                        span,
                        max_min_content: contributions.min_content,
                        max_max_content: contributions.max_content,
                        max_min_size: contributions.minimum,
                        item_ids: vec![item.id()],
                    });
                }
            }
        }
    }

    let mut converted_indefinite_items = Vec::new();
    let mut masonry_sizing_items = Vec::new();
    for group in &indefinite_groups {
        for start_index in candidate_starts(input.tracks.len(), group.span) {
            let span = TrackSpan::new(start_index + 1, start_index + 1 + group.span);
            let contribution = contribution_with_span_area(
                input.axis,
                span,
                ItemContributionFacts {
                    area: GridArea::new(1, 1, 1, 1),
                    min_content: group.max_min_content,
                    max_content: group.max_max_content,
                    preferred: ContributionSize::Auto,
                    min_size: ContributionSize::Definite(group.max_min_size),
                    max_size: ContributionSize::Infinite,
                    margin_before: 0.0,
                    margin_after: 0.0,
                    automatic_minimum_applies: false,
                },
            );
            converted_indefinite_items.push(DefiniteLaneIntrinsicItem {
                id: "indefinite-group",
                span,
                contribution,
            });

            let content_spans = content_track_spans_in_span(
                &input.content_sized_tracks,
                start_index,
                group.span,
                input.tracks.len(),
            );
            let content_track_count = content_spans
                .iter()
                .map(|span| span.checked_len().expect("span already validated"))
                .sum::<usize>();
            for content_span in content_spans {
                masonry_sizing_items.push(masonry_sizing_contribution(
                    MasonrySizingProjection {
                        axis: input.axis,
                        full_span: span,
                        content_span,
                        tracks: &input.tracks,
                        available: input.available,
                        gap: input.gap,
                        content_track_count,
                    },
                    group,
                ));
            }
        }
    }

    let mut track_slice = match (input.axis, input.available) {
        (GridAxis::Column, Some(available)) => {
            TrackSizingSlice::definite_columns(available, input.gap)
        }
        (GridAxis::Row, Some(available)) => TrackSizingSlice::definite_rows(available, input.gap),
        (GridAxis::Column, None) => TrackSizingSlice::indefinite_columns(input.gap),
        (GridAxis::Row, None) => TrackSizingSlice::indefinite_rows(input.gap),
    };
    for track in input.tracks {
        track_slice = track_slice.track(track);
    }
    for item in definite_items
        .iter()
        .filter(|item| span_overlaps_content_tracks(item.span, &input.content_sized_tracks))
    {
        track_slice = track_slice.item(item.contribution);
    }
    for item in masonry_sizing_items {
        track_slice = track_slice.item(item);
    }
    let final_track_report = track_slice
        .try_solve()
        .map_err(|_| OracleGridError::SpanOutOfRange)?;

    Ok(LaneIntrinsicSizingReport {
        definite_items,
        indefinite_groups,
        converted_indefinite_items,
        final_track_report,
    })
}

fn candidate_starts(track_count: usize, span: usize) -> Vec<usize> {
    let span = span.max(1).min(track_count);
    (0..=track_count - span).collect()
}

fn content_track_spans_in_span(
    content_sized_tracks: &[usize],
    start_index: usize,
    span: usize,
    track_count: usize,
) -> Vec<TrackSpan> {
    let end_index = (start_index + span.max(1)).min(track_count);
    let mut tracks = content_sized_tracks
        .iter()
        .copied()
        .filter(|track_index| (start_index..end_index).contains(track_index))
        .collect::<Vec<_>>();
    tracks.sort_unstable();
    tracks.dedup();

    let mut spans = Vec::new();
    let mut cursor = 0;
    while cursor < tracks.len() {
        let start = tracks[cursor];
        let mut end = start + 1;
        cursor += 1;
        while cursor < tracks.len() && tracks[cursor] == end {
            end += 1;
            cursor += 1;
        }
        spans.push(TrackSpan::new(start + 1, end + 1));
    }
    spans
}

struct MasonrySizingProjection<'a> {
    axis: GridAxis,
    full_span: TrackSpan,
    content_span: TrackSpan,
    tracks: &'a [GridTrack],
    available: Option<f32>,
    gap: f32,
    content_track_count: usize,
}

fn masonry_sizing_contribution(
    projection: MasonrySizingProjection<'_>,
    group: &IndefiniteLaneContributionGroup,
) -> ItemContributionFacts {
    let MasonrySizingProjection {
        axis,
        full_span,
        content_span: span,
        tracks,
        available,
        gap,
        content_track_count,
    } = projection;
    let start_index = span.start - 1;
    let end_index = span.end - 1;
    let full_start_index = full_span.start - 1;
    let full_end_index = full_span.end - 1;
    let full_target = tracks[full_start_index..full_end_index]
        .iter()
        .map(|track| masonry_track_minimum_size(*track, group))
        .fold(0.0, f32::max);
    let full_existing = tracks[full_start_index..full_end_index]
        .iter()
        .map(|track| initialized_track_base(*track, available))
        .sum::<f32>()
        + gap
            * full_span
                .checked_len()
                .expect("span already validated")
                .saturating_sub(1) as f32;
    let content_existing = tracks[start_index..end_index]
        .iter()
        .map(|track| initialized_track_base(*track, available))
        .sum::<f32>()
        + gap
            * span
                .checked_len()
                .expect("span already validated")
                .saturating_sub(1) as f32;
    let content_span_len = span.checked_len().expect("span already validated");
    let deficit_share = (full_target - full_existing).max(0.0) * content_span_len as f32
        / content_track_count.max(1) as f32;
    let size = content_existing + deficit_share;
    let max_content = tracks[start_index..end_index]
        .iter()
        .map(|track| masonry_track_maximum_size(*track, size, group))
        .fold(0.0, f32::max);

    contribution_with_span_area(
        axis,
        span,
        ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: size,
            max_content,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Definite(size),
            max_size: ContributionSize::Infinite,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: false,
        },
    )
}

fn masonry_track_minimum_size(track: GridTrack, group: &IndefiniteLaneContributionGroup) -> f32 {
    match track.min {
        TrackMin::MinContent => group.max_min_content,
        TrackMin::MaxContent => group.max_max_content,
        TrackMin::Auto | TrackMin::Fixed(_) | TrackMin::Percent(_) => group.max_min_size,
    }
}

fn masonry_track_maximum_size(
    track: GridTrack,
    minimum_size: f32,
    group: &IndefiniteLaneContributionGroup,
) -> f32 {
    match track.max {
        TrackMax::MaxContent | TrackMax::Auto | TrackMax::FitContent(_) => group.max_max_content,
        TrackMax::Fixed(_) | TrackMax::Percent(_) | TrackMax::Flex(_) => minimum_size,
    }
}

fn initialized_track_base(track: GridTrack, available: Option<f32>) -> f32 {
    match track.min {
        TrackMin::Fixed(size) => size,
        TrackMin::Percent(factor) => available.map_or(0.0, |available| available * factor),
        TrackMin::Auto | TrackMin::MinContent | TrackMin::MaxContent => 0.0,
    }
}

fn span_overlaps_content_tracks(span: TrackSpan, content_sized_tracks: &[usize]) -> bool {
    let start = span.start - 1;
    let end = span.end - 1;
    content_sized_tracks
        .iter()
        .any(|track_index| (start..end).contains(track_index))
}

fn contribution_with_span_area(
    axis: GridAxis,
    span: TrackSpan,
    mut contribution: ItemContributionFacts,
) -> ItemContributionFacts {
    let span_len = span.checked_len().expect("span already validated");
    contribution.area = match axis {
        GridAxis::Column => GridArea::new(span.start, 1, span_len, 1),
        GridAxis::Row => GridArea::new(1, span.start, 1, span_len),
    };
    contribution
}
