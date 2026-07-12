use super::{
    CacheKeyContext, ComputeInputOf, ComputeOutputOf, LayoutInputOf, LayoutScalar, NodeInputOf,
    NodeOutputOf, Size,
};
use crate::compute::{LayoutResultOf, LeafMeasureInputOf};

pub trait Traverse {
    type Node: Copy + Eq;
    type Scalar: LayoutScalar;
    type Children<'a>: Iterator<Item = Self::Node>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_>;
    fn child_count(&self, node: Self::Node) -> usize;
    fn child(&self, node: Self::Node, index: usize) -> Self::Node;
}

pub trait LayoutTree: Traverse {
    type MeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar>;
    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar>;

    fn has_leaf_measurement(&self, _node: Self::Node) -> bool {
        false
    }

    fn measure_leaf(
        &self,
        _node: Self::Node,
        _input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        None
    }

    fn cache_context(&self) -> CacheKeyContext {
        CacheKeyContext::new()
    }

    fn cache_get(
        &self,
        _node: Self::Node,
        _input: &ComputeInputOf<Self::Scalar>,
        _context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>> {
        None
    }
}

pub(crate) trait Compute<M = core::convert::Infallible>: Traverse {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar>;
    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar>;
    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);
    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, M>;
}

pub(crate) trait Round<M = core::convert::Infallible>: Traverse {
    fn unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<Self::Node, NodeOutputOf<Self::Scalar>, Self::Scalar, M>;
    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);
}

pub(crate) trait CacheAccess<M = core::convert::Infallible> {
    type Node: Copy + Eq;
    type Scalar: LayoutScalar;

    fn cache_context(&self) -> CacheKeyContext;
    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>>;
    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
        output: ComputeOutputOf<Self::Scalar>,
    );
    fn cache_clear(&mut self, node: Self::Node);
}

pub(crate) fn compute_cached<Tree, ComputeFn, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    compute: ComputeFn,
) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: CacheAccess<M> + ?Sized,
    ComputeFn: FnOnce(
        &mut Tree,
        Tree::Node,
        ComputeInputOf<Tree::Scalar>,
    )
        -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>,
{
    let context = tree.cache_context();
    if let Some(output) = tree.cache_get(node, &input, context) {
        return Ok(output);
    }

    let output = compute(tree, node, input)?;
    tree.cache_store(node, &input, context, output);
    Ok(output)
}
