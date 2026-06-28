use super::*;
use surgeist_layout::{CacheKeyContext, CalcGeneration};

fn cache_test_input() -> ComputeInput {
    ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::new(None, None),
        parent: Size::new(Some(300.0), Some(200.0)),
        available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    }
}

fn static_cache_context() -> CacheKeyContext {
    CacheKeyContext::static_no_calc()
}

#[test]
fn cache_reuses_measure_and_layout_results_for_matching_inputs() {
    let mut cache = Cache::new();
    let input = ComputeInput {
        run_mode: RunMode::ComputeSize,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::new(None, None),
        parent: Size::new(Some(300.0), Some(200.0)),
        available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    };
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
fn cache_miss_when_run_mode_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store_with_context(
        &base,
        static_cache_context(),
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0)),
    );

    let mut changed = base;
    changed.run_mode = RunMode::ComputeSize;

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

    let mut changed = base;
    changed.sizing_mode = SizingMode::ContentSize;

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

    let mut changed = base;
    changed.axis = RequestedAxis::Horizontal;

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

    let mut changed = base;
    changed.parent = Size::new(Some(200.0), Some(40.0));

    assert_eq!(
        cache.get_with_context(&changed, static_cache_context()),
        None
    );
}

#[test]
fn cache_miss_when_calc_generation_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store_with_context(
        &base,
        CacheKeyContext::new(CalcGeneration::new(1)),
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0)),
    );

    assert_eq!(
        cache.get_with_context(&base, CacheKeyContext::new(CalcGeneration::new(2))),
        None
    );
}

#[test]
fn cache_hit_when_known_width_matches_cached_size_even_if_available_width_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    let output = ComputeOutput::from_outer_size(Size::new(20.0, 10.0));
    cache.store_with_context(&base, static_cache_context(), output);

    let mut changed = base;
    changed.known.width = Some(20.0);
    changed.available.width = Available::MIN_CONTENT;

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
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::new(Some(80.0), Some(24.0)),
        parent: Size::new(None, None),
        available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    };

    let first = compute_cached(&mut probe, 7, input, |tree, _node, _input| {
        tree.calls += 1;
        ComputeOutput::from_outer_size(Size::new(80.0, 24.0))
    });
    let second = compute_cached(&mut probe, 7, input, |tree, _node, _input| {
        tree.calls += 1;
        ComputeOutput::from_outer_size(Size::new(10.0, 10.0))
    });

    assert_eq!(first, ComputeOutput::from_outer_size(Size::new(80.0, 24.0)));
    assert_eq!(second, first);
    assert_eq!(probe.calls, 1);
}

#[test]
fn compute_cached_uses_cache_access_context_generation() {
    struct Probe {
        cache: Cache,
        generation: CalcGeneration,
        calls: usize,
    }

    impl CacheAccess for Probe {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> CacheKeyContext {
            CacheKeyContext::new(self.generation)
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

    let input = cache_test_input();
    let mut probe = Probe {
        cache: Cache::new(),
        generation: CalcGeneration::new(1),
        calls: 0,
    };

    let first = compute_cached(&mut probe, 7, input, |tree, _node, _input| {
        tree.calls += 1;
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0))
    });
    probe.generation = CalcGeneration::new(2);
    let second = compute_cached(&mut probe, 7, input, |tree, _node, _input| {
        tree.calls += 1;
        ComputeOutput::from_outer_size(Size::new(30.0, 10.0))
    });

    assert_eq!(first.size.width, 20.0);
    assert_eq!(second.size.width, 30.0);
    assert_eq!(probe.calls, 2);
}

#[test]
fn f64_cache_context_remains_tree_context_only() {
    let context = CacheKeyContext::new(CalcGeneration::static_no_calc());

    assert_eq!(context.calc_generation(), CalcGeneration::static_no_calc());
}

#[test]
fn f64_cache_key_distinguishes_available_values_that_collide_as_f32() {
    let mut cache = CacheOf::<f64>::new();
    let context = CacheKeyContext::static_no_calc();
    let base = ComputeInputOf {
        run_mode: RunMode::ComputeSize,
        sizing_mode: SizingMode::ContentSize,
        axis: RequestedAxis::Horizontal,
        known: Size::NONE,
        parent: Size::NONE,
        available: Size::new(
            AvailableOf::definite(16_777_216.0),
            AvailableOf::MAX_CONTENT,
        ),
    };
    let nearby = ComputeInputOf {
        available: Size::new(
            AvailableOf::definite(16_777_217.0),
            AvailableOf::MAX_CONTENT,
        ),
        ..base
    };

    let output = ComputeOutputOf::<f64>::from_outer_size(Size::new(1.0, 1.0));
    cache.store_with_context(&base, context, output);

    assert_eq!(cache.get_with_context(&base, context), Some(output));
    assert_eq!(cache.get_with_context(&nearby, context), None);
}
