use super::fixtures::{
    PublicBlockTree, computed_overflow, fri06_atomic_participation, public_final_output,
};
use super::*;

fn lp64(absolute_px: f64, percent_fraction: f64) -> LengthPercentageOf<f64> {
    LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
        .expect("test coefficients are finite")
}

#[derive(Clone)]
struct Fri06C03FixedSizeTree<S: LayoutScalar> {
    root: NodeInputOf<S>,
    child_input: LayoutInputOf<S>,
    child_node_input: NodeInputOf<S>,
    child_layout_input_calls: Cell<usize>,
    child_compute_calls: usize,
}

impl<S: LayoutScalar> Traverse for Fri06C03FixedSizeTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        const ROOT_CHILDREN: &[u32] = &[1];
        const NO_CHILDREN: &[u32] = &[];
        if node == 0 {
            ROOT_CHILDREN.iter().copied()
        } else {
            NO_CHILDREN.iter().copied()
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == 0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        assert_eq!((node, index), (0, 0));
        1
    }
}

impl<S: LayoutScalar> Compute<()> for Fri06C03FixedSizeTree<S> {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        if node == 0 {
            &self.root
        } else {
            &self.child_node_input
        }
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        if node == 0 {
            LayoutInputOf::box_input(self.root.clone())
        } else {
            self.child_layout_input_calls
                .set(self.child_layout_input_calls.get() + 1);
            self.child_input.clone()
        }
    }

    fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutputOf<S>) {}

    fn compute_child(
        &mut self,
        _node: Self::Node,
        _input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S, ()> {
        self.child_compute_calls += 1;
        panic!("non-box inline participants must not be recursively measured")
    }
}

fn fri06_c03_fixed_size_output<S: LayoutScalar>(
    child_input: LayoutInputOf<S>,
) -> (ComputeOutputOf<S>, usize, usize) {
    let mut tree = Fri06C03FixedSizeTree {
        root: NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(100.0)),
                PreferredSizeOf::px(S::from_f64(50.0)),
            ),
            ..NodeInputOf::default()
        },
        child_input,
        child_node_input: NodeInputOf::non_box(),
        child_layout_input_calls: Cell::new(0),
        child_compute_calls: 0,
    };
    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        ),
    )
    .expect("definite block ComputeSize succeeds");
    (
        output,
        tree.child_layout_input_calls.get(),
        tree.child_compute_calls,
    )
}

fn assert_fri06_c03_fixed_metric_participant<S: LayoutScalar>(
    input: LayoutInputOf<S>,
    expected_first_baseline: S,
    expected_last_baseline: S,
) {
    let (output, layout_input_calls, child_compute_calls) = fri06_c03_fixed_size_output(input);
    assert_eq!(
        output.size,
        Size::new(S::from_f64(100.0), S::from_f64(50.0))
    );
    assert_eq!(output.first_baselines.y, Some(expected_first_baseline));
    assert_eq!(output.last_baselines.y, Some(expected_last_baseline));
    assert!(
        layout_input_calls > 1,
        "metric participant must retain line layout"
    );
    assert_eq!(child_compute_calls, 0);
}

#[test]
fn fri06_c03_fixed_metric_text_retains_requested_baseline_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        assert_fri06_c03_fixed_metric_participant(
            LayoutInputOf::inline_text(
                InlineTextInputOf::try_new(vec![
                    ShapedInlineSegmentOf::try_new(
                        InlineSegmentId::new(1),
                        S::from_f64(9.0),
                        InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0))
                            .unwrap(),
                        BidiLevel::try_new(0).unwrap(),
                        InlineWhitespaceEdge::Preserve,
                        InlineBreakOpportunityOf::prohibited(),
                    )
                    .unwrap(),
                ])
                .unwrap(),
            ),
            S::from_f64(8.0),
            S::from_f64(8.0),
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_fixed_metric_break_retains_requested_baseline_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        assert_fri06_c03_fixed_metric_participant(
            LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(
                InlineMetricsOf::from_ascent_descent(S::from_f64(7.0), S::from_f64(3.0)).unwrap(),
            )),
            S::from_f64(7.0),
            S::from_f64(17.0),
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_fixed_metric_boundary_retains_requested_baseline_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        assert_fri06_c03_fixed_metric_participant(
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                InlineBoundaryKind::Start,
                InlineMetricsOf::from_ascent_descent(S::from_f64(6.0), S::from_f64(4.0)).unwrap(),
            )),
            S::from_f64(6.0),
            S::from_f64(6.0),
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_fixed_size_only_control_keeps_early_return_call_accounting_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let (size_only, layout_input_calls, child_compute_calls) = fri06_c03_fixed_size_output(
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                InlineBoundaryKind::Start,
                InlineMetricsOf::from_ascent_descent(S::ZERO, S::ZERO).unwrap(),
            )),
        );
        assert_eq!(
            size_only.size,
            Size::new(S::from_f64(100.0), S::from_f64(50.0))
        );
        assert_eq!(size_only.first_baselines, Point::NONE);
        assert_eq!(size_only.last_baselines, Point::NONE);
        assert_eq!(
            layout_input_calls, 1,
            "size-only control keeps the fixed-size fast path"
        );
        assert_eq!(child_compute_calls, 0);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[derive(Clone, Copy, Debug)]
enum Fri08C07T01InlineRunStart {
    Text,
    InlineBoundary,
    ExplicitLineBreak,
    InlineBox,
}

fn fri08_c07_t01_text<S: LayoutScalar>(
    segment_id: u64,
    advance: f64,
    ascent: f64,
    descent: f64,
) -> LayoutInputOf<S> {
    LayoutInputOf::inline_text(
        InlineTextInputOf::try_new(vec![
            ShapedInlineSegmentOf::try_new(
                InlineSegmentId::new(segment_id),
                S::from_f64(advance),
                InlineMetricsOf::from_ascent_descent(S::from_f64(ascent), S::from_f64(descent))
                    .unwrap(),
                BidiLevel::try_new(0).unwrap(),
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            )
            .unwrap(),
        ])
        .unwrap(),
    )
}

fn fri08_c07_t01_inline_transition_tree<S: LayoutScalar>(
    start: Fri08C07T01InlineRunStart,
) -> PublicBlockTree<S> {
    let fixed_size = |width, height| {
        Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::px(S::from_f64(height)),
        )
    };
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1, 2, 3, 4, 5, 6, 7])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_children(4, [])
        .with_children(5, [])
        .with_children(6, [])
        .with_children(7, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: fixed_size(1.0, 10.0),
                margin: Edges {
                    bottom: LengthAutoOf::px(S::from_f64(7.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: fixed_size(20.0, 12.0),
                ..NodeInputOf::default()
            },
        )
        .with_style(4, NodeInputOf::non_box())
        .with_layout_input(4, fri08_c07_t01_text(404, 10.0, 8.0, 2.0))
        .with_style(
            5,
            NodeInputOf {
                display: Display::InlineBlock,
                position: Position::Absolute,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: fixed_size(4.0, 3.0),
                ..NodeInputOf::default()
            },
        )
        .with_style(6, NodeInputOf::non_box())
        .with_layout_input(
            6,
            LayoutInputOf::line_break(LineBreakInputOf::new().hidden()),
        )
        .with_style(
            7,
            NodeInputOf {
                display: Display::Block,
                size: fixed_size(1.0, 5.0),
                margin: Edges {
                    top: LengthAutoOf::px(S::from_f64(3.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );

    tree = match start {
        Fri08C07T01InlineRunStart::Text => tree
            .with_style(3, NodeInputOf::non_box())
            .with_layout_input(3, fri08_c07_t01_text(303, 0.0, 0.0, 0.0)),
        Fri08C07T01InlineRunStart::InlineBoundary => tree
            .with_style(3, NodeInputOf::non_box())
            .with_layout_input(
                3,
                LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                    InlineBoundaryKind::Start,
                    InlineMetricsOf::from_ascent_descent(S::ZERO, S::ZERO).unwrap(),
                )),
            ),
        Fri08C07T01InlineRunStart::ExplicitLineBreak => {
            tree.with_style(3, NodeInputOf::non_box())
                .with_layout_input(
                    3,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(
                        InlineMetricsOf::from_ascent_descent(S::ZERO, S::ZERO).unwrap(),
                    )),
                )
        }
        Fri08C07T01InlineRunStart::InlineBox => tree.with_style(
            3,
            NodeInputOf {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: fixed_size(0.0, 0.0),
                ..NodeInputOf::default()
            },
        ),
    };

    tree
}

fn assert_fri08_c07_t01_inline_transition_roles<S: LayoutScalar>() {
    for start in [
        Fri08C07T01InlineRunStart::Text,
        Fri08C07T01InlineRunStart::InlineBoundary,
        Fri08C07T01InlineRunStart::ExplicitLineBreak,
        Fri08C07T01InlineRunStart::InlineBox,
    ] {
        let tree = fri08_c07_t01_inline_transition_tree(start);
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(100.0))))
                .unwrap(),
        )
        .unwrap();

        let root = public_final_output(&batch, 0);
        let prefix = public_final_output(&batch, 1);
        let float = public_final_output(&batch, 2);
        let role = public_final_output(&batch, 3);
        let text = public_final_output(&batch, 4);
        let absolute = public_final_output(&batch, 5);
        let hidden_break = public_final_output(&batch, 6);
        let trailing = public_final_output(&batch, 7);
        let text_fragment = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| entry.node() == 4)
            .unwrap()
            .fragment();

        assert_eq!(prefix.source_index, SourceIndex::ZERO, "{start:?}");
        assert_eq!(float.source_index, SourceIndex::new(1), "{start:?}");
        assert_eq!(role.source_index, SourceIndex::new(2), "{start:?}");
        assert_eq!(text.source_index, SourceIndex::new(3), "{start:?}");
        assert_eq!(absolute.source_index, SourceIndex::new(4), "{start:?}");
        assert_eq!(hidden_break.source_index, SourceIndex::new(5), "{start:?}");
        assert_eq!(trailing.source_index, SourceIndex::new(6), "{start:?}");

        assert_eq!(prefix.location, Point::ZERO, "{start:?}");
        assert_eq!(prefix.margin.bottom, S::from_f64(7.0), "{start:?}");
        assert_eq!(
            float.location,
            Point::new(S::ZERO, S::from_f64(10.0)),
            "{start:?}"
        );
        assert_eq!(
            text.location,
            Point::new(S::from_f64(20.0), S::from_f64(17.0)),
            "{start:?}"
        );
        assert_eq!(
            text.size,
            Size::new(S::from_f64(10.0), S::from_f64(10.0)),
            "{start:?}"
        );
        assert_eq!(
            absolute.location,
            Point::new(S::ZERO, S::from_f64(17.0)),
            "{start:?}"
        );
        assert_eq!(hidden_break.size, Size::ZERO, "{start:?}");
        assert_eq!(
            trailing.location,
            Point::new(S::ZERO, S::from_f64(30.0)),
            "{start:?}"
        );
        assert_eq!(trailing.margin.top, S::from_f64(3.0), "{start:?}");
        assert_eq!(
            root.size,
            Size::new(S::from_f64(100.0), S::from_f64(35.0)),
            "{start:?}"
        );
        assert_eq!(
            text_fragment.baseline(),
            Point::new(S::from_f64(20.0), S::from_f64(25.0)),
            "{start:?}"
        );
        assert_eq!(
            text_fragment.line_index(),
            usize::from(matches!(
                start,
                Fri08C07T01InlineRunStart::ExplicitLineBreak
            )),
            "{start:?}"
        );
        assert_eq!(
            root.scroll_geometry.unwrap().scrollable_overflow(),
            ScrollRectOf::try_new(
                Point::ZERO,
                Size::new(S::from_f64(100.0), S::from_f64(35.0))
            )
            .unwrap(),
            "{start:?}",
        );
    }
}

#[test]
fn fri08_c07_t01_inline_transition_all_visible_start_roles_preserve_shared_state_both_scalars() {
    assert_fri08_c07_t01_inline_transition_roles::<f32>();
    assert_fri08_c07_t01_inline_transition_roles::<f64>();
}

fn assert_fri08_c07_t01_invalid_input_ordering<S: LayoutScalar>() {
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(0, NodeInputOf::default())
        .with_style(1, NodeInputOf::default())
        .with_layout_input(1, fri08_c07_t01_text(101, 1.0, 1.0, 0.0))
        .with_style(
            2,
            NodeInputOf {
                display: Display::InlineBlock,
                ..NodeInputOf::default()
            },
        );
    let error = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(100.0))))
            .unwrap(),
    )
    .expect_err("the first invalid public input must be rejected");

    assert_eq!(error.site(), LayoutErrorSiteOf::Node(1));
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::NonBoxNodeRole {
            reason: NonBoxNodeRoleError::NonCanonicalNodeInput,
        })
    ));
}

#[test]
fn fri08_c07_t01_inline_transition_public_validation_keeps_source_error_order_both_scalars() {
    assert_fri08_c07_t01_invalid_input_ordering::<f32>();
    assert_fri08_c07_t01_invalid_input_ordering::<f64>();
}

#[test]
fn block_lays_out_atomic_inline_children_on_one_line() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(20.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 20.0));
}

#[test]
fn f64_block_layout_preserves_fractional_child_offsets() {
    let large = 16_777_217.25_f64;
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(40.0), PreferredSizeOf::px(5.25)),
                margin: Edges {
                    top: LengthAutoOf::px(large),
                    bottom: LengthAutoOf::px(0.25),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(40.0), PreferredSizeOf::px(7.5)),
                margin: Edges {
                    top: LengthAutoOf::px(0.375),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(AvailableOf::definite(100.0), AvailableOf::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the first child")
            .location,
        Point::new(0.0, large)
    );
    assert_eq!(
        tree.output(2)
            .expect("block layout must stage output for the second child")
            .location,
        Point::new(0.0, large + 5.25 + 0.375)
    );
    assert_eq!(
        tree.output(0)
            .expect("block layout must stage output for the root node")
            .size,
        Size::new(100.0, large + 5.25 + 0.375 + 7.5)
    );
}

#[test]
fn f64_block_layout_resolves_affine_values_without_narrowing() {
    let large = 16_777_217.25_f64;
    let container_width = 16_777_220.5_f64;
    let margin_left = lp64(large, 0.10);
    let width = lp64(large + 0.25, 0.50);
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(container_width), PreferredSizeOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::value(width), PreferredSizeOf::px(4.5)),
                margin: Edges {
                    left: LengthAutoOf::value(margin_left),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(
            AvailableOf::definite(container_width),
            AvailableOf::MAX_CONTENT,
        ),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the child")
            .location,
        Point::new(18_454_939.3, 0.0)
    );
    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the child")
            .size,
        Size::new(25_165_827.75, 4.5)
    );
}

#[test]
fn f64_inline_layout_preserves_large_atomic_inline_offsets() {
    let large = 16_777_217.25_f64;
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(large + 20.0), PreferredSizeOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSizeOf::px(large), PreferredSizeOf::px(10.5)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSizeOf::px(9.75), PreferredSizeOf::px(20.25)),
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(
            AvailableOf::definite(large + 20.0),
            AvailableOf::MAX_CONTENT,
        ),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the first child")
            .location,
        Point::new(0.0, 9.75)
    );
    assert_eq!(
        tree.output(2)
            .expect("block layout must stage output for the second child")
            .location,
        Point::new(large, 0.0)
    );
    assert_eq!(
        tree.output(0)
            .expect("block layout must stage output for the root node")
            .size,
        Size::new(large + 20.0, 20.25)
    );
}

#[test]
fn vertical_rl_block_places_atomic_inline_run_at_inline_start_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                border: Edges::all(Length::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(55.0, 5.0)
    );
}

#[test]
fn inline_grid_uses_grid_tracks_and_participates_as_atomic_inline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineGrid,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_grid_lanes_uses_lanes_tracks_and_participates_as_atomic_inline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineGridLanes,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineGridLanes,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn block_wraps_atomic_inline_children_between_items() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(40.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size.height, 20.0);
}

#[test]
fn block_atomic_inline_run_honors_line_break_child() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 2.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(20.0, 12.0)
    );
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 16.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 32.0));
}

#[test]
fn ordinary_block_child_receives_parent_non_horizontal_containing_flow() {
    let parent_flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                ..NodeInput::default()
            },
        )
        .style(1, NodeInput::default());

    crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(80.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::definite(80.0)),
        ),
    )
    .unwrap();

    assert!(
        tree.inputs(1)
            .iter()
            .all(|input| input.containing_flow_axes() == parent_flow_axes)
    );
}

#[test]
fn block_line_break_conversion_with_metadata_preserves_current_output() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_direction(Direction::Rtl)
                .with_writing_mode(WritingMode::HorizontalTb)
                .with_vertical_align(VerticalAlign::Top)
                .with_clear(Clear::Both)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.inputs(2), &[]);
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 18.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 48.0));
}

#[test]
fn block_line_break_metrics_create_empty_line_height() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 15.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::default()
            },
        )
        .children(0, [1, 2])
        .line_break(1, LineBreakInput::new().with_metrics(metrics))
        .line_break(2, LineBreakInput::new().with_metrics(metrics));

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size.height, 60.0);
    assert_eq!(output.first_baselines.y, Some(15.0));
    assert_eq!(output.last_baselines.y, Some(55.0));
    assert_eq!(tree.layout(1).unwrap().location.y, 15.0);
    assert_eq!(tree.layout(2).unwrap().location.y, 35.0);
}

#[test]
fn block_inline_boundaries_are_reported_as_zero_size_inline_controls() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(18.0, 13.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.inputs(1), &[]);
    assert_eq!(tree.inputs(3), &[]);
    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 13.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 3.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 13.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(20.0, 1.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 18.0));
}

#[test]
fn block_inline_boundaries_before_overwide_first_inline_block_do_not_create_leading_line() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(50.0, 35.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(40.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(20.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 35.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 25.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(20.0, 50.0));
}

#[test]
fn vertical_block_inline_boundaries_use_parent_flow() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalRl),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalRl),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(66.0, 0.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(66.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(66.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
}

#[test]
fn vertical_lr_block_inline_boundaries_use_parent_flow() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalLr),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalLr),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(14.0, 0.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(4.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(14.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
}

#[test]
fn hidden_line_break_does_not_split_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new().hidden())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(tree.inputs(2), &[]);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 10.0));
}

#[test]
fn block_atomic_inline_run_never_computes_line_break_as_box() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(tree.inputs(2), &[]);
}

#[test]
fn vertical_rl_line_break_is_laid_out_as_zero_size_inline_control() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(12.0), PreferredSize::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(66.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().location.x, 46.0);
}

#[test]
fn vertical_lr_line_break_is_laid_out_as_zero_size_inline_control() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalLr)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSize::px(12.0), PreferredSize::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(14.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().location.x, 22.0);
}

#[test]
fn vertical_line_break_clear_is_accepted_without_active_exclusions() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            1,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .with_clear(Clear::Both),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
}

#[test]
#[should_panic(expected = "line-break flow must match containing inline flow")]
fn vertical_parent_rejects_clear_even_when_line_break_input_defaults_horizontal() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(1, LineBreakInput::new().with_clear(Clear::Both));

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "line-break flow must match containing inline flow")]
fn vertical_parent_rejects_default_line_break_flow_until_input_is_layout_ready() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Ltr,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(1, LineBreakInput::new());

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "inline boundary flow must match containing inline flow")]
fn vertical_parent_rejects_default_inline_boundary_flow_until_input_is_layout_ready() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Ltr,
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
fn hidden_vertical_line_break_does_not_create_inline_control() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .hidden(),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(12.0), PreferredSize::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(68.0, 30.0)
    );
}

fn inline_break_clear_tree(
    clear: Clear,
    float_side: Float,
) -> crate::test_support::layout_tree::OracleTree {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: float_side,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(clear)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
}

#[test]
fn line_break_clear_left_moves_following_inline_segment_below_left_float() {
    let mut tree = inline_break_clear_tree(Clear::Left, Float::Left);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(100.0, 10.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_right_moves_following_inline_segment_below_right_float() {
    let mut tree = inline_break_clear_tree(Clear::Right, Float::Right);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_both_uses_greater_left_or_right_float_bottom() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4, 5])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(70.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            4,
            LineBreakInput::new()
                .with_clear(Clear::Both)
                .with_metrics(metrics),
        )
        .style(
            5,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(60.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(80.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(5).unwrap().location,
        Point::new(0.0, 70.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 80.0));
}

#[test]
fn line_break_clear_at_run_end_moves_following_block_below_float() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(Clear::Left)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(25.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(100.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 60.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 70.0));
}

#[test]
fn line_break_clear_left_ignores_right_float_and_preserves_alignment() {
    let mut tree = inline_break_clear_tree(Clear::Left, Float::Right).style(
        0,
        NodeInput {
            display: Display::Block,
            text_align: TextAlign::LegacyRight,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(100.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(120.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(105.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 50.0));
}

#[test]
fn line_break_clear_right_ignores_left_float_and_preserves_alignment() {
    let mut tree = inline_break_clear_tree(Clear::Right, Float::Left).style(
        0,
        NodeInput {
            display: Display::Block,
            text_align: TextAlign::LegacyCenter,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(130.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(150.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(133.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 50.0));
}

#[test]
fn line_break_clear_that_is_noop_after_line_height_preserves_alignment() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(Clear::Left)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(15.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(180.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(200.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(185.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_none_preserves_existing_single_run_layout_near_float() {
    let mut tree = inline_break_clear_tree(Clear::None, Float::Left);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(100.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(80.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 50.0));
}

#[test]
fn block_min_content_atomic_inline_run_uses_max_item_advance() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(40.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(60.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(60.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 20.0)
    );
}

#[test]
fn atomic_inline_auto_margins_resolve_to_zero() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                margin: Edges {
                    left: LengthAuto::AUTO,
                    right: LengthAuto::AUTO,
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    let child = tree.final_layout(1).unwrap();
    assert_eq!(child.location, Point::new(0.0, 0.0));
    assert_eq!(child.margin.left, 0.0);
    assert_eq!(child.margin.right, 0.0);
}

#[test]
fn inline_block_intrinsic_width_shrink_wraps_children() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(70.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(70.0, 20.0));
    assert_eq!(tree.final_layout(0).unwrap().size.width, 70.0);
}

#[test]
fn inline_block_uses_bottom_synthesized_baseline_when_child_has_no_baseline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_block_uses_inner_first_baseline_for_atomic_alignment() {
    let measured_inline_block = ComputeOutput::from_sizes_and_baselines(
        Size::new(10.0, 30.0),
        Size::new(10.0, 30.0),
        crate::Baselines {
            first: Point::new(None, Some(5.0)),
            last: Point::new(None, Some(25.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .measure(1, measured_inline_block);

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.y, 20.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
    assert_eq!(tree.final_layout(0).unwrap().size.height, 50.0);
}

#[test]
fn inline_block_keeps_child_margins_inside_atomic_wrapper() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                margin: Edges {
                    top: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(20.0, 15.0));
    assert_eq!(tree.final_layout(2).unwrap().location.y, 5.0);
    assert_eq!(tree.final_layout(0).unwrap().size.height, 15.0);
}

#[test]
fn inline_grid_can_host_subgrid_descendant() {
    let subgrid_track = || {
        TrackComponent::Subgrid(crate::SubgridTrack {
            name_components: Vec::new(),
        })
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::InlineGrid,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                grid_template_columns: vec![TrackComponent::px(80.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![subgrid_track()],
                grid_template_rows: vec![subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(80.0, 30.0));
}

#[test]
fn block_positions_block_children_around_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(10.0)),
                margin: Edges {
                    bottom: LengthAuto::px(7.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(15.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(10.0)),
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 27.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(10.0, 17.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 35.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 45.0));
}

#[test]
fn block_hidden_and_absolute_children_do_not_split_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::None,
                float: Float::Left,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                position: Position::Absolute,
                float: Float::Left,
                size: Size::new(PreferredSize::px(5.0), PreferredSize::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(10.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 10.0));
}

#[test]
fn block_rtl_atomic_inline_run_places_items_from_right_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn block_rtl_atomic_inline_run_mirrors_line_break_output_x() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new().with_direction(Direction::Rtl))
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 2.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 12.0)
    );
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(70.0, 18.0)
    );
}

#[test]
fn block_legacy_right_rtl_aligns_atomic_inline_run_to_physical_right_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn block_atomic_inline_run_alignment_uses_resolved_inner_width() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                min_size: Size::new(MinSize::px(100.0), MinSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.x, 25.0);
}

#[test]
fn block_legacy_center_aligns_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(25.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(45.0, 0.0)
    );
}

#[test]
fn block_inline_run_content_size_includes_visible_overflow_and_relative_inset() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                inset: Edges {
                    left: LengthAuto::px(15.0),
                    top: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(80.0, 30.0)),
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.content_size, Size::new(95.0, 35.0));
}

#[test]
fn block_inline_run_content_size_accounts_for_negative_relative_inset_after_content() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                inset: Edges {
                    top: LengthAuto::px(-5.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .measure(
            2,
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 30.0)),
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.content_size.height, 45.0);
}

#[test]
fn block_reports_inline_run_first_and_last_baselines() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::definite(100.0)),
            ),
        )
        .unwrap();

    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(20.0));
}

#[test]
fn block_reports_inline_run_baseline_including_padding() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                padding: Edges {
                    top: Length::px(10.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.first_baselines.y, Some(30.0));
    assert_eq!(output.last_baselines.y, Some(30.0));
}

#[test]
fn block_definite_compute_size_keeps_inline_run_baselines() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(20.0));
}

#[test]
fn block_definite_compute_size_keeps_block_child_baselines() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        crate::Baselines {
            first: Point::new(None, Some(7.0)),
            last: Point::new(None, Some(17.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .measure(1, child_output);

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(7.0));
    assert_eq!(output.last_baselines.y, Some(17.0));
}

fn assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis<S: LayoutScalar>(
    writing_mode: WritingMode,
) where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let child_output = ComputeOutputOf::from_sizes_and_baselines(
        Size::new(S::from_f64(70.0), S::from_f64(110.0)),
        Size::new(S::from_f64(70.0), S::from_f64(110.0)),
        BaselinesOf {
            first: Point::new(Some(S::from_f64(7.0)), None),
            last: Point::new(Some(S::from_f64(11.0)), None),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(120.0)),
                    PreferredSizeOf::AUTO,
                ),
                padding: Edges {
                    top: LengthOf::px(S::from_f64(5.0)),
                    left: LengthOf::px(S::from_f64(3.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode,
                margin: Edges {
                    top: LengthAutoOf::px(S::from_f64(17.0)),
                    left: LengthAutoOf::px(S::from_f64(11.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .measure(1, child_output);

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(120.0)), Some(S::from_f64(160.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(120.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("block layout succeeds");

    assert_eq!(
        tree.layout(1).expect("child layout is staged").location,
        Point::new(S::from_f64(14.0), S::from_f64(22.0))
    );
    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(21.0)), None)
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(S::from_f64(25.0)), None)
    );
}

#[test]
fn orthogonal_baseline_block_translation_uses_physical_x_for_f32() {
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f32>(
        WritingMode::VerticalRl,
    );
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn orthogonal_baseline_block_translation_uses_physical_x_for_f64() {
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f64>(
        WritingMode::VerticalRl,
    );
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f64>(
        WritingMode::SidewaysLr,
    );
}

fn assert_block_aggregates_physical_baselines_on_both_axes<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(140.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                writing_mode: WritingMode::VerticalRl,
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(17.0)),
                    LengthAutoOf::px(S::from_f64(5.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(19.0)),
                    LengthAutoOf::px(S::from_f64(7.0)),
                    LengthAutoOf::px(S::from_f64(23.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            1,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(20.0)),
                Size::new(S::from_f64(70.0), S::from_f64(20.0)),
                BaselinesOf {
                    first: Point::new(Some(S::from_f64(7.0)), None),
                    last: Point::new(Some(S::from_f64(11.0)), None),
                },
            ),
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(30.0), S::from_f64(40.0)),
                Size::new(S::from_f64(30.0), S::from_f64(40.0)),
                BaselinesOf {
                    first: Point::new(None, Some(S::from_f64(9.0))),
                    last: Point::new(None, Some(S::from_f64(15.0))),
                },
            ),
        );

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(140.0)), Some(S::from_f64(200.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(140.0)),
                AvailableOf::definite(S::from_f64(200.0)),
            ),
        ),
    )
    .expect("block layout succeeds");
    let first_child = tree.layout(1).expect("first child layout is staged");
    let second_child = tree.layout(2).expect("second child layout is staged");

    assert_eq!(
        output.first_baselines,
        Point::new(
            Some(first_child.location.x + S::from_f64(7.0)),
            Some(second_child.location.y + S::from_f64(9.0)),
        )
    );
    assert_eq!(
        output.last_baselines,
        Point::new(
            Some(first_child.location.x + S::from_f64(11.0)),
            Some(second_child.location.y + S::from_f64(15.0)),
        )
    );
}

#[test]
fn physical_baseline_block_aggregates_both_axes_for_f32() {
    assert_block_aggregates_physical_baselines_on_both_axes::<f32>();
}

#[test]
fn physical_baseline_block_aggregates_both_axes_for_f64() {
    assert_block_aggregates_physical_baselines_on_both_axes::<f64>();
}

fn assert_block_preserves_a_child_y_baseline<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(120.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                writing_mode: WritingMode::HorizontalTb,
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(17.0)),
                    LengthAutoOf::px(S::from_f64(5.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            1,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(40.0)),
                Size::new(S::from_f64(70.0), S::from_f64(40.0)),
                BaselinesOf {
                    first: Point::new(None, Some(S::from_f64(9.0))),
                    last: Point::new(None, Some(S::from_f64(15.0))),
                },
            ),
        );

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(120.0)), Some(S::from_f64(160.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(
                AvailableOf::definite(S::from_f64(120.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("block layout succeeds");
    let child = tree.layout(1).expect("child layout is staged");

    assert_eq!(
        output.first_baselines,
        Point::new(None, Some(child.location.y + S::from_f64(9.0)))
    );
    assert_eq!(
        output.last_baselines,
        Point::new(None, Some(child.location.y + S::from_f64(15.0)))
    );
}

#[test]
fn physical_baseline_block_preserves_y_for_f32() {
    assert_block_preserves_a_child_y_baseline::<f32>();
}

#[test]
fn physical_baseline_block_preserves_y_for_f64() {
    assert_block_preserves_a_child_y_baseline::<f64>();
}

#[test]
fn block_definite_compute_size_keeps_non_empty_flex_child_baselines() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        crate::Baselines {
            first: Point::new(None, Some(9.0)),
            last: Point::new(None, Some(19.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT)
        .measure(1, child_output);

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(9.0));
    assert_eq!(output.last_baselines.y, Some(19.0));
}
