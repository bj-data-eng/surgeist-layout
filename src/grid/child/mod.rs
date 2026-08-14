use super::*;
use crate::error::{layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::{
    FlowAxes, LogicalAxis, LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis,
    PhysicalProgression,
};
use crate::output::PhysicalBaseline;
use crate::scroll::{ScrollContributionAccumulatorOf, UsedOverflow};
use crate::sizing::resolve::{
    SizingResolutionError, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};
use crate::{
    BaselinesOf, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant,
    LayoutOperation,
};

mod absolute;
mod baseline;
mod scroll;
mod subgrid_context;

pub(super) use absolute::*;
pub(super) use baseline::*;
pub(super) use scroll::*;
pub(super) use subgrid_context::*;

pub(super) struct GridChildrenLayout<S: LayoutScalar = Scalar> {
    pub(super) visible_content_size: Size<S>,
    pub(super) contributions: ScrollContributionAccumulatorOf<S>,
    pub(super) baselines: BaselinesOf<S>,
    pub(super) baseline_groups: GridBaselineGroups<S>,
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
        let child_style = placements.item_input(source_index).clone();
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
            style: child_style.clone(),
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
            placements,
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
        let child_style = &item.style;
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
    pub(super) style: &'a GridContainerProjection<'a, S>,
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
    pub(super) style: GridItemProjection<S>,
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
    child_style: &GridItemProjection<Tree::Scalar>,
    container_style: &GridContainerProjection<'_, Tree::Scalar>,
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
    child_style: &GridItemProjection<S>,
    container_style: &GridContainerProjection<'_, S>,
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
    child_style: &GridItemProjection<S>,
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
