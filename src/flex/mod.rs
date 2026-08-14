use super::{
    AlignContent, AlignItems, AspectRatioOf, AvailableOf, BaselinesOf, BoxSizing, Compute,
    ComputeInputOf, ComputeOutputOf, ContainingLayoutContext, Direction, Edges, FlexDirection,
    FlexWrap, LayoutResultOf, LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionStatus,
    ParentFormattingContext, Point, RequestedAxis, RunMode, Size, SizingMode, Traverse,
};
use crate::error::{SizingAlgorithm, layout_own_geometry_error};
use crate::geometry::{FlowAxes, LogicalAxis, PhysicalAxis, PhysicalProgression, PhysicalSide};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt,
    OptionalMinimumSizeFloorExt, OptionalSizeExt, OptionalSizeMaxExt, UncheckedOptionalSizeSubExt,
    resolution_optional, resolution_or_zero, resolve_containing_padding_border,
};
use crate::scroll::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, ScrollOriginAxes, ScrollOriginProgression,
    canonical_scroll_box_from_source,
};
use crate::sizing::resolve::{
    EdgesResultExt, SizeResultExt, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};

mod absolute;
mod alignment;
mod flexible_lengths;
mod input;
mod intrinsic;
mod items;
mod lines;
mod scroll;

use absolute::{layout_absolute_children, layout_hidden_children};
use alignment::{
    align_items_on_cross_axis, align_lines_on_cross_axis, alignment_fallback, alignment_offset,
    first_final_vertical_baseline, first_vertical_baseline, last_final_vertical_baseline,
    last_vertical_baseline, line_cross_size, line_free_space, resolve_main_axis_auto_margins,
    stretch_lines_on_cross_axis,
};
use flexible_lengths::resolve_flexible_lengths;
use input::FlexContainerProjection;
use intrinsic::{
    intrinsic_content_main_size, resolved_cross_layout_constants, resolved_layout_constants,
};
use items::{
    CollectedFlexItem, FinalFlexItem, ResolvedFlexItem, clamp_available, collect_items,
    final_layout,
};
use lines::{CollapsedFlexStruts, FlexLine, FlexLineCollectionRound, collect_flex_lines};
use scroll::{
    flex_container_scroll_box, flex_container_scroll_geometry, flex_scroll_contributions,
    retain_flex_scroll_geometry,
};

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
    let constants =
        self::input::with_flex_container_projection::<Tree, M, _>(tree, node, |style| {
            self::input::with_flex_scroll_projections::<Tree, M, _>(tree, node, |scroll_box, _| {
                Constants::new::<Tree, M>(tree, node, &style, scroll_box, input)
            })
        })?;
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
        resolve_flex_round(tree, node, input, &constants, second_items, second_lines)?
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
            &constants,
            collected_items.clone(),
            lines,
        )?
    };
    let container_sizes = container_sizes(input, &layout_constants, &resolved_items, &lines);
    let final_scroll_box = if input.run_mode().is_perform_layout() {
        Some(self::input::with_flex_scroll_projections::<Tree, M, _>(
            tree,
            node,
            |scroll_box, _| {
                flex_container_scroll_box::<_, S, M>(
                    node,
                    input.run_mode(),
                    scroll_box,
                    &layout_constants,
                    container_sizes.output,
                )
            },
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
        let scroll_geometry = self::input::with_flex_scroll_projections::<Tree, M, _>(
            tree,
            node,
            |scroll_box_projection, scroll_target_projection| {
                flex_container_scroll_geometry::<_, S, M>(
                    node,
                    input.run_mode(),
                    scroll_box_projection,
                    scroll_target_projection,
                    &layout_constants,
                    scroll_box,
                    contributions,
                )
            },
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
    constants: &Constants<Tree::Scalar>,
    mut collected_items: Vec<CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>>,
    mut lines: Vec<FlexLine<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ResolvedFlexRound<Tree>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let mut layout_constants =
        resolved_layout_constants(tree, node, input, constants, &mut collected_items, &lines)?;
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
    gap_input: Size<LengthOf<S>>,
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
        style: &FlexContainerProjection<'_, S>,
        scroll_box_projection: crate::scroll::ScrollBoxProjection<'_, S>,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        let flow_axes = style.common.flow_axes;
        let axes = FlexAxes::new(flow_axes, style.flex_direction, style.flex_wrap);
        let (padding, border) = resolve_containing_padding_border(
            input.containing_flow_axes(),
            input.parent(),
            *style.common.padding,
            *style.common.border,
            resolve_length_or_zero,
            |edges| edges.transpose_with_node(tree, node),
        )?;
        let padding_border = (padding + border).sum_axes();
        let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
            padding_border
        } else {
            Size::<S>::ZERO
        };

        let (style_size, min_size, max_size) = match input.sizing_mode() {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::InherentSize => {
                let style_size = Size::new(
                    resolve_preferred_optional(
                        &style.common.size.width,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        true,
                    ),
                    resolve_preferred_optional(
                        &style.common.size.height,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        true,
                    ),
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(*style.common.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let min_size = Size::new(
                    resolve_minimum_optional(
                        &style.common.min_size.width,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        true,
                    ),
                    resolve_minimum_optional(
                        &style.common.min_size.height,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        true,
                    ),
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(*style.common.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let max_size = Size::new(
                    resolve_maximum_optional(
                        &style.common.max_size.width,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        true,
                    ),
                    resolve_maximum_optional(
                        &style.common.max_size.height,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        true,
                    ),
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(*style.common.aspect_ratio)
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
        let mut scroll_box_source = CanonicalScrollBoxSourceOf::from_projection(
            scroll_box_projection,
            Size::ZERO,
            border,
            padding,
            input.settled_auto_scrollbars(),
        );
        let unconstrained_scroll_box_size = padding_border
            + Size::splat(
                scroll_box_source.scrollbar_width.get() + scroll_box_source.scrollbar_width.get(),
            );
        let scroll_box_size = node_outer_size
            .or(input.available().map(AvailableOf::into_option))
            .or(max_size)
            .unwrap_or(unconstrained_scroll_box_size)
            .zip_map(padding_border, |size, minimum| size.max(minimum));
        scroll_box_source.border_box_size = scroll_box_size;
        let scroll_box = canonical_scroll_box_from_source(scroll_box_source)
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
            gap_input: *style.gap,
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
        LengthPercentageOf, NodeInputOf, ParentFormattingContext, RequestedAxis, WritingMode,
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
            let constants = super::input::with_flex_container_projection::<
                _,
                core::convert::Infallible,
                _,
            >(&tree, 7, |projection| {
                super::input::with_flex_scroll_projections::<_, core::convert::Infallible, _>(
                    &tree,
                    7,
                    |scroll_box, _| {
                        Constants::new::<_, core::convert::Infallible>(
                            &tree,
                            7,
                            &projection,
                            scroll_box,
                            input(flow, parent),
                        )
                    },
                )
            })
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
        let result = super::input::with_flex_container_projection::<_, core::convert::Infallible, _>(
            &failing_tree,
            7,
            |projection| {
                super::input::with_flex_scroll_projections::<_, core::convert::Infallible, _>(
                    &failing_tree,
                    7,
                    |scroll_box, _| {
                        Constants::new::<_, core::convert::Infallible>(
                            &failing_tree,
                            7,
                            &projection,
                            scroll_box,
                            input(
                                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                                Size::splat(Some(largest)),
                            ),
                        )
                    },
                )
            },
        );
        let error = match result {
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
