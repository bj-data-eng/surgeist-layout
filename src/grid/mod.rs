use super::{
    AlignContent, AlignItems, AvailableOf, BaselinesOf, BoxSizing, Compute, ComputeInputOf,
    ComputeOutputOf, DefaultScalar, Direction, Display, Edges, GridAutoFlow, GridPlacement,
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf, LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionStatus, MaxTrackSizingOf,
    MinSizeOf, MinTrackSizingOf, NodeInputOf, NodeOutputOf, Overflow, Point, Position,
    PreferredSizeOf, RequestedAxis, RunMode, Scalar, Size, SizingAlgorithm, SizingMode,
    TrackComponentOf, TrackRepeat, TrackSizingOf, Traverse,
};
use crate::compute::{
    EdgesResultExt, ResolvedPreferredSize, SizeResultExt, layout_own_geometry_error,
    resolve_maximum_optional, resolve_minimum_optional, resolve_preferred_optional,
    resolve_preferred_sizing, sizing_resolution_error,
};
use crate::geometry::{LogicalAxis, LogicalSizeOf, PhysicalAxis};
use crate::layout_math::{
    OptionalSizeExt, UncheckedOptionalSizeSubExt, resolution_optional, resolution_or_zero,
    resolve_containing_padding_border,
};
use crate::node_input::item_order_permutation;
use crate::output::PhysicalBaseline;
use crate::scroll::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, CanonicalScrollGeometrySourceOf,
    ClipMarginSourceOf, OptimalRegionInsetsOf, ScrollOriginAxes, ScrollOriginProgression,
    ScrollbarReservationOf, canonical_scroll_box_from_source,
    canonical_scroll_geometry_from_source, content_box_inset_with_scrollbar,
};

mod alignment;
mod axis;
mod child;
mod lanes;
mod named;
mod placement;
mod subgrid;
#[cfg(test)]
#[path = "../grid_tests.rs"]
mod tests;
mod topology;
mod tracks;

use alignment::*;
pub use axis::GridAxisKind;
use axis::{GridAxisMappingError, GridAxisMappingInput, GridAxisMappingReport, map_grid_axis};
use child::*;
pub use lanes::{
    DefiniteLaneIntrinsicItem, DefiniteLaneIntrinsicItemOf, IndefiniteLaneContributionGroup,
    IndefiniteLaneContributionGroupOf, LaneContributionFacts, LaneContributionFactsOf,
    LaneIntrinsicItem, LaneIntrinsicItemKind, LaneIntrinsicItemOf, LaneIntrinsicSizingInput,
    LaneIntrinsicSizingInputOf, LaneIntrinsicSizingReport, LaneIntrinsicSizingReportOf, LaneItem,
    LaneItemOf, LaneItemOffset, LaneItemOffsetOf, LanePlacementError, LanePlacementInput,
    LanePlacementInputOf, LanePlacementReport, LanePlacementReportOf, LaneTrackSpan,
    LaneTrackSpanLength, grid_axis_for_lanes, lane_axis, lane_intrinsic_sizing, place_lanes,
};
use lanes::{
    GridLanesLayoutInput, LaneIntrinsicTrackSizeInput, apply_grid_lanes_auto_fit_policy,
    column_flow_for_grid_lanes, grid_axis_for_grid_lanes, lane_intrinsic_track_sizes,
    layout_grid_lanes_children, resolve_grid_lanes_placement_with_resolved_tracks,
};
use named::{
    GridAreaNameFacts, NamedGridError, NamedGridLines, build_grid_named_context_with_report,
    empty_grid_named_context, resolve_grid_placement_or_auto_with_report,
    resolve_subgrid_placement,
};
pub use named::{NamedGridErrorReport, NamedGridReport};
use placement::*;
use subgrid::*;
use topology::{ExpandedGridTopology, ExpandedGridTopologyInput};
use tracks::*;

pub struct GridComputationOf<S: LayoutScalar = DefaultScalar> {
    output: ComputeOutputOf<S>,
    report: GridComputationReport,
}

pub type GridComputation = GridComputationOf<DefaultScalar>;

impl<S: LayoutScalar> GridComputationOf<S> {
    pub fn output(&self) -> &ComputeOutputOf<S> {
        &self.output
    }

    pub fn report(&self) -> &GridComputationReport {
        &self.report
    }

    pub fn into_parts(self) -> (ComputeOutputOf<S>, GridComputationReport) {
        (self.output, self.report)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GridComputationReport {
    named_grid: NamedGridReport,
}

impl GridComputationReport {
    pub fn named_grid(&self) -> &NamedGridReport {
        &self.named_grid
    }

    pub fn named_grid_errors(&self) -> &[NamedGridErrorReport] {
        self.named_grid.errors()
    }

    pub fn is_empty(&self) -> bool {
        self.named_grid.is_empty()
    }

    fn merge_named_grid(&mut self, report: NamedGridReport) {
        self.named_grid.extend(report);
    }
}

pub(crate) fn compute_grid<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    Ok(compute_grid_with_report(tree, node, input)?.into_parts().0)
}

pub(crate) fn compute_grid_with_report<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridComputationOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let mut pass_input = input;
    loop {
        let result = compute_grid_with_context_result(
            tree,
            node,
            pass_input,
            GridParentContext::none(),
            GridMeasurementBoundary::Ordinary,
        )?;
        if !input.run_mode().is_perform_layout() {
            return Ok(GridComputationOf {
                output: result.output,
                report: result.report,
            });
        }
        let Some(geometry) = result.output.scroll_geometry else {
            return Ok(GridComputationOf {
                output: result.output,
                report: result.report,
            });
        };
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || !crate::scroll::settled_auto_scrollbars_change_available_geometry(
                geometry, next_state,
            )
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?
        {
            return Ok(GridComputationOf {
                output: result.output,
                report: result.report,
            });
        }
        pass_input = input.with_settled_auto_scrollbars(next_state);
    }
}

struct GridComputeResult<S: LayoutScalar = Scalar> {
    output: ComputeOutputOf<S>,
    report: GridComputationReport,
    baseline_groups: GridBaselineGroups<S>,
}

impl<S: LayoutScalar> GridComputeResult<S> {
    fn from_output(output: ComputeOutputOf<S>) -> Self {
        Self {
            output,
            report: GridComputationReport::default(),
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
        }
    }
}

fn sizing_algorithm_for_grid_display(display: Display) -> SizingAlgorithm {
    if display.inner_display() == Display::GridLanes {
        SizingAlgorithm::GridLanes
    } else {
        SizingAlgorithm::Grid
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StandaloneIntrinsicMinimum {
    MinContent,
    MaxContent,
}

impl StandaloneIntrinsicMinimum {
    fn from_minimum<S: LayoutScalar>(minimum: &MinSizeOf<S>) -> Option<Self> {
        if minimum.is_min_content() {
            Some(Self::MinContent)
        } else if minimum.is_max_content() {
            Some(Self::MaxContent)
        } else {
            None
        }
    }

    const fn available<S: LayoutScalar>(self) -> AvailableOf<S> {
        match self {
            Self::MinContent => AvailableOf::MIN_CONTENT,
            Self::MaxContent => AvailableOf::MAX_CONTENT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GridMeasurementBoundary<S: LayoutScalar> {
    Ordinary,
    StandaloneIntrinsicMinimumProbe(Size<bool>),
    Standalone(Size<Option<S>>),
}

fn standalone_intrinsic_minimum_phases<S: LayoutScalar, Node>(
    style: &NodeInputOf<S>,
    parent_context: &GridParentContext<S, Node>,
) -> Size<Option<StandaloneIntrinsicMinimum>> {
    let style_flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
    let standalone_physical_axes = style_flow_axes.physical_size(LogicalSizeOf::new(
        parent_context.columns.is_none(),
        parent_context.rows.is_none(),
    ));
    Size::new(
        standalone_physical_axes
            .width
            .then(|| StandaloneIntrinsicMinimum::from_minimum(&style.min_size.width))
            .flatten(),
        standalone_physical_axes
            .height
            .then(|| StandaloneIntrinsicMinimum::from_minimum(&style.min_size.height))
            .flatten(),
    )
}

fn standalone_intrinsic_minimum_probe_style<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axes: Size<bool>,
) -> NodeInputOf<S> {
    let mut probe_style = style.clone();
    if axes.width {
        probe_style.min_size.width = MinSizeOf::AUTO;
    }
    if axes.height {
        probe_style.min_size.height = MinSizeOf::AUTO;
    }
    probe_style
}

fn intrinsic_container_available<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    sizing_flow_axes: crate::geometry::FlowAxes,
    parent: Size<Option<S>>,
    available: Size<AvailableOf<S>>,
) -> Result<LogicalSizeOf<AvailableOf<S>>, crate::compute::SizingResolutionError<S>> {
    let algorithm = sizing_algorithm_for_grid_display(style.display);
    let style_size = sizing_flow_axes.logical_size(Size::new(
        resolve_preferred_sizing(
            &style.size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            parent.width,
            true,
        )?,
        resolve_preferred_sizing(
            &style.size.height,
            algorithm,
            PhysicalAxis::Vertical,
            parent.height,
            true,
        )?,
    ));
    let available = sizing_flow_axes.logical_size(available);
    let max_size = sizing_flow_axes.logical_size(constants.node_max_size);
    let content_box_inset_size =
        sizing_flow_axes.logical_size(constants.content_box_inset.sum_axes());
    let max_inner_size = LogicalSizeOf::new(
        max_size
            .inline
            .map(|max| max - content_box_inset_size.inline),
        max_size.block.map(|max| max - content_box_inset_size.block),
    );
    Ok(LogicalSizeOf::new(
        intrinsic_available_for_dimension(style_size.inline).unwrap_or_else(|| {
            intrinsic_available_for_axis(available.inline, max_inner_size.inline)
        }),
        intrinsic_available_for_dimension(style_size.block)
            .unwrap_or_else(|| intrinsic_available_for_axis(available.block, max_inner_size.block)),
    ))
}

fn intrinsic_available_for_dimension<S: LayoutScalar>(
    dimension: ResolvedPreferredSize<S>,
) -> Option<AvailableOf<S>> {
    match dimension {
        ResolvedPreferredSize::MinContent => Some(AvailableOf::MIN_CONTENT),
        ResolvedPreferredSize::MaxContent => Some(AvailableOf::MAX_CONTENT),
        ResolvedPreferredSize::Auto | ResolvedPreferredSize::Definite(_) => None,
    }
}

fn intrinsic_available_for_axis<S: LayoutScalar>(
    available: AvailableOf<S>,
    max_size: Option<S>,
) -> AvailableOf<S> {
    match (available, max_size) {
        (AvailableOf::MaxContent, Some(max_size)) => AvailableOf::Definite(max_size.max(S::ZERO)),
        (available, _) => available,
    }
}

fn intrinsic_available_size_for_axis<S: LayoutScalar>(
    available: AvailableOf<S>,
    max_size: Option<S>,
) -> Option<S> {
    match available {
        AvailableOf::Definite(value) => Some(value),
        AvailableOf::MaxContent => max_size,
        AvailableOf::MinContent => None,
    }
}

fn intrinsic_max_available<S: LayoutScalar>(
    constants: &Constants<S>,
    sizing_flow_axes: crate::geometry::FlowAxes,
    available: Size<AvailableOf<S>>,
) -> LogicalSizeOf<bool> {
    let available = sizing_flow_axes.logical_size(available);
    let max_size = sizing_flow_axes.logical_size(constants.node_max_size);
    LogicalSizeOf::new(
        available.inline == AvailableOf::MaxContent && max_size.inline.is_some(),
        available.block == AvailableOf::MaxContent && max_size.block.is_some(),
    )
}

fn compute_grid_with_context<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(node).clone();
    let intrinsic_minimum = standalone_intrinsic_minimum_phases(&style, &parent_context);
    compute_grid_with_context_and_standalone_intrinsic_minimum(
        tree,
        node,
        input,
        parent_context,
        intrinsic_minimum,
    )
}

fn compute_grid_with_context_and_standalone_intrinsic_minimum<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
    intrinsic_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    Ok(
        compute_grid_with_context_settled_and_standalone_intrinsic_minimum(
            tree,
            node,
            input,
            parent_context,
            intrinsic_minimum,
        )?
        .output,
    )
}

fn compute_grid_with_context_settled_and_standalone_intrinsic_minimum<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
    intrinsic_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridComputeResult<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(node).clone();
    let contextual_minimum = standalone_intrinsic_minimum_phases(&style, &parent_context);
    let intrinsic_minimum = Size::new(
        intrinsic_minimum.width.or(contextual_minimum.width),
        intrinsic_minimum.height.or(contextual_minimum.height),
    );
    if intrinsic_minimum.width.is_some() || intrinsic_minimum.height.is_some() {
        compute_standalone_grid_with_context_settled(
            tree,
            node,
            input,
            parent_context,
            intrinsic_minimum,
        )
    } else {
        compute_grid_with_context_settled(
            tree,
            node,
            input,
            parent_context,
            GridMeasurementBoundary::Ordinary,
        )
    }
}

fn compute_standalone_grid_with_context<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(node).clone();
    let intrinsic_minimum = standalone_intrinsic_minimum_phases(&style, &parent_context);
    Ok(compute_standalone_grid_with_context_settled(
        tree,
        node,
        input,
        parent_context,
        intrinsic_minimum,
    )?
    .output)
}

fn compute_standalone_grid_with_context_settled<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
    intrinsic_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridComputeResult<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let intrinsic_axes = intrinsic_minimum.map(|minimum| minimum.is_some());
    let resolved_intrinsic_minimum = if intrinsic_axes.width || intrinsic_axes.height {
        let available = Size::new(
            intrinsic_minimum.width.map_or(
                input.available().width,
                StandaloneIntrinsicMinimum::available,
            ),
            intrinsic_minimum.height.map_or(
                input.available().height,
                StandaloneIntrinsicMinimum::available,
            ),
        );
        let known = Size::new(
            if intrinsic_axes.width {
                None
            } else {
                input.known().width
            },
            if intrinsic_axes.height {
                None
            } else {
                input.known().height
            },
        );
        let probe_input = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            input.sizing_mode(),
            input.requested_axis(),
            known,
            input.parent(),
            input.containing_layout_context(),
            available,
        )
        .with_settled_auto_scrollbars(input.settled_auto_scrollbars())
        .with_containing_auto_scrollbar_pass(input.containing_auto_scrollbar_pass());
        let measured = compute_grid_with_context_settled(
            tree,
            node,
            probe_input,
            parent_context.clone(),
            GridMeasurementBoundary::StandaloneIntrinsicMinimumProbe(intrinsic_axes),
        )?
        .output
        .size;
        Size::new(
            intrinsic_axes.width.then_some(measured.width),
            intrinsic_axes.height.then_some(measured.height),
        )
    } else {
        Size::NONE
    };
    compute_grid_with_context_settled(
        tree,
        node,
        input,
        parent_context,
        GridMeasurementBoundary::Standalone(resolved_intrinsic_minimum),
    )
}

fn compute_grid_with_context_settled<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
    measurement_boundary: GridMeasurementBoundary<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridComputeResult<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let mut pass_input =
        input.with_settled_auto_scrollbars(crate::scroll::SettledAutoScrollbarState::INITIAL);
    loop {
        let result = compute_grid_with_context_result(
            tree,
            node,
            pass_input,
            parent_context.clone(),
            measurement_boundary,
        )?;
        if !input.run_mode().is_perform_layout() {
            return Ok(result);
        }
        let Some(geometry) = result.output.scroll_geometry else {
            return Ok(result);
        };
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || !crate::scroll::settled_auto_scrollbars_change_available_geometry(
                geometry, next_state,
            )
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?
        {
            return Ok(result);
        }
        pass_input = input.with_settled_auto_scrollbars(next_state);
    }
}

fn compute_grid_with_context_result<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
    measurement_boundary: GridMeasurementBoundary<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridComputeResult<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let mut style = tree.node_input(node).clone();
    let standalone_intrinsic_minimum = match measurement_boundary {
        GridMeasurementBoundary::Ordinary => Size::NONE,
        GridMeasurementBoundary::StandaloneIntrinsicMinimumProbe(axes) => {
            style = standalone_intrinsic_minimum_probe_style(&style, axes);
            Size::NONE
        }
        GridMeasurementBoundary::Standalone(minimum) => minimum,
    };
    let constants = Constants::new_with_reservation::<Tree, M>(
        tree,
        node,
        &style,
        input,
        input.run_mode().is_perform_layout(),
        standalone_intrinsic_minimum,
    )?;

    if input.run_mode() == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return Ok(GridComputeResult::from_output(
            ComputeOutputOf::from_outer_size(Size::new(width, height)),
        ));
    }

    if style.display.establishes_grid_lanes_formatting_context() {
        return compute_grid_lanes_with_context_result(
            tree,
            node,
            input,
            parent_context,
            style,
            constants,
        );
    }

    let initialized_tracks = initialize_grid_tracks::<Tree, M>(
        tree,
        node,
        &style,
        &constants,
        &parent_context,
        input.available(),
    )?;
    let InitializedGridTracks {
        column_tracks,
        row_tracks,
        context,
        placements,
        subgrid_report,
        report,
    } = initialized_tracks;
    debug_assert_eq!(subgrid_report.items.len(), tree.child_count(node));
    debug_assert!(
        subgrid_report
            .items
            .iter()
            .all(|item| !item.column.can_inherit() || item.column.mapping.is_ok())
    );
    debug_assert!(
        subgrid_report
            .items
            .iter()
            .all(|item| !item.row.can_inherit() || item.row.mapping.is_ok())
    );
    let GridContainerContext { gap, lines, .. } = context.clone();
    let column_gutters = context.column_gutters.clone();
    let row_gutters = context.row_gutters.clone();
    debug_assert_eq!(lines.column_explicit_start, context.leading_columns);
    debug_assert_eq!(lines.column_explicit_count, context.explicit_columns);
    debug_assert_eq!(lines.row_explicit_start, context.leading_rows);
    debug_assert_eq!(lines.row_explicit_count, context.explicit_rows);
    let sizing_flow_axes = constants.flow_axes;
    let track_available = intrinsic_container_available(
        &style,
        &constants,
        sizing_flow_axes,
        input.parent(),
        input.available(),
    )
    .map_err(|error| sizing_resolution_error(node, error))?;
    let track_resolution = resolve_grid_track_sizes(
        tree,
        node,
        GridTrackResolutionInput {
            sizing_policy: GridTrackSizingPolicy::Ordinary,
            style: &style,
            constants: &constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            context: context.clone(),
            subgrid_report: &subgrid_report,
            sizing_flow_axes,
            available: track_available,
            intrinsic_max_available: intrinsic_max_available(
                &constants,
                sizing_flow_axes,
                input.available(),
            ),
            placements: &placements,
        },
    )?;
    let GridTrackResolution {
        sizing_phases,
        columns,
        rows,
        column_min_intrinsic_sizes,
        column_max_intrinsic_sizes,
        row_intrinsic_sizes,
    } = track_resolution;
    let track_content_size = LogicalSizeOf::new(
        track_content_sum_with_gutters(&column_tracks, &columns, gap.inline, Some(&column_gutters)),
        track_content_sum_with_gutters(&row_tracks, &rows, gap.block, Some(&row_gutters)),
    );
    let cyclic_percent_content_size = cyclic_percent_track_content_size(
        tree,
        node,
        PercentTrackContent {
            style: &style,
            constants: &constants,
            sizing_flow_axes,
            parent_context: &parent_context,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            columns: &columns,
            rows: &rows,
            gap,
            column_gutters: Some(&column_gutters),
            row_gutters: Some(&row_gutters),
            lines,
            placements: &placements,
        },
    )?;
    let cyclic_percent_content_size = sizing_flow_axes.logical_size(cyclic_percent_content_size);
    let content_size = LogicalSizeOf::new(
        track_content_size
            .inline
            .max(cyclic_percent_content_size.inline),
        track_content_size
            .block
            .max(cyclic_percent_content_size.block),
    );
    let logical_style_size = sizing_flow_axes.logical_size(style.size.clone());
    let logical_node_inner_size = sizing_flow_axes.logical_size(constants.node_inner_size);
    let logical_available_inner_size =
        sizing_flow_axes.logical_size(constants.available_inner_size);
    let intrinsic_sizing_content_size = {
        LogicalSizeOf::new(
            intrinsic_sizing_axis_content_size(IntrinsicSizingAxisInput {
                run_mode: input.run_mode(),
                style_size: logical_style_size.inline,
                content_size: content_size.inline,
                track_content_size: track_content_size.inline,
                definite_size: logical_node_inner_size.inline,
                available_size: logical_available_inner_size.inline,
                tracks: &column_tracks,
            }),
            intrinsic_sizing_axis_content_size(IntrinsicSizingAxisInput {
                run_mode: input.run_mode(),
                style_size: logical_style_size.block,
                content_size: content_size.block,
                track_content_size: track_content_size.block,
                definite_size: logical_node_inner_size.block,
                available_size: logical_available_inner_size.block,
                tracks: &row_tracks,
            }),
        )
    };
    let padding_border_size = (constants.padding + constants.border).sum_axes();
    let intrinsic_sizing_physical_size = constants
        .flow_axes
        .physical_size(intrinsic_sizing_content_size);
    let intrinsic_outer_size = (intrinsic_sizing_physical_size
        + constants.content_box_inset.sum_axes())
    .clamp_min_before_max_optional(constants.node_min_size, constants.node_max_size)
    .max(padding_border_size);
    let output_size = input
        .known()
        .or(constants.node_outer_size)
        .unwrap_or(intrinsic_outer_size)
        .max(padding_border_size);
    let mut content_size = constants.flow_axes.physical_size(content_size);

    let mut baselines = BaselinesOf::NONE;
    let mut baseline_groups = GridBaselineGroups {
        rows: Vec::new(),
        columns: Vec::new(),
    };
    let mut final_scroll_geometry = None;
    if input.run_mode().is_perform_layout() {
        let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes: constants.flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            border_box_size: output_size,
            border: constants.border,
            padding: constants.padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
        })
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let mut child_layout = layout_grid_container_children(
            tree,
            node,
            GridChildLayoutInput {
                sizing_phases,
                style: &style,
                constants: &constants,
                column_tracks: &column_tracks,
                row_tracks: &row_tracks,
                context,
                columns: &columns,
                rows: &rows,
                column_min_intrinsic_sizes: &column_min_intrinsic_sizes,
                column_max_intrinsic_sizes: &column_max_intrinsic_sizes,
                row_intrinsic_sizes: &row_intrinsic_sizes,
                output_size,
                subgrid_report: &subgrid_report,
                parent_context: &parent_context,
                placements: &placements,
                containing_auto_scrollbar_pass: input.settled_auto_scrollbars(),
            },
        )?;
        content_size = max_size(content_size, child_layout.visible_content_size);
        child_layout
            .contributions
            .replace_container_seed(scroll_box.padding_box());
        child_layout
            .contributions
            .exclude_reserved_gutter_from_range();
        let geometry = grid_container_scroll_geometry::<_, Tree::Scalar, M>(
            node,
            input.run_mode(),
            &style,
            &constants,
            scroll_box,
            child_layout.contributions,
            input.settled_auto_scrollbars(),
        )?;
        content_size = max_size(
            content_size,
            geometry
                .canonical_content_size()
                .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?,
        );
        final_scroll_geometry = Some(geometry);
        baselines = child_layout.baselines;
        baseline_groups = child_layout.baseline_groups;
    }

    let output = if input.run_mode() == RunMode::ComputeSize {
        ComputeOutputOf::from_outer_size(output_size)
    } else {
        ComputeOutputOf::from_sizes_and_baselines(output_size, content_size, baselines)
    };
    Ok(GridComputeResult {
        output: ComputeOutputOf {
            scroll_geometry: final_scroll_geometry,
            ..output
        },
        report,
        baseline_groups,
    })
}

fn layout_percent_track_floor<S: LayoutScalar>(
    definite_size: Option<S>,
    available_size: Option<S>,
    tracks: &[TrackSizingOf<S>],
) -> S {
    if definite_size.is_some() || tracks.is_empty() {
        return S::ZERO;
    }
    available_size
        .map(|available| track_basis_dependent_space(tracks, available))
        .unwrap_or(S::ZERO)
}

struct IntrinsicSizingAxisInput<'a, S: LayoutScalar = Scalar> {
    run_mode: RunMode,
    style_size: PreferredSizeOf<S>,
    content_size: S,
    track_content_size: S,
    definite_size: Option<S>,
    available_size: Option<S>,
    tracks: &'a [TrackSizingOf<S>],
}

fn intrinsic_sizing_axis_content_size<S: LayoutScalar>(
    input: IntrinsicSizingAxisInput<'_, S>,
) -> S {
    let IntrinsicSizingAxisInput {
        run_mode,
        style_size,
        content_size,
        track_content_size,
        definite_size,
        available_size,
        tracks,
    } = input;
    if run_mode == RunMode::ComputeSize || style_size.is_auto() {
        return content_size;
    }
    track_content_size.max(layout_percent_track_floor(
        definite_size,
        available_size,
        tracks,
    ))
}

fn compute_grid_lanes_with_context_result<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
    style: NodeInputOf<Tree::Scalar>,
    constants: Constants<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridComputeResult<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let initialized_tracks = initialize_grid_tracks::<Tree, M>(
        tree,
        node,
        &style,
        &constants,
        &parent_context,
        input.available(),
    )?;
    let InitializedGridTracks {
        column_tracks,
        row_tracks,
        context,
        placements,
        subgrid_report,
        report,
    } = initialized_tracks;
    let GridContainerContext { gap, lines, .. } = context.clone();
    let sizing_flow_axes = constants.flow_axes;
    let track_available = intrinsic_container_available(
        &style,
        &constants,
        sizing_flow_axes,
        input.parent(),
        input.available(),
    )
    .map_err(|error| sizing_resolution_error(node, error))?;
    let track_resolution = resolve_grid_track_sizes(
        tree,
        node,
        GridTrackResolutionInput {
            sizing_policy: GridTrackSizingPolicy::Lanes,
            style: &style,
            constants: &constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            context: context.clone(),
            subgrid_report: &subgrid_report,
            sizing_flow_axes,
            available: track_available,
            intrinsic_max_available: intrinsic_max_available(
                &constants,
                sizing_flow_axes,
                input.available(),
            ),
            placements: &placements,
        },
    )?;
    let GridTrackResolution {
        sizing_phases,
        columns,
        rows,
        column_min_intrinsic_sizes,
        column_max_intrinsic_sizes,
        row_intrinsic_sizes,
    } = track_resolution;
    let mut logical_content_size = LogicalSizeOf::new(
        track_content_sum_with_gutters(
            &column_tracks,
            &columns,
            gap.inline,
            Some(&context.column_gutters),
        ),
        track_content_sum_with_gutters(&row_tracks, &rows, gap.block, Some(&context.row_gutters)),
    );
    if let Ok(lane_report) = resolve_grid_lanes_placement_with_resolved_tracks(
        tree,
        node,
        &style,
        &constants,
        context.clone(),
        &columns,
        &rows,
        &placements,
        match grid_axis_for_grid_lanes(&style) {
            GridAxisKind::Column => gap.inline,
            GridAxisKind::Row => gap.block,
        },
        sizing_flow_axes.logical_size(constants.node_inner_size),
    )? {
        let has_explicit_lane_tracks = match lane_report.lane_axis {
            GridAxisKind::Column => !style.grid_template_columns.is_empty(),
            GridAxisKind::Row => !style.grid_template_rows.is_empty(),
        };
        match lane_report.lane_axis.logical_axis() {
            LogicalAxis::Inline => {
                if lane_report.content_size > Tree::Scalar::ZERO {
                    logical_content_size.inline = if has_explicit_lane_tracks {
                        logical_content_size.inline.max(lane_report.content_size)
                    } else {
                        lane_report.content_size
                    };
                }
            }
            LogicalAxis::Block => {
                if lane_report.content_size > Tree::Scalar::ZERO {
                    logical_content_size.block = if has_explicit_lane_tracks {
                        logical_content_size.block.max(lane_report.content_size)
                    } else {
                        lane_report.content_size
                    };
                }
            }
        }
    }
    let cyclic_percent_content_size =
        sizing_flow_axes.logical_size(cyclic_percent_track_content_size(
            tree,
            node,
            PercentTrackContent {
                style: &style,
                constants: &constants,
                sizing_flow_axes,
                parent_context: &parent_context,
                column_tracks: &column_tracks,
                row_tracks: &row_tracks,
                columns: &columns,
                rows: &rows,
                gap,
                column_gutters: Some(&context.column_gutters),
                row_gutters: Some(&context.row_gutters),
                lines,
                placements: &placements,
            },
        )?);
    logical_content_size = LogicalSizeOf::new(
        logical_content_size
            .inline
            .max(cyclic_percent_content_size.inline),
        logical_content_size
            .block
            .max(cyclic_percent_content_size.block),
    );
    let padding_border_size = (constants.padding + constants.border).sum_axes();
    let intrinsic_outer_size = (sizing_flow_axes.physical_size(logical_content_size)
        + constants.content_box_inset.sum_axes())
    .clamp_min_before_max_optional(constants.node_min_size, constants.node_max_size)
    .max(padding_border_size);
    let output_size = input
        .known()
        .or(constants.node_outer_size)
        .unwrap_or(intrinsic_outer_size)
        .max(padding_border_size);
    let mut content_size = sizing_flow_axes.physical_size(logical_content_size);

    let mut baselines = BaselinesOf::NONE;
    let mut baseline_groups = GridBaselineGroups {
        rows: Vec::new(),
        columns: Vec::new(),
    };
    let mut final_scroll_geometry = None;
    if input.run_mode().is_perform_layout() {
        let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes: constants.flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            border_box_size: output_size,
            border: constants.border,
            padding: constants.padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
        })
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let layout_content_box_size =
            (output_size - constants.content_box_inset.sum_axes()).max(Size::ZERO);
        let logical_layout_content_box_size =
            sizing_flow_axes.logical_size(layout_content_box_size);
        let layout_gap = resolved_logical_layout_gap(
            tree,
            node,
            &style,
            &constants,
            sizing_flow_axes,
            logical_layout_content_box_size,
            gap,
        )?;
        let logical_node_inner_size = sizing_flow_axes.logical_size(constants.node_inner_size);
        let logical_available_inner_size =
            sizing_flow_axes.logical_size(constants.available_inner_size);
        let layout_columns = resolved_logical_layout_columns(
            sizing_phases,
            &constants,
            sizing_flow_axes,
            &columns,
            sizing_flow_axes.logical_size(output_size).inline,
            InlineTrackInput {
                tracks: &column_tracks,
                basis: context.percent_basis.inline,
                definite_size: logical_node_inner_size.inline,
                available_size: logical_available_inner_size.inline,
                gap: layout_gap.inline,
                gutters: Some(&context.column_gutters),
                alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
                stretch_empty_auto_to_available: false,
                min_intrinsic_sizes: &column_min_intrinsic_sizes,
                max_intrinsic_sizes: &column_max_intrinsic_sizes,
            },
        );
        let layout_rows = resolved_logical_layout_rows(ResolvedLogicalLayoutRowsInput {
            sizing_phases,
            tracks: &row_tracks,
            constants: &constants,
            sizing_flow_axes,
            intrinsic_rows: &rows,
            output_block: sizing_flow_axes.logical_size(output_size).block,
            gap: layout_gap.block,
            alignment: style.align_content.unwrap_or(AlignContent::Stretch),
            intrinsic_sizes: &row_intrinsic_sizes,
            gutters: Some(&context.row_gutters),
        });
        let mut child_layout = layout_grid_lanes_children(
            tree,
            node,
            GridLanesLayoutInput {
                style: &style,
                constants: &constants,
                container_content_box_size: logical_layout_content_box_size,
                context,
                columns: &layout_columns,
                rows: &layout_rows,
                gap: layout_gap,
                subgrid_report: &subgrid_report,
                placements: &placements,
                containing_auto_scrollbar_pass: input.settled_auto_scrollbars(),
            },
        )?;
        content_size = max_size(content_size, child_layout.visible_content_size);
        child_layout
            .contributions
            .replace_container_seed(scroll_box.padding_box());
        child_layout
            .contributions
            .exclude_reserved_gutter_from_range();
        let geometry = grid_container_scroll_geometry::<_, Tree::Scalar, M>(
            node,
            input.run_mode(),
            &style,
            &constants,
            scroll_box,
            child_layout.contributions,
            input.settled_auto_scrollbars(),
        )?;
        content_size = max_size(
            content_size,
            geometry
                .canonical_content_size()
                .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?,
        );
        final_scroll_geometry = Some(geometry);
        baselines = child_layout.baselines;
        baseline_groups = child_layout.baseline_groups;
    }

    let output = if input.run_mode() == RunMode::ComputeSize {
        ComputeOutputOf::from_outer_size(output_size)
    } else {
        ComputeOutputOf::from_sizes_and_baselines(output_size, content_size, baselines)
    };
    Ok(GridComputeResult {
        output: ComputeOutputOf {
            scroll_geometry: final_scroll_geometry,
            ..output
        },
        report,
        baseline_groups,
    })
}

#[derive(Clone, Debug)]
struct GridParentContext<S: LayoutScalar = Scalar, Node = ()> {
    columns: Option<InheritedGridAxis<S, Node>>,
    rows: Option<InheritedGridAxis<S, Node>>,
}

impl<S: LayoutScalar, Node> GridParentContext<S, Node> {
    fn none() -> Self {
        Self {
            columns: None,
            rows: None,
        }
    }

    fn has_inherited_axis(&self) -> bool {
        self.columns.is_some() || self.rows.is_some()
    }

    fn geometry_only(&self) -> GridParentContext<S> {
        GridParentContext {
            columns: self.columns.as_ref().map(InheritedGridAxis::geometry_only),
            rows: self.rows.as_ref().map(InheritedGridAxis::geometry_only),
        }
    }
}

#[derive(Clone, Debug)]
struct InheritedGridAxis<S: LayoutScalar = Scalar, Node = ()> {
    offset: S,
    gap: S,
    tracks: Vec<S>,
    geometry: UsedGridAxisGeometryOf<S>,
    named_lines: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    major_baselines: Vec<Option<PhysicalBaseline<S>>>,
    minor_baselines: Vec<Option<PhysicalBaseline<S>>>,
    owner_baseline_targets: Option<InheritedGridOwnerBaselineTargets<Node, S>>,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
}

impl<S: LayoutScalar, Node> InheritedGridAxis<S, Node> {
    fn geometry_only(&self) -> InheritedGridAxis<S> {
        InheritedGridAxis {
            offset: self.offset,
            gap: self.gap,
            tracks: self.tracks.clone(),
            geometry: self.geometry.clone(),
            named_lines: self.named_lines.clone(),
            area_facts: self.area_facts.clone(),
            major_baselines: self.major_baselines.clone(),
            minor_baselines: self.minor_baselines.clone(),
            owner_baseline_targets: None,
            parent_start: self.parent_start,
            parent_end: self.parent_end,
            reversed: self.reversed,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct InheritedGridOwnerBaselineTargets<Node, S: LayoutScalar = Scalar> {
    group: AncestorBaselineGroup<Node, S>,
    mapping: CheckedOwnerToCurrentPlacementMap<Node, S>,
}

#[derive(Clone)]
struct GridContainerContext<S: LayoutScalar = Scalar> {
    topology: ExpandedGridTopology<S>,
    gap: LogicalSizeOf<S>,
    column_gutters: OrdinaryGridAxisGuttersOf<S>,
    row_gutters: OrdinaryGridAxisGuttersOf<S>,
    percent_basis: LogicalSizeOf<Option<S>>,
    leading_columns: usize,
    leading_rows: usize,
    lines: GridLines,
    inherited_column_offset: Option<S>,
    inherited_row_offset: Option<S>,
}

impl<S: LayoutScalar> core::ops::Deref for GridContainerContext<S> {
    type Target = ExpandedGridTopology<S>;

    fn deref(&self) -> &Self::Target {
        &self.topology
    }
}

struct InitializedGridTracks<Node, S: LayoutScalar = Scalar> {
    column_tracks: Vec<TrackSizingOf<S>>,
    row_tracks: Vec<TrackSizingOf<S>>,
    context: GridContainerContext<S>,
    placements: GridPlacementContext<Node>,
    subgrid_report: GridSubgridReport<Node>,
    report: GridComputationReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResolvedGridItemPlacement {
    pub(super) column: GridPlacement,
    pub(super) row: GridPlacement,
    pub(super) absolute_column: GridPlacement,
    pub(super) absolute_row: GridPlacement,
    pub(super) in_flow: bool,
}

#[derive(Clone, Debug)]
pub(super) struct GridPlacementContext<Node> {
    pub(super) children: Vec<Node>,
    pub(super) items: Vec<ResolvedGridItemPlacement>,
    pub(super) order_modified_indexes: Vec<crate::SourceIndex>,
    settled_areas: Option<Vec<Option<PlacedGridArea>>>,
}

impl<Node> GridPlacementContext<Node> {
    fn new(children: Vec<Node>, items: Vec<ResolvedGridItemPlacement>) -> Self {
        assert_eq!(
            children.len(),
            items.len(),
            "grid placement context must preserve one placement per child"
        );
        let order_modified_indexes = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| item.in_flow.then_some(crate::SourceIndex::new(index)))
            .collect();
        Self {
            children,
            items,
            order_modified_indexes,
            settled_areas: None,
        }
    }

    fn with_order_modified_indexes(
        mut self,
        order_modified_indexes: Vec<crate::SourceIndex>,
    ) -> Self {
        debug_assert_eq!(
            order_modified_indexes.len(),
            self.items.iter().filter(|item| item.in_flow).count()
        );
        self.order_modified_indexes = order_modified_indexes;
        self
    }

    fn settled_areas(&self) -> &[Option<PlacedGridArea>] {
        self.settled_areas.as_deref().unwrap_or(&[])
    }
}

impl<Node: Copy + Eq> GridPlacementContext<Node> {
    fn checked_child_placements<'a>(
        &'a self,
        children: &'a [Node],
    ) -> impl ExactSizeIterator<Item = (Node, ResolvedGridItemPlacement)> + 'a {
        assert!(
            self.children.as_slice() == children,
            "grid placement context must be consumed with its original child order"
        );
        assert_eq!(
            children.len(),
            self.items.len(),
            "grid placement context must preserve one placement per child"
        );
        children.iter().copied().zip(self.items.iter().copied())
    }
}

#[expect(
    clippy::type_complexity,
    reason = "track initialization preserves the grid node, scalar, and provider error types"
)]
fn initialize_grid_tracks<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    parent_context: &GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
    _available: Size<AvailableOf<Tree::Scalar>>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    InitializedGridTracks<<Tree as Traverse>::Node, Tree::Scalar>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let sizing_flow_axes = constants.flow_axes;
    let resolved_gap = Size::new(
        resolve_length_or_zero(style.gap.width, constants.node_inner_size.width),
        resolve_length_or_zero(style.gap.height, constants.node_inner_size.height),
    )
    .transpose_with_node(tree, node)?;
    let mut gap = sizing_flow_axes.logical_size(resolved_gap);
    if let Some(columns) = &parent_context.columns {
        gap.inline = columns.gap;
    }
    if let Some(rows) = &parent_context.rows {
        gap.block = rows.gap;
    }
    let mut percent_basis = sizing_flow_axes.logical_size(constants.node_inner_size);
    let children = tree.children(node).collect::<Vec<_>>();
    let column_expansion_basis = track_expansion_basis(
        &style.grid_template_columns,
        sizing_flow_axes
            .logical_size(constants.node_inner_size)
            .inline,
        sizing_flow_axes
            .logical_size(constants.available_inner_size)
            .inline,
    );
    let row_expansion_basis = track_expansion_basis(
        &style.grid_template_rows,
        sizing_flow_axes
            .logical_size(constants.node_inner_size)
            .block,
        sizing_flow_axes
            .logical_size(constants.available_inner_size)
            .block,
    );
    let column_expansion = if let Some(columns) = &parent_context.columns {
        TrackExpansionOf::inherited(
            columns
                .tracks
                .iter()
                .copied()
                .map(TrackSizingOf::px)
                .collect(),
        )
    } else {
        expand_track_components_with_origins(
            &style.grid_template_columns,
            column_expansion_basis,
            gap.inline,
            None,
        )
        .map_err(|status| crate::compute::value_resolution_error(node, status))?
    };
    let row_expansion = if let Some(rows) = &parent_context.rows {
        TrackExpansionOf::inherited(rows.tracks.iter().copied().map(TrackSizingOf::px).collect())
    } else {
        expand_track_components_with_origins(
            &style.grid_template_rows,
            row_expansion_basis,
            gap.block,
            None,
        )
        .map_err(|status| crate::compute::value_resolution_error(node, status))?
    };
    let sized_column_tracks = column_expansion
        .tracks
        .iter()
        .map(|track| track.sizing.clone())
        .collect::<Vec<_>>();
    let sized_row_tracks = row_expansion
        .tracks
        .iter()
        .map(|track| track.sizing.clone())
        .collect::<Vec<_>>();
    if percent_basis.inline.is_none() && tracks_need_available_basis(&sized_column_tracks) {
        percent_basis.inline = sizing_flow_axes
            .logical_size(constants.available_inner_size)
            .inline;
    }
    if percent_basis.block.is_none() && tracks_need_available_basis(&sized_row_tracks) {
        percent_basis.block = sizing_flow_axes
            .logical_size(constants.available_inner_size)
            .block;
    }
    let sized_explicit_columns = sized_column_tracks.len();
    let sized_explicit_rows = sized_row_tracks.len();
    let mut report = GridComputationReport::default();
    let geometry_parent_context = parent_context.geometry_only();
    let named_context = match build_grid_named_context_with_report(
        style,
        sized_explicit_columns,
        sized_explicit_rows,
        &geometry_parent_context,
    ) {
        Ok((context, named_report)) => {
            report.merge_named_grid(named_report);
            context
        }
        Err(error) => {
            debug_invalid_named_grid_context(&error);
            report.merge_named_grid(NamedGridReport::from_error(error));
            empty_grid_named_context(sized_explicit_columns, sized_explicit_rows)
        }
    };
    let mut topology = ExpandedGridTopology::new(ExpandedGridTopologyInput {
        columns: column_expansion,
        rows: row_expansion,
        named: named_context,
        auto_columns: &style.grid_auto_columns,
        auto_rows: &style.grid_auto_rows,
        column_basis: percent_basis.inline,
        row_basis: percent_basis.block,
        column_gap: gap.inline,
        row_gap: gap.block,
        inherited_columns: parent_context.columns.is_some(),
        inherited_rows: parent_context.rows.is_some(),
    })
    .map_err(|status| crate::compute::value_resolution_error(node, status))?;
    if let Some(columns) = &parent_context.columns {
        topology.collapsed_columns = columns.geometry.collapsed().to_vec();
    }
    if let Some(rows) = &parent_context.rows {
        topology.collapsed_rows = rows.geometry.collapsed().to_vec();
    }
    let explicit_columns = topology.explicit_columns;
    let explicit_rows = topology.explicit_rows;
    let (mut placements, placement_report) = resolve_grid_child_placements(
        &children,
        tree,
        &topology,
        parent_context.columns.is_some(),
        parent_context.rows.is_some(),
    );
    report.merge_named_grid(placement_report);
    let inherited_columns = parent_context.columns.is_some();
    let inherited_rows = parent_context.rows.is_some();
    let ordinary_settled_placement = !style.display.establishes_grid_lanes_formatting_context()
        && !inherited_columns
        && !inherited_rows;
    let (column_tracks, row_tracks, leading_columns, leading_rows) = if ordinary_settled_placement {
        derive_grid_placement_demand(&mut topology, &mut placements, style.grid_auto_flow)
            .map_err(|error| grid_placement_demand_error(node, error))?;
        topology.collapse_ordinary_auto_fit(placements.settled_areas());
        let mut column_tracks = topology.column_tracks.clone();
        let mut row_tracks = topology.row_tracks.clone();
        for (track, collapsed) in column_tracks.iter_mut().zip(&topology.collapsed_columns) {
            if *collapsed {
                *track = TrackSizingOf::px(Tree::Scalar::ZERO);
            }
        }
        for (track, collapsed) in row_tracks.iter_mut().zip(&topology.collapsed_rows) {
            if *collapsed {
                *track = TrackSizingOf::px(Tree::Scalar::ZERO);
            }
        }
        (
            column_tracks,
            row_tracks,
            topology.column_explicit_start,
            topology.row_explicit_start,
        )
    } else {
        // Inherited subgrid axes and grid-lanes retain their separately sequenced
        // placement policies; ordinary grids never enter this pre-sizing path.
        let mut column_tracks = topology.column_tracks.clone();
        let mut row_tracks = topology.row_tracks.clone();
        let grid_lanes = style.display.establishes_grid_lanes_formatting_context();
        let visible_cell_count = if grid_lanes {
            0
        } else {
            placements
                .checked_child_placements(&children)
                .filter(|(child, _)| is_in_flow_grid_child(tree.node_input(*child)))
                .map(|(_, placement)| {
                    placement_cell_span(placement.column, explicit_columns)
                        * placement_cell_span(placement.row, explicit_rows)
                })
                .sum::<usize>()
        };
        let auto_fit_limit = (!grid_lanes).then_some(visible_cell_count);
        let leading_columns = if inherited_columns {
            0
        } else {
            leading_implicit_tracks_from_placements(
                &placements.items,
                GridAxisKind::Column,
                explicit_columns,
            )
        };
        let leading_rows = if inherited_rows {
            0
        } else {
            leading_implicit_tracks_from_placements(
                &placements.items,
                GridAxisKind::Row,
                explicit_rows,
            )
        };
        if !inherited_columns {
            prepend_auto_tracks(
                &mut column_tracks,
                &style.grid_auto_columns,
                percent_basis.inline,
                gap.inline,
                leading_columns,
                auto_fit_limit,
            )
            .map_err(|status| crate::compute::value_resolution_error(node, status))?;
        }
        if !inherited_rows {
            prepend_auto_tracks(
                &mut row_tracks,
                &style.grid_auto_rows,
                percent_basis.block,
                gap.block,
                leading_rows,
                auto_fit_limit,
            )
            .map_err(|status| crate::compute::value_resolution_error(node, status))?;
        }
        let track_requirement = grid_track_requirement_from_placements(&placements.items);
        if grid_lanes {
            if !inherited_columns {
                let required_columns = (leading_columns + track_requirement.inline)
                    .max(1)
                    .max(column_tracks.len());
                extend_auto_tracks(
                    &mut column_tracks,
                    &style.grid_auto_columns,
                    percent_basis.inline,
                    gap.inline,
                    required_columns,
                )
                .map_err(|status| crate::compute::value_resolution_error(node, status))?;
            }
            if !inherited_rows {
                let required_rows = (leading_rows + track_requirement.block)
                    .max(1)
                    .max(row_tracks.len());
                extend_auto_tracks(
                    &mut row_tracks,
                    &style.grid_auto_rows,
                    percent_basis.block,
                    gap.block,
                    required_rows,
                )
                .map_err(|status| crate::compute::value_resolution_error(node, status))?;
            }
        } else if style.grid_auto_flow.is_column() {
            if !inherited_rows {
                extend_auto_tracks(
                    &mut row_tracks,
                    &style.grid_auto_rows,
                    percent_basis.block,
                    gap.block,
                    track_requirement.block.max(1),
                )
                .map_err(|status| crate::compute::value_resolution_error(node, status))?;
            }
            if !inherited_columns {
                let required_columns = if row_tracks.is_empty() {
                    0
                } else {
                    visible_cell_count.div_ceil(row_tracks.len())
                };
                let required_columns = required_columns
                    .max(leading_columns + track_requirement.inline)
                    .max(column_tracks.len());
                extend_auto_tracks(
                    &mut column_tracks,
                    &style.grid_auto_columns,
                    percent_basis.inline,
                    gap.inline,
                    required_columns,
                )
                .map_err(|status| crate::compute::value_resolution_error(node, status))?;
            }
        } else {
            if !inherited_columns {
                let required_columns = (leading_columns + track_requirement.inline)
                    .max(1)
                    .max(column_tracks.len());
                extend_auto_tracks(
                    &mut column_tracks,
                    &style.grid_auto_columns,
                    percent_basis.inline,
                    gap.inline,
                    required_columns,
                )
                .map_err(|status| crate::compute::value_resolution_error(node, status))?;
            }
            if !inherited_rows {
                let required_rows = if column_tracks.is_empty() {
                    0
                } else {
                    visible_cell_count.div_ceil(column_tracks.len())
                };
                let required_rows = required_rows
                    .max(leading_rows + track_requirement.block)
                    .max(row_tracks.len());
                extend_auto_tracks(
                    &mut row_tracks,
                    &style.grid_auto_rows,
                    percent_basis.block,
                    gap.block,
                    required_rows,
                )
                .map_err(|status| crate::compute::value_resolution_error(node, status))?;
            }
        }
        if grid_lanes {
            let grid_axis = grid_axis_for_grid_lanes(style);
            let (track_count, explicit_start) = match grid_axis {
                GridAxisKind::Column => (column_tracks.len(), leading_columns),
                GridAxisKind::Row => (row_tracks.len(), leading_rows),
            };
            apply_grid_lanes_auto_fit_policy(
                style,
                &mut topology,
                &placements,
                track_count,
                explicit_start,
            )
            .map_err(|error| grid_placement_demand_error(node, error))?;
            for (track, collapsed) in column_tracks.iter_mut().zip(&topology.collapsed_columns) {
                if *collapsed {
                    *track = TrackSizingOf::px(Tree::Scalar::ZERO);
                }
            }
            for (track, collapsed) in row_tracks.iter_mut().zip(&topology.collapsed_rows) {
                if *collapsed {
                    *track = TrackSizingOf::px(Tree::Scalar::ZERO);
                }
            }
        }
        (column_tracks, row_tracks, leading_columns, leading_rows)
    };

    let lines = GridLines {
        column_explicit_start: leading_columns,
        column_explicit_count: explicit_columns,
        row_explicit_start: leading_rows,
        row_explicit_count: explicit_rows,
    };

    let subgrid_report = collect_subgrid_report(tree, node, style);

    let column_gutters = parent_context.columns.as_ref().map_or_else(
        || {
            OrdinaryGridAxisGuttersOf::new(
                column_tracks.len(),
                &topology.collapsed_columns,
                gap.inline,
            )
        },
        |axis| axis.geometry.sizing_gutters(),
    );
    let row_gutters = parent_context.rows.as_ref().map_or_else(
        || OrdinaryGridAxisGuttersOf::new(row_tracks.len(), &topology.collapsed_rows, gap.block),
        |axis| axis.geometry.sizing_gutters(),
    );

    Ok(InitializedGridTracks {
        column_tracks,
        row_tracks,
        context: GridContainerContext {
            topology,
            gap,
            column_gutters,
            row_gutters,
            percent_basis,
            leading_columns,
            leading_rows,
            lines,
            inherited_column_offset: parent_context.columns.as_ref().map(|axis| axis.offset),
            inherited_row_offset: parent_context.rows.as_ref().map(|axis| axis.offset),
        },
        placements,
        subgrid_report,
        report,
    })
}

fn debug_invalid_named_grid_context(_error: &NamedGridError) {}

fn grid_placement_demand_error<Node, S: LayoutScalar, M>(
    container: Node,
    error: GridPlacementDemandError,
) -> LayoutErrorOf<Node, S, M> {
    match error {
        GridPlacementDemandError::AxisCapacity { .. }
        | GridPlacementDemandError::OccupancyCapacity { .. } => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(container),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidBlockScrollGeometry,
            ),
        ),
    }
}

fn resolve_grid_child_placements<Tree, M>(
    children: &[<Tree as Traverse>::Node],
    tree: &Tree,
    topology: &ExpandedGridTopology<Tree::Scalar>,
    subgrid_columns: bool,
    subgrid_rows: bool,
) -> (
    GridPlacementContext<<Tree as Traverse>::Node>,
    NamedGridReport,
)
where
    Tree: Compute<M>,
{
    let mut report = NamedGridReport::default();
    let mut items = Vec::with_capacity(children.len());
    for child in children.iter().copied() {
        let style = tree.node_input(child);
        if style.display == Display::None {
            items.push(ResolvedGridItemPlacement {
                column: style.grid_column,
                row: style.grid_row,
                absolute_column: style.grid_column,
                absolute_row: style.grid_row,
                in_flow: false,
            });
            continue;
        }

        let (column, absolute_column, column_report) =
            resolve_grid_item_axis_placements_with_report(
                &topology.named_columns,
                &style.raw_grid_column,
                style.grid_column,
                subgrid_columns,
            );
        report.extend(column_report);
        let (row, absolute_row, row_report) = resolve_grid_item_axis_placements_with_report(
            &topology.named_rows,
            &style.raw_grid_row,
            style.grid_row,
            subgrid_rows,
        );
        report.extend(row_report);
        items.push(ResolvedGridItemPlacement {
            column,
            row,
            absolute_column,
            absolute_row,
            in_flow: style.position != Position::Absolute,
        });
    }
    let order_modified_indexes = item_order_permutation(
        &items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.in_flow)
            .map(|(index, _)| {
                (
                    tree.node_input(children[index]).item_order,
                    crate::SourceIndex::new(index),
                )
            })
            .collect::<Vec<_>>(),
    );
    (
        GridPlacementContext::new(children.to_vec(), items)
            .with_order_modified_indexes(order_modified_indexes),
        report,
    )
}

fn resolve_grid_item_axis_placements_with_report(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
    subgrid_axis: bool,
) -> (GridPlacement, GridPlacement, NamedGridReport) {
    if subgrid_axis {
        let (absolute, mut report) =
            resolve_absolute_grid_item_axis_placement_with_report(lines, raw, legacy);
        let (placement, placement_report) =
            resolve_subgrid_item_axis_placement_with_report(lines, raw, legacy);
        report.extend_unique(placement_report);
        return (placement, absolute, report);
    }

    if legacy.is_auto() || raw == &super::RawGridPlacement::AUTO {
        let (placement, report) = resolve_grid_item_axis_placement_with_report(lines, raw, legacy);
        return (placement, placement, report);
    }

    let (absolute, mut report) =
        resolve_absolute_grid_item_axis_placement_with_report(lines, raw, legacy);
    let (placement, placement_report) =
        resolve_grid_item_axis_placement_with_report(lines, raw, legacy);
    report.extend(placement_report);
    (placement, absolute, report)
}

#[cfg(test)]
fn resolve_grid_item_axis_placement(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> GridPlacement {
    resolve_grid_item_axis_placement_with_report(lines, raw, legacy).0
}

fn resolve_grid_item_axis_placement_with_report(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> (GridPlacement, NamedGridReport) {
    if raw == &super::RawGridPlacement::AUTO && !legacy.is_auto() {
        return (legacy, NamedGridReport::default());
    }
    let (resolved, report) = resolve_grid_placement_or_auto_with_report(lines, raw, None);
    if resolved.is_auto() && raw_uses_only_numeric_grid_lines(raw) && !legacy.is_auto() {
        (legacy, report)
    } else {
        (resolved, report)
    }
}

#[cfg(test)]
fn resolve_absolute_grid_item_axis_placement(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> GridPlacement {
    resolve_absolute_grid_item_axis_placement_with_report(lines, raw, legacy).0
}

fn resolve_absolute_grid_item_axis_placement_with_report(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> (GridPlacement, NamedGridReport) {
    if !legacy.is_auto() {
        return (legacy, NamedGridReport::default());
    }
    resolve_grid_placement_or_auto_with_report(lines, raw, None)
}

fn resolve_subgrid_item_axis_placement_with_report(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> (GridPlacement, NamedGridReport) {
    if raw == &super::RawGridPlacement::AUTO && !legacy.is_auto() {
        return (legacy, NamedGridReport::default());
    }
    match resolve_subgrid_placement(lines, raw, None) {
        Ok(placement) => (placement, NamedGridReport::default()),
        Err(error) => (GridPlacement::AUTO, NamedGridReport::from_error(error)),
    }
}

fn raw_uses_only_numeric_grid_lines(raw: &super::RawGridPlacement) -> bool {
    raw_grid_line_is_numeric(&raw.start) && raw_grid_line_is_numeric(&raw.end)
}

fn raw_grid_line_is_numeric(line: &super::RawGridLine) -> bool {
    matches!(
        line,
        super::RawGridLine::Auto | super::RawGridLine::Line(_) | super::RawGridLine::Span(_)
    )
}

struct GridTrackResolutionInput<'a, Node, S: LayoutScalar = Scalar> {
    sizing_policy: GridTrackSizingPolicy,
    style: &'a NodeInputOf<S>,
    constants: &'a Constants<S>,
    column_tracks: &'a [TrackSizingOf<S>],
    row_tracks: &'a [TrackSizingOf<S>],
    context: GridContainerContext<S>,
    subgrid_report: &'a GridSubgridReport<Node>,
    sizing_flow_axes: crate::geometry::FlowAxes,
    available: LogicalSizeOf<AvailableOf<S>>,
    intrinsic_max_available: LogicalSizeOf<bool>,
    placements: &'a GridPlacementContext<Node>,
}

#[derive(Clone, Copy)]
enum GridTrackSizingPolicy {
    Ordinary,
    Lanes,
}

#[derive(Clone, Copy)]
struct GridTrackSizingPhases {
    policy: GridTrackSizingPolicy,
}

impl GridTrackSizingPhases {
    fn resolve_inline<S: LayoutScalar>(self, input: InlineTrackInput<'_, S>) -> Vec<S> {
        match self.policy {
            GridTrackSizingPolicy::Ordinary => resolve_inline_tracks(input),
            GridTrackSizingPolicy::Lanes => resolve_lanes_inline_tracks(input),
        }
    }

    fn resolve_block<S: LayoutScalar>(
        self,
        tracks: &[TrackSizingOf<S>],
        basis: Option<S>,
        gap: S,
        alignment: AlignContent,
        intrinsic_sizes: &[S],
        gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
    ) -> Vec<S> {
        match self.policy {
            GridTrackSizingPolicy::Ordinary => {
                resolve_tracks_with_gutters(tracks, basis, gap, alignment, intrinsic_sizes, gutters)
            }
            GridTrackSizingPolicy::Lanes => resolve_lanes_tracks_with_gutters(
                tracks,
                basis,
                gap,
                alignment,
                intrinsic_sizes,
                gutters,
            ),
        }
    }
}

struct GridTrackResolution<S: LayoutScalar = Scalar> {
    sizing_phases: GridTrackSizingPhases,
    columns: Vec<S>,
    rows: Vec<S>,
    column_min_intrinsic_sizes: Vec<S>,
    column_max_intrinsic_sizes: Vec<S>,
    row_intrinsic_sizes: Vec<S>,
}

fn resolve_grid_track_sizes<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridTrackResolutionInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridTrackResolution<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let GridTrackResolutionInput {
        sizing_policy,
        style,
        constants,
        column_tracks,
        row_tracks,
        context,
        subgrid_report,
        sizing_flow_axes,
        available,
        intrinsic_max_available,
        placements,
    } = input;
    let sizing_phases = GridTrackSizingPhases {
        policy: sizing_policy,
    };
    let GridContainerContext {
        topology,
        gap,
        column_gutters,
        row_gutters,
        percent_basis,
        lines,
        ..
    } = context;
    let active_column_gutters = Some(&column_gutters);
    let active_row_gutters = Some(&row_gutters);
    let ExpandedGridTopology {
        named_columns,
        named_rows,
        area_facts,
        ..
    } = topology;
    let logical_node_inner_size = sizing_flow_axes.logical_size(constants.node_inner_size);
    let logical_available_inner_size =
        sizing_flow_axes.logical_size(constants.available_inner_size);
    let logical_node_max_size = sizing_flow_axes.logical_size(constants.node_max_size);
    let logical_content_box_inset_size =
        sizing_flow_axes.logical_size(constants.content_box_inset.sum_axes());
    let column_basis = percent_basis.inline;
    let row_basis = percent_basis.block;
    let intrinsic_grid = IntrinsicGrid {
        style,
        constants,
        sizing_flow_axes,
        column_tracks,
        row_tracks,
        gap,
        column_gutters: active_column_gutters,
        row_gutters: active_row_gutters,
        percent_basis: LogicalSizeOf::new(None, None),
        lines,
        named_columns: &named_columns,
        named_rows: &named_rows,
        area_facts: area_facts.as_ref(),
        subgrid_report,
        placements,
    };
    let (mut column_max_intrinsic_sizes, row_intrinsic_sizes) = intrinsic_track_sizes(
        tree,
        node,
        intrinsic_grid,
        sizing_flow_axes.physical_size(LogicalSizeOf::new(
            AvailableOf::MAX_CONTENT,
            AvailableOf::MAX_CONTENT,
        )),
        IntrinsicGridLowerBounds::default(),
    )?;
    let compute_column_min_intrinsic_sizes = available.inline == AvailableOf::MIN_CONTENT
        || (logical_node_inner_size.inline.is_none()
            && logical_available_inner_size.inline.is_some())
        || column_tracks.iter().any(|track| {
            track.min == MinTrackSizingOf::MinContent
                || track.max == MaxTrackSizingOf::MinContent
                || matches!(
                    track,
                    TrackSizingOf {
                        min: MinTrackSizingOf::Auto,
                        max: MaxTrackSizingOf::Auto
                    }
                )
                || matches!(track.max, MaxTrackSizingOf::FitContent(_))
        });
    let mut column_min_intrinsic_sizes = if compute_column_min_intrinsic_sizes {
        intrinsic_track_sizes(
            tree,
            node,
            intrinsic_grid,
            sizing_flow_axes.physical_size(LogicalSizeOf::new(
                AvailableOf::MIN_CONTENT,
                AvailableOf::MAX_CONTENT,
            )),
            IntrinsicGridLowerBounds::default(),
        )?
        .0
    } else {
        column_max_intrinsic_sizes.clone()
    };
    if compute_column_min_intrinsic_sizes {
        column_max_intrinsic_sizes = intrinsic_track_sizes(
            tree,
            node,
            intrinsic_grid,
            sizing_flow_axes.physical_size(LogicalSizeOf::new(
                AvailableOf::MAX_CONTENT,
                AvailableOf::MAX_CONTENT,
            )),
            IntrinsicGridLowerBounds {
                columns: Some(&column_min_intrinsic_sizes),
                rows: None,
            },
        )?
        .0;
    }
    if style.display.establishes_grid_lanes_formatting_context()
        && grid_axis_for_grid_lanes(style) == GridAxisKind::Column
    {
        let lane_min = lane_intrinsic_track_sizes(
            tree,
            node,
            LaneIntrinsicTrackSizeInput {
                constants,
                axis: GridAxisKind::Column,
                tracks: column_tracks,
                gap: gap.inline,
                available: AvailableOf::MIN_CONTENT,
                available_basis: column_basis,
                gutters: active_column_gutters,
                lines,
                placements,
                subgrid_report,
            },
        )?;
        let lane_max = lane_intrinsic_track_sizes(
            tree,
            node,
            LaneIntrinsicTrackSizeInput {
                constants,
                axis: GridAxisKind::Column,
                tracks: column_tracks,
                gap: gap.inline,
                available: AvailableOf::MAX_CONTENT,
                available_basis: column_basis,
                gutters: active_column_gutters,
                lines,
                placements,
                subgrid_report,
            },
        )?;
        merge_lane_intrinsic_lower_bounds(&mut column_min_intrinsic_sizes, lane_min);
        merge_lane_intrinsic_lower_bounds(&mut column_max_intrinsic_sizes, lane_max);
    }
    let mixed_column_intrinsic_sizes = track_resolution_intrinsic_sizes(
        column_tracks,
        &column_min_intrinsic_sizes,
        &column_max_intrinsic_sizes,
    );
    let column_resolution_intrinsic_sizes = if available.inline == AvailableOf::MIN_CONTENT {
        column_min_intrinsic_sizes.as_slice()
    } else {
        mixed_column_intrinsic_sizes.as_slice()
    };
    let mut columns = {
        sizing_phases.resolve_inline(InlineTrackInput {
            tracks: column_tracks,
            basis: column_basis,
            definite_size: logical_node_inner_size.inline,
            available_size: logical_available_inner_size.inline,
            gap: gap.inline,
            gutters: active_column_gutters,
            alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
            stretch_empty_auto_to_available: intrinsic_max_available.inline
                && logical_node_inner_size.inline.is_none()
                && logical_node_max_size.inline.is_some(),
            min_intrinsic_sizes: &column_min_intrinsic_sizes,
            max_intrinsic_sizes: column_resolution_intrinsic_sizes,
        })
    };
    if logical_node_inner_size.inline.is_none()
        && let Some(max_inline) = logical_node_max_size.inline
    {
        let max_inner_inline =
            (max_inline - logical_content_box_inset_size.inline).max(Tree::Scalar::ZERO);
        if track_sum_with_gutters(&columns, gap.inline, active_column_gutters) > max_inner_inline {
            columns = {
                sizing_phases.resolve_inline(InlineTrackInput {
                    tracks: column_tracks,
                    basis: column_basis,
                    definite_size: logical_node_inner_size.inline,
                    available_size: Some(max_inner_inline),
                    gap: gap.inline,
                    gutters: active_column_gutters,
                    alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
                    stretch_empty_auto_to_available: false,
                    min_intrinsic_sizes: &column_min_intrinsic_sizes,
                    max_intrinsic_sizes: column_resolution_intrinsic_sizes,
                })
            };
        }
    }
    let unconstrained_row_intrinsic_sizes = row_intrinsic_sizes;
    let mut row_intrinsic_sizes = {
        let constrained_row_intrinsic_sizes =
            constrained_row_intrinsic_sizes(tree, node, intrinsic_grid, &columns, gap)?;
        unconstrained_row_intrinsic_sizes
            .iter()
            .copied()
            .zip(constrained_row_intrinsic_sizes)
            .map(|(unconstrained, constrained)| unconstrained.max(constrained))
            .collect::<Vec<_>>()
    };
    if style.display.establishes_grid_lanes_formatting_context()
        && grid_axis_for_grid_lanes(style) == GridAxisKind::Row
    {
        let lane_rows = lane_intrinsic_track_sizes(
            tree,
            node,
            LaneIntrinsicTrackSizeInput {
                constants,
                axis: GridAxisKind::Row,
                tracks: row_tracks,
                gap: gap.block,
                available: AvailableOf::MAX_CONTENT,
                available_basis: row_basis,
                gutters: active_row_gutters,
                lines,
                placements,
                subgrid_report,
            },
        )?;
        merge_lane_intrinsic_lower_bounds(&mut row_intrinsic_sizes, lane_rows);
    }
    let mut rows = {
        sizing_phases.resolve_block(
            row_tracks,
            row_basis,
            gap.block,
            style.align_content.unwrap_or(AlignContent::Stretch),
            &row_intrinsic_sizes,
            active_row_gutters,
        )
    };
    let row_constrained_column_intrinsic_sizes =
        constrained_column_intrinsic_sizes(tree, node, intrinsic_grid, &columns, &rows, gap)?;
    let mut columns_need_resolution = false;
    for (index, contribution) in row_constrained_column_intrinsic_sizes
        .into_iter()
        .enumerate()
    {
        if let Some(min_size) = column_min_intrinsic_sizes.get_mut(index)
            && contribution > *min_size
        {
            *min_size = contribution;
            columns_need_resolution = true;
        }
        if let Some(max_size) = column_max_intrinsic_sizes.get_mut(index)
            && contribution > *max_size
        {
            *max_size = contribution;
            columns_need_resolution = true;
        }
    }
    if columns_need_resolution {
        let mixed_column_intrinsic_sizes = track_resolution_intrinsic_sizes(
            column_tracks,
            &column_min_intrinsic_sizes,
            &column_max_intrinsic_sizes,
        );
        let column_resolution_intrinsic_sizes = if available.inline == AvailableOf::MIN_CONTENT {
            column_min_intrinsic_sizes.as_slice()
        } else {
            mixed_column_intrinsic_sizes.as_slice()
        };
        columns = {
            sizing_phases.resolve_inline(InlineTrackInput {
                tracks: column_tracks,
                basis: column_basis,
                definite_size: logical_node_inner_size.inline,
                available_size: logical_available_inner_size.inline,
                gap: gap.inline,
                gutters: active_column_gutters,
                alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
                stretch_empty_auto_to_available: intrinsic_max_available.inline
                    && logical_node_inner_size.inline.is_none()
                    && logical_node_max_size.inline.is_some(),
                min_intrinsic_sizes: &column_min_intrinsic_sizes,
                max_intrinsic_sizes: column_resolution_intrinsic_sizes,
            })
        };
        let constrained_row_intrinsic_sizes =
            constrained_row_intrinsic_sizes(tree, node, intrinsic_grid, &columns, gap)?;
        row_intrinsic_sizes = unconstrained_row_intrinsic_sizes
            .iter()
            .copied()
            .zip(constrained_row_intrinsic_sizes)
            .map(|(unconstrained, constrained)| unconstrained.max(constrained))
            .collect::<Vec<_>>();
        rows = {
            sizing_phases.resolve_block(
                row_tracks,
                row_basis,
                gap.block,
                style.align_content.unwrap_or(AlignContent::Stretch),
                &row_intrinsic_sizes,
                active_row_gutters,
            )
        };
    }

    Ok(GridTrackResolution {
        sizing_phases,
        columns,
        rows,
        column_min_intrinsic_sizes,
        column_max_intrinsic_sizes,
        row_intrinsic_sizes,
    })
}

fn merge_intrinsic_lower_bounds<S: LayoutScalar>(sizes: &mut [S], lower_bounds: &[S]) {
    for (size, lower_bound) in sizes.iter_mut().zip(lower_bounds) {
        *size = size.max(*lower_bound);
    }
}

fn merge_lane_intrinsic_lower_bounds<S: LayoutScalar>(
    sizes: &mut [S],
    lower_bounds: Result<Vec<S>, LanePlacementError>,
) {
    match lower_bounds {
        Ok(lower_bounds) => merge_intrinsic_lower_bounds(sizes, &lower_bounds),
        Err(
            error @ (LanePlacementError::EmptyTrackList
            | LanePlacementError::InvalidGridAxisStart { .. }
            | LanePlacementError::InvalidGridAxisSpan { .. }
            | LanePlacementError::GridAxisSpanOutOfRange { .. }
            | LanePlacementError::ContentSizedTrackOutOfRange { .. }
            | LanePlacementError::InvalidDefiniteLaneSpan { .. }
            | LanePlacementError::DefiniteLaneSpanOutOfRange { .. }
            | LanePlacementError::InvalidGridFlowToleranceBasis
            | LanePlacementError::InvalidGridFlowToleranceResolution),
        ) => {
            unreachable!("unexpected grid-lanes intrinsic sizing error: {error:?}");
        }
    }
}

struct GridChildLayoutInput<'a, Node, S: LayoutScalar = Scalar> {
    sizing_phases: GridTrackSizingPhases,
    style: &'a NodeInputOf<S>,
    constants: &'a Constants<S>,
    column_tracks: &'a [TrackSizingOf<S>],
    row_tracks: &'a [TrackSizingOf<S>],
    context: GridContainerContext<S>,
    columns: &'a [S],
    rows: &'a [S],
    column_min_intrinsic_sizes: &'a [S],
    column_max_intrinsic_sizes: &'a [S],
    row_intrinsic_sizes: &'a [S],
    output_size: Size<S>,
    subgrid_report: &'a GridSubgridReport<Node>,
    parent_context: &'a GridParentContext<S, Node>,
    placements: &'a GridPlacementContext<Node>,
    containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
}

fn layout_grid_container_children<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridChildLayoutInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridChildrenLayout<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let GridChildLayoutInput {
        sizing_phases,
        style,
        constants,
        column_tracks,
        row_tracks,
        context,
        columns,
        rows,
        column_min_intrinsic_sizes,
        column_max_intrinsic_sizes,
        row_intrinsic_sizes,
        output_size,
        subgrid_report,
        parent_context,
        placements,
        containing_auto_scrollbar_pass,
    } = input;
    let GridContainerContext {
        topology,
        gap,
        column_gutters,
        row_gutters,
        percent_basis: container_percent_basis,
        lines,
        inherited_column_offset,
        inherited_row_offset,
        ..
    } = context;
    let ExpandedGridTopology {
        named_columns,
        named_rows,
        area_facts,
        collapsed_columns,
        collapsed_rows,
        ..
    } = topology;
    let column_basis = container_percent_basis.inline;
    let sizing_flow_axes = constants.flow_axes;
    let layout_content_box_size =
        (output_size - constants.content_box_inset.sum_axes()).max(Size::ZERO);
    let logical_layout_content_box_size = sizing_flow_axes.logical_size(layout_content_box_size);
    let layout_gap = resolved_logical_layout_gap(
        tree,
        node,
        style,
        constants,
        sizing_flow_axes,
        logical_layout_content_box_size,
        gap,
    )?;
    let layout_column_gutters = if parent_context.columns.is_some() {
        column_gutters
    } else {
        OrdinaryGridAxisGuttersOf::new(column_tracks.len(), &collapsed_columns, layout_gap.inline)
    };
    let layout_row_gutters = if parent_context.rows.is_some() {
        row_gutters
    } else {
        OrdinaryGridAxisGuttersOf::new(row_tracks.len(), &collapsed_rows, layout_gap.block)
    };
    let logical_node_inner_size = sizing_flow_axes.logical_size(constants.node_inner_size);
    let logical_available_inner_size =
        sizing_flow_axes.logical_size(constants.available_inner_size);
    let rerun_percent_columns = logical_node_inner_size.inline.is_none() && {
        column_tracks.iter().any(track_has_percent_sizing)
    };
    let (layout_column_min_intrinsic_sizes, layout_column_max_intrinsic_sizes) =
        if layout_gap != gap || rerun_percent_columns {
            let percent_basis = LogicalSizeOf::new(
                rerun_percent_columns.then_some(logical_layout_content_box_size.inline),
                None,
            );
            let intrinsic_grid = IntrinsicGrid {
                style,
                constants,
                sizing_flow_axes,
                column_tracks,
                row_tracks,
                gap: layout_gap,
                column_gutters: Some(&layout_column_gutters),
                row_gutters: Some(&layout_row_gutters),
                percent_basis,
                lines,
                named_columns: &named_columns,
                named_rows: &named_rows,
                area_facts: area_facts.as_ref(),
                subgrid_report,
                placements,
            };
            let (min_columns, _) = intrinsic_track_sizes(
                tree,
                node,
                intrinsic_grid,
                sizing_flow_axes.physical_size(LogicalSizeOf::new(
                    AvailableOf::MIN_CONTENT,
                    AvailableOf::MAX_CONTENT,
                )),
                IntrinsicGridLowerBounds::default(),
            )?;
            let (max_columns, _) = intrinsic_track_sizes(
                tree,
                node,
                intrinsic_grid,
                sizing_flow_axes.physical_size(LogicalSizeOf::new(
                    AvailableOf::MAX_CONTENT,
                    AvailableOf::MAX_CONTENT,
                )),
                IntrinsicGridLowerBounds {
                    columns: Some(&min_columns),
                    rows: None,
                },
            )?;
            (min_columns, max_columns)
        } else {
            (
                column_min_intrinsic_sizes.to_vec(),
                column_max_intrinsic_sizes.to_vec(),
            )
        };
    let layout_intrinsic_columns = if layout_gap != gap || rerun_percent_columns {
        sizing_phases.resolve_inline(InlineTrackInput {
            tracks: column_tracks,
            basis: column_basis,
            definite_size: logical_node_inner_size.inline,
            available_size: logical_available_inner_size.inline,
            gap: layout_gap.inline,
            gutters: Some(&layout_column_gutters),
            alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
            stretch_empty_auto_to_available: false,
            min_intrinsic_sizes: &layout_column_min_intrinsic_sizes,
            max_intrinsic_sizes: &layout_column_max_intrinsic_sizes,
        })
    } else {
        columns.to_vec()
    };
    let layout_columns = resolved_logical_layout_columns(
        sizing_phases,
        constants,
        sizing_flow_axes,
        &layout_intrinsic_columns,
        sizing_flow_axes.logical_size(output_size).inline,
        {
            InlineTrackInput {
                tracks: column_tracks,
                basis: column_basis,
                definite_size: logical_node_inner_size.inline,
                available_size: logical_available_inner_size.inline,
                gap: layout_gap.inline,
                gutters: Some(&layout_column_gutters),
                alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
                stretch_empty_auto_to_available: false,
                min_intrinsic_sizes: &layout_column_min_intrinsic_sizes,
                max_intrinsic_sizes: &layout_column_max_intrinsic_sizes,
            }
        },
    );
    let layout_row_intrinsic_sizes = if layout_columns != columns {
        let percent_basis = LogicalSizeOf::new(
            rerun_percent_columns.then_some(logical_layout_content_box_size.inline),
            None,
        );
        let intrinsic_grid = IntrinsicGrid {
            style,
            constants,
            sizing_flow_axes,
            column_tracks,
            row_tracks,
            gap: layout_gap,
            column_gutters: Some(&layout_column_gutters),
            row_gutters: Some(&layout_row_gutters),
            percent_basis,
            lines,
            named_columns: &named_columns,
            named_rows: &named_rows,
            area_facts: area_facts.as_ref(),
            subgrid_report,
            placements,
        };
        let (_, rows) = intrinsic_track_sizes(
            tree,
            node,
            intrinsic_grid,
            sizing_flow_axes.physical_size(LogicalSizeOf::new(
                AvailableOf::MAX_CONTENT,
                AvailableOf::MAX_CONTENT,
            )),
            IntrinsicGridLowerBounds {
                columns: Some(&layout_columns),
                rows: None,
            },
        )?;
        rows
    } else {
        row_intrinsic_sizes.to_vec()
    };
    let layout_rows = resolved_logical_layout_rows(ResolvedLogicalLayoutRowsInput {
        sizing_phases,
        tracks: row_tracks,
        constants,
        sizing_flow_axes,
        intrinsic_rows: rows,
        output_block: sizing_flow_axes.logical_size(output_size).block,
        gap: layout_gap.block,
        alignment: style.align_content.unwrap_or(AlignContent::Stretch),
        intrinsic_sizes: &layout_row_intrinsic_sizes,
        gutters: Some(&layout_row_gutters),
    });

    layout_grid_children(
        tree,
        node,
        GridLayoutContext {
            style,
            constants,
            container_content_size: layout_content_box_size,
            columns: &layout_columns,
            rows: &layout_rows,
            collapsed_columns: &collapsed_columns,
            collapsed_rows: &collapsed_rows,
            row_tracks,
            gap: layout_gap,
            column_gutters: &layout_column_gutters,
            row_gutters: &layout_row_gutters,
            lines,
            named_columns,
            named_rows,
            area_facts,
            inherited_column_offset,
            inherited_row_offset,
            subgrid_report,
            parent_context,
            placements,
            containing_auto_scrollbar_pass,
        },
    )
}

#[derive(Clone, Copy)]
struct InlineTrackInput<'a, S: LayoutScalar = Scalar> {
    tracks: &'a [TrackSizingOf<S>],
    basis: Option<S>,
    definite_size: Option<S>,
    available_size: Option<S>,
    gap: S,
    gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    alignment: AlignContent,
    stretch_empty_auto_to_available: bool,
    min_intrinsic_sizes: &'a [S],
    max_intrinsic_sizes: &'a [S],
}

struct GridLayoutContext<'a, Node, S: LayoutScalar = Scalar> {
    style: &'a NodeInputOf<S>,
    constants: &'a Constants<S>,
    container_content_size: Size<S>,
    columns: &'a [S],
    rows: &'a [S],
    collapsed_columns: &'a [bool],
    collapsed_rows: &'a [bool],
    row_tracks: &'a [TrackSizingOf<S>],
    gap: LogicalSizeOf<S>,
    column_gutters: &'a OrdinaryGridAxisGuttersOf<S>,
    row_gutters: &'a OrdinaryGridAxisGuttersOf<S>,
    lines: GridLines,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    inherited_column_offset: Option<S>,
    inherited_row_offset: Option<S>,
    subgrid_report: &'a GridSubgridReport<Node>,
    parent_context: &'a GridParentContext<S, Node>,
    placements: &'a GridPlacementContext<Node>,
    containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
}

fn resolved_logical_layout_gap<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    sizing_flow_axes: crate::geometry::FlowAxes,
    content_box_size: LogicalSizeOf<Tree::Scalar>,
    intrinsic_gap: LogicalSizeOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, LogicalSizeOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let physical_content_box_size = sizing_flow_axes.physical_size(content_box_size);
    let physical_intrinsic_gap = sizing_flow_axes.physical_size(intrinsic_gap);
    let gap = Size::new(
        constants.node_inner_size.width.map_or_else(
            || resolve_length_or_zero(style.gap.width, Some(physical_content_box_size.width)),
            |_| Ok(physical_intrinsic_gap.width),
        ),
        constants.node_inner_size.height.map_or_else(
            || resolve_length_or_zero(style.gap.height, Some(physical_content_box_size.height)),
            |_| Ok(physical_intrinsic_gap.height),
        ),
    )
    .transpose_with_node(tree, node)?;
    Ok(sizing_flow_axes.logical_size(gap))
}

fn resolved_logical_layout_columns<S: LayoutScalar>(
    sizing_phases: GridTrackSizingPhases,
    constants: &Constants<S>,
    sizing_flow_axes: crate::geometry::FlowAxes,
    intrinsic_columns: &[S],
    output_inline: S,
    input: InlineTrackInput<'_, S>,
) -> Vec<S> {
    let logical_node_inner_size = sizing_flow_axes.logical_size(constants.node_inner_size);
    let logical_available_inner_size =
        sizing_flow_axes.logical_size(constants.available_inner_size);
    if logical_node_inner_size.inline.is_some()
        || !input.tracks.iter().any(track_needs_layout_resolution)
    {
        return intrinsic_columns.to_vec();
    }

    let logical_content_box_inset_size =
        sizing_flow_axes.logical_size(constants.content_box_inset.sum_axes());
    let content_inline = (output_inline - logical_content_box_inset_size.inline).max(S::ZERO);
    let has_basis_dependent_track = input.tracks.iter().any(track_has_percent_sizing);
    let percent_floor_basis = logical_available_inner_size.inline.filter(|available| {
        has_basis_dependent_track
            && (content_inline - track_basis_dependent_space(input.tracks, *available)).abs()
                <= S::from_f64(0.001)
    });
    let resolution_inline = percent_floor_basis.unwrap_or(content_inline);
    sizing_phases.resolve_inline(InlineTrackInput {
        basis: Some(resolution_inline),
        definite_size: Some(resolution_inline),
        available_size: logical_available_inner_size.inline,
        ..input
    })
}

struct ResolvedLogicalLayoutRowsInput<'a, S: LayoutScalar = Scalar> {
    sizing_phases: GridTrackSizingPhases,
    tracks: &'a [TrackSizingOf<S>],
    constants: &'a Constants<S>,
    sizing_flow_axes: crate::geometry::FlowAxes,
    intrinsic_rows: &'a [S],
    output_block: S,
    gap: S,
    alignment: AlignContent,
    intrinsic_sizes: &'a [S],
    gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
}

fn resolved_logical_layout_rows<S: LayoutScalar>(
    input: ResolvedLogicalLayoutRowsInput<'_, S>,
) -> Vec<S> {
    let ResolvedLogicalLayoutRowsInput {
        sizing_phases,
        tracks,
        constants,
        sizing_flow_axes,
        intrinsic_rows,
        output_block,
        gap,
        alignment,
        intrinsic_sizes,
        gutters,
    } = input;
    let logical_node_inner_size = sizing_flow_axes.logical_size(constants.node_inner_size);
    if logical_node_inner_size.block.is_some() || !tracks.iter().any(track_needs_layout_resolution)
    {
        return intrinsic_rows.to_vec();
    }

    let logical_content_box_inset_size =
        sizing_flow_axes.logical_size(constants.content_box_inset.sum_axes());
    let content_block = (output_block - logical_content_box_inset_size.block).max(S::ZERO);
    sizing_phases.resolve_block(
        tracks,
        Some(content_block),
        gap,
        alignment,
        intrinsic_sizes,
        gutters,
    )
}

fn track_needs_layout_resolution<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    track.depends_on_basis()
}

fn effective_content_box_left<S: LayoutScalar>(
    constants: &Constants<S>,
    content_box_size: Size<S>,
) -> S {
    let padding_border_width = (constants.padding + constants.border).horizontal_sum();
    let outer_width = constants
        .node_outer_size
        .width
        .or(constants.node_max_size.width)
        .map(|width| width.max(padding_border_width))
        .unwrap_or(content_box_size.width + constants.content_box_inset.horizontal_sum());
    constants
        .content_box_inset
        .left
        .min((outer_width - constants.content_box_inset.right).max(S::ZERO))
}

#[derive(Clone, Copy)]
struct Constants<S: LayoutScalar = Scalar> {
    pub(super) flow_axes: crate::geometry::FlowAxes,
    explicit_definite_content_size: Size<Option<S>>,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    available_inner_size: Size<Option<S>>,
    content_box_inset: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
}

impl<S: LayoutScalar> Constants<S> {
    #[cfg(test)]
    fn new<Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &NodeInputOf<S>,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Self::new_with_reservation::<Tree, M>(tree, node, style, input, false, Size::NONE)
    }

    fn new_with_reservation<Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &NodeInputOf<S>,
        input: ComputeInputOf<S>,
        canonical_ordinary_reservation: bool,
        standalone_intrinsic_minimum: Size<Option<S>>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        let (padding, border) = resolve_containing_padding_border(
            input.containing_flow_axes(),
            input.parent(),
            style.padding,
            style.border,
            resolve_length_or_zero,
            |edges| edges.transpose_with_node(tree, node),
        )?;
        let flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
        let padding_border = padding + border;
        let padding_border_size = padding_border.sum_axes();
        let legacy_content_box_inset = content_box_inset_with_scrollbar(
            padding,
            border,
            ScrollbarReservationOf::from_overflow(
                style.overflow,
                style.item_is_replaced,
                style.scrollbar_width.get(),
                style.direction,
            ),
        );
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border_size
        } else {
            Size::ZERO
        };
        let algorithm = sizing_algorithm_for_grid_display(style.display);
        let style_size = if input.sizing_mode() == SizingMode::InherentSize {
            Size::new(
                resolve_preferred_optional(
                    &style.size.width,
                    algorithm,
                    PhysicalAxis::Horizontal,
                    input.parent().width,
                    true,
                )
                .map_err(|error| sizing_resolution_error(node, error))?,
                resolve_preferred_optional(
                    &style.size.height,
                    algorithm,
                    PhysicalAxis::Vertical,
                    input.parent().height,
                    true,
                )
                .map_err(|error| sizing_resolution_error(node, error))?,
            )
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment)
        } else {
            Size::NONE
        };
        let minimum_for_numeric_resolution = Size::new(
            if standalone_intrinsic_minimum.width.is_some() {
                MinSizeOf::AUTO
            } else {
                style.min_size.width.clone()
            },
            if standalone_intrinsic_minimum.height.is_some() {
                MinSizeOf::AUTO
            } else {
                style.min_size.height.clone()
            },
        );
        let resolved_min_size = Size::new(
            resolve_minimum_optional(
                &minimum_for_numeric_resolution.width,
                algorithm,
                PhysicalAxis::Horizontal,
                input.parent().width,
                true,
            )
            .map_err(|error| sizing_resolution_error(node, error))?,
            resolve_minimum_optional(
                &minimum_for_numeric_resolution.height,
                algorithm,
                PhysicalAxis::Vertical,
                input.parent().height,
                true,
            )
            .map_err(|error| sizing_resolution_error(node, error))?,
        )
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let min_size = Size::new(
            standalone_intrinsic_minimum
                .width
                .or(resolved_min_size.width),
            standalone_intrinsic_minimum
                .height
                .or(resolved_min_size.height),
        );
        let max_size = Size::new(
            resolve_maximum_optional(
                &style.max_size.width,
                algorithm,
                PhysicalAxis::Horizontal,
                input.parent().width,
                true,
            )
            .map_err(|error| sizing_resolution_error(node, error))?,
            resolve_maximum_optional(
                &style.max_size.height,
                algorithm,
                PhysicalAxis::Vertical,
                input.parent().height,
                true,
            )
            .map_err(|error| sizing_resolution_error(node, error))?,
        )
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let content_box_inset = if canonical_ordinary_reservation {
            let provisional_outer_size = input
                .known()
                .or(style_size.clamp_min_before_max_optional(min_size, max_size))
                .max_optional(padding_border_size.map(Some));
            let unconstrained_scroll_box_size = padding_border_size
                + Size::splat(style.scrollbar_width.get() + style.scrollbar_width.get());
            let scroll_box_size = provisional_outer_size
                .or(input.available().map(AvailableOf::into_option))
                .or(max_size)
                .unwrap_or(unconstrained_scroll_box_size)
                .max(padding_border_size);
            canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
                flow_axes,
                computed_overflow: style.overflow,
                item_is_replaced: style.item_is_replaced,
                border_box_size: scroll_box_size,
                border,
                padding,
                scrollbar_gutter: style.scrollbar_gutter,
                scrollbar_width: style.scrollbar_width,
                settled_auto_scrollbars: input.settled_auto_scrollbars(),
            })
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?
            .content_box_inset()
        } else {
            legacy_content_box_inset
        };
        let explicit_definite_outer_size = input.known().or(style_size);
        let explicit_definite_content_size =
            explicit_definite_outer_size.sub_optional_unchecked(content_box_inset.sum_axes());
        let node_outer_size = input
            .known()
            .or(style_size.clamp_min_before_max_optional(min_size, max_size))
            .max_optional(padding_border_size.map(Some));
        let node_inner_size = node_outer_size.sub_optional_unchecked(content_box_inset.sum_axes());
        let available_size = input
            .available()
            .zip_map(max_size, intrinsic_available_size_for_axis)
            .clamp_min_before_max_optional(min_size, max_size)
            .max_optional(padding_border_size.map(Some));
        let available_inner_size =
            available_size.sub_optional_unchecked(content_box_inset.sum_axes());

        Ok(Self {
            flow_axes,
            explicit_definite_content_size,
            node_outer_size,
            node_inner_size,
            node_min_size: min_size,
            node_max_size: max_size,
            available_inner_size,
            content_box_inset,
            padding,
            border,
        })
    }
}

fn grid_container_scroll_geometry<Node, S, M>(
    node: Node,
    run_mode: RunMode,
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    contributions: crate::scroll::ScrollContributionAccumulatorOf<S>,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
) -> LayoutResultOf<Node, crate::ScrollGeometryOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes: constants.flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: scroll_box.border_box().size(),
        border: constants.border,
        padding: constants.padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
        contributions,
        origin_axes: ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
        scroll_snap_type: style.scroll_snap_type,
        target_border_box: scroll_box.border_box(),
        target_scroll_margin: style.scroll_margin,
        target_flow_axes: constants.flow_axes,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
}

fn resolve_length_or_zero<S: LayoutScalar>(
    length: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolve_auto_or_zero<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolve_auto_optional<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    resolution_optional(length.resolve_with_status(basis))
}

trait SizeOptionExt {
    type Scalar: LayoutScalar;
    fn clamp_min_before_max_optional(self, min: Self, max: Self) -> Self;
    fn clamp_optional(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        self.clamp_min_before_max_optional(min, max)
    }
    fn max_optional(self, other: Self) -> Self;
}

trait SizeExt {
    type Scalar: LayoutScalar;
    fn max(self, other: Self) -> Self;
    fn clamp_min_before_max_optional(
        self,
        min: Size<Option<Self::Scalar>>,
        max: Size<Option<Self::Scalar>>,
    ) -> Self;
    fn clamp_optional(
        self,
        min: Size<Option<Self::Scalar>>,
        max: Size<Option<Self::Scalar>>,
    ) -> Self
    where
        Self: Sized,
    {
        self.clamp_min_before_max_optional(min, max)
    }
}

impl<S: LayoutScalar> SizeExt for Size<S> {
    type Scalar = S;

    fn max(self, other: Self) -> Self {
        Size::new(self.width.max(other.width), self.height.max(other.height))
    }

    fn clamp_min_before_max_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self {
        Size::new(
            self.width
                .clamp_min_before_max_optional(min.width, max.width),
            self.height
                .clamp_min_before_max_optional(min.height, max.height),
        )
    }
}

impl<S: LayoutScalar> SizeOptionExt for Size<Option<S>> {
    type Scalar = S;

    fn clamp_min_before_max_optional(self, min: Self, max: Self) -> Self {
        self.zip_map(min, |value, min| match (value, min) {
            (Some(value), Some(min)) => Some(value.max(min)),
            (value, _) => value,
        })
        .zip_map(max, |value, max| match (value, max) {
            (Some(value), Some(max)) => Some(value.min(max)),
            (value, _) => value,
        })
    }

    fn max_optional(self, other: Self) -> Self {
        self.zip_map(other, |value, other| match (value, other) {
            (Some(value), Some(other)) => Some(value.max(other)),
            (value, _) => value,
        })
    }
}

trait ScalarExt {
    fn clamp_min_before_max_optional(self, min: Option<Self>, max: Option<Self>) -> Self
    where
        Self: Sized;
}

impl<S: LayoutScalar> ScalarExt for S {
    fn clamp_min_before_max_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = min.map_or(self, |min| self.max(min));
        max.map_or(value, |max| value.min(max))
    }
}

#[cfg(test)]
mod fri06_c13_t06_characterization_tests {
    use super::*;
    use crate::{
        LayoutErrorKindOf, LayoutErrorSiteOf, LayoutInvalidInputOf, LayoutOperation,
        LengthPercentageOf, ParentFormattingContext, RequestedAxis, WritingMode,
    };

    fn input<S: LayoutScalar>(
        containing_flow_axes: crate::geometry::FlowAxes,
        parent: Size<Option<S>>,
    ) -> ComputeInputOf<S> {
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::ContentSize,
            RequestedAxis::Both,
            Size::NONE,
            parent,
            crate::ContainingLayoutContext::new(
                containing_flow_axes,
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        )
    }

    fn percentage_edges<S: LayoutScalar>() -> Edges<LengthOf<S>> {
        Edges::new(
            LengthOf::percent(S::from_f64(0.1)),
            LengthOf::percent(S::from_f64(0.2)),
            LengthOf::percent(S::from_f64(0.3)),
            LengthOf::percent(S::from_f64(0.4)),
        )
    }

    fn expected_percentage_edges<S: LayoutScalar>(basis: S) -> Edges<S> {
        Edges::new(
            S::from_f64(0.1) * basis,
            S::from_f64(0.2) * basis,
            S::from_f64(0.3) * basis,
            S::from_f64(0.4) * basis,
        )
    }

    fn characterize_constants<S: LayoutScalar>(largest: S) {
        crate::layout_math::assert_fri06_c13_t06_resolution_policy::<S>(
            resolution_or_zero,
            resolution_optional,
        );

        let border = Edges::new(
            LengthOf::px(S::from_f64(1.0)),
            LengthOf::px(S::from_f64(2.0)),
            LengthOf::px(S::from_f64(3.0)),
            LengthOf::px(S::from_f64(4.0)),
        );
        let expected_border = Edges::new(
            S::from_f64(1.0),
            S::from_f64(2.0),
            S::from_f64(3.0),
            S::from_f64(4.0),
        );
        let style = NodeInputOf {
            display: crate::Display::Grid,
            padding: percentage_edges(),
            border,
            ..NodeInputOf::default()
        };
        let tree = crate::test_support::layout_tree::OracleTreeOf::new().style(7, style.clone());

        for (flow, parent, expected_padding) in [
            (
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(100.0)),
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(200.0)),
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(None, Some(S::from_f64(200.0))),
                Edges::ZERO,
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), None),
                Edges::ZERO,
            ),
        ] {
            let constants = Constants::new::<_, core::convert::Infallible>(
                &tree,
                7,
                &style,
                input(flow, parent),
            )
            .expect("grid constants edge characterization must resolve");
            assert_eq!(constants.padding, expected_padding);
            assert_eq!(constants.border, expected_border);
        }

        let positive_overflow = LengthOf::value(
            LengthPercentageOf::from_coefficients(largest, S::ONE)
                .expect("finite positive overflow coefficients"),
        );
        let negative_overflow = LengthOf::value(
            LengthPercentageOf::from_coefficients(-largest, -S::ONE)
                .expect("finite negative overflow coefficients"),
        );
        let failing_style = NodeInputOf {
            display: crate::Display::Grid,
            padding: Edges::new(
                LengthOf::ZERO,
                LengthOf::ZERO,
                LengthOf::ZERO,
                positive_overflow,
            ),
            border: Edges::new(
                negative_overflow,
                LengthOf::ZERO,
                LengthOf::ZERO,
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };
        let failing_tree =
            crate::test_support::layout_tree::OracleTreeOf::new().style(7, failing_style.clone());
        let error = match Constants::new::<_, core::convert::Infallible>(
            &failing_tree,
            7,
            &failing_style,
            input(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::splat(Some(largest)),
            ),
        ) {
            Ok(_) => panic!("padding failure must precede the distinct border failure"),
            Err(error) => error,
        };
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(7));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                value: S::INFINITY,
            })
        );
    }

    #[test]
    fn fri06_c13_t06_grid_resolution_edges_and_error_order_preserve_f32() {
        characterize_constants::<f32>(f32::MAX);
    }

    #[test]
    fn fri06_c13_t06_grid_resolution_edges_and_error_order_preserve_f64() {
        characterize_constants::<f64>(f64::MAX);
    }
}

#[cfg(test)]
mod fri06_c13_t05_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(Some(scalar(2.0)), Some(scalar(9.0)))
                .sub_optional_unchecked(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(Some(scalar(-3.0)), Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_min_before_max_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            Size::new(Some(scalar(5.0)), Some(scalar(5.0))).clamp_min_before_max_optional(
                Size::new(Some(scalar(10.0)), Some(scalar(10.0))),
                Size::new(Some(scalar(3.0)), Some(scalar(3.0))),
            ),
            Size::new(Some(scalar(3.0)), Some(scalar(3.0)))
        );
    }

    #[test]
    fn fri06_c13_t05_grid_unchecked_subtraction_and_clamp_order_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_grid_unchecked_subtraction_and_clamp_order_preserve_f64() {
        characterize::<f64>();
    }
}
