use std::collections::HashMap;

use crate::test_support::layout_tree::OracleTreeOf;
use crate::*;

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

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            assert_eq!(input, ComputeInput::HIDDEN);
            self.hidden_children.push(node);
            ComputeOutput::HIDDEN
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::static_no_calc()
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
        crate::CacheKeyContext::static_no_calc(),
        ComputeOutput::from_outer_size(Size::new(1.0, 1.0)),
    );

    assert_eq!(compute_hidden(&mut tree, 1), ComputeOutput::HIDDEN);
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

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            assert_eq!(input, ComputeInput::HIDDEN);
            let _ = self.node_input(node);
            self.hidden_children.push(node);
            ComputeOutput::HIDDEN
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::static_no_calc()
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

    assert_eq!(compute_hidden(&mut tree, 1), ComputeOutput::HIDDEN);
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
        ) -> ComputeOutputOf<f64> {
            assert_eq!(input, ComputeInputOf::HIDDEN);
            self.hidden_children.push(node);
            ComputeOutputOf::HIDDEN
        }
    }

    impl CacheAccess for HiddenTree {
        type Node = u32;
        type Scalar = f64;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::static_no_calc()
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
        crate::CacheKeyContext::static_no_calc(),
        ComputeOutputOf::from_outer_size(Size::new(1.25, 1.5)),
    );

    assert_eq!(compute_hidden(&mut tree, 1), ComputeOutputOf::HIDDEN);
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
    );

    assert_eq!(tree.output(0).size, Size::new(100.0, 50.0));
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

    round_layout(&mut tree, 0);

    let final_layout = tree.output(0);
    assert_eq!(final_layout.location.x, large.round());
    assert_eq!(final_layout.location.y, (large + 0.5).round());
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

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.input = Some(input);
            ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0))
        }
    }

    let mut tree = RootTree {
        style: NodeInput {
            direction: Direction::Rtl,
            overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: 13.0,
            ..NodeInput::default()
        },
        ..RootTree::default()
    };

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    );

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

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.input = Some(input);
            ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0))
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
    );

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

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.input = Some(input);
            let width = input.known.width.unwrap_or(272.0);
            ComputeOutput::from_sizes(Size::new(width, 72.0), Size::new(width, 72.0))
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
    );

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

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.input = Some(input);
            ComputeOutput::from_sizes(
                Size::new(input.known.width.unwrap_or(112.0), 20.0),
                Size::new(input.known.width.unwrap_or(112.0), 20.0),
            )
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
    );

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
        fn unrounded(&self, node: Self::Node) -> NodeOutput {
            self.unrounded[&node]
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

    round_layout(&mut tree, 1);

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
