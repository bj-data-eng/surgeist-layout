use super::contributions::ItemContributionFacts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleGridError {
    NamedLineInheritanceUnsupported,
    BaselineInferenceUnsupported,
    MissingIntrinsicMinTrackFacts,
    NestedGridLanesSubgridIndefiniteUnsupported,
    StandaloneSubgridTraversalUnsupported,
    EmptyTrackList,
    SpanOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubgridEligibilityInput {
    pub requested: bool,
    pub has_parent_grid: bool,
    pub independent_formatting_context: bool,
    pub excluded_from_normal_layout: bool,
    pub parent_is_lanes_in_resolved_axis: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridIneligibleReason {
    NotRequested,
    NoParentGrid,
    IndependentFormattingContext,
    ExcludedFromNormalLayout,
    ParentIsLanesInResolvedAxis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubgridEligibilityReport {
    pub eligible: bool,
    pub reason: Option<SubgridIneligibleReason>,
}

#[must_use]
pub fn subgrid_eligibility(input: SubgridEligibilityInput) -> SubgridEligibilityReport {
    let reason = if !input.requested {
        Some(SubgridIneligibleReason::NotRequested)
    } else if !input.has_parent_grid {
        Some(SubgridIneligibleReason::NoParentGrid)
    } else if input.independent_formatting_context {
        Some(SubgridIneligibleReason::IndependentFormattingContext)
    } else if input.excluded_from_normal_layout {
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    } else if input.parent_is_lanes_in_resolved_axis {
        Some(SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    } else {
        None
    };

    SubgridEligibilityReport {
        eligible: reason.is_none(),
        reason,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrackSpan {
    pub start: usize,
    pub end: usize,
}

impl TrackSpan {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn checked_len(self) -> Result<usize, OracleGridError> {
        if self.start == 0 || self.end <= self.start {
            Err(OracleGridError::SpanOutOfRange)
        } else {
            Ok(self.end - self.start)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OracleGap {
    Normal,
    Length(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OracleGapReport {
    pub specified: OracleGap,
    pub resolved: f32,
}

impl OracleGapReport {
    #[must_use]
    pub const fn length(resolved: f32) -> Self {
        Self {
            specified: OracleGap::Length(resolved),
            resolved,
        }
    }

    #[must_use]
    pub const fn normal_resolved_to(resolved: f32) -> Self {
        Self {
            specified: OracleGap::Normal,
            resolved,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTrackInheritanceInput {
    pub parent_tracks: Vec<f32>,
    pub parent_span: TrackSpan,
    pub reversed: bool,
    pub start_mbp: f32,
    pub end_mbp: f32,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTrackInheritanceReport {
    pub parent_span: TrackSpan,
    pub copied_parent_tracks: Vec<f32>,
    pub reversed: bool,
    pub after_reversal: Vec<f32>,
    pub start_mbp_removed: Vec<f32>,
    pub end_mbp_removed: Vec<f32>,
    pub gap_difference: f32,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
    pub final_tracks: Vec<f32>,
}

pub fn inherit_subgrid_tracks(
    input: SubgridTrackInheritanceInput,
) -> Result<SubgridTrackInheritanceReport, OracleGridError> {
    let span_len = input.parent_span.checked_len()?;
    if input.parent_tracks.is_empty() {
        return Err(OracleGridError::EmptyTrackList);
    }
    if input.parent_span.end > input.parent_tracks.len() + 1 || span_len == 0 {
        return Err(OracleGridError::SpanOutOfRange);
    }

    let start_index = input.parent_span.start - 1;
    let end_index = input.parent_span.end - 1;
    let copied_parent_tracks = input.parent_tracks[start_index..end_index].to_vec();
    let mut after_reversal = copied_parent_tracks.clone();
    if input.reversed {
        after_reversal.reverse();
    }

    let mut start_mbp_removed = after_reversal.clone();
    remove_from_tracks(&mut start_mbp_removed, input.start_mbp, true);

    let mut end_mbp_removed = start_mbp_removed.clone();
    remove_from_tracks(&mut end_mbp_removed, input.end_mbp, false);

    let gap_difference = (input.subgrid_gap.resolved - input.parent_gap.resolved) / 2.0;
    let mut final_tracks = end_mbp_removed.clone();
    if final_tracks.len() > 1 {
        for edge in 0..(final_tracks.len() - 1) {
            final_tracks[edge] = (final_tracks[edge] - gap_difference).max(0.0);
            final_tracks[edge + 1] = (final_tracks[edge + 1] - gap_difference).max(0.0);
        }
    }

    Ok(SubgridTrackInheritanceReport {
        parent_span: input.parent_span,
        copied_parent_tracks,
        reversed: input.reversed,
        after_reversal,
        start_mbp_removed,
        end_mbp_removed,
        gap_difference,
        parent_gap: input.parent_gap,
        subgrid_gap: input.subgrid_gap,
        final_tracks,
    })
}

fn remove_from_tracks(tracks: &mut [f32], mut amount: f32, forwards: bool) {
    let mut indices = (0..tracks.len()).collect::<Vec<_>>();
    if !forwards {
        indices.reverse();
    }

    for index in indices {
        if amount <= 0.0 {
            break;
        }
        let removed = tracks[index].min(amount);
        tracks[index] -= removed;
        amount -= removed;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisEdges {
    pub start: f32,
    pub end: f32,
}

impl AxisEdges {
    #[must_use]
    pub const fn sum(self) -> f32 {
        self.start + self.end
    }
}

impl Default for AxisEdges {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTraversalInput {
    pub ancestor_track_intrinsic_min_eligibility: Vec<bool>,
    pub root_children: Vec<SubgridChild>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubgridChild {
    Subgrid(SubgridNode),
    Leaf(SubgridLeaf),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridAxisKind {
    Inherited,
    Standalone,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridNode {
    pub id: &'static str,
    pub axis: SubgridAxisKind,
    pub reversed: bool,
    pub span_in_parent: TrackSpan,
    pub margins: AxisEdges,
    pub border: AxisEdges,
    pub padding: AxisEdges,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
    pub children: Vec<SubgridChild>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridLeaf {
    pub id: &'static str,
    pub span_in_parent: TrackSpan,
    pub contribution: ItemContributionFacts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridLeafContribution {
    pub id: &'static str,
    pub ancestor_span: TrackSpan,
    pub accumulated_edge_adjustment: Vec<f32>,
    pub accumulated_gap_adjustment: Vec<f32>,
    pub contribution: ItemContributionFacts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTraversalReport {
    pub edge_lower_bounds: Vec<f32>,
    pub leaves: Vec<SubgridLeafContribution>,
}

pub fn traverse_subgrid_intrinsic(
    input: SubgridTraversalInput,
) -> Result<SubgridTraversalReport, OracleGridError> {
    let mut edge_lower_bounds = vec![0.0; input.ancestor_track_intrinsic_min_eligibility.len()];
    let mut leaves = Vec::new();
    let mut stack = input
        .root_children
        .into_iter()
        .rev()
        .map(|child| {
            (
                child,
                TraversalContext {
                    line_offset: 0,
                    line_direction: 1,
                    accumulated_edge_adjustment: vec![0.0; edge_lower_bounds.len()],
                    accumulated_gap_adjustment: vec![0.0; edge_lower_bounds.len()],
                },
            )
        })
        .collect::<Vec<_>>();

    while let Some((child, context)) = stack.pop() {
        match child {
            SubgridChild::Leaf(leaf) => {
                leaf.span_in_parent.checked_len()?;
                leaves.push(SubgridLeafContribution {
                    id: leaf.id,
                    ancestor_span: translate_span_to_ancestor(&context, leaf.span_in_parent)?,
                    accumulated_edge_adjustment: context.accumulated_edge_adjustment,
                    accumulated_gap_adjustment: context.accumulated_gap_adjustment,
                    contribution: leaf.contribution,
                });
            }
            SubgridChild::Subgrid(subgrid) => {
                apply_subgrid_edge_placeholders(
                    &input.ancestor_track_intrinsic_min_eligibility,
                    &mut edge_lower_bounds,
                    &mut stack,
                    subgrid,
                    context,
                )?;
            }
        }
    }

    Ok(SubgridTraversalReport {
        edge_lower_bounds,
        leaves,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct TraversalContext {
    line_offset: isize,
    line_direction: isize,
    accumulated_edge_adjustment: Vec<f32>,
    accumulated_gap_adjustment: Vec<f32>,
}

fn apply_subgrid_edge_placeholders(
    intrinsic_min: &[bool],
    edge_lower_bounds: &mut [f32],
    stack: &mut Vec<(SubgridChild, TraversalContext)>,
    subgrid: SubgridNode,
    mut context: TraversalContext,
) -> Result<(), OracleGridError> {
    if subgrid.axis == SubgridAxisKind::Standalone {
        return Err(OracleGridError::StandaloneSubgridTraversalUnsupported);
    }

    subgrid.span_in_parent.checked_len()?;
    let ancestor_span = translate_span_to_ancestor(&context, subgrid.span_in_parent)?;
    let start_index = ancestor_span.start - 1;
    let end_index = ancestor_span.end - 2;
    if end_index >= intrinsic_min.len()
        || end_index >= edge_lower_bounds.len()
        || context.accumulated_edge_adjustment.len() != edge_lower_bounds.len()
        || context.accumulated_gap_adjustment.len() != edge_lower_bounds.len()
    {
        return Err(OracleGridError::MissingIntrinsicMinTrackFacts);
    }

    let span_len = subgrid.span_in_parent.checked_len()?;
    let child_line_transform =
        child_line_transform(&context, subgrid.span_in_parent, subgrid.reversed);
    let (local_start_index, local_end_index) = edge_track_indices(&child_line_transform, span_len)?;

    let local_start_edge = subgrid.margins.start + subgrid.border.start + subgrid.padding.start;
    let local_end_edge = subgrid.margins.end + subgrid.border.end + subgrid.padding.end;

    if intrinsic_min[local_start_index] {
        context.accumulated_edge_adjustment[local_start_index] += local_start_edge;
        edge_lower_bounds[local_start_index] = edge_lower_bounds[local_start_index]
            .max(context.accumulated_edge_adjustment[local_start_index]);
    }
    if intrinsic_min[local_end_index] {
        context.accumulated_edge_adjustment[local_end_index] += local_end_edge;
        edge_lower_bounds[local_end_index] = edge_lower_bounds[local_end_index]
            .max(context.accumulated_edge_adjustment[local_end_index]);
    }

    let gap_difference = (subgrid.subgrid_gap.resolved - subgrid.parent_gap.resolved) / 2.0;
    for edge_index in start_index..end_index {
        context.accumulated_gap_adjustment[edge_index] += gap_difference;
        context.accumulated_gap_adjustment[edge_index + 1] += gap_difference;
    }

    let child_context = TraversalContext {
        line_offset: child_line_transform.line_offset,
        line_direction: child_line_transform.line_direction,
        accumulated_edge_adjustment: context.accumulated_edge_adjustment,
        accumulated_gap_adjustment: context.accumulated_gap_adjustment,
    };

    for child in subgrid.children.into_iter().rev() {
        stack.push((child, child_context.clone()));
    }

    Ok(())
}

fn child_line_transform(
    context: &TraversalContext,
    span_in_parent: TrackSpan,
    reversed: bool,
) -> TraversalContext {
    let local_offset = if reversed {
        span_in_parent.end as isize + 1
    } else {
        span_in_parent.start as isize - 1
    };
    let local_direction = if reversed { -1 } else { 1 };
    TraversalContext {
        line_offset: context.line_offset + context.line_direction * local_offset,
        line_direction: context.line_direction * local_direction,
        accumulated_edge_adjustment: Vec::new(),
        accumulated_gap_adjustment: Vec::new(),
    }
}

fn translate_span_to_ancestor(
    context: &TraversalContext,
    local_span: TrackSpan,
) -> Result<TrackSpan, OracleGridError> {
    let start_line = map_line_to_ancestor(context, local_span.start);
    let end_line = map_line_to_ancestor(context, local_span.end);
    let start = start_line.min(end_line);
    let end = start_line.max(end_line);
    if start <= 0 || end <= start {
        return Err(OracleGridError::SpanOutOfRange);
    }
    Ok(TrackSpan::new(start as usize, end as usize))
}

fn edge_track_indices(
    context: &TraversalContext,
    span_len: usize,
) -> Result<(usize, usize), OracleGridError> {
    let local_start_line = map_line_to_ancestor(context, 1);
    let local_end_line = map_line_to_ancestor(context, span_len + 1);
    let (start_edge_index, end_edge_index) = if context.line_direction > 0 {
        (local_start_line - 1, local_end_line - 2)
    } else {
        (local_start_line - 2, local_end_line - 1)
    };
    if start_edge_index < 0 || end_edge_index < 0 {
        return Err(OracleGridError::SpanOutOfRange);
    }
    Ok((start_edge_index as usize, end_edge_index as usize))
}

fn map_line_to_ancestor(context: &TraversalContext, line: usize) -> isize {
    context.line_offset + context.line_direction * line as isize
}
