use super::*;

pub(super) type OwnerProgressionTrackFrameOrigins<S> = (Vec<S>, Vec<S>, Vec<S>, Vec<S>);
pub(super) type FinalAncestorBaselineGroupsLayoutResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    FinalAncestorBaselineGroups<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum BaselineGroupKind {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::grid) struct BaselineParticipation {
    pub(in crate::grid) participates: bool,
    pub(in crate::grid) group: Option<BaselineGroupKind>,
    pub(in crate::grid) synthesized: bool,
    pub(in crate::grid) fallback_alignment: Option<AlignItems>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct BaselineGeometry<S: LayoutScalar = Scalar> {
    pub(in crate::grid) available_span_size: S,
    pub(in crate::grid) margin_box_size: S,
    // Border-box baselines are stored separately on PendingGridItem. These
    // fields are the margin-box contributions used by shared baseline groups:
    // block-start margin plus first baseline for major groups, and block-end
    // margin plus distance from last baseline to block-end for minor groups.
    pub(in crate::grid) major_baseline: PhysicalBaseline<S>,
    pub(in crate::grid) minor_baseline: PhysicalBaseline<S>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::grid) struct TrackBaselineGroup<S: LayoutScalar = Scalar> {
    pub(in crate::grid) first: Option<PhysicalBaseline<S>>,
    pub(in crate::grid) last: Option<PhysicalBaseline<S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct GridBaselineGroups<S: LayoutScalar = Scalar> {
    pub(in crate::grid) rows: Vec<TrackBaselineGroup<S>>,
    pub(in crate::grid) columns: Vec<TrackBaselineGroup<S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct FinalAncestorBaselineGroups<Node, S: LayoutScalar = Scalar> {
    rows: AncestorBaselineGroup<Node, S>,
    columns: AncestorBaselineGroup<Node, S>,
    placement_rows: Option<InheritedGridOwnerBaselineTargets<Node, S>>,
    placement_columns: Option<InheritedGridOwnerBaselineTargets<Node, S>>,
    row_child_envelope: Option<ChildBaselineEnvelopeView<S>>,
    column_child_envelope: Option<ChildBaselineEnvelopeView<S>>,
    pub(super) row_downward_major_translation: Vec<S>,
    pub(super) row_downward_minor_translation: Vec<S>,
    pub(super) column_downward_major_translation: Vec<S>,
    pub(super) column_downward_minor_translation: Vec<S>,
}

impl<Node: Copy + PartialEq, S: LayoutScalar> FinalAncestorBaselineGroups<Node, S> {
    pub(super) fn with_parent_context(
        mut self,
        parent_context: &GridParentContext<S, Node>,
    ) -> Self {
        if let Some(rows) = &parent_context.rows {
            self.placement_rows = rows.owner_baseline_targets.as_ref().map(|targets| {
                InheritedGridOwnerBaselineTargets {
                    group: targets.group.clone(),
                    mapping: targets.mapping.clone(),
                }
            });
            self.row_child_envelope = Some(ChildBaselineEnvelopeView {
                major: rows.major_baselines.clone(),
                minor: rows.minor_baselines.clone(),
            });
            self.row_downward_major_translation.fill(S::ZERO);
            self.row_downward_minor_translation.fill(S::ZERO);
        }
        if let Some(columns) = &parent_context.columns {
            self.placement_columns = columns.owner_baseline_targets.as_ref().map(|targets| {
                InheritedGridOwnerBaselineTargets {
                    group: targets.group.clone(),
                    mapping: targets.mapping.clone(),
                }
            });
            self.column_child_envelope = Some(ChildBaselineEnvelopeView {
                major: columns.major_baselines.clone(),
                minor: columns.minor_baselines.clone(),
            });
            self.column_downward_major_translation.fill(S::ZERO);
            self.column_downward_minor_translation.fill(S::ZERO);
        }
        self
    }

    pub(super) fn placement_groups(&self) -> GridBaselineGroups<S> {
        let envelope_groups = |view: &ChildBaselineEnvelopeView<S>| {
            view.major
                .iter()
                .copied()
                .zip(view.minor.iter().copied())
                .map(|(first, last)| TrackBaselineGroup { first, last })
                .collect()
        };
        GridBaselineGroups {
            rows: if self.placement_rows.is_none() {
                self.row_child_envelope
                    .as_ref()
                    .map(&envelope_groups)
                    .unwrap_or_else(|| self.rows.track_groups())
            } else {
                self.rows.track_groups()
            },
            columns: self
                .placement_columns
                .as_ref()
                .map(|targets| targets.group.track_groups())
                .unwrap_or_else(|| self.columns.track_groups()),
        }
    }

    pub(super) fn for_axis(&self, axis: GridAxisKind) -> &AncestorBaselineGroup<Node, S> {
        match axis {
            GridAxisKind::Column => &self.columns,
            GridAxisKind::Row => &self.rows,
        }
    }

    pub(super) fn inherited_targets_for_axis(
        &self,
        axis: GridAxisKind,
    ) -> Option<&InheritedGridOwnerBaselineTargets<Node, S>> {
        match axis {
            GridAxisKind::Column => self.placement_columns.as_ref(),
            GridAxisKind::Row => self.placement_rows.as_ref(),
        }
    }

    pub(super) fn child_envelope_for_axis(
        &self,
        axis: GridAxisKind,
    ) -> Option<&ChildBaselineEnvelopeView<S>> {
        match axis {
            GridAxisKind::Column => self.column_child_envelope.as_ref(),
            GridAxisKind::Row => self.row_child_envelope.as_ref(),
        }
    }
}

#[cfg(test)]
pub(in crate::grid) fn final_ancestor_baseline_groups_for_transport_test<S: LayoutScalar>(
    rows: AncestorBaselineGroup<u32, S>,
    columns: AncestorBaselineGroup<u32, S>,
) -> FinalAncestorBaselineGroups<u32, S> {
    let row_track_count = rows.track_count();
    let column_track_count = columns.track_count();
    let placement_rows = Some(InheritedGridOwnerBaselineTargets {
        group: rows.clone(),
        mapping: CheckedOwnerToCurrentPlacementMap::identity(
            rows.owner(),
            GridAxisKind::Row,
            rows.physical_axis(),
            PhysicalProgression::Increasing,
            row_track_count,
        ),
    });
    let placement_columns = Some(InheritedGridOwnerBaselineTargets {
        group: columns.clone(),
        mapping: CheckedOwnerToCurrentPlacementMap::identity(
            columns.owner(),
            GridAxisKind::Column,
            columns.physical_axis(),
            PhysicalProgression::Increasing,
            column_track_count,
        ),
    });
    FinalAncestorBaselineGroups {
        rows,
        columns,
        placement_rows,
        placement_columns,
        row_child_envelope: None,
        column_child_envelope: None,
        row_downward_major_translation: vec![S::ZERO; row_track_count],
        row_downward_minor_translation: vec![S::ZERO; row_track_count],
        column_downward_major_translation: vec![S::ZERO; column_track_count],
        column_downward_minor_translation: vec![S::ZERO; column_track_count],
    }
}

#[cfg(test)]
pub(in crate::grid) fn final_ancestor_baseline_groups_with_parent_context_for_transport_test<
    S: LayoutScalar,
>(
    groups: FinalAncestorBaselineGroups<u32, S>,
    parent_context: &GridParentContext<S, u32>,
) -> FinalAncestorBaselineGroups<u32, S> {
    groups.with_parent_context(parent_context)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct GridContainerBaselines<S: LayoutScalar = Scalar> {
    pub(in crate::grid) baselines: BaselinesOf<S>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::grid) struct BaselineShim<S: LayoutScalar = Scalar> {
    pub(in crate::grid) before: S,
    pub(in crate::grid) after: S,
}

#[cfg(test)]
impl<S: LayoutScalar> GridBaselineGroups<S> {
    fn shared_baseline(
        &self,
        group_kind: BaselineGroupKind,
        area: GridArea<S>,
    ) -> Option<PhysicalBaseline<S>> {
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
pub(in crate::grid) fn baseline_shim_for_intrinsic_contribution<S: LayoutScalar>(
    participation: BaselineParticipation,
    geometry: BaselineGeometry<S>,
    shared: TrackBaselineGroup<S>,
    expected_axis: PhysicalAxis,
) -> BaselineShim<S> {
    if !participation.participates {
        return BaselineShim::default();
    }

    match participation.group {
        Some(BaselineGroupKind::Major) => BaselineShim {
            before: shared
                .first
                .and_then(|baseline| baseline.coordinate_on(expected_axis))
                .zip(geometry.major_baseline.coordinate_on(expected_axis))
                .map_or(S::ZERO, |(shared, item)| (shared - item).max(S::ZERO)),
            after: S::ZERO,
        },
        Some(BaselineGroupKind::Minor) => BaselineShim {
            before: S::ZERO,
            after: shared
                .last
                .and_then(|baseline| baseline.coordinate_on(expected_axis))
                .zip(geometry.minor_baseline.coordinate_on(expected_axis))
                .map_or(S::ZERO, |(shared, item)| (shared - item).max(S::ZERO)),
        },
        None => BaselineShim::default(),
    }
}

#[cfg(test)]
pub(in crate::grid) fn baseline_offset<S: LayoutScalar>(
    group_kind: BaselineGroupKind,
    shared_baseline: PhysicalBaseline<S>,
    geometry: BaselineGeometry<S>,
    expected_axis: PhysicalAxis,
) -> Option<S> {
    let shared_baseline = shared_baseline.coordinate_on(expected_axis)?;
    match group_kind {
        BaselineGroupKind::Major => geometry
            .major_baseline
            .coordinate_on(expected_axis)
            .map(|baseline| shared_baseline - baseline),
        BaselineGroupKind::Minor => {
            geometry
                .minor_baseline
                .coordinate_on(expected_axis)
                .map(|baseline| {
                    let baseline_delta = shared_baseline - baseline;
                    geometry.available_span_size - baseline_delta - geometry.margin_box_size
                })
        }
    }
}

#[cfg(test)]
pub(in crate::grid) fn baseline_aligned_block_offset<Node: Copy, S: LayoutScalar>(
    item: &PendingGridItem<Node, S>,
    groups: &GridBaselineGroups<S>,
    rows: &[S],
    row_gap: S,
    container_flow_axes: FlowAxes,
) -> Option<S> {
    if !item.baseline_participation.participates || item.block_auto_margins {
        return None;
    }

    let group_kind = item.baseline_participation.group?;
    let shared = groups.shared_baseline(group_kind, item.area)?;
    let margin_box_offset = baseline_offset(
        group_kind,
        shared,
        item.logical_baseline_geometry(rows, row_gap, container_flow_axes),
        container_flow_axes.block_axis(),
    )?;
    Some(margin_box_offset + container_flow_axes.logical_edges(item.margin).block_start)
}

#[cfg(test)]
pub(in crate::grid) fn spanned_track_size<S: LayoutScalar>(
    tracks: &[S],
    start: usize,
    end: usize,
    gap: S,
) -> S {
    let track_sum = tracks[start..end]
        .iter()
        .copied()
        .fold(S::ZERO, |sum, track| sum + track);
    let gap_sum = gap * S::from_usize(end.saturating_sub(start + 1));
    track_sum + gap_sum
}

pub(super) struct OwnerProgressionTrackFrameInput<'a, S: LayoutScalar = Scalar> {
    pub(super) parent_tracks: &'a [S],
    pub(super) parent_boundary_gutters: &'a [S],
    pub(super) parent_line_offsets: Option<&'a [S]>,
    pub(super) parent_span: GridTrackSpan,
    pub(super) current_tracks_before_gutter: &'a [S],
    pub(super) local_parent_boundary_gutters: &'a [S],
    pub(super) current_boundary_gutters: &'a [S],
    pub(super) reversed: bool,
    pub(super) start_mbp: S,
}

pub(super) fn owner_progression_track_frame_origins<S: LayoutScalar>(
    input: OwnerProgressionTrackFrameInput<'_, S>,
) -> Option<OwnerProgressionTrackFrameOrigins<S>> {
    let OwnerProgressionTrackFrameInput {
        parent_tracks,
        parent_boundary_gutters,
        parent_line_offsets,
        parent_span,
        current_tracks_before_gutter,
        local_parent_boundary_gutters,
        current_boundary_gutters,
        reversed,
        start_mbp,
    } = input;
    let track_positions = |tracks: &[S], boundary_gutters: &[S]| {
        if boundary_gutters.len() != tracks.len().saturating_sub(1) {
            return None;
        }
        let mut starts = Vec::with_capacity(tracks.len());
        let mut ends = Vec::with_capacity(tracks.len());
        let mut cursor = S::ZERO;
        for (index, track) in tracks.iter().copied().enumerate() {
            starts.push(cursor);
            cursor = cursor + track;
            ends.push(cursor);
            if let Some(gutter) = boundary_gutters.get(index) {
                cursor = cursor + *gutter;
            }
        }
        Some((starts, ends))
    };
    let (parent_starts, parent_ends) = if let Some(line_offsets) = parent_line_offsets {
        if line_offsets.len() != parent_tracks.len() + 1 {
            return None;
        }
        let starts = line_offsets[..parent_tracks.len()].to_vec();
        let ends = starts
            .iter()
            .copied()
            .zip(parent_tracks.iter().copied())
            .map(|(start, track)| start + track)
            .collect();
        (starts, ends)
    } else {
        track_positions(parent_tracks, parent_boundary_gutters)?
    };
    let track_count = parent_span.checked_len()?;
    if parent_span.end > parent_tracks.len() || current_tracks_before_gutter.len() != track_count {
        return None;
    }
    let span_start = *parent_starts.get(parent_span.start)?;
    let span_end = *parent_ends.get(parent_span.end.checked_sub(1)?)?;
    let (local_starts, _) =
        track_positions(current_tracks_before_gutter, current_boundary_gutters)?;
    let (_, local_ends) =
        track_positions(current_tracks_before_gutter, local_parent_boundary_gutters)?;

    let (parent_first, parent_last) = if reversed {
        (
            parent_ends.clone(),
            parent_starts
                .iter()
                .copied()
                .map(|origin| -origin)
                .collect(),
        )
    } else {
        (
            parent_starts
                .iter()
                .copied()
                .map(|origin| -origin)
                .collect(),
            parent_ends.clone(),
        )
    };
    let (current_first, current_last) = if reversed {
        let content_start = span_end - start_mbp;
        (
            local_starts
                .iter()
                .copied()
                .map(|origin| content_start - origin)
                .collect(),
            local_ends
                .iter()
                .copied()
                .map(|origin| -(content_start - origin))
                .collect(),
        )
    } else {
        (
            local_starts
                .iter()
                .copied()
                .map(|origin| -(span_start + start_mbp + origin))
                .collect(),
            local_ends
                .iter()
                .copied()
                .map(|origin| span_start + start_mbp + origin)
                .collect(),
        )
    };
    Some((parent_first, parent_last, current_first, current_last))
}

pub(super) struct BaselineAlignedAxisInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) item: &'a PendingGridItem<Node, S>,
    pub(super) child_style: &'a GridItemProjection<S>,
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) group: &'a AncestorBaselineGroup<Node, S>,
    pub(super) axis: GridAxisKind,
    pub(super) geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) subgrid_item: Option<SubgridItemReport<Node>>,
    pub(super) container_flow_axes: FlowAxes,
    pub(super) intrinsic_baseline_census: bool,
    pub(super) inherited_owner_targets: Option<&'a InheritedGridOwnerBaselineTargets<Node, S>>,
    pub(super) child_envelope: Option<&'a ChildBaselineEnvelopeView<S>>,
    pub(super) current_grid: Node,
}

pub(super) fn baseline_aligned_axis_offset<Node: Copy + PartialEq, S: LayoutScalar>(
    input: BaselineAlignedAxisInput<'_, Node, S>,
) -> Result<Option<S>, InheritedCurrentGridBaselinePlacementError> {
    let BaselineAlignedAxisInput {
        item,
        child_style,
        container_style,
        group,
        axis,
        geometry,
        row_tracks,
        subgrid_item,
        container_flow_axes,
        intrinsic_baseline_census,
        inherited_owner_targets,
        child_envelope,
        current_grid,
    } = input;
    if subgrid_item.is_some_and(|subgrid_item| {
        [subgrid_item.column, subgrid_item.row]
            .into_iter()
            .any(|report| report.can_inherit() && report.mapping.parent_axis == axis)
    }) {
        return Ok(None);
    }
    let alignment = match axis {
        GridAxisKind::Column => child_style.justify_self.or(container_style.justify_items),
        GridAxisKind::Row => child_style.align_self.or(container_style.align_items),
    }
    .unwrap_or(AlignItems::Stretch);
    let block_auto_margins = matches!(
        item.child_flow_axes.line_over_edge(child_style.margin),
        LengthAutoOf::Auto
    ) || matches!(
        item.child_flow_axes.line_under_edge(child_style.margin),
        LengthAutoOf::Auto
    );
    let (start, end) = match axis {
        GridAxisKind::Column => (item.area.column, item.area.column_end),
        GridAxisKind::Row => (item.area.row, item.area.row_end),
    };
    let synthesized_baseline_cycle = axis == GridAxisKind::Row
        && synthesized_baseline_would_cycle(
            alignment,
            item.output.baselines(),
            item.child_flow_axes,
            row_tracks.get(start..end).unwrap_or(&[]),
        );
    let member_input = |alignment| AncestorBaselineMemberInput {
        source: item.node,
        axis,
        ancestor_span: GridTrackSpan::new(start + 1, end + 1),
        alignment,
        block_auto_margins,
        synthesized_baseline_cycle,
        output: item.output,
        margin: item.margin,
        child_flow_axes: item.child_flow_axes,
        containing_flow_axes: container_flow_axes,
        start_adjustment: S::ZERO,
        end_adjustment: S::ZERO,
    };
    let Some(member) = ancestor_baseline_member(member_input(alignment)) else {
        return Ok(None);
    };
    let logical_margin = container_flow_axes.logical_edges(item.margin);
    let logical_size = container_flow_axes.logical_size(item.output.size);
    let (start_margin, end_margin, item_size) = match axis {
        GridAxisKind::Column => (
            logical_margin.inline_start,
            logical_margin.inline_end,
            logical_size.inline,
        ),
        GridAxisKind::Row => (
            logical_margin.block_start,
            logical_margin.block_end,
            logical_size.block,
        ),
    };
    let item_axis = match axis {
        GridAxisKind::Column => LogicalAxis::Inline,
        GridAxisKind::Row => LogicalAxis::Block,
    };
    let participation = baseline_participation(
        alignment,
        block_auto_margins,
        synthesized_baseline_cycle,
        item.output.baselines(),
        item.child_flow_axes,
    );
    if participation.synthesized
        && container_flow_axes.logical_axis_progression(item_axis)
            != item
                .child_flow_axes
                .logical_axis_progression(LogicalAxis::Block)
    {
        let opposite_alignment = match alignment {
            AlignItems::Baseline => AlignItems::LastBaseline,
            AlignItems::LastBaseline => AlignItems::Baseline,
            _ => return Ok(None),
        };
        let Some(opposite_member) = ancestor_baseline_member(member_input(opposite_alignment))
        else {
            return Ok(None);
        };
        return Ok(group.synthesized_opposite_placement_offset(
            member,
            opposite_member,
            geometry.span_extent(start, end),
            start_margin,
            end_margin,
        ));
    }
    let available_span_size = geometry.span_extent(start, end);
    let margin_box_size = item_size + start_margin + end_margin;
    let offset = if let Some(owner_targets) = inherited_owner_targets
        .filter(|targets| targets.mapping.owner() != targets.mapping.current_grid())
    {
        let placement = InheritedCurrentGridBaselinePlacement::try_derive(
            &owner_targets.group,
            InheritedCurrentGridBaselinePlacementInput {
                axis,
                physical_axis: grid_axis_physical_axis(container_flow_axes, axis),
                mapping: owner_targets.mapping.clone(),
                direct_witness: CurrentGridDirectWitness::new(
                    current_grid,
                    item.node,
                    axis,
                    GridTrackSpan::new(start, end),
                    member.role(),
                ),
                current_grid,
                item: item.node,
            },
        )?;
        group.placement_offset_for_target(
            member,
            placement.translated_target(),
            available_span_size,
            margin_box_size,
            start_margin,
        )
    } else if let Some(child_envelope) = child_envelope {
        let Some(target) = child_envelope.target_for(member) else {
            return Ok(None);
        };
        group.placement_offset_for_target(
            member,
            target,
            available_span_size,
            margin_box_size,
            start_margin,
        )
    } else {
        let Some(offset) =
            group.placement_offset(member, available_span_size, margin_box_size, start_margin)
        else {
            return Ok(None);
        };
        offset
    };
    let owner_direct_end_edge_correction = if axis == GridAxisKind::Column
        && inherited_owner_targets.is_some_and(|targets| {
            targets.mapping.owner() == targets.mapping.current_grid()
                && targets.mapping.current_progression().is_decreasing()
        }) {
        -end_margin
    } else {
        S::ZERO
    };
    let intrinsic_correction = if intrinsic_baseline_census
        && !inherited_owner_targets.is_some_and(|targets| {
            targets.mapping.owner() == targets.mapping.current_grid()
                && !targets.mapping.current_progression().is_decreasing()
        })
        && axis == GridAxisKind::Row
        && row_tracks
            .get(start..end)
            .is_some_and(|tracks| tracks.iter().any(track_accepts_intrinsic_contribution))
    {
        let shim = group.intrinsic_shim(member);
        match participation.group {
            Some(BaselineGroupKind::Major) => (shim.before - start_margin).max(S::ZERO),
            Some(BaselineGroupKind::Minor) => shim.after,
            None => S::ZERO,
        }
    } else {
        S::ZERO
    };
    Ok(Some(
        offset + owner_direct_end_edge_correction + intrinsic_correction,
    ))
}

impl<Node, S: LayoutScalar> PendingGridItem<Node, S> {
    #[cfg(test)]
    fn logical_baseline_geometry(
        &self,
        rows: &[S],
        row_gap: S,
        container_flow_axes: FlowAxes,
    ) -> BaselineGeometry<S> {
        self.logical_baseline_geometry_for_span(
            spanned_track_size(rows, self.area.row, self.area.row_end, row_gap),
            container_flow_axes,
        )
    }

    fn logical_baseline_geometry_for_span(
        &self,
        available_span_size: S,
        container_flow_axes: FlowAxes,
    ) -> BaselineGeometry<S> {
        let logical_margin = container_flow_axes.logical_edges(self.margin);
        let logical_size = container_flow_axes.logical_size(self.output.size);
        let first_baseline = logical_block_coordinate(
            container_flow_axes,
            self.first_baseline.coordinate(),
            self.output.size,
        );
        let last_baseline = logical_block_coordinate(
            container_flow_axes,
            self.last_baseline.coordinate(),
            self.output.size,
        );
        BaselineGeometry {
            available_span_size,
            margin_box_size: logical_size.block
                + logical_margin.block_start
                + logical_margin.block_end,
            major_baseline: PhysicalBaseline::new(
                container_flow_axes.block_axis(),
                logical_margin.block_start + first_baseline,
            ),
            minor_baseline: PhysicalBaseline::new(
                container_flow_axes.block_axis(),
                logical_margin.block_end + logical_size.block - last_baseline,
            ),
        }
    }
}

fn logical_block_coordinate<S: LayoutScalar>(
    flow_axes: FlowAxes,
    coordinate: S,
    physical_size: Size<S>,
) -> S {
    if flow_axes
        .logical_axis_progression(LogicalAxis::Block)
        .is_decreasing()
    {
        flow_axes.block_axis_extent(physical_size) - coordinate
    } else {
        coordinate
    }
}

struct DirectAncestorBaselineMembers<Node, S: LayoutScalar = Scalar> {
    columns: Vec<AncestorBaselineMember<Node, S>>,
    rows: Vec<AncestorBaselineMember<Node, S>>,
}

fn direct_ancestor_baseline_members<Tree, M>(
    _tree: &Tree,
    container_style: &GridContainerProjection<'_, Tree::Scalar>,
    row_tracks: &[TrackSizingOf<Tree::Scalar>],
    items: &[PendingGridItem<<Tree as Traverse>::Node, Tree::Scalar>],
    subgrid_report: &GridSubgridReport<<Tree as Traverse>::Node>,
    container_flow_axes: FlowAxes,
) -> DirectAncestorBaselineMembers<<Tree as Traverse>::Node, Tree::Scalar>
where
    Tree: Compute<M>,
{
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    for item in items {
        let style = &item.style;
        let subgrid_item = subgrid_report.items.get(item.source_index).copied();
        let block_auto_margins = matches!(
            item.child_flow_axes.line_over_edge(style.margin),
            LengthAutoOf::Auto
        ) || matches!(
            item.child_flow_axes.line_under_edge(style.margin),
            LengthAutoOf::Auto
        );
        for (axis, members) in [
            (GridAxisKind::Column, &mut columns),
            (GridAxisKind::Row, &mut rows),
        ] {
            if subgrid_item.is_some_and(|subgrid_item| {
                [subgrid_item.column, subgrid_item.row]
                    .into_iter()
                    .any(|report| report.can_inherit() && report.mapping.parent_axis == axis)
            }) {
                continue;
            }
            let alignment = match axis {
                GridAxisKind::Column => style.justify_self.or(container_style.justify_items),
                GridAxisKind::Row => style.align_self.or(container_style.align_items),
            }
            .unwrap_or(AlignItems::Stretch);
            let (start, end) = match axis {
                GridAxisKind::Column => (item.area.column, item.area.column_end),
                GridAxisKind::Row => (item.area.row, item.area.row_end),
            };
            let synthesized_baseline_cycle = if axis == GridAxisKind::Row {
                synthesized_baseline_would_cycle(
                    alignment,
                    item.output.baselines(),
                    item.child_flow_axes,
                    row_tracks.get(start..end).unwrap_or(&[]),
                )
            } else {
                false
            };
            let member = ancestor_baseline_member(AncestorBaselineMemberInput {
                source: item.node,
                axis,
                ancestor_span: GridTrackSpan::new(start + 1, end + 1),
                alignment,
                block_auto_margins,
                synthesized_baseline_cycle,
                output: item.output,
                margin: item.margin,
                child_flow_axes: item.child_flow_axes,
                containing_flow_axes: container_flow_axes,
                start_adjustment: Tree::Scalar::ZERO,
                end_adjustment: Tree::Scalar::ZERO,
            });
            members.extend(member);
        }
    }
    DirectAncestorBaselineMembers { columns, rows }
}

pub(super) struct FinalAncestorBaselineGroupsInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) constants: &'a Constants<S>,
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) children: &'a [Node],
    pub(super) placed_areas: &'a [Option<GridArea<S>>],
    pub(super) placements: &'a GridPlacementContext<Node, S>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) named_columns: &'a NamedGridLines,
    pub(super) named_rows: &'a NamedGridLines,
    pub(super) area_facts: Option<&'a GridAreaNameFacts>,
}

pub(super) fn final_ancestor_baseline_groups<Tree, M>(
    tree: &mut Tree,
    input: FinalAncestorBaselineGroupsInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
    items: &[PendingGridItem<<Tree as Traverse>::Node, Tree::Scalar>],
) -> FinalAncestorBaselineGroupsLayoutResult<Tree, M>
where
    Tree: Compute<M>,
{
    let row_intrinsic_min_track_facts = input
        .row_tracks
        .iter()
        .map(|track| track.min.is_intrinsic())
        .collect::<Vec<_>>();
    let definite_available = LogicalSizeOf::new(
        AvailableOf::Definite(input.column_geometry.total_extent()),
        AvailableOf::Definite(input.row_geometry.total_extent()),
    );
    let direct_members = direct_ancestor_baseline_members::<Tree, M>(
        tree,
        input.container_style,
        input.row_tracks,
        items,
        input.subgrid_report,
        input.constants.flow_axes,
    );
    let column_available = input.constants.flow_axes.physical_size(definite_available);
    let columns = ancestor_baseline_group_for_final_placement(
        tree,
        FinalAncestorBaselineGroupInput {
            owner: input.node,
            constants: input.constants,
            axis: GridAxisKind::Column,
            track_count: input.columns.len(),
            gap: input.gap,
            available: column_available,
            children: input.children,
            placed_areas: input.placed_areas,
            placements: input.placements,
            subgrid_report: input.subgrid_report,
            named_columns: input.named_columns,
            named_rows: input.named_rows,
            area_facts: input.area_facts,
            column_sizes: input.columns,
            row_sizes: input.rows,
            column_geometry: input.column_geometry,
            row_geometry: input.row_geometry,
            intrinsic_min_track_facts: None,
            direct_members: direct_members.columns,
        },
    )?;
    let row_available = input.constants.flow_axes.physical_size(LogicalSizeOf::new(
        definite_available.inline,
        if input
            .row_tracks
            .iter()
            .any(track_accepts_intrinsic_contribution)
        {
            AvailableOf::MAX_CONTENT
        } else {
            definite_available.block
        },
    ));
    let rows = ancestor_baseline_group_for_final_placement(
        tree,
        FinalAncestorBaselineGroupInput {
            owner: input.node,
            constants: input.constants,
            axis: GridAxisKind::Row,
            track_count: input.rows.len(),
            gap: input.gap,
            available: row_available,
            children: input.children,
            placed_areas: input.placed_areas,
            placements: input.placements,
            subgrid_report: input.subgrid_report,
            named_columns: input.named_columns,
            named_rows: input.named_rows,
            area_facts: input.area_facts,
            column_sizes: input.columns,
            row_sizes: input.rows,
            column_geometry: input.column_geometry,
            row_geometry: input.row_geometry,
            intrinsic_min_track_facts: Some(&row_intrinsic_min_track_facts),
            direct_members: direct_members.rows,
        },
    )?;
    let row_group = rows.group;
    let column_group = columns.group;
    let placement_rows = row_group
        .has_any_target()
        .then(|| InheritedGridOwnerBaselineTargets {
            group: row_group.clone(),
            mapping: CheckedOwnerToCurrentPlacementMap::identity(
                input.node,
                GridAxisKind::Row,
                row_group.physical_axis(),
                input
                    .constants
                    .flow_axes
                    .physical_axis_progression(row_group.physical_axis()),
                row_group.track_count(),
            ),
        });
    let placement_columns =
        column_group
            .has_any_target()
            .then(|| InheritedGridOwnerBaselineTargets {
                group: column_group.clone(),
                mapping: CheckedOwnerToCurrentPlacementMap::identity(
                    input.node,
                    GridAxisKind::Column,
                    column_group.physical_axis(),
                    input
                        .constants
                        .flow_axes
                        .physical_axis_progression(column_group.physical_axis()),
                    column_group.track_count(),
                ),
            });
    Ok(FinalAncestorBaselineGroups {
        rows: row_group,
        columns: column_group,
        placement_rows,
        placement_columns,
        row_child_envelope: None,
        column_child_envelope: None,
        row_downward_major_translation: rows.downward_major_translation,
        row_downward_minor_translation: rows.downward_minor_translation,
        column_downward_major_translation: columns.downward_major_translation,
        column_downward_minor_translation: columns.downward_minor_translation,
    })
}

pub(in crate::grid) fn baseline_groups<Node, S: LayoutScalar>(
    items: &[PendingGridItem<Node, S>],
    row_count: usize,
    column_count: usize,
    container_flow_axes: FlowAxes,
) -> GridBaselineGroups<S> {
    let expected_axis = container_flow_axes.block_axis();
    let mut groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); row_count],
        columns: vec![TrackBaselineGroup::default(); column_count],
    };
    for item in items {
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
                merge_expected_baseline(
                    group,
                    item.logical_baseline_geometry_for_span(
                        item.area.size.block,
                        container_flow_axes,
                    )
                    .major_baseline,
                    expected_axis,
                );
            }
            Some(BaselineGroupKind::Minor) => {
                let Some(row) = item.area.row_end.checked_sub(1) else {
                    continue;
                };
                let Some(group) = groups.rows.get_mut(row).map(|group| &mut group.last) else {
                    continue;
                };
                merge_expected_baseline(
                    group,
                    item.logical_baseline_geometry_for_span(
                        item.area.size.block,
                        container_flow_axes,
                    )
                    .minor_baseline,
                    expected_axis,
                );
            }
            None => {}
        }
    }
    groups
}

pub(in crate::grid) fn merge_expected_baseline<S: LayoutScalar>(
    target: &mut Option<PhysicalBaseline<S>>,
    candidate: PhysicalBaseline<S>,
    expected_axis: PhysicalAxis,
) -> bool {
    if candidate.axis() == expected_axis {
        merge_baseline(target, candidate);
        true
    } else {
        false
    }
}

fn merge_baseline<S: LayoutScalar>(
    target: &mut Option<PhysicalBaseline<S>>,
    candidate: PhysicalBaseline<S>,
) {
    match target {
        Some(current) if current.axis() == candidate.axis() => {
            if candidate.coordinate() > current.coordinate() {
                *current = candidate;
            }
        }
        Some(_) => {}
        None => *target = Some(candidate),
    }
}

pub(in crate::grid) fn grid_container_baselines<Node, S: LayoutScalar>(
    items: &[PendingGridItem<Node, S>],
    groups: &GridBaselineGroups<S>,
    row_offsets: &[S],
    rows: &[S],
    flow_axes: FlowAxes,
) -> GridContainerBaselines<S> {
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

    let mut baselines = BaselinesOf::NONE;
    if let Some(point) = first_occupied_row.and_then(|row| {
        items
            .iter()
            .filter(|item| item.area.row == row)
            .min_by_key(|item| grid_area_start_key(item.area))
            .map(|item| item.first_baseline.translated(item.location))
    }) {
        baselines.record_first(point);
    }
    if let Some(point) = last_occupied_row.and_then(|row| {
        items
            .iter()
            .filter(|item| item.area.row_end.checked_sub(1) == Some(row))
            .max_by_key(|item| grid_area_end_key(item.area))
            .map(|item| {
                item.output
                    .baselines()
                    .last_or_synthesize_block_baseline(item.child_flow_axes, item.output.size)
                    .translated(item.location)
            })
    }) {
        baselines.record_last(point);
    }

    if let Some(first) = first_occupied_row.and_then(|row| {
        groups.rows.get(row).and_then(|group| {
            group
                .first
                .and_then(|baseline| baseline.coordinate_on(flow_axes.block_axis()))
                .map(|baseline| row_offsets[row] + baseline)
        })
    }) {
        baselines.replace_first_axis(
            BaselinesOf::from_block_coordinates(flow_axes, Some(first), None).first,
        );
    }
    if let Some(last) = last_occupied_row.and_then(|row| {
        groups.rows.get(row).and_then(|group| {
            group
                .last
                .and_then(|baseline| baseline.coordinate_on(flow_axes.block_axis()))
                .map(|baseline| row_offsets[row] + rows[row] - baseline)
        })
    }) {
        baselines.replace_last_axis(
            BaselinesOf::from_block_coordinates(flow_axes, None, Some(last)).last,
        );
    }

    GridContainerBaselines { baselines }
}

pub(in crate::grid) fn logical_grid_container_baselines<Node, S: LayoutScalar>(
    items: &[PendingGridItem<Node, S>],
    groups: &GridBaselineGroups<S>,
    row_offsets: &[S],
    rows: &[S],
    flow_axes: FlowAxes,
    containing_size: Size<S>,
) -> GridContainerBaselines<S> {
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

    let mut baselines = BaselinesOf::NONE;
    if let Some(point) = first_occupied_row.and_then(|row| {
        items
            .iter()
            .filter(|item| item.area.row == row)
            .min_by_key(|item| grid_area_start_key(item.area))
            .map(|item| item.first_baseline.translated(item.location))
    }) {
        baselines.record_first(point);
    }
    if let Some(point) = last_occupied_row.and_then(|row| {
        items
            .iter()
            .filter(|item| item.area.row_end.checked_sub(1) == Some(row))
            .max_by_key(|item| grid_area_end_key(item.area))
            .map(|item| {
                item.output
                    .baselines()
                    .last_or_synthesize_block_baseline(item.child_flow_axes, item.output.size)
                    .translated(item.location)
            })
    }) {
        baselines.record_last(point);
    }

    if let Some(first) = first_occupied_row.and_then(|row| {
        groups.rows.get(row).and_then(|group| {
            group
                .first
                .and_then(|baseline| baseline.coordinate_on(flow_axes.block_axis()))
                .map(|baseline| {
                    flow_axes.block_axis_coordinate(flow_axes.physical_point(
                        LogicalPointOf::new(S::ZERO, row_offsets[row] + baseline),
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                        containing_size,
                    ))
                })
        })
    }) {
        baselines.replace_first_axis(
            BaselinesOf::from_block_coordinates(flow_axes, Some(first), None).first,
        );
    }
    if let Some(last) = last_occupied_row.and_then(|row| {
        groups.rows.get(row).and_then(|group| {
            group
                .last
                .and_then(|baseline| baseline.coordinate_on(flow_axes.block_axis()))
                .map(|baseline| {
                    flow_axes.block_axis_coordinate(flow_axes.physical_point(
                        LogicalPointOf::new(S::ZERO, row_offsets[row] + rows[row] - baseline),
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                        containing_size,
                    ))
                })
        })
    }) {
        baselines.replace_last_axis(
            BaselinesOf::from_block_coordinates(flow_axes, None, Some(last)).last,
        );
    }

    GridContainerBaselines { baselines }
}

fn include_occupied_row(first: &mut Option<usize>, last: &mut Option<usize>, row: usize) {
    *first = Some(first.map_or(row, |current| current.min(row)));
    *last = Some(last.map_or(row, |current| current.max(row)));
}

fn grid_area_start_key<S: LayoutScalar>(area: GridArea<S>) -> (usize, usize) {
    (area.row, area.column)
}

fn grid_area_end_key<S: LayoutScalar>(area: GridArea<S>) -> (usize, usize) {
    (
        area.row_end.saturating_sub(1),
        area.column_end.saturating_sub(1),
    )
}

pub(in crate::grid) fn baseline_participation<S: LayoutScalar>(
    align_self: AlignItems,
    block_auto_margins: bool,
    synthesized_baseline_would_cycle: bool,
    baselines: BaselinesOf<S>,
    flow_axes: FlowAxes,
) -> BaselineParticipation {
    let (mut group, synthesized, fallback_alignment) = match align_self {
        AlignItems::Baseline => (
            Some(BaselineGroupKind::Major),
            baselines.first_block(flow_axes).is_none(),
            Some(AlignItems::Start),
        ),
        AlignItems::LastBaseline => (
            Some(BaselineGroupKind::Minor),
            baselines.last_block(flow_axes).is_none(),
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

pub(in crate::grid) fn baseline_participation_for_container<S: LayoutScalar>(
    align_self: AlignItems,
    block_auto_margins: bool,
    synthesized_baseline_would_cycle: bool,
    baselines: BaselinesOf<S>,
    child_flow_axes: FlowAxes,
    container_flow_axes: FlowAxes,
) -> BaselineParticipation {
    let mut participation = baseline_participation(
        align_self,
        block_auto_margins,
        synthesized_baseline_would_cycle,
        baselines,
        child_flow_axes,
    );
    if child_flow_axes.block_axis() != container_flow_axes.block_axis() {
        participation.participates = false;
        participation.group = None;
    }
    participation
}

pub(in crate::grid) fn synthesized_baseline_would_cycle<S: LayoutScalar>(
    align_self: AlignItems,
    baselines: BaselinesOf<S>,
    flow_axes: FlowAxes,
    row_span_tracks: &[TrackSizingOf<S>],
) -> bool {
    let synthesizes = match align_self {
        AlignItems::Baseline => baselines.first_block(flow_axes).is_none(),
        AlignItems::LastBaseline => baselines.last_block(flow_axes).is_none(),
        _ => false,
    };
    synthesizes
        && row_span_tracks.len() > 1
        && row_span_tracks
            .iter()
            .any(track_accepts_intrinsic_contribution)
}
