use super::alignment::FlexItemBaseline;
use super::flexible_lengths::clamp_main_size_axes;
use super::input::{FlexItemProjection, flex_item_order_permutation, with_flex_item_projection};
use super::scroll::{retain_flex_scroll_geometry, retained_flex_child_scroll_geometry};
use super::{Constants, resolve_auto_optional, resolve_auto_or_zero, resolve_length_or_zero};
use crate::error::{SizingAlgorithm, layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::PhysicalAxis;
use crate::layout_math::{MaxBeforeMinScalarClampExt, OptionalSizeExt};
use crate::sizing::resolve::{
    EdgesResultExt, ResolvedFlexBasis, SizeResultExt, resolve_flex_basis, resolve_maximum_optional,
    resolve_minimum_optional, resolve_preferred_optional,
};
use crate::{
    AlignItems, AvailableOf, BoxSizing, Compute, ComputeInputOf, ComputeOutputOf, ComputedOverflow,
    ContainingLayoutContext, Edges, FlexItemCollapse, LayoutResultOf, LayoutScalar, LengthAutoOf,
    NodeOutputOf, ParentFormattingContext, Point, Position, RequestedAxis, RunMode, Size,
    SizingMode, Traverse,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct CollectedFlexItem<Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) source_index: usize,
    pub(super) collapse: FlexItemCollapse,
    pub(super) size: Size<Option<S>>,
    pub(super) initial_output: ComputeOutputOf<S>,
    pub(super) flex_basis: S,
    pub(super) flex_basis_is_definite: bool,
    pub(super) flex_basis_uses_content: bool,
    pub(super) intrinsic_flex_basis: Option<AvailableOf<S>>,
    pub(super) hypothetical_main_size: S,
    pub(super) max_content_main_size: S,
    pub(super) hypothetical_size: Size<S>,
    pub(super) cross_size_is_auto: bool,
    pub(super) automatic_min_main_size: Option<S>,
    pub(super) min_size: Size<Option<S>>,
    pub(super) max_size: Size<Option<S>>,
    pub(super) min_cross_size: Option<S>,
    pub(super) max_cross_size: Option<S>,
    pub(super) margin: Edges<S>,
    pub(super) margin_is_auto: Edges<bool>,
    pub(super) inset: Edges<Option<S>>,
    pub(super) padding: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) overflow: ComputedOverflow,
    pub(super) align_self: AlignItems,
    pub(super) initial_baseline: FlexItemBaseline<S>,
    pub(super) flex_grow_factor: S,
    pub(super) flex_shrink_factor: S,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ResolvedFlexItem<Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) source_index: usize,
    pub(super) size: Size<Option<S>>,
    pub(super) initial_output: ComputeOutputOf<S>,
    pub(super) flex_basis: S,
    pub(super) intrinsic_flex_basis: Option<AvailableOf<S>>,
    pub(super) hypothetical_main_size: S,
    pub(super) max_content_main_size: S,
    pub(super) target_size: Size<S>,
    pub(super) cross_size_is_auto: bool,
    pub(super) automatic_min_main_size: Option<S>,
    pub(super) min_size: Size<Option<S>>,
    pub(super) max_size: Size<Option<S>>,
    pub(super) min_cross_size: Option<S>,
    pub(super) max_cross_size: Option<S>,
    pub(super) margin: Edges<S>,
    pub(super) margin_is_auto: Edges<bool>,
    pub(super) inset: Edges<Option<S>>,
    pub(super) padding: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) align_self: AlignItems,
    pub(super) baseline: FlexItemBaseline<S>,
    pub(super) flex_grow_factor: S,
    pub(super) flex_shrink_factor: S,
    pub(super) offset_main: S,
    pub(super) offset_cross: S,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FinalFlexItem<Node, S: LayoutScalar> {
    pub(super) _node: core::marker::PhantomData<Node>,
    pub(super) source_index: crate::SourceIndex,
    pub(super) output: ComputeOutputOf<S>,
    pub(super) margin: Edges<S>,
    pub(super) align_self: AlignItems,
    pub(super) baseline: FlexItemBaseline<S>,
    pub(super) location: Point<S>,
}

impl<Node, S: LayoutScalar> CollectedFlexItem<Node, S> {
    pub(super) fn is_collapsed(&self) -> bool {
        self.collapse == FlexItemCollapse::Collapsed
    }
}

impl<Node, S: LayoutScalar> From<CollectedFlexItem<Node, S>> for ResolvedFlexItem<Node, S> {
    fn from(item: CollectedFlexItem<Node, S>) -> Self {
        Self {
            node: item.node,
            source_index: item.source_index,
            size: item.size,
            initial_output: item.initial_output,
            flex_basis: item.flex_basis,
            intrinsic_flex_basis: item.intrinsic_flex_basis,
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
pub(super) fn collect_items<Tree, M>(
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
    let mut order_entries = Vec::with_capacity(children.len());
    for (source_index, child) in children.into_iter().enumerate() {
        let collected =
            with_flex_item_projection::<Tree, M, _>(tree, child, |tree, child_style| {
                if child_style.common.position == Position::Absolute
                    || child_style.display == crate::Display::None
                {
                    return Ok(None);
                }

                let item_order = child_style.item_order;
                let item =
                    build_item(tree, child, source_index, &child_style, constants, run_mode)?;
                Ok(Some((item_order, item)))
            })?;
        if let Some((item_order, item)) = collected {
            order_entries.push((item_order, crate::SourceIndex::new(item.source_index)));
            items.push(item);
        }
    }
    let permutation = flex_item_order_permutation(&order_entries);
    let mut items_by_source = items
        .into_iter()
        .map(|item| (crate::SourceIndex::new(item.source_index), item))
        .collect::<std::collections::BTreeMap<_, _>>();
    Ok(permutation
        .into_iter()
        .map(|source_index| {
            items_by_source
                .remove(&source_index)
                .expect("the flex order permutation contains every collected source index")
        })
        .collect())
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex item builder preserves node, scalar, and provider error types"
)]
fn build_item<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    source_index: usize,
    style: &FlexItemProjection<'_, Tree::Scalar>,
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
            *style.common.padding,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *style.common.border,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let margin = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *style.common.margin,
            constants.node_inner_size,
            resolve_auto_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let margin_is_auto = style.common.margin.map(LengthAutoOf::is_auto);
    let inset = style
        .common
        .inset
        .zip_size(constants.node_inner_size, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let padding_border = padding + border;
    let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let authored_size = Size::new(
        resolve_preferred_optional(
            &style.common.size.width,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
            constants.node_inner_size.width,
            true,
        ),
        resolve_preferred_optional(
            &style.common.size.height,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
            constants.node_inner_size.height,
            true,
        ),
    )
    .transpose_with_node(tree, node)?
    .apply_aspect_ratio(*style.common.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let flex_basis_resolution = resolve_flex_basis(
        style.flex_basis,
        constants.axes.main_physical_axis(),
        constants.axes.main_size(constants.node_inner_size),
    )
    .map_err(|error| sizing_resolution_error(node, error))?;
    let flex_basis_uses_content = match flex_basis_resolution {
        ResolvedFlexBasis::Auto => constants.axes.main_size(authored_size).is_none(),
        ResolvedFlexBasis::Content => true,
        ResolvedFlexBasis::MinContent
        | ResolvedFlexBasis::MaxContent
        | ResolvedFlexBasis::Definite(_) => false,
    };
    let intrinsic_flex_basis = match flex_basis_resolution {
        ResolvedFlexBasis::MinContent => Some(AvailableOf::MIN_CONTENT),
        ResolvedFlexBasis::MaxContent => Some(AvailableOf::MAX_CONTENT),
        ResolvedFlexBasis::Auto | ResolvedFlexBasis::Content | ResolvedFlexBasis::Definite(_) => {
            None
        }
    };
    let resolved_flex_basis = match flex_basis_resolution {
        ResolvedFlexBasis::Auto => constants.axes.main_size(authored_size),
        ResolvedFlexBasis::Content
        | ResolvedFlexBasis::MinContent
        | ResolvedFlexBasis::MaxContent => None,
        ResolvedFlexBasis::Definite(flex_basis) => Some({
            let padding_border = constants.axes.main_size(padding_border.sum_axes());
            if style.common.box_sizing == BoxSizing::ContentBox {
                flex_basis + padding_border
            } else {
                flex_basis.max(padding_border)
            }
        }),
    };
    let size = if flex_basis_uses_content || intrinsic_flex_basis.is_some() {
        constants.axes.with_main_size(authored_size, None)
    } else {
        match resolved_flex_basis {
            Some(flex_basis) => constants
                .axes
                .with_main_size(authored_size, Some(flex_basis)),
            None => authored_size,
        }
    };
    let raw_min_size = Size::new(
        resolve_minimum_optional(
            &style.common.min_size.width,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
            constants.node_inner_size.width,
            true,
        ),
        resolve_minimum_optional(
            &style.common.min_size.height,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
            constants.node_inner_size.height,
            true,
        ),
    )
    .transpose_with_node(tree, node)?;
    let raw_max_size = Size::new(
        resolve_maximum_optional(
            &style.common.max_size.width,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
            constants.node_inner_size.width,
            true,
        ),
        resolve_maximum_optional(
            &style.common.max_size.height,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
            constants.node_inner_size.height,
            true,
        ),
    )
    .transpose_with_node(tree, node)?;
    let min_size = raw_min_size
        .apply_aspect_ratio(*style.common.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let max_size = raw_max_size
        .apply_aspect_ratio(*style.common.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let align_self = style.align_self.unwrap_or(constants.align_items);
    let cross_size_is_auto = constants
        .axes
        .cross_size(style.common.size.clone())
        .is_auto();
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
    let use_content_sizing_for_base = intrinsic_flex_basis.is_some()
        || flex_basis_uses_content && style.display == crate::Display::Block;
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
    let child_available = if let Some(intrinsic_flex_basis) = intrinsic_flex_basis {
        constants
            .axes
            .with_main_size(available, intrinsic_flex_basis)
    } else if use_content_sizing_for_base {
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
        )
        .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
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
    } else if intrinsic_flex_basis.is_some() {
        constants.axes.main_size(output.size)
    } else if let Some(ratio) = *style.common.aspect_ratio {
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
                )
                .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
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
    } else if flex_basis_uses_content || intrinsic_flex_basis.is_some() {
        constants.axes.main_size(output.content_size)
    } else {
        constants
            .axes
            .main_size(output.content_size)
            .max(authored_main_size.unwrap_or(Tree::Scalar::ZERO))
    };
    let max_content_main_size = intrinsic_main_size
        .clamp_max_before_min_optional(
            constants.axes.main_size(min_size),
            constants.axes.main_size(max_size),
        )
        .max(padding_border_main);
    let mut target_size = constants
        .axes
        .with_main_size(output.size, hypothetical_main_size);
    if align_self != AlignItems::Stretch
        && cross_size_is_auto
        && let Some(ratio) = *style.common.aspect_ratio
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
            .clamp_max_before_min_optional(
                constants.axes.cross_size(raw_min_size),
                constants.axes.cross_size(raw_max_size),
            )
            .max(constants.axes.cross_size(padding_border.sum_axes())),
    );
    let child_flow_axes = style.common.flow_axes;
    let baseline = FlexItemBaseline::from_output(output, child_flow_axes);

    Ok(CollectedFlexItem {
        node,
        source_index,
        collapse: style.collapse,
        size: authored_size,
        initial_output: output,
        flex_basis,
        flex_basis_is_definite: resolved_flex_basis.is_some(),
        flex_basis_uses_content,
        intrinsic_flex_basis,
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
        overflow: style.common.overflow,
        align_self,
        initial_baseline: baseline,
        flex_grow_factor: style.flex_grow.get(),
        flex_shrink_factor: style.flex_shrink.get(),
    })
}

fn automatic_min_main_size<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    style: &FlexItemProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    box_sizing_adjustment: Size<Tree::Scalar>,
    child_known: Size<Option<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Option<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    if !constants
        .axes
        .main_size(style.common.min_size.clone())
        .is_auto()
        || flex_automatic_minimum_is_zero(style.common.overflow)
    {
        return Ok(None);
    }
    let authored_size = Size::new(
        resolve_preferred_optional(
            &style.common.size.width,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
            constants.node_inner_size.width,
            true,
        ),
        resolve_preferred_optional(
            &style.common.size.height,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
            constants.node_inner_size.height,
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
            constants.node_inner_size.width,
            true,
        ),
        resolve_minimum_optional(
            &style.common.min_size.height,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
            constants.node_inner_size.height,
            true,
        ),
    )
    .transpose_with_node(tree, node)?
    .apply_aspect_ratio(*style.common.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let resolved_max_size = Size::new(
        resolve_maximum_optional(
            &style.common.max_size.width,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
            constants.node_inner_size.width,
            true,
        ),
        resolve_maximum_optional(
            &style.common.max_size.height,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
            constants.node_inner_size.height,
            true,
        ),
    )
    .transpose_with_node(tree, node)?
    .apply_aspect_ratio(*style.common.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let padding = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *style.common.padding,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, node)?;
    let border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *style.common.border,
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
        )
        .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
    )?;

    let mut min_content = constants
        .axes
        .main_size(output.size)
        .clamp_max_before_min_optional(None, constants.axes.main_size(authored_size))
        .clamp_max_before_min_optional(None, constants.axes.main_size(resolved_max_size));
    if let Some(ratio) = *style.common.aspect_ratio
        && let Some(cross) = constants.axes.cross_size(child_known)
    {
        let transferred = constants
            .axes
            .main_size_from_cross_aspect(cross, ratio)
            .clamp_max_before_min_optional(None, constants.axes.main_size(authored_size))
            .clamp_max_before_min_optional(None, constants.axes.main_size(resolved_max_size));
        min_content = if style.common.item_is_replaced {
            min_content.min(transferred)
        } else {
            min_content.max(transferred)
        };
    }
    Ok(Some(
        min_content.max(constants.axes.main_size(padding_border.sum_axes())),
    ))
}

pub(super) fn flex_automatic_minimum_is_zero(overflow: ComputedOverflow) -> bool {
    overflow.x().is_scrollable() || overflow.y().is_scrollable()
}

fn flex_base_known_size<S: LayoutScalar>(
    size: Size<Option<S>>,
    cross_available: AvailableOf<S>,
    style: &FlexItemProjection<'_, S>,
    constants: &Constants<S>,
    margin: Edges<S>,
    margin_is_auto: Edges<bool>,
    align_self: AlignItems,
) -> Size<Option<S>> {
    let mut known = constants.axes.with_main_size(size, None);
    if align_self == AlignItems::Stretch
        && constants
            .axes
            .cross_size(style.common.size.clone())
            .is_auto()
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

pub(super) fn clamp_available<S: LayoutScalar>(
    available: AvailableOf<S>,
    min: Option<S>,
    max: Option<S>,
) -> AvailableOf<S> {
    match available {
        AvailableOf::Definite(value) => {
            AvailableOf::Definite(value.clamp_max_before_min_optional(min, max))
        }
        AvailableOf::MinContent => min.map_or(AvailableOf::MinContent, AvailableOf::Definite),
        AvailableOf::MaxContent => max.map_or(AvailableOf::MaxContent, AvailableOf::Definite),
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex finalizer preserves node, scalar, and provider error types"
)]
pub(super) fn final_layout<Tree, M>(
    tree: &mut Tree,
    container_node: <Tree as Traverse>::Node,
    collected_items: &[CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
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
    for item in collected_items.iter().filter(|item| item.is_collapsed()) {
        tree.set_unrounded(
            item.node,
            NodeOutputOf::with_source_index(crate::SourceIndex::new(item.source_index)),
        );
    }

    let mut final_items = Vec::with_capacity(items.len());
    for item in items {
        let (mut output, baseline) =
            with_flex_item_projection::<Tree, M, _>(tree, item.node, |tree, style| {
                let known = final_item_size::<Tree, M>(tree, item, &style, constants)?;
                let mut output = tree.compute_child(
                    item.node,
                    ComputeInputOf::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        known,
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
                            constants
                                .axes
                                .cross_size(constants.node_inner_size)
                                .map(AvailableOf::definite)
                                .unwrap_or(AvailableOf::MAX_CONTENT),
                        ),
                    )
                    .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
                )?;
                let resolved_flex_basis = match resolve_flex_basis(
                    style.flex_basis,
                    constants.axes.main_physical_axis(),
                    constants.axes.main_size(constants.node_inner_size),
                )
                .map_err(|error| sizing_resolution_error(item.node, error))?
                {
                    ResolvedFlexBasis::Definite(value) => Some(value),
                    ResolvedFlexBasis::Auto
                    | ResolvedFlexBasis::Content
                    | ResolvedFlexBasis::MinContent
                    | ResolvedFlexBasis::MaxContent => None,
                };
                suppress_padding_floor_flex_basis_content_overflow(
                    tree,
                    item,
                    &mut output,
                    resolved_flex_basis,
                    constants,
                );
                let baseline = FlexItemBaseline::from_output(output, style.common.flow_axes);
                Ok((output, baseline))
            })?;
        let location = constants.axes.point_from_main_cross(
            item.final_main_location(constants, output.size),
            item.final_cross_location(constants, output.size),
        );
        let scroll_geometry = super::input::with_flex_item_scroll_projections::<Tree, M, _>(
            tree,
            item.node,
            |scroll_box, scroll_target| {
                retained_flex_child_scroll_geometry(
                    scroll_box,
                    scroll_target,
                    output.size,
                    output.content_size,
                    item.padding,
                    item.border,
                    output.scroll_geometry,
                )
            },
        )
        .map_err(|error| layout_child_geometry_error(container_node, item.node, error))?;
        output = retain_flex_scroll_geometry(output, scroll_geometry);
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
    style: &FlexItemProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Option<Tree::Scalar>>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let padding = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *style.common.padding,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, item.node)?;
    let border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *style.common.border,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, item.node)?;
    let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::<Tree::Scalar>::ZERO
    };
    let authored = Size::new(
        resolve_preferred_optional(
            &style.common.size.width,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
            constants.node_inner_size.width,
            true,
        ),
        resolve_preferred_optional(
            &style.common.size.height,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
            constants.node_inner_size.height,
            true,
        ),
    )
    .transpose_with_node(tree, item.node)?
    .apply_aspect_ratio(*style.common.aspect_ratio)
    .add_optional(box_sizing_adjustment);

    let mut known = Size::new(Some(item.target_size.width), Some(item.target_size.height));
    if constants.axes.main_requested_axis() == RequestedAxis::Horizontal {
        if style.common.size.height.depends_on_basis() {
            known.height = authored.height.or(known.height);
        }
    } else if style.common.size.width.depends_on_basis() {
        known.width = authored.width.or(known.width);
    }
    Ok(known)
}
