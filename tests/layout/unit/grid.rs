use super::support::oracle_tree::{OracleMeasurement, OracleTree};
use super::*;
use surgeist_layout::{
    CalcExpression, CalcResolver, CalcTerm, GridTemplateAreaRow, GridTemplateAreas,
    LayoutCalcStore, RawGridLine, RawGridPlacement,
};

fn baseline_measure(
    width: Scalar,
    height: Scalar,
    first_baseline: Option<Scalar>,
    last_baseline_from_bottom: Option<Scalar>,
) -> ComputeOutput {
    ComputeOutput::from_sizes_and_baselines(
        Size::new(width, height),
        Size::new(width, height),
        surgeist_layout::Baselines {
            first: Point::new(None, first_baseline),
            last: Point::new(
                None,
                last_baseline_from_bottom.map(|from_bottom| height - from_bottom),
            ),
        },
    )
}

fn compute_oracle_grid(tree: &mut OracleTree) {
    compute_root(
        tree,
        1,
        Size::new(Available::Definite(120.0), Available::Definite(120.0)),
    );
    round_layout(tree, 1);
}

fn compute_oracle_grid_output(tree: &mut OracleTree) -> ComputeOutput {
    surgeist_layout::compute_grid(
        tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(120.0), Some(120.0)),
            available: Size::new(Available::Definite(120.0), Available::Definite(120.0)),
        },
    )
}

fn final_y(tree: &OracleTree, node: u32) -> Scalar {
    tree.final_layout(node)
        .expect("node should have a final layout")
        .location
        .y
}

fn final_height(tree: &OracleTree, node: u32) -> Scalar {
    tree.final_layout(node)
        .expect("node should have a final layout")
        .size
        .height
}

#[test]
fn grid_lanes_display_uses_separate_placement_path_before_child_layout() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(&mut self, node: u32, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return self.outputs[&node];
            }

            match node_input.display.inner_display() {
                Display::Grid | Display::GridLanes => {
                    surgeist_layout::compute_grid(self, node, input)
                }
                Display::Block => surgeist_layout::compute_block(self, node, input),
                Display::Flex => compute_flex(self, node, input),
                Display::None => ComputeOutput::HIDDEN,
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
        type Node = u32;
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.compute_node(node, input)
        }
    }

    let mut tree = RecursiveTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::GridLanes,
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let output = tree.compute_node(
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn grid_lanes_content_size_uses_measured_lane_margin_boxes() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(&mut self, node: u32, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return self.outputs[&node];
            }

            match node_input.display.inner_display() {
                Display::Grid | Display::GridLanes => {
                    surgeist_layout::compute_grid(self, node, input)
                }
                Display::Block => surgeist_layout::compute_block(self, node, input),
                Display::Flex => compute_flex(self, node, input),
                Display::None => ComputeOutput::HIDDEN,
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
        type Node = u32;
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.compute_node(node, input)
        }
    }

    let mut tree = RecursiveTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::GridLanes,
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![TrackComponent::px(0.0)],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        );
    }
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));
    tree.outputs
        .insert(3, ComputeOutput::from_outer_size(Size::new(20.0, 15.0)));

    let output = tree.compute_node(
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(20.0, 30.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 15.0));
}

#[test]
fn grid_lanes_content_size_preserves_resolved_track_sum() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(80.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::default()
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(80.0, 20.0));
}

#[test]
fn named_grid_lanes_use_resolved_raw_grid_axis_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(child.location, Point::new(40.0, 0.0));
    assert_eq!(child.size, Size::new(40.0, 20.0));
}

#[test]
fn named_grid_lanes_intrinsic_sizing_uses_resolved_raw_grid_axis_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![
                    TrackComponent::AUTO,
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::AUTO,
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(10.0, 20.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(50.0, 20.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    let named = tree.layout(3).expect("named lane child should be laid out");

    assert_eq!(output.content_size.width, 60.0);
    assert_eq!(named.location.x, 10.0);
    assert_eq!(named.size.width, 50.0);
}

#[test]
fn named_grid_lanes_resolve_repeated_named_start_and_end_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(20.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(30.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["lane"]),
                ],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 2,
                    },
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 4,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(70.0, 0.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    let child = tree.layout(2).expect("named lane child should be laid out");

    assert_eq!(output.size, Size::new(90.0, 0.0));
    assert_eq!(child.location.x, 20.0);
    assert_eq!(child.size.width, 70.0);
}

#[test]
fn grid_lanes_with_rows_template_uses_columns_as_lane_axis_for_intrinsic_width() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3, 4, 5])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_rows: vec![
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                ],
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::default()
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(4, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(5, ComputeOutput::from_outer_size(Size::new(73.0, 30.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(145.0, 45.0));
    assert_eq!(
        tree.layout(5)
            .expect("spanning item should be laid out")
            .location
            .x,
        72.0
    );
}

#[test]
fn grid_lanes_lane_measurement_honors_min_content_width() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_rows: vec![TrackComponent::AUTO, TrackComponent::AUTO],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(Dimension::MIN_CONTENT, Dimension::AUTO),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(Dimension::MAX_CONTENT, Dimension::AUTO),
                ..NodeInput::default()
            },
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .available(Size::new(
                    Available::definite(54.0),
                    Available::definite(15.0),
                )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(Some(72.0), None))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .available(Size::new(Available::definite(72.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .available(Size::new(Available::definite(72.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(None, Some(30.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::definite(30.0))),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(54.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(54.0),
                    Available::definite(30.0),
                )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(Some(54.0), Some(30.0)))
                .parent(Size::new(Some(54.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(54.0),
                    Available::definite(30.0),
                )),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .available(Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .available(Size::new(
                    Available::definite(72.0),
                    Available::definite(15.0),
                )),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(Some(72.0), None))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .available(Size::new(Available::definite(72.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .available(Size::new(Available::definite(72.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(None, Some(30.0)))
                .available(Size::new(Available::MAX_CONTENT, Available::definite(30.0))),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(72.0),
                    Available::definite(30.0),
                )),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(Some(72.0), Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .available(Size::new(
                    Available::definite(72.0),
                    Available::definite(30.0),
                )),
        );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(72.0, 60.0));
    assert!(tree.inputs(2).iter().any(|input| {
        input.run_mode == RunMode::ComputeSize
            && input.available == Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT)
    }));
}

#[test]
fn named_grid_lanes_place_item_between_named_ordinary_grid_lines() {
    let oracle_lines = support::oracle::grid::NamedGridLines::new(
        support::oracle::grid::GridAxis::Column,
        3,
        vec![
            Vec::<&str>::new(),
            vec!["slot-start"],
            vec![],
            vec!["slot-end"],
        ],
    )
    .unwrap();
    let expected = support::oracle::grid::resolve_named_axis_placement(
        &oracle_lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "slot-start".to_string(),
                occurrence: 1,
            },
            end: support::oracle::grid::NamedGridLine::Named {
                name: "slot-end".to_string(),
                occurrence: 1,
            },
        },
        None,
    )
    .unwrap()
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["slot-start"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["slot-end"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "slot-start".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "slot-end".to_string(),
                        index: 1,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

#[test]
fn named_grid_lanes_span_named_implicit_fallback_line() {
    let oracle_lines = support::oracle::grid::NamedGridLines::new(
        support::oracle::grid::GridAxis::Column,
        1,
        vec![vec!["a"], vec!["a"]],
    )
    .unwrap();
    let expected = support::oracle::grid::resolve_named_axis_placement(
        &oracle_lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "a".to_string(),
                occurrence: 2,
            },
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_string()),
                count: 2,
            },
        },
        None,
    )
    .unwrap()
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_columns: vec![TrackComponent::px(40.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "a".to_string(),
                        index: 2,
                    },
                    RawGridLine::NamedSpan {
                        name: "a".to_string(),
                        index: 2,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

#[test]
fn named_grid_lanes_subgrid_axis_uses_inherited_line_names() {
    let parent_lines = support::oracle::grid::NamedGridLines::new(
        support::oracle::grid::GridAxis::Column,
        4,
        vec![
            vec!["a"],
            vec!["b"],
            Vec::<&str>::new(),
            vec!["c"],
            vec!["d"],
        ],
    )
    .unwrap();
    let subgrid = support::oracle::grid::inherit_named_subgrid_lines(
        &parent_lines,
        support::oracle::grid::TrackSpan::new(2, 5),
        false,
        vec![Vec::<String>::new(), Vec::new(), Vec::new(), Vec::new()],
        None,
    )
    .unwrap();
    let expected = support::oracle::grid::resolve_named_subgrid_axis_placement(
        &subgrid.lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "b".to_string(),
                occurrence: 1,
            },
            end: support::oracle::grid::NamedGridLine::Named {
                name: "c".to_string(),
                occurrence: 1,
            },
        },
        None,
    )
    .unwrap()
    .clamped
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(160.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["b"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["c"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["d"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::GridLanes,
                grid_column: GridPlacement::try_lines(2, 5).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "b".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "c".to_string(),
                        index: 1,
                    },
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("subgridded lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

#[test]
fn grid_lanes_reports_synthesized_container_baselines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(20.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::default()
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 20.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(0.0));
}

#[test]
fn grid_lanes_does_not_apply_lane_axis_baseline_offsets() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(20.0)],
                grid_template_rows: vec![TrackComponent::px(0.0)],
                gap: Size::new(Length::ZERO, Length::px(5.0)),
                align_items: Some(AlignItems::Baseline),
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(20.0, 10.0, Some(2.0), None))
        .measure(3, baseline_measure(20.0, 15.0, Some(12.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 0.0);
    assert_eq!(final_y(&tree, 3), 15.0);
}

#[test]
fn grid_lanes_reports_last_baseline_from_spanning_item_end_edge() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(6.0)))
        .measure(3, baseline_measure(30.0, 80.0, None, Some(8.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.last_baselines.y, Some(72.0));
}

#[test]
fn subgrid_template_resolves_to_empty_explicit_tracks_and_grows_implicit_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4, 5, 6]);
    for child in 2..=6 {
        tree.children.insert(child, vec![]);
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::Subgrid(surgeist_layout::SubgridTrack {
                    name_components: vec![surgeist_layout::SubgridLineNameComponent::LineNames(
                        vec!["main".to_string()],
                    )],
                }),
                TrackComponent::Repeat(
                    surgeist_layout::TrackRepetition::auto_fit(vec![
                        surgeist_layout::TrackSizing::px(10.0),
                    ])
                    .expect("valid track repetition"),
                ),
            ],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            grid_auto_columns: vec![TrackComponent::px(10.0)],
            grid_auto_rows: vec![TrackComponent::px(10.0)],
            ..NodeInput::default()
        },
    );
    for child in 2..=6 {
        tree.styles.insert(child, NodeInput::default());
        tree.outputs
            .insert(child, ComputeOutput::from_outer_size(Size::new(10.0, 10.0)));
    }

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(10.0, 50.0));
    assert_eq!(tree.layouts[&6].location.y, 40.0);
}

fn empty_subgrid_track() -> TrackComponent {
    TrackComponent::Subgrid(surgeist_layout::SubgridTrack {
        name_components: Vec::new(),
    })
}

#[test]
fn both_axis_subgrid_zero_gap_auto_placement_advances_fully_auto_children() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::ZERO, Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                grid_row: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 1);

    assert_eq!(
        tree.final_layout(3)
            .expect("first subgrid child should be laid out")
            .location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(4)
            .expect("second subgrid child should be laid out")
            .location,
        Point::new(40.0, 0.0)
    );
}

#[test]
fn row_subgrid_intrinsic_width_uses_inherited_rows_for_column_auto_flow() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                min_size: Size::new(Dimension::MinContent, Dimension::AUTO),
                grid_auto_flow: GridAutoFlow::Column,
                grid_template_rows: vec![empty_subgrid_track()],
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(100.0, 100.0));
    assert_eq!(tree.layout(2).unwrap().size, Size::new(100.0, 100.0));
    assert_eq!(tree.layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.layout(4).unwrap().location, Point::new(0.0, 50.0));
}

#[test]
fn row_subgrid_constrained_sizing_keeps_fixed_descendants_when_sibling_uses_percent() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_row: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(Dimension::px(100.0), Dimension::px(30.0)),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                size: Size::new(Dimension::px(100.0), Dimension::percent(0.5)),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(100.0, 30.0));
    assert_eq!(tree.layout(2).unwrap().size.height, 30.0);
}

fn row_subgrid_auto_track_sizing_tree(
    columns: Vec<TrackComponent>,
    subgrid_column: GridPlacement,
) -> OracleTree {
    OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: columns,
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: subgrid_column,
                margin: Edges {
                    top: LengthAuto::px(7.0),
                    right: LengthAuto::px(11.0),
                    bottom: LengthAuto::px(3.0),
                    left: LengthAuto::px(5.0),
                },
                padding: Edges {
                    top: Length::px(3.0),
                    right: Length::px(5.0),
                    bottom: Length::px(7.0),
                    left: Length::px(11.0),
                },
                border: Edges {
                    top: Length::px(5.0),
                    right: Length::px(7.0),
                    bottom: Length::px(11.0),
                    left: Length::px(3.0),
                },
                ..NodeInput::DEFAULT
            },
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(58.0, 84.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::NONE)
                .available(Size::new(Available::Definite(58.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(58.0, 84.0)))
                .run_mode(RunMode::PerformLayout)
                .available(Size::new(
                    Available::Definite(58.0),
                    Available::Definite(84.0),
                )),
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(58.0, 116.0)))
}

#[test]
fn row_subgrid_auto_track_sizing_fixed_then_auto_uses_descendant_contribution_once() {
    let mut tree = row_subgrid_auto_track_sizing_tree(
        vec![TrackComponent::px(100.0), TrackComponent::AUTO],
        GridPlacement::try_line(1).expect("valid grid line"),
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size.height, 120.0);
    assert_eq!(tree.layout(2).unwrap().size.height, 110.0);
}

#[test]
fn row_subgrid_auto_track_sizing_auto_then_fixed_uses_descendant_contribution_once() {
    let mut tree = row_subgrid_auto_track_sizing_tree(
        vec![TrackComponent::AUTO, TrackComponent::px(100.0)],
        GridPlacement::try_line(2).expect("valid grid line"),
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size.height, 120.0);
    assert_eq!(tree.layout(2).unwrap().size.height, 110.0);
}

#[test]
fn row_subgrid_intrinsic_width_accumulates_standalone_percent_columns() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [5])
        .children(4, [6])
        .children(5, [])
        .children(6, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_rows: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::MinContent, Dimension::AUTO),
                grid_template_columns: vec![
                    TrackComponent::percent(0.2),
                    TrackComponent::percent(0.3),
                ],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            5,
            NodeInput {
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size.width, 200.0);
    assert_eq!(tree.layout(2).unwrap().size.width, 100.0);
}

#[test]
fn subgrid_line_names_place_child_with_inherited_parent_names() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(160.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["b"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["c"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["d"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(2, 5).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "b".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "c".to_string(),
                        index: 1,
                    },
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("subgrid child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_merge_local_names_at_corresponding_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![TrackComponent::Subgrid(
                    surgeist_layout::SubgridTrack::new(vec![
                        vec!["local-start".to_string()],
                        vec![],
                        vec!["middle".to_string()],
                        vec!["local-end".to_string()],
                    ]),
                )],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "local-start".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "middle".to_string(),
                        index: 1,
                    },
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("local-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_clip_parent_area_generated_names_to_span() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(160.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            None,
                            Some("main".to_string()),
                            Some("main".to_string()),
                            None,
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(2, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("area-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_nested_subgrid_inherits_area_generated_names() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4, 5, 6, 7])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(160.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            None,
                            Some("main".to_string()),
                            Some("main".to_string()),
                            None,
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(2, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(4)
        .expect("nested area-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_named_placement_beyond_span_clamps_to_edge_track() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 2).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(3)),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("clamped child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 40.0);
}

#[test]
fn grid_subgrid_declaration_without_parent_grid_keeps_ordinary_grid_fallback() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![empty_subgrid_track()],
            grid_template_rows: vec![TrackComponent::AUTO],
            grid_auto_columns: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn grid_block_child_with_subgrid_tracks_lays_out_as_block_child() {
    assert_non_grid_child_with_subgrid_tracks_lays_out_as_ordinary_child(Display::Block);
}

#[test]
fn grid_flex_child_with_subgrid_tracks_lays_out_as_flex_child() {
    assert_non_grid_child_with_subgrid_tracks_lays_out_as_ordinary_child(Display::Flex);
}

fn assert_non_grid_child_with_subgrid_tracks_lays_out_as_ordinary_child(display: Display) {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(&mut self, node: u32, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return ComputeOutput::from_outer_size(Size::new(30.0, 12.0));
            }

            match node_input.display.inner_display() {
                Display::Grid | Display::GridLanes => {
                    surgeist_layout::compute_grid(self, node, input)
                }
                Display::Block => surgeist_layout::compute_block(self, node, input),
                Display::Flex => compute_flex(self, node, input),
                Display::None => ComputeOutput::HIDDEN,
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
        type Node = u32;
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
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
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display,
            grid_template_columns: vec![empty_subgrid_track()],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].size.width, 40.0);
    assert_eq!(tree.layouts[&3].size, Size::new(30.0, 12.0));
}

#[test]
fn grid_absolute_child_with_subgrid_tracks_does_not_participate_as_subgrid() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            position: Position::Absolute,
            grid_template_columns: vec![empty_subgrid_track()],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(10.0, 10.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(40.0, 20.0));
    assert_eq!(tree.layouts[&2].size, Size::new(10.0, 10.0));
}

#[test]
fn row_subgrid_child_inherits_parent_baseline_group() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4, 5, 6, 7])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(14.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(8.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 4), 6.0);
}

#[test]
fn row_subgrid_inherited_baseline_accounts_for_margin_border_padding() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                border: Edges {
                    top: Length::px(2.0),
                    ..Edges::all(Length::ZERO)
                },
                padding: Edges {
                    top: Length::px(5.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(14.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(8.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 4), 7.0);
}

#[test]
fn row_subgrid_publishes_descendant_baseline_to_parent_row() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(17.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 49.0);
}

#[test]
fn row_subgrid_without_descendant_publication_uses_container_baseline() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(14.0), None))
        .measure(4, baseline_measure(30.0, 20.0, Some(20.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 6.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn sibling_row_subgrids_revisit_inherited_published_baselines() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [4])
        .children(3, [5])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(4, baseline_measure(30.0, 20.0, Some(30.0), None))
        .measure(5, baseline_measure(30.0, 20.0, Some(8.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 5), 62.0);
}

#[test]
fn column_subgrid_baseline_alignment_does_not_grow_auto_parent_row_twice() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![empty_subgrid_track()],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Grid,
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(3, baseline_measure(15.0, 15.0, Some(12.0), None))
        .measure(4, baseline_measure(30.0, 30.0, Some(24.0), None));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    round_layout(&mut tree, 1);

    assert_eq!(output.size, Size::new(45.0, 30.0));
    assert_eq!(final_height(&tree, 2), 30.0);
    assert_eq!(final_y(&tree, 3), 12.0);
    assert_eq!(final_y(&tree, 4), 0.0);
}

#[test]
fn grid_auto_places_children_into_declared_column_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), Dimension::px(120.0).into()],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(200.0, 40.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 40.0));
    assert_eq!(tree.layouts[&3].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(120.0, 40.0));
}

#[test]
fn grid_column_gap_separates_declared_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(210.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), Dimension::px(120.0).into()],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let expected = DefiniteTracks::new(210.0, 10.0)
        .track(Track::px(80.0))
        .track(Track::px(120.0))
        .solve();
    assert_eq!(output.size, Size::new(210.0, 40.0));
    assert_eq!(output.content_size, Size::new(210.0, 40.0));
    assert_eq!(
        tree.layouts[&2].location,
        Point::new(expected.offset(0), 0.0)
    );
    assert_eq!(
        tree.layouts[&3].location,
        Point::new(expected.offset(1), 0.0)
    );
    assert_eq!(tree.layouts[&3].size, Size::new(expected.size(1), 40.0));
}

#[test]
fn grid_auto_placement_continues_into_declared_rows_with_gap() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(205.0), Dimension::px(75.0)),
            grid_template_columns: vec![TrackComponent::px(100.0), Dimension::px(100.0).into()],
            grid_template_rows: vec![TrackComponent::px(30.0), Dimension::px(40.0).into()],
            gap: Size::new(Length::px(5.0), Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());
    tree.styles.insert(4, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(205.0, 75.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 30.0));
    assert_eq!(tree.layouts[&3].location, Point::new(105.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(100.0, 30.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 35.0));
    assert_eq!(tree.layouts[&4].size, Size::new(100.0, 40.0));
}

#[test]
fn grid_display_none_child_does_not_consume_auto_placement_cell() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        hidden_inputs: Vec<u32>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            if input == ComputeInput::HIDDEN {
                self.hidden_inputs.push(node);
            }

            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), Dimension::px(120.0).into()],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.hidden_inputs, vec![2]);
    assert_eq!(tree.layouts[&2].size, Size::ZERO);
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 40.0));
}

#[test]
fn grid_absolute_child_does_not_consume_auto_placement_cell() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), Dimension::px(120.0).into()],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            size: Size::new(Dimension::px(30.0), Dimension::px(12.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 12.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 40.0));
    let absolute_layout_input = tree.inputs[&2]
        .iter()
        .find(|input| input.run_mode == RunMode::PerformLayout)
        .expect("absolute grid child should be laid out");
    let normal_layout_input = tree.inputs[&3]
        .iter()
        .find(|input| input.run_mode == RunMode::PerformLayout)
        .expect("normal grid child should be laid out");
    assert_eq!(
        absolute_layout_input.known,
        Size::new(Some(30.0), Some(12.0))
    );
    assert_eq!(normal_layout_input.known, Size::new(Some(80.0), Some(40.0)));
}

#[test]
fn named_grid_absolute_child_uses_resolved_raw_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["b"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["c"]),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "b".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "c".to_string(),
                        index: 1,
                    },
                ),
                raw_grid_row: RawGridPlacement::lines(1, 2),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(2)
        .expect("absolute child should be laid out");

    assert_eq!(child.location, Point::new(40.0, 0.0));
    assert_eq!(child.size, Size::new(40.0, 20.0));
}

#[test]
fn vertical_grid_absolute_child_maps_rows_to_physical_x_and_columns_to_y() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_template_columns: vec![TrackComponent::px(30.0), TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(50.0), TrackComponent::px(60.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    let child = tree.layout(2).expect("absolute child should be laid out");

    assert_eq!(output.size, Size::new(70.0, 110.0));
    assert_eq!(child.location, Point::new(0.0, 30.0));
    assert_eq!(child.size, Size::new(60.0, 40.0));
}

#[test]
fn named_grid_in_flow_item_occupies_cell_before_auto_sibling() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["taken"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "taken".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let named = tree
        .final_layout(2)
        .expect("named child should be laid out");
    let auto = tree.final_layout(3).expect("auto child should be laid out");

    assert_eq!(named.location, Point::new(0.0, 0.0));
    assert_eq!(named.size, Size::new(40.0, 20.0));
    assert_eq!(auto.location, Point::new(40.0, 0.0));
    assert_eq!(auto.size, Size::new(40.0, 20.0));
}

#[test]
fn grid_absolute_child_without_explicit_size_uses_measured_size() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            if node == 2 {
                return ComputeOutput::from_outer_size(Size::new(36.0, 14.0));
            }
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(60.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(60.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(36.0, 14.0));
    assert_eq!(tree.inputs[&2][0].known, Size::NONE);
    assert_eq!(
        tree.inputs[&2][0].available,
        Size::new(Available::definite(120.0), Available::definite(60.0))
    );
}

#[test]
fn grid_absolute_child_resolves_size_from_opposing_insets() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(60.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(60.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(8.0),
                right: LengthAuto::px(12.0),
                top: LengthAuto::px(6.0),
                bottom: LengthAuto::px(10.0),
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(8.0, 6.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 44.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(100.0), Some(44.0)));
}

#[test]
fn grid_absolute_child_without_horizontal_insets_uses_rtl_start_alignment() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(30.0, input.known.height.unwrap_or(12.0)))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            justify_items: Some(AlignItems::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::AUTO, Dimension::px(12.0)),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(90.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 12.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(None, Some(12.0)));
}

#[test]
fn grid_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(8.0),
                right: LengthAuto::px(12.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(30.0), Dimension::px(12.0)),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(78.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 12.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(30.0), Some(12.0)));
}

#[test]
fn grid_absolute_child_expands_horizontal_auto_margins() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(30.0), Dimension::px(12.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(45.0, 0.0));
    assert_eq!(tree.layouts[&2].margin.left, 45.0);
    assert_eq!(tree.layouts[&2].margin.right, 45.0);
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 12.0));
}

#[test]
fn grid_absolute_child_expands_vertical_auto_margins() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(100.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
            margin: Edges {
                top: LengthAuto::AUTO,
                bottom: LengthAuto::AUTO,
                ..Edges::all(LengthAuto::ZERO)
            },
            inset: Edges {
                top: LengthAuto::px(10.0),
                bottom: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.top, 40.0);
    assert_eq!(tree.layouts[&2].margin.bottom, 40.0);
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 20.0));
}

#[test]
fn grid_absolute_child_percent_size_resolves_against_grid_area() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0), Dimension::px(80.0).into()],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            size: Size::new(Dimension::percent(0.5), Dimension::percent(0.5)),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(120.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 20.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(40.0), Some(20.0)));
}

#[test]
fn grid_absolute_child_percent_padding_resolves_against_grid_area() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0), Dimension::px(80.0).into()],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            size: Size::new(Dimension::px(30.0), Dimension::px(12.0)),
            padding: Edges::all(Length::percent(0.1)),
            border: Edges::all(Length::percent(0.05)),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(120.0, 0.0));
    assert_eq!(tree.layouts[&2].padding, Edges::all(8.0));
    assert_eq!(tree.layouts[&2].border, Edges::all(4.0));
}

#[test]
fn grid_absolute_child_applies_aspect_ratio_to_authored_size() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(30.0), Dimension::AUTO),
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 15.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(30.0), Some(15.0)));
}

#[test]
fn grid_absolute_child_clamps_authored_size_to_min_and_max() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(80.0), Dimension::px(20.0)),
            min_size: Size::new(Dimension::AUTO, Dimension::px(30.0)),
            max_size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 30.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(50.0), Some(30.0)));
}

#[test]
fn grid_absolute_child_content_box_size_includes_padding_and_border() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].size, Size::new(42.0, 32.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(42.0), Some(32.0)));
}

#[test]
fn grid_absolute_child_size_cannot_shrink_below_padding_and_border() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(4.0), Dimension::px(4.0)),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].size, Size::new(12.0, 12.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(12.0), Some(12.0)));
}

#[test]
fn grid_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(10.0),
                right: LengthAuto::px(20.0),
                top: LengthAuto::AUTO,
                bottom: LengthAuto::AUTO,
            },
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].size, Size::new(90.0, 45.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(90.0), Some(45.0)));
}

#[test]
fn grid_absolute_child_available_space_excludes_non_auto_margins() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(8.0),
                input.known.height.unwrap_or(6.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(80.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            position: Position::Absolute,
            margin: Edges {
                left: LengthAuto::px(10.0),
                right: LengthAuto::px(20.0),
                top: LengthAuto::px(3.0),
                bottom: LengthAuto::px(7.0),
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(
        tree.inputs[&2][0].available,
        Size::new(Available::definite(90.0), Available::definite(70.0))
    );
    assert_eq!(
        tree.inputs[&2][0].parent,
        Size::new(Some(120.0), Some(80.0))
    );
}

#[test]
fn grid_auto_placement_creates_implicit_rows_from_auto_rows() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::px(80.0), Dimension::px(120.0).into()],
            grid_template_rows: vec![TrackComponent::px(30.0)],
            grid_auto_rows: vec![TrackComponent::px(40.0)],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());
    tree.styles.insert(4, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(200.0, 75.0));
    assert_eq!(output.content_size, Size::new(200.0, 75.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 30.0));
    assert_eq!(tree.layouts[&3].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(120.0, 30.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 35.0));
    assert_eq!(tree.layouts[&4].size, Size::new(80.0, 40.0));
}

#[test]
fn grid_auto_rows_repeat_for_multiple_implicit_rows() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4, 5]);
    for node in 2..=5 {
        tree.children.insert(node, vec![]);
        tree.styles.insert(node, NodeInput::default());
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::px(50.0)],
            grid_auto_rows: vec![TrackComponent::px(10.0), Dimension::px(20.0).into()],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(50.0, 75.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 10.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 15.0));
    assert_eq!(tree.layouts[&3].size, Size::new(50.0, 20.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 40.0));
    assert_eq!(tree.layouts[&4].size, Size::new(50.0, 10.0));
    assert_eq!(tree.layouts[&5].location, Point::new(0.0, 55.0));
    assert_eq!(tree.layouts[&5].size, Size::new(50.0, 20.0));
}

#[test]
fn grid_compute_size_applies_aspect_ratio_to_max_size() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            panic!("definite grid compute-size should not measure children")
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            max_size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
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
    );

    assert_eq!(output.size, Size::new(50.0, 25.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn grid_content_box_compute_size_does_not_add_scrollbar_to_authored_size() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            panic!("definite grid compute-size should not measure children")
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            overflow: Point {
                x: Overflow::Visible,
                y: Overflow::Scroll,
            },
            scrollbar_width: 15.0,
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
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
    );

    assert_eq!(output.size, Size::new(42.0, 32.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn grid_scrollbar_gutter_does_not_force_outer_size_past_authored_size() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            ComputeOutput::HIDDEN
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(2.0), Dimension::px(4.0)),
            overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: 15.0,
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
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
    );

    assert_eq!(output.size, Size::new(2.0, 4.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn grid_content_size_mode_ignores_authored_size() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            panic!("empty grid content-size should not measure children")
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(30.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::ContentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(30.0, 20.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn grid_item_margins_reduce_stretched_grid_area() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            margin: Edges {
                top: LengthAuto::px(3.0),
                right: LengthAuto::px(7.0),
                bottom: LengthAuto::px(5.0),
                left: LengthAuto::px(11.0),
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let layout_input = tree.inputs[&2]
        .iter()
        .find(|input| input.run_mode == RunMode::PerformLayout)
        .expect("grid item should be laid out");
    assert_eq!(layout_input.known, Size::new(Some(82.0), Some(32.0)));
    assert_eq!(tree.layouts[&2].location, Point::new(11.0, 3.0));
    assert_eq!(tree.layouts[&2].size, Size::new(82.0, 32.0));
    assert_eq!(tree.layouts[&2].margin.left, 11.0);
    assert_eq!(tree.layouts[&2].margin.right, 7.0);
    assert_eq!(tree.layouts[&2].margin.top, 3.0);
    assert_eq!(tree.layouts[&2].margin.bottom, 5.0);
}

#[test]
fn grid_item_with_aspect_ratio_stretches_width_and_keeps_start_aligned_height() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(100.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let layout_input = tree.inputs[&2]
        .iter()
        .find(|input| input.run_mode == RunMode::PerformLayout)
        .expect("grid item should be laid out");
    assert_eq!(layout_input.known, Size::new(Some(100.0), Some(50.0)));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 50.0));
}

#[test]
fn grid_item_expands_inline_auto_margins_after_child_layout() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            self.outputs[&node]
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            margin: Edges {
                top: LengthAuto::ZERO,
                left: LengthAuto::Auto,
                right: LengthAuto::Auto,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(20.0, 40.0)));

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let layout_input = tree.inputs[&2]
        .iter()
        .find(|input| input.run_mode == RunMode::PerformLayout)
        .expect("grid item should be laid out");
    assert_eq!(layout_input.known.width, None);
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
}

#[test]
fn grid_auto_flow_column_places_children_down_rows_then_across_columns() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    for node in 2..=4 {
        tree.children.insert(node, vec![]);
        tree.styles.insert(node, NodeInput::default());
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::AUTO, Dimension::px(50.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), Dimension::px(30.0).into()],
            grid_auto_columns: vec![TrackComponent::px(40.0)],
            grid_auto_flow: GridAutoFlow::Column,
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(120.0, 50.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 20.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 30.0));
    assert_eq!(tree.layouts[&4].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&4].size, Size::new(40.0, 20.0));
}

#[test]
fn grid_definite_column_line_places_item_in_explicit_track() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    for node in 2..=3 {
        tree.children.insert(node, vec![]);
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), Dimension::px(120.0).into()],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_lines(2, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let columns = DefiniteTracks::new(200.0, 0.0)
        .track(Track::px(80.0))
        .track(Track::px(120.0))
        .solve();
    let column_area = LinePlacement::Lines { start: 2, end: 3 }
        .resolve_axis(1)
        .unwrap();
    let expected_column_area = columns.area(
        column_area.start_line as usize,
        column_area.end_line as usize,
    );

    assert_eq!(
        tree.layouts[&2].location,
        Point::new(expected_column_area.start, 0.0)
    );
    assert_eq!(
        tree.layouts[&2].size,
        Size::new(expected_column_area.size, 40.0)
    );
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 40.0));
}

#[test]
fn grid_definite_row_line_places_item_in_explicit_track() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    for node in 2..=3 {
        tree.children.insert(node, vec![]);
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(80.0), Dimension::px(60.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), Dimension::px(40.0).into()],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_row: GridPlacement::try_lines(2, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 20.0));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 40.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 20.0));
}

#[test]
fn grid_definite_column_span_covers_multiple_tracks_and_gap() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(210.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), Dimension::px(120.0).into()],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(210.0, 40.0));
}

#[test]
fn grid_definite_row_span_covers_multiple_tracks_and_gap() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(80.0), Dimension::px(70.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), Dimension::px(40.0).into()],
            gap: Size::new(Length::ZERO, Length::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 70.0));
}

#[test]
fn grid_column_span_auto_places_across_multiple_free_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(150.0), Dimension::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(40.0),
                Dimension::px(50.0).into(),
                Dimension::px(60.0).into(),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_span(2).expect("valid grid span"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let columns = DefiniteTracks::new(150.0, 0.0)
        .track(Track::px(40.0))
        .track(Track::px(50.0))
        .track(Track::px(60.0))
        .solve();
    let mut placement = AutoPlacer::try_new(3, 1, Flow::Row).unwrap();
    let first_area = placement.place(2, 1).unwrap();
    let second_area = placement.place(1, 1).unwrap();
    let expected_first_columns = columns.area(
        first_area.column_start,
        first_area.column_start + first_area.column_span,
    );
    let expected_second_columns = columns.area(
        second_area.column_start,
        second_area.column_start + second_area.column_span,
    );

    assert_eq!(
        tree.layouts[&2].location,
        Point::new(expected_first_columns.start, 0.0)
    );
    assert_eq!(
        tree.layouts[&2].size,
        Size::new(expected_first_columns.size, 20.0)
    );
    assert_eq!(
        tree.layouts[&3].location,
        Point::new(expected_second_columns.start, 0.0)
    );
    assert_eq!(
        tree.layouts[&3].size,
        Size::new(expected_second_columns.size, 20.0)
    );
}

#[test]
fn grid_dense_auto_flow_backfills_earlier_free_cells() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(90.0), Dimension::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(30.0),
                Dimension::px(30.0).into(),
                Dimension::px(30.0).into(),
            ],
            grid_template_rows: vec![TrackComponent::px(10.0), Dimension::px(10.0).into()],
            grid_auto_flow: GridAutoFlow::RowDense,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            grid_column: GridPlacement::try_span(2).expect("valid grid span"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(4, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let columns = DefiniteTracks::new(90.0, 0.0)
        .track(Track::px(30.0))
        .track(Track::px(30.0))
        .track(Track::px(30.0))
        .solve();
    let rows = DefiniteTracks::new(20.0, 0.0)
        .track(Track::px(10.0))
        .track(Track::px(10.0))
        .solve();
    let mut placement = AutoPlacer::try_new(3, 2, Flow::RowDense)
        .unwrap()
        .occupied(GridArea::new(2, 1, 1, 1));
    let third_area = placement.place(2, 1).unwrap();
    let fourth_area = placement.place(1, 1).unwrap();
    let second_columns = columns.area(2, 3);
    let third_columns = columns.area(
        third_area.column_start,
        third_area.column_start + third_area.column_span,
    );
    let third_rows = rows.area(
        third_area.row_start,
        third_area.row_start + third_area.row_span,
    );
    let fourth_columns = columns.area(
        fourth_area.column_start,
        fourth_area.column_start + fourth_area.column_span,
    );
    let fourth_rows = rows.area(
        fourth_area.row_start,
        fourth_area.row_start + fourth_area.row_span,
    );

    assert_eq!(
        tree.layouts[&2].location,
        Point::new(second_columns.start, 0.0)
    );
    assert_eq!(
        tree.layouts[&3].location,
        Point::new(third_columns.start, third_rows.start)
    );
    assert_eq!(
        tree.layouts[&3].size,
        Size::new(third_columns.size, third_rows.size)
    );
    assert_eq!(
        tree.layouts[&4].location,
        Point::new(fourth_columns.start, fourth_rows.start)
    );
    assert_eq!(
        tree.layouts[&4].size,
        Size::new(fourth_columns.size, fourth_rows.size)
    );
}

#[test]
fn grid_dense_row_flow_places_definite_row_items_before_auto_items() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, (2..=10).collect());
    for node in 2..=10 {
        tree.children.insert(node, vec![]);
        tree.styles.insert(node, NodeInput::default());
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(120.0)),
            grid_auto_flow: GridAutoFlow::RowDense,
            grid_template_columns: vec![
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
            ],
            grid_template_rows: vec![
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
            ],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        4,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            size: Size::new(Dimension::px(35.0), Dimension::px(35.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        7,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        9,
        NodeInput {
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(120.0), Some(120.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 40.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 80.0));
    assert_eq!(tree.layouts[&7].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&9].location, Point::new(40.0, 0.0));
}

#[test]
fn grid_definite_column_auto_row_stays_in_auto_placement_order() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), Dimension::px(120.0).into()],
            grid_template_rows: vec![TrackComponent::px(20.0), Dimension::px(20.0).into()],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(
        3,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 20.0));
}

#[test]
fn grid_definite_column_line_span_resolves_from_start_line() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(150.0), Dimension::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(30.0),
                Dimension::px(40.0).into(),
                Dimension::px(50.0).into(),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(5.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line_span(2, 2).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let columns = DefiniteTracks::new(150.0, 5.0)
        .track(Track::px(30.0))
        .track(Track::px(40.0))
        .track(Track::px(50.0))
        .solve();
    let column_area = LinePlacement::LineSpan { start: 2, span: 2 }
        .resolve_axis(1)
        .unwrap();
    let expected_column_area = columns.area(
        column_area.start_line as usize,
        column_area.end_line as usize,
    );

    assert_eq!(
        tree.layouts[&2].location,
        Point::new(expected_column_area.start, 0.0)
    );
    assert_eq!(
        tree.layouts[&2].size,
        Size::new(expected_column_area.size, 20.0)
    );
}

#[test]
fn grid_definite_column_span_line_resolves_to_end_line() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(150.0), Dimension::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(30.0),
                Dimension::px(40.0).into(),
                Dimension::px(50.0).into(),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(5.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_span_line(2, 4).expect("valid grid span line"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let columns = DefiniteTracks::new(150.0, 5.0)
        .track(Track::px(30.0))
        .track(Track::px(40.0))
        .track(Track::px(50.0))
        .solve();
    let column_area = LinePlacement::SpanLine { span: 2, end: 4 }
        .resolve_axis(1)
        .unwrap();
    let expected_column_area = columns.area(
        column_area.start_line as usize,
        column_area.end_line as usize,
    );

    assert_eq!(
        tree.layouts[&2].location,
        Point::new(expected_column_area.start, 0.0)
    );
    assert_eq!(
        tree.layouts[&2].size,
        Size::new(expected_column_area.size, 20.0)
    );
}

#[test]
fn grid_mixed_positive_negative_line_span_counts_actual_tracks_for_auto_growth() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::AUTO),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_lines(2, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let root = tree.final_layout(1).expect("root should be laid out");
    let spanning = tree
        .final_layout(2)
        .expect("spanning child should be laid out");
    let auto = tree.final_layout(3).expect("auto child should be laid out");

    assert_eq!(root.size.height, 20.0);
    assert_eq!(spanning.location, Point::new(40.0, 0.0));
    assert_eq!(spanning.size, Size::new(80.0, 20.0));
    assert_eq!(auto.location, Point::new(0.0, 0.0));
    assert_eq!(auto.size, Size::new(40.0, 20.0));
}

#[test]
fn grid_row_span_auto_placement_creates_enough_implicit_rows() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::px(50.0)],
            grid_auto_rows: vec![
                TrackComponent::px(10.0),
                Dimension::px(20.0).into(),
                Dimension::px(30.0).into(),
            ],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_row: GridPlacement::try_span(3).expect("valid grid span"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 70.0));
}

#[test]
fn grid_definite_column_line_creates_required_implicit_columns() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::AUTO, Dimension::px(10.0)),
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            grid_auto_columns: vec![TrackComponent::px(30.0), Dimension::px(40.0).into()],
            gap: Size::new(Length::px(5.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_lines(3, 4).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(60.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 10.0));
}

#[test]
fn grid_definite_column_end_line_resolves_to_previous_track() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(10.0)),
            grid_template_columns: vec![
                TrackComponent::px(20.0),
                Dimension::px(30.0).into(),
                Dimension::px(40.0).into(),
            ],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            gap: Size::new(Length::px(5.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_end_line(3).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(25.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 10.0));
}

#[test]
fn grid_definite_row_end_line_resolves_to_previous_track() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(20.0), Dimension::px(90.0)),
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![
                TrackComponent::px(10.0),
                Dimension::px(20.0).into(),
                Dimension::px(30.0).into(),
            ],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_row: GridPlacement::try_end_line(3).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 15.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 20.0));
}

#[test]
fn grid_justify_content_center_offsets_tracks_inside_inner_width() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(20.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let expected = align_tracks_report(
        200.0,
        vec![80.0],
        0.0,
        TrackAlignment::Center,
        AlignmentSafety::Unsafe,
    );

    assert_eq!(
        tree.layouts[&2].location,
        Point::new(expected.offsets[0], 0.0)
    );
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 20.0));
}

#[test]
fn grid_align_content_center_offsets_tracks_inside_inner_height() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(80.0), Dimension::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            align_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 30.0));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 40.0));
}

#[test]
fn grid_safe_align_content_falls_back_to_start_when_tracks_overflow() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
            ],
            align_content: Some(AlignContent::SafeCenter),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let expected = align_tracks_report(
        100.0,
        vec![40.0, 40.0, 40.0],
        0.0,
        TrackAlignment::Center,
        AlignmentSafety::Safe,
    );

    assert!(expected.safe_fallback_used);
    assert_eq!(
        tree.layouts[&2].location,
        Point::new(0.0, expected.offsets[0])
    );
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 40.0));
}

#[test]
fn grid_justify_content_space_between_distributes_free_width_between_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(20.0)),
            grid_template_columns: vec![TrackComponent::px(50.0), Dimension::px(50.0).into()],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            justify_content: Some(AlignContent::SpaceBetween),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(150.0, 0.0));
}

#[test]
fn grid_justify_content_space_around_and_evenly_distribute_free_width() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    fn run(alignment: AlignContent) -> (Point, Point) {
        let mut tree = GridTree::default();
        tree.children.insert(1, vec![2, 3]);
        tree.children.insert(2, vec![]);
        tree.children.insert(3, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(200.0), Dimension::px(20.0)),
                grid_template_columns: vec![TrackComponent::px(50.0), Dimension::px(50.0).into()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(alignment),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(2, NodeInput::default());
        tree.styles.insert(3, NodeInput::default());

        surgeist_layout::compute_grid(
            &mut tree,
            1,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(Some(300.0), Some(200.0)),
                available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            },
        );

        (tree.layouts[&2].location, tree.layouts[&3].location)
    }

    assert_eq!(
        run(AlignContent::SpaceAround),
        (Point::new(22.5, 0.0), Point::new(127.5, 0.0))
    );
    assert_eq!(
        run(AlignContent::SpaceEvenly),
        (Point::new(30.0, 0.0), Point::new(120.0, 0.0))
    );
}

#[test]
fn grid_fraction_tracks_share_leftover_space_after_fixed_tracks_and_gaps() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(300.0), Dimension::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(50.0),
                TrackComponent::fr(1.0),
                Dimension::fr(2.0).into(),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());
    tree.styles.insert(4, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let expected = TrackSizingSlice::definite_columns(300.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::flex(1.0))
        .track(GridTrack::flex(2.0))
        .solve();
    assert_eq!(expected.final_tracks.len(), 3);
    assert_eq!(
        tree.layouts[&2].location,
        Point::new(expected.final_tracks[0].offset, 0.0)
    );
    assert_eq!(
        tree.layouts[&2].size,
        Size::new(expected.final_tracks[0].size, 20.0)
    );
    assert!((tree.layouts[&3].location.x - expected.final_tracks[1].offset).abs() < 0.000_001);
    assert!((tree.layouts[&3].size.width - expected.final_tracks[1].size).abs() < 0.000_001);
    assert!((tree.layouts[&4].location.x - expected.final_tracks[2].offset).abs() < 0.000_001);
    assert!((tree.layouts[&4].size.width - expected.final_tracks[2].size).abs() < 0.000_001);
}

#[test]
fn grid_fraction_tracks_use_available_space_when_container_size_is_auto() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::fr(1.0), Dimension::fr(2.0).into()],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(12.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(200.0)),
            available: Size::new(Available::definite(120.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(36.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(48.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(72.0, 20.0));
}

#[test]
fn grid_fraction_tracks_clamp_available_space_to_min_size() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            min_size: Size::new(Dimension::px(180.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::fr(1.0), Dimension::fr(2.0).into()],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(12.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(200.0)),
            available: Size::new(Available::definite(120.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(56.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(68.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(112.0, 20.0));
}

#[test]
fn grid_auto_fraction_tracks_resolve_after_required_tracks_are_known() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::px(20.0)),
            grid_template_columns: vec![TrackComponent::px(50.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            grid_auto_columns: vec![TrackComponent::fr(1.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(200.0), Some(100.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(60.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(140.0, 20.0));
}

#[test]
fn grid_stretch_distributes_free_space_to_auto_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(220.0), Dimension::px(20.0)),
            grid_template_columns: vec![TrackComponent::AUTO, Dimension::AUTO.into()],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(20.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(220.0), Some(100.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let expected = TrackSizingSlice::definite_columns(220.0, 20.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .stretch_auto_tracks()
        .solve();

    assert_eq!(
        tree.layouts[&2].location,
        Point::new(expected.final_tracks[0].offset, 0.0)
    );
    assert_eq!(
        tree.layouts[&2].size,
        Size::new(expected.final_tracks[0].size, 20.0)
    );
    assert_eq!(
        tree.layouts[&3].location,
        Point::new(expected.final_tracks[1].offset, 0.0)
    );
    assert_eq!(
        tree.layouts[&3].size,
        Size::new(expected.final_tracks[1].size, 20.0)
    );
}

#[test]
fn grid_auto_track_uses_single_item_intrinsic_contribution() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            self.outputs[&node]
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(80.0, 24.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let expected_columns = TrackSizingSlice::indefinite_columns(0.0)
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 80.0,
            max_content: 80.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();
    let expected_rows = TrackSizingSlice::indefinite_rows(0.0)
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 24.0,
            max_content: 24.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    assert_eq!(
        output.size,
        Size::new(
            expected_columns.final_tracks[0].size,
            expected_rows.final_tracks[0].size
        )
    );
    assert_eq!(
        output.content_size,
        Size::new(
            expected_columns.final_tracks[0].size,
            expected_rows.final_tracks[0].size
        )
    );
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.layouts[&2].size,
        Size::new(
            expected_columns.final_tracks[0].size,
            expected_rows.final_tracks[0].size
        )
    );
    assert_eq!(tree.inputs[&2][0].run_mode, RunMode::ComputeSize);
    let layout_input = tree.inputs[&2]
        .iter()
        .find(|input| input.run_mode == RunMode::PerformLayout)
        .expect("grid item should be laid out after intrinsic measurement");
    assert_eq!(
        layout_input.known,
        Size::new(
            Some(expected_columns.final_tracks[0].size),
            Some(expected_rows.final_tracks[0].size)
        )
    );
}

#[test]
fn grid_auto_width_does_not_stretch_auto_tracks_to_available_space() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(80.0),
                    input.known.height.unwrap_or(10.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(400.0), None),
            available: Size::new(Available::definite(400.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(160.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 10.0));
    assert_eq!(tree.layouts[&3].location, Point::new(80.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 10.0));
}

#[test]
fn grid_auto_width_uses_max_width_as_track_available_space() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(10.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            max_size: Size::new(Dimension::px(260.0), Dimension::Auto),
            grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(None, None),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(260.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(160.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 10.0));
}

#[test]
fn grid_row_intrinsic_sizing_uses_resolved_column_width() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            match input.available.width {
                Available::Definite(width) if width <= 30.0 => {
                    ComputeOutput::from_outer_size(Size::new(30.0, 20.0))
                }
                _ => ComputeOutput::from_outer_size(Size::new(40.0, 10.0)),
            }
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::px(30.0)],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(30.0, 20.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 20.0));
    assert!(
        tree.inputs[&2]
            .iter()
            .any(|input| input.run_mode == RunMode::ComputeSize
                && input.known.width == Some(30.0)
                && input.available.width == Available::Definite(30.0)),
        "grid row sizing should measure the item against the resolved column width"
    );
}

#[test]
fn grid_layout_percent_columns_rerun_row_sizing_with_resolved_width() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            match input.known.width {
                Some(width) if width <= 80.0 => {
                    ComputeOutput::from_outer_size(Size::new(width, 96.0))
                }
                Some(width) => ComputeOutput::from_outer_size(Size::new(width, 64.0)),
                None => ComputeOutput::from_outer_size(Size::new(100.0, 64.0)),
            }
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(80.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::percent(1.0)],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(80.0, 96.0));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 96.0));
    assert!(
        tree.inputs[&2]
            .iter()
            .any(|input| input.run_mode == RunMode::ComputeSize
                && input.known.width == Some(80.0)
                && input.available.width == Available::Definite(80.0)),
        "layout-time row sizing should be rerun against the resolved percent column width"
    );
}

#[test]
fn nested_subgrid_percent_columns_rerun_rows_after_inherited_width_and_margin() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    left: LengthAuto::px(10.0),
                    right: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::percent(1.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    right: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(100.0, 64.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(None, None))
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::MAX_CONTENT,
                )),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 96.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(Some(80.0), None))
                .available(Size::new(Available::Definite(80.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 96.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, None))
                .available(Size::new(
                    Available::Definite(80.0),
                    Available::Definite(96.0),
                )),
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(100.0, 64.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    round_layout(&mut tree, 1);

    assert_eq!(output.size.height, 96.0);
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(85.0, 96.0));
    assert_eq!(tree.final_layout(3).unwrap().size, Size::new(80.0, 96.0));
}

#[test]
fn row_subgrid_percent_column_leaf_uses_spanned_inline_size_for_row_contribution() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![
                    TrackComponent::percent(0.5),
                    TrackComponent::percent(0.5),
                ],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(50.0, 90.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(Some(50.0), None))
                .available(Size::new(Available::Definite(50.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(50.0, 90.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, None))
                .available(Size::new(
                    Available::Definite(50.0),
                    Available::Definite(90.0),
                )),
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(100.0, 40.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    round_layout(&mut tree, 1);

    assert_eq!(output.size, Size::new(100.0, 90.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(100.0, 90.0));
    assert!(
        tree.inputs(3)
            .iter()
            .any(|input| input.run_mode == RunMode::ComputeSize
                && input.known.width == Some(50.0)
                && input.available.width == Available::Definite(50.0)),
        "row contribution should measure the leaf against its 50px column span"
    );
}

#[test]
fn orthogonal_nested_subgrid_width_includes_full_horizontal_leaf_contribution() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 6])
        .children(3, [4, 5])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                gap: Size::new(Length::px(20.0), Length::px(20.0)),
                border: Edges::all(Length::px(3.0)),
                grid_template_columns: vec![TrackComponent::px(100.0), TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::px(100.0), TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                gap: Size::new(Length::px(100.0), Length::px(100.0)),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_span(2).expect("valid grid span"),
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                gap: Size::new(Length::px(100.0), Length::px(100.0)),
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(4, NodeInput::DEFAULT)
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(24.0, 24.0)))
        .measure(5, ComputeOutput::from_outer_size(Size::new(24.0, 24.0)))
        .measure(6, ComputeOutput::from_outer_size(Size::new(72.0, 24.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size.width, 238.0);
}

#[test]
fn vertical_rl_grid_places_distinct_rows_on_physical_x_axis() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_template_columns: vec![TrackComponent::px(30.0), TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(50.0), TrackComponent::px(60.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT)
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(70.0, 110.0));
    assert_eq!(tree.layout(2).unwrap().location, Point::new(60.0, 0.0));
    assert_eq!(tree.layout(3).unwrap().location, Point::new(0.0, 30.0));
}

#[test]
fn grid_row_intrinsic_sizing_includes_item_vertical_margins() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(50.0, 10.0))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(50.0), Some(100.0)),
            available: Size::new(Available::definite(50.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(50.0, 20.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 10.0));
    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 10.0));
}

#[test]
fn grid_minmax_max_content_minimum_overrides_fixed_maximum() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(40.0),
                input.known.height.unwrap_or(10.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::minmax(
                MinTrackSizing::MAX_CONTENT,
                MaxTrackSizing::px(10.0),
            )],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(40.0, 40.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 40.0));
}

#[test]
fn grid_auto_placed_intrinsic_items_size_their_placed_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            match input.run_mode {
                RunMode::ComputeSize => self.outputs[&node],
                RunMode::PerformRootLayout | RunMode::PerformLayout => {
                    ComputeOutput::from_outer_size(Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ))
                }
                RunMode::PerformHiddenLayout => ComputeOutput::HIDDEN,
            }
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO, Dimension::AUTO.into()],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(10.0, 20.0)));
    tree.outputs
        .insert(3, ComputeOutput::from_outer_size(Size::new(100.0, 20.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(110.0, 20.0));
    assert_eq!(output.content_size, Size::new(110.0, 20.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(10.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(10.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(100.0, 20.0));
}

#[test]
fn grid_intrinsic_column_sizing_treats_horizontal_percent_margins_as_zero() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            match input.run_mode {
                RunMode::ComputeSize => self.outputs[&node],
                RunMode::PerformRootLayout | RunMode::PerformLayout => {
                    ComputeOutput::from_outer_size(Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ))
                }
                RunMode::PerformHiddenLayout => ComputeOutput::HIDDEN,
            }
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::percent(0.5),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::percent(0.5),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size.width, 20.0);
}

#[test]
fn grid_nested_stretch_resolves_block_padding_percent_against_inline_size() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(&mut self, node: u32, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return compute_leaf(input, &node_input, |known, _available| {
                    Size::new(known.width.unwrap_or(0.0), known.height.unwrap_or(0.0))
                });
            }

            match node_input.display.inner_display() {
                Display::Grid | Display::GridLanes => {
                    surgeist_layout::compute_grid(self, node, input)
                }
                Display::Block => surgeist_layout::compute_block(self, node, input),
                Display::Flex => compute_flex(self, node, input),
                Display::None => ComputeOutput::HIDDEN,
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
        type Node = u32;
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
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
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            padding: Edges {
                top: Length::percent(0.2),
                right: Length::ZERO,
                bottom: Length::ZERO,
                left: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(200.0, 40.0));
    assert_eq!(tree.layouts[&2].size, Size::new(200.0, 40.0));
    assert_eq!(tree.layouts[&3].size, Size::new(200.0, 40.0));
}

#[test]
fn grid_nested_percent_margins_resolve_against_resolved_nested_inline_size() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl RecursiveTree {
        fn compute_node(&mut self, node: u32, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return compute_leaf(input, &node_input, |known, _available| {
                    Size::new(known.width.unwrap_or(0.0), known.height.unwrap_or(0.0))
                });
            }

            match node_input.display.inner_display() {
                Display::Grid | Display::GridLanes => {
                    surgeist_layout::compute_grid(self, node, input)
                }
                Display::Block => surgeist_layout::compute_block(self, node, input),
                Display::Flex => compute_flex(self, node, input),
                Display::None => ComputeOutput::HIDDEN,
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
        type Node = u32;
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

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
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
            display: Display::Grid,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::percent(0.5), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::percent(0.45), Dimension::AUTO),
            margin: Edges::all(LengthAuto::percent(0.05)),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(200.0, 10.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 10.0));
    assert_eq!(tree.layouts[&3].location, Point::new(5.0, 5.0));
    assert_eq!(tree.layouts[&3].size, Size::new(45.0, 0.0));
}

#[test]
fn grid_recomputes_min_content_columns_from_resolved_row_height() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            compute_leaf(input, &node_input, |known, _available| match node {
                2 => Size::new(
                    if known.height == Some(40.0) {
                        40.0
                    } else {
                        20.0
                    },
                    known.height.unwrap_or(40.0),
                ),
                3 => Size::new(20.0, 20.0),
                _ => Size::ZERO,
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::MIN_CONTENT],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            writing_mode: WritingMode::VerticalLr,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(3, NodeInput::default());

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(40.0, 60.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 40.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 40.0));
    assert_eq!(tree.layouts[&3].size, Size::new(40.0, 20.0));
}

#[test]
fn grid_spanning_item_redistributes_beyond_fit_content_limit() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            compute_leaf(input, &node_input, |_known, available| {
                if node == 4 && available.width == Available::MIN_CONTENT {
                    Size::new(40.0, 40.0)
                } else if node == 4 {
                    Size::new(80.0, 40.0)
                } else {
                    Size::ZERO
                }
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::Track(surgeist_layout::TrackSizing {
                    min: MinTrackSizing::Auto,
                    max: MaxTrackSizing::MaxContent,
                }),
                TrackComponent::Track(surgeist_layout::TrackSizing {
                    min: MinTrackSizing::Auto,
                    max: MaxTrackSizing::FitContent(Length::px(10.0)),
                }),
            ],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        4,
        NodeInput {
            grid_column: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(80.0, 40.0));
    assert_eq!(tree.layouts[&2].size, Size::new(60.0, 40.0));
    assert_eq!(tree.layouts[&3].location, Point::new(60.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 40.0));
    assert_eq!(tree.layouts[&4].size, Size::new(80.0, 40.0));
}

#[test]
fn grid_spanning_item_grows_auto_track_after_min_content_track() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            compute_leaf(input, &node_input, |_known, available| {
                if node == 4 && available.width == Available::MIN_CONTENT {
                    Size::new(40.0, 10.0)
                } else if node == 4 {
                    Size::new(80.0, 10.0)
                } else {
                    Size::ZERO
                }
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::MIN_CONTENT, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());
    tree.styles.insert(
        4,
        NodeInput {
            grid_column: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(80.0, 50.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 40.0));
    assert_eq!(tree.layouts[&3].location, Point::new(20.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(60.0, 40.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 40.0));
    assert_eq!(tree.layouts[&4].size, Size::new(80.0, 10.0));
}

#[test]
fn grid_clipped_spanning_item_distributes_across_min_content_and_auto_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            compute_leaf(input, &node_input, |_known, available| {
                if node == 4 && available.width == Available::MIN_CONTENT {
                    Size::new(40.0, 10.0)
                } else if node == 4 {
                    Size::new(80.0, 10.0)
                } else {
                    Size::ZERO
                }
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::MIN_CONTENT, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.styles.insert(3, NodeInput::default());
    tree.styles.insert(
        4,
        NodeInput {
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            grid_column: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(80.0, 50.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 40.0));
    assert_eq!(tree.layouts[&3].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(40.0, 40.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 40.0));
    assert_eq!(tree.layouts[&4].size, Size::new(80.0, 10.0));
}

#[test]
fn grid_spanning_item_grows_underfilled_auto_track_first() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            compute_leaf(input, &node_input, |_known, _available| Size::ZERO)
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    for node in 2..=4 {
        tree.children.insert(node, vec![]);
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(320.0), Dimension::px(640.0)),
            grid_template_columns: vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::fr(1.0),
            ],
            grid_template_rows: vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::fr(1.0),
            ],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(Dimension::px(40.0), Dimension::px(30.0)),
            grid_column: GridPlacement::try_line_span(2, 2).expect("valid grid line span"),
            grid_row: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        4,
        NodeInput {
            size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
            grid_column: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            grid_row: GridPlacement::try_line(3).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(320.0, 640.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 50.0));
    assert_eq!(tree.layouts[&3].location, Point::new(100.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(40.0, 30.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 50.0));
    assert_eq!(tree.layouts[&4].size, Size::new(120.0, 20.0));
}

#[test]
fn grid_spanning_item_reserves_percent_track_from_max_content_size() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        unrounded: HashMap<u32, NodeOutput>,
        rounded: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.unrounded.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            compute_leaf(input, &node_input, |_known, available| {
                if node == 2 && available.width == Available::MIN_CONTENT {
                    Size::new(80.0, 40.0)
                } else if node == 2 {
                    Size::new(160.0, 40.0)
                } else {
                    Size::ZERO
                }
            })
        }
    }

    impl Round for GridTree {
        fn unrounded(&self, node: Self::Node) -> NodeOutput {
            self.unrounded[&node]
        }

        fn set_final(&mut self, node: Self::Node, layout: NodeOutput) {
            self.rounded.insert(node, layout);
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4, 5, 6, 7, 8]);
    for node in 2..=8 {
        tree.children.insert(node, vec![]);
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::MIN_CONTENT,
                TrackComponent::MAX_CONTENT,
                TrackComponent::Track(surgeist_layout::TrackSizing::fit_content(Length::px(20.0))),
                TrackComponent::AUTO,
                TrackComponent::px(10.0),
                TrackComponent::percent(0.2),
            ],
            grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line_span(1, 6).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );
    for node in 3..=8 {
        tree.styles.insert(node, NodeInput::default());
    }

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    assert_eq!(output.size, Size::new(160.0, 80.0));
    let mut root_layout = NodeOutput::new();
    root_layout.size = output.size;
    root_layout.content_size = output.content_size;
    tree.unrounded.insert(1, root_layout);

    round_layout(&mut tree, 1);
    assert_eq!(tree.rounded[&3].size, Size::new(10.0, 40.0));
    assert_eq!(tree.rounded[&4].location, Point::new(10.0, 40.0));
    assert_eq!(tree.rounded[&4].size, Size::new(89.0, 40.0));
    assert_eq!(tree.rounded[&5].location, Point::new(99.0, 40.0));
    assert_eq!(tree.rounded[&5].size, Size::new(10.0, 40.0));
    assert_eq!(tree.rounded[&6].location, Point::new(109.0, 40.0));
    assert_eq!(tree.rounded[&6].size, Size::new(9.0, 40.0));
    assert_eq!(tree.rounded[&7].location, Point::new(118.0, 40.0));
    assert_eq!(tree.rounded[&7].size, Size::new(10.0, 40.0));
    assert_eq!(tree.rounded[&8].location, Point::new(128.0, 40.0));
    assert_eq!(tree.rounded[&8].size, Size::new(32.0, 40.0));
}

#[test]
fn grid_spanning_item_counts_definite_minmax_floors_when_reserving_percent_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        unrounded: HashMap<u32, NodeOutput>,
        rounded: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.unrounded.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            let node_input = self.styles[&node].clone();
            compute_leaf(input, &node_input, |_known, available| {
                if node == 2 && available.width == Available::MIN_CONTENT {
                    Size::new(160.0, 40.0)
                } else if node == 2 {
                    Size::new(320.0, 40.0)
                } else {
                    Size::ZERO
                }
            })
        }
    }

    impl Round for GridTree {
        fn unrounded(&self, node: Self::Node) -> NodeOutput {
            self.unrounded[&node]
        }

        fn set_final(&mut self, node: Self::Node, layout: NodeOutput) {
            self.rounded.insert(node, layout);
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, (2..=15).collect());
    for node in 2..=15 {
        tree.children.insert(node, vec![]);
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::MIN_CONTENT,
                TrackComponent::MAX_CONTENT,
                TrackComponent::Track(surgeist_layout::TrackSizing::fit_content(Length::px(20.0))),
                TrackComponent::AUTO,
                TrackComponent::px(10.0),
                TrackComponent::percent(0.2),
                TrackComponent::minmax(MinTrackSizing::px(2.0), MaxTrackSizing::AUTO),
                TrackComponent::minmax(MinTrackSizing::px(2.0), MaxTrackSizing::px(4.0)),
                TrackComponent::minmax(MinTrackSizing::px(2.0), MaxTrackSizing::MIN_CONTENT),
                TrackComponent::minmax(MinTrackSizing::px(2.0), MaxTrackSizing::MAX_CONTENT),
                TrackComponent::minmax(MinTrackSizing::MIN_CONTENT, MaxTrackSizing::MAX_CONTENT),
                TrackComponent::minmax(MinTrackSizing::MIN_CONTENT, MaxTrackSizing::AUTO),
                TrackComponent::minmax(MinTrackSizing::MAX_CONTENT, MaxTrackSizing::AUTO),
            ],
            grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line_span(1, 13).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );
    for node in 3..=15 {
        tree.styles.insert(node, NodeInput::default());
    }

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    let mut root_layout = NodeOutput::new();
    root_layout.size = output.size;
    root_layout.content_size = output.content_size;
    tree.unrounded.insert(1, root_layout);

    round_layout(&mut tree, 1);
    let widths = (3..=15)
        .map(|node| tree.rounded[&node].size.width)
        .collect::<Vec<_>>();
    assert_eq!(output.size, Size::new(322.0, 80.0));
    assert_eq!(
        widths,
        vec![
            11.0, 91.0, 11.0, 11.0, 10.0, 65.0, 2.0, 4.0, 2.0, 2.0, 11.0, 11.0, 91.0
        ]
    );
}

#[test]
fn grid_content_size_includes_visible_child_overflow_content() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
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

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 10.0));
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn grid_content_size_for_later_column_uses_item_grid_area_origin() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::px(50.0), Dimension::px(50.0).into()],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 10.0), Size::new(80.0, 10.0)),
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(50.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 10.0));
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
}

#[test]
fn grid_auto_size_re_resolves_indefinite_percentage_tracks_from_visible_content() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(100.0),
                input.known.height.unwrap_or(100.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    for node in 2..=4 {
        tree.children.insert(node, vec![]);
        tree.styles.insert(node, NodeInput::default());
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::percent(0.4),
                TrackComponent::percent(0.4),
                TrackComponent::percent(0.4),
            ],
            grid_template_rows: vec![TrackComponent::percent(0.5), TrackComponent::percent(0.8)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        4,
        NodeInput {
            grid_column: GridPlacement::try_line(3).expect("valid grid line"),
            grid_row: GridPlacement::try_line(2).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 100.0));
    assert_eq!(tree.layouts[&3].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(40.0, 50.0));
    assert_eq!(tree.layouts[&4].location, Point::new(80.0, 50.0));
    assert_eq!(tree.layouts[&4].size, Size::new(40.0, 80.0));
}

#[test]
fn grid_auto_size_ignores_ineligible_row_subgrid_when_resolving_percent_columns() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(100.0),
                input.known.height.unwrap_or(100.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3]);
    for node in 2..=3 {
        tree.children.insert(node, vec![]);
        tree.styles.insert(node, NodeInput::default());
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::percent(0.5), TrackComponent::percent(0.5)],
            grid_template_rows: vec![empty_subgrid_track()],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size.width, 100.0);
}

#[test]
fn grid_percent_rows_resolve_against_known_layout_height() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2, 3, 4, 5]);
    for node in 2..=5 {
        tree.children.insert(node, vec![]);
        tree.styles.insert(node, NodeInput::default());
    }
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::px(20.0), TrackComponent::percent(0.1)],
            grid_template_rows: vec![TrackComponent::percent(0.3), TrackComponent::percent(0.1)],
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::new(Some(20.0), Some(10.0)),
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 3.0));
    assert_eq!(tree.layouts[&3].size, Size::new(2.0, 3.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 3.0));
    assert_eq!(tree.layouts[&4].size, Size::new(20.0, 1.0));
    assert_eq!(tree.layouts[&5].location, Point::new(20.0, 3.0));
    assert_eq!(tree.layouts[&5].size, Size::new(2.0, 1.0));
}

#[test]
fn grid_defaults_to_implicit_auto_tracks_when_no_auto_tracks_are_authored() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            match input.run_mode {
                RunMode::ComputeSize => self.outputs[&node],
                RunMode::PerformRootLayout | RunMode::PerformLayout => {
                    ComputeOutput::from_outer_size(Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ))
                }
                RunMode::PerformHiddenLayout => ComputeOutput::HIDDEN,
            }
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(70.0, 18.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(70.0, 18.0));
    assert_eq!(output.content_size, Size::new(70.0, 18.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(70.0, 18.0));
}

#[test]
fn grid_spanning_item_distributes_intrinsic_contribution_across_auto_tracks() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            match input.run_mode {
                RunMode::ComputeSize => self.outputs[&node],
                RunMode::PerformRootLayout | RunMode::PerformLayout => {
                    ComputeOutput::from_outer_size(Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ))
                }
                RunMode::PerformHiddenLayout => ComputeOutput::HIDDEN,
            }
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO, Dimension::AUTO.into()],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(100.0, 20.0)));

    let output = surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let expected_columns = TrackSizingSlice::indefinite_columns(0.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 2, 1),
            min_content: 100.0,
            max_content: 100.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();
    let expected_width = expected_columns
        .final_tracks
        .iter()
        .map(|track| track.size)
        .sum::<f32>();

    assert_eq!(output.content_size, Size::new(expected_width, 20.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(expected_width, 20.0));
}

#[test]
fn grid_intrinsic_keyword_tracks_use_single_item_contribution() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            match input.run_mode {
                RunMode::ComputeSize => self.outputs[&node],
                RunMode::PerformRootLayout | RunMode::PerformLayout => {
                    ComputeOutput::from_outer_size(Size::new(
                        input.known.width.unwrap_or(0.0),
                        input.known.height.unwrap_or(0.0),
                    ))
                }
                RunMode::PerformHiddenLayout => ComputeOutput::HIDDEN,
            }
        }
    }

    fn run(track: Dimension) -> (ComputeOutput, NodeOutput) {
        let mut tree = GridTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::AUTO, Dimension::AUTO),
                grid_template_columns: vec![track.into()],
                grid_template_rows: vec![TrackComponent::AUTO],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(2, NodeInput::default());
        tree.outputs
            .insert(2, ComputeOutput::from_outer_size(Size::new(90.0, 22.0)));

        let output = surgeist_layout::compute_grid(
            &mut tree,
            1,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(Some(300.0), Some(200.0)),
                available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            },
        );

        (output, tree.layouts[&2])
    }

    for track in [Dimension::MIN_CONTENT, Dimension::MAX_CONTENT] {
        let (output, child_layout) = run(track);
        assert_eq!(output.content_size, Size::new(90.0, 22.0));
        assert_eq!(child_layout.location, Point::new(0.0, 0.0));
        assert_eq!(child_layout.size, Size::new(90.0, 22.0));
    }
}

#[test]
fn grid_align_items_center_offsets_smaller_child_within_grid_area() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(80.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(30.0, 10.0)));

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 15.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 10.0));
}

#[test]
fn grid_align_self_overrides_parent_align_items() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(80.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            align_self: Some(AlignItems::End),
            ..NodeInput::default()
        },
    );
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(30.0, 10.0)));

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 30.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 10.0));
}

#[test]
fn grid_aligns_items_to_shared_first_baseline() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(14.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 6.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_aligns_items_to_shared_last_baseline() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(4.0)))
        .measure(3, baseline_measure(30.0, 30.0, None, Some(10.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 14.0);
    assert_eq!(final_y(&tree, 3), 10.0);
}

#[test]
fn grid_reports_first_baseline_from_first_row_grid_order() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(120.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(7.0), None))
        .measure(3, baseline_measure(30.0, 20.0, Some(9.0), None));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.first_baselines.y, Some(7.0));
}

#[test]
fn grid_reports_last_baseline_from_last_row_grid_order() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(120.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(6.0)))
        .measure(3, baseline_measure(30.0, 30.0, None, Some(8.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.last_baselines.y, Some(62.0));
}

#[test]
fn grid_reports_first_baseline_from_shared_major_group_before_fallback_item() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(3, baseline_measure(30.0, 20.0, Some(14.0), None));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.first_baselines.y, Some(14.0));
}

#[test]
fn grid_reports_last_baseline_from_shared_minor_group_before_fallback_item() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(30.0)],
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(6.0)))
        .measure(3, baseline_measure(30.0, 20.0, None, Some(2.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.last_baselines.y, Some(64.0));
}

#[test]
fn grid_reports_last_baseline_from_spanning_item_that_occupies_last_row() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(6.0)))
        .measure(3, baseline_measure(30.0, 80.0, None, Some(8.0)));

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.last_baselines.y, Some(72.0));
}

#[test]
fn grid_aligns_first_baseline_with_block_margins() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    bottom: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(14.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 6.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_aligns_last_baseline_with_block_margins() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    bottom: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(4.0)))
        .measure(3, baseline_measure(30.0, 30.0, None, Some(10.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 14.0);
    assert_eq!(final_y(&tree, 3), 10.0);
}

#[test]
fn grid_aligns_first_baseline_for_item_spanning_rows_with_gap() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(120.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::ZERO, Length::px(7.0)),
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(8.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(14.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 6.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_aligns_last_baseline_for_item_spanning_rows_with_gap() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(120.0)),
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::ZERO, Length::px(7.0)),
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(4.0)))
        .measure(3, baseline_measure(30.0, 30.0, None, Some(10.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_y(&tree, 2), 61.0);
    assert_eq!(final_y(&tree, 3), 57.0);
}

#[test]
fn grid_baseline_increases_auto_row_size() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(6.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 42.0);
    assert_eq!(final_y(&tree, 3), 12.0);
}

#[test]
fn grid_last_baseline_increases_auto_row_size() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::LastBaseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, None, Some(2.0)))
        .measure(3, baseline_measure(30.0, 25.0, None, Some(12.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 30.0);
    assert_eq!(final_y(&tree, 3), 5.0);
}

#[test]
fn grid_absolute_baseline_child_does_not_affect_row_baseline_shim() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(6.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 20.0);
}

#[test]
fn grid_auto_block_margin_baseline_child_does_not_affect_row_baseline_shim() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                margin: Edges {
                    top: LengthAuto::Auto,
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(6.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 30.0);
}

#[test]
fn grid_baseline_less_child_spanning_intrinsic_row_uses_fallback_without_shim() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, ComputeOutput::from_outer_size(Size::new(30.0, 10.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 20.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_fixed_row_baseline_seeds_spanning_auto_row_shim() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, baseline_measure(30.0, 30.0, Some(6.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 42.0);
    assert_eq!(final_y(&tree, 3), 12.0);
}

#[test]
fn grid_constrained_row_baseline_sizing_uses_layout_mode() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure_when(
            3,
            OracleMeasurement::new(baseline_measure(30.0, 30.0, Some(6.0), None))
                .run_mode(RunMode::PerformLayout),
        );

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 42.0);
    assert_eq!(final_y(&tree, 3), 12.0);
}

#[test]
fn grid_baseline_less_child_in_fixed_row_does_not_grow_intrinsic_sizing() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                align_items: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(2, baseline_measure(30.0, 20.0, Some(18.0), None))
        .measure(3, ComputeOutput::from_outer_size(Size::new(30.0, 30.0)));

    compute_oracle_grid(&mut tree);

    assert_eq!(final_height(&tree, 1), 20.0);
    assert_eq!(final_y(&tree, 2), 12.0);
    assert_eq!(final_y(&tree, 3), 0.0);
}

#[test]
fn grid_justify_items_center_offsets_smaller_child_within_grid_area() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(80.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            justify_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(30.0, 10.0)));

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(25.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 10.0));
}

#[test]
fn grid_child_calc_size_and_margin_resolve_against_grid_area() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
        calcs: LayoutCalcStore,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(10.0),
            ))
        }

        fn calc_resolver(&self) -> &dyn CalcResolver {
            &self.calcs
        }
    }

    let mut tree = GridTree::default();
    let width = tree.calcs.push(CalcExpression::sum([
        CalcTerm::px(10.0),
        CalcTerm::percent(0.5),
    ]));
    let margin = tree.calcs.push(CalcExpression::sum([
        CalcTerm::px(5.0),
        CalcTerm::percent(0.1),
    ]));
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::calc(width), Dimension::px(10.0)),
            margin: Edges {
                left: LengthAuto::calc(margin),
                right: LengthAuto::ZERO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::Definite(100.0), Available::Definite(40.0)),
        },
    );

    assert_eq!(
        tree.inputs[&2].last().map(|input| input.known),
        Some(Size::new(Some(60.0), Some(10.0)))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(15.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(60.0, 10.0));
}

#[test]
fn grid_safe_justify_self_falls_back_to_start_when_item_overflows() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(100.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(150.0), Dimension::px(50.0)),
            justify_self: Some(AlignItems::SafeCenter),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(150.0, 50.0));
}

#[test]
fn grid_justify_self_overrides_parent_justify_items() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for GridTree {
        type Node = u32;
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.outputs.get(&node).copied().unwrap_or_else(|| {
                ComputeOutput::from_outer_size(Size::new(
                    input.known.width.unwrap_or(0.0),
                    input.known.height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(80.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            justify_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            justify_self: Some(AlignItems::End),
            ..NodeInput::default()
        },
    );
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(30.0, 10.0)));

    surgeist_layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(50.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 10.0));
}

#[test]
fn named_grid_column_places_item_between_repeated_named_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "a".to_string(),
                        index: 2,
                    },
                    RawGridLine::NamedSpan {
                        name: "a".to_string(),
                        index: 1,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 40.0);
    assert_eq!(child.size.width, 40.0);
}

#[test]
fn named_grid_spanning_item_counts_resolved_lines_for_auto_track_growth() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["b"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("a".to_string()),
                    RawGridLine::BareIdent("b".to_string()),
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let spanning = tree
        .final_layout(2)
        .expect("spanning child should be laid out");
    let auto = tree.final_layout(3).expect("auto child should be laid out");

    assert_eq!(spanning.location, Point::new(0.0, 0.0));
    assert_eq!(spanning.size, Size::new(80.0, 20.0));
    assert_eq!(auto.location, Point::new(0.0, 20.0));
    assert_eq!(auto.size, Size::new(40.0, 20.0));
}

#[test]
fn named_grid_template_area_bare_name_uses_generated_start_and_end_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            Some("foo".to_string()),
                            Some("foo".to_string()),
                            Some("bar".to_string()),
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("foo".to_string()),
                    RawGridLine::BareIdent("foo".to_string()),
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn named_grid_invalid_template_areas_keep_explicit_line_names() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["foo"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![
                        GridTemplateAreaRow {
                            cells: vec![Some("bad".to_string()), Some("bad".to_string())],
                        },
                        GridTemplateAreaRow {
                            cells: vec![Some("bad".to_string()), None],
                        },
                    ],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "foo".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 40.0);
    assert_eq!(child.size.width, 40.0);
}

#[test]
fn named_grid_bare_ident_is_distinct_from_explicit_named_line() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["foo-start"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["foo", "foo-end"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("foo".to_string()),
                    RawGridLine::BareIdent("foo".to_string()),
                ),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "foo".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let bare = tree.final_layout(2).expect("bare child should be laid out");
    let explicit = tree
        .final_layout(3)
        .expect("explicit child should be laid out");

    assert_eq!(bare.location.x, 0.0);
    assert_eq!(bare.size.width, 40.0);
    assert_eq!(explicit.location.x, 40.0);
    assert_eq!(explicit.size.width, 40.0);
}

#[test]
fn named_grid_negative_occurrence_and_missing_occurrence_extend_tracks() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(160.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_columns: vec![TrackComponent::px(40.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "a".to_string(),
                        index: -1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "a".to_string(),
                        index: 4,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let negative = tree
        .final_layout(2)
        .expect("negative occurrence child should be laid out");
    let missing = tree
        .final_layout(3)
        .expect("missing occurrence child should be laid out");

    assert_eq!(negative.location.x, 80.0);
    assert_eq!(negative.size.width, 40.0);
    assert_eq!(missing.location.x, 120.0);
    assert_eq!(missing.size.width, 40.0);
}

#[test]
fn named_grid_lone_named_span_auto_defaults_to_one_track_auto_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(80.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedSpan {
                        name: "a".to_string(),
                        index: 2,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 40.0);
}

#[test]
fn named_grid_start_after_end_and_equal_lines_normalize() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(40.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::lines(3, 1),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::lines(2, 2),
                raw_grid_row: RawGridPlacement::line(2),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let swapped = tree
        .final_layout(2)
        .expect("swapped child should be laid out");
    let equal = tree
        .final_layout(3)
        .expect("equal child should be laid out");

    assert_eq!(swapped.location.x, 0.0);
    assert_eq!(swapped.size.width, 80.0);
    assert_eq!(equal.location.x, 40.0);
    assert_eq!(equal.size.width, 40.0);
}

#[test]
fn subgrid_intrinsic_row_sizing_uses_subgrid_content_not_parent_height() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4, 5, 6, 7])
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::MinContent, Dimension::AUTO),
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                grid_row: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(25.0), Dimension::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            5,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(100.0), Dimension::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(50.0), Dimension::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            7,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(75.0), Dimension::px(25.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 1);
    let child = tree.final_layout(2).expect("child grid should be laid out");
    let subgrid = tree.final_layout(3).expect("subgrid should be laid out");

    assert_eq!(child.size, Size::new(100.0, 100.0));
    assert_eq!(subgrid.size, Size::new(100.0, 100.0));
}
