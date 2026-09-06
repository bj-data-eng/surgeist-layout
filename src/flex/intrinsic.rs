use super::items::{
    CollectedFlexItem, ResolvedFlexItem, clamp_available, flex_automatic_minimum_is_zero,
};
use super::lines::FlexLine;
use super::{Constants, resolve_length_or_zero};
use crate::geometry::LogicalAxis;
use crate::layout_math::{MaxBeforeMinScalarClampExt, OptionalSizeExt};
use crate::sizing::resolve::SizeResultExt;
use crate::{
    AlignItems, AvailableOf, Compute, ComputeInputOf, ContainingLayoutContext, LayoutResultOf,
    LayoutScalar, ParentFormattingContext, RunMode, Size, SizingMode, Traverse,
};

pub(super) fn intrinsic_content_main_size<Node, S: LayoutScalar>(
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

pub(super) fn resolved_layout_constants<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
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
    constants.gap = constants
        .gap_input
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
                .map_or(item.flex_base.outer(), |min| {
                    item.flex_base.outer().max(min)
                })
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
        && item.flex_base.outer() <= padding_border
        && tree.child_count(item.node) == 0
        && constants.axes.main_size(item.initial_output.content_size) > item.flex_base.outer();
    let clamping_basis = Some(style_preferred.map_or(item.flex_base.outer(), |preferred| {
        item.flex_base.outer().max(preferred)
    }));
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
        && item.flex_base.outer() <= padding_border
        && style_min.is_none()
        && tree.child_count(item.node) == 0
        && constants.axes.main_size(item.initial_output.size) <= item.flex_base.outer()
        && constants.axes.main_size(item.initial_output.content_size) <= item.flex_base.outer()
    {
        return Ok(item.flex_base.outer() + constants.axes.main_edge_sum(item.margin));
    }

    let cross_available = intrinsic_item_cross_available(input, constants, item);
    let needs_stretched_cross_measure = item.align_self == AlignItems::Stretch
        && constants.axes.cross_size(item.size).is_none()
        && cross_available.into_option().is_some();

    let contribution = match (style_preferred, max_main <= min_main) {
        _ if flex_automatic_minimum_is_zero(item.overflow) => item.flex_base.outer().max(min_main),
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
                    .max(item.flex_base.outer())
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
                    .max(item.flex_base.outer())
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

pub(super) fn resolved_cross_layout_constants<S: LayoutScalar>(
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
