use super::{ComputeInput, ComputeOutput, NodeInput, NodeOutput};

pub trait Traverse {
    type Node: Copy + Eq;
    type Children<'a>: Iterator<Item = Self::Node>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_>;
    fn child_count(&self, node: Self::Node) -> usize;
    fn child(&self, node: Self::Node, index: usize) -> Self::Node;
}

pub trait Compute: Traverse {
    fn node_input(&self, node: Self::Node) -> &NodeInput;
    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput);
    fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput;
}

pub trait Round: Traverse {
    fn unrounded(&self, node: Self::Node) -> NodeOutput;
    fn set_final(&mut self, node: Self::Node, layout: NodeOutput);
}

pub trait CacheAccess {
    type Node: Copy + Eq;

    fn cache_get(&self, node: Self::Node, input: &ComputeInput) -> Option<ComputeOutput>;
    fn cache_store(&mut self, node: Self::Node, input: &ComputeInput, output: ComputeOutput);
    fn cache_clear(&mut self, node: Self::Node);
}

pub fn compute_cached<Tree, ComputeFn>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInput,
    compute: ComputeFn,
) -> ComputeOutput
where
    Tree: CacheAccess + ?Sized,
    ComputeFn: FnOnce(&mut Tree, Tree::Node, ComputeInput) -> ComputeOutput,
{
    if let Some(output) = tree.cache_get(node, &input) {
        return output;
    }

    let output = compute(tree, node, input);
    tree.cache_store(node, &input, output);
    output
}
