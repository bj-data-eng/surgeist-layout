use crate::{
    Available, Baselines, CollapsibleMarginOf, ComputeOutput, Direction, Display, Edges, FlowAxes,
    LayoutScalar, Length, LengthAuto, LengthPercentageOf, LengthResolutionStatus, MaxTrackSizing,
    MinTrackSizing, PhysicalAxis, PhysicalBlockMarginCollapse, PhysicalBlockMarginCollapseOf,
    PhysicalSide, Point, PreferredSize, Scalar, Size, SizingCalculation, TrackComponent,
    TrackComponentList, TrackFlexFactor, TrackRepeatCount, TrackSizing, WritingMode,
};

fn assert_physical_block_margin_collapse_maps_all_flow_axes<S: LayoutScalar>() {
    let none = PhysicalBlockMarginCollapseOf::<S>::NONE;
    let block_start = CollapsibleMarginOf::from_margin(S::from_f64(5.0));
    let block_end = CollapsibleMarginOf::from_margin(S::from_f64(-3.0));
    let flows = [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::HorizontalTb, Direction::Rtl),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Rtl),
        (WritingMode::VerticalLr, Direction::Ltr),
        (WritingMode::VerticalLr, Direction::Rtl),
        (WritingMode::SidewaysRl, Direction::Ltr),
        (WritingMode::SidewaysRl, Direction::Rtl),
        (WritingMode::SidewaysLr, Direction::Ltr),
        (WritingMode::SidewaysLr, Direction::Rtl),
    ];

    for (writing_mode, direction) in flows {
        let flow = FlowAxes::new(writing_mode, direction);
        let carrier =
            PhysicalBlockMarginCollapseOf::from_block_flow(flow, block_start, block_end, true);

        for side in [
            PhysicalSide::Top,
            PhysicalSide::Right,
            PhysicalSide::Bottom,
            PhysicalSide::Left,
        ] {
            let expected = if side == flow.block_start() {
                block_start
            } else if side == flow.block_end() {
                block_end
            } else {
                CollapsibleMarginOf::ZERO
            };
            assert_eq!(carrier.at(side), expected);
            assert_eq!(none.at(side), CollapsibleMarginOf::ZERO);
        }

        let compatible_flow = match flow.block_start() {
            PhysicalSide::Top | PhysicalSide::Bottom => {
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl)
            }
            PhysicalSide::Right => FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr),
            PhysicalSide::Left => FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        };
        let orthogonal_flow = match flow.block_axis() {
            PhysicalAxis::Horizontal => FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            PhysicalAxis::Vertical => FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        };

        assert!(carrier.can_collapse_through(flow));
        assert!(carrier.can_collapse_through(compatible_flow));
        assert!(!carrier.can_collapse_through(orthogonal_flow));
        assert!(!none.can_collapse_through(flow));
    }
}

#[test]
fn physical_block_margin_collapse_maps_all_flow_axes_in_f32() {
    let default_none: PhysicalBlockMarginCollapse = PhysicalBlockMarginCollapse::NONE;
    assert_eq!(default_none, PhysicalBlockMarginCollapseOf::<f32>::NONE);
    assert_physical_block_margin_collapse_maps_all_flow_axes::<f32>();
}

#[test]
fn physical_block_margin_collapse_maps_all_flow_axes_in_f64() {
    assert_physical_block_margin_collapse_maps_all_flow_axes::<f64>();
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

    assert!(!PreferredSize::AUTO.depends_on_basis());
    assert!(!PreferredSize::px(12.0).depends_on_basis());
    assert!(PreferredSize::percent(0.25).depends_on_basis());
}

#[test]
fn layout_lengths_resolve_optional_basis_consistently() {
    let px_without_basis = Length::px(12.0).resolve_with_status(None);
    assert_eq!(px_without_basis.value, Some(12.0));
    assert_eq!(px_without_basis.status(), LengthResolutionStatus::Resolved);

    let percent_without_basis = Length::percent(0.25).resolve_with_status(None);
    assert_eq!(percent_without_basis.value, None);
    assert_eq!(
        percent_without_basis.status(),
        LengthResolutionStatus::MissingBasis
    );
    assert_eq!(
        Length::percent(0.25).resolve_with_status(Some(80.0)).value,
        Some(20.0)
    );
    assert_eq!(Length::percent(0.25).resolve_optional(None), None);
    assert_eq!(
        Length::percent(0.25).resolve_optional(Some(80.0)),
        Some(20.0)
    );

    let auto_resolution = LengthAuto::AUTO.resolve_with_status(Some(80.0));
    assert_eq!(auto_resolution.value, None);
    assert_eq!(auto_resolution.status(), LengthResolutionStatus::NonNumeric);
    assert_eq!(
        LengthAuto::percent(0.25).resolve_optional(Some(80.0)),
        Some(20.0)
    );
    assert_eq!(
        PreferredSize::percent(0.25)
            .resolve_simple_with_status(Some(80.0))
            .expect("affine preferred size is supported")
            .value,
        Some(20.0),
    );
}

fn mixed(absolute_px: f32, percent_fraction: f32) -> LengthPercentageOf {
    LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
        .expect("test coefficients are finite")
}

#[test]
fn affine_values_resolve_px_and_percent_coefficients_inline() {
    let value = mixed(12.0, 0.25);
    let length = Length::value(value);

    assert_eq!(value.absolute_px(), 12.0);
    assert_eq!(value.percent_fraction(), 0.25);
    assert!(length.depends_on_basis());
    assert_eq!(length.resolve_optional(Some(80.0)), Some(32.0));
    assert_eq!(length.resolve_optional(None), None);
}

#[test]
fn affine_values_report_basis_dependency_and_percent_fraction() {
    let px_only = Length::value(mixed(12.0, 0.0));
    let with_percent = Length::value(mixed(12.0, 0.25));

    assert!(!px_only.depends_on_basis());
    assert!(with_percent.depends_on_basis());
    assert_eq!(px_only.resolve_optional(None), Some(12.0));

    let unresolved = with_percent.resolve_with_status(None);
    assert_eq!(unresolved.value, None);
    assert!(unresolved.depends_on_basis);
    assert_eq!(with_percent.percent_fraction(), 0.25);
}

#[test]
fn affine_track_sizing_reports_signed_percent_fraction() {
    let value = mixed(0.0, 0.25);
    let track = TrackSizing::new(
        MinTrackSizing::Calculation(SizingCalculation::value(value)),
        MaxTrackSizing::Calculation(SizingCalculation::value(mixed(80.0, 0.0))),
    );

    assert_eq!(track.percent_fraction(), 0.25);
    assert_eq!(
        Length::value(value).resolve_optional(Some(200.0)),
        Some(50.0)
    );
}

#[test]
fn non_numeric_values_report_non_numeric_status() {
    assert_eq!(
        LengthAuto::AUTO.resolve_with_status(Some(40.0)).status(),
        LengthResolutionStatus::NonNumeric
    );
    assert_eq!(
        PreferredSize::AUTO
            .resolve_simple_with_status(Some(40.0))
            .expect("auto remains an existing non-numeric keyword")
            .status(),
        LengthResolutionStatus::NonNumeric
    );
    assert_eq!(
        PreferredSize::MIN_CONTENT
            .resolve_simple_with_status(Some(40.0))
            .expect("min-content remains an existing non-numeric keyword")
            .status(),
        LengthResolutionStatus::NonNumeric
    );
    assert_eq!(
        PreferredSize::MAX_CONTENT
            .resolve_simple_with_status(Some(40.0))
            .expect("max-content remains an existing non-numeric keyword")
            .status(),
        LengthResolutionStatus::NonNumeric
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
    assert!(
        TrackSizing::fit_content(SizingCalculation::value(mixed(0.0, 0.25))).depends_on_basis()
    );
    assert!(
        !TrackSizing::flex(TrackFlexFactor::try_new(1.0).expect("valid factor")).depends_on_basis()
    );
}

#[test]
fn affine_percent_track_participates_in_percent_detection() {
    let track = TrackSizing::new(
        MinTrackSizing::Calculation(SizingCalculation::value(mixed(20.0, 0.10))),
        MaxTrackSizing::Calculation(SizingCalculation::value(mixed(80.0, 0.0))),
    );

    assert!(track.depends_on_basis());
    assert_eq!(track.percent_fraction(), 0.10);
}

#[test]
fn affine_px_only_track_does_not_request_percent_rerun() {
    let track = TrackSizing::new(
        MinTrackSizing::Calculation(SizingCalculation::value(mixed(30.0, 0.0))),
        MaxTrackSizing::Calculation(SizingCalculation::value(mixed(80.0, 0.0))),
    );

    assert!(!track.depends_on_basis());
    assert_eq!(track.percent_fraction(), 0.0);
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
