use super::*;

#[derive(Clone, Copy)]
pub(in crate::grid) struct SubgridChildParentContextInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(in crate::grid) item: SubgridItemReport<Node>,
    pub(in crate::grid) child_style: &'a NodeInputOf<S>,
    pub(in crate::grid) area: GridArea<S>,
    pub(in crate::grid) content_box_size: Size<S>,
    pub(in crate::grid) columns: &'a [S],
    pub(in crate::grid) rows: &'a [S],
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) parent_named_columns: &'a NamedGridLines,
    pub(in crate::grid) parent_named_rows: &'a NamedGridLines,
    pub(in crate::grid) parent_area_facts: Option<&'a GridAreaNameFacts>,
    pub(in crate::grid) parent_baseline_groups: &'a GridBaselineGroups<S>,
    pub(in crate::grid) margin: Edges<Option<S>>,
    pub(in crate::grid) border: Edges<S>,
    pub(in crate::grid) padding: Edges<S>,
}

pub(in crate::grid) fn subgrid_child_parent_context<Node, S: LayoutScalar>(
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

pub(in crate::grid) fn subgrid_child_parent_context_with_geometry<Node, S: LayoutScalar>(
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
pub(in crate::grid) fn subgrid_child_parent_context_from_ancestor_groups<Node, S: LayoutScalar>(
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

pub(in crate::grid) fn subgrid_child_parent_context_from_ancestor_groups_with_geometry<
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
pub(in crate::grid) enum SubgridChildContextError<S: LayoutScalar> {
    ValueResolution(LengthResolutionStatus<S>),
    TrackInheritance(SubgridTrackInheritanceError),
    BaselineInheritance(SubgridBaselineInheritanceError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum SubgridBaselineInheritanceError {
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

pub(in crate::grid) fn subgrid_child_context_error<Node, S, M>(
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

pub(in crate::grid) fn subgrid_child_context_container_error<Node, S, M>(
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

pub(in crate::grid) fn inherited_subgrid_layout_tracks<S: LayoutScalar>(
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

pub(in crate::grid) fn child_subgrid_gap<S: LayoutScalar>(
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

pub(super) struct SubgridBaselineRefreshInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) container_style: &'a NodeInputOf<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) named_columns: NamedGridLines,
    pub(super) named_rows: NamedGridLines,
    pub(super) area_facts: Option<GridAreaNameFacts>,
    pub(super) template_area_expanded_axes: TemplateAreaExpandedAxes,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) ancestor_baseline_groups: &'a FinalAncestorBaselineGroups<Node, S>,
    pub(super) containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
}

pub(super) fn refresh_subgrid_items_with_baselines<Tree, M>(
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
