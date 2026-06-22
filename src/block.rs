use super::inline::{
    AtomicInlineInput, AtomicInlineItem, AtomicInlineLayoutItem, layout_atomic_inline_items,
};
use super::{
    Available, Baselines, BoxSizing, Clear, CollapsibleMargin, Compute, ComputeInput,
    ComputeOutput, Dimension, Direction, Edges, Float, Length, LengthAuto, NodeInput, NodeOutput,
    Overflow, Point, Position, RequestedAxis, RunMode, Scalar, Size, SizingMode, TextAlign,
    Traverse, VerticalAlign, WritingMode,
};

pub fn compute_block<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInput,
) -> ComputeOutput
where
    Tree: Compute,
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
        return ComputeOutput::from_outer_size(Size::new(width, height));
    }
    if input.run_mode == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
        && !normal_flow_children_can_establish_baseline(tree, &children)
    {
        return ComputeOutput::from_outer_size(Size::new(width, height));
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
                (output_size.width - constants.content_box_inset.horizontal_sum()).max(0.0);
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
        let mut output =
            ComputeOutput::from_sizes_and_baselines(output_size, Size::ZERO, final_pass.baselines);
        output.top_margin = top_margin;
        output.bottom_margin = bottom_margin;
        output.margins_can_collapse_through = margins_can_collapse_through;
        output
    } else {
        layout_floats(tree, &final_pass.pending_floats, output_size, &constants);
        let content_size = max_content_size(
            final_pass.content_size,
            layout_absolute_children(
                tree,
                &children,
                &final_pass.static_positions,
                output_size,
                &constants,
            ),
        );
        let mut output = ComputeOutput::from_sizes_and_baselines(
            output_size,
            content_size,
            final_pass.baselines,
        );
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
        let style = tree.node_input(child);
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

struct PendingFloat<Node> {
    node: Node,
    order: u32,
    side: Float,
    clear: Clear,
    y: Scalar,
    size: Size,
    content_size: Size,
    scrollbar_size: Size,
    border: Edges,
    padding: Edges,
    margin: Edges,
}

#[derive(Clone, Copy, Debug)]
struct ActiveFloat {
    side: Float,
    x: Scalar,
    y: Scalar,
    width: Scalar,
    height: Scalar,
}

impl ActiveFloat {
    fn bottom(self) -> Scalar {
        self.y + self.height
    }

    fn overlaps_y(self, y: Scalar) -> bool {
        y >= self.y && y < self.bottom()
    }
}

#[derive(Clone, Debug)]
struct FloatExclusions {
    content_width: Scalar,
    inset: Edges,
    active: Vec<ActiveFloat>,
}

impl FloatExclusions {
    fn new(content_width: Scalar, inset: Edges) -> Self {
        Self {
            content_width,
            inset,
            active: Vec::new(),
        }
    }

    fn place_float<Node>(&mut self, float: &PendingFloat<Node>, y: Scalar) -> Point {
        let margin_box = float.size + float.margin.sum_axes();
        let mut candidate_y = self.clearance_y(y, float.clear);

        loop {
            let (left_edge, right_edge, next_y) = self.available_band(candidate_y);
            let available_width = (right_edge - left_edge).max(0.0);
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
        y: Scalar,
        size: Size,
        margin: Edges,
        clear: Clear,
        fallback_x: Scalar,
    ) -> Point {
        let mut candidate_y = self.clearance_y(y, clear);
        loop {
            let (left_edge, right_edge, next_y) = self.available_band(candidate_y);
            let margin_box_width = size.width + margin.horizontal_sum();
            let fallback_left = fallback_x - margin.left;
            let fallback_right = fallback_x + size.width + margin.right;
            if margin_box_width <= (right_edge - left_edge).max(0.0) {
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

    fn clearance_y(&self, y: Scalar, clear: Clear) -> Scalar {
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
            .fold(y, Scalar::max)
    }

    fn available_band(&self, y: Scalar) -> (Scalar, Scalar, Option<Scalar>) {
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
            next_y = Some(next_y.map_or(float.bottom(), |current: Scalar| {
                current.min(float.bottom())
            }));
        }

        (left_edge, right_edge, next_y)
    }
}

struct InFlowResult<Node> {
    content_size: Size,
    baselines: Baselines,
    static_positions: Vec<(Node, Point)>,
    pending_floats: Vec<PendingFloat<Node>>,
    cursor_y: Scalar,
    top_margin: CollapsibleMargin,
    active_margin: CollapsibleMargin,
    active_margin_can_collapse_with_parent: bool,
    all_in_flow_children_can_collapse_through: bool,
}

impl<Node> InFlowResult<Node> {
    fn top_margin(&self, constants: &Constants) -> CollapsibleMargin {
        if constants.collapse_top_margin {
            self.top_margin
        } else {
            constants.own_top_margin
        }
    }

    fn bottom_margin(&self, constants: &Constants) -> CollapsibleMargin {
        if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
            self.active_margin
        } else {
            constants.own_bottom_margin
        }
    }

    fn auto_height(&self, constants: &Constants) -> Scalar {
        let bottom_margin_offset =
            if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
                0.0
            } else {
                self.active_margin.resolve()
            };
        (self.cursor_y + bottom_margin_offset + constants.content_box_inset.bottom)
            .max(constants.content_box_inset.vertical_sum())
    }
}

fn layout_in_flow_children<Tree>(
    tree: &mut Tree,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants,
    input: ComputeInput,
    inner_width: Option<Scalar>,
    set_layout: bool,
) -> InFlowResult<<Tree as Traverse>::Node>
where
    Tree: Compute,
{
    let node_inner_size = Size::new(inner_width, constants.node_inner_size.height);
    let mut cursor_y = constants.content_box_inset.top;
    let mut content_size = Size::ZERO;
    let mut first_baseline = None;
    let mut last_baseline = None;
    let mut static_positions = Vec::new();
    let mut active_margin = CollapsibleMargin::ZERO;
    let mut top_margin = CollapsibleMargin::ZERO;
    let mut is_collapsing_first_margin = constants.collapse_top_margin;
    let mut all_in_flow_children_can_collapse_through = true;
    let mut active_margin_can_collapse_with_parent = constants.collapse_top_margin;
    let mut pending_floats = Vec::new();
    let mut float_intrinsics = FloatIntrinsics::new(
        inner_width
            .map(Available::definite)
            .unwrap_or(input.available.width),
    );
    let content_width = inner_width
        .or(input.available.width.into_option())
        .unwrap_or(0.0);
    let mut float_exclusions = FloatExclusions::new(content_width, constants.content_box_inset);

    let mut index = 0;
    while index < children.len() {
        let order = index;
        let child = children[index];
        let child_style = tree.node_input(child).clone();
        if child_style.display == super::Display::None {
            if set_layout {
                tree.set_unrounded(child, NodeOutput::with_order(order as u32));
                tree.compute_child(child, ComputeInput::HIDDEN);
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
            index += 1;
            while index < children.len() {
                let run_style = tree.node_input(children[index]);
                if run_style.display == super::Display::None
                    || run_style.position == Position::Absolute
                {
                    index += 1;
                    continue;
                }
                if run_style.float != Float::None {
                    break;
                }
                if !run_style.display.is_inline_level() {
                    break;
                }
                index += 1;
            }

            let collapsed_margin = active_margin.resolve();
            cursor_y += collapsed_margin;
            if is_collapsing_first_margin {
                is_collapsing_first_margin = false;
            }

            let placement = layout_atomic_inline_run(
                tree,
                &children[run_start..index],
                AtomicInlineRunContext {
                    order_start: run_start as u32,
                    cursor_y,
                    constants,
                    input,
                    node_inner_size,
                    set_layout,
                },
            );
            content_size.width = content_size.width.max(placement.content_size.width);
            content_size.height = content_size.height.max(placement.content_size.height);
            static_positions.extend(placement.static_positions);
            if let Some(baseline) = placement.first_baseline {
                let absolute_baseline = cursor_y + baseline;
                first_baseline.get_or_insert(absolute_baseline);
            }
            if let Some(baseline) = placement.last_baseline {
                last_baseline = Some(cursor_y + baseline);
            }
            cursor_y += placement.size.height;
            active_margin = CollapsibleMargin::ZERO;
            active_margin_can_collapse_with_parent = false;
            all_in_flow_children_can_collapse_through = false;
            continue;
        }

        let unresolved_margin = child_style
            .margin
            .zip_inline_size(node_inner_size, resolve_auto_optional);
        let child_padding = child_style
            .padding
            .zip_inline_size(node_inner_size, resolve_length_or_zero);
        let child_border = child_style
            .border
            .zip_inline_size(node_inner_size, resolve_length_or_zero);
        let child_non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(0.0));
        let available_child_width = node_inner_size
            .width
            .or(input.available.width.into_option())
            .map(|width| (width - child_non_auto_margin.horizontal_sum()).max(0.0));
        let child_known = in_flow_child_known_size(
            &child_style,
            child_padding + child_border,
            node_inner_size,
            available_child_width,
        );
        let output = tree.compute_child(
            child,
            ComputeInput {
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
                    Available::MAX_CONTENT,
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
            };
            let float_location = float_exclusions.place_float(&pending_float, cursor_y);
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
                Size::new(node_inner_size.width, Some(0.0)),
                resolve_auto_optional,
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
        cursor_y += collapsed_margin;
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
                NodeOutput {
                    order: order as u32,
                    location,
                    size: output.size,
                    content_size: output.content_size,
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
        baselines: Baselines {
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

struct InlineRunPlacement<Node> {
    size: Size,
    content_size: Size,
    static_positions: Vec<(Node, Point)>,
    first_baseline: Option<Scalar>,
    last_baseline: Option<Scalar>,
}

struct AtomicInlineRunContext<'a> {
    order_start: u32,
    cursor_y: Scalar,
    constants: &'a Constants,
    input: ComputeInput,
    node_inner_size: Size<Option<Scalar>>,
    set_layout: bool,
}

fn layout_atomic_inline_run<Tree>(
    tree: &mut Tree,
    run: &[<Tree as Traverse>::Node],
    context: AtomicInlineRunContext<'_>,
) -> InlineRunPlacement<<Tree as Traverse>::Node>
where
    Tree: Compute,
{
    let AtomicInlineRunContext {
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
        let child_style = tree.node_input(child).clone();
        if child_style.display == super::Display::None {
            if set_layout {
                tree.set_unrounded(child, NodeOutput::with_order(order_start + offset as u32));
                tree.compute_child(child, ComputeInput::HIDDEN);
            }
            continue;
        }
        if child_style.position == Position::Absolute {
            static_positions.push((child, absolute_static_position(cursor_y, constants)));
            continue;
        }
        let child_padding = child_style
            .padding
            .zip_inline_size(node_inner_size, resolve_length_or_zero);
        let child_border = child_style
            .border
            .zip_inline_size(node_inner_size, resolve_length_or_zero);
        let output = tree.compute_child(
            child,
            ComputeInput {
                run_mode: input.run_mode.for_child(),
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(node_inner_size.width, None),
                available: Size::new(
                    node_inner_size
                        .width
                        .map(Available::definite)
                        .unwrap_or(input.available.width),
                    Available::MAX_CONTENT,
                ),
            },
        );
        let unresolved_margin = child_style
            .margin
            .zip_inline_size(node_inner_size, resolve_auto_optional);
        let child_margin = resolve_atomic_inline_margin(unresolved_margin);

        let item = AtomicInlineItem {
            order: order_start + offset as u32,
            size: output.size,
            content_size: output.content_size,
            margin: child_margin,
            padding: child_padding,
            border: child_border,
            scrollbar_size: child_scrollbar_size(&child_style),
            first_baseline: if child_style.vertical_align == VerticalAlign::Top {
                Some(0.0)
            } else {
                output.last_baselines.y.or(output.first_baselines.y)
            },
        };
        run_children.push((child, child_style, output, item));
        items.push(item);
    }

    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: node_inner_size
            .width
            .map(Available::definite)
            .unwrap_or(input.available.width),
        writing_mode: constants.writing_mode,
        items,
    });
    let run_offset = inline_run_offset(report.size.width, constants, node_inner_size.width);
    let mut content_size = content_size_contribution(
        Point::new(run_offset, cursor_y - constants.content_box_inset.top),
        report.size,
        report.content_size,
        Point::new(Overflow::Visible, Overflow::Visible),
    );

    for ((child, child_style, output, _source_item), item) in run_children.iter().zip(&report.items)
    {
        let inset_offset = relative_inset_offset(
            child_style.inset.zip_size(
                Size::new(node_inner_size.width, Some(0.0)),
                resolve_auto_optional,
            ),
            constants.direction,
        );
        let item_x = inline_item_x(
            *item,
            report.size.width,
            constants.direction,
            constants.writing_mode,
        );
        let location = Point::new(
            run_offset + item_x + inset_offset.x,
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
            let inset_offset = relative_inset_offset(
                child_style.inset.zip_size(
                    Size::new(node_inner_size.width, Some(0.0)),
                    resolve_auto_optional,
                ),
                constants.direction,
            );

            tree.set_unrounded(
                *child,
                NodeOutput {
                    order: item.order,
                    location: Point::new(
                        constants.content_box_inset.left + run_offset + item_x + inset_offset.x,
                        cursor_y + item.location.y + inset_offset.y,
                    ),
                    size: item.size,
                    content_size: item.content_size,
                    scrollbar_size: item.scrollbar_size,
                    border: item.border,
                    padding: item.padding,
                    margin: item.margin,
                },
            );
        }
    }

    InlineRunPlacement {
        size: report.size,
        content_size,
        static_positions,
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
    }
}

fn inline_run_offset(
    run_width: Scalar,
    constants: &Constants,
    resolved_inner_width: Option<Scalar>,
) -> Scalar {
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
    let free_space = (container_inner_width - run_width).max(0.0);
    match (constants.text_align, constants.direction) {
        (TextAlign::Auto, Direction::Ltr)
        | (TextAlign::LegacyLeft, Direction::Ltr)
        | (TextAlign::LegacyLeft, Direction::Rtl) => 0.0,
        (TextAlign::Auto, Direction::Rtl)
        | (TextAlign::LegacyRight, Direction::Ltr)
        | (TextAlign::LegacyRight, Direction::Rtl) => free_space,
        (TextAlign::LegacyCenter, _) => free_space / 2.0,
    }
}

fn inline_item_x(
    item: AtomicInlineLayoutItem,
    run_width: Scalar,
    direction: Direction,
    writing_mode: WritingMode,
) -> Scalar {
    if direction == Direction::Rtl && writing_mode == WritingMode::HorizontalTb {
        run_width - item.location.x - item.size.width
    } else {
        item.location.x
    }
}

fn layout_floats<Tree>(
    tree: &mut Tree,
    floats: &[PendingFloat<<Tree as Traverse>::Node>],
    container_size: Size,
    constants: &Constants,
) where
    Tree: Compute,
{
    let mut float_exclusions = FloatExclusions::new(
        (container_size.width - constants.content_box_inset.horizontal_sum()).max(0.0),
        constants.content_box_inset,
    );

    for float in floats {
        let location = float_exclusions.place_float(float, float.y);
        tree.set_unrounded(
            float.node,
            NodeOutput {
                order: float.order,
                location,
                size: float.size,
                content_size: float.content_size,
                scrollbar_size: float.scrollbar_size,
                border: float.border,
                padding: float.padding,
                margin: float.margin,
            },
        );
    }
}

struct FloatIntrinsics {
    available_width: Available,
    contribution: Scalar,
}

impl FloatIntrinsics {
    const fn new(available_width: Available) -> Self {
        Self {
            available_width,
            contribution: 0.0,
        }
    }

    fn add(&mut self, width: Scalar, _float: Float, _clear: Clear) {
        match self.available_width {
            Available::Definite(_) => {}
            Available::MinContent => self.contribution = self.contribution.max(width),
            Available::MaxContent => self.contribution += width,
        }
    }

    const fn result(&self) -> Scalar {
        self.contribution
    }
}

fn child_margin_can_collapse_with_parent(style: &NodeInput) -> bool {
    style.display == super::Display::Block && style.position == Position::Relative
}

fn in_flow_child_known_size(
    style: &NodeInput,
    padding_border: Edges,
    parent: Size<Option<Scalar>>,
    available_width: Option<Scalar>,
) -> Size<Option<Scalar>> {
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let min_size = style
        .min_size
        .zip_map(parent, resolve_dimension)
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let mut max_size = style
        .max_size
        .zip_map(parent, resolve_dimension)
        .add_optional(box_sizing_adjustment);
    let aspect_height_limit = style
        .aspect_ratio
        .zip(max_size.height)
        .and_then(|(ratio, height)| max_size.width.is_none().then_some(height * ratio));
    if let Some(width) = aspect_height_limit {
        max_size.width = Some(width);
    }
    let mut known = style
        .size
        .zip_map(parent, resolve_dimension)
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

fn in_flow_child_available_width(
    style: &NodeInput,
    available_width: Option<Scalar>,
    fallback: Available,
) -> Available {
    match style.size.width {
        Dimension::MinContent => Available::MIN_CONTENT,
        Dimension::MaxContent => Available::MAX_CONTENT,
        Dimension::Px(_)
        | Dimension::Percent(_)
        | Dimension::Calc(_)
        | Dimension::Fr(_)
        | Dimension::Auto => available_width.map(Available::definite).unwrap_or(fallback),
    }
}

fn relative_inset_offset(inset: Edges<Option<Scalar>>, direction: Direction) -> Point {
    Point::new(
        if direction == Direction::Rtl {
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

fn resolve_in_flow_margin(
    margin: Edges<Option<Scalar>>,
    child_size: Size,
    container_width: Option<Scalar>,
) -> Edges {
    let non_auto_horizontal = margin.left.unwrap_or(0.0) + margin.right.unwrap_or(0.0);
    let auto_count = usize::from(margin.left.is_none()) + usize::from(margin.right.is_none());
    let auto_horizontal = if auto_count == 0 {
        0.0
    } else {
        container_width
            .map(|width| (width - child_size.width - non_auto_horizontal).max(0.0))
            .unwrap_or(0.0)
            / auto_count as Scalar
    };

    Edges {
        left: margin.left.unwrap_or(auto_horizontal),
        right: margin.right.unwrap_or(auto_horizontal),
        top: margin.top.unwrap_or(0.0),
        bottom: margin.bottom.unwrap_or(0.0),
    }
}

fn resolve_atomic_inline_margin(margin: Edges<Option<Scalar>>) -> Edges {
    margin.map(|value| value.unwrap_or(0.0))
}

fn in_flow_child_x(size: Size, margin: Edges, constants: &Constants) -> Scalar {
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
                x -= if constants.text_align == TextAlign::LegacyCenter {
                    free_space / 2.0
                } else {
                    free_space
                };
            }
            (TextAlign::LegacyRight, Direction::Ltr)
            | (TextAlign::LegacyCenter, Direction::Ltr) => {
                x += if constants.text_align == TextAlign::LegacyCenter {
                    free_space / 2.0
                } else {
                    free_space
                };
            }
        }
    }

    x
}

fn absolute_static_position(cursor_y: Scalar, constants: &Constants) -> Point {
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

fn content_size_contribution(
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

fn max_content_size(a: Size, b: Size) -> Size {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

fn layout_absolute_children<Tree>(
    tree: &mut Tree,
    children: &[<Tree as Traverse>::Node],
    static_positions: &[(<Tree as Traverse>::Node, Point)],
    container: Size,
    constants: &Constants,
) -> Size
where
    Tree: Compute,
{
    let area_start_x = constants.border.left + constants.scrollbar_gutter.left;
    let max_area_start_x = (container.width - constants.border.right).max(constants.border.left);
    let area_offset = Point::new(area_start_x.min(max_area_start_x), constants.border.top);
    let area_size = Size::new(
        (container.width
            - constants.border.horizontal_sum()
            - constants.scrollbar_gutter.horizontal_sum())
        .max(0.0),
        (container.height
            - constants.border.vertical_sum()
            - constants.scrollbar_gutter.vertical_sum())
        .max(0.0),
    );
    let available = Size::new(
        Available::definite(area_size.width),
        Available::definite(area_size.height),
    );

    let mut absolute_content_size = Size::ZERO;
    for (order, child) in children.iter().copied().enumerate() {
        let style = tree.node_input(child).clone();
        if style.position != Position::Absolute || style.display == super::Display::None {
            continue;
        }

        let padding = style
            .padding
            .zip_inline_size(area_size.map(Some), resolve_length_or_zero);
        let border = style
            .border
            .zip_inline_size(area_size.map(Some), resolve_length_or_zero);
        let unresolved_margin = style
            .margin
            .zip_inline_size(area_size.map(Some), resolve_auto_optional);
        let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(0.0));
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::ZERO
        };
        let min_size = style
            .min_size
            .zip_map(area_size.map(Some), resolve_dimension)
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment)
            .or(padding_border.sum_axes().map(Some))
            .max_optional(padding_border.sum_axes().map(Some));
        let max_size = style
            .max_size
            .zip_map(area_size.map(Some), resolve_dimension)
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);
        let style_size = style
            .size
            .zip_map(area_size.map(Some), resolve_dimension)
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
            .zip_size(area_size.map(Some), resolve_auto_optional);
        if known_size.width.is_none()
            && let (Some(left), Some(right)) = (inset.left, inset.right)
        {
            known_size.width = Some(
                (area_size.width - non_auto_margin.horizontal_sum() - left - right)
                    .max(0.0)
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
                    .max(0.0)
                    .clamp_optional(min_size.height, max_size.height),
            );
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_optional(min_size, max_size);
        }

        let output = tree.compute_child(
            child,
            ComputeInput {
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

        tree.set_unrounded(
            child,
            NodeOutput {
                order: order as u32,
                location,
                size: final_size,
                content_size: output.content_size,
                scrollbar_size: child_scrollbar_size(&style),
                border,
                padding,
                margin,
            },
        );
    }

    absolute_content_size
}

struct AbsoluteAxis {
    start: Option<Scalar>,
    end: Option<Scalar>,
    direction: Direction,
    area_start: Scalar,
    area_size: Scalar,
    size: Scalar,
    margin_start: Scalar,
    margin_end: Scalar,
    static_position: Scalar,
}

impl AbsoluteAxis {
    fn location(self) -> Scalar {
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

fn resolve_absolute_margin(
    margin: Edges<Option<Scalar>>,
    inset: Edges<Option<Scalar>>,
    style_size: Size<Option<Scalar>>,
    final_size: Size,
    area_size: Size,
) -> Edges {
    let non_auto = Edges {
        left: if inset.left.is_some() {
            margin.left.unwrap_or(0.0)
        } else {
            0.0
        },
        right: if inset.right.is_some() {
            margin.right.unwrap_or(0.0)
        } else {
            0.0
        },
        top: if inset.top.is_some() {
            margin.top.unwrap_or(0.0)
        } else {
            0.0
        },
        bottom: if inset.bottom.is_some() {
            margin.bottom.unwrap_or(0.0)
        } else {
            0.0
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
struct AutoMarginAxis {
    start_is_auto: bool,
    end_is_auto: bool,
    start: Option<Scalar>,
    end: Option<Scalar>,
    area_size: Scalar,
    style_size: Option<Scalar>,
    item_size: Scalar,
    non_auto_margin_sum: Scalar,
}

fn auto_margin_size(axis: AutoMarginAxis) -> Scalar {
    let auto_count = usize::from(axis.start_is_auto) + usize::from(axis.end_is_auto);
    if auto_count == 0 || axis.start.is_none() && axis.end.is_none() {
        return 0.0;
    }

    let available = axis
        .end
        .map(|end| axis.area_size - end - axis.start.unwrap_or(0.0))
        .unwrap_or(axis.item_size);
    let free_space = available - axis.item_size - axis.non_auto_margin_sum;
    if auto_count == 2
        && axis
            .style_size
            .is_none_or(|style_size| style_size >= free_space)
    {
        0.0
    } else {
        free_space / auto_count as Scalar
    }
}

#[derive(Clone, Copy, Debug)]
struct Constants {
    node_outer_size: Size<Option<Scalar>>,
    node_inner_size: Size<Option<Scalar>>,
    node_min_size: Size<Option<Scalar>>,
    node_max_size: Size<Option<Scalar>>,
    direction: Direction,
    writing_mode: WritingMode,
    text_align: TextAlign,
    border: Edges,
    padding_border_size: Size,
    scrollbar_gutter: Edges,
    content_box_inset: Edges,
    own_top_margin: CollapsibleMargin,
    own_bottom_margin: CollapsibleMargin,
    collapse_top_margin: bool,
    collapse_bottom_margin: bool,
    can_collapse_through: bool,
}

impl Constants {
    fn with_inner_width(mut self, width: Option<Scalar>) -> Self {
        self.node_inner_size.width = width;
        if let Some(width) = width {
            self.node_outer_size.width = Some(width + self.content_box_inset.horizontal_sum());
        }
        self
    }

    fn new(style: &NodeInput, input: ComputeInput) -> Self {
        let padding = style
            .padding
            .zip_inline_size(input.parent, resolve_length_or_zero);
        let border = style
            .border
            .zip_inline_size(input.parent, resolve_length_or_zero);
        let scrollbar_gutter = Size::new(
            if style.overflow.y == Overflow::Scroll {
                style.scrollbar_width
            } else {
                0.0
            },
            if style.overflow.x == Overflow::Scroll {
                style.scrollbar_width
            } else {
                0.0
            },
        );
        let scrollbar_gutter = match style.direction {
            Direction::Ltr => Edges {
                right: scrollbar_gutter.width,
                bottom: scrollbar_gutter.height,
                ..Edges::ZERO
            },
            Direction::Rtl => Edges {
                left: scrollbar_gutter.width,
                bottom: scrollbar_gutter.height,
                ..Edges::ZERO
            },
        };
        let padding_border_size = (padding + border).sum_axes();
        let content_box_inset = padding + border + scrollbar_gutter;
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
                    .zip_map(input.parent, resolve_dimension)
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let min_size = style
                    .min_size
                    .zip_map(input.parent, resolve_dimension)
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let max_size = style
                    .max_size
                    .zip_map(input.parent, resolve_dimension)
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
            && padding.top == 0.0
            && padding.bottom == 0.0
            && border.top == 0.0
            && border.bottom == 0.0
            && !matches!(style_size.height, Some(height) if height > 0.0)
            && !matches!(min_size.height, Some(height) if height > 0.0);
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
            padding_border_size,
            scrollbar_gutter,
            content_box_inset,
            own_top_margin: CollapsibleMargin::from_margin(
                resolve_auto_optional(style.margin.top, input.parent.width).unwrap_or(0.0),
            ),
            own_bottom_margin: CollapsibleMargin::from_margin(
                resolve_auto_optional(style.margin.bottom, input.parent.width).unwrap_or(0.0),
            ),
            collapse_top_margin: is_margin_collapsing_block
                && !is_root
                && style.position == Position::Relative
                && !blocks_margin_collapse
                && padding.top == 0.0
                && border.top == 0.0,
            collapse_bottom_margin: is_margin_collapsing_block
                && !is_root
                && style.position == Position::Relative
                && !blocks_margin_collapse
                && padding.bottom == 0.0
                && border.bottom == 0.0
                && style_size.height.is_none(),
            can_collapse_through,
        }
    }
}

fn child_scrollbar_size(style: &NodeInput) -> Size {
    Size::new(
        if style.overflow.y == Overflow::Scroll {
            style.scrollbar_width
        } else {
            0.0
        },
        if style.overflow.x == Overflow::Scroll {
            style.scrollbar_width
        } else {
            0.0
        },
    )
}

fn resolve_length_or_zero(length: Length, basis: Option<Scalar>) -> Scalar {
    length.resolve_or_zero(basis)
}

fn resolve_auto_optional(length: LengthAuto, basis: Option<Scalar>) -> Option<Scalar> {
    length.resolve_optional(basis)
}

fn resolve_dimension(dimension: Dimension, basis: Option<Scalar>) -> Option<Scalar> {
    dimension.resolve_optional(basis)
}

trait SizeOptionExt {
    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size) -> Size;
    fn add_optional(self, amount: Size) -> Self;
    fn sub_optional(self, amount: Size) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<Scalar>) -> Self;
    fn clamp_optional(self, min: Self, max: Self) -> Self;
    fn max_optional(self, min: Self) -> Self;
}

impl SizeOptionExt for Size<Option<Scalar>> {
    fn or(self, other: Self) -> Self {
        Size::new(self.width.or(other.width), self.height.or(other.height))
    }

    fn unwrap_or(self, fallback: Size) -> Size {
        Size::new(
            self.width.unwrap_or(fallback.width),
            self.height.unwrap_or(fallback.height),
        )
    }

    fn add_optional(self, amount: Size) -> Self {
        Size::new(
            self.width.map(|width| width + amount.width),
            self.height.map(|height| height + amount.height),
        )
    }

    fn sub_optional(self, amount: Size) -> Self {
        Size::new(
            self.width.map(|width| width - amount.width),
            self.height.map(|height| height - amount.height),
        )
    }

    fn apply_aspect_ratio(self, aspect_ratio: Option<Scalar>) -> Self {
        let Some(ratio) = aspect_ratio else {
            return self;
        };
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

trait SizeConcreteExt {
    fn clamp_optional(self, min: Size<Option<Scalar>>, max: Size<Option<Scalar>>) -> Self;
    fn max_optional(self, min: Size<Option<Scalar>>) -> Self;
}

impl SizeConcreteExt for Size {
    fn clamp_optional(self, min: Size<Option<Scalar>>, max: Size<Option<Scalar>>) -> Self {
        Size::new(
            self.width.clamp_optional(min.width, max.width),
            self.height.clamp_optional(min.height, max.height),
        )
    }

    fn max_optional(self, min: Size<Option<Scalar>>) -> Self {
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

impl ScalarExt for Scalar {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = max.map_or(self, |max| self.min(max));
        min.map_or(value, |min| value.max(min))
    }
}
