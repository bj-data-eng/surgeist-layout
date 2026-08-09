use crate::test_support::scroll_geometry::{
    assert_scroll_padding_inputs_exact, scroll_padding_inputs,
};
use crate::*;

fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

fn fri06_mr02_scroll_padding_cases<S: LayoutScalar>() -> [(ScrollPaddingOf<S>, Edges<S>); 2] {
    let [first, second] = scroll_padding_inputs();

    [
        (
            first,
            Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO),
        ),
        (
            second,
            Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
        ),
    ]
}

fn assert_fri06_mr02_scroll_padding_leaf<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    for (scroll_padding, expected) in fri06_mr02_scroll_padding_cases() {
        let style = NodeInputOf::<S> {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(size.width),
                PreferredSizeOf::px(size.height),
            ),
            scroll_padding,
            ..NodeInputOf::default()
        };
        let input = ComputeInputOf::leaf_layout(
            size.map(Some),
            size.map(Some),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            size.map(AvailableOf::definite),
        )
        .expect("test leaf input is valid");
        let geometry = compute_leaf(input, &style, |_measurement| Ok::<_, ()>(size))
            .expect("leaf scroll-padding characterization succeeds")
            .scroll_geometry
            .expect("performed leaf layout emits geometry");

        assert_eq!(geometry.resolved_scroll_padding(), expected);
    }
}

#[test]
fn fri06_mr02_scroll_padding_leaf_preserves_auto_and_value_on_each_physical_edge() {
    assert_fri06_mr02_scroll_padding_leaf::<f32>();
    assert_fri06_mr02_scroll_padding_leaf::<f64>();
}

#[test]
fn fri08_c07_t05_scroll_fixture_leaf_rows_preserve_exact_auto_and_value_edges() {
    fn assert_rows<S: LayoutScalar>() {
        assert_scroll_padding_inputs_exact::<S>();
        assert_eq!(
            fri06_mr02_scroll_padding_cases::<S>().map(|(_, expected)| expected),
            [
                Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO,),
                Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
            ]
        );
    }

    assert_rows::<f32>();
    assert_rows::<f64>();
}

fn assert_measured_leaf_block_margin_collapse_uses_own_logical_block_extent<S: LayoutScalar>() {
    let containing_flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);

    for writing_mode in [
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        let leaf_flow = FlowAxes::new(writing_mode, Direction::Ltr);
        let style = NodeInputOf::<S> {
            display: Display::Block,
            writing_mode,
            ..NodeInputOf::default()
        };
        let input = ComputeInputOf::<S>::leaf_layout(
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                containing_flow,
                crate::ParentFormattingContext::BlockFlow,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        )
        .expect("valid direct leaf input");

        let zero_block = compute_leaf(input, &style, |_measurement| {
            Ok::<_, ()>(Size::new(S::ZERO, S::from_f64(12.0)))
        })
        .expect("leaf measurement succeeds");
        assert_eq!(
            zero_block.block_margin_collapse.at(leaf_flow.block_start()),
            CollapsibleMarginOf::ZERO
        );
        assert_eq!(
            zero_block.block_margin_collapse.at(leaf_flow.block_end()),
            CollapsibleMarginOf::ZERO
        );
        assert!(
            zero_block
                .block_margin_collapse
                .can_collapse_through(leaf_flow)
        );
        assert!(
            !zero_block
                .block_margin_collapse
                .can_collapse_through(containing_flow)
        );

        let nonzero_block = compute_leaf(input, &style, |_measurement| {
            Ok::<_, ()>(Size::new(S::from_f64(12.0), S::ZERO))
        })
        .expect("leaf measurement succeeds");
        assert!(
            !nonzero_block
                .block_margin_collapse
                .can_collapse_through(leaf_flow)
        );
    }
}

#[test]
fn parent_context_gates_measured_leaf_boundary_collapse_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let style = NodeInputOf::<S> {
            display: Display::Block,
            ..NodeInputOf::default()
        };

        for (parent_context, expected_collapse) in [
            (ParentFormattingContext::BlockFlow, true),
            (ParentFormattingContext::Flex, false),
            (ParentFormattingContext::Grid, false),
            (ParentFormattingContext::NoParent, false),
        ] {
            let input = ComputeInputOf::<S>::leaf_layout(
                Size::NONE,
                Size::NONE,
                ContainingLayoutContext::new(flow_axes, parent_context),
                Size::splat(AvailableOf::MAX_CONTENT),
            )
            .expect("valid direct leaf input");
            let output = compute_leaf(input, &style, |_measurement| Ok::<_, ()>(Size::ZERO))
                .expect("leaf measurement succeeds");

            assert_eq!(
                output.block_margin_collapse.can_collapse_through(flow_axes),
                expected_collapse,
                "unexpected measured-leaf boundary collapse for {parent_context:?}"
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn measured_leaf_block_margin_collapse_uses_own_logical_block_extent_in_f32() {
    assert_measured_leaf_block_margin_collapse_uses_own_logical_block_extent::<f32>();
}

#[test]
fn measured_leaf_block_margin_collapse_uses_own_logical_block_extent_in_f64() {
    assert_measured_leaf_block_margin_collapse_uses_own_logical_block_extent::<f64>();
}

fn assert_leaf_uses_containing_flow_for_percentage_edges<S: LayoutScalar>() {
    let input = ComputeInputOf::<S>::leaf_layout(
        Size::NONE,
        Size::new(Some(S::from_f64(5_000.0)), Some(S::from_f64(3_000.0))),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(
            AvailableOf::definite(S::from_f64(5_000.0)),
            AvailableOf::definite(S::from_f64(8_000.0)),
        ),
    )
    .expect("valid direct leaf input");
    let style = NodeInputOf::<S> {
        margin: Edges::all(LengthAutoOf::percent(S::from_f64(0.1))),
        padding: Edges::all(LengthOf::percent(S::from_f64(0.2))),
        border: Edges::all(LengthOf::percent(S::from_f64(0.3))),
        ..NodeInputOf::default()
    };

    compute_leaf(input, &style, |measurement| {
        assert_eq!(
            measurement.available_content_size(),
            Size::new(
                MeasurementAvailableOf::definite(S::from_f64(1_400.0)).unwrap(),
                MeasurementAvailableOf::definite(S::from_f64(4_400.0)).unwrap(),
            )
        );
        Ok::<_, ()>(Size::ZERO)
    })
    .expect("leaf percentage resolution succeeds");
}

#[test]
fn leaf_uses_containing_flow_for_percentage_edges_in_f32() {
    assert_leaf_uses_containing_flow_for_percentage_edges::<f32>();
}

#[test]
fn leaf_uses_containing_flow_for_percentage_edges_in_f64() {
    assert_leaf_uses_containing_flow_for_percentage_edges::<f64>();
}

#[test]
fn leaf_layout_returns_known_size_without_calling_measure() {
    let input = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(120.0), Some(48.0)),
        Size::new(Some(500.0), Some(400.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );

    let output = compute_leaf(input, &NodeInput::default(), |_input| -> Result<Size, ()> {
        panic!("known dimensions should not require measurement")
    })
    .unwrap();

    assert_eq!(output.size, Size::new(120.0, 48.0));
}

#[test]
fn leaf_layout_uses_measure_for_auto_dimensions() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(None, None),
        Size::new(Some(500.0), Some(400.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(300.0), Available::MAX_CONTENT),
    );

    let output = compute_leaf(input, &NodeInput::default(), |measure_input| {
        let known = measure_input.known_content_size();
        let available = measure_input.available_content_size();
        assert_eq!(known, Size::new(None, None));
        assert_eq!(
            available.width,
            MeasurementAvailable::definite(300.0).unwrap()
        );
        Ok::<_, ()>(Size::new(90.0, 18.0))
    })
    .unwrap();

    assert_eq!(output.size, Size::new(90.0, 18.0));
    assert_eq!(output.content_size, Size::new(90.0, 18.0));
}

#[test]
fn leaf_layout_adds_padding_and_border_to_measured_outer_size() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(None, None),
        Size::new(Some(200.0), Some(100.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    let node_input = NodeInput {
        padding: Edges::all(Length::px(3.0)),
        border: Edges::all(Length::px(2.0)),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &node_input, |_input| {
        Ok::<_, ()>(Size::new(40.0, 12.0))
    })
    .unwrap();

    assert_eq!(output.size, Size::new(50.0, 22.0));
    assert_eq!(output.content_size, Size::new(46.0, 18.0));
}

#[test]
fn leaf_layout_reserves_scrollbar_gutter_for_scroll_overflow() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(200.0), Some(100.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(100.0), Available::definite(50.0)),
    );
    let node_input = NodeInput {
        overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(15.0).unwrap(),
        padding: Edges::all(Length::px(2.0)),
        border: Edges::all(Length::px(1.0)),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &node_input, |measure_input| {
        let available = measure_input.available_content_size();
        assert_eq!(
            available.width,
            MeasurementAvailable::definite(79.0).unwrap()
        );
        assert_eq!(
            available.height,
            MeasurementAvailable::definite(44.0).unwrap()
        );
        Ok::<_, ()>(Size::new(40.0, 12.0))
    })
    .unwrap();

    assert_eq!(output.size, Size::new(61.0, 18.0));
    assert_eq!(output.content_size, Size::new(59.0, 16.0));
}

#[test]
fn leaf_measurement_available_size_floors_below_insets_at_zero() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(200.0), Some(100.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(8.0), Available::definite(6.0)),
    );
    let node_input = NodeInput {
        overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
        padding: Edges::all(Length::px(4.0)),
        border: Edges::all(Length::px(3.0)),
        ..NodeInput::default()
    };

    compute_leaf(input, &node_input, |measure_input| {
        let available = measure_input.available_content_size();
        assert_eq!(
            available.width,
            MeasurementAvailable::definite(0.0).unwrap()
        );
        assert_eq!(
            available.height,
            MeasurementAvailable::definite(0.0).unwrap()
        );
        Ok::<_, ()>(Size::ZERO)
    })
    .unwrap();
}

#[test]
fn leaf_measurement_known_size_is_content_space_and_floored() {
    let input = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(4.0), None),
        Size::new(Some(200.0), Some(100.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    let node_input = NodeInput {
        padding: Edges::all(Length::px(3.0)),
        border: Edges::all(Length::px(2.0)),
        ..NodeInput::default()
    };

    compute_leaf(input, &node_input, |measure_input| {
        assert_eq!(
            measure_input.known_content_size(),
            Size::new(Some(0.0), None)
        );
        Ok::<_, ()>(Size::new(8.0, 4.0))
    })
    .unwrap();
}

#[test]
fn leaf_measurement_provider_error_is_preserved() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );

    let error = compute_leaf(input, &NodeInput::default(), |_input| {
        Err::<Size, _>("provider failed")
    })
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::Measurement("provider failed")
    );
}

#[test]
fn leaf_measurement_rejects_negative_provider_width() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(200.0), Some(100.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );

    let error = compute_leaf(input, &NodeInput::default(), |_input| {
        Ok::<_, ()>(Size::new(-1.0, 10.0))
    })
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    let LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(error)) = error.kind()
    else {
        panic!("expected invalid measurement output");
    };
    assert_eq!(error.axis(), PhysicalAxis::Horizontal);
    assert_eq!(
        error.error(),
        NonNegativeFiniteScalarErrorOf::Negative { value: -1.0 }
    );
}

#[test]
fn leaf_measurement_rejects_nan_provider_height() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );

    let error = compute_leaf(input, &NodeInput::default(), |_input| {
        Ok::<_, ()>(Size::new(10.0, f32::NAN))
    })
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    let LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(error)) = error.kind()
    else {
        panic!("expected invalid measurement output");
    };
    assert_eq!(error.axis(), PhysicalAxis::Vertical);
    let NonNegativeFiniteScalarErrorOf::NonFinite { value } = error.error() else {
        panic!("expected non-finite scalar error");
    };
    assert!(value.is_nan());
}

#[test]
fn leaf_layout_preserves_physical_end_scrollbar_gutter_for_rtl() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(200.0), Some(100.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(100.0), Available::definite(50.0)),
    );
    let node_input = NodeInput {
        direction: Direction::Rtl,
        overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(15.0).unwrap(),
        padding: Edges::all(Length::px(2.0)),
        border: Edges::all(Length::px(1.0)),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &node_input, |measure_input| {
        let available = measure_input.available_content_size();
        assert_eq!(
            available.width,
            MeasurementAvailable::definite(79.0).unwrap()
        );
        assert_eq!(
            available.height,
            MeasurementAvailable::definite(44.0).unwrap()
        );
        Ok::<_, ()>(Size::new(40.0, 12.0))
    })
    .unwrap();

    assert_eq!(output.size, Size::new(61.0, 18.0));
    assert_eq!(output.content_size, Size::new(59.0, 16.0));
}

#[test]
fn leaf_uses_validated_aspect_ratio() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(120.0), Some(80.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(120.0), Available::MAX_CONTENT),
    );
    let style = NodeInput {
        size: Size::new(PreferredSize::px(60.0), PreferredSize::AUTO),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &style, |_input| Ok::<_, ()>(Size::new(10.0, 10.0))).unwrap();

    assert_eq!(output.size, Size::new(60.0, 30.0));
}

#[test]
fn f64_leaf_layout_preserves_fractional_precision() {
    let input = ComputeInputOf::<f64>::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(200.0), Some(100.0)),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::definite(123.125), AvailableOf::MAX_CONTENT),
    );
    let style = NodeInputOf::<f64> {
        padding: Edges::all(LengthOf::px(0.125)),
        border: Edges::all(LengthOf::px(0.0625)),
        ..NodeInputOf::<f64>::default()
    };

    let output = compute_leaf(input, &style, |measure_input| {
        let known = measure_input.known_content_size();
        let available = measure_input.available_content_size();
        assert_eq!(known, Size::NONE);
        assert_eq!(
            available.width,
            MeasurementAvailableOf::<f64>::definite(122.75).unwrap()
        );
        Ok::<_, ()>(Size::new(16_777_217.25_f64, 7.75))
    })
    .unwrap();

    assert_eq!(output.size, Size::new(16_777_217.625, 8.125));
    assert_eq!(output.content_size, Size::new(16_777_217.5, 8.0));
}

#[test]
fn f64_leaf_measurement_rejects_infinite_provider_height() {
    let input = ComputeInputOf::<f64>::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
    );

    let error = compute_leaf(input, &NodeInputOf::<f64>::default(), |_input| {
        Ok::<_, ()>(Size::new(10.0, f64::INFINITY))
    })
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    let LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(error)) =
        error.kind()
    else {
        panic!("expected invalid measurement output");
    };
    assert_eq!(error.axis(), PhysicalAxis::Vertical);
    assert_eq!(
        error.error(),
        NonNegativeFiniteScalarErrorOf::NonFinite {
            value: f64::INFINITY
        }
    );
}

#[test]
fn fri04_c03_leaf_root_leaf_consumes_nested_sizes_in_compute_and_layout_modes() {
    fn calculation(value: f32) -> SizingCalculation {
        SizingCalculation::value(LengthPercentageOf::px(value).expect("finite sizing value"))
    }

    let style = NodeInput {
        size: Size::new(
            PreferredSize::calculation(
                SizingCalculation::max(vec![calculation(60.0), calculation(45.0)])
                    .expect("nonempty maximum"),
            ),
            PreferredSize::calculation(SizingCalculation::clamp(
                Some(calculation(20.0)),
                calculation(40.0),
                Some(calculation(70.0)),
            )),
        ),
        min_size: Size::new(
            MinSize::calculation(
                SizingCalculation::min(vec![calculation(-8.0), calculation(-3.0)])
                    .expect("nonempty minimum"),
            ),
            MinSize::calculation(
                SizingCalculation::max(vec![calculation(10.0), calculation(15.0)])
                    .expect("nonempty maximum"),
            ),
        ),
        max_size: Size::new(
            MaxSize::calculation(SizingCalculation::clamp(
                None,
                calculation(55.0),
                Some(calculation(90.0)),
            )),
            MaxSize::calculation(
                SizingCalculation::max(vec![calculation(45.0), calculation(35.0)])
                    .expect("nonempty maximum"),
            ),
        ),
        ..NodeInput::default()
    };

    for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
        let input = ComputeInput::for_child(
            run_mode,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(80.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::definite(80.0)),
        );

        let output = compute_leaf(input, &style, |_input| Ok::<_, ()>(Size::new(1.0, 1.0)))
            .expect("leaf sizing calculations resolve");
        assert_eq!(output.size, Size::new(55.0, 40.0));
    }
}

#[test]
fn fri04_c03_leaf_root_leaf_preserves_missing_basis_by_run_mode() {
    let percentage = SizingCalculation::max(vec![
        SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite sizing value")),
        SizingCalculation::value(
            LengthPercentageOf::from_percent_fraction(0.5).expect("finite percentage"),
        ),
    ])
    .expect("nonempty maximum");
    let style = NodeInput {
        size: Size::new(
            PreferredSize::calculation(percentage),
            PreferredSize::px(20.0),
        ),
        ..NodeInput::default()
    };
    let context = ContainingLayoutContext::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ParentFormattingContext::NoParent,
    );

    let compute_size = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        context,
        Size::splat(Available::MAX_CONTENT),
    );
    let output = compute_leaf(compute_size, &style, |_input| {
        Ok::<_, ()>(Size::new(30.0, 5.0))
    })
    .expect("intrinsic computation retains its missing-basis fallback");
    assert_eq!(output.size, Size::new(30.0, 20.0));

    let layout = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        context,
        Size::splat(Available::MAX_CONTENT),
    );
    let error = compute_leaf(layout, &style, |_input| Ok::<_, ()>(Size::new(30.0, 5.0)))
        .expect_err("layout requires the missing percentage basis");
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::MissingContext(LayoutMissingContext::RequiredBasis)
    );
}

#[test]
fn fri04_c04_leaf_block_positioned_leaf_calc_size_any_and_intrinsic_availability() {
    let context = ContainingLayoutContext::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ParentFormattingContext::NoParent,
    );
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(200.0), Some(100.0)),
        context,
        Size::new(Available::definite(150.0), Available::definite(90.0)),
    );
    let style = NodeInput {
        size: Size::new(
            PreferredSize::MIN_CONTENT,
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::Any,
                CalcSizeCalculation::from_coefficients(40.0, 0.5, 0.0)
                    .expect("finite calc-size coefficients"),
            )
            .expect("Any calc-size without size is valid"),
        ),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &style, |measurement| {
        assert_eq!(
            measurement.available_content_size().width,
            MeasurementAvailable::MIN_CONTENT
        );
        Ok::<_, ()>(Size::new(32.0, 12.0))
    })
    .expect("supported leaf contextual sizing resolves");

    assert_eq!(output.size, Size::new(32.0, 90.0));
}

#[test]
fn fri04_c04_leaf_block_positioned_leaf_reports_exact_unsupported_payload() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::splat(Some(100.0)),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::splat(Available::definite(100.0)),
    );
    let style = NodeInput {
        min_size: Size::new(MinSize::AUTO, MinSize::STRETCH),
        ..NodeInput::default()
    };

    let error = compute_leaf(input, &style, |_measurement| Ok::<_, ()>(Size::ZERO))
        .expect_err("later-owned minimum stretch must be rejected");

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected exact sizing capability, got {:?}", error.kind());
    };
    assert_eq!(unsupported.property(), SizingProperty::Minimum);
    assert_eq!(unsupported.behavior(), SizingBehavior::Stretch);
    assert_eq!(unsupported.algorithm(), SizingAlgorithm::Leaf);
    assert_eq!(unsupported.axis(), PhysicalAxis::Vertical);
}

fn fri04_c04_leaf_block_positioned_assert_leaf_unsupported(
    style: NodeInput,
    property: SizingProperty,
    behavior: SizingBehavior,
    axis: PhysicalAxis,
) {
    let error = compute_leaf(
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(100.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::definite(100.0)),
        ),
        &style,
        |_measurement| -> Result<Size, ()> { panic!("unsupported sizing must precede measure") },
    )
    .expect_err("later-owned leaf sizing must be rejected");
    assert_eq!(error.site(), LayoutErrorSite::Standalone);
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
        (property, behavior, SizingAlgorithm::Leaf, axis)
    );
}

#[test]
fn fri04_c04_leaf_block_positioned_leaf_front_door_covers_all_unsupported_states() {
    let calculation = || {
        SizingCalculation::value(
            LengthPercentageOf::px(10.0).expect("finite fit-content calculation"),
        )
    };
    for (value, behavior) in [
        (PreferredSize::STRETCH, SizingBehavior::Stretch),
        (PreferredSize::FIT_CONTENT, SizingBehavior::FitContent),
        (PreferredSize::CONTAIN, SizingBehavior::Contain),
        (
            PreferredSize::fit_content_function(calculation()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_leaf_block_positioned_assert_leaf_unsupported(
            NodeInput {
                size: Size::new(value, PreferredSize::AUTO),
                ..NodeInput::default()
            },
            SizingProperty::Preferred,
            behavior,
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
            MinSize::fit_content_function(calculation()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_leaf_block_positioned_assert_leaf_unsupported(
            NodeInput {
                min_size: Size::new(MinSize::AUTO, value),
                ..NodeInput::default()
            },
            SizingProperty::Minimum,
            behavior,
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
            MaxSize::fit_content_function(calculation()),
            SizingBehavior::FitContentFunction,
        ),
    ] {
        fri04_c04_leaf_block_positioned_assert_leaf_unsupported(
            NodeInput {
                max_size: Size::new(value, MaxSize::NONE),
                ..NodeInput::default()
            },
            SizingProperty::Maximum,
            behavior,
            PhysicalAxis::Horizontal,
        );
    }

    let calc = || CalcSizeCalculation::value(LengthPercentageOf::ZERO);
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
        fri04_c04_leaf_block_positioned_assert_leaf_unsupported(
            NodeInput {
                size: Size::new(
                    PreferredSize::AUTO,
                    PreferredSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                ..NodeInput::default()
            },
            SizingProperty::Preferred,
            SizingBehavior::CalcSize(expected),
            PhysicalAxis::Vertical,
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
        fri04_c04_leaf_block_positioned_assert_leaf_unsupported(
            NodeInput {
                min_size: Size::new(
                    MinSize::calc_size(basis, calc()).expect("valid calc-size"),
                    MinSize::AUTO,
                ),
                ..NodeInput::default()
            },
            SizingProperty::Minimum,
            SizingBehavior::CalcSize(expected),
            PhysicalAxis::Horizontal,
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
        fri04_c04_leaf_block_positioned_assert_leaf_unsupported(
            NodeInput {
                max_size: Size::new(
                    MaxSize::NONE,
                    MaxSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                ..NodeInput::default()
            },
            SizingProperty::Maximum,
            SizingBehavior::CalcSize(expected),
            PhysicalAxis::Vertical,
        );
    }
}

#[test]
fn fri04_c04_leaf_block_positioned_missing_full_percentage_preserves_property_fallbacks() {
    let full = || {
        CalcSizeCalculation::from_coefficients(10.0, 0.5, 0.5)
            .expect("finite FullPercentage calculation")
    };
    let style = NodeInput {
        size: Size::new(
            PreferredSize::calc_size(PreferredSizeCalcBasis::FullPercentage, full())
                .expect("valid preferred calc-size"),
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::Any,
                CalcSizeCalculation::from_coefficients(25.0, 0.5, 0.0)
                    .expect("finite Any calculation"),
            )
            .expect("valid Any calc-size"),
        ),
        min_size: Size::new(
            MinSize::calc_size(MinSizeCalcBasis::FullPercentage, full())
                .expect("valid minimum calc-size"),
            MinSize::calc_size(MinSizeCalcBasis::FullPercentage, full())
                .expect("valid minimum calc-size"),
        ),
        max_size: Size::new(
            MaxSize::calc_size(MaxSizeCalcBasis::FullPercentage, full())
                .expect("valid maximum calc-size"),
            MaxSize::calc_size(MaxSizeCalcBasis::FullPercentage, full())
                .expect("valid maximum calc-size"),
        ),
        ..NodeInput::default()
    };
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::splat(Available::MAX_CONTENT),
    );

    let output = compute_leaf(input, &style, |_measurement| {
        Ok::<_, ()>(Size::new(30.0, 12.0))
    })
    .expect("missing FullPercentage keeps preferred/minimum auto and maximum none");

    assert_eq!(output.size, Size::new(30.0, 25.0));
}

#[test]
fn fri04_c04_leaf_block_positioned_calc_size_invalid_numeric_maps_exactly() {
    let style = NodeInput {
        size: Size::new(
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::Any,
                CalcSizeCalculation::from_coefficients(f32::MAX, f32::MAX, 0.0)
                    .expect("finite calc-size coefficients"),
            )
            .expect("valid Any calc-size"),
            PreferredSize::AUTO,
        ),
        ..NodeInput::default()
    };
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::splat(Some(100.0)),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::splat(Available::definite(100.0)),
    );
    let error = compute_leaf(input, &style, |_measurement| Ok::<_, ()>(Size::ZERO))
        .expect_err("overflowing calc-size must fail");
    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { value })
            if *value == f32::INFINITY
    ));
}

fn fri05_c03_leaf_layout_input(size: Size<f32>) -> ComputeInput {
    ComputeInput::leaf_layout(
        Size::NONE,
        size.map(Some),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        size.map(Available::definite),
    )
    .expect("FRI-05 leaf input is valid")
}

fn fri05_c03_leaf_measurement_size(input: LeafMeasureInput) -> Size<f32> {
    assert_eq!(input.known_content_size(), Size::NONE);
    input.available_content_size().map(|available| {
        available
            .definite_value()
            .expect("FRI-05 test availability is definite")
            .get()
    })
}

fn fri05_c03_leaf_gutter_at(
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

fn fri05_c03_leaf_all_flow_axes() -> [FlowAxes; 10] {
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

#[test]
fn fri05_c03_leaf_geometry_direct_emits_flow_clip_and_target_geometry() {
    for flow_axes in fri05_c03_leaf_all_flow_axes() {
        let overflow = match flow_axes.block_axis() {
            PhysicalAxis::Horizontal => computed_overflow(Overflow::Scroll, Overflow::Hidden),
            PhysicalAxis::Vertical => computed_overflow(Overflow::Hidden, Overflow::Scroll),
        };
        let style = NodeInput {
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_width: ScrollbarWidth::try_new(7.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        };
        let expected_content_size = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(93.0, 80.0),
            PhysicalAxis::Vertical => Size::new(100.0, 73.0),
        };
        let mut measured_inputs = Vec::new();
        let output = compute_leaf(
            fri05_c03_leaf_layout_input(Size::new(100.0, 80.0)),
            &style,
            |input| {
                measured_inputs.push(fri05_c03_leaf_measurement_size(input));
                Ok::<_, ()>(Size::new(20.0, 10.0))
            },
        )
        .expect("forced-scroll leaf layout succeeds");

        assert_eq!(
            measured_inputs,
            vec![expected_content_size],
            "{flow_axes:?}"
        );
        let geometry = output
            .scroll_geometry
            .expect("performed leaf layout emits canonical geometry");
        assert_eq!(geometry.flow_axes(), flow_axes);
        assert_eq!(geometry.content_box().size(), expected_content_size);
        assert_eq!(output.content_size, Size::new(100.0, 80.0), "{flow_axes:?}");
        assert_eq!(
            output.content_size,
            geometry.scrollable_overflow().size(),
            "{flow_axes:?}"
        );
        assert!(
            fri05_c03_leaf_gutter_at(geometry.gutters(), flow_axes.inline_end()).is_some(),
            "missing inline-end gutter for {flow_axes:?}"
        );
        for side in [
            PhysicalSide::Top,
            PhysicalSide::Right,
            PhysicalSide::Bottom,
            PhysicalSide::Left,
        ] {
            if side != flow_axes.inline_end() {
                assert_eq!(fri05_c03_leaf_gutter_at(geometry.gutters(), side), None);
            }
        }

        let zero_thickness = NodeInput {
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::ZERO,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        };
        let zero_geometry = compute_leaf(
            fri05_c03_leaf_layout_input(Size::new(100.0, 80.0)),
            &zero_thickness,
            |_input| Ok::<_, ()>(Size::new(120.0, 100.0)),
        )
        .expect("zero-thickness flow mapping succeeds")
        .scroll_geometry
        .expect("zero-thickness leaf still emits geometry");
        let x_start = if flow_axes.inline_axis() == PhysicalAxis::Horizontal {
            flow_axes.inline_start()
        } else {
            flow_axes.block_start()
        };
        let y_start = if flow_axes.inline_axis() == PhysicalAxis::Vertical {
            flow_axes.inline_start()
        } else {
            flow_axes.block_start()
        };
        let expected_x = match x_start {
            PhysicalSide::Left => (0.0, 20.0),
            PhysicalSide::Right => (-20.0, 0.0),
            PhysicalSide::Top | PhysicalSide::Bottom => unreachable!(),
        };
        let expected_y = match y_start {
            PhysicalSide::Top => (0.0, 20.0),
            PhysicalSide::Bottom => (-20.0, 0.0),
            PhysicalSide::Right | PhysicalSide::Left => unreachable!(),
        };
        let range = zero_geometry.physical_range();
        assert_eq!((range.x().minimum(), range.x().maximum()), expected_x);
        assert_eq!((range.y().minimum(), range.y().maximum()), expected_y);
        assert_eq!(zero_geometry.scrollbar_size(), Size::ZERO);
    }

    let visible = compute_leaf(
        fri05_c03_leaf_layout_input(Size::new(40.0, 30.0)),
        &NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(30.0)),
            ..NodeInput::default()
        },
        |_input| Ok::<_, ()>(Size::new(60.0, 50.0)),
    )
    .expect("visible-overflow leaf succeeds")
    .scroll_geometry
    .expect("visible-overflow leaf emits geometry");
    assert_eq!(visible.overflow_clip().x(), None);
    assert_eq!(visible.overflow_clip().y(), None);
    assert_eq!(
        visible.physical_range(),
        PhysicalScrollRange::try_new(0.0, 0.0, 0.0, 0.0).unwrap()
    );

    let scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let scroll_padding = ScrollPadding::new(
        ScrollPaddingValue::value(LengthPercentageOf::px(2.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(4.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(3.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(1.0).unwrap()),
    );
    let style = NodeInput {
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Rtl,
        overflow: computed_overflow(Overflow::Visible, Overflow::Clip),
        overflow_clip_margin: OverflowClipMargin::try_new(OverflowClipBox::BorderBox, 3.0).unwrap(),
        size: Size::new(PreferredSize::px(40.0), PreferredSize::px(30.0)),
        scroll_padding,
        scroll_margin,
        scroll_snap_type: ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Both,
            strictness: ScrollSnapStrictness::Mandatory,
        },
        scroll_snap_align: snap_align,
        scroll_snap_stop: ScrollSnapStop::Always,
        ..NodeInput::default()
    };
    let output = compute_leaf(
        fri05_c03_leaf_layout_input(Size::new(40.0, 30.0)),
        &style,
        |_input| Ok::<_, ()>(Size::new(60.0, 50.0)),
    )
    .expect("partial-clip leaf layout succeeds");
    let geometry = output.scroll_geometry.expect("leaf geometry is present");
    assert_eq!(geometry.overflow_clip().x(), None);
    let y_clip = geometry
        .overflow_clip()
        .y()
        .expect("only the clipped y axis has a clip interval");
    assert_eq!((y_clip.minimum(), y_clip.maximum()), (-3.0, 33.0));
    assert_eq!(geometry.used_overflow_x(), Overflow::Visible);
    assert_eq!(geometry.used_overflow_y(), Overflow::Clip);
    assert_eq!(
        geometry.resolved_scroll_padding(),
        Edges::new(2.0, 4.0, 3.0, 1.0)
    );
    assert_eq!(
        geometry.optimal_viewing_region(),
        ScrollRect::try_new(Point::new(1.0, 2.0), Size::new(35.0, 25.0)).unwrap()
    );
    assert_eq!(
        geometry.scroll_snap_type(),
        ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Both,
            strictness: ScrollSnapStrictness::Mandatory,
        }
    );
    let target = geometry.target();
    assert_eq!(
        target.border_box(),
        ScrollRect::try_new(Point::ZERO, Size::new(40.0, 30.0)).unwrap()
    );
    assert_eq!(target.scroll_margin(), scroll_margin);
    assert_eq!(
        target.flow_axes(),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl)
    );
    assert_eq!(target.snap_align(), snap_align);
    assert_eq!(target.snap_stop(), ScrollSnapStop::Always);

    let reverse_partial = compute_leaf(
        fri05_c03_leaf_layout_input(Size::new(40.0, 30.0)),
        &NodeInput {
            overflow: computed_overflow(Overflow::Clip, Overflow::Visible),
            overflow_clip_margin: OverflowClipMargin::try_new(OverflowClipBox::BorderBox, 2.0)
                .unwrap(),
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(30.0)),
            ..NodeInput::default()
        },
        |_input| Ok::<_, ()>(Size::new(60.0, 50.0)),
    )
    .expect("reverse partial clip succeeds")
    .scroll_geometry
    .expect("reverse partial clip emits geometry");
    let x_clip = reverse_partial
        .overflow_clip()
        .x()
        .expect("only x is clipped");
    assert_eq!((x_clip.minimum(), x_clip.maximum()), (-2.0, 42.0));
    assert_eq!(reverse_partial.overflow_clip().y(), None);
}

fn fri05_c03_leaf_auto_case(
    style: NodeInput,
    measured: Size<f32>,
    expected_inputs: &[Size<f32>],
    expected_content_box: Size<f32>,
    expected_scrollbar_size: Size<f32>,
) -> ComputeOutput {
    let mut measured_inputs = Vec::new();
    let output = compute_leaf(
        fri05_c03_leaf_layout_input(Size::new(100.0, 100.0)),
        &style,
        |input| {
            measured_inputs.push(fri05_c03_leaf_measurement_size(input));
            Ok::<_, ()>(measured)
        },
    )
    .expect("auto-gutter leaf layout succeeds");

    assert_eq!(measured_inputs, expected_inputs);
    let geometry = output
        .scroll_geometry
        .expect("performed leaf layout emits stable geometry");
    assert_eq!(geometry.content_box().size(), expected_content_box);
    assert_eq!(geometry.scrollbar_size(), expected_scrollbar_size);
    output
}

#[test]
fn fri05_c03_leaf_auto_direct_runs_only_monotone_geometry_changing_passes() {
    let automatic = NodeInput {
        overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
        scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        ..NodeInput::default()
    };

    fri05_c03_leaf_auto_case(
        automatic.clone(),
        Size::new(120.0, 100.0),
        &[
            Size::new(100.0, 100.0),
            Size::new(100.0, 85.0),
            Size::new(85.0, 85.0),
        ],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_leaf_auto_case(
        automatic.clone(),
        Size::new(100.0, 120.0),
        &[
            Size::new(100.0, 100.0),
            Size::new(85.0, 100.0),
            Size::new(85.0, 85.0),
        ],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_leaf_auto_case(
        automatic.clone(),
        Size::new(80.0, 80.0),
        &[Size::new(100.0, 100.0)],
        Size::new(100.0, 100.0),
        Size::ZERO,
    );
    fri05_c03_leaf_auto_case(
        automatic.clone(),
        Size::new(120.0, 80.0),
        &[Size::new(100.0, 100.0), Size::new(100.0, 85.0)],
        Size::new(100.0, 85.0),
        Size::new(0.0, 15.0),
    );
    fri05_c03_leaf_auto_case(
        automatic,
        Size::new(80.0, 120.0),
        &[Size::new(100.0, 100.0), Size::new(85.0, 100.0)],
        Size::new(85.0, 100.0),
        Size::new(15.0, 0.0),
    );

    fri05_c03_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(80.0, 80.0),
        &[Size::new(85.0, 85.0)],
        Size::new(85.0, 85.0),
        Size::new(15.0, 15.0),
    );
    fri05_c03_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::Stable,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(80.0, 80.0),
        &[Size::new(85.0, 100.0)],
        Size::new(85.0, 100.0),
        Size::new(15.0, 0.0),
    );
    fri05_c03_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(60.0, 80.0),
        &[Size::new(70.0, 100.0)],
        Size::new(70.0, 100.0),
        Size::new(30.0, 0.0),
    );
    fri05_c03_leaf_auto_case(
        NodeInput {
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::ZERO,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
        Size::new(120.0, 80.0),
        &[Size::new(100.0, 100.0)],
        Size::new(100.0, 100.0),
        Size::ZERO,
    );
}

#[test]
fn fri05_c03_leaf_auto_compute_size_keeps_zero_call_fast_path_and_no_geometry() {
    let context = ContainingLayoutContext::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ParentFormattingContext::NoParent,
    );
    let fully_known = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::splat(Some(100.0)),
        Size::splat(Some(100.0)),
        context,
        Size::splat(Available::definite(100.0)),
    );
    let style = NodeInput {
        display: Display::Block,
        overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
        scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        ..NodeInput::default()
    };
    let mut calls = 0;
    let known_output = compute_leaf(fully_known, &style, |_input| {
        calls += 1;
        Ok::<_, ()>(Size::new(120.0, 100.0))
    })
    .expect("fully known ComputeSize succeeds");
    assert_eq!(calls, 0);
    assert_eq!(known_output.scroll_geometry, None);

    let measured = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::splat(Some(100.0)),
        context,
        Size::splat(Available::definite(100.0)),
    );
    let measured_style = NodeInput {
        display: Display::Block,
        overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
        scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
        max_size: Size::new(MaxSize::px(100.0), MaxSize::px(100.0)),
        ..NodeInput::default()
    };
    let mut measured_inputs = Vec::new();
    let measured_output = compute_leaf(measured, &measured_style, |input| {
        measured_inputs.push(fri05_c03_leaf_measurement_size(input));
        Ok::<_, ()>(Size::new(120.0, 100.0))
    })
    .expect("measured ComputeSize succeeds");
    assert_eq!(
        measured_inputs,
        [
            Size::new(100.0, 100.0),
            Size::new(100.0, 85.0),
            Size::new(85.0, 85.0),
        ]
    );
    assert_eq!(measured_output.scroll_geometry, None);
}

#[test]
fn fri05_c03_integration_padding_seed_direct_measured_leaf_retains_gutter_area_in_both_scalar_lanes()
 {
    fn assert_lane<S: LayoutScalar>() {
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

        fn overflow_at_flow_axes(
            flow_axes: FlowAxes,
            inline: Overflow,
            block: Overflow,
        ) -> ComputedOverflow {
            match flow_axes.inline_axis() {
                PhysicalAxis::Horizontal => computed_overflow(inline, block),
                PhysicalAxis::Vertical => computed_overflow(block, inline),
            }
        }

        fn expected_axis_range<S: LayoutScalar>(
            geometry: ScrollGeometryOf<S>,
            origin_end: PhysicalSide,
        ) -> (S, S) {
            let Some(gutter) = gutter_at(geometry.gutters(), origin_end) else {
                return (S::ZERO, S::ZERO);
            };
            let thickness = match origin_end.axis() {
                PhysicalAxis::Horizontal => gutter.size().width,
                PhysicalAxis::Vertical => gutter.size().height,
            };
            match origin_end {
                PhysicalSide::Top | PhysicalSide::Left => (S::ZERO - thickness, S::ZERO),
                PhysicalSide::Right | PhysicalSide::Bottom => (S::ZERO, thickness),
            }
        }

        let scalar = S::from_f64;
        let size = Size::new(scalar(100.0), scalar(80.0));
        for flow_axes in fri05_c03_leaf_all_flow_axes() {
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
                    writing_mode: flow_axes.writing_mode(),
                    direction: flow_axes.direction(),
                    overflow: overflow_at_flow_axes(flow_axes, inline, block),
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
                let input = ComputeInputOf::<S>::leaf_layout(
                    Size::NONE,
                    size.map(Some),
                    ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                    size.map(AvailableOf::definite),
                )
                .expect("FRI-05 direct measured-leaf input is valid");
                let output = compute_leaf(input, &style, |_measurement| {
                    Ok::<_, ()>(Size::new(scalar(2.0), scalar(3.0)))
                })
                .expect("guttered direct measured leaf lays out");
                let geometry = output
                    .scroll_geometry
                    .expect("performed direct measured leaf emits geometry");

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

                let x_end = if flow_axes.inline_axis() == PhysicalAxis::Horizontal {
                    flow_axes.inline_end()
                } else {
                    flow_axes.block_end()
                };
                let y_end = if flow_axes.inline_axis() == PhysicalAxis::Vertical {
                    flow_axes.inline_end()
                } else {
                    flow_axes.block_end()
                };
                let range = geometry.physical_range();
                assert_eq!(
                    (range.x().minimum(), range.x().maximum()),
                    expected_axis_range(geometry, x_end),
                    "x range must derive from the retained padding seed for {case}/{flow_axes:?}"
                );
                assert_eq!(
                    (range.y().minimum(), range.y().maximum()),
                    expected_axis_range(geometry, y_end),
                    "y range must derive from the retained padding seed for {case}/{flow_axes:?}"
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
