use super::fixtures::{
    CalcBlockTree, PublicBlockTree, computed_overflow, fri05_c03_block_union_content_size,
    fri06_atomic_participation, lp, public_final_output, scalar_value,
};
use super::*;

type ScrollBlockTree = OracleTree;

fn assert_fri08_c07_t05_geometry_error_input_fields<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let context = ContainingLayoutContext::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ParentFormattingContext::NoParent,
    );

    for run_mode in [RunMode::PerformRootLayout, RunMode::PerformLayout] {
        let input = fri06_mr02_geometry_error_input::<S>(run_mode);

        assert_eq!(input.run_mode(), run_mode);
        assert_eq!(input.sizing_mode(), SizingMode::InherentSize);
        assert_eq!(input.requested_axis(), RequestedAxis::Both);
        assert_eq!(input.known(), Size::NONE);
        assert_eq!(input.parent(), Size::splat(Some(largest)));
        assert_eq!(input.containing_layout_context(), context);
        assert_eq!(
            input.available(),
            Size::splat(AvailableOf::definite(largest))
        );
        assert_eq!(
            input.settled_auto_scrollbars(),
            crate::scroll::SettledAutoScrollbarState::INITIAL
        );
        assert_eq!(
            input.containing_auto_scrollbar_pass(),
            crate::scroll::SettledAutoScrollbarState::INITIAL
        );
    }
}

#[test]
fn fri08_c07_t05_scroll_fixture_selects_scalar_and_preserves_every_compute_input_field() {
    assert_eq!(fri06_mr02_geometry_error_largest_finite::<f32>(), f32::MAX);
    assert_eq!(fri06_mr02_geometry_error_largest_finite::<f64>(), f64::MAX);
    assert_fri08_c07_t05_geometry_error_input_fields::<f32>();
    assert_fri08_c07_t05_geometry_error_input_fields::<f64>();
}

fn assert_fri06_mr02_geometry_error_block_own<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite();
    let style = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(largest), PreferredSizeOf::px(S::ONE)),
        padding: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        border: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        ..NodeInputOf::default()
    };

    for (run_mode, operation, invariant) in [
        (
            RunMode::PerformRootLayout,
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        (
            RunMode::PerformLayout,
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        ),
    ] {
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(7, [])
            .style(7, style.clone());
        let error = crate::compute_block(&mut tree, 7, fri06_mr02_geometry_error_input(run_mode))
            .expect_err("overflowing block geometry must fail");

        fri06_mr02_geometry_error_assert(error, LayoutErrorSiteOf::Node(7), operation, invariant);
    }
}

fn assert_fri06_mr02_geometry_error_block_child<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(7, [11])
        .children(11, [])
        .style(
            7,
            NodeInputOf {
                display: Display::Block,
                size: size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .style(11, NodeInputOf::default())
        .measure(
            11,
            ComputeOutputOf::from_sizes(
                Size::new(S::from_f64(10.0), S::from_f64(10.0)),
                Size::splat(S::INFINITY),
            ),
        );
    let error = crate::compute_block(
        &mut tree,
        7,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            size.map(Some),
            size.map(Some),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            size.map(AvailableOf::definite),
        ),
    )
    .expect_err("invalid retained child geometry must fail");

    fri06_mr02_geometry_error_assert(
        error,
        LayoutErrorSiteOf::ContainerSubject {
            container: 7,
            subject: 11,
        },
        LayoutOperation::ChildLayout,
        LayoutInternalInvariant::InvalidBlockScrollGeometry,
    );
}

fn fri06_mr02_geometry_error_inline_container_style<S: LayoutScalar>() -> NodeInputOf<S> {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(largest),
            PreferredSizeOf::px(S::from_f64(10.0)),
        ),
        padding: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        ..NodeInputOf::default()
    }
}

fn assert_fri06_mr02_geometry_error_block_inline_child<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(7, [11])
        .children(11, [])
        .style(7, fri06_mr02_geometry_error_inline_container_style())
        .style(
            11,
            NodeInputOf {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSizeOf::px(largest), PreferredSizeOf::px(S::ONE)),
                ..NodeInputOf::default()
            },
        )
        .measure(
            11,
            ComputeOutputOf::from_outer_size(Size::new(largest, S::ONE)),
        );
    let error = crate::compute_block(
        &mut tree,
        7,
        fri06_mr02_geometry_error_input(RunMode::PerformLayout),
    )
    .expect_err("overflowing atomic-inline fragment geometry must fail first");

    fri06_mr02_geometry_error_assert(
        error,
        LayoutErrorSiteOf::ContainerSubject {
            container: 7,
            subject: 11,
        },
        LayoutOperation::ChildLayout,
        LayoutInternalInvariant::InvalidBlockScrollGeometry,
    );
    assert_eq!(tree.inputs(11).len(), 1);
    assert_eq!(largest + largest, S::INFINITY);
}

fn assert_fri08_c07_t02_scroll_source_block_paths<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let container_size = Size::new(scalar(100.0), scalar(80.0));
    let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::End);
    let scroll_margin =
        ScrollMarginOf::try_new(scalar(1.0), scalar(-2.0), scalar(3.0), scalar(-4.0)).unwrap();
    let scroll_padding = ScrollPaddingOf::new(
        ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(2.0)).unwrap()),
        ScrollPaddingValueOf::AUTO,
        ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(4.0)).unwrap()),
        ScrollPaddingValueOf::AUTO,
    );
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                overflow_clip_margin: OverflowClipMarginOf::try_new(
                    OverflowClipBox::PaddingBox,
                    scalar(3.0),
                )
                .unwrap(),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(6.0)).unwrap(),
                size: container_size.map(PreferredSizeOf::px),
                padding: Edges::all(LengthOf::px(scalar(2.0))),
                scroll_padding,
                scroll_snap_type: ScrollSnapType::Enabled {
                    axis: ScrollSnapAxis::Both,
                    strictness: ScrollSnapStrictness::Mandatory,
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: Size::new(
                    PreferredSizeOf::px(scalar(130.0)),
                    PreferredSizeOf::px(scalar(90.0)),
                ),
                scroll_margin,
                scroll_snap_align: snap_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(5.0)).unwrap(),
                size: Size::new(
                    PreferredSizeOf::px(scalar(40.0)),
                    PreferredSizeOf::px(scalar(30.0)),
                ),
                scroll_margin,
                scroll_snap_align: snap_align,
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes(Size::new(scalar(40.0), scalar(30.0)), Size::ZERO),
        );

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            container_size.map(Some),
            container_size.map(Some),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            container_size.map(AvailableOf::definite),
        ),
    )
    .unwrap();

    let container = output.scroll_geometry.unwrap();
    assert_eq!(container.flow_axes(), flow_axes);
    assert_eq!(container.resolved_scroll_padding().top, scalar(2.0));
    assert_eq!(container.resolved_scroll_padding().bottom, scalar(4.0));
    assert!(container.overflow_clip().x().is_some());
    assert!(container.overflow_clip().y().is_some());
    assert_ne!(container.scrollbar_size(), Size::ZERO);
    assert_eq!(
        container.scroll_snap_type(),
        ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Both,
            strictness: ScrollSnapStrictness::Mandatory,
        }
    );

    let existing = tree.layout(1).unwrap();
    let existing_geometry = existing.scroll_geometry.unwrap();
    assert_eq!(existing_geometry.border_box().size(), existing.size);
    assert_eq!(existing_geometry.target().flow_axes(), flow_axes);
    assert_eq!(existing_geometry.target().scroll_margin(), scroll_margin);
    assert_eq!(existing_geometry.target().snap_align(), snap_align);
    assert_eq!(
        existing_geometry.target().snap_stop(),
        ScrollSnapStop::Always
    );

    let reconstructed = tree.layout(2).unwrap();
    let reconstructed_geometry = reconstructed.scroll_geometry.unwrap();
    assert_eq!(reconstructed_geometry.flow_axes(), flow_axes);
    assert_eq!(
        reconstructed_geometry.target().border_box().size(),
        reconstructed.size
    );
    assert_eq!(
        reconstructed_geometry.target().scroll_margin(),
        scroll_margin
    );
    assert_eq!(reconstructed_geometry.target().snap_align(), snap_align);
    let range = reconstructed_geometry.physical_range();
    assert_eq!(range.x().minimum(), S::ZERO);
    assert_eq!(range.x().maximum(), S::ZERO);
    assert_eq!(range.y().minimum(), S::ZERO);
    assert_eq!(range.y().maximum(), S::ZERO);
}

#[test]
fn fri08_c07_t02_scroll_source_block_preserves_existing_reconstruction_and_gutter_policy() {
    assert_fri08_c07_t02_scroll_source_block_paths::<f32>();
    assert_fri08_c07_t02_scroll_source_block_paths::<f64>();
}

#[test]
fn fri08_c07_t02_scroll_source_block_preserves_caller_local_errors() {
    assert_fri06_mr02_geometry_error_block_own::<f32>();
    assert_fri06_mr02_geometry_error_block_own::<f64>();
    assert_fri06_mr02_geometry_error_block_child::<f32>();
    assert_fri06_mr02_geometry_error_block_child::<f64>();
}

#[test]
fn fri06_mr02_geometry_error_block_own_preserves_root_and_child_mapping_both_scalars() {
    assert_fri06_mr02_geometry_error_block_own::<f32>();
    assert_fri06_mr02_geometry_error_block_own::<f64>();
}

#[test]
fn fri06_mr02_geometry_error_block_child_preserves_container_subject_both_scalars() {
    assert_fri06_mr02_geometry_error_block_child::<f32>();
    assert_fri06_mr02_geometry_error_block_child::<f64>();
}

#[test]
fn fri06_mr02_geometry_error_block_inline_child_preserves_subject_both_scalars() {
    assert_fri06_mr02_geometry_error_block_inline_child::<f32>();
    assert_fri06_mr02_geometry_error_block_inline_child::<f64>();
}

#[test]
fn fri08_c07_t05_scroll_fixture_block_assertion_preserves_error_identity() {
    assert_fri06_mr02_geometry_error_block_own::<f32>();
    assert_fri06_mr02_geometry_error_block_own::<f64>();
    assert_fri06_mr02_geometry_error_block_child::<f32>();
    assert_fri06_mr02_geometry_error_block_child::<f64>();
    assert_fri06_mr02_geometry_error_block_inline_child::<f32>();
    assert_fri06_mr02_geometry_error_block_inline_child::<f64>();
}

fn fri06_mr02_scroll_padding_cases<S: LayoutScalar>() -> [(ScrollPaddingOf<S>, Edges<S>); 2] {
    let [first, second] = scroll_padding_inputs();

    [
        (
            first,
            Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO),
        ),
        (
            second,
            Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
        ),
    ]
}

fn assert_fri06_mr02_scroll_padding_block<S: LayoutScalar>() {
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    for (scroll_padding, expected) in fri06_mr02_scroll_padding_cases() {
        let style = NodeInputOf::<S> {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(size.width),
                PreferredSizeOf::px(size.height),
            ),
            scroll_padding,
            ..NodeInputOf::default()
        };
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [])
            .style(0, style);
        let output = crate::compute_block(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                size.map(Some),
                size.map(Some),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                size.map(AvailableOf::definite),
            ),
        )
        .expect("block scroll-padding characterization succeeds");
        let geometry = output
            .scroll_geometry
            .expect("performed block layout emits geometry");

        assert_eq!(geometry.resolved_scroll_padding(), expected);
    }
}

#[test]
fn fri06_mr02_scroll_padding_block_preserves_auto_and_value_on_each_physical_edge() {
    assert_fri06_mr02_scroll_padding_block::<f32>();
    assert_fri06_mr02_scroll_padding_block::<f64>();
}

#[test]
fn fri08_c07_t05_scroll_fixture_block_rows_preserve_exact_auto_and_value_edges() {
    fn assert_rows<S: LayoutScalar>() {
        assert_scroll_padding_inputs_exact::<S>();
        assert_eq!(
            fri06_mr02_scroll_padding_cases::<S>().map(|(_, expected)| expected),
            [
                Edges::new(S::from_f64(11.0), S::ZERO, S::from_f64(33.0), S::ZERO,),
                Edges::new(S::ZERO, S::from_f64(22.0), S::ZERO, S::from_f64(44.0)),
            ]
        );
    }

    assert_rows::<f32>();
    assert_rows::<f64>();
}

#[test]
fn block_child_context_is_complete_for_layout_sizing_and_absolute_paths() {
    assert_block_child_context_is_complete::<f32>();
    assert_block_child_context_is_complete::<f64>();
}

fn assert_block_child_context_is_complete<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let expected =
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::BlockFlow);

    for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalRl,
                    direction: Direction::Rtl,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .style(1, NodeInputOf::default())
            .style(
                2,
                NodeInputOf {
                    position: Position::Absolute,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(30.0)),
                        PreferredSizeOf::px(S::from_f64(12.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .measure(
                1,
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(40.0), S::from_f64(20.0))),
            )
            .measure(
                2,
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(30.0), S::from_f64(12.0))),
            );

        crate::compute_block(
            &mut tree,
            0,
            ComputeInputOf::for_child(
                run_mode,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(S::from_f64(300.0)), Some(S::from_f64(240.0))),
                crate::ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::splat(AvailableOf::definite(S::from_f64(300.0))),
            ),
        )
        .expect("block context capture layout succeeds");

        let normal_inputs = tree.inputs(1);
        assert!(
            !normal_inputs.is_empty(),
            "block must request its in-flow child"
        );
        assert!(
            normal_inputs
                .iter()
                .all(|input| input.containing_layout_context() == expected),
            "every block in-flow request must use the parent axes and BlockFlow role: {normal_inputs:#?}"
        );

        if run_mode == RunMode::ComputeSize {
            assert!(
                normal_inputs.iter().any(|input| {
                    input.run_mode() == RunMode::ComputeSize
                        && input.sizing_mode() == SizingMode::InherentSize
                }),
                "block intrinsic sizing must request the child through the complete context"
            );
        } else {
            assert!(
                normal_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::PerformLayout),
                "block normal layout must request the child through the complete context"
            );
            let absolute_inputs = tree.inputs(2);
            assert!(
                absolute_inputs
                    .iter()
                    .any(|input| input.run_mode() == RunMode::PerformLayout),
                "block absolute scheduling must request the child"
            );
            assert!(
                absolute_inputs
                    .iter()
                    .all(|input| input.containing_layout_context() == expected),
                "every block absolute request must use the parent axes and BlockFlow role: {absolute_inputs:#?}"
            );
        }
    }
}

fn assert_positive_physical_range(range: PhysicalScrollRange, maximum: Size) {
    assert_eq!(range.x().minimum(), 0.0);
    assert_eq!(range.x().maximum(), maximum.width);
    assert_eq!(range.y().minimum(), 0.0);
    assert_eq!(range.y().maximum(), maximum.height);
}

fn fri04_c03_block_positioned_value<S: LayoutScalar>(value: f64) -> SizingCalculationOf<S> {
    SizingCalculationOf::value(
        LengthPercentageOf::px(scalar_value(value)).expect("test sizing value is finite"),
    )
}

fn fri04_c03_block_positioned_nested<S: LayoutScalar>(
    minimum: f64,
    preferred: f64,
    maximum: f64,
) -> SizingCalculationOf<S> {
    let preferred = SizingCalculationOf::max(vec![
        fri04_c03_block_positioned_value(preferred),
        SizingCalculationOf::min(vec![
            fri04_c03_block_positioned_value(preferred - 5.0),
            fri04_c03_block_positioned_value(preferred + 5.0),
        ])
        .expect("nested minimum is nonempty"),
    ])
    .expect("nested maximum is nonempty");
    SizingCalculationOf::clamp(
        Some(fri04_c03_block_positioned_value(minimum)),
        preferred,
        Some(fri04_c03_block_positioned_value(maximum)),
    )
}

#[test]
fn fri04_c03_block_positioned_ordinary_block_consumes_nested_constraints_and_non_negative_results()
{
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(160.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        20.0, 80.0, 120.0,
                    )),
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        20.0, 70.0, 120.0,
                    )),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_block_positioned_nested(40.0, 90.0, 110.0)),
                    MinSize::calculation(fri04_c03_block_positioned_nested(30.0, 60.0, 90.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_block_positioned_nested(30.0, 85.0, 100.0)),
                    MaxSize::calculation(fri04_c03_block_positioned_nested(30.0, 65.0, 100.0)),
                ),
                ..NodeInput::default()
            },
        )
        .with_style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        -40.0, -20.0, -10.0,
                    )),
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        -30.0, -15.0, -5.0,
                    )),
                ),
                ..NodeInput::default()
            },
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(300.0)))
            .expect("valid viewport"),
    )
    .expect("ordinary block calculations resolve");

    assert_eq!(public_final_output(&batch, 1).size, Size::new(90.0, 65.0));
    assert_eq!(public_final_output(&batch, 2).size, Size::ZERO);
}

#[test]
fn fri04_c03_block_positioned_absolute_consumes_nested_properties_and_inset_derived_sizing() {
    let absolute =
        |size: Size<PreferredSize>, min_size: Size<MinSize>, max_size: Size<MaxSize>, inset| {
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                size,
                min_size,
                max_size,
                inset,
                ..NodeInput::default()
            }
        };
    let nested_size = Size::new(
        PreferredSize::calculation(fri04_c03_block_positioned_nested(20.0, 80.0, 120.0)),
        PreferredSize::calculation(fri04_c03_block_positioned_nested(20.0, 70.0, 120.0)),
    );
    let nested_min = Size::new(
        MinSize::calculation(fri04_c03_block_positioned_nested(30.0, 60.0, 90.0)),
        MinSize::calculation(fri04_c03_block_positioned_nested(30.0, 50.0, 80.0)),
    );
    let nested_max = Size::new(
        MaxSize::calculation(fri04_c03_block_positioned_nested(40.0, 75.0, 100.0)),
        MaxSize::calculation(fri04_c03_block_positioned_nested(40.0, 65.0, 90.0)),
    );
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2, 3])
        .with_children(1, [])
        .with_children(2, [])
        .with_children(3, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(160.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            absolute(
                nested_size,
                nested_min.clone(),
                nested_max.clone(),
                Edges::all(LengthAuto::AUTO),
            ),
        )
        .with_style(
            2,
            absolute(
                Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
                nested_min,
                nested_max,
                Edges {
                    top: LengthAuto::px(10.0),
                    right: LengthAuto::px(20.0),
                    bottom: LengthAuto::px(10.0),
                    left: LengthAuto::px(20.0),
                },
            ),
        )
        .with_style(
            3,
            absolute(
                Size::new(
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        -40.0, -20.0, -10.0,
                    )),
                    PreferredSize::calculation(fri04_c03_block_positioned_nested(
                        -30.0, -15.0, -5.0,
                    )),
                ),
                Size::new(MinSize::AUTO, MinSize::AUTO),
                Size::new(MaxSize::NONE, MaxSize::NONE),
                Edges::all(LengthAuto::AUTO),
            ),
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(300.0)))
            .expect("valid viewport"),
    )
    .expect("positioned calculations resolve");

    assert_eq!(public_final_output(&batch, 1).size, Size::new(75.0, 65.0));
    assert_eq!(public_final_output(&batch, 2).size, Size::new(75.0, 65.0));
    assert_eq!(
        public_final_output(&batch, 2).location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(public_final_output(&batch, 3).size, Size::ZERO);
}

#[test]
fn fri04_c03_block_positioned_compute_size_preserves_missing_basis_as_indefinite() {
    let percentage = SizingCalculation::max(vec![
        fri04_c03_block_positioned_value(10.0),
        SizingCalculation::value(
            LengthPercentageOf::from_percent_fraction(0.5).expect("finite percentage"),
        ),
    ])
    .expect("nested maximum is nonempty");
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(
                    PreferredSize::calculation(percentage.clone()),
                    PreferredSize::calculation(percentage.clone()),
                ),
                min_size: Size::new(
                    MinSize::calculation(percentage.clone()),
                    MinSize::calculation(percentage.clone()),
                ),
                max_size: Size::new(
                    MaxSize::calculation(percentage.clone()),
                    MaxSize::calculation(percentage),
                ),
                ..NodeInput::default()
            },
        )
        .measure(1, ComputeOutput::from_outer_size(Size::new(30.0, 20.0)));

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .expect("intrinsic block sizing retains the missing-basis fallback");

    assert_eq!(output.size, Size::new(30.0, 20.0));
    assert!(
        tree.inputs(1).iter().any(|input| {
            input.run_mode() == RunMode::ComputeSize && input.parent() == Size::NONE
        })
    );
}

#[test]
fn fri04_c03_block_positioned_invalid_numeric_propagates_from_both_consumers() {
    let invalid = || {
        SizingCalculation::min(vec![
            SizingCalculation::value(
                LengthPercentageOf::from_coefficients(f32::MAX, 1.0)
                    .expect("finite overflowing coefficients"),
            ),
            fri04_c03_block_positioned_value(10.0),
        ])
        .expect("nested minimum is nonempty")
    };

    for position in [Position::Relative, Position::Absolute] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, NodeInput::default())
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    position,
                    size: Size::new(
                        PreferredSize::calculation(invalid()),
                        PreferredSize::px(10.0),
                    ),
                    ..NodeInput::default()
                },
            );
        let request = LayoutRootRequest::viewport(Size::new(
            Available::definite(f32::MAX),
            Available::definite(80.0),
        ))
        .expect("largest finite viewport is valid");

        let error = compute_layout(&tree, 0, request)
            .expect_err("invalid numeric sizing must return no completed batch");
        assert_eq!(error.site(), LayoutErrorSite::Node(1));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert_eq!(
            error.kind(),
            &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
                value: f32::INFINITY,
            })
        );
    }
}

enum Fri04C04SizingValue {
    Preferred(PreferredSize),
    Minimum(MinSize),
    Maximum(MaxSize),
}

fn fri04_c04_leaf_block_positioned_style(
    value: Fri04C04SizingValue,
    position: Position,
    axis: PhysicalAxis,
) -> NodeInput {
    let mut style = NodeInput {
        display: Display::Block,
        position,
        ..NodeInput::default()
    };
    match (value, axis) {
        (Fri04C04SizingValue::Preferred(value), PhysicalAxis::Horizontal) => {
            style.size.width = value;
        }
        (Fri04C04SizingValue::Preferred(value), PhysicalAxis::Vertical) => {
            style.size.height = value;
        }
        (Fri04C04SizingValue::Minimum(value), PhysicalAxis::Horizontal) => {
            style.min_size.width = value;
        }
        (Fri04C04SizingValue::Minimum(value), PhysicalAxis::Vertical) => {
            style.min_size.height = value;
        }
        (Fri04C04SizingValue::Maximum(value), PhysicalAxis::Horizontal) => {
            style.max_size.width = value;
        }
        (Fri04C04SizingValue::Maximum(value), PhysicalAxis::Vertical) => {
            style.max_size.height = value;
        }
    }
    style
}

fn fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
    value: Fri04C04SizingValue,
    property: SizingProperty,
    behavior: SizingBehavior,
    axis: PhysicalAxis,
    position: Position,
) {
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(200.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            fri04_c04_leaf_block_positioned_style(value, position, axis),
        );
    let error = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(200.0)))
            .expect("valid viewport"),
    )
    .expect_err("later-owned block sizing must be rejected");
    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected sizing capability, got {:?}", error.kind());
    };
    assert_eq!(
        (
            unsupported.property(),
            unsupported.behavior(),
            unsupported.algorithm(),
            unsupported.axis(),
        ),
        (
            property,
            behavior,
            if position == Position::Absolute {
                SizingAlgorithm::Positioned
            } else {
                SizingAlgorithm::Block
            },
            axis,
        )
    );
}

#[test]
fn fri04_c04_leaf_block_positioned_block_and_absolute_cover_all_unsupported_states() {
    let sizing = || {
        SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite sizing calculation"))
    };
    let calc = || CalcSizeCalculation::value(LengthPercentageOf::ZERO);

    for position in [Position::Relative, Position::Absolute] {
        for (value, behavior) in [
            (PreferredSize::STRETCH, SizingBehavior::Stretch),
            (PreferredSize::FIT_CONTENT, SizingBehavior::FitContent),
            (PreferredSize::CONTAIN, SizingBehavior::Contain),
            (
                PreferredSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Preferred(value),
                SizingProperty::Preferred,
                behavior,
                PhysicalAxis::Horizontal,
                position,
            );
        }
        if position == Position::Absolute {
            for (value, behavior) in [
                (PreferredSize::MIN_CONTENT, SizingBehavior::MinContent),
                (PreferredSize::MAX_CONTENT, SizingBehavior::MaxContent),
            ] {
                fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                    Fri04C04SizingValue::Preferred(value),
                    SizingProperty::Preferred,
                    behavior,
                    PhysicalAxis::Vertical,
                    position,
                );
            }
        }
        for (value, behavior) in [
            (MinSize::MIN_CONTENT, SizingBehavior::MinContent),
            (MinSize::MAX_CONTENT, SizingBehavior::MaxContent),
            (MinSize::STRETCH, SizingBehavior::Stretch),
            (MinSize::FIT_CONTENT, SizingBehavior::FitContent),
            (MinSize::CONTAIN, SizingBehavior::Contain),
            (
                MinSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Minimum(value),
                SizingProperty::Minimum,
                behavior,
                PhysicalAxis::Vertical,
                position,
            );
        }
        for (value, behavior) in [
            (MaxSize::MIN_CONTENT, SizingBehavior::MinContent),
            (MaxSize::MAX_CONTENT, SizingBehavior::MaxContent),
            (MaxSize::STRETCH, SizingBehavior::Stretch),
            (MaxSize::FIT_CONTENT, SizingBehavior::FitContent),
            (MaxSize::CONTAIN, SizingBehavior::Contain),
            (
                MaxSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Maximum(value),
                SizingProperty::Maximum,
                behavior,
                PhysicalAxis::Horizontal,
                position,
            );
        }

        for (basis, expected) in [
            (PreferredSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
            (
                PreferredSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                PreferredSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (
                PreferredSizeCalcBasis::Stretch,
                CalcSizeBehaviorBasis::Stretch,
            ),
            (
                PreferredSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (
                PreferredSizeCalcBasis::Contain,
                CalcSizeBehaviorBasis::Contain,
            ),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Preferred(
                    PreferredSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                SizingProperty::Preferred,
                SizingBehavior::CalcSize(expected),
                PhysicalAxis::Vertical,
                position,
            );
        }
        for (basis, expected) in [
            (MinSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
            (
                MinSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                MinSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (MinSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
            (
                MinSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (MinSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Minimum(
                    MinSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                SizingProperty::Minimum,
                SizingBehavior::CalcSize(expected),
                PhysicalAxis::Horizontal,
                position,
            );
        }
        for (basis, expected) in [
            (MaxSizeCalcBasis::None, CalcSizeBehaviorBasis::None),
            (
                MaxSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                MaxSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (MaxSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
            (
                MaxSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (MaxSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
        ] {
            fri04_c04_leaf_block_positioned_assert_block_path_unsupported(
                Fri04C04SizingValue::Maximum(
                    MaxSize::calc_size(basis, calc()).expect("valid calc-size"),
                ),
                SizingProperty::Maximum,
                SizingBehavior::CalcSize(expected),
                PhysicalAxis::Vertical,
                position,
            );
        }
    }
}

#[test]
fn fri04_c04_leaf_block_positioned_block_and_absolute_calc_size_geometry() {
    let preferred = || {
        Size::new(
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::Any,
                CalcSizeCalculation::from_coefficients(20.0, 0.5, 0.0)
                    .expect("finite Any calculation"),
            )
            .expect("valid Any calc-size"),
            PreferredSize::calc_size(
                PreferredSizeCalcBasis::FullPercentage,
                CalcSizeCalculation::from_coefficients(10.0, 0.0, 0.5)
                    .expect("finite FullPercentage calculation"),
            )
            .expect("valid FullPercentage calc-size"),
        )
    };
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::px(160.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: preferred(),
                ..NodeInput::default()
            },
        )
        .with_style(
            2,
            NodeInput {
                display: Display::Block,
                position: Position::Absolute,
                size: preferred(),
                ..NodeInput::default()
            },
        );

    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::new(
            Available::definite(200.0),
            Available::definite(160.0),
        ))
        .expect("valid viewport"),
    )
    .expect("supported block and positioned calc-size values resolve");

    assert_eq!(public_final_output(&batch, 1).size, Size::new(120.0, 90.0));
    assert_eq!(public_final_output(&batch, 2).size, Size::new(120.0, 90.0));
}

#[test]
fn fri04_c04_leaf_block_positioned_absolute_grid_and_block_inner_displays_are_positioned() {
    for display in [Display::Block, Display::Grid] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display,
                    position: Position::Absolute,
                    size: Size::new(PreferredSize::STRETCH, PreferredSize::AUTO),
                    ..NodeInput::default()
                },
            );
        let error = compute_layout(
            &tree,
            0,
            LayoutRootRequest::viewport(Size::splat(Available::definite(100.0)))
                .expect("valid viewport"),
        )
        .expect_err("absolute sizing must reject stretch before inner display dispatch");
        let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
            unsupported,
        )) = error.kind()
        else {
            panic!("expected positioned sizing capability");
        };
        assert_eq!(unsupported.algorithm(), SizingAlgorithm::Positioned);
        assert_eq!(unsupported.axis(), PhysicalAxis::Horizontal);
        assert_eq!(error.site(), LayoutErrorSite::Node(1));
    }
}

fn perform_scroll_block(tree: &mut ScrollBlockTree) -> ComputeOutput {
    crate::compute_block(
        tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(40.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::definite(40.0)),
        ),
    )
    .unwrap()
}

fn child_scroll_geometry(
    overflow: ComputedOverflow,
    size: Size,
    scrollable_overflow: ScrollRect,
) -> ScrollGeometry {
    child_scroll_geometry_with_edges(
        overflow,
        size,
        scrollable_overflow,
        Edges::ZERO,
        Edges::ZERO,
    )
}

fn child_scroll_geometry_with_edges(
    overflow: ComputedOverflow,
    size: Size,
    scrollable_overflow: ScrollRect,
    padding: Edges<f32>,
    border: Edges<f32>,
) -> ScrollGeometry {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut contributions =
        crate::scroll::ScrollContributionAccumulatorOf::new(scrollable_overflow);
    contributions.include_direct_line(scrollable_overflow);
    crate::scroll::canonical_scroll_geometry_from_source(
        crate::scroll::CanonicalScrollGeometrySourceOf {
            flow_axes,
            computed_overflow: overflow,
            item_is_replaced: false,
            border_box_size: size,
            border,
            padding,
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: ScrollbarWidth::ZERO,
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            clip_margin: crate::scroll::ClipMarginSourceOf::default(),
            scroll_padding: crate::scroll::OptimalRegionInsetsOf::default(),
            contributions,
            origin_axes: crate::scroll::ScrollOriginAxes::new(
                crate::scroll::ScrollOriginProgression::FlowEndward,
                crate::scroll::ScrollOriginProgression::FlowEndward,
            ),
            scroll_snap_type: ScrollSnapType::default(),
            target_border_box: ScrollRect::try_new(Point::ZERO, size).unwrap(),
            target_scroll_margin: ScrollMargin::default(),
            target_flow_axes: flow_axes,
            target_snap_align: ScrollSnapAlign::default(),
            target_snap_stop: ScrollSnapStop::default(),
        },
    )
    .expect("canonical block-test source facts produce geometry")
}

#[test]
fn block_layout_emits_scroll_geometry_for_scroll_overflow() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            overflow: computed_overflow(Overflow::Scroll, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_uses_visible_child_overflow_content_size() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(130.0, 70.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(130.0, 70.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(30.0, 30.0));
}

#[test]
fn block_scroll_geometry_clips_hidden_child_overflow_from_parent_range() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(160.0, 90.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(100.0, 40.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_preserves_negative_child_overflow_origin() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            inset: Edges {
                left: LengthAuto::px(-20.0),
                top: LengthAuto::px(-5.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            position: Position::Relative,
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-20.0, -5.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(120.0, 45.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_distinguishes_visible_hidden_clip_and_scroll() {
    fn run(overflow: ComputedOverflow) -> ScrollGeometry {
        let mut tree = ScrollBlockTree::default();
        tree.insert_children(1, vec![]);
        tree.insert_style(
            1,
            NodeInput {
                display: Display::Block,
                overflow,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        );

        perform_scroll_block(&mut tree).scroll_geometry.unwrap()
    }

    let visible = run(computed_overflow(Overflow::Visible, Overflow::Visible));
    assert_eq!(visible.overflow_clip(), None);
    assert_positive_physical_range(visible.physical_range(), Size::ZERO);

    let hidden = run(computed_overflow(Overflow::Hidden, Overflow::Hidden));
    assert_eq!(hidden.overflow_clip(), Some(hidden.scrollport()));
    assert_eq!(
        hidden
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(3.0, 4.0).unwrap()),
        PhysicalScrollOffset::try_new(0.0, 0.0).unwrap()
    );

    let clip = run(computed_overflow(Overflow::Clip, Overflow::Clip));
    assert_eq!(clip.overflow_clip(), Some(clip.scrollport()));
    assert_positive_physical_range(clip.physical_range(), Size::ZERO);

    let scroll = run(computed_overflow(Overflow::Scroll, Overflow::Scroll));
    assert_eq!(scroll.overflow_clip(), Some(scroll.scrollport()));
    assert_positive_physical_range(scroll.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_uses_node_local_padding_border_and_gutter_coordinates() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            padding: Edges::all(Length::px(2.0)),
            border: Edges::all(Length::px(3.0)),
            ..NodeInput::default()
        },
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().origin(), Point::new(13.0, 3.0));
    assert_eq!(geometry.scrollport().size(), Size::new(84.0, 34.0));
    assert_eq!(
        geometry.gutters().vertical(),
        Some(ScrollRect::try_new(Point::new(3.0, 3.0), Size::new(10.0, 34.0)).unwrap())
    );
}

#[test]
fn block_scroll_geometry_includes_absolute_child_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            inset: Edges {
                left: LengthAuto::px(90.0),
                top: LengthAuto::px(35.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(45.0, 25.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(135.0, 60.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(35.0, 20.0));
}

#[test]
fn block_scroll_geometry_includes_final_content_box_after_size_resolution() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            min_size: Size::new(MinSize::px(140.0), MinSize::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(60.0), Some(40.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(60.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(140.0, 80.0)).unwrap()
    );
}

#[test]
fn block_scroll_geometry_includes_inline_child_origin_bearing_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Hidden),
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(fri06_atomic_participation()),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut inline_output = ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0));
    inline_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(20.0, 10.0),
        ScrollRect::try_new(Point::new(-12.0, -3.0), Size::new(70.0, 26.0)).unwrap(),
    ));
    tree.insert_measure(2, inline_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-12.0, -3.0)
    );
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(70.0, 26.0));
    assert_positive_physical_range(geometry.physical_range(), Size::new(18.0, 13.0));
}

#[test]
fn block_scroll_geometry_clips_hidden_inline_child_overflow_from_parent_range() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(fri06_atomic_participation()),
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ..NodeInput::default()
        },
    );
    let mut inline_output =
        ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(150.0, 80.0));
    inline_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        Size::new(30.0, 10.0),
        ScrollRect::try_new(Point::new(-20.0, -7.0), Size::new(180.0, 92.0)).unwrap(),
    ));
    tree.insert_measure(2, inline_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(100.0, 40.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_includes_segmented_inline_overflow_rects() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2, 3, 4, 5]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(5, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(fri06_atomic_participation()),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        5,
        NodeInput {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(fri06_atomic_participation()),
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut first_inline = ComputeOutput::from_sizes(Size::new(10.0, 10.0), Size::new(10.0, 10.0));
    first_inline.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(10.0, 10.0),
        ScrollRect::try_new(Point::new(-20.0, 0.0), Size::new(30.0, 10.0)).unwrap(),
    ));
    let mut second_inline = ComputeOutput::from_sizes(Size::new(10.0, 10.0), Size::new(10.0, 10.0));
    second_inline.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(10.0, 10.0),
        ScrollRect::try_new(Point::new(-7.0, 0.0), Size::new(25.0, 12.0)).unwrap(),
    ));
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 50.0)));
    tree.insert_measure(3, first_inline);
    tree.insert_measure(5, second_inline);
    tree.insert_style(4, NodeInput::default());

    let mut segmented = tree.line_break(
        4,
        LineBreakInput::new()
            .with_clear(Clear::Left)
            .with_metrics(metrics),
    );

    let output = crate::compute_block(
        &mut segmented,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(80.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-7.0, 0.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(107.0, 80.0)
    );
}

#[test]
fn block_scroll_geometry_includes_float_child_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut float_output = ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(30.0, 10.0));
    float_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(30.0, 10.0),
        ScrollRect::try_new(Point::ZERO, Size::new(140.0, 55.0)).unwrap(),
    ));
    tree.insert_measure(2, float_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(140.0, 55.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(40.0, 15.0));
}

#[test]
fn block_float_child_node_output_retains_canonical_scroll_geometry() {
    let padding = Edges::all(Length::px(2.0));
    let border = Edges::all(Length::px(1.0));
    let resolved_padding = Edges::all(2.0);
    let resolved_border = Edges::all(1.0);
    let child_compute_overflow =
        ScrollRect::try_new(Point::new(-8.0, -3.0), Size::new(50.0, 20.0)).unwrap();
    let mut float_output = ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(70.0, 32.0));
    float_output.scroll_geometry = Some(child_scroll_geometry_with_edges(
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        Size::new(30.0, 10.0),
        child_compute_overflow,
        resolved_padding,
        resolved_border,
    ));

    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            padding,
            border,
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, float_output);

    perform_scroll_block(&mut tree);

    let child_layout = tree.layout(2).expect("child layout is staged");
    assert_eq!(child_layout.size, Size::new(30.0, 10.0));
    assert_eq!(child_layout.content_size, Size::new(70.0, 32.0));
    assert_eq!(child_layout.padding, resolved_padding);
    assert_eq!(child_layout.border, resolved_border);

    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow(), child_compute_overflow);
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        child_compute_overflow.origin()
    );
    assert_eq!(geometry, float_output.scroll_geometry.unwrap());
}

#[test]
fn block_scroll_geometry_includes_absolute_margin_box_with_area_offset() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(80.0)),
            padding: Edges::all(Length::px(7.0)),
            border: Edges::all(Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            inset: Edges {
                left: LengthAuto::px(90.0),
                top: LengthAuto::px(60.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            margin: Edges {
                left: LengthAuto::px(4.0),
                top: LengthAuto::px(3.0),
                right: LengthAuto::px(6.0),
                bottom: LengthAuto::px(7.0),
            },
            ..NodeInput::default()
        },
    );
    let mut absolute_output =
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(50.0, 20.0));
    absolute_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Visible, Overflow::Visible),
        Size::new(20.0, 10.0),
        ScrollRect::try_new(Point::new(-2.0, -1.0), Size::new(60.0, 25.0)).unwrap(),
    ));
    tree.insert_measure(2, absolute_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(5.0, 5.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(152.0, 87.0)
    );
    assert_eq!(output.content_size, Size::new(152.0, 87.0));
}

#[test]
fn block_child_node_output_recomputes_child_scroll_geometry() {
    let mut child_output = ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(80.0, 45.0));
    child_output.scroll_geometry = None;

    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, child_output);

    perform_scroll_block(&mut tree);

    let geometry = tree
        .layout(2)
        .expect("child layout is staged")
        .scroll_geometry
        .unwrap();
    assert_eq!(geometry.scrollport().size(), Size::new(50.0, 20.0));
    assert_positive_physical_range(geometry.physical_range(), Size::new(30.0, 25.0));
}

#[test]
fn block_child_node_output_keeps_hidden_child_own_scroll_range() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(160.0, 90.0)),
    );

    perform_scroll_block(&mut tree);

    let geometry = tree
        .layout(2)
        .expect("child layout is staged")
        .scroll_geometry
        .unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(160.0, 90.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(110.0, 70.0));
}

#[test]
fn block_absolute_child_scroll_geometry_uses_final_node_output_size() {
    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(120.0, 30.0)),
    );

    perform_scroll_block(&mut tree);

    let child_layout = tree.layout(2).expect("child layout is staged");
    assert_eq!(child_layout.size.width, 100.0);
    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size().width, 100.0);
    assert_positive_physical_range(geometry.physical_range(), Size::new(20.0, 20.0));
}

#[test]
fn block_child_node_output_preserves_child_scrollable_overflow_origin() {
    let child_overflow =
        ScrollRect::try_new(Point::new(-15.0, -4.0), Size::new(95.0, 49.0)).unwrap();
    let mut child_output = ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(80.0, 45.0));
    child_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        Size::new(50.0, 20.0),
        child_overflow,
    ));

    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, child_output);

    perform_scroll_block(&mut tree);

    let geometry = tree
        .layout(2)
        .expect("child layout is staged")
        .scroll_geometry
        .unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-15.0, -4.0)
    );
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(95.0, 49.0));
}

#[test]
fn block_inline_child_node_output_uses_final_inline_item_geometry() {
    let child_overflow =
        ScrollRect::try_new(Point::new(-9.0, -3.0), Size::new(74.0, 34.0)).unwrap();
    let mut child_output = ComputeOutput::from_sizes(Size::new(40.0, 12.0), Size::new(65.0, 31.0));
    child_output.scroll_geometry = Some(child_scroll_geometry(
        computed_overflow(Overflow::Hidden, Overflow::Hidden),
        Size::new(40.0, 12.0),
        child_overflow,
    ));

    let mut tree = ScrollBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(fri06_atomic_participation()),
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, child_output);

    perform_scroll_block(&mut tree);

    let child_layout = tree.layout(2).expect("child layout is staged");
    assert_eq!(child_layout.size, Size::new(40.0, 12.0));
    assert_eq!(child_layout.content_size, Size::new(65.0, 31.0));
    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size(), child_layout.size);
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
}

#[test]
fn block_fixed_parent_height_keeps_orthogonal_child_inline_known() {
    let mut tree = CalcBlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(162.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Grid,
            writing_mode: WritingMode::VerticalRl,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .expect("fixed-height block layout succeeds");

    assert!(tree.inputs(2).iter().any(|input| {
        input.known().height == Some(162.0)
            && input.parent().height == Some(162.0)
            && input.available().height == Available::definite(162.0)
    }));
}

#[derive(Default)]
struct CalcLeafTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    layouts: HashMap<u32, NodeOutput>,
    invalid_leaf_measurement: bool,
}

impl Traverse for CalcLeafTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map_or([].as_slice(), Vec::as_slice)
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl Compute for CalcLeafTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        if self.child_count(node) > 0 {
            return compute_block(self, node, input);
        }

        let style = self.styles[&node].clone();
        let invalid_leaf_measurement = self.invalid_leaf_measurement;
        compute_leaf(input, &style, |measure_input| {
            let known = measure_input.known_content_size();
            let available = measure_input
                .available_content_size()
                .map(MeasurementAvailable::into_available);
            Ok::<_, core::convert::Infallible>(Size::new(
                if invalid_leaf_measurement {
                    f32::NAN
                } else {
                    known
                        .width
                        .or_else(|| available.width.into_option())
                        .unwrap_or(0.0)
                },
                known.height.unwrap_or(10.0),
            ))
        })
        .map_err(|error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                error.operation(),
                error.kind().clone(),
            )
        })
    }
}

#[test]
fn calc_leaf_tree_propagates_leaf_measurement_error_instead_of_panicking() {
    let mut tree = CalcLeafTree {
        invalid_leaf_measurement: true,
        ..CalcLeafTree::default()
    };
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(0, NodeInput::default());
    tree.styles.insert(1, NodeInput::default());

    let error = compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(output))
            if output.axis() == PhysicalAxis::Horizontal
    ));
}

#[test]
fn block_inline_affine_leaf_uses_public_leaf_path() {
    let mut tree = CalcLeafTree::default();
    let width = lp(10.0, 0.5);
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(fri06_atomic_participation()),
            size: Size::new(PreferredSize::value(width), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    let output = compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&1].size.width, 60.0);
    assert_eq!(output.content_size.width, 100.0);
}

#[test]
fn unresolved_symbolic_vertical_margin_is_not_treated_as_auto_margin() {
    let mut tree = CalcLeafTree::default();
    let margin = lp(0.0, 0.25);
    tree.styles.insert(
        1,
        NodeInput {
            margin: Edges {
                top: LengthAuto::value(margin),
                ..Edges::<Scalar>::ZERO.map(|_| LengthAuto::px(0.0))
            },
            ..NodeInput::default()
        },
    );

    let resolved =
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr)
            .zip_physical_edges_with_inline_extent(
                tree.styles[&1].margin,
                Size::new(None, None),
                |length, basis| length.resolve_auto_with_status(basis),
            );
    let resolved = resolve_logical_in_flow_margin(
        crate::geometry::LogicalEdgesOf::new(
            resolved.left,
            resolved.right,
            resolved.top,
            resolved.bottom,
        ),
        crate::geometry::LogicalSizeOf::new(10.0, 10.0),
        None,
    );

    assert_eq!(resolved.block_start, 0.0);
}

#[test]
fn invalid_numeric_margin_keeps_explicit_failure_without_panicking() {
    let margin = LengthAuto::value(
        LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX).expect("finite coefficients"),
    )
    .resolve_auto_with_status(Some(10.0));

    let resolved = resolve_logical_in_flow_margin(
        crate::geometry::LogicalEdgesOf::new(
            ResolvedLengthAuto::Resolved(0.0),
            ResolvedLengthAuto::Resolved(0.0),
            margin,
            ResolvedLengthAuto::Resolved(0.0),
        ),
        crate::geometry::LogicalSizeOf::new(10.0, 10.0),
        Some(10.0),
    );

    assert_eq!(resolved.block_start, 0.0);
}

#[derive(Default)]
struct Fri05C03BlockPassTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    child_output: Option<ComputeOutput>,
    child_inputs: Vec<ComputeInput>,
    layouts: Vec<(u32, NodeOutput)>,
}

impl Traverse for Fri05C03BlockPassTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl Compute for Fri05C03BlockPassTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInput {
        LayoutInput::box_input(self.styles[&node].clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.push((node, layout));
    }

    fn compute_child(
        &mut self,
        _node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, ComputeOutput, Self::Scalar> {
        self.child_inputs.push(input);
        Ok(self
            .child_output
            .expect("FRI-05 block pass child output is configured"))
    }
}

fn fri05_c03_block_input(size: Size<f32>, flow_axes: FlowAxes) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        size.map(Some),
        size.map(Some),
        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
        size.map(Available::definite),
    )
}

fn fri05_c03_block_overflow_at_flow_axes(
    flow_axes: FlowAxes,
    inline: Overflow,
    block: Overflow,
) -> ComputedOverflow {
    match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => computed_overflow(inline, block),
        PhysicalAxis::Vertical => computed_overflow(block, inline),
    }
}

fn fri05_c03_block_gutter_at(
    gutters: ScrollbarGutterRects,
    side: PhysicalSide,
) -> Option<ScrollRect> {
    match side {
        PhysicalSide::Top => gutters.top(),
        PhysicalSide::Right => gutters.right(),
        PhysicalSide::Bottom => gutters.bottom(),
        PhysicalSide::Left => gutters.left(),
    }
}

fn fri05_c03_block_all_flow_axes() -> [FlowAxes; 10] {
    [
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysRl, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
    ]
}

fn fri05_c03_empty_block_geometry(style: NodeInput) -> ScrollGeometry {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let size = Size::new(100.0, 80.0);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![]);
    tree.styles.insert(0, style);
    crate::compute_block(&mut tree, 0, fri05_c03_block_input(size, flow_axes))
        .expect("FRI-05 empty block layout succeeds")
        .scroll_geometry
        .expect("performed block layout emits geometry")
}

#[test]
fn fri05_c03_block_reservation_places_forced_and_stable_gutters_in_all_flows() {
    for flow_axes in fri05_c03_block_all_flow_axes() {
        let style = |overflow, gutter, width| NodeInput {
            display: Display::Block,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow,
            scrollbar_gutter: gutter,
            scrollbar_width: ScrollbarWidth::try_new(width).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        };
        let forced = fri05_c03_empty_block_geometry(style(
            fri05_c03_block_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Scroll),
            ScrollbarGutter::Auto,
            7.0,
        ));
        let stable = fri05_c03_empty_block_geometry(style(
            fri05_c03_block_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::Stable,
            7.0,
        ));
        let both = fri05_c03_empty_block_geometry(style(
            fri05_c03_block_overflow_at_flow_axes(flow_axes, Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::StableBothEdges,
            7.0,
        ));

        for (geometry, expected_sides) in [
            (forced, vec![flow_axes.inline_end()]),
            (stable, vec![flow_axes.inline_end()]),
            (both, vec![flow_axes.inline_start(), flow_axes.inline_end()]),
        ] {
            for side in [
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
            ] {
                assert_eq!(
                    fri05_c03_block_gutter_at(geometry.gutters(), side).is_some(),
                    expected_sides.contains(&side),
                    "unexpected {side:?} gutter for {flow_axes:?}: {geometry:#?}"
                );
            }
        }

        let expected_one_edge = match flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(7.0, 0.0),
            PhysicalAxis::Vertical => Size::new(0.0, 7.0),
        };
        assert_eq!(forced.scrollbar_size(), expected_one_edge, "{flow_axes:?}");
        assert_eq!(stable.scrollbar_size(), expected_one_edge, "{flow_axes:?}");
        assert_eq!(both.scrollbar_size(), expected_one_edge + expected_one_edge);

        let zero = fri05_c03_empty_block_geometry(style(
            fri05_c03_block_overflow_at_flow_axes(flow_axes, Overflow::Scroll, Overflow::Scroll),
            ScrollbarGutter::StableBothEdges,
            0.0,
        ));
        assert_eq!(zero.scrollbar_size(), Size::ZERO, "{flow_axes:?}");
        assert_eq!(zero.gutters().top(), None);
        assert_eq!(zero.gutters().right(), None);
        assert_eq!(zero.gutters().bottom(), None);
        assert_eq!(zero.gutters().left(), None);
    }
}

fn fri05_c03_block_auto_case(
    gutter: ScrollbarGutter,
    child_size: Size<f32>,
    expected_states: &[(bool, bool)],
    expected_reservation: Size<f32>,
) {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let size = Size::splat(100.0);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_gutter: gutter,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_sizes(child_size, child_size));

    let output = crate::compute_block(&mut tree, 0, fri05_c03_block_input(size, flow_axes))
        .expect("FRI-05 auto block layout succeeds");
    assert!(
        tree.child_inputs.iter().all(|input| {
            input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
        }),
        "each block child starts node-local auto settlement at INITIAL: {:#?}",
        tree.child_inputs
    );
    let states = tree
        .child_inputs
        .iter()
        .map(|input| {
            let state = input.containing_auto_scrollbar_pass();
            (
                state.at(PhysicalAxis::Horizontal),
                state.at(PhysicalAxis::Vertical),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(states, expected_states, "child size {child_size:?}");
    assert_eq!(
        tree.layouts.iter().filter(|(node, _)| *node == 1).count(),
        expected_states.len(),
        "each geometry-changing evaluation stages the child once"
    );
    assert!(
        states.len() <= 3,
        "auto geometry must settle within three passes"
    );
    let geometry = output
        .scroll_geometry
        .expect("stable auto block output includes geometry");
    assert_eq!(geometry.scrollbar_size(), expected_reservation);
}

#[test]
fn fri05_c03_block_auto_runs_only_monotone_geometry_changing_evaluations() {
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(80.0, 80.0),
        &[(false, false)],
        Size::ZERO,
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(120.0, 80.0),
        &[(false, false), (true, false)],
        Size::new(0.0, 15.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(80.0, 120.0),
        &[(false, false), (false, true)],
        Size::new(15.0, 0.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(120.0, 100.0),
        &[(false, false), (true, false), (true, true)],
        Size::new(15.0, 15.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Auto,
        Size::new(100.0, 120.0),
        &[(false, false), (false, true), (true, true)],
        Size::new(15.0, 15.0),
    );
}

#[test]
fn fri05_c03_block_auto_stable_reservations_skip_redundant_full_evaluations() {
    fri05_c03_block_auto_case(
        ScrollbarGutter::Stable,
        Size::new(80.0, 120.0),
        &[(false, false)],
        Size::new(15.0, 0.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::StableBothEdges,
        Size::new(60.0, 120.0),
        &[(false, false)],
        Size::new(30.0, 0.0),
    );
    fri05_c03_block_auto_case(
        ScrollbarGutter::Stable,
        Size::new(90.0, 120.0),
        &[(false, false), (true, true)],
        Size::new(15.0, 15.0),
    );
}

#[test]
fn fri05_c03_block_tiny_saturates_opposing_reservations_before_child_layout() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let size = Size::new(2.0, 20.0);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(2.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_outer_size(Size::ZERO));

    let output = crate::compute_block(&mut tree, 0, fri05_c03_block_input(size, flow_axes))
        .expect("tiny block geometry remains supported");
    assert_eq!(tree.child_inputs.len(), 1);
    assert_eq!(tree.child_inputs[0].known().width, Some(0.0));
    let geometry = output
        .scroll_geometry
        .expect("tiny performed block emits geometry");
    assert_eq!(geometry.border_box().size(), size);
    assert_eq!(geometry.content_box().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollport().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollbar_size(), Size::new(2.0, 0.0));
    let left = geometry.gutters().left().expect("left gutter is retained");
    let right = geometry
        .gutters()
        .right()
        .expect("right gutter is retained");
    assert_eq!(left.size(), Size::new(1.0, 20.0));
    assert_eq!(right.size(), Size::new(1.0, 20.0));
    assert_eq!(left.origin(), Point::ZERO);
    assert_eq!(right.origin(), Point::new(1.0, 0.0));
}

#[test]
fn fri05_c03_block_tiny_max_size_below_raw_edges_keeps_layout_geometry_coherent() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            max_size: Size::new(MaxSize::NONE, MaxSize::px(12.0)),
            border: Edges {
                top: Length::px(15.0),
                bottom: Length::px(15.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_outer_size(Size::ZERO));

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .expect("raw edges larger than max size remain supported");

    assert_eq!(tree.child_inputs.len(), 1);
    assert_eq!(
        tree.child_inputs[0].settled_auto_scrollbars(),
        crate::scroll::SettledAutoScrollbarState::INITIAL
    );
    assert_eq!(
        tree.child_inputs[0].containing_auto_scrollbar_pass(),
        crate::scroll::SettledAutoScrollbarState::INITIAL
    );
    assert_eq!(
        tree.child_inputs[0].available().width,
        Available::definite(100.0)
    );
    let child = tree
        .layouts
        .iter()
        .find_map(|(node, layout)| (*node == 1).then_some(*layout))
        .expect("the coherent pass stages its child");
    assert_eq!(child.location, Point::new(0.0, 15.0));

    assert_eq!(output.size, Size::new(100.0, 30.0));
    let geometry = output
        .scroll_geometry
        .expect("performed block emits canonical geometry");
    assert_eq!(geometry.border_box().size(), Size::new(100.0, 30.0));
    assert_eq!(geometry.padding_box().origin(), Point::new(0.0, 15.0));
    assert_eq!(geometry.padding_box().size(), Size::new(100.0, 0.0));
    assert_eq!(geometry.content_box(), geometry.padding_box());
    assert_eq!(geometry.scrollport(), geometry.padding_box());
    assert_eq!(geometry.physical_range().x().minimum(), 0.0);
    assert_eq!(geometry.physical_range().x().maximum(), 0.0);
    assert_eq!(geometry.physical_range().y().minimum(), 0.0);
    assert_eq!(geometry.physical_range().y().maximum(), 0.0);
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
}

#[test]
fn fri05_c03_block_tiny_max_inline_size_below_raw_edges_keeps_child_space_zero() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(20.0)),
            max_size: Size::new(MaxSize::px(12.0), MaxSize::NONE),
            border: Edges {
                right: Length::px(10.0),
                left: Length::px(10.0),
                ..Edges::all(Length::ZERO)
            },
            padding: Edges {
                right: Length::px(5.0),
                left: Length::px(5.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_outer_size(Size::ZERO));

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(None, Some(20.0)),
            Size::new(None, Some(20.0)),
            ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
            Size::new(Available::MAX_CONTENT, Available::definite(20.0)),
        ),
    )
    .expect("raw inline edges larger than max size remain supported");

    assert_eq!(
        tree.child_inputs
            .iter()
            .map(|input| {
                let state = input.settled_auto_scrollbars();
                (
                    state.at(PhysicalAxis::Horizontal),
                    state.at(PhysicalAxis::Vertical),
                )
            })
            .collect::<Vec<_>>(),
        [(false, false), (false, false)]
    );
    assert_eq!(
        tree.child_inputs
            .iter()
            .map(|input| {
                let state = input.containing_auto_scrollbar_pass();
                (
                    state.at(PhysicalAxis::Horizontal),
                    state.at(PhysicalAxis::Vertical),
                )
            })
            .collect::<Vec<_>>(),
        [(false, false), (false, false)]
    );
    assert_eq!(
        tree.child_inputs[0].available().width,
        Available::MAX_CONTENT
    );
    assert_eq!(tree.child_inputs[1].known().width, Some(0.0));
    assert_eq!(
        tree.child_inputs[1].available().width,
        Available::definite(0.0)
    );
    let child = tree
        .layouts
        .iter()
        .find_map(|(node, layout)| (*node == 1).then_some(*layout))
        .expect("the final coherent pass stages its child");
    assert_eq!(child.location, Point::new(15.0, 0.0));

    assert_eq!(output.size, Size::new(30.0, 20.0));
    let geometry = output
        .scroll_geometry
        .expect("performed block emits canonical geometry");
    assert_eq!(geometry.border_box().size(), output.size);
    assert_eq!(geometry.padding_box().origin(), Point::new(10.0, 0.0));
    assert_eq!(geometry.padding_box().size(), Size::new(10.0, 20.0));
    assert_eq!(geometry.content_box().origin(), Point::new(15.0, 0.0));
    assert_eq!(geometry.content_box().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollport(), geometry.padding_box());
    assert_eq!(geometry.physical_range().x().maximum(), 0.0);
    assert_eq!(geometry.physical_range().y().maximum(), 0.0);
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
}

#[test]
fn fri05_c03_block_contribution_current_sources_retain_targets_and_union_content_size() {
    #[derive(Clone, Copy)]
    enum ChildKind {
        InFlow,
        Float,
        Inline,
        Absolute,
    }

    let scroll_margin = ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    for kind in [
        ChildKind::InFlow,
        ChildKind::Float,
        ChildKind::Inline,
        ChildKind::Absolute,
    ] {
        let (display, float, position, inset, expected_overflow) = match kind {
            ChildKind::InFlow => (
                Display::Block,
                Float::None,
                Position::Relative,
                Edges::all(LengthAuto::AUTO),
                Size::new(30.0, 15.0),
            ),
            ChildKind::Float => (
                Display::Block,
                Float::Left,
                Position::Relative,
                Edges::all(LengthAuto::AUTO),
                Size::new(30.0, 15.0),
            ),
            ChildKind::Inline => (
                Display::InlineBlock,
                Float::None,
                Position::Relative,
                Edges::all(LengthAuto::AUTO),
                Size::new(30.0, 15.0),
            ),
            ChildKind::Absolute => (
                Display::Block,
                Float::None,
                Position::Absolute,
                Edges {
                    top: LengthAuto::px(12.0),
                    left: LengthAuto::px(15.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                Size::new(45.0, 27.0),
            ),
        };
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display,
                    atomic_inline_participation: display
                        .is_inline_level()
                        .then_some(fri06_atomic_participation()),
                    float,
                    position,
                    inset,
                    size: Size::new(PreferredSize::px(30.0), PreferredSize::px(15.0)),
                    scroll_margin,
                    scroll_snap_align: snap_align,
                    scroll_snap_stop: ScrollSnapStop::Always,
                    ..NodeInput::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap(),
        )
        .expect("each current block-owned contribution source lays out");

        let root = public_final_output(&batch, 0);
        let root_geometry = root
            .scroll_geometry
            .expect("root block geometry is present");
        assert_eq!(root_geometry.scrollable_overflow().origin(), Point::ZERO);
        assert_eq!(
            root_geometry.scrollable_overflow().size(),
            expected_overflow
        );
        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));

        let child = public_final_output(&batch, 1);
        let target = child
            .scroll_geometry
            .expect("every performed block-owned child retains geometry")
            .target();
        assert_eq!(target.border_box().size(), child.size);
        assert_eq!(target.scroll_margin(), scroll_margin);
        assert_eq!(target.snap_align(), snap_align);
        assert_eq!(target.snap_stop(), ScrollSnapStop::Always);
    }
}

fn fri05_c03_block_contribution_fallback_child(
    display: Display,
    overflow: ComputedOverflow,
) -> (NodeOutput, NodeOutput) {
    let scroll_padding = ScrollPadding::new(
        ScrollPaddingValue::value(LengthPercentageOf::px(2.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(4.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(3.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(1.0).unwrap()),
    );
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Rtl,
                overflow,
                overflow_clip_margin: OverflowClipMargin::try_new(OverflowClipBox::BorderBox, 3.0)
                    .unwrap(),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(9.0).unwrap(),
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(60.0)),
                border: Edges::new(
                    Length::px(1.0),
                    Length::px(2.0),
                    Length::px(3.0),
                    Length::px(4.0),
                ),
                padding: Edges::new(
                    Length::px(5.0),
                    Length::px(6.0),
                    Length::px(7.0),
                    Length::px(8.0),
                ),
                margin: Edges::new(
                    LengthAuto::px(2.0),
                    LengthAuto::px(3.0),
                    LengthAuto::px(4.0),
                    LengthAuto::px(5.0),
                ),
                scroll_padding,
                scroll_margin: ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap(),
                scroll_snap_type: ScrollSnapType::Enabled {
                    axis: ScrollSnapAxis::Block,
                    strictness: ScrollSnapStrictness::Mandatory,
                },
                scroll_snap_align: ScrollSnapAlign::new(
                    ScrollSnapAlignValue::End,
                    ScrollSnapAlignValue::Center,
                ),
                scroll_snap_stop: ScrollSnapStop::Always,
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(200.0))).unwrap(),
    )
    .expect("a block stages fallback geometry for its flex/grid child");

    assert_eq!(
        batch
            .final_entries()
            .iter()
            .filter(|entry| entry.node() == 1)
            .count(),
        1,
        "the direct child is staged exactly once"
    );
    (
        public_final_output(&batch, 0),
        public_final_output(&batch, 1),
    )
}

fn fri05_c03_assert_block_contribution_fallback_common(root: NodeOutput, child: NodeOutput) {
    let geometry = child
        .scroll_geometry
        .expect("a performed block-owned child has canonical geometry");
    let target = geometry.target();
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    let scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);

    assert_eq!(geometry.flow_axes(), flow_axes);
    assert_eq!(geometry.border_box().origin(), Point::ZERO);
    assert_eq!(geometry.border_box().size(), child.size);
    assert_eq!(
        geometry.padding_box(),
        ScrollRect::try_new(
            Point::new(child.border.left, child.border.top),
            Size::new(
                child.size.width - child.border.horizontal_sum(),
                child.size.height - child.border.vertical_sum(),
            ),
        )
        .unwrap()
    );
    assert_eq!(
        geometry.resolved_scroll_padding(),
        Edges::new(2.0, 4.0, 3.0, 1.0)
    );
    assert_eq!(
        geometry.scroll_snap_type(),
        ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Block,
            strictness: ScrollSnapStrictness::Mandatory,
        }
    );
    assert_eq!(target.border_box(), geometry.border_box());
    assert_eq!(target.scroll_margin(), scroll_margin);
    assert_eq!(target.flow_axes(), flow_axes);
    assert_eq!(target.snap_align(), snap_align);
    assert_eq!(target.snap_stop(), ScrollSnapStop::Always);

    let root_geometry = root
        .scroll_geometry
        .expect("the performed block root has canonical geometry");
    let seed = root_geometry.padding_box();
    let seed_origin = seed.origin();
    let seed_size = seed.size();
    let contribution_origin = Point::new(
        child.location.x - child.margin.left.max(0.0),
        child.location.y - child.margin.top.max(0.0),
    );
    let contribution_end = Point::new(
        child.location.x + child.size.width + child.margin.right.max(0.0),
        child.location.y + child.size.height + child.margin.bottom.max(0.0),
    );
    let expected_origin = Point::new(
        seed_origin.x.min(contribution_origin.x),
        seed_origin.y.min(contribution_origin.y),
    );
    let expected_end = Point::new(
        (seed_origin.x + seed_size.width).max(contribution_end.x),
        (seed_origin.y + seed_size.height).max(contribution_end.y),
    );
    let expected_overflow = ScrollRect::try_new(
        expected_origin,
        Size::new(
            expected_end.x - expected_origin.x,
            expected_end.y - expected_origin.y,
        ),
    )
    .unwrap();
    assert_eq!(root_geometry.scrollable_overflow(), expected_overflow);
    assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
}

fn fri05_c06_assert_block_reserved_gutter_geometry(geometry: ScrollGeometry) {
    assert_eq!(geometry.border_box().size(), Size::new(100.0, 100.0));
    assert_eq!(geometry.padding_box().size(), Size::new(100.0, 100.0));
    assert_eq!(geometry.scrollport().origin(), Point::new(15.0, 0.0));
    assert_eq!(geometry.scrollport().size(), Size::new(70.0, 100.0));
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(100.0, 150.0)).unwrap(),
        "reserved gutters remain part of complete scrollable overflow"
    );

    let range = geometry.physical_range();
    assert_eq!(
        range.x().maximum() - range.x().minimum(),
        0.0,
        "reserved gutters do not create horizontal scroll range"
    );
    assert_eq!(
        range.y().maximum() - range.y().minimum(),
        50.0,
        "vertical child overflow remains reachable"
    );
}

#[test]
fn fri05_c06_block_reserved_gutter_stable_both_edges_excludes_horizontal_range() {
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
                scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(70.0), PreferredSize::px(150.0)),
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(200.0))).unwrap(),
    )
    .expect("stable both-edge block layout succeeds");

    fri05_c06_assert_block_reserved_gutter_geometry(
        public_final_output(&batch, 0)
            .scroll_geometry
            .expect("the block front door emits canonical geometry"),
    );
}

#[test]
fn fri05_c06_block_reserved_gutter_retained_child_fallback_excludes_horizontal_range() {
    let mut tree = Fri05C03BlockPassTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(200.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Flex,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.child_output = Some(ComputeOutput::from_sizes_and_baselines(
        Size::new(100.0, 100.0),
        Size::new(70.0, 150.0),
        Baselines::NONE,
    ));

    crate::compute_block(
        &mut tree,
        0,
        fri05_c03_block_input(
            Size::new(200.0, 200.0),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .expect("the block front door stages retained-child fallback geometry");
    let child = tree
        .layouts
        .iter()
        .rev()
        .find_map(|(node, output)| (*node == 1).then_some(*output))
        .expect("the retained child is staged");

    fri05_c06_assert_block_reserved_gutter_geometry(
        child
            .scroll_geometry
            .expect("the retained-child fallback emits canonical geometry"),
    );
}

#[test]
fn fri05_c03_block_contribution_flex_fallback_retains_target_and_box_sources() {
    let (root, child) = fri05_c03_block_contribution_fallback_child(
        Display::Flex,
        computed_overflow(Overflow::Hidden, Overflow::Scroll),
    );
    fri05_c03_assert_block_contribution_fallback_common(root, child);

    let geometry = child.scroll_geometry.unwrap();
    let padding_box = geometry.padding_box();
    let expected_scrollport = ScrollRect::try_new(
        Point::new(padding_box.origin().x + 9.0, padding_box.origin().y),
        Size::new(padding_box.size().width - 18.0, padding_box.size().height),
    )
    .unwrap();
    assert_eq!(geometry.scrollport(), expected_scrollport);
    assert_eq!(geometry.scrollbar_size(), Size::new(18.0, 0.0));
    assert_eq!(geometry.gutters().top(), None);
    assert_eq!(geometry.gutters().bottom(), None);
    assert_eq!(geometry.gutters().left().unwrap().size().width, 9.0);
    assert_eq!(geometry.gutters().right().unwrap().size().width, 9.0);
    let x_clip = geometry.overflow_clip().x().unwrap();
    let y_clip = geometry.overflow_clip().y().unwrap();
    assert_eq!(
        (x_clip.minimum(), x_clip.maximum()),
        (
            expected_scrollport.origin().x,
            expected_scrollport.origin().x + expected_scrollport.size().width,
        )
    );
    assert_eq!(
        (y_clip.minimum(), y_clip.maximum()),
        (
            expected_scrollport.origin().y,
            expected_scrollport.origin().y + expected_scrollport.size().height,
        )
    );
    assert_eq!(geometry.used_overflow_x(), Overflow::Hidden);
    assert_eq!(geometry.used_overflow_y(), Overflow::Scroll);
    assert_eq!(
        geometry.content_box(),
        ScrollRect::try_new(
            Point::new(
                expected_scrollport.origin().x + child.padding.left,
                expected_scrollport.origin().y + child.padding.top,
            ),
            Size::new(
                expected_scrollport.size().width - child.padding.horizontal_sum(),
                expected_scrollport.size().height - child.padding.vertical_sum(),
            ),
        )
        .unwrap()
    );
}

#[test]
fn fri05_c03_block_contribution_flex_and_grid_fallback_seed_padding_with_stable_gutters() {
    let cases = [Display::Flex, Display::Grid].map(|display| {
        let (root, child) = fri05_c03_block_contribution_fallback_child(
            display,
            computed_overflow(Overflow::Hidden, Overflow::Scroll),
        );
        fri05_c03_assert_block_contribution_fallback_common(root, child);
        (display, child)
    });

    assert_eq!(
        cases.map(|(_, child)| child.scroll_geometry.unwrap().scrollable_overflow()),
        cases.map(|(_, child)| child.scroll_geometry.unwrap().padding_box()),
        "flex and grid fallback must both retain their own padding and gutter area"
    );

    for (display, child) in cases {
        let geometry = child
            .scroll_geometry
            .expect("the fallback child has canonical geometry");
        let expected_range = (0.0, 0.0, 0.0, 0.0);
        assert_ne!(geometry.padding_box(), geometry.scrollport(), "{display:?}");
        assert_ne!(
            geometry.padding_box(),
            geometry.content_box(),
            "{display:?}"
        );
        assert_eq!(
            (
                geometry.physical_range().x().minimum(),
                geometry.physical_range().x().maximum(),
                geometry.physical_range().y().minimum(),
                geometry.physical_range().y().maximum(),
            ),
            expected_range,
            "{display:?}"
        );
        assert_eq!(
            child.content_box_size(),
            geometry.content_box().size(),
            "{display:?}"
        );
        assert_eq!(
            child.scrollbar_size(),
            geometry.scrollbar_size(),
            "{display:?}"
        );
        assert_eq!(
            child.scrollbar_size(),
            geometry.scrollbar_size(),
            "{display:?}"
        );
        assert_eq!(
            geometry.target().border_box(),
            geometry.border_box(),
            "{display:?}"
        );
    }
}

#[test]
fn fri05_c03_block_contribution_grid_fallback_retains_target_and_clip_sources() {
    let (root, child) = fri05_c03_block_contribution_fallback_child(
        Display::Grid,
        computed_overflow(Overflow::Clip, Overflow::Visible),
    );
    fri05_c03_assert_block_contribution_fallback_common(root, child);

    let geometry = child.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport(), geometry.padding_box());
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
    assert_eq!(geometry.gutters().top(), None);
    assert_eq!(geometry.gutters().right(), None);
    assert_eq!(geometry.gutters().bottom(), None);
    assert_eq!(geometry.gutters().left(), None);
    let x_clip = geometry
        .overflow_clip()
        .x()
        .expect("the child's x clip retains its border-box clip margin");
    assert_eq!(
        (x_clip.minimum(), x_clip.maximum()),
        (-3.0, child.size.width + 3.0)
    );
    assert_eq!(geometry.overflow_clip().y(), None);
    assert_eq!(geometry.used_overflow_x(), Overflow::Clip);
    assert_eq!(geometry.used_overflow_y(), Overflow::Visible);
    assert_eq!(
        geometry.content_box(),
        ScrollRect::try_new(
            Point::new(
                geometry.scrollport().origin().x + child.padding.left,
                geometry.scrollport().origin().y + child.padding.top,
            ),
            Size::new(
                geometry.scrollport().size().width - child.padding.horizontal_sum(),
                geometry.scrollport().size().height - child.padding.vertical_sum(),
            ),
        )
        .unwrap()
    );
}

#[test]
fn fri05_c03_block_contribution_terminal_padding_extends_final_in_flow_ends() {
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                padding: Edges {
                    right: Length::px(3.0),
                    bottom: Length::px(4.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap(),
    )
    .expect("terminal-padding block layout succeeds");

    let root = public_final_output(&batch, 0);
    let geometry = root
        .scroll_geometry
        .expect("root block geometry is present");
    assert_eq!(geometry.scrollable_overflow().origin(), Point::ZERO);
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(33.0, 24.0));
    assert_eq!(root.content_size, Size::new(33.0, 24.0));
}

#[test]
fn fri05_c03_block_negative_margin_families_use_only_positive_outsets() {
    for position in [Position::Relative, Position::Absolute] {
        let inset = match position {
            Position::Relative => Edges {
                top: LengthAuto::px(25.0),
                left: LengthAuto::px(105.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            Position::Absolute => Edges {
                top: LengthAuto::px(30.0),
                left: LengthAuto::px(90.0),
                ..Edges::all(LengthAuto::AUTO)
            },
        };
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    position,
                    inset,
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                    margin: Edges {
                        top: LengthAuto::px(-20.0),
                        right: LengthAuto::px(7.0),
                        bottom: LengthAuto::px(5.0),
                        left: LengthAuto::px(-30.0),
                    },
                    ..NodeInput::default()
                },
            );
        let result = std::panic::catch_unwind(|| {
            compute_layout(
                &tree,
                0,
                LayoutRootRequest::viewport(Size::splat(Available::definite(120.0))).unwrap(),
            )
        });
        let batch = result
            .expect("valid negative margins never panic")
            .expect("valid negative margins never produce an invalid synthetic rectangle");

        let root = public_final_output(&batch, 0);
        let child = public_final_output(&batch, 1);
        assert!(child.size.width > 0.0 && child.size.height > 0.0);
        let padding_box = root
            .scroll_geometry
            .expect("root block geometry is present")
            .padding_box();
        let padding_origin = padding_box.origin();
        let padding_size = padding_box.size();
        let expected_origin = Point::new(
            padding_origin
                .x
                .min(child.location.x - child.margin.left.max(0.0)),
            padding_origin
                .y
                .min(child.location.y - child.margin.top.max(0.0)),
        );
        let expected_end = Point::new(
            (padding_origin.x + padding_size.width)
                .max(child.location.x + child.size.width + child.margin.right.max(0.0)),
            (padding_origin.y + padding_size.height)
                .max(child.location.y + child.size.height + child.margin.bottom.max(0.0)),
        );
        let expected = ScrollRect::try_new(
            expected_origin,
            Size::new(
                expected_end.x - expected_origin.x,
                expected_end.y - expected_origin.y,
            ),
        )
        .unwrap();
        assert_eq!(
            root.scroll_geometry.unwrap().scrollable_overflow(),
            expected
        );
        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
    }
}

#[test]
fn fri05_c03_integration_padding_seed_direct_block_retains_gutter_area_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>()
    where
        crate::test_support::layout_tree::OracleTreeOf<S>:
            Compute + Traverse<Node = u32, Scalar = S>,
    {
        fn gutter_at<S: LayoutScalar>(
            gutters: ScrollbarGutterRectsOf<S>,
            side: PhysicalSide,
        ) -> Option<ScrollRectOf<S>> {
            match side {
                PhysicalSide::Top => gutters.top(),
                PhysicalSide::Right => gutters.right(),
                PhysicalSide::Bottom => gutters.bottom(),
                PhysicalSide::Left => gutters.left(),
            }
        }

        let scalar = scalar_value::<S>;
        let size = Size::new(scalar(100.0), scalar(80.0));
        for flow_axes in fri05_c03_block_all_flow_axes() {
            for (case, inline, block, scrollbar_gutter, expected_sides) in [
                (
                    "forced-block",
                    Overflow::Hidden,
                    Overflow::Scroll,
                    ScrollbarGutter::Auto,
                    vec![flow_axes.inline_end()],
                ),
                (
                    "stable-block",
                    Overflow::Hidden,
                    Overflow::Hidden,
                    ScrollbarGutter::Stable,
                    vec![flow_axes.inline_end()],
                ),
                (
                    "both-edge-block",
                    Overflow::Hidden,
                    Overflow::Hidden,
                    ScrollbarGutter::StableBothEdges,
                    vec![flow_axes.inline_start(), flow_axes.inline_end()],
                ),
                (
                    "forced-inline",
                    Overflow::Scroll,
                    Overflow::Hidden,
                    ScrollbarGutter::Auto,
                    vec![flow_axes.block_end()],
                ),
            ] {
                let style = NodeInputOf::<S> {
                    display: Display::Block,
                    writing_mode: flow_axes.writing_mode(),
                    direction: flow_axes.direction(),
                    overflow: fri05_c03_block_overflow_at_flow_axes(flow_axes, inline, block),
                    scrollbar_gutter,
                    scrollbar_width: ScrollbarWidthOf::try_new(scalar(7.0)).unwrap(),
                    size: Size::new(
                        PreferredSizeOf::px(size.width),
                        PreferredSizeOf::px(size.height),
                    ),
                    padding: Edges::all(LengthOf::px(scalar(3.0))),
                    border: Edges::all(LengthOf::px(scalar(2.0))),
                    ..NodeInputOf::default()
                };
                let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                    .children(0, [])
                    .style(0, style);
                let output = crate::compute_block(
                    &mut tree,
                    0,
                    ComputeInputOf::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        size.map(Some),
                        size.map(Some),
                        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                        size.map(AvailableOf::definite),
                    ),
                )
                .expect("guttered direct block lays out");
                let geometry = output
                    .scroll_geometry
                    .expect("performed direct block emits geometry");

                assert_ne!(
                    geometry.padding_box(),
                    geometry.scrollport(),
                    "{case}/{flow_axes:?}"
                );
                assert_eq!(
                    geometry.scrollable_overflow(),
                    geometry.padding_box(),
                    "the canonical own padding box must remain complete overflow for {case}/{flow_axes:?}"
                );
                for side in [
                    PhysicalSide::Top,
                    PhysicalSide::Right,
                    PhysicalSide::Bottom,
                    PhysicalSide::Left,
                ] {
                    assert_eq!(
                        gutter_at(geometry.gutters(), side).is_some(),
                        expected_sides.contains(&side),
                        "unexpected {side:?} gutter for {case}/{flow_axes:?}"
                    );
                }

                let range = geometry.physical_range();
                assert_eq!(
                    (range.x().minimum(), range.x().maximum()),
                    (S::ZERO, S::ZERO),
                    "x range must exclude static gutter reservation for {case}/{flow_axes:?}"
                );
                assert_eq!(
                    (range.y().minimum(), range.y().maximum()),
                    (S::ZERO, S::ZERO),
                    "y range must exclude static gutter reservation for {case}/{flow_axes:?}"
                );

                let node_output = NodeOutputOf::<S>::new().with_scroll_geometry(Some(geometry));
                assert_eq!(
                    node_output.content_box_size(),
                    geometry.content_box().size()
                );
                assert_eq!(node_output.scrollbar_size(), geometry.scrollbar_size());
                assert_eq!(geometry.target().border_box(), geometry.border_box());
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri05_c03_integration_absolute_top_gutter_offsets_reduced_area_margin_contribution_and_tiny_origins()
 {
    fn assert_lane<S: LayoutScalar>()
    where
        crate::test_support::layout_tree::OracleTreeOf<S>:
            Compute + Traverse<Node = u32, Scalar = S>,
    {
        let scalar = scalar_value::<S>;
        let top_gutter_flows = [
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
            FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
            FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl),
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        ];

        for flow_axes in top_gutter_flows {
            assert_eq!(flow_axes.inline_end(), PhysicalSide::Top);
            let container_size = Size::new(scalar(100.0), scalar(80.0));
            let child_size = Size::new(scalar(110.0), scalar(70.0));
            let child_margin = Edges::new(scalar(3.0), scalar(7.0), scalar(11.0), scalar(2.0));
            let container_style = NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: fri05_c03_block_overflow_at_flow_axes(
                    flow_axes,
                    Overflow::Hidden,
                    Overflow::Scroll,
                ),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(10.0)).unwrap(),
                size: Size::new(
                    PreferredSizeOf::px(container_size.width),
                    PreferredSizeOf::px(container_size.height),
                ),
                border: Edges::all(LengthOf::px(scalar(5.0))),
                ..NodeInputOf::default()
            };
            let child_style = NodeInputOf::<S> {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(
                    PreferredSizeOf::px(child_size.width),
                    PreferredSizeOf::px(child_size.height),
                ),
                inset: Edges {
                    top: LengthAutoOf::px(S::ZERO),
                    left: LengthAutoOf::px(S::ZERO),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                margin: Edges::new(
                    LengthAutoOf::px(child_margin.top),
                    LengthAutoOf::px(child_margin.right),
                    LengthAutoOf::px(child_margin.bottom),
                    LengthAutoOf::px(child_margin.left),
                ),
                ..NodeInputOf::default()
            };
            let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(0, container_style)
                .style(1, child_style)
                .measure(1, ComputeOutputOf::from_outer_size(child_size));
            let output = crate::compute_block(
                &mut tree,
                0,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    container_size.map(Some),
                    container_size.map(Some),
                    ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                    container_size.map(AvailableOf::definite),
                ),
            )
            .expect("top-gutter absolute layout succeeds");
            let geometry = output
                .scroll_geometry
                .expect("performed block emits geometry");
            let top_gutter = geometry
                .gutters()
                .top()
                .expect("flow reserves a top gutter");
            assert_eq!(top_gutter.size().height, scalar(10.0));
            assert_eq!(geometry.scrollport().origin().y, scalar(15.0));
            assert_eq!(
                geometry.scrollport().size(),
                Size::new(scalar(90.0), scalar(60.0))
            );

            let perform_inputs = tree
                .inputs(1)
                .iter()
                .filter(|input| input.run_mode() == RunMode::PerformLayout)
                .copied()
                .collect::<Vec<_>>();
            assert_eq!(perform_inputs.len(), 1, "absolute child is performed once");
            assert_eq!(
                perform_inputs[0].parent(),
                Size::new(Some(scalar(90.0)), Some(scalar(60.0)))
            );
            assert_eq!(
                perform_inputs[0].available(),
                Size::new(
                    AvailableOf::definite(scalar(90.0)),
                    AvailableOf::definite(scalar(60.0)),
                )
            );

            let child = tree.layout(1).expect("absolute child is staged");
            assert_eq!(child.location, Point::new(scalar(7.0), scalar(18.0)));
            assert_eq!(child.size, child_size);
            assert_eq!(child.margin, child_margin);
            let expected_overflow = ScrollRectOf::try_new(
                Point::new(scalar(5.0), scalar(5.0)),
                Size::new(scalar(119.0), scalar(94.0)),
            )
            .unwrap();
            assert_eq!(
                geometry.scrollable_overflow(),
                expected_overflow,
                "the final absolute margin area contributes exactly once"
            );
            assert_eq!(output.content_size, expected_overflow.size());

            let tiny_size = Size::splat(scalar(2.0));
            let tiny_child_size = Size::splat(scalar(1.0));
            let tiny_container = NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                scrollbar_width: ScrollbarWidthOf::try_new(scalar(15.0)).unwrap(),
                size: Size::new(
                    PreferredSizeOf::px(tiny_size.width),
                    PreferredSizeOf::px(tiny_size.height),
                ),
                border: Edges::new(
                    LengthOf::px(scalar(1.0)),
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    LengthOf::px(scalar(1.0)),
                ),
                ..NodeInputOf::default()
            };
            let tiny_child = NodeInputOf::<S> {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(
                    PreferredSizeOf::px(tiny_child_size.width),
                    PreferredSizeOf::px(tiny_child_size.height),
                ),
                inset: Edges {
                    top: LengthAutoOf::px(S::ZERO),
                    left: LengthAutoOf::px(S::ZERO),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                ..NodeInputOf::default()
            };
            let mut tiny_tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(0, tiny_container)
                .style(1, tiny_child)
                .measure(1, ComputeOutputOf::from_outer_size(tiny_child_size));
            let tiny_output = crate::compute_block(
                &mut tiny_tree,
                0,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    tiny_size.map(Some),
                    tiny_size.map(Some),
                    ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
                    tiny_size.map(AvailableOf::definite),
                ),
            )
            .expect("tiny top-gutter absolute layout stays ordered");
            let tiny_geometry = tiny_output
                .scroll_geometry
                .expect("tiny geometry is present");
            assert_eq!(tiny_geometry.scrollport().size(), Size::ZERO);
            let tiny_child = tiny_tree.layout(1).expect("tiny absolute child is staged");
            assert_eq!(tiny_child.location, tiny_geometry.scrollport().origin());
            let tiny_input = tiny_tree
                .inputs(1)
                .iter()
                .find(|input| input.run_mode() == RunMode::PerformLayout)
                .expect("tiny absolute child receives a perform input");
            assert_eq!(tiny_input.parent(), Size::splat(Some(S::ZERO)));
            assert_eq!(
                tiny_input.available(),
                Size::splat(AvailableOf::definite(S::ZERO))
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn assert_fri08_c07_t03_optional_math_block_results<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    let style = NodeInputOf::<S> {
        display: Display::Block,
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(PreferredSizeOf::px(scalar(4.0)), PreferredSizeOf::AUTO),
        padding: Edges::new(
            LengthOf::px(scalar(7.0)),
            LengthOf::px(scalar(5.0)),
            LengthOf::px(scalar(4.0)),
            LengthOf::px(scalar(3.0)),
        ),
        ..NodeInputOf::default()
    };
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [])
        .style(0, style);
    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(scalar(100.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        ),
    )
    .unwrap_or_else(|_| panic!("finite block sizing must succeed"));

    assert_eq!(output.size, Size::new(scalar(8.0), scalar(11.0)));

    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let overflowing = LengthPercentageOf::from_coefficients(largest, S::ONE)
        .unwrap_or_else(|_| panic!("finite coefficients must be accepted"));
    let mut failing_tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::value(overflowing), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        );
    let error = crate::compute_block(
        &mut failing_tree,
        0,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(largest), Some(scalar(100.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::new(AvailableOf::definite(largest), AvailableOf::MAX_CONTENT),
        ),
    )
    .expect_err("non-finite block sizing must preserve its error");

    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert_eq!(
        error.kind(),
        &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
            value: S::INFINITY,
        })
    );
}

#[test]
fn fri08_c07_t03_optional_math_block_results_preserve_both_scalar_lanes() {
    assert_fri08_c07_t03_optional_math_block_results::<f32>();
    assert_fri08_c07_t03_optional_math_block_results::<f64>();
}
