use super::fixtures::{FlexTree, computed_overflow};
use super::*;

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

    let root = PublicLayoutTreeOf::new().style(
        0,
        NodeInput {
            display: Display::Flex,
            min_size: Size::new(MinSize::AUTO, MinSize::STRETCH),
            ..NodeInput::default()
        },
    );
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
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            padding: Edges::all(Length::px(5.0)),
            border: Edges::all(Length::px(1.0)),
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(30.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 20.0), Size::new(40.0, 20.0)),
    );
    tree.insert_measure(
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

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(6.0, 6.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(40.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(56.0, 6.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(30.0, 30.0)
    );

    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(40.0), Some(20.0)));
    assert_eq!(tree.inputs(3)[0].known(), Size::new(Some(30.0), Some(30.0)));
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
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
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
    assert_eq!(tree.inputs(2)[0].run_mode(), RunMode::ComputeSize);
}

#[test]
fn flex_row_auto_main_item_uses_content_sizing_for_base_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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

    let base_input = tree.inputs(2)[0];
    assert_eq!(base_input.sizing_mode(), SizingMode::ContentSize);
    assert_eq!(base_input.known().width, None);
    assert_eq!(base_input.known().height, Some(10.0));
    assert_eq!(base_input.available().width, Available::MAX_CONTENT);
    assert_eq!(base_input.available().height, Available::definite(10.0));
}

#[test]
fn flex_row_hidden_overflow_item_has_zero_automatic_minimum() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutput::from_outer_size(Size::new(0.0, 50.0)))
                .known(Size::new(Some(0.0), Some(50.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 50.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(40.0, 50.0)));

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

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(0.0, 50.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(40.0, 50.0)
    );
}

#[test]
fn flex_column_hidden_overflow_aspect_item_has_zero_automatic_minimum() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            flex_direction: FlexDirection::Column,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutput::from_outer_size(Size::new(100.0, 0.0)))
                .known(Size::new(Some(100.0), Some(0.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 50.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(20.0, 50.0)));

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

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(20.0, 50.0)
    );
}

#[test]
fn flex_column_cross_axis_hidden_overflow_aspect_item_has_zero_automatic_minimum() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            flex_direction: FlexDirection::Column,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
            ..NodeInput::default()
        },
    );
    tree = tree
        .measure_when(
            2,
            OracleMeasurementOf::new(ComputeOutput::from_outer_size(Size::new(100.0, 0.0)))
                .known(Size::new(Some(100.0), Some(0.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 50.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(20.0, 50.0)));

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

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(20.0, 50.0)
    );
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
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::None,
            size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree = tree.measure_when(
        3,
        OracleMeasurementOf::new(ComputeOutput::HIDDEN).run_mode(RunMode::PerformHiddenLayout),
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
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged"),
        NodeOutput::with_source_index(crate::SourceIndex::new(1))
    );
    assert_eq!(
        tree.inputs(3),
        vec![ComputeInput::hidden(crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr,),
            crate::ParentFormattingContext::Flex
        ))]
    );
}

#[test]
fn flex_row_stretch_transfers_cross_size_through_aspect_ratio() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(50.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 50.0)
    );
    assert_eq!(
        tree.inputs(2).last().unwrap().known(),
        Size::new(Some(100.0), Some(50.0))
    );
}

#[test]
fn flex_row_stretched_aspect_ratio_item_does_not_shrink_below_transferred_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(200.0, 100.0)
    );
}

#[test]
fn flex_replaced_automatic_minimum_selects_smaller_suggestion_and_preserves_cross_stretch_in_both_scalar_lanes()
 {
    assert_flex_replaced_automatic_minimum_selects_smaller_suggestion::<f32>();
    assert_flex_replaced_automatic_minimum_selects_smaller_suggestion::<f64>();
}

fn assert_flex_replaced_automatic_minimum_selects_smaller_suggestion<S: LayoutScalar>() {
    let mut widths = Vec::new();
    let mut heights = Vec::new();
    for item_is_replaced in [true, false] {
        let mut tree = FlexTree::default();
        tree.insert_children(1, [2]);
        tree.insert_children(2, []);
        tree.insert_style(
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
        tree.insert_style(
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
        let expected_width = if item_is_replaced {
            S::from_f64(60.0)
        } else {
            S::from_f64(90.0)
        };
        tree = tree
            .measure_when(
                2,
                OracleMeasurementOf::new(ComputeOutputOf::from_outer_size(Size::new(
                    expected_width,
                    S::from_f64(20.0),
                )))
                .known(Size::new(Some(expected_width), Some(S::from_f64(20.0)))),
            )
            .measure(
                2,
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(90.0), S::from_f64(10.0))),
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

        let layout = tree.layout(2).expect("child layout is staged");
        widths.push(layout.size.width);
        heights.push(layout.size.height);
    }

    assert_eq!(widths, [S::from_f64(60.0), S::from_f64(90.0)]);
    assert_eq!(heights, [S::from_f64(20.0), S::from_f64(20.0)]);
}

#[test]
fn flex_row_aspect_ratio_auto_min_respects_authored_width_cap() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(300.0), PreferredSize::px(100.0)),
            align_items: Some(AlignItems::Stretch),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(50.0, 100.0)
    );
}

#[test]
fn flex_row_uses_flex_basis_as_the_main_base_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(30.0, 10.0)
    );
    assert_eq!(
        tree.inputs(2).last().unwrap().known(),
        Size::new(Some(30.0), Some(10.0))
    );
}

#[test]
fn flex_row_flex_basis_zero_preserves_padding_border_without_authored_content_width() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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
    tree.insert_style(
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
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(22.0, 14.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(22.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(0.0, 12.0)
    );
    assert_eq!(
        tree.inputs(2).last().unwrap().known(),
        Size::new(Some(22.0), Some(14.0))
    );
}

#[test]
fn flex_row_flex_basis_padding_floor_preserves_leaf_content_intrinsic_size() {
    let mut tree = FlexTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            ..NodeInput::default()
        },
    );
    tree.insert_style(
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
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(0.0, 10.0), Size::new(120.0, 10.0)),
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
    assert_eq!(
        tree.layout(2)
            .expect("child layout is staged")
            .content_size
            .width,
        120.0
    );
}

use crate::{LengthPercentageOf, NodeInput, PreferredSize};

#[test]
fn flex_percent_dependent_affine_size_requests_definite_cross_rerun() {
    let height = LengthPercentageOf::from_coefficients(10.0, 0.50).expect("finite coefficients");
    let mut child = NodeInput::default();
    child.size.height = PreferredSize::value(height);

    assert!(child.size.height.depends_on_basis());
}
