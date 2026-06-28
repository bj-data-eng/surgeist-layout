use std::collections::HashMap;

use super::lanes::*;
use super::tracks::*;
use super::*;
use crate::test_support::{
    self as lts,
    layout_tree::{OracleMeasurement, OracleTree},
};
use crate::*;
use lts::oracle::grid::{
    AlignmentSafety, AutoPlacer, ContributionSize, DefiniteTracks, Flow,
    GridArea as OracleGridArea, GridTrack, ItemContributionFacts, LinePlacement, Track,
    TrackAlignment, TrackSizingSlice, align_tracks_report,
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
        crate::Baselines {
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
    crate::compute_grid(
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

#[test]
fn lane_intrinsic_item_exposes_exactly_one_kind() {
    let contribution = LaneContributionFacts {
        min_content: 1.0,
        max_content: 2.0,
        min_size: 0.0,
        automatic_minimum_applies: true,
    };
    let span = LaneTrackSpanLength::new(2).expect("span should be nonzero");
    let item = LaneIntrinsicItem::indefinite("item", span, contribution);

    assert!(matches!(
        item.kind(),
        LaneIntrinsicItemKind::Indefinite { span } if span.get() == 2
    ));
}

#[test]
fn lane_intrinsic_item_rejects_malformed_definite_span_without_track_context() {
    let contribution = LaneContributionFacts {
        min_content: 1.0,
        max_content: 2.0,
        min_size: 0.0,
        automatic_minimum_applies: true,
    };
    let span = LaneTrackSpan::new(0, 1);
    let err = LaneIntrinsicItem::definite("item", span, contribution)
        .expect_err("malformed definite span should be rejected at construction");

    assert_eq!(err, LanePlacementError::InvalidDefiniteLaneSpan { span });
}

#[test]
fn lane_track_span_length_rejects_zero() {
    assert!(LaneTrackSpanLength::new(0).is_none());
}

#[test]
fn lane_errors_carry_context() {
    let err = place_lanes(LanePlacementInput {
        grid_axis_tracks: 2,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: GridFlowTolerance::Infinite,
        tolerance_basis: 0.0,
        items: vec![LaneItem {
            item: "item",
            grid_axis_span: 1,
            definite_grid_axis_start: Some(0),
            lane_axis_margin_box: 1.0,
        }],
    })
    .expect_err("zero grid axis start should be rejected with context");

    assert_eq!(err, LanePlacementError::InvalidGridAxisStart { start: 0 });
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
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
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
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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
    let oracle_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
        3,
        vec![
            Vec::<&str>::new(),
            vec!["slot-start"],
            vec![],
            vec!["slot-end"],
        ],
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_axis_placement(
        &oracle_lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "slot-start".to_string(),
                occurrence: 1,
            },
            end: lts::oracle::grid::NamedGridLine::Named {
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
    let oracle_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
        1,
        vec![vec!["a"], vec!["a"]],
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_axis_placement(
        &oracle_lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "a".to_string(),
                occurrence: 2,
            },
            end: lts::oracle::grid::NamedGridLine::Span {
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
    let parent_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
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
    let subgrid = lts::oracle::grid::inherit_named_subgrid_lines(
        &parent_lines,
        lts::oracle::grid::TrackSpan::new(2, 5),
        false,
        vec![Vec::<String>::new(), Vec::new(), Vec::new(), Vec::new()],
        None,
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_subgrid_axis_placement(
        &subgrid.lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "b".to_string(),
                occurrence: 1,
            },
            end: lts::oracle::grid::NamedGridLine::Named {
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
                TrackComponent::Subgrid(crate::SubgridTrack {
                    name_components: vec![crate::SubgridLineNameComponent::LineNames(vec![
                        "main".to_string(),
                    ])],
                }),
                TrackComponent::Repeat(
                    crate::TrackRepetition::auto_fit(vec![crate::TrackSizing::px(10.0)])
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

    let output = crate::compute_grid(
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
    TrackComponent::Subgrid(crate::SubgridTrack {
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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
                grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack::new(
                    vec![
                        vec!["local-start".to_string()],
                        vec![],
                        vec!["middle".to_string()],
                        vec!["local-end".to_string()],
                    ],
                ))],
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

    let output = crate::compute_grid(
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
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
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

    crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    let output = crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    let output = crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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
        .occupied(OracleGridArea::new(2, 1, 1, 1));
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

        crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    let output = crate::compute_grid(
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
            area: OracleGridArea::new(1, 1, 1, 1),
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
            area: OracleGridArea::new(1, 1, 1, 1),
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
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

    let output = crate::compute_grid(
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
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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
                TrackComponent::Track(crate::TrackSizing {
                    min: MinTrackSizing::Auto,
                    max: MaxTrackSizing::MaxContent,
                }),
                TrackComponent::Track(crate::TrackSizing {
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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
                TrackComponent::Track(crate::TrackSizing::fit_content(Length::px(20.0))),
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

    let output = crate::compute_grid(
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
                TrackComponent::Track(crate::TrackSizing::fit_content(Length::px(20.0))),
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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

    let output = crate::compute_grid(
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
            area: OracleGridArea::new(1, 1, 2, 1),
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

        let output = crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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

    crate::compute_grid(
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
fn invalid_named_grid_context_is_reported() {
    let mut tree = OracleTree::new().children(1, []).style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
            grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
            grid_template_areas: GridTemplateAreas {
                rows: vec![
                    GridTemplateAreaRow {
                        cells: vec![Some("bad".to_string()), Some("bad".to_string())],
                    },
                    GridTemplateAreaRow {
                        cells: vec![Some("bad".to_string())],
                    },
                ],
            },
            ..NodeInput::DEFAULT
        },
    );

    let result = crate::compute_grid_with_report(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(120.0), Some(20.0)),
            available: Size::new(Available::Definite(120.0), Available::Definite(20.0)),
        },
    );

    assert!(result.report().named_grid_errors().contains(
        &NamedGridErrorReport::TemplateAreaRowLengthMismatch {
            row: 2,
            expected: 2,
            actual: 1,
        },
    ));
}

#[test]
fn invalid_named_grid_context_fallback_is_reported() {
    let mut tree = OracleTree::new().children(1, []).style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::line_names(["auto"]),
                TrackComponent::px(40.0),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::DEFAULT
        },
    );

    let result = crate::compute_grid_with_report(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(40.0), Some(20.0)),
            available: Size::new(Available::Definite(40.0), Available::Definite(20.0)),
        },
    );

    assert!(result.report().named_grid_errors().contains(
        &NamedGridErrorReport::ReservedLineName {
            name: "auto".to_string(),
        },
    ));
}

#[test]
fn invalid_grid_item_placement_reports_one_authored_fallback_once() {
    let mut tree = OracleTree::new()
        .children(1, [2])
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
                raw_grid_column: RawGridPlacement::new(RawGridLine::Line(0), RawGridLine::Auto),
                ..NodeInput::DEFAULT
            },
        );

    let result = crate::compute_grid_with_report(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(40.0), Some(20.0)),
            available: Size::new(Available::Definite(40.0), Available::Definite(20.0)),
        },
    );

    let zero_line_count = result
        .report()
        .named_grid_errors()
        .iter()
        .filter(|error| **error == NamedGridErrorReport::ZeroLine)
        .count();

    assert_eq!(zero_line_count, 1);
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

#[test]
fn lane_axis_margin_box_measurement_resolves_calc_margins_against_grid_axis() {
    let mut store = LayoutCalcStore::new();
    let margin = store.push(CalcExpression::sum([
        CalcTerm::px(4.0),
        CalcTerm::percent(0.10),
    ]));
    let child_style = NodeInput {
        margin: Edges {
            left: LengthAuto::calc(margin),
            right: LengthAuto::px(6.0),
            top: LengthAuto::ZERO,
            bottom: LengthAuto::ZERO,
        },
        ..NodeInput::default()
    };
    let container_style = NodeInput::default();
    let constants = Constants {
        node_outer_size: Size::new(Some(200.0), Some(80.0)),
        node_inner_size: Size::new(Some(200.0), Some(80.0)),
        node_min_size: Size::NONE,
        node_max_size: Size::NONE,
        available_inner_size: Size::new(Some(200.0), Some(80.0)),
        content_box_inset: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let mut tree = LaneMarginMeasureTree {
        child_style: child_style.clone(),
        resolver: store,
        child_output: ComputeOutput::from_sizes_and_baselines(
            Size::new(50.0, 12.0),
            Size::new(50.0, 12.0),
            Baselines::NONE,
        ),
        last_input: None,
    };

    let measured = measure_lane_axis_margin_box_with_grid_axis(
        &mut tree,
        LaneMarginMeasureTree::CHILD,
        LaneAxisMarginBoxMeasureInput {
            child_style: &child_style,
            container_style: &container_style,
            constants: &constants,
            lane_axis: GridAxisKind::Column,
            grid_axis: GridAxisKind::Column,
            grid_axis_size: 200.0,
        },
    );

    assert_eq!(measured, 80.0);
    let input = tree
        .last_input
        .expect("measurement should compute the child");
    assert_eq!(input.known.width, Some(170.0));
    assert_eq!(input.parent.width, Some(200.0));
    assert_eq!(input.available.width, Available::Definite(170.0));
}

struct LaneMarginMeasureTree {
    child_style: NodeInput,
    resolver: LayoutCalcStore,
    child_output: ComputeOutput,
    last_input: Option<ComputeInput>,
}

impl LaneMarginMeasureTree {
    const ROOT: usize = 0;
    const CHILD: usize = 1;
}

impl Traverse for LaneMarginMeasureTree {
    type Node = usize;
    type Scalar = Scalar;
    type Children<'a> = std::vec::IntoIter<Self::Node>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        match node {
            Self::ROOT => vec![Self::CHILD].into_iter(),
            _ => Vec::new().into_iter(),
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == Self::ROOT)
    }

    fn child(&self, _node: Self::Node, index: usize) -> Self::Node {
        assert_eq!(index, 0);
        Self::CHILD
    }
}

impl Compute for LaneMarginMeasureTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        assert_eq!(node, Self::CHILD);
        &self.child_style
    }

    fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {
        unreachable!("lane margin measurement should not write layout output");
    }

    fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
        assert_eq!(node, Self::CHILD);
        self.last_input = Some(input);
        self.child_output
    }

    fn calc_resolver(&self) -> &dyn CalcResolver {
        &self.resolver
    }
}

#[test]
fn resolve_inline_tracks_accepts_f64_track_inputs() {
    let tracks = [
        TrackSizingOf::<f64>::px(10.25),
        TrackSizingOf::<f64>::AUTO,
        TrackSizingOf::<f64>::fr(0.5),
    ];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: Some(90.75_f64),
        definite_size: Some(90.75_f64),
        available_size: Some(90.75_f64),
        gap: 0.25_f64,
        alignment: AlignContent::Stretch,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[1.5_f64, 2.5_f64, 3.5_f64],
        max_intrinsic_sizes: &[4.5_f64, 5.5_f64, 6.5_f64],
    });

    assert_eq!(sizes, vec![10.25_f64, 42.75_f64, 37.25_f64]);
}

#[test]
fn auto_repeat_count_uses_f64_saturating_floor() {
    let tracks = [TrackSizingOf::<f64>::px(10.0)];
    let reserved = ReservedTrackSpace::<f64> {
        count: 1,
        size: 10.25,
    };

    let count = auto_repeat_count(&tracks, Some(43.0_f64), 0.25_f64, reserved, &NoCalcResolver);

    assert_eq!(count, 3);
}

#[test]
fn distribute_intrinsic_span_preserves_f64_fractional_shares() {
    let mut sizes = vec![1.25_f64, 3.75_f64];
    let tracks = [TrackSizingOf::<f64>::AUTO, TrackSizingOf::<f64>::AUTO];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        12.0_f64,
        &NoCalcResolver,
    );

    assert_eq!(sizes, vec![6.0_f64, 6.0_f64]);
}

#[test]
fn px_only_calc_max_track_does_not_force_max_intrinsic_resolution() {
    let mut resolver = LayoutCalcStore::new();
    let calc = resolver.push(CalcExpression::sum([CalcTerm::px(24.0)]));
    let tracks = [TrackSizing::new(
        MinTrackSizing::MinContent,
        MaxTrackSizing::Length(Length::calc(calc)),
    )];

    let sizes = track_resolution_intrinsic_sizes(&tracks, &[11.0], &[99.0], &resolver);

    assert_eq!(sizes, vec![11.0]);
}

#[test]
fn basis_dependent_calc_max_track_uses_max_intrinsic_resolution() {
    let mut resolver = LayoutCalcStore::new();
    let calc = resolver.push(CalcExpression::sum([CalcTerm::percent(0.5)]));
    let tracks = [TrackSizing::new(
        MinTrackSizing::MinContent,
        MaxTrackSizing::Length(Length::calc(calc)),
    )];

    let sizes = track_resolution_intrinsic_sizes(&tracks, &[11.0], &[99.0], &resolver);

    assert_eq!(sizes, vec![99.0]);
}

fn subgrid_track() -> Vec<TrackComponent> {
    subgrid_track_of()
}

fn subgrid_track_of<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::Subgrid(SubgridTrack {
        name_components: Vec::new(),
    })]
}

#[test]
fn lane_intrinsic_public_inputs_accept_non_default_scalar() {
    let facts = LaneContributionFactsOf::<f64> {
        min_content: 1.25_f64,
        max_content: 2.5_f64,
        min_size: 0.75_f64,
        automatic_minimum_applies: true,
    };
    let item = LaneIntrinsicItemOf::<f64>::indefinite(
        "wide",
        LaneTrackSpanLength::new(2).expect("span should be nonzero"),
        facts,
    );
    let input = LaneIntrinsicSizingInputOf::<f64> {
        axis: GridAxisKind::Column,
        available: Some(10.5_f64),
        gap: 1.5_f64,
        tracks: vec![TrackSizingOf::<f64>::AUTO],
        content_sized_tracks: vec![0],
        items: vec![item],
    };

    assert_eq!(input.gap, 1.5_f64);
    assert_eq!(input.items[0].contribution().max_content, 2.5_f64);

    let placement_input = LanePlacementInputOf::<_, f64> {
        grid_axis_tracks: 1,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 1.5_f64,
        tolerance: GridFlowToleranceOf::Percent(0.25_f64),
        tolerance_basis: 10.5_f64,
        items: Vec::<LaneItemOf<&str, f64>>::new(),
    };

    assert_eq!(
        placement_input.tolerance,
        GridFlowToleranceOf::Percent(0.25_f64)
    );
}

#[test]
fn lane_public_helpers_compute_with_non_default_scalar() {
    let placement = place_lanes(LanePlacementInputOf::<_, f64> {
        grid_axis_tracks: 2,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 0.5,
        tolerance: GridFlowToleranceOf::Normal { font_size: 0.0 },
        tolerance_basis: 0.0,
        items: vec![
            LaneItemOf {
                item: "a",
                grid_axis_span: 1,
                definite_grid_axis_start: None,
                lane_axis_margin_box: 10.25,
            },
            LaneItemOf {
                item: "b",
                grid_axis_span: 1,
                definite_grid_axis_start: None,
                lane_axis_margin_box: 12.5,
            },
        ],
    })
    .expect("f64 lane placement should compute");

    assert_eq!(placement.content_size, 12.5);
    assert_eq!(placement.item_offsets[1].offset, 0.0);

    let intrinsic = lane_intrinsic_sizing(LaneIntrinsicSizingInputOf::<f64> {
        axis: GridAxisKind::Column,
        available: Some(80.0),
        gap: 1.25,
        tracks: vec![TrackSizingOf::<f64>::AUTO],
        content_sized_tracks: vec![0],
        items: vec![
            LaneIntrinsicItemOf::<f64>::definite(
                "definite",
                LaneTrackSpan::new(1, 2),
                LaneContributionFactsOf {
                    min_content: 9.5,
                    max_content: 14.25,
                    min_size: 7.0,
                    automatic_minimum_applies: true,
                },
            )
            .expect("span is valid"),
        ],
    })
    .expect("f64 lane intrinsic sizing should compute");

    assert_eq!(intrinsic.final_track_sizes, vec![9.5]);
}

#[test]
fn grid_lanes_compute_result_accepts_non_default_scalar() {
    #[derive(Clone)]
    struct F64GridTree {
        styles: Vec<NodeInputOf<f64>>,
        children: Vec<Vec<usize>>,
        layouts: Vec<NodeOutputOf<f64>>,
    }

    impl Traverse for F64GridTree {
        type Node = usize;
        type Scalar = f64;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[node][index]
        }
    }

    impl Compute for F64GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInputOf<f64> {
            &self.styles[node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<f64>) {
            self.layouts[node] = layout;
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInputOf<f64>,
        ) -> ComputeOutputOf<f64> {
            let style = &self.styles[node];
            let size = input.known.unwrap_or(Size::new(
                style
                    .size
                    .width
                    .resolve_with(input.parent.width, &NoCalcResolver)
                    .or_else(|| input.available.width.into_option())
                    .unwrap_or(0.0),
                style
                    .size
                    .height
                    .resolve_with(input.parent.height, &NoCalcResolver)
                    .or_else(|| input.available.height.into_option())
                    .unwrap_or(0.0),
            ));
            ComputeOutputOf::from_sizes(size, size)
        }
    }

    let root_style = NodeInputOf::<f64> {
        display: Display::GridLanes,
        size: Size::new(DimensionOf::px(120.0), DimensionOf::px(90.0)),
        grid_template_columns: vec![TrackSizingOf::px(60.0).into()],
        grid_auto_rows: vec![TrackSizingOf::px(40.0).into()],
        ..NodeInputOf::default()
    };
    let child_style = NodeInputOf::<f64> {
        size: Size::new(DimensionOf::px(30.0), DimensionOf::px(20.0)),
        ..NodeInputOf::default()
    };
    let mut tree = F64GridTree {
        styles: vec![root_style, child_style],
        children: vec![vec![1], Vec::new()],
        layouts: vec![NodeOutputOf::new(), NodeOutputOf::new()],
    };

    let computation = compute_grid_with_report(
        &mut tree,
        0,
        ComputeInputOf {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(AvailableOf::MAX_CONTENT),
        },
    );
    let (output, report) = computation.into_parts();

    assert!(report.is_empty());
    assert_eq!(output.size, Size::new(120.0, 90.0));
    assert_eq!(tree.layouts[1].size, Size::new(30.0, 20.0));
}

#[test]
fn shared_grid_contexts_accept_non_default_scalar() {
    let named_lines = named::NamedGridLines::new(GridAxisKind::Column, 1);
    let inherited_axis = InheritedGridAxis::<f64> {
        offset: 0.25,
        gap: 1.5,
        tracks: vec![10.0, 20.0],
        named_lines: named_lines.clone(),
        area_facts: None,
        major_baselines: vec![Some(2.0)],
        minor_baselines: vec![None],
        parent_start: 0,
        parent_end: 2,
        reversed: false,
        start_mbp: 0.5,
        end_mbp: 0.75,
        gap_difference: 0.25,
    };
    let parent_context = GridParentContext::<f64> {
        columns: Some(inherited_axis),
        rows: None,
    };
    assert!(parent_context.has_inherited_axis());

    let lines = GridLines {
        column_explicit_start: 0,
        column_explicit_count: 1,
        row_explicit_start: 0,
        row_explicit_count: 1,
    };
    let container_context = GridContainerContext::<f64> {
        gap: Size::new(1.0, 2.0),
        column_basis: Some(100.0),
        row_basis: None,
        explicit_columns: 1,
        explicit_rows: 1,
        named_columns: named_lines.clone(),
        named_rows: named::NamedGridLines::new(GridAxisKind::Row, 1),
        area_facts: None,
        leading_columns: 0,
        leading_rows: 0,
        lines,
        inherited_column_offset: Some(0.25),
        inherited_row_offset: None,
    };
    let constants = Constants::<f64> {
        node_outer_size: Size::splat(Some(120.0)),
        node_inner_size: Size::splat(Some(100.0)),
        node_min_size: Size::NONE,
        node_max_size: Size::NONE,
        available_inner_size: Size::splat(Some(100.0)),
        content_box_inset: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let style = NodeInputOf::<f64>::default();
    let tracks = vec![TrackSizingOf::<f64>::AUTO];
    let placements = GridPlacementContext::new(Vec::<usize>::new(), Vec::new());
    let subgrid_report = GridSubgridReport { items: Vec::new() };

    let _initialized = InitializedGridTracks::<usize, f64> {
        column_tracks: tracks.clone(),
        row_tracks: tracks.clone(),
        context: container_context.clone(),
        placements: GridPlacementContext::new(Vec::new(), Vec::new()),
        subgrid_report: GridSubgridReport { items: Vec::new() },
        report: GridComputationReport::default(),
    };
    let _track_input = GridTrackResolutionInput::<usize, f64> {
        style: &style,
        constants: &constants,
        column_tracks: &tracks,
        row_tracks: &tracks,
        context: container_context.clone(),
        subgrid_report: &subgrid_report,
        available: Size::splat(AvailableOf::<f64>::MAX_CONTENT),
        intrinsic_max_available: Size::splat(false),
        placements: &placements,
    };
    let _track_resolution = GridTrackResolution::<f64> {
        columns: vec![10.0],
        rows: vec![20.0],
        column_min_intrinsic_sizes: vec![1.0],
        column_max_intrinsic_sizes: vec![2.0],
        row_intrinsic_sizes: vec![3.0],
    };
    let _child_input = GridChildLayoutInput::<usize, f64> {
        style: &style,
        constants: &constants,
        column_tracks: &tracks,
        row_tracks: &tracks,
        context: container_context.clone(),
        columns: &[10.0],
        rows: &[20.0],
        column_min_intrinsic_sizes: &[1.0],
        column_max_intrinsic_sizes: &[2.0],
        row_intrinsic_sizes: &[3.0],
        output_size: Size::new(100.0, 100.0),
        subgrid_report: &subgrid_report,
        parent_context: &parent_context,
        placements: &placements,
    };
    let _layout_context = GridLayoutContext::<usize, f64> {
        style: &style,
        constants: &constants,
        container_content_size: Size::new(100.0, 100.0),
        columns: &[10.0],
        rows: &[20.0],
        row_tracks: &tracks,
        gap: Size::new(1.0, 2.0),
        lines,
        named_columns: named_lines,
        named_rows: named::NamedGridLines::new(GridAxisKind::Row, 1),
        area_facts: None,
        inherited_column_offset: Some(0.25),
        inherited_row_offset: None,
        subgrid_report: &subgrid_report,
        parent_context: &parent_context,
        placements: &placements,
    };
}

#[test]
fn grid_child_pure_helpers_accept_non_default_scalar() {
    let geometry = BaselineGeometry::<f64> {
        available_span_size: 80.0,
        margin_box_size: 30.0,
        major_baseline: 12.5,
        minor_baseline: 7.25,
    };
    let shared = TrackBaselineGroup::<f64> {
        first: Some(20.0),
        last: Some(10.0),
    };

    assert_eq!(
        baseline_shim_for_intrinsic_contribution(
            BaselineParticipation {
                participates: true,
                group: Some(BaselineGroupKind::Major),
                synthesized: false,
                fallback_alignment: None,
            },
            geometry,
            shared,
        ),
        BaselineShim::<f64> {
            before: 7.5,
            after: 0.0,
        }
    );
    assert_eq!(
        baseline_offset(BaselineGroupKind::Minor, 10.0_f64, geometry),
        47.25
    );
    assert_eq!(spanned_track_size(&[10.0_f64, 20.0, 30.0], 0, 3, 2.5), 65.0);

    assert_eq!(
        grid_item_axis(GridItemAxis::<f64> {
            area_size: 100.0,
            size: 20.0,
            margin_start: None,
            margin_end: None,
            alignment: AlignItems::Center,
            direction: Direction::Ltr,
        }),
        ResolvedGridItemAxis::<f64> {
            offset: 40.0,
            margin_start: 40.0,
            margin_end: 40.0,
        }
    );

    assert_eq!(
        absolute_grid_axis(AbsoluteGridAxis::<f64> {
            area_location: 5.0,
            static_area_location: 10.0,
            area_size: 100.0,
            static_area_size: 80.0,
            size: 20.0,
            margin_start: Some(2.5),
            margin_end: Some(7.5),
            inset_start: None,
            inset_end: None,
            alignment: AlignItems::End,
            direction: Direction::Ltr,
        }),
        ResolvedAbsoluteGridAxis::<f64> {
            location: 62.5,
            margin_start: 2.5,
            margin_end: 7.5,
        }
    );
}

#[test]
fn grid_child_pending_and_subgrid_inheritance_helpers_accept_non_default_scalar() {
    let area = GridArea::<f64> {
        column: 0,
        row: 0,
        column_end: 1,
        row_end: 2,
        size: Size::new(40.0, 90.0),
    };
    let item = PendingGridItem::<_, f64> {
        node: "child",
        order: 0,
        area,
        output: ComputeOutputOf::<f64>::from_sizes_and_baselines(
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
            BaselinesOf {
                first: Point::new(None, Some(8.0)),
                last: Point::new(None, Some(22.0)),
            },
        ),
        horizontal_axis: ResolvedGridItemAxis::<f64> {
            offset: 0.0,
            margin_start: 0.0,
            margin_end: 0.0,
        },
        vertical_axis: ResolvedGridItemAxis::<f64> {
            offset: 0.0,
            margin_start: 3.0,
            margin_end: 5.0,
        },
        relative_offset: Point::<f64>::ZERO,
        first_baseline: 8.0,
        last_baseline: 22.0,
        published_row_baselines: None,
        block_offset: 0.0,
        block_auto_margins: false,
        baseline_participation: BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Major),
            synthesized: false,
            fallback_alignment: None,
        },
        margin: Edges::new(0.0, 0.0, 3.0, 5.0),
        scrollbar_size: Size::ZERO,
        border: Edges::ZERO,
        padding: Edges::ZERO,
        overflow: Point::new(Overflow::Visible, Overflow::Visible),
    };

    let groups = baseline_groups(std::slice::from_ref(&item), 2, 1);
    assert_eq!(groups.rows[0].first, Some(11.0));
    assert_eq!(
        baseline_aligned_block_offset(&item, &groups, &[40.0_f64, 40.0], 10.0),
        Some(3.0)
    );

    let axis = InheritedGridAxis::<f64> {
        offset: 0.0,
        gap: 10.0,
        tracks: vec![40.0, 40.0],
        named_lines: named::NamedGridLines::new(GridAxisKind::Row, 2),
        area_facts: None,
        major_baselines: vec![None, None],
        minor_baselines: vec![None, None],
        parent_start: 0,
        parent_end: 2,
        reversed: false,
        start_mbp: 1.5,
        end_mbp: 2.5,
        gap_difference: 0.25,
    };
    let published = publish_row_baseline_groups(&groups.rows, &axis);
    assert_eq!(
        published,
        vec![PublishedTrackBaselineGroup::<f64> {
            parent_index: 0,
            group: TrackBaselineGroup {
                first: Some(12.75),
                last: None,
            },
        }]
    );

    let inherited = inherit_subgrid_tracks(SubgridTrackInheritanceInput::<f64> {
        parent_tracks: &[20.0, 30.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 2.0,
        end_mbp: 4.0,
        parent_gap: 6.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();
    assert_eq!(inherited.gap_difference, 2.0);
    assert_eq!(inherited.final_tracks, vec![16.0, 24.0]);

    let inherited_baselines = inherit_subgrid_baselines(SubgridBaselineInheritanceInput::<f64> {
        parent_major: &[Some(9.0), Some(17.0)],
        parent_minor: &[Some(4.0), Some(6.0)],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 2.0,
        end_mbp: 4.0,
        parent_gap: 6.0,
        subgrid_gap: inherited.resolved_subgrid_gap,
    })
    .unwrap();
    assert_eq!(inherited_baselines.gap_difference, 2.0);
    assert_eq!(inherited_baselines.final_major, vec![Some(5.0), Some(15.0)]);
    assert_eq!(inherited_baselines.final_minor, vec![Some(2.0), Some(0.0)]);

    let (layout_tracks, layout_gap) =
        inherited_subgrid_layout_tracks(GridAxisKind::Row, &inherited);
    assert_eq!(layout_tracks, vec![16.0, 24.0]);
    assert_eq!(layout_gap, 10.0);

    let offset_style = NodeInputOf::<f64>::default();
    let offsets = grid_axis_offsets(GridAxisOffsetsInput::<f64> {
        style: &offset_style,
        axis: GridAxisKind::Column,
        tracks: &[12.5, 17.5],
        inherited_offset: Some(1.25),
        content_box_left: 0.0,
        content_box_size: Size::new(60.0, 20.0),
        content_box_inset: Edges::new(0.0, 0.0, 0.0, 2.0),
        alignment: GridAlignment {
            start: 0.5,
            gap: 3.25,
        },
    });
    assert_eq!(offsets, vec![3.75, 19.5]);

    let child_style = NodeInputOf::<f64> {
        display: Display::Grid,
        grid_template_rows: subgrid_track_of(),
        ..NodeInputOf::default()
    };
    let parent_context = subgrid_child_parent_context(SubgridChildParentContextInput::<_, f64> {
        item: SubgridItemReport {
            node: "child",
            column: SubgridAxisReport {
                mapping: Ok(GridAxisMappingReport {
                    queried_axis: GridAxisKind::Column,
                    parent_axis: GridAxisKind::Column,
                    child_axis: GridAxisKind::Column,
                    reversed: false,
                }),
                eligibility: SubgridEligibility {
                    eligible: false,
                    reason: Some(SubgridIneligibleReason::NotRequested),
                },
            },
            row: SubgridAxisReport {
                mapping: Ok(GridAxisMappingReport {
                    queried_axis: GridAxisKind::Row,
                    parent_axis: GridAxisKind::Row,
                    child_axis: GridAxisKind::Row,
                    reversed: false,
                }),
                eligibility: SubgridEligibility {
                    eligible: true,
                    reason: None,
                },
            },
        },
        child_style: &child_style,
        area,
        content_box_size: Size::new(40.0, 90.0),
        columns: &[40.0],
        rows: &[20.0, 30.0],
        gap: Size::new(0.0, 6.0),
        parent_named_columns: &named::NamedGridLines::new(GridAxisKind::Column, 1),
        parent_named_rows: &named::NamedGridLines::new(GridAxisKind::Row, 2),
        parent_area_facts: None,
        parent_baseline_groups: &GridBaselineGroups::<f64> {
            rows: groups.rows,
            columns: vec![TrackBaselineGroup::default()],
        },
        margin: Edges::ZERO.map(Some),
        border: Edges::ZERO,
        padding: Edges::ZERO,
        resolver: &NoCalcResolver,
    });

    let rows = parent_context.rows.expect("row subgrid should inherit");
    assert_eq!(rows.tracks, vec![20.0, 30.0]);
    assert_eq!(rows.gap, 6.0);
}

#[test]
fn grid_alignment_accepts_f64_and_preserves_fractional_distribution() {
    let alignment = grid_alignment::<f64>(9_000_000.75_f64, 3, 0.25_f64, AlignContent::SpaceAround);

    assert_eq!(alignment.start, 1_500_000.125_f64);
    assert_eq!(alignment.gap, 3_000_000.5_f64);
}

#[test]
fn public_grid_placement_rejects_zero_line_and_span() {
    assert_eq!(GridLine::new(0), None);
    assert_eq!(GridSpan::new(0), None);
    assert!(GridLine::new(1).is_some());
    assert!(GridSpan::new(1).is_some());
    assert_eq!(GridPlacement::try_line(0), None);
    assert_eq!(GridPlacement::try_lines(0, 1), None);
    assert_eq!(GridPlacement::try_lines(1, 0), None);
    assert_eq!(GridPlacement::try_line_span(0, 1), None);
    assert_eq!(GridPlacement::try_line_span(1, 0), None);
    assert_eq!(GridPlacement::try_span_line(0, 1), None);
    assert_eq!(GridPlacement::try_span_line(1, 0), None);
    assert_eq!(GridPlacement::try_span(0), None);
}

#[test]
fn grid_placement_fields_are_constructed_through_validated_values() {
    let placement = GridPlacement::line_span(
        GridLine::new(2).expect("valid line"),
        GridSpan::new(3).expect("valid span"),
    );

    assert_eq!(placement.start(), Some(GridLine::new(2).unwrap()));
    assert_eq!(placement.span(), Some(GridSpan::new(3).unwrap()));
}

#[test]
fn named_lines_preserve_explicit_names_and_fixed_repeats() {
    let lines = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["a"]),
            TrackComponent::px(20.0),
            TrackComponent::line_names(["b", "a"]),
            TrackComponent::Repeat(
                TrackRepetition::count_components(
                    2,
                    vec![
                        TrackComponent::line_names(["c"]),
                        TrackComponent::px(10.0),
                        TrackComponent::line_names(["d"]),
                    ],
                )
                .expect("valid track repetition"),
            ),
        ],
        3,
    )
    .unwrap();

    assert_eq!(lines.named_occurrences("a"), vec![1, 2]);
    assert_eq!(lines.named_occurrences("b"), vec![2]);
    assert_eq!(lines.named_occurrences("c"), vec![2, 3]);
    assert_eq!(lines.named_occurrences("d"), vec![3, 4]);
}

#[test]
fn named_lines_preserve_duplicate_source_order_names() {
    let lines = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["a", "b", "a"]),
            TrackComponent::px(20.0),
        ],
        1,
    )
    .unwrap();

    assert_eq!(lines.named_occurrences("a"), vec![1, 1]);
    assert_eq!(
        lines
            .entries_on_line(1)
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b", "a"]
    );
}

#[test]
fn named_lines_reject_reserved_explicit_line_names() {
    let error = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["auto"]),
            TrackComponent::px(20.0),
        ],
        1,
    )
    .unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::ReservedLineName {
            name: "auto".to_string(),
        }
    );

    let repeat_error = named::named_lines_from_track_components(
        GridAxisKind::Row,
        &[TrackComponent::Repeat(
            TrackRepetition::count_components(
                2,
                vec![
                    TrackComponent::line_names(["span"]),
                    TrackComponent::px(10.0),
                ],
            )
            .expect("valid track repetition"),
        )],
        2,
    )
    .unwrap_err();

    assert_eq!(
        repeat_error,
        named::NamedGridError::ReservedLineName {
            name: "span".to_string(),
        }
    );
}

#[test]
fn named_lines_classify_unresolved_auto_repeat_names() {
    let error = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["before"]),
            TrackComponent::Repeat(
                TrackRepetition::auto_fit_components(vec![
                    TrackComponent::line_names(["inside"]),
                    TrackComponent::px(10.0),
                    TrackComponent::px(10.0),
                ])
                .expect("valid track repetition"),
            ),
            TrackComponent::line_names(["after"]),
        ],
        3,
    )
    .unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::UnresolvedAutoRepeatNames {
            axis: GridAxisKind::Column
        }
    );
}

#[test]
fn named_lines_validate_auto_repeat_names_before_unresolved_classification() {
    let error = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[TrackComponent::Repeat(
            TrackRepetition::auto_fit_components(vec![
                TrackComponent::line_names(["auto"]),
                TrackComponent::px(10.0),
                TrackComponent::px(10.0),
            ])
            .expect("valid track repetition"),
        )],
        3,
    )
    .unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::ReservedLineName {
            name: "auto".to_string(),
        }
    );
}

#[test]
fn named_lines_return_empty_local_map_for_subgrid() {
    let lines =
        named::named_lines_from_track_components(GridAxisKind::Row, &subgrid_track(), 2).unwrap();

    assert_eq!(lines.axis, GridAxisKind::Row);
    assert_eq!(lines.explicit_track_count, 2);
    assert!(lines.named_occurrences("anything").is_empty());
}

#[test]
fn named_lines_add_template_area_generated_names_and_facts() {
    let base = named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[TrackComponent::line_names(["explicit"])],
        0,
    )
    .unwrap();
    let areas = crate::GridTemplateAreas {
        rows: vec![
            crate::GridTemplateAreaRow {
                cells: vec![Some("head".to_string()), Some("head".to_string())],
            },
            crate::GridTemplateAreaRow {
                cells: vec![Some("nav".to_string()), Some("main".to_string())],
            },
        ],
    };

    let lines = named::add_area_generated_lines(GridAxisKind::Column, base, &areas).unwrap();

    assert_eq!(lines.explicit_track_count, 2);
    assert_eq!(lines.named_occurrences("explicit"), vec![1]);
    assert_eq!(lines.named_occurrences("head-start"), vec![1]);
    assert_eq!(lines.named_occurrences("head-end"), vec![3]);
    assert_eq!(lines.named_occurrences("nav-start"), vec![1]);
    assert_eq!(lines.named_occurrences("main-start"), vec![2]);
    assert_eq!(lines.area_facts.area_order, vec!["head", "nav", "main"]);
    assert_eq!(lines.area_facts.row_count, 2);
    assert_eq!(lines.area_facts.column_count, 2);
    assert!(lines.area_facts.rows_valid);
    assert!(lines.area_facts.columns_valid);
    assert_eq!(
        lines.area_facts.area_rectangles,
        vec![
            named::GridAreaNameRectangle {
                name: "head".to_string(),
                row_start: 1,
                row_end: 2,
                column_start: 1,
                column_end: 3,
                row_start_name: 1,
                row_end_name: 2,
                column_start_name: 1,
                column_end_name: 3,
            },
            named::GridAreaNameRectangle {
                name: "nav".to_string(),
                row_start: 2,
                row_end: 3,
                column_start: 1,
                column_end: 2,
                row_start_name: 2,
                row_end_name: 3,
                column_start_name: 1,
                column_end_name: 2,
            },
            named::GridAreaNameRectangle {
                name: "main".to_string(),
                row_start: 2,
                row_end: 3,
                column_start: 2,
                column_end: 3,
                row_start_name: 2,
                row_end_name: 3,
                column_start_name: 2,
                column_end_name: 3,
            },
        ]
    );
}

#[test]
fn named_lines_ignore_template_area_null_cells() {
    let base =
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Row, &[], 0).unwrap();
    let areas = crate::GridTemplateAreas {
        rows: vec![
            crate::GridTemplateAreaRow {
                cells: vec![None, Some("main".to_string())],
            },
            crate::GridTemplateAreaRow {
                cells: vec![None, Some("main".to_string())],
            },
        ],
    };

    let lines = named::add_area_generated_lines(GridAxisKind::Row, base, &areas).unwrap();

    assert_eq!(lines.named_occurrences("main-start"), vec![1]);
    assert_eq!(lines.named_occurrences("main-end"), vec![3]);
    assert!(lines.named_occurrences(".-start").is_empty());
}

#[test]
fn named_lines_reject_invalid_template_area_row_widths() {
    let areas = crate::GridTemplateAreas {
        rows: vec![
            crate::GridTemplateAreaRow {
                cells: vec![Some("a".to_string()), Some("a".to_string())],
            },
            crate::GridTemplateAreaRow {
                cells: vec![Some("a".to_string())],
            },
        ],
    };

    let error = named::GridAreaNameFacts::from_specified_areas(&areas).unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::TemplateAreaRowLengthMismatch {
            row: 2,
            expected: 2,
            actual: 1,
        }
    );
}

fn named_grid_lines() -> named::NamedGridLines {
    named::named_lines_from_track_components(
        GridAxisKind::Column,
        &[
            TrackComponent::line_names(["a", "foo-start"]),
            TrackComponent::px(20.0),
            TrackComponent::line_names(["a", "foo", "foo-end"]),
            TrackComponent::px(20.0),
            TrackComponent::line_names(["a"]),
        ],
        2,
    )
    .unwrap()
}

#[test]
fn named_grid_resolver_places_between_repeated_line_and_named_span() {
    let placement = named::resolve_grid_placement(
        &named_grid_lines(),
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 1,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(2, 3).expect("valid grid lines")
    );
}

#[test]
fn named_grid_resolver_uses_side_aware_bare_ident_before_plain_name() {
    let lines = named_grid_lines();

    let bare = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::BareIdent("foo".to_string()),
            RawGridLine::BareIdent("foo".to_string()),
        ),
        None,
    )
    .unwrap();
    let explicit = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "foo".to_string(),
                index: 1,
            },
            RawGridLine::NamedLine {
                name: "foo".to_string(),
                index: 1,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        bare,
        GridPlacement::try_lines(1, 2).expect("valid grid lines")
    );
    assert_eq!(
        explicit,
        GridPlacement::try_line_span(2, 1).expect("valid grid line span")
    );
}

#[test]
fn named_grid_resolver_handles_negative_and_missing_occurrences() {
    let lines = named_grid_lines();

    let negative = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: -1,
            },
            RawGridLine::Auto,
        ),
        None,
    )
    .unwrap();
    let missing_after = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 4,
            },
            RawGridLine::Auto,
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        negative,
        GridPlacement::try_line(3).expect("valid grid line")
    );
    assert_eq!(
        missing_after,
        GridPlacement::try_line(4).expect("valid grid line")
    );
}

#[test]
fn named_grid_resolver_normalizes_spans_and_conflicts() {
    let lines = named_grid_lines();

    let lone_named_span = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::Auto,
        ),
        Some(2),
    )
    .unwrap();
    let both_spans = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(RawGridLine::Span(2), RawGridLine::Span(3)),
        Some(1),
    )
    .unwrap();
    let mixed_named_span = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::Span(3),
        ),
        Some(1),
    )
    .unwrap();
    let mixed_anonymous_span_first = named::resolve_grid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::Span(3),
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
        ),
        Some(1),
    )
    .unwrap();
    let start_after_end =
        named::resolve_grid_placement(&lines, &RawGridPlacement::lines(3, 1), None).unwrap();
    let equal_lines =
        named::resolve_grid_placement(&lines, &RawGridPlacement::lines(2, 2), None).unwrap();

    assert_eq!(
        lone_named_span,
        GridPlacement::try_line_span(2, 1).expect("valid grid line span")
    );
    assert_eq!(
        both_spans,
        GridPlacement::try_line_span(1, 2).expect("valid grid line span")
    );
    assert_eq!(
        mixed_named_span,
        GridPlacement::try_line_span(1, 1).expect("valid grid line span")
    );
    assert_eq!(
        mixed_anonymous_span_first,
        GridPlacement::try_line_span(1, 3).expect("valid grid line span")
    );
    assert_eq!(
        start_after_end,
        GridPlacement::try_lines(1, 3).expect("valid grid lines")
    );
    assert_eq!(
        equal_lines,
        GridPlacement::try_line_span(2, 1).expect("valid grid line span")
    );
}

#[test]
fn named_grid_placement_context_ignores_non_in_flow_track_requirements() {
    let placements = vec![
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(100).expect("valid grid line"),
            row: GridPlacement::try_line(100).expect("valid grid line"),
            absolute_column: GridPlacement::try_line(100).expect("valid grid line"),
            absolute_row: GridPlacement::try_line(100).expect("valid grid line"),
            in_flow: false,
        },
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(-10).expect("valid grid line"),
            row: GridPlacement::AUTO,
            absolute_column: GridPlacement::try_line(-10).expect("valid grid line"),
            absolute_row: GridPlacement::AUTO,
            in_flow: false,
        },
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(2).expect("valid grid line"),
            row: GridPlacement::try_line(3).expect("valid grid line"),
            absolute_column: GridPlacement::try_line(2).expect("valid grid line"),
            absolute_row: GridPlacement::try_line(3).expect("valid grid line"),
            in_flow: true,
        },
    ];

    assert_eq!(
        grid_track_requirement_from_placements(&placements),
        Size::new(2, 3)
    );
    assert_eq!(
        leading_implicit_tracks_from_placements(&placements, GridAxisKind::Column, 2),
        0
    );
}

#[test]
fn grid_axis_placement_preserves_out_of_range_numeric_lines() {
    let lines =
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 2).unwrap();

    assert_eq!(
        resolve_grid_item_axis_placement(
            &lines,
            &RawGridPlacement::line(-5),
            GridPlacement::try_line(-5).expect("valid grid line"),
        ),
        GridPlacement::try_line(-5).expect("valid grid line")
    );
    assert_eq!(
        resolve_grid_item_axis_placement(
            &lines,
            &RawGridPlacement::line(5),
            GridPlacement::try_line(5).expect("valid grid line"),
        ),
        GridPlacement::try_line(5).expect("valid grid line")
    );
}

#[test]
fn named_grid_invalid_raw_placement_falls_back_to_auto() {
    let lines = named_grid_lines();

    assert_eq!(
        named::resolve_grid_placement_or_auto(
            &lines,
            &RawGridPlacement::new(RawGridLine::Line(0), RawGridLine::Auto),
            None,
        ),
        GridPlacement::AUTO
    );
    assert_eq!(
        named::resolve_grid_placement_or_auto(
            &lines,
            &RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "auto".to_string(),
                    index: 1,
                },
                RawGridLine::Auto,
            ),
            None,
        ),
        GridPlacement::AUTO
    );
    assert_eq!(
        named::resolve_grid_placement_or_auto(
            &lines,
            &RawGridPlacement::new(RawGridLine::Span(0), RawGridLine::Auto),
            Some(1),
        ),
        GridPlacement::AUTO
    );
    assert_eq!(
        named::resolve_grid_placement_or_auto(
            &lines,
            &RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "missing".to_string(),
                    index: -4,
                },
                RawGridLine::Auto,
            ),
            None,
        ),
        GridPlacement::AUTO
    );
}

#[test]
fn named_grid_placement_fallback_is_reported() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let (placement, report) = named::resolve_grid_placement_or_auto_with_report(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 0,
            },
            RawGridLine::Auto,
        ),
        None,
    );

    assert_eq!(placement, GridPlacement::AUTO);
    assert!(report.errors().contains(&NamedGridErrorReport::ZeroLine));
}

#[test]
fn named_grid_implicit_named_line_is_not_reported_as_fallback() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let (placement, report) = named::resolve_grid_placement_or_auto_with_report(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "implicit".to_string(),
                index: 1,
            },
            RawGridLine::Auto,
        ),
        None,
    );

    assert_eq!(
        placement,
        GridPlacement::try_line(4).expect("valid implicit grid line")
    );
    assert!(report.is_empty());
}

#[test]
fn subgrid_axis_placement_reports_one_authored_fallback_once() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let (placement, absolute, report) = resolve_grid_item_axis_placements_with_report(
        &lines,
        &RawGridPlacement::new(RawGridLine::Line(0), RawGridLine::Auto),
        GridPlacement::AUTO,
        true,
    );

    assert_eq!(placement, GridPlacement::AUTO);
    assert_eq!(absolute, GridPlacement::AUTO);
    assert_eq!(
        report
            .errors()
            .iter()
            .filter(|error| **error == NamedGridErrorReport::ZeroLine)
            .count(),
        1
    );
}

#[test]
fn named_lines_reject_non_rectangular_template_areas() {
    let areas = crate::GridTemplateAreas {
        rows: vec![
            crate::GridTemplateAreaRow {
                cells: vec![Some("a".to_string()), Some("a".to_string())],
            },
            crate::GridTemplateAreaRow {
                cells: vec![Some("a".to_string()), None],
            },
        ],
    };

    let error = named::GridAreaNameFacts::from_specified_areas(&areas).unwrap_err();

    assert_eq!(
        error,
        named::NamedGridError::NonRectangularTemplateArea {
            name: "a".to_string(),
        }
    );
}

#[test]
fn named_lines_treat_default_template_areas_as_noop() {
    let base =
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 1).unwrap();
    let lines = named::add_area_generated_lines(
        GridAxisKind::Column,
        base,
        &crate::GridTemplateAreas::default(),
    )
    .unwrap();

    assert_eq!(lines.explicit_track_count, 1);
    assert_eq!(lines.line_names.len(), 2);
    assert!(lines.area_facts.area_order.is_empty());
}

#[test]
fn subgrid_line_names_expand_auto_fill_and_fixed_slots() {
    let names = named::expand_subgrid_local_line_names(
        GridAxisKind::Column,
        4,
        &[
            SubgridLineNameComponent::LineNames(vec!["start".to_string()]),
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                line_name_sets: vec![vec!["fill".to_string()]],
            },
            SubgridLineNameComponent::LineNames(vec!["end".to_string()]),
        ],
    )
    .unwrap();

    assert_eq!(names.len(), 5);
    assert_eq!(
        local_line_names(&names),
        vec![
            vec!["start"],
            vec!["fill"],
            vec!["fill"],
            vec!["fill"],
            vec!["end"],
        ]
    );
}

#[test]
fn subgrid_line_names_inherit_parent_explicit_and_local_names() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let local = local_subgrid_entries(&[&["local-start"], &[], &["middle"], &["local-end"]]);

    let lines = named::inherit_subgrid_named_lines(&parent, 2, 5, false, &local, None).unwrap();

    assert_eq!(
        entry_names(lines.entries_on_line(1)),
        vec!["b", "local-start"]
    );
    assert_eq!(entry_names(lines.entries_on_line(3)), vec!["c", "middle"]);
    assert_eq!(
        entry_names(lines.entries_on_line(4)),
        vec!["d", "local-end"]
    );
    assert_eq!(
        lines.entries_on_line(1)[1].origin,
        named::LineNameOrigin::LocalSubgrid
    );
}

#[test]
fn subgrid_line_names_reinherit_local_parent_names() {
    let parent = named_parent_lines(2, &[&["outer"], &[], &["outer-end"]]);
    let outer_local = local_subgrid_entries(&[&["local-start"], &[], &["local-end"]]);
    let outer =
        named::inherit_subgrid_named_lines(&parent, 1, 3, false, &outer_local, None).unwrap();
    let nested_local = local_subgrid_entries(&[&[], &[], &[]]);

    let nested =
        named::inherit_subgrid_named_lines(&outer, 1, 3, false, &nested_local, None).unwrap();

    assert_eq!(
        entry_names(nested.entries_on_line(1)),
        vec!["outer", "local-start"]
    );
    assert_eq!(
        entry_names(nested.entries_on_line(3)),
        vec!["outer-end", "local-end"]
    );
}

#[test]
fn subgrid_line_names_reverse_parent_line_order() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let local = local_subgrid_entries(&[&[], &[], &[], &[]]);

    let lines = named::inherit_subgrid_named_lines(&parent, 2, 5, true, &local, None).unwrap();

    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["d"]);
    assert_eq!(entry_names(lines.entries_on_line(2)), vec!["c"]);
    assert_eq!(entry_names(lines.entries_on_line(4)), vec!["b"]);
}

#[test]
fn subgrid_intrinsic_parent_context_uses_actual_span_and_reversal() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let report = SubgridAxisReport {
        mapping: Ok(GridAxisMappingReport {
            queried_axis: GridAxisKind::Column,
            parent_axis: GridAxisKind::Column,
            child_axis: GridAxisKind::Column,
            reversed: true,
        }),
        eligibility: SubgridEligibility {
            eligible: true,
            reason: None,
        },
    };

    let axis = intrinsic_subgrid_axis_parent_context(
        report,
        GridArea {
            row: 0,
            column: 1,
            row_end: 1,
            column_end: 4,
            size: Size::<Scalar>::ZERO,
        },
        Size::<Scalar>::ZERO,
        &parent,
        &parent,
        None,
    )
    .unwrap();
    let local = local_subgrid_entries(&[&[], &[], &[], &[]]);
    let lines = named::inherit_subgrid_named_lines(
        &axis.named_lines,
        axis.parent_start + 1,
        axis.parent_end + 1,
        axis.reversed,
        &local,
        axis.area_facts.as_ref(),
    )
    .unwrap();

    assert_eq!(axis.parent_start, 1);
    assert_eq!(axis.parent_end, 4);
    assert!(axis.reversed);
    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["d"]);
    assert_eq!(entry_names(lines.entries_on_line(4)), vec!["b"]);
}

#[test]
fn subgrid_line_names_recompute_area_generated_names_clipped_to_span() {
    let areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![
                Some("a".to_string()),
                Some("a".to_string()),
                Some("a".to_string()),
                Some("a".to_string()),
            ],
        }],
    };
    let parent = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &areas,
    )
    .unwrap();
    let local = local_subgrid_entries(&[&[], &[], &[]]);

    let lines =
        named::inherit_subgrid_named_lines(&parent, 2, 4, false, &local, Some(&parent.area_facts))
            .unwrap();

    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["a-start"]);
    assert_eq!(entry_names(lines.entries_on_line(3)), vec!["a-end"]);
}

#[test]
fn subgrid_area_facts_preserve_reversed_orientation_and_axis_validity() {
    let areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![
                None,
                Some("main".to_string()),
                Some("main".to_string()),
                None,
            ],
        }],
    };
    let parent_lines = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &areas,
    )
    .unwrap();
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            parent_lines.clone(),
            Some(parent_lines.area_facts.clone()),
            1,
            3,
            true,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &NodeInput {
            grid_template_columns: subgrid_track(),
            ..NodeInput::DEFAULT
        },
        2,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.named_occurrences("main-start"), vec![3]);
    assert_eq!(context.columns.named_occurrences("main-end"), vec![1]);
    assert!(facts.columns_valid);
    assert!(!facts.rows_valid);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 3);
    assert_eq!(rectangle.column_start_name, 3);
    assert_eq!(rectangle.column_end_name, 1);
}

#[test]
fn subgrid_local_area_facts_clamp_to_inherited_span() {
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4)
                .unwrap(),
            None,
            0,
            2,
            false,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &NodeInput {
            grid_template_columns: subgrid_track(),
            grid_template_areas: crate::GridTemplateAreas {
                rows: vec![crate::GridTemplateAreaRow {
                    cells: vec![
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                    ],
                }],
            },
            ..NodeInput::DEFAULT
        },
        2,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.explicit_track_count, 2);
    assert_eq!(context.columns.named_occurrences("wide-start"), vec![1]);
    assert_eq!(context.columns.named_occurrences("wide-end"), vec![3]);
    assert_eq!(facts.column_count, 2);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 3);
}

#[test]
fn subgrid_duplicate_area_facts_merge_with_parent_clipped_boundaries() {
    let parent_areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![Some("same".to_string()), None, None, None],
        }],
    };
    let parent_lines = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &parent_areas,
    )
    .unwrap();
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            parent_lines.clone(),
            Some(parent_lines.area_facts.clone()),
            0,
            3,
            false,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &NodeInput {
            grid_template_columns: subgrid_track(),
            grid_template_areas: crate::GridTemplateAreas {
                rows: vec![crate::GridTemplateAreaRow {
                    cells: vec![None, Some("same".to_string()), None],
                }],
            },
            ..NodeInput::DEFAULT
        },
        3,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.named_occurrences("same-start"), vec![1]);
    assert_eq!(context.columns.named_occurrences("same-end"), vec![2]);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 2);
}

#[test]
fn subgrid_named_placement_clamps_beyond_explicit_span() {
    let lines = named_parent_lines(2, &[&["a"], &[], &["a"]]);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: -3,
            },
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 4,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(1, 3).expect("valid grid lines")
    );
}

#[test]
fn subgrid_named_placement_resolves_wpt_line_names_before_clamping_to_span() {
    let parent = named_parent_lines(6, &[&["a"], &[], &[], &[], &["b"], &[], &["a", "b"]]);
    let local = local_subgrid_entries(&[&["x"], &["b"], &[], &[], &["b"]]);
    let lines = named::inherit_subgrid_named_lines(&parent, 2, 6, false, &local, None).unwrap();

    assert_eq!(lines.named_occurrences("b"), vec![2, 4, 5]);

    let cases = [
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(4, 5).expect("valid grid lines"),
        ),
    ];

    for (raw, expected) in cases {
        let placement = named::resolve_subgrid_placement(&lines, &raw, None).unwrap();
        assert_eq!(placement, expected, "raw placement {raw:?}");
    }
}

#[test]
fn subgrid_named_placement_resolves_wpt_named_spans_before_clamping_to_span() {
    let parent = named_parent_lines(6, &[&["a"], &[], &[], &[], &["b"], &[], &["a", "b"]]);
    let local = local_subgrid_entries(&[&["x"], &["b"], &[], &[], &["b"]]);
    let lines = named::inherit_subgrid_named_lines(&parent, 2, 6, false, &local, None).unwrap();

    let cases = [
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 2,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(1, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
    ];

    for (raw, expected) in cases {
        let placement = named::resolve_subgrid_placement(&lines, &raw, None).unwrap();
        assert_eq!(placement, expected, "raw placement {raw:?}");
    }
}

#[test]
fn subgrid_named_placement_expands_collapsed_clamp_to_edge_track() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 1);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(3)),
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(1, 2).expect("valid grid lines")
    );
}

#[test]
fn subgrid_named_span_counts_implicit_names_beyond_end_before_clamping() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 10);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 1,
            },
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 8,
            },
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(10, 11).expect("valid grid lines")
    );
}

fn baseline_test_item(
    row: usize,
    column: usize,
    row_span: usize,
    align_self: AlignItems,
    first: Scalar,
    last: Scalar,
    height: Scalar,
) -> PendingGridItem<()> {
    PendingGridItem {
        node: (),
        order: 0,
        area: GridArea {
            row,
            column,
            row_end: row + row_span,
            column_end: column + 1,
            size: Size::new(40.0, height),
        },
        output: ComputeOutput::from_sizes_and_baselines(
            Size::new(40.0, height),
            Size::ZERO,
            Baselines {
                first: Point::new(None, Some(first)),
                last: Point::new(None, Some(last)),
            },
        ),
        horizontal_axis: ResolvedGridItemAxis {
            offset: 0.0,
            margin_start: 0.0,
            margin_end: 0.0,
        },
        vertical_axis: ResolvedGridItemAxis {
            offset: 0.0,
            margin_start: 0.0,
            margin_end: 0.0,
        },
        relative_offset: Point::ZERO,
        first_baseline: first,
        last_baseline: last,
        published_row_baselines: None,
        block_offset: 0.0,
        block_auto_margins: false,
        baseline_participation: BaselineParticipation {
            participates: matches!(align_self, AlignItems::Baseline | AlignItems::LastBaseline),
            group: match align_self {
                AlignItems::Baseline => Some(BaselineGroupKind::Major),
                AlignItems::LastBaseline => Some(BaselineGroupKind::Minor),
                _ => None,
            },
            synthesized: false,
            fallback_alignment: None,
        },
        margin: Edges::ZERO,
        scrollbar_size: Size::ZERO,
        border: Edges::ZERO,
        padding: Edges::ZERO,
        overflow: Point::new(Overflow::Visible, Overflow::Visible),
    }
}

fn named_parent_lines(
    explicit_track_count: usize,
    line_names: &[&[&str]],
) -> named::NamedGridLines {
    let mut lines = named::NamedGridLines::new(GridAxisKind::Column, explicit_track_count);
    for (line_index, names) in line_names.iter().enumerate() {
        lines.line_names[line_index] = names
            .iter()
            .map(|name| named::LineNameEntry {
                name: (*name).to_string(),
                origin: named::LineNameOrigin::Explicit,
            })
            .collect();
    }
    lines
}

fn test_inherited_axis(
    named_lines: named::NamedGridLines,
    area_facts: Option<named::GridAreaNameFacts>,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
) -> InheritedGridAxis {
    let track_count = parent_end - parent_start;
    InheritedGridAxis {
        offset: 0.0,
        gap: 0.0,
        tracks: vec![0.0; track_count],
        named_lines,
        area_facts,
        major_baselines: vec![None; track_count],
        minor_baselines: vec![None; track_count],
        parent_start,
        parent_end,
        reversed,
        start_mbp: 0.0,
        end_mbp: 0.0,
        gap_difference: 0.0,
    }
}

fn local_subgrid_entries(line_names: &[&[&str]]) -> Vec<Vec<named::LineNameEntry>> {
    line_names
        .iter()
        .map(|names| {
            names
                .iter()
                .map(|name| named::LineNameEntry {
                    name: (*name).to_string(),
                    origin: named::LineNameOrigin::LocalSubgrid,
                })
                .collect()
        })
        .collect()
}

fn local_line_names(line_names: &[Vec<named::LineNameEntry>]) -> Vec<Vec<&str>> {
    line_names
        .iter()
        .map(|entries| entry_names(entries))
        .collect()
}

fn entry_names(entries: &[named::LineNameEntry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.name.as_str()).collect()
}

#[test]
fn row_baselines_choose_first_baseline_for_first_group() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 22.0, 30.0),
        baseline_test_item(0, 1, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 2);

    assert_eq!(groups.rows[0].first, Some(14.0));
}

#[test]
fn row_baselines_choose_last_baseline_for_last_group() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::LastBaseline, 8.0, 22.0, 30.0),
        baseline_test_item(0, 1, 2, AlignItems::LastBaseline, 8.0, 18.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 2);

    assert_eq!(groups.rows[1].last, Some(12.0));
}

#[test]
fn row_baselines_keep_first_groups_per_start_row() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 22.0, 30.0),
        baseline_test_item(1, 0, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 1);

    assert_eq!(groups.rows[0].first, Some(8.0));
    assert_eq!(groups.rows[1].first, Some(14.0));
}

#[test]
fn row_baselines_keep_last_groups_per_end_row() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 22.0, 30.0),
        baseline_test_item(1, 0, 1, AlignItems::LastBaseline, 8.0, 18.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2, 1);

    assert_eq!(groups.rows[0].last, Some(8.0));
    assert_eq!(groups.rows[1].last, Some(12.0));
}

#[test]
fn published_row_baselines_map_reversed_subgrid_back_to_parent_rows() {
    let axis = InheritedGridAxis {
        offset: 0.0,
        gap: 10.0,
        tracks: vec![40.0, 40.0, 40.0],
        named_lines: named::NamedGridLines::new(GridAxisKind::Row, 3),
        area_facts: None,
        major_baselines: vec![None, None, None],
        minor_baselines: vec![None, None, None],
        parent_start: 1,
        parent_end: 4,
        reversed: true,
        start_mbp: 3.0,
        end_mbp: 7.0,
        gap_difference: -2.0,
    };
    let published = publish_row_baseline_groups(
        &[
            TrackBaselineGroup {
                first: Some(10.0),
                last: None,
            },
            TrackBaselineGroup {
                first: Some(20.0),
                last: Some(6.0),
            },
            TrackBaselineGroup {
                first: None,
                last: Some(12.0),
            },
        ],
        &axis,
    );

    assert_eq!(
        published,
        vec![
            PublishedTrackBaselineGroup {
                parent_index: 3,
                group: TrackBaselineGroup {
                    first: Some(11.0),
                    last: None,
                },
            },
            PublishedTrackBaselineGroup {
                parent_index: 2,
                group: TrackBaselineGroup {
                    first: Some(16.0),
                    last: Some(2.0),
                },
            },
            PublishedTrackBaselineGroup {
                parent_index: 1,
                group: TrackBaselineGroup {
                    first: None,
                    last: Some(17.0),
                },
            },
        ]
    );
}

#[test]
fn empty_published_row_baselines_do_not_suppress_item_fallback() {
    let mut item = baseline_test_item(0, 0, 1, AlignItems::Baseline, 9.0, 11.0, 20.0);
    item.published_row_baselines = Some(Vec::new());

    let groups = baseline_groups(&[item], 1, 1);

    assert_eq!(groups.rows[0].first, Some(9.0));
}

#[test]
fn baseline_groups_columns_are_default_filled_to_grid_width() {
    let items = vec![baseline_test_item(
        0,
        0,
        1,
        AlignItems::Baseline,
        8.0,
        22.0,
        30.0,
    )];

    let groups = baseline_groups(&items, 1, 3);

    assert_eq!(groups.columns, vec![TrackBaselineGroup::default(); 3],);
}

#[test]
fn spanned_track_size_counts_tracks_and_internal_gaps() {
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 0, 1, 7.0), 20.0);
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 0, 2, 7.0), 67.0);
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 1, 3, 7.0), 57.0);
}

#[test]
fn baseline_offset_major_uses_margin_box_baseline() {
    let offset = baseline_offset(
        BaselineGroupKind::Major,
        20.0,
        BaselineGeometry {
            available_span_size: 70.0,
            margin_box_size: 40.0,
            major_baseline: 14.0,
            minor_baseline: 12.0,
        },
    );

    assert_eq!(offset, 6.0);
}

#[test]
fn baseline_offset_minor_uses_alignment_context_end() {
    let offset = baseline_offset(
        BaselineGroupKind::Minor,
        18.0,
        BaselineGeometry {
            available_span_size: 70.0,
            margin_box_size: 40.0,
            major_baseline: 14.0,
            minor_baseline: 12.0,
        },
    );

    assert_eq!(offset, 24.0);
}

#[test]
fn baseline_offset_major_allows_row_spanning_gap_area() {
    let offset = baseline_offset(
        BaselineGroupKind::Major,
        14.0,
        BaselineGeometry {
            available_span_size: 90.0,
            margin_box_size: 30.0,
            major_baseline: 8.0,
            minor_baseline: 10.0,
        },
    );

    assert_eq!(offset, 6.0);
}

#[test]
fn baseline_offset_minor_allows_row_spanning_gap_area() {
    let offset = baseline_offset(
        BaselineGroupKind::Minor,
        14.0,
        BaselineGeometry {
            available_span_size: 90.0,
            margin_box_size: 30.0,
            major_baseline: 8.0,
            minor_baseline: 10.0,
        },
    );

    assert_eq!(offset, 56.0);
}

#[test]
fn baseline_shim_for_intrinsic_contribution_first_grows_before_item() {
    let shim = baseline_shim_for_intrinsic_contribution(
        BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Major),
            synthesized: false,
            fallback_alignment: Some(AlignItems::Start),
        },
        BaselineGeometry {
            available_span_size: 40.0,
            margin_box_size: 30.0,
            major_baseline: 6.0,
            minor_baseline: 8.0,
        },
        TrackBaselineGroup {
            first: Some(18.0),
            last: Some(12.0),
        },
    );

    assert_eq!(
        shim,
        BaselineShim {
            before: 12.0,
            after: 0.0,
        }
    );
}

#[test]
fn baseline_shim_for_intrinsic_contribution_last_grows_after_item() {
    let shim = baseline_shim_for_intrinsic_contribution(
        BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Minor),
            synthesized: false,
            fallback_alignment: Some(AlignItems::End),
        },
        BaselineGeometry {
            available_span_size: 40.0,
            margin_box_size: 30.0,
            major_baseline: 6.0,
            minor_baseline: 2.0,
        },
        TrackBaselineGroup {
            first: Some(18.0),
            last: Some(12.0),
        },
    );

    assert_eq!(
        shim,
        BaselineShim {
            before: 0.0,
            after: 10.0,
        }
    );
}

#[test]
fn baseline_shim_for_intrinsic_contribution_nonparticipant_is_zero() {
    let shim = baseline_shim_for_intrinsic_contribution(
        BaselineParticipation {
            participates: false,
            group: None,
            synthesized: false,
            fallback_alignment: None,
        },
        BaselineGeometry {
            available_span_size: 40.0,
            margin_box_size: 30.0,
            major_baseline: 6.0,
            minor_baseline: 2.0,
        },
        TrackBaselineGroup {
            first: Some(18.0),
            last: Some(12.0),
        },
    );

    assert_eq!(shim, BaselineShim::default());
}

#[test]
fn baseline_shim_for_intrinsic_contribution_synthesized_baseline_participates() {
    let participation = baseline_participation(AlignItems::Baseline, false, false, Baselines::NONE);

    let shim = baseline_shim_for_intrinsic_contribution(
        participation,
        BaselineGeometry {
            available_span_size: 40.0,
            margin_box_size: 30.0,
            major_baseline: 6.0,
            minor_baseline: 2.0,
        },
        TrackBaselineGroup {
            first: Some(18.0),
            last: Some(12.0),
        },
    );

    assert_eq!(
        shim,
        BaselineShim {
            before: 12.0,
            after: 0.0
        }
    );
}

#[test]
fn baseline_aligned_block_offset_first_single_row_item() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 1, 2);

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0], 0.0),
        Some(6.0)
    );
    assert_eq!(
        baseline_aligned_block_offset(&items[1], &groups, &[40.0], 0.0),
        Some(0.0)
    );
}

#[test]
fn baseline_aligned_block_offset_first_spanning_item() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::Baseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 2, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 2, 2);

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0, 40.0], 7.0),
        Some(6.0)
    );
}

#[test]
fn baseline_aligned_block_offset_last_single_row_item() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 1, 2);

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0], 0.0),
        Some(14.0)
    );
    assert_eq!(
        baseline_aligned_block_offset(&items[1], &groups, &[40.0], 0.0),
        Some(10.0)
    );
}

#[test]
fn baseline_aligned_block_offset_last_spanning_item() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 2, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    let groups = baseline_groups(&items, 2, 2);

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0, 40.0], 7.0),
        Some(61.0)
    );
}

#[test]
fn baseline_aligned_block_offset_first_and_last_include_margins() {
    let mut first_items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];
    first_items[0].vertical_axis.margin_start = 3.0;
    first_items[0].vertical_axis.margin_end = 5.0;
    let first_groups = baseline_groups(&first_items, 1, 2);

    assert_eq!(
        baseline_aligned_block_offset(&first_items[0], &first_groups, &[40.0], 0.0),
        Some(6.0)
    );

    let mut last_items = vec![
        baseline_test_item(0, 0, 1, AlignItems::LastBaseline, 8.0, 16.0, 20.0),
        baseline_test_item(0, 1, 1, AlignItems::LastBaseline, 14.0, 20.0, 30.0),
    ];
    last_items[0].vertical_axis.margin_start = 3.0;
    last_items[0].vertical_axis.margin_end = 5.0;
    let last_groups = baseline_groups(&last_items, 1, 2);

    assert_eq!(
        baseline_aligned_block_offset(&last_items[0], &last_groups, &[40.0], 0.0),
        Some(14.0)
    );
}

#[test]
fn baseline_aligned_block_offset_returns_none_without_group_baseline() {
    let items = [baseline_test_item(
        0,
        0,
        1,
        AlignItems::Baseline,
        8.0,
        16.0,
        20.0,
    )];
    let groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default()],
        columns: vec![TrackBaselineGroup::default()],
    };

    assert_eq!(
        baseline_aligned_block_offset(&items[0], &groups, &[40.0], 0.0),
        None
    );
}

#[test]
fn baseline_participation_rejects_block_auto_margins() {
    let participation = baseline_participation(AlignItems::Baseline, true, false, Baselines::NONE);

    assert_eq!(
        participation,
        BaselineParticipation {
            participates: false,
            group: Some(BaselineGroupKind::Major),
            synthesized: true,
            fallback_alignment: Some(AlignItems::Start),
        }
    );
}

#[test]
fn baseline_participation_accepts_synthesized_baselines() {
    let participation =
        baseline_participation(AlignItems::LastBaseline, false, false, Baselines::NONE);

    assert_eq!(
        participation,
        BaselineParticipation {
            participates: true,
            group: Some(BaselineGroupKind::Minor),
            synthesized: true,
            fallback_alignment: Some(AlignItems::End),
        }
    );
}

#[test]
fn grid_axis_mapping_supports_horizontal_rtl_reversal() {
    let report = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            direction: Direction::Rtl,
            ..NodeInput::default()
        },
        child_style: &NodeInput::default(),
    })
    .unwrap();

    assert_eq!(report.parent_axis, GridAxisKind::Column);
    assert_eq!(report.child_axis, GridAxisKind::Column);
    assert!(report.reversed);
}

#[test]
fn grid_axis_mapping_maps_child_vertical_axes_to_parent_physical_axes() {
    let column = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Column,
        parent_style: &NodeInput::default(),
        child_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
    })
    .unwrap();
    let row = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Row,
        parent_style: &NodeInput::default(),
        child_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
    })
    .unwrap();

    assert_eq!(column.parent_axis, GridAxisKind::Row);
    assert_eq!(column.child_axis, GridAxisKind::Column);
    assert_eq!(row.parent_axis, GridAxisKind::Column);
    assert_eq!(row.child_axis, GridAxisKind::Row);
}

#[test]
fn grid_axis_mapping_maps_vertical_parent_axes_to_horizontal_child_physical_axes() {
    let column = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
        child_style: &NodeInput::default(),
    })
    .unwrap();
    let row = map_grid_axis(GridAxisMappingInput {
        queried_axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
        child_style: &NodeInput::default(),
    })
    .unwrap();

    assert_eq!(column.parent_axis, GridAxisKind::Row);
    assert_eq!(column.child_axis, GridAxisKind::Column);
    assert!(column.reversed);
    assert_eq!(row.parent_axis, GridAxisKind::Column);
    assert_eq!(row.child_axis, GridAxisKind::Row);
    assert!(!row.reversed);
}

#[test]
fn vertical_subgrid_percentage_gap_uses_flow_relative_axis_basis() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalLr,
        gap: Size::new(Length::Percent(0.10), Length::Percent(0.10)),
        ..NodeInput::default()
    };
    let area_size = Size::new(300.0, 500.0);

    assert_eq!(
        child_subgrid_gap(&style, GridAxisKind::Column, area_size, &NoCalcResolver),
        ResolvedSubgridGap::Length(50.0)
    );
    assert_eq!(
        child_subgrid_gap(&style, GridAxisKind::Row, area_size, &NoCalcResolver),
        ResolvedSubgridGap::Length(30.0)
    );
}

#[test]
fn grid_area_physical_origin_maps_vertical_grid_tracks_without_collapsing_rows() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let column_offsets = [0.0, 30.0];
    let row_offsets = [60.0, 0.0];

    assert_eq!(
        grid_area_physical_origin(
            &style,
            &column_offsets,
            &row_offsets,
            GridArea {
                column: 0,
                column_end: 1,
                row: 0,
                row_end: 1,
                size: Size::new(30.0, 50.0),
            },
        ),
        Point::new(60.0, 0.0)
    );
    assert_eq!(
        grid_area_physical_origin(
            &style,
            &column_offsets,
            &row_offsets,
            GridArea {
                column: 1,
                column_end: 2,
                row: 1,
                row_end: 2,
                size: Size::new(40.0, 60.0),
            },
        ),
        Point::new(0.0, 30.0)
    );
}

#[test]
fn vertical_grid_axis_offsets_add_local_inset_to_inherited_offsets() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalLr,
        ..NodeInput::default()
    };
    let tracks = [20.0, 30.0];
    let alignment = GridAlignment {
        start: 7.0,
        gap: 5.0,
    };
    let content_box_inset = Edges {
        left: 11.0,
        right: 0.0,
        top: 13.0,
        bottom: 0.0,
    };

    let column_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style: &style,
        axis: GridAxisKind::Column,
        tracks: &tracks,
        inherited_offset: Some(100.0),
        content_box_left: 0.0,
        content_box_size: Size::new(300.0, 400.0),
        content_box_inset,
        alignment,
    });
    let row_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style: &style,
        axis: GridAxisKind::Row,
        tracks: &tracks,
        inherited_offset: Some(200.0),
        content_box_left: 0.0,
        content_box_size: Size::new(300.0, 400.0),
        content_box_inset,
        alignment,
    });

    assert_eq!(column_offsets, vec![120.0, 145.0]);
    assert_eq!(row_offsets, vec![218.0, 243.0]);
}

#[test]
fn absolute_grid_item_axis_placement_preserves_end_only_first_line() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 3);

    let placement = resolve_absolute_grid_item_axis_placement(
        &lines,
        &RawGridPlacement::new(RawGridLine::Auto, RawGridLine::Line(1)),
        GridPlacement::try_end_line(1).expect("valid grid line"),
    );

    assert_eq!(
        placement,
        GridPlacement::try_end_line(1).expect("valid grid line")
    );
}

#[test]
fn absolute_grid_axis_area_uses_left_edge_for_definite_rtl_range() {
    let tracks = vec![30.0; 8];
    let offsets = rtl_offsets(&tracks, 0.0, 240.0, 0.0, 0.0);

    let area = absolute_grid_axis_area(AbsoluteGridAxisInput {
        placement: GridPlacement::try_lines(3, 5).expect("valid grid lines"),
        tracks: &tracks,
        offsets: &offsets,
        gap: 0.0,
        padding_box_location: 0.0,
        padding_box_size: 240.0,
        is_reverse: true,
        explicit_start: 0,
        explicit_count: 8,
        reverse_positive_line_offset_adjustment: 0.0,
    });

    assert_eq!(area.location, 120.0);
    assert_eq!(area.size, 60.0);
}

#[test]
fn grid_item_sizing_transfers_min_block_through_aspect_ratio_to_inline_size() {
    let child_style = NodeInput {
        min_size: Size::new(Dimension::AUTO, Dimension::px(50.0)),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let sizing = grid_item_sizing(
        &child_style,
        &NodeInput::default(),
        Size::new(100.0, 100.0),
        Size::splat(Some(100.0)),
        &NoCalcResolver,
    );

    assert_eq!(sizing.known, Size::new(Some(200.0), Some(100.0)));
}

#[test]
fn grid_item_sizing_keeps_inline_stretch_when_min_inline_defines_aspect_ratio() {
    let child_style = NodeInput {
        min_size: Size::new(Dimension::px(50.0), Dimension::AUTO),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let sizing = grid_item_sizing(
        &child_style,
        &NodeInput::default(),
        Size::new(100.0, 100.0),
        Size::splat(Some(100.0)),
        &NoCalcResolver,
    );

    assert_eq!(sizing.known, Size::new(Some(100.0), Some(50.0)));
}

#[test]
fn subgrid_eligibility_reports_first_blocking_reason() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: false,
        child_style: &NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(report.reason, Some(SubgridIneligibleReason::NoParentGrid));
}

#[test]
fn subgrid_eligibility_rejects_non_grid_container_display() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Block,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report.reason,
        Some(SubgridIneligibleReason::UnsupportedDisplay)
    );
}

#[test]
fn subgrid_eligibility_rejects_excluded_children() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            position: Position::Absolute,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report.reason,
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    );
}

#[test]
fn subgrid_eligibility_rejects_display_none_children() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::None,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report.reason,
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    );
}

#[test]
fn subgrid_eligibility_allows_clipped_overflow() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            overflow: Point::new(Overflow::Hidden, Overflow::Visible),
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(report.eligible);
    assert_eq!(report.reason, None);
}

#[test]
fn subgrid_axis_report_allows_supported_vertical_parent_mapping_to_inherit() {
    let report = subgrid_axis_report(
        &NodeInput {
            display: Display::Grid,
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
        &NodeInput {
            display: Display::Grid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
        GridAxisKind::Column,
    );

    assert!(report.eligibility.eligible);
    assert_eq!(
        report.mapping,
        Ok(GridAxisMappingReport {
            queried_axis: GridAxisKind::Column,
            parent_axis: GridAxisKind::Row,
            child_axis: GridAxisKind::Column,
            reversed: true,
        })
    );
    assert!(report.can_inherit());
}

fn subgrid_item_report(parent: &NodeInput, child: &NodeInput) -> SubgridItemReport<()> {
    SubgridItemReport {
        node: (),
        column: subgrid_axis_report(parent, child, GridAxisKind::Column),
        row: subgrid_axis_report(parent, child, GridAxisKind::Row),
    }
}

fn grid_area(column: usize, column_end: usize, row: usize, row_end: usize) -> GridArea {
    GridArea {
        column,
        column_end,
        row,
        row_end,
        size: Size::ZERO,
    }
}

#[test]
fn intrinsic_subgrid_context_is_needed_for_both_axis_subgrids() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 3, 0, 2),
        &NoCalcResolver,
    ));
}

#[test]
fn intrinsic_subgrid_context_is_not_needed_for_single_column_both_axis_subgrid() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(!needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
        &NoCalcResolver,
    ));
}

#[test]
fn intrinsic_subgrid_context_is_needed_for_row_subgrid_with_percent_columns() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: vec![TrackComponent::percent(0.5)],
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
        &NoCalcResolver,
    ));
}

#[test]
fn intrinsic_subgrid_context_uses_mapped_parent_axis_for_orthogonal_subgrid() {
    let parent = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Column,
        grid_template_columns: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
        &NoCalcResolver,
    ));
}

#[test]
fn subgrid_eligibility_rejects_grid_lanes_parent_in_lane_axis() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            display: Display::GridLanes,
            grid_auto_flow: GridAutoFlow::Row,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            grid_template_rows: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report.reason,
        Some(SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    );
}

#[test]
fn subgrid_eligibility_allows_grid_lanes_parent_in_grid_axis() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::GridLanes,
            grid_auto_flow: GridAutoFlow::Row,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(report.eligible);
    assert_eq!(report.reason, None);
}

#[test]
fn subgrid_eligibility_treats_inline_grid_lanes_parent_as_lanes() {
    let rejected = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            display: Display::InlineGridLanes,
            grid_auto_flow: GridAutoFlow::Row,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::InlineGrid,
            grid_template_rows: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        rejected.reason,
        Some(SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    );

    let allowed = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::InlineGridLanes,
            grid_auto_flow: GridAutoFlow::Row,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::InlineGrid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(allowed.eligible);
    assert_eq!(allowed.reason, None);
}

#[test]
fn subgrid_eligibility_allows_ordinary_grid_parent_in_both_axes() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let report = subgrid_eligibility(SubgridEligibilityInput {
            axis,
            parent_style: &parent,
            has_parent_grid: true,
            child_style: &child,
        });

        assert!(report.eligible, "{axis:?} subgrid should be eligible");
        assert_eq!(report.reason, None);
    }
}

#[test]
fn subgrid_eligibility_allows_grid_lanes_child_display() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::GridLanes,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert!(report.eligible);
    assert_eq!(report.reason, None);
}

#[test]
fn subgrid_eligibility_allows_inline_grid_child_display() {
    for display in [Display::InlineGrid, Display::InlineGridLanes] {
        let report = subgrid_eligibility(SubgridEligibilityInput {
            axis: GridAxisKind::Column,
            parent_style: &NodeInput {
                display: Display::Grid,
                ..NodeInput::default()
            },
            has_parent_grid: true,
            child_style: &NodeInput {
                display,
                grid_template_columns: subgrid_track(),
                ..NodeInput::default()
            },
        });

        assert!(report.eligible, "{display:?} should be eligible");
        assert_eq!(report.reason, None);
    }
}

#[test]
fn subgrid_track_inheritance_copies_parent_columns_for_span() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 60.0, 90.0],
        parent_span: GridTrackSpan::new(2, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.copied_parent_tracks, vec![60.0, 90.0]);
    assert_eq!(report.final_tracks, vec![60.0, 90.0]);
}

#[test]
fn subgrid_track_inheritance_reverses_copied_columns_before_mbp_consumption() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 60.0, 90.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: true,
        start_mbp: 10.0,
        end_mbp: 20.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.after_reversal, vec![90.0, 60.0, 40.0]);
    assert_eq!(report.final_tracks, vec![80.0, 60.0, 20.0]);
}

#[test]
fn subgrid_track_inheritance_consumes_start_and_end_mbp_across_tracks() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[5.0, 20.0, 10.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 12.0,
        end_mbp: 25.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(0.0),
    })
    .unwrap();

    assert_eq!(report.start_mbp_removed, vec![0.0, 13.0, 10.0]);
    assert_eq!(report.end_mbp_removed, vec![0.0, 0.0, 0.0]);
    assert_eq!(report.final_tracks, vec![0.0, 0.0, 0.0]);
}

#[test]
fn subgrid_track_inheritance_resolves_normal_gap_to_parent_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[50.0, 50.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Normal,
    })
    .unwrap();

    assert_eq!(report.resolved_subgrid_gap, 20.0);
    assert_eq!(report.gap_difference, 0.0);
    assert_eq!(report.final_tracks, vec![50.0, 50.0]);
}

#[test]
fn subgrid_track_inheritance_applies_explicit_gap_difference_to_internal_edges() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[50.0, 50.0, 50.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(20.0),
    })
    .unwrap();

    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.final_tracks, vec![45.0, 40.0, 45.0]);
}

#[test]
fn column_subgrid_layout_tracks_expand_collapsed_tracks_into_shifted_lines() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(150.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Column, &report);

    assert_eq!(report.final_tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(tracks, vec![175.0, 100.0, 25.0]);
    assert_eq!(gap, 0.0);
}

#[test]
fn row_subgrid_layout_tracks_keep_collapsed_tracks_with_resolved_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(150.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Row, &report);

    assert_eq!(report.final_tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(gap, 150.0);
}

#[test]
fn subgrid_layout_tracks_keep_non_collapsed_gap_sizing() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Length(100.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Column, &report);

    assert_eq!(report.final_tracks, vec![60.0, 60.0]);
    assert_eq!(tracks, vec![60.0, 60.0]);
    assert_eq!(gap, 100.0);
}

#[test]
fn subgrid_track_inheritance_expands_tracks_for_smaller_subgrid_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 40.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.gap_difference, -5.0);
    assert_eq!(report.final_tracks, vec![45.0, 45.0]);
}

#[test]
fn subgrid_baselines_apply_negative_gap_difference_to_internal_edges() {
    let report = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &[Some(13.0), Some(20.0)],
        parent_minor: &[Some(5.0), Some(20.0)],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: 10.0,
    })
    .unwrap();

    assert_eq!(report.gap_difference, -5.0);
    assert_eq!(report.final_major, vec![Some(18.0), Some(25.0)]);
    assert_eq!(report.final_minor, vec![Some(10.0), Some(25.0)]);
}

#[test]
fn subgrid_baselines_reverse_and_adjust_edges() {
    let report = inherit_subgrid_baselines(SubgridBaselineInheritanceInput {
        parent_major: &[Some(6.0), None, Some(14.0)],
        parent_minor: &[Some(3.0), Some(8.0), None],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: true,
        start_mbp: 2.0,
        end_mbp: 5.0,
        parent_gap: 12.0,
        subgrid_gap: 12.0,
    })
    .unwrap();

    assert_eq!(
        report.after_reversal_major,
        vec![Some(14.0), None, Some(6.0)]
    );
    assert_eq!(report.final_major, vec![Some(12.0), None, Some(6.0)]);
    assert_eq!(report.final_minor, vec![None, Some(8.0), Some(-2.0)]);
}

#[test]
fn column_subgrid_context_preserves_inherited_baseline_groups() {
    let parent_style = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        grid_template_columns: subgrid_track(),
        grid_template_rows: vec![TrackComponent::px(20.0)],
        ..NodeInput::default()
    };
    let parent_baseline_groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default()],
        columns: vec![
            TrackBaselineGroup {
                first: Some(8.0),
                last: Some(3.0),
            },
            TrackBaselineGroup {
                first: Some(14.0),
                last: Some(5.0),
            },
        ],
    };
    let parent_named_columns = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let parent_named_rows = named::NamedGridLines::new(GridAxisKind::Row, 1);

    let context = subgrid_child_parent_context(SubgridChildParentContextInput {
        item: SubgridItemReport {
            node: (),
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        },
        child_style: &child_style,
        area: GridArea {
            row: 0,
            column: 0,
            row_end: 1,
            column_end: 2,
            size: Size::new(80.0, 20.0),
        },
        content_box_size: Size::new(80.0, 20.0),
        columns: &[40.0, 40.0],
        rows: &[20.0],
        gap: Size::ZERO,
        parent_named_columns: &parent_named_columns,
        parent_named_rows: &parent_named_rows,
        parent_area_facts: None,
        parent_baseline_groups: &parent_baseline_groups,
        margin: Edges::all(Some(0.0)),
        border: Edges::ZERO,
        padding: Edges::ZERO,
        resolver: &NoCalcResolver,
    });

    let columns = context.columns.expect("column subgrid should inherit");
    assert_eq!(columns.major_baselines, vec![Some(8.0), Some(14.0)]);
    assert_eq!(columns.minor_baselines, vec![Some(3.0), Some(5.0)]);
}

#[test]
fn subgrid_track_inheritance_rejects_empty_parent_tracks() {
    let err = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[],
        parent_span: GridTrackSpan::new(1, 2),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Normal,
    })
    .unwrap_err();

    assert_eq!(err, SubgridTrackInheritanceError::EmptyTrackList);
}

#[test]
fn subgrid_track_inheritance_rejects_invalid_parent_spans() {
    for span in [
        GridTrackSpan::new(0, 2),
        GridTrackSpan::new(2, 2),
        GridTrackSpan::new(3, 2),
        GridTrackSpan::new(1, 4),
    ] {
        let err = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
            parent_tracks: &[10.0, 20.0],
            parent_span: span,
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: 0.0,
            subgrid_gap: ResolvedSubgridGap::Normal,
        })
        .unwrap_err();

        assert_eq!(err, SubgridTrackInheritanceError::SpanOutOfRange);
    }
}

fn traversal_leaf(node: u32, start: usize, end: usize) -> SubgridTraversalChild<u32> {
    SubgridTraversalChild::Leaf(SubgridTraversalLeaf {
        node,
        span_in_parent: GridTrackSpan::new(start, end),
        available_inline_size: None,
        available_inline_size_is_known: false,
    })
}

fn traversal_subgrid(
    node: u32,
    start: usize,
    end: usize,
    children: Vec<SubgridTraversalChild<u32>>,
) -> SubgridTraversalChild<u32> {
    SubgridTraversalChild::Subgrid(SubgridTraversalNode {
        node,
        axis: SubgridTraversalAxis::Inherited,
        reversed: false,
        span_in_parent: GridTrackSpan::new(start, end),
        available_inline_size: None,
        available_inline_size_is_known: false,
        queried_axis_fully_inherited: true,
        margins: SubgridAxisEdges::default(),
        border: SubgridAxisEdges::default(),
        padding: SubgridAxisEdges::default(),
        parent_gap: 0.0,
        subgrid_gap: 0.0,
        children,
    })
}

#[test]
fn subgrid_traversal_keeps_edge_lower_bounds_off_non_intrinsic_tracks() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[false, false]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 3),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges {
                start: 10.0,
                end: 12.0,
            },
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: Vec::new(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 0.0]);
}

#[test]
fn subgrid_traversal_places_edge_lower_bounds_in_ancestor_track_space() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(2, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges {
                start: 20.0,
                end: 30.0,
            },
            parent_gap: 20.0,
            subgrid_gap: 10.0,
            children: vec![traversal_leaf(2, 1, 2)],
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 20.0, 0.0, 30.0]);
}

#[test]
fn subgrid_traversal_reports_missing_intrinsic_min_facts() {
    let err = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Unknown,
        root_children: vec![traversal_subgrid(1, 1, 2, Vec::new())],
    })
    .unwrap_err();

    assert_eq!(err, SubgridTraversalError::MissingIntrinsicMinTrackFacts);
}

#[test]
fn subgrid_traversal_accumulates_edge_adjustment_in_nested_translated_span() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges {
                start: 2.0,
                end: 4.0,
            },
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
                node: 2,
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(2, 3),
                available_inline_size: None,
                available_inline_size_is_known: false,
                queried_axis_fully_inherited: true,
                margins: SubgridAxisEdges {
                    start: 3.0,
                    end: 5.0,
                },
                border: SubgridAxisEdges::default(),
                padding: SubgridAxisEdges::default(),
                parent_gap: 0.0,
                subgrid_gap: 0.0,
                children: vec![traversal_leaf(3, 1, 2)],
            })],
        })],
    })
    .unwrap();

    assert_eq!(report.leaves[0].ancestor_span, GridTrackSpan::new(2, 3));
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![2.0, 8.0, 4.0]
    );
}

#[test]
fn subgrid_traversal_accumulates_gap_adjustment_through_nested_subgrids() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
                node: 2,
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(2, 3),
                available_inline_size: None,
                available_inline_size_is_known: false,
                queried_axis_fully_inherited: true,
                margins: SubgridAxisEdges::default(),
                border: SubgridAxisEdges::default(),
                padding: SubgridAxisEdges::default(),
                parent_gap: 20.0,
                subgrid_gap: 28.0,
                children: vec![traversal_leaf(3, 1, 2)],
            })],
        })],
    })
    .unwrap();

    assert_eq!(
        report.leaves[0].accumulated_gap_adjustment,
        vec![5.0, 10.0, 5.0]
    );
}

#[test]
fn subgrid_traversal_applies_gap_adjustment_to_internal_edges() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: vec![traversal_leaf(2, 2, 3)],
        })],
    })
    .unwrap();

    assert_eq!(report.leaves[0].ancestor_span, GridTrackSpan::new(2, 3));
    assert_eq!(
        report.leaves[0].accumulated_gap_adjustment,
        vec![5.0, 10.0, 5.0]
    );
}

#[test]
fn subgrid_traversal_uses_positive_gap_adjustments_as_empty_track_lower_bounds() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 10.0,
            children: Vec::new(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![5.0, 10.0, 10.0, 5.0]);
}

#[test]
fn subgrid_traversal_combines_empty_edge_and_gap_lower_bounds() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 5),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges {
                start: 21.0,
                end: 9.0,
            },
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: Vec::new(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![26.0, 10.0, 10.0, 14.0]);
}

#[test]
fn subgrid_traversal_ignores_gap_adjustment_for_single_track_subgrid() {
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 2),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 30.0,
            children: vec![traversal_leaf(2, 1, 2)],
        })],
    })
    .unwrap();

    assert_eq!(report.leaves[0].accumulated_gap_adjustment, vec![0.0]);
}

#[test]
fn subgrid_traversal_rejects_standalone_subgrid_explicitly() {
    let err = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            node: 1,
            axis: SubgridTraversalAxis::Standalone,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 2),
            available_inline_size: None,
            available_inline_size_is_known: false,
            queried_axis_fully_inherited: true,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: vec![traversal_leaf(2, 1, 2)],
        })],
    })
    .unwrap_err();

    assert_eq!(
        err,
        SubgridTraversalError::StandaloneSubgridTraversalUnsupported
    );
}

#[test]
fn fr_span_contribution_distributes_by_flex_factor() {
    let tracks = [TrackSizing::fr(1.0), TrackSizing::fr(2.0)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [20.0, 40.0]);
}

#[test]
fn fr_span_contribution_subtracts_non_flex_base_tracks() {
    let tracks = [TrackSizing::MIN_CONTENT, TrackSizing::fr(1.0)];
    let mut sizes = [10.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        40.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [10.0, 30.0]);
}

#[test]
fn fr_span_contribution_normalizes_sub_one_factors() {
    let tracks = [TrackSizing::fr(0.2), TrackSizing::fr(0.3)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [24.0, 36.0]);
}

#[test]
fn fr_span_contribution_normalizes_sub_one_factors_after_non_flex_tracks() {
    let tracks = [
        TrackSizing::px(9.0),
        TrackSizing::fr(0.5),
        TrackSizing::fr(0.5),
    ];
    let mut sizes = [0.0, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        18.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [0.0, 4.5, 4.5]);
}

#[test]
fn fr_span_contribution_splits_zero_factors_evenly() {
    let tracks = [TrackSizing::fr(0.0), TrackSizing::fr(0.0)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [30.0, 30.0]);
}

#[test]
fn fr_span_contribution_keeps_indefinite_percent_tracks_for_initial_sizing() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::fit_content(Length::px(20.0)),
        TrackSizing::AUTO,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
        TrackSizing::fr(1.0),
        TrackSizing::fr(2.0),
    ];
    let mut sizes = [0.0; 8];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        160.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 50.0, 100.0]);
}

#[test]
fn fr_span_contribution_reserves_resolved_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::fit_content(Length::px(20.0)),
        TrackSizing::AUTO,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
        TrackSizing::fr(1.0),
        TrackSizing::fr(2.0),
    ];
    let mut sizes = [0.0; 8];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        Some(160.0),
        160.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 39.333332, 78.666664]);
}

#[test]
fn max_content_span_prefers_max_content_track() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
    ];
    let mut sizes = [80.0, 80.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        320.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [80.0, 230.0, 0.0]);
}

#[test]
fn max_content_span_prefers_max_content_track_over_min_content_maximum() {
    let tracks = [
        TrackSizing::MAX_CONTENT,
        TrackSizing::minmax(MinTrackSizing::MAX_CONTENT, MaxTrackSizing::MIN_CONTENT),
    ];
    let mut sizes = [40.0, 20.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        80.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [60.0, 20.0]);
}

#[test]
fn min_content_span_counts_indefinite_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [0.0, 0.0, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MinContent {
            prioritize_min_tracks: false,
        },
        None,
        160.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [42.666668, 42.666668, 0.0, 0.0]);
}

#[test]
fn max_content_span_keeps_indefinite_percent_tracks_for_initial_sizing() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [42.666668, 42.666668, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        320.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [42.666668, 267.3333, 0.0, 0.0]);
}

#[test]
fn max_content_span_reserves_resolved_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [42.666668, 42.666668, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        Some(320.0),
        320.0,
        &NoCalcResolver,
    );

    assert_eq!(sizes, [42.666668, 203.33333, 0.0, 0.0]);
}

#[test]
fn indefinite_flex_tracks_keep_span_resolved_bases() {
    let tracks = [TrackSizing::fr(1.0), TrackSizing::fr(2.0)];
    let sizes = resolve_tracks(
        &tracks,
        None,
        0.0,
        AlignContent::Start,
        &[20.0, 40.0],
        &NoCalcResolver,
    );

    assert_eq!(sizes, [20.0, 40.0]);
}

#[test]
fn inline_sub_one_flex_tracks_keep_non_spanned_track_proportional_to_used_fraction() {
    let tracks = [
        TrackSizing::fr(0.2),
        TrackSizing::fr(0.3),
        TrackSizing::fr(0.5),
    ];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: None,
        definite_size: None,
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[24.0, 36.0, 0.0],
        max_intrinsic_sizes: &[24.0, 36.0, 0.0],
    });

    assert_eq!(sizes, [24.0, 36.0, 9.0]);
}

#[test]
fn sub_one_flex_track_content_sum_includes_unfilled_fraction() {
    let tracks = [
        TrackSizing::fr(0.2),
        TrackSizing::fr(0.3),
        TrackSizing::fr(0.5),
    ];

    assert_eq!(track_content_sum(&tracks, &[24.0, 36.0, 9.0], 0.0), 78.0);
}

#[test]
fn tracks_shrink_between_min_and_max_bounds() {
    let sizes =
        distribute_tracks_between_bounds(&[40.0, 20.0, 40.0], &[40.0, 40.0, 40.0], 0.0, 110.0);

    assert_eq!(sizes, [40.0, 30.0, 40.0]);
}

#[test]
fn tracks_stop_shrinking_at_minimum_bounds() {
    let sizes =
        distribute_tracks_between_bounds(&[40.0, 20.0, 40.0], &[40.0, 40.0, 40.0], 0.0, 90.0);

    assert_eq!(sizes, [40.0, 20.0, 40.0]);
}

#[test]
fn inline_minmax_tracks_shrink_to_minimum_bounds() {
    let tracks = [
        TrackSizing::px(40.0),
        TrackSizing::minmax(MinTrackSizing::px(20.0), MaxTrackSizing::px(40.0)),
        TrackSizing::px(40.0),
    ];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: Some(90.0),
        definite_size: Some(90.0),
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[0.0, 0.0, 0.0],
        max_intrinsic_sizes: &[0.0, 0.0, 0.0],
    });

    assert_eq!(sizes, [40.0, 20.0, 40.0]);
}

#[test]
fn inline_minmax_tracks_interpolate_inside_bounds() {
    let tracks = [
        TrackSizing::px(40.0),
        TrackSizing::minmax(MinTrackSizing::px(20.0), MaxTrackSizing::px(40.0)),
        TrackSizing::px(40.0),
    ];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: Some(110.0),
        definite_size: Some(110.0),
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[0.0, 0.0, 0.0],
        max_intrinsic_sizes: &[0.0, 0.0, 0.0],
    });

    assert_eq!(sizes, [40.0, 30.0, 40.0]);
}

#[test]
fn inline_minmax_max_content_minimum_overrides_fixed_maximum() {
    let tracks = [TrackSizing::minmax(
        MinTrackSizing::MAX_CONTENT,
        MaxTrackSizing::px(10.0),
    )];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: None,
        definite_size: None,
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[20.0],
        max_intrinsic_sizes: &[40.0],
    });

    assert_eq!(sizes, [40.0]);
}

#[test]
fn inline_minmax_auto_minimum_allows_fixed_maximum() {
    let tracks = [TrackSizing::minmax(
        MinTrackSizing::AUTO,
        MaxTrackSizing::px(10.0),
    )];
    let sizes = resolve_inline_tracks(InlineTrackInput {
        resolver: &NoCalcResolver,
        tracks: &tracks,
        basis: None,
        definite_size: None,
        available_size: None,
        gap: 0.0,
        alignment: AlignContent::Start,
        stretch_empty_auto_to_available: false,
        min_intrinsic_sizes: &[20.0],
        max_intrinsic_sizes: &[40.0],
    });

    assert_eq!(sizes, [10.0]);
}

#[test]
fn definite_flex_tracks_respect_larger_base_tracks() {
    let tracks = [
        TrackSizing::px(40.0),
        TrackSizing::fr(1.0),
        TrackSizing::fr(1.0),
    ];
    let sizes = resolve_tracks(
        &tracks,
        Some(200.0),
        0.0,
        AlignContent::Start,
        &[0.0, 100.0, 0.0],
        &NoCalcResolver,
    );

    assert_eq!(sizes, [40.0, 100.0, 60.0]);
}

#[test]
fn grid_calc_percent_track_needs_layout_resolution() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(20.0),
        CalcTerm::percent(0.10),
    ]));
    let track = TrackSizing::new(
        MinTrackSizing::Length(Length::calc(id)),
        MaxTrackSizing::Length(Length::px(100.0)),
    );

    assert!(track.depends_on_basis_with(&store));
    assert_eq!(track.percent_fraction_with(&store), 0.10);
}
