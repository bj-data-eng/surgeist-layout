use crate::*;

fn assert_leaf_uses_containing_flow_for_percentage_edges<S: LayoutScalar>() {
    let input = ComputeInputOf::<S>::leaf_layout(
        Size::NONE,
        Size::new(Some(S::from_f64(5_000.0)), Some(S::from_f64(3_000.0))),
        crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        Size::new(Available::definite(100.0), Available::definite(50.0)),
    );
    let node_input = NodeInput {
        overflow: Point::new(Overflow::Visible, Overflow::Scroll),
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
    assert_eq!(output.content_size, Size::new(44.0, 16.0));
}

#[test]
fn leaf_measurement_available_size_floors_below_insets_at_zero() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(200.0), Some(100.0)),
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        Size::new(Available::definite(8.0), Available::definite(6.0)),
    );
    let node_input = NodeInput {
        overflow: Point::new(Overflow::Visible, Overflow::Scroll),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        Size::new(Available::definite(100.0), Available::definite(50.0)),
    );
    let node_input = NodeInput {
        direction: Direction::Rtl,
        overflow: Point::new(Overflow::Visible, Overflow::Scroll),
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
    assert_eq!(output.content_size, Size::new(44.0, 16.0));
}

#[test]
fn leaf_uses_validated_aspect_ratio() {
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(120.0), Some(80.0)),
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
        Size::new(Available::definite(120.0), Available::MAX_CONTENT),
    );
    let style = NodeInput {
        size: Size::new(Dimension::px(60.0), Dimension::AUTO),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
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
