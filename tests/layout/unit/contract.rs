use super::*;

#[test]
fn layout_scalar_is_single_precision() {
    assert_eq!(std::mem::size_of::<Scalar>(), std::mem::size_of::<f32>());
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
        surgeist_layout::Point::new(Overflow::Visible, Overflow::Visible)
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
