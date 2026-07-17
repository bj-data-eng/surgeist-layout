use super::{
    AlignContent, AlignItems, AspectRatioOf, AvailableOf, BaselinesOf, BoxSizing, Compute,
    ComputeInputOf, ComputeOutputOf, ComputedOverflow, ContainingLayoutContext, Direction, Edges,
    FlexDirection, FlexWrap, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf,
    LayoutInternalInvariant, LayoutOperation, LayoutResultOf, LayoutScalar, LengthAutoOf, LengthOf,
    LengthResolutionOf, LengthResolutionStatus, NodeInputOf, NodeOutputOf, ParentFormattingContext,
    Point, Position, RequestedAxis, RunMode, Size, SizingMode, Traverse,
};
use crate::compute::{
    EdgesResultExt, ResolvedFlexBasis, SizeResultExt, SizingAlgorithm, resolve_flex_basis,
    resolve_maximum_optional, resolve_minimum_optional, resolve_preferred_optional,
    sizing_resolution_error,
};
use crate::geometry::{FlowAxes, LogicalAxis, PhysicalAxis, PhysicalProgression, PhysicalSide};
use crate::node_input::item_order_permutation;
use crate::output::PhysicalBaseline;
use crate::scroll::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, CanonicalScrollGeometryErrorOf,
    CanonicalScrollGeometrySourceOf, ClipMarginSourceOf, OptimalRegionInsetOf,
    OptimalRegionInsetsOf, ScrollContributionAccumulatorOf, ScrollOriginAxes,
    ScrollOriginProgression, canonical_scroll_box_from_source,
    canonical_scroll_geometry_from_source, rebuild_canonical_scroll_geometry_for_border_box,
};
use crate::sizing::{MaxSizeOf, MinSizeOf, PreferredSizeOf};

fn resolve_preferred_size<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    value: &Size<PreferredSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
) -> LayoutResultOf<Node, Size<Option<S>>, S, M> {
    Ok(Size::new(
        resolve_preferred_optional(
            &value.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(node, error))?,
        resolve_preferred_optional(
            &value.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(node, error))?,
    ))
}

fn resolve_minimum_size<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    value: &Size<MinSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
) -> LayoutResultOf<Node, Size<Option<S>>, S, M> {
    Ok(Size::new(
        resolve_minimum_optional(
            &value.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(node, error))?,
        resolve_minimum_optional(
            &value.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(node, error))?,
    ))
}

fn resolve_maximum_size<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    value: &Size<MaxSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
) -> LayoutResultOf<Node, Size<Option<S>>, S, M> {
    Ok(Size::new(
        resolve_maximum_optional(
            &value.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(node, error))?,
        resolve_maximum_optional(
            &value.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(node, error))?,
    ))
}

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

    let collected_items = collect_items(tree, node, &constants, input.run_mode())?;
    let permutation = item_order_permutation(
        &collected_items
            .iter()
            .map(|item| {
                (
                    tree.node_input(item.node).item_order,
                    crate::SourceIndex::new(item.source_index),
                )
            })
            .collect::<Vec<_>>(),
    );
    let mut items_by_source = collected_items
        .into_iter()
        .map(|item| (crate::SourceIndex::new(item.source_index), item))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut collected_items = permutation
        .into_iter()
        .map(|source_index| {
            items_by_source
                .remove(&source_index)
                .expect("the flex order permutation contains every collected source index")
        })
        .collect::<Vec<_>>();
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
    let container_sizes = container_sizes(input, &layout_constants, &resolved_items, &lines);
    let final_scroll_box = if input.run_mode().is_perform_layout() {
        Some(flex_container_scroll_box::<_, S, M>(
            node,
            input.run_mode(),
            &style,
            &layout_constants,
            container_sizes.output,
        )?)
    } else {
        None
    };
    let (absolute_contributions, final_items) = if input.run_mode().is_perform_layout() {
        let final_items = final_layout(tree, node, &resolved_items, &layout_constants)?;
        let absolute_contributions = layout_absolute_children(
            tree,
            node,
            &layout_constants,
            final_scroll_box.expect("performed flex layout derives its final canonical box"),
        )?;
        layout_hidden_children(tree, node, layout_constants.axes.flow_axes())?;
        (absolute_contributions, Some(final_items))
    } else {
        (Vec::new(), None)
    };

    let mut final_geometry_and_content_size = None;
    if input.run_mode().is_perform_layout() {
        let final_items = final_items
            .as_deref()
            .expect("performed flex layout retains its final in-flow items");
        let scroll_box =
            final_scroll_box.expect("performed flex layout retains its final canonical box");
        let mut contributions = flex_scroll_contributions(
            final_items,
            &absolute_contributions,
            &resolved_items,
            &lines,
            &layout_constants,
            scroll_box,
        )
        .map_err(|error| flex_own_geometry_error(node, input.run_mode(), error))?;
        contributions.exclude_reserved_gutter_from_range();
        let scroll_geometry = flex_container_scroll_geometry::<_, S, M>(
            node,
            input.run_mode(),
            &style,
            &layout_constants,
            scroll_box,
            contributions,
        )?;
        let content_size = contributions
            .content_size_from_anchor(scroll_geometry.content_box().origin())
            .map_err(|error| flex_own_geometry_error(node, input.run_mode(), error))?;
        final_geometry_and_content_size = Some((scroll_geometry, content_size));
    }
    let mut output = container_output(
        input,
        &layout_constants,
        &resolved_items,
        final_items.as_deref(),
        &lines,
        final_geometry_and_content_size.map(|(_, content_size)| content_size),
        container_sizes,
    );
    if let Some((scroll_geometry, _)) = final_geometry_and_content_size {
        output.scroll_geometry = Some(scroll_geometry);
    }
    Ok(output)
}

#[derive(Clone, Copy)]
struct Constants<S: LayoutScalar> {
    flow_axes: crate::geometry::FlowAxes,
    axes: FlexAxes,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    min_outer_size: Size<Option<S>>,
    max_outer_size: Size<Option<S>>,
    max_inner_size: Size<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
    effective_border: Edges<S>,
    padding_border_size: Size<S>,
    scrollbar_gutter: Edges<S>,
    content_box_inset: Edges<S>,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
    gap: Size<S>,
    align_items: AlignItems,
    authored_align_content: Option<AlignContent>,
    align_content: AlignContent,
    authored_justify_content: Option<AlignContent>,
    justify_content: AlignContent,
    wraps: bool,
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
        let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
        let axes = FlexAxes::new(flow_axes, style.flex_direction, style.flex_wrap);
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
        let padding_border = (padding + border).sum_axes();
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border
        } else {
            Size::<S>::ZERO
        };

        let (style_size, min_size, max_size) = match input.sizing_mode() {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::InherentSize => {
                let style_size = resolve_preferred_size::<_, _, M>(
                    node,
                    &style.size,
                    input.parent(),
                    SizingAlgorithm::Flex,
                )?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let min_size = resolve_minimum_size::<_, _, M>(
                    node,
                    &style.min_size,
                    input.parent(),
                    SizingAlgorithm::Flex,
                )?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let max_size = resolve_maximum_size::<_, _, M>(
                    node,
                    &style.max_size,
                    input.parent(),
                    SizingAlgorithm::Flex,
                )?
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
        let unconstrained_scroll_box_size =
            padding_border + Size::splat(style.scrollbar_width.get() + style.scrollbar_width.get());
        let scroll_box_size = node_outer_size
            .or(input.available().map(AvailableOf::into_option))
            .or(max_size)
            .unwrap_or(unconstrained_scroll_box_size)
            .zip_map(padding_border, |size, minimum| size.max(minimum));
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
        .map_err(|error| flex_own_geometry_error(node, input.run_mode(), error))?;
        let effective_border = scroll_box.effective_border();
        let scrollbar_gutter = scroll_box.effective_gutter();
        let content_box_inset =
            effective_border + scrollbar_gutter + scroll_box.effective_padding();
        let content_box_inset_size = content_box_inset.sum_axes();
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
            flow_axes,
            axes,
            node_outer_size,
            node_inner_size,
            min_outer_size: min_size,
            max_outer_size: max_size,
            max_inner_size,
            border,
            padding,
            effective_border,
            padding_border_size: padding_border,
            scrollbar_gutter,
            content_box_inset,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            gap,
            align_items: style.align_items.unwrap_or(AlignItems::Stretch),
            authored_align_content: style.align_content,
            align_content: style.align_content.unwrap_or(AlignContent::Stretch),
            authored_justify_content: style.justify_content,
            justify_content: style.justify_content.unwrap_or(AlignContent::FlexStart),
            wraps: matches!(
                style.flex_wrap,
                super::FlexWrap::Wrap | super::FlexWrap::WrapReverse
            ),
            available: input.available(),
            available_main: axes.main_size(input.available()),
        })
    }

    fn with_final_scroll_box(mut self, scroll_box: CanonicalScrollBoxOf<S>) -> Self {
        self.node_outer_size = scroll_box.border_box().size().map(Some);
        self.node_inner_size = scroll_box.content_box().size().map(Some);
        self.effective_border = scroll_box.effective_border();
        self.scrollbar_gutter = scroll_box.effective_gutter();
        self.content_box_inset = scroll_box.content_box_inset();
        self
    }
}

/// Resolved flex main/cross roles for one container.
///
/// This is the sole flex owner for translating a container's logical flow into
/// physical axes, sides, and progressions. It is derived only from the
/// container's `FlowAxes`, `FlexDirection`, and `FlexWrap`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FlexAxes {
    flow_axes: FlowAxes,
    main_logical_axis: LogicalAxis,
    cross_logical_axis: LogicalAxis,
    main_physical_axis: PhysicalAxis,
    cross_physical_axis: PhysicalAxis,
    main_start_side: PhysicalSide,
    main_end_side: PhysicalSide,
    cross_start_side: PhysicalSide,
    cross_end_side: PhysicalSide,
    main_reversed: bool,
    cross_reversed: bool,
    main_progression: PhysicalProgression,
    cross_progression: PhysicalProgression,
}

impl FlexAxes {
    #[must_use]
    pub(crate) const fn new(
        flow_axes: FlowAxes,
        flex_direction: FlexDirection,
        flex_wrap: FlexWrap,
    ) -> Self {
        let main_reversed = flex_direction.is_reverse();
        let cross_reversed = matches!(flex_wrap, FlexWrap::WrapReverse);
        let (
            main_logical_axis,
            cross_logical_axis,
            main_physical_axis,
            cross_physical_axis,
            normal_main_start_side,
            normal_main_end_side,
            normal_cross_start_side,
            normal_cross_end_side,
        ) = if flex_direction.is_row() {
            (
                LogicalAxis::Inline,
                LogicalAxis::Block,
                flow_axes.inline_axis(),
                flow_axes.block_axis(),
                flow_axes.inline_start(),
                flow_axes.inline_end(),
                flow_axes.block_start(),
                flow_axes.block_end(),
            )
        } else {
            (
                LogicalAxis::Block,
                LogicalAxis::Inline,
                flow_axes.block_axis(),
                flow_axes.inline_axis(),
                flow_axes.block_start(),
                flow_axes.block_end(),
                flow_axes.inline_start(),
                flow_axes.inline_end(),
            )
        };
        let (main_start_side, main_end_side) = if main_reversed {
            (normal_main_end_side, normal_main_start_side)
        } else {
            (normal_main_start_side, normal_main_end_side)
        };
        let (cross_start_side, cross_end_side) = if cross_reversed {
            (normal_cross_end_side, normal_cross_start_side)
        } else {
            (normal_cross_start_side, normal_cross_end_side)
        };

        Self {
            flow_axes,
            main_logical_axis,
            cross_logical_axis,
            main_physical_axis,
            cross_physical_axis,
            main_start_side,
            main_end_side,
            cross_start_side,
            cross_end_side,
            main_reversed,
            cross_reversed,
            main_progression: Self::reverse_progression(
                flow_axes.physical_axis_progression(main_physical_axis),
                main_reversed,
            ),
            cross_progression: Self::reverse_progression(
                flow_axes.physical_axis_progression(cross_physical_axis),
                cross_reversed,
            ),
        }
    }

    #[must_use]
    pub(crate) const fn flow_axes(self) -> FlowAxes {
        self.flow_axes
    }

    #[must_use]
    pub(crate) const fn flow_direction(self) -> Direction {
        self.flow_axes.direction()
    }

    #[must_use]
    pub(crate) const fn scroll_origin_axes(self) -> ScrollOriginAxes {
        let main = if self.main_reversed {
            ScrollOriginProgression::FlowStartward
        } else {
            ScrollOriginProgression::FlowEndward
        };
        let cross = if self.cross_reversed {
            ScrollOriginProgression::FlowStartward
        } else {
            ScrollOriginProgression::FlowEndward
        };
        match self.main_logical_axis {
            LogicalAxis::Inline => ScrollOriginAxes::new(main, cross),
            LogicalAxis::Block => ScrollOriginAxes::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) const fn main_logical_axis(self) -> LogicalAxis {
        self.main_logical_axis
    }

    #[must_use]
    pub(crate) const fn cross_logical_axis(self) -> LogicalAxis {
        self.cross_logical_axis
    }

    #[must_use]
    pub(crate) const fn main_physical_axis(self) -> PhysicalAxis {
        self.main_physical_axis
    }

    #[must_use]
    pub(crate) const fn cross_physical_axis(self) -> PhysicalAxis {
        self.cross_physical_axis
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_start_side(self) -> PhysicalSide {
        self.main_start_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_end_side(self) -> PhysicalSide {
        self.main_end_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_start_side(self) -> PhysicalSide {
        self.cross_start_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_end_side(self) -> PhysicalSide {
        self.cross_end_side
    }

    #[must_use]
    pub(crate) const fn main_is_reversed(self) -> bool {
        self.main_reversed
    }

    #[must_use]
    pub(crate) const fn cross_is_reversed(self) -> bool {
        self.cross_reversed
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_progression(self) -> PhysicalProgression {
        self.main_progression
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_progression(self) -> PhysicalProgression {
        self.cross_progression
    }

    #[must_use]
    pub(crate) fn main_size<T>(self, size: Size<T>) -> T {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        }
    }

    #[must_use]
    pub(crate) fn cross_size<T>(self, size: Size<T>) -> T {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        }
    }

    #[must_use]
    pub(crate) fn size_from_main_cross<T>(self, main: T, cross: T) -> Size<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Size::new(main, cross),
            PhysicalAxis::Vertical => Size::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) fn with_main_size<T>(self, size: Size<T>, value: T) -> Size<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Size::new(value, size.height),
            PhysicalAxis::Vertical => Size::new(size.width, value),
        }
    }

    #[must_use]
    pub(crate) fn with_cross_size<T>(self, size: Size<T>, value: T) -> Size<T> {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => Size::new(value, size.height),
            PhysicalAxis::Vertical => Size::new(size.width, value),
        }
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn main_point<T>(self, point: Point<T>) -> T {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => point.x,
            PhysicalAxis::Vertical => point.y,
        }
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn cross_point<T>(self, point: Point<T>) -> T {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => point.x,
            PhysicalAxis::Vertical => point.y,
        }
    }

    #[must_use]
    pub(crate) fn point_from_main_cross<T>(self, main: T, cross: T) -> Point<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Point::new(main, cross),
            PhysicalAxis::Vertical => Point::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) fn main_start_edge<T>(self, edges: Edges<T>) -> T {
        self.edge_at_side(edges, self.main_start_side)
    }

    #[must_use]
    pub(crate) fn main_end_edge<T>(self, edges: Edges<T>) -> T {
        self.edge_at_side(edges, self.main_end_side)
    }

    #[must_use]
    pub(crate) fn cross_start_edge<T>(self, edges: Edges<T>) -> T {
        self.edge_at_side(edges, self.cross_start_side)
    }

    #[must_use]
    pub(crate) fn cross_end_edge<T>(self, edges: Edges<T>) -> T {
        self.edge_at_side(edges, self.cross_end_side)
    }

    #[must_use]
    pub(crate) fn normal_main_start_edge<T>(self, edges: Edges<T>) -> T {
        self.edge_at_side(edges, self.normal_axis_start_side(self.main_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_main_end_edge<T>(self, edges: Edges<T>) -> T {
        self.edge_at_side(edges, self.normal_axis_end_side(self.main_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_cross_start_edge<T>(self, edges: Edges<T>) -> T {
        self.edge_at_side(edges, self.normal_axis_start_side(self.cross_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_cross_end_edge<T>(self, edges: Edges<T>) -> T {
        self.edge_at_side(edges, self.normal_axis_end_side(self.cross_logical_axis))
    }

    #[must_use]
    pub(crate) fn main_edge_sum<S: LayoutScalar>(self, edges: Edges<S>) -> S {
        self.main_start_edge(edges) + self.main_end_edge(edges)
    }

    #[must_use]
    pub(crate) fn cross_edge_sum<S: LayoutScalar>(self, edges: Edges<S>) -> S {
        self.cross_start_edge(edges) + self.cross_end_edge(edges)
    }

    pub(crate) fn set_main_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.main_start_side, value);
    }

    pub(crate) fn set_main_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.main_end_side, value);
    }

    pub(crate) fn set_cross_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.cross_start_side, value);
    }

    pub(crate) fn set_cross_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.cross_end_side, value);
    }

    #[must_use]
    pub(crate) const fn main_requested_axis(self) -> RequestedAxis {
        Self::requested_axis_for(self.main_physical_axis)
    }

    #[must_use]
    pub(crate) const fn cross_requested_axis(self) -> RequestedAxis {
        Self::requested_axis_for(self.cross_physical_axis)
    }

    #[must_use]
    pub(crate) fn main_size_from_cross_aspect<S: LayoutScalar>(
        self,
        cross: S,
        aspect_ratio: AspectRatioOf<S>,
    ) -> S {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => cross * aspect_ratio.get(),
            PhysicalAxis::Vertical => cross / aspect_ratio.get(),
        }
    }

    #[must_use]
    pub(crate) fn main_position_from_start<S: LayoutScalar>(
        self,
        container: Size<S>,
        start_inset: S,
        offset: S,
        item_size: S,
        relative_offset: S,
    ) -> S {
        let coordinate = start_inset + offset + relative_offset;
        match self.main_progression {
            PhysicalProgression::Increasing => coordinate,
            PhysicalProgression::Decreasing => self.main_size(container) - coordinate - item_size,
        }
    }

    #[must_use]
    pub(crate) fn cross_position_from_start<S: LayoutScalar>(
        self,
        container: Size<S>,
        start_inset: S,
        offset: S,
        item_size: S,
        relative_offset: S,
    ) -> S {
        let coordinate = start_inset + offset + relative_offset;
        match self.cross_progression {
            PhysicalProgression::Increasing => coordinate,
            PhysicalProgression::Decreasing => self.cross_size(container) - coordinate - item_size,
        }
    }

    #[must_use]
    pub(crate) fn main_offset_from_normal_flow<S: LayoutScalar>(self, offset: S) -> S {
        if self.main_reversed { -offset } else { offset }
    }

    #[must_use]
    pub(crate) fn cross_offset_from_normal_flow<S: LayoutScalar>(self, offset: S) -> S {
        if self.cross_reversed { -offset } else { offset }
    }

    #[must_use]
    pub(crate) fn main_position_from_normal_start<S: LayoutScalar>(
        self,
        container: Size<S>,
        offset: S,
        item_size: S,
    ) -> S {
        let flex_offset = if self.main_reversed {
            self.main_size(container) - offset - item_size
        } else {
            offset
        };
        self.main_position_from_start(container, S::ZERO, flex_offset, item_size, S::ZERO)
    }

    #[must_use]
    pub(crate) fn cross_position_from_normal_start<S: LayoutScalar>(
        self,
        container: Size<S>,
        offset: S,
        item_size: S,
    ) -> S {
        let flex_offset = if self.cross_reversed {
            self.cross_size(container) - offset - item_size
        } else {
            offset
        };
        self.cross_position_from_start(container, S::ZERO, flex_offset, item_size, S::ZERO)
    }

    #[must_use]
    const fn normal_axis_start_side(self, axis: LogicalAxis) -> PhysicalSide {
        match axis {
            LogicalAxis::Inline => self.flow_axes.inline_start(),
            LogicalAxis::Block => self.flow_axes.block_start(),
        }
    }

    #[must_use]
    const fn normal_axis_end_side(self, axis: LogicalAxis) -> PhysicalSide {
        self.normal_axis_start_side(axis).opposite()
    }

    const fn reverse_progression(
        progression: PhysicalProgression,
        reversed: bool,
    ) -> PhysicalProgression {
        if !reversed {
            return progression;
        }

        match progression {
            PhysicalProgression::Increasing => PhysicalProgression::Decreasing,
            PhysicalProgression::Decreasing => PhysicalProgression::Increasing,
        }
    }

    fn edge_at_side<T>(self, edges: Edges<T>, side: PhysicalSide) -> T {
        match side {
            PhysicalSide::Top => edges.top,
            PhysicalSide::Right => edges.right,
            PhysicalSide::Bottom => edges.bottom,
            PhysicalSide::Left => edges.left,
        }
    }

    fn set_edge_at_side<T>(self, edges: &mut Edges<T>, side: PhysicalSide, value: T) {
        match side {
            PhysicalSide::Top => edges.top = value,
            PhysicalSide::Right => edges.right = value,
            PhysicalSide::Bottom => edges.bottom = value,
            PhysicalSide::Left => edges.left = value,
        }
    }

    const fn requested_axis_for(axis: PhysicalAxis) -> RequestedAxis {
        match axis {
            PhysicalAxis::Horizontal => RequestedAxis::Horizontal,
            PhysicalAxis::Vertical => RequestedAxis::Vertical,
        }
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
    source_index: usize,
    size: Size<Option<S>>,
    initial_output: ComputeOutputOf<S>,
    flex_basis: S,
    flex_basis_is_definite: bool,
    flex_basis_uses_content: bool,
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
    overflow: ComputedOverflow,
    align_self: AlignItems,
    initial_baseline: FlexItemBaseline<S>,
    flex_grow_factor: S,
    flex_shrink_factor: S,
}

#[derive(Clone, Copy, Debug)]
struct ResolvedFlexItem<Node, S: LayoutScalar> {
    node: Node,
    source_index: usize,
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
    source_index: crate::SourceIndex,
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
    align_self: AlignItems,
    baseline: FlexItemBaseline<S>,
    location: Point<S>,
}

#[derive(Clone, Copy, Debug)]
struct FlexChildContribution<S: LayoutScalar> {
    source_index: crate::SourceIndex,
    location: Point<S>,
    margin: Edges<S>,
    geometry: super::ScrollGeometryOf<S>,
    in_flow: bool,
}

type FlexChildContributionsResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    Vec<FlexChildContribution<<Tree as Traverse>::Scalar>>,
    <Tree as Traverse>::Scalar,
    M,
>;

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
            source_index: item.source_index,
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
    for (source_index, child) in children.into_iter().enumerate() {
        let child_style = tree.node_input(child).clone();
        if child_style.position == Position::Absolute || child_style.display == super::Display::None
        {
            continue;
        }

        let child = build_item(tree, child, source_index, &child_style, constants, run_mode)?;
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
    source_index: usize,
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
    let authored_size = resolve_preferred_size::<_, _, M>(
        node,
        &style.size,
        constants.node_inner_size,
        SizingAlgorithm::Flex,
    )?
    .apply_aspect_ratio(style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let flex_basis_resolution = resolve_flex_basis(
        &style.flex_basis,
        constants.axes.main_physical_axis(),
        constants.axes.main_size(constants.node_inner_size),
    )
    .map_err(|error| sizing_resolution_error(node, error))?;
    let flex_basis_uses_content = match flex_basis_resolution {
        ResolvedFlexBasis::Auto => constants.axes.main_size(authored_size).is_none(),
        ResolvedFlexBasis::Content => true,
        ResolvedFlexBasis::Definite(_) => false,
    };
    let resolved_flex_basis = match flex_basis_resolution {
        ResolvedFlexBasis::Auto => constants.axes.main_size(authored_size),
        ResolvedFlexBasis::Content => None,
        ResolvedFlexBasis::Definite(flex_basis) => Some({
            let padding_border = constants.axes.main_size(padding_border.sum_axes());
            if style.box_sizing == BoxSizing::ContentBox {
                flex_basis + padding_border
            } else {
                flex_basis.max(padding_border)
            }
        }),
    };
    let size = if flex_basis_uses_content {
        constants.axes.with_main_size(authored_size, None)
    } else {
        match resolved_flex_basis {
            Some(flex_basis) => constants
                .axes
                .with_main_size(authored_size, Some(flex_basis)),
            None => authored_size,
        }
    };
    let raw_min_size = resolve_minimum_size::<_, _, M>(
        node,
        &style.min_size,
        constants.node_inner_size,
        SizingAlgorithm::Flex,
    )?;
    let raw_max_size = resolve_maximum_size::<_, _, M>(
        node,
        &style.max_size,
        constants.node_inner_size,
        SizingAlgorithm::Flex,
    )?;
    let min_size = raw_min_size
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let max_size = raw_max_size
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let align_self = style.align_self.unwrap_or(constants.align_items);
    let cross_size_is_auto = constants.axes.cross_size(style.size.clone()).is_auto();
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
    let available = constants.axes.with_cross_size(
        available,
        clamp_available(
            constants.axes.cross_size(available),
            constants.axes.cross_size(min_size),
            constants.axes.cross_size(max_size),
        ),
    );
    let use_content_sizing_for_base =
        flex_basis_uses_content && style.display == super::Display::Block;
    let mut child_known = size;
    if !constants.wraps
        && use_content_sizing_for_base
        && align_self == AlignItems::Stretch
        && cross_size_is_auto
        && !constants.axes.cross_start_edge(margin_is_auto)
        && !constants.axes.cross_end_edge(margin_is_auto)
        && let Some(cross_size) = constants.axes.cross_size(available).into_option()
    {
        child_known = constants.axes.with_cross_size(
            child_known,
            Some((cross_size - constants.axes.cross_edge_sum(margin)).max(Tree::Scalar::ZERO)),
        );
    }
    let mut child_known_for_base = flex_base_known_size(
        constants.axes.with_main_size(size, None),
        constants.axes.cross_size(available),
        style,
        constants,
        margin,
        margin_is_auto,
        align_self,
    );
    let padding_border_main = constants.axes.main_size(padding_border.sum_axes());
    let flex_basis_floor_may_override_content = padding_border_main > Tree::Scalar::ZERO
        || (tree.child_count(node) == 0 && constants.axes.main_size(authored_size).is_some());
    if let Some(flex_basis) = resolved_flex_basis
        && flex_basis <= padding_border_main
        && flex_basis_floor_may_override_content
    {
        child_known_for_base = constants
            .axes
            .with_main_size(child_known_for_base, Some(flex_basis));
    }
    let child_available = if use_content_sizing_for_base {
        constants.axes.with_main_size(
            available,
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
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
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
    let flex_basis = if let Some(flex_basis) = resolved_flex_basis {
        flex_basis
    } else if let Some(ratio) = style.aspect_ratio {
        if let Some(cross) = constants.axes.cross_size(child_known_for_base) {
            constants.axes.main_size_from_cross_aspect(cross, ratio)
        } else {
            constants.axes.main_size(output.size)
        }
    } else {
        constants.axes.main_size(
            tree.compute_child(
                node,
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::ContentSize,
                    constants.axes.main_requested_axis(),
                    child_known_for_base,
                    constants.axes.with_main_size(available_inner_size, None),
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::Flex,
                    ),
                    constants
                        .axes
                        .with_main_size(child_available, AvailableOf::MAX_CONTENT),
                ),
            )?
            .size,
        )
    };
    let hypothetical_main_size = clamp_main_size_axes(
        flex_basis,
        automatic_min_main_size,
        constants.axes.main_size(min_size),
        constants.axes.main_size(max_size),
    );
    let authored_main_size = constants.axes.main_size(authored_size);
    let flex_basis_uses_padding_floor = resolved_flex_basis.is_some()
        && flex_basis <= padding_border_main
        && style.flex_grow.get() == Tree::Scalar::ZERO
        && (tree.child_count(node) > 0
            || constants.axes.main_size(output.content_size) <= flex_basis);
    let intrinsic_main_size = if flex_basis_uses_padding_floor {
        flex_basis
    } else if style.flex_basis.is_auto() && authored_main_size.is_some() {
        authored_main_size.unwrap_or(Tree::Scalar::ZERO)
    } else if flex_basis_uses_content {
        constants.axes.main_size(output.content_size)
    } else {
        constants
            .axes
            .main_size(output.content_size)
            .max(authored_main_size.unwrap_or(Tree::Scalar::ZERO))
    };
    let max_content_main_size = intrinsic_main_size
        .clamp_optional(
            constants.axes.main_size(min_size),
            constants.axes.main_size(max_size),
        )
        .max(padding_border_main);
    let mut target_size = constants
        .axes
        .with_main_size(output.size, hypothetical_main_size);
    if align_self != AlignItems::Stretch
        && cross_size_is_auto
        && let Some(ratio) = style.aspect_ratio
    {
        let transferred_cross = match constants.axes.main_physical_axis() {
            PhysicalAxis::Horizontal => hypothetical_main_size / ratio.get(),
            PhysicalAxis::Vertical => hypothetical_main_size * ratio.get(),
        };
        target_size = constants
            .axes
            .with_cross_size(target_size, transferred_cross);
    }
    target_size = constants.axes.with_cross_size(
        target_size,
        constants
            .axes
            .cross_size(target_size)
            .clamp_optional(
                constants.axes.cross_size(raw_min_size),
                constants.axes.cross_size(raw_max_size),
            )
            .max(constants.axes.cross_size(padding_border.sum_axes())),
    );
    let child_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let baseline = FlexItemBaseline::from_output(output, child_flow_axes);

    Ok(CollectedFlexItem {
        node,
        source_index,
        size: authored_size,
        initial_output: output,
        flex_basis,
        flex_basis_is_definite: resolved_flex_basis.is_some(),
        flex_basis_uses_content,
        hypothetical_main_size,
        max_content_main_size,
        hypothetical_size: target_size,
        cross_size_is_auto,
        automatic_min_main_size,
        min_size,
        max_size,
        min_cross_size: constants.axes.cross_size(raw_min_size),
        max_cross_size: constants.axes.cross_size(raw_max_size),
        margin,
        margin_is_auto,
        inset,
        padding,
        border,
        overflow: style.overflow,
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
    if !constants.axes.main_size(style.min_size.clone()).is_auto()
        || flex_automatic_minimum_is_zero(style.overflow)
    {
        return Ok(None);
    }
    let authored_size = resolve_preferred_size::<_, _, M>(
        node,
        &style.size,
        constants.node_inner_size,
        SizingAlgorithm::Flex,
    )?
    .apply_aspect_ratio(style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let min_size = resolve_minimum_size::<_, _, M>(
        node,
        &style.min_size,
        constants.node_inner_size,
        SizingAlgorithm::Flex,
    )?
    .apply_aspect_ratio(style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let resolved_max_size = resolve_maximum_size::<_, _, M>(
        node,
        &style.max_size,
        constants.node_inner_size,
        SizingAlgorithm::Flex,
    )?
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

    let available = constants.axes.size_from_main_cross(
        AvailableOf::MIN_CONTENT,
        clamp_available(
            constants
                .axes
                .cross_size(constants.node_inner_size)
                .map(AvailableOf::definite)
                .unwrap_or(AvailableOf::MAX_CONTENT),
            constants.axes.cross_size(min_size),
            constants.axes.cross_size(resolved_max_size),
        ),
    );
    let output = tree.compute_child(
        node,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::ContentSize,
            constants.axes.main_requested_axis(),
            child_known,
            constants
                .axes
                .with_main_size(constants.node_inner_size, None),
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
            available,
        ),
    )?;

    let mut min_content = constants
        .axes
        .main_size(output.size)
        .clamp_optional(None, constants.axes.main_size(authored_size))
        .clamp_optional(None, constants.axes.main_size(resolved_max_size));
    if let Some(ratio) = style.aspect_ratio
        && let Some(cross) = constants.axes.cross_size(child_known)
    {
        let transferred = constants
            .axes
            .main_size_from_cross_aspect(cross, ratio)
            .clamp_optional(None, constants.axes.main_size(authored_size))
            .clamp_optional(None, constants.axes.main_size(resolved_max_size));
        min_content = if style.item_is_replaced {
            min_content.min(transferred)
        } else {
            min_content.max(transferred)
        };
    }
    Ok(Some(
        min_content.max(constants.axes.main_size(padding_border.sum_axes())),
    ))
}

fn flex_automatic_minimum_is_zero(overflow: ComputedOverflow) -> bool {
    overflow.x().is_scrollable() || overflow.y().is_scrollable()
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
    let mut known = constants.axes.with_main_size(size, None);
    if align_self == AlignItems::Stretch
        && constants.axes.cross_size(style.size.clone()).is_auto()
        && constants.axes.cross_size(known).is_none()
        && !constants.axes.cross_start_edge(margin_is_auto)
        && !constants.axes.cross_end_edge(margin_is_auto)
        && let Some(cross) = cross_available.into_option()
    {
        known = constants.axes.with_cross_size(
            known,
            Some((cross - constants.axes.cross_edge_sum(margin)).max(S::ZERO)),
        );
    }
    known
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
                constants.axes.main_size(constants.gap)
            };
            let next_size = gap
                + constants.axes.main_size(items[end].hypothetical_size)
                + constants.axes.main_edge_sum(items[end].margin);
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
    constants.axes.main_size(constants.node_inner_size)
}

fn flex_line_collection_size<S: LayoutScalar>(constants: &Constants<S>) -> Option<S> {
    constants
        .axes
        .main_size(constants.node_inner_size)
        .or_else(|| constants.axes.main_size(constants.max_inner_size))
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
    let cross_gap = constants.axes.cross_size(constants.gap);
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
            constants.axes.main_size(constants.gap),
            justify_content,
            constants.axes.main_is_reversed(),
            true,
        );
        let mut cross_size = Tree::Scalar::ZERO;

        for (index, item_index) in (line.start..line.end).enumerate() {
            if index > 0 {
                main_cursor = main_cursor
                    + alignment_offset(
                        free_space,
                        item_count,
                        constants.axes.main_size(constants.gap),
                        justify_content,
                        constants.axes.main_is_reversed(),
                        false,
                    );
            }

            let item = &mut resolved_items[item_index];
            determine_hypothetical_cross_size(tree, item, constants)?;
            item.offset_main = main_cursor + item.margin_main_start(constants);
            item.offset_cross = cross_cursor + constants.axes.cross_start_edge(item.margin);

            main_cursor = main_cursor
                + constants.axes.main_size(item.target_size)
                + constants.axes.main_edge_sum(item.margin);
            cross_size = Tree::Scalar::max(
                cross_size,
                constants.axes.cross_size(item.target_size)
                    + constants.axes.cross_edge_sum(item.margin),
            );
        }
        cross_size = Tree::Scalar::max(
            cross_size,
            line_cross_size(&resolved_items[line.start..line.end], constants),
        );

        line.main_size = main_cursor;
        line.cross_size = if single_line {
            constants
                .axes
                .cross_size(constants.node_inner_size)
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
    let Some(container_cross_size) = constants.axes.cross_size(constants.node_inner_size) else {
        return;
    };
    let line_count = lines.len();
    let cross_gap = constants.axes.cross_size(constants.gap);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(line_count.saturating_sub(1));
    let free_space = container_cross_size - used_cross_size;
    let align_content = alignment_fallback(free_space, line_count, constants.align_content);
    let mut cross_cursor = alignment_offset(
        free_space,
        line_count,
        cross_gap,
        align_content,
        constants.axes.cross_is_reversed(),
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
                    constants.axes.cross_is_reversed(),
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
    let padding_border_cross = constants
        .axes
        .cross_size((item.padding + item.border).sum_axes());
    let authored_cross = constants
        .axes
        .cross_size(item.size)
        .map(|cross| {
            cross.clamp_optional(
                constants.axes.cross_size(item.min_size),
                constants.axes.cross_size(item.max_size),
            )
        })
        .map(|cross| cross.max(padding_border_cross));
    let available_cross = clamp_available(
        constants
            .axes
            .cross_size(constants.node_inner_size)
            .map(AvailableOf::definite)
            .unwrap_or(constants.axes.cross_size(constants.available)),
        constants.axes.cross_size(item.min_size),
        constants.axes.cross_size(item.max_size),
    );
    let available_cross = match available_cross {
        AvailableOf::Definite(value) => AvailableOf::Definite(value.max(padding_border_cross)),
        other => other,
    };
    let measured_cross = if let Some(authored_cross) = authored_cross {
        authored_cross
    } else {
        let main_size_changed = (constants.axes.main_size(item.target_size)
            - constants.axes.main_size(item.initial_output.size))
        .abs()
            > Tree::Scalar::from_f64(0.001);
        if item.initial_output.content_size == item.initial_output.size && !main_size_changed {
            constants
                .axes
                .cross_size(item.initial_output.size)
                .clamp_optional(
                    constants.axes.cross_size(item.min_size),
                    constants.axes.cross_size(item.max_size),
                )
                .max(padding_border_cross)
        } else {
            let measured = tree.compute_child(
                item.node,
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::ContentSize,
                    constants.axes.cross_requested_axis(),
                    constants.axes.size_from_main_cross(
                        Some(constants.axes.main_size(item.target_size)),
                        authored_cross,
                    ),
                    constants.node_inner_size,
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::Flex,
                    ),
                    constants.axes.size_from_main_cross(
                        constants
                            .axes
                            .main_size(constants.node_inner_size)
                            .map(AvailableOf::definite)
                            .unwrap_or(AvailableOf::MAX_CONTENT),
                        available_cross,
                    ),
                ),
            )?;
            item.baseline.refresh(measured);
            constants
                .axes
                .cross_size(measured.size)
                .clamp_optional(
                    constants.axes.cross_size(item.min_size),
                    constants.axes.cross_size(item.max_size),
                )
                .max(padding_border_cross)
        }
    };

    item.target_size = constants
        .axes
        .with_cross_size(item.target_size, measured_cross);
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

    let Some(container_cross_size) = constants.axes.cross_size(constants.node_inner_size) else {
        return;
    };
    let cross_gap = constants.axes.cross_size(constants.gap);
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
    let cross_axis = constants.axes.cross_physical_axis();
    let max_baseline = max_line_baseline(items, cross_axis);
    for item in items {
        resolve_cross_axis_auto_margins(item, line_cross_size, constants);
        let outer_cross_size = constants.axes.cross_size(item.target_size)
            + constants.axes.cross_edge_sum(item.margin);
        let free_space = line_cross_size - outer_cross_size;
        if item.align_self == AlignItems::Stretch && item.cross_size_is_auto {
            let stretched_cross_size = S::max(
                S::ZERO,
                line_cross_size - constants.axes.cross_edge_sum(item.margin),
            );
            let stretched_cross_size = clamp_cross_size(item, stretched_cross_size);
            item.target_size = constants
                .axes
                .with_cross_size(item.target_size, stretched_cross_size);
        }
        let alignment_offset = match item.align_self.safe_fallback(free_space) {
            AlignItems::Start => {
                if constants.axes.cross_is_reversed() {
                    free_space
                } else {
                    S::ZERO
                }
            }
            AlignItems::End | AlignItems::LastBaseline => {
                if constants.axes.cross_is_reversed() {
                    S::ZERO
                } else {
                    free_space
                }
            }
            AlignItems::FlexStart | AlignItems::Stretch => S::ZERO,
            AlignItems::Center => free_space / S::from_f64(2.0),
            AlignItems::FlexEnd => free_space,
            AlignItems::Baseline if item.baseline.axis(item.target_size) == cross_axis => {
                max_baseline - item.baseline.value(item.target_size, item.margin)
            }
            AlignItems::Baseline
                if constants.wraps && constants.axes.flow_direction() == Direction::Rtl =>
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
            constants.axes.cross_start_edge(item.margin)
        };
        item.offset_cross = line_cross_offset + line_over_margin + alignment_offset;
    }
}

fn line_cross_size<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let cross_axis = constants.axes.cross_physical_axis();
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
    let outer_cross_size =
        constants.axes.cross_size(item.target_size) + constants.axes.cross_edge_sum(item.margin);
    if item.align_self == AlignItems::Baseline
        && item.baseline.axis(item.target_size) == constants.axes.cross_physical_axis()
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

fn first_vertical_baseline<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.first()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .find(|item| {
            constants.axes.main_logical_axis() == LogicalAxis::Block
                || item.align_self == AlignItems::Baseline
        })
        .or_else(|| line_items.first())?;
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    let location = constants.axes.point_from_main_cross(
        constants.axes.main_position_from_start(
            container,
            constants.axes.main_start_edge(constants.content_box_inset),
            item.offset_main,
            constants.axes.main_size(item.target_size),
            S::ZERO,
        ),
        constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            item.offset_cross,
            constants.axes.cross_size(item.target_size),
            S::ZERO,
        ),
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
        .find(|item| {
            constants.axes.main_logical_axis() == LogicalAxis::Block
                || item.align_self == AlignItems::Baseline
        })
        .or_else(|| line_items.last())?;
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    let location = constants.axes.point_from_main_cross(
        constants.axes.main_position_from_start(
            container,
            constants.axes.main_start_edge(constants.content_box_inset),
            item.offset_main,
            constants.axes.main_size(item.target_size),
            S::ZERO,
        ),
        constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            item.offset_cross,
            constants.axes.cross_size(item.target_size),
            S::ZERO,
        ),
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
        .find(|item| {
            constants.axes.main_logical_axis() == LogicalAxis::Block
                || item.align_self == AlignItems::Baseline
        })
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
        .find(|item| {
            constants.axes.main_logical_axis() == LogicalAxis::Block
                || item.align_self == AlignItems::Baseline
        })
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

fn resolve_cross_axis_auto_margins<Node, S: LayoutScalar>(
    item: &mut ResolvedFlexItem<Node, S>,
    line_cross_size: S,
    constants: &Constants<S>,
) {
    let auto_start = constants.axes.cross_start_edge(item.margin_is_auto);
    let auto_end = constants.axes.cross_end_edge(item.margin_is_auto);
    if !auto_start && !auto_end {
        return;
    }
    if auto_start {
        constants
            .axes
            .set_cross_start_edge(&mut item.margin, S::ZERO);
    }
    if auto_end {
        constants.axes.set_cross_end_edge(&mut item.margin, S::ZERO);
    }

    let free_space = line_cross_size
        - constants.axes.cross_size(item.target_size)
        - constants.axes.cross_edge_sum(item.margin);
    if auto_start && auto_end {
        let margin = free_space / S::from_f64(2.0);
        constants
            .axes
            .set_cross_start_edge(&mut item.margin, margin);
        constants.axes.set_cross_end_edge(&mut item.margin, margin);
    } else if auto_start {
        constants
            .axes
            .set_cross_start_edge(&mut item.margin, free_space);
    } else if auto_end {
        constants
            .axes
            .set_cross_end_edge(&mut item.margin, free_space);
    }
}

fn line_free_space<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
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
                constants.axes.main_size(constants.gap)
            };
            gap + constants.axes.main_size(item.target_size)
                + constants.axes.main_edge_sum(item.margin)
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
            AlignContent::Start => {
                if layout_is_flex_reversed {
                    free_space
                } else {
                    S::ZERO
                }
            }
            AlignContent::FlexStart => S::ZERO,
            AlignContent::End => {
                if layout_is_flex_reversed {
                    S::ZERO
                } else {
                    free_space
                }
            }
            AlignContent::FlexEnd => free_space,
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
        constants.axes.main_start_edge(self.margin)
    }

    fn margin_main_start_is_auto(&self, constants: &Constants<S>) -> bool {
        constants.axes.main_start_edge(self.margin_is_auto)
    }

    fn margin_main_end_is_auto(&self, constants: &Constants<S>) -> bool {
        constants.axes.main_end_edge(self.margin_is_auto)
    }

    fn set_margin_main_start(&mut self, constants: &Constants<S>, value: S) {
        constants.axes.set_main_start_edge(&mut self.margin, value);
    }

    fn set_margin_main_end(&mut self, constants: &Constants<S>, value: S) {
        constants.axes.set_main_end_edge(&mut self.margin, value);
    }

    fn final_main_location(&self, constants: &Constants<S>, output_size: Size<S>) -> S {
        let container = constants
            .node_outer_size
            .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
        constants.axes.main_position_from_start(
            container,
            constants.axes.main_start_edge(constants.content_box_inset),
            self.offset_main,
            constants.axes.main_size(output_size),
            self.relative_main_inset(constants),
        )
    }

    fn relative_main_inset(&self, constants: &Constants<S>) -> S {
        let normal_offset = constants
            .axes
            .normal_main_start_edge(self.inset)
            .or_else(|| {
                constants
                    .axes
                    .normal_main_end_edge(self.inset)
                    .map(|inset| -inset)
            })
            .unwrap_or(S::ZERO);
        constants.axes.main_offset_from_normal_flow(normal_offset)
    }

    fn final_cross_location(&self, constants: &Constants<S>, output_size: Size<S>) -> S {
        let container = constants
            .node_outer_size
            .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
        constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            self.offset_cross,
            constants.axes.cross_size(output_size),
            self.relative_cross_inset(constants),
        )
    }

    fn relative_cross_inset(&self, constants: &Constants<S>) -> S {
        let normal_offset = constants
            .axes
            .normal_cross_start_edge(self.inset)
            .or_else(|| {
                constants
                    .axes
                    .normal_cross_end_edge(self.inset)
                    .map(|inset| -inset)
            })
            .unwrap_or(S::ZERO);
        constants.axes.cross_offset_from_normal_flow(normal_offset)
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
    let mut frozen = vec![false; items.len()];
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let initial_free_space = container_main_size - flex_used_space(items, constants, &frozen);

    for (item, frozen) in items.iter_mut().zip(&mut frozen) {
        item.target_size = constants
            .axes
            .with_main_size(item.target_size, item.flex_basis);
        if item.flex_grow_factor == S::ZERO || item.flex_basis > item.hypothetical_main_size {
            item.target_size = constants
                .axes
                .with_main_size(item.target_size, item.hypothetical_main_size);
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
            let clamped = clamp_main_size(item, constants.axes, grown_main_size);
            item.target_size = constants.axes.with_main_size(item.target_size, clamped);
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
    let mut frozen = vec![false; items.len()];
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let initial_free_space = container_main_size - flex_used_space(items, constants, &frozen);

    for (item, frozen) in items.iter_mut().zip(&mut frozen) {
        item.target_size = constants
            .axes
            .with_main_size(item.target_size, item.flex_basis);
        if item.flex_shrink_factor == S::ZERO || item.flex_basis < item.hypothetical_main_size {
            item.target_size = constants
                .axes
                .with_main_size(item.target_size, item.hypothetical_main_size);
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
            let clamped =
                clamp_main_size(item, constants.axes, S::max(S::ZERO, shrunken_main_size));
            item.target_size = constants.axes.with_main_size(item.target_size, clamped);
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
    items
        .iter()
        .zip(frozen)
        .enumerate()
        .map(|(index, (item, frozen))| {
            let gap = if index == 0 {
                S::ZERO
            } else {
                constants.axes.main_size(constants.gap)
            };
            let main_size = if *frozen {
                constants.axes.main_size(item.target_size)
            } else {
                item.flex_basis
            };
            gap + main_size + constants.axes.main_edge_sum(item.margin)
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
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 {
                S::ZERO
            } else {
                constants.axes.main_size(constants.gap)
            };
            gap + constants.axes.main_size(item.target_size)
                + constants.axes.main_edge_sum(item.margin)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

fn clamp_main_size<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    axes: FlexAxes,
    value: S,
) -> S {
    clamp_main_size_axes(
        value,
        item.automatic_min_main_size,
        axes.main_size(item.min_size),
        axes.main_size(item.max_size),
    )
}

fn clamp_cross_size<Node, S: LayoutScalar>(item: &ResolvedFlexItem<Node, S>, value: S) -> S {
    value.clamp_optional(item.min_cross_size, item.max_cross_size)
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

#[derive(Clone, Copy)]
struct FlexContainerSizes<S: LayoutScalar> {
    output: Size<S>,
    intrinsic_content: Size<S>,
}

fn container_sizes<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    resolved_items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
) -> FlexContainerSizes<S> {
    let line_cross_gap =
        constants.axes.cross_size(constants.gap) * S::from_usize(lines.len().saturating_sub(1));
    let content_main = intrinsic_content_main_size(input, constants, resolved_items, lines);
    let content_cross = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + line_cross_gap;
    let content_size = constants
        .axes
        .size_from_main_cross(content_main, content_cross);
    let outer_size = constants
        .node_outer_size
        .unwrap_or(content_size + constants.content_box_inset.sum_axes())
        .clamp_optional(constants.min_outer_size, constants.max_outer_size);
    let mut output_size = input
        .known()
        .or(constants.node_outer_size)
        .unwrap_or(outer_size)
        .max_optional(constants.padding_border_size.map(Some));
    if constants
        .axes
        .main_size(constants.node_outer_size)
        .is_none()
        && lines.len() > 1
        && let AvailableOf::Definite(available_main) = constants.axes.main_size(input.available())
    {
        output_size = constants.axes.with_main_size(
            output_size,
            constants.axes.main_size(output_size).max(available_main),
        );
    }

    FlexContainerSizes {
        output: output_size,
        intrinsic_content: content_size,
    }
}

fn container_output<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    resolved_items: &[ResolvedFlexItem<Node, S>],
    final_items: Option<&[FinalFlexItem<Node, S>]>,
    lines: &[FlexLine<S>],
    final_content_size: Option<Size<S>>,
    container_sizes: FlexContainerSizes<S>,
) -> ComputeOutputOf<S> {
    let FlexContainerSizes {
        output: output_size,
        intrinsic_content: content_size,
    } = container_sizes;
    let content_size = if input.run_mode().is_perform_layout() {
        final_content_size.expect("perform-layout flex output requires accumulated content extent")
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

fn flex_container_scroll_box<Node, S, M>(
    node: Node,
    run_mode: RunMode,
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    output_size: Size<S>,
) -> LayoutResultOf<Node, CanonicalScrollBoxOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
        flow_axes: constants.flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: output_size,
        border: constants.border,
        padding: constants.padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: constants.settled_auto_scrollbars,
    })
    .map_err(|error| flex_own_geometry_error(node, run_mode, error))
}

fn flex_scroll_contributions<Node, S: LayoutScalar>(
    final_items: &[FinalFlexItem<Node, S>],
    absolute_contributions: &[FlexChildContribution<S>],
    resolved_items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
) -> Result<ScrollContributionAccumulatorOf<S>, crate::scroll::ScrollContributionErrorOf<S>> {
    let mut children = final_items
        .iter()
        .map(|item| FlexChildContribution {
            source_index: item.source_index,
            location: item.location,
            margin: item.margin,
            geometry: item
                .output
                .scroll_geometry
                .expect("final flex items retain canonical geometry"),
            in_flow: true,
        })
        .chain(absolute_contributions.iter().copied())
        .collect::<Vec<_>>();
    children.sort_by_key(|child| child.source_index);

    let mut contributions = ScrollContributionAccumulatorOf::new(scroll_box.padding_box());
    let mut inline_end = None;
    let mut block_end = None;
    for child in children {
        if child.in_flow {
            contributions.include_in_flow_geometry(child.location, child.margin, child.geometry)?;
            let border_size = child.geometry.border_box().size();
            if border_size.width > S::ZERO && border_size.height > S::ZERO {
                include_farthest_flow_end(
                    &mut inline_end,
                    constants.flow_axes.inline_end(),
                    child_flow_end(child, constants.flow_axes.inline_end()),
                );
                include_farthest_flow_end(
                    &mut block_end,
                    constants.flow_axes.block_end(),
                    child_flow_end(child, constants.flow_axes.block_end()),
                );
            }
        } else {
            contributions.include_current_out_of_flow_geometry(
                child.location,
                child.margin,
                child.geometry,
            )?;
        }
    }

    for (axis, coordinate) in [
        (LogicalAxis::Inline, inline_end),
        (LogicalAxis::Block, block_end),
    ] {
        if let Some(coordinate) = coordinate {
            contributions.record_final_in_flow_end(constants.flow_axes, axis, coordinate)?;
        }
    }
    include_flex_alignment_subjects(
        &mut contributions,
        final_items,
        resolved_items,
        lines,
        constants,
    )?;
    contributions.include_terminal_padding(constants.padding)?;
    Ok(contributions)
}

fn include_flex_alignment_subjects<Node, S: LayoutScalar>(
    contributions: &mut ScrollContributionAccumulatorOf<S>,
    final_items: &[FinalFlexItem<Node, S>],
    resolved_items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Result<(), crate::scroll::ScrollContributionErrorOf<S>> {
    if let Some(authored) = constants.authored_justify_content
        && lines.iter().any(|line| {
            let free_space = line_free_space(&resolved_items[line.start..line.end], constants);
            !safe_alignment_lands_at_origin_start(authored, free_space)
        })
        && let Some((minimum, maximum)) = final_item_subject_interval(final_items, constants)
    {
        set_alignment_subject_interval(
            contributions,
            constants.axes.main_physical_axis(),
            minimum,
            maximum,
        )?;
    }

    if let Some(authored) = constants.authored_align_content
        && lines.len() > 1
        && let Some(free_space) = line_cross_free_space(lines, constants)
        && !safe_alignment_lands_at_origin_start(authored, free_space)
        && let Some((minimum, maximum)) = line_subject_interval(lines, constants)
    {
        set_alignment_subject_interval(
            contributions,
            constants.axes.cross_physical_axis(),
            minimum,
            maximum,
        )?;
    }
    Ok(())
}

fn safe_alignment_lands_at_origin_start<S: LayoutScalar>(
    authored: AlignContent,
    free_space: S,
) -> bool {
    free_space < S::ZERO
        && matches!(
            authored,
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter
        )
}

fn final_item_subject_interval<Node, S: LayoutScalar>(
    final_items: &[FinalFlexItem<Node, S>],
    constants: &Constants<S>,
) -> Option<(S, S)> {
    final_items.iter().fold(None, |bounds, item| {
        let border_box = item
            .output
            .scroll_geometry
            .expect("final flex items retain canonical geometry")
            .border_box();
        let (origin, end) = match constants.axes.main_physical_axis() {
            PhysicalAxis::Horizontal => {
                let origin = item.location.x + border_box.origin().x;
                (
                    origin - item.margin.left.max(S::ZERO),
                    origin + border_box.size().width + item.margin.right.max(S::ZERO),
                )
            }
            PhysicalAxis::Vertical => {
                let origin = item.location.y + border_box.origin().y;
                (
                    origin - item.margin.top.max(S::ZERO),
                    origin + border_box.size().height + item.margin.bottom.max(S::ZERO),
                )
            }
        };
        Some(bounds.map_or((origin, end), |(minimum, maximum): (S, S)| {
            (minimum.min(origin), maximum.max(end))
        }))
    })
}

fn line_cross_free_space<S: LayoutScalar>(
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<S> {
    let container_cross_size = constants.axes.cross_size(constants.node_inner_size)?;
    let cross_gap = constants.axes.cross_size(constants.gap);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(lines.len().saturating_sub(1));
    Some(container_cross_size - used_cross_size)
}

fn line_subject_interval<S: LayoutScalar>(
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<(S, S)> {
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    lines.iter().fold(None, |bounds, line| {
        let origin = constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            line.offset_cross,
            line.cross_size,
            S::ZERO,
        );
        let end = origin + line.cross_size;
        Some(bounds.map_or((origin, end), |(minimum, maximum): (S, S)| {
            (minimum.min(origin), maximum.max(end))
        }))
    })
}

fn set_alignment_subject_interval<S: LayoutScalar>(
    contributions: &mut ScrollContributionAccumulatorOf<S>,
    axis: PhysicalAxis,
    minimum: S,
    maximum: S,
) -> Result<(), crate::scroll::ScrollContributionErrorOf<S>> {
    let subject = match axis {
        PhysicalAxis::Horizontal => super::ScrollRectOf::try_new(
            Point::new(minimum, S::ZERO),
            Size::new(maximum - minimum, S::ZERO),
        ),
        PhysicalAxis::Vertical => super::ScrollRectOf::try_new(
            Point::new(S::ZERO, minimum),
            Size::new(S::ZERO, maximum - minimum),
        ),
    }?;
    contributions.set_active_alignment_subject(axis, subject);
    Ok(())
}

fn child_flow_end<S: LayoutScalar>(child: FlexChildContribution<S>, side: PhysicalSide) -> S {
    let border_box = child.geometry.border_box();
    let origin = border_box.origin();
    let size = border_box.size();
    match side {
        PhysicalSide::Top => child.location.y + origin.y - child.margin.top.max(S::ZERO),
        PhysicalSide::Right => {
            child.location.x + origin.x + size.width + child.margin.right.max(S::ZERO)
        }
        PhysicalSide::Bottom => {
            child.location.y + origin.y + size.height + child.margin.bottom.max(S::ZERO)
        }
        PhysicalSide::Left => child.location.x + origin.x - child.margin.left.max(S::ZERO),
    }
}

fn include_farthest_flow_end<S: LayoutScalar>(
    end: &mut Option<S>,
    side: PhysicalSide,
    candidate: S,
) {
    *end = Some(end.map_or(candidate, |current| match side {
        PhysicalSide::Top | PhysicalSide::Left => current.min(candidate),
        PhysicalSide::Right | PhysicalSide::Bottom => current.max(candidate),
    }));
}

fn flex_container_scroll_geometry<Node, S, M>(
    node: Node,
    run_mode: RunMode,
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    contributions: ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<Node, super::ScrollGeometryOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes: constants.flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: scroll_box.border_box().size(),
        border: constants.border,
        padding: constants.padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: constants.settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: flex_scroll_padding(style.scroll_padding),
        contributions,
        origin_axes: constants.axes.scroll_origin_axes(),
        scroll_snap_type: style.scroll_snap_type,
        target_border_box: scroll_box.border_box(),
        target_scroll_margin: style.scroll_margin,
        target_flow_axes: constants.flow_axes,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
    .map_err(|error| flex_own_geometry_error(node, run_mode, error))
}

fn flex_scroll_padding<S: LayoutScalar>(
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

fn flex_own_geometry_error<Node, S, M, E>(
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

fn flex_child_geometry_error<Node, S, M, E>(
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

fn retained_flex_child_scroll_geometry<S: LayoutScalar>(
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

    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
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
        scroll_padding: flex_scroll_padding(style.scroll_padding),
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

fn intrinsic_content_main_size<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
) -> S {
    if constants
        .axes
        .main_size(constants.node_outer_size)
        .is_none()
        && constants.axes.main_logical_axis() == LogicalAxis::Inline
        && constants.axes.main_size(input.available()) == AvailableOf::MAX_CONTENT
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
    let gap = constants.axes.main_size(constants.gap);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 { S::ZERO } else { gap };
            gap + item.max_content_main_size + constants.axes.main_edge_sum(item.margin)
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
    let gap_basis = constants.axes.size_from_main_cross(
        constants.axes.main_size(constants.node_inner_size),
        constants
            .axes
            .cross_size(original_inner_size)
            .and(constants.axes.cross_size(constants.node_inner_size)),
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
    let fallback_outer_main_size = if constants
        .axes
        .main_size(constants.node_outer_size)
        .is_none()
    {
        let content_main = match constants.axes.main_size(input.available()) {
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
        Some(
            content_main
                + constants
                    .axes
                    .main_size(constants.content_box_inset.sum_axes()),
        )
    } else {
        None
    };
    let Some(outer_main_size) = constants
        .axes
        .main_size(constants.node_outer_size)
        .or(fallback_outer_main_size)
    else {
        return Ok(());
    };

    let outer_main_size = outer_main_size
        .clamp_optional(
            constants.axes.main_size(constants.min_outer_size),
            constants.axes.main_size(constants.max_outer_size),
        )
        .max(
            constants
                .axes
                .main_size(constants.content_box_inset.sum_axes())
                - constants
                    .axes
                    .main_size(constants.scrollbar_gutter.sum_axes()),
        );
    let inner_main_size = (outer_main_size
        - constants
            .axes
            .main_size(constants.content_box_inset.sum_axes()))
    .max(Tree::Scalar::ZERO);

    constants.node_outer_size = constants
        .axes
        .with_main_size(constants.node_outer_size, Some(outer_main_size));
    constants.node_inner_size = constants
        .axes
        .with_main_size(constants.node_inner_size, Some(inner_main_size));
    constants.available_main = AvailableOf::definite(inner_main_size);
    Ok(())
}

fn flex_basis_line_main_size<Node, S: LayoutScalar>(
    items: &[CollectedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let gap = constants.axes.main_size(constants.gap);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 { S::ZERO } else { gap };
            let padding_border = constants
                .axes
                .main_size((item.padding + item.border).sum_axes());
            let main_size = constants
                .axes
                .main_size(item.min_size)
                .map_or(item.flex_basis, |min| item.flex_basis.max(min))
                .max(padding_border);
            gap + main_size + constants.axes.main_edge_sum(item.margin)
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
        let gap = constants.axes.main_size(constants.gap);
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
    let style_min = constants.axes.main_size(item.min_size);
    let style_preferred = (!item.flex_basis_uses_content)
        .then(|| constants.axes.main_size(item.size))
        .flatten();
    let style_max = constants.axes.main_size(item.max_size);
    let padding_border = constants
        .axes
        .main_size((item.padding + item.border).sum_axes());
    let contentful_padding_floor_item = item.flex_basis_is_definite
        && item.flex_basis <= padding_border
        && tree.child_count(item.node) == 0
        && constants.axes.main_size(item.initial_output.content_size) > item.flex_basis;
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
        && constants.axes.main_size(item.initial_output.size) <= item.flex_basis
        && constants.axes.main_size(item.initial_output.content_size) <= item.flex_basis
    {
        return Ok(item.flex_basis + constants.axes.main_edge_sum(item.margin));
    }

    let cross_available = intrinsic_item_cross_available(input, constants, item);
    let needs_stretched_cross_measure = item.align_self == AlignItems::Stretch
        && constants.axes.cross_size(item.size).is_none()
        && cross_available.into_option().is_some();

    let contribution = match (style_preferred, max_main <= min_main) {
        _ if flex_automatic_minimum_is_zero(item.overflow) => item.flex_basis.max(min_main),
        (Some(preferred), _) if max_main <= preferred => preferred.min(max_main).max(min_main),
        (_, true) => min_main,
        _ if constants.axes.main_logical_axis() == LogicalAxis::Inline
            && constants.axes.main_size(input.available()) == AvailableOf::MinContent =>
        {
            min_main
        }
        _ if !needs_stretched_cross_measure => {
            if constants.axes.main_logical_axis() == LogicalAxis::Inline {
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
            let child_available = constants
                .axes
                .with_cross_size(input.available(), cross_available);
            let measured = constants.axes.main_size(
                tree.compute_child(
                    item.node,
                    ComputeInputOf::for_child(
                        RunMode::ComputeSize,
                        SizingMode::InherentSize,
                        constants.axes.main_requested_axis(),
                        child_known,
                        constants.node_inner_size,
                        ContainingLayoutContext::new(
                            constants.flow_axes,
                            ParentFormattingContext::Flex,
                        ),
                        child_available,
                    ),
                )?
                .size,
            );

            if constants.axes.main_logical_axis() == LogicalAxis::Inline {
                measured.clamp_optional(style_min, style_max)
            } else {
                measured
                    .max(item.flex_basis)
                    .clamp_optional(style_min, style_max)
            }
        }
    };

    Ok(contribution + constants.axes.main_edge_sum(item.margin))
}

fn intrinsic_item_cross_available<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    item: &CollectedFlexItem<Node, S>,
) -> AvailableOf<S> {
    let cross_margin_sum = constants.axes.cross_edge_sum(item.margin);
    let child_min_cross = constants
        .axes
        .cross_size(item.min_size)
        .map(|value| value + cross_margin_sum);
    let child_max_cross = constants
        .axes
        .cross_size(item.max_size)
        .map(|value| value + cross_margin_sum);
    let parent_cross = constants.axes.cross_size(constants.node_inner_size);
    let cross_available = constants.axes.cross_size(input.available());
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
    let mut known = constants.axes.with_main_size(item.size, None);
    if item.align_self == AlignItems::Stretch
        && constants.axes.cross_size(known).is_none()
        && let Some(cross) = cross_available.into_option()
    {
        known = constants.axes.with_cross_size(
            known,
            Some((cross - constants.axes.cross_edge_sum(item.margin)).max(S::ZERO)),
        );
    }
    known
}

fn resolved_cross_layout_constants<S: LayoutScalar>(
    constants: &Constants<S>,
    lines: &[FlexLine<S>],
) -> Constants<S> {
    if constants
        .axes
        .cross_size(constants.node_outer_size)
        .is_some()
    {
        return *constants;
    }

    let line_cross_gap =
        constants.axes.cross_size(constants.gap) * S::from_usize(lines.len().saturating_sub(1));
    let content_cross = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + line_cross_gap;
    let cross_inset = constants
        .axes
        .cross_size(constants.content_box_inset.sum_axes());
    let outer_cross_size = (content_cross + cross_inset)
        .clamp_optional(
            constants.axes.cross_size(constants.min_outer_size),
            constants.axes.cross_size(constants.max_outer_size),
        )
        .max(
            cross_inset
                - constants
                    .axes
                    .cross_size(constants.scrollbar_gutter.sum_axes()),
        )
        .max(constants.axes.cross_size(constants.padding_border_size));
    let inner_cross_size = (outer_cross_size - cross_inset).max(S::ZERO);

    let mut constants = *constants;
    constants.node_outer_size = constants
        .axes
        .with_cross_size(constants.node_outer_size, Some(outer_cross_size));
    constants.node_inner_size = constants
        .axes
        .with_cross_size(constants.node_inner_size, Some(inner_cross_size));
    constants.max_inner_size = constants.max_inner_size.or(constants.node_inner_size);
    constants
}

fn max_option<S: LayoutScalar>(a: Option<S>, b: Option<S>) -> Option<S> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex finalizer preserves node, scalar, and provider error types"
)]
fn final_layout<Tree, M>(
    tree: &mut Tree,
    container_node: <Tree as Traverse>::Node,
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
                ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
                constants.axes.size_from_main_cross(
                    constants
                        .axes
                        .main_size(constants.node_inner_size)
                        .map(AvailableOf::definite)
                        .unwrap_or(AvailableOf::MAX_CONTENT),
                    constants
                        .axes
                        .cross_size(constants.node_inner_size)
                        .map(AvailableOf::definite)
                        .unwrap_or(AvailableOf::MAX_CONTENT),
                ),
            ),
        )?;
        let resolved_flex_basis = match resolve_flex_basis(
            &style.flex_basis,
            constants.axes.main_physical_axis(),
            constants.axes.main_size(constants.node_inner_size),
        )
        .map_err(|error| sizing_resolution_error(item.node, error))?
        {
            ResolvedFlexBasis::Definite(value) => Some(value),
            ResolvedFlexBasis::Auto | ResolvedFlexBasis::Content => None,
        };
        suppress_padding_floor_flex_basis_content_overflow(
            tree,
            item,
            &mut output,
            resolved_flex_basis,
            constants,
        );
        let child_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
        let baseline = FlexItemBaseline::from_output(output, child_flow_axes);
        let location = constants.axes.point_from_main_cross(
            item.final_main_location(constants, output.size),
            item.final_cross_location(constants, output.size),
        );
        let scroll_geometry = retained_flex_child_scroll_geometry(
            &style,
            output.size,
            output.content_size,
            item.padding,
            item.border,
            output.scroll_geometry,
        )
        .map_err(|error| flex_child_geometry_error(container_node, item.node, error))?;
        output.scroll_geometry = Some(scroll_geometry);
        tree.set_unrounded(
            item.node,
            NodeOutputOf::<Tree::Scalar> {
                source_index: crate::SourceIndex::new(item.source_index),
                location,
                size: output.size,
                content_size: output.content_size,
                border: item.border,
                padding: item.padding,
                margin: item.margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        );
        final_items.push(FinalFlexItem {
            _node: core::marker::PhantomData,
            source_index: crate::SourceIndex::new(item.source_index),
            output,
            margin: item.margin,
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
    let Some(resolved_flex_basis) = resolved_flex_basis else {
        return;
    };
    let padding_border = constants
        .axes
        .main_size((item.padding + item.border).sum_axes());
    if item.flex_grow_factor == S::ZERO
        && resolved_flex_basis <= padding_border
        && tree.child_count(item.node) == 0
        && constants.axes.main_size(output.size) <= item.flex_basis
        && constants.axes.main_size(output.content_size) <= item.flex_basis
        && constants.axes.main_size(item.target_size) <= padding_border
    {
        output.content_size = constants.axes.with_main_size(
            output.content_size,
            constants.axes.main_size(item.target_size),
        );
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
    let authored = resolve_preferred_size::<_, _, M>(
        item.node,
        &style.size,
        constants.node_inner_size,
        SizingAlgorithm::Flex,
    )?
    .apply_aspect_ratio(style.aspect_ratio)
    .add_optional(box_sizing_adjustment);

    let mut known = Size::new(Some(item.target_size.width), Some(item.target_size.height));
    if constants.axes.main_requested_axis() == RequestedAxis::Horizontal {
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
    final_scroll_box: CanonicalScrollBoxOf<Tree::Scalar>,
) -> FlexChildContributionsResult<Tree, M>
where
    Tree: Compute<M>,
{
    let settled_constants = constants.with_final_scroll_box(final_scroll_box);
    let constants = &settled_constants;
    let children = tree.children(node).collect::<Vec<_>>();
    let mut contributions = Vec::new();
    let inset_relative_size = final_scroll_box.scrollport().size().map(Some);
    let available = final_scroll_box
        .scrollport()
        .size()
        .map(AvailableOf::definite);

    for (source_index, child) in children.into_iter().enumerate() {
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
        let min_size = resolve_minimum_size::<_, _, M>(
            child,
            &style.min_size,
            inset_relative_size,
            SizingAlgorithm::Positioned,
        )?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let max_size = resolve_maximum_size::<_, _, M>(
            child,
            &style.max_size,
            inset_relative_size,
            SizingAlgorithm::Positioned,
        )?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let mut known_size = resolve_preferred_size::<_, _, M>(
            child,
            &style.size,
            inset_relative_size,
            SizingAlgorithm::Positioned,
        )?
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
                ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
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
        let scroll_geometry = retained_flex_child_scroll_geometry(
            &style,
            final_size,
            output.content_size,
            padding,
            border,
            output.scroll_geometry,
        )
        .map_err(|error| flex_child_geometry_error(node, child, error))?;

        tree.set_unrounded(
            child,
            NodeOutputOf::<Tree::Scalar> {
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
        contributions.push(FlexChildContribution {
            source_index: crate::SourceIndex::new(source_index),
            location,
            margin,
            geometry: scroll_geometry,
            in_flow: false,
        });
    }
    Ok(contributions)
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
    for (source_index, child) in children.into_iter().enumerate() {
        if tree.node_input(child).display != super::Display::None {
            continue;
        }

        tree.set_unrounded(
            child,
            NodeOutputOf::with_source_index(crate::SourceIndex::new(source_index)),
        );
        tree.compute_child(
            child,
            ComputeInputOf::hidden(ContainingLayoutContext::new(
                containing_flow_axes,
                ParentFormattingContext::Flex,
            )),
        )?;
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
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    let main_start = constants.axes.normal_main_start_edge(inset);
    let main_end = constants.axes.normal_main_end_edge(inset);
    let main_start_scrollbar = scrollbar_gutter_at_side(
        constants
            .axes
            .normal_axis_start_side(constants.axes.main_logical_axis()),
        constants.scrollbar_gutter,
    );
    let main_end_scrollbar = scrollbar_gutter_at_side(
        constants
            .axes
            .normal_axis_end_side(constants.axes.main_logical_axis()),
        constants.scrollbar_gutter,
    );
    let main = if let Some(start) = main_start {
        constants
            .axes
            .normal_main_start_edge(constants.effective_border)
            + main_start_scrollbar
            + start
            + constants.axes.normal_main_start_edge(margin)
    } else if let Some(end) = main_end {
        constants.axes.main_size(container)
            - constants
                .axes
                .normal_main_end_edge(constants.effective_border)
            - main_end_scrollbar
            - constants.axes.main_size(size)
            - end
            - constants.axes.normal_main_end_edge(margin)
    } else {
        absolute_main_alignment(size, margin, container, constants)
    };
    let main = if main_start.is_some() || main_end.is_some() {
        constants.axes.main_position_from_normal_start(
            container,
            main,
            constants.axes.main_size(size),
        )
    } else {
        constants.axes.main_position_from_start(
            container,
            S::ZERO,
            main,
            constants.axes.main_size(size),
            S::ZERO,
        )
    };
    let cross_start = constants.axes.normal_cross_start_edge(inset);
    let cross_end = constants.axes.normal_cross_end_edge(inset);
    let cross_start_scrollbar = scrollbar_gutter_at_side(
        constants
            .axes
            .normal_axis_start_side(constants.axes.cross_logical_axis()),
        constants.scrollbar_gutter,
    );
    let cross_end_scrollbar = scrollbar_gutter_at_side(
        constants
            .axes
            .normal_axis_end_side(constants.axes.cross_logical_axis()),
        constants.scrollbar_gutter,
    );
    let cross = if let Some(start) = cross_start {
        constants
            .axes
            .normal_cross_start_edge(constants.effective_border)
            + cross_start_scrollbar
            + start
            + constants.axes.normal_cross_start_edge(margin)
    } else if let Some(end) = cross_end {
        constants.axes.cross_size(container)
            - constants
                .axes
                .normal_cross_end_edge(constants.effective_border)
            - cross_end_scrollbar
            - constants.axes.cross_size(size)
            - end
            - constants.axes.normal_cross_end_edge(margin)
    } else {
        absolute_cross_alignment(size, margin, container, align_self, constants)
    };
    let cross = if cross_start.is_some() || cross_end.is_some() {
        constants.axes.cross_position_from_normal_start(
            container,
            cross,
            constants.axes.cross_size(size),
        )
    } else {
        constants.axes.cross_position_from_start(
            container,
            S::ZERO,
            cross,
            constants.axes.cross_size(size),
            S::ZERO,
        )
    };

    constants.axes.point_from_main_cross(main, cross)
}

fn scrollbar_gutter_at_side<S: LayoutScalar>(side: PhysicalSide, gutter: Edges<S>) -> S {
    match side {
        PhysicalSide::Top => gutter.top,
        PhysicalSide::Right => gutter.right,
        PhysicalSide::Bottom => gutter.bottom,
        PhysicalSide::Left => gutter.left,
    }
}

fn absolute_main_alignment<S: LayoutScalar>(
    size: Size<S>,
    margin: Edges<S>,
    container: Size<S>,
    constants: &Constants<S>,
) -> S {
    let content_start = constants.axes.main_start_edge(constants.content_box_inset);
    let content_end = constants.axes.main_end_edge(constants.content_box_inset);
    let free_space = constants.axes.main_size(container)
        - content_start
        - content_end
        - constants.axes.main_size(size);
    let alignment = constants.justify_content.safe_fallback(free_space);
    let start_edge = || content_start + constants.axes.main_start_edge(margin);
    let end_edge = || {
        constants.axes.main_size(container)
            - content_end
            - constants.axes.main_size(size)
            - constants.axes.main_end_edge(margin)
    };
    match alignment {
        AlignContent::Start => {
            if constants.axes.main_is_reversed() {
                end_edge()
            } else {
                start_edge()
            }
        }
        AlignContent::End => {
            if constants.axes.main_is_reversed() {
                start_edge()
            } else {
                end_edge()
            }
        }
        AlignContent::Stretch | AlignContent::SpaceBetween | AlignContent::FlexStart => {
            start_edge()
        }
        AlignContent::FlexEnd => end_edge(),
        AlignContent::Center | AlignContent::SpaceAround | AlignContent::SpaceEvenly => {
            (constants.axes.main_size(container) + content_start
                - content_end
                - constants.axes.main_size(size)
                + constants.axes.main_start_edge(margin)
                - constants.axes.main_end_edge(margin))
                / S::from_f64(2.0)
        }
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
    let content_start = constants.axes.cross_start_edge(constants.content_box_inset);
    let content_end = constants.axes.cross_end_edge(constants.content_box_inset);
    let free_space = constants.axes.cross_size(container)
        - content_start
        - content_end
        - constants.axes.cross_size(size);
    let start_edge = || content_start + constants.axes.cross_start_edge(margin);
    let end_edge = || {
        constants.axes.cross_size(container)
            - content_end
            - constants.axes.cross_size(size)
            - constants.axes.cross_end_edge(margin)
    };
    match align_self.safe_fallback(free_space) {
        AlignItems::Start => {
            if constants.axes.cross_is_reversed() {
                end_edge()
            } else {
                start_edge()
            }
        }
        AlignItems::End | AlignItems::LastBaseline => {
            if constants.axes.cross_is_reversed() {
                start_edge()
            } else {
                end_edge()
            }
        }
        AlignItems::FlexStart | AlignItems::Stretch | AlignItems::Baseline => start_edge(),
        AlignItems::FlexEnd => end_edge(),
        AlignItems::Center => {
            (constants.axes.cross_size(container) + content_start
                - content_end
                - constants.axes.cross_size(size)
                + constants.axes.cross_start_edge(margin)
                - constants.axes.cross_end_edge(margin))
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

#[cfg(test)]
mod final_baseline_selection_tests {
    use super::*;

    fn default_flow_axes<S: LayoutScalar>() -> FlowAxes {
        let style = NodeInputOf::<S>::default();
        FlowAxes::new(style.writing_mode, style.direction)
    }

    fn final_item<S: LayoutScalar>(
        align_self: AlignItems,
        location_y: f64,
        baseline_y: f64,
    ) -> FinalFlexItem<(), S> {
        let size = Size::new(S::from_f64(10.0), S::from_f64(10.0));
        let output = ComputeOutputOf::from_sizes_and_baselines(
            size,
            size,
            BaselinesOf::first(Point::new(None, Some(S::from_f64(baseline_y)))),
        );
        FinalFlexItem {
            _node: core::marker::PhantomData,
            source_index: crate::SourceIndex::ZERO,
            output,
            margin: Edges::ZERO,
            align_self,
            baseline: FlexItemBaseline::from_output(output, default_flow_axes::<S>()),
            location: Point::new(S::ZERO, S::from_f64(location_y)),
        }
    }

    fn constants<S: LayoutScalar>(flex_direction: FlexDirection) -> Constants<S> {
        let flow_axes = default_flow_axes::<S>();
        Constants {
            flow_axes,
            axes: FlexAxes::new(flow_axes, flex_direction, FlexWrap::NoWrap),
            node_outer_size: Size::NONE,
            node_inner_size: Size::NONE,
            min_outer_size: Size::NONE,
            max_outer_size: Size::NONE,
            max_inner_size: Size::NONE,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            effective_border: Edges::ZERO,
            padding_border_size: Size::ZERO,
            scrollbar_gutter: Edges::ZERO,
            content_box_inset: Edges::ZERO,
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            gap: Size::ZERO,
            align_items: AlignItems::Stretch,
            authored_align_content: None,
            align_content: AlignContent::Stretch,
            authored_justify_content: None,
            justify_content: AlignContent::FlexStart,
            wraps: false,
            available: Size::splat(AvailableOf::MAX_CONTENT),
            available_main: AvailableOf::MAX_CONTENT,
        }
    }

    fn assert_final_baselines_follow_main_logical_axis<S: LayoutScalar>() {
        let items = [
            final_item::<S>(AlignItems::Stretch, 10.0, 1.0),
            final_item::<S>(AlignItems::Baseline, 20.0, 2.0),
            final_item::<S>(AlignItems::Stretch, 30.0, 3.0),
        ];
        let lines = [FlexLine {
            start: 0,
            end: items.len(),
            main_size: S::ZERO,
            cross_size: S::ZERO,
            offset_cross: S::ZERO,
        }];
        for (flex_direction, first, last) in [
            (FlexDirection::Row, 22.0, 22.0),
            (FlexDirection::Column, 11.0, 33.0),
        ] {
            let constants = constants::<S>(flex_direction);
            assert_eq!(
                first_final_vertical_baseline(&items, &lines, &constants),
                Some(Point::new(None, Some(S::from_f64(first)))),
                "the first final baseline follows the resolved main logical axis"
            );
            assert_eq!(
                last_final_vertical_baseline(&items, &lines, &constants),
                Some(Point::new(None, Some(S::from_f64(last)))),
                "the last final baseline follows the resolved main logical axis"
            );
        }
    }

    #[test]
    fn logical_flex_placement_final_baselines_follow_main_logical_axis_for_f32() {
        assert_final_baselines_follow_main_logical_axis::<f32>();
    }

    #[test]
    fn logical_flex_placement_final_baselines_follow_main_logical_axis_for_f64() {
        assert_final_baselines_follow_main_logical_axis::<f64>();
    }
}
