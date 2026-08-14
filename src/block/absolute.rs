use super::Constants;
use super::scroll::retained_child_scroll_geometry;
use super::sizing::{
    maximum_size, minimum_size, preferred_size, resolve_auto_optional, resolve_length_or_zero,
};
use crate::error::layout_child_geometry_error;
use crate::geometry::{LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalSide};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt,
    OptionalSizeExt, OptionalSizeMaxExt,
};
use crate::scroll::ScrollContributionAccumulatorOf;
use crate::sizing::resolve::{EdgesResultExt, SizeResultExt};
use crate::{
    AvailableOf, BoxSizing, Compute, ComputeInputOf, ContainingLayoutContext, Direction, Edges,
    LayoutInputOf, LayoutResultOf, LayoutScalar, NodeOutputOf, ParentFormattingContext, Point,
    Position, RequestedAxis, RunMode, Size, SizingAlgorithm, SizingMode, Traverse,
};

pub(super) fn absolute_static_position<S: LayoutScalar>(
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

pub(super) fn layout_absolute_children<Tree, S, M>(
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
        if style.position != Position::Absolute || style.display == crate::Display::None {
            continue;
        }

        let padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.padding,
                area_size.map(Some),
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, child)?;
        let border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.border,
                area_size.map(Some),
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, child)?;
        let unresolved_margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.margin,
                area_size.map(Some),
                resolve_auto_optional,
            )
            .transpose_with_node(tree, child)?;
        let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(S::ZERO));
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::ZERO
        };
        let min_size = minimum_size(
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
        let max_size = maximum_size(
            &style.max_size,
            area_size.map(Some),
            SizingAlgorithm::Positioned,
            true,
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let style_size = preferred_size(
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
            .clamp_max_before_min_optional(min_size, max_size);
        let inset = style
            .inset
            .zip_size(area_size.map(Some), resolve_auto_optional)
            .transpose_with_node(tree, child)?;
        if known_size.width.is_none()
            && let (Some(left), Some(right)) = (inset.left, inset.right)
        {
            known_size.width = Some(
                (area_size.width - non_auto_margin.horizontal_sum() - left - right)
                    .max(S::ZERO)
                    .clamp_max_before_min_optional(min_size.width, max_size.width),
            );
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_max_before_min_optional(min_size, max_size);
        }
        if known_size.height.is_none()
            && let (Some(top), Some(bottom)) = (inset.top, inset.bottom)
        {
            known_size.height = Some(
                (area_size.height - non_auto_margin.vertical_sum() - top - bottom)
                    .max(S::ZERO)
                    .clamp_max_before_min_optional(min_size.height, max_size.height),
            );
            known_size = known_size
                .apply_aspect_ratio(style.aspect_ratio)
                .clamp_max_before_min_optional(min_size, max_size);
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
            .clamp_max_before_min_optional(min_size, max_size);
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
            crate::scroll::ScrollBoxProjection::from_node(&style),
            crate::scroll::ScrollTargetProjection::from_node(&style),
            final_size,
            output.content_size,
            padding,
            border,
            output.scroll_geometry,
        )
        .map_err(|error| layout_child_geometry_error(container_node, child, error))?;
        contributions
            .include_current_out_of_flow_geometry(location, margin, scroll_geometry)
            .map_err(|error| layout_child_geometry_error(container_node, child, error))?;
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
