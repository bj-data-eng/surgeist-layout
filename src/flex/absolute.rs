use super::scroll::{
    FlexChildContribution, FlexChildContributionsResult, retain_flex_scroll_geometry,
    retained_flex_child_scroll_geometry,
};
use super::{Constants, resolve_auto_optional, resolve_length_or_zero};
use crate::error::{SizingAlgorithm, layout_child_geometry_error};
use crate::geometry::{FlowAxes, LogicalEdgesOf, PhysicalAxis};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinSizeClampExt, OptionalMinimumSizeFloorExt,
    OptionalSizeExt, OptionalSizeMaxExt,
};
use crate::scroll::CanonicalScrollBoxOf;
use crate::sizing::resolve::{
    EdgesResultExt, SizeResultExt, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};
use crate::{
    AlignContent, AlignItems, AvailableOf, BoxSizing, Compute, ComputeInputOf,
    ContainingLayoutContext, Edges, FlexItemCollapse, LayoutResultOf, LayoutScalar, NodeOutputOf,
    ParentFormattingContext, Point, Position, RequestedAxis, RunMode, Size, SizingMode, Traverse,
};

pub(super) fn layout_absolute_children<Tree, M>(
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
        if style.position != Position::Absolute || style.display == crate::Display::None {
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
        let min_size = Size::new(
            resolve_minimum_optional(
                &style.min_size.width,
                SizingAlgorithm::Positioned,
                PhysicalAxis::Horizontal,
                inset_relative_size.width,
                true,
            ),
            resolve_minimum_optional(
                &style.min_size.height,
                SizingAlgorithm::Positioned,
                PhysicalAxis::Vertical,
                inset_relative_size.height,
                true,
            ),
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let max_size = Size::new(
            resolve_maximum_optional(
                &style.max_size.width,
                SizingAlgorithm::Positioned,
                PhysicalAxis::Horizontal,
                inset_relative_size.width,
                true,
            ),
            resolve_maximum_optional(
                &style.max_size.height,
                SizingAlgorithm::Positioned,
                PhysicalAxis::Vertical,
                inset_relative_size.height,
                true,
            ),
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let mut known_size = Size::new(
            resolve_preferred_optional(
                &style.size.width,
                SizingAlgorithm::Positioned,
                PhysicalAxis::Horizontal,
                inset_relative_size.width,
                true,
            ),
            resolve_preferred_optional(
                &style.size.height,
                SizingAlgorithm::Positioned,
                PhysicalAxis::Vertical,
                inset_relative_size.height,
                true,
            ),
        )
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
            crate::scroll::ScrollBoxProjection::from_node(&style),
            crate::scroll::ScrollTargetProjection::from_node(&style),
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

pub(super) fn layout_hidden_children<Tree, M>(
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
        if style.display != crate::Display::None && !is_collapsed_in_flow {
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
