use crate::measurement::LeafMeasureInputOf;
use crate::{
    CacheKeyContext, CompletedLayoutBatchOf, ComputeInputOf, ComputeOutputOf,
    FloatExclusionIntervalOf, FloatExclusionQueryOf, InlineFragmentOutputOf, LayoutInputOf,
    LayoutScalar, NodeInputOf, NodeOutputOf, Size,
};

type FloatExclusionProviderResultOf<S, M> = Result<Option<FloatExclusionIntervalOf<S>>, M>;

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

    /// Returns committed unrounded output for a node in a warm cached subtree.
    ///
    /// `None` prevents reuse of a container cache hit whose descendants could
    /// not otherwise be restored for source-ordered rounding and publication.
    fn unrounded_layout(&self, _node: Self::Node) -> Option<NodeOutputOf<Self::Scalar>> {
        None
    }

    /// Queries one shape-backed float exclusion interval for a physical band.
    ///
    /// Outer `None` means the requested provider is absent. `Ok(None)` means
    /// the shape has no exclusion intersection with this band. Layout invokes
    /// this provider only for a shape float overlapping a finite candidate band.
    fn float_exclusion_interval(
        &self,
        _node: Self::Node,
        _query: FloatExclusionQueryOf<Self::Scalar>,
    ) -> Option<FloatExclusionProviderResultOf<Self::Scalar, Self::MeasureError>> {
        None
    }

    /// Returns the committed unrounded fragments for an inline-text node.
    ///
    /// `Some(&[])` is a committed empty fragment state. `None` is absence of
    /// committed state and becomes an invariant error when a warm inline-text
    /// node reaches rounding.
    fn unrounded_inline_fragments(
        &self,
        _node: Self::Node,
    ) -> Option<&[InlineFragmentOutputOf<Self::Scalar>]> {
        None
    }
}

/// Atomically prepares and commits every state class in a completed layout batch.
///
/// Preparation is immutable and may fail. Commit receives an owned prepared
/// replacement under exclusive access and is infallible.
pub trait LayoutBatchSink<Node, S: LayoutScalar> {
    type Error;
    type Prepared;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<Node, S>,
    ) -> Result<Self::Prepared, Self::Error>;

    fn commit_layout_batch(&mut self, prepared: Self::Prepared);
}
