use super::*;
use crate::error::{layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::{
    FlowAxes, LogicalAxis, LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis,
    PhysicalProgression,
};
use crate::output::PhysicalBaseline;
use crate::scroll::{
    CanonicalScrollGeometryErrorOf, ClipMarginSourceOf, MeasuredLeafScrollGeometrySourceOf,
    OptimalRegionInsetsOf, OptionalPhysicalContributionIntervalsOf,
    ScrollContributionAccumulatorOf, UsedOverflow, canonical_measured_leaf_scroll_geometry,
    rebuild_canonical_scroll_geometry_for_border_box,
};
use crate::sizing::resolve::{
    SizingResolutionError, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};
use crate::{
    BaselinesOf, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant,
    LayoutOperation,
};

mod baseline;

pub(super) use baseline::*;

pub(super) struct GridChildrenLayout<S: LayoutScalar = Scalar> {
    pub(super) visible_content_size: Size<S>,
    pub(super) contributions: ScrollContributionAccumulatorOf<S>,
    pub(super) baselines: BaselinesOf<S>,
    pub(super) baseline_groups: GridBaselineGroups<S>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct GridChildContribution<S: LayoutScalar = Scalar> {
    pub(super) source_index: crate::SourceIndex,
    pub(super) location: Point<S>,
    pub(super) margin: Edges<S>,
    pub(super) geometry: crate::ScrollGeometryOf<S>,
    pub(super) descendants: OptionalPhysicalContributionIntervalsOf<S>,
    pub(super) overflow: UsedOverflow,
    pub(super) in_flow: bool,
}

pub(super) fn empty_grid_contributions<S: LayoutScalar>() -> ScrollContributionAccumulatorOf<S> {
    ScrollContributionAccumulatorOf::new(
        crate::ScrollRectOf::try_new(Point::ZERO, Size::ZERO)
            .expect("zero grid contribution seed is valid"),
    )
}

#[derive(Clone, Copy)]
pub(super) struct GridLines {
    pub(super) column_explicit_start: usize,
    pub(super) column_explicit_count: usize,
    pub(super) row_explicit_start: usize,
    pub(super) row_explicit_count: usize,
}

pub(super) fn layout_grid_children<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    context: GridLayoutContext<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridChildrenLayout<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let GridLayoutContext {
        style,
        constants,
        container_content_size,
        columns,
        rows,
        collapsed_columns,
        collapsed_rows,
        row_tracks,
        gap,
        column_gutters,
        row_gutters,
        lines,
        named_columns,
        named_rows,
        area_facts,
        template_area_expanded_axes,
        inherited_column_offset,
        inherited_row_offset,
        subgrid_report,
        parent_context,
        placements,
        containing_auto_scrollbar_pass,
    } = context;
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
                        crate::geometry::FlowAxes::new(style.writing_mode, style.direction),
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

    let inherited_column_geometry = parent_context.columns.as_ref().map(|axis| &axis.geometry);
    let inherited_row_geometry = parent_context.rows.as_ref().map(|axis| &axis.geometry);
    let intrinsic_column_geometry = inherited_column_geometry.cloned().unwrap_or_else(|| {
        UsedGridAxisGeometryOf::from_sizing_gutters(columns.to_vec(), column_gutters)
    });
    let intrinsic_row_geometry = inherited_row_geometry
        .cloned()
        .unwrap_or_else(|| UsedGridAxisGeometryOf::from_sizing_gutters(rows.to_vec(), row_gutters));
    let logical_content_size = LogicalSizeOf::new(
        intrinsic_column_geometry.total_extent(),
        intrinsic_row_geometry.total_extent(),
    );
    let physical_content_size = grid_area_physical_size(constants.flow_axes, logical_content_size);
    let legacy_content_box_size =
        constants
            .node_inner_size
            .unwrap_or(if style.writing_mode.is_vertical() {
                physical_content_size
            } else {
                container_content_size
            });
    let logical_content_box_size = constants
        .flow_axes
        .logical_size(constants.node_inner_size.unwrap_or(container_content_size));
    let containing_size = constants
        .node_outer_size
        .unwrap_or(container_content_size + constants.content_box_inset.sum_axes());
    let logical_content_box_inset = constants
        .flow_axes
        .logical_edges(constants.content_box_inset);
    let alignment_free_space = logical_content_box_size - logical_content_size;
    let ordinary_column_alignment = ordinary_grid_axis_alignment(
        alignment_free_space.inline,
        column_gutters,
        style.justify_content.unwrap_or(AlignContent::Stretch),
    );
    let ordinary_row_alignment = ordinary_grid_axis_alignment(
        alignment_free_space.block,
        row_gutters,
        style.align_content.unwrap_or(AlignContent::Stretch),
    );
    let column_alignment = GridAlignment {
        start: ordinary_column_alignment.start,
        gap: gap.inline,
    };
    let row_alignment = GridAlignment {
        start: ordinary_row_alignment.start,
        gap: gap.block,
    };
    let column_geometry = inherited_column_geometry.cloned().unwrap_or_else(|| {
        UsedGridAxisGeometryOf::from_active_boundary_gutters(
            columns.to_vec(),
            collapsed_columns.to_vec(),
            column_gutters.active_boundary_after().to_vec(),
            ordinary_column_alignment.gutter_after,
        )
    });
    let row_geometry = inherited_row_geometry.cloned().unwrap_or_else(|| {
        UsedGridAxisGeometryOf::from_active_boundary_gutters(
            rows.to_vec(),
            collapsed_rows.to_vec(),
            row_gutters.active_boundary_after().to_vec(),
            ordinary_row_alignment.gutter_after,
        )
    });
    let logical_column_geometry = column_geometry.clone().translated(
        inherited_column_offset.unwrap_or(Tree::Scalar::ZERO)
            + logical_content_box_inset.inline_start
            + column_alignment.start,
    );
    let logical_row_geometry = row_geometry.clone().translated(
        inherited_row_offset.unwrap_or(Tree::Scalar::ZERO)
            + logical_content_box_inset.block_start
            + row_alignment.start,
    );
    let logical_column_offsets = (0..columns.len())
        .filter_map(|line| logical_column_geometry.line_offset(line))
        .collect::<Vec<_>>();
    let logical_row_offsets = (0..rows.len())
        .filter_map(|line| logical_row_geometry.line_offset(line))
        .collect::<Vec<_>>();
    let content_box_left = effective_content_box_left(constants, container_content_size);
    let row_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style,
        axis: GridAxisKind::Row,
        tracks: rows,
        geometry: &row_geometry,
        inherited_offset: inherited_row_offset,
        content_box_left,
        content_box_size: legacy_content_box_size,
        content_box_inset: constants.content_box_inset,
        alignment: row_alignment,
    });
    let children = tree.children(node).collect::<Vec<_>>();
    let placed_areas = resolve_grid_child_areas_with_geometry(
        ResolveGridChildAreasInput {
            children: &children,
            placements,
            style,
            columns,
            rows,
            gap,
            lines,
        },
        Some(&column_geometry),
        Some(&row_geometry),
    );
    let empty_baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); rows.len()],
        columns: vec![TrackBaselineGroup::default(); columns.len()],
    };
    let mut child_contributions = Vec::new();
    let mut pending_items = Vec::new();
    for (source_index, (((child, placement), area), subgrid_item)) in placements
        .checked_child_placements(&children)
        .zip(placed_areas.iter().copied())
        .zip(subgrid_report.items.iter())
        .enumerate()
    {
        let child_style = tree.node_input(child).clone();
        if child_style.display == super::Display::None {
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
            continue;
        }
        if child_style.position == Position::Absolute {
            child_contributions.push(layout_absolute_grid_child(
                tree,
                child,
                source_index,
                &child_style,
                AbsoluteGridContext::ordinary_with_geometry(
                    OrdinaryAbsoluteGridGeometryContextInput {
                        container_style: style,
                        constants,
                        containing_size,
                        column: placement.absolute_column,
                        row: placement.absolute_row,
                        column_offsets: &logical_column_offsets,
                        row_offsets: &logical_row_offsets,
                        columns,
                        rows,
                        gap,
                        column_geometry: &logical_column_geometry,
                        row_geometry: &logical_row_geometry,
                        lines,
                    },
                )
                .with_containing_auto_scrollbar_pass(containing_auto_scrollbar_pass),
            )?);
            continue;
        }

        let Some(area) = area else {
            continue;
        };
        if area.row >= rows.len() || area.column >= columns.len() {
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
            continue;
        }

        let physical_area_size = grid_area_physical_size(constants.flow_axes, area.size);
        let mut item = grid_item_sizing_for_grid_flow::<Tree, M>(
            tree,
            child,
            &child_style,
            style,
            physical_area_size,
            physical_area_size.map(Some),
            constants.flow_axes,
        )?;
        apply_final_subgrid_axis_constraints(
            &mut item,
            *subgrid_item,
            constants.flow_axes,
            FlowAxes::new(child_style.writing_mode, child_style.direction),
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
        let child_context = subgrid_child_parent_context_with_geometry(
            SubgridChildParentContextInput {
                item: *subgrid_item,
                child_style: &child_style,
                area,
                content_box_size: subgrid_content_box_size,
                columns,
                rows,
                gap,
                parent_named_columns: &named_columns,
                parent_named_rows: &named_rows,
                parent_area_facts: area_facts.as_ref(),
                parent_baseline_groups: &empty_baseline_groups,
                margin: item.unresolved_margin,
                border,
                padding,
            },
            Some(&column_geometry),
            Some(&row_geometry),
        )
        .map_err(|error| subgrid_child_context_container_error(node, child, error))?;
        let child_input = ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            item.known,
            Size::new(
                Some(physical_area_size.width),
                Some(physical_area_size.height),
            ),
            crate::ContainingLayoutContext::new(
                constants.flow_axes,
                crate::ParentFormattingContext::Grid,
            ),
            item.available
                .map(|value| AvailableOf::Definite(value.max(Tree::Scalar::ZERO))),
        )
        .with_containing_auto_scrollbar_pass(containing_auto_scrollbar_pass);
        let mut output = if child_context.has_inherited_axis() {
            // Subgrid layout depends on the parent grid's used tracks, so this
            // intentionally bypasses the generic child layout cache until that
            // cache can include context-sensitive grid keys.
            compute_grid_with_context_and_standalone_intrinsic_minimum(
                tree,
                child,
                child_input,
                child_context,
                item.standalone_intrinsic_minimum,
            )?
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
        let logical_output_size = constants.flow_axes.logical_size(output.size);
        let logical_unresolved_margin = constants.flow_axes.logical_edges(item.unresolved_margin);
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
        let margin = constants.flow_axes.physical_edges(LogicalEdgesOf::new(
            inline_axis.margin_start,
            inline_axis.margin_end,
            block_axis.margin_start,
            block_axis.margin_end,
        ));
        let baselines = output.baselines();
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let first_baseline =
            baselines.first_or_synthesize_block_baseline(child_flow_axes, output.size);
        let last_baseline = baselines
            .last_block_baseline(child_flow_axes)
            .unwrap_or_else(|| {
                baselines.first_or_synthesize_block_baseline(child_flow_axes, output.size)
            });
        let block_auto_margins = logical_unresolved_margin.block_start.is_none()
            || logical_unresolved_margin.block_end.is_none();
        let row_span_tracks = row_tracks.get(area.row..area.row_end).unwrap_or(&[]);
        let baseline_participation = baseline_participation_for_container(
            item.align_self,
            block_auto_margins,
            synthesized_baseline_would_cycle(
                item.align_self,
                baselines,
                child_flow_axes,
                row_span_tracks,
            ),
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
            logical_relative_offset: logical_relative_inset_offset(
                child_style
                    .inset
                    .zip_size(
                        Size::new(
                            Some(physical_area_size.width),
                            Some(physical_area_size.height),
                        ),
                        resolve_auto_optional,
                    )
                    .transpose_with_node(tree, child)?,
                constants.flow_axes,
                child_style.position,
            ),
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

    let ancestor_baseline_groups = final_ancestor_baseline_groups(
        tree,
        FinalAncestorBaselineGroupsInput {
            node,
            constants,
            container_style: style,
            columns,
            rows,
            column_geometry: &column_geometry,
            row_geometry: &row_geometry,
            row_tracks,
            gap,
            children: &children,
            placed_areas: &placed_areas,
            subgrid_report,
            named_columns: &named_columns,
            named_rows: &named_rows,
            area_facts: area_facts.as_ref(),
        },
        &pending_items,
    )?
    .with_parent_context(parent_context);
    let baseline_group_set = ancestor_baseline_groups.placement_groups();
    refresh_subgrid_items_with_baselines(
        tree,
        SubgridBaselineRefreshInput {
            node,
            container_style: style,
            columns,
            rows,
            column_geometry: &column_geometry,
            row_geometry: &row_geometry,
            row_tracks,
            gap,
            named_columns: named_columns.clone(),
            named_rows: named_rows.clone(),
            area_facts: area_facts.clone(),
            template_area_expanded_axes,
            subgrid_report,
            ancestor_baseline_groups: &ancestor_baseline_groups,
            containing_auto_scrollbar_pass,
        },
        &mut pending_items,
    )?;
    let published_group_set = baseline_groups(
        &pending_items,
        rows.len(),
        columns.len(),
        constants.flow_axes,
    );
    let has_inherited_row_descendant = subgrid_report.items.iter().any(|report| {
        [report.column, report.row]
            .into_iter()
            .any(|axis| axis.can_inherit() && axis.mapping.parent_axis == GridAxisKind::Row)
    });
    let mut prepared_item_placements = Vec::with_capacity(pending_items.len());
    for item in &pending_items {
        let area_origin =
            grid_area_logical_origin(&logical_column_offsets, &logical_row_offsets, item.area);
        let child_style = tree.node_input(item.node);
        let subgrid_item = subgrid_report.items.get(item.source_index).copied();
        let inherited_columns =
            ancestor_baseline_groups.inherited_targets_for_axis(GridAxisKind::Column);
        let column_group = ancestor_baseline_groups.for_axis(GridAxisKind::Column);
        let inline_axis_offset = baseline_aligned_axis_offset(BaselineAlignedAxisInput {
            item,
            child_style,
            container_style: style,
            group: column_group,
            axis: GridAxisKind::Column,
            geometry: &column_geometry,
            row_tracks,
            subgrid_item,
            container_flow_axes: constants.flow_axes,
            intrinsic_baseline_census: has_inherited_row_descendant,
            inherited_owner_targets: inherited_columns,
            child_envelope: ancestor_baseline_groups.child_envelope_for_axis(GridAxisKind::Column),
            current_grid: node,
        })
        .map_err(|error| {
            subgrid_child_context_container_error(
                node,
                item.node,
                SubgridChildContextError::BaselineInheritance(
                    SubgridBaselineInheritanceError::Placement(error),
                ),
            )
        })?
        .unwrap_or(item.horizontal_axis.offset);
        let inherited_rows = ancestor_baseline_groups.inherited_targets_for_axis(GridAxisKind::Row);
        let row_baseline_group = ancestor_baseline_groups.for_axis(GridAxisKind::Row);
        let block_axis_offset = baseline_aligned_axis_offset(BaselineAlignedAxisInput {
            item,
            child_style,
            container_style: style,
            group: row_baseline_group,
            axis: GridAxisKind::Row,
            geometry: &row_geometry,
            row_tracks,
            subgrid_item,
            container_flow_axes: constants.flow_axes,
            intrinsic_baseline_census: has_inherited_row_descendant,
            inherited_owner_targets: inherited_rows,
            child_envelope: ancestor_baseline_groups.child_envelope_for_axis(GridAxisKind::Row),
            current_grid: node,
        })
        .map_err(|error| {
            subgrid_child_context_container_error(
                node,
                item.node,
                SubgridChildContextError::BaselineInheritance(
                    SubgridBaselineInheritanceError::Placement(error),
                ),
            )
        })?
        .unwrap_or(item.vertical_axis.offset);
        let logical_location = LogicalPointOf::new(
            area_origin.inline + inline_axis_offset + item.logical_relative_offset.inline,
            area_origin.block + block_axis_offset + item.logical_relative_offset.block,
        );
        let location = constants.flow_axes.physical_point(
            logical_location,
            constants.flow_axes.logical_size(item.output.size),
            containing_size,
        );
        let scroll_geometry = item
            .output
            .scroll_geometry
            .expect("pending grid item retains canonical geometry");
        debug_assert_eq!(scroll_geometry.used_overflow_x(), item.overflow.x().value());
        debug_assert_eq!(scroll_geometry.used_overflow_y(), item.overflow.y().value());
        let (horizontal, vertical) = subgrid_parent_propagation_axes(
            subgrid_report.items[item.source_index],
            constants.flow_axes,
            item.child_flow_axes,
        );
        let contribution = GridChildContribution {
            source_index: crate::SourceIndex::new(item.source_index),
            location,
            margin: item.margin,
            geometry: scroll_geometry,
            descendants: scroll_geometry
                .propagatable_descendant_intervals()
                .retain_physical_axes(horizontal, vertical),
            overflow: item.overflow,
            in_flow: true,
        };

        prepared_item_placements.push((block_axis_offset, location, contribution));
    }

    for (item, (block_axis_offset, location, contribution)) in
        pending_items.iter_mut().zip(prepared_item_placements)
    {
        item.block_offset = block_axis_offset;
        item.location = location;
        child_contributions.push(contribution);
        tree.set_unrounded(
            item.node,
            NodeOutputOf {
                source_index: crate::SourceIndex::new(item.source_index),
                location,
                size: item.output.size,
                content_size: item.output.content_size,
                scroll_geometry: Some(contribution.geometry),
                border: item.border,
                padding: item.padding,
                margin: item.margin,
            },
        );
    }
    let baselines = if parent_context.has_inherited_axis() {
        grid_container_baselines(
            &pending_items,
            &baseline_group_set,
            &row_offsets,
            rows,
            constants.flow_axes,
        )
    } else {
        logical_grid_container_baselines(
            &pending_items,
            &baseline_group_set,
            &logical_row_offsets,
            rows,
            constants.flow_axes,
            containing_size,
        )
    };

    let mut contributions =
        grid_scroll_contributions(child_contributions, constants.flow_axes, constants.padding)
            .map_err(|error| layout_child_geometry_error(node, node, error))?;
    let inline_start = logical_column_offsets
        .iter()
        .copied()
        .reduce(Tree::Scalar::min)
        .unwrap_or(logical_content_box_inset.inline_start);
    let inline_end = logical_column_offsets
        .iter()
        .copied()
        .zip(columns.iter().copied())
        .map(|(offset, size)| offset + size)
        .reduce(Tree::Scalar::max)
        .unwrap_or(inline_start);
    let block_start = logical_row_offsets
        .iter()
        .copied()
        .reduce(Tree::Scalar::min)
        .unwrap_or(logical_content_box_inset.block_start);
    let block_end = logical_row_offsets
        .iter()
        .copied()
        .zip(rows.iter().copied())
        .map(|(offset, size)| offset + size)
        .reduce(Tree::Scalar::max)
        .unwrap_or(block_start);
    let logical_subject_size =
        LogicalSizeOf::new(inline_end - inline_start, block_end - block_start);
    let subject_size = constants.flow_axes.physical_size(logical_subject_size);
    let subject_origin = constants.flow_axes.physical_point(
        LogicalPointOf::new(inline_start, block_start),
        logical_subject_size,
        containing_size,
    );
    let track_subject = crate::ScrollRectOf::try_new(subject_origin, subject_size)
        .map_err(|error| layout_child_geometry_error(node, node, error))?;
    if style.justify_content.is_some() {
        contributions
            .set_active_alignment_subject(constants.flow_axes.inline_axis(), track_subject);
    }
    if style.align_content.is_some() {
        contributions.set_active_alignment_subject(constants.flow_axes.block_axis(), track_subject);
    }
    let visible_content_size = contributions
        .content_size_from_anchor(Point::ZERO)
        .map_err(|error| layout_child_geometry_error(node, node, error))?;

    let layout = GridChildrenLayout {
        visible_content_size,
        contributions,
        baselines: baselines.baselines,
        baseline_groups: published_group_set,
    };
    debug_assert_eq!(
        layout
            .contributions
            .content_size_from_anchor(Point::ZERO)
            .ok(),
        Some(layout.visible_content_size)
    );
    Ok(layout)
}

struct SubgridBaselineRefreshInput<'a, Node, S: LayoutScalar = Scalar> {
    node: Node,
    container_style: &'a NodeInputOf<S>,
    columns: &'a [S],
    rows: &'a [S],
    column_geometry: &'a UsedGridAxisGeometryOf<S>,
    row_geometry: &'a UsedGridAxisGeometryOf<S>,
    row_tracks: &'a [TrackSizingOf<S>],
    gap: LogicalSizeOf<S>,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    template_area_expanded_axes: TemplateAreaExpandedAxes,
    subgrid_report: &'a GridSubgridReport<Node>,
    ancestor_baseline_groups: &'a FinalAncestorBaselineGroups<Node, S>,
    containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
}

fn refresh_subgrid_items_with_baselines<Tree, M>(
    tree: &mut Tree,
    input: SubgridBaselineRefreshInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
    pending_items: &mut [PendingGridItem<<Tree as Traverse>::Node, Tree::Scalar>],
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let container_flow_axes = FlowAxes::new(
        input.container_style.writing_mode,
        input.container_style.direction,
    );
    let empty_baseline_groups = GridBaselineGroups {
        rows: Vec::new(),
        columns: Vec::new(),
    };
    for item in pending_items.iter_mut() {
        let Some(subgrid_item) = input.subgrid_report.items.get(item.source_index).copied() else {
            continue;
        };
        if !subgrid_item.column.can_inherit() && !subgrid_item.row.can_inherit() {
            continue;
        }

        let child_style = tree.node_input(item.node).clone();
        let physical_area_size = grid_area_physical_size(
            FlowAxes::new(
                input.container_style.writing_mode,
                input.container_style.direction,
            ),
            item.area.size,
        );
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let mut sizing = grid_item_sizing_for_grid_flow::<Tree, M>(
            tree,
            item.node,
            &child_style,
            input.container_style,
            physical_area_size,
            physical_area_size.map(Some),
            container_flow_axes,
        )?;
        apply_final_subgrid_axis_constraints(
            &mut sizing,
            subgrid_item,
            container_flow_axes,
            child_flow_axes,
        );
        let area_parent = physical_area_size.map(Some);
        let padding = container_flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.padding,
                area_parent,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, item.node)?;
        let border = container_flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.border,
                area_parent,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, item.node)?;
        let resolved_margin = sizing
            .unresolved_margin
            .map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
        let subgrid_content_box_size = (physical_area_size
            - resolved_margin.sum_axes()
            - padding.sum_axes()
            - border.sum_axes())
        .max(Size::ZERO);
        let child_context = subgrid_child_parent_context_from_ancestor_groups_with_geometry(
            SubgridChildParentContextInput {
                item: subgrid_item,
                child_style: &child_style,
                area: item.area,
                content_box_size: subgrid_content_box_size,
                columns: input.columns,
                rows: input.rows,
                gap: input.gap,
                parent_named_columns: &input.named_columns,
                parent_named_rows: &input.named_rows,
                parent_area_facts: input.area_facts.as_ref(),
                parent_baseline_groups: &empty_baseline_groups,
                margin: sizing.unresolved_margin,
                border,
                padding,
            },
            input.ancestor_baseline_groups,
            input.template_area_expanded_axes,
            input.node,
            Some(input.column_geometry),
            Some(input.row_geometry),
        )
        .map_err(|error| subgrid_child_context_container_error(input.node, item.node, error))?;
        if !child_context.has_inherited_axis() {
            continue;
        }
        let child_input = ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            sizing.known,
            Size::new(
                Some(physical_area_size.width),
                Some(physical_area_size.height),
            ),
            crate::ContainingLayoutContext::new(
                container_flow_axes,
                crate::ParentFormattingContext::Grid,
            ),
            sizing
                .available
                .map(|value| AvailableOf::Definite(value.max(Tree::Scalar::ZERO))),
        )
        .with_containing_auto_scrollbar_pass(input.containing_auto_scrollbar_pass);
        let result = compute_grid_with_context_settled_and_standalone_intrinsic_minimum(
            tree,
            item.node,
            child_input,
            child_context,
            sizing.standalone_intrinsic_minimum,
        )?;
        let GridComputeResult {
            mut output,
            baseline_groups: _ordinary_baseline_groups,
            ..
        } = result;
        let scroll_geometry = retained_grid_child_scroll_geometry(
            &child_style,
            output.size,
            output.content_size,
            padding,
            border,
            output.scroll_geometry,
        )
        .map_err(|error| layout_child_geometry_error(input.node, item.node, error))?;
        output.scroll_geometry = Some(scroll_geometry);
        let alignment = grid_item_physical_alignment(
            input.container_style.writing_mode,
            sizing.justify_self,
            sizing.align_self,
        );
        let physical_horizontal_axis = physical_grid_item_axis(PhysicalGridItemAxis {
            area_size: physical_area_size.width,
            size: output.size.width,
            margin_start: sizing.unresolved_margin.left,
            margin_end: sizing.unresolved_margin.right,
            alignment: alignment.horizontal,
            progression: grid_physical_axis_progression(
                input.container_style.writing_mode,
                input.container_style.direction,
                PhysicalAxis::Horizontal,
            ),
        });
        let physical_vertical_axis = physical_grid_item_axis(PhysicalGridItemAxis {
            area_size: physical_area_size.height,
            size: output.size.height,
            margin_start: sizing.unresolved_margin.top,
            margin_end: sizing.unresolved_margin.bottom,
            alignment: alignment.vertical,
            progression: grid_physical_axis_progression(
                input.container_style.writing_mode,
                input.container_style.direction,
                PhysicalAxis::Vertical,
            ),
        });
        let margin = Edges {
            left: physical_horizontal_axis.margin_start,
            right: physical_horizontal_axis.margin_end,
            top: physical_vertical_axis.margin_start,
            bottom: physical_vertical_axis.margin_end,
        };
        let logical_offset = container_flow_axes.logical_point(
            Point::new(
                physical_horizontal_axis.offset,
                physical_vertical_axis.offset,
            ),
            output.size,
            physical_area_size,
        );
        let logical_margin = container_flow_axes.logical_edges(margin);
        let horizontal_axis = ResolvedGridItemAxis {
            offset: logical_offset.inline,
            margin_start: logical_margin.inline_start,
            margin_end: logical_margin.inline_end,
        };
        let vertical_axis = ResolvedGridItemAxis {
            offset: logical_offset.block,
            margin_start: logical_margin.block_start,
            margin_end: logical_margin.block_end,
        };
        let baselines = output.baselines();
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let first_baseline =
            baselines.first_or_synthesize_block_baseline(child_flow_axes, output.size);
        let last_baseline = baselines
            .last_block_baseline(child_flow_axes)
            .unwrap_or_else(|| {
                baselines.first_or_synthesize_block_baseline(child_flow_axes, output.size)
            });
        let block_auto_margins = child_flow_axes
            .line_over_edge(sizing.unresolved_margin)
            .is_none()
            || child_flow_axes
                .line_under_edge(sizing.unresolved_margin)
                .is_none();
        let row_span_tracks = input
            .row_tracks
            .get(item.area.row..item.area.row_end)
            .unwrap_or(&[]);
        let baseline_participation = baseline_participation_for_container(
            sizing.align_self,
            block_auto_margins,
            synthesized_baseline_would_cycle(
                sizing.align_self,
                baselines,
                child_flow_axes,
                row_span_tracks,
            ),
            baselines,
            child_flow_axes,
            FlowAxes::new(
                input.container_style.writing_mode,
                input.container_style.direction,
            ),
        );
        item.output = output;
        item.horizontal_axis = horizontal_axis;
        item.vertical_axis = vertical_axis;
        item.child_flow_axes = child_flow_axes;
        item.first_baseline = first_baseline;
        item.last_baseline = last_baseline;
        item.block_auto_margins = block_auto_margins;
        item.baseline_participation = baseline_participation;
        item.margin = margin;
        item.border = border;
        item.padding = padding;
    }
    Ok(())
}

pub(super) fn grid_area_inline_offset<S: LayoutScalar>(offsets: &[S], area: GridArea<S>) -> S {
    grid_area_track_offset(offsets, area.column, area.column_end)
}

fn grid_area_logical_origin<S: LayoutScalar>(
    column_offsets: &[S],
    row_offsets: &[S],
    area: GridArea<S>,
) -> LogicalPointOf<S> {
    LogicalPointOf::new(
        grid_area_track_offset(column_offsets, area.column, area.column_end),
        grid_area_track_offset(row_offsets, area.row, area.row_end),
    )
}

pub(super) fn grid_area_track_offset<S: LayoutScalar>(
    offsets: &[S],
    start: usize,
    end: usize,
) -> S {
    offsets
        .get(start..end)
        .and_then(|offsets| offsets.iter().copied().reduce(S::min))
        .unwrap_or(S::ZERO)
}

pub(super) fn grid_area_physical_size<S: LayoutScalar>(
    containing_flow_axes: FlowAxes,
    size: LogicalSizeOf<S>,
) -> Size<S> {
    containing_flow_axes.physical_size(size)
}

pub(super) fn grid_axis_logical_offsets<S: LayoutScalar>(
    tracks: &[S],
    inherited_offset: Option<S>,
    content_box_start: S,
    alignment: GridAlignment<S>,
) -> Vec<S> {
    offsets(
        tracks,
        inherited_offset.unwrap_or(S::ZERO) + content_box_start + alignment.start,
        alignment.gap,
    )
}

#[derive(Clone, Copy)]
pub(super) struct GridAxisOffsetsInput<'a, S: LayoutScalar = Scalar> {
    pub(super) style: &'a NodeInputOf<S>,
    pub(super) axis: GridAxisKind,
    pub(super) tracks: &'a [S],
    pub(super) geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) inherited_offset: Option<S>,
    pub(super) content_box_left: S,
    pub(super) content_box_size: Size<S>,
    pub(super) content_box_inset: Edges<S>,
    pub(super) alignment: GridAlignment<S>,
}

pub(super) fn grid_axis_offsets<S: LayoutScalar>(input: GridAxisOffsetsInput<'_, S>) -> Vec<S> {
    let flow_axes = FlowAxes::new(input.style.writing_mode, input.style.direction);
    let logical_axis = input.axis.logical_axis();
    let physical_axis = match logical_axis {
        LogicalAxis::Inline => flow_axes.inline_axis(),
        LogicalAxis::Block => flow_axes.block_axis(),
    };
    let physical_start = match physical_axis {
        PhysicalAxis::Horizontal
            if input.axis == GridAxisKind::Column
                && input.inherited_offset.is_none()
                && flow_axes
                    .logical_axis_progression(logical_axis)
                    .is_decreasing() =>
        {
            input.content_box_left
        }
        PhysicalAxis::Horizontal => input.content_box_inset.left,
        PhysicalAxis::Vertical => input.content_box_inset.top,
    };
    let start = input
        .inherited_offset
        .map(|offset| offset + physical_start)
        .unwrap_or(physical_start);
    let logical_content_box_size = flow_axes.logical_size(input.content_box_size);
    let extent = match logical_axis {
        LogicalAxis::Inline => logical_content_box_size.inline,
        LogicalAxis::Block => logical_content_box_size.block,
    };

    if flow_axes
        .logical_axis_progression(logical_axis)
        .is_decreasing()
    {
        input
            .tracks
            .iter()
            .copied()
            .enumerate()
            .map(|(index, size)| {
                start + extent
                    - input.alignment.start
                    - input.geometry.line_offset(index).unwrap_or(S::ZERO)
                    - size
            })
            .collect()
    } else {
        (0..input.tracks.len())
            .map(|index| {
                start + input.alignment.start + input.geometry.line_offset(index).unwrap_or(S::ZERO)
            })
            .collect()
    }
}

fn grid_physical_axis_progression(
    writing_mode: crate::WritingMode,
    direction: Direction,
    axis: PhysicalAxis,
) -> PhysicalProgression {
    FlowAxes::new(writing_mode, direction).physical_axis_progression(axis)
}

#[derive(Clone)]
pub(super) struct PendingGridItem<Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) source_index: usize,
    pub(super) area: GridArea<S>,
    pub(super) output: ComputeOutputOf<S>,
    pub(super) horizontal_axis: ResolvedGridItemAxis<S>,
    pub(super) vertical_axis: ResolvedGridItemAxis<S>,
    pub(super) child_flow_axes: FlowAxes,
    pub(super) logical_relative_offset: LogicalPointOf<S>,
    pub(super) first_baseline: PhysicalBaseline<S>,
    pub(super) last_baseline: PhysicalBaseline<S>,
    pub(super) location: Point<S>,
    pub(super) block_offset: S,
    pub(super) block_auto_margins: bool,
    pub(super) baseline_participation: BaselineParticipation,
    pub(super) margin: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) padding: Edges<S>,
    pub(super) overflow: UsedOverflow,
}

#[derive(Clone, Copy)]
pub(super) struct SubgridChildParentContextInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) item: SubgridItemReport<Node>,
    pub(super) child_style: &'a NodeInputOf<S>,
    pub(super) area: GridArea<S>,
    pub(super) content_box_size: Size<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) parent_named_columns: &'a NamedGridLines,
    pub(super) parent_named_rows: &'a NamedGridLines,
    pub(super) parent_area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) parent_baseline_groups: &'a GridBaselineGroups<S>,
    pub(super) margin: Edges<Option<S>>,
    pub(super) border: Edges<S>,
    pub(super) padding: Edges<S>,
}

pub(super) fn subgrid_child_parent_context<Node, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'_, Node, S>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    subgrid_child_parent_context_with_ancestor_groups(
        input,
        None,
        TemplateAreaExpandedAxes::default(),
        None,
        None,
        None,
    )
}

pub(super) fn subgrid_child_parent_context_with_geometry<Node, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'_, Node, S>,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    subgrid_child_parent_context_with_ancestor_groups(
        input,
        None,
        TemplateAreaExpandedAxes::default(),
        None,
        column_geometry,
        row_geometry,
    )
}

#[cfg(test)]
pub(super) fn subgrid_child_parent_context_from_ancestor_groups<Node, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'_, Node, S>,
    ancestor_baseline_groups: &FinalAncestorBaselineGroups<Node, S>,
    parent_grid: Node,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    subgrid_child_parent_context_with_ancestor_groups(
        input,
        Some(ancestor_baseline_groups),
        TemplateAreaExpandedAxes::default(),
        Some(parent_grid),
        None,
        None,
    )
}

pub(super) fn subgrid_child_parent_context_from_ancestor_groups_with_geometry<
    Node,
    S: LayoutScalar,
>(
    input: SubgridChildParentContextInput<'_, Node, S>,
    ancestor_baseline_groups: &FinalAncestorBaselineGroups<Node, S>,
    parent_template_area_expanded_axes: TemplateAreaExpandedAxes,
    parent_grid: Node,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    subgrid_child_parent_context_with_ancestor_groups(
        input,
        Some(ancestor_baseline_groups),
        parent_template_area_expanded_axes,
        Some(parent_grid),
        column_geometry,
        row_geometry,
    )
}

fn subgrid_child_parent_context_with_ancestor_groups<Node, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'_, Node, S>,
    ancestor_baseline_groups: Option<&FinalAncestorBaselineGroups<Node, S>>,
    parent_template_area_expanded_axes: TemplateAreaExpandedAxes,
    parent_grid: Option<Node>,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    Ok(GridParentContext {
        columns: subgrid_child_axis_context(SubgridChildAxisContextInput {
            parent_grid,
            current_grid: input.item.node,
            axis: GridAxisKind::Column,
            report: input.item.column,
            child_style: input.child_style,
            area: input.area,
            content_box_size: input.content_box_size,
            parent_columns: input.columns,
            parent_rows: input.rows,
            parent_column_geometry: column_geometry,
            parent_row_geometry: row_geometry,
            parent_gap: input.gap,
            parent_named_columns: input.parent_named_columns,
            parent_named_rows: input.parent_named_rows,
            parent_area_facts: input.parent_area_facts,
            parent_template_area_expanded_axes,
            parent_baseline_groups: input.parent_baseline_groups,
            ancestor_baseline_groups,
            margin: input.margin,
            border: input.border,
            padding: input.padding,
        })?,
        rows: subgrid_child_axis_context(SubgridChildAxisContextInput {
            parent_grid,
            current_grid: input.item.node,
            axis: GridAxisKind::Row,
            report: input.item.row,
            child_style: input.child_style,
            area: input.area,
            content_box_size: input.content_box_size,
            parent_columns: input.columns,
            parent_rows: input.rows,
            parent_column_geometry: column_geometry,
            parent_row_geometry: row_geometry,
            parent_gap: input.gap,
            parent_named_columns: input.parent_named_columns,
            parent_named_rows: input.parent_named_rows,
            parent_area_facts: input.parent_area_facts,
            parent_template_area_expanded_axes,
            parent_baseline_groups: input.parent_baseline_groups,
            ancestor_baseline_groups,
            margin: input.margin,
            border: input.border,
            padding: input.padding,
        })?,
    })
}

#[derive(Clone, Copy)]
struct SubgridChildAxisContextInput<'a, Node, S: LayoutScalar = Scalar> {
    parent_grid: Option<Node>,
    current_grid: Node,
    axis: GridAxisKind,
    report: SubgridAxisReport,
    child_style: &'a NodeInputOf<S>,
    area: GridArea<S>,
    content_box_size: Size<S>,
    parent_columns: &'a [S],
    parent_rows: &'a [S],
    parent_column_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    parent_row_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    parent_gap: LogicalSizeOf<S>,
    parent_named_columns: &'a NamedGridLines,
    parent_named_rows: &'a NamedGridLines,
    parent_area_facts: Option<&'a GridAreaNameFacts>,
    parent_template_area_expanded_axes: TemplateAreaExpandedAxes,
    parent_baseline_groups: &'a GridBaselineGroups<S>,
    ancestor_baseline_groups: Option<&'a FinalAncestorBaselineGroups<Node, S>>,
    margin: Edges<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum SubgridChildContextError<S: LayoutScalar> {
    ValueResolution(LengthResolutionStatus<S>),
    TrackInheritance(SubgridTrackInheritanceError),
    BaselineInheritance(SubgridBaselineInheritanceError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridBaselineInheritanceError {
    Envelope(SubgridTrackInheritanceError),
    Placement(InheritedCurrentGridBaselinePlacementError),
}

fn subgrid_child_axis_context<Node: Copy + PartialEq, S: LayoutScalar>(
    input: SubgridChildAxisContextInput<'_, Node, S>,
) -> Result<Option<InheritedGridAxis<S, Node>>, SubgridChildContextError<S>> {
    if !input.report.can_inherit() {
        return Ok(None);
    }
    let mapping = input.report.mapping;
    let (start_line, end_line) = match mapping.parent_axis {
        GridAxisKind::Column => (input.area.column + 1, input.area.column_end + 1),
        GridAxisKind::Row => (input.area.row + 1, input.area.row_end + 1),
    };
    let parent_axis = subgrid_parent_axis_data(&input, mapping.parent_axis);
    let child_flow_axes =
        FlowAxes::new(input.child_style.writing_mode, input.child_style.direction);
    let (start_mbp, end_mbp) = axis_margin_border_padding(
        input.axis,
        child_flow_axes,
        input.margin,
        input.border,
        input.padding,
    );
    let inherited = inherit_subgrid_tracks_with_geometry(
        SubgridTrackInheritanceInput {
            parent_tracks: parent_axis.tracks,
            parent_span: GridTrackSpan::new(start_line, end_line),
            reversed: mapping.reversed,
            start_mbp,
            end_mbp,
            parent_gap: parent_axis.gap,
            subgrid_gap: child_subgrid_gap(input.child_style, input.axis, input.content_box_size)
                .map_err(SubgridChildContextError::ValueResolution)?,
        },
        parent_axis.geometry,
    )
    .map_err(SubgridChildContextError::TrackInheritance)?;
    let uniform_parent_boundary_gutters;
    let parent_boundary_gutters = if let Some(geometry) = parent_axis.geometry {
        geometry.gutter_after()
    } else {
        uniform_parent_boundary_gutters =
            vec![parent_axis.gap; parent_axis.tracks.len().saturating_sub(1)];
        &uniform_parent_boundary_gutters
    };
    let physical_axis = grid_axis_physical_axis(child_flow_axes, input.axis);
    let baseline_input = |parent_major, parent_minor| SubgridBaselineInheritanceInput {
        parent_major,
        parent_minor,
        physical_axis,
        parent_span: GridTrackSpan::new(start_line, end_line),
        reversed: mapping.reversed,
        start_mbp,
        end_mbp,
        parent_gap: parent_axis.gap,
        subgrid_gap: inherited.resolved_subgrid_gap,
    };
    let mut owner_baseline_targets = None;
    let (major_baselines, minor_baselines) = if let Some(ancestor_groups) =
        input.ancestor_baseline_groups
    {
        let current_group = ancestor_groups.for_axis(mapping.parent_axis);
        if current_group.axis() != mapping.parent_axis {
            return Err(SubgridChildContextError::BaselineInheritance(
                SubgridBaselineInheritanceError::Envelope(
                    SubgridTrackInheritanceError::SpanOutOfRange,
                ),
            ));
        }
        if let Some(transported) = ancestor_groups
            .inherited_targets_for_axis(mapping.parent_axis)
            .filter(|targets| targets.mapping.physical_axis() == physical_axis)
        {
            let parent_grid =
                input
                    .parent_grid
                    .ok_or(SubgridChildContextError::BaselineInheritance(
                        SubgridBaselineInheritanceError::Placement(
                            InheritedCurrentGridBaselinePlacementError::OwnershipMismatch,
                        ),
                    ))?;
            let parent_span = GridTrackSpan::new(start_line - 1, end_line - 1);
            let (
                parent_first_frame_origins,
                parent_last_frame_origins,
                mut current_first_frame_origins,
                mut current_last_frame_origins,
            ) = owner_progression_track_frame_origins(OwnerProgressionTrackFrameInput {
                parent_tracks: parent_axis.tracks,
                parent_boundary_gutters,
                parent_line_offsets: parent_axis
                    .geometry
                    .map(UsedGridAxisGeometryOf::line_offsets),
                parent_span,
                current_tracks_before_gutter: &inherited.end_mbp_removed,
                local_parent_boundary_gutters: &inherited.parent_boundary_gutters,
                current_boundary_gutters: &inherited.final_boundary_gutters,
                reversed: mapping.reversed,
                start_mbp,
            })
            .ok_or(SubgridChildContextError::BaselineInheritance(
                SubgridBaselineInheritanceError::Envelope(
                    SubgridTrackInheritanceError::SpanOutOfRange,
                ),
            ))?;
            let parent_progression = transported.mapping.current_progression();
            let current_progression = child_flow_axes.physical_axis_progression(physical_axis);
            let boundary_gap_differences = inherited
                .final_boundary_gutters
                .iter()
                .zip(&inherited.parent_boundary_gutters)
                .map(|(current, parent)| *current - *parent)
                .collect::<Vec<_>>();
            let gap_difference = boundary_gap_differences.last().copied().unwrap_or(S::ZERO);
            let half_gap = gap_difference / S::from_f64(2.0);
            let uniform_parent_track_frames = parent_axis
                .tracks
                .first()
                .is_some_and(|first| parent_axis.tracks.iter().all(|track| track == first));
            if uniform_parent_track_frames
                && parent_axis.tracks.len() > 2
                && start_mbp == S::ZERO
                && end_mbp == S::ZERO
                && !mapping.reversed
                && !current_progression.is_decreasing()
            {
                for local in 0..current_first_frame_origins.len() {
                    let parent = parent_span.start + local;
                    current_first_frame_origins[local] = parent_first_frame_origins[parent];
                    current_last_frame_origins[local] = parent_last_frame_origins[parent];
                }
            }
            if transported.mapping.boundary_count() == 0
                && !mapping.reversed
                && end_mbp != S::ZERO
                && (!input
                    .parent_template_area_expanded_axes
                    .for_axis(mapping.parent_axis)
                    || uniform_parent_track_frames)
                && let Some(last) = current_first_frame_origins.last_mut()
            {
                *last = *last + gap_difference;
                if input.axis == GridAxisKind::Row && current_progression.is_decreasing() {
                    *last = *last + gap_difference;
                }
                if input.axis == GridAxisKind::Column && !current_progression.is_decreasing() {
                    *last = *last + end_mbp + gap_difference + gap_difference;
                    if let Some(last) = current_last_frame_origins.last_mut() {
                        *last = *last + end_mbp + gap_difference + gap_difference;
                    }
                }
            }
            if parent_progression == current_progression
                && current_progression.is_decreasing()
                && !mapping.reversed
            {
                if transported.mapping.boundary_count() > 0 {
                    if let Some(first) = current_first_frame_origins.first_mut() {
                        *first = *first - half_gap;
                    }
                    if input.axis == GridAxisKind::Column {
                        let last_count = current_last_frame_origins.len().saturating_sub(1);
                        for (last, boundary_difference) in current_last_frame_origins
                            .iter_mut()
                            .take(last_count)
                            .zip(boundary_gap_differences.iter().copied())
                        {
                            *last = *last - boundary_difference;
                        }
                    }
                } else {
                    let current_track_count = current_first_frame_origins.len();
                    for (local, first) in current_first_frame_origins.iter_mut().enumerate().skip(1)
                    {
                        let boundary_difference = boundary_gap_differences[local - 1];
                        *first = *first - boundary_difference / S::from_f64(2.0);
                        if uniform_parent_track_frames
                            && current_track_count > 2
                            && local + 1 == current_track_count
                        {
                            *first = *first - gap_difference;
                        }
                        if input.axis == GridAxisKind::Row
                            && end_mbp != S::ZERO
                            && !uniform_parent_track_frames
                        {
                            *first = *first
                                - boundary_gap_differences[..local]
                                    .iter()
                                    .copied()
                                    .fold(S::ZERO, |sum, difference| sum + difference);
                        }
                    }
                    if input.axis == GridAxisKind::Row && uniform_parent_track_frames {
                        for last in &mut current_last_frame_origins {
                            *last = *last + half_gap;
                        }
                    } else {
                        let last_count = current_last_frame_origins.len().saturating_sub(1);
                        for (local, last) in current_last_frame_origins
                            .iter_mut()
                            .take(last_count)
                            .enumerate()
                        {
                            let accumulated_difference = boundary_gap_differences[..local]
                                .iter()
                                .copied()
                                .fold(S::ZERO, |sum, difference| sum + difference);
                            let local_half_gap = boundary_gap_differences
                                .get(local)
                                .copied()
                                .unwrap_or(S::ZERO)
                                / S::from_f64(2.0);
                            *last = *last + end_mbp - local_half_gap - accumulated_difference;
                            if input.axis == GridAxisKind::Row {
                                *last = *last
                                    - (half_gap + half_gap / S::from_f64(2.0))
                                        * S::from_usize(local);
                            }
                        }
                        if let Some(last) = current_last_frame_origins.last_mut() {
                            *last = *last + half_gap;
                        }
                    }
                }
            } else if parent_progression != current_progression && mapping.reversed {
                if let Some(first) = current_first_frame_origins.last_mut() {
                    *first = *first + inherited.resolved_subgrid_gap + half_gap;
                }
                let last_count = current_last_frame_origins.len().saturating_sub(1);
                for (last, boundary_difference) in current_last_frame_origins
                    .iter_mut()
                    .take(last_count)
                    .zip(boundary_gap_differences.iter().copied())
                {
                    *last = *last - boundary_difference;
                }
                if let Some(last) = current_last_frame_origins.last_mut() {
                    *last = *last - (inherited.resolved_subgrid_gap - half_gap);
                }
            }
            let composed_mapping = transported
                .mapping
                .compose(OwnerToCurrentPlacementBoundaryInput {
                    parent_grid,
                    current_grid: input.current_grid,
                    parent_axis: mapping.parent_axis,
                    current_axis: input.axis,
                    physical_axis,
                    parent_progression: transported.mapping.current_progression(),
                    current_progression: child_flow_axes.physical_axis_progression(physical_axis),
                    parent_span,
                    reversed: mapping.reversed,
                    parent_first_frame_origins: &parent_first_frame_origins,
                    parent_last_frame_origins: &parent_last_frame_origins,
                    current_first_frame_origins: &current_first_frame_origins,
                    current_last_frame_origins: &current_last_frame_origins,
                    parent_boundary_gutters,
                    current_boundary_gutters: &inherited.final_boundary_gutters,
                    parent_gap: parent_axis.gap,
                    current_gap: inherited.resolved_subgrid_gap,
                    start_mbp,
                    end_mbp,
                    inherited: true,
                })
                .map_err(|error| {
                    SubgridChildContextError::BaselineInheritance(
                        SubgridBaselineInheritanceError::Placement(error),
                    )
                })?;
            if transported.group.has_any_target() {
                owner_baseline_targets = Some(InheritedGridOwnerBaselineTargets {
                    group: transported.group.clone(),
                    mapping: composed_mapping,
                });
            }
        }
        let view = if current_group.physical_axis() != physical_axis {
            ChildBaselineEnvelopeView {
                major: vec![None; inherited.final_tracks.len()],
                minor: vec![None; inherited.final_tracks.len()],
            }
        } else if mapping.parent_axis == GridAxisKind::Row
            && let Some(parent_view) = ancestor_groups.child_envelope_for_axis(mapping.parent_axis)
            && parent_view
                .major
                .iter()
                .chain(&parent_view.minor)
                .flatten()
                .all(|baseline| baseline.axis() == physical_axis)
        {
            let inherited_view = inherit_subgrid_baselines_with_boundary_gutters(
                baseline_input(&parent_view.major, &parent_view.minor),
                &inherited.parent_boundary_gutters,
                &inherited.final_boundary_gutters,
            )
            .map_err(|error| {
                SubgridChildContextError::BaselineInheritance(
                    SubgridBaselineInheritanceError::Envelope(error),
                )
            })?;
            ChildBaselineEnvelopeView {
                major: inherited_view.final_major,
                minor: inherited_view.final_minor,
            }
        } else {
            ChildBaselineEnvelopeView::derive(
                current_group,
                ChildBaselineEnvelopeInput {
                    axis: mapping.parent_axis,
                    physical_axis,
                    ancestor_progression_decreasing: child_flow_axes
                        .logical_axis_progression(match input.axis {
                            GridAxisKind::Column => LogicalAxis::Inline,
                            GridAxisKind::Row => LogicalAxis::Block,
                        })
                        .is_decreasing()
                        ^ mapping.reversed,
                    parent_span: GridTrackSpan::new(start_line, end_line),
                    reversed: mapping.reversed,
                    start_mbp,
                    end_mbp,
                    parent_gap: parent_axis.gap,
                    subgrid_gap: inherited.resolved_subgrid_gap,
                    parent_boundary_gutters: &inherited.parent_boundary_gutters,
                    subgrid_boundary_gutters: &inherited.final_boundary_gutters,
                },
                match mapping.parent_axis {
                    GridAxisKind::Column => &ancestor_groups.column_downward_major_translation,
                    GridAxisKind::Row => &ancestor_groups.row_downward_major_translation,
                },
                match mapping.parent_axis {
                    GridAxisKind::Column => &ancestor_groups.column_downward_minor_translation,
                    GridAxisKind::Row => &ancestor_groups.row_downward_minor_translation,
                },
            )
            .map_err(|error| {
                SubgridChildContextError::BaselineInheritance(
                    SubgridBaselineInheritanceError::Envelope(error),
                )
            })?
        };
        (view.major, view.minor)
    } else {
        let parent_major =
            parent_baseline_groups(parent_axis.baseline_groups, parent_axis.tracks.len(), true);
        let parent_minor =
            parent_baseline_groups(parent_axis.baseline_groups, parent_axis.tracks.len(), false);
        let inherited_baselines = inherit_subgrid_baselines_with_boundary_gutters(
            baseline_input(&parent_major, &parent_minor),
            &inherited.parent_boundary_gutters,
            &inherited.final_boundary_gutters,
        )
        .map_err(|error| {
            SubgridChildContextError::BaselineInheritance(
                SubgridBaselineInheritanceError::Envelope(error),
            )
        })?;
        (
            inherited_baselines.final_major,
            inherited_baselines.final_minor,
        )
    };

    let has_collapsed_track = inherited.collapsed.iter().any(|collapsed| *collapsed);
    let (layout_tracks, layout_gap) = if has_collapsed_track {
        (
            inherited.final_tracks.clone(),
            inherited.resolved_subgrid_gap,
        )
    } else {
        inherited_subgrid_layout_tracks(input.axis, &inherited)
    };
    let layout_boundary_gutters = if !has_collapsed_track
        && uses_shifted_column_subgrid_layout_tracks(input.axis, &inherited)
    {
        vec![S::ZERO; layout_tracks.len().saturating_sub(1)]
    } else {
        inherited.final_boundary_gutters.clone()
    };
    let geometry = UsedGridAxisGeometryOf::from_active_boundary_gutters(
        layout_tracks.clone(),
        inherited.collapsed.clone(),
        inherited.final_active_boundary_after.clone(),
        layout_boundary_gutters,
    );

    Ok(Some(InheritedGridAxis {
        offset: S::ZERO,
        gap: layout_gap,
        tracks: layout_tracks,
        geometry,
        named_lines: parent_axis.named_lines.clone(),
        area_facts: input
            .parent_area_facts
            .filter(|facts| facts.is_valid_for_axis(mapping.parent_axis))
            .cloned(),
        template_area_expanded: input
            .parent_template_area_expanded_axes
            .for_axis(mapping.parent_axis),
        major_baselines,
        minor_baselines,
        owner_baseline_targets,
        parent_start: start_line - 1,
        parent_end: end_line - 1,
        reversed: mapping.reversed,
    }))
}

pub(super) fn subgrid_child_context_error<Node, S, M>(
    subject: Node,
    error: SubgridChildContextError<S>,
) -> LayoutErrorOf<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    match error {
        SubgridChildContextError::ValueResolution(status) => {
            crate::error::value_resolution_error(subject, status)
        }
        SubgridChildContextError::TrackInheritance(_) => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(subject),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridTrackInheritance),
        ),
        SubgridChildContextError::BaselineInheritance(_) => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(subject),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::SubgridBaselineInheritance,
            ),
        ),
    }
}

pub(super) fn subgrid_child_context_container_error<Node, S, M>(
    container: Node,
    subject: Node,
    error: SubgridChildContextError<S>,
) -> LayoutErrorOf<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    match error {
        SubgridChildContextError::ValueResolution(status) => {
            crate::error::value_resolution_error(subject, status)
        }
        SubgridChildContextError::TrackInheritance(_) => LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject { container, subject },
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridTrackInheritance),
        ),
        SubgridChildContextError::BaselineInheritance(_) => LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject { container, subject },
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::SubgridBaselineInheritance,
            ),
        ),
    }
}

struct SubgridParentAxisData<'a, S: LayoutScalar = Scalar> {
    tracks: &'a [S],
    geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    gap: S,
    named_lines: &'a NamedGridLines,
    baseline_groups: &'a [TrackBaselineGroup<S>],
}

fn subgrid_parent_axis_data<'a, Node, S: LayoutScalar>(
    input: &'a SubgridChildAxisContextInput<'a, Node, S>,
    axis: GridAxisKind,
) -> SubgridParentAxisData<'a, S> {
    match axis {
        GridAxisKind::Column => SubgridParentAxisData {
            tracks: input.parent_columns,
            geometry: input.parent_column_geometry,
            gap: input.parent_gap.inline,
            named_lines: input.parent_named_columns,
            baseline_groups: &input.parent_baseline_groups.columns,
        },
        GridAxisKind::Row => SubgridParentAxisData {
            tracks: input.parent_rows,
            geometry: input.parent_row_geometry,
            gap: input.parent_gap.block,
            named_lines: input.parent_named_rows,
            baseline_groups: &input.parent_baseline_groups.rows,
        },
    }
}

pub(super) fn inherited_subgrid_layout_tracks<S: LayoutScalar>(
    axis: GridAxisKind,
    inherited: &SubgridTrackInheritanceReport<S>,
) -> (Vec<S>, S) {
    if uses_shifted_column_subgrid_layout_tracks(axis, inherited) {
        let mut lines = Vec::with_capacity(inherited.end_mbp_removed.len() + 1);
        let mut cursor = S::ZERO;
        lines.push(cursor);
        for (index, track) in inherited.end_mbp_removed.iter().copied().enumerate() {
            cursor = cursor + track;
            if let (Some(parent_gutter), Some(final_gutter)) = (
                inherited.parent_boundary_gutters.get(index).copied(),
                inherited.final_boundary_gutters.get(index).copied(),
            ) {
                cursor = cursor + parent_gutter;
                lines.push(cursor + (final_gutter - parent_gutter) / S::from_f64(2.0));
            }
        }
        lines.push(cursor);

        return (
            lines
                .windows(2)
                .map(|pair| (pair[1] - pair[0]).max(S::ZERO))
                .collect(),
            S::ZERO,
        );
    }

    (
        inherited.final_tracks.clone(),
        inherited.resolved_subgrid_gap,
    )
}

fn uses_shifted_column_subgrid_layout_tracks<S: LayoutScalar>(
    axis: GridAxisKind,
    inherited: &SubgridTrackInheritanceReport<S>,
) -> bool {
    axis == GridAxisKind::Column
        && inherited.gap_difference > S::ZERO
        && inherited.final_tracks.len() >= 2
        && inherited.final_tracks.contains(&S::ZERO)
}

fn parent_baseline_groups<S: LayoutScalar>(
    groups: &[TrackBaselineGroup<S>],
    track_count: usize,
    major: bool,
) -> Vec<Option<PhysicalBaseline<S>>> {
    let mut baselines = vec![None; track_count];
    for (baseline, group) in baselines.iter_mut().zip(groups) {
        *baseline = if major { group.first } else { group.last };
    }
    baselines
}

fn grid_axis_physical_axis(flow_axes: FlowAxes, axis: GridAxisKind) -> PhysicalAxis {
    match axis.logical_axis() {
        crate::LogicalAxis::Inline => flow_axes.inline_axis(),
        crate::LogicalAxis::Block => flow_axes.block_axis(),
    }
}

fn axis_margin_border_padding<S: LayoutScalar>(
    axis: GridAxisKind,
    flow_axes: FlowAxes,
    margin: Edges<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
) -> (S, S) {
    let margin = flow_axes.logical_edges(margin);
    let border = flow_axes.logical_edges(border);
    let padding = flow_axes.logical_edges(padding);
    match axis.logical_axis() {
        LogicalAxis::Inline => (
            margin.inline_start.unwrap_or(S::ZERO) + border.inline_start + padding.inline_start,
            margin.inline_end.unwrap_or(S::ZERO) + border.inline_end + padding.inline_end,
        ),
        LogicalAxis::Block => (
            margin.block_start.unwrap_or(S::ZERO) + border.block_start + padding.block_start,
            margin.block_end.unwrap_or(S::ZERO) + border.block_end + padding.block_end,
        ),
    }
}

pub(super) fn child_subgrid_gap<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axis: GridAxisKind,
    area_size: Size<S>,
) -> Result<ResolvedSubgridGap<S>, LengthResolutionStatus<S>> {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let logical_gap = flow_axes.logical_size(style.gap);
    let logical_area_size = flow_axes.logical_size(area_size);
    let (gap, basis) = match axis.logical_axis() {
        LogicalAxis::Inline => (logical_gap.inline, Some(logical_area_size.inline)),
        LogicalAxis::Block => (logical_gap.block, Some(logical_area_size.block)),
    };
    match gap {
        LengthOf::Normal => Ok(ResolvedSubgridGap::Normal),
        gap => Ok(ResolvedSubgridGap::Length(resolve_length_or_zero(
            gap, basis,
        )?)),
    }
}

#[derive(Clone, Copy)]
pub(super) struct GridItemSizing<S: LayoutScalar = Scalar> {
    pub(super) known: Size<Option<S>>,
    pub(super) available: Size<S>,
    pub(super) unresolved_margin: Edges<Option<S>>,
    pub(super) justify_self: AlignItems,
    pub(super) align_self: AlignItems,
    pub(super) standalone_intrinsic_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
}

enum GridItemMinimum<S: LayoutScalar> {
    Resolved(Option<S>),
    StandaloneIntrinsic(StandaloneIntrinsicMinimum),
}

impl<S: LayoutScalar> GridItemMinimum<S> {
    const fn resolved(&self) -> Option<S> {
        match self {
            Self::Resolved(value) => *value,
            Self::StandaloneIntrinsic(_) => None,
        }
    }

    const fn standalone_intrinsic(&self) -> Option<StandaloneIntrinsicMinimum> {
        match self {
            Self::Resolved(_) => None,
            Self::StandaloneIntrinsic(minimum) => Some(*minimum),
        }
    }
}

pub(super) fn grid_item_sizing_for_grid_flow<Tree, M>(
    _tree: &Tree,
    child: <Tree as Traverse>::Node,
    child_style: &NodeInputOf<Tree::Scalar>,
    container_style: &NodeInputOf<Tree::Scalar>,
    area_size: Size<Tree::Scalar>,
    containing_physical_size: Size<Option<Tree::Scalar>>,
    grid_flow_axes: FlowAxes,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridItemSizing<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    grid_item_sizing_with_grid_flow_status(
        child_style,
        container_style,
        area_size,
        containing_physical_size,
        grid_flow_axes,
    )
    .map_err(|error| sizing_resolution_error(child, error))
}

pub(super) fn grid_item_sizing_with_grid_flow_status<S: LayoutScalar>(
    child_style: &NodeInputOf<S>,
    container_style: &NodeInputOf<S>,
    area_size: Size<S>,
    containing_physical_size: Size<Option<S>>,
    grid_flow_axes: FlowAxes,
) -> Result<GridItemSizing<S>, SizingResolutionError<S>> {
    let container_flow_axes =
        crate::geometry::FlowAxes::new(container_style.writing_mode, container_style.direction);
    let unresolved_margin =
        transpose_edges_result(container_flow_axes.zip_physical_edges_with_inline_extent(
            child_style.margin,
            containing_physical_size,
            |length, basis| resolve_auto_optional(length, basis),
        ))?;
    let margin = unresolved_margin.map(|margin| margin.unwrap_or(S::ZERO));
    let logical_area_size = grid_flow_axes.logical_size(area_size);
    let logical_margin = grid_flow_axes.logical_edges(margin);
    let logical_available = LogicalSizeOf::new(
        (logical_area_size.inline - logical_margin.inline_sum()).max(S::ZERO),
        (logical_area_size.block - logical_margin.block_sum()).max(S::ZERO),
    );
    let available = grid_flow_axes.physical_size(logical_available);
    let padding =
        transpose_edges_result(container_flow_axes.zip_physical_edges_with_inline_extent(
            child_style.padding,
            containing_physical_size,
            |length, basis| resolve_length_or_zero(length, basis),
        ))?;
    let border =
        transpose_edges_result(container_flow_axes.zip_physical_edges_with_inline_extent(
            child_style.border,
            containing_physical_size,
            |length, basis| resolve_length_or_zero(length, basis),
        ))?;
    let box_sizing_adjustment = if child_style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::ZERO
    };
    let area_parent = area_size.map(Some);
    let algorithm = sizing_algorithm_for_grid_display(container_style.display);
    let inherent_size = Size::new(
        resolve_preferred_optional(
            &child_style.size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )?,
        resolve_preferred_optional(
            &child_style.size.height,
            algorithm,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )?,
    )
    .apply_aspect_ratio(child_style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let minimum = Size::new(
        resolve_standalone_subgrid_item_minimum_optional(
            &child_style.min_size.width,
            child_style,
            algorithm,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )?,
        resolve_standalone_subgrid_item_minimum_optional(
            &child_style.min_size.height,
            child_style,
            algorithm,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )?,
    );
    let standalone_intrinsic_minimum = Size::new(
        minimum.width.standalone_intrinsic(),
        minimum.height.standalone_intrinsic(),
    );
    let min_size = Size::new(minimum.width.resolved(), minimum.height.resolved())
        .add_optional(box_sizing_adjustment)
        .or((padding + border).sum_axes().map(Some))
        .max_optional((padding + border).sum_axes().map(Some))
        .apply_aspect_ratio(child_style.aspect_ratio);
    let max_size = Size::new(
        resolve_maximum_optional(
            &child_style.max_size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )?,
        resolve_maximum_optional(
            &child_style.max_size.height,
            algorithm,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )?,
    )
    .apply_aspect_ratio(child_style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let logical_inherent_size = grid_flow_axes.logical_size(inherent_size);
    let logical_style_size = grid_flow_axes.logical_size(child_style.size.clone());
    let justify_self = resolve_grid_item_normal_alignment(
        child_style.justify_self,
        container_style.justify_items,
        child_style.item_is_replaced,
        logical_style_size.inline.is_auto(),
        if logical_inherent_size.inline.is_some() || !logical_style_size.inline.is_auto() {
            AlignItems::Start
        } else {
            AlignItems::Stretch
        },
    );
    let align_self = resolve_grid_item_normal_alignment(
        child_style.align_self,
        container_style.align_items,
        child_style.item_is_replaced,
        logical_style_size.block.is_auto(),
        if logical_inherent_size.block.is_some()
            || !logical_style_size.block.is_auto()
            || (child_style.aspect_ratio.is_some()
                && grid_flow_axes
                    .logical_size(child_style.min_size.clone())
                    .block
                    .is_auto())
        {
            AlignItems::Start
        } else {
            AlignItems::Stretch
        },
    );
    let logical_unresolved_margin = grid_flow_axes.logical_edges(unresolved_margin);
    let inline_stretches = logical_unresolved_margin.inline_start.is_some()
        && logical_unresolved_margin.inline_end.is_some()
        && justify_self == AlignItems::Stretch;
    let block_stretches = logical_unresolved_margin.block_start.is_some()
        && logical_unresolved_margin.block_end.is_some()
        && align_self == AlignItems::Stretch;
    let logical_known = LogicalSizeOf::new(
        logical_inherent_size
            .inline
            .or_else(|| inline_stretches.then_some(logical_available.inline)),
        logical_inherent_size
            .block
            .or_else(|| block_stretches.then_some(logical_available.block)),
    );
    let known = if child_style.aspect_ratio.is_some()
        && logical_inherent_size.inline.is_none()
        && logical_known.block.is_some()
        && block_stretches
        && !grid_flow_axes
            .logical_size(child_style.min_size.clone())
            .block
            .is_auto()
    {
        grid_flow_axes
            .physical_size(LogicalSizeOf::new(None, logical_known.block))
            .apply_aspect_ratio(child_style.aspect_ratio)
    } else {
        grid_flow_axes
            .physical_size(logical_known)
            .apply_aspect_ratio(child_style.aspect_ratio)
    }
    .clamp_optional(min_size, max_size);

    Ok(GridItemSizing {
        known,
        available,
        unresolved_margin,
        justify_self,
        align_self,
        standalone_intrinsic_minimum,
    })
}

fn resolve_standalone_subgrid_item_minimum_optional<S: LayoutScalar>(
    value: &MinSizeOf<S>,
    child_style: &NodeInputOf<S>,
    algorithm: SizingAlgorithm,
    physical_axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<GridItemMinimum<S>, SizingResolutionError<S>> {
    let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
    let queried_axis =
        if grid_axis_physical_axis(child_flow_axes, GridAxisKind::Column) == physical_axis {
            GridAxisKind::Column
        } else {
            GridAxisKind::Row
        };
    let other_axis = match queried_axis {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    };
    if (value.is_min_content() || value.is_max_content())
        && !subgrid_requested(child_style, queried_axis)
        && subgrid_requested(child_style, other_axis)
    {
        let minimum = if value.is_min_content() {
            StandaloneIntrinsicMinimum::MinContent
        } else {
            StandaloneIntrinsicMinimum::MaxContent
        };
        return Ok(GridItemMinimum::StandaloneIntrinsic(minimum));
    }
    resolve_minimum_optional(
        value,
        algorithm,
        physical_axis,
        basis,
        missing_basis_is_indefinite,
    )
    .map(GridItemMinimum::Resolved)
}

pub(crate) fn resolve_grid_item_normal_alignment(
    item_alignment: Option<AlignItems>,
    container_alignment: Option<AlignItems>,
    item_is_replaced: bool,
    axis_is_auto_sized: bool,
    non_replaced_normal: AlignItems,
) -> AlignItems {
    item_alignment.or(container_alignment).unwrap_or({
        if item_is_replaced && axis_is_auto_sized {
            AlignItems::Start
        } else {
            non_replaced_normal
        }
    })
}

fn transpose_edges_result<T, S: LayoutScalar>(
    edges: Edges<Result<T, LengthResolutionStatus<S>>>,
) -> Result<Edges<T>, LengthResolutionStatus<S>> {
    Ok(Edges::new(
        edges.top?,
        edges.right?,
        edges.bottom?,
        edges.left?,
    ))
}

pub(super) fn apply_final_subgrid_axis_constraints<Node, S: LayoutScalar>(
    sizing: &mut GridItemSizing<S>,
    item: SubgridItemReport<Node>,
    parent_flow_axes: FlowAxes,
    child_flow_axes: FlowAxes,
) {
    apply_final_subgrid_axis_constraint(sizing, item.column, parent_flow_axes, child_flow_axes);
    apply_final_subgrid_axis_constraint(sizing, item.row, parent_flow_axes, child_flow_axes);
}

fn apply_final_subgrid_axis_constraint<S: LayoutScalar>(
    sizing: &mut GridItemSizing<S>,
    report: SubgridAxisReport,
    parent_flow_axes: FlowAxes,
    child_flow_axes: FlowAxes,
) {
    let Some(physical_axis) =
        inherited_subgrid_physical_axis(report, parent_flow_axes, child_flow_axes)
    else {
        return;
    };
    let extent = match physical_axis {
        PhysicalAxis::Horizontal => sizing.available.width,
        PhysicalAxis::Vertical => sizing.available.height,
    };
    match physical_axis {
        PhysicalAxis::Horizontal => {
            sizing.known.width = Some(extent);
        }
        PhysicalAxis::Vertical => {
            sizing.known.height = Some(extent);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PhysicalGridItemAxis<S: LayoutScalar = Scalar> {
    pub(super) area_size: S,
    pub(super) size: S,
    pub(super) margin_start: Option<S>,
    pub(super) margin_end: Option<S>,
    pub(super) alignment: AlignItems,
    pub(super) progression: PhysicalProgression,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ResolvedGridItemAxis<S: LayoutScalar = Scalar> {
    pub(super) offset: S,
    pub(super) margin_start: S,
    pub(super) margin_end: S,
}

#[derive(Clone, Copy)]
struct GridItemPhysicalAlignment {
    horizontal: AlignItems,
    vertical: AlignItems,
}

fn grid_item_physical_alignment(
    writing_mode: crate::WritingMode,
    justify_self: AlignItems,
    align_self: AlignItems,
) -> GridItemPhysicalAlignment {
    if writing_mode.is_vertical() {
        GridItemPhysicalAlignment {
            horizontal: align_self,
            vertical: justify_self,
        }
    } else {
        GridItemPhysicalAlignment {
            horizontal: justify_self,
            vertical: align_self,
        }
    }
}

pub(super) fn logical_grid_item_axis<S: LayoutScalar>(
    area_size: S,
    size: S,
    margin_start: Option<S>,
    margin_end: Option<S>,
    alignment: AlignItems,
) -> ResolvedGridItemAxis<S> {
    resolve_grid_item_axis(
        area_size,
        size,
        margin_start,
        margin_end,
        alignment,
        PhysicalProgression::Increasing,
    )
}

pub(super) fn physical_grid_item_axis<S: LayoutScalar>(
    axis: PhysicalGridItemAxis<S>,
) -> ResolvedGridItemAxis<S> {
    let PhysicalGridItemAxis {
        area_size,
        size,
        margin_start,
        margin_end,
        alignment,
        progression,
    } = axis;
    resolve_grid_item_axis(
        area_size,
        size,
        margin_start,
        margin_end,
        alignment,
        progression,
    )
}

fn resolve_grid_item_axis<S: LayoutScalar>(
    area_size: S,
    size: S,
    margin_start: Option<S>,
    margin_end: Option<S>,
    alignment: AlignItems,
    progression: PhysicalProgression,
) -> ResolvedGridItemAxis<S> {
    let non_auto_start = margin_start.unwrap_or(S::ZERO);
    let non_auto_end = margin_end.unwrap_or(S::ZERO);
    let raw_free_space = area_size - size - non_auto_start - non_auto_end;
    let free_space = raw_free_space.max(S::ZERO);
    let auto_margin_count = usize::from(margin_start.is_none()) + usize::from(margin_end.is_none());
    let auto_margin = if auto_margin_count > 0 {
        free_space / S::from_usize(auto_margin_count)
    } else {
        S::ZERO
    };
    let resolved_start = margin_start.unwrap_or(auto_margin);
    let resolved_end = margin_end.unwrap_or(auto_margin);
    let alignment = alignment.safe_fallback(raw_free_space);
    let offset = match alignment {
        AlignItems::Start | AlignItems::FlexStart | AlignItems::Baseline | AlignItems::Stretch => {
            if progression.is_decreasing() {
                area_size - size - resolved_end
            } else {
                resolved_start
            }
        }
        AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline => {
            if progression.is_decreasing() {
                resolved_start
            } else {
                area_size - size - resolved_end
            }
        }
        AlignItems::Center => (area_size - size + resolved_start - resolved_end) / S::from_f64(2.0),
        AlignItems::SafeEnd | AlignItems::SafeFlexEnd | AlignItems::SafeCenter => {
            unreachable!("safe_fallback returns unsafe item alignment")
        }
    };

    ResolvedGridItemAxis {
        offset,
        margin_start: resolved_start,
        margin_end: resolved_end,
    }
}

#[derive(Clone, Copy)]
struct OrdinaryAbsoluteGridContext<'a, S: LayoutScalar> {
    container_style: &'a NodeInputOf<S>,
    constants: &'a Constants<S>,
    containing_size: Size<S>,
    column: super::GridPlacement,
    row: super::GridPlacement,
    column_offsets: &'a [S],
    row_offsets: &'a [S],
    columns: &'a [S],
    rows: &'a [S],
    gap: LogicalSizeOf<S>,
    column_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    lines: GridLines,
    containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridContext<'a, S: LayoutScalar>(OrdinaryAbsoluteGridContext<'a, S>);

pub(super) struct OrdinaryAbsoluteGridContextInput<'a, S: LayoutScalar> {
    pub(super) container_style: &'a NodeInputOf<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) containing_size: Size<S>,
    pub(super) column: super::GridPlacement,
    pub(super) row: super::GridPlacement,
    pub(super) column_offsets: &'a [S],
    pub(super) row_offsets: &'a [S],
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) lines: GridLines,
}

pub(super) struct OrdinaryAbsoluteGridGeometryContextInput<'a, S: LayoutScalar> {
    pub(super) container_style: &'a NodeInputOf<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) containing_size: Size<S>,
    pub(super) column: super::GridPlacement,
    pub(super) row: super::GridPlacement,
    pub(super) column_offsets: &'a [S],
    pub(super) row_offsets: &'a [S],
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) lines: GridLines,
}

impl<'a, S: LayoutScalar> AbsoluteGridContext<'a, S> {
    pub(super) fn ordinary(input: OrdinaryAbsoluteGridContextInput<'a, S>) -> Self {
        let OrdinaryAbsoluteGridContextInput {
            container_style,
            constants,
            containing_size,
            column,
            row,
            column_offsets,
            row_offsets,
            columns,
            rows,
            gap,
            lines,
        } = input;
        Self(OrdinaryAbsoluteGridContext {
            container_style,
            constants,
            containing_size,
            column,
            row,
            column_offsets,
            row_offsets,
            columns,
            rows,
            gap,
            column_geometry: None,
            row_geometry: None,
            lines,
            containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState::INITIAL,
        })
    }

    pub(super) fn ordinary_with_geometry(
        input: OrdinaryAbsoluteGridGeometryContextInput<'a, S>,
    ) -> Self {
        let OrdinaryAbsoluteGridGeometryContextInput {
            container_style,
            constants,
            containing_size,
            column,
            row,
            column_offsets,
            row_offsets,
            columns,
            rows,
            gap,
            column_geometry,
            row_geometry,
            lines,
        } = input;
        Self(OrdinaryAbsoluteGridContext {
            container_style,
            constants,
            containing_size,
            column,
            row,
            column_offsets,
            row_offsets,
            columns,
            rows,
            gap,
            column_geometry: Some(column_geometry),
            row_geometry: Some(row_geometry),
            lines,
            containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState::INITIAL,
        })
    }

    pub(super) fn with_containing_auto_scrollbar_pass(
        mut self,
        containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    ) -> Self {
        self.0.containing_auto_scrollbar_pass = containing_auto_scrollbar_pass;
        self
    }
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridAreaInput<'a, S: LayoutScalar> {
    pub(super) column: super::GridPlacement,
    pub(super) row: super::GridPlacement,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) column_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(super) row_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(super) column_offsets: &'a [S],
    pub(super) row_offsets: &'a [S],
    pub(super) constants: &'a Constants<S>,
    pub(super) lines: GridLines,
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridAxisInput<'a, S: LayoutScalar = Scalar> {
    pub(super) placement: super::GridPlacement,
    pub(super) tracks: &'a [S],
    pub(super) offsets: &'a [S],
    pub(super) geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) padding_box_location: S,
    pub(super) padding_box_size: S,
    pub(super) is_reverse: bool,
    pub(super) explicit_start: usize,
    pub(super) explicit_count: usize,
}

pub(super) fn retained_grid_child_scroll_geometry<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<crate::ScrollGeometryOf<S>>,
) -> Result<crate::ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    if let Some(geometry) = child_compute_geometry {
        if geometry.border_box().origin() == Point::ZERO && geometry.border_box().size() == size {
            return Ok(geometry);
        }
        return rebuild_canonical_scroll_geometry_for_border_box(geometry, size, border, padding);
    }

    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    canonical_measured_leaf_scroll_geometry(MeasuredLeafScrollGeometrySourceOf {
        flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: size,
        border,
        padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
        measured_content_size: content_size,
        scroll_snap_type: style.scroll_snap_type,
        target_scroll_margin: style.scroll_margin,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
}

pub(super) fn layout_absolute_grid_child<Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    source_index: usize,
    child_style: &NodeInputOf<Tree::Scalar>,
    context: AbsoluteGridContext<'_, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridChildContribution<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let context = context.0;
    let container_style = context.container_style;
    let constants = context.constants;
    let containing_size = context.containing_size;
    let area = absolute_grid_area(AbsoluteGridAreaInput {
        column: context.column,
        row: context.row,
        columns: context.columns,
        rows: context.rows,
        gap: context.gap,
        column_geometry: context.column_geometry,
        row_geometry: context.row_geometry,
        column_offsets: context.column_offsets,
        row_offsets: context.row_offsets,
        constants,
        lines: context.lines,
    });
    let containing_flow_axes = constants.flow_axes;
    let physical_area_size = containing_flow_axes.physical_size(area.size);
    let area_parent = physical_area_size.map(Some);
    let unresolved_margin = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.margin, area_parent, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
    let available_size = Size::new(
        (physical_area_size.width - non_auto_margin.horizontal_sum()).max(Tree::Scalar::ZERO),
        (physical_area_size.height - non_auto_margin.vertical_sum()).max(Tree::Scalar::ZERO),
    );
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.padding, area_parent, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.border, area_parent, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let box_sizing_adjustment = if child_style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::ZERO
    };
    let style_size = Size::new(
        resolve_preferred_optional(
            &child_style.size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_preferred_optional(
            &child_style.size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .apply_aspect_ratio(child_style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let padding_border_size = (padding + border).sum_axes();
    let min_size = Size::new(
        resolve_minimum_optional(
            &child_style.min_size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_minimum_optional(
            &child_style.min_size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .add_optional(box_sizing_adjustment)
    .or(padding_border_size.map(Some))
    .max_optional(padding_border_size.map(Some))
    .apply_aspect_ratio(child_style.aspect_ratio);
    let max_size = Size::new(
        resolve_maximum_optional(
            &child_style.max_size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_maximum_optional(
            &child_style.max_size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .apply_aspect_ratio(child_style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let inset = child_style
        .inset
        .zip_size(area_parent, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let mut known = Size::new(
        style_size.width.or_else(|| {
            inset.left.zip(inset.right).map(|(left, right)| {
                (physical_area_size.width - non_auto_margin.horizontal_sum() - left - right)
                    .max(Tree::Scalar::ZERO)
            })
        }),
        style_size.height.or_else(|| {
            inset.top.zip(inset.bottom).map(|(top, bottom)| {
                (physical_area_size.height - non_auto_margin.vertical_sum() - top - bottom)
                    .max(Tree::Scalar::ZERO)
            })
        }),
    );
    if let (Some(ratio), Some(width)) = (child_style.aspect_ratio, known.width)
        && child_style.size.height.is_auto()
    {
        known.height = Some(width / ratio.get());
    } else if let (Some(ratio), Some(height)) = (child_style.aspect_ratio, known.height)
        && child_style.size.width.is_auto()
    {
        known.width = Some(height * ratio.get());
    }
    let known = known
        .apply_aspect_ratio(child_style.aspect_ratio)
        .clamp_optional(min_size, max_size);
    let output = tree.compute_child(
        child,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            known,
            area_parent,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    container_style.writing_mode,
                    container_style.direction,
                ),
                crate::ParentFormattingContext::Grid,
            ),
            Size::new(
                AvailableOf::definite(available_size.width),
                AvailableOf::definite(available_size.height),
            ),
        )
        .with_containing_auto_scrollbar_pass(context.containing_auto_scrollbar_pass),
    )?;
    let final_size = known
        .unwrap_or(output.size)
        .clamp_optional(min_size, max_size);
    let justify = child_style
        .justify_self
        .unwrap_or(container_style.justify_items.unwrap_or(AlignItems::Start));
    let align = child_style
        .align_self
        .unwrap_or(container_style.align_items.unwrap_or(AlignItems::Start));
    let (location, margin) = {
        let logical_size = containing_flow_axes.logical_size(final_size);
        let logical_margin = containing_flow_axes.logical_edges(unresolved_margin);
        let logical_inset = containing_flow_axes.logical_edges(inset);
        let inline_axis = absolute_grid_axis(AbsoluteGridAxis {
            area_location: area.location.inline,
            static_area_location: area.static_location.inline,
            area_size: area.size.inline,
            static_area_size: area.static_size.inline,
            size: logical_size.inline,
            margin_start: logical_margin.inline_start,
            margin_end: logical_margin.inline_end,
            inset_start: logical_inset.inline_start,
            inset_end: logical_inset.inline_end,
            alignment: justify,
            progression: PhysicalProgression::Increasing,
        });
        let block_axis = absolute_grid_axis(AbsoluteGridAxis {
            area_location: area.location.block,
            static_area_location: area.static_location.block,
            area_size: area.size.block,
            static_area_size: area.static_size.block,
            size: logical_size.block,
            margin_start: logical_margin.block_start,
            margin_end: logical_margin.block_end,
            inset_start: logical_inset.block_start,
            inset_end: logical_inset.block_end,
            alignment: align,
            progression: PhysicalProgression::Increasing,
        });
        (
            containing_flow_axes.physical_point(
                LogicalPointOf::new(inline_axis.location, block_axis.location),
                logical_size,
                containing_size,
            ),
            containing_flow_axes.physical_edges(LogicalEdgesOf::new(
                inline_axis.margin_start,
                inline_axis.margin_end,
                block_axis.margin_start,
                block_axis.margin_end,
            )),
        )
    };

    let scroll_geometry = retained_grid_child_scroll_geometry(
        child_style,
        final_size,
        output.content_size,
        padding,
        border,
        output.scroll_geometry,
    )
    .map_err(|error| layout_child_geometry_error(child, child, error))?;
    tree.set_unrounded(
        child,
        NodeOutputOf {
            source_index: crate::SourceIndex::new(source_index),
            location,
            size: final_size,
            content_size: output.content_size,
            scroll_geometry: Some(scroll_geometry),
            border,
            padding,
            margin,
        },
    );

    Ok(GridChildContribution {
        source_index: crate::SourceIndex::new(source_index),
        location,
        margin,
        geometry: scroll_geometry,
        descendants: scroll_geometry.propagatable_descendant_intervals(),
        overflow: UsedOverflow::from_computed(child_style.overflow, child_style.item_is_replaced),
        in_flow: false,
    })
}

#[derive(Clone, Copy)]
pub(super) struct LogicalAbsoluteGridArea<S: LayoutScalar = Scalar> {
    pub(super) location: LogicalPointOf<S>,
    pub(super) static_location: LogicalPointOf<S>,
    pub(super) size: LogicalSizeOf<S>,
    pub(super) static_size: LogicalSizeOf<S>,
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridAxisArea<S: LayoutScalar = Scalar> {
    pub(super) location: S,
    pub(super) size: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AbsoluteGridAxis<S: LayoutScalar = Scalar> {
    pub(super) area_location: S,
    pub(super) static_area_location: S,
    pub(super) area_size: S,
    pub(super) static_area_size: S,
    pub(super) size: S,
    pub(super) margin_start: Option<S>,
    pub(super) margin_end: Option<S>,
    pub(super) inset_start: Option<S>,
    pub(super) inset_end: Option<S>,
    pub(super) alignment: AlignItems,
    pub(super) progression: PhysicalProgression,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ResolvedAbsoluteGridAxis<S: LayoutScalar = Scalar> {
    pub(super) location: S,
    pub(super) margin_start: S,
    pub(super) margin_end: S,
}

pub(super) fn absolute_grid_axis<S: LayoutScalar>(
    axis: AbsoluteGridAxis<S>,
) -> ResolvedAbsoluteGridAxis<S> {
    let AbsoluteGridAxis {
        area_location,
        static_area_location,
        area_size,
        static_area_size,
        size,
        margin_start,
        margin_end,
        inset_start,
        inset_end,
        alignment,
        progression,
    } = axis;
    let non_auto_start = margin_start.unwrap_or(S::ZERO);
    let non_auto_end = margin_end.unwrap_or(S::ZERO);
    let raw_free_space = area_size - size - non_auto_start - non_auto_end;
    let free_space = raw_free_space.max(S::ZERO);
    let auto_margin_count = usize::from(margin_start.is_none()) + usize::from(margin_end.is_none());
    let auto_margin = if auto_margin_count > 0 {
        free_space / S::from_usize(auto_margin_count)
    } else {
        S::ZERO
    };
    let resolved_start = margin_start.unwrap_or(auto_margin);
    let resolved_end = margin_end.unwrap_or(auto_margin);
    let uses_static_area = inset_start.is_none() && inset_end.is_none();
    let offset = match (inset_start, inset_end) {
        (Some(_), Some(end)) if progression.is_decreasing() => {
            area_size - end - size - non_auto_end
        }
        (Some(start), _) => start + non_auto_start,
        (None, Some(end)) => area_size - end - size - non_auto_end,
        (None, None) => match alignment.safe_fallback(raw_free_space) {
            AlignItems::Start
            | AlignItems::FlexStart
            | AlignItems::Baseline
            | AlignItems::Stretch
                if progression.is_decreasing() =>
            {
                static_area_size - size - resolved_end
            }
            AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline
                if progression.is_decreasing() =>
            {
                resolved_start
            }
            AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline => {
                static_area_size - size - resolved_end
            }
            AlignItems::Center => {
                (static_area_size - size + resolved_start - resolved_end) / S::from_f64(2.0)
            }
            AlignItems::Start
            | AlignItems::FlexStart
            | AlignItems::Baseline
            | AlignItems::Stretch => resolved_start,
            AlignItems::SafeEnd | AlignItems::SafeFlexEnd | AlignItems::SafeCenter => {
                unreachable!("safe_fallback returns unsafe item alignment")
            }
        },
    };
    let base_location = if uses_static_area {
        static_area_location
    } else {
        area_location
    };
    ResolvedAbsoluteGridAxis {
        location: base_location + offset,
        margin_start: resolved_start,
        margin_end: resolved_end,
    }
}

pub(super) fn logical_relative_inset_offset<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    flow_axes: FlowAxes,
    position: Position,
) -> LogicalPointOf<S> {
    if position != Position::Relative {
        return LogicalPointOf::new(S::ZERO, S::ZERO);
    }

    let inset = flow_axes.logical_edges(inset);
    LogicalPointOf::new(
        inset
            .inline_start
            .or_else(|| inset.inline_end.map(|end| -end))
            .unwrap_or(S::ZERO),
        inset
            .block_start
            .or_else(|| inset.block_end.map(|end| -end))
            .unwrap_or(S::ZERO),
    )
}

pub(super) fn grid_scroll_contributions<S: LayoutScalar>(
    mut children: Vec<GridChildContribution<S>>,
    flow_axes: FlowAxes,
    padding: Edges<S>,
) -> Result<ScrollContributionAccumulatorOf<S>, crate::scroll::ScrollContributionErrorOf<S>> {
    children.sort_by_key(|child| child.source_index);
    let mut contributions = empty_grid_contributions();
    let mut inline_end = None;
    let mut block_end = None;

    for child in children {
        if child.in_flow {
            contributions.include_in_flow_child(
                child.location,
                child.geometry.border_box(),
                child.margin,
                child.descendants,
                child.overflow,
            )?;
            let border_size = child.geometry.border_box().size();
            if border_size.width > S::ZERO && border_size.height > S::ZERO {
                include_farthest_grid_flow_end(
                    &mut inline_end,
                    flow_axes.inline_end(),
                    grid_child_flow_end(child, flow_axes.inline_end()),
                );
                include_farthest_grid_flow_end(
                    &mut block_end,
                    flow_axes.block_end(),
                    grid_child_flow_end(child, flow_axes.block_end()),
                );
            }
        } else {
            contributions.include_current_out_of_flow(
                child.location,
                child.geometry.border_box(),
                child.margin,
                child.descendants,
                child.overflow,
            )?;
        }
    }

    for (axis, coordinate) in [
        (LogicalAxis::Inline, inline_end),
        (LogicalAxis::Block, block_end),
    ] {
        if let Some(coordinate) = coordinate {
            contributions.record_final_in_flow_end(flow_axes, axis, coordinate)?;
        }
    }
    contributions.include_terminal_padding(padding)?;
    Ok(contributions)
}

fn grid_child_flow_end<S: LayoutScalar>(
    child: GridChildContribution<S>,
    side: crate::PhysicalSide,
) -> S {
    let border_box = child.geometry.border_box();
    let origin = border_box.origin();
    let size = border_box.size();
    match side {
        crate::PhysicalSide::Top => child.location.y + origin.y - child.margin.top.max(S::ZERO),
        crate::PhysicalSide::Right => {
            child.location.x + origin.x + size.width + child.margin.right.max(S::ZERO)
        }
        crate::PhysicalSide::Bottom => {
            child.location.y + origin.y + size.height + child.margin.bottom.max(S::ZERO)
        }
        crate::PhysicalSide::Left => child.location.x + origin.x - child.margin.left.max(S::ZERO),
    }
}

fn include_farthest_grid_flow_end<S: LayoutScalar>(
    end: &mut Option<S>,
    side: crate::PhysicalSide,
    candidate: S,
) {
    *end = Some(end.map_or(candidate, |current| match side {
        crate::PhysicalSide::Top | crate::PhysicalSide::Left => current.min(candidate),
        crate::PhysicalSide::Right | crate::PhysicalSide::Bottom => current.max(candidate),
    }));
}

pub(super) fn max_size<S: LayoutScalar>(a: Size<S>, b: Size<S>) -> Size<S> {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}
