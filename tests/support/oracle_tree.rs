use std::collections::HashMap;

use surgeist_layout::{
    Available, Compute, ComputeInput, ComputeOutput, Display, NodeInput, NodeOutput, RequestedAxis,
    Round, RunMode, Size, SizingMode, Traverse, compute_block, compute_flex, compute_grid,
};

static DEFAULT_NODE_INPUT: NodeInput = NodeInput::DEFAULT;

#[derive(Clone, Debug, Default)]
pub struct OracleTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    measurements: HashMap<u32, Vec<OracleMeasurement>>,
    inputs: HashMap<u32, Vec<ComputeInput>>,
    layouts: HashMap<u32, NodeOutput>,
    final_layouts: HashMap<u32, NodeOutput>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OracleMeasurement {
    run_mode: Option<RunMode>,
    sizing_mode: Option<SizingMode>,
    axis: Option<RequestedAxis>,
    known: Option<Size<Option<f32>>>,
    parent: Option<Size<Option<f32>>>,
    available: Option<Size<Available>>,
    output: ComputeOutput,
}

impl OracleMeasurement {
    pub const fn new(output: ComputeOutput) -> Self {
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

    pub const fn known(mut self, known: Size<Option<f32>>) -> Self {
        self.known = Some(known);
        self
    }

    pub const fn parent(mut self, parent: Size<Option<f32>>) -> Self {
        self.parent = Some(parent);
        self
    }

    pub const fn available(mut self, available: Size<Available>) -> Self {
        self.available = Some(available);
        self
    }

    fn matches(self, input: ComputeInput) -> bool {
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

impl OracleTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    pub fn style(mut self, node: u32, style: NodeInput) -> Self {
        self.styles.insert(node, style);
        self
    }

    pub fn measure(mut self, node: u32, output: ComputeOutput) -> Self {
        self.measurements
            .entry(node)
            .or_default()
            .push(OracleMeasurement::new(output));
        self
    }

    pub fn measure_when(mut self, node: u32, measurement: OracleMeasurement) -> Self {
        self.measurements.entry(node).or_default().push(measurement);
        self
    }

    pub fn inputs(&self, node: u32) -> &[ComputeInput] {
        self.inputs.get(&node).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn layout(&self, node: u32) -> Option<NodeOutput> {
        self.layouts.get(&node).copied()
    }

    pub fn final_layout(&self, node: u32) -> Option<NodeOutput> {
        self.final_layouts.get(&node).copied()
    }
}

impl Traverse for OracleTree {
    type Node = u32;
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
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl Compute for OracleTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        self.styles.get(&node).unwrap_or(&DEFAULT_NODE_INPUT)
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
        self.inputs.entry(node).or_default().push(input);

        if let Some(measurements) = self.measurements.get(&node) {
            for measurement in measurements {
                if measurement.matches(input) {
                    return measurement.output;
                }
            }

            panic!("no oracle measurement matched node {node} input {input:?}");
        }

        match self.node_input(node).display.inner_display() {
            Display::Block => compute_block(self, node, input),
            Display::Flex => compute_flex(self, node, input),
            Display::Grid | Display::GridLanes => compute_grid(self, node, input),
            Display::None => {
                self.set_unrounded(node, NodeOutput::with_order(0));
                ComputeOutput::HIDDEN
            }
            Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }
}

impl Round for OracleTree {
    fn unrounded(&self, node: Self::Node) -> NodeOutput {
        self.layouts
            .get(&node)
            .copied()
            .unwrap_or_else(NodeOutput::new)
    }

    fn set_final(&mut self, node: Self::Node, layout: NodeOutput) {
        self.final_layouts.insert(node, layout);
    }
}
