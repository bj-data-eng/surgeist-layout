use super::*;
use crate::geometry::{FlowAxes, LogicalSizeOf};
use crate::geometry::{PhysicalAxis, PhysicalProgression};
use crate::output::PhysicalBaseline;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct GridSubgridReport<Node> {
    pub(super) items: Vec<SubgridItemReport<Node>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubgridItemReport<Node> {
    pub(super) node: Node,
    pub(super) column: SubgridAxisReport,
    pub(super) row: SubgridAxisReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SubgridAxisReport {
    pub(super) mapping: Result<GridAxisMappingReport, GridAxisMappingError>,
    pub(super) eligibility: SubgridEligibility,
}

impl SubgridAxisReport {
    pub(super) fn can_inherit(self) -> bool {
        self.eligibility.eligible && self.mapping.is_ok()
    }
}

pub(super) fn inherited_subgrid_physical_axis(
    report: SubgridAxisReport,
    parent_flow_axes: crate::geometry::FlowAxes,
    child_flow_axes: crate::geometry::FlowAxes,
) -> Option<PhysicalAxis> {
    if !report.can_inherit() {
        return None;
    }
    let mapping = report.mapping.ok()?;
    let physical_axis = physical_axis_for_grid_axis(parent_flow_axes, mapping.parent_axis);
    if physical_axis != physical_axis_for_grid_axis(child_flow_axes, mapping.child_axis) {
        return None;
    }
    Some(physical_axis)
}

pub(super) fn subgrid_parent_propagation_axes(
    item: SubgridItemReport<impl Copy>,
    parent_flow_axes: crate::geometry::FlowAxes,
    child_flow_axes: crate::geometry::FlowAxes,
) -> (bool, bool) {
    let column_axis =
        inherited_subgrid_physical_axis(item.column, parent_flow_axes, child_flow_axes);
    let row_axis = inherited_subgrid_physical_axis(item.row, parent_flow_axes, child_flow_axes);
    if column_axis.is_none() && row_axis.is_none() {
        return (true, true);
    }

    (
        column_axis == Some(PhysicalAxis::Horizontal) || row_axis == Some(PhysicalAxis::Horizontal),
        column_axis == Some(PhysicalAxis::Vertical) || row_axis == Some(PhysicalAxis::Vertical),
    )
}

fn physical_axis_for_grid_axis(
    flow_axes: crate::geometry::FlowAxes,
    axis: GridAxisKind,
) -> PhysicalAxis {
    match axis {
        GridAxisKind::Column => flow_axes.inline_axis(),
        GridAxisKind::Row => flow_axes.block_axis(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SubgridEligibility {
    pub(super) eligible: bool,
    pub(super) reason: Option<SubgridIneligibleReason>,
}

#[derive(Clone, Copy)]
pub(super) struct SubgridEligibilityInput<'a, S: LayoutScalar = Scalar> {
    pub(super) axis: GridAxisKind,
    pub(super) parent_style: &'a NodeInputOf<S>,
    pub(super) has_parent_grid: bool,
    pub(super) child_style: &'a NodeInputOf<S>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridIneligibleReason {
    NotRequested,
    NoParentGrid,
    UnsupportedDisplay,
    IndependentFormattingContext,
    ExcludedFromNormalLayout,
    ParentIsLanesInResolvedAxis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GridTrackSpan {
    pub(super) start: usize,
    pub(super) end: usize,
}

impl GridTrackSpan {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(super) fn checked_len(self) -> Option<usize> {
        self.end
            .checked_sub(self.start)
            .filter(|length| *length > 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum IntrinsicMinTrackFacts<'a> {
    Known(&'a [bool]),
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "retained missing-facts state for subgrid traversal error parity"
        )
    )]
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridTraversalError {
    MissingIntrinsicMinTrackFacts,
    StandaloneSubgridTraversalUnsupported,
    SpanOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridTraversalAxis {
    Inherited,
    Standalone,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubgridAxisEdges<S: LayoutScalar = Scalar> {
    pub(super) start: S,
    pub(super) end: S,
}

impl<S: LayoutScalar> Default for SubgridAxisEdges<S> {
    fn default() -> Self {
        Self {
            start: S::ZERO,
            end: S::ZERO,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridTraversalInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts<'a>,
    pub(super) root_children: Vec<SubgridTraversalChild<Node, S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum SubgridTraversalChild<Node, S: LayoutScalar = Scalar> {
    Subgrid(SubgridTraversalNode<Node, S>),
    Leaf(SubgridTraversalLeaf<Node, S>),
}

type SubgridTraversalChildren<Node, S = Scalar> = Vec<SubgridTraversalChild<Node, S>>;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridTraversalNode<Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) axis: SubgridTraversalAxis,
    pub(super) reversed: bool,
    pub(super) span_in_parent: GridTrackSpan,
    pub(super) available_inline_size: Option<S>,
    pub(super) available_inline_size_is_known: bool,
    pub(super) queried_axis_fully_inherited: bool,
    pub(super) margins: SubgridAxisEdges<S>,
    pub(super) border: SubgridAxisEdges<S>,
    pub(super) padding: SubgridAxisEdges<S>,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: S,
    pub(super) children: Vec<SubgridTraversalChild<Node, S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridTraversalLeaf<Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) span_in_parent: GridTrackSpan,
    pub(super) available_inline_size: Option<S>,
    pub(super) available_inline_size_is_known: bool,
    pub(super) align_self: AlignItems,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridLeafContribution<Node, S: LayoutScalar = Scalar> {
    pub(super) root_node: Option<Node>,
    pub(super) root_axis_fully_inherited: bool,
    pub(super) node: Node,
    pub(super) ancestor_span: GridTrackSpan,
    pub(super) available_inline_size: Option<S>,
    pub(super) available_inline_size_is_known: bool,
    pub(super) align_self: AlignItems,
    pub(super) accumulated_edge_adjustment: Vec<S>,
    pub(super) accumulated_gap_adjustment: Vec<S>,
    accumulated_start_adjustment: Vec<S>,
    accumulated_end_adjustment: Vec<S>,
    outer_half_gap: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubgridAncestorBaselineAdjustments<S: LayoutScalar = Scalar> {
    pub(super) start: S,
    pub(super) end: S,
}

impl<Node, S: LayoutScalar> SubgridLeafContribution<Node, S> {
    pub(super) fn scalar_adjustment(&self) -> Option<S> {
        let start = self.ancestor_span.start.checked_sub(1)?;
        let end = self.ancestor_span.end.checked_sub(1)?;
        let edge_adjustments = self.accumulated_edge_adjustment.get(start..end)?;
        let gap_adjustments = self.accumulated_gap_adjustment.get(start..end)?;
        Some(
            edge_adjustments
                .iter()
                .chain(gap_adjustments)
                .copied()
                .fold(S::ZERO, |sum, value| sum + value),
        )
    }

    pub(super) fn ancestor_baseline_adjustments(
        &self,
        containing_flow_axes: FlowAxes,
        axis: GridAxisKind,
    ) -> Option<SubgridAncestorBaselineAdjustments<S>> {
        let start = self.ancestor_span.start.checked_sub(1)?;
        let end = self.ancestor_span.end.checked_sub(2)?;
        let mut adjustments = SubgridAncestorBaselineAdjustments {
            start: *self.accumulated_start_adjustment.get(start)?,
            end: *self.accumulated_end_adjustment.get(end)?,
        };
        let logical_axis = match axis {
            GridAxisKind::Column => LogicalAxis::Inline,
            GridAxisKind::Row => LogicalAxis::Block,
        };
        if containing_flow_axes
            .logical_axis_progression(logical_axis)
            .is_decreasing()
        {
            adjustments.start = adjustments.start + self.outer_half_gap;
            adjustments.end = adjustments.end - self.outer_half_gap;
        }
        Some(adjustments)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridTraversalReport<Node, S: LayoutScalar = Scalar> {
    pub(super) edge_lower_bounds: Vec<S>,
    pub(super) leaves: Vec<SubgridLeafContribution<Node, S>>,
    pub(super) baseline_views: Vec<SubgridBaselineViewTransform<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubgridBaselineViewTransform<S: LayoutScalar = Scalar> {
    pub(super) parent_span: GridTrackSpan,
    pub(super) reversed: bool,
    pub(super) root: bool,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: S,
}

pub(super) fn traverse_subgrid_intrinsic<Node, S: LayoutScalar>(
    input: SubgridTraversalInput<'_, Node, S>,
) -> Result<SubgridTraversalReport<Node, S>, SubgridTraversalError>
where
    Node: Copy,
{
    let intrinsic_min = match input.ancestor_track_intrinsic_min_eligibility {
        IntrinsicMinTrackFacts::Known(facts) => facts,
        IntrinsicMinTrackFacts::Unknown => {
            return Err(SubgridTraversalError::MissingIntrinsicMinTrackFacts);
        }
    };
    let mut edge_lower_bounds = vec![S::ZERO; intrinsic_min.len()];
    let mut leaves = Vec::new();
    let mut baseline_views = Vec::new();
    let mut stack = input
        .root_children
        .into_iter()
        .rev()
        .map(|child| {
            (
                child,
                SubgridTraversalContext {
                    root_node: None,
                    root_axis_fully_inherited: true,
                    line_offset: 0,
                    line_direction: 1,
                    available_inline_size: None,
                    available_inline_size_is_known: false,
                    accumulated_edge_adjustment: vec![S::ZERO; intrinsic_min.len()],
                    accumulated_gap_adjustment: vec![S::ZERO; intrinsic_min.len()],
                    accumulated_start_adjustment: vec![S::ZERO; intrinsic_min.len()],
                    accumulated_end_adjustment: vec![S::ZERO; intrinsic_min.len()],
                    outer_half_gap: S::ZERO,
                },
            )
        })
        .collect::<Vec<_>>();

    while let Some((child, context)) = stack.pop() {
        match child {
            SubgridTraversalChild::Leaf(leaf) => {
                leaf.span_in_parent
                    .checked_len()
                    .ok_or(SubgridTraversalError::SpanOutOfRange)?;
                leaves.push(SubgridLeafContribution {
                    root_node: context.root_node.or(Some(leaf.node)),
                    root_axis_fully_inherited: context.root_axis_fully_inherited,
                    node: leaf.node,
                    ancestor_span: translate_span_to_ancestor(&context, leaf.span_in_parent)?,
                    available_inline_size: leaf
                        .available_inline_size
                        .or(context.available_inline_size),
                    available_inline_size_is_known: leaf.available_inline_size_is_known
                        || context.available_inline_size_is_known,
                    align_self: leaf.align_self,
                    accumulated_edge_adjustment: context.accumulated_edge_adjustment,
                    accumulated_gap_adjustment: context.accumulated_gap_adjustment,
                    accumulated_start_adjustment: context.accumulated_start_adjustment,
                    accumulated_end_adjustment: context.accumulated_end_adjustment,
                    outer_half_gap: context.outer_half_gap,
                });
            }
            SubgridTraversalChild::Subgrid(subgrid) => {
                baseline_views.push(SubgridBaselineViewTransform {
                    parent_span: translate_span_to_ancestor(&context, subgrid.span_in_parent)?,
                    reversed: subgrid.reversed,
                    root: context.root_node.is_none(),
                    parent_gap: subgrid.parent_gap,
                    subgrid_gap: subgrid.subgrid_gap,
                });
                apply_subgrid_edge_placeholders(
                    intrinsic_min,
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
        baseline_views,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct SubgridTraversalContext<Node, S: LayoutScalar = Scalar> {
    root_node: Option<Node>,
    root_axis_fully_inherited: bool,
    line_offset: isize,
    line_direction: isize,
    available_inline_size: Option<S>,
    available_inline_size_is_known: bool,
    accumulated_edge_adjustment: Vec<S>,
    accumulated_gap_adjustment: Vec<S>,
    accumulated_start_adjustment: Vec<S>,
    accumulated_end_adjustment: Vec<S>,
    outer_half_gap: S,
}

type SubgridTraversalStackEntry<Node, S = Scalar> = (
    SubgridTraversalChild<Node, S>,
    SubgridTraversalContext<Node, S>,
);

fn apply_subgrid_edge_placeholders<Node, S: LayoutScalar>(
    intrinsic_min: &[bool],
    edge_lower_bounds: &mut [S],
    stack: &mut Vec<SubgridTraversalStackEntry<Node, S>>,
    subgrid: SubgridTraversalNode<Node, S>,
    mut context: SubgridTraversalContext<Node, S>,
) -> Result<(), SubgridTraversalError>
where
    Node: Copy,
{
    if subgrid.axis == SubgridTraversalAxis::Standalone {
        return Err(SubgridTraversalError::StandaloneSubgridTraversalUnsupported);
    }

    let span_len = subgrid
        .span_in_parent
        .checked_len()
        .ok_or(SubgridTraversalError::SpanOutOfRange)?;
    let ancestor_span = translate_span_to_ancestor(&context, subgrid.span_in_parent)?;
    let start_index = ancestor_span.start - 1;
    let end_index = ancestor_span.end - 2;
    if end_index >= intrinsic_min.len()
        || end_index >= edge_lower_bounds.len()
        || context.accumulated_edge_adjustment.len() != edge_lower_bounds.len()
        || context.accumulated_gap_adjustment.len() != edge_lower_bounds.len()
        || context.accumulated_start_adjustment.len() != edge_lower_bounds.len()
        || context.accumulated_end_adjustment.len() != edge_lower_bounds.len()
    {
        return Err(SubgridTraversalError::MissingIntrinsicMinTrackFacts);
    }

    let child_line_transform =
        child_line_transform(&context, subgrid.span_in_parent, subgrid.reversed);
    let (local_start_index, local_end_index) = edge_track_indices(&child_line_transform, span_len)?;
    if local_start_index >= intrinsic_min.len() || local_end_index >= intrinsic_min.len() {
        return Err(SubgridTraversalError::MissingIntrinsicMinTrackFacts);
    }

    let local_start_edge = subgrid.margins.start + subgrid.border.start + subgrid.padding.start;
    let local_end_edge = subgrid.margins.end + subgrid.border.end + subgrid.padding.end;

    let start_role_adjustment = if child_line_transform.line_direction > 0 {
        &mut context.accumulated_start_adjustment
    } else {
        &mut context.accumulated_end_adjustment
    };
    start_role_adjustment[local_start_index] =
        start_role_adjustment[local_start_index] + local_start_edge;
    if intrinsic_min[local_start_index] {
        context.accumulated_edge_adjustment[local_start_index] =
            context.accumulated_edge_adjustment[local_start_index] + local_start_edge;
        edge_lower_bounds[local_start_index] = edge_lower_bounds[local_start_index]
            .max(context.accumulated_edge_adjustment[local_start_index]);
    }
    let end_role_adjustment = if child_line_transform.line_direction > 0 {
        &mut context.accumulated_end_adjustment
    } else {
        &mut context.accumulated_start_adjustment
    };
    end_role_adjustment[local_end_index] = end_role_adjustment[local_end_index] + local_end_edge;
    if intrinsic_min[local_end_index] {
        context.accumulated_edge_adjustment[local_end_index] =
            context.accumulated_edge_adjustment[local_end_index] + local_end_edge;
        edge_lower_bounds[local_end_index] = edge_lower_bounds[local_end_index]
            .max(context.accumulated_edge_adjustment[local_end_index]);
    }

    let empty_subgrid = subgrid.children.is_empty();
    let gap_difference = (subgrid.subgrid_gap - subgrid.parent_gap) / S::from_f64(2.0);
    if context.root_node.is_none() {
        context.outer_half_gap = gap_difference;
    }
    for edge_index in start_index..end_index {
        context.accumulated_gap_adjustment[edge_index] =
            context.accumulated_gap_adjustment[edge_index] + gap_difference;
        context.accumulated_gap_adjustment[edge_index + 1] =
            context.accumulated_gap_adjustment[edge_index + 1] + gap_difference;
        context.accumulated_end_adjustment[edge_index] =
            context.accumulated_end_adjustment[edge_index] + gap_difference;
        context.accumulated_start_adjustment[edge_index + 1] =
            context.accumulated_start_adjustment[edge_index + 1] + gap_difference;
    }
    if empty_subgrid {
        for edge_index in start_index..=end_index {
            if intrinsic_min[edge_index] {
                let lower_bound = context.accumulated_edge_adjustment[edge_index]
                    + context.accumulated_gap_adjustment[edge_index].max(S::ZERO);
                edge_lower_bounds[edge_index] = edge_lower_bounds[edge_index].max(lower_bound);
            }
        }
    }

    let (available_inline_size, available_inline_size_is_known) =
        if subgrid.available_inline_size.is_some() {
            (
                subgrid.available_inline_size,
                subgrid.available_inline_size_is_known,
            )
        } else {
            (
                context.available_inline_size,
                context.available_inline_size_is_known,
            )
        };
    let child_context = SubgridTraversalContext {
        root_node: context.root_node.or(Some(subgrid.node)),
        root_axis_fully_inherited: context.root_axis_fully_inherited
            && subgrid.queried_axis_fully_inherited,
        line_offset: child_line_transform.line_offset,
        line_direction: child_line_transform.line_direction,
        available_inline_size,
        available_inline_size_is_known,
        accumulated_edge_adjustment: context.accumulated_edge_adjustment,
        accumulated_gap_adjustment: context.accumulated_gap_adjustment,
        accumulated_start_adjustment: context.accumulated_start_adjustment,
        accumulated_end_adjustment: context.accumulated_end_adjustment,
        outer_half_gap: context.outer_half_gap,
    };

    for child in subgrid.children.into_iter().rev() {
        stack.push((child, child_context.clone()));
    }

    Ok(())
}

fn child_line_transform<Node, S: LayoutScalar>(
    context: &SubgridTraversalContext<Node, S>,
    span_in_parent: GridTrackSpan,
    reversed: bool,
) -> SubgridTraversalContext<Node, S>
where
    Node: Copy,
{
    let local_offset = if reversed {
        span_in_parent.end as isize + 1
    } else {
        span_in_parent.start as isize - 1
    };
    let local_direction = if reversed { -1 } else { 1 };
    SubgridTraversalContext {
        root_node: context.root_node,
        root_axis_fully_inherited: context.root_axis_fully_inherited,
        line_offset: context.line_offset + context.line_direction * local_offset,
        line_direction: context.line_direction * local_direction,
        available_inline_size: context.available_inline_size,
        available_inline_size_is_known: context.available_inline_size_is_known,
        accumulated_edge_adjustment: Vec::new(),
        accumulated_gap_adjustment: Vec::new(),
        accumulated_start_adjustment: Vec::new(),
        accumulated_end_adjustment: Vec::new(),
        outer_half_gap: context.outer_half_gap,
    }
}

fn translate_span_to_ancestor<Node, S: LayoutScalar>(
    context: &SubgridTraversalContext<Node, S>,
    local_span: GridTrackSpan,
) -> Result<GridTrackSpan, SubgridTraversalError>
where
    Node: Copy,
{
    let start_line = map_line_to_ancestor(context, local_span.start);
    let end_line = map_line_to_ancestor(context, local_span.end);
    let start = start_line.min(end_line);
    let end = start_line.max(end_line);
    if start <= 0 || end <= start {
        return Err(SubgridTraversalError::SpanOutOfRange);
    }
    Ok(GridTrackSpan::new(start as usize, end as usize))
}

fn edge_track_indices<Node, S: LayoutScalar>(
    context: &SubgridTraversalContext<Node, S>,
    span_len: usize,
) -> Result<(usize, usize), SubgridTraversalError>
where
    Node: Copy,
{
    let local_start_line = map_line_to_ancestor(context, 1);
    let local_end_line = map_line_to_ancestor(context, span_len + 1);
    let (start_edge_index, end_edge_index) = if context.line_direction > 0 {
        (local_start_line - 1, local_end_line - 2)
    } else {
        (local_start_line - 2, local_end_line - 1)
    };
    if start_edge_index < 0 || end_edge_index < 0 {
        return Err(SubgridTraversalError::SpanOutOfRange);
    }
    Ok((start_edge_index as usize, end_edge_index as usize))
}

fn map_line_to_ancestor<Node, S: LayoutScalar>(
    context: &SubgridTraversalContext<Node, S>,
    line: usize,
) -> isize
where
    Node: Copy,
{
    context.line_offset + context.line_direction * line as isize
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ResolvedSubgridGap<S: LayoutScalar = Scalar> {
    Normal,
    Length(S),
}

impl<S: LayoutScalar> ResolvedSubgridGap<S> {
    const fn resolve(self, parent_gap: S) -> S {
        match self {
            Self::Normal => parent_gap,
            Self::Length(length) => length,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SubgridTrackInheritanceInput<'a, S: LayoutScalar = Scalar> {
    pub(super) parent_tracks: &'a [S],
    pub(super) parent_span: GridTrackSpan,
    pub(super) reversed: bool,
    pub(super) start_mbp: S,
    pub(super) end_mbp: S,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: ResolvedSubgridGap<S>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridTrackInheritanceReport<S: LayoutScalar = Scalar> {
    pub(super) parent_span: GridTrackSpan,
    pub(super) copied_parent_tracks: Vec<S>,
    pub(super) reversed: bool,
    pub(super) after_reversal: Vec<S>,
    pub(super) start_mbp_removed: Vec<S>,
    pub(super) end_mbp_removed: Vec<S>,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: ResolvedSubgridGap<S>,
    pub(super) resolved_subgrid_gap: S,
    pub(super) gap_difference: S,
    pub(super) final_tracks: Vec<S>,
}

#[derive(Clone, Copy)]
pub(super) struct SubgridBaselineInheritanceInput<'a, S: LayoutScalar = Scalar> {
    pub(super) parent_major: &'a [Option<PhysicalBaseline<S>>],
    pub(super) parent_minor: &'a [Option<PhysicalBaseline<S>>],
    pub(super) physical_axis: PhysicalAxis,
    pub(super) parent_span: GridTrackSpan,
    pub(super) reversed: bool,
    pub(super) start_mbp: S,
    pub(super) end_mbp: S,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: S,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridBaselineInheritanceReport<S: LayoutScalar = Scalar> {
    pub(super) parent_span: GridTrackSpan,
    pub(super) reversed: bool,
    pub(super) start_mbp: S,
    pub(super) end_mbp: S,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: S,
    pub(super) gap_difference: S,
    pub(super) sliced_major: Vec<Option<PhysicalBaseline<S>>>,
    pub(super) sliced_minor: Vec<Option<PhysicalBaseline<S>>>,
    pub(super) after_reversal_major: Vec<Option<PhysicalBaseline<S>>>,
    pub(super) after_reversal_minor: Vec<Option<PhysicalBaseline<S>>>,
    pub(super) after_mbp_major: Vec<Option<PhysicalBaseline<S>>>,
    pub(super) after_mbp_minor: Vec<Option<PhysicalBaseline<S>>>,
    pub(super) final_major: Vec<Option<PhysicalBaseline<S>>>,
    pub(super) final_minor: Vec<Option<PhysicalBaseline<S>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ChildBaselineEnvelopeView<S: LayoutScalar = Scalar> {
    pub(super) major: Vec<Option<PhysicalBaseline<S>>>,
    pub(super) minor: Vec<Option<PhysicalBaseline<S>>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OwnerTrackPlacementEntry<S: LayoutScalar = Scalar> {
    owner_track: usize,
    first_frame_translation: S,
    last_frame_translation: S,
    first_gutter_translation: S,
    last_gutter_translation: S,
}

impl<S: LayoutScalar> OwnerTrackPlacementEntry<S> {
    fn frame_translation(self, role: AncestorBaselineRole) -> S {
        match role {
            AncestorBaselineRole::First => self.first_frame_translation,
            AncestorBaselineRole::Last => self.last_frame_translation,
        }
    }

    fn gutter_translation(self, role: AncestorBaselineRole) -> S {
        match role {
            AncestorBaselineRole::First => self.first_gutter_translation,
            AncestorBaselineRole::Last => self.last_gutter_translation,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct CheckedOwnerToCurrentPlacementMap<Node, S: LayoutScalar = Scalar> {
    owner: Node,
    current_grid: Node,
    current_axis: GridAxisKind,
    owner_axis: GridAxisKind,
    physical_axis: PhysicalAxis,
    owner_progression: PhysicalProgression,
    current_progression: PhysicalProgression,
    boundary_count: usize,
    tracks: Vec<OwnerTrackPlacementEntry<S>>,
}

#[derive(Clone, Copy)]
pub(super) struct OwnerToCurrentPlacementBoundaryInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) parent_grid: Node,
    pub(super) current_grid: Node,
    pub(super) parent_axis: GridAxisKind,
    pub(super) current_axis: GridAxisKind,
    pub(super) physical_axis: PhysicalAxis,
    pub(super) parent_progression: PhysicalProgression,
    pub(super) current_progression: PhysicalProgression,
    pub(super) parent_span: GridTrackSpan,
    pub(super) reversed: bool,
    pub(super) parent_first_frame_origins: &'a [S],
    pub(super) parent_last_frame_origins: &'a [S],
    pub(super) current_first_frame_origins: &'a [S],
    pub(super) current_last_frame_origins: &'a [S],
    pub(super) parent_gap: S,
    pub(super) current_gap: S,
    pub(super) start_mbp: S,
    pub(super) end_mbp: S,
    pub(super) inherited: bool,
}

impl<Node: Copy + PartialEq, S: LayoutScalar> CheckedOwnerToCurrentPlacementMap<Node, S> {
    pub(super) fn identity(
        owner: Node,
        axis: GridAxisKind,
        physical_axis: PhysicalAxis,
        progression: PhysicalProgression,
        track_count: usize,
    ) -> Self {
        Self {
            owner,
            current_grid: owner,
            current_axis: axis,
            owner_axis: axis,
            physical_axis,
            owner_progression: progression,
            current_progression: progression,
            boundary_count: 0,
            tracks: (0..track_count)
                .map(|owner_track| OwnerTrackPlacementEntry {
                    owner_track,
                    first_frame_translation: S::ZERO,
                    last_frame_translation: S::ZERO,
                    first_gutter_translation: S::ZERO,
                    last_gutter_translation: S::ZERO,
                })
                .collect(),
        }
    }

    pub(super) fn compose(
        &self,
        input: OwnerToCurrentPlacementBoundaryInput<'_, Node, S>,
    ) -> Result<Self, InheritedCurrentGridBaselinePlacementError> {
        if input.parent_grid != self.current_grid {
            return Err(InheritedCurrentGridBaselinePlacementError::OwnershipMismatch);
        }
        let track_count = input
            .parent_span
            .checked_len()
            .filter(|_| input.parent_span.end <= self.tracks.len())
            .ok_or(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange)?;
        if input.parent_first_frame_origins.len() != self.tracks.len()
            || input.parent_last_frame_origins.len() != self.tracks.len()
            || input.current_first_frame_origins.len() != track_count
            || input.current_last_frame_origins.len() != track_count
        {
            return Err(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange);
        }
        if !input.inherited
            || input.parent_axis != self.current_axis
            || input.physical_axis != self.physical_axis
            || input.parent_progression != self.current_progression
        {
            return Err(InheritedCurrentGridBaselinePlacementError::UnusableInheritedMapping);
        }
        if !input.parent_gap.is_finite()
            || !input.current_gap.is_finite()
            || !input.start_mbp.is_finite()
            || !input.end_mbp.is_finite()
            || input
                .parent_first_frame_origins
                .iter()
                .chain(input.parent_last_frame_origins)
                .chain(input.current_first_frame_origins)
                .chain(input.current_last_frame_origins)
                .any(|origin| !origin.is_finite())
        {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }

        let half_gap = (input.current_gap - input.parent_gap) / S::from_f64(2.0);
        if !half_gap.is_finite() {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }
        let mut tracks = Vec::with_capacity(track_count);
        for local in 0..track_count {
            let parent = if input.reversed {
                input.parent_span.end - 1 - local
            } else {
                input.parent_span.start + local
            };
            let parent_entry = self
                .tracks
                .get(parent)
                .copied()
                .ok_or(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange)?;
            let mapped_edge = |role| match (role, input.reversed) {
                (AncestorBaselineRole::First, false) | (AncestorBaselineRole::Last, true) => {
                    BaselineMappedEdge::Start
                }
                (AncestorBaselineRole::Last, false) | (AncestorBaselineRole::First, true) => {
                    BaselineMappedEdge::End
                }
            };
            let boundary_gutter = |role| {
                let crossing = match role {
                    AncestorBaselineRole::First => local > 0,
                    AncestorBaselineRole::Last => local + 1 < track_count,
                };
                if !crossing || half_gap == S::ZERO {
                    S::ZERO
                } else {
                    match mapped_edge(role) {
                        BaselineMappedEdge::Start => half_gap,
                        BaselineMappedEdge::End => -half_gap,
                    }
                }
            };
            let first_frame_translation = parent_entry.first_frame_translation
                + input.current_first_frame_origins[local]
                - input.parent_first_frame_origins[parent];
            let last_frame_translation = parent_entry.last_frame_translation
                + input.current_last_frame_origins[local]
                - input.parent_last_frame_origins[parent];
            let first_gutter_translation = parent_entry.first_gutter_translation
                + boundary_gutter(AncestorBaselineRole::First);
            let last_gutter_translation =
                parent_entry.last_gutter_translation + boundary_gutter(AncestorBaselineRole::Last);
            if [
                first_frame_translation,
                last_frame_translation,
                first_gutter_translation,
                last_gutter_translation,
            ]
            .into_iter()
            .any(|translation| !translation.is_finite())
            {
                return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
            }
            tracks.push(OwnerTrackPlacementEntry {
                owner_track: parent_entry.owner_track,
                first_frame_translation,
                last_frame_translation,
                first_gutter_translation,
                last_gutter_translation,
            });
        }

        Ok(Self {
            owner: self.owner,
            current_grid: input.current_grid,
            current_axis: input.current_axis,
            owner_axis: self.owner_axis,
            physical_axis: self.physical_axis,
            owner_progression: self.owner_progression,
            current_progression: input.current_progression,
            boundary_count: self.boundary_count + 1,
            tracks,
        })
    }

    pub(super) const fn owner(&self) -> Node {
        self.owner
    }

    pub(super) const fn current_grid(&self) -> Node {
        self.current_grid
    }

    pub(super) const fn current_axis(&self) -> GridAxisKind {
        self.current_axis
    }

    pub(super) const fn owner_axis(&self) -> GridAxisKind {
        self.owner_axis
    }

    pub(super) const fn physical_axis(&self) -> PhysicalAxis {
        self.physical_axis
    }

    pub(super) const fn current_progression(&self) -> PhysicalProgression {
        self.current_progression
    }

    pub(super) const fn boundary_count(&self) -> usize {
        self.boundary_count
    }

    pub(super) fn track_count(&self) -> usize {
        self.tracks.len()
    }

    pub(super) fn owner_track_for_local(&self, local: usize) -> Option<usize> {
        self.tracks.get(local).map(|entry| entry.owner_track)
    }

    pub(super) fn translations_for(
        &self,
        local: usize,
        role: AncestorBaselineRole,
    ) -> Option<(S, S)> {
        let entry = self.tracks.get(local).copied()?;
        Some((
            entry.frame_translation(role),
            entry.gutter_translation(role),
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BaselineMappedEdge {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CurrentGridDirectWitness<Node> {
    current_grid: Node,
    item: Node,
    axis: GridAxisKind,
    local_span: GridTrackSpan,
    role: AncestorBaselineRole,
}

impl<Node: Copy> CurrentGridDirectWitness<Node> {
    pub(super) const fn new(
        current_grid: Node,
        item: Node,
        axis: GridAxisKind,
        local_span: GridTrackSpan,
        role: AncestorBaselineRole,
    ) -> Self {
        Self {
            current_grid,
            item,
            axis,
            local_span,
            role,
        }
    }

    #[cfg(test)]
    pub(super) fn set_local_span_for_test(&mut self, local_span: GridTrackSpan) {
        self.local_span = local_span;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct InheritedCurrentGridBaselinePlacement<Node, S: LayoutScalar = Scalar> {
    axis: GridAxisKind,
    physical_axis: PhysicalAxis,
    role: AncestorBaselineRole,
    selected_owner_track: usize,
    selected_local_track: usize,
    immutable_owner_target: S,
    frame_translation: S,
    accumulated_gutter_translation: S,
    translated_target: S,
    mapping_witness: CheckedOwnerToCurrentPlacementMap<Node, S>,
    current_grid_direct_witness: CurrentGridDirectWitness<Node>,
}

#[derive(Clone, Debug)]
pub(super) struct InheritedCurrentGridBaselinePlacementInput<Node, S: LayoutScalar = Scalar> {
    pub(super) axis: GridAxisKind,
    pub(super) physical_axis: PhysicalAxis,
    pub(super) mapping: CheckedOwnerToCurrentPlacementMap<Node, S>,
    pub(super) direct_witness: CurrentGridDirectWitness<Node>,
    pub(super) current_grid: Node,
    pub(super) item: Node,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InheritedCurrentGridBaselinePlacementError {
    AxisMismatch,
    PhysicalAxisMismatch,
    SpanOutOfRange,
    SelectedTrackOutOfRange,
    RoleTargetMismatch,
    OwnershipMismatch,
    UnusableInheritedMapping,
    NonFinite,
}

impl<Node: Copy + PartialEq, S: LayoutScalar> InheritedCurrentGridBaselinePlacement<Node, S> {
    pub(super) fn try_derive(
        group: &AncestorBaselineGroup<Node, S>,
        input: InheritedCurrentGridBaselinePlacementInput<Node, S>,
    ) -> Result<Self, InheritedCurrentGridBaselinePlacementError> {
        let owner_axis_matches = group.axis() == input.mapping.owner_axis();
        let current_axis_matches = input.mapping.current_axis() == input.axis;
        if owner_axis_matches != current_axis_matches {
            return Err(InheritedCurrentGridBaselinePlacementError::AxisMismatch);
        }
        if group.physical_axis() != input.physical_axis {
            return Err(InheritedCurrentGridBaselinePlacementError::PhysicalAxisMismatch);
        }

        let mapping = &input.mapping;
        let track_count = mapping.track_count();
        if track_count == 0
            || mapping
                .tracks
                .iter()
                .any(|entry| entry.owner_track >= group.track_count())
        {
            return Err(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange);
        }
        let selected_local_track = match input.direct_witness.role {
            AncestorBaselineRole::First => input.direct_witness.local_span.start,
            AncestorBaselineRole::Last => input
                .direct_witness
                .local_span
                .end
                .checked_sub(1)
                .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?,
        };
        if input.direct_witness.local_span.checked_len().is_none()
            || input.direct_witness.local_span.end > track_count
            || selected_local_track >= track_count
        {
            return Err(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange);
        }
        let selected_owner_track = mapping
            .owner_track_for_local(selected_local_track)
            .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;

        let role = input.direct_witness.role;
        let target = group
            .target_record_for_progression(
                role,
                selected_owner_track,
                mapping.owner_progression != mapping.current_progression,
            )
            .filter(|target| {
                target.role() == role
                    && target.selected_ancestor_track() == selected_owner_track
                    && target.selected_owner_edge()
                        == match role {
                            AncestorBaselineRole::First => BaselineOwnerEdge::Start,
                            AncestorBaselineRole::Last => BaselineOwnerEdge::End,
                        }
            });
        let target =
            target.ok_or(InheritedCurrentGridBaselinePlacementError::RoleTargetMismatch)?;

        if input.direct_witness.current_grid != input.current_grid
            || input.direct_witness.item != input.item
            || mapping.owner() != group.owner()
            || mapping.current_grid() != input.current_grid
        {
            return Err(InheritedCurrentGridBaselinePlacementError::OwnershipMismatch);
        }
        if mapping.current_axis() != input.axis
            || mapping.owner_axis() != group.axis()
            || mapping.physical_axis() != input.physical_axis
            || input.direct_witness.axis != input.axis
            || (mapping.boundary_count() == 0 && mapping.owner() != mapping.current_grid())
            || (mapping.boundary_count() > 0 && mapping.owner() == mapping.current_grid())
        {
            return Err(InheritedCurrentGridBaselinePlacementError::UnusableInheritedMapping);
        }

        let immutable_owner_target = target.finite_owner_logical_target();
        let (frame_translation, accumulated_gutter_translation) = mapping
            .translations_for(selected_local_track, role)
            .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;
        if !immutable_owner_target.is_finite()
            || !frame_translation.is_finite()
            || !accumulated_gutter_translation.is_finite()
        {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }
        let translated_target =
            immutable_owner_target + frame_translation + accumulated_gutter_translation;
        if !translated_target.is_finite() {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }

        Ok(Self {
            axis: input.axis,
            physical_axis: input.physical_axis,
            role,
            selected_owner_track,
            selected_local_track,
            immutable_owner_target,
            frame_translation,
            accumulated_gutter_translation,
            translated_target,
            mapping_witness: input.mapping,
            current_grid_direct_witness: input.direct_witness,
        })
    }

    #[cfg(test)]
    pub(super) const fn selected_local_track(&self) -> usize {
        self.selected_local_track
    }

    #[cfg(test)]
    pub(super) const fn immutable_owner_target(&self) -> S {
        self.immutable_owner_target
    }

    #[cfg(test)]
    pub(super) const fn frame_translation(&self) -> S {
        self.frame_translation
    }

    #[cfg(test)]
    pub(super) const fn accumulated_gutter_translation(&self) -> S {
        self.accumulated_gutter_translation
    }

    pub(super) const fn translated_target(&self) -> S {
        self.translated_target
    }
}

#[cfg(test)]
type InheritedPlacementPreparationInput<'a, Node, S> = (
    &'a AncestorBaselineGroup<Node, S>,
    InheritedCurrentGridBaselinePlacementInput<Node, S>,
);

#[cfg(test)]
pub(super) fn prepare_inherited_current_grid_baseline_placements<
    Node: Copy + PartialEq,
    S: LayoutScalar,
    const N: usize,
>(
    inputs: [InheritedPlacementPreparationInput<'_, Node, S>; N],
) -> Result<
    Vec<InheritedCurrentGridBaselinePlacement<Node, S>>,
    InheritedCurrentGridBaselinePlacementError,
> {
    inputs
        .into_iter()
        .map(|(group, input)| InheritedCurrentGridBaselinePlacement::try_derive(group, input))
        .collect()
}

#[derive(Clone, Copy)]
pub(super) struct ChildBaselineEnvelopeInput<S: LayoutScalar = Scalar> {
    pub(super) axis: GridAxisKind,
    pub(super) physical_axis: PhysicalAxis,
    pub(super) ancestor_progression_decreasing: bool,
    pub(super) parent_span: GridTrackSpan,
    pub(super) reversed: bool,
    pub(super) start_mbp: S,
    pub(super) end_mbp: S,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: S,
}

impl<S: LayoutScalar> ChildBaselineEnvelopeView<S> {
    pub(super) fn target_for<Node: Copy>(
        &self,
        member: AncestorBaselineMember<Node, S>,
    ) -> Option<S> {
        match member.role() {
            AncestorBaselineRole::First => self.major.get(member.selected_track()),
            AncestorBaselineRole::Last => self.minor.get(member.selected_track()),
        }
        .copied()
        .flatten()
        .map(PhysicalBaseline::coordinate)
    }

    pub(super) fn derive<Node: Copy>(
        group: &AncestorBaselineGroup<Node, S>,
        input: ChildBaselineEnvelopeInput<S>,
        downward_major_translation: &[S],
        downward_minor_translation: &[S],
    ) -> Result<Self, SubgridTrackInheritanceError> {
        if group.physical_axis() != input.physical_axis {
            return Err(SubgridTrackInheritanceError::SpanOutOfRange);
        }
        let mut parent_major = group
            .target_records_for_child_view(input.reversed)
            .map(|track| {
                track.first.map(|target| {
                    PhysicalBaseline::new(input.physical_axis, target.finite_owner_logical_target())
                })
            })
            .collect::<Vec<_>>();
        for (baseline, translation) in parent_major
            .iter_mut()
            .zip(downward_major_translation.iter().copied())
        {
            subtract_baseline(baseline, translation, input.physical_axis);
        }
        let mut parent_minor = group
            .target_records_for_child_view(input.reversed)
            .map(|track| {
                track.last.map(|target| {
                    PhysicalBaseline::new(input.physical_axis, target.finite_owner_logical_target())
                })
            })
            .collect::<Vec<_>>();
        for (baseline, translation) in parent_minor
            .iter_mut()
            .zip(downward_minor_translation.iter().copied())
        {
            subtract_baseline(baseline, -translation, input.physical_axis);
        }
        if input.reversed {
            let (major_translation, minor_translation) = group.reversed_child_view_translations();
            for (baseline, translation) in parent_major
                .iter_mut()
                .zip(major_translation.iter().copied())
            {
                subtract_baseline(baseline, translation, input.physical_axis);
            }
            for (baseline, translation) in parent_minor
                .iter_mut()
                .zip(minor_translation.iter().copied())
            {
                subtract_baseline(baseline, translation, input.physical_axis);
            }
        }
        let (start_mbp, end_mbp, parent_gap) = if input.reversed {
            (S::ZERO, S::ZERO, input.subgrid_gap)
        } else {
            (input.start_mbp, input.end_mbp, input.parent_gap)
        };
        let inherited = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
            parent_major: &parent_major,
            parent_minor: &parent_minor,
            physical_axis: input.physical_axis,
            parent_span: input.parent_span,
            reversed: input.reversed,
            start_mbp,
            end_mbp,
            parent_gap,
            subgrid_gap: input.subgrid_gap,
        })?;
        let edge_translation = if input.axis == GridAxisKind::Column && !input.reversed {
            -(input.start_mbp + input.end_mbp)
        } else {
            S::ZERO
        };
        let mut major = inherited.final_major;
        let mut minor = inherited.final_minor;
        for baseline in &mut major {
            subtract_baseline(baseline, edge_translation, input.physical_axis);
        }
        for baseline in &mut minor {
            subtract_baseline(baseline, edge_translation, input.physical_axis);
        }
        if !input.reversed && input.ancestor_progression_decreasing {
            let half_gap = (input.subgrid_gap - input.parent_gap) / S::from_f64(2.0);
            for baseline in &mut major {
                subtract_baseline(baseline, half_gap, input.physical_axis);
            }
            for baseline in &mut minor {
                subtract_baseline(baseline, -half_gap, input.physical_axis);
            }
        }
        Ok(Self { major, minor })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridTrackInheritanceError {
    EmptyTrackList,
    SpanOutOfRange,
}

pub(super) fn inherit_subgrid_baselines<S: LayoutScalar>(
    input: SubgridBaselineInheritanceInput<'_, S>,
) -> Result<SubgridBaselineInheritanceReport<S>, SubgridTrackInheritanceError> {
    let span_len = input
        .parent_span
        .checked_len()
        .ok_or(SubgridTrackInheritanceError::SpanOutOfRange)?;
    if input.parent_major.is_empty() || input.parent_minor.is_empty() {
        return Err(SubgridTrackInheritanceError::EmptyTrackList);
    }
    if input.parent_major.len() != input.parent_minor.len()
        || input.parent_span.start == 0
        || input.parent_span.end > input.parent_major.len() + 1
        || span_len == 0
    {
        return Err(SubgridTrackInheritanceError::SpanOutOfRange);
    }

    let start_index = input.parent_span.start - 1;
    let end_index = input.parent_span.end - 1;
    let sliced_major = input.parent_major[start_index..end_index].to_vec();
    let sliced_minor = input.parent_minor[start_index..end_index].to_vec();

    let mut after_reversal_major = sliced_major.clone();
    let mut after_reversal_minor = sliced_minor.clone();
    if input.reversed {
        after_reversal_major.reverse();
        after_reversal_minor.reverse();
    }

    let mut after_mbp_major = after_reversal_major.clone();
    if let Some(first_major) = after_mbp_major.first_mut() {
        subtract_baseline(first_major, input.start_mbp, input.physical_axis);
    }

    let mut after_mbp_minor = after_reversal_minor.clone();
    if let Some(last_minor) = after_mbp_minor.last_mut() {
        subtract_baseline(last_minor, input.end_mbp, input.physical_axis);
    }

    let gap_difference = (input.subgrid_gap - input.parent_gap) / S::from_f64(2.0);
    let mut final_major = after_mbp_major.clone();
    let mut final_minor = after_mbp_minor.clone();
    for baseline in final_major.iter_mut().skip(1) {
        subtract_baseline(baseline, gap_difference, input.physical_axis);
    }
    let minor_count = final_minor.len().saturating_sub(1);
    for baseline in final_minor.iter_mut().take(minor_count) {
        subtract_baseline(baseline, gap_difference, input.physical_axis);
    }

    Ok(SubgridBaselineInheritanceReport {
        parent_span: input.parent_span,
        reversed: input.reversed,
        start_mbp: input.start_mbp,
        end_mbp: input.end_mbp,
        parent_gap: input.parent_gap,
        subgrid_gap: input.subgrid_gap,
        gap_difference,
        sliced_major,
        sliced_minor,
        after_reversal_major,
        after_reversal_minor,
        after_mbp_major,
        after_mbp_minor,
        final_major,
        final_minor,
    })
}

pub(super) fn inherit_subgrid_tracks<S: LayoutScalar>(
    input: SubgridTrackInheritanceInput<'_, S>,
) -> Result<SubgridTrackInheritanceReport<S>, SubgridTrackInheritanceError> {
    let span_len = input
        .parent_span
        .checked_len()
        .ok_or(SubgridTrackInheritanceError::SpanOutOfRange)?;
    if input.parent_tracks.is_empty() {
        return Err(SubgridTrackInheritanceError::EmptyTrackList);
    }
    if input.parent_span.start == 0
        || input.parent_span.end > input.parent_tracks.len() + 1
        || span_len == 0
    {
        return Err(SubgridTrackInheritanceError::SpanOutOfRange);
    }

    let start_index = input.parent_span.start - 1;
    let end_index = input.parent_span.end - 1;
    let copied_parent_tracks = input.parent_tracks[start_index..end_index].to_vec();
    let mut after_reversal = copied_parent_tracks.clone();
    if input.reversed {
        after_reversal.reverse();
    }

    let mut start_mbp_removed = after_reversal.clone();
    consume_track_space(
        &mut start_mbp_removed,
        input.start_mbp,
        TrackSpaceEdge::Start,
    );

    let mut end_mbp_removed = start_mbp_removed.clone();
    consume_track_space(&mut end_mbp_removed, input.end_mbp, TrackSpaceEdge::End);

    let resolved_subgrid_gap = input.subgrid_gap.resolve(input.parent_gap);
    let gap_difference = (resolved_subgrid_gap - input.parent_gap) / S::from_f64(2.0);
    let mut final_tracks = end_mbp_removed.clone();
    if final_tracks.len() > 1 {
        for edge in 0..(final_tracks.len() - 1) {
            final_tracks[edge] = (final_tracks[edge] - gap_difference).max(S::ZERO);
            final_tracks[edge + 1] = (final_tracks[edge + 1] - gap_difference).max(S::ZERO);
        }
    }

    Ok(SubgridTrackInheritanceReport {
        parent_span: input.parent_span,
        copied_parent_tracks,
        reversed: input.reversed,
        after_reversal,
        start_mbp_removed,
        end_mbp_removed,
        parent_gap: input.parent_gap,
        subgrid_gap: input.subgrid_gap,
        resolved_subgrid_gap,
        gap_difference,
        final_tracks,
    })
}

fn subtract_baseline<S: LayoutScalar>(
    baseline: &mut Option<PhysicalBaseline<S>>,
    amount: S,
    expected_axis: PhysicalAxis,
) {
    if let Some(current) = baseline
        .as_ref()
        .filter(|baseline| baseline.axis() == expected_axis)
        .copied()
    {
        *baseline = Some(PhysicalBaseline::new(
            current.axis(),
            current.coordinate() - amount,
        ));
    }
}

#[derive(Clone, Copy)]
enum TrackSpaceEdge {
    Start,
    End,
}

fn consume_track_space<S: LayoutScalar>(tracks: &mut [S], mut amount: S, edge: TrackSpaceEdge) {
    match edge {
        TrackSpaceEdge::Start => {
            for track in tracks.iter_mut() {
                if amount <= S::ZERO {
                    break;
                }
                let removed = (*track).min(amount);
                *track = *track - removed;
                amount = amount - removed;
            }
        }
        TrackSpaceEdge::End => {
            for track in tracks.iter_mut().rev() {
                if amount <= S::ZERO {
                    break;
                }
                let removed = (*track).min(amount);
                *track = *track - removed;
                amount = amount - removed;
            }
        }
    }
}

pub(super) fn collect_subgrid_report<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    parent_style: &NodeInputOf<Tree::Scalar>,
) -> GridSubgridReport<<Tree as Traverse>::Node>
where
    Tree: Compute<M>,
{
    let items = tree
        .children(node)
        .map(|child| {
            let child_style = tree.node_input(child);
            SubgridItemReport {
                node: child,
                column: subgrid_axis_report(parent_style, child_style, GridAxisKind::Column),
                row: subgrid_axis_report(parent_style, child_style, GridAxisKind::Row),
            }
        })
        .collect();

    GridSubgridReport { items }
}

pub(super) struct GridSubgridIntrinsicTraversalInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) axis: GridAxisKind,
    pub(super) containing_flow_axes: crate::geometry::FlowAxes,
    pub(super) children: &'a [Node],
    pub(super) placed_areas: &'a [Option<GridArea<S>>],
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) named_columns: &'a NamedGridLines,
    pub(super) named_rows: &'a NamedGridLines,
    pub(super) area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) parent_gap: Size<S>,
    pub(super) column_sizes: &'a [S],
    pub(super) row_sizes: &'a [S],
    pub(super) container_size: Size<Option<S>>,
    pub(super) intrinsic_min_track_facts: IntrinsicMinTrackFacts<'a>,
}

#[expect(
    clippy::type_complexity,
    reason = "nested traversal preserves both session errors and existing traversal outcomes"
)]
pub(super) fn collect_grid_subgrid_intrinsic_traversal<Tree, M>(
    tree: &Tree,
    input: GridSubgridIntrinsicTraversalInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Result<SubgridTraversalReport<<Tree as Traverse>::Node, Tree::Scalar>, SubgridTraversalError>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let mut root_children = Vec::new();
    for ((child, area), item_report) in input
        .children
        .iter()
        .copied()
        .zip(input.placed_areas.iter().copied())
        .zip(input.subgrid_report.items.iter().copied())
    {
        let Some(area) = area else {
            continue;
        };
        let child_style = tree.node_input(child);
        if !is_in_flow_grid_child(child_style) {
            continue;
        }
        let Some(child_axis) =
            inherited_subgrid_axis_for_parent_axis(child_style, item_report, input.axis)
        else {
            continue;
        };
        if !subgrid_requested(child_style, child_axis) {
            continue;
        }
        let area_size = intrinsic_traversal_area_size(
            area,
            input.column_sizes,
            input.row_sizes,
            input.parent_gap,
            input.container_size,
        );
        let Some(child) = subgrid_traversal_child(
            tree,
            child,
            child_style,
            area,
            area_size,
            item_report,
            child_axis,
            input.containing_flow_axes,
            input.parent_gap,
            input.named_columns,
            input.named_rows,
            input.area_facts,
            match input.axis {
                GridAxisKind::Column => child_style.justify_self,
                GridAxisKind::Row => child_style.align_self,
            }
            .unwrap_or(AlignItems::Stretch),
        )?
        else {
            continue;
        };
        root_children.push(child);
    }

    Ok(traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: input.intrinsic_min_track_facts,
        root_children,
    }))
}

#[expect(
    clippy::too_many_arguments,
    reason = "subgrid traversal child creation carries explicit grid layout phase inputs"
)]
#[expect(
    clippy::type_complexity,
    reason = "nested child setup preserves both session errors and optional traversal outcomes"
)]
fn subgrid_traversal_child<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    area: GridArea<Tree::Scalar>,
    area_size: Size<Tree::Scalar>,
    item_report: SubgridItemReport<<Tree as Traverse>::Node>,
    queried_axis: GridAxisKind,
    containing_flow_axes: crate::geometry::FlowAxes,
    parent_gap: Size<Tree::Scalar>,
    parent_named_columns: &NamedGridLines,
    parent_named_rows: &NamedGridLines,
    parent_area_facts: Option<&GridAreaNameFacts>,
    align_self: AlignItems,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Option<SubgridTraversalChild<<Tree as Traverse>::Node, Tree::Scalar>>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let axis_report = match queried_axis {
        GridAxisKind::Column => item_report.column,
        GridAxisKind::Row => item_report.row,
    };
    if !subgrid_requested(style, queried_axis) {
        return Ok(Some(SubgridTraversalChild::Leaf(SubgridTraversalLeaf {
            node,
            span_in_parent: area_span(area, queried_axis),
            available_inline_size: (queried_axis == GridAxisKind::Row)
                .then_some(area_size.width)
                .filter(|width| *width > Tree::Scalar::ZERO),
            available_inline_size_is_known: false,
            align_self,
        })));
    }

    let axis = if axis_report.can_inherit() {
        SubgridTraversalAxis::Inherited
    } else {
        SubgridTraversalAxis::Standalone
    };
    if axis == SubgridTraversalAxis::Standalone {
        return Ok(None);
    }
    let Some(mapping) = axis_report.mapping.ok() else {
        return Ok(None);
    };
    let reversed = mapping.reversed;
    let parent_axis = mapping.parent_axis;
    let span_in_parent = area_span(area, parent_axis);
    let parent_axis_gap = axis_size(parent_gap, parent_axis);
    let physical_area_size = grid_area_physical_size(
        containing_flow_axes,
        LogicalSizeOf::new(area_size.width, area_size.height),
    );
    let area_basis = physical_area_size.map(Some);
    let resolved_margin = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.margin, area_basis, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, node)?
        .map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
    let resolved_border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, area_basis, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let resolved_padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, area_basis, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let current_flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
    let (margins, border, padding) = traversal_axis_edges(
        queried_axis,
        current_flow_axes,
        resolved_margin,
        resolved_border,
        resolved_padding,
    );
    let physical_content_box_size = (physical_area_size
        - resolved_margin.sum_axes()
        - resolved_border.sum_axes()
        - resolved_padding.sum_axes())
    .max(Size::ZERO);
    let logical_content_box_size = current_flow_axes.logical_size(physical_content_box_size);
    let content_box_size = Size::new(
        logical_content_box_size.inline,
        logical_content_box_size.block,
    );
    let subgrid_gap = Size::new(
        resolved_subgrid_axis_gap(
            style,
            GridAxisKind::Column,
            item_report.column,
            parent_gap,
            content_box_size,
        )
        .map_err(|status| crate::compute::value_resolution_error(node, status))?,
        resolved_subgrid_axis_gap(
            style,
            GridAxisKind::Row,
            item_report.row,
            parent_gap,
            content_box_size,
        )
        .map_err(|status| crate::compute::value_resolution_error(node, status))?,
    );
    let subgrid_axis_gap = axis_size(subgrid_gap, queried_axis);
    let Some((children, queried_axis_fully_inherited)) = subgrid_traversal_children(
        tree,
        node,
        style,
        area,
        content_box_size,
        item_report,
        queried_axis,
        containing_flow_axes,
        subgrid_gap,
        parent_named_columns,
        parent_named_rows,
        parent_area_facts,
    )?
    else {
        return Ok(None);
    };

    Ok(Some(SubgridTraversalChild::Subgrid(SubgridTraversalNode {
        node,
        axis,
        reversed,
        span_in_parent,
        available_inline_size: (queried_axis == GridAxisKind::Row)
            .then_some(content_box_size.width)
            .filter(|width| *width > Tree::Scalar::ZERO),
        available_inline_size_is_known: queried_axis == GridAxisKind::Row
            && track_components_have_percent_sizing(&style.grid_template_columns),
        queried_axis_fully_inherited,
        margins,
        border,
        padding,
        parent_gap: parent_axis_gap,
        subgrid_gap: subgrid_axis_gap,
        children,
    })))
}

#[expect(
    clippy::too_many_arguments,
    reason = "subgrid traversal recursion preserves retained report and oracle parity inputs"
)]
#[expect(
    clippy::type_complexity,
    reason = "nested traversal preserves the session error envelope and child report"
)]
fn subgrid_traversal_children<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    area: GridArea<Tree::Scalar>,
    content_box_size: Size<Tree::Scalar>,
    item_report: SubgridItemReport<<Tree as Traverse>::Node>,
    queried_axis: GridAxisKind,
    containing_flow_axes: crate::geometry::FlowAxes,
    gap: Size<Tree::Scalar>,
    parent_named_columns: &NamedGridLines,
    parent_named_rows: &NamedGridLines,
    parent_area_facts: Option<&GridAreaNameFacts>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Option<(
        SubgridTraversalChildren<<Tree as Traverse>::Node, Tree::Scalar>,
        bool,
    )>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let parent_context = GridParentContext::<Tree::Scalar, <Tree as Traverse>::Node> {
        columns: intrinsic_subgrid_axis_parent_context(
            item_report.column,
            area,
            gap,
            parent_named_columns,
            parent_named_rows,
            parent_area_facts,
        ),
        rows: intrinsic_subgrid_axis_parent_context(
            item_report.row,
            area,
            gap,
            parent_named_columns,
            parent_named_rows,
            parent_area_facts,
        ),
    };
    let available = Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT);
    let constants = Constants::new::<Tree, M>(
        tree,
        node,
        style,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                containing_flow_axes,
                crate::ParentFormattingContext::Grid,
            ),
            available,
        ),
    )?;
    let initialized = initialize_grid_tracks::<Tree, M>(
        tree,
        node,
        style,
        &constants,
        &parent_context,
        available,
    )?;
    let column_count = initialized.column_tracks.len();
    let row_count = initialized.row_tracks.len();
    if column_count == 0 || row_count == 0 {
        return Ok(Some((Vec::new(), true)));
    }

    let zero_columns = vec![Tree::Scalar::ZERO; column_count];
    let zero_rows = vec![Tree::Scalar::ZERO; row_count];
    let traversal_columns = traversal_child_area_tracks(
        parent_context.columns.as_ref(),
        &initialized.column_tracks,
        content_box_size.width,
        gap.width,
        style.justify_content.unwrap_or(AlignContent::Stretch),
    );
    let traversal_rows = parent_context
        .rows
        .as_ref()
        .map_or_else(|| zero_rows.clone(), |axis| axis.tracks.clone());
    let inherited_queried_track_count = match queried_axis {
        GridAxisKind::Column => parent_context
            .columns
            .as_ref()
            .map(|axis| axis.tracks.len())
            .unwrap_or(column_count),
        GridAxisKind::Row => parent_context
            .rows
            .as_ref()
            .map(|axis| axis.tracks.len())
            .unwrap_or(row_count),
    };
    let children = tree.children(node).collect::<Vec<_>>();
    let placed_areas = resolve_grid_child_areas(ResolveGridChildAreasInput {
        children: &children,
        placements: &initialized.placements,
        style,
        columns: &zero_columns,
        rows: &zero_rows,
        gap: LogicalSizeOf::new(gap.width, gap.height),
        lines: initialized.context.lines,
    });
    let queried_axis_fully_inherited =
        placed_areas
            .iter()
            .flatten()
            .all(|area| match queried_axis {
                GridAxisKind::Column => area.column_end <= inherited_queried_track_count,
                GridAxisKind::Row => area.row_end <= inherited_queried_track_count,
            });
    let mut traversal_children = Vec::new();
    for ((child, child_area), child_report) in children
        .into_iter()
        .zip(placed_areas)
        .zip(initialized.subgrid_report.items)
    {
        let Some(child_area) = child_area else {
            continue;
        };
        let child_style = tree.node_input(child);
        if !is_in_flow_grid_child(child_style) {
            continue;
        }
        let child_axis =
            traversal_child_axis_for_parent_axis(child_style, child_report, queried_axis);
        let child_area_size = intrinsic_traversal_area_size(
            child_area,
            &traversal_columns,
            &traversal_rows,
            gap,
            Size::new(Some(content_box_size.width), Some(content_box_size.height)),
        );
        if let Some(child) = subgrid_traversal_child(
            tree,
            child,
            child_style,
            child_area,
            child_area_size,
            child_report,
            child_axis,
            crate::geometry::FlowAxes::new(style.writing_mode, style.direction),
            gap,
            &initialized.context.named_columns,
            &initialized.context.named_rows,
            initialized.context.area_facts.as_ref(),
            match queried_axis {
                GridAxisKind::Column => child_style.justify_self.or(style.justify_items),
                GridAxisKind::Row => child_style.align_self.or(style.align_items),
            }
            .unwrap_or(AlignItems::Stretch),
        )? {
            traversal_children.push(child);
        }
    }

    Ok(Some((traversal_children, queried_axis_fully_inherited)))
}

fn inherited_subgrid_axis_for_parent_axis<Node, S: LayoutScalar>(
    style: &NodeInputOf<S>,
    item_report: SubgridItemReport<Node>,
    parent_axis: GridAxisKind,
) -> Option<GridAxisKind>
where
    Node: Copy,
{
    [GridAxisKind::Column, GridAxisKind::Row]
        .into_iter()
        .find(|axis| {
            if !subgrid_requested(style, *axis) {
                return false;
            }
            let report = match axis {
                GridAxisKind::Column => item_report.column,
                GridAxisKind::Row => item_report.row,
            };
            report
                .mapping
                .is_ok_and(|mapping| report.can_inherit() && mapping.parent_axis == parent_axis)
        })
}

fn traversal_child_axis_for_parent_axis<Node, S: LayoutScalar>(
    style: &NodeInputOf<S>,
    item_report: SubgridItemReport<Node>,
    parent_axis: GridAxisKind,
) -> GridAxisKind
where
    Node: Copy,
{
    inherited_subgrid_axis_for_parent_axis(style, item_report, parent_axis).unwrap_or(parent_axis)
}

fn traversal_child_area_tracks<Node, S: LayoutScalar>(
    inherited: Option<&InheritedGridAxis<S, Node>>,
    tracks: &[TrackSizingOf<S>],
    content_width: S,
    gap: S,
    alignment: AlignContent,
) -> Vec<S> {
    if let Some(axis) = inherited {
        return axis.tracks.clone();
    }

    let intrinsic_sizes = vec![S::ZERO; tracks.len()];
    resolve_inline_tracks(InlineTrackInput {
        tracks,
        basis: Some(content_width),
        definite_size: Some(content_width),
        available_size: Some(content_width),
        gap,
        alignment,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &intrinsic_sizes,
        max_intrinsic_sizes: &intrinsic_sizes,
    })
}

pub(super) fn intrinsic_subgrid_axis_parent_context<Node, S: LayoutScalar>(
    report: SubgridAxisReport,
    area: GridArea<S>,
    gap: Size<S>,
    named_columns: &NamedGridLines,
    named_rows: &NamedGridLines,
    area_facts: Option<&GridAreaNameFacts>,
) -> Option<InheritedGridAxis<S, Node>> {
    if !report.can_inherit() {
        return None;
    }
    let mapping = report.mapping.ok()?;
    let (parent_start, parent_end) = match mapping.parent_axis {
        GridAxisKind::Column => (area.column, area.column_end),
        GridAxisKind::Row => (area.row, area.row_end),
    };
    let (parent_gap, named_lines) = match mapping.parent_axis {
        GridAxisKind::Column => (gap.width, named_columns),
        GridAxisKind::Row => (gap.height, named_rows),
    };
    let track_count = parent_end.saturating_sub(parent_start);

    Some(InheritedGridAxis {
        offset: S::ZERO,
        gap: parent_gap,
        tracks: vec![S::ZERO; track_count],
        geometry: UsedGridAxisGeometryOf::new(
            vec![S::ZERO; track_count],
            vec![false; track_count],
            parent_gap,
        ),
        named_lines: named_lines.clone(),
        area_facts: area_facts
            .filter(|facts| facts.is_valid_for_axis(mapping.parent_axis))
            .cloned(),
        major_baselines: vec![None; track_count],
        minor_baselines: vec![None; track_count],
        owner_baseline_targets: None,
        parent_start,
        parent_end,
        reversed: mapping.reversed,
    })
}

fn area_span<S: LayoutScalar>(area: GridArea<S>, axis: GridAxisKind) -> GridTrackSpan {
    match axis {
        GridAxisKind::Column => GridTrackSpan::new(area.column + 1, area.column_end + 1),
        GridAxisKind::Row => GridTrackSpan::new(area.row + 1, area.row_end + 1),
    }
}

fn axis_size<S: LayoutScalar>(size: Size<S>, axis: GridAxisKind) -> S {
    match axis {
        GridAxisKind::Column => size.width,
        GridAxisKind::Row => size.height,
    }
}

fn resolved_subgrid_axis_gap<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axis: GridAxisKind,
    report: SubgridAxisReport,
    parent_gap: Size<S>,
    content_box_size: Size<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    let logical_gap =
        crate::geometry::FlowAxes::new(style.writing_mode, style.direction).logical_size(style.gap);
    let gap = match axis {
        GridAxisKind::Column => logical_gap.inline,
        GridAxisKind::Row => logical_gap.block,
    };
    match gap {
        LengthOf::Normal => {
            let parent_axis = report
                .mapping
                .ok()
                .filter(|_| report.can_inherit())
                .map_or(axis, |mapping| mapping.parent_axis);
            Ok(axis_size(parent_gap, parent_axis))
        }
        gap => resolve_length_or_zero(gap, Some(axis_size(content_box_size, axis))),
    }
}

fn traversal_axis_edges<S: LayoutScalar>(
    axis: GridAxisKind,
    flow_axes: crate::geometry::FlowAxes,
    margin: Edges<S>,
    border: Edges<S>,
    padding: Edges<S>,
) -> (
    SubgridAxisEdges<S>,
    SubgridAxisEdges<S>,
    SubgridAxisEdges<S>,
) {
    let margin = flow_axes.logical_edges(margin);
    let border = flow_axes.logical_edges(border);
    let padding = flow_axes.logical_edges(padding);
    match axis {
        GridAxisKind::Column => (
            SubgridAxisEdges {
                start: margin.inline_start,
                end: margin.inline_end,
            },
            SubgridAxisEdges {
                start: border.inline_start,
                end: border.inline_end,
            },
            SubgridAxisEdges {
                start: padding.inline_start,
                end: padding.inline_end,
            },
        ),
        GridAxisKind::Row => (
            SubgridAxisEdges {
                start: margin.block_start,
                end: margin.block_end,
            },
            SubgridAxisEdges {
                start: border.block_start,
                end: border.block_end,
            },
            SubgridAxisEdges {
                start: padding.block_start,
                end: padding.block_end,
            },
        ),
    }
}

fn intrinsic_traversal_area_size<S: LayoutScalar>(
    area: GridArea<S>,
    columns: &[S],
    rows: &[S],
    gap: Size<S>,
    container_size: Size<Option<S>>,
) -> Size<S> {
    Size::new(
        intrinsic_traversal_axis_area_size(
            area.column,
            area.column_end,
            columns,
            gap.width,
            container_size.width,
        ),
        intrinsic_traversal_axis_area_size(
            area.row,
            area.row_end,
            rows,
            gap.height,
            container_size.height,
        ),
    )
}

fn intrinsic_traversal_axis_area_size<S: LayoutScalar>(
    start: usize,
    end: usize,
    tracks: &[S],
    gap: S,
    definite_container_size: Option<S>,
) -> S {
    if tracks.is_empty() {
        return definite_container_size.unwrap_or(S::ZERO);
    }
    if start >= tracks.len() {
        return S::ZERO;
    }
    if start == 0 && end == tracks.len() {
        definite_container_size.unwrap_or_else(|| track_span_sum(tracks, start, end, gap))
    } else {
        track_span_sum(tracks, start, end, gap)
    }
}

pub(super) fn subgrid_eligibility<S: LayoutScalar>(
    input: SubgridEligibilityInput<'_, S>,
) -> SubgridEligibility {
    let reason = if !subgrid_requested(input.child_style, input.axis) {
        Some(SubgridIneligibleReason::NotRequested)
    } else if !input.has_parent_grid {
        Some(SubgridIneligibleReason::NoParentGrid)
    } else if establishes_independent_formatting_context(input.child_style) {
        Some(SubgridIneligibleReason::IndependentFormattingContext)
    } else if excluded_from_normal_layout(input.child_style) {
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    } else if !subgrid_container_display_supported(input.child_style.display) {
        Some(SubgridIneligibleReason::UnsupportedDisplay)
    } else if parent_is_lanes_in_resolved_axis(input.parent_style, input.axis) {
        Some(SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    } else {
        None
    };

    SubgridEligibility {
        eligible: reason.is_none(),
        reason,
    }
}

pub(super) fn subgrid_axis_report<S: LayoutScalar>(
    parent_style: &NodeInputOf<S>,
    child_style: &NodeInputOf<S>,
    axis: GridAxisKind,
) -> SubgridAxisReport {
    SubgridAxisReport {
        mapping: map_grid_axis(GridAxisMappingInput {
            queried_axis: axis,
            parent_style,
            child_style,
        }),
        eligibility: subgrid_eligibility(SubgridEligibilityInput {
            axis,
            parent_style,
            has_parent_grid: true,
            child_style,
        }),
    }
}

fn subgrid_requested<S: LayoutScalar>(style: &NodeInputOf<S>, axis: GridAxisKind) -> bool {
    let components = match axis {
        GridAxisKind::Column => &style.grid_template_columns,
        GridAxisKind::Row => &style.grid_template_rows,
    };

    components
        .iter()
        .any(|component| matches!(component, TrackComponentOf::Subgrid(_)))
}

const fn subgrid_container_display_supported(display: Display) -> bool {
    display.establishes_grid_formatting_context()
}

const fn excluded_from_normal_layout<S: LayoutScalar>(style: &NodeInputOf<S>) -> bool {
    matches!(style.display, Display::None) || matches!(style.position, Position::Absolute)
}

const fn establishes_independent_formatting_context<S: LayoutScalar>(
    _style: &NodeInputOf<S>,
) -> bool {
    false
}

fn parent_is_lanes_in_resolved_axis<S: LayoutScalar>(
    parent_style: &NodeInputOf<S>,
    axis: GridAxisKind,
) -> bool {
    parent_style
        .display
        .establishes_grid_lanes_formatting_context()
        && lane_axis(parent_style.grid_auto_flow) == axis
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use super::*;
    use crate::{
        Compute, ComputeInput, ComputeOutput, DefaultScalar, LayoutErrorKind, LayoutErrorSite,
        LayoutInput, LayoutInvalidInput, LayoutOperation, LayoutResultOf, LengthPercentageOf,
        NodeInput, NodeOutput, PreferredSize, Size, SubgridTrack, TrackComponent, Traverse,
    };

    struct TraversalTree {
        input: NodeInput,
    }

    impl Traverse for TraversalTree {
        type Node = u32;
        type Scalar = DefaultScalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("traversal test tree has no children")
        }
    }

    impl Compute<Infallible> for TraversalTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.input
        }

        fn layout_input(&self, _node: Self::Node) -> LayoutInput {
            LayoutInput::box_input(self.input.clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> LayoutResultOf<Self::Node, ComputeOutput, Self::Scalar, Infallible> {
            unreachable!("subgrid traversal setup must not compute children")
        }
    }

    fn assert_resolved_subgrid_axis_gap_uses_node_logical_axes<S: LayoutScalar>() {
        for (writing_mode, direction) in [
            (crate::WritingMode::VerticalRl, crate::Direction::Ltr),
            (crate::WritingMode::SidewaysLr, crate::Direction::Rtl),
        ] {
            let style = NodeInputOf::<S> {
                display: Display::Grid,
                writing_mode,
                direction,
                gap: Size::new(
                    LengthOf::px(S::from_f64(7.0)),
                    LengthOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            };
            let parent_style = NodeInputOf::<S> {
                display: Display::Grid,
                ..NodeInputOf::default()
            };

            for (axis, expected) in [
                (GridAxisKind::Column, S::from_f64(11.0)),
                (GridAxisKind::Row, S::from_f64(7.0)),
            ] {
                let actual = resolved_subgrid_axis_gap(
                    &style,
                    axis,
                    subgrid_axis_report(&parent_style, &style, axis),
                    Size::ZERO,
                    Size::splat(S::from_f64(100.0)),
                )
                .expect("fixed gap should resolve");

                assert_eq!(
                    actual, expected,
                    "{writing_mode:?} {direction:?} {axis:?} must select the node's logical gap"
                );
            }
        }
    }

    #[test]
    fn resolved_subgrid_axis_gap_uses_node_logical_axes_f32() {
        assert_resolved_subgrid_axis_gap_uses_node_logical_axes::<f32>();
    }

    #[test]
    fn resolved_subgrid_axis_gap_uses_node_logical_axes_f64() {
        assert_resolved_subgrid_axis_gap_uses_node_logical_axes::<f64>();
    }

    #[test]
    fn subgrid_traversal_propagates_track_initialization_failure() {
        let overflowing =
            LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
        let parent_style = NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        };
        let style = NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::Subgrid(SubgridTrack::new(vec![]))],
            grid_template_rows: vec![TrackComponent::from(overflowing)],
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(f32::MAX)),
            ..NodeInput::default()
        };
        let tree = TraversalTree {
            input: style.clone(),
        };
        let item_report = SubgridItemReport {
            node: 0,
            column: subgrid_axis_report(&parent_style, &style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &style, GridAxisKind::Row),
        };
        let named_columns = NamedGridLines::new(GridAxisKind::Column, 1);
        let named_rows = NamedGridLines::new(GridAxisKind::Row, 1);

        let error = subgrid_traversal_children::<TraversalTree, Infallible>(
            &tree,
            0,
            &style,
            GridArea {
                column: 0,
                row: 0,
                column_end: 1,
                row_end: 1,
                size: LogicalSizeOf::new(20.0, 20.0),
            },
            Size::new(20.0, 20.0),
            item_report,
            GridAxisKind::Column,
            crate::geometry::FlowAxes::new(style.writing_mode, style.direction),
            Size::ZERO,
            &named_columns,
            &named_rows,
            None,
        )
        .unwrap_err();

        assert_eq!(error.site(), LayoutErrorSite::Node(0));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert!(matches!(
            error.kind(),
            LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
        ));
    }
}
