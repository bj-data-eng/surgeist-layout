use super::{
    Available, Baselines, ComputeOutput, Dimension, Display, Edges, Length, LengthAuto,
    NoCalcResolver, Point, Size,
};

#[test]
fn dimension_conversions_keep_semantic_variants() {
    assert_eq!(Dimension::from(Length::px(8.0)), Dimension::px(8.0));
    assert_eq!(
        Dimension::from(LengthAuto::percent(0.75)),
        Dimension::percent(0.75)
    );
    assert_eq!(Dimension::from(LengthAuto::AUTO), Dimension::AUTO);
}

#[test]
fn edge_axis_sums_match_layout_axis_expectations() {
    let edges = Edges::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(edges.sum_axes(), Size::new(6.0, 4.0));
}

#[test]
fn available_space_only_exposes_definite_values() {
    assert_eq!(Available::definite(12.0).into_option(), Some(12.0));
    assert_eq!(Available::MIN_CONTENT.into_option(), None);
    assert_eq!(Available::MAX_CONTENT.into_option(), None);
}

#[test]
fn layout_lengths_report_basis_dependency() {
    assert!(!Length::NORMAL.depends_on_basis());
    assert!(!Length::px(12.0).depends_on_basis());
    assert!(Length::percent(0.25).depends_on_basis());

    assert!(!LengthAuto::AUTO.depends_on_basis());
    assert!(!LengthAuto::px(12.0).depends_on_basis());
    assert!(LengthAuto::percent(0.25).depends_on_basis());

    assert!(!Dimension::AUTO.depends_on_basis());
    assert!(!Dimension::px(12.0).depends_on_basis());
    assert!(Dimension::percent(0.25).depends_on_basis());
}

#[test]
fn layout_lengths_resolve_optional_basis_consistently() {
    assert_eq!(Length::px(12.0).resolve_or_zero(None), 12.0);
    assert_eq!(Length::percent(0.25).resolve_or_zero(None), 0.0);
    assert_eq!(Length::percent(0.25).resolve_or_zero(Some(80.0)), 20.0);
    assert_eq!(Length::percent(0.25).resolve_optional(None), None);
    assert_eq!(
        Length::percent(0.25).resolve_optional(Some(80.0)),
        Some(20.0)
    );

    assert_eq!(LengthAuto::AUTO.resolve_or_zero(Some(80.0)), 0.0);
    assert_eq!(
        LengthAuto::percent(0.25).resolve_optional(Some(80.0)),
        Some(20.0)
    );
    assert_eq!(
        Dimension::percent(0.25).resolve_optional(Some(80.0)),
        Some(20.0)
    );
}

#[test]
fn no_calc_resolver_keeps_plain_values_working() {
    let resolver = NoCalcResolver;
    assert_eq!(
        Length::px(8.0).resolve_with(Some(40.0), &resolver),
        Some(8.0)
    );
    assert_eq!(
        Length::percent(0.5).resolve_with(Some(40.0), &resolver),
        Some(20.0)
    );
}

#[test]
fn compute_output_preserves_first_and_last_baselines() {
    let output = ComputeOutput::from_sizes_and_baselines(
        Size::new(40.0, 30.0),
        Size::ZERO,
        Baselines {
            first: Point::new(None, Some(8.0)),
            last: Point::new(None, Some(24.0)),
        },
    );

    assert_eq!(output.first_baselines.y, Some(8.0));
    assert_eq!(output.last_baselines.y, Some(24.0));
}

#[test]
fn compute_output_from_sizes_has_no_explicit_baselines() {
    let output = ComputeOutput::from_sizes(Size::new(40.0, 30.0), Size::ZERO);

    assert_eq!(output.first_baselines, Point::NONE);
    assert_eq!(output.last_baselines, Point::NONE);
}

#[test]
fn inline_display_values_preserve_outer_participation_and_inner_context() {
    assert!(Display::InlineBlock.is_inline_level());
    assert!(Display::InlineGrid.is_inline_level());
    assert!(Display::InlineGridLanes.is_inline_level());

    assert_eq!(Display::InlineBlock.inner_display(), Display::Block);
    assert_eq!(Display::InlineGrid.inner_display(), Display::Grid);
    assert_eq!(Display::InlineGridLanes.inner_display(), Display::GridLanes);

    assert!(!Display::Block.is_inline_level());
    assert_eq!(Display::Grid.inner_display(), Display::Grid);
}

#[test]
fn grid_formatting_context_values_include_inline_grid_variants() {
    assert!(Display::Grid.establishes_grid_formatting_context());
    assert!(Display::GridLanes.establishes_grid_formatting_context());
    assert!(Display::InlineGrid.establishes_grid_formatting_context());
    assert!(Display::InlineGridLanes.establishes_grid_formatting_context());
    assert!(!Display::InlineBlock.establishes_grid_formatting_context());

    assert!(!Display::Grid.establishes_grid_lanes_formatting_context());
    assert!(Display::GridLanes.establishes_grid_lanes_formatting_context());
    assert!(!Display::InlineGrid.establishes_grid_lanes_formatting_context());
    assert!(Display::InlineGridLanes.establishes_grid_lanes_formatting_context());
}
