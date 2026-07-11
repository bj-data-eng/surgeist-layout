use crate::*;

#[test]
fn leaf_layout_returns_known_size_without_calling_measure() {
    let input = ComputeInput {
        run_mode: RunMode::ComputeSize,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::new(Some(120.0), Some(48.0)),
        parent: Size::new(Some(500.0), Some(400.0)),
        available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    };

    let output = compute_leaf(input, &NodeInput::default(), |_known, _available| {
        panic!("known dimensions should not require measurement")
    });

    assert_eq!(output.size, Size::new(120.0, 48.0));
}

#[test]
fn leaf_layout_uses_measure_for_auto_dimensions() {
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::new(None, None),
        parent: Size::new(Some(500.0), Some(400.0)),
        available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
    };

    let output = compute_leaf(input, &NodeInput::default(), |known, available| {
        assert_eq!(known, Size::new(None, None));
        assert_eq!(available.width, Available::definite(300.0));
        Size::new(90.0, 18.0)
    });

    assert_eq!(output.size, Size::new(90.0, 18.0));
    assert_eq!(output.content_size, Size::new(90.0, 18.0));
}

#[test]
fn leaf_layout_adds_padding_and_border_to_measured_outer_size() {
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::new(None, None),
        parent: Size::new(Some(200.0), Some(100.0)),
        available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    };
    let node_input = NodeInput {
        padding: Edges::all(Length::px(3.0)),
        border: Edges::all(Length::px(2.0)),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &node_input, |_known, _available| {
        Size::new(40.0, 12.0)
    });

    assert_eq!(output.size, Size::new(50.0, 22.0));
    assert_eq!(output.content_size, Size::new(46.0, 18.0));
}

#[test]
fn leaf_layout_reserves_scrollbar_gutter_for_scroll_overflow() {
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(200.0), Some(100.0)),
        available: Size::new(Available::definite(100.0), Available::definite(50.0)),
    };
    let node_input = NodeInput {
        overflow: Point::new(Overflow::Visible, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(15.0).unwrap(),
        padding: Edges::all(Length::px(2.0)),
        border: Edges::all(Length::px(1.0)),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &node_input, |_known, available| {
        assert_eq!(available.width, Available::definite(79.0));
        assert_eq!(available.height, Available::definite(44.0));
        Size::new(40.0, 12.0)
    });

    assert_eq!(output.size, Size::new(61.0, 18.0));
    assert_eq!(output.content_size, Size::new(44.0, 16.0));
}

#[test]
fn leaf_layout_preserves_physical_end_scrollbar_gutter_for_rtl() {
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(200.0), Some(100.0)),
        available: Size::new(Available::definite(100.0), Available::definite(50.0)),
    };
    let node_input = NodeInput {
        direction: Direction::Rtl,
        overflow: Point::new(Overflow::Visible, Overflow::Scroll),
        scrollbar_width: crate::ScrollbarWidthOf::try_new(15.0).unwrap(),
        padding: Edges::all(Length::px(2.0)),
        border: Edges::all(Length::px(1.0)),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &node_input, |_known, available| {
        assert_eq!(available.width, Available::definite(79.0));
        assert_eq!(available.height, Available::definite(44.0));
        Size::new(40.0, 12.0)
    });

    assert_eq!(output.size, Size::new(61.0, 18.0));
    assert_eq!(output.content_size, Size::new(44.0, 16.0));
}

#[test]
fn leaf_uses_validated_aspect_ratio() {
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(120.0), Some(80.0)),
        available: Size::new(Available::definite(120.0), Available::MAX_CONTENT),
    };
    let style = NodeInput {
        size: Size::new(Dimension::px(60.0), Dimension::AUTO),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &style, |_known, _available| Size::new(10.0, 10.0));

    assert_eq!(output.size, Size::new(60.0, 30.0));
}

#[test]
fn f64_leaf_layout_preserves_fractional_precision() {
    let input = ComputeInputOf::<f64> {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(200.0), Some(100.0)),
        available: Size::new(AvailableOf::definite(123.125), AvailableOf::MAX_CONTENT),
    };
    let style = NodeInputOf::<f64> {
        padding: Edges::all(LengthOf::px(0.125)),
        border: Edges::all(LengthOf::px(0.0625)),
        ..NodeInputOf::<f64>::default()
    };

    let output = compute_leaf(input, &style, |known, available| {
        assert_eq!(known, Size::NONE);
        assert_eq!(available.width, AvailableOf::definite(122.75));
        Size::new(16_777_217.25_f64, 7.75)
    });

    assert_eq!(output.size, Size::new(16_777_217.625, 8.125));
    assert_eq!(output.content_size, Size::new(16_777_217.5, 8.0));
}
