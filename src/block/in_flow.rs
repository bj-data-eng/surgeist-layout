use super::absolute::absolute_static_position;
use super::inline_run::{
    InlineRunContext, InlineRunTransitionState, VisibleInlineRunTransition, inline_run_end,
    visible_inline_boundary_in_flow, visible_line_break_in_flow,
};
use super::scroll::retained_child_scroll_geometry;
use super::sizing::{
    maximum_size, minimum_size, preferred_size, resolve_auto_optional, resolve_length_or_zero,
};
use super::*;

pub(super) fn normal_flow_children_can_establish_baseline<Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
) -> bool
where
    Tree: Compute<M>,
{
    children.iter().copied().any(|child| {
        let style = match InlineParticipantProjection::lookup::<Tree, M>(tree, child).into_kind() {
            InlineParticipantKindOf::InlineText(_) => return true,
            InlineParticipantKindOf::Box(style) => style,
            InlineParticipantKindOf::LineBreak(input) => {
                return !input.display().is_none()
                    && input.metrics().line_extent() > Tree::Scalar::ZERO;
            }
            InlineParticipantKindOf::InlineBoundary(input) => {
                return input.metrics().line_extent() > Tree::Scalar::ZERO;
            }
        };
        if style.display == crate::Display::None
            || style.position == Position::Absolute
            || style.float != Float::None
        {
            return false;
        }

        style.display.is_inline_level()
            || style.display.inner_display() == crate::Display::Block
            || tree.child_count(child) > 0 && style.display.inner_display() == crate::Display::Flex
    })
}

pub(super) struct InFlowResult<Node, S: LayoutScalar> {
    pub(super) content_size: LogicalSizeOf<S>,
    pub(super) scroll_content_size: LogicalSizeOf<S>,
    pub(super) owned_float_block_end: S,
    pub(super) resolved_terminal_float_block_end: Option<S>,
    pub(super) contributions: ScrollContributionAccumulatorOf<S>,
    pub(super) baselines: BaselinesOf<S>,
    pub(super) static_positions: Vec<(Node, Point<S>)>,
    pub(super) pending_floats: Vec<PendingFloat<Node, S>>,
    pub(super) cursor_block: S,
    pub(super) top_margin: CollapsibleMarginOf<S>,
    pub(super) active_margin: CollapsibleMarginOf<S>,
    pub(super) active_margin_can_collapse_with_parent: bool,
    pub(super) all_in_flow_children_can_collapse_through: bool,
}

impl<Node, S: LayoutScalar> InFlowResult<Node, S> {
    pub(super) fn top_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if constants.collapse_top_margin {
            self.top_margin
        } else {
            constants.own_top_margin
        }
    }

    pub(super) fn bottom_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
            self.active_margin
        } else {
            constants.own_bottom_margin
        }
    }

    pub(super) fn auto_block(&self, constants: &Constants<S>) -> S {
        let bottom_margin_offset =
            if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
                S::ZERO
            } else {
                self.active_margin.resolve()
            };
        let content_box_inset = constants.logical_content_box_inset();
        let in_flow_block_end = self.cursor_block + bottom_margin_offset;
        let float_block_end = self
            .resolved_terminal_float_block_end
            .unwrap_or(self.owned_float_block_end)
            .max(self.owned_float_block_end);
        (in_flow_block_end.max(float_block_end) + content_box_inset.block_end)
            .max(content_box_inset.block_sum())
    }
}

pub(super) struct InFlowPassContext<'a, S: LayoutScalar, Node> {
    pub(super) inner_inline: Option<S>,
    pub(super) set_layout: bool,
    pub(super) inherited: Option<&'a InheritedFloatExclusions<S, Node>>,
}

enum InFlowChildStart<S: LayoutScalar> {
    VisibleInlineRun(VisibleInlineRunTransition),
    FlowBox(Box<BlockChildProjection<S>>),
}

pub(super) fn layout_in_flow_children<Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<S>,
    input: ComputeInputOf<S>,
    pass: InFlowPassContext<'_, S, <Tree as Traverse>::Node>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InFlowResult<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowPassContext {
        inner_inline,
        set_layout,
        inherited,
    } = pass;
    let logical_node_inner_size =
        LogicalSizeOf::new(inner_inline, constants.logical_node_inner_size().block);
    let node_inner_size = constants.flow_axes.physical_size(logical_node_inner_size);
    let mut cursor_block = constants.logical_content_box_inset().block_start;
    let mut content_size = LogicalSizeOf::new(S::ZERO, S::ZERO);
    let mut scroll_content_size = LogicalSizeOf::new(S::ZERO, S::ZERO);
    let mut owned_float_block_end = constants.logical_content_box_inset().block_start;
    let mut resolved_terminal_float_block_end = None;
    let mut baselines = BaselinesOf::NONE;
    let mut static_positions = Vec::new();
    let mut active_margin = CollapsibleMarginOf::<S>::ZERO;
    let mut top_margin = CollapsibleMarginOf::<S>::ZERO;
    let mut is_collapsing_first_margin = constants.collapse_top_margin;
    let mut all_in_flow_children_can_collapse_through = true;
    let mut active_margin_can_collapse_with_parent = constants.collapse_top_margin;
    let mut pending_floats = Vec::new();
    let mut float_intrinsics = FloatIntrinsics::new(
        inner_inline
            .map(AvailableOf::<S>::definite)
            .unwrap_or(constants.available_content.width),
    );
    let content_inline_size = inner_inline
        .or(constants
            .flow_axes
            .logical_size(constants.available_content)
            .inline
            .into_option())
        .unwrap_or(S::ZERO);
    let containing_size = constants.containing_size(logical_node_inner_size);
    let mut float_exclusions = FloatExclusions::new(
        constants.flow_axes,
        containing_size,
        content_inline_size,
        constants.logical_content_box_inset(),
    );
    if set_layout && let Some(inherited) = inherited {
        float_exclusions.inherit_into_child(inherited);
    }
    let content_box_size = Size::new(
        inner_inline
            .or(constants.node_inner_size.width)
            .or(constants.available_content.width.into_option())
            .unwrap_or(S::ZERO),
        constants.node_inner_size.height.unwrap_or(S::ZERO),
    );
    let content_box_origin = Point::new(
        constants.content_box_inset.left,
        constants.content_box_inset.top,
    );
    let contribution_seed = crate::ScrollRectOf::try_new(content_box_origin, content_box_size)
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
    let mut contributions = ScrollContributionAccumulatorOf::new(contribution_seed);

    let mut index = 0;
    while index < children.len() {
        let source_index = index;
        let child = children[index];
        let child_start =
            match InlineParticipantProjection::lookup::<Tree, M>(tree, child).into_kind() {
                InlineParticipantKindOf::Box(style) => {
                    if style.display == crate::Display::None {
                        if set_layout {
                            tree.set_unrounded(
                                child,
                                NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                    source_index,
                                )),
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
                        index += 1;
                        continue;
                    }
                    if style.position == Position::Absolute {
                        static_positions.push((
                            child,
                            absolute_static_position(
                                cursor_block + active_margin.resolve(),
                                constants,
                                constants.containing_size(logical_node_inner_size),
                            ),
                        ));
                        index += 1;
                        continue;
                    }
                    if style.display.is_inline_level() && style.float.is_none() {
                        let run_start = index;
                        index = inline_run_end(tree, children, constants, index + 1);
                        InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                            run_start, index,
                        ))
                    } else {
                        InFlowChildStart::FlowBox(style)
                    }
                }
                InlineParticipantKindOf::InlineText(_) => {
                    let run_start = index;
                    index = inline_run_end(tree, children, constants, index + 1);
                    InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                        run_start, index,
                    ))
                }
                InlineParticipantKindOf::LineBreak(line_break) => {
                    if line_break.display().is_none() {
                        if set_layout {
                            tree.set_unrounded(
                                child,
                                NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                    source_index,
                                )),
                            );
                        }
                        index += 1;
                        continue;
                    }
                    visible_line_break_in_flow(
                        tree,
                        child,
                        constants.writing_mode,
                        constants.direction,
                    );

                    let run_start = index;
                    index = inline_run_end(tree, children, constants, index + 1);
                    InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                        run_start, index,
                    ))
                }
                InlineParticipantKindOf::InlineBoundary(_) => {
                    visible_inline_boundary_in_flow(
                        tree,
                        child,
                        constants.writing_mode,
                        constants.direction,
                    );

                    let run_start = index;
                    index = inline_run_end(tree, children, constants, index + 1);
                    InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                        run_start, index,
                    ))
                }
            };
        let child_style = match child_start {
            InFlowChildStart::VisibleInlineRun(transition) => {
                transition.apply(
                    tree,
                    node,
                    children,
                    InlineRunContext {
                        source_index_start: transition.start,
                        cursor_block,
                        owned_float_block_end,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    InlineRunTransitionState {
                        cursor_block: &mut cursor_block,
                        content_size: &mut content_size,
                        scroll_content_size: &mut scroll_content_size,
                        baselines: &mut baselines,
                        static_positions: &mut static_positions,
                        resolved_terminal_float_block_end: &mut resolved_terminal_float_block_end,
                        active_margin: &mut active_margin,
                        is_collapsing_first_margin: &mut is_collapsing_first_margin,
                        active_margin_can_collapse_with_parent:
                            &mut active_margin_can_collapse_with_parent,
                        all_in_flow_children_can_collapse_through:
                            &mut all_in_flow_children_can_collapse_through,
                        float_exclusions: &float_exclusions,
                        contributions: &mut contributions,
                    },
                )?;
                continue;
            }
            InFlowChildStart::FlowBox(style) => *style,
        };

        let unresolved_margin = constants.flow_axes.zip_physical_edges_with_inline_extent(
            child_style.margin,
            node_inner_size,
            |length, basis| length.resolve_auto_with_status(basis),
        );
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
        let parent_logical_unresolved_margin = constants.flow_axes.logical_edges(unresolved_margin);
        let parent_logical_available = constants
            .flow_axes
            .logical_size(constants.available_content);
        let child_flow_axes = child_style.flow_axes;
        let child_parent_size = constants.child_containing_block_size(child_flow_axes);
        let child_logical_node_inner_size = child_flow_axes.logical_size(child_parent_size);
        let child_logical_available = child_flow_axes.logical_size(constants.available_content);
        let child_non_auto_margin = child_flow_axes
            .logical_edges(unresolved_margin)
            .map(resolved_length_auto_fallback_zero);
        let available_child_inline = child_logical_node_inner_size
            .inline
            .or(child_logical_available.inline.into_option())
            .map(|inline| (inline - child_non_auto_margin.inline_sum()).max(S::ZERO));
        let child_known = in_flow_child_known_size::<Tree, M>(
            tree,
            child,
            &child_style,
            child_padding + child_border,
            child_flow_axes,
            child_logical_node_inner_size,
            available_child_inline,
        )?;
        let mut child_input = ComputeInputOf::<S>::for_child(
            input.run_mode().for_child(),
            SizingMode::InherentSize,
            RequestedAxis::Both,
            child_known,
            child_parent_size,
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
            child_flow_axes.physical_size(LogicalSizeOf::new(
                in_flow_child_available_inline(
                    &child_style,
                    child_flow_axes,
                    available_child_inline,
                    child_logical_available.inline,
                ),
                AvailableOf::<S>::MAX_CONTENT,
            )),
        )
        .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars());
        let avoids_float_exclusions = block_child_avoids_float_exclusions(&child_style);
        let inherits_float_exclusions = set_layout
            && child_style.float.is_none()
            && child_style.display.inner_display() == crate::Display::Block
            && !avoids_float_exclusions
            && !float_exclusions.is_empty();
        let places_against_float_exclusions =
            set_layout && avoids_float_exclusions && !float_exclusions.is_empty();
        let mut output = tree.compute_child(
            child,
            if inherits_float_exclusions || places_against_float_exclusions {
                child_input.with_run_mode(RunMode::ComputeSize)
            } else {
                child_input
            },
        )?;

        let mut logical_child_size = constants.flow_axes.logical_size(output.size);
        let mut logical_child_margin = resolve_logical_in_flow_margin(
            parent_logical_unresolved_margin,
            logical_child_size,
            logical_node_inner_size
                .inline
                .or(parent_logical_available.inline.into_option()),
        );
        let mut child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        if inherits_float_exclusions {
            let preview_top_margin = output
                .block_margin_collapse
                .at(constants.flow_axes.block_start())
                .collapse_with_margin(
                    child_margin.at_physical_side(constants.flow_axes.block_start()),
                );
            let child_margin_can_collapse_with_parent =
                child_margin_can_collapse_with_parent(&child_style);
            let collapsed_margin = if is_collapsing_first_margin {
                if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                    active_margin.resolve()
                } else {
                    active_margin.collapse_with(preview_top_margin).resolve()
                }
            } else {
                active_margin.collapse_with(preview_top_margin).resolve()
            };
            let preview_cursor_block = cursor_block + collapsed_margin;
            let child_logical_location = LogicalPointOf::new(
                in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
                preview_cursor_block,
            );
            let child_logical_location = if child_style.clear == Clear::None {
                child_logical_location
            } else {
                LogicalPointOf::new(
                    child_logical_location.inline,
                    float_exclusions.clearance_block(preview_cursor_block, child_style.clear),
                )
            };
            let inherited = float_exclusions.for_ordinary_child(child_logical_location);
            debug_assert_eq!(child_style.display.inner_display(), crate::Display::Block);
            output = compute_block_with_inherited_float_exclusions(
                tree,
                child,
                child_input.with_settled_auto_scrollbars(
                    crate::scroll::SettledAutoScrollbarState::INITIAL,
                ),
                inherited,
            )?;
            logical_child_size = constants.flow_axes.logical_size(output.size);
            logical_child_margin = resolve_logical_in_flow_margin(
                parent_logical_unresolved_margin,
                logical_child_size,
                logical_node_inner_size
                    .inline
                    .or(parent_logical_available.inline.into_option()),
            );
            child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        } else if places_against_float_exclusions {
            let preview_top_margin = output
                .block_margin_collapse
                .at(constants.flow_axes.block_start())
                .collapse_with_margin(
                    child_margin.at_physical_side(constants.flow_axes.block_start()),
                );
            let child_margin_can_collapse_with_parent =
                child_margin_can_collapse_with_parent(&child_style);
            let collapsed_margin = if is_collapsing_first_margin {
                if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                    active_margin.resolve()
                } else {
                    active_margin.collapse_with(preview_top_margin).resolve()
                }
            } else {
                active_margin.collapse_with(preview_top_margin).resolve()
            };
            let preview_cursor_block = cursor_block + collapsed_margin;
            let containing_size = constants.containing_size(logical_node_inner_size);
            let preview_logical_location = LogicalPointOf::new(
                in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
                preview_cursor_block,
            );
            let preview_fallback = constants.flow_axes.physical_point(
                preview_logical_location,
                logical_child_size,
                containing_size,
            );
            let inline_size_is_auto =
                parent_inline_preferred_size_is_auto(&child_style, constants.flow_axes);
            let placement = float_exclusions.place_bfc_block(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: true,
                },
                BfcBandCandidate {
                    block_start: preview_cursor_block,
                    size: output.size,
                    margin: child_margin,
                    clear: child_style.clear,
                    fallback: preview_fallback,
                    inline_size_is_auto,
                },
            )?;

            if inline_size_is_auto {
                let parent_non_auto_margin =
                    parent_logical_unresolved_margin.map(resolved_length_auto_fallback_zero);
                let band_child_inline =
                    (placement.available_inline - parent_non_auto_margin.inline_sum()).max(S::ZERO);
                let band_available_child_inline =
                    if child_flow_axes.inline_axis() == constants.flow_axes.inline_axis() {
                        Some(band_child_inline)
                    } else {
                        available_child_inline
                    };
                let band_child_known = in_flow_child_known_size::<Tree, M>(
                    tree,
                    child,
                    &child_style,
                    child_padding + child_border,
                    child_flow_axes,
                    child_logical_node_inner_size,
                    band_available_child_inline,
                )?;
                let mut band_available = child_flow_axes.physical_size(LogicalSizeOf::new(
                    in_flow_child_available_inline(
                        &child_style,
                        child_flow_axes,
                        band_available_child_inline,
                        child_logical_available.inline,
                    ),
                    AvailableOf::<S>::MAX_CONTENT,
                ));
                set_parent_inline_available(
                    &mut band_available,
                    constants.flow_axes,
                    band_child_inline,
                );
                child_input = ComputeInputOf::<S>::for_child(
                    input.run_mode().for_child(),
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    band_child_known,
                    child_parent_size,
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::BlockFlow,
                    ),
                    band_available,
                )
                .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars());
            }

            output = tree.compute_child(child, child_input)?;
            logical_child_size = constants.flow_axes.logical_size(output.size);
            logical_child_margin = resolve_logical_in_flow_margin(
                parent_logical_unresolved_margin,
                logical_child_size,
                logical_node_inner_size
                    .inline
                    .or(parent_logical_available.inline.into_option()),
            );
            child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        }
        if !child_style.float.is_none() {
            let margin_box_inline = logical_child_size.inline + logical_child_margin.inline_sum();
            float_intrinsics.add(margin_box_inline, child_style.float, child_style.clear);
            content_size.inline = content_size.inline.max(float_intrinsics.result());
            if !input.run_mode().is_perform_layout() {
                index += 1;
                continue;
            }
            let pending_float = PendingFloat {
                node: child,
                source_index,
                side: child_style.float,
                clear: child_style.clear,
                block_start: cursor_block,
                size: output.size,
                content_size: output.content_size,
                border: child_border,
                padding: child_padding,
                margin: child_margin,
                float_exclusion: child_style.float_exclusion,
                child_compute_geometry: output.scroll_geometry,
            };
            let float_location = float_exclusions.place_float(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: set_layout,
                },
                &pending_float,
            )?;
            if set_layout {
                pending_floats.push(pending_float);
            }
            let containing_size = constants.containing_size(logical_node_inner_size);
            let logical_location =
                constants
                    .flow_axes
                    .logical_point(float_location, output.size, containing_size);
            let content_box_inset = constants.logical_content_box_inset();
            let float_inline_end = logical_location.inline
                + logical_child_size.inline
                + logical_child_margin.inline_end
                - content_box_inset.inline_start;
            let float_block_end =
                logical_location.block + logical_child_size.block + logical_child_margin.block_end;
            content_size.inline = content_size.inline.max(float_inline_end);
            content_size.block = content_size
                .block
                .max(float_block_end - content_box_inset.block_start);
            owned_float_block_end = owned_float_block_end.max(float_block_end);
            index += 1;
            continue;
        }
        resolved_terminal_float_block_end = None;
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
        let top_margin_set = output
            .block_margin_collapse
            .at(constants.flow_axes.block_start())
            .collapse_with_margin(child_margin.at_physical_side(constants.flow_axes.block_start()));
        let bottom_margin_set = output
            .block_margin_collapse
            .at(constants.flow_axes.block_end())
            .collapse_with_margin(child_margin.at_physical_side(constants.flow_axes.block_end()));
        let child_margin_can_collapse_with_parent =
            child_margin_can_collapse_with_parent(&child_style);
        let base_block = cursor_block;
        let collapsed_margin = if is_collapsing_first_margin {
            if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                top_margin = top_margin.collapse_with(top_margin_set);
            }
            is_collapsing_first_margin = false;
            if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                active_margin.resolve()
            } else {
                active_margin.collapse_with(top_margin_set).resolve()
            }
        } else {
            active_margin.collapse_with(top_margin_set).resolve()
        };
        cursor_block = cursor_block + collapsed_margin;
        let logical_location = LogicalPointOf::new(
            in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
            cursor_block,
        );
        let containing_size = constants.containing_size(logical_node_inner_size);
        let logical_fallback_location = constants.flow_axes.physical_point(
            logical_location,
            logical_child_size,
            containing_size,
        );
        let fallback_location = Point::new(
            logical_fallback_location.x + inset_offset.x,
            logical_fallback_location.y + inset_offset.y,
        );
        let location = if avoids_float_exclusions {
            let placement = float_exclusions.place_bfc_block(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: set_layout,
                },
                BfcBandCandidate {
                    block_start: cursor_block,
                    size: output.size,
                    margin: child_margin,
                    clear: child_style.clear,
                    fallback: Point::new(
                        fallback_location.x - inset_offset.x,
                        fallback_location.y - inset_offset.y,
                    ),
                    inline_size_is_auto: parent_inline_preferred_size_is_auto(
                        &child_style,
                        constants.flow_axes,
                    ),
                },
            )?;
            Point::new(
                placement.location.x + inset_offset.x,
                placement.location.y + inset_offset.y,
            )
        } else if child_style.clear != Clear::None {
            let cleared_logical_location = LogicalPointOf::new(
                logical_location.inline,
                float_exclusions.clearance_block(cursor_block, child_style.clear),
            );
            let cleared_location = constants.flow_axes.physical_point(
                cleared_logical_location,
                logical_child_size,
                containing_size,
            );
            Point::new(
                cleared_location.x + inset_offset.x,
                cleared_location.y + inset_offset.y,
            )
        } else {
            fallback_location
        };
        if set_layout {
            let scroll_geometry = with_block_scroll_projections::<Tree, M, _>(
                tree,
                child,
                |box_projection, target_projection| {
                    retained_child_scroll_geometry(
                        box_projection,
                        target_projection,
                        output.size,
                        output.content_size,
                        child_padding,
                        child_border,
                        output.scroll_geometry,
                    )
                },
            )
            .map_err(|error| layout_child_geometry_error(node, child, error))?;
            contributions
                .include_in_flow_geometry(location, child_margin, scroll_geometry)
                .map_err(|error| layout_child_geometry_error(node, child, error))?;
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size: output.size,
                    content_size: output.content_size,
                    border: child_border,
                    padding: child_padding,
                    margin: child_margin,
                    ..NodeOutputOf::new()
                }
                .with_scroll_geometry(Some(scroll_geometry)),
            );
        }

        let child_block_end = if avoids_float_exclusions || child_style.clear != Clear::None {
            constants
                .flow_axes
                .logical_point(location, output.size, containing_size)
                .block
                + logical_child_size.block
        } else {
            logical_location.block + logical_child_size.block
        };
        let contribution = content_size_contribution(
            Point::new(
                location.x - constants.content_box_inset.left,
                location.y - constants.content_box_inset.top,
            ),
            output.size,
            output.content_size,
            child_style.overflow,
            child_style.item_is_replaced,
        );
        let logical_contribution = constants.flow_axes.logical_size(contribution);
        let child_inline_content = (logical_child_margin.inline_sum() + logical_child_size.inline)
            .max(logical_contribution.inline + logical_child_margin.inline_end);
        let child_block_content = logical_contribution
            .block
            .max(child_block_end - constants.logical_content_box_inset().block_start);
        content_size.inline = content_size.inline.max(child_inline_content);
        content_size.block = content_size.block.max(child_block_content);
        scroll_content_size.inline = scroll_content_size.inline.max(child_inline_content);
        scroll_content_size.block = scroll_content_size.block.max(child_block_content);
        if let Some(baseline) = output.baselines().first_block_baseline(child_flow_axes) {
            baselines.record_first(baseline.translated(location));
        }
        if let Some(baseline) = output.baselines().last_block_baseline(child_flow_axes) {
            baselines.record_last(baseline.translated(location));
        }
        if output
            .block_margin_collapse
            .can_collapse_through(constants.flow_axes)
        {
            cursor_block = if child_style.clear == Clear::None {
                base_block + logical_child_size.block
            } else {
                child_block_end
            };
            active_margin = active_margin
                .collapse_with(top_margin_set)
                .collapse_with(bottom_margin_set);
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        } else {
            all_in_flow_children_can_collapse_through = false;
            cursor_block = child_block_end;
            active_margin = bottom_margin_set;
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        }
        index += 1;
    }

    Ok(InFlowResult {
        content_size,
        scroll_content_size,
        owned_float_block_end,
        resolved_terminal_float_block_end,
        contributions,
        baselines,
        static_positions,
        pending_floats,
        cursor_block,
        top_margin,
        active_margin,
        active_margin_can_collapse_with_parent,
        all_in_flow_children_can_collapse_through,
    })
}

fn child_margin_can_collapse_with_parent<S: LayoutScalar>(style: &BlockChildProjection<S>) -> bool {
    style.display == crate::Display::Block && style.position == Position::Relative
}

fn block_child_avoids_float_exclusions<S: LayoutScalar>(style: &BlockChildProjection<S>) -> bool {
    style.display != crate::Display::None
        && !style.display.is_inline_level()
        && style.position != Position::Absolute
        && style.float.is_none()
        && (matches!(
            style.display,
            crate::Display::Flex | crate::Display::Grid | crate::Display::GridLanes
        ) || (!style.item_is_replaced
            && style.overflow.establishes_independent_formatting_context()))
}

fn parent_inline_preferred_size_is_auto<S: LayoutScalar>(
    style: &BlockChildProjection<S>,
    parent_flow_axes: crate::geometry::FlowAxes,
) -> bool {
    match parent_flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => style.size.width.is_auto(),
        PhysicalAxis::Vertical => style.size.height.is_auto(),
    }
}

fn set_parent_inline_available<S: LayoutScalar>(
    available: &mut Size<AvailableOf<S>>,
    parent_flow_axes: crate::geometry::FlowAxes,
    value: S,
) {
    match parent_flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => available.width = AvailableOf::definite(value),
        PhysicalAxis::Vertical => available.height = AvailableOf::definite(value),
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the private child-size helper preserves the session's generic error envelope"
)]
fn in_flow_child_known_size<Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    style: &BlockChildProjection<Tree::Scalar>,
    padding_border: Edges<Tree::Scalar>,
    child_flow_axes: crate::geometry::FlowAxes,
    parent: LogicalSizeOf<Option<Tree::Scalar>>,
    available_inline: Option<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Option<Tree::Scalar>>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let parent = child_flow_axes.physical_size(parent);
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let min_size = minimum_size(&style.min_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let mut max_size = maximum_size(&style.max_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .add_optional(box_sizing_adjustment);
    let aspect_height_limit = style
        .aspect_ratio
        .zip(max_size.height)
        .and_then(|(ratio, height)| max_size.width.is_none().then_some(height * ratio.get()));
    if let Some(width) = aspect_height_limit {
        max_size.width = Some(width);
    }
    let known = preferred_size(&style.size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .clamp_max_before_min_optional(min_size, max_size);

    let mut known = child_flow_axes.logical_size(known);
    let min_size = child_flow_axes.logical_size(min_size);
    let max_size = child_flow_axes.logical_size(max_size);
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => style.size.width.clone(),
        crate::PhysicalAxis::Vertical => style.size.height.clone(),
    };
    if !style.item_is_table
        && !style.item_is_replaced
        && known.inline.is_none()
        && !inline_size.is_min_content()
        && !inline_size.is_max_content()
    {
        known.inline = available_inline
            .map(|inline| inline.clamp_max_before_min_optional(min_size.inline, max_size.inline));
        if aspect_height_limit.is_some() {
            let physical_known = child_flow_axes.physical_size(known);
            known = child_flow_axes.logical_size(
                physical_known
                    .apply_aspect_ratio(style.aspect_ratio)
                    .clamp_max_before_min_optional(
                        child_flow_axes.physical_size(min_size),
                        child_flow_axes.physical_size(max_size),
                    ),
            );
        }
    }

    Ok(child_flow_axes.physical_size(known))
}

fn in_flow_child_available_inline<S: LayoutScalar>(
    style: &BlockChildProjection<S>,
    child_flow_axes: crate::geometry::FlowAxes,
    available_inline: Option<S>,
    fallback: AvailableOf<S>,
) -> AvailableOf<S> {
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => style.size.width.clone(),
        crate::PhysicalAxis::Vertical => style.size.height.clone(),
    };
    if inline_size.is_min_content() {
        AvailableOf::<S>::MIN_CONTENT
    } else if inline_size.is_max_content() {
        AvailableOf::<S>::MAX_CONTENT
    } else {
        available_inline
            .map(AvailableOf::<S>::definite)
            .unwrap_or(fallback)
    }
}

pub(super) fn relative_inset_offset<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    flow_axes: crate::geometry::FlowAxes,
) -> Point<S> {
    let logical_inset = flow_axes.logical_edges(inset);
    let logical_offset = LogicalPointOf::new(
        logical_inset
            .inline_start
            .or_else(|| logical_inset.inline_end.map(|end| -end))
            .unwrap_or(S::ZERO),
        logical_inset
            .block_start
            .or_else(|| logical_inset.block_end.map(|end| -end))
            .unwrap_or(S::ZERO),
    );
    flow_axes.physical_point(
        logical_offset,
        LogicalSizeOf::new(S::ZERO, S::ZERO),
        Size::ZERO,
    )
}

pub(crate) fn resolve_logical_in_flow_margin<S: LayoutScalar>(
    margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    child_size: LogicalSizeOf<S>,
    container_inline: Option<S>,
) -> LogicalEdgesOf<S> {
    let non_auto_inline = resolved_length_auto_fallback_zero(margin.inline_start)
        + resolved_length_auto_fallback_zero(margin.inline_end);
    let auto_count = usize::from(matches!(margin.inline_start, ResolvedLengthAutoOf::Auto))
        + usize::from(matches!(margin.inline_end, ResolvedLengthAutoOf::Auto));
    let auto_inline = if auto_count == 0 {
        S::ZERO
    } else {
        container_inline
            .map(|inline| (inline - child_size.inline - non_auto_inline).max(S::ZERO))
            .unwrap_or(S::ZERO)
            / S::from_usize(auto_count)
    };

    LogicalEdgesOf::new(
        resolved_length_auto_or(margin.inline_start, auto_inline),
        resolved_length_auto_or(margin.inline_end, auto_inline),
        resolved_length_auto_fallback_zero(margin.block_start),
        resolved_length_auto_fallback_zero(margin.block_end),
    )
}

fn resolved_length_auto_or<S: LayoutScalar>(value: ResolvedLengthAutoOf<S>, auto_fallback: S) -> S {
    match value {
        ResolvedLengthAutoOf::Auto => auto_fallback,
        ResolvedLengthAutoOf::Resolved(value) => value,
        // Missing-basis symbolic margins keep the algorithm's historical
        // unresolved-as-zero fallback and do not participate in auto distribution.
        ResolvedLengthAutoOf::Unresolved(
            UnresolvedLengthReason::Basis | UnresolvedLengthReason::InvalidNumeric,
        ) => S::ZERO,
    }
}

fn resolved_length_auto_fallback_zero<S: LayoutScalar>(value: ResolvedLengthAutoOf<S>) -> S {
    resolved_length_auto_or(value, S::ZERO)
}

fn in_flow_child_inline_offset<S: LayoutScalar>(
    size: LogicalSizeOf<S>,
    margin: LogicalEdgesOf<S>,
    constants: &Constants<S>,
) -> S {
    let logical_content_box_inset = constants.logical_content_box_inset();
    let logical_inner_size = constants.logical_node_inner_size();
    let mut inline = logical_content_box_inset.inline_start + margin.inline_start;

    let container_inner_inline = logical_inner_size
        .inline
        .or_else(|| {
            constants
                .logical_node_outer_size()
                .inline
                .map(|inline| inline - logical_content_box_inset.inline_sum())
        })
        .unwrap_or(size.inline + margin.inline_sum());
    let item_outer_inline = size.inline + margin.inline_sum();
    if item_outer_inline < container_inner_inline {
        let free_space = container_inner_inline - item_outer_inline;
        match constants.text_align {
            TextAlign::Auto => {}
            TextAlign::LegacyCenter => inline = inline + free_space / S::from_f64(2.0),
            TextAlign::LegacyLeft
                if constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Inline)
                    .is_decreasing() =>
            {
                inline = inline + free_space;
            }
            TextAlign::LegacyRight
                if !constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Inline)
                    .is_decreasing() =>
            {
                inline = inline + free_space;
            }
            TextAlign::LegacyLeft | TextAlign::LegacyRight => {}
        }
    }

    inline
}

pub(super) fn content_size_contribution<S: LayoutScalar>(
    location: Point<S>,
    size: Size<S>,
    content_size: Size<S>,
    overflow: ComputedOverflow,
    item_is_replaced: bool,
) -> Size<S> {
    let overflow = UsedOverflow::from_computed(overflow, item_is_replaced);
    let contribution_size = Size::new(
        if overflow.x().value() == Overflow::Visible {
            size.width.max(content_size.width)
        } else {
            size.width
        },
        if overflow.y().value() == Overflow::Visible {
            size.height.max(content_size.height)
        } else {
            size.height
        },
    );
    let max_x = (location.x + contribution_size.width).max(S::ZERO);
    let min_x = location.x.min(S::ZERO);
    let max_y = (location.y + contribution_size.height).max(S::ZERO);
    let min_y = location.y.min(S::ZERO);
    Size::new(max_x - min_x, max_y - min_y)
}

pub(super) fn block_final_in_flow_end<S: LayoutScalar>(
    content_box: crate::ScrollRectOf<S>,
    flow_axes: crate::FlowAxes,
    axis: crate::LogicalAxis,
    extent: S,
) -> S {
    let origin = content_box.origin();
    let size = content_box.size();
    let side = match axis {
        crate::LogicalAxis::Inline => flow_axes.inline_end(),
        crate::LogicalAxis::Block => flow_axes.block_end(),
    };
    match side {
        PhysicalSide::Top => origin.y + size.height - extent,
        PhysicalSide::Right => origin.x + extent,
        PhysicalSide::Bottom => origin.y + extent,
        PhysicalSide::Left => origin.x + size.width - extent,
    }
}
