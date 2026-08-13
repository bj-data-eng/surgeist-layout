use super::contracts::{CacheAccess, Compute, Round, UnroundedInlineFragmentState};
use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf,
};
use crate::scroll::SettledAutoScrollbarState;
use crate::{
    CompletedLayoutBatchOf, ComputeInputOf, ComputeOutputOf, InlineFragmentOutputEntryOf,
    InlineFragmentOutputOf, LayoutCacheClearEntry, LayoutCacheStoreEntryOf, LayoutInputOf,
    LayoutOutputEntryOf, LayoutScalar, LayoutTree, NodeInputOf, NodeOutputOf, RunMode, Size,
    Traverse,
};

#[cfg(test)]
std::thread_local! {
    static HIDDEN_COMPUTE_SESSION_REQUESTS: std::cell::RefCell<Vec<(
        SettledAutoScrollbarState,
        SettledAutoScrollbarState,
    )>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub(crate) fn trace_hidden_compute_session_requests<T>(
    operation: impl FnOnce() -> T,
) -> (
    T,
    Vec<(SettledAutoScrollbarState, SettledAutoScrollbarState)>,
) {
    HIDDEN_COMPUTE_SESSION_REQUESTS.with(|requests| requests.borrow_mut().clear());
    let result = operation();
    let requests = HIDDEN_COMPUTE_SESSION_REQUESTS.with(|requests| requests.take());
    (result, requests)
}

pub(super) struct ComputeSession<'a, Tree>
where
    Tree: LayoutTree,
{
    tree: &'a Tree,
    unrounded_entries: Vec<LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    final_entries: Vec<LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    unrounded_inline_fragment_groups: Vec<StagedInlineFragmentGroup<Tree::Node, Tree::Scalar>>,
    final_inline_fragment_groups: Vec<StagedInlineFragmentGroup<Tree::Node, Tree::Scalar>>,
    warm_inline_text_nodes: Vec<Tree::Node>,
    cache_store_entries: Vec<LayoutCacheStoreEntryOf<Tree::Node, Tree::Scalar>>,
    cache_clear_entries: Vec<LayoutCacheClearEntry<Tree::Node>>,
    invalidated_nodes: Vec<Tree::Node>,
}

struct StagedInlineFragmentGroup<Node, S: LayoutScalar> {
    node: Node,
    fragments: Option<Vec<InlineFragmentOutputOf<S>>>,
}

impl<'a, Tree> ComputeSession<'a, Tree>
where
    Tree: LayoutTree,
{
    pub(super) fn new(tree: &'a Tree, invalidated_nodes: Vec<Tree::Node>) -> Self {
        let cache_clear_entries = invalidated_nodes
            .iter()
            .copied()
            .map(LayoutCacheClearEntry::new)
            .collect();
        Self {
            tree,
            unrounded_entries: Vec::new(),
            final_entries: Vec::new(),
            unrounded_inline_fragment_groups: Vec::new(),
            final_inline_fragment_groups: Vec::new(),
            warm_inline_text_nodes: Vec::new(),
            cache_store_entries: Vec::new(),
            cache_clear_entries,
            invalidated_nodes,
        }
    }

    fn complete(self) -> CompletedLayoutBatchOf<Tree::Node, Tree::Scalar> {
        let unrounded_inline_fragments = self
            .final_inline_fragment_groups
            .iter()
            .flat_map(|final_group| {
                self.unrounded_inline_fragment_groups
                    .iter()
                    .find(|group| group.node == final_group.node)
                    .and_then(|group| group.fragments.as_deref())
                    .unwrap_or(&[])
                    .iter()
                    .copied()
                    .map(|fragment| InlineFragmentOutputEntryOf::new(final_group.node, fragment))
            })
            .collect();
        let final_inline_fragments = self
            .final_inline_fragment_groups
            .iter()
            .flat_map(|group| {
                group
                    .fragments
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .copied()
                    .map(|fragment| InlineFragmentOutputEntryOf::new(group.node, fragment))
            })
            .collect();
        CompletedLayoutBatchOf::from_entries(
            self.unrounded_entries,
            self.final_entries,
            unrounded_inline_fragments,
            final_inline_fragments,
            self.cache_store_entries,
            self.cache_clear_entries,
            self.invalidated_nodes,
        )
    }

    pub(super) fn complete_for_root(
        mut self,
        root: Tree::Node,
    ) -> CompletedLayoutBatchOf<Tree::Node, Tree::Scalar> {
        let mut visited = Vec::new();
        let mut pending = vec![root];
        let mut ordered_unrounded = Vec::with_capacity(self.unrounded_entries.len());
        let mut ordered_final = Vec::with_capacity(self.final_entries.len());
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            if let Some(entry) = take_layout_entry(&mut self.unrounded_entries, node) {
                ordered_unrounded.push(entry);
            }
            if let Some(entry) = take_layout_entry(&mut self.final_entries, node) {
                ordered_final.push(entry);
            }
            let children = self.tree.children(node).collect::<Vec<_>>();
            pending.extend(children.into_iter().rev());
        }
        assert!(
            self.unrounded_entries.is_empty() && self.final_entries.is_empty(),
            "completed output nodes remain reachable from the validated root"
        );
        self.unrounded_entries = ordered_unrounded;
        self.final_entries = ordered_final;
        self.complete()
    }

    fn staged_source_index(&self, node: Tree::Node) -> crate::SourceIndex {
        self.unrounded_entries
            .iter()
            .rev()
            .find(|entry| entry.node() == node)
            .map(|entry| entry.output().source_index)
            .unwrap_or(crate::SourceIndex::ZERO)
    }

    fn subtree_uses_shape_provider(&self, root: Tree::Node) -> bool {
        let mut visited = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            if self.tree.node_input(node).float_exclusion == crate::FloatExclusion::Shape {
                return true;
            }
            pending.extend(self.tree.children(node));
        }
        false
    }

    fn restore_committed_subtree(&mut self, root: Tree::Node) -> bool {
        let mut visited = Vec::new();
        let mut pending = vec![root];
        let mut restored = Vec::new();
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            let Some(output) = self.tree.unrounded_layout(node) else {
                return false;
            };
            restored.push((node, output));
            let children = self.tree.children(node).collect::<Vec<_>>();
            pending.extend(children.into_iter().rev());
        }
        for (node, output) in restored {
            self.set_unrounded(node, output);
        }
        true
    }

    fn compute_child_uncached(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
    {
        let style = self.node_input(node).clone();
        if self.tree.has_leaf_measurement(node) {
            return crate::measurement::compute_tree_leaf(self.tree, node, input, &style);
        }

        match style.display.inner_display() {
            crate::Display::Block => crate::block::compute_block(self, node, input),
            crate::Display::Flex => crate::flex::compute_flex(self, node, input),
            crate::Display::Grid | crate::Display::GridLanes => {
                crate::grid::compute_grid(self, node, input)
            }
            crate::Display::None => super::compute_hidden(
                self,
                node,
                self.staged_source_index(node),
                input.containing_layout_context(),
                input.containing_auto_scrollbar_pass(),
            ),
            crate::Display::InlineBlock
            | crate::Display::InlineGrid
            | crate::Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }
}

impl<Tree> Traverse for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;
    type Children<'b>
        = Tree::Children<'b>
    where
        Self: 'b;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.tree.children(node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<Tree> Compute<Tree::MeasureError> for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        self.tree.layout_input(node)
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>) {
        if let Some(index) = self
            .unrounded_entries
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.unrounded_entries.remove(index);
        }
        self.unrounded_entries
            .push(LayoutOutputEntryOf::new(node, layout));
    }

    fn set_unrounded_inline_fragment_state(
        &mut self,
        node: Self::Node,
        fragments: Option<Vec<InlineFragmentOutputOf<Self::Scalar>>>,
    ) {
        if fragments.is_some() && self.warm_inline_text_nodes.contains(&node) {
            return;
        }
        set_inline_fragment_group(&mut self.unrounded_inline_fragment_groups, node, fragments);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, Tree::MeasureError>
    {
        if input.run_mode().is_perform_layout()
            && matches!(self.tree.layout_input(node), LayoutInputOf::InlineText(_))
        {
            let context = <Self as CacheAccess<Tree::MeasureError>>::cache_context(self);
            if self.invalidated_nodes.is_empty()
                && let Some(output) = <Self as CacheAccess<Tree::MeasureError>>::cache_get(
                    self, node, &input, context,
                )
            {
                if !self.warm_inline_text_nodes.contains(&node) {
                    self.warm_inline_text_nodes.push(node);
                }
                return Ok(output);
            }

            let known = input.known();
            let size = Size::new(
                known.width.unwrap_or(Self::Scalar::ZERO),
                known.height.unwrap_or(Self::Scalar::ZERO),
            );
            let output = ComputeOutputOf::from_sizes(size, size);
            <Self as CacheAccess<Tree::MeasureError>>::cache_store(
                self, node, &input, context, output,
            );
            return Ok(output);
        }

        let style = self.node_input(node).clone();
        if input.run_mode() == RunMode::PerformHiddenLayout || style.display == crate::Display::None
        {
            #[cfg(test)]
            HIDDEN_COMPUTE_SESSION_REQUESTS.with(|requests| {
                requests.borrow_mut().push((
                    input.settled_auto_scrollbars(),
                    input.containing_auto_scrollbar_pass(),
                ));
            });
            return super::compute_hidden(
                self,
                node,
                self.staged_source_index(node),
                input.containing_layout_context(),
                input.containing_auto_scrollbar_pass(),
            );
        }

        if input.run_mode().is_perform_layout() && self.child_count(node) != 0 {
            let input = input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL);
            if self.subtree_uses_shape_provider(node) {
                let context = <Self as CacheAccess<Tree::MeasureError>>::cache_context(self);
                if let Some(output) = <Self as CacheAccess<Tree::MeasureError>>::cache_get(
                    self, node, &input, context,
                ) && self.restore_committed_subtree(node)
                {
                    return Ok(output);
                }
                let output = self.compute_child_uncached(node, input)?;
                <Self as CacheAccess<Tree::MeasureError>>::cache_store(
                    self, node, &input, context, output,
                );
                return Ok(output);
            }
            return self.compute_child_uncached(node, input);
        }

        super::contracts::compute_cached(self, node, input, |session, node, input| {
            session.compute_child_uncached(
                node,
                input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL),
            )
        })
    }

    fn float_exclusion_interval(
        &self,
        node: Self::Node,
        query: crate::FloatExclusionQueryOf<Self::Scalar>,
    ) -> Option<Result<Option<crate::FloatExclusionIntervalOf<Self::Scalar>>, Tree::MeasureError>>
    {
        self.tree.float_exclusion_interval(node, query)
    }
}

impl<Tree> Round<Tree::MeasureError> for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    fn unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<Self::Node, NodeOutputOf<Self::Scalar>, Self::Scalar, Tree::MeasureError>
    {
        self.unrounded_entries
            .iter()
            .rev()
            .find(|entry| entry.node() == node)
            .map(LayoutOutputEntryOf::output)
            .ok_or_else(|| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::RoundingFinalization,
                    LayoutErrorKindOf::InternalInvariant(
                        LayoutInternalInvariant::MissingStagedUnroundedOutput,
                    ),
                )
            })
    }

    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>) {
        if let Some(index) = self
            .final_entries
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.final_entries.remove(index);
        }
        self.final_entries
            .push(LayoutOutputEntryOf::new(node, layout));
    }

    fn unrounded_inline_fragment_state(
        &self,
        node: Self::Node,
    ) -> UnroundedInlineFragmentState<'_, Self::Scalar> {
        if let Some(group) = self
            .unrounded_inline_fragment_groups
            .iter()
            .find(|group| group.node == node)
        {
            return match &group.fragments {
                Some(fragments) => UnroundedInlineFragmentState::Present(fragments),
                None => UnroundedInlineFragmentState::Absent,
            };
        }

        if matches!(self.tree.layout_input(node), LayoutInputOf::InlineText(_)) {
            return self.tree.unrounded_inline_fragments(node).map_or(
                UnroundedInlineFragmentState::Missing,
                UnroundedInlineFragmentState::Present,
            );
        }

        UnroundedInlineFragmentState::Absent
    }

    fn set_final_inline_fragments(
        &mut self,
        node: Self::Node,
        unrounded: Vec<InlineFragmentOutputOf<Self::Scalar>>,
        final_fragments: Vec<InlineFragmentOutputOf<Self::Scalar>>,
    ) {
        set_inline_fragment_group(
            &mut self.unrounded_inline_fragment_groups,
            node,
            Some(unrounded),
        );
        set_inline_fragment_group(
            &mut self.final_inline_fragment_groups,
            node,
            Some(final_fragments),
        );
    }
}

fn set_inline_fragment_group<Node, S>(
    groups: &mut Vec<StagedInlineFragmentGroup<Node, S>>,
    node: Node,
    fragments: Option<Vec<InlineFragmentOutputOf<S>>>,
) where
    Node: Copy + Eq,
    S: LayoutScalar,
{
    if let Some(group) = groups.iter_mut().find(|group| group.node == node) {
        group.fragments = fragments;
    } else {
        groups.push(StagedInlineFragmentGroup { node, fragments });
    }
}

fn take_layout_entry<Node, S>(
    entries: &mut Vec<LayoutOutputEntryOf<Node, S>>,
    node: Node,
) -> Option<LayoutOutputEntryOf<Node, S>>
where
    Node: Copy + Eq,
    S: LayoutScalar,
{
    entries
        .iter()
        .position(|entry| entry.node() == node)
        .map(|index| entries.swap_remove(index))
}

impl<Tree> CacheAccess<Tree::MeasureError> for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;

    fn cache_context(&self) -> crate::CacheKeyContext {
        self.tree.cache_context()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: crate::CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>> {
        if self.invalidated_nodes.contains(&node) {
            return None;
        }
        self.tree.cache_get(node, input, context)
    }

    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: crate::CacheKeyContext,
        output: ComputeOutputOf<Self::Scalar>,
    ) {
        if let Some(index) = self.cache_store_entries.iter().position(|entry| {
            entry.node() == node && entry.input() == input && entry.context() == context
        }) {
            self.cache_store_entries.remove(index);
        }
        self.cache_store_entries
            .push(LayoutCacheStoreEntryOf::new(node, *input, context, output));
    }

    fn cache_clear(&mut self, node: Self::Node) {
        if self
            .cache_clear_entries
            .iter()
            .any(|entry| entry.node() == node)
        {
            return;
        }
        self.cache_clear_entries
            .push(LayoutCacheClearEntry::new(node));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BidiLevel, DefaultScalar, InlineBreakOpportunityOf, InlineMetricsOf, InlineSegmentId,
        InlineTextInputOf, InlineWhitespaceEdge, LayoutErrorSite, NodeInput, Point, ScrollRectOf,
        ShapedInlineSegmentOf, round_layout,
    };

    struct BoundedDagTree {
        input: NodeInput,
        children: Vec<Vec<u32>>,
        adjacency_queries: std::cell::Cell<usize>,
        remaining_adjacency_budget: std::cell::Cell<usize>,
    }

    impl Traverse for BoundedDagTree {
        type Node = u32;
        type Scalar = DefaultScalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            let remaining = self.remaining_adjacency_budget.get();
            assert!(
                remaining > 0,
                "completion traversal exceeded one adjacency query per unique node"
            );
            self.remaining_adjacency_budget.set(remaining - 1);
            self.adjacency_queries.set(self.adjacency_queries.get() + 1);
            self.children[node as usize].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[node as usize].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[node as usize][index]
        }
    }

    impl LayoutTree for BoundedDagTree {
        type MeasureError = ();

        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.input
        }

        fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<DefaultScalar> {
            LayoutInputOf::box_input(self.input.clone())
        }
    }

    #[test]
    fn fri06_c04_float_lifecycle_completion_shared_repeated_dag_is_source_ordered_and_bounded() {
        let tree = BoundedDagTree {
            input: NodeInput::default(),
            children: vec![
                vec![1, 1, 2, 2],
                vec![2, 2, 3, 3],
                vec![4, 4],
                vec![4, 4, 5, 5],
                vec![5, 5],
                Vec::new(),
            ],
            adjacency_queries: std::cell::Cell::new(0),
            remaining_adjacency_budget: std::cell::Cell::new(6),
        };
        let mut session = ComputeSession::new(&tree, Vec::new());
        for node in [2, 4, 5, 3, 1, 0] {
            Compute::set_unrounded(&mut session, node, NodeOutputOf::new());
            Round::set_final(&mut session, node, NodeOutputOf::new());
        }

        let batch = session.complete_for_root(0);
        let expected_source_order = [0, 1, 2, 4, 5, 3];

        assert_eq!(
            batch
                .unrounded_entries()
                .iter()
                .map(LayoutOutputEntryOf::node)
                .collect::<Vec<_>>(),
            expected_source_order
        );
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .map(LayoutOutputEntryOf::node)
                .collect::<Vec<_>>(),
            expected_source_order
        );
        assert_eq!(tree.adjacency_queries.get(), 6);
        assert_eq!(tree.remaining_adjacency_budget.get(), 0);
    }

    #[test]
    fn compute_session_rejects_missing_staged_unrounded_output() {
        let tree = crate::test_support::layout_tree::PublicLayoutTreeOf::new()
            .style(0, NodeInput::default());
        let session = ComputeSession::new(&tree, Vec::new());

        let error = Round::unrounded(&session, 0).unwrap_err();

        assert_eq!(error.site(), LayoutErrorSite::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::MissingStagedUnroundedOutput,
            )
        );
    }

    fn fri06_c01_fragment<S: LayoutScalar>(segment: u64) -> InlineFragmentOutputOf<S> {
        InlineFragmentOutputOf::new(
            InlineSegmentId::new(segment),
            ScrollRectOf::try_new(
                Point::new(S::from_f64(0.25), S::from_f64(0.5)),
                Size::new(S::from_f64(4.5), S::from_f64(2.25)),
            )
            .unwrap(),
            Point::new(S::from_f64(0.25), S::from_f64(2.0)),
            0,
            segment as usize,
            None,
        )
    }

    fn fri06_c01_text_input<S: LayoutScalar>() -> InlineTextInputOf<S> {
        InlineTextInputOf::try_new(vec![
            ShapedInlineSegmentOf::try_new(
                InlineSegmentId::new(1),
                S::from_f64(4.5),
                InlineMetricsOf::from_ascent_descent(S::from_f64(1.5), S::from_f64(0.75)).unwrap(),
                BidiLevel::try_new(0).unwrap(),
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            )
            .unwrap(),
        ])
        .unwrap()
    }

    struct FragmentTree<S: LayoutScalar> {
        input: NodeInputOf<S>,
        layout_input: LayoutInputOf<S>,
        committed: Option<Vec<InlineFragmentOutputOf<S>>>,
        readback_calls: std::cell::Cell<usize>,
    }

    impl<S: LayoutScalar> Traverse for FragmentTree<S> {
        type Node = u32;
        type Scalar = S;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("fragment test tree has no children")
        }
    }

    impl<S: LayoutScalar> LayoutTree for FragmentTree<S> {
        type MeasureError = ();

        fn node_input(&self, _node: Self::Node) -> &NodeInputOf<S> {
            &self.input
        }

        fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<S> {
            self.layout_input.clone()
        }

        fn unrounded_inline_fragments(
            &self,
            _node: Self::Node,
        ) -> Option<&[InlineFragmentOutputOf<S>]> {
            self.readback_calls.set(self.readback_calls.get() + 1);
            self.committed.as_deref()
        }
    }

    fn assert_fri06_c02_fragment_rounding_and_readback<S: LayoutScalar>() {
        let staged_tree = FragmentTree::<S> {
            input: NodeInputOf::non_box(),
            layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
            committed: None,
            readback_calls: std::cell::Cell::new(0),
        };
        let mut staged = ComputeSession::new(&staged_tree, Vec::new());
        Compute::set_unrounded(&mut staged, 0, NodeOutputOf::new());
        Compute::set_unrounded_inline_fragment_state(
            &mut staged,
            0,
            Some(vec![fri06_c01_fragment(1)]),
        );
        round_layout(&mut staged, 0).unwrap();
        assert_eq!(staged_tree.readback_calls.get(), 0);
        let staged_batch = staged.complete();

        assert_eq!(staged_batch.unrounded_inline_fragments().len(), 1);
        assert_eq!(staged_batch.final_inline_fragments().len(), 1);
        assert_eq!(
            staged_batch.unrounded_inline_fragments()[0]
                .fragment()
                .rect()
                .origin(),
            Point::new(S::from_f64(0.25), S::from_f64(0.5))
        );
        assert_eq!(
            staged_batch.final_inline_fragments()[0]
                .fragment()
                .rect()
                .origin(),
            Point::new(S::ZERO, S::from_f64(1.0))
        );

        let warm_tree = FragmentTree::<S> {
            input: NodeInputOf::non_box(),
            layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
            committed: Some(vec![fri06_c01_fragment(1)]),
            readback_calls: std::cell::Cell::new(0),
        };
        let mut warm = ComputeSession::new(&warm_tree, Vec::new());
        Compute::set_unrounded(&mut warm, 0, NodeOutputOf::new());
        round_layout(&mut warm, 0).unwrap();
        assert_eq!(warm_tree.readback_calls.get(), 1);
        let warm_batch = warm.complete();
        assert_eq!(
            warm_batch.unrounded_inline_fragments(),
            staged_batch.unrounded_inline_fragments()
        );
        assert_eq!(
            warm_batch.final_inline_fragments(),
            staged_batch.final_inline_fragments()
        );

        let empty_tree = FragmentTree::<S> {
            input: NodeInputOf::non_box(),
            layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
            committed: Some(Vec::new()),
            readback_calls: std::cell::Cell::new(0),
        };
        let mut empty = ComputeSession::new(&empty_tree, Vec::new());
        Compute::set_unrounded(&mut empty, 0, NodeOutputOf::new());
        round_layout(&mut empty, 0).unwrap();
        assert_eq!(empty_tree.readback_calls.get(), 1);
        let empty_batch = empty.complete();
        assert!(empty_batch.unrounded_inline_fragments().is_empty());
        assert!(empty_batch.final_inline_fragments().is_empty());
    }

    #[test]
    fn fri06_c02_cache_staged_and_committed_nonempty_and_empty_fragments_round_once_both_scalars() {
        assert_fri06_c02_fragment_rounding_and_readback::<f32>();
        assert_fri06_c02_fragment_rounding_and_readback::<f64>();
    }

    #[test]
    fn fri06_c02_cache_missing_warm_fragment_state_fails_without_publication_both_scalars() {
        fn assert_lane<S: LayoutScalar>() {
            let tree = FragmentTree::<S> {
                input: NodeInputOf::non_box(),
                layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
                committed: None,
                readback_calls: std::cell::Cell::new(0),
            };
            let mut session = ComputeSession::new(&tree, Vec::new());
            Compute::set_unrounded(&mut session, 0, NodeOutputOf::new());

            let error = round_layout(&mut session, 0).unwrap_err();
            assert_eq!(tree.readback_calls.get(), 1);

            assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
            assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
            assert_eq!(
                error.kind(),
                &LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingCachedInlineFragmentState,
                )
            );
            assert!(session.final_entries.is_empty());
            assert!(session.final_inline_fragment_groups.is_empty());
        }

        assert_lane::<f32>();
        assert_lane::<f64>();
    }

    #[test]
    fn fri06_c02_cache_hidden_text_needs_no_committed_fragment_state_both_scalars() {
        fn assert_lane<S: LayoutScalar>() {
            let tree = FragmentTree::<S> {
                input: NodeInputOf::non_box(),
                layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
                committed: None,
                readback_calls: std::cell::Cell::new(0),
            };
            let mut session = ComputeSession::new(&tree, Vec::new());
            Compute::set_unrounded(&mut session, 0, NodeOutputOf::new());
            Compute::set_unrounded_inline_fragment_state(&mut session, 0, None);

            round_layout(&mut session, 0).unwrap();
            assert_eq!(tree.readback_calls.get(), 0);

            let batch = session.complete();
            assert!(batch.unrounded_inline_fragments().is_empty());
            assert!(batch.final_inline_fragments().is_empty());
        }

        assert_lane::<f32>();
        assert_lane::<f64>();
    }
}
