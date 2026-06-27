use super::*;
#[derive(Clone, Copy)]
pub(super) struct IntrinsicGrid<'a, Node> {
    pub(super) style: &'a NodeInput,
    pub(super) constants: &'a Constants,
    pub(super) column_tracks: &'a [TrackSizing],
    pub(super) row_tracks: &'a [TrackSizing],
    pub(super) gap: Size,
    pub(super) percent_basis: Size<Option<Scalar>>,
    pub(super) lines: GridLines,
    pub(super) named_columns: &'a NamedGridLines,
    pub(super) named_rows: &'a NamedGridLines,
    pub(super) area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<Node>,
}

#[derive(Clone, Copy, Default)]
pub(super) struct IntrinsicGridLowerBounds<'a> {
    pub(super) columns: Option<&'a [Scalar]>,
    pub(super) rows: Option<&'a [Scalar]>,
}

#[derive(Clone, Copy)]
struct RowIntrinsicContribution {
    start: usize,
    end: usize,
    contributes_to_row_size: bool,
    contribution_kind: IntrinsicSpanContribution,
    contribution: Scalar,
    participation: BaselineParticipation,
    geometry: BaselineGeometry,
}

pub(super) fn intrinsic_track_sizes<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    grid: IntrinsicGrid<'_, <Tree as Traverse>::Node>,
    available: Size<Available>,
    lower_bounds: IntrinsicGridLowerBounds<'_>,
) -> (Vec<Scalar>, Vec<Scalar>)
where
    Tree: Compute,
{
    let style = grid.style;
    let constants = grid.constants;
    let column_tracks = grid.column_tracks;
    let row_tracks = grid.row_tracks;
    let column_count = column_tracks.len();
    let row_count = row_tracks.len();
    let mut columns: Vec<Scalar> = lower_bounds
        .columns
        .map(|bounds| bounds.iter().copied().take(column_count).collect())
        .unwrap_or_else(|| vec![0.0; column_count]);
    columns.resize(column_count, 0.0);
    let mut rows: Vec<Scalar> = lower_bounds
        .rows
        .map(|bounds| bounds.iter().copied().take(row_count).collect())
        .unwrap_or_else(|| vec![0.0; row_count]);
    rows.resize(row_count, 0.0);
    let mut row_contributions = Vec::new();
    let zero_columns: Vec<Scalar> = vec![0.0; column_count];
    let zero_rows: Vec<Scalar> = vec![0.0; row_count];
    let children = tree.children(node).collect::<Vec<_>>();
    let placed_areas = resolve_grid_child_areas(ResolveGridChildAreasInput {
        children: &children,
        placements: grid.placements,
        style,
        columns: &zero_columns,
        rows: &zero_rows,
        gap: Size::ZERO,
        lines: grid.lines,
    });
    let column_area_sizes = columns.clone();
    let row_area_sizes = rows.clone();
    apply_subgrid_intrinsic_contributions(
        tree,
        SubgridIntrinsicContributionInput {
            constants,
            axis: GridAxisKind::Column,
            tracks: column_tracks,
            sizes: &mut columns,
            percent_basis: grid.percent_basis.width,
            gap: grid.gap.width,
            container_gap: grid.gap,
            available,
            children: &children,
            placed_areas: &placed_areas,
            subgrid_report: grid.subgrid_report,
            named_columns: grid.named_columns,
            named_rows: grid.named_rows,
            area_facts: grid.area_facts,
            column_sizes: &column_area_sizes,
            row_sizes: &row_area_sizes,
        },
    );
    let column_area_sizes = columns.clone();
    let row_area_sizes = rows.clone();
    apply_subgrid_intrinsic_contributions(
        tree,
        SubgridIntrinsicContributionInput {
            constants,
            axis: GridAxisKind::Row,
            tracks: row_tracks,
            sizes: &mut rows,
            percent_basis: grid.percent_basis.height,
            gap: grid.gap.height,
            container_gap: grid.gap,
            available,
            children: &children,
            placed_areas: &placed_areas,
            subgrid_report: grid.subgrid_report,
            named_columns: grid.named_columns,
            named_rows: grid.named_rows,
            area_facts: grid.area_facts,
            column_sizes: &column_area_sizes,
            row_sizes: &row_area_sizes,
        },
    );

    for (index, (child, area)) in children.into_iter().zip(placed_areas).enumerate() {
        let child_style = tree.node_input(child).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }

        let Some(mut area) = area else {
            continue;
        };
        if area.row >= row_count || area.column >= column_count {
            continue;
        }

        let column_start = area.column;
        let column_end = area.column_end;
        let row_start = area.row;
        let row_end = area.row_end;
        let column_span_tracks = column_tracks.get(column_start..column_end);
        let row_span_tracks = row_tracks.get(row_start..row_end);
        area.size = Size::new(
            track_span_sum(&columns, column_start, column_end, grid.gap.width),
            track_span_sum(&rows, row_start, row_end, grid.gap.height),
        );
        let inherited_column_subgrid = grid.subgrid_report.items.get(index).is_some_and(|item| {
            item_inherits_parent_axis(&child_style, *item, GridAxisKind::Column)
        });
        let inherited_row_subgrid =
            grid.subgrid_report.items.get(index).is_some_and(|item| {
                item_inherits_parent_axis(&child_style, *item, GridAxisKind::Row)
            });
        let contributes_column = !inherited_column_subgrid
            && !scroll_container_auto_minimum_zero_inline(&child_style)
            && column_span_tracks.is_some_and(|tracks| {
                tracks
                    .iter()
                    .any(|track| track_accepts_intrinsic_contribution(*track))
            });
        let align_self = child_style
            .align_self
            .or(style.align_items)
            .unwrap_or(AlignItems::Stretch);
        let contributes_row = !inherited_row_subgrid
            && !scroll_container_auto_minimum_zero_block(&child_style)
            && row_span_tracks.is_some_and(|tracks| {
                tracks
                    .iter()
                    .any(|track| track_accepts_intrinsic_contribution(*track))
            });
        let row_baseline_candidate = !inherited_row_subgrid
            && row_span_tracks.is_some()
            && matches!(align_self, AlignItems::Baseline | AlignItems::LastBaseline);
        if !contributes_column && !contributes_row && !row_baseline_candidate {
            continue;
        }

        let spans_min_content_column =
            column_tracks
                .get(column_start..column_end)
                .is_some_and(|tracks| {
                    tracks
                        .iter()
                        .any(|track| track_accepts_min_content_span_priority(*track))
                });
        let sizing = grid_item_sizing(
            &child_style,
            style,
            area.size,
            Size::splat(Some(area.size.width)),
            tree.calc_resolver(),
        );
        let output = if available.width == Available::MIN_CONTENT
            && child_style.overflow.x.clips_contents()
            && !spans_min_content_column
        {
            ComputeOutput::HIDDEN
        } else {
            compute_intrinsic_grid_child(
                tree,
                child,
                IntrinsicGridChildInput {
                    child_style: &child_style,
                    grid,
                    area,
                    columns: &columns,
                    rows: &rows,
                    sizing,
                    subgrid_item: grid.subgrid_report.items.get(index).copied(),
                    input: ComputeInput {
                        run_mode: if matches!(
                            align_self,
                            AlignItems::Baseline | AlignItems::LastBaseline
                        ) {
                            RunMode::PerformLayout
                        } else {
                            RunMode::ComputeSize
                        },
                        sizing_mode: SizingMode::InherentSize,
                        axis: RequestedAxis::Both,
                        known: Size::NONE,
                        parent: Size::new(
                            constants.node_inner_size.width,
                            constants.node_inner_size.height,
                        ),
                        available,
                    },
                },
            )
        };
        let resolver = tree.calc_resolver();
        let margin =
            intrinsic_contribution_margin(&child_style, constants.node_inner_size.width, resolver);

        if contributes_column {
            let contribution_kind =
                IntrinsicSpanContribution::for_axis(available.width, child_style.overflow.x);
            if column_end == column_start + 1 {
                columns[column_start] =
                    columns[column_start].max(output.size.width + margin.horizontal_sum());
            } else if available.width == Available::MIN_CONTENT
                && column_span_tracks.is_some_and(|tracks| {
                    tracks
                        .iter()
                        .any(|track| track_percent_fraction(track, resolver) > 0.0)
                        && tracks
                            .iter()
                            .all(|track| track_flex_factor(*track).is_none())
                })
            {
                distribute_min_content_span_with_percent(
                    &mut columns[column_start..column_end],
                    &column_tracks[column_start..column_end],
                    child_style.overflow.x,
                    grid.percent_basis.width,
                    output.size.width + margin.horizontal_sum(),
                    resolver,
                );
            } else {
                distribute_intrinsic_span(
                    &mut columns[column_start..column_end],
                    &column_tracks[column_start..column_end],
                    contribution_kind,
                    grid.percent_basis.width,
                    span_contribution(
                        output.size.width + margin.horizontal_sum(),
                        column_end - column_start,
                        grid.gap.width,
                    ),
                    resolver,
                );
            }
        }
        if contributes_row || row_baseline_candidate {
            let contribution_kind =
                IntrinsicSpanContribution::for_axis(available.height, child_style.overflow.y);
            let baselines = output.baselines();
            let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
                &child_style,
                constants,
                tree.calc_resolver(),
            );
            let participation = baseline_participation(
                align_self,
                block_auto_margins,
                row_span_tracks.is_some_and(|tracks| {
                    synthesized_baseline_would_cycle(align_self, baselines, tracks)
                }),
                baselines,
            );
            row_contributions.push(RowIntrinsicContribution {
                start: row_start,
                end: row_end,
                contributes_to_row_size: contributes_row,
                contribution_kind,
                contribution: output.size.height + margin.vertical_sum(),
                participation,
                geometry: baseline_geometry_for_intrinsic_contribution(output, margin),
            });
        }
    }

    let row_baseline_groups =
        row_baseline_groups_for_intrinsic_contributions(&row_contributions, row_count);
    let resolver = tree.calc_resolver();
    for item in row_contributions {
        if !item.contributes_to_row_size {
            continue;
        }
        let shim = row_baseline_shim(item, &row_baseline_groups);
        let contribution = item.contribution + shim.before + shim.after;
        if item.end == item.start + 1 {
            rows[item.start] = rows[item.start].max(contribution);
        } else {
            distribute_intrinsic_span(
                &mut rows[item.start..item.end],
                &row_tracks[item.start..item.end],
                item.contribution_kind,
                grid.percent_basis.height,
                span_contribution(contribution, item.end - item.start, grid.gap.height),
                resolver,
            );
        }
    }

    (columns, rows)
}

fn row_baseline_groups_for_intrinsic_contributions(
    contributions: &[RowIntrinsicContribution],
    row_count: usize,
) -> Vec<TrackBaselineGroup> {
    let mut groups = vec![TrackBaselineGroup::default(); row_count];
    for item in contributions {
        if !item.participation.participates {
            continue;
        }

        match item.participation.group {
            Some(BaselineGroupKind::Major) => {
                if let Some(group) = groups.get_mut(item.start).map(|group| &mut group.first) {
                    *group = Some(group.unwrap_or(0.0).max(item.geometry.major_baseline));
                }
            }
            Some(BaselineGroupKind::Minor) => {
                if let Some(row) = item.end.checked_sub(1)
                    && let Some(group) = groups.get_mut(row).map(|group| &mut group.last)
                {
                    *group = Some(group.unwrap_or(0.0).max(item.geometry.minor_baseline));
                }
            }
            None => {}
        }
    }
    groups
}

fn row_baseline_shim(
    item: RowIntrinsicContribution,
    groups: &[TrackBaselineGroup],
) -> BaselineShim {
    let Some(group_kind) = item.participation.group else {
        return BaselineShim::default();
    };
    let group_index = match group_kind {
        BaselineGroupKind::Major => item.start,
        BaselineGroupKind::Minor => item.end.saturating_sub(1),
    };
    let shared = groups.get(group_index).copied().unwrap_or_default();
    baseline_shim_for_intrinsic_contribution(item.participation, item.geometry, shared)
}

fn baseline_geometry_for_intrinsic_contribution(
    output: ComputeOutput,
    margin: Edges,
) -> BaselineGeometry {
    let baselines = output.baselines();
    let first_baseline = baselines.first_or_synthesize_block(output.size);
    let last_baseline = baselines.last_or_synthesize_block(output.size);
    BaselineGeometry {
        available_span_size: 0.0,
        margin_box_size: output.size.height + margin.vertical_sum(),
        major_baseline: margin.top + first_baseline,
        minor_baseline: margin.bottom + output.size.height - last_baseline,
    }
}

fn block_auto_margins_for_intrinsic_contribution(
    style: &NodeInput,
    constants: &Constants,
    resolver: &dyn CalcResolver,
) -> bool {
    let margin = style.margin.zip_inline_size(
        Size::splat(constants.node_inner_size.width),
        |length, basis| resolve_auto_optional_with(length, basis, resolver),
    );
    margin.top.is_none() || margin.bottom.is_none()
}

struct IntrinsicGridChildInput<'a, Node> {
    child_style: &'a NodeInput,
    grid: IntrinsicGrid<'a, Node>,
    area: GridArea,
    columns: &'a [Scalar],
    rows: &'a [Scalar],
    sizing: GridItemSizing,
    subgrid_item: Option<SubgridItemReport<Node>>,
    input: ComputeInput,
}

fn compute_intrinsic_grid_child<Tree>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    args: IntrinsicGridChildInput<'_, <Tree as Traverse>::Node>,
) -> ComputeOutput
where
    Tree: Compute,
{
    let IntrinsicGridChildInput {
        child_style,
        grid,
        area,
        columns,
        rows,
        mut sizing,
        subgrid_item,
        input,
    } = args;

    let Some(subgrid_item) = subgrid_item else {
        return tree.compute_child(child, input);
    };
    if !matches!(
        child_style.display.inner_display(),
        Display::Grid | Display::GridLanes
    ) {
        return tree.compute_child(child, input);
    }
    let resolver = tree.calc_resolver();
    let needs_context = needs_intrinsic_subgrid_context(child_style, subgrid_item, area, resolver);
    if !input.run_mode.is_perform_layout() && !needs_context {
        return tree.compute_child(child, input);
    }

    stretch_subgridded_axes(&mut sizing, subgrid_item);
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
    let margin = sizing.unresolved_margin.map(|margin| margin.unwrap_or(0.0));
    let content_box_size =
        (area.size - margin.sum_axes() - padding.sum_axes() - border.sum_axes()).max(Size::ZERO);
    let baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); grid.row_tracks.len()],
        columns: vec![TrackBaselineGroup::default(); grid.column_tracks.len()],
    };
    let child_context = subgrid_child_parent_context(SubgridChildParentContextInput {
        item: subgrid_item,
        child_style,
        area,
        content_box_size,
        columns,
        rows,
        gap: grid.gap,
        parent_named_columns: grid.named_columns,
        parent_named_rows: grid.named_rows,
        parent_area_facts: grid.area_facts,
        parent_baseline_groups: &baseline_groups,
        margin: sizing.unresolved_margin,
        border,
        padding,
        resolver,
    });
    if !child_context.has_inherited_axis() {
        return tree.compute_child(child, input);
    }

    let (known, parent, available) = if input.run_mode.is_perform_layout() {
        (
            sizing.known,
            Size::new(Some(area.size.width), Some(area.size.height)),
            Size::new(
                Available::Definite(sizing.available.width.max(0.0)),
                input.available.height,
            ),
        )
    } else {
        (
            input.known,
            intrinsic_subgrid_child_parent(input.parent, area.size, subgrid_item),
            input.available,
        )
    };
    let child_input = ComputeInput {
        known,
        parent,
        available,
        ..input
    };
    compute_grid_with_context_result(tree, child, child_input, child_context).output
}

pub(super) fn needs_intrinsic_subgrid_context<Node>(
    style: &NodeInput,
    item: SubgridItemReport<Node>,
    area: GridArea,
    resolver: &dyn CalcResolver,
) -> bool
where
    Node: Copy,
{
    let inherits_rows = item_inherits_parent_axis(style, item, GridAxisKind::Row);
    let inherits_columns = item_inherits_parent_axis(style, item, GridAxisKind::Column);
    let spans_multiple_inherited_columns = area.column_end > area.column + 1;

    (inherits_rows && inherits_columns && spans_multiple_inherited_columns)
        || (inherits_rows
            && (style.grid_auto_flow.is_column()
                || track_components_have_percent_sizing(&style.grid_template_columns, resolver)))
        || (inherits_columns
            && !style.size.height.is_auto()
            && track_components_have_percent_sizing(&style.grid_template_rows, resolver))
}

pub(super) fn track_components_have_percent_sizing(
    components: &[TrackComponent],
    resolver: &dyn CalcResolver,
) -> bool {
    components
        .iter()
        .any(|component| track_component_has_percent_sizing(component, resolver))
}

fn track_component_has_percent_sizing(
    component: &TrackComponent,
    resolver: &dyn CalcResolver,
) -> bool {
    match component {
        TrackComponent::Track(track) => track_has_percent_sizing(track, resolver),
        TrackComponent::Repeat(repeat) => repeat
            .components
            .iter()
            .any(|component| track_component_has_percent_sizing(component, resolver)),
        _ => false,
    }
}

fn intrinsic_subgrid_child_parent<Node>(
    input_parent: Size<Option<Scalar>>,
    area_size: Size,
    item: SubgridItemReport<Node>,
) -> Size<Option<Scalar>> {
    let mut parent = input_parent;
    apply_intrinsic_subgrid_axis_parent(&mut parent, area_size, item.column);
    apply_intrinsic_subgrid_axis_parent(&mut parent, area_size, item.row);
    parent
}

fn apply_intrinsic_subgrid_axis_parent(
    parent: &mut Size<Option<Scalar>>,
    area_size: Size,
    report: SubgridAxisReport,
) {
    if !report.can_inherit() {
        return;
    }
    match report
        .mapping
        .expect("inheritable subgrid axis must have a valid mapping")
        .parent_axis
    {
        GridAxisKind::Column => parent.width = Some(area_size.width),
        GridAxisKind::Row => parent.height = Some(area_size.height),
    }
}

struct SubgridIntrinsicContributionInput<'a, Node> {
    constants: &'a Constants,
    axis: GridAxisKind,
    tracks: &'a [TrackSizing],
    sizes: &'a mut [Scalar],
    percent_basis: Option<Scalar>,
    gap: Scalar,
    container_gap: Size,
    available: Size<Available>,
    children: &'a [Node],
    placed_areas: &'a [Option<GridArea>],
    subgrid_report: &'a GridSubgridReport<Node>,
    named_columns: &'a NamedGridLines,
    named_rows: &'a NamedGridLines,
    area_facts: Option<&'a GridAreaNameFacts>,
    column_sizes: &'a [Scalar],
    row_sizes: &'a [Scalar],
}

fn apply_subgrid_intrinsic_contributions<Tree>(
    tree: &mut Tree,
    input: SubgridIntrinsicContributionInput<'_, <Tree as Traverse>::Node>,
) -> Vec<<Tree as Traverse>::Node>
where
    Tree: Compute,
{
    if input.tracks.is_empty() || input.subgrid_report.items.is_empty() {
        return Vec::new();
    }

    let intrinsic_min_track_facts = input
        .tracks
        .iter()
        .map(|track| track.min.is_intrinsic())
        .collect::<Vec<_>>();
    let Ok(report) = collect_grid_subgrid_intrinsic_traversal(
        tree,
        GridSubgridIntrinsicTraversalInput {
            axis: input.axis,
            children: input.children,
            placed_areas: input.placed_areas,
            subgrid_report: input.subgrid_report,
            named_columns: input.named_columns,
            named_rows: input.named_rows,
            area_facts: input.area_facts,
            parent_gap: input.container_gap,
            column_sizes: input.column_sizes,
            row_sizes: input.row_sizes,
            container_size: input.constants.node_inner_size,
            intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(&intrinsic_min_track_facts),
        },
    ) else {
        return Vec::new();
    };

    for (index, lower_bound) in report.edge_lower_bounds.into_iter().enumerate() {
        if let Some(size) = input.sizes.get_mut(index) {
            *size = size.max(lower_bound);
        }
    }

    let mut leaves = report.leaves;
    leaves.sort_by_key(|leaf| {
        leaf.ancestor_span
            .end
            .saturating_sub(leaf.ancestor_span.start)
    });
    let mut contributing_roots = Vec::new();
    for leaf in leaves {
        let child_style = tree.node_input(leaf.node).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }
        if subgrid_leaf_size_depends_on_queried_axis(&child_style, input.axis, tree.calc_resolver())
        {
            continue;
        }
        if scroll_container_auto_minimum_zero(&child_style, input.axis) {
            continue;
        }
        let start = leaf.ancestor_span.start - 1;
        let end = leaf.ancestor_span.end - 1;
        let Some(span_tracks) = input.tracks.get(start..end) else {
            continue;
        };
        if !span_tracks
            .iter()
            .any(|track| track_accepts_intrinsic_contribution(*track))
        {
            continue;
        }

        let row_available_inline_size = (input.axis == GridAxisKind::Row
            && child_style.size.width.is_auto())
        .then_some(leaf.available_inline_size)
        .flatten()
        .filter(|width| *width > 0.0);
        if input.axis == GridAxisKind::Row
            && child_style.size.width.is_auto()
            && row_available_inline_size.is_none()
        {
            continue;
        }
        let row_known_inline_size =
            row_available_inline_size.filter(|_| leaf.available_inline_size_is_known);
        let available = if let Some(width) = row_available_inline_size {
            Size::new(Available::Definite(width), input.available.height)
        } else {
            input.available
        };
        let output = tree.compute_child(
            leaf.node,
            ComputeInput {
                run_mode: RunMode::ComputeSize,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::new(row_known_inline_size, None),
                parent: Size::new(
                    input.constants.node_inner_size.width,
                    input.constants.node_inner_size.height,
                ),
                available,
            },
        );
        let margin = intrinsic_contribution_margin(
            &child_style,
            input.constants.node_inner_size.width,
            tree.calc_resolver(),
        );
        let contribution = axis_size(output.size, input.axis)
            + axis_margin_sum(margin, input.axis)
            + adjustment_sum(&leaf.accumulated_edge_adjustment, start, end)
            + adjustment_sum(&leaf.accumulated_gap_adjustment, start, end);
        let contribution_kind = IntrinsicSpanContribution::for_axis(
            axis_available(input.available, input.axis),
            axis_overflow(&child_style, input.axis),
        );
        if let Some(root) = leaf.root_node
            && leaf.root_axis_fully_inherited
            && !contributing_roots.contains(&root)
        {
            contributing_roots.push(root);
        }
        if end == start + 1 {
            input.sizes[start] = input.sizes[start].max(contribution);
        } else if input.axis == GridAxisKind::Column
            && axis_available(input.available, input.axis) == Available::MIN_CONTENT
            && span_tracks
                .iter()
                .any(|track| track_percent_fraction(track, tree.calc_resolver()) > 0.0)
            && span_tracks
                .iter()
                .all(|track| track_flex_factor(*track).is_none())
        {
            distribute_min_content_span_with_percent(
                &mut input.sizes[start..end],
                span_tracks,
                child_style.overflow.x,
                input.percent_basis,
                contribution,
                tree.calc_resolver(),
            );
        } else {
            distribute_intrinsic_span(
                &mut input.sizes[start..end],
                span_tracks,
                contribution_kind,
                input.percent_basis,
                span_contribution(contribution, end - start, input.gap),
                tree.calc_resolver(),
            );
        }
    }
    contributing_roots
}

fn scroll_container_auto_minimum_zero(style: &NodeInput, axis: GridAxisKind) -> bool {
    match axis {
        GridAxisKind::Column => scroll_container_auto_minimum_zero_inline(style),
        GridAxisKind::Row => scroll_container_auto_minimum_zero_block(style),
    }
}

fn subgrid_leaf_size_depends_on_queried_axis(
    style: &NodeInput,
    axis: GridAxisKind,
    resolver: &dyn CalcResolver,
) -> bool {
    match axis {
        GridAxisKind::Column => style.size.width.depends_on_basis_with(resolver),
        GridAxisKind::Row => style.size.height.depends_on_basis_with(resolver),
    }
}

fn item_inherits_parent_axis<Node>(
    style: &NodeInput,
    item: SubgridItemReport<Node>,
    parent_axis: GridAxisKind,
) -> bool
where
    Node: Copy,
{
    [GridAxisKind::Column, GridAxisKind::Row]
        .into_iter()
        .any(|child_axis| {
            if !track_components_request_subgrid(style, child_axis) {
                return false;
            }
            let report = match child_axis {
                GridAxisKind::Column => item.column,
                GridAxisKind::Row => item.row,
            };
            report
                .mapping
                .is_ok_and(|mapping| report.can_inherit() && mapping.parent_axis == parent_axis)
        })
}

fn track_components_request_subgrid(style: &NodeInput, axis: GridAxisKind) -> bool {
    let components = match axis {
        GridAxisKind::Column => &style.grid_template_columns,
        GridAxisKind::Row => &style.grid_template_rows,
    };

    components
        .iter()
        .any(|component| matches!(component, TrackComponent::Subgrid(_)))
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

fn axis_available(available: Size<Available>, axis: GridAxisKind) -> Available {
    match axis {
        GridAxisKind::Column => available.width,
        GridAxisKind::Row => available.height,
    }
}

fn axis_overflow(style: &NodeInput, axis: GridAxisKind) -> Overflow {
    match axis {
        GridAxisKind::Column => style.overflow.x,
        GridAxisKind::Row => style.overflow.y,
    }
}

fn adjustment_sum(adjustments: &[Scalar], start: usize, end: usize) -> Scalar {
    adjustments.get(start..end).unwrap_or_default().iter().sum()
}

pub(super) fn constrained_row_intrinsic_sizes<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    grid: IntrinsicGrid<'_, <Tree as Traverse>::Node>,
    columns: &[Scalar],
    gap: Size,
) -> Vec<Scalar>
where
    Tree: Compute,
{
    let row_count = grid.row_tracks.len();
    let mut rows: Vec<Scalar> = vec![0.0; row_count];
    if columns.is_empty() || row_count == 0 {
        return rows;
    }
    let mut row_contributions = Vec::new();

    let zero_rows: Vec<Scalar> = vec![0.0; row_count];
    let children = tree.children(node).collect::<Vec<_>>();
    let placed_areas = resolve_grid_child_areas(ResolveGridChildAreasInput {
        children: &children,
        placements: grid.placements,
        style: grid.style,
        columns,
        rows: &zero_rows,
        gap,
        lines: grid.lines,
    });
    let published_row_subgrid_roots = if grid
        .subgrid_report
        .items
        .iter()
        .zip(children.iter().copied())
        .any(|(item, child)| {
            item_inherits_parent_axis(tree.node_input(child), *item, GridAxisKind::Row)
        }) {
        apply_subgrid_intrinsic_contributions(
            tree,
            SubgridIntrinsicContributionInput {
                constants: grid.constants,
                axis: GridAxisKind::Row,
                tracks: grid.row_tracks,
                sizes: &mut rows,
                percent_basis: grid.percent_basis.height,
                gap: gap.height,
                container_gap: gap,
                available: Size::new(
                    Available::Definite(track_sum(columns, gap.width)),
                    Available::MAX_CONTENT,
                ),
                children: &children,
                placed_areas: &placed_areas,
                subgrid_report: grid.subgrid_report,
                named_columns: grid.named_columns,
                named_rows: grid.named_rows,
                area_facts: grid.area_facts,
                column_sizes: columns,
                row_sizes: &zero_rows,
            },
        )
    } else {
        Vec::new()
    };

    for (index, (child, area)) in children.into_iter().zip(placed_areas).enumerate() {
        let child_style = tree.node_input(child).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }

        let Some(area) = area else {
            continue;
        };
        if area.row >= row_count || area.column >= columns.len() {
            continue;
        }
        if scroll_container_auto_minimum_zero_block(&child_style) {
            continue;
        }
        if area.row_end > row_count {
            continue;
        }
        if let Some(item) = grid.subgrid_report.items.get(index)
            && item_inherits_parent_axis(&child_style, *item, GridAxisKind::Row)
            && published_row_subgrid_roots.contains(&child)
        {
            continue;
        }
        let sizing = grid_item_sizing(
            &child_style,
            grid.style,
            area.size,
            Size::splat(Some(area.size.width)),
            tree.calc_resolver(),
        );
        let margin = intrinsic_contribution_margin(
            &child_style,
            Some(area.size.width),
            tree.calc_resolver(),
        );
        let output = compute_intrinsic_grid_child(
            tree,
            child,
            IntrinsicGridChildInput {
                child_style: &child_style,
                grid,
                area,
                columns,
                rows: &zero_rows,
                sizing,
                subgrid_item: grid.subgrid_report.items.get(index).copied(),
                input: ComputeInput {
                    run_mode: if matches!(
                        sizing.align_self,
                        AlignItems::Baseline | AlignItems::LastBaseline
                    ) {
                        RunMode::PerformLayout
                    } else {
                        RunMode::ComputeSize
                    },
                    sizing_mode: SizingMode::InherentSize,
                    axis: RequestedAxis::Both,
                    known: Size::new(sizing.known.width, None),
                    parent: Size::new(Some(area.size.width), Some(area.size.height)),
                    available: Size::new(
                        Available::definite(sizing.available.width),
                        Available::MAX_CONTENT,
                    ),
                },
            },
        );
        let baselines = output.baselines();
        let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
            &child_style,
            grid.constants,
            tree.calc_resolver(),
        );
        let row_span_tracks = grid.row_tracks.get(area.row..area.row_end).unwrap_or(&[]);
        let participation = baseline_participation(
            sizing.align_self,
            block_auto_margins,
            synthesized_baseline_would_cycle(sizing.align_self, baselines, row_span_tracks),
            baselines,
        );
        row_contributions.push(RowIntrinsicContribution {
            start: area.row,
            end: area.row_end,
            contributes_to_row_size: true,
            contribution_kind: IntrinsicSpanContribution::MaxContent,
            contribution: output.size.height + margin.vertical_sum(),
            participation,
            geometry: baseline_geometry_for_intrinsic_contribution(output, margin),
        });
    }

    let row_baseline_groups =
        row_baseline_groups_for_intrinsic_contributions(&row_contributions, row_count);
    let resolver = tree.calc_resolver();
    for item in row_contributions {
        if !item.contributes_to_row_size {
            continue;
        }
        let shim = if grid
            .row_tracks
            .get(item.start..item.end)
            .is_some_and(|tracks| {
                tracks
                    .iter()
                    .any(|track| track_accepts_intrinsic_contribution(*track))
            }) {
            row_baseline_shim(item, &row_baseline_groups)
        } else {
            BaselineShim::default()
        };
        let contribution = item.contribution + shim.before + shim.after;
        if item.end == item.start + 1 {
            rows[item.start] = rows[item.start].max(contribution);
        } else {
            distribute_intrinsic_span(
                &mut rows[item.start..item.end],
                &grid.row_tracks[item.start..item.end],
                item.contribution_kind,
                grid.percent_basis.height,
                span_contribution(contribution, item.end - item.start, gap.height),
                resolver,
            );
        }
    }

    rows
}

pub(super) fn constrained_column_intrinsic_sizes<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    grid: IntrinsicGrid<'_, <Tree as Traverse>::Node>,
    columns: &[Scalar],
    rows: &[Scalar],
    gap: Size,
) -> Vec<Scalar>
where
    Tree: Compute,
{
    let column_count = grid.column_tracks.len();
    let mut column_sizes: Vec<Scalar> = vec![0.0; column_count];
    if column_count == 0 || rows.is_empty() {
        return column_sizes;
    }

    let children = tree.children(node).collect::<Vec<_>>();
    let placed_areas = resolve_grid_child_areas(ResolveGridChildAreasInput {
        children: &children,
        placements: grid.placements,
        style: grid.style,
        columns,
        rows,
        gap,
        lines: grid.lines,
    });

    for (child, area) in children.into_iter().zip(placed_areas) {
        let child_style = tree.node_input(child).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }

        let Some(area) = area else {
            continue;
        };
        if area.column >= column_count || area.row >= rows.len() {
            continue;
        }
        if area.column_end != area.column + 1 {
            continue;
        }
        if scroll_container_auto_minimum_zero_inline(&child_style) {
            continue;
        }
        if !child_style.writing_mode.is_vertical() {
            continue;
        }

        let sizing = grid_item_sizing(
            &child_style,
            grid.style,
            area.size,
            Size::splat(Some(area.size.width)),
            tree.calc_resolver(),
        );
        let margin = sizing.unresolved_margin.map(|margin| margin.unwrap_or(0.0));
        let output = tree.compute_child(
            child,
            ComputeInput {
                run_mode: RunMode::ComputeSize,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::new(None, sizing.known.height),
                parent: Size::new(Some(area.size.width), Some(area.size.height)),
                available: Size::new(
                    Available::MIN_CONTENT,
                    Available::definite(sizing.available.height),
                ),
            },
        );
        column_sizes[area.column] =
            column_sizes[area.column].max(output.size.width + margin.horizontal_sum());
    }

    column_sizes
}

#[derive(Clone, Copy)]
pub(super) struct PercentTrackContent<'a, Node> {
    pub(super) style: &'a NodeInput,
    pub(super) constants: &'a Constants,
    pub(super) parent_context: &'a GridParentContext,
    pub(super) column_tracks: &'a [TrackSizing],
    pub(super) row_tracks: &'a [TrackSizing],
    pub(super) columns: &'a [Scalar],
    pub(super) rows: &'a [Scalar],
    pub(super) gap: Size,
    pub(super) lines: GridLines,
    pub(super) placements: &'a GridPlacementContext<Node>,
}

pub(super) fn cyclic_percent_track_content_size<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: PercentTrackContent<'_, <Tree as Traverse>::Node>,
) -> Size
where
    Tree: Compute,
{
    let PercentTrackContent {
        style,
        constants,
        parent_context,
        column_tracks,
        row_tracks,
        columns,
        rows,
        gap,
        lines,
        placements,
    } = input;

    if constants.node_inner_size.width.is_some() && constants.node_inner_size.height.is_some() {
        return Size::ZERO;
    }

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
    let column_offsets = offsets(columns, 0.0, gap.width);
    let row_offsets = offsets(rows, 0.0, gap.height);
    let mut content_size = Size::ZERO;
    let accumulate_standalone_percent_columns =
        inherits_opposite_subgrid_axis(parent_context, GridAxisKind::Column);
    let accumulate_standalone_percent_rows =
        inherits_opposite_subgrid_axis(parent_context, GridAxisKind::Row);
    let mut column_content: Vec<Scalar> = vec![0.0; columns.len()];
    let mut row_content: Vec<Scalar> = vec![0.0; rows.len()];
    for (child, area) in children.into_iter().zip(placed_areas) {
        let child_style = tree.node_input(child).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }
        let Some(area) = area else {
            continue;
        };
        if area.row >= rows.len() || area.column >= columns.len() {
            continue;
        }

        let column_span = &column_tracks[area.column..area.column_end.min(column_tracks.len())];
        let row_span = &row_tracks[area.row..area.row_end.min(row_tracks.len())];
        let spans_percent_column = constants.node_inner_size.width.is_none()
            && {
                let resolver = tree.calc_resolver();
                column_span
                    .iter()
                    .any(|track| track_has_percent_sizing(track, resolver))
            }
            && !column_span
                .iter()
                .any(|track| track_accepts_intrinsic_contribution(*track));
        let spans_percent_row = constants.node_inner_size.height.is_none()
            && {
                let resolver = tree.calc_resolver();
                row_span
                    .iter()
                    .any(|track| track_has_percent_sizing(track, resolver))
            }
            && !row_span
                .iter()
                .any(|track| track_accepts_intrinsic_contribution(*track));
        if !spans_percent_column && !spans_percent_row {
            continue;
        }

        let output = tree.compute_child(
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
                available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            },
        );
        let location = Point::new(column_offsets[area.column], row_offsets[area.row]);
        let contribution = content_size_contribution(
            location,
            output.size,
            output.content_size,
            child_style.overflow,
        );
        content_size = max_size(content_size, contribution);
        if spans_percent_column {
            let contribution = axis_content_contribution(
                location.x,
                output.size.width,
                output.content_size.width,
                child_style.overflow.x,
            );
            content_size.width = content_size.width.max(contribution);
            if accumulate_standalone_percent_columns
                && area.column_end == area.column + 1
                && let Some(size) = column_content.get_mut(area.column)
            {
                *size = (*size).max(contribution);
            }
        }
        if spans_percent_row {
            let contribution = axis_content_contribution(
                location.y,
                output.size.height,
                output.content_size.height,
                child_style.overflow.y,
            );
            content_size.height = content_size.height.max(contribution);
            if accumulate_standalone_percent_rows
                && area.row_end == area.row + 1
                && let Some(size) = row_content.get_mut(area.row)
            {
                *size = (*size).max(contribution);
            }
        }
    }

    if accumulate_standalone_percent_columns {
        content_size.width = content_size
            .width
            .max(track_sum(&column_content, gap.width));
    }
    if accumulate_standalone_percent_rows {
        content_size.height = content_size.height.max(track_sum(&row_content, gap.height));
    }

    content_size
}

fn inherits_opposite_subgrid_axis(parent_context: &GridParentContext, axis: GridAxisKind) -> bool {
    // Additive standalone percent sizing is only for grids that actually inherit
    // the opposite subgrid axis; raw fallback `subgrid` declarations stay ordinary grids.
    match axis {
        GridAxisKind::Column => parent_context.rows.is_some(),
        GridAxisKind::Row => parent_context.columns.is_some(),
    }
}

fn axis_content_contribution(
    location: Scalar,
    size: Scalar,
    content_size: Scalar,
    overflow: Overflow,
) -> Scalar {
    let contribution_size = if overflow == Overflow::Visible {
        size.max(content_size)
    } else {
        size
    };
    if contribution_size <= 0.0 {
        return 0.0;
    }
    let max = (location + contribution_size).max(0.0);
    let min = location.min(0.0);
    max - min
}

pub(super) fn track_has_percent_sizing(track: &TrackSizing, resolver: &dyn CalcResolver) -> bool {
    track.depends_on_basis_with(resolver)
}

pub(super) fn scroll_container_auto_minimum_zero_inline(style: &NodeInput) -> bool {
    style.overflow.x.is_scrollable() && style.size.width.is_auto()
}

pub(super) fn scroll_container_auto_minimum_zero_block(style: &NodeInput) -> bool {
    style.overflow.y.is_scrollable() && style.size.height.is_auto()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IntrinsicSpanContribution {
    MinContent { prioritize_min_tracks: bool },
    MaxContent,
}

impl IntrinsicSpanContribution {
    const fn for_axis(available: Available, overflow: Overflow) -> Self {
        match available {
            Available::MaxContent | Available::Definite(_) => Self::MaxContent,
            Available::MinContent => Self::MinContent {
                prioritize_min_tracks: overflow.clips_contents(),
            },
        }
    }
}

pub(super) fn distribute_intrinsic_span(
    sizes: &mut [Scalar],
    tracks: &[TrackSizing],
    kind: IntrinsicSpanContribution,
    percent_basis: Option<Scalar>,
    contribution: Scalar,
    resolver: &dyn CalcResolver,
) {
    let flex_indexes = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| track_flex_factor(*track).is_some().then_some(index))
        .collect::<Vec<_>>();
    if !flex_indexes.is_empty() {
        let contribution =
            contribution - percent_basis.unwrap_or(0.0) * track_percent_sum(tracks, resolver);
        let current =
            sizes.iter().sum::<Scalar>() + intrinsic_span_definite_track_space(tracks, resolver);
        let extra = (contribution - current).max(0.0);
        if extra == 0.0 {
            return;
        }

        let flex_sum = flex_indexes
            .iter()
            .map(|index| track_flex_factor(tracks[*index]).unwrap_or(0.0))
            .sum::<Scalar>();
        for index in flex_indexes.iter().copied() {
            let share = if flex_sum > 0.0 {
                extra * track_flex_factor(tracks[index]).unwrap_or(0.0) / flex_sum
            } else {
                extra / flex_indexes.len() as Scalar
            };
            sizes[index] += share;
        }
        return;
    }

    let auto_indexes = intrinsic_span_distribution_indexes(tracks, kind);
    if auto_indexes.is_empty() {
        return;
    }

    let contribution = if kind == IntrinsicSpanContribution::MaxContent {
        contribution - percent_basis.unwrap_or(0.0) * track_percent_sum(tracks, resolver)
    } else {
        intrinsic_span_non_percent_contribution(tracks, contribution, resolver)
    };
    let current =
        sizes.iter().sum::<Scalar>() + intrinsic_span_definite_space(tracks, kind, resolver);
    let extra = (contribution - current).max(0.0);
    if extra == 0.0 {
        return;
    }

    let divisor = intrinsic_span_distribution_count(tracks, kind, auto_indexes.len(), resolver);
    distribute_intrinsic_extra(sizes, &auto_indexes, extra, divisor);
}

pub(super) fn distribute_intrinsic_extra(
    sizes: &mut [Scalar],
    indexes: &[usize],
    extra: Scalar,
    divisor: usize,
) {
    if indexes.is_empty() || extra <= 0.0 {
        return;
    }
    if divisor > indexes.len() {
        let share = extra / divisor as Scalar;
        for index in indexes {
            sizes[*index] += share;
        }
        return;
    }

    let mut sorted = indexes.to_vec();
    sorted.sort_by(|left, right| sizes[*left].total_cmp(&sizes[*right]));
    let mut remaining = extra;
    let mut active_count = 1;
    while active_count < sorted.len() {
        let current = sizes[sorted[active_count - 1]];
        let next = sizes[sorted[active_count]];
        let needed = (next - current).max(0.0) * active_count as Scalar;
        if needed > 0.0 && remaining <= needed {
            let share = remaining / active_count as Scalar;
            for index in &sorted[..active_count] {
                sizes[*index] += share;
            }
            return;
        }
        for index in &sorted[..active_count] {
            sizes[*index] += next - current;
        }
        remaining -= needed;
        active_count += 1;
    }

    let share = remaining / active_count as Scalar;
    for index in &sorted[..active_count] {
        sizes[*index] += share;
    }
}

pub(super) fn distribute_min_content_span_with_percent(
    sizes: &mut [Scalar],
    tracks: &[TrackSizing],
    overflow: Overflow,
    percent_basis: Option<Scalar>,
    min_content_contribution: Scalar,
    resolver: &dyn CalcResolver,
) {
    let fixed_space = intrinsic_span_minimum_floor_space(tracks, resolver);
    let percent_space = percent_basis.unwrap_or(0.0) * track_percent_sum(tracks, resolver);
    let extra = (min_content_contribution - fixed_space - percent_space).max(0.0);
    let indexes = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            let accepts =
                track_accepts_percent_min_content_span(*track, overflow, percent_basis, resolver);
            accepts.then_some(index)
        })
        .collect::<Vec<_>>();
    distribute_intrinsic_extra(sizes, &indexes, extra, indexes.len());
}

pub(super) fn intrinsic_span_distribution_indexes(
    tracks: &[TrackSizing],
    kind: IntrinsicSpanContribution,
) -> Vec<usize> {
    if let IntrinsicSpanContribution::MinContent {
        prioritize_min_tracks: true,
    } = kind
    {
        let min_content_indexes = tracks
            .iter()
            .enumerate()
            .filter_map(|(index, track)| {
                track_accepts_min_content_span_priority(*track).then_some(index)
            })
            .collect::<Vec<_>>();
        if !min_content_indexes.is_empty() {
            return min_content_indexes;
        }
    }

    if kind == IntrinsicSpanContribution::MaxContent {
        let max_content_indexes = tracks
            .iter()
            .enumerate()
            .filter_map(|(index, track)| {
                track_accepts_max_content_span_priority(*track).then_some(index)
            })
            .collect::<Vec<_>>();
        if !max_content_indexes.is_empty() {
            return max_content_indexes;
        }

        let auto_indexes = tracks
            .iter()
            .enumerate()
            .filter_map(|(index, track)| track_accepts_auto_span_priority(*track).then_some(index))
            .collect::<Vec<_>>();
        if !auto_indexes.is_empty() {
            return auto_indexes;
        }
    }

    tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| track_accepts_intrinsic_contribution(*track).then_some(index))
        .collect::<Vec<_>>()
}

pub(super) fn intrinsic_span_non_percent_contribution(
    tracks: &[TrackSizing],
    contribution: Scalar,
    resolver: &dyn CalcResolver,
) -> Scalar {
    contribution * (1.0 - track_percent_sum(tracks, resolver)).clamp(0.0, 1.0)
}

pub(super) fn track_percent_sum(tracks: &[TrackSizing], resolver: &dyn CalcResolver) -> Scalar {
    tracks
        .iter()
        .map(|track| track_percent_fraction(track, resolver))
        .sum::<Scalar>()
}

pub(super) fn intrinsic_span_distribution_count(
    tracks: &[TrackSizing],
    kind: IntrinsicSpanContribution,
    distribution_count: usize,
    resolver: &dyn CalcResolver,
) -> usize {
    if kind
        == (IntrinsicSpanContribution::MinContent {
            prioritize_min_tracks: false,
        })
    {
        let count = tracks
            .iter()
            .filter(|track| {
                track_accepts_intrinsic_contribution(**track)
                    || track_percent_fraction(track, resolver) > 0.0
            })
            .count();
        return count.max(distribution_count).max(1);
    }

    distribution_count.max(1)
}

pub(super) fn intrinsic_span_definite_space(
    tracks: &[TrackSizing],
    kind: IntrinsicSpanContribution,
    resolver: &dyn CalcResolver,
) -> Scalar {
    if kind != IntrinsicSpanContribution::MaxContent {
        return 0.0;
    }

    tracks
        .iter()
        .filter(|track| {
            track_percent_fraction(track, resolver) == 0.0 && track_flex_factor(**track).is_none()
        })
        .map(|track| track_min_floor_space(*track, resolver))
        .sum()
}

pub(super) fn intrinsic_span_definite_track_space(
    tracks: &[TrackSizing],
    resolver: &dyn CalcResolver,
) -> Scalar {
    tracks
        .iter()
        .filter(|track| {
            !track_accepts_intrinsic_contribution(**track)
                && track_flex_factor(**track).is_none()
                && track_percent_fraction(track, resolver) == 0.0
        })
        .map(|track| track_base_size(*track, None, 0.0, resolver))
        .sum()
}

pub(super) fn intrinsic_span_minimum_floor_space(
    tracks: &[TrackSizing],
    resolver: &dyn CalcResolver,
) -> Scalar {
    tracks
        .iter()
        .filter(|track| {
            track_percent_fraction(track, resolver) == 0.0 && track_flex_factor(**track).is_none()
        })
        .map(|track| track_min_floor_space(*track, resolver))
        .sum()
}

pub(super) fn track_min_floor_space(track: TrackSizing, resolver: &dyn CalcResolver) -> Scalar {
    track
        .min
        .percent_fraction_with(resolver)
        .eq(&0.0)
        .then(|| match track.min {
            MinTrackSizing::Length(length) => length.resolve_with(None, resolver),
            MinTrackSizing::Auto | MinTrackSizing::MinContent | MinTrackSizing::MaxContent => None,
        })
        .flatten()
        .or_else(|| {
            (!track_accepts_intrinsic_contribution(track))
                .then(|| track_base_size(track, None, 0.0, resolver))
        })
        .unwrap_or(0.0)
}

pub(super) fn track_percent_fraction(track: &TrackSizing, resolver: &dyn CalcResolver) -> Scalar {
    track.percent_fraction_with(resolver)
}

pub(super) fn span_contribution(contribution: Scalar, span: usize, gap: Scalar) -> Scalar {
    (contribution - gap * span.saturating_sub(1) as Scalar).max(0.0)
}

pub(super) fn track_accepts_intrinsic_contribution(track: TrackSizing) -> bool {
    track.min.is_intrinsic() || track.max.is_intrinsic()
}

pub(super) fn track_has_definite_min_floor(
    track: TrackSizing,
    resolver: &dyn CalcResolver,
) -> bool {
    match track.min {
        MinTrackSizing::Length(length) => length.resolve_with(None, resolver).is_some(),
        MinTrackSizing::Auto | MinTrackSizing::MinContent | MinTrackSizing::MaxContent => false,
    }
}

pub(super) fn track_accepts_min_content_span_priority(track: TrackSizing) -> bool {
    matches!(track.min, MinTrackSizing::MinContent)
        || matches!(track.max, MaxTrackSizing::MinContent)
}

pub(super) fn track_accepts_max_content_span_priority(track: TrackSizing) -> bool {
    (matches!(track.min, MinTrackSizing::MaxContent)
        && !matches!(track.max, MaxTrackSizing::MinContent))
        || matches!(
            track,
            TrackSizing {
                min: MinTrackSizing::Auto,
                max: MaxTrackSizing::MaxContent
            }
        )
}

pub(super) fn track_accepts_auto_span_priority(track: TrackSizing) -> bool {
    matches!(track.min, MinTrackSizing::Auto) || matches!(track.max, MaxTrackSizing::Auto)
}

pub(super) fn track_accepts_percent_min_content_span(
    track: TrackSizing,
    overflow: Overflow,
    percent_basis: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> bool {
    if percent_basis.is_none() && track_percent_fraction(&track, resolver) > 0.0 {
        return true;
    }
    if track_has_definite_min_floor(track, resolver) {
        return false;
    }
    if overflow.clips_contents() {
        track_accepts_min_content_span_priority(track)
            || track_accepts_max_content_span_priority(track)
    } else {
        track_accepts_intrinsic_contribution(track)
    }
}

pub(super) fn intrinsic_contribution_margin(
    style: &NodeInput,
    inner_node_width: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> Edges {
    Edges {
        top: resolve_auto_or_zero_with(style.margin.top, inner_node_width, resolver),
        right: resolve_auto_or_zero_with(style.margin.right, Some(0.0), resolver),
        bottom: resolve_auto_or_zero_with(style.margin.bottom, inner_node_width, resolver),
        left: resolve_auto_or_zero_with(style.margin.left, Some(0.0), resolver),
    }
}

pub(super) fn resolve_tracks(
    tracks: &[TrackSizing],
    basis: Option<Scalar>,
    gap: Scalar,
    alignment: AlignContent,
    intrinsic_sizes: &[Scalar],
    resolver: &dyn CalcResolver,
) -> Vec<Scalar> {
    let gap_total = gap * tracks.len().saturating_sub(1) as Scalar;
    let base_sizes = tracks
        .iter()
        .enumerate()
        .map(|(index, track)| match track.max {
            MaxTrackSizing::Flex(_) => track_min_size(
                track.min,
                basis,
                intrinsic_at(intrinsic_sizes, index),
                resolver,
            ),
            _ => track_base_size(
                *track,
                basis,
                intrinsic_at(intrinsic_sizes, index),
                resolver,
            ),
        })
        .collect::<Vec<_>>();
    let auto_count = tracks
        .iter()
        .filter(|track| {
            matches!(
                track,
                TrackSizing {
                    min: MinTrackSizing::Auto,
                    max: MaxTrackSizing::Auto
                }
            )
        })
        .count();
    let fr_size = resolve_flex_fraction(tracks, &base_sizes, basis.map(|basis| basis - gap_total));
    let flex_used = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            track_flex_factor(*track).map(|factor| base_sizes[index].max(factor * fr_size))
        })
        .sum::<Scalar>();
    let fixed_sum = tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| track_flex_factor(**track).is_none())
        .map(|(index, _)| base_sizes[index])
        .sum::<Scalar>();
    let auto_size = if alignment == AlignContent::Stretch && auto_count > 0 {
        basis
            .map(|basis| {
                ((basis - gap_total - fixed_sum - flex_used).max(0.0)) / auto_count as Scalar
            })
            .unwrap_or(0.0)
    } else {
        0.0
    };

    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| match track {
            TrackSizing {
                max: MaxTrackSizing::Flex(value),
                ..
            } => base_sizes[index].max(value * fr_size),
            TrackSizing {
                min: MinTrackSizing::Auto,
                max: MaxTrackSizing::Auto,
            } => intrinsic_at(intrinsic_sizes, index) + auto_size,
            track => {
                let intrinsic = intrinsic_at(intrinsic_sizes, index);
                let base = base_sizes[index];
                let min = track_growth_floor(*track, basis, intrinsic, resolver);
                track_growth_limit(*track, basis, intrinsic, resolver)
                    .map(|limit| base.min(limit.max(min)))
                    .unwrap_or(base)
            }
        })
        .collect()
}

pub(super) fn track_growth_floor(
    track: TrackSizing,
    basis: Option<Scalar>,
    intrinsic: Scalar,
    resolver: &dyn CalcResolver,
) -> Scalar {
    match track.min {
        MinTrackSizing::Auto => 0.0,
        min => track_min_size(min, basis, intrinsic, resolver),
    }
}

pub(super) fn resolve_flex_fraction(
    tracks: &[TrackSizing],
    base_sizes: &[Scalar],
    space_to_fill: Option<Scalar>,
) -> Scalar {
    if !tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizing::Flex(_)))
    {
        return 0.0;
    }

    if let Some(space_to_fill) = space_to_fill {
        return find_size_of_fr(tracks, base_sizes, space_to_fill.max(0.0));
    }

    let flex_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            track_flex_factor(*track).map(|factor| {
                if factor > 1.0 {
                    base_sizes[index] / factor
                } else {
                    base_sizes[index]
                }
            })
        })
        .fold(0.0, Scalar::max);
    let occupied_sub_one_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            let factor = track_flex_factor(*track)?;
            (base_sizes.get(index).copied().unwrap_or(0.0) > 0.0 && factor < 1.0).then_some(factor)
        })
        .sum::<Scalar>();

    if occupied_sub_one_fraction > 0.0 && occupied_sub_one_fraction < 1.0 {
        flex_fraction * occupied_sub_one_fraction
    } else {
        flex_fraction
    }
}

pub(super) fn find_size_of_fr(
    tracks: &[TrackSizing],
    base_sizes: &[Scalar],
    space_to_fill: Scalar,
) -> Scalar {
    if space_to_fill <= 0.0 {
        return 0.0;
    }

    let mut hypothetical = Scalar::INFINITY;
    loop {
        let previous = hypothetical;
        let mut used_space = 0.0;
        let mut flex_sum = 0.0;
        for (index, track) in tracks.iter().enumerate() {
            if let Some(factor) = track_flex_factor(*track)
                && factor * hypothetical >= base_sizes[index]
            {
                flex_sum += factor;
            } else {
                used_space += base_sizes[index];
            }
        }

        hypothetical = (space_to_fill - used_space) / flex_sum.max(1.0);
        let valid = tracks.iter().enumerate().all(|(index, track)| {
            if let Some(factor) = track_flex_factor(*track) {
                factor * hypothetical >= base_sizes[index] || factor * previous < base_sizes[index]
            } else {
                true
            }
        });
        if valid {
            return hypothetical.max(0.0);
        }
    }
}

pub(super) fn track_flex_factor(track: TrackSizing) -> Option<Scalar> {
    if let MaxTrackSizing::Flex(value) = track.max {
        Some(value)
    } else {
        None
    }
}

pub(super) fn resolve_inline_tracks(input: InlineTrackInput<'_>) -> Vec<Scalar> {
    let InlineTrackInput {
        resolver,
        tracks,
        basis,
        definite_size,
        available_size,
        gap,
        alignment,
        stretch_empty_auto_to_available,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
    } = input;

    let max_tracks = resolve_tracks_with_intrinsics(
        tracks,
        basis,
        gap,
        AlignContent::Start,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        resolver,
    );
    let min_tracks = resolve_track_min_bounds(
        tracks,
        basis,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        resolver,
    );
    let max_content = track_sum(&max_tracks, gap);
    let min_content = track_sum(&min_tracks, gap);

    if let Some(available_size) = definite_size.or(available_size)
        && max_content > 0.0
        && available_size < max_content
    {
        let target = available_size.clamp(min_content, max_content);
        return distribute_tracks_between_bounds(&min_tracks, &max_tracks, gap, target);
    }

    if tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizing::FitContent(_)))
    {
        return resolve_fit_content_tracks(
            tracks,
            basis.or(available_size),
            min_intrinsic_sizes,
            max_intrinsic_sizes,
            resolver,
        );
    }

    resolve_tracks_with_intrinsics(
        tracks,
        basis.or_else(|| {
            stretch_empty_auto_track_basis(
                tracks,
                available_size,
                alignment,
                stretch_empty_auto_to_available,
                max_intrinsic_sizes,
            )
        }),
        gap,
        alignment,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        resolver,
    )
}

fn stretch_empty_auto_track_basis(
    tracks: &[TrackSizing],
    available_size: Option<Scalar>,
    alignment: AlignContent,
    enabled: bool,
    max_intrinsic_sizes: &[Scalar],
) -> Option<Scalar> {
    if !enabled || alignment != AlignContent::Stretch {
        return None;
    }

    let has_empty_auto_track = tracks.iter().enumerate().any(|(index, track)| {
        matches!(
            track,
            TrackSizing {
                min: MinTrackSizing::Auto,
                max: MaxTrackSizing::Auto
            }
        ) && intrinsic_at(max_intrinsic_sizes, index) == 0.0
    });
    let has_non_auto_track = tracks.iter().any(|track| {
        !matches!(
            track,
            TrackSizing {
                min: MinTrackSizing::Auto,
                max: MaxTrackSizing::Auto
            }
        )
    });

    (has_empty_auto_track && has_non_auto_track)
        .then_some(available_size)
        .flatten()
}

pub(super) fn resolve_track_min_bounds(
    tracks: &[TrackSizing],
    basis: Option<Scalar>,
    min_intrinsic_sizes: &[Scalar],
    max_intrinsic_sizes: &[Scalar],
    resolver: &dyn CalcResolver,
) -> Vec<Scalar> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let intrinsic = match track.min {
                MinTrackSizing::MaxContent => intrinsic_at(max_intrinsic_sizes, index),
                _ => intrinsic_at(min_intrinsic_sizes, index),
            };
            track_min_size(track.min, basis, intrinsic, resolver)
        })
        .collect()
}

pub(super) fn resolve_tracks_with_intrinsics(
    tracks: &[TrackSizing],
    basis: Option<Scalar>,
    gap: Scalar,
    alignment: AlignContent,
    min_intrinsic_sizes: &[Scalar],
    max_intrinsic_sizes: &[Scalar],
    resolver: &dyn CalcResolver,
) -> Vec<Scalar> {
    let gap_total = gap * tracks.len().saturating_sub(1) as Scalar;
    let base_sizes = tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let min_intrinsic = intrinsic_at(min_intrinsic_sizes, index);
            let max_intrinsic = intrinsic_at(max_intrinsic_sizes, index);
            match track.max {
                MaxTrackSizing::Flex(_) => max_intrinsic.max(track_min_size_for_intrinsics(
                    track.min,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                    resolver,
                )),
                _ => track_base_size_for_intrinsics(
                    *track,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                    resolver,
                ),
            }
        })
        .collect::<Vec<_>>();
    let auto_count = tracks
        .iter()
        .filter(|track| {
            matches!(
                track,
                TrackSizing {
                    min: MinTrackSizing::Auto,
                    max: MaxTrackSizing::Auto
                }
            )
        })
        .count();
    let fr_size = resolve_flex_fraction(tracks, &base_sizes, basis.map(|basis| basis - gap_total));
    let flex_used = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            track_flex_factor(*track).map(|factor| base_sizes[index].max(factor * fr_size))
        })
        .sum::<Scalar>();
    let fixed_sum = tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| track_flex_factor(**track).is_none())
        .map(|(index, _)| base_sizes[index])
        .sum::<Scalar>();
    let auto_size = if alignment == AlignContent::Stretch && auto_count > 0 {
        basis
            .map(|basis| {
                ((basis - gap_total - fixed_sum - flex_used).max(0.0)) / auto_count as Scalar
            })
            .unwrap_or(0.0)
    } else {
        0.0
    };

    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let min_intrinsic = intrinsic_at(min_intrinsic_sizes, index);
            let max_intrinsic = intrinsic_at(max_intrinsic_sizes, index);
            match track {
                TrackSizing {
                    max: MaxTrackSizing::Flex(value),
                    ..
                } => base_sizes[index].max(value * fr_size),
                TrackSizing {
                    min: MinTrackSizing::Auto,
                    max: MaxTrackSizing::Auto,
                } => max_intrinsic + auto_size,
                track => {
                    let base = base_sizes[index];
                    let min = track_growth_floor_for_intrinsics(
                        *track,
                        basis,
                        min_intrinsic,
                        max_intrinsic,
                        resolver,
                    );
                    track_growth_limit_for_intrinsics(
                        *track,
                        basis,
                        min_intrinsic,
                        max_intrinsic,
                        resolver,
                    )
                    .map(|limit| base.min(limit.max(min)))
                    .unwrap_or(base)
                }
            }
        })
        .collect()
}

pub(super) fn track_base_size_for_intrinsics(
    track: TrackSizing,
    basis: Option<Scalar>,
    min_intrinsic: Scalar,
    max_intrinsic: Scalar,
    resolver: &dyn CalcResolver,
) -> Scalar {
    let min =
        track_min_size_for_intrinsics(track.min, basis, min_intrinsic, max_intrinsic, resolver);
    let max_base = match track.max {
        MaxTrackSizing::Length(length) => {
            length
                .resolve_with(basis, resolver)
                .unwrap_or_else(|| match length {
                    length
                        if length.depends_on_basis_with(resolver) || length.requires_resolver() =>
                    {
                        max_intrinsic
                    }
                    Length::Normal => 0.0,
                    Length::Px(_) => length.resolve_with(None, resolver).unwrap_or(0.0),
                    _ => unreachable!(
                        "basis-dependent and resolver-required lengths are handled above"
                    ),
                })
        }
        MaxTrackSizing::Flex(_) => 0.0,
        MaxTrackSizing::Auto | MaxTrackSizing::MaxContent => max_intrinsic,
        MaxTrackSizing::MinContent => min_intrinsic,
        MaxTrackSizing::FitContent(limit) => {
            let limit = limit.resolve_with(basis, resolver).unwrap_or(max_intrinsic);
            max_intrinsic.min(limit)
        }
    };
    min.max(max_base)
}

pub(super) fn track_min_size_for_intrinsics(
    min: MinTrackSizing,
    basis: Option<Scalar>,
    min_intrinsic: Scalar,
    max_intrinsic: Scalar,
    resolver: &dyn CalcResolver,
) -> Scalar {
    match min {
        MinTrackSizing::Length(length) => length.resolve_with(basis, resolver).unwrap_or(0.0),
        MinTrackSizing::Auto | MinTrackSizing::MaxContent => max_intrinsic,
        MinTrackSizing::MinContent => min_intrinsic,
    }
}

pub(super) fn track_growth_floor_for_intrinsics(
    track: TrackSizing,
    basis: Option<Scalar>,
    min_intrinsic: Scalar,
    max_intrinsic: Scalar,
    resolver: &dyn CalcResolver,
) -> Scalar {
    match track.min {
        MinTrackSizing::Auto => 0.0,
        min => track_min_size_for_intrinsics(min, basis, min_intrinsic, max_intrinsic, resolver),
    }
}

pub(super) fn track_growth_limit_for_intrinsics(
    track: TrackSizing,
    basis: Option<Scalar>,
    min_intrinsic: Scalar,
    max_intrinsic: Scalar,
    resolver: &dyn CalcResolver,
) -> Option<Scalar> {
    match track.max {
        MaxTrackSizing::Length(length) | MaxTrackSizing::FitContent(length) => {
            length.resolve_with(basis, resolver).or(match length {
                length if length.depends_on_basis_with(resolver) || length.requires_resolver() => {
                    Some(max_intrinsic)
                }
                Length::Normal => Some(0.0),
                Length::Px(_) => None,
                _ => {
                    unreachable!("basis-dependent and resolver-required lengths are handled above")
                }
            })
        }
        MaxTrackSizing::MinContent => Some(min_intrinsic),
        MaxTrackSizing::MaxContent | MaxTrackSizing::Auto => Some(max_intrinsic),
        MaxTrackSizing::Flex(_) => None,
    }
}

pub(super) fn resolve_fit_content_tracks(
    tracks: &[TrackSizing],
    basis: Option<Scalar>,
    min_intrinsic_sizes: &[Scalar],
    max_intrinsic_sizes: &[Scalar],
    resolver: &dyn CalcResolver,
) -> Vec<Scalar> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| match track.max {
            MaxTrackSizing::FitContent(limit) => {
                let min_content = intrinsic_at(min_intrinsic_sizes, index);
                let max_content = intrinsic_at(max_intrinsic_sizes, index);
                let limit = limit.resolve_with(basis, resolver).unwrap_or(max_content);
                max_content.min(min_content.max(limit))
            }
            _ => track_base_size_for_intrinsics(
                *track,
                basis,
                intrinsic_at(min_intrinsic_sizes, index),
                intrinsic_at(max_intrinsic_sizes, index),
                resolver,
            ),
        })
        .collect()
}

pub(super) fn distribute_tracks_between_bounds(
    min_tracks: &[Scalar],
    max_tracks: &[Scalar],
    gap: Scalar,
    target: Scalar,
) -> Vec<Scalar> {
    let min_sum = track_sum(min_tracks, gap);
    let max_sum = track_sum(max_tracks, gap);
    if target <= min_sum {
        return min_tracks.to_vec();
    }
    if target >= max_sum {
        return max_tracks.to_vec();
    }

    let mut resolved = max_tracks.to_vec();
    let shrink = (max_sum - target).max(0.0);
    let shrink_capacity = max_tracks
        .iter()
        .zip(min_tracks)
        .map(|(max, min)| (max - min).max(0.0))
        .sum::<Scalar>();
    if shrink_capacity == 0.0 {
        return resolved;
    }

    let ratio = (shrink / shrink_capacity).min(1.0);
    for (index, resolved) in resolved.iter_mut().enumerate() {
        let capacity = (max_tracks[index] - min_tracks[index]).max(0.0);
        *resolved -= capacity * ratio;
    }
    resolved
}

pub(super) fn extend_auto_tracks(
    tracks: &mut Vec<TrackSizing>,
    auto_tracks: &[TrackComponent],
    basis: Option<Scalar>,
    gap: Scalar,
    required_count: usize,
    resolver: &dyn CalcResolver,
) {
    let auto_tracks = expand_track_components(auto_tracks, basis, gap, None, resolver);
    let mut index = 0;
    while tracks.len() < required_count {
        let track = if auto_tracks.is_empty() {
            TrackSizing::AUTO
        } else {
            auto_tracks[index]
        };
        tracks.push(track);
        if !auto_tracks.is_empty() {
            index = (index + 1) % auto_tracks.len();
        }
    }
}

pub(super) fn prepend_auto_tracks(
    tracks: &mut Vec<TrackSizing>,
    auto_tracks: &[TrackComponent],
    basis: Option<Scalar>,
    gap: Scalar,
    required_count: usize,
    auto_fit_limit: Option<usize>,
    resolver: &dyn CalcResolver,
) {
    if required_count == 0 {
        return;
    }

    let auto_tracks = expand_track_components(auto_tracks, basis, gap, auto_fit_limit, resolver);
    let generated = if auto_tracks.is_empty() {
        vec![TrackSizing::AUTO; required_count]
    } else {
        (0..required_count)
            .map(|index| {
                let phase = (auto_tracks.len() + index + auto_tracks.len()
                    - required_count % auto_tracks.len())
                    % auto_tracks.len();
                auto_tracks[phase]
            })
            .collect::<Vec<_>>()
    };
    tracks.splice(0..0, generated);
}

pub(super) fn expand_track_components(
    components: &[TrackComponent],
    basis: Option<Scalar>,
    gap: Scalar,
    auto_fit_limit: Option<usize>,
    resolver: &dyn CalcResolver,
) -> Vec<TrackSizing> {
    if subgrid_components(components) {
        return Vec::new();
    }

    let mut tracks = Vec::new();
    let reserved = reserved_track_space(components, basis, gap, resolver);
    for component in components {
        match component {
            TrackComponent::Track(track) => tracks.push(*track),
            TrackComponent::Repeat(repetition) => {
                let repeated_tracks = repetition.sizing_tracks();
                let count = match repetition.repeat {
                    TrackRepeat::Count(count) => count,
                    TrackRepeat::AutoFill => {
                        auto_repeat_count(&repeated_tracks, basis, gap, reserved, resolver)
                    }
                    TrackRepeat::AutoFit => {
                        auto_repeat_count(&repeated_tracks, basis, gap, reserved, resolver)
                            .min(auto_fit_limit.unwrap_or(usize::MAX))
                            .max(1)
                    }
                };
                for _ in 0..count {
                    tracks.extend(repeated_tracks.iter().copied());
                }
            }
            TrackComponent::LineNames(_) => {}
            TrackComponent::Subgrid(_) => unreachable!("subgrid templates return before expansion"),
        }
    }
    tracks
}

pub(super) fn track_expansion_basis(
    components: &[TrackComponent],
    node_basis: Option<Scalar>,
    available_basis: Option<Scalar>,
) -> Option<Scalar> {
    if subgrid_components(components) {
        return None;
    }

    node_basis.or_else(|| {
        auto_repeat_components(components)
            .then_some(available_basis)
            .flatten()
    })
}

pub(super) fn subgrid_components(components: &[TrackComponent]) -> bool {
    components
        .iter()
        .any(|component| matches!(component, TrackComponent::Subgrid(_)))
}

pub(super) fn auto_repeat_components(components: &[TrackComponent]) -> bool {
    components.iter().any(|component| {
        matches!(
            component,
            TrackComponent::Repeat(repetition)
                if matches!(repetition.repeat, TrackRepeat::AutoFill | TrackRepeat::AutoFit)
        )
    })
}

pub(super) fn tracks_need_available_basis(tracks: &[TrackSizing]) -> bool {
    tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizing::Flex(_)))
}

#[derive(Clone, Copy)]
pub(super) struct ReservedTrackSpace {
    pub(super) count: usize,
    pub(super) size: Scalar,
}

pub(super) fn reserved_track_space(
    components: &[TrackComponent],
    basis: Option<Scalar>,
    gap: Scalar,
    resolver: &dyn CalcResolver,
) -> ReservedTrackSpace {
    let mut count = 0;
    let mut size = 0.0;
    for component in components {
        match component {
            TrackComponent::Track(track) => {
                count += 1;
                size += track_base_size(*track, basis, 0.0, resolver);
            }
            TrackComponent::Repeat(repetition) => {
                if let TrackRepeat::Count(repeat_count) = repetition.repeat {
                    let repeated_tracks = repetition.sizing_tracks();
                    count += repeat_count * repeated_tracks.len();
                    size += repeat_count as Scalar
                        * repeated_tracks
                            .iter()
                            .map(|track| track_base_size(*track, basis, 0.0, resolver))
                            .sum::<Scalar>();
                }
            }
            TrackComponent::LineNames(_) | TrackComponent::Subgrid(_) => {}
        }
    }

    if count > 1 {
        size += gap * (count - 1) as Scalar;
    }

    ReservedTrackSpace { count, size }
}

pub(super) fn auto_repeat_count(
    tracks: &[TrackSizing],
    basis: Option<Scalar>,
    gap: Scalar,
    reserved: ReservedTrackSpace,
    resolver: &dyn CalcResolver,
) -> usize {
    let Some(basis) = basis else {
        return 1;
    };
    if tracks.is_empty() {
        return 0;
    }
    let track_sum = tracks
        .iter()
        .map(|track| track_base_size(*track, Some(basis), 0.0, resolver).max(1.0))
        .sum::<Scalar>();
    let repeat_size = track_sum + gap * tracks.len() as Scalar;
    if repeat_size <= 0.0 {
        1
    } else {
        let available = if reserved.count == 0 {
            basis + gap
        } else {
            basis - reserved.size
        };
        (available / repeat_size).floor().max(1.0) as usize
    }
}

pub(super) fn intrinsic_at(intrinsic_sizes: &[Scalar], index: usize) -> Scalar {
    intrinsic_sizes.get(index).copied().unwrap_or(0.0)
}

pub(super) fn track_resolution_intrinsic_sizes(
    tracks: &[TrackSizing],
    min_intrinsic_sizes: &[Scalar],
    max_intrinsic_sizes: &[Scalar],
    resolver: &dyn CalcResolver,
) -> Vec<Scalar> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            if track.min == MinTrackSizing::MaxContent
                || match track.max {
                    MaxTrackSizing::Auto | MaxTrackSizing::Flex(_) | MaxTrackSizing::MaxContent => {
                        true
                    }
                    MaxTrackSizing::Length(length) => length.depends_on_basis_with(resolver),
                    MaxTrackSizing::FitContent(_) | MaxTrackSizing::MinContent => false,
                }
            {
                intrinsic_at(max_intrinsic_sizes, index)
            } else if track.min == MinTrackSizing::MinContent
                || track.max == MaxTrackSizing::MinContent
            {
                intrinsic_at(min_intrinsic_sizes, index)
            } else {
                intrinsic_at(max_intrinsic_sizes, index)
            }
        })
        .collect()
}

pub(super) fn track_base_size(
    track: TrackSizing,
    basis: Option<Scalar>,
    intrinsic: Scalar,
    resolver: &dyn CalcResolver,
) -> Scalar {
    let min = track_min_size(track.min, basis, intrinsic, resolver);
    let max_base = match track.max {
        MaxTrackSizing::Length(length) => length.resolve_with(basis, resolver).unwrap_or(0.0),
        MaxTrackSizing::Flex(_) => 0.0,
        MaxTrackSizing::Auto | MaxTrackSizing::MinContent | MaxTrackSizing::MaxContent => intrinsic,
        MaxTrackSizing::FitContent(limit) => {
            let limit = limit.resolve_with(basis, resolver).unwrap_or(intrinsic);
            intrinsic.min(limit)
        }
    };
    min.max(max_base)
}

pub(super) fn track_min_size(
    min: MinTrackSizing,
    basis: Option<Scalar>,
    intrinsic: Scalar,
    resolver: &dyn CalcResolver,
) -> Scalar {
    match min {
        MinTrackSizing::Length(length) => length.resolve_with(basis, resolver).unwrap_or(0.0),
        MinTrackSizing::Auto | MinTrackSizing::MinContent | MinTrackSizing::MaxContent => intrinsic,
    }
}

pub(super) fn track_growth_limit(
    track: TrackSizing,
    basis: Option<Scalar>,
    intrinsic: Scalar,
    resolver: &dyn CalcResolver,
) -> Option<Scalar> {
    match track.max {
        MaxTrackSizing::Length(length) => length.resolve_with(basis, resolver),
        MaxTrackSizing::FitContent(limit) => {
            let min = track_min_size(track.min, basis, intrinsic, resolver);
            Some(
                intrinsic
                    .max(min)
                    .min(limit.resolve_with(basis, resolver).unwrap_or(intrinsic)),
            )
        }
        MaxTrackSizing::Flex(_)
        | MaxTrackSizing::Auto
        | MaxTrackSizing::MinContent
        | MaxTrackSizing::MaxContent => None,
    }
}

pub(super) fn track_sum(sizes: &[Scalar], gap: Scalar) -> Scalar {
    sizes.iter().sum::<Scalar>() + gap * sizes.len().saturating_sub(1) as Scalar
}

pub(super) fn track_content_sum(tracks: &[TrackSizing], sizes: &[Scalar], gap: Scalar) -> Scalar {
    track_sum(sizes, gap) + sub_one_flex_unfilled_space(tracks, sizes)
}

fn sub_one_flex_unfilled_space(tracks: &[TrackSizing], sizes: &[Scalar]) -> Scalar {
    let flex_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            let factor =
                track_flex_factor(*track).filter(|factor| *factor > 0.0 && *factor < 1.0)?;
            let size = sizes.get(index).copied().unwrap_or(0.0);
            (size > 0.0).then_some(size / factor)
        })
        .min_by(|left, right| left.total_cmp(right));

    let Some(flex_fraction) = flex_fraction else {
        return 0.0;
    };

    let mut occupied_fraction = 0.0;
    for (index, track) in tracks.iter().enumerate() {
        let Some(factor) =
            track_flex_factor(*track).filter(|factor| *factor > 0.0 && *factor < 1.0)
        else {
            continue;
        };
        let size = sizes.get(index).copied().unwrap_or(0.0);
        if size > factor * flex_fraction + 0.001 {
            occupied_fraction += factor;
        }
    }

    if occupied_fraction > 0.0 && occupied_fraction < 1.0 {
        flex_fraction * (1.0 - occupied_fraction)
    } else {
        0.0
    }
}

pub(super) fn offsets(sizes: &[Scalar], start: Scalar, gap: Scalar) -> Vec<Scalar> {
    let mut cursor = start;
    sizes
        .iter()
        .map(|size| {
            let offset = cursor;
            cursor += *size + gap;
            offset
        })
        .collect()
}

pub(super) fn rtl_offsets(
    sizes: &[Scalar],
    content_box_left: Scalar,
    content_box_width: Scalar,
    start: Scalar,
    gap: Scalar,
) -> Vec<Scalar> {
    if content_box_width <= 0.0 {
        return vec![content_box_left; sizes.len()];
    }

    let mut cursor = content_box_left + content_box_width - start;
    sizes
        .iter()
        .map(|size| {
            cursor -= *size;
            let offset = cursor;
            cursor -= gap;
            offset
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CalcExpression, CalcTerm, LayoutCalcStore};

    #[test]
    fn px_only_calc_max_track_does_not_force_max_intrinsic_resolution() {
        let mut resolver = LayoutCalcStore::new();
        let calc = resolver.push(CalcExpression::sum([CalcTerm::px(24.0)]));
        let tracks = [TrackSizing::new(
            MinTrackSizing::MinContent,
            MaxTrackSizing::Length(Length::calc(calc)),
        )];

        let sizes = track_resolution_intrinsic_sizes(&tracks, &[11.0], &[99.0], &resolver);

        assert_eq!(sizes, vec![11.0]);
    }

    #[test]
    fn basis_dependent_calc_max_track_uses_max_intrinsic_resolution() {
        let mut resolver = LayoutCalcStore::new();
        let calc = resolver.push(CalcExpression::sum([CalcTerm::percent(0.5)]));
        let tracks = [TrackSizing::new(
            MinTrackSizing::MinContent,
            MaxTrackSizing::Length(Length::calc(calc)),
        )];

        let sizes = track_resolution_intrinsic_sizes(&tracks, &[11.0], &[99.0], &resolver);

        assert_eq!(sizes, vec![99.0]);
    }
}
