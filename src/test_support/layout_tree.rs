use std::collections::HashMap;

use crate::{
    AvailableOf, Compute, ComputeInputOf, ComputeOutputOf, DefaultScalar, DimensionOf, Display,
    InlineBoundaryInputOf, LayoutInputOf, LayoutScalar, LineBreakInput, LineBreakInputOf,
    NodeInput, NodeInputOf, NodeOutput, NodeOutputOf, RequestedAxis, Round, RunMode, Size,
    SizingMode, Traverse, compute_block, compute_flex, compute_grid,
};

pub type OracleTree = OracleTreeOf<DefaultScalar>;
pub type OracleMeasurement = OracleMeasurementOf<DefaultScalar>;

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

    pub const fn sizing_mode(mut self, sizing_mode: SizingMode) -> Self {
        self.sizing_mode = Some(sizing_mode);
        self
    }

    pub const fn axis(mut self, axis: RequestedAxis) -> Self {
        self.axis = Some(axis);
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
        matches_or_any(self.run_mode, input.run_mode)
            && matches_or_any(self.sizing_mode, input.sizing_mode)
            && matches_or_any(self.axis, input.axis)
            && matches_or_any(self.known, input.known)
            && matches_or_any(self.parent, input.parent)
            && matches_or_any(self.available, input.available)
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

    pub fn style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::box_input(style));
        self
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

    pub fn output(&self, node: u32) -> NodeOutputOf<S> {
        self.final_layout(node)
            .or_else(|| self.layout(node))
            .unwrap_or_default()
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

impl Compute for OracleTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        match self.layout_inputs.get(&node) {
            Some(input) => input
                .as_box()
                .unwrap_or_else(|| panic!("line break node has no box NodeInput")),
            None => panic!("oracle node {node} must define a layout input"),
        }
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<DefaultScalar> {
        self.layout_inputs
            .get(&node)
            .cloned()
            .unwrap_or_else(|| panic!("oracle node {node} must define a layout input"))
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<DefaultScalar>,
    ) -> ComputeOutputOf<DefaultScalar> {
        self.compute_inputs.entry(node).or_default().push(input);

        if let Some(output) = self.recorded_measurement(node, input) {
            return output;
        }

        match self.node_input(node).display.inner_display() {
            Display::Block => compute_block(self, node, input),
            Display::Flex => compute_flex(self, node, input),
            Display::Grid | Display::GridLanes => compute_grid(self, node, input),
            Display::None => {
                self.set_unrounded(node, NodeOutput::with_order(0));
                ComputeOutputOf::HIDDEN
            }
            Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }
}

impl Compute for OracleTreeOf<f64> {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<f64> {
        match self.layout_inputs.get(&node) {
            Some(input) => input
                .as_box()
                .unwrap_or_else(|| panic!("line break node has no box NodeInput")),
            None => panic!("oracle node {node} must define a layout input"),
        }
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<f64> {
        self.layout_inputs
            .get(&node)
            .cloned()
            .unwrap_or_else(|| panic!("oracle node {node} must define a layout input"))
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<f64>) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<f64>,
    ) -> ComputeOutputOf<f64> {
        self.compute_inputs.entry(node).or_default().push(input);

        if let Some(output) = self.recorded_measurement(node, input) {
            return output;
        }

        match self.node_input(node).display.inner_display() {
            Display::Block => return compute_block(self, node, input),
            Display::Flex => return compute_flex(self, node, input),
            Display::Grid | Display::GridLanes => {
                panic!("f64 oracle grid support is reserved for Task 7")
            }
            Display::None => {}
            Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }

        let style = self.node_input(node);
        if style.display == Display::None {
            self.set_unrounded(node, NodeOutputOf::with_order(0));
            return ComputeOutputOf::HIDDEN;
        }

        let width = input
            .known
            .width
            .or_else(|| resolve_dimension(style.size.width, input.parent.width));
        let height = input
            .known
            .height
            .or_else(|| resolve_dimension(style.size.height, input.parent.height));
        let size = Size::new(
            width.unwrap_or_else(|| input.available.width.into_option().unwrap_or(0.0)),
            height.unwrap_or_else(|| input.available.height.into_option().unwrap_or(0.0)),
        );

        ComputeOutputOf::from_sizes(size, size)
    }
}

impl<S: LayoutScalar> Round for OracleTreeOf<S> {
    fn unrounded(&self, node: Self::Node) -> NodeOutputOf<S> {
        self.layouts
            .get(&node)
            .copied()
            .unwrap_or_else(NodeOutputOf::new)
    }

    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
        self.final_layouts.insert(node, layout);
    }
}

fn resolve_dimension<S: LayoutScalar>(dimension: DimensionOf<S>, basis: Option<S>) -> Option<S> {
    dimension.resolve_optional(basis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayoutInput, LineBreakDisplay};

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
