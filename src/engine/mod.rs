pub(crate) mod contracts;
mod root;
mod session;
mod validation;

pub(crate) use root::{compute_flex_item_root, compute_hidden, compute_root};
#[cfg(test)]
pub(crate) use session::trace_hidden_compute_session_requests;
pub(crate) use validation::validate_layout_request;

use crate::error::LayoutResultOf;
use crate::{
    CompletedLayoutBatchOf, LayoutRootContextOf, LayoutRootRequestOf, LayoutTree, Traverse,
};

type ValidatedLayoutResult<Tree> = LayoutResultOf<
    <Tree as Traverse>::Node,
    CompletedLayoutBatchOf<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

pub(crate) fn compute_validated_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    invalidated_nodes: Vec<Tree::Node>,
) -> ValidatedLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    let mut session = session::ComputeSession::new(tree, invalidated_nodes);
    match request.context() {
        LayoutRootContextOf::Viewport => {
            compute_root(&mut session, root, request.available())?;
        }
        LayoutRootContextOf::FlexItemUnderViewport(context) => {
            compute_flex_item_root(&mut session, root, request.available(), context)?;
        }
    }

    match request.rounding_mode() {
        crate::LayoutRoundingMode::NearestCssPixel => {
            crate::compute::round_layout(&mut session, root)?
        }
    }

    Ok(session.complete_for_root(root))
}
