use std::collections::HashMap;

use crate::flex::FlexAxes;
use crate::geometry::PhysicalProgression;
use crate::*;

fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

fn fri04_c03_flex_value(value: f32) -> SizingCalculation {
    SizingCalculation::value(LengthPercentageOf::px(value).expect("test sizing value is finite"))
}

fn fri04_c03_flex_nested(minimum: f32, preferred: f32, maximum: f32) -> SizingCalculation {
    let preferred = SizingCalculation::max(vec![
        fri04_c03_flex_value(preferred),
        SizingCalculation::min(vec![
            fri04_c03_flex_value(preferred - 5.0),
            fri04_c03_flex_value(preferred + 5.0),
        ])
        .expect("nested minimum is nonempty"),
    ])
    .expect("nested maximum is nonempty");
    SizingCalculation::clamp(
        Some(fri04_c03_flex_value(minimum)),
        preferred,
        Some(fri04_c03_flex_value(maximum)),
    )
}

fn fri04_c03_flex_percentage_nested(
    minimum: f32,
    percentage: f32,
    maximum: f32,
) -> SizingCalculation {
    let preferred = SizingCalculation::max(vec![
        SizingCalculation::value(
            LengthPercentageOf::from_percent_fraction(percentage)
                .expect("test percentage is finite"),
        ),
        SizingCalculation::min(vec![
            fri04_c03_flex_value(minimum + 5.0),
            fri04_c03_flex_value(maximum - 5.0),
        ])
        .expect("nested minimum is nonempty"),
    ])
    .expect("nested maximum is nonempty");
    SizingCalculation::clamp(
        Some(fri04_c03_flex_value(minimum)),
        preferred,
        Some(fri04_c03_flex_value(maximum)),
    )
}

#[test]
fn fri04_c04_flex_dispatch_auto_uses_preferred_main_size_but_content_bypasses_it() {
    fn first_known_main_size(
        preferred_main_size: PreferredSize,
        flex_basis: FlexBasis,
    ) -> Option<f32> {
        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    display: Display::Flex,
                    size: Size::new(PreferredSize::px(200.0), PreferredSize::px(40.0)),
                    ..NodeInput::default()
                },
            )
            .style(
                2,
                NodeInput {
                    size: Size::new(preferred_main_size, PreferredSize::px(20.0)),
                    min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                    flex_basis,
                    ..NodeInput::default()
                },
            )
            .measure(2, ComputeOutput::from_outer_size(Size::new(25.0, 20.0)));

        compute_flex(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(300.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::definite(300.0)),
            ),
        )
        .expect("supported flex basis resolves");

        tree.inputs(2)
            .first()
            .expect("flex item is measured")
            .known()
            .width
    }

    assert_eq!(
        first_known_main_size(PreferredSize::px(80.0), FlexBasis::AUTO),
        Some(80.0)
    );
    assert_eq!(
        first_known_main_size(PreferredSize::AUTO, FlexBasis::AUTO),
        None
    );
    assert_eq!(
        first_known_main_size(PreferredSize::px(80.0), FlexBasis::CONTENT),
        None
    );
}

fn fri04_c04_flex_dispatch_first_item_input(
    direction: FlexDirection,
    container_main: Option<f32>,
    child: NodeInput,
) -> ComputeInput {
    let container_size = match direction {
        FlexDirection::Row | FlexDirection::RowReverse => Size::new(
            container_main.map_or(PreferredSize::AUTO, PreferredSize::px),
            PreferredSize::px(100.0),
        ),
        FlexDirection::Column | FlexDirection::ColumnReverse => Size::new(
            PreferredSize::px(100.0),
            container_main.map_or(PreferredSize::AUTO, PreferredSize::px),
        ),
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                size: container_size,
                flex_direction: direction,
                ..NodeInput::default()
            },
        )
        .style(2, child)
        .measure(2, ComputeOutput::from_outer_size(Size::new(25.0, 20.0)));

    compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(300.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .expect("supported flex dispatch resolves");

    *tree
        .inputs(2)
        .first()
        .expect("flex item receives a sizing input")
}

#[test]
fn fri04_c04_flex_dispatch_numeric_and_calc_size_bases_use_each_physical_main_axis() {
    let ordinary = || {
        FlexBasis::calculation(SizingCalculation::value(
            LengthPercentageOf::from_percent_fraction(0.5).expect("finite percentage"),
        ))
    };
    let any = || {
        FlexBasis::calc_size(
            FlexBasisCalcBasis::Any,
            CalcSizeCalculation::from_coefficients(10.0, 0.5, 0.0).expect("finite Any calculation"),
        )
        .expect("Any calculation does not reference size")
    };
    let full = || {
        FlexBasis::calc_size(
            FlexBasisCalcBasis::FullPercentage,
            CalcSizeCalculation::from_coefficients(10.0, 0.1, 0.5)
                .expect("finite FullPercentage calculation"),
        )
        .expect("valid FullPercentage calculation")
    };

    for (direction, axis) in [
        (FlexDirection::Row, PhysicalAxis::Horizontal),
        (FlexDirection::Column, PhysicalAxis::Vertical),
    ] {
        for (basis, expected) in [(ordinary(), 100.0), (any(), 110.0), (full(), 130.0)] {
            let input = fri04_c04_flex_dispatch_first_item_input(
                direction,
                Some(200.0),
                NodeInput {
                    min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                    flex_basis: basis,
                    ..NodeInput::default()
                },
            );
            assert_eq!(
                match axis {
                    PhysicalAxis::Horizontal => input.known().width,
                    PhysicalAxis::Vertical => input.known().height,
                },
                Some(expected)
            );
        }

        let any_missing = fri04_c04_flex_dispatch_first_item_input(
            direction,
            None,
            NodeInput {
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: any(),
                ..NodeInput::default()
            },
        );
        let full_missing = fri04_c04_flex_dispatch_first_item_input(
            direction,
            None,
            NodeInput {
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: full(),
                ..NodeInput::default()
            },
        );
        let ordinary_missing = fri04_c04_flex_dispatch_first_item_input(
            direction,
            None,
            NodeInput {
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: ordinary(),
                ..NodeInput::default()
            },
        );
        let main = |input: ComputeInput| match axis {
            PhysicalAxis::Horizontal => input.known().width,
            PhysicalAxis::Vertical => input.known().height,
        };
        assert_eq!(main(any_missing), Some(10.0));
        assert_eq!(main(full_missing), None);
        assert_eq!(main(ordinary_missing), None);
    }
}

fn fri04_c04_flex_dispatch_assert_error(
    container: NodeInput,
    child: NodeInput,
    property: SizingProperty,
    behavior: SizingBehavior,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
) {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(1, container)
        .style(2, child)
        .measure(2, ComputeOutput::from_outer_size(Size::splat(10.0)));
    let error = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(200.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::definite(200.0)),
        ),
    )
    .expect_err("later-owned flex sizing must be rejected");
    assert_eq!(error.site(), LayoutErrorSite::Node(2));
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected exact sizing capability, got {:?}", error.kind());
    };
    assert_eq!(
        (
            unsupported.property(),
            unsupported.behavior(),
            unsupported.algorithm(),
            unsupported.axis(),
        ),
        (property, behavior, algorithm, axis)
    );
}

#[test]
fn fri04_c04_flex_dispatch_direct_and_keyword_bases_return_exact_payloads() {
    let sizing =
        || SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite calculation"));
    let calc = || CalcSizeCalculation::value(LengthPercentageOf::ZERO);
    let container = || NodeInput {
        display: Display::Flex,
        size: Size::new(PreferredSize::px(200.0), PreferredSize::px(100.0)),
        ..NodeInput::default()
    };

    for (value, behavior) in [
        (PreferredSize::MIN_CONTENT, SizingBehavior::MinContent),
        (PreferredSize::MAX_CONTENT, SizingBehavior::MaxContent),
        (PreferredSize::STRETCH, SizingBehavior::Stretch),
        (PreferredSize::FIT_CONTENT, SizingBehavior::FitContent),
        (PreferredSize::CONTAIN, SizingBehavior::Contain),
        (
            PreferredSize::fit_content_function(sizing()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                size: Size::new(value, PreferredSize::AUTO),
                ..NodeInput::default()
            },
            SizingProperty::Preferred,
            behavior,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
        );
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
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                min_size: Size::new(MinSize::AUTO, value),
                ..NodeInput::default()
            },
            SizingProperty::Minimum,
            behavior,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
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
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                max_size: Size::new(value, MaxSize::NONE),
                ..NodeInput::default()
            },
            SizingProperty::Maximum,
            behavior,
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
        );
    }
    for (value, behavior) in [
        (FlexBasis::MIN_CONTENT, SizingBehavior::MinContent),
        (FlexBasis::MAX_CONTENT, SizingBehavior::MaxContent),
        (FlexBasis::STRETCH, SizingBehavior::Stretch),
        (FlexBasis::FIT_CONTENT, SizingBehavior::FitContent),
        (FlexBasis::CONTAIN, SizingBehavior::Contain),
        (
            FlexBasis::fit_content_function(sizing()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_flex_dispatch_assert_error(
            NodeInput {
                flex_direction: FlexDirection::Column,
                ..container()
            },
            NodeInput {
                flex_basis: value,
                ..NodeInput::default()
            },
            SizingProperty::FlexBasis,
            behavior,
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
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
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                size: Size::new(
                    PreferredSize::calc_size(basis, calc()).expect("valid calc-size"),
                    PreferredSize::AUTO,
                ),
                ..NodeInput::default()
            },
            SizingProperty::Preferred,
            SizingBehavior::CalcSize(expected),
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
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
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                min_size: Size::new(
                    MinSize::AUTO,
                    MinSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                ..NodeInput::default()
            },
            SizingProperty::Minimum,
            SizingBehavior::CalcSize(expected),
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
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
        fri04_c04_flex_dispatch_assert_error(
            container(),
            NodeInput {
                max_size: Size::new(
                    MaxSize::calc_size(basis, calc()).expect("valid calc-size"),
                    MaxSize::NONE,
                ),
                ..NodeInput::default()
            },
            SizingProperty::Maximum,
            SizingBehavior::CalcSize(expected),
            SizingAlgorithm::Flex,
            PhysicalAxis::Horizontal,
        );
    }
    for (basis, expected) in [
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
        fri04_c04_flex_dispatch_assert_error(
            NodeInput {
                flex_direction: FlexDirection::Column,
                ..container()
            },
            NodeInput {
                flex_basis: FlexBasis::calc_size(basis, calc()).expect("valid calc-size"),
                ..NodeInput::default()
            },
            SizingProperty::FlexBasis,
            SizingBehavior::CalcSize(expected),
            SizingAlgorithm::Flex,
            PhysicalAxis::Vertical,
        );
    }
}

#[test]
fn fri04_c04_flex_dispatch_container_item_root_and_absolute_report_consuming_algorithm() {
    let container_style = || NodeInput {
        display: Display::Flex,
        size: Size::new(PreferredSize::px(200.0), PreferredSize::px(100.0)),
        ..NodeInput::default()
    };

    fri04_c04_flex_dispatch_assert_error(
        container_style(),
        NodeInput {
            size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
            ..NodeInput::default()
        },
        SizingProperty::Preferred,
        SizingBehavior::MinContent,
        SizingAlgorithm::Flex,
        PhysicalAxis::Horizontal,
    );
    fri04_c04_flex_dispatch_assert_error(
        container_style(),
        NodeInput {
            position: Position::Absolute,
            size: Size::new(PreferredSize::AUTO, PreferredSize::MAX_CONTENT),
            ..NodeInput::default()
        },
        SizingProperty::Preferred,
        SizingBehavior::MaxContent,
        SizingAlgorithm::Positioned,
        PhysicalAxis::Vertical,
    );

    let mut container_tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                size: Size::new(PreferredSize::STRETCH, PreferredSize::px(100.0)),
                ..NodeInput::default()
            },
        );
    let container_error = compute_flex(
        &mut container_tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(200.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::definite(200.0)),
        ),
    )
    .expect_err("flex container stretch is later-owned");
    assert_eq!(container_error.site(), LayoutErrorSite::Node(1));
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        container_unsupported,
    )) = container_error.kind()
    else {
        panic!("expected flex container sizing capability");
    };
    assert_eq!(container_unsupported.algorithm(), SizingAlgorithm::Flex);

    struct FlexRootTree {
        style: NodeInput,
    }

    impl Traverse for FlexRootTree {
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
            panic!("flex root has no children")
        }
    }

    impl LayoutTree for FlexRootTree {
        type MeasureError = core::convert::Infallible;

        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, _node: Self::Node) -> LayoutInput {
            LayoutInput::box_input(self.style.clone())
        }
    }

    let root = FlexRootTree {
        style: NodeInput {
            display: Display::Flex,
            min_size: Size::new(MinSize::AUTO, MinSize::STRETCH),
            ..NodeInput::default()
        },
    };
    let root_error = compute_layout(
        &root,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(100.0)))
            .expect("valid root request"),
    )
    .expect_err("flex root minimum stretch is later-owned");
    assert_eq!(root_error.site(), LayoutErrorSite::Node(0));
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        root_unsupported,
    )) = root_error.kind()
    else {
        panic!("expected flex root sizing capability");
    };
    assert_eq!(root_unsupported.algorithm(), SizingAlgorithm::Flex);
    assert_eq!(root_unsupported.property(), SizingProperty::Minimum);
    assert_eq!(root_unsupported.axis(), PhysicalAxis::Vertical);
}

#[test]
fn fri04_c04_flex_dispatch_invalid_numeric_preserves_item_node_site() {
    let invalid = || {
        SizingCalculation::value(
            LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX)
                .expect("finite sizing coefficients"),
        )
    };
    let styles = [
        NodeInput {
            size: Size::new(PreferredSize::calculation(invalid()), PreferredSize::AUTO),
            ..NodeInput::default()
        },
        NodeInput {
            min_size: Size::new(MinSize::calculation(invalid()), MinSize::AUTO),
            ..NodeInput::default()
        },
        NodeInput {
            max_size: Size::new(MaxSize::calculation(invalid()), MaxSize::NONE),
            ..NodeInput::default()
        },
        NodeInput {
            flex_basis: FlexBasis::calculation(invalid()),
            ..NodeInput::default()
        },
    ];

    for style in styles {
        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    display: Display::Flex,
                    size: Size::new(PreferredSize::px(200.0), PreferredSize::px(200.0)),
                    ..NodeInput::default()
                },
            )
            .style(2, style);
        let error = compute_flex(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(200.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::definite(200.0)),
            ),
        )
        .expect_err("overflowing flex sizing calculation must fail");
        assert_eq!(error.site(), LayoutErrorSite::Node(2));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert!(matches!(
            error.kind(),
            LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { value })
                if *value == f32::INFINITY
        ));
    }
}

#[test]
fn fri04_c03_flex_row_layout_consumes_nested_container_item_and_absolute_properties() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2, 3, 4, 5, 6])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(180.0, 200.0, 220.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(100.0, 120.0, 140.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(150.0, 170.0, 190.0)),
                    MinSize::calculation(fri04_c03_flex_nested(80.0, 90.0, 110.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(200.0, 230.0, 250.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(120.0, 150.0, 170.0)),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(60.0, 80.0, 100.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(40.0, 60.0, 80.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(20.0, 40.0, 60.0)),
                    MinSize::calculation(fri04_c03_flex_nested(30.0, 50.0, 70.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(80.0, 100.0, 120.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(45.0, 55.0, 65.0)),
                ),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_percentage_nested(
                    60.0, 0.35, 80.0,
                )),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(-40.0, -20.0, -10.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(-30.0, -15.0, -5.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(-30.0, -20.0, -10.0)),
                    MinSize::calculation(fri04_c03_flex_nested(-20.0, -10.0, -5.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(-30.0, -20.0, -10.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(-20.0, -10.0, -5.0)),
                ),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_nested(-30.0, -10.0, -5.0)),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                position: Position::Absolute,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(70.0, 90.0, 110.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(35.0, 45.0, 55.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(50.0, 60.0, 70.0)),
                    MinSize::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(90.0, 100.0, 120.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(40.0, 45.0, 50.0)),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(10.0, 20.0, 30.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(30.0, 40.0, 50.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                ),
                ..NodeInput::default()
            },
        )
        .style(
            6,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(10.0, 20.0, 30.0)),
                    PreferredSize::calculation(fri04_c03_flex_percentage_nested(20.0, 0.2, 30.0)),
                ),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_nested(10.0, 20.0, 30.0)),
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
            Size::new(Some(300.0), Some(240.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::definite(240.0)),
        ),
    )
    .expect("row flex calculations resolve");

    assert_eq!(output.size, Size::new(200.0, 120.0));
    assert_eq!(
        tree.output(2).expect("normal child is laid out").size,
        Size::new(70.0, 55.0)
    );
    assert_eq!(
        tree.output(3).expect("negative child is laid out").size,
        Size::ZERO
    );
    assert_eq!(
        tree.output(4).expect("absolute child is laid out").size,
        Size::new(90.0, 45.0)
    );
    assert_eq!(
        tree.output(5)
            .expect("automatic-minimum child is laid out")
            .size,
        Size::new(30.0, 20.0)
    );
    assert_eq!(
        tree.output(6)
            .expect("basis-dependent final-known child is laid out")
            .size,
        Size::new(20.0, 25.0)
    );
}

#[test]
fn fri04_c03_flex_column_layout_maps_nested_main_and_cross_calculations_to_physical_axes() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(180.0)),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_nested(35.0, 45.0, 55.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(70.0, 90.0, 110.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_flex_nested(30.0, 40.0, 50.0)),
                    MinSize::calculation(fri04_c03_flex_nested(40.0, 50.0, 60.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_flex_nested(38.0, 42.0, 48.0)),
                    MaxSize::calculation(fri04_c03_flex_nested(90.0, 100.0, 120.0)),
                ),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_percentage_nested(
                    60.0,
                    75.0 / 180.0,
                    90.0,
                )),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_flex_percentage_nested(20.0, 0.2, 40.0)),
                    PreferredSize::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
                ),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_nested(20.0, 30.0, 40.0)),
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
            Size::new(Some(140.0), Some(180.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(140.0), Available::definite(180.0)),
        ),
    )
    .expect("column flex calculations resolve");

    assert_eq!(
        tree.output(2).expect("column child is laid out").size,
        Size::new(42.0, 75.0)
    );
    assert_eq!(
        tree.inputs(2)
            .last()
            .expect("final child request is recorded")
            .known(),
        Size::new(Some(42.0), Some(75.0))
    );
    assert_eq!(
        tree.output(3)
            .expect("column final-known child is laid out")
            .size,
        Size::new(28.0, 30.0)
    );
}

#[test]
fn fri04_c03_flex_compute_size_missing_numeric_basis_uses_content_not_authored_main_size() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(PreferredSize::px(90.0), PreferredSize::px(10.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_basis: FlexBasis::calculation(fri04_c03_flex_percentage_nested(
                    10.0, 0.5, 80.0,
                )),
                ..NodeInput::default()
            },
        )
        .measure(
            2,
            ComputeOutput::from_sizes(Size::new(35.0, 10.0), Size::new(35.0, 10.0)),
        );

    let output = compute_flex(
        &mut tree,
        1,
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
    .expect("missing flex-basis percentage context uses content sizing");

    assert_eq!(output.size, Size::new(35.0, 10.0));
    assert!(tree.inputs(2).iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize
            && input.sizing_mode() == SizingMode::ContentSize
            && input.parent().width.is_none()
    }));
}

#[test]
fn fri04_c03_flex_invalid_numeric_propagates_for_every_numeric_property_role() {
    let invalid = || {
        SizingCalculation::min(vec![
            SizingCalculation::value(
                LengthPercentageOf::from_coefficients(f32::MAX, 1.0)
                    .expect("finite overflowing coefficients"),
            ),
            fri04_c03_flex_value(10.0),
        ])
        .expect("nested minimum is nonempty")
    };

    for role in ["preferred", "minimum", "maximum", "flex-basis"] {
        let mut child = NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            ..NodeInput::default()
        };
        match role {
            "preferred" => child.size.width = PreferredSize::calculation(invalid()),
            "minimum" => child.min_size.width = MinSize::calculation(invalid()),
            "maximum" => child.max_size.width = MaxSize::calculation(invalid()),
            "flex-basis" => child.flex_basis = FlexBasis::calculation(invalid()),
            _ => unreachable!(),
        }

        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    display: Display::Flex,
                    size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(40.0)),
                    ..NodeInput::default()
                },
            )
            .style(2, child)
            .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

        let error = compute_flex(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(f32::MAX), Some(40.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(Available::definite(f32::MAX), Available::definite(40.0)),
            ),
        )
        .expect_err("invalid numeric flex property must fail");

        assert_eq!(error.site(), LayoutErrorSite::Node(2), "role: {role}");
        assert_eq!(
            error.operation(),
            LayoutOperation::ValueResolution,
            "role: {role}"
        );
        assert_eq!(
            error.kind(),
            &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
                value: f32::INFINITY,
            }),
            "role: {role}"
        );
    }
}

#[test]
fn flex_child_context_is_complete_for_layout_sizing_and_absolute_paths() {
    assert_flex_child_context_is_complete::<f32>();
    assert_flex_child_context_is_complete::<f64>();
}

fn assert_flex_child_context_is_complete<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let flow_axes = FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);
    let expected =
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::Flex);

    for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    writing_mode: WritingMode::VerticalLr,
                    direction: Direction::Ltr,
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

        crate::compute_flex(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                run_mode,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(300.0)), Some(S::from_f64(240.0))),
                crate::ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(AvailableOf::definite(S::from_f64(300.0))),
            ),
        )
        .expect("flex context capture layout succeeds");

        let normal_inputs = tree.inputs(1);
        assert!(
            !normal_inputs.is_empty(),
            "flex must request its in-flow child"
        );
        assert!(
            normal_inputs
                .iter()
                .all(|input| input.containing_layout_context() == expected),
            "every flex in-flow request must use the parent axes and Flex role: {normal_inputs:#?}"
        );

        if run_mode == RunMode::ComputeSize {
            assert!(
                normal_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::ComputeSize),
                "flex intrinsic sizing must request the child through the complete context"
            );
        } else {
            assert!(
                normal_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::PerformLayout),
                "flex normal layout must request the child through the complete context"
            );
            let absolute_inputs = tree.inputs(2);
            assert!(
                absolute_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::PerformLayout),
                "flex absolute scheduling must request the child"
            );
            assert!(
                absolute_inputs
                    .iter()
                    .all(|input| input.containing_layout_context() == expected),
                "every flex absolute request must use the parent axes and Flex role: {absolute_inputs:#?}"
            );
        }
    }
}

#[derive(Clone, Copy)]
struct FlexAxesExpectation {
    main_logical_axis: LogicalAxis,
    cross_logical_axis: LogicalAxis,
    main_physical_axis: PhysicalAxis,
    cross_physical_axis: PhysicalAxis,
    main_start_side: PhysicalSide,
    main_end_side: PhysicalSide,
    cross_start_side: PhysicalSide,
    cross_end_side: PhysicalSide,
    main_reversed: bool,
    cross_reversed: bool,
    main_progression: PhysicalProgression,
    cross_progression: PhysicalProgression,
}

#[derive(Clone, Copy)]
struct FlexAxesCase {
    writing_mode: WritingMode,
    direction: Direction,
    flex_direction: FlexDirection,
    normal: FlexAxesExpectation,
    wrap_reverse: FlexAxesExpectation,
}

fn assert_flex_axes_expectation(axes: FlexAxes, expectation: FlexAxesExpectation) {
    assert_eq!(axes.main_logical_axis(), expectation.main_logical_axis);
    assert_eq!(axes.cross_logical_axis(), expectation.cross_logical_axis);
    assert_eq!(axes.main_physical_axis(), expectation.main_physical_axis);
    assert_eq!(axes.cross_physical_axis(), expectation.cross_physical_axis);
    assert_eq!(axes.main_start_side(), expectation.main_start_side);
    assert_eq!(axes.main_end_side(), expectation.main_end_side);
    assert_eq!(axes.cross_start_side(), expectation.cross_start_side);
    assert_eq!(axes.cross_end_side(), expectation.cross_end_side);
    assert_eq!(axes.main_is_reversed(), expectation.main_reversed);
    assert_eq!(axes.cross_is_reversed(), expectation.cross_reversed);
    assert_eq!(axes.main_progression(), expectation.main_progression);
    assert_eq!(axes.cross_progression(), expectation.cross_progression);
}

#[test]
fn flex_axes_matrix_covers_all_flows_directions_and_flex_directions() {
    use LogicalAxis::{Block, Inline};
    use PhysicalAxis::{Horizontal, Vertical};
    use PhysicalProgression::{Decreasing, Increasing};
    use PhysicalSide::{Bottom, Left, Right, Top};

    macro_rules! expectation {
        (
            $main_logical_axis:ident,
            $cross_logical_axis:ident,
            $main_physical_axis:ident,
            $cross_physical_axis:ident,
            $main_start_side:ident,
            $main_end_side:ident,
            $cross_start_side:ident,
            $cross_end_side:ident,
            $main_reversed:expr,
            $cross_reversed:expr,
            $main_progression:ident,
            $cross_progression:ident
        ) => {
            FlexAxesExpectation {
                main_logical_axis: $main_logical_axis,
                cross_logical_axis: $cross_logical_axis,
                main_physical_axis: $main_physical_axis,
                cross_physical_axis: $cross_physical_axis,
                main_start_side: $main_start_side,
                main_end_side: $main_end_side,
                cross_start_side: $cross_start_side,
                cross_end_side: $cross_end_side,
                main_reversed: $main_reversed,
                cross_reversed: $cross_reversed,
                main_progression: $main_progression,
                cross_progression: $cross_progression,
            }
        };
    }

    macro_rules! case {
        ($writing_mode:expr, $direction:expr, $flex_direction:expr, $normal:expr, $wrap_reverse:expr) => {
            FlexAxesCase {
                writing_mode: $writing_mode,
                direction: $direction,
                flex_direction: $flex_direction,
                normal: $normal,
                wrap_reverse: $wrap_reverse,
            }
        };
    }

    let cases = [
        case!(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Horizontal, Vertical, Left, Right, Top, Bottom, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Horizontal, Vertical, Left, Right, Bottom, Top, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Horizontal, Vertical, Right, Left, Top, Bottom, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Horizontal, Vertical, Right, Left, Bottom, Top, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Vertical, Horizontal, Top, Bottom, Left, Right, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Vertical, Horizontal, Top, Bottom, Right, Left, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Vertical, Horizontal, Bottom, Top, Left, Right, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Vertical, Horizontal, Bottom, Top, Right, Left, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Horizontal, Vertical, Right, Left, Top, Bottom, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Horizontal, Vertical, Right, Left, Bottom, Top, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Horizontal, Vertical, Left, Right, Top, Bottom, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Horizontal, Vertical, Left, Right, Bottom, Top, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Vertical, Horizontal, Top, Bottom, Right, Left, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Vertical, Horizontal, Top, Bottom, Left, Right, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Vertical, Horizontal, Bottom, Top, Right, Left, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Vertical, Horizontal, Bottom, Top, Left, Right, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, false, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, false, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, true, false,
                Increasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, true, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, false, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, false, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalRl,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, true, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, true, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::VerticalLr,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, false, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, false, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, true, false,
                Increasing, Decreasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, true, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, false, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, false, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysRl,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, true, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, true, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Ltr,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, false, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, false, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Ltr,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, true, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, true, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Ltr,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, false, false,
                Increasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, false, true,
                Increasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Ltr,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, true, false,
                Decreasing, Decreasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, true, true,
                Decreasing, Increasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Rtl,
            FlexDirection::Row,
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Left, Right, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Top, Bottom, Right, Left, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Rtl,
            FlexDirection::RowReverse,
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Left, Right, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Inline, Block, Vertical, Horizontal, Bottom, Top, Right, Left, true, true,
                Decreasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Rtl,
            FlexDirection::Column,
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Top, Bottom, false, false,
                Increasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Left, Right, Bottom, Top, false, true,
                Increasing, Decreasing
            )
        ),
        case!(
            WritingMode::SidewaysLr,
            Direction::Rtl,
            FlexDirection::ColumnReverse,
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Top, Bottom, true, false,
                Decreasing, Increasing
            ),
            expectation!(
                Block, Inline, Horizontal, Vertical, Right, Left, Bottom, Top, true, true,
                Decreasing, Decreasing
            )
        ),
    ];

    assert_eq!(cases.len(), 40);
    for case in cases {
        let flow_axes = FlowAxes::new(case.writing_mode, case.direction);
        let normal = FlexAxes::new(flow_axes, case.flex_direction, FlexWrap::Wrap);
        let wrap_reverse = FlexAxes::new(flow_axes, case.flex_direction, FlexWrap::WrapReverse);

        assert_eq!(normal.flow_direction(), case.direction);
        assert_eq!(wrap_reverse.flow_direction(), case.direction);
        assert_flex_axes_expectation(normal, case.normal);
        assert_flex_axes_expectation(wrap_reverse, case.wrap_reverse);

        assert_eq!(normal.main_logical_axis(), wrap_reverse.main_logical_axis());
        assert_eq!(
            normal.cross_logical_axis(),
            wrap_reverse.cross_logical_axis()
        );
        assert_eq!(
            normal.main_physical_axis(),
            wrap_reverse.main_physical_axis()
        );
        assert_eq!(
            normal.cross_physical_axis(),
            wrap_reverse.cross_physical_axis()
        );
        assert_eq!(normal.main_start_side(), wrap_reverse.main_start_side());
        assert_eq!(normal.main_end_side(), wrap_reverse.main_end_side());
        assert_eq!(normal.main_is_reversed(), wrap_reverse.main_is_reversed());
        assert_eq!(normal.main_progression(), wrap_reverse.main_progression());
        assert_ne!(normal.cross_start_side(), wrap_reverse.cross_start_side());
        assert_ne!(normal.cross_end_side(), wrap_reverse.cross_end_side());
        assert_ne!(normal.cross_progression(), wrap_reverse.cross_progression());
    }
}

#[test]
fn flex_axes_selectors_and_mutators_follow_the_resolved_mapping() {
    let axes = FlexAxes::new(
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        FlexDirection::ColumnReverse,
        FlexWrap::WrapReverse,
    );
    let size = Size::new(3.0, 5.0);
    let point = Point::new(7.0, 11.0);
    let mut edges = Edges::new(2.0, 3.0, 5.0, 7.0);

    assert_eq!(
        axes.flow_axes(),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr)
    );
    assert_eq!(axes.main_size(size), 3.0);
    assert_eq!(axes.cross_size(size), 5.0);
    assert_eq!(axes.size_from_main_cross(13.0, 17.0), Size::new(13.0, 17.0));
    assert_eq!(axes.with_main_size(size, 19.0), Size::new(19.0, 5.0));
    assert_eq!(axes.with_cross_size(size, 23.0), Size::new(3.0, 23.0));
    assert_eq!(axes.main_point(point), 7.0);
    assert_eq!(axes.cross_point(point), 11.0);
    assert_eq!(
        axes.point_from_main_cross(29.0, 31.0),
        Point::new(29.0, 31.0)
    );

    assert_eq!(axes.main_start_edge(edges), 3.0);
    assert_eq!(axes.main_end_edge(edges), 7.0);
    assert_eq!(axes.cross_start_edge(edges), 2.0);
    assert_eq!(axes.cross_end_edge(edges), 5.0);
    assert_eq!(axes.main_edge_sum(edges), 10.0);
    assert_eq!(axes.cross_edge_sum(edges), 7.0);
    axes.set_main_start_edge(&mut edges, 37.0);
    axes.set_main_end_edge(&mut edges, 41.0);
    axes.set_cross_start_edge(&mut edges, 43.0);
    axes.set_cross_end_edge(&mut edges, 47.0);
    assert_eq!(edges, Edges::new(43.0, 37.0, 47.0, 41.0));

    assert_eq!(axes.main_requested_axis(), crate::RequestedAxis::Horizontal);
    assert_eq!(axes.cross_requested_axis(), crate::RequestedAxis::Vertical);
    assert_eq!(
        axes.main_size_from_cross_aspect(
            11.0,
            AspectRatio::new(2.0).expect("finite positive aspect ratio"),
        ),
        22.0
    );

    let vertical_main = FlexAxes::new(
        FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        FlexDirection::Row,
        FlexWrap::NoWrap,
    );
    assert_eq!(
        vertical_main.main_size_from_cross_aspect(
            22.0,
            AspectRatio::new(2.0).expect("finite positive aspect ratio"),
        ),
        11.0
    );
    assert_eq!(vertical_main.main_size(size), 5.0);
    assert_eq!(vertical_main.cross_size(size), 3.0);
    assert_eq!(
        vertical_main.size_from_main_cross(13.0, 17.0),
        Size::new(17.0, 13.0)
    );
    assert_eq!(
        vertical_main.with_main_size(size, 19.0),
        Size::new(3.0, 19.0)
    );
    assert_eq!(
        vertical_main.with_cross_size(size, 23.0),
        Size::new(23.0, 5.0)
    );
    assert_eq!(vertical_main.main_point(point), 11.0);
    assert_eq!(vertical_main.cross_point(point), 7.0);
    assert_eq!(vertical_main.main_requested_axis(), RequestedAxis::Vertical);
    assert_eq!(
        vertical_main.cross_requested_axis(),
        RequestedAxis::Horizontal
    );
    assert_eq!(
        vertical_main.point_from_main_cross(29.0, 31.0),
        Point::new(31.0, 29.0)
    );

    let mut vertical_edges = Edges::new(2.0, 3.0, 5.0, 7.0);
    assert_eq!(vertical_main.main_start_edge(vertical_edges), 5.0);
    assert_eq!(vertical_main.main_end_edge(vertical_edges), 2.0);
    assert_eq!(vertical_main.cross_start_edge(vertical_edges), 7.0);
    assert_eq!(vertical_main.cross_end_edge(vertical_edges), 3.0);
    assert_eq!(vertical_main.main_edge_sum(vertical_edges), 7.0);
    assert_eq!(vertical_main.cross_edge_sum(vertical_edges), 10.0);
    vertical_main.set_main_start_edge(&mut vertical_edges, 37.0);
    vertical_main.set_main_end_edge(&mut vertical_edges, 41.0);
    vertical_main.set_cross_start_edge(&mut vertical_edges, 43.0);
    vertical_main.set_cross_end_edge(&mut vertical_edges, 47.0);
    assert_eq!(vertical_edges, Edges::new(41.0, 47.0, 37.0, 43.0));
    assert_eq!(
        FlexAxes::new(
            FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
            FlexDirection::Row,
            FlexWrap::Wrap,
        ),
        vertical_main
    );
}

fn output_from_known_or(input: ComputeInput, fallback: Size) -> ComputeOutput {
    let size = Size::new(
        input.known().width.unwrap_or(fallback.width),
        input.known().height.unwrap_or(fallback.height),
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
fn flex_direction_retains_row_column_and_reverse_classification() {
    assert!(FlexDirection::Row.is_row());
    assert!(FlexDirection::RowReverse.is_row());
    assert!(FlexDirection::Column.is_column());
    assert!(FlexDirection::ColumnReverse.is_column());
    assert!(!FlexDirection::Row.is_reverse());
    assert!(FlexDirection::RowReverse.is_reverse());
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
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(30.0)),
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

    assert_eq!(output.size, Size::new(200.0, 42.0));
    assert_eq!(output.content_size, Size::new(198.0, 40.0));

    assert_eq!(tree.layouts[&2].location, Point::new(6.0, 6.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(56.0, 6.0));
    assert_eq!(tree.layouts[&3].size, Size::new(30.0, 30.0));

    assert_eq!(
        tree.inputs[&2][0].known(),
        Size::new(Some(40.0), Some(20.0))
    );
    assert_eq!(
        tree.inputs[&3][0].known(),
        Size::new(Some(30.0), Some(30.0))
    );
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
                size: Size::new(PreferredSizeOf::px(container_width), PreferredSizeOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
                size: Size::new(PreferredSizeOf::px(20.125), PreferredSizeOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::Block,
                flex_grow: FlexGrowOf::try_new(3.0).unwrap(),
                size: Size::new(PreferredSizeOf::px(20.125), PreferredSizeOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        );

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInputOf::<f64>::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(container_width), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(container_width),
                AvailableOf::MAX_CONTENT,
            ),
        ),
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
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
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
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 20.0)));

    let error = compute_flex(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(120.0), Some(40.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(120.0), Available::definite(40.0)),
        ),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
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
                let size = if input.run_mode() == RunMode::PerformLayout
                    && input.known().width == Some(80.0)
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
            size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
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

    assert!(tree.inputs[&1].iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize && input.known().width == Some(80.0)
    }));
    assert!(tree.inputs[&1].iter().any(|input| {
        input.run_mode() == RunMode::PerformLayout && input.known().width == Some(80.0)
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = compute_flex(
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
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    let output = compute_flex(
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
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(20.0, 10.0));
    assert_eq!(tree.inputs[&2][0].run_mode(), RunMode::ComputeSize);
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
                ComputeOutput::from_outer_size(Size::new(0.0, input.known().height.unwrap_or(10.0)))
            })
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(10.0)),
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
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(50.0), Some(10.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(50.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    let base_input = tree.inputs[&2][0];
    assert_eq!(base_input.sizing_mode(), SizingMode::ContentSize);
    assert_eq!(base_input.known().width, None);
    assert_eq!(base_input.known().height, Some(10.0));
    assert_eq!(base_input.available().width, Available::MAX_CONTENT);
    assert_eq!(base_input.available().height, Available::definite(10.0));
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
                    input.known().width.unwrap_or(40.0),
                    input.known().height.unwrap_or(50.0),
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
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
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
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(20.0), Some(50.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        ),
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
                    input.known().width.unwrap_or(40.0),
                    input.known().height.unwrap_or(50.0),
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
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Hidden),
            flex_basis: FlexBasis::px(0.0),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
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
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(20.0), Some(50.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        ),
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
                    input.known().width.unwrap_or(40.0),
                    input.known().height.unwrap_or(50.0),
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
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            flex_basis: FlexBasis::px(0.0),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
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
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(20.0), Some(50.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(20.0), Available::MAX_CONTENT),
        ),
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
            min_size: Size::new(MinSize::px(100.0), MinSize::px(40.0)),
            max_size: Size::new(MaxSize::px(100.0), MaxSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = compute_flex(
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
                if input.run_mode() == RunMode::PerformLayout {
                    ComputeOutput::from_sizes(
                        Size::new(input.known().width.unwrap(), input.known().height.unwrap()),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::None,
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
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

    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(
        tree.layouts[&3],
        NodeOutput::with_source_index(crate::SourceIndex::new(1))
    );
    assert_eq!(
        tree.inputs[&3],
        vec![ComputeInput::hidden(crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr,),
            crate::ParentFormattingContext::Flex
        ))]
    );
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
                        Size::new(input.known().width.unwrap(), input.known().height.unwrap()),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(25.0), PreferredSize::px(10.0)),
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
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(12.0)),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
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
    assert_eq!(output.content_size, Size::new(100.0, 41.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(25.0, 10.0));
    assert_eq!(tree.layouts[&3].location, Point::new(7.0, 9.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 12.0));
    assert_eq!(
        tree.inputs[&3][0].known(),
        Size::new(Some(20.0), Some(12.0))
    );
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
            size: Size::new(PreferredSize::px(400.0), PreferredSize::px(300.0)),
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
        tree.inputs[&2][0].known(),
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
            size: Size::new(PreferredSize::px(400.0), PreferredSize::px(300.0)),
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
        tree.inputs[&2][0].known(),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
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
            max_size: Size::new(MaxSize::NONE, MaxSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
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
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
                        input.known().width.unwrap_or(0.0),
                        input.known().height.unwrap_or(0.0),
                    ),
                    Size::new(
                        input.known().width.unwrap_or(0.0),
                        input.known().height.unwrap_or(0.0),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
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
                        input.known().width.unwrap_or(0.0),
                        input.known().height.unwrap_or(0.0),
                    ),
                    Size::new(
                        input.known().width.unwrap_or(0.0),
                        input.known().height.unwrap_or(0.0),
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
        authored_size.inputs[&2][0].known(),
        Size::new(Some(22.0), Some(14.0))
    );
    assert_eq!(authored_size.layouts[&2].size, Size::new(22.0, 14.0));

    let mut max_size = tree_with_child(NodeInput {
        position: Position::Absolute,
        max_size: Size::new(MaxSize::px(12.0), MaxSize::px(12.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            justify_content: Some(AlignContent::Center),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
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

    assert_eq!(output.size, Size::new(200.0, 20.0));
    assert_eq!(output.content_size, Size::new(200.0, 20.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(105.0, 20.0));
    assert_eq!(tree.layouts[&3].location, Point::new(105.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(95.0, 20.0));

    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
        Size::new(Some(105.0), Some(20.0))
    );
    assert_eq!(
        tree.inputs[&3].last().unwrap().known(),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            flex_grow: FlexGrowOf::try_new(0.5).unwrap(),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(70.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
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
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(0.0),
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
            size: Size::new(PreferredSize::px(730.0), PreferredSize::px(300.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(100.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            padding: Edges::all(Length::px(10.0)),
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
            Size::new(Some(730.0), Some(300.0)),
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

    assert_eq!(tree.layouts[&2].size.width, 730.0);
    assert_eq!(
        tree.inputs[&2]
            .last()
            .expect("child should be laid out")
            .known()
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
                    && input.run_mode() == RunMode::ComputeSize
                    && input.available().width == Available::MIN_CONTENT
                {
                    return Ok(ComputeOutput::from_outer_size(Size::new(90.0, 20.0)));
                }

                let fallback = if node == 2 {
                    Size::new(160.0, 20.0)
                } else {
                    Size::new(40.0, 20.0)
                };
                ComputeOutput::from_outer_size(Size::new(
                    input.known().width.unwrap_or(fallback.width),
                    input.known().height.unwrap_or(fallback.height),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(20.0)),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
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

    assert_eq!(output.size, Size::new(100.0, 20.0));
    assert!(
        tree.inputs[&2].iter().any(|input| {
            input.run_mode() == RunMode::ComputeSize
                && input.available().width == Available::MIN_CONTENT
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
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(10.0)),
            min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
            flex_shrink: FlexShrinkOf::try_new(0.5).unwrap(),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            flex_wrap: FlexWrap::Wrap,
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3, 4] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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

    assert_eq!(output.size, Size::new(100.0, 38.0));
    assert_eq!(output.content_size, Size::new(100.0, 38.0));
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
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            flex_wrap: FlexWrap::Wrap,
            gap: Size::new(Length::ZERO, Length::px(4.0)),
            ..NodeInput::default()
        },
    );
    for child in [2, 3, 4] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 38.0));
    assert_eq!(output.content_size, Size::new(100.0, 38.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 14.0));
    assert_eq!(tree.layouts[&4].location, Point::new(0.0, 28.0));
}

#[test]
fn flex_order_modified_sequence_precedes_wrapping_and_preserves_source_identity_in_both_scalar_lanes()
 {
    assert_flex_order_modified_sequence_precedes_wrapping::<f32>();
    assert_flex_order_modified_sequence_precedes_wrapping::<f64>();
}

fn assert_flex_order_modified_sequence_precedes_wrapping<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    for (direction, expected_x) in [
        (FlexDirection::Row, [0.0, 30.0, 0.0, 30.0]),
        (FlexDirection::RowReverse, [30.0, 0.0, 30.0, 0.0]),
    ] {
        let item_style = |order| NodeInputOf::<S> {
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.0)),
                PreferredSizeOf::px(S::from_f64(10.0)),
            ),
            item_order: ItemOrder::new(order),
            flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
            ..NodeInputOf::default()
        };
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2, 3, 4, 5, 6])
            .children(1, [])
            .children(2, [])
            .children(3, [])
            .children(4, [])
            .children(5, [])
            .children(6, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    flex_direction: direction,
                    flex_wrap: FlexWrap::Wrap,
                    align_content: Some(AlignContent::FlexStart),
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(60.0)),
                        PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(1, item_style(2))
            .style(
                2,
                NodeInputOf {
                    display: Display::None,
                    item_order: ItemOrder::new(i32::MIN),
                    ..item_style(0)
                },
            )
            .style(3, item_style(-1))
            .style(
                4,
                NodeInputOf {
                    position: Position::Absolute,
                    item_order: ItemOrder::new(i32::MIN),
                    ..item_style(0)
                },
            )
            .style(5, item_style(2))
            .style(6, item_style(0));

        compute_flex(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(60.0)), Some(S::from_f64(20.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(
                    AvailableOf::definite(S::from_f64(60.0)),
                    AvailableOf::definite(S::from_f64(20.0)),
                ),
            ),
        )
        .expect("order-modified wrapped flex layout succeeds");

        for (node, expected_source, x, y) in [
            (3, 2, expected_x[0], 0.0),
            (6, 5, expected_x[1], 0.0),
            (1, 0, expected_x[2], 10.0),
            (5, 4, expected_x[3], 10.0),
        ] {
            let layout = tree.layout(node).expect("in-flow child layout is staged");
            assert_eq!(layout.source_index, SourceIndex::new(expected_source));
            assert_eq!(layout.location, Point::new(S::from_f64(x), S::from_f64(y)));
        }
        assert!(
            tree.inputs(2)
                .iter()
                .any(|input| input.run_mode() == RunMode::PerformHiddenLayout),
            "hidden child scheduling remains outside the in-flow permutation"
        );
        assert_eq!(
            tree.layout(4)
                .expect("absolute child layout is staged")
                .source_index,
            SourceIndex::new(3)
        );
    }
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                size: Size::new(PreferredSize::px(25.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
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
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(0.0),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
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

    assert_eq!(output.first_baselines.y, Some(7.0));
    assert_eq!(output.last_baselines.y, Some(7.0));
}

fn assert_flex_uses_the_orthogonal_child_line_over_margin_for_baselines<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [2])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::Baseline),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(200.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(70.0)),
                    PreferredSizeOf::px(S::from_f64(110.0)),
                ),
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(3.0)),
                    LengthAutoOf::px(S::from_f64(7.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(19.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(110.0)),
                Size::new(S::from_f64(70.0), S::from_f64(110.0)),
                BaselinesOf {
                    first: Point::new(Some(S::from_f64(17.0)), None),
                    last: Point::new(Some(S::from_f64(29.0)), None),
                },
            ),
        );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(200.0)), Some(S::from_f64(160.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(200.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("flex layout succeeds");

    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(36.0)), None)
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(S::from_f64(36.0)), None)
    );
}

#[test]
fn orthogonal_baseline_flex_uses_line_over_margin_for_f32() {
    assert_flex_uses_the_orthogonal_child_line_over_margin_for_baselines::<f32>();
}

#[test]
fn orthogonal_baseline_flex_uses_line_over_margin_for_f64() {
    assert_flex_uses_the_orthogonal_child_line_over_margin_for_baselines::<f64>();
}

fn assert_flex_translates_orthogonal_child_baselines_on_the_child_block_axis<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [2])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::Baseline),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(200.0)),
                    PreferredSizeOf::px(S::from_f64(160.0)),
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
            2,
            NodeInputOf::<S> {
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(70.0)),
                    PreferredSizeOf::px(S::from_f64(110.0)),
                ),
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(17.0)),
                    LengthAutoOf::px(S::from_f64(7.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(110.0)),
                Size::new(S::from_f64(70.0), S::from_f64(110.0)),
                BaselinesOf {
                    first: Point::new(Some(S::from_f64(17.0)), None),
                    last: Point::new(Some(S::from_f64(29.0)), None),
                },
            ),
        );

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(200.0)), Some(S::from_f64(160.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(200.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("flex layout succeeds");

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(S::from_f64(14.0), S::from_f64(22.0))
    );
    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(31.0)), None)
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(S::from_f64(31.0)), None)
    );
}

#[test]
fn orthogonal_baseline_flex_translation_uses_physical_x_for_f32() {
    assert_flex_translates_orthogonal_child_baselines_on_the_child_block_axis::<f32>();
}

#[test]
fn orthogonal_baseline_flex_translation_uses_physical_x_for_f64() {
    assert_flex_translates_orthogonal_child_baselines_on_the_child_block_axis::<f64>();
}

struct BaselineRefreshTree<S: LayoutScalar> {
    styles: HashMap<u32, NodeInputOf<S>>,
    layouts: HashMap<u32, NodeOutputOf<S>>,
    initial_child_main: S,
}

impl<S: LayoutScalar> Traverse for BaselineRefreshTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        match node {
            1 => [2].iter().copied(),
            _ => [].iter().copied(),
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == 1)
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        2
    }
}

impl<S: LayoutScalar> Compute for BaselineRefreshTree<S> {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.styles[&node].clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S> {
        assert_eq!(node, 2, "the focused flex tree exposes one measured child");
        let main = input.known().height.unwrap_or(self.initial_child_main);
        let size = Size::new(main / S::from_f64(2.0), main);
        Ok(ComputeOutputOf::from_sizes_and_baselines(
            size,
            size,
            BaselinesOf {
                first: Point::new(Some(size.width), None),
                last: Point::new(Some(size.width), None),
            },
        ))
    }
}

fn assert_logical_flex_sizing_orthogonal_refreshes_mapped_main<S: LayoutScalar>(
    container_main: f64,
    child_main: f64,
    expected_child_size: Size<S>,
) {
    let mut tree = BaselineRefreshTree {
        styles: HashMap::from([
            (
                1,
                NodeInputOf::<S> {
                    display: Display::Flex,
                    writing_mode: WritingMode::VerticalRl,
                    flex_direction: FlexDirection::Row,
                    size: Size::new(
                        PreferredSizeOf::AUTO,
                        PreferredSizeOf::px(S::from_f64(container_main)),
                    ),
                    ..NodeInputOf::default()
                },
            ),
            (
                2,
                NodeInputOf::<S> {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalRl,
                    size: Size::new(
                        PreferredSizeOf::AUTO,
                        PreferredSizeOf::px(S::from_f64(child_main)),
                    ),
                    min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    flex_grow: FlexGrowOf::try_new(S::ONE).expect("one is a valid flex grow"),
                    flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                    ..NodeInputOf::default()
                },
            ),
        ]),
        layouts: HashMap::new(),
        initial_child_main: S::from_f64(child_main),
    };

    let output = compute_flex(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(None, Some(S::from_f64(container_main))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::MAX_CONTENT,
                AvailableOf::definite(S::from_f64(container_main)),
            ),
        ),
    )
    .expect("flex layout succeeds");

    assert_eq!(tree.layouts[&2].size, expected_child_size);
    assert_eq!(
        tree.layouts[&2].location,
        Point::new(S::ZERO, S::ZERO),
        "the corrected mapped main size reaches final physical placement"
    );
    assert_eq!(
        output.first_baselines,
        Point::new(Some(expected_child_size.width), None),
        "the refreshed vertical child synthesizes a size-dependent physical-x baseline"
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(expected_child_size.width), None),
        "the refreshed vertical child retains the selected physical-x baseline"
    );
}

fn assert_logical_flex_sizing_orthogonal_refresh_grow<S: LayoutScalar>() {
    assert_logical_flex_sizing_orthogonal_refreshes_mapped_main::<S>(
        160.0,
        40.0,
        Size::new(S::from_f64(80.0), S::from_f64(160.0)),
    );
}

fn assert_logical_flex_sizing_orthogonal_refresh_shrink<S: LayoutScalar>() {
    assert_logical_flex_sizing_orthogonal_refreshes_mapped_main::<S>(
        100.0,
        160.0,
        Size::new(S::from_f64(50.0), S::from_f64(100.0)),
    );
}

#[test]
fn logical_flex_sizing_orthogonal_refresh_grow_for_f32() {
    assert_logical_flex_sizing_orthogonal_refresh_grow::<f32>();
}

#[test]
fn logical_flex_sizing_orthogonal_refresh_grow_for_f64() {
    assert_logical_flex_sizing_orthogonal_refresh_grow::<f64>();
}

#[test]
fn logical_flex_sizing_orthogonal_refresh_shrink_for_f32() {
    assert_logical_flex_sizing_orthogonal_refresh_shrink::<f32>();
}

#[test]
fn logical_flex_sizing_orthogonal_refresh_shrink_for_f64() {
    assert_logical_flex_sizing_orthogonal_refresh_shrink::<f64>();
}

#[test]
fn logical_flex_placement_baseline_refresh_grow_projects_physical_x_for_f32() {
    assert_logical_flex_sizing_orthogonal_refresh_grow::<f32>();
}

#[test]
fn logical_flex_placement_baseline_refresh_grow_projects_physical_x_for_f64() {
    assert_logical_flex_sizing_orthogonal_refresh_grow::<f64>();
}

#[test]
fn logical_flex_placement_baseline_refresh_shrink_projects_physical_x_for_f32() {
    assert_logical_flex_sizing_orthogonal_refresh_shrink::<f32>();
}

#[test]
fn logical_flex_placement_baseline_refresh_shrink_projects_physical_x_for_f64() {
    assert_logical_flex_sizing_orthogonal_refresh_shrink::<f64>();
}

struct FinalSizeSelectorTree<S: LayoutScalar> {
    styles: HashMap<u32, NodeInputOf<S>>,
    layouts: HashMap<u32, NodeOutputOf<S>>,
    final_known: Option<Size<Option<S>>>,
}

impl<S: LayoutScalar> Traverse for FinalSizeSelectorTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        match node {
            1 => [2].iter().copied(),
            _ => [].iter().copied(),
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == 1)
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        2
    }
}

impl<S: LayoutScalar> Compute for FinalSizeSelectorTree<S> {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        LayoutInputOf::box_input(self.styles[&node].clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S> {
        assert_eq!(node, 2, "the focused flex tree exposes one child");
        if input.run_mode() == RunMode::PerformLayout {
            self.final_known = Some(input.known());
            let size = Size::new(
                input.known().width.unwrap_or(S::from_f64(75.0)),
                input.known().height.unwrap_or(S::from_f64(20.0)),
            );
            return Ok(ComputeOutputOf::from_sizes(size, size));
        }

        Ok(ComputeOutputOf::from_sizes(
            Size::new(S::from_f64(75.0), S::from_f64(20.0)),
            Size::new(S::from_f64(75.0), S::from_f64(20.0)),
        ))
    }
}

fn assert_logical_flex_final_size_selector_uses_vertical_row_main_axis<S: LayoutScalar>(
    writing_mode: WritingMode,
) {
    let mut tree = FinalSizeSelectorTree {
        styles: HashMap::from([
            (
                1,
                NodeInputOf::<S> {
                    display: Display::Flex,
                    writing_mode,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(200.0)),
                        PreferredSizeOf::px(S::from_f64(100.0)),
                    ),
                    flex_direction: FlexDirection::Row,
                    ..NodeInputOf::default()
                },
            ),
            (
                2,
                NodeInputOf::<S> {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::percent(S::from_f64(0.25)),
                        PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    min_size: Size::new(MinSizeOf::px(S::from_f64(75.0)), MinSizeOf::ZERO),
                    ..NodeInputOf::default()
                },
            ),
        ]),
        layouts: HashMap::new(),
        final_known: None,
    };

    compute_flex(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(200.0)), Some(S::from_f64(100.0))),
            crate::ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(200.0)),
                AvailableOf::definite(S::from_f64(100.0)),
            ),
        ),
    )
    .expect("vertical or sideways flex row layout succeeds");

    assert_eq!(
        tree.final_known
            .expect("final layout request is recorded")
            .width,
        Some(S::from_f64(50.0)),
        "the percentage-dependent physical width is refined after a vertical main-axis row"
    );
    assert_eq!(
        tree.layouts[&2].size,
        Size::new(S::from_f64(50.0), S::from_f64(20.0)),
        "the corrected final known width reaches child output"
    );
}

#[test]
fn logical_flex_placement_final_size_selector_maps_vertical_row_for_f32() {
    assert_logical_flex_final_size_selector_uses_vertical_row_main_axis::<f32>(
        WritingMode::VerticalLr,
    );
}

#[test]
fn logical_flex_placement_final_size_selector_maps_vertical_row_for_f64() {
    assert_logical_flex_final_size_selector_uses_vertical_row_main_axis::<f64>(
        WritingMode::VerticalLr,
    );
}

#[test]
fn logical_flex_placement_final_size_selector_maps_sideways_row_for_f32() {
    assert_logical_flex_final_size_selector_uses_vertical_row_main_axis::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn logical_flex_placement_final_size_selector_maps_sideways_row_for_f64() {
    assert_logical_flex_final_size_selector_uses_vertical_row_main_axis::<f64>(
        WritingMode::SidewaysLr,
    );
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
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(0.0),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            align_items: Some(AlignItems::Baseline),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
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
                    input.known().width.unwrap_or(20.0),
                    input.known().height.unwrap_or(10.0),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::AUTO),
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
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 40.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
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
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(50.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            aspect_ratio: AspectRatio::new(2.0),
            flex_grow: FlexGrowOf::try_new(0.0).unwrap(),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 50.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            min_size: Size::new(MinSize::AUTO, MinSize::px(40.0)),
            aspect_ratio: AspectRatio::new(2.0),
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

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(200.0, 100.0));
}

#[test]
fn flex_replaced_automatic_minimum_selects_smaller_suggestion_and_preserves_cross_stretch_in_both_scalar_lanes()
 {
    assert_flex_replaced_automatic_minimum_selects_smaller_suggestion::<f32>();
    assert_flex_replaced_automatic_minimum_selects_smaller_suggestion::<f64>();
}

fn assert_flex_replaced_automatic_minimum_selects_smaller_suggestion<S: LayoutScalar>() {
    #[derive(Default)]
    struct FlexTree<S: LayoutScalar> {
        styles: HashMap<u32, NodeInputOf<S>>,
        layouts: HashMap<u32, NodeOutputOf<S>>,
    }

    impl<S: LayoutScalar> Traverse for FlexTree<S> {
        type Node = u32;
        type Scalar = S;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            match node {
                1 => [2].iter().copied(),
                _ => [].iter().copied(),
            }
        }

        fn child_count(&self, node: Self::Node) -> usize {
            usize::from(node == 1)
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            2
        }
    }

    impl<S: LayoutScalar> Compute for FlexTree<S> {
        fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<S>) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInputOf<S>,
        ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S> {
            let size = Size::new(
                input.known().width.unwrap_or(S::from_f64(90.0)),
                input.known().height.unwrap_or(S::from_f64(10.0)),
            );
            Ok(ComputeOutputOf::from_sizes(size, size))
        }
    }

    let mut widths = Vec::new();
    let mut heights = Vec::new();
    for item_is_replaced in [true, false] {
        let mut tree = FlexTree::default();
        tree.styles.insert(
            1,
            NodeInputOf {
                display: Display::Flex,
                align_items: Some(AlignItems::Stretch),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(60.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInputOf {
                item_is_replaced,
                aspect_ratio: AspectRatioOf::new(S::from_f64(2.0)),
                flex_basis: FlexBasisOf::px(S::from_f64(90.0)),
                flex_grow: FlexGrowOf::try_new(S::ZERO).expect("zero is a valid flex grow"),
                flex_shrink: FlexShrinkOf::try_new(S::ONE).expect("one is a valid flex shrink"),
                ..NodeInputOf::default()
            },
        );

        compute_flex(
            &mut tree,
            1,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(60.0)), Some(S::from_f64(20.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                Size::new(
                    AvailableOf::definite(S::from_f64(60.0)),
                    AvailableOf::definite(S::from_f64(20.0)),
                ),
            ),
        )
        .expect("replaced automatic-minimum flex layout succeeds");

        let layout = tree.layouts[&2];
        widths.push(layout.size.width);
        heights.push(layout.size.height);
    }

    assert_eq!(widths, [S::from_f64(60.0), S::from_f64(90.0)]);
    assert_eq!(heights, [S::from_f64(20.0), S::from_f64(20.0)]);
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
            size: Size::new(PreferredSize::px(300.0), PreferredSize::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(100.0)),
            aspect_ratio: AspectRatio::new(2.0),
            flex_grow: FlexGrowOf::try_new(0.0).unwrap(),
            flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
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
            Size::new(Available::definite(500.0), Available::definite(100.0)),
        ),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
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
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
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
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
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
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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

    assert_eq!(output.size, Size::new(100.0, 60.0));
    assert_eq!(output.content_size, Size::new(100.0, 60.0));
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
                    input.known().width.unwrap_or(80.0),
                    input.known().height.unwrap_or(10.0),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
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
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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

    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 28.0));
    assert_eq!(tree.layouts[&3].size, Size::new(80.0, 28.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 32.0));
    assert_eq!(
        tree.inputs[&3].last().unwrap().known(),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
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
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(60.0)),
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
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(10.0)),
                flex_shrink: FlexShrinkOf::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        );
    }

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
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            max_size: Size::new(MaxSize::px(60.0), MaxSize::NONE),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            align_items: Some(AlignItems::FlexStart),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            flex_direction: FlexDirection::RowReverse,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
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
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
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
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(100.0)),
            flex_direction: FlexDirection::ColumnReverse,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
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
                    input.known().width.unwrap_or(10.0),
                    input.known().height.unwrap_or(10.0),
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
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(10.0)),
            flex_basis: FlexBasis::px(30.0),
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

    assert_eq!(tree.layouts[&2].size, Size::new(30.0, 10.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
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
            size: Size::new(PreferredSize::px(12.0), PreferredSize::px(12.0)),
            flex_basis: FlexBasis::px(0.0),
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
            size: Size::new(PreferredSize::px(12.0), PreferredSize::px(12.0)),
            flex_basis: FlexBasis::px(0.0),
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

    assert_eq!(output.size, Size::new(22.0, 14.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(22.0, 14.0));
    assert_eq!(tree.layouts[&3].location, Point::new(22.0, 0.0));
    assert_eq!(tree.layouts[&3].size, Size::new(0.0, 12.0));
    assert_eq!(
        tree.inputs[&2].last().unwrap().known(),
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
            flex_basis: FlexBasis::px(0.0),
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

    assert_eq!(output.size.width, 120.0);
    assert_eq!(output.content_size.width, 120.0);
    assert_eq!(tree.layouts[&2].content_size.width, 120.0);
}

use crate::{LengthPercentageOf, NodeInput, PreferredSize};

#[test]
fn flex_percent_dependent_affine_size_requests_definite_cross_rerun() {
    let height = LengthPercentageOf::from_coefficients(10.0, 0.50).expect("finite coefficients");
    let mut child = NodeInput::default();
    child.size.height = PreferredSize::value(height);

    assert!(child.size.height.depends_on_basis());
}

fn fri05_c04_flex_all_flow_axes() -> [FlowAxes; 10] {
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

fn fri05_c04_flex_overflow_at_flow_axes(
    flow_axes: FlowAxes,
    inline: Overflow,
    block: Overflow,
) -> ComputedOverflow {
    match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => computed_overflow(inline, block),
        PhysicalAxis::Vertical => computed_overflow(block, inline),
    }
}

fn fri05_c04_flex_input(size: Size<f32>, flow_axes: FlowAxes) -> ComputeInput {
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
    assert_eq!(in_flow.scrollbar_size, in_flow_geometry.scrollbar_size());
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
    assert_eq!(absolute.scrollbar_size, absolute_geometry.scrollbar_size());
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

fn fri05_c04_assert_flow_range(
    geometry: ScrollGeometry,
    flow_axes: FlowAxes,
    inline: (f32, f32),
    block: (f32, f32),
    context: &str,
) {
    let expected = FlowRelativeScrollRange::try_new(inline.0, inline.1, block.0, block.1)
        .expect("expected flow range is ordered");
    assert_eq!(
        geometry.physical_range(),
        flow_axes.physical_scroll_range(expected),
        "{context}"
    );
    assert_eq!(
        flow_axes.flow_relative_scroll_range(geometry.physical_range()),
        expected,
        "{context}"
    );
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

fn fri05_c04_flex_alignment_output(
    justify_content: Option<AlignContent>,
    align_content: Option<AlignContent>,
    wrap: FlexWrap,
    child_sizes: &[Size<f32>],
) -> (ComputeOutput, crate::test_support::layout_tree::OracleTree) {
    let size = Size::new(100.0, 80.0);
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let children = (1..=u32::try_from(child_sizes.len()).unwrap()).collect::<Vec<_>>();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, children.iter().copied())
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                size: size.map(PreferredSize::px),
                flex_wrap: wrap,
                align_content,
                align_items: Some(AlignItems::FlexStart),
                justify_content,
                ..NodeInput::default()
            },
        );
    for (child, child_size) in children.into_iter().zip(child_sizes.iter().copied()) {
        tree = tree.children(child, []).style(
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
        .expect("alignment-aware flex layout succeeds");
    (output, tree)
}

fn fri05_c04_assert_physical_range(output: ComputeOutput, expected: (f32, f32, f32, f32)) {
    let range = output.scroll_geometry.unwrap().physical_range();
    assert_eq!(
        (
            range.x().minimum(),
            range.x().maximum(),
            range.y().minimum(),
            range.y().maximum(),
        ),
        expected
    );
}

#[test]
fn fri05_c04_flex_alignment_justify_subjects_cover_start_end_center_space_none_and_safe_fallback() {
    for (alignment, expected) in [
        (Some(AlignContent::Start), (0.0, 40.0, 0.0, 0.0)),
        (Some(AlignContent::End), (-40.0, 0.0, 0.0, 0.0)),
        (Some(AlignContent::Center), (-20.0, 20.0, 0.0, 0.0)),
        (None, (0.0, 40.0, 0.0, 0.0)),
        (Some(AlignContent::SafeEnd), (0.0, 40.0, 0.0, 0.0)),
    ] {
        let (output, _) = fri05_c04_flex_alignment_output(
            alignment,
            None,
            FlexWrap::NoWrap,
            &[Size::new(140.0, 20.0)],
        );
        fri05_c04_assert_physical_range(output, expected);
    }

    let (distributed, tree) = fri05_c04_flex_alignment_output(
        Some(AlignContent::SpaceBetween),
        None,
        FlexWrap::NoWrap,
        &[Size::new(20.0, 20.0), Size::new(20.0, 20.0)],
    );
    fri05_c04_assert_physical_range(distributed, (0.0, 0.0, 0.0, 0.0));
    assert_eq!(tree.layout(1).unwrap().location.x, 0.0);
    assert_eq!(tree.layout(2).unwrap().location.x, 80.0);
}

#[test]
fn fri05_c04_flex_alignment_main_subject_includes_positive_margins_and_gaps_once() {
    let size = Size::new(100.0, 80.0);
    let layout = |justify_content, gap, children: &[(f32, Edges<LengthAuto>)]| {
        let child_ids = (1..=u32::try_from(children.len()).unwrap()).collect::<Vec<_>>();
        let mut tree = crate::test_support::layout_tree::OracleTree::new()
            .children(0, child_ids.iter().copied())
            .style(
                0,
                NodeInput {
                    display: Display::Flex,
                    overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                    size: size.map(PreferredSize::px),
                    gap: Size::new(Length::px(gap), Length::ZERO),
                    align_items: Some(AlignItems::FlexStart),
                    justify_content: Some(justify_content),
                    ..NodeInput::default()
                },
            );
        for (child, (width, margin)) in child_ids.into_iter().zip(children.iter().copied()) {
            tree = tree.children(child, []).style(
                child,
                NodeInput {
                    size: Size::new(PreferredSize::px(width), PreferredSize::px(20.0)),
                    min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                    flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                    margin,
                    ..NodeInput::default()
                },
            );
        }
        compute_flex(
            &mut tree,
            0,
            fri05_c04_flex_input(
                size,
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ),
        )
        .expect("margin-aware alignment layout succeeds")
    };

    let start_margin = layout(
        AlignContent::End,
        0.0,
        &[(
            120.0,
            Edges {
                left: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::ZERO)
            },
        )],
    );
    fri05_c04_assert_physical_range(start_margin, (-40.0, 0.0, 0.0, 0.0));

    let end_margin = layout(
        AlignContent::Start,
        0.0,
        &[(
            120.0,
            Edges {
                right: LengthAuto::px(20.0),
                ..Edges::all(LengthAuto::ZERO)
            },
        )],
    );
    fri05_c04_assert_physical_range(end_margin, (0.0, 40.0, 0.0, 0.0));

    let gap = layout(
        AlignContent::End,
        20.0,
        &[
            (
                40.0,
                Edges {
                    left: LengthAuto::px(10.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
            ),
            (
                40.0,
                Edges {
                    right: LengthAuto::px(10.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
            ),
        ],
    );
    fri05_c04_assert_physical_range(gap, (-20.0, 0.0, 0.0, 0.0));
}

#[test]
fn fri05_c04_flex_alignment_align_content_records_only_applicable_multiline_line_subject() {
    let (inapplicable, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::End),
        FlexWrap::NoWrap,
        &[Size::new(20.0, 120.0)],
    );
    fri05_c04_assert_physical_range(inapplicable, (0.0, 0.0, 0.0, 40.0));

    let (wrapped_single_line, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::End),
        FlexWrap::Wrap,
        &[Size::new(20.0, 20.0), Size::new(20.0, 20.0)],
    );
    fri05_c04_assert_physical_range(wrapped_single_line, (0.0, 0.0, 0.0, 0.0));

    let (empty_wrapped, _) =
        fri05_c04_flex_alignment_output(None, Some(AlignContent::End), FlexWrap::Wrap, &[]);
    fri05_c04_assert_physical_range(empty_wrapped, (0.0, 0.0, 0.0, 0.0));

    let (oversized_single_line, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::End),
        FlexWrap::Wrap,
        &[Size::new(20.0, 120.0)],
    );
    fri05_c04_assert_physical_range(oversized_single_line, (0.0, 0.0, 0.0, 0.0));

    let multiline_sizes = [Size::new(60.0, 60.0), Size::new(60.0, 60.0)];
    let (applicable, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::End),
        FlexWrap::Wrap,
        &multiline_sizes,
    );
    fri05_c04_assert_physical_range(applicable, (0.0, 0.0, -40.0, 0.0));

    let (safe, _) = fri05_c04_flex_alignment_output(
        None,
        Some(AlignContent::SafeEnd),
        FlexWrap::Wrap,
        &multiline_sizes,
    );
    fri05_c04_assert_physical_range(safe, (0.0, 0.0, 0.0, 40.0));
}

#[test]
fn fri05_c04_flex_alignment_main_subject_projects_all_flows_and_orientations() {
    for flow_axes in fri05_c04_flex_all_flow_axes() {
        for direction in [
            FlexDirection::Row,
            FlexDirection::RowReverse,
            FlexDirection::Column,
            FlexDirection::ColumnReverse,
        ] {
            let axes = FlexAxes::new(flow_axes, direction, FlexWrap::NoWrap);
            let size = axes.size_from_main_cross(100.0, 80.0);
            let child_size = axes.size_from_main_cross(140.0, 20.0);
            let mut tree = crate::test_support::layout_tree::OracleTree::new()
                .children(0, [1])
                .children(1, [])
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
                        align_items: Some(AlignItems::FlexStart),
                        justify_content: Some(AlignContent::Center),
                        ..NodeInput::default()
                    },
                )
                .style(
                    1,
                    NodeInput {
                        size: child_size.map(PreferredSize::px),
                        min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                        flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                        ..NodeInput::default()
                    },
                );
            let output = compute_flex(&mut tree, 0, fri05_c04_flex_input(size, flow_axes))
                .expect("mapped alignment flex layout succeeds");
            let main = (-20.0, 20.0);
            let (inline, block) = if direction.is_row() {
                (main, (0.0, 0.0))
            } else {
                ((0.0, 0.0), main)
            };
            fri05_c04_assert_flow_range(
                output.scroll_geometry.unwrap(),
                flow_axes,
                inline,
                block,
                &format!("{flow_axes:?} {direction:?}"),
            );
        }
    }
}

#[test]
fn fri05_c04_flex_alignment_subject_bounds_farther_absolute_and_nested_start_overflow() {
    let size = Size::new(100.0, 80.0);
    let absolute = |left| NodeInput {
        display: Display::Block,
        position: Position::Absolute,
        size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
        inset: Edges::new(
            LengthAuto::px(0.0),
            LengthAuto::AUTO,
            LengthAuto::AUTO,
            LengthAuto::px(left),
        ),
        ..NodeInput::default()
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 3, 4])
        .children(1, [2])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(
            0,
            NodeInput {
                display: Display::Flex,
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                size: size.map(PreferredSize::px),
                align_items: Some(AlignItems::FlexStart),
                justify_content: Some(AlignContent::Center),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: ComputedOverflow::VISIBLE,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(20.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                ..NodeInput::default()
            },
        )
        .style(2, absolute(-100.0))
        .style(3, absolute(-100.0))
        .style(4, absolute(160.0));

    let output = compute_flex(
        &mut tree,
        0,
        fri05_c04_flex_input(
            size,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("bounded alignment overflow layout succeeds");
    let geometry = output.scroll_geometry.unwrap();
    let overflow = geometry.scrollable_overflow();
    assert!(
        overflow.origin().x < -100.0,
        "nested start overflow is retained"
    );
    assert_eq!(overflow.origin().x + overflow.size().width, 170.0);
    fri05_c04_assert_physical_range(output, (-20.0, 70.0, 0.0, 0.0));
}
