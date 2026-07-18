use std::collections::BTreeMap;

use super::inline::{
    AtomicInlineBoxParticipant, ForcedLineBreakControlOf, InlineBoundaryControlOf,
    InlineControlAlignment, InlineFlowOf, InlineParticipant, InlineRunInput, InlineRunReport,
    ShapedTextParticipantOf, ShapedTextRunInputOf, layout_inline_run, layout_shaped_text_run,
};
use super::value::{ResolvedLengthAutoOf, UnresolvedLengthReason};
use super::{
    AspectRatioOf, AvailableOf, BaselinesOf, BoxSizing, Clear, CollapsibleMarginOf, Compute,
    ComputeInputOf, ComputeOutputOf, ComputedOverflow, ContainingLayoutContext, Direction, Edges,
    Float, InlineBoundaryInputOf, InlineFragmentOutputOf, LayoutErrorKindOf, LayoutErrorOf,
    LayoutErrorSiteOf, LayoutInputOf, LayoutInternalInvariant, LayoutOperation, LayoutResultOf,
    LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionOf, LengthResolutionStatus,
    LineBreakInputOf, NodeInputOf, NodeOutputOf, Overflow, ParentFormattingContext,
    PhysicalBlockMarginCollapseOf, Point, Position, RequestedAxis, RunMode, Size, SizingAlgorithm,
    SizingMode, TextAlign, Traverse, VerticalAlign, WritingMode,
};
use crate::compute::{
    EdgesResultExt, SizeResultExt, SizingResolutionError, resolve_maximum_optional,
    resolve_minimum_optional, resolve_preferred_optional,
};
use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalSide};
use crate::scroll::{
    CanonicalScrollBoxSourceOf, CanonicalScrollGeometryErrorOf, CanonicalScrollGeometrySourceOf,
    ClipMarginSourceOf, OptimalRegionInsetOf, OptimalRegionInsetsOf,
    ScrollContributionAccumulatorOf, ScrollOriginAxes, ScrollOriginProgression, UsedOverflow,
    canonical_scroll_box_from_source, canonical_scroll_geometry_from_source,
    rebuild_canonical_scroll_geometry_for_border_box, scrollbar_size_from_overflow,
};

pub(crate) fn compute_block<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let scrollbar_width = tree.node_input(node).scrollbar_width.get();
    let mut pass_input = input;
    loop {
        let output = compute_block_inner::<Tree, Tree::Scalar, M>(tree, node, pass_input)?;
        if !input.run_mode().is_perform_layout() {
            return Ok(output);
        }
        let Some(geometry) = output.scroll_geometry else {
            return Ok(output);
        };
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || scrollbar_width == Tree::Scalar::ZERO
            || !crate::scroll::settled_auto_scrollbars_change_available_geometry(
                geometry, next_state,
            )
            .map_err(|error| block_own_geometry_error(node, input.run_mode(), error))?
        {
            return Ok(output);
        }
        pass_input = input.with_settled_auto_scrollbars(next_state);
    }
}

fn edge_at_physical_side<T: Copy>(edges: Edges<T>, side: PhysicalSide) -> T {
    match side {
        PhysicalSide::Top => edges.top,
        PhysicalSide::Right => edges.right,
        PhysicalSide::Bottom => edges.bottom,
        PhysicalSide::Left => edges.left,
    }
}

fn compute_block_inner<Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let style = tree.node_input(node).clone();
    let constants = Constants::new::<Tree, M>(tree, node, &style, input)?;
    let children = tree.children(node).collect::<Vec<_>>();

    if children.is_empty()
        && input.run_mode() == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return Ok(ComputeOutputOf::<S>::from_outer_size(Size::new(
            width, height,
        )));
    }
    if input.run_mode() == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
        && !normal_flow_children_can_establish_baseline(tree, &children)
    {
        return Ok(ComputeOutputOf::<S>::from_outer_size(Size::new(
            width, height,
        )));
    }

    let logical_inner_size = constants.logical_node_inner_size();
    let needs_final_pass = input.run_mode().is_perform_layout()
        && (logical_inner_size.inline.is_none()
            || constants
                .flow_axes
                .logical_axis_progression(crate::LogicalAxis::Block)
                .is_decreasing()
                && logical_inner_size.block.is_none());
    let intrinsic_pass = layout_in_flow_children(
        tree,
        node,
        &children,
        &constants,
        input,
        logical_inner_size.inline,
        input.run_mode().is_perform_layout() && !needs_final_pass,
    )?;
    let logical_intrinsic_outer_size = LogicalSizeOf::new(
        intrinsic_pass.content_size.inline + constants.logical_content_box_inset().inline_sum(),
        intrinsic_pass.auto_block(&constants),
    );
    let logical_intrinsic_outer_size = LogicalSizeOf::new(
        logical_intrinsic_outer_size.inline.clamp_optional(
            constants.logical_node_min_size().inline,
            constants.logical_node_max_size().inline,
        ),
        logical_intrinsic_outer_size.block.clamp_optional(
            constants.logical_node_min_size().block,
            constants.logical_node_max_size().block,
        ),
    )
    .max_optional(constants.logical_padding_border_size().map(Some));
    let logical_outer_size = constants
        .logical_node_outer_size()
        .unwrap_or(logical_intrinsic_outer_size)
        .max_optional(constants.logical_padding_border_size().map(Some));
    let provisional_logical_output_size = constants
        .flow_axes
        .logical_size(input.known())
        .or(constants.logical_node_outer_size())
        .unwrap_or(logical_outer_size)
        .max_optional(constants.logical_padding_border_size().map(Some));
    let (final_constants, final_pass) = if needs_final_pass {
        let logical_inner_size = LogicalSizeOf::new(
            Some(
                (provisional_logical_output_size.inline
                    - constants.logical_content_box_inset().inline_sum())
                .max(S::ZERO),
            ),
            Some(
                (provisional_logical_output_size.block
                    - constants.logical_content_box_inset().block_sum())
                .max(S::ZERO),
            ),
        );
        let final_constants = constants.with_logical_node_inner_size(logical_inner_size);
        let final_pass = layout_in_flow_children(
            tree,
            node,
            &children,
            &final_constants,
            input,
            logical_inner_size.inline,
            true,
        )?;
        (final_constants, final_pass)
    } else {
        (constants, intrinsic_pass)
    };
    let logical_output_size = LogicalSizeOf::new(
        provisional_logical_output_size.inline,
        constants
            .flow_axes
            .logical_size(input.known())
            .block
            .or(final_constants.logical_node_outer_size().block)
            .unwrap_or_else(|| final_pass.auto_block(&final_constants))
            .clamp_optional(
                final_constants.logical_node_min_size().block,
                final_constants.logical_node_max_size().block,
            )
            .max(final_constants.logical_padding_border_size().block),
    );
    let output_size = final_constants.flow_axes.physical_size(logical_output_size);
    let top_margin = final_pass.top_margin(&final_constants);
    let bottom_margin = final_pass.bottom_margin(&final_constants);
    let margins_can_collapse_through = final_constants.can_collapse_through
        && final_pass.all_in_flow_children_can_collapse_through;
    let block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
        final_constants.flow_axes,
        top_margin,
        bottom_margin,
        margins_can_collapse_through,
    );

    if input.run_mode() == RunMode::ComputeSize {
        let mut output = ComputeOutputOf::<S>::from_sizes_and_baselines(
            output_size,
            Size::ZERO,
            final_pass.baselines,
        );
        output.block_margin_collapse = block_margin_collapse;
        Ok(output)
    } else {
        let final_scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes: final_constants.flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            border_box_size: output_size,
            border: final_constants.border,
            padding: final_constants.padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: final_constants.settled_auto_scrollbars,
        })
        .map_err(|error| block_own_geometry_error(node, input.run_mode(), error))?;
        let mut contributions = final_pass.contributions;
        contributions.replace_container_seed(final_scroll_box.padding_box());
        contributions.exclude_reserved_gutter_from_range();
        for (axis, extent) in [
            (crate::LogicalAxis::Inline, final_pass.content_size.inline),
            (crate::LogicalAxis::Block, final_pass.content_size.block),
        ] {
            contributions
                .record_final_in_flow_end(
                    final_constants.flow_axes,
                    axis,
                    block_final_in_flow_end(
                        final_scroll_box.content_box(),
                        final_constants.flow_axes,
                        axis,
                        extent,
                    ),
                )
                .map_err(|error| block_own_geometry_error(node, input.run_mode(), error))?;
        }
        layout_floats(
            tree,
            node,
            &final_pass.pending_floats,
            output_size,
            &final_constants,
            &mut contributions,
        )?;
        layout_absolute_children(
            tree,
            node,
            &children,
            &final_pass.static_positions,
            output_size,
            &final_constants,
            &mut contributions,
        )?;
        contributions
            .include_terminal_padding(final_constants.padding)
            .map_err(|error| block_own_geometry_error(node, input.run_mode(), error))?;
        let scroll_geometry = block_scroll_geometry::<Tree, S, M>(
            node,
            input.run_mode(),
            &style,
            &final_constants,
            output_size,
            contributions,
        )?;
        let content_size = contributions
            .content_size_from_anchor(scroll_geometry.content_box().origin())
            .map_err(|error| block_own_geometry_error(node, input.run_mode(), error))?;
        let mut output = ComputeOutputOf::<S>::from_sizes_and_baselines(
            output_size,
            content_size,
            final_pass.baselines,
        );
        output.scroll_geometry = Some(scroll_geometry);
        output.block_margin_collapse = block_margin_collapse;
        Ok(output)
    }
}

fn normal_flow_children_can_establish_baseline<Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
) -> bool
where
    Tree: Compute<M>,
{
    children.iter().copied().any(|child| {
        let style = match tree.layout_input(child) {
            LayoutInputOf::InlineText(_) => return true,
            LayoutInputOf::Box(style) => style,
            LayoutInputOf::LineBreak(_) | LayoutInputOf::InlineBoundary(_) => return false,
        };
        if style.display == super::Display::None
            || style.position == Position::Absolute
            || style.float != Float::None
        {
            return false;
        }

        style.display.is_inline_level()
            || style.display.inner_display() == super::Display::Block
            || tree.child_count(child) > 0 && style.display.inner_display() == super::Display::Flex
    })
}

struct PendingFloat<Node, S: LayoutScalar> {
    node: Node,
    source_index: usize,
    side: Float,
    clear: Clear,
    y: S,
    size: Size<S>,
    content_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
    margin: Edges<S>,
    style: Box<NodeInputOf<S>>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
}

#[derive(Clone, Copy, Debug)]
struct ActiveFloat<S: LayoutScalar> {
    side: Float,
    x: S,
    y: S,
    width: S,
    height: S,
}

impl<S: LayoutScalar> ActiveFloat<S> {
    fn bottom(self) -> S {
        self.y + self.height
    }

    fn overlaps_y(self, y: S) -> bool {
        y >= self.y && y < self.bottom()
    }
}

#[derive(Clone, Debug)]
struct FloatExclusions<S: LayoutScalar> {
    content_width: S,
    inset: Edges<S>,
    active: Vec<ActiveFloat<S>>,
}

impl<S: LayoutScalar> FloatExclusions<S> {
    fn new(content_width: S, inset: Edges<S>) -> Self {
        Self {
            content_width,
            inset,
            active: Vec::new(),
        }
    }

    fn place_float<Node>(&mut self, float: &PendingFloat<Node, S>, y: S) -> Point<S> {
        let margin_box = float.size + float.margin.sum_axes();
        let mut candidate_y = self.clearance_y(y, float.clear);

        loop {
            let (left_edge, right_edge, next_y) = self.available_band(candidate_y);
            let available_width = (right_edge - left_edge).max(S::ZERO);
            if margin_box.width <= available_width || next_y.is_none() {
                let location = match float.side {
                    Float::Left | Float::None => Point::new(
                        left_edge + float.margin.left,
                        candidate_y + float.margin.top,
                    ),
                    Float::Right => Point::new(
                        right_edge - float.margin.right - float.size.width,
                        candidate_y + float.margin.top,
                    ),
                };
                self.active.push(ActiveFloat {
                    side: float.side,
                    x: location.x - float.margin.left,
                    y: location.y - float.margin.top,
                    width: margin_box.width,
                    height: margin_box.height,
                });
                return location;
            }
            candidate_y = next_y.unwrap();
        }
    }

    fn place_bfc_block(
        &self,
        y: S,
        size: Size<S>,
        margin: Edges<S>,
        clear: Clear,
        fallback_x: S,
    ) -> Point<S> {
        let mut candidate_y = self.clearance_y(y, clear);
        loop {
            let (left_edge, right_edge, next_y) = self.available_band(candidate_y);
            let margin_box_width = size.width + margin.horizontal_sum();
            let fallback_left = fallback_x - margin.left;
            let fallback_right = fallback_x + size.width + margin.right;
            if margin_box_width <= (right_edge - left_edge).max(S::ZERO) {
                if fallback_left >= left_edge && fallback_right <= right_edge {
                    return Point::new(fallback_x, candidate_y);
                }
                return Point::new(left_edge + margin.left, candidate_y);
            }
            if let Some(next_y) = next_y {
                candidate_y = next_y;
            } else {
                return Point::new(fallback_x, candidate_y);
            }
        }
    }

    fn clearance_y(&self, y: S, clear: Clear) -> S {
        let clears_left = matches!(clear, Clear::Left | Clear::Both);
        let clears_right = matches!(clear, Clear::Right | Clear::Both);
        if !clears_left && !clears_right {
            return y;
        }

        self.active
            .iter()
            .copied()
            .filter(|float| {
                (clears_left && float.side == Float::Left)
                    || (clears_right && float.side == Float::Right)
            })
            .map(ActiveFloat::bottom)
            .fold(y, S::max)
    }

    fn available_band(&self, y: S) -> (S, S, Option<S>) {
        let mut left_edge = self.inset.left;
        let mut right_edge = self.inset.left + self.content_width;
        let mut next_y = None;

        for float in self
            .active
            .iter()
            .copied()
            .filter(|float| float.overlaps_y(y))
        {
            match float.side {
                Float::Left => left_edge = left_edge.max(float.x + float.width),
                Float::Right => right_edge = right_edge.min(float.x),
                Float::None => {}
            }
            next_y = Some(next_y.map_or(float.bottom(), |current: S| current.min(float.bottom())));
        }

        (left_edge, right_edge, next_y)
    }
}

struct InFlowResult<Node, S: LayoutScalar> {
    content_size: LogicalSizeOf<S>,
    contributions: ScrollContributionAccumulatorOf<S>,
    baselines: BaselinesOf<S>,
    static_positions: Vec<(Node, Point<S>)>,
    pending_floats: Vec<PendingFloat<Node, S>>,
    cursor_block: S,
    top_margin: CollapsibleMarginOf<S>,
    active_margin: CollapsibleMarginOf<S>,
    active_margin_can_collapse_with_parent: bool,
    all_in_flow_children_can_collapse_through: bool,
}

impl<Node, S: LayoutScalar> InFlowResult<Node, S> {
    fn top_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if constants.collapse_top_margin {
            self.top_margin
        } else {
            constants.own_top_margin
        }
    }

    fn bottom_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
            self.active_margin
        } else {
            constants.own_bottom_margin
        }
    }

    fn auto_block(&self, constants: &Constants<S>) -> S {
        let bottom_margin_offset =
            if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
                S::ZERO
            } else {
                self.active_margin.resolve()
            };
        let content_box_inset = constants.logical_content_box_inset();
        (self.cursor_block + bottom_margin_offset + content_box_inset.block_end)
            .max(content_box_inset.block_sum())
    }
}

fn inline_run_end<Tree, M>(
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
                if style.display == super::Display::None || style.position == Position::Absolute {
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

fn visible_line_break_in_flow<Tree, M>(
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
    if crate::geometry::FlowAxes::new(flow_writing_mode, flow_direction).inline_axis()
        == PhysicalAxis::Vertical
        && line_break.clear() != Clear::None
    {
        panic!("vertical line-break clear layout is not implemented");
    }
    if line_break.writing_mode() != flow_writing_mode || line_break.direction() != flow_direction {
        panic!("line-break flow must match containing inline flow");
    }
    Some(line_break)
}

fn visible_inline_boundary_in_flow<Tree, M>(
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InlineClearCandidate {
    end: usize,
    clear: Clear,
}

fn next_inline_clear_candidate<Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    start: usize,
    run_end: usize,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<InlineClearCandidate>
where
    Tree: Compute<M>,
{
    for (index, child) in children
        .iter()
        .copied()
        .enumerate()
        .take(run_end)
        .skip(start)
    {
        if let Some(line_break) =
            visible_line_break_in_flow(tree, child, flow_writing_mode, flow_direction)
        {
            if crate::geometry::FlowAxes::new(flow_writing_mode, flow_direction).inline_axis()
                == PhysicalAxis::Vertical
            {
                continue;
            }
            let clear = line_break.clear();
            if clear != Clear::None {
                return Some(InlineClearCandidate {
                    end: index + 1,
                    clear,
                });
            }
        }
    }
    None
}

fn inline_run_contains_clear<Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    run_start: usize,
    run_end: usize,
    constants: &Constants<<Tree as Traverse>::Scalar>,
) -> bool
where
    Tree: Compute<M>,
{
    next_inline_clear_candidate(
        tree,
        children,
        run_start,
        run_end,
        constants.writing_mode,
        constants.direction,
    )
    .is_some()
}

fn layout_in_flow_children<Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<S>,
    input: ComputeInputOf<S>,
    inner_inline: Option<S>,
    set_layout: bool,
) -> LayoutResultOf<<Tree as Traverse>::Node, InFlowResult<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let logical_node_inner_size =
        LogicalSizeOf::new(inner_inline, constants.logical_node_inner_size().block);
    let node_inner_size = constants.flow_axes.physical_size(logical_node_inner_size);
    let mut cursor_block = constants.logical_content_box_inset().block_start;
    let mut float_bfc_cursor_y = constants.content_box_inset.top;
    let mut content_size = LogicalSizeOf::new(S::ZERO, S::ZERO);
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
    let content_width = inner_inline
        .or(constants.available_content.width.into_option())
        .unwrap_or(S::ZERO);
    let mut float_exclusions = FloatExclusions::new(content_width, constants.content_box_inset);
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
    let contribution_seed = super::ScrollRectOf::try_new(content_box_origin, content_box_size)
        .map_err(|error| block_own_geometry_error(node, input.run_mode(), error))?;
    let mut contributions = ScrollContributionAccumulatorOf::new(contribution_seed);

    let mut index = 0;
    while index < children.len() {
        let source_index = index;
        let child = children[index];
        let child_style = match tree.layout_input(child) {
            LayoutInputOf::Box(style) => *style,
            LayoutInputOf::InlineText(_) => {
                let run_start = index;
                index = inline_run_end(tree, children, constants, index + 1);

                let collapsed_margin = active_margin.resolve();
                cursor_block = cursor_block + collapsed_margin;
                float_bfc_cursor_y = float_bfc_cursor_y + collapsed_margin;
                if is_collapsing_first_margin {
                    is_collapsing_first_margin = false;
                }

                let placement = layout_inline_run_with_clear(
                    tree,
                    node,
                    children,
                    run_start..index,
                    InlineRunContext {
                        source_index_start: run_start,
                        cursor_block,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    &float_exclusions,
                    &mut contributions,
                )?;
                let placement_content_size =
                    constants.flow_axes.logical_size(placement.content_size);
                content_size.inline = content_size.inline.max(placement_content_size.inline);
                content_size.block = content_size.block.max(placement_content_size.block);
                record_inline_run_baselines(&mut baselines, &placement, cursor_block, constants);
                static_positions.extend(placement.static_positions);
                cursor_block =
                    cursor_block + constants.flow_axes.logical_size(placement.size).block;
                float_bfc_cursor_y = float_bfc_cursor_y + placement.size.height;
                active_margin = CollapsibleMarginOf::<S>::ZERO;
                active_margin_can_collapse_with_parent = false;
                all_in_flow_children_can_collapse_through = false;
                continue;
            }
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

                let collapsed_margin = active_margin.resolve();
                cursor_block = cursor_block + collapsed_margin;
                float_bfc_cursor_y = float_bfc_cursor_y + collapsed_margin;
                if is_collapsing_first_margin {
                    is_collapsing_first_margin = false;
                }

                let placement = layout_inline_run_with_clear(
                    tree,
                    node,
                    children,
                    run_start..index,
                    InlineRunContext {
                        source_index_start: run_start,
                        cursor_block,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    &float_exclusions,
                    &mut contributions,
                )?;
                let placement_content_size =
                    constants.flow_axes.logical_size(placement.content_size);
                content_size.inline = content_size.inline.max(placement_content_size.inline);
                content_size.block = content_size.block.max(placement_content_size.block);
                record_inline_run_baselines(&mut baselines, &placement, cursor_block, constants);
                static_positions.extend(placement.static_positions);
                cursor_block =
                    cursor_block + constants.flow_axes.logical_size(placement.size).block;
                float_bfc_cursor_y = float_bfc_cursor_y + placement.size.height;
                active_margin = CollapsibleMarginOf::<S>::ZERO;
                active_margin_can_collapse_with_parent = false;
                all_in_flow_children_can_collapse_through = false;
                continue;
            }
            LayoutInputOf::InlineBoundary(_) => {
                visible_inline_boundary_in_flow(
                    tree,
                    child,
                    constants.writing_mode,
                    constants.direction,
                );

                let run_start = index;
                index = inline_run_end(tree, children, constants, index + 1);

                let collapsed_margin = active_margin.resolve();
                cursor_block = cursor_block + collapsed_margin;
                float_bfc_cursor_y = float_bfc_cursor_y + collapsed_margin;
                if is_collapsing_first_margin {
                    is_collapsing_first_margin = false;
                }

                let placement = layout_inline_run_with_clear(
                    tree,
                    node,
                    children,
                    run_start..index,
                    InlineRunContext {
                        source_index_start: run_start,
                        cursor_block,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    &float_exclusions,
                    &mut contributions,
                )?;
                let placement_content_size =
                    constants.flow_axes.logical_size(placement.content_size);
                content_size.inline = content_size.inline.max(placement_content_size.inline);
                content_size.block = content_size.block.max(placement_content_size.block);
                record_inline_run_baselines(&mut baselines, &placement, cursor_block, constants);
                static_positions.extend(placement.static_positions);
                cursor_block =
                    cursor_block + constants.flow_axes.logical_size(placement.size).block;
                float_bfc_cursor_y = float_bfc_cursor_y + placement.size.height;
                active_margin = CollapsibleMarginOf::<S>::ZERO;
                active_margin_can_collapse_with_parent = false;
                all_in_flow_children_can_collapse_through = false;
                continue;
            }
        };
        if child_style.display == super::Display::None {
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
            index += 1;
            continue;
        }
        if child_style.position == Position::Absolute {
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

        if child_style.display.is_inline_level() && child_style.float.is_none() {
            let run_start = index;
            index = inline_run_end(tree, children, constants, index + 1);

            let collapsed_margin = active_margin.resolve();
            cursor_block = cursor_block + collapsed_margin;
            float_bfc_cursor_y = float_bfc_cursor_y + collapsed_margin;
            if is_collapsing_first_margin {
                is_collapsing_first_margin = false;
            }

            let placement = layout_inline_run_with_clear(
                tree,
                node,
                children,
                run_start..index,
                InlineRunContext {
                    source_index_start: run_start,
                    cursor_block,
                    constants,
                    input,
                    node_inner_size,
                    set_layout,
                },
                &float_exclusions,
                &mut contributions,
            )?;
            let placement_content_size = constants.flow_axes.logical_size(placement.content_size);
            content_size.inline = content_size.inline.max(placement_content_size.inline);
            content_size.block = content_size.block.max(placement_content_size.block);
            record_inline_run_baselines(&mut baselines, &placement, cursor_block, constants);
            static_positions.extend(placement.static_positions);
            cursor_block = cursor_block + constants.flow_axes.logical_size(placement.size).block;
            float_bfc_cursor_y = float_bfc_cursor_y + placement.size.height;
            active_margin = CollapsibleMarginOf::<S>::ZERO;
            active_margin_can_collapse_with_parent = false;
            all_in_flow_children_can_collapse_through = false;
            continue;
        }

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
        let child_flow_axes =
            crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction);
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
        let output = tree.compute_child(
            child,
            ComputeInputOf::<S>::for_child(
                input.run_mode().for_child(),
                SizingMode::InherentSize,
                RequestedAxis::Both,
                child_known,
                child_parent_size,
                ContainingLayoutContext::new(
                    constants.flow_axes,
                    ParentFormattingContext::BlockFlow,
                ),
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
            .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars()),
        )?;

        let logical_child_size = constants.flow_axes.logical_size(output.size);
        let logical_child_margin = resolve_logical_in_flow_margin(
            parent_logical_unresolved_margin,
            logical_child_size,
            logical_node_inner_size
                .inline
                .or(parent_logical_available.inline.into_option()),
        );
        let child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        if !child_style.float.is_none() {
            let margin_box = output.size + child_margin.sum_axes();
            float_intrinsics.add(margin_box.width, child_style.float, child_style.clear);
            let pending_float = PendingFloat {
                node: child,
                source_index,
                side: child_style.float,
                clear: child_style.clear,
                y: float_bfc_cursor_y,
                size: output.size,
                content_size: output.content_size,
                border: child_border,
                padding: child_padding,
                margin: child_margin,
                style: Box::new(child_style),
                child_compute_geometry: output.scroll_geometry,
            };
            let float_location = float_exclusions.place_float(&pending_float, float_bfc_cursor_y);
            if set_layout {
                pending_floats.push(pending_float);
            }
            let float_content_size = constants.flow_axes.logical_size(Size::new(
                float_intrinsics.result(),
                float_location.y - constants.content_box_inset.top
                    + output.size.height
                    + child_margin.bottom,
            ));
            content_size.inline = content_size.inline.max(float_content_size.inline);
            content_size.block = content_size.block.max(float_content_size.block);
            index += 1;
            continue;
        }
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
            .collapse_with_margin(edge_at_physical_side(
                child_margin,
                constants.flow_axes.block_start(),
            ));
        let bottom_margin_set = output
            .block_margin_collapse
            .at(constants.flow_axes.block_end())
            .collapse_with_margin(edge_at_physical_side(
                child_margin,
                constants.flow_axes.block_end(),
            ));
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
        float_bfc_cursor_y = float_bfc_cursor_y + collapsed_margin;
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
        let establishes_bfc = !child_style.item_is_replaced
            && child_style
                .overflow
                .establishes_independent_formatting_context();
        let location = if establishes_bfc {
            let placement = float_exclusions.place_bfc_block(
                float_bfc_cursor_y,
                output.size,
                child_margin,
                child_style.clear,
                fallback_location.x - inset_offset.x,
            );
            Point::new(placement.x + inset_offset.x, placement.y + inset_offset.y)
        } else if child_style.clear != Clear::None {
            Point::new(
                fallback_location.x,
                float_exclusions.clearance_y(float_bfc_cursor_y, child_style.clear)
                    + inset_offset.y,
            )
        } else {
            fallback_location
        };
        if set_layout {
            let scroll_geometry = retained_child_scroll_geometry(
                &child_style,
                output.size,
                output.content_size,
                child_padding,
                child_border,
                output.scroll_geometry,
            )
            .map_err(|error| block_child_geometry_error(node, child, error))?;
            contributions
                .include_in_flow_geometry(location, child_margin, scroll_geometry)
                .map_err(|error| block_child_geometry_error(node, child, error))?;
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

        let child_block_end = if establishes_bfc || child_style.clear != Clear::None {
            constants
                .flow_axes
                .logical_point(location, output.size, containing_size)
                .block
                + logical_child_size.block
        } else {
            logical_location.block + logical_child_size.block
        };
        let child_physical_bottom = (location.y - inset_offset.y) + output.size.height;
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
        content_size.inline = content_size
            .inline
            .max(logical_child_margin.inline_sum() + logical_child_size.inline)
            .max(logical_contribution.inline + logical_child_margin.inline_end);
        content_size.block = content_size
            .block
            .max(logical_contribution.block)
            .max(child_block_end - constants.logical_content_box_inset().block_start);
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
            float_bfc_cursor_y = if child_style.clear == Clear::None {
                float_bfc_cursor_y + output.size.height
            } else {
                child_physical_bottom
            };
            active_margin = active_margin
                .collapse_with(top_margin_set)
                .collapse_with(bottom_margin_set);
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        } else {
            all_in_flow_children_can_collapse_through = false;
            cursor_block = child_block_end;
            float_bfc_cursor_y = child_physical_bottom;
            active_margin = bottom_margin_set;
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        }
        index += 1;
    }

    Ok(InFlowResult {
        content_size,
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

struct InlineRunPlacement<Node, S: LayoutScalar> {
    size: Size<S>,
    content_size: Size<S>,
    static_positions: Vec<(Node, Point<S>)>,
    baselines: BaselinesOf<S>,
    first_baseline: Option<S>,
    last_baseline: Option<S>,
}

struct InlineRunContext<'a, S: LayoutScalar> {
    source_index_start: usize,
    cursor_block: S,
    constants: &'a Constants<S>,
    input: ComputeInputOf<S>,
    node_inner_size: Size<Option<S>>,
    set_layout: bool,
}

struct InlineSegmentsContext<'a, S: LayoutScalar> {
    source_index_start: usize,
    cursor_block: S,
    constants: &'a Constants<S>,
    input: ComputeInputOf<S>,
    node_inner_size: Size<Option<S>>,
    set_layout: bool,
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

enum InlineRunChild<Node, S: LayoutScalar> {
    Box {
        child: Node,
        source_index: usize,
        style: Box<NodeInputOf<S>>,
        output: ComputeOutputOf<S>,
    },
    LineBreak {
        child: Node,
        source_index: usize,
    },
    Boundary {
        child: Node,
        source_index: usize,
    },
}

fn layout_inline_segments<Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    run: &[<Tree as Traverse>::Node],
    context: InlineSegmentsContext<'_, S>,
    float_exclusions: &FloatExclusions<S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InlineRunPlacement<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let InlineSegmentsContext {
        source_index_start,
        mut cursor_block,
        constants,
        input,
        node_inner_size,
        set_layout,
    } = context;
    let mut offset = 0;
    let mut content_size: Size<S> = Size::ZERO;
    let mut static_positions = Vec::new();
    let mut first_baseline = None;
    let mut last_baseline = None;
    let start_y = cursor_block;

    while offset < run.len() {
        let mut segment_end = run.len();
        let mut segment_clear = Clear::None;
        let mut scan_start = offset;
        while let Some(candidate) = next_inline_clear_candidate(
            tree,
            run,
            scan_start,
            run.len(),
            constants.writing_mode,
            constants.direction,
        ) {
            let probe = layout_inline_run_children(
                tree,
                container,
                &run[offset..candidate.end],
                InlineRunContext {
                    source_index_start: source_index_start + offset,
                    cursor_block,
                    constants,
                    input,
                    node_inner_size,
                    set_layout: false,
                },
                contributions,
            )?;
            let segment_bottom = cursor_block + probe.size.height;
            if float_exclusions.clearance_y(segment_bottom, candidate.clear) > segment_bottom {
                segment_end = candidate.end;
                segment_clear = candidate.clear;
                break;
            }
            scan_start = candidate.end;
        }

        let placement = layout_inline_run_children(
            tree,
            container,
            &run[offset..segment_end],
            InlineRunContext {
                source_index_start: source_index_start + offset,
                cursor_block,
                constants,
                input,
                node_inner_size,
                set_layout,
            },
            contributions,
        )?;

        content_size.width = content_size.width.max(placement.content_size.width);
        content_size.height = content_size.height.max(placement.content_size.height);
        static_positions.extend(placement.static_positions);
        if let Some(baseline) = placement.first_baseline {
            first_baseline.get_or_insert(cursor_block - start_y + baseline);
        }
        if let Some(baseline) = placement.last_baseline {
            last_baseline = Some(cursor_block - start_y + baseline);
        }

        cursor_block = cursor_block + placement.size.height;
        if segment_clear != Clear::None {
            cursor_block = float_exclusions.clearance_y(cursor_block, segment_clear);
            content_size.height = content_size
                .height
                .max(cursor_block - constants.content_box_inset.top);
        }
        offset = segment_end;
    }

    Ok(InlineRunPlacement {
        size: Size::new(content_size.width, cursor_block - start_y),
        content_size,
        static_positions,
        baselines: BaselinesOf::NONE,
        first_baseline,
        last_baseline,
    })
}

fn layout_inline_run_with_clear<Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    run: core::ops::Range<usize>,
    context: InlineRunContext<'_, S>,
    float_exclusions: &FloatExclusions<S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InlineRunPlacement<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let run_start = run.start;
    let run_end = run.end;
    if !inline_run_contains_clear(tree, children, run_start, run_end, context.constants) {
        return layout_inline_run_children(
            tree,
            container,
            &children[run_start..run_end],
            context,
            contributions,
        );
    }

    layout_inline_segments(
        tree,
        container,
        &children[run_start..run_end],
        InlineSegmentsContext {
            source_index_start: context.source_index_start,
            cursor_block: context.cursor_block,
            constants: context.constants,
            input: context.input,
            node_inner_size: context.node_inner_size,
            set_layout: context.set_layout,
        },
        float_exclusions,
        contributions,
    )
}

fn layout_shaped_text_child<Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    child: <Tree as Traverse>::Node,
    text: &super::InlineTextInputOf<S>,
    context: InlineRunContext<'_, S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InlineRunPlacement<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let InlineRunContext {
        source_index_start,
        cursor_block,
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
    let participants = text
        .segments()
        .iter()
        .copied()
        .map(|segment| ShapedTextParticipantOf {
            source_index: source_index_start,
            segment,
        })
        .collect();
    let report = layout_shaped_text_run(ShapedTextRunInputOf {
        available_inline_extent,
        flow_axes: constants.flow_axes,
        text_align: constants.text_align,
        participants,
    });
    let report_logical_size = LogicalSizeOf::new(report.inline_extent, report.block_extent);
    let report_size = constants.flow_axes.physical_size(report_logical_size);
    let logical_content_box_inset = constants.logical_content_box_inset();
    let containing_size = constants.containing_size(logical_node_inner_size);
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

    let mut fragments = Vec::with_capacity(report.fragments.len());
    let mut union_min = None;
    let mut union_max = None;
    for source in &report.fragments {
        let logical_size = LogicalSizeOf::new(source.inline_extent, source.block_extent);
        let size = constants.flow_axes.physical_size(logical_size);
        let location = project_point(source.inline_start, source.block_start, logical_size);
        let rect = super::ScrollRectOf::try_new(location, size).map_err(|error| {
            block_inline_geometry_error(container, Some(child), input.run_mode(), error)
        })?;
        let baseline = project_point(
            source.inline_start,
            source.baseline,
            LogicalSizeOf::new(S::ZERO, S::ZERO),
        );
        union_min = Some(union_min.map_or(location, |current: Point<S>| {
            Point::new(current.x.min(location.x), current.y.min(location.y))
        }));
        let maximum = Point::new(location.x + size.width, location.y + size.height);
        union_max = Some(union_max.map_or(maximum, |current: Point<S>| {
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
    }

    let anchor = report
        .anchors
        .iter()
        .find(|anchor| anchor.source_index == source_index_start)
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

    if set_layout {
        tree.set_unrounded(
            child,
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(source_index_start),
                location: text_location,
                size: text_size,
                content_size: text_size,
                ..NodeOutputOf::new()
            },
        );
        tree.set_unrounded_inline_fragment_state(child, Some(fragments));
        let direct_line = super::ScrollRectOf::try_new(
            project_point(S::ZERO, S::ZERO, report_logical_size),
            report_size,
        )
        .map_err(|error| {
            block_inline_geometry_error(container, Some(child), input.run_mode(), error)
        })?;
        contributions.include_direct_line(direct_line);
    }

    Ok(InlineRunPlacement {
        size: report_size,
        content_size: report_size,
        static_positions: Vec::new(),
        baselines: BaselinesOf::NONE,
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
    })
}

fn layout_inline_run_children<Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    run: &[<Tree as Traverse>::Node],
    context: InlineRunContext<'_, S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InlineRunPlacement<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    if let Some((text_offset, child, text)) =
        run.iter()
            .copied()
            .enumerate()
            .find_map(|(offset, child)| match tree.layout_input(child) {
                LayoutInputOf::InlineText(text) => Some((offset, child, text)),
                _ => None,
            })
    {
        if run.len() != 1 {
            return Err(crate::LayoutErrorOf::new(
                crate::LayoutErrorSiteOf::Node(child),
                crate::LayoutOperation::ChildLayout,
                crate::LayoutErrorKindOf::UnsupportedCapability(
                    crate::LayoutUnsupportedCapability::LaterFriBehavior,
                ),
            ));
        }
        return layout_shaped_text_child(
            tree,
            container,
            child,
            &text,
            InlineRunContext {
                source_index_start: context.source_index_start + text_offset,
                ..context
            },
            contributions,
        );
    }

    let InlineRunContext {
        source_index_start,
        cursor_block,
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
    let mut items = Vec::with_capacity(run.len());
    let mut run_children = Vec::with_capacity(run.len());
    let mut static_positions = Vec::new();
    for (offset, child) in run.iter().copied().enumerate() {
        let source_index = source_index_start + offset;
        let child_style = match tree.layout_input(child) {
            LayoutInputOf::Box(style) => *style,
            LayoutInputOf::InlineText(_) => {
                return Err(crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(child),
                    crate::LayoutOperation::ChildLayout,
                    crate::LayoutErrorKindOf::UnsupportedCapability(
                        crate::LayoutUnsupportedCapability::LaterFriBehavior,
                    ),
                ));
            }
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
                .unwrap();

                run_children.push(InlineRunChild::LineBreak {
                    child,
                    source_index,
                });
                items.push(InlineParticipant::forced_line_break(
                    forced_line_break_control(source_index, line_break, available_inline_extent),
                ));
                continue;
            }
            LayoutInputOf::InlineBoundary(_) => {
                let boundary = visible_inline_boundary_in_flow(
                    tree,
                    child,
                    constants.writing_mode,
                    constants.direction,
                )
                .unwrap();

                run_children.push(InlineRunChild::Boundary {
                    child,
                    source_index,
                });
                items.push(InlineParticipant::inline_boundary(inline_boundary_control(
                    source_index,
                    boundary,
                    available_inline_extent,
                )));
                continue;
            }
        };
        if child_style.display == super::Display::None {
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
                constants
                    .flow_axes
                    .physical_size(LogicalSizeOf::new(logical_node_inner_size.inline, None)),
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

        let child_flow_axes =
            crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction);
        let item = InlineParticipant::Box(AtomicInlineBoxParticipant {
            source_index,
            size: output.size,
            content_size: output.content_size,
            margin: child_margin,
            padding: child_padding,
            border: child_border,
            scrollbar_size: child_scrollbar_size(&child_style),
            first_baseline: if child_style.vertical_align == VerticalAlign::Top {
                Some(S::ZERO)
            } else {
                output
                    .baselines()
                    .last_block(child_flow_axes)
                    .or_else(|| output.baselines().first_block(child_flow_axes))
            },
        });
        run_children.push(InlineRunChild::Box {
            child,
            source_index,
            style: Box::new(child_style),
            output,
        });
        items.push(item);
    }

    let report = layout_inline_run(InlineRunInput {
        available_width: available_inline_extent,
        writing_mode: constants.writing_mode,
        direction: constants.direction,
        items,
    });
    let run_offset = inline_run_offset(
        constants.flow_axes.logical_size(report.size).inline,
        constants,
        logical_node_inner_size.inline,
    );
    let logical_content_box_inset = constants.logical_content_box_inset();
    let report_location = project_inline_report_item(
        Point::ZERO,
        report.size,
        report.size,
        cursor_block,
        logical_content_box_inset.inline_start + run_offset,
        constants,
        containing_size,
    );
    if set_layout {
        let direct_line =
            super::ScrollRectOf::try_new(report_location, report.size).map_err(|error| {
                block_inline_geometry_error(
                    container,
                    run.first().copied(),
                    input.run_mode(),
                    error,
                )
            })?;
        contributions.include_direct_line(direct_line);
    }
    let mut content_size = content_size_contribution(
        Point::new(
            report_location.x - constants.content_box_inset.left,
            report_location.y - constants.content_box_inset.top,
        ),
        report.size,
        report.content_size,
        ComputedOverflow::VISIBLE,
        false,
    );

    let report_items_by_source_index = report
        .items
        .iter()
        .copied()
        .map(|item| (item.source_index, item))
        .collect::<BTreeMap<_, _>>();
    for run_child in &run_children {
        match run_child {
            InlineRunChild::Box {
                child,
                source_index,
                style: child_style,
                output,
            } => {
                let item = report_items_by_source_index[source_index];
                let inset_offset = relative_inset_offset(
                    constants
                        .flow_axes
                        .zip_physical_edges_with_inline_extent(
                            child_style.inset,
                            node_inner_size,
                            |length, basis| resolve_auto_optional(length, basis),
                        )
                        .transpose_with_node(tree, *child)?,
                    constants.flow_axes,
                );
                let projected_location = project_inline_report_item(
                    item.location,
                    item.size,
                    report.size,
                    cursor_block,
                    logical_content_box_inset.inline_start + run_offset,
                    constants,
                    containing_size,
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
                    item.size,
                    output.content_size,
                    child_style.overflow,
                    child_style.item_is_replaced,
                );
                content_size = max_content_size(content_size, contribution);

                if set_layout {
                    let scroll_geometry = retained_child_scroll_geometry(
                        child_style,
                        item.size,
                        item.content_size,
                        item.padding,
                        item.border,
                        output.scroll_geometry,
                    )
                    .map_err(|error| block_child_geometry_error(container, *child, error))?;
                    contributions
                        .include_in_flow_geometry(location, item.margin, scroll_geometry)
                        .map_err(|error| block_child_geometry_error(container, *child, error))?;
                    tree.set_unrounded(
                        *child,
                        NodeOutputOf::<S> {
                            source_index: crate::SourceIndex::new(item.source_index),
                            location,
                            size: item.size,
                            content_size: item.content_size,
                            border: item.border,
                            padding: item.padding,
                            margin: item.margin,
                            ..NodeOutputOf::new()
                        }
                        .with_scroll_geometry(Some(scroll_geometry)),
                    );
                }
            }
            InlineRunChild::LineBreak {
                child,
                source_index,
            } => {
                if set_layout {
                    let item = report_items_by_source_index[source_index];
                    tree.set_unrounded(
                        *child,
                        NodeOutputOf::<S> {
                            source_index: crate::SourceIndex::new(item.source_index),
                            location: project_inline_report_item(
                                item.location,
                                Size::ZERO,
                                report.size,
                                cursor_block,
                                logical_content_box_inset.inline_start + run_offset,
                                constants,
                                containing_size,
                            ),
                            ..NodeOutputOf::new()
                        },
                    );
                }
            }
            InlineRunChild::Boundary {
                child,
                source_index,
            } => {
                if set_layout {
                    let item = report_items_by_source_index[source_index];
                    tree.set_unrounded(
                        *child,
                        NodeOutputOf::<S> {
                            source_index: crate::SourceIndex::new(item.source_index),
                            location: project_inline_report_item(
                                item.location,
                                Size::ZERO,
                                report.size,
                                cursor_block,
                                logical_content_box_inset.inline_start + run_offset,
                                constants,
                                containing_size,
                            ),
                            ..NodeOutputOf::new()
                        },
                    );
                }
            }
        }
    }

    Ok(InlineRunPlacement {
        size: report.size,
        content_size,
        static_positions,
        baselines: inline_report_baselines(
            &report,
            cursor_block,
            logical_content_box_inset.inline_start + run_offset,
            constants,
            containing_size,
        ),
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
    })
}

fn project_inline_report_item<S: LayoutScalar>(
    report_location: Point<S>,
    item_size: Size<S>,
    report_size: Size<S>,
    cursor_block: S,
    inline_start: S,
    constants: &Constants<S>,
    containing_size: Size<S>,
) -> Point<S> {
    let flow_axes = constants.flow_axes;
    let local = flow_axes.logical_point(report_location, item_size, report_size);
    flow_axes.physical_point(
        LogicalPointOf::new(inline_start + local.inline, cursor_block + local.block),
        flow_axes.logical_size(item_size),
        containing_size,
    )
}

fn inline_report_baselines<S: LayoutScalar>(
    report: &InlineRunReport<S>,
    cursor_block: S,
    inline_start: S,
    constants: &Constants<S>,
    containing_size: Size<S>,
) -> BaselinesOf<S> {
    let flow_axes = constants.flow_axes;
    if flow_axes.inline_axis() == PhysicalAxis::Horizontal {
        return BaselinesOf::NONE;
    }

    let mut block_start = None;
    let mut block_end = None;
    for item in &report.items {
        let origin = flow_axes.logical_point(item.location, item.size, report.size);
        let end = origin.block + flow_axes.logical_size(item.size).block;
        block_start =
            Some(block_start.map_or(origin.block, |current: S| current.min(origin.block)));
        block_end = Some(block_end.map_or(end, |current: S| current.max(end)));
    }
    let (Some(block_start), Some(block_end)) = (block_start, block_end) else {
        return BaselinesOf::NONE;
    };

    let block_coordinate = |side| {
        let logical_block = if side == flow_axes.block_start() {
            cursor_block + block_start
        } else {
            cursor_block + block_end
        };
        flow_axes.block_axis_coordinate(flow_axes.physical_point(
            LogicalPointOf::new(inline_start, logical_block),
            LogicalSizeOf::new(S::ZERO, S::ZERO),
            containing_size,
        ))
    };

    BaselinesOf::from_block_coordinates(
        flow_axes,
        Some(block_coordinate(flow_axes.line_under())),
        Some(block_coordinate(flow_axes.line_over())),
    )
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

fn inline_run_offset<S: LayoutScalar>(
    run_inline: S,
    constants: &Constants<S>,
    resolved_inner_inline: Option<S>,
) -> S {
    let logical_content_box_inset = constants.logical_content_box_inset();
    let container_inner_inline = constants
        .logical_node_inner_size()
        .inline
        .or(resolved_inner_inline)
        .or_else(|| {
            constants
                .logical_node_outer_size()
                .inline
                .map(|inline| inline - logical_content_box_inset.inline_sum())
        })
        .unwrap_or(run_inline);
    let free_space = (container_inner_inline - run_inline).max(S::ZERO);
    match constants.text_align {
        TextAlign::Auto => S::ZERO,
        TextAlign::LegacyLeft
            if constants
                .flow_axes
                .logical_axis_progression(crate::LogicalAxis::Inline)
                .is_decreasing() =>
        {
            free_space
        }
        TextAlign::LegacyRight
            if !constants
                .flow_axes
                .logical_axis_progression(crate::LogicalAxis::Inline)
                .is_decreasing() =>
        {
            free_space
        }
        TextAlign::LegacyCenter => free_space / S::from_f64(2.0),
        TextAlign::LegacyLeft | TextAlign::LegacyRight => S::ZERO,
    }
}

fn layout_floats<Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    floats: &[PendingFloat<<Tree as Traverse>::Node, S>],
    container_size: Size<S>,
    constants: &Constants<S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let mut float_exclusions = FloatExclusions::new(
        (container_size.width - constants.content_box_inset.horizontal_sum()).max(S::ZERO),
        constants.content_box_inset,
    );

    for float in floats {
        let location = float_exclusions.place_float(float, float.y);
        let scroll_geometry = retained_child_scroll_geometry(
            &float.style,
            float.size,
            float.content_size,
            float.padding,
            float.border,
            float.child_compute_geometry,
        )
        .map_err(|error| block_child_geometry_error(container, float.node, error))?;
        contributions
            .include_in_flow_geometry(location, float.margin, scroll_geometry)
            .map_err(|error| block_child_geometry_error(container, float.node, error))?;
        tree.set_unrounded(
            float.node,
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(float.source_index),
                location,
                size: float.size,
                content_size: float.content_size,
                border: float.border,
                padding: float.padding,
                margin: float.margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        );
    }

    Ok(())
}

struct FloatIntrinsics<S: LayoutScalar> {
    available_width: AvailableOf<S>,
    contribution: S,
}

impl<S: LayoutScalar> FloatIntrinsics<S> {
    const fn new(available_width: AvailableOf<S>) -> Self {
        Self {
            available_width,
            contribution: S::ZERO,
        }
    }

    fn add(&mut self, width: S, _float: Float, _clear: Clear) {
        match self.available_width {
            AvailableOf::<S>::Definite(_) => {}
            AvailableOf::<S>::MinContent => self.contribution = self.contribution.max(width),
            AvailableOf::<S>::MaxContent => self.contribution = self.contribution + width,
        }
    }

    const fn result(&self) -> S {
        self.contribution
    }
}

fn child_margin_can_collapse_with_parent<S: LayoutScalar>(style: &NodeInputOf<S>) -> bool {
    style.display == super::Display::Block && style.position == Position::Relative
}

#[expect(
    clippy::type_complexity,
    reason = "the private child-size helper preserves the session's generic error envelope"
)]
fn in_flow_child_known_size<Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
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
    let min_size = resolve_minimum_size(&style.min_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let mut max_size = resolve_maximum_size(&style.max_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .add_optional(box_sizing_adjustment);
    let aspect_height_limit = style
        .aspect_ratio
        .zip(max_size.height)
        .and_then(|(ratio, height)| max_size.width.is_none().then_some(height * ratio.get()));
    if let Some(width) = aspect_height_limit {
        max_size.width = Some(width);
    }
    let known = resolve_preferred_size(&style.size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .clamp_optional(min_size, max_size);

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
        known.inline =
            available_inline.map(|inline| inline.clamp_optional(min_size.inline, max_size.inline));
        if aspect_height_limit.is_some() {
            let physical_known = child_flow_axes.physical_size(known);
            known = child_flow_axes.logical_size(
                physical_known
                    .apply_aspect_ratio(style.aspect_ratio)
                    .clamp_optional(
                        child_flow_axes.physical_size(min_size),
                        child_flow_axes.physical_size(max_size),
                    ),
            );
        }
    }

    Ok(child_flow_axes.physical_size(known))
}

fn in_flow_child_available_inline<S: LayoutScalar>(
    style: &NodeInputOf<S>,
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

fn relative_inset_offset<S: LayoutScalar>(
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

pub(super) fn resolve_logical_in_flow_margin<S: LayoutScalar>(
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

fn resolve_atomic_inline_margin<S: LayoutScalar>(margin: Edges<Option<S>>) -> Edges<S> {
    margin.map(|value| value.unwrap_or(S::ZERO))
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

fn absolute_static_position<S: LayoutScalar>(
    cursor_block: S,
    constants: &Constants<S>,
    containing_size: Size<S>,
) -> Point<S> {
    constants.flow_axes.physical_point(
        LogicalPointOf::new(
            constants.logical_content_box_inset().inline_start,
            cursor_block,
        ),
        LogicalSizeOf::new(S::ZERO, S::ZERO),
        containing_size,
    )
}

fn content_size_contribution<S: LayoutScalar>(
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

fn block_final_in_flow_end<S: LayoutScalar>(
    content_box: super::ScrollRectOf<S>,
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

fn block_scroll_geometry<Tree, S, M>(
    node: <Tree as Traverse>::Node,
    run_mode: RunMode,
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    contributions: ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, super::ScrollGeometryOf<S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let target_border_box = super::ScrollRectOf::try_new(Point::ZERO, output_size)
        .map_err(|error| block_own_geometry_error(node, run_mode, error))?;
    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes: constants.flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: output_size,
        border: constants.border,
        padding: constants.padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: constants.settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: block_scroll_padding(style.scroll_padding),
        contributions,
        origin_axes: ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
        scroll_snap_type: style.scroll_snap_type,
        target_border_box,
        target_scroll_margin: style.scroll_margin,
        target_flow_axes: constants.flow_axes,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
    .map_err(|error| block_own_geometry_error(node, run_mode, error))
}

fn block_scroll_padding<S: LayoutScalar>(
    scroll_padding: crate::ScrollPaddingOf<S>,
) -> OptimalRegionInsetsOf<S> {
    fn inset<S: LayoutScalar>(value: crate::ScrollPaddingValueOf<S>) -> OptimalRegionInsetOf<S> {
        match value {
            crate::ScrollPaddingValueOf::Value(value) => OptimalRegionInsetOf::Value(value),
            crate::ScrollPaddingValueOf::Auto => OptimalRegionInsetOf::Auto,
        }
    }

    OptimalRegionInsetsOf::new(
        inset(scroll_padding.top()),
        inset(scroll_padding.right()),
        inset(scroll_padding.bottom()),
        inset(scroll_padding.left()),
    )
}

fn block_own_geometry_error<Node, S, M, E>(
    node: Node,
    run_mode: RunMode,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let _ = error;
    let (operation, invariant) = if run_mode == RunMode::PerformRootLayout {
        (
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        )
    } else {
        (
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        )
    };
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        operation,
        LayoutErrorKindOf::InternalInvariant(invariant),
    )
}

fn block_child_geometry_error<Node, S, M, E>(
    container: Node,
    subject: Node,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let _ = error;
    LayoutErrorOf::new(
        LayoutErrorSiteOf::ContainerSubject { container, subject },
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidBlockScrollGeometry),
    )
}

fn block_inline_geometry_error<Node, S, M, E>(
    container: Node,
    subject: Option<Node>,
    run_mode: RunMode,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    match subject {
        Some(subject) => block_child_geometry_error(container, subject, error),
        None => block_own_geometry_error(container, run_mode, error),
    }
}

fn retained_child_scroll_geometry<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
) -> Result<super::ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    if let Some(geometry) = child_compute_geometry {
        if geometry.border_box().origin() == Point::ZERO && geometry.border_box().size() == size {
            return Ok(geometry);
        }
        return rebuild_canonical_scroll_geometry_for_border_box(geometry, size, border, padding);
    }

    let flow_axes = crate::FlowAxes::new(style.writing_mode, style.direction);
    let settled_auto_scrollbars = crate::scroll::SettledAutoScrollbarState::INITIAL;
    let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
        flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: size,
        border,
        padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars,
    })?;
    let content_box = scroll_box.content_box();
    let direct_content = super::ScrollRectOf::try_new(
        content_box.origin(),
        Size::new(
            content_box.size().width.max(content_size.width),
            content_box.size().height.max(content_size.height),
        ),
    )
    .map_err(CanonicalScrollGeometryErrorOf::ScrollableOverflow)?;
    let mut contributions = ScrollContributionAccumulatorOf::new(scroll_box.padding_box());
    contributions.exclude_reserved_gutter_from_range();
    contributions.include_direct_line(direct_content);

    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: size,
        border,
        padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: block_scroll_padding(style.scroll_padding),
        contributions,
        origin_axes: ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
        scroll_snap_type: style.scroll_snap_type,
        target_border_box: scroll_box.border_box(),
        target_scroll_margin: style.scroll_margin,
        target_flow_axes: flow_axes,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
}

fn max_content_size<S: LayoutScalar>(a: Size<S>, b: Size<S>) -> Size<S> {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

fn layout_absolute_children<Tree, S, M>(
    tree: &mut Tree,
    container_node: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    static_positions: &[(<Tree as Traverse>::Node, Point<S>)],
    container: Size<S>,
    constants: &Constants<S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let area_start_x = constants.effective_border.left + constants.scrollbar_gutter.left;
    let max_area_start_x =
        (container.width - constants.effective_border.right).max(constants.effective_border.left);
    let area_start_y = constants.effective_border.top + constants.scrollbar_gutter.top;
    let max_area_start_y =
        (container.height - constants.effective_border.bottom).max(constants.effective_border.top);
    let area_offset = Point::new(
        area_start_x.min(max_area_start_x),
        area_start_y.min(max_area_start_y),
    );
    let area_size = Size::new(
        (container.width
            - constants.effective_border.horizontal_sum()
            - constants.scrollbar_gutter.horizontal_sum())
        .max(S::ZERO),
        (container.height
            - constants.effective_border.vertical_sum()
            - constants.scrollbar_gutter.vertical_sum())
        .max(S::ZERO),
    );
    let available = Size::new(
        AvailableOf::<S>::definite(area_size.width),
        AvailableOf::<S>::definite(area_size.height),
    );

    for (source_index, child) in children.iter().copied().enumerate() {
        let LayoutInputOf::Box(style) = tree.layout_input(child) else {
            continue;
        };
        if style.position != Position::Absolute || style.display == super::Display::None {
            continue;
        }

        let padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.padding,
                area_size.map(Some),
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.border,
                area_size.map(Some),
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let unresolved_margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.margin,
                area_size.map(Some),
                |length, basis| resolve_auto_optional(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(S::ZERO));
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::ZERO
        };
        let min_size = resolve_minimum_size(
            &style.min_size,
            area_size.map(Some),
            SizingAlgorithm::Positioned,
            true,
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .or(padding_border.sum_axes().map(Some))
        .max_optional(padding_border.sum_axes().map(Some));
        let max_size = resolve_maximum_size(
            &style.max_size,
            area_size.map(Some),
            SizingAlgorithm::Positioned,
            true,
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let style_size = resolve_preferred_size(
            &style.size,
            area_size.map(Some),
            SizingAlgorithm::Positioned,
            true,
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let aspect_max_size = if style.aspect_ratio.is_some()
            && style_size.width.is_none()
            && style_size.height.is_none()
            && max_size.width.is_some()
            && max_size.height.is_some()
        {
            max_size
        } else {
            Size::NONE
        };
        let mut known_size = style_size
            .or(aspect_max_size)
            .clamp_optional(min_size, max_size);
        let inset = style
            .inset
            .zip_size(area_size.map(Some), |length, basis| {
                resolve_auto_optional(length, basis)
            })
            .transpose_with_node(tree, child)?;
        if known_size.width.is_none()
            && let (Some(left), Some(right)) = (inset.left, inset.right)
        {
            known_size.width = Some(
                (area_size.width - non_auto_margin.horizontal_sum() - left - right)
                    .max(S::ZERO)
                    .clamp_optional(min_size.width, max_size.width),
            );
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_optional(min_size, max_size);
        }
        if known_size.height.is_none()
            && let (Some(top), Some(bottom)) = (inset.top, inset.bottom)
        {
            known_size.height = Some(
                (area_size.height - non_auto_margin.vertical_sum() - top - bottom)
                    .max(S::ZERO)
                    .clamp_optional(min_size.height, max_size.height),
            );
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_optional(min_size, max_size);
        }

        let output = tree.compute_child(
            child,
            ComputeInputOf::<S>::for_child(
                RunMode::PerformLayout,
                SizingMode::ContentSize,
                RequestedAxis::Both,
                known_size,
                area_size.map(Some),
                ContainingLayoutContext::new(
                    constants.flow_axes,
                    ParentFormattingContext::BlockFlow,
                ),
                available,
            )
            .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
        )?;
        let final_size = known_size
            .unwrap_or(output.size)
            .clamp_optional(min_size, max_size);
        let margin =
            resolve_absolute_margin(unresolved_margin, inset, style_size, final_size, area_size);
        let static_position = static_positions
            .iter()
            .find_map(|(node, position)| (*node == child).then_some(*position))
            .unwrap_or_else(|| {
                absolute_static_position(
                    constants.logical_content_box_inset().block_start,
                    constants,
                    container,
                )
            });
        let static_x_direction =
            static_axis_direction(constants.flow_axes, PhysicalAxis::Horizontal);
        let static_y_direction = static_axis_direction(constants.flow_axes, PhysicalAxis::Vertical);
        let location = Point::new(
            AbsoluteAxis {
                start: inset.left,
                end: inset.right,
                direction: if inset.left.is_none() && inset.right.is_none() {
                    static_x_direction
                } else {
                    constants.direction
                },
                area_start: area_offset.x,
                area_size: area_size.width,
                size: final_size.width,
                margin_start: margin.left,
                margin_end: margin.right,
                static_position: static_position.x,
            }
            .location(),
            AbsoluteAxis {
                start: inset.top,
                end: inset.bottom,
                direction: if inset.top.is_none() && inset.bottom.is_none() {
                    static_y_direction
                } else {
                    Direction::Ltr
                },
                area_start: area_offset.y,
                area_size: area_size.height,
                size: final_size.height,
                margin_start: margin.top,
                margin_end: margin.bottom,
                static_position: static_position.y,
            }
            .location(),
        );
        let scroll_geometry = retained_child_scroll_geometry(
            &style,
            final_size,
            output.content_size,
            padding,
            border,
            output.scroll_geometry,
        )
        .map_err(|error| block_child_geometry_error(container_node, child, error))?;
        contributions
            .include_current_out_of_flow_geometry(location, margin, scroll_geometry)
            .map_err(|error| block_child_geometry_error(container_node, child, error))?;
        tree.set_unrounded(
            child,
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(source_index),
                location,
                size: final_size,
                content_size: output.content_size,
                border,
                padding,
                margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        );
    }

    Ok(())
}

fn static_axis_direction(flow_axes: crate::geometry::FlowAxes, axis: PhysicalAxis) -> Direction {
    let start = if flow_axes.inline_axis() == axis {
        flow_axes.inline_start()
    } else {
        flow_axes.block_start()
    };
    if matches!(start, PhysicalSide::Right | PhysicalSide::Bottom) {
        Direction::Rtl
    } else {
        Direction::Ltr
    }
}

struct AbsoluteAxis<S: LayoutScalar> {
    start: Option<S>,
    end: Option<S>,
    direction: Direction,
    area_start: S,
    area_size: S,
    size: S,
    margin_start: S,
    margin_end: S,
    static_position: S,
}

impl<S: LayoutScalar> AbsoluteAxis<S> {
    fn location(self) -> S {
        if self.direction == Direction::Rtl
            && let (Some(_), Some(end)) = (self.start, self.end)
        {
            return self.area_start + self.area_size - end - self.size - self.margin_end;
        }

        if let Some(start) = self.start {
            self.area_start + start + self.margin_start
        } else if let Some(end) = self.end {
            self.area_start + self.area_size - end - self.size - self.margin_end
        } else if self.direction == Direction::Rtl {
            self.static_position - self.size - self.margin_end
        } else {
            self.static_position + self.margin_start
        }
    }
}

fn resolve_absolute_margin<S: LayoutScalar>(
    margin: Edges<Option<S>>,
    inset: Edges<Option<S>>,
    style_size: Size<Option<S>>,
    final_size: Size<S>,
    area_size: Size<S>,
) -> Edges<S> {
    let non_auto = Edges {
        left: if inset.left.is_some() {
            margin.left.unwrap_or(S::ZERO)
        } else {
            S::ZERO
        },
        right: if inset.right.is_some() {
            margin.right.unwrap_or(S::ZERO)
        } else {
            S::ZERO
        },
        top: if inset.top.is_some() {
            margin.top.unwrap_or(S::ZERO)
        } else {
            S::ZERO
        },
        bottom: if inset.bottom.is_some() {
            margin.bottom.unwrap_or(S::ZERO)
        } else {
            S::ZERO
        },
    };
    let auto_width = auto_margin_size(AutoMarginAxis {
        start_is_auto: margin.left.is_none(),
        end_is_auto: margin.right.is_none(),
        start: inset.left,
        end: inset.right,
        area_size: area_size.width,
        style_size: style_size.width,
        item_size: final_size.width,
        non_auto_margin_sum: non_auto.horizontal_sum(),
    });
    let auto_height = auto_margin_size(AutoMarginAxis {
        start_is_auto: margin.top.is_none(),
        end_is_auto: margin.bottom.is_none(),
        start: inset.top,
        end: inset.bottom,
        area_size: area_size.height,
        style_size: style_size.height,
        item_size: final_size.height,
        non_auto_margin_sum: non_auto.vertical_sum(),
    });

    Edges {
        left: margin.left.unwrap_or(auto_width),
        right: margin.right.unwrap_or(auto_width),
        top: margin.top.unwrap_or(auto_height),
        bottom: margin.bottom.unwrap_or(auto_height),
    }
}

#[derive(Clone, Copy)]
struct AutoMarginAxis<S: LayoutScalar> {
    start_is_auto: bool,
    end_is_auto: bool,
    start: Option<S>,
    end: Option<S>,
    area_size: S,
    style_size: Option<S>,
    item_size: S,
    non_auto_margin_sum: S,
}

fn auto_margin_size<S: LayoutScalar>(axis: AutoMarginAxis<S>) -> S {
    let auto_count = usize::from(axis.start_is_auto) + usize::from(axis.end_is_auto);
    if auto_count == 0 || axis.start.is_none() && axis.end.is_none() {
        return S::ZERO;
    }

    let available = axis
        .end
        .map(|end| axis.area_size - end - axis.start.unwrap_or(S::ZERO))
        .unwrap_or(axis.item_size);
    let free_space = available - axis.item_size - axis.non_auto_margin_sum;
    if auto_count == 2
        && axis
            .style_size
            .is_none_or(|style_size| style_size >= free_space)
    {
        S::ZERO
    } else {
        free_space / S::from_usize(auto_count)
    }
}

#[derive(Clone, Copy, Debug)]
enum ChildContainingBlockExtent<S: LayoutScalar> {
    Definite(S),
    FinalAutoDerived(S),
}

impl<S: LayoutScalar> ChildContainingBlockExtent<S> {
    fn value(self) -> S {
        match self {
            Self::Definite(value) | Self::FinalAutoDerived(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Constants<S: LayoutScalar> {
    flow_axes: crate::geometry::FlowAxes,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    child_containing_block_extent: Size<Option<ChildContainingBlockExtent<S>>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    direction: Direction,
    writing_mode: WritingMode,
    text_align: TextAlign,
    border: Edges<S>,
    padding: Edges<S>,
    effective_border: Edges<S>,
    padding_border_size: Size<S>,
    scrollbar_gutter: Edges<S>,
    content_box_inset: Edges<S>,
    available_content: Size<AvailableOf<S>>,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
    own_top_margin: CollapsibleMarginOf<S>,
    own_bottom_margin: CollapsibleMarginOf<S>,
    collapse_top_margin: bool,
    collapse_bottom_margin: bool,
    can_collapse_through: bool,
}

impl<S: LayoutScalar> Constants<S> {
    fn logical_node_outer_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_outer_size)
    }

    fn logical_node_inner_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_inner_size)
    }

    fn logical_node_min_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_min_size)
    }

    fn logical_node_max_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_max_size)
    }

    fn logical_padding_border_size(&self) -> LogicalSizeOf<S> {
        self.flow_axes.logical_size(self.padding_border_size)
    }

    fn logical_content_box_inset(&self) -> LogicalEdgesOf<S> {
        self.flow_axes.logical_edges(self.content_box_inset)
    }

    fn containing_size(&self, inner_size: LogicalSizeOf<Option<S>>) -> Size<S> {
        self.node_outer_size
            .unwrap_or(self.flow_axes.physical_size(LogicalSizeOf::new(
                inner_size.inline.unwrap_or(S::ZERO)
                    + self.logical_content_box_inset().inline_sum(),
                inner_size.block.unwrap_or(S::ZERO) + self.logical_content_box_inset().block_sum(),
            )))
    }

    fn child_containing_block_size(
        &self,
        child_flow_axes: crate::geometry::FlowAxes,
    ) -> Size<Option<S>> {
        let child_logical_extent = child_flow_axes.logical_size(self.child_containing_block_extent);
        let child_inline_extent = match child_logical_extent.inline {
            Some(ChildContainingBlockExtent::Definite(value)) => Some(value),
            Some(ChildContainingBlockExtent::FinalAutoDerived(value))
                if self.flow_axes.inline_axis() == child_flow_axes.inline_axis() =>
            {
                Some(value)
            }
            Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => None,
        };
        let child_block_extent = child_logical_extent
            .block
            .map(ChildContainingBlockExtent::value);
        child_flow_axes.physical_size(LogicalSizeOf::new(child_inline_extent, child_block_extent))
    }

    fn with_logical_node_inner_size(mut self, inner_size: LogicalSizeOf<Option<S>>) -> Self {
        let final_inner_size = self.flow_axes.physical_size(inner_size);
        self.child_containing_block_extent =
            final_inner_size.zip_map(self.child_containing_block_extent, |value, previous| {
                value.map(|value| match previous {
                    Some(ChildContainingBlockExtent::Definite(_)) => {
                        ChildContainingBlockExtent::Definite(value)
                    }
                    Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => {
                        ChildContainingBlockExtent::FinalAutoDerived(value)
                    }
                })
            });
        self.node_inner_size = final_inner_size;
        let content_box_inset = self.logical_content_box_inset();
        self.node_outer_size = self.flow_axes.physical_size(LogicalSizeOf::new(
            inner_size
                .inline
                .map(|inline| inline + content_box_inset.inline_sum()),
            inner_size
                .block
                .map(|block| block + content_box_inset.block_sum()),
        ));
        self
    }

    fn new<Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &NodeInputOf<S>,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        let flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
        let padding = input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(
                style.padding,
                input.parent(),
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, node)?;
        let border = input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.border, input.parent(), |length, basis| {
                resolve_length_or_zero(length, basis)
            })
            .transpose_with_node(tree, node)?;
        let own_logical_margin = flow_axes.logical_edges(style.margin);
        let collapsible_margin = flow_axes.physical_edges(LogicalEdgesOf::new(
            LengthAutoOf::ZERO,
            LengthAutoOf::ZERO,
            own_logical_margin.block_start,
            own_logical_margin.block_end,
        ));
        let margin = input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(
                collapsible_margin,
                input.parent(),
                |length, basis| resolve_auto_optional(length, basis),
            )
            .transpose_with_node(tree, node)?;
        let padding_border_size = (padding + border).sum_axes();
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border_size
        } else {
            Size::ZERO
        };
        let (style_size, min_size, max_size) = match input.sizing_mode() {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::InherentSize => {
                let style_size = resolve_preferred_size(
                    &style.size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let min_size = resolve_minimum_size(
                    &style.min_size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let max_size = resolve_maximum_size(
                    &style.max_size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                (style_size, min_size, max_size)
            }
        };
        let min_max_definite_size = min_size.zip_map(max_size, |min, max| match (min, max) {
            (Some(min), Some(max)) if max <= min => Some(min),
            _ => None,
        });
        let is_root = input.run_mode() == RunMode::PerformRootLayout;
        let boundary_margins_can_collapse =
            input.parent_formatting_context() == ParentFormattingContext::BlockFlow;
        let blocks_margin_collapse =
            !style.item_is_replaced && style.overflow.establishes_independent_formatting_context();
        let is_margin_collapsing_block = style.display == super::Display::Block;
        let can_collapse_through = is_margin_collapsing_block
            && boundary_margins_can_collapse
            && !is_root
            && !blocks_margin_collapse
            && style.position == Position::Relative
            && flow_axes.logical_edges(padding).block_start == S::ZERO
            && flow_axes.logical_edges(padding).block_end == S::ZERO
            && flow_axes.logical_edges(border).block_start == S::ZERO
            && flow_axes.logical_edges(border).block_end == S::ZERO
            && !matches!(flow_axes.logical_size(style_size).block, Some(block) if block > S::ZERO)
            && !matches!(flow_axes.logical_size(min_size).block, Some(block) if block > S::ZERO);
        let node_outer_size = input
            .known()
            .or(min_max_definite_size)
            .or(style_size.clamp_optional(min_size, max_size))
            .max_optional(padding_border_size.map(Some));
        let unconstrained_scroll_box_size = padding_border_size
            + Size::splat(style.scrollbar_width.get() + style.scrollbar_width.get());
        let scroll_box_size = node_outer_size
            .or(input.available().map(AvailableOf::into_option))
            .or(max_size)
            .unwrap_or(unconstrained_scroll_box_size)
            .zip_map(padding_border_size, |size, minimum| size.max(minimum));
        let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            border_box_size: scroll_box_size,
            border,
            padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
        })
        .map_err(|error| block_own_geometry_error(node, input.run_mode(), error))?;
        let effective_border = scroll_box.effective_border();
        let scrollbar_gutter = scroll_box.effective_gutter();
        let content_box_inset =
            effective_border + scrollbar_gutter + scroll_box.effective_padding();
        let content_box_inset_size = content_box_inset.sum_axes();
        let node_inner_size = node_outer_size.sub_optional(content_box_inset_size);
        let available_content =
            input
                .available()
                .zip_map(content_box_inset_size, |available, inset| match available {
                    AvailableOf::Definite(value) => {
                        AvailableOf::Definite((value - inset).max(S::ZERO))
                    }
                    AvailableOf::MinContent => AvailableOf::MinContent,
                    AvailableOf::MaxContent => AvailableOf::MaxContent,
                });
        let logical_padding = flow_axes.logical_edges(padding);
        let logical_border = flow_axes.logical_edges(border);
        let logical_style_size = flow_axes.logical_size(style_size);
        let logical_margin = flow_axes.logical_edges(margin);

        Ok(Self {
            flow_axes,
            node_outer_size,
            node_inner_size,
            child_containing_block_extent: node_inner_size
                .map(|value| value.map(ChildContainingBlockExtent::Definite)),
            node_min_size: min_size,
            node_max_size: max_size,
            direction: style.direction,
            writing_mode: style.writing_mode,
            text_align: style.text_align,
            border,
            padding,
            effective_border,
            padding_border_size,
            scrollbar_gutter,
            content_box_inset,
            available_content,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            own_top_margin: CollapsibleMarginOf::<S>::from_margin(
                logical_margin.block_start.unwrap_or(S::ZERO),
            ),
            own_bottom_margin: CollapsibleMarginOf::<S>::from_margin(
                logical_margin.block_end.unwrap_or(S::ZERO),
            ),
            collapse_top_margin: is_margin_collapsing_block
                && boundary_margins_can_collapse
                && !is_root
                && style.position == Position::Relative
                && !blocks_margin_collapse
                && logical_padding.block_start == S::ZERO
                && logical_border.block_start == S::ZERO,
            collapse_bottom_margin: is_margin_collapsing_block
                && boundary_margins_can_collapse
                && !is_root
                && style.position == Position::Relative
                && !blocks_margin_collapse
                && logical_padding.block_end == S::ZERO
                && logical_border.block_end == S::ZERO
                && logical_style_size.block.is_none(),
            can_collapse_through,
        })
    }
}

fn child_scrollbar_size<S: LayoutScalar>(style: &NodeInputOf<S>) -> Size<S> {
    scrollbar_size_from_overflow(
        style.overflow,
        style.item_is_replaced,
        style.scrollbar_width.get(),
    )
}

fn resolve_preferred_size<S: LayoutScalar>(
    size: &Size<super::PreferredSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_preferred_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_preferred_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

fn resolve_minimum_size<S: LayoutScalar>(
    size: &Size<super::MinSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_minimum_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_minimum_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

fn resolve_maximum_size<S: LayoutScalar>(
    size: &Size<super::MaxSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_maximum_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_maximum_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

fn resolve_auto_optional<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    resolution_optional(length.resolve_with_status(basis))
}

fn resolve_length_or_zero<S: LayoutScalar>(
    length: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolution_or_zero<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution
            .value
            .expect("resolved length resolution must carry a value")),
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => Ok(S::ZERO),
    }
}

fn resolution_optional<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution.value),
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => Ok(None),
    }
}

trait SizeOptionExt<S: LayoutScalar> {
    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size<S>) -> Size<S>;
    fn add_optional(self, amount: Size<S>) -> Self;
    fn sub_optional(self, amount: Size<S>) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<S>>) -> Self;
    fn clamp_optional(self, min: Self, max: Self) -> Self;
    fn max_optional(self, min: Self) -> Self;
}

impl<S: LayoutScalar> SizeOptionExt<S> for Size<Option<S>> {
    fn or(self, other: Self) -> Self {
        Size::new(self.width.or(other.width), self.height.or(other.height))
    }

    fn unwrap_or(self, fallback: Size<S>) -> Size<S> {
        Size::new(
            self.width.unwrap_or(fallback.width),
            self.height.unwrap_or(fallback.height),
        )
    }

    fn add_optional(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| width + amount.width),
            self.height.map(|height| height + amount.height),
        )
    }

    fn sub_optional(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| (width - amount.width).max(S::ZERO)),
            self.height
                .map(|height| (height - amount.height).max(S::ZERO)),
        )
    }

    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<S>>) -> Self {
        let Some(ratio) = aspect_ratio else {
            return self;
        };
        let ratio = ratio.get();
        match (self.width, self.height) {
            (Some(width), None) => Size::new(Some(width), Some(width / ratio)),
            (None, Some(height)) => Size::new(Some(height * ratio), Some(height)),
            _ => self,
        }
    }

    fn clamp_optional(self, min: Self, max: Self) -> Self {
        Size::new(
            self.width
                .map(|value| value.clamp_optional(min.width, max.width)),
            self.height
                .map(|value| value.clamp_optional(min.height, max.height)),
        )
    }

    fn max_optional(self, min: Self) -> Self {
        Size::new(
            self.width
                .zip(min.width)
                .map(|(value, min)| value.max(min))
                .or(self.width),
            self.height
                .zip(min.height)
                .map(|(value, min)| value.max(min))
                .or(self.height),
        )
    }
}

trait SizeConcreteExt<S: LayoutScalar> {
    fn clamp_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self;
}

impl<S: LayoutScalar> SizeConcreteExt<S> for Size<S> {
    fn clamp_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self {
        Size::new(
            self.width.clamp_optional(min.width, max.width),
            self.height.clamp_optional(min.height, max.height),
        )
    }
}

trait ScalarExt {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self
    where
        Self: Sized;
}

impl<S: LayoutScalar> ScalarExt for S {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = max.map_or(self, |max| self.min(max));
        min.map_or(value, |min| value.max(min))
    }
}
