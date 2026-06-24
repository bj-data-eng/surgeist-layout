use super::*;
use surgeist_layout::{CalcExpression, CalcResolver, CalcTerm, LayoutCalcStore};

#[test]
fn block_lays_out_atomic_inline_children_on_one_line() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

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
fn vertical_rl_block_places_atomic_inline_run_at_inline_start_edge() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
                border: Edges::all(Length::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(55.0, 5.0)
    );
}

#[test]
fn inline_grid_uses_grid_tracks_and_participates_as_atomic_inline() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_grid_lanes_uses_lanes_tracks_and_participates_as_atomic_inline() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineGridLanes,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn block_wraps_atomic_inline_children_between_items() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(40.0), Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

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
fn block_min_content_atomic_inline_run_uses_max_item_advance() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                size: Size::new(Dimension::px(40.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(60.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

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
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                margin: Edges {
                    left: LengthAuto::AUTO,
                    right: LengthAuto::AUTO,
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    let child = tree.final_layout(1).unwrap();
    assert_eq!(child.location, Point::new(0.0, 0.0));
    assert_eq!(child.margin.left, 0.0);
    assert_eq!(child.margin.right, 0.0);
}

#[test]
fn inline_block_intrinsic_width_shrink_wraps_children() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(70.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(70.0, 20.0));
    assert_eq!(tree.final_layout(0).unwrap().size.width, 70.0);
}

#[test]
fn inline_block_uses_bottom_synthesized_baseline_when_child_has_no_baseline() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_block_uses_inner_last_baseline_for_atomic_alignment() {
    let measured_inline_block = ComputeOutput::from_sizes_and_baselines(
        Size::new(10.0, 30.0),
        Size::new(10.0, 30.0),
        surgeist_layout::Baselines {
            first: Point::new(None, Some(5.0)),
            last: Point::new(None, Some(25.0)),
        },
    );
    let mut tree = support::oracle_tree::OracleTree::new()
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
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .measure(1, measured_inline_block);

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().location.y, 0.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
    assert_eq!(tree.final_layout(0).unwrap().size.height, 30.0);
}

#[test]
fn inline_block_keeps_child_margins_inside_atomic_wrapper() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                margin: Edges {
                    top: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(20.0, 15.0));
    assert_eq!(tree.final_layout(2).unwrap().location.y, 5.0);
    assert_eq!(tree.final_layout(0).unwrap().size.height, 15.0);
}

#[test]
fn inline_grid_can_host_subgrid_descendant() {
    let subgrid_track = || {
        TrackComponent::Subgrid(surgeist_layout::SubgridTrack {
            name_components: Vec::new(),
        })
    };
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::InlineGrid,
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
                size: Size::new(Dimension::px(80.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(80.0, 30.0));
}

#[test]
fn block_positions_block_children_around_atomic_inline_run() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(10.0)),
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
                size: Size::new(Dimension::px(10.0), Dimension::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(15.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(10.0)),
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
    );
    round_layout(&mut tree, 0);

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
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
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
                position: Position::Absolute,
                float: Float::Left,
                size: Size::new(Dimension::px(5.0), Dimension::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

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
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

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
fn block_legacy_right_rtl_aligns_atomic_inline_run_to_physical_right_edge() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                text_align: TextAlign::LegacyRight,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

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
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                min_size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(50.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().location.x, 25.0);
}

#[test]
fn block_legacy_center_aligns_atomic_inline_run() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

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
    let mut tree = support::oracle_tree::OracleTree::new()
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
                overflow: Point::new(Overflow::Visible, Overflow::Visible),
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

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size, Size::new(95.0, 35.0));
}

#[test]
fn block_inline_run_content_size_accounts_for_negative_relative_inset_after_content() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                overflow: Point::new(Overflow::Visible, Overflow::Visible),
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

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size.height, 45.0);
}

#[test]
fn block_reports_inline_run_first_and_last_baselines() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(Available::definite(100.0)),
        },
    );

    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(20.0));
}

#[test]
fn block_reports_inline_run_baseline_including_padding() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.first_baselines.y, Some(30.0));
    assert_eq!(output.last_baselines.y, Some(30.0));
}

#[test]
fn block_definite_compute_size_keeps_inline_run_baselines() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(20.0));
}

#[test]
fn block_definite_compute_size_keeps_block_child_baselines() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        surgeist_layout::Baselines {
            first: Point::new(None, Some(7.0)),
            last: Point::new(None, Some(17.0)),
        },
    );
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
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

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(7.0));
    assert_eq!(output.last_baselines.y, Some(17.0));
}

#[test]
fn block_definite_compute_size_keeps_non_empty_flex_child_baselines() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        surgeist_layout::Baselines {
            first: Point::new(None, Some(9.0)),
            last: Point::new(None, Some(19.0)),
        },
    );
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
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

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(9.0));
    assert_eq!(output.last_baselines.y, Some(19.0));
}

#[test]
fn block_layout_stacks_in_flow_children_vertically() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                right: Length::px(5.0),
                bottom: Length::px(7.0),
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(30.0, 12.0), Size::new(30.0, 12.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformRootLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 41.0));
    assert_eq!(output.content_size, Size::new(30.0, 29.0));
    assert_eq!(tree.layouts[&2].location, Point::new(18.0, 6.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.left, 6.0);
    assert_eq!(tree.layouts[&3].location, Point::new(12.0, 21.0));
    assert_eq!(tree.layouts[&3].size, Size::new(30.0, 12.0));
    assert_eq!(tree.inputs[&2][0].parent, Size::new(Some(82.0), None));
    assert_eq!(tree.inputs[&3][0].parent, Size::new(Some(82.0), None));
}

#[test]
fn block_in_flow_calc_margin_resolves_against_containing_block_width() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
        calcs: LayoutCalcStore,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(10.0),
            ))
        }

        fn calc_resolver(&self) -> &dyn CalcResolver {
            &self.calcs
        }
    }

    let mut tree = BlockTree::default();
    let margin_left = tree.calcs.push(CalcExpression::sum([
        CalcTerm::percent(0.1),
        CalcTerm::px(-4.0),
    ]));
    let width = tree.calcs.push(CalcExpression::sum([
        CalcTerm::percent(0.5),
        CalcTerm::px(20.0),
    ]));
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::calc(width), Dimension::AUTO),
            margin: Edges {
                left: LengthAuto::calc(margin_left),
                right: LengthAuto::ZERO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(200.0), None),
            available: Size::new(Available::Definite(200.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(120.0), None));
    assert_eq!(tree.layouts[&2].location, Point::new(16.0, 0.0));
    assert_eq!(tree.layouts[&2].margin.left, 16.0);
    assert_eq!(tree.layouts[&2].size, Size::new(120.0, 10.0));
}

#[test]
fn block_auto_width_includes_in_flow_child_horizontal_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            item_is_table: true,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformRootLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(3.0, 0.0));
    assert_eq!(output.size, Size::new(32.0, 10.0));
    assert_eq!(output.content_size, Size::new(32.0, 10.0));
}

#[test]
fn block_float_contributes_to_intrinsic_width_and_places_from_right_edge() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::AUTO, Dimension::px(80.0)),
            border: Edges::all(Length::px(2.0)),
            ..NodeInput::default()
        },
    );
    for node in [2, 3, 4] {
        tree.styles.insert(
            node,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
                ..NodeInput::default()
            },
        );
        tree.outputs.insert(
            node,
            ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
        );
    }

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(154.0, 80.0));
    assert_eq!(tree.layouts[&2].location, Point::new(102.0, 2.0));
    assert_eq!(tree.layouts[&3].location, Point::new(52.0, 2.0));
    assert_eq!(tree.layouts[&4].location, Point::new(2.0, 2.0));
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(100.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats_above_full_width_float() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 200.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_overflow_clip_zero_width_child_ignores_float_exclusion_without_clear() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Clip, Overflow::Clip),
                size: Size::new(Dimension::px(0.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_right_alignment_without_float_exclusion() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(150.0, 0.0)
    );
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_center_alignment_without_float_exclusion() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(75.0, 0.0)
    );
}

#[test]
fn block_bfc_float_content_size_height_excludes_container_top_inset() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                border: Edges {
                    top: Length::px(5.0),
                    ..Edges::all(Length::ZERO)
                },
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
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(50.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 15.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().content_size.height, 30.0);
}

#[test]
fn block_bfc_clear_only_visible_child_keeps_normal_x_while_clearing_y() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(50.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(150.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                clear: surgeist_layout::Clear::Left,
                overflow: Point::new(Overflow::Visible, Overflow::Visible),
                size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(150.0, 50.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 70.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_left_sits_below_left_float_row() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: surgeist_layout::Clear::Left,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 100.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_right_sits_below_all_right_floats() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: surgeist_layout::Clear::Right,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 200.0)
    );
}

#[test]
fn block_layout_collapses_adjacent_in_flow_vertical_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 20.0));
    assert_eq!(output.size, Size::new(100.0, 30.0));
    assert_eq!(output.content_size, Size::new(100.0, 30.0));
}

#[test]
fn block_layout_collapses_first_child_top_margin_through_parent() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(output.top_margin.resolve(), 10.0);
    assert_eq!(output.bottom_margin.resolve(), 0.0);
}

#[test]
fn block_scroll_container_keeps_first_child_top_margin_inside() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 10.0));
    assert_eq!(output.size, Size::new(100.0, 15.0));
    assert_eq!(output.content_size, Size::new(100.0, 15.0));
    assert_eq!(output.top_margin.resolve(), 0.0);
    assert_eq!(output.bottom_margin.resolve(), 0.0);
    assert!(!output.margins_can_collapse_through);
}

#[test]
fn block_rtl_scrollbar_gutter_uses_left_inset() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(10.0),
            ))
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            scrollbar_width: 17.0,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(17.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(83.0, 10.0));
}

#[test]
fn block_layout_collapses_last_child_bottom_margin_through_parent() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(output.top_margin.resolve(), 0.0);
    assert_eq!(output.bottom_margin.resolve(), 10.0);
}

#[test]
fn block_layout_keeps_grid_child_margins_inside_parent_flow() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(50.0, 20.0))
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(50.0, 30.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.top, 10.0);
}

#[test]
fn block_layout_collapses_margins_through_empty_in_flow_child() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            border: Edges {
                top: Length::px(1.0),
                right: Length::ZERO,
                bottom: Length::px(1.0),
                left: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                bottom: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(7.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    let mut empty_output = ComputeOutput::from_sizes(Size::new(100.0, 0.0), Size::new(100.0, 0.0));
    empty_output.margins_can_collapse_through = true;
    tree.outputs.insert(2, empty_output);
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 11.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 11.0));
    assert_eq!(output.size, Size::new(100.0, 22.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
}

#[test]
fn block_empty_auto_height_can_collapse_through() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 0.0));
    assert!(output.margins_can_collapse_through);
}

#[test]
fn block_with_padding_reports_own_margins_when_child_collapse_is_blocked() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            margin: Edges {
                top: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            padding: Edges {
                top: Length::px(1.0),
                bottom: Length::px(1.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 2.0));
    assert_eq!(output.top_margin.resolve(), 8.0);
    assert_eq!(output.bottom_margin.resolve(), 6.0);
    assert!(!output.margins_can_collapse_through);
}

#[test]
fn block_layout_positions_in_flow_children_from_right_edge_in_rtl() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            padding: Edges {
                top: Length::ZERO,
                right: Length::px(5.0),
                bottom: Length::ZERO,
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(7.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(tree.layouts[&2].location, Point::new(67.0, 1.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.right, 7.0);
}

#[test]
fn block_layout_expands_horizontal_auto_margins_for_in_flow_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::AUTO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::AUTO,
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
}

#[test]
fn block_content_size_includes_visible_child_overflow_content() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 10.0));
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn block_relative_child_inset_offsets_final_layout_location() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(13.0, 3.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn block_layout_stretches_auto_width_in_flow_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_sizes(
                Size::new(input.known.width.unwrap(), 10.0),
                Size::new(input.known.width.unwrap(), 10.0),
            )
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            padding: Edges {
                top: Length::ZERO,
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                left: LengthAuto::px(3.0),
                right: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.inputs[&2][0].known.width, Some(76.0));
    assert_eq!(tree.layouts[&2].size, Size::new(76.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(8.0, 0.0));
    assert_eq!(output.content_size, Size::new(88.0, 10.0));
    assert_eq!(output.size, Size::new(100.0, 10.0));
}

#[test]
fn block_compute_size_uses_in_flow_children_for_auto_height() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_sizes(
                Size::new(input.known.width.unwrap(), 10.0),
                Size::new(input.known.width.unwrap(), 10.0),
            )
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::px(7.0),
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.inputs[&2][0].run_mode, RunMode::ComputeSize);
    assert_eq!(tree.inputs[&2][0].known.width, Some(76.0));
    assert_eq!(output.size, Size::new(100.0, 26.0));
    assert_eq!(output.content_size, Size::ZERO);
    assert!(tree.layouts.is_empty());
}

#[test]
fn block_compute_size_uses_definite_min_max_without_measuring_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            panic!("definite min/max compute-size should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            min_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            max_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn block_definite_compute_size_keeps_grid_children_on_fast_path_until_grid_baselines() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            panic!("definite grid compute-size should stay on the fast path")
        }
    }

    for display in [Display::Grid, Display::GridLanes] {
        let mut tree = BlockTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![3]);
        tree.children.insert(3, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                min_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
                max_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInput {
                display,
                ..NodeInput::default()
            },
        );
        tree.styles.insert(3, NodeInput::default());

        let output = surgeist_layout::compute_block(
            &mut tree,
            1,
            ComputeInput {
                run_mode: RunMode::ComputeSize,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(Some(500.0), Some(400.0)),
                available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
            },
        );

        assert_eq!(output.size, Size::new(100.0, 40.0));
        assert_eq!(output.content_size, Size::ZERO);
    }
}

#[test]
fn block_auto_height_clamps_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            max_size: Size::new(Dimension::AUTO, Dimension::px(12.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 20.0), Size::new(100.0, 20.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 20.0));
}

#[test]
fn block_auto_size_applies_aspect_ratio_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            aspect_ratio: Some(2.0),
            max_size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(50.0, 25.0));
}

#[test]
fn block_legacy_text_align_offsets_table_child_in_free_inline_space() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_outer_size(Size::new(60.0, 10.0))
        }
    }

    fn run(text_align: TextAlign, direction: Direction) -> NodeOutput {
        let mut tree = BlockTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                direction,
                text_align,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInput {
                display: Display::Block,
                item_is_table: true,
                ..NodeInput::default()
            },
        );

        surgeist_layout::compute_block(
            &mut tree,
            1,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(Some(300.0), Some(200.0)),
                available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
            },
        );

        tree.layouts[&2]
    }

    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Ltr).location.x,
        70.0
    );
    assert_eq!(
        run(TextAlign::LegacyRight, Direction::Ltr).location.x,
        140.0
    );
    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Rtl).location.x,
        70.0
    );
    assert_eq!(run(TextAlign::LegacyLeft, Direction::Rtl).location.x, 0.0);
}

#[test]
fn block_layout_lays_out_absolute_children_without_flow_contribution_and_hides_display_none() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            if input.run_mode == RunMode::PerformHiddenLayout {
                ComputeOutput::HIDDEN
            } else {
                self.outputs[&node]
            }
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        4,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(40.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(80.0, 32.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(87.0, 41.0));
    assert_eq!(tree.layouts[&2].location, Point::new(1.0, 1.0));
    assert_eq!(tree.layouts[&3].location, Point::new(8.0, 10.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&4], NodeOutput::with_order(2));
    assert_eq!(tree.inputs[&4], vec![ComputeInput::HIDDEN]);
}

#[test]
fn block_absolute_child_without_insets_uses_static_position_after_flow() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(98.0, 10.0), Size::new(98.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 5.0), Size::new(20.0, 5.0)),
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(tree.layouts[&2].location, Point::new(1.0, 1.0));
    assert_eq!(tree.layouts[&3].location, Point::new(1.0, 11.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 5.0));
}

#[test]
fn block_absolute_child_auto_size_applies_aspect_ratio_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            aspect_ratio: Some(2.0),
            max_size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 25.0));
}

#[test]
fn block_absolute_child_auto_size_resolves_from_opposing_insets() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            output_from_known_or(input, Size::ZERO)
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            ..NodeInput::default()
        },
    );

    let output = surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(74.0), Some(24.0)));
    assert_eq!(tree.layouts[&2].location, Point::new(8.0, 14.0));
    assert_eq!(tree.layouts[&2].size, Size::new(74.0, 24.0));
}

#[test]
fn block_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known.width.unwrap_or(0.0),
                input.known.height.unwrap_or(0.0),
            ))
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            aspect_ratio: Some(2.0),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(80.0), Some(40.0)));
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 40.0));
}

#[test]
fn block_absolute_child_expands_horizontal_auto_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            output_from_known_or(input, Size::ZERO)
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(20.0), Some(10.0)));
    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn block_absolute_child_large_width_keeps_horizontal_auto_margins_zero() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0))
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(Dimension::px(70.0), Dimension::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(70.0), Some(10.0)));
    assert_eq!(tree.layouts[&2].margin.left, 0.0);
    assert_eq!(tree.layouts[&2].margin.right, 0.0);
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(70.0, 10.0));
}

#[test]
fn block_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            output_from_known_or(input, Size::ZERO)
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
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
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    surgeist_layout::compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.inputs[&2][0].known, Size::new(Some(20.0), Some(10.0)));
    assert_eq!(tree.layouts[&2].location, Point::new(62.0, 1.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}
