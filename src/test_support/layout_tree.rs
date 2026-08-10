use std::collections::HashMap;

use crate::{
    AvailableOf, Compute, ComputeInputOf, ComputeOutputOf, DefaultScalar, Display,
    InlineBoundaryInputOf, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInputOf,
    LayoutInternalInvariant, LayoutOperation, LayoutScalar, LayoutTree, LeafMeasureInputOf,
    LineBreakInput, LineBreakInputOf, NodeInputOf, NodeOutputOf, RequestedAxis, Round, RunMode,
    Size, SizingMode, Traverse, compute_block, compute_flex, compute_grid,
};

pub type OracleTree = OracleTreeOf<DefaultScalar>;
pub type OracleMeasurement = OracleMeasurementOf<DefaultScalar>;

#[derive(Clone, Debug)]
pub struct PublicLayoutTreeOf<S: LayoutScalar = DefaultScalar> {
    children: HashMap<u32, Vec<u32>>,
    layout_inputs: HashMap<u32, LayoutInputOf<S>>,
    measurements: HashMap<u32, Size<S>>,
    non_box_input: NodeInputOf<S>,
}

impl<S: LayoutScalar> Default for PublicLayoutTreeOf<S> {
    fn default() -> Self {
        Self {
            children: HashMap::new(),
            layout_inputs: HashMap::new(),
            measurements: HashMap::new(),
            non_box_input: NodeInputOf::non_box(),
        }
    }
}

impl<S: LayoutScalar> PublicLayoutTreeOf<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    pub fn insert_children(&mut self, node: u32, children: impl IntoIterator<Item = u32>) {
        self.children.insert(node, children.into_iter().collect());
    }

    pub fn style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::box_input(style));
        self
    }

    pub fn input(mut self, node: u32, input: LayoutInputOf<S>) -> Self {
        self.layout_inputs.insert(node, input);
        self
    }

    pub fn insert_input(&mut self, node: u32, input: LayoutInputOf<S>) {
        self.layout_inputs.insert(node, input);
    }

    pub fn measure(mut self, node: u32, size: Size<S>) -> Self {
        self.measurements.insert(node, size);
        self
    }
}

impl<S: LayoutScalar> Traverse for PublicLayoutTreeOf<S> {
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

impl<S: LayoutScalar> LayoutTree for PublicLayoutTreeOf<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        let input = self
            .layout_inputs
            .get(&node)
            .unwrap_or_else(|| panic!("public layout node {node} must define a layout input"));
        input.as_box().unwrap_or(&self.non_box_input)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.layout_inputs
            .get(&node)
            .cloned()
            .unwrap_or_else(|| panic!("public layout node {node} must define a layout input"))
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.measurements.contains_key(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        _input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.measurements.get(&node).copied().map(Ok)
    }
}

#[derive(Clone, Debug, Default)]
pub struct OracleTreeOf<S: LayoutScalar = DefaultScalar> {
    children: HashMap<u32, Vec<u32>>,
    layout_inputs: HashMap<u32, LayoutInputOf<S>>,
    measurements: HashMap<u32, Vec<OracleMeasurementOf<S>>>,
    compute_inputs: HashMap<u32, Vec<ComputeInputOf<S>>>,
    layouts: HashMap<u32, NodeOutputOf<S>>,
    final_layouts: HashMap<u32, NodeOutputOf<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OracleMeasurementOf<S: LayoutScalar = DefaultScalar> {
    run_mode: Option<RunMode>,
    sizing_mode: Option<SizingMode>,
    axis: Option<RequestedAxis>,
    known: Option<Size<Option<S>>>,
    parent: Option<Size<Option<S>>>,
    available: Option<Size<AvailableOf<S>>>,
    output: ComputeOutputOf<S>,
}

impl<S: LayoutScalar> OracleMeasurementOf<S> {
    pub const fn new(output: ComputeOutputOf<S>) -> Self {
        Self {
            run_mode: None,
            sizing_mode: None,
            axis: None,
            known: None,
            parent: None,
            available: None,
            output,
        }
    }

    pub const fn run_mode(mut self, run_mode: RunMode) -> Self {
        self.run_mode = Some(run_mode);
        self
    }

    pub const fn known(mut self, known: Size<Option<S>>) -> Self {
        self.known = Some(known);
        self
    }

    pub const fn parent(mut self, parent: Size<Option<S>>) -> Self {
        self.parent = Some(parent);
        self
    }

    pub const fn available(mut self, available: Size<AvailableOf<S>>) -> Self {
        self.available = Some(available);
        self
    }

    fn matches(self, input: ComputeInputOf<S>) -> bool {
        matches_or_any(self.run_mode, input.run_mode())
            && matches_or_any(self.sizing_mode, input.sizing_mode())
            && matches_or_any(self.axis, input.requested_axis())
            && matches_or_any(self.known, input.known())
            && matches_or_any(self.parent, input.parent())
            && matches_or_any(self.available, input.available())
    }
}

fn matches_or_any<T: Copy + PartialEq>(expected: Option<T>, actual: T) -> bool {
    match expected {
        Some(expected) => expected == actual,
        None => true,
    }
}

impl<S: LayoutScalar> OracleTreeOf<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    pub fn insert_children(&mut self, node: u32, children: impl IntoIterator<Item = u32>) {
        self.children.insert(node, children.into_iter().collect());
    }

    pub fn style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::box_input(style));
        self
    }

    pub fn insert_style(&mut self, node: u32, style: NodeInputOf<S>) {
        self.layout_inputs
            .insert(node, LayoutInputOf::box_input(style));
    }

    pub fn line_break(mut self, node: u32, input: LineBreakInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::LineBreak(input));
        self
    }

    pub fn inline_boundary(mut self, node: u32, input: InlineBoundaryInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::InlineBoundary(input));
        self
    }

    pub fn measure(mut self, node: u32, output: ComputeOutputOf<S>) -> Self {
        self.measurements
            .entry(node)
            .or_default()
            .push(OracleMeasurementOf::new(output));
        self
    }

    pub fn insert_measure(&mut self, node: u32, output: ComputeOutputOf<S>) {
        self.measurements
            .entry(node)
            .or_default()
            .push(OracleMeasurementOf::new(output));
    }

    pub fn measure_when(mut self, node: u32, measurement: OracleMeasurementOf<S>) -> Self {
        self.measurements.entry(node).or_default().push(measurement);
        self
    }

    pub fn unrounded(mut self, node: u32, layout: NodeOutputOf<S>) -> Self {
        self.layouts.insert(node, layout);
        self
    }

    pub fn inputs(&self, node: u32) -> &[ComputeInputOf<S>] {
        self.compute_inputs
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.layouts.get(&node).copied()
    }

    pub fn final_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.final_layouts.get(&node).copied()
    }

    pub fn output(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.final_layout(node).or_else(|| self.layout(node))
    }

    fn recorded_measurement(
        &self,
        node: u32,
        input: ComputeInputOf<S>,
    ) -> Option<ComputeOutputOf<S>> {
        self.measurements.get(&node).map(|measurements| {
            for measurement in measurements {
                if measurement.matches(input) {
                    return measurement.output;
                }
            }

            panic!("no oracle measurement matched node {node} input {input:?}");
        })
    }
}

impl<S: LayoutScalar> Traverse for OracleTreeOf<S> {
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

impl<S: LayoutScalar> Compute for OracleTreeOf<S> {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        match self.layout_inputs.get(&node) {
            Some(input) => input
                .as_box()
                .unwrap_or_else(|| panic!("line break node has no box NodeInput")),
            None => panic!("oracle node {node} must define a layout input"),
        }
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.layout_inputs
            .get(&node)
            .cloned()
            .unwrap_or_else(|| panic!("oracle node {node} must define a layout input"))
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<S>,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        self.compute_inputs.entry(node).or_default().push(input);

        if let Some(output) = self.recorded_measurement(node, input) {
            return Ok(output);
        }

        match self.node_input(node).display.inner_display() {
            Display::Block => compute_block(self, node, input),
            Display::Flex => compute_flex(self, node, input),
            Display::Grid | Display::GridLanes => compute_grid(self, node, input),
            Display::None => {
                self.set_unrounded(
                    node,
                    NodeOutputOf::with_source_index(crate::SourceIndex::ZERO),
                );
                Ok(ComputeOutputOf::HIDDEN)
            }
            Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }
}

impl<S: LayoutScalar> Round for OracleTreeOf<S> {
    fn unrounded(&self, node: Self::Node) -> crate::LayoutResultOf<Self::Node, NodeOutputOf<S>, S> {
        self.layouts.get(&node).copied().ok_or_else(|| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingStagedUnroundedOutput,
                ),
            )
        })
    }

    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
        self.final_layouts.insert(node, layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ContainingLayoutContext, Direction, FlowAxes, LayoutErrorKind, LayoutErrorSite,
        LayoutInput, LayoutInternalInvariant, LayoutOperation, LayoutRootRequestOf,
        LineBreakDisplay, ParentFormattingContext, PreferredSizeOf, SourceIndex, WritingMode,
        compute_layout,
    };

    fn assert_fri06_c13_t01_public_tree_typed_contract<S: LayoutScalar>()
    where
        PublicLayoutTreeOf<S>:
            LayoutTree<MeasureError = core::convert::Infallible> + Traverse<Node = u32, Scalar = S>,
    {
        let style = NodeInputOf::<S> {
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(41.0)),
                PreferredSizeOf::px(S::from_f64(17.0)),
            ),
            ..NodeInputOf::default()
        };
        let line_break = LineBreakInputOf::<S>::new().hidden();
        let tree = PublicLayoutTreeOf::new()
            .children(1, [7, 3, 5])
            .style(1, style.clone())
            .input(7, LayoutInputOf::line_break(line_break));

        assert_eq!(
            <PublicLayoutTreeOf<S> as Traverse>::children(&tree, 1).collect::<Vec<_>>(),
            [7, 3, 5]
        );
        assert_eq!(tree.child_count(1), 3);
        assert_eq!(tree.child(1, 0), 7);
        assert_eq!(tree.child(1, 1), 3);
        assert_eq!(tree.child(1, 2), 5);
        assert_eq!(tree.node_input(1), &style);
        assert_eq!(tree.layout_input(1), LayoutInputOf::box_input(style));
        assert_eq!(tree.layout_input(7), LayoutInputOf::line_break(line_break));
        assert_eq!(tree.node_input(7), &NodeInputOf::non_box());
    }

    #[test]
    fn fri06_c13_t01_public_tree_preserves_typed_inputs_and_child_order_in_both_scalar_lanes() {
        assert_fri06_c13_t01_public_tree_typed_contract::<f32>();
        assert_fri06_c13_t01_public_tree_typed_contract::<f64>();
    }

    fn final_output<S: LayoutScalar>(
        batch: &crate::CompletedLayoutBatchOf<u32, S>,
        node: u32,
    ) -> NodeOutputOf<S> {
        batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .expect("ordinary public layout publishes the requested node")
            .output()
    }

    fn assert_fri06_c13_t01_public_layout_and_measurement<S: LayoutScalar>() {
        let fixed_size = Size::new(S::from_f64(31.0), S::from_f64(29.0));
        let ordinary = PublicLayoutTreeOf::new().style(
            1,
            NodeInputOf {
                size: fixed_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
        assert!(!ordinary.has_leaf_measurement(1));

        let request = || {
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(100.0))))
                .expect("finite viewport request")
        };
        let ordinary_batch =
            compute_layout(&ordinary, 1, request()).expect("ordinary fixed box lays out");
        assert_eq!(final_output(&ordinary_batch, 1).size, fixed_size);

        let measured_size = Size::new(S::from_f64(19.0), S::from_f64(23.0));
        let measured = PublicLayoutTreeOf::new()
            .style(2, NodeInputOf::default())
            .measure(2, measured_size);
        assert!(measured.has_leaf_measurement(2));

        let measured_request = || {
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
                .expect("max-content viewport request")
        };
        let first =
            compute_layout(&measured, 2, measured_request()).expect("measured leaf lays out");
        let second =
            compute_layout(&measured, 2, measured_request()).expect("measurement is deterministic");
        assert_eq!(final_output(&first, 2).size, measured_size);
        assert_eq!(final_output(&second, 2), final_output(&first, 2));
    }

    #[test]
    fn fri06_c13_t01_public_tree_preserves_measurement_absence_presence_and_public_layout() {
        assert_fri06_c13_t01_public_layout_and_measurement::<f32>();
        assert_fri06_c13_t01_public_layout_and_measurement::<f64>();
    }

    #[test]
    #[should_panic(expected = "public layout node 1 must define a layout input")]
    fn fri06_c13_t01_public_tree_missing_input_preserves_panic_boundary() {
        let tree = PublicLayoutTreeOf::<f32>::new();

        let _ = tree.node_input(1);
    }

    fn fri06_mr01_oracle_generic_input<S: LayoutScalar>() -> ComputeInputOf<S> {
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        )
    }

    fn assert_fri06_mr01_oracle_generic_records_and_prefers_measurement<S: LayoutScalar>()
    where
        OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
    {
        let style = NodeInputOf::<S> {
            display: Display::Grid,
            ..NodeInputOf::default()
        };
        let expected =
            ComputeOutputOf::from_outer_size(Size::new(S::from_f64(17.0), S::from_f64(23.0)));
        let input = fri06_mr01_oracle_generic_input();
        let mut tree = OracleTreeOf::new()
            .style(1, style.clone())
            .measure(1, expected);

        assert_eq!(tree.node_input(1), &style);
        assert_eq!(
            tree.layout_input(1),
            LayoutInputOf::box_input(style.clone())
        );
        assert_eq!(tree.compute_child(1, input).unwrap(), expected);
        assert_eq!(tree.inputs(1), &[input]);
        assert_eq!(tree.layout(1), None);
    }

    #[test]
    fn fri06_mr01_oracle_generic_records_inputs_and_prefers_measurements_in_both_scalar_lanes() {
        assert_fri06_mr01_oracle_generic_records_and_prefers_measurement::<f32>();
        assert_fri06_mr01_oracle_generic_records_and_prefers_measurement::<f64>();
    }

    fn assert_fri06_mr01_oracle_generic_hidden_and_dispatch<S: LayoutScalar>()
    where
        OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
    {
        let input = fri06_mr01_oracle_generic_input();
        let mut hidden = OracleTreeOf::new().style(
            1,
            NodeInputOf {
                display: Display::None,
                ..NodeInputOf::default()
            },
        );

        assert_eq!(
            hidden.compute_child(1, input).unwrap(),
            ComputeOutputOf::HIDDEN
        );
        assert_eq!(hidden.inputs(1), &[input]);
        assert_eq!(
            hidden.layout(1),
            Some(NodeOutputOf::with_source_index(SourceIndex::ZERO))
        );

        for (index, display) in [
            Display::Block,
            Display::Flex,
            Display::Grid,
            Display::GridLanes,
        ]
        .into_iter()
        .enumerate()
        {
            let node = u32::try_from(index + 2).unwrap();
            let expected_size = Size::new(S::from_f64(31.0), S::from_f64(29.0));
            let mut tree = OracleTreeOf::new().style(
                node,
                NodeInputOf {
                    display,
                    size: Size::new(
                        PreferredSizeOf::px(expected_size.width),
                        PreferredSizeOf::px(expected_size.height),
                    ),
                    ..NodeInputOf::default()
                },
            );

            let output = tree.compute_child(node, input).unwrap();

            assert_eq!(tree.inputs(node), &[input]);
            assert_eq!(output.size, expected_size, "unexpected {display:?} output");
        }
    }

    #[test]
    fn fri06_mr01_oracle_generic_stages_hidden_and_dispatches_algorithms_in_both_scalar_lanes() {
        assert_fri06_mr01_oracle_generic_hidden_and_dispatch::<f32>();
        assert_fri06_mr01_oracle_generic_hidden_and_dispatch::<f64>();
    }

    #[test]
    fn output_returns_none_without_a_staged_layout() {
        let tree = OracleTree::new();

        assert_eq!(tree.output(41), None);
    }

    #[test]
    fn rounding_missing_unrounded_output_returns_typed_error() {
        let mut tree = OracleTree::new();

        let error = crate::round_layout(&mut tree, 41)
            .expect_err("rounding without staged output must fail instead of synthesizing output");

        assert_eq!(error.site(), LayoutErrorSite::Node(41));
        assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
        assert_eq!(
            error.kind(),
            &LayoutErrorKind::InternalInvariant(
                LayoutInternalInvariant::MissingStagedUnroundedOutput,
            )
        );
        assert_eq!(tree.final_layout(41), None);
    }

    #[test]
    fn layout_input_returns_declared_line_break() {
        let input = LineBreakInput::new().hidden();
        let tree = OracleTree::new().line_break(1, input);

        assert_eq!(tree.layout_input(1), LayoutInput::LineBreak(input));
        assert_eq!(
            tree.layout_input(1).as_line_break().unwrap().display(),
            LineBreakDisplay::None
        );
    }

    #[test]
    #[should_panic(expected = "line break node has no box NodeInput")]
    fn node_input_panics_for_line_break_node() {
        let tree = OracleTree::new().line_break(1, LineBreakInput::new());

        let _ = tree.node_input(1);
    }

    #[test]
    #[should_panic(expected = "oracle node 1 must define a layout input")]
    fn node_input_panics_for_missing_node() {
        let tree = OracleTree::new();

        let _ = tree.node_input(1);
    }
}
