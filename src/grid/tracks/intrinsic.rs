use super::subgrid_intrinsic::{
    IntrinsicGridChildInput, SubgridIntrinsicContributionInput, SubgridIntrinsicContributionReport,
    apply_subgrid_intrinsic_contributions, compute_intrinsic_grid_child,
};
use super::*;

use super::flexible::span_contribution_with_gutters;
use crate::SizingCalculationOf;
use crate::geometry::FlowAxes;
use crate::scroll::UsedOverflowAxis;

fn resolve_track_calculation_optional<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> Option<S> {
    super::ordinary::resolution_optional(resolve_track_calculation(calculation, basis))
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct IntrinsicGrid<'a, Node, S: LayoutScalar = Scalar> {
    pub(in crate::grid) style: &'a GridContainerProjection<'a, S>,
    pub(in crate::grid) constants: &'a Constants<S>,
    pub(in crate::grid) sizing_flow_axes: FlowAxes,
    pub(in crate::grid) column_tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) row_tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) column_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(in crate::grid) row_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(in crate::grid) percent_basis: LogicalSizeOf<Option<S>>,
    pub(in crate::grid) lines: GridLines,
    pub(in crate::grid) named_columns: &'a NamedGridLines,
    pub(in crate::grid) named_rows: &'a NamedGridLines,
    pub(in crate::grid) area_facts: Option<&'a GridAreaNameFacts>,
    pub(in crate::grid) subgrid_report: &'a GridSubgridReport<Node>,
    pub(in crate::grid) placements: &'a GridPlacementContext<Node, S>,
}

impl<'a, Node, S: LayoutScalar> IntrinsicGrid<'a, Node, S> {
    fn gutters(self, axis: GridAxisKind) -> Option<&'a OrdinaryGridAxisGuttersOf<S>> {
        match axis {
            GridAxisKind::Column => self.column_gutters,
            GridAxisKind::Row => self.row_gutters,
        }
    }

    fn span_extent(self, axis: GridAxisKind, sizes: &[S], start: usize, end: usize) -> S {
        track_span_sum_with_gutters(
            sizes,
            start,
            end,
            match axis {
                GridAxisKind::Column => self.gap.inline,
                GridAxisKind::Row => self.gap.block,
            },
            self.gutters(axis),
        )
    }
}

#[derive(Clone, Copy, Default)]
pub(in crate::grid) struct IntrinsicGridLowerBounds<'a, S: LayoutScalar = Scalar> {
    pub(in crate::grid) columns: Option<&'a [S]>,
    pub(in crate::grid) rows: Option<&'a [S]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum AncestorBaselineRole {
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum BaselineOwnerEdge {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct AncestorBaselineTarget<S: LayoutScalar = Scalar> {
    role: AncestorBaselineRole,
    selected_ancestor_track: usize,
    selected_owner_edge: BaselineOwnerEdge,
    finite_owner_logical_target: S,
}

impl<S: LayoutScalar> AncestorBaselineTarget<S> {
    fn from_member<Node>(member: AncestorBaselineMember<Node, S>, target: S) -> Self {
        Self {
            role: member.role,
            selected_ancestor_track: member.selected_track,
            selected_owner_edge: match member.role {
                AncestorBaselineRole::First => BaselineOwnerEdge::Start,
                AncestorBaselineRole::Last => BaselineOwnerEdge::End,
            },
            finite_owner_logical_target: target,
        }
    }

    pub(in crate::grid) const fn role(self) -> AncestorBaselineRole {
        self.role
    }

    pub(in crate::grid) const fn selected_ancestor_track(self) -> usize {
        self.selected_ancestor_track
    }

    pub(in crate::grid) const fn selected_owner_edge(self) -> BaselineOwnerEdge {
        self.selected_owner_edge
    }

    pub(in crate::grid) const fn finite_owner_logical_target(self) -> S {
        self.finite_owner_logical_target
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::grid) struct TrackAncestorBaselineTargets<S: LayoutScalar = Scalar> {
    pub(in crate::grid) first: Option<AncestorBaselineTarget<S>>,
    pub(in crate::grid) last: Option<AncestorBaselineTarget<S>>,
}

fn reduce_complete_target<S: LayoutScalar>(
    slot: &mut Option<AncestorBaselineTarget<S>>,
    candidate: AncestorBaselineTarget<S>,
) {
    if slot.is_none_or(|current| {
        candidate.finite_owner_logical_target > current.finite_owner_logical_target
    }) {
        *slot = Some(candidate);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FlattenedScalarContribution<Node, S: LayoutScalar = Scalar> {
    pub(super) source: Node,
    axis: GridAxisKind,
    pub(super) ancestor_span: GridTrackSpan,
    pub(super) contribution_kind: IntrinsicSpanContribution,
    pub(super) contribution: S,
}

impl<Node, S: LayoutScalar> FlattenedScalarContribution<Node, S> {
    pub(super) fn new(
        source: Node,
        axis: GridAxisKind,
        ancestor_span: GridTrackSpan,
        contribution_kind: IntrinsicSpanContribution,
        contribution: S,
    ) -> Self {
        Self {
            source,
            axis,
            ancestor_span,
            contribution_kind,
            contribution,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct AncestorBaselineMember<Node, S: LayoutScalar = Scalar> {
    source: Node,
    axis: GridAxisKind,
    physical_axis: crate::geometry::PhysicalAxis,
    ancestor_span: GridTrackSpan,
    selected_track: usize,
    role: AncestorBaselineRole,
    containing_logical_distance: S,
    ancestor_adjustment: S,
    opposite_containing_logical_distance: S,
    opposite_ancestor_adjustment: S,
}

impl<Node, S: LayoutScalar> AncestorBaselineMember<Node, S> {
    pub(in crate::grid) fn role(self) -> AncestorBaselineRole {
        self.role
    }

    pub(in crate::grid) fn selected_track(self) -> usize {
        self.selected_track
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct AncestorBaselineGroup<Node, S: LayoutScalar = Scalar> {
    owner: Node,
    axis: GridAxisKind,
    physical_axis: crate::geometry::PhysicalAxis,
    targets: Vec<TrackAncestorBaselineTargets<S>>,
    reversed_targets: Vec<TrackAncestorBaselineTargets<S>>,
    reversed_major_translation: Vec<S>,
    reversed_minor_translation: Vec<S>,
}

impl<Node: Copy, S: LayoutScalar> AncestorBaselineGroup<Node, S> {
    pub(in crate::grid) fn reduce(
        owner: Node,
        axis: GridAxisKind,
        physical_axis: crate::geometry::PhysicalAxis,
        track_count: usize,
        members: impl IntoIterator<Item = AncestorBaselineMember<Node, S>>,
    ) -> Self {
        let mut targets = vec![TrackAncestorBaselineTargets::default(); track_count];
        let mut reversed_targets = vec![TrackAncestorBaselineTargets::default(); track_count];
        let mut reversed_major_translation = vec![S::ZERO; track_count];
        let mut reversed_minor_translation = vec![S::ZERO; track_count];
        for member in members {
            if member.axis != axis || member.physical_axis != physical_axis {
                continue;
            }
            let expected_track = match member.role {
                AncestorBaselineRole::First => member.ancestor_span.start.checked_sub(1),
                AncestorBaselineRole::Last => member.ancestor_span.end.checked_sub(2),
            };
            if expected_track != Some(member.selected_track) {
                continue;
            }
            let Some(track) = targets.get_mut(member.selected_track) else {
                continue;
            };
            let target =
                AncestorBaselineTarget::from_member(member, member.containing_logical_distance);
            match member.role {
                AncestorBaselineRole::First => reduce_complete_target(&mut track.first, target),
                AncestorBaselineRole::Last => reduce_complete_target(&mut track.last, target),
            }
            let Some(reversed_track) = reversed_targets.get_mut(member.selected_track) else {
                continue;
            };
            let reversed_target = AncestorBaselineTarget::from_member(
                member,
                member.opposite_containing_logical_distance,
            );
            let replace_reversed = match member.role {
                AncestorBaselineRole::First => reversed_track.first,
                AncestorBaselineRole::Last => reversed_track.last,
            }
            .is_none_or(|current| {
                reversed_target.finite_owner_logical_target > current.finite_owner_logical_target
            });
            match member.role {
                AncestorBaselineRole::First => {
                    reduce_complete_target(&mut reversed_track.first, reversed_target)
                }
                AncestorBaselineRole::Last => {
                    reduce_complete_target(&mut reversed_track.last, reversed_target)
                }
            };
            if replace_reversed {
                match member.role {
                    AncestorBaselineRole::First => {
                        reversed_major_translation[member.selected_track] =
                            member.opposite_ancestor_adjustment;
                    }
                    AncestorBaselineRole::Last => {
                        reversed_minor_translation[member.selected_track] =
                            member.opposite_ancestor_adjustment;
                    }
                }
            }
        }
        Self {
            owner,
            axis,
            physical_axis,
            targets,
            reversed_targets,
            reversed_major_translation,
            reversed_minor_translation,
        }
    }

    pub(in crate::grid) fn intrinsic_shim(
        &self,
        member: AncestorBaselineMember<Node, S>,
    ) -> BaselineShim<S> {
        let Some(shared) = self.target_for(member) else {
            return BaselineShim::default();
        };
        let group = match member.role {
            AncestorBaselineRole::First => BaselineGroupKind::Major,
            AncestorBaselineRole::Last => BaselineGroupKind::Minor,
        };
        let baseline =
            PhysicalBaseline::new(self.physical_axis, member.containing_logical_distance);
        let shared = PhysicalBaseline::new(self.physical_axis, shared);
        let track = match member.role {
            AncestorBaselineRole::First => TrackBaselineGroup {
                first: Some(shared),
                last: None,
            },
            AncestorBaselineRole::Last => TrackBaselineGroup {
                first: None,
                last: Some(shared),
            },
        };
        baseline_shim_for_intrinsic_contribution(
            BaselineParticipation {
                participates: true,
                group: Some(group),
                synthesized: false,
                fallback_alignment: None,
            },
            BaselineGeometry {
                available_span_size: S::ZERO,
                margin_box_size: S::ZERO,
                major_baseline: baseline,
                minor_baseline: baseline,
            },
            track,
            self.physical_axis,
        )
    }

    pub(in crate::grid) fn target_for(&self, member: AncestorBaselineMember<Node, S>) -> Option<S> {
        if member.axis != self.axis || member.physical_axis != self.physical_axis {
            return None;
        }
        self.target_record(member.role, member.selected_track)
            .map(AncestorBaselineTarget::finite_owner_logical_target)
    }

    pub(in crate::grid) fn placement_offset(
        &self,
        member: AncestorBaselineMember<Node, S>,
        available_span_size: S,
        margin_box_size: S,
        start_margin: S,
    ) -> Option<S> {
        let shared = self.target_for(member)?;
        Some(self.placement_offset_for_target(
            member,
            shared,
            available_span_size,
            margin_box_size,
            start_margin,
        ))
    }

    pub(in crate::grid) fn placement_offset_for_target(
        &self,
        member: AncestorBaselineMember<Node, S>,
        shared: S,
        available_span_size: S,
        margin_box_size: S,
        start_margin: S,
    ) -> S {
        match member.role {
            AncestorBaselineRole::First => {
                shared - member.containing_logical_distance + start_margin
            }
            AncestorBaselineRole::Last => {
                available_span_size
                    - (shared - member.containing_logical_distance)
                    - margin_box_size
                    + start_margin
            }
        }
    }

    pub(in crate::grid) fn synthesized_opposite_placement_offset(
        &self,
        member: AncestorBaselineMember<Node, S>,
        opposite_member: AncestorBaselineMember<Node, S>,
        available_span_size: S,
        start_margin: S,
        end_margin: S,
    ) -> Option<S> {
        let opposite_target = self
            .target_record(
                match member.role {
                    AncestorBaselineRole::First => AncestorBaselineRole::Last,
                    AncestorBaselineRole::Last => AncestorBaselineRole::First,
                },
                member.selected_track,
            )?
            .finite_owner_logical_target();
        match member.role {
            AncestorBaselineRole::First => Some(
                available_span_size - opposite_target + opposite_member.containing_logical_distance,
            ),
            AncestorBaselineRole::Last => Some(
                opposite_target - opposite_member.containing_logical_distance + start_margin
                    - end_margin,
            ),
        }
    }

    pub(in crate::grid) const fn axis(&self) -> GridAxisKind {
        self.axis
    }

    pub(in crate::grid) const fn owner(&self) -> Node {
        self.owner
    }

    pub(in crate::grid) const fn physical_axis(&self) -> crate::geometry::PhysicalAxis {
        self.physical_axis
    }

    pub(in crate::grid) fn track_count(&self) -> usize {
        self.targets.len()
    }

    pub(in crate::grid) fn has_any_target(&self) -> bool {
        self.targets
            .iter()
            .any(|targets| targets.first.is_some() || targets.last.is_some())
    }

    pub(in crate::grid) fn target_record(
        &self,
        role: AncestorBaselineRole,
        selected_track: usize,
    ) -> Option<AncestorBaselineTarget<S>> {
        let track = self.targets.get(selected_track)?;
        match role {
            AncestorBaselineRole::First => track.first,
            AncestorBaselineRole::Last => track.last,
        }
    }

    pub(in crate::grid) fn target_record_for_progression(
        &self,
        role: AncestorBaselineRole,
        selected_track: usize,
        opposite_progression: bool,
    ) -> Option<AncestorBaselineTarget<S>> {
        let targets = if opposite_progression {
            &self.reversed_targets
        } else {
            &self.targets
        };
        let track = targets.get(selected_track)?;
        match role {
            AncestorBaselineRole::First => track.first,
            AncestorBaselineRole::Last => track.last,
        }
    }

    pub(in crate::grid) fn track_groups(&self) -> Vec<TrackBaselineGroup<S>> {
        self.targets
            .iter()
            .map(|targets| TrackBaselineGroup {
                first: targets.first.map(|target| {
                    PhysicalBaseline::new(self.physical_axis, target.finite_owner_logical_target)
                }),
                last: targets.last.map(|target| {
                    PhysicalBaseline::new(self.physical_axis, target.finite_owner_logical_target)
                }),
            })
            .collect()
    }

    pub(in crate::grid) fn target_records_for_child_view(
        &self,
        reversed: bool,
    ) -> impl Iterator<Item = TrackAncestorBaselineTargets<S>> + '_ {
        if reversed {
            self.reversed_targets.iter().copied()
        } else {
            self.targets.iter().copied()
        }
    }

    pub(in crate::grid) fn reversed_child_view_translations(&self) -> (&[S], &[S]) {
        (
            &self.reversed_major_translation,
            &self.reversed_minor_translation,
        )
    }
}

#[derive(Clone, Copy)]
pub(super) struct RowIntrinsicContribution<Node, S: LayoutScalar = Scalar> {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) contributes_to_row_size: bool,
    pub(super) contribution_kind: IntrinsicSpanContribution,
    pub(super) contribution: S,
    pub(super) baseline_member: Option<AncestorBaselineMember<Node, S>>,
}

#[expect(
    clippy::type_complexity,
    reason = "intrinsic sizing returns both grid axes through the session error envelope"
)]
pub(in crate::grid) fn intrinsic_track_sizes<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    grid: IntrinsicGrid<'_, <Tree as Traverse>::Node, Tree::Scalar>,
    available: Size<AvailableOf<Tree::Scalar>>,
    lower_bounds: IntrinsicGridLowerBounds<'_, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (Vec<Tree::Scalar>, Vec<Tree::Scalar>), Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = grid.style;
    let constants = grid.constants;
    let column_tracks = grid.column_tracks;
    let row_tracks = grid.row_tracks;
    let logical_available = grid.sizing_flow_axes.logical_size(available);
    let column_count = column_tracks.len();
    let row_count = row_tracks.len();
    let mut columns: Vec<Tree::Scalar> = lower_bounds
        .columns
        .map(|bounds| bounds.iter().copied().take(column_count).collect())
        .unwrap_or_else(|| vec![Tree::Scalar::ZERO; column_count]);
    columns.resize(column_count, Tree::Scalar::ZERO);
    let mut rows: Vec<Tree::Scalar> = lower_bounds
        .rows
        .map(|bounds| bounds.iter().copied().take(row_count).collect())
        .unwrap_or_else(|| vec![Tree::Scalar::ZERO; row_count]);
    rows.resize(row_count, Tree::Scalar::ZERO);
    let mut row_contributions = Vec::new();
    let zero_columns: Vec<Tree::Scalar> = vec![Tree::Scalar::ZERO; column_count];
    let zero_rows: Vec<Tree::Scalar> = vec![Tree::Scalar::ZERO; row_count];
    let children = tree.children(node).collect::<Vec<_>>();
    let column_geometry = grid
        .column_gutters
        .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(zero_columns.clone(), gutters));
    let row_geometry = grid
        .row_gutters
        .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(zero_rows.clone(), gutters));
    let placed_areas = resolve_grid_child_areas_with_geometry(
        ResolveGridChildAreasInput {
            children: &children,
            placements: grid.placements,
            style,
            columns: &zero_columns,
            rows: &zero_rows,
            gap: LogicalSizeOf::new(Tree::Scalar::ZERO, Tree::Scalar::ZERO),
            lines: grid.lines,
        },
        column_geometry.as_ref(),
        row_geometry.as_ref(),
    );
    let column_area_sizes = columns.clone();
    let row_area_sizes = rows.clone();
    let column_subgrid_contributions = apply_subgrid_intrinsic_contributions(
        tree,
        SubgridIntrinsicContributionInput {
            owner: node,
            constants,
            container_style: style,
            placements: grid.placements,
            axis: GridAxisKind::Column,
            tracks: column_tracks,
            sizes: &mut columns,
            percent_basis: grid.percent_basis.inline,
            gap: grid.gap.inline,
            gutters: grid.column_gutters,
            column_gutters: grid.column_gutters,
            row_gutters: grid.row_gutters,
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
    )?;
    let column_area_sizes = columns.clone();
    let row_area_sizes = rows.clone();
    let row_subgrid_contributions = apply_subgrid_intrinsic_contributions(
        tree,
        SubgridIntrinsicContributionInput {
            owner: node,
            constants,
            container_style: style,
            placements: grid.placements,
            axis: GridAxisKind::Row,
            tracks: row_tracks,
            sizes: &mut rows,
            percent_basis: grid.percent_basis.block,
            gap: grid.gap.block,
            gutters: grid.row_gutters,
            column_gutters: grid.column_gutters,
            row_gutters: grid.row_gutters,
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
    )?;
    row_contributions.extend(row_subgrid_contributions.row_contributions);

    for (index, (child, area)) in children.into_iter().zip(placed_areas).enumerate() {
        let child_style = grid.placements.item_input(index);
        if !is_in_flow_grid_child(child_style) {
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
        area.size = LogicalSizeOf::new(
            grid.span_extent(GridAxisKind::Column, &columns, column_start, column_end),
            grid.span_extent(GridAxisKind::Row, &rows, row_start, row_end),
        );
        let inherited_column_subgrid = grid.subgrid_report.items.get(index).is_some_and(|item| {
            item_inherits_parent_axis(child_style, *item, GridAxisKind::Column)
        });
        let inherited_row_subgrid =
            grid.subgrid_report.items.get(index).is_some_and(|item| {
                item_inherits_parent_axis(child_style, *item, GridAxisKind::Row)
            });
        let contributes_column = !inherited_column_subgrid
            && (!scroll_container_auto_minimum_zero_for_grid_axis(
                child_style,
                grid.sizing_flow_axes,
                GridAxisKind::Column,
            ) || (inherited_row_subgrid
                && logical_available.inline == AvailableOf::MAX_CONTENT))
            && column_span_tracks
                .is_some_and(|tracks| tracks.iter().any(track_accepts_intrinsic_contribution));
        let align_self = child_style
            .align_self
            .or(style.align_items)
            .unwrap_or(AlignItems::Stretch);
        let justify_self = child_style
            .justify_self
            .or(style.justify_items)
            .unwrap_or(AlignItems::Stretch);
        let contributes_row = !inherited_row_subgrid
            && (!scroll_container_auto_minimum_zero_for_grid_axis(
                child_style,
                grid.sizing_flow_axes,
                GridAxisKind::Row,
            ) || (inherited_column_subgrid
                && logical_available.block == AvailableOf::MAX_CONTENT))
            && row_span_tracks
                .is_some_and(|tracks| tracks.iter().any(track_accepts_intrinsic_contribution));
        let row_baseline_candidate = !inherited_row_subgrid
            && row_span_tracks.is_some()
            && matches!(align_self, AlignItems::Baseline | AlignItems::LastBaseline);
        if !contributes_column && !contributes_row && !row_baseline_candidate {
            continue;
        }

        let spans_min_content_column = column_tracks
            .get(column_start..column_end)
            .is_some_and(|tracks| tracks.iter().any(track_accepts_min_content_span_priority));
        let output = if logical_available.inline == AvailableOf::MIN_CONTENT
            && grid_axis_used_overflow(&child_style, grid.sizing_flow_axes, GridAxisKind::Column)
                .clips_contents()
            && !spans_min_content_column
        {
            ComputeOutputOf::HIDDEN
        } else {
            compute_intrinsic_grid_child(
                tree,
                child,
                IntrinsicGridChildInput {
                    child_style,
                    grid,
                    area,
                    columns: &columns,
                    rows: &rows,
                    subgrid_item: grid.subgrid_report.items.get(index).copied(),
                    input: ComputeInputOf::for_child(
                        if matches!(align_self, AlignItems::Baseline | AlignItems::LastBaseline)
                            || matches!(
                                justify_self,
                                AlignItems::Baseline | AlignItems::LastBaseline
                            )
                        {
                            RunMode::PerformLayout
                        } else {
                            RunMode::ComputeSize
                        },
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        Size::new(
                            constants.node_inner_size.width,
                            constants.node_inner_size.height,
                        ),
                        crate::ContainingLayoutContext::new(
                            grid.constants.flow_axes,
                            crate::ParentFormattingContext::Grid,
                        ),
                        available,
                    ),
                },
            )?
        };
        let margin = intrinsic_contribution_margin(
            child_style,
            constants.flow_axes,
            constants.node_inner_size,
        )
        .map_err(|status| crate::error::value_resolution_error(child, status))?;
        let logical_margin = grid.sizing_flow_axes.logical_edges(margin);
        let column_contribution_size = grid_axis_intrinsic_contribution_size(
            child_style,
            grid.sizing_flow_axes,
            output.size,
            output.content_size,
            GridAxisKind::Column,
        );
        let row_contribution_size = grid_axis_intrinsic_contribution_size(
            child_style,
            grid.sizing_flow_axes,
            output.size,
            output.content_size,
            GridAxisKind::Row,
        );

        if contributes_column {
            let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
            let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
                child_style,
                constants,
                child_flow_axes,
            )
            .map_err(|status| crate::error::value_resolution_error(child, status))?;
            let column_baseline_member = ancestor_baseline_member(AncestorBaselineMemberInput {
                source: child,
                axis: GridAxisKind::Column,
                ancestor_span: GridTrackSpan::new(column_start + 1, column_end + 1),
                alignment: justify_self,
                block_auto_margins,
                synthesized_baseline_cycle: column_span_tracks.is_some_and(|tracks| {
                    synthesized_baseline_would_cycle(
                        justify_self,
                        output.baselines(),
                        child_flow_axes,
                        tracks,
                    )
                }),
                output,
                margin,
                child_flow_axes,
                containing_flow_axes: constants.flow_axes,
                start_adjustment: Tree::Scalar::ZERO,
                end_adjustment: Tree::Scalar::ZERO,
            });
            let column_baseline_shim =
                column_baseline_member.map_or(BaselineShim::default(), |member| {
                    column_subgrid_contributions
                        .ancestor_baseline_group
                        .intrinsic_shim(member)
                });
            let column_contribution = column_contribution_size
                + logical_margin.inline_sum()
                + column_baseline_shim.before
                + column_baseline_shim.after;
            let contribution_kind = IntrinsicSpanContribution::for_axis(
                logical_available.inline,
                grid_axis_used_overflow(&child_style, grid.sizing_flow_axes, GridAxisKind::Column),
            );
            if column_end == column_start + 1 {
                columns[column_start] = columns[column_start].max(column_contribution);
            } else if logical_available.inline == AvailableOf::MIN_CONTENT
                && column_span_tracks.is_some_and(|tracks| {
                    tracks.iter().any(track_has_percent_sizing)
                        && tracks
                            .iter()
                            .all(|track| track_flex_factor(track).is_none())
                })
            {
                distribute_min_content_span_with_percent(
                    &mut columns[column_start..column_end],
                    &column_tracks[column_start..column_end],
                    grid_axis_used_overflow(
                        &child_style,
                        grid.sizing_flow_axes,
                        GridAxisKind::Column,
                    ),
                    grid.percent_basis.inline,
                    column_contribution,
                );
            } else {
                distribute_intrinsic_span(
                    &mut columns[column_start..column_end],
                    &column_tracks[column_start..column_end],
                    contribution_kind,
                    grid.percent_basis.inline,
                    span_contribution_with_gutters(
                        column_contribution,
                        column_start,
                        column_end,
                        grid.gap.inline,
                        grid.column_gutters,
                    ),
                );
            }
        }
        if contributes_row || row_baseline_candidate {
            let contribution_kind = IntrinsicSpanContribution::for_axis(
                logical_available.block,
                grid_axis_used_overflow(&child_style, grid.sizing_flow_axes, GridAxisKind::Row),
            );
            let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
            let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
                child_style,
                constants,
                child_flow_axes,
            )
            .map_err(|status| crate::error::value_resolution_error(child, status))?;
            let baseline_member = ancestor_baseline_member(AncestorBaselineMemberInput {
                source: child,
                axis: GridAxisKind::Row,
                ancestor_span: GridTrackSpan::new(row_start + 1, row_end + 1),
                alignment: align_self,
                block_auto_margins,
                synthesized_baseline_cycle: row_span_tracks.is_some_and(|tracks| {
                    synthesized_baseline_would_cycle(
                        align_self,
                        output.baselines(),
                        child_flow_axes,
                        tracks,
                    )
                }),
                output,
                margin,
                child_flow_axes,
                containing_flow_axes: constants.flow_axes,
                start_adjustment: Tree::Scalar::ZERO,
                end_adjustment: Tree::Scalar::ZERO,
            });
            row_contributions.push(RowIntrinsicContribution {
                start: row_start,
                end: row_end,
                contributes_to_row_size: contributes_row,
                contribution_kind,
                contribution: row_contribution_size + logical_margin.block_sum(),
                baseline_member,
            });
        }
    }

    let row_baseline_groups = ancestor_baseline_group_for_intrinsic_contributions(
        node,
        &row_contributions,
        row_count,
        constants.flow_axes.block_axis(),
    );
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
                grid.percent_basis.block,
                span_contribution_with_gutters(
                    contribution,
                    item.start,
                    item.end,
                    grid.gap.block,
                    grid.row_gutters,
                ),
            );
        }
    }
    Ok((columns, rows))
}

fn ancestor_baseline_group_for_intrinsic_contributions<Node: Copy, S: LayoutScalar>(
    owner: Node,
    contributions: &[RowIntrinsicContribution<Node, S>],
    row_count: usize,
    expected_axis: crate::geometry::PhysicalAxis,
) -> AncestorBaselineGroup<Node, S> {
    AncestorBaselineGroup::reduce(
        owner,
        GridAxisKind::Row,
        expected_axis,
        row_count,
        contributions
            .iter()
            .filter_map(|contribution| contribution.baseline_member),
    )
}

fn row_baseline_shim<Node: Copy, S: LayoutScalar>(
    item: RowIntrinsicContribution<Node, S>,
    group: &AncestorBaselineGroup<Node, S>,
) -> BaselineShim<S> {
    let Some(member) = item.baseline_member else {
        return BaselineShim::default();
    };
    group.intrinsic_shim(member)
}

fn redistribute_intrinsic_baseline_envelopes<S: LayoutScalar>(
    sizes: &mut [S],
    shims: &[BaselineShim<S>],
    views: &[SubgridBaselineViewTransform<S>],
    containing_flow_axes: FlowAxes,
) {
    let deltas = intrinsic_baseline_envelope_deltas(sizes, shims, views, containing_flow_axes);
    for (size, delta) in sizes.iter_mut().zip(deltas) {
        *size = (*size + delta).max(S::ZERO);
    }
}

fn intrinsic_baseline_envelope_deltas<S: LayoutScalar>(
    track_sizes: &[S],
    shims: &[BaselineShim<S>],
    views: &[SubgridBaselineViewTransform<S>],
    containing_flow_axes: FlowAxes,
) -> Vec<S> {
    let mut deltas = vec![S::ZERO; track_sizes.len()];
    if !containing_flow_axes
        .logical_axis_progression(LogicalAxis::Block)
        .is_decreasing()
    {
        return deltas;
    }
    for transform in views {
        if !transform.root {
            continue;
        }
        let Some(span_len) = transform.parent_span.checked_len() else {
            continue;
        };
        let Some(last_local_index) = span_len.checked_sub(1).filter(|index| *index > 0) else {
            continue;
        };
        let first_ancestor_index = if transform.reversed {
            transform.parent_span.end.saturating_sub(2)
        } else {
            transform.parent_span.start.saturating_sub(1)
        };
        let Some(first_shim) = shims.get(first_ancestor_index).copied() else {
            continue;
        };
        let first_envelope = first_shim.before + first_shim.after;
        for local_index in 1..=last_local_index {
            let ancestor_index = if transform.reversed {
                transform.parent_span.end.saturating_sub(local_index + 2)
            } else {
                transform.parent_span.start.saturating_sub(1) + local_index
            };
            let Some(source_size) = track_sizes.get(ancestor_index).copied() else {
                continue;
            };
            if first_ancestor_index >= track_sizes.len() {
                continue;
            }
            let Some(shim) = shims.get(ancestor_index).copied() else {
                continue;
            };
            let half_gap = (transform.subgrid_gap - transform.parent_gap) / S::from_f64(2.0);
            let transfer = (shim.after - first_envelope + half_gap)
                .max(S::ZERO)
                .min(source_size);
            deltas[ancestor_index] = deltas[ancestor_index] - transfer;
            deltas[first_ancestor_index] = deltas[first_ancestor_index] + transfer;
        }
    }
    deltas
}

fn intrinsic_baseline_shim_census<Node: Copy, S: LayoutScalar>(
    contributions: &[RowIntrinsicContribution<Node, S>],
    group: &AncestorBaselineGroup<Node, S>,
    track_count: usize,
) -> Vec<BaselineShim<S>> {
    let mut shims: Vec<BaselineShim<S>> = vec![BaselineShim::default(); track_count];
    for contribution in contributions {
        let shim = row_baseline_shim(*contribution, group);
        if let Some(first) = shims.get_mut(contribution.start) {
            first.before = first.before.max(shim.before);
        }
        if let Some(last_index) = contribution.end.checked_sub(1)
            && let Some(last) = shims.get_mut(last_index)
        {
            last.after = last.after.max(shim.after);
        }
    }
    shims
}

fn intrinsic_downward_minor_translations<S: LayoutScalar>(
    shims: &[BaselineShim<S>],
    views: &[SubgridBaselineViewTransform<S>],
    track_count: usize,
    containing_flow_axes: FlowAxes,
) -> Vec<S> {
    let mut translations = vec![S::ZERO; track_count];
    if !containing_flow_axes
        .logical_axis_progression(LogicalAxis::Block)
        .is_decreasing()
    {
        return translations;
    }
    for transform in views {
        if !transform.root {
            continue;
        }
        let Some(span_len) = transform.parent_span.checked_len() else {
            continue;
        };
        let Some(last_local_index) = span_len.checked_sub(1).filter(|index| *index > 0) else {
            continue;
        };
        let last_ancestor_index = if transform.reversed {
            transform
                .parent_span
                .end
                .saturating_sub(last_local_index + 2)
        } else {
            transform.parent_span.start.saturating_sub(1) + last_local_index
        };
        let Some(last) = shims.get(last_ancestor_index) else {
            continue;
        };
        for local_index in 0..=last_local_index {
            let ancestor_index = if transform.reversed {
                transform.parent_span.end.saturating_sub(local_index + 2)
            } else {
                transform.parent_span.start.saturating_sub(1) + local_index
            };
            let Some((shim, current)) = shims
                .get(ancestor_index)
                .zip(translations.get_mut(ancestor_index))
            else {
                continue;
            };
            *current = current.max((last.after - shim.after).max(S::ZERO));
        }
    }
    translations
}

pub(in crate::grid) struct AncestorBaselineMemberInput<Node, S: LayoutScalar = Scalar> {
    pub(in crate::grid) source: Node,
    pub(in crate::grid) axis: GridAxisKind,
    pub(in crate::grid) ancestor_span: GridTrackSpan,
    pub(in crate::grid) alignment: AlignItems,
    pub(in crate::grid) block_auto_margins: bool,
    pub(in crate::grid) synthesized_baseline_cycle: bool,
    pub(in crate::grid) output: ComputeOutputOf<S>,
    pub(in crate::grid) margin: Edges<S>,
    pub(in crate::grid) child_flow_axes: FlowAxes,
    pub(in crate::grid) containing_flow_axes: FlowAxes,
    pub(in crate::grid) start_adjustment: S,
    pub(in crate::grid) end_adjustment: S,
}

pub(in crate::grid) fn ancestor_baseline_member<Node: Copy, S: LayoutScalar>(
    input: AncestorBaselineMemberInput<Node, S>,
) -> Option<AncestorBaselineMember<Node, S>> {
    let physical_axis = grid_axis_physical_axis(input.containing_flow_axes, input.axis);
    let baselines = input.output.baselines();
    let mut participation = baseline_participation(
        input.alignment,
        input.block_auto_margins,
        input.synthesized_baseline_cycle,
        baselines,
        input.child_flow_axes,
    );
    if input.child_flow_axes.block_axis() != physical_axis {
        participation.participates = false;
        participation.group = None;
    }
    if !participation.participates {
        return None;
    }

    let (role, baseline, selected_track, ancestor_adjustment, opposite_ancestor_adjustment) =
        match participation.group? {
            BaselineGroupKind::Major => (
                AncestorBaselineRole::First,
                baselines
                    .first_or_synthesize_block_baseline(input.child_flow_axes, input.output.size),
                input.ancestor_span.start.checked_sub(1)?,
                input.start_adjustment,
                input.end_adjustment,
            ),
            BaselineGroupKind::Minor => (
                AncestorBaselineRole::Last,
                baselines
                    .last_block_baseline(input.child_flow_axes)
                    .unwrap_or_else(|| {
                        baselines.first_or_synthesize_block_baseline(
                            input.child_flow_axes,
                            input.output.size,
                        )
                    }),
                input.ancestor_span.end.checked_sub(2)?,
                input.end_adjustment,
                input.start_adjustment,
            ),
        };
    let physical_coordinate = baseline.coordinate_on(physical_axis)?;
    let physical_extent = grid_axis_size(input.containing_flow_axes, input.output.size, input.axis);
    let logical_margin = input.containing_flow_axes.logical_edges(input.margin);
    let (start_margin, end_margin) = match input.axis {
        GridAxisKind::Column => (logical_margin.inline_start, logical_margin.inline_end),
        GridAxisKind::Row => (logical_margin.block_start, logical_margin.block_end),
    };
    let decreasing = input
        .containing_flow_axes
        .physical_axis_progression(physical_axis)
        .is_decreasing();
    let containing_logical_distance = match role {
        AncestorBaselineRole::First => {
            start_margin
                + if decreasing {
                    physical_extent - physical_coordinate
                } else {
                    physical_coordinate
                }
        }
        AncestorBaselineRole::Last => {
            end_margin
                + if decreasing {
                    physical_coordinate
                } else {
                    physical_extent - physical_coordinate
                }
        }
    } + ancestor_adjustment;
    let opposite_containing_logical_distance = match role {
        AncestorBaselineRole::First => {
            end_margin
                + if decreasing {
                    physical_coordinate
                } else {
                    physical_extent - physical_coordinate
                }
        }
        AncestorBaselineRole::Last => {
            start_margin
                + if decreasing {
                    physical_extent - physical_coordinate
                } else {
                    physical_coordinate
                }
        }
    } + opposite_ancestor_adjustment;

    Some(AncestorBaselineMember {
        source: input.source,
        axis: input.axis,
        physical_axis,
        ancestor_span: input.ancestor_span,
        selected_track,
        role,
        containing_logical_distance,
        ancestor_adjustment,
        opposite_containing_logical_distance,
        opposite_ancestor_adjustment,
    })
}

pub(in crate::grid) struct FinalAncestorBaselineGroupInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(in crate::grid) owner: Node,
    pub(in crate::grid) constants: &'a Constants<S>,
    pub(in crate::grid) axis: GridAxisKind,
    pub(in crate::grid) track_count: usize,
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) available: Size<AvailableOf<S>>,
    pub(in crate::grid) children: &'a [Node],
    pub(in crate::grid) placed_areas: &'a [Option<GridArea<S>>],
    pub(in crate::grid) placements: &'a GridPlacementContext<Node, S>,
    pub(in crate::grid) subgrid_report: &'a GridSubgridReport<Node>,
    pub(in crate::grid) named_columns: &'a NamedGridLines,
    pub(in crate::grid) named_rows: &'a NamedGridLines,
    pub(in crate::grid) area_facts: Option<&'a GridAreaNameFacts>,
    pub(in crate::grid) column_sizes: &'a [S],
    pub(in crate::grid) row_sizes: &'a [S],
    pub(in crate::grid) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) intrinsic_min_track_facts: Option<&'a [bool]>,
    pub(in crate::grid) direct_members: Vec<AncestorBaselineMember<Node, S>>,
}

pub(in crate::grid) struct FinalAncestorBaselineGroup<Node, S: LayoutScalar = Scalar> {
    pub(in crate::grid) group: AncestorBaselineGroup<Node, S>,
    pub(in crate::grid) downward_major_translation: Vec<S>,
    pub(in crate::grid) downward_minor_translation: Vec<S>,
}

type FinalAncestorBaselineGroupLayoutResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    FinalAncestorBaselineGroup<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>;

pub(in crate::grid) fn ancestor_baseline_group_for_final_placement<Tree, M>(
    tree: &mut Tree,
    input: FinalAncestorBaselineGroupInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> FinalAncestorBaselineGroupLayoutResult<Tree, M>
where
    Tree: Compute<M>,
{
    let physical_axis = grid_axis_physical_axis(input.constants.flow_axes, input.axis);
    let mut members = input.direct_members;
    if input.track_count == 0 || input.subgrid_report.items.is_empty() {
        return Ok(FinalAncestorBaselineGroup {
            group: AncestorBaselineGroup::reduce(
                input.owner,
                input.axis,
                physical_axis,
                input.track_count,
                members,
            ),
            downward_major_translation: vec![Tree::Scalar::ZERO; input.track_count],
            downward_minor_translation: vec![Tree::Scalar::ZERO; input.track_count],
        });
    }

    let fallback_track_facts = vec![false; input.track_count];
    let track_facts = input
        .intrinsic_min_track_facts
        .unwrap_or(&fallback_track_facts);
    let column_gutters = input.column_geometry.sizing_gutters();
    let row_gutters = input.row_geometry.sizing_gutters();
    let Ok(report) = collect_grid_subgrid_intrinsic_traversal(
        tree,
        GridSubgridIntrinsicTraversalInput {
            axis: input.axis,
            containing_flow_axes: input.constants.flow_axes,
            children: input.children,
            placed_areas: input.placed_areas,
            subgrid_report: input.subgrid_report,
            placements: input.placements,
            named_columns: input.named_columns,
            named_rows: input.named_rows,
            area_facts: input.area_facts,
            parent_gap: Size::new(input.gap.inline, input.gap.block),
            column_gutters: Some(&column_gutters),
            row_gutters: Some(&row_gutters),
            column_sizes: input.column_sizes,
            row_sizes: input.row_sizes,
            container_size: input.constants.node_inner_size,
            intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(track_facts),
        },
    )?
    else {
        return Ok(FinalAncestorBaselineGroup {
            group: AncestorBaselineGroup::reduce(
                input.owner,
                input.axis,
                physical_axis,
                input.track_count,
                members,
            ),
            downward_major_translation: vec![Tree::Scalar::ZERO; input.track_count],
            downward_minor_translation: vec![Tree::Scalar::ZERO; input.track_count],
        });
    };

    let baseline_views = report.baseline_views;
    for leaf in report.leaves {
        let child_style = &leaf.style;
        if !is_in_flow_grid_child(child_style)
            || !matches!(
                leaf.align_self,
                AlignItems::Baseline | AlignItems::LastBaseline
            )
        {
            continue;
        }
        let start = leaf.ancestor_span.start.saturating_sub(1);
        let end = leaf.ancestor_span.end.saturating_sub(1);
        if start >= end || end > input.track_count {
            continue;
        }

        let row_available_inline_size = (input.axis == GridAxisKind::Row
            && child_style.size.width.is_auto())
        .then_some(leaf.available_inline_size)
        .flatten()
        .filter(|width| *width > Tree::Scalar::ZERO);
        let known_inline =
            row_available_inline_size.filter(|_| leaf.available_inline_size_is_known);
        let available = if let Some(width) = row_available_inline_size {
            Size::new(AvailableOf::Definite(width), input.available.height)
        } else {
            input.available
        };
        let child_input = ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(known_inline, None),
            input.constants.node_inner_size,
            crate::ContainingLayoutContext::new(
                input.constants.flow_axes,
                crate::ParentFormattingContext::Grid,
            ),
            available,
        );
        let output = if let Some(parent_context) = &leaf.standalone_parent_context
            && tree.child_count(leaf.node) > 0
        {
            compute_standalone_grid_with_context(
                tree,
                leaf.node,
                child_input,
                parent_context.as_ref().clone(),
            )?
        } else {
            tree.compute_child(leaf.node, child_input)?
        };
        let margin = intrinsic_contribution_margin(
            child_style,
            input.constants.flow_axes,
            input.constants.node_inner_size,
        )
        .map_err(|status| crate::error::value_resolution_error(leaf.node, status))?;
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
            child_style,
            input.constants,
            child_flow_axes,
        )
        .map_err(|status| crate::error::value_resolution_error(leaf.node, status))?;
        let synthesized_baseline_cycle = leaf.ancestor_span.end
            > leaf.ancestor_span.start.saturating_add(1)
            && match leaf.align_self {
                AlignItems::Baseline => output.baselines().first_block(child_flow_axes).is_none(),
                AlignItems::LastBaseline => {
                    output.baselines().last_block(child_flow_axes).is_none()
                }
                _ => false,
            };
        let Some(adjustments) =
            leaf.ancestor_baseline_adjustments(input.constants.flow_axes, input.axis)
        else {
            continue;
        };
        if let Some(member) = ancestor_baseline_member(AncestorBaselineMemberInput {
            source: leaf.node,
            axis: input.axis,
            ancestor_span: leaf.ancestor_span,
            alignment: leaf.align_self,
            block_auto_margins,
            synthesized_baseline_cycle,
            output,
            margin,
            child_flow_axes,
            containing_flow_axes: input.constants.flow_axes,
            start_adjustment: adjustments.start,
            end_adjustment: adjustments.end,
        }) {
            members.push(member);
        }
    }

    let group = AncestorBaselineGroup::reduce(
        input.owner,
        input.axis,
        physical_axis,
        input.track_count,
        members.iter().copied(),
    );
    let (downward_major_translation, downward_minor_translation) = if input.axis
        == GridAxisKind::Row
        && input
            .intrinsic_min_track_facts
            .is_some_and(|facts| facts.iter().any(|fact| *fact))
    {
        let mut shims: Vec<BaselineShim<Tree::Scalar>> =
            vec![BaselineShim::default(); input.track_count];
        for member in members {
            let shim = group.intrinsic_shim(member);
            if let Some(track) = shims.get_mut(member.selected_track) {
                track.before = track.before.max(shim.before);
                track.after = track.after.max(shim.after);
            }
        }
        let mut descendant_tracks = vec![false; input.track_count];
        for view in baseline_views.iter().filter(|view| !view.root) {
            let start = view.parent_span.start.saturating_sub(1);
            let end = view
                .parent_span
                .end
                .saturating_sub(1)
                .min(input.track_count);
            if let Some(tracks) = descendant_tracks.get_mut(start..end) {
                tracks.fill(true);
            }
        }
        let mut major = shims
            .iter()
            .zip(&descendant_tracks)
            .map(|(shim, descendant)| {
                descendant
                    .then_some(shim.before)
                    .unwrap_or(Tree::Scalar::ZERO)
            })
            .collect::<Vec<_>>();
        if let Some(first) = major.first_mut() {
            *first = Tree::Scalar::ZERO;
        }
        let mut minor = intrinsic_downward_minor_translations(
            &shims,
            &baseline_views,
            input.track_count,
            input.constants.flow_axes,
        );
        for (translation, descendant) in minor.iter_mut().zip(descendant_tracks) {
            if !descendant {
                *translation = Tree::Scalar::ZERO;
            }
        }
        (major, minor)
    } else {
        (
            vec![Tree::Scalar::ZERO; input.track_count],
            vec![Tree::Scalar::ZERO; input.track_count],
        )
    };
    Ok(FinalAncestorBaselineGroup {
        group,
        downward_major_translation,
        downward_minor_translation,
    })
}

#[cfg(test)]
pub(in crate::grid) fn baseline_geometry_for_intrinsic_contribution<S: LayoutScalar>(
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
    flow_axes: FlowAxes,
) -> BaselineGeometry<S> {
    let baselines = output.baselines();
    let first_baseline = baselines.first_or_synthesize_block_baseline(flow_axes, output.size);
    let last_baseline = baselines.last_or_synthesize_block_baseline(flow_axes, output.size);
    BaselineGeometry {
        available_span_size: S::ZERO,
        margin_box_size: flow_axes.block_axis_extent(output.size)
            + flow_axes.line_over_edge(margin)
            + flow_axes.line_under_edge(margin),
        major_baseline: PhysicalBaseline::new(
            first_baseline.axis(),
            flow_axes.line_over_edge(margin) + first_baseline.coordinate(),
        ),
        minor_baseline: PhysicalBaseline::new(
            last_baseline.axis(),
            flow_axes.line_under_edge(margin) + flow_axes.block_axis_extent(output.size)
                - last_baseline.coordinate(),
        ),
    }
}

pub(super) fn block_auto_margins_for_intrinsic_contribution<S: LayoutScalar>(
    style: &GridItemProjection<S>,
    constants: &Constants<S>,
    child_flow_axes: FlowAxes,
) -> Result<bool, LengthResolutionStatus<S>> {
    let margin = constants.flow_axes.zip_physical_edges_with_inline_extent(
        style.margin,
        constants.node_inner_size,
        |length, basis| resolve_auto_optional(length, basis),
    );
    Ok(child_flow_axes.line_over_edge(margin)?.is_none()
        || child_flow_axes.line_under_edge(margin)?.is_none())
}

pub(super) fn grid_axis_intrinsic_contribution_size<S: LayoutScalar>(
    style: &GridItemProjection<S>,
    flow_axes: FlowAxes,
    size: Size<S>,
    content_size: Size<S>,
    axis: GridAxisKind,
) -> S {
    let item_size = grid_axis_size(flow_axes, size, axis);
    if grid_axis_used_overflow(style, flow_axes, axis).value() == Overflow::Visible {
        item_size.max(grid_axis_size(flow_axes, content_size, axis))
    } else {
        item_size
    }
}

pub(super) fn scroll_container_auto_minimum_zero_for_grid_axis<S: LayoutScalar>(
    style: &GridItemProjection<S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> bool {
    grid_axis_computed_overflow(style, flow_axes, axis).is_scrollable()
        && grid_axis_size(flow_axes, style.size.clone(), axis).is_auto()
}

pub(in crate::grid) fn constrained_row_intrinsic_sizes<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    grid: IntrinsicGrid<'_, <Tree as Traverse>::Node, Tree::Scalar>,
    columns: &[Tree::Scalar],
    gap: LogicalSizeOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Vec<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let row_count = grid.row_tracks.len();
    let mut rows: Vec<Tree::Scalar> = vec![Tree::Scalar::ZERO; row_count];
    if columns.is_empty() || row_count == 0 {
        return Ok(rows);
    }
    let zero_rows: Vec<Tree::Scalar> = vec![Tree::Scalar::ZERO; row_count];
    let children = tree.children(node).collect::<Vec<_>>();
    let column_geometry = grid
        .column_gutters
        .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(columns.to_vec(), gutters));
    let row_geometry = grid
        .row_gutters
        .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(zero_rows.clone(), gutters));
    let placed_areas = resolve_grid_child_areas_with_geometry(
        ResolveGridChildAreasInput {
            children: &children,
            placements: grid.placements,
            style: grid.style,
            columns,
            rows: &zero_rows,
            gap,
            lines: grid.lines,
        },
        column_geometry.as_ref(),
        row_geometry.as_ref(),
    );
    let subgrid_contributions =
        if grid
            .subgrid_report
            .items
            .iter()
            .enumerate()
            .any(|(index, item)| {
                item_inherits_parent_axis(
                    grid.placements.item_input(index),
                    *item,
                    GridAxisKind::Row,
                )
            })
        {
            apply_subgrid_intrinsic_contributions(
                tree,
                SubgridIntrinsicContributionInput {
                    owner: node,
                    constants: grid.constants,
                    container_style: grid.style,
                    placements: grid.placements,
                    axis: GridAxisKind::Row,
                    tracks: grid.row_tracks,
                    sizes: &mut rows,
                    percent_basis: grid.percent_basis.block,
                    gap: gap.block,
                    gutters: grid.row_gutters,
                    column_gutters: grid.column_gutters,
                    row_gutters: grid.row_gutters,
                    container_gap: gap,
                    available: Size::new(
                        AvailableOf::Definite(track_sum_with_gutters(
                            columns,
                            gap.inline,
                            grid.column_gutters,
                        )),
                        AvailableOf::MAX_CONTENT,
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
            )?
        } else {
            SubgridIntrinsicContributionReport {
                contributing_roots: Vec::new(),
                row_contributions: Vec::new(),
                baseline_views: Vec::new(),
                ancestor_baseline_group: AncestorBaselineGroup::reduce(
                    node,
                    GridAxisKind::Row,
                    grid.constants.flow_axes.block_axis(),
                    row_count,
                    core::iter::empty::<
                        AncestorBaselineMember<<Tree as Traverse>::Node, Tree::Scalar>,
                    >(),
                ),
            }
        };
    let SubgridIntrinsicContributionReport {
        contributing_roots: published_row_subgrid_roots,
        mut row_contributions,
        baseline_views,
        ..
    } = subgrid_contributions;

    for (index, (child, area)) in children.into_iter().zip(placed_areas).enumerate() {
        let child_style = grid.placements.item_input(index);
        if !is_in_flow_grid_child(child_style) {
            continue;
        }

        let Some(area) = area else {
            continue;
        };
        if area.row >= row_count || area.column >= columns.len() {
            continue;
        }
        if scroll_container_auto_minimum_zero_for_grid_axis(
            child_style,
            grid.sizing_flow_axes,
            GridAxisKind::Row,
        ) {
            continue;
        }
        if area.row_end > row_count {
            continue;
        }
        if let Some(item) = grid.subgrid_report.items.get(index)
            && item_inherits_parent_axis(child_style, *item, GridAxisKind::Row)
            && published_row_subgrid_roots.contains(&child)
        {
            continue;
        }
        let physical_area_size = grid_area_physical_size(grid.constants.flow_axes, area.size);
        let sizing = grid_item_sizing_for_grid_flow::<Tree, M>(
            tree,
            child,
            child_style,
            grid.style,
            physical_area_size,
            physical_area_size.map(Some),
            grid.sizing_flow_axes,
        )?;
        let margin = intrinsic_contribution_margin(
            child_style,
            grid.constants.flow_axes,
            physical_area_size.map(Some),
        )
        .map_err(|status| crate::error::value_resolution_error(child, status))?;
        let logical_sizing_known = grid.sizing_flow_axes.logical_size(sizing.known);
        let logical_sizing_available = grid.sizing_flow_axes.logical_size(sizing.available);
        let output = compute_intrinsic_grid_child(
            tree,
            child,
            IntrinsicGridChildInput {
                child_style,
                grid,
                area,
                columns,
                rows: &zero_rows,
                subgrid_item: grid.subgrid_report.items.get(index).copied(),
                input: ComputeInputOf::for_child(
                    if matches!(
                        sizing.align_self,
                        AlignItems::Baseline | AlignItems::LastBaseline
                    ) {
                        RunMode::PerformLayout
                    } else {
                        RunMode::ComputeSize
                    },
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    grid.sizing_flow_axes
                        .physical_size(LogicalSizeOf::new(logical_sizing_known.inline, None)),
                    physical_area_size.map(Some),
                    crate::ContainingLayoutContext::new(
                        grid.constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    grid.sizing_flow_axes.physical_size(LogicalSizeOf::new(
                        AvailableOf::definite(logical_sizing_available.inline),
                        AvailableOf::MAX_CONTENT,
                    )),
                ),
            },
        )?;
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
            child_style,
            grid.constants,
            child_flow_axes,
        )
        .map_err(|status| crate::error::value_resolution_error(child, status))?;
        let row_span_tracks = grid.row_tracks.get(area.row..area.row_end).unwrap_or(&[]);
        let baseline_member = ancestor_baseline_member(AncestorBaselineMemberInput {
            source: child,
            axis: GridAxisKind::Row,
            ancestor_span: GridTrackSpan::new(area.row + 1, area.row_end + 1),
            alignment: sizing.align_self,
            block_auto_margins,
            synthesized_baseline_cycle: synthesized_baseline_would_cycle(
                sizing.align_self,
                output.baselines(),
                child_flow_axes,
                row_span_tracks,
            ),
            output,
            margin,
            child_flow_axes,
            containing_flow_axes: grid.constants.flow_axes,
            start_adjustment: Tree::Scalar::ZERO,
            end_adjustment: Tree::Scalar::ZERO,
        });
        row_contributions.push(RowIntrinsicContribution {
            start: area.row,
            end: area.row_end,
            contributes_to_row_size: true,
            contribution_kind: IntrinsicSpanContribution::MaxContent,
            contribution: grid_axis_intrinsic_contribution_size(
                child_style,
                grid.sizing_flow_axes,
                output.size,
                output.content_size,
                GridAxisKind::Row,
            ) + grid.sizing_flow_axes.logical_edges(margin).block_sum(),
            baseline_member,
        });
    }

    let row_baseline_groups = ancestor_baseline_group_for_intrinsic_contributions(
        node,
        &row_contributions,
        row_count,
        grid.constants.flow_axes.block_axis(),
    );
    let row_baseline_shims =
        intrinsic_baseline_shim_census(&row_contributions, &row_baseline_groups, row_count);
    for item in row_contributions {
        if !item.contributes_to_row_size {
            continue;
        }
        let shim = if grid
            .row_tracks
            .get(item.start..item.end)
            .is_some_and(|tracks| tracks.iter().any(track_accepts_intrinsic_contribution))
        {
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
                grid.percent_basis.block,
                span_contribution_with_gutters(
                    contribution,
                    item.start,
                    item.end,
                    gap.block,
                    grid.row_gutters,
                ),
            );
        }
    }
    redistribute_intrinsic_baseline_envelopes(
        &mut rows,
        &row_baseline_shims,
        &baseline_views,
        grid.constants.flow_axes,
    );

    Ok(rows)
}

pub(in crate::grid) fn constrained_column_intrinsic_sizes<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    grid: IntrinsicGrid<'_, <Tree as Traverse>::Node, Tree::Scalar>,
    columns: &[Tree::Scalar],
    rows: &[Tree::Scalar],
    gap: LogicalSizeOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Vec<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let column_count = grid.column_tracks.len();
    let mut column_sizes: Vec<Tree::Scalar> = vec![Tree::Scalar::ZERO; column_count];
    if column_count == 0 || rows.is_empty() {
        return Ok(column_sizes);
    }

    let children = tree.children(node).collect::<Vec<_>>();
    let column_geometry = grid
        .column_gutters
        .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(columns.to_vec(), gutters));
    let row_geometry = grid
        .row_gutters
        .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(rows.to_vec(), gutters));
    let placed_areas = resolve_grid_child_areas_with_geometry(
        ResolveGridChildAreasInput {
            children: &children,
            placements: grid.placements,
            style: grid.style,
            columns,
            rows,
            gap,
            lines: grid.lines,
        },
        column_geometry.as_ref(),
        row_geometry.as_ref(),
    );

    for (index, (child, area)) in children.into_iter().zip(placed_areas).enumerate() {
        let child_style = grid.placements.item_input(index);
        if !is_in_flow_grid_child(child_style) {
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
        if scroll_container_auto_minimum_zero_for_grid_axis(
            child_style,
            grid.sizing_flow_axes,
            GridAxisKind::Column,
        ) {
            continue;
        }
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        if child_flow_axes.inline_axis() == grid.sizing_flow_axes.inline_axis() {
            continue;
        }

        let physical_area_size = grid_area_physical_size(grid.constants.flow_axes, area.size);
        let sizing = grid_item_sizing_for_grid_flow::<Tree, M>(
            tree,
            child,
            child_style,
            grid.style,
            physical_area_size,
            physical_area_size.map(Some),
            grid.sizing_flow_axes,
        )?;
        let margin = grid.sizing_flow_axes.logical_edges(
            sizing
                .unresolved_margin
                .map(|margin| margin.unwrap_or(Tree::Scalar::ZERO)),
        );
        let logical_sizing_known = grid.sizing_flow_axes.logical_size(sizing.known);
        let logical_sizing_available = grid.sizing_flow_axes.logical_size(sizing.available);
        let output = tree.compute_child(
            child,
            ComputeInputOf::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                grid.sizing_flow_axes
                    .physical_size(LogicalSizeOf::new(None, logical_sizing_known.block)),
                physical_area_size.map(Some),
                crate::ContainingLayoutContext::new(
                    grid.constants.flow_axes,
                    crate::ParentFormattingContext::Grid,
                ),
                grid.sizing_flow_axes.physical_size(LogicalSizeOf::new(
                    AvailableOf::MIN_CONTENT,
                    AvailableOf::definite(logical_sizing_available.block),
                )),
            ),
        )?;
        column_sizes[area.column] = column_sizes[area.column].max(
            grid_axis_intrinsic_contribution_size(
                child_style,
                grid.sizing_flow_axes,
                output.size,
                output.content_size,
                GridAxisKind::Column,
            ) + margin.inline_sum(),
        );
    }

    Ok(column_sizes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum IntrinsicSpanContribution {
    MinContent { prioritize_min_tracks: bool },
    MaxContent,
}

impl IntrinsicSpanContribution {
    pub(in crate::grid) const fn for_axis<S: LayoutScalar>(
        available: AvailableOf<S>,
        overflow: UsedOverflowAxis,
    ) -> Self {
        match available {
            AvailableOf::MaxContent | AvailableOf::Definite(_) => Self::MaxContent,
            AvailableOf::MinContent => Self::MinContent {
                prioritize_min_tracks: overflow.clips_contents(),
            },
        }
    }
}

pub(in crate::grid) fn distribute_intrinsic_span<S: LayoutScalar>(
    sizes: &mut [S],
    tracks: &[TrackSizingOf<S>],
    kind: IntrinsicSpanContribution,
    percent_basis: Option<S>,
    contribution: S,
) {
    let flex_indexes = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| track_flex_factor(track).is_some().then_some(index))
        .collect::<Vec<_>>();
    if !flex_indexes.is_empty() {
        let contribution =
            contribution - track_basis_dependent_space(tracks, percent_basis.unwrap_or(S::ZERO));
        let current = sizes
            .iter()
            .copied()
            .fold(S::ZERO, |sum, value| sum + value)
            + intrinsic_span_definite_track_space(tracks);
        let extra = (contribution - current).max(S::ZERO);
        if extra == S::ZERO {
            return;
        }

        let flex_sum = flex_indexes
            .iter()
            .map(|index| track_flex_factor(&tracks[*index]).unwrap_or(S::ZERO))
            .fold(S::ZERO, |sum, value| sum + value);
        for index in flex_indexes.iter().copied() {
            let share = if flex_sum > S::ZERO {
                extra * track_flex_factor(&tracks[index]).unwrap_or(S::ZERO) / flex_sum
            } else {
                extra / S::from_usize(flex_indexes.len())
            };
            sizes[index] = sizes[index] + share;
        }
        return;
    }

    let auto_indexes = intrinsic_span_distribution_indexes(tracks, kind);
    if auto_indexes.is_empty() {
        return;
    }

    let contribution = if kind == IntrinsicSpanContribution::MaxContent {
        contribution - track_basis_dependent_space(tracks, percent_basis.unwrap_or(S::ZERO))
    } else {
        intrinsic_span_non_percent_contribution(tracks, contribution)
    };
    let current = sizes
        .iter()
        .copied()
        .fold(S::ZERO, |sum, value| sum + value)
        + intrinsic_span_definite_space(tracks, kind);
    let extra = (contribution - current).max(S::ZERO);
    if extra == S::ZERO {
        return;
    }

    let divisor = intrinsic_span_distribution_count(tracks, kind, auto_indexes.len());
    distribute_intrinsic_extra(sizes, &auto_indexes, extra, divisor);
}

pub(super) fn scalar_total_cmp<S: LayoutScalar>(left: S, right: S) -> core::cmp::Ordering {
    left.to_f64().total_cmp(&right.to_f64())
}

pub(in crate::grid) fn distribute_intrinsic_extra<S: LayoutScalar>(
    sizes: &mut [S],
    indexes: &[usize],
    extra: S,
    divisor: usize,
) {
    if indexes.is_empty() || extra <= S::ZERO {
        return;
    }
    if divisor > indexes.len() {
        let share = extra / S::from_usize(divisor);
        for index in indexes {
            sizes[*index] = sizes[*index] + share;
        }
        return;
    }

    let mut sorted = indexes.to_vec();
    sorted.sort_by(|left, right| scalar_total_cmp(sizes[*left], sizes[*right]));
    let mut remaining = extra;
    let mut active_count = 1;
    while active_count < sorted.len() {
        let current = sizes[sorted[active_count - 1]];
        let next = sizes[sorted[active_count]];
        let needed = (next - current).max(S::ZERO) * S::from_usize(active_count);
        if needed > S::ZERO && remaining <= needed {
            let share = remaining / S::from_usize(active_count);
            for index in &sorted[..active_count] {
                sizes[*index] = sizes[*index] + share;
            }
            return;
        }
        for index in &sorted[..active_count] {
            sizes[*index] = sizes[*index] + next - current;
        }
        remaining = remaining - needed;
        active_count += 1;
    }

    let share = remaining / S::from_usize(active_count);
    for index in &sorted[..active_count] {
        sizes[*index] = sizes[*index] + share;
    }
}

pub(in crate::grid) fn distribute_min_content_span_with_percent<S: LayoutScalar>(
    sizes: &mut [S],
    tracks: &[TrackSizingOf<S>],
    overflow: UsedOverflowAxis,
    percent_basis: Option<S>,
    min_content_contribution: S,
) {
    let fixed_space = intrinsic_span_minimum_floor_space(tracks);
    let percent_space = track_basis_dependent_space(tracks, percent_basis.unwrap_or(S::ZERO));
    let extra = (min_content_contribution - fixed_space - percent_space).max(S::ZERO);
    let indexes = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            let accepts = track_accepts_percent_min_content_span(track, overflow, percent_basis);
            accepts.then_some(index)
        })
        .collect::<Vec<_>>();
    distribute_intrinsic_extra(sizes, &indexes, extra, indexes.len());
}

pub(in crate::grid) fn intrinsic_span_distribution_indexes<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
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
                track_accepts_min_content_span_priority(track).then_some(index)
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
                track_accepts_max_content_span_priority(track).then_some(index)
            })
            .collect::<Vec<_>>();
        if !max_content_indexes.is_empty() {
            return max_content_indexes;
        }

        let auto_indexes = tracks
            .iter()
            .enumerate()
            .filter_map(|(index, track)| track_accepts_auto_span_priority(track).then_some(index))
            .collect::<Vec<_>>();
        if !auto_indexes.is_empty() {
            return auto_indexes;
        }
    }

    tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| track_accepts_intrinsic_contribution(track).then_some(index))
        .collect::<Vec<_>>()
}

pub(in crate::grid) fn intrinsic_span_non_percent_contribution<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    contribution: S,
) -> S {
    (contribution - track_basis_dependent_space(tracks, contribution)).max(S::ZERO)
}

pub(in crate::grid) fn track_basis_dependent_space<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: S,
) -> S {
    tracks
        .iter()
        .filter(|track| track_has_percent_sizing(track))
        .map(|track| {
            track_base_size(track, Some(basis), S::ZERO)
                - track_base_size(track, Some(S::ZERO), S::ZERO)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

pub(in crate::grid) fn intrinsic_span_distribution_count<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    kind: IntrinsicSpanContribution,
    distribution_count: usize,
) -> usize {
    if kind
        == (IntrinsicSpanContribution::MinContent {
            prioritize_min_tracks: false,
        })
    {
        let count = tracks
            .iter()
            .filter(|track| {
                track_accepts_intrinsic_contribution(track) || track_has_percent_sizing(track)
            })
            .count();
        return count.max(distribution_count).max(1);
    }

    distribution_count.max(1)
}

pub(in crate::grid) fn intrinsic_span_definite_space<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    kind: IntrinsicSpanContribution,
) -> S {
    if kind != IntrinsicSpanContribution::MaxContent {
        return S::ZERO;
    }

    tracks
        .iter()
        .filter(|track| !track_has_percent_sizing(track) && track_flex_factor(track).is_none())
        .map(track_min_floor_space)
        .fold(S::ZERO, |sum, value| sum + value)
}

pub(in crate::grid) fn intrinsic_span_definite_track_space<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
) -> S {
    tracks
        .iter()
        .filter(|track| {
            !track_accepts_intrinsic_contribution(track)
                && track_flex_factor(track).is_none()
                && !track_has_percent_sizing(track)
        })
        .map(|track| track_base_size(track, None, S::ZERO))
        .fold(S::ZERO, |sum, value| sum + value)
}

pub(in crate::grid) fn intrinsic_span_minimum_floor_space<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
) -> S {
    tracks
        .iter()
        .filter(|track| !track_has_percent_sizing(track) && track_flex_factor(track).is_none())
        .map(|track| track_min_floor_space(track))
        .fold(S::ZERO, |sum, value| sum + value)
}

pub(in crate::grid) fn track_min_floor_space<S: LayoutScalar>(track: &TrackSizingOf<S>) -> S {
    (!track.min.depends_on_basis())
        .then(|| match &track.min {
            MinTrackSizingOf::Calculation(calculation) => {
                resolve_track_calculation_optional(calculation, None)
            }
            MinTrackSizingOf::Auto
            | MinTrackSizingOf::MinContent
            | MinTrackSizingOf::MaxContent => None,
        })
        .flatten()
        .or_else(|| {
            (!track_accepts_intrinsic_contribution(track))
                .then(|| track_base_size(track, None, S::ZERO))
        })
        .unwrap_or(S::ZERO)
}

pub(in crate::grid) fn track_accepts_intrinsic_contribution<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
) -> bool {
    track.min.is_intrinsic() || track.max.is_intrinsic()
}

pub(in crate::grid) fn track_has_definite_min_floor<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
) -> bool {
    match &track.min {
        MinTrackSizingOf::Calculation(calculation) => {
            resolve_track_calculation_optional(calculation, None).is_some()
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => {
            false
        }
    }
}

pub(in crate::grid) fn track_accepts_min_content_span_priority<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
) -> bool {
    matches!(track.min, MinTrackSizingOf::MinContent)
        || matches!(track.max, MaxTrackSizingOf::MinContent)
}

pub(in crate::grid) fn track_accepts_max_content_span_priority<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
) -> bool {
    (matches!(track.min, MinTrackSizingOf::MaxContent)
        && !matches!(track.max, MaxTrackSizingOf::MinContent))
        || matches!(
            track,
            TrackSizingOf {
                min: MinTrackSizingOf::Auto,
                max: MaxTrackSizingOf::MaxContent
            }
        )
}

pub(in crate::grid) fn track_accepts_auto_span_priority<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
) -> bool {
    matches!(track.min, MinTrackSizingOf::Auto) || matches!(track.max, MaxTrackSizingOf::Auto)
}

pub(in crate::grid) fn track_accepts_percent_min_content_span<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    overflow: UsedOverflowAxis,
    percent_basis: Option<S>,
) -> bool {
    if percent_basis.is_none() && track_has_percent_sizing(track) {
        return true;
    }
    if track_has_definite_min_floor(track) {
        return false;
    }
    if overflow.clips_contents() {
        track_accepts_min_content_span_priority(track)
            || track_accepts_max_content_span_priority(track)
    } else {
        track_accepts_intrinsic_contribution(track)
    }
}

pub(in crate::grid) fn intrinsic_contribution_margin<S: LayoutScalar>(
    style: &GridItemProjection<S>,
    containing_flow_axes: crate::geometry::FlowAxes,
    containing_physical_size: Size<Option<S>>,
) -> Result<Edges<S>, LengthResolutionStatus<S>> {
    let margin = containing_flow_axes.zip_physical_edges_with_inline_extent(
        style.margin,
        containing_physical_size,
        resolve_auto_or_zero,
    );
    Ok(Edges::new(
        margin.top?,
        margin.right?,
        margin.bottom?,
        margin.left?,
    ))
}
