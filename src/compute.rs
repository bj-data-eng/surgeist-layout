use super::{
    AspectRatioOf, AvailableOf, BoxSizing, CacheAccess, CollapsibleMarginOf, Compute,
    ComputeInputOf, ComputeOutputOf, DefaultScalar, Edges, InlineFragmentOutputEntryOf,
    InlineFragmentOutputOf, LayoutCacheClearEntry, LayoutCacheStoreEntryOf, LayoutInputOf,
    LayoutOutputEntryOf, LayoutRootContextOf, LayoutRootRequestOf, LayoutScalar,
    LengthResolutionOf, LengthResolutionStatus, NodeInputOf, NodeOutputOf, NonNegativeFiniteOf,
    NonNegativeFiniteScalarErrorOf, PhysicalBlockMarginCollapseOf, Point, Position, Round, RunMode,
    Size, SizingMode, Traverse,
};
use crate::engine::{self, contracts::UnroundedInlineFragmentState};
use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant,
    LayoutInvalidInputOf, LayoutOperation, LayoutResultOf, LeafMeasureErrorOf, SizingAlgorithm,
    invalid_measurement_output, sizing_resolution_error, sizing_resolution_error_at_site,
    value_resolution_error,
};
#[cfg(test)]
pub(crate) use crate::error::{LayoutErrorSite, value_resolution_error_at_site};
use crate::geometry::{FlowAxes, PhysicalAxis, PhysicalSide};
use crate::layout_math::{
    MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt, OptionalMinimumSizeFloorExt,
    OptionalSizeExt,
};
use crate::scalar::round_layout_coordinate;
use crate::scroll::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollGeometryErrorOf,
    CanonicalScrollRangeSeedPolicy, CanonicalScrollSourceBuilderOf, ClipMarginSourceOf,
    MeasuredLeafContentBoxInsetSourceOf, MeasuredLeafScrollGeometrySourceOf, OptimalRegionInsetOf,
    OptimalRegionInsetsOf, ScrollOriginAxes, ScrollOriginProgression, SettledAutoScrollbarState,
    canonical_measured_leaf_scroll_geometry, measured_leaf_content_box_inset,
    rebuild_rounded_canonical_scroll_geometry,
};
use crate::sizing::resolve::{
    ResolvedPreferredSize, SizingResolutionError, resolve_maximum_optional,
    resolve_minimum_optional, resolve_preferred_sizing,
};
use crate::{CompletedLayoutBatchOf, LayoutTree};

impl<S: LayoutScalar> OptimalRegionInsetsOf<S> {
    pub(crate) fn from_scroll_padding(scroll_padding: crate::ScrollPaddingOf<S>) -> Self {
        fn inset<S: LayoutScalar>(
            value: crate::ScrollPaddingValueOf<S>,
        ) -> OptimalRegionInsetOf<S> {
            match value {
                crate::ScrollPaddingValueOf::Auto => OptimalRegionInsetOf::Auto,
                crate::ScrollPaddingValueOf::Value(value) => OptimalRegionInsetOf::Value(value),
            }
        }

        Self::new(
            inset(scroll_padding.top()),
            inset(scroll_padding.right()),
            inset(scroll_padding.bottom()),
            inset(scroll_padding.left()),
        )
    }
}

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

type CompletedTreeBatch<Tree> =
    CompletedLayoutBatchOf<<Tree as super::Traverse>::Node, <Tree as super::Traverse>::Scalar>;

#[expect(
    clippy::type_complexity,
    reason = "the public root boundary preserves the tree node, scalar, and provider error types"
)]
pub fn compute_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> LayoutResultOf<
    Tree::Node,
    CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>,
    Tree::Scalar,
    Tree::MeasureError,
>
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
) -> LayoutResultOf<Tree::Node, CompletedTreeBatch<Tree>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    let invalidated_nodes = engine::validate_layout_request(tree, root, changed_nodes)?;

    let mut session = ComputeSession::new(tree, invalidated_nodes);
    match request.context() {
        LayoutRootContextOf::Viewport => {
            compute_root(&mut session, root, request.available())?;
        }
        LayoutRootContextOf::FlexItemUnderViewport(context) => {
            compute_flex_item_root(&mut session, root, request.available(), context)?;
        }
    }

    match request.rounding_mode() {
        super::LayoutRoundingMode::NearestCssPixel => round_layout(&mut session, root)?,
    }

    Ok(session.complete_for_root(root))
}

struct ComputeSession<'a, Tree>
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
    fn new(tree: &'a Tree, invalidated_nodes: Vec<Tree::Node>) -> Self {
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

    fn complete_for_root(
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
            if self.tree.node_input(node).float_exclusion == super::FloatExclusion::Shape {
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
            return self.compute_tree_leaf(node, input, &style);
        }

        match style.display.inner_display() {
            super::Display::Block => crate::block::compute_block(self, node, input),
            super::Display::Flex => crate::flex::compute_flex(self, node, input),
            super::Display::Grid | super::Display::GridLanes => {
                crate::grid::compute_grid(self, node, input)
            }
            super::Display::None => compute_hidden(
                self,
                node,
                self.staged_source_index(node),
                input.containing_layout_context(),
                input.containing_auto_scrollbar_pass(),
            ),
            super::Display::InlineBlock
            | super::Display::InlineGrid
            | super::Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }

    fn compute_tree_leaf(
        &self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        style: &NodeInputOf<Tree::Scalar>,
    ) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
    {
        let resolved = resolve_leaf_values_for_input(input, style)
            .map_err(|error| sizing_resolution_error(node, error))?;

        let site = LayoutErrorSiteOf::Node(node);
        compute_leaf_with_resolved_values(site, input, style, resolved, |measure_input| match self
            .tree
            .measure_leaf(node, measure_input)
        {
            Some(Ok(output)) => Ok(output),
            Some(Err(error)) => Err(LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::Measurement(error),
            )),
            None => Err(LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingLeafMeasurementProvider,
                ),
            )),
        })
    }
}

fn root_scroll_error<Node, S, M, E>(node: Node, error: E) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let _ = error;
    let kind =
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry);

    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        kind,
    )
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
        if input.run_mode() == RunMode::PerformHiddenLayout || style.display == super::Display::None
        {
            #[cfg(test)]
            HIDDEN_COMPUTE_SESSION_REQUESTS.with(|requests| {
                requests.borrow_mut().push((
                    input.settled_auto_scrollbars(),
                    input.containing_auto_scrollbar_pass(),
                ));
            });
            return compute_hidden(
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

        crate::engine::contracts::compute_cached(self, node, input, |session, node, input| {
            session.compute_child_uncached(
                node,
                input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL),
            )
        })
    }

    fn float_exclusion_interval(
        &self,
        node: Self::Node,
        query: super::FloatExclusionQueryOf<Self::Scalar>,
    ) -> Option<Result<Option<super::FloatExclusionIntervalOf<Self::Scalar>>, Tree::MeasureError>>
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

    fn cache_context(&self) -> super::CacheKeyContext {
        self.tree.cache_context()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: super::CacheKeyContext,
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
        context: super::CacheKeyContext,
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

pub(crate) fn compute_hidden<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    source_index: crate::SourceIndex,
    containing_layout_context: crate::ContainingLayoutContext,
    containing_auto_scrollbar_pass: SettledAutoScrollbarState,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    ComputeOutputOf<<Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>
where
    Tree: Compute<M>
        + CacheAccess<M, Node = <Tree as Traverse>::Node, Scalar = <Tree as Traverse>::Scalar>,
{
    tree.cache_clear(node);
    tree.set_unrounded_inline_fragment_state(node, None);
    tree.set_unrounded(node, NodeOutputOf::with_source_index(source_index));

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        match tree.layout_input(child) {
            LayoutInputOf::Box(_) => {
                tree.set_unrounded(
                    child,
                    NodeOutputOf::with_source_index(crate::SourceIndex::new(index)),
                );
                let descendant_context = crate::ContainingLayoutContext::new(
                    containing_layout_context.flow_axes(),
                    crate::ParentFormattingContext::NoParent,
                );
                tree.compute_child(
                    child,
                    ComputeInputOf::hidden_in_containing_pass(
                        descendant_context,
                        containing_auto_scrollbar_pass,
                    ),
                )?;
            }
            LayoutInputOf::InlineText(_)
            | LayoutInputOf::LineBreak(_)
            | LayoutInputOf::InlineBoundary(_) => {
                tree.cache_clear(child);
                tree.set_unrounded_inline_fragment_state(child, None);
                tree.set_unrounded(
                    child,
                    NodeOutputOf::with_source_index(crate::SourceIndex::new(index)),
                );
            }
        }
    }

    Ok(ComputeOutputOf::HIDDEN)
}

pub(crate) fn compute_root<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    available: Size<AvailableOf<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(root).clone();
    let containing_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let containing_layout_context = super::ContainingLayoutContext::new(
        containing_flow_axes,
        super::ParentFormattingContext::NoParent,
    );
    let parent = available.map(AvailableOf::into_option);
    let known =
        root_known_inline::<Tree, M>(tree, root, &style, containing_flow_axes, available, parent)?;
    let output = tree.compute_child(
        root,
        ComputeInputOf::root_layout(known, parent, containing_layout_context, available),
    )?;
    let root_edges = resolve_root_edges(tree, root, &style, containing_flow_axes, parent)?;
    let scroll_geometry = root_scroll_geometry(root, &style, output, &root_edges)?;
    let location = root_start_location(containing_flow_axes, output.size, available);

    tree.set_unrounded(
        root,
        NodeOutputOf {
            source_index: crate::SourceIndex::ZERO,
            location,
            size: output.size,
            content_size: output.content_size,
            padding: root_edges.padding,
            border: root_edges.border,
            margin: root_edges.margin,
            ..NodeOutputOf::new()
        }
        .with_scroll_geometry(scroll_geometry),
    );
    Ok(())
}

fn compute_flex_item_root<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    available: Size<AvailableOf<Tree::Scalar>>,
    context: super::FlexItemRootContextOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(root).clone();
    let containing_flow_axes = context.parent_flow_axes();
    let containing_layout_context = super::ContainingLayoutContext::new(
        containing_flow_axes,
        super::ParentFormattingContext::Flex,
    );
    let parent = context.viewport_available().map(AvailableOf::into_option);
    let known =
        root_known_inline::<Tree, M>(tree, root, &style, containing_flow_axes, available, parent)?;
    let output = tree.compute_child(
        root,
        ComputeInputOf::flex_item_root(known, parent, containing_layout_context, available),
    )?;
    let root_edges = resolve_root_edges(tree, root, &style, containing_flow_axes, parent)?;
    let scroll_geometry = root_scroll_geometry(root, &style, output, &root_edges)?;
    tree.set_unrounded(
        root,
        NodeOutputOf {
            source_index: crate::SourceIndex::ZERO,
            location: Point::ZERO,
            size: output.size,
            content_size: output.content_size,
            padding: root_edges.padding,
            border: root_edges.border,
            margin: root_edges.margin,
            ..NodeOutputOf::new()
        }
        .with_scroll_geometry(scroll_geometry),
    );
    Ok(())
}

struct RootEdges<S: LayoutScalar> {
    padding: Edges<S>,
    border: Edges<S>,
    margin: Edges<S>,
}

fn root_scroll_geometry<Node, S, M>(
    node: Node,
    style: &NodeInputOf<S>,
    output: ComputeOutputOf<S>,
    edges: &RootEdges<S>,
) -> LayoutResultOf<Node, Option<crate::ScrollGeometryOf<S>>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    if style.display == super::Display::None {
        return Ok(None);
    }

    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let settled_auto_scrollbars = SettledAutoScrollbarState::INITIAL;
    let source = match output.scroll_geometry {
        Some(ref geometry) => CanonicalRetainedScrollSourceOf::Existing(geometry),
        None => CanonicalRetainedScrollSourceOf::Reconstruct {
            content_size: output.content_size,
        },
    };
    CanonicalScrollSourceBuilderOf::for_node(
        style,
        flow_axes,
        output.size,
        edges.border,
        edges.padding,
        settled_auto_scrollbars,
        ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
    )
    .geometry_from_retained_source(
        source,
        CanonicalScrollRangeSeedPolicy::IncludeReservedGutter,
    )
    .map(Some)
    .map_err(|error| root_scroll_error(node, error))
}

type RootKnownInlineResult<Node, S, M> = LayoutResultOf<Node, Size<Option<S>>, S, M>;

fn resolve_root_edges<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing_flow_axes: FlowAxes,
    parent: Size<Option<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, RootEdges<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let margin = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.margin, parent, |length, basis| {
            resolve_auto_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;

    Ok(RootEdges {
        padding,
        border,
        margin,
    })
}

fn root_known_inline<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing_flow_axes: FlowAxes,
    fill_available: Size<AvailableOf<Tree::Scalar>>,
    percentage_parent: Size<Option<Tree::Scalar>>,
) -> RootKnownInlineResult<<Tree as Traverse>::Node, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let inline_axis = containing_flow_axes.inline_axis();
    if style.display.is_inline_level()
        || style.item_is_replaced
        || !root_physical_axis_value(style.size.clone(), inline_axis).is_auto()
        || !root_physical_axis_value(style.min_size.clone(), inline_axis).is_auto()
    {
        return Ok(Size::NONE);
    }

    let Some(available_inline) =
        root_physical_axis_value(fill_available, inline_axis).into_option()
    else {
        return Ok(Size::NONE);
    };
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, percentage_parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, percentage_parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let padding_border_size = (padding + border).sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };
    let max_size = Size::new(
        resolve_maximum_optional(
            &style.max_size.width,
            SizingAlgorithm::Block,
            PhysicalAxis::Horizontal,
            percentage_parent.width,
            false,
        ),
        resolve_maximum_optional(
            &style.max_size.height,
            SizingAlgorithm::Block,
            PhysicalAxis::Vertical,
            percentage_parent.height,
            false,
        ),
    );
    if matches!(max_size.width, Err(SizingResolutionError::Unsupported(_)))
        || matches!(max_size.height, Err(SizingResolutionError::Unsupported(_)))
    {
        // The root optimization cannot know whether a root with block display is
        // measured as a leaf. Defer contextual rejection to the actual consumer.
        return Ok(Size::NONE);
    }
    let max_size = Size::new(
        max_size
            .width
            .map_err(|error| sizing_resolution_error(node, error))?,
        max_size
            .height
            .map_err(|error| sizing_resolution_error(node, error))?,
    )
    .add_optional(box_sizing_adjustment);

    Ok(root_known_on_axis(
        inline_axis,
        available_inline
            .clamp_max_before_min_optional(None, root_physical_axis_value(max_size, inline_axis)),
    ))
}

fn root_physical_axis_value<T>(size: Size<T>, axis: PhysicalAxis) -> T {
    match axis {
        PhysicalAxis::Horizontal => size.width,
        PhysicalAxis::Vertical => size.height,
    }
}

fn root_known_on_axis<S: LayoutScalar>(axis: PhysicalAxis, value: S) -> Size<Option<S>> {
    match axis {
        PhysicalAxis::Horizontal => Size::new(Some(value), None),
        PhysicalAxis::Vertical => Size::new(None, Some(value)),
    }
}

fn root_start_location<S: LayoutScalar>(
    containing_flow_axes: FlowAxes,
    root_size: Size<S>,
    available: Size<AvailableOf<S>>,
) -> Point<S> {
    Point::new(
        root_start_coordinate(
            containing_flow_axes.inline_start(),
            containing_flow_axes.block_start(),
            root_size,
            available,
            PhysicalAxis::Horizontal,
        ),
        root_start_coordinate(
            containing_flow_axes.inline_start(),
            containing_flow_axes.block_start(),
            root_size,
            available,
            PhysicalAxis::Vertical,
        ),
    )
}

fn root_start_coordinate<S: LayoutScalar>(
    inline_start: PhysicalSide,
    block_start: PhysicalSide,
    root_size: Size<S>,
    available: Size<AvailableOf<S>>,
    axis: PhysicalAxis,
) -> S {
    let start_side = if inline_start.axis() == axis {
        inline_start
    } else {
        block_start
    };
    match start_side {
        PhysicalSide::Top | PhysicalSide::Left => S::ZERO,
        PhysicalSide::Right | PhysicalSide::Bottom => root_physical_axis_value(available, axis)
            .into_option()
            .map_or(S::ZERO, |extent| {
                extent - root_physical_axis_value(root_size, axis)
            }),
    }
}

pub(crate) fn round_layout<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Round<M>,
{
    round_layout_inner(tree, root, Tree::Scalar::ZERO, Tree::Scalar::ZERO)
}

fn round_layout_inner<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    cumulative_x: Tree::Scalar,
    cumulative_y: Tree::Scalar,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Round<M>,
{
    let unrounded = tree.unrounded(node)?;
    let mut layout = unrounded;
    let parent_cumulative_x = cumulative_x;
    let parent_cumulative_y = cumulative_y;
    let cumulative_x = cumulative_x + unrounded.location.x;
    let cumulative_y = cumulative_y + unrounded.location.y;

    layout.location.x = round_layout_coordinate(unrounded.location.x);
    layout.location.y = round_layout_coordinate(unrounded.location.y);
    layout.size.width = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x);
    layout.size.height = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y);
    layout.content_size.width =
        round_layout_coordinate(cumulative_x + unrounded.content_size.width)
            - round_layout_coordinate(cumulative_x);
    layout.content_size.height =
        round_layout_coordinate(cumulative_y + unrounded.content_size.height)
            - round_layout_coordinate(cumulative_y);
    layout.border.left = round_layout_coordinate(cumulative_x + unrounded.border.left)
        - round_layout_coordinate(cumulative_x);
    layout.border.right = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x + unrounded.size.width - unrounded.border.right);
    layout.border.top = round_layout_coordinate(cumulative_y + unrounded.border.top)
        - round_layout_coordinate(cumulative_y);
    layout.border.bottom = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y + unrounded.size.height - unrounded.border.bottom);
    layout.padding.left = round_layout_coordinate(cumulative_x + unrounded.padding.left)
        - round_layout_coordinate(cumulative_x);
    layout.padding.right = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x + unrounded.size.width - unrounded.padding.right);
    layout.padding.top = round_layout_coordinate(cumulative_y + unrounded.padding.top)
        - round_layout_coordinate(cumulative_y);
    layout.padding.bottom = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y + unrounded.size.height - unrounded.padding.bottom);
    let scroll_geometry = unrounded
        .scroll_geometry
        .map(|geometry| {
            rebuild_rounded_canonical_scroll_geometry(
                geometry,
                Point::new(cumulative_x, cumulative_y),
            )
        })
        .transpose()
        .map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::InvalidRoundedScrollGeometry,
                ),
            )
        })?;
    layout = layout.with_scroll_geometry(scroll_geometry);

    let fragment_phases = match tree.unrounded_inline_fragment_state(node) {
        UnroundedInlineFragmentState::Absent => None,
        UnroundedInlineFragmentState::Missing => {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingCachedInlineFragmentState,
                ),
            ));
        }
        UnroundedInlineFragmentState::Present(fragments) => {
            let unrounded_fragments = fragments.to_vec();
            let final_fragments = fragments
                .iter()
                .copied()
                .map(|fragment| {
                    round_inline_fragment(
                        node,
                        fragment,
                        Point::new(parent_cumulative_x, parent_cumulative_y),
                    )
                })
                .collect::<LayoutResultOf<_, Vec<_>, _, _>>()?;
            Some((unrounded_fragments, final_fragments))
        }
    };

    tree.set_final(node, layout);
    if let Some((unrounded_fragments, final_fragments)) = fragment_phases {
        tree.set_final_inline_fragments(node, unrounded_fragments, final_fragments);
    }

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        round_layout_inner(tree, child, cumulative_x, cumulative_y)?;
    }
    Ok(())
}

fn round_inline_fragment<Node, S, M>(
    node: Node,
    fragment: InlineFragmentOutputOf<S>,
    cumulative_origin: Point<S>,
) -> LayoutResultOf<Node, InlineFragmentOutputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let rect = fragment.rect();
    let origin = rect.origin();
    let size = rect.size();
    let rounded_origin = Point::new(
        round_layout_coordinate(cumulative_origin.x + origin.x)
            - round_layout_coordinate(cumulative_origin.x),
        round_layout_coordinate(cumulative_origin.y + origin.y)
            - round_layout_coordinate(cumulative_origin.y),
    );
    let rounded_end = Point::new(
        round_layout_coordinate(cumulative_origin.x + origin.x + size.width)
            - round_layout_coordinate(cumulative_origin.x),
        round_layout_coordinate(cumulative_origin.y + origin.y + size.height)
            - round_layout_coordinate(cumulative_origin.y),
    );
    let rounded_rect = super::ScrollRectOf::try_new(
        rounded_origin,
        Size::new(
            (rounded_end.x - rounded_origin.x).max(S::ZERO),
            (rounded_end.y - rounded_origin.y).max(S::ZERO),
        ),
    )
    .map_err(|_| invalid_rounded_inline_fragment_error(node))?;
    let baseline = fragment.baseline();
    let rounded_baseline = Point::new(
        round_layout_coordinate(cumulative_origin.x + baseline.x)
            - round_layout_coordinate(cumulative_origin.x),
        round_layout_coordinate(cumulative_origin.y + baseline.y)
            - round_layout_coordinate(cumulative_origin.y),
    );
    if !rounded_baseline.x.is_finite() || !rounded_baseline.y.is_finite() {
        return Err(invalid_rounded_inline_fragment_error(node));
    }
    Ok(InlineFragmentOutputOf::new(
        fragment.segment_id(),
        rounded_rect,
        rounded_baseline,
        fragment.line_index(),
        fragment.visual_index(),
        fragment.replacement_inline_extent(),
    ))
}

fn invalid_rounded_inline_fragment_error<Node, S, M>(node: Node) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RoundingFinalization,
        LayoutErrorKindOf::InternalInvariant(
            LayoutInternalInvariant::InvalidRoundedInlineFragmentGeometry,
        ),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafMeasureInputOf<S: LayoutScalar = DefaultScalar> {
    known_content_size: Size<Option<S>>,
    available_content_size: Size<MeasurementAvailableOf<S>>,
}

pub type LeafMeasureInput = LeafMeasureInputOf<DefaultScalar>;

impl<S: LayoutScalar> LeafMeasureInputOf<S> {
    #[must_use]
    pub const fn known_content_size(&self) -> Size<Option<S>> {
        self.known_content_size
    }

    #[must_use]
    pub const fn available_content_size(&self) -> Size<MeasurementAvailableOf<S>> {
        self.available_content_size
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeasurementAvailableOf<S: LayoutScalar = DefaultScalar> {
    Definite(NonNegativeFiniteOf<S>),
    MinContent,
    MaxContent,
}

pub type MeasurementAvailable = MeasurementAvailableOf<DefaultScalar>;

impl<S: LayoutScalar> MeasurementAvailableOf<S> {
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    pub fn definite(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self::Definite(NonNegativeFiniteOf::new(value)?))
    }

    #[must_use]
    pub const fn definite_value(self) -> Option<NonNegativeFiniteOf<S>> {
        match self {
            Self::Definite(value) => Some(value),
            Self::MinContent | Self::MaxContent => None,
        }
    }

    #[must_use]
    pub const fn into_available(self) -> AvailableOf<S> {
        match self {
            Self::Definite(value) => AvailableOf::Definite(value.get()),
            Self::MinContent => AvailableOf::MinContent,
            Self::MaxContent => AvailableOf::MaxContent,
        }
    }

    fn from_content_space(value: AvailableOf<S>) -> Result<Self, S> {
        match value {
            AvailableOf::Definite(value) => Ok(Self::Definite(
                NonNegativeFiniteOf::new(finite_floor_at_zero(value)?)
                    .expect("finite content-space availability is non-negative"),
            )),
            AvailableOf::MinContent => Ok(Self::MinContent),
            AvailableOf::MaxContent => Ok(Self::MaxContent),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafResolvedValues<S: LayoutScalar> {
    margin: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
    node_size: Size<Option<S>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    preferred_intrinsic_availability: Size<Option<AvailableOf<S>>>,
    flex_basis_intrinsic_availability: Size<Option<AvailableOf<S>>>,
    aspect_ratio: Option<AspectRatioOf<S>>,
}

fn resolve_leaf_values<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolve_auto: impl Fn(super::LengthAutoOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
    resolve_length: impl Fn(super::LengthOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
) -> Result<LeafResolvedValues<S>, SizingResolutionError<S>>
where
    S: LayoutScalar,
{
    let margin = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.margin, input.parent(), resolve_auto),
    )?;
    let padding = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.padding, input.parent(), &resolve_length),
    )?;
    let border = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.border, input.parent(), resolve_length),
    )?;
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };

    let (node_size, node_min_size, node_max_size, preferred_intrinsic_availability, aspect_ratio) =
        match input.sizing_mode() {
            SizingMode::ContentSize => (input.known(), Size::NONE, Size::NONE, Size::NONE, None),
            SizingMode::InherentSize => {
                let missing_basis_is_indefinite = input.run_mode() == RunMode::ComputeSize;
                let preferred = Size::new(
                    resolve_preferred_sizing(
                        &style.size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_preferred_sizing(
                        &style.size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                );
                let style_size = preferred
                    .map(|resolved| match resolved {
                        ResolvedPreferredSize::Definite(value) => Some(value),
                        ResolvedPreferredSize::Auto
                        | ResolvedPreferredSize::MinContent
                        | ResolvedPreferredSize::MaxContent => None,
                    })
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let preferred_intrinsic_availability = preferred.map(|resolved| match resolved {
                    ResolvedPreferredSize::MinContent => Some(AvailableOf::MIN_CONTENT),
                    ResolvedPreferredSize::MaxContent => Some(AvailableOf::MAX_CONTENT),
                    ResolvedPreferredSize::Auto | ResolvedPreferredSize::Definite(_) => None,
                });
                let style_min_size = Size::new(
                    resolve_minimum_optional(
                        &style.min_size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_minimum_optional(
                        &style.min_size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                )
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let style_max_size = Size::new(
                    resolve_maximum_optional(
                        &style.max_size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_maximum_optional(
                        &style.max_size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                )
                .add_optional(box_sizing_adjustment);

                (
                    input.known().or(style_size),
                    style_min_size,
                    style_max_size,
                    preferred_intrinsic_availability,
                    style.aspect_ratio,
                )
            }
        };
    let flex_basis_intrinsic =
        if input.parent_formatting_context() == super::ParentFormattingContext::Flex {
            if style.flex_basis.is_min_content() {
                Some(AvailableOf::MIN_CONTENT)
            } else if style.flex_basis.is_max_content() {
                Some(AvailableOf::MAX_CONTENT)
            } else {
                None
            }
        } else {
            None
        };
    let flex_basis_intrinsic_availability = input
        .available()
        .map(|available| flex_basis_intrinsic.filter(|intrinsic| *intrinsic == available));

    Ok(LeafResolvedValues {
        margin,
        padding,
        border,
        node_size,
        node_min_size,
        node_max_size,
        preferred_intrinsic_availability,
        flex_basis_intrinsic_availability,
        aspect_ratio,
    })
}

fn resolve_leaf_values_for_input<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
) -> Result<LeafResolvedValues<S>, SizingResolutionError<S>>
where
    S: LayoutScalar,
{
    resolve_leaf_values(
        input,
        style,
        |length, basis| resolve_leaf_auto(input, length, basis),
        |length, basis| resolve_leaf_length(input, length, basis),
    )
}

fn transpose_leaf_edges<S, E>(edges: Edges<Result<S, E>>) -> Result<Edges<S>, E> {
    Ok(Edges::new(
        edges.top?,
        edges.right?,
        edges.bottom?,
        edges.left?,
    ))
}

pub fn compute_leaf<S, M>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    mut measure: impl FnMut(LeafMeasureInputOf<S>) -> Result<Size<S>, M>,
) -> LayoutResultOf<(), ComputeOutputOf<S>, S, M>
where
    S: LayoutScalar,
{
    let site = LayoutErrorSiteOf::Standalone;
    let resolved = resolve_leaf_values_for_input(input, style)
        .map_err(|error| sizing_resolution_error_at_site(site, error))?;

    compute_leaf_with_resolved_values(site, input, style, resolved, |measure_input| {
        measure(measure_input).map_err(|error| {
            LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::Measurement(error),
            )
        })
    })
}

fn compute_leaf_with_resolved_values<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolved: LeafResolvedValues<S>,
    mut measure: impl FnMut(LeafMeasureInputOf<S>) -> LayoutResultOf<Node, Size<S>, S, M>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let LeafResolvedValues {
        padding,
        border,
        node_size,
        node_min_size,
        node_max_size,
        ..
    } = resolved;
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let leaf_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let block_start = leaf_flow_axes.block_start();
    let block_end = leaf_flow_axes.block_end();
    let node_block_size = match leaf_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => node_size.width,
        PhysicalAxis::Vertical => node_size.height,
    };
    let node_min_block_size = match leaf_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => node_min_size.width,
        PhysicalAxis::Vertical => node_min_size.height,
    };

    let prevents_margin_collapse = input.parent_formatting_context()
        != super::ParentFormattingContext::BlockFlow
        || style.display != super::Display::Block
        || !style.item_is_replaced && style.overflow.establishes_independent_formatting_context()
        || style.position == Position::Absolute
        || padding.at_physical_side(block_start) > S::ZERO
        || padding.at_physical_side(block_end) > S::ZERO
        || border.at_physical_side(block_start) > S::ZERO
        || border.at_physical_side(block_end) > S::ZERO
        || matches!(node_block_size, Some(size) if size > S::ZERO)
        || matches!(node_min_block_size, Some(size) if size > S::ZERO);

    if input.run_mode() == RunMode::ComputeSize
        && prevents_margin_collapse
        && let Size {
            width: Some(width),
            height: Some(height),
        } = node_size
    {
        let size = Size::new(width, height)
            .clamp_max_before_min_optional(node_min_size, node_max_size)
            .max_optional(padding_border_size.map(Some));
        return Ok(ComputeOutputOf::from_outer_size(size));
    }

    let mut pass_input = input;
    let mut reusable_measurement = None;
    loop {
        let pass = leaf_pass_input(site, pass_input, style, &resolved)?;
        let measured = match reusable_measurement.take() {
            Some((measurement_input, measured)) if measurement_input == pass.measurement_input => {
                measured
            }
            _ => validate_measurement_output(measure(pass.measurement_input)?)
                .map_err(|error| leaf_measurement_error_at_site(site, error))?,
        };
        let unclamped = pass_input
            .known()
            .or(resolved.node_size)
            .unwrap_or(measured + pass.content_box_inset_size);
        let height_is_definite =
            pass_input.known().height.is_some() || resolved.node_size.height.is_some();
        let aspect_height = if height_is_definite {
            unclamped.height
        } else {
            unclamped.height.max(
                resolved
                    .aspect_ratio
                    .map(|ratio| unclamped.width / ratio.get())
                    .unwrap_or(S::ZERO),
            )
        };
        let aspect_size = Size::new(unclamped.width, aspect_height)
            .clamp_max_before_min_optional(resolved.node_min_size, resolved.node_max_size)
            .max_optional(padding_border_size.map(Some));

        let mut output =
            ComputeOutputOf::from_sizes(aspect_size, measured + resolved.padding.sum_axes());
        let can_collapse_through = !prevents_margin_collapse
            && leaf_flow_axes.logical_size(aspect_size).block == S::ZERO
            && leaf_flow_axes.logical_size(measured).block == S::ZERO;
        output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
            leaf_flow_axes,
            CollapsibleMarginOf::ZERO,
            CollapsibleMarginOf::ZERO,
            can_collapse_through,
        );
        let geometry =
            canonical_measured_leaf_scroll_geometry(MeasuredLeafScrollGeometrySourceOf {
                flow_axes: leaf_flow_axes,
                computed_overflow: style.overflow,
                item_is_replaced: style.item_is_replaced,
                border_box_size: aspect_size,
                border: resolved.border,
                padding: resolved.padding,
                scrollbar_gutter: style.scrollbar_gutter,
                scrollbar_width: style.scrollbar_width,
                settled_auto_scrollbars: pass_input.settled_auto_scrollbars(),
                clip_margin: ClipMarginSourceOf::new(
                    style.overflow_clip_margin.clip_box(),
                    style.overflow_clip_margin.margin(),
                ),
                scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
                measured_content_size: measured,
                scroll_snap_type: style.scroll_snap_type,
                target_scroll_margin: style.scroll_margin,
                target_snap_align: style.scroll_snap_align,
                target_snap_stop: style.scroll_snap_stop,
            })
            .map_err(|error| leaf_scroll_error_at_site(site, pass_input.run_mode(), error))?;
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || style.scrollbar_width.get() == S::ZERO
        {
            if input.run_mode().is_perform_layout() {
                output.content_size = geometry.canonical_content_size().map_err(|error| {
                    leaf_scroll_error_at_site(site, pass_input.run_mode(), error)
                })?;
                output.scroll_geometry = Some(geometry);
            }
            return Ok(output);
        }

        reusable_measurement = Some((pass.measurement_input, measured));
        pass_input = pass_input.with_settled_auto_scrollbars(next_state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafPassInputOf<S: LayoutScalar> {
    content_box_inset_size: Size<S>,
    measurement_input: LeafMeasureInputOf<S>,
}

fn leaf_pass_input<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolved: &LeafResolvedValues<S>,
) -> LayoutResultOf<Node, LeafPassInputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let content_box_inset = measured_leaf_content_box_inset(MeasuredLeafContentBoxInsetSourceOf {
        flow_axes: FlowAxes::new(style.writing_mode, style.direction),
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: input.settled_auto_scrollbars(),
        padding: resolved.padding,
        border: resolved.border,
    });
    let content_box_inset_size = content_box_inset.sum_axes();
    let available = Size::new(
        input
            .known()
            .width
            .map(AvailableOf::definite)
            .unwrap_or(input.available().width)
            .sub_margin(resolved.margin.horizontal_sum())
            .set_optional(input.known().width)
            .set_optional(resolved.node_size.width)
            .map_definite(|value| {
                value.clamp_max_before_min_optional(
                    resolved.node_min_size.width,
                    resolved.node_max_size.width,
                ) - content_box_inset.horizontal_sum()
            }),
        input
            .known()
            .height
            .map(AvailableOf::definite)
            .unwrap_or(input.available().height)
            .sub_margin(resolved.margin.vertical_sum())
            .set_optional(input.known().height)
            .set_optional(resolved.node_size.height)
            .map_definite(|value| {
                value.clamp_max_before_min_optional(
                    resolved.node_min_size.height,
                    resolved.node_max_size.height,
                ) - content_box_inset.vertical_sum()
            }),
    )
    .zip_map(
        resolved.preferred_intrinsic_availability,
        |available, intrinsic| intrinsic.unwrap_or(available),
    )
    .zip_map(
        resolved.flex_basis_intrinsic_availability,
        |available, intrinsic| intrinsic.unwrap_or(available),
    );

    let known_content_size = match input.run_mode() {
        RunMode::ComputeSize => input.known(),
        RunMode::PerformRootLayout | RunMode::PerformLayout => Size::NONE,
        RunMode::PerformHiddenLayout => {
            unreachable!("hidden layout uses ComputeOutput::HIDDEN")
        }
    }
    .zip_map(content_box_inset_size, |value, inset| {
        value.map(|value| finite_floor_at_zero(value - inset))
    });
    let known_content_size =
        transpose_leaf_known_content_size(known_content_size).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })?;
    let available_content_size =
        measurement_available_content_size(available).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })?;
    let measurement_input = LeafMeasureInputOf {
        known_content_size,
        available_content_size,
    };
    Ok(LeafPassInputOf {
        content_box_inset_size,
        measurement_input,
    })
}

fn leaf_scroll_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    run_mode: RunMode,
    _error: CanonicalScrollGeometryErrorOf<S>,
) -> LayoutErrorOf<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let (operation, invariant) = match (site, run_mode) {
        (LayoutErrorSiteOf::Standalone, _) => (
            LayoutOperation::LeafMeasurement,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        (_, RunMode::PerformRootLayout) => (
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        _ => (
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        ),
    };
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::InternalInvariant(invariant),
    )
}

fn validate_measurement_output<S, M>(measured: Size<S>) -> Result<Size<S>, LeafMeasureErrorOf<S, M>>
where
    S: LayoutScalar,
{
    let width = NonNegativeFiniteOf::new(measured.width)
        .map_err(|error| invalid_measurement_output(PhysicalAxis::Horizontal, error))?;
    let height = NonNegativeFiniteOf::new(measured.height)
        .map_err(|error| invalid_measurement_output(PhysicalAxis::Vertical, error))?;

    Ok(Size::new(width.get(), height.get()))
}

fn finite_floor_at_zero<S>(value: S) -> Result<S, S>
where
    S: LayoutScalar,
{
    if value.is_finite() {
        Ok(value.max(S::ZERO))
    } else {
        Err(value)
    }
}

fn transpose_leaf_known_content_size<S>(
    size: Size<Option<Result<S, S>>>,
) -> Result<Size<Option<S>>, S>
where
    S: LayoutScalar,
{
    Ok(Size::new(size.width.transpose()?, size.height.transpose()?))
}

fn measurement_available_content_size<S>(
    available: Size<AvailableOf<S>>,
) -> Result<Size<MeasurementAvailableOf<S>>, S>
where
    S: LayoutScalar,
{
    Ok(Size::new(
        MeasurementAvailableOf::from_content_space(available.width)?,
        MeasurementAvailableOf::from_content_space(available.height)?,
    ))
}

fn invalid_numeric_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    operation: LayoutOperation,
    value: S,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value }),
    )
}

fn leaf_measurement_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    error: LeafMeasureErrorOf<S, M>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let kind = match error {
        LeafMeasureErrorOf::Provider(error) => LayoutErrorKindOf::Measurement(error),
        LeafMeasureErrorOf::InvalidOutput(error) => {
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(error))
        }
    };
    LayoutErrorOf::new(site, LayoutOperation::LeafMeasurement, kind)
}

fn resolve_length_or_zero_fallible<S>(
    length: super::LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    resolution_or_zero_fallible(length.resolve_with_status(basis))
}

fn resolve_auto_or_zero_fallible<S>(
    length: super::LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    Ok(resolution_optional_fallible(length.resolve_with_status(basis))?.unwrap_or(S::ZERO))
}

fn resolve_leaf_auto<S>(
    input: ComputeInputOf<S>,
    length: super::LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    Ok(resolve_leaf_optional(input, length.resolve_with_status(basis))?.unwrap_or(S::ZERO))
}

fn resolve_leaf_length<S>(
    input: ComputeInputOf<S>,
    length: super::LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    let resolution = length.resolve_with_status(basis);
    if input.run_mode() == RunMode::ComputeSize
        && matches!(resolution.status(), LengthResolutionStatus::MissingBasis)
    {
        return Ok(S::ZERO);
    }

    resolution_or_zero_fallible(resolution)
}

fn resolve_leaf_optional<S>(
    input: ComputeInputOf<S>,
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    if input.run_mode() == RunMode::ComputeSize
        && matches!(resolution.status(), LengthResolutionStatus::MissingBasis)
    {
        return Ok(None);
    }

    resolution_optional_fallible(resolution)
}

fn resolution_or_zero_fallible<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution
            .value
            .expect("resolved length resolution must carry a value")),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            Err(resolution.status())
        }
        LengthResolutionStatus::NonNumeric => Ok(S::ZERO),
    }
}

fn resolution_optional_fallible<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution.value),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            Err(resolution.status())
        }
        LengthResolutionStatus::NonNumeric => Ok(None),
    }
}

pub(crate) trait SizeResultExt<S: LayoutScalar> {
    type Output;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>;
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<S, LengthResolutionStatus<S>>> {
    type Output = Size<S>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|status| value_resolution_error(node, status))?,
            self.height
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<Option<S>, LengthResolutionStatus<S>>> {
    type Output = Size<Option<S>>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|status| value_resolution_error(node, status))?,
            self.height
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<Option<S>, SizingResolutionError<S>>> {
    type Output = Size<Option<S>>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|error| sizing_resolution_error(node, error))?,
            self.height
                .map_err(|error| sizing_resolution_error(node, error))?,
        ))
    }
}

pub(crate) trait EdgesResultExt<S: LayoutScalar> {
    type Output;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>;
}

impl<S: LayoutScalar> EdgesResultExt<S> for super::Edges<Result<S, LengthResolutionStatus<S>>> {
    type Output = super::Edges<S>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(super::Edges::new(
            self.top
                .map_err(|status| value_resolution_error(node, status))?,
            self.right
                .map_err(|status| value_resolution_error(node, status))?,
            self.bottom
                .map_err(|status| value_resolution_error(node, status))?,
            self.left
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> EdgesResultExt<S>
    for super::Edges<Result<Option<S>, LengthResolutionStatus<S>>>
{
    type Output = super::Edges<Option<S>>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(super::Edges::new(
            self.top
                .map_err(|status| value_resolution_error(node, status))?,
            self.right
                .map_err(|status| value_resolution_error(node, status))?,
            self.bottom
                .map_err(|status| value_resolution_error(node, status))?,
            self.left
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

#[cfg(test)]
mod fri08_c07_t03_optional_math_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0)).max_optional(Size::new(None, Some(scalar(15.0)))),
            Size::new(scalar(4.0), scalar(15.0))
        );
        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0))
                .max_optional(Size::new(Some(scalar(9.0)), Some(scalar(3.0)),)),
            Size::new(scalar(9.0), scalar(12.0))
        );
    }

    #[test]
    fn fri08_c07_t03_optional_math_compute_minimum_floor_preserves_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri08_c07_t03_optional_math_compute_minimum_floor_preserves_f64() {
        characterize::<f64>();
    }
}

#[cfg(test)]
mod fri06_c13_t05_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let optional = Size::new(None, Some(scalar(9.0)));

        assert_eq!(
            optional.or(Size::new(Some(scalar(4.0)), Some(scalar(3.0)))),
            Size::new(Some(scalar(4.0)), Some(scalar(9.0)))
        );
        assert_eq!(
            optional.unwrap_or(Size::new(scalar(6.0), scalar(7.0))),
            Size::new(scalar(6.0), scalar(9.0))
        );
        assert_eq!(
            optional.add_optional(Size::new(scalar(2.0), scalar(3.0))),
            Size::new(None, Some(scalar(12.0)))
        );

        let Some(ratio) = AspectRatioOf::new(scalar(2.0)) else {
            panic!("finite positive test aspect ratio must be accepted");
        };
        assert_eq!(
            Size::new(Some(scalar(12.0)), None).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(12.0)), Some(scalar(6.0)))
        );
        assert_eq!(
            Size::new(None, Some(scalar(7.0))).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(14.0)), Some(scalar(7.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_max_before_min_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            scalar(5.0).clamp_max_before_min_optional(Some(scalar(10.0)), Some(scalar(3.0))),
            scalar(10.0)
        );
    }

    #[test]
    fn fri06_c13_t05_compute_optional_math_and_clamp_order_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_compute_optional_math_and_clamp_order_preserve_f64() {
        characterize::<f64>();
    }
}

trait AvailableExt {
    type Scalar: LayoutScalar;

    fn sub_margin(self, margin: Self::Scalar) -> Self;
    fn set_optional(self, value: Option<Self::Scalar>) -> Self;
    fn map_definite(self, f: impl FnOnce(Self::Scalar) -> Self::Scalar) -> Self;
}

impl<S: LayoutScalar> AvailableExt for AvailableOf<S> {
    type Scalar = S;

    fn sub_margin(self, margin: S) -> Self {
        self.map_definite(|value| value - margin)
    }

    fn set_optional(self, value: Option<S>) -> Self {
        value.map_or(self, AvailableOf::definite)
    }

    fn map_definite(self, f: impl FnOnce(S) -> S) -> Self {
        match self {
            AvailableOf::Definite(value) => AvailableOf::Definite(f(value)),
            AvailableOf::MinContent => AvailableOf::MinContent,
            AvailableOf::MaxContent => AvailableOf::MaxContent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BidiLevel, InlineBreakOpportunityOf, InlineFragmentOutputOf, InlineMetricsOf,
        InlineSegmentId, InlineTextInputOf, InlineWhitespaceEdge, LayoutInputOf, LayoutTree,
        NodeInput, ScrollRectOf, ShapedInlineSegmentOf, Traverse,
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

    fn assert_fri06_c01_fragment_rounding_overflow<S: LayoutScalar>(largest: S) {
        let fragment = InlineFragmentOutputOf::new(
            InlineSegmentId::new(1),
            ScrollRectOf::try_new(Point::new(largest, S::ZERO), Size::ZERO).unwrap(),
            Point::new(largest, S::ZERO),
            0,
            0,
            None,
        );

        let error = round_inline_fragment::<u32, S, ()>(7, fragment, Point::new(largest, S::ZERO))
            .unwrap_err();

        assert_eq!(error.site(), LayoutErrorSiteOf::Node(7));
        assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidRoundedInlineFragmentGeometry,
            )
        );
    }

    #[test]
    fn fri06_c02_rounding_overflow_returns_typed_error_without_panic() {
        assert_fri06_c01_fragment_rounding_overflow(f32::MAX);
        assert_fri06_c01_fragment_rounding_overflow(f64::MAX);
    }

    fn assert_fri06_mr02_layout_round_fragment_boundaries<S: LayoutScalar>() {
        for (value, expected) in [
            (-2.0, -2.0),
            (-1.51, -2.0),
            (-1.5, -1.0),
            (-1.49, -1.0),
            (-0.51, -1.0),
            (-0.5, 0.0),
            (-0.49, 0.0),
            (0.0, 0.0),
            (0.49, 0.0),
            (0.5, 1.0),
            (0.51, 1.0),
            (1.49, 1.0),
            (1.5, 2.0),
            (1.51, 2.0),
            (2.0, 2.0),
            (-1_048_576.5, -1_048_576.0),
            (1_048_576.5, 1_048_577.0),
        ] {
            let value = S::from_f64(value);
            let expected = S::from_f64(expected);
            let fragment = InlineFragmentOutputOf::new(
                InlineSegmentId::new(1),
                ScrollRectOf::try_new(Point::new(value, value), Size::ZERO).unwrap(),
                Point::new(value, value),
                0,
                0,
                None,
            );

            let rounded = round_inline_fragment::<u32, S, ()>(7, fragment, Point::ZERO).unwrap();

            assert_eq!(rounded.rect().origin(), Point::new(expected, expected));
            assert_eq!(rounded.baseline(), Point::new(expected, expected));
        }

        for (value, cumulative, expected) in [
            (0.25, 0.25, 1.0),
            (0.5, -0.25, 0.0),
            (-0.5, -0.25, -1.0),
            (1.49, 10.25, 2.0),
            (-1.49, -10.25, -2.0),
        ] {
            let value = S::from_f64(value);
            let cumulative = S::from_f64(cumulative);
            let expected = S::from_f64(expected);
            let fragment = InlineFragmentOutputOf::new(
                InlineSegmentId::new(1),
                ScrollRectOf::try_new(Point::new(value, value), Size::ZERO).unwrap(),
                Point::new(value, value),
                0,
                0,
                None,
            );

            let rounded = round_inline_fragment::<u32, S, ()>(
                7,
                fragment,
                Point::new(cumulative, cumulative),
            )
            .unwrap();

            assert_eq!(rounded.rect().origin(), Point::new(expected, expected));
            assert_eq!(rounded.baseline(), Point::new(expected, expected));
        }
    }

    #[test]
    fn fri06_mr02_layout_round_fragments_baselines_and_cumulative_origins_are_preserved() {
        assert_fri06_mr02_layout_round_fragment_boundaries::<f32>();
        assert_fri06_mr02_layout_round_fragment_boundaries::<f64>();
    }

    #[test]
    fn fri06_mr02_layout_round_fragment_overflow_preserves_typed_error() {
        assert_fri06_c01_fragment_rounding_overflow(f32::MAX);
        assert_fri06_c01_fragment_rounding_overflow(f64::MAX);
    }

    fn assert_fri06_c13_t06_leaf_missing_basis_counterexample<S: LayoutScalar>() {
        let style = NodeInputOf::<S> {
            padding: Edges::all(super::super::LengthOf::percent(S::from_f64(0.5))),
            ..NodeInputOf::default()
        };
        let context = super::super::ContainingLayoutContext::new(
            FlowAxes::new(
                super::super::WritingMode::HorizontalTb,
                super::super::Direction::Ltr,
            ),
            super::super::ParentFormattingContext::NoParent,
        );
        let available = Size::splat(AvailableOf::MAX_CONTENT);
        let measured = Size::new(S::from_f64(12.0), S::from_f64(8.0));

        let content_size_input =
            ComputeInputOf::leaf_content_size(Size::NONE, Size::NONE, context, available)
                .expect("indefinite intrinsic leaf input is valid");
        let content_size = compute_leaf(content_size_input, &style, |_input| Ok::<_, ()>(measured))
            .expect("ComputeSize explicitly treats missing edge basis as zero");
        assert_eq!(content_size.size, measured);

        let layout_input = ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, context, available)
            .expect("indefinite layout leaf input is valid");
        let error = compute_leaf(layout_input, &style, |_input| -> Result<Size<S>, ()> {
            panic!("layout missing-basis failure must precede measurement")
        })
        .expect_err("layout keeps missing edge basis fallible");
        assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::MissingContext(super::super::LayoutMissingContext::RequiredBasis,)
        );
    }

    #[test]
    fn fri06_c13_t06_leaf_missing_basis_compute_size_and_layout_differ_f32() {
        assert_fri06_c13_t06_leaf_missing_basis_counterexample::<f32>();
    }

    #[test]
    fn fri06_c13_t06_leaf_missing_basis_compute_size_and_layout_differ_f64() {
        assert_fri06_c13_t06_leaf_missing_basis_counterexample::<f64>();
    }
}
