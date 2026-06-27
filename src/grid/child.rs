use super::*;
use crate::Baselines;

pub(super) struct GridChildrenLayout {
    pub(super) visible_content_size: Size,
    pub(super) first_baseline: Option<Scalar>,
    pub(super) last_baseline: Option<Scalar>,
    pub(super) baseline_groups: GridBaselineGroups,
}

#[derive(Clone, Copy)]
pub(super) struct GridLines {
    pub(super) column_explicit_start: usize,
    pub(super) column_explicit_count: usize,
    pub(super) row_explicit_start: usize,
    pub(super) row_explicit_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BaselineGroupKind {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct BaselineParticipation {
    pub(super) participates: bool,
    pub(super) group: Option<BaselineGroupKind>,
    pub(super) synthesized: bool,
    pub(super) fallback_alignment: Option<AlignItems>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BaselineGeometry {
    pub(super) available_span_size: Scalar,
    pub(super) margin_box_size: Scalar,
    // Border-box baselines are stored separately on PendingGridItem. These
    // fields are the margin-box contributions used by shared baseline groups:
    // block-start margin plus first baseline for major groups, and block-end
    // margin plus distance from last baseline to block-end for minor groups.
    pub(super) major_baseline: Scalar,
    pub(super) minor_baseline: Scalar,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TrackBaselineGroup {
    pub(super) first: Option<Scalar>,
    pub(super) last: Option<Scalar>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct GridBaselineGroups {
    pub(super) rows: Vec<TrackBaselineGroup>,
    pub(super) columns: Vec<TrackBaselineGroup>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct GridContainerBaselines {
    pub(super) first: Option<Scalar>,
    pub(super) last: Option<Scalar>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PublishedTrackBaselineGroup {
    pub(super) parent_index: usize,
    pub(super) group: TrackBaselineGroup,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct BaselineShim {
    pub(super) before: Scalar,
    pub(super) after: Scalar,
}

impl GridBaselineGroups {
    fn shared_baseline(&self, group_kind: BaselineGroupKind, area: GridArea) -> Option<Scalar> {
        match group_kind {
            BaselineGroupKind::Major => self.rows.get(area.row)?.first,
            BaselineGroupKind::Minor => {
                let row = area.row_end.checked_sub(1)?;
                self.rows.get(row)?.last
            }
        }
    }
}

// Surgeist currently lays out horizontal writing mode only. Column-axis
// baseline groups use the same data model but are not applied until vertical
// writing-mode grid tests are introduced.
pub(super) fn baseline_shim_for_intrinsic_contribution(
    participation: BaselineParticipation,
    geometry: BaselineGeometry,
    shared: TrackBaselineGroup,
) -> BaselineShim {
    if !participation.participates {
        return BaselineShim::default();
    }

    match participation.group {
        Some(BaselineGroupKind::Major) => BaselineShim {
            before: shared.first.map_or(0.0, |baseline| {
                (baseline - geometry.major_baseline).max(0.0)
            }),
            after: 0.0,
        },
        Some(BaselineGroupKind::Minor) => BaselineShim {
            before: 0.0,
            after: shared.last.map_or(0.0, |baseline| {
                (baseline - geometry.minor_baseline).max(0.0)
            }),
        },
        None => BaselineShim::default(),
    }
}

pub(super) fn baseline_offset(
    group_kind: BaselineGroupKind,
    shared_baseline: Scalar,
    geometry: BaselineGeometry,
) -> Scalar {
    match group_kind {
        BaselineGroupKind::Major => shared_baseline - geometry.major_baseline,
        BaselineGroupKind::Minor => {
            let baseline_delta = shared_baseline - geometry.minor_baseline;
            geometry.available_span_size - baseline_delta - geometry.margin_box_size
        }
    }
}

pub(super) fn spanned_track_size(
    tracks: &[Scalar],
    start: usize,
    end: usize,
    gap: Scalar,
) -> Scalar {
    let track_sum = tracks[start..end].iter().copied().sum::<Scalar>();
    let gap_sum = gap * end.saturating_sub(start + 1) as Scalar;
    track_sum + gap_sum
}

pub(super) fn baseline_aligned_block_offset<Node: Copy>(
    item: &PendingGridItem<Node>,
    groups: &GridBaselineGroups,
    rows: &[Scalar],
    row_gap: Scalar,
) -> Option<Scalar> {
    if !item.baseline_participation.participates || item.block_auto_margins {
        return None;
    }

    let group_kind = item.baseline_participation.group?;
    let shared = groups.shared_baseline(group_kind, item.area)?;
    let margin_box_offset =
        baseline_offset(group_kind, shared, item.baseline_geometry(rows, row_gap));
    Some(margin_box_offset + item.vertical_axis.margin_start)
}

pub(super) fn layout_grid_children<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    context: GridLayoutContext<'_, <Tree as Traverse>::Node>,
) -> GridChildrenLayout
where
    Tree: Compute,
{
    let GridLayoutContext {
        style,
        constants,
        container_content_size,
        columns,
        rows,
        row_tracks,
        gap,
        lines,
        named_columns,
        named_rows,
        area_facts,
        inherited_column_offset,
        inherited_row_offset,
        subgrid_report,
        parent_context,
        placements,
    } = context;

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

    let logical_content_size =
        Size::new(track_sum(columns, gap.width), track_sum(rows, gap.height));
    let physical_content_size = grid_area_physical_size(style.writing_mode, logical_content_size);
    let content_box_size =
        constants
            .node_inner_size
            .unwrap_or(if style.writing_mode.is_vertical() {
                physical_content_size
            } else {
                container_content_size
            });
    let axis_content_box_size = grid_area_logical_size(style.writing_mode, content_box_size);
    let alignment_free_space = axis_content_box_size - logical_content_size;
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
    let content_box_left = effective_content_box_left(constants, container_content_size);
    let inherited_rtl_column_line_adjustment =
        if inherited_column_offset.is_some() && style.direction.is_rtl() {
            constants.content_box_inset.right - constants.content_box_inset.left
        } else {
            0.0
        };
    let column_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style,
        axis: GridAxisKind::Column,
        tracks: columns,
        inherited_offset: inherited_column_offset,
        content_box_left,
        content_box_size,
        content_box_inset: constants.content_box_inset,
        alignment: column_alignment,
    });
    let row_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style,
        axis: GridAxisKind::Row,
        tracks: rows,
        inherited_offset: inherited_row_offset,
        content_box_left,
        content_box_size,
        content_box_inset: constants.content_box_inset,
        alignment: row_alignment,
    });
    let children = tree.children(node).collect::<Vec<_>>();
    let placed_areas = resolve_grid_child_areas(ResolveGridChildAreasInput {
        children: &children,
        placements,
        style,
        columns,
        rows,
        gap,
        lines,
    });
    let empty_baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); rows.len()],
        columns: vec![TrackBaselineGroup::default(); columns.len()],
    };
    let mut visible_content_size = Size::ZERO;
    let mut pending_items = Vec::new();
    for (order, (((child, placement), area), subgrid_item)) in placements
        .checked_child_placements(&children)
        .zip(placed_areas)
        .zip(subgrid_report.items.iter())
        .enumerate()
    {
        let child_style = tree.node_input(child).clone();
        if child_style.display == super::Display::None {
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
                        lines,
                        column: placement.absolute_column,
                        row: placement.absolute_row,
                        column_line_offset_adjustment: inherited_rtl_column_line_adjustment,
                    },
                ),
            );
            continue;
        }

        let Some(area) = area else {
            continue;
        };
        if area.row >= rows.len() || area.column >= columns.len() {
            tree.set_unrounded(child, NodeOutput::with_order(order as u32));
            tree.compute_child(child, ComputeInput::HIDDEN);
            continue;
        }

        let physical_area_size = grid_area_physical_size(style.writing_mode, area.size);
        let mut item = grid_item_sizing(
            &child_style,
            style,
            physical_area_size,
            Size::splat(Some(physical_area_size.width)),
            tree.calc_resolver(),
        );
        stretch_subgridded_axes(&mut item, *subgrid_item);
        let area_width_basis = Size::splat(Some(physical_area_size.width));
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
        let subgrid_content_box_size = (physical_area_size
            - resolved_margin.sum_axes()
            - padding.sum_axes()
            - border.sum_axes())
        .max(Size::ZERO);
        let child_context = subgrid_child_parent_context(SubgridChildParentContextInput {
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
            resolver: tree.calc_resolver(),
        });
        let child_input = ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: item.known,
            parent: Size::new(
                Some(physical_area_size.width),
                Some(physical_area_size.height),
            ),
            available: item
                .available
                .map(|value| Available::Definite(value.max(0.0))),
        };
        let output = if child_context.has_inherited_axis() {
            // Subgrid layout depends on the parent grid's used tracks, so this
            // intentionally bypasses the generic child layout cache until that
            // cache can include context-sensitive grid keys.
            compute_grid_with_context(tree, child, child_input, child_context)
        } else {
            tree.compute_child(child, child_input)
        };
        let scrollbar_size = Size::new(
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
        );
        let alignment =
            grid_item_physical_alignment(style.writing_mode, item.justify_self, item.align_self);
        let horizontal_axis = grid_item_axis(GridItemAxis {
            area_size: physical_area_size.width,
            size: output.size.width,
            margin_start: item.unresolved_margin.left,
            margin_end: item.unresolved_margin.right,
            alignment: alignment.horizontal,
            direction: grid_physical_axis_direction(
                style.writing_mode,
                style.direction,
                PhysicalGridAxis::Horizontal,
            ),
        });
        let vertical_axis = grid_item_axis(GridItemAxis {
            area_size: physical_area_size.height,
            size: output.size.height,
            margin_start: item.unresolved_margin.top,
            margin_end: item.unresolved_margin.bottom,
            alignment: alignment.vertical,
            direction: grid_physical_axis_direction(
                style.writing_mode,
                style.direction,
                PhysicalGridAxis::Vertical,
            ),
        });
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
        let row_span_tracks = row_tracks.get(area.row..area.row_end).unwrap_or(&[]);
        let baseline_participation = baseline_participation(
            item.align_self,
            block_auto_margins,
            synthesized_baseline_would_cycle(item.align_self, baselines, row_span_tracks),
            baselines,
        );
        pending_items.push(PendingGridItem {
            node: child,
            order: order as u32,
            area,
            output,
            horizontal_axis,
            vertical_axis,
            relative_offset: relative_inset_offset(
                child_style.inset.zip_size(
                    Size::new(
                        Some(physical_area_size.width),
                        Some(physical_area_size.height),
                    ),
                    |length, basis| resolve_auto_optional_with(length, basis, tree.calc_resolver()),
                ),
                style.direction,
                child_style.position,
            ),
            first_baseline,
            last_baseline,
            published_row_baselines: None,
            block_offset: vertical_axis.offset,
            block_auto_margins,
            baseline_participation,
            margin,
            scrollbar_size,
            border,
            padding,
            overflow: child_style.overflow,
        });
    }

    let mut published_group_set = baseline_groups(&pending_items, rows.len(), columns.len());
    let mut baseline_group_set = published_group_set.clone();
    merge_inherited_baseline_groups(&mut baseline_group_set, parent_context);
    for _ in 0..=pending_items.len() {
        refresh_subgrid_items_with_baselines(
            tree,
            SubgridBaselineRefreshInput {
                container_style: style,
                columns,
                rows,
                row_tracks,
                gap,
                named_columns: named_columns.clone(),
                named_rows: named_rows.clone(),
                area_facts: area_facts.clone(),
                subgrid_report,
                baseline_groups: &baseline_group_set,
            },
            &mut pending_items,
        );
        let next_published_group_set = baseline_groups(&pending_items, rows.len(), columns.len());
        if next_published_group_set == published_group_set {
            break;
        }
        published_group_set = next_published_group_set;
        baseline_group_set = published_group_set.clone();
        merge_inherited_baseline_groups(&mut baseline_group_set, parent_context);
    }
    for item in &mut pending_items {
        let area_origin =
            grid_area_physical_origin(style, &column_offsets, &row_offsets, item.area);
        let block_axis_offset =
            baseline_aligned_block_offset(item, &baseline_group_set, rows, gap.height)
                .unwrap_or_else(|| grid_item_block_axis_offset(style.writing_mode, item));
        item.block_offset = block_axis_offset;
        let axis_offset = grid_item_physical_offset(style.writing_mode, item, block_axis_offset);
        let location = Point::new(
            area_origin.x + axis_offset.x + item.relative_offset.x,
            area_origin.y + axis_offset.y + item.relative_offset.y,
        );
        visible_content_size = max_size(
            visible_content_size,
            content_size_contribution(
                Point::new(location.x - area_origin.x, location.y - area_origin.y),
                item.output.size,
                item.output.content_size,
                item.overflow,
            ),
        );

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
    let baselines =
        grid_container_baselines(&pending_items, &baseline_group_set, &row_offsets, rows);

    GridChildrenLayout {
        visible_content_size,
        first_baseline: baselines.first,
        last_baseline: baselines.last,
        baseline_groups: published_group_set,
    }
}

struct SubgridBaselineRefreshInput<'a, Node> {
    container_style: &'a NodeInput,
    columns: &'a [Scalar],
    rows: &'a [Scalar],
    row_tracks: &'a [TrackSizing],
    gap: Size,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    subgrid_report: &'a GridSubgridReport<Node>,
    baseline_groups: &'a GridBaselineGroups,
}

fn refresh_subgrid_items_with_baselines<Tree>(
    tree: &mut Tree,
    input: SubgridBaselineRefreshInput<'_, <Tree as Traverse>::Node>,
    pending_items: &mut [PendingGridItem<<Tree as Traverse>::Node>],
) where
    Tree: Compute,
{
    for item in pending_items.iter_mut() {
        let Some(subgrid_item) = input.subgrid_report.items.get(item.order as usize).copied()
        else {
            continue;
        };
        if !subgrid_item.column.can_inherit() && !subgrid_item.row.can_inherit() {
            continue;
        }

        let child_style = tree.node_input(item.node).clone();
        let physical_area_size =
            grid_area_physical_size(input.container_style.writing_mode, item.area.size);
        let mut sizing = grid_item_sizing(
            &child_style,
            input.container_style,
            physical_area_size,
            Size::splat(Some(physical_area_size.width)),
            tree.calc_resolver(),
        );
        stretch_subgridded_axes(&mut sizing, subgrid_item);
        let area_width_basis = Size::splat(Some(physical_area_size.width));
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
        let resolved_margin = sizing.unresolved_margin.map(|margin| margin.unwrap_or(0.0));
        let subgrid_content_box_size = (physical_area_size
            - resolved_margin.sum_axes()
            - padding.sum_axes()
            - border.sum_axes())
        .max(Size::ZERO);
        let child_context = subgrid_child_parent_context(SubgridChildParentContextInput {
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
            parent_baseline_groups: input.baseline_groups,
            margin: sizing.unresolved_margin,
            border,
            padding,
            resolver: tree.calc_resolver(),
        });
        if !child_context.has_inherited_axis() {
            continue;
        }

        let child_input = ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: sizing.known,
            parent: Size::new(
                Some(physical_area_size.width),
                Some(physical_area_size.height),
            ),
            available: sizing
                .available
                .map(|value| Available::Definite(value.max(0.0))),
        };
        let row_axis = child_context.rows.clone();
        let result = compute_grid_with_context_result(tree, item.node, child_input, child_context);
        let output = result.output;
        let alignment = grid_item_physical_alignment(
            input.container_style.writing_mode,
            sizing.justify_self,
            sizing.align_self,
        );
        let horizontal_axis = grid_item_axis(GridItemAxis {
            area_size: physical_area_size.width,
            size: output.size.width,
            margin_start: sizing.unresolved_margin.left,
            margin_end: sizing.unresolved_margin.right,
            alignment: alignment.horizontal,
            direction: grid_physical_axis_direction(
                input.container_style.writing_mode,
                input.container_style.direction,
                PhysicalGridAxis::Horizontal,
            ),
        });
        let vertical_axis = grid_item_axis(GridItemAxis {
            area_size: physical_area_size.height,
            size: output.size.height,
            margin_start: sizing.unresolved_margin.top,
            margin_end: sizing.unresolved_margin.bottom,
            alignment: alignment.vertical,
            direction: grid_physical_axis_direction(
                input.container_style.writing_mode,
                input.container_style.direction,
                PhysicalGridAxis::Vertical,
            ),
        });
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
            sizing.unresolved_margin.top.is_none() || sizing.unresolved_margin.bottom.is_none();
        let row_span_tracks = input
            .row_tracks
            .get(item.area.row..item.area.row_end)
            .unwrap_or(&[]);
        let baseline_participation = baseline_participation(
            sizing.align_self,
            block_auto_margins,
            synthesized_baseline_would_cycle(sizing.align_self, baselines, row_span_tracks),
            baselines,
        );

        item.output = output;
        item.horizontal_axis = horizontal_axis;
        item.vertical_axis = vertical_axis;
        item.first_baseline = first_baseline;
        item.last_baseline = last_baseline;
        item.published_row_baselines = row_axis
            .as_ref()
            .map(|axis| publish_row_baseline_groups(&result.baseline_groups.rows, axis));
        item.block_auto_margins = block_auto_margins;
        item.baseline_participation = baseline_participation;
        item.margin = margin;
        item.border = border;
        item.padding = padding;
    }
}

pub(super) fn grid_area_inline_offset(offsets: &[Scalar], area: GridArea) -> Scalar {
    grid_area_track_offset(offsets, area.column, area.column_end)
}

fn grid_area_track_offset(offsets: &[Scalar], start: usize, end: usize) -> Scalar {
    offsets
        .get(start..end)
        .and_then(|offsets| offsets.iter().copied().reduce(Scalar::min))
        .unwrap_or(0.0)
}

pub(super) fn grid_area_physical_origin(
    style: &NodeInput,
    column_offsets: &[Scalar],
    row_offsets: &[Scalar],
    area: GridArea,
) -> Point {
    if style.writing_mode.is_vertical() {
        Point::new(
            grid_area_track_offset(row_offsets, area.row, area.row_end),
            grid_area_track_offset(column_offsets, area.column, area.column_end),
        )
    } else {
        Point::new(
            grid_area_track_offset(column_offsets, area.column, area.column_end),
            grid_area_track_offset(row_offsets, area.row, area.row_end),
        )
    }
}

fn grid_area_physical_size(writing_mode: crate::WritingMode, size: Size) -> Size {
    if writing_mode.is_vertical() {
        Size::new(size.height, size.width)
    } else {
        size
    }
}

fn grid_area_logical_size(writing_mode: crate::WritingMode, size: Size) -> Size {
    if writing_mode.is_vertical() {
        Size::new(size.height, size.width)
    } else {
        size
    }
}

#[derive(Clone, Copy)]
pub(super) struct GridAxisOffsetsInput<'a> {
    pub(super) style: &'a NodeInput,
    pub(super) axis: GridAxisKind,
    pub(super) tracks: &'a [Scalar],
    pub(super) inherited_offset: Option<Scalar>,
    pub(super) content_box_left: Scalar,
    pub(super) content_box_size: Size,
    pub(super) content_box_inset: Edges,
    pub(super) alignment: GridAlignment,
}

pub(super) fn grid_axis_offsets(input: GridAxisOffsetsInput<'_>) -> Vec<Scalar> {
    if !input.style.writing_mode.is_vertical() {
        return horizontal_grid_axis_offsets(input);
    }

    match input.axis {
        GridAxisKind::Column => {
            let start = input
                .inherited_offset
                .map(|offset| offset + input.content_box_inset.top)
                .unwrap_or(input.content_box_inset.top);
            if input.style.direction.is_rtl() {
                rtl_offsets(
                    input.tracks,
                    start,
                    input.content_box_size.height,
                    input.alignment.start,
                    input.alignment.gap,
                )
            } else {
                offsets(
                    input.tracks,
                    start + input.alignment.start,
                    input.alignment.gap,
                )
            }
        }
        GridAxisKind::Row => {
            let start = input
                .inherited_offset
                .map(|offset| offset + input.content_box_inset.left)
                .unwrap_or(input.content_box_inset.left);
            if input.style.writing_mode == crate::WritingMode::VerticalRl {
                rtl_offsets(
                    input.tracks,
                    start,
                    input.content_box_size.width,
                    input.alignment.start,
                    input.alignment.gap,
                )
            } else {
                offsets(
                    input.tracks,
                    start + input.alignment.start,
                    input.alignment.gap,
                )
            }
        }
    }
}

fn horizontal_grid_axis_offsets(input: GridAxisOffsetsInput<'_>) -> Vec<Scalar> {
    match input.axis {
        GridAxisKind::Column => {
            if let Some(offset) = input.inherited_offset {
                if input.style.direction.is_rtl() {
                    rtl_offsets(
                        input.tracks,
                        offset + input.content_box_inset.left,
                        input.content_box_size.width,
                        input.alignment.start,
                        input.alignment.gap,
                    )
                } else {
                    offsets(
                        input.tracks,
                        offset + input.content_box_inset.left + input.alignment.start,
                        input.alignment.gap,
                    )
                }
            } else if input.style.direction.is_rtl() {
                rtl_offsets(
                    input.tracks,
                    input.content_box_left,
                    input.content_box_size.width,
                    input.alignment.start,
                    input.alignment.gap,
                )
            } else {
                offsets(
                    input.tracks,
                    input.content_box_inset.left + input.alignment.start,
                    input.alignment.gap,
                )
            }
        }
        GridAxisKind::Row => {
            if let Some(offset) = input.inherited_offset {
                offsets(
                    input.tracks,
                    offset + input.content_box_inset.top,
                    input.alignment.gap,
                )
            } else {
                offsets(
                    input.tracks,
                    input.content_box_inset.top + input.alignment.start,
                    input.alignment.gap,
                )
            }
        }
    }
}

#[derive(Clone, Copy)]
enum PhysicalGridAxis {
    Horizontal,
    Vertical,
}

fn grid_physical_axis_direction(
    writing_mode: crate::WritingMode,
    direction: Direction,
    axis: PhysicalGridAxis,
) -> Direction {
    match (writing_mode, axis) {
        (crate::WritingMode::HorizontalTb, PhysicalGridAxis::Horizontal) => direction,
        (crate::WritingMode::VerticalRl, PhysicalGridAxis::Horizontal) => Direction::Rtl,
        (crate::WritingMode::VerticalLr, PhysicalGridAxis::Horizontal) => Direction::Ltr,
        (
            crate::WritingMode::VerticalRl | crate::WritingMode::VerticalLr,
            PhysicalGridAxis::Vertical,
        ) => direction,
        _ => Direction::Ltr,
    }
}

fn grid_item_block_axis_offset<Node>(
    writing_mode: crate::WritingMode,
    item: &PendingGridItem<Node>,
) -> Scalar {
    if writing_mode.is_vertical() {
        item.horizontal_axis.offset
    } else {
        item.vertical_axis.offset
    }
}

fn grid_item_physical_offset<Node>(
    writing_mode: crate::WritingMode,
    item: &PendingGridItem<Node>,
    block_axis_offset: Scalar,
) -> Point {
    if writing_mode.is_vertical() {
        Point::new(block_axis_offset, item.vertical_axis.offset)
    } else {
        Point::new(item.horizontal_axis.offset, block_axis_offset)
    }
}

#[derive(Clone)]
pub(super) struct PendingGridItem<Node> {
    pub(super) node: Node,
    pub(super) order: u32,
    pub(super) area: GridArea,
    pub(super) output: ComputeOutput,
    pub(super) horizontal_axis: ResolvedGridItemAxis,
    pub(super) vertical_axis: ResolvedGridItemAxis,
    pub(super) relative_offset: Point,
    pub(super) first_baseline: Scalar,
    pub(super) last_baseline: Scalar,
    pub(super) published_row_baselines: Option<Vec<PublishedTrackBaselineGroup>>,
    pub(super) block_offset: Scalar,
    pub(super) block_auto_margins: bool,
    pub(super) baseline_participation: BaselineParticipation,
    pub(super) margin: Edges,
    pub(super) scrollbar_size: Size,
    pub(super) border: Edges,
    pub(super) padding: Edges,
    pub(super) overflow: Point<Overflow>,
}

impl<Node> PendingGridItem<Node> {
    fn baseline_geometry(&self, rows: &[Scalar], row_gap: Scalar) -> BaselineGeometry {
        self.baseline_geometry_for_span(spanned_track_size(
            rows,
            self.area.row,
            self.area.row_end,
            row_gap,
        ))
    }

    fn baseline_geometry_for_span(&self, available_span_size: Scalar) -> BaselineGeometry {
        BaselineGeometry {
            available_span_size,
            margin_box_size: self.vertical_axis.margin_start
                + self.output.size.height
                + self.vertical_axis.margin_end,
            major_baseline: self.vertical_axis.margin_start + self.first_baseline,
            minor_baseline: self.vertical_axis.margin_end + self.output.size.height
                - self.last_baseline,
        }
    }
}

pub(super) fn baseline_groups<Node>(
    items: &[PendingGridItem<Node>],
    row_count: usize,
    column_count: usize,
) -> GridBaselineGroups {
    let mut groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); row_count],
        columns: vec![TrackBaselineGroup::default(); column_count],
    };
    for item in items {
        if merge_published_row_baselines(&mut groups.rows, item) {
            continue;
        }
        if !item.baseline_participation.participates || item.block_auto_margins {
            continue;
        }
        match item.baseline_participation.group {
            Some(BaselineGroupKind::Major) => {
                let Some(group) = groups
                    .rows
                    .get_mut(item.area.row)
                    .map(|group| &mut group.first)
                else {
                    continue;
                };
                *group = Some(
                    group.unwrap_or(0.0).max(
                        item.baseline_geometry_for_span(item.area.size.height)
                            .major_baseline,
                    ),
                );
            }
            Some(BaselineGroupKind::Minor) => {
                let Some(row) = item.area.row_end.checked_sub(1) else {
                    continue;
                };
                let Some(group) = groups.rows.get_mut(row).map(|group| &mut group.last) else {
                    continue;
                };
                *group = Some(
                    group.unwrap_or(0.0).max(
                        item.baseline_geometry_for_span(item.area.size.height)
                            .minor_baseline,
                    ),
                );
            }
            None => {}
        }
    }
    groups
}

fn merge_published_row_baselines<Node>(
    rows: &mut [TrackBaselineGroup],
    item: &PendingGridItem<Node>,
) -> bool {
    let Some(published) = &item.published_row_baselines else {
        return false;
    };
    let mut merged = false;
    for published in published {
        let Some(parent_group) = rows.get_mut(published.parent_index) else {
            continue;
        };
        if let Some(first) = published.group.first {
            parent_group.first = Some(
                parent_group
                    .first
                    .map_or(first, |current| current.max(first)),
            );
            merged = true;
        }
        if let Some(last) = published.group.last {
            parent_group.last = Some(parent_group.last.map_or(last, |current| current.max(last)));
            merged = true;
        }
    }
    merged
}

pub(super) fn publish_row_baseline_groups(
    local_groups: &[TrackBaselineGroup],
    axis: &InheritedGridAxis,
) -> Vec<PublishedTrackBaselineGroup> {
    let parent_span_len = axis.parent_end.saturating_sub(axis.parent_start);
    local_groups
        .iter()
        .copied()
        .take(parent_span_len)
        .enumerate()
        .filter_map(|(local_index, group)| {
            let parent_index = if axis.reversed {
                axis.parent_end.checked_sub(local_index + 1)?
            } else {
                axis.parent_start + local_index
            };
            let internal_gap_adjustment =
                axis.gap_difference * internal_gap_edge_count(local_groups.len(), local_index);
            let first = group.first.map(|baseline| {
                baseline
                    + internal_gap_adjustment
                    + if local_index == 0 {
                        axis.start_mbp
                    } else {
                        0.0
                    }
            });
            let last = group.last.map(|baseline| {
                baseline
                    + internal_gap_adjustment
                    + if local_index + 1 == local_groups.len() {
                        axis.end_mbp
                    } else {
                        0.0
                    }
            });
            (first.is_some() || last.is_some()).then_some(PublishedTrackBaselineGroup {
                parent_index,
                group: TrackBaselineGroup { first, last },
            })
        })
        .collect()
}

fn internal_gap_edge_count(track_count: usize, track_index: usize) -> Scalar {
    if track_count < 2 {
        return 0.0;
    }
    let before = usize::from(track_index > 0);
    let after = usize::from(track_index + 1 < track_count);
    (before + after) as Scalar
}

pub(super) fn grid_container_baselines<Node>(
    items: &[PendingGridItem<Node>],
    groups: &GridBaselineGroups,
    row_offsets: &[Scalar],
    rows: &[Scalar],
) -> GridContainerBaselines {
    let mut first_occupied_row = None;
    let mut last_occupied_row = None;
    for (row, group) in groups.rows.iter().enumerate() {
        if group.first.is_some() || group.last.is_some() {
            include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, row);
        }
    }
    for item in items {
        include_occupied_row(
            &mut first_occupied_row,
            &mut last_occupied_row,
            item.area.row,
        );
        if let Some(row) = item.area.row_end.checked_sub(1) {
            include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, row);
        }
    }

    let first = first_occupied_row.and_then(|row| {
        groups
            .rows
            .get(row)
            .and_then(|group| group.first.map(|baseline| row_offsets[row] + baseline))
            .or_else(|| {
                items
                    .iter()
                    .filter(|item| item.area.row == row)
                    .min_by_key(|item| grid_area_start_key(item.area))
                    .map(|item| {
                        row_offsets[item.area.row] + item.block_offset + item.first_baseline
                    })
            })
    });

    let last = last_occupied_row.and_then(|row| {
        groups
            .rows
            .get(row)
            .and_then(|group| {
                group
                    .last
                    .map(|baseline| row_offsets[row] + rows[row] - baseline)
            })
            .or_else(|| {
                items
                    .iter()
                    .filter(|item| item.area.row_end.checked_sub(1) == Some(row))
                    .max_by_key(|item| grid_area_end_key(item.area))
                    .map(|item| row_offsets[item.area.row] + item.block_offset + item.last_baseline)
            })
    });

    GridContainerBaselines { first, last }
}

fn merge_inherited_baseline_groups(
    groups: &mut GridBaselineGroups,
    parent_context: &GridParentContext,
) {
    if let Some(rows) = &parent_context.rows {
        merge_axis_baselines(&mut groups.rows, rows);
    }
    if let Some(columns) = &parent_context.columns {
        merge_axis_baselines(&mut groups.columns, columns);
    }
}

fn merge_axis_baselines(groups: &mut [TrackBaselineGroup], axis: &InheritedGridAxis) {
    for (group, baseline) in groups.iter_mut().zip(&axis.major_baselines) {
        if let Some(baseline) = *baseline {
            group.first = Some(
                group
                    .first
                    .map_or(baseline, |current| current.max(baseline)),
            );
        }
    }
    for (group, baseline) in groups.iter_mut().zip(&axis.minor_baselines) {
        if let Some(baseline) = *baseline {
            group.last = Some(group.last.map_or(baseline, |current| current.max(baseline)));
        }
    }
}

fn include_occupied_row(first: &mut Option<usize>, last: &mut Option<usize>, row: usize) {
    *first = Some(first.map_or(row, |current| current.min(row)));
    *last = Some(last.map_or(row, |current| current.max(row)));
}

fn grid_area_start_key(area: GridArea) -> (usize, usize) {
    (area.row, area.column)
}

fn grid_area_end_key(area: GridArea) -> (usize, usize) {
    (
        area.row_end.saturating_sub(1),
        area.column_end.saturating_sub(1),
    )
}

pub(super) fn baseline_participation(
    align_self: AlignItems,
    block_auto_margins: bool,
    synthesized_baseline_would_cycle: bool,
    baselines: Baselines,
) -> BaselineParticipation {
    let (mut group, synthesized, fallback_alignment) = match align_self {
        AlignItems::Baseline => (
            Some(BaselineGroupKind::Major),
            baselines.first.y.is_none(),
            Some(AlignItems::Start),
        ),
        AlignItems::LastBaseline => (
            Some(BaselineGroupKind::Minor),
            baselines.last.y.is_none(),
            Some(AlignItems::End),
        ),
        _ => (None, false, None),
    };
    if synthesized && synthesized_baseline_would_cycle {
        group = None;
    }

    BaselineParticipation {
        participates: group.is_some() && !block_auto_margins,
        group,
        synthesized,
        fallback_alignment,
    }
}

pub(super) fn synthesized_baseline_would_cycle(
    align_self: AlignItems,
    baselines: Baselines,
    row_span_tracks: &[TrackSizing],
) -> bool {
    let synthesizes = match align_self {
        AlignItems::Baseline => baselines.first.y.is_none(),
        AlignItems::LastBaseline => baselines.last.y.is_none(),
        _ => false,
    };
    synthesizes
        && row_span_tracks.len() > 1
        && row_span_tracks
            .iter()
            .any(|track| track_accepts_intrinsic_contribution(*track))
}

#[derive(Clone, Copy)]
pub(super) struct SubgridChildParentContextInput<'a, Node> {
    pub(super) item: SubgridItemReport<Node>,
    pub(super) child_style: &'a NodeInput,
    pub(super) area: GridArea,
    pub(super) content_box_size: Size,
    pub(super) columns: &'a [Scalar],
    pub(super) rows: &'a [Scalar],
    pub(super) gap: Size,
    pub(super) parent_named_columns: &'a NamedGridLines,
    pub(super) parent_named_rows: &'a NamedGridLines,
    pub(super) parent_area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) parent_baseline_groups: &'a GridBaselineGroups,
    pub(super) margin: Edges<Option<Scalar>>,
    pub(super) border: Edges,
    pub(super) padding: Edges,
    pub(super) resolver: &'a dyn CalcResolver,
}

pub(super) fn subgrid_child_parent_context<Node>(
    input: SubgridChildParentContextInput<'_, Node>,
) -> GridParentContext {
    GridParentContext {
        columns: subgrid_child_axis_context(SubgridChildAxisContextInput {
            axis: GridAxisKind::Column,
            report: input.item.column,
            child_style: input.child_style,
            area: input.area,
            content_box_size: input.content_box_size,
            parent_columns: input.columns,
            parent_rows: input.rows,
            parent_gap: input.gap,
            parent_named_columns: input.parent_named_columns,
            parent_named_rows: input.parent_named_rows,
            parent_area_facts: input.parent_area_facts,
            parent_baseline_groups: input.parent_baseline_groups,
            margin: input.margin,
            border: input.border,
            padding: input.padding,
            resolver: input.resolver,
        }),
        rows: subgrid_child_axis_context(SubgridChildAxisContextInput {
            axis: GridAxisKind::Row,
            report: input.item.row,
            child_style: input.child_style,
            area: input.area,
            content_box_size: input.content_box_size,
            parent_columns: input.columns,
            parent_rows: input.rows,
            parent_gap: input.gap,
            parent_named_columns: input.parent_named_columns,
            parent_named_rows: input.parent_named_rows,
            parent_area_facts: input.parent_area_facts,
            parent_baseline_groups: input.parent_baseline_groups,
            margin: input.margin,
            border: input.border,
            padding: input.padding,
            resolver: input.resolver,
        }),
    }
}

#[derive(Clone, Copy)]
struct SubgridChildAxisContextInput<'a> {
    axis: GridAxisKind,
    report: SubgridAxisReport,
    child_style: &'a NodeInput,
    area: GridArea,
    content_box_size: Size,
    parent_columns: &'a [Scalar],
    parent_rows: &'a [Scalar],
    parent_gap: Size,
    parent_named_columns: &'a NamedGridLines,
    parent_named_rows: &'a NamedGridLines,
    parent_area_facts: Option<&'a GridAreaNameFacts>,
    parent_baseline_groups: &'a GridBaselineGroups,
    margin: Edges<Option<Scalar>>,
    border: Edges,
    padding: Edges,
    resolver: &'a dyn CalcResolver,
}

fn subgrid_child_axis_context(
    input: SubgridChildAxisContextInput<'_>,
) -> Option<InheritedGridAxis> {
    if !input.report.can_inherit() {
        return None;
    }
    let mapping = input.report.mapping.ok()?;
    let (start_line, end_line) = match mapping.parent_axis {
        GridAxisKind::Column => (input.area.column + 1, input.area.column_end + 1),
        GridAxisKind::Row => (input.area.row + 1, input.area.row_end + 1),
    };
    let parent_axis = subgrid_parent_axis_data(&input, mapping.parent_axis);
    let (start_mbp, end_mbp) =
        axis_margin_border_padding(input.axis, input.margin, input.border, input.padding);
    let inherited = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: parent_axis.tracks,
        parent_span: GridTrackSpan::new(start_line, end_line),
        reversed: mapping.reversed,
        start_mbp,
        end_mbp,
        parent_gap: parent_axis.gap,
        subgrid_gap: child_subgrid_gap(
            input.child_style,
            input.axis,
            input.content_box_size,
            input.resolver,
        ),
    })
    .ok()?;
    let parent_major =
        parent_baseline_groups(parent_axis.baseline_groups, parent_axis.tracks.len(), true);
    let parent_minor =
        parent_baseline_groups(parent_axis.baseline_groups, parent_axis.tracks.len(), false);
    let inherited_baselines = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &parent_major,
        parent_minor: &parent_minor,
        parent_span: GridTrackSpan::new(start_line, end_line),
        reversed: mapping.reversed,
        start_mbp,
        end_mbp,
        parent_gap: parent_axis.gap,
        subgrid_gap: inherited.resolved_subgrid_gap,
    })
    .ok()?;

    let (layout_tracks, layout_gap) = inherited_subgrid_layout_tracks(input.axis, &inherited);

    Some(InheritedGridAxis {
        offset: 0.0,
        gap: layout_gap,
        tracks: layout_tracks,
        named_lines: parent_axis.named_lines.clone(),
        area_facts: input
            .parent_area_facts
            .filter(|facts| facts.is_valid_for_axis(mapping.parent_axis))
            .cloned(),
        major_baselines: inherited_baselines.final_major,
        minor_baselines: inherited_baselines.final_minor,
        parent_start: start_line - 1,
        parent_end: end_line - 1,
        reversed: mapping.reversed,
        start_mbp,
        end_mbp,
        gap_difference: inherited.gap_difference,
    })
}

struct SubgridParentAxisData<'a> {
    tracks: &'a [Scalar],
    gap: Scalar,
    named_lines: &'a NamedGridLines,
    baseline_groups: &'a [TrackBaselineGroup],
}

fn subgrid_parent_axis_data<'a>(
    input: &'a SubgridChildAxisContextInput<'a>,
    axis: GridAxisKind,
) -> SubgridParentAxisData<'a> {
    match axis {
        GridAxisKind::Column => SubgridParentAxisData {
            tracks: input.parent_columns,
            gap: input.parent_gap.width,
            named_lines: input.parent_named_columns,
            baseline_groups: &input.parent_baseline_groups.columns,
        },
        GridAxisKind::Row => SubgridParentAxisData {
            tracks: input.parent_rows,
            gap: input.parent_gap.height,
            named_lines: input.parent_named_rows,
            baseline_groups: &input.parent_baseline_groups.rows,
        },
    }
}

pub(super) fn inherited_subgrid_layout_tracks(
    axis: GridAxisKind,
    inherited: &SubgridTrackInheritanceReport,
) -> (Vec<Scalar>, Scalar) {
    if axis == GridAxisKind::Column
        && inherited.gap_difference > 0.0
        && inherited.final_tracks.len() >= 2
        && inherited.final_tracks.contains(&0.0)
    {
        let mut lines = Vec::with_capacity(inherited.end_mbp_removed.len() + 1);
        let mut cursor = 0.0;
        lines.push(cursor);
        for (index, track) in inherited.end_mbp_removed.iter().copied().enumerate() {
            cursor += track;
            if index + 1 < inherited.end_mbp_removed.len() {
                cursor += inherited.parent_gap;
                lines.push(cursor + inherited.gap_difference);
            }
        }
        lines.push(cursor);

        return (
            lines
                .windows(2)
                .map(|pair| (pair[1] - pair[0]).max(0.0))
                .collect(),
            0.0,
        );
    }

    (
        inherited.final_tracks.clone(),
        inherited.resolved_subgrid_gap,
    )
}

fn parent_baseline_groups(
    groups: &[TrackBaselineGroup],
    track_count: usize,
    major: bool,
) -> Vec<Option<Scalar>> {
    let mut baselines = vec![None; track_count];
    for (baseline, group) in baselines.iter_mut().zip(groups) {
        *baseline = if major { group.first } else { group.last };
    }
    baselines
}

fn axis_margin_border_padding(
    axis: GridAxisKind,
    margin: Edges<Option<Scalar>>,
    border: Edges,
    padding: Edges,
) -> (Scalar, Scalar) {
    match axis {
        GridAxisKind::Column => (
            margin.left.unwrap_or(0.0) + border.left + padding.left,
            margin.right.unwrap_or(0.0) + border.right + padding.right,
        ),
        GridAxisKind::Row => (
            margin.top.unwrap_or(0.0) + border.top + padding.top,
            margin.bottom.unwrap_or(0.0) + border.bottom + padding.bottom,
        ),
    }
}

pub(super) fn child_subgrid_gap(
    style: &NodeInput,
    axis: GridAxisKind,
    area_size: Size,
    resolver: &dyn CalcResolver,
) -> ResolvedSubgridGap {
    let (gap, basis) = match axis {
        GridAxisKind::Column => (
            style.gap.width,
            Some(grid_axis_physical_size(style.writing_mode, axis, area_size)),
        ),
        GridAxisKind::Row => (
            style.gap.height,
            Some(grid_axis_physical_size(style.writing_mode, axis, area_size)),
        ),
    };
    match gap {
        Length::Normal => ResolvedSubgridGap::Normal,
        gap => ResolvedSubgridGap::Length(resolve_length_or_zero_with(gap, basis, resolver)),
    }
}

fn grid_axis_physical_size(
    writing_mode: crate::WritingMode,
    axis: GridAxisKind,
    size: Size,
) -> Scalar {
    match (writing_mode.is_vertical(), axis) {
        (false, GridAxisKind::Column) | (true, GridAxisKind::Row) => size.width,
        (false, GridAxisKind::Row) | (true, GridAxisKind::Column) => size.height,
    }
}

#[derive(Clone, Copy)]
pub(super) struct GridItemSizing {
    pub(super) known: Size<Option<Scalar>>,
    pub(super) available: Size,
    pub(super) unresolved_margin: Edges<Option<Scalar>>,
    pub(super) justify_self: AlignItems,
    pub(super) align_self: AlignItems,
}

pub(super) fn grid_item_sizing(
    child_style: &NodeInput,
    container_style: &NodeInput,
    area_size: Size,
    area_width_basis: Size<Option<Scalar>>,
    resolver: &dyn CalcResolver,
) -> GridItemSizing {
    let unresolved_margin = child_style
        .margin
        .zip_inline_size(area_width_basis, |length, basis| {
            resolve_auto_optional_with(length, basis, resolver)
        });
    let margin = unresolved_margin.map(|margin| margin.unwrap_or(0.0));
    let available = Size::new(
        (area_size.width - margin.horizontal_sum()).max(0.0),
        (area_size.height - margin.vertical_sum()).max(0.0),
    );
    let padding = child_style
        .padding
        .zip_inline_size(area_width_basis, |length, basis| {
            resolve_length_or_zero_with(length, basis, resolver)
        });
    let border = child_style
        .border
        .zip_inline_size(area_width_basis, |length, basis| {
            resolve_length_or_zero_with(length, basis, resolver)
        });
    let box_sizing_adjustment = if child_style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::ZERO
    };
    let area_parent = Size::new(Some(area_size.width), Some(area_size.height));
    let inherent_size = child_style
        .size
        .zip_map(area_parent, |dimension, basis| {
            resolve_dimension_with(dimension, basis, resolver)
        })
        .apply_aspect_ratio(child_style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let min_size = child_style
        .min_size
        .zip_map(area_parent, |dimension, basis| {
            resolve_dimension_with(dimension, basis, resolver)
        })
        .add_optional(box_sizing_adjustment)
        .or((padding + border).sum_axes().map(Some))
        .max_optional((padding + border).sum_axes().map(Some))
        .apply_aspect_ratio(child_style.aspect_ratio);
    let max_size = child_style
        .max_size
        .zip_map(area_parent, |dimension, basis| {
            resolve_dimension_with(dimension, basis, resolver)
        })
        .apply_aspect_ratio(child_style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let justify_self = child_style
        .justify_self
        .or(container_style.justify_items)
        .unwrap_or_else(|| {
            if inherent_size.width.is_some() || !child_style.size.width.is_auto() {
                AlignItems::Start
            } else {
                AlignItems::Stretch
            }
        });
    let align_self = child_style
        .align_self
        .or(container_style.align_items)
        .unwrap_or_else(|| {
            if inherent_size.height.is_some()
                || !child_style.size.height.is_auto()
                || (child_style.aspect_ratio.is_some() && child_style.min_size.height.is_auto())
            {
                AlignItems::Start
            } else {
                AlignItems::Stretch
            }
        });
    let width_stretches = unresolved_margin.left.is_some()
        && unresolved_margin.right.is_some()
        && justify_self == AlignItems::Stretch;
    let height_stretches = unresolved_margin.top.is_some()
        && unresolved_margin.bottom.is_some()
        && align_self == AlignItems::Stretch;
    let width = inherent_size
        .width
        .or_else(|| width_stretches.then_some(available.width));
    let height = inherent_size
        .height
        .or_else(|| height_stretches.then_some(available.height));
    let known = if let (Some(ratio), None, Some(height)) =
        (child_style.aspect_ratio, inherent_size.width, height)
        && height_stretches
        && !child_style.min_size.height.is_auto()
    {
        Size::new(Some(height * ratio.get()), Some(height))
    } else {
        Size { width, height }.apply_aspect_ratio(child_style.aspect_ratio)
    }
    .clamp_optional(min_size, max_size);

    GridItemSizing {
        known,
        available,
        unresolved_margin,
        justify_self,
        align_self,
    }
}

pub(super) fn stretch_subgridded_axes<Node>(
    sizing: &mut GridItemSizing,
    item: SubgridItemReport<Node>,
) {
    stretch_subgridded_axis(sizing, item.column);
    stretch_subgridded_axis(sizing, item.row);
}

fn stretch_subgridded_axis(sizing: &mut GridItemSizing, report: SubgridAxisReport) {
    if !report.can_inherit() {
        return;
    }

    match report
        .mapping
        .expect("inheritable subgrid axis must have a valid mapping")
        .parent_axis
    {
        GridAxisKind::Column => {
            sizing.known.width = Some(sizing.available.width);
        }
        GridAxisKind::Row => {
            sizing.known.height = Some(sizing.available.height);
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct GridItemAxis {
    pub(super) area_size: Scalar,
    pub(super) size: Scalar,
    pub(super) margin_start: Option<Scalar>,
    pub(super) margin_end: Option<Scalar>,
    pub(super) alignment: AlignItems,
    pub(super) direction: Direction,
}

#[derive(Clone, Copy)]
pub(super) struct ResolvedGridItemAxis {
    pub(super) offset: Scalar,
    pub(super) margin_start: Scalar,
    pub(super) margin_end: Scalar,
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

pub(super) fn grid_item_axis(axis: GridItemAxis) -> ResolvedGridItemAxis {
    let GridItemAxis {
        area_size,
        size,
        margin_start,
        margin_end,
        alignment,
        direction,
    } = axis;
    let non_auto_start = margin_start.unwrap_or(0.0);
    let non_auto_end = margin_end.unwrap_or(0.0);
    let raw_free_space = area_size - size - non_auto_start - non_auto_end;
    let free_space = raw_free_space.max(0.0);
    let auto_margin_count = usize::from(margin_start.is_none()) + usize::from(margin_end.is_none());
    let auto_margin = if auto_margin_count > 0 {
        free_space / auto_margin_count as Scalar
    } else {
        0.0
    };
    let resolved_start = margin_start.unwrap_or(auto_margin);
    let resolved_end = margin_end.unwrap_or(auto_margin);
    let alignment = alignment.safe_fallback(raw_free_space);
    let offset = match alignment {
        AlignItems::Start | AlignItems::FlexStart | AlignItems::Baseline | AlignItems::Stretch => {
            if direction.is_rtl() {
                area_size - size - resolved_end
            } else {
                resolved_start
            }
        }
        AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline => {
            if direction.is_rtl() {
                resolved_start
            } else {
                area_size - size - resolved_end
            }
        }
        AlignItems::Center => (area_size - size + resolved_start - resolved_end) / 2.0,
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
pub(super) struct AbsoluteGridContext<'a> {
    pub(super) container_style: &'a NodeInput,
    pub(super) constants: &'a Constants,
    pub(super) column: super::GridPlacement,
    pub(super) row: super::GridPlacement,
    pub(super) column_offsets: &'a [Scalar],
    pub(super) row_offsets: &'a [Scalar],
    pub(super) columns: &'a [Scalar],
    pub(super) rows: &'a [Scalar],
    pub(super) gap: Size,
    pub(super) lines: GridLines,
    pub(super) column_line_offset_adjustment: Scalar,
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridAreaInput<'a> {
    pub(super) column: super::GridPlacement,
    pub(super) row: super::GridPlacement,
    pub(super) columns: &'a [Scalar],
    pub(super) rows: &'a [Scalar],
    pub(super) column_offsets: &'a [Scalar],
    pub(super) row_offsets: &'a [Scalar],
    pub(super) gap: Size,
    pub(super) constants: &'a Constants,
    pub(super) columns_are_rtl: bool,
    pub(super) lines: GridLines,
    pub(super) column_line_offset_adjustment: Scalar,
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridAxisInput<'a> {
    pub(super) placement: super::GridPlacement,
    pub(super) tracks: &'a [Scalar],
    pub(super) offsets: &'a [Scalar],
    pub(super) gap: Scalar,
    pub(super) padding_box_location: Scalar,
    pub(super) padding_box_size: Scalar,
    pub(super) is_reverse: bool,
    pub(super) explicit_start: usize,
    pub(super) explicit_count: usize,
    pub(super) reverse_positive_line_offset_adjustment: Scalar,
}

pub(super) fn layout_absolute_grid_child<Tree>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    order: u32,
    child_style: &NodeInput,
    context: AbsoluteGridContext<'_>,
) -> Size
where
    Tree: Compute,
{
    let AbsoluteGridContext {
        container_style,
        constants,
        column,
        row,
        column_offsets,
        row_offsets,
        columns,
        rows,
        gap,
        lines,
        column_line_offset_adjustment,
    } = context;
    let area = if container_style.writing_mode.is_vertical() {
        absolute_grid_area(AbsoluteGridAreaInput {
            column: row,
            row: column,
            columns: rows,
            rows: columns,
            column_offsets: row_offsets,
            row_offsets: column_offsets,
            gap: Size::new(gap.height, gap.width),
            constants,
            columns_are_rtl: container_style.writing_mode == crate::WritingMode::VerticalRl,
            lines: GridLines {
                column_explicit_start: lines.row_explicit_start,
                column_explicit_count: lines.row_explicit_count,
                row_explicit_start: lines.column_explicit_start,
                row_explicit_count: lines.column_explicit_count,
            },
            column_line_offset_adjustment: 0.0,
        })
    } else {
        absolute_grid_area(AbsoluteGridAreaInput {
            column,
            row,
            columns,
            rows,
            column_offsets,
            row_offsets,
            gap,
            constants,
            columns_are_rtl: container_style.direction.is_rtl(),
            lines,
            column_line_offset_adjustment,
        })
    };
    let area_parent = Size::new(Some(area.size.width), Some(area.size.height));
    let resolver = tree.calc_resolver();
    let unresolved_margin = child_style
        .margin
        .zip_inline_size(Size::splat(Some(area.size.width)), |length, basis| {
            resolve_auto_optional_with(length, basis, resolver)
        });
    let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(0.0));
    let available_size = Size::new(
        (area.size.width - non_auto_margin.horizontal_sum()).max(0.0),
        (area.size.height - non_auto_margin.vertical_sum()).max(0.0),
    );
    let area_width_basis = Size::splat(Some(area.size.width));
    let padding = child_style
        .padding
        .zip_inline_size(area_width_basis, |length, basis| {
            resolve_length_or_zero_with(length, basis, resolver)
        });
    let border = child_style
        .border
        .zip_inline_size(area_width_basis, |length, basis| {
            resolve_length_or_zero_with(length, basis, resolver)
        });
    let box_sizing_adjustment = if child_style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::ZERO
    };
    let style_size = child_style
        .size
        .zip_map(area_parent, |dimension, basis| {
            resolve_dimension_with(dimension, basis, resolver)
        })
        .apply_aspect_ratio(child_style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let padding_border_size = (padding + border).sum_axes();
    let min_size = child_style
        .min_size
        .zip_map(area_parent, |dimension, basis| {
            resolve_dimension_with(dimension, basis, resolver)
        })
        .add_optional(box_sizing_adjustment)
        .or(padding_border_size.map(Some))
        .max_optional(padding_border_size.map(Some))
        .apply_aspect_ratio(child_style.aspect_ratio);
    let max_size = child_style
        .max_size
        .zip_map(area_parent, |dimension, basis| {
            resolve_dimension_with(dimension, basis, resolver)
        })
        .apply_aspect_ratio(child_style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let inset = child_style.inset.zip_size(area_parent, |length, basis| {
        resolve_auto_optional_with(length, basis, resolver)
    });
    let mut known = Size::new(
        style_size.width.or_else(|| {
            inset.left.zip(inset.right).map(|(left, right)| {
                (area.size.width - non_auto_margin.horizontal_sum() - left - right).max(0.0)
            })
        }),
        style_size.height.or_else(|| {
            inset.top.zip(inset.bottom).map(|(top, bottom)| {
                (area.size.height - non_auto_margin.vertical_sum() - top - bottom).max(0.0)
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
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known,
            parent: area_parent,
            available: Size::new(
                Available::definite(available_size.width),
                Available::definite(available_size.height),
            ),
        },
    );
    let final_size = known
        .unwrap_or(output.size)
        .clamp_optional(min_size, max_size);
    let scrollbar_size = Size::new(
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
    );
    let horizontal_axis = absolute_grid_axis(AbsoluteGridAxis {
        area_location: area.location.x,
        static_area_location: area.static_location.x,
        area_size: area.size.width,
        static_area_size: area.static_size.width,
        size: final_size.width,
        margin_start: unresolved_margin.left,
        margin_end: unresolved_margin.right,
        inset_start: inset.left,
        inset_end: inset.right,
        alignment: child_style
            .justify_self
            .unwrap_or(container_style.justify_items.unwrap_or(AlignItems::Start)),
        direction: grid_physical_axis_direction(
            container_style.writing_mode,
            container_style.direction,
            PhysicalGridAxis::Horizontal,
        ),
    });
    let vertical_axis = absolute_grid_axis(AbsoluteGridAxis {
        area_location: area.location.y,
        static_area_location: area.static_location.y,
        area_size: area.size.height,
        static_area_size: area.static_size.height,
        size: final_size.height,
        margin_start: unresolved_margin.top,
        margin_end: unresolved_margin.bottom,
        inset_start: inset.top,
        inset_end: inset.bottom,
        alignment: child_style
            .align_self
            .unwrap_or(container_style.align_items.unwrap_or(AlignItems::Start)),
        direction: grid_physical_axis_direction(
            container_style.writing_mode,
            container_style.direction,
            PhysicalGridAxis::Vertical,
        ),
    });
    let location = Point::new(horizontal_axis.location, vertical_axis.location);
    let margin = Edges {
        left: horizontal_axis.margin_start,
        right: horizontal_axis.margin_end,
        top: vertical_axis.margin_start,
        bottom: vertical_axis.margin_end,
    };

    tree.set_unrounded(
        child,
        NodeOutput {
            order,
            location,
            size: final_size,
            content_size: output.content_size,
            scrollbar_size,
            border,
            padding,
            margin,
        },
    );

    content_size_contribution(
        Point::new(
            location.x - constants.content_box_inset.left,
            location.y - constants.content_box_inset.top,
        ),
        final_size,
        output.content_size,
        child_style.overflow,
    )
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridArea {
    pub(super) location: Point,
    pub(super) static_location: Point,
    pub(super) size: Size,
    pub(super) static_size: Size,
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridAxisArea {
    pub(super) location: Scalar,
    pub(super) size: Scalar,
}

#[derive(Clone, Copy)]
pub(super) struct AbsoluteGridAxis {
    pub(super) area_location: Scalar,
    pub(super) static_area_location: Scalar,
    pub(super) area_size: Scalar,
    pub(super) static_area_size: Scalar,
    pub(super) size: Scalar,
    pub(super) margin_start: Option<Scalar>,
    pub(super) margin_end: Option<Scalar>,
    pub(super) inset_start: Option<Scalar>,
    pub(super) inset_end: Option<Scalar>,
    pub(super) alignment: AlignItems,
    pub(super) direction: Direction,
}

#[derive(Clone, Copy)]
pub(super) struct ResolvedAbsoluteGridAxis {
    pub(super) location: Scalar,
    pub(super) margin_start: Scalar,
    pub(super) margin_end: Scalar,
}

pub(super) fn absolute_grid_axis(axis: AbsoluteGridAxis) -> ResolvedAbsoluteGridAxis {
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
        direction,
    } = axis;
    let non_auto_start = margin_start.unwrap_or(0.0);
    let non_auto_end = margin_end.unwrap_or(0.0);
    let raw_free_space = area_size - size - non_auto_start - non_auto_end;
    let free_space = raw_free_space.max(0.0);
    let auto_margin_count = usize::from(margin_start.is_none()) + usize::from(margin_end.is_none());
    let auto_margin = if auto_margin_count > 0 {
        free_space / auto_margin_count as Scalar
    } else {
        0.0
    };
    let resolved_start = margin_start.unwrap_or(auto_margin);
    let resolved_end = margin_end.unwrap_or(auto_margin);
    let uses_static_area = inset_start.is_none() && inset_end.is_none();
    let offset = match (inset_start, inset_end) {
        (Some(_), Some(end)) if direction.is_rtl() => area_size - end - size - non_auto_end,
        (Some(start), _) => start + non_auto_start,
        (None, Some(end)) => area_size - end - size - non_auto_end,
        (None, None) => match alignment.safe_fallback(raw_free_space) {
            AlignItems::Start
            | AlignItems::FlexStart
            | AlignItems::Baseline
            | AlignItems::Stretch
                if direction.is_rtl() =>
            {
                static_area_size - size - resolved_end
            }
            AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline
                if direction.is_rtl() =>
            {
                resolved_start
            }
            AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline => {
                static_area_size - size - resolved_end
            }
            AlignItems::Center => (static_area_size - size + resolved_start - resolved_end) / 2.0,
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

pub(super) fn relative_inset_offset(
    inset: Edges<Option<Scalar>>,
    direction: Direction,
    position: Position,
) -> Point {
    if position != Position::Relative {
        return Point::ZERO;
    }

    Point::new(
        if direction.is_rtl() {
            inset
                .right
                .map(|right| -right)
                .or(inset.left)
                .unwrap_or(0.0)
        } else {
            inset
                .left
                .or_else(|| inset.right.map(|right| -right))
                .unwrap_or(0.0)
        },
        inset
            .top
            .or_else(|| inset.bottom.map(|bottom| -bottom))
            .unwrap_or(0.0),
    )
}

pub(super) fn max_size(a: Size, b: Size) -> Size {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

pub(super) fn content_size_contribution(
    location: Point,
    size: Size,
    content_size: Size,
    overflow: Point<Overflow>,
) -> Size {
    let contribution_size = Size::new(
        if overflow.x == Overflow::Visible {
            size.width.max(content_size.width)
        } else {
            size.width
        },
        if overflow.y == Overflow::Visible {
            size.height.max(content_size.height)
        } else {
            size.height
        },
    );
    if contribution_size.width <= 0.0 || contribution_size.height <= 0.0 {
        return Size::ZERO;
    }

    let max_x = (location.x + contribution_size.width).max(0.0);
    let min_x = location.x.min(0.0);
    let max_y = (location.y + contribution_size.height).max(0.0);
    let min_y = location.y.min(0.0);
    Size::new(max_x - min_x, max_y - min_y)
}
