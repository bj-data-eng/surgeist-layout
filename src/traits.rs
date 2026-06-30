use super::{
    CacheKeyContext, CalcResolver, ComputeInputOf, ComputeOutputOf, LayoutInputOf, LayoutScalar,
    NoCalcResolver, NodeInputOf, NodeOutputOf,
};

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

pub trait Compute: Traverse {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar>;
    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar>;
    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);
    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> ComputeOutputOf<Self::Scalar>;

    fn calc_resolver(&self) -> &dyn CalcResolver<Self::Scalar> {
        &NoCalcResolver
    }
}

pub trait Round: Traverse {
    fn unrounded(&self, node: Self::Node) -> NodeOutputOf<Self::Scalar>;
    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);
}

pub trait CacheAccess {
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

pub fn compute_cached<Tree, ComputeFn>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    compute: ComputeFn,
) -> ComputeOutputOf<Tree::Scalar>
where
    Tree: CacheAccess + ?Sized,
    ComputeFn: FnOnce(
        &mut Tree,
        Tree::Node,
        ComputeInputOf<Tree::Scalar>,
    ) -> ComputeOutputOf<Tree::Scalar>,
{
    let context = tree.cache_context();
    if let Some(output) = tree.cache_get(node, &input, context) {
        return output;
    }

    let output = compute(tree, node, input);
    tree.cache_store(node, &input, context, output);
    output
}
