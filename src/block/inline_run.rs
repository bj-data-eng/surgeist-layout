use super::in_flow::{content_size_contribution, relative_inset_offset};
use super::*;

pub(super) fn inline_run_end<Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<<Tree as Traverse>::Scalar>,
    mut index: usize,
) -> usize
where
    Tree: Compute<M>,
{
    while index < children.len() {
        match tree.layout_input(children[index]) {
            LayoutInputOf::Box(style) => {
                if style.display == crate::Display::None || style.position == Position::Absolute {
                    index += 1;
                    continue;
                }
                if style.float != Float::None || !style.display.is_inline_level() {
                    break;
                }
            }
            LayoutInputOf::LineBreak(input) => {
                if input.display().is_none() {
                    index += 1;
                    continue;
                }
                visible_line_break_in_flow(
                    tree,
                    children[index],
                    constants.writing_mode,
                    constants.direction,
                );
            }
            LayoutInputOf::InlineText(_) => {}
            LayoutInputOf::InlineBoundary(_) => {
                visible_inline_boundary_in_flow(
                    tree,
                    children[index],
                    constants.writing_mode,
                    constants.direction,
                );
            }
        }
        index += 1;
    }
    index
}

pub(super) fn visible_line_break_in_flow<Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<LineBreakInputOf<<Tree as Traverse>::Scalar>>
where
    Tree: Compute<M>,
{
    let LayoutInputOf::LineBreak(line_break) = tree.layout_input(child) else {
        return None;
    };
    if line_break.display().is_none() {
        return None;
    }
    if line_break.writing_mode() != flow_writing_mode || line_break.direction() != flow_direction {
        panic!("line-break flow must match containing inline flow");
    }
    Some(line_break)
}

pub(super) fn visible_inline_boundary_in_flow<Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<InlineBoundaryInputOf<<Tree as Traverse>::Scalar>>
where
    Tree: Compute<M>,
{
    let LayoutInputOf::InlineBoundary(boundary) = tree.layout_input(child) else {
        return None;
    };
    if boundary.writing_mode() != flow_writing_mode || boundary.direction() != flow_direction {
        panic!("inline boundary flow must match containing inline flow");
    }
    Some(boundary)
}

#[derive(Clone, Copy)]
pub(super) struct VisibleInlineRunTransition {
    pub(super) start: usize,
    pub(super) end: usize,
}

impl VisibleInlineRunTransition {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(super) fn apply<Tree, S, M>(
        self,
        tree: &mut Tree,
        node: <Tree as Traverse>::Node,
        children: &[<Tree as Traverse>::Node],
        mut context: InlineRunContext<'_, S>,
        state: InlineRunTransitionState<'_, <Tree as Traverse>::Node, S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, (), S, M>
    where
        Tree: Compute<M, Scalar = S>,
        S: LayoutScalar,
    {
        let InlineRunTransitionState {
            cursor_block,
            content_size,
            scroll_content_size,
            baselines,
            static_positions,
            resolved_terminal_float_block_end,
            active_margin,
            is_collapsing_first_margin,
            active_margin_can_collapse_with_parent,
            all_in_flow_children_can_collapse_through,
            float_exclusions,
            contributions,
        } = state;
        *cursor_block = *cursor_block + active_margin.resolve();
        *is_collapsing_first_margin = false;
        context.cursor_block = *cursor_block;
        let constants = context.constants;
        let placement = layout_inline_run_children(
            tree,
            node,
            &children[self.start..self.end],
            context,
            float_exclusions,
            contributions,
        )?;

        let placement_content_size = constants.flow_axes.logical_size(placement.content_size);
        content_size.inline = content_size.inline.max(placement_content_size.inline);
        content_size.block = content_size.block.max(placement_content_size.block);
        let placement_scroll_content_size = constants
            .flow_axes
            .logical_size(placement.scroll_content_size);
        scroll_content_size.inline = scroll_content_size
            .inline
            .max(placement_scroll_content_size.inline);
        scroll_content_size.block = scroll_content_size
            .block
            .max(placement_scroll_content_size.block);
        record_inline_run_baselines(baselines, &placement, *cursor_block, constants);
        *cursor_block = *cursor_block + placement.logical_block_extent(constants.flow_axes);
        static_positions.extend(placement.static_positions);
        *resolved_terminal_float_block_end = placement.resolved_float_terminal_block_end;
        *active_margin = CollapsibleMarginOf::<S>::ZERO;
        *active_margin_can_collapse_with_parent = false;
        *all_in_flow_children_can_collapse_through = false;
        Ok(())
    }
}

pub(super) struct InlineRunTransitionState<'a, Node, S: LayoutScalar> {
    pub(super) cursor_block: &'a mut S,
    pub(super) content_size: &'a mut LogicalSizeOf<S>,
    pub(super) scroll_content_size: &'a mut LogicalSizeOf<S>,
    pub(super) baselines: &'a mut BaselinesOf<S>,
    pub(super) static_positions: &'a mut Vec<(Node, Point<S>)>,
    pub(super) resolved_terminal_float_block_end: &'a mut Option<S>,
    pub(super) active_margin: &'a mut CollapsibleMarginOf<S>,
    pub(super) is_collapsing_first_margin: &'a mut bool,
    pub(super) active_margin_can_collapse_with_parent: &'a mut bool,
    pub(super) all_in_flow_children_can_collapse_through: &'a mut bool,
    pub(super) float_exclusions: &'a FloatExclusions<S, Node>,
    pub(super) contributions: &'a mut ScrollContributionAccumulatorOf<S>,
}

struct InlineRunPlacement<Node, S: LayoutScalar> {
    size: Size<S>,
    content_size: Size<S>,
    scroll_content_size: Size<S>,
    static_positions: Vec<(Node, Point<S>)>,
    baselines: BaselinesOf<S>,
    first_baseline: Option<S>,
    last_baseline: Option<S>,
    resolved_float_terminal_block_end: Option<S>,
}

impl<Node, S: LayoutScalar> InlineRunPlacement<Node, S> {
    fn logical_block_extent(&self, flow_axes: crate::geometry::FlowAxes) -> S {
        flow_axes.logical_size(self.size).block
    }
}

pub(super) struct InlineRunContext<'a, S: LayoutScalar> {
    pub(super) source_index_start: usize,
    pub(super) cursor_block: S,
    pub(super) owned_float_block_end: S,
    pub(super) constants: &'a Constants<S>,
    pub(super) input: ComputeInputOf<S>,
    pub(super) node_inner_size: Size<Option<S>>,
    pub(super) set_layout: bool,
}

fn resolved_inline_float_terminal_block_end<S: LayoutScalar>(
    report: &crate::inline::MixedInlineRunReportOf<S>,
    cursor_block: S,
    owned_float_block_end: S,
) -> Option<S> {
    let terminal_line_block_end = cursor_block + report.block_extent;
    report.float_edge_phase.map(|phase| {
        if terminal_line_block_end > owned_float_block_end {
            terminal_line_block_end
        } else {
            (terminal_line_block_end + phase).max(owned_float_block_end)
        }
    })
}

fn forced_line_break_control<S: LayoutScalar>(
    source_index: usize,
    input: LineBreakInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> ForcedLineBreakControlOf<S> {
    ForcedLineBreakControlOf::new(
        source_index,
        InlineFlowOf::new(
            input.writing_mode(),
            input.direction(),
            available_inline_extent,
        ),
        input.metrics(),
        InlineControlAlignment::from(input.vertical_align()),
        input.clear(),
    )
}

fn inline_boundary_control<S: LayoutScalar>(
    source_index: usize,
    input: InlineBoundaryInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> InlineBoundaryControlOf<S> {
    InlineBoundaryControlOf::new(
        source_index,
        input.kind(),
        InlineFlowOf::new(
            input.writing_mode(),
            input.direction(),
            available_inline_extent,
        ),
        input.metrics(),
        InlineControlAlignment::from(input.vertical_align()),
    )
}

fn layout_inline_run_children<Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    run: &[<Tree as Traverse>::Node],
    context: InlineRunContext<'_, S>,
    float_exclusions: &FloatExclusions<S, <Tree as Traverse>::Node>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InlineRunPlacement<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let InlineRunContext {
        source_index_start,
        cursor_block,
        owned_float_block_end,
        constants,
        input,
        node_inner_size,
        set_layout,
    } = context;
    let logical_node_inner_size = constants.flow_axes.logical_size(node_inner_size);
    let available_inline_extent = logical_node_inner_size
        .inline
        .map(AvailableOf::<S>::definite)
        .unwrap_or(
            constants
                .flow_axes
                .logical_size(constants.available_content)
                .inline,
        );
    let containing_size = constants.containing_size(logical_node_inner_size);
    let mut participants = Vec::new();
    let mut atomic_children = Vec::new();
    let mut control_children = Vec::new();
    let mut published_text = Vec::new();
    let mut static_positions = Vec::new();
    for (offset, child) in run.iter().copied().enumerate() {
        let source_index = source_index_start + offset;
        let child_style = match tree.layout_input(child) {
            LayoutInputOf::InlineText(text) => {
                published_text.push((child, source_index, Vec::new(), None, None));
                participants.extend(text.segments().iter().copied().map(|segment| {
                    MixedInlineParticipantOf::ShapedText(ShapedTextParticipantOf {
                        source_index,
                        segment,
                    })
                }));
                continue;
            }
            LayoutInputOf::Box(style) => *style,
            LayoutInputOf::LineBreak(line_break) => {
                if line_break.display().is_none() {
                    if set_layout {
                        tree.set_unrounded(
                            child,
                            NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                source_index,
                            )),
                        );
                    }
                    continue;
                }
                let line_break = visible_line_break_in_flow(
                    tree,
                    child,
                    constants.writing_mode,
                    constants.direction,
                )
                .expect("visible line-break input remains visible after validation");
                participants.push(MixedInlineParticipantOf::ForcedLineBreak(
                    forced_line_break_control(source_index, line_break, available_inline_extent),
                ));
                control_children.push((child, source_index));
                continue;
            }
            LayoutInputOf::InlineBoundary(_) => {
                let boundary = visible_inline_boundary_in_flow(
                    tree,
                    child,
                    constants.writing_mode,
                    constants.direction,
                )
                .expect("inline-boundary input remains present after validation");
                participants.push(MixedInlineParticipantOf::Boundary(inline_boundary_control(
                    source_index,
                    boundary,
                    available_inline_extent,
                )));
                control_children.push((child, source_index));
                continue;
            }
        };
        if child_style.display == crate::Display::None {
            if set_layout {
                tree.set_unrounded(
                    child,
                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(source_index)),
                );
                tree.compute_child(
                    child,
                    ComputeInputOf::<S>::hidden_in_containing_pass(
                        ContainingLayoutContext::new(
                            constants.flow_axes,
                            ParentFormattingContext::BlockFlow,
                        ),
                        input.settled_auto_scrollbars(),
                    ),
                )?;
            }
            continue;
        }
        if child_style.position == Position::Absolute {
            static_positions.push((
                child,
                absolute_static_position(cursor_block, constants, containing_size),
            ));
            continue;
        }

        let participation = child_style.atomic_inline_participation.ok_or_else(|| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(child),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::AtomicInlineParticipation {
                    reason: AtomicInlineParticipationRoleError::MissingForAtomicInline,
                }),
            )
        })?;
        let child_padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.padding,
                node_inner_size,
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let child_border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.border,
                node_inner_size,
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let output = tree.compute_child(
            child,
            ComputeInputOf::<S>::for_child(
                input.run_mode().for_child(),
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                constants.definite_child_containing_block_size(),
                ContainingLayoutContext::new(
                    constants.flow_axes,
                    ParentFormattingContext::BlockFlow,
                ),
                constants.flow_axes.physical_size(LogicalSizeOf::new(
                    available_inline_extent,
                    AvailableOf::<S>::MAX_CONTENT,
                )),
            )
            .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars()),
        )?;
        let unresolved_margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.margin,
                node_inner_size,
                |length, basis| resolve_auto_optional(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let child_margin = resolve_atomic_inline_margin(unresolved_margin);
        let item = atomic_inline_box_participant(
            source_index,
            child_style.clone(),
            output,
            child_margin,
            child_padding,
            child_border,
            constants.flow_axes,
        );
        participants.push(MixedInlineParticipantOf::Atomic {
            item,
            participation,
        });
        atomic_children.push((child, source_index, child_style, output));
    }
    let logical_content_box_inset = constants.logical_content_box_inset();
    let mut provider_error = None;
    let report = layout_mixed_inline_run_with_band_source(
        MixedInlineRunInputOf {
            available_inline_extent,
            flow_axes: constants.flow_axes,
            text_align: constants.text_align,
            participants,
        },
        |block_start, block_end| {
            let query_block_start = cursor_block + block_start;
            let query_block_end = cursor_block + block_end;
            let band = if !set_layout || provider_error.is_some() {
                float_exclusions.query_rectangular_line_band(query_block_start, query_block_end)
            } else {
                match float_exclusions.query_provider_band(
                    tree,
                    container,
                    query_block_start,
                    query_block_end,
                ) {
                    Ok(band) => band,
                    Err(error) => {
                        provider_error = Some(error);
                        FloatBand {
                            inline_start: float_exclusions.containing_inline_start,
                            inline_end: float_exclusions.containing_inline_end,
                            next_transition: None,
                            #[cfg(test)]
                            evaluated: 0,
                        }
                    }
                }
            };
            LogicalLineBandQueryResultOf {
                inline_start: band.inline_start - logical_content_box_inset.inline_start,
                inline_end: band.inline_end - logical_content_box_inset.inline_start,
                next_transition: band
                    .next_transition
                    .map(|transition| transition - cursor_block),
            }
        },
        |block, clear| {
            float_exclusions.clearance_for_line_intent(cursor_block + block, clear) - cursor_block
        },
    );
    if let Some(error) = provider_error {
        return Err(error);
    }
    let resolved_float_terminal_block_end =
        resolved_inline_float_terminal_block_end(&report, cursor_block, owned_float_block_end);
    let report_logical_size = LogicalSizeOf::new(report.inline_extent, report.block_extent);
    let report_size = constants.flow_axes.physical_size(report_logical_size);
    let project_point = |inline: S, block: S, size: LogicalSizeOf<S>| {
        constants.flow_axes.physical_point(
            LogicalPointOf::new(
                logical_content_box_inset.inline_start + inline,
                cursor_block + block,
            ),
            size,
            containing_size,
        )
    };

    let mut text_content_size = Size::ZERO;
    for source in &report.fragments {
        let logical_size = LogicalSizeOf::new(source.inline_extent, source.block_extent);
        let size = constants.flow_axes.physical_size(logical_size);
        let location = project_point(source.inline_start, source.block_start, logical_size);
        let rect = crate::ScrollRectOf::try_new(location, size).map_err(|error| {
            block_inline_geometry_error(
                container,
                run.get(source.source_index - source_index_start).copied(),
                input.run_mode(),
                error,
            )
        })?;
        let baseline = project_point(
            source.inline_start,
            source.baseline,
            LogicalSizeOf::new(S::ZERO, S::ZERO),
        );
        let (_, _, fragments, union_min, union_max) = published_text
            .iter_mut()
            .find(|(_, source_index, _, _, _)| *source_index == source.source_index)
            .expect("every shaped source retains its text publication group");
        *union_min = Some(union_min.map_or(location, |current: Point<S>| {
            Point::new(current.x.min(location.x), current.y.min(location.y))
        }));
        let maximum = Point::new(location.x + size.width, location.y + size.height);
        *union_max = Some(union_max.map_or(maximum, |current: Point<S>| {
            Point::new(current.x.max(maximum.x), current.y.max(maximum.y))
        }));
        fragments.push(InlineFragmentOutputOf::new(
            source.segment_id,
            rect,
            baseline,
            source.line_index,
            source.visual_index,
            source.replacement_inline_extent,
        ));
        text_content_size = max_content_size(
            text_content_size,
            content_size_contribution(
                Point::new(
                    location.x - constants.content_box_inset.left,
                    location.y - constants.content_box_inset.top,
                ),
                size,
                size,
                ComputedOverflow::VISIBLE,
                false,
            ),
        );
        if set_layout {
            contributions.include_direct_line(rect);
        }
    }

    if set_layout {
        for (child, source_index, fragments, union_min, union_max) in published_text {
            let anchor = report
                .anchors
                .iter()
                .find(|anchor| anchor.source_index == source_index)
                .map_or(Point::ZERO, |anchor| {
                    project_point(
                        anchor.inline_start,
                        anchor.block_start,
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                    )
                });
            let (text_location, text_size) = match (union_min, union_max) {
                (Some(minimum), Some(maximum)) => (
                    minimum,
                    Size::new(maximum.x - minimum.x, maximum.y - minimum.y),
                ),
                _ => (anchor, Size::ZERO),
            };
            tree.compute_child(
                child,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::ContentSize,
                    RequestedAxis::Both,
                    text_size.map(Some),
                    node_inner_size,
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::BlockFlow,
                    ),
                    constants.available_content,
                )
                .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
            )?;
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location: text_location,
                    size: text_size,
                    content_size: text_size,
                    ..NodeOutputOf::new()
                },
            );
            tree.set_unrounded_inline_fragment_state(child, Some(fragments));
        }
    }

    let atomic_sources = report
        .atomics
        .iter()
        .map(|source| (source.item.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    let mut content_size = report_size;
    let mut scroll_content_size = text_content_size;
    for (child, source_index, child_style, output) in atomic_children {
        let source = atomic_sources[&source_index];
        let logical_size = constants.flow_axes.logical_size(source.item.size);
        let projected_location =
            project_point(source.inline_start, source.block_start, logical_size);
        let inset_offset = relative_inset_offset(
            constants
                .flow_axes
                .zip_physical_edges_with_inline_extent(
                    child_style.inset,
                    node_inner_size,
                    |length, basis| resolve_auto_optional(length, basis),
                )
                .transpose_with_node(tree, child)?,
            constants.flow_axes,
        );
        let location = Point::new(
            projected_location.x + inset_offset.x,
            projected_location.y + inset_offset.y,
        );
        let contribution = content_size_contribution(
            Point::new(
                location.x - constants.content_box_inset.left,
                location.y - constants.content_box_inset.top,
            ),
            source.item.size,
            output.content_size,
            child_style.overflow,
            child_style.item_is_replaced,
        );
        content_size = max_content_size(content_size, contribution);
        scroll_content_size = max_content_size(scroll_content_size, contribution);

        if set_layout {
            let scroll_geometry = retained_child_scroll_geometry(
                &child_style,
                source.item.size,
                source.item.content_size,
                source.item.padding,
                source.item.border,
                output.scroll_geometry,
            )
            .map_err(|error| layout_child_geometry_error(container, child, error))?;
            contributions
                .include_in_flow_geometry(location, source.item.margin, scroll_geometry)
                .map_err(|error| layout_child_geometry_error(container, child, error))?;
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size: source.item.size,
                    content_size: source.item.content_size,
                    border: source.item.border,
                    padding: source.item.padding,
                    margin: source.item.margin,
                    ..NodeOutputOf::new()
                }
                .with_scroll_geometry(Some(scroll_geometry)),
            );
        }
    }

    let control_sources = report
        .controls
        .iter()
        .map(|source| (source.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    if set_layout {
        for (child, source_index) in control_children {
            let source = control_sources[&source_index];
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location: project_point(
                        source.inline_start,
                        source.block_start,
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                    ),
                    ..NodeOutputOf::new()
                },
            );
        }
    }

    let projected_baseline = |baseline| {
        constants.flow_axes.block_axis_coordinate(project_point(
            S::ZERO,
            baseline,
            LogicalSizeOf::new(S::ZERO, S::ZERO),
        ))
    };
    let baselines = BaselinesOf::from_block_coordinates(
        constants.flow_axes,
        report.first_baseline.map(projected_baseline),
        report.last_baseline.map(projected_baseline),
    );

    Ok(InlineRunPlacement {
        size: report_size,
        content_size,
        scroll_content_size,
        static_positions,
        baselines,
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
        resolved_float_terminal_block_end,
    })
}

fn record_inline_run_baselines<S: LayoutScalar>(
    baselines: &mut BaselinesOf<S>,
    placement: &InlineRunPlacement<impl Copy, S>,
    cursor_block: S,
    constants: &Constants<S>,
) {
    if constants.flow_axes.inline_axis() == PhysicalAxis::Vertical {
        baselines.record_first(placement.baselines.first);
        baselines.record_last(placement.baselines.last);
        return;
    }

    if let Some(baseline) = placement.first_baseline {
        baselines.record_first(
            BaselinesOf::from_block_coordinates(
                constants.flow_axes,
                Some(cursor_block + baseline),
                None,
            )
            .first,
        );
    }
    if let Some(baseline) = placement.last_baseline {
        baselines.record_last(
            BaselinesOf::from_block_coordinates(
                constants.flow_axes,
                None,
                Some(cursor_block + baseline),
            )
            .last,
        );
    }
}

fn atomic_inline_box_participant<S: LayoutScalar>(
    source_index: usize,
    child_style: NodeInputOf<S>,
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
    containing_flow_axes: crate::geometry::FlowAxes,
) -> AtomicInlineBoxParticipant<S> {
    let logical_size = containing_flow_axes.logical_size(output.size);
    let used_overflow =
        UsedOverflow::from_computed(child_style.overflow, child_style.item_is_replaced);
    let block_overflow = match containing_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => used_overflow.x(),
        PhysicalAxis::Vertical => used_overflow.y(),
    };
    let selected_inner_baseline = (child_style.vertical_align == VerticalAlign::Baseline
        && block_overflow.value() == Overflow::Visible)
        .then(|| {
            let baselines = output.baselines();
            containing_flow_axes
                .block_axis_coordinate(baselines.first)
                .or_else(|| containing_flow_axes.block_axis_coordinate(baselines.last))
                .map(|physical| {
                    if containing_flow_axes
                        .logical_axis_progression(crate::LogicalAxis::Block)
                        .is_decreasing()
                    {
                        logical_size.block - physical
                    } else {
                        physical
                    }
                })
        })
        .flatten();
    AtomicInlineBoxParticipant {
        source_index,
        size: output.size,
        content_size: output.content_size,
        margin,
        padding,
        border,
        scrollbar_size: child_scrollbar_size(&child_style),
        first_baseline: selected_inner_baseline,
        alignment: child_style.vertical_align.into(),
    }
}

fn resolve_atomic_inline_margin<S: LayoutScalar>(margin: Edges<Option<S>>) -> Edges<S> {
    margin.map(|value| value.unwrap_or(S::ZERO))
}
