use super::*;
use crate::compute::{
    ResolvedPreferredSize, SizingResolutionError, resolve_maximum_optional,
    resolve_minimum_optional, resolve_preferred_sizing, sizing_resolution_error,
};
use crate::geometry::{LogicalAxis, LogicalPointOf, LogicalSizeOf, PhysicalAxis};
use crate::scroll::{UsedOverflow, scrollbar_size_from_overflow};
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
    InvalidGridFlowToleranceBasis,
    InvalidGridFlowToleranceResolution,
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
    let tolerance = resolve_tolerance(input.tolerance, input.tolerance_basis)?;
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
) -> LayoutResultOf<(), Result<LaneIntrinsicSizingReportOf<S>, LanePlacementError>, S> {
    lane_intrinsic_sizing_with(input, LayoutErrorSiteOf::Standalone)
}

pub(super) fn lane_intrinsic_sizing_with<Node, S, M>(
    input: LaneIntrinsicSizingInputOf<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, Result<LaneIntrinsicSizingReportOf<S>, LanePlacementError>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    if input.content_sized_tracks.is_empty() || input.tracks.is_empty() {
        return Ok(Err(LanePlacementError::EmptyTrackList));
    }
    if let Some(track_index) = input
        .content_sized_tracks
        .iter()
        .copied()
        .find(|track_index| *track_index >= input.tracks.len())
    {
        return Ok(Err(LanePlacementError::ContentSizedTrackOutOfRange {
            track_index,
            tracks: input.tracks.len(),
        }));
    }

    let mut definite_items = Vec::new();
    let mut indefinite_groups: Vec<IndefiniteLaneContributionGroupOf<S>> = Vec::new();

    for item in &input.items {
        match item.kind() {
            LaneIntrinsicItemKind::Definite { span } => {
                if span.len().is_none() || span.end > input.tracks.len() + 1 {
                    return Ok(Err(LanePlacementError::DefiniteLaneSpanOutOfRange {
                        span,
                        tracks: input.tracks.len(),
                    }));
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
                return Ok(Err(
                    LanePlacementError::NestedGridLanesSubgridIndefiniteUnsupported,
                ));
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
                    site,
                )?);
            }
        }
    }

    let mut final_track_sizes = input
        .tracks
        .iter()
        .map(|track| initialized_track_base(track.clone(), input.available, site))
        .collect::<LayoutResultOf<Node, Vec<_>, S, M>>()?;
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
            site,
        )?;
    }
    for item in &sizing_items {
        apply_lane_sizing_contribution(
            &mut final_track_sizes,
            &input.tracks,
            input.gap,
            input.available,
            *item,
            site,
        )?;
    }

    Ok(Ok(LaneIntrinsicSizingReportOf {
        definite_items,
        indefinite_groups,
        converted_indefinite_items,
        final_track_sizes,
    }))
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

fn masonry_sizing_contribution<Node, S, M>(
    projection: MasonrySizingProjection<'_, S>,
    group: &IndefiniteLaneContributionGroupOf<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, DefiniteLaneIntrinsicItemOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
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
        .map(|track| masonry_track_minimum_size(track.clone(), group))
        .fold(S::ZERO, S::max);
    let full_existing = tracks[full_start_index..full_end_index]
        .iter()
        .map(|track| initialized_track_base(track.clone(), available, site))
        .collect::<LayoutResultOf<Node, Vec<_>, S, M>>()?
        .into_iter()
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
        .map(|track| initialized_track_base(track.clone(), available, site))
        .collect::<LayoutResultOf<Node, Vec<_>, S, M>>()?
        .into_iter()
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
        .map(|track| masonry_track_maximum_size(track.clone(), size, group))
        .fold(S::ZERO, S::max);

    Ok(DefiniteLaneIntrinsicItemOf {
        id: "indefinite-group",
        span,
        contribution: LaneContributionFactsOf {
            min_content: size,
            max_content,
            min_size: size,
            automatic_minimum_applies: false,
        },
    })
}

fn masonry_track_minimum_size<S: LayoutScalar>(
    track: TrackSizingOf<S>,
    group: &IndefiniteLaneContributionGroupOf<S>,
) -> S {
    match track.min {
        MinTrackSizingOf::MinContent => group.max_min_content,
        MinTrackSizingOf::MaxContent => group.max_max_content,
        MinTrackSizingOf::Auto | MinTrackSizingOf::Calculation(_) => group.max_min_size,
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
        MaxTrackSizingOf::Calculation(_) | MaxTrackSizingOf::Flex(_) => minimum_size,
    }
}

fn initialized_track_base<Node, S, M>(
    track: TrackSizingOf<S>,
    available: Option<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, S, S, M>
where
    S: LayoutScalar,
{
    match track.min {
        MinTrackSizingOf::Calculation(calculation) => {
            let resolution = super::tracks::resolve_track_calculation(&calculation, available);
            resolution_or_zero(resolution, site)
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => {
            Ok(S::ZERO)
        }
    }
}

fn resolution_or_zero<Node, S, M>(
    resolution: LengthResolutionOf<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, S, S, M>
where
    S: LayoutScalar,
{
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution
            .value
            .expect("resolved length resolution must carry a value")),
        LengthResolutionStatus::InvalidNumeric { .. } => Err(
            crate::compute::value_resolution_error_at_site(site, resolution.status()),
        ),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => Ok(S::ZERO),
    }
}

fn span_overlaps_content_tracks(span: LaneTrackSpan, content_sized_tracks: &[usize]) -> bool {
    let start = span.start - 1;
    let end = span.end - 1;
    content_sized_tracks
        .iter()
        .any(|track_index| (start..end).contains(track_index))
}

fn apply_lane_sizing_contribution<Node, S, M>(
    sizes: &mut [S],
    tracks: &[TrackSizingOf<S>],
    gap: S,
    available: Option<S>,
    item: DefiniteLaneIntrinsicItemOf<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, (), S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let start = item.span.start - 1;
    let end = item.span.end - 1;
    let Some(span_tracks) = tracks.get(start..end) else {
        return Ok(());
    };
    if span_tracks.is_empty() || end > sizes.len() {
        return Ok(());
    }
    let contribution = item.contribution.contributions();
    if end == start + 1 {
        sizes[start] = sizes[start].max(lane_track_minimum_size(
            span_tracks[0].clone(),
            contribution,
            available,
            site,
        )?);
        return Ok(());
    }

    let target = span_contribution(contribution.minimum, end - start, gap);
    let current = sizes[start..end]
        .iter()
        .copied()
        .fold(S::ZERO, |sum, size| sum + size)
        + span_tracks
            .iter()
            .map(|track| {
                if track_accepts_intrinsic_contribution(track) {
                    Ok(S::ZERO)
                } else {
                    initialized_track_base(track.clone(), available, site)
                }
            })
            .collect::<LayoutResultOf<Node, Vec<_>, S, M>>()?
            .into_iter()
            .fold(S::ZERO, |sum, size| sum + size);
    let extra = (target - current).max(S::ZERO);
    if extra == S::ZERO {
        return Ok(());
    }
    let share = extra / S::from_usize(end - start);
    for size in &mut sizes[start..end] {
        *size = *size + share;
    }
    Ok(())
}

fn lane_track_minimum_size<Node, S, M>(
    track: TrackSizingOf<S>,
    contribution: LaneContributionsOf<S>,
    available: Option<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, S, S, M>
where
    S: LayoutScalar,
{
    match track.min {
        MinTrackSizingOf::MinContent => Ok(contribution.min_content),
        MinTrackSizingOf::MaxContent => Ok(contribution.max_content),
        MinTrackSizingOf::Auto => Ok(contribution.minimum),
        MinTrackSizingOf::Calculation(_) => initialized_track_base(track, available, site),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "lane placement resolution keeps explicit grid layout phase inputs separate"
)]
#[expect(
    clippy::type_complexity,
    reason = "lane placement preserves the session's node, scalar, and provider error types"
)]
pub(super) fn resolve_grid_lanes_placement_with_resolved_tracks<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    context: GridContainerContext<Tree::Scalar>,
    columns: &[Tree::Scalar],
    rows: &[Tree::Scalar],
    placements: &GridPlacementContext<<Tree as Traverse>::Node>,
    grid_axis_gap: Tree::Scalar,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Result<LanePlacementReportOf<<Tree as Traverse>::Node, Tree::Scalar>, LanePlacementError>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let grid_axis = grid_axis_for_grid_lanes(style);
    let lane_axis = lane_axis_for_grid_lanes(style);
    let grid_axis_tracks = match grid_axis {
        GridAxisKind::Column => columns,
        GridAxisKind::Row => rows,
    };
    if grid_axis_tracks.is_empty() {
        return Ok(Err(LanePlacementError::EmptyTrackList));
    }

    let tolerance = match resolve_tolerance(
        style.grid_flow_tolerance,
        match grid_axis {
            GridAxisKind::Column => context.percent_basis.inline.unwrap_or(Tree::Scalar::ZERO),
            GridAxisKind::Row => context.percent_basis.block.unwrap_or(Tree::Scalar::ZERO),
        },
    ) {
        Ok(tolerance) => tolerance,
        Err(error) => return Ok(Err(error)),
    };
    let lane_gap = match lane_axis {
        GridAxisKind::Column => context.gap.inline,
        GridAxisKind::Row => context.gap.block,
    };
    let mut running = vec![Tree::Scalar::ZERO; grid_axis_tracks.len()];
    let mut item_offsets = Vec::new();
    let mut running_positions_after_each_item = Vec::new();
    let mut cursor = 0usize;
    let mut content_size = Tree::Scalar::ZERO;

    let children = tree.children(node).collect::<Vec<_>>();
    let _ = placements.checked_child_placements(&children);
    for source_index in &placements.order_modified_indexes {
        let source_slot = source_index.get();
        let child = children[source_slot];
        let placement = placements.items[source_slot];
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
                    return Ok(Err(LanePlacementError::InvalidGridAxisStart {
                        start: start_line,
                    }));
                }
                if grid_axis_span == 0 {
                    return Ok(Err(LanePlacementError::InvalidGridAxisSpan {
                        span: grid_axis_span,
                    }));
                }
                let start = start_line - 1;
                if start + grid_axis_span > grid_axis_tracks.len() {
                    return Ok(Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: start_line,
                        span: grid_axis_span,
                        tracks: grid_axis_tracks.len(),
                    }));
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
        )?;
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

    Ok(Ok(trace.into_report()))
}

pub(super) struct GridLanesLayoutInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) style: &'a NodeInputOf<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) container_content_box_size: LogicalSizeOf<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) context: GridContainerContext<S>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<Node>,
    pub(super) containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
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

#[expect(
    clippy::type_complexity,
    reason = "lane intrinsic sizing preserves the session's node, scalar, and provider error types"
)]
pub(super) fn lane_intrinsic_track_sizes<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: LaneIntrinsicTrackSizeInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Result<Vec<Tree::Scalar>, LanePlacementError>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
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
        .filter_map(|(index, track)| track_accepts_intrinsic_contribution(track).then_some(index))
        .collect::<Vec<_>>();
    if tracks.is_empty() || content_sized_tracks.is_empty() {
        return Ok(Ok(vec![Tree::Scalar::ZERO; tracks.len()]));
    }

    let children = tree.children(node).collect::<Vec<_>>();
    let mut items = Vec::new();
    let _ = placements.checked_child_placements(&children);
    for source_index in &placements.order_modified_indexes {
        let source_slot = source_index.get();
        let child = children[source_slot];
        let placement = placements.items[source_slot];
        let child_style = tree.node_input(child).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }
        if scroll_container_auto_minimum_zero(&child_style, constants.flow_axes, axis) {
            continue;
        }
        let placement = match axis {
            GridAxisKind::Column => placement.column,
            GridAxisKind::Row => placement.row,
        };
        let (definite_grid_axis_start, grid_axis_span) =
            lane_grid_axis_facts(placement, tracks.len(), grid_axis_lines(lines, axis));
        let contribution =
            lane_child_contribution_facts(tree, child, &child_style, constants, axis, available)?;
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
            match LaneIntrinsicItemOf::definite(
                "definite-item",
                LaneTrackSpan::new(start, start + grid_axis_span),
                contribution,
            ) {
                Ok(item) => item,
                Err(error) => return Ok(Err(error)),
            }
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

    lane_intrinsic_sizing_with(
        LaneIntrinsicSizingInputOf {
            axis,
            available: available_basis,
            gap,
            tracks: tracks.to_vec(),
            content_sized_tracks,
            items,
        },
        LayoutErrorSiteOf::Node(node),
    )
    .map(|result| result.map(|report| report.final_track_sizes))
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

fn lane_child_contribution_facts<Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    child_style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    axis: GridAxisKind,
    available: AvailableOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, LaneContributionFactsOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let preferred_size = grid_lanes_child_sizing_preflight(
        child_style,
        Size::new(
            constants.node_inner_size.width,
            constants.node_inner_size.height,
        ),
    )
    .map_err(|error| sizing_resolution_error(child, error))?;
    let min_available =
        lane_child_intrinsic_available(constants.flow_axes, axis, preferred_size, available);
    let max_available = lane_child_intrinsic_available(
        constants.flow_axes,
        axis,
        preferred_size,
        AvailableOf::MAX_CONTENT,
    );
    let min_output = tree.compute_child(
        child,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(
                constants.node_inner_size.width,
                constants.node_inner_size.height,
            ),
            crate::ContainingLayoutContext::new(
                constants.flow_axes,
                crate::ParentFormattingContext::Grid,
            ),
            min_available,
        ),
    )?;
    let max_output = tree.compute_child(
        child,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(
                constants.node_inner_size.width,
                constants.node_inner_size.height,
            ),
            crate::ContainingLayoutContext::new(
                constants.flow_axes,
                crate::ParentFormattingContext::Grid,
            ),
            max_available,
        ),
    )?;
    let margin =
        intrinsic_contribution_margin(child_style, constants.flow_axes, constants.node_inner_size)
            .map_err(|status| crate::compute::value_resolution_error(child, status))?;
    let used_overflow = grid_axis_used_overflow(child_style, constants.flow_axes, axis);
    let min_output_size = lane_axis_size(constants.flow_axes.logical_size(min_output.size), axis);
    let min_content_size = lane_axis_size(
        constants.flow_axes.logical_size(min_output.content_size),
        axis,
    );
    let max_output_size = lane_axis_size(constants.flow_axes.logical_size(max_output.size), axis);
    let max_content_size = lane_axis_size(
        constants.flow_axes.logical_size(max_output.content_size),
        axis,
    );
    let min_contribution = if used_overflow.value() == Overflow::Visible {
        min_output_size.max(min_content_size)
    } else {
        min_output_size
    };
    let max_contribution = if used_overflow.value() == Overflow::Visible {
        max_output_size.max(max_content_size)
    } else {
        max_output_size
    };
    let logical_margin = constants.flow_axes.logical_edges(margin);
    let margin = lane_axis_margin_sum(logical_margin, axis);
    Ok(LaneContributionFactsOf {
        min_content: min_contribution + margin,
        max_content: max_contribution + margin,
        min_size: if automatic_minimum_applies(child_style, constants.flow_axes, axis) {
            min_contribution + margin
        } else {
            Tree::Scalar::ZERO
        },
        automatic_minimum_applies: automatic_minimum_applies(
            child_style,
            constants.flow_axes,
            axis,
        ),
    })
}

fn automatic_minimum_applies<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: crate::geometry::FlowAxes,
    axis: GridAxisKind,
) -> bool {
    !scroll_container_auto_minimum_zero(style, flow_axes, axis)
}

fn scroll_container_auto_minimum_zero<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: crate::geometry::FlowAxes,
    axis: GridAxisKind,
) -> bool {
    grid_axis_computed_overflow(style, flow_axes, axis).is_scrollable()
        && grid_axis_size(flow_axes, style.size.clone(), axis).is_auto()
}

fn lane_axis_size<S: LayoutScalar>(size: LogicalSizeOf<S>, axis: GridAxisKind) -> S {
    match axis.logical_axis() {
        LogicalAxis::Inline => size.inline,
        LogicalAxis::Block => size.block,
    }
}

fn lane_axis_margin_sum<S: LayoutScalar>(
    margin: crate::geometry::LogicalEdgesOf<S>,
    axis: GridAxisKind,
) -> S {
    match axis.logical_axis() {
        LogicalAxis::Inline => margin.inline_sum(),
        LogicalAxis::Block => margin.block_sum(),
    }
}

pub(super) fn layout_grid_lanes_children<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridLanesLayoutInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridChildrenLayout<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let GridLanesLayoutInput {
        style,
        constants,
        container_content_box_size,
        columns,
        rows,
        gap,
        mut context,
        subgrid_report,
        placements,
        containing_auto_scrollbar_pass,
    } = input;

    if columns.is_empty() || rows.is_empty() {
        for (source_index, child) in tree
            .children(node)
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
        {
            tree.set_unrounded(
                child,
                NodeOutputOf::with_source_index(crate::SourceIndex::new(source_index)),
            );
            tree.compute_child(
                child,
                ComputeInputOf::hidden_in_containing_pass(
                    crate::ContainingLayoutContext::new(
                        constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    containing_auto_scrollbar_pass,
                ),
            )?;
        }
        return Ok(GridChildrenLayout {
            visible_content_size: Size::ZERO,
            contributions: empty_grid_contributions(),
            baselines: BaselinesOf::NONE,
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
        });
    }

    let flow_axes = constants.flow_axes;
    let track_content_size =
        LogicalSizeOf::new(track_sum(columns, gap.inline), track_sum(rows, gap.block));
    let content_box_size = flow_axes
        .logical_size(constants.node_inner_size)
        .unwrap_or(container_content_box_size);
    let alignment_free_space = content_box_size - track_content_size;
    let column_alignment = grid_alignment(
        alignment_free_space.inline,
        columns.len(),
        gap.inline,
        style.justify_content.unwrap_or(AlignContent::Stretch),
    );
    let row_alignment = grid_alignment(
        alignment_free_space.block,
        rows.len(),
        gap.block,
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
    )?
    else {
        return Ok(GridChildrenLayout {
            visible_content_size: Size::ZERO,
            contributions: empty_grid_contributions(),
            baselines: BaselinesOf::NONE,
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
        });
    };
    let logical_content_box_inset = flow_axes.logical_edges(constants.content_box_inset);
    let column_offsets = grid_axis_logical_offsets(
        columns,
        None,
        logical_content_box_inset.inline_start,
        column_alignment,
    );
    let row_offsets = grid_axis_logical_offsets(
        rows,
        None,
        logical_content_box_inset.block_start,
        row_alignment,
    );
    let containing_size = constants.node_outer_size.unwrap_or(
        flow_axes.physical_size(container_content_box_size)
            + constants.content_box_inset.sum_axes(),
    );
    let lane_axis = lane_report.lane_axis;
    let lane_axis_alignment_start = match lane_axis.logical_axis() {
        LogicalAxis::Inline => column_alignment.start + logical_content_box_inset.inline_start,
        LogicalAxis::Block => row_alignment.start + logical_content_box_inset.block_start,
    };
    let children = tree.children(node).collect::<Vec<_>>();
    let empty_baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); rows.len()],
        columns: vec![TrackBaselineGroup::default(); columns.len()],
    };
    let mut pending_items = Vec::new();
    let mut child_contributions = Vec::new();

    for (source_index, (child, placement)) in
        placements.checked_child_placements(&children).enumerate()
    {
        let child_style = tree.node_input(child).clone();
        if child_style.display == Display::None {
            tree.set_unrounded(
                child,
                NodeOutputOf::with_source_index(crate::SourceIndex::new(source_index)),
            );
            tree.compute_child(
                child,
                ComputeInputOf::hidden(crate::ContainingLayoutContext::new(
                    constants.flow_axes,
                    crate::ParentFormattingContext::Grid,
                )),
            )?;
            continue;
        }
        if child_style.position == Position::Absolute {
            child_contributions.push(layout_absolute_grid_child(
                tree,
                child,
                source_index,
                &child_style,
                AbsoluteGridContext::ordinary(OrdinaryAbsoluteGridContextInput {
                    container_style: style,
                    constants,
                    containing_size,
                    column: placement.absolute_column,
                    row: placement.absolute_row,
                    column_offsets: &column_offsets,
                    row_offsets: &row_offsets,
                    columns,
                    rows,
                    gap,
                    lines: context.lines,
                })
                .with_containing_auto_scrollbar_pass(containing_auto_scrollbar_pass),
            )?);
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
        let area = match lane_report.grid_axis {
            GridAxisKind::Column => {
                let inline = track_sum(
                    &columns[start..end.min(columns.len())],
                    column_alignment.gap,
                );
                GridArea {
                    column: start,
                    row: 0,
                    column_end: end,
                    row_end: 1,
                    size: LogicalSizeOf::new(inline, item_offset.lane_axis_margin_box),
                }
            }
            GridAxisKind::Row => {
                let block = track_sum(&rows[start..end.min(rows.len())], row_alignment.gap);
                GridArea {
                    column: 0,
                    row: start,
                    column_end: 1,
                    row_end: end,
                    size: LogicalSizeOf::new(item_offset.lane_axis_margin_box, block),
                }
            }
        };

        let physical_area_size = flow_axes.physical_size(area.size);
        let item = grid_item_sizing_for_grid_flow::<Tree, M>(
            tree,
            child,
            &child_style,
            style,
            physical_area_size,
            physical_area_size.map(Some),
            flow_axes,
        )?;
        let area_parent = physical_area_size.map(Some);
        let padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.padding,
                area_parent,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, child)?;
        let border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.border,
                area_parent,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, child)?;
        let resolved_margin = item
            .unresolved_margin
            .map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
        let subgrid_content_box_size = (physical_area_size
            - resolved_margin.sum_axes()
            - padding.sum_axes()
            - border.sum_axes())
        .max(Size::ZERO);
        let child_context = subgrid_child_parent_context(SubgridChildParentContextInput {
            item: *subgrid_report
                .items
                .get(source_index)
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
        })
        .map_err(|error| subgrid_child_context_container_error(node, child, error))?;
        let child_flow_axes =
            crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction);
        let child_input = ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            item.known,
            physical_area_size.map(Some),
            crate::ContainingLayoutContext::new(
                constants.flow_axes,
                crate::ParentFormattingContext::Grid,
            ),
            item.available
                .map(|value| AvailableOf::Definite(value.max(Tree::Scalar::ZERO))),
        )
        .with_containing_auto_scrollbar_pass(containing_auto_scrollbar_pass);
        let mut output = if child_context.has_inherited_axis() {
            compute_grid_with_context(tree, child, child_input, child_context)?
        } else {
            tree.compute_child(child, child_input)?
        };
        let scroll_geometry = retained_grid_child_scroll_geometry(
            &child_style,
            output.size,
            output.content_size,
            padding,
            border,
            output.scroll_geometry,
        )
        .map_err(|error| grid_child_geometry_error(node, child, error))?;
        output.scroll_geometry = Some(scroll_geometry);
        let logical_output_size = flow_axes.logical_size(output.size);
        let logical_unresolved_margin = flow_axes.logical_edges(item.unresolved_margin);
        let inline_axis = logical_grid_item_axis(
            area.size.inline,
            logical_output_size.inline,
            logical_unresolved_margin.inline_start,
            logical_unresolved_margin.inline_end,
            item.justify_self,
        );
        let block_axis = logical_grid_item_axis(
            area.size.block,
            logical_output_size.block,
            logical_unresolved_margin.block_start,
            logical_unresolved_margin.block_end,
            item.align_self,
        );
        let logical_relative_offset = logical_relative_inset_offset(
            child_style
                .inset
                .zip_size(physical_area_size.map(Some), resolve_auto_optional)
                .transpose_with_node(tree, child)?,
            flow_axes,
            child_style.position,
        );
        let margin = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            inline_axis.margin_start,
            inline_axis.margin_end,
            block_axis.margin_start,
            block_axis.margin_end,
        ));
        let baselines = output.baselines();
        let first_baseline =
            baselines.first_or_synthesize_block_baseline(child_flow_axes, output.size);
        let last_baseline =
            baselines.last_or_synthesize_block_baseline(child_flow_axes, output.size);
        let block_auto_margins = child_flow_axes
            .line_over_edge(item.unresolved_margin)
            .is_none()
            || child_flow_axes
                .line_under_edge(item.unresolved_margin)
                .is_none();
        let baseline_participation = baseline_participation_for_container(
            item.align_self,
            block_auto_margins,
            false,
            baselines,
            child_flow_axes,
            constants.flow_axes,
        );
        pending_items.push(PendingGridItem {
            node: child,
            source_index,
            area,
            output,
            horizontal_axis: inline_axis,
            vertical_axis: block_axis,
            child_flow_axes,
            logical_relative_offset,
            first_baseline,
            last_baseline,
            location: Point::ZERO,
            published_row_baselines: None,
            block_offset: block_axis.offset,
            block_auto_margins,
            baseline_participation,
            margin,
            scrollbar_size: scrollbar_size_from_overflow(
                child_style.overflow,
                child_style.item_is_replaced,
                child_style.scrollbar_width.get(),
            ),
            border,
            padding,
            overflow: UsedOverflow::from_computed(
                child_style.overflow,
                child_style.item_is_replaced,
            ),
        });
    }

    for item in &mut pending_items {
        // Grid-lanes keeps its container baseline selection attached to the
        // selected running bucket while final placement projects that bucket
        // along the lane axis.
        let item_offset = lane_report
            .item_offsets
            .iter()
            .find(|offset| offset.item == item.node);
        let lane_offset = item_offset.map_or(Tree::Scalar::ZERO, |offset| offset.offset);
        let selected_lane_offset = item_offset.map_or(lane_axis_alignment_start, |offset| {
            let start = offset.grid_axis_start - 1;
            grid_area_track_offset(&column_offsets, start, start + offset.grid_axis_span)
        });
        let logical_location = match lane_axis.logical_axis() {
            LogicalAxis::Inline => LogicalPointOf::new(
                selected_lane_offset
                    + lane_offset
                    + item.horizontal_axis.offset
                    + item.logical_relative_offset.inline,
                grid_area_track_offset(&row_offsets, 0, 1)
                    + item.vertical_axis.offset
                    + item.logical_relative_offset.block,
            ),
            LogicalAxis::Block => LogicalPointOf::new(
                grid_area_inline_offset(&column_offsets, item.area)
                    + item.horizontal_axis.offset
                    + item.logical_relative_offset.inline,
                lane_axis_alignment_start
                    + lane_offset
                    + item.vertical_axis.offset
                    + item.logical_relative_offset.block,
            ),
        };
        let location = flow_axes.physical_point(
            logical_location,
            flow_axes.logical_size(item.output.size),
            containing_size,
        );
        let baseline_location = match lane_axis.logical_axis() {
            LogicalAxis::Inline => flow_axes.physical_point(
                LogicalPointOf::new(
                    lane_axis_alignment_start
                        + lane_offset
                        + item.horizontal_axis.offset
                        + item.logical_relative_offset.inline,
                    grid_area_track_offset(&row_offsets, item.area.row, item.area.row_end)
                        + item.vertical_axis.offset
                        + item.logical_relative_offset.block,
                ),
                flow_axes.logical_size(item.output.size),
                containing_size,
            ),
            LogicalAxis::Block => location,
        };
        let area_origin = LogicalPointOf::new(
            grid_area_inline_offset(&column_offsets, item.area),
            grid_area_track_offset(&row_offsets, item.area.row, item.area.row_end),
        );
        item.block_offset = logical_location.block - area_origin.block;
        item.location = baseline_location;
        let scroll_geometry = item
            .output
            .scroll_geometry
            .expect("pending grid-lanes item retains canonical geometry");
        let subgrid_item = subgrid_report
            .items
            .get(item.source_index)
            .copied()
            .expect("grid-lanes subgrid report preserves source identity");
        let (horizontal, vertical) =
            subgrid_parent_propagation_axes(subgrid_item, flow_axes, item.child_flow_axes);
        child_contributions.push(GridChildContribution {
            source_index: crate::SourceIndex::new(item.source_index),
            location,
            margin: item.margin,
            geometry: scroll_geometry,
            descendants: scroll_geometry
                .propagatable_descendant_intervals()
                .retain_physical_axes(horizontal, vertical),
            overflow: item.overflow,
            in_flow: true,
        });
        tree.set_unrounded(
            item.node,
            NodeOutputOf {
                source_index: crate::SourceIndex::new(item.source_index),
                location,
                size: item.output.size,
                content_size: item.output.content_size,
                scroll_geometry: Some(scroll_geometry),
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
    let baselines = logical_grid_container_baselines(
        &pending_items,
        &baseline_groups,
        &row_offsets,
        rows,
        flow_axes,
        containing_size,
    );
    let mut contributions =
        grid_scroll_contributions(child_contributions, flow_axes, constants.padding)
            .map_err(|error| grid_child_geometry_error(node, node, error))?;
    let inline_start = column_offsets
        .iter()
        .copied()
        .reduce(Tree::Scalar::min)
        .unwrap_or(logical_content_box_inset.inline_start);
    let inline_end = column_offsets
        .iter()
        .copied()
        .zip(columns.iter().copied())
        .map(|(offset, size)| offset + size)
        .reduce(Tree::Scalar::max)
        .unwrap_or(inline_start);
    let block_start = row_offsets
        .iter()
        .copied()
        .reduce(Tree::Scalar::min)
        .unwrap_or(logical_content_box_inset.block_start);
    let block_end = row_offsets
        .iter()
        .copied()
        .zip(rows.iter().copied())
        .map(|(offset, size)| offset + size)
        .reduce(Tree::Scalar::max)
        .unwrap_or(block_start);
    let logical_subject_size =
        LogicalSizeOf::new(inline_end - inline_start, block_end - block_start);
    let subject_size = flow_axes.physical_size(logical_subject_size);
    let subject_origin = flow_axes.physical_point(
        LogicalPointOf::new(inline_start, block_start),
        logical_subject_size,
        containing_size,
    );
    let track_subject = crate::ScrollRectOf::try_new(subject_origin, subject_size)
        .map_err(|error| grid_child_geometry_error(node, node, error))?;
    if style.justify_content.is_some() {
        contributions.set_active_alignment_subject(flow_axes.inline_axis(), track_subject);
    }
    if style.align_content.is_some() {
        contributions.set_active_alignment_subject(flow_axes.block_axis(), track_subject);
    }
    let visible_content_size = contributions
        .content_size_from_anchor(Point::ZERO)
        .map_err(|error| grid_child_geometry_error(node, node, error))?;

    Ok(GridChildrenLayout {
        visible_content_size,
        contributions,
        baselines: baselines.baselines,
        baseline_groups,
    })
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

pub(super) fn measure_lane_axis_margin_box_with_grid_axis<Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    input: LaneAxisMarginBoxMeasureInput<'_, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let LaneAxisMarginBoxMeasureInput {
        child_style,
        container_style,
        constants,
        lane_axis,
        grid_axis,
        grid_axis_size,
    } = input;
    let flow_axes = constants.flow_axes;
    let logical_parent = match grid_axis.logical_axis() {
        LogicalAxis::Inline => LogicalSizeOf::new(Some(grid_axis_size), None),
        LogicalAxis::Block => LogicalSizeOf::new(None, Some(grid_axis_size)),
    };
    let containing_physical_size = flow_axes.physical_size(logical_parent);
    let preferred_size = grid_lanes_child_sizing_preflight(child_style, containing_physical_size)
        .map_err(|error| sizing_resolution_error(child, error))?;
    let logical_preferred_size = flow_axes.logical_size(preferred_size);
    let (margin, known, parent, available) = {
        let unresolved_margin = flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.margin,
                containing_physical_size,
                resolve_auto_optional,
            )
            .transpose_with_node(tree, child)?;
        let margin = unresolved_margin.map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
        let logical_margin = flow_axes.logical_edges(margin);
        let logical_style_size = flow_axes.logical_size(child_style.size.clone());
        let mut logical_known = LogicalSizeOf::new(None, None);
        let mut logical_parent = LogicalSizeOf::new(None, None);
        let mut logical_available =
            LogicalSizeOf::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT);
        match lane_axis.logical_axis() {
            LogicalAxis::Inline => {
                logical_available.inline =
                    intrinsic_available_for_dimension(logical_preferred_size.inline);
            }
            LogicalAxis::Block => {
                logical_available.block =
                    intrinsic_available_for_dimension(logical_preferred_size.block);
            }
        }
        match grid_axis.logical_axis() {
            LogicalAxis::Inline => {
                let available_inline =
                    (grid_axis_size - logical_margin.inline_sum()).max(Tree::Scalar::ZERO);
                let justify_self = resolve_grid_item_normal_alignment(
                    child_style.justify_self,
                    container_style.justify_items,
                    child_style.item_is_replaced,
                    logical_style_size.inline.is_auto(),
                    AlignItems::Stretch,
                );
                logical_known.inline = definite_preferred_size(logical_preferred_size.inline)
                    .or_else(|| (justify_self == AlignItems::Stretch).then_some(available_inline));
                logical_parent.inline = Some(grid_axis_size);
                logical_available.inline = AvailableOf::Definite(available_inline);
            }
            LogicalAxis::Block => {
                let available_block =
                    (grid_axis_size - logical_margin.block_sum()).max(Tree::Scalar::ZERO);
                let align_self = resolve_grid_item_normal_alignment(
                    child_style.align_self,
                    container_style.align_items,
                    child_style.item_is_replaced,
                    logical_style_size.block.is_auto(),
                    AlignItems::Stretch,
                );
                logical_known.block = definite_preferred_size(logical_preferred_size.block)
                    .or_else(|| {
                        (align_self == AlignItems::Stretch && child_style.aspect_ratio.is_none())
                            .then_some(available_block)
                    });
                logical_parent.block = Some(grid_axis_size);
                logical_available.block = AvailableOf::Definite(available_block);
            }
        }
        (
            logical_margin,
            flow_axes.physical_size(logical_known),
            flow_axes.physical_size(logical_parent),
            flow_axes.physical_size(logical_available),
        )
    };
    let output = tree.compute_child(
        child,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            known,
            parent,
            crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::Grid),
            available,
        ),
    )?;
    Ok(
        lane_axis_size(flow_axes.logical_size(output.size), lane_axis)
            + lane_axis_margin_sum(margin, lane_axis),
    )
}

fn lane_child_intrinsic_available<S: LayoutScalar>(
    flow_axes: crate::geometry::FlowAxes,
    grid_axis: GridAxisKind,
    preferred_size: Size<ResolvedPreferredSize<S>>,
    grid_axis_available: AvailableOf<S>,
) -> Size<AvailableOf<S>> {
    let logical_size = flow_axes.logical_size(preferred_size);
    let logical_available = match grid_axis.logical_axis() {
        LogicalAxis::Inline => LogicalSizeOf::new(
            grid_axis_available,
            intrinsic_available_for_dimension(logical_size.block),
        ),
        LogicalAxis::Block => LogicalSizeOf::new(
            intrinsic_available_for_dimension(logical_size.inline),
            grid_axis_available,
        ),
    };
    flow_axes.physical_size(logical_available)
}

fn intrinsic_available_for_dimension<S: LayoutScalar>(
    dimension: ResolvedPreferredSize<S>,
) -> AvailableOf<S> {
    match dimension {
        ResolvedPreferredSize::MinContent => AvailableOf::MIN_CONTENT,
        ResolvedPreferredSize::MaxContent
        | ResolvedPreferredSize::Auto
        | ResolvedPreferredSize::Definite(_) => AvailableOf::MAX_CONTENT,
    }
}

fn definite_preferred_size<S: LayoutScalar>(dimension: ResolvedPreferredSize<S>) -> Option<S> {
    match dimension {
        ResolvedPreferredSize::Definite(value) => Some(value),
        ResolvedPreferredSize::Auto
        | ResolvedPreferredSize::MinContent
        | ResolvedPreferredSize::MaxContent => None,
    }
}

fn grid_lanes_child_sizing_preflight<S: LayoutScalar>(
    child_style: &NodeInputOf<S>,
    parent: Size<Option<S>>,
) -> Result<Size<ResolvedPreferredSize<S>>, SizingResolutionError<S>> {
    let preferred_size = Size::new(
        resolve_preferred_sizing(
            &child_style.size.width,
            SizingAlgorithm::GridLanes,
            PhysicalAxis::Horizontal,
            parent.width,
            true,
        )?,
        resolve_preferred_sizing(
            &child_style.size.height,
            SizingAlgorithm::GridLanes,
            PhysicalAxis::Vertical,
            parent.height,
            true,
        )?,
    );
    resolve_minimum_optional(
        &child_style.min_size.width,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Horizontal,
        parent.width,
        true,
    )?;
    resolve_minimum_optional(
        &child_style.min_size.height,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Vertical,
        parent.height,
        true,
    )?;
    resolve_maximum_optional(
        &child_style.max_size.width,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Horizontal,
        parent.width,
        true,
    )?;
    resolve_maximum_optional(
        &child_style.max_size.height,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Vertical,
        parent.height,
        true,
    )?;
    Ok(preferred_size)
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

fn resolve_tolerance<S: LayoutScalar>(
    tolerance: GridFlowToleranceOf<S>,
    basis: S,
) -> Result<S, LanePlacementError> {
    let Ok(basis) = PercentageBasisOf::definite(basis) else {
        return Err(LanePlacementError::InvalidGridFlowToleranceBasis);
    };
    let basis_value = basis
        .definite_value()
        .expect("validated definite basis")
        .get();

    match tolerance {
        GridFlowToleranceOf::Normal { font_size } => finite_tolerance(font_size),
        GridFlowToleranceOf::Length(length) => {
            resolve_length_tolerance(length.resolve_against(basis))
        }
        GridFlowToleranceOf::Percent(factor) => finite_tolerance(factor * basis_value),
        GridFlowToleranceOf::Infinite => Ok(S::INFINITY),
    }
}

fn resolve_length_tolerance<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<S, LanePlacementError> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution
            .value
            .ok_or(LanePlacementError::InvalidGridFlowToleranceResolution)
            .and_then(finite_tolerance),
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => {
            Err(LanePlacementError::InvalidGridFlowToleranceResolution)
        }
    }
}

fn finite_tolerance<S: LayoutScalar>(value: S) -> Result<S, LanePlacementError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(LanePlacementError::InvalidGridFlowToleranceResolution)
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
