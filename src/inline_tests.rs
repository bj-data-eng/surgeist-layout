use crate::Available;
use crate::inline::{
    AtomicInlineBoxParticipant, InlineControlAlignment, InlineParticipant,
    InlineParticipantLayoutKind, InlineRunInput, LogicalLineBandQueryResultOf,
    MixedInlineParticipantOf, MixedInlineRunInputOf, PostLineClearIntent,
    inline_candidate_scan_visits, inline_run_max_content_width, inline_run_min_content_width,
    layout_inline_run, layout_mixed_inline_run, layout_mixed_inline_run_with_band_source,
    mapped_post_line_clear_intent, reset_inline_candidate_scan_visits,
};
use crate::*;

fn mr02_text<S: LayoutScalar>(
    source_index: usize,
    id: u64,
    extent: f64,
    bidi_level: u8,
    whitespace_edge: InlineWhitespaceEdge,
    following_break: InlineBreakOpportunityOf<S>,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::ShapedText(crate::inline::ShapedTextParticipantOf {
        source_index,
        segment: ShapedInlineSegmentOf::try_new(
            InlineSegmentId::new(id),
            S::from_f64(extent),
            InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0)).unwrap(),
            BidiLevel::try_new(bidi_level).unwrap(),
            whitespace_edge,
            following_break,
        )
        .unwrap(),
    })
}

fn mr02_atomic<S: LayoutScalar>(
    source_index: usize,
    extent: f64,
    block_extent: f64,
    baseline: f64,
    bidi_level: u8,
    following_break: InlineBreakOpportunityOf<S>,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::Atomic {
        item: AtomicInlineBoxParticipant {
            source_index,
            size: Size::new(S::from_f64(extent), S::from_f64(block_extent)),
            content_size: Size::new(S::from_f64(extent), S::from_f64(block_extent)),
            margin: Edges::ZERO,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline: Some(S::from_f64(baseline)),
            alignment: InlineControlAlignment::Baseline,
        },
        participation: AtomicInlineParticipationOf::try_new(
            BidiLevel::try_new(bidi_level).unwrap(),
            following_break,
        )
        .unwrap(),
    }
}

fn mr02_input<S: LayoutScalar>(
    available_inline_extent: AvailableOf<S>,
    participants: Vec<MixedInlineParticipantOf<S>>,
) -> MixedInlineRunInputOf<S> {
    MixedInlineRunInputOf {
        available_inline_extent,
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        text_align: TextAlign::Auto,
        participants,
    }
}

#[test]
fn fri06_mr02_inline_linear_empty_strut_and_forced_clear_preserve_metrics_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let metrics =
            InlineMetricsOf::from_ascent_descent(S::from_f64(9.0), S::from_f64(3.0)).unwrap();
        let report = layout_mixed_inline_run(mr02_input(
            AvailableOf::MAX_CONTENT,
            vec![mixed_forced_line_break(7, flow_axes, metrics, Clear::Both)],
        ));

        assert_eq!(report.inline_extent, S::ZERO);
        assert_eq!(report.block_extent, S::from_f64(24.0));
        assert_eq!(report.first_baseline, Some(S::from_f64(9.0)));
        assert_eq!(report.last_baseline, Some(S::from_f64(21.0)));
        assert!(report.fragments.is_empty());
        assert!(report.anchors.is_empty());
        assert_eq!(report.controls.len(), 1);
        assert_eq!(report.controls[0].source_index, 7);
        assert_eq!(report.controls[0].line_index, 0);
        assert_eq!(report.controls[0].visual_index, None);
        assert_eq!(
            report.post_line_clear_intents,
            [PostLineClearIntent::Both, PostLineClearIntent::None]
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr02_inline_linear_discard_break_replacement_and_overwide_progress_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let discarded = layout_mixed_inline_run(mr02_input(
            AvailableOf::definite(S::from_f64(15.0)),
            vec![
                mr02_text(
                    10,
                    10,
                    4.0,
                    0,
                    InlineWhitespaceEdge::DiscardAtLineStart,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                mr02_text(
                    11,
                    11,
                    10.0,
                    0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::allowed(),
                ),
                mr02_text(
                    12,
                    12,
                    5.0,
                    0,
                    InlineWhitespaceEdge::DiscardAtLineEnd,
                    InlineBreakOpportunityOf::allowed(),
                ),
                mr02_text(
                    13,
                    13,
                    30.0,
                    0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ],
        ));
        assert_eq!(discarded.fragments.len(), 2);
        assert_eq!(discarded.fragments[0].source_index, 11);
        assert_eq!(discarded.fragments[0].inline_start, S::ZERO);
        assert_eq!(discarded.fragments[0].line_index, 0);
        assert_eq!(discarded.fragments[1].source_index, 13);
        assert_eq!(discarded.fragments[1].line_index, 1);
        assert_eq!(discarded.fragments[1].inline_extent, S::from_f64(30.0));
        assert_eq!(discarded.anchors.len(), 4);
        assert_eq!(discarded.anchors[0].source_index, 10);
        assert_eq!(discarded.anchors[0].inline_start, S::ZERO);
        assert_eq!(discarded.anchors[2].source_index, 12);
        assert_eq!(discarded.anchors[2].inline_start, S::from_f64(10.0));
        assert_eq!(discarded.line_bands.len(), 2);

        let replacement = S::from_f64(4.0);
        let replaced = layout_mixed_inline_run(mr02_input(
            AvailableOf::definite(S::from_f64(15.0)),
            vec![
                mr02_text(
                    20,
                    20,
                    12.0,
                    0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::try_allowed_with_replacement(replacement).unwrap(),
                ),
                mr02_text(
                    21,
                    21,
                    20.0,
                    0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ],
        ));
        assert_eq!(replaced.fragments.len(), 2);
        assert_eq!(replaced.fragments[0].line_index, 0);
        assert_eq!(
            replaced.fragments[0].replacement_inline_extent,
            Some(replacement)
        );
        assert_eq!(replaced.fragments[1].line_index, 1);
        assert_eq!(replaced.fragments[1].replacement_inline_extent, None);
        assert_eq!(replaced.inline_extent, S::from_f64(20.0));

        let endpoint_breaks = layout_mixed_inline_run(mr02_input(
            AvailableOf::definite(S::from_f64(5.0)),
            vec![
                mr02_text(
                    30,
                    30,
                    5.0,
                    0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::allowed(),
                ),
                mr02_text(
                    31,
                    31,
                    6.0,
                    0,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::mandatory(),
                ),
            ],
        ));
        assert_eq!(endpoint_breaks.fragments.len(), 2);
        assert_eq!(endpoint_breaks.fragments[0].line_index, 0);
        assert_eq!(endpoint_breaks.fragments[1].line_index, 1);
        assert_eq!(endpoint_breaks.line_bands.len(), 3);
        assert_eq!(endpoint_breaks.first_baseline, Some(S::from_f64(8.0)));
        assert_eq!(endpoint_breaks.last_baseline, Some(S::from_f64(28.0)));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr02_inline_linear_bidi_mixed_sources_preserve_visual_order_and_baselines_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let boundary_metrics =
            InlineMetricsOf::from_ascent_descent(S::from_f64(6.0), S::from_f64(2.0)).unwrap();
        let report = layout_mixed_inline_run(mr02_input(
            AvailableOf::MAX_CONTENT,
            vec![
                mr02_text(
                    40,
                    40,
                    10.0,
                    1,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                mixed_boundary(
                    41,
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
                    boundary_metrics,
                ),
                mr02_atomic(
                    42,
                    5.0,
                    12.0,
                    9.0,
                    2,
                    InlineBreakOpportunityOf::prohibited(),
                ),
                mr02_text(
                    43,
                    43,
                    7.0,
                    1,
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                ),
            ],
        ));

        assert_eq!(report.inline_extent, S::from_f64(22.0));
        assert_eq!(report.block_extent, S::from_f64(12.0));
        assert_eq!(report.first_baseline, Some(S::from_f64(9.0)));
        assert_eq!(report.last_baseline, Some(S::from_f64(9.0)));
        assert_eq!(report.fragments.len(), 2);
        assert_eq!(report.fragments[0].source_index, 40);
        assert_eq!(report.fragments[0].inline_start, S::from_f64(12.0));
        assert_eq!(report.fragments[0].block_start, S::from_f64(1.0));
        assert_eq!(report.fragments[0].visual_index, 3);
        assert_eq!(report.fragments[1].source_index, 43);
        assert_eq!(report.fragments[1].inline_start, S::ZERO);
        assert_eq!(report.fragments[1].visual_index, 0);
        assert_eq!(report.atomics.len(), 1);
        assert_eq!(report.atomics[0].item.source_index, 42);
        assert_eq!(report.atomics[0].inline_start, S::from_f64(7.0));
        assert_eq!(report.atomics[0].visual_index, 1);
        assert_eq!(report.controls.len(), 1);
        assert_eq!(report.controls[0].source_index, 41);
        assert_eq!(report.controls[0].inline_start, S::from_f64(12.0));
        assert_eq!(report.controls[0].visual_index, Some(2));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn mr02_intrinsic_extents<S: LayoutScalar>() -> (S, S) {
    let replacement = S::from_f64(0.05);
    let participants = vec![
        mr02_text(
            50,
            50,
            100.0,
            0,
            InlineWhitespaceEdge::DiscardAtLineStart,
            InlineBreakOpportunityOf::prohibited(),
        ),
        mr02_text(
            51,
            51,
            0.1,
            0,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::prohibited(),
        ),
        mr02_text(
            52,
            52,
            0.2,
            0,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::try_allowed_with_replacement(replacement).unwrap(),
        ),
        mr02_text(
            53,
            53,
            0.3,
            0,
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::mandatory(),
        ),
        mixed_forced_line_break(
            54,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0)).unwrap(),
            Clear::None,
        ),
        mr02_text(
            55,
            55,
            0.4,
            0,
            InlineWhitespaceEdge::DiscardAtLineEnd,
            InlineBreakOpportunityOf::prohibited(),
        ),
    ];
    let min = layout_mixed_inline_run(mr02_input(AvailableOf::MIN_CONTENT, participants.clone()));
    let max = layout_mixed_inline_run(mr02_input(AvailableOf::MAX_CONTENT, participants));
    (min.inline_extent, max.inline_extent)
}

#[test]
fn fri06_mr02_inline_linear_intrinsic_extents_preserve_scalar_addition_order_both_scalars() {
    let (min_f32, max_f32) = mr02_intrinsic_extents::<f32>();
    let expected_min_f32 = ((0.0_f32 + 0.1) + 0.2) + 0.05;
    let expected_max_f32 = ((0.0_f32 + 0.1) + 0.2) + 0.3;
    assert_eq!(min_f32.to_bits(), expected_min_f32.to_bits());
    assert_eq!(max_f32.to_bits(), expected_max_f32.to_bits());

    let (min_f64, max_f64) = mr02_intrinsic_extents::<f64>();
    let expected_min_f64 = ((0.0_f64 + 0.1) + 0.2) + 0.05;
    let expected_max_f64 = ((0.0_f64 + 0.1) + 0.2) + 0.3;
    assert_eq!(min_f64.to_bits(), expected_min_f64.to_bits());
    assert_eq!(max_f64.to_bits(), expected_max_f64.to_bits());
}

#[test]
fn fri06_mr02_inline_linear_float_band_retry_preserves_queries_and_reselection_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let mut queries = Vec::new();
        let report = layout_mixed_inline_run_with_band_source(
            mr02_input(
                AvailableOf::definite(S::from_f64(100.0)),
                vec![
                    mr02_atomic(60, 30.0, 10.0, 8.0, 0, InlineBreakOpportunityOf::allowed()),
                    mr02_atomic(61, 30.0, 10.0, 8.0, 0, InlineBreakOpportunityOf::allowed()),
                    mr02_atomic(
                        62,
                        30.0,
                        10.0,
                        8.0,
                        0,
                        InlineBreakOpportunityOf::prohibited(),
                    ),
                ],
            ),
            |block_start, block_end| {
                queries.push((block_start, block_end));
                if block_start == S::ZERO {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::from_f64(50.0),
                        inline_end: S::from_f64(50.0),
                        next_transition: Some(S::from_f64(10.0)),
                    }
                } else if block_start == S::from_f64(10.0) {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::from_f64(20.0),
                        inline_end: S::from_f64(60.0),
                        next_transition: None,
                    }
                } else {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::ZERO,
                        inline_end: S::from_f64(100.0),
                        next_transition: None,
                    }
                }
            },
            |block, _| block,
        );

        assert_eq!(
            queries,
            [
                (S::ZERO, S::from_f64(10.0)),
                (S::from_f64(10.0), S::from_f64(20.0)),
                (S::from_f64(20.0), S::from_f64(30.0)),
            ]
        );
        assert_eq!(report.line_bands.len(), 2);
        assert_eq!(report.atomics[0].item.source_index, 60);
        assert_eq!(report.atomics[0].inline_start, S::from_f64(20.0));
        assert_eq!(report.atomics[0].block_start, S::from_f64(10.0));
        assert_eq!(report.atomics[1].line_index, 1);
        assert_eq!(report.atomics[2].line_index, 1);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn mr02_candidate_scan_work(participant_count: usize) -> usize {
    let participants = (0..participant_count)
        .map(|source_index| {
            mr02_atomic::<f64>(
                source_index,
                1.0,
                10.0,
                8.0,
                0,
                InlineBreakOpportunityOf::prohibited(),
            )
        })
        .collect();
    reset_inline_candidate_scan_visits();
    let report =
        layout_mixed_inline_run(mr02_input(AvailableOf::definite(1_000_000.0), participants));
    assert_eq!(report.atomics.len(), participant_count);
    inline_candidate_scan_visits()
}

#[test]
fn fri06_mr02_inline_linear_candidate_scan_work_is_bounded_without_timing() {
    let work_64 = mr02_candidate_scan_work(64);
    let work_128 = mr02_candidate_scan_work(128);

    eprintln!("linear candidate scan work: n=64 => {work_64}, n=128 => {work_128}");
    assert_eq!(work_64, 64 * 2);
    assert_eq!(work_128, 128 * 2);
    assert_eq!(work_128, work_64 * 2);
}

fn forced_line_break(source_index: usize, metrics: InlineMetrics) -> InlineParticipant {
    forced_line_break_for(
        source_index,
        WritingMode::HorizontalTb,
        Direction::Ltr,
        metrics,
    )
}

fn forced_line_break_for(
    source_index: usize,
    writing_mode: WritingMode,
    direction: Direction,
    metrics: InlineMetrics,
) -> InlineParticipant {
    InlineParticipant::forced_line_break(crate::inline::ForcedLineBreakControlOf::new(
        source_index,
        crate::inline::InlineFlowOf::new(writing_mode, direction, Available::MAX_CONTENT),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        Clear::None,
    ))
}

fn inline_boundary_control(
    source_index: usize,
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    metrics: InlineMetrics,
    alignment: crate::inline::InlineControlAlignment,
) -> crate::inline::InlineBoundaryControlOf {
    crate::inline::InlineBoundaryControlOf::new(
        source_index,
        kind,
        crate::inline::InlineFlowOf::new(writing_mode, direction, Available::MAX_CONTENT),
        metrics,
        alignment,
    )
}

fn inline_boundary_participant(
    source_index: usize,
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    metrics: InlineMetrics,
) -> InlineParticipant {
    InlineParticipant::inline_boundary(inline_boundary_control(
        source_index,
        kind,
        writing_mode,
        direction,
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
    ))
}

fn mixed_forced_line_break<S: LayoutScalar>(
    source_index: usize,
    flow_axes: FlowAxes,
    metrics: InlineMetricsOf<S>,
    clear: Clear,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::ForcedLineBreak(crate::inline::ForcedLineBreakControlOf::new(
        source_index,
        crate::inline::InlineFlowOf::new(
            flow_axes.writing_mode(),
            flow_axes.direction(),
            AvailableOf::MAX_CONTENT,
        ),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        clear,
    ))
}

fn mixed_boundary<S: LayoutScalar>(
    source_index: usize,
    flow_axes: FlowAxes,
    metrics: InlineMetricsOf<S>,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::Boundary(crate::inline::InlineBoundaryControlOf::new(
        source_index,
        InlineBoundaryKind::Start,
        crate::inline::InlineFlowOf::new(
            flow_axes.writing_mode(),
            flow_axes.direction(),
            AvailableOf::MAX_CONTENT,
        ),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
    ))
}

#[test]
fn fri06_c03_clear_private_builder_maps_none_start_end_and_both_in_all_flows() {
    let writing_modes = [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ];
    for writing_mode in writing_modes {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            assert_eq!(
                mapped_post_line_clear_intent(flow_axes, Clear::None),
                PostLineClearIntent::None
            );
            assert_eq!(
                mapped_post_line_clear_intent(flow_axes, Clear::Left),
                PostLineClearIntent::LineStart
            );
            assert_eq!(
                mapped_post_line_clear_intent(flow_axes, Clear::Right),
                PostLineClearIntent::LineEnd
            );
            assert_eq!(
                mapped_post_line_clear_intent(flow_axes, Clear::Both),
                PostLineClearIntent::Both
            );
        }
    }
}

#[test]
fn fri06_c03_strut_private_builder_retains_control_baselines_and_post_line_intent_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let boundary_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(18.0), S::from_f64(13.0))
                .unwrap();
        let break_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();
        let report = layout_mixed_inline_run(MixedInlineRunInputOf {
            available_inline_extent: AvailableOf::MAX_CONTENT,
            flow_axes,
            text_align: TextAlign::Auto,
            participants: vec![
                mixed_boundary(0, flow_axes, boundary_metrics),
                mixed_forced_line_break(1, flow_axes, break_metrics, Clear::Both),
            ],
        });

        assert_eq!(report.block_extent, S::from_f64(40.0));
        assert_eq!(report.first_baseline, Some(S::from_f64(15.0)));
        assert_eq!(report.last_baseline, Some(S::from_f64(35.0)));
        assert_eq!(
            report.post_line_clear_intents,
            [PostLineClearIntent::Both, PostLineClearIntent::None]
        );
        assert_eq!(report.controls[0].visual_index, Some(0));
        assert_eq!(report.controls[1].visual_index, None);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_private_callback_retries_transition_and_reselects_same_cursor_both_scalars()
{
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let participant = |source_index, following_break| MixedInlineParticipantOf::Atomic {
            item: AtomicInlineBoxParticipant {
                source_index,
                size: Size::new(S::from_f64(30.0), S::from_f64(10.0)),
                content_size: Size::new(S::from_f64(30.0), S::from_f64(10.0)),
                margin: Edges::ZERO,
                padding: Edges::ZERO,
                border: Edges::ZERO,
                scrollbar_size: Size::ZERO,
                first_baseline: Some(S::from_f64(8.0)),
                alignment: InlineControlAlignment::Baseline,
            },
            participation: AtomicInlineParticipationOf::try_new(
                BidiLevel::try_new(0).unwrap(),
                following_break,
            )
            .unwrap(),
        };
        let mut queries = Vec::new();
        let report = layout_mixed_inline_run_with_band_source(
            MixedInlineRunInputOf {
                available_inline_extent: AvailableOf::definite(S::from_f64(100.0)),
                flow_axes,
                text_align: TextAlign::Auto,
                participants: vec![
                    participant(0, InlineBreakOpportunityOf::allowed()),
                    participant(1, InlineBreakOpportunityOf::allowed()),
                    participant(2, InlineBreakOpportunityOf::prohibited()),
                ],
            },
            |block_start, block_end| {
                queries.push((block_start, block_end));
                if block_start == S::ZERO {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::from_f64(50.0),
                        inline_end: S::from_f64(50.0),
                        next_transition: Some(S::from_f64(10.0)),
                    }
                } else if block_start == S::from_f64(10.0) {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::from_f64(20.0),
                        inline_end: S::from_f64(60.0),
                        next_transition: None,
                    }
                } else {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::ZERO,
                        inline_end: S::from_f64(100.0),
                        next_transition: None,
                    }
                }
            },
            |block, _| block,
        );

        assert_eq!(
            queries,
            [
                (S::ZERO, S::from_f64(10.0)),
                (S::from_f64(10.0), S::from_f64(20.0)),
                (S::from_f64(20.0), S::from_f64(30.0)),
            ]
        );
        assert_eq!(report.atomics.len(), 3);
        assert_eq!(report.atomics[0].item.source_index, 0);
        assert_eq!(report.atomics[0].line_index, 0);
        assert_eq!(report.atomics[0].inline_start, S::from_f64(20.0));
        assert_eq!(report.atomics[0].block_start, S::from_f64(10.0));
        assert_eq!(report.atomics[1].item.source_index, 1);
        assert_eq!(report.atomics[1].line_index, 1);
        assert_eq!(report.atomics[2].item.source_index, 2);
        assert_eq!(report.atomics[2].line_index, 1);
        assert_eq!(report.line_bands.len(), 2);
        assert_eq!(report.line_bands[0].block_start, S::from_f64(10.0));
        assert_eq!(report.line_bands[0].inline_start, S::from_f64(20.0));
        assert_eq!(report.line_bands[0].inline_end, S::from_f64(60.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn inline_axis_mapping_maps_horizontal_tb_ltr() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);

    assert_eq!(
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(30.0, 12.0)),
        Size::new(30.0, 12.0)
    );
    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(30.0, 80.0),
        ),
        Point::new(5.0, 7.0)
    );
}

#[test]
fn inline_axis_mapping_maps_horizontal_tb_rtl() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);

    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(30.0, 80.0),
        ),
        Point::new(15.0, 7.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_rl_ltr() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);

    assert_eq!(
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(30.0, 12.0)),
        Size::new(12.0, 30.0)
    );
    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(80.0, 30.0),
        ),
        Point::new(69.0, 5.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_rl_rtl() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);

    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(80.0, 30.0),
        ),
        Point::new(69.0, 15.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_lr_ltr() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);

    assert_eq!(
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(30.0, 12.0)),
        Size::new(12.0, 30.0)
    );
    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(80.0, 30.0),
        ),
        Point::new(7.0, 5.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_lr_rtl() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl);

    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(80.0, 30.0),
        ),
        Point::new(7.0, 15.0)
    );
}

#[test]
fn atomic_inline_line_aligns_items_to_max_baseline() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::definite(200.0),
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(7.0)),
            InlineParticipant::new(1, Size::new(10.0, 20.0), Edges::ZERO, Some(12.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.first_baseline, Some(12.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 5.0));
    assert_eq!(report.items[1].location, Point::new(20.0, 0.0));
}

#[test]
fn atomic_inline_items_wrap_between_items_for_definite_width() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::definite(25.0),
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            InlineParticipant::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 20.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(20.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
}

#[test]
fn atomic_inline_min_content_available_wraps_to_max_item_advance() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MIN_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(40.0, 10.0), Edges::ZERO, Some(10.0)),
            InlineParticipant::new(1, Size::new(60.0, 10.0), Edges::ZERO, Some(10.0)),
            InlineParticipant::new(2, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
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
        InlineParticipant::new(
            0,
            Size::new(25.0, 10.0),
            Edges::new(0.0, 5.0, 0.0, 5.0),
            Some(10.0),
        ),
        InlineParticipant::new(
            1,
            Size::new(100.0, 10.0),
            Edges::new(0.0, 0.0, 0.0, 10.0),
            Some(10.0),
        ),
        InlineParticipant::new(2, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
    ];

    assert_eq!(inline_run_min_content_width(&items), 110.0);
    assert_eq!(inline_run_max_content_width(&items), 195.0);
}

#[test]
fn atomic_inline_horizontal_rtl_maps_item_origins_in_report() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Rtl,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            forced_line_break(1, metrics),
            InlineParticipant::new(2, Size::new(30.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.items[0].location, Point::new(10.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(10.0, 10.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 10.0));
}

#[test]
fn forced_line_break_control_preserves_layout_ready_fields() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let control = crate::inline::ForcedLineBreakControlOf::new(
        7,
        crate::inline::InlineFlowOf::new(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            Available::definite(320.0),
        ),
        metrics,
        crate::inline::InlineControlAlignment::Top,
        Clear::Both,
    );

    assert_eq!(control.source_index(), 7);
    assert_eq!(control.flow().writing_mode(), WritingMode::HorizontalTb);
    assert_eq!(control.flow().direction(), Direction::Rtl);
    assert_eq!(
        control.flow().available_inline_extent(),
        Available::definite(320.0)
    );
    assert_eq!(control.metrics(), metrics);
    assert_eq!(
        control.alignment(),
        crate::inline::InlineControlAlignment::Top
    );
    assert_eq!(control.clear(), Clear::Both);
}

#[test]
fn inline_boundary_control_preserves_layout_ready_fields() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let control = inline_boundary_control(
        9,
        InlineBoundaryKind::Start,
        WritingMode::VerticalRl,
        Direction::Rtl,
        metrics,
        crate::inline::InlineControlAlignment::Top,
    );

    assert_eq!(control.source_index(), 9);
    assert_eq!(control.kind(), InlineBoundaryKind::Start);
    assert_eq!(control.flow().writing_mode(), WritingMode::VerticalRl);
    assert_eq!(control.flow().direction(), Direction::Rtl);
    assert_eq!(
        control.flow().available_inline_extent(),
        Available::MAX_CONTENT
    );
    assert_eq!(control.metrics(), metrics);
    assert_eq!(
        control.alignment(),
        crate::inline::InlineControlAlignment::Top
    );
    assert_eq!(
        InlineParticipant::inline_boundary(control),
        InlineParticipant::Boundary(control)
    );
}

#[test]
fn inline_boundaries_expand_horizontal_line_metrics_without_advance() {
    let start_metrics = InlineMetrics::from_line_height_and_baseline(10.0, 8.0).unwrap();
    let end_metrics = InlineMetrics::from_line_height_and_baseline(30.0, 20.0).unwrap();
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            inline_boundary_participant(
                0,
                InlineBoundaryKind::Start,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                start_metrics,
            ),
            InlineParticipant::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(8.0)),
            inline_boundary_participant(
                2,
                InlineBoundaryKind::End,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                end_metrics,
            ),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 30.0));
    assert_eq!(report.first_baseline, Some(20.0));
    assert_eq!(report.last_baseline, Some(20.0));
    assert_eq!(
        report.items[0].kind,
        InlineParticipantLayoutKind::InlineBoundaryStart
    );
    assert_eq!(report.items[0].location, Point::new(0.0, 20.0));
    assert_eq!(report.items[0].size, Size::ZERO);
    assert_eq!(report.items[1].kind, InlineParticipantLayoutKind::Box);
    assert_eq!(report.items[1].location, Point::new(0.0, 12.0));
    assert_eq!(
        report.items[2].kind,
        InlineParticipantLayoutKind::InlineBoundaryEnd
    );
    assert_eq!(report.items[2].location, Point::new(20.0, 20.0));
    assert_eq!(report.items[2].size, Size::ZERO);
}

#[test]
fn inline_boundaries_before_overwide_first_box_do_not_create_leading_line() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(50.0, 35.0).unwrap();
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::definite(20.0),
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            inline_boundary_participant(
                0,
                InlineBoundaryKind::Start,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                boundary_metrics,
            ),
            InlineParticipant::new(1, Size::new(40.0, 10.0), Edges::ZERO, Some(8.0)),
        ],
    });

    assert_eq!(report.size, Size::new(40.0, 50.0));
    assert_eq!(report.first_baseline, Some(35.0));
    assert_eq!(report.last_baseline, Some(35.0));
    assert_eq!(
        report.items[0].kind,
        InlineParticipantLayoutKind::InlineBoundaryStart
    );
    assert_eq!(report.items[0].location, Point::new(0.0, 35.0));
    assert_eq!(report.items[1].kind, InlineParticipantLayoutKind::Box);
    assert_eq!(report.items[1].location, Point::new(0.0, 27.0));
}

#[test]
fn forced_line_break_control_can_be_used_as_atomic_inline_item() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 8.0).unwrap();
    let control = crate::inline::ForcedLineBreakControlOf::new(
        1,
        crate::inline::InlineFlowOf::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            Available::MAX_CONTENT,
        ),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        Clear::None,
    );

    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            InlineParticipant::forced_line_break(control),
            InlineParticipant::new(2, Size::new(15.0, 12.0), Edges::ZERO, Some(8.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 24.0));
    assert_eq!(
        report.items[1].kind,
        InlineParticipantLayoutKind::ForcedLineBreak
    );
    assert_eq!(report.items[1].source_index, 1);
    assert_eq!(report.items[1].location, Point::new(20.0, 10.0));
    assert_eq!(report.items[1].size, Size::ZERO);
}

#[test]
fn atomic_inline_intrinsic_widths_split_at_forced_line_breaks() {
    let items = vec![
        InlineParticipant::new(
            0,
            Size::new(25.0, 10.0),
            Edges::new(0.0, 5.0, 0.0, 5.0),
            Some(10.0),
        ),
        forced_line_break(1, InlineMetrics::default()),
        InlineParticipant::new(
            2,
            Size::new(100.0, 10.0),
            Edges::new(0.0, 0.0, 0.0, 10.0),
            Some(10.0),
        ),
        InlineParticipant::new(3, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
        forced_line_break(4, InlineMetrics::default()),
    ];

    assert_eq!(inline_run_min_content_width(&items), 110.0);
    assert_eq!(inline_run_max_content_width(&items), 160.0);
}

#[test]
fn atomic_inline_vertical_margins_participate_in_line_metrics() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![InlineParticipant::new(
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
fn atomic_inline_sideways_lr_ltr_maps_inline_progression_bottom_to_top() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::definite(70.0),
        writing_mode: WritingMode::SidewaysLr,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
            InlineParticipant::new(1, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
        ],
    });

    assert_eq!(report.items[0].location, Point::new(0.0, 20.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 0.0));
}

#[test]
fn atomic_inline_empty_items_report_zero_size_and_no_baselines() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
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
        AtomicInlineParticipation, Available, BidiLevel, Display, InlineBreakOpportunity,
        NodeInput, PreferredSize, Size, TrackComponent, compute_root, round_layout,
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
                size: Size::new(PreferredSize::px(50.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        );

        for (node, (_, size)) in (1_u32..).zip(item_sizes) {
            tree = tree.style(node, atomic_inline_node_input(display, size));
        }

        compute_root(&mut tree, 0, Size::splat(Available::definite(50.0))).unwrap();
        round_layout(&mut tree, 0).unwrap();

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
        let atomic_inline_participation = Some(
            AtomicInlineParticipation::try_new(
                BidiLevel::try_new(0).unwrap(),
                InlineBreakOpportunity::allowed(),
            )
            .unwrap(),
        );
        match display.inner_display() {
            Display::Grid => NodeInput {
                display,
                grid_template_columns: vec![TrackComponent::px(size.width)],
                grid_template_rows: vec![TrackComponent::px(size.height)],
                atomic_inline_participation,
                ..NodeInput::DEFAULT
            },
            Display::GridLanes => NodeInput {
                display,
                grid_template_columns: vec![TrackComponent::px(size.width)],
                grid_template_rows: vec![TrackComponent::px(size.height)],
                atomic_inline_participation,
                ..NodeInput::DEFAULT
            },
            _ => NodeInput {
                display,
                size: Size::new(
                    PreferredSize::px(size.width),
                    PreferredSize::px(size.height),
                ),
                atomic_inline_participation,
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
