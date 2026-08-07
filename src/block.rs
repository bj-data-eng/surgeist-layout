use std::collections::BTreeMap;

use super::inline::{
    AtomicInlineBoxParticipant, ForcedLineBreakControlOf, InlineBoundaryControlOf,
    InlineControlAlignment, InlineFlowOf, LogicalLineBandQueryResultOf, MixedInlineParticipantOf,
    MixedInlineRunInputOf, PostLineClearIntent, ShapedTextParticipantOf,
    layout_mixed_inline_run_with_band_source,
};
use super::value::{ResolvedLengthAutoOf, UnresolvedLengthReason};
use super::{
    AvailableOf, BaselinesOf, BoxSizing, Clear, CollapsibleMarginOf, Compute, ComputeInputOf,
    ComputeOutputOf, ComputedOverflow, ContainingLayoutContext, Direction, Edges, Float,
    FloatExclusion, FloatExclusionIntervalErrorOf, FloatExclusionIntervalOf, FloatExclusionQueryOf,
    InlineBoundaryInputOf, InlineFragmentOutputOf, LayoutErrorKindOf, LayoutErrorOf,
    LayoutErrorSiteOf, LayoutInputOf, LayoutInvalidInputOf, LayoutMissingContext, LayoutOperation,
    LayoutResultOf, LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionOf,
    LengthResolutionStatus, LineBreakInputOf, NodeInputOf, NodeOutputOf, Overflow,
    ParentFormattingContext, PhysicalBlockMarginCollapseOf, Point, Position, RequestedAxis,
    RunMode, ScrollRectOf, Size, SizingAlgorithm, SizingMode, TextAlign, Traverse, VerticalAlign,
    WritingMode,
};
use crate::compute::{
    AtomicInlineParticipationRoleError, EdgesResultExt, SizeResultExt, SizingResolutionError,
    layout_child_geometry_error, layout_own_geometry_error, resolve_maximum_optional,
    resolve_minimum_optional, resolve_preferred_optional,
};
use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalSide};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt,
    OptionalSizeExt,
};
use crate::scroll::{
    CanonicalScrollBoxSourceOf, CanonicalScrollGeometryErrorOf, CanonicalScrollGeometrySourceOf,
    ClipMarginSourceOf, OptimalRegionInsetsOf, ScrollContributionAccumulatorOf, ScrollOriginAxes,
    ScrollOriginProgression, UsedOverflow, canonical_scroll_box_from_source,
    canonical_scroll_geometry_from_source, rebuild_canonical_scroll_geometry_for_border_box,
    scrollbar_size_from_overflow,
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
        let final_scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes: final_constants.flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            border_box_size: output_size,
            border: final_constants.border,
            padding: final_constants.padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: final_constants.settled_auto_scrollbars,
        })
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let mut contributions = final_pass.contributions;
        contributions.replace_container_seed(final_scroll_box.padding_box());
        contributions.exclude_reserved_gutter_from_range();
        for (axis, extent) in [
            (
                crate::LogicalAxis::Inline,
                final_pass.scroll_content_size.inline,
            ),
            (
                crate::LogicalAxis::Block,
                final_pass.scroll_content_size.block,
            ),
        ] {
            contributions
                .record_final_in_flow_end(
                    final_constants.flow_axes,
                    axis,
                    block_final_in_flow_end(
                        final_scroll_box.content_box(),
                        final_constants.flow_axes,
                        axis,
                        extent,
                    ),
                )
                .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        }
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
        contributions
            .include_terminal_padding(final_constants.padding)
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let scroll_geometry = block_scroll_geometry::<Tree, S, M>(
            node,
            input.run_mode(),
            &style,
            &final_constants,
            output_size,
            contributions,
        )?;
        let content_size = contributions
            .content_size_from_anchor(scroll_geometry.content_box().origin())
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let mut output = ComputeOutputOf::<S>::from_sizes_and_baselines(
            output_size,
            content_size,
            final_pass.baselines,
        );
        output.scroll_geometry = Some(scroll_geometry);
        output.block_margin_collapse = block_margin_collapse;
        Ok(output)
    }
}

fn normal_flow_children_can_establish_baseline<Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
) -> bool
where
    Tree: Compute<M>,
{
    children.iter().copied().any(|child| {
        let style = match tree.layout_input(child) {
            LayoutInputOf::InlineText(_) => return true,
            LayoutInputOf::Box(style) => style,
            LayoutInputOf::LineBreak(input) => {
                return !input.display().is_none()
                    && input.metrics().line_extent() > Tree::Scalar::ZERO;
            }
            LayoutInputOf::InlineBoundary(input) => {
                return input.metrics().line_extent() > Tree::Scalar::ZERO;
            }
        };
        if style.display == super::Display::None
            || style.position == Position::Absolute
            || style.float != Float::None
        {
            return false;
        }

        style.display.is_inline_level()
            || style.display.inner_display() == super::Display::Block
            || tree.child_count(child) > 0 && style.display.inner_display() == super::Display::Flex
    })
}

struct PendingFloat<Node, S: LayoutScalar> {
    node: Node,
    source_index: usize,
    side: Float,
    clear: Clear,
    block_start: S,
    size: Size<S>,
    content_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
    margin: Edges<S>,
    style: Box<NodeInputOf<S>>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FloatLedgerSide {
    LineStart,
    LineEnd,
}

impl FloatLedgerSide {
    fn from_float(side: Float) -> Self {
        match side {
            Float::Left | Float::None => Self::LineStart,
            Float::Right => Self::LineEnd,
        }
    }

    fn is_cleared_by(self, clear: Clear) -> bool {
        match clear {
            Clear::None => false,
            Clear::Left => self == Self::LineStart,
            Clear::Right => self == Self::LineEnd,
            Clear::Both => true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PhysicalMarginBox<S: LayoutScalar> {
    origin: Point<S>,
    size: Size<S>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FloatLedgerOrder(usize);

#[derive(Clone, Copy, Debug)]
struct FloatLedgerEntry<S: LayoutScalar, Node> {
    node: Node,
    side: FloatLedgerSide,
    exclusion: FloatExclusion,
    physical_margin_box: PhysicalMarginBox<S>,
    inline_start: S,
    inline_end: S,
    block_start: S,
    block_end: S,
    ledger_order: FloatLedgerOrder,
}

impl<S: LayoutScalar, Node: Copy> FloatLedgerEntry<S, Node> {
    fn overlaps_block_span(self, block_start: S, block_end: S) -> bool {
        self.block_start < block_end && block_start < self.block_end
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FloatBand<S: LayoutScalar> {
    pub(super) inline_start: S,
    pub(super) inline_end: S,
    pub(super) next_transition: Option<S>,
    #[cfg(test)]
    pub(super) evaluated: usize,
}

#[derive(Clone, Copy, Debug)]
struct BfcBandPlacement<S: LayoutScalar> {
    location: Point<S>,
    available_inline: S,
}

#[derive(Clone, Copy, Debug)]
struct BfcBandCandidate<S: LayoutScalar> {
    block_start: S,
    size: Size<S>,
    margin: Edges<S>,
    clear: Clear,
    fallback: Point<S>,
    inline_size_is_auto: bool,
}

#[derive(Clone, Copy)]
struct ProviderBandContext<'a, Tree, Node> {
    tree: &'a Tree,
    container: Node,
    enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FloatBandQueryPurpose {
    PhysicalMarginBoxCollision,
    RectangularLineBand,
    ProviderBand,
}

#[derive(Clone, Debug)]
pub(super) struct FloatExclusions<S: LayoutScalar, Node = ()> {
    flow_axes: crate::geometry::FlowAxes,
    containing_size: Size<S>,
    containing_inline_start: S,
    containing_inline_end: S,
    ledger: Vec<FloatLedgerEntry<S, Node>>,
}

#[derive(Clone, Debug)]
pub(crate) struct InheritedFloatExclusions<S: LayoutScalar, Node> {
    parent_flow_axes: crate::geometry::FlowAxes,
    parent_containing_size: Size<S>,
    child_logical_location: LogicalPointOf<S>,
    ledger: Vec<FloatLedgerEntry<S, Node>>,
}

impl<S: LayoutScalar, Node: Copy> FloatExclusions<S, Node> {
    pub(super) fn new(
        flow_axes: crate::geometry::FlowAxes,
        containing_size: Size<S>,
        content_inline_size: S,
        inset: LogicalEdgesOf<S>,
    ) -> Self {
        Self {
            flow_axes,
            containing_size,
            containing_inline_start: inset.inline_start,
            containing_inline_end: inset.inline_start + content_inline_size,
            ledger: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.ledger.is_empty()
    }

    fn for_ordinary_child(
        &self,
        child_logical_location: LogicalPointOf<S>,
    ) -> InheritedFloatExclusions<S, Node> {
        InheritedFloatExclusions {
            parent_flow_axes: self.flow_axes,
            parent_containing_size: self.containing_size,
            child_logical_location,
            ledger: self.ledger.clone(),
        }
    }

    fn inherit_into_child(&mut self, inherited: &InheritedFloatExclusions<S, Node>) {
        let child_size = self.containing_size;
        let child_logical_size = inherited.parent_flow_axes.logical_size(child_size);
        let child_physical_origin = inherited.parent_flow_axes.physical_point(
            inherited.child_logical_location,
            child_logical_size,
            inherited.parent_containing_size,
        );
        let mut inherited_order = self.ledger.len();
        self.ledger
            .extend(inherited.ledger.iter().copied().map(|entry| {
                let physical_origin = Point::new(
                    entry.physical_margin_box.origin.x - child_physical_origin.x,
                    entry.physical_margin_box.origin.y - child_physical_origin.y,
                );
                let physical_size = entry.physical_margin_box.size;
                let logical_origin =
                    self.flow_axes
                        .logical_point(physical_origin, physical_size, child_size);
                let logical_size = self.flow_axes.logical_size(physical_size);
                let parent_side = match entry.side {
                    FloatLedgerSide::LineStart => inherited.parent_flow_axes.inline_start(),
                    FloatLedgerSide::LineEnd => inherited.parent_flow_axes.inline_end(),
                };
                let side = if parent_side == self.flow_axes.inline_start() {
                    FloatLedgerSide::LineStart
                } else if parent_side == self.flow_axes.inline_end() {
                    FloatLedgerSide::LineEnd
                } else {
                    let distance_from_start =
                        (logical_origin.inline - self.containing_inline_start).abs();
                    let distance_from_end = (self.containing_inline_end
                        - (logical_origin.inline + logical_size.inline))
                        .abs();
                    if distance_from_start <= distance_from_end {
                        FloatLedgerSide::LineStart
                    } else {
                        FloatLedgerSide::LineEnd
                    }
                };
                let ledger_order = FloatLedgerOrder(inherited_order);
                inherited_order += 1;
                FloatLedgerEntry {
                    node: entry.node,
                    side,
                    physical_margin_box: PhysicalMarginBox {
                        origin: physical_origin,
                        size: physical_size,
                    },
                    inline_start: logical_origin.inline,
                    inline_end: logical_origin.inline + logical_size.inline,
                    block_start: logical_origin.block,
                    block_end: logical_origin.block + logical_size.block,
                    ledger_order,
                    ..entry
                }
            }));
    }

    fn place_float<Tree, M>(
        &mut self,
        provider: ProviderBandContext<'_, Tree, Node>,
        float: &PendingFloat<Node, S>,
    ) -> LayoutResultOf<Node, Point<S>, S, M>
    where
        Tree: Compute<M, Node = Node, Scalar = S>,
    {
        let side = FloatLedgerSide::from_float(float.side);
        let logical_size = self.flow_axes.logical_size(float.size);
        let logical_margin = self.flow_axes.logical_edges(float.margin);
        let margin_box_size = LogicalSizeOf::new(
            logical_size.inline + logical_margin.inline_sum(),
            logical_size.block + logical_margin.block_sum(),
        );
        let mut candidate_block = self.clearance_block(float.block_start, float.clear);

        loop {
            let candidate_block_end = candidate_block + margin_box_size.block;
            let band = if provider.enabled {
                self.query_provider_band(
                    provider.tree,
                    provider.container,
                    candidate_block,
                    candidate_block_end,
                )?
            } else {
                self.query_physical_margin_box_collisions(candidate_block, candidate_block_end)
            };
            let available_inline = (band.inline_end - band.inline_start).max(S::ZERO);
            if margin_box_size.inline <= available_inline || band.next_transition.is_none() {
                let margin_box_inline = match side {
                    FloatLedgerSide::LineStart => band.inline_start,
                    FloatLedgerSide::LineEnd => band.inline_end - margin_box_size.inline,
                };
                let margin_box_origin = LogicalPointOf::new(margin_box_inline, candidate_block);
                let content_origin = LogicalPointOf::new(
                    margin_box_inline + logical_margin.inline_start,
                    candidate_block + logical_margin.block_start,
                );
                let physical_margin_origin = self.flow_axes.physical_point(
                    margin_box_origin,
                    margin_box_size,
                    self.containing_size,
                );
                let physical_margin_size = self.flow_axes.physical_size(margin_box_size);
                let ledger_order = FloatLedgerOrder(self.ledger.len());
                self.ledger.push(FloatLedgerEntry {
                    node: float.node,
                    side,
                    exclusion: float.style.float_exclusion,
                    physical_margin_box: PhysicalMarginBox {
                        origin: physical_margin_origin,
                        size: physical_margin_size,
                    },
                    inline_start: margin_box_inline,
                    inline_end: margin_box_inline + margin_box_size.inline,
                    block_start: candidate_block,
                    block_end: candidate_block + margin_box_size.block,
                    ledger_order,
                });
                return Ok(self.flow_axes.physical_point(
                    content_origin,
                    logical_size,
                    self.containing_size,
                ));
            }
            candidate_block = band
                .next_transition
                .expect("a retry is entered only with a finite later transition");
        }
    }

    fn place_bfc_block<Tree, M>(
        &self,
        provider: ProviderBandContext<'_, Tree, Node>,
        candidate: BfcBandCandidate<S>,
    ) -> LayoutResultOf<Node, BfcBandPlacement<S>, S, M>
    where
        Tree: Compute<M, Node = Node, Scalar = S>,
    {
        let BfcBandCandidate {
            block_start,
            size,
            margin,
            clear,
            fallback,
            inline_size_is_auto,
        } = candidate;
        let logical_size = self.flow_axes.logical_size(size);
        let logical_margin = self.flow_axes.logical_edges(margin);
        let margin_box_inline = logical_size.inline + logical_margin.inline_sum();
        let margin_box_block = logical_size.block + logical_margin.block_sum();
        let fallback_logical = self
            .flow_axes
            .logical_point(fallback, size, self.containing_size);
        let mut candidate_block = self.clearance_block(block_start, clear);
        loop {
            let candidate_block_end = candidate_block + margin_box_block;
            let band = if provider.enabled {
                self.query_provider_band(
                    provider.tree,
                    provider.container,
                    candidate_block,
                    candidate_block_end,
                )?
            } else {
                self.query_physical_margin_box_collisions(candidate_block, candidate_block_end)
            };
            let fallback_start = fallback_logical.inline - logical_margin.inline_start;
            let fallback_end =
                fallback_logical.inline + logical_size.inline + logical_margin.inline_end;
            let available_inline = (band.inline_end - band.inline_start).max(S::ZERO);
            let required_inline = if inline_size_is_auto {
                logical_margin.inline_sum()
            } else {
                margin_box_inline
            };
            if required_inline <= available_inline {
                let inline = if !inline_size_is_auto
                    && fallback_start >= band.inline_start
                    && fallback_end <= band.inline_end
                {
                    fallback_logical.inline
                } else {
                    band.inline_start + logical_margin.inline_start
                };
                return Ok(BfcBandPlacement {
                    location: self.flow_axes.physical_point(
                        LogicalPointOf::new(inline, candidate_block),
                        logical_size,
                        self.containing_size,
                    ),
                    available_inline,
                });
            }
            if let Some(next_transition) = band.next_transition {
                candidate_block = next_transition;
            } else {
                return Ok(BfcBandPlacement {
                    location: self.flow_axes.physical_point(
                        LogicalPointOf::new(fallback_logical.inline, candidate_block),
                        logical_size,
                        self.containing_size,
                    ),
                    available_inline,
                });
            }
        }
    }

    fn clearance_block(&self, block: S, clear: Clear) -> S {
        self.ledger
            .iter()
            .copied()
            .filter(|entry| entry.side.is_cleared_by(clear))
            .map(|entry| entry.block_end)
            .fold(block, S::max)
    }

    fn clearance_for_line_intent(&self, block: S, clear: PostLineClearIntent) -> S {
        self.clearance_block(
            block,
            match clear {
                PostLineClearIntent::None => Clear::None,
                PostLineClearIntent::LineStart => Clear::Left,
                PostLineClearIntent::LineEnd => Clear::Right,
                PostLineClearIntent::Both => Clear::Both,
            },
        )
    }

    fn query_physical_margin_box_collisions(&self, block_start: S, block_end: S) -> FloatBand<S> {
        self.query_band_without_provider(
            block_start,
            block_end,
            FloatBandQueryPurpose::PhysicalMarginBoxCollision,
        )
    }

    pub(super) fn query_rectangular_line_band(&self, block_start: S, block_end: S) -> FloatBand<S> {
        self.query_band_without_provider(
            block_start,
            block_end,
            FloatBandQueryPurpose::RectangularLineBand,
        )
    }

    fn query_provider_band<Tree, M>(
        &self,
        tree: &Tree,
        container: Node,
        block_start: S,
        block_end: S,
    ) -> LayoutResultOf<Node, FloatBand<S>, S, M>
    where
        Tree: Compute<M, Node = Node, Scalar = S>,
    {
        self.query_band_for(
            block_start,
            block_end,
            FloatBandQueryPurpose::ProviderBand,
            |subject, expected| {
                let site = LayoutErrorSiteOf::ContainerSubject { container, subject };
                match tree.float_exclusion_interval(subject, expected) {
                    None => Err(LayoutErrorOf::new(
                        site,
                        LayoutOperation::FloatExclusionQuery,
                        LayoutErrorKindOf::MissingContext(
                            LayoutMissingContext::FloatExclusionProvider,
                        ),
                    )),
                    Some(Err(error)) => Err(LayoutErrorOf::new(
                        site,
                        LayoutOperation::FloatExclusionQuery,
                        LayoutErrorKindOf::Measurement(error),
                    )),
                    Some(Ok(None)) => Ok(None),
                    Some(Ok(Some(interval))) => {
                        let actual = interval.originating_query();
                        if actual != expected {
                            return Err(LayoutErrorOf::new(
                                site,
                                LayoutOperation::FloatExclusionQuery,
                                LayoutErrorKindOf::InvalidInput(
                                    LayoutInvalidInputOf::FloatExclusionProviderOutput {
                                        error: FloatExclusionIntervalErrorOf::QueryMismatch {
                                            expected,
                                            actual,
                                        },
                                    },
                                ),
                            ));
                        }
                        Ok(Some(interval))
                    }
                }
            },
        )
    }

    fn query_band_without_provider(
        &self,
        block_start: S,
        block_end: S,
        purpose: FloatBandQueryPurpose,
    ) -> FloatBand<S> {
        let result = self.query_band_for(
            block_start,
            block_end,
            purpose,
            |_, _| -> Result<Option<FloatExclusionIntervalOf<S>>, core::convert::Infallible> {
                unreachable!("provider-free band purposes never request a shape interval")
            },
        );
        match result {
            Ok(band) => band,
            Err(never) => match never {},
        }
    }

    fn query_band_for<E>(
        &self,
        block_start: S,
        block_end: S,
        purpose: FloatBandQueryPurpose,
        mut shape_provider: impl FnMut(
            Node,
            FloatExclusionQueryOf<S>,
        ) -> Result<Option<FloatExclusionIntervalOf<S>>, E>,
    ) -> Result<FloatBand<S>, E> {
        debug_assert!(
            self.ledger
                .windows(2)
                .all(|pair| pair[0].ledger_order <= pair[1].ledger_order),
            "float ledger remains in source order"
        );
        let mut inline_start = self.containing_inline_start;
        let mut inline_end = self.containing_inline_end;
        let mut next_transition = None;
        #[cfg(test)]
        let mut evaluated = 0;

        for entry in self.ledger.iter().copied() {
            #[cfg(test)]
            {
                evaluated += 1;
            }
            debug_assert!(
                entry.physical_margin_box.origin.x.is_finite()
                    && entry.physical_margin_box.origin.y.is_finite()
                    && entry.physical_margin_box.size.width.is_finite()
                    && entry.physical_margin_box.size.height.is_finite(),
                "placed float margin boxes remain finite"
            );
            if !entry.overlaps_block_span(block_start, block_end) {
                continue;
            }
            let logical_interval = match (purpose, entry.exclusion) {
                (FloatBandQueryPurpose::PhysicalMarginBoxCollision, _) => {
                    Some((entry.inline_start, entry.inline_end))
                }
                (FloatBandQueryPurpose::RectangularLineBand, FloatExclusion::MarginBox)
                | (FloatBandQueryPurpose::ProviderBand, FloatExclusion::MarginBox) => {
                    Some((entry.inline_start, entry.inline_end))
                }
                (FloatBandQueryPurpose::RectangularLineBand, FloatExclusion::Shape) => None,
                (FloatBandQueryPurpose::ProviderBand, FloatExclusion::Shape) => {
                    let query = self.provider_query(entry, block_start, block_end);
                    shape_provider(entry.node, query)?
                        .map(|interval| self.logical_inline_interval(interval))
                }
            };
            if let Some((entry_inline_start, entry_inline_end)) = logical_interval {
                match entry.side {
                    FloatLedgerSide::LineStart => {
                        inline_start = inline_start.max(entry_inline_end);
                    }
                    FloatLedgerSide::LineEnd => {
                        inline_end = inline_end.min(entry_inline_start);
                    }
                }
                next_transition = Some(
                    next_transition
                        .map_or(entry.block_end, |current: S| current.min(entry.block_end)),
                );
            }
        }

        Ok(FloatBand {
            inline_start,
            inline_end,
            next_transition,
            #[cfg(test)]
            evaluated,
        })
    }

    fn provider_query(
        &self,
        entry: FloatLedgerEntry<S, Node>,
        block_start: S,
        block_end: S,
    ) -> FloatExclusionQueryOf<S> {
        debug_assert!(block_start.is_finite() && block_end.is_finite());
        debug_assert!(block_start <= block_end);
        let logical_band_size = LogicalSizeOf::new(S::ZERO, block_end - block_start);
        let physical_band_origin = self.flow_axes.physical_point(
            LogicalPointOf::new(S::ZERO, block_start),
            logical_band_size,
            self.containing_size,
        );
        let physical_band_size = self.flow_axes.physical_size(logical_band_size);
        let band_minimum = self.flow_axes.block_axis_coordinate(physical_band_origin);
        let band_maximum = band_minimum + self.flow_axes.block_axis_extent(physical_band_size);
        let margin_box = ScrollRectOf::try_new(
            entry.physical_margin_box.origin,
            entry.physical_margin_box.size,
        )
        .expect("placed float margin boxes satisfy scroll-rectangle invariants");
        FloatExclusionQueryOf::try_new(margin_box, self.flow_axes, band_minimum, band_maximum)
            .expect("canonical finite line bands satisfy provider-query invariants")
    }

    fn logical_inline_interval(&self, interval: FloatExclusionIntervalOf<S>) -> (S, S) {
        let physical_extent = match self.flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => self.containing_size.width,
            PhysicalAxis::Vertical => self.containing_size.height,
        };
        if self
            .flow_axes
            .logical_axis_progression(crate::LogicalAxis::Inline)
            .is_decreasing()
        {
            (
                physical_extent - interval.maximum(),
                physical_extent - interval.minimum(),
            )
        } else {
            (interval.minimum(), interval.maximum())
        }
    }
}

#[cfg(test)]
impl<S: LayoutScalar> FloatExclusions<S> {
    pub(super) fn record_test_float(
        &mut self,
        side: FloatLedgerSide,
        exclusion: FloatExclusion,
        logical_origin: LogicalPointOf<S>,
        logical_size: LogicalSizeOf<S>,
    ) {
        let origin =
            self.flow_axes
                .physical_point(logical_origin, logical_size, self.containing_size);
        let size = self.flow_axes.physical_size(logical_size);
        let ledger_order = FloatLedgerOrder(self.ledger.len());
        self.ledger.push(FloatLedgerEntry {
            node: (),
            side,
            exclusion,
            physical_margin_box: PhysicalMarginBox { origin, size },
            inline_start: logical_origin.inline,
            inline_end: logical_origin.inline + logical_size.inline,
            block_start: logical_origin.block,
            block_end: logical_origin.block + logical_size.block,
            ledger_order,
        });
    }
}

struct InFlowResult<Node, S: LayoutScalar> {
    content_size: LogicalSizeOf<S>,
    scroll_content_size: LogicalSizeOf<S>,
    owned_float_block_end: S,
    resolved_terminal_float_block_end: Option<S>,
    contributions: ScrollContributionAccumulatorOf<S>,
    baselines: BaselinesOf<S>,
    static_positions: Vec<(Node, Point<S>)>,
    pending_floats: Vec<PendingFloat<Node, S>>,
    cursor_block: S,
    top_margin: CollapsibleMarginOf<S>,
    active_margin: CollapsibleMarginOf<S>,
    active_margin_can_collapse_with_parent: bool,
    all_in_flow_children_can_collapse_through: bool,
}

impl<Node, S: LayoutScalar> InFlowResult<Node, S> {
    fn top_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if constants.collapse_top_margin {
            self.top_margin
        } else {
            constants.own_top_margin
        }
    }

    fn bottom_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
            self.active_margin
        } else {
            constants.own_bottom_margin
        }
    }

    fn auto_block(&self, constants: &Constants<S>) -> S {
        let bottom_margin_offset =
            if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
                S::ZERO
            } else {
                self.active_margin.resolve()
            };
        let content_box_inset = constants.logical_content_box_inset();
        let in_flow_block_end = self.cursor_block + bottom_margin_offset;
        let float_block_end = self
            .resolved_terminal_float_block_end
            .unwrap_or(self.owned_float_block_end)
            .max(self.owned_float_block_end);
        (in_flow_block_end.max(float_block_end) + content_box_inset.block_end)
            .max(content_box_inset.block_sum())
    }
}

fn inline_run_end<Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<<Tree as Traverse>::Scalar>,
    mut index: usize,
) -> usize
where
    Tree: Compute<M>,
{
    while index < children.len() {
        match tree.layout_input(children[index]) {
            LayoutInputOf::Box(style) => {
                if style.display == super::Display::None || style.position == Position::Absolute {
                    index += 1;
                    continue;
                }
                if style.float != Float::None || !style.display.is_inline_level() {
                    break;
                }
            }
            LayoutInputOf::LineBreak(input) => {
                if input.display().is_none() {
                    index += 1;
                    continue;
                }
                visible_line_break_in_flow(
                    tree,
                    children[index],
                    constants.writing_mode,
                    constants.direction,
                );
            }
            LayoutInputOf::InlineText(_) => {}
            LayoutInputOf::InlineBoundary(_) => {
                visible_inline_boundary_in_flow(
                    tree,
                    children[index],
                    constants.writing_mode,
                    constants.direction,
                );
            }
        }
        index += 1;
    }
    index
}

fn visible_line_break_in_flow<Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<LineBreakInputOf<<Tree as Traverse>::Scalar>>
where
    Tree: Compute<M>,
{
    let LayoutInputOf::LineBreak(line_break) = tree.layout_input(child) else {
        return None;
    };
    if line_break.display().is_none() {
        return None;
    }
    if line_break.writing_mode() != flow_writing_mode || line_break.direction() != flow_direction {
        panic!("line-break flow must match containing inline flow");
    }
    Some(line_break)
}

fn visible_inline_boundary_in_flow<Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<InlineBoundaryInputOf<<Tree as Traverse>::Scalar>>
where
    Tree: Compute<M>,
{
    let LayoutInputOf::InlineBoundary(boundary) = tree.layout_input(child) else {
        return None;
    };
    if boundary.writing_mode() != flow_writing_mode || boundary.direction() != flow_direction {
        panic!("inline boundary flow must match containing inline flow");
    }
    Some(boundary)
}

struct InFlowPassContext<'a, S: LayoutScalar, Node> {
    inner_inline: Option<S>,
    set_layout: bool,
    inherited: Option<&'a InheritedFloatExclusions<S, Node>>,
}

fn layout_in_flow_children<Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<S>,
    input: ComputeInputOf<S>,
    pass: InFlowPassContext<'_, S, <Tree as Traverse>::Node>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InFlowResult<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowPassContext {
        inner_inline,
        set_layout,
        inherited,
    } = pass;
    let logical_node_inner_size =
        LogicalSizeOf::new(inner_inline, constants.logical_node_inner_size().block);
    let node_inner_size = constants.flow_axes.physical_size(logical_node_inner_size);
    let mut cursor_block = constants.logical_content_box_inset().block_start;
    let mut content_size = LogicalSizeOf::new(S::ZERO, S::ZERO);
    let mut scroll_content_size = LogicalSizeOf::new(S::ZERO, S::ZERO);
    let mut owned_float_block_end = constants.logical_content_box_inset().block_start;
    let mut resolved_terminal_float_block_end = None;
    let mut baselines = BaselinesOf::NONE;
    let mut static_positions = Vec::new();
    let mut active_margin = CollapsibleMarginOf::<S>::ZERO;
    let mut top_margin = CollapsibleMarginOf::<S>::ZERO;
    let mut is_collapsing_first_margin = constants.collapse_top_margin;
    let mut all_in_flow_children_can_collapse_through = true;
    let mut active_margin_can_collapse_with_parent = constants.collapse_top_margin;
    let mut pending_floats = Vec::new();
    let mut float_intrinsics = FloatIntrinsics::new(
        inner_inline
            .map(AvailableOf::<S>::definite)
            .unwrap_or(constants.available_content.width),
    );
    let content_inline_size = inner_inline
        .or(constants
            .flow_axes
            .logical_size(constants.available_content)
            .inline
            .into_option())
        .unwrap_or(S::ZERO);
    let containing_size = constants.containing_size(logical_node_inner_size);
    let mut float_exclusions = FloatExclusions::new(
        constants.flow_axes,
        containing_size,
        content_inline_size,
        constants.logical_content_box_inset(),
    );
    if set_layout && let Some(inherited) = inherited {
        float_exclusions.inherit_into_child(inherited);
    }
    let content_box_size = Size::new(
        inner_inline
            .or(constants.node_inner_size.width)
            .or(constants.available_content.width.into_option())
            .unwrap_or(S::ZERO),
        constants.node_inner_size.height.unwrap_or(S::ZERO),
    );
    let content_box_origin = Point::new(
        constants.content_box_inset.left,
        constants.content_box_inset.top,
    );
    let contribution_seed = super::ScrollRectOf::try_new(content_box_origin, content_box_size)
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
    let mut contributions = ScrollContributionAccumulatorOf::new(contribution_seed);

    let mut index = 0;
    while index < children.len() {
        let source_index = index;
        let child = children[index];
        let child_style = match tree.layout_input(child) {
            LayoutInputOf::Box(style) => *style,
            LayoutInputOf::InlineText(_) => {
                let run_start = index;
                index = inline_run_end(tree, children, constants, index + 1);

                let collapsed_margin = active_margin.resolve();
                cursor_block = cursor_block + collapsed_margin;
                if is_collapsing_first_margin {
                    is_collapsing_first_margin = false;
                }

                let placement = layout_inline_run_children(
                    tree,
                    node,
                    &children[run_start..index],
                    InlineRunContext {
                        source_index_start: run_start,
                        cursor_block,
                        owned_float_block_end,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    &float_exclusions,
                    &mut contributions,
                )?;
                let placement_content_size =
                    constants.flow_axes.logical_size(placement.content_size);
                content_size.inline = content_size.inline.max(placement_content_size.inline);
                content_size.block = content_size.block.max(placement_content_size.block);
                let placement_scroll_content_size = constants
                    .flow_axes
                    .logical_size(placement.scroll_content_size);
                scroll_content_size.inline = scroll_content_size
                    .inline
                    .max(placement_scroll_content_size.inline);
                scroll_content_size.block = scroll_content_size
                    .block
                    .max(placement_scroll_content_size.block);
                record_inline_run_baselines(&mut baselines, &placement, cursor_block, constants);
                cursor_block = cursor_block + placement.logical_block_extent(constants.flow_axes);
                static_positions.extend(placement.static_positions);
                resolved_terminal_float_block_end = placement.resolved_float_terminal_block_end;
                active_margin = CollapsibleMarginOf::<S>::ZERO;
                active_margin_can_collapse_with_parent = false;
                all_in_flow_children_can_collapse_through = false;
                continue;
            }
            LayoutInputOf::LineBreak(line_break) => {
                if line_break.display().is_none() {
                    if set_layout {
                        tree.set_unrounded(
                            child,
                            NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                source_index,
                            )),
                        );
                    }
                    index += 1;
                    continue;
                }
                visible_line_break_in_flow(
                    tree,
                    child,
                    constants.writing_mode,
                    constants.direction,
                );

                let run_start = index;
                index = inline_run_end(tree, children, constants, index + 1);

                let collapsed_margin = active_margin.resolve();
                cursor_block = cursor_block + collapsed_margin;
                if is_collapsing_first_margin {
                    is_collapsing_first_margin = false;
                }

                let placement = layout_inline_run_children(
                    tree,
                    node,
                    &children[run_start..index],
                    InlineRunContext {
                        source_index_start: run_start,
                        cursor_block,
                        owned_float_block_end,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    &float_exclusions,
                    &mut contributions,
                )?;
                let placement_content_size =
                    constants.flow_axes.logical_size(placement.content_size);
                content_size.inline = content_size.inline.max(placement_content_size.inline);
                content_size.block = content_size.block.max(placement_content_size.block);
                let placement_scroll_content_size = constants
                    .flow_axes
                    .logical_size(placement.scroll_content_size);
                scroll_content_size.inline = scroll_content_size
                    .inline
                    .max(placement_scroll_content_size.inline);
                scroll_content_size.block = scroll_content_size
                    .block
                    .max(placement_scroll_content_size.block);
                record_inline_run_baselines(&mut baselines, &placement, cursor_block, constants);
                cursor_block = cursor_block + placement.logical_block_extent(constants.flow_axes);
                static_positions.extend(placement.static_positions);
                resolved_terminal_float_block_end = placement.resolved_float_terminal_block_end;
                active_margin = CollapsibleMarginOf::<S>::ZERO;
                active_margin_can_collapse_with_parent = false;
                all_in_flow_children_can_collapse_through = false;
                continue;
            }
            LayoutInputOf::InlineBoundary(_) => {
                visible_inline_boundary_in_flow(
                    tree,
                    child,
                    constants.writing_mode,
                    constants.direction,
                );

                let run_start = index;
                index = inline_run_end(tree, children, constants, index + 1);

                let collapsed_margin = active_margin.resolve();
                cursor_block = cursor_block + collapsed_margin;
                if is_collapsing_first_margin {
                    is_collapsing_first_margin = false;
                }

                let placement = layout_inline_run_children(
                    tree,
                    node,
                    &children[run_start..index],
                    InlineRunContext {
                        source_index_start: run_start,
                        cursor_block,
                        owned_float_block_end,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                    },
                    &float_exclusions,
                    &mut contributions,
                )?;
                let placement_content_size =
                    constants.flow_axes.logical_size(placement.content_size);
                content_size.inline = content_size.inline.max(placement_content_size.inline);
                content_size.block = content_size.block.max(placement_content_size.block);
                let placement_scroll_content_size = constants
                    .flow_axes
                    .logical_size(placement.scroll_content_size);
                scroll_content_size.inline = scroll_content_size
                    .inline
                    .max(placement_scroll_content_size.inline);
                scroll_content_size.block = scroll_content_size
                    .block
                    .max(placement_scroll_content_size.block);
                record_inline_run_baselines(&mut baselines, &placement, cursor_block, constants);
                cursor_block = cursor_block + placement.logical_block_extent(constants.flow_axes);
                static_positions.extend(placement.static_positions);
                resolved_terminal_float_block_end = placement.resolved_float_terminal_block_end;
                active_margin = CollapsibleMarginOf::<S>::ZERO;
                active_margin_can_collapse_with_parent = false;
                all_in_flow_children_can_collapse_through = false;
                continue;
            }
        };
        if child_style.display == super::Display::None {
            if set_layout {
                tree.set_unrounded(
                    child,
                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(source_index)),
                );
                tree.compute_child(
                    child,
                    ComputeInputOf::<S>::hidden_in_containing_pass(
                        ContainingLayoutContext::new(
                            constants.flow_axes,
                            ParentFormattingContext::BlockFlow,
                        ),
                        input.settled_auto_scrollbars(),
                    ),
                )?;
            }
            index += 1;
            continue;
        }
        if child_style.position == Position::Absolute {
            static_positions.push((
                child,
                absolute_static_position(
                    cursor_block + active_margin.resolve(),
                    constants,
                    constants.containing_size(logical_node_inner_size),
                ),
            ));
            index += 1;
            continue;
        }

        if child_style.display.is_inline_level() && child_style.float.is_none() {
            let run_start = index;
            index = inline_run_end(tree, children, constants, index + 1);

            let collapsed_margin = active_margin.resolve();
            cursor_block = cursor_block + collapsed_margin;
            if is_collapsing_first_margin {
                is_collapsing_first_margin = false;
            }

            let placement = layout_inline_run_children(
                tree,
                node,
                &children[run_start..index],
                InlineRunContext {
                    source_index_start: run_start,
                    cursor_block,
                    owned_float_block_end,
                    constants,
                    input,
                    node_inner_size,
                    set_layout,
                },
                &float_exclusions,
                &mut contributions,
            )?;
            let placement_content_size = constants.flow_axes.logical_size(placement.content_size);
            content_size.inline = content_size.inline.max(placement_content_size.inline);
            content_size.block = content_size.block.max(placement_content_size.block);
            let placement_scroll_content_size = constants
                .flow_axes
                .logical_size(placement.scroll_content_size);
            scroll_content_size.inline = scroll_content_size
                .inline
                .max(placement_scroll_content_size.inline);
            scroll_content_size.block = scroll_content_size
                .block
                .max(placement_scroll_content_size.block);
            record_inline_run_baselines(&mut baselines, &placement, cursor_block, constants);
            cursor_block = cursor_block + placement.logical_block_extent(constants.flow_axes);
            static_positions.extend(placement.static_positions);
            resolved_terminal_float_block_end = placement.resolved_float_terminal_block_end;
            active_margin = CollapsibleMarginOf::<S>::ZERO;
            active_margin_can_collapse_with_parent = false;
            all_in_flow_children_can_collapse_through = false;
            continue;
        }

        let unresolved_margin = constants.flow_axes.zip_physical_edges_with_inline_extent(
            child_style.margin,
            node_inner_size,
            |length, basis| length.resolve_auto_with_status(basis),
        );
        let child_padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.padding,
                node_inner_size,
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let child_border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.border,
                node_inner_size,
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let parent_logical_unresolved_margin = constants.flow_axes.logical_edges(unresolved_margin);
        let parent_logical_available = constants
            .flow_axes
            .logical_size(constants.available_content);
        let child_flow_axes =
            crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction);
        let child_parent_size = constants.child_containing_block_size(child_flow_axes);
        let child_logical_node_inner_size = child_flow_axes.logical_size(child_parent_size);
        let child_logical_available = child_flow_axes.logical_size(constants.available_content);
        let child_non_auto_margin = child_flow_axes
            .logical_edges(unresolved_margin)
            .map(resolved_length_auto_fallback_zero);
        let available_child_inline = child_logical_node_inner_size
            .inline
            .or(child_logical_available.inline.into_option())
            .map(|inline| (inline - child_non_auto_margin.inline_sum()).max(S::ZERO));
        let child_known = in_flow_child_known_size::<Tree, M>(
            tree,
            child,
            &child_style,
            child_padding + child_border,
            child_flow_axes,
            child_logical_node_inner_size,
            available_child_inline,
        )?;
        let mut child_input = ComputeInputOf::<S>::for_child(
            input.run_mode().for_child(),
            SizingMode::InherentSize,
            RequestedAxis::Both,
            child_known,
            child_parent_size,
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
            child_flow_axes.physical_size(LogicalSizeOf::new(
                in_flow_child_available_inline(
                    &child_style,
                    child_flow_axes,
                    available_child_inline,
                    child_logical_available.inline,
                ),
                AvailableOf::<S>::MAX_CONTENT,
            )),
        )
        .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars());
        let avoids_float_exclusions = block_child_avoids_float_exclusions(&child_style);
        let inherits_float_exclusions = set_layout
            && child_style.float.is_none()
            && child_style.display.inner_display() == super::Display::Block
            && !avoids_float_exclusions
            && !float_exclusions.is_empty();
        let places_against_float_exclusions =
            set_layout && avoids_float_exclusions && !float_exclusions.is_empty();
        let mut output = tree.compute_child(
            child,
            if inherits_float_exclusions || places_against_float_exclusions {
                child_input.with_run_mode(RunMode::ComputeSize)
            } else {
                child_input
            },
        )?;

        let mut logical_child_size = constants.flow_axes.logical_size(output.size);
        let mut logical_child_margin = resolve_logical_in_flow_margin(
            parent_logical_unresolved_margin,
            logical_child_size,
            logical_node_inner_size
                .inline
                .or(parent_logical_available.inline.into_option()),
        );
        let mut child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        if inherits_float_exclusions {
            let preview_top_margin = output
                .block_margin_collapse
                .at(constants.flow_axes.block_start())
                .collapse_with_margin(
                    child_margin.at_physical_side(constants.flow_axes.block_start()),
                );
            let child_margin_can_collapse_with_parent =
                child_margin_can_collapse_with_parent(&child_style);
            let collapsed_margin = if is_collapsing_first_margin {
                if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                    active_margin.resolve()
                } else {
                    active_margin.collapse_with(preview_top_margin).resolve()
                }
            } else {
                active_margin.collapse_with(preview_top_margin).resolve()
            };
            let preview_cursor_block = cursor_block + collapsed_margin;
            let child_logical_location = LogicalPointOf::new(
                in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
                preview_cursor_block,
            );
            let child_logical_location = if child_style.clear == Clear::None {
                child_logical_location
            } else {
                LogicalPointOf::new(
                    child_logical_location.inline,
                    float_exclusions.clearance_block(preview_cursor_block, child_style.clear),
                )
            };
            let inherited = float_exclusions.for_ordinary_child(child_logical_location);
            output =
                tree.compute_child_with_inherited_float_exclusions(child, child_input, inherited)?;
            logical_child_size = constants.flow_axes.logical_size(output.size);
            logical_child_margin = resolve_logical_in_flow_margin(
                parent_logical_unresolved_margin,
                logical_child_size,
                logical_node_inner_size
                    .inline
                    .or(parent_logical_available.inline.into_option()),
            );
            child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        } else if places_against_float_exclusions {
            let preview_top_margin = output
                .block_margin_collapse
                .at(constants.flow_axes.block_start())
                .collapse_with_margin(
                    child_margin.at_physical_side(constants.flow_axes.block_start()),
                );
            let child_margin_can_collapse_with_parent =
                child_margin_can_collapse_with_parent(&child_style);
            let collapsed_margin = if is_collapsing_first_margin {
                if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                    active_margin.resolve()
                } else {
                    active_margin.collapse_with(preview_top_margin).resolve()
                }
            } else {
                active_margin.collapse_with(preview_top_margin).resolve()
            };
            let preview_cursor_block = cursor_block + collapsed_margin;
            let containing_size = constants.containing_size(logical_node_inner_size);
            let preview_logical_location = LogicalPointOf::new(
                in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
                preview_cursor_block,
            );
            let preview_fallback = constants.flow_axes.physical_point(
                preview_logical_location,
                logical_child_size,
                containing_size,
            );
            let inline_size_is_auto =
                parent_inline_preferred_size_is_auto(&child_style, constants.flow_axes);
            let placement = float_exclusions.place_bfc_block(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: true,
                },
                BfcBandCandidate {
                    block_start: preview_cursor_block,
                    size: output.size,
                    margin: child_margin,
                    clear: child_style.clear,
                    fallback: preview_fallback,
                    inline_size_is_auto,
                },
            )?;

            if inline_size_is_auto {
                let parent_non_auto_margin =
                    parent_logical_unresolved_margin.map(resolved_length_auto_fallback_zero);
                let band_child_inline =
                    (placement.available_inline - parent_non_auto_margin.inline_sum()).max(S::ZERO);
                let band_available_child_inline =
                    if child_flow_axes.inline_axis() == constants.flow_axes.inline_axis() {
                        Some(band_child_inline)
                    } else {
                        available_child_inline
                    };
                let band_child_known = in_flow_child_known_size::<Tree, M>(
                    tree,
                    child,
                    &child_style,
                    child_padding + child_border,
                    child_flow_axes,
                    child_logical_node_inner_size,
                    band_available_child_inline,
                )?;
                let mut band_available = child_flow_axes.physical_size(LogicalSizeOf::new(
                    in_flow_child_available_inline(
                        &child_style,
                        child_flow_axes,
                        band_available_child_inline,
                        child_logical_available.inline,
                    ),
                    AvailableOf::<S>::MAX_CONTENT,
                ));
                set_parent_inline_available(
                    &mut band_available,
                    constants.flow_axes,
                    band_child_inline,
                );
                child_input = ComputeInputOf::<S>::for_child(
                    input.run_mode().for_child(),
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    band_child_known,
                    child_parent_size,
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::BlockFlow,
                    ),
                    band_available,
                )
                .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars());
            }

            output = tree.compute_child(child, child_input)?;
            logical_child_size = constants.flow_axes.logical_size(output.size);
            logical_child_margin = resolve_logical_in_flow_margin(
                parent_logical_unresolved_margin,
                logical_child_size,
                logical_node_inner_size
                    .inline
                    .or(parent_logical_available.inline.into_option()),
            );
            child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        }
        if !child_style.float.is_none() {
            let margin_box_inline = logical_child_size.inline + logical_child_margin.inline_sum();
            float_intrinsics.add(margin_box_inline, child_style.float, child_style.clear);
            content_size.inline = content_size.inline.max(float_intrinsics.result());
            if !input.run_mode().is_perform_layout() {
                index += 1;
                continue;
            }
            let pending_float = PendingFloat {
                node: child,
                source_index,
                side: child_style.float,
                clear: child_style.clear,
                block_start: cursor_block,
                size: output.size,
                content_size: output.content_size,
                border: child_border,
                padding: child_padding,
                margin: child_margin,
                style: Box::new(child_style),
                child_compute_geometry: output.scroll_geometry,
            };
            let float_location = float_exclusions.place_float(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: set_layout,
                },
                &pending_float,
            )?;
            if set_layout {
                pending_floats.push(pending_float);
            }
            let containing_size = constants.containing_size(logical_node_inner_size);
            let logical_location =
                constants
                    .flow_axes
                    .logical_point(float_location, output.size, containing_size);
            let content_box_inset = constants.logical_content_box_inset();
            let float_inline_end = logical_location.inline
                + logical_child_size.inline
                + logical_child_margin.inline_end
                - content_box_inset.inline_start;
            let float_block_end =
                logical_location.block + logical_child_size.block + logical_child_margin.block_end;
            content_size.inline = content_size.inline.max(float_inline_end);
            content_size.block = content_size
                .block
                .max(float_block_end - content_box_inset.block_start);
            owned_float_block_end = owned_float_block_end.max(float_block_end);
            index += 1;
            continue;
        }
        resolved_terminal_float_block_end = None;
        let inset_offset = relative_inset_offset(
            constants
                .flow_axes
                .zip_physical_edges_with_inline_extent(
                    child_style.inset,
                    node_inner_size,
                    |length, basis| resolve_auto_optional(length, basis),
                )
                .transpose_with_node(tree, child)?,
            constants.flow_axes,
        );
        let top_margin_set = output
            .block_margin_collapse
            .at(constants.flow_axes.block_start())
            .collapse_with_margin(child_margin.at_physical_side(constants.flow_axes.block_start()));
        let bottom_margin_set = output
            .block_margin_collapse
            .at(constants.flow_axes.block_end())
            .collapse_with_margin(child_margin.at_physical_side(constants.flow_axes.block_end()));
        let child_margin_can_collapse_with_parent =
            child_margin_can_collapse_with_parent(&child_style);
        let base_block = cursor_block;
        let collapsed_margin = if is_collapsing_first_margin {
            if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                top_margin = top_margin.collapse_with(top_margin_set);
            }
            is_collapsing_first_margin = false;
            if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                active_margin.resolve()
            } else {
                active_margin.collapse_with(top_margin_set).resolve()
            }
        } else {
            active_margin.collapse_with(top_margin_set).resolve()
        };
        cursor_block = cursor_block + collapsed_margin;
        let logical_location = LogicalPointOf::new(
            in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
            cursor_block,
        );
        let containing_size = constants.containing_size(logical_node_inner_size);
        let logical_fallback_location = constants.flow_axes.physical_point(
            logical_location,
            logical_child_size,
            containing_size,
        );
        let fallback_location = Point::new(
            logical_fallback_location.x + inset_offset.x,
            logical_fallback_location.y + inset_offset.y,
        );
        let location = if avoids_float_exclusions {
            let placement = float_exclusions.place_bfc_block(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: set_layout,
                },
                BfcBandCandidate {
                    block_start: cursor_block,
                    size: output.size,
                    margin: child_margin,
                    clear: child_style.clear,
                    fallback: Point::new(
                        fallback_location.x - inset_offset.x,
                        fallback_location.y - inset_offset.y,
                    ),
                    inline_size_is_auto: parent_inline_preferred_size_is_auto(
                        &child_style,
                        constants.flow_axes,
                    ),
                },
            )?;
            Point::new(
                placement.location.x + inset_offset.x,
                placement.location.y + inset_offset.y,
            )
        } else if child_style.clear != Clear::None {
            let cleared_logical_location = LogicalPointOf::new(
                logical_location.inline,
                float_exclusions.clearance_block(cursor_block, child_style.clear),
            );
            let cleared_location = constants.flow_axes.physical_point(
                cleared_logical_location,
                logical_child_size,
                containing_size,
            );
            Point::new(
                cleared_location.x + inset_offset.x,
                cleared_location.y + inset_offset.y,
            )
        } else {
            fallback_location
        };
        if set_layout {
            let scroll_geometry = retained_child_scroll_geometry(
                &child_style,
                output.size,
                output.content_size,
                child_padding,
                child_border,
                output.scroll_geometry,
            )
            .map_err(|error| layout_child_geometry_error(node, child, error))?;
            contributions
                .include_in_flow_geometry(location, child_margin, scroll_geometry)
                .map_err(|error| layout_child_geometry_error(node, child, error))?;
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size: output.size,
                    content_size: output.content_size,
                    border: child_border,
                    padding: child_padding,
                    margin: child_margin,
                    ..NodeOutputOf::new()
                }
                .with_scroll_geometry(Some(scroll_geometry)),
            );
        }

        let child_block_end = if avoids_float_exclusions || child_style.clear != Clear::None {
            constants
                .flow_axes
                .logical_point(location, output.size, containing_size)
                .block
                + logical_child_size.block
        } else {
            logical_location.block + logical_child_size.block
        };
        let contribution = content_size_contribution(
            Point::new(
                location.x - constants.content_box_inset.left,
                location.y - constants.content_box_inset.top,
            ),
            output.size,
            output.content_size,
            child_style.overflow,
            child_style.item_is_replaced,
        );
        let logical_contribution = constants.flow_axes.logical_size(contribution);
        let child_inline_content = (logical_child_margin.inline_sum() + logical_child_size.inline)
            .max(logical_contribution.inline + logical_child_margin.inline_end);
        let child_block_content = logical_contribution
            .block
            .max(child_block_end - constants.logical_content_box_inset().block_start);
        content_size.inline = content_size.inline.max(child_inline_content);
        content_size.block = content_size.block.max(child_block_content);
        scroll_content_size.inline = scroll_content_size.inline.max(child_inline_content);
        scroll_content_size.block = scroll_content_size.block.max(child_block_content);
        if let Some(baseline) = output.baselines().first_block_baseline(child_flow_axes) {
            baselines.record_first(baseline.translated(location));
        }
        if let Some(baseline) = output.baselines().last_block_baseline(child_flow_axes) {
            baselines.record_last(baseline.translated(location));
        }
        if output
            .block_margin_collapse
            .can_collapse_through(constants.flow_axes)
        {
            cursor_block = if child_style.clear == Clear::None {
                base_block + logical_child_size.block
            } else {
                child_block_end
            };
            active_margin = active_margin
                .collapse_with(top_margin_set)
                .collapse_with(bottom_margin_set);
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        } else {
            all_in_flow_children_can_collapse_through = false;
            cursor_block = child_block_end;
            active_margin = bottom_margin_set;
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        }
        index += 1;
    }

    Ok(InFlowResult {
        content_size,
        scroll_content_size,
        owned_float_block_end,
        resolved_terminal_float_block_end,
        contributions,
        baselines,
        static_positions,
        pending_floats,
        cursor_block,
        top_margin,
        active_margin,
        active_margin_can_collapse_with_parent,
        all_in_flow_children_can_collapse_through,
    })
}

struct InlineRunPlacement<Node, S: LayoutScalar> {
    size: Size<S>,
    content_size: Size<S>,
    scroll_content_size: Size<S>,
    static_positions: Vec<(Node, Point<S>)>,
    baselines: BaselinesOf<S>,
    first_baseline: Option<S>,
    last_baseline: Option<S>,
    resolved_float_terminal_block_end: Option<S>,
}

impl<Node, S: LayoutScalar> InlineRunPlacement<Node, S> {
    fn logical_block_extent(&self, flow_axes: crate::geometry::FlowAxes) -> S {
        flow_axes.logical_size(self.size).block
    }
}

struct InlineRunContext<'a, S: LayoutScalar> {
    source_index_start: usize,
    cursor_block: S,
    owned_float_block_end: S,
    constants: &'a Constants<S>,
    input: ComputeInputOf<S>,
    node_inner_size: Size<Option<S>>,
    set_layout: bool,
}

fn resolved_inline_float_terminal_block_end<S: LayoutScalar>(
    report: &crate::inline::MixedInlineRunReportOf<S>,
    cursor_block: S,
    owned_float_block_end: S,
) -> Option<S> {
    let terminal_line_block_end = cursor_block + report.block_extent;
    report.float_edge_phase.map(|phase| {
        if terminal_line_block_end > owned_float_block_end {
            terminal_line_block_end
        } else {
            (terminal_line_block_end + phase).max(owned_float_block_end)
        }
    })
}

fn forced_line_break_control<S: LayoutScalar>(
    source_index: usize,
    input: LineBreakInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> ForcedLineBreakControlOf<S> {
    ForcedLineBreakControlOf::new(
        source_index,
        InlineFlowOf::new(
            input.writing_mode(),
            input.direction(),
            available_inline_extent,
        ),
        input.metrics(),
        InlineControlAlignment::from(input.vertical_align()),
        input.clear(),
    )
}

fn inline_boundary_control<S: LayoutScalar>(
    source_index: usize,
    input: InlineBoundaryInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> InlineBoundaryControlOf<S> {
    InlineBoundaryControlOf::new(
        source_index,
        input.kind(),
        InlineFlowOf::new(
            input.writing_mode(),
            input.direction(),
            available_inline_extent,
        ),
        input.metrics(),
        InlineControlAlignment::from(input.vertical_align()),
    )
}

fn layout_inline_run_children<Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    run: &[<Tree as Traverse>::Node],
    context: InlineRunContext<'_, S>,
    float_exclusions: &FloatExclusions<S, <Tree as Traverse>::Node>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InlineRunPlacement<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let InlineRunContext {
        source_index_start,
        cursor_block,
        owned_float_block_end,
        constants,
        input,
        node_inner_size,
        set_layout,
    } = context;
    let logical_node_inner_size = constants.flow_axes.logical_size(node_inner_size);
    let available_inline_extent = logical_node_inner_size
        .inline
        .map(AvailableOf::<S>::definite)
        .unwrap_or(
            constants
                .flow_axes
                .logical_size(constants.available_content)
                .inline,
        );
    let containing_size = constants.containing_size(logical_node_inner_size);
    let mut participants = Vec::new();
    let mut atomic_children = Vec::new();
    let mut control_children = Vec::new();
    let mut published_text = Vec::new();
    let mut static_positions = Vec::new();
    for (offset, child) in run.iter().copied().enumerate() {
        let source_index = source_index_start + offset;
        let child_style = match tree.layout_input(child) {
            LayoutInputOf::InlineText(text) => {
                published_text.push((child, source_index, Vec::new(), None, None));
                participants.extend(text.segments().iter().copied().map(|segment| {
                    MixedInlineParticipantOf::ShapedText(ShapedTextParticipantOf {
                        source_index,
                        segment,
                    })
                }));
                continue;
            }
            LayoutInputOf::Box(style) => *style,
            LayoutInputOf::LineBreak(line_break) => {
                if line_break.display().is_none() {
                    if set_layout {
                        tree.set_unrounded(
                            child,
                            NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                source_index,
                            )),
                        );
                    }
                    continue;
                }
                let line_break = visible_line_break_in_flow(
                    tree,
                    child,
                    constants.writing_mode,
                    constants.direction,
                )
                .expect("visible line-break input remains visible after validation");
                participants.push(MixedInlineParticipantOf::ForcedLineBreak(
                    forced_line_break_control(source_index, line_break, available_inline_extent),
                ));
                control_children.push((child, source_index));
                continue;
            }
            LayoutInputOf::InlineBoundary(_) => {
                let boundary = visible_inline_boundary_in_flow(
                    tree,
                    child,
                    constants.writing_mode,
                    constants.direction,
                )
                .expect("inline-boundary input remains present after validation");
                participants.push(MixedInlineParticipantOf::Boundary(inline_boundary_control(
                    source_index,
                    boundary,
                    available_inline_extent,
                )));
                control_children.push((child, source_index));
                continue;
            }
        };
        if child_style.display == super::Display::None {
            if set_layout {
                tree.set_unrounded(
                    child,
                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(source_index)),
                );
                tree.compute_child(
                    child,
                    ComputeInputOf::<S>::hidden_in_containing_pass(
                        ContainingLayoutContext::new(
                            constants.flow_axes,
                            ParentFormattingContext::BlockFlow,
                        ),
                        input.settled_auto_scrollbars(),
                    ),
                )?;
            }
            continue;
        }
        if child_style.position == Position::Absolute {
            static_positions.push((
                child,
                absolute_static_position(cursor_block, constants, containing_size),
            ));
            continue;
        }

        let participation = child_style.atomic_inline_participation.ok_or_else(|| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(child),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::AtomicInlineParticipation {
                    reason: AtomicInlineParticipationRoleError::MissingForAtomicInline,
                }),
            )
        })?;
        let child_padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.padding,
                node_inner_size,
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let child_border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.border,
                node_inner_size,
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let output = tree.compute_child(
            child,
            ComputeInputOf::<S>::for_child(
                input.run_mode().for_child(),
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                constants.definite_child_containing_block_size(),
                ContainingLayoutContext::new(
                    constants.flow_axes,
                    ParentFormattingContext::BlockFlow,
                ),
                constants.flow_axes.physical_size(LogicalSizeOf::new(
                    available_inline_extent,
                    AvailableOf::<S>::MAX_CONTENT,
                )),
            )
            .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars()),
        )?;
        let unresolved_margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                child_style.margin,
                node_inner_size,
                |length, basis| resolve_auto_optional(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let child_margin = resolve_atomic_inline_margin(unresolved_margin);
        let item = atomic_inline_box_participant(
            source_index,
            child_style.clone(),
            output,
            child_margin,
            child_padding,
            child_border,
            constants.flow_axes,
        );
        participants.push(MixedInlineParticipantOf::Atomic {
            item,
            participation,
        });
        atomic_children.push((child, source_index, child_style, output));
    }
    let logical_content_box_inset = constants.logical_content_box_inset();
    let mut provider_error = None;
    let report = layout_mixed_inline_run_with_band_source(
        MixedInlineRunInputOf {
            available_inline_extent,
            flow_axes: constants.flow_axes,
            text_align: constants.text_align,
            participants,
        },
        |block_start, block_end| {
            let query_block_start = cursor_block + block_start;
            let query_block_end = cursor_block + block_end;
            let band = if !set_layout || provider_error.is_some() {
                float_exclusions.query_rectangular_line_band(query_block_start, query_block_end)
            } else {
                match float_exclusions.query_provider_band(
                    tree,
                    container,
                    query_block_start,
                    query_block_end,
                ) {
                    Ok(band) => band,
                    Err(error) => {
                        provider_error = Some(error);
                        FloatBand {
                            inline_start: float_exclusions.containing_inline_start,
                            inline_end: float_exclusions.containing_inline_end,
                            next_transition: None,
                            #[cfg(test)]
                            evaluated: 0,
                        }
                    }
                }
            };
            LogicalLineBandQueryResultOf {
                inline_start: band.inline_start - logical_content_box_inset.inline_start,
                inline_end: band.inline_end - logical_content_box_inset.inline_start,
                next_transition: band
                    .next_transition
                    .map(|transition| transition - cursor_block),
            }
        },
        |block, clear| {
            float_exclusions.clearance_for_line_intent(cursor_block + block, clear) - cursor_block
        },
    );
    if let Some(error) = provider_error {
        return Err(error);
    }
    let resolved_float_terminal_block_end =
        resolved_inline_float_terminal_block_end(&report, cursor_block, owned_float_block_end);
    let report_logical_size = LogicalSizeOf::new(report.inline_extent, report.block_extent);
    let report_size = constants.flow_axes.physical_size(report_logical_size);
    let project_point = |inline: S, block: S, size: LogicalSizeOf<S>| {
        constants.flow_axes.physical_point(
            LogicalPointOf::new(
                logical_content_box_inset.inline_start + inline,
                cursor_block + block,
            ),
            size,
            containing_size,
        )
    };

    let mut text_content_size = Size::ZERO;
    for source in &report.fragments {
        let logical_size = LogicalSizeOf::new(source.inline_extent, source.block_extent);
        let size = constants.flow_axes.physical_size(logical_size);
        let location = project_point(source.inline_start, source.block_start, logical_size);
        let rect = super::ScrollRectOf::try_new(location, size).map_err(|error| {
            block_inline_geometry_error(
                container,
                run.get(source.source_index - source_index_start).copied(),
                input.run_mode(),
                error,
            )
        })?;
        let baseline = project_point(
            source.inline_start,
            source.baseline,
            LogicalSizeOf::new(S::ZERO, S::ZERO),
        );
        let (_, _, fragments, union_min, union_max) = published_text
            .iter_mut()
            .find(|(_, source_index, _, _, _)| *source_index == source.source_index)
            .expect("every shaped source retains its text publication group");
        *union_min = Some(union_min.map_or(location, |current: Point<S>| {
            Point::new(current.x.min(location.x), current.y.min(location.y))
        }));
        let maximum = Point::new(location.x + size.width, location.y + size.height);
        *union_max = Some(union_max.map_or(maximum, |current: Point<S>| {
            Point::new(current.x.max(maximum.x), current.y.max(maximum.y))
        }));
        fragments.push(InlineFragmentOutputOf::new(
            source.segment_id,
            rect,
            baseline,
            source.line_index,
            source.visual_index,
            source.replacement_inline_extent,
        ));
        text_content_size = max_content_size(
            text_content_size,
            content_size_contribution(
                Point::new(
                    location.x - constants.content_box_inset.left,
                    location.y - constants.content_box_inset.top,
                ),
                size,
                size,
                ComputedOverflow::VISIBLE,
                false,
            ),
        );
        if set_layout {
            contributions.include_direct_line(rect);
        }
    }

    if set_layout {
        for (child, source_index, fragments, union_min, union_max) in published_text {
            let anchor = report
                .anchors
                .iter()
                .find(|anchor| anchor.source_index == source_index)
                .map_or(Point::ZERO, |anchor| {
                    project_point(
                        anchor.inline_start,
                        anchor.block_start,
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                    )
                });
            let (text_location, text_size) = match (union_min, union_max) {
                (Some(minimum), Some(maximum)) => (
                    minimum,
                    Size::new(maximum.x - minimum.x, maximum.y - minimum.y),
                ),
                _ => (anchor, Size::ZERO),
            };
            tree.compute_child(
                child,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::ContentSize,
                    RequestedAxis::Both,
                    text_size.map(Some),
                    node_inner_size,
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::BlockFlow,
                    ),
                    constants.available_content,
                )
                .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
            )?;
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location: text_location,
                    size: text_size,
                    content_size: text_size,
                    ..NodeOutputOf::new()
                },
            );
            tree.set_unrounded_inline_fragment_state(child, Some(fragments));
        }
    }

    let atomic_sources = report
        .atomics
        .iter()
        .map(|source| (source.item.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    let mut content_size = report_size;
    let mut scroll_content_size = text_content_size;
    for (child, source_index, child_style, output) in atomic_children {
        let source = atomic_sources[&source_index];
        let logical_size = constants.flow_axes.logical_size(source.item.size);
        let projected_location =
            project_point(source.inline_start, source.block_start, logical_size);
        let inset_offset = relative_inset_offset(
            constants
                .flow_axes
                .zip_physical_edges_with_inline_extent(
                    child_style.inset,
                    node_inner_size,
                    |length, basis| resolve_auto_optional(length, basis),
                )
                .transpose_with_node(tree, child)?,
            constants.flow_axes,
        );
        let location = Point::new(
            projected_location.x + inset_offset.x,
            projected_location.y + inset_offset.y,
        );
        let contribution = content_size_contribution(
            Point::new(
                location.x - constants.content_box_inset.left,
                location.y - constants.content_box_inset.top,
            ),
            source.item.size,
            output.content_size,
            child_style.overflow,
            child_style.item_is_replaced,
        );
        content_size = max_content_size(content_size, contribution);
        scroll_content_size = max_content_size(scroll_content_size, contribution);

        if set_layout {
            let scroll_geometry = retained_child_scroll_geometry(
                &child_style,
                source.item.size,
                source.item.content_size,
                source.item.padding,
                source.item.border,
                output.scroll_geometry,
            )
            .map_err(|error| layout_child_geometry_error(container, child, error))?;
            contributions
                .include_in_flow_geometry(location, source.item.margin, scroll_geometry)
                .map_err(|error| layout_child_geometry_error(container, child, error))?;
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size: source.item.size,
                    content_size: source.item.content_size,
                    border: source.item.border,
                    padding: source.item.padding,
                    margin: source.item.margin,
                    ..NodeOutputOf::new()
                }
                .with_scroll_geometry(Some(scroll_geometry)),
            );
        }
    }

    let control_sources = report
        .controls
        .iter()
        .map(|source| (source.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    if set_layout {
        for (child, source_index) in control_children {
            let source = control_sources[&source_index];
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location: project_point(
                        source.inline_start,
                        source.block_start,
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                    ),
                    ..NodeOutputOf::new()
                },
            );
        }
    }

    let projected_baseline = |baseline| {
        constants.flow_axes.block_axis_coordinate(project_point(
            S::ZERO,
            baseline,
            LogicalSizeOf::new(S::ZERO, S::ZERO),
        ))
    };
    let baselines = BaselinesOf::from_block_coordinates(
        constants.flow_axes,
        report.first_baseline.map(projected_baseline),
        report.last_baseline.map(projected_baseline),
    );

    Ok(InlineRunPlacement {
        size: report_size,
        content_size,
        scroll_content_size,
        static_positions,
        baselines,
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
        resolved_float_terminal_block_end,
    })
}

fn record_inline_run_baselines<S: LayoutScalar>(
    baselines: &mut BaselinesOf<S>,
    placement: &InlineRunPlacement<impl Copy, S>,
    cursor_block: S,
    constants: &Constants<S>,
) {
    if constants.flow_axes.inline_axis() == PhysicalAxis::Vertical {
        baselines.record_first(placement.baselines.first);
        baselines.record_last(placement.baselines.last);
        return;
    }

    if let Some(baseline) = placement.first_baseline {
        baselines.record_first(
            BaselinesOf::from_block_coordinates(
                constants.flow_axes,
                Some(cursor_block + baseline),
                None,
            )
            .first,
        );
    }
    if let Some(baseline) = placement.last_baseline {
        baselines.record_last(
            BaselinesOf::from_block_coordinates(
                constants.flow_axes,
                None,
                Some(cursor_block + baseline),
            )
            .last,
        );
    }
}

fn layout_floats<Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    floats: &[PendingFloat<<Tree as Traverse>::Node, S>],
    container_size: Size<S>,
    constants: &Constants<S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let logical_container_size = constants.flow_axes.logical_size(container_size);
    let logical_inset = constants.logical_content_box_inset();
    let mut float_exclusions = FloatExclusions::new(
        constants.flow_axes,
        container_size,
        (logical_container_size.inline - logical_inset.inline_sum()).max(S::ZERO),
        logical_inset,
    );

    for float in floats {
        let location = float_exclusions.place_float(
            ProviderBandContext {
                tree,
                container,
                enabled: true,
            },
            float,
        )?;
        let scroll_geometry = retained_child_scroll_geometry(
            &float.style,
            float.size,
            float.content_size,
            float.padding,
            float.border,
            float.child_compute_geometry,
        )
        .map_err(|error| layout_child_geometry_error(container, float.node, error))?;
        contributions
            .include_in_flow_geometry(location, float.margin, scroll_geometry)
            .map_err(|error| layout_child_geometry_error(container, float.node, error))?;
        tree.set_unrounded(
            float.node,
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(float.source_index),
                location,
                size: float.size,
                content_size: float.content_size,
                border: float.border,
                padding: float.padding,
                margin: float.margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        );
    }

    Ok(())
}

struct FloatIntrinsics<S: LayoutScalar> {
    available_inline: AvailableOf<S>,
    contribution: S,
    line_start: S,
    line_end: S,
}

impl<S: LayoutScalar> FloatIntrinsics<S> {
    const fn new(available_inline: AvailableOf<S>) -> Self {
        Self {
            available_inline,
            contribution: S::ZERO,
            line_start: S::ZERO,
            line_end: S::ZERO,
        }
    }

    fn add(&mut self, width: S, float: Float, clear: Clear) {
        match self.available_inline {
            AvailableOf::<S>::Definite(_) => {}
            AvailableOf::<S>::MinContent => self.contribution = self.contribution.max(width),
            AvailableOf::<S>::MaxContent => {
                match clear {
                    Clear::None => {}
                    Clear::Left => self.line_start = S::ZERO,
                    Clear::Right => self.line_end = S::ZERO,
                    Clear::Both => {
                        self.line_start = S::ZERO;
                        self.line_end = S::ZERO;
                    }
                }
                match float {
                    Float::Left | Float::None => self.line_start = self.line_start + width,
                    Float::Right => self.line_end = self.line_end + width,
                }
                self.contribution = self.contribution.max(self.line_start + self.line_end);
            }
        }
    }

    const fn result(&self) -> S {
        self.contribution
    }
}

fn child_margin_can_collapse_with_parent<S: LayoutScalar>(style: &NodeInputOf<S>) -> bool {
    style.display == super::Display::Block && style.position == Position::Relative
}

fn block_child_avoids_float_exclusions<S: LayoutScalar>(style: &NodeInputOf<S>) -> bool {
    style.display != super::Display::None
        && !style.display.is_inline_level()
        && style.position != Position::Absolute
        && style.float.is_none()
        && (matches!(
            style.display,
            super::Display::Flex | super::Display::Grid | super::Display::GridLanes
        ) || (!style.item_is_replaced
            && style.overflow.establishes_independent_formatting_context()))
}

fn parent_inline_preferred_size_is_auto<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    parent_flow_axes: crate::geometry::FlowAxes,
) -> bool {
    match parent_flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => style.size.width.is_auto(),
        PhysicalAxis::Vertical => style.size.height.is_auto(),
    }
}

fn set_parent_inline_available<S: LayoutScalar>(
    available: &mut Size<AvailableOf<S>>,
    parent_flow_axes: crate::geometry::FlowAxes,
    value: S,
) {
    match parent_flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => available.width = AvailableOf::definite(value),
        PhysicalAxis::Vertical => available.height = AvailableOf::definite(value),
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the private child-size helper preserves the session's generic error envelope"
)]
fn in_flow_child_known_size<Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    padding_border: Edges<Tree::Scalar>,
    child_flow_axes: crate::geometry::FlowAxes,
    parent: LogicalSizeOf<Option<Tree::Scalar>>,
    available_inline: Option<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Option<Tree::Scalar>>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let parent = child_flow_axes.physical_size(parent);
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let min_size = resolve_minimum_size(&style.min_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let mut max_size = resolve_maximum_size(&style.max_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .add_optional(box_sizing_adjustment);
    let aspect_height_limit = style
        .aspect_ratio
        .zip(max_size.height)
        .and_then(|(ratio, height)| max_size.width.is_none().then_some(height * ratio.get()));
    if let Some(width) = aspect_height_limit {
        max_size.width = Some(width);
    }
    let known = resolve_preferred_size(&style.size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .clamp_max_before_min_optional(min_size, max_size);

    let mut known = child_flow_axes.logical_size(known);
    let min_size = child_flow_axes.logical_size(min_size);
    let max_size = child_flow_axes.logical_size(max_size);
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => style.size.width.clone(),
        crate::PhysicalAxis::Vertical => style.size.height.clone(),
    };
    if !style.item_is_table
        && !style.item_is_replaced
        && known.inline.is_none()
        && !inline_size.is_min_content()
        && !inline_size.is_max_content()
    {
        known.inline = available_inline
            .map(|inline| inline.clamp_max_before_min_optional(min_size.inline, max_size.inline));
        if aspect_height_limit.is_some() {
            let physical_known = child_flow_axes.physical_size(known);
            known = child_flow_axes.logical_size(
                physical_known
                    .apply_aspect_ratio(style.aspect_ratio)
                    .clamp_max_before_min_optional(
                        child_flow_axes.physical_size(min_size),
                        child_flow_axes.physical_size(max_size),
                    ),
            );
        }
    }

    Ok(child_flow_axes.physical_size(known))
}

fn in_flow_child_available_inline<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    child_flow_axes: crate::geometry::FlowAxes,
    available_inline: Option<S>,
    fallback: AvailableOf<S>,
) -> AvailableOf<S> {
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => style.size.width.clone(),
        crate::PhysicalAxis::Vertical => style.size.height.clone(),
    };
    if inline_size.is_min_content() {
        AvailableOf::<S>::MIN_CONTENT
    } else if inline_size.is_max_content() {
        AvailableOf::<S>::MAX_CONTENT
    } else {
        available_inline
            .map(AvailableOf::<S>::definite)
            .unwrap_or(fallback)
    }
}

fn relative_inset_offset<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    flow_axes: crate::geometry::FlowAxes,
) -> Point<S> {
    let logical_inset = flow_axes.logical_edges(inset);
    let logical_offset = LogicalPointOf::new(
        logical_inset
            .inline_start
            .or_else(|| logical_inset.inline_end.map(|end| -end))
            .unwrap_or(S::ZERO),
        logical_inset
            .block_start
            .or_else(|| logical_inset.block_end.map(|end| -end))
            .unwrap_or(S::ZERO),
    );
    flow_axes.physical_point(
        logical_offset,
        LogicalSizeOf::new(S::ZERO, S::ZERO),
        Size::ZERO,
    )
}

pub(super) fn resolve_logical_in_flow_margin<S: LayoutScalar>(
    margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    child_size: LogicalSizeOf<S>,
    container_inline: Option<S>,
) -> LogicalEdgesOf<S> {
    let non_auto_inline = resolved_length_auto_fallback_zero(margin.inline_start)
        + resolved_length_auto_fallback_zero(margin.inline_end);
    let auto_count = usize::from(matches!(margin.inline_start, ResolvedLengthAutoOf::Auto))
        + usize::from(matches!(margin.inline_end, ResolvedLengthAutoOf::Auto));
    let auto_inline = if auto_count == 0 {
        S::ZERO
    } else {
        container_inline
            .map(|inline| (inline - child_size.inline - non_auto_inline).max(S::ZERO))
            .unwrap_or(S::ZERO)
            / S::from_usize(auto_count)
    };

    LogicalEdgesOf::new(
        resolved_length_auto_or(margin.inline_start, auto_inline),
        resolved_length_auto_or(margin.inline_end, auto_inline),
        resolved_length_auto_fallback_zero(margin.block_start),
        resolved_length_auto_fallback_zero(margin.block_end),
    )
}

fn resolved_length_auto_or<S: LayoutScalar>(value: ResolvedLengthAutoOf<S>, auto_fallback: S) -> S {
    match value {
        ResolvedLengthAutoOf::Auto => auto_fallback,
        ResolvedLengthAutoOf::Resolved(value) => value,
        // Missing-basis symbolic margins keep the algorithm's historical
        // unresolved-as-zero fallback and do not participate in auto distribution.
        ResolvedLengthAutoOf::Unresolved(
            UnresolvedLengthReason::Basis | UnresolvedLengthReason::InvalidNumeric,
        ) => S::ZERO,
    }
}

fn resolved_length_auto_fallback_zero<S: LayoutScalar>(value: ResolvedLengthAutoOf<S>) -> S {
    resolved_length_auto_or(value, S::ZERO)
}

fn atomic_inline_box_participant<S: LayoutScalar>(
    source_index: usize,
    child_style: NodeInputOf<S>,
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
    containing_flow_axes: crate::geometry::FlowAxes,
) -> AtomicInlineBoxParticipant<S> {
    let logical_size = containing_flow_axes.logical_size(output.size);
    let used_overflow =
        UsedOverflow::from_computed(child_style.overflow, child_style.item_is_replaced);
    let block_overflow = match containing_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => used_overflow.x(),
        PhysicalAxis::Vertical => used_overflow.y(),
    };
    let selected_inner_baseline = (child_style.vertical_align == VerticalAlign::Baseline
        && block_overflow.value() == Overflow::Visible)
        .then(|| {
            let baselines = output.baselines();
            containing_flow_axes
                .block_axis_coordinate(baselines.first)
                .or_else(|| containing_flow_axes.block_axis_coordinate(baselines.last))
                .map(|physical| {
                    if containing_flow_axes
                        .logical_axis_progression(crate::LogicalAxis::Block)
                        .is_decreasing()
                    {
                        logical_size.block - physical
                    } else {
                        physical
                    }
                })
        })
        .flatten();
    AtomicInlineBoxParticipant {
        source_index,
        size: output.size,
        content_size: output.content_size,
        margin,
        padding,
        border,
        scrollbar_size: child_scrollbar_size(&child_style),
        first_baseline: selected_inner_baseline,
        alignment: child_style.vertical_align.into(),
    }
}

fn resolve_atomic_inline_margin<S: LayoutScalar>(margin: Edges<Option<S>>) -> Edges<S> {
    margin.map(|value| value.unwrap_or(S::ZERO))
}

fn in_flow_child_inline_offset<S: LayoutScalar>(
    size: LogicalSizeOf<S>,
    margin: LogicalEdgesOf<S>,
    constants: &Constants<S>,
) -> S {
    let logical_content_box_inset = constants.logical_content_box_inset();
    let logical_inner_size = constants.logical_node_inner_size();
    let mut inline = logical_content_box_inset.inline_start + margin.inline_start;

    let container_inner_inline = logical_inner_size
        .inline
        .or_else(|| {
            constants
                .logical_node_outer_size()
                .inline
                .map(|inline| inline - logical_content_box_inset.inline_sum())
        })
        .unwrap_or(size.inline + margin.inline_sum());
    let item_outer_inline = size.inline + margin.inline_sum();
    if item_outer_inline < container_inner_inline {
        let free_space = container_inner_inline - item_outer_inline;
        match constants.text_align {
            TextAlign::Auto => {}
            TextAlign::LegacyCenter => inline = inline + free_space / S::from_f64(2.0),
            TextAlign::LegacyLeft
                if constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Inline)
                    .is_decreasing() =>
            {
                inline = inline + free_space;
            }
            TextAlign::LegacyRight
                if !constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Inline)
                    .is_decreasing() =>
            {
                inline = inline + free_space;
            }
            TextAlign::LegacyLeft | TextAlign::LegacyRight => {}
        }
    }

    inline
}

fn absolute_static_position<S: LayoutScalar>(
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

fn content_size_contribution<S: LayoutScalar>(
    location: Point<S>,
    size: Size<S>,
    content_size: Size<S>,
    overflow: ComputedOverflow,
    item_is_replaced: bool,
) -> Size<S> {
    let overflow = UsedOverflow::from_computed(overflow, item_is_replaced);
    let contribution_size = Size::new(
        if overflow.x().value() == Overflow::Visible {
            size.width.max(content_size.width)
        } else {
            size.width
        },
        if overflow.y().value() == Overflow::Visible {
            size.height.max(content_size.height)
        } else {
            size.height
        },
    );
    let max_x = (location.x + contribution_size.width).max(S::ZERO);
    let min_x = location.x.min(S::ZERO);
    let max_y = (location.y + contribution_size.height).max(S::ZERO);
    let min_y = location.y.min(S::ZERO);
    Size::new(max_x - min_x, max_y - min_y)
}

fn block_final_in_flow_end<S: LayoutScalar>(
    content_box: super::ScrollRectOf<S>,
    flow_axes: crate::FlowAxes,
    axis: crate::LogicalAxis,
    extent: S,
) -> S {
    let origin = content_box.origin();
    let size = content_box.size();
    let side = match axis {
        crate::LogicalAxis::Inline => flow_axes.inline_end(),
        crate::LogicalAxis::Block => flow_axes.block_end(),
    };
    match side {
        PhysicalSide::Top => origin.y + size.height - extent,
        PhysicalSide::Right => origin.x + extent,
        PhysicalSide::Bottom => origin.y + extent,
        PhysicalSide::Left => origin.x + size.width - extent,
    }
}

fn block_scroll_geometry<Tree, S, M>(
    node: <Tree as Traverse>::Node,
    run_mode: RunMode,
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    contributions: ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, super::ScrollGeometryOf<S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let target_border_box = super::ScrollRectOf::try_new(Point::ZERO, output_size)
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes: constants.flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: output_size,
        border: constants.border,
        padding: constants.padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: constants.settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
        contributions,
        origin_axes: ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
        scroll_snap_type: style.scroll_snap_type,
        target_border_box,
        target_scroll_margin: style.scroll_margin,
        target_flow_axes: constants.flow_axes,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
}

fn block_inline_geometry_error<Node, S, M, E>(
    container: Node,
    subject: Option<Node>,
    run_mode: RunMode,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    match subject {
        Some(subject) => layout_child_geometry_error(container, subject, error),
        None => layout_own_geometry_error(container, run_mode, error),
    }
}

fn retained_child_scroll_geometry<S: LayoutScalar>(
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

    let flow_axes = crate::FlowAxes::new(style.writing_mode, style.direction);
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
    contributions.exclude_reserved_gutter_from_range();
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
        scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
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

fn max_content_size<S: LayoutScalar>(a: Size<S>, b: Size<S>) -> Size<S> {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

fn layout_absolute_children<Tree, S, M>(
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
        if style.position != Position::Absolute || style.display == super::Display::None {
            continue;
        }

        let padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.padding,
                area_size.map(Some),
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.border,
                area_size.map(Some),
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let unresolved_margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                style.margin,
                area_size.map(Some),
                |length, basis| resolve_auto_optional(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(S::ZERO));
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::ZERO
        };
        let min_size = resolve_minimum_size(
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
        let max_size = resolve_maximum_size(
            &style.max_size,
            area_size.map(Some),
            SizingAlgorithm::Positioned,
            true,
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let style_size = resolve_preferred_size(
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
            .zip_size(area_size.map(Some), |length, basis| {
                resolve_auto_optional(length, basis)
            })
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
            &style,
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
                let style_size = resolve_preferred_size(
                    &style.size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let min_size = resolve_minimum_size(
                    &style.min_size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let max_size = resolve_maximum_size(
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

fn child_scrollbar_size<S: LayoutScalar>(style: &NodeInputOf<S>) -> Size<S> {
    scrollbar_size_from_overflow(
        style.overflow,
        style.item_is_replaced,
        style.scrollbar_width.get(),
    )
}

fn resolve_preferred_size<S: LayoutScalar>(
    size: &Size<super::PreferredSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_preferred_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_preferred_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

fn resolve_minimum_size<S: LayoutScalar>(
    size: &Size<super::MinSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_minimum_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_minimum_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

fn resolve_maximum_size<S: LayoutScalar>(
    size: &Size<super::MaxSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_maximum_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_maximum_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

fn resolve_auto_optional<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    resolution_optional(length.resolve_with_status(basis))
}

fn resolve_length_or_zero<S: LayoutScalar>(
    length: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
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

trait BlockOptionalSizeExt<S: LayoutScalar> {
    fn sub_optional_clamped_to_zero(self, amount: Size<S>) -> Self;
    fn max_optional(self, min: Self) -> Self;
}

impl<S: LayoutScalar> BlockOptionalSizeExt<S> for Size<Option<S>> {
    fn sub_optional_clamped_to_zero(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| (width - amount.width).max(S::ZERO)),
            self.height
                .map(|height| (height - amount.height).max(S::ZERO)),
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

#[cfg(test)]
mod fri06_c13_t05_characterization_tests {
    use super::*;
    use crate::AspectRatioOf;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let optional = Size::new(Some(scalar(8.0)), None);

        assert_eq!(
            optional.or(Size::new(Some(scalar(3.0)), Some(scalar(5.0)))),
            Size::new(Some(scalar(8.0)), Some(scalar(5.0)))
        );
        assert_eq!(
            optional.unwrap_or(Size::new(scalar(13.0), scalar(21.0))),
            Size::new(scalar(8.0), scalar(21.0))
        );
        assert_eq!(
            optional.add_optional(Size::new(scalar(2.0), scalar(3.0))),
            Size::new(Some(scalar(10.0)), None)
        );

        let Some(ratio) = AspectRatioOf::new(scalar(2.0)) else {
            panic!("finite positive test aspect ratio must be accepted");
        };
        assert_eq!(
            Size::new(Some(scalar(12.0)), None).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(12.0)), Some(scalar(6.0)))
        );
        assert_eq!(
            Size::new(None, Some(scalar(7.0))).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(14.0)), Some(scalar(7.0)))
        );

        assert_eq!(
            Size::new(Some(scalar(2.0)), Some(scalar(9.0)))
                .sub_optional_clamped_to_zero(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(Some(S::ZERO), Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_max_before_min_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            scalar(5.0).clamp_max_before_min_optional(Some(scalar(10.0)), Some(scalar(3.0))),
            scalar(10.0)
        );
    }

    #[test]
    fn fri06_c13_t05_block_optional_math_and_zero_clamp_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_block_optional_math_and_zero_clamp_preserve_f64() {
        characterize::<f64>();
    }
}
