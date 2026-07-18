use crate::CacheKeyContext;
use crate::*;

fn cache_test_input() -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(None, None),
        Size::new(Some(300.0), Some(200.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
}

fn static_cache_context() -> CacheKeyContext {
    CacheKeyContext::new()
}

#[test]
fn cache_misses_for_parent_formatting_context_only_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
        let input = ComputeInputOf::leaf_layout(
            Size::NONE,
            Size::new(Some(scalar(300.0)), Some(scalar(200.0))),
            crate::ContainingLayoutContext::new(axes, crate::ParentFormattingContext::Flex),
            Size::splat(AvailableOf::MAX_CONTENT),
        )
        .expect("valid cache input");
        let changed = ComputeInputOf::leaf_layout(
            Size::NONE,
            input.parent(),
            crate::ContainingLayoutContext::new(axes, crate::ParentFormattingContext::Grid),
            input.available(),
        )
        .expect("valid cache input");
        let output = complete_compute_size_output();
        let mut cache = CacheOf::<S>::new();
        cache.store_with_context(&input, static_cache_context(), output);
        assert_eq!(
            cache.get_with_context(&input, static_cache_context()),
            Some(output)
        );
        let warm = cache
            .get_with_context(&input, static_cache_context())
            .expect("an unchanged complete context hits");
        assert_eq!(warm.size, output.size);
        assert_eq!(warm.content_size, output.content_size);
        assert_eq!(warm.first_baselines, output.first_baselines);
        assert_eq!(warm.last_baselines, output.last_baselines);
        assert_eq!(warm.block_margin_collapse, output.block_margin_collapse);
        assert_eq!(
            cache.get_with_context(&changed, static_cache_context()),
            None
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn cache_input_with_containing_flow(
    containing_flow_axes: crate::geometry::FlowAxes,
) -> ComputeInput {
    ComputeInput::leaf_layout(
        Size::new(None, None),
        Size::new(Some(300.0), Some(200.0)),
        crate::ContainingLayoutContext::new(
            containing_flow_axes,
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .expect("test cache input is valid")
}

fn all_flow_axes() -> [crate::geometry::FlowAxes; 10] {
    [
        crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        crate::geometry::FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr),
        crate::geometry::FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        crate::geometry::FlowAxes::new(WritingMode::SidewaysRl, Direction::Ltr),
        crate::geometry::FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl),
        crate::geometry::FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        crate::geometry::FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
    ]
}

fn assert_cache_distinguishes_all_containing_flow_axes<S: LayoutScalar>() {
    let context = CacheKeyContext::new();
    for flow_axes in all_flow_axes() {
        let input = ComputeInputOf::leaf_layout(
            Size::new(None, None),
            Size::new(Some(scalar(300.0)), Some(scalar(200.0))),
            crate::ContainingLayoutContext::new(
                flow_axes,
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
        )
        .expect("test cache input is valid");
        let output = ComputeOutputOf::from_outer_size(Size::new(scalar(20.0), scalar(10.0)));
        let mut cache = CacheOf::<S>::new();

        cache.store_with_context(&input, context, output);

        assert_eq!(cache.get_with_context(&input, context), Some(output));
        for distinct_flow_axes in all_flow_axes() {
            if distinct_flow_axes != flow_axes {
                let distinct_input = ComputeInputOf::leaf_layout(
                    Size::new(None, None),
                    Size::new(Some(scalar(300.0)), Some(scalar(200.0))),
                    crate::ContainingLayoutContext::new(
                        distinct_flow_axes,
                        crate::ParentFormattingContext::NoParent,
                    ),
                    Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
                )
                .expect("test cache input is valid");

                assert_eq!(cache.get_with_context(&distinct_input, context), None);
            }
        }
    }
}

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn compute_size_cache_input<S: LayoutScalar>() -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(None, None),
        Size::new(Some(scalar(300.0)), Some(scalar(200.0))),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
    )
}

fn cache_scroll_rect<S: LayoutScalar>(x: f64, y: f64, width: f64, height: f64) -> ScrollRectOf<S> {
    ScrollRectOf::try_new(
        Point::new(scalar(x), scalar(y)),
        Size::new(scalar(width), scalar(height)),
    )
    .expect("test scroll rect is valid")
}

fn cache_scroll_geometry<S: LayoutScalar>() -> ScrollGeometryOf<S> {
    let flow_axes = crate::FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let scrollable_overflow = cache_scroll_rect(0.0, 0.0, 100.0, 80.0);
    let mut contributions =
        crate::scroll::ScrollContributionAccumulatorOf::new(scrollable_overflow);
    contributions.include_direct_line(scrollable_overflow);
    crate::scroll::canonical_scroll_geometry_from_source(
        crate::scroll::CanonicalScrollGeometrySourceOf {
            flow_axes,
            computed_overflow: ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll)
                .expect("same-group computed overflow is valid"),
            item_is_replaced: false,
            border_box_size: Size::new(scalar(44.0), scalar(24.0)),
            border: Edges::ZERO,
            padding: Edges::ZERO,
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: ScrollbarWidthOf::try_new(scalar(2.0)).unwrap(),
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            clip_margin: crate::scroll::ClipMarginSourceOf::default(),
            scroll_padding: crate::scroll::OptimalRegionInsetsOf::default(),
            contributions,
            origin_axes: crate::scroll::ScrollOriginAxes::new(
                crate::scroll::ScrollOriginProgression::FlowEndward,
                crate::scroll::ScrollOriginProgression::FlowEndward,
            ),
            scroll_snap_type: ScrollSnapType::default(),
            target_border_box: cache_scroll_rect(0.0, 0.0, 44.0, 24.0),
            target_scroll_margin: ScrollMarginOf::default(),
            target_flow_axes: flow_axes,
            target_snap_align: ScrollSnapAlign::default(),
            target_snap_stop: ScrollSnapStop::default(),
        },
    )
    .expect("canonical cache geometry is valid")
}

fn complete_compute_size_output<S: LayoutScalar>() -> ComputeOutputOf<S> {
    ComputeOutputOf {
        size: Size::new(scalar(120.0), scalar(40.0)),
        content_size: Size::new(scalar(180.0), scalar(90.0)),
        scroll_geometry: Some(cache_scroll_geometry()),
        first_baselines: Point::new(Some(scalar(12.0)), Some(scalar(18.0))),
        last_baselines: Point::new(Some(scalar(102.0)), Some(scalar(34.0))),
        block_margin_collapse: PhysicalBlockMarginCollapseOf::from_block_flow(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            CollapsibleMarginOf::from_margin(scalar(9.0)).collapse_with_margin(scalar(-3.0)),
            CollapsibleMarginOf::from_margin(scalar(4.0)).collapse_with_margin(scalar(-6.0)),
            true,
        ),
    }
}

fn cache_returns_complete_compute_size_output<S: LayoutScalar>() {
    let mut cache = CacheOf::<S>::new();
    let input = compute_size_cache_input();
    let output = complete_compute_size_output();

    assert_eq!(cache.get_with_context(&input, static_cache_context()), None);
    cache.store_with_context(&input, static_cache_context(), output);
    assert_eq!(
        cache.get_with_context(&input, static_cache_context()),
        Some(output)
    );
}

#[test]
fn compute_size_cache_hit_returns_complete_f32_output() {
    cache_returns_complete_compute_size_output::<f32>();
}

#[test]
fn compute_size_cache_hit_returns_complete_f64_output() {
    cache_returns_complete_compute_size_output::<f64>();
}

#[test]
fn cache_distinguishes_all_containing_flow_axes_for_f32() {
    assert_cache_distinguishes_all_containing_flow_axes::<f32>();
}

#[test]
fn cache_distinguishes_all_containing_flow_axes_for_f64() {
    assert_cache_distinguishes_all_containing_flow_axes::<f64>();
}

#[test]
fn cache_reuses_measure_and_layout_results_for_matching_inputs() {
    let mut cache = Cache::new();
    let input = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(None, None),
        Size::new(Some(300.0), Some(200.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    let output = ComputeOutput::from_outer_size(Size::new(120.0, 40.0));

    assert_eq!(cache.get_with_context(&input, static_cache_context()), None);
    cache.store_with_context(&input, static_cache_context(), output);
    assert_eq!(
        cache.get_with_context(&input, static_cache_context()),
        Some(output)
    );
    assert_eq!(cache.clear(), ClearState::Cleared);
    assert_eq!(cache.clear(), ClearState::AlreadyEmpty);
}

#[test]
fn cache_misses_for_otherwise_identical_input_with_different_containing_flow() {
    let mut cache = Cache::new();
    let horizontal = cache_input_with_containing_flow(crate::geometry::FlowAxes::new(
        WritingMode::HorizontalTb,
        Direction::Ltr,
    ));
    let vertical = cache_input_with_containing_flow(crate::geometry::FlowAxes::new(
        WritingMode::VerticalRl,
        Direction::Ltr,
    ));
    let output = ComputeOutput::from_outer_size(Size::new(20.0, 10.0));

    cache.store_with_context(&horizontal, static_cache_context(), output);

    assert_eq!(
        cache.get_with_context(&horizontal, static_cache_context()),
        Some(output)
    );
    assert_eq!(
        cache.get_with_context(&vertical, static_cache_context()),
        None
    );
}

#[test]
fn cache_miss_when_run_mode_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store_with_context(
        &base,
        static_cache_context(),
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0)),
    );

    let changed = ComputeInput::for_child(
        RunMode::ComputeSize,
        base.sizing_mode(),
        base.requested_axis(),
        base.known(),
        base.parent(),
        crate::ContainingLayoutContext::new(
            base.containing_flow_axes(),
            crate::ParentFormattingContext::NoParent,
        ),
        base.available(),
    );

    assert_eq!(
        cache.get_with_context(&changed, static_cache_context()),
        None
    );
}

#[test]
fn cache_miss_when_sizing_mode_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store_with_context(
        &base,
        static_cache_context(),
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0)),
    );

    let changed = ComputeInput::for_child(
        base.run_mode(),
        SizingMode::ContentSize,
        base.requested_axis(),
        base.known(),
        base.parent(),
        crate::ContainingLayoutContext::new(
            base.containing_flow_axes(),
            crate::ParentFormattingContext::NoParent,
        ),
        base.available(),
    );

    assert_eq!(
        cache.get_with_context(&changed, static_cache_context()),
        None
    );
}

#[test]
fn cache_miss_when_requested_axis_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store_with_context(
        &base,
        static_cache_context(),
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0)),
    );

    let changed = ComputeInput::for_child(
        base.run_mode(),
        base.sizing_mode(),
        RequestedAxis::Horizontal,
        base.known(),
        base.parent(),
        crate::ContainingLayoutContext::new(
            base.containing_flow_axes(),
            crate::ParentFormattingContext::NoParent,
        ),
        base.available(),
    );

    assert_eq!(
        cache.get_with_context(&changed, static_cache_context()),
        None
    );
}

#[test]
fn cache_miss_when_parent_size_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store_with_context(
        &base,
        static_cache_context(),
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0)),
    );

    let changed = ComputeInput::for_child(
        base.run_mode(),
        base.sizing_mode(),
        base.requested_axis(),
        base.known(),
        Size::new(Some(200.0), Some(40.0)),
        crate::ContainingLayoutContext::new(
            base.containing_flow_axes(),
            crate::ParentFormattingContext::NoParent,
        ),
        base.available(),
    );

    assert_eq!(
        cache.get_with_context(&changed, static_cache_context()),
        None
    );
}

#[test]
fn cache_hit_when_known_width_matches_cached_size_even_if_available_width_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    let output = ComputeOutput::from_outer_size(Size::new(20.0, 10.0));
    cache.store_with_context(&base, static_cache_context(), output);

    let changed = ComputeInput::for_child(
        base.run_mode(),
        base.sizing_mode(),
        base.requested_axis(),
        Size::new(Some(20.0), base.known().height),
        base.parent(),
        crate::ContainingLayoutContext::new(
            base.containing_flow_axes(),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MIN_CONTENT, base.available().height),
    );

    assert_eq!(
        cache.get_with_context(&changed, static_cache_context()),
        Some(output)
    );
}

#[test]
fn cached_compute_uses_tree_cache_before_running_expensive_layout() {
    struct Probe {
        cache: Cache,
        calls: usize,
    }

    impl CacheAccess for Probe {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> CacheKeyContext {
            static_cache_context()
        }

        fn cache_get(
            &self,
            _node: Self::Node,
            input: &ComputeInput,
            context: CacheKeyContext,
        ) -> Option<ComputeOutput> {
            self.cache.get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            _node: Self::Node,
            input: &ComputeInput,
            context: CacheKeyContext,
            output: ComputeOutput,
        ) {
            self.cache.store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, _node: Self::Node) {
            self.cache.clear();
        }
    }

    let mut probe = Probe {
        cache: Cache::new(),
        calls: 0,
    };
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(80.0), Some(24.0)),
        Size::new(None, None),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );

    let first = compute_cached(&mut probe, 7, input, |tree, _node, _input| {
        tree.calls += 1;
        Ok(ComputeOutput::from_outer_size(Size::new(80.0, 24.0)))
    });
    let second = compute_cached(&mut probe, 7, input, |tree, _node, _input| {
        tree.calls += 1;
        Ok(ComputeOutput::from_outer_size(Size::new(10.0, 10.0)))
    });

    assert_eq!(
        first,
        Ok(ComputeOutput::from_outer_size(Size::new(80.0, 24.0)))
    );
    assert_eq!(second, first);
    assert_eq!(probe.calls, 1);
}

#[test]
fn f64_cache_context_remains_tree_context_only() {
    let context = CacheKeyContext::new();

    assert_eq!(context, CacheKeyContext);
}

#[test]
fn f64_cache_key_distinguishes_available_values_that_collide_as_f32() {
    let mut cache = CacheOf::<f64>::new();
    let context = CacheKeyContext::new();
    let base = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::ContentSize,
        RequestedAxis::Horizontal,
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(
            AvailableOf::definite(16_777_216.0),
            AvailableOf::MAX_CONTENT,
        ),
    );
    let nearby = ComputeInputOf::for_child(
        base.run_mode(),
        base.sizing_mode(),
        base.requested_axis(),
        base.known(),
        base.parent(),
        crate::ContainingLayoutContext::new(
            base.containing_flow_axes(),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(
            AvailableOf::definite(16_777_217.0),
            AvailableOf::MAX_CONTENT,
        ),
    );

    let output = ComputeOutputOf::<f64>::from_outer_size(Size::new(1.0, 1.0));
    cache.store_with_context(&base, context, output);

    assert_eq!(cache.get_with_context(&base, context), Some(output));
    assert_eq!(cache.get_with_context(&nearby, context), None);
}

struct Fri05C03CacheLeafTree {
    style: NodeInput,
    cache: std::cell::RefCell<Cache>,
    measurement_inputs: std::cell::RefCell<Vec<LeafMeasureInput>>,
}

impl Traverse for Fri05C03CacheLeafTree {
    type Node = u32;
    type Scalar = f32;
    type Children<'a> = std::iter::Empty<u32>;

    fn children(&self, _node: Self::Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Self::Node) -> usize {
        0
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        unreachable!("FRI-05 cache leaf has no children")
    }
}

impl LayoutTree for Fri05C03CacheLeafTree {
    type MeasureError = ();

    fn node_input(&self, _node: Self::Node) -> &NodeInput {
        &self.style
    }

    fn layout_input(&self, _node: Self::Node) -> LayoutInput {
        LayoutInput::box_input(self.style.clone())
    }

    fn has_leaf_measurement(&self, _node: Self::Node) -> bool {
        true
    }

    fn measure_leaf(
        &self,
        _node: Self::Node,
        input: LeafMeasureInput,
    ) -> Option<Result<Size<f32>, Self::MeasureError>> {
        self.measurement_inputs.borrow_mut().push(input);
        Some(Ok(Size::new(120.0, 100.0)))
    }

    fn cache_get(
        &self,
        _node: Self::Node,
        input: &ComputeInput,
        context: CacheKeyContext,
    ) -> Option<ComputeOutput> {
        self.cache.borrow().get_with_context(input, context)
    }
}

impl Fri05C03CacheLeafTree {
    fn apply_cache_entries(&self, entries: &[LayoutCacheStoreEntryOf<u32>]) {
        let mut cache = self.cache.borrow_mut();
        for entry in entries {
            cache.store_with_context(entry.input(), entry.context(), entry.output());
        }
    }

    fn measured_available_sizes(&self) -> Vec<Size<f32>> {
        self.measurement_inputs
            .borrow()
            .iter()
            .map(|input| {
                assert_eq!(input.known_content_size(), Size::NONE);
                input.available_content_size().map(|available| {
                    available
                        .definite_value()
                        .expect("FRI-05 cache measurement availability is definite")
                        .get()
                })
            })
            .collect()
    }
}

#[test]
fn fri05_c03_leaf_cache_tree_backed_stores_only_stable_ordinary_result() {
    let tree = Fri05C03CacheLeafTree {
        style: NodeInput {
            overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        cache: std::cell::RefCell::new(Cache::new()),
        measurement_inputs: std::cell::RefCell::new(Vec::new()),
    };
    let available = Size::splat(Available::definite(100.0));
    let request = LayoutRootRequest::viewport(available).unwrap();

    let cold = compute_layout(&tree, 0, request).expect("cold measured leaf layout succeeds");
    assert_eq!(
        tree.measured_available_sizes(),
        [
            Size::new(100.0, 100.0),
            Size::new(100.0, 85.0),
            Size::new(85.0, 85.0),
        ]
    );
    assert_eq!(cold.cache_store_entries().len(), 1);
    let ordinary_input = ComputeInput::root_layout(
        Size::NONE,
        Size::splat(Some(100.0)),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        available,
    );
    assert_eq!(cold.cache_store_entries()[0].input(), &ordinary_input);
    let cold_output = cold.unrounded_entries()[0].output();
    let cold_geometry = cold_output
        .scroll_geometry
        .expect("cold stable leaf output includes geometry");
    assert_eq!(cold_geometry.content_box().size(), Size::new(85.0, 85.0));
    assert_eq!(cold_geometry.scrollbar_size(), Size::new(15.0, 15.0));

    tree.apply_cache_entries(cold.cache_store_entries());
    tree.measurement_inputs.borrow_mut().clear();
    let warm = compute_layout(&tree, 0, request).expect("warm measured leaf layout succeeds");
    assert!(tree.measurement_inputs.borrow().is_empty());
    assert!(warm.cache_store_entries().is_empty());
    assert_eq!(warm.unrounded_entries()[0].output(), cold_output);
}

#[test]
fn fri05_c03_leaf_cache_key_construction_and_matching_use_exact_state_bits() {
    let ordinary = cache_test_input();
    let states = [
        crate::scroll::SettledAutoScrollbarState::new(false, false),
        crate::scroll::SettledAutoScrollbarState::new(true, false),
        crate::scroll::SettledAutoScrollbarState::new(false, true),
        crate::scroll::SettledAutoScrollbarState::new(true, true),
    ];
    let inputs = states.map(|state| ordinary.with_settled_auto_scrollbars(state));
    assert_eq!(inputs[0], ordinary);
    for (index, input) in inputs.iter().enumerate().skip(1) {
        assert_ne!(
            *input, ordinary,
            "state {index} must be part of input identity"
        );
    }

    for (stored_index, stored_input) in inputs.iter().enumerate() {
        let mut cache = Cache::new();
        let output = ComputeOutput::from_outer_size(Size::new(
            40.0 + stored_index as f32,
            20.0 + stored_index as f32,
        ));
        cache.store_with_context(stored_input, static_cache_context(), output);

        for (lookup_index, lookup_input) in inputs.iter().enumerate() {
            assert_eq!(
                cache.get_with_context(lookup_input, static_cache_context()),
                (lookup_index == stored_index).then_some(output),
                "stored state {stored_index}, lookup state {lookup_index}"
            );
        }
    }
}

#[test]
fn fri05_c04_flex_auto_cache_partitions_known_geometry_by_containing_pass_bits() {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let known = Size::new(Some(40.0), Some(20.0));
    let initial = crate::scroll::SettledAutoScrollbarState::INITIAL;
    let horizontal = crate::scroll::SettledAutoScrollbarState::new(true, false);
    let first = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        known,
        Size::splat(Some(100.0)),
        ContainingLayoutContext::new(axes, ParentFormattingContext::Flex),
        Size::splat(Available::definite(100.0)),
    )
    .with_containing_auto_scrollbar_pass(initial);
    let second = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        known,
        Size::new(Some(100.0), Some(85.0)),
        ContainingLayoutContext::new(axes, ParentFormattingContext::Flex),
        Size::new(Available::definite(100.0), Available::definite(85.0)),
    )
    .with_containing_auto_scrollbar_pass(horizontal);

    assert_eq!(first.settled_auto_scrollbars(), initial);
    assert_eq!(second.settled_auto_scrollbars(), initial);
    assert_eq!(first.known(), second.known());
    assert_ne!(
        first.containing_auto_scrollbar_pass(),
        second.containing_auto_scrollbar_pass()
    );

    let output = ComputeOutput::from_outer_size(Size::new(40.0, 20.0));
    let mut cache = Cache::new();
    cache.store_with_context(&first, static_cache_context(), output);
    assert_eq!(
        cache.get_with_context(&first, static_cache_context()),
        Some(output)
    );
    assert_eq!(
        cache.get_with_context(&second, static_cache_context()),
        None,
        "known-size matching must not erase the containing-pass discriminator"
    );

    cache.store_with_context(&second, static_cache_context(), output);
    assert_eq!(
        cache.get_with_context(&second, static_cache_context()),
        Some(output)
    );
    assert_eq!(
        cache.get_with_context(&first, static_cache_context()),
        None,
        "the warm entry belongs only to the pass that produced it"
    );
}

#[test]
fn fri05_c04_flex_auto_input_constructors_default_both_private_states_to_initial() {
    let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let context = ContainingLayoutContext::new(axes, ParentFormattingContext::Flex);
    let available = Size::splat(Available::definite(100.0));
    let inputs = [
        ComputeInput::leaf_layout(Size::NONE, Size::splat(Some(100.0)), context, available)
            .unwrap(),
        ComputeInput::leaf_content_size(Size::NONE, Size::splat(Some(100.0)), context, available)
            .unwrap(),
        ComputeInput::root_layout(Size::NONE, Size::splat(Some(100.0)), context, available),
        ComputeInput::flex_item_root(Size::NONE, Size::splat(Some(100.0)), context, available),
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::ContentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(100.0)),
            context,
            available,
        ),
        ComputeInput::hidden(context),
    ];
    for input in inputs {
        assert_eq!(
            input.settled_auto_scrollbars(),
            crate::scroll::SettledAutoScrollbarState::INITIAL
        );
        assert_eq!(
            input.containing_auto_scrollbar_pass(),
            crate::scroll::SettledAutoScrollbarState::INITIAL
        );
    }
}
