use super::{
    AlignContent, AlignItems, AspectRatioOf, AvailableOf, BaselinesOf, BoxSizing, Compute,
    ComputeInputOf, ComputeOutputOf, ContainingLayoutContext, Direction, Edges, FlexDirection,
    FlexItemCollapse, FlexWrap, LayoutResultOf, LayoutScalar, LengthAutoOf, LengthOf,
    LengthResolutionStatus, NodeInputOf, NodeOutputOf, ParentFormattingContext, Point, Position,
    RequestedAxis, RunMode, Size, SizingMode, Traverse,
};
use crate::error::{
    SizingAlgorithm, layout_child_geometry_error, layout_own_geometry_error,
    sizing_resolution_error,
};
use crate::geometry::{
    FlowAxes, LogicalAxis, LogicalEdgesOf, PhysicalAxis, PhysicalProgression, PhysicalSide,
};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt,
    OptionalMinimumSizeFloorExt, OptionalSizeExt, OptionalSizeMaxExt, UncheckedOptionalSizeSubExt,
    resolution_optional, resolution_or_zero, resolve_containing_padding_border,
};
use crate::scroll::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf,
    CanonicalScrollGeometryErrorOf, CanonicalScrollRangeSeedPolicy, CanonicalScrollSourceBuilderOf,
    ScrollContributionAccumulatorOf, ScrollOriginAxes, ScrollOriginProgression,
    canonical_scroll_box_from_source,
};
use crate::sizing::resolve::{
    EdgesResultExt, SizeResultExt, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};
use crate::sizing::{MaxSizeOf, MinSizeOf, PreferredSizeOf};

mod alignment;
mod flexible_lengths;
mod items;
mod lines;

use alignment::{
    align_items_on_cross_axis, align_lines_on_cross_axis, alignment_fallback, alignment_offset,
    first_final_vertical_baseline, first_vertical_baseline, last_final_vertical_baseline,
    last_vertical_baseline, line_cross_size, line_free_space, resolve_main_axis_auto_margins,
    stretch_lines_on_cross_axis,
};
use flexible_lengths::resolve_flexible_lengths;
use items::{
    CollectedFlexItem, FinalFlexItem, ResolvedFlexItem, clamp_available, collect_items,
    final_layout, flex_automatic_minimum_is_zero,
};
use lines::{CollapsedFlexStruts, FlexLine, FlexLineCollectionRound, collect_flex_lines};

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
    let mut pass_input = input;
    loop {
        let output = compute_flex_inner::<Tree, Tree::Scalar, M>(tree, node, pass_input)?;
        if !input.run_mode().is_perform_layout() {
            return Ok(output);
        }
        let Some(geometry) = output.scroll_geometry else {
            return Ok(output);
        };
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || !crate::scroll::settled_auto_scrollbars_change_available_geometry(
                geometry, next_state,
            )
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?
        {
            return Ok(output);
        }
        pass_input = input.with_settled_auto_scrollbars(next_state);
    }
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
    let has_collapsed_item = collected_items.iter().any(CollectedFlexItem::is_collapsed);
    let (layout_constants, resolved_items, lines) = if has_collapsed_item {
        let first_lines = collect_flex_lines(
            &collected_items,
            &constants,
            FlexLineCollectionRound::Normal,
        );
        let (_, _, first_lines) = resolve_flex_round(
            tree,
            node,
            input,
            &style,
            &constants,
            collected_items.clone(),
            first_lines,
        )?;
        let struts = CollapsedFlexStruts::capture(&collected_items, &first_lines);
        let second_collection_lines = collect_flex_lines(
            &collected_items,
            &constants,
            FlexLineCollectionRound::Collapsed,
        );
        let (second_items, second_lines) =
            struts.prepare_second_round(&collected_items, &second_collection_lines);
        resolve_flex_round(
            tree,
            node,
            input,
            &style,
            &constants,
            second_items,
            second_lines,
        )?
    } else {
        let lines = collect_flex_lines(
            &collected_items,
            &constants,
            FlexLineCollectionRound::Normal,
        );
        resolve_flex_round(
            tree,
            node,
            input,
            &style,
            &constants,
            collected_items.clone(),
            lines,
        )?
    };
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
        let final_items = final_layout(
            tree,
            node,
            &collected_items,
            &resolved_items,
            &layout_constants,
        )?;
        let absolute_contributions = layout_absolute_children(
            tree,
            node,
            &layout_constants,
            final_scroll_box.expect("performed flex layout derives its final canonical box"),
        )?;
        layout_hidden_children(
            tree,
            node,
            layout_constants.axes.flow_axes(),
            layout_constants.settled_auto_scrollbars,
        )?;
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
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
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
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        final_geometry_and_content_size = Some((scroll_geometry, content_size));
    }
    let output = container_output(
        input,
        &layout_constants,
        &resolved_items,
        final_items.as_deref(),
        &lines,
        final_geometry_and_content_size.map(|(_, content_size)| content_size),
        container_sizes,
    );
    Ok(
        final_geometry_and_content_size.map_or(output, |(scroll_geometry, _)| {
            retain_flex_scroll_geometry(output, scroll_geometry)
        }),
    )
}

type ResolvedFlexRound<Tree> = (
    Constants<<Tree as Traverse>::Scalar>,
    Vec<ResolvedFlexItem<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>>,
    Vec<FlexLine<<Tree as Traverse>::Scalar>>,
);

fn resolve_flex_round<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    style: &NodeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    mut collected_items: Vec<CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>>,
    mut lines: Vec<FlexLine<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ResolvedFlexRound<Tree>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let mut layout_constants = resolved_layout_constants(
        tree,
        node,
        input,
        style,
        constants,
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
    Ok((layout_constants, resolved_items, lines))
}

fn retain_flex_scroll_geometry<S: LayoutScalar>(
    output: ComputeOutputOf<S>,
    scroll_geometry: super::ScrollGeometryOf<S>,
) -> ComputeOutputOf<S> {
    ComputeOutputOf {
        scroll_geometry: Some(scroll_geometry),
        ..output
    }
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
    padding_border_size: Size<S>,
    scrollport_inset: Edges<S>,
    non_gutter_box_inset: Edges<S>,
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
        let (padding, border) = resolve_containing_padding_border(
            input.containing_flow_axes(),
            input.parent(),
            style.padding,
            style.border,
            resolve_length_or_zero,
            |edges| edges.transpose_with_node(tree, node),
        )?;
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
            .or(min_max_definite_size
                .or(style_size.clamp_max_before_min_optional(min_size, max_size)))
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
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let scrollport_inset = scroll_box.effective_border() + scroll_box.effective_gutter();
        let non_gutter_box_inset = scroll_box.effective_border() + scroll_box.effective_padding();
        let content_box_inset = scroll_box.content_box_inset();
        let content_box_inset_size = content_box_inset.sum_axes();
        let node_inner_size = node_outer_size
            .sub_optional_unchecked(content_box_inset_size)
            .max_optional(Size::<S>::ZERO.map(Some));
        let max_inner_size = max_size
            .sub_optional_unchecked(content_box_inset_size)
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
            padding_border_size: padding_border,
            scrollport_inset,
            non_gutter_box_inset,
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
        self.scrollport_inset = scroll_box.effective_border() + scroll_box.effective_gutter();
        self.non_gutter_box_inset = scroll_box.effective_border() + scroll_box.effective_padding();
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

    #[cfg(test)]
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
    pub(crate) fn main_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.main_start_side)
    }

    #[must_use]
    pub(crate) fn main_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.main_end_side)
    }

    #[must_use]
    pub(crate) fn cross_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.cross_start_side)
    }

    #[must_use]
    pub(crate) fn cross_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.cross_end_side)
    }

    #[must_use]
    pub(crate) fn normal_main_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_start_side(self.main_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_main_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_end_side(self.main_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_cross_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_start_side(self.cross_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_cross_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_end_side(self.cross_logical_axis))
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

    #[cfg(test)]
    pub(crate) fn set_cross_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.cross_start_side, value);
    }

    #[cfg(test)]
    pub(crate) fn set_cross_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.cross_end_side, value);
    }

    pub(crate) fn set_normal_cross_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(
            edges,
            self.normal_axis_start_side(self.cross_logical_axis),
            value,
        );
    }

    pub(crate) fn set_normal_cross_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(
            edges,
            self.normal_axis_end_side(self.cross_logical_axis),
            value,
        );
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
        if item_count == 0 && line.contains_collapsed_slot {
            line.main_size = Tree::Scalar::ZERO;
            line.cross_size = if single_line {
                constants
                    .axes
                    .cross_size(constants.node_inner_size)
                    .unwrap_or(line.strut_floor)
            } else {
                line.strut_floor
            };
            line.offset_cross = cross_cursor;
            cross_cursor = cross_cursor + line.cross_size + cross_gap;
            continue;
        }
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
        cross_size = Tree::Scalar::max(cross_size, line.strut_floor);

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
            cross.clamp_max_before_min_optional(
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
                .clamp_max_before_min_optional(
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
                        item.intrinsic_flex_basis.unwrap_or_else(|| {
                            constants
                                .axes
                                .main_size(constants.node_inner_size)
                                .map(AvailableOf::definite)
                                .unwrap_or(AvailableOf::MAX_CONTENT)
                        }),
                        available_cross,
                    ),
                )
                .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
            )?;
            item.baseline.refresh(measured);
            constants
                .axes
                .cross_size(measured.size)
                .clamp_max_before_min_optional(
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
        .clamp_max_before_min_optional(constants.min_outer_size, constants.max_outer_size);
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
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
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
    CanonicalScrollSourceBuilderOf::for_node(
        style,
        constants.flow_axes,
        scroll_box.border_box().size(),
        constants.border,
        constants.padding,
        constants.settled_auto_scrollbars,
        constants.axes.scroll_origin_axes(),
    )
    .geometry_from_contributions(contributions, scroll_box.border_box())
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
}

fn retained_flex_child_scroll_geometry<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
) -> Result<super::ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let settled_auto_scrollbars = crate::scroll::SettledAutoScrollbarState::INITIAL;
    let source = match child_compute_geometry {
        Some(ref geometry) => CanonicalRetainedScrollSourceOf::Existing(geometry),
        None => CanonicalRetainedScrollSourceOf::Reconstruct { content_size },
    };
    CanonicalScrollSourceBuilderOf::for_node(
        style,
        flow_axes,
        size,
        border,
        padding,
        settled_auto_scrollbars,
        ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
    )
    .geometry_from_retained_source(
        source,
        CanonicalScrollRangeSeedPolicy::IncludeReservedGutter,
    )
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
        .clamp_max_before_min_optional(
            constants.axes.main_size(constants.min_outer_size),
            constants.axes.main_size(constants.max_outer_size),
        )
        .max(
            constants
                .axes
                .main_size(constants.non_gutter_box_inset.sum_axes()),
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
    let style_preferred = (!item.flex_basis_uses_content && item.intrinsic_flex_basis.is_none())
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
                    .clamp_max_before_min_optional(style_min, style_max)
            } else {
                item.max_content_main_size
                    .max(item.flex_basis)
                    .clamp_max_before_min_optional(style_min, style_max)
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
                    )
                    .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
                )?
                .size,
            );

            if constants.axes.main_logical_axis() == LogicalAxis::Inline {
                measured.clamp_max_before_min_optional(style_min, style_max)
            } else {
                measured
                    .max(item.flex_basis)
                    .clamp_max_before_min_optional(style_min, style_max)
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
        .clamp_max_before_min_optional(
            constants.axes.cross_size(constants.min_outer_size),
            constants.axes.cross_size(constants.max_outer_size),
        )
        .max(
            constants
                .axes
                .cross_size(constants.non_gutter_box_inset.sum_axes()),
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
    let absolute_containing_size = final_scroll_box.scrollport().size();
    let inset_relative_size = absolute_containing_size.map(Some);
    let available = absolute_containing_size.map(AvailableOf::definite);

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
                .clamp_max_before_min_optional(min_size, max_size);
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
                .clamp_max_before_min_optional(min_size, max_size);
        }
        known_size = known_size
            .clamp_max_before_min_optional(min_size, max_size)
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
            )
            .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
        )?;
        let final_size = known_size
            .unwrap_or(output.size)
            .clamp_max_before_min_optional(min_size, max_size)
            .max_optional(padding_border.sum_axes().map(Some));
        let margin = resolve_absolute_margins(
            margin,
            inset,
            final_size,
            absolute_containing_size,
            constants.flow_axes,
        );
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
        .map_err(|error| layout_child_geometry_error(node, child, error))?;
        let output = retain_flex_scroll_geometry(output, scroll_geometry);

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
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let children = tree.children(node).collect::<Vec<_>>();
    for (source_index, child) in children.into_iter().enumerate() {
        let style = tree.node_input(child);
        let is_collapsed_in_flow = style.position != Position::Absolute
            && style.flex_item_collapse == FlexItemCollapse::Collapsed;
        if style.display != super::Display::None && !is_collapsed_in_flow {
            continue;
        }

        tree.set_unrounded(
            child,
            NodeOutputOf::with_source_index(crate::SourceIndex::new(source_index)),
        );
        tree.compute_child(
            child,
            ComputeInputOf::hidden_in_containing_pass(
                ContainingLayoutContext::new(containing_flow_axes, ParentFormattingContext::Flex),
                settled_auto_scrollbars,
            ),
        )?;
    }
    Ok(())
}

fn resolve_absolute_margins<S: LayoutScalar>(
    margin: Edges<Option<S>>,
    inset: Edges<Option<S>>,
    size: Size<S>,
    containing_size: Size<S>,
    flow_axes: FlowAxes,
) -> Edges<S> {
    let logical_margin = flow_axes.logical_edges(margin);
    let logical_inset = flow_axes.logical_edges(inset);
    let logical_size = flow_axes.logical_size(size);
    let logical_containing_size = flow_axes.logical_size(containing_size);
    let (inline_start, inline_end) = resolve_absolute_axis_margins(
        logical_margin.inline_start,
        logical_margin.inline_end,
        logical_inset.inline_start,
        logical_inset.inline_end,
        logical_size.inline,
        logical_containing_size.inline,
        true,
    );
    let (block_start, block_end) = resolve_absolute_axis_margins(
        logical_margin.block_start,
        logical_margin.block_end,
        logical_inset.block_start,
        logical_inset.block_end,
        logical_size.block,
        logical_containing_size.block,
        false,
    );

    flow_axes.physical_edges(LogicalEdgesOf::new(
        inline_start,
        inline_end,
        block_start,
        block_end,
    ))
}

fn resolve_absolute_axis_margins<S: LayoutScalar>(
    start_margin: Option<S>,
    end_margin: Option<S>,
    start_inset: Option<S>,
    end_inset: Option<S>,
    target_size: S,
    containing_size: S,
    anchor_negative_two_auto_at_start: bool,
) -> (S, S) {
    let fixed_start = start_margin.unwrap_or(S::ZERO);
    let fixed_end = end_margin.unwrap_or(S::ZERO);
    let (Some(start_inset), Some(end_inset)) = (start_inset, end_inset) else {
        return (fixed_start, fixed_end);
    };
    let remaining =
        containing_size - start_inset - end_inset - target_size - fixed_start - fixed_end;

    match (start_margin.is_none(), end_margin.is_none()) {
        (false, false) => (fixed_start, fixed_end),
        (true, false) => (remaining, fixed_end),
        (false, true) => (fixed_start, remaining),
        (true, true) if anchor_negative_two_auto_at_start && remaining < S::ZERO => {
            (S::ZERO, remaining)
        }
        (true, true) => {
            let half = remaining / S::from_usize(2);
            (half, half)
        }
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
    let main = if let Some(start) = main_start {
        constants
            .axes
            .normal_main_start_edge(constants.scrollport_inset)
            + start
            + constants.axes.normal_main_start_edge(margin)
    } else if let Some(end) = main_end {
        constants.axes.main_size(container)
            - constants
                .axes
                .normal_main_end_edge(constants.scrollport_inset)
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
    let cross = if let Some(start) = cross_start {
        constants
            .axes
            .normal_cross_start_edge(constants.scrollport_inset)
            + start
            + constants.axes.normal_cross_start_edge(margin)
    } else if let Some(end) = cross_end {
        constants.axes.cross_size(container)
            - constants
                .axes
                .normal_cross_end_edge(constants.scrollport_inset)
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

#[cfg(test)]
mod fri08_c07_t03_optional_math_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(None, Some(scalar(12.0))).max_optional(Size::new(Some(scalar(9.0)), None)),
            Size::new(None, Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(Some(scalar(4.0)), Some(scalar(12.0)))
                .max_optional(Size::new(Some(scalar(9.0)), Some(scalar(3.0)))),
            Size::new(Some(scalar(9.0)), Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0)).max_optional(Size::new(None, Some(scalar(15.0)))),
            Size::new(scalar(4.0), scalar(15.0))
        );
    }

    #[test]
    fn fri08_c07_t03_optional_math_flex_componentwise_floors_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri08_c07_t03_optional_math_flex_componentwise_floors_preserve_f64() {
        characterize::<f64>();
    }
}

#[cfg(test)]
mod fri06_c13_t06_characterization_tests {
    use super::*;
    use crate::{
        LayoutErrorKindOf, LayoutErrorSiteOf, LayoutInvalidInputOf, LayoutOperation,
        LengthPercentageOf, ParentFormattingContext, RequestedAxis, WritingMode,
    };

    fn input<S: LayoutScalar>(
        containing_flow_axes: FlowAxes,
        parent: Size<Option<S>>,
    ) -> ComputeInputOf<S> {
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::ContentSize,
            RequestedAxis::Both,
            Size::NONE,
            parent,
            crate::ContainingLayoutContext::new(
                containing_flow_axes,
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        )
    }

    fn percentage_edges<S: LayoutScalar>() -> Edges<LengthOf<S>> {
        Edges::new(
            LengthOf::percent(S::from_f64(0.1)),
            LengthOf::percent(S::from_f64(0.2)),
            LengthOf::percent(S::from_f64(0.3)),
            LengthOf::percent(S::from_f64(0.4)),
        )
    }

    fn expected_percentage_edges<S: LayoutScalar>(basis: S) -> Edges<S> {
        Edges::new(
            S::from_f64(0.1) * basis,
            S::from_f64(0.2) * basis,
            S::from_f64(0.3) * basis,
            S::from_f64(0.4) * basis,
        )
    }

    fn characterize_constants<S: LayoutScalar>(largest: S) {
        crate::layout_math::assert_fri06_c13_t06_resolution_policy::<S>(
            resolution_or_zero,
            resolution_optional,
        );

        let border = Edges::new(
            LengthOf::px(S::from_f64(1.0)),
            LengthOf::px(S::from_f64(2.0)),
            LengthOf::px(S::from_f64(3.0)),
            LengthOf::px(S::from_f64(4.0)),
        );
        let expected_border = Edges::new(
            S::from_f64(1.0),
            S::from_f64(2.0),
            S::from_f64(3.0),
            S::from_f64(4.0),
        );
        let style = NodeInputOf {
            display: crate::Display::Flex,
            padding: percentage_edges(),
            border,
            ..NodeInputOf::default()
        };
        let tree = crate::test_support::layout_tree::OracleTreeOf::new().style(7, style.clone());

        for (flow, parent, expected_padding) in [
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(100.0)),
            ),
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(200.0)),
            ),
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(None, Some(S::from_f64(200.0))),
                Edges::ZERO,
            ),
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), None),
                Edges::ZERO,
            ),
        ] {
            let constants = Constants::new::<_, core::convert::Infallible>(
                &tree,
                7,
                &style,
                input(flow, parent),
            )
            .expect("flex constants edge characterization must resolve");
            assert_eq!(constants.padding, expected_padding);
            assert_eq!(constants.border, expected_border);
        }

        let positive_overflow = LengthOf::value(
            LengthPercentageOf::from_coefficients(largest, S::ONE)
                .expect("finite positive overflow coefficients"),
        );
        let negative_overflow = LengthOf::value(
            LengthPercentageOf::from_coefficients(-largest, -S::ONE)
                .expect("finite negative overflow coefficients"),
        );
        let failing_style = NodeInputOf {
            display: crate::Display::Flex,
            padding: Edges::new(
                LengthOf::ZERO,
                LengthOf::ZERO,
                LengthOf::ZERO,
                positive_overflow,
            ),
            border: Edges::new(
                negative_overflow,
                LengthOf::ZERO,
                LengthOf::ZERO,
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };
        let failing_tree =
            crate::test_support::layout_tree::OracleTreeOf::new().style(7, failing_style.clone());
        let error = match Constants::new::<_, core::convert::Infallible>(
            &failing_tree,
            7,
            &failing_style,
            input(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::splat(Some(largest)),
            ),
        ) {
            Ok(_) => panic!("padding failure must precede the distinct border failure"),
            Err(error) => error,
        };
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(7));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                value: S::INFINITY,
            })
        );
    }

    #[test]
    fn fri06_c13_t06_flex_resolution_edges_and_error_order_preserve_f32() {
        characterize_constants::<f32>(f32::MAX);
    }

    #[test]
    fn fri06_c13_t06_flex_resolution_edges_and_error_order_preserve_f64() {
        characterize_constants::<f64>(f64::MAX);
    }
}

#[cfg(test)]
mod fri06_c13_t05_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(Some(scalar(2.0)), Some(scalar(9.0)))
                .sub_optional_unchecked(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(Some(scalar(-3.0)), Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(None, Some(scalar(9.0)))
                .sub_optional_unchecked(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(None, Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_max_before_min_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            Size::new(Some(scalar(5.0)), Some(scalar(5.0))).clamp_max_before_min_optional(
                Size::new(Some(scalar(10.0)), Some(scalar(10.0))),
                Size::new(Some(scalar(3.0)), Some(scalar(3.0))),
            ),
            Size::new(Some(scalar(10.0)), Some(scalar(10.0)))
        );
    }

    #[test]
    fn fri06_c13_t05_flex_unchecked_subtraction_and_clamp_order_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_flex_unchecked_subtraction_and_clamp_order_preserve_f64() {
        characterize::<f64>();
    }
}
