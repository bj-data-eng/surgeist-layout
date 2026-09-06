use super::*;

fn layout_children<S: LayoutScalar>(
    axes: FlexAxes,
    flow: FlowAxes,
    main: f64,
    children: [NodeInputOf<S>; 2],
) -> [NodeOutputOf<S>; 2] {
    let size = axes.size_from_main_cross(S::from_f64(main), S::from_f64(100.0));
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: flow.writing_mode(),
                direction: flow.direction(),
                flex_direction: if axes.main_logical_axis() == LogicalAxis::Inline {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                },
                align_items: Some(AlignItems::Baseline),
                size: size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .style(1, children[0].clone())
        .style(2, children[1].clone());
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(size.map(AvailableOf::definite)).unwrap(),
    )
    .expect("valid flex layout succeeds");
    [1, 2].map(|node| {
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .expect("the child has staged output")
            .output()
    })
}

fn assert_inner_basis_shrink<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let axes = FlexAxes::new(flow, direction, FlexWrap::NoWrap);
        for (box_sizing, basis, padding, border) in [
            (BoxSizing::ContentBox, 100.0, 50.0, 0.0),
            (BoxSizing::BorderBox, 150.0, 50.0, 0.0),
            (BoxSizing::ContentBox, 100.0, 30.0, 20.0),
        ] {
            let mut first = NodeInputOf {
                box_sizing,
                flex_basis: FlexBasisOf::px(S::from_f64(basis)),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            };
            axes.set_main_start_edge(&mut first.padding, LengthOf::px(S::from_f64(padding)));
            axes.set_main_end_edge(&mut first.border, LengthOf::px(S::from_f64(border)));
            let second = NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(100.0)),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            };
            let outputs = layout_children(axes, flow, 200.0, [first, second]);
            assert_eq!(
                outputs.map(|output| axes.main_size(output.size)),
                [S::from_f64(125.0), S::from_f64(75.0)],
                "{direction:?} {box_sizing:?}: padding and border do not carry shrink weight"
            );
        }

        let mut first = NodeInputOf {
            box_sizing: BoxSizing::ContentBox,
            flex_basis: FlexBasisOf::px(S::ZERO),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        };
        axes.set_main_start_edge(&mut first.padding, LengthOf::px(S::from_f64(50.0)));
        let second = NodeInputOf {
            flex_basis: FlexBasisOf::px(S::from_f64(100.0)),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        };
        let outputs = layout_children(axes, flow, 100.0, [first, second]);
        assert_eq!(
            outputs.map(|output| axes.main_size(output.size)),
            [S::from_f64(50.0), S::from_f64(50.0)],
            "a zero inner basis preserves the padding floor without shrinking"
        );
    }
}

#[test]
fn flex_contract_inner_basis_shrink_f32() {
    assert_inner_basis_shrink::<f32>();
}

#[test]
fn flex_contract_inner_basis_shrink_f64() {
    assert_inner_basis_shrink::<f64>();
}

fn assert_freeze_before_initial_space<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let axes = FlexAxes::new(flow, direction, FlexWrap::NoWrap);
        let growing = [
            NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(100.0)),
                max_size: axes.with_main_size(
                    Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE),
                    MaxSizeOf::px(S::from_f64(50.0)),
                ),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(50.0)),
                flex_grow: FlexGrowOf::try_new(S::from_f64(0.5)).unwrap(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        ];
        let outputs = layout_children(axes, flow, 200.0, growing);
        assert_eq!(
            outputs.map(|output| axes.main_size(output.size)),
            [S::from_f64(50.0), S::from_f64(100.0)],
            "fractional growth uses the frozen target in initial free space"
        );
        let shrinking = [
            NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(50.0)),
                min_size: axes.with_main_size(
                    Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    MinSizeOf::px(S::from_f64(100.0)),
                ),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(150.0)),
                flex_shrink: FlexShrinkOf::try_new(S::from_f64(0.5)).unwrap(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        ];
        let outputs = layout_children(axes, flow, 150.0, shrinking);
        assert_eq!(
            outputs.map(|output| axes.main_size(output.size)),
            [S::from_f64(100.0), S::from_f64(100.0)],
            "fractional shrink uses the frozen target in initial free space"
        );
    }
}

#[test]
fn flex_contract_initial_free_space_f32() {
    assert_freeze_before_initial_space::<f32>();
}

#[test]
fn flex_contract_initial_free_space_f64() {
    assert_freeze_before_initial_space::<f64>();
}

fn assert_auto_margin_baseline_exclusion<S: LayoutScalar>() {
    for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalLr] {
        let flow = FlowAxes::new(writing_mode, Direction::Ltr);
        let axes = FlexAxes::new(flow, FlexDirection::Row, FlexWrap::NoWrap);
        for auto_start in [true, false] {
            for (first_cross, second_cross) in [(100.0, 20.0), (20.0, 100.0)] {
                let mut children = [first_cross, second_cross].map(|cross| NodeInputOf {
                    writing_mode,
                    size: axes
                        .size_from_main_cross(S::from_f64(50.0), S::from_f64(cross))
                        .map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                });
                if auto_start {
                    axes.set_cross_start_edge(&mut children[0].margin, LengthAutoOf::AUTO);
                } else {
                    axes.set_cross_end_edge(&mut children[0].margin, LengthAutoOf::AUTO);
                }
                let outputs = layout_children(axes, flow, 200.0, children);
                assert_eq!(
                    axes.cross_point(outputs[1].location),
                    S::ZERO,
                    "{writing_mode:?}: the auto-margin item does not contribute to the baseline group"
                );
                let first_offset = if auto_start { 100.0 - first_cross } else { 0.0 };
                assert_eq!(
                    axes.cross_point(outputs[0].location),
                    S::from_f64(first_offset),
                    "{writing_mode:?}: baseline alignment does not override the resolved auto margin"
                );
            }
        }
    }
}

#[test]
fn flex_contract_auto_margin_baseline_f32() {
    assert_auto_margin_baseline_exclusion::<f32>();
}

#[test]
fn flex_contract_auto_margin_baseline_f64() {
    assert_auto_margin_baseline_exclusion::<f64>();
}

fn assert_auto_margin_does_not_expand_baseline_line<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = OracleTreeOf::new().children(0, [1, 2]).style(
        0,
        NodeInputOf {
            display: Display::Flex,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(200.0)),
                PreferredSizeOf::AUTO,
            ),
            align_items: Some(AlignItems::Baseline),
            ..NodeInputOf::default()
        },
    );
    for (node, height, baseline) in [(1, 100.0, 100.0), (2, 20.0, 5.0)] {
        let size = Size::new(S::from_f64(50.0), S::from_f64(height));
        tree = tree
            .children(node, [])
            .style(
                node,
                NodeInputOf {
                    size: size.map(PreferredSizeOf::px),
                    margin: Edges {
                        top: if node == 1 {
                            LengthAutoOf::AUTO
                        } else {
                            LengthAutoOf::ZERO
                        },
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..NodeInputOf::default()
                },
            )
            .measure(
                node,
                ComputeOutputOf::from_sizes_and_baselines(
                    size,
                    size,
                    BaselinesOf::first(Point::new(None, Some(S::from_f64(baseline)))),
                ),
            );
    }
    let output = compute_flex(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(200.0)), None),
            ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
            Size::new(
                AvailableOf::definite(S::from_f64(200.0)),
                AvailableOf::MAX_CONTENT,
            ),
        ),
    )
    .expect("baseline line layout succeeds");
    assert_eq!(
        output.size.height,
        S::from_f64(100.0),
        "an excluded baseline must not expand the line to 115px"
    );
    assert_eq!(tree.layout(2).unwrap().location.y, S::ZERO);
}

#[test]
fn flex_contract_auto_margin_line_size_f32() {
    assert_auto_margin_does_not_expand_baseline_line::<f32>();
}

#[test]
fn flex_contract_auto_margin_line_size_f64() {
    assert_auto_margin_does_not_expand_baseline_line::<f64>();
}
