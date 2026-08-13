pub(crate) mod contracts;
mod root;
mod rounding;
mod session;
mod validation;

use crate::error::LayoutResultOf;
use crate::{
    CompletedLayoutBatchOf, LayoutRootContextOf, LayoutRootRequestOf, LayoutTree, Traverse,
};
pub(crate) use root::{compute_flex_item_root, compute_hidden, compute_root};
#[cfg(test)]
pub(crate) use rounding::round_layout;
#[cfg(test)]
pub(crate) use session::trace_hidden_compute_session_requests;

type CompletedTreeBatch<Tree> =
    CompletedLayoutBatchOf<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;

type ComputeLayoutResult<Tree> = LayoutResultOf<
    <Tree as Traverse>::Node,
    CompletedTreeBatch<Tree>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

pub fn compute_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    compute_layout_invalidated(tree, root, request, &[])
}

pub fn compute_layout_invalidated<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    changed_nodes: &[Tree::Node],
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    let invalidated_nodes = validation::validate_layout_request(tree, root, changed_nodes)?;

    compute_validated_layout(tree, root, request, invalidated_nodes)
}

fn compute_validated_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    invalidated_nodes: Vec<Tree::Node>,
) -> ComputeLayoutResult<Tree>
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
        crate::LayoutRoundingMode::NearestCssPixel => rounding::round_layout(&mut session, root)?,
    }

    Ok(session.complete_for_root(root))
}
