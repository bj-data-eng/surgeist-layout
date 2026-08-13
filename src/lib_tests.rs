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
fn fri06_c05_contract_float_exclusion_surface_is_opaque_cache_neutral_and_active() {
    let node_input = include_str!("node_input.rs");
    let traits = format!(
        "{}\n{}",
        include_str!("tree.rs"),
        include_str!("engine/contracts.rs")
    );
    let compute = include_str!("compute.rs");
    let block = include_str!("block.rs");
    let cache = include_str!("cache.rs");
    let public_front_door = include_str!("lib.rs");

    for public_name in [
        "FloatExclusionQueryOf",
        "FloatExclusionQuery",
        "FloatExclusionIntervalOf",
        "FloatExclusionInterval",
        "FloatExclusionIntervalErrorOf",
        "FloatExclusionIntervalError",
    ] {
        assert!(
            public_front_door.contains(public_name),
            "{public_name} is reexported"
        );
    }

    let shape = node_input
        .split_once("pub enum FloatExclusion")
        .unwrap()
        .1
        .split_once("impl Float")
        .unwrap()
        .0;
    assert!(
        shape.contains("Shape,"),
        "Shape is a payload-free closed state"
    );
    assert!(
        !shape.contains("Shape("),
        "Shape carries no compatibility payload"
    );
    assert!(shape.contains("#[default]") && shape.contains("MarginBox"));

    for type_name in ["FloatExclusionQueryOf", "FloatExclusionIntervalOf"] {
        let section = node_input
            .split_once(&format!("pub struct {type_name}"))
            .unwrap()
            .1
            .split_once('}')
            .unwrap()
            .0;
        assert!(!section.contains("pub "), "{type_name} fields stay private");
    }
    assert!(!node_input.contains("FloatExclusionQueryOf::default"));
    assert!(!node_input.contains("Default for FloatExclusionQueryOf"));
    assert!(!node_input.contains("Default for FloatExclusionIntervalOf"));
    assert!(
        !cache.contains("revision"),
        "the cache context has no revision field"
    );
    assert!(!node_input.contains(concat!("ShapeExclusion", "Query")));
    assert!(!public_front_door.contains(concat!("ShapeExclusion", "Query")));

    assert_eq!(traits.matches("fn float_exclusion_interval(").count(), 2);
    assert!(traits.contains("Option<FloatExclusionProviderResultOf<Self::Scalar"));
    assert!(
        traits.contains("None\n    }"),
        "the provider defaults to no result"
    );
    assert!(compute.contains(".float_exclusion_interval("));
    assert!(block.contains("FloatExclusion::Shape"));
    assert!(block.contains("FloatExclusionIntervalErrorOf::QueryMismatch"));
    assert!(block.contains("LayoutMissingContext::FloatExclusionProvider"));
    assert!(!public_front_door.contains("Provider invocation and float-band refinement"));
}

#[test]
fn fri06_c01_contract_aggregate_public_surface_covers_every_cycle_break_and_addition() {
    let node_input = include_str!("node_input.rs");
    let output = include_str!("output.rs");
    let traits = format!(
        "{}\n{}",
        include_str!("tree.rs"),
        include_str!("engine/contracts.rs")
    );
    let error = include_str!("error.rs");
    let public_front_door = include_str!("lib.rs");

    for public_name in [
        "InlineSegmentId",
        "BidiLevel",
        "InlineWhitespaceEdge",
        "InlineBreakKind",
        "InlineBreakOpportunityOf",
        "InlineBreakOpportunity",
        "ShapedInlineSegmentOf",
        "ShapedInlineSegment",
        "InlineTextInputOf",
        "InlineTextInput",
        "AtomicInlineParticipationOf",
        "AtomicInlineParticipation",
        "InlineFragmentOutputOf",
        "InlineFragmentOutput",
        "InlineFragmentOutputEntryOf",
        "InlineFragmentOutputEntry",
        "FloatExclusion",
        "FloatExclusionQueryOf",
        "FloatExclusionQuery",
        "FloatExclusionIntervalOf",
        "FloatExclusionInterval",
        "FloatExclusionIntervalErrorOf",
        "FloatExclusionIntervalError",
        "LayoutBatchSink",
        "compute_layout_invalidated",
    ] {
        assert!(
            public_front_door.contains(public_name),
            "{public_name} is present at the crate front door"
        );
    }

    for required_source in [
        "InlineText(InlineTextInputOf<S>)",
        "pub atomic_inline_participation: Option<AtomicInlineParticipationOf<S>>",
        "pub float_exclusion: FloatExclusion",
        "pub fn non_box() -> Self",
        "Bottom",
    ] {
        assert!(
            node_input.contains(required_source),
            "missing {required_source}"
        );
    }
    for required_source in [
        "pub fn unrounded_inline_fragments(&self)",
        "pub fn final_inline_fragments(&self)",
        "pub fn invalidated_nodes(&self)",
        "pub fn apply_to<Sink>(&self",
    ] {
        assert!(
            output.contains(required_source),
            "missing {required_source}"
        );
    }
    for required_source in [
        "fn float_exclusion_interval(",
        "fn unrounded_inline_fragments(",
        "pub trait LayoutBatchSink",
    ] {
        assert!(
            traits.contains(required_source),
            "missing {required_source}"
        );
    }
    for required_source in [
        "FloatExclusionProviderOutput",
        "InvalidationNodeNotReachable",
        "MissingCachedInlineFragmentState",
        "FloatExclusionProvider",
        "FloatExclusionQuery",
        "CacheInvalidation",
    ] {
        assert!(error.contains(required_source), "missing {required_source}");
    }

    for forbidden_compatibility_name in [
        concat!("ShapeExclusion", "Query"),
        concat!("ShapeExclusion", "Interval"),
        concat!("InlineText", "Run"),
        concat!("DirtyLayout", "Request"),
    ] {
        assert!(!node_input.contains(forbidden_compatibility_name));
        assert!(!output.contains(forbidden_compatibility_name));
        assert!(!public_front_door.contains(forbidden_compatibility_name));
    }
}

#[test]
fn fri05_c01_node_input_removed_phase_unsafe_surfaces_are_absent_from_public_sources() {
    let node_input = include_str!("node_input.rs");
    let scroll = include_str!("scroll.rs");
    let public_front_door = include_str!("lib.rs");

    assert!(!node_input.contains(concat!("pub const fn clips_", "contents")));
    assert!(!node_input.contains(concat!("pub const fn blocks_margin_", "collapse")));
    assert!(!scroll.contains(concat!("is_phase_one_", "deferred")));
    assert!(!scroll.contains(concat!("ScrollOverflow", "CouplingPolicy")));
    assert!(!public_front_door.contains(concat!("ScrollOverflow", "CouplingPolicy")));

    for removed_variant in [
        concat!("Overflow", "Auto"),
        concat!("OverflowClip", "Margin"),
        concat!("ScrollbarGutter", "Stable"),
        concat!("ScrollbarGutter", "BothEdges"),
        concat!("LayoutOwnedMixedAxisOverflow", "Coupling"),
    ] {
        assert!(!scroll.contains(removed_variant));
    }
    assert_eq!(
        scroll.matches(concat!("Scroll", "Padding")).count(),
        4,
        "scroll retains only the canonical input references in its owned inset constructor"
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
fn fri05_c02_carrier_private_fields_constructors_and_no_default_are_static() {
    fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
        source
            .split_once(start)
            .expect("start marker")
            .1
            .split_once(end)
            .expect("end marker")
            .0
    }

    let scroll = include_str!("scroll.rs");
    let public_front_door = include_str!("lib.rs");
    let carrier_sections = [
        (
            "PhysicalClipAxisOf",
            between(
                scroll,
                "pub struct PhysicalClipAxisOf",
                "pub struct OverflowClipOf",
            ),
        ),
        (
            "OverflowClipOf",
            between(
                scroll,
                "pub struct OverflowClipOf",
                "pub struct ScrollTargetGeometryOf",
            ),
        ),
        (
            "ScrollTargetGeometryOf",
            between(
                scroll,
                "pub struct ScrollTargetGeometryOf",
                "/// Construction error for a signed physical or flow-relative scroll coordinate.",
            ),
        ),
    ];

    for (type_name, section) in carrier_sections {
        let fields = section
            .split_once('{')
            .expect("carrier fields begin")
            .1
            .split_once('}')
            .expect("carrier fields end")
            .0;
        assert!(!fields.contains("pub "), "{type_name} fields stay private");
        for public_constructor in [
            "pub fn new(",
            "pub const fn new(",
            "pub fn try_new(",
            "pub const fn try_new(",
        ] {
            assert!(
                !section.contains(public_constructor),
                "{type_name} has no public constructor"
            );
        }
        assert!(
            !scroll.contains(&format!("Default for {type_name}")),
            "{type_name} has no Default implementation"
        );

        let declaration = format!("pub struct {type_name}");
        let declaration_index = scroll.find(&declaration).expect("carrier declaration");
        let derive_start = scroll[..declaration_index]
            .rfind("#[derive(")
            .expect("carrier derive");
        let derive = &scroll[derive_start..declaration_index];
        assert!(
            !derive.contains("Default"),
            "{type_name} does not derive Default"
        );
    }

    for public_name in [
        "PhysicalClipAxisOf",
        "PhysicalClipAxis",
        "OverflowClipOf",
        "OverflowClip",
        "ScrollTargetGeometryOf",
        "ScrollTargetGeometry",
        "ScrollRectErrorOf",
        "ScrollRectError",
    ] {
        assert!(
            public_front_door.contains(public_name),
            "{public_name} is reexported"
        );
    }
}

#[test]
fn fri05_c03_legacy_surface_rect_has_only_the_typed_public_constructor() {
    let scroll = include_str!("scroll.rs");
    let rect_impl = scroll
        .split_once("impl<S: LayoutScalar> ScrollRectOf<S> {")
        .expect("rectangle implementation")
        .1
        .split_once("/// A finite ordered physical clip interval.")
        .expect("rectangle implementation end")
        .0;

    assert!(!rect_impl.contains("pub fn new("));
    assert_eq!(rect_impl.matches("pub fn try_new(").count(), 1);
}

#[test]
fn fri05_c03_public_geometry_surface_has_exact_read_only_accessors() {
    let scroll = include_str!("scroll.rs");
    let public_front_door = include_str!("lib.rs");

    fn assert_read_only_output_carrier(source: &str, type_name: &str, section_end: &str) {
        let declaration = format!("pub struct {type_name}");
        let declaration_index = source.find(&declaration).expect("carrier declaration");
        let section = source[declaration_index..]
            .split_once(section_end)
            .expect("carrier section end")
            .0;
        let fields = section
            .split_once('{')
            .expect("carrier fields begin")
            .1
            .split_once('}')
            .expect("carrier fields end")
            .0;
        assert!(!fields.contains("pub "), "{type_name} fields stay private");
        for public_constructor in [
            "pub fn new(",
            "pub const fn new(",
            "pub fn try_new(",
            "pub const fn try_new(",
        ] {
            assert!(
                !section.contains(public_constructor),
                "{type_name} has no public constructor"
            );
        }
        assert!(
            !source.contains(&format!("Default for {type_name}")),
            "{type_name} has no Default implementation"
        );

        let derive_start = source[..declaration_index]
            .rfind("#[derive(")
            .expect("carrier derive");
        assert!(
            !source[derive_start..declaration_index].contains("Default"),
            "{type_name} does not derive Default"
        );
    }

    assert_read_only_output_carrier(
        scroll,
        "ScrollbarGutterRectsOf",
        "pub(crate) struct ClipMarginSourceOf",
    );
    assert_read_only_output_carrier(
        scroll,
        "ScrollGeometryOf",
        "pub(crate) enum CanonicalScrollRectFact",
    );

    let geometry_impl = scroll
        .split_once("pub struct ScrollGeometryOf")
        .expect("canonical public geometry declaration")
        .1
        .split_once("pub type ScrollGeometry")
        .expect("canonical default-scalar alias")
        .1
        .split_once("pub(crate) enum CanonicalScrollRectFact")
        .expect("canonical geometry implementation end")
        .0;

    let geometry_accessors = [
        "pub const fn flow_axes(self) -> FlowAxes",
        "pub const fn used_overflow_x(self) -> Overflow",
        "pub const fn used_overflow_y(self) -> Overflow",
        "pub const fn border_box(self) -> ScrollRectOf<S>",
        "pub const fn padding_box(self) -> ScrollRectOf<S>",
        "pub const fn content_box(self) -> ScrollRectOf<S>",
        "pub const fn scrollport(self) -> ScrollRectOf<S>",
        "pub const fn overflow_clip(self) -> OverflowClipOf<S>",
        "pub const fn scrollable_overflow(self) -> ScrollRectOf<S>",
        "pub const fn physical_range(self) -> PhysicalScrollRangeOf<S>",
        "pub const fn gutters(self) -> ScrollbarGutterRectsOf<S>",
        "pub const fn scrollbar_size(self) -> Size<S>",
        "pub const fn resolved_scroll_padding(self) -> Edges<S>",
        "pub const fn optimal_viewing_region(self) -> ScrollRectOf<S>",
        "pub const fn scroll_snap_type(self) -> ScrollSnapType",
        "pub const fn target(self) -> ScrollTargetGeometryOf<S>",
    ];
    for accessor in geometry_accessors {
        assert!(
            geometry_impl.contains(accessor),
            "missing accessor: {accessor}"
        );
    }
    assert_eq!(
        geometry_impl.matches("pub const fn ").count(),
        geometry_accessors.len(),
        "canonical geometry has only the exact D-03 accessor set"
    );
    assert!(!geometry_impl.contains("pub fn "));

    let gutter_impl = scroll
        .split_once("pub struct ScrollbarGutterRectsOf")
        .expect("public gutter output declaration")
        .1
        .split_once("pub type ScrollbarGutterRects")
        .expect("gutter default-scalar alias")
        .1
        .split_once("pub(crate) struct ClipMarginSourceOf")
        .expect("gutter output implementation end")
        .0;
    let gutter_accessors = [
        "pub const fn top(self) -> Option<ScrollRectOf<S>>",
        "pub const fn right(self) -> Option<ScrollRectOf<S>>",
        "pub const fn bottom(self) -> Option<ScrollRectOf<S>>",
        "pub const fn left(self) -> Option<ScrollRectOf<S>>",
    ];
    for accessor in gutter_accessors {
        assert!(
            gutter_impl.contains(accessor),
            "missing accessor: {accessor}"
        );
    }
    assert_eq!(
        gutter_impl.matches("pub const fn ").count(),
        gutter_accessors.len(),
        "gutter output has only four physical-edge accessors"
    );
    assert!(!gutter_impl.contains("pub fn "));

    for public_name in [
        "ScrollGeometryOf",
        "ScrollGeometry",
        "ScrollbarGutterRectsOf",
        "ScrollbarGutterRects",
    ] {
        assert!(
            public_front_door.contains(public_name),
            "{public_name} is reexported"
        );
    }
}

#[test]
fn fri05_c03_legacy_surface_is_absent_from_public_source() {
    let scroll = include_str!("scroll.rs");
    let public_front_door = include_str!("lib.rs");
    let public_scroll_reexports = public_front_door
        .split_once("pub use scroll::{")
        .expect("public scroll reexports")
        .1
        .split_once("};")
        .expect("public scroll reexports end")
        .0;

    for removed_declaration in [
        "pub enum ScrollOverflowExposure",
        "pub struct ScrollContainerAxis",
        "pub struct ScrollContainerFacts",
        "pub fn scroll_container_facts_from_overflow",
        "pub enum ScrollUnsupportedFeature",
    ] {
        assert!(
            !scroll.contains(removed_declaration),
            "retained public legacy declaration: {removed_declaration}"
        );
    }
    for removed_reexport in [
        "ScrollOverflowExposure",
        "ScrollContainerAxis",
        "ScrollContainerFacts",
        "ScrollUnsupportedFeature",
    ] {
        assert!(
            !public_scroll_reexports.contains(removed_reexport),
            "retained legacy reexport: {removed_reexport}"
        );
    }

    let rect_impl = scroll
        .split_once("impl<S: LayoutScalar> ScrollRectOf<S> {")
        .expect("rectangle implementation")
        .1
        .split_once("/// A finite ordered physical clip interval.")
        .expect("rectangle implementation end")
        .0;
    assert!(!rect_impl.contains("pub fn new("));

    let geometry_impl = scroll
        .split_once("impl<S: LayoutScalar> ScrollGeometryOf<S> {")
        .expect("geometry implementation")
        .1
        .split_once("#[cfg(test)]\nmod fri05_c02_carrier_tests")
        .expect("production scroll source end")
        .0;
    assert!(!geometry_impl.contains("pub fn new("));
    assert!(!geometry_impl.contains("pub const fn container("));
}

#[test]
fn fri05_c03_root_block_legacy_absence_production_paths_and_bridge_accounting() {
    let scroll = include_str!("scroll.rs");
    let compute = include_str!("compute.rs");
    let block = include_str!("block.rs");
    let public_front_door = include_str!("lib.rs");
    let scroll_production = scroll
        .split("#[cfg(test)]\nmod fri05_c02_carrier_tests")
        .next()
        .expect("production scroll source");
    let root_block_production =
        format!("{scroll_production}\n{compute}\n{block}\n{public_front_door}");

    let forbidden = [
        "ScrollUnsupportedFeature",
        "ScrollRectOf::new",
        "ScrollGeometryOf::new",
        "ScrollContainerAxis",
        "ScrollContainerFacts",
        "scroll_geometry_from_layout",
        "scrollable_overflow_from_layout_content_size",
        "scrollable_overflow_from_content_size",
        "scroll_rect_union",
        "ScrollBoxRectsOf",
        "scroll_box_rects_from_border_box",
        "#[allow(dead_code)]",
        "#[allow(clippy::too_many_arguments)]",
    ]
    .into_iter()
    .filter(|symbol| root_block_production.contains(symbol))
    .collect::<Vec<_>>();
    assert!(
        forbidden.is_empty(),
        "root/block production retains legacy paths or their lint allowances: {forbidden:?}"
    );

    assert_eq!(
        block
            .matches("scrollbar_size: scroll_geometry.scrollbar_size()")
            .count(),
        0,
        "migrated block outputs synchronize through the canonical output helper"
    );

    let flex = include_str!("flex.rs");
    let grid_child = include_str!("grid/child.rs");
    let grid_lanes = include_str!("grid/lanes.rs");
    assert_eq!(
        flex.matches("scrollbar_size: item_scrollbar_size(").count(),
        0
    );
    assert_eq!(
        grid_child
            .matches("scrollbar_size: item.scrollbar_size")
            .count(),
        0
    );
    assert_eq!(
        grid_child
            .matches("scroll_geometry: Some(scroll_geometry),\n            scrollbar_size,\n")
            .count(),
        0
    );
    assert_eq!(
        grid_lanes
            .matches("scrollbar_size: item.scrollbar_size")
            .count(),
        0
    );
}

#[test]
fn fri05_c04_flex_bridge_accounting_accepts_grid_family_closure() {
    let flex = include_str!("flex.rs");
    let grid_child = include_str!("grid/child.rs");
    let grid_lanes = include_str!("grid/lanes.rs");
    let flex_bridges = flex.matches("scrollbar_size: item_scrollbar_size(").count();
    assert_eq!(
        flex_bridges, 0,
        "C04-T1 must remove both legacy flex direct writers; found {flex_bridges}"
    );

    let grid_item_bridges = grid_child
        .matches("scrollbar_size: item.scrollbar_size")
        .count();
    let grid_local_bridges = grid_child
        .matches("scroll_geometry: Some(scroll_geometry),\n            scrollbar_size,\n")
        .count();
    let lanes_bridges = grid_lanes
        .matches("scrollbar_size: item.scrollbar_size")
        .count();
    assert_eq!(grid_item_bridges, 0);
    assert_eq!(grid_local_bridges, 0);
    assert_eq!(lanes_bridges, 0);
    assert_eq!(grid_item_bridges + grid_local_bridges + lanes_bridges, 0);
}

#[test]
fn fri08_c03_public_removal_nested_lanes_unsupported_symbols_are_absent() {
    let lanes = include_str!("grid/lanes.rs");
    let grid = include_str!("grid/mod.rs");
    let public_front_door = include_str!("lib.rs");

    for removed_declaration in [
        "NestedIndefiniteSubgrid { span: LaneTrackSpanLength }",
        "pub const fn nested_indefinite_subgrid(",
        "NestedGridLanesSubgridIndefiniteUnsupported",
    ] {
        assert!(
            !lanes.contains(removed_declaration),
            "retained removed grid-lanes declaration: {removed_declaration}"
        );
    }
    for removed_use in [
        "LaneIntrinsicItemKind::NestedIndefiniteSubgrid",
        "LaneIntrinsicItemOf::nested_indefinite_subgrid",
        "LanePlacementError::NestedGridLanesSubgridIndefiniteUnsupported",
    ] {
        assert!(
            !lanes.contains(removed_use) && !grid.contains(removed_use),
            "retained removed grid-lanes production use: {removed_use}"
        );
    }
    assert!(
        public_front_door.contains("LaneIntrinsicItemKind"),
        "the retained definite/indefinite public item kind stays reexported"
    );
}

#[test]
fn fri05_c04_flex_round_cache_publication_has_one_canonical_geometry_path() {
    let flex = include_str!("flex.rs");

    assert!(
        !flex.contains("output.scroll_geometry = Some(scroll_geometry)"),
        "flex output must retain canonical geometry through one publication helper"
    );
    assert_eq!(
        flex.matches("retain_flex_scroll_geometry(").count(),
        3,
        "three aggregate/child publication sites must use the retention helper"
    );
    assert_eq!(
        flex.matches("fn retain_flex_scroll_geometry<").count(),
        1,
        "one flex geometry-retention helper must own publication"
    );
}

#[test]
fn fri05_c04_flex_legacy_absence_accepts_downstream_grid_closure() {
    let flex = include_str!("flex.rs");
    let forbidden = [
        "ScrollbarReservationOf",
        "content_box_inset_with_scrollbar",
        "scrollbar_size_from_overflow",
        "item_scrollbar_size",
        "visible_content_size",
        "content_size_contribution",
        "scrollbar_gutter_at_side",
        "scrollbar_gutter: Edges<S>",
        "effective_border: Edges<S>",
        "scroll_geometry: None,\n                scrollbar_size:",
    ]
    .into_iter()
    .filter(|symbol| flex.contains(symbol))
    .collect::<Vec<_>>();
    assert!(
        forbidden.is_empty(),
        "flex retains legacy reservation/projection/discard paths: {forbidden:?}"
    );

    let grid_child = include_str!("grid/child.rs");
    let grid_lanes = include_str!("grid/lanes.rs");
    let downstream_bridges = [
        (
            "ordinary-grid retained item bridge",
            grid_child
                .matches("scrollbar_size: item.scrollbar_size")
                .count(),
        ),
        (
            "ordinary-grid absolute scrollbar bridge",
            grid_child
                .matches("scroll_geometry: Some(scroll_geometry),\n            scrollbar_size,\n")
                .count(),
        ),
        (
            "grid-lanes retained item bridge",
            grid_lanes
                .matches("scrollbar_size: item.scrollbar_size")
                .count(),
        ),
    ];
    assert_eq!(
        downstream_bridges,
        [
            ("ordinary-grid retained item bridge", 0),
            ("ordinary-grid absolute scrollbar bridge", 0),
            ("grid-lanes retained item bridge", 0),
        ]
    );
    assert_eq!(
        downstream_bridges
            .into_iter()
            .map(|(_, count)| count)
            .sum::<usize>(),
        0
    );
}

#[test]
fn fri05_c05_grid_round_cache_has_no_independent_scrollbar_projection() {
    let output = include_str!("output.rs");
    let compute = include_str!("compute.rs");
    let cache = include_str!("cache.rs");

    assert!(
        !output.contains("pub scrollbar_size: Size<S>"),
        "rounded and cached output must not retain mutable scrollbar state outside canonical geometry"
    );
    assert!(
        !output.contains("self.scrollbar_size = geometry.scrollbar_size()"),
        "geometry publication must not synchronize an independent scrollbar projection"
    );
    assert!(
        !compute.contains("layout.scrollbar_size.width = round("),
        "rounding must rebuild scrollbar reservation only from canonical geometry sources"
    );
    assert!(
        !compute.contains("layout.scrollbar_size.height = round("),
        "rounding must rebuild scrollbar reservation only from canonical geometry sources"
    );
    assert!(
        !cache.contains("scrollbar_size"),
        "ordinary cache identity must carry only the canonical output value"
    );
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LegacySourceToken {
    text: String,
    offset: usize,
}

fn fri05_c05_lex_production_tokens(source: &str) -> Result<Vec<LegacySourceToken>, String> {
    fn quoted_end(bytes: &[u8], start: usize) -> Result<usize, String> {
        let mut index = start + 1;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' => index += 2,
                b'"' => return Ok(index + 1),
                _ => index += 1,
            }
        }
        Err(format!("unterminated quoted literal at byte {start}"))
    }

    fn identifier_start(character: char) -> bool {
        character == '_' || character.is_alphabetic()
    }

    fn identifier_continue(character: char) -> bool {
        identifier_start(character)
            || character.is_numeric()
            || matches!(
                character,
                '\u{0300}'..='\u{036f}'
                    | '\u{1ab0}'..='\u{1aff}'
                    | '\u{1dc0}'..='\u{1dff}'
                    | '\u{20d0}'..='\u{20ff}'
                    | '\u{fe20}'..='\u{fe2f}'
            )
    }

    fn malformed_character(start: usize, byte: bool, unterminated: bool) -> String {
        let state = if unterminated {
            "unterminated"
        } else {
            "malformed"
        };
        let kind = if byte {
            "byte character literal"
        } else {
            "character literal"
        };
        format!("{state} {kind} at byte {start}")
    }

    fn has_closing_quote(bytes: &[u8], start: usize) -> bool {
        bytes[start..]
            .iter()
            .take_while(|byte| !matches!(byte, b'\n' | b'\r'))
            .any(|byte| *byte == b'\'')
    }

    fn char_end(
        source: &str,
        bytes: &[u8],
        start: usize,
        byte: bool,
    ) -> Result<Option<usize>, String> {
        let value = start + 1;
        if value >= bytes.len() || matches!(bytes[value], b'\n' | b'\r') {
            return Err(malformed_character(start, byte, true));
        }

        if bytes[value] == b'\'' {
            return Err(malformed_character(start, byte, false));
        }

        let after_value = if bytes[value] == b'\\' {
            let escape = value + 1;
            let Some(kind) = bytes.get(escape).copied() else {
                return Err(malformed_character(start, byte, true));
            };
            match kind {
                b'n' | b'r' | b't' | b'\\' | b'0' | b'\'' | b'"' => escape + 1,
                b'x' if bytes
                    .get(escape + 1..escape + 3)
                    .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit)) =>
                {
                    escape + 3
                }
                b'u' if !byte && bytes.get(escape + 1) == Some(&b'{') => {
                    let mut index = escape + 2;
                    let mut digits = 0usize;
                    while let Some(character) = bytes.get(index).copied() {
                        match character {
                            b'}' => break,
                            b'_' => index += 1,
                            character if character.is_ascii_hexdigit() && digits < 6 => {
                                digits += 1;
                                index += 1;
                            }
                            _ => return Err(malformed_character(start, byte, false)),
                        }
                    }
                    if digits == 0 || bytes.get(index) != Some(&b'}') {
                        return Err(malformed_character(
                            start,
                            byte,
                            !has_closing_quote(bytes, value),
                        ));
                    }
                    index + 1
                }
                _ => return Err(malformed_character(start, byte, false)),
            }
        } else {
            let character = source[value..]
                .chars()
                .next()
                .expect("character literal value starts inside source");
            if byte && !character.is_ascii() {
                return Err(malformed_character(start, byte, false));
            }
            value + character.len_utf8()
        };

        if bytes.get(after_value) == Some(&b'\'') {
            return Ok(Some(after_value + 1));
        }

        if !byte && bytes[value] != b'\\' {
            let first = source[value..]
                .chars()
                .next()
                .expect("character literal value starts inside source");
            if identifier_start(first) {
                let mut lifetime_end = after_value;
                while let Some(character) = source[lifetime_end..].chars().next()
                    && identifier_continue(character)
                {
                    lifetime_end += character.len_utf8();
                }
                if bytes.get(lifetime_end) != Some(&b'\'') {
                    return Ok(None);
                }
            }
        }

        Err(malformed_character(
            start,
            byte,
            !has_closing_quote(bytes, after_value),
        ))
    }

    fn raw_end(bytes: &[u8], r_index: usize) -> Result<Option<usize>, String> {
        let mut quote = r_index + 1;
        while bytes.get(quote) == Some(&b'#') {
            quote += 1;
        }
        if bytes.get(quote) != Some(&b'"') {
            return Ok(None);
        }
        let hashes = quote - r_index - 1;
        let mut index = quote + 1;
        while index < bytes.len() {
            if bytes[index] == b'"'
                && bytes.get(index + 1..index + 1 + hashes) == Some(&bytes[r_index + 1..quote])
            {
                return Ok(Some(index + 1 + hashes));
            }
            index += 1;
        }
        Err(format!("unterminated raw string at byte {r_index}"))
    }

    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            index += 2;
            while index < bytes.len() && !matches!(bytes[index], b'\n' | b'\r') {
                index += 1;
            }
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            let start = index;
            let mut depth = 1usize;
            index += 2;
            while index < bytes.len() && depth != 0 {
                if bytes.get(index..index + 2) == Some(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes.get(index..index + 2) == Some(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            if depth != 0 {
                return Err(format!("unterminated block comment at byte {start}"));
            }
            continue;
        }

        let raw_prefix = if bytes[index] == b'r' {
            Some(index)
        } else if matches!(bytes[index], b'b' | b'c') && bytes.get(index + 1) == Some(&b'r') {
            Some(index + 1)
        } else {
            None
        };
        if let Some(r_index) = raw_prefix
            && let Some(end) = raw_end(bytes, r_index)?
        {
            index = end;
            continue;
        }

        if bytes[index] == b'"' {
            index = quoted_end(bytes, index)?;
            continue;
        }
        if matches!(bytes[index], b'b' | b'c') && bytes.get(index + 1) == Some(&b'"') {
            index = quoted_end(bytes, index + 1)?;
            continue;
        }
        if bytes[index] == b'\''
            && let Some(end) = char_end(source, bytes, index, false)?
        {
            index = end;
            continue;
        }
        if bytes[index] == b'b' && bytes.get(index + 1) == Some(&b'\'') {
            index = char_end(source, bytes, index + 1, true)?
                .expect("byte character prefix cannot be a lifetime");
            continue;
        }

        if bytes[index] == b'r'
            && bytes.get(index + 1) == Some(&b'#')
            && source
                .get(index + 2..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(identifier_start)
        {
            let start = index + 2;
            index = start;
            while let Some(character) = source[index..].chars().next()
                && identifier_continue(character)
            {
                index += character.len_utf8();
            }
            tokens.push(LegacySourceToken {
                text: source[start..index].to_owned(),
                offset: start,
            });
            continue;
        }
        let character = source[index..]
            .chars()
            .next()
            .expect("index is inside source");
        if identifier_start(character) {
            let start = index;
            index += character.len_utf8();
            while let Some(character) = source[index..].chars().next()
                && identifier_continue(character)
            {
                index += character.len_utf8();
            }
            tokens.push(LegacySourceToken {
                text: source[start..index].to_owned(),
                offset: start,
            });
            continue;
        }

        let start = index;
        index += character.len_utf8();
        tokens.push(LegacySourceToken {
            text: character.to_string(),
            offset: start,
        });
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum CfgTruth {
        False,
        True,
        Unknown,
    }

    fn matching_token(
        tokens: &[LegacySourceToken],
        start: usize,
        open: &str,
        close: &str,
        context: &str,
    ) -> Result<usize, String> {
        let mut depth = 0usize;
        for (index, token) in tokens.iter().enumerate().skip(start) {
            if token.text == open {
                depth += 1;
            } else if token.text == close {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(index);
                }
            }
        }
        Err(format!(
            "unclosed {context} at byte {}",
            tokens[start].offset
        ))
    }

    fn meta_arguments(
        tokens: &[LegacySourceToken],
        start: usize,
        end: usize,
        context: &str,
    ) -> Result<Vec<(usize, usize)>, String> {
        let mut arguments = Vec::new();
        let mut argument = start;
        let mut delimiters = Vec::new();
        for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
            match token.text.as_str() {
                "(" => delimiters.push(")"),
                "[" => delimiters.push("]"),
                "{" => delimiters.push("}"),
                ")" | "]" | "}" => {
                    if delimiters.pop() != Some(token.text.as_str()) {
                        return Err(format!(
                            "malformed {context} at byte {}",
                            tokens[index].offset
                        ));
                    }
                }
                "," if delimiters.is_empty() => {
                    arguments.push((argument, index));
                    argument = index + 1;
                }
                _ => {}
            }
        }
        if !delimiters.is_empty() {
            return Err(format!(
                "malformed {context} at byte {}",
                tokens[start.saturating_sub(1)].offset
            ));
        }
        arguments.push((argument, end));
        Ok(arguments)
    }

    fn cfg_truth(
        tokens: &[LegacySourceToken],
        start: usize,
        end: usize,
    ) -> Result<CfgTruth, String> {
        if start >= end {
            return Err("empty cfg predicate".to_owned());
        }
        if end == start + 1 {
            return Ok(if tokens[start].text == "test" {
                CfgTruth::False
            } else {
                CfgTruth::Unknown
            });
        }
        if tokens.get(start + 1).map(|token| token.text.as_str()) != Some("(") {
            return Ok(CfgTruth::Unknown);
        }
        let close = matching_token(tokens, start + 1, "(", ")", "cfg predicate")?;
        if close + 1 != end {
            return Err(format!(
                "malformed cfg predicate at byte {}",
                tokens[start].offset
            ));
        }
        let arguments = meta_arguments(tokens, start + 2, close, "cfg predicate")?;
        match tokens[start].text.as_str() {
            "not" => {
                if arguments.len() != 1 || arguments[0].0 == arguments[0].1 {
                    return Err(format!(
                        "malformed cfg not predicate at byte {}",
                        tokens[start].offset
                    ));
                }
                Ok(match cfg_truth(tokens, arguments[0].0, arguments[0].1)? {
                    CfgTruth::False => CfgTruth::True,
                    CfgTruth::True => CfgTruth::False,
                    CfgTruth::Unknown => CfgTruth::Unknown,
                })
            }
            "all" => {
                let mut result = CfgTruth::True;
                for (argument_start, argument_end) in arguments {
                    if argument_start == argument_end {
                        continue;
                    }
                    match cfg_truth(tokens, argument_start, argument_end)? {
                        CfgTruth::False => return Ok(CfgTruth::False),
                        CfgTruth::Unknown => result = CfgTruth::Unknown,
                        CfgTruth::True => {}
                    }
                }
                Ok(result)
            }
            "any" => {
                let mut result = CfgTruth::False;
                for (argument_start, argument_end) in arguments {
                    if argument_start == argument_end {
                        continue;
                    }
                    match cfg_truth(tokens, argument_start, argument_end)? {
                        CfgTruth::True => return Ok(CfgTruth::True),
                        CfgTruth::Unknown => result = CfgTruth::Unknown,
                        CfgTruth::False => {}
                    }
                }
                Ok(result)
            }
            _ => Ok(CfgTruth::Unknown),
        }
    }

    fn cfg_attribute_excludes(
        tokens: &[LegacySourceToken],
        start: usize,
        end: usize,
    ) -> Result<bool, String> {
        let Some(name) = tokens.get(start).map(|token| token.text.as_str()) else {
            return Err("empty outer attribute".to_owned());
        };
        if !matches!(name, "cfg" | "cfg_attr") {
            return Ok(false);
        }
        if tokens.get(start + 1).map(|token| token.text.as_str()) != Some("(") {
            return Err(format!(
                "malformed {name} attribute at byte {}",
                tokens[start].offset
            ));
        }
        let close = matching_token(tokens, start + 1, "(", ")", "cfg attribute")?;
        if close + 1 != end {
            return Err(format!(
                "malformed {name} attribute at byte {}",
                tokens[start].offset
            ));
        }
        let arguments = meta_arguments(tokens, start + 2, close, name)?;
        if name == "cfg" {
            if arguments.len() != 1 || arguments[0].0 == arguments[0].1 {
                return Err(format!(
                    "malformed cfg attribute at byte {}",
                    tokens[start].offset
                ));
            }
            return Ok(cfg_truth(tokens, arguments[0].0, arguments[0].1)? == CfgTruth::False);
        }
        if arguments.len() < 2 || arguments[0].0 == arguments[0].1 {
            return Err(format!(
                "malformed cfg_attr attribute at byte {}",
                tokens[start].offset
            ));
        }
        if cfg_truth(tokens, arguments[0].0, arguments[0].1)? != CfgTruth::True {
            return Ok(false);
        }
        for (argument_start, argument_end) in arguments.into_iter().skip(1) {
            if argument_start == argument_end {
                return Err(format!(
                    "malformed cfg_attr attribute at byte {}",
                    tokens[start].offset
                ));
            }
            if cfg_attribute_excludes(tokens, argument_start, argument_end)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn item_end(tokens: &[LegacySourceToken], start: usize) -> Result<usize, String> {
        let mut keyword = start;
        loop {
            match tokens.get(keyword).map(|token| token.text.as_str()) {
                Some("pub") => {
                    keyword += 1;
                    if tokens.get(keyword).map(|token| token.text.as_str()) == Some("(") {
                        keyword = matching_token(tokens, keyword, "(", ")", "visibility")? + 1;
                    }
                }
                Some("async" | "unsafe" | "default" | "extern") => keyword += 1,
                Some("const")
                    if tokens.get(keyword + 1).map(|token| token.text.as_str()) == Some("fn") =>
                {
                    keyword += 1;
                }
                _ => break,
            }
        }
        let kind = tokens.get(keyword).map(|token| token.text.as_str());
        let semicolon_item = matches!(kind, Some("const" | "static" | "type"));
        let recognized = matches!(
            kind,
            Some(
                "const"
                    | "enum"
                    | "fn"
                    | "impl"
                    | "mod"
                    | "static"
                    | "struct"
                    | "trait"
                    | "type"
                    | "union"
            )
        );
        let mut delimiters = Vec::new();
        for index in start..tokens.len() {
            match tokens[index].text.as_str() {
                "(" => delimiters.push(")"),
                "[" => delimiters.push("]"),
                "{" if semicolon_item => delimiters.push("}"),
                "{" if delimiters.is_empty() => {
                    return Ok(matching_token(tokens, index, "{", "}", "attributed item")? + 1);
                }
                "{" => delimiters.push("}"),
                ")" | "]" | "}" => {
                    if delimiters.pop() != Some(tokens[index].text.as_str()) {
                        return Err(format!(
                            "malformed attributed item at byte {}",
                            tokens[start].offset
                        ));
                    }
                }
                ";" if delimiters.is_empty() => return Ok(index + 1),
                "," if delimiters.is_empty() && !recognized => return Ok(index + 1),
                _ => {}
            }
        }
        Err(format!(
            "outer attribute has no complete item at byte {}",
            tokens[start].offset
        ))
    }

    let mut omitted = vec![false; tokens.len()];
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].text != "#"
            || tokens.get(index + 1).map(|token| token.text.as_str()) != Some("[")
        {
            index += 1;
            continue;
        }
        let attributes_start = index;
        let mut attributes_end = index;
        let mut excludes = false;
        while tokens.get(attributes_end).map(|token| token.text.as_str()) == Some("#")
            && tokens
                .get(attributes_end + 1)
                .map(|token| token.text.as_str())
                == Some("[")
        {
            let close = matching_token(&tokens, attributes_end + 1, "[", "]", "outer attribute")
                .map_err(|_| {
                    format!(
                        "unclosed outer attribute at byte {}",
                        tokens[attributes_end].offset
                    )
                })?;
            excludes |= cfg_attribute_excludes(&tokens, attributes_end + 2, close)?;
            attributes_end = close + 1;
        }
        if excludes {
            let end = item_end(&tokens, attributes_end)?;
            omitted[attributes_start..end].fill(true);
            index = end;
        } else {
            index = attributes_end;
        }
    }

    Ok(tokens
        .into_iter()
        .zip(omitted)
        .filter_map(|(token, omitted)| (!omitted).then_some(token))
        .collect())
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct LegacyScrollbarAccounting {
    inline_carrier_fields: usize,
    inline_carrier_writers: usize,
    inline_carrier_projections: usize,
    block_carrier_writers: usize,
    output_accessors: usize,
    output_projections: usize,
    geometry_accessors: usize,
    node_output_structs: usize,
    node_output_scroll_geometry_fields: usize,
    node_output_aliases: usize,
    node_output_impls: usize,
}

fn fri05_c05_audit_legacy_source(
    path: &str,
    source: &str,
) -> Result<LegacyScrollbarAccounting, String> {
    fn has(tokens: &[LegacySourceToken], index: usize, pattern: &[&str]) -> bool {
        pattern.iter().enumerate().all(|(offset, expected)| {
            tokens.get(index + offset).map(|token| token.text.as_str()) == Some(*expected)
        })
    }

    fn owner_before_brace(tokens: &[LegacySourceToken], brace: usize) -> Option<&str> {
        let mut index = brace.checked_sub(1)?;
        if tokens[index].text == ">" {
            let mut depth = 1usize;
            while index > 0 && depth != 0 {
                index -= 1;
                match tokens[index].text.as_str() {
                    ">" => depth += 1,
                    "<" => depth -= 1,
                    _ => {}
                }
            }
            index = index.checked_sub(1)?;
        }
        tokens.get(index).map(|token| token.text.as_str())
    }

    fn enclosing_owner(tokens: &[LegacySourceToken], target: usize) -> Option<&str> {
        let mut stack = Vec::new();
        for (index, token) in tokens.iter().enumerate().take(target) {
            match token.text.as_str() {
                "{" => stack.push(index),
                "}" => {
                    stack.pop();
                }
                _ => {}
            }
        }
        stack
            .into_iter()
            .rev()
            .find_map(|brace| owner_before_brace(tokens, brace))
    }

    fn enclosing_function(tokens: &[LegacySourceToken], target: usize) -> Option<&str> {
        let mut stack = Vec::new();
        for (index, token) in tokens.iter().enumerate().take(target) {
            match token.text.as_str() {
                "{" => stack.push(index),
                "}" => {
                    stack.pop();
                }
                _ => {}
            }
        }
        for brace in stack.into_iter().rev() {
            let mut index = brace;
            while index > 0 {
                index -= 1;
                match tokens[index].text.as_str() {
                    "fn" => return tokens.get(index + 1).map(|token| token.text.as_str()),
                    "{" | "}" | ";" => break,
                    _ => {}
                }
            }
        }
        None
    }

    fn declaration_end(tokens: &[LegacySourceToken], start: usize) -> usize {
        let mut angle = 0usize;
        let mut index = start;
        while index < tokens.len() {
            match tokens[index].text.as_str() {
                "<" => angle += 1,
                ">" => angle = angle.saturating_sub(1),
                ";" | "{" if angle == 0 => return index,
                _ => {}
            }
            index += 1;
        }
        tokens.len()
    }

    let tokens = fri05_c05_lex_production_tokens(source)
        .map_err(|error| format!("{path}: lexical error: {error}"))?;
    let mut accounting = LegacyScrollbarAccounting::default();

    for index in 0..tokens.len() {
        let token = tokens[index].text.as_str();
        if matches!(token, "struct" | "enum" | "union")
            && tokens.get(index + 1).map(|token| token.text.as_str()) == Some("NodeOutputOf")
        {
            if path != "src/output.rs" || token != "struct" {
                return Err(format!(
                    "{path}: shadow NodeOutputOf {token} declaration at byte {}",
                    tokens[index].offset
                ));
            }
            accounting.node_output_structs += 1;
        }
        if token == "type" {
            let end = declaration_end(&tokens, index + 1);
            if tokens[index + 1..end]
                .iter()
                .any(|token| token.text == "NodeOutputOf")
            {
                if path != "src/output.rs"
                    || tokens.get(index + 1).map(|token| token.text.as_str()) != Some("NodeOutput")
                {
                    return Err(format!(
                        "{path}: NodeOutputOf compatibility alias at byte {}",
                        tokens[index].offset
                    ));
                }
                accounting.node_output_aliases += 1;
            }
        }
        if token == "impl" {
            let end = declaration_end(&tokens, index + 1);
            if tokens[index + 1..end]
                .iter()
                .any(|token| token.text == "NodeOutputOf")
            {
                if path != "src/output.rs" {
                    return Err(format!(
                        "{path}: NodeOutputOf compatibility impl at byte {}",
                        tokens[index].offset
                    ));
                }
                accounting.node_output_impls += 1;
            }
        }
        if path == "src/output.rs"
            && token == "scroll_geometry"
            && index > 0
            && has(
                &tokens,
                index - 1,
                &[
                    "pub",
                    "scroll_geometry",
                    ":",
                    "Option",
                    "<",
                    "ScrollGeometryOf",
                    "<",
                    "S",
                    ">",
                    ">",
                ],
            )
            && enclosing_owner(&tokens, index) == Some("NodeOutputOf")
        {
            accounting.node_output_scroll_geometry_fields += 1;
        }

        if token != "scrollbar_size" {
            continue;
        }
        let owner = enclosing_owner(&tokens, index);
        let previous = index.checked_sub(1);
        let allowed = if path == "src/inline.rs"
            && previous.is_some_and(|previous| {
                has(
                    &tokens,
                    previous,
                    &["pub", "scrollbar_size", ":", "Size", "<", "S", ">"],
                )
            })
            && matches!(
                owner,
                Some("AtomicInlineBoxParticipant" | "InlineParticipantLayoutItem")
            ) {
            accounting.inline_carrier_fields += 1;
            true
        } else if path == "src/inline.rs"
            && owner == Some("InlineParticipantLayoutItem")
            && (has(
                &tokens,
                index,
                &["scrollbar_size", ":", "Size", ":", ":", "ZERO"],
            ) || has(
                &tokens,
                index,
                &["scrollbar_size", ":", "item", ".", "scrollbar_size"],
            ))
        {
            accounting.inline_carrier_writers += 1;
            true
        } else if path == "src/inline.rs"
            && previous.is_some_and(|previous| has(&tokens, previous, &[".", "scrollbar_size"]))
            && index >= 2
            && tokens[index - 2].text == "item"
            && owner == Some("InlineParticipantLayoutItem")
        {
            accounting.inline_carrier_projections += 1;
            true
        } else if path == "src/block.rs"
            && has(
                &tokens,
                index,
                &[
                    "scrollbar_size",
                    ":",
                    "child_scrollbar_size",
                    "(",
                    "&",
                    "child_style",
                    ")",
                ],
            )
            && owner == Some("AtomicInlineBoxParticipant")
        {
            accounting.block_carrier_writers += 1;
            true
        } else if path == "src/output.rs"
            && previous.is_some_and(|previous| {
                has(
                    &tokens,
                    previous,
                    &[
                        "fn",
                        "scrollbar_size",
                        "(",
                        "self",
                        ")",
                        "-",
                        ">",
                        "Size",
                        "<",
                        "S",
                        ">",
                    ],
                )
            })
            && owner == Some("NodeOutputOf")
        {
            accounting.output_accessors += 1;
            true
        } else if path == "src/output.rs"
            && index >= 2
            && has(
                &tokens,
                index - 2,
                &["geometry", ".", "scrollbar_size", "(", ")"],
            )
            && enclosing_function(&tokens, index) == Some("scrollbar_size")
        {
            accounting.output_projections += 1;
            true
        } else if path == "src/scroll.rs"
            && previous.is_some_and(|previous| {
                has(
                    &tokens,
                    previous,
                    &[
                        "fn",
                        "scrollbar_size",
                        "(",
                        "self",
                        ")",
                        "-",
                        ">",
                        "Size",
                        "<",
                        "S",
                        ">",
                    ],
                )
            })
            && owner == Some("ScrollGeometryOf")
        {
            accounting.geometry_accessors += 1;
            true
        } else {
            false
        };
        if !allowed {
            return Err(format!(
                "{path}: forbidden scrollbar_size token at byte {} in owner {owner:?}",
                tokens[index].offset
            ));
        }
    }

    Ok(accounting)
}

#[test]
fn fri05_c05_grid_legacy_absence_lexer_fails_closed_at_rust_token_boundaries() {
    for (source, expected) in [
        (
            "const BAD: char = 'ab'; scrollbar_size",
            "src/cache.rs: lexical error: malformed character literal at byte 18",
        ),
        (
            "const BAD: char = '\\\\",
            "src/cache.rs: lexical error: unterminated character literal at byte 18",
        ),
        (
            "const BAD: u8 = b'ab'; scrollbar_size",
            "src/cache.rs: lexical error: malformed byte character literal at byte 17",
        ),
        (
            "const BAD: u8 = b'\\\\",
            "src/cache.rs: lexical error: unterminated byte character literal at byte 17",
        ),
        (
            "const BAD: &str = \"scrollbar_size",
            "src/cache.rs: lexical error: unterminated quoted literal at byte 18",
        ),
        (
            "const BAD: &str = r##\"scrollbar_size\"#;",
            "src/cache.rs: lexical error: unterminated raw string at byte 18",
        ),
        (
            "fn clean() {} /* scrollbar_size",
            "src/cache.rs: lexical error: unterminated block comment at byte 14",
        ),
    ] {
        assert_eq!(
            fri05_c05_audit_legacy_source("src/cache.rs", source),
            Err(expected.to_owned()),
            "malformed literals and comments fail closed without exposing their contents"
        );
    }

    let literal_tokens =
        fri05_c05_lex_production_tokens("'\\n' '\\u{1f980}' '\u{00e9}' b'\\n' b'\\x7f'")
            .expect("valid escaped, Unicode, and byte character literals lex");
    assert!(
        literal_tokens.is_empty(),
        "valid character literal contents stay ignored"
    );

    let lifetime_tokens = fri05_c05_lex_production_tokens(
        "fn borrow<'a>(value: &'a str) -> &'_ str { 'retry: loop { break 'retry value; } }",
    )
    .expect("lifetimes and labels lex");
    assert_eq!(
        lifetime_tokens
            .iter()
            .filter(|token| matches!(token.text.as_str(), "a" | "_" | "retry"))
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>(),
        ["a", "a", "_", "retry", "retry"],
        "lifetimes and labels remain identifiers rather than character literals"
    );

    let unicode_tokens = fri05_c05_lex_production_tokens(
        "let scrollbar_size\u{03bb} = 0; let \u{03bb}scrollbar_size = 0; let cafe\u{0301} = 0;",
    )
    .expect("valid Unicode identifier boundaries lex");
    assert!(
        unicode_tokens
            .iter()
            .all(|token| token.text != "scrollbar_size"),
        "a forbidden spelling embedded in a valid Unicode identifier is not a standalone token"
    );
}

#[test]
fn fri05_c05_grid_legacy_absence_cfg_attr_omits_exactly_one_test_only_item() {
    for hidden in [
        "#[cfg_attr(not(test), cfg(test))] mod hidden { fn f() { scrollbar_size; } }",
        "#[cfg_attr(any(not(test), test), cfg(all(test)))] fn hidden() { scrollbar_size; }",
        "#[cfg_attr(all(not(test), any(not(test), test)), cfg(not(not(test))))] impl Hidden { fn f() { scrollbar_size; } }",
        "#[cfg_attr(not(test), cfg(test))] const HIDDEN: () = { scrollbar_size; };",
        "#[cfg_attr(not(test), cfg(test))] static HIDDEN: usize = scrollbar_size;",
        "#[cfg_attr(not(test), cfg(test))] type Hidden = scrollbar_size;",
        "#[allow(dead_code)] /* between attributes */ #[cfg_attr(not(test), cfg(test))] pub(crate) fn hidden() { scrollbar_size; }",
        "#[cfg_attr(not(test), cfg_attr(not(test), cfg(test)))] fn hidden() { scrollbar_size; }",
    ] {
        assert_eq!(
            fri05_c05_audit_legacy_source("src/cache.rs", hidden),
            Ok(LegacyScrollbarAccounting::default()),
            "cfg_attr forms that deterministically exclude an item from production are omitted"
        );
    }

    for production in [
        "#[cfg_attr(test, cfg(test))] fn production() { scrollbar_size; }",
        "#[cfg_attr(all(not(test), any()), cfg(test))] mod production { fn f() { scrollbar_size; } }",
        "#[cfg_attr(unix, cfg(test))] const PRODUCTION_SOMEWHERE: usize = scrollbar_size;",
    ] {
        assert!(
            fri05_c05_audit_legacy_source("src/cache.rs", production).is_err(),
            "cfg_attr that can leave an item in production must not hide it"
        );
    }

    assert_eq!(
        fri05_c05_audit_legacy_source(
            "src/cache.rs",
            "#[cfg_attr(not(test), cfg(test))] const HIDDEN: () = { scrollbar_size; }; fn production() { scrollbar_size; }",
        ),
        Err(
            "src/cache.rs: forbidden scrollbar_size token at byte 92 in owner Some(\")\")"
                .to_owned()
        ),
        "only the attributed item is omitted and the following production item is audited"
    );
    assert_eq!(
        fri05_c05_audit_legacy_source(
            "src/cache.rs",
            "#[cfg_attr(not(test), cfg(test)) fn hidden() { scrollbar_size; }",
        ),
        Err("src/cache.rs: lexical error: unclosed outer attribute at byte 0".to_owned()),
        "malformed outer attributes fail closed"
    );
}

#[test]
fn fri05_c05_grid_legacy_absence_inventories_every_production_source() {
    for ignored in [
        "// scrollbar_size\nfn clean() {}",
        "/* outer scrollbar_size /* nested scrollbar_size */ still ignored */ fn clean() {}",
        "const NORMAL: &str = \"scrollbar_size\";",
        "const RAW: &str = r###\"scrollbar_size\"###;",
        "const BYTE: &[u8] = b\"scrollbar_size\";",
        "const RAW_BYTE: &[u8] = br##\"scrollbar_size\"##;",
        "const CHARACTER: char = 's'; const BYTE_CHARACTER: u8 = b's'; // scrollbar_size",
    ] {
        assert_eq!(
            fri05_c05_audit_legacy_source("src/cache.rs", ignored),
            Ok(LegacyScrollbarAccounting::default()),
            "comments and string/character literal contents are not source tokens"
        );
    }
    assert!(
        fri05_c05_lex_production_tokens("'s' b's'")
            .expect("character literals lex")
            .is_empty(),
        "character and byte-character contents do not become tokens"
    );
    assert_eq!(
        fri05_c05_lex_production_tokens("scrollbar_/* ignored */size")
            .expect("comment boundary lexes")
            .into_iter()
            .map(|token| token.text)
            .collect::<Vec<_>>(),
        ["scrollbar_", "size"],
        "removing ignored text preserves identifier token boundaries"
    );
    for forbidden in [
        "struct CompatibilityCarrier < S > { pub scrollbar_size : Size < S > }",
        "struct NodeOutputOf < S > { scrollbar_size : Size < S > }",
        "type Shadow < S > = NodeOutputOf < S >;",
        "fn write() { self . scrollbar_size\n= geometry . scrollbar_size ( ) ; }",
        "struct CompatibilityCarrier < S > { r#scrollbar_size : Size < S > }",
    ] {
        assert!(
            fri05_c05_audit_legacy_source("src/cache.rs", forbidden).is_err(),
            "token-equivalent forbidden fields, aliases, and writers are rejected"
        );
    }

    let sources = [
        ("src/block.rs", include_str!("block.rs")),
        ("src/cache.rs", include_str!("cache.rs")),
        ("src/compute.rs", include_str!("compute.rs")),
        (
            "src/engine/contracts.rs",
            include_str!("engine/contracts.rs"),
        ),
        ("src/engine/mod.rs", include_str!("engine/mod.rs")),
        (
            "src/engine/validation.rs",
            include_str!("engine/validation.rs"),
        ),
        ("src/error.rs", include_str!("error.rs")),
        ("src/flex.rs", include_str!("flex.rs")),
        ("src/geometry.rs", include_str!("geometry.rs")),
        ("src/grid/alignment.rs", include_str!("grid/alignment.rs")),
        ("src/grid/axis.rs", include_str!("grid/axis.rs")),
        ("src/grid/child.rs", include_str!("grid/child.rs")),
        ("src/grid/lanes.rs", include_str!("grid/lanes.rs")),
        ("src/grid/mod.rs", include_str!("grid/mod.rs")),
        ("src/grid/named.rs", include_str!("grid/named.rs")),
        ("src/grid/placement.rs", include_str!("grid/placement.rs")),
        ("src/grid/subgrid.rs", include_str!("grid/subgrid.rs")),
        ("src/grid/topology.rs", include_str!("grid/topology.rs")),
        ("src/grid/tracks.rs", include_str!("grid/tracks.rs")),
        ("src/inline.rs", include_str!("inline.rs")),
        ("src/layout_math.rs", include_str!("layout_math.rs")),
        ("src/lib.rs", include_str!("lib.rs")),
        ("src/measurement.rs", include_str!("measurement.rs")),
        ("src/node_input.rs", include_str!("node_input.rs")),
        ("src/output.rs", include_str!("output.rs")),
        ("src/scalar.rs", include_str!("scalar.rs")),
        ("src/scroll.rs", include_str!("scroll.rs")),
        ("src/sizing.rs", include_str!("sizing.rs")),
        ("src/sizing/resolve.rs", include_str!("sizing/resolve.rs")),
        (
            "src/test_support/scroll_geometry.rs",
            include_str!("test_support/scroll_geometry.rs"),
        ),
        ("src/tree.rs", include_str!("tree.rs")),
        ("src/value.rs", include_str!("value.rs")),
    ];

    fn production_rust_sources(directory: &std::path::Path, paths: &mut Vec<String>) {
        for entry in std::fs::read_dir(directory).expect("production source directory is readable")
        {
            let path = entry.expect("production source entry is readable").path();
            if path.is_dir() {
                if path.file_name().and_then(|name| name.to_str()) != Some("test_support") {
                    production_rust_sources(&path, paths);
                }
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs")
                && !path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with("_tests.rs"))
            {
                paths.push(
                    path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
                        .expect("production source is inside the repository")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }

    let mut inventoried = Vec::new();
    production_rust_sources(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut inventoried,
    );
    inventoried.push("src/test_support/scroll_geometry.rs".to_owned());
    inventoried.sort();
    let declared = sources.iter().map(|(path, ..)| *path).collect::<Vec<_>>();
    assert_eq!(
        inventoried, declared,
        "every production source and shared scroll fixture must have an exact scrollbar classification"
    );

    let mut observed = Vec::new();
    for (path, source) in sources {
        observed.push((
            path,
            fri05_c05_audit_legacy_source(path, source).unwrap_or_else(|error| panic!("{error}")),
        ));
    }
    assert_eq!(
        observed
            .iter()
            .filter(|(_, accounting)| *accounting != LegacyScrollbarAccounting::default())
            .collect::<Vec<_>>(),
        vec![
            &(
                "src/block.rs",
                LegacyScrollbarAccounting {
                    block_carrier_writers: 1,
                    ..LegacyScrollbarAccounting::default()
                }
            ),
            &(
                "src/inline.rs",
                LegacyScrollbarAccounting {
                    inline_carrier_fields: 1,
                    ..LegacyScrollbarAccounting::default()
                }
            ),
            &(
                "src/output.rs",
                LegacyScrollbarAccounting {
                    output_accessors: 1,
                    output_projections: 1,
                    node_output_structs: 1,
                    node_output_scroll_geometry_fields: 1,
                    node_output_aliases: 1,
                    node_output_impls: 2,
                    ..LegacyScrollbarAccounting::default()
                }
            ),
            &(
                "src/scroll.rs",
                LegacyScrollbarAccounting {
                    geometry_accessors: 1,
                    ..LegacyScrollbarAccounting::default()
                }
            ),
        ],
        "only the exact private carriers and canonical geometry projections are allowed"
    );
}

#[test]
fn fri08_remediation_engine_contract_is_algorithm_neutral() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let shared_contract = std::fs::read_to_string(manifest_dir.join("src/engine/contracts.rs"))
        .or_else(|_| std::fs::read_to_string(manifest_dir.join("src/traits.rs")))
        .unwrap_or_else(|error| panic!("shared recursive contract is unreadable: {error}"));

    for forbidden in [
        "InheritedFloatExclusions",
        "crate::block",
        "compute_block_with_inherited_float_exclusions",
    ] {
        assert!(
            !shared_contract.contains(forbidden),
            "the shared recursive engine contract retains algorithm-specific dependency or dispatch: {forbidden}"
        );
    }

    assert!(manifest_dir.join("src/engine/contracts.rs").is_file());
    assert!(!manifest_dir.join("src/traits.rs").exists());
    let tree = std::fs::read_to_string(manifest_dir.join("src/tree.rs"))
        .unwrap_or_else(|error| panic!("public tree contract owner is unreadable: {error}"));
    let public_front_door = include_str!("lib.rs");
    assert!(
        public_front_door.contains("pub use tree::{LayoutBatchSink, LayoutTree, Traverse};"),
        "the crate root must preserve the exact public host-trait reexport inventory"
    );
    assert!(!public_front_door.contains("pub mod tree"));
    assert!(!public_front_door.contains("pub mod engine"));
    for declaration in [
        "pub trait Traverse",
        "pub trait LayoutTree",
        "pub trait LayoutBatchSink",
    ] {
        assert!(
            tree.contains(declaration),
            "src/tree.rs must own {declaration}"
        );
    }
    for declaration in [
        "pub(crate) trait Compute",
        "pub(crate) trait Round",
        "pub(crate) trait CacheAccess",
        "pub(crate) enum UnroundedInlineFragmentState",
        "pub(crate) fn compute_cached",
    ] {
        assert!(
            shared_contract.contains(declaration),
            "src/engine/contracts.rs must own {declaration}"
        );
    }
}

#[test]
fn fri08_remediation_engine_validation_has_one_owner() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let compute = include_str!("compute.rs");
    let engine = include_str!("engine/mod.rs");
    let validation =
        std::fs::read_to_string(manifest_dir.join("src/engine/validation.rs")).unwrap_or_default();

    for declaration in [
        "fn invalidation_closure",
        "fn validate_layout_tree",
        "fn non_box_node_role_error",
        "fn root_input_error",
    ] {
        assert!(
            !compute.contains(declaration),
            "src/compute.rs retains validation or invalidation declaration: {declaration}"
        );
        assert!(
            validation.contains(declaration),
            "src/engine/validation.rs must own validation or invalidation declaration: {declaration}"
        );
    }

    assert!(engine.contains("mod validation;"));
    assert!(!engine.contains("pub mod validation;"));
    assert!(validation.contains("pub(crate) fn validate_layout_request"));
    assert!(validation.contains("LayoutUnsupportedCapability::LaterFriBehavior"));

    let validation_call = compute
        .find("engine::validate_layout_request")
        .unwrap_or_else(|| panic!("public orchestration calls the validation owner"));
    let session_creation = compute.find("ComputeSession::new").unwrap_or_else(|| {
        panic!("public orchestration still creates the staged session during T01")
    });
    assert!(
        validation_call < session_creation,
        "public orchestration must validate before creating session state"
    );
}

#[test]
fn fri08_remediation_measurement_has_one_owner() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let compute = include_str!("compute.rs");
    let error = include_str!("error.rs");
    let scroll = include_str!("scroll.rs");
    let measurement =
        std::fs::read_to_string(manifest_dir.join("src/measurement.rs")).unwrap_or_default();

    for declaration in [
        "pub struct LeafMeasureInputOf",
        "pub enum MeasurementAvailableOf",
        "struct LeafResolvedValues",
        "pub fn compute_leaf",
        "fn compute_tree_leaf",
        "fn compute_leaf_with_resolved_values",
        "fn leaf_pass_input",
        "fn validate_measurement_output",
        "fn leaf_measurement_error_at_site",
    ] {
        assert!(
            !compute.contains(declaration),
            "src/compute.rs retains leaf measurement declaration: {declaration}"
        );
        assert!(
            measurement.contains(declaration),
            "src/measurement.rs must own leaf measurement declaration: {declaration}"
        );
    }

    for declaration in ["pub enum LeafMeasureErrorOf", "pub type LeafMeasureError"] {
        assert!(
            !error.contains(declaration),
            "src/error.rs retains standalone leaf measurement error: {declaration}"
        );
        assert!(
            measurement.contains(declaration),
            "src/measurement.rs must own standalone leaf measurement error: {declaration}"
        );
    }

    assert!(include_str!("lib.rs").contains("mod measurement;"));
    assert!(include_str!("tree.rs").contains("crate::measurement::LeafMeasureInputOf"));
    assert!(!compute.contains("pub(crate) fn from_scroll_padding"));
    assert_eq!(
        scroll.matches("pub(crate) fn from_scroll_padding").count(),
        1,
        "the scroll-padding constructor must have one scroll owner"
    );
}

#[test]
fn fri08_remediation_sizing_resolution_has_one_owner() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let compute = include_str!("compute.rs");
    let sizing = include_str!("sizing.rs");
    let error = include_str!("error.rs");
    let resolver =
        std::fs::read_to_string(manifest_dir.join("src/sizing/resolve.rs")).unwrap_or_default();
    let non_owners = [
        ("src/compute.rs", compute),
        ("src/error.rs", error),
        ("src/block.rs", include_str!("block.rs")),
        ("src/flex.rs", include_str!("flex.rs")),
        ("src/grid/mod.rs", include_str!("grid/mod.rs")),
        ("src/grid/child.rs", include_str!("grid/child.rs")),
        ("src/grid/lanes.rs", include_str!("grid/lanes.rs")),
    ];

    let resolver_declarations = [
        "pub(crate) enum ResolvedPreferredSize",
        "pub(crate) enum ResolvedFlexBasis",
        "fn percentage_basis",
        "fn resolve_dispatched_numeric",
        "pub(crate) fn resolve_preferred_sizing",
        "pub(crate) fn resolve_preferred_optional",
        "pub(crate) fn resolve_minimum_optional",
        "pub(crate) fn resolve_maximum_optional",
        "pub(crate) fn resolve_flex_basis",
        "pub(crate) enum SizingResolutionError",
        "pub(crate) fn resolve_length_or_zero_fallible",
        "pub(crate) fn resolve_auto_or_zero_fallible",
        "pub(crate) fn resolution_or_zero_fallible",
        "pub(crate) fn resolution_optional_fallible",
        "pub(crate) trait SizeResultExt",
        "pub(crate) trait EdgesResultExt",
    ];
    for declaration in resolver_declarations {
        for (path, source) in non_owners {
            assert!(
                !source.contains(declaration),
                "{path} retains shared sizing resolver declaration: {declaration}"
            );
        }
        assert!(
            resolver.contains(declaration),
            "src/sizing/resolve.rs must own shared sizing resolver declaration: {declaration}"
        );
    }

    assert!(sizing.contains("pub(crate) mod resolve;"));
    assert!(!sizing.contains("pub mod resolve;"));
    assert!(!include_str!("lib.rs").contains("pub mod sizing"));
    for (path, source) in [
        ("src/block.rs", include_str!("block.rs")),
        ("src/flex.rs", include_str!("flex.rs")),
        ("src/grid/mod.rs", include_str!("grid/mod.rs")),
        ("src/grid/child.rs", include_str!("grid/child.rs")),
        ("src/grid/lanes.rs", include_str!("grid/lanes.rs")),
    ] {
        assert!(
            source.contains("crate::sizing::resolve"),
            "{path} must consume the shared sizing resolver owner directly"
        );
    }
}

#[test]
fn fri08_remediation_public_api_inventory_is_compatible() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let error = std::fs::read_to_string(manifest_dir.join("src/error.rs")).unwrap_or_default();
    let measurement =
        std::fs::read_to_string(manifest_dir.join("src/measurement.rs")).unwrap_or_default();
    let compute = include_str!("compute.rs");
    let public_front_door = include_str!("lib.rs");

    let public_error_declarations = [
        "pub type LayoutResultOf",
        "pub type LayoutResult",
        "pub struct LayoutErrorOf",
        "pub type LayoutError",
        "pub enum LayoutErrorSiteOf",
        "pub type LayoutErrorSite",
        "pub enum LayoutOperation",
        "pub enum LayoutErrorKindOf",
        "pub type LayoutErrorKind",
        "pub enum LayoutInvalidInputOf",
        "pub type LayoutInvalidInput",
        "pub enum AtomicInlineParticipationRoleError",
        "pub enum NonBoxNodeRoleError",
        "pub enum FloatExclusionRoleError",
        "pub enum LayoutMissingContext",
        "pub enum SizingProperty",
        "pub enum SizingAlgorithm",
        "pub enum CalcSizeBehaviorBasis",
        "pub enum SizingBehavior",
        "pub struct UnsupportedSizingBehavior",
        "pub enum LayoutUnsupportedCapability",
        "pub enum LayoutInternalInvariant",
        "pub struct InvalidMeasurementOutputOf",
        "pub type InvalidMeasurementOutput",
    ];
    for declaration in public_error_declarations {
        assert!(
            error.contains(declaration),
            "src/error.rs must own {declaration}"
        );
        assert!(
            !compute.contains(declaration),
            "src/compute.rs must not retain {declaration}"
        );
    }

    for declaration in [
        "pub enum LeafMeasureErrorOf",
        "pub type LeafMeasureError",
        "pub struct LeafMeasureInputOf",
        "pub type LeafMeasureInput",
        "pub enum MeasurementAvailableOf",
        "pub type MeasurementAvailable",
        "pub fn compute_leaf",
    ] {
        assert!(
            measurement.contains(declaration),
            "src/measurement.rs must own {declaration}"
        );
        assert!(
            !compute.contains(declaration),
            "src/compute.rs must not retain {declaration}"
        );
    }

    fn reexported_names<'a>(source: &'a str, owner: &str) -> Vec<&'a str> {
        source
            .split_once(&format!("pub use {owner}::{{"))
            .unwrap_or_else(|| panic!("the crate front door reexports the private {owner} owner"))
            .1
            .split_once("};")
            .unwrap_or_else(|| panic!("the {owner} reexport declaration is complete"))
            .0
            .split(|character: char| !(character.is_alphanumeric() || character == '_'))
            .filter(|name| !name.is_empty())
            .collect()
    }

    assert_eq!(
        reexported_names(public_front_door, "error"),
        [
            "AtomicInlineParticipationRoleError",
            "CalcSizeBehaviorBasis",
            "FloatExclusionRoleError",
            "InvalidMeasurementOutput",
            "InvalidMeasurementOutputOf",
            "LayoutError",
            "LayoutErrorKind",
            "LayoutErrorKindOf",
            "LayoutErrorOf",
            "LayoutErrorSite",
            "LayoutErrorSiteOf",
            "LayoutInternalInvariant",
            "LayoutInvalidInput",
            "LayoutInvalidInputOf",
            "LayoutMissingContext",
            "LayoutOperation",
            "LayoutResult",
            "LayoutResultOf",
            "LayoutUnsupportedCapability",
            "NonBoxNodeRoleError",
            "SizingAlgorithm",
            "SizingBehavior",
            "SizingProperty",
            "UnsupportedSizingBehavior",
        ],
        "the public error reexport inventory remains exact"
    );
    assert_eq!(
        reexported_names(public_front_door, "compute"),
        ["compute_layout", "compute_layout_invalidated",],
        "compute retains only public root orchestration during R02"
    );
    assert_eq!(
        reexported_names(public_front_door, "measurement"),
        [
            "LeafMeasureError",
            "LeafMeasureErrorOf",
            "LeafMeasureInput",
            "LeafMeasureInputOf",
            "MeasurementAvailable",
            "MeasurementAvailableOf",
            "compute_leaf",
        ],
        "measurement owns the unchanged public leaf facade"
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
fn fri05_c07_public_surface_removed_phase_unsafe_contracts_fail_closed() {
    let node_input = include_str!("node_input.rs");
    let output = include_str!("output.rs");
    let scroll = include_str!("scroll.rs");
    let public_front_door = include_str!("lib.rs");
    let production = format!("{node_input}\n{output}\n{scroll}\n{public_front_door}");
    let tokens = fri05_c05_lex_production_tokens(&production)
        .expect("public FRI-05 sources must remain lexically auditable");

    for removed in [
        "ScrollOverflowExposure",
        "ScrollContainerAxis",
        "ScrollContainerFacts",
        "scroll_container_facts_from_overflow",
        "ScrollUnsupportedFeature",
        "ScrollOverflowCouplingPolicy",
        "LayoutOwnedMixedAxisOverflowCoupling",
        "LiveScrollOffset",
        "CurrentScrollOffset",
        "ScrollState",
    ] {
        assert!(
            tokens.iter().all(|token| token.text != removed),
            "removed phase-unsafe surface remains: {removed}"
        );
    }

    assert!(!node_input.contains("pub overflow: Point<Overflow>"));
    assert!(!output.contains("pub scrollbar_size:"));
    assert!(!node_input.contains("ScrollPaddingValueOf::Deferred"));
    assert!(!node_input.contains("pub const fn clips_contents"));
    assert!(!node_input.contains("pub const fn blocks_margin_collapse"));
    let rect_impl = scroll
        .split_once("impl<S: LayoutScalar> ScrollRectOf<S> {")
        .expect("rectangle implementation")
        .1
        .split_once("/// A finite ordered physical clip interval.")
        .expect("rectangle implementation end")
        .0;
    assert!(!rect_impl.contains("pub fn new("));
    assert_eq!(rect_impl.matches("pub fn try_new(").count(), 1);

    for (type_name, section_end) in [
        ("PhysicalClipAxisOf", "pub struct OverflowClipOf"),
        ("OverflowClipOf", "pub struct ScrollTargetGeometryOf"),
        (
            "ScrollTargetGeometryOf",
            "/// Construction error for a signed physical or flow-relative scroll coordinate.",
        ),
        (
            "ScrollbarGutterRectsOf",
            "pub(crate) struct ClipMarginSourceOf",
        ),
        (
            "ScrollGeometryOf",
            "pub(crate) enum CanonicalScrollRectFact",
        ),
    ] {
        let declaration = format!("pub struct {type_name}");
        let declaration_index = scroll.find(&declaration).expect("carrier declaration");
        let section = scroll[declaration_index..]
            .split_once(section_end)
            .expect("carrier section end")
            .0;
        let fields = section
            .split_once('{')
            .expect("carrier fields begin")
            .1
            .split_once('}')
            .expect("carrier fields end")
            .0;
        assert!(!fields.contains("pub "), "{type_name} fields stay private");
        assert!(
            !section.contains("pub fn new("),
            "{type_name} has no constructor"
        );
        assert!(
            !section.contains("pub const fn new("),
            "{type_name} has no constructor"
        );
        assert!(
            !section.contains("pub fn try_new("),
            "{type_name} has no constructor"
        );
        assert!(
            !section.contains("pub const fn try_new("),
            "{type_name} has no constructor"
        );
        assert!(
            !scroll.contains(&format!("Default for {type_name}")),
            "{type_name} has no placeholder default"
        );
    }
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
