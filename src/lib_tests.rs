use crate::{
    Available, Baselines, CalcExpression, CalcResolutionStatus, CalcResolver, CalcTerm,
    ComputeOutput, Dimension, Display, Edges, LayoutCalcStore, Length, LengthAuto, MaxTrackSizing,
    MinTrackSizing, NoCalcResolver, Point, Scalar, Size, TrackComponent, TrackComponentList,
    TrackRepeatCount, TrackSizing,
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
fn layout_calc_store_resolves_px_and_percent_terms() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(12.0),
        CalcTerm::percent(0.25),
    ]));

    assert_eq!(id.index(), 0);
    assert_eq!(store.len(), 1);
    assert!(!store.is_empty());
    assert!(store.calc_depends_on_basis(id));

    let resolved = store.resolve_calc(id, Some(80.0));
    assert_eq!(resolved.value, Some(32.0));
    assert!(resolved.depends_on_basis);
}

#[test]
fn layout_calc_store_reports_basis_dependency_and_unresolved_percent() {
    let mut store = LayoutCalcStore::new();
    let px_only = store.push(CalcExpression::sum([CalcTerm::px(12.0)]));
    let with_percent = store.push(CalcExpression::sum([
        CalcTerm::px(12.0),
        CalcTerm::percent(0.25),
    ]));
    let unknown = super::CalcId::from_raw_for_tests(99);

    assert!(!store.calc_depends_on_basis(px_only));
    assert!(store.calc_depends_on_basis(with_percent));
    assert!(!store.calc_depends_on_basis(unknown));

    assert_eq!(store.resolve_calc(px_only, None).value, Some(12.0));

    let unresolved = store.resolve_calc(with_percent, None);
    assert_eq!(unresolved.value, None);
    assert!(unresolved.depends_on_basis);

    let unknown_resolution = store.resolve_calc(unknown, Some(80.0));
    assert_eq!(unknown_resolution.value, None);
    assert!(!unknown_resolution.depends_on_basis);
}

#[test]
fn missing_calc_id_reports_missing_expression() {
    let store = LayoutCalcStore::new();
    let missing = super::CalcId::from_raw_for_tests(99);
    let resolution = store.resolve_calc(missing, Some(80.0));

    assert_eq!(resolution.value, None);
    assert!(resolution.is_missing_expression());
    assert_eq!(resolution.status(), CalcResolutionStatus::MissingExpression);
}

#[test]
fn calc_values_require_an_explicit_resolver() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([CalcTerm::px(8.0)]));

    assert!(Length::calc(id).requires_resolver());
    assert_eq!(
        Length::calc(id)
            .resolve_with_status(Some(40.0), &NoCalcResolver)
            .status(),
        CalcResolutionStatus::MissingResolver
    );
}

#[test]
fn layout_calc_store_reports_signed_percent_fraction() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::percent(0.50),
        CalcTerm::percent(-0.25),
    ]));
    let track = TrackSizing::new(
        MinTrackSizing::Length(Length::calc(id)),
        MaxTrackSizing::Length(Length::px(80.0)),
    );

    assert_eq!(store.calc_percent_fraction(id), Some(0.25));
    assert_eq!(track.percent_fraction_with(&store), 0.25);
    assert_eq!(store.resolve_calc(id, Some(200.0)).value, Some(50.0));
}

#[test]
fn length_calc_resolves_through_resolver_hook() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(8.0),
        CalcTerm::percent(0.5),
    ]));
    let length = Length::calc(id);

    assert_eq!(length.resolve_with(Some(40.0), &store), Some(28.0));
    assert_eq!(length.resolve_with(None, &store), None);
}

#[test]
fn length_calc_reports_basis_dependency_through_resolver_hook() {
    let mut store = LayoutCalcStore::new();
    let px_only = store.push(CalcExpression::sum([CalcTerm::px(8.0)]));
    let with_percent = store.push(CalcExpression::sum([
        CalcTerm::px(8.0),
        CalcTerm::percent(0.5),
    ]));

    assert!(Length::calc(px_only).depends_on_basis());
    assert!(!Length::calc(px_only).depends_on_basis_with(&store));
    assert!(Length::calc(with_percent).depends_on_basis_with(&store));
}

#[test]
#[should_panic(expected = "calc values require an explicit resolver")]
fn length_calc_cannot_resolve_without_resolver() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([CalcTerm::px(8.0)]));

    let _ = Length::calc(id).resolve_optional(Some(40.0));
}

#[test]
#[should_panic(expected = "calc values require an explicit resolver")]
fn length_auto_calc_cannot_resolve_without_resolver() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([CalcTerm::px(8.0)]));

    let _ = LengthAuto::calc(id).resolve_or_zero(Some(40.0));
}

#[test]
#[should_panic(expected = "calc values require an explicit resolver")]
fn dimension_calc_cannot_resolve_without_resolver() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([CalcTerm::px(8.0)]));

    let _ = Dimension::calc(id).resolve_optional(Some(40.0));
}

#[test]
fn non_numeric_values_report_non_numeric_status() {
    assert_eq!(
        LengthAuto::AUTO
            .resolve_with_status(Some(40.0), &NoCalcResolver)
            .status(),
        CalcResolutionStatus::NonNumeric
    );
    assert_eq!(
        Dimension::fr(1.0)
            .resolve_with_status(Some(40.0), &NoCalcResolver)
            .status(),
        CalcResolutionStatus::NonNumeric
    );
    assert_eq!(
        Dimension::AUTO
            .resolve_with_status(Some(40.0), &NoCalcResolver)
            .status(),
        CalcResolutionStatus::NonNumeric
    );
    assert_eq!(
        Dimension::MIN_CONTENT
            .resolve_with_status(Some(40.0), &NoCalcResolver)
            .status(),
        CalcResolutionStatus::NonNumeric
    );
    assert_eq!(
        Dimension::MAX_CONTENT
            .resolve_with_status(Some(40.0), &NoCalcResolver)
            .status(),
        CalcResolutionStatus::NonNumeric
    );
}

#[test]
fn aspect_ratio_rejects_non_positive_or_non_finite_values() {
    assert!(super::AspectRatio::new(1.5).is_some());
    assert_eq!(super::AspectRatio::new(0.0), None);
    assert_eq!(super::AspectRatio::new(-1.0), None);
    assert_eq!(super::AspectRatio::new(Scalar::NAN), None);
    assert_eq!(super::AspectRatio::new(Scalar::INFINITY), None);
}

#[test]
fn track_repetition_rejects_zero_count_and_empty_components() {
    assert!(TrackRepeatCount::new(0).is_none());
    assert!(TrackRepeatCount::new(2).is_some());
    assert!(TrackComponentList::try_from(Vec::<TrackComponent>::new()).is_err());
}

#[test]
fn track_sizing_components_empty_slice_uses_default_scalar_api() {
    assert!(super::track_sizing_components(&[]).is_empty());
}

#[test]
fn track_sizing_reports_basis_dependency() {
    assert!(!TrackSizing::px(12.0).depends_on_basis());
    assert!(TrackSizing::percent(0.25).depends_on_basis());
    assert!(TrackSizing::fit_content(Length::percent(0.25)).depends_on_basis());
    assert!(!TrackSizing::fr(1.0).depends_on_basis());
}

#[test]
fn calc_percent_track_participates_in_percent_detection() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(20.0),
        CalcTerm::percent(0.10),
    ]));
    let track = TrackSizing::new(
        MinTrackSizing::Length(Length::calc(id)),
        MaxTrackSizing::Length(Length::px(80.0)),
    );

    assert!(track.depends_on_basis_with(&store));
    assert_eq!(track.percent_fraction_with(&store), 0.10);
}

#[test]
fn calc_px_only_track_does_not_request_percent_rerun() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(20.0),
        CalcTerm::px(10.0),
    ]));
    let track = TrackSizing::new(
        MinTrackSizing::Length(Length::calc(id)),
        MaxTrackSizing::Length(Length::px(80.0)),
    );

    assert!(!track.depends_on_basis_with(&store));
    assert_eq!(track.percent_fraction_with(&store), 0.0);
}

#[test]
fn track_sizing_definite_uses_shared_optional_basis_resolution() {
    let track = TrackSizing::percent(0.25);
    assert_eq!(track.min.definite(None), None);
    assert_eq!(track.min.definite(Some(80.0)), Some(20.0));
    assert_eq!(track.max.definite(None), None);
    assert_eq!(track.max.definite(Some(80.0)), Some(20.0));
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
