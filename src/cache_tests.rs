use crate::CacheKeyContext;
use crate::*;

fn cache_test_input() -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(None, None),
        Size::new(Some(300.0), Some(200.0)),
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
}

fn static_cache_context() -> CacheKeyContext {
    CacheKeyContext::new()
}

fn cache_input_with_containing_flow(
    containing_flow_axes: crate::geometry::FlowAxes,
) -> ComputeInput {
    ComputeInput::leaf_layout(
        Size::new(None, None),
        Size::new(Some(300.0), Some(200.0)),
        containing_flow_axes,
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
            flow_axes,
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
                    distinct_flow_axes,
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
    )
}

fn cache_scroll_rect<S: LayoutScalar>(x: f64, y: f64, width: f64, height: f64) -> ScrollRectOf<S> {
    ScrollRectOf::new(
        Point::new(scalar(x), scalar(y)),
        Size::new(scalar(width), scalar(height)),
    )
    .expect("test scroll rect is valid")
}

fn cache_scroll_geometry<S: LayoutScalar>() -> ScrollGeometryOf<S> {
    let scroll_axis =
        ScrollContainerAxis::from_overflow(Overflow::Scroll).expect("scroll overflow is supported");
    let container = ScrollContainerFacts::new(scroll_axis, scroll_axis);

    ScrollGeometryOf::new(
        crate::FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        container,
        cache_scroll_rect(2.0, 3.0, 40.0, 20.0),
        Some(cache_scroll_rect(1.0, 1.0, 44.0, 24.0)),
        cache_scroll_rect(0.0, 0.0, 100.0, 80.0),
        PhysicalScrollRangeOf::try_new(scalar(-60.0), scalar(0.0), scalar(-60.0), scalar(0.0))
            .expect("test signed physical range is valid"),
        ScrollbarGutterRectsOf::new(
            Some(cache_scroll_rect(2.0, 21.0, 30.0, 2.0)),
            Some(cache_scroll_rect(40.0, 3.0, 2.0, 18.0)),
        ),
    )
    .expect("test scroll geometry is valid")
}

fn complete_compute_size_output<S: LayoutScalar>() -> ComputeOutputOf<S> {
    ComputeOutputOf {
        size: Size::new(scalar(120.0), scalar(40.0)),
        content_size: Size::new(scalar(180.0), scalar(90.0)),
        scroll_geometry: Some(cache_scroll_geometry()),
        first_baselines: Point::new(Some(scalar(12.0)), Some(scalar(18.0))),
        last_baselines: Point::new(Some(scalar(102.0)), Some(scalar(34.0))),
        top_margin: CollapsibleMarginOf::from_margin(scalar(9.0))
            .collapse_with_margin(scalar(-3.0)),
        bottom_margin: CollapsibleMarginOf::from_margin(scalar(4.0))
            .collapse_with_margin(scalar(-6.0)),
        margins_can_collapse_through: true,
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        base.containing_flow_axes(),
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
        base.containing_flow_axes(),
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
        base.containing_flow_axes(),
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
        base.containing_flow_axes(),
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
        base.containing_flow_axes(),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        base.containing_flow_axes(),
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
