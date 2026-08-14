use crate::block::BlockChildProjection;
use crate::{
    Compute, InlineBoundaryInputOf, InlineTextInputOf, LayoutInputOf, LayoutScalar,
    LineBreakInputOf, Traverse,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InlineParticipantKindOf<S: LayoutScalar> {
    Box(Box<BlockChildProjection<S>>),
    InlineText(InlineTextInputOf<S>),
    LineBreak(LineBreakInputOf<S>),
    InlineBoundary(InlineBoundaryInputOf<S>),
}

/// One settled box, text, break, or boundary participant in an inline context.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InlineParticipantProjection<S: LayoutScalar> {
    kind: InlineParticipantKindOf<S>,
}

impl<S: LayoutScalar> InlineParticipantProjection<S> {
    #[must_use]
    fn from_layout_input(input: LayoutInputOf<S>) -> Self {
        let kind = match input {
            LayoutInputOf::Box(input) => {
                InlineParticipantKindOf::Box(Box::new(BlockChildProjection::from_node(&input)))
            }
            LayoutInputOf::InlineText(input) => InlineParticipantKindOf::InlineText(input),
            LayoutInputOf::LineBreak(input) => InlineParticipantKindOf::LineBreak(input),
            LayoutInputOf::InlineBoundary(input) => InlineParticipantKindOf::InlineBoundary(input),
        };
        Self { kind }
    }

    #[must_use]
    pub(crate) fn lookup<Tree, M>(tree: &Tree, node: <Tree as Traverse>::Node) -> Self
    where
        Tree: Compute<M, Scalar = S>,
    {
        Self::from_layout_input(tree.layout_input(node))
    }

    #[must_use]
    pub(crate) fn into_kind(self) -> InlineParticipantKindOf<S> {
        self.kind
    }
}
