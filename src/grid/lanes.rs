use super::*;
use crate::scroll::scrollbar_size_from_overflow;
use crate::{
    GridFlowToleranceOf, LengthResolutionOf, LengthResolutionStatus, MaxTrackSizingOf,
    MinTrackSizingOf, PercentageBasisOf,
};
use std::num::NonZeroUsize;

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementInputOf<Item, S: LayoutScalar = DefaultScalar> {
    pub grid_axis_tracks: usize,
    pub auto_flow: GridAutoFlow,
    pub lane_gap: S,
    pub tolerance: GridFlowToleranceOf<S>,
    pub tolerance_basis: S,
    pub items: Vec<LaneItemOf<Item, S>>,
}

pub type LanePlacementInput<Item> = LanePlacementInputOf<Item, DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemOf<Item, S: LayoutScalar = DefaultScalar> {
    pub item: Item,
    pub grid_axis_span: usize,
    pub definite_grid_axis_start: Option<usize>,
    pub lane_axis_margin_box: S,
}

pub type LaneItem<Item> = LaneItemOf<Item, DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemOffsetOf<Item, S: LayoutScalar = DefaultScalar> {
    pub item: Item,
    pub grid_axis_start: usize,
    pub grid_axis_span: usize,
    pub offset: S,
    pub lane_axis_margin_box: S,
}

pub type LaneItemOffset<Item> = LaneItemOffsetOf<Item, DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementReportOf<Item, S: LayoutScalar = DefaultScalar> {
    pub lane_axis: GridAxisKind,
    pub grid_axis: GridAxisKind,
    pub item_offsets: Vec<LaneItemOffsetOf<Item, S>>,
    pub content_size: S,
}

pub type LanePlacementReport<Item> = LanePlacementReportOf<Item, DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct LanePlacementTraceOf<Item, S: LayoutScalar = DefaultScalar> {
    pub(super) report: LanePlacementReportOf<Item, S>,
    pub(super) running_positions_after_each_item: Vec<Vec<S>>,
    pub(super) final_cursor: usize,
}

impl<Item, S: LayoutScalar> LanePlacementTraceOf<Item, S> {
    fn into_report(self) -> LanePlacementReportOf<Item, S> {
        self.report
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanePlacementError {
    EmptyTrackList,
    InvalidGridAxisStart {
        start: usize,
    },
    InvalidGridAxisSpan {
        span: usize,
    },
    GridAxisSpanOutOfRange {
        start: usize,
        span: usize,
        tracks: usize,
    },
    ContentSizedTrackOutOfRange {
        track_index: usize,
        tracks: usize,
    },
    InvalidDefiniteLaneSpan {
        span: LaneTrackSpan,
    },
    DefiniteLaneSpanOutOfRange {
        span: LaneTrackSpan,
        tracks: usize,
    },
    NestedGridLanesSubgridIndefiniteUnsupported,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaneContributionFactsOf<S: LayoutScalar = DefaultScalar> {
    pub min_content: S,
    pub max_content: S,
    pub min_size: S,
    pub automatic_minimum_applies: bool,
}

pub type LaneContributionFacts = LaneContributionFactsOf<DefaultScalar>;

impl<S: LayoutScalar> LaneContributionFactsOf<S> {
    fn contributions(self) -> LaneContributionsOf<S> {
        let minimum = if self.automatic_minimum_applies {
            self.min_content
        } else {
            self.min_size
        };
        LaneContributionsOf {
            minimum,
            min_content: self.min_content,
            max_content: self.max_content,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LaneContributionsOf<S: LayoutScalar = DefaultScalar> {
    minimum: S,
    min_content: S,
    max_content: S,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaneTrackSpan {
    pub start: usize,
    pub end: usize,
}

impl LaneTrackSpan {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn len(self) -> Option<usize> {
        if self.start == 0 || self.end <= self.start {
            None
        } else {
            Some(self.end - self.start)
        }
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingInputOf<S: LayoutScalar = DefaultScalar> {
    pub axis: GridAxisKind,
    pub available: Option<S>,
    pub gap: S,
    pub tracks: Vec<TrackSizingOf<S>>,
    pub content_sized_tracks: Vec<usize>,
    pub items: Vec<LaneIntrinsicItemOf<S>>,
}

pub type LaneIntrinsicSizingInput = LaneIntrinsicSizingInputOf<DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicItemOf<S: LayoutScalar = DefaultScalar> {
    id: &'static str,
    kind: LaneIntrinsicItemKind,
    contribution: LaneContributionFactsOf<S>,
}

pub type LaneIntrinsicItem = LaneIntrinsicItemOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaneIntrinsicItemKind {
    Definite { span: LaneTrackSpan },
    Indefinite { span: LaneTrackSpanLength },
    NestedIndefiniteSubgrid { span: LaneTrackSpanLength },
}

impl<S: LayoutScalar> LaneIntrinsicItemOf<S> {
    pub fn definite(
        id: &'static str,
        span: LaneTrackSpan,
        contribution: LaneContributionFactsOf<S>,
    ) -> Result<Self, LanePlacementError> {
        if span.len().is_none() {
            return Err(LanePlacementError::InvalidDefiniteLaneSpan { span });
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
        contribution: LaneContributionFactsOf<S>,
    ) -> Self {
        Self {
            id,
            kind: LaneIntrinsicItemKind::Indefinite { span },
            contribution,
        }
    }

    #[must_use]
    pub const fn nested_indefinite_subgrid(
        id: &'static str,
        span: LaneTrackSpanLength,
        contribution: LaneContributionFactsOf<S>,
    ) -> Self {
        Self {
            id,
            kind: LaneIntrinsicItemKind::NestedIndefiniteSubgrid { span },
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
    pub const fn contribution(&self) -> LaneContributionFactsOf<S> {
        self.contribution
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DefiniteLaneIntrinsicItemOf<S: LayoutScalar = DefaultScalar> {
    pub id: &'static str,
    pub span: LaneTrackSpan,
    pub contribution: LaneContributionFactsOf<S>,
}

pub type DefiniteLaneIntrinsicItem = DefiniteLaneIntrinsicItemOf<DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct IndefiniteLaneContributionGroupOf<S: LayoutScalar = DefaultScalar> {
    pub span: usize,
    pub max_min_content: S,
    pub max_max_content: S,
    pub max_min_size: S,
    pub item_ids: Vec<&'static str>,
}

pub type IndefiniteLaneContributionGroup = IndefiniteLaneContributionGroupOf<DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingReportOf<S: LayoutScalar = DefaultScalar> {
    pub definite_items: Vec<DefiniteLaneIntrinsicItemOf<S>>,
    pub indefinite_groups: Vec<IndefiniteLaneContributionGroupOf<S>>,
    pub converted_indefinite_items: Vec<DefiniteLaneIntrinsicItemOf<S>>,
    pub final_track_sizes: Vec<S>,
}

pub type LaneIntrinsicSizingReport = LaneIntrinsicSizingReportOf<DefaultScalar>;

#[must_use]
pub const fn lane_axis(auto_flow: GridAutoFlow) -> GridAxisKind {
    if auto_flow.is_column() {
        GridAxisKind::Column
    } else {
        GridAxisKind::Row
    }
}

#[must_use]
pub const fn grid_axis_for_lanes(auto_flow: GridAutoFlow) -> GridAxisKind {
    match lane_axis(auto_flow) {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    }
}

pub(super) fn lane_axis_for_grid_lanes<S: LayoutScalar>(style: &NodeInputOf<S>) -> GridAxisKind {
    let has_columns = !style.grid_template_columns.is_empty();
    let has_rows = !style.grid_template_rows.is_empty();
    match (has_columns, has_rows) {
        (false, true) => GridAxisKind::Column,
        (true, false) => GridAxisKind::Row,
        _ => lane_axis(style.grid_auto_flow),
    }
}

pub(super) fn grid_axis_for_grid_lanes<S: LayoutScalar>(style: &NodeInputOf<S>) -> GridAxisKind {
    match lane_axis_for_grid_lanes(style) {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    }
}

pub(super) fn column_flow_for_grid_lanes<S: LayoutScalar>(style: &NodeInputOf<S>) -> bool {
    grid_axis_for_grid_lanes(style) == GridAxisKind::Row
}

pub fn place_lanes<Item, S: LayoutScalar>(
    input: LanePlacementInputOf<Item, S>,
) -> Result<LanePlacementReportOf<Item, S>, LanePlacementError> {
    place_lanes_with_trace(input).map(LanePlacementTraceOf::into_report)
}

fn place_lanes_with_trace<Item, S: LayoutScalar>(
    input: LanePlacementInputOf<Item, S>,
) -> Result<LanePlacementTraceOf<Item, S>, LanePlacementError> {
    if input.grid_axis_tracks == 0 {
        return Err(LanePlacementError::EmptyTrackList);
    }

    let mut running = vec![S::ZERO; input.grid_axis_tracks];
    let mut item_offsets = Vec::new();
    let mut running_positions_after_each_item = Vec::new();
    let mut cursor = 0usize;
    let tolerance = resolve_tolerance(input.tolerance, input.tolerance_basis);
    let mut content_size = S::ZERO;

    for item in input.items {
        let (start_zero, span) = match item.definite_grid_axis_start {
            Some(start_line) => {
                if start_line == 0 {
                    return Err(LanePlacementError::InvalidGridAxisStart { start: start_line });
                }
                if item.grid_axis_span == 0 {
                    return Err(LanePlacementError::InvalidGridAxisSpan {
                        span: item.grid_axis_span,
                    });
                }
                let start_zero = start_line - 1;
                if start_zero + item.grid_axis_span > input.grid_axis_tracks {
                    return Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: start_line,
                        span: item.grid_axis_span,
                        tracks: input.grid_axis_tracks,
                    });
                }
                (start_zero, item.grid_axis_span)
            }
            None => {
                let span = item.grid_axis_span.clamp(1, input.grid_axis_tracks);
                let start_zero = if matches!(input.tolerance, GridFlowToleranceOf::Infinite) {
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
            .fold(S::ZERO, S::max);
        let new_position = previous + item.lane_axis_margin_box + input.lane_gap;
        content_size = content_size.max(new_position - input.lane_gap);
        for position in &mut running[start_zero..start_zero + span] {
            *position = new_position;
        }

        item_offsets.push(LaneItemOffsetOf {
            item: item.item,
            grid_axis_start: start_zero + 1,
            grid_axis_span: span,
            offset: previous,
            lane_axis_margin_box: item.lane_axis_margin_box,
        });
        running_positions_after_each_item.push(running.clone());
        cursor = (start_zero + span) % input.grid_axis_tracks;
    }

    Ok(LanePlacementTraceOf {
        report: LanePlacementReportOf {
            lane_axis: lane_axis(input.auto_flow),
            grid_axis: grid_axis_for_lanes(input.auto_flow),
            item_offsets,
            content_size,
        },
        running_positions_after_each_item,
        final_cursor: cursor,
    })
}

pub fn lane_intrinsic_sizing<S: LayoutScalar>(
    input: LaneIntrinsicSizingInputOf<S>,
) -> Result<LaneIntrinsicSizingReportOf<S>, LanePlacementError> {
    lane_intrinsic_sizing_with(input)
}

pub(super) fn lane_intrinsic_sizing_with<S: LayoutScalar>(
    input: LaneIntrinsicSizingInputOf<S>,
) -> Result<LaneIntrinsicSizingReportOf<S>, LanePlacementError> {
    if input.content_sized_tracks.is_empty() || input.tracks.is_empty() {
        return Err(LanePlacementError::EmptyTrackList);
    }
    if let Some(track_index) = input
        .content_sized_tracks
        .iter()
        .copied()
        .find(|track_index| *track_index >= input.tracks.len())
    {
        return Err(LanePlacementError::ContentSizedTrackOutOfRange {
            track_index,
            tracks: input.tracks.len(),
        });
    }

    let mut definite_items = Vec::new();
    let mut indefinite_groups: Vec<IndefiniteLaneContributionGroupOf<S>> = Vec::new();

    for item in &input.items {
        match item.kind() {
            LaneIntrinsicItemKind::Definite { span } => {
                if span.len().is_none() || span.end > input.tracks.len() + 1 {
                    return Err(LanePlacementError::DefiniteLaneSpanOutOfRange {
                        span,
                        tracks: input.tracks.len(),
                    });
                }
                definite_items.push(DefiniteLaneIntrinsicItemOf {
                    id: item.id(),
                    span,
                    contribution: item.contribution(),
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
                    indefinite_groups.push(IndefiniteLaneContributionGroupOf {
                        span,
                        max_min_content: contributions.min_content,
                        max_max_content: contributions.max_content,
                        max_min_size: contributions.minimum,
                        item_ids: vec![item.id()],
                    });
                }
            }
            LaneIntrinsicItemKind::NestedIndefiniteSubgrid { .. } => {
                return Err(LanePlacementError::NestedGridLanesSubgridIndefiniteUnsupported);
            }
        }
    }

    let mut converted_indefinite_items = Vec::new();
    let mut sizing_items = Vec::new();
    for group in &indefinite_groups {
        for start_index in candidate_starts(input.tracks.len(), group.span) {
            let span = LaneTrackSpan::new(start_index + 1, start_index + 1 + group.span);
            let contribution = LaneContributionFactsOf {
                min_content: group.max_min_content,
                max_content: group.max_max_content,
                min_size: group.max_min_size,
                automatic_minimum_applies: false,
            };
            converted_indefinite_items.push(DefiniteLaneIntrinsicItemOf {
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
                .map(|span| span.len().expect("span already validated"))
                .sum::<usize>();
            for content_span in content_spans {
                sizing_items.push(masonry_sizing_contribution(
                    MasonrySizingProjection {
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

    let mut final_track_sizes = input
        .tracks
        .iter()
        .map(|track| initialized_track_base(*track, input.available))
        .collect::<Vec<_>>();
    for item in definite_items
        .iter()
        .filter(|item| span_overlaps_content_tracks(item.span, &input.content_sized_tracks))
    {
        apply_lane_sizing_contribution(
            &mut final_track_sizes,
            &input.tracks,
            input.gap,
            input.available,
            *item,
        );
    }
    for item in &sizing_items {
        apply_lane_sizing_contribution(
            &mut final_track_sizes,
            &input.tracks,
            input.gap,
            input.available,
            *item,
        );
    }

    Ok(LaneIntrinsicSizingReportOf {
        definite_items,
        indefinite_groups,
        converted_indefinite_items,
        final_track_sizes,
    })
}

fn candidate_starts(track_count: usize, span: usize) -> impl Iterator<Item = usize> {
    let span = span.max(1).min(track_count);
    0..=track_count - span
}

fn content_track_spans_in_span(
    content_sized_tracks: &[usize],
    start_index: usize,
    span: usize,
    track_count: usize,
) -> Vec<LaneTrackSpan> {
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
        spans.push(LaneTrackSpan::new(start + 1, end + 1));
    }
    spans
}

#[derive(Clone, Copy)]
struct MasonrySizingProjection<'a, S: LayoutScalar = Scalar> {
    full_span: LaneTrackSpan,
    content_span: LaneTrackSpan,
    tracks: &'a [TrackSizingOf<S>],
    available: Option<S>,
    gap: S,
    content_track_count: usize,
}

fn masonry_sizing_contribution<S: LayoutScalar>(
    projection: MasonrySizingProjection<'_, S>,
    group: &IndefiniteLaneContributionGroupOf<S>,
) -> DefiniteLaneIntrinsicItemOf<S> {
    let MasonrySizingProjection {
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
        .fold(S::ZERO, S::max);
    let full_existing = tracks[full_start_index..full_end_index]
        .iter()
        .map(|track| initialized_track_base(*track, available))
        .fold(S::ZERO, |sum, size| sum + size)
        + gap
            * S::from_usize(
                full_span
                    .len()
                    .expect("span already validated")
                    .saturating_sub(1),
            );
    let content_existing = tracks[start_index..end_index]
        .iter()
        .map(|track| initialized_track_base(*track, available))
        .fold(S::ZERO, |sum, size| sum + size)
        + gap
            * S::from_usize(
                span.len()
                    .expect("span already validated")
                    .saturating_sub(1),
            );
    let content_span_len = span.len().expect("span already validated");
    let deficit_share = (full_target - full_existing).max(S::ZERO)
        * S::from_usize(content_span_len)
        / S::from_usize(content_track_count.max(1));
    let size = content_existing + deficit_share;
    let max_content = tracks[start_index..end_index]
        .iter()
        .map(|track| masonry_track_maximum_size(*track, size, group))
        .fold(S::ZERO, S::max);

    DefiniteLaneIntrinsicItemOf {
        id: "indefinite-group",
        span,
        contribution: LaneContributionFactsOf {
            min_content: size,
            max_content,
            min_size: size,
            automatic_minimum_applies: false,
        },
    }
}

fn masonry_track_minimum_size<S: LayoutScalar>(
    track: TrackSizingOf<S>,
    group: &IndefiniteLaneContributionGroupOf<S>,
) -> S {
    match track.min {
        MinTrackSizingOf::MinContent => group.max_min_content,
        MinTrackSizingOf::MaxContent => group.max_max_content,
        MinTrackSizingOf::Auto | MinTrackSizingOf::Length(_) => group.max_min_size,
    }
}

fn masonry_track_maximum_size<S: LayoutScalar>(
    track: TrackSizingOf<S>,
    minimum_size: S,
    group: &IndefiniteLaneContributionGroupOf<S>,
) -> S {
    match track.max {
        MaxTrackSizingOf::MinContent => group.max_min_content,
        MaxTrackSizingOf::MaxContent | MaxTrackSizingOf::Auto | MaxTrackSizingOf::FitContent(_) => {
            group.max_max_content
        }
        MaxTrackSizingOf::Length(_) | MaxTrackSizingOf::Flex(_) => minimum_size,
    }
}

fn initialized_track_base<S: LayoutScalar>(track: TrackSizingOf<S>, available: Option<S>) -> S {
    match track.min {
        MinTrackSizingOf::Length(length) => {
            resolution_or_zero(length.resolve_with_status(available))
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => {
            S::ZERO
        }
    }
}

fn resolution_or_zero<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> S {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution
            .value
            .expect("resolved length resolution must carry a value"),
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric
        | LengthResolutionStatus::NonNumeric => S::ZERO,
    }
}

fn span_overlaps_content_tracks(span: LaneTrackSpan, content_sized_tracks: &[usize]) -> bool {
    let start = span.start - 1;
    let end = span.end - 1;
    content_sized_tracks
        .iter()
        .any(|track_index| (start..end).contains(track_index))
}

fn apply_lane_sizing_contribution<S: LayoutScalar>(
    sizes: &mut [S],
    tracks: &[TrackSizingOf<S>],
    gap: S,
    available: Option<S>,
    item: DefiniteLaneIntrinsicItemOf<S>,
) {
    let start = item.span.start - 1;
    let end = item.span.end - 1;
    let Some(span_tracks) = tracks.get(start..end) else {
        return;
    };
    if span_tracks.is_empty() || end > sizes.len() {
        return;
    }
    let contribution = item.contribution.contributions();
    if end == start + 1 {
        sizes[start] = sizes[start].max(lane_track_minimum_size(
            span_tracks[0],
            contribution,
            available,
        ));
        return;
    }

    let target = span_contribution(contribution.minimum, end - start, gap);
    let current = sizes[start..end]
        .iter()
        .copied()
        .fold(S::ZERO, |sum, size| sum + size)
        + span_tracks
            .iter()
            .map(|track| {
                if track_accepts_intrinsic_contribution(*track) {
                    S::ZERO
                } else {
                    initialized_track_base(*track, available)
                }
            })
            .fold(S::ZERO, |sum, size| sum + size);
    let extra = (target - current).max(S::ZERO);
    if extra == S::ZERO {
        return;
    }
    let share = extra / S::from_usize(end - start);
    for size in &mut sizes[start..end] {
        *size = *size + share;
    }
}

fn lane_track_minimum_size<S: LayoutScalar>(
    track: TrackSizingOf<S>,
    contribution: LaneContributionsOf<S>,
    available: Option<S>,
) -> S {
    match track.min {
        MinTrackSizingOf::MinContent => contribution.min_content,
        MinTrackSizingOf::MaxContent => contribution.max_content,
        MinTrackSizingOf::Auto => contribution.minimum,
        MinTrackSizingOf::Length(_) => initialized_track_base(track, available),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "lane placement resolution keeps explicit grid layout phase inputs separate"
)]
pub(super) fn resolve_grid_lanes_placement_with_resolved_tracks<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    context: GridContainerContext<Tree::Scalar>,
    columns: &[Tree::Scalar],
    rows: &[Tree::Scalar],
    placements: &GridPlacementContext<<Tree as Traverse>::Node>,
    grid_axis_gap: Tree::Scalar,
) -> Result<LanePlacementReportOf<<Tree as Traverse>::Node, Tree::Scalar>, LanePlacementError>
where
    Tree: Compute,
{
    let grid_axis = grid_axis_for_grid_lanes(style);
    let lane_axis = lane_axis_for_grid_lanes(style);
    let grid_axis_tracks = match grid_axis {
        GridAxisKind::Column => columns,
        GridAxisKind::Row => rows,
    };
    if grid_axis_tracks.is_empty() {
        return Err(LanePlacementError::EmptyTrackList);
    }

    let tolerance = resolve_tolerance(
        style.grid_flow_tolerance,
        match grid_axis {
            GridAxisKind::Column => context.column_basis.unwrap_or(Tree::Scalar::ZERO),
            GridAxisKind::Row => context.row_basis.unwrap_or(Tree::Scalar::ZERO),
        },
    );
    let lane_gap = match lane_axis {
        GridAxisKind::Column => context.gap.width,
        GridAxisKind::Row => context.gap.height,
    };
    let mut running = vec![Tree::Scalar::ZERO; grid_axis_tracks.len()];
    let mut item_offsets = Vec::new();
    let mut running_positions_after_each_item = Vec::new();
    let mut cursor = 0usize;
    let mut content_size = Tree::Scalar::ZERO;

    let children = tree.children(node).collect::<Vec<_>>();
    for (child, placement) in placements.checked_child_placements(&children) {
        let child_style = tree.node_input(child).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }
        let placement = match grid_axis {
            GridAxisKind::Column => placement.column,
            GridAxisKind::Row => placement.row,
        };
        let (definite_grid_axis_start, grid_axis_span) = lane_grid_axis_facts(
            placement,
            grid_axis_tracks.len(),
            grid_axis_lines(context.lines, grid_axis),
        );
        let (start, span) = match definite_grid_axis_start {
            Some(start_line) => {
                if start_line == 0 {
                    return Err(LanePlacementError::InvalidGridAxisStart { start: start_line });
                }
                if grid_axis_span == 0 {
                    return Err(LanePlacementError::InvalidGridAxisSpan {
                        span: grid_axis_span,
                    });
                }
                let start = start_line - 1;
                if start + grid_axis_span > grid_axis_tracks.len() {
                    return Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: start_line,
                        span: grid_axis_span,
                        tracks: grid_axis_tracks.len(),
                    });
                }
                (start, grid_axis_span)
            }
            None => {
                let span = grid_axis_span.clamp(1, grid_axis_tracks.len());
                let start = if matches!(style.grid_flow_tolerance, GridFlowToleranceOf::Infinite) {
                    infinite_candidate_start(cursor, span, grid_axis_tracks.len())
                } else {
                    finite_candidate_start(&running, cursor, span, tolerance)
                };
                (start, span)
            }
        };
        let end = start + span;
        let grid_axis_size = if start < end {
            track_sum(&grid_axis_tracks[start..end], grid_axis_gap)
        } else {
            Tree::Scalar::ZERO
        };
        let lane_axis_margin_box = measure_lane_axis_margin_box_with_grid_axis(
            tree,
            child,
            LaneAxisMarginBoxMeasureInput {
                child_style: &child_style,
                container_style: style,
                constants,
                lane_axis,
                grid_axis,
                grid_axis_size,
            },
        );
        let previous = running[start..end]
            .iter()
            .copied()
            .fold(Tree::Scalar::ZERO, Tree::Scalar::max);
        let new_position = previous + lane_axis_margin_box + lane_gap;
        content_size = content_size.max(new_position - lane_gap);
        for position in &mut running[start..end] {
            *position = new_position;
        }
        item_offsets.push(LaneItemOffsetOf {
            item: child,
            grid_axis_start: start + 1,
            grid_axis_span: span,
            offset: previous,
            lane_axis_margin_box,
        });
        running_positions_after_each_item.push(running.clone());
        cursor = (start + span) % grid_axis_tracks.len();
    }

    let trace = LanePlacementTraceOf {
        report: LanePlacementReportOf {
            lane_axis,
            grid_axis,
            item_offsets,
            content_size,
        },
        running_positions_after_each_item,
        final_cursor: cursor,
    };

    Ok(trace.into_report())
}

pub(super) struct GridLanesLayoutInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) style: &'a NodeInputOf<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) container_content_size: Size<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: Size<S>,
    pub(super) context: GridContainerContext<S>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<Node>,
}

#[derive(Clone, Copy)]
pub(super) struct LaneIntrinsicTrackSizeInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) constants: &'a Constants<S>,
    pub(super) axis: GridAxisKind,
    pub(super) tracks: &'a [TrackSizingOf<S>],
    pub(super) gap: S,
    pub(super) available: AvailableOf<S>,
    pub(super) available_basis: Option<S>,
    pub(super) lines: GridLines,
    pub(super) placements: &'a GridPlacementContext<Node>,
}

pub(super) fn lane_intrinsic_track_sizes<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: LaneIntrinsicTrackSizeInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> Result<Vec<Tree::Scalar>, LanePlacementError>
where
    Tree: Compute,
{
    let LaneIntrinsicTrackSizeInput {
        constants,
        axis,
        tracks,
        gap,
        available,
        available_basis,
        lines,
        placements,
    } = input;
    let content_sized_tracks = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| track_accepts_intrinsic_contribution(*track).then_some(index))
        .collect::<Vec<_>>();
    if tracks.is_empty() || content_sized_tracks.is_empty() {
        return Ok(vec![Tree::Scalar::ZERO; tracks.len()]);
    }

    let children = tree.children(node).collect::<Vec<_>>();
    let mut items = Vec::new();
    for (child, placement) in placements.checked_child_placements(&children) {
        let child_style = tree.node_input(child).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }
        if scroll_container_auto_minimum_zero(&child_style, axis) {
            continue;
        }
        let placement = match axis {
            GridAxisKind::Column => placement.column,
            GridAxisKind::Row => placement.row,
        };
        let (definite_grid_axis_start, grid_axis_span) =
            lane_grid_axis_facts(placement, tracks.len(), grid_axis_lines(lines, axis));
        let contribution =
            lane_child_contribution_facts(tree, child, &child_style, constants, axis, available);
        let item = if definite_grid_axis_start.is_none()
            && lane_child_has_unsupported_indefinite_subgrid(&child_style, axis)
        {
            LaneIntrinsicItemOf::nested_indefinite_subgrid(
                "nested-subgrid",
                LaneTrackSpanLength::new(grid_axis_span)
                    .unwrap_or_else(|| LaneTrackSpanLength::new(1).expect("one is nonzero")),
                contribution,
            )
        } else if let Some(start) = definite_grid_axis_start {
            LaneIntrinsicItemOf::definite(
                "definite-item",
                LaneTrackSpan::new(start, start + grid_axis_span),
                contribution,
            )?
        } else {
            LaneIntrinsicItemOf::indefinite(
                "indefinite-item",
                LaneTrackSpanLength::new(grid_axis_span)
                    .unwrap_or_else(|| LaneTrackSpanLength::new(1).expect("one is nonzero")),
                contribution,
            )
        };
        items.push(item);
    }

    lane_intrinsic_sizing_with(LaneIntrinsicSizingInputOf {
        axis,
        available: available_basis,
        gap,
        tracks: tracks.to_vec(),
        content_sized_tracks,
        items,
    })
    .map(|report| report.final_track_sizes)
}

fn lane_child_has_unsupported_indefinite_subgrid<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axis: GridAxisKind,
) -> bool {
    let axis_has_subgrid = match axis {
        GridAxisKind::Column => subgrid_components(&style.grid_template_columns),
        GridAxisKind::Row => subgrid_components(&style.grid_template_rows),
    };
    axis_has_subgrid && style.display.establishes_grid_formatting_context()
}

fn lane_child_contribution_facts<Tree>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    child_style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    axis: GridAxisKind,
    available: AvailableOf<Tree::Scalar>,
) -> LaneContributionFactsOf<Tree::Scalar>
where
    Tree: Compute,
{
    let min_available = lane_child_intrinsic_available(axis, child_style, available);
    let max_available = lane_child_intrinsic_available(axis, child_style, AvailableOf::MAX_CONTENT);
    let min_output = tree.compute_child(
        child,
        ComputeInputOf {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(
                constants.node_inner_size.width,
                constants.node_inner_size.height,
            ),
            available: min_available,
        },
    );
    let max_output = tree.compute_child(
        child,
        ComputeInputOf {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(
                constants.node_inner_size.width,
                constants.node_inner_size.height,
            ),
            available: max_available,
        },
    );
    let margin = intrinsic_contribution_margin(child_style, constants.node_inner_size.width);
    LaneContributionFactsOf {
        min_content: axis_size(min_output.size, axis) + axis_margin_sum(margin, axis),
        max_content: axis_size(max_output.size, axis) + axis_margin_sum(margin, axis),
        min_size: if automatic_minimum_applies(child_style, axis) {
            axis_size(min_output.size, axis) + axis_margin_sum(margin, axis)
        } else {
            Tree::Scalar::ZERO
        },
        automatic_minimum_applies: automatic_minimum_applies(child_style, axis),
    }
}

fn automatic_minimum_applies<S: LayoutScalar>(style: &NodeInputOf<S>, axis: GridAxisKind) -> bool {
    !scroll_container_auto_minimum_zero(style, axis)
}

fn scroll_container_auto_minimum_zero<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axis: GridAxisKind,
) -> bool {
    match axis {
        GridAxisKind::Column => scroll_container_auto_minimum_zero_inline(style),
        GridAxisKind::Row => scroll_container_auto_minimum_zero_block(style),
    }
}

fn axis_size<S: LayoutScalar>(size: Size<S>, axis: GridAxisKind) -> S {
    match axis {
        GridAxisKind::Column => size.width,
        GridAxisKind::Row => size.height,
    }
}

fn axis_margin_sum<S: LayoutScalar>(margin: Edges<S>, axis: GridAxisKind) -> S {
    match axis {
        GridAxisKind::Column => margin.horizontal_sum(),
        GridAxisKind::Row => margin.vertical_sum(),
    }
}

pub(super) fn layout_grid_lanes_children<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridLanesLayoutInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> GridChildrenLayout<Tree::Scalar>
where
    Tree: Compute,
{
    let GridLanesLayoutInput {
        style,
        constants,
        container_content_size,
        columns,
        rows,
        gap,
        mut context,
        subgrid_report,
        placements,
    } = input;

    if columns.is_empty() || rows.is_empty() {
        for (order, child) in tree
            .children(node)
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
        {
            tree.set_unrounded(child, NodeOutputOf::with_order(order as u32));
            tree.compute_child(child, ComputeInputOf::HIDDEN);
        }
        return GridChildrenLayout {
            visible_content_size: Size::ZERO,
            first_baseline: None,
            last_baseline: None,
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
        };
    }

    let track_content_size = Size::new(track_sum(columns, gap.width), track_sum(rows, gap.height));
    let content_box_size = constants.node_inner_size.unwrap_or(container_content_size);
    let alignment_free_space = content_box_size - track_content_size;
    let column_alignment = grid_alignment(
        alignment_free_space.width,
        columns.len(),
        gap.width,
        style.justify_content.unwrap_or(AlignContent::Stretch),
    );
    let row_alignment = grid_alignment(
        alignment_free_space.height,
        rows.len(),
        gap.height,
        style.align_content.unwrap_or(AlignContent::Stretch),
    );
    context.gap = gap;
    let grid_axis_gap = match grid_axis_for_grid_lanes(style) {
        GridAxisKind::Column => column_alignment.gap,
        GridAxisKind::Row => row_alignment.gap,
    };
    let Ok(lane_report) = resolve_grid_lanes_placement_with_resolved_tracks(
        tree,
        node,
        style,
        constants,
        context.clone(),
        columns,
        rows,
        placements,
        grid_axis_gap,
    ) else {
        return GridChildrenLayout {
            visible_content_size: Size::ZERO,
            first_baseline: None,
            last_baseline: None,
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
        };
    };
    let content_box_left = effective_content_box_left(constants, container_content_size);
    let column_offsets = if style.direction.is_rtl() {
        rtl_offsets(
            columns,
            content_box_left,
            content_box_size.width,
            column_alignment.start,
            column_alignment.gap,
        )
    } else {
        offsets(
            columns,
            constants.content_box_inset.left + column_alignment.start,
            column_alignment.gap,
        )
    };
    let row_offsets = offsets(
        rows,
        constants.content_box_inset.top + row_alignment.start,
        row_alignment.gap,
    );
    let lane_axis = lane_report.lane_axis;
    let lane_axis_alignment_start = match lane_axis {
        GridAxisKind::Column => column_alignment.start + constants.content_box_inset.left,
        GridAxisKind::Row => row_alignment.start + constants.content_box_inset.top,
    };
    let children = tree.children(node).collect::<Vec<_>>();
    let empty_baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); rows.len()],
        columns: vec![TrackBaselineGroup::default(); columns.len()],
    };
    let mut pending_items = Vec::new();
    let mut visible_content_size = Size::ZERO;

    for (order, (child, placement)) in placements.checked_child_placements(&children).enumerate() {
        let child_style = tree.node_input(child).clone();
        if child_style.display == Display::None {
            tree.set_unrounded(child, NodeOutputOf::with_order(order as u32));
            tree.compute_child(child, ComputeInputOf::HIDDEN);
            continue;
        }
        if child_style.position == Position::Absolute {
            visible_content_size = max_size(
                visible_content_size,
                layout_absolute_grid_child(
                    tree,
                    child,
                    order as u32,
                    &child_style,
                    AbsoluteGridContext {
                        container_style: style,
                        constants,
                        column_offsets: &column_offsets,
                        row_offsets: &row_offsets,
                        columns,
                        rows,
                        gap,
                        lines: context.lines,
                        column: placement.absolute_column,
                        row: placement.absolute_row,
                        column_line_offset_adjustment: Tree::Scalar::ZERO,
                    },
                ),
            );
            continue;
        }
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }
        let Some(item_offset) = lane_report
            .item_offsets
            .iter()
            .find(|item| item.item == child)
        else {
            continue;
        };

        let start = item_offset.grid_axis_start - 1;
        let end = start + item_offset.grid_axis_span;
        let (area, area_origin, area_size) = match lane_report.grid_axis {
            GridAxisKind::Column => {
                let width = track_sum(
                    &columns[start..end.min(columns.len())],
                    column_alignment.gap,
                );
                let x = column_offsets
                    .get(start..end.min(column_offsets.len()))
                    .and_then(|offsets| offsets.iter().copied().reduce(Tree::Scalar::min))
                    .unwrap_or(Tree::Scalar::ZERO);
                (
                    GridArea {
                        column: start,
                        row: 0,
                        column_end: end,
                        row_end: 1,
                        size: Size::new(width, item_offset.lane_axis_margin_box),
                    },
                    Point::new(x, lane_axis_alignment_start + item_offset.offset),
                    Size::new(width, item_offset.lane_axis_margin_box),
                )
            }
            GridAxisKind::Row => {
                let height = track_sum(&rows[start..end.min(rows.len())], row_alignment.gap);
                let y = row_offsets
                    .get(start)
                    .copied()
                    .unwrap_or(Tree::Scalar::ZERO);
                let x = if style.direction.is_rtl() {
                    content_box_left + content_box_size.width
                        - column_alignment.start
                        - item_offset.offset
                        - item_offset.lane_axis_margin_box
                } else {
                    lane_axis_alignment_start + item_offset.offset
                };
                (
                    GridArea {
                        column: 0,
                        row: start,
                        column_end: 1,
                        row_end: end,
                        size: Size::new(item_offset.lane_axis_margin_box, height),
                    },
                    Point::new(x, y),
                    Size::new(item_offset.lane_axis_margin_box, height),
                )
            }
        };

        let item = grid_item_sizing(
            &child_style,
            style,
            area_size,
            Size::splat(Some(area_size.width)),
        );
        let area_width_basis = Size::splat(Some(area_size.width));
        let padding = child_style
            .padding
            .zip_inline_size(area_width_basis, |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let border = child_style
            .border
            .zip_inline_size(area_width_basis, |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let resolved_margin = item
            .unresolved_margin
            .map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
        let subgrid_content_box_size =
            (area_size - resolved_margin.sum_axes() - padding.sum_axes() - border.sum_axes())
                .max(Size::ZERO);
        let child_context = subgrid_child_parent_context(SubgridChildParentContextInput {
            item: *subgrid_report
                .items
                .get(order)
                .expect("grid-lanes subgrid report must preserve one item per child"),
            child_style: &child_style,
            area,
            content_box_size: subgrid_content_box_size,
            columns,
            rows,
            gap,
            parent_named_columns: &context.named_columns,
            parent_named_rows: &context.named_rows,
            parent_area_facts: context.area_facts.as_ref(),
            parent_baseline_groups: &empty_baseline_groups,
            margin: item.unresolved_margin,
            border,
            padding,
        });
        let child_input = ComputeInputOf {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: item.known,
            parent: Size::new(Some(area_size.width), Some(area_size.height)),
            available: item
                .available
                .map(|value| AvailableOf::Definite(value.max(Tree::Scalar::ZERO))),
        };
        let output = if child_context.has_inherited_axis() {
            compute_grid_with_context(tree, child, child_input, child_context)
        } else {
            tree.compute_child(child, child_input)
        };
        let horizontal_axis = grid_item_axis(GridItemAxis {
            area_size: area_size.width,
            size: output.size.width,
            margin_start: item.unresolved_margin.left,
            margin_end: item.unresolved_margin.right,
            alignment: item.justify_self,
            direction: style.direction,
        });
        let vertical_axis = grid_item_axis(GridItemAxis {
            area_size: area_size.height,
            size: output.size.height,
            margin_start: item.unresolved_margin.top,
            margin_end: item.unresolved_margin.bottom,
            alignment: item.align_self,
            direction: Direction::Ltr,
        });
        let relative_offset = relative_inset_offset(
            child_style.inset.zip_size(
                Size::new(Some(area_size.width), Some(area_size.height)),
                resolve_auto_optional,
            ),
            style.direction,
            child_style.position,
        );
        let margin = Edges {
            left: horizontal_axis.margin_start,
            right: horizontal_axis.margin_end,
            top: vertical_axis.margin_start,
            bottom: vertical_axis.margin_end,
        };
        let baselines = output.baselines();
        let first_baseline = baselines.first_or_synthesize_block(output.size);
        let last_baseline = baselines.last_or_synthesize_block(output.size);
        let block_auto_margins =
            item.unresolved_margin.top.is_none() || item.unresolved_margin.bottom.is_none();
        let baseline_participation =
            baseline_participation(item.align_self, block_auto_margins, false, baselines);
        let location = Point::new(
            area_origin.x + horizontal_axis.offset + relative_offset.x,
            area_origin.y + vertical_axis.offset + relative_offset.y,
        );
        visible_content_size = max_size(
            visible_content_size,
            content_size_contribution(
                Point::new(location.x - area_origin.x, location.y - area_origin.y),
                output.size,
                output.content_size,
                child_style.overflow,
            ),
        );
        pending_items.push(PendingGridItem {
            node: child,
            order: order as u32,
            area,
            output,
            horizontal_axis,
            vertical_axis,
            relative_offset,
            first_baseline,
            last_baseline,
            published_row_baselines: None,
            block_offset: vertical_axis.offset,
            block_auto_margins,
            baseline_participation,
            margin,
            scrollbar_size: scrollbar_size_from_overflow(
                child_style.overflow,
                child_style.scrollbar_width,
            ),
            border,
            padding,
            overflow: child_style.overflow,
        });
    }

    for item in &mut pending_items {
        // WebKit currently skips masonry baseline offset calculations. Surgeist
        // keeps grid-lanes baseline offsets disabled for lane-axis placement,
        // but still reports synthesized container baselines from final item
        // geometry below.
        let item_offset = lane_report
            .item_offsets
            .iter()
            .find(|offset| offset.item == item.node);
        let location = Point::new(
            match lane_axis {
                GridAxisKind::Column => match item_offset {
                    Some(offset) if style.direction.is_rtl() => {
                        content_box_left + content_box_size.width
                            - column_alignment.start
                            - offset.offset
                            - offset.lane_axis_margin_box
                            + item.horizontal_axis.offset
                            + item.relative_offset.x
                    }
                    _ => {
                        lane_axis_alignment_start
                            + item_offset.map_or(Tree::Scalar::ZERO, |offset| offset.offset)
                            + item.horizontal_axis.offset
                            + item.relative_offset.x
                    }
                },
                GridAxisKind::Row => {
                    grid_area_inline_offset(&column_offsets, item.area)
                        + item.horizontal_axis.offset
                        + item.relative_offset.x
                }
            },
            match lane_axis {
                GridAxisKind::Column => {
                    row_offsets[item.area.row] + item.vertical_axis.offset + item.relative_offset.y
                }
                GridAxisKind::Row => {
                    lane_axis_alignment_start
                        + item_offset.map_or(Tree::Scalar::ZERO, |offset| offset.offset)
                        + item.vertical_axis.offset
                        + item.relative_offset.y
                }
            },
        );
        item.block_offset = location.y - row_offsets[item.area.row];
        tree.set_unrounded(
            item.node,
            NodeOutputOf {
                order: item.order,
                location,
                size: item.output.size,
                content_size: item.output.content_size,
                scroll_geometry: None,
                scrollbar_size: item.scrollbar_size,
                border: item.border,
                padding: item.padding,
                margin: item.margin,
            },
        );
    }
    let baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); rows.len()],
        columns: vec![TrackBaselineGroup::default(); columns.len()],
    };
    let baselines = grid_container_baselines(&pending_items, &baseline_groups, &row_offsets, rows);

    GridChildrenLayout {
        visible_content_size,
        first_baseline: baselines.first,
        last_baseline: baselines.last,
        baseline_groups,
    }
}

#[derive(Clone, Copy)]
pub(super) struct LaneAxisMarginBoxMeasureInput<'a, S: LayoutScalar = Scalar> {
    pub(super) child_style: &'a NodeInputOf<S>,
    pub(super) container_style: &'a NodeInputOf<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) lane_axis: GridAxisKind,
    pub(super) grid_axis: GridAxisKind,
    pub(super) grid_axis_size: S,
}

pub(super) fn measure_lane_axis_margin_box_with_grid_axis<Tree>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    input: LaneAxisMarginBoxMeasureInput<'_, Tree::Scalar>,
) -> Tree::Scalar
where
    Tree: Compute,
{
    let LaneAxisMarginBoxMeasureInput {
        child_style,
        container_style,
        constants,
        lane_axis,
        grid_axis,
        grid_axis_size,
    } = input;
    let area_width_basis = match grid_axis {
        GridAxisKind::Column => Size::splat(Some(grid_axis_size)),
        GridAxisKind::Row => constants.node_inner_size,
    };
    let (margin, known, parent, available) = {
        let unresolved_margin = child_style
            .margin
            .zip_inline_size(area_width_basis, |length, basis| {
                resolve_auto_optional(length, basis)
            });
        let margin = unresolved_margin.map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
        let mut known = Size::NONE;
        let mut parent = Size::NONE;
        let mut available = Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT);
        match lane_axis {
            GridAxisKind::Column => {
                available.width = intrinsic_available_for_dimension(child_style.size.width);
            }
            GridAxisKind::Row => {
                available.height = intrinsic_available_for_dimension(child_style.size.height);
            }
        }
        match grid_axis {
            GridAxisKind::Column => {
                let available_width =
                    (grid_axis_size - margin.horizontal_sum()).max(Tree::Scalar::ZERO);
                let justify_self = child_style
                    .justify_self
                    .or(container_style.justify_items)
                    .unwrap_or(AlignItems::Stretch);
                known.width = resolve_dimension(child_style.size.width, Some(grid_axis_size))
                    .or_else(|| (justify_self == AlignItems::Stretch).then_some(available_width));
                parent.width = Some(grid_axis_size);
                available.width = AvailableOf::Definite(available_width);
            }
            GridAxisKind::Row => {
                let available_height =
                    (grid_axis_size - margin.vertical_sum()).max(Tree::Scalar::ZERO);
                let align_self = child_style
                    .align_self
                    .or(container_style.align_items)
                    .unwrap_or(AlignItems::Stretch);
                known.height = resolve_dimension(child_style.size.height, Some(grid_axis_size))
                    .or_else(|| {
                        (align_self == AlignItems::Stretch && child_style.aspect_ratio.is_none())
                            .then_some(available_height)
                    });
                parent.height = Some(grid_axis_size);
                available.height = AvailableOf::Definite(available_height);
            }
        }
        (margin, known, parent, available)
    };
    let output = tree.compute_child(
        child,
        ComputeInputOf {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known,
            parent,
            available,
        },
    );
    match lane_axis {
        GridAxisKind::Column => output.size.width + margin.horizontal_sum(),
        GridAxisKind::Row => output.size.height + margin.vertical_sum(),
    }
}

fn lane_child_intrinsic_available<S: LayoutScalar>(
    grid_axis: GridAxisKind,
    child_style: &NodeInputOf<S>,
    grid_axis_available: AvailableOf<S>,
) -> Size<AvailableOf<S>> {
    match grid_axis {
        GridAxisKind::Column => Size::new(
            grid_axis_available,
            intrinsic_available_for_dimension(child_style.size.height),
        ),
        GridAxisKind::Row => Size::new(
            intrinsic_available_for_dimension(child_style.size.width),
            grid_axis_available,
        ),
    }
}

fn intrinsic_available_for_dimension<S: LayoutScalar>(dimension: DimensionOf<S>) -> AvailableOf<S> {
    match dimension {
        DimensionOf::MinContent => AvailableOf::MIN_CONTENT,
        DimensionOf::MaxContent => AvailableOf::MAX_CONTENT,
        DimensionOf::Value(_) | DimensionOf::Fr(_) | DimensionOf::Auto => AvailableOf::MAX_CONTENT,
    }
}

fn lane_grid_axis_facts(
    placement: GridPlacement,
    track_count: usize,
    lines: GridAxisLines,
) -> (Option<usize>, usize) {
    if placement.is_auto() {
        return (None, 1);
    }

    if let Some((start, span)) = definite_axis_start_and_span(
        placement,
        track_count,
        lines.explicit_start,
        lines.explicit_count,
    ) {
        return (Some(start + 1), span);
    }

    (None, placement.span().map(|span| span.get()).unwrap_or(1))
}

#[derive(Clone, Copy)]
struct GridAxisLines {
    explicit_start: usize,
    explicit_count: usize,
}

fn grid_axis_lines(lines: GridLines, axis: GridAxisKind) -> GridAxisLines {
    match axis {
        GridAxisKind::Column => GridAxisLines {
            explicit_start: lines.column_explicit_start,
            explicit_count: lines.column_explicit_count,
        },
        GridAxisKind::Row => GridAxisLines {
            explicit_start: lines.row_explicit_start,
            explicit_count: lines.row_explicit_count,
        },
    }
}

fn resolve_tolerance<S: LayoutScalar>(tolerance: GridFlowToleranceOf<S>, basis: S) -> S {
    let Ok(basis) = PercentageBasisOf::definite(basis) else {
        return S::NAN;
    };
    let basis_value = basis
        .definite_value()
        .expect("validated definite basis")
        .get();

    match tolerance {
        GridFlowToleranceOf::Normal { font_size } => font_size,
        GridFlowToleranceOf::Length(length) => resolution_or_zero(length.resolve_against(basis)),
        GridFlowToleranceOf::Percent(factor) => factor * basis_value,
        GridFlowToleranceOf::Infinite => S::INFINITY,
    }
}

fn infinite_candidate_start(cursor: usize, span: usize, track_count: usize) -> usize {
    if cursor + span > track_count {
        0
    } else {
        cursor
    }
}

fn finite_candidate_start<S: LayoutScalar>(
    running: &[S],
    cursor: usize,
    span: usize,
    tolerance: S,
) -> usize {
    let track_count = running.len();
    let max_start = track_count + 1 - span;
    let shifted_cursor = if cursor >= max_start { 0 } else { cursor };
    let absolute_shortest = (0..max_start)
        .map(|start| max_running_position(running, start, span))
        .fold(S::INFINITY, S::min);

    for offset in 0..max_start {
        let start = (shifted_cursor + offset) % max_start;
        if max_running_position(running, start, span) <= absolute_shortest + tolerance {
            return start;
        }
    }

    0
}

fn max_running_position<S: LayoutScalar>(running: &[S], start: usize, span: usize) -> S {
    running[start..start + span]
        .iter()
        .copied()
        .fold(S::ZERO, S::max)
}
