use super::{
    AlignContent, AlignItems, AspectRatioOf, AvailableOf, BaselinesOf, BoxSizing, Compute,
    ComputeInputOf, ComputeOutputOf, DimensionOf, Direction, Edges, FlexDirection, LayoutResultOf,
    LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionOf, LengthResolutionStatus, NodeInputOf,
    NodeOutputOf, Overflow, Point, Position, RequestedAxis, RunMode, Size, SizingMode, Traverse,
};
use crate::compute::{EdgesResultExt, SizeResultExt};
use crate::geometry::{FlowAxes, PhysicalAxis};
use crate::output::PhysicalBaseline;
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, scrollbar_size_from_overflow,
};

pub(crate) fn compute_flex<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    compute_flex_inner::<Tree, Tree::Scalar, M>(tree, node, input)
}

fn compute_flex_inner<Tree, S, M>(
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
    if input.run_mode() == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return Ok(ComputeOutputOf::<S>::from_outer_size(Size::new(
            width, height,
        )));
    }

    let mut collected_items = collect_items(tree, node, &constants, input.run_mode())?;
    let mut lines = collect_flex_lines(&collected_items, &constants);

    let mut layout_constants = resolved_layout_constants(
        tree,
        node,
        input,
        &style,
        &constants,
        &mut collected_items,
        &lines,
    )?;
    let mut resolved_items = resolve_lines(tree, &collected_items, &mut lines, &layout_constants)?;
    let cross_layout_constants = resolved_cross_layout_constants(&layout_constants, &lines);
    if cross_layout_constants.node_inner_size != layout_constants.node_inner_size {
        layout_constants = cross_layout_constants;
        resolved_items = resolve_lines(tree, &collected_items, &mut lines, &layout_constants)?;
    } else {
        layout_constants = cross_layout_constants;
    }
    let (absolute_content_size, final_items) = if input.run_mode().is_perform_layout() {
        let final_items = final_layout(tree, &resolved_items, &layout_constants)?;
        let absolute_content_size = layout_absolute_children(tree, node, &layout_constants)?;
        layout_hidden_children(tree, node, layout_constants.flow_axes)?;
        (absolute_content_size, Some(final_items))
    } else {
        (Size::<S>::ZERO, None)
    };

    Ok(container_output(
        input,
        &style,
        &layout_constants,
        &resolved_items,
        final_items.as_deref(),
        &lines,
        absolute_content_size,
    ))
}

#[derive(Clone, Copy)]
struct Constants<S: LayoutScalar> {
    flow_axes: crate::geometry::FlowAxes,
    direction: FlexDirection,
    layout_direction: Direction,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    min_outer_size: Size<Option<S>>,
    max_outer_size: Size<Option<S>>,
    max_inner_size: Size<Option<S>>,
    border: Edges<S>,
    padding_border_size: Size<S>,
    scrollbar_gutter: Point<S>,
    content_box_inset: Edges<S>,
    gap: Size<S>,
    align_items: AlignItems,
    align_content: AlignContent,
    justify_content: AlignContent,
    wraps: bool,
    wrap_reverse: bool,
    available: Size<AvailableOf<S>>,
    available_main: AvailableOf<S>,
}

impl<S: LayoutScalar> Constants<S> {
    fn new<Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &NodeInputOf<S>,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
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
        let scrollbar_reservation = ScrollbarReservationOf::from_overflow(
            style.overflow,
            style.scrollbar_width.get(),
            style.direction,
        );
        let scrollbar_gutter = Point::new(
            scrollbar_reservation.size().width,
            scrollbar_reservation.size().height,
        );
        let content_box_inset =
            content_box_inset_with_scrollbar(padding, border, scrollbar_reservation);
        let padding_border = (padding + border).sum_axes();
        let content_box_inset_size = content_box_inset.sum_axes();
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border
        } else {
            Size::<S>::ZERO
        };

        let (style_size, min_size, max_size) = match input.sizing_mode() {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::InherentSize => {
                let style_size = style
                    .size
                    .zip_map(input.parent(), |dimension, basis| {
                        resolve_dimension(dimension, basis)
                    })
                    .transpose_with_node(tree, node)?
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let min_size = style
                    .min_size
                    .zip_map(input.parent(), |dimension, basis| {
                        resolve_dimension(dimension, basis)
                    })
                    .transpose_with_node(tree, node)?
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let max_size = style
                    .max_size
                    .zip_map(input.parent(), |dimension, basis| {
                        resolve_dimension(dimension, basis)
                    })
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
        let node_outer_size = input
            .known()
            .or(min_max_definite_size.or(style_size.clamp_optional(min_size, max_size)))
            .max_optional(padding_border.map(Some));
        let node_inner_size = node_outer_size
            .sub_optional(content_box_inset_size)
            .max_optional(Size::<S>::ZERO.map(Some));
        let max_inner_size = max_size
            .sub_optional(content_box_inset_size)
            .max_optional(Size::<S>::ZERO.map(Some));
        let gap = style
            .gap
            .zip_map(node_inner_size, |length, basis| {
                resolve_length_or_zero(length, basis)
            })
            .transpose_with_node(tree, node)?;

        Ok(Self {
            flow_axes: crate::geometry::FlowAxes::new(style.writing_mode, style.direction),
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
            available: input.available(),
            available_main: input.available().main(style.flex_direction),
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct FlexItemBaseline<S: LayoutScalar> {
    flow_axes: FlowAxes,
    measured: Option<PhysicalBaseline<S>>,
}

impl<S: LayoutScalar> FlexItemBaseline<S> {
    fn from_output(output: ComputeOutputOf<S>, flow_axes: FlowAxes) -> Self {
        Self {
            flow_axes,
            measured: output.baselines().first_block_baseline(flow_axes),
        }
    }

    fn refresh(&mut self, output: ComputeOutputOf<S>) {
        self.measured = output.baselines().first_block_baseline(self.flow_axes);
    }

    fn physical(self, size: Size<S>) -> PhysicalBaseline<S> {
        self.measured.unwrap_or_else(|| {
            BaselinesOf::NONE.first_or_synthesize_block_baseline(self.flow_axes, size)
        })
    }

    fn axis(self, size: Size<S>) -> PhysicalAxis {
        self.physical(size).axis()
    }

    fn value(self, size: Size<S>, margin: Edges<S>) -> S {
        self.physical(size).coordinate() + self.flow_axes.line_over_edge(margin)
    }

    fn margin_box_size(self, size: Size<S>, margin: Edges<S>) -> S {
        self.flow_axes.block_axis_extent(size)
            + self.flow_axes.line_over_edge(margin)
            + self.flow_axes.line_under_edge(margin)
    }

    fn has_auto_line_margin(self, margin_is_auto: Edges<bool>) -> bool {
        self.flow_axes.line_over_edge(margin_is_auto)
            || self.flow_axes.line_under_edge(margin_is_auto)
    }

    fn translated(self, size: Size<S>, location: Point<S>) -> Point<Option<S>> {
        self.physical(size).translated(location)
    }
}

#[derive(Clone, Copy, Debug)]
struct CollectedFlexItem<Node, S: LayoutScalar> {
    node: Node,
    order: u32,
    size: Size<Option<S>>,
    initial_output: ComputeOutputOf<S>,
    flex_basis: S,
    flex_basis_is_definite: bool,
    hypothetical_main_size: S,
    max_content_main_size: S,
    hypothetical_size: Size<S>,
    cross_size_is_auto: bool,
    automatic_min_main_size: Option<S>,
    min_size: Size<Option<S>>,
    max_size: Size<Option<S>>,
    min_cross_size: Option<S>,
    max_cross_size: Option<S>,
    margin: Edges<S>,
    margin_is_auto: Edges<bool>,
    inset: Edges<Option<S>>,
    padding: Edges<S>,
    border: Edges<S>,
    overflow: Point<Overflow>,
    scrollbar_width_value: S,
    align_self: AlignItems,
    initial_baseline: FlexItemBaseline<S>,
    flex_grow_factor: S,
    flex_shrink_factor: S,
}

#[derive(Clone, Copy, Debug)]
struct ResolvedFlexItem<Node, S: LayoutScalar> {
    node: Node,
    order: u32,
    size: Size<Option<S>>,
    initial_output: ComputeOutputOf<S>,
    flex_basis: S,
    hypothetical_main_size: S,
    max_content_main_size: S,
    target_size: Size<S>,
    cross_size_is_auto: bool,
    automatic_min_main_size: Option<S>,
    min_size: Size<Option<S>>,
    max_size: Size<Option<S>>,
    min_cross_size: Option<S>,
    max_cross_size: Option<S>,
    margin: Edges<S>,
    margin_is_auto: Edges<bool>,
    inset: Edges<Option<S>>,
    padding: Edges<S>,
    border: Edges<S>,
    overflow: Point<Overflow>,
    scrollbar_width_value: S,
    align_self: AlignItems,
    baseline: FlexItemBaseline<S>,
    flex_grow_factor: S,
    flex_shrink_factor: S,
    offset_main: S,
    offset_cross: S,
}

#[derive(Clone, Copy, Debug)]
struct FinalFlexItem<Node, S: LayoutScalar> {
    _node: core::marker::PhantomData<Node>,
    output: ComputeOutputOf<S>,
    overflow: Point<Overflow>,
    align_self: AlignItems,
    baseline: FlexItemBaseline<S>,
    location: Point<S>,
}

#[derive(Clone, Copy, Debug)]
struct FlexLine<S: LayoutScalar> {
    start: usize,
    end: usize,
    main_size: S,
    cross_size: S,
    offset_cross: S,
}

impl<Node, S: LayoutScalar> From<CollectedFlexItem<Node, S>> for ResolvedFlexItem<Node, S> {
    fn from(item: CollectedFlexItem<Node, S>) -> Self {
        Self {
            node: item.node,
            order: item.order,
            size: item.size,
            initial_output: item.initial_output,
            flex_basis: item.flex_basis,
            hypothetical_main_size: item.hypothetical_main_size,
            max_content_main_size: item.max_content_main_size,
            target_size: item.hypothetical_size,
            cross_size_is_auto: item.cross_size_is_auto,
            automatic_min_main_size: item.automatic_min_main_size,
            min_size: item.min_size,
            max_size: item.max_size,
            min_cross_size: item.min_cross_size,
            max_cross_size: item.max_cross_size,
            margin: item.margin,
            margin_is_auto: item.margin_is_auto,
            inset: item.inset,
            padding: item.padding,
            border: item.border,
            overflow: item.overflow,
            scrollbar_width_value: item.scrollbar_width_value,
            align_self: item.align_self,
            baseline: item.initial_baseline,
            flex_grow_factor: item.flex_grow_factor,
            flex_shrink_factor: item.flex_shrink_factor,
            offset_main: S::ZERO,
            offset_cross: S::ZERO,
        }
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex collector preserves node, scalar, and provider error types"
)]
fn collect_items<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    constants: &Constants<Tree::Scalar>,
    run_mode: RunMode,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Vec<CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let children = tree.children(node).collect::<Vec<_>>();
    let mut items = Vec::with_capacity(children.len());
    for (order, child) in children.into_iter().enumerate() {
        let child_style = tree.node_input(child).clone();
        if child_style.position == Position::Absolute || child_style.display == super::Display::None
        {
            continue;
        }

        let child = build_item(tree, child, order as u32, &child_style, constants, run_mode)?;
        items.push(child);
    }
    Ok(items)
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex item builder preserves node, scalar, and provider error types"
)]
fn build_item<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    order: u32,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    run_mode: RunMode,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let padding = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            style.padding,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            style.border,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let margin = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            style.margin,
            constants.node_inner_size,
            resolve_auto_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let margin_is_auto = style.margin.map(LengthAutoOf::is_auto);
    let inset = style
        .inset
        .zip_size(constants.node_inner_size, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let padding_border = padding + border;
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let authored_size = style
        .size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .transpose_with_node(tree, node)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let size = authored_size;
    let resolved_flex_basis = resolve_dimension(
        style.flex_basis,
        constants.node_inner_size.main(constants.direction),
    )
    .map_err(|status| crate::compute::value_resolution_error(node, status))?
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
            resolve_dimension(dimension, basis)
        })
        .transpose_with_node(tree, node)?;
    let raw_max_size = style
        .max_size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .transpose_with_node(tree, node)?;
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
            .map(AvailableOf::definite)
            .or_else(|| constants.max_inner_size.width.map(AvailableOf::definite))
            .unwrap_or(constants.available.width),
        constants
            .node_inner_size
            .height
            .map(AvailableOf::definite)
            .or_else(|| constants.max_inner_size.height.map(AvailableOf::definite))
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
            Some((cross_size - margin.cross_sum(direction)).max(Tree::Scalar::ZERO)),
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
    let flex_basis_floor_may_override_content = padding_border_main > Tree::Scalar::ZERO
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
            if constants.available_main == AvailableOf::MIN_CONTENT {
                AvailableOf::MIN_CONTENT
            } else {
                AvailableOf::MAX_CONTENT
            },
        )
    } else {
        available
    };
    let output = tree.compute_child(
        node,
        ComputeInputOf::for_child(
            run_mode,
            if use_content_sizing_for_base {
                SizingMode::ContentSize
            } else {
                SizingMode::InherentSize
            },
            RequestedAxis::Both,
            child_known,
            available_inner_size,
            constants.flow_axes,
            child_available,
        ),
    )?;
    let automatic_min_main_size = automatic_min_main_size(
        tree,
        node,
        style,
        constants,
        box_sizing_adjustment,
        child_known_for_base,
    )?;
    let flex_basis = if let Some(flex_basis) = resolved_flex_basis.or_else(|| size.main(direction))
    {
        flex_basis
    } else if let Some(ratio) = style.aspect_ratio {
        if let Some(cross) = child_known_for_base.cross(direction) {
            main_size_from_cross_aspect(direction, cross, ratio)
        } else {
            output.size.main(direction)
        }
    } else {
        tree.compute_child(
            node,
            ComputeInputOf::for_child(
                RunMode::ComputeSize,
                SizingMode::ContentSize,
                requested_axis(direction),
                child_known_for_base,
                available_inner_size.with_main(direction, None),
                constants.flow_axes,
                child_available.with_main(direction, AvailableOf::MAX_CONTENT),
            ),
        )?
        .size
        .main(direction)
    };
    let hypothetical_main_size = clamp_main_size_axes(
        flex_basis,
        automatic_min_main_size,
        min_size.main(direction),
        max_size.main(direction),
    );
    let authored_main_size = authored_size.main(direction);
    let flex_basis_uses_padding_floor = resolved_flex_basis.is_some()
        && flex_basis <= padding_border_main
        && style.flex_grow.get() == Tree::Scalar::ZERO
        && (tree.child_count(node) > 0 || output.content_size.main(direction) <= flex_basis);
    let intrinsic_main_size = if flex_basis_uses_padding_floor {
        flex_basis
    } else if style.flex_basis == DimensionOf::Auto && authored_main_size.is_some() {
        authored_main_size.unwrap_or(Tree::Scalar::ZERO)
    } else {
        output
            .content_size
            .main(direction)
            .max(authored_main_size.unwrap_or(Tree::Scalar::ZERO))
    };
    let max_content_main_size = intrinsic_main_size
        .clamp_optional(min_size.main(direction), max_size.main(direction))
        .max(padding_border_main);
    let mut target_size = output.size.with_main(direction, hypothetical_main_size);
    if align_self != AlignItems::Stretch
        && cross_size_is_auto
        && let Some(ratio) = style.aspect_ratio
    {
        let ratio = ratio.get();
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
    let child_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let baseline = FlexItemBaseline::from_output(output, child_flow_axes);

    Ok(CollectedFlexItem {
        node,
        order,
        size: authored_size,
        initial_output: output,
        flex_basis,
        flex_basis_is_definite: resolved_flex_basis.is_some(),
        hypothetical_main_size,
        max_content_main_size,
        hypothetical_size: target_size,
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
        scrollbar_width_value: style.scrollbar_width.get(),
        align_self,
        initial_baseline: baseline,
        flex_grow_factor: style.flex_grow.get(),
        flex_shrink_factor: style.flex_shrink.get(),
    })
}

fn automatic_min_main_size<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    box_sizing_adjustment: Size<Tree::Scalar>,
    child_known: Size<Option<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Option<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let direction = constants.direction;
    if !style.min_size.main(direction).is_auto() || flex_automatic_minimum_is_zero(style.overflow) {
        return Ok(None);
    }
    let authored_size = style
        .size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .transpose_with_node(tree, node)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let min_size = style
        .min_size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .transpose_with_node(tree, node)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let resolved_max_size = style
        .max_size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .transpose_with_node(tree, node)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let padding = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            style.padding,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            style.border,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let padding_border = padding + border;

    let available = Size::from_main_cross(
        direction,
        AvailableOf::MIN_CONTENT,
        clamp_available(
            constants
                .node_inner_size
                .cross(direction)
                .map(AvailableOf::definite)
                .unwrap_or(AvailableOf::MAX_CONTENT),
            min_size.cross(direction),
            resolved_max_size.cross(direction),
        ),
    );
    let output = tree.compute_child(
        node,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::ContentSize,
            requested_axis(direction),
            child_known,
            constants.node_inner_size.with_main(direction, None),
            constants.flow_axes,
            available,
        ),
    )?;

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
    Ok(Some(
        min_content.max(padding_border.sum_axes().main(direction)),
    ))
}

fn flex_automatic_minimum_is_zero(overflow: Point<Overflow>) -> bool {
    matches!(overflow.x, Overflow::Hidden | Overflow::Scroll)
        || matches!(overflow.y, Overflow::Hidden | Overflow::Scroll)
}

fn flex_base_known_size<S: LayoutScalar>(
    size: Size<Option<S>>,
    cross_available: AvailableOf<S>,
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    margin: Edges<S>,
    margin_is_auto: Edges<bool>,
    align_self: AlignItems,
) -> Size<Option<S>> {
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
            Some((cross - margin.cross_sum(direction)).max(S::ZERO)),
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

fn clamp_available<S: LayoutScalar>(
    available: AvailableOf<S>,
    min: Option<S>,
    max: Option<S>,
) -> AvailableOf<S> {
    match available {
        AvailableOf::Definite(value) => AvailableOf::Definite(value.clamp_optional(min, max)),
        AvailableOf::MinContent => min.map_or(AvailableOf::MinContent, AvailableOf::Definite),
        AvailableOf::MaxContent => max.map_or(AvailableOf::MaxContent, AvailableOf::Definite),
    }
}

fn collect_flex_lines<Node, S: LayoutScalar>(
    items: &[CollectedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> Vec<FlexLine<S>>
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
            AvailableOf::Definite(size) => size,
            AvailableOf::MinContent => {
                return (0..items.len())
                    .map(|index| FlexLine::new(index, index + 1))
                    .collect();
            }
            AvailableOf::MaxContent => return vec![FlexLine::new(0, items.len())],
        },
    };

    let mut lines = Vec::new();
    let mut start = 0;
    while start < items.len() {
        let mut line_main_size = S::ZERO;
        let mut end = start;

        while end < items.len() {
            let gap = if end == start {
                S::ZERO
            } else {
                constants.gap.main(direction)
            };
            let next_size = gap
                + items[end].hypothetical_size.main(direction)
                + items[end].margin.main_sum(direction);
            if end > start && line_main_size + next_size > container_main_size {
                break;
            }

            line_main_size = line_main_size + next_size;
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

fn flex_main_size<S: LayoutScalar>(constants: &Constants<S>) -> Option<S> {
    constants.node_inner_size.main(constants.direction)
}

fn flex_line_collection_size<S: LayoutScalar>(constants: &Constants<S>) -> Option<S> {
    constants
        .node_inner_size
        .main(constants.direction)
        .or_else(|| constants.max_inner_size.main(constants.direction))
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex resolver preserves node, scalar, and provider error types"
)]
fn resolve_lines<Tree, M>(
    tree: &mut Tree,
    items: &[CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
    lines: &mut [FlexLine<Tree::Scalar>],
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Vec<ResolvedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let mut resolved_items = items
        .iter()
        .copied()
        .map(ResolvedFlexItem::from)
        .collect::<Vec<_>>();
    let direction = constants.direction;
    let cross_gap = constants.gap.cross(direction);
    let mut cross_cursor = Tree::Scalar::ZERO;
    let single_line = !constants.wraps;

    for line in &mut *lines {
        resolve_flexible_lengths(&mut resolved_items[line.start..line.end], constants);

        let item_count = line.end - line.start;
        resolve_main_axis_auto_margins(&mut resolved_items[line.start..line.end], constants);
        let free_space = line_free_space(&resolved_items[line.start..line.end], constants);
        let justify_content = alignment_fallback(free_space, item_count, constants.justify_content);
        let mut main_cursor = alignment_offset(
            free_space,
            item_count,
            constants.gap.main(direction),
            justify_content,
            direction.is_reverse(),
            true,
        );
        let mut cross_size = Tree::Scalar::ZERO;

        let mut item_indices = (line.start..line.end).collect::<Vec<_>>();
        if direction.is_reverse() {
            item_indices.reverse();
        }

        for (index, item_index) in item_indices.into_iter().enumerate() {
            if index > 0 {
                main_cursor = main_cursor
                    + alignment_offset(
                        free_space,
                        item_count,
                        constants.gap.main(direction),
                        justify_content,
                        direction.is_reverse(),
                        false,
                    );
            }

            let item = &mut resolved_items[item_index];
            determine_hypothetical_cross_size(tree, item, constants)?;
            item.offset_main = main_cursor + item.margin_main_start(constants);
            item.offset_cross = cross_cursor
                + item
                    .margin
                    .cross_start(direction, constants.layout_direction);

            main_cursor =
                main_cursor + item.target_size.main(direction) + item.margin.main_sum(direction);
            cross_size = Tree::Scalar::max(
                cross_size,
                item.target_size.cross(direction) + item.margin.cross_sum(direction),
            );
        }
        cross_size = Tree::Scalar::max(
            cross_size,
            line_cross_size(&resolved_items[line.start..line.end], constants),
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
            &mut resolved_items[line.start..line.end],
            line.cross_size,
            cross_cursor,
            constants,
        );
        cross_cursor = cross_cursor + line.cross_size + cross_gap;
    }

    stretch_lines_on_cross_axis(&mut resolved_items, lines, constants);
    align_lines_on_cross_axis(&mut resolved_items, lines, constants);
    Ok(resolved_items)
}

fn align_lines_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    lines: &mut [FlexLine<S>],
    constants: &Constants<S>,
) {
    let direction = constants.direction;
    let Some(container_cross_size) = constants.node_inner_size.cross(direction) else {
        return;
    };
    let line_count = lines.len();
    let cross_gap = constants.gap.cross(direction);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(line_count.saturating_sub(1));
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
            cross_cursor = cross_cursor
                + alignment_offset(
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
            item.offset_cross = item.offset_cross + delta;
        }
        cross_cursor = cross_cursor + line.cross_size;
    }
}

fn determine_hypothetical_cross_size<Tree, M>(
    tree: &mut Tree,
    item: &mut ResolvedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<M>,
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
            .map(AvailableOf::definite)
            .unwrap_or(constants.available.cross(direction)),
        item.min_size.cross(direction),
        item.max_size.cross(direction),
    );
    let available_cross = match available_cross {
        AvailableOf::Definite(value) => AvailableOf::Definite(value.max(padding_border_cross)),
        other => other,
    };
    let measured_cross = if let Some(authored_cross) = authored_cross {
        authored_cross
    } else {
        let main_size_changed =
            (item.target_size.main(direction) - item.initial_output.size.main(direction)).abs()
                > Tree::Scalar::from_f64(0.001);
        if item.initial_output.content_size == item.initial_output.size && !main_size_changed {
            item.initial_output
                .size
                .cross(direction)
                .clamp_optional(
                    item.min_size.cross(direction),
                    item.max_size.cross(direction),
                )
                .max(padding_border_cross)
        } else {
            let measured = tree.compute_child(
                item.node,
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::ContentSize,
                    if direction.is_row() {
                        RequestedAxis::Vertical
                    } else {
                        RequestedAxis::Horizontal
                    },
                    Size::from_main_cross(
                        direction,
                        Some(item.target_size.main(direction)),
                        authored_cross,
                    ),
                    constants.node_inner_size,
                    constants.flow_axes,
                    Size::from_main_cross(
                        direction,
                        constants
                            .node_inner_size
                            .main(direction)
                            .map(AvailableOf::definite)
                            .unwrap_or(AvailableOf::MAX_CONTENT),
                        available_cross,
                    ),
                ),
            )?;
            item.baseline.refresh(measured);
            measured
                .size
                .cross(direction)
                .clamp_optional(
                    item.min_size.cross(direction),
                    item.max_size.cross(direction),
                )
                .max(padding_border_cross)
        }
    };

    item.target_size = item.target_size.with_cross(direction, measured_cross);
    Ok(())
}

fn stretch_lines_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    lines: &mut [FlexLine<S>],
    constants: &Constants<S>,
) {
    if constants.align_content != AlignContent::Stretch {
        return;
    }

    let direction = constants.direction;
    let Some(container_cross_size) = constants.node_inner_size.cross(direction) else {
        return;
    };
    let cross_gap = constants.gap.cross(direction);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(lines.len().saturating_sub(1));
    if used_cross_size >= container_cross_size || lines.is_empty() {
        return;
    }

    let addition = (container_cross_size - used_cross_size) / S::from_usize(lines.len());
    for line in lines {
        line.cross_size = line.cross_size + addition;
        align_items_on_cross_axis(
            &mut items[line.start..line.end],
            line.cross_size,
            line.offset_cross,
            constants,
        );
    }
}

fn align_reversed_lines_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    lines: &mut [FlexLine<S>],
    free_space: S,
    cross_gap: S,
    align_content: AlignContent,
) {
    let line_count = lines.len();
    let mut total_cross_offset = S::ZERO;

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
            item.offset_cross = item.offset_cross + delta;
        }
        total_cross_offset = total_cross_offset + line_alignment_offset + line.cross_size;
    }
}

fn resolve_main_axis_auto_margins<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) {
    for item in &mut *items {
        if item.margin_main_start_is_auto(constants) {
            item.set_margin_main_start(constants, S::ZERO);
        }
        if item.margin_main_end_is_auto(constants) {
            item.set_margin_main_end(constants, S::ZERO);
        }
    }

    let free_space = line_free_space(items, constants);
    if free_space <= S::ZERO {
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

    let margin = free_space / S::from_usize(auto_margin_count);
    for item in items {
        if item.margin_main_start_is_auto(constants) {
            item.set_margin_main_start(constants, margin);
        }
        if item.margin_main_end_is_auto(constants) {
            item.set_margin_main_end(constants, margin);
        }
    }
}

fn align_items_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    line_cross_size: S,
    line_cross_offset: S,
    constants: &Constants<S>,
) {
    let direction = constants.direction;
    let cross_axis = flex_cross_axis(direction);
    let max_baseline = max_line_baseline(items, cross_axis);
    for item in items {
        resolve_cross_axis_auto_margins(item, line_cross_size, constants);
        let outer_cross_size = item.target_size.cross(direction) + item.margin.cross_sum(direction);
        let free_space = line_cross_size - outer_cross_size;
        if item.align_self == AlignItems::Stretch && item.cross_size_is_auto {
            let stretched_cross_size =
                S::max(S::ZERO, line_cross_size - item.margin.cross_sum(direction));
            let stretched_cross_size = clamp_cross_size(item, stretched_cross_size);
            item.target_size = item.target_size.with_cross(direction, stretched_cross_size);
        }
        let alignment_offset = match item.align_self.safe_fallback(free_space) {
            AlignItems::Start => S::ZERO,
            AlignItems::End | AlignItems::LastBaseline => free_space,
            AlignItems::FlexStart | AlignItems::Stretch => {
                if constants.wrap_reverse {
                    free_space
                } else {
                    S::ZERO
                }
            }
            AlignItems::Center => free_space / S::from_f64(2.0),
            AlignItems::FlexEnd => {
                if constants.wrap_reverse {
                    S::ZERO
                } else {
                    free_space
                }
            }
            AlignItems::Baseline
                if direction.is_row() && item.baseline.axis(item.target_size) == cross_axis =>
            {
                max_baseline - item.baseline.value(item.target_size, item.margin)
            }
            AlignItems::Baseline
                if constants.wraps && constants.layout_direction == Direction::Rtl =>
            {
                free_space
            }
            AlignItems::Baseline => S::ZERO,
            AlignItems::SafeEnd | AlignItems::SafeFlexEnd | AlignItems::SafeCenter => {
                unreachable!("safe_fallback returns unsafe item alignment")
            }
        };
        let line_over_margin = if item.align_self == AlignItems::Baseline
            && item.baseline.axis(item.target_size) == cross_axis
        {
            item.baseline.flow_axes.line_over_edge(item.margin)
        } else {
            item.margin
                .cross_start(direction, constants.layout_direction)
        };
        item.offset_cross = line_cross_offset + line_over_margin + alignment_offset;
    }
}

fn line_cross_size<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let cross_axis = flex_cross_axis(constants.direction);
    let max_baseline = max_line_baseline(items, cross_axis);
    items
        .iter()
        .map(|item| line_item_cross_size(item, max_baseline, constants))
        .fold(S::ZERO, S::max)
}

fn line_item_cross_size<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    max_baseline: S,
    constants: &Constants<S>,
) -> S {
    let direction = constants.direction;
    let outer_cross_size = item.target_size.cross(direction) + item.margin.cross_sum(direction);
    if item.align_self == AlignItems::Baseline
        && direction.is_row()
        && item.baseline.axis(item.target_size) == flex_cross_axis(direction)
        && !item.baseline.has_auto_line_margin(item.margin_is_auto)
    {
        return max_baseline - item.baseline.value(item.target_size, item.margin)
            + item.baseline.margin_box_size(item.target_size, item.margin);
    }

    outer_cross_size
}

fn max_line_baseline<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    cross_axis: PhysicalAxis,
) -> S {
    items
        .iter()
        .filter(|item| {
            item.align_self == AlignItems::Baseline
                && item.baseline.axis(item.target_size) == cross_axis
        })
        .map(|item| item.baseline.value(item.target_size, item.margin))
        .fold(S::ZERO, S::max)
}

fn flex_cross_axis(direction: FlexDirection) -> PhysicalAxis {
    if direction.is_row() {
        PhysicalAxis::Vertical
    } else {
        PhysicalAxis::Horizontal
    }
}

fn first_vertical_baseline<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.first()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .find(|item| constants.direction.is_column() || item.align_self == AlignItems::Baseline)
        .or_else(|| line_items.first())?;
    let location = Point::from_main_cross(
        constants.direction,
        constants.content_box_inset.main_start(constants.direction) + item.offset_main,
        constants
            .content_box_inset
            .cross_start(constants.direction, constants.layout_direction)
            + item.offset_cross,
    );
    Some(item_physical_baseline(
        location,
        item.target_size,
        item.baseline,
    ))
}

fn last_vertical_baseline<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.last()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .rev()
        .find(|item| constants.direction.is_column() || item.align_self == AlignItems::Baseline)
        .or_else(|| line_items.last())?;
    let location = Point::from_main_cross(
        constants.direction,
        constants.content_box_inset.main_start(constants.direction) + item.offset_main,
        constants
            .content_box_inset
            .cross_start(constants.direction, constants.layout_direction)
            + item.offset_cross,
    );
    Some(item_physical_baseline(
        location,
        item.target_size,
        item.baseline,
    ))
}

fn first_final_vertical_baseline<Node, S: LayoutScalar>(
    items: &[FinalFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.first()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .find(|item| constants.direction.is_column() || item.align_self == AlignItems::Baseline)
        .or_else(|| line_items.first())?;
    Some(item_physical_baseline(
        item.location,
        item.output.size,
        item.baseline,
    ))
}

fn last_final_vertical_baseline<Node, S: LayoutScalar>(
    items: &[FinalFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.last()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .rev()
        .find(|item| constants.direction.is_column() || item.align_self == AlignItems::Baseline)
        .or_else(|| line_items.last())?;
    Some(item_physical_baseline(
        item.location,
        item.output.size,
        item.baseline,
    ))
}

fn item_physical_baseline<S: LayoutScalar>(
    location: Point<S>,
    size: Size<S>,
    baseline: FlexItemBaseline<S>,
) -> Point<Option<S>> {
    baseline.translated(size, location)
}

fn item_scrollbar_size<S: LayoutScalar>(
    overflow: Point<Overflow>,
    scrollbar_width_value: S,
) -> Size<S> {
    scrollbar_size_from_overflow(overflow, scrollbar_width_value)
}

fn resolve_cross_axis_auto_margins<Node, S: LayoutScalar>(
    item: &mut ResolvedFlexItem<Node, S>,
    line_cross_size: S,
    constants: &Constants<S>,
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
            .set_cross_start(direction, layout_direction, S::ZERO);
    }
    if auto_end {
        item.margin
            .set_cross_end(direction, layout_direction, S::ZERO);
    }

    let free_space =
        line_cross_size - item.target_size.cross(direction) - item.margin.cross_sum(direction);
    if auto_start && auto_end {
        let margin = free_space / S::from_f64(2.0);
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

fn line_free_space<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let direction = constants.direction;
    let Some(container_main_size) = flex_main_size(constants) else {
        return S::ZERO;
    };
    let used_space = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 {
                S::ZERO
            } else {
                constants.gap.main(direction)
            };
            gap + item.target_size.main(direction) + item.margin.main_sum(direction)
        })
        .fold(S::ZERO, |sum, value| sum + value);
    container_main_size - used_space
}

fn alignment_fallback<S: LayoutScalar>(
    free_space: S,
    item_count: usize,
    alignment_mode: AlignContent,
) -> AlignContent {
    let alignment_mode = alignment_mode.safe_fallback(free_space);
    if item_count > 1 && free_space > S::ZERO {
        return alignment_mode;
    }

    match alignment_mode {
        AlignContent::Stretch
        | AlignContent::SpaceBetween
        | AlignContent::SpaceAround
        | AlignContent::SpaceEvenly
            if free_space <= S::ZERO =>
        {
            AlignContent::FlexStart
        }
        AlignContent::Stretch | AlignContent::SpaceBetween => AlignContent::FlexStart,
        AlignContent::SpaceAround | AlignContent::SpaceEvenly => AlignContent::Center,
        mode => mode,
    }
}

fn alignment_offset<S: LayoutScalar>(
    free_space: S,
    item_count: usize,
    gap: S,
    alignment_mode: AlignContent,
    layout_is_flex_reversed: bool,
    is_first: bool,
) -> S {
    if is_first {
        match alignment_mode {
            AlignContent::Start => S::ZERO,
            AlignContent::FlexStart => {
                if layout_is_flex_reversed {
                    free_space
                } else {
                    S::ZERO
                }
            }
            AlignContent::End => free_space,
            AlignContent::FlexEnd => {
                if layout_is_flex_reversed {
                    S::ZERO
                } else {
                    free_space
                }
            }
            AlignContent::Center => free_space / S::from_f64(2.0),
            AlignContent::Stretch | AlignContent::SpaceBetween => S::ZERO,
            AlignContent::SpaceAround => {
                if free_space >= S::ZERO {
                    (free_space / S::from_usize(item_count)) / S::from_f64(2.0)
                } else {
                    free_space / S::from_f64(2.0)
                }
            }
            AlignContent::SpaceEvenly => {
                if free_space >= S::ZERO {
                    free_space / S::from_usize(item_count + 1)
                } else {
                    free_space / S::from_f64(2.0)
                }
            }
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                unreachable!("safe_fallback returns unsafe content alignment")
            }
        }
    } else {
        let free_space = free_space.max(S::ZERO);
        gap + match alignment_mode {
            AlignContent::SpaceBetween => free_space / S::from_usize(item_count - 1),
            AlignContent::SpaceAround => free_space / S::from_usize(item_count),
            AlignContent::SpaceEvenly => free_space / S::from_usize(item_count + 1),
            AlignContent::Start
            | AlignContent::FlexStart
            | AlignContent::End
            | AlignContent::FlexEnd
            | AlignContent::Center
            | AlignContent::Stretch => S::ZERO,
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                unreachable!("safe_fallback returns unsafe content alignment")
            }
        }
    }
}

impl<S: LayoutScalar> FlexLine<S> {
    fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            main_size: S::ZERO,
            cross_size: S::ZERO,
            offset_cross: S::ZERO,
        }
    }
}

impl<Node, S: LayoutScalar> ResolvedFlexItem<Node, S> {
    fn margin_main_start(&self, constants: &Constants<S>) -> S {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin.right
        } else {
            self.margin.main_start(constants.direction)
        }
    }

    fn margin_main_start_is_auto(&self, constants: &Constants<S>) -> bool {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin_is_auto.right
        } else {
            self.margin_is_auto.main_start(constants.direction)
        }
    }

    fn margin_main_end_is_auto(&self, constants: &Constants<S>) -> bool {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin_is_auto.left
        } else {
            self.margin_is_auto.main_end(constants.direction)
        }
    }

    fn set_margin_main_start(&mut self, constants: &Constants<S>, value: S) {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin.right = value;
        } else {
            self.margin.set_main_start(constants.direction, value);
        }
    }

    fn set_margin_main_end(&mut self, constants: &Constants<S>, value: S) {
        if constants.direction.is_row() && constants.layout_direction == Direction::Rtl {
            self.margin.left = value;
        } else {
            self.margin.set_main_end(constants.direction, value);
        }
    }

    fn final_main_location(&self, constants: &Constants<S>, output_size: Size<S>) -> S {
        let direction = constants.direction;
        if constants.layout_direction == Direction::Rtl && direction.is_row() {
            let container_main = constants
                .node_outer_size
                .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO))
                .main(direction);
            return container_main
                - constants.content_box_inset.main_end(direction)
                - self.offset_main
                - self.relative_main_inset(constants)
                - output_size.main(direction);
        }

        constants.content_box_inset.main_start(direction)
            + self.offset_main
            + self.relative_main_inset(constants)
    }

    fn relative_main_inset(&self, constants: &Constants<S>) -> S {
        let direction = constants.direction;
        if constants.layout_direction == Direction::Rtl && direction.is_row() {
            return self
                .inset
                .main_end(direction)
                .or_else(|| self.inset.main_start(direction).map(|inset| -inset))
                .unwrap_or(S::ZERO);
        }

        self.inset
            .main_start(direction)
            .or_else(|| self.inset.main_end(direction).map(|inset| -inset))
            .unwrap_or(S::ZERO)
    }

    fn final_cross_location(&self, constants: &Constants<S>, output_size: Size<S>) -> S {
        let direction = constants.direction;
        if constants.layout_direction == Direction::Rtl && direction.is_column() {
            let container_cross = constants
                .node_outer_size
                .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO))
                .cross(direction);
            return container_cross
                - constants
                    .content_box_inset
                    .cross_start(direction, constants.layout_direction)
                - self.offset_cross
                - self.relative_cross_inset(constants)
                - output_size.cross(direction);
        }

        constants
            .content_box_inset
            .cross_start(direction, constants.layout_direction)
            + self.offset_cross
            + self.relative_cross_inset(constants)
    }

    fn relative_cross_inset(&self, constants: &Constants<S>) -> S {
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
                .unwrap_or(S::ZERO);
        }

        self.inset
            .cross_start(direction, constants.layout_direction)
            .or_else(|| {
                self.inset
                    .cross_end(direction, constants.layout_direction)
                    .map(|inset| -inset)
            })
            .unwrap_or(S::ZERO)
    }
}

fn resolve_flexible_lengths<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) {
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let free_space = container_main_size - occupied_main_size(items, constants);
    if free_space.abs() < S::from_f64(0.0001) {
        return;
    }
    if free_space > S::ZERO {
        distribute_positive_free_space(items, constants);
    } else if free_space < S::ZERO {
        distribute_negative_free_space(items, constants);
    }
}

fn distribute_positive_free_space<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) {
    let direction = constants.direction;
    let mut frozen = vec![false; items.len()];
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let initial_free_space = container_main_size - flex_used_space(items, constants, &frozen);

    for (item, frozen) in items.iter_mut().zip(&mut frozen) {
        item.target_size = item.target_size.with_main(direction, item.flex_basis);
        if item.flex_grow_factor == S::ZERO || item.flex_basis > item.hypothetical_main_size {
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
            .map(|(item, _)| item.flex_grow_factor)
            .fold(S::ZERO, |sum, value| sum + value);
        if grow_sum <= S::ZERO {
            return;
        }
        if grow_sum < S::ONE {
            let partial_free_space = initial_free_space * grow_sum;
            if partial_free_space.abs() < free_space.abs() {
                free_space = partial_free_space;
            }
        }

        let mut total_violation = S::ZERO;
        let mut violations = vec![S::ZERO; items.len()];
        for (index, (item, frozen)) in items.iter_mut().zip(&frozen).enumerate() {
            if *frozen {
                continue;
            }

            let grown_main_size = item.flex_basis + free_space * item.flex_grow_factor / grow_sum;
            let clamped = clamp_main_size(item, direction, grown_main_size);
            item.target_size = item.target_size.with_main(direction, clamped);
            let violation = clamped - grown_main_size;
            violations[index] = violation;
            total_violation = total_violation + violation;
        }

        freeze_violations(&mut frozen, &violations, total_violation);
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
    }
}

fn distribute_negative_free_space<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) {
    let direction = constants.direction;
    let mut frozen = vec![false; items.len()];
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let initial_free_space = container_main_size - flex_used_space(items, constants, &frozen);

    for (item, frozen) in items.iter_mut().zip(&mut frozen) {
        item.target_size = item.target_size.with_main(direction, item.flex_basis);
        if item.flex_shrink_factor == S::ZERO || item.flex_basis < item.hypothetical_main_size {
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
            .map(|(item, _)| item.flex_shrink_factor)
            .fold(S::ZERO, |sum, value| sum + value);
        let scaled_shrink_sum = items
            .iter()
            .zip(&frozen)
            .filter(|(_, frozen)| !**frozen)
            .map(|(item, _)| item.flex_shrink_factor * item.flex_basis)
            .fold(S::ZERO, |sum, value| sum + value);
        if shrink_sum <= S::ZERO || scaled_shrink_sum <= S::ZERO {
            return;
        }
        if shrink_sum < S::ONE {
            let partial_free_space = initial_free_space * shrink_sum;
            if partial_free_space.abs() < free_space.abs() {
                free_space = partial_free_space;
            }
        }

        let mut total_violation = S::ZERO;
        let mut violations = vec![S::ZERO; items.len()];
        for (index, (item, frozen)) in items.iter_mut().zip(&frozen).enumerate() {
            if *frozen {
                continue;
            }

            let scaled_shrink = item.flex_shrink_factor * item.flex_basis;
            let shrunken_main_size =
                item.flex_basis + free_space * scaled_shrink / scaled_shrink_sum;
            let clamped = clamp_main_size(item, direction, S::max(S::ZERO, shrunken_main_size));
            item.target_size = item.target_size.with_main(direction, clamped);
            let violation = clamped - shrunken_main_size;
            violations[index] = violation;
            total_violation = total_violation + violation;
        }

        freeze_violations(&mut frozen, &violations, total_violation);
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
    }
}

fn flex_used_space<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
    frozen: &[bool],
) -> S {
    let direction = constants.direction;
    items
        .iter()
        .zip(frozen)
        .enumerate()
        .map(|(index, (item, frozen))| {
            let gap = if index == 0 {
                S::ZERO
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
        .fold(S::ZERO, |sum, value| sum + value)
}

fn freeze_violations<S: LayoutScalar>(frozen: &mut [bool], violations: &[S], total_violation: S) {
    if total_violation == S::ZERO {
        for frozen in frozen {
            *frozen = true;
        }
    } else if total_violation > S::ZERO {
        for (frozen, violation) in frozen.iter_mut().zip(violations) {
            if *violation > S::ZERO {
                *frozen = true;
            }
        }
    } else {
        for (frozen, violation) in frozen.iter_mut().zip(violations) {
            if *violation < S::ZERO {
                *frozen = true;
            }
        }
    }
}

fn occupied_main_size<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let direction = constants.direction;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 {
                S::ZERO
            } else {
                constants.gap.main(direction)
            };
            gap + item.target_size.main(direction) + item.margin.main_sum(direction)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

fn clamp_main_size<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    direction: FlexDirection,
    value: S,
) -> S {
    clamp_main_size_axes(
        value,
        item.automatic_min_main_size,
        item.min_size.main(direction),
        item.max_size.main(direction),
    )
}

fn clamp_cross_size<Node, S: LayoutScalar>(item: &ResolvedFlexItem<Node, S>, value: S) -> S {
    value.clamp_optional(item.min_cross_size, item.max_cross_size)
}

fn main_size_from_cross_aspect<S: LayoutScalar>(
    direction: FlexDirection,
    cross_size: S,
    aspect_ratio: AspectRatioOf<S>,
) -> S {
    let aspect_ratio = aspect_ratio.get();
    if direction.is_row() {
        cross_size * aspect_ratio
    } else {
        cross_size / aspect_ratio
    }
}

fn clamp_main_size_axes<S: LayoutScalar>(
    value: S,
    automatic_min: Option<S>,
    min: Option<S>,
    max: Option<S>,
) -> S {
    let value = max.map_or(value, |max| value.min(max));
    let value = automatic_min.map_or(value, |min| value.max(min));
    min.map_or(value, |min| value.max(min))
}

fn container_output<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    resolved_items: &[ResolvedFlexItem<Node, S>],
    final_items: Option<&[FinalFlexItem<Node, S>]>,
    lines: &[FlexLine<S>],
    absolute_content_size: Size<S>,
) -> ComputeOutputOf<S> {
    let direction = constants.direction;
    let line_cross_gap =
        constants.gap.cross(direction) * S::from_usize(lines.len().saturating_sub(1));
    let content_main = intrinsic_content_main_size(input, constants, resolved_items, lines);
    let content_cross = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + line_cross_gap;
    let content_size = Size::from_main_cross(direction, content_main, content_cross);
    let outer_size = constants
        .node_outer_size
        .unwrap_or(content_size + constants.content_box_inset.sum_axes())
        .clamp_optional(constants.min_outer_size, constants.max_outer_size);
    let mut output_size = input
        .known()
        .or(constants.node_outer_size)
        .unwrap_or(outer_size)
        .max_optional(constants.padding_border_size.map(Some));
    if constants.node_outer_size.main(direction).is_none()
        && lines.len() > 1
        && let AvailableOf::Definite(available_main) = input.available().main(direction)
    {
        if direction.is_row() {
            output_size.width = output_size.width.max(available_main);
        } else {
            output_size.height = output_size.height.max(available_main);
        }
    }
    let content_size = Size::from_main_cross(style.flex_direction, content_main, content_cross);
    let content_size = if input.run_mode().is_perform_layout() {
        let final_items = final_items.expect("perform-layout flex output requires final items");
        max_size(
            max_size(content_size, visible_content_size(final_items, constants)),
            absolute_content_size,
        )
    } else {
        content_size
    };
    let first_baseline = final_items.map_or_else(
        || first_vertical_baseline(resolved_items, lines, constants),
        |items| first_final_vertical_baseline(items, lines, constants),
    );
    let last_baseline = final_items.map_or_else(
        || last_vertical_baseline(resolved_items, lines, constants),
        |items| last_final_vertical_baseline(items, lines, constants),
    );

    ComputeOutputOf::from_sizes_and_baselines(
        output_size,
        content_size,
        BaselinesOf {
            first: first_baseline.unwrap_or(Point::NONE),
            last: last_baseline.unwrap_or(Point::NONE),
        },
    )
}

fn intrinsic_content_main_size<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
) -> S {
    if constants
        .node_outer_size
        .main(constants.direction)
        .is_none()
        && constants.direction.is_row()
        && input.available().main(constants.direction) == AvailableOf::MAX_CONTENT
    {
        return lines
            .iter()
            .map(|line| max_content_line_main_size(&items[line.start..line.end], constants))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
            .unwrap_or(S::ZERO);
    }

    lines
        .iter()
        .map(|line| line.main_size)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap_or(S::ZERO)
}

fn max_content_line_main_size<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let gap = constants.gap.main(constants.direction);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 { S::ZERO } else { gap };
            gap + item.max_content_main_size + item.margin.main_sum(constants.direction)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

fn resolved_layout_constants<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    items: &mut [CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
    lines: &[FlexLine<Tree::Scalar>],
) -> LayoutResultOf<<Tree as Traverse>::Node, Constants<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let original_inner_size = constants.node_inner_size;
    let mut constants = *constants;
    determine_container_main_size(tree, input, &mut constants, items, lines)?;
    constants.max_inner_size = constants.max_inner_size.or(constants.node_inner_size);
    let gap_basis = Size::from_main_cross(
        constants.direction,
        constants.node_inner_size.main(constants.direction),
        original_inner_size
            .cross(constants.direction)
            .and(constants.node_inner_size.cross(constants.direction)),
    );
    constants.gap = style
        .gap
        .zip_map(gap_basis, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, node)?;
    Ok(constants)
}

fn determine_container_main_size<Tree, M>(
    tree: &mut Tree,
    input: ComputeInputOf<Tree::Scalar>,
    constants: &mut Constants<Tree::Scalar>,
    items: &mut [CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
    lines: &[FlexLine<Tree::Scalar>],
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let direction = constants.direction;
    let fallback_outer_main_size = if constants.node_outer_size.main(direction).is_none() {
        let content_main = match input.available().main(direction) {
            AvailableOf::Definite(available_main) => {
                let longest = lines
                    .iter()
                    .map(|line| flex_basis_line_main_size(&items[line.start..line.end], constants))
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
                    .unwrap_or(Tree::Scalar::ZERO);
                if lines.len() > 1 {
                    longest.max(available_main)
                } else {
                    longest
                }
            }
            AvailableOf::MinContent if constants.wraps => lines
                .iter()
                .map(|line| flex_basis_line_main_size(&items[line.start..line.end], constants))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
                .unwrap_or(Tree::Scalar::ZERO),
            AvailableOf::MinContent | AvailableOf::MaxContent => {
                intrinsic_container_main_size(tree, input, constants, items, lines)?
            }
        };
        Some(content_main + constants.content_box_inset.sum_axes().main(direction))
    } else {
        None
    };
    let Some(outer_main_size) = constants
        .node_outer_size
        .main(direction)
        .or(fallback_outer_main_size)
    else {
        return Ok(());
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
    let inner_main_size = (outer_main_size
        - constants.content_box_inset.sum_axes().main(direction))
    .max(Tree::Scalar::ZERO);

    constants.node_outer_size = constants
        .node_outer_size
        .with_main(direction, Some(outer_main_size));
    constants.node_inner_size = constants
        .node_inner_size
        .with_main(direction, Some(inner_main_size));
    constants.available_main = AvailableOf::definite(inner_main_size);
    Ok(())
}

fn flex_basis_line_main_size<Node, S: LayoutScalar>(
    items: &[CollectedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let direction = constants.direction;
    let gap = constants.gap.main(direction);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 { S::ZERO } else { gap };
            let padding_border = (item.padding + item.border).sum_axes().main(direction);
            let main_size = item
                .min_size
                .main(direction)
                .map_or(item.flex_basis, |min| item.flex_basis.max(min))
                .max(padding_border);
            gap + main_size + item.margin.main_sum(direction)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

fn intrinsic_container_main_size<Tree, M>(
    tree: &mut Tree,
    input: ComputeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    items: &mut [CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
    lines: &[FlexLine<Tree::Scalar>],
) -> LayoutResultOf<<Tree as Traverse>::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let mut largest = Tree::Scalar::ZERO;
    for line in lines {
        let gap = constants.gap.main(constants.direction);
        let mut sum = Tree::Scalar::ZERO;
        for (index, item) in items[line.start..line.end].iter_mut().enumerate() {
            let gap = if index == 0 { Tree::Scalar::ZERO } else { gap };
            sum = sum + gap + intrinsic_item_main_contribution(tree, input, constants, item)?;
        }
        if sum > largest {
            largest = sum;
        }
    }
    Ok(largest)
}

fn intrinsic_item_main_contribution<Tree, M>(
    tree: &mut Tree,
    input: ComputeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    item: &CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let direction = constants.direction;
    let style_min = item.min_size.main(direction);
    let style_preferred = item.size.main(direction);
    let style_max = item.max_size.main(direction);
    let padding_border = (item.padding + item.border).sum_axes().main(direction);
    let contentful_padding_floor_item = item.flex_basis_is_definite
        && item.flex_basis <= padding_border
        && tree.child_count(item.node) == 0
        && item.initial_output.content_size.main(direction) > item.flex_basis;
    let clamping_basis =
        Some(style_preferred.map_or(item.flex_basis, |preferred| item.flex_basis.max(preferred)));
    let flex_basis_min = clamping_basis.filter(|_| item.flex_shrink_factor == Tree::Scalar::ZERO);
    let flex_basis_max = clamping_basis
        .filter(|_| item.flex_grow_factor == Tree::Scalar::ZERO && !contentful_padding_floor_item);
    let min_main = max_option(style_min, flex_basis_min)
        .unwrap_or(item.automatic_min_main_size.unwrap_or(Tree::Scalar::ZERO))
        .max(item.automatic_min_main_size.unwrap_or(Tree::Scalar::ZERO));
    let max_main = style_max
        .and_then(|max| flex_basis_max.map_or(Some(max), |basis| Some(max.min(basis))))
        .or(flex_basis_max)
        .unwrap_or(Tree::Scalar::INFINITY);
    if item.flex_basis_is_definite
        && item.flex_grow_factor == Tree::Scalar::ZERO
        && item.flex_basis <= padding_border
        && style_min.is_none()
        && tree.child_count(item.node) == 0
        && item.initial_output.size.main(direction) <= item.flex_basis
        && item.initial_output.content_size.main(direction) <= item.flex_basis
    {
        return Ok(item.flex_basis + item.margin.main_sum(direction));
    }

    let cross_available = intrinsic_item_cross_available(input, constants, item);
    let needs_stretched_cross_measure = item.align_self == AlignItems::Stretch
        && item.size.cross(direction).is_none()
        && cross_available.into_option().is_some();

    let contribution = match (style_preferred, max_main <= min_main) {
        _ if flex_automatic_minimum_is_zero(item.overflow) => item.flex_basis.max(min_main),
        (Some(preferred), _) if max_main <= preferred => preferred.min(max_main).max(min_main),
        (_, true) => min_main,
        _ if direction.is_row() && input.available().main(direction) == AvailableOf::MinContent => {
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
            let child_available = input.available().with_cross(direction, cross_available);
            let measured = tree
                .compute_child(
                    item.node,
                    ComputeInputOf::for_child(
                        RunMode::ComputeSize,
                        SizingMode::InherentSize,
                        requested_axis(direction),
                        child_known,
                        constants.node_inner_size,
                        constants.flow_axes,
                        child_available,
                    ),
                )?
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

    Ok(contribution + item.margin.main_sum(direction))
}

fn intrinsic_item_cross_available<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    item: &CollectedFlexItem<Node, S>,
) -> AvailableOf<S> {
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
    let cross_available = input.available().cross(direction);
    let cross_available = match cross_available {
        AvailableOf::Definite(value) => AvailableOf::Definite(parent_cross.unwrap_or(value)),
        other => other,
    };
    clamp_available(cross_available, child_min_cross, child_max_cross)
}

fn intrinsic_item_known_size<Node, S: LayoutScalar>(
    constants: &Constants<S>,
    item: &CollectedFlexItem<Node, S>,
    cross_available: AvailableOf<S>,
) -> Size<Option<S>> {
    let direction = constants.direction;
    let mut known = item.size.with_main(direction, None);
    if item.align_self == AlignItems::Stretch
        && known.cross(direction).is_none()
        && let Some(cross) = cross_available.into_option()
    {
        known = known.with_cross(
            direction,
            Some((cross - item.margin.cross_sum(direction)).max(S::ZERO)),
        );
    }
    known
}

fn resolved_cross_layout_constants<S: LayoutScalar>(
    constants: &Constants<S>,
    lines: &[FlexLine<S>],
) -> Constants<S> {
    let direction = constants.direction;
    if constants.node_outer_size.cross(direction).is_some() {
        return *constants;
    }

    let line_cross_gap =
        constants.gap.cross(direction) * S::from_usize(lines.len().saturating_sub(1));
    let content_cross = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + line_cross_gap;
    let cross_inset = constants.content_box_inset.sum_axes().cross(direction);
    let outer_cross_size = (content_cross + cross_inset)
        .clamp_optional(
            constants.min_outer_size.cross(direction),
            constants.max_outer_size.cross(direction),
        )
        .max(cross_inset - constants.scrollbar_gutter.cross(direction))
        .max(constants.padding_border_size.cross(direction));
    let inner_cross_size = (outer_cross_size - cross_inset).max(S::ZERO);

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

fn visible_content_size<Node, S: LayoutScalar>(
    items: &[FinalFlexItem<Node, S>],
    constants: &Constants<S>,
) -> Size<S> {
    items.iter().fold(Size::<S>::ZERO, |content_size, item| {
        let contribution = content_size_contribution(
            Point::new(
                item.location.x - constants.content_box_inset.left,
                item.location.y - constants.content_box_inset.top,
            ),
            item.output.size,
            item.output.content_size,
            item.overflow,
        );
        max_size(content_size, contribution)
    })
}

fn max_size<S: LayoutScalar>(a: Size<S>, b: Size<S>) -> Size<S> {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

fn max_option<S: LayoutScalar>(a: Option<S>, b: Option<S>) -> Option<S> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
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
        return Size::<S>::ZERO;
    }

    let max_x = (location.x + contribution_size.width).max(S::ZERO);
    let min_x = location.x.min(S::ZERO);
    let max_y = (location.y + contribution_size.height).max(S::ZERO);
    let min_y = location.y.min(S::ZERO);
    Size::new(max_x - min_x, max_y - min_y)
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex finalizer preserves node, scalar, and provider error types"
)]
fn final_layout<Tree, M>(
    tree: &mut Tree,
    items: &[ResolvedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Vec<FinalFlexItem<<Tree as Traverse>::Node, Tree::Scalar>>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<M>,
{
    let direction = constants.direction;
    let mut final_items = Vec::with_capacity(items.len());
    for item in items {
        let style = tree.node_input(item.node).clone();
        let known = final_item_size::<Tree, M>(tree, item, &style, constants)?;
        let mut output = tree.compute_child(
            item.node,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                known,
                constants.node_inner_size,
                constants.flow_axes,
                Size::new(
                    constants
                        .node_inner_size
                        .width
                        .map(AvailableOf::definite)
                        .unwrap_or(AvailableOf::MAX_CONTENT),
                    constants
                        .node_inner_size
                        .height
                        .map(AvailableOf::definite)
                        .unwrap_or(AvailableOf::MAX_CONTENT),
                ),
            ),
        )?;
        let resolved_flex_basis =
            resolve_dimension(style.flex_basis, constants.node_inner_size.main(direction))
                .map_err(|status| crate::compute::value_resolution_error(item.node, status))?;
        suppress_padding_floor_flex_basis_content_overflow(
            tree,
            item,
            &mut output,
            resolved_flex_basis,
            constants,
        );
        let child_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
        let baseline = FlexItemBaseline::from_output(output, child_flow_axes);
        let location = Point::from_main_cross(
            direction,
            item.final_main_location(constants, output.size),
            item.final_cross_location(constants, output.size),
        );
        tree.set_unrounded(
            item.node,
            NodeOutputOf {
                order: item.order,
                location,
                size: output.size,
                content_size: output.content_size,
                scroll_geometry: None,
                scrollbar_size: item_scrollbar_size(item.overflow, item.scrollbar_width_value),
                border: item.border,
                padding: item.padding,
                margin: item.margin,
            },
        );
        final_items.push(FinalFlexItem {
            _node: core::marker::PhantomData,
            output,
            overflow: item.overflow,
            align_self: item.align_self,
            baseline,
            location,
        });
    }
    Ok(final_items)
}

fn suppress_padding_floor_flex_basis_content_overflow<Node, S: LayoutScalar>(
    tree: &impl Traverse<Node = Node>,
    item: &ResolvedFlexItem<Node, S>,
    output: &mut ComputeOutputOf<S>,
    resolved_flex_basis: Option<S>,
    constants: &Constants<S>,
) where
    Node: Copy,
{
    let direction = constants.direction;
    let Some(resolved_flex_basis) = resolved_flex_basis else {
        return;
    };
    let padding_border = (item.padding + item.border).sum_axes().main(direction);
    if item.flex_grow_factor == S::ZERO
        && resolved_flex_basis <= padding_border
        && tree.child_count(item.node) == 0
        && output.size.main(direction) <= item.flex_basis
        && output.content_size.main(direction) <= item.flex_basis
        && item.target_size.main(direction) <= padding_border
    {
        output.content_size = output
            .content_size
            .with_main(direction, item.target_size.main(direction));
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex size helper preserves the session's generic error envelope"
)]
fn final_item_size<Tree, M>(
    tree: &Tree,
    item: &ResolvedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Option<Tree::Scalar>>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let padding = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            style.padding,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, item.node)?;
    let border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            style.border,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, item.node)?;
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::<Tree::Scalar>::ZERO
    };
    let authored = style
        .size
        .zip_map(constants.node_inner_size, |dimension, basis| {
            resolve_dimension(dimension, basis)
        })
        .transpose_with_node(tree, item.node)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);

    let mut known = Size::new(Some(item.target_size.width), Some(item.target_size.height));
    if constants.direction.is_row() {
        if style.size.height.depends_on_basis() {
            known.height = authored.height.or(known.height);
        }
    } else if style.size.width.depends_on_basis() {
        known.width = authored.width.or(known.width);
    }
    Ok(known)
}

fn layout_absolute_children<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let children = tree.children(node).collect::<Vec<_>>();
    let mut content_size: Size<Tree::Scalar> = Size::ZERO;
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
            .map(AvailableOf::definite)
            .unwrap_or(AvailableOf::MAX_CONTENT),
        constants
            .node_outer_size
            .height
            .map(AvailableOf::definite)
            .unwrap_or(AvailableOf::MAX_CONTENT),
    );

    for (order, child) in children.into_iter().enumerate() {
        let style = tree.node_input(child).clone();
        if style.position != Position::Absolute || style.display == super::Display::None {
            continue;
        }
        let padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.padding,
                inset_relative_size,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, child)?;
        let border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.border,
                inset_relative_size,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, child)?;
        let margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.margin,
                inset_relative_size,
                resolve_auto_optional,
            )
            .transpose_with_node(tree, child)?;
        let non_auto_margin = margin.map(|value| value.unwrap_or(Tree::Scalar::ZERO));
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::<Tree::Scalar>::ZERO
        };
        let min_size = style
            .min_size
            .zip_map(inset_relative_size, |dimension, basis| {
                resolve_dimension(dimension, basis)
            })
            .transpose_with_node(tree, child)?
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);
        let max_size = style
            .max_size
            .zip_map(inset_relative_size, |dimension, basis| {
                resolve_dimension(dimension, basis)
            })
            .transpose_with_node(tree, child)?
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);
        let mut known_size = style
            .size
            .zip_map(inset_relative_size, |dimension, basis| {
                resolve_dimension(dimension, basis)
            })
            .transpose_with_node(tree, child)?
            .apply_aspect_ratio(style.aspect_ratio)
            .add_optional(box_sizing_adjustment);

        let inset = style
            .inset
            .zip_size(inset_relative_size, |length, basis| {
                resolve_auto_optional(length, basis)
            })
            .transpose_with_node(tree, child)?;

        if known_size.width.is_none()
            && let (Some(left), Some(right), Some(container_width)) =
                (inset.left, inset.right, inset_relative_size.width)
        {
            known_size.width = Some(
                (container_width - non_auto_margin.horizontal_sum() - left - right)
                    .max(Tree::Scalar::ZERO),
            );
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_optional(min_size, max_size);
        }
        if known_size.height.is_none()
            && let (Some(top), Some(bottom), Some(container_height)) =
                (inset.top, inset.bottom, inset_relative_size.height)
        {
            known_size.height = Some(
                (container_height - non_auto_margin.vertical_sum() - top - bottom)
                    .max(Tree::Scalar::ZERO),
            );
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_optional(min_size, max_size);
        }
        known_size = known_size
            .clamp_optional(min_size, max_size)
            .max_optional(padding_border.sum_axes().map(Some));

        let output = tree.compute_child(
            child,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                known_size,
                constants.node_inner_size,
                constants.flow_axes,
                available,
            ),
        )?;
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
            NodeOutputOf {
                order: order as u32,
                location,
                size: final_size,
                content_size: output.content_size,
                scroll_geometry: None,
                scrollbar_size: item_scrollbar_size(style.overflow, style.scrollbar_width.get()),
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
    Ok(content_size)
}

fn layout_hidden_children<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    containing_flow_axes: crate::geometry::FlowAxes,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let children = tree.children(node).collect::<Vec<_>>();
    for (order, child) in children.into_iter().enumerate() {
        if tree.node_input(child).display != super::Display::None {
            continue;
        }

        tree.set_unrounded(child, NodeOutputOf::with_order(order as u32));
        tree.compute_child(child, ComputeInputOf::hidden(containing_flow_axes))?;
    }
    Ok(())
}

fn resolve_absolute_margins<S: LayoutScalar>(
    margin: Edges<Option<S>>,
    size: Size<S>,
    constants: &Constants<S>,
) -> Edges<S> {
    let non_auto_margin = margin.map(|value| value.unwrap_or(S::ZERO));
    let free_space = Size::new(
        constants.node_inner_size.width.unwrap_or(S::ZERO)
            - size.width
            - non_auto_margin.horizontal_sum(),
        constants.node_inner_size.height.unwrap_or(S::ZERO)
            - size.height
            - non_auto_margin.vertical_sum(),
    );
    let auto_width = match (
        usize::from(margin.left.is_none()) + usize::from(margin.right.is_none()),
        free_space.width,
    ) {
        (0, _) => S::ZERO,
        (count, free_space) => free_space.max(S::ZERO) / S::from_usize(count),
    };
    let auto_height = match (
        usize::from(margin.top.is_none()) + usize::from(margin.bottom.is_none()),
        free_space.height,
    ) {
        (0, _) => S::ZERO,
        (count, free_space) => free_space.max(S::ZERO) / S::from_usize(count),
    };

    Edges {
        top: margin.top.unwrap_or(auto_height),
        right: margin.right.unwrap_or(auto_width),
        bottom: margin.bottom.unwrap_or(auto_height),
        left: margin.left.unwrap_or(auto_width),
    }
}

fn absolute_location<S: LayoutScalar>(
    size: Size<S>,
    margin: Edges<S>,
    inset: Edges<Option<S>>,
    align_self: AlignItems,
    constants: &Constants<S>,
) -> Point<S> {
    let direction = constants.direction;
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    let main_start = inset.main_start(direction);
    let main_end = inset.main_end(direction);
    let main_is_rtl = direction.is_row() && constants.layout_direction.is_rtl();
    let cross_is_rtl = direction.is_column() && constants.layout_direction.is_rtl();
    let main_start_scrollbar = if main_is_rtl {
        constants.scrollbar_gutter.main(direction)
    } else {
        S::ZERO
    };
    let main_end_scrollbar = if main_is_rtl {
        S::ZERO
    } else {
        constants.scrollbar_gutter.main(direction)
    };
    let cross_start_scrollbar = if cross_is_rtl {
        constants.scrollbar_gutter.cross(direction)
    } else {
        S::ZERO
    };
    let cross_end_scrollbar = if cross_is_rtl {
        S::ZERO
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

fn absolute_main_alignment<S: LayoutScalar>(
    size: Size<S>,
    margin: Edges<S>,
    container: Size<S>,
    constants: &Constants<S>,
) -> S {
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
                / S::from_f64(2.0)
        }
        AlignContent::SpaceBetween => content_start + margin.main_start(direction),
        AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
            unreachable!("safe_fallback returns unsafe content alignment")
        }
    }
}

fn absolute_cross_alignment<S: LayoutScalar>(
    size: Size<S>,
    margin: Edges<S>,
    container: Size<S>,
    align_self: AlignItems,
    constants: &Constants<S>,
) -> S {
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
                / S::from_f64(2.0)
        }
        AlignItems::SafeEnd | AlignItems::SafeFlexEnd | AlignItems::SafeCenter => {
            unreachable!("safe_fallback returns unsafe item alignment")
        }
    }
}

fn resolve_length_or_zero<S: LayoutScalar>(
    length: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolve_auto_or_zero<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolve_auto_optional<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    resolution_optional(length.resolve_with_status(basis))
}

fn resolve_dimension<S: LayoutScalar>(
    dimension: DimensionOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    resolution_optional(dimension.resolve_with_status(basis))
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

trait PointExt<S: LayoutScalar> {
    fn from_main_cross(direction: FlexDirection, main: S, cross: S) -> Self;
}

impl<S: LayoutScalar> PointExt<S> for Point<S> {
    fn from_main_cross(direction: FlexDirection, main: S, cross: S) -> Self {
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
    type Scalar: LayoutScalar;
    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size<Self::Scalar>) -> Size<Self::Scalar>;
    fn add_optional(self, amount: Size<Self::Scalar>) -> Self;
    fn sub_optional(self, amount: Size<Self::Scalar>) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<Self::Scalar>>) -> Self;
    fn clamp_optional(self, min: Self, max: Self) -> Self;
    fn max_optional(self, min: Self) -> Self;
}

impl<S: LayoutScalar> SizeOptionExt for Size<Option<S>> {
    type Scalar = S;

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

trait SizeConcreteExt {
    type Scalar: LayoutScalar;
    fn clamp_optional(
        self,
        min: Size<Option<Self::Scalar>>,
        max: Size<Option<Self::Scalar>>,
    ) -> Self;
    fn max_optional(self, min: Size<Option<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> SizeConcreteExt for Size<S> {
    type Scalar = S;

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

trait EdgeAxisExt {
    type Scalar: LayoutScalar;
    fn main_start(self, direction: FlexDirection) -> Self::Scalar;
    fn main_end(self, direction: FlexDirection) -> Self::Scalar;
    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> Self::Scalar;
    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> Self::Scalar;
    fn set_main_start(&mut self, direction: FlexDirection, value: Self::Scalar);
    fn set_main_end(&mut self, direction: FlexDirection, value: Self::Scalar);
    fn set_cross_start(
        &mut self,
        direction: FlexDirection,
        layout_direction: Direction,
        value: Self::Scalar,
    );
    fn set_cross_end(
        &mut self,
        direction: FlexDirection,
        layout_direction: Direction,
        value: Self::Scalar,
    );
}

impl<S: LayoutScalar> EdgeAxisExt for Edges<S> {
    type Scalar = S;

    fn main_start(self, direction: FlexDirection) -> S {
        if direction.is_row() {
            self.left
        } else {
            self.top
        }
    }

    fn main_end(self, direction: FlexDirection) -> S {
        if direction.is_row() {
            self.right
        } else {
            self.bottom
        }
    }

    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> S {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.top,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.left,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.right,
        }
    }

    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> S {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.bottom,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.right,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.left,
        }
    }

    fn set_main_start(&mut self, direction: FlexDirection, value: S) {
        if direction.is_row() {
            self.left = value;
        } else {
            self.top = value;
        }
    }

    fn set_main_end(&mut self, direction: FlexDirection, value: S) {
        if direction.is_row() {
            self.right = value;
        } else {
            self.bottom = value;
        }
    }

    fn set_cross_start(&mut self, direction: FlexDirection, layout_direction: Direction, value: S) {
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

    fn set_cross_end(&mut self, direction: FlexDirection, layout_direction: Direction, value: S) {
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
    type Scalar: LayoutScalar;
    fn main_start(self, direction: FlexDirection) -> Option<Self::Scalar>;
    fn main_end(self, direction: FlexDirection) -> Option<Self::Scalar>;
    fn cross_start(
        self,
        direction: FlexDirection,
        layout_direction: Direction,
    ) -> Option<Self::Scalar>;
    fn cross_end(
        self,
        direction: FlexDirection,
        layout_direction: Direction,
    ) -> Option<Self::Scalar>;
}

impl<S: LayoutScalar> OptionEdgeAxisExt for Edges<Option<S>> {
    type Scalar = S;

    fn main_start(self, direction: FlexDirection) -> Option<S> {
        if direction.is_row() {
            self.left
        } else {
            self.top
        }
    }

    fn main_end(self, direction: FlexDirection) -> Option<S> {
        if direction.is_row() {
            self.right
        } else {
            self.bottom
        }
    }

    fn cross_start(self, direction: FlexDirection, layout_direction: Direction) -> Option<S> {
        match (direction, layout_direction) {
            (FlexDirection::Row | FlexDirection::RowReverse, _) => self.top,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Ltr) => self.left,
            (FlexDirection::Column | FlexDirection::ColumnReverse, Direction::Rtl) => self.right,
        }
    }

    fn cross_end(self, direction: FlexDirection, layout_direction: Direction) -> Option<S> {
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
