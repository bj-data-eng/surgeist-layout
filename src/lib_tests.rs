use crate::{
    Available, Baselines, CollapsibleMarginOf, ComputeOutput, Direction, Display, Edges,
    FlexItemCollapse, FloatExclusionInterval, FloatExclusionIntervalError,
    FloatExclusionIntervalErrorOf, FloatExclusionIntervalOf, FloatExclusionQuery,
    FloatExclusionQueryOf, FlowAxes, LayoutOperation, LayoutScalar, Length, LengthAuto,
    LengthPercentageOf, LengthResolutionStatus, MaxTrackSizing, MinTrackSizing, PhysicalAxis,
    PhysicalBlockMarginCollapse, PhysicalBlockMarginCollapseOf, PhysicalSide, Point, PreferredSize,
    Scalar, Size, SizingCalculation, TrackComponent, TrackComponentList, TrackFlexFactor,
    TrackRepeatCount, TrackSizing, WritingMode,
};

#[test]
fn fri07_c02_model_public_type_is_two_state_and_has_exact_required_traits() {
    fn assert_traits<
        T: Clone + Copy + core::fmt::Debug + Default + Eq + core::hash::Hash + PartialEq,
    >() {
    }

    assert_traits::<FlexItemCollapse>();
    assert_eq!(FlexItemCollapse::default(), FlexItemCollapse::Normal);

    let states = [FlexItemCollapse::Normal, FlexItemCollapse::Collapsed];
    let names = states.map(|state| match state {
        FlexItemCollapse::Normal => "normal",
        FlexItemCollapse::Collapsed => "collapsed",
    });
    assert_eq!(names, ["normal", "collapsed"]);
}

#[test]
fn fri07_c02_model_all_node_input_construction_paths_are_normal() {
    fn collapse_of<S: LayoutScalar>(input: &crate::NodeInputOf<S>) -> FlexItemCollapse {
        input.flex_item_collapse
    }

    assert_eq!(
        collapse_of(&crate::NodeInput::DEFAULT),
        FlexItemCollapse::Normal
    );
    assert_eq!(
        collapse_of(&crate::NodeInputOf::<f32>::default()),
        FlexItemCollapse::Normal
    );
    assert_eq!(
        collapse_of(&crate::NodeInputOf::<f64>::default()),
        FlexItemCollapse::Normal
    );
    assert_eq!(
        collapse_of(&crate::NodeInputOf::<f32>::non_box()),
        FlexItemCollapse::Normal
    );
    assert_eq!(
        collapse_of(&crate::NodeInputOf::<f64>::non_box()),
        FlexItemCollapse::Normal
    );
}

#[test]
fn fri07_c02_model_collapsed_is_inert_outside_in_flow_flex_participation() {
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    use crate::{
        AvailableOf, CompletedLayoutBatchOf, GridPlacement, LayoutRootRequestOf, NodeInputOf,
        NodeOutputOf, Position, PreferredSizeOf, SubgridTrack, TrackComponentOf, compute_layout,
    };

    fn sized<S: LayoutScalar>(display: Display, width: f64, height: f64) -> NodeInputOf<S> {
        NodeInputOf {
            display,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::px(S::from_f64(height)),
            ),
            ..NodeInputOf::default()
        }
    }

    fn with_collapse<S: LayoutScalar>(
        mut input: NodeInputOf<S>,
        collapse: FlexItemCollapse,
    ) -> NodeInputOf<S> {
        input.flex_item_collapse = collapse;
        input
    }

    fn assert_output_fields_equal<S: LayoutScalar>(
        context: &str,
        normal: NodeOutputOf<S>,
        collapsed: NodeOutputOf<S>,
    ) {
        assert_eq!(normal.source_index, collapsed.source_index, "{context}");
        assert_eq!(normal.location, collapsed.location, "{context}");
        assert_eq!(normal.size, collapsed.size, "{context}");
        assert_eq!(normal.content_size, collapsed.content_size, "{context}");
        assert_eq!(
            normal.scroll_geometry, collapsed.scroll_geometry,
            "{context}"
        );
        assert_eq!(normal.border, collapsed.border, "{context}");
        assert_eq!(normal.padding, collapsed.padding, "{context}");
        assert_eq!(normal.margin, collapsed.margin, "{context}");
    }

    fn assert_batches_equal<S: LayoutScalar>(
        context: &str,
        normal: &CompletedLayoutBatchOf<u32, S>,
        collapsed: &CompletedLayoutBatchOf<u32, S>,
    ) {
        assert_eq!(
            normal.unrounded_entries().len(),
            collapsed.unrounded_entries().len(),
            "{context} unrounded entry count"
        );
        for (normal_entry, collapsed_entry) in normal
            .unrounded_entries()
            .iter()
            .zip(collapsed.unrounded_entries())
        {
            assert_eq!(normal_entry.node(), collapsed_entry.node(), "{context}");
            assert_output_fields_equal(context, normal_entry.output(), collapsed_entry.output());
        }

        assert_eq!(
            normal.final_entries().len(),
            collapsed.final_entries().len(),
            "{context} final entry count"
        );
        for (normal_entry, collapsed_entry) in
            normal.final_entries().iter().zip(collapsed.final_entries())
        {
            assert_eq!(normal_entry.node(), collapsed_entry.node(), "{context}");
            assert_output_fields_equal(context, normal_entry.output(), collapsed_entry.output());
        }

        assert_eq!(
            normal.unrounded_inline_fragments(),
            collapsed.unrounded_inline_fragments(),
            "{context} unrounded inline fragments"
        );
        assert_eq!(
            normal.final_inline_fragments(),
            collapsed.final_inline_fragments(),
            "{context} final inline fragments"
        );
        assert_eq!(
            normal.cache_store_entries(),
            collapsed.cache_store_entries(),
            "{context} cache stores"
        );
        assert_eq!(
            normal.cache_clear_entries(),
            collapsed.cache_clear_entries(),
            "{context} cache clears"
        );
        assert_eq!(
            normal.invalidated_nodes(),
            collapsed.invalidated_nodes(),
            "{context} invalidated nodes"
        );
    }

    fn assert_case<S, Build>(context: &str, build: Build)
    where
        S: LayoutScalar,
        Build: Fn(FlexItemCollapse) -> PublicLayoutTreeOf<S>,
    {
        let available = Size::new(
            AvailableOf::definite(S::from_f64(180.0)),
            AvailableOf::definite(S::from_f64(120.0)),
        );
        let request = LayoutRootRequestOf::viewport(available).expect("finite viewport");
        let normal = compute_layout(&build(FlexItemCollapse::Normal), 0, request)
            .expect("normal inert-context layout succeeds");
        let collapsed = compute_layout(&build(FlexItemCollapse::Collapsed), 0, request)
            .expect("collapsed inert-context layout succeeds");
        assert_batches_equal(context, &normal, &collapsed);
    }

    fn assert_lane<S: LayoutScalar>() {
        assert_case::<S, _>("root", |collapse| {
            PublicLayoutTreeOf::new()
                .children(0, [])
                .style(0, with_collapse(sized(Display::Flex, 90.0, 50.0), collapse))
        });

        for (context, display) in [
            ("block child", Display::Block),
            ("grid child", Display::Grid),
            ("grid-lanes child", Display::GridLanes),
        ] {
            assert_case::<S, _>(context, |collapse| {
                PublicLayoutTreeOf::new()
                    .children(0, [1])
                    .children(1, [])
                    .style(0, sized(display, 120.0, 80.0))
                    .style(
                        1,
                        with_collapse(sized(Display::Block, 30.0, 20.0), collapse),
                    )
            });
        }

        assert_case::<S, _>("subgrid child", |collapse| {
            let root = NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(120.0)),
                    PreferredSizeOf::px(S::from_f64(80.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(S::from_f64(120.0))],
                grid_template_rows: vec![TrackComponentOf::px(S::from_f64(80.0))],
                ..NodeInputOf::default()
            };
            let subgrid = NodeInputOf {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                    Vec::new(),
                ))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))],
                grid_column: GridPlacement::try_lines(1, -1).expect("full column span"),
                grid_row: GridPlacement::try_lines(1, -1).expect("full row span"),
                ..NodeInputOf::default()
            };
            PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [2])
                .children(2, [])
                .style(0, root)
                .style(1, subgrid)
                .style(
                    2,
                    with_collapse(sized(Display::Block, 30.0, 20.0), collapse),
                )
        });

        assert_case::<S, _>("measured leaf", |collapse| {
            PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, sized(Display::Block, 120.0, 80.0))
                .style(1, with_collapse(NodeInputOf::<S>::default(), collapse))
                .measure(1, Size::new(S::from_f64(33.0), S::from_f64(17.0)))
        });

        assert_case::<S, _>("child of positioned context", |collapse| {
            PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [2])
                .children(2, [])
                .style(0, sized(Display::Block, 120.0, 80.0))
                .style(
                    1,
                    NodeInputOf {
                        position: Position::Absolute,
                        ..sized(Display::Block, 80.0, 40.0)
                    },
                )
                .style(
                    2,
                    with_collapse(sized(Display::Block, 30.0, 20.0), collapse),
                )
        });

        assert_case::<S, _>("absolute flex child", |collapse| {
            PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, sized(Display::Flex, 120.0, 80.0))
                .style(
                    1,
                    with_collapse(
                        NodeInputOf {
                            position: Position::Absolute,
                            ..sized(Display::Block, 30.0, 20.0)
                        },
                        collapse,
                    ),
                )
        });

        assert_case::<S, _>("display-none flex child", |collapse| {
            PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [2])
                .children(2, [])
                .style(0, sized(Display::Flex, 120.0, 80.0))
                .style(
                    1,
                    with_collapse(
                        NodeInputOf {
                            display: Display::None,
                            ..NodeInputOf::default()
                        },
                        collapse,
                    ),
                )
                .style(2, sized(Display::Block, 30.0, 20.0))
        });
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c01_contract_float_exclusion_public_aliases_and_operations_are_exact() {
    fn aliases(
        _: Option<FloatExclusionQuery>,
        _: Option<FloatExclusionInterval>,
        _: Option<FloatExclusionIntervalError>,
    ) {
    }
    fn generic_aliases(
        _: Option<FloatExclusionQueryOf<f64>>,
        _: Option<FloatExclusionIntervalOf<f64>>,
        _: Option<FloatExclusionIntervalErrorOf<f64>>,
    ) {
    }
    aliases(None, None, None);
    generic_aliases(None, None, None);

    let operation_name = |operation| match operation {
        LayoutOperation::RootLayout => "root",
        LayoutOperation::ChildLayout => "child",
        LayoutOperation::HiddenLayout => "hidden",
        LayoutOperation::LeafMeasurement => "measure",
        LayoutOperation::ValueResolution => "resolve",
        LayoutOperation::CacheAccess => "cache",
        LayoutOperation::CacheInvalidation => "invalidate",
        LayoutOperation::FloatExclusionQuery => "float-exclusion",
        LayoutOperation::RoundingFinalization => "round",
        LayoutOperation::GridLanePlacement => "grid-lanes",
    };
    assert_eq!(
        operation_name(LayoutOperation::FloatExclusionQuery),
        "float-exclusion"
    );
}

#[test]
fn fri05_c01_computed_overflow_public_reexports_compose() {
    use crate::{ComputedOverflow, ComputedOverflowError, Overflow};

    let pair: ComputedOverflow = ComputedOverflow::try_new(Overflow::Auto, Overflow::Hidden)
        .expect("canonical public pair constructs");
    assert_eq!((pair.x(), pair.y()), (Overflow::Auto, Overflow::Hidden));

    let error: ComputedOverflowError = ComputedOverflow::try_new(Overflow::Clip, Overflow::Scroll)
        .expect_err("cross-group public pair is rejected");
    assert_eq!(
        error,
        ComputedOverflowError::NonCanonicalPair {
            x: Overflow::Clip,
            y: Overflow::Scroll,
        }
    );
}

#[test]
fn fri05_c01_scroll_input_public_aliases_and_reexports_compose() {
    use crate::{
        LengthPercentageOf, OverflowClipBox, OverflowClipMargin, ScrollMargin, ScrollMarginError,
        ScrollPadding, ScrollPaddingValue, ScrollSnapAlign, ScrollSnapAlignValue, ScrollSnapAxis,
        ScrollSnapStop, ScrollSnapStrictness, ScrollSnapType, ScrollbarGutter,
    };

    let clip_margin: OverflowClipMargin =
        OverflowClipMargin::try_new(OverflowClipBox::ContentBox, 3.0)
            .expect("default scalar clip margin");
    assert_eq!(clip_margin.margin(), 3.0);

    let value: ScrollPaddingValue = ScrollPaddingValue::value(
        LengthPercentageOf::from_percent_fraction(0.25).expect("finite percentage"),
    );
    let padding: ScrollPadding = ScrollPadding::new(
        ScrollPaddingValue::AUTO,
        value,
        ScrollPaddingValue::AUTO,
        value,
    );
    assert_eq!(padding.right(), value);

    let margin: ScrollMargin =
        ScrollMargin::try_new(-1.0, 2.0, 3.0, 4.0).expect("finite signed margins");
    assert_eq!(margin.top(), -1.0);
    let _: ScrollMarginError = ScrollMargin::try_new(f32::NAN, 0.0, 0.0, 0.0)
        .expect_err("public default-scalar error alias");

    let _ = ScrollbarGutter::StableBothEdges;
    let _ = ScrollSnapType::Enabled {
        axis: ScrollSnapAxis::Inline,
        strictness: ScrollSnapStrictness::Mandatory,
    };
    let alignment = ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::Start);
    assert_eq!(alignment.block(), ScrollSnapAlignValue::Center);
    let _ = ScrollSnapStop::Always;

    let generic_clip = crate::OverflowClipMarginOf::<f64>::try_new(OverflowClipBox::BorderBox, 5.0)
        .expect("generic clip margin");
    let generic_padding = crate::ScrollPaddingOf::<f64>::default();
    let generic_margin = crate::ScrollMarginOf::<f64>::default();
    assert_eq!(generic_clip.margin(), 5.0);
    assert!(generic_padding.top().is_auto());
    assert_eq!(generic_margin.left(), 0.0);
}

#[test]
fn fri05_c02_carrier_public_aliases_reexports_and_rect_error_compose() {
    use crate::{
        OverflowClip, OverflowClipOf, PhysicalClipAxis, PhysicalClipAxisOf, ScrollRect,
        ScrollRectError, ScrollRectErrorOf, ScrollTargetGeometry, ScrollTargetGeometryOf,
    };

    fn accept_default_carriers(
        _: Option<PhysicalClipAxis>,
        _: Option<OverflowClip>,
        _: Option<ScrollTargetGeometry>,
    ) {
    }
    fn accept_generic_carriers(
        _: Option<PhysicalClipAxisOf<f64>>,
        _: Option<OverflowClipOf<f64>>,
        _: Option<ScrollTargetGeometryOf<f64>>,
    ) {
    }

    accept_default_carriers(None, None, None);
    accept_generic_carriers(None, None, None);

    let error: ScrollRectError =
        ScrollRect::try_new(Point::new(f32::MAX, 0.0), Size::new(f32::MAX, 0.0))
            .expect_err("default-scalar rectangle error alias");
    assert_eq!(
        error,
        ScrollRectErrorOf::NonFiniteEnd {
            axis: PhysicalAxis::Horizontal,
            value: f32::INFINITY,
            origin: f32::MAX,
            size: f32::MAX,
        }
    );

    let generic_error: ScrollRectErrorOf<f64> =
        crate::ScrollRectOf::try_new(Point::new(0.0, f64::MAX), Size::new(0.0, f64::MAX))
            .expect_err("generic rectangle error reexport");
    assert_eq!(
        generic_error,
        ScrollRectErrorOf::NonFiniteEnd {
            axis: PhysicalAxis::Vertical,
            value: f64::INFINITY,
            origin: f64::MAX,
            size: f64::MAX,
        }
    );
}

#[test]
fn fri05_c07_public_surface_default_and_f64_input_error_output_contracts_compose() {
    fn checked_input<S: crate::LayoutScalar>() -> crate::NodeInputOf<S> {
        let overflow =
            crate::ComputedOverflow::try_new(crate::Overflow::Auto, crate::Overflow::Scroll)
                .expect("canonical computed overflow pair");
        let clip_margin: Result<
            crate::OverflowClipMarginOf<S>,
            crate::NonNegativeFiniteScalarErrorOf<S>,
        > = crate::OverflowClipMarginOf::try_new(crate::OverflowClipBox::PaddingBox, S::ZERO);
        let scrollbar_width: Result<
            crate::ScrollbarWidthOf<S>,
            crate::NonNegativeFiniteScalarErrorOf<S>,
        > = crate::ScrollbarWidthOf::try_new(S::ZERO);
        let scroll_margin: Result<crate::ScrollMarginOf<S>, crate::ScrollMarginErrorOf<S>> =
            crate::ScrollMarginOf::try_new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        let scroll_padding = crate::ScrollPaddingOf::new(
            crate::ScrollPaddingValueOf::AUTO,
            crate::ScrollPaddingValueOf::auto(),
            crate::ScrollPaddingValueOf::default(),
            crate::ScrollPaddingValueOf::value(crate::LengthPercentageOf::ZERO),
        );
        crate::NodeInputOf::<S> {
            overflow,
            overflow_clip_margin: clip_margin.expect("finite clip margin"),
            scrollbar_gutter: crate::ScrollbarGutter::StableBothEdges,
            scrollbar_width: scrollbar_width.expect("finite scrollbar width"),
            scroll_padding,
            scroll_margin: scroll_margin.expect("finite scroll margin"),
            scroll_snap_type: crate::ScrollSnapType::Enabled {
                axis: crate::ScrollSnapAxis::Block,
                strictness: crate::ScrollSnapStrictness::Proximity,
            },
            scroll_snap_align: crate::ScrollSnapAlign::new(
                crate::ScrollSnapAlignValue::Start,
                crate::ScrollSnapAlignValue::Center,
            ),
            scroll_snap_stop: crate::ScrollSnapStop::Always,
            ..crate::NodeInputOf::<S>::default()
        }
    }

    fn inspect_read_only_output<S: crate::LayoutScalar>(
        output: crate::NodeOutputOf<S>,
        geometry: Option<crate::ScrollGeometryOf<S>>,
        clip_axis: Option<crate::PhysicalClipAxisOf<S>>,
        clip: Option<crate::OverflowClipOf<S>>,
        gutters: Option<crate::ScrollbarGutterRectsOf<S>>,
        target: Option<crate::ScrollTargetGeometryOf<S>>,
    ) {
        let _ = (output.content_box_size(), output.scrollbar_size());
        if let Some(axis) = clip_axis {
            let _ = (axis.minimum(), axis.maximum());
        }
        if let Some(clip) = clip {
            let _ = (clip.x(), clip.y());
        }
        if let Some(gutters) = gutters {
            let _ = (
                gutters.top(),
                gutters.right(),
                gutters.bottom(),
                gutters.left(),
            );
        }
        if let Some(target) = target {
            let _ = (
                target.border_box(),
                target.scroll_margin(),
                target.flow_axes(),
                target.snap_align(),
                target.snap_stop(),
            );
        }
        if let Some(geometry) = geometry {
            let range = geometry.physical_range();
            let _ = (
                geometry.flow_axes(),
                geometry.used_overflow_x(),
                geometry.used_overflow_y(),
                geometry.border_box(),
                geometry.padding_box(),
                geometry.content_box(),
                geometry.scrollport(),
                geometry.overflow_clip(),
                geometry.scrollable_overflow(),
                range.x().minimum(),
                range.x().maximum(),
                range.y().minimum(),
                range.y().maximum(),
                geometry.gutters(),
                geometry.scrollbar_size(),
                geometry.resolved_scroll_padding(),
                geometry.optimal_viewing_region(),
                geometry.scroll_snap_type(),
                geometry.target(),
            );
        }
    }

    fn checked_coordinates<S: crate::LayoutScalar>() {
        let physical_offset: Result<
            crate::PhysicalScrollOffsetOf<S>,
            crate::ScrollCoordinateErrorOf<S>,
        > = crate::PhysicalScrollOffsetOf::try_new(S::ZERO, S::ZERO);
        let flow_offset: Result<
            crate::FlowRelativeScrollOffsetOf<S>,
            crate::ScrollCoordinateErrorOf<S>,
        > = crate::FlowRelativeScrollOffsetOf::try_new(S::ZERO, S::ZERO);
        let physical_range: Result<
            crate::PhysicalScrollRangeOf<S>,
            crate::ScrollCoordinateErrorOf<S>,
        > = crate::PhysicalScrollRangeOf::try_new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        let flow_range: Result<
            crate::FlowRelativeScrollRangeOf<S>,
            crate::ScrollCoordinateErrorOf<S>,
        > = crate::FlowRelativeScrollRangeOf::try_new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        let rect: Result<crate::ScrollRectOf<S>, crate::ScrollRectErrorOf<S>> =
            crate::ScrollRectOf::try_new(crate::Point::ZERO, crate::Size::ZERO);

        let physical_offset = physical_offset.expect("finite physical offset");
        let flow_offset = flow_offset.expect("finite flow-relative offset");
        let physical_range = physical_range.expect("finite ordered physical range");
        let flow_range = flow_range.expect("finite ordered flow-relative range");
        assert_eq!(physical_range.clamp(physical_offset), physical_offset);
        assert_eq!(flow_range.clamp(flow_offset), flow_offset);
        assert_eq!(rect.expect("finite rectangle").size(), crate::Size::ZERO);

        let _: Option<crate::PhysicalScrollAxisRangeOf<S>> = Some(physical_range.x());
        let _: Option<crate::FlowRelativeScrollAxisRangeOf<S>> = Some(flow_range.inline());
    }

    let default = checked_input::<f32>();
    let generic = checked_input::<f64>();
    assert_eq!(default.overflow.x(), crate::Overflow::Auto);
    assert_eq!(generic.overflow.y(), crate::Overflow::Scroll);
    let _: crate::OverflowClipMargin = default.overflow_clip_margin;
    let _: crate::ScrollbarGutter = default.scrollbar_gutter;
    let _: crate::ScrollbarWidth = default.scrollbar_width;
    let _: crate::ScrollPadding = default.scroll_padding;
    let _: crate::ScrollPaddingValue = default.scroll_padding.top();
    let _: crate::ScrollMargin = default.scroll_margin;
    let _: crate::ScrollSnapType = default.scroll_snap_type;
    let _: crate::ScrollSnapAlign = default.scroll_snap_align;
    let _: crate::ScrollSnapStop = default.scroll_snap_stop;
    let _: crate::OverflowClipMarginOf<f64> = generic.overflow_clip_margin;
    let _: crate::ScrollbarWidthOf<f64> = generic.scrollbar_width;
    let _: crate::ScrollPaddingOf<f64> = generic.scroll_padding;
    let _: crate::ScrollPaddingValueOf<f64> = generic.scroll_padding.top();
    let _: crate::ScrollMarginOf<f64> = generic.scroll_margin;
    checked_coordinates::<f32>();
    checked_coordinates::<f64>();

    let _: crate::NodeInput = default;
    let _: crate::ComputedOverflowError =
        crate::ComputedOverflow::try_new(crate::Overflow::Visible, crate::Overflow::Auto)
            .expect_err("noncanonical pair");
    let _: crate::ScrollMarginError =
        crate::ScrollMargin::try_new(f32::NAN, 0.0, 0.0, 0.0).expect_err("non-finite margin");
    let _: crate::ScrollRectError =
        crate::ScrollRect::try_new(crate::Point::new(f32::NAN, 0.0), crate::Size::ZERO)
            .expect_err("non-finite rectangle");
    let _: crate::ScrollCoordinateError =
        crate::PhysicalScrollRange::try_new(1.0, 0.0, 0.0, 0.0).expect_err("inverted range");

    let _: Option<crate::PhysicalScrollOffset> = None;
    let _: Option<crate::FlowRelativeScrollOffset> = None;
    let _: Option<crate::PhysicalScrollAxisRange> = None;
    let _: Option<crate::FlowRelativeScrollAxisRange> = None;
    let _: Option<crate::PhysicalScrollRange> = None;
    let _: Option<crate::FlowRelativeScrollRange> = None;
    let _: Option<crate::ScrollRect> = None;
    let _: Option<crate::PhysicalClipAxis> = None;
    let _: Option<crate::OverflowClip> = None;
    let _: Option<crate::ScrollbarGutterRects> = None;
    let _: Option<crate::ScrollTargetGeometry> = None;
    let _: Option<crate::ScrollGeometry> = None;
    inspect_read_only_output(crate::NodeOutput::default(), None, None, None, None, None);
    inspect_read_only_output(
        crate::NodeOutputOf::<f64>::default(),
        None,
        None,
        None,
        None,
        None,
    );
}

#[test]
fn fri04_c04_dispatch_public_descriptor_front_door_has_closed_copy_hash_contract() {
    fn assert_closed<T: Clone + Copy + core::fmt::Debug + Eq + core::hash::Hash + PartialEq>() {}

    assert_closed::<crate::SizingProperty>();
    assert_closed::<crate::SizingAlgorithm>();
    assert_closed::<crate::CalcSizeBehaviorBasis>();
    assert_closed::<crate::SizingBehavior>();
    assert_closed::<crate::UnsupportedSizingBehavior>();
    assert_closed::<crate::LayoutUnsupportedCapability>();

    let _ = crate::SizingProperty::Preferred;
    let _ = crate::SizingAlgorithm::Positioned;
    let _ = crate::CalcSizeBehaviorBasis::Content;
    let _ = crate::SizingBehavior::CalcSize(crate::CalcSizeBehaviorBasis::None);
}

#[test]
fn fri04_c06_public_surface_default_and_f64_checked_reexports_compose() {
    use crate::{
        CalcSizeBehaviorBasis, CalcSizeCalculation, CalcSizeCalculationErrorOf,
        CalcSizeCalculationOf, CalcSizeConstructionError, FlexBasis, FlexBasisCalcBasis,
        FlexBasisOf, LayoutUnsupportedCapability, MaxSize, MaxSizeCalcBasis, MaxSizeOf,
        MaxTrackSizingOf, MinSize, MinSizeCalcBasis, MinSizeOf, MinTrackSizingOf,
        PreferredSizeCalcBasis, PreferredSizeOf, SizingAlgorithm, SizingBehavior,
        SizingCalculationError, SizingCalculationOf, SizingProperty, TrackFlexFactorOf,
        TrackSizingOf, UnsupportedSizingBehavior,
    };

    fn affine<S: LayoutScalar>(absolute_px: f64, percent_fraction: f64) -> LengthPercentageOf<S> {
        LengthPercentageOf::from_coefficients(
            S::from_f64(absolute_px),
            S::from_f64(percent_fraction),
        )
        .expect("characterization coefficients are finite")
    }

    let default_min = SizingCalculation::min(vec![
        SizingCalculation::value(affine::<f32>(8.0, 0.0)),
        SizingCalculation::value(affine::<f32>(12.0, 0.1)),
    ])
    .expect("ordinary minimum is nonempty");
    let default_max = SizingCalculation::max(vec![
        SizingCalculation::value(affine::<f32>(48.0, 0.0)),
        SizingCalculation::value(affine::<f32>(64.0, 0.0)),
    ])
    .expect("ordinary maximum is nonempty");
    let default_ordinary: SizingCalculation = SizingCalculation::clamp(
        Some(default_min),
        SizingCalculation::value(affine::<f32>(40.0, 0.0)),
        Some(default_max),
    );

    let default_preferred: PreferredSize = PreferredSize::calculation(default_ordinary.clone());
    let default_minimum: MinSize = MinSize::calculation(default_ordinary.clone());
    let default_maximum: MaxSize = MaxSize::calculation(default_ordinary.clone());
    let default_flex: FlexBasis = FlexBasis::calculation(default_ordinary.clone());
    assert!(default_preferred.is_calculation());
    assert!(default_minimum.is_calculation());
    assert!(default_maximum.is_calculation());
    assert!(default_flex.is_calculation());
    assert_eq!(PreferredSize::default(), PreferredSize::AUTO);
    assert_eq!(MinSize::default(), MinSize::AUTO);
    assert_eq!(MaxSize::default(), MaxSize::NONE);
    assert_eq!(FlexBasis::default(), FlexBasis::AUTO);
    assert!(FlexBasis::CONTENT.is_content());

    let default_calc: CalcSizeCalculation = CalcSizeCalculation::from_coefficients(4.0, 0.25, 0.5)
        .expect("default calc-size coefficients are finite");
    assert!(
        PreferredSize::calc_size(PreferredSizeCalcBasis::Auto, default_calc.clone())
            .expect("preferred calc-size basis is valid")
            .is_calc_size()
    );
    assert!(
        MinSize::calc_size(MinSizeCalcBasis::MinContent, default_calc.clone())
            .expect("minimum calc-size basis is valid")
            .is_calc_size()
    );
    assert!(
        MaxSize::calc_size(MaxSizeCalcBasis::None, default_calc.clone())
            .expect("maximum calc-size basis is valid")
            .is_calc_size()
    );
    assert!(
        FlexBasis::calc_size(FlexBasisCalcBasis::Content, default_calc)
            .expect("flex calc-size basis is valid")
            .is_calc_size()
    );

    let default_factor: TrackFlexFactor =
        TrackFlexFactor::try_new(1.5).expect("default track flex is finite and non-negative");
    let default_track: TrackSizing = TrackSizing::new(
        MinTrackSizing::Calculation(default_ordinary),
        MaxTrackSizing::flex(default_factor),
    );
    assert!(default_track.max.is_flexible());
    assert!(TrackFlexFactor::try_new(-1.0).is_err());

    let f64_min = SizingCalculationOf::<f64>::min(vec![
        SizingCalculationOf::value(affine::<f64>(8.0, 0.0)),
        SizingCalculationOf::value(affine::<f64>(12.0, 0.1)),
    ])
    .expect("generic ordinary minimum is nonempty");
    let f64_max = SizingCalculationOf::<f64>::max(vec![
        SizingCalculationOf::value(affine::<f64>(48.0, 0.0)),
        SizingCalculationOf::value(affine::<f64>(64.0, 0.0)),
    ])
    .expect("generic ordinary maximum is nonempty");
    let f64_ordinary: SizingCalculationOf<f64> = SizingCalculationOf::clamp(
        Some(f64_min),
        SizingCalculationOf::value(affine::<f64>(40.0, 0.0)),
        Some(f64_max),
    );

    let f64_preferred: PreferredSizeOf<f64> = PreferredSizeOf::calculation(f64_ordinary.clone());
    let f64_minimum: MinSizeOf<f64> = MinSizeOf::calculation(f64_ordinary.clone());
    let f64_maximum: MaxSizeOf<f64> = MaxSizeOf::calculation(f64_ordinary.clone());
    let f64_flex: FlexBasisOf<f64> = FlexBasisOf::calculation(f64_ordinary.clone());
    assert!(f64_preferred.is_calculation());
    assert!(f64_minimum.is_calculation());
    assert!(f64_maximum.is_calculation());
    assert!(f64_flex.is_calculation());

    let f64_calc: CalcSizeCalculationOf<f64> =
        CalcSizeCalculationOf::from_coefficients(4.0, 0.25, 0.5)
            .expect("generic calc-size coefficients are finite");
    assert!(
        PreferredSizeOf::<f64>::calc_size(PreferredSizeCalcBasis::FullPercentage, f64_calc.clone())
            .expect("generic preferred calc-size basis is valid")
            .is_calc_size()
    );
    assert!(
        MinSizeOf::<f64>::calc_size(MinSizeCalcBasis::Auto, f64_calc.clone())
            .expect("generic minimum calc-size basis is valid")
            .is_calc_size()
    );
    assert!(
        MaxSizeOf::<f64>::calc_size(MaxSizeCalcBasis::MaxContent, f64_calc.clone())
            .expect("generic maximum calc-size basis is valid")
            .is_calc_size()
    );
    assert!(
        FlexBasisOf::<f64>::calc_size(FlexBasisCalcBasis::Content, f64_calc)
            .expect("generic flex calc-size basis is valid")
            .is_calc_size()
    );

    let f64_factor: TrackFlexFactorOf<f64> =
        TrackFlexFactorOf::try_new(2.0).expect("generic track flex is finite and non-negative");
    let f64_track: TrackSizingOf<f64> = TrackSizingOf::new(
        MinTrackSizingOf::Calculation(f64_ordinary),
        MaxTrackSizingOf::flex(f64_factor),
    );
    assert!(f64_track.max.is_flexible());
    assert!(TrackFlexFactorOf::<f64>::try_new(f64::INFINITY).is_err());

    let shape_error: SizingCalculationError =
        SizingCalculation::min(Vec::new()).expect_err("empty extrema are rejected");
    assert_eq!(shape_error, SizingCalculationError::EmptyArguments);
    let default_coefficient_error: CalcSizeCalculationErrorOf<f32> =
        CalcSizeCalculation::from_coefficients(f32::NAN, 0.0, 0.0)
            .expect_err("non-finite default coefficients are rejected");
    assert!(matches!(
        default_coefficient_error,
        CalcSizeCalculationErrorOf::InvalidAbsolutePx(_)
    ));
    let f64_coefficient_error: CalcSizeCalculationErrorOf<f64> =
        CalcSizeCalculationOf::from_coefficients(0.0, 0.0, f64::NAN)
            .expect_err("non-finite generic coefficients are rejected");
    assert!(matches!(
        f64_coefficient_error,
        CalcSizeCalculationErrorOf::InvalidSizeFraction(_)
    ));
    let construction_error: CalcSizeConstructionError =
        PreferredSize::calc_size(PreferredSizeCalcBasis::Any, CalcSizeCalculation::size())
            .expect_err("Any basis cannot consume a size reference");
    assert_eq!(
        construction_error,
        CalcSizeConstructionError::SizeReferenceWithAnyBasis
    );

    fn inspect_descriptor(
        descriptor: UnsupportedSizingBehavior,
    ) -> (
        SizingProperty,
        SizingBehavior,
        SizingAlgorithm,
        PhysicalAxis,
        LayoutUnsupportedCapability,
    ) {
        (
            descriptor.property(),
            descriptor.behavior(),
            descriptor.algorithm(),
            descriptor.axis(),
            LayoutUnsupportedCapability::SizingBehavior(descriptor),
        )
    }

    let _inspect: fn(
        UnsupportedSizingBehavior,
    ) -> (
        SizingProperty,
        SizingBehavior,
        SizingAlgorithm,
        PhysicalAxis,
        LayoutUnsupportedCapability,
    ) = inspect_descriptor;
    let property = SizingProperty::FlexBasis;
    let algorithm = SizingAlgorithm::GridLanes;
    let behavior = SizingBehavior::CalcSize(CalcSizeBehaviorBasis::Content);
    let capability = LayoutUnsupportedCapability::LaterFriBehavior;
    assert_eq!(property, SizingProperty::FlexBasis);
    assert_eq!(algorithm, SizingAlgorithm::GridLanes);
    assert_eq!(
        behavior,
        SizingBehavior::CalcSize(CalcSizeBehaviorBasis::Content)
    );
    assert_eq!(capability, LayoutUnsupportedCapability::LaterFriBehavior);
}

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
