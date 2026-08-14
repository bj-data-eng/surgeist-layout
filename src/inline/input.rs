use crate::block::BlockChildProjection;
use crate::{
    Compute, InlineBoundaryInputOf, InlineTextInputOf, LayoutInputOf, LayoutScalar,
    LineBreakInputOf, Traverse,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InlineParticipantKindOf<'a, S: LayoutScalar> {
    Box(Box<BlockChildProjection<'a, S>>),
    InlineText(InlineTextInputOf<S>),
    LineBreak(LineBreakInputOf<S>),
    InlineBoundary(InlineBoundaryInputOf<S>),
}

/// One settled box, text, break, or boundary participant in an inline context.
///
/// The owned lookup result anchors the borrowed common-box facts exposed by
/// [`Self::kind`] without copying them into another projection.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InlineParticipantProjection<S: LayoutScalar> {
    input: LayoutInputOf<S>,
}

impl<S: LayoutScalar> InlineParticipantProjection<S> {
    #[must_use]
    pub(crate) fn lookup<Tree, M>(tree: &Tree, node: <Tree as Traverse>::Node) -> Self
    where
        Tree: Compute<M, Scalar = S>,
    {
        Self {
            input: tree.layout_input(node),
        }
    }

    #[must_use]
    pub(crate) fn kind(&self) -> InlineParticipantKindOf<'_, S> {
        match &self.input {
            LayoutInputOf::Box(input) => {
                InlineParticipantKindOf::Box(Box::new(BlockChildProjection::from_node(input)))
            }
            LayoutInputOf::InlineText(input) => InlineParticipantKindOf::InlineText(input.clone()),
            LayoutInputOf::LineBreak(input) => InlineParticipantKindOf::LineBreak(*input),
            LayoutInputOf::InlineBoundary(input) => InlineParticipantKindOf::InlineBoundary(*input),
        }
    }
}
