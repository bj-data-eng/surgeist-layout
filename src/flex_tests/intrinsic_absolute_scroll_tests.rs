use super::fixtures::{
    FlexTree, assert_fri07_c02_composition_finite_output, computed_overflow,
    fri05_c04_assert_flow_range, fri05_c04_flex_all_flow_axes, fri05_c04_flex_input,
    fri05_c04_flex_overflow_at_flow_axes, fri07_c01_composition_output,
    fri07_c02_collapse_round_request,
};
use super::*;

type RecursiveTree = OracleTree;
fn fri07_c01_absolute_auto_margin_layout<S: LayoutScalar>(
    flow_axes: FlowAxes,
    mut container: NodeInputOf<S>,
    child: NodeInputOf<S>,
) -> NodeOutputOf<S> {
    container.display = Display::Flex;
    container.writing_mode = flow_axes.writing_mode();
    container.direction = flow_axes.direction();
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(1, container)
        .style(2, child);
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(300.0))))
            .expect("absolute auto-margin viewport is finite"),
    )
    .expect("absolute auto-margin layout succeeds");

    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 2)
        .expect("absolute flex child is published")
        .output()
}

fn assert_fri07_c01_absolute_auto_margin_auto_inset_zeroes_axis<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    let output = fri07_c01_absolute_auto_margin_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        NodeInputOf {
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(100.0)),
                PreferredSizeOf::px(S::from_f64(40.0)),
            ),
            ..NodeInputOf::default()
        },
        NodeInputOf {
            position: Position::Absolute,
            inset: Edges {
                left: px(0.0),
                top: px(0.0),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(20.0)),
                PreferredSizeOf::px(S::from_f64(10.0)),
            ),
            margin: Edges {
                top: LengthAutoOf::AUTO,
                right: LengthAutoOf::AUTO,
                bottom: LengthAutoOf::AUTO,
                left: LengthAutoOf::AUTO,
            },
            ..NodeInputOf::default()
        },
    );

    assert_eq!(output.margin, Edges::ZERO);
    assert_eq!(output.location, Point::ZERO);
}

#[test]
fn fri07_c01_absolute_auto_margin_auto_inset_zeroes_used_margins_in_both_scalar_lanes() {
    assert_fri07_c01_absolute_auto_margin_auto_inset_zeroes_axis::<f32>();
    assert_fri07_c01_absolute_auto_margin_auto_inset_zeroes_axis::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_start_auto_inset_matrix<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    let preferred_px = |value| PreferredSizeOf::px(S::from_f64(value));
    let container = || NodeInputOf {
        size: Size::new(preferred_px(100.0), preferred_px(40.0)),
        ..NodeInputOf::default()
    };

    for (name, inset, margin, expected_margin, expected_location) in [
        (
            "horizontal start auto",
            Edges {
                top: px(0.0),
                right: px(11.0),
                bottom: px(0.0),
                left: LengthAutoOf::AUTO,
            },
            Edges {
                top: px(0.0),
                right: px(7.0),
                bottom: px(0.0),
                left: LengthAutoOf::AUTO,
            },
            Edges::new(S::ZERO, S::from_f64(7.0), S::ZERO, S::ZERO),
            Point::new(S::from_f64(62.0), S::ZERO),
        ),
        (
            "vertical start auto",
            Edges {
                top: LengthAutoOf::AUTO,
                right: px(0.0),
                bottom: px(5.0),
                left: px(0.0),
            },
            Edges {
                top: LengthAutoOf::AUTO,
                right: px(0.0),
                bottom: px(9.0),
                left: px(0.0),
            },
            Edges::new(S::ZERO, S::ZERO, S::from_f64(9.0), S::ZERO),
            Point::new(S::ZERO, S::from_f64(16.0)),
        ),
    ] {
        let output = fri07_c01_absolute_auto_margin_layout(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            container(),
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: Size::new(preferred_px(20.0), preferred_px(10.0)),
                margin,
                ..NodeInputOf::default()
            },
        );

        assert_eq!(output.margin, expected_margin, "{name} used margins");
        assert_eq!(output.location, expected_location, "{name} placement");
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_start_auto_insets_zero_only_auto_margins() {
    assert_fri07_c01_absolute_auto_margin_start_auto_inset_matrix::<f32>();
    assert_fri07_c01_absolute_auto_margin_start_auto_inset_matrix::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_definite_inset_matrix<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    let preferred_px = |value| PreferredSizeOf::px(S::from_f64(value));
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let container = || NodeInputOf {
        size: Size::new(preferred_px(100.0), preferred_px(40.0)),
        ..NodeInputOf::default()
    };
    let inset = Edges {
        top: px(0.0),
        right: px(20.0),
        bottom: px(0.0),
        left: px(10.0),
    };

    for (name, width, left, right, expected_left, expected_right, expected_x) in [
        ("no auto", 20.0, px(3.0), px(5.0), 3.0, 5.0, 13.0),
        (
            "one auto positive",
            20.0,
            LengthAutoOf::AUTO,
            px(5.0),
            45.0,
            5.0,
            55.0,
        ),
        (
            "one auto negative",
            80.0,
            LengthAutoOf::AUTO,
            px(5.0),
            -15.0,
            5.0,
            -5.0,
        ),
        (
            "two auto positive",
            20.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            25.0,
            25.0,
            35.0,
        ),
        (
            "two auto zero",
            70.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            0.0,
            0.0,
            10.0,
        ),
        (
            "two auto negative inline",
            100.0,
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            0.0,
            -30.0,
            10.0,
        ),
    ] {
        let output = fri07_c01_absolute_auto_margin_layout(
            flow_axes,
            container(),
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: Size::new(preferred_px(width), preferred_px(10.0)),
                margin: Edges {
                    top: px(0.0),
                    right,
                    bottom: px(0.0),
                    left,
                },
                ..NodeInputOf::default()
            },
        );

        assert_eq!(
            output.margin.left,
            S::from_f64(expected_left),
            "{name} left"
        );
        assert_eq!(
            output.margin.right,
            S::from_f64(expected_right),
            "{name} right"
        );
        assert_eq!(output.location.x, S::from_f64(expected_x), "{name} x");
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_definite_insets_use_signed_inset_modified_space() {
    assert_fri07_c01_absolute_auto_margin_definite_inset_matrix::<f32>();
    assert_fri07_c01_absolute_auto_margin_definite_inset_matrix::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_flow_mapping<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let container_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            S::from_f64(100.0),
            S::from_f64(60.0),
        ));
        let child_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            S::from_f64(120.0),
            S::from_f64(20.0),
        ));
        let inset = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            px(0.0),
            px(0.0),
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
        ));
        let margin = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            px(0.0),
            px(0.0),
        ));
        let output = fri07_c01_absolute_auto_margin_layout(
            flow_axes,
            NodeInputOf {
                size: container_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: child_size.map(PreferredSizeOf::px),
                margin,
                ..NodeInputOf::default()
            },
        );
        let logical_margin = flow_axes.logical_edges(output.margin);

        assert_eq!(
            logical_margin.inline_start,
            S::ZERO,
            "{flow_axes:?} inline start"
        );
        assert_eq!(
            logical_margin.inline_end,
            S::from_f64(-20.0),
            "{flow_axes:?} inline end"
        );
        match flow_axes.inline_start() {
            PhysicalSide::Left => assert_eq!(output.location.x, S::ZERO, "{flow_axes:?} x"),
            PhysicalSide::Right => {
                assert_eq!(output.location.x, S::from_f64(-20.0), "{flow_axes:?} x")
            }
            PhysicalSide::Top => assert_eq!(output.location.y, S::ZERO, "{flow_axes:?} y"),
            PhysicalSide::Bottom => {
                assert_eq!(output.location.y, S::from_f64(-20.0), "{flow_axes:?} y")
            }
        }
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_negative_inline_space_uses_containing_flow_start() {
    assert_fri07_c01_absolute_auto_margin_flow_mapping::<f32>();
    assert_fri07_c01_absolute_auto_margin_flow_mapping::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_negative_block_space_divides<S: LayoutScalar>() {
    let px = |value| LengthAutoOf::px(S::from_f64(value));
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let container_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            S::from_f64(60.0),
            S::from_f64(100.0),
        ));
        let child_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            S::from_f64(20.0),
            S::from_f64(120.0),
        ));
        let inset = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
            px(0.0),
            px(0.0),
        ));
        let margin = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            px(0.0),
            px(0.0),
            LengthAutoOf::AUTO,
            LengthAutoOf::AUTO,
        ));
        let output = fri07_c01_absolute_auto_margin_layout(
            flow_axes,
            NodeInputOf {
                size: container_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: child_size.map(PreferredSizeOf::px),
                margin,
                ..NodeInputOf::default()
            },
        );
        let logical_margin = flow_axes.logical_edges(output.margin);

        assert_eq!(
            logical_margin.block_start,
            S::from_f64(-10.0),
            "{flow_axes:?} block start"
        );
        assert_eq!(
            logical_margin.block_end,
            S::from_f64(-10.0),
            "{flow_axes:?} block end"
        );
        match flow_axes.block_axis() {
            PhysicalAxis::Horizontal => {
                assert_eq!(output.location.x, S::from_f64(-10.0), "{flow_axes:?} x")
            }
            PhysicalAxis::Vertical => {
                assert_eq!(output.location.y, S::from_f64(-10.0), "{flow_axes:?} y")
            }
        }
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_negative_block_space_divides_normally() {
    assert_fri07_c01_absolute_auto_margin_negative_block_space_divides::<f32>();
    assert_fri07_c01_absolute_auto_margin_negative_block_space_divides::<f64>();
}

fn assert_fri07_c01_absolute_auto_margin_padding_border_box_sizing<S: LayoutScalar>() {
    let length = |value| LengthOf::px(S::from_f64(value));
    let auto_length = |value| LengthAutoOf::px(S::from_f64(value));
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let container = || NodeInputOf {
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(preferred(120.0), preferred(80.0)),
        padding: Edges::new(length(10.0), length(10.0), length(10.0), length(10.0)),
        border: Edges::new(length(5.0), length(5.0), length(5.0), length(5.0)),
        ..NodeInputOf::default()
    };
    let child_edges = Edges::new(length(1.0), length(4.0), length(1.0), length(2.0));
    let child_padding = Edges::new(length(1.0), length(7.0), length(1.0), length(3.0));
    let inset = Edges {
        top: auto_length(0.0),
        right: auto_length(20.0),
        bottom: auto_length(0.0),
        left: auto_length(10.0),
    };
    let auto_inline_margin = Edges {
        top: LengthAutoOf::ZERO,
        right: LengthAutoOf::AUTO,
        bottom: LengthAutoOf::ZERO,
        left: LengthAutoOf::AUTO,
    };

    for (box_sizing, expected_size, expected_margin, expected_x) in [
        (BoxSizing::ContentBox, 36.0, 22.0, 37.0),
        (BoxSizing::BorderBox, 20.0, 30.0, 45.0),
    ] {
        let output = fri07_c01_absolute_auto_margin_layout(
            flow_axes,
            container(),
            NodeInputOf {
                position: Position::Absolute,
                box_sizing,
                inset,
                size: Size::new(preferred(20.0), preferred(10.0)),
                padding: child_padding,
                border: child_edges,
                margin: auto_inline_margin,
                ..NodeInputOf::default()
            },
        );

        assert_eq!(
            output.size.width,
            S::from_f64(expected_size),
            "{box_sizing:?} width"
        );
        assert_eq!(
            output.margin.left,
            S::from_f64(expected_margin),
            "{box_sizing:?} left"
        );
        assert_eq!(
            output.margin.right,
            S::from_f64(expected_margin),
            "{box_sizing:?} right"
        );
        assert_eq!(
            output.location.x,
            S::from_f64(expected_x),
            "{box_sizing:?} x"
        );
    }
}

#[test]
fn fri07_c01_absolute_auto_margin_uses_containing_padding_box_and_used_border_box() {
    assert_fri07_c01_absolute_auto_margin_padding_border_box_sizing::<f32>();
    assert_fri07_c01_absolute_auto_margin_padding_border_box_sizing::<f64>();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C01IntrinsicMeasureError {
    ProviderFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C01IntrinsicMeasureMode {
    Values,
    ProviderFailure,
    NonFinite,
}

#[derive(Clone, Debug)]
struct Fri07C01IntrinsicTree<S: LayoutScalar> {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInputOf<S>>,
    measured_nodes: Vec<u32>,
    leaf_requests: RefCell<HashMap<u32, Vec<LeafMeasureInputOf<S>>>>,
    mode: Fri07C01IntrinsicMeasureMode,
}

impl<S: LayoutScalar> Fri07C01IntrinsicTree<S> {
    fn new(mode: Fri07C01IntrinsicMeasureMode) -> Self {
        Self {
            children: HashMap::new(),
            styles: HashMap::new(),
            measured_nodes: Vec::new(),
            leaf_requests: RefCell::new(HashMap::new()),
            mode,
        }
    }

    fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    fn style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.styles.insert(node, style);
        self
    }

    fn measured(mut self, node: u32) -> Self {
        self.measured_nodes.push(node);
        self
    }

    fn leaf_requests(&self, node: u32) -> Vec<LeafMeasureInputOf<S>> {
        self.leaf_requests
            .borrow()
            .get(&node)
            .cloned()
            .unwrap_or_default()
    }
}

impl<S: LayoutScalar> Traverse for Fri07C01IntrinsicTree<S> {
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

impl<S: LayoutScalar> LayoutTree for Fri07C01IntrinsicTree<S> {
    type MeasureError = Fri07C01IntrinsicMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        self.styles
            .get(&node)
            .unwrap_or_else(|| panic!("intrinsic test node {node} must have style"))
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.measured_nodes.contains(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.leaf_requests
            .borrow_mut()
            .entry(node)
            .or_default()
            .push(input);
        match self.mode {
            Fri07C01IntrinsicMeasureMode::ProviderFailure => {
                Some(Err(Fri07C01IntrinsicMeasureError::ProviderFailure))
            }
            Fri07C01IntrinsicMeasureMode::NonFinite => {
                Some(Ok(Size::new(Self::Scalar::INFINITY, Self::Scalar::ONE)))
            }
            Fri07C01IntrinsicMeasureMode::Values => {
                let available = input.available_content_size();
                let intrinsic = |available| match available {
                    MeasurementAvailableOf::MinContent => Some(Self::Scalar::from_f64(20.0)),
                    MeasurementAvailableOf::MaxContent => Some(Self::Scalar::from_f64(100.0)),
                    MeasurementAvailableOf::Definite(_) => None,
                };
                Some(Ok(Size::new(
                    intrinsic(available.width).unwrap_or(Self::Scalar::from_f64(10.0)),
                    intrinsic(available.height).unwrap_or(Self::Scalar::from_f64(10.0)),
                )))
            }
        }
    }
}

fn fri07_c01_intrinsic_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .expect("intrinsic public layout publishes the requested node")
        .output()
}

fn fri07_c01_intrinsic_leaf_tree<S: LayoutScalar>(
    direction: FlexDirection,
    child_writing_mode: WritingMode,
    mode: Fri07C01IntrinsicMeasureMode,
) -> Fri07C01IntrinsicTree<S> {
    let container_size = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(200.0)),
            PreferredSizeOf::px(S::from_f64(40.0)),
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(40.0)),
            PreferredSizeOf::px(S::from_f64(200.0)),
        ),
    };
    let preferred = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(77.0)),
            PreferredSizeOf::AUTO,
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSizeOf::AUTO,
            PreferredSizeOf::px(S::from_f64(77.0)),
        ),
    };

    Fri07C01IntrinsicTree::new(mode)
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                flex_direction: direction,
                size: container_size,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode: child_writing_mode,
                size: preferred.clone(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::MIN_CONTENT,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                writing_mode: child_writing_mode,
                size: preferred,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::MAX_CONTENT,
                ..NodeInputOf::default()
            },
        )
        .measured(2)
        .measured(3)
}

fn fri07_c01_intrinsic_child_container_tree<S: LayoutScalar>(
    direction: FlexDirection,
    child_writing_mode: WritingMode,
) -> Fri07C01IntrinsicTree<S> {
    let container_size = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(200.0)),
            PreferredSizeOf::px(S::from_f64(40.0)),
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(40.0)),
            PreferredSizeOf::px(S::from_f64(200.0)),
        ),
    };
    let preferred = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            PreferredSizeOf::px(S::from_f64(77.0)),
            PreferredSizeOf::AUTO,
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSizeOf::AUTO,
            PreferredSizeOf::px(S::from_f64(77.0)),
        ),
    };

    Fri07C01IntrinsicTree::new(Fri07C01IntrinsicMeasureMode::Values)
        .children(1, [2, 3])
        .children(2, [4])
        .children(3, [5])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                flex_direction: direction,
                size: container_size,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: child_writing_mode,
                size: preferred.clone(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::MIN_CONTENT,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Block,
                writing_mode: child_writing_mode,
                size: preferred,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis: FlexBasisOf::MAX_CONTENT,
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                writing_mode: child_writing_mode,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                writing_mode: child_writing_mode,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .measured(4)
        .measured(5)
}

fn fri07_c01_intrinsic_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(300.0))))
        .expect("intrinsic test viewport is finite")
}

fn fri07_c01_intrinsic_recomputation_tree<S: LayoutScalar>(
    flex_basis: FlexBasisOf<S>,
) -> Fri07C01IntrinsicTree<S> {
    Fri07C01IntrinsicTree::new(Fri07C01IntrinsicMeasureMode::Values)
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(200.0)),
                    PreferredSizeOf::px(S::from_f64(40.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(77.0)),
                    PreferredSizeOf::AUTO,
                ),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_basis,
                flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow"),
                ..NodeInputOf::default()
            },
        )
        .measured(2)
}

fn assert_fri07_c01_intrinsic_provider_constraint_survives_recomputation<S: LayoutScalar>() {
    let scenarios = [
        (
            FlexBasisOf::<S>::MIN_CONTENT,
            MeasurementAvailableOf::MIN_CONTENT,
        ),
        (
            FlexBasisOf::<S>::MAX_CONTENT,
            MeasurementAvailableOf::MAX_CONTENT,
        ),
    ];
    let observed = scenarios
        .iter()
        .map(|(flex_basis, _)| {
            let tree = fri07_c01_intrinsic_recomputation_tree(flex_basis.clone());
            let batch = compute_layout(&tree, 1, fri07_c01_intrinsic_request())
                .expect("intrinsic flex basis remains supported through final layout");
            let requests = tree
                .leaf_requests(2)
                .into_iter()
                .map(|input| {
                    (
                        input.known_content_size().width,
                        input.available_content_size().width,
                    )
                })
                .collect::<Vec<_>>();
            (requests, fri07_c01_intrinsic_output(&batch, 2).size.width)
        })
        .collect::<Vec<_>>();
    let expected = scenarios
        .into_iter()
        .map(|(_, expected)| {
            (
                vec![
                    (None, expected),
                    (Some(S::from_f64(200.0)), expected),
                    (None, expected),
                ],
                S::from_f64(200.0),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        observed, expected,
        "the provider must receive each selected intrinsic main-axis constraint for initial, cross-recomputation, and final requests while grown final geometry remains 200px",
    );
}

#[test]
fn fri07_c01_intrinsic_public_layout_retains_provider_constraint_through_final_layout() {
    assert_fri07_c01_intrinsic_provider_constraint_survives_recomputation::<f32>();
    assert_fri07_c01_intrinsic_provider_constraint_survives_recomputation::<f64>();
}

fn assert_fri07_c01_intrinsic_leaf_geometry<S: LayoutScalar>() {
    for (direction, child_writing_mode, axis) in [
        (
            FlexDirection::Row,
            WritingMode::HorizontalTb,
            PhysicalAxis::Horizontal,
        ),
        (
            FlexDirection::Column,
            WritingMode::HorizontalTb,
            PhysicalAxis::Vertical,
        ),
        (
            FlexDirection::Column,
            WritingMode::VerticalRl,
            PhysicalAxis::Vertical,
        ),
    ] {
        let tree = fri07_c01_intrinsic_leaf_tree::<S>(
            direction,
            child_writing_mode,
            Fri07C01IntrinsicMeasureMode::Values,
        );
        let batch = compute_layout(&tree, 1, fri07_c01_intrinsic_request())
            .expect("direct intrinsic flex bases are supported");
        let min = fri07_c01_intrinsic_output(&batch, 2).size;
        let max = fri07_c01_intrinsic_output(&batch, 3).size;
        let main = |size: Size<S>| match axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        };
        assert_eq!(main(min), S::from_f64(20.0));
        assert_eq!(main(max), S::from_f64(100.0));
    }
}

#[test]
fn fri07_c01_intrinsic_public_layout_preserves_distinct_leaf_geometry_in_both_scalar_lanes() {
    assert_fri07_c01_intrinsic_leaf_geometry::<f32>();
    assert_fri07_c01_intrinsic_leaf_geometry::<f64>();
}

fn assert_fri07_c01_intrinsic_child_container_geometry<S: LayoutScalar>() {
    for (direction, child_writing_mode, axis) in [
        (
            FlexDirection::Row,
            WritingMode::HorizontalTb,
            PhysicalAxis::Horizontal,
        ),
        (
            FlexDirection::Column,
            WritingMode::VerticalRl,
            PhysicalAxis::Vertical,
        ),
    ] {
        let tree = fri07_c01_intrinsic_child_container_tree::<S>(direction, child_writing_mode);
        let batch = compute_layout(&tree, 1, fri07_c01_intrinsic_request())
            .expect("intrinsic child-container flex bases are supported");
        let min = fri07_c01_intrinsic_output(&batch, 2).size;
        let max = fri07_c01_intrinsic_output(&batch, 3).size;
        let main = |size: Size<S>| match axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        };
        assert_eq!(main(min), S::from_f64(20.0));
        assert_eq!(main(max), S::from_f64(100.0));
    }
}

#[test]
fn fri07_c01_intrinsic_public_layout_preserves_child_container_geometry_in_both_scalar_lanes() {
    assert_fri07_c01_intrinsic_child_container_geometry::<f32>();
    assert_fri07_c01_intrinsic_child_container_geometry::<f64>();
}

fn assert_fri07_c01_intrinsic_measurement_errors<S: LayoutScalar>() {
    let provider_tree = fri07_c01_intrinsic_leaf_tree::<S>(
        FlexDirection::Row,
        WritingMode::HorizontalTb,
        Fri07C01IntrinsicMeasureMode::ProviderFailure,
    );
    let provider_error = compute_layout(&provider_tree, 1, fri07_c01_intrinsic_request())
        .expect_err("intrinsic provider failure must remain typed");
    assert_eq!(provider_error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(provider_error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        provider_error.kind(),
        LayoutErrorKindOf::Measurement(Fri07C01IntrinsicMeasureError::ProviderFailure)
    ));

    let non_finite_tree = fri07_c01_intrinsic_leaf_tree::<S>(
        FlexDirection::Row,
        WritingMode::HorizontalTb,
        Fri07C01IntrinsicMeasureMode::NonFinite,
    );
    let non_finite_error = compute_layout(&non_finite_tree, 1, fri07_c01_intrinsic_request())
        .expect_err("intrinsic non-finite provider output must remain typed");
    assert_eq!(non_finite_error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(
        non_finite_error.operation(),
        LayoutOperation::LeafMeasurement
    );
    let LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(invalid)) =
        non_finite_error.kind()
    else {
        panic!("expected invalid measurement output, got {non_finite_error:?}");
    };
    assert_eq!(invalid.axis(), PhysicalAxis::Horizontal);
}

#[test]
fn fri07_c01_intrinsic_provider_failure_and_non_finite_output_remain_exact_in_both_scalar_lanes() {
    assert_fri07_c01_intrinsic_measurement_errors::<f32>();
    assert_fri07_c01_intrinsic_measurement_errors::<f64>();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C01CompositionMeasureMode {
    Values,
    Failure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C01CompositionMeasureError {
    ProviderFailure,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Fri07C01CompositionRetained<S: LayoutScalar> {
    unrounded: HashMap<u32, NodeOutputOf<S>>,
    final_outputs: HashMap<u32, NodeOutputOf<S>>,
    caches: HashMap<u32, CacheOf<S>>,
}

#[derive(Clone, Debug)]
struct Fri07C01CompositionTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    measure_mode: Cell<Fri07C01CompositionMeasureMode>,
    measurement_requests: RefCell<Vec<(u32, LeafMeasureInputOf<S>)>>,
    cache_queries: RefCell<Vec<(u32, bool)>>,
    retained: Fri07C01CompositionRetained<S>,
}

impl<S: LayoutScalar> Fri07C01CompositionTree<S> {
    fn new() -> Self {
        let px = |value| PreferredSizeOf::px(S::from_f64(value));
        let auto_px = |value| LengthAutoOf::px(S::from_f64(value));
        let intrinsic_margin = Edges {
            top: LengthAutoOf::AUTO,
            right: LengthAutoOf::ZERO,
            bottom: LengthAutoOf::AUTO,
            left: LengthAutoOf::ZERO,
        };
        let intrinsic_item = |basis, order, replaced| NodeInputOf {
            item_is_replaced: replaced,
            item_order: ItemOrder::new(order),
            size: Size::new(PreferredSizeOf::AUTO, px(50.4)),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            flex_basis: basis,
            flex_grow: FlexGrowOf::try_new(S::ZERO).expect("zero is a valid flex grow"),
            flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
            margin: intrinsic_margin,
            ..NodeInputOf::default()
        };
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2, 3, 4])
            .children(2, [])
            .children(3, [])
            .children(4, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(px(130.0), px(40.0)),
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(5.0))
                        .expect("composition scrollbar width is finite"),
                    align_items: Some(AlignItems::FlexStart),
                    ..NodeInputOf::default()
                },
            )
            .style(2, intrinsic_item(FlexBasisOf::MIN_CONTENT, 2, false))
            .style(
                3,
                NodeInputOf {
                    position: Position::Absolute,
                    item_order: ItemOrder::new(-100),
                    inset: Edges::new(auto_px(0.0), auto_px(20.0), auto_px(0.0), auto_px(10.0)),
                    size: Size::new(px(20.0), px(10.0)),
                    margin: Edges::all(LengthAutoOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .style(4, intrinsic_item(FlexBasisOf::MAX_CONTENT, -2, true));

        Self {
            tree,
            measure_mode: Cell::new(Fri07C01CompositionMeasureMode::Values),
            measurement_requests: RefCell::new(Vec::new()),
            cache_queries: RefCell::new(Vec::new()),
            retained: Fri07C01CompositionRetained::default(),
        }
    }

    fn request() -> LayoutRootRequestOf<S> {
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(200.0))))
            .expect("composition viewport is finite")
    }

    fn apply_cache_entry(
        retained: &mut Fri07C01CompositionRetained<S>,
        entry: &LayoutCacheStoreEntryOf<u32, S>,
    ) {
        retained
            .caches
            .entry(entry.node())
            .or_default()
            .store_with_context(entry.input(), entry.context(), entry.output());
    }
}

impl<S: LayoutScalar> Traverse for Fri07C01CompositionTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C01CompositionTree<S> {
    type MeasureError = Fri07C01CompositionMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        matches!(node, 2 | 4)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.measurement_requests.borrow_mut().push((node, input));
        if self.measure_mode.get() == Fri07C01CompositionMeasureMode::Failure && node == 4 {
            return Some(Err(Fri07C01CompositionMeasureError::ProviderFailure));
        }

        let width = match input.available_content_size().width {
            MeasurementAvailableOf::MinContent => S::from_f64(40.4),
            MeasurementAvailableOf::MaxContent => S::from_f64(100.4),
            MeasurementAvailableOf::Definite(width) => width.get(),
        };
        Some(Ok(Size::new(width, S::from_f64(50.4))))
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<NodeOutputOf<S>> {
        self.retained.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri07C01CompositionTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = Fri07C01CompositionRetained<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut prepared = self.retained.clone();
        for node in batch.invalidated_nodes() {
            prepared.unrounded.remove(node);
            prepared.final_outputs.remove(node);
            prepared.caches.remove(node);
        }
        for entry in batch.unrounded_entries() {
            prepared.unrounded.insert(entry.node(), entry.output());
        }
        for entry in batch.final_entries() {
            prepared.final_outputs.insert(entry.node(), entry.output());
        }
        for entry in batch.cache_clear_entries() {
            prepared.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            Self::apply_cache_entry(&mut prepared, entry);
        }
        Ok(prepared)
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

fn fri07_c01_composition_assert_near<S: LayoutScalar>(actual: S, expected: f64, context: &str) {
    let difference = (actual.to_f64() - expected).abs();
    assert!(
        difference <= 0.000_02,
        "{context}: expected {expected}, got {}",
        actual.to_f64()
    );
}

fn fri07_c01_composition_geometry<S: LayoutScalar>() -> Vec<f64> {
    let tree = Fri07C01CompositionTree::<S>::new();
    let batch = compute_layout(&tree, 1, Fri07C01CompositionTree::<S>::request())
        .expect("composed intrinsic and margin layout succeeds");
    let root = fri07_c01_composition_output(batch.unrounded_entries(), 1);
    let min = fri07_c01_composition_output(batch.unrounded_entries(), 2);
    let absolute = fri07_c01_composition_output(batch.unrounded_entries(), 3);
    let max = fri07_c01_composition_output(batch.unrounded_entries(), 4);
    let rounded_min = fri07_c01_composition_output(batch.final_entries(), 2);
    let rounded_max = fri07_c01_composition_output(batch.final_entries(), 4);

    assert_eq!(min.source_index, SourceIndex::new(0));
    assert_eq!(absolute.source_index, SourceIndex::new(1));
    assert_eq!(max.source_index, SourceIndex::new(2));
    fri07_c01_composition_assert_near(max.location.x, 0.0, "order-modified max x");
    fri07_c01_composition_assert_near(max.size.width, 100.4, "replaced max-content width");
    fri07_c01_composition_assert_near(min.location.x, 100.4, "order-modified min x");
    fri07_c01_composition_assert_near(min.size.width, 40.4, "non-replaced min-content width");
    for (name, output) in [("min", min), ("max", max)] {
        fri07_c01_composition_assert_near(output.location.y, 0.0, &format!("{name} y"));
        fri07_c01_composition_assert_near(output.size.height, 50.4, &format!("{name} height"));
        fri07_c01_composition_assert_near(output.margin.top, 0.0, &format!("{name} top"));
        fri07_c01_composition_assert_near(output.margin.bottom, -15.4, &format!("{name} bottom"));
    }
    fri07_c01_composition_assert_near(absolute.margin.left, 37.5, "absolute left margin");
    fri07_c01_composition_assert_near(absolute.margin.right, 37.5, "absolute right margin");
    fri07_c01_composition_assert_near(absolute.margin.top, 12.5, "absolute top margin");
    fri07_c01_composition_assert_near(absolute.margin.bottom, 12.5, "absolute bottom margin");
    fri07_c01_composition_assert_near(absolute.location.x, 47.5, "absolute x");
    fri07_c01_composition_assert_near(absolute.location.y, 12.5, "absolute y");

    let scroll = root
        .scroll_geometry
        .expect("composed auto overflow publishes scroll geometry");
    assert_eq!(scroll.used_overflow_x(), Overflow::Auto);
    assert_eq!(scroll.used_overflow_y(), Overflow::Auto);
    assert_eq!(scroll.scrollbar_size(), Size::splat(S::from_f64(5.0)));
    assert_eq!(
        scroll.scrollport().size(),
        Size::new(S::from_f64(125.0), S::from_f64(35.0))
    );
    fri07_c01_composition_assert_near(
        scroll.physical_range().x().maximum(),
        15.8,
        "settled horizontal scroll range",
    );
    fri07_c01_composition_assert_near(
        scroll.physical_range().y().maximum(),
        15.4,
        "settled vertical scroll range",
    );
    assert_eq!(rounded_max.location.x, S::ZERO);
    assert_eq!(
        rounded_max.size,
        Size::new(S::from_f64(100.0), S::from_f64(50.0))
    );
    assert_eq!(rounded_min.location.x, S::from_f64(100.0));
    assert_eq!(
        rounded_min.size,
        Size::new(S::from_f64(41.0), S::from_f64(50.0))
    );

    vec![
        min.location.x.to_f64(),
        min.size.width.to_f64(),
        min.margin.bottom.to_f64(),
        max.location.x.to_f64(),
        max.size.width.to_f64(),
        max.margin.bottom.to_f64(),
        absolute.location.x.to_f64(),
        absolute.location.y.to_f64(),
        scroll.physical_range().x().maximum().to_f64(),
        scroll.physical_range().y().maximum().to_f64(),
    ]
}

#[test]
fn fri07_c01_composition_order_replaced_overflow_absolute_rounding_and_scalars_agree() {
    let f32_geometry = fri07_c01_composition_geometry::<f32>();
    let f64_geometry = fri07_c01_composition_geometry::<f64>();

    assert_eq!(f32_geometry.len(), f64_geometry.len());
    for (index, (f32_value, f64_value)) in f32_geometry.into_iter().zip(f64_geometry).enumerate() {
        assert!(
            (f32_value - f64_value).abs() <= 0.000_02,
            "composition scalar lane mismatch at field {index}: {f32_value} versus {f64_value}"
        );
    }
}

fn assert_fri07_c01_composition_replaced_intrinsic_sizing<S: LayoutScalar>() {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let auto_px = |value| LengthAutoOf::px(S::from_f64(value));
    for (replaced, expected_width) in [(true, 50.0), (false, 60.0)] {
        let tree = Fri07C01IntrinsicTree::new(Fri07C01IntrinsicMeasureMode::Values)
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(px(50.0), px(20.0)),
                    align_items: Some(AlignItems::Stretch),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    item_is_replaced: replaced,
                    aspect_ratio: AspectRatioOf::new(S::from_f64(3.0)),
                    flex_basis: FlexBasisOf::MAX_CONTENT,
                    flex_grow: FlexGrowOf::try_new(S::ZERO).expect("zero is a valid flex grow"),
                    flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    position: Position::Absolute,
                    inset: Edges {
                        top: auto_px(0.0),
                        left: auto_px(0.0),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    size: Size::new(px(10.0), px(5.0)),
                    margin: Edges::all(LengthAutoOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .measured(2);
        let batch = compute_layout(&tree, 1, Fri07C01CompositionTree::<S>::request())
            .expect("replaced intrinsic composition layout succeeds");
        let intrinsic = fri07_c01_composition_output(batch.unrounded_entries(), 2);
        let absolute = fri07_c01_composition_output(batch.unrounded_entries(), 3);

        fri07_c01_composition_assert_near(
            intrinsic.size.width,
            expected_width,
            if replaced {
                "replaced intrinsic automatic minimum"
            } else {
                "non-replaced intrinsic automatic minimum"
            },
        );
        fri07_c01_composition_assert_near(intrinsic.size.height, 20.0, "intrinsic cross stretch");
        assert_eq!(intrinsic.source_index, SourceIndex::new(0));
        assert_eq!(absolute.source_index, SourceIndex::new(1));
        assert_eq!(absolute.margin, Edges::ZERO);
        assert_eq!(absolute.location, Point::ZERO);
        assert!(
            tree.leaf_requests(2).iter().any(|input| {
                input.available_content_size().width == MeasurementAvailableOf::MAX_CONTENT
            }),
            "max-content basis must reach the provider for replaced={replaced}"
        );
    }
}

#[test]
fn fri07_c01_composition_intrinsic_replaced_and_non_replaced_sizing_remain_distinct() {
    assert_fri07_c01_composition_replaced_intrinsic_sizing::<f32>();
    assert_fri07_c01_composition_replaced_intrinsic_sizing::<f64>();
}

fn assert_fri07_c01_composition_cache_and_atomicity<S: LayoutScalar>() {
    let mut tree = Fri07C01CompositionTree::<S>::new();
    let request = Fri07C01CompositionTree::<S>::request();
    let cold = compute_layout(&tree, 1, request).expect("cold composition layout succeeds");
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();
    let cold_measurements = tree.measurement_requests.borrow().len();
    assert!(
        cold_measurements > 0,
        "cold layout must invoke intrinsic measurement"
    );
    cold.apply_to(&mut tree)
        .expect("infallible composition batch commit succeeds");

    tree.cache_queries.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm composition layout succeeds");
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert!(
        tree.cache_queries.borrow().iter().any(|(_, hit)| *hit),
        "warm composition layout must reuse a committed cache entry"
    );
    assert!(
        tree.measurement_requests.borrow()[cold_measurements..]
            .iter()
            .all(|(node, input)| match node {
                2 => input.available_content_size().width == MeasurementAvailableOf::MIN_CONTENT,
                4 => matches!(
                    input.available_content_size().width,
                    MeasurementAvailableOf::MinContent | MeasurementAvailableOf::MaxContent
                ),
                _ => false,
            }),
        "warm recomputation must preserve intrinsic measurement constraints"
    );

    tree.measure_mode
        .set(Fri07C01CompositionMeasureMode::Failure);
    let retained_before_failure = tree.retained.clone();
    let error = compute_layout_invalidated(&tree, 1, request, &[4])
        .expect_err("invalidated intrinsic provider failure returns no batch");
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(4));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(Fri07C01CompositionMeasureError::ProviderFailure)
    ));
    assert_eq!(tree.retained, retained_before_failure);
}

#[test]
fn fri07_c01_composition_cache_cold_warm_and_failed_measurement_are_atomic() {
    assert_fri07_c01_composition_cache_and_atomicity::<f32>();
    assert_fri07_c01_composition_cache_and_atomicity::<f64>();
}

const FRI07_C03_COMPOSED_SCALAR_TOLERANCE: f64 = 0.000_02;

#[derive(Clone, Copy, Debug)]
struct Fri07C03ComposedCase {
    swap_intrinsic_bases: bool,
    collapse_max_item: bool,
    reverse_order: bool,
    reverse_source: bool,
    flow: FlowAxes,
    direction: FlexDirection,
    wrap: FlexWrap,
    replaced: bool,
    cross_auto_margin_pattern: usize,
    absolute_pattern: usize,
    overflow: ComputedOverflow,
    container_main: f64,
}

impl Fri07C03ComposedCase {
    fn deterministic() -> Self {
        Self {
            swap_intrinsic_bases: false,
            collapse_max_item: false,
            reverse_order: false,
            reverse_source: false,
            flow: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            replaced: false,
            cross_auto_margin_pattern: 3,
            absolute_pattern: 0,
            overflow: computed_overflow(Overflow::Visible, Overflow::Clip),
            container_main: 120.0,
        }
    }

    fn axes(self) -> FlexAxes {
        FlexAxes::new(self.flow, self.direction, self.wrap)
    }

    fn children(self) -> [u32; 4] {
        if self.reverse_source {
            [3, 4, 2, 5]
        } else {
            [2, 3, 4, 5]
        }
    }

    fn source_index(self, node: u32) -> SourceIndex {
        let index = self
            .children()
            .iter()
            .position(|child| *child == node)
            .expect("every composed child has a source position");
        SourceIndex::new(index)
    }
}

#[derive(Clone, Debug)]
struct Fri07C03ComposedTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    axes: FlexAxes,
    measure_mode: Cell<Fri07C03ComposedMeasureMode>,
    requests: RefCell<Vec<(u32, LeafMeasureInputOf<S>)>>,
    cache_queries: RefCell<Vec<(u32, bool)>>,
    retained: Fri07C01CompositionRetained<S>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C03ComposedMeasureMode {
    Values,
    FailIntrinsic,
    FailSecondRound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri07C03ComposedMeasureError {
    Intrinsic,
    SecondRound,
}

impl<S: LayoutScalar> Traverse for Fri07C03ComposedTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri07C03ComposedTree<S> {
    type MeasureError = Fri07C03ComposedMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        matches!(node, 2 | 3)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if !matches!(node, 2 | 3) {
            return None;
        }
        let node_request_index = self
            .requests
            .borrow()
            .iter()
            .filter(|(request_node, _)| *request_node == node)
            .count();
        self.requests.borrow_mut().push((node, input));
        match (self.measure_mode.get(), node, node_request_index) {
            (Fri07C03ComposedMeasureMode::FailIntrinsic, 2, 0) => {
                return Some(Err(Fri07C03ComposedMeasureError::Intrinsic));
            }
            (Fri07C03ComposedMeasureMode::FailSecondRound, 2, 1) => {
                return Some(Err(Fri07C03ComposedMeasureError::SecondRound));
            }
            _ => {}
        }
        let main = match (node, self.axes.main_size(input.available_content_size())) {
            (2, MeasurementAvailableOf::MinContent) => S::from_f64(20.0),
            (2, MeasurementAvailableOf::MaxContent) => S::from_f64(45.0),
            (3, MeasurementAvailableOf::MinContent) => S::from_f64(25.0),
            (3, MeasurementAvailableOf::MaxContent) => S::from_f64(60.0),
            (_, MeasurementAvailableOf::Definite(value)) => value.get(),
            _ => unreachable!("only composed intrinsic leaves are measured"),
        };
        let cross = if node == 2 { 20.0 } else { 30.0 };
        Some(Ok(self.axes.size_from_main_cross(main, S::from_f64(cross))))
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<NodeOutputOf<S>> {
        self.retained.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri07C03ComposedTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = Fri07C01CompositionRetained<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut prepared = self.retained.clone();
        for node in batch.invalidated_nodes() {
            prepared.unrounded.remove(node);
            prepared.final_outputs.remove(node);
            prepared.caches.remove(node);
        }
        for entry in batch.unrounded_entries() {
            prepared.unrounded.insert(entry.node(), entry.output());
        }
        for entry in batch.final_entries() {
            prepared.final_outputs.insert(entry.node(), entry.output());
        }
        for entry in batch.cache_clear_entries() {
            prepared.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            Fri07C01CompositionTree::apply_cache_entry(&mut prepared, entry);
        }
        Ok(prepared)
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

fn fri07_c03_composed_layout_tree<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
    collapsed_main: f64,
) -> Fri07C03ComposedTree<S> {
    let axes = case.axes();
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let length = |value| LengthOf::px(S::from_f64(value));
    let auto_length = |value| LengthAutoOf::px(S::from_f64(value));
    let mut cross_margin = Edges::all(LengthAutoOf::ZERO);
    if matches!(case.cross_auto_margin_pattern, 1 | 3) {
        axes.set_normal_cross_start_edge(&mut cross_margin, LengthAutoOf::AUTO);
    }
    if matches!(case.cross_auto_margin_pattern, 2 | 3) {
        axes.set_normal_cross_end_edge(&mut cross_margin, LengthAutoOf::AUTO);
    }
    let (min_basis, max_basis) = if case.swap_intrinsic_bases {
        (FlexBasisOf::MAX_CONTENT, FlexBasisOf::MIN_CONTENT)
    } else {
        (FlexBasisOf::MIN_CONTENT, FlexBasisOf::MAX_CONTENT)
    };
    let (min_order, max_order) = if case.reverse_order {
        (ItemOrder::new(3), ItemOrder::new(-3))
    } else {
        (ItemOrder::new(-3), ItemOrder::new(3))
    };
    let (inset, absolute_margin) = match case.absolute_pattern {
        0 => (
            Edges::new(
                auto_length(5.0),
                auto_length(20.0),
                auto_length(15.0),
                auto_length(10.0),
            ),
            Edges::all(LengthAutoOf::AUTO),
        ),
        1 => (
            Edges {
                top: auto_length(5.0),
                left: auto_length(10.0),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            Edges::all(LengthAutoOf::AUTO),
        ),
        2 => (
            Edges::all(auto_length(40.0)),
            Edges::all(LengthAutoOf::AUTO),
        ),
        _ => unreachable!("the absolute pattern strategy is bounded"),
    };
    let intrinsic_item = |basis, order, collapse, margin, replaced| NodeInputOf {
        item_order: order,
        item_is_replaced: replaced,
        flex_item_collapse: collapse,
        size: axes.size_from_main_cross(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
        min_size: axes.size_from_main_cross(MinSizeOf::ZERO, MinSizeOf::ZERO),
        flex_basis: basis,
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
        margin,
        ..NodeInputOf::default()
    };
    let collapsed = NodeInputOf {
        item_order: ItemOrder::new(0),
        flex_item_collapse: FlexItemCollapse::Collapsed,
        size: axes.size_from_main_cross(preferred(collapsed_main), preferred(50.0)),
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
        overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
        scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(3.0))
            .expect("collapsed scrollbar width is finite"),
        ..NodeInputOf::default()
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, case.children())
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: case.flow.writing_mode(),
                direction: case.flow.direction(),
                flex_direction: case.direction,
                flex_wrap: case.wrap,
                size: axes
                    .size_from_main_cross(preferred(case.container_main), PreferredSizeOf::AUTO),
                gap: axes.size_from_main_cross(length(5.0), length(4.0)),
                align_content: Some(AlignContent::FlexStart),
                align_items: Some(AlignItems::FlexStart),
                overflow: case.overflow,
                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(3.0))
                    .expect("composed scrollbar width is finite"),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            intrinsic_item(
                min_basis,
                min_order,
                FlexItemCollapse::Normal,
                cross_margin,
                case.replaced,
            ),
        )
        .style(
            3,
            intrinsic_item(
                max_basis,
                max_order,
                if case.collapse_max_item {
                    FlexItemCollapse::Collapsed
                } else {
                    FlexItemCollapse::Normal
                },
                Edges::all(LengthAutoOf::ZERO),
                false,
            ),
        )
        .style(4, collapsed)
        .style(
            5,
            NodeInputOf {
                position: Position::Absolute,
                inset,
                size: Size::new(preferred(20.0), preferred(10.0)),
                margin: absolute_margin,
                ..NodeInputOf::default()
            },
        );

    Fri07C03ComposedTree {
        tree,
        axes,
        measure_mode: Cell::new(Fri07C03ComposedMeasureMode::Values),
        requests: RefCell::new(Vec::new()),
        cache_queries: RefCell::new(Vec::new()),
        retained: Fri07C01CompositionRetained::default(),
    }
}

#[derive(Clone, Debug)]
struct Fri07C03ComposedSnapshot<S: LayoutScalar> {
    outputs: [NodeOutputOf<S>; 5],
    requests: Vec<(u32, LeafMeasureInputOf<S>)>,
}

impl<S: LayoutScalar> Fri07C03ComposedSnapshot<S> {
    fn output(&self, node: u32) -> NodeOutputOf<S> {
        self.outputs[(node - 1) as usize]
    }

    fn geometry(&self) -> Vec<f64> {
        let mut geometry = Vec::new();
        for output in self.outputs {
            geometry.extend([
                output.location.x.to_f64(),
                output.location.y.to_f64(),
                output.size.width.to_f64(),
                output.size.height.to_f64(),
                output.margin.top.to_f64(),
                output.margin.right.to_f64(),
                output.margin.bottom.to_f64(),
                output.margin.left.to_f64(),
            ]);
        }
        let scroll = self
            .output(1)
            .scroll_geometry
            .expect("composed root publishes scroll geometry");
        geometry.extend([
            scroll.physical_range().x().minimum().to_f64(),
            scroll.physical_range().x().maximum().to_f64(),
            scroll.physical_range().y().minimum().to_f64(),
            scroll.physical_range().y().maximum().to_f64(),
            scroll.scrollbar_size().width.to_f64(),
            scroll.scrollbar_size().height.to_f64(),
        ]);
        geometry
    }
}

fn fri07_c03_composed_layout_snapshot<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
    collapsed_main: f64,
) -> Fri07C03ComposedSnapshot<S> {
    let tree = fri07_c03_composed_layout_tree::<S>(case, collapsed_main);
    let batch = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("all four completed flex capabilities compose");
    let outputs = core::array::from_fn(|index| {
        fri07_c01_composition_output(batch.unrounded_entries(), (index + 1) as u32)
    });
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        for node in 1..=5 {
            assert_fri07_c02_composition_finite_output(
                fri07_c01_composition_output(entries, node),
                &format!("C03 composed node {node}"),
            );
        }
    }
    Fri07C03ComposedSnapshot {
        outputs,
        requests: tree.requests.into_inner(),
    }
}

fn fri07_c03_expected_intrinsic<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
    node: u32,
) -> MeasurementAvailableOf<S> {
    match (case.swap_intrinsic_bases, node) {
        (false, 2) | (true, 3) => MeasurementAvailableOf::MIN_CONTENT,
        (false, 3) | (true, 2) => MeasurementAvailableOf::MAX_CONTENT,
        _ => unreachable!("only the two intrinsic items have basis expectations"),
    }
}

fn assert_fri07_c03_composed_layout_case<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
) -> Fri07C03ComposedSnapshot<S> {
    let axes = case.axes();
    let snapshot = fri07_c03_composed_layout_snapshot::<S>(case, 70.0);
    for node in 2..=5 {
        assert_eq!(
            snapshot.output(node).source_index,
            case.source_index(node),
            "node {node} remains associated with its raw source position for {case:?}"
        );
    }
    assert_eq!(
        snapshot.output(4),
        NodeOutputOf::with_source_index(case.source_index(4)),
        "the strut item publishes a zero box"
    );
    if case.collapse_max_item {
        assert_eq!(
            snapshot.output(3),
            NodeOutputOf::with_source_index(case.source_index(3)),
            "the rotated collapsed intrinsic item publishes no geometry"
        );
    }

    for node in [2, 3] {
        let expected = fri07_c03_expected_intrinsic::<S>(case, node);
        let intrinsic_count = snapshot
            .requests
            .iter()
            .filter(|(request_node, input)| {
                *request_node == node && axes.main_size(input.available_content_size()) == expected
            })
            .count();
        assert!(
            intrinsic_count >= 1,
            "node {node} must retain its selected intrinsic constraint for {case:?}; requests={:?}",
            snapshot.requests
        );
        let collapse_round_markers = snapshot
            .requests
            .iter()
            .filter(|(request_node, input)| {
                *request_node == node
                    && axes.main_size(input.available_content_size()) == expected
                    && axes.main_size(input.known_content_size()).is_none()
                    && match axes.cross_size(input.available_content_size()) {
                        MeasurementAvailableOf::Definite(value) => value.get() > S::from_f64(50.0),
                        MeasurementAvailableOf::MinContent | MeasurementAvailableOf::MaxContent => {
                            false
                        }
                    }
            })
            .count();
        assert!(
            (1..=2).contains(&collapse_round_markers),
            "node {node} observes no more than two complete collapsed-layout settlements for {case:?}; requests={:?}",
            snapshot.requests
        );
    }

    assert!(
        axes.cross_size(snapshot.output(1).size) >= S::from_f64(50.0),
        "the collapsed item's first-round 50px used cross size remains a line strut"
    );
    let min = snapshot.output(2);
    let cross_start = axes.normal_cross_start_edge(min.margin);
    let cross_end = axes.normal_cross_end_edge(min.margin);
    match case.cross_auto_margin_pattern {
        0 => assert_eq!((cross_start, cross_end), (S::ZERO, S::ZERO)),
        1 => {
            assert!(cross_start >= S::ZERO);
            assert_eq!(cross_end, S::ZERO);
        }
        2 => {
            assert_eq!(cross_start, S::ZERO);
            assert!(cross_end >= S::ZERO);
        }
        3 => {
            assert!(cross_start >= S::ZERO);
            fri07_c01_composition_assert_near(
                cross_start - cross_end,
                0.0,
                "paired ordinary cross auto margins",
            );
        }
        _ => unreachable!("the cross auto-margin strategy is bounded"),
    }

    let absolute = snapshot.output(5);
    let containing_scrollport = snapshot
        .output(1)
        .scroll_geometry
        .expect("composed root publishes its inset containing geometry")
        .scrollport();
    let containing_size = containing_scrollport.size();
    let containing_origin = containing_scrollport.origin();
    match case.absolute_pattern {
        0 => {
            fri07_c01_composition_assert_near(
                absolute.margin.left + absolute.margin.right,
                containing_size.width.to_f64() - 50.0,
                "definite horizontal inset-modified margin sum",
            );
            fri07_c01_composition_assert_near(
                absolute.margin.top + absolute.margin.bottom,
                containing_size.height.to_f64() - 30.0,
                "definite vertical inset-modified margin sum",
            );
            fri07_c01_composition_assert_near(
                absolute.location.x - absolute.margin.left,
                containing_origin.x.to_f64() + 10.0,
                "absolute definite left inset",
            );
            fri07_c01_composition_assert_near(
                absolute.location.y - absolute.margin.top,
                containing_origin.y.to_f64() + 5.0,
                "absolute definite top inset",
            );
        }
        1 => {
            assert_eq!(absolute.margin, Edges::ZERO);
            assert_eq!(
                absolute.location,
                Point::new(
                    containing_origin.x + S::from_f64(10.0),
                    containing_origin.y + S::from_f64(5.0),
                )
            );
        }
        2 => {
            fri07_c01_composition_assert_near(
                absolute.margin.left + absolute.margin.right,
                containing_size.width.to_f64() - 100.0,
                "negative horizontal inset-modified margin sum",
            );
            fri07_c01_composition_assert_near(
                absolute.margin.top + absolute.margin.bottom,
                containing_size.height.to_f64() - 90.0,
                "negative vertical inset-modified margin sum",
            );
        }
        _ => unreachable!("the absolute strategy is bounded"),
    }

    let root_scroll = snapshot
        .output(1)
        .scroll_geometry
        .expect("composed root publishes settled scroll geometry");
    assert_eq!(root_scroll.used_overflow_x(), case.overflow.x());
    assert_eq!(root_scroll.used_overflow_y(), case.overflow.y());

    let payload_control = fri07_c03_composed_layout_snapshot::<S>(case, 370.0);
    for node in [1, 2, 3, 5] {
        assert_eq!(
            snapshot.output(node),
            payload_control.output(node),
            "the collapsed item's first-round main size and scroll state cannot contribute to committed node {node}"
        );
    }
    snapshot
}

#[test]
fn fri07_c03_composed_layout_exact_geometry_margins_strut_absolute_and_scroll() {
    let case = Fri07C03ComposedCase::deterministic();
    let snapshot = assert_fri07_c03_composed_layout_case::<f64>(case);
    let root = snapshot.output(1);
    let min = snapshot.output(2);
    let max = snapshot.output(3);
    let absolute = snapshot.output(5);

    assert_eq!(root.size, Size::new(120.0, 50.0));
    assert_eq!(root.content_size, Size::new(120.0, 50.0));
    assert_eq!(min.location, Point::new(0.0, 15.0));
    assert_eq!(min.size, Size::new(20.0, 20.0));
    assert_eq!(min.margin, Edges::new(15.0, 0.0, 15.0, 0.0));
    assert_eq!(max.location, Point::new(25.0, 0.0));
    assert_eq!(max.size, Size::new(60.0, 30.0));
    assert_eq!(absolute.location, Point::new(45.0, 15.0));
    assert_eq!(absolute.margin, Edges::new(10.0, 35.0, 10.0, 35.0));
    let scroll = root
        .scroll_geometry
        .expect("deterministic root publishes scroll geometry");
    assert_eq!(scroll.used_overflow_x(), Overflow::Visible);
    assert_eq!(scroll.used_overflow_y(), Overflow::Clip);
    assert_eq!(scroll.scrollbar_size(), Size::ZERO);
    assert_eq!(scroll.physical_range().x().minimum(), 0.0);
    assert_eq!(scroll.physical_range().x().maximum(), 0.0);
    assert_eq!(scroll.physical_range().y().minimum(), 0.0);
    assert_eq!(scroll.physical_range().y().maximum(), 0.0);
}

fn fri07_c03_composed_layout_cases() -> Vec<Fri07C03ComposedCase> {
    let mut cases = Vec::new();
    let base = Fri07C03ComposedCase::deterministic();
    for swap_intrinsic_bases in [false, true] {
        cases.push(Fri07C03ComposedCase {
            swap_intrinsic_bases,
            ..base
        });
    }
    for collapse_max_item in [false, true] {
        cases.push(Fri07C03ComposedCase {
            collapse_max_item,
            ..base
        });
    }
    for (reverse_order, reverse_source) in [(false, false), (true, false), (false, true)] {
        cases.push(Fri07C03ComposedCase {
            reverse_order,
            reverse_source,
            ..base
        });
    }
    for flow in [
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
    ] {
        cases.push(Fri07C03ComposedCase { flow, ..base });
    }
    for direction in [
        FlexDirection::Row,
        FlexDirection::RowReverse,
        FlexDirection::Column,
        FlexDirection::ColumnReverse,
    ] {
        cases.push(Fri07C03ComposedCase { direction, ..base });
    }
    for wrap in [FlexWrap::NoWrap, FlexWrap::Wrap, FlexWrap::WrapReverse] {
        cases.push(Fri07C03ComposedCase {
            wrap,
            container_main: if wrap == FlexWrap::NoWrap {
                120.0
            } else {
                70.0
            },
            ..base
        });
    }
    for replaced in [false, true] {
        cases.push(Fri07C03ComposedCase { replaced, ..base });
    }
    for cross_auto_margin_pattern in 0..4 {
        cases.push(Fri07C03ComposedCase {
            cross_auto_margin_pattern,
            ..base
        });
    }
    for absolute_pattern in 0..3 {
        cases.push(Fri07C03ComposedCase {
            absolute_pattern,
            ..base
        });
    }
    for overflow in [
        computed_overflow(Overflow::Visible, Overflow::Clip),
        computed_overflow(Overflow::Hidden, Overflow::Auto),
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        computed_overflow(Overflow::Scroll, Overflow::Hidden),
    ] {
        cases.push(Fri07C03ComposedCase { overflow, ..base });
    }
    cases
}

#[test]
fn fri07_c03_composed_layout_paired_controls_rotate_every_owned_dimension() {
    let cases = fri07_c03_composed_layout_cases();
    assert_eq!(
        cases.len(),
        30,
        "the deterministic control set stays bounded"
    );
    for (index, case) in cases.into_iter().enumerate() {
        let f32_snapshot = assert_fri07_c03_composed_layout_case::<f32>(case);
        let f64_snapshot = assert_fri07_c03_composed_layout_case::<f64>(case);
        let f32_geometry = f32_snapshot.geometry();
        let f64_geometry = f64_snapshot.geometry();
        assert_eq!(f32_geometry.len(), f64_geometry.len());
        for (field, (f32_value, f64_value)) in
            f32_geometry.into_iter().zip(f64_geometry).enumerate()
        {
            assert!(
                (f32_value - f64_value).abs() <= FRI07_C03_COMPOSED_SCALAR_TOLERANCE,
                "deterministic control {index} field {field} differs across scalar lanes: {f32_value} versus {f64_value}; case={case:?}"
            );
        }
    }

    let normal = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        collapse_max_item: false,
        ..Fri07C03ComposedCase::deterministic()
    });
    let collapsed = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        collapse_max_item: true,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(normal.output(3).size, Size::ZERO);
    assert_eq!(collapsed.output(3).size, Size::ZERO);

    let min_basis = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        swap_intrinsic_bases: false,
        ..Fri07C03ComposedCase::deterministic()
    });
    let max_basis = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        swap_intrinsic_bases: true,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_eq!(min_basis.output(2).size.width, 20.0);
    assert_eq!(max_basis.output(2).size.width, 45.0);

    let source_forward =
        assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase::deterministic());
    let source_reverse = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        reverse_source: true,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_eq!(
        source_forward.output(2).location,
        source_reverse.output(2).location
    );
    assert_ne!(
        source_forward.output(2).source_index,
        source_reverse.output(2).source_index
    );

    let order_reverse = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        reverse_order: true,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(
        source_forward.output(2).location,
        order_reverse.output(2).location
    );
    assert_eq!(source_forward.output(2).size, order_reverse.output(2).size);

    let vertical_flow = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        flow: FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_eq!(source_forward.output(1).size, Size::new(120.0, 50.0));
    assert_eq!(vertical_flow.output(1).size, Size::new(50.0, 120.0));

    let row_reverse = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        direction: FlexDirection::RowReverse,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(
        source_forward.output(2).location,
        row_reverse.output(2).location
    );

    let narrow_nowrap = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        container_main: 70.0,
        ..Fri07C03ComposedCase::deterministic()
    });
    let narrow_wrap = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        wrap: FlexWrap::Wrap,
        container_main: 70.0,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(narrow_nowrap.output(1).size, narrow_wrap.output(1).size);

    let non_replaced = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        swap_intrinsic_bases: true,
        replaced: false,
        container_main: 40.0,
        ..Fri07C03ComposedCase::deterministic()
    });
    let replaced = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        swap_intrinsic_bases: true,
        replaced: true,
        container_main: 40.0,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_eq!(
        non_replaced.output(2).size,
        replaced.output(2).size,
        "direct intrinsic-basis geometry remains selected by the provider while replacedness rotates independently"
    );

    let no_cross_auto = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        cross_auto_margin_pattern: 0,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(
        no_cross_auto.output(2).margin,
        source_forward.output(2).margin
    );

    let auto_inset = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        absolute_pattern: 1,
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(source_forward.output(5).margin, auto_inset.output(5).margin);

    let forced_scroll = assert_fri07_c03_composed_layout_case::<f64>(Fri07C03ComposedCase {
        overflow: computed_overflow(Overflow::Scroll, Overflow::Hidden),
        ..Fri07C03ComposedCase::deterministic()
    });
    assert_ne!(
        source_forward
            .output(1)
            .scroll_geometry
            .expect("visible control has scroll geometry")
            .scrollbar_size(),
        forced_scroll
            .output(1)
            .scroll_geometry
            .expect("forced-scroll control has scroll geometry")
            .scrollbar_size()
    );
}

fn fri07_c03_flow(selector: usize) -> FlowAxes {
    [
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysRl, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
    ][selector]
}

fn fri07_c03_direction(selector: usize) -> FlexDirection {
    [
        FlexDirection::Row,
        FlexDirection::RowReverse,
        FlexDirection::Column,
        FlexDirection::ColumnReverse,
    ][selector]
}

fn fri07_c03_wrap(selector: usize) -> FlexWrap {
    [FlexWrap::NoWrap, FlexWrap::Wrap, FlexWrap::WrapReverse][selector]
}

fn fri07_c03_overflow(selector: usize) -> ComputedOverflow {
    [
        computed_overflow(Overflow::Visible, Overflow::Clip),
        computed_overflow(Overflow::Hidden, Overflow::Auto),
        computed_overflow(Overflow::Auto, Overflow::Scroll),
        computed_overflow(Overflow::Scroll, Overflow::Hidden),
    ][selector]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn fri07_c03_composed_layout_bounded_property_preserves_invariants(
        swap_intrinsic_bases in any::<bool>(),
        collapse_max_item in any::<bool>(),
        reverse_order in any::<bool>(),
        reverse_source in any::<bool>(),
        flow_selector in 0usize..6,
        direction_selector in 0usize..4,
        wrap_selector in 0usize..3,
        replaced in any::<bool>(),
        cross_auto_margin_pattern in 0usize..4,
        absolute_pattern in 0usize..3,
        overflow_selector in 0usize..4,
        container_main in 70u16..151,
    ) {
        let case = Fri07C03ComposedCase {
            swap_intrinsic_bases,
            collapse_max_item,
            reverse_order,
            reverse_source,
            flow: fri07_c03_flow(flow_selector),
            direction: fri07_c03_direction(direction_selector),
            wrap: fri07_c03_wrap(wrap_selector),
            replaced,
            cross_auto_margin_pattern,
            absolute_pattern,
            overflow: fri07_c03_overflow(overflow_selector),
            container_main: f64::from(container_main),
        };
        let f32_snapshot = assert_fri07_c03_composed_layout_case::<f32>(case);
        let f64_snapshot = assert_fri07_c03_composed_layout_case::<f64>(case);
        let f32_geometry = f32_snapshot.geometry();
        let f64_geometry = f64_snapshot.geometry();
        prop_assert_eq!(f32_geometry.len(), f64_geometry.len());
        for (field, (f32_value, f64_value)) in
            f32_geometry.into_iter().zip(f64_geometry).enumerate()
        {
            prop_assert!(
                (f32_value - f64_value).abs() <= FRI07_C03_COMPOSED_SCALAR_TOLERANCE,
                "property field {} differs across scalar lanes: {} versus {}; case={:?}",
                field,
                f32_value,
                f64_value,
                case,
            );
        }

        let basis_control = assert_fri07_c03_composed_layout_case::<f64>(
            Fri07C03ComposedCase {
                swap_intrinsic_bases: !case.swap_intrinsic_bases,
                ..case
            },
        );
        prop_assert_ne!(
            f64_snapshot.output(2).size,
            basis_control.output(2).size,
            "the paired basis control must change only the selected intrinsic geometry"
        );

        let source_control = assert_fri07_c03_composed_layout_case::<f64>(
            Fri07C03ComposedCase {
                reverse_source: !case.reverse_source,
                ..case
            },
        );
        prop_assert_eq!(
            f64_snapshot.output(2).location,
            source_control.output(2).location,
            "source rotation cannot change order-modified physical geometry"
        );
        prop_assert_ne!(
            f64_snapshot.output(2).source_index,
            source_control.output(2).source_index,
            "source rotation remains observable in stable source association"
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Fri07C03ComposedStateMeasurement<S: LayoutScalar> {
    node: u32,
    known_main: Option<S>,
    known_cross: Option<S>,
    available_main: MeasurementAvailableOf<S>,
    available_cross: MeasurementAvailableOf<S>,
}

fn fri07_c03_composed_state_measurements<S: LayoutScalar>(
    tree: &Fri07C03ComposedTree<S>,
) -> Vec<Fri07C03ComposedStateMeasurement<S>> {
    tree.requests
        .borrow()
        .iter()
        .map(|(node, input)| Fri07C03ComposedStateMeasurement {
            node: *node,
            known_main: tree.axes.main_size(input.known_content_size()),
            known_cross: tree.axes.cross_size(input.known_content_size()),
            available_main: tree.axes.main_size(input.available_content_size()),
            available_cross: tree.axes.cross_size(input.available_content_size()),
        })
        .collect()
}

fn fri07_c03_composed_state_definite<S: LayoutScalar>(value: f64) -> MeasurementAvailableOf<S> {
    MeasurementAvailableOf::definite(S::from_f64(value))
        .expect("composed state measurement target is finite and non-negative")
}

fn assert_fri07_c03_composed_state_exact_round_bounds<S: LayoutScalar>() {
    let tree = fri07_c03_composed_layout_tree::<S>(Fri07C03ComposedCase::deterministic(), 70.0);
    compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("bounded composed state layout succeeds");
    let measurement = |node, available_main, available_cross| Fri07C03ComposedStateMeasurement {
        node,
        known_main: None,
        known_cross: None,
        available_main,
        available_cross,
    };
    assert_eq!(
        fri07_c03_composed_state_measurements(&tree),
        [
            measurement(
                2,
                MeasurementAvailableOf::MIN_CONTENT,
                fri07_c03_composed_state_definite(300.0),
            ),
            measurement(
                3,
                MeasurementAvailableOf::MAX_CONTENT,
                fri07_c03_composed_state_definite(300.0),
            ),
            measurement(
                2,
                MeasurementAvailableOf::MIN_CONTENT,
                fri07_c03_composed_state_definite(20.0),
            ),
            measurement(
                3,
                MeasurementAvailableOf::MAX_CONTENT,
                fri07_c03_composed_state_definite(30.0),
            ),
        ],
        "the complete composed tree performs only the existing intrinsic pass and one finite collapsed replay"
    );
}

#[test]
fn fri07_c03_composed_state_measurement_trace_has_exact_finite_round_bound() {
    assert_fri07_c03_composed_state_exact_round_bounds::<f32>();
    assert_fri07_c03_composed_state_exact_round_bounds::<f64>();
}

fn fri07_c03_composed_state_geometry<S: LayoutScalar>(
    unrounded: &[LayoutOutputEntryOf<u32, S>],
    final_outputs: &[LayoutOutputEntryOf<u32, S>],
) -> Vec<f64> {
    let mut geometry = Vec::new();
    for entries in [unrounded, final_outputs] {
        for node in 1..=5 {
            let output = fri07_c01_composition_output(entries, node);
            geometry.extend([
                output.location.x.to_f64(),
                output.location.y.to_f64(),
                output.size.width.to_f64(),
                output.size.height.to_f64(),
                output.margin.top.to_f64(),
                output.margin.right.to_f64(),
                output.margin.bottom.to_f64(),
                output.margin.left.to_f64(),
            ]);
        }
    }
    geometry
}

fn assert_fri07_c03_composed_state_cache_rounding_and_scalar<S: LayoutScalar>() -> Vec<f64> {
    let case = Fri07C03ComposedCase {
        container_main: 120.5,
        ..Fri07C03ComposedCase::deterministic()
    };
    let mut tree = fri07_c03_composed_layout_tree::<S>(case, 70.25);
    let request = fri07_c02_collapse_round_request();
    let cold = compute_layout(&tree, 1, request).expect("cold composed state layout succeeds");
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();
    let cold_unrounded_fragments = cold.unrounded_inline_fragments().to_vec();
    let cold_final_fragments = cold.final_inline_fragments().to_vec();
    let cold_measurements = fri07_c03_composed_state_measurements(&tree);
    assert_eq!(cold_measurements.len(), 4);

    for node in 1..=5 {
        let unrounded = fri07_c01_composition_output(&cold_unrounded, node);
        let rounded = fri07_c01_composition_output(&cold_final, node);
        assert_eq!(unrounded.source_index, rounded.source_index);
        for (unrounded_start, unrounded_size, rounded_start, rounded_size) in [
            (
                unrounded.location.x,
                unrounded.size.width,
                rounded.location.x,
                rounded.size.width,
            ),
            (
                unrounded.location.y,
                unrounded.size.height,
                rounded.location.y,
                rounded.size.height,
            ),
        ] {
            fri07_c01_composition_assert_near(
                rounded_start,
                unrounded_start.to_f64().round(),
                "rounded composed source start",
            );
            fri07_c01_composition_assert_near(
                rounded_start + rounded_size,
                (unrounded_start + unrounded_size).to_f64().round(),
                "rounded composed source end",
            );
        }
    }
    let unrounded_absolute = fri07_c01_composition_output(&cold_unrounded, 5);
    let rounded_absolute = fri07_c01_composition_output(&cold_final, 5);
    assert_ne!(unrounded_absolute.location.x, rounded_absolute.location.x);
    assert_eq!(
        fri07_c01_composition_output(&cold_unrounded, 4),
        NodeOutputOf::with_source_index(case.source_index(4))
    );
    assert_eq!(
        fri07_c01_composition_output(&cold_final, 4),
        NodeOutputOf::with_source_index(case.source_index(4))
    );

    cold.apply_to(&mut tree)
        .expect("cold composed state batch commit succeeds");
    let cold_retained = tree.retained.clone();
    assert!(!cold_retained.caches.is_empty());

    tree.cache_queries.borrow_mut().clear();
    tree.requests.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm composed state layout succeeds");
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert_eq!(warm.unrounded_inline_fragments(), cold_unrounded_fragments);
    assert_eq!(warm.final_inline_fragments(), cold_final_fragments);
    assert!(
        tree.cache_queries.borrow().iter().any(|(_, hit)| *hit),
        "warm composed state layout reuses committed cache facts"
    );
    assert!(
        fri07_c03_composed_state_measurements(&tree).len() <= cold_measurements.len(),
        "warm cache use cannot introduce another flex or collapse round"
    );
    warm.apply_to(&mut tree)
        .expect("warm composed state batch commit succeeds");
    assert_eq!(tree.retained.unrounded, cold_retained.unrounded);
    assert_eq!(tree.retained.final_outputs, cold_retained.final_outputs);
    for entry in warm.cache_store_entries().iter().rev() {
        let committed =
            tree.retained.caches[&entry.node()].get_with_context(entry.input(), entry.context());
        assert_eq!(
            committed,
            Some(entry.output()),
            "every warm staged cache fact is committed through the existing cache owner"
        );
    }

    fri07_c03_composed_state_geometry(&cold_unrounded, &cold_final)
}

#[test]
fn fri07_c03_composed_state_cold_warm_rounding_and_scalar_lanes_agree() {
    let f32_geometry = assert_fri07_c03_composed_state_cache_rounding_and_scalar::<f32>();
    let f64_geometry = assert_fri07_c03_composed_state_cache_rounding_and_scalar::<f64>();
    assert_eq!(f32_geometry.len(), f64_geometry.len());
    for (field, (f32_value, f64_value)) in f32_geometry.into_iter().zip(f64_geometry).enumerate() {
        assert!(
            (f32_value - f64_value).abs() <= FRI07_C03_COMPOSED_SCALAR_TOLERANCE,
            "composed state field {field} differs across scalar lanes: {f32_value} versus {f64_value}"
        );
    }
}

fn assert_fri07_c03_composed_state_failure_is_atomic<S: LayoutScalar>(
    mode: Fri07C03ComposedMeasureMode,
    expected_error: Fri07C03ComposedMeasureError,
    expected_requests: usize,
) {
    let case = Fri07C03ComposedCase::deterministic();
    let request = fri07_c02_collapse_round_request();
    let mut tree = fri07_c03_composed_layout_tree::<S>(case, 70.0);
    let initial =
        compute_layout(&tree, 1, request).expect("initial composed state layout succeeds");
    initial
        .apply_to(&mut tree)
        .expect("initial composed state batch commit succeeds");

    tree.requests.borrow_mut().clear();
    tree.measure_mode.set(mode);
    let retained_before_failure = tree.retained.clone();
    let error = compute_layout_invalidated(&tree, 1, request, &[1, 2])
        .expect_err("composed provider failure returns no partial batch");
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(error) if *error == expected_error
    ));
    assert_eq!(
        tree.retained, retained_before_failure,
        "failed composed layout commits neither partial output nor cache"
    );
    assert_eq!(
        fri07_c03_composed_state_measurements(&tree).len(),
        expected_requests,
        "failure occurs at its exact bounded measurement phase"
    );

    tree.requests.borrow_mut().clear();
    tree.measure_mode.set(Fri07C03ComposedMeasureMode::Values);
    let recovery = compute_layout_invalidated(&tree, 1, request, &[1, 2])
        .expect("composed state recovers after provider failure");
    let mut fresh_tree = fri07_c03_composed_layout_tree::<S>(case, 70.0);
    let fresh = compute_layout_invalidated(&fresh_tree, 1, request, &[1, 2])
        .expect("fresh composed state layout succeeds");
    assert_eq!(recovery.unrounded_entries(), fresh.unrounded_entries());
    assert_eq!(recovery.final_entries(), fresh.final_entries());
    assert_eq!(
        recovery.unrounded_inline_fragments(),
        fresh.unrounded_inline_fragments()
    );
    assert_eq!(
        recovery.final_inline_fragments(),
        fresh.final_inline_fragments()
    );
    recovery
        .apply_to(&mut tree)
        .expect("recovery batch commit succeeds");
    fresh
        .apply_to(&mut fresh_tree)
        .expect("fresh batch commit succeeds");
    assert_eq!(tree.retained.unrounded, fresh_tree.retained.unrounded);
    assert_eq!(
        tree.retained.final_outputs,
        fresh_tree.retained.final_outputs
    );
    let recovered_warm = compute_layout(&tree, 1, request)
        .expect("recovered composed cache serves a complete warm layout");
    let fresh_warm = compute_layout(&fresh_tree, 1, request)
        .expect("fresh composed cache serves a complete warm layout");
    assert_eq!(
        recovered_warm.unrounded_entries(),
        fresh_warm.unrounded_entries(),
        "recovery cache behavior matches a fresh tree"
    );
    assert_eq!(recovered_warm.final_entries(), fresh_warm.final_entries());
}

#[test]
fn fri07_c03_composed_state_intrinsic_and_second_round_failures_are_atomic_and_recoverable() {
    for (mode, expected_error, expected_requests) in [
        (
            Fri07C03ComposedMeasureMode::FailIntrinsic,
            Fri07C03ComposedMeasureError::Intrinsic,
            1,
        ),
        (
            Fri07C03ComposedMeasureMode::FailSecondRound,
            Fri07C03ComposedMeasureError::SecondRound,
            3,
        ),
    ] {
        assert_fri07_c03_composed_state_failure_is_atomic::<f32>(
            mode,
            expected_error,
            expected_requests,
        );
        assert_fri07_c03_composed_state_failure_is_atomic::<f64>(
            mode,
            expected_error,
            expected_requests,
        );
    }
}

fn fri07_c03_composed_state_batch<S: LayoutScalar>(
    case: Fri07C03ComposedCase,
    collapsed_main: f64,
    absolute_collapse: FlexItemCollapse,
) -> CompletedLayoutBatchOf<u32, S> {
    let mut tree = fri07_c03_composed_layout_tree::<S>(case, collapsed_main);
    let mut absolute = tree.tree.node_input(5).clone();
    absolute.flex_item_collapse = absolute_collapse;
    tree.tree = core::mem::take(&mut tree.tree).style(5, absolute);
    compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect("composed state control layout succeeds")
}

fn assert_fri07_c03_composed_state_settlement_and_inert_absolute<S: LayoutScalar>() {
    let case = Fri07C03ComposedCase {
        overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
        container_main: 120.5,
        ..Fri07C03ComposedCase::deterministic()
    };
    let baseline = fri07_c03_composed_state_batch::<S>(case, 70.25, FlexItemCollapse::Normal);
    let hostile = fri07_c03_composed_state_batch::<S>(case, 370.25, FlexItemCollapse::Normal);
    assert_eq!(baseline.unrounded_entries(), hostile.unrounded_entries());
    assert_eq!(baseline.final_entries(), hostile.final_entries());
    let root = fri07_c01_composition_output(baseline.unrounded_entries(), 1);
    let scroll = root
        .scroll_geometry
        .expect("composed overflow control publishes scroll geometry");
    assert_eq!(scroll.used_overflow_x(), Overflow::Auto);
    assert_eq!(scroll.used_overflow_y(), Overflow::Scroll);
    assert_eq!(scroll.scrollbar_size().width, S::from_f64(3.0));
    assert_eq!(
        fri07_c01_composition_output(baseline.unrounded_entries(), 4),
        NodeOutputOf::with_source_index(case.source_index(4))
    );
    assert_eq!(
        fri07_c01_composition_output(baseline.final_entries(), 4),
        NodeOutputOf::with_source_index(case.source_index(4))
    );

    let collapsed_absolute =
        fri07_c03_composed_state_batch::<S>(case, 70.25, FlexItemCollapse::Collapsed);
    assert_eq!(
        baseline.unrounded_entries(),
        collapsed_absolute.unrounded_entries(),
        "collapse remains inert on the composed absolute child"
    );
    assert_eq!(baseline.final_entries(), collapsed_absolute.final_entries());
}

#[test]
fn fri07_c03_composed_state_settlement_excludes_collapsed_facts_and_absolute_is_inert() {
    assert_fri07_c03_composed_state_settlement_and_inert_absolute::<f32>();
    assert_fri07_c03_composed_state_settlement_and_inert_absolute::<f64>();
}

fn fri07_c03_composed_state_assert_unsupported_basis(
    flex_basis: FlexBasisOf<f64>,
    behavior: SizingBehavior,
) {
    let case = Fri07C03ComposedCase::deterministic();
    let mut tree = fri07_c03_composed_layout_tree::<f64>(case, 70.0);
    let mut intrinsic = tree.tree.node_input(2).clone();
    intrinsic.flex_basis = flex_basis;
    tree.tree = core::mem::take(&mut tree.tree).style(2, intrinsic);
    let error = compute_layout(&tree, 1, fri07_c02_collapse_round_request())
        .expect_err("later-owned flex basis remains unsupported in the composed tree");
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    let LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected exact sizing capability payload, got {error:?}");
    };
    assert_eq!(unsupported.property(), SizingProperty::FlexBasis);
    assert_eq!(unsupported.behavior(), behavior);
    assert_eq!(unsupported.algorithm(), SizingAlgorithm::Flex);
    assert_eq!(unsupported.axis(), PhysicalAxis::Horizontal);
}

#[test]
fn fri07_c03_composed_state_later_owned_flex_basis_payloads_remain_exact() {
    let sizing = || {
        SizingCalculationOf::value(
            LengthPercentageOf::px(10.0).expect("finite composed sizing calculation"),
        )
    };
    for (flex_basis, behavior) in [
        (FlexBasisOf::STRETCH, SizingBehavior::Stretch),
        (FlexBasisOf::FIT_CONTENT, SizingBehavior::FitContent),
        (FlexBasisOf::CONTAIN, SizingBehavior::Contain),
        (
            FlexBasisOf::fit_content_function(sizing()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri07_c03_composed_state_assert_unsupported_basis(flex_basis, behavior);
    }

    let calc = CalcSizeCalculationOf::value(LengthPercentageOf::ZERO);
    for (basis, payload) in [
        (FlexBasisCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
        (FlexBasisCalcBasis::Content, CalcSizeBehaviorBasis::Content),
        (
            FlexBasisCalcBasis::MinContent,
            CalcSizeBehaviorBasis::MinContent,
        ),
        (
            FlexBasisCalcBasis::MaxContent,
            CalcSizeBehaviorBasis::MaxContent,
        ),
        (FlexBasisCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
        (
            FlexBasisCalcBasis::FitContent,
            CalcSizeBehaviorBasis::FitContent,
        ),
        (FlexBasisCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
    ] {
        fri07_c03_composed_state_assert_unsupported_basis(
            FlexBasisOf::calc_size(basis, calc.clone()).expect("valid composed calc-size"),
            SizingBehavior::CalcSize(payload),
        );
    }
}

fn assert_fri06_mr02_geometry_error_flex_own<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite();
    let style = NodeInputOf {
        display: Display::Flex,
        size: Size::new(PreferredSizeOf::px(largest), PreferredSizeOf::px(S::ONE)),
        padding: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        border: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        ..NodeInputOf::default()
    };

    for (run_mode, operation, invariant) in [
        (
            RunMode::PerformRootLayout,
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        (
            RunMode::PerformLayout,
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        ),
    ] {
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(7, [])
            .style(7, style.clone());
        let error = compute_flex(&mut tree, 7, fri06_mr02_geometry_error_input(run_mode))
            .expect_err("overflowing flex geometry must fail");

        fri06_mr02_geometry_error_assert(error, LayoutErrorSiteOf::Node(7), operation, invariant);
    }
}

fn assert_fri06_mr02_geometry_error_flex_child<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(7, [11])
        .children(11, [])
        .style(
            7,
            NodeInputOf {
                display: Display::Flex,
                size: size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .style(11, NodeInputOf::default())
        .measure(
            11,
            ComputeOutputOf::from_sizes(
                Size::new(S::from_f64(10.0), S::from_f64(10.0)),
                Size::splat(S::INFINITY),
            ),
        );
    let error = compute_flex(
        &mut tree,
        7,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            size.map(Some),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            size.map(AvailableOf::definite),
        ),
    )
    .expect_err("invalid retained flex child geometry must fail");

    fri06_mr02_geometry_error_assert(
        error,
        LayoutErrorSiteOf::ContainerSubject {
            container: 7,
            subject: 11,
        },
        LayoutOperation::ChildLayout,
        LayoutInternalInvariant::InvalidBlockScrollGeometry,
    );
}

fn assert_fri08_c07_t02_scroll_source_flex_paths<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let size = Size::new(scalar(100.0), scalar(80.0));
    let flow_axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl);
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let scroll_margin =
        ScrollMarginOf::try_new(scalar(-1.0), scalar(2.0), scalar(-3.0), scalar(4.0)).unwrap();
    let mut tree = OracleTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                flex_direction: FlexDirection::RowReverse,
                flex_wrap: FlexWrap::WrapReverse,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                overflow_clip_margin: OverflowClipMarginOf::try_new(
                    OverflowClipBox::ContentBox,
                    scalar(2.0),
                )
                .unwrap(),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(7.0)).unwrap(),
                size: size.map(PreferredSizeOf::px),
                padding: Edges::all(LengthOf::px(scalar(3.0))),
                scroll_padding: ScrollPaddingOf::new(
                    ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(1.0)).unwrap()),
                    ScrollPaddingValueOf::AUTO,
                    ScrollPaddingValueOf::AUTO,
                    ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(4.0)).unwrap()),
                ),
                scroll_snap_type: ScrollSnapType::Enabled {
                    axis: ScrollSnapAxis::Inline,
                    strictness: ScrollSnapStrictness::Proximity,
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: Size::new(
                    PreferredSizeOf::px(scalar(140.0)),
                    PreferredSizeOf::px(scalar(90.0)),
                ),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
                scroll_margin,
                scroll_snap_align: snap_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                position: Position::Absolute,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: Size::new(
                    PreferredSizeOf::px(scalar(30.0)),
                    PreferredSizeOf::px(scalar(20.0)),
                ),
                inset: Edges::new(
                    LengthAutoOf::px(scalar(5.0)),
                    LengthAutoOf::AUTO,
                    LengthAutoOf::AUTO,
                    LengthAutoOf::px(scalar(6.0)),
                ),
                scroll_margin,
                scroll_snap_align: snap_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes(
                Size::new(scalar(30.0), scalar(20.0)),
                Size::new(scalar(42.0), scalar(31.0)),
            ),
        );

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            size.map(Some),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            size.map(AvailableOf::definite),
        ),
    )
    .unwrap();

    let container = output.scroll_geometry.unwrap();
    assert_eq!(container.flow_axes(), flow_axes);
    assert!(container.overflow_clip().x().is_some());
    assert!(container.overflow_clip().y().is_some());
    assert_ne!(container.scrollbar_size(), Size::ZERO);
    assert_eq!(container.resolved_scroll_padding().top, scalar(1.0));
    assert_eq!(container.resolved_scroll_padding().left, scalar(4.0));
    assert_eq!(
        container.scroll_snap_type(),
        ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Inline,
            strictness: ScrollSnapStrictness::Proximity,
        }
    );
    let container_range = container.physical_range();
    assert!(container_range.x().minimum() <= S::ZERO);
    assert!(container_range.y().minimum() <= S::ZERO);

    let existing = tree.layout(1).unwrap();
    let existing_geometry = existing.scroll_geometry.unwrap();
    assert_eq!(existing_geometry.border_box().size(), existing.size);
    assert_eq!(existing_geometry.target().flow_axes(), flow_axes);
    assert_eq!(existing_geometry.target().scroll_margin(), scroll_margin);
    assert_eq!(existing_geometry.target().snap_align(), snap_align);

    let reconstructed = tree.layout(2).unwrap();
    let reconstructed_geometry = reconstructed.scroll_geometry.unwrap();
    assert_eq!(reconstructed_geometry.flow_axes(), flow_axes);
    assert_eq!(
        reconstructed_geometry.target().border_box().size(),
        reconstructed.size
    );
    assert_eq!(
        reconstructed_geometry.target().scroll_margin(),
        scroll_margin
    );
    assert_eq!(reconstructed_geometry.target().snap_align(), snap_align);
    assert_eq!(
        reconstructed_geometry.target().snap_stop(),
        ScrollSnapStop::Always
    );
    assert_eq!(
        reconstructed_geometry.scrollable_overflow().size(),
        Size::new(scalar(42.0), scalar(31.0))
    );
}

#[test]
fn fri08_c07_t02_scroll_source_flex_preserves_existing_reconstruction_and_origins() {
    assert_fri08_c07_t02_scroll_source_flex_paths::<f32>();
    assert_fri08_c07_t02_scroll_source_flex_paths::<f64>();
}

#[test]
fn fri08_c07_t02_scroll_source_flex_preserves_caller_local_errors() {
    assert_fri06_mr02_geometry_error_flex_own::<f32>();
    assert_fri06_mr02_geometry_error_flex_own::<f64>();
    assert_fri06_mr02_geometry_error_flex_child::<f32>();
    assert_fri06_mr02_geometry_error_flex_child::<f64>();
}

#[test]
fn fri06_mr02_geometry_error_flex_own_preserves_root_and_child_mapping_both_scalars() {
    assert_fri06_mr02_geometry_error_flex_own::<f32>();
    assert_fri06_mr02_geometry_error_flex_own::<f64>();
}

#[test]
fn fri06_mr02_geometry_error_flex_child_preserves_container_subject_both_scalars() {
    assert_fri06_mr02_geometry_error_flex_child::<f32>();
    assert_fri06_mr02_geometry_error_flex_child::<f64>();
}

#[test]
fn fri08_c07_t05_scroll_fixture_flex_assertion_preserves_error_identity() {
    assert_fri06_mr02_geometry_error_flex_own::<f32>();
    assert_fri06_mr02_geometry_error_flex_own::<f64>();
    assert_fri06_mr02_geometry_error_flex_child::<f32>();
    assert_fri06_mr02_geometry_error_flex_child::<f64>();
}

fn fri06_mr02_scroll_padding_cases<S: LayoutScalar>() -> [(ScrollPaddingOf<S>, Edges<S>); 2] {
    let [first, second] = scroll_padding_inputs();

    [
        (
            first,
            Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO),
        ),
        (
            second,
            Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
        ),
    ]
}

fn assert_fri06_mr02_scroll_padding_flex<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    for (scroll_padding, expected) in fri06_mr02_scroll_padding_cases() {
        let style = NodeInputOf::<S> {
            display: Display::Flex,
            size: Size::new(
                PreferredSizeOf::px(size.width),
                PreferredSizeOf::px(size.height),
            ),
            scroll_padding,
            ..NodeInputOf::default()
        };
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [])
            .style(0, style);
        let output = compute_flex(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                size.map(Some),
                size.map(Some),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                size.map(AvailableOf::definite),
            ),
        )
        .expect("flex scroll-padding characterization succeeds");
        let geometry = output
            .scroll_geometry
            .expect("performed flex layout emits geometry");

        assert_eq!(geometry.resolved_scroll_padding(), expected);
    }
}

#[test]
fn fri06_mr02_scroll_padding_flex_preserves_auto_and_value_on_each_physical_edge() {
    assert_fri06_mr02_scroll_padding_flex::<f32>();
    assert_fri06_mr02_scroll_padding_flex::<f64>();
}

#[test]
fn fri08_c07_t05_scroll_fixture_flex_rows_preserve_exact_auto_and_value_edges() {
    fn assert_rows<S: LayoutScalar>() {
        assert_scroll_padding_inputs_exact::<S>();
        assert_eq!(
            fri06_mr02_scroll_padding_cases::<S>().map(|(_, expected)| expected),
            [
                Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO,),
                Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
            ]
        );
    }

    assert_rows::<f32>();
    assert_rows::<f64>();
}

#[test]
fn flex_content_size_includes_visible_child_overflow_content() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(40.0, 10.0)
    );
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn flex_final_content_size_uses_rerun_output() {
    let mut tree = FlexTree::default();
    tree.insert_children(0, vec![1]);
    tree.insert_children(1, vec![]);
    tree.insert_style(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        1,
        NodeInput {
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            1,
            OracleMeasurementOf::new(ComputeOutput::from_sizes(
                Size::new(80.0, 40.0),
                Size::new(80.0, 40.0),
            ))
            .run_mode(RunMode::PerformLayout)
            .known(Size::new(Some(80.0), Some(10.0))),
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
        );

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(80.0), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(80.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert!(tree.inputs(1).iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize && input.known().width == Some(80.0)
    }));
    assert!(tree.inputs(1).iter().any(|input| {
        input.run_mode() == RunMode::PerformLayout && input.known().width == Some(80.0)
    }));
    assert_eq!(output.content_size.height, 40.0);
}

#[test]
fn flex_relative_child_inset_offsets_final_layout_location() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(7.0, 3.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn flex_relative_child_trailing_inset_offsets_negative() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            inset: Edges {
                right: LengthAuto::px(5.0),
                bottom: LengthAuto::px(2.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(-5.0, -2.0)
    );
}

#[test]
fn flex_container_reserves_scrollbar_gutter_from_inner_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(0.0), PreferredSize::px(10.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::new(100.0, 40.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(90.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::ZERO
    );
}

#[test]
fn flex_scrollbar_gutter_uses_left_inset_for_rtl_containers() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn flex_child_layout_records_scrollbar_size_for_scroll_overflow() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(7.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2)
            .expect("child layout is staged")
            .scrollbar_size(),
        Size::new(7.0, 7.0)
    );
}

#[test]
fn flex_absolute_child_uses_insets_without_affecting_flow() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(25.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(12.0)),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 12.0), Size::new(80.0, 32.0)),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::new(100.0, 41.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(25.0, 10.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(7.0, 9.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(20.0, 12.0)
    );
    assert_eq!(tree.inputs(3)[0].known(), Size::new(Some(20.0), Some(12.0)));
}

#[test]
fn flex_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(400.0), PreferredSize::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges::all(LengthAuto::percent(0.05)),
            aspect_ratio: AspectRatio::new(3.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(2)[0].known(),
        Size::new(Some(360.0), Some(120.0))
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(20.0, 15.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(360.0, 120.0)
    );
}

#[test]
fn flex_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(400.0), PreferredSize::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::percent(0.1),
                right: LengthAuto::percent(0.1),
                top: LengthAuto::percent(0.05),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(PreferredSize::percent(0.4), PreferredSize::AUTO),
            aspect_ratio: AspectRatio::new(3.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(2)[0].known(),
        Size::new(Some(160.0), Some(160.0 / 3.0))
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(200.0, 15.0)
    );
}

#[test]
fn flex_absolute_child_max_height_shrinks_flex_grandchild() {
    let mut tree = RecursiveTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![3]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
            flex_direction: FlexDirection::Column,
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            flex_direction: FlexDirection::Column,
            inset: Edges {
                bottom: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            max_size: Size::new(MaxSize::NONE, MaxSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            flex_basis: FlexBasis::px(150.0),
            flex_shrink: FlexShrinkOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 80.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 100.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(100.0, 100.0)
    );
}

#[test]
fn flex_absolute_child_cross_alignment_honors_wrap_reverse() {
    fn layout_child(
        align_self: AlignItems,
        flex_direction: FlexDirection,
        layout_direction: Direction,
    ) -> NodeOutput {
        let mut tree = FlexTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    direction: layout_direction,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    flex_direction,
                    flex_wrap: FlexWrap::WrapReverse,
                    ..NodeInput::default()
                },
            )
            .style(
                2,
                NodeInput {
                    direction: layout_direction,
                    position: Position::Absolute,
                    align_self: Some(align_self),
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                    ..NodeInput::default()
                },
            )
            .measure(2, ComputeOutput::from_outer_size(Size::splat(20.0)));

        compute_flex(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(100.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(Available::definite(100.0), Available::MAX_CONTENT),
            ),
        )
        .unwrap();

        tree.layout(2).expect("absolute child layout is staged")
    }

    let default_layout = layout_child(AlignItems::Stretch, FlexDirection::Row, Direction::Ltr);
    assert_eq!(default_layout.location, Point::new(0.0, 80.0));
    assert_eq!(default_layout.size, Size::new(20.0, 20.0));

    let flex_end_layout = layout_child(AlignItems::FlexEnd, FlexDirection::Row, Direction::Ltr);
    assert_eq!(flex_end_layout.location, Point::new(0.0, 0.0));
    assert_eq!(flex_end_layout.size, Size::new(20.0, 20.0));

    let column_rtl_layout =
        layout_child(AlignItems::Stretch, FlexDirection::Column, Direction::Rtl);
    assert_eq!(column_rtl_layout.location, Point::new(0.0, 0.0));
    assert_eq!(column_rtl_layout.size, Size::new(20.0, 20.0));

    let column_rtl_flex_end_layout =
        layout_child(AlignItems::FlexEnd, FlexDirection::Column, Direction::Rtl);
    assert_eq!(column_rtl_flex_end_layout.location, Point::new(80.0, 0.0));
    assert_eq!(column_rtl_flex_end_layout.size, Size::new(20.0, 20.0));
}

#[test]
fn flex_absolute_child_cross_start_margin_uses_physical_edge_in_rtl_column() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            justify_content: Some(AlignContent::FlexEnd),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            direction: Direction::Rtl,
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::px(10.0),
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(100.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(90.0, 80.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(10.0, 10.0)
    );
}

#[test]
fn flex_absolute_child_uses_min_size_when_min_exceeds_max_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                right: LengthAuto::px(10.0),
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            min_size: Size::new(MinSize::px(50.0), MinSize::px(60.0)),
            max_size: Size::new(MaxSize::px(40.0), MaxSize::px(30.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(100.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(40.0, 30.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(50.0, 60.0)
    );
}

#[test]
fn flex_absolute_child_size_cannot_shrink_below_padding_and_border() {
    fn tree_with_child(child_style: NodeInput) -> FlexTree {
        let mut tree = FlexTree::default();
        tree.insert_children(1, vec![2]);
        tree.insert_children(2, vec![]);
        tree.insert_style(1, NodeInput::default());
        tree.insert_style(2, child_style);
        tree
    }

    fn run(tree: &mut FlexTree) {
        compute_flex(
            tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            ),
        )
        .unwrap();
    }

    let padding = Edges {
        top: Length::px(2.0),
        right: Length::px(4.0),
        bottom: Length::px(6.0),
        left: Length::px(8.0),
    };
    let border = Edges {
        top: Length::px(1.0),
        right: Length::px(3.0),
        bottom: Length::px(5.0),
        left: Length::px(7.0),
    };

    let mut authored_size = tree_with_child(NodeInput {
        position: Position::Absolute,
        size: Size::new(PreferredSize::px(12.0), PreferredSize::px(12.0)),
        padding,
        border,
        ..NodeInput::default()
    });
    run(&mut authored_size);
    assert_eq!(
        authored_size.inputs(2)[0].known(),
        Size::new(Some(22.0), Some(14.0))
    );
    assert_eq!(
        authored_size
            .layout(2)
            .expect("child layout is staged")
            .size,
        Size::new(22.0, 14.0)
    );

    let mut max_size = tree_with_child(NodeInput {
        position: Position::Absolute,
        max_size: Size::new(MaxSize::px(12.0), MaxSize::px(12.0)),
        padding,
        border,
        ..NodeInput::default()
    });
    run(&mut max_size);
    assert_eq!(
        max_size.layout(2).expect("child layout is staged").size,
        Size::new(22.0, 14.0)
    );
}

#[test]
fn flex_absolute_child_layout_records_scrollbar_size_for_scroll_overflow() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(8.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2)
            .expect("child layout is staged")
            .scrollbar_size(),
        Size::new(8.0, 8.0)
    );
}

#[test]
fn flex_absolute_child_can_resolve_from_trailing_insets() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                right: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(72.0, 34.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn fri07_c01_absolute_auto_margin_original_auto_end_inset_zeroes_inline_margins() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        0.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        0.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
}

#[test]
fn flex_absolute_child_without_insets_uses_flex_alignment() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            justify_content: Some(AlignContent::Center),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(40.0, 15.0)
    );
}

fn fri05_c04_empty_flex_output(style: NodeInput, size: Size<f32>) -> ComputeOutput {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [])
        .style(0, style);
    compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
        .expect("FRI-05 empty flex layout succeeds")
}

fn fri05_c04_flex_gutter_at(
    gutters: ScrollbarGutterRects,
    side: PhysicalSide,
) -> Option<ScrollRect> {
    match side {
        PhysicalSide::Top => gutters.top(),
        PhysicalSide::Right => gutters.right(),
        PhysicalSide::Bottom => gutters.bottom(),
        PhysicalSide::Left => gutters.left(),
    }
}

fn fri05_c04_assert_zero_range(geometry: ScrollGeometry, context: &str) {
    let range = geometry.physical_range();
    assert_eq!(
        (
            range.x().minimum(),
            range.x().maximum(),
            range.y().minimum(),
            range.y().maximum(),
        ),
        (0.0, 0.0, 0.0, 0.0),
        "{context}"
    );
}

#[test]
fn fri05_c04_flex_geometry_empty_and_simple_nonoverflowing_publish_canonical_boxes_all_flows() {
    let size = Size::new(100.0, 80.0);
    let border = Edges::all(Length::px(2.0));
    let padding = Edges::all(Length::px(3.0));
    let scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let expected_border_box = ScrollRect::try_new(Point::ZERO, size).unwrap();
    let expected_padding_box =
        ScrollRect::try_new(Point::new(2.0, 2.0), Size::new(96.0, 76.0)).unwrap();
    let expected_content_box =
        ScrollRect::try_new(Point::new(5.0, 5.0), Size::new(90.0, 70.0)).unwrap();

    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let style = NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            border,
            padding,
            scroll_margin,
            scroll_snap_align: snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInput::default()
        };
        let output = fri05_c04_empty_flex_output(style.clone(), size);
        let geometry = output
            .scroll_geometry
            .expect("performed empty flex emits canonical geometry");

        assert_eq!(geometry.flow_axes(), flow_axes);
        assert_eq!(geometry.used_overflow_x(), Overflow::Visible);
        assert_eq!(geometry.used_overflow_y(), Overflow::Visible);
        assert_eq!(geometry.border_box(), expected_border_box);
        assert_eq!(geometry.padding_box(), expected_padding_box);
        assert_eq!(geometry.content_box(), expected_content_box);
        assert_eq!(geometry.scrollport(), expected_padding_box);
        assert_eq!(geometry.scrollable_overflow(), expected_padding_box);
        assert_eq!(geometry.overflow_clip().x(), None);
        assert_eq!(geometry.overflow_clip().y(), None);
        assert_eq!(geometry.scrollbar_size(), Size::ZERO);
        assert_eq!(geometry.target().border_box(), expected_border_box);
        assert_eq!(geometry.target().flow_axes(), flow_axes);
        assert_eq!(geometry.target().scroll_margin(), scroll_margin);
        assert_eq!(geometry.target().snap_align(), snap_align);
        assert_eq!(geometry.target().snap_stop(), ScrollSnapStop::Always);
        fri05_c04_assert_zero_range(geometry, &format!("empty {flow_axes:?}"));

        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(0, style)
            .style(
                1,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(8.0)),
                    min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                    ..NodeInput::default()
                },
            );
        let simple = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
            .expect("FRI-05 simple flex layout succeeds");
        let simple_geometry = simple
            .scroll_geometry
            .expect("performed simple flex emits canonical geometry");
        assert_eq!(simple_geometry.border_box(), expected_border_box);
        assert_eq!(simple_geometry.padding_box(), expected_padding_box);
        assert_eq!(simple_geometry.content_box(), expected_content_box);
        assert_eq!(simple_geometry.scrollport(), expected_padding_box);
        assert_eq!(simple_geometry.scrollable_overflow(), expected_padding_box);
        assert_eq!(simple_geometry.target().border_box(), expected_border_box);
        fri05_c04_assert_zero_range(simple_geometry, &format!("simple {flow_axes:?}"));
    }
}

#[test]
fn fri05_c04_flex_geometry_forced_stable_both_zero_and_tiny_saturate_all_flows() {
    let size = Size::new(100.0, 80.0);
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let style = |overflow, gutter, width| NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_gutter: gutter,
            scrollbar_width: ScrollbarWidth::try_new(width).unwrap(),
            size: Size::new(
                PreferredSize::px(size.width),
                PreferredSize::px(size.height),
            ),
            ..NodeInput::default()
        };
        let forced = fri05_c04_empty_flex_output(
            style(
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Scroll),
                ScrollbarGutter::Auto,
                7.0,
            ),
            size,
        )
        .scroll_geometry
        .expect("forced-scroll flex emits geometry");
        let stable = fri05_c04_empty_flex_output(
            style(
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
                ScrollbarGutter::Stable,
                7.0,
            ),
            size,
        )
        .scroll_geometry
        .expect("stable-gutter flex emits geometry");
        let both = fri05_c04_empty_flex_output(
            style(
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
                ScrollbarGutter::StableBothEdges,
                7.0,
            ),
            size,
        )
        .scroll_geometry
        .expect("both-edge flex emits geometry");

        for (case, geometry, expected_sides) in [
            ("forced", forced, vec![flow_axes.inline_end()]),
            ("stable", stable, vec![flow_axes.inline_end()]),
            (
                "both",
                both,
                vec![flow_axes.inline_start(), flow_axes.inline_end()],
            ),
        ] {
            assert_eq!(geometry.flow_axes(), flow_axes, "{case}/{flow_axes:?}");
            assert_eq!(geometry.border_box(), geometry.padding_box());
            assert_eq!(geometry.content_box(), geometry.scrollport());
            let scrollport = geometry.scrollport();
            let x_clip = geometry.overflow_clip().x().expect("x clip is present");
            let y_clip = geometry.overflow_clip().y().expect("y clip is present");
            assert_eq!(
                (x_clip.minimum(), x_clip.maximum()),
                (
                    scrollport.origin().x,
                    scrollport.origin().x + scrollport.size().width,
                )
            );
            assert_eq!(
                (y_clip.minimum(), y_clip.maximum()),
                (
                    scrollport.origin().y,
                    scrollport.origin().y + scrollport.size().height,
                )
            );
            assert_eq!(geometry.target().border_box(), geometry.border_box());
            assert_eq!(geometry.target().flow_axes(), flow_axes);
            for side in [
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
            ] {
                assert_eq!(
                    fri05_c04_flex_gutter_at(geometry.gutters(), side).is_some(),
                    expected_sides.contains(&side),
                    "unexpected {side:?} gutter for {case}/{flow_axes:?}: {geometry:#?}"
                );
            }
            fri05_c04_assert_zero_range(geometry, &format!("{case} {flow_axes:?}"));
        }

        let expected_one_edge = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(7.0, 0.0),
            PhysicalAxis::Vertical => Size::new(0.0, 7.0),
        };
        assert_eq!(forced.scrollbar_size(), expected_one_edge, "{flow_axes:?}");
        assert_eq!(stable.scrollbar_size(), expected_one_edge, "{flow_axes:?}");
        assert_eq!(both.scrollbar_size(), expected_one_edge + expected_one_edge);

        let zero_width = fri05_c04_empty_flex_output(
            style(
                computed_overflow(Overflow::Scroll, Overflow::Scroll),
                ScrollbarGutter::StableBothEdges,
                0.0,
            ),
            size,
        )
        .scroll_geometry
        .expect("zero-width scrollbar flex emits geometry");
        assert_eq!(zero_width.scrollbar_size(), Size::ZERO);
        assert_eq!(zero_width.scrollport(), zero_width.padding_box());
        assert_eq!(zero_width.gutters().top(), None);
        assert_eq!(zero_width.gutters().right(), None);
        assert_eq!(zero_width.gutters().bottom(), None);
        assert_eq!(zero_width.gutters().left(), None);
        fri05_c04_assert_zero_range(zero_width, &format!("zero width {flow_axes:?}"));

        let tiny_size = Size::new(5.0, 3.0);
        let tiny = fri05_c04_empty_flex_output(
            NodeInput {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                size: Size::new(
                    PreferredSize::px(tiny_size.width),
                    PreferredSize::px(tiny_size.height),
                ),
                ..NodeInput::default()
            },
            tiny_size,
        )
        .scroll_geometry
        .expect("tiny both-edge flex emits geometry");
        let expected_tiny_reservation = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(tiny_size.width, 0.0),
            PhysicalAxis::Vertical => Size::new(0.0, tiny_size.height),
        };
        assert_eq!(tiny.scrollbar_size(), expected_tiny_reservation);
        assert_eq!(
            match flow_axes.inline_axis() {
                PhysicalAxis::Horizontal => tiny.scrollport().size().width,
                PhysicalAxis::Vertical => tiny.scrollport().size().height,
            },
            0.0,
            "tiny inline scrollport saturates for {flow_axes:?}"
        );
        assert!(fri05_c04_flex_gutter_at(tiny.gutters(), flow_axes.inline_start()).is_some());
        assert!(fri05_c04_flex_gutter_at(tiny.gutters(), flow_axes.inline_end()).is_some());
        fri05_c04_assert_zero_range(tiny, &format!("tiny {flow_axes:?}"));

        let zero_size = Size::ZERO;
        let zero = fri05_c04_empty_flex_output(
            NodeInput {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(0.0)),
                ..NodeInput::default()
            },
            zero_size,
        )
        .scroll_geometry
        .expect("zero-size flex emits ordered geometry");
        assert_eq!(zero.border_box().size(), Size::ZERO);
        assert_eq!(zero.padding_box().size(), Size::ZERO);
        assert_eq!(zero.content_box().size(), Size::ZERO);
        assert_eq!(zero.scrollport().size(), Size::ZERO);
        assert_eq!(zero.scrollbar_size(), Size::ZERO);
        assert_eq!(zero.gutters().top(), None);
        assert_eq!(zero.gutters().right(), None);
        assert_eq!(zero.gutters().bottom(), None);
        assert_eq!(zero.gutters().left(), None);
        fri05_c04_assert_zero_range(zero, &format!("zero box {flow_axes:?}"));
    }
}

fn fri05_c04_child_geometry_source(style: NodeInput, size: Size<f32>) -> ComputeOutput {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(9, [])
        .style(9, style);
    crate::compute_block(&mut tree, 9, fri05_c04_flex_input(size, flow_axes))
        .expect("child geometry source block lays out")
}

#[test]
fn fri05_c04_flex_child_geometry_direct_retains_in_flow_and_rebuilds_absolute_target() {
    let parent_size = Size::new(120.0, 80.0);
    let child_flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let in_flow_scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let in_flow_snap_align =
        ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::End);
    let in_flow_style = NodeInput {
        display: Display::Block,
        writing_mode: child_flow_axes.writing_mode(),
        direction: child_flow_axes.direction(),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        scrollbar_gutter: ScrollbarGutter::StableBothEdges,
        scrollbar_width: ScrollbarWidth::try_new(4.0).unwrap(),
        size: Size::new(PreferredSize::px(24.0), PreferredSize::px(18.0)),
        min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
        scroll_margin: in_flow_scroll_margin,
        scroll_snap_align: in_flow_snap_align,
        scroll_snap_stop: ScrollSnapStop::Always,
        ..NodeInput::default()
    };
    let absolute_size = Size::new(30.0, 20.0);
    let current_absolute_scroll_margin = ScrollMargin::try_new(8.0, 7.0, 6.0, 5.0).unwrap();
    let absolute_style = NodeInput {
        position: Position::Absolute,
        size: Size::new(
            PreferredSize::px(absolute_size.width),
            PreferredSize::px(absolute_size.height),
        ),
        inset: Edges::new(
            LengthAuto::px(3.0),
            LengthAuto::AUTO,
            LengthAuto::AUTO,
            LengthAuto::px(5.0),
        ),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        scrollbar_width: ScrollbarWidth::try_new(3.0).unwrap(),
        scroll_margin: current_absolute_scroll_margin,
        ..NodeInput::default()
    };
    let retained_absolute_scroll_margin = ScrollMargin::try_new(-5.0, 4.0, -3.0, 2.0).unwrap();
    let retained_absolute_snap_align =
        ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let stale_absolute = fri05_c04_child_geometry_source(
        NodeInput {
            position: Position::Relative,
            scroll_margin: retained_absolute_scroll_margin,
            scroll_snap_align: retained_absolute_snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..absolute_style.clone()
        },
        Size::new(10.0, 8.0),
    );
    let stale_border_box = stale_absolute
        .scroll_geometry
        .expect("source output has geometry")
        .border_box();

    let parent_flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: Size::new(
                    PreferredSize::px(parent_size.width),
                    PreferredSize::px(parent_size.height),
                ),
                ..NodeInput::default()
            },
        )
        .style(1, in_flow_style)
        .style(2, absolute_style)
        .measure(2, stale_absolute);
    compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(parent_size, parent_flow_axes),
    )
    .expect("flex child geometry layout succeeds");

    let in_flow = tree.layout(1).expect("in-flow child is staged");
    let in_flow_geometry = in_flow
        .scroll_geometry
        .expect("in-flow child retains canonical geometry");
    assert_eq!(in_flow_geometry.border_box().size(), in_flow.size);
    assert_eq!(
        in_flow_geometry.target().border_box(),
        in_flow_geometry.border_box()
    );
    assert_eq!(in_flow_geometry.target().flow_axes(), child_flow_axes);
    assert_eq!(
        in_flow_geometry.target().scroll_margin(),
        in_flow_scroll_margin
    );
    assert_eq!(in_flow_geometry.target().snap_align(), in_flow_snap_align);
    assert_eq!(
        in_flow_geometry.target().snap_stop(),
        ScrollSnapStop::Always
    );
    assert_eq!(in_flow.scrollbar_size(), in_flow_geometry.scrollbar_size());

    let absolute = tree.layout(2).expect("absolute child is staged");
    let absolute_geometry = absolute
        .scroll_geometry
        .expect("absolute child retains canonical geometry");
    assert_ne!(absolute_geometry.border_box(), stale_border_box);
    assert_eq!(absolute.size, absolute_size);
    assert_eq!(absolute_geometry.border_box().size(), absolute_size);
    assert_eq!(
        absolute_geometry.target().border_box(),
        absolute_geometry.border_box()
    );
    assert_eq!(
        absolute_geometry.target().scroll_margin(),
        retained_absolute_scroll_margin
    );
    assert_ne!(
        absolute_geometry.target().scroll_margin(),
        current_absolute_scroll_margin
    );
    assert_eq!(
        absolute_geometry.target().snap_align(),
        retained_absolute_snap_align
    );
    assert_eq!(
        absolute_geometry.target().snap_stop(),
        ScrollSnapStop::Always
    );
    assert_eq!(
        absolute.scrollbar_size(),
        absolute_geometry.scrollbar_size()
    );
}

fn fri05_c04_flex_child_geometry_tiny_absolute_styles(
    flow_axes: FlowAxes,
) -> (NodeInput, NodeInput) {
    (
        NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            flex_direction: FlexDirection::Column,
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            max_size: Size::new(MaxSize::NONE, MaxSize::px(5.0)),
            ..NodeInput::default()
        },
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(0.0), PreferredSize::px(0.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            inset: Edges::new(
                LengthAuto::AUTO,
                LengthAuto::AUTO,
                LengthAuto::px(0.0),
                LengthAuto::AUTO,
            ),
            ..NodeInput::default()
        },
    )
}

#[test]
fn fri05_c04_flex_child_geometry_direct_auto_max_tiny_gutter_keeps_absolute_inputs_non_negative_all_flows()
 {
    let available_size = Size::new(100.0, 100.0);

    for flow_axes in fri05_c04_flex_all_flow_axes() {
        let (root_style, absolute_style) =
            fri05_c04_flex_child_geometry_tiny_absolute_styles(flow_axes);
        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(0, root_style)
            .style(1, absolute_style);
        let output = compute_flex(
            &mut tree,
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                available_size.map(Some),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                available_size.map(Available::definite),
            ),
        )
        .unwrap_or_else(|error| panic!("tiny absolute flex succeeds for {flow_axes:?}: {error:?}"));

        assert_eq!(output.size, Size::new(100.0, 5.0), "{flow_axes:?}");
        let root_geometry = output
            .scroll_geometry
            .expect("performed flex retains final canonical geometry");
        assert_eq!(
            root_geometry.scrollport().size(),
            Size::new(90.0, 0.0),
            "{flow_axes:?}"
        );

        let absolute = tree
            .layout(1)
            .expect("tiny absolute child is staged without a negative basis");
        let absolute_geometry = absolute
            .scroll_geometry
            .expect("tiny absolute child retains canonical geometry");
        assert_eq!(absolute.size, Size::ZERO, "{flow_axes:?}");
        assert_eq!(absolute_geometry.border_box().size(), Size::ZERO);
        assert_eq!(
            absolute_geometry.target().border_box(),
            absolute_geometry.border_box()
        );
        assert_eq!(
            absolute.location.y,
            root_geometry.scrollport().origin().y + root_geometry.scrollport().size().height,
            "bottom: 0 uses the final saturated scrollport for {flow_axes:?}"
        );

        let child_input = tree
            .inputs(1)
            .iter()
            .find(|input| input.run_mode() == RunMode::PerformLayout)
            .expect("absolute child receives a perform-layout request");
        assert_eq!(
            child_input.parent(),
            root_geometry.content_box().size().map(Some),
            "final canonical content-box basis for {flow_axes:?}"
        );
        assert_eq!(
            child_input.available(),
            root_geometry.scrollport().size().map(Available::definite),
            "final canonical available space for {flow_axes:?}"
        );

        let mut ordinary_root = fri05_c04_flex_child_geometry_tiny_absolute_styles(flow_axes).0;
        ordinary_root.size.height = PreferredSize::px(80.0);
        ordinary_root.max_size.height = MaxSize::NONE;
        let ordinary_absolute = fri05_c04_flex_child_geometry_tiny_absolute_styles(flow_axes).1;
        let mut ordinary_tree = crate::test_support::layout_tree::OracleTree::new()
            .children(0, [1])
            .children(1, [])
            .style(0, ordinary_root)
            .style(1, ordinary_absolute);
        let ordinary = compute_flex(
            &mut ordinary_tree,
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                available_size.map(Some),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                available_size.map(Available::definite),
            ),
        )
        .unwrap_or_else(|error| {
            panic!("ordinary absolute flex succeeds for {flow_axes:?}: {error:?}")
        });
        let ordinary_geometry = ordinary
            .scroll_geometry
            .expect("ordinary flex retains canonical geometry");
        let ordinary_child = ordinary_tree
            .layout(1)
            .expect("ordinary absolute child remains staged");
        assert_eq!(ordinary.size, Size::new(100.0, 80.0), "{flow_axes:?}");
        assert_eq!(
            ordinary_child.location.y,
            ordinary_geometry.scrollport().origin().y
                + ordinary_geometry.scrollport().size().height,
            "ordinary bottom placement remains on the settled scrollport for {flow_axes:?}"
        );
    }
}

fn fri05_c04_positive_margin_rect(output: NodeOutput) -> ScrollRect {
    let top = output.margin.top.max(0.0);
    let right = output.margin.right.max(0.0);
    let bottom = output.margin.bottom.max(0.0);
    let left = output.margin.left.max(0.0);
    ScrollRect::try_new(
        Point::new(output.location.x - left, output.location.y - top),
        Size::new(
            output.size.width + left + right,
            output.size.height + top + bottom,
        ),
    )
    .unwrap()
}

fn fri05_c04_union_rects(rects: impl IntoIterator<Item = ScrollRect>) -> ScrollRect {
    let mut rects = rects.into_iter();
    let first = rects.next().expect("the test union is nonempty");
    let mut minimum = first.origin();
    let mut maximum = Point::new(
        first.origin().x + first.size().width,
        first.origin().y + first.size().height,
    );
    for rect in rects {
        minimum.x = minimum.x.min(rect.origin().x);
        minimum.y = minimum.y.min(rect.origin().y);
        maximum.x = maximum.x.max(rect.origin().x + rect.size().width);
        maximum.y = maximum.y.max(rect.origin().y + rect.size().height);
    }
    ScrollRect::try_new(
        minimum,
        Size::new(maximum.x - minimum.x, maximum.y - minimum.y),
    )
    .unwrap()
}

#[test]
fn fri05_c04_flex_contribution_positive_outsets_negative_margins_and_source_order_are_exact() {
    let size = Size::new(10.0, 10.0);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: size.map(PreferredSize::px),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                item_order: ItemOrder::new(10),
                size: Size::new(PreferredSize::px(7.0), PreferredSize::px(4.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                margin: Edges::new(
                    LengthAuto::px(3.0),
                    LengthAuto::px(5.0),
                    LengthAuto::px(2.0),
                    LengthAuto::px(4.0),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                item_order: ItemOrder::new(-10),
                size: Size::new(PreferredSize::px(6.0), PreferredSize::px(3.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                margin: Edges::new(
                    LengthAuto::px(-7.0),
                    LengthAuto::px(-11.0),
                    LengthAuto::px(-5.0),
                    LengthAuto::px(-13.0),
                ),
                ..NodeInput::default()
            },
        );

    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            size,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("flex contribution layout succeeds");
    let first = tree.layout(1).expect("first source output is retained");
    let second = tree.layout(2).expect("second source output is retained");
    assert_eq!(first.source_index, SourceIndex::new(0));
    assert_eq!(second.source_index, SourceIndex::new(1));

    let expected = fri05_c04_union_rects([
        ScrollRect::try_new(Point::ZERO, size).unwrap(),
        fri05_c04_positive_margin_rect(first),
        fri05_c04_positive_margin_rect(second),
    ]);
    let geometry = output.scroll_geometry.expect("flex geometry is present");
    assert_eq!(geometry.scrollable_overflow(), expected);
    let expected_maximum = Point::new(
        expected.origin().x + expected.size().width,
        expected.origin().y + expected.size().height,
    );
    assert_eq!(
        output.content_size,
        Size::new(
            expected_maximum.x.max(0.0) - expected.origin().x.min(0.0),
            expected_maximum.y.max(0.0) - expected.origin().y.min(0.0),
        ),
        "negative starts and positive ends remain independent"
    );
}

#[test]
fn fri05_c04_flex_contribution_terminal_padding_extends_only_the_final_in_flow_ends() {
    let size = Size::new(10.0, 8.0);
    let padding = Edges {
        right: Length::px(4.0),
        bottom: Length::px(3.0),
        ..Edges::all(Length::ZERO)
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: size.map(PreferredSize::px),
                padding,
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(12.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            size,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("terminal-padding flex layout succeeds");
    let child = tree.layout(1).unwrap();
    let overflow = output.scroll_geometry.unwrap().scrollable_overflow();

    assert_eq!(overflow.origin(), Point::ZERO);
    assert_eq!(
        overflow.size().width,
        child.location.x + child.size.width + 4.0
    );
    assert_eq!(
        overflow.size().height,
        child.location.y + child.size.height + 3.0
    );
}

fn fri05_c04_flex_nested_output(
    overflow: ComputedOverflow,
    child_size: Size<f32>,
) -> ComputeOutput {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: Size::ZERO.map(PreferredSize::px),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow,
                size: child_size.map(PreferredSize::px),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                align_self: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(30.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                ..NodeInput::default()
            },
        );
    compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            Size::ZERO,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("nested flex contribution layout succeeds")
}

#[test]
fn fri05_c04_flex_nested_visible_and_trapped_axes_preserve_zero_area_intervals_independently() {
    for (overflow, child_size, expected) in [
        (
            computed_overflow(Overflow::Visible, Overflow::Clip),
            Size::new(0.0, 5.0),
            Size::new(20.0, 0.0),
        ),
        (
            computed_overflow(Overflow::Clip, Overflow::Visible),
            Size::new(5.0, 0.0),
            Size::new(0.0, 30.0),
        ),
        (
            computed_overflow(Overflow::Clip, Overflow::Clip),
            Size::new(0.0, 5.0),
            Size::ZERO,
        ),
        (
            computed_overflow(Overflow::Hidden, Overflow::Scroll),
            Size::new(0.0, 5.0),
            Size::ZERO,
        ),
        (
            computed_overflow(Overflow::Scroll, Overflow::Auto),
            Size::new(5.0, 0.0),
            Size::ZERO,
        ),
        (
            computed_overflow(Overflow::Auto, Overflow::Hidden),
            Size::new(5.0, 0.0),
            Size::ZERO,
        ),
    ] {
        let output = fri05_c04_flex_nested_output(overflow, child_size);
        let geometry = output
            .scroll_geometry
            .expect("nested flex geometry is present");
        assert_eq!(geometry.scrollable_overflow().origin(), Point::ZERO);
        assert_eq!(
            geometry.scrollable_overflow().size(),
            expected,
            "{overflow:?}"
        );
        assert_eq!(output.content_size, expected, "{overflow:?}");
    }
}

#[test]
fn fri05_c04_flex_absolute_margin_and_visible_descendant_contribute_once_without_terminal_padding()
{
    let size = Size::ZERO;
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                size: size.map(PreferredSize::px),
                padding: Edges {
                    right: Length::px(4.0),
                    bottom: Length::px(3.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                overflow: ComputedOverflow::VISIBLE,
                size: Size::new(PreferredSize::px(5.0), PreferredSize::px(5.0)),
                inset: Edges::new(
                    LengthAuto::px(0.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(10.0),
                ),
                margin: Edges {
                    right: LengthAuto::px(7.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(9.0), PreferredSize::px(12.0)),
                ..NodeInput::default()
            },
        );
    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            size,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("absolute flex contribution layout succeeds");
    let absolute = tree.layout(1).expect("absolute output is retained");
    let own_margin = fri05_c04_positive_margin_rect(absolute);
    let own_max_x = own_margin.origin().x + own_margin.size().width;
    let geometry = output
        .scroll_geometry
        .expect("absolute flex geometry is present");

    assert_eq!(geometry.scrollable_overflow().origin(), Point::ZERO);
    assert_eq!(geometry.scrollable_overflow().size().width, own_max_x);
    assert_eq!(geometry.scrollable_overflow().size().height, 12.0);
    assert_eq!(output.content_size, geometry.scrollable_overflow().size());
    assert_ne!(geometry.scrollable_overflow().size().width, own_max_x + 4.0);
}

fn fri05_c04_flex_origin_output(
    flow_axes: FlowAxes,
    direction: FlexDirection,
    wrap: FlexWrap,
) -> (ScrollGeometry, ScrollGeometry) {
    let axes = FlexAxes::new(flow_axes, direction, wrap);
    let size = axes.size_from_main_cross(100.0, 80.0);
    let child_size = axes.size_from_main_cross(140.0, 60.0);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: fri05_c04_flex_overflow_at_flow_axes(
                    flow_axes,
                    Overflow::Scroll,
                    Overflow::Scroll,
                ),
                size: size.map(PreferredSize::px),
                flex_direction: direction,
                flex_wrap: wrap,
                align_content: Some(AlignContent::FlexStart),
                align_items: Some(AlignItems::FlexStart),
                justify_content: Some(AlignContent::FlexStart),
                ..NodeInput::default()
            },
        );
    for child in [1, 2] {
        tree = tree.style(
            child,
            NodeInput {
                size: child_size.map(PreferredSize::px),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
        .expect("origin-aware flex layout succeeds");
    let unrounded = output
        .scroll_geometry
        .expect("performed flex layout has geometry");
    tree.set_unrounded(
        0,
        NodeOutput {
            size: output.size,
            content_size: output.content_size,
            ..NodeOutput::new()
        }
        .with_scroll_geometry(Some(unrounded)),
    );
    crate::round_layout(&mut tree, 0).expect("canonical flex geometry rounds");
    let rounded = tree
        .final_layout(0)
        .and_then(|output| output.scroll_geometry)
        .expect("rounded flex geometry is retained");
    (unrounded, rounded)
}

#[test]
fn fri05_c04_flex_origin_main_cross_progressions_project_all_flows_before_and_after_rounding() {
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        for direction in [
            FlexDirection::Row,
            FlexDirection::RowReverse,
            FlexDirection::Column,
            FlexDirection::ColumnReverse,
        ] {
            for wrap in [FlexWrap::Wrap, FlexWrap::WrapReverse] {
                let main = if direction.is_reverse() {
                    (-40.0, 0.0)
                } else {
                    (0.0, 40.0)
                };
                let cross = if wrap == FlexWrap::WrapReverse {
                    (-40.0, 0.0)
                } else {
                    (0.0, 40.0)
                };
                let (inline, block) = if direction.is_row() {
                    (main, cross)
                } else {
                    (cross, main)
                };
                let context = format!("{flow_axes:?} {direction:?} {wrap:?}");
                let (unrounded, rounded) = fri05_c04_flex_origin_output(flow_axes, direction, wrap);
                fri05_c04_assert_flow_range(unrounded, flow_axes, inline, block, &context);
                fri05_c04_assert_flow_range(rounded, flow_axes, inline, block, &context);
            }
        }
    }
}

#[derive(Default)]
struct Fri05C04FlexAutoPassTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    child_output: Option<ComputeOutput>,
    child_outputs: HashMap<u32, ComputeOutput>,
    child_inputs: Vec<ComputeInput>,
    child_requests: Vec<(u32, ComputeInput)>,
    layouts: Vec<(u32, NodeOutput)>,
}

impl Traverse for Fri05C04FlexAutoPassTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl Compute for Fri05C04FlexAutoPassTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInput {
        LayoutInput::box_input(self.styles[&node].clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.push((node, layout));
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, ComputeOutput, Self::Scalar> {
        self.child_inputs.push(input);
        self.child_requests.push((node, input));
        if self.styles[&node].display == Display::Flex && self.child_count(node) != 0 {
            return compute_flex(
                self,
                node,
                input.with_settled_auto_scrollbars(
                    crate::scroll::SettledAutoScrollbarState::INITIAL,
                ),
            );
        }
        Ok(self.child_outputs.get(&node).copied().unwrap_or_else(|| {
            self.child_output
                .expect("FRI-05 flex auto child output is configured")
        }))
    }
}

fn fri05_c04_flex_auto_states(inputs: &[ComputeInput]) -> Vec<(bool, bool)> {
    assert!(
        inputs.iter().all(|input| {
            input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
        }),
        "each direct child request must begin node-local auto settlement at INITIAL: {inputs:#?}"
    );
    let mut states = inputs
        .iter()
        .map(|input| {
            let state = input.containing_auto_scrollbar_pass();
            (
                state.at(PhysicalAxis::Horizontal),
                state.at(PhysicalAxis::Vertical),
            )
        })
        .collect::<Vec<_>>();
    states.dedup();
    states
}

fn fri05_c04_flex_auto_absolute_case(
    flow_axes: FlowAxes,
    container_size: Size<f32>,
    child_size: Size<f32>,
    overflow: ComputedOverflow,
    gutter: ScrollbarGutter,
    scrollbar_width: f32,
) -> (ComputeOutput, Fri05C04FlexAutoPassTree) {
    let (left, right) = match flow_axes.physical_axis_progression(PhysicalAxis::Horizontal) {
        PhysicalProgression::Increasing => (LengthAuto::px(0.0), LengthAuto::AUTO),
        PhysicalProgression::Decreasing => (LengthAuto::AUTO, LengthAuto::px(0.0)),
    };
    let (top, bottom) = match flow_axes.physical_axis_progression(PhysicalAxis::Vertical) {
        PhysicalProgression::Increasing => (LengthAuto::px(0.0), LengthAuto::AUTO),
        PhysicalProgression::Decreasing => (LengthAuto::AUTO, LengthAuto::px(0.0)),
    };
    let mut tree = Fri05C04FlexAutoPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_gutter: gutter,
            scrollbar_width: ScrollbarWidth::try_new(scrollbar_width).unwrap(),
            size: container_size.map(PreferredSize::px),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            size: child_size.map(PreferredSize::px),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            inset: Edges::new(top, right, bottom, left),
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_sizes(child_size, child_size));

    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(container_size, flow_axes),
    )
    .expect("monotone flex auto layout succeeds");
    (output, tree)
}

type Fri05C04AutoStateBits = (bool, bool);
type Fri05C04AutoRequestStates = (Vec<Fri05C04AutoStateBits>, Vec<Fri05C04AutoStateBits>);

fn fri05_c04_flex_auto_request_states(
    requests: &[(u32, ComputeInput)],
    node: u32,
) -> Fri05C04AutoRequestStates {
    let matching = requests
        .iter()
        .filter_map(|(requested, input)| (*requested == node).then_some(*input))
        .collect::<Vec<_>>();
    let local = matching
        .iter()
        .map(|input| {
            let state = input.settled_auto_scrollbars();
            (
                state.at(PhysicalAxis::Horizontal),
                state.at(PhysicalAxis::Vertical),
            )
        })
        .collect::<Vec<_>>();
    let mut containing = Vec::new();
    for state in matching.iter().map(|input| {
        let state = input.containing_auto_scrollbar_pass();
        (
            state.at(PhysicalAxis::Horizontal),
            state.at(PhysicalAxis::Vertical),
        )
    }) {
        if !containing.contains(&state) {
            containing.push(state);
        }
    }
    (local, containing)
}

fn fri05_c04_flex_under_flex_case(
    inner_overflows: bool,
) -> (ComputeOutput, Fri05C04FlexAutoPassTree) {
    let mut tree = Fri05C04FlexAutoPassTree::default();
    tree.children
        .insert(0, if inner_overflows { vec![1] } else { vec![1, 3] });
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::splat_clone(PreferredSize::px(100.0)),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Flex,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::splat_clone(PreferredSize::px(40.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            flex_shrink: FlexShrink::try_new(0.0).unwrap(),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: if inner_overflows {
                Position::Absolute
            } else {
                Position::Relative
            },
            size: if inner_overflows {
                Size::new(PreferredSize::px(60.0), PreferredSize::px(20.0))
            } else {
                Size::splat_clone(PreferredSize::px(20.0))
            },
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            inset: Edges::new(
                LengthAuto::px(0.0),
                LengthAuto::AUTO,
                LengthAuto::AUTO,
                LengthAuto::px(0.0),
            ),
            ..NodeInput::default()
        },
    );
    tree.child_outputs.insert(
        2,
        ComputeOutput::from_sizes(
            if inner_overflows {
                Size::new(60.0, 20.0)
            } else {
                Size::splat(20.0)
            },
            if inner_overflows {
                Size::new(60.0, 20.0)
            } else {
                Size::splat(20.0)
            },
        ),
    );
    if !inner_overflows {
        tree.children.insert(3, vec![]);
        tree.styles.insert(
            3,
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                inset: Edges::new(
                    LengthAuto::px(0.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(0.0),
                ),
                ..NodeInput::default()
            },
        );
        tree.child_outputs.insert(
            3,
            ComputeOutput::from_sizes(Size::new(120.0, 80.0), Size::new(120.0, 80.0)),
        );
    }

    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            Size::splat(100.0),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("real flex-under-flex auto layout succeeds");
    (output, tree)
}

#[test]
fn fri05_c04_flex_auto_direct_nested_passes_separate_local_and_containing_state() {
    let (outer, tree) = fri05_c04_flex_under_flex_case(false);
    assert_eq!(
        outer.scroll_geometry.unwrap().scrollbar_size(),
        Size::new(0.0, 15.0)
    );
    let inner = tree
        .layouts
        .iter()
        .rev()
        .find_map(|(node, output)| (*node == 1).then_some(*output))
        .expect("outer retains the stable inner flex output");
    assert_eq!(inner.scroll_geometry.unwrap().scrollbar_size(), Size::ZERO);

    let (inner_local, inner_containing) =
        fri05_c04_flex_auto_request_states(&tree.child_requests, 1);
    assert!(inner_local.iter().all(|state| *state == (false, false)));
    assert_eq!(inner_containing, [(false, false), (true, false)]);
    let (grandchild_local, grandchild_containing) =
        fri05_c04_flex_auto_request_states(&tree.child_requests, 2);
    assert!(
        grandchild_local
            .iter()
            .all(|state| *state == (false, false))
    );
    assert!(
        grandchild_containing
            .iter()
            .all(|state| *state == (false, false))
    );
}

#[test]
fn fri05_c04_flex_auto_direct_inner_settlement_becomes_grandchild_containing_pass() {
    let (outer, tree) = fri05_c04_flex_under_flex_case(true);
    assert_eq!(outer.scroll_geometry.unwrap().scrollbar_size(), Size::ZERO);
    let inner = tree
        .layouts
        .iter()
        .rev()
        .find_map(|(node, output)| (*node == 1).then_some(*output))
        .expect("outer retains the independently settled inner flex output");
    assert_eq!(
        inner.scroll_geometry.unwrap().scrollbar_size(),
        Size::new(0.0, 15.0)
    );

    let (inner_local, inner_containing) =
        fri05_c04_flex_auto_request_states(&tree.child_requests, 1);
    assert!(inner_local.iter().all(|state| *state == (false, false)));
    assert!(
        inner_containing
            .iter()
            .all(|state| *state == (false, false))
    );
    let (grandchild_local, grandchild_containing) =
        fri05_c04_flex_auto_request_states(&tree.child_requests, 2);
    assert!(
        grandchild_local
            .iter()
            .all(|state| *state == (false, false))
    );
    assert_eq!(grandchild_containing, [(false, false), (true, false)]);
}

#[test]
fn fri05_c04_flex_auto_root_axis_cases_settle_monotonically_from_actual_pass_geometry() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let size = Size::splat(100.0);
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
        let (output, tree) = fri05_c04_flex_auto_absolute_case(
            flow_axes,
            size,
            child_size,
            computed_overflow(Overflow::Auto, Overflow::Auto),
            ScrollbarGutter::Auto,
            15.0,
        );
        let states = fri05_c04_flex_auto_states(&tree.child_inputs);
        assert_eq!(states, expected_states, "child size {child_size:?}");
        assert!(states.len() <= 3);
        assert_eq!(
            output.scroll_geometry.unwrap().scrollbar_size(),
            expected_scrollbars,
            "child size {child_size:?}"
        );
    }
}

#[test]
fn fri05_c04_flex_auto_alignment_subject_start_overflow_can_induce_other_axis() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let size = Size::splat(100.0);
    let child_size = Size::new(120.0, 100.0);
    let mut tree = Fri05C04FlexAutoPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: size.map(PreferredSize::px),
            align_items: Some(AlignItems::FlexStart),
            justify_content: Some(AlignContent::End),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            size: child_size.map(PreferredSize::px),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            flex_shrink: FlexShrink::try_new(0.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_sizes(child_size, child_size));

    let output = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
        .expect("alignment-subject auto layout succeeds");
    assert_eq!(
        fri05_c04_flex_auto_states(&tree.child_inputs),
        [(false, false), (true, false), (true, true)]
    );
    let range = output.scroll_geometry.unwrap().physical_range();
    assert!(
        range.x().minimum() < 0.0,
        "the actual start subject is observed"
    );
    assert_eq!(range.x().maximum(), 0.0);
}

#[test]
fn fri05_c04_flex_reservation_forced_stable_both_zero_and_auto_map_all_flows() {
    let size = Size::new(100.0, 80.0);
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        for (overflow, gutter, width, inline_start, inline_end) in [
            (
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Scroll),
                ScrollbarGutter::Auto,
                15.0,
                false,
                true,
            ),
            (
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
                ScrollbarGutter::Stable,
                15.0,
                false,
                true,
            ),
            (
                fri05_c04_flex_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
                ScrollbarGutter::StableBothEdges,
                15.0,
                true,
                true,
            ),
            (
                computed_overflow(Overflow::Scroll, Overflow::Scroll),
                ScrollbarGutter::StableBothEdges,
                0.0,
                false,
                false,
            ),
        ] {
            let (output, _) = fri05_c04_flex_auto_absolute_case(
                flow_axes,
                size,
                Size::new(20.0, 20.0),
                overflow,
                gutter,
                width,
            );
            let gutters = output.scroll_geometry.unwrap().gutters();
            for side in [
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
            ] {
                let expected = (side == flow_axes.inline_start() && inline_start)
                    || (side == flow_axes.inline_end() && inline_end);
                assert_eq!(
                    fri05_c04_flex_gutter_at(gutters, side).is_some(),
                    expected,
                    "{flow_axes:?} {side:?}"
                );
            }
        }

        let (auto_output, auto) = fri05_c04_flex_auto_absolute_case(
            flow_axes,
            size,
            Size::new(120.0, 20.0),
            computed_overflow(Overflow::Auto, Overflow::Auto),
            ScrollbarGutter::Auto,
            15.0,
        );
        assert_eq!(
            fri05_c04_flex_auto_states(&auto.child_inputs),
            [(false, false), (true, false)],
            "{flow_axes:?}"
        );
        let expected_auto_side = if flow_axes.inline_axis() == PhysicalAxis::Horizontal {
            flow_axes.block_end()
        } else {
            flow_axes.inline_end()
        };
        let auto_gutters = auto_output.scroll_geometry.unwrap().gutters();
        for side in [
            PhysicalSide::Top,
            PhysicalSide::Right,
            PhysicalSide::Bottom,
            PhysicalSide::Left,
        ] {
            assert_eq!(
                fri05_c04_flex_gutter_at(auto_gutters, side).is_some(),
                side == expected_auto_side,
                "auto {flow_axes:?} {side:?}"
            );
        }
    }
}

#[test]
fn fri05_c04_flex_tiny_induced_reservations_saturate_without_extra_evaluations() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let (output, tree) = fri05_c04_flex_auto_absolute_case(
        flow_axes,
        Size::splat(2.0),
        Size::new(3.0, 2.0),
        computed_overflow(Overflow::Auto, Overflow::Auto),
        ScrollbarGutter::Auto,
        15.0,
    );
    assert_eq!(
        fri05_c04_flex_auto_states(&tree.child_inputs),
        [(false, false), (true, false), (true, true)]
    );
    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.content_box().size(), Size::ZERO);
    assert_eq!(geometry.scrollport().size(), Size::ZERO);
    assert_eq!(geometry.scrollbar_size(), Size::splat(2.0));
    assert!(
        [
            geometry.border_box(),
            geometry.padding_box(),
            geometry.content_box(),
            geometry.scrollport(),
        ]
        .into_iter()
        .all(|rect| rect.size().width >= 0.0 && rect.size().height >= 0.0)
    );

    let mut measurement_tree = Fri05C04FlexAutoPassTree::default();
    measurement_tree.children.insert(0, vec![1]);
    measurement_tree.children.insert(1, vec![]);
    measurement_tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(PreferredSize::px(2.0), PreferredSize::px(2.0)),
            ..NodeInput::default()
        },
    );
    measurement_tree.styles.insert(1, NodeInput::default());
    measurement_tree.child_output = Some(ComputeOutput::from_outer_size(Size::splat(1.0)));
    let measurement = compute_flex(
        &mut measurement_tree,
        0,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::splat(Some(2.0)),
            Size::splat(Some(2.0)),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            Size::splat(Available::definite(2.0)),
        ),
    )
    .expect("fixed flex measurement remains geometry-free");
    assert!(measurement.scroll_geometry.is_none());
    assert!(measurement_tree.child_inputs.is_empty());
}

fn assert_fri08_c07_t03_optional_math_flex_results<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    let style = NodeInputOf::<S> {
        display: Display::Flex,
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
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [])
        .style(0, style);
    let output = crate::compute_flex(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(scalar(100.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        ),
    )
    .unwrap_or_else(|_| panic!("finite flex sizing must succeed"));

    assert_eq!(output.size, Size::new(scalar(8.0), scalar(11.0)));

    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let overflowing = LengthPercentageOf::from_coefficients(largest, S::ONE)
        .unwrap_or_else(|_| panic!("finite coefficients must be accepted"));
    let mut failing_tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::value(overflowing), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        );
    let error = crate::compute_flex(
        &mut failing_tree,
        0,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(largest), Some(scalar(100.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(AvailableOf::definite(largest), AvailableOf::MAX_CONTENT),
        ),
    )
    .expect_err("non-finite flex sizing must preserve its error");

    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
            value: S::INFINITY,
        })
    );
}

#[test]
fn fri08_c07_t03_optional_math_flex_results_preserve_both_scalar_lanes() {
    assert_fri08_c07_t03_optional_math_flex_results::<f32>();
    assert_fri08_c07_t03_optional_math_flex_results::<f64>();
}
