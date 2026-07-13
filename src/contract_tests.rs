use crate::DefaultScalar;
use crate::*;

#[test]
fn default_scalar_remains_single_precision() {
    assert_eq!(
        std::mem::size_of::<DefaultScalar>(),
        std::mem::size_of::<f32>()
    );
    assert_eq!(std::mem::size_of::<Scalar>(), std::mem::size_of::<f32>());
}

#[test]
fn layout_scalar_supports_f32_and_f64() {
    fn assert_scalar<S: crate::LayoutScalar>() {
        assert!(S::ONE.is_finite());
        assert_eq!(S::ZERO + S::ONE, S::ONE);
        assert_eq!(S::from_usize(3), S::ONE + S::ONE + S::ONE);
        assert_eq!(S::from_f64(-2.5).abs(), S::from_f64(2.5));
        assert_eq!(S::from_f64(4.75).floor_to_usize_saturating(), 4);
        assert_eq!(S::NAN.floor_to_usize_saturating(), 0);
        assert_eq!(S::from_f64(-1.0).floor_to_usize_saturating(), 0);
        assert_eq!(S::INFINITY.floor_to_usize_saturating(), usize::MAX);
        assert_eq!(
            S::from_f64(usize::MAX as f64 * 2.0).floor_to_usize_saturating(),
            usize::MAX
        );
    }

    assert_scalar::<f32>();
    assert_scalar::<f64>();
}

#[test]
fn value_types_support_f64_scalar_lane() {
    let length = crate::LengthOf::<f64>::percent(0.25);
    let length = length.resolve(400.0);
    assert_eq!(length.value, Some(100.0));
    assert_eq!(length.status(), crate::LengthResolutionStatus::Resolved);

    let dimension = crate::DimensionOf::<f64>::px(42.5);
    let dimension = dimension.resolve(1000.0);
    assert_eq!(dimension.value, Some(42.5));
    assert_eq!(dimension.status(), crate::LengthResolutionStatus::Resolved);

    let ratio = crate::AspectRatioOf::<f64>::new(16.0 / 9.0)
        .expect("positive finite f64 aspect ratio should be accepted");
    assert_eq!(ratio.get(), 16.0 / 9.0);

    assert!(crate::AspectRatioOf::<f64>::new(f64::INFINITY).is_none());
}

#[test]
fn node_input_and_output_support_f64_scalar_lane() {
    let input = crate::NodeInputOf::<f64> {
        size: crate::Size::new(
            crate::DimensionOf::px(123.5),
            crate::DimensionOf::percent(0.25),
        ),
        margin: crate::Edges::all(crate::LengthAutoOf::px(2.5)),
        flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
        ..crate::NodeInputOf::<f64>::default()
    };

    let width = input.size.width.resolve(1000.0);
    let height = input.size.height.resolve(400.0);
    assert_eq!(width.value, Some(123.5));
    assert_eq!(width.status(), LengthResolutionStatus::Resolved);
    assert_eq!(height.value, Some(100.0));
    assert_eq!(height.status(), LengthResolutionStatus::Resolved);

    let precision_sentinel = 16_777_217.0_f64;
    let output = crate::NodeOutputOf::<f64> {
        size: crate::Size::new(precision_sentinel, 10.0),
        ..crate::NodeOutputOf::<f64>::default()
    };
    let compute_output =
        crate::ComputeOutputOf::<f64>::from_outer_size(crate::Size::new(precision_sentinel, 4.0));

    assert_eq!(output.size.width, precision_sentinel);
    assert_eq!(compute_output.size.width, precision_sentinel);
}

#[test]
fn compute_output_defaults_to_no_scroll_geometry() {
    let output = ComputeOutput::from_outer_size(Size::new(10.0, 20.0));

    assert_eq!(output.scroll_geometry, None);
}

#[test]
fn node_output_defaults_to_no_scroll_geometry() {
    let output = NodeOutput::with_order(7);

    assert_eq!(output.scroll_geometry, None);
}

#[test]
fn f32_default_keeps_representative_layout_types_smaller_than_f64_lane() {
    assert!(
        std::mem::size_of::<crate::ComputeOutput>()
            < std::mem::size_of::<crate::ComputeOutputOf<f64>>()
    );
    assert!(
        std::mem::size_of::<crate::NodeOutput>() < std::mem::size_of::<crate::NodeOutputOf<f64>>()
    );
    assert!(
        std::mem::size_of::<crate::CollapsibleMargin>()
            < std::mem::size_of::<crate::CollapsibleMarginOf<f64>>()
    );
    assert!(std::mem::size_of::<crate::Cache>() < std::mem::size_of::<crate::CacheOf<f64>>());
}

#[test]
fn f64_affine_resolution_preserves_large_coordinate_precision() {
    let value = crate::LengthPercentageOf::<f64>::from_coefficients(16_777_217.0, 0.5)
        .expect("finite coefficients");

    let resolution = crate::LengthOf::value(value).resolve_with_status(Some(21.0));
    assert_eq!(resolution.value, Some(16_777_227.5));
    assert!(resolution.depends_on_basis);
}

#[test]
fn geometry_supports_default_and_f64_scalars() {
    let default_size = crate::Size::new(2.0, 3.0);
    assert_eq!(default_size.width, 2.0);

    assert_eq!(crate::Point::<f64>::ZERO, Point::new(0.0, 0.0));
    assert_eq!(crate::Size::<f64>::ZERO, Size::new(0.0, 0.0));
    assert_eq!(crate::Edges::<f64>::ZERO, Edges::new(0.0, 0.0, 0.0, 0.0));

    let f64_size = crate::Size::<f64>::new(2.0_f64, 3.0_f64);
    assert_eq!(f64_size.height, 3.0_f64);

    let f64_edges = crate::Edges::<f64>::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(f64_edges.horizontal_sum(), 6.0_f64);
    assert_eq!(f64_edges.vertical_sum(), 4.0_f64);
    assert_eq!(f64_edges.sum_axes(), Size::new(6.0_f64, 4.0_f64));
}

#[test]
fn scroll_geometry_core_is_scalar_generic() {
    fn assert_scalar<S: crate::LayoutScalar>() {
        let range = crate::PhysicalScrollRangeOf::<S>::try_new(S::ZERO, S::ZERO, S::ZERO, S::ZERO)
            .expect("zero physical range is valid");
        assert_eq!(range.x().minimum(), S::ZERO);
        assert_eq!(range.y().maximum(), S::ZERO);
    }

    assert_scalar::<f32>();
    assert_scalar::<f64>();
}

#[test]
fn scroll_geometry_public_surface_uses_signed_physical_ranges() {
    fn assert_scalar<S: crate::LayoutScalar>() {
        let flow_axes = crate::FlowAxes::new(crate::WritingMode::VerticalRl, crate::Direction::Rtl);
        let range = crate::PhysicalScrollRangeOf::<S>::try_new(
            -S::from_f64(40.0),
            S::ZERO,
            -S::from_f64(30.0),
            S::ZERO,
        )
        .expect("finite signed physical range is valid");
        let scroll_axis = crate::ScrollContainerAxis::from_overflow(crate::Overflow::Scroll)
            .expect("scroll overflow is supported");
        let scrollport = crate::ScrollRectOf::new(
            crate::Point::ZERO,
            crate::Size::new(S::from_f64(100.0), S::from_f64(40.0)),
        )
        .expect("finite scrollport is valid");
        let geometry = crate::ScrollGeometryOf::new(
            flow_axes,
            crate::ScrollContainerFacts::new(scroll_axis, scroll_axis),
            scrollport,
            Some(scrollport),
            scrollport,
            range,
            crate::ScrollbarGutterRectsOf::new(None, None),
        )
        .expect("scroll geometry accepts signed exposed ranges");

        assert_eq!(geometry.flow_axes(), flow_axes);
        assert_eq!(geometry.physical_range(), range);
    }

    assert_scalar::<f32>();
    assert_scalar::<f64>();
}

#[test]
fn length_values_resolve_against_a_containing_size() {
    let px = Length::px(24.0).resolve(320.0);
    let percent = Length::percent(0.25).resolve(320.0);

    assert_eq!(px.value, Some(24.0));
    assert_eq!(px.status(), LengthResolutionStatus::Resolved);
    assert_eq!(percent.value, Some(80.0));
    assert_eq!(percent.status(), LengthResolutionStatus::Resolved);
}

#[test]
fn auto_lengths_resolve_to_optional_values() {
    let px = LengthAuto::px(12.0).resolve(200.0);
    let percent = LengthAuto::percent(0.5).resolve(200.0);
    let auto = LengthAuto::AUTO.resolve(200.0);

    assert_eq!(px.value, Some(12.0));
    assert_eq!(px.status(), LengthResolutionStatus::Resolved);
    assert_eq!(percent.value, Some(100.0));
    assert_eq!(percent.status(), LengthResolutionStatus::Resolved);
    assert_eq!(auto.value, None);
    assert_eq!(auto.status(), LengthResolutionStatus::NonNumeric);
}

#[test]
fn dimensions_preserve_layout_sizing_semantics() {
    let px = Dimension::px(42.0).resolve(100.0);
    let percent = Dimension::percent(0.25).resolve(100.0);
    let auto = Dimension::AUTO.resolve(100.0);

    assert_eq!(px.value, Some(42.0));
    assert_eq!(px.status(), LengthResolutionStatus::Resolved);
    assert_eq!(percent.value, Some(25.0));
    assert_eq!(percent.status(), LengthResolutionStatus::Resolved);
    assert_eq!(auto.value, None);
    assert_eq!(auto.status(), LengthResolutionStatus::NonNumeric);
    assert!(Dimension::MIN_CONTENT.is_min_content());
    assert!(Dimension::MAX_CONTENT.is_max_content());
}

#[test]
fn available_space_preserves_definite_min_and_max_content() {
    assert_eq!(Available::definite(128.0).into_option(), Some(128.0));
    assert_eq!(Available::MIN_CONTENT.into_option(), None);
    assert_eq!(Available::MAX_CONTENT.into_option(), None);
}

#[test]
fn sizes_and_edges_offer_algorithm_friendly_mapping() {
    let size = Size::new(100.0, 50.0).map(|value| value * 2.0);
    assert_eq!(size, Size::new(200.0, 100.0));

    let edges = Edges::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(edges.horizontal_sum(), 6.0);
    assert_eq!(edges.vertical_sum(), 4.0);
    assert_eq!(
        edges.zip_size(Size::new(10.0, 20.0), |edge, basis| edge + basis),
        Edges::new(21.0, 12.0, 23.0, 14.0)
    );
}

#[test]
fn node_input_defaults_match_the_layout_contract() {
    let node_input = NodeInput::default();

    assert_eq!(node_input.display, Display::Flex);
    assert_eq!(node_input.box_sizing, BoxSizing::BorderBox);
    assert_eq!(node_input.direction, Direction::Ltr);
    assert_eq!(node_input.text_align, TextAlign::Auto);
    assert_eq!(
        node_input.overflow,
        crate::Point::new(Overflow::Visible, Overflow::Visible)
    );
    assert_eq!(node_input.scrollbar_width.get(), 0.0);
    assert_eq!(node_input.position, Position::Relative);
    assert_eq!(node_input.inset, Edges::all(LengthAuto::AUTO));
    assert_eq!(node_input.size, Size::new(Dimension::AUTO, Dimension::AUTO));
    assert_eq!(
        node_input.min_size,
        Size::new(Dimension::AUTO, Dimension::AUTO)
    );
    assert_eq!(
        node_input.max_size,
        Size::new(Dimension::AUTO, Dimension::AUTO)
    );
    assert_eq!(node_input.margin, Edges::all(LengthAuto::ZERO));
    assert_eq!(node_input.padding, Edges::all(Length::ZERO));
    assert_eq!(node_input.border, Edges::all(Length::ZERO));
    assert_eq!(node_input.gap, Size::new(Length::NORMAL, Length::NORMAL));
    assert_eq!(node_input.flex_direction, FlexDirection::Row);
    assert_eq!(node_input.flex_wrap, FlexWrap::NoWrap);
    assert_eq!(node_input.flex_basis, Dimension::AUTO);
    assert_eq!(node_input.flex_grow.get(), 0.0);
    assert_eq!(node_input.flex_shrink.get(), 1.0);
    assert_eq!(
        node_input.grid_template_columns,
        Vec::<TrackComponent>::new()
    );
    assert_eq!(node_input.grid_template_rows, Vec::<TrackComponent>::new());
    assert_eq!(node_input.grid_auto_columns, Vec::<TrackComponent>::new());
    assert_eq!(node_input.grid_auto_rows, Vec::<TrackComponent>::new());
    assert_eq!(node_input.grid_auto_flow, GridAutoFlow::Row);
}

#[test]
fn node_input_numeric_wrappers_reject_negative_and_non_finite_values() {
    fn assert_rejects_invalid<T: core::fmt::Debug + PartialEq>(
        construct: impl Fn(f32) -> Result<T, NonNegativeFiniteScalarErrorOf<f32>>,
    ) {
        assert_eq!(
            construct(-1.0),
            Err(NonNegativeFiniteScalarErrorOf::Negative { value: -1.0 })
        );
        match construct(f32::NAN) {
            Err(NonNegativeFiniteScalarErrorOf::NonFinite { value }) => assert!(value.is_nan()),
            other => panic!("expected non-finite rejection for NaN, got {other:?}"),
        }
        assert_eq!(
            construct(f32::INFINITY),
            Err(NonNegativeFiniteScalarErrorOf::NonFinite {
                value: f32::INFINITY
            })
        );
    }

    assert_eq!(ScrollbarWidth::try_new(12.0).unwrap().get(), 12.0);
    assert_eq!(FlexGrow::try_new(2.0).unwrap().get(), 2.0);
    assert_eq!(FlexShrink::try_new(0.5).unwrap().get(), 0.5);

    assert_rejects_invalid(ScrollbarWidth::try_new);
    assert_rejects_invalid(FlexGrow::try_new);
    assert_rejects_invalid(FlexShrink::try_new);
}

#[test]
fn node_input_defaults_use_property_specific_numeric_wrappers() {
    let node_input = NodeInput::default();

    assert_eq!(node_input.scrollbar_width.get(), 0.0);
    assert_eq!(node_input.flex_grow.get(), 0.0);
    assert_eq!(node_input.flex_shrink.get(), 1.0);

    let node_input = NodeInputOf::<f64> {
        scrollbar_width: crate::ScrollbarWidthOf::try_new(3.0).unwrap(),
        flex_grow: FlexGrowOf::try_new(4.0).unwrap(),
        flex_shrink: FlexShrinkOf::try_new(5.0).unwrap(),
        ..NodeInputOf::<f64>::default()
    };

    assert_eq!(node_input.scrollbar_width.get(), 3.0);
    assert_eq!(node_input.flex_grow.get(), 4.0);
    assert_eq!(node_input.flex_shrink.get(), 5.0);
}

#[test]
fn line_break_input_defaults_to_visible_horizontal_break_context() {
    let input = LineBreakInput::default();
    assert_eq!(input.display(), LineBreakDisplay::Break);
    assert_eq!(input.direction(), Direction::Ltr);
    assert_eq!(input.writing_mode(), WritingMode::HorizontalTb);
    assert_eq!(input.vertical_align(), VerticalAlign::Baseline);
    assert_eq!(input.clear(), Clear::None);
}

#[test]
fn line_break_input_carries_inline_metrics() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 15.0).unwrap();
    let input = LineBreakInput::new().with_metrics(metrics);

    assert_eq!(input.metrics(), metrics);
    assert_eq!(input.metrics().line_extent(), 20.0);
}

#[test]
fn line_break_input_supports_f64_metrics() {
    let metrics = InlineMetricsOf::<f64>::from_line_height_and_baseline(32.0, 25.0).unwrap();
    let input = LineBreakInputOf::<f64>::new().with_metrics(metrics);

    assert_eq!(input.metrics().baseline(), 25.0);
}

#[test]
fn inline_boundary_input_requires_explicit_metrics() {
    let metrics = InlineMetrics::from_line_height_and_baseline(28.0, 20.0).unwrap();
    let input = InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics)
        .with_writing_mode(WritingMode::VerticalRl)
        .with_direction(Direction::Rtl)
        .with_vertical_align(VerticalAlign::Top);

    assert_eq!(input.kind(), InlineBoundaryKind::Start);
    assert_eq!(input.metrics(), metrics);
    assert_eq!(input.writing_mode(), WritingMode::VerticalRl);
    assert_eq!(input.direction(), Direction::Rtl);
    assert_eq!(input.vertical_align(), VerticalAlign::Top);
}

#[test]
fn inline_boundary_input_supports_f64_metrics() {
    let metrics = InlineMetricsOf::<f64>::from_line_height_and_baseline(40.0, 30.0).unwrap();
    let input = InlineBoundaryInputOf::<f64>::new(InlineBoundaryKind::End, metrics);

    assert_eq!(input.kind(), InlineBoundaryKind::End);
    assert_eq!(input.metrics().line_extent(), 40.0);
    assert_eq!(input.metrics().baseline(), 30.0);
}

#[test]
fn inline_metrics_validate_line_box_invariants() {
    let metrics = InlineMetrics::try_new(12.0, 18.0).unwrap();

    assert_eq!(metrics.baseline(), 12.0);
    assert_eq!(metrics.line_extent(), 18.0);
    assert_eq!(metrics.after_baseline(), 6.0);

    assert_eq!(
        InlineMetrics::try_new(19.0, 18.0),
        Err(InlineMetricsError::BaselineExceedsLineExtent {
            baseline: 19.0,
            line_extent: 18.0,
        })
    );
    assert_eq!(
        InlineMetrics::from_line_height_and_baseline(10.0, 12.0),
        Err(InlineMetricsError::BaselineExceedsLineHeight {
            baseline: 12.0,
            line_height: 10.0,
        })
    );
}

#[test]
fn inline_metrics_reject_non_finite_and_negative_values() {
    assert!(matches!(
        InlineMetrics::try_new(f32::NAN, 18.0),
        Err(InlineMetricsError::NonFinite { value }) if value.is_nan()
    ));
    assert_eq!(
        InlineMetrics::try_new(12.0, -18.0),
        Err(InlineMetricsError::Negative { value: -18.0 })
    );
}

#[test]
fn inline_metrics_support_f64_scalar_lane() {
    let metrics = InlineMetricsOf::<f64>::from_line_height_and_baseline(
        9_000_000_000_000.0,
        8_000_000_000_000.0,
    )
    .unwrap();

    assert_eq!(metrics.after_baseline(), 1_000_000_000_000.0);
}

#[test]
fn layout_input_distinguishes_box_from_line_break() {
    let box_input = LayoutInput::box_input(NodeInput::default());
    assert!(box_input.as_box().is_some());
    assert!(box_input.as_line_break().is_none());

    let line_break = LayoutInput::line_break(LineBreakInput::new().hidden());
    assert!(line_break.as_box().is_none());
    assert_eq!(
        line_break.as_line_break().unwrap().display(),
        LineBreakDisplay::None
    );
}

#[test]
fn layout_input_distinguishes_inline_boundary_from_boxes_and_breaks() {
    let metrics = InlineMetrics::from_line_height_and_baseline(18.0, 14.0).unwrap();
    let boundary = InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics);
    let layout_input = LayoutInput::inline_boundary(boundary);

    assert!(layout_input.as_box().is_none());
    assert!(layout_input.as_line_break().is_none());
    assert_eq!(layout_input.as_inline_boundary(), Some(boundary));
}

#[test]
fn node_input_does_not_carry_line_break_state() {
    let input = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };

    let layout_input = LayoutInput::box_input(input);
    assert!(layout_input.as_line_break().is_none());
}

#[test]
fn physical_geometry_retains_only_physical_components() {
    let size = Size::new(80.0, 24.0);
    assert_eq!(size.width, 80.0);
    assert_eq!(size.height, 24.0);

    let edges = Edges::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(edges.horizontal_sum(), 6.0);
    assert_eq!(edges.vertical_sum(), 4.0);
    assert_eq!(edges.sum_axes(), Size::new(6.0, 4.0));

    let point = Point::new(5.0, 9.0);
    assert_eq!(point.transpose(), Point::new(9.0, 5.0));
    assert_eq!(point.x, 5.0);
    assert_eq!(point.y, 9.0);
}

#[test]
fn node_input_defaults_include_flex_alignment_inputs() {
    let node_input = NodeInput::default();
    assert_eq!(node_input.align_items, None);
    assert_eq!(node_input.align_self, None);
    assert_eq!(node_input.justify_items, None);
    assert_eq!(node_input.justify_self, None);
    assert_eq!(node_input.align_content, None);
    assert_eq!(node_input.justify_content, None);
    assert_eq!(AlignContent::Start.reversed(), AlignContent::End);
    assert_eq!(AlignContent::Stretch.reversed(), AlignContent::End);
    assert_eq!(AlignItems::Stretch, AlignItems::Stretch);
}

#[test]
fn collapsible_margins_preserve_css_block_collapse_rules() {
    let margins = CollapsibleMargin::from_margin(12.0)
        .collapse_with_margin(4.0)
        .collapse_with_margin(-3.0)
        .collapse_with_margin(-8.0);

    assert_eq!(margins.resolve(), 4.0);
}

#[test]
fn public_flow_axes_cover_every_writing_mode_and_direction() {
    let cases = [
        (
            WritingMode::HorizontalTb,
            Direction::Ltr,
            PhysicalAxis::Horizontal,
            PhysicalAxis::Vertical,
            PhysicalSide::Left,
            PhysicalSide::Top,
            PhysicalSide::Top,
        ),
        (
            WritingMode::HorizontalTb,
            Direction::Rtl,
            PhysicalAxis::Horizontal,
            PhysicalAxis::Vertical,
            PhysicalSide::Right,
            PhysicalSide::Top,
            PhysicalSide::Top,
        ),
        (
            WritingMode::VerticalRl,
            Direction::Ltr,
            PhysicalAxis::Vertical,
            PhysicalAxis::Horizontal,
            PhysicalSide::Top,
            PhysicalSide::Right,
            PhysicalSide::Right,
        ),
        (
            WritingMode::VerticalRl,
            Direction::Rtl,
            PhysicalAxis::Vertical,
            PhysicalAxis::Horizontal,
            PhysicalSide::Bottom,
            PhysicalSide::Right,
            PhysicalSide::Right,
        ),
        (
            WritingMode::VerticalLr,
            Direction::Ltr,
            PhysicalAxis::Vertical,
            PhysicalAxis::Horizontal,
            PhysicalSide::Top,
            PhysicalSide::Left,
            PhysicalSide::Right,
        ),
        (
            WritingMode::VerticalLr,
            Direction::Rtl,
            PhysicalAxis::Vertical,
            PhysicalAxis::Horizontal,
            PhysicalSide::Bottom,
            PhysicalSide::Left,
            PhysicalSide::Right,
        ),
        (
            WritingMode::SidewaysRl,
            Direction::Ltr,
            PhysicalAxis::Vertical,
            PhysicalAxis::Horizontal,
            PhysicalSide::Top,
            PhysicalSide::Right,
            PhysicalSide::Right,
        ),
        (
            WritingMode::SidewaysRl,
            Direction::Rtl,
            PhysicalAxis::Vertical,
            PhysicalAxis::Horizontal,
            PhysicalSide::Bottom,
            PhysicalSide::Right,
            PhysicalSide::Right,
        ),
        (
            WritingMode::SidewaysLr,
            Direction::Ltr,
            PhysicalAxis::Vertical,
            PhysicalAxis::Horizontal,
            PhysicalSide::Bottom,
            PhysicalSide::Left,
            PhysicalSide::Left,
        ),
        (
            WritingMode::SidewaysLr,
            Direction::Rtl,
            PhysicalAxis::Vertical,
            PhysicalAxis::Horizontal,
            PhysicalSide::Top,
            PhysicalSide::Left,
            PhysicalSide::Left,
        ),
    ];

    for (writing_mode, direction, inline_axis, block_axis, inline_start, block_start, line_over) in
        cases
    {
        let flow_axes = FlowAxes::new(writing_mode, direction);

        assert_eq!(flow_axes.writing_mode(), writing_mode);
        assert_eq!(flow_axes.direction(), direction);
        assert_eq!(flow_axes.inline_axis(), inline_axis);
        assert_eq!(flow_axes.block_axis(), block_axis);
        assert_eq!(flow_axes.inline_start(), inline_start);
        assert_eq!(flow_axes.inline_end(), inline_start.opposite());
        assert_eq!(flow_axes.block_start(), block_start);
        assert_eq!(flow_axes.block_end(), block_start.opposite());
        assert_eq!(flow_axes.line_over(), line_over);
        assert_eq!(flow_axes.line_under(), line_over.opposite());
    }
}

#[test]
fn public_leaf_construction_retains_explicit_containing_flow() {
    let flow_axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr);
    let known = Size::new(Some(120.0), None);
    let parent = Size::new(Some(640.0), Some(480.0));
    let available = Size::new(Available::definite(640.0), Available::definite(480.0));

    let layout = ComputeInput::leaf_layout(known, parent, flow_axes, available)
        .expect("finite direct leaf layout input");
    let content_size = ComputeInput::leaf_content_size(known, parent, flow_axes, available)
        .expect("finite direct leaf content-size input");

    assert_eq!(layout.containing_flow_axes(), flow_axes);
    assert_eq!(content_size.containing_flow_axes(), flow_axes);
}

#[test]
fn public_diagnostics_report_physical_axes() {
    let root_error =
        LayoutRootRequest::viewport(Size::new(Available::definite(-1.0), Available::MAX_CONTENT))
            .expect_err("negative physical width is rejected");
    assert_eq!(root_error.axis(), PhysicalAxis::Horizontal);

    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let input = ComputeInput::leaf_layout(
        Size::NONE,
        Size::new(Some(640.0), Some(480.0)),
        flow_axes,
        Size::new(Available::definite(640.0), Available::definite(480.0)),
    )
    .expect("finite direct leaf input");
    let error = compute_leaf(input, &NodeInput::default(), |_| {
        Ok::<_, ()>(Size::new(-1.0, 0.0))
    })
    .expect_err("negative measurement output is rejected");
    let LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(output)) = error.kind()
    else {
        panic!("expected an invalid measurement output diagnostic");
    };
    assert_eq!(output.axis(), PhysicalAxis::Horizontal);
}

#[test]
fn node_input_default_retains_horizontal_tb_ltr_for_both_scalar_lanes() {
    fn assert_default_flow<S: LayoutScalar>() {
        let input = NodeInputOf::<S>::default();
        assert_eq!(input.writing_mode, WritingMode::HorizontalTb);
        assert_eq!(input.direction, Direction::Ltr);
    }

    assert_default_flow::<f32>();
    assert_default_flow::<f64>();
}
