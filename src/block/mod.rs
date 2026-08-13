use std::collections::BTreeMap;

use super::inline::{
    AtomicInlineBoxParticipant, ForcedLineBreakControlOf, InlineBoundaryControlOf,
    InlineControlAlignment, InlineFlowOf, LogicalLineBandQueryResultOf, MixedInlineParticipantOf,
    MixedInlineRunInputOf, ShapedTextParticipantOf, layout_mixed_inline_run_with_band_source,
};
use super::value::{ResolvedLengthAutoOf, UnresolvedLengthReason};
use super::{
    AvailableOf, BaselinesOf, BoxSizing, Clear, CollapsibleMarginOf, Compute, ComputeInputOf,
    ComputeOutputOf, ComputedOverflow, ContainingLayoutContext, Direction, Edges, Float,
    InlineBoundaryInputOf, InlineFragmentOutputOf, LayoutErrorKindOf, LayoutErrorOf,
    LayoutErrorSiteOf, LayoutInputOf, LayoutInvalidInputOf, LayoutOperation, LayoutResultOf,
    LayoutScalar, LengthAutoOf, LineBreakInputOf, NodeInputOf, NodeOutputOf, Overflow,
    ParentFormattingContext, PhysicalBlockMarginCollapseOf, Point, Position, RequestedAxis,
    RunMode, Size, SizingAlgorithm, SizingMode, TextAlign, Traverse, VerticalAlign, WritingMode,
};
use crate::error::{
    AtomicInlineParticipationRoleError, layout_child_geometry_error, layout_own_geometry_error,
};
use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalSide};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinScalarClampExt, OptionalSizeExt,
    OptionalSizeMaxExt, resolve_containing_padding_border,
};
use crate::scroll::{
    CanonicalScrollBoxSourceOf, ScrollContributionAccumulatorOf, UsedOverflow,
    canonical_scroll_box_from_source,
};
use crate::sizing::resolve::{EdgesResultExt, SizeResultExt};

#[cfg(test)]
use super::LengthOf;

mod absolute;
mod floats;
mod in_flow;
mod inline_run;
mod scroll;
mod sizing;

use absolute::layout_absolute_children;
use scroll::{finish_scroll_geometry, prepare_scroll_contributions};
use sizing::{
    BlockOptionalSizeSubExt, maximum_size, minimum_size, preferred_size, resolve_auto_optional,
    resolve_length_or_zero,
};

#[cfg(test)]
pub(crate) use in_flow::resolve_logical_in_flow_margin;
use in_flow::{
    InFlowPassContext, layout_in_flow_children, normal_flow_children_can_establish_baseline,
};

pub(crate) use floats::FloatExclusions;
#[cfg(test)]
pub(crate) use floats::FloatLedgerSide;
use floats::{
    BfcBandCandidate, FloatBand, FloatIntrinsics, InheritedFloatExclusions, PendingFloat,
    ProviderBandContext, layout_floats,
};

pub(crate) fn compute_block<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    compute_block_with_optional_inherited_float_exclusions(tree, node, input, None)
}

pub(crate) fn compute_block_with_inherited_float_exclusions<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    inherited: InheritedFloatExclusions<Tree::Scalar, <Tree as Traverse>::Node>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    compute_block_with_optional_inherited_float_exclusions(tree, node, input, Some(inherited))
}

fn compute_block_with_optional_inherited_float_exclusions<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    inherited: Option<InheritedFloatExclusions<Tree::Scalar, <Tree as Traverse>::Node>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let scrollbar_width = tree.node_input(node).scrollbar_width.get();
    let mut pass_input = input;
    loop {
        let output = compute_block_inner::<Tree, Tree::Scalar, M>(
            tree,
            node,
            pass_input,
            inherited.as_ref(),
        )?;
        if !input.run_mode().is_perform_layout() {
            return Ok(output);
        }
        let Some(geometry) = output.scroll_geometry else {
            return Ok(output);
        };
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || scrollbar_width == Tree::Scalar::ZERO
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

fn compute_block_inner<Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<S>,
    inherited: Option<&InheritedFloatExclusions<S, <Tree as Traverse>::Node>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let style = tree.node_input(node).clone();
    let constants = Constants::new::<Tree, M>(tree, node, &style, input)?;
    let children = tree.children(node).collect::<Vec<_>>();

    if children.is_empty()
        && input.run_mode() == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return Ok(ComputeOutputOf::<S>::from_outer_size(Size::new(
            width, height,
        )));
    }
    if input.run_mode() == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
        && !normal_flow_children_can_establish_baseline(tree, &children)
    {
        return Ok(ComputeOutputOf::<S>::from_outer_size(Size::new(
            width, height,
        )));
    }

    let logical_inner_size = constants.logical_node_inner_size();
    let needs_final_pass = input.run_mode().is_perform_layout()
        && (logical_inner_size.inline.is_none()
            || constants
                .flow_axes
                .logical_axis_progression(crate::LogicalAxis::Block)
                .is_decreasing()
                && logical_inner_size.block.is_none());
    let intrinsic_pass = layout_in_flow_children(
        tree,
        node,
        &children,
        &constants,
        input,
        InFlowPassContext {
            inner_inline: logical_inner_size.inline,
            set_layout: input.run_mode().is_perform_layout() && !needs_final_pass,
            inherited,
        },
    )?;
    let logical_intrinsic_outer_size = LogicalSizeOf::new(
        intrinsic_pass.content_size.inline + constants.logical_content_box_inset().inline_sum(),
        intrinsic_pass.auto_block(&constants),
    );
    let logical_intrinsic_outer_size = LogicalSizeOf::new(
        logical_intrinsic_outer_size
            .inline
            .clamp_max_before_min_optional(
                constants.logical_node_min_size().inline,
                constants.logical_node_max_size().inline,
            ),
        logical_intrinsic_outer_size
            .block
            .clamp_max_before_min_optional(
                constants.logical_node_min_size().block,
                constants.logical_node_max_size().block,
            ),
    )
    .max_optional(constants.logical_padding_border_size().map(Some));
    let logical_outer_size = constants
        .logical_node_outer_size()
        .unwrap_or(logical_intrinsic_outer_size)
        .max_optional(constants.logical_padding_border_size().map(Some));
    let provisional_logical_output_size = constants
        .flow_axes
        .logical_size(input.known())
        .or(constants.logical_node_outer_size())
        .unwrap_or(logical_outer_size)
        .max_optional(constants.logical_padding_border_size().map(Some));
    let (final_constants, final_pass) = if needs_final_pass {
        let logical_inner_size = LogicalSizeOf::new(
            Some(
                (provisional_logical_output_size.inline
                    - constants.logical_content_box_inset().inline_sum())
                .max(S::ZERO),
            ),
            Some(
                (provisional_logical_output_size.block
                    - constants.logical_content_box_inset().block_sum())
                .max(S::ZERO),
            ),
        );
        let final_constants = constants.with_logical_node_inner_size(logical_inner_size);
        let final_pass = layout_in_flow_children(
            tree,
            node,
            &children,
            &final_constants,
            input,
            InFlowPassContext {
                inner_inline: logical_inner_size.inline,
                set_layout: true,
                inherited,
            },
        )?;
        (final_constants, final_pass)
    } else {
        (constants, intrinsic_pass)
    };
    let logical_output_size = LogicalSizeOf::new(
        provisional_logical_output_size.inline,
        constants
            .flow_axes
            .logical_size(input.known())
            .block
            .or(final_constants.logical_node_outer_size().block)
            .unwrap_or_else(|| final_pass.auto_block(&final_constants))
            .clamp_max_before_min_optional(
                final_constants.logical_node_min_size().block,
                final_constants.logical_node_max_size().block,
            )
            .max(final_constants.logical_padding_border_size().block),
    );
    let output_size = final_constants.flow_axes.physical_size(logical_output_size);
    let top_margin = final_pass.top_margin(&final_constants);
    let bottom_margin = final_pass.bottom_margin(&final_constants);
    let margins_can_collapse_through = final_constants.can_collapse_through
        && final_pass.all_in_flow_children_can_collapse_through;
    let block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
        final_constants.flow_axes,
        top_margin,
        bottom_margin,
        margins_can_collapse_through,
    );

    if input.run_mode() == RunMode::ComputeSize {
        let mut output = ComputeOutputOf::<S>::from_sizes_and_baselines(
            output_size,
            Size::ZERO,
            final_pass.baselines,
        );
        output.block_margin_collapse = block_margin_collapse;
        Ok(output)
    } else {
        let mut contributions = prepare_scroll_contributions::<_, _, M>(
            node,
            input.run_mode(),
            &style,
            &final_constants,
            output_size,
            final_pass.scroll_content_size,
            final_pass.contributions,
        )?;
        layout_floats(
            tree,
            node,
            &final_pass.pending_floats,
            output_size,
            &final_constants,
            &mut contributions,
        )?;
        layout_absolute_children(
            tree,
            node,
            &children,
            &final_pass.static_positions,
            output_size,
            &final_constants,
            &mut contributions,
        )?;
        let published_scroll = finish_scroll_geometry::<Tree, S, M>(
            node,
            input.run_mode(),
            &style,
            &final_constants,
            output_size,
            contributions,
        )?;
        let mut output = ComputeOutputOf::<S>::from_sizes_and_baselines(
            output_size,
            published_scroll.content_size,
            final_pass.baselines,
        );
        output.scroll_geometry = Some(published_scroll.geometry);
        output.block_margin_collapse = block_margin_collapse;
        Ok(output)
    }
}

#[derive(Clone, Copy, Debug)]
enum ChildContainingBlockExtent<S: LayoutScalar> {
    Definite(S),
    FinalAutoDerived(S),
}

impl<S: LayoutScalar> ChildContainingBlockExtent<S> {
    fn value(self) -> S {
        match self {
            Self::Definite(value) | Self::FinalAutoDerived(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Constants<S: LayoutScalar> {
    flow_axes: crate::geometry::FlowAxes,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    child_containing_block_extent: Size<Option<ChildContainingBlockExtent<S>>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    direction: Direction,
    writing_mode: WritingMode,
    text_align: TextAlign,
    border: Edges<S>,
    padding: Edges<S>,
    effective_border: Edges<S>,
    padding_border_size: Size<S>,
    scrollbar_gutter: Edges<S>,
    content_box_inset: Edges<S>,
    available_content: Size<AvailableOf<S>>,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
    own_top_margin: CollapsibleMarginOf<S>,
    own_bottom_margin: CollapsibleMarginOf<S>,
    collapse_top_margin: bool,
    collapse_bottom_margin: bool,
    can_collapse_through: bool,
}

impl<S: LayoutScalar> Constants<S> {
    fn logical_node_outer_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_outer_size)
    }

    fn logical_node_inner_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_inner_size)
    }

    fn logical_node_min_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_min_size)
    }

    fn logical_node_max_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_max_size)
    }

    fn logical_padding_border_size(&self) -> LogicalSizeOf<S> {
        self.flow_axes.logical_size(self.padding_border_size)
    }

    fn logical_content_box_inset(&self) -> LogicalEdgesOf<S> {
        self.flow_axes.logical_edges(self.content_box_inset)
    }

    fn containing_size(&self, inner_size: LogicalSizeOf<Option<S>>) -> Size<S> {
        self.node_outer_size
            .unwrap_or(self.flow_axes.physical_size(LogicalSizeOf::new(
                inner_size.inline.unwrap_or(S::ZERO)
                    + self.logical_content_box_inset().inline_sum(),
                inner_size.block.unwrap_or(S::ZERO) + self.logical_content_box_inset().block_sum(),
            )))
    }

    fn child_containing_block_size(
        &self,
        child_flow_axes: crate::geometry::FlowAxes,
    ) -> Size<Option<S>> {
        let child_logical_extent = child_flow_axes.logical_size(self.child_containing_block_extent);
        let child_inline_extent = match child_logical_extent.inline {
            Some(ChildContainingBlockExtent::Definite(value)) => Some(value),
            Some(ChildContainingBlockExtent::FinalAutoDerived(value))
                if self.flow_axes.inline_axis() == child_flow_axes.inline_axis() =>
            {
                Some(value)
            }
            Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => None,
        };
        let child_block_extent = child_logical_extent
            .block
            .map(ChildContainingBlockExtent::value);
        child_flow_axes.physical_size(LogicalSizeOf::new(child_inline_extent, child_block_extent))
    }

    fn definite_child_containing_block_size(&self) -> Size<Option<S>> {
        self.child_containing_block_extent
            .map(|extent| match extent {
                Some(ChildContainingBlockExtent::Definite(value)) => Some(value),
                Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => None,
            })
    }

    fn with_logical_node_inner_size(mut self, inner_size: LogicalSizeOf<Option<S>>) -> Self {
        let final_inner_size = self.flow_axes.physical_size(inner_size);
        self.child_containing_block_extent =
            final_inner_size.zip_map(self.child_containing_block_extent, |value, previous| {
                value.map(|value| match previous {
                    Some(ChildContainingBlockExtent::Definite(_)) => {
                        ChildContainingBlockExtent::Definite(value)
                    }
                    Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => {
                        ChildContainingBlockExtent::FinalAutoDerived(value)
                    }
                })
            });
        self.node_inner_size = final_inner_size;
        let content_box_inset = self.logical_content_box_inset();
        self.node_outer_size = self.flow_axes.physical_size(LogicalSizeOf::new(
            inner_size
                .inline
                .map(|inline| inline + content_box_inset.inline_sum()),
            inner_size
                .block
                .map(|block| block + content_box_inset.block_sum()),
        ));
        self
    }

    fn new<Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &NodeInputOf<S>,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        let flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
        let (padding, border) = resolve_containing_padding_border(
            input.containing_flow_axes(),
            input.parent(),
            style.padding,
            style.border,
            resolve_length_or_zero,
            |edges| edges.transpose_with_node(tree, node),
        )?;
        let own_logical_margin = flow_axes.logical_edges(style.margin);
        let collapsible_margin = flow_axes.physical_edges(LogicalEdgesOf::new(
            LengthAutoOf::ZERO,
            LengthAutoOf::ZERO,
            own_logical_margin.block_start,
            own_logical_margin.block_end,
        ));
        let margin = input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(
                collapsible_margin,
                input.parent(),
                |length, basis| resolve_auto_optional(length, basis),
            )
            .transpose_with_node(tree, node)?;
        let padding_border_size = (padding + border).sum_axes();
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border_size
        } else {
            Size::ZERO
        };
        let (style_size, min_size, max_size) = match input.sizing_mode() {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::InherentSize => {
                let style_size =
                    preferred_size(&style.size, input.parent(), SizingAlgorithm::Block, true)
                        .transpose_with_node(tree, node)?
                        .apply_aspect_ratio(style.aspect_ratio)
                        .add_optional(box_sizing_adjustment);
                let min_size = minimum_size(
                    &style.min_size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let max_size = maximum_size(
                    &style.max_size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
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
        let is_root = input.run_mode() == RunMode::PerformRootLayout;
        let boundary_margins_can_collapse =
            input.parent_formatting_context() == ParentFormattingContext::BlockFlow;
        let blocks_margin_collapse =
            !style.item_is_replaced && style.overflow.establishes_independent_formatting_context();
        let is_margin_collapsing_block = style.display == super::Display::Block;
        let can_collapse_through = is_margin_collapsing_block
            && boundary_margins_can_collapse
            && !is_root
            && !blocks_margin_collapse
            && style.position == Position::Relative
            && flow_axes.logical_edges(padding).block_start == S::ZERO
            && flow_axes.logical_edges(padding).block_end == S::ZERO
            && flow_axes.logical_edges(border).block_start == S::ZERO
            && flow_axes.logical_edges(border).block_end == S::ZERO
            && !matches!(flow_axes.logical_size(style_size).block, Some(block) if block > S::ZERO)
            && !matches!(flow_axes.logical_size(min_size).block, Some(block) if block > S::ZERO);
        let node_outer_size = input
            .known()
            .or(min_max_definite_size)
            .or(style_size.clamp_max_before_min_optional(min_size, max_size))
            .max_optional(padding_border_size.map(Some));
        let unconstrained_scroll_box_size = padding_border_size
            + Size::splat(style.scrollbar_width.get() + style.scrollbar_width.get());
        let scroll_box_size = node_outer_size
            .or(input.available().map(AvailableOf::into_option))
            .or(max_size)
            .unwrap_or(unconstrained_scroll_box_size)
            .zip_map(padding_border_size, |size, minimum| size.max(minimum));
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
        let effective_border = scroll_box.effective_border();
        let scrollbar_gutter = scroll_box.effective_gutter();
        let content_box_inset =
            effective_border + scrollbar_gutter + scroll_box.effective_padding();
        let content_box_inset_size = content_box_inset.sum_axes();
        let node_inner_size = node_outer_size.sub_optional_clamped_to_zero(content_box_inset_size);
        let available_content =
            input
                .available()
                .zip_map(content_box_inset_size, |available, inset| match available {
                    AvailableOf::Definite(value) => {
                        AvailableOf::Definite((value - inset).max(S::ZERO))
                    }
                    AvailableOf::MinContent => AvailableOf::MinContent,
                    AvailableOf::MaxContent => AvailableOf::MaxContent,
                });
        let logical_padding = flow_axes.logical_edges(padding);
        let logical_border = flow_axes.logical_edges(border);
        let logical_style_size = flow_axes.logical_size(style_size);
        let logical_margin = flow_axes.logical_edges(margin);

        Ok(Self {
            flow_axes,
            node_outer_size,
            node_inner_size,
            child_containing_block_extent: node_inner_size
                .map(|value| value.map(ChildContainingBlockExtent::Definite)),
            node_min_size: min_size,
            node_max_size: max_size,
            direction: style.direction,
            writing_mode: style.writing_mode,
            text_align: style.text_align,
            border,
            padding,
            effective_border,
            padding_border_size,
            scrollbar_gutter,
            content_box_inset,
            available_content,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            own_top_margin: CollapsibleMarginOf::<S>::from_margin(
                logical_margin.block_start.unwrap_or(S::ZERO),
            ),
            own_bottom_margin: CollapsibleMarginOf::<S>::from_margin(
                logical_margin.block_end.unwrap_or(S::ZERO),
            ),
            collapse_top_margin: is_margin_collapsing_block
                && boundary_margins_can_collapse
                && !is_root
                && style.position == Position::Relative
                && !blocks_margin_collapse
                && logical_padding.block_start == S::ZERO
                && logical_border.block_start == S::ZERO,
            collapse_bottom_margin: is_margin_collapsing_block
                && boundary_margins_can_collapse
                && !is_root
                && style.position == Position::Relative
                && !blocks_margin_collapse
                && logical_padding.block_end == S::ZERO
                && logical_border.block_end == S::ZERO
                && logical_style_size.block.is_none(),
            can_collapse_through,
        })
    }
}

#[cfg(test)]
mod fri06_c13_t06_characterization_tests {
    use super::*;
    use crate::LengthPercentageOf;
    use crate::layout_math::{resolution_optional, resolution_or_zero};

    fn input<S: LayoutScalar>(
        containing_flow_axes: crate::geometry::FlowAxes,
        parent: Size<Option<S>>,
    ) -> ComputeInputOf<S> {
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::ContentSize,
            RequestedAxis::Both,
            Size::NONE,
            parent,
            ContainingLayoutContext::new(containing_flow_axes, ParentFormattingContext::NoParent),
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
            display: crate::Display::Block,
            padding: percentage_edges(),
            border,
            ..NodeInputOf::default()
        };
        let tree = crate::test_support::layout_tree::OracleTreeOf::new().style(7, style.clone());

        for (flow, parent, expected_padding) in [
            (
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(100.0)),
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(200.0)),
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(None, Some(S::from_f64(200.0))),
                Edges::ZERO,
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
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
            .expect("block constants edge characterization must resolve");
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
            display: crate::Display::Block,
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
        let error = Constants::new::<_, core::convert::Infallible>(
            &failing_tree,
            7,
            &failing_style,
            input(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::splat(Some(largest)),
            ),
        )
        .expect_err("padding failure must precede the distinct border failure");
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
    fn fri06_c13_t06_block_resolution_edges_and_error_order_preserve_f32() {
        characterize_constants::<f32>(f32::MAX);
    }

    #[test]
    fn fri06_c13_t06_block_resolution_edges_and_error_order_preserve_f64() {
        characterize_constants::<f64>(f64::MAX);
    }
}
