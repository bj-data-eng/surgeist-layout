use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use crate::test_support::layout_tree::OracleTreeOf;
use crate::*;

#[derive(Clone, Debug, Default)]
struct RootSessionTree<M = &'static str> {
    children: HashMap<u32, Vec<u32>>,
    inputs: HashMap<u32, LayoutInput>,
    measurements: HashMap<u32, Result<Size, M>>,
    leaf_nodes: HashSet<u32>,
    measured_nodes: RefCell<Vec<u32>>,
    caches: RefCell<HashMap<u32, Cache>>,
}

impl<M> RootSessionTree<M> {
    fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    fn style(mut self, node: u32, style: NodeInput) -> Self {
        self.inputs.insert(node, LayoutInput::box_input(style));
        self
    }

    fn measure(mut self, node: u32, output: Result<Size, M>) -> Self {
        self.leaf_nodes.insert(node);
        self.measurements.insert(node, output);
        self
    }

    fn leaf_without_provider(mut self, node: u32) -> Self {
        self.leaf_nodes.insert(node);
        self
    }

    fn measured_nodes(&self) -> Vec<u32> {
        self.measured_nodes.borrow().clone()
    }
}

impl<M> Traverse for RootSessionTree<M> {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<M: Clone> LayoutTree for RootSessionTree<M> {
    type MeasureError = M;

    fn node_input(&self, node: Self::Node) -> &NodeInput {
        self.inputs[&node]
            .as_box()
            .expect("test root session node is a box")
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        self.inputs[&node].clone()
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.leaf_nodes.contains(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        _input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.measured_nodes.borrow_mut().push(node);
        self.measurements.get(&node).cloned()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>> {
        self.caches
            .borrow()
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context))
    }
}

fn root_cache_input(available: Size<Available>) -> ComputeInput {
    ComputeInput {
        run_mode: RunMode::PerformRootLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: available.map(Available::into_option),
        available,
    }
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
        .expect_err("overflowing content-space arithmetic must return no completed batch");

    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
            if *value == -S::INFINITY
    ));
    assert_eq!(tree.measure_calls.get(), 0);
}

#[test]
fn root_request_rejects_invalid_definite_availability() {
    let cases = [
        (
            Size::new(Available::definite(-1.0), Available::MAX_CONTENT),
            Axis::Horizontal,
            NonNegativeFiniteScalarErrorOf::Negative { value: -1.0 },
        ),
        (
            Size::new(Available::definite(f32::NAN), Available::MAX_CONTENT),
            Axis::Horizontal,
            NonNegativeFiniteScalarErrorOf::NonFinite { value: f32::NAN },
        ),
        (
            Size::new(Available::MAX_CONTENT, Available::definite(f32::INFINITY)),
            Axis::Vertical,
            NonNegativeFiniteScalarErrorOf::NonFinite {
                value: f32::INFINITY,
            },
        ),
    ];

    for (available, axis, scalar_error) in cases {
        let error = LayoutRootRequest::viewport(available).unwrap_err();

        assert_eq!(error.axis(), axis);
        match (error.scalar(), scalar_error) {
            (
                NonNegativeFiniteScalarErrorOf::Negative { value },
                NonNegativeFiniteScalarErrorOf::Negative { value: expected },
            ) => assert_eq!(value, expected),
            (
                NonNegativeFiniteScalarErrorOf::NonFinite { value },
                NonNegativeFiniteScalarErrorOf::NonFinite { value: expected },
            ) => {
                if expected.is_nan() {
                    assert!(value.is_nan());
                } else {
                    assert_eq!(value, expected);
                }
            }
            (actual, expected) => panic!("expected {expected:?}, got {actual:?}"),
        }
    }

    let valid_viewport = Size::new(Available::definite(100.0), Available::definite(80.0));
    let flex_context = FlexItemRootContext::under_viewport(valid_viewport).unwrap();
    let error = LayoutRootRequest::flex_item_under_viewport(
        Size::new(Available::definite(-2.0), Available::MAX_CONTENT),
        flex_context,
    )
    .unwrap_err();
    assert_eq!(error.axis(), Axis::Horizontal);
    assert_eq!(
        error.scalar(),
        NonNegativeFiniteScalarErrorOf::Negative { value: -2.0 }
    );
}

#[test]
fn root_request_preserves_distinct_validated_contexts_and_rounding_policy() {
    let available = Size::new(Available::definite(640.0), Available::definite(480.0));
    let viewport = LayoutRootRequest::viewport(available).unwrap();
    let flex_context = FlexItemRootContext::under_viewport(available).unwrap();
    let flex_item = LayoutRootRequest::flex_item_under_viewport(available, flex_context).unwrap();

    assert_eq!(viewport.available(), available);
    assert_eq!(
        viewport.rounding_mode(),
        LayoutRoundingMode::NearestCssPixel
    );
    assert_eq!(viewport.context(), LayoutRootContext::Viewport);
    assert_eq!(
        flex_item.context(),
        LayoutRootContext::FlexItemUnderViewport(flex_context)
    );
    assert_eq!(flex_context.viewport_available(), available);
}

#[test]
fn compute_layout_success_returns_completed_batch_without_tree_mutation() {
    let style = NodeInput {
        size: Size::new(Dimension::px(10.25), Dimension::px(20.5)),
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
        size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
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
fn compute_layout_uses_a_matching_root_cache_hit_without_staging_a_store() {
    let style = NodeInput {
        size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
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
            size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
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
    assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
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
fn f32_tree_leaf_constraint_overflow_returns_typed_error_before_measurement() {
    assert_tree_leaf_constraint_overflow(f32::MAX);
}

#[test]
fn f64_tree_leaf_constraint_overflow_returns_typed_error_before_measurement() {
    assert_tree_leaf_constraint_overflow(f64::MAX);
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
            if output.axis() == Axis::Horizontal
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
fn compute_layout_stops_after_first_recursive_child_error() {
    let tree = RootSessionTree::default()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .style(2, NodeInput::default())
        .measure(1, Err("first child failed"))
        .measure(2, Ok(Size::new(20.0, 10.0)));
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
        &LayoutErrorKind::Measurement("first child failed")
    );
    assert_eq!(tree.measured_nodes(), vec![1]);
}

#[test]
fn compute_layout_reports_consumed_invalid_numeric_resolution() {
    let invalid_padding =
        LengthPercentageOf::from_coefficients(-f32::MAX, -1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
            padding: Edges::new(
                Length::value(invalid_padding),
                Length::ZERO,
                Length::ZERO,
                Length::ZERO,
            ),
            ..NodeInput::default()
        },
    );
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
            value: f32::NEG_INFINITY,
        })
    );
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
                size: Size::new(Dimension::value(overflowing), Dimension::AUTO),
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
                size: Size::new(Dimension::value(overflowing), Dimension::AUTO),
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
fn compute_layout_uses_flex_root_viewport_context_as_parent_basis() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(Dimension::percent(0.5), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    let viewport = Size::new(Available::definite(200.0), Available::definite(80.0));
    let request = LayoutRootRequest::flex_item_under_viewport(
        Size::splat(Available::MAX_CONTENT),
        FlexItemRootContext::under_viewport(viewport).unwrap(),
    )
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("flex-item root layout succeeds");

    assert_eq!(
        batch.unrounded_entries()[0].output().size,
        Size::new(100.0, 20.0)
    );
    assert_eq!(batch.unrounded_entries()[0].output().padding, Edges::ZERO);
    assert_eq!(batch.unrounded_entries()[0].output().border, Edges::ZERO);
}

#[test]
fn compute_layout_rejects_overflowing_affine_grid_auto_fit_track() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let track = TrackSizing::from(Length::value(overflowing));
    let repeat = TrackRepetition::auto_fit(vec![track]).expect("nonempty repeated track list");
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::Repeat(repeat)],
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(20.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
    ));
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
                grid_template_columns: vec![TrackComponent::from(Length::px(20.0))],
                grid_template_rows: vec![TrackComponent::from(Length::px(20.0))],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponent::from(Length::value(overflowing))],
                size: Size::new(Dimension::AUTO, Dimension::px(f32::MAX)),
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

#[test]
fn hidden_layout_clears_cache_sets_zero_layout_and_hides_children() {
    #[derive(Default)]
    struct HiddenTree {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutput>,
        caches: HashMap<u32, Cache>,
        styles: HashMap<u32, NodeInput>,
        hidden_children: Vec<u32>,
    }

    impl Traverse for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for HiddenTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                assert_eq!(input, ComputeInput::HIDDEN);
                self.hidden_children.push(node);
                ComputeOutput::HIDDEN
            })
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutput> {
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
            output: ComputeOutput,
        ) {
            self.caches
                .get_mut(&node)
                .unwrap()
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(1, NodeInput::default());
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());
    tree.caches.insert(1, Cache::new());
    tree.caches.insert(2, Cache::new());
    tree.caches.insert(3, Cache::new());
    tree.caches.get_mut(&1).unwrap().store_with_context(
        &ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::new(Some(1.0), Some(1.0)),
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
        crate::CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(1.0, 1.0)),
    );

    assert_eq!(compute_hidden(&mut tree, 1).unwrap(), ComputeOutput::HIDDEN);
    assert_eq!(tree.layouts[&1], NodeOutput::with_order(0));
    assert_eq!(tree.hidden_children, vec![2, 3]);
    assert!(tree.caches[&1].is_empty());
}

#[test]
fn hidden_layout_writes_zero_line_break_output_without_box_compute() {
    #[derive(Default)]
    struct HiddenTree {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutput>,
        caches: HashMap<u32, Cache>,
        inputs: HashMap<u32, LayoutInput>,
        hidden_children: Vec<u32>,
    }

    impl Traverse for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for HiddenTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            self.inputs[&node]
                .as_box()
                .unwrap_or_else(|| panic!("line break node {node} has no box NodeInput"))
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            self.inputs[&node].clone()
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                assert_eq!(input, ComputeInput::HIDDEN);
                let _ = self.node_input(node);
                self.hidden_children.push(node);
                ComputeOutput::HIDDEN
            })
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutput> {
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
            output: ComputeOutput,
        ) {
            self.caches
                .get_mut(&node)
                .unwrap()
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.inputs
        .insert(1, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(2, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(3, LayoutInput::line_break(LineBreakInput::new()));
    tree.caches.insert(1, Cache::new());
    tree.caches.insert(2, Cache::new());
    tree.caches.insert(3, Cache::new());

    assert_eq!(compute_hidden(&mut tree, 1).unwrap(), ComputeOutput::HIDDEN);
    assert_eq!(tree.hidden_children, vec![2]);
    assert_eq!(tree.layouts[&1], NodeOutput::with_order(0));
    assert_eq!(tree.layouts[&3], NodeOutput::with_order(0));
    assert!(tree.caches[&1].is_empty());
    assert!(tree.caches[&3].is_empty());
}

#[test]
fn hidden_compute_sets_inline_boundary_children_to_hidden_output() {
    #[derive(Default)]
    struct HiddenTree {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutput>,
        caches: HashMap<u32, Cache>,
        inputs: HashMap<u32, LayoutInput>,
        hidden_children: Vec<u32>,
    }

    impl Traverse for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for HiddenTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            self.inputs[&node]
                .as_box()
                .unwrap_or_else(|| panic!("inline boundary node {node} has no box NodeInput"))
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            self.inputs[&node].clone()
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                assert_eq!(input, ComputeInput::HIDDEN);
                let _ = self.node_input(node);
                self.hidden_children.push(node);
                ComputeOutput::HIDDEN
            })
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutput> {
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
            output: ComputeOutput,
        ) {
            self.caches
                .get_mut(&node)
                .unwrap()
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let metrics = InlineMetrics::from_line_height_and_baseline(16.0, 12.0).unwrap();
    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.inputs
        .insert(1, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(2, LayoutInput::box_input(NodeInput::default()));
    tree.inputs.insert(
        3,
        LayoutInput::inline_boundary(InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics)),
    );
    tree.caches.insert(1, Cache::new());
    tree.caches.insert(2, Cache::new());
    tree.caches.insert(3, Cache::new());

    assert_eq!(compute_hidden(&mut tree, 1).unwrap(), ComputeOutput::HIDDEN);
    assert_eq!(tree.hidden_children, vec![2]);
    assert_eq!(tree.layouts[&1], NodeOutput::with_order(0));
    assert_eq!(tree.layouts[&3], NodeOutput::with_order(0));
    assert!(tree.caches[&1].is_empty());
    assert!(tree.caches[&3].is_empty());
}

#[test]
fn f64_compute_hidden_clears_layout_with_f64_output_type() {
    #[derive(Default)]
    struct HiddenTree {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutputOf<f64>>,
        caches: HashMap<u32, CacheOf<f64>>,
        styles: HashMap<u32, NodeInputOf<f64>>,
        hidden_children: Vec<u32>,
    }

    impl Traverse for HiddenTree {
        type Node = u32;
        type Scalar = f64;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for HiddenTree {
        fn node_input(&self, node: Self::Node) -> &NodeInputOf<f64> {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<f64>) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInputOf<f64>,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                assert_eq!(input, ComputeInputOf::HIDDEN);
                self.hidden_children.push(node);
                ComputeOutputOf::HIDDEN
            })
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = f64;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInputOf<f64>,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutputOf<f64>> {
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInputOf<f64>,
            context: crate::CacheKeyContext,
            output: ComputeOutputOf<f64>,
        ) {
            self.caches
                .get_mut(&node)
                .unwrap()
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(1, NodeInputOf::<f64>::default());
    tree.styles.insert(2, NodeInputOf::<f64>::default());
    tree.caches.insert(1, CacheOf::<f64>::new());
    tree.caches.insert(2, CacheOf::<f64>::new());
    tree.caches.get_mut(&1).unwrap().store_with_context(
        &ComputeInputOf::<f64> {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::new(Some(1.25), Some(1.5)),
            parent: Size::NONE,
            available: Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
        },
        crate::CacheKeyContext::new(),
        ComputeOutputOf::from_outer_size(Size::new(1.25, 1.5)),
    );

    assert_eq!(
        compute_hidden(&mut tree, 1).unwrap(),
        ComputeOutputOf::HIDDEN
    );
    assert_eq!(tree.layouts[&1], NodeOutputOf::with_order(0));
    assert_eq!(tree.hidden_children, vec![2]);
    assert!(tree.caches[&1].is_empty());
}

#[test]
fn f64_tree_can_run_root_layout_smoke_test() {
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new().style(
        0,
        NodeInputOf::<f64> {
            display: Display::Block,
            size: Size::new(DimensionOf::px(100.0), DimensionOf::px(50.0)),
            ..NodeInputOf::<f64>::default()
        },
    );

    compute_root(
        &mut tree,
        0,
        Size::new(AvailableOf::definite(100.0), AvailableOf::definite(50.0)),
    )
    .unwrap();

    assert_eq!(
        tree.output(0)
            .expect("root layout must stage output for the root node")
            .size,
        Size::new(100.0, 50.0)
    );
}

struct SingleRootTree {
    style: NodeInput,
    output: ComputeOutput,
    layouts: HashMap<u32, NodeOutput>,
    input: Option<ComputeInput>,
}

impl SingleRootTree {
    fn new(style: NodeInput) -> Self {
        Self {
            style,
            output: ComputeOutput::from_outer_size(Size::ZERO),
            layouts: HashMap::new(),
            input: None,
        }
    }
}

impl Traverse for SingleRootTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Empty<u32>;

    fn children(&self, _node: Self::Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Self::Node) -> usize {
        0
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        unreachable!("root test tree has no children")
    }
}

impl Compute for SingleRootTree {
    fn node_input(&self, _node: Self::Node) -> &NodeInput {
        &self.style
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        _node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        Ok({
            self.input = Some(input);
            self.output
        })
    }
}

#[test]
fn root_layout_emits_scroll_geometry_for_scroll_overflow() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollport(),
        ScrollRect::new(Point::ZERO, Size::new(90.0, 30.0)).unwrap()
    );
    assert_eq!(geometry.range().maximum_offset(), Size::new(40.0, 40.0));
    assert_eq!(
        geometry
            .range()
            .clamp(ScrollOffset::new(Point::new(99.0, -5.0))),
        ScrollOffset::new(Point::new(40.0, 0.0))
    );
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
}

#[test]
fn root_layout_emits_visible_scroll_geometry_without_range() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Visible, Overflow::Visible),
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), None);
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(130.0, 70.0)).unwrap()
    );
    assert_eq!(geometry.range().maximum_offset(), Size::ZERO);
}

#[test]
fn root_layout_emits_clip_geometry_without_range() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Clip, Overflow::Clip),
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_eq!(geometry.range().maximum_offset(), Size::ZERO);
}

#[test]
fn root_scroll_geometry_range_accounts_for_padding_border_and_gutter() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Hidden, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        padding: Edges::all(Length::px(2.0)),
        border: Edges::all(Length::px(3.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollport(),
        ScrollRect::new(Point::new(3.0, 3.0), Size::new(84.0, 34.0)).unwrap()
    );
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::new(5.0, 5.0), Size::new(130.0, 70.0)).unwrap()
    );
    assert_eq!(geometry.range().maximum_offset(), Size::new(48.0, 38.0));
    assert_eq!(
        geometry
            .range()
            .clamp(ScrollOffset::new(Point::new(99.0, 99.0))),
        ScrollOffset::new(Point::new(48.0, 38.0))
    );
}

#[test]
fn root_scroll_geometry_preserves_child_origin_bearing_scrollable_overflow() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        ..NodeInput::default()
    });
    let child_overflow = ScrollRect::new(Point::new(-12.0, -4.0), Size::new(160.0, 74.0)).unwrap();
    let child_geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        child_overflow,
    )
    .unwrap();
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));
    tree.output.scroll_geometry = Some(child_geometry);

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
    assert_eq!(geometry.range().maximum_offset(), Size::new(48.0, 30.0));
}

#[test]
fn f64_round_layout_preserves_large_coordinates() {
    let large = 16_777_217.25_f64;
    let mut tree = OracleTreeOf::<f64>::new()
        .style(0, NodeInputOf::<f64>::default())
        .unrounded(
            0,
            NodeOutputOf::<f64> {
                location: Point::new(large, large + 0.5),
                size: Size::new(10.5, 20.25),
                ..NodeOutputOf::<f64>::default()
            },
        );

    round_layout(&mut tree, 0).unwrap();

    let final_layout = tree
        .output(0)
        .expect("rounding must stage final output for the root node");
    assert_eq!(final_layout.location.x, large.round());
    assert_eq!(final_layout.location.y, (large + 0.5).round());
}

#[test]
fn round_layout_rounds_scroll_geometry_with_node_output() {
    let mut tree = OracleTreeOf::<f64>::new().unrounded(
        0,
        NodeOutputOf::<f64> {
            location: Point::new(10.25, 20.25),
            size: Size::new(100.5, 40.5),
            content_size: Size::new(120.5, 70.5),
            scroll_geometry: Some(
                crate::scroll::scroll_geometry_from_layout(
                    WritingMode::HorizontalTb,
                    Direction::Ltr,
                    Point::new(Overflow::Hidden, Overflow::Hidden),
                    Size::new(100.5, 40.5),
                    Edges::ZERO,
                    Edges::all(0.25),
                    0.0,
                    ScrollRectOf::new(Point::new(0.25, 0.25), Size::new(120.5, 70.5)).unwrap(),
                )
                .unwrap(),
            ),
            ..NodeOutputOf::<f64>::default()
        },
    );

    round_layout(&mut tree, 0).unwrap();

    let geometry = tree
        .output(0)
        .expect("rounding must stage final output for the root node")
        .scroll_geometry
        .unwrap();
    assert_eq!(geometry.scrollport().origin(), Point::new(1.0, 1.0));
    assert_eq!(geometry.scrollport().size(), Size::new(100.0, 40.0));
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(1.0, 1.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(120.0, 70.0)
    );
    assert_eq!(geometry.range().maximum_offset(), Size::new(20.0, 30.0));
}

#[test]
fn round_layout_diagnostics_rejects_invalid_rounded_scroll_geometry() {
    let scrollable_overflow = ScrollRect::new(Point::new(f32::MAX, 0.0), Size::ZERO).unwrap();
    let scroll_geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(1.0, 1.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();
    let mut tree = OracleTreeOf::<f32>::new().unrounded(
        0,
        NodeOutput {
            location: Point::new(f32::MAX, 0.0),
            scroll_geometry: Some(scroll_geometry),
            ..NodeOutput::new()
        },
    );

    let error = round_layout(&mut tree, 0)
        .expect_err("invalid rounded scroll geometry must not stage final output");

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::InternalInvariant(LayoutInternalInvariant::InvalidRoundedScrollGeometry)
    );
    assert_eq!(tree.final_layout(0), None);
}

#[test]
fn root_layout_stores_child_output_as_root_layout() {
    #[derive(Default)]
    struct RootTree {
        style: NodeInput,
        layout: Option<NodeOutput>,
        input: Option<ComputeInput>,
    }

    impl Traverse for RootTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("root has no children in this test")
        }
    }

    impl Compute for RootTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, layout: NodeOutput) {
            self.layout = Some(layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.input = Some(input);
                ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0))
            })
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
            direction: Direction::Rtl,
            overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(13.0).unwrap(),
            ..NodeInput::default()
        },
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    )
    .unwrap();

    assert_eq!(
        tree.input,
        Some(ComputeInput {
            run_mode: RunMode::PerformRootLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::new(Some(200.0), None),
            parent: Size::new(Some(200.0), Some(100.0)),
            available: Size::new(Available::definite(200.0), Available::definite(100.0)),
        })
    );
    let layout = tree.layout.expect("root layout should be stored");
    assert_eq!(layout.location, crate::Point::new(120.0, 0.0));
    assert_eq!(layout.size, Size::new(80.0, 20.0));
    assert_eq!(layout.content_size, Size::new(80.0, 20.0));
    assert_eq!(layout.scrollbar_size, Size::new(13.0, 13.0));
}

#[test]
fn inline_level_root_keeps_intrinsic_width_under_definite_viewport() {
    #[derive(Default)]
    struct RootTree {
        style: NodeInput,
        layout: Option<NodeOutput>,
        input: Option<ComputeInput>,
    }

    impl Traverse for RootTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("root has no children in this test")
        }
    }

    impl Compute for RootTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, layout: NodeOutput) {
            self.layout = Some(layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.input = Some(input);
                ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0))
            })
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
            display: Display::InlineGrid,
            ..NodeInput::default()
        },
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    )
    .unwrap();

    assert_eq!(
        tree.input.expect("root should be computed").known,
        Size::NONE
    );
    assert_eq!(
        tree.layout.expect("root layout should be stored").size,
        Size::new(80.0, 20.0)
    );
}

#[test]
fn max_width_root_uses_clamped_available_width_under_definite_viewport() {
    #[derive(Default)]
    struct RootTree {
        style: NodeInput,
        layout: Option<NodeOutput>,
        input: Option<ComputeInput>,
    }

    impl Traverse for RootTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("root has no children in this test")
        }
    }

    impl Compute for RootTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, layout: NodeOutput) {
            self.layout = Some(layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.input = Some(input);
                let width = input.known.width.unwrap_or(272.0);
                ComputeOutput::from_sizes(Size::new(width, 72.0), Size::new(width, 72.0))
            })
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
            display: Display::Grid,
            max_size: Size::new(Dimension::px(260.0), Dimension::AUTO),
            ..NodeInput::default()
        },
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(800.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.input.expect("root should be computed").known,
        Size::new(Some(260.0), None)
    );
    assert_eq!(
        tree.layout.expect("root layout should be stored").size,
        Size::new(260.0, 72.0)
    );
}

#[test]
fn block_root_with_max_width_uses_clamped_available_outer_width() {
    #[derive(Default)]
    struct RootTree {
        style: NodeInput,
        layout: Option<NodeOutput>,
        input: Option<ComputeInput>,
    }

    impl Traverse for RootTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("root has no children in this test")
        }
    }

    impl Compute for RootTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, layout: NodeOutput) {
            self.layout = Some(layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.input = Some(input);
                ComputeOutput::from_sizes(
                    Size::new(input.known.width.unwrap_or(112.0), 20.0),
                    Size::new(input.known.width.unwrap_or(112.0), 20.0),
                )
            })
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
            display: Display::Grid,
            box_sizing: BoxSizing::ContentBox,
            max_size: Size::new(Dimension::px(260.0), Dimension::AUTO),
            padding: Edges::new(
                Length::px(1.0),
                Length::px(5.0),
                Length::px(1.0),
                Length::px(5.0),
            ),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(800.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.input.expect("root should be computed").known.width,
        Some(272.0)
    );
    assert_eq!(
        tree.layout
            .expect("root layout should be stored")
            .size
            .width,
        272.0
    );
}

#[test]
fn round_layout_uses_cumulative_viewport_edges() {
    #[derive(Default)]
    struct RoundTree {
        children: HashMap<u32, Vec<u32>>,
        unrounded: HashMap<u32, NodeOutput>,
        final_layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for RoundTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Round for RoundTree {
        fn unrounded(
            &self,
            node: Self::Node,
        ) -> crate::LayoutResultOf<Self::Node, NodeOutput, Self::Scalar> {
            Ok(self.unrounded[&node])
        }

        fn set_final(&mut self, node: Self::Node, layout: NodeOutput) {
            self.final_layouts.insert(node, layout);
        }
    }

    let mut tree = RoundTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.unrounded.insert(
        1,
        NodeOutput {
            location: Point::new(0.2, 0.0),
            size: Size::new(10.4, 10.0),
            content_size: Size::new(10.4, 10.0),
            border: Edges::all(0.4),
            padding: Edges::all(0.6),
            ..NodeOutput::new()
        },
    );
    tree.unrounded.insert(
        2,
        NodeOutput {
            location: Point::new(-0.5, 0.0),
            size: Size::new(10.0, 10.0),
            content_size: Size::new(10.0, 10.0),
            border: Edges::all(0.6),
            padding: Edges::all(0.4),
            scrollbar_size: Size::new(0.6, 1.4),
            ..NodeOutput::new()
        },
    );

    round_layout(&mut tree, 1).unwrap();

    assert_eq!(tree.final_layouts[&1].location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layouts[&1].size.width, 11.0);
    assert_eq!(tree.final_layouts[&1].content_size.width, 11.0);
    assert_eq!(tree.final_layouts[&1].border.left, 1.0);
    assert_eq!(tree.final_layouts[&1].border.right, 1.0);
    assert_eq!(tree.final_layouts[&1].padding.left, 1.0);
    assert_eq!(tree.final_layouts[&1].padding.right, 1.0);

    assert_eq!(tree.final_layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layouts[&2].size.width, 10.0);
    assert_eq!(tree.final_layouts[&2].content_size.width, 10.0);
    assert_eq!(tree.final_layouts[&2].scrollbar_size, Size::new(1.0, 1.0));
    assert_eq!(tree.final_layouts[&2].border.left, 0.0);
    assert_eq!(tree.final_layouts[&2].border.right, 1.0);
}
