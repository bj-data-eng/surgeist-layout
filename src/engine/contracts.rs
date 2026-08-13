use crate::error::LayoutResultOf;
use crate::{
    CacheKeyContext, ComputeInputOf, ComputeOutputOf, InlineFragmentOutputOf, LayoutInputOf,
    LayoutScalar, NodeInputOf, NodeOutputOf, RunMode, Traverse,
};

pub(crate) trait Compute<M = core::convert::Infallible>: Traverse {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar>;
    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar>;
    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);
    fn set_unrounded_inline_fragment_state(
        &mut self,
        _node: Self::Node,
        _fragments: Option<Vec<InlineFragmentOutputOf<Self::Scalar>>>,
    ) {
    }
    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, M>;

    fn float_exclusion_interval(
        &self,
        _node: Self::Node,
        _query: crate::FloatExclusionQueryOf<Self::Scalar>,
    ) -> Option<Result<Option<crate::FloatExclusionIntervalOf<Self::Scalar>>, M>> {
        None
    }
}

pub(crate) trait Round<M = core::convert::Infallible>: Traverse {
    fn unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<Self::Node, NodeOutputOf<Self::Scalar>, Self::Scalar, M>;
    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);

    fn unrounded_inline_fragment_state(
        &self,
        _node: Self::Node,
    ) -> UnroundedInlineFragmentState<'_, Self::Scalar> {
        UnroundedInlineFragmentState::Absent
    }

    fn set_final_inline_fragments(
        &mut self,
        _node: Self::Node,
        _unrounded: Vec<InlineFragmentOutputOf<Self::Scalar>>,
        _final_fragments: Vec<InlineFragmentOutputOf<Self::Scalar>>,
    ) {
    }
}

pub(crate) enum UnroundedInlineFragmentState<'a, S: LayoutScalar> {
    Absent,
    Missing,
    Present(&'a [InlineFragmentOutputOf<S>]),
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
    if input.run_mode() == RunMode::PerformHiddenLayout {
        return compute(tree, node, input);
    }

    let context = tree.cache_context();
    if let Some(output) = tree.cache_get(node, &input, context) {
        return Ok(output);
    }

    let output = compute(tree, node, input)?;
    tree.cache_store(node, &input, context, output);
    Ok(output)
}
