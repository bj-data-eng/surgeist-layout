use super::*;
use crate::NoCalcResolver;

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementInput<Item> {
    pub grid_axis_tracks: usize,
    pub auto_flow: GridAutoFlow,
    pub lane_gap: Scalar,
    pub tolerance: GridFlowTolerance,
    pub tolerance_basis: Scalar,
    pub items: Vec<LaneItem<Item>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItem<Item> {
    pub item: Item,
    pub grid_axis_span: usize,
    pub definite_grid_axis_start: Option<usize>,
    pub lane_axis_margin_box: Scalar,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemOffset<Item> {
    pub item: Item,
    pub grid_axis_start: usize,
    pub grid_axis_span: usize,
    pub offset: Scalar,
    pub lane_axis_margin_box: Scalar,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementReport<Item> {
    pub lane_axis: GridAxisKind,
    pub grid_axis: GridAxisKind,
    pub item_offsets: Vec<LaneItemOffset<Item>>,
    pub running_positions_after_each_item: Vec<Vec<Scalar>>,
    pub content_size: Scalar,
    pub final_cursor: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanePlacementError {
    EmptyTrackList,
    SpanOutOfRange,
    NestedGridLanesSubgridIndefiniteUnsupported,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaneContributionFacts {
    pub min_content: Scalar,
    pub max_content: Scalar,
    pub min_size: Scalar,
    pub automatic_minimum_applies: bool,
}

impl LaneContributionFacts {
    fn contributions(self) -> LaneContributions {
        let minimum = if self.automatic_minimum_applies {
            self.min_content
        } else {
            self.min_size
        };
        LaneContributions {
            minimum,
            min_content: self.min_content,
            max_content: self.max_content,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LaneContributions {
    minimum: Scalar,
    min_content: Scalar,
    max_content: Scalar,
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

    fn checked_len(self) -> Result<usize, LanePlacementError> {
        if self.start == 0 || self.end <= self.start {
            return Err(LanePlacementError::SpanOutOfRange);
        }
        Ok(self.end - self.start)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingInput {
    pub axis: GridAxisKind,
    pub available: Option<Scalar>,
    pub gap: Scalar,
    pub tracks: Vec<TrackSizing>,
    pub content_sized_tracks: Vec<usize>,
    pub items: Vec<LaneIntrinsicItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicItem {
    pub id: &'static str,
    pub span: usize,
    pub definite_span: Option<LaneTrackSpan>,
    pub contribution: LaneContributionFacts,
    pub nested_indefinite_subgrid: bool,
}

impl LaneIntrinsicItem {
    #[must_use]
    pub const fn definite(
        id: &'static str,
        start: usize,
        end: usize,
        contribution: LaneContributionFacts,
    ) -> Self {
        Self {
            id,
            span: 0,
            definite_span: Some(LaneTrackSpan::new(start, end)),
            contribution,
            nested_indefinite_subgrid: false,
        }
    }

    #[must_use]
    pub const fn indefinite(
        id: &'static str,
        span: usize,
        contribution: LaneContributionFacts,
    ) -> Self {
        Self {
            id,
            span,
            definite_span: None,
            contribution,
            nested_indefinite_subgrid: false,
        }
    }

    #[must_use]
    pub const fn nested_indefinite_subgrid(
        id: &'static str,
        span: usize,
        contribution: LaneContributionFacts,
    ) -> Self {
        Self {
            id,
            span,
            definite_span: None,
            contribution,
            nested_indefinite_subgrid: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DefiniteLaneIntrinsicItem {
    pub id: &'static str,
    pub span: LaneTrackSpan,
    pub contribution: LaneContributionFacts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndefiniteLaneContributionGroup {
    pub span: usize,
    pub max_min_content: Scalar,
    pub max_max_content: Scalar,
    pub max_min_size: Scalar,
    pub item_ids: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingReport {
    pub definite_items: Vec<DefiniteLaneIntrinsicItem>,
    pub indefinite_groups: Vec<IndefiniteLaneContributionGroup>,
    pub converted_indefinite_items: Vec<DefiniteLaneIntrinsicItem>,
    pub final_track_sizes: Vec<Scalar>,
}

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

pub(super) fn lane_axis_for_grid_lanes(style: &NodeInput) -> GridAxisKind {
    let has_columns = !style.grid_template_columns.is_empty();
    let has_rows = !style.grid_template_rows.is_empty();
    match (has_columns, has_rows) {
        (false, true) => GridAxisKind::Column,
        (true, false) => GridAxisKind::Row,
        _ => lane_axis(style.grid_auto_flow),
    }
}

pub(super) fn grid_axis_for_grid_lanes(style: &NodeInput) -> GridAxisKind {
    match lane_axis_for_grid_lanes(style) {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    }
}

pub(super) fn column_flow_for_grid_lanes(style: &NodeInput) -> bool {
    grid_axis_for_grid_lanes(style) == GridAxisKind::Row
}

pub fn place_lanes<Item>(
    input: LanePlacementInput<Item>,
) -> Result<LanePlacementReport<Item>, LanePlacementError> {
    if input.grid_axis_tracks == 0 {
        return Err(LanePlacementError::EmptyTrackList);
    }

    let mut running = vec![0.0; input.grid_axis_tracks];
    let mut item_offsets = Vec::new();
    let mut running_positions_after_each_item = Vec::new();
    let mut cursor = 0usize;
    let tolerance = resolve_tolerance(input.tolerance, input.tolerance_basis);
    let mut content_size: Scalar = 0.0;

    for item in input.items {
        let (start_zero, span) = match item.definite_grid_axis_start {
            Some(start_line) => {
                if start_line == 0 || item.grid_axis_span == 0 {
                    return Err(LanePlacementError::SpanOutOfRange);
                }
                let start_zero = start_line - 1;
                if start_zero + item.grid_axis_span > input.grid_axis_tracks {
                    return Err(LanePlacementError::SpanOutOfRange);
                }
                (start_zero, item.grid_axis_span)
            }
            None => {
                let span = item.grid_axis_span.clamp(1, input.grid_axis_tracks);
                let start_zero = if matches!(input.tolerance, GridFlowTolerance::Infinite) {
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
            .fold(0.0, Scalar::max);
        let new_position = previous + item.lane_axis_margin_box + input.lane_gap;
        content_size = content_size.max(new_position - input.lane_gap);
        for position in &mut running[start_zero..start_zero + span] {
            *position = new_position;
        }

        item_offsets.push(LaneItemOffset {
            item: item.item,
            grid_axis_start: start_zero + 1,
            grid_axis_span: span,
            offset: previous,
            lane_axis_margin_box: item.lane_axis_margin_box,
        });
        running_positions_after_each_item.push(running.clone());
        cursor = (start_zero + span) % input.grid_axis_tracks;
    }

    Ok(LanePlacementReport {
        lane_axis: lane_axis(input.auto_flow),
        grid_axis: grid_axis_for_lanes(input.auto_flow),
        item_offsets,
        running_positions_after_each_item,
        content_size,
        final_cursor: cursor,
    })
}

pub fn lane_intrinsic_sizing(
    input: LaneIntrinsicSizingInput,
) -> Result<LaneIntrinsicSizingReport, LanePlacementError> {
    lane_intrinsic_sizing_with(input, &NoCalcResolver)
}

pub(super) fn lane_intrinsic_sizing_with(
    input: LaneIntrinsicSizingInput,
    resolver: &dyn CalcResolver,
) -> Result<LaneIntrinsicSizingReport, LanePlacementError> {
    if input.content_sized_tracks.is_empty() || input.tracks.is_empty() {
        return Err(LanePlacementError::EmptyTrackList);
    }
    if input
        .content_sized_tracks
        .iter()
        .any(|track_index| *track_index >= input.tracks.len())
    {
        return Err(LanePlacementError::SpanOutOfRange);
    }

    let mut definite_items = Vec::new();
    let mut indefinite_groups: Vec<IndefiniteLaneContributionGroup> = Vec::new();

    for item in &input.items {
        if item.nested_indefinite_subgrid {
            return Err(LanePlacementError::NestedGridLanesSubgridIndefiniteUnsupported);
        }
        if let Some(span) = item.definite_span {
            span.checked_len()?;
            if span.end > input.tracks.len() + 1 {
                return Err(LanePlacementError::SpanOutOfRange);
            }
            definite_items.push(DefiniteLaneIntrinsicItem {
                id: item.id,
                span,
                contribution: item.contribution,
            });
            continue;
        }

        let span = item.span.max(1).min(input.tracks.len());
        let contributions = item.contribution.contributions();
        if let Some(group) = indefinite_groups
            .iter_mut()
            .find(|group| group.span == span)
        {
            group.max_min_content = group.max_min_content.max(contributions.min_content);
            group.max_max_content = group.max_max_content.max(contributions.max_content);
            group.max_min_size = group.max_min_size.max(contributions.minimum);
            group.item_ids.push(item.id);
        } else {
            indefinite_groups.push(IndefiniteLaneContributionGroup {
                span,
                max_min_content: contributions.min_content,
                max_max_content: contributions.max_content,
                max_min_size: contributions.minimum,
                item_ids: vec![item.id],
            });
        }
    }

    let mut converted_indefinite_items = Vec::new();
    let mut sizing_items = Vec::new();
    for group in &indefinite_groups {
        for start_index in candidate_starts(input.tracks.len(), group.span) {
            let span = LaneTrackSpan::new(start_index + 1, start_index + 1 + group.span);
            let contribution = LaneContributionFacts {
                min_content: group.max_min_content,
                max_content: group.max_max_content,
                min_size: group.max_min_size,
                automatic_minimum_applies: false,
            };
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
                sizing_items.push(masonry_sizing_contribution(
                    MasonrySizingProjection {
                        full_span: span,
                        content_span,
                        tracks: &input.tracks,
                        available: input.available,
                        gap: input.gap,
                        content_track_count,
                        resolver,
                    },
                    group,
                ));
            }
        }
    }

    let mut final_track_sizes = input
        .tracks
        .iter()
        .map(|track| initialized_track_base(*track, input.available, resolver))
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
            resolver,
        );
    }
    for item in &sizing_items {
        apply_lane_sizing_contribution(
            &mut final_track_sizes,
            &input.tracks,
            input.gap,
            input.available,
            *item,
            resolver,
        );
    }

    Ok(LaneIntrinsicSizingReport {
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
struct MasonrySizingProjection<'a> {
    full_span: LaneTrackSpan,
    content_span: LaneTrackSpan,
    tracks: &'a [TrackSizing],
    available: Option<Scalar>,
    gap: Scalar,
    content_track_count: usize,
    resolver: &'a dyn CalcResolver,
}

fn masonry_sizing_contribution(
    projection: MasonrySizingProjection<'_>,
    group: &IndefiniteLaneContributionGroup,
) -> DefiniteLaneIntrinsicItem {
    let MasonrySizingProjection {
        full_span,
        content_span: span,
        tracks,
        available,
        gap,
        content_track_count,
        resolver,
    } = projection;
    let start_index = span.start - 1;
    let end_index = span.end - 1;
    let full_start_index = full_span.start - 1;
    let full_end_index = full_span.end - 1;
    let full_target = tracks[full_start_index..full_end_index]
        .iter()
        .map(|track| masonry_track_minimum_size(*track, group))
        .fold(0.0, Scalar::max);
    let full_existing = tracks[full_start_index..full_end_index]
        .iter()
        .map(|track| initialized_track_base(*track, available, resolver))
        .sum::<Scalar>()
        + gap
            * full_span
                .checked_len()
                .expect("span already validated")
                .saturating_sub(1) as Scalar;
    let content_existing = tracks[start_index..end_index]
        .iter()
        .map(|track| initialized_track_base(*track, available, resolver))
        .sum::<Scalar>()
        + gap
            * span
                .checked_len()
                .expect("span already validated")
                .saturating_sub(1) as Scalar;
    let content_span_len = span.checked_len().expect("span already validated");
    let deficit_share = (full_target - full_existing).max(0.0) * content_span_len as Scalar
        / content_track_count.max(1) as Scalar;
    let size = content_existing + deficit_share;
    let max_content = tracks[start_index..end_index]
        .iter()
        .map(|track| masonry_track_maximum_size(*track, size, group))
        .fold(0.0, Scalar::max);

    DefiniteLaneIntrinsicItem {
        id: "indefinite-group",
        span,
        contribution: LaneContributionFacts {
            min_content: size,
            max_content,
            min_size: size,
            automatic_minimum_applies: false,
        },
    }
}

fn masonry_track_minimum_size(
    track: TrackSizing,
    group: &IndefiniteLaneContributionGroup,
) -> Scalar {
    match track.min {
        MinTrackSizing::MinContent => group.max_min_content,
        MinTrackSizing::MaxContent => group.max_max_content,
        MinTrackSizing::Auto | MinTrackSizing::Length(_) => group.max_min_size,
    }
}

fn masonry_track_maximum_size(
    track: TrackSizing,
    minimum_size: Scalar,
    group: &IndefiniteLaneContributionGroup,
) -> Scalar {
    match track.max {
        MaxTrackSizing::MinContent => group.max_min_content,
        MaxTrackSizing::MaxContent | MaxTrackSizing::Auto | MaxTrackSizing::FitContent(_) => {
            group.max_max_content
        }
        MaxTrackSizing::Length(_) | MaxTrackSizing::Flex(_) => minimum_size,
    }
}

fn initialized_track_base(
    track: TrackSizing,
    available: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> Scalar {
    match track.min {
        MinTrackSizing::Length(length) => length.resolve_with(available, resolver).unwrap_or(0.0),
        MinTrackSizing::Auto | MinTrackSizing::MinContent | MinTrackSizing::MaxContent => 0.0,
    }
}

fn span_overlaps_content_tracks(span: LaneTrackSpan, content_sized_tracks: &[usize]) -> bool {
    let start = span.start - 1;
    let end = span.end - 1;
    content_sized_tracks
        .iter()
        .any(|track_index| (start..end).contains(track_index))
}

fn apply_lane_sizing_contribution(
    sizes: &mut [Scalar],
    tracks: &[TrackSizing],
    gap: Scalar,
    available: Option<Scalar>,
    item: DefiniteLaneIntrinsicItem,
    resolver: &dyn CalcResolver,
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
            resolver,
        ));
        return;
    }

    let target = span_contribution(contribution.minimum, end - start, gap);
    let current = sizes[start..end].iter().sum::<Scalar>()
        + span_tracks
            .iter()
            .map(|track| {
                if track_accepts_intrinsic_contribution(*track) {
                    0.0
                } else {
                    initialized_track_base(*track, available, resolver)
                }
            })
            .sum::<Scalar>();
    let extra = (target - current).max(0.0);
    if extra == 0.0 {
        return;
    }
    let share = extra / (end - start) as Scalar;
    for size in &mut sizes[start..end] {
        *size += share;
    }
}

fn lane_track_minimum_size(
    track: TrackSizing,
    contribution: LaneContributions,
    available: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> Scalar {
    match track.min {
        MinTrackSizing::MinContent => contribution.min_content,
        MinTrackSizing::MaxContent => contribution.max_content,
        MinTrackSizing::Auto => contribution.minimum,
        MinTrackSizing::Length(_) => initialized_track_base(track, available, resolver),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_grid_lanes_placement_with_resolved_tracks<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInput,
    constants: &Constants,
    context: GridContainerContext,
    columns: &[Scalar],
    rows: &[Scalar],
    placements: &GridPlacementContext<<Tree as Traverse>::Node>,
    grid_axis_gap: Scalar,
) -> Result<LanePlacementReport<<Tree as Traverse>::Node>, LanePlacementError>
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
            GridAxisKind::Column => context.column_basis.unwrap_or(0.0),
            GridAxisKind::Row => context.row_basis.unwrap_or(0.0),
        },
    );
    let lane_gap = match lane_axis {
        GridAxisKind::Column => context.gap.width,
        GridAxisKind::Row => context.gap.height,
    };
    let mut running = vec![0.0; grid_axis_tracks.len()];
    let mut item_offsets = Vec::new();
    let mut running_positions_after_each_item = Vec::new();
    let mut cursor = 0usize;
    let mut content_size: Scalar = 0.0;

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
                if start_line == 0 || grid_axis_span == 0 {
                    return Err(LanePlacementError::SpanOutOfRange);
                }
                let start = start_line - 1;
                if start + grid_axis_span > grid_axis_tracks.len() {
                    return Err(LanePlacementError::SpanOutOfRange);
                }
                (start, grid_axis_span)
            }
            None => {
                let span = grid_axis_span.clamp(1, grid_axis_tracks.len());
                let start = if matches!(style.grid_flow_tolerance, GridFlowTolerance::Infinite) {
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
            0.0
        };
        let lane_axis_margin_box = measure_lane_axis_margin_box_with_grid_axis(
            tree,
            child,
            &child_style,
            style,
            constants,
            lane_axis,
            grid_axis,
            grid_axis_size,
        );
        let previous = running[start..end].iter().copied().fold(0.0, Scalar::max);
        let new_position = previous + lane_axis_margin_box + lane_gap;
        content_size = content_size.max(new_position - lane_gap);
        for position in &mut running[start..end] {
            *position = new_position;
        }
        item_offsets.push(LaneItemOffset {
            item: child,
            grid_axis_start: start + 1,
            grid_axis_span: span,
            offset: previous,
            lane_axis_margin_box,
        });
        running_positions_after_each_item.push(running.clone());
        cursor = (start + span) % grid_axis_tracks.len();
    }

    Ok(LanePlacementReport {
        lane_axis,
        grid_axis,
        item_offsets,
        running_positions_after_each_item,
        content_size,
        final_cursor: cursor,
    })
}

pub(super) struct GridLanesLayoutInput<'a, Node> {
    pub(super) style: &'a NodeInput,
    pub(super) constants: &'a Constants,
    pub(super) container_content_size: Size,
    pub(super) columns: &'a [Scalar],
    pub(super) rows: &'a [Scalar],
    pub(super) gap: Size,
    pub(super) context: GridContainerContext,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<Node>,
}

#[derive(Clone, Copy)]
pub(super) struct LaneIntrinsicTrackSizeInput<'a, Node> {
    pub(super) constants: &'a Constants,
    pub(super) axis: GridAxisKind,
    pub(super) tracks: &'a [TrackSizing],
    pub(super) gap: Scalar,
    pub(super) available: Available,
    pub(super) available_basis: Option<Scalar>,
    pub(super) lines: GridLines,
    pub(super) placements: &'a GridPlacementContext<Node>,
}

pub(super) fn lane_intrinsic_track_sizes<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: LaneIntrinsicTrackSizeInput<'_, <Tree as Traverse>::Node>,
) -> Result<Vec<Scalar>, LanePlacementError>
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
        return Ok(vec![0.0; tracks.len()]);
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
            LaneIntrinsicItem::nested_indefinite_subgrid(
                "nested-subgrid",
                grid_axis_span,
                contribution,
            )
        } else if let Some(start) = definite_grid_axis_start {
            LaneIntrinsicItem::definite(
                "definite-item",
                start,
                start + grid_axis_span,
                contribution,
            )
        } else {
            LaneIntrinsicItem::indefinite("indefinite-item", grid_axis_span, contribution)
        };
        items.push(item);
    }

    lane_intrinsic_sizing_with(
        LaneIntrinsicSizingInput {
            axis,
            available: available_basis,
            gap,
            tracks: tracks.to_vec(),
            content_sized_tracks,
            items,
        },
        tree.calc_resolver(),
    )
    .map(|report| report.final_track_sizes)
}

fn lane_child_has_unsupported_indefinite_subgrid(style: &NodeInput, axis: GridAxisKind) -> bool {
    let axis_has_subgrid = match axis {
        GridAxisKind::Column => subgrid_components(&style.grid_template_columns),
        GridAxisKind::Row => subgrid_components(&style.grid_template_rows),
    };
    axis_has_subgrid && style.display.establishes_grid_formatting_context()
}

fn lane_child_contribution_facts<Tree>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    child_style: &NodeInput,
    constants: &Constants,
    axis: GridAxisKind,
    available: Available,
) -> LaneContributionFacts
where
    Tree: Compute,
{
    let min_available = lane_child_intrinsic_available(axis, child_style, available);
    let max_available = lane_child_intrinsic_available(axis, child_style, Available::MAX_CONTENT);
    let min_output = tree.compute_child(
        child,
        ComputeInput {
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
        ComputeInput {
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
    let margin = intrinsic_contribution_margin(
        child_style,
        constants.node_inner_size.width,
        tree.calc_resolver(),
    );
    LaneContributionFacts {
        min_content: axis_size(min_output.size, axis) + axis_margin_sum(margin, axis),
        max_content: axis_size(max_output.size, axis) + axis_margin_sum(margin, axis),
        min_size: if automatic_minimum_applies(child_style, axis) {
            axis_size(min_output.size, axis) + axis_margin_sum(margin, axis)
        } else {
            0.0
        },
        automatic_minimum_applies: automatic_minimum_applies(child_style, axis),
    }
}

fn automatic_minimum_applies(style: &NodeInput, axis: GridAxisKind) -> bool {
    !scroll_container_auto_minimum_zero(style, axis)
}

fn scroll_container_auto_minimum_zero(style: &NodeInput, axis: GridAxisKind) -> bool {
    match axis {
        GridAxisKind::Column => scroll_container_auto_minimum_zero_inline(style),
        GridAxisKind::Row => scroll_container_auto_minimum_zero_block(style),
    }
}

fn axis_size(size: Size, axis: GridAxisKind) -> Scalar {
    match axis {
        GridAxisKind::Column => size.width,
        GridAxisKind::Row => size.height,
    }
}

fn axis_margin_sum(margin: Edges, axis: GridAxisKind) -> Scalar {
    match axis {
        GridAxisKind::Column => margin.horizontal_sum(),
        GridAxisKind::Row => margin.vertical_sum(),
    }
}

pub(super) fn layout_grid_lanes_children<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridLanesLayoutInput<'_, <Tree as Traverse>::Node>,
) -> GridChildrenLayout
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
            tree.set_unrounded(child, NodeOutput::with_order(order as u32));
            tree.compute_child(child, ComputeInput::HIDDEN);
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
            tree.set_unrounded(child, NodeOutput::with_order(order as u32));
            tree.compute_child(child, ComputeInput::HIDDEN);
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
                        column_line_offset_adjustment: 0.0,
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
                    .and_then(|offsets| offsets.iter().copied().reduce(Scalar::min))
                    .unwrap_or(0.0);
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
                let y = row_offsets.get(start).copied().unwrap_or(0.0);
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
            tree.calc_resolver(),
        );
        let area_width_basis = Size::splat(Some(area_size.width));
        let padding = child_style
            .padding
            .zip_inline_size(area_width_basis, |length, basis| {
                resolve_length_or_zero_with(length, basis, tree.calc_resolver())
            });
        let border = child_style
            .border
            .zip_inline_size(area_width_basis, |length, basis| {
                resolve_length_or_zero_with(length, basis, tree.calc_resolver())
            });
        let resolved_margin = item.unresolved_margin.map(|margin| margin.unwrap_or(0.0));
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
            resolver: tree.calc_resolver(),
        });
        let child_input = ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: item.known,
            parent: Size::new(Some(area_size.width), Some(area_size.height)),
            available: item
                .available
                .map(|value| Available::Definite(value.max(0.0))),
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
                |length, basis| resolve_auto_optional_with(length, basis, tree.calc_resolver()),
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
            scrollbar_size: Size::new(
                if child_style.overflow.y == Overflow::Scroll {
                    child_style.scrollbar_width
                } else {
                    0.0
                },
                if child_style.overflow.x == Overflow::Scroll {
                    child_style.scrollbar_width
                } else {
                    0.0
                },
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
                            + item_offset.map_or(0.0, |offset| offset.offset)
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
                        + item_offset.map_or(0.0, |offset| offset.offset)
                        + item.vertical_axis.offset
                        + item.relative_offset.y
                }
            },
        );
        item.block_offset = location.y - row_offsets[item.area.row];
        tree.set_unrounded(
            item.node,
            NodeOutput {
                order: item.order,
                location,
                size: item.output.size,
                content_size: item.output.content_size,
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

#[allow(clippy::too_many_arguments)]
fn measure_lane_axis_margin_box_with_grid_axis<Tree>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    child_style: &NodeInput,
    container_style: &NodeInput,
    constants: &Constants,
    lane_axis: GridAxisKind,
    grid_axis: GridAxisKind,
    grid_axis_size: Scalar,
) -> Scalar
where
    Tree: Compute,
{
    let area_width_basis = match grid_axis {
        GridAxisKind::Column => Size::splat(Some(grid_axis_size)),
        GridAxisKind::Row => constants.node_inner_size,
    };
    let (margin, known, parent, available) = {
        let resolver = tree.calc_resolver();
        let unresolved_margin = child_style
            .margin
            .zip_inline_size(area_width_basis, |length, basis| {
                resolve_auto_optional_with(length, basis, resolver)
            });
        let margin = unresolved_margin.map(|margin| margin.unwrap_or(0.0));
        let mut known = Size::NONE;
        let mut parent = Size::NONE;
        let mut available = Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT);
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
                let available_width = (grid_axis_size - margin.horizontal_sum()).max(0.0);
                let justify_self = child_style
                    .justify_self
                    .or(container_style.justify_items)
                    .unwrap_or(AlignItems::Stretch);
                known.width =
                    resolve_dimension_with(child_style.size.width, Some(grid_axis_size), resolver)
                        .or_else(|| {
                            (justify_self == AlignItems::Stretch).then_some(available_width)
                        });
                parent.width = Some(grid_axis_size);
                available.width = Available::Definite(available_width);
            }
            GridAxisKind::Row => {
                let available_height = (grid_axis_size - margin.vertical_sum()).max(0.0);
                let align_self = child_style
                    .align_self
                    .or(container_style.align_items)
                    .unwrap_or(AlignItems::Stretch);
                known.height =
                    resolve_dimension_with(child_style.size.height, Some(grid_axis_size), resolver)
                        .or_else(|| {
                            (align_self == AlignItems::Stretch
                                && child_style.aspect_ratio.is_none())
                            .then_some(available_height)
                        });
                parent.height = Some(grid_axis_size);
                available.height = Available::Definite(available_height);
            }
        }
        (margin, known, parent, available)
    };
    let output = tree.compute_child(
        child,
        ComputeInput {
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

fn lane_child_intrinsic_available(
    grid_axis: GridAxisKind,
    child_style: &NodeInput,
    grid_axis_available: Available,
) -> Size<Available> {
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

fn intrinsic_available_for_dimension(dimension: Dimension) -> Available {
    match dimension {
        Dimension::MinContent => Available::MIN_CONTENT,
        Dimension::MaxContent => Available::MAX_CONTENT,
        Dimension::Px(_)
        | Dimension::Percent(_)
        | Dimension::Calc(_)
        | Dimension::Fr(_)
        | Dimension::Auto => Available::MAX_CONTENT,
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

    (None, placement.span.unwrap_or(1).max(1))
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

fn resolve_tolerance(tolerance: GridFlowTolerance, basis: Scalar) -> Scalar {
    match tolerance {
        GridFlowTolerance::Normal { font_size } => font_size,
        GridFlowTolerance::Length(length) => length.resolve(basis),
        GridFlowTolerance::Percent(factor) => factor * basis,
        GridFlowTolerance::Infinite => Scalar::INFINITY,
    }
}

fn infinite_candidate_start(cursor: usize, span: usize, track_count: usize) -> usize {
    if cursor + span > track_count {
        0
    } else {
        cursor
    }
}

fn finite_candidate_start(
    running: &[Scalar],
    cursor: usize,
    span: usize,
    tolerance: Scalar,
) -> usize {
    let track_count = running.len();
    let max_start = track_count + 1 - span;
    let shifted_cursor = if cursor >= max_start { 0 } else { cursor };
    let absolute_shortest = (0..max_start)
        .map(|start| max_running_position(running, start, span))
        .fold(Scalar::INFINITY, Scalar::min);

    for offset in 0..max_start {
        let start = (shifted_cursor + offset) % max_start;
        if max_running_position(running, start, span) <= absolute_shortest + tolerance {
            return start;
        }
    }

    0
}

fn max_running_position(running: &[Scalar], start: usize, span: usize) -> Scalar {
    running[start..start + span]
        .iter()
        .copied()
        .fold(0.0, Scalar::max)
}
