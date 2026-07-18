use std::collections::HashMap;
use std::collections::HashSet;

use crate::block::resolve_logical_in_flow_margin;
use crate::*;

fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

#[test]
fn block_child_context_is_complete_for_layout_sizing_and_absolute_paths() {
    assert_block_child_context_is_complete::<f32>();
    assert_block_child_context_is_complete::<f64>();
}

fn assert_block_child_context_is_complete<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let expected =
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::BlockFlow);

    for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalRl,
                    direction: Direction::Rtl,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .style(1, NodeInputOf::default())
            .style(
                2,
                NodeInputOf {
                    position: Position::Absolute,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(30.0)),
                        PreferredSizeOf::px(S::from_f64(12.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .measure(
                1,
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(40.0), S::from_f64(20.0))),
            )
            .measure(
                2,
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(30.0), S::from_f64(12.0))),
            );

        crate::compute_block(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                run_mode,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(300.0)), Some(S::from_f64(240.0))),
                crate::ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(AvailableOf::definite(S::from_f64(300.0))),
            ),
        )
        .expect("block context capture layout succeeds");

        let normal_inputs = tree.inputs(1);
        assert!(
            !normal_inputs.is_empty(),
            "block must request its in-flow child"
        );
        assert!(
            normal_inputs
                .iter()
                .all(|input| input.containing_layout_context() == expected),
            "every block in-flow request must use the parent axes and BlockFlow role: {normal_inputs:#?}"
        );

        if run_mode == RunMode::ComputeSize {
            assert!(
                normal_inputs.iter().any(|input| {
                    input.run_mode() == RunMode::ComputeSize
                        && input.sizing_mode() == SizingMode::InherentSize
                }),
                "block intrinsic sizing must request the child through the complete context"
            );
        } else {
            assert!(
                normal_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::PerformLayout),
                "block normal layout must request the child through the complete context"
            );
            let absolute_inputs = tree.inputs(2);
            assert!(
                absolute_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::PerformLayout),
                "block absolute scheduling must request the child"
            );
            assert!(
                absolute_inputs
                    .iter()
                    .all(|input| input.containing_layout_context() == expected),
                "every block absolute request must use the parent axes and BlockFlow role: {absolute_inputs:#?}"
            );
        }
    }
}

fn assert_positive_physical_range(range: PhysicalScrollRange, maximum: Size) {
    assert_eq!(range.x().minimum(), 0.0);
    assert_eq!(range.x().maximum(), maximum.width);
    assert_eq!(range.y().minimum(), 0.0);
    assert_eq!(range.y().maximum(), maximum.height);
}

fn lp(absolute_px: Scalar, percent_fraction: Scalar) -> LengthPercentageOf {
    LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
        .expect("test coefficients are finite")
}

fn lp64(absolute_px: f64, percent_fraction: f64) -> LengthPercentageOf<f64> {
    LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
        .expect("test coefficients are finite")
}

#[derive(Default)]
struct PublicBlockTree<S: LayoutScalar> {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInputOf<S>>,
    leaf_nodes: HashSet<u32>,
    leaf_measurements: HashMap<u32, Size<S>>,
}

impl<S: LayoutScalar> PublicBlockTree<S> {
    fn with_children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    fn with_style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.styles.insert(node, style);
        self
    }

    fn with_measurement(mut self, node: u32, size: Size<S>) -> Self {
        self.leaf_nodes.insert(node);
        self.leaf_measurements.insert(node, size);
        self
    }
}

impl<S: LayoutScalar> Traverse for PublicBlockTree<S> {
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

impl<S: LayoutScalar> LayoutTree for PublicBlockTree<S> {
    type MeasureError = ();

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.styles[&node].clone())
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.leaf_nodes.contains(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        _input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.leaf_measurements.get(&node).copied().map(Ok)
    }
}

fn public_final_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .expect("public layout batch contains the requested node")
        .output()
}

fn scalar_value<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn scalar_percentage<S: LayoutScalar>(
    absolute_px: f64,
    percent_fraction: f64,
) -> LengthPercentageOf<S> {
    LengthPercentageOf::from_coefficients(scalar_value(absolute_px), scalar_value(percent_fraction))
        .expect("test coefficients are finite")
}

fn fri04_c03_block_positioned_value<S: LayoutScalar>(value: f64) -> SizingCalculationOf<S> {
    SizingCalculationOf::value(
        LengthPercentageOf::px(scalar_value(value)).expect("test sizing value is finite"),
    )
}

fn fri04_c03_block_positioned_nested<S: LayoutScalar>(
    minimum: f64,
    preferred: f64,
    maximum: f64,
) -> SizingCalculationOf<S> {
    let preferred = SizingCalculationOf::max(vec![
        fri04_c03_block_positioned_value(preferred),
        SizingCalculationOf::min(vec![
            fri04_c03_block_positioned_value(preferred - 5.0),
            fri04_c03_block_positioned_value(preferred + 5.0),
        ])
        .expect("nested minimum is nonempty"),
    ])
    .expect("nested maximum is nonempty");
    SizingCalculationOf::clamp(
        Some(fri04_c03_block_positioned_value(minimum)),
        preferred,
        Some(fri04_c03_block_positioned_value(maximum)),
    )
}

#[test]
fn fri04_c03_block_positioned_ordinary_block_consumes_nested_constraints_and_non_negative_results()
{
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(160.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        20.0, 80.0, 120.0,
                    )),
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        20.0, 70.0, 120.0,
                    )),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_block_positioned_nested(40.0, 90.0, 110.0)),
                    MinSize::calculation(fri04_c03_block_positioned_nested(30.0, 60.0, 90.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_block_positioned_nested(30.0, 85.0, 100.0)),
                    MaxSize::calculation(fri04_c03_block_positioned_nested(30.0, 65.0, 100.0)),
                ),
                ..NodeInput::default()
            },
        )
        .with_style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        -40.0, -20.0, -10.0,
                    )),
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        -30.0, -15.0, -5.0,
                    )),
                ),
                ..NodeInput::default()
            },
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(300.0)))
            .expect("valid viewport"),
    )
    .expect("ordinary block calculations resolve");

    assert_eq!(public_final_output(&batch, 1).size, Size::new(90.0, 65.0));
    assert_eq!(public_final_output(&batch, 2).size, Size::ZERO);
}

#[test]
fn fri04_c03_block_positioned_absolute_consumes_nested_properties_and_inset_derived_sizing() {
    let absolute =
        |size: Size<PreferredSize>, min_size: Size<MinSize>, max_size: Size<MaxSize>, inset| {
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                size,
                min_size,
                max_size,
                inset,
                ..NodeInput::default()
            }
        };
    let nested_size = Size::new(
        PreferredSize::calculation(fri04_c03_block_positioned_nested(20.0, 80.0, 120.0)),
        PreferredSize::calculation(fri04_c03_block_positioned_nested(20.0, 70.0, 120.0)),
    );
    let nested_min = Size::new(
        MinSize::calculation(fri04_c03_block_positioned_nested(30.0, 60.0, 90.0)),
        MinSize::calculation(fri04_c03_block_positioned_nested(30.0, 50.0, 80.0)),
    );
    let nested_max = Size::new(
        MaxSize::calculation(fri04_c03_block_positioned_nested(40.0, 75.0, 100.0)),
        MaxSize::calculation(fri04_c03_block_positioned_nested(40.0, 65.0, 90.0)),
    );
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2, 3])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(160.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            absolute(
                nested_size,
                nested_min.clone(),
                nested_max.clone(),
                Edges::all(LengthAuto::AUTO),
            ),
        )
        .with_style(
            2,
            absolute(
                Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
                nested_min,
                nested_max,
                Edges {
                    top: LengthAuto::px(10.0),
                    right: LengthAuto::px(20.0),
                    bottom: LengthAuto::px(10.0),
                    left: LengthAuto::px(20.0),
                },
            ),
        )
        .with_style(
            3,
            absolute(
                Size::new(
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        -40.0, -20.0, -10.0,
                    )),
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        -30.0, -15.0, -5.0,
                    )),
                ),
                Size::new(MinSize::AUTO, MinSize::AUTO),
                Size::new(MaxSize::NONE, MaxSize::NONE),
                Edges::all(LengthAuto::AUTO),
            ),
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(300.0)))
            .expect("valid viewport"),
    )
    .expect("positioned calculations resolve");

    assert_eq!(public_final_output(&batch, 1).size, Size::new(75.0, 65.0));
    assert_eq!(public_final_output(&batch, 2).size, Size::new(75.0, 65.0));
    assert_eq!(
        public_final_output(&batch, 2).location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(public_final_output(&batch, 3).size, Size::ZERO);
}

#[test]
fn fri04_c03_block_positioned_compute_size_preserves_missing_basis_as_indefinite() {
    let percentage = SizingCalculation::max(vec![
        fri04_c03_block_positioned_value(10.0),
        SizingCalculation::value(
            LengthPercentageOf::from_percent_fraction(0.5).expect("finite percentage"),
        ),
    ])
    .expect("nested maximum is nonempty");
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(
                    PreferredSize::calculation(percentage.clone()),
                    PreferredSize::calculation(percentage.clone()),
                ),
                min_size: Size::new(
                    MinSize::calculation(percentage.clone()),
                    MinSize::calculation(percentage.clone()),
                ),
                max_size: Size::new(
                    MaxSize::calculation(percentage.clone()),
                    MaxSize::calculation(percentage),
                ),
                ..NodeInput::default()
            },
        )
        .measure(1, ComputeOutput::from_outer_size(Size::new(30.0, 20.0)));

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .expect("intrinsic block sizing retains the missing-basis fallback");

    assert_eq!(output.size, Size::new(30.0, 20.0));
    assert!(
        tree.inputs(1).iter().any(|input| {
            input.run_mode() == RunMode::ComputeSize && input.parent() == Size::NONE
        })
    );
}

#[test]
fn fri04_c03_block_positioned_invalid_numeric_propagates_from_both_consumers() {
    let invalid = || {
        SizingCalculation::min(vec![
            SizingCalculation::value(
                LengthPercentageOf::from_coefficients(f32::MAX, 1.0)
                    .expect("finite overflowing coefficients"),
            ),
            fri04_c03_block_positioned_value(10.0),
        ])
        .expect("nested minimum is nonempty")
    };

    for position in [Position::Relative, Position::Absolute] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, NodeInput::default())
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    position,
                    size: Size::new(
                        PreferredSize::calculation(invalid()),
                        PreferredSize::px(10.0),
                    ),
                    ..NodeInput::default()
                },
            );
        let request = LayoutRootRequest::viewport(Size::new(
            Available::definite(f32::MAX),
            Available::definite(80.0),
        ))
        .expect("largest finite viewport is valid");

        let error = compute_layout(&tree, 0, request)
            .expect_err("invalid numeric sizing must return no completed batch");
        assert_eq!(error.site(), LayoutErrorSite::Node(1));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert_eq!(
            error.kind(),
            &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
                value: f32::INFINITY,
            })
        );
    }
}

enum Fri04C04SizingValue {
    Preferred(PreferredSize),
    Minimum(MinSize),
    Maximum(MaxSize),
}

fn fri04_c04_leaf_block_positioned_style(
    value: Fri04C04SizingValue,
    position: Position,
    axis: PhysicalAxis,
) -> NodeInput {
    let mut style = NodeInput {
        display: Display::Block,
        position,
        ..NodeInput::default()
    };
    match (value, axis) {
        (Fri04C04SizingValue::Preferred(value), PhysicalAxis::Horizontal) => {
            style.size.width = value;
        }
        (Fri04C04SizingValue::Preferred(value), PhysicalAxis::Vertical) => {
            style.size.height = value;
        }
        (Fri04C04SizingValue::Minimum(value), PhysicalAxis::Horizontal) => {
            style.min_size.width = value;
        }
        (Fri04C04SizingValue::Minimum(value), PhysicalAxis::Vertical) => {
            style.min_size.height = value;
        }
        (Fri04C04SizingValue::Maximum(value), PhysicalAxis::Horizontal) => {
            style.max_size.width = value;
        }
        (Fri04C04SizingValue::Maximum(value), PhysicalAxis::Vertical) => {
            style.max_size.height = value;
        }
    }
    style
}

fn fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
    value: Fri04C04SizingValue,
    property: SizingProperty,
    behavior: SizingBehavior,
    axis: PhysicalAxis,
    position: Position,
) {
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(200.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            fri04_c04_leaf_block_positioned_style(value, position, axis),
        );
    let error = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(200.0)))
            .expect("valid viewport"),
    )
    .expect_err("later-owned block sizing must be rejected");
    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected sizing capability, got {:?}", error.kind());
    };
    assert_eq!(
        (
            unsupported.property(),
            unsupported.behavior(),
            unsupported.algorithm(),
            unsupported.axis(),
        ),
        (
            property,
            behavior,
            if position == Position::Absolute {
                SizingAlgorithm::Positioned
            } else {
                SizingAlgorithm::Block
            },
            axis,
        )
    );
}

#[test]
fn fri04_c04_leaf_block_positioned_block_and_absolute_cover_all_unsupported_states() {
    let sizing = || {
        SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite sizing calculation"))
    };
    let calc = || CalcSizeCalculation::value(LengthPercentageOf::ZERO);

    for position in [Position::Relative, Position::Absolute] {
        for (value, behavior) in [
            (PreferredSize::STRETCH, SizingBehavior::Stretch),
            (PreferredSize::FIT_CONTENT, SizingBehavior::FitContent),
            (PreferredSize::CONTAIN, SizingBehavior::Contain),
            (
                PreferredSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Preferred(value),
                SizingProperty::Preferred,
                behavior,
                PhysicalAxis::Horizontal,
                position,
            );
        }
        if position == Position::Absolute {
            for (value, behavior) in [
                (PreferredSize::MIN_CONTENT, SizingBehavior::MinContent),
                (PreferredSize::MAX_CONTENT, SizingBehavior::MaxContent),
            ] {
                fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                    Fri04C04SizingValue::Preferred(value),
                    SizingProperty::Preferred,
                    behavior,
                    PhysicalAxis::Vertical,
                    position,
                );
            }
        }
        for (value, behavior) in [
            (MinSize::MIN_CONTENT, SizingBehavior::MinContent),
            (MinSize::MAX_CONTENT, SizingBehavior::MaxContent),
            (MinSize::STRETCH, SizingBehavior::Stretch),
            (MinSize::FIT_CONTENT, SizingBehavior::FitContent),
            (MinSize::CONTAIN, SizingBehavior::Contain),
            (
                MinSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Minimum(value),
                SizingProperty::Minimum,
                behavior,
                PhysicalAxis::Vertical,
                position,
            );
        }
        for (value, behavior) in [
            (MaxSize::MIN_CONTENT, SizingBehavior::MinContent),
            (MaxSize::MAX_CONTENT, SizingBehavior::MaxContent),
            (MaxSize::STRETCH, SizingBehavior::Stretch),
            (MaxSize::FIT_CONTENT, SizingBehavior::FitContent),
            (MaxSize::CONTAIN, SizingBehavior::Contain),
            (
                MaxSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Maximum(value),
                SizingProperty::Maximum,
                behavior,
                PhysicalAxis::Horizontal,
                position,
            );
        }

        for (basis, expected) in [
            (PreferredSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
            (
                PreferredSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                PreferredSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (
                PreferredSizeCalcBasis::Stretch,
                CalcSizeBehaviorBasis::Stretch,
            ),
            (
                PreferredSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (
                PreferredSizeCalcBasis::Contain,
                CalcSizeBehaviorBasis::Contain,
            ),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Preferred(
                    PreferredSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                SizingProperty::Preferred,
                SizingBehavior::CalcSize(expected),
                PhysicalAxis::Vertical,
                position,
            );
        }
        for (basis, expected) in [
            (MinSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
            (
                MinSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                MinSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (MinSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
            (
                MinSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (MinSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Minimum(
                    MinSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                SizingProperty::Minimum,
                SizingBehavior::CalcSize(expected),
                PhysicalAxis::Horizontal,
                position,
            );
        }
        for (basis, expected) in [
            (MaxSizeCalcBasis::None, CalcSizeBehaviorBasis::None),
            (
                MaxSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                MaxSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (MaxSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
            (
                MaxSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (MaxSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Maximum(
                    MaxSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                SizingProperty::Maximum,
                SizingBehavior::CalcSize(expected),
                PhysicalAxis::Vertical,
                position,
            );
        }
    }
}

#[test]
fn fri04_c04_leaf_block_positioned_block_and_absolute_calc_size_geometry() {
    let preferred = || {
        Size::new(
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::Any,
                CalcSizeCalculation::from_coefficients(20.0, 0.5, 0.0)
                    .expect("finite Any calculation"),
            )
            .expect("valid Any calc-size"),
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::FullPercentage,
                CalcSizeCalculation::from_coefficients(10.0, 0.0, 0.5)
                    .expect("finite FullPercentage calculation"),
            )
            .expect("valid FullPercentage calc-size"),
        )
    };
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(160.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: preferred(),
                ..NodeInput::default()
            },
        )
        .with_style(
            2,
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                size: preferred(),
                ..NodeInput::default()
            },
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::new(
            Available::definite(200.0),
            Available::definite(160.0),
        ))
        .expect("valid viewport"),
    )
    .expect("supported block and positioned calc-size values resolve");

    assert_eq!(public_final_output(&batch, 1).size, Size::new(120.0, 90.0));
    assert_eq!(public_final_output(&batch, 2).size, Size::new(120.0, 90.0));
}

#[test]
fn fri04_c04_leaf_block_positioned_absolute_grid_and_block_inner_displays_are_positioned() {
    for display in [Display::Block, Display::Grid] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display,
                    position: Position::Absolute,
                    size: Size::new(PreferredSize::STRETCH, PreferredSize::AUTO),
                    ..NodeInput::default()
                },
            );
        let error = compute_layout(
            &tree,
            0,
            LayoutRootRequest::viewport(Size::splat(Available::definite(100.0)))
                .expect("valid viewport"),
        )
        .expect_err("absolute sizing must reject stretch before inner display dispatch");
        let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
            unsupported,
        )) = error.kind()
        else {
            panic!("expected positioned sizing capability");
        };
        assert_eq!(unsupported.algorithm(), SizingAlgorithm::Positioned);
        assert_eq!(unsupported.axis(), PhysicalAxis::Horizontal);
        assert_eq!(error.site(), LayoutErrorSite::Node(1));
    }
}

#[test]
fn parent_context_gates_only_block_boundary_collapse_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>()
    where
        crate::test_support::layout_tree::OracleTreeOf<S>:
            Compute + Traverse<Node = u32, Scalar = S>,
    {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        for (parent_context, expected_collapse) in [
            (ParentFormattingContext::BlockFlow, true),
            (ParentFormattingContext::Flex, false),
            (ParentFormattingContext::Grid, false),
            (ParentFormattingContext::NoParent, false),
        ] {
            let mut child_output =
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(40.0), S::ZERO));
            child_output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
                flow_axes,
                CollapsibleMarginOf::from_margin(S::from_f64(3.0)),
                CollapsibleMarginOf::from_margin(S::from_f64(5.0)),
                true,
            );
            let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        size: Size::new(
                            PreferredSizeOf::px(S::from_f64(40.0)),
                            PreferredSizeOf::AUTO,
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        margin: Edges::new(
                            LengthAutoOf::px(S::from_f64(3.0)),
                            LengthAutoOf::ZERO,
                            LengthAutoOf::px(S::from_f64(5.0)),
                            LengthAutoOf::ZERO,
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .measure(1, child_output);
            let output = crate::compute_block(
                &mut tree,
                0,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(100.0))),
                    ContainingLayoutContext::new(flow_axes, parent_context),
                    Size::new(
                        AvailableOf::definite(S::from_f64(100.0)),
                        AvailableOf::MAX_CONTENT,
                    ),
                ),
            )
            .expect("block layout succeeds");

            let collapse = output.block_margin_collapse;
            assert_eq!(
                collapse.at(flow_axes.block_start()).resolve(),
                if expected_collapse {
                    S::from_f64(3.0)
                } else {
                    S::ZERO
                },
                "unexpected block-start collapse for {parent_context:?}"
            );
            assert_eq!(
                collapse.at(flow_axes.block_end()).resolve(),
                if expected_collapse {
                    S::from_f64(5.0)
                } else {
                    S::ZERO
                },
                "unexpected block-end collapse for {parent_context:?}"
            );
            assert_eq!(
                collapse.can_collapse_through(flow_axes),
                expected_collapse,
                "unexpected boundary collapse for {parent_context:?}"
            );
        }

        let mut root_tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [])
            .style(0, NodeInputOf::default());
        let root_output = crate::compute_block(
            &mut root_tree,
            0,
            ComputeInputOf::root_layout(
                Size::NONE,
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(100.0))),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::BlockFlow),
                Size::splat(AvailableOf::definite(S::from_f64(100.0))),
            ),
        )
        .expect("root-mode block layout succeeds");
        assert!(
            !root_output
                .block_margin_collapse
                .can_collapse_through(flow_axes)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn replaced_block_child_keeps_measured_auto_inline_size_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar_value::<S>;
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(scalar(200.0)), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    item_is_replaced: true,
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .with_measurement(1, Size::new(scalar(50.0), scalar(10.0)))
            .with_measurement(2, Size::new(scalar(50.0), scalar(10.0)));
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(scalar(200.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .expect("finite viewport request"),
        )
        .expect("measured block children lay out");

        assert_eq!(public_final_output(&batch, 1).size.width, scalar(50.0));
        assert_eq!(public_final_output(&batch, 2).size.width, scalar(200.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn block_layout_ignores_item_order_for_geometry() {
    let layout = |item_orders: [ItemOrder; 3]| {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::splat_clone(PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[0],
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                2,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[1],
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                3,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[2],
                    size: Size::new(PreferredSize::px(30.0), PreferredSize::px(30.0)),
                    ..NodeInput::default()
                },
            );
        let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0)))
            .expect("finite viewport is valid");
        let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

        [
            public_final_output(&batch, 1),
            public_final_output(&batch, 2),
            public_final_output(&batch, 3),
        ]
    };

    let source_order = layout([ItemOrder::ZERO; 3]);
    let non_default_order = layout([ItemOrder::new(7), ItemOrder::new(-3), ItemOrder::new(2)]);

    assert_eq!(non_default_order, source_order);
    assert_eq!(
        non_default_order.map(|output| (output.source_index, output.location)),
        [
            (SourceIndex::new(0), Point::new(0.0, 0.0)),
            (SourceIndex::new(1), Point::new(0.0, 10.0)),
            (SourceIndex::new(2), Point::new(0.0, 30.0)),
        ]
    );
}

fn assert_ordinary_block_flow<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    expected_first: Point<S>,
    expected_second: Point<S>,
) {
    let scalar = scalar_value::<S>;
    let child_style = NodeInputOf {
        display: Display::Block,
        writing_mode,
        direction,
        size: Size::new(
            PreferredSizeOf::px(scalar(20.0)),
            PreferredSizeOf::px(scalar(10.0)),
        ),
        ..NodeInputOf::default()
    };
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, child_style.clone())
        .with_style(2, child_style);
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
        .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

    assert_eq!(public_final_output(&batch, 1).location, expected_first);
    assert_eq!(public_final_output(&batch, 2).location, expected_second);
}

#[test]
fn ordinary_block_flow_uses_logical_block_progression_for_f32() {
    assert_ordinary_block_flow::<f32>(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::HorizontalTb,
        Direction::Rtl,
        Point::new(80.0, 0.0),
        Point::new(80.0, 10.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalLr,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalLr,
        Direction::Rtl,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysLr,
        Direction::Ltr,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysLr,
        Direction::Rtl,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
}

#[test]
fn ordinary_block_flow_uses_logical_block_progression_for_f64() {
    assert_ordinary_block_flow::<f64>(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::HorizontalTb,
        Direction::Rtl,
        Point::new(80.0, 0.0),
        Point::new(80.0, 10.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalLr,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalLr,
        Direction::Rtl,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysLr,
        Direction::Ltr,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysLr,
        Direction::Rtl,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
}

fn all_writing_mode_directions() -> [(WritingMode, Direction); 10] {
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

fn assert_ordinary_block_boundaries<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let container_size = Size::new(scalar(100.0), scalar(100.0));
    let child_logical_size = crate::geometry::LogicalSizeOf::new(scalar(20.0), scalar(10.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let child_size = flow_axes.physical_size(child_logical_size);
        let relative_inset = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            LengthAutoOf::px(scalar(3.0)),
            LengthAutoOf::AUTO,
            LengthAutoOf::px(scalar(5.0)),
            LengthAutoOf::AUTO,
        ));
        let relative_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(scalar(3.0), scalar(5.0)),
            child_logical_size,
            container_size,
        );
        let relative_tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    position: Position::Relative,
                    size: child_size.map(PreferredSizeOf::px),
                    inset: relative_inset,
                    ..NodeInputOf::default()
                },
            );
        let request =
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid");
        let relative =
            compute_layout(&relative_tree, 0, request).expect("relative block layout succeeds");

        assert_eq!(
            public_final_output(&relative, 1).location,
            relative_expected
        );

        let inline_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(S::ZERO, scalar(10.0)),
            child_logical_size,
            container_size,
        );
        let inline_tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let request =
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid");
        let inline =
            compute_layout(&inline_tree, 0, request).expect("inline block layout succeeds");

        assert_eq!(public_final_output(&inline, 2).location, inline_expected);

        let static_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(S::ZERO, scalar(10.0)),
            child_logical_size,
            container_size,
        );
        let static_tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    position: Position::Absolute,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let request =
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid");
        let static_position =
            compute_layout(&static_tree, 0, request).expect("static fallback layout succeeds");

        assert_eq!(
            public_final_output(&static_position, 2).location,
            static_expected
        );
    }
}

#[test]
fn ordinary_block_boundaries_project_through_containing_flow_for_f32() {
    assert_ordinary_block_boundaries::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_through_containing_flow_for_f64() {
    assert_ordinary_block_boundaries::<f64>();
}

fn inline_run_baseline_point<S: LayoutScalar>(
    flow_axes: crate::geometry::FlowAxes,
    location: Point<S>,
    size: Size<S>,
    side: crate::PhysicalSide,
) -> Point<Option<S>> {
    let coordinate = match side {
        crate::PhysicalSide::Top | crate::PhysicalSide::Left => S::ZERO,
        crate::PhysicalSide::Right => size.width,
        crate::PhysicalSide::Bottom => size.height,
    };
    match flow_axes.block_axis() {
        crate::PhysicalAxis::Horizontal => Point::new(Some(location.x + coordinate), None),
        crate::PhysicalAxis::Vertical => Point::new(None, Some(location.y + coordinate)),
    }
}

fn assert_ordinary_block_boundary_baselines<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let container_size = Size::new(S::from_f64(100.0), S::from_f64(100.0));
    let logical_size = crate::geometry::LogicalSizeOf::new(S::from_f64(20.0), S::from_f64(10.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let child_size = flow_axes.physical_size(logical_size);
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::px(S::from_f64(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let output = crate::compute_block(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                container_size.map(Some),
                crate::ContainingLayoutContext::new(
                    flow_axes,
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(AvailableOf::definite(S::from_f64(100.0))),
            ),
        )
        .expect("block layout succeeds");

        let (expected_first, expected_last) = if flow_axes.block_axis()
            == crate::PhysicalAxis::Horizontal
        {
            let location = flow_axes.physical_point(
                crate::geometry::LogicalPointOf::new(S::ZERO, S::from_f64(10.0)),
                logical_size,
                container_size,
            );
            (
                inline_run_baseline_point(flow_axes, location, child_size, flow_axes.line_under()),
                inline_run_baseline_point(flow_axes, location, child_size, flow_axes.line_over()),
            )
        } else {
            let baseline = Some(S::from_f64(20.0));
            (Point::new(None, baseline), Point::new(None, baseline))
        };
        assert_eq!(output.first_baselines, expected_first);
        assert_eq!(output.last_baselines, expected_last);
    }
}

#[test]
fn ordinary_block_boundaries_project_inline_baselines_for_f32() {
    assert_ordinary_block_boundary_baselines::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_inline_baselines_for_f64() {
    assert_ordinary_block_boundary_baselines::<f64>();
}

fn assert_ordinary_block_boundary_inline_report_overflow<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let root_size = Size::new(scalar(40.0), scalar(100.0));

    for (writing_mode, direction) in all_writing_mode_directions()
        .into_iter()
        .filter(|(writing_mode, _)| *writing_mode != WritingMode::HorizontalTb)
    {
        let expected_scrollable_overflow = match writing_mode {
            WritingMode::VerticalRl | WritingMode::SidewaysRl => ScrollRectOf::try_new(
                Point::new(scalar(-60.0), S::ZERO),
                Size::new(scalar(100.0), scalar(100.0)),
            )
            .expect("finite expected overflow rectangle"),
            WritingMode::VerticalLr | WritingMode::SidewaysLr => {
                ScrollRectOf::try_new(Point::ZERO, Size::new(scalar(40.0), scalar(100.0)))
                    .expect("finite expected overflow rectangle")
            }
            WritingMode::HorizontalTb => unreachable!("horizontal flow is filtered above"),
        };
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    text_align: TextAlign::LegacyCenter,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: root_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::InlineBlock,
                    writing_mode,
                    direction,
                    size: Size::splat_clone(PreferredSizeOf::px(scalar(20.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid"),
        )
        .expect("inline run layout succeeds");
        let root = public_final_output(&batch, 0);

        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
        assert_eq!(
            root.scroll_geometry
                .expect("root always has scroll geometry")
                .scrollable_overflow(),
            expected_scrollable_overflow,
        );
    }
}

#[test]
fn ordinary_block_boundaries_project_vertical_and_sideways_inline_report_overflow_for_f32() {
    assert_ordinary_block_boundary_inline_report_overflow::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_vertical_and_sideways_inline_report_overflow_for_f64() {
    assert_ordinary_block_boundary_inline_report_overflow::<f64>();
}

fn assert_ordinary_block_boundaries_keep_inline_content_coordinates<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let root_size = Size::new(scalar(50.0), scalar(50.0));
    let padding = Edges::new(
        LengthOf::px(scalar(2.0)),
        LengthOf::px(scalar(3.0)),
        LengthOf::px(scalar(5.0)),
        LengthOf::px(scalar(7.0)),
    );
    let border = Edges::new(
        LengthOf::px(scalar(1.0)),
        LengthOf::px(scalar(2.0)),
        LengthOf::px(scalar(3.0)),
        LengthOf::px(scalar(4.0)),
    );
    let expected_content_size = Size::new(scalar(40.0), scalar(45.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: root_size.map(PreferredSizeOf::px),
                    padding,
                    border,
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::InlineBlock,
                    writing_mode,
                    direction,
                    size: expected_content_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(50.0))))
                .expect("finite viewport is valid"),
        )
        .expect("padded inline block layout succeeds");
        let root = public_final_output(&batch, 0);
        let expected_scrollable_overflow = match (writing_mode, direction) {
            (WritingMode::HorizontalTb, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::HorizontalTb, Direction::Rtl)
            | (WritingMode::VerticalRl, Direction::Ltr)
            | (WritingMode::SidewaysRl, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(-2.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalRl, Direction::Rtl)
            | (WritingMode::SidewaysRl, Direction::Rtl) => ScrollRectOf::try_new(
                Point::new(scalar(-2.0), scalar(-5.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalLr, Direction::Ltr)
            | (WritingMode::SidewaysLr, Direction::Rtl) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalLr, Direction::Rtl)
            | (WritingMode::SidewaysLr, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(-5.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
        }
        .expect("finite expected scrollable overflow");

        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
        assert_eq!(
            root.scroll_geometry
                .expect("root always has scroll geometry")
                .scrollable_overflow(),
            expected_scrollable_overflow,
        );
    }
}

#[test]
fn ordinary_block_boundaries_keep_padded_inline_content_coordinates_for_f32() {
    assert_ordinary_block_boundaries_keep_inline_content_coordinates::<f32>();
}

#[test]
fn ordinary_block_boundaries_keep_padded_inline_content_coordinates_for_f64() {
    assert_ordinary_block_boundaries_keep_inline_content_coordinates::<f64>();
}

fn assert_ordinary_block_boundaries_preserve_physical_float_bfc_cursor<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;

    for writing_mode in [WritingMode::VerticalRl, WritingMode::VerticalLr] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    clear: Clear::Left,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
                .expect("finite viewport is valid"),
        )
        .expect("vertical float and BFC layout succeeds");

        assert_eq!(public_final_output(&batch, 2).location.y, scalar(20.0));
        assert_eq!(public_final_output(&batch, 3).location.y, scalar(40.0));
    }
}

#[test]
fn ordinary_block_boundaries_preserve_vertical_physical_float_bfc_cursor_for_f32() {
    assert_ordinary_block_boundaries_preserve_physical_float_bfc_cursor::<f32>();
}

#[test]
fn ordinary_block_boundaries_preserve_vertical_physical_float_bfc_cursor_for_f64() {
    assert_ordinary_block_boundaries_preserve_physical_float_bfc_cursor::<f64>();
}

fn assert_ordinary_block_logical_sizing<S: LayoutScalar>(writing_mode: WritingMode) {
    let scalar = scalar_value::<S>;
    let percentage_thirty = LengthOf::value(scalar_percentage::<S>(0.0, 0.3));
    let percentage_sixty = LengthOf::value(scalar_percentage::<S>(0.0, 0.6));
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(100.0))),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                size: Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO),
                padding: Edges::new(
                    percentage_thirty,
                    LengthOf::ZERO,
                    percentage_sixty,
                    LengthOf::ZERO,
                ),
                ..NodeInputOf::default()
            },
        );
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
        .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");
    let root = public_final_output(&batch, 0);
    let child = public_final_output(&batch, 1);

    assert_eq!(root.size, Size::new(scalar(20.0), scalar(100.0)));
    assert_eq!(child.size, Size::new(scalar(20.0), scalar(100.0)));
    assert_eq!(child.padding.top, scalar(30.0));
    assert_eq!(child.padding.bottom, scalar(60.0));
}

#[test]
fn ordinary_block_logical_sizing_uses_vertical_and_sideways_inline_bases_for_f32() {
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::VerticalRl);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::VerticalLr);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::SidewaysRl);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::SidewaysLr);
}

#[test]
fn ordinary_block_logical_sizing_uses_vertical_and_sideways_inline_bases_for_f64() {
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::VerticalRl);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::VerticalLr);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::SidewaysRl);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::SidewaysLr);
}

fn assert_ordinary_block_collapse_relationship<S: LayoutScalar>(
    child_writing_mode: WritingMode,
    child_direction: Direction,
    measured_leaf: bool,
    expected_second_block_offset: S,
) {
    let scalar = scalar_value::<S>;
    let child_size = if child_writing_mode == WritingMode::HorizontalTb {
        Size::new(
            PreferredSizeOf::px(scalar(10.0)),
            PreferredSizeOf::px(S::ZERO),
        )
    } else {
        Size::new(
            PreferredSizeOf::px(S::ZERO),
            PreferredSizeOf::px(scalar(10.0)),
        )
    };
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: child_writing_mode,
                direction: child_direction,
                size: child_size,
                margin: Edges::new(
                    LengthAutoOf::px(scalar(30.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(10.0)),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        let measured = if child_writing_mode == WritingMode::HorizontalTb {
            Size::new(scalar(10.0), S::ZERO)
        } else {
            Size::new(S::ZERO, scalar(10.0))
        };
        tree = tree.with_measurement(1, measured);
    }
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
        .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

    assert_eq!(
        public_final_output(&batch, 2).location,
        Point::new(S::ZERO, expected_second_block_offset)
    );
}

fn assert_ordinary_block_relationship_matrix<S: LayoutScalar>() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            measured_leaf,
            scalar_value(60.0),
        );
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            measured_leaf,
            scalar_value(60.0),
        );
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::VerticalRl,
            Direction::Ltr,
            measured_leaf,
            scalar_value(100.0),
        );
    }

    for measured_leaf in [false, true] {
        let scalar = scalar_value::<S>;
        let mut tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalLr,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(200.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::HorizontalTb,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                    ..NodeInputOf::default()
                },
            );
        if measured_leaf {
            tree = tree.with_measurement(1, Size::new(scalar(5.0), scalar(10.0)));
        }
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(scalar(100.0)),
            AvailableOf::definite(scalar(200.0)),
        ))
        .expect("finite viewport is valid");

        let batch = compute_layout(&tree, 0, request).expect("orthogonal layout succeeds");
        assert_eq!(
            public_final_output(&batch, 1).size,
            Size::new(scalar(100.0), scalar(10.0))
        );
    }

    for child_direction in [Direction::Ltr, Direction::Rtl] {
        for measured_leaf in [false, true] {
            let scalar = scalar_value::<S>;
            let mut tree = PublicBlockTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: WritingMode::VerticalLr,
                        size: Size::new(
                            PreferredSizeOf::px(scalar(100.0)),
                            PreferredSizeOf::px(scalar(200.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: WritingMode::VerticalLr,
                        direction: child_direction,
                        size: Size::new(PreferredSizeOf::px(scalar(10.0)), PreferredSizeOf::AUTO),
                        ..NodeInputOf::default()
                    },
                );
            if measured_leaf {
                tree = tree.with_measurement(1, Size::new(scalar(10.0), scalar(5.0)));
            }
            let request = LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(scalar(100.0)),
                AvailableOf::definite(scalar(200.0)),
            ))
            .expect("finite viewport is valid");

            let batch = compute_layout(&tree, 0, request).expect("parallel layout succeeds");
            assert_eq!(
                public_final_output(&batch, 1).size,
                Size::new(scalar(10.0), scalar(200.0))
            );
        }
    }
}

fn assert_ordinary_block_opposing_flow_collapse<S: LayoutScalar>(measured_leaf: bool) {
    let scalar = scalar_value::<S>;
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::ZERO),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(30.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(10.0)),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        tree = tree.with_measurement(1, Size::new(S::ZERO, scalar(10.0)));
    }
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(scalar(100.0))))
        .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("opposing block layout succeeds");

    assert_eq!(
        public_final_output(&batch, 1).location,
        Point::new(scalar(30.0), S::ZERO)
    );
    assert_eq!(
        public_final_output(&batch, 2).location,
        Point::new(scalar(60.0), S::ZERO)
    );
}

fn assert_ordinary_block_opposing_flow_collapse_for_scalar<S: LayoutScalar>() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_opposing_flow_collapse::<S>(measured_leaf);
    }
}

fn assert_ordinary_block_orthogonal_inline_margin_subtraction<S: LayoutScalar>(
    measured_leaf: bool,
) {
    let scalar = scalar_value::<S>;
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(200.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(30.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        tree = tree.with_measurement(1, Size::new(scalar(5.0), scalar(10.0)));
    }
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(scalar(100.0)),
        AvailableOf::definite(scalar(200.0)),
    ))
    .expect("finite viewport is valid");

    let batch = compute_layout(&tree, 0, request).expect("orthogonal layout succeeds");

    assert_eq!(
        public_final_output(&batch, 1).size,
        Size::new(scalar(10.0), scalar(10.0))
    );
}

#[test]
fn ordinary_block_orthogonal_preserves_parallel_opposing_and_measured_leaf_relationships_for_f32() {
    assert_ordinary_block_relationship_matrix::<f32>();
}

#[test]
fn ordinary_block_orthogonal_preserves_parallel_opposing_and_measured_leaf_relationships_for_f64() {
    assert_ordinary_block_relationship_matrix::<f64>();
}

#[test]
fn ordinary_block_opposing_flow_collapse_preserves_real_and_measured_leaves_for_f32() {
    assert_ordinary_block_opposing_flow_collapse_for_scalar::<f32>();
}

#[test]
fn ordinary_block_opposing_flow_collapse_preserves_real_and_measured_leaves_for_f64() {
    assert_ordinary_block_opposing_flow_collapse_for_scalar::<f64>();
}

#[test]
fn ordinary_block_orthogonal_subtracts_physical_child_inline_margins_for_f32() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_orthogonal_inline_margin_subtraction::<f32>(measured_leaf);
    }
}

#[test]
fn ordinary_block_orthogonal_subtracts_physical_child_inline_margins_for_f64() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_orthogonal_inline_margin_subtraction::<f64>(measured_leaf);
    }
}

#[derive(Default)]
struct ScrollBlockTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    layouts: HashMap<u32, NodeOutput>,
    outputs: HashMap<u32, ComputeOutput>,
}

impl Traverse for ScrollBlockTree {
    type Node = u32;

    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map_or([].as_slice(), Vec::as_slice)
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

impl Compute for ScrollBlockTree {
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
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        Ok(self.outputs[&node])
    }
}

fn perform_scroll_block(tree: &mut ScrollBlockTree) -> ComputeOutput {
    crate::compute_block(
        tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(40.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::definite(40.0)),
        ),
    )
    .unwrap()
}

fn child_scroll_geometry(
    overflow: ComputedOverflow,
    size: Size,
    scrollable_overflow: ScrollRect,
) -> ScrollGeometry {
    child_scroll_geometry_with_edges(
        overflow,
        size,
        scrollable_overflow,
        Edges::ZERO,
        Edges::ZERO,
    )
}

fn child_scroll_geometry_with_edges(
    overflow: ComputedOverflow,
    size: Size,
    scrollable_overflow: ScrollRect,
    padding: Edges<f32>,
    border: Edges<f32>,
) -> ScrollGeometry {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut contributions =
        crate::scroll::ScrollContributionAccumulatorOf::new(scrollable_overflow);
    contributions.include_direct_line(scrollable_overflow);
    crate::scroll::canonical_scroll_geometry_from_source(
        crate::scroll::CanonicalScrollGeometrySourceOf {
            flow_axes,
            computed_overflow: overflow,
            item_is_replaced: false,
            border_box_size: size,
            border,
            padding,
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: ScrollbarWidth::ZERO,
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            clip_margin: crate::scroll::ClipMarginSourceOf::default(),
            scroll_padding: crate::scroll::OptimalRegionInsetsOf::default(),
            contributions,
            origin_axes: crate::scroll::ScrollOriginAxes::new(
                crate::scroll::ScrollOriginProgression::FlowEndward,
                crate::scroll::ScrollOriginProgression::FlowEndward,
            ),
            scroll_snap_type: ScrollSnapType::default(),
            target_border_box: ScrollRect::try_new(Point::ZERO, size).unwrap(),
            target_scroll_margin: ScrollMargin::default(),
            target_flow_axes: flow_axes,
            target_snap_align: ScrollSnapAlign::default(),
            target_snap_stop: ScrollSnapStop::default(),
        },
    )
    .expect("canonical block-test source facts produce geometry")
}

#[test]
fn block_layout_emits_scroll_geometry_for_scroll_overflow() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            overflow: computed_overflow(Overflow::Scroll, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_uses_visible_child_overflow_content_size() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(130.0, 70.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(130.0, 70.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(30.0, 30.0));
}

#[test]
fn block_scroll_geometry_clips_hidden_child_overflow_from_parent_range() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(160.0, 90.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(100.0, 40.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_preserves_negative_child_overflow_origin() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            inset: Edges {
                left: LengthAuto::px(-20.0),
                top: LengthAuto::px(-5.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            position: Position::Relative,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-20.0, -5.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(120.0, 45.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_distinguishes_visible_hidden_clip_and_scroll() {
    fn run(overflow: ComputedOverflow) -> ScrollGeometry {
        let mut tree = ScrollBlockTree::default();
        tree.children.insert(1, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                overflow,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        );

        perform_scroll_block(&mut tree).scroll_geometry.unwrap()
    }

    let visible = run(computed_overflow(Overflow::Visible, Overflow::Visible));
    assert_eq!(visible.overflow_clip(), None);
    assert_positive_physical_range(visible.physical_range(), Size::ZERO);

    let hidden = run(computed_overflow(Overflow::Hidden, Overflow::Hidden));
    assert_eq!(hidden.overflow_clip(), Some(hidden.scrollport()));
    assert_eq!(
        hidden
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(3.0, 4.0).unwrap()),
        PhysicalScrollOffset::try_new(0.0, 0.0).unwrap()
    );

    let clip = run(computed_overflow(Overflow::Clip, Overflow::Clip));
    assert_eq!(clip.overflow_clip(), Some(clip.scrollport()));
    assert_positive_physical_range(clip.physical_range(), Size::ZERO);

    let scroll = run(computed_overflow(Overflow::Scroll, Overflow::Scroll));
    assert_eq!(scroll.overflow_clip(), Some(scroll.scrollport()));
    assert_positive_physical_range(scroll.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_uses_node_local_padding_border_and_gutter_coordinates() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            padding: Edges::all(Length::px(2.0)),
            border: Edges::all(Length::px(3.0)),
            ..NodeInput::default()
        },
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().origin(), Point::new(13.0, 3.0));
    assert_eq!(geometry.scrollport().size(), Size::new(84.0, 34.0));
    assert_eq!(
        geometry.gutters().vertical(),
        Some(ScrollRect::try_new(Point::new(3.0, 3.0), Size::new(10.0, 34.0)).unwrap())
    );
}

#[test]
fn block_scroll_geometry_includes_absolute_child_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            inset: Edges {
                left: LengthAuto::px(90.0),
                top: LengthAuto::px(35.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(45.0, 25.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(135.0, 60.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(35.0, 20.0));
}

#[test]
fn block_scroll_geometry_includes_final_content_box_after_size_resolution() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            min_size: Size::new(MinSize::px(140.0), MinSize::px(80.0)),
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
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(60.0), Some(40.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(60.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(140.0, 80.0)).unwrap()
    );
}

#[test]
fn block_scroll_geometry_includes_inline_child_origin_bearing_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Hidden),
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::InlineBlock,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut inline_output = ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0));
    inline_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(20.0, 10.0),
        ScrollRect::try_new(Point::new(-12.0, -3.0), Size::new(70.0, 26.0)).unwrap(),
    ));
    tree.outputs.insert(2, inline_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-12.0, -3.0)
    );
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(70.0, 26.0));
    assert_positive_physical_range(geometry.physical_range(), Size::new(18.0, 13.0));
}

#[test]
fn block_scroll_geometry_clips_hidden_inline_child_overflow_from_parent_range() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::InlineBlock,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ..NodeInput::default()
        },
    );
    let mut inline_output =
        ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(150.0, 80.0));
    inline_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        Size::new(30.0, 10.0),
        ScrollRect::try_new(Point::new(-20.0, -7.0), Size::new(180.0, 92.0)).unwrap(),
    ));
    tree.outputs.insert(2, inline_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(100.0, 40.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_includes_segmented_inline_overflow_rects() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2, 3, 4, 5]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(5, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::InlineBlock,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        5,
        NodeInput {
            display: Display::InlineBlock,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut first_inline = ComputeOutput::from_sizes(Size::new(10.0, 10.0), Size::new(10.0, 10.0));
    first_inline.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(10.0, 10.0),
        ScrollRect::try_new(Point::new(-20.0, 0.0), Size::new(30.0, 10.0)).unwrap(),
    ));
    let mut second_inline = ComputeOutput::from_sizes(Size::new(10.0, 10.0), Size::new(10.0, 10.0));
    second_inline.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(10.0, 10.0),
        ScrollRect::try_new(Point::new(-7.0, 0.0), Size::new(25.0, 12.0)).unwrap(),
    ));
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(80.0, 50.0)));
    tree.outputs.insert(3, first_inline);
    tree.outputs.insert(5, second_inline);
    tree.styles.insert(4, NodeInput::default());

    struct SegmentedTree {
        inner: ScrollBlockTree,
        line_break: LineBreakInput,
    }

    impl Traverse for SegmentedTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = <ScrollBlockTree as Traverse>::Children<'a>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.inner.children(node)
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.inner.child_count(node)
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.inner.child(node, index)
        }
    }

    impl Compute for SegmentedTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            self.inner.node_input(node)
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            if node == 4 {
                LayoutInputOf::line_break(self.line_break)
            } else {
                self.inner.layout_input(node)
            }
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.inner.set_unrounded(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            self.inner.compute_child(node, input)
        }
    }

    let mut segmented = SegmentedTree {
        inner: tree,
        line_break: LineBreakInput::new()
            .with_clear(Clear::Left)
            .with_metrics(metrics),
    };

    let output = crate::compute_block(
        &mut segmented,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(80.0)),
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

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-20.0, 0.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(120.0, 80.0)
    );
}

#[test]
fn block_scroll_geometry_includes_float_child_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut float_output = ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(30.0, 10.0));
    float_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(30.0, 10.0),
        ScrollRect::try_new(Point::ZERO, Size::new(140.0, 55.0)).unwrap(),
    ));
    tree.outputs.insert(2, float_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(140.0, 55.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(40.0, 15.0));
}

#[test]
fn block_float_child_node_output_retains_canonical_scroll_geometry() {
    let padding = Edges::all(Length::px(2.0));
    let border = Edges::all(Length::px(1.0));
    let resolved_padding = Edges::all(2.0);
    let resolved_border = Edges::all(1.0);
    let child_compute_overflow =
        ScrollRect::try_new(Point::new(-8.0, -3.0), Size::new(50.0, 20.0)).unwrap();
    let mut float_output = ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(70.0, 32.0));
    float_output.scroll_geometry = Some(child_scroll_geometry_with_edges(
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        Size::new(30.0, 10.0),
        child_compute_overflow,
        resolved_padding,
        resolved_border,
    ));

    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            padding,
            border,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, float_output);

    perform_scroll_block(&mut tree);

    let child_layout = tree.layouts[&2];
    assert_eq!(child_layout.size, Size::new(30.0, 10.0));
    assert_eq!(child_layout.content_size, Size::new(70.0, 32.0));
    assert_eq!(child_layout.padding, resolved_padding);
    assert_eq!(child_layout.border, resolved_border);

    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow(), child_compute_overflow);
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        child_compute_overflow.origin()
    );
    assert_eq!(geometry, float_output.scroll_geometry.unwrap());
}

#[test]
fn block_scroll_geometry_includes_absolute_margin_box_with_area_offset() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
            padding: Edges::all(Length::px(7.0)),
            border: Edges::all(Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            inset: Edges {
                left: LengthAuto::px(90.0),
                top: LengthAuto::px(60.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            margin: Edges {
                left: LengthAuto::px(4.0),
                top: LengthAuto::px(3.0),
                right: LengthAuto::px(6.0),
                bottom: LengthAuto::px(7.0),
            },
            ..NodeInput::default()
        },
    );
    let mut absolute_output =
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(50.0, 20.0));
    absolute_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(20.0, 10.0),
        ScrollRect::try_new(Point::new(-2.0, -1.0), Size::new(60.0, 25.0)).unwrap(),
    ));
    tree.outputs.insert(2, absolute_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(5.0, 5.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(152.0, 87.0)
    );
    assert_eq!(output.content_size, Size::new(152.0, 87.0));
}

#[test]
fn block_child_node_output_recomputes_child_scroll_geometry() {
    let mut child_output = ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(80.0, 45.0));
    child_output.scroll_geometry = None;

    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    perform_scroll_block(&mut tree);

    let geometry = tree.layouts[&2].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size(), Size::new(50.0, 20.0));
    assert_positive_physical_range(geometry.physical_range(), Size::new(30.0, 25.0));
}

#[test]
fn block_child_node_output_keeps_hidden_child_own_scroll_range() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(160.0, 90.0)),
    );

    perform_scroll_block(&mut tree);

    let geometry = tree.layouts[&2].scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(160.0, 90.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(110.0, 70.0));
}

#[test]
fn block_absolute_child_scroll_geometry_uses_final_node_output_size() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(120.0, 30.0)),
    );

    perform_scroll_block(&mut tree);

    let child_layout = tree.layouts[&2];
    assert_eq!(child_layout.size.width, 100.0);
    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size().width, 100.0);
    assert_positive_physical_range(geometry.physical_range(), Size::new(20.0, 20.0));
}

#[test]
fn block_child_node_output_preserves_child_scrollable_overflow_origin() {
    let child_overflow =
        ScrollRect::try_new(Point::new(-15.0, -4.0), Size::new(95.0, 49.0)).unwrap();
    let mut child_output = ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(80.0, 45.0));
    child_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        Size::new(50.0, 20.0),
        child_overflow,
    ));

    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    perform_scroll_block(&mut tree);

    let geometry = tree.layouts[&2].scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-15.0, -4.0)
    );
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(95.0, 49.0));
}

#[test]
fn block_inline_child_node_output_uses_final_inline_item_geometry() {
    let child_overflow =
        ScrollRect::try_new(Point::new(-9.0, -3.0), Size::new(74.0, 34.0)).unwrap();
    let mut child_output = ComputeOutput::from_sizes(Size::new(40.0, 12.0), Size::new(65.0, 31.0));
    child_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        Size::new(40.0, 12.0),
        child_overflow,
    ));

    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::InlineBlock,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    perform_scroll_block(&mut tree);

    let child_layout = tree.layouts[&2];
    assert_eq!(child_layout.size, Size::new(40.0, 12.0));
    assert_eq!(child_layout.content_size, Size::new(65.0, 31.0));
    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size(), child_layout.size);
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
}

fn output_from_known_or(input: ComputeInput, fallback: Size) -> ComputeOutput {
    let size = Size::new(
        input.known().width.unwrap_or(fallback.width),
        input.known().height.unwrap_or(fallback.height),
    );
    ComputeOutput::from_sizes(size, size)
}

#[derive(Default)]
struct CalcBlockTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    layouts: HashMap<u32, NodeOutput>,
    inputs: HashMap<u32, Vec<ComputeInput>>,
}

impl Traverse for CalcBlockTree {
    type Node = u32;

    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map_or([].as_slice(), Vec::as_slice)
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

impl Compute for CalcBlockTree {
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
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        Ok({
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known().width.unwrap_or(0.0),
                input.known().height.unwrap_or(10.0),
            ))
        })
    }
}

#[test]
fn block_fixed_parent_height_keeps_orthogonal_child_inline_known() {
    let mut tree = CalcBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(162.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            writing_mode: WritingMode::VerticalRl,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .expect("fixed-height block layout succeeds");

    assert!(tree.inputs[&2].iter().any(|input| {
        input.known().height == Some(162.0)
            && input.parent().height == Some(162.0)
            && input.available().height == Available::definite(162.0)
    }));
}

#[test]
fn block_lays_out_atomic_inline_children_on_one_line() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(20.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 20.0));
}

#[test]
fn f64_block_layout_preserves_fractional_child_offsets() {
    let large = 16_777_217.25_f64;
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(40.0), PreferredSizeOf::px(5.25)),
                margin: Edges {
                    top: LengthAutoOf::px(large),
                    bottom: LengthAutoOf::px(0.25),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(40.0), PreferredSizeOf::px(7.5)),
                margin: Edges {
                    top: LengthAutoOf::px(0.375),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(AvailableOf::definite(100.0), AvailableOf::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the first child")
            .location,
        Point::new(0.0, large)
    );
    assert_eq!(
        tree.output(2)
            .expect("block layout must stage output for the second child")
            .location,
        Point::new(0.0, large + 5.25 + 0.375)
    );
    assert_eq!(
        tree.output(0)
            .expect("block layout must stage output for the root node")
            .size,
        Size::new(100.0, large + 5.25 + 0.375 + 7.5)
    );
}

#[test]
fn f64_block_layout_resolves_affine_values_without_narrowing() {
    let large = 16_777_217.25_f64;
    let container_width = 16_777_220.5_f64;
    let margin_left = lp64(large, 0.10);
    let width = lp64(large + 0.25, 0.50);
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(container_width), PreferredSizeOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::value(width), PreferredSizeOf::px(4.5)),
                margin: Edges {
                    left: LengthAutoOf::value(margin_left),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(
            AvailableOf::definite(container_width),
            AvailableOf::MAX_CONTENT,
        ),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the child")
            .location,
        Point::new(18_454_939.3, 0.0)
    );
    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the child")
            .size,
        Size::new(25_165_827.75, 4.5)
    );
}

#[test]
fn f64_inline_layout_preserves_large_atomic_inline_offsets() {
    let large = 16_777_217.25_f64;
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(large + 20.0), PreferredSizeOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::InlineBlock,
                size: Size::new(PreferredSizeOf::px(large), PreferredSizeOf::px(10.5)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::InlineBlock,
                size: Size::new(PreferredSizeOf::px(9.75), PreferredSizeOf::px(20.25)),
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(
            AvailableOf::definite(large + 20.0),
            AvailableOf::MAX_CONTENT,
        ),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the first child")
            .location,
        Point::new(0.0, 9.75)
    );
    assert_eq!(
        tree.output(2)
            .expect("block layout must stage output for the second child")
            .location,
        Point::new(large, 0.0)
    );
    assert_eq!(
        tree.output(0)
            .expect("block layout must stage output for the root node")
            .size,
        Size::new(large + 20.0, 20.25)
    );
}

#[test]
fn vertical_rl_block_places_atomic_inline_run_at_inline_start_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                border: Edges::all(Length::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(55.0, 5.0)
    );
}

#[test]
fn inline_grid_uses_grid_tracks_and_participates_as_atomic_inline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_grid_lanes_uses_lanes_tracks_and_participates_as_atomic_inline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineGridLanes,
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineGridLanes,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn block_wraps_atomic_inline_children_between_items() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(40.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size.height, 20.0);
}

#[test]
fn block_atomic_inline_run_honors_line_break_child() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 2.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(20.0, 12.0)
    );
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 16.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 28.0));
}

#[test]
fn ordinary_block_child_receives_parent_non_horizontal_containing_flow() {
    let parent_flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                ..NodeInput::default()
            },
        )
        .style(1, NodeInput::default());

    crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(80.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::definite(80.0)),
        ),
    )
    .unwrap();

    assert!(
        tree.inputs(1)
            .iter()
            .all(|input| input.containing_flow_axes() == parent_flow_axes)
    );
}

#[test]
fn block_line_break_conversion_with_metadata_preserves_current_output() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_direction(Direction::Rtl)
                .with_writing_mode(WritingMode::HorizontalTb)
                .with_vertical_align(VerticalAlign::Top)
                .with_clear(Clear::Both)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.inputs(2), &[]);
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 18.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 36.0));
}

#[test]
fn block_line_break_metrics_create_empty_line_height() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 15.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::default()
            },
        )
        .children(0, [1, 2])
        .line_break(1, LineBreakInput::new().with_metrics(metrics))
        .line_break(2, LineBreakInput::new().with_metrics(metrics));

    let output = crate::compute_block(
        &mut tree,
        0,
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
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size.height, 40.0);
    assert_eq!(output.first_baselines.y, Some(15.0));
    assert_eq!(output.last_baselines.y, Some(35.0));
    assert_eq!(tree.layout(1).unwrap().location.y, 15.0);
    assert_eq!(tree.layout(2).unwrap().location.y, 35.0);
}

#[test]
fn block_inline_boundaries_are_reported_as_zero_size_inline_controls() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(18.0, 13.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.inputs(1), &[]);
    assert_eq!(tree.inputs(3), &[]);
    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 13.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 3.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 13.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(20.0, 1.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 18.0));
}

#[test]
fn block_inline_boundaries_before_overwide_first_inline_block_do_not_create_leading_line() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(50.0, 35.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(40.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(20.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 35.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 25.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(20.0, 50.0));
}

#[test]
fn vertical_block_inline_boundaries_use_parent_flow() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalRl),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalRl),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(66.0, 0.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(70.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(66.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
}

#[test]
fn vertical_lr_block_inline_boundaries_use_parent_flow() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalLr),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalLr),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(14.0, 0.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(14.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
}

#[test]
fn hidden_line_break_does_not_split_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new().hidden())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(tree.inputs(2), &[]);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 10.0));
}

#[test]
fn block_atomic_inline_run_never_computes_line_break_as_box() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(tree.inputs(2), &[]);
}

#[test]
fn vertical_rl_line_break_is_laid_out_as_zero_size_inline_control() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(12.0), PreferredSize::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(66.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().location.x, 48.0);
}

#[test]
fn vertical_lr_line_break_is_laid_out_as_zero_size_inline_control() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalLr)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(12.0), PreferredSize::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(14.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().location.x, 20.0);
}

#[test]
#[should_panic(expected = "vertical line-break clear layout is not implemented")]
fn vertical_line_break_clear_panics_until_vertical_clear_is_modeled() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            1,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .with_clear(Clear::Both),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "vertical line-break clear layout is not implemented")]
fn vertical_parent_rejects_clear_even_when_line_break_input_defaults_horizontal() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(1, LineBreakInput::new().with_clear(Clear::Both));

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "line-break flow must match containing inline flow")]
fn vertical_parent_rejects_default_line_break_flow_until_input_is_layout_ready() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Ltr,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(1, LineBreakInput::new());

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "inline boundary flow must match containing inline flow")]
fn vertical_parent_rejects_default_inline_boundary_flow_until_input_is_layout_ready() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Ltr,
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
fn hidden_vertical_line_break_does_not_create_inline_control() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .hidden(),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(12.0), PreferredSize::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(68.0, 30.0)
    );
}

fn inline_break_clear_tree(
    clear: Clear,
    float_side: Float,
) -> crate::test_support::layout_tree::OracleTree {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: float_side,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(clear)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
}

#[test]
fn line_break_clear_left_moves_following_inline_segment_below_left_float() {
    let mut tree = inline_break_clear_tree(Clear::Left, Float::Left);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_right_moves_following_inline_segment_below_right_float() {
    let mut tree = inline_break_clear_tree(Clear::Right, Float::Right);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_both_uses_greater_left_or_right_float_bottom() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4, 5])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(70.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            4,
            LineBreakInput::new()
                .with_clear(Clear::Both)
                .with_metrics(metrics),
        )
        .style(
            5,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(5).unwrap().location,
        Point::new(0.0, 70.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 80.0));
}

#[test]
fn line_break_clear_at_run_end_moves_following_block_below_float() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(Clear::Left)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(25.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_left_ignores_right_float_and_preserves_alignment() {
    let mut tree = inline_break_clear_tree(Clear::Left, Float::Right).style(
        0,
        NodeInput {
            display: Display::Block,
            text_align: TextAlign::LegacyRight,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(180.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(200.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_right_ignores_left_float_and_preserves_alignment() {
    let mut tree = inline_break_clear_tree(Clear::Right, Float::Left).style(
        0,
        NodeInput {
            display: Display::Block,
            text_align: TextAlign::LegacyCenter,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(90.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(110.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(90.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_that_is_noop_after_line_height_preserves_alignment() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(Clear::Left)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(180.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(200.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_none_preserves_existing_single_run_layout_near_float() {
    let mut tree = inline_break_clear_tree(Clear::None, Float::Left);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn block_min_content_atomic_inline_run_uses_max_item_advance() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(40.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(60.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 20.0)
    );
}

#[test]
fn atomic_inline_auto_margins_resolve_to_zero() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                margin: Edges {
                    left: LengthAuto::AUTO,
                    right: LengthAuto::AUTO,
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    let child = tree.final_layout(1).unwrap();
    assert_eq!(child.location, Point::new(0.0, 0.0));
    assert_eq!(child.margin.left, 0.0);
    assert_eq!(child.margin.right, 0.0);
}

#[test]
fn inline_block_intrinsic_width_shrink_wraps_children() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(70.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(70.0, 20.0));
    assert_eq!(tree.final_layout(0).unwrap().size.width, 70.0);
}

#[test]
fn inline_block_uses_bottom_synthesized_baseline_when_child_has_no_baseline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_block_uses_inner_last_baseline_for_atomic_alignment() {
    let measured_inline_block = ComputeOutput::from_sizes_and_baselines(
        Size::new(10.0, 30.0),
        Size::new(10.0, 30.0),
        crate::Baselines {
            first: Point::new(None, Some(5.0)),
            last: Point::new(None, Some(25.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .measure(1, measured_inline_block);

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.y, 0.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
    assert_eq!(tree.final_layout(0).unwrap().size.height, 30.0);
}

#[test]
fn inline_block_keeps_child_margins_inside_atomic_wrapper() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                margin: Edges {
                    top: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(20.0, 15.0));
    assert_eq!(tree.final_layout(2).unwrap().location.y, 5.0);
    assert_eq!(tree.final_layout(0).unwrap().size.height, 15.0);
}

#[test]
fn inline_grid_can_host_subgrid_descendant() {
    let subgrid_track = || {
        TrackComponent::Subgrid(crate::SubgridTrack {
            name_components: Vec::new(),
        })
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(80.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![subgrid_track()],
                grid_template_rows: vec![subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
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

    assert_eq!(output.size, Size::new(80.0, 30.0));
}

#[test]
fn block_positions_block_children_around_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(10.0)),
                margin: Edges {
                    bottom: LengthAuto::px(7.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(15.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(10.0)),
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 27.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(10.0, 17.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 35.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 45.0));
}

#[test]
fn block_hidden_and_absolute_children_do_not_split_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::None,
                float: Float::Left,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                position: Position::Absolute,
                float: Float::Left,
                size: Size::new(PreferredSize::px(5.0), PreferredSize::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(10.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 10.0));
}

#[test]
fn block_rtl_atomic_inline_run_places_items_from_right_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn block_rtl_atomic_inline_run_mirrors_line_break_output_x() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new().with_direction(Direction::Rtl))
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 2.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 12.0)
    );
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(70.0, 16.0)
    );
}

#[test]
fn block_legacy_right_rtl_aligns_atomic_inline_run_to_physical_right_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn block_atomic_inline_run_alignment_uses_resolved_inner_width() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                min_size: Size::new(MinSize::px(100.0), MinSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.x, 25.0);
}

#[test]
fn block_legacy_center_aligns_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(25.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(45.0, 0.0)
    );
}

#[test]
fn block_inline_run_content_size_includes_visible_overflow_and_relative_inset() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                inset: Edges {
                    left: LengthAuto::px(15.0),
                    top: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(80.0, 30.0)),
        );

    let output = tree
        .compute_child(
            0,
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
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.content_size, Size::new(95.0, 35.0));
}

#[test]
fn block_inline_run_content_size_accounts_for_negative_relative_inset_after_content() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                inset: Edges {
                    top: LengthAuto::px(-5.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .measure(
            2,
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 30.0)),
        );

    let output = tree
        .compute_child(
            0,
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
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.content_size.height, 45.0);
}

#[test]
fn block_reports_inline_run_first_and_last_baselines() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
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
                Size::splat(Available::definite(100.0)),
            ),
        )
        .unwrap();

    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(20.0));
}

#[test]
fn block_reports_inline_run_baseline_including_padding() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                padding: Edges {
                    top: Length::px(10.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
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
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.first_baselines.y, Some(30.0));
    assert_eq!(output.last_baselines.y, Some(30.0));
}

#[test]
fn block_definite_compute_size_keeps_inline_run_baselines() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
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
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(20.0));
}

#[test]
fn block_definite_compute_size_keeps_block_child_baselines() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        crate::Baselines {
            first: Point::new(None, Some(7.0)),
            last: Point::new(None, Some(17.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .measure(1, child_output);

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
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
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(7.0));
    assert_eq!(output.last_baselines.y, Some(17.0));
}

fn assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis<S: LayoutScalar>(
    writing_mode: WritingMode,
) where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let child_output = ComputeOutputOf::from_sizes_and_baselines(
        Size::new(S::from_f64(70.0), S::from_f64(110.0)),
        Size::new(S::from_f64(70.0), S::from_f64(110.0)),
        BaselinesOf {
            first: Point::new(Some(S::from_f64(7.0)), None),
            last: Point::new(Some(S::from_f64(11.0)), None),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(120.0)),
                    PreferredSizeOf::AUTO,
                ),
                padding: Edges {
                    top: LengthOf::px(S::from_f64(5.0)),
                    left: LengthOf::px(S::from_f64(3.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode,
                margin: Edges {
                    top: LengthAutoOf::px(S::from_f64(17.0)),
                    left: LengthAutoOf::px(S::from_f64(11.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .measure(1, child_output);

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(120.0)), Some(S::from_f64(160.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(120.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("block layout succeeds");

    assert_eq!(
        tree.layout(1).expect("child layout is staged").location,
        Point::new(S::from_f64(14.0), S::from_f64(22.0))
    );
    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(21.0)), None)
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(S::from_f64(25.0)), None)
    );
}

#[test]
fn orthogonal_baseline_block_translation_uses_physical_x_for_f32() {
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f32>(
        WritingMode::VerticalRl,
    );
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn orthogonal_baseline_block_translation_uses_physical_x_for_f64() {
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f64>(
        WritingMode::VerticalRl,
    );
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f64>(
        WritingMode::SidewaysLr,
    );
}

fn assert_block_aggregates_physical_baselines_on_both_axes<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(140.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                writing_mode: WritingMode::VerticalRl,
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(17.0)),
                    LengthAutoOf::px(S::from_f64(5.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(19.0)),
                    LengthAutoOf::px(S::from_f64(7.0)),
                    LengthAutoOf::px(S::from_f64(23.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            1,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(20.0)),
                Size::new(S::from_f64(70.0), S::from_f64(20.0)),
                BaselinesOf {
                    first: Point::new(Some(S::from_f64(7.0)), None),
                    last: Point::new(Some(S::from_f64(11.0)), None),
                },
            ),
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(30.0), S::from_f64(40.0)),
                Size::new(S::from_f64(30.0), S::from_f64(40.0)),
                BaselinesOf {
                    first: Point::new(None, Some(S::from_f64(9.0))),
                    last: Point::new(None, Some(S::from_f64(15.0))),
                },
            ),
        );

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(140.0)), Some(S::from_f64(200.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(140.0)),
                AvailableOf::definite(S::from_f64(200.0)),
            ),
        ),
    )
    .expect("block layout succeeds");
    let first_child = tree.layout(1).expect("first child layout is staged");
    let second_child = tree.layout(2).expect("second child layout is staged");

    assert_eq!(
        output.first_baselines,
        Point::new(
            Some(first_child.location.x + S::from_f64(7.0)),
            Some(second_child.location.y + S::from_f64(9.0)),
        )
    );
    assert_eq!(
        output.last_baselines,
        Point::new(
            Some(first_child.location.x + S::from_f64(11.0)),
            Some(second_child.location.y + S::from_f64(15.0)),
        )
    );
}

#[test]
fn physical_baseline_block_aggregates_both_axes_for_f32() {
    assert_block_aggregates_physical_baselines_on_both_axes::<f32>();
}

#[test]
fn physical_baseline_block_aggregates_both_axes_for_f64() {
    assert_block_aggregates_physical_baselines_on_both_axes::<f64>();
}

fn assert_block_preserves_a_child_y_baseline<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(120.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                writing_mode: WritingMode::HorizontalTb,
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(17.0)),
                    LengthAutoOf::px(S::from_f64(5.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            1,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(40.0)),
                Size::new(S::from_f64(70.0), S::from_f64(40.0)),
                BaselinesOf {
                    first: Point::new(None, Some(S::from_f64(9.0))),
                    last: Point::new(None, Some(S::from_f64(15.0))),
                },
            ),
        );

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(120.0)), Some(S::from_f64(160.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(120.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("block layout succeeds");
    let child = tree.layout(1).expect("child layout is staged");

    assert_eq!(
        output.first_baselines,
        Point::new(None, Some(child.location.y + S::from_f64(9.0)))
    );
    assert_eq!(
        output.last_baselines,
        Point::new(None, Some(child.location.y + S::from_f64(15.0)))
    );
}

#[test]
fn physical_baseline_block_preserves_y_for_f32() {
    assert_block_preserves_a_child_y_baseline::<f32>();
}

#[test]
fn physical_baseline_block_preserves_y_for_f64() {
    assert_block_preserves_a_child_y_baseline::<f64>();
}

#[test]
fn block_definite_compute_size_keeps_non_empty_flex_child_baselines() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        crate::Baselines {
            first: Point::new(None, Some(9.0)),
            last: Point::new(None, Some(19.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT)
        .measure(1, child_output);

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
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
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(9.0));
    assert_eq!(output.last_baselines.y, Some(19.0));
}

#[test]
fn block_layout_stacks_in_flow_children_vertically() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                right: Length::px(5.0),
                bottom: Length::px(7.0),
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(30.0, 12.0), Size::new(30.0, 12.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformRootLayout,
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

    assert_eq!(output.size, Size::new(100.0, 41.0));
    assert_eq!(output.content_size, Size::new(98.0, 39.0));
    assert_eq!(tree.layouts[&2].location, Point::new(18.0, 6.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.left, 6.0);
    assert_eq!(tree.layouts[&3].location, Point::new(12.0, 21.0));
    assert_eq!(tree.layouts[&3].size, Size::new(30.0, 12.0));
    assert_eq!(tree.inputs[&2][0].parent(), Size::new(Some(82.0), None));
    assert_eq!(tree.inputs[&3][0].parent(), Size::new(Some(82.0), None));
}

#[test]
fn block_in_flow_affine_margin_resolves_against_containing_block_width() {
    let mut tree = CalcBlockTree::default();
    let margin_left = lp(-4.0, 0.1);
    let width = lp(20.0, 0.5);
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::value(width), PreferredSize::AUTO),
            margin: Edges {
                left: LengthAuto::value(margin_left),
                right: LengthAuto::ZERO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(200.0), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::Definite(200.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs[&2][0].known(), Size::new(Some(120.0), None));
    assert_eq!(tree.layouts[&2].location, Point::new(16.0, 0.0));
    assert_eq!(tree.layouts[&2].margin.left, 16.0);
    assert_eq!(tree.layouts[&2].size, Size::new(120.0, 10.0));
}

#[test]
fn block_container_affine_padding_uses_parent_basis() {
    let mut tree = CalcBlockTree::default();
    let padding = lp(2.0, 0.1);
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            padding: Edges::all(Length::value(padding)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(1, NodeInput::default());

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
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

    assert_eq!(output.content_size.width, 100.0);
}

#[test]
fn block_auto_width_includes_in_flow_child_horizontal_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            item_is_table: true,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformRootLayout,
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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(3.0, 0.0));
    assert_eq!(output.size, Size::new(32.0, 10.0));
    assert_eq!(output.content_size, Size::new(32.0, 10.0));
}

#[test]
fn block_float_contributes_to_intrinsic_width_and_places_from_right_edge() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(80.0)),
            border: Edges::all(Length::px(2.0)),
            ..NodeInput::default()
        },
    );
    for node in [2, 3, 4] {
        tree.styles.insert(
            node,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        );
        tree.outputs.insert(
            node,
            ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
        );
    }

    let output = crate::compute_block(
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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(154.0, 80.0));
    assert_eq!(tree.layouts[&2].location, Point::new(102.0, 2.0));
    assert_eq!(tree.layouts[&3].location, Point::new(52.0, 2.0));
    assert_eq!(tree.layouts[&4].location, Point::new(2.0, 2.0));
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(100.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats_above_full_width_float() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 200.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_overflow_clip_zero_width_child_ignores_float_exclusion_without_clear() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Clip, Overflow::Clip),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_right_alignment_without_float_exclusion() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(150.0, 0.0)
    );
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_center_alignment_without_float_exclusion() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(75.0, 0.0)
    );
}

#[test]
fn block_bfc_float_content_size_height_excludes_container_top_inset() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                border: Edges {
                    top: Length::px(5.0),
                    ..Edges::all(Length::ZERO)
                },
                padding: Edges {
                    top: Length::px(10.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 15.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().content_size.height, 40.0);
}

#[test]
fn block_bfc_clear_only_visible_child_keeps_normal_x_while_clearing_y() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(150.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Left,
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(150.0, 50.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 70.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_left_sits_below_left_float_row() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Left,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 100.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_right_sits_below_all_right_floats() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Right,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 200.0)
    );
}

#[test]
fn block_layout_collapses_adjacent_in_flow_vertical_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 20.0));
    assert_eq!(output.size, Size::new(100.0, 30.0));
    assert_eq!(output.content_size, Size::new(100.0, 30.0));
}

#[test]
fn block_layout_collapses_first_child_top_margin_through_parent() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
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
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        10.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        0.0
    );
}

#[test]
fn block_scroll_container_keeps_first_child_top_margin_inside() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 10.0));
    assert_eq!(output.size, Size::new(100.0, 15.0));
    assert_eq!(output.content_size, Size::new(100.0, 15.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        0.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        0.0
    );
    assert!(
        !output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

#[test]
fn block_rtl_scrollbar_gutter_uses_left_inset() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(10.0),
                ))
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(17.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
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

    crate::compute_block(
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

    assert_eq!(tree.layouts[&2].location, Point::new(17.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(83.0, 10.0));
}

#[test]
fn block_layout_collapses_last_child_bottom_margin_through_parent() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
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
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        0.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        10.0
    );
}

#[test]
fn block_layout_keeps_grid_child_margins_inside_parent_flow() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            Ok(ComputeOutput::from_outer_size(Size::new(50.0, 20.0)))
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(50.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
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

    assert_eq!(output.size, Size::new(50.0, 30.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.top, 10.0);
}

#[test]
fn block_layout_collapses_margins_through_empty_in_flow_child() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            border: Edges {
                top: Length::px(1.0),
                right: Length::ZERO,
                bottom: Length::px(1.0),
                left: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                bottom: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(7.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    let mut empty_output = ComputeOutput::from_sizes(Size::new(100.0, 0.0), Size::new(100.0, 0.0));
    empty_output.block_margin_collapse = PhysicalBlockMarginCollapse::from_block_flow(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        CollapsibleMargin::ZERO,
        CollapsibleMargin::ZERO,
        true,
    );
    tree.outputs.insert(2, empty_output);
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 11.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 11.0));
    assert_eq!(output.size, Size::new(100.0, 22.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
}

#[test]
fn block_empty_auto_height_can_collapse_through() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
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
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 0.0));
    assert!(
        output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

#[test]
fn block_with_padding_reports_own_margins_when_child_collapse_is_blocked() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            margin: Edges {
                top: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            padding: Edges {
                top: Length::px(1.0),
                bottom: Length::px(1.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 2.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        8.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        6.0
    );
    assert!(
        !output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

fn assert_collapsible_percentage_margins_use_containing_inline_extent<S: LayoutScalar>(
    writing_mode: WritingMode,
) where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let top_margin = LengthPercentageOf::<S>::from_coefficients(S::ZERO, S::from_f64(0.25))
        .expect("test coefficients are finite");
    let bottom_margin = LengthPercentageOf::<S>::from_coefficients(S::ZERO, S::from_f64(0.5))
        .expect("test coefficients are finite");
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                margin: Edges {
                    top: LengthAutoOf::value(top_margin),
                    bottom: LengthAutoOf::value(bottom_margin),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                padding: Edges {
                    top: LengthOf::px(S::from_f64(1.0)),
                    bottom: LengthOf::px(S::from_f64(1.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(40.0)), Some(S::from_f64(120.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(writing_mode, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(40.0)),
                AvailableOf::definite(S::from_f64(120.0)),
            ),
        ),
    )
    .expect("block layout succeeds");

    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        S::from_f64(30.0)
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        S::from_f64(60.0)
    );
}

#[test]
fn collapsible_percentage_margins_use_non_horizontal_containing_inline_extent_for_f32() {
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f32>(
        WritingMode::VerticalRl,
    );
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn collapsible_percentage_margins_use_non_horizontal_containing_inline_extent_for_f64() {
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f64>(
        WritingMode::VerticalRl,
    );
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f64>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn block_in_flow_invalid_numeric_horizontal_margin_uses_zero_fallback() {
    let invalid_margin = LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX)
        .expect("test coefficients are finite");
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::AUTO),
                margin: Edges {
                    left: LengthAuto::value(invalid_margin),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(f32::MAX), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(f32::MAX), Available::MAX_CONTENT),
        ),
    )
    .expect("the in-flow invalid-numeric margin falls back to zero");

    assert_eq!(
        tree.output(2)
            .expect("child block receives an in-flow layout")
            .margin
            .left,
        0.0
    );
}

#[test]
fn block_layout_positions_in_flow_children_from_right_edge_in_rtl() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::ZERO,
                right: Length::px(5.0),
                bottom: Length::ZERO,
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(7.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(tree.layouts[&2].location, Point::new(67.0, 1.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.right, 7.0);
}

#[test]
fn block_layout_expands_horizontal_auto_margins_for_in_flow_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::AUTO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::AUTO,
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
}

#[test]
fn block_content_size_includes_visible_child_overflow_content() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 10.0));
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn block_relative_child_inset_offsets_final_layout_location() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
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
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(13.0, 3.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn block_layout_stretches_auto_width_in_flow_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
                    Size::new(input.known().width.unwrap(), 10.0),
                    Size::new(input.known().width.unwrap(), 10.0),
                )
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::ZERO,
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                left: LengthAuto::px(3.0),
                right: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
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

    assert_eq!(tree.inputs[&2][0].known().width, Some(76.0));
    assert_eq!(tree.layouts[&2].size, Size::new(76.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(8.0, 0.0));
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
    assert_eq!(output.size, Size::new(100.0, 10.0));
}

#[test]
fn block_compute_size_uses_in_flow_children_for_auto_height() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
                    Size::new(input.known().width.unwrap(), 10.0),
                    Size::new(input.known().width.unwrap(), 10.0),
                )
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::px(7.0),
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
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

    assert_eq!(tree.inputs[&2][0].run_mode(), RunMode::ComputeSize);
    assert_eq!(tree.inputs[&2][0].known().width, Some(76.0));
    assert_eq!(output.size, Size::new(100.0, 26.0));
    assert_eq!(output.content_size, Size::ZERO);
    assert!(tree.layouts.is_empty());
}

#[test]
fn block_compute_size_uses_definite_min_max_without_measuring_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            min_size: Size::new(MinSize::px(100.0), MinSize::px(40.0)),
            max_size: Size::new(MaxSize::px(100.0), MaxSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
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
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn block_definite_compute_size_keeps_grid_children_on_fast_path_until_grid_baselines() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            panic!("definite grid compute-size should stay on the fast path")
        }
    }

    for display in [Display::Grid, Display::GridLanes] {
        let mut tree = BlockTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![3]);
        tree.children.insert(3, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                min_size: Size::new(MinSize::px(100.0), MinSize::px(40.0)),
                max_size: Size::new(MaxSize::px(100.0), MaxSize::px(40.0)),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInput {
                display,
                ..NodeInput::default()
            },
        );
        tree.styles.insert(3, NodeInput::default());

        let output = crate::compute_block(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::ComputeSize,
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
        assert_eq!(output.content_size, Size::ZERO);
    }
}

#[test]
fn block_auto_height_clamps_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            max_size: Size::new(MaxSize::NONE, MaxSize::px(12.0)),
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
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 20.0), Size::new(100.0, 20.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 20.0));
}

#[test]
fn block_auto_size_applies_aspect_ratio_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            aspect_ratio: AspectRatio::new(2.0),
            max_size: Size::new(MaxSize::px(50.0), MaxSize::NONE),
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
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    let output = crate::compute_block(
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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(50.0, 25.0));
}

#[test]
fn block_legacy_text_align_offsets_table_child_in_free_inline_space() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
            Ok(ComputeOutput::from_outer_size(Size::new(60.0, 10.0)))
        }
    }

    fn run(text_align: TextAlign, direction: Direction) -> NodeOutput {
        let mut tree = BlockTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                direction,
                text_align,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInput {
                display: Display::Block,
                item_is_table: true,
                ..NodeInput::default()
            },
        );

        crate::compute_block(
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

        tree.layouts[&2]
    }

    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Ltr).location.x,
        70.0
    );
    assert_eq!(
        run(TextAlign::LegacyRight, Direction::Ltr).location.x,
        140.0
    );
    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Rtl).location.x,
        70.0
    );
    assert_eq!(run(TextAlign::LegacyLeft, Direction::Rtl).location.x, 0.0);
}

#[test]
fn block_layout_lays_out_absolute_children_without_flow_contribution_and_hides_display_none() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
                if input.run_mode() == RunMode::PerformHiddenLayout {
                    ComputeOutput::HIDDEN
                } else {
                    self.outputs[&node]
                }
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            border: Edges::all(Length::px(1.0)),
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
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        4,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(40.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(80.0, 32.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(98.0, 41.0));
    assert_eq!(tree.layouts[&2].location, Point::new(1.0, 1.0));
    assert_eq!(tree.layouts[&3].location, Point::new(8.0, 10.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 10.0));
    assert_eq!(
        tree.layouts[&4],
        NodeOutput::with_source_index(crate::SourceIndex::new(2))
    );
    assert_eq!(
        tree.inputs[&4],
        vec![ComputeInput::hidden(crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr,),
            crate::ParentFormattingContext::BlockFlow
        ))]
    );
}

#[test]
fn block_absolute_child_without_insets_uses_static_position_after_flow() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            border: Edges::all(Length::px(1.0)),
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
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(98.0, 10.0), Size::new(98.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 5.0), Size::new(20.0, 5.0)),
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(tree.layouts[&2].location, Point::new(1.0, 1.0));
    assert_eq!(tree.layouts[&3].location, Point::new(1.0, 11.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 5.0));
}

#[test]
fn block_absolute_child_auto_size_applies_aspect_ratio_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            aspect_ratio: AspectRatio::new(2.0),
            max_size: Size::new(MaxSize::px(50.0), MaxSize::NONE),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    crate::compute_block(
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

    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 25.0));
}

#[test]
fn block_absolute_child_auto_size_resolves_from_opposing_insets() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                right: LengthAuto::px(17.0),
                top: LengthAuto::px(13.0),
                bottom: LengthAuto::px(11.0),
            },
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
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

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(
        tree.inputs[&2][0].known(),
        Size::new(Some(74.0), Some(24.0))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(8.0, 14.0));
    assert_eq!(tree.layouts[&2].size, Size::new(74.0, 24.0));
}

#[test]
fn block_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(10.0),
                right: LengthAuto::px(10.0),
                top: LengthAuto::AUTO,
                bottom: LengthAuto::AUTO,
            },
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    crate::compute_block(
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
        tree.inputs[&2][0].known(),
        Size::new(Some(80.0), Some(40.0))
    );
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 40.0));
}

#[test]
fn block_absolute_child_expands_horizontal_auto_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
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

    crate::compute_block(
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
        tree.inputs[&2][0].known(),
        Size::new(Some(20.0), Some(10.0))
    );
    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn block_absolute_child_large_width_keeps_horizontal_auto_margins_zero() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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
                ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0))
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(PreferredSize::px(70.0), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    crate::compute_block(
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
        tree.inputs[&2][0].known(),
        Size::new(Some(70.0), Some(10.0))
    );
    assert_eq!(tree.layouts[&2].margin.left, 0.0);
    assert_eq!(tree.layouts[&2].margin.right, 0.0);
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(70.0, 10.0));
}

#[test]
fn block_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
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

    impl Compute for BlockTree {
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

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                right: LengthAuto::px(17.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    crate::compute_block(
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
        tree.inputs[&2][0].known(),
        Size::new(Some(20.0), Some(10.0))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(62.0, 1.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[derive(Default)]
struct CalcLeafTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    layouts: HashMap<u32, NodeOutput>,
    invalid_leaf_measurement: bool,
}

impl Traverse for CalcLeafTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map_or([].as_slice(), Vec::as_slice)
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

impl Compute for CalcLeafTree {
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
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        if self.child_count(node) > 0 {
            return compute_block(self, node, input);
        }

        let style = self.styles[&node].clone();
        let invalid_leaf_measurement = self.invalid_leaf_measurement;
        compute_leaf(input, &style, |measure_input| {
            let known = measure_input.known_content_size();
            let available = measure_input
                .available_content_size()
                .map(MeasurementAvailable::into_available);
            Ok::<_, core::convert::Infallible>(Size::new(
                if invalid_leaf_measurement {
                    f32::NAN
                } else {
                    known
                        .width
                        .or_else(|| available.width.into_option())
                        .unwrap_or(0.0)
                },
                known.height.unwrap_or(10.0),
            ))
        })
        .map_err(|error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                error.operation(),
                error.kind().clone(),
            )
        })
    }
}

#[test]
fn calc_leaf_tree_propagates_leaf_measurement_error_instead_of_panicking() {
    let mut tree = CalcLeafTree {
        invalid_leaf_measurement: true,
        ..CalcLeafTree::default()
    };
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(0, NodeInput::default());
    tree.styles.insert(1, NodeInput::default());

    let error = compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
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
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(output))
            if output.axis() == PhysicalAxis::Horizontal
    ));
}

#[test]
fn block_inline_affine_leaf_uses_public_leaf_path() {
    let mut tree = CalcLeafTree::default();
    let width = lp(10.0, 0.5);
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::InlineBlock,
            size: Size::new(PreferredSize::value(width), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    let output = compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
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

    assert_eq!(tree.layouts[&1].size.width, 60.0);
    assert_eq!(output.content_size.width, 100.0);
}

#[test]
fn unresolved_symbolic_vertical_margin_is_not_treated_as_auto_margin() {
    let mut tree = CalcLeafTree::default();
    let margin = lp(0.0, 0.25);
    tree.styles.insert(
        1,
        NodeInput {
            margin: Edges {
                top: LengthAuto::value(margin),
                ..Edges::<Scalar>::ZERO.map(|_| LengthAuto::px(0.0))
            },
            ..NodeInput::default()
        },
    );

    let resolved =
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr)
            .zip_physical_edges_with_inline_extent(
                tree.styles[&1].margin,
                Size::new(None, None),
                |length, basis| length.resolve_auto_with_status(basis),
            );
    let resolved = resolve_logical_in_flow_margin(
        crate::geometry::LogicalEdgesOf::new(
            resolved.left,
            resolved.right,
            resolved.top,
            resolved.bottom,
        ),
        crate::geometry::LogicalSizeOf::new(10.0, 10.0),
        None,
    );

    assert_eq!(resolved.block_start, 0.0);
}

#[test]
fn invalid_numeric_margin_keeps_explicit_failure_without_panicking() {
    let margin = LengthAuto::value(
        LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX).expect("finite coefficients"),
    )
    .resolve_auto_with_status(Some(10.0));

    let resolved = resolve_logical_in_flow_margin(
        crate::geometry::LogicalEdgesOf::new(
            ResolvedLengthAuto::Resolved(0.0),
            ResolvedLengthAuto::Resolved(0.0),
            margin,
            ResolvedLengthAuto::Resolved(0.0),
        ),
        crate::geometry::LogicalSizeOf::new(10.0, 10.0),
        Some(10.0),
    );

    assert_eq!(resolved.block_start, 0.0);
}

#[derive(Default)]
struct Fri05C03BlockPassTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    child_output: Option<ComputeOutput>,
    child_inputs: Vec<ComputeInput>,
    layouts: Vec<(u32, NodeOutput)>,
}

impl Traverse for Fri05C03BlockPassTree {
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

impl Compute for Fri05C03BlockPassTree {
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
        _node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, ComputeOutput, Self::Scalar> {
        self.child_inputs.push(input);
        Ok(self
            .child_output
            .expect("FRI-05 block pass child output is configured"))
    }
}

fn fri05_c03_block_input(size: Size<f32>, flow_axes: FlowAxes) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        size.map(Some),
        size.map(Some),
        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
        size.map(Available::definite),
    )
}

fn fri05_c03_block_overflow_at_flow_axes(
    flow_axes: FlowAxes,
    inline: Overflow,
    block: Overflow,
) -> ComputedOverflow {
    match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => computed_overflow(inline, block),
        PhysicalAxis::Vertical => computed_overflow(block, inline),
    }
}

fn fri05_c03_block_gutter_at(
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

fn fri05_c03_block_all_flow_axes() -> [FlowAxes; 10] {
    [
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysRl, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
    ]
}

fn fri05_c03_empty_block_geometry(style: NodeInput) -> ScrollGeometry {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let size = Size::new(100.0, 80.0);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![]);
    tree.styles.insert(0, style);
    crate::compute_block(&mut tree, 0, fri05_c03_block_input(size, flow_axes))
        .expect("FRI-05 empty block layout succeeds")
        .scroll_geometry
        .expect("performed block layout emits geometry")
}

#[test]
fn fri05_c03_block_reservation_places_forced_and_stable_gutters_in_all_flows() {
    for flow_axes in fri05_c03_block_all_flow_axes() {
        let style = |overflow, gutter, width| NodeInput {
            display: Display::Block,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_gutter: gutter,
            scrollbar_width: ScrollbarWidth::try_new(width).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        };
        let forced = fri05_c03_empty_block_geometry(style(
            fri05_c03_block_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Scroll),
            ScrollbarGutter::Auto,
            7.0,
        ));
        let stable = fri05_c03_empty_block_geometry(style(
            fri05_c03_block_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::Stable,
            7.0,
        ));
        let both = fri05_c03_empty_block_geometry(style(
            fri05_c03_block_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::StableBothEdges,
            7.0,
        ));

        for (geometry, expected_sides) in [
            (forced, vec![flow_axes.inline_end()]),
            (stable, vec![flow_axes.inline_end()]),
            (both, vec![flow_axes.inline_start(), flow_axes.inline_end()]),
        ] {
            for side in [
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
            ] {
                assert_eq!(
                    fri05_c03_block_gutter_at(geometry.gutters(), side).is_some(),
                    expected_sides.contains(&side),
                    "unexpected {side:?} gutter for {flow_axes:?}: {geometry:#?}"
                );
            }
        }

        let expected_one_edge = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(7.0, 0.0),
            PhysicalAxis::Vertical => Size::new(0.0, 7.0),
        };
        assert_eq!(forced.scrollbar_size(), expected_one_edge, "{flow_axes:?}");
        assert_eq!(stable.scrollbar_size(), expected_one_edge, "{flow_axes:?}");
        assert_eq!(both.scrollbar_size(), expected_one_edge + expected_one_edge);

        let zero = fri05_c03_empty_block_geometry(style(
            fri05_c03_block_overflow_at_flow_axes(flow_axes, Overflow::Scroll, Overflow::Scroll),
            ScrollbarGutter::StableBothEdges,
            0.0,
        ));
        assert_eq!(zero.scrollbar_size(), Size::ZERO, "{flow_axes:?}");
        assert_eq!(zero.gutters().top(), None);
        assert_eq!(zero.gutters().right(), None);
        assert_eq!(zero.gutters().bottom(), None);
        assert_eq!(zero.gutters().left(), None);
    }
}

fn fri05_c03_block_auto_case(
    gutter: ScrollbarGutter,
    child_size: Size<f32>,
    expected_states: &[(bool, bool)],
    expected_reservation: Size<f32>,
) {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let size = Size::splat(100.0);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_gutter: gutter,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_sizes(child_size, child_size));

    let output = crate::compute_block(&mut tree, 0, fri05_c03_block_input(size, flow_axes))
        .expect("FRI-05 auto block layout succeeds");
    assert!(
        tree.child_inputs.iter().all(|input| {
            input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
        }),
        "each block child starts node-local auto settlement at INITIAL: {:#?}",
        tree.child_inputs
    );
    let states = tree
        .child_inputs
        .iter()
        .map(|input| {
            let state = input.containing_auto_scrollbar_pass();
            (
                state.at(PhysicalAxis::Horizontal),
                state.at(PhysicalAxis::Vertical),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(states, expected_states, "child size {child_size:?}");
    assert_eq!(
        tree.layouts.iter().filter(|(node, _)| *node == 1).count(),
        expected_states.len(),
        "each geometry-changing evaluation stages the child once"
    );
    assert!(
        states.len() <= 3,
        "auto geometry must settle within three passes"
    );
    let geometry = output
        .scroll_geometry
        .expect("stable auto block output includes geometry");
    assert_eq!(geometry.scrollbar_size(), expected_reservation);
}

#[test]
fn fri05_c03_block_auto_runs_only_monotone_geometry_changing_evaluations() {
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(80.0, 80.0),
        &[(false, false)],
        Size::ZERO,
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(120.0, 80.0),
        &[(false, false), (true, false)],
        Size::new(0.0, 15.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(80.0, 120.0),
        &[(false, false), (false, true)],
        Size::new(15.0, 0.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(120.0, 100.0),
        &[(false, false), (true, false), (true, true)],
        Size::new(15.0, 15.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(100.0, 120.0),
        &[(false, false), (false, true), (true, true)],
        Size::new(15.0, 15.0),
    );
}

#[test]
fn fri05_c03_block_auto_stable_reservations_skip_redundant_full_evaluations() {
    fri05_c03_block_auto_case(
        ScrollbarGutter::Stable,
        Size::new(80.0, 120.0),
        &[(false, false)],
        Size::new(15.0, 0.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::StableBothEdges,
        Size::new(60.0, 120.0),
        &[(false, false)],
        Size::new(30.0, 0.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Stable,
        Size::new(90.0, 120.0),
        &[(false, false), (true, true)],
        Size::new(15.0, 15.0),
    );
}

#[test]
fn fri05_c03_block_tiny_saturates_opposing_reservations_before_child_layout() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let size = Size::new(2.0, 20.0);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(2.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_outer_size(Size::ZERO));

    let output = crate::compute_block(&mut tree, 0, fri05_c03_block_input(size, flow_axes))
        .expect("tiny block geometry remains supported");
    assert_eq!(tree.child_inputs.len(), 1);
    assert_eq!(tree.child_inputs[0].known().width, Some(0.0));
    let geometry = output
        .scroll_geometry
        .expect("tiny performed block emits geometry");
    assert_eq!(geometry.border_box().size(), size);
    assert_eq!(geometry.content_box().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollport().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollbar_size(), Size::new(2.0, 0.0));
    let left = geometry.gutters().left().expect("left gutter is retained");
    let right = geometry
        .gutters()
        .right()
        .expect("right gutter is retained");
    assert_eq!(left.size(), Size::new(1.0, 20.0));
    assert_eq!(right.size(), Size::new(1.0, 20.0));
    assert_eq!(left.origin(), Point::ZERO);
    assert_eq!(right.origin(), Point::new(1.0, 0.0));
}

#[test]
fn fri05_c03_block_tiny_max_size_below_raw_edges_keeps_layout_geometry_coherent() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            max_size: Size::new(MaxSize::NONE, MaxSize::px(12.0)),
            border: Edges {
                top: Length::px(15.0),
                bottom: Length::px(15.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_outer_size(Size::ZERO));

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .expect("raw edges larger than max size remain supported");

    assert_eq!(tree.child_inputs.len(), 1);
    assert_eq!(
        tree.child_inputs[0].settled_auto_scrollbars(),
        crate::scroll::SettledAutoScrollbarState::INITIAL
    );
    assert_eq!(
        tree.child_inputs[0].containing_auto_scrollbar_pass(),
        crate::scroll::SettledAutoScrollbarState::INITIAL
    );
    assert_eq!(
        tree.child_inputs[0].available().width,
        Available::definite(100.0)
    );
    let child = tree
        .layouts
        .iter()
        .find_map(|(node, layout)| (*node == 1).then_some(*layout))
        .expect("the coherent pass stages its child");
    assert_eq!(child.location, Point::new(0.0, 15.0));

    assert_eq!(output.size, Size::new(100.0, 30.0));
    let geometry = output
        .scroll_geometry
        .expect("performed block emits canonical geometry");
    assert_eq!(geometry.border_box().size(), Size::new(100.0, 30.0));
    assert_eq!(geometry.padding_box().origin(), Point::new(0.0, 15.0));
    assert_eq!(geometry.padding_box().size(), Size::new(100.0, 0.0));
    assert_eq!(geometry.content_box(), geometry.padding_box());
    assert_eq!(geometry.scrollport(), geometry.padding_box());
    assert_eq!(geometry.physical_range().x().minimum(), 0.0);
    assert_eq!(geometry.physical_range().x().maximum(), 0.0);
    assert_eq!(geometry.physical_range().y().minimum(), 0.0);
    assert_eq!(geometry.physical_range().y().maximum(), 0.0);
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
}

#[test]
fn fri05_c03_block_tiny_max_inline_size_below_raw_edges_keeps_child_space_zero() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(20.0)),
            max_size: Size::new(MaxSize::px(12.0), MaxSize::NONE),
            border: Edges {
                right: Length::px(10.0),
                left: Length::px(10.0),
                ..Edges::all(Length::ZERO)
            },
            padding: Edges {
                right: Length::px(5.0),
                left: Length::px(5.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_outer_size(Size::ZERO));

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(None, Some(20.0)),
            Size::new(None, Some(20.0)),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            Size::new(Available::MAX_CONTENT, Available::definite(20.0)),
        ),
    )
    .expect("raw inline edges larger than max size remain supported");

    assert_eq!(
        tree.child_inputs
            .iter()
            .map(|input| {
                let state = input.settled_auto_scrollbars();
                (
                    state.at(PhysicalAxis::Horizontal),
                    state.at(PhysicalAxis::Vertical),
                )
            })
            .collect::<Vec<_>>(),
        [(false, false), (false, false)]
    );
    assert_eq!(
        tree.child_inputs
            .iter()
            .map(|input| {
                let state = input.containing_auto_scrollbar_pass();
                (
                    state.at(PhysicalAxis::Horizontal),
                    state.at(PhysicalAxis::Vertical),
                )
            })
            .collect::<Vec<_>>(),
        [(false, false), (false, false)]
    );
    assert_eq!(
        tree.child_inputs[0].available().width,
        Available::MAX_CONTENT
    );
    assert_eq!(tree.child_inputs[1].known().width, Some(0.0));
    assert_eq!(
        tree.child_inputs[1].available().width,
        Available::definite(0.0)
    );
    let child = tree
        .layouts
        .iter()
        .find_map(|(node, layout)| (*node == 1).then_some(*layout))
        .expect("the final coherent pass stages its child");
    assert_eq!(child.location, Point::new(15.0, 0.0));

    assert_eq!(output.size, Size::new(30.0, 20.0));
    let geometry = output
        .scroll_geometry
        .expect("performed block emits canonical geometry");
    assert_eq!(geometry.border_box().size(), output.size);
    assert_eq!(geometry.padding_box().origin(), Point::new(10.0, 0.0));
    assert_eq!(geometry.padding_box().size(), Size::new(10.0, 20.0));
    assert_eq!(geometry.content_box().origin(), Point::new(15.0, 0.0));
    assert_eq!(geometry.content_box().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollport(), geometry.padding_box());
    assert_eq!(geometry.physical_range().x().maximum(), 0.0);
    assert_eq!(geometry.physical_range().y().maximum(), 0.0);
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
}

fn fri05_c03_block_union_content_size<S: LayoutScalar>(output: NodeOutputOf<S>) -> Size<S> {
    let geometry = output
        .scroll_geometry
        .expect("a performed block has canonical geometry");
    let anchor = geometry.content_box().origin();
    let overflow = geometry.scrollable_overflow();
    let overflow_origin = overflow.origin();
    let overflow_size = overflow.size();
    let overflow_end = Point::new(
        overflow_origin.x + overflow_size.width,
        overflow_origin.y + overflow_size.height,
    );

    Size::new(
        anchor.x.max(overflow_end.x) - anchor.x.min(overflow_origin.x),
        anchor.y.max(overflow_end.y) - anchor.y.min(overflow_origin.y),
    )
}

#[test]
fn fri05_c03_block_contribution_current_sources_retain_targets_and_union_content_size() {
    #[derive(Clone, Copy)]
    enum ChildKind {
        InFlow,
        Float,
        Inline,
        Absolute,
    }

    let scroll_margin = ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    for kind in [
        ChildKind::InFlow,
        ChildKind::Float,
        ChildKind::Inline,
        ChildKind::Absolute,
    ] {
        let (display, float, position, inset, expected_overflow) = match kind {
            ChildKind::InFlow => (
                Display::Block,
                Float::None,
                Position::Relative,
                Edges::all(LengthAuto::AUTO),
                Size::new(30.0, 15.0),
            ),
            ChildKind::Float => (
                Display::Block,
                Float::Left,
                Position::Relative,
                Edges::all(LengthAuto::AUTO),
                Size::new(30.0, 15.0),
            ),
            ChildKind::Inline => (
                Display::InlineBlock,
                Float::None,
                Position::Relative,
                Edges::all(LengthAuto::AUTO),
                Size::new(30.0, 15.0),
            ),
            ChildKind::Absolute => (
                Display::Block,
                Float::None,
                Position::Absolute,
                Edges {
                    top: LengthAuto::px(12.0),
                    left: LengthAuto::px(15.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                Size::new(45.0, 27.0),
            ),
        };
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display,
                    float,
                    position,
                    inset,
                    size: Size::new(PreferredSize::px(30.0), PreferredSize::px(15.0)),
                    scroll_margin,
                    scroll_snap_align: snap_align,
                    scroll_snap_stop: ScrollSnapStop::Always,
                    ..NodeInput::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap(),
        )
        .expect("each current block-owned contribution source lays out");

        let root = public_final_output(&batch, 0);
        let root_geometry = root
            .scroll_geometry
            .expect("root block geometry is present");
        assert_eq!(root_geometry.scrollable_overflow().origin(), Point::ZERO);
        assert_eq!(
            root_geometry.scrollable_overflow().size(),
            expected_overflow
        );
        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));

        let child = public_final_output(&batch, 1);
        let target = child
            .scroll_geometry
            .expect("every performed block-owned child retains geometry")
            .target();
        assert_eq!(target.border_box().size(), child.size);
        assert_eq!(target.scroll_margin(), scroll_margin);
        assert_eq!(target.snap_align(), snap_align);
        assert_eq!(target.snap_stop(), ScrollSnapStop::Always);
    }
}

fn fri05_c03_block_contribution_fallback_child(
    display: Display,
    overflow: ComputedOverflow,
) -> (NodeOutput, NodeOutput) {
    let scroll_padding = ScrollPadding::new(
        ScrollPaddingValue::value(LengthPercentageOf::px(2.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(4.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(3.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(1.0).unwrap()),
    );
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Rtl,
                overflow,
                overflow_clip_margin: OverflowClipMargin::try_new(OverflowClipBox::BorderBox, 3.0)
                    .unwrap(),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(9.0).unwrap(),
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(60.0)),
                border: Edges::new(
                    Length::px(1.0),
                    Length::px(2.0),
                    Length::px(3.0),
                    Length::px(4.0),
                ),
                padding: Edges::new(
                    Length::px(5.0),
                    Length::px(6.0),
                    Length::px(7.0),
                    Length::px(8.0),
                ),
                margin: Edges::new(
                    LengthAuto::px(2.0),
                    LengthAuto::px(3.0),
                    LengthAuto::px(4.0),
                    LengthAuto::px(5.0),
                ),
                scroll_padding,
                scroll_margin: ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap(),
                scroll_snap_type: ScrollSnapType::Enabled {
                    axis: ScrollSnapAxis::Block,
                    strictness: ScrollSnapStrictness::Mandatory,
                },
                scroll_snap_align: ScrollSnapAlign::new(
                    ScrollSnapAlignValue::End,
                    ScrollSnapAlignValue::Center,
                ),
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(200.0))).unwrap(),
    )
    .expect("a block stages fallback geometry for its flex/grid child");

    assert_eq!(
        batch
            .final_entries()
            .iter()
            .filter(|entry| entry.node() == 1)
            .count(),
        1,
        "the direct child is staged exactly once"
    );
    (
        public_final_output(&batch, 0),
        public_final_output(&batch, 1),
    )
}

fn fri05_c03_assert_block_contribution_fallback_common(root: NodeOutput, child: NodeOutput) {
    let geometry = child
        .scroll_geometry
        .expect("a performed block-owned child has canonical geometry");
    let target = geometry.target();
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    let scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);

    assert_eq!(geometry.flow_axes(), flow_axes);
    assert_eq!(geometry.border_box().origin(), Point::ZERO);
    assert_eq!(geometry.border_box().size(), child.size);
    assert_eq!(
        geometry.padding_box(),
        ScrollRect::try_new(
            Point::new(child.border.left, child.border.top),
            Size::new(
                child.size.width - child.border.horizontal_sum(),
                child.size.height - child.border.vertical_sum(),
            ),
        )
        .unwrap()
    );
    assert_eq!(
        geometry.resolved_scroll_padding(),
        Edges::new(2.0, 4.0, 3.0, 1.0)
    );
    assert_eq!(
        geometry.scroll_snap_type(),
        ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Block,
            strictness: ScrollSnapStrictness::Mandatory,
        }
    );
    assert_eq!(target.border_box(), geometry.border_box());
    assert_eq!(target.scroll_margin(), scroll_margin);
    assert_eq!(target.flow_axes(), flow_axes);
    assert_eq!(target.snap_align(), snap_align);
    assert_eq!(target.snap_stop(), ScrollSnapStop::Always);

    let root_geometry = root
        .scroll_geometry
        .expect("the performed block root has canonical geometry");
    let seed = root_geometry.padding_box();
    let seed_origin = seed.origin();
    let seed_size = seed.size();
    let contribution_origin = Point::new(
        child.location.x - child.margin.left.max(0.0),
        child.location.y - child.margin.top.max(0.0),
    );
    let contribution_end = Point::new(
        child.location.x + child.size.width + child.margin.right.max(0.0),
        child.location.y + child.size.height + child.margin.bottom.max(0.0),
    );
    let expected_origin = Point::new(
        seed_origin.x.min(contribution_origin.x),
        seed_origin.y.min(contribution_origin.y),
    );
    let expected_end = Point::new(
        (seed_origin.x + seed_size.width).max(contribution_end.x),
        (seed_origin.y + seed_size.height).max(contribution_end.y),
    );
    let expected_overflow = ScrollRect::try_new(
        expected_origin,
        Size::new(
            expected_end.x - expected_origin.x,
            expected_end.y - expected_origin.y,
        ),
    )
    .unwrap();
    assert_eq!(root_geometry.scrollable_overflow(), expected_overflow);
    assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
}

fn fri05_c06_assert_block_reserved_gutter_geometry(geometry: ScrollGeometry) {
    assert_eq!(geometry.border_box().size(), Size::new(100.0, 100.0));
    assert_eq!(geometry.padding_box().size(), Size::new(100.0, 100.0));
    assert_eq!(geometry.scrollport().origin(), Point::new(15.0, 0.0));
    assert_eq!(geometry.scrollport().size(), Size::new(70.0, 100.0));
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(100.0, 150.0)).unwrap(),
        "reserved gutters remain part of complete scrollable overflow"
    );

    let range = geometry.physical_range();
    assert_eq!(
        range.x().maximum() - range.x().minimum(),
        0.0,
        "reserved gutters do not create horizontal scroll range"
    );
    assert_eq!(
        range.y().maximum() - range.y().minimum(),
        50.0,
        "vertical child overflow remains reachable"
    );
}

#[test]
fn fri05_c06_block_reserved_gutter_stable_both_edges_excludes_horizontal_range() {
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(70.0), PreferredSize::px(150.0)),
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(200.0))).unwrap(),
    )
    .expect("stable both-edge block layout succeeds");

    fri05_c06_assert_block_reserved_gutter_geometry(
        public_final_output(&batch, 0)
            .scroll_geometry
            .expect("the block front door emits canonical geometry"),
    );
}

#[test]
fn fri05_c06_block_reserved_gutter_retained_child_fallback_excludes_horizontal_range() {
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(200.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Flex,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_sizes_and_baselines(
        Size::new(100.0, 100.0),
        Size::new(70.0, 150.0),
        Baselines::NONE,
    ));

    crate::compute_block(
        &mut tree,
        0,
        fri05_c03_block_input(
            Size::new(200.0, 200.0),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("the block front door stages retained-child fallback geometry");
    let child = tree
        .layouts
        .iter()
        .rev()
        .find_map(|(node, output)| (*node == 1).then_some(*output))
        .expect("the retained child is staged");

    fri05_c06_assert_block_reserved_gutter_geometry(
        child
            .scroll_geometry
            .expect("the retained-child fallback emits canonical geometry"),
    );
}

#[test]
fn fri05_c03_block_contribution_flex_fallback_retains_target_and_box_sources() {
    let (root, child) = fri05_c03_block_contribution_fallback_child(
        Display::Flex,
        computed_overflow(Overflow::Hidden, Overflow::Scroll),
    );
    fri05_c03_assert_block_contribution_fallback_common(root, child);

    let geometry = child.scroll_geometry.unwrap();
    let padding_box = geometry.padding_box();
    let expected_scrollport = ScrollRect::try_new(
        Point::new(padding_box.origin().x + 9.0, padding_box.origin().y),
        Size::new(padding_box.size().width - 18.0, padding_box.size().height),
    )
    .unwrap();
    assert_eq!(geometry.scrollport(), expected_scrollport);
    assert_eq!(geometry.scrollbar_size(), Size::new(18.0, 0.0));
    assert_eq!(geometry.gutters().top(), None);
    assert_eq!(geometry.gutters().bottom(), None);
    assert_eq!(geometry.gutters().left().unwrap().size().width, 9.0);
    assert_eq!(geometry.gutters().right().unwrap().size().width, 9.0);
    let x_clip = geometry.overflow_clip().x().unwrap();
    let y_clip = geometry.overflow_clip().y().unwrap();
    assert_eq!(
        (x_clip.minimum(), x_clip.maximum()),
        (
            expected_scrollport.origin().x,
            expected_scrollport.origin().x + expected_scrollport.size().width,
        )
    );
    assert_eq!(
        (y_clip.minimum(), y_clip.maximum()),
        (
            expected_scrollport.origin().y,
            expected_scrollport.origin().y + expected_scrollport.size().height,
        )
    );
    assert_eq!(geometry.used_overflow_x(), Overflow::Hidden);
    assert_eq!(geometry.used_overflow_y(), Overflow::Scroll);
    assert_eq!(
        geometry.content_box(),
        ScrollRect::try_new(
            Point::new(
                expected_scrollport.origin().x + child.padding.left,
                expected_scrollport.origin().y + child.padding.top,
            ),
            Size::new(
                expected_scrollport.size().width - child.padding.horizontal_sum(),
                expected_scrollport.size().height - child.padding.vertical_sum(),
            ),
        )
        .unwrap()
    );
}

#[test]
fn fri05_c03_block_contribution_flex_and_grid_fallback_seed_padding_with_stable_gutters() {
    let cases = [Display::Flex, Display::Grid].map(|display| {
        let (root, child) = fri05_c03_block_contribution_fallback_child(
            display,
            computed_overflow(Overflow::Hidden, Overflow::Scroll),
        );
        fri05_c03_assert_block_contribution_fallback_common(root, child);
        (display, child)
    });

    assert_eq!(
        cases.map(|(_, child)| child.scroll_geometry.unwrap().scrollable_overflow()),
        cases.map(|(_, child)| child.scroll_geometry.unwrap().padding_box()),
        "flex and grid fallback must both retain their own padding and gutter area"
    );

    for (display, child) in cases {
        let geometry = child
            .scroll_geometry
            .expect("the fallback child has canonical geometry");
        let expected_range = (0.0, 0.0, 0.0, 0.0);
        assert_ne!(geometry.padding_box(), geometry.scrollport(), "{display:?}");
        assert_ne!(
            geometry.padding_box(),
            geometry.content_box(),
            "{display:?}"
        );
        assert_eq!(
            (
                geometry.physical_range().x().minimum(),
                geometry.physical_range().x().maximum(),
                geometry.physical_range().y().minimum(),
                geometry.physical_range().y().maximum(),
            ),
            expected_range,
            "{display:?}"
        );
        assert_eq!(
            child.content_box_size(),
            geometry.content_box().size(),
            "{display:?}"
        );
        assert_eq!(
            child.scrollbar_size(),
            geometry.scrollbar_size(),
            "{display:?}"
        );
        assert_eq!(
            child.scrollbar_size(),
            geometry.scrollbar_size(),
            "{display:?}"
        );
        assert_eq!(
            geometry.target().border_box(),
            geometry.border_box(),
            "{display:?}"
        );
    }
}

#[test]
fn fri05_c03_block_contribution_grid_fallback_retains_target_and_clip_sources() {
    let (root, child) = fri05_c03_block_contribution_fallback_child(
        Display::Grid,
        computed_overflow(Overflow::Clip, Overflow::Visible),
    );
    fri05_c03_assert_block_contribution_fallback_common(root, child);

    let geometry = child.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport(), geometry.padding_box());
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
    assert_eq!(geometry.gutters().top(), None);
    assert_eq!(geometry.gutters().right(), None);
    assert_eq!(geometry.gutters().bottom(), None);
    assert_eq!(geometry.gutters().left(), None);
    let x_clip = geometry
        .overflow_clip()
        .x()
        .expect("the child's x clip retains its border-box clip margin");
    assert_eq!(
        (x_clip.minimum(), x_clip.maximum()),
        (-3.0, child.size.width + 3.0)
    );
    assert_eq!(geometry.overflow_clip().y(), None);
    assert_eq!(geometry.used_overflow_x(), Overflow::Clip);
    assert_eq!(geometry.used_overflow_y(), Overflow::Visible);
    assert_eq!(
        geometry.content_box(),
        ScrollRect::try_new(
            Point::new(
                geometry.scrollport().origin().x + child.padding.left,
                geometry.scrollport().origin().y + child.padding.top,
            ),
            Size::new(
                geometry.scrollport().size().width - child.padding.horizontal_sum(),
                geometry.scrollport().size().height - child.padding.vertical_sum(),
            ),
        )
        .unwrap()
    );
}

#[test]
fn fri05_c03_block_contribution_terminal_padding_extends_final_in_flow_ends() {
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                padding: Edges {
                    right: Length::px(3.0),
                    bottom: Length::px(4.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap(),
    )
    .expect("terminal-padding block layout succeeds");

    let root = public_final_output(&batch, 0);
    let geometry = root
        .scroll_geometry
        .expect("root block geometry is present");
    assert_eq!(geometry.scrollable_overflow().origin(), Point::ZERO);
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(33.0, 24.0));
    assert_eq!(root.content_size, Size::new(33.0, 24.0));
}

#[test]
fn fri05_c03_block_negative_margin_families_use_only_positive_outsets() {
    for position in [Position::Relative, Position::Absolute] {
        let inset = match position {
            Position::Relative => Edges {
                top: LengthAuto::px(25.0),
                left: LengthAuto::px(105.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            Position::Absolute => Edges {
                top: LengthAuto::px(30.0),
                left: LengthAuto::px(90.0),
                ..Edges::all(LengthAuto::AUTO)
            },
        };
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    position,
                    inset,
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                    margin: Edges {
                        top: LengthAuto::px(-20.0),
                        right: LengthAuto::px(7.0),
                        bottom: LengthAuto::px(5.0),
                        left: LengthAuto::px(-30.0),
                    },
                    ..NodeInput::default()
                },
            );
        let result = std::panic::catch_unwind(|| {
            compute_layout(
                &tree,
                0,
                LayoutRootRequest::viewport(Size::splat(Available::definite(120.0))).unwrap(),
            )
        });
        let batch = result
            .expect("valid negative margins never panic")
            .expect("valid negative margins never produce an invalid synthetic rectangle");

        let root = public_final_output(&batch, 0);
        let child = public_final_output(&batch, 1);
        assert!(child.size.width > 0.0 && child.size.height > 0.0);
        let padding_box = root
            .scroll_geometry
            .expect("root block geometry is present")
            .padding_box();
        let padding_origin = padding_box.origin();
        let padding_size = padding_box.size();
        let expected_origin = Point::new(
            padding_origin
                .x
                .min(child.location.x - child.margin.left.max(0.0)),
            padding_origin
                .y
                .min(child.location.y - child.margin.top.max(0.0)),
        );
        let expected_end = Point::new(
            (padding_origin.x + padding_size.width)
                .max(child.location.x + child.size.width + child.margin.right.max(0.0)),
            (padding_origin.y + padding_size.height)
                .max(child.location.y + child.size.height + child.margin.bottom.max(0.0)),
        );
        let expected = ScrollRect::try_new(
            expected_origin,
            Size::new(
                expected_end.x - expected_origin.x,
                expected_end.y - expected_origin.y,
            ),
        )
        .unwrap();
        assert_eq!(
            root.scroll_geometry.unwrap().scrollable_overflow(),
            expected
        );
        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
    }
}

#[test]
fn fri05_c03_integration_padding_seed_direct_block_retains_gutter_area_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>()
    where
        crate::test_support::layout_tree::OracleTreeOf<S>:
            Compute + Traverse<Node = u32, Scalar = S>,
    {
        fn gutter_at<S: LayoutScalar>(
            gutters: ScrollbarGutterRectsOf<S>,
            side: PhysicalSide,
        ) -> Option<ScrollRectOf<S>> {
            match side {
                PhysicalSide::Top => gutters.top(),
                PhysicalSide::Right => gutters.right(),
                PhysicalSide::Bottom => gutters.bottom(),
                PhysicalSide::Left => gutters.left(),
            }
        }

        let scalar = scalar_value::<S>;
        let size = Size::new(scalar(100.0), scalar(80.0));
        for flow_axes in fri05_c03_block_all_flow_axes() {
            for (case, inline, block, scrollbar_gutter, expected_sides) in [
                (
                    "forced-block",
                    Overflow::Hidden,
                    Overflow::Scroll,
                    ScrollbarGutter::Auto,
                    vec![flow_axes.inline_end()],
                ),
                (
                    "stable-block",
                    Overflow::Hidden,
                    Overflow::Hidden,
                    ScrollbarGutter::Stable,
                    vec![flow_axes.inline_end()],
                ),
                (
                    "both-edge-block",
                    Overflow::Hidden,
                    Overflow::Hidden,
                    ScrollbarGutter::StableBothEdges,
                    vec![flow_axes.inline_start(), flow_axes.inline_end()],
                ),
                (
                    "forced-inline",
                    Overflow::Scroll,
                    Overflow::Hidden,
                    ScrollbarGutter::Auto,
                    vec![flow_axes.block_end()],
                ),
            ] {
                let style = NodeInputOf::<S> {
                    display: Display::Block,
                    writing_mode: flow_axes.writing_mode(),
                    direction: flow_axes.direction(),
                    overflow: fri05_c03_block_overflow_at_flow_axes(flow_axes, inline, block),
                    scrollbar_gutter,
                    scrollbar_width: ScrollbarWidthOf::try_new(scalar(7.0)).unwrap(),
                    size: Size::new(
                        PreferredSizeOf::px(size.width),
                        PreferredSizeOf::px(size.height),
                    ),
                    padding: Edges::all(LengthOf::px(scalar(3.0))),
                    border: Edges::all(LengthOf::px(scalar(2.0))),
                    ..NodeInputOf::default()
                };
                let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                    .children(0, [])
                    .style(0, style);
                let output = crate::compute_block(
                    &mut tree,
                    0,
                    ComputeInputOf::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        size.map(Some),
                        size.map(Some),
                        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                        size.map(AvailableOf::definite),
                    ),
                )
                .expect("guttered direct block lays out");
                let geometry = output
                    .scroll_geometry
                    .expect("performed direct block emits geometry");

                assert_ne!(
                    geometry.padding_box(),
                    geometry.scrollport(),
                    "{case}/{flow_axes:?}"
                );
                assert_eq!(
                    geometry.scrollable_overflow(),
                    geometry.padding_box(),
                    "the canonical own padding box must remain complete overflow for {case}/{flow_axes:?}"
                );
                for side in [
                    PhysicalSide::Top,
                    PhysicalSide::Right,
                    PhysicalSide::Bottom,
                    PhysicalSide::Left,
                ] {
                    assert_eq!(
                        gutter_at(geometry.gutters(), side).is_some(),
                        expected_sides.contains(&side),
                        "unexpected {side:?} gutter for {case}/{flow_axes:?}"
                    );
                }

                let range = geometry.physical_range();
                assert_eq!(
                    (range.x().minimum(), range.x().maximum()),
                    (S::ZERO, S::ZERO),
                    "x range must exclude static gutter reservation for {case}/{flow_axes:?}"
                );
                assert_eq!(
                    (range.y().minimum(), range.y().maximum()),
                    (S::ZERO, S::ZERO),
                    "y range must exclude static gutter reservation for {case}/{flow_axes:?}"
                );

                let node_output = NodeOutputOf::<S>::new().with_scroll_geometry(Some(geometry));
                assert_eq!(
                    node_output.content_box_size(),
                    geometry.content_box().size()
                );
                assert_eq!(node_output.scrollbar_size(), geometry.scrollbar_size());
                assert_eq!(geometry.target().border_box(), geometry.border_box());
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri05_c03_integration_absolute_top_gutter_offsets_reduced_area_margin_contribution_and_tiny_origins()
 {
    fn assert_lane<S: LayoutScalar>()
    where
        crate::test_support::layout_tree::OracleTreeOf<S>:
            Compute + Traverse<Node = u32, Scalar = S>,
    {
        let scalar = scalar_value::<S>;
        let top_gutter_flows = [
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
            FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
            FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl),
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        ];

        for flow_axes in top_gutter_flows {
            assert_eq!(flow_axes.inline_end(), PhysicalSide::Top);
            let container_size = Size::new(scalar(100.0), scalar(80.0));
            let child_size = Size::new(scalar(110.0), scalar(70.0));
            let child_margin = Edges::new(scalar(3.0), scalar(7.0), scalar(11.0), scalar(2.0));
            let container_style = NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: fri05_c03_block_overflow_at_flow_axes(
                    flow_axes,
                    Overflow::Hidden,
                    Overflow::Scroll,
                ),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(10.0)).unwrap(),
                size: Size::new(
                    PreferredSizeOf::px(container_size.width),
                    PreferredSizeOf::px(container_size.height),
                ),
                border: Edges::all(LengthOf::px(scalar(5.0))),
                ..NodeInputOf::default()
            };
            let child_style = NodeInputOf::<S> {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(
                    PreferredSizeOf::px(child_size.width),
                    PreferredSizeOf::px(child_size.height),
                ),
                inset: Edges {
                    top: LengthAutoOf::px(S::ZERO),
                    left: LengthAutoOf::px(S::ZERO),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                margin: Edges::new(
                    LengthAutoOf::px(child_margin.top),
                    LengthAutoOf::px(child_margin.right),
                    LengthAutoOf::px(child_margin.bottom),
                    LengthAutoOf::px(child_margin.left),
                ),
                ..NodeInputOf::default()
            };
            let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(0, container_style)
                .style(1, child_style)
                .measure(1, ComputeOutputOf::from_outer_size(child_size));
            let output = crate::compute_block(
                &mut tree,
                0,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    container_size.map(Some),
                    container_size.map(Some),
                    ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                    container_size.map(AvailableOf::definite),
                ),
            )
            .expect("top-gutter absolute layout succeeds");
            let geometry = output
                .scroll_geometry
                .expect("performed block emits geometry");
            let top_gutter = geometry
                .gutters()
                .top()
                .expect("flow reserves a top gutter");
            assert_eq!(top_gutter.size().height, scalar(10.0));
            assert_eq!(geometry.scrollport().origin().y, scalar(15.0));
            assert_eq!(
                geometry.scrollport().size(),
                Size::new(scalar(90.0), scalar(60.0))
            );

            let perform_inputs = tree
                .inputs(1)
                .iter()
                .filter(|input| input.run_mode() == RunMode::PerformLayout)
                .copied()
                .collect::<Vec<_>>();
            assert_eq!(perform_inputs.len(), 1, "absolute child is performed once");
            assert_eq!(
                perform_inputs[0].parent(),
                Size::new(Some(scalar(90.0)), Some(scalar(60.0)))
            );
            assert_eq!(
                perform_inputs[0].available(),
                Size::new(
                    AvailableOf::definite(scalar(90.0)),
                    AvailableOf::definite(scalar(60.0)),
                )
            );

            let child = tree.layout(1).expect("absolute child is staged");
            assert_eq!(child.location, Point::new(scalar(7.0), scalar(18.0)));
            assert_eq!(child.size, child_size);
            assert_eq!(child.margin, child_margin);
            let expected_overflow = ScrollRectOf::try_new(
                Point::new(scalar(5.0), scalar(5.0)),
                Size::new(scalar(119.0), scalar(94.0)),
            )
            .unwrap();
            assert_eq!(
                geometry.scrollable_overflow(),
                expected_overflow,
                "the final absolute margin area contributes exactly once"
            );
            assert_eq!(output.content_size, expected_overflow.size());

            let tiny_size = Size::splat(scalar(2.0));
            let tiny_child_size = Size::splat(scalar(1.0));
            let tiny_container = NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(15.0)).unwrap(),
                size: Size::new(
                    PreferredSizeOf::px(tiny_size.width),
                    PreferredSizeOf::px(tiny_size.height),
                ),
                border: Edges::new(
                    LengthOf::px(scalar(1.0)),
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    LengthOf::px(scalar(1.0)),
                ),
                ..NodeInputOf::default()
            };
            let tiny_child = NodeInputOf::<S> {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(
                    PreferredSizeOf::px(tiny_child_size.width),
                    PreferredSizeOf::px(tiny_child_size.height),
                ),
                inset: Edges {
                    top: LengthAutoOf::px(S::ZERO),
                    left: LengthAutoOf::px(S::ZERO),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                ..NodeInputOf::default()
            };
            let mut tiny_tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(0, tiny_container)
                .style(1, tiny_child)
                .measure(1, ComputeOutputOf::from_outer_size(tiny_child_size));
            let tiny_output = crate::compute_block(
                &mut tiny_tree,
                0,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    tiny_size.map(Some),
                    tiny_size.map(Some),
                    ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                    tiny_size.map(AvailableOf::definite),
                ),
            )
            .expect("tiny top-gutter absolute layout stays ordered");
            let tiny_geometry = tiny_output
                .scroll_geometry
                .expect("tiny geometry is present");
            assert_eq!(tiny_geometry.scrollport().size(), Size::ZERO);
            let tiny_child = tiny_tree.layout(1).expect("tiny absolute child is staged");
            assert_eq!(tiny_child.location, tiny_geometry.scrollport().origin());
            let tiny_input = tiny_tree
                .inputs(1)
                .iter()
                .find(|input| input.run_mode() == RunMode::PerformLayout)
                .expect("tiny absolute child receives a perform input");
            assert_eq!(tiny_input.parent(), Size::splat(Some(S::ZERO)));
            assert_eq!(
                tiny_input.available(),
                Size::splat(AvailableOf::definite(S::ZERO))
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}
