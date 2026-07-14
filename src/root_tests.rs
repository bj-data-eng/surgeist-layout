use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use crate::test_support::layout_tree::OracleTreeOf;
use crate::*;

fn assert_positive_physical_range<S: LayoutScalar>(
    range: PhysicalScrollRangeOf<S>,
    maximum: Size<S>,
) {
    assert_eq!(range.x().minimum(), S::ZERO);
    assert_eq!(range.x().maximum(), maximum.width);
    assert_eq!(range.y().minimum(), S::ZERO);
    assert_eq!(range.y().maximum(), maximum.height);
}

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

#[derive(Clone, Debug, Default)]
struct PublicFlowTree<S: LayoutScalar> {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInputOf<S>>,
    caches: RefCell<HashMap<u32, CacheOf<S>>>,
    cache_inputs: RefCell<Vec<(u32, ComputeInputOf<S>)>>,
}

impl<S: LayoutScalar> PublicFlowTree<S> {
    fn with_children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    fn with_style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.styles.insert(node, style);
        self
    }

    fn apply_cache_entries(&self, entries: &[LayoutCacheStoreEntryOf<u32, S>]) {
        let mut caches = self.caches.borrow_mut();
        for entry in entries {
            caches.entry(entry.node()).or_default().store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
    }

    fn cache_inputs(&self, node: u32) -> Vec<ComputeInputOf<S>> {
        self.cache_inputs
            .borrow()
            .iter()
            .filter_map(|(recorded_node, input)| (*recorded_node == node).then_some(*input))
            .collect()
    }

    fn clear_cache_inputs(&self) {
        self.cache_inputs.borrow_mut().clear();
    }
}

impl<S: LayoutScalar> Traverse for PublicFlowTree<S> {
    type Node = u32;
    type Scalar = S;
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

impl<S: LayoutScalar> LayoutTree for PublicFlowTree<S> {
    type MeasureError = ();

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.styles[&node].clone())
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        self.cache_inputs.borrow_mut().push((node, *input));
        self.caches
            .borrow()
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context))
    }
}

struct FlowRootLeafTree<S: LayoutScalar> {
    style: NodeInputOf<S>,
    measurement: RefCell<Option<LeafMeasureInputOf<S>>>,
}

impl<S: LayoutScalar> FlowRootLeafTree<S> {
    fn new(style: NodeInputOf<S>) -> Self {
        Self {
            style,
            measurement: RefCell::new(None),
        }
    }
}

impl<S: LayoutScalar> Traverse for FlowRootLeafTree<S> {
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
        unreachable!("flow-root leaf test tree has no children")
    }
}

impl<S: LayoutScalar> LayoutTree for FlowRootLeafTree<S> {
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
        input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.measurement.replace(Some(input));
        Some(Ok(Size::ZERO))
    }
}

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn single_final_output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .first()
        .expect("single root must produce one final output")
        .output()
}

fn public_flow_output<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .expect("public layout batch contains the requested node")
        .output()
}

fn logical_flex_leaf<S: LayoutScalar>(width: f64, height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            DimensionOf::px(scalar::<S>(width)),
            DimensionOf::px(scalar::<S>(height)),
        ),
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink factor"),
        ..NodeInputOf::default()
    }
}

#[derive(Clone, Copy, Debug)]
struct LogicalFlexChildFlow {
    writing_mode: WritingMode,
    direction: Direction,
}

fn logical_flex_opposing_flow(flow: LogicalFlexChildFlow) -> LogicalFlexChildFlow {
    LogicalFlexChildFlow {
        writing_mode: match flow.writing_mode {
            WritingMode::HorizontalTb => WritingMode::HorizontalTb,
            WritingMode::VerticalRl => WritingMode::VerticalLr,
            WritingMode::VerticalLr => WritingMode::VerticalRl,
            WritingMode::SidewaysRl => WritingMode::SidewaysLr,
            WritingMode::SidewaysLr => WritingMode::SidewaysRl,
        },
        direction: match flow.writing_mode {
            WritingMode::HorizontalTb => match flow.direction {
                Direction::Ltr => Direction::Rtl,
                Direction::Rtl => Direction::Ltr,
            },
            WritingMode::VerticalRl
            | WritingMode::VerticalLr
            | WritingMode::SidewaysRl
            | WritingMode::SidewaysLr => flow.direction,
        },
    }
}

fn logical_flex_orthogonal_flow(flow: LogicalFlexChildFlow) -> LogicalFlexChildFlow {
    LogicalFlexChildFlow {
        writing_mode: match flow.writing_mode {
            WritingMode::HorizontalTb => WritingMode::VerticalLr,
            WritingMode::VerticalRl
            | WritingMode::VerticalLr
            | WritingMode::SidewaysRl
            | WritingMode::SidewaysLr => WritingMode::HorizontalTb,
        },
        direction: flow.direction,
    }
}

fn logical_flex_all_flow_expected(
    writing_mode: WritingMode,
    direction: Direction,
    flex_direction: FlexDirection,
) -> [(f64, f64); 3] {
    match (writing_mode, direction, flex_direction) {
        (WritingMode::HorizontalTb, Direction::Ltr, FlexDirection::Row)
        | (WritingMode::HorizontalTb, Direction::Rtl, FlexDirection::RowReverse) => {
            [(0.0, 0.0), (10.0, 0.0), (30.0, 0.0)]
        }
        (WritingMode::HorizontalTb, Direction::Ltr, FlexDirection::RowReverse)
        | (WritingMode::HorizontalTb, Direction::Rtl, FlexDirection::Row) => {
            [(90.0, 0.0), (70.0, 0.0), (40.0, 0.0)]
        }
        (WritingMode::HorizontalTb, Direction::Ltr, FlexDirection::Column) => {
            [(0.0, 0.0), (0.0, 10.0), (0.0, 30.0)]
        }
        (WritingMode::HorizontalTb, Direction::Ltr, FlexDirection::ColumnReverse) => {
            [(0.0, 90.0), (0.0, 70.0), (0.0, 40.0)]
        }
        (WritingMode::HorizontalTb, Direction::Rtl, FlexDirection::Column) => {
            [(90.0, 0.0), (80.0, 10.0), (70.0, 30.0)]
        }
        (WritingMode::HorizontalTb, Direction::Rtl, FlexDirection::ColumnReverse) => {
            [(90.0, 90.0), (80.0, 70.0), (70.0, 40.0)]
        }
        (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Ltr, FlexDirection::Row)
        | (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::RowReverse,
        ) => [(90.0, 0.0), (80.0, 10.0), (70.0, 30.0)],
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::RowReverse,
        )
        | (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Rtl, FlexDirection::Row) => {
            [(90.0, 90.0), (80.0, 70.0), (70.0, 40.0)]
        }
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::Column,
        ) => [(90.0, 0.0), (70.0, 0.0), (40.0, 0.0)],
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
        ) => [(0.0, 0.0), (10.0, 0.0), (30.0, 0.0)],
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::Column,
        ) => [(90.0, 90.0), (70.0, 80.0), (40.0, 70.0)],
        (
            WritingMode::VerticalRl | WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
        ) => [(0.0, 90.0), (10.0, 80.0), (30.0, 70.0)],
        (WritingMode::VerticalLr, Direction::Ltr, FlexDirection::Row)
        | (WritingMode::VerticalLr, Direction::Rtl, FlexDirection::RowReverse)
        | (WritingMode::SidewaysLr, Direction::Rtl, FlexDirection::Row)
        | (WritingMode::SidewaysLr, Direction::Ltr, FlexDirection::RowReverse) => {
            [(0.0, 0.0), (0.0, 10.0), (0.0, 30.0)]
        }
        (WritingMode::VerticalLr, Direction::Ltr, FlexDirection::RowReverse)
        | (WritingMode::VerticalLr, Direction::Rtl, FlexDirection::Row)
        | (WritingMode::SidewaysLr, Direction::Rtl, FlexDirection::RowReverse)
        | (WritingMode::SidewaysLr, Direction::Ltr, FlexDirection::Row) => {
            [(0.0, 90.0), (0.0, 70.0), (0.0, 40.0)]
        }
        (WritingMode::VerticalLr, Direction::Ltr, FlexDirection::Column)
        | (WritingMode::SidewaysLr, Direction::Rtl, FlexDirection::Column) => {
            [(0.0, 0.0), (10.0, 0.0), (30.0, 0.0)]
        }
        (WritingMode::VerticalLr, Direction::Ltr, FlexDirection::ColumnReverse)
        | (WritingMode::SidewaysLr, Direction::Rtl, FlexDirection::ColumnReverse) => {
            [(90.0, 0.0), (70.0, 0.0), (40.0, 0.0)]
        }
        (WritingMode::VerticalLr, Direction::Rtl, FlexDirection::Column)
        | (WritingMode::SidewaysLr, Direction::Ltr, FlexDirection::Column) => {
            [(0.0, 90.0), (10.0, 80.0), (30.0, 70.0)]
        }
        (WritingMode::VerticalLr, Direction::Rtl, FlexDirection::ColumnReverse)
        | (WritingMode::SidewaysLr, Direction::Ltr, FlexDirection::ColumnReverse) => {
            [(90.0, 90.0), (70.0, 80.0), (40.0, 70.0)]
        }
    }
}

fn assert_logical_flex_placement_vertical_lr_row_projects_inline_main<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.0, 20.0))
        .with_style(2, logical_flex_leaf(10.0, 20.0));
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 1).location,
        Point::new(scalar(0.0), scalar(0.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).location,
        Point::new(scalar(0.0), scalar(20.0))
    );
}

#[test]
fn logical_flex_placement_vertical_lr_row_projects_inline_main_for_f32() {
    assert_logical_flex_placement_vertical_lr_row_projects_inline_main::<f32>();
}

#[test]
fn logical_flex_placement_vertical_lr_row_projects_inline_main_for_f64() {
    assert_logical_flex_placement_vertical_lr_row_projects_inline_main::<f64>();
}

fn assert_logical_flex_boundaries_reverse_and_wrap_reverse_project_once<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let reversed = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::RowReverse,
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.0, 20.0))
        .with_style(2, logical_flex_leaf(10.0, 20.0));
    let reversed_batch = compute_layout(
        &reversed,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("reversed non-leaf flex root layout succeeds");
    assert_eq!(
        public_flow_output(reversed_batch.final_entries(), 1).location,
        Point::new(scalar(0.0), scalar(80.0))
    );
    assert_eq!(
        public_flow_output(reversed_batch.final_entries(), 2).location,
        Point::new(scalar(0.0), scalar(60.0))
    );

    let wrapped = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::WrapReverse,
                align_content: Some(AlignContent::FlexStart),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.0, 60.0))
        .with_style(2, logical_flex_leaf(10.0, 60.0));
    let wrapped_batch = compute_layout(
        &wrapped,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("wrapped non-leaf flex root layout succeeds");
    assert_eq!(
        public_flow_output(wrapped_batch.final_entries(), 1).location,
        Point::new(scalar(90.0), scalar(0.0))
    );
    assert_eq!(
        public_flow_output(wrapped_batch.final_entries(), 2).location,
        Point::new(scalar(80.0), scalar(0.0))
    );
}

#[test]
fn logical_flex_boundaries_reverse_and_wrap_reverse_project_once_for_f32() {
    assert_logical_flex_boundaries_reverse_and_wrap_reverse_project_once::<f32>();
}

#[test]
fn logical_flex_boundaries_reverse_and_wrap_reverse_project_once_for_f64() {
    assert_logical_flex_boundaries_reverse_and_wrap_reverse_project_once::<f64>();
}

fn assert_logical_flex_placement_wrap_reverse_keeps_logical_and_flex_alignment_distinct<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2, 3, 4])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_children(4, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::WrapReverse,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                align_self: Some(AlignItems::Start),
                ..logical_flex_leaf(10.0, 10.0)
            },
        )
        .with_style(
            2,
            NodeInputOf {
                align_self: Some(AlignItems::FlexStart),
                ..logical_flex_leaf(10.0, 10.0)
            },
        )
        .with_style(
            3,
            NodeInputOf {
                align_self: Some(AlignItems::End),
                ..logical_flex_leaf(10.0, 10.0)
            },
        )
        .with_style(
            4,
            NodeInputOf {
                align_self: Some(AlignItems::FlexEnd),
                ..logical_flex_leaf(10.0, 10.0)
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("wrap-reverse logical and flex alignment succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 1).location,
        Point::new(scalar(0.0), scalar(0.0)),
        "logical start remains tied to the container flow start"
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).location,
        Point::new(scalar(10.0), scalar(90.0)),
        "flex start follows the wrap-reversed cross axis"
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 3).location,
        Point::new(scalar(20.0), scalar(90.0)),
        "logical end remains tied to the container flow end"
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 4).location,
        Point::new(scalar(30.0), scalar(0.0)),
        "flex end follows the wrap-reversed cross axis"
    );
}

#[test]
fn logical_flex_placement_wrap_reverse_distinguishes_logical_and_flex_alignment_for_f32() {
    assert_logical_flex_placement_wrap_reverse_keeps_logical_and_flex_alignment_distinct::<f32>();
}

#[test]
fn logical_flex_placement_wrap_reverse_distinguishes_logical_and_flex_alignment_for_f64() {
    assert_logical_flex_placement_wrap_reverse_keeps_logical_and_flex_alignment_distinct::<f64>();
}

fn assert_logical_flex_placement_maps_auto_margins_and_relative_trailing_inset<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                position: Position::Relative,
                margin: Edges {
                    top: LengthAutoOf::AUTO,
                    left: LengthAutoOf::AUTO,
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                inset: Edges {
                    bottom: LengthAutoOf::px(scalar(5.0)),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                ..logical_flex_leaf(10.0, 20.0)
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("logical auto-margin layout succeeds");
    let output = public_flow_output(batch.final_entries(), 1);
    assert_eq!(output.margin.top, scalar(80.0));
    assert_eq!(output.margin.left, scalar(90.0));
    assert_eq!(output.location, Point::new(scalar(90.0), scalar(75.0)));
}

#[test]
fn logical_flex_placement_maps_auto_margins_and_relative_trailing_inset_for_f32() {
    assert_logical_flex_placement_maps_auto_margins_and_relative_trailing_inset::<f32>();
}

#[test]
fn logical_flex_placement_maps_auto_margins_and_relative_trailing_inset_for_f64() {
    assert_logical_flex_placement_maps_auto_margins_and_relative_trailing_inset::<f64>();
}

fn assert_logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    struct Case {
        name: &'static str,
        writing_mode: WritingMode,
        direction: Direction,
        flex_direction: FlexDirection,
        flex_wrap: FlexWrap,
        relative_location: Point<f64>,
        absolute_location: Point<f64>,
    }

    for case in [
        Case {
            name: "horizontal LTR row reverse",
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            flex_direction: FlexDirection::RowReverse,
            flex_wrap: FlexWrap::NoWrap,
            relative_location: Point::new(100.0, 20.0),
            absolute_location: Point::new(10.0, 20.0),
        },
        Case {
            name: "horizontal LTR row wrap reverse",
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::WrapReverse,
            relative_location: Point::new(10.0, 110.0),
            absolute_location: Point::new(10.0, 20.0),
        },
        Case {
            name: "vertical RL RTL row reverse",
            writing_mode: WritingMode::VerticalRl,
            direction: Direction::Rtl,
            flex_direction: FlexDirection::RowReverse,
            flex_wrap: FlexWrap::NoWrap,
            relative_location: Point::new(60.0, -40.0),
            absolute_location: Point::new(60.0, 50.0),
        },
        Case {
            name: "sideways LR RTL row wrap reverse",
            writing_mode: WritingMode::SidewaysLr,
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::WrapReverse,
            relative_location: Point::new(100.0, 20.0),
            absolute_location: Point::new(10.0, 20.0),
        },
        Case {
            name: "horizontal RTL column reverse",
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Rtl,
            flex_direction: FlexDirection::ColumnReverse,
            flex_wrap: FlexWrap::NoWrap,
            relative_location: Point::new(60.0, 110.0),
            absolute_location: Point::new(60.0, 20.0),
        },
    ] {
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::splat(DimensionOf::px(scalar(100.0))),
                    writing_mode: case.writing_mode,
                    direction: case.direction,
                    flex_direction: case.flex_direction,
                    flex_wrap: case.flex_wrap,
                    align_content: Some(AlignContent::FlexStart),
                    align_items: Some(AlignItems::FlexStart),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    position: Position::Relative,
                    inset: Edges {
                        top: LengthAutoOf::px(scalar(20.0)),
                        right: LengthAutoOf::px(scalar(30.0)),
                        bottom: LengthAutoOf::px(scalar(40.0)),
                        left: LengthAutoOf::px(scalar(10.0)),
                    },
                    ..logical_flex_leaf(10.0, 10.0)
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    position: Position::Absolute,
                    inset: Edges {
                        top: LengthAutoOf::px(scalar(20.0)),
                        right: LengthAutoOf::px(scalar(30.0)),
                        bottom: LengthAutoOf::px(scalar(40.0)),
                        left: LengthAutoOf::px(scalar(10.0)),
                    },
                    ..logical_flex_leaf(10.0, 10.0)
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("valid viewport request"),
        )
        .expect("positioned inset precedence layout succeeds");

        assert_eq!(
            public_flow_output(batch.final_entries(), 1).location,
            Point::new(
                scalar(case.relative_location.x),
                scalar(case.relative_location.y),
            ),
            "{} relative positioning keeps normal-flow authored-edge precedence",
            case.name
        );
        assert_eq!(
            public_flow_output(batch.final_entries(), 2).location,
            Point::new(
                scalar(case.absolute_location.x),
                scalar(case.absolute_location.y),
            ),
            "{} absolute positioning keeps normal-flow authored-edge precedence",
            case.name
        );
    }
}

#[test]
fn logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence_for_f32() {
    assert_logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence::<f32>();
}

#[test]
fn logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence_for_f64() {
    assert_logical_flex_boundaries_positioned_insets_keep_normal_flow_precedence::<f64>();
}

fn assert_logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                overflow: Point::new(Overflow::Visible, Overflow::Scroll),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                position: Position::Relative,
                inset: Edges {
                    top: LengthAutoOf::px(scalar(95.5)),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                ..logical_flex_leaf(10.0, 20.0)
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("visible overflow and scroll projection succeed");
    assert_eq!(
        public_flow_output(batch.final_entries(), 1).location,
        Point::new(scalar(0.0), scalar(96.0))
    );
    let root = public_flow_output(batch.final_entries(), 0);
    assert_eq!(root.content_size.height, scalar(116.0));
    assert!(root.scroll_geometry.is_some());
}

#[test]
fn logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical_for_f32() {
    assert_logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical::<f32>();
}

#[test]
fn logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical_for_f64() {
    assert_logical_flex_boundaries_keep_visible_content_scroll_and_rounding_physical::<f64>();
}

fn assert_logical_flex_boundaries_absolute_static_alignment_and_all_flows<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let absolute = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                position: Position::Absolute,
                align_self: Some(AlignItems::FlexEnd),
                ..logical_flex_leaf(10.0, 20.0)
            },
        );
    let batch = compute_layout(
        &absolute,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("logical absolute static alignment succeeds");
    assert_eq!(
        public_flow_output(batch.final_entries(), 1).location,
        Point::new(scalar(90.0), scalar(0.0))
    );

    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for flex_direction in [
                FlexDirection::Row,
                FlexDirection::RowReverse,
                FlexDirection::Column,
                FlexDirection::ColumnReverse,
            ] {
                let parallel_flow = LogicalFlexChildFlow {
                    writing_mode,
                    direction,
                };
                let opposing_flow = logical_flex_opposing_flow(parallel_flow);
                let orthogonal_flow = logical_flex_orthogonal_flow(parallel_flow);
                let tree = PublicFlowTree::default()
                    .with_children(0, [1, 2, 3])
                    .with_children(1, [4])
                    .with_children(2, [5])
                    .with_children(3, [6])
                    .with_children(4, [])
                    .with_children(5, [])
                    .with_children(6, [])
                    .with_style(
                        0,
                        NodeInputOf {
                            display: Display::Flex,
                            writing_mode,
                            direction,
                            size: Size::splat(DimensionOf::px(scalar(100.0))),
                            flex_direction,
                            justify_content: Some(AlignContent::FlexStart),
                            align_items: Some(AlignItems::Start),
                            ..NodeInputOf::default()
                        },
                    )
                    .with_style(
                        1,
                        NodeInputOf {
                            writing_mode: parallel_flow.writing_mode,
                            direction: parallel_flow.direction,
                            ..logical_flex_leaf(10.0, 10.0)
                        },
                    )
                    .with_style(
                        2,
                        NodeInputOf {
                            writing_mode: opposing_flow.writing_mode,
                            direction: opposing_flow.direction,
                            ..logical_flex_leaf(20.0, 20.0)
                        },
                    )
                    .with_style(
                        3,
                        NodeInputOf {
                            writing_mode: orthogonal_flow.writing_mode,
                            direction: orthogonal_flow.direction,
                            ..logical_flex_leaf(30.0, 30.0)
                        },
                    )
                    .with_style(4, logical_flex_leaf(4.0, 5.0))
                    .with_style(5, logical_flex_leaf(6.0, 7.0))
                    .with_style(6, logical_flex_leaf(8.0, 9.0));
                let batch = compute_layout(
                    &tree,
                    0,
                    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(
                        100.0,
                    ))))
                    .expect("valid viewport request"),
                )
                .expect("all logical flex directions complete without fallback");
                assert_eq!(batch.final_entries().len(), 7);
                for (node, (x, y)) in [1_u32, 2, 3]
                    .into_iter()
                    .zip(logical_flex_all_flow_expected(
                        writing_mode,
                        direction,
                        flex_direction,
                    ))
                {
                    assert_eq!(
                        public_flow_output(batch.final_entries(), node).location,
                        Point::new(scalar(x), scalar(y)),
                        "{writing_mode:?} {direction:?} {flex_direction:?} must project child {node} through its physical axis and progression"
                    );
                }
                for (node, child_flow) in [
                    (4_u32, parallel_flow),
                    (5, opposing_flow),
                    (6, orthogonal_flow),
                ] {
                    let (descendant_x, descendant_y) =
                        logical_flex_descendant_expected(node, child_flow);
                    assert_eq!(
                        public_flow_output(batch.final_entries(), node).location,
                        Point::new(scalar(descendant_x), scalar(descendant_y)),
                        "{writing_mode:?} {direction:?} {flex_direction:?} must retain {child_flow:?} for descendant {node}"
                    );
                }
            }
        }
    }
}

fn logical_flex_descendant_expected(node: u32, child_flow: LogicalFlexChildFlow) -> (f64, f64) {
    match (child_flow.writing_mode, child_flow.direction) {
        (WritingMode::HorizontalTb, Direction::Ltr) => (0.0, 0.0),
        (WritingMode::HorizontalTb, Direction::Rtl) => match node {
            4 => (6.0, 0.0),
            5 => (14.0, 0.0),
            6 => (22.0, 0.0),
            _ => unreachable!("all-flow descendant fixture has nodes 4 through 6"),
        },
        (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Ltr) => match node {
            4 => (6.0, 0.0),
            5 => (14.0, 0.0),
            6 => (22.0, 0.0),
            _ => unreachable!("all-flow descendant fixture has nodes 4 through 6"),
        },
        (WritingMode::VerticalRl | WritingMode::SidewaysRl, Direction::Rtl) => match node {
            4 => (6.0, 5.0),
            5 => (14.0, 13.0),
            6 => (22.0, 21.0),
            _ => unreachable!("all-flow descendant fixture has nodes 4 through 6"),
        },
        (WritingMode::VerticalLr, Direction::Ltr) => (0.0, 0.0),
        (WritingMode::VerticalLr, Direction::Rtl) | (WritingMode::SidewaysLr, Direction::Ltr) => {
            match node {
                4 => (0.0, 5.0),
                5 => (0.0, 13.0),
                6 => (0.0, 21.0),
                _ => unreachable!("all-flow descendant fixture has nodes 4 through 6"),
            }
        }
        (WritingMode::SidewaysLr, Direction::Rtl) => (0.0, 0.0),
    }
}

fn assert_logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    for (flex_direction, start, flex_start, end, flex_end) in [
        (
            FlexDirection::RowReverse,
            (0.0, 0.0),
            (90.0, 0.0),
            (90.0, 0.0),
            (0.0, 0.0),
        ),
        (
            FlexDirection::ColumnReverse,
            (0.0, 0.0),
            (0.0, 90.0),
            (0.0, 90.0),
            (0.0, 0.0),
        ),
    ] {
        for (alignment, expected) in [
            (AlignContent::Start, start),
            (AlignContent::FlexStart, flex_start),
            (AlignContent::End, end),
            (AlignContent::FlexEnd, flex_end),
        ] {
            let tree = PublicFlowTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Flex,
                        size: Size::splat(DimensionOf::px(scalar(100.0))),
                        flex_direction,
                        justify_content: Some(alignment),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(1, logical_flex_leaf(10.0, 10.0));
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                    .expect("valid viewport request"),
            )
            .expect("reversed main alignment layout succeeds");
            assert_eq!(
                public_flow_output(batch.final_entries(), 1).location,
                Point::new(scalar(expected.0), scalar(expected.1)),
                "{flex_direction:?} {alignment:?} keeps logical and flex-relative main alignment distinct"
            );
        }
    }
}

fn assert_logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    for (alignment, expected_y) in [
        (AlignContent::Start, 10.0),
        (AlignContent::FlexStart, 90.0),
        (AlignContent::End, 90.0),
        (AlignContent::FlexEnd, 10.0),
    ] {
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::splat(DimensionOf::px(scalar(100.0))),
                    flex_wrap: FlexWrap::WrapReverse,
                    align_content: Some(alignment),
                    ..NodeInputOf::default()
                },
            )
            .with_style(1, logical_flex_leaf(60.0, 10.0))
            .with_style(2, logical_flex_leaf(60.0, 10.0));
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("valid viewport request"),
        )
        .expect("wrap-reversed line alignment layout succeeds");
        assert_eq!(
            public_flow_output(batch.final_entries(), 1).location,
            Point::new(scalar(0.0), scalar(expected_y)),
            "wrap-reverse {alignment:?} keeps logical and flex-relative line alignment distinct"
        );
    }
}

fn assert_logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    for (flex_direction, start, flex_start, end, flex_end) in [
        (
            FlexDirection::RowReverse,
            (0.0, 0.0),
            (90.0, 0.0),
            (90.0, 0.0),
            (0.0, 0.0),
        ),
        (
            FlexDirection::ColumnReverse,
            (0.0, 0.0),
            (0.0, 90.0),
            (0.0, 90.0),
            (0.0, 0.0),
        ),
    ] {
        for (alignment, expected) in [
            (AlignContent::Start, start),
            (AlignContent::FlexStart, flex_start),
            (AlignContent::End, end),
            (AlignContent::FlexEnd, flex_end),
        ] {
            let tree = PublicFlowTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Flex,
                        size: Size::splat(DimensionOf::px(scalar(100.0))),
                        flex_direction,
                        justify_content: Some(alignment),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        position: Position::Absolute,
                        ..logical_flex_leaf(10.0, 10.0)
                    },
                );
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                    .expect("valid viewport request"),
            )
            .expect("absolute reversed main alignment layout succeeds");
            assert_eq!(
                public_flow_output(batch.final_entries(), 1).location,
                Point::new(scalar(expected.0), scalar(expected.1)),
                "absolute {flex_direction:?} {alignment:?} keeps logical and flex-relative main alignment distinct"
            );
        }
    }
}

fn assert_logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2, 3, 4])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_children(4, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::WrapReverse,
                ..NodeInputOf::default()
            },
        );
    let tree = [
        (1, AlignItems::Start),
        (2, AlignItems::FlexStart),
        (3, AlignItems::End),
        (4, AlignItems::FlexEnd),
    ]
    .into_iter()
    .fold(tree, |tree, (node, align_self)| {
        tree.with_style(
            node,
            NodeInputOf {
                position: Position::Absolute,
                align_self: Some(align_self),
                ..logical_flex_leaf(10.0, 10.0)
            },
        )
    });
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("absolute wrap-reverse logical and flex alignment succeeds");

    for (node, expected_y) in [(1, 0.0), (2, 90.0), (3, 90.0), (4, 0.0)] {
        assert_eq!(
            public_flow_output(batch.final_entries(), node).location,
            Point::new(scalar(0.0), scalar(expected_y)),
            "absolute item {node} keeps its logical or flex-relative cross alignment"
        );
    }
}

#[test]
fn logical_flex_boundaries_absolute_static_alignment_and_all_flows_for_f32() {
    assert_logical_flex_boundaries_absolute_static_alignment_and_all_flows::<f32>();
}

#[test]
fn logical_flex_boundaries_absolute_static_alignment_and_all_flows_for_f64() {
    assert_logical_flex_boundaries_absolute_static_alignment_and_all_flows::<f64>();
}

#[test]
fn logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords_for_f32() {
    assert_logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords::<f32>(
    );
}

#[test]
fn logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords_for_f64() {
    assert_logical_flex_placement_reversed_alignment_distinguishes_logical_and_flex_keywords::<f64>(
    );
}

#[test]
fn logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords_for_f32()
 {
    assert_logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords::<f32>();
}

#[test]
fn logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords_for_f64()
 {
    assert_logical_flex_placement_wrap_reverse_align_content_distinguishes_logical_and_flex_keywords::<f64>();
}

#[test]
fn logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords_for_f32()
 {
    assert_logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords::<f32>();
}

#[test]
fn logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords_for_f64()
 {
    assert_logical_flex_boundaries_absolute_reversed_main_alignment_distinguishes_logical_and_flex_keywords::<f64>();
}

#[test]
fn logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment_for_f32()
{
    assert_logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment::<
        f32,
    >();
}

#[test]
fn logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment_for_f64()
{
    assert_logical_flex_boundaries_absolute_wrap_reverse_distinguishes_logical_and_flex_alignment::<
        f64,
    >();
}

fn assert_logical_flex_sizing_vertical_lr_row_uses_container_inline_axis<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                size: Size::new(DimensionOf::px(scalar(10.0)), DimensionOf::px(scalar(20.0))),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::SidewaysLr,
                size: Size::new(DimensionOf::px(scalar(10.0)), DimensionOf::px(scalar(20.0))),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
            .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 1).size,
        Size::new(scalar(10.0), scalar(20.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).size,
        Size::new(scalar(10.0), scalar(20.0))
    );
}

#[test]
fn logical_flex_sizing_vertical_lr_row_uses_container_inline_axis_for_f32() {
    assert_logical_flex_sizing_vertical_lr_row_uses_container_inline_axis::<f32>();
}

#[test]
fn logical_flex_sizing_vertical_lr_row_uses_container_inline_axis_for_f64() {
    assert_logical_flex_sizing_vertical_lr_row_uses_container_inline_axis::<f64>();
}

fn assert_logical_ordinary_grid_container_sizing<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_outer_size = crate::geometry::LogicalSizeOf::new(scalar(70.0), scalar(110.0));
    let logical_style_size = crate::geometry::LogicalSizeOf::new(scalar(80.0), scalar(120.0));
    let logical_min_size = crate::geometry::LogicalSizeOf::new(scalar(60.0), scalar(100.0));
    let logical_gap = crate::geometry::LogicalSizeOf::new(
        LengthOf::percent(scalar(0.1)),
        LengthOf::percent(scalar(0.2)),
    );

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    direction,
                    size: flow_axes
                        .physical_size(logical_style_size)
                        .map(DimensionOf::px),
                    min_size: flow_axes
                        .physical_size(logical_min_size)
                        .map(DimensionOf::px),
                    max_size: flow_axes
                        .physical_size(logical_outer_size)
                        .map(DimensionOf::px),
                    gap: flow_axes.physical_size(logical_gap),
                    grid_template_columns: vec![TrackComponentOf::px(scalar(30.0))],
                    grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                    grid_auto_columns: vec![TrackComponentOf::px(scalar(33.0))],
                    grid_auto_rows: vec![TrackComponentOf::px(scalar(48.0))],
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                    grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("ordinary grid root layout succeeds");
        let expected = flow_axes.physical_size(logical_outer_size);
        let output = public_flow_output(batch.unrounded_entries(), 0);

        assert_eq!(output.size, expected, "{writing_mode:?} {direction:?}");
        assert_eq!(
            output.content_size, expected,
            "{writing_mode:?} {direction:?}"
        );
    }
}

#[test]
fn logical_ordinary_grid_container_sizing_f32() {
    assert_logical_ordinary_grid_container_sizing::<f32>();
}

#[test]
fn logical_ordinary_grid_container_sizing_f64() {
    assert_logical_ordinary_grid_container_sizing::<f64>();
}

fn assert_logical_ordinary_grid_intrinsic_reruns_public_leaves<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let physical_leaf_size =
        Size::new(DimensionOf::px(scalar(17.0)), DimensionOf::px(scalar(31.0)));
    let expected_size = Size::new(scalar(17.0), scalar(31.0));
    let relationships = [
        (
            WritingMode::HorizontalTb,
            Direction::Ltr,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "parallel",
        ),
        (
            WritingMode::HorizontalTb,
            Direction::Rtl,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "opposing",
        ),
        (
            WritingMode::HorizontalTb,
            Direction::Ltr,
            WritingMode::VerticalRl,
            Direction::Ltr,
            "parent-horizontal-child-vertical",
        ),
        (
            WritingMode::SidewaysLr,
            Direction::Rtl,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "parent-sideways-child-horizontal",
        ),
    ];

    for (parent_writing_mode, parent_direction, child_writing_mode, child_direction, label) in
        relationships
    {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::InlineGrid,
                    writing_mode: parent_writing_mode,
                    direction: parent_direction,
                    grid_template_columns: vec![TrackComponentOf::AUTO],
                    grid_template_rows: vec![TrackComponentOf::AUTO],
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: child_writing_mode,
                    direction: child_direction,
                    size: physical_leaf_size,
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("ordinary grid public leaf layout succeeds");

        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 0).size,
            expected_size,
            "{label}"
        );
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 1).size,
            expected_size,
            "{label}"
        );
    }
}

#[test]
fn logical_ordinary_grid_intrinsic_reruns_public_leaves_f32() {
    assert_logical_ordinary_grid_intrinsic_reruns_public_leaves::<f32>();
}

#[test]
fn logical_ordinary_grid_intrinsic_reruns_public_leaves_f64() {
    assert_logical_ordinary_grid_intrinsic_reruns_public_leaves::<f64>();
}

#[derive(Clone, Copy, Debug)]
struct LogicalGridChildFlow {
    writing_mode: WritingMode,
    direction: Direction,
}

fn logical_grid_opposing_flow(flow: LogicalGridChildFlow) -> LogicalGridChildFlow {
    LogicalGridChildFlow {
        writing_mode: match flow.writing_mode {
            WritingMode::HorizontalTb => WritingMode::HorizontalTb,
            WritingMode::VerticalRl => WritingMode::VerticalLr,
            WritingMode::VerticalLr => WritingMode::VerticalRl,
            WritingMode::SidewaysRl => WritingMode::SidewaysLr,
            WritingMode::SidewaysLr => WritingMode::SidewaysRl,
        },
        direction: match flow.writing_mode {
            WritingMode::HorizontalTb => match flow.direction {
                Direction::Ltr => Direction::Rtl,
                Direction::Rtl => Direction::Ltr,
            },
            WritingMode::VerticalRl
            | WritingMode::VerticalLr
            | WritingMode::SidewaysRl
            | WritingMode::SidewaysLr => flow.direction,
        },
    }
}

fn logical_grid_orthogonal_flow(flow: LogicalGridChildFlow) -> LogicalGridChildFlow {
    LogicalGridChildFlow {
        writing_mode: match flow.writing_mode {
            WritingMode::HorizontalTb => WritingMode::VerticalLr,
            WritingMode::VerticalRl
            | WritingMode::VerticalLr
            | WritingMode::SidewaysRl
            | WritingMode::SidewaysLr => WritingMode::HorizontalTb,
        },
        direction: flow.direction,
    }
}

fn nearest_css_pixel<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

fn assert_logical_ordinary_grid_absolute_static<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_container_size = crate::geometry::LogicalSizeOf::new(scalar(70.5), scalar(110.25));
    let logical_child_size = crate::geometry::LogicalSizeOf::new(scalar(11.25), scalar(13.5));
    let explicit_margin =
        crate::geometry::LogicalEdgesOf::new(scalar(1.25), scalar(2.5), scalar(3.75), scalar(4.25));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical_container_size = flow_axes.physical_size(logical_container_size);
        let physical_child_size = flow_axes.physical_size(logical_child_size);
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2, 3, 4])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_children(4, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    direction,
                    size: physical_container_size.map(DimensionOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(scalar(30.25)),
                        TrackComponentOf::px(scalar(40.25)),
                    ],
                    grid_template_rows: vec![
                        TrackComponentOf::px(scalar(50.25)),
                        TrackComponentOf::px(scalar(60.0)),
                    ],
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(DimensionOf::px),
                    position: Position::Absolute,
                    grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                    grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                    margin: flow_axes.physical_edges(explicit_margin.map(LengthAutoOf::px)),
                    inset: flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                        LengthAutoOf::px(scalar(2.25)),
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::px(scalar(3.5)),
                    )),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(DimensionOf::px),
                    position: Position::Absolute,
                    grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                    grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                    margin: flow_axes.physical_edges(explicit_margin.map(LengthAutoOf::px)),
                    justify_self: Some(AlignItems::End),
                    align_self: Some(AlignItems::Center),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(DimensionOf::px),
                    position: Position::Absolute,
                    grid_row: GridPlacement::try_line(2).expect("valid grid row"),
                    margin: flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                    )),
                    justify_self: Some(AlignItems::End),
                    align_self: Some(AlignItems::End),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                4,
                NodeInputOf {
                    display: Display::None,
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("logical ordinary-grid absolute layout succeeds");

        let explicit_inset_location = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(scalar(33.75), scalar(89.0)),
            logical_child_size,
            physical_container_size,
        );
        let aligned_inline = scalar(40.25) - logical_child_size.inline - explicit_margin.inline_end;
        let aligned_block = (scalar(60.0) - logical_child_size.block + explicit_margin.block_start
            - explicit_margin.block_end)
            / scalar(2.0);
        let aligned_location = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(
                scalar(30.25) + aligned_inline,
                scalar(50.25) + aligned_block,
            ),
            logical_child_size,
            physical_container_size,
        );
        let static_location = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(
                (logical_container_size.inline - logical_child_size.inline) / scalar(2.0),
                scalar(50.25) + (scalar(60.0) - logical_child_size.block) / scalar(2.0),
            ),
            logical_child_size,
            physical_container_size,
        );

        for (node, expected_location) in [
            (1, explicit_inset_location),
            (2, aligned_location),
            (3, static_location),
        ] {
            let unrounded = public_flow_output(batch.unrounded_entries(), node);
            let rounded = public_flow_output(batch.final_entries(), node);
            assert_eq!(
                unrounded.location, expected_location,
                "{writing_mode:?} {direction:?} absolute child {node} must project its logical area once"
            );
            assert_eq!(unrounded.size, physical_child_size);
            assert_eq!(
                rounded.location,
                Point::new(
                    nearest_css_pixel(unrounded.location.x),
                    nearest_css_pixel(unrounded.location.y),
                )
            );
        }
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 4),
            NodeOutputOf::with_order(3)
        );
    }
}

#[test]
fn logical_ordinary_grid_absolute_static_f32() {
    assert_logical_ordinary_grid_absolute_static::<f32>();
}

#[test]
fn logical_ordinary_grid_absolute_static_f64() {
    assert_logical_ordinary_grid_absolute_static::<f64>();
}

fn grid_lanes_absolute_expected_location<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    node: u32,
) -> Point<S> {
    let scalar = scalar::<S>;
    let logical_origin = match node {
        1 => crate::geometry::LogicalPointOf::new(scalar(39.25), scalar(96.75)),
        2 => crate::geometry::LogicalPointOf::new(scalar(62.25), scalar(81.0)),
        3 => crate::geometry::LogicalPointOf::new(scalar(32.375), scalar(81.25)),
        _ => unreachable!("grid-lanes fixture has nodes 1 through 3"),
    };
    let flow_axes = FlowAxes::new(writing_mode, direction);
    flow_axes.physical_point(
        logical_origin,
        crate::geometry::LogicalSizeOf::new(scalar(11.25), scalar(13.5)),
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            scalar(76.0),
            scalar(118.0),
        )),
    )
}

fn grid_lanes_nearest_css_pixel<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

fn assert_logical_grid_lanes_absolute_static<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_container_size = crate::geometry::LogicalSizeOf::new(scalar(76.0), scalar(118.0));
    let logical_child_size = crate::geometry::LogicalSizeOf::new(scalar(11.25), scalar(13.5));
    let explicit_margin =
        crate::geometry::LogicalEdgesOf::new(scalar(1.25), scalar(2.5), scalar(3.75), scalar(4.25));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical_container_size = flow_axes.physical_size(logical_container_size);
        let physical_child_size = flow_axes.physical_size(logical_child_size);
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::GridLanes,
                    writing_mode,
                    direction,
                    size: physical_container_size.map(DimensionOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(scalar(30.25)),
                        TrackComponentOf::px(scalar(40.25)),
                    ],
                    grid_template_rows: vec![
                        TrackComponentOf::px(scalar(50.25)),
                        TrackComponentOf::px(scalar(60.0)),
                    ],
                    gap: flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
                        LengthOf::px(scalar(5.5)),
                        LengthOf::px(scalar(7.75)),
                    )),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(DimensionOf::px),
                    position: Position::Absolute,
                    grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                    grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                    margin: flow_axes.physical_edges(explicit_margin.map(LengthAutoOf::px)),
                    inset: flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                        LengthAutoOf::px(scalar(2.25)),
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::px(scalar(3.5)),
                    )),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(DimensionOf::px),
                    position: Position::Absolute,
                    grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                    grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                    margin: flow_axes.physical_edges(explicit_margin.map(LengthAutoOf::px)),
                    justify_self: Some(AlignItems::End),
                    align_self: Some(AlignItems::Center),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: physical_child_size.map(DimensionOf::px),
                    position: Position::Absolute,
                    grid_row: GridPlacement::try_line(2).expect("valid grid row"),
                    margin: flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                    )),
                    justify_self: Some(AlignItems::End),
                    align_self: Some(AlignItems::End),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("grid-lanes absolute layout succeeds");

        for node in [1, 2, 3] {
            let expected_location =
                grid_lanes_absolute_expected_location(writing_mode, direction, node);
            let unrounded = public_flow_output(batch.unrounded_entries(), node);
            let rounded = public_flow_output(batch.final_entries(), node);
            assert_eq!(
                unrounded.location, expected_location,
                "{writing_mode:?} {direction:?} grid-lanes absolute child {node} must preserve its C07 projection"
            );
            assert_eq!(unrounded.size, physical_child_size);
            assert_eq!(
                rounded.location,
                Point::new(
                    grid_lanes_nearest_css_pixel(unrounded.location.x),
                    grid_lanes_nearest_css_pixel(unrounded.location.y),
                )
            );
            assert_eq!(
                rounded.size,
                Size::new(
                    grid_lanes_nearest_css_pixel(unrounded.location.x + unrounded.size.width)
                        - rounded.location.x,
                    grid_lanes_nearest_css_pixel(unrounded.location.y + unrounded.size.height)
                        - rounded.location.y,
                )
            );
        }
    }
}

#[test]
fn logical_grid_lanes_absolute_static_f32() {
    assert_logical_grid_lanes_absolute_static::<f32>();
}

#[test]
fn logical_grid_lanes_absolute_static_f64() {
    assert_logical_grid_lanes_absolute_static::<f64>();
}

fn logical_axis_value<S: LayoutScalar>(
    size: crate::geometry::LogicalSizeOf<S>,
    axis: LogicalAxis,
) -> S {
    match axis {
        LogicalAxis::Inline => size.inline,
        LogicalAxis::Block => size.block,
    }
}

fn logical_axis_start<S: LayoutScalar>(
    edges: crate::geometry::LogicalEdgesOf<S>,
    axis: LogicalAxis,
) -> S {
    match axis {
        LogicalAxis::Inline => edges.inline_start,
        LogicalAxis::Block => edges.block_start,
    }
}

fn logical_axis_margin_sum<S: LayoutScalar>(
    edges: crate::geometry::LogicalEdgesOf<S>,
    axis: LogicalAxis,
) -> S {
    match axis {
        LogicalAxis::Inline => edges.inline_sum(),
        LogicalAxis::Block => edges.block_sum(),
    }
}

fn assert_logical_grid_lanes_axes<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_track_totals = crate::geometry::LogicalSizeOf::new(scalar(70.0), scalar(110.0));
    let logical_gap = crate::geometry::LogicalSizeOf::new(scalar(7.0), scalar(11.0));
    let logical_container_size = logical_track_totals + logical_gap;
    let child_logical_sizes = [
        crate::geometry::LogicalSizeOf::new(scalar(10.0), scalar(13.0)),
        crate::geometry::LogicalSizeOf::new(scalar(12.0), scalar(17.0)),
        crate::geometry::LogicalSizeOf::new(scalar(11.0), scalar(19.0)),
    ];
    let child_logical_margins = [
        crate::geometry::LogicalEdgesOf::new(scalar(1.0), scalar(2.0), scalar(3.0), scalar(4.0)),
        crate::geometry::LogicalEdgesOf::new(scalar(2.0), scalar(1.0), scalar(4.0), scalar(3.0)),
        crate::geometry::LogicalEdgesOf::new(scalar(3.0), scalar(2.0), scalar(1.0), scalar(5.0)),
    ];

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let physical_container_size = flow_axes.physical_size(logical_container_size);
        let parent_flow = LogicalFlexChildFlow {
            writing_mode,
            direction,
        };
        let child_flows = [
            parent_flow,
            logical_flex_opposing_flow(parent_flow),
            logical_flex_orthogonal_flow(parent_flow),
        ];

        for (grid_auto_flow, row_flow) in [(GridAutoFlow::Row, true), (GridAutoFlow::Column, false)]
        {
            let lane_axis = if row_flow {
                LogicalAxis::Block
            } else {
                LogicalAxis::Inline
            };
            let first_margin_box = logical_axis_value(
                flow_axes.logical_size(
                    FlowAxes::new(child_flows[0].writing_mode, child_flows[0].direction)
                        .physical_size(child_logical_sizes[0]),
                ),
                lane_axis,
            ) + logical_axis_margin_sum(
                flow_axes.logical_edges(
                    FlowAxes::new(child_flows[0].writing_mode, child_flows[0].direction)
                        .physical_edges(child_logical_margins[0]),
                ),
                lane_axis,
            );
            let expected_origins = if row_flow {
                [
                    crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                    crate::geometry::LogicalPointOf::new(scalar(37.0), S::ZERO),
                    crate::geometry::LogicalPointOf::new(
                        S::ZERO,
                        first_margin_box + logical_gap.block,
                    ),
                ]
            } else {
                [
                    crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                    crate::geometry::LogicalPointOf::new(S::ZERO, scalar(61.0)),
                    crate::geometry::LogicalPointOf::new(
                        first_margin_box + logical_gap.inline,
                        S::ZERO,
                    ),
                ]
            };

            let mut tree = PublicFlowTree::default()
                .with_children(0, [1, 2, 3])
                .with_children(1, [])
                .with_children(2, [])
                .with_children(3, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::GridLanes,
                        writing_mode,
                        direction,
                        size: physical_container_size.map(DimensionOf::px),
                        grid_auto_flow,
                        grid_template_columns: vec![
                            TrackComponentOf::px(scalar(30.0)),
                            TrackComponentOf::px(scalar(40.0)),
                        ],
                        grid_template_rows: vec![
                            TrackComponentOf::px(scalar(50.0)),
                            TrackComponentOf::px(scalar(60.0)),
                        ],
                        gap: flow_axes.physical_size(logical_gap.map(LengthOf::px)),
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        justify_items: Some(AlignItems::Start),
                        align_items: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                );

            for ((node, child_flow), (logical_size, logical_margin)) in [1, 2, 3]
                .into_iter()
                .zip(child_flows)
                .zip(child_logical_sizes.into_iter().zip(child_logical_margins))
            {
                let child_flow_axes = FlowAxes::new(child_flow.writing_mode, child_flow.direction);
                let mut child_style = NodeInputOf {
                    display: Display::Block,
                    writing_mode: child_flow.writing_mode,
                    direction: child_flow.direction,
                    size: child_flow_axes
                        .physical_size(logical_size)
                        .map(DimensionOf::px),
                    margin: child_flow_axes.physical_edges(logical_margin.map(LengthAutoOf::px)),
                    ..NodeInputOf::default()
                };
                if row_flow {
                    child_style.grid_column =
                        GridPlacement::try_line(if node == 2 { 2 } else { 1 })
                            .expect("valid grid column");
                } else {
                    child_style.grid_row = GridPlacement::try_line(if node == 2 { 2 } else { 1 })
                        .expect("valid grid row");
                }
                tree = tree.with_style(node, child_style);
            }

            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                    .expect("valid viewport request"),
            )
            .expect("logical grid-lanes public layout succeeds");

            let container = public_flow_output(batch.unrounded_entries(), 0);
            assert_eq!(
                container.size, physical_container_size,
                "{writing_mode:?} {direction:?} {grid_auto_flow:?} container size must project logical tracks and gaps"
            );
            assert_eq!(
                container.content_size, physical_container_size,
                "{writing_mode:?} {direction:?} {grid_auto_flow:?} content extent must stay physical at the output boundary"
            );
            for ((node, child_flow), (logical_size, logical_margin)) in [1, 2, 3]
                .into_iter()
                .zip(child_flows)
                .zip(child_logical_sizes.into_iter().zip(child_logical_margins))
            {
                let child_flow_axes = FlowAxes::new(child_flow.writing_mode, child_flow.direction);
                let physical_size = child_flow_axes.physical_size(logical_size);
                let parent_logical_size = flow_axes.logical_size(physical_size);
                let parent_logical_margin =
                    flow_axes.logical_edges(child_flow_axes.physical_edges(logical_margin));
                let expected_logical_origin = expected_origins[(node - 1) as usize]
                    + crate::geometry::LogicalPointOf::new(
                        logical_axis_start(parent_logical_margin, LogicalAxis::Inline),
                        logical_axis_start(parent_logical_margin, LogicalAxis::Block),
                    );
                let expected_location = flow_axes.physical_point(
                    expected_logical_origin,
                    parent_logical_size,
                    physical_container_size,
                );
                let output = public_flow_output(batch.unrounded_entries(), node);
                assert_eq!(
                    output.size, physical_size,
                    "{writing_mode:?} {direction:?} {grid_auto_flow:?} child {node} must retain physical output geometry"
                );
                assert_eq!(
                    output.location, expected_location,
                    "{writing_mode:?} {direction:?} {grid_auto_flow:?} child {node} must place from logical lanes"
                );
            }

            let intrinsic_child_flow = child_flows[2];
            let intrinsic_child_flow_axes = FlowAxes::new(
                intrinsic_child_flow.writing_mode,
                intrinsic_child_flow.direction,
            );
            let intrinsic_parent_logical_size = if row_flow {
                crate::geometry::LogicalSizeOf::new(scalar(30.0), scalar(20.0))
            } else {
                crate::geometry::LogicalSizeOf::new(scalar(20.0), scalar(50.0))
            };
            let intrinsic_physical_size = flow_axes.physical_size(intrinsic_parent_logical_size);
            let intrinsic_tree = PublicFlowTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::InlineGridLanes,
                        writing_mode,
                        direction,
                        grid_auto_flow,
                        grid_template_columns: if row_flow {
                            vec![TrackComponentOf::AUTO, TrackComponentOf::px(scalar(40.0))]
                        } else {
                            vec![
                                TrackComponentOf::px(scalar(30.0)),
                                TrackComponentOf::px(scalar(40.0)),
                            ]
                        },
                        grid_template_rows: if row_flow {
                            vec![
                                TrackComponentOf::px(scalar(50.0)),
                                TrackComponentOf::px(scalar(60.0)),
                            ]
                        } else {
                            vec![TrackComponentOf::AUTO, TrackComponentOf::px(scalar(60.0))]
                        },
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: intrinsic_child_flow.writing_mode,
                        direction: intrinsic_child_flow.direction,
                        size: intrinsic_child_flow_axes
                            .physical_size(
                                intrinsic_child_flow_axes.logical_size(intrinsic_physical_size),
                            )
                            .map(DimensionOf::px),
                        grid_column: if row_flow {
                            GridPlacement::try_line(1).expect("valid intrinsic grid column")
                        } else {
                            GridPlacement::AUTO
                        },
                        grid_row: if row_flow {
                            GridPlacement::AUTO
                        } else {
                            GridPlacement::try_line(1).expect("valid intrinsic grid row")
                        },
                        ..NodeInputOf::default()
                    },
                );
            let intrinsic_batch = compute_layout(
                &intrinsic_tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                    .expect("valid intrinsic viewport request"),
            )
            .expect("logical intrinsic grid-lanes public layout succeeds");
            assert_eq!(
                public_flow_output(intrinsic_batch.unrounded_entries(), 0).size,
                flow_axes.physical_size(logical_track_totals),
                "{writing_mode:?} {direction:?} {grid_auto_flow:?} intrinsic lanes must size on their logical grid axis"
            );
        }
    }

    assert_logical_grid_lanes_absolute_static::<S>();
}

#[test]
fn logical_grid_lanes_axes_f32() {
    assert_logical_grid_lanes_axes::<f32>();
}

#[test]
fn logical_grid_lanes_axes_f64() {
    assert_logical_grid_lanes_axes::<f64>();
}

fn assert_logical_inherited_grid_axis_contexts_public<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let parent_flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    let logical_parent_size = crate::geometry::LogicalSizeOf::new(scalar(77.0), scalar(121.0));
    let parent_size = parent_flow.physical_size(logical_parent_size);

    for (writing_mode, direction) in root_writing_mode_directions() {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode: parent_flow.writing_mode(),
                    direction: parent_flow.direction(),
                    size: parent_size.map(DimensionOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(scalar(30.0)),
                        TrackComponentOf::px(scalar(40.0)),
                    ],
                    grid_template_rows: vec![
                        TrackComponentOf::px(scalar(50.0)),
                        TrackComponentOf::px(scalar(60.0)),
                    ],
                    gap: parent_flow.physical_size(crate::geometry::LogicalSizeOf::new(
                        LengthOf::px(scalar(7.0)),
                        LengthOf::px(scalar(11.0)),
                    )),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    direction,
                    grid_column: GridPlacement::try_lines(1, -1).expect("valid subgrid columns"),
                    grid_row: GridPlacement::try_lines(1, -1).expect("valid subgrid rows"),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        vec![],
                    ))],
                    grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid inherited grid viewport request"),
        )
        .expect("public inherited grid layout succeeds");
        let child = public_flow_output(batch.unrounded_entries(), 1);
        assert_eq!(
            child.size, parent_size,
            "{writing_mode:?} {direction:?} must preserve inherited physical extent"
        );
        assert_eq!(
            child.content_size, parent_size,
            "{writing_mode:?} {direction:?} must preserve inherited physical content extent"
        );
    }
}

#[test]
fn logical_inherited_grid_axis_contexts_public_f32() {
    assert_logical_inherited_grid_axis_contexts_public::<f32>();
}

#[test]
fn logical_inherited_grid_axis_contexts_public_f64() {
    assert_logical_inherited_grid_axis_contexts_public::<f64>();
}

fn assert_logical_subgrid_axes<S: LayoutScalar>() {
    #[derive(Clone, Copy)]
    struct ExpectedTopology {
        inherited_physical_axis: PhysicalAxis,
        parent_axis: GridAxisKind,
        reversed: bool,
    }

    fn expected_axis(
        writing_mode: WritingMode,
        direction: Direction,
        axis: GridAxisKind,
    ) -> (PhysicalAxis, bool) {
        match (writing_mode, direction, axis) {
            (WritingMode::HorizontalTb, Direction::Ltr, GridAxisKind::Column) => {
                (PhysicalAxis::Horizontal, true)
            }
            (WritingMode::HorizontalTb, Direction::Rtl, GridAxisKind::Column) => {
                (PhysicalAxis::Horizontal, false)
            }
            (WritingMode::HorizontalTb, _, GridAxisKind::Row) => (PhysicalAxis::Vertical, true),
            (
                WritingMode::VerticalRl | WritingMode::SidewaysRl,
                Direction::Ltr,
                GridAxisKind::Column,
            ) => (PhysicalAxis::Vertical, true),
            (
                WritingMode::VerticalRl | WritingMode::SidewaysRl,
                Direction::Rtl,
                GridAxisKind::Column,
            ) => (PhysicalAxis::Vertical, false),
            (WritingMode::VerticalRl | WritingMode::SidewaysRl, _, GridAxisKind::Row) => {
                (PhysicalAxis::Horizontal, false)
            }
            (WritingMode::VerticalLr, Direction::Ltr, GridAxisKind::Column) => {
                (PhysicalAxis::Vertical, true)
            }
            (WritingMode::VerticalLr, Direction::Rtl, GridAxisKind::Column) => {
                (PhysicalAxis::Vertical, false)
            }
            (WritingMode::VerticalLr, _, GridAxisKind::Row) => (PhysicalAxis::Horizontal, true),
            (WritingMode::SidewaysLr, Direction::Ltr, GridAxisKind::Column) => {
                (PhysicalAxis::Vertical, false)
            }
            (WritingMode::SidewaysLr, Direction::Rtl, GridAxisKind::Column) => {
                (PhysicalAxis::Vertical, true)
            }
            (WritingMode::SidewaysLr, _, GridAxisKind::Row) => (PhysicalAxis::Horizontal, true),
        }
    }

    fn expected_topology(
        parent_writing_mode: WritingMode,
        parent_direction: Direction,
        child_flow: LogicalFlexChildFlow,
        child_axis: GridAxisKind,
    ) -> ExpectedTopology {
        let (inherited_physical_axis, child_increases) =
            expected_axis(child_flow.writing_mode, child_flow.direction, child_axis);
        let (parent_inline_axis, parent_inline_increases) =
            expected_axis(parent_writing_mode, parent_direction, GridAxisKind::Column);
        let (parent_block_axis, parent_block_increases) =
            expected_axis(parent_writing_mode, parent_direction, GridAxisKind::Row);
        let (parent_axis, parent_increases) = if parent_inline_axis == inherited_physical_axis {
            (GridAxisKind::Column, parent_inline_increases)
        } else {
            debug_assert_eq!(parent_block_axis, inherited_physical_axis);
            (GridAxisKind::Row, parent_block_increases)
        };
        ExpectedTopology {
            inherited_physical_axis,
            parent_axis,
            reversed: parent_increases != child_increases,
        }
    }

    let scalar = scalar::<S>;
    let logical_parent_size = crate::geometry::LogicalSizeOf::new(scalar(77.0), scalar(121.0));
    let logical_gap = crate::geometry::LogicalSizeOf::new(scalar(7.0), scalar(11.0));

    for (parent_writing_mode, parent_direction) in root_writing_mode_directions() {
        let parent_flow = LogicalFlexChildFlow {
            writing_mode: parent_writing_mode,
            direction: parent_direction,
        };
        let parent_flow_axes = FlowAxes::new(parent_writing_mode, parent_direction);
        let parent_size = parent_flow_axes.physical_size(logical_parent_size);
        for child_flow in [
            parent_flow,
            logical_flex_opposing_flow(parent_flow),
            logical_flex_orthogonal_flow(parent_flow),
        ] {
            let child_flow_axes = FlowAxes::new(child_flow.writing_mode, child_flow.direction);
            for axis in [GridAxisKind::Column, GridAxisKind::Row] {
                let topology =
                    expected_topology(parent_writing_mode, parent_direction, child_flow, axis);
                let inherited_physical_axis = topology.inherited_physical_axis;
                let parent_axis = topology.parent_axis;
                let (cross_first_track, cross_second_track, cross_gap) = match parent_axis {
                    GridAxisKind::Column => (scalar(50.0), scalar(60.0), scalar(11.0)),
                    GridAxisKind::Row => (scalar(30.0), scalar(40.0), scalar(7.0)),
                };
                let mut child_style = NodeInputOf {
                    display: Display::Grid,
                    writing_mode: child_flow.writing_mode,
                    direction: child_flow.direction,
                    grid_column: GridPlacement::try_lines(1, -1)
                        .expect("valid subgrid column span"),
                    grid_row: GridPlacement::try_lines(1, -1).expect("valid subgrid row span"),
                    ..NodeInputOf::default()
                };
                match axis {
                    GridAxisKind::Column => {
                        child_style.grid_template_columns =
                            vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))];
                        child_style.grid_template_rows = vec![
                            TrackComponentOf::px(cross_first_track),
                            TrackComponentOf::px(cross_second_track),
                        ];
                    }
                    GridAxisKind::Row => {
                        child_style.grid_template_rows =
                            vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))];
                        child_style.grid_template_columns = vec![
                            TrackComponentOf::px(cross_first_track),
                            TrackComponentOf::px(cross_second_track),
                        ];
                    }
                }
                child_style.gap =
                    child_flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
                        if axis == GridAxisKind::Column {
                            LengthOf::Normal
                        } else {
                            LengthOf::px(cross_gap)
                        },
                        if axis == GridAxisKind::Row {
                            LengthOf::Normal
                        } else {
                            LengthOf::px(cross_gap)
                        },
                    ));
                let tree = PublicFlowTree::default()
                    .with_children(0, [1])
                    .with_children(1, [2])
                    .with_children(2, [])
                    .with_style(
                        0,
                        NodeInputOf {
                            display: Display::Grid,
                            writing_mode: parent_writing_mode,
                            direction: parent_direction,
                            size: parent_size.map(DimensionOf::px),
                            grid_template_columns: vec![
                                TrackComponentOf::px(scalar(30.0)),
                                TrackComponentOf::px(scalar(40.0)),
                            ],
                            grid_template_rows: vec![
                                TrackComponentOf::px(scalar(50.0)),
                                TrackComponentOf::px(scalar(60.0)),
                            ],
                            gap: parent_flow_axes.physical_size(logical_gap.map(LengthOf::px)),
                            ..NodeInputOf::default()
                        },
                    )
                    .with_style(1, child_style)
                    .with_style(
                        2,
                        NodeInputOf {
                            display: Display::Block,
                            writing_mode: child_flow.writing_mode,
                            direction: child_flow.direction,
                            grid_column: if axis == GridAxisKind::Column {
                                GridPlacement::try_lines(2, 3)
                                    .expect("valid inherited column placement")
                            } else {
                                GridPlacement::try_lines(1, 2)
                                    .expect("valid cross-axis column placement")
                            },
                            grid_row: if axis == GridAxisKind::Row {
                                GridPlacement::try_lines(2, 3)
                                    .expect("valid inherited row placement")
                            } else {
                                GridPlacement::try_lines(1, 2)
                                    .expect("valid cross-axis row placement")
                            },
                            ..NodeInputOf::default()
                        },
                    );

                let batch = compute_layout(
                    &tree,
                    0,
                    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(
                        200.0,
                    ))))
                    .expect("valid subgrid viewport request"),
                )
                .expect("logical subgrid public layout succeeds");
                let child = public_flow_output(batch.unrounded_entries(), 1);
                let inherited_extent = match inherited_physical_axis {
                    PhysicalAxis::Horizontal => child.size.width,
                    PhysicalAxis::Vertical => child.size.height,
                };
                let expected_extent = match inherited_physical_axis {
                    PhysicalAxis::Horizontal => parent_size.width,
                    PhysicalAxis::Vertical => parent_size.height,
                };
                assert_eq!(
                    inherited_extent, expected_extent,
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must preserve the inherited physical extent"
                );
                let child_inputs = tree.cache_inputs(1);
                assert!(
                    child_inputs
                        .iter()
                        .any(|input| input.containing_flow_axes() == child_flow_axes),
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must compute the subgrid through its child flow: {child_inputs:?}"
                );
                assert!(
                    child_inputs.iter().any(|input| {
                        let inherited_extent = match topology.inherited_physical_axis {
                            PhysicalAxis::Horizontal => parent_size.width,
                            PhysicalAxis::Vertical => parent_size.height,
                        };
                        let (known, available) = match topology.inherited_physical_axis {
                            PhysicalAxis::Horizontal => {
                                (input.known().width, input.available().width)
                            }
                            PhysicalAxis::Vertical => {
                                (input.known().height, input.available().height)
                            }
                        };
                        input.containing_flow_axes() == child_flow_axes
                            && known == Some(inherited_extent)
                            && available == AvailableOf::definite(inherited_extent)
                    }),
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must project the inherited physical size and available area through the child flow: {child_inputs:?}"
                );

                let descendant = public_flow_output(batch.unrounded_entries(), 2);
                let descendant_origin = parent_flow_axes.logical_point(
                    descendant.location,
                    descendant.size,
                    parent_size,
                );
                let descendant_size = parent_flow_axes.logical_size(descendant.size);
                let (first_track, second_track, gap) = match parent_axis {
                    GridAxisKind::Column => (scalar(30.0), scalar(40.0), scalar(7.0)),
                    GridAxisKind::Row => (scalar(50.0), scalar(60.0), scalar(11.0)),
                };
                let expected_offset = if topology.reversed {
                    S::ZERO
                } else {
                    first_track + gap
                };
                let (actual_offset, actual_extent) = match parent_axis {
                    GridAxisKind::Column => (descendant_origin.inline, descendant_size.inline),
                    GridAxisKind::Row => (descendant_origin.block, descendant_size.block),
                };
                assert_eq!(
                    actual_offset, expected_offset,
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must place the descendant on the mapped inherited track"
                );
                assert_eq!(
                    actual_extent,
                    if topology.reversed {
                        first_track
                    } else {
                        second_track
                    },
                    "{parent_writing_mode:?} {parent_direction:?} {child_flow:?} {axis:?} must preserve the mapped inherited track extent"
                );
            }
        }
    }
}

#[test]
fn logical_subgrid_axes_f32() {
    assert_logical_subgrid_axes::<f32>();
}

#[test]
fn logical_subgrid_axes_f64() {
    assert_logical_subgrid_axes::<f64>();
}

fn assert_nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let vertical = FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [2])
        .with_children(2, [3])
        .with_children(3, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(DimensionOf::MAX_CONTENT, DimensionOf::px(scalar(40.0))),
                grid_template_columns: vec![
                    TrackComponentOf::AUTO,
                    TrackComponentOf::AUTO,
                    TrackComponentOf::AUTO,
                ],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalLr,
                grid_column: GridPlacement::try_lines(1, 3)
                    .expect("outer subgrid spans two of three parent columns"),
                grid_template_columns: vec![TrackComponentOf::px(scalar(40.0))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(1)
                    .expect("inner subgrid spans one of two inherited tracks"),
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            3,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::px(scalar(10.0))),
                ..NodeInputOf::default()
            },
        );

    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
            .expect("valid nested provisional subgrid viewport request"),
    )
    .expect("nested orthogonal partial subgrid layout succeeds");

    let inputs = tree.cache_inputs(1);
    assert!(
        inputs.iter().any(|input| {
            input.run_mode() == RunMode::ComputeSize
                && input.known() == Size::new(Some(scalar(20.0)), None)
                && input.parent() == Size::new(Some(scalar(20.0)), Some(S::ZERO))
                && input.containing_flow_axes() == vertical
                && input.available()
                    == Size::new(
                        AvailableOf::definite(scalar(20.0)),
                        AvailableOf::MAX_CONTENT,
                    )
        }),
        "nested partial subgrid node 1 must retain a resolved cross-axis span and provisional other axis: {inputs:?}"
    );
}

#[test]
fn nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis_f32()
{
    assert_nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis::<
        f32,
    >();
}

#[test]
fn nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis_f64()
{
    assert_nested_orthogonal_partial_subgrid_preserves_resolved_cross_axis_and_provisional_other_axis::<
        f64,
    >();
}

fn assert_subgrid_mbp_preserves_area_basis_and_content_capacity<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let assert_approximately = |actual: S, expected: S, label: &str| {
        assert!(
            (actual - expected).abs() <= S::from_f64(0.000_1),
            "{label}: expected {expected:?}, got {actual:?}"
        );
    };
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [2])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    DimensionOf::px(scalar(100.0)),
                    DimensionOf::px(scalar(40.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(scalar(100.0))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponentOf::px(scalar(40.0))],
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(8.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(5.0)),
                ),
                border: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::px(scalar(9.0)),
                    LengthOf::ZERO,
                    LengthOf::px(scalar(6.0)),
                ),
                padding: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::percent(scalar(0.10)),
                    LengthOf::ZERO,
                    LengthOf::percent(scalar(0.07)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    DimensionOf::percent(scalar(1.0)),
                    DimensionOf::px(scalar(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
            .expect("valid asymmetric subgrid MBP viewport request"),
    )
    .expect("asymmetric subgrid MBP layout succeeds");
    let subgrid = public_flow_output(batch.unrounded_entries(), 1);
    let descendant = public_flow_output(batch.unrounded_entries(), 2);

    assert_eq!(subgrid.location.x, scalar(5.0));
    assert_eq!(subgrid.size.width, scalar(87.0));
    assert_eq!(subgrid.margin.left, scalar(5.0));
    assert_eq!(subgrid.margin.right, scalar(8.0));
    assert_eq!(subgrid.border.left, scalar(6.0));
    assert_eq!(subgrid.border.right, scalar(9.0));
    assert_approximately(
        subgrid.padding.left,
        scalar(7.0),
        "left padding resolves against the raw 100px grid area",
    );
    assert_approximately(
        subgrid.padding.right,
        scalar(10.0),
        "right padding resolves against the raw 100px grid area",
    );
    assert_approximately(
        descendant.location.x,
        scalar(13.0),
        "descendant local x is the subgrid border and padding inset",
    );
    assert_approximately(
        descendant.size.width,
        scalar(55.0),
        "descendant width is the subgrid content capacity",
    );
    assert_approximately(
        subgrid.location.x + descendant.location.x,
        scalar(18.0),
        "subgrid and descendant local coordinates compose to the root-space x",
    );
}

#[test]
fn subgrid_mbp_preserves_area_basis_and_content_capacity_f32() {
    assert_subgrid_mbp_preserves_area_basis_and_content_capacity::<f32>();
}

#[test]
fn subgrid_mbp_preserves_area_basis_and_content_capacity_f64() {
    assert_subgrid_mbp_preserves_area_basis_and_content_capacity::<f64>();
}

fn assert_logical_ordinary_grid_public_contexts<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let viewport = Size::splat(AvailableOf::definite(scalar(200.0)));
    let containing_flow = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let grid_tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [3])
        .with_children(3, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                size: Size::new(
                    DimensionOf::px(scalar(110.0)),
                    DimensionOf::px(scalar(70.0)),
                ),
                grid_template_columns: vec![
                    TrackComponentOf::px(scalar(30.0)),
                    TrackComponentOf::px(scalar(40.0)),
                ],
                grid_template_rows: vec![
                    TrackComponentOf::px(scalar(50.0)),
                    TrackComponentOf::px(scalar(60.0)),
                ],
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                position: Position::Absolute,
                size: Size::new(
                    DimensionOf::px(scalar(10.25)),
                    DimensionOf::px(scalar(20.25)),
                ),
                grid_column: GridPlacement::try_lines(2, 3).expect("valid grid columns"),
                grid_row: GridPlacement::try_lines(2, 3).expect("valid grid rows"),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::None,
                ..NodeInputOf::default()
            },
        )
        .with_style(3, NodeInputOf::default());

    let viewport_batch = compute_layout(
        &grid_tree,
        0,
        LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
    )
    .expect("viewport ordinary-grid context succeeds");
    let child_entry = viewport_batch
        .cache_store_entries()
        .iter()
        .find(|entry| entry.node() == 1 && entry.input().run_mode() == RunMode::PerformLayout)
        .expect("absolute grid child stores a layout cache entry");
    assert_eq!(child_entry.input().containing_flow_axes(), containing_flow);
    assert_eq!(
        child_entry.output().size,
        Size::new(scalar(10.25), scalar(20.25))
    );
    assert_eq!(
        public_flow_output(viewport_batch.final_entries(), 0).size,
        Size::new(scalar(110.0), scalar(70.0))
    );
    assert_eq!(
        public_flow_output(viewport_batch.unrounded_entries(), 2),
        NodeOutputOf::with_order(1)
    );
    assert_eq!(
        public_flow_output(viewport_batch.unrounded_entries(), 3),
        NodeOutputOf::with_order(0)
    );

    grid_tree.apply_cache_entries(viewport_batch.cache_store_entries());
    grid_tree.clear_cache_inputs();
    let warm_batch = compute_layout(
        &grid_tree,
        0,
        LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
    )
    .expect("warm viewport ordinary-grid context succeeds");
    assert!(
        grid_tree
            .cache_inputs(1)
            .iter()
            .any(|input| *input == *child_entry.input())
    );
    assert!(
        warm_batch.cache_store_entries().iter().all(|entry| {
            entry.node() != 1 || entry.input().run_mode() != RunMode::PerformLayout
        })
    );

    let flex_batch = compute_layout(
        &grid_tree,
        0,
        LayoutRootRequestOf::flex_item_under_viewport(
            viewport,
            FlexItemRootContextOf::under_viewport(viewport).expect("valid flex item root context"),
        )
        .expect("valid flex item root request"),
    )
    .expect("flex-item ordinary-grid context succeeds");
    assert_eq!(
        public_flow_output(flex_batch.final_entries(), 1),
        public_flow_output(warm_batch.final_entries(), 1)
    );
}

#[test]
fn logical_ordinary_grid_public_contexts_f32() {
    assert_logical_ordinary_grid_public_contexts::<f32>();
}

#[test]
fn logical_ordinary_grid_public_contexts_f64() {
    assert_logical_ordinary_grid_public_contexts::<f64>();
}

fn assert_logical_ordinary_grid_in_flow_placement_public_output<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let logical_container_size = crate::geometry::LogicalSizeOf::new(scalar(70.0), scalar(110.0));
    let child_size = Size::new(scalar(11.25), scalar(13.5));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let parallel_flow = LogicalGridChildFlow {
            writing_mode,
            direction,
        };
        let opposing_flow = logical_grid_opposing_flow(parallel_flow);
        let orthogonal_flow = logical_grid_orthogonal_flow(parallel_flow);
        let child_flows = [
            parallel_flow,
            opposing_flow,
            orthogonal_flow,
            logical_grid_opposing_flow(orthogonal_flow),
        ];
        let area_origins = [
            (scalar(0.0), scalar(0.0), scalar(30.0), scalar(50.0)),
            (scalar(30.0), scalar(0.0), scalar(40.0), scalar(50.0)),
            (scalar(0.0), scalar(50.0), scalar(30.0), scalar(60.0)),
            (scalar(30.0), scalar(50.0), scalar(40.0), scalar(60.0)),
        ];
        let alignments = [
            (AlignItems::End, AlignItems::Center),
            (AlignItems::Center, AlignItems::Start),
            (AlignItems::Start, AlignItems::End),
            (AlignItems::End, AlignItems::End),
        ];
        let logical_margins = [
            crate::geometry::LogicalEdgesOf::new(
                scalar(1.25),
                scalar(2.5),
                scalar(3.75),
                scalar(4.25),
            ),
            crate::geometry::LogicalEdgesOf::new(
                scalar(2.25),
                scalar(1.5),
                scalar(4.5),
                scalar(3.25),
            ),
            crate::geometry::LogicalEdgesOf::new(
                scalar(3.5),
                scalar(2.0),
                scalar(1.25),
                scalar(5.0),
            ),
            crate::geometry::LogicalEdgesOf::new(
                scalar(1.5),
                scalar(3.75),
                scalar(2.25),
                scalar(4.5),
            ),
        ];
        let relative_offsets = [
            crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
            crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
            crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
            crate::geometry::LogicalPointOf::new(scalar(2.5), -scalar(1.25)),
        ];

        let mut tree = PublicFlowTree::default()
            .with_children(0, [1, 2, 3, 4])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_children(4, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    direction,
                    size: flow_axes
                        .physical_size(logical_container_size)
                        .map(DimensionOf::px),
                    grid_template_columns: vec![
                        TrackComponentOf::px(scalar(30.0)),
                        TrackComponentOf::px(scalar(40.0)),
                    ],
                    grid_template_rows: vec![
                        TrackComponentOf::px(scalar(50.0)),
                        TrackComponentOf::px(scalar(60.0)),
                    ],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            );

        for (index, ((child_flow, (justify_self, align_self)), logical_margin)) in child_flows
            .into_iter()
            .zip(alignments)
            .zip(logical_margins)
            .enumerate()
        {
            let logical_inset = crate::geometry::LogicalEdgesOf::new(
                LengthAutoOf::px(relative_offsets[index].inline),
                LengthAutoOf::AUTO,
                LengthAutoOf::px(relative_offsets[index].block),
                LengthAutoOf::AUTO,
            );
            tree = tree.with_style(
                index as u32 + 1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: child_flow.writing_mode,
                    direction: child_flow.direction,
                    size: child_size.map(DimensionOf::px),
                    margin: flow_axes.physical_edges(logical_margin.map(LengthAutoOf::px)),
                    inset: flow_axes.physical_edges(logical_inset),
                    position: Position::Relative,
                    justify_self: Some(justify_self),
                    align_self: Some(align_self),
                    grid_column: GridPlacement::try_line(index as isize % 2 + 1)
                        .expect("test grid column is valid"),
                    grid_row: GridPlacement::try_line(index as isize / 2 + 1)
                        .expect("test grid row is valid"),
                    ..NodeInputOf::default()
                },
            );
        }

        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(200.0))))
                .expect("valid viewport request"),
        )
        .expect("logical ordinary-grid in-flow placement succeeds");
        let root_unrounded = public_flow_output(batch.unrounded_entries(), 0);

        for (
            index,
            (
                (
                    (inline_origin, block_origin, inline_size, block_size),
                    (justify_self, align_self),
                ),
                logical_margin,
            ),
        ) in area_origins
            .into_iter()
            .zip(alignments)
            .zip(logical_margins)
            .enumerate()
        {
            let logical_child_size = flow_axes.logical_size(child_size);
            let inline_offset = match justify_self {
                AlignItems::Start => logical_margin.inline_start,
                AlignItems::End => {
                    inline_size - logical_child_size.inline - logical_margin.inline_end
                }
                AlignItems::Center => {
                    (inline_size - logical_child_size.inline + logical_margin.inline_start
                        - logical_margin.inline_end)
                        / scalar(2.0)
                }
                _ => unreachable!("the test only uses resolved item alignments"),
            };
            let block_offset = match align_self {
                AlignItems::Start => logical_margin.block_start,
                AlignItems::End => block_size - logical_child_size.block - logical_margin.block_end,
                AlignItems::Center => {
                    (block_size - logical_child_size.block + logical_margin.block_start
                        - logical_margin.block_end)
                        / scalar(2.0)
                }
                _ => unreachable!("the test only uses resolved item alignments"),
            };
            let logical_location = crate::geometry::LogicalPointOf::new(
                inline_origin + inline_offset + relative_offsets[index].inline,
                block_origin + block_offset + relative_offsets[index].block,
            );
            let expected_location = flow_axes.physical_point(
                logical_location,
                logical_child_size,
                flow_axes.physical_size(logical_container_size),
            );
            let unrounded = public_flow_output(batch.unrounded_entries(), index as u32 + 1);
            let rounded = public_flow_output(batch.final_entries(), index as u32 + 1);
            let physical_margin = flow_axes.physical_edges(logical_margin);
            let cumulative_x = root_unrounded.location.x + unrounded.location.x;
            let cumulative_y = root_unrounded.location.y + unrounded.location.y;

            assert_eq!(
                unrounded.location,
                expected_location,
                "{writing_mode:?} {direction:?} child {} must project its logical grid area once",
                index + 1
            );
            assert_eq!(unrounded.size, child_size);
            assert_eq!(unrounded.margin, physical_margin);
            assert_eq!(
                rounded.location,
                Point::new(
                    nearest_css_pixel(unrounded.location.x),
                    nearest_css_pixel(unrounded.location.y),
                )
            );
            assert_eq!(
                rounded.size,
                Size::new(
                    nearest_css_pixel(cumulative_x + unrounded.size.width)
                        - nearest_css_pixel(cumulative_x),
                    nearest_css_pixel(cumulative_y + unrounded.size.height)
                        - nearest_css_pixel(cumulative_y),
                )
            );
        }
    }
}

#[test]
fn logical_ordinary_grid_in_flow_placement_public_output_f32() {
    assert_logical_ordinary_grid_in_flow_placement_public_output::<f32>();
}

#[test]
fn logical_ordinary_grid_in_flow_placement_public_output_f64() {
    assert_logical_ordinary_grid_in_flow_placement_public_output::<f64>();
}

fn assert_logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions<
    S: LayoutScalar,
>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [3])
        .with_children(2, [4])
        .with_children(3, [])
        .with_children(4, [])
        .with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(DimensionOf::px(scalar(30.0)), DimensionOf::px(scalar(60.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            3,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::px(scalar(30.0))),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            4,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::px(scalar(70.0))),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(30.0)),
            AvailableOf::definite(scalar(60.0)),
        ))
        .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 1).size,
        Size::new(scalar(30.0), scalar(30.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).size,
        Size::new(scalar(30.0), scalar(70.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 1).content_size,
        Size::new(scalar(20.0), scalar(30.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).content_size,
        Size::new(scalar(20.0), scalar(70.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 0)
            .content_size
            .height,
        scalar(100.0)
    );
}

#[test]
fn logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions_for_f32() {
    assert_logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions::<f32>();
}

#[test]
fn logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions_for_f64() {
    assert_logical_flex_intrinsic_vertical_lr_row_uses_unequal_intrinsic_contributions::<f64>();
}

fn flex_item_style<S: LayoutScalar>(flex_basis: S) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::splat(DimensionOf::px(scalar(10.0))),
        flex_basis: DimensionOf::px(flex_basis),
        flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow factor"),
        ..NodeInputOf::default()
    }
}

fn assert_logical_flex_sizing_wrap_thresholds_select_container_axes<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    for direction in [
        FlexDirection::Row,
        FlexDirection::RowReverse,
        FlexDirection::Column,
        FlexDirection::ColumnReverse,
    ] {
        let (container_size, bases, expected_sizes) = if direction.is_row() {
            (
                Size::new(DimensionOf::px(scalar(80.0)), DimensionOf::px(scalar(50.0))),
                [scalar(30.0), scalar(30.0), scalar(20.0)],
                [
                    Size::new(scalar(10.0), scalar(50.0)),
                    Size::new(scalar(10.0), scalar(30.0)),
                    Size::new(scalar(10.0), scalar(20.0)),
                ],
            )
        } else {
            (
                Size::new(
                    DimensionOf::px(scalar(100.0)),
                    DimensionOf::px(scalar(80.0)),
                ),
                [scalar(60.0), scalar(60.0), scalar(40.0)],
                [
                    Size::new(scalar(100.0), scalar(10.0)),
                    Size::new(scalar(60.0), scalar(10.0)),
                    Size::new(scalar(40.0), scalar(10.0)),
                ],
            )
        };
        let tree = PublicFlowTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInputOf {
                    writing_mode: WritingMode::VerticalLr,
                    size: container_size,
                    flex_direction: direction,
                    flex_wrap: FlexWrap::Wrap,
                    ..NodeInputOf::default()
                },
            )
            .with_style(1, flex_item_style(bases[0]))
            .with_style(2, flex_item_style(bases[1]))
            .with_style(3, flex_item_style(bases[2]));
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(scalar(100.0)),
                AvailableOf::definite(scalar(100.0)),
            ))
            .expect("valid viewport request"),
        )
        .expect("non-leaf flex root layout succeeds");

        for (node, expected_size) in [1_u32, 2, 3].into_iter().zip(expected_sizes) {
            assert_eq!(
                public_flow_output(batch.final_entries(), node).size,
                expected_size
            );
        }
    }
}

#[test]
fn logical_flex_sizing_wrap_thresholds_select_container_axes_for_f32() {
    assert_logical_flex_sizing_wrap_thresholds_select_container_axes::<f32>();
}

#[test]
fn logical_flex_sizing_wrap_thresholds_select_container_axes_for_f64() {
    assert_logical_flex_sizing_wrap_thresholds_select_container_axes::<f64>();
}

fn assert_logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let item = NodeInputOf {
        display: Display::Block,
        size: Size::splat(DimensionOf::px(scalar(10.0))),
        flex_basis: DimensionOf::px(scalar(45.0)),
        flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow factor"),
        margin: Edges::all(LengthAutoOf::percent(scalar(0.1))),
        ..NodeInputOf::default()
    };
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2, 3])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    DimensionOf::px(scalar(100.0)),
                    DimensionOf::px(scalar(200.0)),
                ),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                gap: Size::new(
                    LengthOf::percent(scalar(0.1)),
                    LengthOf::percent(scalar(0.1)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, item.clone())
        .with_style(2, item.clone())
        .with_style(3, item);
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(100.0)),
            AvailableOf::definite(scalar(200.0)),
        ))
        .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    for node in [1_u32, 2, 3] {
        let output = public_flow_output(batch.final_entries(), node);
        assert_eq!(output.margin, Edges::all(scalar(20.0)));
    }
    assert_eq!(
        public_flow_output(batch.final_entries(), 1).size.height,
        scalar(50.0)
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 2).size.height,
        scalar(50.0)
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 3).size.height,
        scalar(160.0)
    );
}

#[test]
fn logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes_for_f32() {
    assert_logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes::<f32>();
}

#[test]
fn logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes_for_f64() {
    assert_logical_flex_intrinsic_percentage_margin_and_gap_use_container_axes::<f64>();
}

fn assert_logical_flex_sizing_preserves_horizontal_and_child_flow_ownership<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let tree = PublicFlowTree::default()
        .with_children(0, [1, 2, 3])
        .with_children(1, [4])
        .with_children(2, [5])
        .with_children(3, [6])
        .with_children(4, [])
        .with_children(5, [])
        .with_children(6, [])
        .with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    DimensionOf::px(scalar(100.0)),
                    DimensionOf::px(scalar(120.0)),
                ),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(DimensionOf::px(scalar(30.0)), DimensionOf::px(scalar(40.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(DimensionOf::px(scalar(30.0)), DimensionOf::px(scalar(40.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            3,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::HorizontalTb,
                size: Size::new(DimensionOf::px(scalar(30.0)), DimensionOf::px(scalar(40.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            4,
            NodeInputOf {
                display: Display::Block,
                flex_basis: DimensionOf::percent(scalar(0.5)),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            5,
            NodeInputOf {
                display: Display::Block,
                flex_basis: DimensionOf::percent(scalar(0.5)),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            6,
            NodeInputOf {
                display: Display::Block,
                flex_basis: DimensionOf::percent(scalar(0.5)),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(100.0)),
            AvailableOf::definite(scalar(120.0)),
        ))
        .expect("valid viewport request"),
    )
    .expect("non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(batch.final_entries(), 4).size,
        Size::new(scalar(30.0), scalar(20.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 5).size,
        Size::new(scalar(30.0), scalar(20.0))
    );
    assert_eq!(
        public_flow_output(batch.final_entries(), 6).size,
        Size::new(scalar(15.0), scalar(40.0))
    );

    let horizontal = PublicFlowTree::default()
        .with_children(0, [1, 2, 3])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_style(
            0,
            NodeInputOf {
                size: Size::new(
                    DimensionOf::px(scalar(100.0)),
                    DimensionOf::px(scalar(80.0)),
                ),
                flex_wrap: FlexWrap::Wrap,
                gap: Size::new(LengthOf::ZERO, LengthOf::percent(scalar(0.1))),
                align_content: Some(AlignContent::FlexStart),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, flex_item_style(scalar(60.0)))
        .with_style(2, flex_item_style(scalar(60.0)))
        .with_style(3, flex_item_style(scalar(40.0)));
    let horizontal_batch = compute_layout(
        &horizontal,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(100.0)),
            AvailableOf::definite(scalar(80.0)),
        ))
        .expect("valid viewport request"),
    )
    .expect("horizontal non-leaf flex root layout succeeds");

    assert_eq!(
        public_flow_output(horizontal_batch.final_entries(), 1).size,
        Size::new(scalar(100.0), scalar(10.0))
    );
    assert_eq!(
        public_flow_output(horizontal_batch.final_entries(), 2).size,
        Size::new(scalar(60.0), scalar(10.0))
    );
    assert_eq!(
        public_flow_output(horizontal_batch.final_entries(), 3).size,
        Size::new(scalar(40.0), scalar(10.0))
    );
    assert_eq!(
        public_flow_output(horizontal_batch.final_entries(), 0)
            .content_size
            .height,
        scalar(28.0)
    );
}

#[test]
fn logical_flex_sizing_preserves_horizontal_and_child_flow_ownership_for_f32() {
    assert_logical_flex_sizing_preserves_horizontal_and_child_flow_ownership::<f32>();
}

#[test]
fn logical_flex_sizing_preserves_horizontal_and_child_flow_ownership_for_f64() {
    assert_logical_flex_sizing_preserves_horizontal_and_child_flow_ownership::<f64>();
}

fn assert_logical_flex_public_contexts<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let vertical_containing_flow = FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl);
    let horizontal_containing_flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);

    // A non-leaf flex root passes its own vertical containing flow to children
    // whose own flows differ, so percentage sizing remains owned by the parent.
    assert_logical_flex_sizing_preserves_horizontal_and_child_flow_ownership::<S>();

    let flex_root = PublicFlowTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.0, 20.0))
        .with_style(2, logical_flex_leaf(10.0, 20.0));
    let viewport = Size::splat(AvailableOf::definite(scalar(100.0)));
    let flex_root_batch = compute_layout(
        &flex_root,
        0,
        LayoutRootRequestOf::flex_item_under_viewport(
            viewport,
            FlexItemRootContextOf::under_viewport(viewport)
                .expect("valid flex item root viewport context"),
        )
        .expect("valid flex item root request"),
    )
    .expect("public flex item root layout succeeds");
    assert_eq!(
        public_flow_output(flex_root_batch.final_entries(), 0).location,
        Point::ZERO
    );
    assert_eq!(
        public_flow_output(flex_root_batch.final_entries(), 0).size,
        Size::splat(scalar(100.0))
    );
    assert_eq!(
        public_flow_output(flex_root_batch.final_entries(), 1).location,
        Point::new(S::ZERO, S::ZERO)
    );
    assert_eq!(
        public_flow_output(flex_root_batch.final_entries(), 2).location,
        Point::new(S::ZERO, scalar(20.0))
    );

    let cache_tree = |writing_mode, direction| {
        PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    writing_mode,
                    direction,
                    size: Size::splat(DimensionOf::px(scalar(100.0))),
                    flex_direction: FlexDirection::Row,
                    ..NodeInputOf::default()
                },
            )
            .with_style(1, logical_flex_leaf(10.25, 20.25))
    };
    let cache_request =
        LayoutRootRequestOf::viewport(viewport).expect("valid cache viewport request");
    let vertical_cache_tree = cache_tree(WritingMode::VerticalLr, Direction::Rtl);
    let cold_cache_batch = compute_layout(&vertical_cache_tree, 0, cache_request)
        .expect("cold non-leaf flex cache traversal succeeds");
    let cold_child_entry = cold_cache_batch
        .cache_store_entries()
        .iter()
        .find(|entry| entry.node() == 1 && entry.input().run_mode() == RunMode::PerformLayout)
        .expect("cold flex traversal stages the child final-layout cache output");
    assert_eq!(
        cold_child_entry.input().containing_flow_axes(),
        vertical_containing_flow
    );
    assert_eq!(
        cold_child_entry.output().size,
        Size::new(scalar(10.25), scalar(20.25))
    );
    assert_eq!(cold_child_entry.output().content_size, Size::ZERO);
    assert_eq!(
        public_flow_output(cold_cache_batch.final_entries(), 1),
        NodeOutputOf {
            location: Point::new(S::ZERO, scalar(80.0)),
            size: Size::new(scalar(10.0), scalar(20.0)),
            ..NodeOutputOf::with_order(0)
        }
    );

    vertical_cache_tree.apply_cache_entries(cold_cache_batch.cache_store_entries());
    vertical_cache_tree.clear_cache_inputs();
    let warm_cache_batch = compute_layout(&vertical_cache_tree, 0, cache_request)
        .expect("matching public flex cache traversal succeeds");
    assert!(
        vertical_cache_tree
            .cache_inputs(1)
            .iter()
            .any(|input| *input == *cold_child_entry.input())
    );
    assert!(
        warm_cache_batch.cache_store_entries().iter().all(|entry| {
            entry.node() != 1 || entry.input().run_mode() != RunMode::PerformLayout
        })
    );
    assert_eq!(
        public_flow_output(warm_cache_batch.final_entries(), 1),
        public_flow_output(cold_cache_batch.final_entries(), 1)
    );

    let horizontal_cache_tree = cache_tree(WritingMode::HorizontalTb, Direction::Ltr);
    horizontal_cache_tree.apply_cache_entries(&[*cold_child_entry]);
    let distinct_flow_batch = compute_layout(&horizontal_cache_tree, 0, cache_request)
        .expect("distinct-flow public flex cache traversal succeeds");
    assert!(
        horizontal_cache_tree
            .cache_inputs(1)
            .iter()
            .any(|input| input.containing_flow_axes() == horizontal_containing_flow)
    );
    assert!(
        distinct_flow_batch
            .cache_store_entries()
            .iter()
            .any(|entry| {
                entry.node() == 1
                    && entry.input().run_mode() == RunMode::PerformLayout
                    && entry.input().containing_flow_axes() == horizontal_containing_flow
            })
    );

    let hidden = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [2])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                size: Size::splat(DimensionOf::px(scalar(100.0))),
                flex_direction: FlexDirection::Row,
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::None,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                ..NodeInputOf::default()
            },
        )
        .with_style(2, logical_flex_leaf(20.0, 10.0));
    let hidden_batch = compute_layout(
        &hidden,
        0,
        LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
    )
    .expect("hidden flex descendant layout succeeds");
    assert_eq!(
        hidden_batch
            .cache_clear_entries()
            .iter()
            .map(|entry| entry.node())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    for node in [1, 2] {
        assert_eq!(
            public_flow_output(hidden_batch.unrounded_entries(), node),
            NodeOutputOf::with_order(0)
        );
        assert_eq!(
            public_flow_output(hidden_batch.final_entries(), node),
            NodeOutputOf::with_order(0)
        );
    }

    let fractional = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalLr,
                size: Size::splat(DimensionOf::px(scalar(100.5))),
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::FlexEnd),
                justify_content: Some(AlignContent::FlexEnd),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, logical_flex_leaf(10.25, 20.25));
    let fractional_batch = compute_layout(
        &fractional,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.5))))
            .expect("valid fractional viewport request"),
    )
    .expect("fractional non-horizontal flex layout succeeds");
    assert_eq!(
        public_flow_output(fractional_batch.unrounded_entries(), 1).location,
        Point::new(scalar(90.25), scalar(80.25))
    );
    assert_eq!(
        public_flow_output(fractional_batch.unrounded_entries(), 1).size,
        Size::new(scalar(10.25), scalar(20.25))
    );
    assert_eq!(
        public_flow_output(fractional_batch.final_entries(), 1).location,
        Point::new(scalar(90.0), scalar(80.0))
    );
    assert_eq!(
        public_flow_output(fractional_batch.final_entries(), 1).size,
        Size::new(scalar(11.0), scalar(21.0))
    );
}

#[test]
fn logical_flex_public_contexts_preserve_flow_and_physical_output_for_f32() {
    assert_logical_flex_public_contexts::<f32>();
}

#[test]
fn logical_flex_public_contexts_preserve_flow_and_physical_output_for_f64() {
    assert_logical_flex_public_contexts::<f64>();
}

fn assert_viewport_root_logical_inline_auto_fill<S: LayoutScalar>(
    writing_mode: WritingMode,
    expected_location: Point<S>,
) {
    let tree = FlowRootLeafTree::new(NodeInputOf::<S> {
        writing_mode,
        size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::AUTO),
        ..NodeInputOf::default()
    });
    let viewport = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );
    let request = LayoutRootRequestOf::viewport(viewport).expect("valid viewport request");

    let batch = compute_layout(&tree, 0, request).expect("root layout succeeds");
    let output = single_final_output(&batch);

    assert_eq!(output.size, Size::new(scalar(20.0), scalar(110.0)));
    assert_eq!(output.location, expected_location);
}

fn assert_horizontal_viewport_root_logical_inline_auto_fill<S: LayoutScalar>() {
    let tree = FlowRootLeafTree::new(NodeInputOf::<S> {
        writing_mode: WritingMode::HorizontalTb,
        size: Size::new(DimensionOf::AUTO, DimensionOf::px(scalar(30.0))),
        ..NodeInputOf::default()
    });
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    ))
    .expect("valid viewport request");

    let batch = compute_layout(&tree, 0, request).expect("root layout succeeds");
    let output = single_final_output(&batch);

    assert_eq!(output.size, Size::new(scalar(70.0), scalar(30.0)));
    assert_eq!(output.location, Point::ZERO);
}

#[test]
fn root_flow_logical_inline_auto_fill_and_start_placement_work_for_f32() {
    assert_horizontal_viewport_root_logical_inline_auto_fill::<f32>();
    assert_viewport_root_logical_inline_auto_fill::<f32>(
        WritingMode::VerticalRl,
        Point::new(50.0, 0.0),
    );
    assert_viewport_root_logical_inline_auto_fill::<f32>(
        WritingMode::SidewaysLr,
        Point::new(0.0, 0.0),
    );
}

#[test]
fn root_flow_logical_inline_auto_fill_and_start_placement_work_for_f64() {
    assert_horizontal_viewport_root_logical_inline_auto_fill::<f64>();
    assert_viewport_root_logical_inline_auto_fill::<f64>(
        WritingMode::VerticalRl,
        Point::new(50.0, 0.0),
    );
    assert_viewport_root_logical_inline_auto_fill::<f64>(
        WritingMode::SidewaysLr,
        Point::new(0.0, 0.0),
    );
}

fn root_writing_mode_directions() -> [(WritingMode, Direction); 10] {
    [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::HorizontalTb, Direction::Rtl),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Rtl),
        (WritingMode::VerticalLr, Direction::Ltr),
        (WritingMode::VerticalLr, Direction::Rtl),
        (WritingMode::SidewaysRl, Direction::Ltr),
        (WritingMode::SidewaysRl, Direction::Rtl),
        (WritingMode::SidewaysLr, Direction::Ltr),
        (WritingMode::SidewaysLr, Direction::Rtl),
    ]
}

fn assert_ordinary_block_root_contexts<S: LayoutScalar>() {
    let viewport = Size::new(
        AvailableOf::definite(scalar::<S>(100.0)),
        AvailableOf::definite(scalar::<S>(100.0)),
    );
    let logical_size = crate::geometry::LogicalSizeOf::new(scalar::<S>(20.0), scalar::<S>(10.0));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let size = flow_axes.physical_size(logical_size);
        let style = NodeInputOf::<S> {
            display: Display::Block,
            writing_mode,
            direction,
            size: size.map(DimensionOf::px),
            ..NodeInputOf::default()
        };

        let viewport_tree = FlowRootLeafTree::new(style.clone());
        let viewport_batch = compute_layout(
            &viewport_tree,
            0,
            LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
        )
        .expect("viewport root layout succeeds");
        let viewport_output = single_final_output(&viewport_batch);
        assert_eq!(viewport_output.size, size);
        assert_eq!(
            viewport_output.location,
            flow_axes.physical_point(
                crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                logical_size,
                Size::new(scalar::<S>(100.0), scalar::<S>(100.0)),
            )
        );

        let flex_tree = FlowRootLeafTree::new(style);
        let flex_batch = compute_layout(
            &flex_tree,
            0,
            LayoutRootRequestOf::flex_item_under_viewport(
                viewport,
                FlexItemRootContextOf::under_viewport(viewport)
                    .expect("valid flex root viewport context"),
            )
            .expect("valid flex root request"),
        )
        .expect("flex root layout succeeds");
        assert_eq!(single_final_output(&flex_batch).size, size);
    }
}

#[test]
fn ordinary_block_root_contexts_preserve_all_flow_mappings_for_f32() {
    assert_ordinary_block_root_contexts::<f32>();
}

#[test]
fn ordinary_block_root_contexts_preserve_all_flow_mappings_for_f64() {
    assert_ordinary_block_root_contexts::<f64>();
}

fn assert_ordinary_block_root_contexts_clear_hidden_descendants<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let viewport = Size::splat(AvailableOf::definite(scalar(100.0)));

    for (writing_mode, direction) in root_writing_mode_directions() {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [2])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::splat(DimensionOf::px(scalar(100.0))),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::None,
                    writing_mode: WritingMode::HorizontalTb,
                    direction: Direction::Ltr,
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::splat(DimensionOf::px(scalar(20.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
        )
        .expect("hidden descendant layout succeeds");

        assert_eq!(
            batch
                .cache_clear_entries()
                .iter()
                .map(|entry| entry.node())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        for node in [1, 2] {
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), node),
                NodeOutputOf::with_order(0)
            );
            assert_eq!(
                public_flow_output(batch.final_entries(), node),
                NodeOutputOf::with_order(0)
            );
        }
    }
}

#[test]
fn ordinary_block_root_contexts_clear_hidden_descendants_for_all_flows_f32() {
    assert_ordinary_block_root_contexts_clear_hidden_descendants::<f32>();
}

#[test]
fn ordinary_block_root_contexts_clear_hidden_descendants_for_all_flows_f64() {
    assert_ordinary_block_root_contexts_clear_hidden_descendants::<f64>();
}

fn fractional_child_rect<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
) -> (Point<S>, Size<S>, Point<S>, Size<S>) {
    let scalar = scalar::<S>;
    match (writing_mode, direction) {
        (WritingMode::HorizontalTb, Direction::Ltr) => (
            Point::ZERO,
            Size::new(scalar(10.25), scalar(20.25)),
            Point::ZERO,
            Size::new(scalar(10.0), scalar(20.0)),
        ),
        (WritingMode::HorizontalTb, Direction::Rtl) => (
            Point::new(scalar(90.25), S::ZERO),
            Size::new(scalar(10.25), scalar(20.25)),
            Point::new(scalar(90.0), S::ZERO),
            Size::new(scalar(11.0), scalar(20.0)),
        ),
        (WritingMode::VerticalRl, Direction::Ltr) | (WritingMode::SidewaysRl, Direction::Ltr) => (
            Point::new(scalar(80.25), S::ZERO),
            Size::new(scalar(20.25), scalar(10.25)),
            Point::new(scalar(80.0), S::ZERO),
            Size::new(scalar(21.0), scalar(10.0)),
        ),
        (WritingMode::VerticalRl, Direction::Rtl) => (
            Point::new(scalar(80.25), scalar(90.25)),
            Size::new(scalar(20.25), scalar(10.25)),
            Point::new(scalar(80.0), scalar(90.0)),
            Size::new(scalar(21.0), scalar(11.0)),
        ),
        (WritingMode::VerticalLr, Direction::Ltr) | (WritingMode::SidewaysLr, Direction::Rtl) => (
            Point::ZERO,
            Size::new(scalar(20.25), scalar(10.25)),
            Point::ZERO,
            Size::new(scalar(20.0), scalar(10.0)),
        ),
        (WritingMode::VerticalLr, Direction::Rtl) | (WritingMode::SidewaysLr, Direction::Ltr) => (
            Point::new(S::ZERO, scalar(90.25)),
            Size::new(scalar(20.25), scalar(10.25)),
            Point::new(S::ZERO, scalar(90.0)),
            Size::new(scalar(20.0), scalar(11.0)),
        ),
        (WritingMode::SidewaysRl, Direction::Rtl) => (
            Point::new(scalar(80.25), scalar(90.25)),
            Size::new(scalar(20.25), scalar(10.25)),
            Point::new(scalar(80.0), scalar(90.0)),
            Size::new(scalar(21.0), scalar(11.0)),
        ),
    }
}

fn assert_ordinary_block_root_contexts_round_fractional_physical_edges<S: LayoutScalar>() {
    let scalar = scalar::<S>;
    let root_size = Size::splat(scalar(100.5));
    let viewport = root_size.map(AvailableOf::definite);

    for (writing_mode, direction) in root_writing_mode_directions() {
        let (
            expected_unrounded_location,
            expected_unrounded_size,
            expected_final_location,
            expected_final_size,
        ) = fractional_child_rect(writing_mode, direction);
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: root_size.map(DimensionOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: expected_unrounded_size.map(DimensionOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
        )
        .expect("fractional root layout succeeds");
        let unrounded_child = public_flow_output(batch.unrounded_entries(), 1);
        let final_root = public_flow_output(batch.final_entries(), 0);
        let final_child = public_flow_output(batch.final_entries(), 1);

        assert_eq!(unrounded_child.location, expected_unrounded_location);
        assert_eq!(unrounded_child.size, expected_unrounded_size);
        assert_eq!(final_child.location, expected_final_location);
        assert_eq!(final_child.size, expected_final_size);
        assert_eq!(final_root.size, Size::splat(scalar(101.0)));
    }
}

#[test]
fn ordinary_block_root_contexts_round_fractional_physical_edges_for_all_flows_f32() {
    assert_ordinary_block_root_contexts_round_fractional_physical_edges::<f32>();
}

#[test]
fn ordinary_block_root_contexts_round_fractional_physical_edges_for_all_flows_f64() {
    assert_ordinary_block_root_contexts_round_fractional_physical_edges::<f64>();
}

fn assert_root_flow_opposite_edge_uses_only_definite_extent<S: LayoutScalar>() {
    let style = NodeInputOf::<S> {
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::px(scalar(30.0))),
        ..NodeInputOf::default()
    };
    let definite_tree = FlowRootLeafTree::new(style.clone());
    let definite_request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    ))
    .expect("valid definite viewport request");
    let definite =
        compute_layout(&definite_tree, 0, definite_request).expect("definite root layout succeeds");
    assert_eq!(
        single_final_output(&definite).location,
        Point::new(scalar(50.0), S::ZERO)
    );

    let intrinsic_tree = FlowRootLeafTree::new(style);
    let intrinsic_request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::MAX_CONTENT,
        AvailableOf::definite(scalar(110.0)),
    ))
    .expect("valid intrinsic viewport request");
    let intrinsic = compute_layout(&intrinsic_tree, 0, intrinsic_request)
        .expect("intrinsic root layout succeeds");
    assert_eq!(single_final_output(&intrinsic).location, Point::ZERO);

    let sideways_style = NodeInputOf::<S> {
        writing_mode: WritingMode::SidewaysLr,
        size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::px(scalar(30.0))),
        ..NodeInputOf::default()
    };
    let sideways_definite_tree = FlowRootLeafTree::new(sideways_style.clone());
    let sideways_definite_request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    ))
    .expect("valid definite sideways viewport request");
    let sideways_definite = compute_layout(&sideways_definite_tree, 0, sideways_definite_request)
        .expect("definite sideways root layout succeeds");
    assert_eq!(
        single_final_output(&sideways_definite).location,
        Point::new(S::ZERO, scalar(80.0))
    );

    let sideways_intrinsic_tree = FlowRootLeafTree::new(sideways_style);
    let sideways_intrinsic_request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::MAX_CONTENT,
    ))
    .expect("valid intrinsic sideways viewport request");
    let sideways_intrinsic =
        compute_layout(&sideways_intrinsic_tree, 0, sideways_intrinsic_request)
            .expect("intrinsic sideways root layout succeeds");
    assert_eq!(
        single_final_output(&sideways_intrinsic).location,
        Point::ZERO
    );
}

#[test]
fn root_flow_opposite_edge_uses_only_definite_extent_for_f32() {
    assert_root_flow_opposite_edge_uses_only_definite_extent::<f32>();
}

#[test]
fn root_flow_opposite_edge_uses_only_definite_extent_for_f64() {
    assert_root_flow_opposite_edge_uses_only_definite_extent::<f64>();
}

fn assert_root_and_flex_root_percentage_edges_use_logical_inline_basis<S: LayoutScalar>() {
    let style = NodeInputOf::<S> {
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::px(scalar(30.0))),
        margin: Edges::all(LengthAutoOf::percent(scalar(0.3))),
        padding: Edges::all(LengthOf::percent(scalar(0.1))),
        border: Edges::all(LengthOf::percent(scalar(0.2))),
        ..NodeInputOf::default()
    };
    let viewport = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );

    let viewport_tree = FlowRootLeafTree::new(style.clone());
    let viewport_batch = compute_layout(
        &viewport_tree,
        0,
        LayoutRootRequestOf::viewport(viewport).expect("valid viewport request"),
    )
    .expect("viewport root layout succeeds");
    let viewport_output = single_final_output(&viewport_batch);
    assert_eq!(viewport_output.margin, Edges::all(scalar(33.0)));
    assert_eq!(viewport_output.padding, Edges::all(scalar(11.0)));
    assert_eq!(viewport_output.border, Edges::all(scalar(22.0)));

    let flex_tree = FlowRootLeafTree::new(style);
    let flex_batch = compute_layout(
        &flex_tree,
        0,
        LayoutRootRequestOf::flex_item_under_viewport(
            Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
            FlexItemRootContextOf::under_viewport(viewport)
                .expect("valid flex root viewport context"),
        )
        .expect("valid flex root request"),
    )
    .expect("flex root layout succeeds");
    let flex_output = single_final_output(&flex_batch);
    assert_eq!(flex_output.location, Point::ZERO);
    assert_eq!(flex_output.margin, Edges::all(scalar(33.0)));
    assert_eq!(flex_output.padding, Edges::all(scalar(11.0)));
    assert_eq!(flex_output.border, Edges::all(scalar(22.0)));
}

#[test]
fn root_flow_percentage_edges_use_vertical_inline_extent_for_f32() {
    assert_root_and_flex_root_percentage_edges_use_logical_inline_basis::<f32>();
}

#[test]
fn root_flow_percentage_edges_use_vertical_inline_extent_for_f64() {
    assert_root_and_flex_root_percentage_edges_use_logical_inline_basis::<f64>();
}

fn assert_flex_root_percentage_parent_is_separate_from_host_fill<S: LayoutScalar>() {
    let host = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );

    for writing_mode in [WritingMode::VerticalRl, WritingMode::SidewaysLr] {
        let style = NodeInputOf::<S> {
            writing_mode,
            size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::AUTO),
            max_size: Size::new(DimensionOf::AUTO, DimensionOf::percent(scalar(0.8))),
            padding: Edges::new(
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
            ),
            border: Edges::new(
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };

        for (viewport_height, expected_height, expected_edge) in
            [(210.0, 110.0, 8.0), (100.0, 80.0, 4.0)]
        {
            let viewport = Size::new(
                AvailableOf::definite(scalar(130.0)),
                AvailableOf::definite(scalar(viewport_height)),
            );
            let tree = FlowRootLeafTree::new(style.clone());
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::flex_item_under_viewport(
                    host,
                    FlexItemRootContextOf::under_viewport(viewport)
                        .expect("valid flex root viewport context"),
                )
                .expect("valid flex root request"),
            )
            .expect("flex root layout succeeds");
            let output = single_final_output(&batch);

            assert_eq!(output.location, Point::ZERO);
            assert_eq!(
                output.size,
                Size::new(scalar(20.0), scalar(expected_height))
            );
            assert_eq!(
                output.padding,
                Edges::new(
                    scalar(expected_edge),
                    S::ZERO,
                    scalar(expected_edge),
                    S::ZERO,
                )
            );
            assert_eq!(
                output.border,
                Edges::new(
                    scalar(expected_edge),
                    S::ZERO,
                    scalar(expected_edge),
                    S::ZERO,
                )
            );
        }
    }
}

#[test]
fn flex_root_percentage_parent_separates_host_fill_for_f32() {
    assert_flex_root_percentage_parent_is_separate_from_host_fill::<f32>();
}

#[test]
fn flex_root_percentage_parent_separates_host_fill_for_f64() {
    assert_flex_root_percentage_parent_is_separate_from_host_fill::<f64>();
}

fn assert_flex_root_flow_known_inline_uses_host_availability<S: LayoutScalar>() {
    let host = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );
    let viewport = Size::new(
        AvailableOf::definite(scalar(130.0)),
        AvailableOf::definite(scalar(210.0)),
    );

    for writing_mode in [WritingMode::VerticalRl, WritingMode::SidewaysLr] {
        let style = NodeInputOf::<S> {
            writing_mode,
            size: Size::new(DimensionOf::px(scalar(20.0)), DimensionOf::AUTO),
            ..NodeInputOf::default()
        };
        let tree = FlowRootLeafTree::new(style.clone());
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::flex_item_under_viewport(
                host,
                FlexItemRootContextOf::under_viewport(viewport)
                    .expect("valid flex root viewport context"),
            )
            .expect("valid flex root request"),
        )
        .expect("flex root layout succeeds");
        let output = single_final_output(&batch);

        assert_eq!(output.size, Size::new(scalar(20.0), scalar(110.0)));
        assert_eq!(output.location, Point::ZERO);

        let unavailable_tree = FlowRootLeafTree::new(style);
        let unavailable = compute_layout(
            &unavailable_tree,
            0,
            LayoutRootRequestOf::flex_item_under_viewport(
                Size::new(
                    AvailableOf::definite(scalar(70.0)),
                    AvailableOf::MAX_CONTENT,
                ),
                FlexItemRootContextOf::under_viewport(viewport)
                    .expect("valid flex root viewport context"),
            )
            .expect("valid intrinsic flex root request"),
        )
        .expect("intrinsic flex root layout succeeds");
        let unavailable_output = single_final_output(&unavailable);

        assert_eq!(unavailable_output.size, Size::new(scalar(20.0), S::ZERO));
        assert_eq!(unavailable_output.location, Point::ZERO);
    }
}

#[test]
fn flex_root_flow_known_inline_uses_host_availability_for_f32() {
    assert_flex_root_flow_known_inline_uses_host_availability::<f32>();
}

#[test]
fn flex_root_flow_known_inline_uses_host_availability_for_f64() {
    assert_flex_root_flow_known_inline_uses_host_availability::<f64>();
}

fn root_cache_input(available: Size<Available>) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformRootLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        available.map(Available::into_option),
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        available,
    )
}

fn assert_public_scroll_geometry_error_without_batch(
    tree: &RootSessionTree,
    available: Size<Available>,
    expected_site: LayoutErrorSite<u32>,
    expected_operation: LayoutOperation,
    expected_invariant: LayoutInternalInvariant,
) {
    let request =
        LayoutRootRequest::viewport(available).expect("finite root availability is valid");
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compute_layout(tree, 0, request)
    }));
    let error = match outcome {
        Ok(Err(error)) => error,
        Ok(Ok(_)) => panic!("scroll-geometry overflow must not return a completed layout batch"),
        Err(_) => panic!("scroll-geometry overflow must not unwind from compute_layout"),
    };

    let expected_kind = LayoutErrorKind::InternalInvariant(expected_invariant);
    assert_eq!(
        (error.site(), error.operation(), error.kind()),
        (expected_site, expected_operation, &expected_kind)
    );
}

fn overflowing_scroll_edges() -> Edges<Length> {
    Edges {
        left: Length::px(f32::MAX),
        ..Edges::all(Length::ZERO)
    }
}

#[test]
fn scroll_geometry_error_maps_root_block_overflow_through_the_public_front_door() {
    let tree: RootSessionTree = RootSessionTree::default().style(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(f32::MAX), Dimension::px(1.0)),
            padding: overflowing_scroll_edges(),
            border: overflowing_scroll_edges(),
            ..NodeInput::default()
        },
    );

    assert_public_scroll_geometry_error_without_batch(
        &tree,
        Size::new(Available::definite(f32::MAX), Available::definite(1.0)),
        LayoutErrorSite::Node(0),
        LayoutOperation::RootLayout,
        LayoutInternalInvariant::InvalidRootScrollGeometry,
    );
}

#[test]
fn scroll_geometry_error_maps_non_root_block_overflow_through_the_public_front_door() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(1.0)),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(f32::MAX), Dimension::px(1.0)),
                padding: overflowing_scroll_edges(),
                border: overflowing_scroll_edges(),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                ..NodeInput::default()
            },
        );

    assert_public_scroll_geometry_error_without_batch(
        &tree,
        Size::new(Available::definite(100.0), Available::definite(1.0)),
        LayoutErrorSite::Node(1),
        LayoutOperation::ChildLayout,
        LayoutInternalInvariant::InvalidBlockScrollGeometry,
    );
}

#[test]
fn scroll_geometry_error_maps_block_inline_float_and_absolute_overflow_to_the_subject() {
    let available = Size::new(Available::definite(100.0), Available::definite(1.0));
    let variants = [
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(f32::MAX), Dimension::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(f32::MAX), Dimension::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            float: Float::Left,
            display: Display::Block,
            size: Size::new(Dimension::px(f32::MAX), Dimension::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            position: Position::Absolute,
            display: Display::Block,
            size: Size::new(Dimension::px(f32::MAX), Dimension::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    ];

    for child_style in variants {
        let tree: RootSessionTree = RootSessionTree::default()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(Dimension::px(100.0), Dimension::px(1.0)),
                    ..NodeInput::default()
                },
            )
            .style(1, child_style)
            .measure(1, Ok(Size::new(f32::MAX, 1.0)));

        assert_public_scroll_geometry_error_without_batch(
            &tree,
            available,
            LayoutErrorSite::ContainerSubject {
                container: 0,
                subject: 1,
            },
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        );
    }
}

#[test]
fn scroll_geometry_error_maps_rounding_overflow_through_the_public_front_door() {
    let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let available = Size::splat(Available::definite(f32::MAX));
    let style = NodeInput {
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(Dimension::px(1.0), Dimension::px(1.0)),
        ..NodeInput::default()
    };
    let mut output = ComputeOutput::from_outer_size(Size::new(1.0, 1.0));
    output.scroll_geometry = Some(
        ScrollGeometry::new(
            flow_axes,
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
            ),
            ScrollRect::new(Point::ZERO, Size::new(1.0, 1.0)).unwrap(),
            Some(ScrollRect::new(Point::ZERO, Size::new(1.0, 1.0)).unwrap()),
            ScrollRect::new(Point::ZERO, Size::new(f32::MAX, 1.0)).unwrap(),
            PhysicalScrollRange::try_new(-f32::MAX, 0.0, 0.0, 0.0).unwrap(),
            ScrollbarGutterRects::new(None, None),
        )
        .unwrap(),
    );
    let mut cache = Cache::new();
    cache.store_with_context(
        &ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            available.map(Available::into_option),
            flow_axes,
            available,
        ),
        CacheKeyContext::new(),
        output,
    );
    let tree: RootSessionTree = RootSessionTree::default().style(0, style);
    tree.caches.borrow_mut().insert(0, cache);

    assert_public_scroll_geometry_error_without_batch(
        &tree,
        available,
        LayoutErrorSite::Node(0),
        LayoutOperation::RoundingFinalization,
        LayoutInternalInvariant::InvalidRoundedScrollGeometry,
    );
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
            PhysicalAxis::Horizontal,
            NonNegativeFiniteScalarErrorOf::Negative { value: -1.0 },
        ),
        (
            Size::new(Available::definite(f32::NAN), Available::MAX_CONTENT),
            PhysicalAxis::Horizontal,
            NonNegativeFiniteScalarErrorOf::NonFinite { value: f32::NAN },
        ),
        (
            Size::new(Available::MAX_CONTENT, Available::definite(f32::INFINITY)),
            PhysicalAxis::Vertical,
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
    assert_eq!(error.axis(), PhysicalAxis::Horizontal);
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

fn assert_logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow<
    S: LayoutScalar,
>() {
    #[derive(Default)]
    struct HiddenTree<S: LayoutScalar> {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutputOf<S>>,
        caches: HashMap<u32, CacheOf<S>>,
        styles: HashMap<u32, NodeInputOf<S>>,
        calls: Vec<(u32, ComputeInputOf<S>)>,
        cache_get_calls: Cell<usize>,
        cache_store_calls: usize,
    }

    impl<S: LayoutScalar> Traverse for HiddenTree<S> {
        type Node = u32;
        type Scalar = S;
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

    impl<S: LayoutScalar> Compute for HiddenTree<S> {
        fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInputOf<S>,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            let expected_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
            assert_eq!(input, ComputeInputOf::hidden(expected_axes));
            self.calls.push((node, input));
            compute_hidden(self, node, input.containing_flow_axes())
        }
    }

    impl<S: LayoutScalar> CacheAccess for HiddenTree<S> {
        type Node = u32;
        type Scalar = S;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInputOf<S>,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutputOf<S>> {
            self.cache_get_calls.set(self.cache_get_calls.get() + 1);
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInputOf<S>,
            context: crate::CacheKeyContext,
            output: ComputeOutputOf<S>,
        ) {
            self.cache_store_calls += 1;
            self.caches
                .get_mut(&node)
                .expect("test hidden node cache exists")
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![3]);
    tree.children.insert(3, vec![]);
    for node in [1, 2, 3] {
        tree.styles.insert(node, NodeInputOf::default());
        tree.caches.insert(node, CacheOf::new());
        tree.caches.get_mut(&node).unwrap().store_with_context(
            &ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::splat(Some(scalar::<S>(1.0))),
                Size::NONE,
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::splat(AvailableOf::MAX_CONTENT),
            ),
            CacheKeyContext::new(),
            ComputeOutputOf::from_outer_size(Size::splat(scalar::<S>(1.0))),
        );
    }

    let expected_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let expected_input = ComputeInputOf::hidden(expected_axes);
    assert_eq!(
        compute_hidden(&mut tree, 1, expected_axes).unwrap(),
        ComputeOutputOf::HIDDEN
    );
    assert_eq!(tree.calls, vec![(2, expected_input), (3, expected_input)]);
    for node in [1, 2, 3] {
        assert_eq!(tree.layouts[&node], NodeOutputOf::with_order(0));
        assert!(tree.caches[&node].is_empty());
    }
    assert_eq!(tree.cache_get_calls.get(), 0);
    assert_eq!(tree.cache_store_calls, 0);
}

#[test]
fn logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow_for_f32() {
    assert_logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow::<f32>();
}

#[test]
fn logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow_for_f64() {
    assert_logical_flex_public_contexts_hidden_layout_recurses_with_containing_flow::<f64>();
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
                assert_eq!(
                    input,
                    ComputeInput::hidden(crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ))
                );
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

    assert_eq!(
        compute_hidden(
            &mut tree,
            1,
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        )
        .unwrap(),
        ComputeOutput::HIDDEN
    );
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
                assert_eq!(
                    input,
                    ComputeInput::hidden(crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ))
                );
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

    assert_eq!(
        compute_hidden(
            &mut tree,
            1,
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        )
        .unwrap(),
        ComputeOutput::HIDDEN
    );
    assert_eq!(tree.hidden_children, vec![2]);
    assert_eq!(tree.layouts[&1], NodeOutput::with_order(0));
    assert_eq!(tree.layouts[&3], NodeOutput::with_order(0));
    assert!(tree.caches[&1].is_empty());
    assert!(tree.caches[&3].is_empty());
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
    assert_positive_physical_range(geometry.physical_range(), Size::new(40.0, 40.0));
    assert_eq!(
        geometry
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(99.0, -5.0).unwrap()),
        PhysicalScrollOffset::try_new(40.0, 0.0).unwrap()
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
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
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
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
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
    assert_positive_physical_range(geometry.physical_range(), Size::new(48.0, 38.0));
    assert_eq!(
        geometry
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(99.0, 99.0).unwrap()),
        PhysicalScrollOffset::try_new(48.0, 38.0).unwrap()
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
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
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
    assert_positive_physical_range(geometry.physical_range(), Size::new(48.0, 30.0));
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
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
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
    assert_positive_physical_range(geometry.physical_range(), Size::new(20.0, 30.0));
}

#[test]
fn round_layout_diagnostics_rejects_invalid_rounded_scroll_geometry() {
    let scrollable_overflow = ScrollRect::new(Point::new(f32::MAX, 0.0), Size::ZERO).unwrap();
    let scroll_geometry = crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
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
        Some(ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(200.0), None),
            Size::new(Some(200.0), Some(100.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Rtl),
            Size::new(Available::definite(200.0), Available::definite(100.0))
        ))
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
        tree.input.expect("root should be computed").known(),
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
                let width = input.known().width.unwrap_or(272.0);
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
        tree.input.expect("root should be computed").known(),
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
                    Size::new(input.known().width.unwrap_or(112.0), 20.0),
                    Size::new(input.known().width.unwrap_or(112.0), 20.0),
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
        tree.input.expect("root should be computed").known().width,
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
