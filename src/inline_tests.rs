use crate::Available;
use crate::inline::{
    AtomicInlineInput, AtomicInlineItem, AtomicInlineLayoutItemKind,
    atomic_inline_max_content_width, atomic_inline_min_content_width, layout_atomic_inline_items,
};
use crate::*;

#[test]
fn atomic_inline_line_aligns_items_to_max_baseline() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(200.0),
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(7.0)),
            AtomicInlineItem::new(1, Size::new(10.0, 20.0), Edges::ZERO, Some(12.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.first_baseline, Some(12.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 5.0));
    assert_eq!(report.items[1].location, Point::new(20.0, 0.0));
}

#[test]
fn atomic_inline_items_wrap_between_items_for_definite_width() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(25.0),
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 20.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(20.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
}

#[test]
fn atomic_inline_line_geometry_clamps_item_baseline_to_its_box() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(124.0, 64.0), Edges::ZERO, Some(94.0)),
            AtomicInlineItem::new(1, Size::new(10.0, 0.0), Edges::ZERO, Some(0.0)),
        ],
    });

    assert_eq!(report.size, Size::new(134.0, 64.0));
    assert_eq!(report.first_baseline, Some(64.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(124.0, 64.0));
}

#[test]
fn atomic_inline_min_content_available_wraps_to_max_item_advance() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MIN_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(40.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::new(1, Size::new(60.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::new(2, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(60.0, 30.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(30.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 20.0));
}

#[test]
fn atomic_inline_intrinsic_widths_use_max_item_and_sum() {
    let items = vec![
        AtomicInlineItem::new(
            0,
            Size::new(25.0, 10.0),
            Edges::new(0.0, 5.0, 0.0, 5.0),
            Some(10.0),
        ),
        AtomicInlineItem::new(
            1,
            Size::new(100.0, 10.0),
            Edges::new(0.0, 0.0, 0.0, 10.0),
            Some(10.0),
        ),
        AtomicInlineItem::new(2, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
    ];

    assert_eq!(atomic_inline_min_content_width(&items), 110.0);
    assert_eq!(atomic_inline_max_content_width(&items), 195.0);
}

#[test]
fn atomic_inline_forced_line_break_starts_next_line() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::forced_line_break(1),
            AtomicInlineItem::new(2, Size::new(15.0, 12.0), Edges::ZERO, Some(8.0)),
            AtomicInlineItem::forced_line_break(3),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 22.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(18.0));
    assert_eq!(report.items.len(), 4);
    assert_eq!(report.items[0].kind, AtomicInlineLayoutItemKind::Box);
    assert_eq!(report.items[0].location, Point::new(0.0, 0.0));
    assert_eq!(
        report.items[1].kind,
        AtomicInlineLayoutItemKind::ForcedLineBreak
    );
    assert_eq!(report.items[1].location, Point::new(20.0, 10.0));
    assert_eq!(report.items[1].size, Size::ZERO);
    assert_eq!(report.items[2].kind, AtomicInlineLayoutItemKind::Box);
    assert_eq!(report.items[2].location, Point::new(0.0, 10.0));
    assert_eq!(
        report.items[3].kind,
        AtomicInlineLayoutItemKind::ForcedLineBreak
    );
    assert_eq!(report.items[3].location, Point::new(15.0, 18.0));
}

#[test]
fn atomic_inline_intrinsic_widths_split_at_forced_line_breaks() {
    let items = vec![
        AtomicInlineItem::new(
            0,
            Size::new(25.0, 10.0),
            Edges::new(0.0, 5.0, 0.0, 5.0),
            Some(10.0),
        ),
        AtomicInlineItem::forced_line_break(1),
        AtomicInlineItem::new(
            2,
            Size::new(100.0, 10.0),
            Edges::new(0.0, 0.0, 0.0, 10.0),
            Some(10.0),
        ),
        AtomicInlineItem::new(3, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
        AtomicInlineItem::forced_line_break(4),
    ];

    assert_eq!(atomic_inline_min_content_width(&items), 110.0);
    assert_eq!(atomic_inline_max_content_width(&items), 160.0);
}

#[test]
fn atomic_inline_vertical_margins_participate_in_line_metrics() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![AtomicInlineItem::new(
            0,
            Size::new(20.0, 10.0),
            Edges::new(3.0, 0.0, 7.0, 0.0),
            Some(6.0),
        )],
    });

    assert_eq!(report.size, Size::new(20.0, 20.0));
    assert_eq!(report.first_baseline, Some(9.0));
    assert_eq!(report.last_baseline, Some(9.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 3.0));
}

#[test]
fn atomic_inline_vertical_rl_places_line_against_right_edge() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(70.0),
        writing_mode: WritingMode::VerticalRl,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
            AtomicInlineItem::new(1, Size::new(10.0, 0.0), Edges::ZERO, Some(0.0)),
            AtomicInlineItem::new(2, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
        ],
    });

    assert_eq!(report.size, Size::new(70.0, 40.0));
    assert_eq!(report.items[0].location, Point::new(50.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(65.0, 20.0));
    assert_eq!(report.items[2].location, Point::new(50.0, 20.0));
}

#[test]
fn atomic_inline_empty_items_report_zero_size_and_no_baselines() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: Vec::new(),
    });

    assert_eq!(report.size, Size::ZERO);
    assert_eq!(report.content_size, Size::ZERO);
    assert_eq!(report.first_baseline, None);
    assert_eq!(report.last_baseline, None);
    assert!(report.items.is_empty());
}

mod root_oracle {
    use crate::test_support::oracle::{grid, inline};

    #[test]
    fn oracle_atomic_inline_item_metrics_include_margins_and_baseline() {
        let item = inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges {
                top: 2.0,
                right: 3.0,
                bottom: 4.0,
                left: 5.0,
            },
            first_baseline: Some(7.0),
        };

        let metrics = inline::AtomicInlineMetrics::from_item(item);

        assert_eq!(metrics.advance, 28.0);
        assert_eq!(metrics.baseline, 9.0);
        assert_eq!(metrics.descent, 7.0);
        assert_eq!(metrics.margin_box_size, inline::InlineSize::new(28.0, 16.0));
    }

    #[test]
    fn oracle_atomic_inline_item_synthesizes_missing_baseline_from_bottom_edge() {
        let item = inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: None,
        };

        let metrics = inline::AtomicInlineMetrics::from_item(item);

        assert_eq!(metrics.baseline, 10.0);
        assert_eq!(metrics.descent, 0.0);
        assert!(metrics.synthesized_baseline);
    }

    #[test]
    fn oracle_atomic_inline_item_rejects_baseline_before_top_edge() {
        let item = inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(-1.0),
        };

        assert_eq!(
            inline::AtomicInlineMetrics::try_from_item(item),
            Err(inline::AtomicInlineError::BaselineOutOfRange {
                id: "a",
                first_baseline: -1.0,
                height: 10.0,
            })
        );
    }

    #[test]
    fn oracle_atomic_inline_item_rejects_baseline_after_bottom_edge() {
        let item = inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(11.0),
        };

        assert_eq!(
            inline::AtomicInlineMetrics::try_from_item(item),
            Err(inline::AtomicInlineError::BaselineOutOfRange {
                id: "a",
                first_baseline: 11.0,
                height: 10.0,
            })
        );
    }

    #[test]
    fn oracle_atomic_inline_line_aligns_items_to_max_baseline() {
        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(200.0),
            items: vec![
                inline::AtomicInlineItemFacts {
                    id: "short",
                    size: inline::InlineSize::new(20.0, 10.0),
                    margin: inline::InlineEdges::ZERO,
                    first_baseline: Some(7.0),
                },
                inline::AtomicInlineItemFacts {
                    id: "tall",
                    size: inline::InlineSize::new(10.0, 20.0),
                    margin: inline::InlineEdges::ZERO,
                    first_baseline: Some(12.0),
                },
            ],
        });

        assert_eq!(report.size, inline::InlineSize::new(30.0, 20.0));
        assert_eq!(report.first_baseline, Some(12.0));
        assert_eq!(report.last_baseline, Some(12.0));
        assert_eq!(report.lines.len(), 1);
        assert_eq!(
            report.lines[0],
            inline::AtomicInlineLine {
                start_item: 0,
                end_item: 2,
                y: 0.0,
                width: 30.0,
                height: 20.0,
                baseline: 12.0,
                descent: 8.0,
            }
        );
        assert_eq!(report.items[0].id, "short");
        assert_eq!(report.items[0].location, inline::InlinePoint::new(0.0, 5.0));
        assert_eq!(
            report.items[1].location,
            inline::InlinePoint::new(20.0, 0.0)
        );
    }

    #[test]
    fn oracle_atomic_inline_line_positions_margin_boxes_and_border_boxes() {
        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(200.0),
            items: vec![
                inline::AtomicInlineItemFacts {
                    id: "a",
                    size: inline::InlineSize::new(20.0, 10.0),
                    margin: inline::InlineEdges {
                        top: 2.0,
                        right: 3.0,
                        bottom: 4.0,
                        left: 5.0,
                    },
                    first_baseline: Some(7.0),
                },
                inline::AtomicInlineItemFacts {
                    id: "b",
                    size: inline::InlineSize::new(10.0, 20.0),
                    margin: inline::InlineEdges {
                        top: 1.0,
                        right: 2.0,
                        bottom: 3.0,
                        left: 4.0,
                    },
                    first_baseline: Some(12.0),
                },
            ],
        });

        assert_eq!(report.size, inline::InlineSize::new(44.0, 24.0));
        assert_eq!(
            report.lines[0],
            inline::AtomicInlineLine {
                start_item: 0,
                end_item: 2,
                y: 0.0,
                width: 44.0,
                height: 24.0,
                baseline: 13.0,
                descent: 11.0,
            }
        );
        assert_eq!(report.items[0].location, inline::InlinePoint::new(5.0, 6.0));
        assert_eq!(
            report.items[1].location,
            inline::InlinePoint::new(32.0, 1.0)
        );
    }

    #[test]
    fn oracle_atomic_inline_wraps_between_items_for_definite_width() {
        let item = |id| inline::AtomicInlineItemFacts {
            id,
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        };

        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(25.0),
            items: vec![item("a"), item("b")],
        });

        assert_eq!(report.size, inline::InlineSize::new(20.0, 20.0));
        assert_eq!(report.lines.len(), 2);
        assert_eq!(
            report.lines[0],
            inline::AtomicInlineLine {
                start_item: 0,
                end_item: 1,
                y: 0.0,
                width: 20.0,
                height: 10.0,
                baseline: 10.0,
                descent: 0.0,
            }
        );
        assert_eq!(
            report.lines[1],
            inline::AtomicInlineLine {
                start_item: 1,
                end_item: 2,
                y: 10.0,
                width: 20.0,
                height: 10.0,
                baseline: 10.0,
                descent: 0.0,
            }
        );
        assert_eq!(report.items[0].location, inline::InlinePoint::new(0.0, 0.0));
        assert_eq!(
            report.items[1].location,
            inline::InlinePoint::new(0.0, 10.0)
        );
        assert_eq!(report.first_baseline, Some(10.0));
        assert_eq!(report.last_baseline, Some(20.0));
    }

    #[test]
    fn oracle_atomic_inline_intrinsic_widths_use_max_item_and_sum() {
        let items = vec![
            inline::AtomicInlineItemFacts {
                id: "a",
                size: inline::InlineSize::new(25.0, 10.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(10.0),
            },
            inline::AtomicInlineItemFacts {
                id: "b",
                size: inline::InlineSize::new(100.0, 10.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(10.0),
            },
            inline::AtomicInlineItemFacts {
                id: "c",
                size: inline::InlineSize::new(95.0, 10.0),
                margin: inline::InlineEdges {
                    left: 10.0,
                    right: 7.0,
                    ..inline::InlineEdges::ZERO
                },
                first_baseline: Some(10.0),
            },
        ];

        assert_eq!(inline::atomic_inline_min_content_width(&items), 112.0);
        assert_eq!(inline::atomic_inline_max_content_width(&items), 237.0);
    }

    #[test]
    fn oracle_atomic_inline_min_content_wraps_at_max_item_advance() {
        let items = vec![
            inline::AtomicInlineItemFacts {
                id: "wide",
                size: inline::InlineSize::new(95.0, 10.0),
                margin: inline::InlineEdges {
                    left: 10.0,
                    right: 7.0,
                    ..inline::InlineEdges::ZERO
                },
                first_baseline: Some(10.0),
            },
            inline::AtomicInlineItemFacts {
                id: "next",
                size: inline::InlineSize::new(50.0, 10.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(10.0),
            },
        ];

        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::MinContent,
            items,
        });

        assert_eq!(report.size, inline::InlineSize::new(112.0, 20.0));
        assert_eq!(report.lines.len(), 2);
        assert_eq!(report.lines[0].width, 112.0);
        assert_eq!(report.lines[1].width, 50.0);
    }

    #[test]
    fn oracle_atomic_inline_too_wide_item_overflows_without_empty_line() {
        let item = |id, width| inline::AtomicInlineItemFacts {
            id,
            size: inline::InlineSize::new(width, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        };

        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(25.0),
            items: vec![item("wide", 40.0), item("next", 10.0)],
        });

        assert_eq!(report.size, inline::InlineSize::new(40.0, 20.0));
        assert_eq!(report.lines.len(), 2);
        assert_eq!(report.lines[0].start_item, 0);
        assert_eq!(report.lines[0].end_item, 1);
        assert_eq!(report.lines[0].width, 40.0);
        assert_eq!(report.lines[1].start_item, 1);
        assert_eq!(report.lines[1].end_item, 2);
        assert_eq!(report.lines[1].width, 10.0);
    }

    #[test]
    fn oracle_atomic_inline_wrapper_preserves_outer_and_inner_display_roles() {
        let cases = [
            (
                inline::InlineOuterDisplay::Block,
                inline::InnerFormattingContext::Block,
            ),
            (
                inline::InlineOuterDisplay::Grid,
                inline::InnerFormattingContext::Grid,
            ),
            (
                inline::InlineOuterDisplay::GridLanes,
                inline::InnerFormattingContext::GridLanes,
            ),
        ];

        for (outer_display, inner_context) in cases {
            let wrapper = inline::AtomicInlineWrapperFacts::new(
                "wrapper",
                outer_display,
                inline::InlineSize::new(40.0, 20.0),
                inline::InlineEdges::ZERO,
                Some(15.0),
            );

            assert_eq!(wrapper.outer_display, outer_display);
            assert_eq!(wrapper.inner_context(), inner_context);
            assert_eq!(
                wrapper.as_item(),
                inline::AtomicInlineItemFacts {
                    id: "wrapper",
                    size: inline::InlineSize::new(40.0, 20.0),
                    margin: inline::InlineEdges::ZERO,
                    first_baseline: Some(15.0),
                }
            );
        }
    }

    #[test]
    fn oracle_atomic_inline_wrapper_metrics_use_outer_box_and_margins() {
        let cases = [
            inline::InlineOuterDisplay::Block,
            inline::InlineOuterDisplay::Grid,
            inline::InlineOuterDisplay::GridLanes,
        ];

        for outer_display in cases {
            let wrapper = inline::AtomicInlineWrapperFacts::new(
                "wrapper",
                outer_display,
                inline::InlineSize::new(40.0, 20.0),
                inline::InlineEdges {
                    top: 2.0,
                    right: 3.0,
                    bottom: 4.0,
                    left: 5.0,
                },
                Some(15.0),
            );

            let metrics = inline::AtomicInlineMetrics::from_item(wrapper.as_item());

            assert_eq!(metrics.advance, 48.0);
            assert_eq!(metrics.baseline, 17.0);
            assert_eq!(metrics.descent, 9.0);
            assert_eq!(metrics.margin_box_size, inline::InlineSize::new(48.0, 26.0));
        }
    }

    #[test]
    fn oracle_atomic_inline_wrapper_produces_grid_contribution_facts() {
        let wrapper = inline::AtomicInlineWrapperFacts::new(
            "inline-grid",
            inline::InlineOuterDisplay::Grid,
            inline::InlineSize::new(80.0, 30.0),
            inline::InlineEdges::ZERO,
            Some(24.0),
        );

        let contribution = inline::atomic_inline_grid_item_facts(
            wrapper,
            grid::GridArea::new(1, 1, 1, 1),
            80.0,
            80.0,
        );

        assert_eq!(contribution.id, "inline-grid");
        assert_eq!(contribution.outer_display, inline::InlineOuterDisplay::Grid);
        assert_eq!(
            contribution.inner_context(),
            inline::InnerFormattingContext::Grid
        );
        assert_eq!(contribution.item.area, grid::GridArea::new(1, 1, 1, 1));
        assert_eq!(contribution.item.min_content, 80.0);
        assert_eq!(contribution.item.max_content, 80.0);
        assert_eq!(
            contribution.item.preferred,
            grid::ContributionSize::Definite(80.0)
        );
        assert_eq!(contribution.item.margin_before, 0.0);
        assert_eq!(contribution.item.margin_after, 0.0);
        assert_eq!(contribution.item.contributions().max_content, 80.0);
    }

    #[test]
    fn oracle_atomic_inline_grid_lanes_contribution_preserves_margins() {
        let wrapper = inline::AtomicInlineWrapperFacts::new(
            "inline-grid-lanes",
            inline::InlineOuterDisplay::GridLanes,
            inline::InlineSize::new(80.0, 30.0),
            inline::InlineEdges {
                left: 5.0,
                right: 7.0,
                ..inline::InlineEdges::ZERO
            },
            Some(24.0),
        );

        let contribution = inline::atomic_inline_grid_item_facts(
            wrapper,
            grid::GridArea::new(1, 1, 1, 1),
            60.0,
            80.0,
        );

        assert_eq!(contribution.id, "inline-grid-lanes");
        assert_eq!(
            contribution.outer_display,
            inline::InlineOuterDisplay::GridLanes
        );
        assert_eq!(
            contribution.inner_context(),
            inline::InnerFormattingContext::GridLanes
        );
        assert_eq!(contribution.item.margin_before, 5.0);
        assert_eq!(contribution.item.margin_after, 7.0);
        assert_eq!(contribution.item.contributions().min_content, 72.0);
        assert_eq!(contribution.item.contributions().max_content, 92.0);
    }
}

mod root_layout_oracle {
    use crate::test_support::layout_tree::OracleTree;
    use crate::test_support::oracle::inline;
    use crate::{
        Available, Dimension, Display, NodeInput, Size, TrackComponent, compute_root, round_layout,
    };

    fn assert_atomic_inline_layout_matches_oracle(display: Display) {
        let item_sizes = [
            ("first", Size::new(20.0, 10.0)),
            ("second", Size::new(30.0, 20.0)),
            ("third", Size::new(15.0, 10.0)),
        ];
        let mut tree = OracleTree::new().children(0, [1, 2, 3]).style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(50.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        );

        for (node, (_, size)) in (1_u32..).zip(item_sizes) {
            tree = tree.style(node, atomic_inline_node_input(display, size));
        }

        compute_root(&mut tree, 0, Size::splat(Available::definite(50.0)));
        round_layout(&mut tree, 0);

        let expected = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(50.0),
            items: item_sizes
                .into_iter()
                .map(|(id, size)| inline::AtomicInlineItemFacts {
                    id,
                    size: inline::InlineSize::new(size.width, size.height),
                    margin: inline::InlineEdges::ZERO,
                    first_baseline: None,
                })
                .collect(),
        });

        let root = tree.final_layout(0).expect("root layout");
        assert_layout_close(root.size.width, 50.0, "root width");
        assert_layout_close(root.size.height, expected.size.height, "root height");

        for (index, expected_item) in expected.items.iter().enumerate() {
            let node = (index + 1) as u32;
            let actual = tree.final_layout(node).expect("child layout");
            assert_layout_close(
                actual.location.x,
                expected_item.location.x,
                &format!("node {node} x"),
            );
            assert_layout_close(
                actual.location.y,
                expected_item.location.y,
                &format!("node {node} y"),
            );
            assert_layout_close(
                actual.size.width,
                expected_item.size.width,
                &format!("node {node} width"),
            );
            assert_layout_close(
                actual.size.height,
                expected_item.size.height,
                &format!("node {node} height"),
            );
        }
    }

    fn atomic_inline_node_input(display: Display, size: Size<f32>) -> NodeInput {
        match display.inner_display() {
            Display::Grid => NodeInput {
                display,
                grid_template_columns: vec![TrackComponent::px(size.width)],
                grid_template_rows: vec![TrackComponent::px(size.height)],
                ..NodeInput::DEFAULT
            },
            Display::GridLanes => NodeInput {
                display,
                grid_template_columns: vec![TrackComponent::px(size.width)],
                grid_template_rows: vec![TrackComponent::px(size.height)],
                ..NodeInput::DEFAULT
            },
            _ => NodeInput {
                display,
                size: Size::new(Dimension::px(size.width), Dimension::px(size.height)),
                ..NodeInput::DEFAULT
            },
        }
    }

    fn assert_layout_close(actual: f32, expected: f32, label: &str) {
        assert!(
            (actual - expected).abs() <= 0.000_1,
            "{label}: expected {expected}, got {actual}"
        );
    }

    #[test]
    fn oracle_layout_inline_block_line_matches_layout() {
        assert_atomic_inline_layout_matches_oracle(Display::InlineBlock);
    }

    #[test]
    fn oracle_layout_inline_grid_line_matches_layout() {
        assert_atomic_inline_layout_matches_oracle(Display::InlineGrid);
    }

    #[test]
    fn oracle_layout_inline_grid_lanes_line_matches_layout() {
        assert_atomic_inline_layout_matches_oracle(Display::InlineGridLanes);
    }
}
