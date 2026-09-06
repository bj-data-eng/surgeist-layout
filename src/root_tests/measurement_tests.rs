use super::fixtures::{
    FlowRootLeafTree, Fri06C02StatefulTextTree, Fri06C05ShapeProvider, RootSessionTree,
    computed_overflow, fri05_c03_tree_leaf_layout, fri06_atomic_participation,
    fri06_c02_stateful_request, fri06_mr02_geometry_error_largest_finite, scalar,
    single_final_output,
};
use super::*;

#[test]
fn fri06_c05_provider_cache_warm_valid_output_reuses_provider_result_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree =
            Fri06C02StatefulTextTree::new_shape_provider(Fri06C05ShapeProvider::Interval {
                minimum: S::ZERO,
                maximum: S::from_f64(8.0),
            });

        let cold = compute_layout(&tree, 0, request).expect("cold provider layout succeeds");
        let cold_query_count = tree.shape_queries.get();
        assert_eq!(
            cold_query_count, 1,
            "intrinsic computation and rounding must not add provider queries"
        );
        cold.apply_to(&mut tree)
            .expect("cold provider batch commits");

        let warm = compute_layout(&tree, 0, request).expect("warm provider layout succeeds");

        assert_eq!(warm.unrounded_entries(), cold.unrounded_entries());
        assert_eq!(warm.final_entries(), cold.final_entries());
        assert_eq!(
            warm.unrounded_inline_fragments(),
            cold.unrounded_inline_fragments()
        );
        assert_eq!(warm.final_inline_fragments(), cold.final_inline_fragments());
        assert_eq!(
            tree.shape_queries.get(),
            cold_query_count,
            "a valid warm output must not rerun the provider"
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_dirty_exact_closure_bypasses_stale_hits_and_recomputes_transitions_both_scalars()
 {
    fn source_order<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<(u32, usize)> {
        batch
            .final_entries()
            .iter()
            .map(|entry| (entry.node(), entry.output().source_index.get()))
            .collect()
    }

    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new_shape_provider(Fri06C05ShapeProvider::Empty);
        let cold = compute_layout(&tree, 0, request).expect("empty provider layout succeeds");
        let cold_final = cold.final_entries().to_vec();
        let cold_fragments = cold.final_inline_fragments().to_vec();
        let expected_source_order = source_order(&cold);
        cold.apply_to(&mut tree)
            .expect("empty provider batch commits");

        let warm_queries = tree.shape_queries.get();
        let warm = compute_layout(&tree, 0, request).expect("empty warm layout succeeds");
        assert_eq!(tree.shape_queries.get(), warm_queries);
        assert_eq!(warm.final_entries(), cold_final);
        assert_eq!(warm.final_inline_fragments(), cold_fragments);

        tree.shape_provider = Fri06C05ShapeProvider::Interval {
            minimum: S::ZERO,
            maximum: S::from_f64(8.0),
        };
        tree.retained.dirty = vec![1, 1];
        tree.cache_queries.borrow_mut().clear();
        let partial_query_start = tree.shape_queries.get();
        let partial = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("dirty partial provider layout succeeds");
        assert_eq!(partial.invalidated_nodes(), &[0, 1]);
        assert_eq!(tree.retained.dirty, [1, 1]);
        assert_eq!(tree.shape_queries.get() - partial_query_start, 1);
        assert!(
            tree.cache_queries
                .borrow()
                .iter()
                .all(|(node, _)| ![0, 1].contains(node)),
            "the exact closure must bypass stale root and float lookups"
        );
        assert_ne!(partial.final_entries(), cold_final);
        assert_eq!(source_order(&partial), expected_source_order);
        partial
            .apply_to(&mut tree)
            .expect("partial provider replacement commits");
        assert!(tree.retained.dirty.is_empty());

        tree.shape_provider = Fri06C05ShapeProvider::Interval {
            minimum: S::ZERO,
            maximum: S::from_f64(15.25),
        };
        tree.retained.dirty = vec![1];
        tree.cache_queries.borrow_mut().clear();
        let full_query_start = tree.shape_queries.get();
        let full = compute_layout_invalidated(&tree, 0, request, &tree.retained.dirty)
            .expect("dirty full provider layout succeeds");
        assert_eq!(full.invalidated_nodes(), &[0, 1]);
        assert_eq!(tree.shape_queries.get() - full_query_start, 1);
        assert_ne!(full.final_entries(), partial.final_entries());
        assert_eq!(source_order(&full), expected_source_order);
        assert_eq!(
            full.final_inline_fragments()
                .iter()
                .map(|entry| (entry.node(), entry.fragment().segment_id()))
                .collect::<Vec<_>>(),
            cold_fragments
                .iter()
                .map(|entry| (entry.node(), entry.fragment().segment_id()))
                .collect::<Vec<_>>(),
            "provider geometry changes must preserve text source association"
        );
        full.apply_to(&mut tree)
            .expect("full provider replacement commits");
        assert!(tree.retained.dirty.is_empty());
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn replaced_viewport_and_flex_item_roots_keep_measured_auto_inline_size_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar::<S>;
        let available = Size::new(
            AvailableOf::definite(scalar(200.0)),
            AvailableOf::MAX_CONTENT,
        );
        let parent_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);

        for replaced in [true, false] {
            let style = NodeInputOf {
                display: Display::Block,
                item_is_replaced: replaced,
                ..NodeInputOf::default()
            };
            let viewport_tree = FlowRootLeafTree::new(style.clone())
                .with_natural_size(Size::new(scalar(50.0), scalar(10.0)));
            let viewport = compute_layout(
                &viewport_tree,
                0,
                LayoutRootRequestOf::viewport(available).expect("finite viewport request"),
            )
            .expect("measured viewport root lays out");
            assert_eq!(
                single_final_output(&viewport).size.width,
                scalar(if replaced { 50.0 } else { 200.0 })
            );

            let flex_tree = FlowRootLeafTree::new(style)
                .with_natural_size(Size::new(scalar(50.0), scalar(10.0)));
            let flex = compute_layout(
                &flex_tree,
                0,
                LayoutRootRequestOf::flex_item_under_viewport(
                    available,
                    FlexItemRootContextOf::under_viewport(available, parent_axes)
                        .expect("finite flex-item context"),
                )
                .expect("finite flex-item request"),
            )
            .expect("measured flex-item root lays out");
            assert_eq!(
                single_final_output(&flex).size.width,
                scalar(if replaced { 50.0 } else { 200.0 })
            );
            assert!(flex.cache_store_entries().iter().any(|entry| {
                entry.input().containing_layout_context()
                    == ContainingLayoutContext::new(parent_axes, ParentFormattingContext::Flex)
            }));
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

struct ConstraintOverflowTree<S: LayoutScalar> {
    style: NodeInputOf<S>,
    measure_calls: Cell<usize>,
}

impl<S: LayoutScalar> Traverse for ConstraintOverflowTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Empty<u32>
    where
        Self: 'a;

    fn children(&self, _node: Self::Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Self::Node) -> usize {
        0
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        unreachable!("constraint overflow test tree has no children")
    }
}

impl<S: LayoutScalar> LayoutTree for ConstraintOverflowTree<S> {
    type MeasureError = ();

    fn node_input(&self, _node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        &self.style
    }

    fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.style.clone())
    }

    fn has_leaf_measurement(&self, _node: Self::Node) -> bool {
        true
    }

    fn measure_leaf(
        &self,
        _node: Self::Node,
        _input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.measure_calls.set(self.measure_calls.get() + 1);
        Some(Ok(Size::ZERO))
    }
}

fn assert_tree_leaf_constraint_overflow<S: LayoutScalar>(largest_finite: S) {
    let tree = ConstraintOverflowTree {
        style: NodeInputOf {
            padding: Edges::all(LengthOf::px(largest_finite)),
            ..NodeInputOf::default()
        },
        measure_calls: Cell::new(0),
    };
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(largest_finite)))
        .expect("largest finite root availability is valid");

    let error = compute_layout(&tree, 0, request)
        .expect_err("overflowing resolved padding must return no completed batch");

    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
            if *value == S::INFINITY
    ));
    assert_eq!(tree.measure_calls.get(), 0);
}

#[test]
fn f32_tree_leaf_constraint_overflow_returns_typed_error_before_measurement() {
    assert_tree_leaf_constraint_overflow(f32::MAX);
}

#[test]
fn f64_tree_leaf_constraint_overflow_returns_typed_error_before_measurement() {
    assert_tree_leaf_constraint_overflow(f64::MAX);
}

#[test]
fn compute_layout_provider_error_returns_no_completed_batch() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Err("measure failed"));
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::Measurement("measure failed")
    );
}

#[test]
fn compute_layout_rejects_claimed_leaf_without_provider() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .leaf_without_provider(1);
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InternalInvariant(
            LayoutInternalInvariant::MissingLeafMeasurementProvider
        )
    );
    assert_eq!(tree.measured_nodes(), vec![1]);
}

#[test]
fn compute_layout_rejects_invalid_provider_output_without_batch() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Ok(Size::new(f32::NAN, 10.0)));
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let result = compute_layout(&tree, 0, request);
    let error = match result {
        Ok(_) => panic!("invalid provider output must not complete a layout batch"),
        Err(error) => error,
    };

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(output))
            if output.axis() == PhysicalAxis::Horizontal
    ));
    let LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(output)) = error.kind()
    else {
        panic!("invalid provider output must retain its measurement diagnostic");
    };
    let NonNegativeFiniteScalarErrorOf::NonFinite { value } = output.error() else {
        panic!("invalid provider output must retain the rejected scalar");
    };
    assert!(value.is_nan());
}

#[test]
fn compute_layout_rejects_measured_child_invalid_affine_width_without_batch() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::value(overflowing), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .measure(1, Ok(Size::new(12.0, 8.0)));
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        })
    );
    assert!(tree.measured_nodes().is_empty());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_rejects_measured_child_invalid_affine_padding_without_batch() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                padding: Edges::all(Length::value(overflowing)),
                ..NodeInput::default()
            },
        )
        .measure(1, Ok(Size::new(12.0, 8.0)));
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        })
    );
    assert!(tree.measured_nodes().is_empty());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_rejects_root_measured_leaf_invalid_affine_width_without_batch() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [])
        .style(
            0,
            NodeInput {
                size: Size::new(PreferredSize::value(overflowing), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .measure(0, Ok(Size::new(12.0, 8.0)));
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        })
    );
    assert!(tree.measured_nodes().is_empty());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_rejects_root_measured_leaf_invalid_affine_padding_without_batch() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [])
        .style(
            0,
            NodeInput {
                padding: Edges::all(Length::value(overflowing)),
                ..NodeInput::default()
            },
        )
        .measure(0, Ok(Size::new(12.0, 8.0)));
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        })
    );
    assert!(tree.measured_nodes().is_empty());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn fri04_c03_leaf_root_root_front_door_consumes_leaf_and_inner_display_calculations() {
    fn calculation(value: f32) -> SizingCalculation {
        SizingCalculation::value(LengthPercentageOf::px(value).expect("finite sizing value"))
    }

    fn style(display: Display) -> NodeInput {
        NodeInput {
            display,
            size: Size::new(
                PreferredSize::calculation(
                    SizingCalculation::max(vec![calculation(60.0), calculation(45.0)])
                        .expect("nonempty maximum"),
                ),
                PreferredSize::calculation(SizingCalculation::clamp(
                    Some(calculation(20.0)),
                    calculation(40.0),
                    Some(calculation(70.0)),
                )),
            ),
            min_size: Size::new(
                MinSize::calculation(
                    SizingCalculation::min(vec![calculation(-8.0), calculation(-3.0)])
                        .expect("nonempty minimum"),
                ),
                MinSize::calculation(
                    SizingCalculation::max(vec![calculation(10.0), calculation(15.0)])
                        .expect("nonempty maximum"),
                ),
            ),
            max_size: Size::new(
                MaxSize::calculation(SizingCalculation::clamp(
                    None,
                    calculation(55.0),
                    Some(calculation(90.0)),
                )),
                MaxSize::calculation(
                    SizingCalculation::max(vec![calculation(45.0), calculation(35.0)])
                        .expect("nonempty maximum"),
                ),
            ),
            ..NodeInput::default()
        }
    }

    let request = || {
        LayoutRootRequest::viewport(Size::new(
            Available::definite(100.0),
            Available::definite(80.0),
        ))
        .expect("valid root request")
    };
    let leaf: RootSessionTree = RootSessionTree::default()
        .children(0, [])
        .style(0, style(Display::Block))
        .measure(0, Ok(Size::new(1.0, 1.0)));
    let leaf_batch = compute_layout(&leaf, 0, request()).expect("root leaf layout succeeds");
    assert_eq!(
        leaf_batch.unrounded_entries()[0].output().size,
        Size::new(55.0, 40.0)
    );

    let inner: RootSessionTree = RootSessionTree::default()
        .children(0, [])
        .style(0, style(Display::Block));
    let inner_batch = compute_layout(&inner, 0, request()).expect("root block layout succeeds");
    assert_eq!(
        inner_batch.unrounded_entries()[0].output().size,
        Size::new(55.0, 40.0)
    );
}

#[test]
fn fri04_c04_leaf_block_positioned_root_reports_actual_leaf_or_block_algorithm() {
    let style = || NodeInput {
        display: Display::Block,
        size: Size::new(PreferredSize::AUTO, PreferredSize::STRETCH),
        ..NodeInput::default()
    };
    let request = || {
        LayoutRootRequest::viewport(Size::splat(Available::definite(100.0)))
            .expect("valid viewport")
    };
    let cases: [(RootSessionTree<&'static str>, SizingAlgorithm); 2] = [
        (
            RootSessionTree::default()
                .children(0, [])
                .style(0, style())
                .measure(0, Ok(Size::new(10.0, 10.0))),
            SizingAlgorithm::Leaf,
        ),
        (
            RootSessionTree::default().children(0, []).style(0, style()),
            SizingAlgorithm::Block,
        ),
    ];

    for (tree, expected_algorithm) in cases {
        let error = compute_layout(&tree, 0, request())
            .expect_err("later-owned root sizing must be rejected");
        assert_eq!(error.site(), LayoutErrorSite::Node(0));
        let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
            unsupported,
        )) = error.kind()
        else {
            panic!("expected exact root sizing capability");
        };
        assert_eq!(unsupported.property(), SizingProperty::Preferred);
        assert_eq!(unsupported.behavior(), SizingBehavior::Stretch);
        assert_eq!(unsupported.algorithm(), expected_algorithm);
        assert_eq!(unsupported.axis(), PhysicalAxis::Vertical);
    }
}

#[test]
fn fri04_c04_leaf_block_positioned_root_leaf_and_block_supported_geometry() {
    let calc_size = || {
        Size::new(
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::Any,
                CalcSizeCalculation::from_coefficients(20.0, 0.5, 0.0)
                    .expect("finite Any calculation"),
            )
            .expect("valid Any calc-size"),
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::FullPercentage,
                CalcSizeCalculation::from_coefficients(10.0, 0.0, 0.5)
                    .expect("finite FullPercentage calculation"),
            )
            .expect("valid FullPercentage calc-size"),
        )
    };
    let request = || {
        LayoutRootRequest::viewport(Size::new(
            Available::definite(200.0),
            Available::definite(160.0),
        ))
        .expect("valid viewport")
    };
    let leaf: RootSessionTree<&'static str> = RootSessionTree::default()
        .children(0, [])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: calc_size(),
                ..NodeInput::default()
            },
        )
        .measure(0, Ok(Size::new(1.0, 1.0)));
    let leaf_batch = compute_layout(&leaf, 0, request()).expect("root leaf calc-size resolves");
    assert_eq!(
        leaf_batch.unrounded_entries()[0].output().size,
        Size::new(120.0, 90.0)
    );

    for intrinsic in [PreferredSize::MIN_CONTENT, PreferredSize::MAX_CONTENT] {
        let block: RootSessionTree<&'static str> =
            RootSessionTree::default().children(0, []).style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(intrinsic.clone(), intrinsic),
                    ..NodeInput::default()
                },
            );
        let batch = compute_layout(&block, 0, request())
            .expect("root block preferred intrinsic sizing resolves");
        assert_eq!(batch.unrounded_entries()[0].output().size, Size::ZERO);
    }
}

fn fri05_c03_tree_leaf_auto_case(
    style: NodeInput,
    measured: Size<f32>,
    expected_inputs: &[Size<f32>],
    expected_content_box: Size<f32>,
    expected_scrollbar_size: Size<f32>,
) {
    let (output, inputs) = fri05_c03_tree_leaf_layout(style, measured, Size::new(100.0, 100.0));
    assert_eq!(inputs, expected_inputs);
    let geometry = output
        .scroll_geometry
        .expect("tree-backed leaf publishes stable geometry");
    assert_eq!(geometry.content_box().size(), expected_content_box);
    assert_eq!(geometry.scrollbar_size(), expected_scrollbar_size);
    assert_eq!(output.scrollbar_size(), expected_scrollbar_size);
}

#[test]
fn fri05_c03_leaf_auto_tree_backed_runs_exact_monotone_passes() {
    let automatic = NodeInput {
        overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
        scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        ..NodeInput::default()
    };
    fri05_c03_tree_leaf_auto_case(
        automatic.clone(),
        Size::new(120.0, 100.0),
        &[
            Size::new(100.0, 100.0),
            Size::new(100.0, 85.0),
            Size::new(85.0, 85.0),
        ],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_tree_leaf_auto_case(
        automatic.clone(),
        Size::new(100.0, 120.0),
        &[
            Size::new(100.0, 100.0),
            Size::new(85.0, 100.0),
            Size::new(85.0, 85.0),
        ],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_tree_leaf_auto_case(
        automatic.clone(),
        Size::new(80.0, 80.0),
        &[Size::new(100.0, 100.0)],
        Size::new(100.0, 100.0),
        Size::ZERO,
    );
    fri05_c03_tree_leaf_auto_case(
        automatic.clone(),
        Size::new(120.0, 80.0),
        &[Size::new(100.0, 100.0), Size::new(100.0, 85.0)],
        Size::new(100.0, 85.0),
        Size::new(0.0, 15.0),
    );
    fri05_c03_tree_leaf_auto_case(
        automatic,
        Size::new(80.0, 120.0),
        &[Size::new(100.0, 100.0), Size::new(85.0, 100.0)],
        Size::new(85.0, 100.0),
        Size::new(15.0, 0.0),
    );
    fri05_c03_tree_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(80.0, 80.0),
        &[Size::new(85.0, 85.0)],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_tree_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::Stable,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(80.0, 80.0),
        &[Size::new(85.0, 100.0)],
        Size::new(85.0, 100.0),
        Size::new(15.0, 0.0),
    );
    fri05_c03_tree_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(60.0, 80.0),
        &[Size::new(70.0, 100.0)],
        Size::new(70.0, 100.0),
        Size::new(30.0, 0.0),
    );
    fri05_c03_tree_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::ZERO,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(120.0, 80.0),
        &[Size::new(100.0, 100.0)],
        Size::new(100.0, 100.0),
        Size::ZERO,
    );
}

fn assert_fri08_c07_t03_optional_math_leaf_results<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let style = NodeInputOf::<S> {
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(PreferredSizeOf::px(scalar(4.0)), PreferredSizeOf::AUTO),
        padding: Edges::new(
            LengthOf::px(scalar(7.0)),
            LengthOf::px(scalar(5.0)),
            LengthOf::px(scalar(4.0)),
            LengthOf::px(scalar(3.0)),
        ),
        ..NodeInputOf::default()
    };
    let input = ComputeInputOf::leaf_layout(
        Size::NONE,
        Size::splat(Some(scalar(100.0))),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::splat(AvailableOf::MAX_CONTENT),
    )
    .unwrap_or_else(|_| panic!("finite leaf layout input must be valid"));
    let output = compute_leaf(input, &style, |_measurement| {
        Ok::<_, core::convert::Infallible>(Size::new(scalar(2.0), scalar(6.0)))
    })
    .unwrap_or_else(|_| panic!("finite leaf sizing must succeed"));

    assert_eq!(output.size, Size::new(scalar(8.0), scalar(17.0)));

    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let overflowing = LengthPercentageOf::from_coefficients(largest, S::ONE)
        .unwrap_or_else(|_| panic!("finite coefficients must be accepted"));
    let failing_style = NodeInputOf::<S> {
        size: Size::new(PreferredSizeOf::value(overflowing), PreferredSizeOf::AUTO),
        ..NodeInputOf::default()
    };
    let failing_input = ComputeInputOf::leaf_layout(
        Size::NONE,
        Size::new(Some(largest), Some(scalar(100.0))),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::definite(largest), AvailableOf::MAX_CONTENT),
    )
    .unwrap_or_else(|_| panic!("finite leaf layout input must be valid"));
    let error = compute_leaf(failing_input, &failing_style, |_measurement| {
        Ok::<_, core::convert::Infallible>(Size::ZERO)
    })
    .expect_err("non-finite leaf sizing must preserve its error");

    assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
            value: S::INFINITY,
        })
    );
}

#[test]
fn fri08_c07_t03_optional_math_leaf_results_preserve_both_scalar_lanes() {
    assert_fri08_c07_t03_optional_math_leaf_results::<f32>();
    assert_fri08_c07_t03_optional_math_leaf_results::<f64>();
}
