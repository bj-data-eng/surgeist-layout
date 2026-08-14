use crate::node_projection::CommonBoxProjection;
use crate::scroll::{ScrollBoxProjection, ScrollTargetProjection};
use crate::{
    AtomicInlineParticipationOf, Clear, Compute, Display, Float, FloatExclusion, LayoutScalar,
    NodeInputOf, ScrollbarGutter, ScrollbarWidthOf, TextAlign, Traverse, VerticalAlign,
};

/// Block-container facts settled at the block algorithm entry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlockContainerProjection<'a, S: LayoutScalar> {
    pub(super) common: CommonBoxProjection<'a, S>,
    pub(super) display: Display,
    pub(super) text_align: TextAlign,
    pub(super) scrollbar_gutter: ScrollbarGutter,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
}

impl<'a, S: LayoutScalar> BlockContainerProjection<'a, S> {
    #[must_use]
    pub(super) fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            common: CommonBoxProjection::from_node(input),
            display: input.display,
            text_align: input.text_align,
            scrollbar_gutter: input.scrollbar_gutter,
            scrollbar_width: input.scrollbar_width,
        }
    }
}

/// Block-child facts retained after child role lookup.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlockChildProjection<'a, S: LayoutScalar> {
    pub(super) common: CommonBoxProjection<'a, S>,
    pub(super) display: Display,
    pub(super) float: Float,
    pub(super) clear: Clear,
    pub(super) float_exclusion: FloatExclusion,
    pub(super) atomic_inline_participation: Option<AtomicInlineParticipationOf<S>>,
    pub(super) vertical_align: VerticalAlign,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
}

impl<'a, S: LayoutScalar> BlockChildProjection<'a, S> {
    #[must_use]
    pub(crate) fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            common: CommonBoxProjection::from_node(input),
            display: input.display,
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
pub(super) fn block_container_projection<'a, Tree, M>(
    tree: &'a Tree,
    node: <Tree as Traverse>::Node,
) -> BlockContainerProjection<'a, Tree::Scalar>
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
