use super::fixtures::{
    Fri06C02StatefulTextTree, Fri06C05ShapeProvider, PublicFlowTree, RootSessionTree,
    computed_overflow, fri05_c03_block_root_state, fri05_c04_assert_initial_local_auto_state,
    fri05_c04_local_auto_state, fri05_c04_nested_flex_auto_tree, fri06_c02_final_node,
    fri06_c02_segment, fri06_c02_stateful_request, fri06_c02_text_nodes_batch,
    fri06_mr02_geometry_error_largest_finite, public_flow_output, public_layout_tree, scalar,
};
use super::*;

#[test]
fn fri08_c07_t02_scroll_source_root_preserves_error_identity_without_batch() {
    assert_fri06_mr02_geometry_error_public_root_has_no_batch::<f32>();
    assert_fri06_mr02_geometry_error_public_root_has_no_batch::<f64>();
}

#[test]
fn fri06_c04_float_lifecycle_cold_warm_dirty_failures_baseline_and_content_are_atomic_both_scalars()
{
    fn assert_lane<S: LayoutScalar>() {
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(40.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let mut tree = Fri06C02StatefulTextTree::new_float_mixed();
        let cold = compute_layout(&tree, 0, request).expect("cold float/line layout succeeds");
        let cold_unrounded = cold.unrounded_entries().to_vec();
        let cold_final = cold.final_entries().to_vec();
        let cold_fragments = cold.unrounded_inline_fragments().to_vec();
        assert_eq!(
            public_flow_output(&cold_unrounded, 0).size.height,
            S::from_f64(12.5)
        );
        assert_eq!(
            public_flow_output(&cold_unrounded, 0).content_size,
            Size::new(S::from_f64(40.0), S::from_f64(12.5)),
        );
        assert_eq!(cold_fragments[0].fragment().baseline().y, S::from_f64(8.0));
        assert!(
            cold.cache_store_entries()
                .iter()
                .any(|entry| entry.node() == 1)
        );
        cold.apply_to(&mut tree).expect("cold float batch commits");

        let warm = compute_layout(&tree, 0, request).expect("warm float/line layout succeeds");
        assert_eq!(warm.unrounded_entries(), cold_unrounded);
        assert_eq!(warm.final_entries(), cold_final);
        assert_eq!(warm.unrounded_inline_fragments(), cold_fragments);
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 1)
        );
        warm.apply_to(&mut tree)
            .expect("warm float batch recommits");

        let stale_float = tree.retained.unrounded_nodes[&1];
        let stale_text = tree.retained.unrounded_nodes[&2];
        tree.replace_float_inline_extent(25.75);
        tree.retained.dirty = vec![1, 1];
        let replacement = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("dirty float replacement stages");
        assert_eq!(replacement.invalidated_nodes(), &[0, 1]);
        replacement
            .apply_to(&mut tree)
            .expect("dirty float replacement commits");
        assert_ne!(tree.retained.unrounded_nodes[&1], stale_float);
        assert_eq!(
            tree.retained.unrounded_nodes[&1].size.width,
            S::from_f64(25.75)
        );
        assert_ne!(
            tree.retained.unrounded_nodes[&2].location,
            stale_text.location
        );
        assert!(tree.retained.dirty.is_empty());

        tree.replace_float_inline_extent(26.25);
        tree.retained.dirty = vec![1];
        tree.add_failing_float_path_control();
        let before_layout_failure = tree.retained.clone();
        let error = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect_err("invalid float-path control pairing rejects the whole layout batch");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(9));
        assert_eq!(tree.retained, before_layout_failure);

        tree.remove_failing_float_path_control();
        let rejected = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("replacement after layout failure stages");
        tree.reject_preparation = true;
        let before_preparation_failure = tree.retained.clone();
        assert_eq!(
            rejected.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, before_preparation_failure);
        assert_eq!(tree.retained.dirty, [1]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_lifecycle_static_one_ledger_query_and_single_publication_paths() {
    let block = include_str!("../block/inline_run.rs");
    let floats = include_str!("../block/floats.rs");
    assert_eq!(floats.matches("struct FloatLedgerEntry<").count(), 1);
    assert_eq!(
        floats
            .matches("ledger: Vec<FloatLedgerEntry<S, Node>>")
            .count(),
        2
    );
    assert_eq!(floats.matches("fn query_band_for<").count(), 1);
    assert_eq!(floats.matches("struct FloatExclusions<").count(), 1);
    for legacy in [
        concat!("layout_inline_", "segments"),
        concat!("left_", "floats"),
        concat!("right_", "floats"),
        concat!("physical_", "left"),
        concat!("physical_", "right"),
        concat!("FloatBand", "Table"),
    ] {
        assert!(
            !floats.contains(legacy),
            "legacy float path remains: {legacy}"
        );
    }

    let float_publication = floats
        .split_once("fn layout_floats<")
        .unwrap()
        .1
        .split_once("\npub(super) struct FloatIntrinsics<")
        .unwrap()
        .0;
    assert_eq!(
        float_publication
            .matches(".include_in_flow_geometry(location, float.margin, scroll_geometry)")
            .count(),
        1,
    );
    assert_eq!(float_publication.matches("tree.set_unrounded(").count(), 1);

    let line_publication = block
        .split_once("fn layout_inline_run_children<")
        .unwrap()
        .1
        .split_once("\nfn record_inline_run_baselines<")
        .unwrap()
        .0;
    assert_eq!(
        line_publication
            .matches("contributions.include_direct_line(rect);")
            .count(),
        1,
    );

    let cache = include_str!("../cache.rs");
    assert!(!cache.contains("float_provider"));
    assert!(!cache.contains("shape_provider"));
}

#[test]
fn fri06_c02_fragment_publication_is_per_node_source_ordered_and_retains_empty_anchors_both_scalars()
 {
    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c02_text_nodes_batch(
            vec![
                (
                    1,
                    vec![
                        fri06_c02_segment(
                            11,
                            8.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::mandatory(),
                        ),
                        fri06_c02_segment(
                            12,
                            6.0,
                            InlineWhitespaceEdge::Preserve,
                            InlineBreakOpportunityOf::prohibited(),
                        ),
                    ],
                ),
                (
                    2,
                    vec![fri06_c02_segment(
                        21,
                        7.0,
                        InlineWhitespaceEdge::DiscardAtBoth,
                        InlineBreakOpportunityOf::mandatory(),
                    )],
                ),
            ],
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
            Size::new(
                AvailableOf::definite(S::from_f64(30.0)),
                AvailableOf::MAX_CONTENT,
            ),
        );
        assert_eq!(
            batch
                .final_inline_fragments()
                .iter()
                .map(|entry| (entry.node(), entry.fragment().segment_id().get()))
                .collect::<Vec<_>>(),
            vec![(1, 11), (1, 12)]
        );
        let first = fri06_c02_final_node(&batch, 1);
        assert_eq!(first.location, Point::ZERO);
        assert_eq!(first.size, Size::new(S::from_f64(8.0), S::from_f64(20.0)));
        assert_eq!(first.content_size, first.size);
        assert_eq!(first.border, Edges::ZERO);
        assert_eq!(first.padding, Edges::ZERO);
        assert_eq!(first.margin, Edges::ZERO);
        assert!(first.scroll_geometry.is_none());
        let empty = fri06_c02_final_node(&batch, 2);
        assert_eq!(
            empty.location,
            Point::new(S::from_f64(6.0), S::from_f64(10.0))
        );
        assert_eq!(empty.size, Size::ZERO);
        assert_eq!(empty.content_size, Size::ZERO);
        assert!(empty.scroll_geometry.is_none());

        let hidden = fri06_c02_text_nodes_batch(
            vec![(
                1,
                vec![fri06_c02_segment(
                    31,
                    10.0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                )],
            )],
            NodeInputOf {
                display: Display::None,
                ..NodeInputOf::default()
            },
            Size::splat(AvailableOf::definite(S::from_f64(30.0))),
        );
        assert!(hidden.final_inline_fragments().is_empty());
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

type Fri06C03FragmentIdentity<S> = (u32, InlineSegmentId, usize, usize, Option<S>);

#[test]
fn fri06_c05_provider_atomicity_layout_and_preparation_failures_publish_nothing_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree =
            Fri06C02StatefulTextTree::new_shape_provider(Fri06C05ShapeProvider::Interval {
                minimum: S::ZERO,
                maximum: S::from_f64(8.0),
            });
        let cold = compute_layout(&tree, 0, request).expect("provider layout succeeds");
        cold.apply_to(&mut tree).expect("provider batch commits");

        tree.shape_provider = Fri06C05ShapeProvider::Failure;
        tree.retained.dirty = vec![1];
        let before_layout_failure = tree.retained.clone();
        let error = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect_err("provider failure returns no batch");
        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1,
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        assert_eq!(error.kind(), &LayoutErrorKindOf::Measurement(()));
        assert_eq!(tree.retained, before_layout_failure);
        assert_eq!(tree.retained.dirty, [1]);

        tree.shape_provider = Fri06C05ShapeProvider::Empty;
        let replacement = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("provider recovery stages a complete batch");
        assert_eq!(replacement.invalidated_nodes(), &[0, 1]);
        tree.reject_preparation = true;
        let before_preparation_failure = tree.retained.clone();
        assert_eq!(
            replacement.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, before_preparation_failure);
        assert_eq!(tree.retained.dirty, [1]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_lifecycle_mixed_cold_warm_rounding_dirty_replacement_scroll_and_failure_are_atomic_both_scalars()
 {
    fn fragment_identity<S: LayoutScalar>(
        entries: &[InlineFragmentOutputEntryOf<u32, S>],
    ) -> Vec<Fri06C03FragmentIdentity<S>> {
        entries
            .iter()
            .map(|entry| {
                let fragment = entry.fragment();
                (
                    entry.node(),
                    fragment.segment_id(),
                    fragment.line_index(),
                    fragment.visual_index(),
                    fragment.replacement_inline_extent(),
                )
            })
            .collect()
    }

    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new_mixed();
        let cold = compute_layout(&tree, 0, request).expect("cold mixed layout succeeds");
        assert_eq!(tree.fragment_readbacks.get(), 0);
        assert!(
            cold.cache_store_entries()
                .iter()
                .any(|entry| entry.node() == 1),
            "cold mixed layout must stage the text cache state used by the warm pass"
        );
        let cold_unrounded_identity = fragment_identity(cold.unrounded_inline_fragments());
        let cold_final_identity = fragment_identity(cold.final_inline_fragments());
        assert_eq!(
            cold_unrounded_identity,
            vec![(1, InlineSegmentId::new(91), 0, 0, None)]
        );
        assert_eq!(cold_final_identity, cold_unrounded_identity);
        assert_eq!(
            cold.unrounded_inline_fragments()
                .iter()
                .map(|entry| entry.fragment().baseline())
                .collect::<Vec<_>>(),
            vec![Point::new(S::ZERO, S::from_f64(8.0))],
            "the mixed source must retain its exact unrounded baseline"
        );
        assert_eq!(
            cold.final_inline_fragments()
                .iter()
                .map(|entry| entry.fragment().baseline())
                .collect::<Vec<_>>(),
            vec![Point::new(S::ZERO, S::from_f64(8.0))],
            "cumulative-origin rounding must publish the exact final baseline"
        );
        let cold_unrounded = cold.unrounded_entries().to_vec();
        let cold_final = cold.final_entries().to_vec();
        let cold_unrounded_fragments = cold.unrounded_inline_fragments().to_vec();
        let cold_final_fragments = cold.final_inline_fragments().to_vec();
        let unrounded_root = cold_unrounded
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        let unrounded_range = unrounded_root.scroll_geometry.unwrap().physical_range();
        assert_eq!(unrounded_range.x().maximum(), S::from_f64(5.25));
        assert_eq!(unrounded_range.y().maximum(), S::from_f64(0.5));
        let source_indices = |entries: &[LayoutOutputEntryOf<u32, S>]| {
            entries
                .iter()
                .filter(|entry| (1..=4).contains(&entry.node()))
                .map(|entry| (entry.node(), entry.output().source_index))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            source_indices(&cold_unrounded),
            vec![
                (1, SourceIndex::new(0)),
                (2, SourceIndex::new(1)),
                (3, SourceIndex::new(2)),
                (4, SourceIndex::new(3)),
            ]
        );
        assert_eq!(source_indices(&cold_final), source_indices(&cold_unrounded));
        cold.apply_to(&mut tree).expect("cold mixed batch commits");
        assert!(tree.retained.unrounded_nodes.contains_key(&1));
        assert!(tree.retained.final_nodes.contains_key(&1));
        assert!(
            tree.retained.caches.contains_key(&1),
            "cold commit must retain the text cache entry"
        );
        assert_eq!(
            tree.retained.unrounded_fragments[&1],
            cold_unrounded_fragments
                .iter()
                .map(InlineFragmentOutputEntryOf::fragment)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            tree.retained.final_fragments[&1],
            cold_final_fragments
                .iter()
                .map(InlineFragmentOutputEntryOf::fragment)
                .collect::<Vec<_>>()
        );

        let warm = compute_layout(&tree, 0, request).expect("warm mixed layout succeeds");
        assert_eq!(
            tree.fragment_readbacks.get(),
            1,
            "warm mixed layout must read the committed text fragment state"
        );
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 1),
            "the warm cached text node must avoid a cache store/recompute"
        );
        assert_eq!(warm.unrounded_entries(), cold_unrounded);
        assert_eq!(warm.final_entries(), cold_final);
        assert_eq!(warm.unrounded_inline_fragments(), cold_unrounded_fragments);
        assert_eq!(warm.final_inline_fragments(), cold_final_fragments);
        warm.apply_to(&mut tree)
            .expect("warm mixed batch recommits");

        let stale_atomic = tree.retained.unrounded_nodes[&2];
        tree.replace_atomic_inline_extent(27.75);
        tree.retained.dirty = vec![2, 2];
        let replacement = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("dirty atomic replacement stages");
        assert_eq!(replacement.invalidated_nodes(), &[0, 2]);
        assert_eq!(
            replacement.unrounded_inline_fragments(),
            cold_unrounded_fragments,
            "unaffected warm text fragments republish during exact atomic-path replacement"
        );
        replacement
            .apply_to(&mut tree)
            .expect("dirty atomic replacement commits");
        assert_ne!(tree.retained.unrounded_nodes[&2], stale_atomic);
        assert_eq!(
            tree.retained.unrounded_nodes[&2].size.width,
            S::from_f64(27.75)
        );

        tree.replace_atomic_inline_extent(29.25);
        tree.retained.dirty = vec![2];
        let rejected = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("second dirty atomic replacement stages");
        tree.reject_preparation = true;
        let retained_before_rejection = tree.retained.clone();
        assert_eq!(
            rejected.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, retained_before_rejection);
        assert_eq!(tree.retained.dirty, [2]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_lifecycle_unified_mixed_publication_contributes_each_geometry_once() {
    let block_source = include_str!("../block/inline_run.rs");
    let unified_path = block_source
        .split_once("fn layout_inline_run_children<")
        .expect("unified mixed layout function remains present")
        .1
        .split_once("\nfn record_inline_run_baselines<")
        .expect("unified mixed layout function keeps its narrow source boundary")
        .0;

    assert_eq!(
        unified_path
            .matches("layout_mixed_inline_run_with_band_source(")
            .count(),
        1
    );

    let fragment_projection = unified_path
        .split_once("for source in &report.fragments {")
        .expect("unified text-fragment projection remains present")
        .1
        .split_once("\n    if set_layout {\n        for (child, source_index, fragments")
        .expect("text-fragment contribution stays before text-node publication")
        .0;
    assert_eq!(
        fragment_projection
            .matches("fragments.push(InlineFragmentOutputOf::new(")
            .count(),
        1
    );
    assert_eq!(
        fragment_projection
            .matches("contributions.include_direct_line(rect);")
            .count(),
        1
    );

    let text_publication = unified_path
        .split_once(
            "for (child, source_index, fragments, union_min, union_max) in published_text {",
        )
        .expect("unified text-node publication remains present")
        .1
        .split_once("\n    let atomic_sources =")
        .expect("text-node publication stays before atomic projection")
        .0;
    assert_eq!(text_publication.matches("tree.set_unrounded(").count(), 1);
    assert_eq!(
        text_publication
            .matches("tree.set_unrounded_inline_fragment_state(")
            .count(),
        1
    );

    let atomic_publication = unified_path
        .split_once("for (child, source_index, child_style, output) in atomic_children {")
        .expect("unified atomic projection remains present")
        .1
        .split_once("\n    let control_sources =")
        .expect("atomic projection stays before control publication")
        .0;
    assert_eq!(
        atomic_publication
            .matches(".include_in_flow_geometry(location, source.item.margin, scroll_geometry)")
            .count(),
        1
    );
    assert_eq!(atomic_publication.matches("tree.set_unrounded(").count(), 1);

    let control_publication = unified_path
        .split_once("for (child, source_index) in control_children {")
        .expect("unified zero-geometry control publication remains present")
        .1
        .split_once("\n    let projected_baseline =")
        .expect("control publication stays before baseline projection")
        .0;
    assert_eq!(
        control_publication.matches("tree.set_unrounded(").count(),
        1
    );
    assert_eq!(control_publication.matches("contributions.").count(), 0);

    for deleted_path in [
        concat!("layout_shaped_text_", "children"),
        concat!("layout_vertical_inline_", "run"),
        concat!("layout_vertical_inline_", "lines"),
    ] {
        assert!(!block_source.contains(deleted_path));
    }
}

#[test]
fn fri06_c02_cache_cold_warm_and_dirty_replacement_use_committed_state_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new(vec![fri06_c02_segment(
            61,
            9.25,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);

        let cold = compute_layout(&tree, 0, request).expect("cold text layout succeeds");
        let cold_unrounded = cold.unrounded_entries().to_vec();
        let cold_final = cold.final_entries().to_vec();
        let cold_unrounded_fragments = cold.unrounded_inline_fragments().to_vec();
        let cold_final_fragments = cold.final_inline_fragments().to_vec();
        assert_eq!(tree.fragment_readbacks.get(), 0);
        cold.apply_to(&mut tree).expect("cold batch commits");
        assert!(tree.retained.dirty.is_empty());
        assert_eq!(tree.retained.unrounded_fragments[&1].len(), 1);
        assert!(!tree.retained.caches.is_empty());

        let warm = compute_layout(&tree, 0, request).expect("warm text layout restores fragments");
        assert_eq!(warm.unrounded_entries(), cold_unrounded);
        assert_eq!(warm.final_entries(), cold_final);
        assert_eq!(warm.unrounded_inline_fragments(), cold_unrounded_fragments);
        assert_eq!(warm.final_inline_fragments(), cold_final_fragments);
        assert_eq!(tree.fragment_readbacks.get(), 1);
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 1),
            "the committed text cache must serve the warm text node"
        );
        warm.apply_to(&mut tree).expect("warm batch recommits");

        let stale_text_output = tree.retained.unrounded_nodes[&1];
        let stale_fragments = tree.retained.unrounded_fragments[&1].clone();
        let stale_caches = tree.retained.caches.clone();
        tree.replace_text(vec![fri06_c02_segment(
            62,
            13.75,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);
        tree.retained.dirty = vec![1, 1];

        let invalidated = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("dirty text layout succeeds");
        assert_eq!(invalidated.invalidated_nodes(), &[0, 1]);
        assert_eq!(tree.retained.dirty, [1, 1]);
        invalidated
            .apply_to(&mut tree)
            .expect("dirty replacement batch commits");

        assert!(tree.retained.dirty.is_empty());
        assert_ne!(tree.retained.unrounded_nodes[&1], stale_text_output);
        assert_ne!(tree.retained.unrounded_fragments[&1], stale_fragments);
        assert_ne!(tree.retained.caches, stale_caches);
        assert_eq!(
            tree.retained.unrounded_fragments[&1][0].segment_id(),
            InlineSegmentId::new(62)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_cache_committed_empty_fragment_state_replays_warm_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new(vec![fri06_c02_segment(
            63,
            6.5,
            InlineWhitespaceEdge::DiscardAtBoth,
            InlineBreakOpportunityOf::prohibited(),
        )]);

        let cold = compute_layout(&tree, 0, request).expect("cold discarded text layout succeeds");
        assert!(cold.unrounded_inline_fragments().is_empty());
        cold.apply_to(&mut tree)
            .expect("committed empty fragment state is retained");
        assert_eq!(tree.unrounded_inline_fragments(1), Some([].as_slice()));
        tree.fragment_readbacks.set(0);
        assert!(
            tree.retained.caches.contains_key(&1),
            "the committed empty fragment state must pair with a warm text cache"
        );

        let warm = compute_layout(&tree, 0, request)
            .expect("Some(&[]) is valid warm committed fragment state");
        assert!(warm.unrounded_inline_fragments().is_empty());
        assert!(warm.final_inline_fragments().is_empty());
        assert_eq!(tree.fragment_readbacks.get(), 1);
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 1)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_transaction_layout_and_preparation_failures_preserve_retained_state_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new(vec![fri06_c02_segment(
            81,
            7.5,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);
        let cold = compute_layout(&tree, 0, request).expect("cold text layout succeeds");
        cold.apply_to(&mut tree).expect("cold batch commits");
        assert!(
            tree.retained.caches.contains_key(&1),
            "transaction proof requires committed text cache state"
        );

        tree.replace_text(vec![fri06_c02_segment(
            82,
            8.5,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        )]);
        tree.retained.dirty = vec![1];
        tree.add_failing_noncanonical_control();
        let before_layout_failure = tree.retained.clone();

        let error = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect_err("noncanonical control pairing fails before retained-state mutation");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::NonBoxNodeRole {
                reason: NonBoxNodeRoleError::NonCanonicalNodeInput,
            })
        );
        assert_eq!(tree.retained, before_layout_failure);

        tree.remove_failing_noncanonical_control();
        let replacement = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("dirty text replacement stages successfully");
        tree.reject_preparation = true;
        let before_preparation_failure = tree.retained.clone();
        assert_eq!(
            replacement.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, before_preparation_failure);
        assert_eq!(tree.retained.dirty, [1]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn flex_item_root_uses_explicit_parent_axes_for_percentage_and_cache_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let parent_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let item_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
        let viewport = Size::new(
            AvailableOf::definite(scalar::<S>(200.0)),
            AvailableOf::definite(scalar::<S>(80.0)),
        );
        let available = Size::new(
            AvailableOf::definite(scalar::<S>(140.0)),
            AvailableOf::definite(scalar::<S>(300.0)),
        );
        let root_style = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            direction: Direction::Rtl,
            padding: Edges::new(
                LengthOf::percent(scalar::<S>(0.0625)),
                LengthOf::percent(scalar::<S>(0.125)),
                LengthOf::percent(scalar::<S>(0.25)),
                LengthOf::percent(scalar::<S>(0.5)),
            ),
            ..NodeInputOf::<S>::default()
        };
        let child_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(scalar::<S>(10.0)),
                PreferredSizeOf::px(scalar::<S>(20.0)),
            ),
            ..NodeInputOf::<S>::default()
        };
        let tree = PublicFlowTree::default()
            .with_children(0, [])
            .with_style(0, root_style.clone());
        let request = LayoutRootRequestOf::flex_item_under_viewport(
            available,
            FlexItemRootContextOf::under_viewport(viewport, parent_axes)
                .expect("finite flex-item root context"),
        )
        .expect("finite flex-item root request");

        let cold = compute_layout(&tree, 0, request).expect("cold flex-item root layout");
        let root_entry = cold
            .cache_store_entries()
            .iter()
            .find(|entry| {
                entry.node() == 0 && entry.input().run_mode() == RunMode::PerformRootLayout
            })
            .expect("cold root compute is cached");
        let root_input = *root_entry.input();
        let root_output = root_entry.output();
        let descendant_tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, root_style)
            .with_style(1, child_style);
        let descendant_batch =
            compute_layout(&descendant_tree, 0, request).expect("root with descendant layout");
        let child_input = descendant_batch
            .cache_store_entries()
            .iter()
            .find(|entry| entry.node() == 1 && entry.input().run_mode() == RunMode::PerformLayout)
            .expect("child layout compute is cached")
            .input();

        assert_eq!(
            root_input.containing_layout_context(),
            ContainingLayoutContext::new(parent_axes, ParentFormattingContext::Flex)
        );
        assert_eq!(
            root_input.known(),
            Size::new(Some(scalar::<S>(140.0)), None)
        );
        assert_eq!(
            child_input.containing_layout_context(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::BlockFlow)
        );

        let root = public_flow_output(cold.unrounded_entries(), 0);
        let expected_padding = Edges::new(
            scalar::<S>(12.5),
            scalar::<S>(25.0),
            scalar::<S>(50.0),
            scalar::<S>(100.0),
        );
        assert_eq!(root.padding, expected_padding);
        let logical_padding = parent_axes.logical_edges(root.padding);
        assert_eq!(logical_padding.inline_start, scalar::<S>(100.0));
        assert_eq!(logical_padding.inline_end, scalar::<S>(25.0));
        assert_eq!(logical_padding.block_start, scalar::<S>(12.5));
        assert_eq!(logical_padding.block_end, scalar::<S>(50.0));

        let cache_context = CacheKeyContext::new();
        let mut cache = CacheOf::<S>::new();
        cache.store_with_context(&root_input, cache_context, root_output);
        assert_eq!(
            cache.get_with_context(&root_input, cache_context),
            Some(root_output)
        );
        let role_only = ComputeInputOf::flex_item_root(
            root_input.known(),
            root_input.parent(),
            ContainingLayoutContext::new(parent_axes, ParentFormattingContext::NoParent),
            root_input.available(),
        );
        assert_eq!(cache.get_with_context(&role_only, cache_context), None);
        let axes_only = ComputeInputOf::flex_item_root(
            root_input.known(),
            root_input.parent(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::Flex),
            root_input.available(),
        );
        assert_eq!(cache.get_with_context(&axes_only, cache_context), None);

        tree.apply_cache_entries(cold.cache_store_entries());
        tree.clear_cache_inputs();
        let warm = compute_layout(&tree, 0, request).expect("warm flex-item root layout");
        assert_eq!(
            public_flow_output(warm.unrounded_entries(), 0),
            public_flow_output(cold.unrounded_entries(), 0)
        );
        assert_eq!(
            public_flow_output(warm.final_entries(), 0),
            public_flow_output(cold.final_entries(), 0)
        );
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 0),
            "the identical root context should hit the applied cold cache"
        );

        let viewport_tree = PublicFlowTree::default().with_children(0, []).with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                ..NodeInputOf::<S>::default()
            },
        );
        let viewport_batch = compute_layout(
            &viewport_tree,
            0,
            LayoutRootRequestOf::viewport(available).expect("finite viewport request"),
        )
        .expect("viewport layout");
        let viewport_input = viewport_batch
            .cache_store_entries()
            .iter()
            .find(|entry| {
                entry.node() == 0 && entry.input().run_mode() == RunMode::PerformRootLayout
            })
            .expect("viewport root compute is cached")
            .input();
        assert_eq!(
            viewport_input.containing_layout_context(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::NoParent)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn root_cache_input(available: Size<Available>) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformRootLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        available.map(Available::into_option),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        available,
    )
}

fn assert_fri06_mr02_geometry_error_public_root_has_no_batch<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    for display in [
        Display::Block,
        Display::Flex,
        Display::Grid,
        Display::GridLanes,
    ] {
        let style = NodeInputOf {
            display,
            size: Size::new(PreferredSizeOf::px(largest), PreferredSizeOf::px(S::ONE)),
            padding: Edges {
                left: LengthOf::px(largest),
                ..Edges::all(LengthOf::ZERO)
            },
            border: Edges {
                left: LengthOf::px(largest),
                ..Edges::all(LengthOf::ZERO)
            },
            grid_template_columns: vec![TrackComponentOf::AUTO],
            ..NodeInputOf::default()
        };
        let tree = public_layout_tree(
            HashMap::from([(0, LayoutInputOf::box_input(style))]),
            HashMap::from([(0, Vec::new())]),
        );
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(largest),
            AvailableOf::definite(S::ONE),
        ))
        .unwrap();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compute_layout(&tree, 0, request)
        }));
        let error = match outcome {
            Ok(Err(error)) => error,
            Ok(Ok(_)) => panic!("{display:?} overflow must publish no completed batch"),
            Err(_) => panic!("{display:?} overflow must not unwind"),
        };

        assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidRootScrollGeometry
            )
        ));
    }
}

#[test]
fn fri06_mr02_geometry_error_public_root_preserves_no_publication_both_scalars() {
    assert_fri06_mr02_geometry_error_public_root_has_no_batch::<f32>();
    assert_fri06_mr02_geometry_error_public_root_has_no_batch::<f64>();
}

#[test]
fn compute_layout_success_returns_completed_batch_without_tree_mutation() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.25), PreferredSize::px(20.5)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("root layout succeeds");

    assert_eq!(batch.unrounded_entries().len(), 1);
    assert_eq!(batch.unrounded_entries()[0].node(), 0);
    assert_eq!(
        batch.unrounded_entries()[0].output().size,
        Size::new(10.25, 20.5)
    );
    assert_eq!(batch.final_entries().len(), 1);
    assert_eq!(batch.final_entries()[0].node(), 0);
    assert_eq!(
        batch.final_entries()[0].output().size,
        Size::new(10.0, 21.0)
    );
}

#[test]
fn compute_layout_stages_cache_store_with_the_cold_root_output() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request).expect("cold root layout succeeds");

    assert_eq!(batch.cache_store_entries().len(), 1);
    let entry = &batch.cache_store_entries()[0];
    assert_eq!(entry.node(), 0);
    assert_eq!(entry.output().size, Size::new(10.0, 20.0));
    let mut applied_cache = Cache::new();
    applied_cache.store_with_context(entry.input(), entry.context(), entry.output());
    assert_eq!(
        applied_cache.get_with_context(entry.input(), entry.context()),
        Some(entry.output())
    );
}

#[test]
fn fri05_c05_grid_geometry_root_front_door_stages_only_canonical_ordinary_output() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(90.0)],
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("ordinary grid root layout succeeds");
    let unrounded = batch.unrounded_entries()[0].output();
    let geometry = unrounded
        .scroll_geometry
        .expect("ordinary grid root publishes canonical geometry");
    assert_eq!(geometry.used_overflow_x(), Overflow::Scroll);
    assert_eq!(geometry.used_overflow_y(), Overflow::Scroll);
    assert_eq!(geometry.target().border_box(), geometry.border_box());
    assert_eq!(unrounded.content_box_size(), geometry.content_box().size());
    assert_eq!(batch.cache_store_entries().len(), 1);
    assert_eq!(
        batch.cache_store_entries()[0]
            .output()
            .scroll_geometry
            .expect("stable root cache entry retains geometry"),
        geometry
    );
    assert!(batch.final_entries()[0].output().scroll_geometry.is_some());
}

#[test]
fn compute_layout_uses_a_matching_root_cache_hit_without_staging_a_store() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let cached = ComputeOutput::from_outer_size(Size::new(33.0, 44.0));
    let mut cache = Cache::new();
    cache.store_with_context(&input, CacheKeyContext::new(), cached);
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request).expect("cached root layout succeeds");

    assert_eq!(
        batch.unrounded_entries()[0].output().size,
        Size::new(33.0, 44.0)
    );
    assert!(batch.cache_store_entries().is_empty());
}

#[test]
fn compute_layout_root_diagnostics_reject_invalid_cached_scroll_geometry_without_batch() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(f32::NAN, 20.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let error = compute_layout(&tree, 0, request)
        .expect_err("invalid cached root output must not complete a layout batch");

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry)
    );
}

#[test]
fn compute_layout_ignores_cached_container_output_until_the_subtree_is_complete() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Ok(Size::new(12.0, 8.0)));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let cached = ComputeOutput::from_outer_size(Size::new(33.0, 44.0));
    let mut cache = Cache::new();
    cache.store_with_context(&input, CacheKeyContext::new(), cached);
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request)
        .expect("a cached container request must return a complete layout batch");

    for node in [0, 1] {
        assert!(
            batch
                .unrounded_entries()
                .iter()
                .any(|entry| entry.node() == node)
        );
        assert!(
            batch
                .final_entries()
                .iter()
                .any(|entry| entry.node() == node)
        );
    }
    assert_ne!(
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .expect("root output must be staged")
            .output()
            .size,
        cached.size
    );
    let measured_nodes = tree.measured_nodes();
    assert!(!measured_nodes.is_empty());
    assert!(measured_nodes.iter().all(|node| *node == 1));
}

#[test]
fn compute_layout_cached_container_failure_returns_no_batch() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Err("measure failed"));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(33.0, 44.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(available).unwrap();

    let error = compute_layout(&tree, 0, request)
        .expect_err("a cached container must not hide a descendant provider failure");

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::Measurement("measure failed")
    );
    assert_eq!(tree.measured_nodes(), vec![1]);
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_stages_hidden_root_cache_clear_without_a_store() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("hidden root layout succeeds");

    assert!(batch.cache_store_entries().is_empty());
    assert_eq!(batch.cache_clear_entries().len(), 1);
    assert_eq!(batch.cache_clear_entries()[0].node(), 0);
}

#[test]
fn compute_layout_failure_drops_staged_cache_effects_without_mutating_tree_cache() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Err("measure failed"));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(7.0, 9.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(available).unwrap();

    let result = compute_layout(&tree, 0, request);

    assert!(result.is_err());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_preserves_nested_subgrid_resolution_failure() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::from(
                    LengthPercentageOf::px(20.0).expect("finite track"),
                )],
                grid_template_rows: vec![TrackComponent::from(
                    LengthPercentageOf::px(20.0).expect("finite track"),
                )],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponent::from(overflowing)],
                size: Size::new(PreferredSize::AUTO, PreferredSize::px(f32::MAX)),
                ..NodeInput::default()
            },
        );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(20.0),
        Available::definite(20.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
    ));
}

fn fri05_c04_flex_auto_public_tree(nested: bool, child_size: Size<f32>) -> PublicFlowTree<f32> {
    let container = NodeInput {
        display: Display::Flex,
        overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
        scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        align_items: Some(AlignItems::FlexStart),
        ..NodeInput::default()
    };
    let absolute = NodeInput {
        display: Display::Block,
        position: Position::Absolute,
        size: child_size.map(PreferredSize::px),
        inset: Edges::new(
            LengthAuto::px(0.0),
            LengthAuto::AUTO,
            LengthAuto::AUTO,
            LengthAuto::px(0.0),
        ),
        ..NodeInput::default()
    };

    if nested {
        PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [2])
            .with_children(2, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(1, container)
            .with_style(2, absolute)
    } else {
        PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, container)
            .with_style(1, absolute)
    }
}

#[test]
fn fri05_c04_flex_auto_nested_nonoverflow_keeps_local_initial_and_outer_cache_passes() {
    let tree = fri05_c04_nested_flex_auto_tree(false);
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    let cold = compute_layout(&tree, 0, request).expect("cold nested flex layout succeeds");
    let outer = public_flow_output(cold.unrounded_entries(), 0);
    let inner = public_flow_output(cold.unrounded_entries(), 1);
    assert_eq!(
        outer.scroll_geometry.unwrap().scrollbar_size(),
        Size::new(0.0, 15.0)
    );
    assert_eq!(inner.scroll_geometry.unwrap().scrollbar_size(), Size::ZERO);

    let grandchild_requests = tree.cache_inputs(2);
    fri05_c04_assert_initial_local_auto_state(&grandchild_requests);
    assert!(
        grandchild_requests
            .iter()
            .all(|input| fri05_c03_block_root_state(input) == (false, false))
    );
    for node in [0, 1, 2, 3] {
        assert_eq!(
            cold.unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only stable cold output is published for node {node}"
        );
    }

    tree.apply_cache_entries(cold.cache_store_entries());
    tree.clear_cache_inputs();
    let warm = compute_layout(&tree, 0, request).expect("warm nested flex layout succeeds");
    for node in [0, 1, 2, 3] {
        assert_eq!(
            public_flow_output(warm.unrounded_entries(), node),
            public_flow_output(cold.unrounded_entries(), node),
            "warm output remains the stable cold output for node {node}"
        );
        assert_eq!(
            warm.unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only stable warm output is published for node {node}"
        );
    }
}

#[test]
fn fri05_c04_flex_auto_root_and_nested_publish_stable_output_with_exact_pass_cache_bits() {
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    for nested in [false, true] {
        for (child_size, expected_states, expected_scrollbars) in [
            (Size::new(80.0, 80.0), vec![(false, false)], Size::ZERO),
            (
                Size::new(120.0, 80.0),
                vec![(false, false), (true, false)],
                Size::new(0.0, 15.0),
            ),
            (
                Size::new(80.0, 120.0),
                vec![(false, false), (false, true)],
                Size::new(15.0, 0.0),
            ),
            (
                Size::new(120.0, 100.0),
                vec![(false, false), (true, false), (true, true)],
                Size::splat(15.0),
            ),
            (
                Size::new(100.0, 120.0),
                vec![(false, false), (false, true), (true, true)],
                Size::splat(15.0),
            ),
        ] {
            let tree = fri05_c04_flex_auto_public_tree(nested, child_size);
            let container = u32::from(nested);
            let absolute = container + 1;
            let cold = compute_layout(&tree, 0, request).expect("cold flex auto layout succeeds");
            let output = public_flow_output(cold.unrounded_entries(), container);
            assert_eq!(
                output.scroll_geometry.unwrap().scrollbar_size(),
                expected_scrollbars,
                "nested={nested}, child={child_size:?}"
            );
            for node in [container, absolute] {
                assert_eq!(
                    cold.unrounded_entries()
                        .iter()
                        .filter(|entry| entry.node() == node)
                        .count(),
                    1,
                    "only stable node {node} output is published for nested={nested}"
                );
            }

            let cache_inputs = tree.cache_inputs(absolute);
            assert!(
                cache_inputs
                    .iter()
                    .all(|input| fri05_c04_local_auto_state(input) == (false, false)),
                "nested={nested}, child={child_size:?}: child-local state must start at INITIAL"
            );
            assert_eq!(
                cache_inputs
                    .iter()
                    .map(fri05_c03_block_root_state)
                    .collect::<Vec<_>>(),
                expected_states,
                "nested={nested}, child={child_size:?}: {cache_inputs:#?}"
            );
            assert_eq!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| {
                        entry.node() == absolute
                            && entry.input().run_mode() == RunMode::PerformLayout
                    })
                    .map(|entry| fri05_c03_block_root_state(entry.input()))
                    .collect::<Vec<_>>(),
                expected_states,
                "nested={nested}, child={child_size:?}"
            );
            assert!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| entry.node() == absolute)
                    .all(|entry| fri05_c04_local_auto_state(entry.input()) == (false, false)),
                "nested={nested}, child={child_size:?}: cached child-local state stays INITIAL"
            );
            let child_cache_inputs = cold
                .cache_store_entries()
                .iter()
                .filter(|entry| {
                    entry.node() == absolute && entry.input().run_mode() == RunMode::PerformLayout
                })
                .map(LayoutCacheStoreEntryOf::input)
                .collect::<Vec<_>>();
            assert!(
                child_cache_inputs
                    .iter()
                    .all(|input| input.known() == child_cache_inputs[0].known()),
                "distinct containing passes stage separate entries for identical known child geometry"
            );
            assert!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| entry.node() == container)
                    .all(|entry| fri05_c03_block_root_state(entry.input()) == (false, false)),
                "no speculative container pass is cached under an ordinary request"
            );

            tree.apply_cache_entries(cold.cache_store_entries());
            tree.clear_cache_inputs();
            let warm = compute_layout(&tree, 0, request).expect("warm flex auto layout succeeds");
            assert_eq!(
                public_flow_output(warm.unrounded_entries(), container),
                public_flow_output(cold.unrounded_entries(), container),
                "nested={nested}, child={child_size:?}"
            );
            assert_eq!(
                public_flow_output(warm.final_entries(), container),
                public_flow_output(cold.final_entries(), container),
                "nested={nested}, child={child_size:?}"
            );
            for node in [container, absolute] {
                assert_eq!(
                    warm.unrounded_entries()
                        .iter()
                        .filter(|entry| entry.node() == node)
                        .count(),
                    1,
                    "only stable warm output is published for node {node}"
                );
            }
        }
    }
}
