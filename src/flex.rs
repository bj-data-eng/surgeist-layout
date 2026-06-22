use super::{
    AlignContent, AlignItems, Available, Baselines, BoxSizing, CalcResolver, Compute, ComputeInput,
    ComputeOutput, Dimension, Direction, Edges, FlexDirection, Length, LengthAuto, NodeInput,
    NodeOutput, Overflow, Point, Position, RequestedAxis, RunMode, Scalar, Size, SizingMode,
    Traverse,
};

pub fn compute_flex<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInput,
) -> ComputeOutput
where
    Tree: Compute,
{
    let style = tree.node_input(node).clone();
    let constants = Constants::new(&style, input, tree.calc_resolver());
    if input.run_mode == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return ComputeOutput::from_outer_size(Size::new(width, height));
    }

    let mut items = collect_items(tree, node, &constants, input.run_mode);
    let mut lines = collect_flex_lines(&items, &constants);

    let mut layout_constants =
        resolved_layout_constants(tree, input, &style, &constants, &mut items, &lines);
    resolve_lines(tree, &mut items, &mut lines, &layout_constants);
    let cross_layout_constants = resolved_cross_layout_constants(&layout_constants, &lines);
    if cross_layout_constants.node_inner_size != layout_constants.node_inner_size {
        layout_constants = cross_layout_constants;
        resolve_lines(tree, &mut items, &mut lines, &layout_constants);
    } else {
        layout_constants = cross_layout_constants;
    }
    let absolute_content_size = if input.run_mode.is_perform_layout() {
        final_layout(tree, &mut items, &layout_constants);
        let absolute_content_size = layout_absolute_children(tree, node, &layout_constants);
        layout_hidden_children(tree, node);
        absolute_content_size
    } else {
        Size::ZERO
    };

    container_output(
        input,
        &style,
        &layout_constants,
        &items,
        &lines,
        absolute_content_size,
    )
}

#[derive(Clone, Copy)]
struct Constants {
    direction: FlexDirection,
    layout_direction: Direction,
    node_outer_size: Size<Option<Scalar>>,
    node_inner_size: Size<Option<Scalar>>,
    min_outer_size: Size<Option<Scalar>>,
    max_outer_size: Size<Option<Scalar>>,
    max_inner_size: Size<Option<Scalar>>,
    border: Edges,
    padding_border_size: Size,
    scrollbar_gutter: Point,
    content_box_inset: Edges,
    gap: Size,
    align_items: AlignItems,
    align_content: AlignContent,
    justify_content: AlignContent,
    wraps: bool,
    wrap_reverse: bool,
    available: Size<Available>,
    available_main: Available,
}

impl Constants {
    fn new(style: &NodeInput, input: ComputeInput, resolver: &dyn CalcResolver) -> Self {
        let padding = style
            .padding
            .zip_inline_size(input.parent, |length, basis| {
                resolve_length_or_zero(length, basis, resolver)
            });
        let border = style.border.zip_inline_size(input.parent, |length, basis| {
            resolve_length_or_zero(length, basis, resolver)
        });
        let scrollbar_gutter = Point::new(
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
        let mut content_box_inset = padding + border;
        content_box_inset.bottom += scrollbar_gutter.y;
        match style.direction {
            Direction::Ltr => content_box_inset.right += scrollbar_gutter.x,
            Direction::Rtl => content_box_inset.left += scrollbar_gutter.x,
        }
        let padding_border = (padding + border).sum_axes();
        let content_box_inset_size = content_box_inset.sum_axes();
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border
        } else {
            Size::ZERO
        };

        let (style_size, min_size, max_size) = match input.sizing_mode {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::InherentSize => {
                let style_size = style
                    .size
                    .zip_map(input.parent, |dimension, basis| {
                        resolve_dimension(dimension, basis, resolver)
                    })
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let min_size = style
                    .min_size
                    .zip_map(input.parent, |dimension, basis| {
                        resolve_dimension(dimension, basis, resolver)
                    })
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let max_size = style
                    .max_size
                    .zip_map(input.parent, |dimension, basis| {
                        resolve_dimension(dimension, basis, resolver)
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
        let node_outer_size = input
            .known
            .or(min_max_definite_size.or(style_size.clamp_optional(min_size, max_size)))
            .max_optional(padding_border.map(Some));
        let node_inner_size = node_outer_size
            .sub_optional(content_box_inset_size)
            .max_optional(Size::ZERO.map(Some));
        let max_inner_size = max_size
            .sub_optional(content_box_inset_size)
            .max_optional(Size::ZERO.map(Some));
        let gap = style.gap.zip_map(node_inner_size, |length, basis| {
            resolve_length_or_zero(length, basis, resolver)
        });

        Self {
            direction: style.flex_direction,
            layout_direction: style.direction,
            node_outer_size,
            node_inner_size,
            min_outer_size: min_size,
            max_outer_size: max_size,
            max_inner_size,
            border,
            padding_border_size: padding_border,
            scrollbar_gutter,
            content_box_inset,
            gap,
            align_items: style.align_items.unwrap_or(AlignItems::Stretch),
            align_content: style.align_content.unwrap_or(AlignContent::Stretch),
            justify_content: style.justify_content.unwrap_or(AlignContent::FlexStart),
            wraps: matches!(
                style.flex_wrap,
                super::FlexWrap::Wrap | super::FlexWrap::WrapReverse
            ),
            wrap_reverse: style.flex_wrap == super::FlexWrap::WrapReverse,
            available: input.available,
            available_main: input.available.main(style.flex_direction),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct FlexItem<Node> {
    node: Node,
    order: u32,
    size: Size<Option<Scalar>>,
    output: ComputeOutput,
    flex_basis: Scalar,
    flex_basis_is_definite: bool,
    hypothetical_main_size: Scalar,
    max_content_main_size: Scalar,
    target_size: Size,
    cross_size_is_auto: bool,
    automatic_min_main_size: Option<Scalar>,
    min_size: Size<Option<Scalar>>,
    max_size: Size<Option<Scalar>>,
    min_cross_size: Option<Scalar>,
    max_cross_size: Option<Scalar>,
    margin: Edges,
    margin_is_auto: Edges<bool>,
    inset: Edges<Option<Scalar>>,
    padding: Edges,
    border: Edges,
    overflow: Point<Overflow>,
    scrollbar_width: Scalar,
    align_self: AlignItems,
    baseline: Scalar,
    flex_grow: Scalar,
    flex_shrink: Scalar,
    offset_main: Scalar,
    offset_cross: Scalar,
}

#[derive(Clone, Copy, Debug)]
struct FlexLine {
    start: usize,
    end: usize,
    main_size: Scalar,
    cross_size: Scalar,
    offset_cross: Scalar,
}

fn collect_items<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    constants: &Constants,
    run_mode: RunMode,
) -> Vec<FlexItem<<Tree as Traverse>::Node>>
where
    Tree: Compute,
{
    let children = tree.children(node).collect::<Vec<_>>();
    let mut items = Vec::with_capacity(children.len());
    for (order, child) in children.into_iter().enumerate() {
        let child_style = tree.node_input(child).clone();
        if child_style.position == Position::Absolute || child_style.display == super::Display::None
        {
            continue;
        }

        let child = build_item(tree, child, order as u32, &child_style, constants, run_mode);
        items.push(child);
    }
    items
}

fn build_item<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    order: u32,
    style: &NodeInput,
    constants: &Constants,
    run_mode: RunMode,
) -> FlexItem<<Tree as Traverse>::Node>
where
    Tree: Compute,
{
    let resolver = tree.calc_resolver();
    let padding = style
        .padding
        .zip_inline_size(constants.node_inner_size, |length, basis| {
            resolve_length_or_zero(length, basis, resolver)
        });
    let border = style
        .border
        .zip_inline_size(constants.node_inner_size, |length, basis| {
            resolve_length_or_zero(length, basis, resolver)
        });
    let margin = style
        .margin
        .zip_inline_size(constants.node_inner_size, |length, basis| {
            resolve_auto_or_zero(length, basis, resolver)
        });
    let margin_is_auto = style.margin.map(LengthAuto::is_auto);
    let inset = style
        .inset
        .zip_size(constants.node_inner_size, |length, basis| {
            resolve_auto_optional(length, basis, resolver)
        });
    let padding_border = padding + border;
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let authored_size = style
        .size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis, resolver)
        })
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let size = authored_size;
    let resolved_flex_basis = resolve_dimension(
        style.flex_basis,
        constants.node_inner_size.main(constants.direction),
        resolver,
    )
    .map(|flex_basis| {
        let padding_border = padding_border.sum_axes().main(constants.direction);
        if style.box_sizing == BoxSizing::ContentBox {
            flex_basis + padding_border
        } else {
            flex_basis.max(padding_border)
        }
    });
    let size = match resolved_flex_basis {
        Some(flex_basis) => size.with_main(constants.direction, Some(flex_basis)),
        None => size,
    };
    let raw_min_size = style
        .min_size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis, resolver)
        });
    let raw_max_size = style
        .max_size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis, resolver)
        });
    let min_size = raw_min_size
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let max_size = raw_max_size
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let direction = constants.direction;
    let align_self = style.align_self.unwrap_or(constants.align_items);
    let cross_size_is_auto = style.size.cross(direction).is_auto();
    let available_inner_size = constants.node_inner_size.or(constants.max_inner_size);
    let available = Size::new(
        constants
            .node_inner_size
            .width
            .map(Available::definite)
            .or_else(|| constants.max_inner_size.width.map(Available::definite))
            .unwrap_or(constants.available.width),
        constants
            .node_inner_size
            .height
            .map(Available::definite)
            .or_else(|| constants.max_inner_size.height.map(Available::definite))
            .unwrap_or(constants.available.height),
    );
    let available = available.with_cross(
        direction,
        clamp_available(
            available.cross(direction),
            min_size.cross(direction),
            max_size.cross(direction),
        ),
    );
    let main_size_is_auto = size.main(direction).is_none();
    let use_content_sizing_for_base = main_size_is_auto && style.display == super::Display::Block;
    let mut child_known = size;
    if !constants.wraps
        && use_content_sizing_for_base
        && align_self == AlignItems::Stretch
        && cross_size_is_auto
        && !margin_is_auto.cross_start(direction, constants.layout_direction)
        && !margin_is_auto.cross_end(direction, constants.layout_direction)
        && let Some(cross_size) = available.cross(direction).into_option()
    {
        child_known = child_known.with_cross(
            direction,
            Some((cross_size - margin.cross_sum(direction)).max(0.0)),
        );
    }
    let mut child_known_for_base = flex_base_known_size(
        size.with_main(direction, None),
        available.cross(direction),
        style,
        constants,
        margin,
        margin_is_auto,
        align_self,
    );
    let padding_border_main = padding_border.sum_axes().main(direction);
    let flex_basis_floor_may_override_content = padding_border_main > 0.0
        || (tree.child_count(node) == 0 && authored_size.main(direction).is_some());
    if let Some(flex_basis) = resolved_flex_basis
        && flex_basis <= padding_border_main
        && flex_basis_floor_may_override_content
    {
        child_known_for_base = child_known_for_base.with_main(direction, Some(flex_basis));
    }
    let child_available = if use_content_sizing_for_base {
        available.with_main(
            direction,
            if constants.available_main == Available::MIN_CONTENT {
                Available::MIN_CONTENT
            } else {
                Available::MAX_CONTENT
            },
        )
    } else {
        available
    };
    let output = tree.compute_child(
        node,
        ComputeInput {
            run_mode,
            sizing_mode: if use_content_sizing_for_base {
                SizingMode::ContentSize
            } else {
                SizingMode::InherentSize
            },
            axis: RequestedAxis::Both,
            known: child_known,
            parent: available_inner_size,
            available: child_available,
        },
    );
    let automatic_min_main_size = automatic_min_main_size(
        tree,
        node,
        style,
        constants,
        box_sizing_adjustment,
        child_known_for_base,
    );
    let flex_basis = resolved_flex_basis
        .or_else(|| size.main(direction))
        .unwrap_or_else(|| {
            if let Some(ratio) = style.aspect_ratio {
                if let Some(cross) = child_known_for_base.cross(direction) {
                    return main_size_from_cross_aspect(direction, cross, ratio);
                }

                return output.size.main(direction);
            }
            tree.compute_child(
                node,
                ComputeInput {
                    run_mode: RunMode::ComputeSize,
                    sizing_mode: SizingMode::ContentSize,
                    axis: requested_axis(direction),
                    known: child_known_for_base,
                    parent: available_inner_size.with_main(direction, None),
                    available: child_available.with_main(direction, Available::MAX_CONTENT),
                },
            )
            .size
            .main(direction)
        });
    let hypothetical_main_size = clamp_main_size_axes(
        flex_basis,
        automatic_min_main_size,
        min_size.main(direction),
        max_size.main(direction),
    );
    let authored_main_size = authored_size.main(direction);
    let flex_basis_uses_padding_floor = resolved_flex_basis.is_some()
        && flex_basis <= padding_border_main
        && style.flex_grow == 0.0
        && (tree.child_count(node) > 0 || output.content_size.main(direction) <= flex_basis);
    let intrinsic_main_size = if flex_basis_uses_padding_floor {
        flex_basis
    } else if style.flex_basis == Dimension::Auto && authored_main_size.is_some() {
        authored_main_size.unwrap_or(0.0)
    } else {
        output
            .content_size
            .main(direction)
            .max(authored_main_size.unwrap_or(0.0))
    };
    let max_content_main_size = intrinsic_main_size
        .clamp_optional(min_size.main(direction), max_size.main(direction))
        .max(padding_border_main);
    let mut target_size = output.size.with_main(direction, hypothetical_main_size);
    if align_self != AlignItems::Stretch
        && cross_size_is_auto
        && let Some(ratio) = style.aspect_ratio
    {
        let transferred_cross = if direction.is_row() {
            hypothetical_main_size / ratio
        } else {
            hypothetical_main_size * ratio
        };
        target_size = target_size.with_cross(direction, transferred_cross);
    }
    target_size = target_size.with_cross(
        direction,
        target_size
            .cross(direction)
            .clamp_optional(raw_min_size.cross(direction), raw_max_size.cross(direction))
            .max(padding_border.sum_axes().cross(direction)),
    );
    let baseline = output.baselines().first_or_synthesize_block(output.size)
        + margin.cross_start(constants.direction, constants.layout_direction);

    FlexItem {
        node,
        order,
        size: authored_size,
        output,
        flex_basis,
        flex_basis_is_definite: resolved_flex_basis.is_some(),
        hypothetical_main_size,
        max_content_main_size,
        target_size,
        cross_size_is_auto,
        automatic_min_main_size,
        min_size,
        max_size,
        min_cross_size: raw_min_size.cross(direction),
        max_cross_size: raw_max_size.cross(direction),
        margin,
        margin_is_auto,
        inset,
        padding,
        border,
        overflow: style.overflow,
        scrollbar_width: style.scrollbar_width,
        align_self,
        baseline,
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,
        offset_main: 0.0,
        offset_cross: 0.0,
    }
}

fn automatic_min_main_size<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInput,
    constants: &Constants,
    box_sizing_adjustment: Size,
    child_known: Size<Option<Scalar>>,
) -> Option<Scalar>
where
    Tree: Compute,
{
    let direction = constants.direction;
    if !style.min_size.main(direction).is_auto() || flex_automatic_minimum_is_zero(style.overflow) {
        return None;
    }

    let resolver = tree.calc_resolver();
    let authored_size = style
        .size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis, resolver)
        })
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let min_size = style
        .min_size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis, resolver)
        })
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let resolved_max_size = style
        .max_size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis, resolver)
        })
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let padding_border = style
        .padding
        .zip_inline_size(constants.node_inner_size, |length, basis| {
            resolve_length_or_zero(length, basis, resolver)
        })
        + style
            .border
            .zip_inline_size(constants.node_inner_size, |length, basis| {
                resolve_length_or_zero(length, basis, resolver)
            });

    let available = Size::from_main_cross(
        direction,
        Available::MIN_CONTENT,
        clamp_available(
            constants
                .node_inner_size
                .cross(direction)
                .map(Available::definite)
                .unwrap_or(Available::MAX_CONTENT),
            min_size.cross(direction),
            resolved_max_size.cross(direction),
        ),
    );
    let output = tree.compute_child(
        node,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::ContentSize,
            axis: requested_axis(direction),
            known: child_known,
            parent: constants.node_inner_size.with_main(direction, None),
            available,
        },
    );

    let mut min_content = output
        .size
        .main(direction)
        .clamp_optional(None, authored_size.main(direction))
        .clamp_optional(None, resolved_max_size.main(direction));
    if let Some(ratio) = style.aspect_ratio
        && let Some(cross) = child_known.cross(direction)
    {
        let transferred = main_size_from_cross_aspect(direction, cross, ratio)
            .clamp_optional(None, authored_size.main(direction))
            .clamp_optional(None, resolved_max_size.main(direction));
        min_content = min_content.max(transferred);
    }
    Some(min_content.max(padding_border.sum_axes().main(direction)))
}

fn flex_automatic_minimum_is_zero(overflow: Point<Overflow>) -> bool {
    matches!(overflow.x, Overflow::Hidden | Overflow::Scroll)
        || matches!(overflow.y, Overflow::Hidden | Overflow::Scroll)
}

fn flex_base_known_size(
    size: Size<Option<Scalar>>,
    cross_available: Available,
    style: &NodeInput,
    constants: &Constants,
    margin: Edges,
    margin_is_auto: Edges<bool>,
    align_self: AlignItems,
) -> Size<Option<Scalar>> {
    let direction = constants.direction;
    let mut known = size.with_main(direction, None);
    if align_self == AlignItems::Stretch
        && style.size.cross(direction).is_auto()
        && known.cross(direction).is_none()
        && !margin_is_auto.cross_start(direction, constants.layout_direction)
        && !margin_is_auto.cross_end(direction, constants.layout_direction)
        && let Some(cross) = cross_available.into_option()
    {
        known = known.with_cross(
            direction,
            Some((cross - margin.cross_sum(direction)).max(0.0)),
        );
    }
    known
}

fn requested_axis(direction: FlexDirection) -> RequestedAxis {
    if direction.is_row() {
        RequestedAxis::Horizontal
    } else {
        RequestedAxis::Vertical
    }
}

fn clamp_available(available: Available, min: Option<Scalar>, max: Option<Scalar>) -> Available {
    match available {
        Available::Definite(value) => Available::Definite(value.clamp_optional(min, max)),
        Available::MinContent => min.map_or(Available::MinContent, Available::Definite),
        Available::MaxContent => max.map_or(Available::MaxContent, Available::Definite),
    }
}

fn collect_flex_lines<Node>(items: &[FlexItem<Node>], constants: &Constants) -> Vec<FlexLine>
where
    Node: Copy,
{
    let direction = constants.direction;
    if !constants.wraps {
        return vec![FlexLine::new(0, items.len())];
    }

    let container_main_size = match flex_line_collection_size(constants) {
        Some(size) => size,
        None => match constants.available_main {
            Available::Definite(size) => size,
            Available::MinContent => {
                return (0..items.len())
                    .map(|index| FlexLine::new(index, index + 1))
                    .collect();
            }
            Available::MaxContent => return vec![FlexLine::new(0, items.len())],
        },
    };

    let mut lines = Vec::new();
    let mut start = 0;
    while start < items.len() {
        let mut line_main_size = 0.0;
        let mut end = start;

        while end < items.len() {
            let gap = if end == start {
                0.0
            } else {
                constants.gap.main(direction)
            };
            let next_size = gap
                + items[end].target_size.main(direction)
                + items[end].margin.main_sum(direction);
            if end > start && line_main_size + next_size > container_main_size {
                break;
            }

            line_main_size += next_size;
            end += 1;
        }

        lines.push(FlexLine::new(start, end));
        start = end;
    }

    if lines.is_empty() {
        lines.push(FlexLine::new(0, 0));
    }
    lines
}

fn flex_main_size(constants: &Constants) -> Option<Scalar> {
    constants.node_inner_size.main(constants.direction)
}

fn flex_line_collection_size(constants: &Constants) -> Option<Scalar> {
    constants
        .node_inner_size
        .main(constants.direction)
        .or_else(|| constants.max_inner_size.main(constants.direction))
}

fn resolve_lines<Tree>(
    tree: &mut Tree,
    items: &mut [FlexItem<<Tree as Traverse>::Node>],
    lines: &mut [FlexLine],
    constants: &Constants,
) where
    Tree: Compute,
{
    let direction = constants.direction;
    let cross_gap = constants.gap.cross(direction);
    let mut cross_cursor = 0.0;
    let single_line = !constants.wraps;

    for line in &mut *lines {
        resolve_flexible_lengths(&mut items[line.start..line.end], constants);

        let item_count = line.end - line.start;
        resolve_main_axis_auto_margins(&mut items[line.start..line.end], constants);
        let free_space = line_free_space(&items[line.start..line.end], constants);
        let justify_content = alignment_fallback(free_space, item_count, constants.justify_content);
        let mut main_cursor = alignment_offset(
            free_space,
            item_count,
            constants.gap.main(direction),
            justify_content,
            direction.is_reverse(),
            true,
        );
        let mut cross_size = 0.0;

        let mut item_indices = (line.start..line.end).collect::<Vec<_>>();
        if direction.is_reverse() {
            item_indices.reverse();
        }

        for (index, item_index) in item_indices.into_iter().enumerate() {
            if index > 0 {
                main_cursor += alignment_offset(
                    free_space,
                    item_count,
                    constants.gap.main(direction),
                    justify_content,
                    direction.is_reverse(),
                    false,
                );
            }

            let item = &mut items[item_index];
            determine_hypothetical_cross_size(tree, item, constants);
            item.offset_main = main_cursor + item.margin_main_start(constants);
            item.offset_cross = cross_cursor
                + item
                    .margin
                    .cross_start(direction, constants.layout_direction);

            main_cursor += item.target_size.main(direction) + item.margin.main_sum(direction);
            cross_size = Scalar::max(
                cross_size,
                item.target_size.cross(direction) + item.margin.cross_sum(direction),
            );
        }
        cross_size = Scalar::max(
            cross_size,
            line_cross_size(&items[line.start..line.end], constants),
        );

        line.main_size = main_cursor;
        line.cross_size = if single_line {
            constants
                .node_inner_size
                .cross(direction)
                .unwrap_or(cross_size)
        } else {
            cross_size
        };
        line.offset_cross = cross_cursor;
        align_items_on_cross_axis(
            &mut items[line.start..line.end],
            line.cross_size,
            cross_cursor,
            constants,
        );
        cross_cursor += line.cross_size + cross_gap;
    }

    stretch_lines_on_cross_axis(items, lines, constants);
    align_lines_on_cross_axis(items, lines, constants);
}

fn align_lines_on_cross_axis<Node>(
    items: &mut [FlexItem<Node>],
    lines: &mut [FlexLine],
    constants: &Constants,
) {
    let direction = constants.direction;
    let Some(container_cross_size) = constants.node_inner_size.cross(direction) else {
        return;
    };
    let line_count = lines.len();
    let cross_gap = constants.gap.cross(direction);
    let used_cross_size = lines.iter().map(|line| line.cross_size).sum::<Scalar>()
        + cross_gap * line_count.saturating_sub(1) as Scalar;
    let free_space = container_cross_size - used_cross_size;
    let align_content = alignment_fallback(free_space, line_count, constants.align_content);
    if constants.wrap_reverse {
        align_reversed_lines_on_cross_axis(items, lines, free_space, cross_gap, align_content);
        return;
    }

    let mut cross_cursor = alignment_offset(
        free_space,
        line_count,
        cross_gap,
        align_content,
        false,
        true,
    );
    for (index, line) in lines.iter_mut().enumerate() {
        if index > 0 {
            cross_cursor += alignment_offset(
                free_space,
                line_count,
                cross_gap,
                align_content,
                false,
                false,
            );
        }
        let delta = cross_cursor - line.offset_cross;
        line.offset_cross = cross_cursor;
        for item in &mut items[line.start..line.end] {
            item.offset_cross += delta;
        }
        cross_cursor += line.cross_size;
    }
}

fn determine_hypothetical_cross_size<Tree>(
    tree: &mut Tree,
    item: &mut FlexItem<<Tree as Traverse>::Node>,
    constants: &Constants,
) where
    Tree: Compute,
{
    let direction = constants.direction;
    let padding_border_cross = (item.padding + item.border).sum_axes().cross(direction);
    let authored_cross = item
        .size
        .cross(direction)
        .map(|cross| {
            cross.clamp_optional(
                item.min_size.cross(direction),
                item.max_size.cross(direction),
            )
        })
        .map(|cross| cross.max(padding_border_cross));
    let available_cross = clamp_available(
        constants
            .node_inner_size
            .cross(direction)
            .map(Available::definite)
            .unwrap_or(constants.available.cross(direction)),
        item.min_size.cross(direction),
        item.max_size.cross(direction),
    );
    let available_cross = match available_cross {
        Available::Definite(value) => Available::Definite(value.max(padding_border_cross)),
        other => other,
    };
    let measured_cross = authored_cross.unwrap_or_else(|| {
        let main_size_changed =
            (item.target_size.main(direction) - item.output.size.main(direction)).abs() > 0.001;
        if item.output.content_size == item.output.size && !main_size_changed {
            return item
                .output
                .size
                .cross(direction)
                .clamp_optional(
                    item.min_size.cross(direction),
                    item.max_size.cross(direction),
                )
                .max(padding_border_cross);
        }

        tree.compute_child(
            item.node,
            ComputeInput {
                run_mode: RunMode::ComputeSize,
                sizing_mode: SizingMode::ContentSize,
                axis: if direction.is_row() {
                    RequestedAxis::Vertical
                } else {
                    RequestedAxis::Horizontal
                },
                known: Size::from_main_cross(
                    direction,
                    Some(item.target_size.main(direction)),
                    authored_cross,
                ),
                parent: constants.node_inner_size,
                available: Size::from_main_cross(
                    direction,
                    constants
                        .node_inner_size
                        .main(direction)
                        .map(Available::definite)
                        .unwrap_or(Available::MAX_CONTENT),
                    available_cross,
                ),
            },
        )
        .size
        .cross(direction)
        .clamp_optional(
            item.min_size.cross(direction),
            item.max_size.cross(direction),
        )
        .max(padding_border_cross)
    });

    item.target_size = item.target_size.with_cross(direction, measured_cross);
}

fn stretch_lines_on_cross_axis<Node>(
    items: &mut [FlexItem<Node>],
    lines: &mut [FlexLine],
    constants: &Constants,
) {
    if constants.align_content != AlignContent::Stretch {
        return;
    }

    let direction = constants.direction;
    let Some(container_cross_size) = constants.node_inner_size.cross(direction) else {
        return;
    };
    let cross_gap = constants.gap.cross(direction);
    let used_cross_size = lines.iter().map(|line| line.cross_size).sum::<Scalar>()
        + cross_gap * lines.len().saturating_sub(1) as Scalar;
    if used_cross_size >= container_cross_size || lines.is_empty() {
        return;
    }

    let addition = (container_cross_size - used_cross_size) / lines.len() as Scalar;
    for line in lines {
        line.cross_size += addition;
        align_items_on_cross_axis(
            &mut items[line.start..line.end],
            line.cross_size,
            line.offset_cross,
            constants,
        );
    }
}

fn align_reversed_lines_on_cross_axis<Node>(
    items: &mut [FlexItem<Node>],
    lines: &mut [FlexLine],
    free_space: Scalar,
    cross_gap: Scalar,
    align_content: AlignContent,
) {
    let line_count = lines.len();
    let mut total_cross_offset = 0.0;

    for (reverse_index, line_index) in (0..lines.len()).rev().enumerate() {
        let line_alignment_offset = alignment_offset(
            free_space,
            line_count,
            cross_gap,
            align_content,
            true,
            reverse_index == 0,
        );
        let line = &mut lines[line_index];
        let aligned_cross_offset = total_cross_offset + line_alignment_offset;
        let delta = aligned_cross_offset - line.offset_cross;
        line.offset_cross = aligned_cross_offset;
        for item in &mut items[line.start..line.end] {
            item.offset_cross += delta;
        }
        total_cross_offset += line_alignment_offset + line.cross_size;
    }
}

fn resolve_main_axis_auto_margins<Node>(items: &mut [FlexItem<Node>], constants: &Constants) {
    for item in &mut *items {
        if item.margin_main_start_is_auto(constants) {
            item.set_margin_main_start(constants, 0.0);
        }
        if item.margin_main_end_is_auto(constants) {
            item.set_margin_main_end(constants, 0.0);
        }
    }

    let free_space = line_free_space(items, constants);
    if free_space <= 0.0 {
        return;
    }

    let auto_margin_count = items
        .iter()
        .map(|item| {
            usize::from(item.margin_main_start_is_auto(constants))
                + usize::from(item.margin_main_end_is_auto(constants))
        })
        .sum::<usize>();
    if auto_margin_count == 0 {
        return;
    }

    let margin = free_space / auto_margin_count as Scalar;
    for item in items {
        if item.margin_main_start_is_auto(constants) {
            item.set_margin_main_start(constants, margin);
        }
        if item.margin_main_end_is_auto(constants) {
            item.set_margin_main_end(constants, margin);
        }
    }
}

fn align_items_on_cross_axis<Node>(
    items: &mut [FlexItem<Node>],
    line_cross_size: Scalar,
    line_cross_offset: Scalar,
    constants: &Constants,
) {
    let direction = constants.direction;
    let max_baseline = max_line_baseline(items);
    for item in items {
        resolve_cross_axis_auto_margins(item, line_cross_size, constants);
        let outer_cross_size = item.target_size.cross(direction) + item.margin.cross_sum(direction);
        let free_space = line_cross_size - outer_cross_size;
        if item.align_self == AlignItems::Stretch && item.cross_size_is_auto {
            let stretched_cross_size =
                Scalar::max(0.0, line_cross_size - item.margin.cross_sum(direction));
            let stretched_cross_size = clamp_cross_size(item, stretched_cross_size);
            item.target_size = item.target_size.with_cross(direction, stretched_cross_size);
        }
        let alignment_offset = match item.align_self.safe_fallback(free_space) {
            AlignItems::Start => 0.0,
            AlignItems::End | AlignItems::LastBaseline => free_space,
            AlignItems::FlexStart | AlignItems::Stretch => {
                if constants.wrap_reverse {
                    free_space
                } else {
                    0.0
                }
            }
            AlignItems::Center => free_space / 2.0,
            AlignItems::FlexEnd => {
                if constants.wrap_reverse {
                    0.0
                } else {
                    free_space
                }
            }
            AlignItems::Baseline if direction.is_row() => max_baseline - item.baseline,
            AlignItems::Baseline
                if constants.wraps && constants.layout_direction == Direction::Rtl =>
            {
                free_space
            }
            AlignItems::Baseline => 0.0,
            AlignItems::SafeEnd | AlignItems::SafeFlexEnd | AlignItems::SafeCenter => {
                unreachable!("safe_fallback returns unsafe item alignment")
            }
        };
        item.offset_cross = line_cross_offset
            + item
                .margin
                .cross_start(direction, constants.layout_direction)
            + alignment_offset;
    }
}

fn line_cross_size<Node>(items: &[FlexItem<Node>], constants: &Constants) -> Scalar {
    let max_baseline = max_line_baseline(items);
    items
        .iter()
        .map(|item| line_item_cross_size(item, max_baseline, constants))
        .fold(0.0, Scalar::max)
}

fn line_item_cross_size<Node>(
    item: &FlexItem<Node>,
    max_baseline: Scalar,
    constants: &Constants,
) -> Scalar {
    let direction = constants.direction;
    let outer_cross_size = item.target_size.cross(direction) + item.margin.cross_sum(direction);
    if item.align_self == AlignItems::Baseline
        && direction.is_row()
        && !item
            .margin_is_auto
            .cross_start(direction, constants.layout_direction)
        && !item
            .margin_is_auto
            .cross_end(direction, constants.layout_direction)
    {
        return max_baseline - item.baseline + outer_cross_size;
    }

    outer_cross_size
}

fn max_line_baseline<Node>(items: &[FlexItem<Node>]) -> Scalar {
    items
        .iter()
        .filter(|item| item.align_self == AlignItems::Baseline)
        .map(|item| item.baseline)
        .fold(0.0, Scalar::max)
}

fn first_vertical_baseline<Node>(
    items: &[FlexItem<Node>],
    lines: &[FlexLine],
    constants: &Constants,
) -> Option<Scalar> {
    let line = lines.first()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .find(|item| constants.direction.is_column() || item.align_self == AlignItems::Baseline)
        .or_else(|| line_items.first())?;
    let baseline_cross_offset = constants
        .content_box_inset
        .cross_start(constants.direction, constants.layout_direction)
        + item.offset_cross
        + item.baseline
        - item
            .margin
            .cross_start(constants.direction, constants.layout_direction);
    Some(if constants.direction.is_row() {
        baseline_cross_offset
    } else {
        constants.content_box_inset.main_start(constants.direction)
            + item.offset_main
            + item.baseline
            - item.margin_main_start(constants)
    })
}

fn last_vertical_baseline<Node>(
    items: &[FlexItem<Node>],
    lines: &[FlexLine],
    constants: &Constants,
) -> Option<Scalar> {
    let line = lines.last()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .rev()
        .find(|item| constants.direction.is_column() || item.align_self == AlignItems::Baseline)
        .or_else(|| line_items.last())?;
    let baseline_cross_offset = constants
        .content_box_inset
        .cross_start(constants.direction, constants.layout_direction)
        + item.offset_cross
        + item.baseline
        - item
            .margin
            .cross_start(constants.direction, constants.layout_direction);
    Some(if constants.direction.is_row() {
        baseline_cross_offset
    } else {
        constants.content_box_inset.main_start(constants.direction)
            + item.offset_main
            + item.baseline
            - item.margin_main_start(constants)
    })
}

fn item_scrollbar_size(overflow: Point<Overflow>, scrollbar_width: Scalar) -> Size {
    Size::new(
        if overflow.y == Overflow::Scroll {
            scrollbar_width
        } else {
            0.0
        },
        if overflow.x == Overflow::Scroll {
            scrollbar_width
        } else {
            0.0
        },
    )
}

fn resolve_cross_axis_auto_margins<Node>(
    item: &mut FlexItem<Node>,
    line_cross_size: Scalar,
    constants: &Constants,
) {
    let direction = constants.direction;
    let layout_direction = constants.layout_direction;
    let auto_start = item.margin_is_auto.cross_start(direction, layout_direction);
    let auto_end = item.margin_is_auto.cross_end(direction, layout_direction);
    if !auto_start && !auto_end {
        return;
    }
    if auto_start {
        item.margin
            .set_cross_start(direction, layout_direction, 0.0);
    }
    if auto_end {
        item.margin.set_cross_end(direction, layout_direction, 0.0);
    }

    let free_space =
        line_cross_size - item.target_size.cross(direction) - item.margin.cross_sum(direction);
    if auto_start && auto_end {
        let margin = free_space / 2.0;
        item.margin
            .set_cross_start(direction, layout_direction, margin);
        item.margin
            .set_cross_end(direction, layout_direction, margin);
    } else if auto_start {
        item.margin
            .set_cross_start(direction, layout_direction, free_space);
    } else if auto_end {
        item.margin
            .set_cross_end(direction, layout_direction, free_space);
    }
}

fn line_free_space<Node>(items: &[FlexItem<Node>], constants: &Constants) -> Scalar {
    let direction = constants.direction;
    let Some(container_main_size) = flex_main_size(constants) else {
        return 0.0;
    };
    let used_space = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 {
                0.0
            } else {
                constants.gap.main(direction)
            };
            gap + item.target_size.main(direction) + item.margin.main_sum(direction)
        })
        .sum::<Scalar>();
    container_main_size - used_space
}

fn alignment_fallback(
    free_space: Scalar,
    item_count: usize,
    alignment_mode: AlignContent,
) -> AlignContent {
    let alignment_mode = alignment_mode.safe_fallback(free_space);
    if item_count > 1 && free_space > 0.0 {
        return alignment_mode;
    }

    match alignment_mode {
        AlignContent::Stretch
        | AlignContent::SpaceBetween
        | AlignContent::SpaceAround
        | AlignContent::SpaceEvenly
            if free_space <= 0.0 =>
        {
            AlignContent::FlexStart
        }
        AlignContent::Stretch | AlignContent::SpaceBetween => AlignContent::FlexStart,
        AlignContent::SpaceAround | AlignContent::SpaceEvenly => AlignContent::Center,
        mode => mode,
    }
}

fn alignment_offset(
    free_space: Scalar,
    item_count: usize,
    gap: Scalar,
    alignment_mode: AlignContent,
    layout_is_flex_reversed: bool,
    is_first: bool,
) -> Scalar {
    if is_first {
        match alignment_mode {
            AlignContent::Start => 0.0,
            AlignContent::FlexStart => {
                if layout_is_flex_reversed {
                    free_space
                } else {
                    0.0
                }
            }
            AlignContent::End => free_space,
            AlignContent::FlexEnd => {
                if layout_is_flex_reversed {
                    0.0
                } else {
                    free_space
                }
            }
            AlignContent::Center => free_space / 2.0,
            AlignContent::Stretch | AlignContent::SpaceBetween => 0.0,
            AlignContent::SpaceAround => {
                if free_space >= 0.0 {
                    (free_space / item_count as Scalar) / 2.0
                } else {
                    free_space / 2.0
                }
            }
            AlignContent::SpaceEvenly => {
                if free_space >= 0.0 {
                    free_space / (item_count + 1) as Scalar
                } else {
                    free_space / 2.0
                }
            }
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                unreachable!("safe_fallback returns unsafe content alignment")
            }
        }
    } else {
        let free_space = free_space.max(0.0);
        gap + match alignment_mode {
            AlignContent::SpaceBetween => free_space / (item_count - 1) as Scalar,
            AlignContent::SpaceAround => free_space / item_count as Scalar,
            AlignContent::SpaceEvenly => free_space / (item_count + 1) as Scalar,
            AlignContent::Start
            | AlignContent::FlexStart
            | AlignContent::End
            | AlignContent::FlexEnd
            | AlignContent::Center
            | AlignContent::Stretch => 0.0,
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                unreachable!("safe_fallback returns unsafe content alignment")
            }
        }
    }
}

impl FlexLine {
    fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            main_size: 0.0,
            cross_size: 0.0,
            offset_cross: 0.0,
        }
    }
}

impl<Node> FlexItem<Node> {
    fn margin_main_start(&self, constants: &Constants) -> Scalar {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin.right
        } else {
            self.margin.main_start(constants.direction)
        }
    }

    fn margin_main_start_is_auto(&self, constants: &Constants) -> bool {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin_is_auto.right
        } else {
            self.margin_is_auto.main_start(constants.direction)
        }
    }

    fn margin_main_end_is_auto(&self, constants: &Constants) -> bool {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin_is_auto.left
        } else {
            self.margin_is_auto.main_end(constants.direction)
        }
    }

    fn set_margin_main_start(&mut self, constants: &Constants, value: Scalar) {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin.right = value;
        } else {
            self.margin.set_main_start(constants.direction, value);
        }
    }

    fn set_margin_main_end(&mut self, constants: &Constants, value: Scalar) {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin.left = value;
        } else {
            self.margin.set_main_end(constants.direction, value);
        }
    }

    fn final_main_location(&self, constants: &Constants) -> Scalar {
        let direction = constants.direction;
        if constants.layout_direction == Direction::Rtl && direction.is_row() {
            let container_main = constants
                .node_outer_size
                .unwrap_or(constants.node_inner_size.unwrap_or(Size::ZERO))
                .main(direction);
            return container_main
                - constants.content_box_inset.main_end(direction)
                - self.offset_main
                - self.relative_main_inset(constants)
                - self.output.size.main(direction);
        }

        constants.content_box_inset.main_start(direction)
            + self.offset_main
            + self.relative_main_inset(constants)
    }

    fn relative_main_inset(&self, constants: &Constants) -> Scalar {
        let direction = constants.direction;
        if constants.layout_direction == Direction::Rtl && direction.is_row() {
            return self
                .inset
                .main_end(direction)
                .or_else(|| self.inset.main_start(direction).map(|inset| -inset))
                .unwrap_or(0.0);
        }

        self.inset
            .main_start(direction)
            .or_else(|| self.inset.main_end(direction).map(|inset| -inset))
            .unwrap_or(0.0)
    }

    fn final_cross_location(&self, constants: &Constants) -> Scalar {
        let direction = constants.direction;
        if constants.layout_direction == Direction::Rtl && direction.is_column() {
            let container_cross = constants
                .node_outer_size
                .unwrap_or(constants.node_inner_size.unwrap_or(Size::ZERO))
                .cross(direction);
            return container_cross
                - constants
                    .content_box_inset
                    .cross_start(direction, constants.layout_direction)
                - self.offset_cross
                - self.relative_cross_inset(constants)
                - self.output.size.cross(direction);
        }

        constants
            .content_box_inset
            .cross_start(direction, constants.layout_direction)
            + self.offset_cross
            + self.relative_cross_inset(constants)
    }

    fn relative_cross_inset(&self, constants: &Constants) -> Scalar {
        let direction = constants.direction;
        if constants.layout_direction == Direction::Rtl && direction.is_column() {
            return self
                .inset
                .cross_start(direction, constants.layout_direction)
                .or_else(|| {
                    self.inset
                        .cross_end(direction, constants.layout_direction)
                        .map(|inset| -inset)
                })
                .unwrap_or(0.0);
        }

        self.inset
            .cross_start(direction, constants.layout_direction)
            .or_else(|| {
                self.inset
                    .cross_end(direction, constants.layout_direction)
                    .map(|inset| -inset)
            })
            .unwrap_or(0.0)
    }
}

fn resolve_flexible_lengths<Node>(items: &mut [FlexItem<Node>], constants: &Constants) {
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let free_space = container_main_size - occupied_main_size(items, constants);
    if free_space.abs() < 0.0001 {
        return;
    }
    if free_space > 0.0 {
        distribute_positive_free_space(items, constants);
    } else if free_space < 0.0 {
        distribute_negative_free_space(items, constants);
    }
}

fn distribute_positive_free_space<Node>(items: &mut [FlexItem<Node>], constants: &Constants) {
    let direction = constants.direction;
    let mut frozen = vec![false; items.len()];
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let initial_free_space = container_main_size - flex_used_space(items, constants, &frozen);

    for (item, frozen) in items.iter_mut().zip(&mut frozen) {
        item.target_size = item.target_size.with_main(direction, item.flex_basis);
        if item.flex_grow == 0.0 || item.flex_basis > item.hypothetical_main_size {
            item.target_size = item
                .target_size
                .with_main(direction, item.hypothetical_main_size);
            *frozen = true;
        }
    }

    loop {
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
        let mut free_space = container_main_size - flex_used_space(items, constants, &frozen);
        let grow_sum = items
            .iter()
            .zip(&frozen)
            .filter(|(_, frozen)| !**frozen)
            .map(|(item, _)| item.flex_grow)
            .sum::<Scalar>();
        if grow_sum <= 0.0 {
            return;
        }
        if grow_sum < 1.0 {
            let partial_free_space = initial_free_space * grow_sum;
            if partial_free_space.abs() < free_space.abs() {
                free_space = partial_free_space;
            }
        }

        let mut total_violation = 0.0;
        let mut violations = vec![0.0; items.len()];
        for (index, (item, frozen)) in items.iter_mut().zip(&frozen).enumerate() {
            if *frozen {
                continue;
            }

            let grown_main_size = item.flex_basis + free_space * item.flex_grow / grow_sum;
            let clamped = clamp_main_size(item, direction, grown_main_size);
            item.target_size = item.target_size.with_main(direction, clamped);
            let violation = clamped - grown_main_size;
            violations[index] = violation;
            total_violation += violation;
        }

        freeze_violations(&mut frozen, &violations, total_violation);
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
    }
}

fn distribute_negative_free_space<Node>(items: &mut [FlexItem<Node>], constants: &Constants) {
    let direction = constants.direction;
    let mut frozen = vec![false; items.len()];
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let initial_free_space = container_main_size - flex_used_space(items, constants, &frozen);

    for (item, frozen) in items.iter_mut().zip(&mut frozen) {
        item.target_size = item.target_size.with_main(direction, item.flex_basis);
        if item.flex_shrink == 0.0 || item.flex_basis < item.hypothetical_main_size {
            item.target_size = item
                .target_size
                .with_main(direction, item.hypothetical_main_size);
            *frozen = true;
        }
    }

    loop {
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
        let mut free_space = container_main_size - flex_used_space(items, constants, &frozen);
        let shrink_sum = items
            .iter()
            .zip(&frozen)
            .filter(|(_, frozen)| !**frozen)
            .map(|(item, _)| item.flex_shrink)
            .sum::<Scalar>();
        let scaled_shrink_sum = items
            .iter()
            .zip(&frozen)
            .filter(|(_, frozen)| !**frozen)
            .map(|(item, _)| item.flex_shrink * item.flex_basis)
            .sum::<Scalar>();
        if shrink_sum <= 0.0 || scaled_shrink_sum <= 0.0 {
            return;
        }
        if shrink_sum < 1.0 {
            let partial_free_space = initial_free_space * shrink_sum;
            if partial_free_space.abs() < free_space.abs() {
                free_space = partial_free_space;
            }
        }

        let mut total_violation = 0.0;
        let mut violations = vec![0.0; items.len()];
        for (index, (item, frozen)) in items.iter_mut().zip(&frozen).enumerate() {
            if *frozen {
                continue;
            }

            let scaled_shrink = item.flex_shrink * item.flex_basis;
            let shrunken_main_size =
                item.flex_basis + free_space * scaled_shrink / scaled_shrink_sum;
            let clamped = clamp_main_size(item, direction, Scalar::max(0.0, shrunken_main_size));
            item.target_size = item.target_size.with_main(direction, clamped);
            let violation = clamped - shrunken_main_size;
            violations[index] = violation;
            total_violation += violation;
        }

        freeze_violations(&mut frozen, &violations, total_violation);
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
    }
}

fn flex_used_space<Node>(
    items: &[FlexItem<Node>],
    constants: &Constants,
    frozen: &[bool],
) -> Scalar {
    let direction = constants.direction;
    items
        .iter()
        .zip(frozen)
        .enumerate()
        .map(|(index, (item, frozen))| {
            let gap = if index == 0 {
                0.0
            } else {
                constants.gap.main(direction)
            };
            let main_size = if *frozen {
                item.target_size.main(direction)
            } else {
                item.flex_basis
            };
            gap + main_size + item.margin.main_sum(direction)
        })
        .sum::<Scalar>()
}

fn freeze_violations(frozen: &mut [bool], violations: &[Scalar], total_violation: Scalar) {
    if total_violation == 0.0 {
        for frozen in frozen {
            *frozen = true;
        }
    } else if total_violation > 0.0 {
        for (frozen, violation) in frozen.iter_mut().zip(violations) {
            if *violation > 0.0 {
                *frozen = true;
            }
        }
    } else {
        for (frozen, violation) in frozen.iter_mut().zip(violations) {
            if *violation < 0.0 {
                *frozen = true;
            }
        }
    }
}

fn occupied_main_size<Node>(items: &[FlexItem<Node>], constants: &Constants) -> Scalar {
    let direction = constants.direction;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 {
                0.0
            } else {
                constants.gap.main(direction)
            };
            gap + item.target_size.main(direction) + item.margin.main_sum(direction)
        })
        .sum::<Scalar>()
}

fn clamp_main_size<Node>(item: &FlexItem<Node>, direction: FlexDirection, value: Scalar) -> Scalar {
    clamp_main_size_axes(
        value,
        item.automatic_min_main_size,
        item.min_size.main(direction),
        item.max_size.main(direction),
    )
}

fn clamp_cross_size<Node>(item: &FlexItem<Node>, value: Scalar) -> Scalar {
    value.clamp_optional(item.min_cross_size, item.max_cross_size)
}

fn main_size_from_cross_aspect(
    direction: FlexDirection,
    cross_size: Scalar,
    aspect_ratio: Scalar,
) -> Scalar {
    if direction.is_row() {
        cross_size * aspect_ratio
    } else {
        cross_size / aspect_ratio
    }
}

fn clamp_main_size_axes(
    value: Scalar,
    automatic_min: Option<Scalar>,
    min: Option<Scalar>,
    max: Option<Scalar>,
) -> Scalar {
    let value = max.map_or(value, |max| value.min(max));
    let value = automatic_min.map_or(value, |min| value.max(min));
    min.map_or(value, |min| value.max(min))
}

fn container_output<Node>(
    input: ComputeInput,
    style: &NodeInput,
    constants: &Constants,
    items: &[FlexItem<Node>],
    lines: &[FlexLine],
    absolute_content_size: Size,
) -> ComputeOutput {
    let direction = constants.direction;
    let line_cross_gap = constants.gap.cross(direction) * lines.len().saturating_sub(1) as Scalar;
    let content_main = intrinsic_content_main_size(input, constants, items, lines);
    let content_cross = lines.iter().map(|line| line.cross_size).sum::<Scalar>() + line_cross_gap;
    let content_size = Size::from_main_cross(direction, content_main, content_cross);
    let outer_size = constants
        .node_outer_size
        .unwrap_or(content_size + constants.content_box_inset.sum_axes())
        .clamp_optional(constants.min_outer_size, constants.max_outer_size);
    let mut output_size = input
        .known
        .or(constants.node_outer_size)
        .unwrap_or(outer_size)
        .max_optional(constants.padding_border_size.map(Some));
    if constants.node_outer_size.main(direction).is_none()
        && lines.len() > 1
        && let Available::Definite(available_main) = input.available.main(direction)
    {
        if direction.is_row() {
            output_size.width = output_size.width.max(available_main);
        } else {
            output_size.height = output_size.height.max(available_main);
        }
    }
    let content_size = Size::from_main_cross(style.flex_direction, content_main, content_cross);
    let content_size = if input.run_mode.is_perform_layout() {
        max_size(
            max_size(content_size, visible_content_size(items, constants)),
            absolute_content_size,
        )
    } else {
        content_size
    };
    let first_baseline = first_vertical_baseline(items, lines, constants);
    let last_baseline = last_vertical_baseline(items, lines, constants);

    ComputeOutput::from_sizes_and_baselines(
        output_size,
        content_size,
        Baselines {
            first: Point::new(None, first_baseline),
            last: Point::new(None, last_baseline),
        },
    )
}

fn intrinsic_content_main_size<Node>(
    input: ComputeInput,
    constants: &Constants,
    items: &[FlexItem<Node>],
    lines: &[FlexLine],
) -> Scalar {
    if constants
        .node_outer_size
        .main(constants.direction)
        .is_none()
        && constants.direction.is_row()
        && input.available.main(constants.direction) == Available::MAX_CONTENT
    {
        return lines
            .iter()
            .map(|line| max_content_line_main_size(&items[line.start..line.end], constants))
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0);
    }

    lines
        .iter()
        .map(|line| line.main_size)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0)
}

fn max_content_line_main_size<Node>(items: &[FlexItem<Node>], constants: &Constants) -> Scalar {
    let gap = constants.gap.main(constants.direction);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 { 0.0 } else { gap };
            gap + item.max_content_main_size + item.margin.main_sum(constants.direction)
        })
        .sum()
}

fn resolved_layout_constants<Tree>(
    tree: &mut Tree,
    input: ComputeInput,
    style: &NodeInput,
    constants: &Constants,
    items: &mut [FlexItem<<Tree as Traverse>::Node>],
    lines: &[FlexLine],
) -> Constants
where
    Tree: Compute,
{
    let original_inner_size = constants.node_inner_size;
    let mut constants = *constants;
    determine_container_main_size(tree, input, &mut constants, items, lines);
    constants.max_inner_size = constants.max_inner_size.or(constants.node_inner_size);
    let gap_basis = Size::from_main_cross(
        constants.direction,
        constants.node_inner_size.main(constants.direction),
        original_inner_size
            .cross(constants.direction)
            .and(constants.node_inner_size.cross(constants.direction)),
    );
    let resolver = tree.calc_resolver();
    constants.gap = style.gap.zip_map(gap_basis, |length, basis| {
        resolve_length_or_zero(length, basis, resolver)
    });
    constants
}

fn determine_container_main_size<Tree>(
    tree: &mut Tree,
    input: ComputeInput,
    constants: &mut Constants,
    items: &mut [FlexItem<<Tree as Traverse>::Node>],
    lines: &[FlexLine],
) where
    Tree: Compute,
{
    let direction = constants.direction;
    let Some(outer_main_size) = constants.node_outer_size.main(direction).or_else(|| {
        let content_main = match input.available.main(direction) {
            Available::Definite(available_main) => {
                let longest = lines
                    .iter()
                    .map(|line| flex_basis_line_main_size(&items[line.start..line.end], constants))
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or(0.0);
                if lines.len() > 1 {
                    longest.max(available_main)
                } else {
                    longest
                }
            }
            Available::MinContent if constants.wraps => lines
                .iter()
                .map(|line| flex_basis_line_main_size(&items[line.start..line.end], constants))
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or(0.0),
            Available::MinContent | Available::MaxContent => {
                intrinsic_container_main_size(tree, input, constants, items, lines)
            }
        };
        Some(content_main + constants.content_box_inset.sum_axes().main(direction))
    }) else {
        return;
    };

    let outer_main_size = outer_main_size
        .clamp_optional(
            constants.min_outer_size.main(direction),
            constants.max_outer_size.main(direction),
        )
        .max(
            constants.content_box_inset.sum_axes().main(direction)
                - constants.scrollbar_gutter.main(direction),
        );
    let inner_main_size =
        (outer_main_size - constants.content_box_inset.sum_axes().main(direction)).max(0.0);

    constants.node_outer_size = constants
        .node_outer_size
        .with_main(direction, Some(outer_main_size));
    constants.node_inner_size = constants
        .node_inner_size
        .with_main(direction, Some(inner_main_size));
    constants.available_main = Available::definite(inner_main_size);
}

fn flex_basis_line_main_size<Node>(items: &[FlexItem<Node>], constants: &Constants) -> Scalar {
    let direction = constants.direction;
    let gap = constants.gap.main(direction);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 { 0.0 } else { gap };
            let padding_border = (item.padding + item.border).sum_axes().main(direction);
            let main_size = item
                .min_size
                .main(direction)
                .map_or(item.flex_basis, |min| item.flex_basis.max(min))
                .max(padding_border);
            gap + main_size + item.margin.main_sum(direction)
        })
        .sum()
}

fn intrinsic_container_main_size<Tree>(
    tree: &mut Tree,
    input: ComputeInput,
    constants: &Constants,
    items: &mut [FlexItem<<Tree as Traverse>::Node>],
    lines: &[FlexLine],
) -> Scalar
where
    Tree: Compute,
{
    lines
        .iter()
        .map(|line| {
            let gap = constants.gap.main(constants.direction);
            items[line.start..line.end]
                .iter_mut()
                .enumerate()
                .map(|(index, item)| {
                    let gap = if index == 0 { 0.0 } else { gap };
                    gap + intrinsic_item_main_contribution(tree, input, constants, item)
                })
                .sum::<Scalar>()
        })
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0)
}

fn intrinsic_item_main_contribution<Tree>(
    tree: &mut Tree,
    input: ComputeInput,
    constants: &Constants,
    item: &mut FlexItem<<Tree as Traverse>::Node>,
) -> Scalar
where
    Tree: Compute,
{
    let direction = constants.direction;
    let style_min = item.min_size.main(direction);
    let style_preferred = item.size.main(direction);
    let style_max = item.max_size.main(direction);
    let padding_border = (item.padding + item.border).sum_axes().main(direction);
    let contentful_padding_floor_item = item.flex_basis_is_definite
        && item.flex_basis <= padding_border
        && tree.child_count(item.node) == 0
        && item.output.content_size.main(direction) > item.flex_basis;
    let clamping_basis =
        Some(style_preferred.map_or(item.flex_basis, |preferred| item.flex_basis.max(preferred)));
    let flex_basis_min = clamping_basis.filter(|_| item.flex_shrink == 0.0);
    let flex_basis_max =
        clamping_basis.filter(|_| item.flex_grow == 0.0 && !contentful_padding_floor_item);
    let min_main = max_option(style_min, flex_basis_min)
        .unwrap_or(item.automatic_min_main_size.unwrap_or(0.0))
        .max(item.automatic_min_main_size.unwrap_or(0.0));
    let max_main = style_max
        .and_then(|max| flex_basis_max.map_or(Some(max), |basis| Some(max.min(basis))))
        .or(flex_basis_max)
        .unwrap_or(Scalar::INFINITY);
    if item.flex_basis_is_definite
        && item.flex_grow == 0.0
        && item.flex_basis <= padding_border
        && style_min.is_none()
        && tree.child_count(item.node) == 0
        && item.output.size.main(direction) <= item.flex_basis
        && item.output.content_size.main(direction) <= item.flex_basis
    {
        return item.flex_basis + item.margin.main_sum(direction);
    }

    let cross_available = intrinsic_item_cross_available(input, constants, item);
    let needs_stretched_cross_measure = item.align_self == AlignItems::Stretch
        && item.size.cross(direction).is_none()
        && cross_available.into_option().is_some();

    let contribution = match (style_preferred, max_main <= min_main) {
        _ if flex_automatic_minimum_is_zero(item.overflow) => item.flex_basis.max(min_main),
        (Some(preferred), _) if max_main <= preferred => preferred.min(max_main).max(min_main),
        (_, true) => min_main,
        _ if direction.is_row() && input.available.main(direction) == Available::MinContent => {
            min_main
        }
        _ if !needs_stretched_cross_measure => {
            if direction.is_row() {
                item.max_content_main_size
                    .clamp_optional(style_min, style_max)
            } else {
                item.max_content_main_size
                    .max(item.flex_basis)
                    .clamp_optional(style_min, style_max)
            }
        }
        _ => {
            let child_known = intrinsic_item_known_size(constants, item, cross_available);
            let child_available = input.available.with_cross(direction, cross_available);
            let measured = tree
                .compute_child(
                    item.node,
                    ComputeInput {
                        run_mode: RunMode::ComputeSize,
                        sizing_mode: SizingMode::InherentSize,
                        axis: requested_axis(direction),
                        known: child_known,
                        parent: constants.node_inner_size,
                        available: child_available,
                    },
                )
                .size
                .main(direction);

            if direction.is_row() {
                measured.clamp_optional(style_min, style_max)
            } else {
                measured
                    .max(item.flex_basis)
                    .clamp_optional(style_min, style_max)
            }
        }
    };

    contribution + item.margin.main_sum(direction)
}

fn intrinsic_item_cross_available<Node>(
    input: ComputeInput,
    constants: &Constants,
    item: &FlexItem<Node>,
) -> Available {
    let direction = constants.direction;
    let cross_margin_sum = item.margin.cross_sum(direction);
    let child_min_cross = item
        .min_size
        .cross(direction)
        .map(|value| value + cross_margin_sum);
    let child_max_cross = item
        .max_size
        .cross(direction)
        .map(|value| value + cross_margin_sum);
    let parent_cross = constants.node_inner_size.cross(direction);
    let cross_available = input.available.cross(direction);
    let cross_available = match cross_available {
        Available::Definite(value) => Available::Definite(parent_cross.unwrap_or(value)),
        other => other,
    };
    clamp_available(cross_available, child_min_cross, child_max_cross)
}

fn intrinsic_item_known_size<Node>(
    constants: &Constants,
    item: &FlexItem<Node>,
    cross_available: Available,
) -> Size<Option<Scalar>> {
    let direction = constants.direction;
    let mut known = item.size.with_main(direction, None);
    if item.align_self == AlignItems::Stretch
        && known.cross(direction).is_none()
        && let Some(cross) = cross_available.into_option()
    {
        known = known.with_cross(
            direction,
            Some((cross - item.margin.cross_sum(direction)).max(0.0)),
        );
    }
    known
}

fn resolved_cross_layout_constants(constants: &Constants, lines: &[FlexLine]) -> Constants {
    let direction = constants.direction;
    if constants.node_outer_size.cross(direction).is_some() {
        return *constants;
    }

    let line_cross_gap = constants.gap.cross(direction) * lines.len().saturating_sub(1) as Scalar;
    let content_cross = lines.iter().map(|line| line.cross_size).sum::<Scalar>() + line_cross_gap;
    let cross_inset = constants.content_box_inset.sum_axes().cross(direction);
    let outer_cross_size = (content_cross + cross_inset)
        .clamp_optional(
            constants.min_outer_size.cross(direction),
            constants.max_outer_size.cross(direction),
        )
        .max(cross_inset - constants.scrollbar_gutter.cross(direction))
        .max(constants.padding_border_size.cross(direction));
    let inner_cross_size = (outer_cross_size - cross_inset).max(0.0);

    let mut constants = *constants;
    constants.node_outer_size = constants
        .node_outer_size
        .with_cross(direction, Some(outer_cross_size));
    constants.node_inner_size = constants
        .node_inner_size
        .with_cross(direction, Some(inner_cross_size));
    constants.max_inner_size = constants.max_inner_size.or(constants.node_inner_size);
    constants
}

fn visible_content_size<Node>(items: &[FlexItem<Node>], constants: &Constants) -> Size {
    items.iter().fold(Size::ZERO, |content_size, item| {
        let location = Point::from_main_cross(
            constants.direction,
            item.final_main_location(constants),
            item.final_cross_location(constants),
        );
        let contribution = content_size_contribution(
            Point::new(
                location.x - constants.content_box_inset.left,
                location.y - constants.content_box_inset.top,
            ),
            item.output.size,
            item.output.content_size,
            item.overflow,
        );
        max_size(content_size, contribution)
    })
}

fn max_size(a: Size, b: Size) -> Size {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

fn max_option(a: Option<Scalar>, b: Option<Scalar>) -> Option<Scalar> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
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

fn final_layout<Tree>(
    tree: &mut Tree,
    items: &mut [FlexItem<<Tree as Traverse>::Node>],
    constants: &Constants,
) where
    Tree: Compute,
{
    let direction = constants.direction;
    for item in items {
        let style = tree.node_input(item.node).clone();
        let known = {
            let resolver = tree.calc_resolver();
            final_item_size(item, &style, constants, resolver)
        };
        item.output = tree.compute_child(
            item.node,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known,
                parent: constants.node_inner_size,
                available: Size::new(
                    constants
                        .node_inner_size
                        .width
                        .map(Available::definite)
                        .unwrap_or(Available::MAX_CONTENT),
                    constants
                        .node_inner_size
                        .height
                        .map(Available::definite)
                        .unwrap_or(Available::MAX_CONTENT),
                ),
            },
        );
        let resolved_flex_basis = {
            let resolver = tree.calc_resolver();
            resolve_dimension(
                style.flex_basis,
                constants.node_inner_size.main(direction),
                resolver,
            )
        };
        suppress_padding_floor_flex_basis_content_overflow(
            tree,
            item,
            resolved_flex_basis,
            constants,
        );
        item.baseline = item
            .output
            .baselines()
            .first_or_synthesize_block(item.output.size)
            + item
                .margin
                .cross_start(direction, constants.layout_direction);
        let location = Point::from_main_cross(
            direction,
            item.final_main_location(constants),
            item.final_cross_location(constants),
        );
        tree.set_unrounded(
            item.node,
            NodeOutput {
                order: item.order,
                location,
                size: item.output.size,
                content_size: item.output.content_size,
                scrollbar_size: item_scrollbar_size(item.overflow, item.scrollbar_width),
                border: item.border,
                padding: item.padding,
                margin: item.margin,
            },
        );
    }
}

fn suppress_padding_floor_flex_basis_content_overflow<Node>(
    tree: &impl Traverse<Node = Node>,
    item: &mut FlexItem<Node>,
    resolved_flex_basis: Option<Scalar>,
    constants: &Constants,
) where
    Node: Copy,
{
    let direction = constants.direction;
    let Some(resolved_flex_basis) = resolved_flex_basis else {
        return;
    };
    let padding_border = (item.padding + item.border).sum_axes().main(direction);
    if item.flex_grow == 0.0
        && resolved_flex_basis <= padding_border
        && tree.child_count(item.node) == 0
        && item.output.size.main(direction) <= item.flex_basis
        && item.output.content_size.main(direction) <= item.flex_basis
        && item.target_size.main(direction) <= padding_border
    {
        item.output.content_size = item
            .output
            .content_size
            .with_main(direction, item.target_size.main(direction));
    }
}

fn final_item_size<Node>(
    item: &FlexItem<Node>,
    style: &NodeInput,
    constants: &Constants,
    resolver: &dyn CalcResolver,
) -> Size<Option<Scalar>> {
    let padding = style
        .padding
        .zip_inline_size(constants.node_inner_size, |length, basis| {
            resolve_length_or_zero(length, basis, resolver)
        });
    let border = style
        .border
        .zip_inline_size(constants.node_inner_size, |length, basis| {
            resolve_length_or_zero(length, basis, resolver)
        });
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::ZERO
    };
    let authored = style
        .size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis, resolver)
        })
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);

    let mut known = Size::new(Some(item.target_size.width), Some(item.target_size.height));
    if constants.direction.is_row() {
        if style.size.height.depends_on_basis_with(resolver) {
            known.height = authored.height.or(known.height);
        }
    } else if style.size.width.depends_on_basis_with(resolver) {
        known.width = authored.width.or(known.width);
    }
    known
}

fn layout_absolute_children<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    constants: &Constants,
) -> Size
where
    Tree: Compute,
{
    let children = tree.children(node).collect::<Vec<_>>();
    let mut content_size = Size::ZERO;
    let inset_relative_size = constants
        .node_outer_size
        .sub_optional(constants.border.sum_axes())
        .sub_optional(Size::new(
            constants.scrollbar_gutter.x,
            constants.scrollbar_gutter.y,
        ));
    let available = Size::new(
        constants
            .node_outer_size
            .width
            .map(Available::definite)
            .unwrap_or(Available::MAX_CONTENT),
        constants
            .node_outer_size
            .height
            .map(Available::definite)
            .unwrap_or(Available::MAX_CONTENT),
    );

    for (order, child) in children.into_iter().enumerate() {
        let style = tree.node_input(child).clone();
        if style.position != Position::Absolute || style.display == super::Display::None {
            continue;
        }

        let resolver = tree.calc_resolver();
        let padding = style
            .padding
            .zip_inline_size(inset_relative_size, |length, basis| {
                resolve_length_or_zero(length, basis, resolver)
            });
        let border = style
            .border
            .zip_inline_size(inset_relative_size, |length, basis| {
                resolve_length_or_zero(length, basis, resolver)
            });
        let margin = style
            .margin
            .zip_inline_size(inset_relative_size, |length, basis| {
                resolve_auto_optional(length, basis, resolver)
            });
        let non_auto_margin = margin.map(|value| value.unwrap_or(0.0));
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::ZERO
        };
        let min_size = style
            .min_size
            .zip_map(inset_relative_size, |dimension, basis| {
                resolve_dimension(dimension, basis, resolver)
            })
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);
        let max_size = style
            .max_size
            .zip_map(inset_relative_size, |dimension, basis| {
                resolve_dimension(dimension, basis, resolver)
            })
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);
        let mut known_size = style
            .size
            .zip_map(inset_relative_size, |dimension, basis| {
                resolve_dimension(dimension, basis, resolver)
            })
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);

        let inset = style.inset.zip_size(inset_relative_size, |length, basis| {
            resolve_auto_optional(length, basis, resolver)
        });

        if known_size.width.is_none()
            && let (Some(left), Some(right), Some(container_width)) =
                (inset.left, inset.right, inset_relative_size.width)
        {
            known_size.width =
                Some((container_width - non_auto_margin.horizontal_sum() - left - right).max(0.0));
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_optional(min_size, max_size);
        }
        if known_size.height.is_none()
            && let (Some(top), Some(bottom), Some(container_height)) =
                (inset.top, inset.bottom, inset_relative_size.height)
        {
            known_size.height =
                Some((container_height - non_auto_margin.vertical_sum() - top - bottom).max(0.0));
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_optional(min_size, max_size);
        }
        known_size = known_size
            .clamp_optional(min_size, max_size)
            .max_optional(padding_border.sum_axes().map(Some));

        let output = tree.compute_child(
            child,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: known_size,
                parent: constants.node_inner_size,
                available,
            },
        );
        let final_size = known_size
            .unwrap_or(output.size)
            .clamp_optional(min_size, max_size)
            .max_optional(padding_border.sum_axes().map(Some));
        let margin = resolve_absolute_margins(margin, final_size, constants);
        let location = absolute_location(
            final_size,
            margin,
            inset,
            style.align_self.unwrap_or(constants.align_items),
            constants,
        );

        tree.set_unrounded(
            child,
            NodeOutput {
                order: order as u32,
                location,
                size: final_size,
                content_size: output.content_size,
                scrollbar_size: item_scrollbar_size(style.overflow, style.scrollbar_width),
                border,
                padding,
                margin,
            },
        );
        let contribution = content_size_contribution(
            Point::new(
                location.x - constants.content_box_inset.left,
                location.y - constants.content_box_inset.top,
            ),
            final_size,
            output.content_size,
            style.overflow,
        );
        content_size = Size::new(
            content_size.width.max(contribution.width),
            content_size.height.max(contribution.height),
        );
    }
    content_size
}

fn layout_hidden_children<Tree>(tree: &mut Tree, node: <Tree as Traverse>::Node)
where
    Tree: Compute,
{
    let children = tree.children(node).collect::<Vec<_>>();
    for (order, child) in children.into_iter().enumerate() {
        if tree.node_input(child).display != super::Display::None {
            continue;
        }

        tree.set_unrounded(child, NodeOutput::with_order(order as u32));
        tree.compute_child(child, ComputeInput::HIDDEN);
    }
}

fn resolve_absolute_margins(
    margin: Edges<Option<Scalar>>,
    size: Size,
    constants: &Constants,
) -> Edges {
    let non_auto_margin = margin.map(|value| value.unwrap_or(0.0));
    let free_space = Size::new(
        constants.node_inner_size.width.unwrap_or(0.0)
            - size.width
            - non_auto_margin.horizontal_sum(),
        constants.node_inner_size.height.unwrap_or(0.0)
            - size.height
            - non_auto_margin.vertical_sum(),
    );
    let auto_width = match (
        usize::from(margin.left.is_none()) + usize::from(margin.right.is_none()),
        free_space.width,
    ) {
        (0, _) => 0.0,
        (count, free_space) => free_space.max(0.0) / count as Scalar,
    };
    let auto_height = match (
        usize::from(margin.top.is_none()) + usize::from(margin.bottom.is_none()),
        free_space.height,
    ) {
        (0, _) => 0.0,
        (count, free_space) => free_space.max(0.0) / count as Scalar,
    };

    Edges {
        top: margin.top.unwrap_or(auto_height),
        right: margin.right.unwrap_or(auto_width),
        bottom: margin.bottom.unwrap_or(auto_height),
        left: margin.left.unwrap_or(auto_width),
    }
}

fn absolute_location(
    size: Size,
    margin: Edges,
    inset: Edges<Option<Scalar>>,
    align_self: AlignItems,
    constants: &Constants,
) -> Point {
    let direction = constants.direction;
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::ZERO));
    let main_start = inset.main_start(direction);
    let main_end = inset.main_end(direction);
    let main_is_rtl = direction.is_row() && constants.layout_direction.is_rtl();
    let cross_is_rtl = direction.is_column() && constants.layout_direction.is_rtl();
    let main_start_scrollbar = if main_is_rtl {
        constants.scrollbar_gutter.main(direction)
    } else {
        0.0
    };
    let main_end_scrollbar = if main_is_rtl {
        0.0
    } else {
        constants.scrollbar_gutter.main(direction)
    };
    let cross_start_scrollbar = if cross_is_rtl {
        constants.scrollbar_gutter.cross(direction)
    } else {
        0.0
    };
    let cross_end_scrollbar = if cross_is_rtl {
        0.0
    } else {
        constants.scrollbar_gutter.cross(direction)
    };
    let main = if direction.is_row()
        && constants.layout_direction.is_rtl()
        && main_start.is_some()
        && let Some(end) = main_end
    {
        container.main(direction)
            - constants.border.main_end(direction)
            - main_end_scrollbar
            - size.main(direction)
            - end
            - margin.main_end(direction)
    } else if let Some(start) = main_start {
        constants.border.main_start(direction)
            + main_start_scrollbar
            + start
            + margin.main_start(direction)
    } else if let Some(end) = main_end {
        container.main(direction)
            - constants.border.main_end(direction)
            - main_end_scrollbar
            - size.main(direction)
            - end
            - margin.main_end(direction)
    } else {
        absolute_main_alignment(size, margin, container, constants)
    };
    let (
        cross_start,
        cross_end,
        border_cross_start,
        border_cross_end,
        margin_cross_start,
        margin_cross_end,
    ) = if direction.is_row() {
        (
            inset.top,
            inset.bottom,
            constants.border.top,
            constants.border.bottom,
            margin.top,
            margin.bottom,
        )
    } else {
        (
            inset.left,
            inset.right,
            constants.border.left,
            constants.border.right,
            margin.left,
            margin.right,
        )
    };
    let cross = if let Some(start) = cross_start {
        border_cross_start + cross_start_scrollbar + start + margin_cross_start
    } else if let Some(end) = cross_end {
        container.cross(direction)
            - border_cross_end
            - cross_end_scrollbar
            - size.cross(direction)
            - end
            - margin_cross_end
    } else {
        absolute_cross_alignment(size, margin, container, align_self, constants)
    };

    Point::from_main_cross(direction, main, cross)
}

fn absolute_main_alignment(
    size: Size,
    margin: Edges,
    container: Size,
    constants: &Constants,
) -> Scalar {
    let direction = constants.direction;
    let content_start = constants.content_box_inset.main_start(direction);
    let content_end = constants.content_box_inset.main_end(direction);
    let free_space = container.main(direction) - content_start - content_end - size.main(direction);
    let alignment = constants.justify_content.safe_fallback(free_space);
    let reversed_main =
        direction.is_reverse() ^ (direction.is_row() && constants.layout_direction.is_rtl());
    match alignment {
        AlignContent::Start
        | AlignContent::Stretch
        | AlignContent::SpaceBetween
        | AlignContent::FlexStart
            if !reversed_main =>
        {
            content_start + margin.main_start(direction)
        }
        AlignContent::End | AlignContent::FlexEnd if !reversed_main => {
            container.main(direction)
                - content_end
                - size.main(direction)
                - margin.main_end(direction)
        }
        AlignContent::Start | AlignContent::FlexEnd => content_start + margin.main_start(direction),
        AlignContent::End | AlignContent::FlexStart | AlignContent::Stretch => {
            container.main(direction)
                - content_end
                - size.main(direction)
                - margin.main_end(direction)
        }
        AlignContent::Center | AlignContent::SpaceAround | AlignContent::SpaceEvenly => {
            (container.main(direction) + content_start - content_end - size.main(direction)
                + margin.main_start(direction)
                - margin.main_end(direction))
                / 2.0
        }
        AlignContent::SpaceBetween => content_start + margin.main_start(direction),
        AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
            unreachable!("safe_fallback returns unsafe content alignment")
        }
    }
}

fn absolute_cross_alignment(
    size: Size,
    margin: Edges,
    container: Size,
    align_self: AlignItems,
    constants: &Constants,
) -> Scalar {
    let direction = constants.direction;
    let content_start = constants
        .content_box_inset
        .cross_start(direction, constants.layout_direction);
    let content_end = constants
        .content_box_inset
        .cross_end(direction, constants.layout_direction);
    let free_space =
        container.cross(direction) - content_start - content_end - size.cross(direction);
    let reversed_cross = constants.wrap_reverse;
    let cross_is_rtl_column = direction.is_column() && constants.layout_direction.is_rtl();
    let start_edge = || {
        if cross_is_rtl_column {
            container.cross(direction)
                - content_start
                - size.cross(direction)
                - margin.cross_start(direction, constants.layout_direction)
        } else {
            content_start + margin.cross_start(direction, constants.layout_direction)
        }
    };
    let end_edge = || {
        if cross_is_rtl_column {
            content_end + margin.cross_end(direction, constants.layout_direction)
        } else {
            container.cross(direction)
                - content_end
                - size.cross(direction)
                - margin.cross_end(direction, constants.layout_direction)
        }
    };
    match align_self.safe_fallback(free_space) {
        AlignItems::Start | AlignItems::FlexStart | AlignItems::Stretch | AlignItems::Baseline
            if !reversed_cross =>
        {
            start_edge()
        }
        AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline if !reversed_cross => {
            end_edge()
        }
        AlignItems::Start | AlignItems::FlexEnd => start_edge(),
        AlignItems::End
        | AlignItems::FlexStart
        | AlignItems::Stretch
        | AlignItems::Baseline
        | AlignItems::LastBaseline => end_edge(),
        AlignItems::Center => {
            (container.cross(direction) + content_start - content_end - size.cross(direction)
                + margin.cross_start(direction, constants.layout_direction)
                - margin.cross_end(direction, constants.layout_direction))
                / 2.0
        }
        AlignItems::SafeEnd | AlignItems::SafeFlexEnd | AlignItems::SafeCenter => {
            unreachable!("safe_fallback returns unsafe item alignment")
        }
    }
}

fn resolve_length_or_zero(
    length: Length,
    basis: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> Scalar {
    length.resolve_with(basis, resolver).unwrap_or(0.0)
}

fn resolve_auto_or_zero(
    length: LengthAuto,
    basis: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> Scalar {
    length.resolve_with(basis, resolver).unwrap_or(0.0)
}

fn resolve_auto_optional(
    length: LengthAuto,
    basis: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> Option<Scalar> {
    length.resolve_with(basis, resolver)
}

fn resolve_dimension(
    dimension: Dimension,
    basis: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> Option<Scalar> {
    dimension.resolve_with(basis, resolver)
}

trait PointExt {
    fn from_main_cross(direction: FlexDirection, main: Scalar, cross: Scalar) -> Self;
}

impl PointExt for Point {
    fn from_main_cross(direction: FlexDirection, main: Scalar, cross: Scalar) -> Self {
        if direction.is_row() {
            Self::new(main, cross)
        } else {
            Self::new(cross, main)
        }
    }
}

trait SizeExt<T> {
    fn from_main_cross(direction: FlexDirection, main: T, cross: T) -> Self;
    fn with_main(self, direction: FlexDirection, value: T) -> Self;
    fn with_cross(self, direction: FlexDirection, value: T) -> Self;
}

impl<T> SizeExt<T> for Size<T> {
    fn from_main_cross(direction: FlexDirection, main: T, cross: T) -> Self {
        if direction.is_row() {
            Self::new(main, cross)
        } else {
            Self::new(cross, main)
        }
    }

    fn with_main(self, direction: FlexDirection, value: T) -> Self {
        if direction.is_row() {
            Self::new(value, self.height)
        } else {
            Self::new(self.width, value)
        }
    }

    fn with_cross(self, direction: FlexDirection, value: T) -> Self {
        if direction.is_row() {
            Self::new(self.width, value)
        } else {
            Self::new(value, self.height)
        }
    }
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

trait EdgeAxisExt {
    fn main_start(self, direction: FlexDirection) -> Scalar;
    fn main_end(self, direction: FlexDirection) -> Scalar;
    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> Scalar;
    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> Scalar;
    fn set_main_start(&mut self, direction: FlexDirection, value: Scalar);
    fn set_main_end(&mut self, direction: FlexDirection, value: Scalar);
    fn set_cross_start(
        &mut self,
        direction: FlexDirection,
        layout_direction: Direction,
        value: Scalar,
    );
    fn set_cross_end(
        &mut self,
        direction: FlexDirection,
        layout_direction: Direction,
        value: Scalar,
    );
}

impl EdgeAxisExt for Edges {
    fn main_start(self, direction: FlexDirection) -> Scalar {
        if direction.is_row() {
            self.left
        } else {
            self.top
        }
    }

    fn main_end(self, direction: FlexDirection) -> Scalar {
        if direction.is_row() {
            self.right
        } else {
            self.bottom
        }
    }

    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> Scalar {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.top,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.left,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.right,
        }
    }

    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> Scalar {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.bottom,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.right,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.left,
        }
    }

    fn set_main_start(&mut self, direction: FlexDirection, value: Scalar) {
        if direction.is_row() {
            self.left = value;
        } else {
            self.top = value;
        }
    }

    fn set_main_end(&mut self, direction: FlexDirection, value: Scalar) {
        if direction.is_row() {
            self.right = value;
        } else {
            self.bottom = value;
        }
    }

    fn set_cross_start(
        &mut self,
        direction: FlexDirection,
        layout_direction: Direction,
        value: Scalar,
    ) {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.top = value,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => {
                self.left = value;
            }
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => {
                self.right = value;
            }
        }
    }

    fn set_cross_end(
        &mut self,
        direction: FlexDirection,
        layout_direction: Direction,
        value: Scalar,
    ) {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.bottom = value,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => {
                self.right = value;
            }
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => {
                self.left = value;
            }
        }
    }
}

trait BoolEdgeAxisExt {
    fn main_start(self, direction: FlexDirection) -> bool;
    fn main_end(self, direction: FlexDirection) -> bool;
    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> bool;
    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> bool;
}

trait OptionEdgeAxisExt {
    fn main_start(self, direction: FlexDirection) -> Option<Scalar>;
    fn main_end(self, direction: FlexDirection) -> Option<Scalar>;
    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> Option<Scalar>;
    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> Option<Scalar>;
}

impl OptionEdgeAxisExt for Edges<Option<Scalar>> {
    fn main_start(self, direction: FlexDirection) -> Option<Scalar> {
        if direction.is_row() {
            self.left
        } else {
            self.top
        }
    }

    fn main_end(self, direction: FlexDirection) -> Option<Scalar> {
        if direction.is_row() {
            self.right
        } else {
            self.bottom
        }
    }

    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> Option<Scalar> {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.top,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.left,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.right,
        }
    }

    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> Option<Scalar> {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.bottom,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.right,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.left,
        }
    }
}

impl BoolEdgeAxisExt for Edges<bool> {
    fn main_start(self, direction: FlexDirection) -> bool {
        if direction.is_row() {
            self.left
        } else {
            self.top
        }
    }

    fn main_end(self, direction: FlexDirection) -> bool {
        if direction.is_row() {
            self.right
        } else {
            self.bottom
        }
    }

    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> bool {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.top,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.left,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.right,
        }
    }

    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> bool {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.bottom,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.right,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CalcExpression, CalcTerm, LayoutCalcStore};

    #[test]
    fn flex_percent_dependent_calc_size_requests_definite_cross_rerun() {
        let mut store = LayoutCalcStore::new();
        let height = store.push(CalcExpression::sum([
            CalcTerm::px(10.0),
            CalcTerm::percent(0.50),
        ]));
        let mut child = NodeInput::default();
        child.size.height = Dimension::calc(height);

        assert!(child.size.height.depends_on_basis_with(&store));
    }
}
