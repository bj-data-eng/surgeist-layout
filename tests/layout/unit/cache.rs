use super::*;

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

    assert_eq!(cache.get(&input), None);
    cache.store(&input, output);
    assert_eq!(cache.get(&input), Some(output));
    assert_eq!(cache.clear(), ClearState::Cleared);
    assert_eq!(cache.clear(), ClearState::AlreadyEmpty);
}

#[test]
fn cached_compute_uses_tree_cache_before_running_expensive_layout() {
    struct Probe {
        cache: Cache,
        calls: usize,
    }

    impl CacheAccess for Probe {
        type Node = u32;

        fn cache_get(&self, _node: Self::Node, input: &ComputeInput) -> Option<ComputeOutput> {
            self.cache.get(input)
        }

        fn cache_store(&mut self, _node: Self::Node, input: &ComputeInput, output: ComputeOutput) {
            self.cache.store(input, output);
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
