use std::collections::BTreeMap;

use super::inline::{
    AtomicInlineBoxParticipant, ForcedLineBreakControlOf, InlineBoundaryControlOf,
    InlineControlAlignment, InlineFlowOf, InlineParticipant, InlineRunInput, layout_inline_run,
};
use super::value::{ResolvedLengthAutoOf, UnresolvedLengthReason};
use super::{
    AspectRatioOf, AvailableOf, BaselinesOf, BoxSizing, Clear, CollapsibleMarginOf, Compute,
    ComputeInputOf, ComputeOutputOf, DimensionOf, Direction, Edges, Float, InlineBoundaryInputOf,
    LayoutInputOf, LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionOf,
    LengthResolutionStatus, LineBreakInputOf, NodeInputOf, NodeOutputOf, Overflow, Point, Position,
    RequestedAxis, RunMode, Size, SizingMode, TextAlign, Traverse, VerticalAlign, WritingMode,
};
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, scroll_geometry_from_layout,
    scrollbar_size_from_overflow,
};

pub fn compute_block<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> ComputeOutputOf<Tree::Scalar>
where
    Tree: Compute,
{
    compute_block_inner::<Tree, Tree::Scalar>(tree, node, input)
}

fn compute_block_inner<Tree, S>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<S>,
) -> ComputeOutputOf<S>
where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    let style = tree.node_input(node).clone();
    let constants = Constants::new(&style, input);
    let children = tree.children(node).collect::<Vec<_>>();

    if children.is_empty()
        && input.run_mode == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return ComputeOutputOf::<S>::from_outer_size(Size::new(width, height));
    }
    if input.run_mode == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
        && !normal_flow_children_can_establish_baseline(tree, &children)
    {
        return ComputeOutputOf::<S>::from_outer_size(Size::new(width, height));
    }

    let intrinsic_pass = layout_in_flow_children(
        tree,
        &children,
        &constants,
        input,
        constants.node_inner_size.width,
        input.run_mode.is_perform_layout() && constants.node_inner_size.width.is_some(),
    );
    let auto_height = intrinsic_pass.auto_height(&constants);
    let intrinsic_outer_size = Size::new(
        intrinsic_pass.content_size.width + constants.content_box_inset.horizontal_sum(),
        auto_height,
    )
    .clamp_optional(constants.node_min_size, constants.node_max_size)
    .max_optional(constants.padding_border_size.map(Some));
    let outer_size = constants
        .node_outer_size
        .unwrap_or(intrinsic_outer_size)
        .max_optional(constants.padding_border_size.map(Some));
    let output_size = input
        .known
        .or(constants.node_outer_size)
        .unwrap_or(outer_size)
        .max_optional(constants.padding_border_size.map(Some));
    let final_pass =
        if input.run_mode.is_perform_layout() && constants.node_inner_size.width.is_none() {
            let inner_width =
                (output_size.width - constants.content_box_inset.horizontal_sum()).max(S::ZERO);
            layout_in_flow_children(tree, &children, &constants, input, Some(inner_width), true)
        } else {
            intrinsic_pass
        };
    let auto_height = final_pass.auto_height(&constants);
    let output_size = Size::new(
        output_size.width,
        input
            .known
            .height
            .or(constants.node_outer_size.height)
            .unwrap_or(auto_height)
            .clamp_optional(
                constants.node_min_size.height,
                constants.node_max_size.height,
            )
            .max(constants.padding_border_size.height),
    );
    let top_margin = final_pass.top_margin(&constants);
    let bottom_margin = final_pass.bottom_margin(&constants);
    let margins_can_collapse_through =
        constants.can_collapse_through && final_pass.all_in_flow_children_can_collapse_through;

    if input.run_mode == RunMode::ComputeSize {
        let mut output = ComputeOutputOf::<S>::from_sizes_and_baselines(
            output_size,
            Size::ZERO,
            final_pass.baselines,
        );
        output.top_margin = top_margin;
        output.bottom_margin = bottom_margin;
        output.margins_can_collapse_through = margins_can_collapse_through;
        output
    } else {
        layout_floats(tree, &final_pass.pending_floats, output_size, &constants);
        let absolute = layout_absolute_children(
            tree,
            &children,
            &final_pass.static_positions,
            output_size,
            &constants,
        );
        let content_size = max_content_size(final_pass.content_size, absolute.content_size);
        let scrollable_overflow = crate::scroll::scroll_rect_union(
            final_pass.scrollable_overflow,
            final_content_box_scroll_rect(&style, output_size, constants.padding, constants.border),
        )
        .expect("block scrollable overflow includes final content box");
        let scrollable_overflow = match absolute.scrollable_overflow {
            Some(absolute) => crate::scroll::scroll_rect_union(scrollable_overflow, absolute)
                .expect("block scrollable overflow union remains valid"),
            None => scrollable_overflow,
        };
        let mut output = ComputeOutputOf::<S>::from_sizes_and_baselines(
            output_size,
            content_size,
            final_pass.baselines,
        );
        output.scroll_geometry = Some(block_scroll_geometry(
            &style,
            &constants,
            output_size,
            scrollable_overflow,
        ));
        output.top_margin = top_margin;
        output.bottom_margin = bottom_margin;
        output.margins_can_collapse_through = margins_can_collapse_through;
        output
    }
}

fn normal_flow_children_can_establish_baseline<Tree>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
) -> bool
where
    Tree: Compute,
{
    children.iter().copied().any(|child| {
        let LayoutInputOf::Box(style) = tree.layout_input(child) else {
            return false;
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
    order: u32,
    side: Float,
    clear: Clear,
    y: S,
    size: Size<S>,
    content_size: Size<S>,
    scrollbar_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
    margin: Edges<S>,
    style: Box<NodeInputOf<S>>,
    scrollable_overflow: super::ScrollRectOf<S>,
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
    content_size: Size<S>,
    scrollable_overflow: super::ScrollRectOf<S>,
    baselines: BaselinesOf<S>,
    static_positions: Vec<(Node, Point<S>)>,
    pending_floats: Vec<PendingFloat<Node, S>>,
    cursor_y: S,
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

    fn auto_height(&self, constants: &Constants<S>) -> S {
        let bottom_margin_offset =
            if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
                S::ZERO
            } else {
                self.active_margin.resolve()
            };
        (self.cursor_y + bottom_margin_offset + constants.content_box_inset.bottom)
            .max(constants.content_box_inset.vertical_sum())
    }
}

fn inline_run_end<Tree>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<<Tree as Traverse>::Scalar>,
    mut index: usize,
) -> usize
where
    Tree: Compute,
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

fn visible_line_break_in_flow<Tree>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<LineBreakInputOf<<Tree as Traverse>::Scalar>>
where
    Tree: Compute,
{
    let LayoutInputOf::LineBreak(line_break) = tree.layout_input(child) else {
        return None;
    };
    if line_break.display().is_none() {
        return None;
    }
    if flow_writing_mode != WritingMode::HorizontalTb && line_break.clear() != Clear::None {
        panic!("vertical line-break clear layout is not implemented");
    }
    if line_break.writing_mode() != flow_writing_mode || line_break.direction() != flow_direction {
        panic!("line-break flow must match containing inline flow");
    }
    Some(line_break)
}

fn visible_inline_boundary_in_flow<Tree>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<InlineBoundaryInputOf<<Tree as Traverse>::Scalar>>
where
    Tree: Compute,
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

fn next_inline_clear_candidate<Tree>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    start: usize,
    run_end: usize,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<InlineClearCandidate>
where
    Tree: Compute,
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
            if flow_writing_mode != WritingMode::HorizontalTb {
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

fn inline_run_contains_clear<Tree>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    run_start: usize,
    run_end: usize,
    constants: &Constants<<Tree as Traverse>::Scalar>,
) -> bool
where
    Tree: Compute,
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

fn layout_in_flow_children<Tree, S>(
    tree: &mut Tree,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<S>,
    input: ComputeInputOf<S>,
    inner_width: Option<S>,
    set_layout: bool,
) -> InFlowResult<<Tree as Traverse>::Node, S>
where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    let node_inner_size = Size::new(inner_width, constants.node_inner_size.height);
    let mut cursor_y = constants.content_box_inset.top;
    let mut content_size: Size<S> = Size::ZERO;
    let mut first_baseline = None;
    let mut last_baseline = None;
    let mut static_positions = Vec::new();
    let mut active_margin = CollapsibleMarginOf::<S>::ZERO;
    let mut top_margin = CollapsibleMarginOf::<S>::ZERO;
    let mut is_collapsing_first_margin = constants.collapse_top_margin;
    let mut all_in_flow_children_can_collapse_through = true;
    let mut active_margin_can_collapse_with_parent = constants.collapse_top_margin;
    let mut pending_floats = Vec::new();
    let mut float_intrinsics = FloatIntrinsics::new(
        inner_width
            .map(AvailableOf::<S>::definite)
            .unwrap_or(input.available.width),
    );
    let content_width = inner_width
        .or(input.available.width.into_option())
        .unwrap_or(S::ZERO);
    let mut float_exclusions = FloatExclusions::new(content_width, constants.content_box_inset);
    let content_box_size = Size::new(
        inner_width
            .or(constants.node_inner_size.width)
            .or(input.available.width.into_option())
            .unwrap_or(S::ZERO),
        constants.node_inner_size.height.unwrap_or(S::ZERO),
    );
    let content_box_origin = Point::new(
        constants.content_box_inset.left,
        constants.content_box_inset.top,
    );
    let mut scrollable_overflow =
        ScrollableOverflowAccumulator::new(content_box_origin, content_box_size);

    let mut index = 0;
    while index < children.len() {
        let order = index;
        let child = children[index];
        let child_style = match tree.layout_input(child) {
            LayoutInputOf::Box(style) => *style,
            LayoutInputOf::LineBreak(line_break) => {
                if line_break.display().is_none() {
                    if set_layout {
                        tree.set_unrounded(child, NodeOutputOf::<S>::with_order(order as u32));
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
                cursor_y = cursor_y + collapsed_margin;
                if is_collapsing_first_margin {
                    is_collapsing_first_margin = false;
                }

                let placement = layout_inline_run_with_clear(
                    tree,
                    children,
                    run_start,
                    index,
                    InlineRunContext {
                        order_start: run_start as u32,
                        cursor_y,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    &float_exclusions,
                );
                content_size.width = content_size.width.max(placement.content_size.width);
                content_size.height = content_size.height.max(placement.content_size.height);
                scrollable_overflow.include_rect(placement.scrollable_overflow);
                static_positions.extend(placement.static_positions);
                if let Some(baseline) = placement.first_baseline {
                    let absolute_baseline = cursor_y + baseline;
                    first_baseline.get_or_insert(absolute_baseline);
                }
                if let Some(baseline) = placement.last_baseline {
                    last_baseline = Some(cursor_y + baseline);
                }
                cursor_y = cursor_y + placement.size.height;
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
                cursor_y = cursor_y + collapsed_margin;
                if is_collapsing_first_margin {
                    is_collapsing_first_margin = false;
                }

                let placement = layout_inline_run_with_clear(
                    tree,
                    children,
                    run_start,
                    index,
                    InlineRunContext {
                        order_start: run_start as u32,
                        cursor_y,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    &float_exclusions,
                );
                content_size.width = content_size.width.max(placement.content_size.width);
                content_size.height = content_size.height.max(placement.content_size.height);
                scrollable_overflow.include_rect(placement.scrollable_overflow);
                static_positions.extend(placement.static_positions);
                if let Some(baseline) = placement.first_baseline {
                    let absolute_baseline = cursor_y + baseline;
                    first_baseline.get_or_insert(absolute_baseline);
                }
                if let Some(baseline) = placement.last_baseline {
                    last_baseline = Some(cursor_y + baseline);
                }
                cursor_y = cursor_y + placement.size.height;
                active_margin = CollapsibleMarginOf::<S>::ZERO;
                active_margin_can_collapse_with_parent = false;
                all_in_flow_children_can_collapse_through = false;
                continue;
            }
        };
        if child_style.display == super::Display::None {
            if set_layout {
                tree.set_unrounded(child, NodeOutputOf::<S>::with_order(order as u32));
                tree.compute_child(child, ComputeInputOf::<S>::HIDDEN);
            }
            index += 1;
            continue;
        }
        if child_style.position == Position::Absolute {
            static_positions.push((
                child,
                absolute_static_position(cursor_y + active_margin.resolve(), constants),
            ));
            index += 1;
            continue;
        }

        if child_style.display.is_inline_level() && child_style.float.is_none() {
            let run_start = index;
            index = inline_run_end(tree, children, constants, index + 1);

            let collapsed_margin = active_margin.resolve();
            cursor_y = cursor_y + collapsed_margin;
            if is_collapsing_first_margin {
                is_collapsing_first_margin = false;
            }

            let placement = layout_inline_run_with_clear(
                tree,
                children,
                run_start,
                index,
                InlineRunContext {
                    order_start: run_start as u32,
                    cursor_y,
                    constants,
                    input,
                    node_inner_size,
                    set_layout,
                },
                &float_exclusions,
            );
            content_size.width = content_size.width.max(placement.content_size.width);
            content_size.height = content_size.height.max(placement.content_size.height);
            scrollable_overflow.include_rect(placement.scrollable_overflow);
            static_positions.extend(placement.static_positions);
            if let Some(baseline) = placement.first_baseline {
                let absolute_baseline = cursor_y + baseline;
                first_baseline.get_or_insert(absolute_baseline);
            }
            if let Some(baseline) = placement.last_baseline {
                last_baseline = Some(cursor_y + baseline);
            }
            cursor_y = cursor_y + placement.size.height;
            active_margin = CollapsibleMarginOf::<S>::ZERO;
            active_margin_can_collapse_with_parent = false;
            all_in_flow_children_can_collapse_through = false;
            continue;
        }

        let unresolved_margin = child_style
            .margin
            .zip_inline_size(node_inner_size, |length, basis| {
                length.resolve_auto_with_status(basis)
            });
        let child_padding = child_style
            .padding
            .zip_inline_size(node_inner_size, |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let child_border = child_style
            .border
            .zip_inline_size(node_inner_size, |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let child_non_auto_margin = unresolved_margin.map(resolved_length_auto_fallback_zero);
        let available_child_width = node_inner_size
            .width
            .or(input.available.width.into_option())
            .map(|width| (width - child_non_auto_margin.horizontal_sum()).max(S::ZERO));
        let child_known = in_flow_child_known_size(
            &child_style,
            child_padding + child_border,
            node_inner_size,
            available_child_width,
        );
        let output = tree.compute_child(
            child,
            ComputeInputOf::<S> {
                run_mode: input.run_mode.for_child(),
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: child_known,
                parent: Size::new(node_inner_size.width, None),
                available: Size::new(
                    in_flow_child_available_width(
                        &child_style,
                        available_child_width,
                        input.available.width,
                    ),
                    AvailableOf::<S>::MAX_CONTENT,
                ),
            },
        );

        let child_margin = resolve_in_flow_margin(
            unresolved_margin,
            output.size,
            node_inner_size
                .width
                .or(input.available.width.into_option()),
        );
        if !child_style.float.is_none() {
            let margin_box = output.size + child_margin.sum_axes();
            float_intrinsics.add(margin_box.width, child_style.float, child_style.clear);
            let float_parent_overflow = child_scrollable_overflow_for_parent(
                &child_style,
                output.size,
                output.content_size,
                child_padding,
                child_border,
                output.scroll_geometry,
            );
            let pending_float = PendingFloat {
                node: child,
                order: order as u32,
                side: child_style.float,
                clear: child_style.clear,
                y: cursor_y,
                size: output.size,
                content_size: output.content_size,
                scrollbar_size: child_scrollbar_size(&child_style),
                border: child_border,
                padding: child_padding,
                margin: child_margin,
                style: Box::new(child_style),
                scrollable_overflow: float_parent_overflow,
                child_compute_geometry: output.scroll_geometry,
            };
            let float_location = float_exclusions.place_float(&pending_float, cursor_y);
            scrollable_overflow.include_child(
                float_location,
                pending_float.size,
                pending_float.content_size,
                pending_float.margin,
                pending_float.style.overflow,
            );
            scrollable_overflow.include_translated_child_overflow(
                float_location,
                pending_float.scrollable_overflow,
            );
            if set_layout {
                pending_floats.push(pending_float);
            }
            content_size.width = content_size.width.max(float_intrinsics.result());
            content_size.height = content_size.height.max(
                float_location.y - constants.content_box_inset.top
                    + output.size.height
                    + child_margin.bottom,
            );
            index += 1;
            continue;
        }
        let inset_offset = relative_inset_offset(
            child_style.inset.zip_size(
                Size::new(node_inner_size.width, Some(S::ZERO)),
                |length, basis| resolve_auto_optional(length, basis),
            ),
            constants.direction,
        );
        let top_margin_set = output.top_margin.collapse_with_margin(child_margin.top);
        let bottom_margin_set = output
            .bottom_margin
            .collapse_with_margin(child_margin.bottom);
        let child_margin_can_collapse_with_parent =
            child_margin_can_collapse_with_parent(&child_style);
        let base_y = cursor_y;
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
        cursor_y = cursor_y + collapsed_margin;
        let layout_constants = if inner_width.is_some() {
            constants.with_inner_width(inner_width)
        } else {
            *constants
        };
        let fallback_location = Point::new(
            in_flow_child_x(output.size, child_margin, &layout_constants) + inset_offset.x,
            cursor_y + inset_offset.y,
        );
        let establishes_bfc = child_style.overflow.x.blocks_margin_collapse()
            || child_style.overflow.y.blocks_margin_collapse();
        let location = if establishes_bfc {
            let placement = float_exclusions.place_bfc_block(
                cursor_y,
                output.size,
                child_margin,
                child_style.clear,
                fallback_location.x - inset_offset.x,
            );
            Point::new(placement.x + inset_offset.x, placement.y + inset_offset.y)
        } else if child_style.clear != Clear::None {
            Point::new(
                fallback_location.x,
                float_exclusions.clearance_y(cursor_y, child_style.clear) + inset_offset.y,
            )
        } else {
            fallback_location
        };
        if set_layout {
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    order: order as u32,
                    location,
                    size: output.size,
                    content_size: output.content_size,
                    scroll_geometry: Some(child_node_scroll_geometry(
                        &child_style,
                        output.size,
                        output.content_size,
                        child_padding,
                        child_border,
                        output.scroll_geometry,
                    )),
                    scrollbar_size: child_scrollbar_size(&child_style),
                    border: child_border,
                    padding: child_padding,
                    margin: child_margin,
                },
            );
        }

        let child_bottom = (location.y - inset_offset.y) + output.size.height;
        let contribution = content_size_contribution(
            Point::new(
                location.x - constants.content_box_inset.left,
                location.y - constants.content_box_inset.top,
            ),
            output.size,
            output.content_size,
            child_style.overflow,
        );
        content_size.width = content_size
            .width
            .max(child_margin.left + output.size.width + child_margin.right)
            .max(contribution.width + child_margin.right);
        content_size.height = content_size
            .height
            .max(contribution.height)
            .max(child_bottom - constants.content_box_inset.top);
        scrollable_overflow.include_child(
            location,
            output.size,
            output.content_size,
            child_margin,
            child_style.overflow,
        );
        let child_overflow = child_scrollable_overflow_for_parent(
            &child_style,
            output.size,
            output.content_size,
            child_padding,
            child_border,
            output.scroll_geometry,
        );
        scrollable_overflow.include_translated_child_overflow(location, child_overflow);
        if let Some(baseline) = output.first_baselines.y {
            let absolute_baseline = location.y + baseline;
            first_baseline.get_or_insert(absolute_baseline);
        }
        if let Some(baseline) = output.last_baselines.y {
            last_baseline = Some(location.y + baseline);
        }
        if output.margins_can_collapse_through {
            cursor_y = if child_style.clear == Clear::None {
                base_y + output.size.height
            } else {
                child_bottom
            };
            active_margin = active_margin
                .collapse_with(top_margin_set)
                .collapse_with(bottom_margin_set);
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        } else {
            all_in_flow_children_can_collapse_through = false;
            cursor_y = child_bottom;
            active_margin = bottom_margin_set;
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        }
        index += 1;
    }

    InFlowResult {
        content_size,
        scrollable_overflow: scrollable_overflow.finish(),
        baselines: BaselinesOf::<S> {
            first: Point::new(None, first_baseline),
            last: Point::new(None, last_baseline),
        },
        static_positions,
        pending_floats,
        cursor_y,
        top_margin,
        active_margin,
        active_margin_can_collapse_with_parent,
        all_in_flow_children_can_collapse_through,
    }
}

struct InlineRunPlacement<Node, S: LayoutScalar> {
    size: Size<S>,
    content_size: Size<S>,
    scrollable_overflow: super::ScrollRectOf<S>,
    static_positions: Vec<(Node, Point<S>)>,
    first_baseline: Option<S>,
    last_baseline: Option<S>,
}

struct InlineRunContext<'a, S: LayoutScalar> {
    order_start: u32,
    cursor_y: S,
    constants: &'a Constants<S>,
    input: ComputeInputOf<S>,
    node_inner_size: Size<Option<S>>,
    set_layout: bool,
}

struct InlineSegmentsContext<'a, S: LayoutScalar> {
    order_start: u32,
    cursor_y: S,
    constants: &'a Constants<S>,
    input: ComputeInputOf<S>,
    node_inner_size: Size<Option<S>>,
    set_layout: bool,
}

fn forced_line_break_control<S: LayoutScalar>(
    order: u32,
    input: LineBreakInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> ForcedLineBreakControlOf<S> {
    ForcedLineBreakControlOf::new(
        order,
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
    order: u32,
    input: InlineBoundaryInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> InlineBoundaryControlOf<S> {
    InlineBoundaryControlOf::new(
        order,
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
        order: u32,
        style: Box<NodeInputOf<S>>,
        output: ComputeOutputOf<S>,
    },
    LineBreak {
        child: Node,
        order: u32,
    },
    Boundary {
        child: Node,
        order: u32,
    },
}

fn layout_inline_segments<Tree, S>(
    tree: &mut Tree,
    run: &[<Tree as Traverse>::Node],
    context: InlineSegmentsContext<'_, S>,
    float_exclusions: &FloatExclusions<S>,
) -> InlineRunPlacement<<Tree as Traverse>::Node, S>
where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    let InlineSegmentsContext {
        order_start,
        mut cursor_y,
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
    let start_y = cursor_y;
    let mut scrollable_overflow: Option<super::ScrollRectOf<S>> = None;

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
                &run[offset..candidate.end],
                InlineRunContext {
                    order_start: order_start + offset as u32,
                    cursor_y,
                    constants,
                    input,
                    node_inner_size,
                    set_layout: false,
                },
            );
            let segment_bottom = cursor_y + probe.size.height;
            if float_exclusions.clearance_y(segment_bottom, candidate.clear) > segment_bottom {
                segment_end = candidate.end;
                segment_clear = candidate.clear;
                break;
            }
            scan_start = candidate.end;
        }

        let placement = layout_inline_run_children(
            tree,
            &run[offset..segment_end],
            InlineRunContext {
                order_start: order_start + offset as u32,
                cursor_y,
                constants,
                input,
                node_inner_size,
                set_layout,
            },
        );

        content_size.width = content_size.width.max(placement.content_size.width);
        content_size.height = content_size.height.max(placement.content_size.height);
        scrollable_overflow = Some(match scrollable_overflow {
            Some(existing) => {
                crate::scroll::scroll_rect_union(existing, placement.scrollable_overflow)
                    .expect("segmented inline scroll overflow union remains valid")
            }
            None => placement.scrollable_overflow,
        });
        static_positions.extend(placement.static_positions);
        if let Some(baseline) = placement.first_baseline {
            first_baseline.get_or_insert(cursor_y - start_y + baseline);
        }
        if let Some(baseline) = placement.last_baseline {
            last_baseline = Some(cursor_y - start_y + baseline);
        }

        cursor_y = cursor_y + placement.size.height;
        if segment_clear != Clear::None {
            cursor_y = float_exclusions.clearance_y(cursor_y, segment_clear);
            content_size.height = content_size
                .height
                .max(cursor_y - constants.content_box_inset.top);
        }
        offset = segment_end;
    }

    InlineRunPlacement {
        size: Size::new(content_size.width, cursor_y - start_y),
        content_size,
        scrollable_overflow: scrollable_overflow.unwrap_or_else(|| {
            super::ScrollRectOf::new(
                Point::new(constants.content_box_inset.left, start_y),
                Size::ZERO,
            )
            .expect("empty segmented inline scroll overflow is valid")
        }),
        static_positions,
        first_baseline,
        last_baseline,
    }
}

fn layout_inline_run_with_clear<Tree, S>(
    tree: &mut Tree,
    children: &[<Tree as Traverse>::Node],
    run_start: usize,
    run_end: usize,
    context: InlineRunContext<'_, S>,
    float_exclusions: &FloatExclusions<S>,
) -> InlineRunPlacement<<Tree as Traverse>::Node, S>
where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    if !inline_run_contains_clear(tree, children, run_start, run_end, context.constants) {
        return layout_inline_run_children(tree, &children[run_start..run_end], context);
    }

    layout_inline_segments(
        tree,
        &children[run_start..run_end],
        InlineSegmentsContext {
            order_start: context.order_start,
            cursor_y: context.cursor_y,
            constants: context.constants,
            input: context.input,
            node_inner_size: context.node_inner_size,
            set_layout: context.set_layout,
        },
        float_exclusions,
    )
}

fn layout_inline_run_children<Tree, S>(
    tree: &mut Tree,
    run: &[<Tree as Traverse>::Node],
    context: InlineRunContext<'_, S>,
) -> InlineRunPlacement<<Tree as Traverse>::Node, S>
where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    let InlineRunContext {
        order_start,
        cursor_y,
        constants,
        input,
        node_inner_size,
        set_layout,
    } = context;
    let mut items = Vec::with_capacity(run.len());
    let mut run_children = Vec::with_capacity(run.len());
    let mut static_positions = Vec::new();
    for (offset, child) in run.iter().copied().enumerate() {
        let order = order_start + offset as u32;
        let child_style = match tree.layout_input(child) {
            LayoutInputOf::Box(style) => *style,
            LayoutInputOf::LineBreak(line_break) => {
                if line_break.display().is_none() {
                    if set_layout {
                        tree.set_unrounded(child, NodeOutputOf::<S>::with_order(order));
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

                run_children.push(InlineRunChild::LineBreak { child, order });
                items.push(InlineParticipant::forced_line_break(
                    forced_line_break_control(
                        order,
                        line_break,
                        node_inner_size
                            .width
                            .map(AvailableOf::<S>::definite)
                            .unwrap_or(input.available.width),
                    ),
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

                run_children.push(InlineRunChild::Boundary { child, order });
                items.push(InlineParticipant::inline_boundary(inline_boundary_control(
                    order,
                    boundary,
                    node_inner_size
                        .width
                        .map(AvailableOf::<S>::definite)
                        .unwrap_or(input.available.width),
                )));
                continue;
            }
        };
        if child_style.display == super::Display::None {
            if set_layout {
                tree.set_unrounded(child, NodeOutputOf::<S>::with_order(order));
                tree.compute_child(child, ComputeInputOf::<S>::HIDDEN);
            }
            continue;
        }
        if child_style.position == Position::Absolute {
            static_positions.push((child, absolute_static_position(cursor_y, constants)));
            continue;
        }
        let child_padding = child_style
            .padding
            .zip_inline_size(node_inner_size, |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let child_border = child_style
            .border
            .zip_inline_size(node_inner_size, |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let output = tree.compute_child(
            child,
            ComputeInputOf::<S> {
                run_mode: input.run_mode.for_child(),
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(node_inner_size.width, None),
                available: Size::new(
                    node_inner_size
                        .width
                        .map(AvailableOf::<S>::definite)
                        .unwrap_or(input.available.width),
                    AvailableOf::<S>::MAX_CONTENT,
                ),
            },
        );
        let unresolved_margin = child_style
            .margin
            .zip_inline_size(node_inner_size, |length, basis| {
                resolve_auto_optional(length, basis)
            });
        let child_margin = resolve_atomic_inline_margin(unresolved_margin);

        let item = InlineParticipant::Box(AtomicInlineBoxParticipant {
            order,
            size: output.size,
            content_size: output.content_size,
            margin: child_margin,
            padding: child_padding,
            border: child_border,
            scrollbar_size: child_scrollbar_size(&child_style),
            first_baseline: if child_style.vertical_align == VerticalAlign::Top {
                Some(S::ZERO)
            } else {
                output.last_baselines.y.or(output.first_baselines.y)
            },
        });
        run_children.push(InlineRunChild::Box {
            child,
            order,
            style: Box::new(child_style),
            output,
        });
        items.push(item);
    }

    let report = layout_inline_run(InlineRunInput {
        available_width: node_inner_size
            .width
            .map(AvailableOf::<S>::definite)
            .unwrap_or(input.available.width),
        writing_mode: constants.writing_mode,
        direction: constants.direction,
        items,
    });
    let run_offset = inline_run_offset(report.size.width, constants, node_inner_size.width);
    let mut content_size = content_size_contribution(
        Point::new(run_offset, cursor_y - constants.content_box_inset.top),
        report.size,
        report.content_size,
        Point::new(Overflow::Visible, Overflow::Visible),
    );

    let report_items_by_order = report
        .items
        .iter()
        .copied()
        .map(|item| (item.order, item))
        .collect::<BTreeMap<_, _>>();
    let mut run_scrollable_overflow = ScrollableOverflowAccumulator::new(Point::ZERO, report.size);

    for run_child in &run_children {
        match run_child {
            InlineRunChild::Box {
                child,
                order,
                style: child_style,
                output,
            } => {
                let item = report_items_by_order[order];
                let inset_offset = relative_inset_offset(
                    child_style.inset.zip_size(
                        Size::new(node_inner_size.width, Some(S::ZERO)),
                        |length, basis| resolve_auto_optional(length, basis),
                    ),
                    constants.direction,
                );
                let run_child_location = Point::new(
                    item.location.x + inset_offset.x,
                    item.location.y + inset_offset.y,
                );
                let child_overflow = child_scrollable_overflow_for_parent(
                    child_style,
                    item.size,
                    item.content_size,
                    item.padding,
                    item.border,
                    output.scroll_geometry,
                );
                run_scrollable_overflow
                    .include_translated_child_overflow(run_child_location, child_overflow);
                let location = Point::new(
                    run_offset + item.location.x + inset_offset.x,
                    cursor_y + item.location.y + inset_offset.y - constants.content_box_inset.top,
                );
                let contribution = content_size_contribution(
                    location,
                    item.size,
                    output.content_size,
                    child_style.overflow,
                );
                content_size = max_content_size(content_size, contribution);

                if set_layout {
                    tree.set_unrounded(
                        *child,
                        NodeOutputOf::<S> {
                            order: item.order,
                            location: Point::new(
                                constants.content_box_inset.left
                                    + run_offset
                                    + item.location.x
                                    + inset_offset.x,
                                cursor_y + item.location.y + inset_offset.y,
                            ),
                            size: item.size,
                            content_size: item.content_size,
                            scroll_geometry: Some(child_node_scroll_geometry(
                                child_style,
                                item.size,
                                item.content_size,
                                item.padding,
                                item.border,
                                output.scroll_geometry,
                            )),
                            scrollbar_size: item.scrollbar_size,
                            border: item.border,
                            padding: item.padding,
                            margin: item.margin,
                        },
                    );
                }
            }
            InlineRunChild::LineBreak { child, order } => {
                if set_layout {
                    let item = report_items_by_order[order];
                    tree.set_unrounded(
                        *child,
                        NodeOutputOf::<S> {
                            order: item.order,
                            location: Point::new(
                                constants.content_box_inset.left + run_offset + item.location.x,
                                cursor_y + item.location.y,
                            ),
                            size: Size::ZERO,
                            content_size: Size::ZERO,
                            scroll_geometry: None,
                            scrollbar_size: Size::ZERO,
                            border: Edges::ZERO,
                            padding: Edges::ZERO,
                            margin: Edges::ZERO,
                        },
                    );
                }
            }
            InlineRunChild::Boundary { child, order } => {
                if set_layout {
                    let item = report_items_by_order[order];
                    tree.set_unrounded(
                        *child,
                        NodeOutputOf::<S> {
                            order: item.order,
                            location: Point::new(
                                constants.content_box_inset.left + run_offset + item.location.x,
                                cursor_y + item.location.y,
                            ),
                            size: Size::ZERO,
                            content_size: Size::ZERO,
                            scroll_geometry: None,
                            scrollbar_size: Size::ZERO,
                            border: Edges::ZERO,
                            padding: Edges::ZERO,
                            margin: Edges::ZERO,
                        },
                    );
                }
            }
        }
    }

    InlineRunPlacement {
        size: report.size,
        content_size,
        scrollable_overflow: translate_scroll_rect(
            run_scrollable_overflow.finish(),
            Point::new(constants.content_box_inset.left + run_offset, cursor_y),
        ),
        static_positions,
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
    }
}

fn inline_run_offset<S: LayoutScalar>(
    run_width: S,
    constants: &Constants<S>,
    resolved_inner_width: Option<S>,
) -> S {
    let container_inner_width = constants
        .node_inner_size
        .width
        .or(resolved_inner_width)
        .or_else(|| {
            constants
                .node_outer_size
                .width
                .map(|width| width - constants.content_box_inset.horizontal_sum())
        })
        .unwrap_or(run_width);
    let free_space = (container_inner_width - run_width).max(S::ZERO);
    match (constants.text_align, constants.direction) {
        (TextAlign::Auto, Direction::Ltr)
        | (TextAlign::LegacyLeft, Direction::Ltr)
        | (TextAlign::LegacyLeft, Direction::Rtl) => S::ZERO,
        (TextAlign::Auto, Direction::Rtl)
        | (TextAlign::LegacyRight, Direction::Ltr)
        | (TextAlign::LegacyRight, Direction::Rtl) => free_space,
        (TextAlign::LegacyCenter, _) => free_space / S::from_f64(2.0),
    }
}

fn layout_floats<Tree, S>(
    tree: &mut Tree,
    floats: &[PendingFloat<<Tree as Traverse>::Node, S>],
    container_size: Size<S>,
    constants: &Constants<S>,
) where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    let mut float_exclusions = FloatExclusions::new(
        (container_size.width - constants.content_box_inset.horizontal_sum()).max(S::ZERO),
        constants.content_box_inset,
    );

    for float in floats {
        let location = float_exclusions.place_float(float, float.y);
        tree.set_unrounded(
            float.node,
            NodeOutputOf::<S> {
                order: float.order,
                location,
                size: float.size,
                content_size: float.content_size,
                scroll_geometry: Some(child_node_scroll_geometry(
                    &float.style,
                    float.size,
                    float.content_size,
                    float.padding,
                    float.border,
                    float.child_compute_geometry,
                )),
                scrollbar_size: float.scrollbar_size,
                border: float.border,
                padding: float.padding,
                margin: float.margin,
            },
        );
    }
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

fn in_flow_child_known_size<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    padding_border: Edges<S>,
    parent: Size<Option<S>>,
    available_width: Option<S>,
) -> Size<Option<S>> {
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let min_size = style
        .min_size
        .zip_map(parent, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let mut max_size = style
        .max_size
        .zip_map(parent, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .add_optional(box_sizing_adjustment);
    let aspect_height_limit = style
        .aspect_ratio
        .zip(max_size.height)
        .and_then(|(ratio, height)| max_size.width.is_none().then_some(height * ratio.get()));
    if let Some(width) = aspect_height_limit {
        max_size.width = Some(width);
    }
    let mut known = style
        .size
        .zip_map(parent, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .clamp_optional(min_size, max_size);

    if !style.item_is_table
        && known.width.is_none()
        && !style.size.width.is_min_content()
        && !style.size.width.is_max_content()
    {
        known.width =
            available_width.map(|width| width.clamp_optional(min_size.width, max_size.width));
        if aspect_height_limit.is_some() {
            known = known
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_optional(min_size, max_size);
        }
    }

    known
}

fn in_flow_child_available_width<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    available_width: Option<S>,
    fallback: AvailableOf<S>,
) -> AvailableOf<S> {
    if style.size.width.is_min_content() {
        AvailableOf::<S>::MIN_CONTENT
    } else if style.size.width.is_max_content() {
        AvailableOf::<S>::MAX_CONTENT
    } else {
        available_width
            .map(AvailableOf::<S>::definite)
            .unwrap_or(fallback)
    }
}

fn relative_inset_offset<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    direction: Direction,
) -> Point<S> {
    Point::new(
        if direction == Direction::Rtl {
            inset
                .right
                .map(|right| -right)
                .or(inset.left)
                .unwrap_or(S::ZERO)
        } else {
            inset
                .left
                .or_else(|| inset.right.map(|right| -right))
                .unwrap_or(S::ZERO)
        },
        inset
            .top
            .or_else(|| inset.bottom.map(|bottom| -bottom))
            .unwrap_or(S::ZERO),
    )
}

pub(super) fn resolve_in_flow_margin<S: LayoutScalar>(
    margin: Edges<ResolvedLengthAutoOf<S>>,
    child_size: Size<S>,
    container_width: Option<S>,
) -> Edges<S> {
    let non_auto_horizontal = resolved_length_auto_fallback_zero(margin.left)
        + resolved_length_auto_fallback_zero(margin.right);
    let auto_count = usize::from(matches!(margin.left, ResolvedLengthAutoOf::Auto))
        + usize::from(matches!(margin.right, ResolvedLengthAutoOf::Auto));
    let auto_horizontal = if auto_count == 0 {
        S::ZERO
    } else {
        container_width
            .map(|width| (width - child_size.width - non_auto_horizontal).max(S::ZERO))
            .unwrap_or(S::ZERO)
            / S::from_usize(auto_count)
    };

    Edges {
        left: resolved_length_auto_or(margin.left, auto_horizontal),
        right: resolved_length_auto_or(margin.right, auto_horizontal),
        top: resolved_length_auto_fallback_zero(margin.top),
        bottom: resolved_length_auto_fallback_zero(margin.bottom),
    }
}

fn resolved_length_auto_or<S: LayoutScalar>(value: ResolvedLengthAutoOf<S>, auto_fallback: S) -> S {
    match value {
        ResolvedLengthAutoOf::Auto => auto_fallback,
        ResolvedLengthAutoOf::Resolved(value) => value,
        // Missing-basis symbolic margins keep the algorithm's historical
        // unresolved-as-zero fallback and do not participate in auto distribution.
        ResolvedLengthAutoOf::Unresolved(UnresolvedLengthReason::Basis) => S::ZERO,
        ResolvedLengthAutoOf::Unresolved(UnresolvedLengthReason::InvalidNumeric) => {
            panic!("invalid numeric length resolution")
        }
    }
}

fn resolved_length_auto_fallback_zero<S: LayoutScalar>(value: ResolvedLengthAutoOf<S>) -> S {
    resolved_length_auto_or(value, S::ZERO)
}

fn resolve_atomic_inline_margin<S: LayoutScalar>(margin: Edges<Option<S>>) -> Edges<S> {
    margin.map(|value| value.unwrap_or(S::ZERO))
}

fn in_flow_child_x<S: LayoutScalar>(
    size: Size<S>,
    margin: Edges<S>,
    constants: &Constants<S>,
) -> S {
    let mut x = if constants.direction == Direction::Rtl {
        let container = constants.node_outer_size.unwrap_or(
            constants
                .node_inner_size
                .unwrap_or(size + margin.sum_axes()),
        );
        container.width - constants.content_box_inset.right - size.width - margin.right
    } else {
        constants.content_box_inset.left + margin.left
    };

    let container_inner_width = constants
        .node_inner_size
        .width
        .or_else(|| {
            constants
                .node_outer_size
                .width
                .map(|width| width - constants.content_box_inset.horizontal_sum())
        })
        .unwrap_or(size.width + margin.horizontal_sum());
    let item_outer_width = size.width + margin.horizontal_sum();
    if item_outer_width < container_inner_width {
        let free_space = container_inner_width - item_outer_width;
        match (constants.text_align, constants.direction) {
            (TextAlign::Auto, _)
            | (TextAlign::LegacyLeft, Direction::Ltr)
            | (TextAlign::LegacyRight, Direction::Rtl) => {}
            (TextAlign::LegacyLeft, Direction::Rtl) | (TextAlign::LegacyCenter, Direction::Rtl) => {
                x = x - if constants.text_align == TextAlign::LegacyCenter {
                    free_space / S::from_f64(2.0)
                } else {
                    free_space
                };
            }
            (TextAlign::LegacyRight, Direction::Ltr)
            | (TextAlign::LegacyCenter, Direction::Ltr) => {
                x = x + if constants.text_align == TextAlign::LegacyCenter {
                    free_space / S::from_f64(2.0)
                } else {
                    free_space
                };
            }
        }
    }

    x
}

fn absolute_static_position<S: LayoutScalar>(cursor_y: S, constants: &Constants<S>) -> Point<S> {
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::ZERO));
    let x = if constants.direction == Direction::Rtl {
        container.width - constants.content_box_inset.right
    } else {
        constants.content_box_inset.left
    };
    Point::new(x, cursor_y)
}

fn content_size_contribution<S: LayoutScalar>(
    location: Point<S>,
    size: Size<S>,
    content_size: Size<S>,
    overflow: Point<Overflow>,
) -> Size<S> {
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
    if contribution_size.width <= S::ZERO || contribution_size.height <= S::ZERO {
        return Size::ZERO;
    }

    let max_x = (location.x + contribution_size.width).max(S::ZERO);
    let min_x = location.x.min(S::ZERO);
    let max_y = (location.y + contribution_size.height).max(S::ZERO);
    let min_y = location.y.min(S::ZERO);
    Size::new(max_x - min_x, max_y - min_y)
}

fn block_scroll_geometry<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    scrollable_overflow: super::ScrollRectOf<S>,
) -> super::ScrollGeometryOf<S> {
    scroll_geometry_from_layout(
        style.writing_mode,
        style.direction,
        style.overflow,
        output_size,
        constants.padding,
        constants.border,
        style.scrollbar_width,
        scrollable_overflow,
    )
    .expect("block scroll geometry is derived from finite non-negative layout output")
}

fn child_node_scroll_geometry<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
) -> super::ScrollGeometryOf<S> {
    let scrollable_overflow = child_scrollable_overflow(
        style,
        size,
        content_size,
        padding,
        border,
        child_compute_geometry,
    );
    scroll_geometry_from_layout(
        style.writing_mode,
        style.direction,
        style.overflow,
        size,
        padding,
        border,
        style.scrollbar_width,
        scrollable_overflow,
    )
    .expect("child scroll geometry is derived from finite non-negative layout output")
}

fn child_scrollable_overflow<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
) -> super::ScrollRectOf<S> {
    let base = crate::scroll::scrollable_overflow_from_layout_content_size(
        style.direction,
        style.overflow,
        size,
        padding,
        border,
        style.scrollbar_width,
        content_size,
    )
    .expect("child scrollable overflow is derived from finite non-negative layout output");
    let Some(child_compute_geometry) = child_compute_geometry else {
        return base;
    };

    crate::scroll::scroll_rect_union(base, child_compute_geometry.scrollable_overflow())
        .expect("child scrollable overflow union remains valid")
}

fn child_scrollable_overflow_for_parent<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
) -> super::ScrollRectOf<S> {
    let child_overflow = child_scrollable_overflow(
        style,
        size,
        content_size,
        padding,
        border,
        child_compute_geometry,
    );
    project_child_scrollable_overflow_for_parent(style.overflow, size, child_overflow)
}

fn project_child_scrollable_overflow_for_parent<S: LayoutScalar>(
    overflow: Point<Overflow>,
    size: Size<S>,
    child_overflow: super::ScrollRectOf<S>,
) -> super::ScrollRectOf<S> {
    let child_origin = child_overflow.origin();
    let child_size = child_overflow.size();
    let child_end = Point::new(
        child_origin.x + child_size.width,
        child_origin.y + child_size.height,
    );
    let projected_origin = Point::new(
        if overflow.x == Overflow::Visible {
            child_origin.x
        } else {
            S::ZERO
        },
        if overflow.y == Overflow::Visible {
            child_origin.y
        } else {
            S::ZERO
        },
    );
    let projected_end = Point::new(
        if overflow.x == Overflow::Visible {
            child_end.x
        } else {
            size.width
        },
        if overflow.y == Overflow::Visible {
            child_end.y
        } else {
            size.height
        },
    );

    super::ScrollRectOf::new(
        projected_origin,
        Size::new(
            (projected_end.x - projected_origin.x).max(S::ZERO),
            (projected_end.y - projected_origin.y).max(S::ZERO),
        ),
    )
    .expect("projected child scrollable overflow remains valid")
}

fn translate_scroll_rect<S: LayoutScalar>(
    rect: super::ScrollRectOf<S>,
    offset: Point<S>,
) -> super::ScrollRectOf<S> {
    super::ScrollRectOf::new(
        Point::new(rect.origin().x + offset.x, rect.origin().y + offset.y),
        rect.size(),
    )
    .expect("translated scroll rect remains valid")
}

fn final_content_box_scroll_rect<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
) -> super::ScrollRectOf<S> {
    let reservation = ScrollbarReservationOf::from_overflow(
        style.overflow,
        style.scrollbar_width,
        style.direction,
    );
    let inset = content_box_inset_with_scrollbar(padding, border, reservation);
    let content_size = Size::new(
        (size.width - inset.horizontal_sum()).max(S::ZERO),
        (size.height - inset.vertical_sum()).max(S::ZERO),
    );
    super::ScrollRectOf::new(Point::new(inset.left, inset.top), content_size)
        .expect("final content box scroll rect is valid")
}

#[derive(Clone, Copy, Debug)]
struct ScrollableOverflowAccumulator<S: LayoutScalar> {
    rect: super::ScrollRectOf<S>,
}

impl<S: LayoutScalar> ScrollableOverflowAccumulator<S> {
    fn new(content_box_origin: Point<S>, content_box_size: Size<S>) -> Self {
        Self {
            rect: super::ScrollRectOf::new(content_box_origin, content_box_size)
                .expect("content box size is non-negative"),
        }
    }

    fn include_rect(&mut self, rect: super::ScrollRectOf<S>) {
        self.rect = crate::scroll::scroll_rect_union(self.rect, rect)
            .expect("scrollable overflow union remains valid");
    }

    fn include_translated_child_overflow(
        &mut self,
        location: Point<S>,
        overflow: super::ScrollRectOf<S>,
    ) {
        self.include_rect(translate_scroll_rect(overflow, location));
    }

    fn include_child(
        &mut self,
        location: Point<S>,
        size: Size<S>,
        content_size: Size<S>,
        margin: Edges<S>,
        overflow: Point<Overflow>,
    ) {
        let margin_rect = super::ScrollRectOf::new(
            Point::new(location.x - margin.left, location.y - margin.top),
            size + margin.sum_axes(),
        )
        .expect("child margin rect is non-negative");
        self.include_rect(margin_rect);

        let visible_size = Size::new(
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
        self.include_rect(
            super::ScrollRectOf::new(location, visible_size)
                .expect("visible overflow rect is non-negative"),
        );
    }

    fn finish(self) -> super::ScrollRectOf<S> {
        self.rect
    }
}

fn max_content_size<S: LayoutScalar>(a: Size<S>, b: Size<S>) -> Size<S> {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

struct AbsoluteLayoutResult<S: LayoutScalar> {
    content_size: Size<S>,
    scrollable_overflow: Option<super::ScrollRectOf<S>>,
}

fn layout_absolute_children<Tree, S>(
    tree: &mut Tree,
    children: &[<Tree as Traverse>::Node],
    static_positions: &[(<Tree as Traverse>::Node, Point<S>)],
    container: Size<S>,
    constants: &Constants<S>,
) -> AbsoluteLayoutResult<S>
where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    let area_start_x = constants.border.left + constants.scrollbar_gutter.left;
    let max_area_start_x = (container.width - constants.border.right).max(constants.border.left);
    let area_offset = Point::new(area_start_x.min(max_area_start_x), constants.border.top);
    let area_size = Size::new(
        (container.width
            - constants.border.horizontal_sum()
            - constants.scrollbar_gutter.horizontal_sum())
        .max(S::ZERO),
        (container.height
            - constants.border.vertical_sum()
            - constants.scrollbar_gutter.vertical_sum())
        .max(S::ZERO),
    );
    let available = Size::new(
        AvailableOf::<S>::definite(area_size.width),
        AvailableOf::<S>::definite(area_size.height),
    );

    let mut absolute_content_size = Size::ZERO;
    let mut absolute_scrollable_overflow: Option<super::ScrollRectOf<S>> = None;
    for (order, child) in children.iter().copied().enumerate() {
        let LayoutInputOf::Box(style) = tree.layout_input(child) else {
            continue;
        };
        if style.position != Position::Absolute || style.display == super::Display::None {
            continue;
        }

        let padding = style
            .padding
            .zip_inline_size(area_size.map(Some), |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let border = style
            .border
            .zip_inline_size(area_size.map(Some), |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let unresolved_margin = style
            .margin
            .zip_inline_size(area_size.map(Some), |length, basis| {
                resolve_auto_optional(length, basis)
            });
        let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(S::ZERO));
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::ZERO
        };
        let min_size = style
            .min_size
            .zip_map(area_size.map(Some), |dimension, basis| {
                resolve_dimension(dimension, basis)
            })
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment)
            .or(padding_border.sum_axes().map(Some))
            .max_optional(padding_border.sum_axes().map(Some));
        let max_size = style
            .max_size
            .zip_map(area_size.map(Some), |dimension, basis| {
                resolve_dimension(dimension, basis)
            })
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);
        let style_size = style
            .size
            .zip_map(area_size.map(Some), |dimension, basis| {
                resolve_dimension(dimension, basis)
            })
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
        let inset = style.inset.zip_size(area_size.map(Some), |length, basis| {
            resolve_auto_optional(length, basis)
        });
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
            ComputeInputOf::<S> {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::ContentSize,
                axis: RequestedAxis::Both,
                known: known_size,
                parent: area_size.map(Some),
                available,
            },
        );
        let final_size = known_size
            .unwrap_or(output.size)
            .clamp_optional(min_size, max_size);
        let margin =
            resolve_absolute_margin(unresolved_margin, inset, style_size, final_size, area_size);
        let mut static_position = static_positions
            .iter()
            .find_map(|(node, position)| (*node == child).then_some(*position))
            .unwrap_or_else(|| {
                absolute_static_position(constants.content_box_inset.top, constants)
            });
        if constants.direction == Direction::Rtl && inset.left.is_none() && inset.right.is_none() {
            static_position.x = container.width - constants.content_box_inset.right;
        }
        let location = Point::new(
            AbsoluteAxis {
                start: inset.left,
                end: inset.right,
                direction: constants.direction,
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
                direction: Direction::Ltr,
                area_start: area_offset.y,
                area_size: area_size.height,
                size: final_size.height,
                margin_start: margin.top,
                margin_end: margin.bottom,
                static_position: static_position.y,
            }
            .location(),
        );
        absolute_content_size = max_content_size(
            absolute_content_size,
            content_size_contribution(
                Point::new(location.x - area_offset.x, location.y - area_offset.y),
                final_size,
                output.content_size,
                style.overflow,
            ),
        );
        let margin_box_origin = Point::new(location.x - margin.left, location.y - margin.top);
        let mut absolute_accumulator =
            ScrollableOverflowAccumulator::new(margin_box_origin, final_size + margin.sum_axes());
        absolute_accumulator.include_child(
            location,
            final_size,
            output.content_size,
            margin,
            style.overflow,
        );
        let child_overflow = child_scrollable_overflow_for_parent(
            &style,
            final_size,
            output.content_size,
            padding,
            border,
            output.scroll_geometry,
        );
        absolute_accumulator.include_translated_child_overflow(location, child_overflow);
        let child_rect = absolute_accumulator.finish();
        absolute_scrollable_overflow = Some(match absolute_scrollable_overflow {
            Some(existing) => crate::scroll::scroll_rect_union(existing, child_rect)
                .expect("absolute scroll overflow union remains valid"),
            None => child_rect,
        });

        tree.set_unrounded(
            child,
            NodeOutputOf::<S> {
                order: order as u32,
                location,
                size: final_size,
                content_size: output.content_size,
                scroll_geometry: Some(child_node_scroll_geometry(
                    &style,
                    final_size,
                    output.content_size,
                    padding,
                    border,
                    output.scroll_geometry,
                )),
                scrollbar_size: child_scrollbar_size(&style),
                border,
                padding,
                margin,
            },
        );
    }

    AbsoluteLayoutResult {
        content_size: absolute_content_size,
        scrollable_overflow: absolute_scrollable_overflow,
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
struct Constants<S: LayoutScalar> {
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    direction: Direction,
    writing_mode: WritingMode,
    text_align: TextAlign,
    border: Edges<S>,
    padding: Edges<S>,
    padding_border_size: Size<S>,
    scrollbar_gutter: Edges<S>,
    content_box_inset: Edges<S>,
    own_top_margin: CollapsibleMarginOf<S>,
    own_bottom_margin: CollapsibleMarginOf<S>,
    collapse_top_margin: bool,
    collapse_bottom_margin: bool,
    can_collapse_through: bool,
}

impl<S: LayoutScalar> Constants<S> {
    fn with_inner_width(mut self, width: Option<S>) -> Self {
        self.node_inner_size.width = width;
        if let Some(width) = width {
            self.node_outer_size.width = Some(width + self.content_box_inset.horizontal_sum());
        }
        self
    }

    fn new(style: &NodeInputOf<S>, input: ComputeInputOf<S>) -> Self {
        let padding = style
            .padding
            .zip_inline_size(input.parent, |length, basis| {
                resolve_length_or_zero(length, basis)
            });
        let border = style.border.zip_inline_size(input.parent, |length, basis| {
            resolve_length_or_zero(length, basis)
        });
        let scrollbar_reservation = ScrollbarReservationOf::from_overflow(
            style.overflow,
            style.scrollbar_width,
            style.direction,
        );
        let scrollbar_gutter = scrollbar_reservation.inset();
        let padding_border_size = (padding + border).sum_axes();
        let content_box_inset =
            content_box_inset_with_scrollbar(padding, border, scrollbar_reservation);
        let content_box_inset_size = content_box_inset.sum_axes();
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border_size
        } else {
            Size::ZERO
        };
        let (style_size, min_size, max_size) = match input.sizing_mode {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::InherentSize => {
                let style_size = style
                    .size
                    .zip_map(input.parent, |dimension, basis| {
                        resolve_dimension(dimension, basis)
                    })
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let min_size = style
                    .min_size
                    .zip_map(input.parent, |dimension, basis| {
                        resolve_dimension(dimension, basis)
                    })
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let max_size = style
                    .max_size
                    .zip_map(input.parent, |dimension, basis| {
                        resolve_dimension(dimension, basis)
                    })
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                (style_size, min_size, max_size)
            }
        };
        let min_max_definite_size = min_size.zip_map(max_size, |min, max| match (min, max) {
            (Some(min), Some(max)) if max <= min => Some(min),
            _ => None,
        });
        let is_root = input.run_mode == RunMode::PerformRootLayout;
        let blocks_margin_collapse =
            style.overflow.x.blocks_margin_collapse() || style.overflow.y.blocks_margin_collapse();
        let is_margin_collapsing_block = style.display == super::Display::Block;
        let can_collapse_through = is_margin_collapsing_block
            && !is_root
            && !blocks_margin_collapse
            && style.position == Position::Relative
            && padding.top == S::ZERO
            && padding.bottom == S::ZERO
            && border.top == S::ZERO
            && border.bottom == S::ZERO
            && !matches!(style_size.height, Some(height) if height > S::ZERO)
            && !matches!(min_size.height, Some(height) if height > S::ZERO);
        let node_outer_size = input
            .known
            .or(min_max_definite_size)
            .or(style_size.clamp_optional(min_size, max_size))
            .max_optional(padding_border_size.map(Some));
        let node_inner_size = node_outer_size.sub_optional(content_box_inset_size);

        Self {
            node_outer_size,
            node_inner_size,
            node_min_size: min_size,
            node_max_size: max_size,
            direction: style.direction,
            writing_mode: style.writing_mode,
            text_align: style.text_align,
            border,
            padding,
            padding_border_size,
            scrollbar_gutter,
            content_box_inset,
            own_top_margin: CollapsibleMarginOf::<S>::from_margin(
                resolve_auto_optional(style.margin.top, input.parent.width).unwrap_or(S::ZERO),
            ),
            own_bottom_margin: CollapsibleMarginOf::<S>::from_margin(
                resolve_auto_optional(style.margin.bottom, input.parent.width).unwrap_or(S::ZERO),
            ),
            collapse_top_margin: is_margin_collapsing_block
                && !is_root
                && style.position == Position::Relative
                && !blocks_margin_collapse
                && padding.top == S::ZERO
                && border.top == S::ZERO,
            collapse_bottom_margin: is_margin_collapsing_block
                && !is_root
                && style.position == Position::Relative
                && !blocks_margin_collapse
                && padding.bottom == S::ZERO
                && border.bottom == S::ZERO
                && style_size.height.is_none(),
            can_collapse_through,
        }
    }
}

fn child_scrollbar_size<S: LayoutScalar>(style: &NodeInputOf<S>) -> Size<S> {
    scrollbar_size_from_overflow(style.overflow, style.scrollbar_width)
}

fn resolve_auto_optional<S: LayoutScalar>(length: LengthAutoOf<S>, basis: Option<S>) -> Option<S> {
    resolution_optional(length.resolve_with_status(basis))
}

fn resolve_dimension<S: LayoutScalar>(dimension: DimensionOf<S>, basis: Option<S>) -> Option<S> {
    resolution_optional(dimension.resolve_with_status(basis))
}

fn resolve_length_or_zero<S: LayoutScalar>(length: LengthOf<S>, basis: Option<S>) -> S {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolution_or_zero<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> S {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution
            .value
            .expect("resolved length resolution must carry a value"),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => S::ZERO,
        LengthResolutionStatus::InvalidNumeric => panic!("invalid numeric length resolution"),
    }
}

fn resolution_optional<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> Option<S> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution.value,
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => None,
        LengthResolutionStatus::InvalidNumeric => panic!("invalid numeric length resolution"),
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
            self.width.map(|width| width - amount.width),
            self.height.map(|height| height - amount.height),
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
    fn max_optional(self, min: Size<Option<S>>) -> Self;
}

impl<S: LayoutScalar> SizeConcreteExt<S> for Size<S> {
    fn clamp_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self {
        Size::new(
            self.width.clamp_optional(min.width, max.width),
            self.height.clamp_optional(min.height, max.height),
        )
    }

    fn max_optional(self, min: Size<Option<S>>) -> Self {
        Size::new(
            min.width.map_or(self.width, |min| self.width.max(min)),
            min.height.map_or(self.height, |min| self.height.max(min)),
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
