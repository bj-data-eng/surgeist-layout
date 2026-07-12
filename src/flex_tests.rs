use std::collections::HashMap;

use crate::*;

fn output_from_known_or(input: ComputeInput, fallback: Size) -> ComputeOutput {
    let size = Size::new(
        input.known.width.unwrap_or(fallback.width),
        input.known.height.unwrap_or(fallback.height),
    );
    ComputeOutput::from_sizes(size, size)
}

fn fake_leaf_error(
    node: u32,
    error: LayoutError<(), core::convert::Infallible>,
) -> LayoutError<u32> {
    LayoutError::new(
        LayoutErrorSite::Node(node),
        error.operation(),
        error.kind().clone(),
    )
}

#[test]
fn flex_direction_reports_main_cross_and_reverse_axes() {
    assert!(FlexDirection::Row.is_row());
    assert!(FlexDirection::RowReverse.is_row());
    assert!(FlexDirection::Column.is_column());
    assert!(FlexDirection::ColumnReverse.is_column());
    assert!(!FlexDirection::Row.is_reverse());
    assert!(FlexDirection::RowReverse.is_reverse());
    assert_eq!(FlexDirection::Row.main_axis(), Axis::Horizontal);
    assert_eq!(FlexDirection::Row.cross_axis(), Axis::Vertical);
    assert_eq!(FlexDirection::Column.main_axis(), Axis::Vertical);
    assert_eq!(FlexDirection::Column.cross_axis(), Axis::Horizontal);
}

#[test]
fn flex_row_lays_out_fixed_children_with_gap_and_container_insets() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                self.outputs[&node]
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(30.0), Dimension::px(30.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 20.0), Size::new(40.0, 20.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(30.0, 30.0), Size::new(30.0, 30.0)),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(200.0, 42.0));
    assert_eq!(output.content_size, Size::new(80.0, 30.0));

    assert_eq!(tree.layouts[&2].location, Point::new(6.0, 6.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(56.0, 6.0));
    assert_eq!(tree.layouts[&3].size, Size::new(30.0, 30.0));

    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(40.0), Some(20.0)));
    assert_eq!(tree.inputs[&3][0].known, Size::new(Some(30.0), Some(30.0)));
}

#[test]
fn f64_flex_layout_preserves_fractional_growth() {
    let container_width = 16_777_217.75;
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Flex,
                size: Size::new(DimensionOf::px(container_width), DimensionOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
                size: Size::new(DimensionOf::px(20.125), DimensionOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::Block,
                flex_grow: FlexGrowOf::try_new(3.0).unwrap(),
                size: Size::new(DimensionOf::px(20.125), DimensionOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        );

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInputOf::<f64> {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(container_width), None),
            available: Size::new(
                AvailableOf::definite(container_width),
                AvailableOf::MAX_CONTENT,
            ),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(container_width, 10.0));
    assert_eq!(
        tree.output(1)
            .expect("flex layout must stage output for the first child")
            .size
            .width,
        4_194_314.5
    );
    assert_eq!(
        tree.output(2)
            .expect("flex layout must stage output for the second child")
            .size
            .width,
        12_582_903.25
    );
    assert_eq!(
        tree.output(2)
            .expect("flex layout must stage output for the second child")
            .location
            .x,
        4_194_314.5
    );
}

#[test]
fn flex_margin_resolution_handles_invalid_affine_numeric_result_without_panicking() {
    let invalid_margin =
        LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX).expect("finite coefficients");
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                margin: Edges {
                    left: LengthAuto::value(invalid_margin),
                    ..Edges::all(LengthAuto::ZERO)
                },
                size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
                ..NodeInput::default()
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 20.0)));

    let error = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(120.0), Some(40.0)),
            available: Size::new(Available::definite(120.0), Available::definite(40.0)),
        },
    )
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(2));
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
    ));
}

#[test]
fn flex_content_size_includes_visible_child_overflow_content() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 10.0));
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn flex_final_content_size_uses_rerun_output() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                let size = if input.run_mode == RunMode::PerformLayout
                    && input.known.width == Some(80.0)
                {
                    Size::new(80.0, 40.0)
                } else {
                    Size::new(20.0, 10.0)
                };
                ComputeOutput::from_sizes(size, size)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(Dimension::px(80.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(80.0), None),
            available: Size::new(Available::definite(80.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert!(tree.inputs[&1].iter().any(|input| {
        input.run_mode == RunMode::ComputeSize && input.known.width == Some(80.0)
    }));
    assert!(tree.inputs[&1].iter().any(|input| {
        input.run_mode == RunMode::PerformLayout && input.known.width == Some(80.0)
    }));
    assert_eq!(output.content_size.height, 40.0);
}

#[test]
fn flex_relative_child_inset_offsets_final_layout_location() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(7.0, 3.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_relative_child_trailing_inset_offsets_negative() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            inset: Edges {
                right: LengthAuto::px(5.0),
                bottom: LengthAuto::px(2.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(-5.0, -2.0));
}

#[test]
fn flex_compute_size_short_circuits_when_container_size_is_definite() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("definite compute-size should not measure children")
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn flex_compute_size_measures_children_without_perform_layout() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {
            panic!("compute-size must not write child layouts")
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_outer_size(Size::new(20.0, 10.0))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(20.0, 10.0));
    assert_eq!(tree.inputs[&2][0].run_mode, RunMode::ComputeSize);
}

#[test]
fn flex_row_auto_main_item_uses_content_sizing_for_base_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_outer_size(Size::new(0.0, input.known.height.unwrap_or(10.0)))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(50.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(50.0), Some(10.0)),
            available: Size::new(Available::definite(50.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    let base_input = tree.inputs[&2][0];
    assert_eq!(base_input.sizing_mode, SizingMode::ContentSize);
    assert_eq!(base_input.known.width, None);
    assert_eq!(base_input.known.height, Some(10.0));
    assert_eq!(base_input.available.width, Available::MAX_CONTENT);
    assert_eq!(base_input.available.height, Available::definite(10.0));
}

#[test]
fn flex_row_hidden_overflow_item_has_zero_automatic_minimum() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(40.0),
                    input.known.height.unwrap_or(50.0),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(20.0), Some(50.0)),
            available: Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size, Size::new(0.0, 50.0));
    assert_eq!(tree.layouts[&3].size, Size::new(40.0, 50.0));
}

#[test]
fn flex_column_hidden_overflow_aspect_item_has_zero_automatic_minimum() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(40.0),
                    input.known.height.unwrap_or(50.0),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            flex_direction: FlexDirection::Column,
            size: Size::new(Dimension::px(20.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Hidden),
            flex_basis: Dimension::px(0.0),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            aspect_ratio: AspectRatio::new(1.0),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(20.0), Some(50.0)),
            available: Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 50.0));
}

#[test]
fn flex_column_cross_axis_hidden_overflow_aspect_item_has_zero_automatic_minimum() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(40.0),
                    input.known.height.unwrap_or(50.0),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            flex_direction: FlexDirection::Column,
            size: Size::new(Dimension::px(20.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Clip),
            flex_basis: Dimension::px(0.0),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            aspect_ratio: AspectRatio::new(1.0),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(20.0), Some(50.0)),
            available: Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 50.0));
}

#[test]
fn flex_compute_size_uses_definite_min_max_without_measuring_children() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("definite min/max compute-size should not measure children")
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            min_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            max_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
}

#[test]
fn flex_display_none_child_gets_zero_layout_and_hidden_input() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                if input.run_mode == RunMode::PerformLayout {
                    ComputeOutput::from_sizes(
                        Size::new(input.known.width.unwrap(), input.known.height.unwrap()),
                        Size::ZERO,
                    )
                } else {
                    ComputeOutput::HIDDEN
                }
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::None,
            size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&3], NodeOutput::with_order(1));
    assert_eq!(tree.inputs[&3], vec![ComputeInput::HIDDEN]);
}

#[test]
fn flex_container_reserves_scrollbar_gutter_from_inner_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(0.0), Dimension::px(10.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::new(90.0, 40.0));
    assert_eq!(tree.layouts[&2].size, Size::new(90.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::ZERO);
}

#[test]
fn flex_scrollbar_gutter_uses_left_inset_for_rtl_containers() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_child_layout_records_scrollbar_size_for_scroll_overflow() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(7.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].scrollbar_size, Size::new(7.0, 7.0));
}

#[test]
fn flex_absolute_child_uses_insets_without_affecting_flow() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                if node == 3 {
                    return Ok(ComputeOutput::from_sizes(
                        Size::new(input.known.width.unwrap(), input.known.height.unwrap()),
                        Size::new(80.0, 32.0),
                    ));
                }
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(25.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(12.0)),
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::new(87.0, 41.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(25.0, 10.0));
    assert_eq!(tree.layouts[&3].location, Point::new(7.0, 9.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 12.0));
    assert_eq!(tree.inputs[&3][0].known, Size::new(Some(20.0), Some(12.0)));
}

#[test]
fn flex_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(400.0), Dimension::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(
        tree.inputs[&2][0].known,
        Size::new(Some(360.0), Some(120.0))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(20.0, 15.0));
    assert_eq!(tree.layouts[&2].size, Size::new(360.0, 120.0));
}

#[test]
fn flex_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(400.0), Dimension::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::percent(0.1),
                right: LengthAuto::percent(0.1),
                top: LengthAuto::percent(0.05),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(Dimension::percent(0.4), Dimension::AUTO),
            aspect_ratio: AspectRatio::new(3.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(
        tree.inputs[&2][0].known,
        Size::new(Some(160.0), Some(160.0 / 3.0))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(200.0, 15.0));
}

#[test]
fn flex_absolute_child_max_height_shrinks_flex_grandchild() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(
            &mut self,
            node: u32,
            input: ComputeInput,
        ) -> LayoutResultOf<u32, ComputeOutput, Scalar> {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return compute_leaf(input, &node_input, |measure_input| {
                    let known = measure_input.known_content_size();
                    Ok::<_, core::convert::Infallible>(Size::new(
                        known.width.unwrap_or(0.0),
                        known.height.unwrap_or(0.0),
                    ))
                })
                .map_err(|error| fake_leaf_error(node, error));
            }

            match node_input.display.inner_display() {
                Display::Flex => compute_flex(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::None => Ok(ComputeOutput::HIDDEN),
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
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

    impl Compute for RecursiveTree {
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
            self.compute_node(node, input)
        }
    }

    let mut tree = RecursiveTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![3]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
            flex_direction: FlexDirection::Column,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            flex_direction: FlexDirection::Column,
            inset: Edges {
                bottom: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            max_size: Size::new(Dimension::AUTO, Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            flex_basis: Dimension::px(150.0),
            flex_shrink: FlexShrinkOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(200.0)),
            available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 80.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 100.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(100.0, 100.0));
}

#[test]
fn flex_absolute_child_cross_alignment_honors_wrap_reverse() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            Ok(self.outputs_for(node, input))
        }
    }

    impl FlexTree {
        fn new(
            align_self: AlignItems,
            flex_direction: FlexDirection,
            layout_direction: Direction,
        ) -> Self {
            let mut tree = Self::default();
            tree.children.insert(1, vec![2]);
            tree.children.insert(2, vec![]);
            tree.styles.insert(
                1,
                NodeInput {
                    direction: layout_direction,
                    size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                    flex_direction,
                    flex_wrap: FlexWrap::WrapReverse,
                    ..NodeInput::default()
                },
            );
            tree.styles.insert(
                2,
                NodeInput {
                    direction: layout_direction,
                    position: Position::Absolute,
                    align_self: Some(align_self),
                    size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
                    ..NodeInput::default()
                },
            );
            tree
        }

        fn outputs_for(&self, _node: u32, input: ComputeInput) -> ComputeOutput {
            output_from_known_or(input, Size::ZERO)
        }

        fn layout_child(&mut self) -> NodeOutput {
            compute_flex(
                self,
                1,
                ComputeInput {
                    run_mode: RunMode::PerformLayout,
                    sizing_mode: SizingMode::InherentSize,
                    axis: RequestedAxis::Both,
                    known: Size::NONE,
                    parent: Size::new(Some(100.0), Some(100.0)),
                    available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
                },
            )
            .unwrap();
            self.layouts[&2]
        }
    }

    let default_layout =
        FlexTree::new(AlignItems::Stretch, FlexDirection::Row, Direction::Ltr).layout_child();
    assert_eq!(default_layout.location, Point::new(0.0, 80.0));
    assert_eq!(default_layout.size, Size::new(20.0, 20.0));

    let flex_end_layout =
        FlexTree::new(AlignItems::FlexEnd, FlexDirection::Row, Direction::Ltr).layout_child();
    assert_eq!(flex_end_layout.location, Point::new(0.0, 0.0));
    assert_eq!(flex_end_layout.size, Size::new(20.0, 20.0));

    let column_rtl_layout =
        FlexTree::new(AlignItems::Stretch, FlexDirection::Column, Direction::Rtl).layout_child();
    assert_eq!(column_rtl_layout.location, Point::new(0.0, 0.0));
    assert_eq!(column_rtl_layout.size, Size::new(20.0, 20.0));

    let column_rtl_flex_end_layout =
        FlexTree::new(AlignItems::FlexEnd, FlexDirection::Column, Direction::Rtl).layout_child();
    assert_eq!(column_rtl_flex_end_layout.location, Point::new(80.0, 0.0));
    assert_eq!(column_rtl_flex_end_layout.size, Size::new(20.0, 20.0));
}

#[test]
fn flex_absolute_child_cross_start_margin_uses_physical_edge_in_rtl_column() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            justify_content: Some(AlignContent::FlexEnd),
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            direction: Direction::Rtl,
            position: Position::Absolute,
            size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
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
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(100.0)),
            available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(90.0, 80.0));
    assert_eq!(tree.layouts[&2].size, Size::new(10.0, 10.0));
}

#[test]
fn flex_absolute_child_uses_min_size_when_min_exceeds_max_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_sizes(
                    Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ),
                    Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ),
                )
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                right: LengthAuto::px(10.0),
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            min_size: Size::new(Dimension::px(50.0), Dimension::px(60.0)),
            max_size: Size::new(Dimension::px(40.0), Dimension::px(30.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(100.0)),
            available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 30.0));
    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 60.0));
}

#[test]
fn flex_absolute_child_size_cannot_shrink_below_padding_and_border() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_sizes(
                    Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ),
                    Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ),
                )
            })
        }
    }

    fn tree_with_child(child_style: NodeInput) -> FlexTree {
        let mut tree = FlexTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![]);
        tree.styles.insert(1, NodeInput::default());
        tree.styles.insert(2, child_style);
        tree
    }

    fn run(tree: &mut FlexTree) {
        compute_flex(
            tree,
            1,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::NONE,
                available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            },
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
        size: Size::new(Dimension::px(12.0), Dimension::px(12.0)),
        padding,
        border,
        ..NodeInput::default()
    });
    run(&mut authored_size);
    assert_eq!(
        authored_size.inputs[&2][0].known,
        Size::new(Some(22.0), Some(14.0))
    );
    assert_eq!(authored_size.layouts[&2].size, Size::new(22.0, 14.0));

    let mut max_size = tree_with_child(NodeInput {
        position: Position::Absolute,
        max_size: Size::new(Dimension::px(12.0), Dimension::px(12.0)),
        padding,
        border,
        ..NodeInput::default()
    });
    run(&mut max_size);
    assert_eq!(max_size.layouts[&2].size, Size::new(22.0, 14.0));
}

#[test]
fn flex_absolute_child_layout_records_scrollbar_size_for_scroll_overflow() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(8.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].scrollbar_size, Size::new(8.0, 8.0));
}

#[test]
fn flex_absolute_child_can_resolve_from_trailing_insets() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                right: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(72.0, 34.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_absolute_child_expands_auto_margins() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
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
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
}

#[test]
fn flex_absolute_child_without_insets_uses_flex_alignment() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            justify_content: Some(AlignContent::Center),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 15.0));
}

#[test]
fn flex_row_distributes_positive_free_space_with_flex_grow() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(200.0, 20.0));
    assert_eq!(output.content_size, Size::new(200.0, 20.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(105.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(105.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(95.0, 20.0));

    assert_eq!(
        tree.inputs[&2].last().unwrap().known,
        Size::new(Some(105.0), Some(20.0))
    );
    assert_eq!(
        tree.inputs[&3].last().unwrap().known,
        Size::new(Some(95.0), Some(20.0))
    );
}

#[test]
fn flex_row_with_grow_sum_below_one_uses_that_fraction_of_free_space() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            flex_grow: FlexGrowOf::try_new(0.5).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert_eq!(tree.layouts[&2].location, Point::ZERO);
    assert_eq!(tree.layouts[&2].size, Size::new(60.0, 10.0));
}

#[test]
fn flex_row_distributes_negative_free_space_with_flex_shrink() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(80.0), Dimension::px(20.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(70.0), Dimension::px(20.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert!((tree.layouts[&2].size.width - 53.333).abs() < 0.01);
    assert!((tree.layouts[&3].location.x - 53.333).abs() < 0.01);
    assert!((tree.layouts[&3].size.width - 46.667).abs() < 0.01);
    assert_eq!(tree.layouts[&2].size.height, 20.0);
    assert_eq!(tree.layouts[&3].size.height, 20.0);
}

#[test]
fn flex_row_relayouts_content_box_percentage_item_at_shrunk_target() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(730.0), Dimension::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(Dimension::percent(1.0), Dimension::px(100.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
            padding: Edges::all(Length::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(730.0), Some(300.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size.width, 730.0);
    assert_eq!(
        tree.inputs[&2]
            .last()
            .expect("child should be laid out")
            .known
            .width,
        Some(730.0)
    );
}

#[test]
fn flex_row_visible_item_does_not_shrink_below_automatic_min_content_width() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                if node == 2
                    && input.run_mode == RunMode::ComputeSize
                    && input.available.width == Available::MIN_CONTENT
                {
                    return Ok(ComputeOutput::from_outer_size(Size::new(90.0, 20.0)));
                }

                let fallback = if node == 2 {
                    Size::new(160.0, 20.0)
                } else {
                    Size::new(40.0, 20.0)
                };
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(fallback.width),
                    input.known.height.unwrap_or(fallback.height),
                ))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::px(20.0)),
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert!(
        tree.inputs[&2].iter().any(|input| {
            input.run_mode == RunMode::ComputeSize
                && input.available.width == Available::MIN_CONTENT
        }),
        "visible flex item should be measured with min-content for its automatic minimum"
    );
    assert_eq!(tree.layouts[&2].size.width, 90.0);
    assert_eq!(tree.layouts[&3].location.x, 90.0);
    assert_eq!(tree.layouts[&3].size.width, 40.0);
}

#[test]
fn flex_row_with_shrink_sum_below_one_uses_that_fraction_of_negative_free_space() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(80.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(10.0)),
            min_size: Size::new(Dimension::ZERO, Dimension::ZERO),
            flex_shrink: FlexShrinkOf::try_new(0.5).unwrap(),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(80.0, 20.0));
    assert_eq!(tree.layouts[&2].location, Point::ZERO);
    assert_eq!(tree.layouts[&2].size, Size::new(90.0, 10.0));
}

#[test]
fn flex_row_wraps_items_into_multiple_lines() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            flex_wrap: FlexWrap::Wrap,
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3, 4] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(60.0), Dimension::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 38.0));
    assert_eq!(output.content_size, Size::new(60.0, 38.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 14.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 28.0));
}

#[test]
fn flex_row_auto_width_wraps_against_definite_available_width() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            flex_wrap: FlexWrap::Wrap,
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3, 4] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(60.0), Dimension::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 38.0));
    assert_eq!(output.content_size, Size::new(60.0, 38.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 14.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 28.0));
}

#[test]
fn flex_row_justifies_items_on_the_main_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(25.0), Dimension::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert_eq!(tree.layouts[&2].location, Point::new(25.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(50.0, 0.0));
}

#[test]
fn flex_row_aligns_items_on_the_cross_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 15.0));
}

#[test]
fn flex_row_reports_first_child_baseline() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                let size = Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                );
                ComputeOutput::from_sizes_and_first_baselines(
                    size,
                    Size::ZERO,
                    Point::new(None, Some(7.0)),
                )
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.first_baselines.y, Some(7.0));
    assert_eq!(output.last_baselines.y, Some(7.0));
}

#[test]
fn flex_row_aligns_baseline_items_by_child_baselines() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                let baseline = match node {
                    2 => 15.0,
                    3 => 5.0,
                    _ => 0.0,
                };
                let size = Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                );
                ComputeOutput::from_sizes_and_first_baselines(
                    size,
                    Size::ZERO,
                    Point::new(None, Some(baseline)),
                )
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            align_items: Some(AlignItems::Baseline),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert_eq!(tree.layouts[&2].location.y, 0.0);
    assert_eq!(tree.layouts[&3].location.y, 10.0);
    assert_eq!(output.first_baselines.y, Some(15.0));
}

#[test]
fn flex_row_stretches_auto_cross_size_items() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                let size = Size::new(
                    input.known.width.unwrap_or(20.0),
                    input.known.height.unwrap_or(10.0),
                );
                ComputeOutput::from_sizes(size, size)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 40.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known,
        Size::new(Some(20.0), Some(40.0))
    );
}

#[test]
fn flex_row_stretch_transfers_cross_size_through_aspect_ratio() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::new(20.0, 10.0))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(200.0), Dimension::px(50.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            aspect_ratio: AspectRatio::new(2.0),
            flex_grow: FlexGrowOf::try_new(0.0).unwrap(),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 50.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known,
        Size::new(Some(100.0), Some(50.0))
    );
}

#[test]
fn flex_row_stretched_aspect_ratio_item_does_not_shrink_below_transferred_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::new(0.0, 0.0)))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            min_size: Size::new(Dimension::AUTO, Dimension::px(40.0)),
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(200.0, 100.0));
}

#[test]
fn flex_row_aspect_ratio_auto_min_respects_authored_width_cap() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::new(20.0, 10.0)))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(300.0), Dimension::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(50.0), Dimension::px(100.0)),
            aspect_ratio: AspectRatio::new(2.0),
            flex_grow: FlexGrowOf::try_new(0.0).unwrap(),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::definite(100.0)),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 100.0));
}

#[test]
fn flex_row_aligns_wrapped_lines_with_align_content() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Center),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 60.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 18.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 32.0));
}

#[test]
fn flex_column_wrap_with_one_line_honors_align_content_end() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3, 4, 5, 6]);
    for node in 2..=6 {
        tree.children.insert(node, vec![]);
    }
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::End),
            ..NodeInput::default()
        },
    );
    for child in 2..=6 {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(50.0), Dimension::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 100.0));
    for child in 2..=6 {
        assert_eq!(tree.layouts[&child].location.x, 50.0);
    }
}

#[test]
fn flex_row_stretches_wrapped_lines_with_align_content_stretch() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Stretch),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 60.0));
    assert_eq!(output.content_size, Size::new(80.0, 60.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 32.0));
}

#[test]
fn flex_row_stretched_wrapped_line_stretches_auto_cross_size_item() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                let size = Size::new(
                    input.known.width.unwrap_or(80.0),
                    input.known.height.unwrap_or(10.0),
                );
                ComputeOutput::from_sizes(size, size)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::Wrap,
            align_content: Some(AlignContent::Stretch),
            align_items: Some(AlignItems::Stretch),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 28.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 28.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 32.0));
    assert_eq!(
        tree.inputs[&3].last().unwrap().known,
        Size::new(Some(80.0), Some(28.0))
    );
}

#[test]
fn flex_row_wrap_reverse_places_lines_from_the_reversed_cross_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_content: Some(AlignContent::FlexStart),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 50.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 36.0));
}

#[test]
fn flex_row_wrap_reverse_flips_flex_start_item_alignment() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 50.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_row_wrap_reverse_respects_reversed_align_content() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_content: Some(AlignContent::FlexEnd),
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(Dimension::px(80.0), Dimension::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 14.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 0.0));
}

#[test]
fn flex_row_growth_respects_max_main_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            max_size: Size::new(Dimension::px(60.0), Dimension::AUTO),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size, Size::new(60.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(60.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(140.0, 20.0));
}

#[test]
fn flex_row_distributes_positive_space_to_main_axis_auto_margins() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            margin: Edges::new(
                LengthAuto::ZERO,
                LengthAuto::ZERO,
                LengthAuto::ZERO,
                LengthAuto::AUTO,
            ),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&2].margin.left, 80.0);
}

#[test]
fn flex_row_distributes_cross_axis_auto_margins() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            margin: Edges::new(
                LengthAuto::AUTO,
                LengthAuto::ZERO,
                LengthAuto::AUTO,
                LengthAuto::ZERO,
            ),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 15.0));
    assert_eq!(tree.layouts[&2].margin.top, 15.0);
    assert_eq!(tree.layouts[&2].margin.bottom, 15.0);
}

#[test]
fn flex_row_reverse_places_items_from_the_reversed_main_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            flex_direction: FlexDirection::RowReverse,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(50.0, 0.0));
}

#[test]
fn flex_row_rtl_places_items_from_the_right_edge() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(50.0, 0.0));
}

#[test]
fn flex_row_rtl_relative_insets_follow_rtl_main_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            inset: Edges {
                left: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            inset: Edges {
                right: LengthAuto::px(7.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(85.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(53.0, 0.0));
}

#[test]
fn flex_column_rtl_aligns_cross_start_to_the_right_edge() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::FlexStart),
            size: Size::new(Dimension::px(100.0), Dimension::px(80.0)),
            padding: Edges {
                left: Length::px(4.0),
                right: Length::px(6.0),
                top: Length::ZERO,
                bottom: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(74.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn flex_column_rtl_cross_axis_auto_margin_uses_rtl_edges() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            direction: Direction::Rtl,
            flex_direction: FlexDirection::Column,
            size: Size::new(Dimension::px(100.0), Dimension::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            margin: Edges {
                left: LengthAuto::px(3.0),
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
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].margin.right, 77.0);
    assert_eq!(tree.layouts[&2].margin.left, 3.0);
    assert_eq!(tree.layouts[&2].location, Point::new(3.0, 0.0));
}

#[test]
fn flex_column_reverse_places_items_from_the_reversed_main_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(output_from_known_or(input, Size::ZERO))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(100.0)),
            flex_direction: FlexDirection::ColumnReverse,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 80.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 50.0));
}

#[test]
fn flex_row_uses_flex_basis_as_the_main_base_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                let size = Size::new(
                    input.known.width.unwrap_or(10.0),
                    input.known.height.unwrap_or(10.0),
                );
                ComputeOutput::from_sizes(size, size)
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::AUTO, Dimension::px(10.0)),
            flex_basis: Dimension::px(30.0),
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 10.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known,
        Size::new(Some(30.0), Some(10.0))
    );
}

#[test]
fn flex_row_flex_basis_zero_preserves_padding_border_without_authored_content_width() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::new(34.0, 10.0))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(12.0), Dimension::px(12.0)),
            flex_basis: Dimension::px(0.0),
            padding: Edges {
                left: Length::px(8.0),
                top: Length::px(2.0),
                right: Length::px(4.0),
                bottom: Length::px(6.0),
            },
            border: Edges {
                left: Length::px(7.0),
                top: Length::px(1.0),
                right: Length::px(3.0),
                bottom: Length::px(5.0),
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(12.0), Dimension::px(12.0)),
            flex_basis: Dimension::px(0.0),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size, Size::new(22.0, 14.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(22.0, 14.0));
    assert_eq!(tree.layouts[&3].location, Point::new(22.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(0.0, 12.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known,
        Size::new(Some(22.0), Some(14.0))
    );
}

#[test]
fn flex_row_flex_basis_padding_floor_preserves_leaf_content_intrinsic_size() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
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

    impl Compute for FlexTree {
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
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(ComputeOutput::from_sizes(
                Size::new(0.0, 10.0),
                Size::new(120.0, 10.0),
            ))
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            flex_basis: Dimension::px(0.0),
            padding: Edges {
                left: Length::px(10.0),
                right: Length::px(10.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    )
    .unwrap();

    assert_eq!(output.size.width, 120.0);
    assert_eq!(output.content_size.width, 120.0);
    assert_eq!(tree.layouts[&2].content_size.width, 120.0);
}

use crate::{Dimension, LengthPercentageOf, NodeInput};

#[test]
fn flex_percent_dependent_affine_size_requests_definite_cross_rerun() {
    let height = LengthPercentageOf::from_coefficients(10.0, 0.50).expect("finite coefficients");
    let mut child = NodeInput::default();
    child.size.height = Dimension::value(height);

    assert!(child.size.height.depends_on_basis());
}
