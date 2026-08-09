use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct OrdinaryGridAxisGuttersOf<S: LayoutScalar = Scalar> {
    collapsed: Vec<bool>,
    gutter_after: Vec<S>,
}

impl<S: LayoutScalar> OrdinaryGridAxisGuttersOf<S> {
    pub(super) fn new(track_count: usize, collapsed: &[bool], gap: S) -> Self {
        let mut collapsed = collapsed.to_vec();
        collapsed.resize(track_count, false);
        let gutter_after = (0..track_count.saturating_sub(1))
            .map(|index| {
                if collapsed[index] || collapsed[index + 1] {
                    S::ZERO
                } else {
                    gap
                }
            })
            .collect();
        Self {
            collapsed,
            gutter_after,
        }
    }

    pub(super) fn from_boundary_gutters(
        track_count: usize,
        collapsed: &[bool],
        gutter_after: &[S],
    ) -> Self {
        let mut collapsed = collapsed.to_vec();
        collapsed.resize(track_count, false);
        let mut gutter_after = gutter_after.to_vec();
        gutter_after.resize(track_count.saturating_sub(1), S::ZERO);
        Self {
            collapsed,
            gutter_after,
        }
    }

    pub(super) fn collapsed(&self) -> &[bool] {
        &self.collapsed
    }

    pub(super) fn gutter_after(&self) -> &[S] {
        &self.gutter_after
    }

    pub(super) fn active_gap_total(&self) -> S {
        self.gutter_after
            .iter()
            .copied()
            .fold(S::ZERO, |sum, gutter| sum + gutter)
    }

    pub(super) fn span_gap_total(&self, start: usize, end: usize) -> S {
        if start >= end || end > self.collapsed.len() {
            return S::ZERO;
        }
        self.gutter_after[start..end.saturating_sub(1)]
            .iter()
            .copied()
            .fold(S::ZERO, |sum, gutter| sum + gutter)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UsedGridAxisGeometryOf<S: LayoutScalar = Scalar> {
    sizes: Vec<S>,
    collapsed: Vec<bool>,
    gutter_after: Vec<S>,
    line_offsets: Vec<S>,
}

impl<S: LayoutScalar> UsedGridAxisGeometryOf<S> {
    pub(super) fn new(sizes: Vec<S>, collapsed: Vec<bool>, gap: S) -> Self {
        let gutters = OrdinaryGridAxisGuttersOf::new(sizes.len(), &collapsed, gap);
        Self::from_sizing_gutters(sizes, &gutters)
    }

    pub(super) fn from_sizing_gutters(
        sizes: Vec<S>,
        gutters: &OrdinaryGridAxisGuttersOf<S>,
    ) -> Self {
        Self::from_boundary_gutters(
            sizes,
            gutters.collapsed().to_vec(),
            gutters.gutter_after().to_vec(),
        )
    }

    pub(super) fn from_boundary_gutters(
        sizes: Vec<S>,
        collapsed: Vec<bool>,
        gutter_after: Vec<S>,
    ) -> Self {
        let mut collapsed = collapsed;
        collapsed.resize(sizes.len(), false);
        let mut gutter_after = gutter_after;
        gutter_after.resize(sizes.len().saturating_sub(1), S::ZERO);
        let mut line_offsets = Vec::with_capacity(sizes.len() + 1);
        let mut cursor = S::ZERO;
        line_offsets.push(cursor);
        for (index, size) in sizes.iter().copied().enumerate() {
            cursor = cursor + size;
            if let Some(gutter) = gutter_after.get(index) {
                cursor = cursor + *gutter;
            }
            line_offsets.push(cursor);
        }
        Self {
            sizes,
            collapsed,
            gutter_after,
            line_offsets,
        }
    }

    pub(super) fn sizes(&self) -> &[S] {
        &self.sizes
    }

    pub(super) fn collapsed(&self) -> &[bool] {
        &self.collapsed
    }

    pub(super) fn gutter_after(&self) -> &[S] {
        &self.gutter_after
    }

    pub(super) fn sizing_gutters(&self) -> OrdinaryGridAxisGuttersOf<S> {
        OrdinaryGridAxisGuttersOf::from_boundary_gutters(
            self.sizes.len(),
            &self.collapsed,
            &self.gutter_after,
        )
    }

    pub(super) fn line_offsets(&self) -> &[S] {
        &self.line_offsets
    }

    pub(super) fn active_gap_total(&self) -> S {
        self.gutter_after
            .iter()
            .copied()
            .fold(S::ZERO, |sum, gutter| sum + gutter)
    }

    pub(super) fn total_extent(&self) -> S {
        self.sizes
            .iter()
            .copied()
            .fold(S::ZERO, |sum, size| sum + size)
            + self.active_gap_total()
    }

    pub(super) fn span_extent(&self, start: usize, end: usize) -> S {
        if start >= end || end > self.sizes.len() {
            return S::ZERO;
        }
        self.sizes[start..end]
            .iter()
            .copied()
            .fold(S::ZERO, |sum, size| sum + size)
            + self.gutter_after[start..end.saturating_sub(1)]
                .iter()
                .copied()
                .fold(S::ZERO, |sum, gutter| sum + gutter)
    }

    pub(super) fn line_offset(&self, line: usize) -> Option<S> {
        self.line_offsets.get(line).copied()
    }

    pub(super) fn translated(mut self, offset: S) -> Self {
        for line_offset in &mut self.line_offsets {
            *line_offset = *line_offset + offset;
        }
        self
    }
}
use crate::geometry::{FlowAxes, LogicalSizeOf};
use crate::scroll::{UsedOverflow, UsedOverflowAxis};
use crate::{
    LengthResolutionOf, LengthResolutionStatus, MaxTrackSizingOf, MinTrackSizingOf,
    PercentageBasisOf, SizingCalculationOf,
};

pub(super) fn resolve_track_calculation<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> LengthResolutionOf<S> {
    let basis = match basis {
        Some(value) => match PercentageBasisOf::definite(value) {
            Ok(basis) => basis,
            Err(_) => {
                return LengthResolutionOf::invalid_numeric(value, calculation.depends_on_basis());
            }
        },
        None => PercentageBasisOf::MISSING,
    };
    let resolution = calculation.resolve_against(basis);
    match resolution.status() {
        LengthResolutionStatus::Resolved => LengthResolutionOf::definite(
            resolution
                .value
                .expect("resolved sizing calculation must carry a value")
                .max(S::ZERO),
            calculation.depends_on_basis(),
        ),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            resolution
        }
        LengthResolutionStatus::NonNumeric => {
            unreachable!("a sizing calculation always has numeric program semantics")
        }
    }
}

fn resolve_track_calculation_optional<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> Option<S> {
    resolution_optional(resolve_track_calculation(calculation, basis))
}
#[derive(Clone, Copy)]
pub(super) struct IntrinsicGrid<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) style: &'a NodeInputOf<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) sizing_flow_axes: FlowAxes,
    pub(super) column_tracks: &'a [TrackSizingOf<S>],
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) column_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) row_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) percent_basis: LogicalSizeOf<Option<S>>,
    pub(super) lines: GridLines,
    pub(super) named_columns: &'a NamedGridLines,
    pub(super) named_rows: &'a NamedGridLines,
    pub(super) area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<Node>,
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
pub(super) struct IntrinsicGridLowerBounds<'a, S: LayoutScalar = Scalar> {
    pub(super) columns: Option<&'a [S]>,
    pub(super) rows: Option<&'a [S]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AncestorBaselineRole {
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BaselineOwnerEdge {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AncestorBaselineTarget<S: LayoutScalar = Scalar> {
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

    pub(super) const fn role(self) -> AncestorBaselineRole {
        self.role
    }

    pub(super) const fn selected_ancestor_track(self) -> usize {
        self.selected_ancestor_track
    }

    pub(super) const fn selected_owner_edge(self) -> BaselineOwnerEdge {
        self.selected_owner_edge
    }

    pub(super) const fn finite_owner_logical_target(self) -> S {
        self.finite_owner_logical_target
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TrackAncestorBaselineTargets<S: LayoutScalar = Scalar> {
    pub(super) first: Option<AncestorBaselineTarget<S>>,
    pub(super) last: Option<AncestorBaselineTarget<S>>,
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
struct FlattenedScalarContribution<Node, S: LayoutScalar = Scalar> {
    source: Node,
    axis: GridAxisKind,
    ancestor_span: GridTrackSpan,
    contribution_kind: IntrinsicSpanContribution,
    contribution: S,
}

impl<Node, S: LayoutScalar> FlattenedScalarContribution<Node, S> {
    fn new(
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
pub(super) struct AncestorBaselineMember<Node, S: LayoutScalar = Scalar> {
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
    pub(super) fn role(self) -> AncestorBaselineRole {
        self.role
    }

    pub(super) fn selected_track(self) -> usize {
        self.selected_track
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct AncestorBaselineGroup<Node, S: LayoutScalar = Scalar> {
    owner: Node,
    axis: GridAxisKind,
    physical_axis: crate::geometry::PhysicalAxis,
    targets: Vec<TrackAncestorBaselineTargets<S>>,
    reversed_targets: Vec<TrackAncestorBaselineTargets<S>>,
    reversed_major_translation: Vec<S>,
    reversed_minor_translation: Vec<S>,
}

impl<Node: Copy, S: LayoutScalar> AncestorBaselineGroup<Node, S> {
    pub(super) fn reduce(
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

    pub(super) fn intrinsic_shim(
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

    pub(super) fn target_for(&self, member: AncestorBaselineMember<Node, S>) -> Option<S> {
        if member.axis != self.axis || member.physical_axis != self.physical_axis {
            return None;
        }
        self.target_record(member.role, member.selected_track)
            .map(AncestorBaselineTarget::finite_owner_logical_target)
    }

    pub(super) fn placement_offset(
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

    pub(super) fn placement_offset_for_target(
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

    pub(super) fn synthesized_opposite_placement_offset(
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

    pub(super) const fn axis(&self) -> GridAxisKind {
        self.axis
    }

    pub(super) const fn owner(&self) -> Node {
        self.owner
    }

    pub(super) const fn physical_axis(&self) -> crate::geometry::PhysicalAxis {
        self.physical_axis
    }

    pub(super) fn track_count(&self) -> usize {
        self.targets.len()
    }

    pub(super) fn has_any_target(&self) -> bool {
        self.targets
            .iter()
            .any(|targets| targets.first.is_some() || targets.last.is_some())
    }

    pub(super) fn target_record(
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

    pub(super) fn target_record_for_progression(
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

    pub(super) fn track_groups(&self) -> Vec<TrackBaselineGroup<S>> {
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

    pub(super) fn target_records_for_child_view(
        &self,
        reversed: bool,
    ) -> impl Iterator<Item = TrackAncestorBaselineTargets<S>> + '_ {
        if reversed {
            self.reversed_targets.iter().copied()
        } else {
            self.targets.iter().copied()
        }
    }

    pub(super) fn reversed_child_view_translations(&self) -> (&[S], &[S]) {
        (
            &self.reversed_major_translation,
            &self.reversed_minor_translation,
        )
    }
}

#[derive(Clone, Copy)]
struct RowIntrinsicContribution<Node, S: LayoutScalar = Scalar> {
    start: usize,
    end: usize,
    contributes_to_row_size: bool,
    contribution_kind: IntrinsicSpanContribution,
    contribution: S,
    baseline_member: Option<AncestorBaselineMember<Node, S>>,
}

#[expect(
    clippy::type_complexity,
    reason = "intrinsic sizing returns both grid axes through the session error envelope"
)]
pub(super) fn intrinsic_track_sizes<Tree, M>(
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
        area.size = LogicalSizeOf::new(
            grid.span_extent(GridAxisKind::Column, &columns, column_start, column_end),
            grid.span_extent(GridAxisKind::Row, &rows, row_start, row_end),
        );
        let inherited_column_subgrid = grid.subgrid_report.items.get(index).is_some_and(|item| {
            item_inherits_parent_axis(&child_style, *item, GridAxisKind::Column)
        });
        let inherited_row_subgrid =
            grid.subgrid_report.items.get(index).is_some_and(|item| {
                item_inherits_parent_axis(&child_style, *item, GridAxisKind::Row)
            });
        let contributes_column = !inherited_column_subgrid
            && !scroll_container_auto_minimum_zero_for_grid_axis(
                &child_style,
                grid.sizing_flow_axes,
                GridAxisKind::Column,
            )
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
            && !scroll_container_auto_minimum_zero_for_grid_axis(
                &child_style,
                grid.sizing_flow_axes,
                GridAxisKind::Row,
            )
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
                    child_style: &child_style,
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
            &child_style,
            constants.flow_axes,
            constants.node_inner_size,
        )
        .map_err(|status| crate::compute::value_resolution_error(child, status))?;
        let logical_margin = grid.sizing_flow_axes.logical_edges(margin);
        let column_contribution_size = grid_axis_intrinsic_contribution_size(
            &child_style,
            grid.sizing_flow_axes,
            output.size,
            output.content_size,
            GridAxisKind::Column,
        );
        let row_contribution_size = grid_axis_intrinsic_contribution_size(
            &child_style,
            grid.sizing_flow_axes,
            output.size,
            output.content_size,
            GridAxisKind::Row,
        );

        if contributes_column {
            let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
            let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
                &child_style,
                constants,
                child_flow_axes,
            )
            .map_err(|status| crate::compute::value_resolution_error(child, status))?;
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
                &child_style,
                constants,
                child_flow_axes,
            )
            .map_err(|status| crate::compute::value_resolution_error(child, status))?;
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

pub(super) struct AncestorBaselineMemberInput<Node, S: LayoutScalar = Scalar> {
    pub(super) source: Node,
    pub(super) axis: GridAxisKind,
    pub(super) ancestor_span: GridTrackSpan,
    pub(super) alignment: AlignItems,
    pub(super) block_auto_margins: bool,
    pub(super) synthesized_baseline_cycle: bool,
    pub(super) output: ComputeOutputOf<S>,
    pub(super) margin: Edges<S>,
    pub(super) child_flow_axes: FlowAxes,
    pub(super) containing_flow_axes: FlowAxes,
    pub(super) start_adjustment: S,
    pub(super) end_adjustment: S,
}

pub(super) fn ancestor_baseline_member<Node: Copy, S: LayoutScalar>(
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

pub(super) struct FinalAncestorBaselineGroupInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) owner: Node,
    pub(super) constants: &'a Constants<S>,
    pub(super) axis: GridAxisKind,
    pub(super) track_count: usize,
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) available: Size<AvailableOf<S>>,
    pub(super) children: &'a [Node],
    pub(super) placed_areas: &'a [Option<GridArea<S>>],
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) named_columns: &'a NamedGridLines,
    pub(super) named_rows: &'a NamedGridLines,
    pub(super) area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) column_sizes: &'a [S],
    pub(super) row_sizes: &'a [S],
    pub(super) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) intrinsic_min_track_facts: Option<&'a [bool]>,
    pub(super) direct_members: Vec<AncestorBaselineMember<Node, S>>,
}

pub(super) struct FinalAncestorBaselineGroup<Node, S: LayoutScalar = Scalar> {
    pub(super) group: AncestorBaselineGroup<Node, S>,
    pub(super) downward_major_translation: Vec<S>,
    pub(super) downward_minor_translation: Vec<S>,
}

type FinalAncestorBaselineGroupLayoutResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    FinalAncestorBaselineGroup<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>;

pub(super) fn ancestor_baseline_group_for_final_placement<Tree, M>(
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
        let child_style = tree.node_input(leaf.node).clone();
        if !is_in_flow_grid_child(&child_style)
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
        let output = tree.compute_child(
            leaf.node,
            ComputeInputOf::for_child(
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
            ),
        )?;
        let margin = intrinsic_contribution_margin(
            &child_style,
            input.constants.flow_axes,
            input.constants.node_inner_size,
        )
        .map_err(|status| crate::compute::value_resolution_error(leaf.node, status))?;
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
            &child_style,
            input.constants,
            child_flow_axes,
        )
        .map_err(|status| crate::compute::value_resolution_error(leaf.node, status))?;
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
pub(super) fn baseline_geometry_for_intrinsic_contribution<S: LayoutScalar>(
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

fn block_auto_margins_for_intrinsic_contribution<S: LayoutScalar>(
    style: &NodeInputOf<S>,
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

struct IntrinsicGridChildInput<'a, Node, S: LayoutScalar = Scalar> {
    child_style: &'a NodeInputOf<S>,
    grid: IntrinsicGrid<'a, Node, S>,
    area: GridArea<S>,
    columns: &'a [S],
    rows: &'a [S],
    subgrid_item: Option<SubgridItemReport<Node>>,
    input: ComputeInputOf<S>,
}

fn compute_intrinsic_grid_child<Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    args: IntrinsicGridChildInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let IntrinsicGridChildInput {
        child_style,
        grid,
        area,
        columns,
        rows,
        subgrid_item,
        input,
    } = args;

    let Some(subgrid_item) = subgrid_item else {
        return tree.compute_child(child, input);
    };
    if !subgrid_item.column.can_inherit() && !subgrid_item.row.can_inherit() {
        return tree.compute_child(child, input);
    }
    let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
    let column_constraint =
        intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
            report: subgrid_item.column,
            area,
            parent_flow_axes: grid.constants.flow_axes,
            child_flow_axes,
            explicit_parent_content_size: grid.constants.explicit_definite_content_size,
            parent_column_count: columns.len(),
            parent_row_count: rows.len(),
            tracks: grid.column_tracks,
            gap: grid.gap.inline,
        });
    let row_constraint = intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
        report: subgrid_item.row,
        area,
        parent_flow_axes: grid.constants.flow_axes,
        child_flow_axes,
        explicit_parent_content_size: grid.constants.explicit_definite_content_size,
        parent_column_count: columns.len(),
        parent_row_count: rows.len(),
        tracks: grid.row_tracks,
        gap: grid.gap.block,
    });
    let mut physical_area_size = grid_area_physical_size(grid.constants.flow_axes, area.size);
    apply_resolved_intrinsic_subgrid_area_constraints(
        &mut physical_area_size,
        [column_constraint, row_constraint],
    );
    let sizing = grid_item_sizing_for_grid_flow::<Tree, M>(
        tree,
        child,
        child_style,
        grid.style,
        physical_area_size,
        physical_area_size.map(Some),
        grid.sizing_flow_axes,
    )?;
    let input = intrinsic_subgrid_child_input(input, sizing, [column_constraint, row_constraint]);
    if !matches!(
        child_style.display.inner_display(),
        Display::Grid | Display::GridLanes
    ) {
        return tree.compute_child(child, input);
    }
    let needs_context = needs_intrinsic_subgrid_context(child_style, subgrid_item, area);
    if !input.run_mode().is_perform_layout() && !needs_context {
        return tree.compute_child(child, input);
    }

    let area_parent = physical_area_size.map(Some);
    let padding = grid
        .constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.padding,
            area_parent,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let border = grid
        .constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.border,
            area_parent,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let margin = sizing
        .unresolved_margin
        .map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
    let content_box_size =
        (physical_area_size - margin.sum_axes() - padding.sum_axes() - border.sum_axes())
            .max(Size::ZERO);
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
    })
    .map_err(|error| subgrid_child_context_error(child, error))?;
    if !child_context.has_inherited_axis() {
        return tree.compute_child(child, input);
    }

    Ok(compute_grid_with_context_result(tree, child, input, child_context)?.output)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum IntrinsicSubgridAxisAuthority<S: LayoutScalar> {
    FinalContainerContent(S),
    FinalTrackSpan(S),
    Unknown,
}

impl<S: LayoutScalar> IntrinsicSubgridAxisAuthority<S> {
    fn extent(self) -> Option<S> {
        match self {
            Self::FinalContainerContent(extent) | Self::FinalTrackSpan(extent) => Some(extent),
            Self::Unknown => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct IntrinsicSubgridAxisConstraint<S: LayoutScalar> {
    physical_axis: crate::geometry::PhysicalAxis,
    authority: IntrinsicSubgridAxisAuthority<S>,
}

struct IntrinsicSubgridAxisConstraintInput<'a, S: LayoutScalar> {
    report: SubgridAxisReport,
    area: GridArea<S>,
    parent_flow_axes: FlowAxes,
    child_flow_axes: FlowAxes,
    explicit_parent_content_size: Size<Option<S>>,
    parent_column_count: usize,
    parent_row_count: usize,
    tracks: &'a [TrackSizingOf<S>],
    gap: S,
}

fn intrinsic_subgrid_axis_constraint<S: LayoutScalar>(
    input: IntrinsicSubgridAxisConstraintInput<'_, S>,
) -> Option<IntrinsicSubgridAxisConstraint<S>> {
    let IntrinsicSubgridAxisConstraintInput {
        report,
        area,
        parent_flow_axes,
        child_flow_axes,
        explicit_parent_content_size,
        parent_column_count,
        parent_row_count,
        tracks,
        gap,
    } = input;
    let physical_axis = inherited_subgrid_physical_axis(report, parent_flow_axes, child_flow_axes)?;
    let mapping = report.mapping.ok()?;
    let (start, end, count) = match mapping.parent_axis {
        GridAxisKind::Column => (area.column, area.column_end, parent_column_count),
        GridAxisKind::Row => (area.row, area.row_end, parent_row_count),
    };
    let explicit_container_extent = if start == 0 && end == count {
        match physical_axis {
            crate::geometry::PhysicalAxis::Horizontal => explicit_parent_content_size.width,
            crate::geometry::PhysicalAxis::Vertical => explicit_parent_content_size.height,
        }
    } else {
        None
    };
    let authority = if let Some(extent) = explicit_container_extent {
        IntrinsicSubgridAxisAuthority::FinalContainerContent(extent)
    } else if let Some(extent) = exact_static_track_span(tracks, start, end, gap) {
        IntrinsicSubgridAxisAuthority::FinalTrackSpan(extent)
    } else {
        IntrinsicSubgridAxisAuthority::Unknown
    };
    Some(IntrinsicSubgridAxisConstraint {
        physical_axis,
        authority,
    })
}

fn exact_static_track_span<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    start: usize,
    end: usize,
    gap: S,
) -> Option<S> {
    let span = tracks.get(start..end)?;
    let mut extent = S::ZERO;
    for track in span {
        let (min, max) = (track.min.definite(None)?, track.max.definite(None)?);
        if min != max {
            return None;
        }
        extent = extent + min;
    }
    let gap_count = end.checked_sub(start)?.checked_sub(1)?;
    Some(extent + gap * S::from_f64(gap_count as f64))
}

pub(super) fn needs_intrinsic_subgrid_context<Node, S: LayoutScalar>(
    style: &NodeInputOf<S>,
    item: SubgridItemReport<Node>,
    area: GridArea<S>,
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
                || track_components_have_percent_sizing(&style.grid_template_columns)))
        || (inherits_columns
            && !style.size.height.is_auto()
            && track_components_have_percent_sizing(&style.grid_template_rows))
}

pub(super) fn track_components_have_percent_sizing<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> bool {
    components
        .iter()
        .any(|component| track_component_has_percent_sizing(component))
}

fn track_component_has_percent_sizing<S: LayoutScalar>(component: &TrackComponentOf<S>) -> bool {
    match component {
        TrackComponentOf::Track(track) => track_has_percent_sizing(track),
        TrackComponentOf::Repeat(repeat) => repeat
            .components()
            .iter()
            .any(|component| track_component_has_percent_sizing(component)),
        _ => false,
    }
}

fn intrinsic_subgrid_child_input<S: LayoutScalar>(
    input: ComputeInputOf<S>,
    sizing: GridItemSizing<S>,
    constraints: [Option<IntrinsicSubgridAxisConstraint<S>>; 2],
) -> ComputeInputOf<S> {
    let mut known = input.known();
    let mut parent = input.parent();
    let mut available = input.available();
    for constraint in constraints.into_iter().flatten() {
        let Some(raw_area_extent) = constraint.authority.extent() else {
            continue;
        };
        let border_box_extent =
            intrinsic_subgrid_border_box_extent(sizing, constraint.physical_axis, raw_area_extent);
        match constraint.physical_axis {
            crate::geometry::PhysicalAxis::Horizontal => {
                known.width = Some(border_box_extent);
                parent.width = Some(raw_area_extent);
                available.width = AvailableOf::Definite(border_box_extent);
            }
            crate::geometry::PhysicalAxis::Vertical => {
                known.height = Some(border_box_extent);
                parent.height = Some(raw_area_extent);
                available.height = AvailableOf::Definite(border_box_extent);
            }
        }
    }
    ComputeInputOf::for_child(
        input.run_mode(),
        input.sizing_mode(),
        input.requested_axis(),
        known,
        parent,
        input.containing_layout_context(),
        available,
    )
}

fn intrinsic_subgrid_border_box_extent<S: LayoutScalar>(
    sizing: GridItemSizing<S>,
    axis: crate::geometry::PhysicalAxis,
    extent: S,
) -> S {
    let margin = sizing
        .unresolved_margin
        .map(|value| value.unwrap_or(S::ZERO));
    let margin_sum = match axis {
        crate::geometry::PhysicalAxis::Horizontal => margin.left + margin.right,
        crate::geometry::PhysicalAxis::Vertical => margin.top + margin.bottom,
    };
    (extent - margin_sum).max(S::ZERO)
}

fn apply_resolved_intrinsic_subgrid_area_constraints<S: LayoutScalar>(
    area_size: &mut Size<S>,
    constraints: [Option<IntrinsicSubgridAxisConstraint<S>>; 2],
) {
    for constraint in constraints.into_iter().flatten() {
        let Some(extent) = constraint.authority.extent() else {
            continue;
        };
        match constraint.physical_axis {
            crate::geometry::PhysicalAxis::Horizontal => area_size.width = extent,
            crate::geometry::PhysicalAxis::Vertical => area_size.height = extent,
        }
    }
}

struct SubgridIntrinsicContributionInput<'a, Node, S: LayoutScalar = Scalar> {
    owner: Node,
    constants: &'a Constants<S>,
    container_style: &'a NodeInputOf<S>,
    axis: GridAxisKind,
    tracks: &'a [TrackSizingOf<S>],
    sizes: &'a mut [S],
    percent_basis: Option<S>,
    gap: S,
    gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    column_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    row_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    container_gap: LogicalSizeOf<S>,
    available: Size<AvailableOf<S>>,
    children: &'a [Node],
    placed_areas: &'a [Option<GridArea<S>>],
    subgrid_report: &'a GridSubgridReport<Node>,
    named_columns: &'a NamedGridLines,
    named_rows: &'a NamedGridLines,
    area_facts: Option<&'a GridAreaNameFacts>,
    column_sizes: &'a [S],
    row_sizes: &'a [S],
}

struct SubgridIntrinsicContributionReport<Node, S: LayoutScalar = Scalar> {
    contributing_roots: Vec<Node>,
    row_contributions: Vec<RowIntrinsicContribution<Node, S>>,
    ancestor_baseline_group: AncestorBaselineGroup<Node, S>,
    baseline_views: Vec<SubgridBaselineViewTransform<S>>,
}

type SubgridIntrinsicContributionResult<Node, S, M> =
    LayoutResultOf<Node, SubgridIntrinsicContributionReport<Node, S>, S, M>;

fn apply_subgrid_intrinsic_contributions<Tree, M>(
    tree: &mut Tree,
    input: SubgridIntrinsicContributionInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> SubgridIntrinsicContributionResult<<Tree as Traverse>::Node, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    if input.tracks.is_empty() || input.subgrid_report.items.is_empty() {
        return Ok(SubgridIntrinsicContributionReport {
            contributing_roots: Vec::new(),
            row_contributions: Vec::new(),
            baseline_views: Vec::new(),
            ancestor_baseline_group: AncestorBaselineGroup::reduce(
                input.owner,
                input.axis,
                grid_axis_physical_axis(input.constants.flow_axes, input.axis),
                input.tracks.len(),
                core::iter::empty::<AncestorBaselineMember<<Tree as Traverse>::Node, Tree::Scalar>>(
                ),
            ),
        });
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
            containing_flow_axes: input.constants.flow_axes,
            children: input.children,
            placed_areas: input.placed_areas,
            subgrid_report: input.subgrid_report,
            named_columns: input.named_columns,
            named_rows: input.named_rows,
            area_facts: input.area_facts,
            parent_gap: Size::new(input.container_gap.inline, input.container_gap.block),
            column_gutters: input.column_gutters,
            row_gutters: input.row_gutters,
            column_sizes: input.column_sizes,
            row_sizes: input.row_sizes,
            container_size: input.constants.node_inner_size,
            intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(&intrinsic_min_track_facts),
        },
    )?
    else {
        return Ok(SubgridIntrinsicContributionReport {
            contributing_roots: Vec::new(),
            row_contributions: Vec::new(),
            baseline_views: Vec::new(),
            ancestor_baseline_group: AncestorBaselineGroup::reduce(
                input.owner,
                input.axis,
                grid_axis_physical_axis(input.constants.flow_axes, input.axis),
                input.tracks.len(),
                core::iter::empty::<AncestorBaselineMember<<Tree as Traverse>::Node, Tree::Scalar>>(
                ),
            ),
        });
    };

    for (index, lower_bound) in report.edge_lower_bounds.into_iter().enumerate() {
        if let Some(size) = input.sizes.get_mut(index) {
            *size = size.max(lower_bound);
        }
    }

    let baseline_views = report.baseline_views;
    let mut leaves = report.leaves;
    leaves.sort_by_key(|leaf| {
        leaf.ancestor_span
            .end
            .saturating_sub(leaf.ancestor_span.start)
    });
    let mut flattened_contributions = Vec::new();
    let mut ancestor_baseline_members = Vec::new();
    let mut contributing_roots = Vec::new();
    for leaf in leaves {
        let child_style = tree.node_input(leaf.node).clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }
        if subgrid_leaf_size_depends_on_queried_axis(
            &child_style,
            input.constants.flow_axes,
            input.axis,
        ) {
            continue;
        }
        if scroll_container_auto_minimum_zero(&child_style, input.constants.flow_axes, input.axis) {
            continue;
        }
        let start = leaf.ancestor_span.start - 1;
        let end = leaf.ancestor_span.end - 1;
        let Some(span_tracks) = input.tracks.get(start..end) else {
            continue;
        };
        if !span_tracks.iter().any(track_accepts_intrinsic_contribution) {
            continue;
        }

        let row_available_inline_size = (input.axis == GridAxisKind::Row
            && child_style.size.width.is_auto())
        .then_some(leaf.available_inline_size)
        .flatten()
        .filter(|width| *width > Tree::Scalar::ZERO);
        if input.axis == GridAxisKind::Row
            && child_style.size.width.is_auto()
            && row_available_inline_size.is_none()
        {
            continue;
        }
        let row_known_inline_size =
            row_available_inline_size.filter(|_| leaf.available_inline_size_is_known);
        let available = if let Some(width) = row_available_inline_size {
            Size::new(AvailableOf::Definite(width), input.available.height)
        } else {
            input.available
        };
        let output = tree.compute_child(
            leaf.node,
            ComputeInputOf::for_child(
                if input.axis == GridAxisKind::Row
                    && matches!(
                        leaf.align_self,
                        AlignItems::Baseline | AlignItems::LastBaseline
                    )
                {
                    RunMode::PerformLayout
                } else {
                    RunMode::ComputeSize
                },
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::new(row_known_inline_size, None),
                Size::new(
                    input.constants.node_inner_size.width,
                    input.constants.node_inner_size.height,
                ),
                crate::ContainingLayoutContext::new(
                    input.constants.flow_axes,
                    crate::ParentFormattingContext::Grid,
                ),
                available,
            ),
        )?;
        let margin = intrinsic_contribution_margin(
            &child_style,
            input.constants.flow_axes,
            input.constants.node_inner_size,
        )
        .map_err(|status| crate::compute::value_resolution_error(leaf.node, status))?;
        let Some(scalar_adjustment) = leaf.scalar_adjustment() else {
            continue;
        };
        let Some(baseline_adjustments) =
            leaf.ancestor_baseline_adjustments(input.constants.flow_axes, input.axis)
        else {
            continue;
        };
        let contribution = grid_axis_intrinsic_contribution_size(
            &child_style,
            input.constants.flow_axes,
            output.size,
            output.content_size,
            input.axis,
        ) + axis_margin_sum(margin, input.axis)
            + scalar_adjustment;
        let contribution_kind = IntrinsicSpanContribution::for_axis(
            axis_available(input.available, input.axis),
            grid_axis_used_overflow(&child_style, input.constants.flow_axes, input.axis),
        );
        if let Some(root) = leaf.root_node
            && leaf.root_axis_fully_inherited
            && !contributing_roots.contains(&root)
        {
            contributing_roots.push(root);
        }
        let flattened = FlattenedScalarContribution::new(
            leaf.node,
            input.axis,
            leaf.ancestor_span,
            contribution_kind,
            contribution,
        );
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
            &child_style,
            input.constants,
            child_flow_axes,
        )
        .map_err(|status| crate::compute::value_resolution_error(leaf.node, status))?;
        let member = ancestor_baseline_member(AncestorBaselineMemberInput {
            source: leaf.node,
            axis: input.axis,
            ancestor_span: leaf.ancestor_span,
            alignment: leaf.align_self,
            block_auto_margins,
            synthesized_baseline_cycle: synthesized_baseline_would_cycle(
                leaf.align_self,
                output.baselines(),
                child_flow_axes,
                span_tracks,
            ),
            output,
            margin,
            child_flow_axes,
            containing_flow_axes: input.constants.flow_axes,
            start_adjustment: baseline_adjustments.start,
            end_adjustment: baseline_adjustments.end,
        });
        if let Some(member) = member {
            ancestor_baseline_members.push(member);
        }
        flattened_contributions.push((flattened, member));
    }

    if input.axis == GridAxisKind::Column {
        for (index, (child, area)) in input
            .children
            .iter()
            .copied()
            .zip(input.placed_areas.iter().copied())
            .enumerate()
        {
            let Some(area) = area else {
                continue;
            };
            let child_style = tree.node_input(child).clone();
            if !is_in_flow_grid_child(&child_style)
                || input.subgrid_report.items.get(index).is_some_and(|item| {
                    item_inherits_parent_axis(&child_style, *item, GridAxisKind::Column)
                })
            {
                continue;
            }
            let alignment = child_style
                .justify_self
                .or(input.container_style.justify_items)
                .unwrap_or(AlignItems::Stretch);
            if !matches!(alignment, AlignItems::Baseline | AlignItems::LastBaseline) {
                continue;
            }
            let (start, end) = (area.column, area.column_end);
            let Some(span_tracks) = input.tracks.get(start..end) else {
                continue;
            };
            let output = tree.compute_child(
                child,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    input.constants.node_inner_size,
                    crate::ContainingLayoutContext::new(
                        input.constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    input.available,
                ),
            )?;
            let margin = intrinsic_contribution_margin(
                &child_style,
                input.constants.flow_axes,
                input.constants.node_inner_size,
            )
            .map_err(|status| crate::compute::value_resolution_error(child, status))?;
            let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
            let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
                &child_style,
                input.constants,
                child_flow_axes,
            )
            .map_err(|status| crate::compute::value_resolution_error(child, status))?;
            if let Some(member) = ancestor_baseline_member(AncestorBaselineMemberInput {
                source: child,
                axis: GridAxisKind::Column,
                ancestor_span: GridTrackSpan::new(start + 1, end + 1),
                alignment,
                block_auto_margins,
                synthesized_baseline_cycle: synthesized_baseline_would_cycle(
                    alignment,
                    output.baselines(),
                    child_flow_axes,
                    span_tracks,
                ),
                output,
                margin,
                child_flow_axes,
                containing_flow_axes: input.constants.flow_axes,
                start_adjustment: Tree::Scalar::ZERO,
                end_adjustment: Tree::Scalar::ZERO,
            }) {
                ancestor_baseline_members.push(member);
            }
        }
    }

    let ancestor_baseline_group = AncestorBaselineGroup::reduce(
        input.owner,
        input.axis,
        grid_axis_physical_axis(input.constants.flow_axes, input.axis),
        input.tracks.len(),
        ancestor_baseline_members,
    );
    let mut row_contributions = Vec::new();
    for (flattened, member) in flattened_contributions {
        let start = flattened.ancestor_span.start - 1;
        let end = flattened.ancestor_span.end - 1;
        let Some(span_tracks) = input.tracks.get(start..end) else {
            continue;
        };
        let shim = member.map_or(BaselineShim::default(), |member| {
            ancestor_baseline_group.intrinsic_shim(member)
        });
        let contribution = flattened.contribution + shim.before + shim.after;
        if input.axis == GridAxisKind::Row {
            row_contributions.push(RowIntrinsicContribution {
                start,
                end,
                contributes_to_row_size: true,
                contribution_kind: flattened.contribution_kind,
                contribution: flattened.contribution,
                baseline_member: member,
            });
        } else if end == start + 1 {
            input.sizes[start] = input.sizes[start].max(contribution);
        } else if axis_available(input.available, input.axis) == AvailableOf::MIN_CONTENT
            && span_tracks.iter().any(track_has_percent_sizing)
            && span_tracks
                .iter()
                .all(|track| track_flex_factor(track).is_none())
        {
            let child_style = tree.node_input(flattened.source);
            distribute_min_content_span_with_percent(
                &mut input.sizes[start..end],
                span_tracks,
                grid_axis_used_overflow(child_style, input.constants.flow_axes, input.axis),
                input.percent_basis,
                contribution,
            );
        } else {
            distribute_intrinsic_span(
                &mut input.sizes[start..end],
                span_tracks,
                flattened.contribution_kind,
                input.percent_basis,
                span_contribution_with_gutters(contribution, start, end, input.gap, input.gutters),
            );
        }
    }
    Ok(SubgridIntrinsicContributionReport {
        contributing_roots,
        row_contributions,
        ancestor_baseline_group,
        baseline_views,
    })
}

fn scroll_container_auto_minimum_zero<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> bool {
    scroll_container_auto_minimum_zero_for_grid_axis(style, flow_axes, axis)
}

fn subgrid_leaf_size_depends_on_queried_axis<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> bool {
    grid_axis_size(flow_axes, style.size.clone(), axis).depends_on_basis()
}

pub(super) fn item_inherits_parent_axis<Node, S: LayoutScalar>(
    style: &NodeInputOf<S>,
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

fn track_components_request_subgrid<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axis: GridAxisKind,
) -> bool {
    let components = match axis {
        GridAxisKind::Column => &style.grid_template_columns,
        GridAxisKind::Row => &style.grid_template_rows,
    };

    components
        .iter()
        .any(|component| matches!(component, TrackComponentOf::Subgrid(_)))
}

fn axis_margin_sum<S: LayoutScalar>(margin: Edges<S>, axis: GridAxisKind) -> S {
    match axis {
        GridAxisKind::Column => margin.horizontal_sum(),
        GridAxisKind::Row => margin.vertical_sum(),
    }
}

fn axis_available<S: LayoutScalar>(
    available: Size<AvailableOf<S>>,
    axis: GridAxisKind,
) -> AvailableOf<S> {
    match axis {
        GridAxisKind::Column => available.width,
        GridAxisKind::Row => available.height,
    }
}

pub(super) fn grid_axis_used_overflow<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> UsedOverflowAxis {
    let overflow = UsedOverflow::from_computed(style.overflow, style.item_is_replaced);
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => overflow.x(),
        crate::geometry::PhysicalAxis::Vertical => overflow.y(),
    }
}

pub(super) fn grid_axis_computed_overflow<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> Overflow {
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => style.overflow.x(),
        crate::geometry::PhysicalAxis::Vertical => style.overflow.y(),
    }
}

fn grid_axis_physical_axis(
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> crate::geometry::PhysicalAxis {
    match axis.logical_axis() {
        crate::LogicalAxis::Inline => flow_axes.inline_axis(),
        crate::LogicalAxis::Block => flow_axes.block_axis(),
    }
}

pub(super) fn grid_axis_size<T>(flow_axes: FlowAxes, size: Size<T>, axis: GridAxisKind) -> T {
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => size.width,
        crate::geometry::PhysicalAxis::Vertical => size.height,
    }
}

fn grid_axis_intrinsic_contribution_size<S: LayoutScalar>(
    style: &NodeInputOf<S>,
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

fn scroll_container_auto_minimum_zero_for_grid_axis<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> bool {
    grid_axis_computed_overflow(style, flow_axes, axis).is_scrollable()
        && grid_axis_size(flow_axes, style.size.clone(), axis).is_auto()
}

pub(super) fn constrained_row_intrinsic_sizes<Tree, M>(
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
    let subgrid_contributions = if grid
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
                owner: node,
                constants: grid.constants,
                container_style: grid.style,
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
                core::iter::empty::<AncestorBaselineMember<<Tree as Traverse>::Node, Tree::Scalar>>(
                ),
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
        if scroll_container_auto_minimum_zero_for_grid_axis(
            &child_style,
            grid.sizing_flow_axes,
            GridAxisKind::Row,
        ) {
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
        let physical_area_size = grid_area_physical_size(grid.constants.flow_axes, area.size);
        let sizing = grid_item_sizing_for_grid_flow::<Tree, M>(
            tree,
            child,
            &child_style,
            grid.style,
            physical_area_size,
            physical_area_size.map(Some),
            grid.sizing_flow_axes,
        )?;
        let margin = intrinsic_contribution_margin(
            &child_style,
            grid.constants.flow_axes,
            physical_area_size.map(Some),
        )
        .map_err(|status| crate::compute::value_resolution_error(child, status))?;
        let logical_sizing_known = grid.sizing_flow_axes.logical_size(sizing.known);
        let logical_sizing_available = grid.sizing_flow_axes.logical_size(sizing.available);
        let output = compute_intrinsic_grid_child(
            tree,
            child,
            IntrinsicGridChildInput {
                child_style: &child_style,
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
            &child_style,
            grid.constants,
            child_flow_axes,
        )
        .map_err(|status| crate::compute::value_resolution_error(child, status))?;
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
                &child_style,
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

pub(super) fn constrained_column_intrinsic_sizes<Tree, M>(
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
        if scroll_container_auto_minimum_zero_for_grid_axis(
            &child_style,
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
            &child_style,
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
                &child_style,
                grid.sizing_flow_axes,
                output.size,
                output.content_size,
                GridAxisKind::Column,
            ) + margin.inline_sum(),
        );
    }

    Ok(column_sizes)
}

#[derive(Clone, Copy)]
pub(super) struct PercentTrackContent<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) style: &'a NodeInputOf<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) sizing_flow_axes: FlowAxes,
    pub(super) parent_context: &'a GridParentContext<S, Node>,
    pub(super) column_tracks: &'a [TrackSizingOf<S>],
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) column_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) row_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) lines: GridLines,
    pub(super) placements: &'a GridPlacementContext<Node>,
}

pub(super) fn cyclic_percent_track_content_size<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: PercentTrackContent<'_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let PercentTrackContent {
        style,
        constants,
        sizing_flow_axes,
        parent_context,
        column_tracks,
        row_tracks,
        columns,
        rows,
        gap,
        column_gutters,
        row_gutters,
        lines,
        placements,
    } = input;

    let logical_node_inner_size = sizing_flow_axes.logical_size(constants.node_inner_size);
    if logical_node_inner_size.inline.is_some() && logical_node_inner_size.block.is_some() {
        return Ok(Size::ZERO);
    }

    let children = tree.children(node).collect::<Vec<_>>();
    let column_geometry = column_gutters
        .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(columns.to_vec(), gutters));
    let row_geometry = row_gutters
        .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(rows.to_vec(), gutters));
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
        column_geometry.as_ref(),
        row_geometry.as_ref(),
    );
    let column_offsets = column_gutters.map_or_else(
        || offsets(columns, Tree::Scalar::ZERO, gap.inline),
        |gutters| {
            UsedGridAxisGeometryOf::from_sizing_gutters(columns.to_vec(), gutters)
                .line_offsets()
                .to_vec()
        },
    );
    let row_offsets = row_gutters.map_or_else(
        || offsets(rows, Tree::Scalar::ZERO, gap.block),
        |gutters| {
            UsedGridAxisGeometryOf::from_sizing_gutters(rows.to_vec(), gutters)
                .line_offsets()
                .to_vec()
        },
    );
    let mut content_size = LogicalSizeOf::new(Tree::Scalar::ZERO, Tree::Scalar::ZERO);
    let accumulate_standalone_percent_columns =
        inherits_opposite_subgrid_axis(parent_context, GridAxisKind::Column);
    let accumulate_standalone_percent_rows =
        inherits_opposite_subgrid_axis(parent_context, GridAxisKind::Row);
    let mut column_content: Vec<Tree::Scalar> = vec![Tree::Scalar::ZERO; columns.len()];
    let mut row_content: Vec<Tree::Scalar> = vec![Tree::Scalar::ZERO; rows.len()];
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
        let spans_percent_column = logical_node_inner_size.inline.is_none()
            && { column_span.iter().any(track_has_percent_sizing) }
            && !column_span.iter().any(track_accepts_intrinsic_contribution);
        let spans_percent_row = logical_node_inner_size.block.is_none()
            && { row_span.iter().any(track_has_percent_sizing) }
            && !row_span.iter().any(track_accepts_intrinsic_contribution);
        if !spans_percent_column && !spans_percent_row {
            continue;
        }

        let output = tree.compute_child(
            child,
            ComputeInputOf::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(
                    constants.node_inner_size.width,
                    constants.node_inner_size.height,
                ),
                crate::ContainingLayoutContext::new(
                    constants.flow_axes,
                    crate::ParentFormattingContext::Grid,
                ),
                Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
            ),
        )?;
        let output_size = sizing_flow_axes.logical_size(output.size);
        let output_content_size = sizing_flow_axes.logical_size(output.content_size);
        if spans_percent_column {
            let contribution = axis_content_contribution(
                column_offsets[area.column],
                output_size.inline,
                output_content_size.inline,
                grid_axis_used_overflow(&child_style, sizing_flow_axes, GridAxisKind::Column),
            );
            content_size.inline = content_size.inline.max(contribution);
            if accumulate_standalone_percent_columns
                && area.column_end == area.column + 1
                && let Some(size) = column_content.get_mut(area.column)
            {
                *size = (*size).max(contribution);
            }
        }
        if spans_percent_row {
            let contribution = axis_content_contribution(
                row_offsets[area.row],
                output_size.block,
                output_content_size.block,
                grid_axis_used_overflow(&child_style, sizing_flow_axes, GridAxisKind::Row),
            );
            content_size.block = content_size.block.max(contribution);
            if accumulate_standalone_percent_rows
                && area.row_end == area.row + 1
                && let Some(size) = row_content.get_mut(area.row)
            {
                *size = (*size).max(contribution);
            }
        }
    }

    if accumulate_standalone_percent_columns {
        content_size.inline = content_size.inline.max(track_sum_with_gutters(
            &column_content,
            gap.inline,
            column_gutters,
        ));
    }
    if accumulate_standalone_percent_rows {
        content_size.block =
            content_size
                .block
                .max(track_sum_with_gutters(&row_content, gap.block, row_gutters));
    }

    Ok(sizing_flow_axes.physical_size(content_size))
}

fn inherits_opposite_subgrid_axis<Node, S: LayoutScalar>(
    parent_context: &GridParentContext<S, Node>,
    axis: GridAxisKind,
) -> bool {
    // Additive standalone percent sizing is only for grids that actually inherit
    // the opposite subgrid axis; raw fallback `subgrid` declarations stay ordinary grids.
    match axis {
        GridAxisKind::Column => parent_context.rows.is_some(),
        GridAxisKind::Row => parent_context.columns.is_some(),
    }
}

fn axis_content_contribution<S: LayoutScalar>(
    location: S,
    size: S,
    content_size: S,
    overflow: UsedOverflowAxis,
) -> S {
    let contribution_size = if overflow.value() == Overflow::Visible {
        size.max(content_size)
    } else {
        size
    };
    if contribution_size <= S::ZERO {
        return S::ZERO;
    }
    let max = (location + contribution_size).max(S::ZERO);
    let min = location.min(S::ZERO);
    max - min
}

pub(super) fn track_has_percent_sizing<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    track.depends_on_basis()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IntrinsicSpanContribution {
    MinContent { prioritize_min_tracks: bool },
    MaxContent,
}

impl IntrinsicSpanContribution {
    const fn for_axis<S: LayoutScalar>(
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

pub(super) fn distribute_intrinsic_span<S: LayoutScalar>(
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

fn scalar_total_cmp<S: LayoutScalar>(left: S, right: S) -> core::cmp::Ordering {
    left.to_f64().total_cmp(&right.to_f64())
}

pub(super) fn distribute_intrinsic_extra<S: LayoutScalar>(
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

pub(super) fn distribute_min_content_span_with_percent<S: LayoutScalar>(
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

pub(super) fn intrinsic_span_distribution_indexes<S: LayoutScalar>(
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

pub(super) fn intrinsic_span_non_percent_contribution<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    contribution: S,
) -> S {
    (contribution - track_basis_dependent_space(tracks, contribution)).max(S::ZERO)
}

pub(super) fn track_basis_dependent_space<S: LayoutScalar>(
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

pub(super) fn intrinsic_span_distribution_count<S: LayoutScalar>(
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

pub(super) fn intrinsic_span_definite_space<S: LayoutScalar>(
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

pub(super) fn intrinsic_span_definite_track_space<S: LayoutScalar>(
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

pub(super) fn intrinsic_span_minimum_floor_space<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
) -> S {
    tracks
        .iter()
        .filter(|track| !track_has_percent_sizing(track) && track_flex_factor(track).is_none())
        .map(|track| track_min_floor_space(track))
        .fold(S::ZERO, |sum, value| sum + value)
}

pub(super) fn track_min_floor_space<S: LayoutScalar>(track: &TrackSizingOf<S>) -> S {
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

pub(super) fn span_contribution<S: LayoutScalar>(contribution: S, span: usize, gap: S) -> S {
    (contribution - gap * S::from_usize(span.saturating_sub(1))).max(S::ZERO)
}

pub(super) fn track_accepts_intrinsic_contribution<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
) -> bool {
    track.min.is_intrinsic() || track.max.is_intrinsic()
}

pub(super) fn track_has_definite_min_floor<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    match &track.min {
        MinTrackSizingOf::Calculation(calculation) => {
            resolve_track_calculation_optional(calculation, None).is_some()
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => {
            false
        }
    }
}

pub(super) fn track_accepts_min_content_span_priority<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
) -> bool {
    matches!(track.min, MinTrackSizingOf::MinContent)
        || matches!(track.max, MaxTrackSizingOf::MinContent)
}

pub(super) fn track_accepts_max_content_span_priority<S: LayoutScalar>(
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

pub(super) fn track_accepts_auto_span_priority<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    matches!(track.min, MinTrackSizingOf::Auto) || matches!(track.max, MaxTrackSizingOf::Auto)
}

pub(super) fn track_accepts_percent_min_content_span<S: LayoutScalar>(
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

pub(super) fn intrinsic_contribution_margin<S: LayoutScalar>(
    style: &NodeInputOf<S>,
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

pub(super) fn resolve_lanes_tracks<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    intrinsic_sizes: &[S],
) -> Vec<S> {
    let gap_total = gap * S::from_usize(tracks.len().saturating_sub(1));
    let base_sizes = tracks
        .iter()
        .enumerate()
        .map(|(index, track)| match track.max {
            MaxTrackSizingOf::Flex(_) => {
                track_min_size(&track.min, basis, intrinsic_at(intrinsic_sizes, index))
            }
            _ => track_base_size(track, basis, intrinsic_at(intrinsic_sizes, index)),
        })
        .collect::<Vec<_>>();
    let auto_count = tracks
        .iter()
        .filter(|track| {
            matches!(
                track,
                TrackSizingOf {
                    min: MinTrackSizingOf::Auto,
                    max: MaxTrackSizingOf::Auto
                }
            )
        })
        .count();
    let fr_size = resolve_flex_fraction(tracks, &base_sizes, basis.map(|basis| basis - gap_total));
    let flex_used = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            track_flex_factor(track).map(|factor| base_sizes[index].max(factor * fr_size))
        })
        .fold(S::ZERO, |sum, value| sum + value);
    let fixed_sum = tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| track_flex_factor(track).is_none())
        .map(|(index, _)| base_sizes[index])
        .fold(S::ZERO, |sum, value| sum + value);
    let auto_size = if alignment == AlignContent::Stretch && auto_count > 0 {
        basis
            .map(|basis| {
                ((basis - gap_total - fixed_sum - flex_used).max(S::ZERO))
                    / S::from_usize(auto_count)
            })
            .unwrap_or(S::ZERO)
    } else {
        S::ZERO
    };

    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| match track {
            TrackSizingOf {
                max: MaxTrackSizingOf::Flex(value),
                ..
            } => base_sizes[index].max(value.get() * fr_size),
            TrackSizingOf {
                min: MinTrackSizingOf::Auto,
                max: MaxTrackSizingOf::Auto,
            } => intrinsic_at(intrinsic_sizes, index) + auto_size,
            track => {
                let intrinsic = intrinsic_at(intrinsic_sizes, index);
                let base = base_sizes[index];
                let min = track_growth_floor(track, basis, intrinsic);
                track_growth_limit(track, basis, intrinsic)
                    .map(|limit| base.min(limit.max(min)))
                    .unwrap_or(base)
            }
        })
        .collect()
}

pub(super) fn track_growth_floor<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    intrinsic: S,
) -> S {
    match &track.min {
        MinTrackSizingOf::Auto => S::ZERO,
        min => track_min_size(min, basis, intrinsic),
    }
}

pub(super) fn resolve_flex_fraction<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    base_sizes: &[S],
    space_to_fill: Option<S>,
) -> S {
    if !tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizingOf::Flex(_)))
    {
        return S::ZERO;
    }

    if let Some(space_to_fill) = space_to_fill {
        return find_size_of_fr(tracks, base_sizes, space_to_fill.max(S::ZERO));
    }

    let flex_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            track_flex_factor(track).map(|factor| {
                if factor > S::ONE {
                    base_sizes[index] / factor
                } else {
                    base_sizes[index]
                }
            })
        })
        .fold(S::ZERO, S::max);
    let occupied_sub_one_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            let factor = track_flex_factor(track)?;
            (base_sizes.get(index).copied().unwrap_or(S::ZERO) > S::ZERO && factor < S::ONE)
                .then_some(factor)
        })
        .fold(S::ZERO, |sum, value| sum + value);

    if occupied_sub_one_fraction > S::ZERO && occupied_sub_one_fraction < S::ONE {
        flex_fraction * occupied_sub_one_fraction
    } else {
        flex_fraction
    }
}

pub(super) fn find_size_of_fr<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    base_sizes: &[S],
    space_to_fill: S,
) -> S {
    if space_to_fill <= S::ZERO {
        return S::ZERO;
    }

    let mut hypothetical = S::INFINITY;
    loop {
        let previous = hypothetical;
        let mut used_space = S::ZERO;
        let mut flex_sum = S::ZERO;
        for (index, track) in tracks.iter().enumerate() {
            if let Some(factor) = track_flex_factor(track)
                && factor * hypothetical >= base_sizes[index]
            {
                flex_sum = flex_sum + factor;
            } else {
                used_space = used_space + base_sizes[index];
            }
        }

        hypothetical = (space_to_fill - used_space) / flex_sum.max(S::ONE);
        let valid = tracks.iter().enumerate().all(|(index, track)| {
            if let Some(factor) = track_flex_factor(track) {
                factor * hypothetical >= base_sizes[index] || factor * previous < base_sizes[index]
            } else {
                true
            }
        });
        if valid {
            return hypothetical.max(S::ZERO);
        }
    }
}

pub(super) fn track_flex_factor<S: LayoutScalar>(track: &TrackSizingOf<S>) -> Option<S> {
    if let MaxTrackSizingOf::Flex(value) = &track.max {
        Some(value.get())
    } else {
        None
    }
}

fn track_has_auto_maximum<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    matches!(track.max, MaxTrackSizingOf::Auto)
}

pub(super) fn resolve_lanes_inline_tracks<S: LayoutScalar>(
    input: InlineTrackInput<'_, S>,
) -> Vec<S> {
    let InlineTrackInput {
        tracks,
        basis,
        definite_size,
        available_size,
        gap,
        alignment,
        stretch_empty_auto_to_available,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        ..
    } = input;

    let max_tracks = resolve_lanes_tracks_with_intrinsics(
        tracks,
        basis,
        gap,
        AlignContent::Start,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
    );
    let min_tracks =
        resolve_track_min_bounds(tracks, basis, min_intrinsic_sizes, max_intrinsic_sizes);
    let max_content = track_sum(&max_tracks, gap);
    let min_content = track_sum(&min_tracks, gap);

    if let Some(available_size) = definite_size.or(available_size)
        && max_content > S::ZERO
        && available_size < max_content
    {
        let target = available_size.max(min_content).min(max_content);
        return distribute_tracks_between_bounds(&min_tracks, &max_tracks, gap, target);
    }

    if tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizingOf::FitContent(_)))
    {
        return resolve_fit_content_tracks(
            tracks,
            basis.or(available_size),
            min_intrinsic_sizes,
            max_intrinsic_sizes,
        );
    }

    resolve_lanes_tracks_with_intrinsics(
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
    )
}

fn stretch_empty_auto_track_basis<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    available_size: Option<S>,
    alignment: AlignContent,
    enabled: bool,
    max_intrinsic_sizes: &[S],
) -> Option<S> {
    if !enabled || alignment != AlignContent::Stretch {
        return None;
    }

    let has_empty_auto_track = tracks.iter().enumerate().any(|(index, track)| {
        matches!(
            track,
            TrackSizingOf {
                min: MinTrackSizingOf::Auto,
                max: MaxTrackSizingOf::Auto
            }
        ) && intrinsic_at(max_intrinsic_sizes, index) == S::ZERO
    });
    let has_non_auto_track = tracks.iter().any(|track| {
        !matches!(
            track,
            TrackSizingOf {
                min: MinTrackSizingOf::Auto,
                max: MaxTrackSizingOf::Auto
            }
        )
    });

    (has_empty_auto_track && has_non_auto_track)
        .then_some(available_size)
        .flatten()
}

pub(super) fn resolve_track_min_bounds<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
) -> Vec<S> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let intrinsic = match track.min {
                MinTrackSizingOf::MaxContent => intrinsic_at(max_intrinsic_sizes, index),
                _ => intrinsic_at(min_intrinsic_sizes, index),
            };
            track_min_size(&track.min, basis, intrinsic)
        })
        .collect()
}

pub(super) fn resolve_lanes_tracks_with_intrinsics<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
) -> Vec<S> {
    let gap_total = gap * S::from_usize(tracks.len().saturating_sub(1));
    let base_sizes = tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let min_intrinsic = intrinsic_at(min_intrinsic_sizes, index);
            let max_intrinsic = intrinsic_at(max_intrinsic_sizes, index);
            match track.max {
                MaxTrackSizingOf::Flex(_) => max_intrinsic.max(track_min_size_for_intrinsics(
                    &track.min,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                )),
                _ => track_base_size_for_intrinsics(track, basis, min_intrinsic, max_intrinsic),
            }
        })
        .collect::<Vec<_>>();
    let auto_count = tracks
        .iter()
        .filter(|track| {
            matches!(
                track,
                TrackSizingOf {
                    min: MinTrackSizingOf::Auto,
                    max: MaxTrackSizingOf::Auto
                }
            )
        })
        .count();
    let fr_size = resolve_flex_fraction(tracks, &base_sizes, basis.map(|basis| basis - gap_total));
    let flex_used = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            track_flex_factor(track).map(|factor| base_sizes[index].max(factor * fr_size))
        })
        .fold(S::ZERO, |sum, value| sum + value);
    let fixed_sum = tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| track_flex_factor(track).is_none())
        .map(|(index, _)| base_sizes[index])
        .fold(S::ZERO, |sum, value| sum + value);
    let auto_size = if alignment == AlignContent::Stretch && auto_count > 0 {
        basis
            .map(|basis| {
                ((basis - gap_total - fixed_sum - flex_used).max(S::ZERO))
                    / S::from_usize(auto_count)
            })
            .unwrap_or(S::ZERO)
    } else {
        S::ZERO
    };

    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let min_intrinsic = intrinsic_at(min_intrinsic_sizes, index);
            let max_intrinsic = intrinsic_at(max_intrinsic_sizes, index);
            match track {
                TrackSizingOf {
                    max: MaxTrackSizingOf::Flex(value),
                    ..
                } => base_sizes[index].max(value.get() * fr_size),
                TrackSizingOf {
                    min: MinTrackSizingOf::Auto,
                    max: MaxTrackSizingOf::Auto,
                } => max_intrinsic + auto_size,
                track => {
                    let base = base_sizes[index];
                    let min = track_growth_floor_for_intrinsics(
                        track,
                        basis,
                        min_intrinsic,
                        max_intrinsic,
                    );
                    track_growth_limit_for_intrinsics(track, basis, min_intrinsic, max_intrinsic)
                        .map(|limit| base.min(limit.max(min)))
                        .unwrap_or(base)
                }
            }
        })
        .collect()
}

#[derive(Clone)]
struct OrdinaryTrackState<'a, S: LayoutScalar> {
    sizing_functions: &'a TrackSizingOf<S>,
    base_size: S,
    growth_limit: Option<S>,
    fit_content_limit: Option<S>,
    flex_factor: Option<S>,
    auto_max_stretch_eligible: bool,
    collapsed: bool,
}

impl<'a, S: LayoutScalar> OrdinaryTrackState<'a, S> {
    fn new(sizing_functions: &'a TrackSizingOf<S>, collapsed: bool) -> Self {
        Self {
            sizing_functions,
            base_size: S::ZERO,
            growth_limit: None,
            fit_content_limit: None,
            flex_factor: track_flex_factor(sizing_functions),
            auto_max_stretch_eligible: track_has_auto_maximum(sizing_functions),
            collapsed,
        }
    }

    fn apply_intrinsic_contributions(
        &mut self,
        basis: Option<S>,
        min_intrinsic: S,
        max_intrinsic: S,
    ) {
        if self.collapsed {
            self.base_size = S::ZERO;
            self.growth_limit = Some(S::ZERO);
            self.fit_content_limit = None;
            self.flex_factor = None;
            self.auto_max_stretch_eligible = false;
            return;
        }

        self.fit_content_limit = match &self.sizing_functions.max {
            MaxTrackSizingOf::FitContent(limit) => Some(resolution_or_fallback(
                resolve_track_calculation(limit, basis),
                max_intrinsic,
            )),
            _ => None,
        };
        self.base_size = match self.fit_content_limit {
            Some(limit) => max_intrinsic.min(min_intrinsic.max(limit)),
            None => match self.sizing_functions.max {
                MaxTrackSizingOf::Flex(_) => max_intrinsic.max(track_min_size_for_intrinsics(
                    &self.sizing_functions.min,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                )),
                _ => track_base_size_for_intrinsics(
                    self.sizing_functions,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                ),
            },
        };
        self.growth_limit = self
            .fit_content_limit
            .map(|limit| max_intrinsic.min(min_intrinsic.max(limit)))
            .or_else(|| {
                track_growth_limit_for_intrinsics(
                    self.sizing_functions,
                    basis,
                    min_intrinsic,
                    max_intrinsic,
                )
            });
        let floor = track_growth_floor_for_intrinsics(
            self.sizing_functions,
            basis,
            min_intrinsic,
            max_intrinsic,
        );
        if let Some(growth_limit) = self.growth_limit {
            self.base_size = self.base_size.min(growth_limit.max(floor));
        }
    }
}

fn ordinary_track_states<'a, S: LayoutScalar>(
    tracks: &'a [TrackSizingOf<S>],
    basis: Option<S>,
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Vec<OrdinaryTrackState<'a, S>> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let collapsed = gutters
                .and_then(|gutters| gutters.collapsed().get(index))
                .copied()
                .unwrap_or(false);
            let mut state = OrdinaryTrackState::new(track, collapsed);
            state.apply_intrinsic_contributions(
                basis,
                intrinsic_at(min_intrinsic_sizes, index),
                intrinsic_at(max_intrinsic_sizes, index),
            );
            state
        })
        .collect()
}

fn resolve_ordinary_track_phases<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Vec<S> {
    let gap_total = gutters.map_or_else(
        || gap * S::from_usize(tracks.len().saturating_sub(1)),
        OrdinaryGridAxisGuttersOf::active_gap_total,
    );
    let mut states = ordinary_track_states(
        tracks,
        basis,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    );
    let base_sizes = states
        .iter()
        .map(|state| state.base_size)
        .collect::<Vec<_>>();
    let fr_size = resolve_flex_fraction(tracks, &base_sizes, basis.map(|size| size - gap_total));
    for state in &mut states {
        if let Some(flex_factor) = state.flex_factor {
            state.base_size = state.base_size.max(flex_factor * fr_size);
        }
    }

    let flex_used = states
        .iter()
        .filter(|state| state.flex_factor.is_some())
        .map(|state| state.base_size)
        .fold(S::ZERO, |sum, value| sum + value);
    let fixed_sum = states
        .iter()
        .filter(|state| state.flex_factor.is_none())
        .map(|state| state.base_size)
        .fold(S::ZERO, |sum, value| sum + value);
    let auto_count = states
        .iter()
        .filter(|state| state.auto_max_stretch_eligible && !state.collapsed)
        .count();
    let auto_size = if alignment == AlignContent::Stretch && auto_count > 0 {
        basis
            .map(|size| {
                ((size - gap_total - fixed_sum - flex_used).max(S::ZERO))
                    / S::from_usize(auto_count)
            })
            .unwrap_or(S::ZERO)
    } else {
        S::ZERO
    };
    states
        .into_iter()
        .map(|state| {
            if state.auto_max_stretch_eligible {
                state.base_size + auto_size
            } else {
                state.base_size
            }
        })
        .collect()
}

#[cfg(test)]
pub(super) fn resolve_tracks<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    intrinsic_sizes: &[S],
) -> Vec<S> {
    resolve_tracks_with_gutters(tracks, basis, gap, alignment, intrinsic_sizes, None)
}

pub(super) fn resolve_tracks_with_gutters<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    alignment: AlignContent,
    intrinsic_sizes: &[S],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Vec<S> {
    resolve_ordinary_track_phases(
        tracks,
        basis,
        gap,
        alignment,
        intrinsic_sizes,
        intrinsic_sizes,
        gutters,
    )
}

pub(super) fn resolve_inline_tracks<S: LayoutScalar>(input: InlineTrackInput<'_, S>) -> Vec<S> {
    let InlineTrackInput {
        tracks,
        basis,
        definite_size,
        available_size,
        gap,
        alignment,
        stretch_empty_auto_to_available,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    } = input;

    let max_tracks = resolve_ordinary_track_phases(
        tracks,
        basis,
        gap,
        AlignContent::Start,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    );
    let mut min_tracks =
        resolve_track_min_bounds(tracks, basis, min_intrinsic_sizes, max_intrinsic_sizes);
    if let Some(gutters) = gutters {
        for (size, collapsed) in min_tracks.iter_mut().zip(gutters.collapsed()) {
            if *collapsed {
                *size = S::ZERO;
            }
        }
    }
    let max_content = track_sum_with_gutters(&max_tracks, gap, gutters);
    let min_content = track_sum_with_gutters(&min_tracks, gap, gutters);
    if let Some(available_size) = definite_size.or(available_size)
        && max_content > S::ZERO
        && available_size < max_content
    {
        let target = available_size.max(min_content).min(max_content);
        return distribute_tracks_between_bounds_with_gutters(
            &min_tracks,
            &max_tracks,
            gap,
            gutters,
            target,
        );
    }

    let phase_basis = basis.or_else(|| {
        stretch_empty_auto_track_basis(
            tracks,
            available_size,
            alignment,
            stretch_empty_auto_to_available,
            max_intrinsic_sizes,
        )
    });
    resolve_ordinary_track_phases(
        tracks,
        phase_basis,
        gap,
        alignment,
        min_intrinsic_sizes,
        max_intrinsic_sizes,
        gutters,
    )
}

pub(super) fn track_base_size_for_intrinsics<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    min_intrinsic: S,
    max_intrinsic: S,
) -> S {
    let min = track_min_size_for_intrinsics(&track.min, basis, min_intrinsic, max_intrinsic);
    let max_base = match &track.max {
        MaxTrackSizingOf::Calculation(calculation) => {
            resolution_or_else(resolve_track_calculation(calculation, basis), || {
                if calculation.depends_on_basis() {
                    max_intrinsic
                } else {
                    resolution_or_zero(resolve_track_calculation(calculation, None))
                }
            })
        }
        MaxTrackSizingOf::Flex(_) => S::ZERO,
        MaxTrackSizingOf::Auto | MaxTrackSizingOf::MaxContent => max_intrinsic,
        MaxTrackSizingOf::MinContent => min_intrinsic,
        MaxTrackSizingOf::FitContent(limit) => {
            let limit =
                resolution_or_fallback(resolve_track_calculation(limit, basis), max_intrinsic);
            max_intrinsic.min(limit)
        }
    };
    min.max(max_base)
}

pub(super) fn track_min_size_for_intrinsics<S: LayoutScalar>(
    min: &MinTrackSizingOf<S>,
    basis: Option<S>,
    min_intrinsic: S,
    max_intrinsic: S,
) -> S {
    match min {
        MinTrackSizingOf::Calculation(calculation) => {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MaxContent => max_intrinsic,
        MinTrackSizingOf::MinContent => min_intrinsic,
    }
}

pub(super) fn track_growth_floor_for_intrinsics<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    min_intrinsic: S,
    max_intrinsic: S,
) -> S {
    match &track.min {
        MinTrackSizingOf::Auto => S::ZERO,
        min => track_min_size_for_intrinsics(min, basis, min_intrinsic, max_intrinsic),
    }
}

pub(super) fn track_growth_limit_for_intrinsics<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    min_intrinsic: S,
    max_intrinsic: S,
) -> Option<S> {
    match &track.max {
        MaxTrackSizingOf::Calculation(calculation) | MaxTrackSizingOf::FitContent(calculation) => {
            resolution_optional(resolve_track_calculation(calculation, basis))
                .or_else(|| calculation.depends_on_basis().then_some(max_intrinsic))
        }
        MaxTrackSizingOf::MinContent => Some(min_intrinsic),
        MaxTrackSizingOf::MaxContent | MaxTrackSizingOf::Auto => Some(max_intrinsic),
        MaxTrackSizingOf::Flex(_) => None,
    }
}

pub(super) fn resolve_fit_content_tracks<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
) -> Vec<S> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| match &track.max {
            MaxTrackSizingOf::FitContent(limit) => {
                let min_content = intrinsic_at(min_intrinsic_sizes, index);
                let max_content = intrinsic_at(max_intrinsic_sizes, index);
                let limit =
                    resolution_or_fallback(resolve_track_calculation(limit, basis), max_content);
                max_content.min(min_content.max(limit))
            }
            _ => track_base_size_for_intrinsics(
                track,
                basis,
                intrinsic_at(min_intrinsic_sizes, index),
                intrinsic_at(max_intrinsic_sizes, index),
            ),
        })
        .collect()
}

pub(super) fn distribute_tracks_between_bounds<S: LayoutScalar>(
    min_tracks: &[S],
    max_tracks: &[S],
    gap: S,
    target: S,
) -> Vec<S> {
    distribute_tracks_between_bounds_with_gutters(min_tracks, max_tracks, gap, None, target)
}

fn distribute_tracks_between_bounds_with_gutters<S: LayoutScalar>(
    min_tracks: &[S],
    max_tracks: &[S],
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
    target: S,
) -> Vec<S> {
    let min_sum = track_sum_with_gutters(min_tracks, gap, gutters);
    let max_sum = track_sum_with_gutters(max_tracks, gap, gutters);
    if target <= min_sum {
        return min_tracks.to_vec();
    }
    if target >= max_sum {
        return max_tracks.to_vec();
    }

    let mut resolved = max_tracks.to_vec();
    let shrink = (max_sum - target).max(S::ZERO);
    let shrink_capacity = max_tracks
        .iter()
        .zip(min_tracks)
        .map(|(max, min)| (*max - *min).max(S::ZERO))
        .fold(S::ZERO, |sum, value| sum + value);
    if shrink_capacity == S::ZERO {
        return resolved;
    }

    let ratio = (shrink / shrink_capacity).min(S::ONE);
    for (index, resolved) in resolved.iter_mut().enumerate() {
        let capacity = (max_tracks[index] - min_tracks[index]).max(S::ZERO);
        *resolved = *resolved - capacity * ratio;
    }
    resolved
}

pub(super) fn extend_auto_tracks<S: LayoutScalar>(
    tracks: &mut Vec<TrackSizingOf<S>>,
    auto_tracks: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    required_count: usize,
) -> Result<(), LengthResolutionStatus<S>> {
    let auto_tracks = expand_track_components(auto_tracks, basis, gap, None)?;
    let mut index = 0;
    while tracks.len() < required_count {
        let track = if auto_tracks.is_empty() {
            TrackSizingOf::AUTO
        } else {
            auto_tracks[index].clone()
        };
        tracks.push(track);
        if !auto_tracks.is_empty() {
            index = (index + 1) % auto_tracks.len();
        }
    }
    Ok(())
}

pub(super) fn prepend_auto_tracks<S: LayoutScalar>(
    tracks: &mut Vec<TrackSizingOf<S>>,
    auto_tracks: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    required_count: usize,
    auto_fit_limit: Option<usize>,
) -> Result<(), LengthResolutionStatus<S>> {
    if required_count == 0 {
        return Ok(());
    }

    let auto_tracks = expand_track_components(auto_tracks, basis, gap, auto_fit_limit)?;
    let generated = if auto_tracks.is_empty() {
        vec![TrackSizingOf::AUTO; required_count]
    } else {
        (0..required_count)
            .map(|index| {
                let phase = (auto_tracks.len() + index + auto_tracks.len()
                    - required_count % auto_tracks.len())
                    % auto_tracks.len();
                auto_tracks[phase].clone()
            })
            .collect::<Vec<_>>()
    };
    tracks.splice(0..0, generated);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AutoRepeatTrackOrigin {
    pub(super) kind: TrackRepeat,
    pub(super) repeat_group: usize,
    pub(super) repetition_index: usize,
    pub(super) track_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ExpandedTrackOf<S: LayoutScalar = Scalar> {
    pub(super) sizing: TrackSizingOf<S>,
    pub(super) auto_repeat: Option<AutoRepeatTrackOrigin>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TrackExpansionOf<S: LayoutScalar = Scalar> {
    pub(super) tracks: Vec<ExpandedTrackOf<S>>,
}

impl<S: LayoutScalar> TrackExpansionOf<S> {
    pub(super) fn inherited(tracks: Vec<TrackSizingOf<S>>) -> Self {
        Self {
            tracks: tracks
                .into_iter()
                .map(|sizing| ExpandedTrackOf {
                    sizing,
                    auto_repeat: None,
                })
                .collect(),
        }
    }
}

pub(super) fn expand_track_components_with_origins<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    auto_fit_limit: Option<usize>,
) -> Result<TrackExpansionOf<S>, LengthResolutionStatus<S>> {
    if subgrid_components(components) {
        return Ok(TrackExpansionOf { tracks: Vec::new() });
    }

    validate_track_components(components, basis)?;
    let mut tracks = Vec::new();
    let mut auto_repeat_group = 0;
    let reserved = reserved_track_space(components, basis, gap);
    for component in components {
        match component {
            TrackComponentOf::Track(sizing) => tracks.push(ExpandedTrackOf {
                sizing: sizing.clone(),
                auto_repeat: None,
            }),
            TrackComponentOf::Repeat(repetition) => {
                let repeated_tracks = repetition.sizing_tracks();
                let count = match repetition.repeat() {
                    TrackRepeat::Count(count) => count.get(),
                    TrackRepeat::AutoFill => {
                        auto_repeat_count(&repeated_tracks, basis, gap, reserved)
                    }
                    TrackRepeat::AutoFit => {
                        auto_repeat_count(&repeated_tracks, basis, gap, reserved)
                            .min(auto_fit_limit.unwrap_or(usize::MAX))
                            .max(1)
                    }
                };
                let repeat_group = auto_repeat_group;
                let auto_repeat_kind = match repetition.repeat() {
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                        auto_repeat_group += 1;
                        Some(repetition.repeat())
                    }
                    TrackRepeat::Count(_) => None,
                };
                for repetition_index in 0..count {
                    tracks.extend(repeated_tracks.iter().cloned().enumerate().map(
                        |(track_index, sizing)| ExpandedTrackOf {
                            sizing,
                            auto_repeat: auto_repeat_kind.map(|kind| AutoRepeatTrackOrigin {
                                kind,
                                repeat_group,
                                repetition_index,
                                track_index,
                            }),
                        },
                    ));
                }
            }
            TrackComponentOf::LineNames(_) => {}
            TrackComponentOf::Subgrid(_) => {
                unreachable!("subgrid templates return before expansion")
            }
        }
    }
    Ok(TrackExpansionOf { tracks })
}

pub(super) fn expand_track_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    auto_fit_limit: Option<usize>,
) -> Result<Vec<TrackSizingOf<S>>, LengthResolutionStatus<S>> {
    Ok(
        expand_track_components_with_origins(components, basis, gap, auto_fit_limit)?
            .tracks
            .into_iter()
            .map(|track| track.sizing)
            .collect(),
    )
}

fn validate_track_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    for component in components {
        match component {
            TrackComponentOf::Track(track) => validate_track_sizing(track, basis)?,
            TrackComponentOf::Repeat(repetition) => {
                validate_track_components(repetition.components(), basis)?;
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    Ok(())
}

fn validate_track_sizing<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    if let MinTrackSizingOf::Calculation(calculation) = &track.min {
        validate_track_calculation(calculation, basis)?;
    }
    match &track.max {
        MaxTrackSizingOf::Calculation(calculation) | MaxTrackSizingOf::FitContent(calculation) => {
            validate_track_calculation(calculation, basis)?;
        }
        MaxTrackSizingOf::Flex(_)
        | MaxTrackSizingOf::Auto
        | MaxTrackSizingOf::MinContent
        | MaxTrackSizingOf::MaxContent => {}
    }
    Ok(())
}

fn validate_track_calculation<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    let resolution = resolve_track_calculation(calculation, basis);
    match resolution.status() {
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::Resolved | LengthResolutionStatus::MissingBasis => Ok(()),
        LengthResolutionStatus::NonNumeric => Err(LengthResolutionStatus::NonNumeric),
    }
}

pub(super) fn track_expansion_basis<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    node_basis: Option<S>,
    available_basis: Option<S>,
) -> Option<S> {
    if subgrid_components(components) {
        return None;
    }

    node_basis.or_else(|| {
        auto_repeat_components(components)
            .then_some(available_basis)
            .flatten()
    })
}

pub(super) fn subgrid_components<S: LayoutScalar>(components: &[TrackComponentOf<S>]) -> bool {
    components
        .iter()
        .any(|component| matches!(component, TrackComponentOf::Subgrid(_)))
}

pub(super) fn auto_repeat_components<S: LayoutScalar>(components: &[TrackComponentOf<S>]) -> bool {
    components.iter().any(|component| {
        matches!(
            component,
            TrackComponentOf::Repeat(repetition)
                if matches!(repetition.repeat(), TrackRepeat::AutoFill | TrackRepeat::AutoFit)
        )
    })
}

pub(super) fn tracks_need_available_basis<S: LayoutScalar>(tracks: &[TrackSizingOf<S>]) -> bool {
    tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizingOf::Flex(_)))
}

#[derive(Clone, Copy)]
pub(super) struct ReservedTrackSpace<S: LayoutScalar = Scalar> {
    pub(super) count: usize,
    pub(super) size: S,
}

pub(super) fn reserved_track_space<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
) -> ReservedTrackSpace<S> {
    let mut count = 0;
    let mut size = S::ZERO;
    for component in components {
        match component {
            TrackComponentOf::Track(track) => {
                count += 1;
                size = size + track_base_size(track, basis, S::ZERO);
            }
            TrackComponentOf::Repeat(repetition) => {
                if let TrackRepeat::Count(repeat_count) = repetition.repeat() {
                    let repeated_tracks = repetition.sizing_tracks();
                    count += repeat_count.get() * repeated_tracks.len();
                    size = size
                        + S::from_usize(repeat_count.get())
                            * repeated_tracks
                                .iter()
                                .map(|track| track_base_size(track, basis, S::ZERO))
                                .fold(S::ZERO, |sum, value| sum + value);
                }
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }

    if count > 1 {
        size = size + gap * S::from_usize(count - 1);
    }

    ReservedTrackSpace { count, size }
}

pub(super) fn auto_repeat_count<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    reserved: ReservedTrackSpace<S>,
) -> usize {
    let Some(basis) = basis else {
        return 1;
    };
    if tracks.is_empty() {
        return 0;
    }
    let track_sum = tracks
        .iter()
        .map(|track| track_base_size(track, Some(basis), S::ZERO).max(S::ONE))
        .fold(S::ZERO, |sum, value| sum + value);
    let repeat_size = track_sum + gap * S::from_usize(tracks.len());
    if repeat_size <= S::ZERO {
        1
    } else {
        let available = if reserved.count == 0 {
            basis + gap
        } else {
            basis - reserved.size
        };
        (available / repeat_size)
            .floor()
            .max(S::ONE)
            .floor_to_usize_saturating()
    }
}

pub(super) fn intrinsic_at<S: LayoutScalar>(intrinsic_sizes: &[S], index: usize) -> S {
    intrinsic_sizes.get(index).copied().unwrap_or(S::ZERO)
}

pub(super) fn track_resolution_intrinsic_sizes<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    min_intrinsic_sizes: &[S],
    max_intrinsic_sizes: &[S],
) -> Vec<S> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            if track.min == MinTrackSizingOf::MaxContent
                || match &track.max {
                    MaxTrackSizingOf::Auto
                    | MaxTrackSizingOf::Flex(_)
                    | MaxTrackSizingOf::MaxContent => true,
                    MaxTrackSizingOf::Calculation(calculation) => calculation.depends_on_basis(),
                    MaxTrackSizingOf::FitContent(_) | MaxTrackSizingOf::MinContent => false,
                }
            {
                intrinsic_at(max_intrinsic_sizes, index)
            } else if track.min == MinTrackSizingOf::MinContent
                || track.max == MaxTrackSizingOf::MinContent
            {
                intrinsic_at(min_intrinsic_sizes, index)
            } else {
                intrinsic_at(max_intrinsic_sizes, index)
            }
        })
        .collect()
}

pub(super) fn track_base_size<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    intrinsic: S,
) -> S {
    let min = track_min_size(&track.min, basis, intrinsic);
    let max_base = match &track.max {
        MaxTrackSizingOf::Calculation(calculation) => {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MaxTrackSizingOf::Flex(_) => S::ZERO,
        MaxTrackSizingOf::Auto | MaxTrackSizingOf::MinContent | MaxTrackSizingOf::MaxContent => {
            intrinsic
        }
        MaxTrackSizingOf::FitContent(limit) => {
            let limit = resolution_or_fallback(resolve_track_calculation(limit, basis), intrinsic);
            intrinsic.min(limit)
        }
    };
    min.max(max_base)
}

pub(super) fn track_min_size<S: LayoutScalar>(
    min: &MinTrackSizingOf<S>,
    basis: Option<S>,
    intrinsic: S,
) -> S {
    match min {
        MinTrackSizingOf::Calculation(calculation) => {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => {
            intrinsic
        }
    }
}

pub(super) fn track_growth_limit<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    intrinsic: S,
) -> Option<S> {
    match &track.max {
        MaxTrackSizingOf::Calculation(calculation) => {
            resolution_optional(resolve_track_calculation(calculation, basis))
        }
        MaxTrackSizingOf::FitContent(limit) => {
            let min = track_min_size(&track.min, basis, intrinsic);
            Some(intrinsic.max(min).min(resolution_or_fallback(
                resolve_track_calculation(limit, basis),
                intrinsic,
            )))
        }
        MaxTrackSizingOf::Flex(_)
        | MaxTrackSizingOf::Auto
        | MaxTrackSizingOf::MinContent
        | MaxTrackSizingOf::MaxContent => None,
    }
}

fn resolution_or_zero<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> S {
    resolution_or_fallback(resolution, S::ZERO)
}

fn resolution_or_fallback<S: LayoutScalar>(resolution: LengthResolutionOf<S>, fallback: S) -> S {
    resolution_or_else(resolution, || fallback)
}

fn resolution_or_else<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
    fallback: impl FnOnce() -> S,
) -> S {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution
            .value
            .expect("resolved length resolution must carry a value"),
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => fallback(),
    }
}

fn resolution_optional<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> Option<S> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution.value,
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => None,
    }
}

pub(super) fn track_sum<S: LayoutScalar>(sizes: &[S], gap: S) -> S {
    sizes
        .iter()
        .copied()
        .fold(S::ZERO, |sum, value| sum + value)
        + gap * S::from_usize(sizes.len().saturating_sub(1))
}

pub(super) fn track_sum_with_gutters<S: LayoutScalar>(
    sizes: &[S],
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    sizes
        .iter()
        .copied()
        .fold(S::ZERO, |sum, value| sum + value)
        + gutters.map_or_else(
            || gap * S::from_usize(sizes.len().saturating_sub(1)),
            OrdinaryGridAxisGuttersOf::active_gap_total,
        )
}

pub(super) fn track_span_sum_with_gutters<S: LayoutScalar>(
    sizes: &[S],
    start: usize,
    end: usize,
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    if start >= end || end > sizes.len() {
        return S::ZERO;
    }
    sizes[start..end]
        .iter()
        .copied()
        .fold(S::ZERO, |sum, size| sum + size)
        + gutters.map_or_else(
            || gap * S::from_usize(end.saturating_sub(start + 1)),
            |gutters| gutters.span_gap_total(start, end),
        )
}

fn span_contribution_with_gutters<S: LayoutScalar>(
    contribution: S,
    start: usize,
    end: usize,
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    let gutter_total = gutters.map_or_else(
        || gap * S::from_usize(end.saturating_sub(start + 1)),
        |gutters| gutters.span_gap_total(start, end),
    );
    (contribution - gutter_total).max(S::ZERO)
}

pub(super) fn track_content_sum<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    sizes: &[S],
    gap: S,
) -> S {
    track_sum(sizes, gap) + sub_one_flex_unfilled_space(tracks, sizes)
}

pub(super) fn track_content_sum_with_gutters<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    sizes: &[S],
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    track_sum_with_gutters(sizes, gap, gutters) + sub_one_flex_unfilled_space(tracks, sizes)
}

fn sub_one_flex_unfilled_space<S: LayoutScalar>(tracks: &[TrackSizingOf<S>], sizes: &[S]) -> S {
    let flex_fraction = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            let factor =
                track_flex_factor(track).filter(|factor| *factor > S::ZERO && *factor < S::ONE)?;
            let size = sizes.get(index).copied().unwrap_or(S::ZERO);
            (size > S::ZERO).then_some(size / factor)
        })
        .min_by(|left, right| scalar_total_cmp(*left, *right));

    let Some(flex_fraction) = flex_fraction else {
        return S::ZERO;
    };

    let mut occupied_fraction = S::ZERO;
    for (index, track) in tracks.iter().enumerate() {
        let Some(factor) =
            track_flex_factor(track).filter(|factor| *factor > S::ZERO && *factor < S::ONE)
        else {
            continue;
        };
        let size = sizes.get(index).copied().unwrap_or(S::ZERO);
        if size > factor * flex_fraction + S::from_f64(0.001) {
            occupied_fraction = occupied_fraction + factor;
        }
    }

    if occupied_fraction > S::ZERO && occupied_fraction < S::ONE {
        flex_fraction * (S::ONE - occupied_fraction)
    } else {
        S::ZERO
    }
}

pub(super) fn offsets<S: LayoutScalar>(sizes: &[S], start: S, gap: S) -> Vec<S> {
    let mut cursor = start;
    sizes
        .iter()
        .map(|size| {
            let offset = cursor;
            cursor = cursor + *size + gap;
            offset
        })
        .collect()
}

#[cfg(test)]
pub(super) fn rtl_offsets<S: LayoutScalar>(
    sizes: &[S],
    content_box_left: S,
    content_box_width: S,
    start: S,
    gap: S,
) -> Vec<S> {
    if content_box_width <= S::ZERO {
        return vec![content_box_left; sizes.len()];
    }

    let mut cursor = content_box_left + content_box_width - start;
    sizes
        .iter()
        .map(|size| {
            cursor = cursor - *size;
            let offset = cursor;
            cursor = cursor - gap;
            offset
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::layout_tree::OracleTree;
    use crate::{
        Length, LengthPercentageOf, NodeInput, SizingCalculationOf, TrackComponent, TrackSizing,
    };

    fn fri04_c03_grid_track_nested<S: LayoutScalar>(target: f64) -> SizingCalculationOf<S> {
        let value = |value| {
            SizingCalculationOf::value(
                LengthPercentageOf::px(S::from_f64(value)).expect("finite test track value"),
            )
        };
        SizingCalculationOf::clamp(
            Some(value(target - 10.0)),
            SizingCalculationOf::max(vec![
                value(target),
                SizingCalculationOf::min(vec![value(target - 5.0), value(target + 5.0)])
                    .expect("nested minimum is nonempty"),
            ])
            .expect("nested maximum is nonempty"),
            Some(value(target + 10.0)),
        )
    }

    #[test]
    fn fri04_c03_grid_track_exact_static_span_accepts_basis_independent_nested_programs() {
        let tracks = [
            TrackSizingOf::calculation(fri04_c03_grid_track_nested::<f64>(30.0)),
            TrackSizingOf::calculation(fri04_c03_grid_track_nested::<f64>(40.0)),
        ];
        assert_eq!(exact_static_track_span(&tracks, 0, 2, 7.0), Some(77.0));

        let dependent = SizingCalculationOf::max(vec![
            fri04_c03_grid_track_nested::<f64>(30.0),
            SizingCalculationOf::value(
                LengthPercentageOf::from_percent_fraction(0.1).expect("finite test percentage"),
            ),
        ])
        .expect("nested maximum is nonempty");
        assert_eq!(
            exact_static_track_span(&[TrackSizingOf::calculation(dependent)], 0, 1, 0.0),
            None
        );
    }

    #[test]
    fn intrinsic_subgrid_constraints_distinguish_final_authority_from_unknown_spans() {
        fn axis_report(parent_axis: GridAxisKind, eligible: bool) -> SubgridAxisReport {
            SubgridAxisReport {
                mapping: Ok(GridAxisMappingReport {
                    queried_axis: parent_axis,
                    parent_axis,
                    child_axis: parent_axis,
                    reversed: false,
                }),
                eligibility: SubgridEligibility {
                    eligible,
                    reason: (!eligible).then_some(SubgridIneligibleReason::NotRequested),
                },
            }
        }

        fn assert_lanes<S: LayoutScalar>() {
            let column_constraint =
                intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
                    report: axis_report(GridAxisKind::Column, true),
                    area: GridArea {
                        column: 0,
                        column_end: 2,
                        row: 0,
                        row_end: 1,
                        size: LogicalSizeOf::new(S::ZERO, S::ZERO),
                    },
                    parent_flow_axes: crate::geometry::FlowAxes::new(
                        crate::WritingMode::VerticalRl,
                        crate::Direction::Ltr,
                    ),
                    child_flow_axes: crate::geometry::FlowAxes::new(
                        crate::WritingMode::VerticalRl,
                        crate::Direction::Ltr,
                    ),
                    explicit_parent_content_size: Size::new(
                        Some(S::from_f64(41.0)),
                        Some(S::from_f64(97.0)),
                    ),
                    parent_column_count: 2,
                    parent_row_count: 1,
                    tracks: &[
                        TrackSizingOf::px(S::from_f64(30.0)),
                        TrackSizingOf::px(S::from_f64(40.0)),
                    ],
                    gap: S::from_f64(7.0),
                })
                .expect("eligible full-span column subgrid has a constraint");
            assert_eq!(
                column_constraint,
                IntrinsicSubgridAxisConstraint {
                    physical_axis: crate::geometry::PhysicalAxis::Vertical,
                    authority: IntrinsicSubgridAxisAuthority::FinalContainerContent(S::from_f64(
                        97.0
                    )),
                }
            );

            let row_constraint =
                intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
                    report: axis_report(GridAxisKind::Row, true),
                    area: GridArea {
                        column: 0,
                        column_end: 1,
                        row: 1,
                        row_end: 2,
                        size: LogicalSizeOf::new(S::ZERO, S::ZERO),
                    },
                    parent_flow_axes: crate::geometry::FlowAxes::new(
                        crate::WritingMode::VerticalRl,
                        crate::Direction::Ltr,
                    ),
                    child_flow_axes: crate::geometry::FlowAxes::new(
                        crate::WritingMode::VerticalRl,
                        crate::Direction::Ltr,
                    ),
                    explicit_parent_content_size: Size::new(
                        Some(S::from_f64(41.0)),
                        Some(S::from_f64(97.0)),
                    ),
                    parent_column_count: 1,
                    parent_row_count: 2,
                    tracks: &[TrackSizingOf::AUTO, TrackSizingOf::px(S::from_f64(40.0))],
                    gap: S::from_f64(7.0),
                })
                .expect("eligible partial-span row subgrid has a constraint");
            assert_eq!(
                row_constraint,
                IntrinsicSubgridAxisConstraint {
                    physical_axis: crate::geometry::PhysicalAxis::Horizontal,
                    authority: IntrinsicSubgridAxisAuthority::FinalTrackSpan(S::from_f64(40.0)),
                }
            );

            let unknown = intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
                report: axis_report(GridAxisKind::Row, true),
                area: GridArea {
                    column: 0,
                    column_end: 1,
                    row: 1,
                    row_end: 2,
                    size: LogicalSizeOf::new(S::ZERO, S::ZERO),
                },
                parent_flow_axes: crate::geometry::FlowAxes::new(
                    crate::WritingMode::VerticalRl,
                    crate::Direction::Ltr,
                ),
                child_flow_axes: crate::geometry::FlowAxes::new(
                    crate::WritingMode::VerticalRl,
                    crate::Direction::Ltr,
                ),
                explicit_parent_content_size: Size::new(
                    Some(S::from_f64(41.0)),
                    Some(S::from_f64(97.0)),
                ),
                parent_column_count: 1,
                parent_row_count: 2,
                tracks: &[TrackSizingOf::AUTO, TrackSizingOf::AUTO],
                gap: S::from_f64(7.0),
            })
            .expect("eligible partial-span row subgrid has a constraint");
            assert_eq!(unknown.authority, IntrinsicSubgridAxisAuthority::Unknown);
        }

        assert_lanes::<f32>();
        assert_lanes::<f64>();
    }

    #[test]
    fn nested_orthogonal_partial_subgrids_keep_intrinsic_axes_provisional() {
        fn axis_report(parent_axis: GridAxisKind, child_axis: GridAxisKind) -> SubgridAxisReport {
            SubgridAxisReport {
                mapping: Ok(GridAxisMappingReport {
                    queried_axis: child_axis,
                    parent_axis,
                    child_axis,
                    reversed: false,
                }),
                eligibility: SubgridEligibility {
                    eligible: true,
                    reason: None,
                },
            }
        }

        fn assert_lanes<S: LayoutScalar>() {
            let horizontal_style = NodeInput::default();
            let horizontal =
                FlowAxes::new(horizontal_style.writing_mode, horizontal_style.direction);
            let vertical = FlowAxes::new(crate::WritingMode::VerticalLr, crate::Direction::Ltr);
            let outer_constraint = IntrinsicSubgridAxisConstraint {
                physical_axis: inherited_subgrid_physical_axis(
                    axis_report(GridAxisKind::Column, GridAxisKind::Row),
                    horizontal,
                    vertical,
                )
                .expect("mapped outer partial subgrid has a physical axis"),
                authority: IntrinsicSubgridAxisAuthority::Unknown,
            };
            let outer = intrinsic_subgrid_child_input(
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::new(None, Some(S::from_f64(19.0))),
                    Size::new(None, Some(S::from_f64(29.0))),
                    crate::ContainingLayoutContext::new(
                        horizontal,
                        crate::ParentFormattingContext::Grid,
                    ),
                    Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MIN_CONTENT),
                ),
                GridItemSizing {
                    known: Size::new(Some(S::from_f64(31.0)), Some(S::from_f64(37.0))),
                    available: Size::new(S::from_f64(41.0), S::from_f64(43.0)),
                    unresolved_margin: Edges::ZERO.map(Some),
                    justify_self: AlignItems::Stretch,
                    align_self: AlignItems::Stretch,
                },
                [Some(outer_constraint), None],
            );
            assert_eq!(outer.known().width, None);
            assert_eq!(outer.parent().width, None);
            assert_eq!(outer.available().width, AvailableOf::MAX_CONTENT);

            let inner_constraint = IntrinsicSubgridAxisConstraint {
                physical_axis: inherited_subgrid_physical_axis(
                    axis_report(GridAxisKind::Row, GridAxisKind::Column),
                    vertical,
                    horizontal,
                )
                .expect("mapped inner partial subgrid has a physical axis"),
                authority: IntrinsicSubgridAxisAuthority::Unknown,
            };
            let inner = intrinsic_subgrid_child_input(
                outer,
                GridItemSizing {
                    known: Size::new(Some(S::from_f64(47.0)), Some(S::from_f64(53.0))),
                    available: Size::new(S::from_f64(59.0), S::from_f64(61.0)),
                    unresolved_margin: Edges::ZERO.map(Some),
                    justify_self: AlignItems::Stretch,
                    align_self: AlignItems::Stretch,
                },
                [Some(inner_constraint), None],
            );
            assert_eq!(inner.known().width, None);
            assert_eq!(inner.parent().width, None);
            assert_eq!(inner.available().width, AvailableOf::MAX_CONTENT);
        }

        assert_lanes::<f32>();
        assert_lanes::<f64>();
    }

    #[test]
    fn vertical_intrinsic_grid_percentage_edges_use_logical_area_basis() {
        let parent_style = NodeInput {
            display: Display::Grid,
            writing_mode: crate::WritingMode::VerticalRl,
            justify_items: Some(AlignItems::Start),
            align_items: Some(AlignItems::Start),
            ..NodeInput::default()
        };
        let child_style = NodeInput {
            display: Display::Grid,
            writing_mode: crate::WritingMode::VerticalRl,
            grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack {
                name_components: Vec::new(),
            })],
            padding: Edges::all(Length::percent(0.1)),
            ..NodeInput::default()
        };
        let mut tree = OracleTree::new()
            .children(2, [])
            .style(2, child_style.clone());
        let constants = Constants {
            flow_axes: crate::geometry::FlowAxes::new(
                parent_style.writing_mode,
                parent_style.direction,
            ),
            explicit_definite_content_size: Size::new(Some(100.0), Some(200.0)),
            node_outer_size: Size::new(Some(100.0), Some(200.0)),
            node_inner_size: Size::new(Some(100.0), Some(200.0)),
            node_min_size: Size::NONE,
            node_max_size: Size::NONE,
            available_inner_size: Size::new(Some(100.0), Some(200.0)),
            content_box_inset: Edges::ZERO,
            padding: Edges::ZERO,
            border: Edges::ZERO,
        };
        let area = GridArea {
            column: 0,
            column_end: 1,
            row: 0,
            row_end: 1,
            size: LogicalSizeOf::new(200.0, 100.0),
        };
        let columns = [200.0];
        let rows = [100.0];
        let named_columns = NamedGridLines::new(GridAxisKind::Column, 1);
        let named_rows = NamedGridLines::new(GridAxisKind::Row, 1);
        let placements = GridPlacementContext::new(Vec::<u32>::new(), Vec::new());
        let subgrid_report = GridSubgridReport { items: Vec::new() };
        let subgrid_item = SubgridItemReport {
            node: 2,
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        };
        let output = compute_intrinsic_grid_child(
            &mut tree,
            2,
            IntrinsicGridChildInput {
                child_style: &child_style,
                grid: IntrinsicGrid {
                    style: &parent_style,
                    constants: &constants,
                    sizing_flow_axes: constants.flow_axes,
                    column_tracks: &[TrackSizing::px(200.0)],
                    row_tracks: &[TrackSizing::px(100.0)],
                    gap: LogicalSizeOf::new(0.0, 0.0),
                    column_gutters: None,
                    row_gutters: None,
                    percent_basis: LogicalSizeOf::new(None, None),
                    lines: GridLines {
                        column_explicit_start: 0,
                        column_explicit_count: 1,
                        row_explicit_start: 0,
                        row_explicit_count: 1,
                    },
                    named_columns: &named_columns,
                    named_rows: &named_rows,
                    area_facts: None,
                    subgrid_report: &subgrid_report,
                    placements: &placements,
                },
                area,
                columns: &columns,
                rows: &rows,
                subgrid_item: Some(subgrid_item),
                input: ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    Size::NONE,
                    crate::ContainingLayoutContext::new(
                        constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    Size::new(AvailableOf::definite(100.0), AvailableOf::definite(200.0)),
                ),
            },
        )
        .unwrap();

        assert_eq!(output.size, Size::new(40.0, 200.0));
    }

    #[test]
    fn vertical_intrinsic_subgrid_percentage_gap_uses_logical_content_box() {
        let parent_style = NodeInput {
            display: Display::Grid,
            writing_mode: crate::WritingMode::VerticalRl,
            justify_items: Some(AlignItems::Start),
            align_items: Some(AlignItems::Start),
            ..NodeInput::default()
        };
        let child_style = NodeInput {
            display: Display::Grid,
            writing_mode: crate::WritingMode::VerticalRl,
            grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack {
                name_components: Vec::new(),
            })],
            grid_template_rows: vec![TrackComponent::px(100.0)],
            gap: Size::new(Length::ZERO, Length::percent(0.1)),
            padding: Edges::all(Length::percent(0.1)),
            justify_items: Some(AlignItems::Start),
            align_items: Some(AlignItems::Start),
            ..NodeInput::default()
        };
        let mut tree = OracleTree::new()
            .children(2, [3, 4])
            .children(3, [])
            .children(4, [])
            .style(2, child_style.clone())
            .style(3, NodeInput::default())
            .style(4, NodeInput::default());
        let constants = Constants {
            flow_axes: crate::geometry::FlowAxes::new(
                parent_style.writing_mode,
                parent_style.direction,
            ),
            explicit_definite_content_size: Size::new(Some(100.0), Some(200.0)),
            node_outer_size: Size::new(Some(100.0), Some(200.0)),
            node_inner_size: Size::new(Some(100.0), Some(200.0)),
            node_min_size: Size::NONE,
            node_max_size: Size::NONE,
            available_inner_size: Size::new(Some(100.0), Some(200.0)),
            content_box_inset: Edges::ZERO,
            padding: Edges::ZERO,
            border: Edges::ZERO,
        };
        let area = GridArea {
            column: 0,
            column_end: 2,
            row: 0,
            row_end: 1,
            size: LogicalSizeOf::new(200.0, 100.0),
        };
        let columns = [100.0, 100.0];
        let rows = [100.0];
        let named_columns = NamedGridLines::new(GridAxisKind::Column, 2);
        let named_rows = NamedGridLines::new(GridAxisKind::Row, 1);
        let placements = GridPlacementContext::new(Vec::<u32>::new(), Vec::new());
        let subgrid_report = GridSubgridReport { items: Vec::new() };
        let subgrid_item = SubgridItemReport {
            node: 2,
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        };
        compute_intrinsic_grid_child(
            &mut tree,
            2,
            IntrinsicGridChildInput {
                child_style: &child_style,
                grid: IntrinsicGrid {
                    style: &parent_style,
                    constants: &constants,
                    sizing_flow_axes: constants.flow_axes,
                    column_tracks: &[TrackSizing::px(100.0), TrackSizing::px(100.0)],
                    row_tracks: &[TrackSizing::px(100.0)],
                    gap: LogicalSizeOf::new(0.0, 0.0),
                    column_gutters: None,
                    row_gutters: None,
                    percent_basis: LogicalSizeOf::new(None, None),
                    lines: GridLines {
                        column_explicit_start: 0,
                        column_explicit_count: 2,
                        row_explicit_start: 0,
                        row_explicit_count: 1,
                    },
                    named_columns: &named_columns,
                    named_rows: &named_rows,
                    area_facts: None,
                    subgrid_report: &subgrid_report,
                    placements: &placements,
                },
                area,
                columns: &columns,
                rows: &rows,
                subgrid_item: Some(subgrid_item),
                input: ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    Size::NONE,
                    crate::ContainingLayoutContext::new(
                        constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    Size::new(AvailableOf::definite(100.0), AvailableOf::definite(200.0)),
                ),
            },
        )
        .unwrap();

        assert_eq!(
            tree.layout(4)
                .expect("second subgrid item should be laid out")
                .location
                .y,
            108.0
        );
    }
}
