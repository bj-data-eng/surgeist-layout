use super::fixtures::{BlockTree, computed_overflow};
use super::*;

#[test]
fn block_layout_lays_out_absolute_children_without_flow_contribution_and_hides_display_none() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            border: Edges::all(Length::px(1.0)),
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
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        4,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(40.0, 10.0)),
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(80.0, 32.0)),
    );
    tree = tree.measure_when(
        4,
        crate::test_support::layout_tree::OracleMeasurement::new(ComputeOutput::HIDDEN)
            .run_mode(RunMode::PerformHiddenLayout),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(98.0, 41.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(1.0, 1.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(8.0, 10.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(4).expect("child layout is staged"),
        NodeOutput::with_source_index(crate::SourceIndex::new(2))
    );
    assert_eq!(
        tree.inputs(4),
        vec![ComputeInput::hidden(crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr,),
            crate::ParentFormattingContext::BlockFlow
        ))]
    );
}

#[test]
fn block_absolute_child_without_insets_uses_static_position_after_flow() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            border: Edges::all(Length::px(1.0)),
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
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(98.0, 10.0), Size::new(98.0, 10.0)),
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 5.0), Size::new(20.0, 5.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(1.0, 1.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(1.0, 11.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(20.0, 5.0)
    );
}

#[test]
fn block_absolute_child_auto_size_applies_aspect_ratio_to_max_size() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            aspect_ratio: AspectRatio::new(2.0),
            max_size: Size::new(MaxSize::px(50.0), MaxSize::NONE),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(50.0, 25.0)
    );
}

#[test]
fn block_absolute_child_auto_size_resolves_from_opposing_insets() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                right: LengthAuto::px(17.0),
                top: LengthAuto::px(13.0),
                bottom: LengthAuto::px(11.0),
            },
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(74.0), Some(24.0)));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(8.0, 14.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(74.0, 24.0)
    );
}

#[test]
fn block_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(10.0),
                right: LengthAuto::px(10.0),
                top: LengthAuto::AUTO,
                bottom: LengthAuto::AUTO,
            },
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(80.0), Some(40.0)));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(80.0, 40.0)
    );
}

#[test]
fn block_absolute_child_expands_horizontal_auto_margins() {
    let mut tree = BlockTree::default();
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
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(20.0), Some(10.0)));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        40.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        40.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(40.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn block_absolute_child_large_width_keeps_horizontal_auto_margins_zero() {
    let mut tree = BlockTree::default();
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
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(PreferredSize::px(70.0), PreferredSize::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(70.0), Some(10.0)));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        0.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        0.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(70.0, 10.0)
    );
}

#[test]
fn block_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                right: LengthAuto::px(17.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(20.0), Some(10.0)));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(62.0, 1.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}
