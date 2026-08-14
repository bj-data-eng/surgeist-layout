use super::*;
use crate::error::{layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::{LogicalAxis, LogicalPointOf, LogicalSizeOf, PhysicalAxis};
use crate::scroll::UsedOverflow;
use crate::sizing::resolve::{
    ResolvedPreferredSize, SizingResolutionError, resolve_maximum_optional,
    resolve_minimum_optional, resolve_preferred_sizing,
};
use crate::{
    GridFlowToleranceOf, LayoutErrorSiteOf, LengthResolutionOf, LengthResolutionStatus,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridLanesItemContainingBlockOf<S: LayoutScalar = DefaultScalar> {
    grid_axis: GridAxisKind,
    logical_size: LogicalSizeOf<Option<S>>,
    physical_size: Size<Option<S>>,
}

impl<S: LayoutScalar> GridLanesItemContainingBlockOf<S> {
    pub(super) fn new(
        flow_axes: crate::geometry::FlowAxes,
        grid_axis: GridAxisKind,
        grid_axis_size: S,
        container_content_box_size: LogicalSizeOf<Option<S>>,
    ) -> Self {
        let logical_size = match grid_axis.logical_axis() {
            LogicalAxis::Inline => {
                LogicalSizeOf::new(Some(grid_axis_size), container_content_box_size.block)
            }
            LogicalAxis::Block => {
                LogicalSizeOf::new(container_content_box_size.inline, Some(grid_axis_size))
            }
        };
        Self {
            grid_axis,
            logical_size,
            physical_size: flow_axes.physical_size(logical_size),
        }
    }

    const fn grid_axis(self) -> GridAxisKind {
        self.grid_axis
    }

    const fn logical_size(self) -> LogicalSizeOf<Option<S>> {
        self.logical_size
    }

    const fn physical_size(self) -> Size<Option<S>> {
        self.physical_size
    }

    fn definite_logical_size(self) -> LogicalSizeOf<S> {
        LogicalSizeOf::new(
            self.logical_size
                .inline
                .expect("final grid-lanes inline containing extent is definite"),
            self.logical_size
                .block
                .expect("final grid-lanes block containing extent is definite"),
        )
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LaneIntrinsicBaselineRole {
    None,
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct LaneIntrinsicEdgeFactsOf<S: LayoutScalar = Scalar> {
    pub(super) start_mbp: S,
    pub(super) end_mbp: S,
    pub(super) start_half_gap: S,
    pub(super) end_half_gap: S,
}

#[derive(Clone, Debug, PartialEq)]
struct LaneIntrinsicEquivalenceKeyOf<S: LayoutScalar = Scalar> {
    span: usize,
    candidate_starts: Vec<usize>,
    baseline_role: LaneIntrinsicBaselineRole,
    edges: LaneIntrinsicEdgeFactsOf<S>,
    contribution_kind: IntrinsicSpanContribution,
}

#[derive(Clone, Debug)]
pub(super) struct ProjectedLaneIntrinsicItemOf<S: LayoutScalar = Scalar> {
    pub(super) id: &'static str,
    pub(super) kind: LaneIntrinsicItemKind,
    pub(super) candidate_starts: Option<Vec<usize>>,
    pub(super) contribution: LaneContributionFactsOf<S>,
    pub(super) baseline_role: LaneIntrinsicBaselineRole,
    pub(super) edges: LaneIntrinsicEdgeFactsOf<S>,
    pub(super) contribution_kind: IntrinsicSpanContribution,
}

#[derive(Clone, Debug)]
struct ProjectedLaneIntrinsicGroupOf<S: LayoutScalar = Scalar> {
    key: LaneIntrinsicEquivalenceKeyOf<S>,
    max_min_content: S,
    max_max_content: S,
    max_minimum: S,
    item_ids: Vec<&'static str>,
}

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

pub(super) fn lane_axis_for_grid_lanes<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
) -> GridAxisKind {
    let has_columns = !style.grid_template_columns.is_empty();
    let has_rows = !style.grid_template_rows.is_empty();
    match (has_columns, has_rows) {
        (false, true) => GridAxisKind::Column,
        (true, false) => GridAxisKind::Row,
        _ => lane_axis(style.grid_auto_flow),
    }
}

pub(super) fn grid_axis_for_grid_lanes<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
) -> GridAxisKind {
    match lane_axis_for_grid_lanes(style) {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    }
}

pub(super) fn column_flow_for_grid_lanes<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
) -> bool {
    grid_axis_for_grid_lanes(style) == GridAxisKind::Row
}

pub(super) fn apply_grid_lanes_auto_fit_policy<Node, S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    topology: &mut ExpandedGridTopology<S>,
    placements: &GridPlacementContext<Node, S>,
    track_count: usize,
    explicit_start: usize,
) -> Result<(), GridPlacementDemandError> {
    let axis = grid_axis_for_grid_lanes(style);
    let lines = GridAxisLines {
        explicit_start,
        explicit_count: match axis {
            GridAxisKind::Column => topology.explicit_columns,
            GridAxisKind::Row => topology.explicit_rows,
        },
    };
    let lanes_placements = placements
        .items
        .iter()
        .filter(|placement| placement.in_flow)
        .map(|placement| {
            let placement = match axis {
                GridAxisKind::Column => placement.column,
                GridAxisKind::Row => placement.row,
            };
            let (definite_start, span) = lane_grid_axis_facts(placement, track_count, lines);
            super::topology::LanesAutoFitPlacement {
                definite_start: definite_start.map(|line| line - 1),
                span,
            }
        })
        .collect::<Vec<_>>();
    topology.apply_lanes_auto_fit_policy(axis, track_count, explicit_start, &lanes_placements)
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
    let collapsed = vec![false; input.grid_axis_tracks];

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
                    infinite_candidate_start(cursor, span, &collapsed)
                } else {
                    finite_candidate_start(&running, cursor, span, tolerance, &collapsed)
                };
                let Some(start_zero) = start_zero else {
                    return Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: 1,
                        span,
                        tracks: input.grid_axis_tracks,
                    });
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
    let projected_items = input
        .items
        .iter()
        .map(|item| ProjectedLaneIntrinsicItemOf {
            id: item.id(),
            kind: item.kind(),
            candidate_starts: None,
            contribution: item.contribution(),
            baseline_role: LaneIntrinsicBaselineRole::None,
            edges: LaneIntrinsicEdgeFactsOf::default(),
            contribution_kind: IntrinsicSpanContribution::MinContent {
                prioritize_min_tracks: false,
            },
        })
        .collect::<Vec<_>>();
    lane_intrinsic_sizing_projected_with(&input, &projected_items, None, site)
}

pub(super) fn lane_intrinsic_sizing_projected_with<Node, S, M>(
    input: &LaneIntrinsicSizingInputOf<S>,
    projected_items: &[ProjectedLaneIntrinsicItemOf<S>],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
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
    let mut definite_virtual_items = Vec::new();
    let mut projected_groups: Vec<ProjectedLaneIntrinsicGroupOf<S>> = Vec::new();
    let collapsed = gutters
        .map(OrdinaryGridAxisGuttersOf::collapsed)
        .unwrap_or(&[]);

    for item in projected_items {
        match item.kind {
            LaneIntrinsicItemKind::Definite { span } => {
                if span.len().is_none() || span.end > input.tracks.len() + 1 {
                    return Ok(Err(LanePlacementError::DefiniteLaneSpanOutOfRange {
                        span,
                        tracks: input.tracks.len(),
                    }));
                }
                let definite = DefiniteLaneIntrinsicItemOf {
                    id: item.id,
                    span,
                    contribution: item.contribution,
                };
                definite_items.push(definite);
                definite_virtual_items.push((definite, item.edges, item.contribution_kind));
            }
            LaneIntrinsicItemKind::Indefinite { span } => {
                let span = span.get().min(input.tracks.len());
                let contributions = item.contribution.contributions();
                let active_starts = active_candidate_starts(input.tracks.len(), span, collapsed);
                let mut candidate_starts =
                    item.candidate_starts
                        .as_ref()
                        .map_or(active_starts.clone(), |candidates| {
                            candidates
                                .iter()
                                .copied()
                                .filter(|candidate| active_starts.contains(candidate))
                                .collect()
                        });
                candidate_starts.sort_unstable();
                candidate_starts.dedup();
                let key = LaneIntrinsicEquivalenceKeyOf {
                    span,
                    candidate_starts,
                    baseline_role: item.baseline_role,
                    edges: item.edges,
                    contribution_kind: item.contribution_kind,
                };
                if let Some(group) = projected_groups.iter_mut().find(|group| group.key == key) {
                    group.max_min_content = group.max_min_content.max(contributions.min_content);
                    group.max_max_content = group.max_max_content.max(contributions.max_content);
                    group.max_minimum = group.max_minimum.max(contributions.minimum);
                    group.item_ids.push(item.id);
                } else {
                    projected_groups.push(ProjectedLaneIntrinsicGroupOf {
                        key,
                        max_min_content: contributions.min_content,
                        max_max_content: contributions.max_content,
                        max_minimum: contributions.minimum,
                        item_ids: vec![item.id],
                    });
                }
            }
        }
    }

    let indefinite_groups = projected_groups
        .iter()
        .map(|group| IndefiniteLaneContributionGroupOf {
            span: group.key.span,
            max_min_content: group.max_min_content,
            max_max_content: group.max_max_content,
            max_min_size: group.max_minimum,
            item_ids: group.item_ids.clone(),
        })
        .collect::<Vec<_>>();
    let mut converted_indefinite_items = Vec::new();
    let mut virtual_items = definite_virtual_items;
    for group in &projected_groups {
        for start_index in &group.key.candidate_starts {
            let span = LaneTrackSpan::new(*start_index + 1, *start_index + 1 + group.key.span);
            let contribution = LaneContributionFactsOf {
                min_content: group.max_min_content,
                max_content: group.max_max_content,
                min_size: group.max_minimum,
                automatic_minimum_applies: false,
            };
            let item = DefiniteLaneIntrinsicItemOf {
                id: "indefinite-group",
                span,
                contribution,
            };
            converted_indefinite_items.push(item);
            virtual_items.push((item, group.key.edges, group.key.contribution_kind));
        }
    }

    let mut minimum_sizes = vec![S::ZERO; input.tracks.len()];
    let mut min_content_sizes = vec![S::ZERO; input.tracks.len()];
    let mut max_content_sizes = vec![S::ZERO; input.tracks.len()];
    for (item, _edges, contribution_kind) in virtual_items {
        if !span_overlaps_content_tracks(item.span, &input.content_sized_tracks) {
            continue;
        }
        let start = item.span.start - 1;
        let end = item.span.end - 1;
        let contributions = item.contribution.contributions();
        for (sizes, kind, contribution) in [
            (&mut minimum_sizes, contribution_kind, contributions.minimum),
            (
                &mut min_content_sizes,
                IntrinsicSpanContribution::MinContent {
                    prioritize_min_tracks: matches!(
                        contribution_kind,
                        IntrinsicSpanContribution::MinContent {
                            prioritize_min_tracks: true
                        }
                    ),
                },
                contributions.min_content,
            ),
            (
                &mut max_content_sizes,
                IntrinsicSpanContribution::MaxContent,
                contributions.max_content,
            ),
        ] {
            apply_ordinary_intrinsic_contribution(
                sizes,
                OrdinaryIntrinsicContributionInput {
                    tracks: &input.tracks,
                    start,
                    end,
                    kind,
                    percent_basis: input.available,
                    contribution,
                    gap: input.gap,
                    gutters,
                },
            );
        }
    }

    let initialized_track_sizes = input
        .tracks
        .iter()
        .map(|track| initialized_track_base(track.clone(), input.available, site))
        .collect::<LayoutResultOf<Node, Vec<_>, S, M>>()?;
    let final_track_sizes = input
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let contribution = match track.min {
                MinTrackSizingOf::MinContent => min_content_sizes[index],
                MinTrackSizingOf::MaxContent => max_content_sizes[index],
                MinTrackSizingOf::Auto => minimum_sizes[index],
                MinTrackSizingOf::Calculation(_) => S::ZERO,
            };
            initialized_track_sizes[index].max(contribution)
        })
        .collect();

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

fn active_candidate_starts(track_count: usize, span: usize, collapsed: &[bool]) -> Vec<usize> {
    candidate_starts(track_count, span)
        .filter(|start| {
            collapsed
                .get(*start..*start + span)
                .is_none_or(|candidate| candidate.iter().all(|collapsed| !collapsed))
        })
        .collect()
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
            crate::error::value_resolution_error_at_site(site, resolution.status()),
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
    style: &GridContainerProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    context: GridContainerContext<Tree::Scalar>,
    columns: &[Tree::Scalar],
    rows: &[Tree::Scalar],
    placements: &GridPlacementContext<<Tree as Traverse>::Node, Tree::Scalar>,
    grid_axis_gap: Tree::Scalar,
    container_content_box_size: LogicalSizeOf<Option<Tree::Scalar>>,
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
    let grid_axis_gutters = match grid_axis {
        GridAxisKind::Column => &context.column_gutters,
        GridAxisKind::Row => &context.row_gutters,
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
    let collapsed = grid_axis_gutters.collapsed();
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
        let child_style = placements.item_input(source_slot);
        if !is_in_flow_grid_child(child_style) {
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
                    infinite_candidate_start(cursor, span, collapsed)
                } else {
                    finite_candidate_start(&running, cursor, span, tolerance, collapsed)
                };
                let Some(start) = start else {
                    return Ok(Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: 1,
                        span,
                        tracks: grid_axis_tracks.len(),
                    }));
                };
                (start, span)
            }
        };
        let end = start + span;
        let grid_axis_size = if start < end {
            track_span_sum_with_gutters(
                grid_axis_tracks,
                start,
                end,
                grid_axis_gap,
                Some(grid_axis_gutters),
            )
        } else {
            Tree::Scalar::ZERO
        };
        let containing_block = GridLanesItemContainingBlockOf::new(
            constants.flow_axes,
            grid_axis,
            grid_axis_size,
            container_content_box_size,
        );
        let lane_axis_margin_box = measure_lane_axis_margin_box_with_grid_axis(
            tree,
            child,
            LaneAxisMarginBoxMeasureInput {
                child_style,
                container_style: style,
                constants,
                lane_axis,
                containing_block,
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
    pub(super) style: &'a GridContainerProjection<'a, S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) container_content_box_size: LogicalSizeOf<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) context: GridContainerContext<S>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<Node, S>,
    pub(super) containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
}

#[derive(Clone, Copy)]
pub(super) struct LaneIntrinsicTrackSizeInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) axis: GridAxisKind,
    pub(super) tracks: &'a [TrackSizingOf<S>],
    pub(super) gap: S,
    pub(super) available: AvailableOf<S>,
    pub(super) available_basis: Option<S>,
    pub(super) gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) lines: GridLines,
    pub(super) placements: &'a GridPlacementContext<Node, S>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
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
        container_style,
        constants,
        axis,
        tracks,
        gap,
        available,
        available_basis,
        gutters,
        lines,
        placements,
        subgrid_report,
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
        let child_style = placements.item_input(source_slot);
        if !is_in_flow_grid_child(child_style) {
            continue;
        }
        if scroll_container_auto_minimum_zero(child_style, constants.flow_axes, axis) {
            continue;
        }
        let placement = match axis {
            GridAxisKind::Column => placement.column,
            GridAxisKind::Row => placement.row,
        };
        let (definite_grid_axis_start, grid_axis_span) =
            lane_grid_axis_facts(placement, tracks.len(), grid_axis_lines(lines, axis));
        let item_report = subgrid_report
            .items
            .get(source_slot)
            .copied()
            .expect("grid-lanes subgrid report must preserve one item per child");
        if let Some(child_axis) = inherited_subgrid_axis_for_parent_axis(item_report, axis) {
            let axis_report = match child_axis {
                GridAxisKind::Column => item_report.column,
                GridAxisKind::Row => item_report.row,
            };
            let reversed = axis_report.mapping.reversed;
            let wrapper_span = grid_axis_span.max(1).min(tracks.len());
            let collapsed = gutters
                .map(OrdinaryGridAxisGuttersOf::collapsed)
                .unwrap_or(&[]);
            let wrapper_starts = definite_grid_axis_start.map_or_else(
                || active_candidate_starts(tracks.len(), wrapper_span, collapsed),
                |start| vec![start - 1],
            );
            let wrapper_edges = lane_child_edge_facts(
                tree,
                child,
                child_style,
                constants.flow_axes,
                constants.node_inner_size,
                axis,
            )?;
            collect_nested_lane_intrinsic_items(
                tree,
                child,
                placements.child_input(source_slot),
                constants,
                NestedLaneIntrinsicProjectionOf {
                    root_track_count: tracks.len(),
                    axis: child_axis,
                    wrapper_span,
                    wrapper_starts,
                    reversed,
                    parent_gap: gap,
                    accumulated_edges: LaneIntrinsicEdgeFactsOf::default(),
                    wrapper_edges,
                },
                available,
                &mut items,
            )?;
            continue;
        }
        let child_facts = lane_child_contribution_facts(
            tree,
            child,
            child_style,
            LaneChildIntrinsicMeasurementContextOf {
                container_style,
                constants,
                containing_flow_axes: constants.flow_axes,
                axis,
                available,
            },
        )?;
        let item = if let Some(start) = definite_grid_axis_start {
            match LaneIntrinsicItemOf::definite(
                "definite-item",
                LaneTrackSpan::new(start, start + grid_axis_span),
                child_facts.contribution,
            ) {
                Ok(item) => item,
                Err(error) => return Ok(Err(error)),
            }
        } else {
            LaneIntrinsicItemOf::indefinite(
                "indefinite-item",
                LaneTrackSpanLength::new(grid_axis_span)
                    .unwrap_or_else(|| LaneTrackSpanLength::new(1).expect("one is nonzero")),
                child_facts.contribution,
            )
        };
        items.push(ProjectedLaneIntrinsicItemOf {
            id: item.id(),
            kind: item.kind(),
            candidate_starts: None,
            contribution: item.contribution(),
            baseline_role: child_facts.baseline_role,
            edges: child_facts.edges,
            contribution_kind: child_facts.contribution_kind,
        });
    }

    let sizing_input = LaneIntrinsicSizingInputOf {
        axis,
        available: available_basis,
        gap,
        tracks: tracks.to_vec(),
        content_sized_tracks,
        items: Vec::new(),
    };
    lane_intrinsic_sizing_projected_with(
        &sizing_input,
        &items,
        gutters,
        LayoutErrorSiteOf::Node(node),
    )
    .map(|result| result.map(|report| report.final_track_sizes))
}

#[derive(Clone, Copy)]
struct LaneChildIntrinsicFactsOf<S: LayoutScalar = Scalar> {
    contribution: LaneContributionFactsOf<S>,
    baseline_role: LaneIntrinsicBaselineRole,
    edges: LaneIntrinsicEdgeFactsOf<S>,
    contribution_kind: IntrinsicSpanContribution,
}

#[derive(Clone)]
pub(super) struct NestedLaneIntrinsicProjectionOf<S: LayoutScalar = Scalar> {
    pub(super) root_track_count: usize,
    pub(super) axis: GridAxisKind,
    pub(super) wrapper_span: usize,
    pub(super) wrapper_starts: Vec<usize>,
    pub(super) reversed: bool,
    pub(super) parent_gap: S,
    pub(super) accumulated_edges: LaneIntrinsicEdgeFactsOf<S>,
    pub(super) wrapper_edges: LaneIntrinsicEdgeFactsOf<S>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NestedLaneCandidateGroupOf<S: LayoutScalar = Scalar> {
    pub(super) starts: Vec<usize>,
    pub(super) edges: LaneIntrinsicEdgeFactsOf<S>,
}

fn add_lane_intrinsic_edges<S: LayoutScalar>(
    left: LaneIntrinsicEdgeFactsOf<S>,
    right: LaneIntrinsicEdgeFactsOf<S>,
) -> LaneIntrinsicEdgeFactsOf<S> {
    LaneIntrinsicEdgeFactsOf {
        start_mbp: left.start_mbp + right.start_mbp,
        end_mbp: left.end_mbp + right.end_mbp,
        start_half_gap: left.start_half_gap + right.start_half_gap,
        end_half_gap: left.end_half_gap + right.end_half_gap,
    }
}

fn translated_nested_lane_start(
    wrapper_start: usize,
    wrapper_span: usize,
    local_start: usize,
    child_span: usize,
    reversed: bool,
) -> usize {
    if reversed {
        wrapper_start + wrapper_span - local_start - child_span
    } else {
        wrapper_start + local_start
    }
}

pub(super) fn nested_lane_candidate_groups<S: LayoutScalar>(
    projection: &NestedLaneIntrinsicProjectionOf<S>,
    child_span: usize,
    local_starts: impl IntoIterator<Item = usize>,
) -> Vec<NestedLaneCandidateGroupOf<S>> {
    let mut groups: Vec<NestedLaneCandidateGroupOf<S>> = Vec::new();
    for local_start in local_starts {
        let touches_local_start = local_start == 0;
        let touches_local_end = local_start + child_span == projection.wrapper_span;
        for wrapper_start in &projection.wrapper_starts {
            let mut boundary_edges = LaneIntrinsicEdgeFactsOf::default();
            if touches_local_start {
                if projection.reversed {
                    boundary_edges.end_mbp = projection.wrapper_edges.start_mbp;
                    boundary_edges.end_half_gap = projection.wrapper_edges.start_half_gap;
                } else {
                    boundary_edges.start_mbp = projection.wrapper_edges.start_mbp;
                    boundary_edges.start_half_gap = projection.wrapper_edges.start_half_gap;
                }
            }
            if touches_local_end {
                if projection.reversed {
                    boundary_edges.start_mbp = projection.wrapper_edges.end_mbp;
                    boundary_edges.start_half_gap = projection.wrapper_edges.end_half_gap;
                } else {
                    boundary_edges.end_mbp = projection.wrapper_edges.end_mbp;
                    boundary_edges.end_half_gap = projection.wrapper_edges.end_half_gap;
                }
            }
            let edges = add_lane_intrinsic_edges(projection.accumulated_edges, boundary_edges);
            let start = translated_nested_lane_start(
                *wrapper_start,
                projection.wrapper_span,
                local_start,
                child_span,
                projection.reversed,
            );
            if start + child_span > projection.root_track_count {
                continue;
            }
            if let Some(group) = groups.iter_mut().find(|group| group.edges == edges) {
                group.starts.push(start);
            } else {
                groups.push(NestedLaneCandidateGroupOf {
                    starts: vec![start],
                    edges,
                });
            }
        }
    }
    for group in &mut groups {
        group.starts.sort_unstable();
        group.starts.dedup();
    }
    groups
}

fn collect_nested_lane_intrinsic_items<Tree, M>(
    tree: &mut Tree,
    wrapper: <Tree as Traverse>::Node,
    wrapper_input: &GridChildInput<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    mut projection: NestedLaneIntrinsicProjectionOf<Tree::Scalar>,
    available: AvailableOf<Tree::Scalar>,
    items: &mut Vec<ProjectedLaneIntrinsicItemOf<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    if projection.wrapper_starts.is_empty() || projection.wrapper_span == 0 {
        return Ok(());
    }
    let wrapper_container = wrapper_input
        .nested_container()
        .expect("a nested lanes wrapper must retain its container projection input")
        .projection();
    let wrapper_flow_axes = wrapper_container.common.flow_axes;
    let logical_gap = wrapper_flow_axes.logical_size(*wrapper_container.gap);
    let gap_value = match projection.axis {
        GridAxisKind::Column => logical_gap.inline,
        GridAxisKind::Row => logical_gap.block,
    };
    let logical_containing_size = wrapper_flow_axes.logical_size(constants.node_inner_size);
    let gap_basis = match projection.axis {
        GridAxisKind::Column => logical_containing_size.inline,
        GridAxisKind::Row => logical_containing_size.block,
    };
    let wrapper_gap = match gap_value {
        LengthOf::Normal => projection.parent_gap,
        value => resolve_length_or_zero(value, gap_basis)
            .map_err(|status| crate::error::value_resolution_error(wrapper, status))?,
    };
    let half_gap_difference = (wrapper_gap - projection.parent_gap) / Tree::Scalar::from_f64(2.0);
    projection.wrapper_edges.start_half_gap = half_gap_difference;
    projection.wrapper_edges.end_half_gap = half_gap_difference;

    let children = tree.children(wrapper).collect::<Vec<_>>();
    let child_inputs = children
        .iter()
        .copied()
        .map(|child| input::project_grid_child_input!(tree, child))
        .collect::<Vec<_>>();
    let order = order_modified_indexes(
        &child_inputs
            .iter()
            .enumerate()
            .filter_map(|(index, input)| {
                let style = input.item();
                is_in_flow_grid_child(style)
                    .then_some((style.item_order, crate::SourceIndex::new(index)))
            })
            .collect::<Vec<_>>(),
    );
    for source_index in order {
        let child = children[source_index.get()];
        let child_input = &child_inputs[source_index.get()];
        let child_style = child_input.item();
        if !is_in_flow_grid_child(child_style)
            || scroll_container_auto_minimum_zero(child_style, wrapper_flow_axes, projection.axis)
        {
            continue;
        }
        let placement = match projection.axis {
            GridAxisKind::Column => child_style.grid_column,
            GridAxisKind::Row => child_style.grid_row,
        };
        let (definite_start, child_span) = lane_grid_axis_facts(
            placement,
            projection.wrapper_span,
            GridAxisLines {
                explicit_start: 0,
                explicit_count: projection.wrapper_span,
            },
        );
        let child_span = child_span.max(1).min(projection.wrapper_span);
        let local_starts = definite_start.map_or_else(
            || candidate_starts(projection.wrapper_span, child_span).collect::<Vec<_>>(),
            |start| vec![start - 1],
        );
        let candidate_groups = nested_lane_candidate_groups(&projection, child_span, local_starts);
        if candidate_groups.is_empty() {
            continue;
        }

        let item_report = SubgridItemReport {
            node: child,
            column: subgrid_axis_report(&wrapper_container, child_input, GridAxisKind::Column),
            row: subgrid_axis_report(&wrapper_container, child_input, GridAxisKind::Row),
        };
        if let Some(child_axis) =
            inherited_subgrid_axis_for_parent_axis(item_report, projection.axis)
        {
            let axis_report = match child_axis {
                GridAxisKind::Column => item_report.column,
                GridAxisKind::Row => item_report.row,
            };
            let mapping_reversed = axis_report.mapping.reversed;
            let wrapper_edges = lane_child_edge_facts(
                tree,
                child,
                child_style,
                wrapper_flow_axes,
                constants.node_inner_size,
                projection.axis,
            )?;
            for group in candidate_groups {
                collect_nested_lane_intrinsic_items(
                    tree,
                    child,
                    child_input,
                    constants,
                    NestedLaneIntrinsicProjectionOf {
                        root_track_count: projection.root_track_count,
                        axis: child_axis,
                        wrapper_span: child_span,
                        wrapper_starts: group.starts,
                        reversed: projection.reversed ^ mapping_reversed,
                        parent_gap: wrapper_gap,
                        accumulated_edges: group.edges,
                        wrapper_edges,
                    },
                    available,
                    items,
                )?;
            }
            continue;
        }

        let child_facts = lane_child_contribution_facts(
            tree,
            child,
            child_style,
            LaneChildIntrinsicMeasurementContextOf {
                container_style: &wrapper_container,
                constants,
                containing_flow_axes: wrapper_flow_axes,
                axis: projection.axis,
                available,
            },
        )?;
        for group in candidate_groups {
            let adjustment = group.edges.start_mbp
                + group.edges.end_mbp
                + group.edges.start_half_gap
                + group.edges.end_half_gap;
            let contribution = LaneContributionFactsOf {
                min_content: (child_facts.contribution.min_content + adjustment)
                    .max(Tree::Scalar::ZERO),
                max_content: (child_facts.contribution.max_content + adjustment)
                    .max(Tree::Scalar::ZERO),
                min_size: (child_facts.contribution.min_size + adjustment).max(Tree::Scalar::ZERO),
                automatic_minimum_applies: child_facts.contribution.automatic_minimum_applies,
            };
            items.push(ProjectedLaneIntrinsicItemOf {
                id: "nested-descendant",
                kind: LaneIntrinsicItemKind::Indefinite {
                    span: LaneTrackSpanLength::new(child_span)
                        .expect("nested descendant spans are nonzero"),
                },
                candidate_starts: Some(group.starts),
                contribution,
                baseline_role: child_facts.baseline_role,
                edges: add_lane_intrinsic_edges(group.edges, child_facts.edges),
                contribution_kind: child_facts.contribution_kind,
            });
        }
    }
    Ok(())
}

fn lane_child_edge_facts<Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    child_style: &GridItemProjection<Tree::Scalar>,
    containing_flow_axes: crate::geometry::FlowAxes,
    containing_size: Size<Option<Tree::Scalar>>,
    axis: GridAxisKind,
) -> LayoutResultOf<<Tree as Traverse>::Node, LaneIntrinsicEdgeFactsOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let margin = intrinsic_contribution_margin(child_style, containing_flow_axes, containing_size)
        .map_err(|status| crate::error::value_resolution_error(child, status))?;
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.padding,
            containing_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.border,
            containing_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let logical_mbp = containing_flow_axes.logical_edges(margin + padding + border);
    let (start_mbp, end_mbp) = match axis.logical_axis() {
        LogicalAxis::Inline => (logical_mbp.inline_start, logical_mbp.inline_end),
        LogicalAxis::Block => (logical_mbp.block_start, logical_mbp.block_end),
    };
    Ok(LaneIntrinsicEdgeFactsOf {
        start_mbp,
        end_mbp,
        start_half_gap: Tree::Scalar::ZERO,
        end_half_gap: Tree::Scalar::ZERO,
    })
}

#[derive(Clone, Copy)]
struct LaneChildIntrinsicMeasurementContextOf<'a, S: LayoutScalar = Scalar> {
    container_style: &'a GridContainerProjection<'a, S>,
    constants: &'a Constants<S>,
    containing_flow_axes: crate::geometry::FlowAxes,
    axis: GridAxisKind,
    available: AvailableOf<S>,
}

fn lane_child_contribution_facts<Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    child_style: &GridItemProjection<Tree::Scalar>,
    context: LaneChildIntrinsicMeasurementContextOf<'_, Tree::Scalar>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    LaneChildIntrinsicFactsOf<Tree::Scalar>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let LaneChildIntrinsicMeasurementContextOf {
        container_style,
        constants,
        containing_flow_axes,
        axis,
        available,
    } = context;
    let preferred_size = grid_lanes_child_sizing_preflight(
        child_style,
        Size::new(
            constants.node_inner_size.width,
            constants.node_inner_size.height,
        ),
    )
    .map_err(|error| sizing_resolution_error(child, error))?;
    let min_available =
        lane_child_intrinsic_available(containing_flow_axes, axis, preferred_size, available);
    let max_available = lane_child_intrinsic_available(
        containing_flow_axes,
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
                containing_flow_axes,
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
                containing_flow_axes,
                crate::ParentFormattingContext::Grid,
            ),
            max_available,
        ),
    )?;
    let margin =
        intrinsic_contribution_margin(child_style, containing_flow_axes, constants.node_inner_size)
            .map_err(|status| crate::error::value_resolution_error(child, status))?;
    let used_overflow = grid_axis_used_overflow(child_style, containing_flow_axes, axis);
    let min_output_size = lane_axis_size(containing_flow_axes.logical_size(min_output.size), axis);
    let min_content_size = lane_axis_size(
        containing_flow_axes.logical_size(min_output.content_size),
        axis,
    );
    let max_output_size = lane_axis_size(containing_flow_axes.logical_size(max_output.size), axis);
    let max_content_size = lane_axis_size(
        containing_flow_axes.logical_size(max_output.content_size),
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
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.padding,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.border,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let logical_margin = containing_flow_axes.logical_edges(margin);
    let logical_mbp = containing_flow_axes.logical_edges(margin + padding + border);
    let margin_sum = lane_axis_margin_sum(logical_margin, axis);
    let (start_mbp, end_mbp) = match axis.logical_axis() {
        LogicalAxis::Inline => (logical_mbp.inline_start, logical_mbp.inline_end),
        LogicalAxis::Block => (logical_mbp.block_start, logical_mbp.block_end),
    };
    let alignment = match axis {
        GridAxisKind::Column => child_style.justify_self.or(container_style.justify_items),
        GridAxisKind::Row => child_style.align_self.or(container_style.align_items),
    }
    .unwrap_or(AlignItems::Stretch);
    let baseline_role = match alignment {
        AlignItems::Baseline => LaneIntrinsicBaselineRole::First,
        AlignItems::LastBaseline => LaneIntrinsicBaselineRole::Last,
        _ => LaneIntrinsicBaselineRole::None,
    };
    let automatic_minimum = automatic_minimum_applies(child_style, containing_flow_axes, axis);
    Ok(LaneChildIntrinsicFactsOf {
        contribution: LaneContributionFactsOf {
            min_content: min_contribution + margin_sum,
            max_content: max_contribution + margin_sum,
            min_size: if automatic_minimum {
                min_contribution + margin_sum
            } else {
                Tree::Scalar::ZERO
            },
            automatic_minimum_applies: automatic_minimum,
        },
        baseline_role,
        edges: LaneIntrinsicEdgeFactsOf {
            start_mbp,
            end_mbp,
            start_half_gap: Tree::Scalar::ZERO,
            end_half_gap: Tree::Scalar::ZERO,
        },
        contribution_kind: IntrinsicSpanContribution::for_axis(available, used_overflow),
    })
}

fn automatic_minimum_applies<S: LayoutScalar>(
    style: &GridItemProjection<S>,
    flow_axes: crate::geometry::FlowAxes,
    axis: GridAxisKind,
) -> bool {
    !scroll_container_auto_minimum_zero(style, flow_axes, axis)
}

fn scroll_container_auto_minimum_zero<S: LayoutScalar>(
    style: &GridItemProjection<S>,
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
    let track_content_size = LogicalSizeOf::new(
        track_sum_with_gutters(columns, gap.inline, Some(&context.column_gutters)),
        track_sum_with_gutters(rows, gap.block, Some(&context.row_gutters)),
    );
    let content_box_size = flow_axes
        .logical_size(constants.node_inner_size)
        .unwrap_or(container_content_box_size);
    let alignment_free_space = content_box_size - track_content_size;
    let column_has_collapse = context
        .column_gutters
        .collapsed()
        .iter()
        .any(|value| *value);
    let row_has_collapse = context.row_gutters.collapsed().iter().any(|value| *value);
    let legacy_column_alignment = grid_alignment(
        alignment_free_space.inline,
        columns.len(),
        gap.inline,
        style.justify_content.unwrap_or(AlignContent::Stretch),
    );
    let legacy_row_alignment = grid_alignment(
        alignment_free_space.block,
        rows.len(),
        gap.block,
        style.align_content.unwrap_or(AlignContent::Stretch),
    );
    let column_alignment = if column_has_collapse {
        ordinary_grid_axis_alignment(
            alignment_free_space.inline,
            &context.column_gutters,
            style.justify_content.unwrap_or(AlignContent::Stretch),
        )
    } else {
        OrdinaryGridAxisAlignment {
            start: legacy_column_alignment.start,
            gutter_after: vec![legacy_column_alignment.gap; columns.len().saturating_sub(1)],
        }
    };
    let row_alignment = if row_has_collapse {
        ordinary_grid_axis_alignment(
            alignment_free_space.block,
            &context.row_gutters,
            style.align_content.unwrap_or(AlignContent::Stretch),
        )
    } else {
        OrdinaryGridAxisAlignment {
            start: legacy_row_alignment.start,
            gutter_after: vec![legacy_row_alignment.gap; rows.len().saturating_sub(1)],
        }
    };
    context.column_gutters = OrdinaryGridAxisGuttersOf::from_active_boundary_gutters(
        columns.len(),
        context.column_gutters.collapsed(),
        context.column_gutters.active_boundary_after(),
        &column_alignment.gutter_after,
    );
    context.row_gutters = OrdinaryGridAxisGuttersOf::from_active_boundary_gutters(
        rows.len(),
        context.row_gutters.collapsed(),
        context.row_gutters.active_boundary_after(),
        &row_alignment.gutter_after,
    );
    context.gap = gap;
    let grid_axis_gap = match grid_axis_for_grid_lanes(style) {
        GridAxisKind::Column => gap.inline,
        GridAxisKind::Row => gap.block,
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
        content_box_size.map(Some),
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
    let column_geometry =
        UsedGridAxisGeometryOf::from_sizing_gutters(columns.to_vec(), &context.column_gutters)
            .translated(logical_content_box_inset.inline_start + column_alignment.start);
    let row_geometry =
        UsedGridAxisGeometryOf::from_sizing_gutters(rows.to_vec(), &context.row_gutters)
            .translated(logical_content_box_inset.block_start + row_alignment.start);
    let column_offsets = if column_has_collapse {
        column_geometry.line_offsets().to_vec()
    } else {
        grid_axis_logical_offsets(
            columns,
            None,
            logical_content_box_inset.inline_start,
            legacy_column_alignment,
        )
    };
    let row_offsets = if row_has_collapse {
        row_geometry.line_offsets().to_vec()
    } else {
        grid_axis_logical_offsets(
            rows,
            None,
            logical_content_box_inset.block_start,
            legacy_row_alignment,
        )
    };
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
        let child_style = placements.item_input(source_index).clone();
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
            let absolute_context = if column_has_collapse || row_has_collapse {
                AbsoluteGridContext::ordinary_with_geometry(
                    OrdinaryAbsoluteGridGeometryContextInput {
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
                        column_geometry: &column_geometry,
                        row_geometry: &row_geometry,
                        lines: context.lines,
                    },
                )
            } else {
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
            };
            child_contributions.push(layout_absolute_grid_child(
                tree,
                child,
                source_index,
                &child_style,
                absolute_context
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
        let grid_axis_size = match lane_report.grid_axis {
            GridAxisKind::Column => track_span_sum_with_gutters(
                columns,
                start,
                end.min(columns.len()),
                gap.inline,
                Some(&context.column_gutters),
            ),
            GridAxisKind::Row => track_span_sum_with_gutters(
                rows,
                start,
                end.min(rows.len()),
                gap.block,
                Some(&context.row_gutters),
            ),
        };
        let containing_block = GridLanesItemContainingBlockOf::new(
            flow_axes,
            lane_report.grid_axis,
            grid_axis_size,
            content_box_size.map(Some),
        );
        let containing_logical_size = containing_block.definite_logical_size();
        let area = match lane_report.grid_axis {
            GridAxisKind::Column => GridArea {
                column: start,
                row: 0,
                column_end: end,
                row_end: 1,
                size: containing_logical_size,
            },
            GridAxisKind::Row => GridArea {
                column: 0,
                row: start,
                column_end: 1,
                row_end: end,
                size: containing_logical_size,
            },
        };

        let physical_area_size = containing_block
            .physical_size()
            .map(|extent| extent.expect("final grid-lanes containing extent is definite"));
        let mut item = grid_item_sizing_for_grid_flow::<Tree, M>(
            child,
            &child_style,
            style,
            subgrid_report.items.get(source_index).copied(),
            physical_area_size,
            physical_area_size.map(Some),
            flow_axes,
        )?;
        normalize_grid_lanes_stacking_axis_sizing(
            &child_style,
            style,
            lane_axis,
            flow_axes,
            item_offset.lane_axis_margin_box,
            &mut item,
        );
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
        let child_container_style = placements
            .nested_container_input(source_index)
            .map(GridContainerInput::projection);
        let child_context = subgrid_child_parent_context(SubgridChildParentContextInput {
            item: *subgrid_report
                .items
                .get(source_index)
                .expect("grid-lanes subgrid report must preserve one item per child"),
            child_style: &child_style,
            child_container_style,
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
        .map_err(|error| layout_child_geometry_error(node, child, error))?;
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
            style: child_style.clone(),
            nested_container: placements.nested_container_input(source_index).cloned(),
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
            block_offset: block_axis.offset,
            block_auto_margins,
            baseline_participation,
            margin,
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
        let has_single_template_axis =
            style.grid_template_columns.is_empty() != style.grid_template_rows.is_empty();
        let selected_inline_lane_offset = if has_single_template_axis {
            lane_axis_alignment_start
        } else {
            item_offset.map_or(lane_axis_alignment_start, |offset| {
                let start = offset.grid_axis_start - 1;
                grid_area_track_offset(&column_offsets, start, start + offset.grid_axis_span)
            })
        };
        let selected_block_track_offset = if has_single_template_axis {
            grid_area_track_offset(&row_offsets, item.area.row, item.area.row_end)
        } else {
            grid_area_track_offset(&row_offsets, 0, 1)
        };
        let logical_location = match lane_axis.logical_axis() {
            LogicalAxis::Inline => LogicalPointOf::new(
                selected_inline_lane_offset
                    + lane_offset
                    + item.horizontal_axis.offset
                    + item.logical_relative_offset.inline,
                selected_block_track_offset
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
            .map_err(|error| layout_child_geometry_error(node, node, error))?;
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
        .map_err(|error| layout_child_geometry_error(node, node, error))?;
    if style.justify_content.is_some() {
        contributions.set_active_alignment_subject(flow_axes.inline_axis(), track_subject);
    }
    if style.align_content.is_some() {
        contributions.set_active_alignment_subject(flow_axes.block_axis(), track_subject);
    }
    let visible_content_size = contributions
        .content_size_from_anchor(Point::ZERO)
        .map_err(|error| layout_child_geometry_error(node, node, error))?;

    Ok(GridChildrenLayout {
        visible_content_size,
        contributions,
        baselines: baselines.baselines,
        baseline_groups,
    })
}

fn normalize_grid_lanes_stacking_axis_sizing<S: LayoutScalar>(
    child_style: &GridItemProjection<S>,
    container_style: &GridContainerProjection<'_, S>,
    lane_axis: GridAxisKind,
    flow_axes: crate::geometry::FlowAxes,
    lane_axis_margin_box: S,
    item: &mut GridItemSizing<S>,
) {
    let logical_style_size = flow_axes.logical_size(child_style.size.clone());
    let mut logical_known = flow_axes.logical_size(item.known);
    let logical_margin = flow_axes.logical_edges(
        item.unresolved_margin
            .map(|margin| margin.unwrap_or(S::ZERO)),
    );
    match lane_axis.logical_axis() {
        LogicalAxis::Inline => {
            let alignment = resolve_grid_item_normal_alignment(
                child_style.justify_self,
                container_style.justify_items,
                child_style.item_is_replaced,
                logical_style_size.inline.is_auto(),
                AlignItems::Start,
            );
            if alignment != AlignItems::Stretch && logical_style_size.inline.is_auto() {
                logical_known.inline =
                    Some((lane_axis_margin_box - logical_margin.inline_sum()).max(S::ZERO));
            }
            item.justify_self = alignment;
        }
        LogicalAxis::Block => {
            let alignment = resolve_grid_item_normal_alignment(
                child_style.align_self,
                container_style.align_items,
                child_style.item_is_replaced,
                logical_style_size.block.is_auto(),
                AlignItems::Start,
            );
            if alignment != AlignItems::Stretch && logical_style_size.block.is_auto() {
                logical_known.block =
                    Some((lane_axis_margin_box - logical_margin.block_sum()).max(S::ZERO));
            }
            item.align_self = alignment;
        }
    }
    item.known = flow_axes.physical_size(logical_known);
}

#[derive(Clone, Copy)]
pub(super) struct LaneAxisMarginBoxMeasureInput<'a, S: LayoutScalar = Scalar> {
    pub(super) child_style: &'a GridItemProjection<S>,
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) lane_axis: GridAxisKind,
    pub(super) containing_block: GridLanesItemContainingBlockOf<S>,
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
        containing_block,
    } = input;
    let flow_axes = constants.flow_axes;
    let containing_physical_size = containing_block.physical_size();
    let preferred_size = grid_lanes_child_sizing_preflight(child_style, containing_physical_size)
        .map_err(|error| sizing_resolution_error(child, error))?;
    let logical_preferred_size = flow_axes.logical_size(preferred_size);
    let logical_container_preferred_size = flow_axes.logical_size(container_style.size.clone());
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
        let logical_parent = containing_block.logical_size();
        let mut logical_available =
            LogicalSizeOf::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT);
        match lane_axis.logical_axis() {
            LogicalAxis::Inline => {
                logical_available.inline = lane_axis_intrinsic_available(
                    logical_preferred_size.inline,
                    &logical_container_preferred_size.inline,
                );
            }
            LogicalAxis::Block => {
                logical_available.block = lane_axis_intrinsic_available(
                    logical_preferred_size.block,
                    &logical_container_preferred_size.block,
                );
            }
        }
        match containing_block.grid_axis().logical_axis() {
            LogicalAxis::Inline => {
                let grid_axis_size = logical_parent
                    .inline
                    .expect("grid-lanes grid-axis containing extent is definite");
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
                logical_available.inline = AvailableOf::Definite(available_inline);
            }
            LogicalAxis::Block => {
                let grid_axis_size = logical_parent
                    .block
                    .expect("grid-lanes grid-axis containing extent is definite");
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

fn lane_axis_intrinsic_available<S: LayoutScalar>(
    child: ResolvedPreferredSize<S>,
    container: &crate::PreferredSizeOf<S>,
) -> AvailableOf<S> {
    if matches!(child, ResolvedPreferredSize::Auto) && container.is_min_content() {
        AvailableOf::MIN_CONTENT
    } else {
        intrinsic_available_for_dimension(child)
    }
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
    child_style: &GridItemProjection<S>,
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

fn infinite_candidate_start(cursor: usize, span: usize, collapsed: &[bool]) -> Option<usize> {
    let max_start = collapsed.len().checked_add(1)?.checked_sub(span)?;
    let shifted_cursor = if cursor >= max_start { 0 } else { cursor };
    (0..max_start)
        .map(|offset| (shifted_cursor + offset) % max_start)
        .find(|start| lanes_candidate_is_retained(collapsed, *start, span))
}

fn lanes_candidate_is_retained(collapsed: &[bool], start: usize, span: usize) -> bool {
    collapsed
        .get(start..start.saturating_add(span))
        .is_some_and(|candidate| candidate.iter().all(|track| !*track))
}

fn finite_candidate_start<S: LayoutScalar>(
    running: &[S],
    cursor: usize,
    span: usize,
    tolerance: S,
    collapsed: &[bool],
) -> Option<usize> {
    let track_count = running.len();
    let max_start = track_count.checked_add(1)?.checked_sub(span)?;
    let shifted_cursor = if cursor >= max_start { 0 } else { cursor };
    let absolute_shortest = (0..max_start)
        .filter(|start| lanes_candidate_is_retained(collapsed, *start, span))
        .map(|start| max_running_position(running, start, span))
        .fold(S::INFINITY, S::min);

    for offset in 0..max_start {
        let start = (shifted_cursor + offset) % max_start;
        if lanes_candidate_is_retained(collapsed, start, span)
            && max_running_position(running, start, span) <= absolute_shortest + tolerance
        {
            return Some(start);
        }
    }
    None
}

fn max_running_position<S: LayoutScalar>(running: &[S], start: usize, span: usize) -> S {
    running[start..start + span]
        .iter()
        .copied()
        .fold(S::ZERO, S::max)
}
