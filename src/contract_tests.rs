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
fn fri06_c02_contract_block_has_no_c02_text_fallback_spelling() {
    let block = include_str!("block.rs");
    assert!(
        !block.contains("LaterFriBehavior"),
        "C02 text paths must be closed while C03 mixed behavior remains typed indirectly"
    );
}

#[test]
fn fri06_c02_contract_inline_has_no_shaping_or_measurement_path() {
    let inline = include_str!("inline.rs");
    for forbidden in ["shape", "glyph", "font", "measure_leaf"] {
        assert!(
            !inline.contains(forbidden),
            "inline production contains forbidden shaping or measurement spelling {forbidden}"
        );
    }
}

#[test]
fn fri06_c02_contract_text_source_has_no_owned_dead_code_allowance() {
    let inline = include_str!("inline.rs");
    let text_source = inline
        .split_once("pub(super) struct ShapedTextParticipantOf")
        .unwrap()
        .1
        .split_once("pub(super) struct AtomicInlineBoxParticipant")
        .unwrap()
        .0;
    assert!(
        !text_source.contains("#[allow(dead_code)]"),
        "the consumed C02 text source retains an obsolete dead-code allowance"
    );
}

#[test]
fn fri06_c02_contract_cache_key_context_remains_one_unit_declaration() {
    let cache = include_str!("cache.rs");
    assert_eq!(
        cache
            .lines()
            .filter(|line| *line == "pub struct CacheKeyContext;")
            .count(),
        1
    );
    assert_eq!(core::mem::size_of::<CacheKeyContext>(), 0);
}

#[test]
fn parent_formatting_context_is_closed_and_exact() {
    fn name(context: ParentFormattingContext) -> &'static str {
        match context {
            crate::ParentFormattingContext::NoParent => "no-parent",
            crate::ParentFormattingContext::BlockFlow => "block-flow",
            crate::ParentFormattingContext::Flex => "flex",
            crate::ParentFormattingContext::Grid => "grid",
        }
    }

    let contexts = [
        crate::ParentFormattingContext::NoParent,
        crate::ParentFormattingContext::BlockFlow,
        crate::ParentFormattingContext::Flex,
        crate::ParentFormattingContext::Grid,
    ];

    assert_eq!(
        contexts.map(name),
        ["no-parent", "block-flow", "flex", "grid"]
    );
}

#[test]
fn containing_layout_context_keeps_flow_and_role_together() {
    fn assert_traits<T: Clone + Copy + std::fmt::Debug + Eq + PartialEq>() {}

    assert_traits::<ParentFormattingContext>();
    assert_traits::<ContainingLayoutContext>();

    let flow_axes = [
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
    ];
    let formatting_contexts = [
        crate::ParentFormattingContext::NoParent,
        crate::ParentFormattingContext::BlockFlow,
        crate::ParentFormattingContext::Flex,
        crate::ParentFormattingContext::Grid,
    ];

    for flow_axes in flow_axes {
        for formatting_context in formatting_contexts {
            let context = crate::ContainingLayoutContext::new(flow_axes, formatting_context);

            assert_eq!(context.flow_axes(), flow_axes);
            assert_eq!(context.formatting_context(), formatting_context);
        }
    }
}

#[test]
fn flex_item_root_context_requires_explicit_parent_axes() {
    fn assert_traits<T: Clone + Copy + std::fmt::Debug + PartialEq>() {}

    assert_traits::<FlexItemRootContextOf<f32>>();
    assert_traits::<FlexItemRootContextOf<f64>>();

    fn assert_lane<S: LayoutScalar>() {
        let viewport = Size::new(
            AvailableOf::definite(S::from_f64(640.0)),
            AvailableOf::definite(S::from_f64(480.0)),
        );
        let parent_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
        let context = FlexItemRootContextOf::<S>::under_viewport(viewport, parent_axes)
            .expect("finite viewport availability is valid");

        assert_eq!(context.viewport_available(), viewport);
        assert_eq!(context.parent_flow_axes(), parent_axes);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn value_types_support_f64_scalar_lane() {
    let length = crate::LengthOf::<f64>::percent(0.25);
    let length = length.resolve(400.0);
    assert_eq!(length.value, Some(100.0));
    assert_eq!(length.status(), crate::LengthResolutionStatus::Resolved);

    let dimension = crate::PreferredSizeOf::<f64>::px(42.5);
    let dimension = dimension
        .resolve_simple_with_status(Some(1000.0))
        .expect("affine preferred size is supported");
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
            crate::PreferredSizeOf::px(123.5),
            crate::PreferredSizeOf::percent(0.25),
        ),
        margin: crate::Edges::all(crate::LengthAutoOf::px(2.5)),
        flex_grow: FlexGrowOf::try_new(1.0).unwrap(),
        ..crate::NodeInputOf::<f64>::default()
    };

    let width = input
        .size
        .width
        .resolve_simple_with_status(Some(1000.0))
        .expect("affine preferred width is supported");
    let height = input
        .size
        .height
        .resolve_simple_with_status(Some(400.0))
        .expect("affine preferred height is supported");
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

fn fri06_c01_segment<S: LayoutScalar>(
    id: u64,
    whitespace: InlineWhitespaceEdge,
    following_break: InlineBreakOpportunityOf<S>,
) -> ShapedInlineSegmentOf<S> {
    ShapedInlineSegmentOf::try_new(
        InlineSegmentId::new(id),
        S::from_f64(12.0),
        InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0)).unwrap(),
        BidiLevel::try_new(1).unwrap(),
        whitespace,
        following_break,
    )
    .unwrap()
}

fn assert_fri06_c01_inline_model<S: LayoutScalar>() {
    fn assert_copy<T: Clone + Copy + core::fmt::Debug + PartialEq>() {}
    fn assert_owned<T: Clone + core::fmt::Debug + PartialEq>() {}

    assert_copy::<InlineSegmentId>();
    assert_copy::<BidiLevel>();
    assert_copy::<InlineWhitespaceEdge>();
    assert_copy::<InlineBreakKind>();
    assert_copy::<InlineBreakOpportunityOf<S>>();
    assert_copy::<ShapedInlineSegmentOf<S>>();
    assert_copy::<AtomicInlineParticipationOf<S>>();
    assert_owned::<InlineTextInputOf<S>>();

    let id = InlineSegmentId::new(42);
    assert_eq!(id.get(), 42);
    assert!(!BidiLevel::try_new(0).unwrap().is_rtl());
    assert!(BidiLevel::try_new(125).unwrap().is_rtl());
    assert_eq!(
        BidiLevel::try_new(126),
        Err(BidiLevelError::OutOfRange { level: 126 })
    );

    let prohibited = InlineBreakOpportunityOf::<S>::prohibited();
    let allowed = InlineBreakOpportunityOf::<S>::allowed();
    let mandatory = InlineBreakOpportunityOf::<S>::mandatory();
    let replacement =
        InlineBreakOpportunityOf::<S>::try_allowed_with_replacement(S::from_f64(3.0)).unwrap();
    assert_eq!(prohibited.kind(), InlineBreakKind::Prohibited);
    assert_eq!(allowed.kind(), InlineBreakKind::Allowed);
    assert_eq!(mandatory.kind(), InlineBreakKind::Mandatory);
    assert_eq!(replacement.kind(), InlineBreakKind::AllowedWithReplacement);
    assert_eq!(
        replacement.replacement_inline_extent(),
        Some(S::from_f64(3.0))
    );
    assert_eq!(allowed.replacement_inline_extent(), None);
    for rejected in [S::from_f64(-1.0), S::INFINITY, S::NAN] {
        assert!(matches!(
            InlineBreakOpportunityOf::<S>::try_allowed_with_replacement(rejected),
            Err(InlineTextInputErrorOf::InvalidReplacementInlineExtent { .. })
        ));
    }
    let negative_zero =
        InlineBreakOpportunityOf::<S>::try_allowed_with_replacement(-S::ZERO).unwrap();
    assert_eq!(negative_zero.replacement_inline_extent(), Some(S::ZERO));

    for whitespace in [
        InlineWhitespaceEdge::Preserve,
        InlineWhitespaceEdge::DiscardAtLineStart,
        InlineWhitespaceEdge::DiscardAtLineEnd,
        InlineWhitespaceEdge::DiscardAtBoth,
    ] {
        let segment = fri06_c01_segment(7, whitespace, allowed);
        assert_eq!(segment.segment_id(), InlineSegmentId::new(7));
        assert_eq!(segment.inline_extent(), S::from_f64(12.0));
        assert_eq!(
            segment.metrics(),
            InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0)).unwrap()
        );
        assert_eq!(segment.bidi_level(), BidiLevel::try_new(1).unwrap());
        assert_eq!(segment.whitespace_edge(), whitespace);
        assert_eq!(segment.following_break(), allowed);
    }

    for whitespace_edge in [
        InlineWhitespaceEdge::DiscardAtLineStart,
        InlineWhitespaceEdge::DiscardAtLineEnd,
        InlineWhitespaceEdge::DiscardAtBoth,
    ] {
        assert!(matches!(
            ShapedInlineSegmentOf::try_new(
                InlineSegmentId::new(8),
                S::from_f64(1.0),
                InlineMetricsOf::default(),
                BidiLevel::try_new(0).unwrap(),
                whitespace_edge,
                replacement,
            ),
            Err(InlineTextInputErrorOf::ReplacementWithDiscardableWhitespace { .. })
        ));
    }
    for inline_extent in [S::from_f64(-1.0), S::INFINITY, S::NAN] {
        assert!(matches!(
            ShapedInlineSegmentOf::try_new(
                InlineSegmentId::new(8),
                inline_extent,
                InlineMetricsOf::default(),
                BidiLevel::try_new(0).unwrap(),
                InlineWhitespaceEdge::Preserve,
                prohibited,
            ),
            Err(InlineTextInputErrorOf::InvalidInlineExtent { .. })
        ));
    }

    assert_eq!(
        InlineTextInputOf::<S>::try_new(Vec::new()),
        Err(InlineTextInputErrorOf::Empty)
    );
    let first = fri06_c01_segment(1, InlineWhitespaceEdge::Preserve, prohibited);
    assert_eq!(
        InlineTextInputOf::try_new(vec![first, first]),
        Err(InlineTextInputErrorOf::DuplicateSegmentId {
            segment_id: InlineSegmentId::new(1),
        })
    );
    let text = InlineTextInputOf::try_new(vec![first]).unwrap();
    assert_eq!(text.segments(), &[first]);
    assert_eq!(
        LayoutInputOf::inline_text(text.clone()).as_inline_text(),
        Some(&text)
    );

    let atomic =
        AtomicInlineParticipationOf::try_new(BidiLevel::try_new(2).unwrap(), allowed).unwrap();
    assert_eq!(atomic.bidi_level(), BidiLevel::try_new(2).unwrap());
    assert_eq!(atomic.following_break(), allowed);
    for following_break in [prohibited, allowed, mandatory] {
        assert!(
            AtomicInlineParticipationOf::try_new(BidiLevel::try_new(0).unwrap(), following_break,)
                .is_ok()
        );
    }
    assert!(matches!(
        AtomicInlineParticipationOf::try_new(BidiLevel::try_new(0).unwrap(), replacement),
        Err(AtomicInlineParticipationErrorOf::ReplacementNotAllowed { .. })
    ));

    let default = NodeInputOf::<S>::default();
    assert_eq!(default.atomic_inline_participation, None);
    assert_eq!(default.float_exclusion, FloatExclusion::MarginBox);
    let non_box = NodeInputOf::<S>::non_box();
    assert_eq!(
        non_box,
        NodeInputOf {
            display: Display::None,
            ..NodeInputOf::default()
        }
    );
    assert_eq!(non_box.display, Display::None);
    assert_eq!(non_box.atomic_inline_participation, None);
    assert_eq!(non_box.float_exclusion, FloatExclusion::MarginBox);
    let vertical_align_name = |value| match value {
        VerticalAlign::Baseline => "baseline",
        VerticalAlign::Top => "top",
        VerticalAlign::Bottom => "bottom",
    };
    assert_eq!(vertical_align_name(VerticalAlign::Bottom), "bottom");
}

#[test]
fn fri06_c01_inline_model_validates_and_exposes_both_scalar_lanes() {
    assert_fri06_c01_inline_model::<f32>();
    assert_fri06_c01_inline_model::<f64>();
}

fn fri06_mr02_segments<S: LayoutScalar>(
    ids: impl IntoIterator<Item = u64>,
) -> Vec<ShapedInlineSegmentOf<S>> {
    ids.into_iter()
        .map(|id| {
            fri06_c01_segment(
                id,
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            )
        })
        .collect()
}

#[test]
fn fri06_mr02_duplicate_id_empty_input_is_rejected_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        assert_eq!(
            InlineTextInputOf::<S>::try_new(Vec::new()),
            Err(InlineTextInputErrorOf::Empty)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr02_duplicate_id_unique_long_input_preserves_order_and_value_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let segments = fri06_mr02_segments::<S>(0..2_048);
        let text = InlineTextInputOf::try_new(segments.clone()).unwrap();

        assert_eq!(text.segments(), segments);
        assert_eq!(text.clone(), text);
        assert_eq!(
            text.segments()
                .iter()
                .map(|segment| segment.segment_id().get())
                .collect::<Vec<_>>(),
            (0..2_048).collect::<Vec<_>>()
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr02_duplicate_id_first_possible_repeat_returns_current_id_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        assert_eq!(
            InlineTextInputOf::try_new(fri06_mr02_segments::<S>([17, 17, 99])),
            Err(InlineTextInputErrorOf::DuplicateSegmentId {
                segment_id: InlineSegmentId::new(17),
            })
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr02_duplicate_id_final_repeat_returns_current_id_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        assert_eq!(
            InlineTextInputOf::try_new(fri06_mr02_segments::<S>([3, 4, 5, 3])),
            Err(InlineTextInputErrorOf::DuplicateSegmentId {
                segment_id: InlineSegmentId::new(3),
            })
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr02_duplicate_id_competing_families_return_first_repeat_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        assert_eq!(
            InlineTextInputOf::try_new(fri06_mr02_segments::<S>([2, 900, 900, 2])),
            Err(InlineTextInputErrorOf::DuplicateSegmentId {
                segment_id: InlineSegmentId::new(900),
            })
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[derive(Clone)]
struct Fri06C01Tree<S: LayoutScalar> {
    inputs: Vec<LayoutInputOf<S>>,
    nodes: Vec<NodeInputOf<S>>,
    children: Vec<Vec<usize>>,
    measured: Vec<bool>,
    cache_reads: std::cell::Cell<usize>,
    provider_calls: std::cell::Cell<usize>,
}

impl<S: LayoutScalar> Traverse for Fri06C01Tree<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children[node].iter().copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children[node].len()
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[node][index]
    }
}

impl<S: LayoutScalar> LayoutTree for Fri06C01Tree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        &self.nodes[node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        self.inputs[node].clone()
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.measured[node]
    }

    fn cache_get(
        &self,
        _node: Self::Node,
        _input: &ComputeInputOf<Self::Scalar>,
        _context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>> {
        self.cache_reads.set(self.cache_reads.get() + 1);
        None
    }

    fn float_exclusion_interval(
        &self,
        _node: Self::Node,
        _query: FloatExclusionQueryOf<Self::Scalar>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<Self::Scalar>>, Self::MeasureError>> {
        self.provider_calls.set(self.provider_calls.get() + 1);
        None
    }
}

fn fri06_c01_tree<S: LayoutScalar>(
    input: LayoutInputOf<S>,
    node: NodeInputOf<S>,
) -> Fri06C01Tree<S> {
    Fri06C01Tree {
        inputs: vec![input],
        nodes: vec![node],
        children: vec![Vec::new()],
        measured: vec![false],
        cache_reads: std::cell::Cell::new(0),
        provider_calls: std::cell::Cell::new(0),
    }
}

fn fri06_c01_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(100.0)))).unwrap()
}

fn assert_fri06_c01_non_box<S: LayoutScalar>() {
    let segment: ShapedInlineSegmentOf<S> = fri06_c01_segment(
        1,
        InlineWhitespaceEdge::Preserve,
        InlineBreakOpportunityOf::prohibited(),
    );
    let text = InlineTextInputOf::try_new(vec![segment]).unwrap();
    let valid = fri06_c01_tree(
        LayoutInputOf::inline_text(text.clone()),
        NodeInputOf::non_box(),
    );
    let error = compute_layout(&valid, 0, fri06_c01_request()).unwrap_err();
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    assert_eq!(
        error.kind(),
        &LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::LaterFriBehavior)
    );
    assert_eq!(valid.cache_reads.get(), 0);

    let noncanonical = fri06_c01_tree(
        LayoutInputOf::inline_text(text.clone()),
        NodeInputOf::default(),
    );
    assert!(matches!(
        compute_layout(&noncanonical, 0, fri06_c01_request())
            .unwrap_err()
            .kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::NonBoxNodeRole {
            reason: NonBoxNodeRoleError::NonCanonicalNodeInput,
        })
    ));

    let mut childful = fri06_c01_tree(
        LayoutInputOf::inline_text(text.clone()),
        NodeInputOf::non_box(),
    );
    childful
        .inputs
        .push(LayoutInputOf::box_input(NodeInputOf::default()));
    childful.nodes.push(NodeInputOf::default());
    childful.children[0].push(1);
    childful.children.push(Vec::new());
    childful.measured.push(false);
    assert!(matches!(
        compute_layout(&childful, 0, fri06_c01_request())
            .unwrap_err()
            .kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::NonBoxNodeRole {
            reason: NonBoxNodeRoleError::HasChildren,
        })
    ));

    let mut measured = fri06_c01_tree(LayoutInputOf::inline_text(text), NodeInputOf::non_box());
    measured.measured[0] = true;
    assert!(matches!(
        compute_layout(&measured, 0, fri06_c01_request())
            .unwrap_err()
            .kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::NonBoxNodeRole {
            reason: NonBoxNodeRoleError::HasLeafMeasurement,
        })
    ));
}

#[test]
fn fri06_c01_non_box_pairing_and_text_handoff_are_typed_and_cache_neutral() {
    assert_fri06_c01_non_box::<f32>();
    assert_fri06_c01_non_box::<f64>();
}

fn assert_fri06_mr01_non_box_precedence<S: LayoutScalar>(input: LayoutInputOf<S>) {
    let cases = [
        (
            false,
            false,
            false,
            NonBoxNodeRoleError::NonCanonicalNodeInput,
        ),
        (
            false,
            true,
            false,
            NonBoxNodeRoleError::NonCanonicalNodeInput,
        ),
        (
            false,
            false,
            true,
            NonBoxNodeRoleError::NonCanonicalNodeInput,
        ),
        (
            false,
            true,
            true,
            NonBoxNodeRoleError::NonCanonicalNodeInput,
        ),
        (true, true, false, NonBoxNodeRoleError::HasChildren),
        (true, true, true, NonBoxNodeRoleError::HasChildren),
        (true, false, true, NonBoxNodeRoleError::HasLeafMeasurement),
    ];

    for (canonical_input, has_children, has_leaf_measurement, expected_reason) in cases {
        let node_input = if canonical_input {
            NodeInputOf::non_box()
        } else {
            NodeInputOf::default()
        };
        let mut tree = fri06_c01_tree(input.clone(), node_input);
        tree.measured[0] = has_leaf_measurement;
        if has_children {
            tree.inputs
                .push(LayoutInputOf::box_input(NodeInputOf::default()));
            tree.nodes.push(NodeInputOf::default());
            tree.children[0].push(1);
            tree.children.push(Vec::new());
            tree.measured.push(false);
        }

        let error = compute_layout(&tree, 0, fri06_c01_request()).unwrap_err();

        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::NonBoxNodeRole {
                reason: expected_reason,
            })
        );
    }
}

#[test]
fn fri06_mr01_non_box_inline_text_precedence_is_exact_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let segment: ShapedInlineSegmentOf<S> = fri06_c01_segment(
            1,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        );
        let text = InlineTextInputOf::try_new(vec![segment]).unwrap();
        assert_fri06_mr01_non_box_precedence(LayoutInputOf::inline_text(text));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr01_non_box_line_break_precedence_is_exact_in_both_scalar_lanes() {
    assert_fri06_mr01_non_box_precedence::<f32>(LayoutInputOf::line_break(LineBreakInputOf::new()));
    assert_fri06_mr01_non_box_precedence::<f64>(LayoutInputOf::line_break(LineBreakInputOf::new()));
}

#[test]
fn fri06_mr01_non_box_inline_boundary_precedence_is_exact_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        assert_fri06_mr01_non_box_precedence(LayoutInputOf::inline_boundary(
            InlineBoundaryInputOf::<S>::new(InlineBoundaryKind::Start, InlineMetricsOf::default()),
        ));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn assert_fri06_c01_box_roles<S: LayoutScalar>() {
    let child_tree = |style: NodeInputOf<S>| Fri06C01Tree {
        inputs: vec![
            LayoutInputOf::box_input(NodeInputOf::default()),
            LayoutInputOf::box_input(style.clone()),
        ],
        nodes: vec![NodeInputOf::default(), style],
        children: vec![vec![1], Vec::new()],
        measured: vec![false, false],
        cache_reads: std::cell::Cell::new(0),
        provider_calls: std::cell::Cell::new(0),
    };
    for display in [
        Display::InlineBlock,
        Display::InlineGrid,
        Display::InlineGridLanes,
    ] {
        let missing_atomic = NodeInputOf::<S> {
            display,
            ..NodeInputOf::default()
        };
        let tree = child_tree(missing_atomic);
        assert!(matches!(
            compute_layout(&tree, 0, fri06_c01_request())
                .unwrap_err()
                .kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::AtomicInlineParticipation {
                reason: AtomicInlineParticipationRoleError::MissingForAtomicInline,
            })
        ));
        assert_eq!(tree.cache_reads.get(), 0);
        assert_eq!(tree.provider_calls.get(), 0);
    }

    let atomic = AtomicInlineParticipationOf::try_new(
        BidiLevel::try_new(0).unwrap(),
        InlineBreakOpportunityOf::mandatory(),
    )
    .unwrap();
    let extraneous_atomic = NodeInputOf::<S> {
        atomic_inline_participation: Some(atomic),
        ..NodeInputOf::default()
    };
    let tree = child_tree(extraneous_atomic);
    assert!(matches!(
        compute_layout(&tree, 0, fri06_c01_request())
            .unwrap_err()
            .kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::AtomicInlineParticipation {
            reason: AtomicInlineParticipationRoleError::UnexpectedForNonAtomic,
        })
    ));
    assert_eq!(tree.cache_reads.get(), 0);
    assert_eq!(tree.provider_calls.get(), 0);

    for (style, expected) in [
        (
            NodeInputOf::<S> {
                float_exclusion: FloatExclusion::Shape,
                ..NodeInputOf::default()
            },
            FloatExclusionRoleError::NonFloating,
        ),
        (
            NodeInputOf::<S> {
                display: Display::None,
                float_exclusion: FloatExclusion::Shape,
                ..NodeInputOf::default()
            },
            FloatExclusionRoleError::Hidden,
        ),
        (
            NodeInputOf::<S> {
                position: Position::Absolute,
                float: Float::Left,
                float_exclusion: FloatExclusion::Shape,
                ..NodeInputOf::default()
            },
            FloatExclusionRoleError::Absolute,
        ),
    ] {
        let tree = fri06_c01_tree(LayoutInputOf::box_input(style.clone()), style);
        assert!(matches!(
            compute_layout(&tree, 0, fri06_c01_request()).unwrap_err().kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::FloatExclusionRole {
                reason,
            }) if *reason == expected
        ));
        assert_eq!(tree.cache_reads.get(), 0);
        assert_eq!(tree.provider_calls.get(), 0);
    }
}

#[test]
fn fri06_c05_provider_role_invalid_shapes_reject_before_cache_and_provider_both_scalars() {
    assert_fri06_c01_box_roles::<f32>();
    assert_fri06_c01_box_roles::<f64>();
}

#[test]
fn fri06_c01_non_box_atomic_and_shape_roles_reject_before_cache_activity() {
    assert_fri06_c01_box_roles::<f32>();
    assert_fri06_c01_box_roles::<f64>();
}

fn assert_fri06_c01_float_exclusion_contract<S: LayoutScalar>() {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let margin_box = ScrollRectOf::try_new(
        Point::new(S::from_f64(-10.0), S::from_f64(20.0)),
        Size::new(S::from_f64(100.0), S::from_f64(40.0)),
    )
    .unwrap();
    let query =
        FloatExclusionQueryOf::try_new(margin_box, axes, S::from_f64(21.0), S::from_f64(59.0))
            .unwrap();
    assert_eq!(query.margin_box(), margin_box);
    assert_eq!(query.flow_axes(), axes);
    assert_eq!(query.band_minimum(), S::from_f64(21.0));
    assert_eq!(query.band_maximum(), S::from_f64(59.0));

    for (minimum, maximum, expected) in [
        (-5.0, 25.0, Some((-5.0, 25.0))),
        (-50.0, 25.0, Some((-10.0, 25.0))),
        (80.0, 120.0, Some((80.0, 90.0))),
        (-50.0, -20.0, None),
        (100.0, 120.0, None),
        (4.0, 4.0, None),
    ] {
        let interval =
            FloatExclusionIntervalOf::try_new(query, S::from_f64(minimum), S::from_f64(maximum))
                .unwrap();
        assert_eq!(
            interval.map(|interval| (interval.minimum(), interval.maximum())),
            expected.map(|(minimum, maximum)| (S::from_f64(minimum), S::from_f64(maximum)))
        );
    }

    assert!(matches!(
        FloatExclusionQueryOf::try_new(margin_box, axes, S::NAN, S::ZERO),
        Err(FloatExclusionIntervalErrorOf::NonFiniteBandMinimum { .. })
    ));
    assert!(matches!(
        FloatExclusionQueryOf::try_new(margin_box, axes, S::ZERO, S::INFINITY),
        Err(FloatExclusionIntervalErrorOf::NonFiniteBandMaximum { .. })
    ));
    assert!(matches!(
        FloatExclusionQueryOf::try_new(margin_box, axes, S::ONE, S::ZERO),
        Err(FloatExclusionIntervalErrorOf::InvertedBand { .. })
    ));
    assert!(matches!(
        FloatExclusionIntervalOf::try_new(query, S::NAN, S::ZERO),
        Err(FloatExclusionIntervalErrorOf::NonFiniteIntervalMinimum { .. })
    ));
    assert!(matches!(
        FloatExclusionIntervalOf::try_new(query, S::ZERO, S::INFINITY),
        Err(FloatExclusionIntervalErrorOf::NonFiniteIntervalMaximum { .. })
    ));
    assert!(matches!(
        FloatExclusionIntervalOf::try_new(query, S::ONE, S::ZERO),
        Err(FloatExclusionIntervalErrorOf::InvertedInterval { .. })
    ));

    let vertical_query = FloatExclusionQueryOf::try_new(
        margin_box,
        FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        S::from_f64(-10.0),
        S::from_f64(90.0),
    )
    .unwrap();
    let vertical =
        FloatExclusionIntervalOf::try_new(vertical_query, S::from_f64(-20.0), S::from_f64(100.0))
            .unwrap()
            .unwrap();
    assert_eq!(
        (vertical.minimum(), vertical.maximum()),
        (S::from_f64(20.0), S::from_f64(60.0))
    );
}

#[test]
fn fri06_c01_float_exclusion_query_and_interval_validate_both_scalar_lanes() {
    assert_fri06_c01_float_exclusion_contract::<f32>();
    assert_fri06_c01_float_exclusion_contract::<f64>();
}

fn assert_fri06_c05_query_mismatch_error_contract<S: LayoutScalar>() {
    let margin_box =
        ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(30.0), S::from_f64(40.0)))
            .unwrap();
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let expected =
        FloatExclusionQueryOf::try_new(margin_box, axes, S::from_f64(2.0), S::from_f64(8.0))
            .unwrap();
    let actual =
        FloatExclusionQueryOf::try_new(margin_box, axes, S::from_f64(12.0), S::from_f64(18.0))
            .unwrap();
    let mismatch = FloatExclusionIntervalErrorOf::QueryMismatch { expected, actual };

    assert_eq!(
        mismatch.to_string(),
        "provider interval query must match the requested query",
    );
    assert!(std::error::Error::source(&mismatch).is_none());

    let variant_name = |error: FloatExclusionIntervalErrorOf<S>| match error {
        FloatExclusionIntervalErrorOf::NonFiniteBandMinimum { .. } => "band-minimum",
        FloatExclusionIntervalErrorOf::NonFiniteBandMaximum { .. } => "band-maximum",
        FloatExclusionIntervalErrorOf::InvertedBand { .. } => "band-order",
        FloatExclusionIntervalErrorOf::NonFiniteIntervalMinimum { .. } => "interval-minimum",
        FloatExclusionIntervalErrorOf::NonFiniteIntervalMaximum { .. } => "interval-maximum",
        FloatExclusionIntervalErrorOf::InvertedInterval { .. } => "interval-order",
        FloatExclusionIntervalErrorOf::QueryMismatch { .. } => "query-mismatch",
    };
    assert_eq!(variant_name(mismatch), "query-mismatch");
}

#[test]
fn fri06_c05_provider_error_query_mismatch_is_exhaustive_and_exact_both_scalars() {
    assert_fri06_c05_query_mismatch_error_contract::<f32>();
    assert_fri06_c05_query_mismatch_error_contract::<f64>();
}

#[test]
fn fri06_c01_float_exclusion_default_provider_returns_none() {
    let tree = fri06_c01_tree::<f32>(
        LayoutInputOf::box_input(NodeInput::default()),
        NodeInput::default(),
    );
    let margin_box = ScrollRect::try_new(Point::ZERO, Size::new(10.0, 20.0)).unwrap();
    let query = FloatExclusionQuery::try_new(
        margin_box,
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        1.0,
        2.0,
    )
    .unwrap();
    assert_eq!(tree.float_exclusion_interval(0, query), None);
}

#[test]
fn fri06_c01_float_exclusion_diagnostics_preserve_provider_and_site_context() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ProviderFailure(u8);

    let missing = LayoutErrorOf::<u16, f64, ProviderFailure>::new(
        LayoutErrorSiteOf::ContainerSubject {
            container: 3,
            subject: 7,
        },
        LayoutOperation::FloatExclusionQuery,
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::FloatExclusionProvider),
    );
    assert_eq!(
        missing.site(),
        LayoutErrorSiteOf::ContainerSubject {
            container: 3,
            subject: 7,
        }
    );
    assert_eq!(missing.operation(), LayoutOperation::FloatExclusionQuery);

    let query = FloatExclusionQueryOf::try_new(
        ScrollRectOf::try_new(Point::ZERO, Size::new(10.0_f64, 20.0)).unwrap(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        4.0,
        8.0,
    )
    .unwrap();
    assert_eq!((query.band_minimum(), query.band_maximum()), (4.0, 8.0));
    let invalid_output = FloatExclusionIntervalOf::try_new(query, 9.0, 8.0).unwrap_err();
    let invalid = LayoutErrorOf::<u16, f64, ProviderFailure>::new(
        LayoutErrorSiteOf::ContainerSubject {
            container: 3,
            subject: 7,
        },
        LayoutOperation::FloatExclusionQuery,
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::FloatExclusionProviderOutput {
            error: invalid_output,
        }),
    );
    assert!(matches!(
        invalid.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::FloatExclusionProviderOutput {
            error: FloatExclusionIntervalErrorOf::InvertedInterval { .. },
        })
    ));

    let provider = LayoutErrorOf::<u16, f64, ProviderFailure>::new(
        LayoutErrorSiteOf::ContainerSubject {
            container: 3,
            subject: 7,
        },
        LayoutOperation::FloatExclusionQuery,
        LayoutErrorKindOf::Measurement(ProviderFailure(11)),
    );
    assert_eq!(
        provider.kind(),
        &LayoutErrorKindOf::Measurement(ProviderFailure(11))
    );
}

#[test]
fn public_order_source_types_and_defaults_are_exact() {
    use crate::{ItemOrder, SourceIndex};

    assert_eq!(ItemOrder::default(), ItemOrder::ZERO);
    assert_eq!(NodeInput::DEFAULT.item_order, ItemOrder::ZERO);
    assert_eq!(NodeInputOf::<f32>::default().item_order, ItemOrder::ZERO);
    assert_eq!(NodeInputOf::<f64>::default().item_order, ItemOrder::ZERO);

    let mut orders = [
        ItemOrder::new(i32::MAX),
        ItemOrder::new(0),
        ItemOrder::new(i32::MIN),
    ];
    assert_eq!(orders.map(ItemOrder::get), [i32::MAX, 0, i32::MIN]);
    orders.sort();
    assert_eq!(orders.map(ItemOrder::get), [i32::MIN, 0, i32::MAX]);

    assert_eq!(SourceIndex::ZERO.get(), 0);
    assert_eq!(SourceIndex::new(7).get(), 7);
    assert!(SourceIndex::new(2) < SourceIndex::new(7));
}

#[test]
fn node_output_source_index_is_unambiguous() {
    fn assert_source_index(_: SourceIndex) {}

    let default_output = NodeOutput::default();
    let constructed_output = NodeOutput::new();
    let indexed_output = NodeOutput::with_source_index(SourceIndex::new(7));

    assert_source_index(default_output.source_index);
    assert_eq!(default_output.source_index, SourceIndex::ZERO);
    assert_eq!(constructed_output.source_index, SourceIndex::ZERO);
    assert_eq!(indexed_output.source_index, SourceIndex::new(7));
}

#[test]
fn compute_output_defaults_to_no_scroll_geometry() {
    let output = ComputeOutput::from_outer_size(Size::new(10.0, 20.0));

    assert_eq!(output.scroll_geometry, None);
}

#[test]
fn node_output_defaults_to_no_scroll_geometry() {
    let output = NodeOutput::with_source_index(crate::SourceIndex::new(7));

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
fn fri05_c02_carrier_public_aliases_and_generic_traits_are_available() {
    fn assert_traits<T: Clone + Copy + core::fmt::Debug + PartialEq>() {}

    assert_traits::<crate::PhysicalClipAxis>();
    assert_traits::<crate::PhysicalClipAxisOf<f64>>();
    assert_traits::<crate::OverflowClip>();
    assert_traits::<crate::OverflowClipOf<f64>>();
    assert_traits::<crate::ScrollTargetGeometry>();
    assert_traits::<crate::ScrollTargetGeometryOf<f64>>();
    assert_traits::<crate::ScrollRectError>();
    assert_traits::<crate::ScrollRectErrorOf<f64>>();
}

struct Fri05C03ContractGeometryFacts<S: crate::LayoutScalar> {
    flow_axes: crate::FlowAxes,
    overflow: crate::ComputedOverflow,
    item_is_replaced: bool,
    border_box_size: crate::Size<S>,
    padding: crate::Edges<S>,
    border: crate::Edges<S>,
    scrollbar_width: S,
    scrollable_overflow: crate::ScrollRectOf<S>,
}

fn fri05_c03_contract_geometry<S: crate::LayoutScalar>(
    facts: Fri05C03ContractGeometryFacts<S>,
) -> crate::ScrollGeometryOf<S> {
    let mut contributions =
        crate::scroll::ScrollContributionAccumulatorOf::new(facts.scrollable_overflow);
    contributions.include_direct_line(facts.scrollable_overflow);
    crate::scroll::canonical_scroll_geometry_from_source(
        crate::scroll::CanonicalScrollGeometrySourceOf {
            flow_axes: facts.flow_axes,
            computed_overflow: facts.overflow,
            item_is_replaced: facts.item_is_replaced,
            border_box_size: facts.border_box_size,
            border: facts.border,
            padding: facts.padding,
            scrollbar_gutter: crate::ScrollbarGutter::Auto,
            scrollbar_width: crate::ScrollbarWidthOf::try_new(facts.scrollbar_width).unwrap(),
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            clip_margin: crate::scroll::ClipMarginSourceOf::default(),
            scroll_padding: crate::scroll::OptimalRegionInsetsOf::default(),
            contributions,
            origin_axes: crate::scroll::ScrollOriginAxes::new(
                crate::scroll::ScrollOriginProgression::FlowEndward,
                crate::scroll::ScrollOriginProgression::FlowEndward,
            ),
            scroll_snap_type: crate::ScrollSnapType::default(),
            target_border_box: crate::ScrollRectOf::try_new(
                crate::Point::ZERO,
                facts.border_box_size,
            )
            .unwrap(),
            target_scroll_margin: crate::ScrollMarginOf::default(),
            target_flow_axes: facts.flow_axes,
            target_snap_align: crate::ScrollSnapAlign::default(),
            target_snap_stop: crate::ScrollSnapStop::default(),
        },
    )
    .expect("canonical contract source facts produce geometry")
}

#[test]
fn fri05_c03_public_geometry_all_exact_accessors_compose_in_both_scalar_lanes() {
    fn assert_scalar<S: crate::LayoutScalar>() {
        let scalar = S::from_f64;
        let flow_axes =
            crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr);
        let scrollable_overflow = crate::ScrollRectOf::try_new(
            crate::Point::new(scalar(-5.0), scalar(-3.0)),
            crate::Size::new(scalar(140.0), scalar(70.0)),
        )
        .expect("finite overflow source is valid");
        let geometry = fri05_c03_contract_geometry(Fri05C03ContractGeometryFacts {
            flow_axes,
            overflow: crate::ComputedOverflow::try_new(
                crate::Overflow::Scroll,
                crate::Overflow::Scroll,
            )
            .expect("same-group computed overflow is valid"),
            item_is_replaced: false,
            border_box_size: crate::Size::new(scalar(100.0), scalar(40.0)),
            padding: crate::Edges::all(scalar(2.0)),
            border: crate::Edges::all(scalar(1.0)),
            scrollbar_width: scalar(10.0),
            scrollable_overflow,
        });

        assert_eq!(geometry.flow_axes(), flow_axes);
        assert_eq!(geometry.used_overflow_x(), crate::Overflow::Scroll);
        assert_eq!(geometry.used_overflow_y(), crate::Overflow::Scroll);
        assert_eq!(geometry.border_box().origin(), crate::Point::ZERO);
        assert_eq!(
            geometry.border_box().size(),
            crate::Size::new(scalar(100.0), scalar(40.0))
        );
        assert_eq!(
            geometry.padding_box().origin(),
            crate::Point::new(scalar(1.0), scalar(1.0))
        );
        assert_eq!(
            geometry.content_box().size(),
            crate::Size::new(scalar(84.0), scalar(24.0))
        );
        assert_eq!(
            geometry.scrollport().size(),
            crate::Size::new(scalar(88.0), scalar(28.0))
        );
        assert!(geometry.overflow_clip().x().is_some());
        assert!(geometry.overflow_clip().y().is_some());
        assert_eq!(geometry.scrollable_overflow(), scrollable_overflow);
        let range = geometry.physical_range();
        assert!(range.x().minimum() <= S::ZERO);
        assert!(range.x().maximum() >= S::ZERO);
        assert!(range.y().minimum() <= S::ZERO);
        assert!(range.y().maximum() >= S::ZERO);
        let gutters = geometry.gutters();
        assert_eq!(gutters.top(), None);
        assert!(gutters.right().is_some());
        assert!(gutters.bottom().is_some());
        assert_eq!(gutters.left(), None);
        assert_eq!(geometry.scrollbar_size(), crate::Size::splat(scalar(10.0)));
        assert_eq!(geometry.resolved_scroll_padding(), crate::Edges::ZERO);
        assert_eq!(geometry.optimal_viewing_region(), geometry.scrollport());
        assert_eq!(geometry.scroll_snap_type(), crate::ScrollSnapType::None);
        let target = geometry.target();
        assert_eq!(target.border_box(), geometry.border_box());
        assert_eq!(target.flow_axes(), flow_axes);
        assert_eq!(
            target.scroll_margin(),
            crate::ScrollMarginOf::<S>::default()
        );
        assert_eq!(target.snap_align(), crate::ScrollSnapAlign::default());
        assert_eq!(target.snap_stop(), crate::ScrollSnapStop::Normal);
    }

    assert_scalar::<f32>();
    assert_scalar::<f64>();
}

#[test]
fn fri05_c03_output_helper_no_geometry_fallback_saturates_each_scalar_lane() {
    fn assert_scalar<S: crate::LayoutScalar>() {
        fn geometry<S: crate::LayoutScalar>(
            x: crate::Overflow,
            y: crate::Overflow,
            size: crate::Size<S>,
            scrollbar_width: S,
        ) -> crate::ScrollGeometryOf<S> {
            fri05_c03_contract_geometry(Fri05C03ContractGeometryFacts {
                flow_axes: crate::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                overflow: crate::ComputedOverflow::try_new(x, y)
                    .expect("same-group output-helper overflow is valid"),
                item_is_replaced: false,
                border_box_size: size,
                padding: crate::Edges::ZERO,
                border: crate::Edges::ZERO,
                scrollbar_width,
                scrollable_overflow: crate::ScrollRectOf::try_new(crate::Point::ZERO, size)
                    .expect("output-helper overflow rect is valid"),
            })
        }

        let no_geometry = crate::NodeOutputOf::<S> {
            size: crate::Size::new(S::from_f64(2.0), S::from_f64(3.0)),
            padding: crate::Edges::new(
                S::from_f64(2.0),
                S::from_f64(2.0),
                S::from_f64(2.0),
                S::from_f64(2.0),
            ),
            border: crate::Edges::all(S::from_f64(1.0)),
            ..crate::NodeOutputOf::<S>::new()
        };

        assert_eq!(no_geometry.content_box_size(), crate::Size::ZERO);
        assert_eq!(no_geometry.scrollbar_size(), crate::Size::ZERO);

        for (x, y, size, width, expected_scrollbars) in [
            (
                crate::Overflow::Visible,
                crate::Overflow::Visible,
                crate::Size::splat(S::from_f64(40.0)),
                S::from_f64(10.0),
                crate::Size::ZERO,
            ),
            (
                crate::Overflow::Scroll,
                crate::Overflow::Auto,
                crate::Size::splat(S::from_f64(40.0)),
                S::from_f64(10.0),
                crate::Size::new(S::ZERO, S::from_f64(10.0)),
            ),
            (
                crate::Overflow::Scroll,
                crate::Overflow::Scroll,
                crate::Size::splat(S::from_f64(40.0)),
                S::from_f64(10.0),
                crate::Size::splat(S::from_f64(10.0)),
            ),
            (
                crate::Overflow::Scroll,
                crate::Overflow::Scroll,
                crate::Size::splat(S::from_f64(2.0)),
                S::from_f64(15.0),
                crate::Size::splat(S::from_f64(2.0)),
            ),
        ] {
            let geometry = geometry(x, y, size, width);
            let output = crate::NodeOutputOf::<S> {
                size: crate::Size::splat(S::from_f64(999.0)),
                scroll_geometry: Some(geometry),
                ..crate::NodeOutputOf::<S>::new()
            };

            assert_eq!(output.scrollbar_size(), expected_scrollbars);
            assert_eq!(output.content_box_size(), geometry.content_box().size());
        }
    }

    assert_scalar::<f32>();
    assert_scalar::<f64>();

    let output = include_str!("output.rs");
    assert!(output.contains("pub const fn scrollbar_size(self) -> Size<S>"));
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
fn property_fields_preserve_layout_sizing_semantics() {
    let px = PreferredSize::px(42.0)
        .resolve_simple_with_status(Some(100.0))
        .expect("affine preferred size is supported");
    let percent = PreferredSize::percent(0.25)
        .resolve_simple_with_status(Some(100.0))
        .expect("affine preferred size is supported");
    let auto = PreferredSize::AUTO
        .resolve_simple_with_status(Some(100.0))
        .expect("auto remains an existing non-numeric keyword");

    assert_eq!(px.value, Some(42.0));
    assert_eq!(px.status(), LengthResolutionStatus::Resolved);
    assert_eq!(percent.value, Some(25.0));
    assert_eq!(percent.status(), LengthResolutionStatus::Resolved);
    assert_eq!(auto.value, None);
    assert_eq!(auto.status(), LengthResolutionStatus::NonNumeric);
    assert!(PreferredSize::MIN_CONTENT.is_min_content());
    assert!(PreferredSize::MAX_CONTENT.is_max_content());
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
    assert_eq!(node_input.overflow, ComputedOverflow::VISIBLE);
    assert_eq!(node_input.scrollbar_width.get(), 0.0);
    assert_eq!(node_input.position, Position::Relative);
    assert_eq!(node_input.inset, Edges::all(LengthAuto::AUTO));
    assert_eq!(
        node_input.size,
        Size::new(PreferredSize::AUTO, PreferredSize::AUTO)
    );
    assert_eq!(
        node_input.min_size,
        Size::new(crate::MinSize::AUTO, crate::MinSize::AUTO)
    );
    assert_eq!(
        node_input.max_size,
        Size::new(crate::MaxSize::NONE, crate::MaxSize::NONE)
    );
    assert_eq!(node_input.margin, Edges::all(LengthAuto::ZERO));
    assert_eq!(node_input.padding, Edges::all(Length::ZERO));
    assert_eq!(node_input.border, Edges::all(Length::ZERO));
    assert_eq!(node_input.gap, Size::new(Length::NORMAL, Length::NORMAL));
    assert_eq!(node_input.flex_direction, FlexDirection::Row);
    assert_eq!(node_input.flex_wrap, FlexWrap::NoWrap);
    assert_eq!(node_input.flex_basis, crate::FlexBasis::AUTO);
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

    let layout = ComputeInput::leaf_layout(
        known,
        parent,
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::NoParent),
        available,
    )
    .expect("finite direct leaf layout input");
    let content_size = ComputeInput::leaf_content_size(
        known,
        parent,
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::NoParent),
        available,
    )
    .expect("finite direct leaf content-size input");

    assert_eq!(layout.containing_flow_axes(), flow_axes);
    assert_eq!(content_size.containing_flow_axes(), flow_axes);
}

#[test]
fn compute_input_requires_complete_containing_layout_context() {
    let context = crate::ContainingLayoutContext::new(
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        crate::ParentFormattingContext::Grid,
    );
    let input = ComputeInput::leaf_layout(
        Size::NONE,
        Size::new(Some(640.0), Some(480.0)),
        context,
        Size::new(Available::definite(640.0), Available::definite(480.0)),
    )
    .expect("finite direct leaf input");

    assert_eq!(input.containing_layout_context(), context);
    assert_eq!(
        input.parent_formatting_context(),
        crate::ParentFormattingContext::Grid
    );
    assert_eq!(input.containing_flow_axes(), context.flow_axes());
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
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::NoParent),
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
