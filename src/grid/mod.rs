use super::{
    AlignContent, AlignItems, Available, Baselines, BoxSizing, Compute, ComputeInput,
    ComputeOutput, Dimension, Direction, Display, Edges, GridAutoFlow, GridFlowTolerance,
    GridPlacement, Length, LengthAuto, MaxTrackSizing, MinTrackSizing, NodeInput, NodeOutput,
    Overflow, Point, Position, RequestedAxis, RunMode, Scalar, Size, SizingMode, TrackComponent,
    TrackRepeat, TrackSizing, Traverse,
};

mod alignment;
mod axis;
mod child;
mod lanes;
mod named;
mod placement;
mod subgrid;
#[cfg(test)]
mod tests;
mod tracks;

use alignment::*;
pub use axis::GridAxisKind;
use axis::{GridAxisMappingError, GridAxisMappingInput, GridAxisMappingReport, map_grid_axis};
use child::*;
pub use lanes::{
    DefiniteLaneIntrinsicItem, IndefiniteLaneContributionGroup, LaneContributionFacts,
    LaneIntrinsicItem, LaneIntrinsicSizingInput, LaneIntrinsicSizingReport, LaneItem,
    LaneItemOffset, LanePlacementError, LanePlacementInput, LanePlacementReport, LaneTrackSpan,
    grid_axis_for_lanes, lane_axis, lane_intrinsic_sizing, place_lanes,
};
use lanes::{
    GridLanesLayoutInput, LaneIntrinsicTrackSizeInput, column_flow_for_grid_lanes,
    grid_axis_for_grid_lanes, lane_intrinsic_track_sizes, layout_grid_lanes_children,
    resolve_grid_lanes_placement_with_resolved_tracks,
};
use named::{
    GridAreaNameFacts, GridNamedContext, NamedGridError, NamedGridLines, build_grid_named_context,
    empty_grid_named_context, resolve_grid_placement_or_auto, resolve_subgrid_placement,
};
use placement::*;
use subgrid::*;
use tracks::*;

pub fn compute_grid<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInput,
) -> ComputeOutput
where
    Tree: Compute,
{
    compute_grid_with_context(tree, node, input, GridParentContext::none())
}

struct GridComputeResult {
    output: ComputeOutput,
    baseline_groups: GridBaselineGroups,
}

impl GridComputeResult {
    fn from_output(output: ComputeOutput) -> Self {
        Self {
            output,
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
        }
    }
}

fn intrinsic_container_available(
    style: &NodeInput,
    constants: &Constants,
    available: Size<Available>,
) -> Size<Available> {
    let max_inner_size = constants
        .node_max_size
        .sub_optional(constants.content_box_inset.sum_axes());
    Size::new(
        intrinsic_available_for_dimension(style.size.width)
            .unwrap_or_else(|| intrinsic_available_for_axis(available.width, max_inner_size.width)),
        intrinsic_available_for_dimension(style.size.height).unwrap_or_else(|| {
            intrinsic_available_for_axis(available.height, max_inner_size.height)
        }),
    )
}

fn intrinsic_available_for_dimension(dimension: Dimension) -> Option<Available> {
    match dimension {
        Dimension::MinContent => Some(Available::MIN_CONTENT),
        Dimension::MaxContent => Some(Available::MAX_CONTENT),
        Dimension::Px(_)
        | Dimension::Percent(_)
        | Dimension::Calc(_)
        | Dimension::Fr(_)
        | Dimension::Auto => None,
    }
}

fn intrinsic_available_for_axis(available: Available, max_size: Option<Scalar>) -> Available {
    match (available, max_size) {
        (Available::MaxContent, Some(max_size)) => Available::Definite(max_size.max(0.0)),
        (available, _) => available,
    }
}

fn intrinsic_available_size_for_axis(
    available: Available,
    max_size: Option<Scalar>,
) -> Option<Scalar> {
    match available {
        Available::Definite(value) => Some(value),
        Available::MaxContent => max_size,
        Available::MinContent => None,
    }
}

fn intrinsic_max_available(constants: &Constants, available: Size<Available>) -> Size<bool> {
    Size::new(
        available.width == Available::MaxContent && constants.node_max_size.width.is_some(),
        available.height == Available::MaxContent && constants.node_max_size.height.is_some(),
    )
}

fn compute_grid_with_context<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInput,
    parent_context: GridParentContext,
) -> ComputeOutput
where
    Tree: Compute,
{
    compute_grid_with_context_result(tree, node, input, parent_context).output
}

fn compute_grid_with_context_result<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInput,
    parent_context: GridParentContext,
) -> GridComputeResult
where
    Tree: Compute,
{
    let style = tree.node_input(node).clone();
    let constants = Constants::new(&style, input);

    if input.run_mode == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return GridComputeResult::from_output(ComputeOutput::from_outer_size(Size::new(
            width, height,
        )));
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

    let initialized_tracks = initialize_grid_tracks(
        tree,
        node,
        &style,
        &constants,
        &parent_context,
        input.available,
    );
    let InitializedGridTracks {
        column_tracks,
        row_tracks,
        context,
        placements,
        subgrid_report,
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
    debug_assert_eq!(lines.column_explicit_start, context.leading_columns);
    debug_assert_eq!(lines.column_explicit_count, context.explicit_columns);
    debug_assert_eq!(lines.row_explicit_start, context.leading_rows);
    debug_assert_eq!(lines.row_explicit_count, context.explicit_rows);
    let track_available = intrinsic_container_available(&style, &constants, input.available);
    let track_resolution = resolve_grid_track_sizes(
        tree,
        node,
        GridTrackResolutionInput {
            style: &style,
            constants: &constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            context: context.clone(),
            subgrid_report: &subgrid_report,
            available: track_available,
            intrinsic_max_available: intrinsic_max_available(&constants, input.available),
            placements: &placements,
        },
    );
    let GridTrackResolution {
        columns,
        rows,
        column_min_intrinsic_sizes,
        column_max_intrinsic_sizes,
        row_intrinsic_sizes,
    } = track_resolution;
    let track_content_size = Size::new(
        track_content_sum(&column_tracks, &columns, gap.width),
        track_content_sum(&row_tracks, &rows, gap.height),
    );
    let cyclic_percent_content_size = cyclic_percent_track_content_size(
        tree,
        node,
        PercentTrackContent {
            style: &style,
            constants: &constants,
            parent_context: &parent_context,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            columns: &columns,
            rows: &rows,
            gap,
            lines,
            placements: &placements,
        },
    );
    let mut content_size = max_size(track_content_size, cyclic_percent_content_size);
    let intrinsic_sizing_content_size = Size::new(
        intrinsic_sizing_axis_content_size(
            input.run_mode,
            style.size.width,
            content_size.width,
            track_content_size.width,
            constants.node_inner_size.width,
            constants.available_inner_size.width,
            &column_tracks,
        ),
        intrinsic_sizing_axis_content_size(
            input.run_mode,
            style.size.height,
            content_size.height,
            track_content_size.height,
            constants.node_inner_size.height,
            constants.available_inner_size.height,
            &row_tracks,
        ),
    );
    let padding_border_size = (constants.padding + constants.border).sum_axes();
    let intrinsic_outer_size = (intrinsic_sizing_content_size
        + constants.content_box_inset.sum_axes())
    .clamp_optional(constants.node_min_size, constants.node_max_size)
    .max(padding_border_size);
    let output_size = input
        .known
        .or(constants.node_outer_size)
        .unwrap_or(intrinsic_outer_size)
        .max(padding_border_size);

    let mut baselines = Baselines::NONE;
    let mut baseline_groups = GridBaselineGroups {
        rows: Vec::new(),
        columns: Vec::new(),
    };
    if input.run_mode.is_perform_layout() {
        let child_layout = layout_grid_container_children(
            tree,
            node,
            GridChildLayoutInput {
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
            },
        );
        content_size = max_size(content_size, child_layout.visible_content_size);
        baselines = Baselines {
            first: Point::new(None, child_layout.first_baseline),
            last: Point::new(None, child_layout.last_baseline),
        };
        baseline_groups = child_layout.baseline_groups;
    }

    let output = if input.run_mode == RunMode::ComputeSize {
        ComputeOutput::from_outer_size(output_size)
    } else {
        ComputeOutput::from_sizes_and_baselines(output_size, content_size, baselines)
    };
    GridComputeResult {
        output,
        baseline_groups,
    }
}

fn layout_percent_track_floor(
    definite_size: Option<Scalar>,
    available_size: Option<Scalar>,
    tracks: &[TrackSizing],
) -> Scalar {
    if definite_size.is_some() || tracks.is_empty() {
        return 0.0;
    }
    available_size
        .map(|available| available * track_percent_sum(tracks))
        .unwrap_or(0.0)
}

fn intrinsic_sizing_axis_content_size(
    run_mode: RunMode,
    style_size: Dimension,
    content_size: Scalar,
    track_content_size: Scalar,
    definite_size: Option<Scalar>,
    available_size: Option<Scalar>,
    tracks: &[TrackSizing],
) -> Scalar {
    if run_mode == RunMode::ComputeSize || style_size.is_auto() {
        return content_size;
    }
    track_content_size.max(layout_percent_track_floor(
        definite_size,
        available_size,
        tracks,
    ))
}

fn compute_grid_lanes_with_context_result<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInput,
    parent_context: GridParentContext,
    style: NodeInput,
    constants: Constants,
) -> GridComputeResult
where
    Tree: Compute,
{
    let initialized_tracks = initialize_grid_tracks(
        tree,
        node,
        &style,
        &constants,
        &parent_context,
        input.available,
    );
    let InitializedGridTracks {
        column_tracks,
        row_tracks,
        context,
        placements,
        subgrid_report,
    } = initialized_tracks;
    let GridContainerContext { gap, lines, .. } = context.clone();
    let track_available = intrinsic_container_available(&style, &constants, input.available);
    let track_resolution = resolve_grid_track_sizes(
        tree,
        node,
        GridTrackResolutionInput {
            style: &style,
            constants: &constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            context: context.clone(),
            subgrid_report: &subgrid_report,
            available: track_available,
            intrinsic_max_available: intrinsic_max_available(&constants, input.available),
            placements: &placements,
        },
    );
    let GridTrackResolution {
        columns,
        rows,
        column_min_intrinsic_sizes,
        column_max_intrinsic_sizes,
        row_intrinsic_sizes,
    } = track_resolution;
    let mut content_size = Size::new(
        track_content_sum(&column_tracks, &columns, gap.width),
        track_content_sum(&row_tracks, &rows, gap.height),
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
            GridAxisKind::Column => gap.width,
            GridAxisKind::Row => gap.height,
        },
    ) {
        match lane_report.lane_axis {
            GridAxisKind::Column => {
                if lane_report.content_size > 0.0 {
                    content_size.width = content_size.width.max(lane_report.content_size);
                }
            }
            GridAxisKind::Row => {
                if lane_report.content_size > 0.0 {
                    content_size.height = content_size.height.max(lane_report.content_size);
                }
            }
        }
    }
    content_size = max_size(
        content_size,
        cyclic_percent_track_content_size(
            tree,
            node,
            PercentTrackContent {
                style: &style,
                constants: &constants,
                parent_context: &parent_context,
                column_tracks: &column_tracks,
                row_tracks: &row_tracks,
                columns: &columns,
                rows: &rows,
                gap,
                lines,
                placements: &placements,
            },
        ),
    );
    let padding_border_size = (constants.padding + constants.border).sum_axes();
    let intrinsic_outer_size = (content_size + constants.content_box_inset.sum_axes())
        .clamp_optional(constants.node_min_size, constants.node_max_size)
        .max(padding_border_size);
    let output_size = input
        .known
        .or(constants.node_outer_size)
        .unwrap_or(intrinsic_outer_size)
        .max(padding_border_size);

    let mut baselines = Baselines::NONE;
    let mut baseline_groups = GridBaselineGroups {
        rows: Vec::new(),
        columns: Vec::new(),
    };
    if input.run_mode.is_perform_layout() {
        let layout_content_box_size =
            (output_size - constants.content_box_inset.sum_axes()).max(Size::ZERO);
        let layout_gap = resolved_layout_gap(&style, &constants, layout_content_box_size, gap);
        let layout_columns = resolved_layout_columns(
            &constants,
            &columns,
            output_size.width,
            InlineTrackInput {
                tracks: &column_tracks,
                basis: context.column_basis,
                definite_size: constants.node_inner_size.width,
                available_size: constants.available_inner_size.width,
                gap: layout_gap.width,
                alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
                stretch_empty_auto_to_available: false,
                min_intrinsic_sizes: &column_min_intrinsic_sizes,
                max_intrinsic_sizes: &column_max_intrinsic_sizes,
            },
        );
        let layout_rows = resolved_layout_rows(
            &row_tracks,
            &constants,
            &rows,
            output_size.height,
            layout_gap.height,
            style.align_content.unwrap_or(AlignContent::Stretch),
            &row_intrinsic_sizes,
        );
        let child_layout = layout_grid_lanes_children(
            tree,
            node,
            GridLanesLayoutInput {
                style: &style,
                constants: &constants,
                container_content_size: layout_content_box_size,
                context,
                columns: &layout_columns,
                rows: &layout_rows,
                gap: layout_gap,
                subgrid_report: &subgrid_report,
                placements: &placements,
            },
        );
        content_size = max_size(content_size, child_layout.visible_content_size);
        baselines = Baselines {
            first: Point::new(None, child_layout.first_baseline),
            last: Point::new(None, child_layout.last_baseline),
        };
        baseline_groups = child_layout.baseline_groups;
    }

    let output = if input.run_mode == RunMode::ComputeSize {
        ComputeOutput::from_outer_size(output_size)
    } else {
        ComputeOutput::from_sizes_and_baselines(output_size, content_size, baselines)
    };
    GridComputeResult {
        output,
        baseline_groups,
    }
}

#[derive(Clone, Debug)]
struct GridParentContext {
    columns: Option<InheritedGridAxis>,
    rows: Option<InheritedGridAxis>,
}

impl GridParentContext {
    fn none() -> Self {
        Self {
            columns: None,
            rows: None,
        }
    }

    fn has_inherited_axis(&self) -> bool {
        self.columns.is_some() || self.rows.is_some()
    }
}

#[derive(Clone, Debug)]
struct InheritedGridAxis {
    offset: Scalar,
    gap: Scalar,
    tracks: Vec<Scalar>,
    named_lines: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    major_baselines: Vec<Option<Scalar>>,
    minor_baselines: Vec<Option<Scalar>>,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
    start_mbp: Scalar,
    end_mbp: Scalar,
    gap_difference: Scalar,
}

#[derive(Clone)]
struct GridContainerContext {
    gap: Size,
    column_basis: Option<Scalar>,
    row_basis: Option<Scalar>,
    explicit_columns: usize,
    explicit_rows: usize,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    leading_columns: usize,
    leading_rows: usize,
    lines: GridLines,
    inherited_column_offset: Option<Scalar>,
    inherited_row_offset: Option<Scalar>,
}

struct InitializedGridTracks<Node> {
    column_tracks: Vec<TrackSizing>,
    row_tracks: Vec<TrackSizing>,
    context: GridContainerContext,
    placements: GridPlacementContext<Node>,
    subgrid_report: GridSubgridReport<Node>,
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
}

impl<Node> GridPlacementContext<Node> {
    fn new(children: Vec<Node>, items: Vec<ResolvedGridItemPlacement>) -> Self {
        assert_eq!(
            children.len(),
            items.len(),
            "grid placement context must preserve one placement per child"
        );
        Self { children, items }
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

fn initialize_grid_tracks<Tree>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInput,
    constants: &Constants,
    parent_context: &GridParentContext,
    _available: Size<Available>,
) -> InitializedGridTracks<<Tree as Traverse>::Node>
where
    Tree: Compute,
{
    let mut gap = Size::new(
        resolve_length_or_zero(style.gap.width, constants.node_inner_size.width),
        resolve_length_or_zero(style.gap.height, constants.node_inner_size.height),
    );
    if let Some(columns) = &parent_context.columns {
        gap.width = columns.gap;
    }
    if let Some(rows) = &parent_context.rows {
        gap.height = rows.gap;
    }
    let mut column_basis = constants.node_inner_size.width;
    let mut row_basis = constants.node_inner_size.height;
    let children = tree.children(node).collect::<Vec<_>>();
    let visible_child_count = children
        .iter()
        .copied()
        .filter(|child| is_in_flow_grid_child(tree.node_input(*child)))
        .count();
    let mut column_tracks = if let Some(columns) = &parent_context.columns {
        columns
            .tracks
            .iter()
            .copied()
            .map(TrackSizing::px)
            .collect()
    } else {
        let column_expansion_basis = track_expansion_basis(
            &style.grid_template_columns,
            constants.node_inner_size.width,
            constants.available_inner_size.width,
        );
        expand_track_components(
            &style.grid_template_columns,
            column_expansion_basis,
            gap.width,
            Some(visible_child_count),
        )
    };
    let mut row_tracks = if let Some(rows) = &parent_context.rows {
        rows.tracks.iter().copied().map(TrackSizing::px).collect()
    } else {
        let row_expansion_basis = track_expansion_basis(
            &style.grid_template_rows,
            constants.node_inner_size.height,
            constants.available_inner_size.height,
        );
        expand_track_components(
            &style.grid_template_rows,
            row_expansion_basis,
            gap.height,
            Some(visible_child_count),
        )
    };
    if column_basis.is_none() && tracks_need_available_basis(&column_tracks) {
        column_basis = constants.available_inner_size.width;
    }
    if row_basis.is_none() && tracks_need_available_basis(&row_tracks) {
        row_basis = constants.available_inner_size.height;
    }
    let explicit_columns = column_tracks.len();
    let explicit_rows = row_tracks.len();
    let named_context =
        build_grid_named_context(style, explicit_columns, explicit_rows, parent_context)
            .unwrap_or_else(|error| {
                debug_invalid_named_grid_context(&error);
                empty_grid_named_context(explicit_columns, explicit_rows)
            });
    let placements = resolve_grid_child_placements(
        &children,
        tree,
        &named_context,
        parent_context.columns.is_some(),
        parent_context.rows.is_some(),
    );
    let visible_cell_count = placements
        .checked_child_placements(&children)
        .filter(|(child, _)| is_in_flow_grid_child(tree.node_input(*child)))
        .map(|(_, placement)| {
            placement_cell_span(placement.column, explicit_columns)
                * placement_cell_span(placement.row, explicit_rows)
        })
        .sum::<usize>();
    let inherited_columns = parent_context.columns.is_some();
    let inherited_rows = parent_context.rows.is_some();
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
        leading_implicit_tracks_from_placements(&placements.items, GridAxisKind::Row, explicit_rows)
    };
    if !inherited_columns {
        prepend_auto_tracks(
            &mut column_tracks,
            &style.grid_auto_columns,
            column_basis,
            gap.width,
            leading_columns,
            Some(visible_child_count),
        );
    }
    if !inherited_rows {
        prepend_auto_tracks(
            &mut row_tracks,
            &style.grid_auto_rows,
            row_basis,
            gap.height,
            leading_rows,
            Some(visible_child_count),
        );
    }
    let track_requirement = grid_track_requirement_from_placements(&placements.items);

    let column_flow = if style.display.establishes_grid_lanes_formatting_context() {
        column_flow_for_grid_lanes(style)
    } else {
        style.grid_auto_flow.is_column()
    };

    if column_flow {
        if !inherited_rows {
            extend_auto_tracks(
                &mut row_tracks,
                &style.grid_auto_rows,
                row_basis,
                gap.height,
                track_requirement.height.max(1),
            );
        }
        if !inherited_columns {
            let required_columns = if row_tracks.is_empty() {
                0
            } else {
                visible_cell_count.div_ceil(row_tracks.len())
            };
            let required_columns = required_columns
                .max(leading_columns + track_requirement.width)
                .max(column_tracks.len());
            extend_auto_tracks(
                &mut column_tracks,
                &style.grid_auto_columns,
                column_basis,
                gap.width,
                required_columns,
            );
        }
    } else {
        if !inherited_columns {
            let required_columns = (leading_columns + track_requirement.width)
                .max(1)
                .max(column_tracks.len());
            extend_auto_tracks(
                &mut column_tracks,
                &style.grid_auto_columns,
                column_basis,
                gap.width,
                required_columns,
            );
        }
        if !inherited_rows {
            let required_rows = if column_tracks.is_empty() {
                0
            } else {
                visible_cell_count.div_ceil(column_tracks.len())
            };
            let required_rows = required_rows
                .max(leading_rows + track_requirement.height)
                .max(row_tracks.len());
            extend_auto_tracks(
                &mut row_tracks,
                &style.grid_auto_rows,
                row_basis,
                gap.height,
                required_rows,
            );
        }
    }

    let lines = GridLines {
        column_explicit_start: leading_columns,
        column_explicit_count: explicit_columns,
        row_explicit_start: leading_rows,
        row_explicit_count: explicit_rows,
    };

    let subgrid_report = collect_subgrid_report(tree, node, style);

    InitializedGridTracks {
        column_tracks,
        row_tracks,
        context: GridContainerContext {
            gap,
            column_basis,
            row_basis,
            explicit_columns,
            explicit_rows,
            named_columns: named_context.columns.clone(),
            named_rows: named_context.rows.clone(),
            area_facts: named_context.area_facts.clone(),
            leading_columns,
            leading_rows,
            lines,
            inherited_column_offset: parent_context.columns.as_ref().map(|axis| axis.offset),
            inherited_row_offset: parent_context.rows.as_ref().map(|axis| axis.offset),
        },
        placements,
        subgrid_report,
    }
}

fn debug_invalid_named_grid_context(_error: &NamedGridError) {}

fn resolve_grid_child_placements<Tree>(
    children: &[<Tree as Traverse>::Node],
    tree: &Tree,
    named_context: &GridNamedContext,
    subgrid_columns: bool,
    subgrid_rows: bool,
) -> GridPlacementContext<<Tree as Traverse>::Node>
where
    Tree: Compute,
{
    let items = children
        .iter()
        .copied()
        .map(|child| {
            let style = tree.node_input(child);
            if style.display == Display::None {
                ResolvedGridItemPlacement {
                    column: style.grid_column,
                    row: style.grid_row,
                    absolute_column: style.grid_column,
                    absolute_row: style.grid_row,
                    in_flow: false,
                }
            } else {
                let absolute_column = resolve_absolute_grid_item_axis_placement(
                    &named_context.columns,
                    &style.raw_grid_column,
                    style.grid_column,
                );
                let absolute_row = resolve_absolute_grid_item_axis_placement(
                    &named_context.rows,
                    &style.raw_grid_row,
                    style.grid_row,
                );
                ResolvedGridItemPlacement {
                    column: if subgrid_columns {
                        resolve_subgrid_item_axis_placement(
                            &named_context.columns,
                            &style.raw_grid_column,
                            style.grid_column,
                        )
                    } else {
                        resolve_grid_item_axis_placement(
                            &named_context.columns,
                            &style.raw_grid_column,
                            style.grid_column,
                        )
                    },
                    row: if subgrid_rows {
                        resolve_subgrid_item_axis_placement(
                            &named_context.rows,
                            &style.raw_grid_row,
                            style.grid_row,
                        )
                    } else {
                        resolve_grid_item_axis_placement(
                            &named_context.rows,
                            &style.raw_grid_row,
                            style.grid_row,
                        )
                    },
                    absolute_column,
                    absolute_row,
                    in_flow: style.position != Position::Absolute,
                }
            }
        })
        .collect();
    GridPlacementContext::new(children.to_vec(), items)
}

fn resolve_grid_item_axis_placement(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> GridPlacement {
    if raw == &super::RawGridPlacement::AUTO && !legacy.is_auto() {
        return legacy;
    }
    let resolved = resolve_grid_placement_or_auto(lines, raw, None);
    if resolved.is_auto() && raw_uses_only_numeric_grid_lines(raw) && !legacy.is_auto() {
        legacy
    } else {
        resolved
    }
}

fn resolve_absolute_grid_item_axis_placement(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> GridPlacement {
    if !legacy.is_auto() {
        return legacy;
    }
    resolve_grid_placement_or_auto(lines, raw, None)
}

fn resolve_subgrid_item_axis_placement(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> GridPlacement {
    if raw == &super::RawGridPlacement::AUTO && !legacy.is_auto() {
        return legacy;
    }
    resolve_subgrid_placement(lines, raw, None).unwrap_or(GridPlacement::AUTO)
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

struct GridTrackResolutionInput<'a, Node> {
    style: &'a NodeInput,
    constants: &'a Constants,
    column_tracks: &'a [TrackSizing],
    row_tracks: &'a [TrackSizing],
    context: GridContainerContext,
    subgrid_report: &'a GridSubgridReport<Node>,
    available: Size<Available>,
    intrinsic_max_available: Size<bool>,
    placements: &'a GridPlacementContext<Node>,
}

struct GridTrackResolution {
    columns: Vec<Scalar>,
    rows: Vec<Scalar>,
    column_min_intrinsic_sizes: Vec<Scalar>,
    column_max_intrinsic_sizes: Vec<Scalar>,
    row_intrinsic_sizes: Vec<Scalar>,
}

fn resolve_grid_track_sizes<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridTrackResolutionInput<'_, <Tree as Traverse>::Node>,
) -> GridTrackResolution
where
    Tree: Compute,
{
    let GridTrackResolutionInput {
        style,
        constants,
        column_tracks,
        row_tracks,
        context,
        subgrid_report,
        available,
        intrinsic_max_available,
        placements,
    } = input;
    let GridContainerContext {
        gap,
        column_basis,
        row_basis,
        lines,
        named_columns,
        named_rows,
        area_facts,
        ..
    } = context;
    let intrinsic_grid = IntrinsicGrid {
        style,
        constants,
        column_tracks,
        row_tracks,
        gap,
        percent_basis: Size::NONE,
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
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        IntrinsicGridLowerBounds::default(),
    );
    let compute_column_min_intrinsic_sizes = available.width == Available::MIN_CONTENT
        || (constants.node_inner_size.width.is_none()
            && constants.available_inner_size.width.is_some())
        || column_tracks.iter().any(|track| {
            track.min == MinTrackSizing::MinContent
                || track.max == MaxTrackSizing::MinContent
                || matches!(
                    track,
                    TrackSizing {
                        min: MinTrackSizing::Auto,
                        max: MaxTrackSizing::Auto
                    }
                )
                || matches!(track.max, MaxTrackSizing::FitContent(_))
        });
    let mut column_min_intrinsic_sizes = if compute_column_min_intrinsic_sizes {
        intrinsic_track_sizes(
            tree,
            node,
            intrinsic_grid,
            Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT),
            IntrinsicGridLowerBounds::default(),
        )
        .0
    } else {
        column_max_intrinsic_sizes.clone()
    };
    if compute_column_min_intrinsic_sizes {
        column_max_intrinsic_sizes = intrinsic_track_sizes(
            tree,
            node,
            intrinsic_grid,
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            IntrinsicGridLowerBounds {
                columns: Some(&column_min_intrinsic_sizes),
                rows: None,
            },
        )
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
                gap: gap.width,
                available: Available::MIN_CONTENT,
                available_basis: column_basis,
                lines,
                placements,
            },
        );
        let lane_max = lane_intrinsic_track_sizes(
            tree,
            node,
            LaneIntrinsicTrackSizeInput {
                constants,
                axis: GridAxisKind::Column,
                tracks: column_tracks,
                gap: gap.width,
                available: Available::MAX_CONTENT,
                available_basis: column_basis,
                lines,
                placements,
            },
        );
        merge_lane_intrinsic_lower_bounds(&mut column_min_intrinsic_sizes, lane_min);
        merge_lane_intrinsic_lower_bounds(&mut column_max_intrinsic_sizes, lane_max);
    }
    let mixed_column_intrinsic_sizes = track_resolution_intrinsic_sizes(
        column_tracks,
        &column_min_intrinsic_sizes,
        &column_max_intrinsic_sizes,
    );
    let column_resolution_intrinsic_sizes = if available.width == Available::MIN_CONTENT {
        column_min_intrinsic_sizes.as_slice()
    } else {
        mixed_column_intrinsic_sizes.as_slice()
    };
    let mut columns = resolve_inline_tracks(InlineTrackInput {
        tracks: column_tracks,
        basis: column_basis,
        definite_size: constants.node_inner_size.width,
        available_size: constants.available_inner_size.width,
        gap: gap.width,
        alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
        stretch_empty_auto_to_available: intrinsic_max_available.width
            && constants.node_inner_size.width.is_none()
            && constants.node_max_size.width.is_some(),
        min_intrinsic_sizes: &column_min_intrinsic_sizes,
        max_intrinsic_sizes: column_resolution_intrinsic_sizes,
    });
    if constants.node_inner_size.width.is_none()
        && let Some(max_width) = constants.node_max_size.width
    {
        let max_inner_width = (max_width - constants.content_box_inset.horizontal_sum()).max(0.0);
        if track_sum(&columns, gap.width) > max_inner_width {
            columns = resolve_inline_tracks(InlineTrackInput {
                tracks: column_tracks,
                basis: column_basis,
                definite_size: constants.node_inner_size.width,
                available_size: Some(max_inner_width),
                gap: gap.width,
                alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
                stretch_empty_auto_to_available: false,
                min_intrinsic_sizes: &column_min_intrinsic_sizes,
                max_intrinsic_sizes: column_resolution_intrinsic_sizes,
            });
        }
    }
    let unconstrained_row_intrinsic_sizes = row_intrinsic_sizes;
    let mut row_intrinsic_sizes = {
        let constrained_row_intrinsic_sizes =
            constrained_row_intrinsic_sizes(tree, node, intrinsic_grid, &columns, gap);
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
                gap: gap.height,
                available: Available::MAX_CONTENT,
                available_basis: row_basis,
                lines,
                placements,
            },
        );
        merge_lane_intrinsic_lower_bounds(&mut row_intrinsic_sizes, lane_rows);
    }
    let mut rows = resolve_tracks(
        row_tracks,
        row_basis,
        gap.height,
        style.align_content.unwrap_or(AlignContent::Stretch),
        &row_intrinsic_sizes,
    );
    let row_constrained_column_intrinsic_sizes =
        constrained_column_intrinsic_sizes(tree, node, intrinsic_grid, &columns, &rows, gap);
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
        let column_resolution_intrinsic_sizes = if available.width == Available::MIN_CONTENT {
            column_min_intrinsic_sizes.as_slice()
        } else {
            mixed_column_intrinsic_sizes.as_slice()
        };
        columns = resolve_inline_tracks(InlineTrackInput {
            tracks: column_tracks,
            basis: column_basis,
            definite_size: constants.node_inner_size.width,
            available_size: constants.available_inner_size.width,
            gap: gap.width,
            alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
            stretch_empty_auto_to_available: intrinsic_max_available.width
                && constants.node_inner_size.width.is_none()
                && constants.node_max_size.width.is_some(),
            min_intrinsic_sizes: &column_min_intrinsic_sizes,
            max_intrinsic_sizes: column_resolution_intrinsic_sizes,
        });
        let constrained_row_intrinsic_sizes =
            constrained_row_intrinsic_sizes(tree, node, intrinsic_grid, &columns, gap);
        row_intrinsic_sizes = unconstrained_row_intrinsic_sizes
            .iter()
            .copied()
            .zip(constrained_row_intrinsic_sizes)
            .map(|(unconstrained, constrained)| unconstrained.max(constrained))
            .collect::<Vec<_>>();
        rows = resolve_tracks(
            row_tracks,
            row_basis,
            gap.height,
            style.align_content.unwrap_or(AlignContent::Stretch),
            &row_intrinsic_sizes,
        );
    }

    GridTrackResolution {
        columns,
        rows,
        column_min_intrinsic_sizes,
        column_max_intrinsic_sizes,
        row_intrinsic_sizes,
    }
}

fn merge_intrinsic_lower_bounds(sizes: &mut [Scalar], lower_bounds: &[Scalar]) {
    for (size, lower_bound) in sizes.iter_mut().zip(lower_bounds) {
        *size = size.max(*lower_bound);
    }
}

fn merge_lane_intrinsic_lower_bounds(
    sizes: &mut [Scalar],
    lower_bounds: Result<Vec<Scalar>, LanePlacementError>,
) {
    match lower_bounds {
        Ok(lower_bounds) => merge_intrinsic_lower_bounds(sizes, &lower_bounds),
        Err(LanePlacementError::NestedGridLanesSubgridIndefiniteUnsupported) => {
            // The first-pass supported scope intentionally leaves indefinite
            // nested grid-lanes subgrid sizing unsupported. Keep that state
            // explicit instead of treating the child as an ordinary lane item.
        }
        Err(error @ (LanePlacementError::EmptyTrackList | LanePlacementError::SpanOutOfRange)) => {
            unreachable!("unexpected grid-lanes intrinsic sizing error: {error:?}");
        }
    }
}

struct GridChildLayoutInput<'a, Node> {
    style: &'a NodeInput,
    constants: &'a Constants,
    column_tracks: &'a [TrackSizing],
    row_tracks: &'a [TrackSizing],
    context: GridContainerContext,
    columns: &'a [Scalar],
    rows: &'a [Scalar],
    column_min_intrinsic_sizes: &'a [Scalar],
    column_max_intrinsic_sizes: &'a [Scalar],
    row_intrinsic_sizes: &'a [Scalar],
    output_size: Size,
    subgrid_report: &'a GridSubgridReport<Node>,
    parent_context: &'a GridParentContext,
    placements: &'a GridPlacementContext<Node>,
}

fn layout_grid_container_children<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridChildLayoutInput<'_, <Tree as Traverse>::Node>,
) -> GridChildrenLayout
where
    Tree: Compute,
{
    let GridChildLayoutInput {
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
    } = input;
    let GridContainerContext {
        gap,
        column_basis,
        lines,
        named_columns,
        named_rows,
        area_facts,
        inherited_column_offset,
        inherited_row_offset,
        ..
    } = context;
    let layout_content_box_size =
        (output_size - constants.content_box_inset.sum_axes()).max(Size::ZERO);
    let layout_gap = resolved_layout_gap(style, constants, layout_content_box_size, gap);
    let rerun_percent_columns = constants.node_inner_size.width.is_none()
        && column_tracks.iter().any(track_has_percent_sizing);
    let (layout_column_min_intrinsic_sizes, layout_column_max_intrinsic_sizes) =
        if layout_gap != gap || rerun_percent_columns {
            let percent_basis = Size::new(
                rerun_percent_columns.then_some(layout_content_box_size.width),
                None,
            );
            let intrinsic_grid = IntrinsicGrid {
                style,
                constants,
                column_tracks,
                row_tracks,
                gap: layout_gap,
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
                Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT),
                IntrinsicGridLowerBounds::default(),
            );
            let (max_columns, _) = intrinsic_track_sizes(
                tree,
                node,
                intrinsic_grid,
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
                IntrinsicGridLowerBounds {
                    columns: Some(&min_columns),
                    rows: None,
                },
            );
            (min_columns, max_columns)
        } else {
            (
                column_min_intrinsic_sizes.to_vec(),
                column_max_intrinsic_sizes.to_vec(),
            )
        };
    let layout_intrinsic_columns = if layout_gap != gap || rerun_percent_columns {
        resolve_inline_tracks(InlineTrackInput {
            tracks: column_tracks,
            basis: column_basis,
            definite_size: constants.node_inner_size.width,
            available_size: constants.available_inner_size.width,
            gap: layout_gap.width,
            alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
            stretch_empty_auto_to_available: false,
            min_intrinsic_sizes: &layout_column_min_intrinsic_sizes,
            max_intrinsic_sizes: &layout_column_max_intrinsic_sizes,
        })
    } else {
        columns.to_vec()
    };
    let layout_columns = resolved_layout_columns(
        constants,
        &layout_intrinsic_columns,
        output_size.width,
        InlineTrackInput {
            tracks: column_tracks,
            basis: column_basis,
            definite_size: constants.node_inner_size.width,
            available_size: constants.available_inner_size.width,
            gap: layout_gap.width,
            alignment: style.justify_content.unwrap_or(AlignContent::Stretch),
            stretch_empty_auto_to_available: false,
            min_intrinsic_sizes: &layout_column_min_intrinsic_sizes,
            max_intrinsic_sizes: &layout_column_max_intrinsic_sizes,
        },
    );
    let layout_row_intrinsic_sizes = if layout_columns != columns {
        let percent_basis = Size::new(
            rerun_percent_columns.then_some(layout_content_box_size.width),
            None,
        );
        let intrinsic_grid = IntrinsicGrid {
            style,
            constants,
            column_tracks,
            row_tracks,
            gap: layout_gap,
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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            IntrinsicGridLowerBounds {
                columns: Some(&layout_columns),
                rows: None,
            },
        );
        rows
    } else {
        row_intrinsic_sizes.to_vec()
    };
    let layout_rows = resolved_layout_rows(
        row_tracks,
        constants,
        rows,
        output_size.height,
        layout_gap.height,
        style.align_content.unwrap_or(AlignContent::Stretch),
        &layout_row_intrinsic_sizes,
    );

    layout_grid_children(
        tree,
        node,
        GridLayoutContext {
            style,
            constants,
            container_content_size: layout_content_box_size,
            columns: &layout_columns,
            rows: &layout_rows,
            row_tracks,
            gap: layout_gap,
            lines,
            named_columns,
            named_rows,
            area_facts,
            inherited_column_offset,
            inherited_row_offset,
            subgrid_report,
            parent_context,
            placements,
        },
    )
}

#[derive(Clone, Copy)]
struct InlineTrackInput<'a> {
    tracks: &'a [TrackSizing],
    basis: Option<Scalar>,
    definite_size: Option<Scalar>,
    available_size: Option<Scalar>,
    gap: Scalar,
    alignment: AlignContent,
    stretch_empty_auto_to_available: bool,
    min_intrinsic_sizes: &'a [Scalar],
    max_intrinsic_sizes: &'a [Scalar],
}

struct GridLayoutContext<'a, Node> {
    style: &'a NodeInput,
    constants: &'a Constants,
    container_content_size: Size,
    columns: &'a [Scalar],
    rows: &'a [Scalar],
    row_tracks: &'a [TrackSizing],
    gap: Size,
    lines: GridLines,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    inherited_column_offset: Option<Scalar>,
    inherited_row_offset: Option<Scalar>,
    subgrid_report: &'a GridSubgridReport<Node>,
    parent_context: &'a GridParentContext,
    placements: &'a GridPlacementContext<Node>,
}

fn resolved_layout_gap(
    style: &NodeInput,
    constants: &Constants,
    content_box_size: Size,
    intrinsic_gap: Size,
) -> Size {
    Size::new(
        constants.node_inner_size.width.map_or_else(
            || resolve_length_or_zero(style.gap.width, Some(content_box_size.width)),
            |_| intrinsic_gap.width,
        ),
        constants.node_inner_size.height.map_or_else(
            || resolve_length_or_zero(style.gap.height, Some(content_box_size.height)),
            |_| intrinsic_gap.height,
        ),
    )
}

fn resolved_layout_columns(
    constants: &Constants,
    intrinsic_columns: &[Scalar],
    output_width: Scalar,
    input: InlineTrackInput<'_>,
) -> Vec<Scalar> {
    if constants.node_inner_size.width.is_some()
        || !input.tracks.iter().any(track_needs_layout_width_resolution)
    {
        return intrinsic_columns.to_vec();
    }

    let content_width = (output_width - constants.content_box_inset.horizontal_sum()).max(0.0);
    let percent_sum = track_percent_sum(input.tracks);
    let percent_floor_basis = constants.available_inner_size.width.filter(|available| {
        percent_sum > 0.0 && (content_width - available * percent_sum).abs() <= 0.001
    });
    let resolution_width = percent_floor_basis.unwrap_or(content_width);
    resolve_inline_tracks(InlineTrackInput {
        basis: Some(resolution_width),
        definite_size: Some(resolution_width),
        available_size: constants.available_inner_size.width,
        ..input
    })
}

fn resolved_layout_rows(
    tracks: &[TrackSizing],
    constants: &Constants,
    intrinsic_rows: &[Scalar],
    output_height: Scalar,
    gap: Scalar,
    alignment: AlignContent,
    intrinsic_sizes: &[Scalar],
) -> Vec<Scalar> {
    if constants.node_inner_size.height.is_some()
        || !tracks.iter().any(track_needs_layout_height_resolution)
    {
        return intrinsic_rows.to_vec();
    }

    let content_height = (output_height - constants.content_box_inset.vertical_sum()).max(0.0);
    resolve_tracks(
        tracks,
        Some(content_height),
        gap,
        alignment,
        intrinsic_sizes,
    )
}

fn track_needs_layout_width_resolution(track: &TrackSizing) -> bool {
    matches!(
        track.max,
        MaxTrackSizing::FitContent(_) | MaxTrackSizing::Length(Length::Percent(_))
    ) || matches!(track.min, MinTrackSizing::Length(Length::Percent(_)))
}

fn track_needs_layout_height_resolution(track: &TrackSizing) -> bool {
    matches!(track.max, MaxTrackSizing::Length(Length::Percent(_)))
        || matches!(track.min, MinTrackSizing::Length(Length::Percent(_)))
}

fn effective_content_box_left(constants: &Constants, content_box_size: Size) -> Scalar {
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
        .min((outer_width - constants.content_box_inset.right).max(0.0))
}

#[derive(Clone, Copy, Debug)]
struct Constants {
    node_outer_size: Size<Option<Scalar>>,
    node_inner_size: Size<Option<Scalar>>,
    node_min_size: Size<Option<Scalar>>,
    node_max_size: Size<Option<Scalar>>,
    available_inner_size: Size<Option<Scalar>>,
    content_box_inset: Edges,
    padding: Edges,
    border: Edges,
}

impl Constants {
    fn new(style: &NodeInput, input: ComputeInput) -> Self {
        let padding = style
            .padding
            .zip_inline_size(input.parent, resolve_length_or_zero);
        let border = style
            .border
            .zip_inline_size(input.parent, resolve_length_or_zero);
        let scrollbar_gutter = Size::new(
            if style.overflow.y == Overflow::Scroll {
                style.scrollbar_width
            } else {
                0.0
            },
            if style.overflow.x == Overflow::Scroll {
                style.scrollbar_width
            } else {
                0.0
            },
        );
        let padding_border = padding + border;
        let mut content_box_inset = padding_border;
        content_box_inset.bottom += scrollbar_gutter.height;
        match style.direction {
            Direction::Ltr => content_box_inset.right += scrollbar_gutter.width,
            Direction::Rtl => content_box_inset.left += scrollbar_gutter.width,
        }
        let padding_border_size = padding_border.sum_axes();
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border_size
        } else {
            Size::ZERO
        };
        let style_size = if input.sizing_mode == SizingMode::InherentSize {
            style
                .size
                .zip_map(input.parent, resolve_dimension)
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment)
        } else {
            Size::NONE
        };
        let min_size = style
            .min_size
            .zip_map(input.parent, resolve_dimension)
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);
        let max_size = style
            .max_size
            .zip_map(input.parent, resolve_dimension)
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);
        let node_outer_size = input
            .known
            .or(style_size.clamp_optional(min_size, max_size))
            .max_optional(padding_border_size.map(Some));
        let node_inner_size = node_outer_size.sub_optional(content_box_inset.sum_axes());
        let available_size = input
            .available
            .zip_map(max_size, intrinsic_available_size_for_axis)
            .clamp_optional(min_size, max_size)
            .max_optional(padding_border_size.map(Some));
        let available_inner_size = available_size.sub_optional(content_box_inset.sum_axes());

        Self {
            node_outer_size,
            node_inner_size,
            node_min_size: min_size,
            node_max_size: max_size,
            available_inner_size,
            content_box_inset,
            padding,
            border,
        }
    }
}

fn resolve_length_or_zero(length: Length, basis: Option<Scalar>) -> Scalar {
    length.resolve_or_zero(basis)
}

fn resolve_auto_or_zero(length: LengthAuto, basis: Option<Scalar>) -> Scalar {
    length.resolve_or_zero(basis)
}

fn resolve_auto_optional(length: LengthAuto, basis: Option<Scalar>) -> Option<Scalar> {
    length.resolve_optional(basis)
}

fn resolve_dimension(dimension: Dimension, basis: Option<Scalar>) -> Option<Scalar> {
    dimension.resolve_optional(basis)
}

trait SizeOptionExt {
    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size) -> Size;
    fn add_optional(self, amount: Size) -> Self;
    fn sub_optional(self, amount: Size) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<Scalar>) -> Self;
    fn clamp_optional(self, min: Self, max: Self) -> Self;
    fn max_optional(self, other: Self) -> Self;
}

trait SizeExt {
    fn max(self, other: Self) -> Self;
    fn clamp_optional(self, min: Size<Option<Scalar>>, max: Size<Option<Scalar>>) -> Self;
}

impl SizeExt for Size {
    fn max(self, other: Self) -> Self {
        Size::new(self.width.max(other.width), self.height.max(other.height))
    }

    fn clamp_optional(self, min: Size<Option<Scalar>>, max: Size<Option<Scalar>>) -> Self {
        Size::new(
            self.width.clamp_optional(min.width, max.width),
            self.height.clamp_optional(min.height, max.height),
        )
    }
}

impl SizeOptionExt for Size<Option<Scalar>> {
    fn or(self, other: Self) -> Self {
        Size::new(self.width.or(other.width), self.height.or(other.height))
    }

    fn unwrap_or(self, fallback: Size) -> Size {
        Size::new(
            self.width.unwrap_or(fallback.width),
            self.height.unwrap_or(fallback.height),
        )
    }

    fn add_optional(self, amount: Size) -> Self {
        self.zip_map(amount, |value, amount| value.map(|value| value + amount))
    }

    fn sub_optional(self, amount: Size) -> Self {
        self.zip_map(amount, |value, amount| value.map(|value| value - amount))
    }

    fn apply_aspect_ratio(self, aspect_ratio: Option<Scalar>) -> Self {
        match (self.width, self.height, aspect_ratio) {
            (Some(width), None, Some(ratio)) => Size::new(Some(width), Some(width / ratio)),
            (None, Some(height), Some(ratio)) => Size::new(Some(height * ratio), Some(height)),
            _ => self,
        }
    }

    fn clamp_optional(self, min: Self, max: Self) -> Self {
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
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self
    where
        Self: Sized;
}

impl ScalarExt for Scalar {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = min.map_or(self, |min| self.max(min));
        max.map_or(value, |max| value.min(max))
    }
}
