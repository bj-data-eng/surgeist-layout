use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use crate::*;

#[derive(Clone, Debug, Default)]
struct InvalidationTree {
    children: HashMap<u32, Vec<u32>>,
    child_adjacency_queries: Cell<usize>,
    styles: HashMap<u32, NodeInput>,
    measured: HashSet<u32>,
    measurements: RefCell<HashMap<u32, Result<Size, &'static str>>>,
    measurement_calls: RefCell<Vec<u32>>,
    cache_queries: RefCell<Vec<(u32, bool)>>,
    caches: RefCell<HashMap<u32, Cache>>,
    traversal_budget: Cell<Option<usize>>,
}

impl InvalidationTree {
    fn exact_tree() -> Self {
        let mut tree = Self::default();
        tree.children.insert(0, vec![1, 2]);
        tree.children.insert(1, vec![3, 4]);
        tree.children.insert(2, vec![5]);
        for node in 0..=5 {
            tree.children.entry(node).or_default();
            tree.styles.insert(node, NodeInput::default());
        }
        tree
    }

    fn with_measured_leaves(mut self) -> Self {
        for (node, size) in [
            (3, Size::new(13.0, 7.0)),
            (4, Size::new(17.0, 11.0)),
            (5, Size::new(19.0, 5.0)),
        ] {
            self.measured.insert(node);
            self.measurements.borrow_mut().insert(node, Ok(size));
        }
        self
    }

    fn with_topology(children: &[(u32, &[u32])]) -> Self {
        let mut tree = Self::default();
        for (node, children) in children {
            tree.children.insert(*node, children.to_vec());
            tree.styles.insert(*node, NodeInput::default());
            for child in *children {
                tree.children.entry(*child).or_default();
                tree.styles.insert(*child, NodeInput::default());
            }
        }
        tree
    }

    fn with_traversal_budget(self, budget: usize) -> Self {
        self.traversal_budget.set(Some(budget));
        self
    }

    fn apply_cache_entries(&self, entries: &[LayoutCacheStoreEntry<u32>]) {
        let mut caches = self.caches.borrow_mut();
        for entry in entries {
            caches.entry(entry.node()).or_default().store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
    }
}

impl Traverse for InvalidationTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.child_adjacency_queries
            .set(self.child_adjacency_queries.get() + 1);
        if let Some(remaining) = self.traversal_budget.get() {
            assert!(
                remaining > 0,
                "cycle traversal exceeded its bounded test budget"
            );
            self.traversal_budget.set(Some(remaining - 1));
        }
        self.children[&node].iter().copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children[&node].len()
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl LayoutTree for InvalidationTree {
    type MeasureError = &'static str;

    fn node_input(&self, node: Self::Node) -> &NodeInput {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInput {
        LayoutInput::box_input(self.styles[&node].clone())
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.measured.contains(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        _input: LeafMeasureInput,
    ) -> Option<Result<Size, Self::MeasureError>> {
        self.measurement_calls.borrow_mut().push(node);
        self.measurements.borrow().get(&node).cloned()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInput,
        context: CacheKeyContext,
    ) -> Option<ComputeOutput> {
        let output = self
            .caches
            .borrow()
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }
}

fn invalidation_request() -> LayoutRootRequest {
    LayoutRootRequest::viewport(Size::new(
        Available::definite(200.0),
        Available::definite(120.0),
    ))
    .unwrap()
}

#[test]
fn fri06_c01_invalidation_exact_source_order_inclusive_closures() {
    let tree = InvalidationTree::exact_tree();
    let cases: &[(&[u32], &[u32])] = &[
        (&[], &[]),
        (&[4], &[0, 1, 4]),
        (&[4, 2], &[0, 1, 4, 2]),
        (&[1, 3], &[0, 1, 3]),
        (&[4, 4, 1, 4], &[0, 1, 4]),
        (&[0], &[0]),
    ];

    for (changed, expected) in cases {
        let batch = compute_layout_invalidated(&tree, 0, invalidation_request(), changed)
            .expect("reachable changed set computes");
        assert_eq!(batch.invalidated_nodes(), *expected, "changed {changed:?}");
        assert_eq!(
            batch
                .cache_clear_entries()
                .iter()
                .map(LayoutCacheClearEntry::node)
                .collect::<Vec<_>>(),
            *expected,
            "closure clears remain source ordered and deduplicated"
        );
    }
}

#[test]
fn fri06_c01_invalidation_self_cycle_returns_typed_topology_error() {
    let tree = InvalidationTree::with_topology(&[(0, &[0])]).with_traversal_budget(8);

    let error = compute_layout_invalidated(&tree, 0, invalidation_request(), &[0])
        .expect_err("self-cycle must fail before layout");

    assert_eq!(
        error.site(),
        LayoutErrorSite::ContainerSubject {
            container: 0,
            subject: 0,
        }
    );
    assert_eq!(error.operation(), LayoutOperation::CacheInvalidation);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::TreeTopologyCycle)
    );
}

#[test]
fn fri06_c01_invalidation_multi_node_cycle_returns_typed_topology_error() {
    let tree = InvalidationTree::with_topology(&[(0, &[1]), (1, &[2]), (2, &[0])])
        .with_traversal_budget(12);

    let error = compute_layout_invalidated(&tree, 0, invalidation_request(), &[2])
        .expect_err("multi-node cycle must fail before layout");

    assert_eq!(
        error.site(),
        LayoutErrorSite::ContainerSubject {
            container: 2,
            subject: 0,
        }
    );
    assert_eq!(error.operation(), LayoutOperation::CacheInvalidation);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::TreeTopologyCycle)
    );
}

#[test]
fn fri06_c01_invalidation_repeated_child_preserves_source_order_and_each_path() {
    let tree = InvalidationTree::with_topology(&[(0, &[1, 1]), (1, &[2]), (2, &[])]);

    let batch = compute_layout_invalidated(&tree, 0, invalidation_request(), &[2])
        .expect("a repeated child identity is not a topology cycle");

    assert_eq!(batch.invalidated_nodes(), &[0, 1, 2]);
}

#[test]
fn fri06_c01_invalidation_dag_unions_every_inclusive_path_in_source_order() {
    let tree = InvalidationTree::with_topology(&[(0, &[1, 2]), (1, &[3]), (2, &[3]), (3, &[])]);

    let batch = compute_layout_invalidated(&tree, 0, invalidation_request(), &[3])
        .expect("a shared DAG descendant is not a topology cycle");

    assert_eq!(batch.invalidated_nodes(), &[0, 1, 3, 2]);
}

#[test]
fn fri06_c01_invalidation_shared_repeated_edge_dag_expands_each_adjacency_once() {
    const DEPTH: u32 = 9;

    let mut topology = Vec::new();
    topology.push((0, vec![1, 1, 2, 2]));
    for level in 1..DEPTH {
        let left = level * 2 - 1;
        let right = level * 2;
        let next_left = left + 2;
        let next_right = right + 2;
        let repeated_children = vec![next_left, next_left, next_right, next_right];
        topology.push((left, repeated_children.clone()));
        topology.push((right, repeated_children));
    }
    topology.push((DEPTH * 2 - 1, Vec::new()));
    topology.push((DEPTH * 2, Vec::new()));
    let borrowed_topology = topology
        .iter()
        .map(|(node, children)| (*node, children.as_slice()))
        .collect::<Vec<_>>();
    let tree = InvalidationTree::with_topology(&borrowed_topology);

    let error = compute_layout_invalidated(&tree, 0, invalidation_request(), &[u32::MAX])
        .expect_err("unreachable subject must fail after bounded graph discovery");

    assert_eq!(error.site(), LayoutErrorSite::Node(u32::MAX));
    assert_eq!(error.operation(), LayoutOperation::CacheInvalidation);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidationNodeNotReachable)
    );
    assert_eq!(tree.child_adjacency_queries.get(), (DEPTH * 2 + 1) as usize);
}

#[test]
fn fri06_c01_invalidation_unreachable_subject_has_exact_diagnostic() {
    let tree = InvalidationTree::exact_tree();
    let caches_before = tree.caches.borrow().clone();

    let error = compute_layout_invalidated(&tree, 0, invalidation_request(), &[4, 99])
        .expect_err("unreachable changed subject must fail before layout");

    assert_eq!(error.site(), LayoutErrorSite::Node(99));
    assert_eq!(error.operation(), LayoutOperation::CacheInvalidation);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidationNodeNotReachable)
    );
    assert_eq!(*tree.caches.borrow(), caches_before);
    assert!(tree.measurement_calls.borrow().is_empty());
}

#[test]
fn fri06_c01_invalidation_bypasses_only_closure_and_stages_replacement() {
    let tree = InvalidationTree::exact_tree().with_measured_leaves();
    let cold = compute_layout(&tree, 0, invalidation_request()).expect("cold layout computes");
    tree.apply_cache_entries(cold.cache_store_entries());
    tree.measurement_calls.borrow_mut().clear();
    tree.cache_queries.borrow_mut().clear();
    let caches_before = tree.caches.borrow().clone();

    let batch = compute_layout_invalidated(&tree, 0, invalidation_request(), &[4])
        .expect("dirty leaf recomputes against preserved committed state");

    assert_eq!(batch.invalidated_nodes(), &[0, 1, 4]);
    let calls = tree.measurement_calls.borrow().clone();
    assert!(calls.contains(&4), "calls were {calls:?}");
    let cache_queries = tree.cache_queries.borrow().clone();
    assert!(
        cache_queries.iter().all(|(node, _)| *node != 4),
        "dirty leaf must bypass every stale lookup: {cache_queries:?}"
    );
    assert!(
        cache_queries
            .iter()
            .any(|(node, hit)| (*node == 3 || *node == 5) && *hit),
        "ordinary descendants must retain warm hits: {cache_queries:?}"
    );
    assert!(
        batch
            .cache_store_entries()
            .iter()
            .any(|entry| entry.node() == 4)
    );
    assert_eq!(*tree.caches.borrow(), caches_before);
    assert_eq!(CacheKeyContext::new(), CacheKeyContext);
    assert_eq!(std::mem::size_of::<CacheKeyContext>(), 0);
}

#[test]
fn fri06_c01_invalidation_legacy_compute_is_empty_dirty_set() {
    let tree = InvalidationTree::exact_tree().with_measured_leaves();
    let legacy = compute_layout(&tree, 0, invalidation_request()).expect("legacy layout computes");
    let explicit = compute_layout_invalidated(&tree, 0, invalidation_request(), &[])
        .expect("empty invalidation computes");

    assert_eq!(legacy, explicit);
    assert!(legacy.invalidated_nodes().is_empty());
}

#[test]
fn fri06_c01_invalidation_layout_failure_preserves_cache_and_dirty_subjects() {
    let tree = InvalidationTree::exact_tree().with_measured_leaves();
    let cold = compute_layout(&tree, 0, invalidation_request()).expect("cold layout computes");
    tree.apply_cache_entries(cold.cache_store_entries());
    tree.measurements
        .borrow_mut()
        .insert(4, Err("dirty measurement failed"));
    tree.measurement_calls.borrow_mut().clear();
    tree.cache_queries.borrow_mut().clear();
    let caches_before = tree.caches.borrow().clone();
    let dirty = vec![4];

    let error = compute_layout_invalidated(&tree, 0, invalidation_request(), &dirty)
        .expect_err("dirty recomputation failure returns no batch");

    assert_eq!(error.site(), LayoutErrorSite::Node(4));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::Measurement("dirty measurement failed")
    );
    assert_eq!(*tree.caches.borrow(), caches_before);
    assert_eq!(dirty, vec![4]);
    assert!(tree.measurement_calls.borrow().contains(&4));
    assert!(
        tree.cache_queries
            .borrow()
            .iter()
            .all(|(node, _)| *node != 4)
    );
}

#[derive(Clone, Debug, PartialEq)]
struct PreparedReplacement {
    unrounded: Vec<LayoutOutputEntry<u32>>,
    final_layout: Vec<LayoutOutputEntry<u32>>,
    unrounded_fragments: Vec<InlineFragmentOutputEntry<u32>>,
    final_fragments: Vec<InlineFragmentOutputEntry<u32>>,
    clears: Vec<LayoutCacheClearEntry<u32>>,
    stores: Vec<LayoutCacheStoreEntry<u32>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Mutation {
    ReplaceUnrounded(u32),
    ReplaceFinal(u32),
    ReplaceUnroundedFragments(u32),
    ReplaceFinalFragments(u32),
    Clear(u32),
    Store(u32),
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TransactionSink {
    fail_preparation: bool,
    unrounded: Vec<LayoutOutputEntry<u32>>,
    final_layout: Vec<LayoutOutputEntry<u32>>,
    unrounded_fragments: Vec<InlineFragmentOutputEntry<u32>>,
    final_fragments: Vec<InlineFragmentOutputEntry<u32>>,
    clears: Vec<LayoutCacheClearEntry<u32>>,
    stores: Vec<LayoutCacheStoreEntry<u32>>,
    mutations: RefCell<Vec<Mutation>>,
}

impl LayoutBatchSink<u32, Scalar> for TransactionSink {
    type Error = &'static str;
    type Prepared = PreparedReplacement;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatch<u32>,
    ) -> Result<Self::Prepared, Self::Error> {
        if self.fail_preparation {
            return Err("immutable preparation failed");
        }
        Ok(PreparedReplacement {
            unrounded: batch.unrounded_entries().to_vec(),
            final_layout: batch.final_entries().to_vec(),
            unrounded_fragments: batch.unrounded_inline_fragments().to_vec(),
            final_fragments: batch.final_inline_fragments().to_vec(),
            clears: batch.cache_clear_entries().to_vec(),
            stores: batch.cache_store_entries().to_vec(),
        })
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.unrounded.clear();
        for entry in prepared.unrounded {
            let node = entry.node();
            self.unrounded.push(entry);
            self.mutations
                .borrow_mut()
                .push(Mutation::ReplaceUnrounded(node));
        }
        self.final_layout.clear();
        for entry in prepared.final_layout {
            let node = entry.node();
            self.final_layout.push(entry);
            self.mutations
                .borrow_mut()
                .push(Mutation::ReplaceFinal(node));
        }
        self.unrounded_fragments.clear();
        for entry in prepared.unrounded_fragments {
            let node = entry.node();
            self.unrounded_fragments.push(entry);
            self.mutations
                .borrow_mut()
                .push(Mutation::ReplaceUnroundedFragments(node));
        }
        self.final_fragments.clear();
        for entry in prepared.final_fragments {
            let node = entry.node();
            self.final_fragments.push(entry);
            self.mutations
                .borrow_mut()
                .push(Mutation::ReplaceFinalFragments(node));
        }
        self.clears.clear();
        for entry in prepared.clears {
            let node = entry.node();
            self.clears.push(entry);
            self.mutations.borrow_mut().push(Mutation::Clear(node));
        }
        self.stores.clear();
        for entry in prepared.stores {
            let node = entry.node();
            self.stores.push(entry);
            self.mutations.borrow_mut().push(Mutation::Store(node));
        }
    }
}

fn transaction_batch() -> CompletedLayoutBatch<u32> {
    let fragment = InlineFragmentOutput::new(
        InlineSegmentId::new(7),
        ScrollRect::try_new(Point::new(1.0, 2.0), Size::new(3.0, 4.0)).unwrap(),
        Point::new(1.0, 5.0),
        0,
        0,
        None,
    );
    let input = ComputeInput::root_layout(
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::splat(Available::definite(100.0)),
    );
    CompletedLayoutBatch::from_entries(
        vec![LayoutOutputEntry::new(1, NodeOutput::new())],
        vec![LayoutOutputEntry::new(1, NodeOutput::new())],
        vec![InlineFragmentOutputEntry::new(1, fragment)],
        vec![InlineFragmentOutputEntry::new(1, fragment)],
        vec![
            LayoutCacheStoreEntry::new(
                1,
                input,
                CacheKeyContext::new(),
                ComputeOutput::from_outer_size(Size::new(10.0, 20.0)),
            ),
            LayoutCacheStoreEntry::new(
                2,
                input,
                CacheKeyContext::new(),
                ComputeOutput::from_outer_size(Size::new(30.0, 40.0)),
            ),
        ],
        vec![LayoutCacheClearEntry::new(1), LayoutCacheClearEntry::new(2)],
        vec![1, 2],
    )
}

fn apply_and_release_dirty(
    batch: &CompletedLayoutBatch<u32>,
    sink: &mut TransactionSink,
    dirty: &mut Vec<u32>,
) -> Result<(), &'static str> {
    batch.apply_to(sink)?;
    dirty.clear();
    Ok(())
}

#[test]
fn fri06_c01_batch_transaction_preparation_failure_mutates_nothing_and_keeps_dirty() {
    let batch = transaction_batch();
    let mut sink = TransactionSink {
        fail_preparation: true,
        unrounded: vec![LayoutOutputEntry::new(90, NodeOutput::new())],
        ..TransactionSink::default()
    };
    let before = sink.clone();
    assert!(sink.mutations.borrow().is_empty());
    let mut dirty = vec![1];

    let error = apply_and_release_dirty(&batch, &mut sink, &mut dirty).unwrap_err();

    assert_eq!(error, "immutable preparation failed");
    assert_eq!(sink, before);
    assert!(sink.mutations.borrow().is_empty());
    assert_eq!(dirty, vec![1]);
}

#[test]
fn fri06_c01_batch_transaction_successful_preparation_uses_no_interior_mutation() {
    let batch = transaction_batch();
    let sink = TransactionSink {
        unrounded: vec![LayoutOutputEntry::new(90, NodeOutput::new())],
        ..TransactionSink::default()
    };
    let before = sink.clone();

    let prepared = sink
        .prepare_layout_batch(&batch)
        .expect("immutable preparation succeeds");

    assert_eq!(sink, before);
    assert!(sink.mutations.borrow().is_empty());
    assert_eq!(prepared.unrounded, batch.unrounded_entries());
    assert_eq!(prepared.final_layout, batch.final_entries());
    assert_eq!(
        prepared.unrounded_fragments,
        batch.unrounded_inline_fragments()
    );
    assert_eq!(prepared.final_fragments, batch.final_inline_fragments());
    assert_eq!(prepared.clears, batch.cache_clear_entries());
    assert_eq!(prepared.stores, batch.cache_store_entries());
}

#[test]
fn fri06_c01_batch_transaction_owned_commit_replaces_every_class_in_order() {
    let batch = transaction_batch();
    let mut sink = TransactionSink::default();
    let mut dirty = vec![1];

    apply_and_release_dirty(&batch, &mut sink, &mut dirty).expect("transaction commits");

    assert_eq!(sink.unrounded, batch.unrounded_entries());
    assert_eq!(sink.final_layout, batch.final_entries());
    assert_eq!(sink.unrounded_fragments, batch.unrounded_inline_fragments());
    assert_eq!(sink.final_fragments, batch.final_inline_fragments());
    assert_eq!(sink.clears, batch.cache_clear_entries());
    assert_eq!(sink.stores, batch.cache_store_entries());
    assert_eq!(
        *sink.mutations.borrow(),
        [
            Mutation::ReplaceUnrounded(1),
            Mutation::ReplaceFinal(1),
            Mutation::ReplaceUnroundedFragments(1),
            Mutation::ReplaceFinalFragments(1),
            Mutation::Clear(1),
            Mutation::Clear(2),
            Mutation::Store(1),
            Mutation::Store(2),
        ]
    );
    assert!(dirty.is_empty());
}
