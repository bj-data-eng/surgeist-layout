use crate::*;
use crate::{CalcResolver, DefaultScalar};

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
    assert_eq!(length.resolve(400.0), 100.0);

    let dimension = crate::DimensionOf::<f64>::px(42.5);
    assert_eq!(dimension.resolve(1000.0), Some(42.5));

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
        flex_grow: 1.0,
        ..crate::NodeInputOf::<f64>::default()
    };

    assert_eq!(input.size.width.resolve(1000.0), Some(123.5));
    assert_eq!(input.size.height.resolve(400.0), Some(100.0));

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
fn f64_calc_resolution_preserves_large_coordinate_precision() {
    let mut store = crate::LayoutCalcStoreOf::<f64>::new();
    let id = store.push(crate::CalcExpressionOf::sum(vec![
        crate::CalcTermOf::px(16_777_217.0),
        crate::CalcTermOf::percent(0.5),
    ]));

    let resolution = store.resolve_calc(id, Some(21.0));
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
fn length_values_resolve_against_a_containing_size() {
    assert_eq!(Length::px(24.0).resolve(320.0), 24.0);
    assert_eq!(Length::percent(0.25).resolve(320.0), 80.0);
}

#[test]
fn auto_lengths_resolve_to_optional_values() {
    assert_eq!(LengthAuto::px(12.0).resolve(200.0), Some(12.0));
    assert_eq!(LengthAuto::percent(0.5).resolve(200.0), Some(100.0));
    assert_eq!(LengthAuto::AUTO.resolve(200.0), None);
}

#[test]
fn dimensions_preserve_layout_sizing_semantics() {
    assert_eq!(Dimension::px(42.0).resolve(100.0), Some(42.0));
    assert_eq!(Dimension::percent(0.25).resolve(100.0), Some(25.0));
    assert_eq!(Dimension::AUTO.resolve(100.0), None);
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
    assert_eq!(node_input.scrollbar_width, 0.0);
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
    assert_eq!(node_input.flex_grow, 0.0);
    assert_eq!(node_input.flex_shrink, 1.0);
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
fn node_input_defaults_to_box_inline_role() {
    assert_eq!(NodeInput::default().inline_role, InlineRole::Box);
}

#[test]
fn inline_role_marks_line_break_semantics_without_changing_display() {
    let input = NodeInput {
        display: Display::Block,
        inline_role: InlineRole::LineBreak,
        ..NodeInput::default()
    };

    assert!(input.inline_role.is_line_break());
    assert_eq!(input.display, Display::Block);
}

#[test]
fn geometry_reports_main_and_cross_components_for_flex_direction() {
    let size = Size::new(80.0, 24.0);
    assert_eq!(size.main(FlexDirection::Row), 80.0);
    assert_eq!(size.cross(FlexDirection::Row), 24.0);
    assert_eq!(size.main(FlexDirection::Column), 24.0);
    assert_eq!(size.cross(FlexDirection::Column), 80.0);
    assert_eq!(
        Size::from_cross(FlexDirection::Row, Some(12.0)),
        Size::new(None, Some(12.0))
    );

    let edges = Edges::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(edges.main_sum(FlexDirection::Row), 6.0);
    assert_eq!(edges.cross_sum(FlexDirection::Row), 4.0);
    assert_eq!(edges.main_sum(FlexDirection::Column), 4.0);
    assert_eq!(edges.cross_sum(FlexDirection::Column), 6.0);

    let point = Point::new(5.0, 9.0);
    assert_eq!(point.transpose(), Point::new(9.0, 5.0));
    assert_eq!(point.main(FlexDirection::Row), 5.0);
    assert_eq!(point.cross(FlexDirection::Column), 5.0);
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
