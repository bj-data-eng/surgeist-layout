use crate::node_projection::CommonBoxProjection;
use crate::scroll::{ScrollBoxProjection, ScrollTargetProjection};
use crate::{
    AspectRatioOf, AtomicInlineParticipationOf, BoxSizing, Clear, Compute, ComputedOverflow,
    Direction, Display, Edges, Float, FloatExclusion, LayoutScalar, LengthAutoOf, LengthOf,
    MaxSizeOf, MinSizeOf, NodeInputOf, Position, PreferredSizeOf, ScrollbarGutter,
    ScrollbarWidthOf, Size, TextAlign, Traverse, VerticalAlign, WritingMode,
};

/// Block-container facts settled at the block algorithm entry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlockContainerProjection<S: LayoutScalar> {
    pub(super) size: Size<PreferredSizeOf<S>>,
    pub(super) min_size: Size<MinSizeOf<S>>,
    pub(super) max_size: Size<MaxSizeOf<S>>,
    pub(super) aspect_ratio: Option<AspectRatioOf<S>>,
    pub(super) margin: Edges<LengthAutoOf<S>>,
    pub(super) padding: Edges<LengthOf<S>>,
    pub(super) border: Edges<LengthOf<S>>,
    pub(super) box_sizing: BoxSizing,
    pub(super) overflow: ComputedOverflow,
    pub(super) position: Position,
    pub(super) item_is_replaced: bool,
    pub(super) display: Display,
    pub(super) direction: Direction,
    pub(super) writing_mode: WritingMode,
    pub(super) text_align: TextAlign,
    pub(super) scrollbar_gutter: ScrollbarGutter,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
}

impl<S: LayoutScalar> BlockContainerProjection<S> {
    #[must_use]
    pub(super) fn from_node(input: &NodeInputOf<S>) -> Self {
        let common = CommonBoxProjection::from_node(input);
        Self {
            size: common.size.clone(),
            min_size: common.min_size.clone(),
            max_size: common.max_size.clone(),
            aspect_ratio: *common.aspect_ratio,
            margin: *common.margin,
            padding: *common.padding,
            border: *common.border,
            box_sizing: common.box_sizing,
            overflow: common.overflow,
            position: common.position,
            item_is_replaced: common.item_is_replaced,
            display: input.display,
            direction: input.direction,
            writing_mode: input.writing_mode,
            text_align: input.text_align,
            scrollbar_gutter: input.scrollbar_gutter,
            scrollbar_width: input.scrollbar_width,
        }
    }
}

/// Block-child facts retained after child role lookup.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlockChildProjection<S: LayoutScalar> {
    pub(super) size: Size<PreferredSizeOf<S>>,
    pub(super) min_size: Size<MinSizeOf<S>>,
    pub(super) max_size: Size<MaxSizeOf<S>>,
    pub(super) aspect_ratio: Option<AspectRatioOf<S>>,
    pub(super) margin: Edges<LengthAutoOf<S>>,
    pub(super) padding: Edges<LengthOf<S>>,
    pub(super) border: Edges<LengthOf<S>>,
    pub(super) box_sizing: BoxSizing,
    pub(super) overflow: ComputedOverflow,
    pub(super) position: Position,
    pub(super) inset: Edges<LengthAutoOf<S>>,
    pub(super) item_is_replaced: bool,
    pub(super) item_is_table: bool,
    pub(super) display: Display,
    pub(super) flow_axes: crate::FlowAxes,
    pub(super) float: Float,
    pub(super) clear: Clear,
    pub(super) float_exclusion: FloatExclusion,
    pub(super) atomic_inline_participation: Option<AtomicInlineParticipationOf<S>>,
    pub(super) vertical_align: VerticalAlign,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
}

impl<S: LayoutScalar> BlockChildProjection<S> {
    #[must_use]
    pub(crate) fn from_node(input: &NodeInputOf<S>) -> Self {
        let common = CommonBoxProjection::from_node(input);
        Self {
            size: common.size.clone(),
            min_size: common.min_size.clone(),
            max_size: common.max_size.clone(),
            aspect_ratio: *common.aspect_ratio,
            margin: *common.margin,
            padding: *common.padding,
            border: *common.border,
            box_sizing: common.box_sizing,
            overflow: common.overflow,
            position: common.position,
            inset: *common.inset,
            item_is_replaced: common.item_is_replaced,
            item_is_table: common.item_is_table,
            display: input.display,
            flow_axes: common.flow_axes,
            float: input.float,
            clear: input.clear,
            float_exclusion: input.float_exclusion,
            atomic_inline_participation: input.atomic_inline_participation,
            vertical_align: input.vertical_align,
            scrollbar_width: input.scrollbar_width,
        }
    }
}

#[must_use]
pub(super) fn block_container_projection<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
) -> BlockContainerProjection<Tree::Scalar>
where
    Tree: Compute<M>,
{
    BlockContainerProjection::from_node(tree.node_input(node))
}

pub(super) fn with_block_scroll_projections<Tree, M, Output>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    consume: impl FnOnce(
        ScrollBoxProjection<'_, Tree::Scalar>,
        ScrollTargetProjection<'_, Tree::Scalar>,
    ) -> Output,
) -> Output
where
    Tree: Compute<M>,
{
    let input = tree.node_input(node);
    consume(
        ScrollBoxProjection::from_node(input),
        ScrollTargetProjection::from_node(input),
    )
}
