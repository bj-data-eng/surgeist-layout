use super::fixtures::lp;

mod root_oracle {
    use crate::test_support::{
        layout_tree::{OracleMeasurement, OracleTree},
        oracle::grid::{
            self, AlignmentSafety, AutoPlacer, ContributionSize, DefiniteTracks,
            EqualShareIntrinsicTracks, Flow, GridArea, GridAxis, GridItemRect, GridTrack,
            GrowthLimit, ItemContributionFacts, ItemContributions, ItemPlacement, LineNameOrigin,
            LinePlacement, NamedGridError, NamedGridLines, NamedLineOccurrence, PlacementError,
            Track, TrackAlignment, TrackMax, TrackMin, TrackSize, TrackSizingError,
            TrackSizingSlice, align_tracks, align_tracks_report, compose_grid_scenario,
        },
    };
    use crate::{
        Available, ComputeInput, ComputeOutput, Display, Length, NodeInput, PreferredSize,
        RequestedAxis, RunMode, Size, SizingMode, TrackComponent,
    };

    fn oracle_lane_span(value: usize) -> grid::LaneTrackSpanLength {
        grid::LaneTrackSpanLength::new(value).expect("valid oracle lane span length")
    }

    fn participating_baseline_item() -> grid::BaselineItemFacts {
        grid::BaselineItemFacts {
            id: "item",
            area: grid::GridArea::new(1, 1, 1, 1),
            block_size: 30.0,
            margin_before: 3.0,
            margin_after: 5.0,
            first_baseline: Some(8.0),
            last_baseline: Some(24.0),
            synthesized_first: false,
            synthesized_last: false,
            alignment: grid::BaselineAlignment::First,
            out_of_flow: false,
            baseline_axis_auto_margins: false,
            spans_intrinsic_track: false,
            baseline_requires_unavailable_subgrid_layout: false,
        }
    }

    fn oracle_baseline_test_item(
        id: &'static str,
        alignment: grid::BaselineAlignment,
    ) -> grid::BaselineItemFacts {
        grid::BaselineItemFacts {
            id,
            area: grid::GridArea::new(1, 1, 1, 1),
            block_size: 20.0,
            margin_before: 0.0,
            margin_after: 0.0,
            first_baseline: Some(8.0),
            last_baseline: Some(16.0),
            synthesized_first: false,
            synthesized_last: false,
            alignment,
            out_of_flow: false,
            baseline_axis_auto_margins: false,
            spans_intrinsic_track: false,
            baseline_requires_unavailable_subgrid_layout: false,
        }
    }

    #[derive(Clone, Copy)]
    struct OracleBaselineItemCase {
        id: &'static str,
        row_start: usize,
        row_span: usize,
        alignment: grid::BaselineAlignment,
        block_size: f32,
        margin_before: f32,
        margin_after: f32,
        first_baseline: Option<f32>,
        last_baseline: Option<f32>,
    }

    fn oracle_baseline_item(case: OracleBaselineItemCase) -> grid::BaselineItemFacts {
        grid::BaselineItemFacts {
            id: case.id,
            area: grid::GridArea::new(1, case.row_start, 1, case.row_span),
            block_size: case.block_size,
            margin_before: case.margin_before,
            margin_after: case.margin_after,
            first_baseline: case.first_baseline,
            last_baseline: case.last_baseline,
            synthesized_first: case.first_baseline.is_none(),
            synthesized_last: case.last_baseline.is_none(),
            alignment: case.alignment,
            out_of_flow: false,
            baseline_axis_auto_margins: false,
            spans_intrinsic_track: false,
            baseline_requires_unavailable_subgrid_layout: false,
        }
    }

    #[test]
    fn fri08_c08_t06_suppression_cleanup_baseline_case_preserves_coupled_inputs_and_defaults() {
        let item = oracle_baseline_item(OracleBaselineItemCase {
            id: "characterized",
            row_start: 2,
            row_span: 3,
            alignment: grid::BaselineAlignment::Last,
            block_size: 37.0,
            margin_before: 4.0,
            margin_after: 6.0,
            first_baseline: None,
            last_baseline: Some(29.0),
        });

        assert_eq!(item.id, "characterized");
        assert_eq!(item.area, grid::GridArea::new(1, 2, 1, 3));
        assert_eq!(item.alignment, grid::BaselineAlignment::Last);
        assert_eq!(item.block_size, 37.0);
        assert_eq!(item.margin_before, 4.0);
        assert_eq!(item.margin_after, 6.0);
        assert_eq!(item.first_baseline, None);
        assert_eq!(item.last_baseline, Some(29.0));
        assert!(item.synthesized_first);
        assert!(!item.synthesized_last);
        assert!(!item.out_of_flow);
        assert!(!item.baseline_axis_auto_margins);
        assert!(!item.spans_intrinsic_track);
        assert!(!item.baseline_requires_unavailable_subgrid_layout);
    }

    #[test]
    fn oracle_baseline_geometry_uses_margin_box_contributions() {
        let geometry =
            grid::BaselineGeometry::from_item(participating_baseline_item(), 40.0).unwrap();

        assert_eq!(geometry.margin_box_size, 38.0);
        assert_eq!(geometry.major_baseline, 11.0);
        assert_eq!(geometry.minor_baseline, 11.0);
    }

    #[test]
    fn oracle_baseline_geometry_rejects_non_participating_facts() {
        let unsupported = grid::OracleGridError::BaselineInferenceUnsupported;
        let cases = [
            grid::BaselineItemFacts {
                alignment: grid::BaselineAlignment::None,
                ..participating_baseline_item()
            },
            grid::BaselineItemFacts {
                out_of_flow: true,
                ..participating_baseline_item()
            },
            grid::BaselineItemFacts {
                baseline_axis_auto_margins: true,
                ..participating_baseline_item()
            },
            grid::BaselineItemFacts {
                synthesized_first: true,
                first_baseline: None,
                spans_intrinsic_track: true,
                ..participating_baseline_item()
            },
            grid::BaselineItemFacts {
                alignment: grid::BaselineAlignment::Last,
                synthesized_last: true,
                last_baseline: None,
                baseline_requires_unavailable_subgrid_layout: true,
                ..participating_baseline_item()
            },
        ];

        for item in cases {
            assert_eq!(
                grid::BaselineGeometry::from_item(item, 40.0),
                Err(unsupported)
            );
        }
    }

    #[test]
    fn oracle_baseline_offset_uses_whole_spanned_area_for_major_group() {
        let offset = grid::baseline_offset(
            grid::BaselineGroupKind::Major,
            20.0,
            grid::BaselineGeometry {
                available_span_size: 75.0,
                margin_box_size: 38.0,
                major_baseline: 11.0,
                minor_baseline: 11.0,
            },
        );

        assert_eq!(offset, 9.0);
    }

    #[test]
    fn oracle_baseline_offset_uses_whole_spanned_area_for_minor_group() {
        let offset = grid::baseline_offset(
            grid::BaselineGroupKind::Minor,
            12.0,
            grid::BaselineGeometry {
                available_span_size: 75.0,
                margin_box_size: 38.0,
                major_baseline: 11.0,
                minor_baseline: 9.0,
            },
        );

        assert_eq!(offset, 34.0);
    }

    #[test]
    fn oracle_baseline_shim_grows_before_for_major_group() {
        let shim = grid::baseline_intrinsic_shim(
            grid::BaselineGroupKind::Major,
            20.0,
            grid::BaselineGeometry {
                available_span_size: 75.0,
                margin_box_size: 38.0,
                major_baseline: 11.0,
                minor_baseline: 11.0,
            },
        );

        assert_eq!(
            shim,
            grid::BaselineShim {
                before: 9.0,
                after: 0.0,
            }
        );
    }

    #[test]
    fn oracle_baseline_shim_grows_after_for_minor_group() {
        let shim = grid::baseline_intrinsic_shim(
            grid::BaselineGroupKind::Minor,
            14.0,
            grid::BaselineGeometry {
                available_span_size: 75.0,
                margin_box_size: 38.0,
                major_baseline: 11.0,
                minor_baseline: 9.0,
            },
        );

        assert_eq!(
            shim,
            grid::BaselineShim {
                before: 0.0,
                after: 5.0,
            }
        );
    }

    #[test]
    fn oracle_baseline_shim_clamps_negative_major_growth_to_zero() {
        let shim = grid::baseline_intrinsic_shim(
            grid::BaselineGroupKind::Major,
            8.0,
            grid::BaselineGeometry {
                available_span_size: 75.0,
                margin_box_size: 38.0,
                major_baseline: 11.0,
                minor_baseline: 9.0,
            },
        );

        assert_eq!(shim, grid::BaselineShim::default());
    }

    #[test]
    fn oracle_baseline_shim_clamps_negative_minor_growth_to_zero() {
        let shim = grid::baseline_intrinsic_shim(
            grid::BaselineGroupKind::Minor,
            7.0,
            grid::BaselineGeometry {
                available_span_size: 75.0,
                margin_box_size: 38.0,
                major_baseline: 11.0,
                minor_baseline: 9.0,
            },
        );

        assert_eq!(shim, grid::BaselineShim::default());
    }

    #[test]
    fn oracle_baseline_participation_rejects_out_of_flow_items() {
        let mut item = oracle_baseline_test_item("abspos", grid::BaselineAlignment::First);
        item.out_of_flow = true;
        let report = grid::baseline_participation(item);

        assert!(!report.participates);
        assert_eq!(report.fallback, Some(grid::BaselineFallback::Start));
    }

    #[test]
    fn oracle_baseline_participation_rejects_auto_margins() {
        let mut item = oracle_baseline_test_item("auto-margin", grid::BaselineAlignment::Last);
        item.baseline_axis_auto_margins = true;
        let report = grid::baseline_participation(item);

        assert!(!report.participates);
        assert_eq!(report.fallback, Some(grid::BaselineFallback::End));
    }

    #[test]
    fn oracle_baseline_participation_falls_back_for_synthesized_intrinsic_cycles() {
        let mut item = oracle_baseline_test_item("synth", grid::BaselineAlignment::First);
        item.first_baseline = None;
        item.synthesized_first = true;
        item.spans_intrinsic_track = true;
        let report = grid::baseline_participation(item);

        assert!(!report.participates);
        assert_eq!(report.fallback, Some(grid::BaselineFallback::Start));
    }

    #[test]
    fn oracle_baseline_participation_falls_back_for_unavailable_subgrid_layout() {
        let mut item = oracle_baseline_test_item("subgrid-synth", grid::BaselineAlignment::First);
        item.first_baseline = None;
        item.synthesized_first = true;
        item.baseline_requires_unavailable_subgrid_layout = true;
        let report = grid::baseline_participation(item);

        assert!(!report.participates);
        assert_eq!(report.fallback, Some(grid::BaselineFallback::Start));
    }

    #[test]
    fn oracle_baseline_participation_none_alignment_does_not_panic() {
        let item = oracle_baseline_test_item("none", grid::BaselineAlignment::None);
        let report = grid::baseline_participation(item);

        assert!(!report.participates);
        assert_eq!(report.group, None);
        assert_eq!(report.fallback, None);
        assert!(!report.used_synthesized_baseline);
    }

    #[test]
    fn oracle_baseline_predicates_ignore_unaligned_synthesized_cycle() {
        let mut item = oracle_baseline_test_item("first-explicit", grid::BaselineAlignment::First);
        item.synthesized_last = true;
        item.spans_intrinsic_track = true;

        let report = grid::baseline_participation(item);
        assert!(report.participates);
        assert_eq!(report.fallback, None);
        assert!(!report.used_synthesized_baseline);
        assert!(grid::BaselineGeometry::from_item(item, 40.0).is_ok());
    }

    #[test]
    fn oracle_baseline_predicates_reject_missing_aligned_first_intrinsic_cycle() {
        let mut item = oracle_baseline_test_item("first-missing", grid::BaselineAlignment::First);
        item.first_baseline = None;
        item.spans_intrinsic_track = true;

        let report = grid::baseline_participation(item);
        assert!(!report.participates);
        assert_eq!(report.fallback, Some(grid::BaselineFallback::Start));
        assert_eq!(
            grid::BaselineGeometry::from_item(item, 40.0),
            Err(grid::OracleGridError::BaselineInferenceUnsupported)
        );
    }

    #[test]
    fn oracle_baseline_predicates_reject_missing_aligned_last_intrinsic_cycle() {
        let mut item = oracle_baseline_test_item("last-missing", grid::BaselineAlignment::Last);
        item.last_baseline = None;
        item.spans_intrinsic_track = true;

        let report = grid::baseline_participation(item);
        assert!(!report.participates);
        assert_eq!(report.fallback, Some(grid::BaselineFallback::End));
        assert_eq!(
            grid::BaselineGeometry::from_item(item, 40.0),
            Err(grid::OracleGridError::BaselineInferenceUnsupported)
        );
    }

    #[test]
    fn oracle_baseline_groups_collect_major_group_on_start_track() {
        let report = grid::baseline_groups(grid::BaselineGroupInput {
            track_count: 3,
            track_sizes: vec![30.0, 40.0, 50.0],
            gap: 5.0,
            items: vec![
                oracle_baseline_item(OracleBaselineItemCase {
                    id: "a",
                    row_start: 1,
                    row_span: 1,
                    alignment: grid::BaselineAlignment::First,
                    block_size: 20.0,
                    margin_before: 3.0,
                    margin_after: 2.0,
                    first_baseline: Some(8.0),
                    last_baseline: Some(16.0),
                }),
                oracle_baseline_item(OracleBaselineItemCase {
                    id: "b",
                    row_start: 1,
                    row_span: 1,
                    alignment: grid::BaselineAlignment::First,
                    block_size: 24.0,
                    margin_before: 1.0,
                    margin_after: 1.0,
                    first_baseline: Some(12.0),
                    last_baseline: Some(18.0),
                }),
            ],
        })
        .unwrap();

        assert_eq!(report.major[0], Some(13.0));
        assert_eq!(report.minor, vec![None, None, None]);
    }

    #[test]
    fn oracle_baseline_groups_collect_minor_group_on_end_track_for_spanning_item() {
        let report = grid::baseline_groups(grid::BaselineGroupInput {
            track_count: 3,
            track_sizes: vec![30.0, 40.0, 50.0],
            gap: 5.0,
            items: vec![oracle_baseline_item(OracleBaselineItemCase {
                id: "span",
                row_start: 1,
                row_span: 2,
                alignment: grid::BaselineAlignment::Last,
                block_size: 30.0,
                margin_before: 2.0,
                margin_after: 4.0,
                first_baseline: Some(8.0),
                last_baseline: Some(22.0),
            })],
        })
        .unwrap();

        assert_eq!(report.minor[1], Some(12.0));
    }

    #[test]
    fn oracle_baseline_groups_preserve_nonparticipants_without_updating_group() {
        let mut nonparticipant = oracle_baseline_item(OracleBaselineItemCase {
            id: "absolute",
            row_start: 1,
            row_span: 1,
            alignment: grid::BaselineAlignment::First,
            block_size: 80.0,
            margin_before: 20.0,
            margin_after: 0.0,
            first_baseline: Some(40.0),
            last_baseline: Some(60.0),
        });
        nonparticipant.out_of_flow = true;
        let mut empty_row_nonparticipant = oracle_baseline_item(OracleBaselineItemCase {
            id: "empty-row-absolute",
            row_start: 2,
            row_span: 1,
            alignment: grid::BaselineAlignment::First,
            block_size: 80.0,
            margin_before: 20.0,
            margin_after: 0.0,
            first_baseline: Some(40.0),
            last_baseline: Some(60.0),
        });
        empty_row_nonparticipant.out_of_flow = true;

        let report = grid::baseline_groups(grid::BaselineGroupInput {
            track_count: 2,
            track_sizes: vec![30.0, 40.0],
            gap: 5.0,
            items: vec![
                oracle_baseline_item(OracleBaselineItemCase {
                    id: "participant",
                    row_start: 1,
                    row_span: 1,
                    alignment: grid::BaselineAlignment::First,
                    block_size: 20.0,
                    margin_before: 1.0,
                    margin_after: 0.0,
                    first_baseline: Some(6.0),
                    last_baseline: Some(14.0),
                }),
                nonparticipant,
                empty_row_nonparticipant,
            ],
        })
        .unwrap();

        assert_eq!(report.participation.len(), 3);
        assert_eq!(report.participation[0].id, "participant");
        assert_eq!(report.participation[1].id, "absolute");
        assert_eq!(report.participation[2].id, "empty-row-absolute");
        assert!(!report.participation[1].participates);
        assert!(!report.participation[2].participates);
        assert_eq!(report.major[0], Some(7.0));
        assert_eq!(report.major[1], None);
    }

    #[test]
    fn oracle_baseline_groups_reject_invalid_track_and_row_spans() {
        let valid_item = oracle_baseline_item(OracleBaselineItemCase {
            id: "item",
            row_start: 1,
            row_span: 1,
            alignment: grid::BaselineAlignment::First,
            block_size: 20.0,
            margin_before: 0.0,
            margin_after: 0.0,
            first_baseline: Some(6.0),
            last_baseline: Some(14.0),
        });
        let invalid_start = grid::BaselineItemFacts {
            area: grid::GridArea::new(1, 0, 1, 1),
            ..valid_item
        };
        let invalid_span = grid::BaselineItemFacts {
            area: grid::GridArea::new(1, 1, 1, 0),
            ..valid_item
        };
        let beyond_tracks = grid::BaselineItemFacts {
            area: grid::GridArea::new(1, 2, 1, 2),
            ..valid_item
        };

        let cases = [
            grid::BaselineGroupInput {
                track_count: 0,
                track_sizes: vec![],
                gap: 0.0,
                items: vec![],
            },
            grid::BaselineGroupInput {
                track_count: 2,
                track_sizes: vec![30.0],
                gap: 0.0,
                items: vec![valid_item],
            },
            grid::BaselineGroupInput {
                track_count: 2,
                track_sizes: vec![30.0, 40.0],
                gap: 0.0,
                items: vec![invalid_start],
            },
            grid::BaselineGroupInput {
                track_count: 2,
                track_sizes: vec![30.0, 40.0],
                gap: 0.0,
                items: vec![invalid_span],
            },
            grid::BaselineGroupInput {
                track_count: 2,
                track_sizes: vec![30.0, 40.0],
                gap: 0.0,
                items: vec![beyond_tracks],
            },
        ];

        for input in cases {
            assert!(grid::baseline_groups(input).is_err());
        }
    }

    #[test]
    fn oracle_baseline_groups_collect_spanning_major_group_on_start_track() {
        let report = grid::baseline_groups(grid::BaselineGroupInput {
            track_count: 4,
            track_sizes: vec![20.0, 30.0, 40.0, 50.0],
            gap: 5.0,
            items: vec![oracle_baseline_item(OracleBaselineItemCase {
                id: "span-major",
                row_start: 2,
                row_span: 2,
                alignment: grid::BaselineAlignment::First,
                block_size: 30.0,
                margin_before: 2.0,
                margin_after: 3.0,
                first_baseline: Some(9.0),
                last_baseline: Some(21.0),
            })],
        })
        .unwrap();

        assert_eq!(report.major[1], Some(11.0));
        assert_eq!(report.major[2], None);
    }

    #[test]
    fn oracle_container_baselines_prefer_major_and_minor_groups() {
        let report = grid::container_baselines(grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0, 30.0],
            groups: grid::BaselineGroupReport {
                major: vec![Some(14.0), None],
                minor: vec![None, Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: vec![
                grid::ContainerBaselineFallbackItem {
                    id: "first",
                    area: grid::GridArea::new(1, 1, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 8.0,
                    last_baseline: 20.0,
                },
                grid::ContainerBaselineFallbackItem {
                    id: "last",
                    area: grid::GridArea::new(2, 1, 1, 1),
                    block_offset: 40.0,
                    first_baseline: 10.0,
                    last_baseline: 24.0,
                },
            ],
        })
        .unwrap();

        assert_eq!(report.first, Some(14.0));
        assert_eq!(report.last, Some(64.0));
    }

    #[test]
    fn oracle_container_baselines_use_minor_group_for_first_when_major_missing() {
        let report = grid::container_baselines(grid::ContainerBaselineInput {
            track_offsets: vec![0.0],
            track_sizes: vec![30.0],
            groups: grid::BaselineGroupReport {
                major: vec![None],
                minor: vec![Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        })
        .unwrap();

        assert_eq!(report.first, Some(24.0));
        assert_eq!(report.last, Some(24.0));
    }

    #[test]
    fn oracle_container_baselines_use_major_group_for_last_when_minor_missing() {
        let report = grid::container_baselines(grid::ContainerBaselineInput {
            track_offsets: vec![40.0],
            track_sizes: vec![30.0],
            groups: grid::BaselineGroupReport {
                major: vec![Some(12.0)],
                minor: vec![None],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        })
        .unwrap();

        assert_eq!(report.first, Some(52.0));
        assert_eq!(report.last, Some(52.0));
    }

    #[test]
    fn oracle_container_baselines_fallback_by_grid_order_and_synthesis() {
        let report = grid::container_baselines(grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0, 30.0],
            groups: grid::BaselineGroupReport {
                major: vec![None, None],
                minor: vec![None, None],
                participation: Vec::new(),
            },
            fallback_items: vec![
                grid::ContainerBaselineFallbackItem {
                    id: "row-2-col-1",
                    area: grid::GridArea::new(1, 2, 1, 1),
                    block_offset: 40.0,
                    first_baseline: 70.0,
                    last_baseline: 40.0,
                },
                grid::ContainerBaselineFallbackItem {
                    id: "row-1-col-2-synth-first",
                    area: grid::GridArea::new(2, 1, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 30.0,
                    last_baseline: 6.0,
                },
                grid::ContainerBaselineFallbackItem {
                    id: "row-1-col-1",
                    area: grid::GridArea::new(1, 1, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 8.0,
                    last_baseline: 22.0,
                },
            ],
        })
        .unwrap();

        assert_eq!(report.first, Some(8.0));
        assert_eq!(report.last, Some(40.0));
    }

    #[test]
    fn oracle_container_baselines_last_fallback_uses_spanned_end_edge() {
        let report = grid::container_baselines(grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0, 80.0],
            track_sizes: vec![30.0, 30.0, 30.0],
            groups: grid::BaselineGroupReport {
                major: vec![None, None, None],
                minor: vec![None, None, None],
                participation: Vec::new(),
            },
            fallback_items: vec![
                grid::ContainerBaselineFallbackItem {
                    id: "starts-later",
                    area: grid::GridArea::new(1, 2, 1, 1),
                    block_offset: 40.0,
                    first_baseline: 11.0,
                    last_baseline: 55.0,
                },
                grid::ContainerBaselineFallbackItem {
                    id: "spans-to-last-row",
                    area: grid::GridArea::new(2, 1, 1, 3),
                    block_offset: 0.0,
                    first_baseline: 8.0,
                    last_baseline: 92.0,
                },
            ],
        })
        .unwrap();

        assert_eq!(report.first, Some(8.0));
        assert_eq!(report.last, Some(92.0));
    }

    #[test]
    fn oracle_container_baselines_return_none_for_empty_input() {
        let report = grid::container_baselines(grid::ContainerBaselineInput {
            track_offsets: Vec::new(),
            track_sizes: Vec::new(),
            groups: grid::BaselineGroupReport {
                major: Vec::new(),
                minor: Vec::new(),
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        })
        .unwrap();

        assert_eq!(report.first, None);
        assert_eq!(report.last, None);
    }

    #[test]
    fn oracle_container_baselines_reject_vector_shape_mismatches() {
        let cases = [
            grid::ContainerBaselineInput {
                track_offsets: vec![0.0, 40.0],
                track_sizes: vec![30.0],
                groups: grid::BaselineGroupReport {
                    major: vec![Some(14.0), None],
                    minor: vec![None, Some(6.0)],
                    participation: Vec::new(),
                },
                fallback_items: Vec::new(),
            },
            grid::ContainerBaselineInput {
                track_offsets: vec![0.0, 40.0],
                track_sizes: vec![30.0, 30.0],
                groups: grid::BaselineGroupReport {
                    major: vec![Some(14.0)],
                    minor: vec![None, Some(6.0)],
                    participation: Vec::new(),
                },
                fallback_items: Vec::new(),
            },
            grid::ContainerBaselineInput {
                track_offsets: vec![0.0, 40.0],
                track_sizes: vec![30.0, 30.0],
                groups: grid::BaselineGroupReport {
                    major: vec![Some(14.0), None],
                    minor: vec![Some(6.0)],
                    participation: Vec::new(),
                },
                fallback_items: Vec::new(),
            },
        ];

        for input in cases {
            let error = grid::container_baselines(input).unwrap_err();

            assert_eq!(error, grid::OracleGridError::SpanOutOfRange);
        }
    }

    #[test]
    fn oracle_container_baselines_reject_invalid_fallback_spans() {
        let valid_item = grid::ContainerBaselineFallbackItem {
            id: "fallback",
            area: grid::GridArea::new(1, 1, 1, 1),
            block_offset: 0.0,
            first_baseline: 8.0,
            last_baseline: 22.0,
        };
        let cases = [
            grid::ContainerBaselineFallbackItem {
                area: grid::GridArea::new(1, 0, 1, 1),
                ..valid_item
            },
            grid::ContainerBaselineFallbackItem {
                area: grid::GridArea::new(1, 1, 1, 0),
                ..valid_item
            },
            grid::ContainerBaselineFallbackItem {
                area: grid::GridArea::new(1, 2, 1, 2),
                ..valid_item
            },
            grid::ContainerBaselineFallbackItem {
                area: grid::GridArea::new(0, 1, 1, 1),
                ..valid_item
            },
            grid::ContainerBaselineFallbackItem {
                area: grid::GridArea::new(1, 1, 0, 1),
                ..valid_item
            },
        ];

        for item in cases {
            let error = grid::container_baselines(grid::ContainerBaselineInput {
                track_offsets: vec![0.0, 40.0],
                track_sizes: vec![30.0, 30.0],
                groups: grid::BaselineGroupReport {
                    major: vec![None, None],
                    minor: vec![None, None],
                    participation: Vec::new(),
                },
                fallback_items: vec![item],
            })
            .unwrap_err();

            assert_eq!(error, grid::OracleGridError::SpanOutOfRange);
        }
    }

    #[test]
    fn grid_definite_tracks_distribute_leftover_space_to_fr_tracks() {
        let tracks = DefiniteTracks::new(300.0, 10.0)
            .track(Track::px(50.0))
            .track(Track::fr(1.0))
            .track(Track::fr(2.0))
            .solve();

        let one_fr = 230.0 / 3.0;
        assert_eq!(tracks.sizes().len(), 3);
        assert_eq!(tracks.size(0), 50.0);
        assert_eq!(tracks.offset(0), 0.0);
        assert!((tracks.size(1) - one_fr).abs() < 0.000_001);
        assert_eq!(tracks.offset(1), 60.0);
        assert!((tracks.size(2) - one_fr * 2.0).abs() < 0.000_001);
        assert!((tracks.offset(2) - (70.0 + one_fr)).abs() < 0.000_001);
    }

    #[test]
    fn grid_explicit_tracks_resolve_percent_and_fr_after_fixed_tracks_and_gaps() {
        let tracks = DefiniteTracks::new(400.0, 20.0)
            .track(Track::px(80.0))
            .track(Track::percent(0.25))
            .track(Track::fr(1.0))
            .track(Track::fr(3.0))
            .solve();

        assert_eq!(tracks.size(0), 80.0);
        assert_eq!(tracks.size(1), 100.0);
        assert_eq!(tracks.size(2), 40.0);
        assert_eq!(tracks.size(3), 120.0);
        assert_eq!(tracks.offset(0), 0.0);
        assert_eq!(tracks.offset(1), 100.0);
        assert_eq!(tracks.offset(2), 220.0);
        assert_eq!(tracks.offset(3), 280.0);
    }

    #[test]
    fn grid_fraction_tracks_do_not_expand_sub_one_factor_to_all_leftover_space() {
        let tracks = DefiniteTracks::new(200.0, 0.0)
            .track(Track::px(50.0))
            .track(Track::fr(0.5))
            .solve();

        assert_eq!(tracks.size(0), 50.0);
        assert_eq!(tracks.size(1), 75.0);

        let report = TrackSizingSlice::definite_columns(200.0, 0.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::flex(0.5))
            .solve();

        assert_eq!(report.flex_fraction, Some(150.0));
        assert_eq!(report.final_tracks[1].size, 75.0);
    }

    #[test]
    fn grid_line_area_resolves_spans_across_tracks_and_gaps() {
        let tracks = DefiniteTracks::new(150.0, 5.0)
            .track(Track::px(30.0))
            .track(Track::px(40.0))
            .track(Track::px(50.0))
            .solve();

        let area = tracks.area(2, 4);

        assert_eq!(area.start, 35.0);
        assert_eq!(area.size, 95.0);
    }

    #[test]
    fn grid_auto_placement_places_row_column_and_dense_items() {
        let mut row = AutoPlacer::try_new(3, 2, Flow::Row)
            .unwrap()
            .occupied(GridArea::new(2, 1, 1, 1));
        assert_eq!(row.place(2, 1).unwrap(), GridArea::new(1, 2, 2, 1));
        assert_eq!(row.place(1, 1).unwrap(), GridArea::new(3, 2, 1, 1));

        let mut column = AutoPlacer::try_new(2, 3, Flow::Column)
            .unwrap()
            .occupied(GridArea::new(1, 2, 1, 1));
        assert_eq!(column.place(1, 2).unwrap(), GridArea::new(2, 1, 1, 2));
        assert_eq!(column.place(1, 1).unwrap(), GridArea::new(2, 3, 1, 1));

        let mut column_dense = AutoPlacer::try_new(2, 3, Flow::ColumnDense)
            .unwrap()
            .occupied(GridArea::new(1, 1, 1, 1))
            .occupied(GridArea::new(1, 2, 1, 1));
        assert_eq!(column_dense.place(1, 1).unwrap(), GridArea::new(1, 3, 1, 1));

        let mut dense = AutoPlacer::try_new(3, 2, Flow::RowDense)
            .unwrap()
            .occupied(GridArea::new(2, 1, 1, 1))
            .occupied(GridArea::new(1, 2, 2, 1));
        assert_eq!(dense.place(1, 1).unwrap(), GridArea::new(1, 1, 1, 1));
    }

    #[test]
    fn grid_auto_placement_reports_zero_explicit_tracks() {
        assert_eq!(
            AutoPlacer::try_new(0, 1, Flow::Row).unwrap_err(),
            PlacementError::NoExplicitTracks(GridAxis::Column)
        );
        assert_eq!(
            AutoPlacer::try_new(1, 0, Flow::Row).unwrap_err(),
            PlacementError::NoExplicitTracks(GridAxis::Row)
        );
    }

    #[test]
    fn grid_auto_placement_reports_row_flow_span_wider_than_columns() {
        let mut placer = AutoPlacer::try_new(2, 1, Flow::Row).unwrap();

        assert_eq!(
            placer.place(3, 1).unwrap_err(),
            PlacementError::SpanExceedsExplicitTracks {
                axis: GridAxis::Column,
                span: 3,
                explicit_tracks: 2,
            }
        );
    }

    #[test]
    fn grid_auto_placement_reports_column_flow_span_taller_than_rows() {
        let mut placer = AutoPlacer::try_new(1, 2, Flow::Column).unwrap();

        assert_eq!(
            placer.place(1, 3).unwrap_err(),
            PlacementError::SpanExceedsExplicitTracks {
                axis: GridAxis::Row,
                span: 3,
                explicit_tracks: 2,
            }
        );
    }

    #[test]
    fn grid_placement_resolves_start_and_end_lines() {
        let placement = LinePlacement::Lines { start: 2, end: 5 }
            .resolve_axis(1)
            .unwrap();

        assert_eq!(placement.start_line, 2);
        assert_eq!(placement.end_line, 5);
        assert_eq!(placement.span, 3);
    }

    #[test]
    fn grid_placement_resolves_start_line_plus_span() {
        let placement = LinePlacement::LineSpan { start: 3, span: 2 }
            .resolve_axis(1)
            .unwrap();

        assert_eq!(placement.start_line, 3);
        assert_eq!(placement.end_line, 5);
        assert_eq!(placement.span, 2);
    }

    #[test]
    fn grid_placement_resolves_span_plus_end_line() {
        let placement = LinePlacement::SpanLine { span: 2, end: 5 }
            .resolve_axis(1)
            .unwrap();

        assert_eq!(placement.start_line, 3);
        assert_eq!(placement.end_line, 5);
        assert_eq!(placement.span, 2);
    }

    #[test]
    fn grid_placement_defaults_auto_auto_to_one_track_span() {
        let placement = LinePlacement::Auto.resolve_axis(4).unwrap();

        assert_eq!(placement.start_line, 4);
        assert_eq!(placement.end_line, 5);
        assert_eq!(placement.span, 1);
    }

    #[test]
    fn grid_placement_extends_implicit_tracks_after_explicit_grid() {
        let placement = LinePlacement::Line(4).resolve_axis(1).unwrap();

        assert_eq!(placement.start_line, 4);
        assert_eq!(placement.end_line, 5);
        assert_eq!(placement.span, 1);
        assert_eq!(placement.implicit_after(3), 1);
    }

    #[test]
    fn grid_item_placement_resolves_two_axes_to_area() {
        let placement = ItemPlacement {
            column: LinePlacement::LineSpan { start: 2, span: 2 },
            row: LinePlacement::SpanLine { span: 2, end: 4 },
        }
        .resolve(1, 1)
        .unwrap();

        assert_eq!(placement.column.start_line, 2);
        assert_eq!(placement.column.end_line, 4);
        assert_eq!(placement.row.start_line, 2);
        assert_eq!(placement.row.end_line, 4);
        assert_eq!(placement.area(), GridArea::new(2, 2, 2, 2));
    }

    fn named_columns(explicit_track_count: usize, line_names: Vec<Vec<&str>>) -> NamedGridLines {
        NamedGridLines::new(GridAxis::Column, explicit_track_count, line_names).unwrap()
    }

    #[test]
    fn oracle_named_grid_lines_empty_initializes_all_explicit_lines() {
        let lines = NamedGridLines::empty(GridAxis::Column, 2);

        assert_eq!(lines.explicit_track_count, 2);
        assert!(lines.line_names(1).is_empty());
        assert!(lines.line_names(2).is_empty());
        assert!(lines.line_names(3).is_empty());
    }

    #[test]
    fn oracle_named_grid_lines_return_names_by_one_based_line() {
        let lines = named_columns(2, vec![vec!["a"], vec!["b", "c"], vec![]]);

        assert_eq!(lines.line_names(1), vec!["a"]);
        assert_eq!(lines.line_names(2), vec!["b", "c"]);
        assert!(lines.line_names(3).is_empty());
        assert!(lines.line_names(0).is_empty());
    }

    #[test]
    fn oracle_named_grid_lines_reject_reserved_names() {
        let auto_err =
            NamedGridLines::new(GridAxis::Column, 1, vec![vec!["auto"], vec![]]).unwrap_err();
        let span_err =
            NamedGridLines::new(GridAxis::Column, 1, vec![vec!["span"], vec![]]).unwrap_err();

        assert_eq!(
            auto_err,
            NamedGridError::ReservedLineName {
                name: "auto".to_owned(),
            }
        );
        assert_eq!(
            span_err,
            NamedGridError::ReservedLineName {
                name: "span".to_owned(),
            }
        );
    }

    #[test]
    fn oracle_named_line_occurrence_shape_is_exported() {
        let occurrence = NamedLineOccurrence {
            line: 2,
            origin: LineNameOrigin::Explicit,
        };

        assert_eq!(occurrence.line, 2);
        assert_eq!(occurrence.origin, LineNameOrigin::Explicit);
    }

    #[test]
    fn oracle_named_grid_lines_preserve_repeated_names_in_source_order() {
        let lines = named_columns(3, vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]]);

        assert_eq!(lines.named_occurrences("a"), vec![1, 2, 3]);
        assert_eq!(lines.named_occurrences("b"), vec![2, 4]);
    }

    #[test]
    fn oracle_named_grid_lines_reject_mismatched_line_count() {
        let err = grid::NamedGridLines::new(grid::GridAxis::Row, 2, vec![vec!["a"], vec!["b"]])
            .unwrap_err();

        assert_eq!(
            err,
            grid::NamedGridError::LineNameCountMismatch {
                axis: grid::GridAxis::Row,
                explicit_track_count: 2,
                line_count: 2,
            }
        );
    }

    #[test]
    fn oracle_named_fixed_repeat_expands_line_names_between_tracks() {
        let expanded = grid::expand_named_fixed_repeat(
            grid::GridAxis::Column,
            2,
            [
                grid::NamedTrackComponent::LineNames(vec!["a".to_owned()]),
                grid::NamedTrackComponent::Track,
                grid::NamedTrackComponent::LineNames(vec!["b".to_owned()]),
                grid::NamedTrackComponent::Track,
                grid::NamedTrackComponent::LineNames(vec!["c".to_owned()]),
            ],
        )
        .unwrap();

        assert_eq!(expanded.explicit_track_count, 4);
        assert_eq!(expanded.named_occurrences("a"), vec![1, 3]);
        assert_eq!(expanded.named_occurrences("b"), vec![2, 4]);
        assert_eq!(expanded.named_occurrences("c"), vec![3, 5]);
    }

    #[test]
    fn oracle_named_fixed_repeat_merges_adjacent_line_name_lists() {
        let expanded = grid::expand_named_fixed_repeat(
            grid::GridAxis::Column,
            2,
            [
                grid::NamedTrackComponent::LineNames(vec!["start".to_owned()]),
                grid::NamedTrackComponent::Track,
                grid::NamedTrackComponent::LineNames(vec!["end".to_owned()]),
                grid::NamedTrackComponent::LineNames(vec!["next".to_owned()]),
                grid::NamedTrackComponent::Track,
            ],
        )
        .unwrap();

        assert_eq!(expanded.explicit_track_count, 4);
        assert_eq!(expanded.line_names(2), vec!["end", "next"]);
        assert_eq!(expanded.line_names(3), vec!["start"]);
    }

    #[test]
    fn oracle_named_fixed_repeat_rejects_zero_repeat() {
        assert_eq!(
            grid::expand_named_fixed_repeat(
                grid::GridAxis::Column,
                0,
                [grid::NamedTrackComponent::Track],
            )
            .unwrap_err(),
            grid::NamedGridError::ZeroRepeat
        );
    }

    #[test]
    fn oracle_named_fixed_repeat_rejects_reserved_line_names() {
        assert_eq!(
            grid::expand_named_fixed_repeat(
                grid::GridAxis::Column,
                1,
                [grid::NamedTrackComponent::LineNames(vec![
                    "span".to_owned(),
                ])],
            )
            .unwrap_err(),
            grid::NamedGridError::ReservedLineName {
                name: "span".to_owned(),
            }
        );
    }

    #[test]
    fn oracle_named_line_lookup_counts_positive_occurrences_from_start() {
        let lines = named_columns(3, vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]]);
        let report = grid::resolve_named_line(&lines, "a", 2).unwrap();

        assert_eq!(report.resolved_line, 2);
        assert_eq!(report.explicit_matches, vec![1, 2, 3]);
        assert!(report.implicit_lines_assumed_named.is_empty());
    }

    #[test]
    fn oracle_named_line_lookup_counts_negative_occurrences_from_end() {
        let lines = named_columns(3, vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]]);
        let report = grid::resolve_named_line(&lines, "a", -1).unwrap();

        assert_eq!(report.resolved_line, 3);
        assert_eq!(report.explicit_matches, vec![1, 2, 3]);
    }

    #[test]
    fn oracle_named_line_lookup_extends_after_for_missing_positive_occurrence() {
        let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
        let report = grid::resolve_named_line(&lines, "a", 4).unwrap();

        assert_eq!(report.resolved_line, 5);
        assert_eq!(report.explicit_matches, vec![1, 3]);
        assert_eq!(report.implicit_lines_assumed_named, vec![4, 5]);
    }

    #[test]
    fn oracle_named_line_lookup_extends_before_for_missing_negative_occurrence() {
        let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
        let report = grid::resolve_named_line(&lines, "a", -3).unwrap();

        assert_eq!(report.resolved_line, 0);
        assert_eq!(report.explicit_matches, vec![1, 3]);
        assert_eq!(report.implicit_lines_assumed_named, vec![0]);
    }

    #[test]
    fn oracle_named_line_lookup_rejects_zero_occurrence() {
        let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

        assert_eq!(
            grid::resolve_named_line(&lines, "a", 0).unwrap_err(),
            grid::NamedGridError::ZeroLine
        );
    }

    #[test]
    fn oracle_named_line_lookup_rejects_reserved_custom_ident() {
        let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

        assert_eq!(
            grid::resolve_named_line(&lines, "auto", 1).unwrap_err(),
            grid::NamedGridError::ReservedLineName {
                name: "auto".to_owned(),
            }
        );
        assert_eq!(
            grid::resolve_named_line(&lines, "span", 1).unwrap_err(),
            grid::NamedGridError::ReservedLineName {
                name: "span".to_owned(),
            }
        );
    }

    #[test]
    fn oracle_named_numeric_positive_line_passes_through() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);

        assert_eq!(grid::resolve_numeric_line(&lines, 3).unwrap(), 3);
    }

    #[test]
    fn oracle_named_numeric_negative_line_counts_from_explicit_end() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);

        assert_eq!(grid::resolve_numeric_line(&lines, -1).unwrap(), 5);
        assert_eq!(grid::resolve_numeric_line(&lines, -2).unwrap(), 4);
    }

    #[test]
    fn oracle_named_numeric_zero_line_is_invalid() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);

        assert_eq!(
            grid::resolve_numeric_line(&lines, 0).unwrap_err(),
            grid::NamedGridError::ZeroLine,
        );
    }

    #[test]
    fn oracle_named_span_from_start_finds_nth_named_line_forward() {
        let lines = named_columns(
            5,
            vec![vec!["a"], vec![], vec!["a"], vec![], vec!["a"], vec![]],
        );

        let report = grid::resolve_named_span_from_start(&lines, 1, "a", 2).unwrap();

        assert_eq!(report.resolved_line, 5);
    }

    #[test]
    fn oracle_named_span_from_start_skips_explicit_end_line_for_implicit_names() {
        let lines = named_columns(4, vec![vec!["a"], vec![], vec!["a"], vec![], vec![]]);

        let report = grid::resolve_named_span_from_start(&lines, 1, "a", 2).unwrap();

        assert_eq!(report.resolved_line, 6);
        assert_eq!(report.explicit_matches, vec![1, 3]);
        assert_eq!(report.implicit_lines_assumed_named, vec![6]);
    }

    #[test]
    fn oracle_named_span_from_end_finds_nth_named_line_backward() {
        let lines = named_columns(
            5,
            vec![vec!["a"], vec![], vec!["a"], vec![], vec!["a"], vec![]],
        );

        let report = grid::resolve_named_span_from_end(&lines, 5, "a", 2).unwrap();

        assert_eq!(report.resolved_line, 1);
    }

    #[test]
    fn oracle_named_span_extends_implicitly_when_name_is_missing() {
        let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

        let report = grid::resolve_named_span_from_start(&lines, 3, "a", 2).unwrap();

        assert_eq!(report.resolved_line, 5);
        assert_eq!(report.implicit_lines_assumed_named, vec![4, 5]);
    }

    #[test]
    fn oracle_named_span_extends_implicitly_backward_when_name_is_missing() {
        let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

        let report = grid::resolve_named_span_from_end(&lines, 1, "a", 2).unwrap();

        assert_eq!(report.resolved_line, -1);
        assert_eq!(report.explicit_matches, vec![1, 3]);
        assert_eq!(report.implicit_lines_assumed_named, vec![0, -1]);
    }

    #[test]
    fn oracle_named_span_rejects_zero_count() {
        let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

        assert_eq!(
            grid::resolve_named_span_from_start(&lines, 1, "a", 0).unwrap_err(),
            grid::NamedGridError::ZeroSpan
        );
        assert_eq!(
            grid::resolve_named_span_from_end(&lines, 3, "a", 0).unwrap_err(),
            grid::NamedGridError::ZeroSpan
        );
    }

    #[test]
    fn oracle_named_span_rejects_reserved_custom_ident() {
        let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

        assert_eq!(
            grid::resolve_named_span_from_start(&lines, 1, "auto", 1).unwrap_err(),
            grid::NamedGridError::ReservedLineName {
                name: "auto".to_owned(),
            }
        );
        assert_eq!(
            grid::resolve_named_span_from_start(&lines, 1, "span", 1).unwrap_err(),
            grid::NamedGridError::ReservedLineName {
                name: "span".to_owned(),
            }
        );
        assert_eq!(
            grid::resolve_named_span_from_end(&lines, 3, "auto", 1).unwrap_err(),
            grid::NamedGridError::ReservedLineName {
                name: "auto".to_owned(),
            }
        );
        assert_eq!(
            grid::resolve_named_span_from_end(&lines, 3, "span", 1).unwrap_err(),
            grid::NamedGridError::ReservedLineName {
                name: "span".to_owned(),
            }
        );
    }

    #[test]
    fn oracle_named_axis_resolves_named_start_and_named_end() {
        let lines = named_columns(4, vec![vec!["a"], vec![], vec!["b"], vec![], vec!["b"]]);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Named {
                    name: "a".to_owned(),
                    occurrence: 1,
                },
                end: grid::NamedGridLine::Named {
                    name: "b".to_owned(),
                    occurrence: 2,
                },
            },
            None,
        )
        .unwrap();

        assert_eq!(report.resolved.start_line, 1);
        assert_eq!(report.resolved.end_line, 5);
        assert_eq!(report.resolved.span, 4);
    }

    #[test]
    fn oracle_named_axis_resolves_line_to_named_span() {
        let lines = named_columns(4, vec![vec!["a"], vec![], vec!["a"], vec![], vec![]]);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Number(1),
                end: grid::NamedGridLine::Span {
                    name: Some("a".to_owned()),
                    count: 2,
                },
            },
            None,
        )
        .unwrap();

        assert_eq!(report.resolved.start_line, 1);
        assert_eq!(report.resolved.end_line, 6);
    }

    #[test]
    fn oracle_named_axis_resolves_required_mixed_forms() {
        let lines = named_columns(
            5,
            vec![vec!["a"], vec![], vec!["b"], vec!["a"], vec![], vec!["b"]],
        );

        let named_to_span = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Named {
                    name: "a".to_owned(),
                    occurrence: 1,
                },
                end: grid::NamedGridLine::Span {
                    name: Some("b".to_owned()),
                    count: 2,
                },
            },
            None,
        )
        .unwrap();
        assert_eq!(
            (
                named_to_span.resolved.start_line,
                named_to_span.resolved.end_line,
            ),
            (1, 6)
        );

        let span_to_number = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Span {
                    name: Some("a".to_owned()),
                    count: 1,
                },
                end: grid::NamedGridLine::Number(6),
            },
            None,
        )
        .unwrap();
        assert_eq!(
            (
                span_to_number.resolved.start_line,
                span_to_number.resolved.end_line,
            ),
            (4, 6)
        );

        let span_to_named = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Span {
                    name: Some("a".to_owned()),
                    count: 1,
                },
                end: grid::NamedGridLine::Named {
                    name: "b".to_owned(),
                    occurrence: 2,
                },
            },
            None,
        )
        .unwrap();
        assert_eq!(
            (
                span_to_named.resolved.start_line,
                span_to_named.resolved.end_line,
            ),
            (4, 6)
        );

        let auto_to_number = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Auto,
                end: grid::NamedGridLine::Number(4),
            },
            Some(2),
        )
        .unwrap();
        assert_eq!(
            (
                auto_to_number.resolved.start_line,
                auto_to_number.resolved.end_line,
            ),
            (3, 4)
        );

        let number_to_auto = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Number(2),
                end: grid::NamedGridLine::Auto,
            },
            Some(4),
        )
        .unwrap();
        assert_eq!(
            (
                number_to_auto.resolved.start_line,
                number_to_auto.resolved.end_line,
            ),
            (2, 3)
        );
    }

    #[test]
    fn oracle_named_axis_drops_end_span_when_both_sides_are_spans() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 3);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Span {
                    name: None,
                    count: 1,
                },
                end: grid::NamedGridLine::Span {
                    name: None,
                    count: 1,
                },
            },
            Some(2),
        )
        .unwrap();

        assert_eq!(
            report.conflict_resolution,
            Some(grid::NamedPlacementConflictResolution::DroppedEndSpan)
        );
        assert_eq!(report.resolved.start_line, 2);
        assert_eq!(report.resolved.end_line, 3);
    }

    #[test]
    fn oracle_named_axis_records_ordered_span_span_normalizations() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Span {
                    name: Some("a".to_owned()),
                    count: 4,
                },
                end: grid::NamedGridLine::Span {
                    name: Some("b".to_owned()),
                    count: 2,
                },
            },
            Some(2),
        )
        .unwrap();

        assert_eq!(
            report.conflict_resolutions,
            vec![
                grid::NamedPlacementConflictResolution::DroppedEndSpan,
                grid::NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne,
            ]
        );
        assert_eq!(
            report.conflict_resolution,
            Some(grid::NamedPlacementConflictResolution::DroppedEndSpan)
        );
        assert_eq!(report.resolved.start_line, 2);
        assert_eq!(report.resolved.end_line, 3);
    }

    #[test]
    fn oracle_named_axis_swaps_reversed_resolved_lines() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Number(4),
                end: grid::NamedGridLine::Number(2),
            },
            None,
        )
        .unwrap();

        assert_eq!(
            report.conflict_resolution,
            Some(grid::NamedPlacementConflictResolution::SwappedResolvedLines)
        );
        assert_eq!(report.resolved.start_line, 2);
        assert_eq!(report.resolved.end_line, 4);
    }

    #[test]
    fn oracle_named_axis_drops_equal_end_line_to_span_one() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Number(3),
                end: grid::NamedGridLine::Number(3),
            },
            None,
        )
        .unwrap();

        assert_eq!(
            report.conflict_resolution,
            Some(grid::NamedPlacementConflictResolution::DroppedEqualEndLine)
        );
        assert_eq!(report.resolved.start_line, 3);
        assert_eq!(report.resolved.end_line, 4);
    }

    #[test]
    fn oracle_named_axis_clears_end_lookup_when_equal_line_drops_end() {
        let lines = named_columns(4, vec![vec![], vec![], vec!["mark"], vec![], vec![]]);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Number(3),
                end: grid::NamedGridLine::Named {
                    name: "mark".to_owned(),
                    occurrence: 1,
                },
            },
            None,
        )
        .unwrap();

        assert_eq!(
            report.normalized_end,
            grid::NamedGridLine::Span {
                name: None,
                count: 1,
            }
        );
        assert!(report.end_lookup.is_none());
        assert_eq!(report.resolved.start_line, 3);
        assert_eq!(report.resolved.end_line, 4);
    }

    #[test]
    fn oracle_named_axis_defaults_lone_start_named_span_to_one() {
        let lines = named_columns(3, vec![vec!["a"], vec![], vec!["a"], vec![]]);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Span {
                    name: Some("a".to_owned()),
                    count: 4,
                },
                end: grid::NamedGridLine::Auto,
            },
            Some(2),
        )
        .unwrap();

        assert_eq!(
            report.conflict_resolution,
            Some(grid::NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne)
        );
        assert_eq!(report.resolved.start_line, 2);
        assert_eq!(report.resolved.end_line, 3);
    }

    #[test]
    fn oracle_named_axis_defaults_lone_end_named_span_to_one() {
        let lines = named_columns(3, vec![vec!["a"], vec![], vec!["a"], vec![]]);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Auto,
                end: grid::NamedGridLine::Span {
                    name: Some("a".to_owned()),
                    count: 4,
                },
            },
            Some(2),
        )
        .unwrap();

        assert_eq!(
            report.conflict_resolution,
            Some(grid::NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne)
        );
        assert_eq!(report.resolved.start_line, 2);
        assert_eq!(report.resolved.end_line, 3);
    }

    #[test]
    fn oracle_named_axis_bare_ident_prefers_side_generated_line_name() {
        let lines = named_columns(
            3,
            vec![vec!["main-start"], vec![], vec![], vec!["main-end"]],
        );
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::BareIdent("main".to_owned()),
                end: grid::NamedGridLine::BareIdent("main".to_owned()),
            },
            None,
        )
        .unwrap();

        assert_eq!(report.resolved.start_line, 1);
        assert_eq!(report.resolved.end_line, 4);
    }

    #[test]
    fn oracle_named_axis_bare_ident_falls_back_to_raw_name_without_side_names() {
        let lines = named_columns(4, vec![vec![], vec!["foo"], vec![], vec!["foo"], vec![]]);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::BareIdent("foo".to_owned()),
                end: grid::NamedGridLine::Number(5),
            },
            None,
        )
        .unwrap();

        assert_eq!(report.start_lookup.as_ref().unwrap().name, "foo");
        assert_eq!(report.resolved.start_line, 2);
        assert_eq!(report.resolved.end_line, 5);
    }

    #[test]
    fn oracle_template_areas_generate_row_and_column_line_names() {
        let areas = grid::TemplateAreas::new([
            vec!["head", "head"],
            vec!["nav", "main"],
            vec!["nav", "main"],
        ])
        .unwrap();

        let columns = grid::area_generated_lines(
            grid::GridAxis::Column,
            &areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 2),
        )
        .unwrap();
        let rows = grid::area_generated_lines(
            grid::GridAxis::Row,
            &areas,
            grid::NamedGridLines::empty(grid::GridAxis::Row, 3),
        )
        .unwrap();

        assert_eq!(columns.line_names(1), vec!["head-start", "nav-start"]);
        assert_eq!(columns.line_names(2), vec!["nav-end", "main-start"]);
        assert_eq!(columns.line_names(3), vec!["head-end", "main-end"]);
        assert_eq!(rows.line_names(1), vec!["head-start"]);
        assert_eq!(
            rows.line_names(2),
            vec!["head-end", "nav-start", "main-start"]
        );
        assert_eq!(rows.line_names(4), vec!["nav-end", "main-end"]);
    }

    #[test]
    fn oracle_template_areas_reject_non_rectangular_area() {
        let err = grid::TemplateAreas::new([vec!["a", "a"], vec!["a", "b"]]).unwrap_err();

        assert_eq!(
            err,
            grid::NamedGridError::AreaNotRectangular {
                area: "a".to_owned(),
            }
        );
    }

    #[test]
    fn oracle_template_areas_reject_empty_matrix() {
        assert_eq!(
            grid::TemplateAreas::new(Vec::<Vec<&str>>::new()).unwrap_err(),
            grid::NamedGridError::EmptyTemplateAreas,
        );
    }

    #[test]
    fn oracle_template_areas_reject_mismatched_row_lengths() {
        let err = grid::TemplateAreas::new([vec!["a", "a"], vec!["a"]]).unwrap_err();

        assert_eq!(
            err,
            grid::NamedGridError::TemplateAreaRowLengthMismatch {
                expected: 2,
                actual: 1,
                row: 2,
            }
        );
    }

    #[test]
    fn oracle_template_areas_treat_dot_runs_as_null_cells() {
        let areas = grid::TemplateAreas::new([vec!["....", "main"]]).unwrap();

        assert!(!areas.contains_area("...."));
        assert!(areas.contains_area("main"));
    }

    #[test]
    fn oracle_template_areas_expand_base_line_map_to_template_size() {
        let areas = grid::TemplateAreas::new([vec!["a", "a", "a"]]).unwrap();
        let columns = grid::area_generated_lines(
            grid::GridAxis::Column,
            &areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 1),
        )
        .unwrap();

        assert_eq!(columns.explicit_track_count, 3);
        assert_eq!(columns.line_names(1), vec!["a-start"]);
        assert_eq!(columns.line_names(4), vec!["a-end"]);
    }

    #[test]
    fn oracle_template_areas_preserve_larger_base_line_map() {
        let areas = grid::TemplateAreas::new([vec!["a"]]).unwrap();
        let columns = grid::area_generated_lines(
            grid::GridAxis::Column,
            &areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 3),
        )
        .unwrap();

        assert_eq!(columns.explicit_track_count, 3);
        assert_eq!(columns.line_names(1), vec!["a-start"]);
        assert_eq!(columns.line_names(2), vec!["a-end"]);
    }

    #[test]
    fn oracle_template_areas_preserve_explicit_names_before_generated_names() {
        let areas = grid::TemplateAreas::new([vec!["a"]]).unwrap();
        let columns = grid::area_generated_lines(
            grid::GridAxis::Column,
            &areas,
            named_columns(1, vec![vec!["explicit"], vec![]]),
        )
        .unwrap();

        assert_eq!(columns.line_names(1), vec!["explicit", "a-start"]);
        assert_eq!(
            columns.line_names[0][1].origin,
            grid::LineNameOrigin::AreaGenerated
        );
    }

    #[test]
    fn oracle_template_areas_generate_facts_for_both_axes() {
        let areas = grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
        let facts = grid::area_generated_facts(
            &areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 2),
            grid::NamedGridLines::empty(grid::GridAxis::Row, 1),
        )
        .unwrap();

        assert_eq!(facts.columns.line_names(1), vec!["a-start"]);
        assert_eq!(facts.columns.line_names(3), vec!["a-end"]);
        assert_eq!(facts.rows.line_names(1), vec!["a-start"]);
        assert_eq!(facts.rows.line_names(2), vec!["a-end"]);
        assert_eq!(facts.areas.area_rectangle("a").unwrap().column_end, 3);
    }

    #[test]
    fn oracle_template_areas_resolve_area_to_generated_named_lines() {
        let areas = grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
        let placement = grid::resolve_named_area(&areas, "a").unwrap();

        assert_eq!(
            placement.column.start,
            grid::NamedGridLine::Named {
                name: "a-start".to_owned(),
                occurrence: 1,
            }
        );
        assert_eq!(
            placement.row.end,
            grid::NamedGridLine::Named {
                name: "a-end".to_owned(),
                occurrence: 1,
            }
        );
    }

    #[test]
    fn oracle_template_areas_reject_missing_area_resolution() {
        let areas = grid::TemplateAreas::new([vec!["a"]]).unwrap();

        assert_eq!(
            grid::resolve_named_area(&areas, "b").unwrap_err(),
            grid::NamedGridError::AreaNotFound {
                area: "b".to_owned(),
            }
        );
    }

    #[test]
    fn oracle_named_grid_resolves_area_generated_names_to_grid_area() {
        let areas = grid::TemplateAreas::new([vec!["head", "head"], vec!["nav", "main"]]).unwrap();
        let columns = grid::area_generated_lines(
            grid::GridAxis::Column,
            &areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 2),
        )
        .unwrap();
        let rows = grid::area_generated_lines(
            grid::GridAxis::Row,
            &areas,
            grid::NamedGridLines::empty(grid::GridAxis::Row, 2),
        )
        .unwrap();

        assert_eq!(columns.named_occurrences("main-start"), vec![2]);
        assert_eq!(rows.named_occurrences("main-start"), vec![2]);

        let report = grid::resolve_named_grid_area_report(&columns, &rows, "main").unwrap();

        assert_eq!(report.area, grid::GridArea::new(2, 2, 1, 1));
        assert_eq!(
            report.column.start_lookup.as_ref().unwrap().name,
            "main-start"
        );
        assert_eq!(report.column.end_lookup.as_ref().unwrap().name, "main-end");
        assert_eq!(report.row.start_lookup.as_ref().unwrap().name, "main-start");
        assert_eq!(report.row.end_lookup.as_ref().unwrap().name, "main-end");
        assert!(
            report
                .column
                .start_lookup
                .as_ref()
                .unwrap()
                .implicit_lines_assumed_named
                .is_empty()
        );
    }

    #[test]
    fn oracle_axis_shorthand_repeats_omitted_custom_ident() {
        let expanded =
            grid::expand_axis_shorthand(grid::NamedGridLine::BareIdent("main".to_owned()), None);

        assert_eq!(
            expanded,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::BareIdent("main".to_owned()),
                end: grid::NamedGridLine::BareIdent("main".to_owned()),
            }
        );
    }

    #[test]
    fn oracle_axis_shorthand_defaults_omitted_non_ident_to_auto() {
        let expanded = grid::expand_axis_shorthand(grid::NamedGridLine::Number(2), None);

        assert_eq!(
            expanded,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Number(2),
                end: grid::NamedGridLine::Auto,
            }
        );
    }

    #[test]
    fn oracle_grid_area_shorthand_repeats_single_custom_ident_to_all_sides() {
        let expanded = grid::expand_grid_area_shorthand(vec![grid::NamedGridLine::BareIdent(
            "main".to_owned(),
        )])
        .unwrap();

        assert_eq!(
            expanded.row.start,
            grid::NamedGridLine::BareIdent("main".to_owned())
        );
        assert_eq!(
            expanded.row.end,
            grid::NamedGridLine::BareIdent("main".to_owned())
        );
        assert_eq!(
            expanded.column.start,
            grid::NamedGridLine::BareIdent("main".to_owned())
        );
        assert_eq!(
            expanded.column.end,
            grid::NamedGridLine::BareIdent("main".to_owned())
        );
    }

    #[test]
    fn oracle_grid_area_shorthand_expands_two_and_four_values() {
        let two = grid::expand_grid_area_shorthand(vec![
            grid::NamedGridLine::BareIdent("row".to_owned()),
            grid::NamedGridLine::BareIdent("col".to_owned()),
        ])
        .unwrap();
        assert_eq!(
            two.row.end,
            grid::NamedGridLine::BareIdent("row".to_owned())
        );
        assert_eq!(
            two.column.end,
            grid::NamedGridLine::BareIdent("col".to_owned())
        );

        let four = grid::expand_grid_area_shorthand(vec![
            grid::NamedGridLine::Number(1),
            grid::NamedGridLine::Number(2),
            grid::NamedGridLine::Number(3),
            grid::NamedGridLine::Number(4),
        ])
        .unwrap();
        assert_eq!(four.row.start, grid::NamedGridLine::Number(1));
        assert_eq!(four.column.start, grid::NamedGridLine::Number(2));
        assert_eq!(four.row.end, grid::NamedGridLine::Number(3));
        assert_eq!(four.column.end, grid::NamedGridLine::Number(4));
    }

    #[test]
    fn oracle_grid_area_shorthand_defaults_omitted_non_idents_to_auto() {
        let expanded = grid::expand_grid_area_shorthand(vec![
            grid::NamedGridLine::Number(2),
            grid::NamedGridLine::Number(3),
            grid::NamedGridLine::Number(4),
        ])
        .unwrap();

        assert_eq!(expanded.row.start, grid::NamedGridLine::Number(2));
        assert_eq!(expanded.row.end, grid::NamedGridLine::Number(4));
        assert_eq!(expanded.column.end, grid::NamedGridLine::Auto);
    }

    #[test]
    fn oracle_named_grid_resolves_subgrid_named_span_into_parent_space() {
        let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["b"], vec!["c"]]);
        let subgrid = grid::inherit_named_subgrid_lines(
            &parent,
            grid::TrackSpan::new(2, 5),
            false,
            vec![vec![], vec![], vec![], vec![]],
            None,
        )
        .unwrap();

        assert_eq!(subgrid.lines.line_names(1), vec!["b"]);
        assert_eq!(subgrid.lines.line_names(4), vec!["c"]);

        let report = grid::resolve_named_axis_placement(
            &subgrid.lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Named {
                    name: "b".to_owned(),
                    occurrence: 1,
                },
                end: grid::NamedGridLine::Span {
                    name: Some("c".to_owned()),
                    count: 1,
                },
            },
            None,
        )
        .unwrap();

        assert_eq!(report.start_lookup.as_ref().unwrap().resolved_line, 1);
        assert_eq!(report.end_lookup.as_ref().unwrap().resolved_line, 4);
        assert_eq!(report.resolved.start_line, 1);
        assert_eq!(report.resolved.end_line, 4);
    }

    #[test]
    fn oracle_named_axis_auto_auto_with_cursor_resolves_one_track_span() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);
        let report = grid::resolve_named_axis_placement(
            &lines,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Auto,
                end: grid::NamedGridLine::Auto,
            },
            Some(3),
        )
        .unwrap();

        assert_eq!(report.resolved.start_line, 3);
        assert_eq!(report.resolved.end_line, 4);
        assert_eq!(report.resolved.span, 1);
    }

    #[test]
    fn oracle_named_axis_unresolved_auto_without_cursor_returns_error() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);

        assert_eq!(
            grid::resolve_named_axis_placement(
                &lines,
                grid::NamedAxisPlacement {
                    start: grid::NamedGridLine::Auto,
                    end: grid::NamedGridLine::Auto,
                },
                None,
            )
            .unwrap_err(),
            grid::NamedGridError::AutoWithoutCursor
        );
    }

    #[test]
    fn oracle_named_axis_maps_line_before_first_error() {
        let lines = grid::NamedGridLines::empty(grid::GridAxis::Column, 4);

        assert_eq!(
            grid::resolve_named_axis_placement(
                &lines,
                grid::NamedAxisPlacement {
                    start: grid::NamedGridLine::Number(-10),
                    end: grid::NamedGridLine::Number(2),
                },
                None,
            )
            .unwrap_err(),
            grid::NamedGridError::LineBeforeFirst {
                axis: grid::GridAxis::Column,
                start_line: -4,
                end_line: 2,
            }
        );
    }

    #[test]
    fn oracle_anonymous_span_offsets_from_known_edge() {
        assert_eq!(grid::resolve_anonymous_span_from_start(2, 3).unwrap(), 5);
        assert_eq!(grid::resolve_anonymous_span_from_end(5, 3).unwrap(), 2);
    }

    #[test]
    fn oracle_anonymous_span_rejects_zero_count() {
        assert_eq!(
            grid::resolve_anonymous_span_from_start(2, 0).unwrap_err(),
            grid::NamedGridError::ZeroSpan
        );
        assert_eq!(
            grid::resolve_anonymous_span_from_end(5, 0).unwrap_err(),
            grid::NamedGridError::ZeroSpan
        );
    }

    #[test]
    fn grid_track_report_initializes_fixed_percent_and_flex_tracks() {
        let report = TrackSizingSlice::definite_columns(400.0, 10.0)
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::percent(0.25))
            .track(GridTrack::flex(1.0))
            .solve();

        assert_eq!(
            report.initialized.tracks,
            vec![
                TrackSize::new(80.0, GrowthLimit::Definite(80.0)),
                TrackSize::new(100.0, GrowthLimit::Definite(100.0)),
                TrackSize::new(0.0, GrowthLimit::Infinite),
            ]
        );
        assert_eq!(report.after_intrinsic_minimums, report.initialized);
        assert_eq!(report.after_content_based_minimums, report.initialized);
        assert_eq!(report.after_spanning_items, report.initialized);
        assert_eq!(report.after_maximize_tracks, report.initialized);
        assert_eq!(report.flex_fraction, Some(200.0));
        assert_eq!(report.final_tracks[0].size, 80.0);
        assert_eq!(report.final_tracks[0].offset, 0.0);
        assert_eq!(report.final_tracks[1].size, 100.0);
        assert_eq!(report.final_tracks[1].offset, 90.0);
        assert_eq!(report.final_tracks[2].size, 200.0);
        assert_eq!(report.final_tracks[2].offset, 200.0);
    }

    #[test]
    fn grid_track_report_initializes_auto_and_intrinsic_keywords() {
        let report = TrackSizingSlice::indefinite_columns(5.0)
            .track(GridTrack::auto())
            .track(GridTrack::new(TrackMin::MinContent, TrackMax::MaxContent))
            .track(GridTrack::new(
                TrackMin::MaxContent,
                TrackMax::FitContent(120.0),
            ))
            .solve();

        assert_eq!(
            report.initialized.tracks,
            vec![
                TrackSize::new(0.0, GrowthLimit::Infinite),
                TrackSize::new(0.0, GrowthLimit::Infinite),
                TrackSize::new(0.0, GrowthLimit::Definite(120.0)),
            ]
        );
        assert_eq!(report.flex_fraction, None);
        assert_eq!(report.final_tracks[0].offset, 0.0);
        assert_eq!(report.final_tracks[1].offset, 5.0);
        assert_eq!(report.final_tracks[2].offset, 10.0);
    }

    #[test]
    fn grid_track_report_initializes_minmax_growth_limits() {
        let report = TrackSizingSlice::definite_columns(200.0, 0.0)
            .track(GridTrack::new(TrackMin::Fixed(40.0), TrackMax::Fixed(90.0)))
            .track(GridTrack::new(TrackMin::Percent(0.25), TrackMax::Auto))
            .solve();

        assert_eq!(
            report.initialized.tracks,
            vec![
                TrackSize::new(40.0, GrowthLimit::Definite(90.0)),
                TrackSize::new(50.0, GrowthLimit::Infinite),
            ]
        );
        assert_eq!(
            report.after_maximize_tracks.tracks,
            vec![
                TrackSize::new(90.0, GrowthLimit::Definite(90.0)),
                TrackSize::new(50.0, GrowthLimit::Infinite),
            ]
        );
        assert_eq!(report.final_tracks[0].size, 90.0);
        assert_eq!(report.final_tracks[1].size, 50.0);
    }

    #[test]
    fn grid_contributions_use_supplied_intrinsic_facts_and_margins() {
        let contributions = ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 40.0,
            max_content: 90.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 5.0,
            margin_after: 7.0,
            automatic_minimum_applies: false,
        }
        .contributions();

        assert_eq!(
            contributions,
            ItemContributions {
                minimum: 12.0,
                min_content: 52.0,
                max_content: 102.0,
                limited_min_content: 52.0,
                limited_max_content: 102.0,
            }
        );
    }

    #[test]
    fn grid_contributions_apply_min_max_and_preferred_limits() {
        let contributions = ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 40.0,
            max_content: 100.0,
            preferred: ContributionSize::Definite(65.0),
            min_size: ContributionSize::Definite(50.0),
            max_size: ContributionSize::Auto,
            margin_before: 2.0,
            margin_after: 3.0,
            automatic_minimum_applies: true,
        }
        .contributions();

        assert_eq!(
            contributions,
            ItemContributions {
                minimum: 55.0,
                min_content: 45.0,
                max_content: 105.0,
                limited_min_content: 55.0,
                limited_max_content: 70.0,
            }
        );
    }

    #[test]
    fn grid_contributions_treat_explicit_infinite_max_as_unlimited() {
        let contributions = ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 20.0,
            max_content: 80.0,
            preferred: ContributionSize::Definite(50.0),
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Infinite,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        }
        .contributions();

        assert_eq!(contributions.minimum, 20.0);
        assert_eq!(contributions.limited_max_content, 50.0);
    }

    #[test]
    fn grid_intrinsic_single_span_grows_minimum_and_content_phases() {
        let report = TrackSizingSlice::indefinite_columns(0.0)
            .track(GridTrack::auto())
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 1),
                min_content: 80.0,
                max_content: 120.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Definite(30.0),
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: false,
            })
            .solve();

        assert_eq!(
            report.after_intrinsic_minimums.tracks,
            vec![TrackSize::new(30.0, GrowthLimit::Infinite)]
        );
        assert_eq!(
            report.after_content_based_minimums.tracks,
            vec![TrackSize::new(80.0, GrowthLimit::Infinite)]
        );
        assert_eq!(report.final_tracks[0].size, 80.0);
    }

    #[test]
    fn grid_intrinsic_single_span_clamps_to_growth_limit() {
        let report = TrackSizingSlice::indefinite_columns(0.0)
            .track(GridTrack::new(TrackMin::Auto, TrackMax::FitContent(40.0)))
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 1),
                min_content: 90.0,
                max_content: 120.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            })
            .solve();

        assert_eq!(
            report.after_content_based_minimums.tracks,
            vec![TrackSize::new(40.0, GrowthLimit::Definite(40.0))]
        );
        assert_eq!(report.final_tracks[0].size, 40.0);
    }

    #[test]
    fn grid_intrinsic_spanning_items_distribute_deficits_across_auto_tracks() {
        let report = TrackSizingSlice::indefinite_columns(10.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 2, 1),
                min_content: 110.0,
                max_content: 140.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            })
            .solve();

        assert_eq!(
            report.after_spanning_items.tracks,
            vec![
                TrackSize::new(50.0, GrowthLimit::Infinite),
                TrackSize::new(50.0, GrowthLimit::Infinite),
            ]
        );
        assert_eq!(report.final_tracks[0].offset, 0.0);
        assert_eq!(report.final_tracks[1].offset, 60.0);
    }

    #[test]
    fn grid_intrinsic_row_spanning_items_use_row_axis() {
        let report = TrackSizingSlice::indefinite_rows(10.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 2),
                min_content: 110.0,
                max_content: 140.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            })
            .solve();

        assert_eq!(
            report.after_spanning_items.tracks,
            vec![
                TrackSize::new(50.0, GrowthLimit::Infinite),
                TrackSize::new(50.0, GrowthLimit::Infinite),
            ]
        );
        assert_eq!(report.final_tracks[0].offset, 0.0);
        assert_eq!(report.final_tracks[1].offset, 60.0);
    }

    #[test]
    fn grid_intrinsic_spanning_items_report_unsupported_mixed_track_categories() {
        let error = TrackSizingSlice::indefinite_columns(10.0)
            .track(GridTrack::new(TrackMin::MinContent, TrackMax::MaxContent))
            .track(GridTrack::auto())
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 2, 1),
                min_content: 110.0,
                max_content: 140.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            })
            .try_solve()
            .unwrap_err();

        assert_eq!(
            error,
            TrackSizingError::UnsupportedSpanningTrackMix {
                axis: GridAxis::Column,
                start: 1,
                span: 2,
            }
        );
    }

    #[test]
    fn grid_maximize_tracks_distributes_free_space_to_finite_growth_limits() {
        let report = TrackSizingSlice::definite_columns(180.0, 0.0)
            .track(GridTrack::new(
                TrackMin::Fixed(50.0),
                TrackMax::Fixed(100.0),
            ))
            .track(GridTrack::new(TrackMin::Fixed(50.0), TrackMax::Fixed(80.0)))
            .solve();

        assert_eq!(
            report.after_maximize_tracks.tracks,
            vec![
                TrackSize::new(100.0, GrowthLimit::Definite(100.0)),
                TrackSize::new(80.0, GrowthLimit::Definite(80.0)),
            ]
        );
        assert_eq!(report.final_tracks[0].size, 100.0);
        assert_eq!(report.final_tracks[1].size, 80.0);
    }

    #[test]
    fn grid_flex_tracks_share_leftover_space_by_factor() {
        let report = TrackSizingSlice::definite_columns(300.0, 10.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::flex(1.0))
            .track(GridTrack::flex(2.0))
            .solve();

        assert_eq!(report.flex_fraction, Some(230.0 / 3.0));
        assert_eq!(
            report.after_flexing.tracks,
            vec![
                TrackSize::new(50.0, GrowthLimit::Definite(50.0)),
                TrackSize::new(230.0 / 3.0, GrowthLimit::Infinite),
                TrackSize::new(460.0 / 3.0, GrowthLimit::Infinite),
            ]
        );
    }

    #[test]
    fn grid_flex_tracks_recompute_fraction_after_oversized_base_tracks() {
        let report = TrackSizingSlice::definite_columns(300.0, 0.0)
            .track(GridTrack::flex(1.0))
            .track(GridTrack::flex(1.0))
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 1),
                min_content: 200.0,
                max_content: 200.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            })
            .solve();

        assert_eq!(report.flex_fraction, Some(100.0));
        assert_eq!(report.final_tracks[0].size, 200.0);
        assert_eq!(report.final_tracks[1].size, 100.0);
    }

    #[test]
    fn grid_flex_tracks_report_zero_fraction_when_no_space_remains() {
        let report = TrackSizingSlice::definite_columns(80.0, 0.0)
            .track(GridTrack::fixed(100.0))
            .track(GridTrack::flex(1.0))
            .solve();

        assert_eq!(report.flex_fraction, Some(0.0));
        assert_eq!(report.final_tracks[0].size, 100.0);
        assert_eq!(report.final_tracks[1].size, 0.0);
    }

    #[test]
    fn grid_stretch_grows_auto_tracks_after_flexing() {
        let report = TrackSizingSlice::definite_columns(120.0, 20.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .stretch_auto_tracks()
            .solve();

        assert_eq!(report.after_maximize_tracks, report.after_spanning_items);
        assert_eq!(
            report.after_stretch.tracks,
            vec![
                TrackSize::new(50.0, GrowthLimit::Infinite),
                TrackSize::new(50.0, GrowthLimit::Infinite),
            ]
        );
        assert_eq!(report.final_tracks[1].offset, 70.0);
    }

    #[test]
    fn grid_auto_placement_reports_placed_areas_cursor_and_implicit_growth() {
        let mut row = AutoPlacer::try_new(2, 1, Flow::Row).unwrap();
        assert_eq!(row.place(1, 1).unwrap(), GridArea::new(1, 1, 1, 1));
        assert_eq!(row.place(2, 1).unwrap(), GridArea::new(1, 2, 2, 1));

        let row_report = row.report();
        assert_eq!(
            row_report.areas,
            vec![GridArea::new(1, 1, 1, 1), GridArea::new(1, 2, 2, 1)]
        );
        assert_eq!(row_report.implicit_columns_after, 0);
        assert_eq!(row_report.implicit_rows_after, 1);
        assert_eq!(row_report.cursor.column, 1);
        assert_eq!(row_report.cursor.row, 3);

        let mut column = AutoPlacer::try_new(1, 2, Flow::Column).unwrap();
        assert_eq!(column.place(1, 1).unwrap(), GridArea::new(1, 1, 1, 1));
        assert_eq!(column.place(1, 2).unwrap(), GridArea::new(2, 1, 1, 2));

        let column_report = column.report();
        assert_eq!(
            column_report.areas,
            vec![GridArea::new(1, 1, 1, 1), GridArea::new(2, 1, 1, 2)]
        );
        assert_eq!(column_report.implicit_columns_after, 1);
        assert_eq!(column_report.implicit_rows_after, 0);
        assert_eq!(column_report.cursor.column, 3);
        assert_eq!(column_report.cursor.row, 1);
    }

    #[test]
    fn grid_equal_share_intrinsic_tracks_distribute_unbounded_spanning_deficits() {
        let tracks = EqualShareIntrinsicTracks::new(3)
            .base(0, 20.0)
            .item(1, 1, 50.0)
            .item(0, 3, 100.0)
            .solve(10.0);

        assert_eq!(tracks.size(0), 30.0);
        assert_eq!(tracks.size(1), 60.0);
        assert_eq!(tracks.size(2), 10.0);
        assert_eq!(tracks.offset(0), 0.0);
        assert_eq!(tracks.offset(1), 40.0);
        assert_eq!(tracks.offset(2), 110.0);
    }

    #[test]
    fn grid_auto_track_uses_stubbed_intrinsic_contribution_for_track_size() {
        let expected = EqualShareIntrinsicTracks::new(1)
            .item(0, 1, 80.0)
            .solve(0.0);
        let mut tree = OracleTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    grid_template_columns: vec![TrackComponent::AUTO],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    ..NodeInput::default()
                },
            )
            .style(2, NodeInput::default())
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(80.0, 10.0),
                    Size::new(80.0, 10.0),
                ))
                .run_mode(RunMode::ComputeSize),
            )
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(80.0, 10.0),
                    Size::new(80.0, 10.0),
                ))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(Some(80.0), Some(20.0))),
            );

        let output = crate::compute_grid(
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
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            ),
        )
        .unwrap();

        assert_eq!(output.size, Size::new(expected.size(0), 20.0));
        assert_eq!(
            tree.inputs(2).last().unwrap().known(),
            Size::new(Some(expected.size(0)), Some(20.0))
        );
        assert_eq!(
            tree.layout(2).unwrap().size,
            Size::new(expected.size(0), 10.0)
        );
    }

    #[test]
    fn grid_alignment_distributes_free_space_after_track_sizing() {
        let start = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::Start);
        assert_eq!(start.offset(0), 0.0);
        assert_eq!(start.offset(1), 60.0);

        let end = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::End);
        assert_eq!(end.offset(0), 90.0);
        assert_eq!(end.offset(1), 150.0);

        let center = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::Center);
        assert_eq!(center.offset(0), 45.0);
        assert_eq!(center.offset(1), 105.0);

        let between = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::SpaceBetween);
        assert_eq!(between.offset(0), 0.0);
        assert_eq!(between.offset(1), 150.0);

        let around = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::SpaceAround);
        assert_eq!(around.offset(0), 22.5);
        assert_eq!(around.offset(1), 127.5);

        let evenly = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::SpaceEvenly);
        assert!((evenly.offset(0) - 30.0).abs() < 0.000_001);
        assert!((evenly.offset(1) - 120.0).abs() < 0.000_001);
    }

    #[test]
    fn grid_alignment_report_exposes_distribution_and_safe_fallback() {
        let center = align_tracks_report(
            200.0,
            vec![50.0, 50.0],
            10.0,
            TrackAlignment::Center,
            AlignmentSafety::Unsafe,
        );
        assert_eq!(center.leading_offset, 45.0);
        assert_eq!(center.distributed_gap, 10.0);
        assert_eq!(center.offsets, vec![45.0, 105.0]);
        assert!(!center.safe_fallback_used);

        let between = align_tracks_report(
            200.0,
            vec![50.0, 50.0],
            10.0,
            TrackAlignment::SpaceBetween,
            AlignmentSafety::Unsafe,
        );
        assert_eq!(between.leading_offset, 0.0);
        assert_eq!(between.distributed_gap, 100.0);
        assert_eq!(between.offsets, vec![0.0, 150.0]);

        let safe = align_tracks_report(
            80.0,
            vec![50.0, 50.0],
            10.0,
            TrackAlignment::Center,
            AlignmentSafety::Safe,
        );
        assert_eq!(safe.leading_offset, 0.0);
        assert_eq!(safe.distributed_gap, 10.0);
        assert_eq!(safe.offsets, vec![0.0, 60.0]);
        assert!(safe.safe_fallback_used);
    }

    #[test]
    fn grid_scenario_composes_phase_reports_into_item_rects() {
        let mut placer = AutoPlacer::try_new(3, 1, Flow::Row).unwrap();
        assert_eq!(placer.place(1, 1).unwrap(), GridArea::new(1, 1, 1, 1));
        assert_eq!(placer.place(2, 1).unwrap(), GridArea::new(2, 1, 2, 1));
        let placement = placer.report();
        let columns = TrackSizingSlice::definite_columns(300.0, 10.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::flex(1.0))
            .track(GridTrack::flex(1.0))
            .solve();
        let rows = TrackSizingSlice::definite_rows(20.0, 0.0)
            .track(GridTrack::fixed(20.0))
            .solve();
        let column_alignment = align_tracks_report(
            300.0,
            columns
                .final_tracks
                .iter()
                .map(|track| track.size)
                .collect(),
            10.0,
            TrackAlignment::Start,
            AlignmentSafety::Unsafe,
        );
        let row_alignment = align_tracks_report(
            20.0,
            rows.final_tracks.iter().map(|track| track.size).collect(),
            0.0,
            TrackAlignment::Start,
            AlignmentSafety::Unsafe,
        );

        let scenario =
            compose_grid_scenario(placement, columns, rows, column_alignment, row_alignment);

        assert_eq!(
            scenario.item_rects,
            vec![
                GridItemRect::new(0.0, 0.0, 50.0, 20.0),
                GridItemRect::new(60.0, 0.0, 240.0, 20.0),
            ]
        );
    }

    #[test]
    fn oracle_tree_stubs_child_measurements_and_records_layout_inputs() {
        let mut tree = OracleTree::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                    grid_template_columns: vec![TrackComponent::px(120.0)],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    gap: Size::new(Length::px(8.0), Length::ZERO),
                    ..NodeInput::default()
                },
            )
            .style(2, NodeInput::default())
            .measure(
                2,
                ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(80.0, 10.0)),
            );

        let output = crate::compute_grid(
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
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            ),
        )
        .unwrap();

        assert_eq!(output.size, Size::new(120.0, 20.0));
        assert_eq!(
            tree.inputs(2).last().unwrap().run_mode(),
            RunMode::PerformLayout
        );
        assert_eq!(
            tree.inputs(2).last().unwrap().known(),
            Size::new(Some(120.0), Some(20.0))
        );
        assert_eq!(tree.layout(2).unwrap().size, Size::new(40.0, 10.0));
    }

    #[test]
    fn oracle_axis_mapping_preserves_parallel_horizontal_axes() {
        let report = grid::map_axis(grid::AxisMappingInput {
            queried_axis: GridAxis::Column,
            parent_writing_mode: grid::OracleWritingMode::HorizontalTb,
            child_writing_mode: grid::OracleWritingMode::HorizontalTb,
            parent_direction: grid::OracleDirection::Ltr,
            child_direction: grid::OracleDirection::Ltr,
            parent_flipped_in_resolved_axis: false,
            child_flipped_in_resolved_axis: false,
        });

        assert_eq!(report.parent_axis, GridAxis::Column);
        assert_eq!(report.child_axis, GridAxis::Column);
        assert!(!report.reversed);
    }

    #[test]
    fn oracle_axis_mapping_reports_reversed_when_flipped_states_differ() {
        let report = grid::map_axis(grid::AxisMappingInput {
            queried_axis: GridAxis::Row,
            parent_writing_mode: grid::OracleWritingMode::HorizontalTb,
            child_writing_mode: grid::OracleWritingMode::HorizontalTb,
            parent_direction: grid::OracleDirection::Rtl,
            child_direction: grid::OracleDirection::Ltr,
            parent_flipped_in_resolved_axis: true,
            child_flipped_in_resolved_axis: false,
        });

        assert_eq!(report.parent_axis, GridAxis::Row);
        assert_eq!(report.child_axis, GridAxis::Row);
        assert!(report.reversed);
    }

    #[test]
    fn oracle_subgrid_name_repeat_expands_to_used_span() {
        let expanded = grid::expand_subgrid_name_list(
            grid::GridAxis::Column,
            4,
            vec![
                grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
                grid::SubgridNameComponent::Repeat {
                    count: grid::SubgridNameRepeatCount::Number(2),
                    line_name_sets: vec![vec!["b".to_owned()]],
                },
                grid::SubgridNameComponent::LineNames(vec!["c".to_owned()]),
            ],
        )
        .unwrap();

        assert_eq!(
            expanded.local_line_names,
            vec![vec!["a"], vec!["b"], vec!["b"], vec!["c"], vec![],]
        );
    }

    #[test]
    fn oracle_subgrid_auto_fill_name_repeat_pads_to_used_span() {
        let expanded = grid::expand_subgrid_name_list(
            grid::GridAxis::Column,
            3,
            vec![grid::SubgridNameComponent::Repeat {
                count: grid::SubgridNameRepeatCount::AutoFill,
                line_name_sets: vec![vec!["b".to_owned()]],
            }],
        )
        .unwrap();

        assert_eq!(
            expanded.local_line_names,
            vec![vec!["b"], vec!["b"], vec!["b"], vec!["b"]]
        );
    }

    #[test]
    fn oracle_subgrid_auto_fill_name_repeat_reserves_trailing_fixed_names() {
        let expanded = grid::expand_subgrid_name_list(
            grid::GridAxis::Column,
            4,
            vec![
                grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
                grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
                grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
                grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
                grid::SubgridNameComponent::Repeat {
                    count: grid::SubgridNameRepeatCount::AutoFill,
                    line_name_sets: vec![vec!["b".to_owned()]],
                },
                grid::SubgridNameComponent::LineNames(vec!["c".to_owned()]),
            ],
        )
        .unwrap();

        assert_eq!(
            expanded.local_line_names,
            vec![vec!["a"], vec!["a"], vec!["a"], vec!["a"], vec!["c"]]
        );
    }

    #[test]
    fn oracle_subgrid_name_repeat_rejects_multiple_auto_fill_repeats() {
        assert_eq!(
            grid::expand_subgrid_name_list(
                grid::GridAxis::Column,
                3,
                vec![
                    grid::SubgridNameComponent::Repeat {
                        count: grid::SubgridNameRepeatCount::AutoFill,
                        line_name_sets: vec![vec!["a".to_owned()]],
                    },
                    grid::SubgridNameComponent::Repeat {
                        count: grid::SubgridNameRepeatCount::AutoFill,
                        line_name_sets: vec![vec!["b".to_owned()]],
                    },
                ],
            )
            .unwrap_err(),
            grid::NamedGridError::MultipleAutoFillRepeats
        );
    }

    #[test]
    fn oracle_subgrid_line_names_merge_parent_and_local_names() {
        let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["c"], vec!["d"]]);
        let report = grid::inherit_named_subgrid_lines(
            &parent,
            grid::TrackSpan::new(2, 5),
            false,
            vec![
                vec!["local-start".to_owned()],
                vec![],
                vec!["middle".to_owned()],
                vec!["local-end".to_owned()],
            ],
            None,
        )
        .unwrap();

        assert_eq!(report.lines.line_names(1), vec!["b", "local-start"]);
        assert_eq!(report.lines.line_names(3), vec!["c", "middle"]);
        assert_eq!(report.lines.line_names(4), vec!["d", "local-end"]);
        assert_eq!(
            report.local_line_names.line_names[0][0].origin,
            grid::LineNameOrigin::LocalSubgrid
        );
    }

    #[test]
    fn oracle_subgrid_line_names_reverse_parent_line_order_when_axis_is_reversed() {
        let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["c"], vec!["d"]]);
        let report = grid::inherit_named_subgrid_lines(
            &parent,
            grid::TrackSpan::new(2, 5),
            true,
            vec![vec![], vec![], vec![], vec![]],
            None,
        )
        .unwrap();

        assert_eq!(report.lines.line_names(1), vec!["d"]);
        assert_eq!(report.lines.line_names(2), vec!["c"]);
        assert_eq!(report.lines.line_names(4), vec!["b"]);
    }

    #[test]
    fn oracle_subgrid_recomputes_area_generated_names_from_clipped_parent_areas() {
        let parent_areas = grid::TemplateAreas::new([vec!["a", "a", "a", "a"]]).unwrap();
        let parent_facts = grid::area_generated_facts(
            &parent_areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 4),
            grid::NamedGridLines::empty(grid::GridAxis::Row, 1),
        )
        .unwrap();
        let parent = parent_facts.columns.clone();

        let report = grid::inherit_named_subgrid_lines(
            &parent,
            grid::TrackSpan::new(2, 4),
            false,
            vec![vec![], vec![], vec![]],
            Some(&parent_facts),
        )
        .unwrap();

        assert_eq!(
            report.clipped_area_sources["a"].parent_span,
            grid::TrackSpan::new(2, 4)
        );
        assert_eq!(report.lines.line_names(1), vec!["a-start"]);
        assert_eq!(report.lines.line_names(3), vec!["a-end"]);
    }

    #[test]
    fn oracle_subgrid_reversed_area_generated_names_follow_parent_boundaries() {
        let parent_areas = grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
        let parent_facts = grid::area_generated_facts(
            &parent_areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 2),
            grid::NamedGridLines::empty(grid::GridAxis::Row, 1),
        )
        .unwrap();

        let report = grid::inherit_named_subgrid_lines(
            &parent_facts.columns,
            grid::TrackSpan::new(1, 3),
            true,
            vec![vec![], vec![], vec![]],
            Some(&parent_facts),
        )
        .unwrap();

        assert_eq!(report.lines.line_names(1), vec!["a-end"]);
        assert_eq!(report.lines.line_names(3), vec!["a-start"]);
    }

    #[test]
    fn oracle_subgrid_line_names_ignore_parent_area_generated_names_until_recomputed() {
        let parent_areas = grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
        let parent_facts = grid::area_generated_facts(
            &parent_areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 2),
            grid::NamedGridLines::empty(grid::GridAxis::Row, 1),
        )
        .unwrap();

        let report = grid::inherit_named_subgrid_lines(
            &parent_facts.columns,
            grid::TrackSpan::new(1, 3),
            false,
            vec![vec![], vec![], vec![]],
            None,
        )
        .unwrap();

        assert!(report.lines.line_names(1).is_empty());
        assert!(report.lines.line_names(3).is_empty());
    }

    #[test]
    fn oracle_subgrid_line_names_order_area_generated_before_local_names() {
        let parent_areas = grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
        let parent_facts = grid::area_generated_facts(
            &parent_areas,
            grid::NamedGridLines::empty(grid::GridAxis::Column, 2),
            grid::NamedGridLines::empty(grid::GridAxis::Row, 1),
        )
        .unwrap();

        let report = grid::inherit_named_subgrid_lines(
            &parent_facts.columns,
            grid::TrackSpan::new(1, 3),
            false,
            vec![
                vec!["local-start".to_owned()],
                vec![],
                vec!["local-end".to_owned()],
            ],
            Some(&parent_facts),
        )
        .unwrap();

        assert_eq!(report.lines.line_names(1), vec!["a-start", "local-start"]);
        assert_eq!(report.lines.line_names(3), vec!["a-end", "local-end"]);
    }

    #[test]
    fn oracle_subgrid_named_placement_clamps_to_subgrid_explicit_lines() {
        let subgrid = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
        let report = grid::resolve_named_subgrid_axis_placement(
            &subgrid,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Named {
                    name: "a".to_owned(),
                    occurrence: -3,
                },
                end: grid::NamedGridLine::Named {
                    name: "a".to_owned(),
                    occurrence: 4,
                },
            },
            None,
        )
        .unwrap();

        assert_eq!(report.unclamped_start_line, 0);
        assert_eq!(report.unclamped_end_line, 5);
        assert_eq!(report.clamped.resolved.start_line, 1);
        assert_eq!(report.clamped.resolved.end_line, 3);
    }

    #[test]
    fn oracle_subgrid_named_placement_expands_collapsed_clamp_to_edge_track() {
        let subgrid = grid::NamedGridLines::empty(grid::GridAxis::Column, 1);
        let report = grid::resolve_named_subgrid_axis_placement(
            &subgrid,
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Number(2),
                end: grid::NamedGridLine::Span {
                    name: None,
                    count: 3,
                },
            },
            None,
        )
        .unwrap();

        assert_eq!(report.unclamped_start_line, 2);
        assert_eq!(report.unclamped_end_line, 5);
        assert_eq!(report.clamped.resolved.start_line, 1);
        assert_eq!(report.clamped.resolved.end_line, 2);
    }

    #[test]
    fn oracle_subgrid_eligibility_accepts_requested_axis_with_parent_grid() {
        let report = grid::subgrid_eligibility(grid::SubgridEligibilityInput {
            requested: true,
            has_parent_grid: true,
            independent_formatting_context: false,
            excluded_from_normal_layout: false,
            parent_is_lanes_in_resolved_axis: false,
        });

        assert!(report.eligible);
        assert_eq!(report.reason, None);
    }

    #[test]
    fn oracle_subgrid_eligibility_rejects_lanes_parent_in_resolved_axis() {
        let report = grid::subgrid_eligibility(grid::SubgridEligibilityInput {
            requested: true,
            has_parent_grid: true,
            independent_formatting_context: false,
            excluded_from_normal_layout: false,
            parent_is_lanes_in_resolved_axis: true,
        });

        assert!(!report.eligible);
        assert_eq!(
            report.reason,
            Some(grid::SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
        );
    }

    #[test]
    fn oracle_subgrid_eligibility_reports_first_blocking_reason() {
        let report = grid::subgrid_eligibility(grid::SubgridEligibilityInput {
            requested: false,
            has_parent_grid: false,
            independent_formatting_context: true,
            excluded_from_normal_layout: true,
            parent_is_lanes_in_resolved_axis: true,
        });

        assert!(!report.eligible);
        assert_eq!(
            report.reason,
            Some(grid::SubgridIneligibleReason::NotRequested)
        );
    }

    #[test]
    fn oracle_subgrid_eligibility_reports_each_blocking_reason() {
        let cases = [
            (
                grid::SubgridEligibilityInput {
                    requested: true,
                    has_parent_grid: false,
                    independent_formatting_context: false,
                    excluded_from_normal_layout: false,
                    parent_is_lanes_in_resolved_axis: false,
                },
                grid::SubgridIneligibleReason::NoParentGrid,
            ),
            (
                grid::SubgridEligibilityInput {
                    requested: true,
                    has_parent_grid: true,
                    independent_formatting_context: false,
                    excluded_from_normal_layout: true,
                    parent_is_lanes_in_resolved_axis: false,
                },
                grid::SubgridIneligibleReason::ExcludedFromNormalLayout,
            ),
        ];

        for (input, reason) in cases {
            assert_eq!(grid::subgrid_eligibility(input).reason, Some(reason));
        }
    }

    #[test]
    fn oracle_subgrid_copies_parent_tracks_for_span() {
        let report = grid::inherit_subgrid_tracks(grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 60.0, 90.0],
            parent_span: grid::TrackSpan::new(2, 4),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: grid::OracleGapReport::length(10.0),
            subgrid_gap: grid::OracleGapReport::length(10.0),
        })
        .unwrap();

        assert_eq!(report.copied_parent_tracks, vec![60.0, 90.0]);
        assert_eq!(report.final_tracks, vec![60.0, 90.0]);
    }

    #[test]
    fn oracle_subgrid_reverses_copied_tracks_before_mbp_removal() {
        let report = grid::inherit_subgrid_tracks(grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 60.0, 90.0],
            parent_span: grid::TrackSpan::new(1, 4),
            reversed: true,
            start_mbp: 10.0,
            end_mbp: 20.0,
            parent_gap: grid::OracleGapReport::length(10.0),
            subgrid_gap: grid::OracleGapReport::length(10.0),
        })
        .unwrap();

        assert_eq!(report.after_reversal, vec![90.0, 60.0, 40.0]);
        assert_eq!(report.final_tracks, vec![80.0, 60.0, 20.0]);
    }

    #[test]
    fn oracle_subgrid_resolves_normal_gap_to_parent_gap() {
        let report = grid::inherit_subgrid_tracks(grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![50.0, 50.0],
            parent_span: grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: grid::OracleGapReport::length(20.0),
            subgrid_gap: grid::OracleGapReport::normal_resolved_to(20.0),
        })
        .unwrap();

        assert_eq!(report.gap_difference, 0.0);
        assert_eq!(report.final_tracks, vec![50.0, 50.0]);
    }

    #[test]
    fn oracle_subgrid_baselines_slice_parent_groups_for_span() {
        let report = grid::inherit_subgrid_baselines(grid::SubgridBaselineInheritanceInput {
            parent_span: grid::TrackSpan::new(2, 4),
            reversed: false,
            parent_gap: grid::OracleGapReport::normal_resolved_to(10.0),
            subgrid_gap: grid::OracleGapReport::length(10.0),
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_major: vec![Some(4.0), Some(8.0), None, Some(6.0)],
            parent_minor: vec![None, Some(5.0), Some(7.0), None],
        })
        .unwrap();

        assert_eq!(report.sliced_major, vec![Some(8.0), None]);
        assert_eq!(report.sliced_minor, vec![Some(5.0), Some(7.0)]);
        assert_eq!(report.after_reversal_major, vec![Some(8.0), None]);
        assert_eq!(report.after_reversal_minor, vec![Some(5.0), Some(7.0)]);
        assert_eq!(report.after_mbp_major, vec![Some(8.0), None]);
        assert_eq!(report.after_mbp_minor, vec![Some(5.0), Some(7.0)]);
        assert_eq!(report.final_major, vec![Some(8.0), None]);
        assert_eq!(report.final_minor, vec![Some(5.0), Some(7.0)]);
    }

    #[test]
    fn oracle_subgrid_baselines_reverse_and_adjust_edges() {
        let report = grid::inherit_subgrid_baselines(grid::SubgridBaselineInheritanceInput {
            parent_span: grid::TrackSpan::new(1, 3),
            reversed: true,
            parent_gap: grid::OracleGapReport::normal_resolved_to(10.0),
            subgrid_gap: grid::OracleGapReport::length(20.0),
            start_mbp: 3.0,
            end_mbp: 5.0,
            parent_major: vec![Some(10.0), Some(14.0)],
            parent_minor: vec![Some(4.0), Some(8.0)],
        })
        .unwrap();

        assert_eq!(report.final_major.len(), 2);
        assert_eq!(report.final_minor.len(), 2);
        assert!(report.reversed);
        assert_eq!(report.start_mbp, 3.0);
        assert_eq!(report.end_mbp, 5.0);
        assert_eq!(report.gap_difference, 5.0);
        assert_eq!(report.sliced_major, vec![Some(10.0), Some(14.0)]);
        assert_eq!(report.sliced_minor, vec![Some(4.0), Some(8.0)]);
        assert_eq!(report.after_reversal_major, vec![Some(14.0), Some(10.0)]);
        assert_eq!(report.after_reversal_minor, vec![Some(8.0), Some(4.0)]);
        assert_eq!(report.after_mbp_major, vec![Some(17.0), Some(10.0)]);
        assert_eq!(report.after_mbp_minor, vec![Some(8.0), Some(9.0)]);
        assert_eq!(report.final_major, vec![Some(12.0), Some(5.0)]);
        assert_eq!(report.final_minor, vec![Some(3.0), Some(4.0)]);
    }

    #[test]
    fn oracle_subgrid_baselines_reject_invalid_spans_and_group_shapes() {
        let base = grid::SubgridBaselineInheritanceInput {
            parent_span: grid::TrackSpan::new(1, 3),
            reversed: false,
            parent_gap: grid::OracleGapReport::length(10.0),
            subgrid_gap: grid::OracleGapReport::length(10.0),
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_major: vec![Some(1.0), Some(2.0)],
            parent_minor: vec![Some(3.0), Some(4.0)],
        };

        let cases = [
            grid::SubgridBaselineInheritanceInput {
                parent_span: grid::TrackSpan::new(0, 1),
                ..base.clone()
            },
            grid::SubgridBaselineInheritanceInput {
                parent_span: grid::TrackSpan::new(2, 2),
                ..base.clone()
            },
            grid::SubgridBaselineInheritanceInput {
                parent_span: grid::TrackSpan::new(1, 4),
                ..base.clone()
            },
            grid::SubgridBaselineInheritanceInput {
                parent_minor: vec![Some(3.0)],
                ..base
            },
        ];

        for input in cases {
            assert!(grid::inherit_subgrid_baselines(input).is_err());
        }
    }

    #[test]
    fn oracle_subgrid_baselines_preserve_none_through_mbp_and_gap_adjustment() {
        let report = grid::inherit_subgrid_baselines(grid::SubgridBaselineInheritanceInput {
            parent_span: grid::TrackSpan::new(1, 3),
            reversed: false,
            parent_gap: grid::OracleGapReport::length(10.0),
            subgrid_gap: grid::OracleGapReport::length(20.0),
            start_mbp: 3.0,
            end_mbp: 5.0,
            parent_major: vec![None, Some(14.0)],
            parent_minor: vec![Some(12.0), None],
        })
        .unwrap();

        assert_eq!(report.gap_difference, 5.0);
        assert_eq!(report.after_mbp_major, vec![None, Some(14.0)]);
        assert_eq!(report.after_mbp_minor, vec![Some(12.0), None]);
        assert_eq!(report.final_major, vec![None, Some(9.0)]);
        assert_eq!(report.final_minor, vec![Some(7.0), None]);
    }

    #[test]
    fn oracle_subgrid_baselines_adjust_each_internal_gap_edge() {
        let report = grid::inherit_subgrid_baselines(grid::SubgridBaselineInheritanceInput {
            parent_span: grid::TrackSpan::new(1, 4),
            reversed: false,
            parent_gap: grid::OracleGapReport::length(10.0),
            subgrid_gap: grid::OracleGapReport::length(20.0),
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_major: vec![Some(20.0), Some(30.0), Some(40.0)],
            parent_minor: vec![Some(10.0), Some(20.0), Some(30.0)],
        })
        .unwrap();

        assert_eq!(report.gap_difference, 5.0);
        assert_eq!(report.final_major, vec![Some(15.0), Some(20.0), Some(35.0)]);
        assert_eq!(report.final_minor, vec![Some(5.0), Some(10.0), Some(25.0)]);
    }

    #[test]
    fn oracle_subgrid_baselines_apply_signed_gap_differences() {
        let cases = [
            (
                grid::OracleGapReport::length(10.0),
                -5.0,
                vec![Some(18.0), Some(25.0)],
                vec![Some(10.0), Some(25.0)],
            ),
            (
                grid::OracleGapReport::length(20.0),
                0.0,
                vec![Some(13.0), Some(20.0)],
                vec![Some(5.0), Some(20.0)],
            ),
        ];

        for (subgrid_gap, gap_difference, final_major, final_minor) in cases {
            let report = grid::inherit_subgrid_baselines(grid::SubgridBaselineInheritanceInput {
                parent_span: grid::TrackSpan::new(1, 3),
                reversed: false,
                parent_gap: grid::OracleGapReport::length(20.0),
                subgrid_gap,
                start_mbp: 3.0,
                end_mbp: 5.0,
                parent_major: vec![Some(10.0), Some(20.0)],
                parent_minor: vec![Some(5.0), Some(15.0)],
            })
            .unwrap();

            assert_eq!(report.gap_difference, gap_difference);
            assert_eq!(report.after_mbp_major, vec![Some(13.0), Some(20.0)]);
            assert_eq!(report.after_mbp_minor, vec![Some(5.0), Some(20.0)]);
            assert_eq!(report.final_major, final_major);
            assert_eq!(report.final_minor, final_minor);
        }
    }

    #[test]
    fn oracle_subgrid_publishes_descendant_baseline_to_ancestor_track() {
        let report = grid::publish_subgrid_baseline(grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: grid::TrackSpan::new(2, 4),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        })
        .unwrap();

        assert_eq!(report.ancestor_track, Some(2));
        assert_eq!(report.group, Some(grid::BaselineGroupKind::Major));
        assert_eq!(report.baseline, Some(75.0));
    }

    #[test]
    fn oracle_subgrid_publishes_reversed_descendant_baseline_to_ancestor_track() {
        let report = grid::publish_subgrid_baseline(grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: true,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: grid::BaselineGroupKind::Minor,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        })
        .unwrap();

        assert_eq!(report.ancestor_track, Some(4));
        assert_eq!(report.group, Some(grid::BaselineGroupKind::Minor));
        assert_eq!(report.baseline, Some(75.0));
    }

    #[test]
    fn oracle_subgrid_publishes_last_local_track_to_ancestor_track() {
        let report = grid::publish_subgrid_baseline(grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 3,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        })
        .unwrap();

        assert_eq!(report.ancestor_track, Some(4));
        assert_eq!(report.group, Some(grid::BaselineGroupKind::Major));
        assert_eq!(report.baseline, Some(75.0));
    }

    #[test]
    fn oracle_subgrid_publishes_reversed_last_local_track_to_ancestor_track() {
        let report = grid::publish_subgrid_baseline(grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: true,
            descendant_local_track: 3,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: grid::BaselineGroupKind::Minor,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        })
        .unwrap();

        assert_eq!(report.ancestor_track, Some(2));
        assert_eq!(report.group, Some(grid::BaselineGroupKind::Minor));
        assert_eq!(report.baseline, Some(75.0));
    }

    #[test]
    fn oracle_subgrid_does_not_publish_synthesized_cycle_fallback() {
        let report = grid::publish_subgrid_baseline(grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: grid::TrackSpan::new(2, 4),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: true,
        })
        .unwrap();

        assert!(!report.published);
        assert_eq!(report.ancestor_track, None);
        assert_eq!(report.group, None);
        assert_eq!(report.baseline, None);
    }

    #[test]
    fn oracle_subgrid_publish_rejects_zero_local_track() {
        let error = grid::publish_subgrid_baseline(grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 0,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        })
        .unwrap_err();

        assert_eq!(error, grid::OracleGridError::SpanOutOfRange);
    }

    #[test]
    fn oracle_subgrid_publish_rejects_local_track_beyond_span() {
        let error = grid::publish_subgrid_baseline(grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 4,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        })
        .unwrap_err();

        assert_eq!(error, grid::OracleGridError::SpanOutOfRange);
    }

    #[test]
    fn oracle_subgrid_applies_gap_difference_to_internal_edges() {
        let report = grid::inherit_subgrid_tracks(grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![50.0, 50.0, 50.0],
            parent_span: grid::TrackSpan::new(1, 4),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: grid::OracleGapReport::length(10.0),
            subgrid_gap: grid::OracleGapReport::length(20.0),
        })
        .unwrap();

        assert_eq!(report.gap_difference, 5.0);
        assert_eq!(report.final_tracks, vec![45.0, 40.0, 45.0]);
    }

    #[test]
    fn oracle_subgrid_adds_negative_gap_difference_to_internal_edges() {
        let report = grid::inherit_subgrid_tracks(grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 40.0],
            parent_span: grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: grid::OracleGapReport::length(20.0),
            subgrid_gap: grid::OracleGapReport::length(10.0),
        })
        .unwrap();

        assert_eq!(report.gap_difference, -5.0);
        assert_eq!(report.final_tracks, vec![45.0, 45.0]);
    }

    #[test]
    fn oracle_subgrid_mbp_removal_clamps_tracks_to_zero() {
        let report = grid::inherit_subgrid_tracks(grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![5.0, 10.0],
            parent_span: grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 20.0,
            end_mbp: 20.0,
            parent_gap: grid::OracleGapReport::length(0.0),
            subgrid_gap: grid::OracleGapReport::length(0.0),
        })
        .unwrap();

        assert_eq!(report.final_tracks, vec![0.0, 0.0]);
    }

    #[test]
    fn oracle_subgrid_mbp_removal_consumes_across_tracks() {
        let report = grid::inherit_subgrid_tracks(grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![5.0, 10.0, 20.0],
            parent_span: grid::TrackSpan::new(1, 4),
            reversed: false,
            start_mbp: 12.0,
            end_mbp: 0.0,
            parent_gap: grid::OracleGapReport::length(0.0),
            subgrid_gap: grid::OracleGapReport::length(0.0),
        })
        .unwrap();

        assert_eq!(report.start_mbp_removed, vec![0.0, 3.0, 20.0]);
        assert_eq!(report.end_mbp_removed, vec![0.0, 3.0, 20.0]);
        assert_eq!(report.final_tracks, vec![0.0, 3.0, 20.0]);
    }

    fn oracle_subgrid_leaf(id: &'static str, start: usize, end: usize) -> grid::SubgridChild {
        grid::SubgridChild::Leaf(grid::SubgridLeaf {
            id,
            span_in_parent: grid::TrackSpan::new(start, end),
            contribution: oracle_lane_facts(20.0, 40.0),
        })
    }

    fn oracle_subgrid_node(
        id: &'static str,
        start: usize,
        end: usize,
        children: Vec<grid::SubgridChild>,
    ) -> grid::SubgridChild {
        grid::SubgridChild::Subgrid(grid::SubgridNode {
            id,
            axis: grid::SubgridAxisKind::Inherited,
            reversed: false,
            span_in_parent: grid::TrackSpan::new(start, end),
            margins: grid::AxisEdges::default(),
            border: grid::AxisEdges::default(),
            padding: grid::AxisEdges::default(),
            parent_gap: grid::OracleGapReport::length(0.0),
            subgrid_gap: grid::OracleGapReport::length(0.0),
            children,
        })
    }

    #[test]
    fn oracle_subgrid_traversal_collects_direct_leaf() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
        })
        .unwrap();

        assert_eq!(report.leaves.len(), 1);
        assert_eq!(report.leaves[0].ancestor_span, grid::TrackSpan::new(1, 2));
    }

    #[test]
    fn oracle_subgrid_traversal_accumulates_edge_mbp_for_intrinsic_tracks() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "sub",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: false,
                span_in_parent: grid::TrackSpan::new(1, 3),
                margins: grid::AxisEdges {
                    start: 3.0,
                    end: 4.0,
                },
                border: grid::AxisEdges {
                    start: 5.0,
                    end: 6.0,
                },
                padding: grid::AxisEdges {
                    start: 7.0,
                    end: 8.0,
                },
                parent_gap: grid::OracleGapReport::length(10.0),
                subgrid_gap: grid::OracleGapReport::length(10.0),
                children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
            })],
        })
        .unwrap();

        assert_eq!(report.edge_lower_bounds, vec![15.0, 18.0]);
        assert_eq!(
            report.leaves[0].accumulated_edge_adjustment,
            vec![15.0, 18.0]
        );
    }

    #[test]
    fn oracle_subgrid_traversal_swaps_edge_mbp_for_reversed_subgrid() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "sub",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: true,
                span_in_parent: grid::TrackSpan::new(1, 3),
                margins: grid::AxisEdges {
                    start: 3.0,
                    end: 4.0,
                },
                border: grid::AxisEdges {
                    start: 5.0,
                    end: 6.0,
                },
                padding: grid::AxisEdges {
                    start: 7.0,
                    end: 8.0,
                },
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
            })],
        })
        .unwrap();

        assert_eq!(report.edge_lower_bounds, vec![18.0, 15.0]);
        assert_eq!(
            report.leaves[0].accumulated_edge_adjustment,
            vec![18.0, 15.0]
        );
    }

    #[test]
    fn oracle_subgrid_traversal_accumulates_interior_edge_mbp_by_track() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "sub",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: false,
                span_in_parent: grid::TrackSpan::new(2, 4),
                margins: grid::AxisEdges {
                    start: 2.0,
                    end: 3.0,
                },
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
            })],
        })
        .unwrap();

        assert_eq!(report.edge_lower_bounds, vec![0.0, 2.0, 3.0, 0.0]);
        assert_eq!(
            report.leaves[0].accumulated_edge_adjustment,
            vec![0.0, 2.0, 3.0, 0.0]
        );
    }

    #[test]
    fn oracle_subgrid_traversal_translates_leaf_span_through_child_subgrid() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![oracle_subgrid_node(
                "sub",
                2,
                4,
                vec![oracle_subgrid_leaf("leaf", 2, 3)],
            )],
        })
        .unwrap();

        assert_eq!(report.leaves[0].ancestor_span, grid::TrackSpan::new(3, 4));
    }

    #[test]
    fn oracle_subgrid_traversal_translates_reversed_leaf_span_from_end_edge() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "sub",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: true,
                span_in_parent: grid::TrackSpan::new(2, 5),
                margins: grid::AxisEdges::default(),
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
            })],
        })
        .unwrap();

        assert_eq!(report.leaves[0].ancestor_span, grid::TrackSpan::new(4, 5));
    }

    #[test]
    fn oracle_subgrid_traversal_preserves_reversed_orientation_through_nested_subgrid() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true, true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "outer",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: true,
                span_in_parent: grid::TrackSpan::new(2, 6),
                margins: grid::AxisEdges::default(),
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                    id: "inner",
                    axis: grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: grid::TrackSpan::new(1, 3),
                    margins: grid::AxisEdges::default(),
                    border: grid::AxisEdges::default(),
                    padding: grid::AxisEdges::default(),
                    parent_gap: grid::OracleGapReport::length(0.0),
                    subgrid_gap: grid::OracleGapReport::length(0.0),
                    children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                })],
            })],
        })
        .unwrap();

        assert_eq!(report.leaves[0].ancestor_span, grid::TrackSpan::new(5, 6));
    }

    #[test]
    fn oracle_subgrid_traversal_accumulates_gap_differences() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "sub",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: false,
                span_in_parent: grid::TrackSpan::new(1, 3),
                margins: grid::AxisEdges::default(),
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(10.0),
                subgrid_gap: grid::OracleGapReport::length(20.0),
                children: vec![oracle_subgrid_leaf("leaf", 2, 3)],
            })],
        })
        .unwrap();

        assert_eq!(
            report.leaves[0].accumulated_gap_adjustment,
            vec![5.0, 5.0, 0.0]
        );
    }

    #[test]
    fn oracle_subgrid_traversal_skips_edge_mbp_for_non_intrinsic_tracks() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![false, false],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "sub",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: false,
                span_in_parent: grid::TrackSpan::new(1, 3),
                margins: grid::AxisEdges {
                    start: 10.0,
                    end: 10.0,
                },
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: Vec::new(),
            })],
        })
        .unwrap();

        assert_eq!(report.edge_lower_bounds, vec![0.0, 0.0]);
    }

    #[test]
    fn oracle_subgrid_traversal_requires_intrinsic_facts_for_edge_mbp() {
        let err = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "sub",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: false,
                span_in_parent: grid::TrackSpan::new(1, 2),
                margins: grid::AxisEdges {
                    start: 1.0,
                    end: 1.0,
                },
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: Vec::new(),
            })],
        })
        .unwrap_err();

        assert_eq!(err, grid::OracleGridError::MissingIntrinsicMinTrackFacts);
    }

    #[test]
    fn oracle_subgrid_traversal_keeps_standalone_as_one_boundary_leaf() {
        let contribution = oracle_lane_facts(30.0, 70.0);
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "root",
                axis: grid::SubgridAxisKind::Standalone(contribution),
                reversed: false,
                span_in_parent: grid::TrackSpan::new(1, 2),
                margins: grid::AxisEdges::default(),
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: Vec::new(),
            })],
        })
        .unwrap();

        assert_eq!(report.leaves.len(), 1);
        assert_eq!(report.leaves[0].id, "root");
        assert_eq!(report.leaves[0].contribution, contribution);
    }

    #[test]
    fn oracle_subgrid_traversal_rejects_invalid_leaf_span() {
        let err = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![oracle_subgrid_leaf("bad", 2, 2)],
        })
        .unwrap_err();

        assert_eq!(err, grid::OracleGridError::SpanOutOfRange);
    }

    #[test]
    fn oracle_subgrid_traversal_supports_mixed_root_children() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![
                oracle_subgrid_leaf("direct", 1, 2),
                oracle_subgrid_node("sub", 2, 4, vec![oracle_subgrid_leaf("nested", 1, 2)]),
            ],
        })
        .unwrap();

        assert_eq!(
            report
                .leaves
                .iter()
                .map(|leaf| (leaf.id, leaf.ancestor_span))
                .collect::<Vec<_>>(),
            vec![
                ("direct", grid::TrackSpan::new(1, 2)),
                ("nested", grid::TrackSpan::new(2, 3)),
            ]
        );
    }

    #[test]
    fn oracle_subgrid_traversal_accumulates_nested_edge_adjustments() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "outer",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: false,
                span_in_parent: grid::TrackSpan::new(1, 4),
                margins: grid::AxisEdges {
                    start: 2.0,
                    end: 4.0,
                },
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                    id: "inner",
                    axis: grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: grid::TrackSpan::new(2, 3),
                    margins: grid::AxisEdges {
                        start: 3.0,
                        end: 5.0,
                    },
                    border: grid::AxisEdges::default(),
                    padding: grid::AxisEdges::default(),
                    parent_gap: grid::OracleGapReport::length(0.0),
                    subgrid_gap: grid::OracleGapReport::length(0.0),
                    children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                })],
            })],
        })
        .unwrap();

        assert_eq!(report.edge_lower_bounds, vec![2.0, 8.0, 4.0]);
        assert_eq!(
            report.leaves[0].accumulated_edge_adjustment,
            vec![2.0, 8.0, 4.0]
        );
    }

    #[test]
    fn oracle_subgrid_traversal_translates_nested_edge_adjustments_to_ancestor_tracks() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "outer",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: false,
                span_in_parent: grid::TrackSpan::new(2, 5),
                margins: grid::AxisEdges::default(),
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(0.0),
                subgrid_gap: grid::OracleGapReport::length(0.0),
                children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                    id: "inner",
                    axis: grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: grid::TrackSpan::new(2, 3),
                    margins: grid::AxisEdges {
                        start: 3.0,
                        end: 5.0,
                    },
                    border: grid::AxisEdges::default(),
                    padding: grid::AxisEdges::default(),
                    parent_gap: grid::OracleGapReport::length(0.0),
                    subgrid_gap: grid::OracleGapReport::length(0.0),
                    children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                })],
            })],
        })
        .unwrap();

        assert_eq!(report.edge_lower_bounds, vec![0.0, 0.0, 8.0, 0.0]);
        assert_eq!(
            report.leaves[0].accumulated_edge_adjustment,
            vec![0.0, 0.0, 8.0, 0.0]
        );
    }

    #[test]
    fn oracle_subgrid_traversal_applies_full_span_internal_gap() {
        let report = grid::traverse_subgrid_intrinsic(grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true],
            root_children: vec![grid::SubgridChild::Subgrid(grid::SubgridNode {
                id: "sub",
                axis: grid::SubgridAxisKind::Inherited,
                reversed: false,
                span_in_parent: grid::TrackSpan::new(1, 3),
                margins: grid::AxisEdges::default(),
                border: grid::AxisEdges::default(),
                padding: grid::AxisEdges::default(),
                parent_gap: grid::OracleGapReport::length(10.0),
                subgrid_gap: grid::OracleGapReport::length(20.0),
                children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
            })],
        })
        .unwrap();

        assert_eq!(report.leaves[0].accumulated_gap_adjustment, vec![5.0, 5.0]);
    }

    #[test]
    fn oracle_grid_lanes_disables_row_axis_item_baseline_offsets() {
        let report = grid::grid_lanes_baseline_policy(grid::GridLanesBaselineInput {
            auto_flow: grid::LaneAutoFlow::Row,
            queried_axis: grid::GridAxis::Row,
            requested_alignment: grid::BaselineAlignment::First,
            has_items: true,
        });

        assert!(!report.applies_item_offsets);
        assert_eq!(
            report.reason,
            Some(grid::GridLanesBaselineReason::WebKitMasonryFallback)
        );
    }

    #[test]
    fn oracle_grid_lanes_disables_column_axis_item_baseline_offsets() {
        let report = grid::grid_lanes_baseline_policy(grid::GridLanesBaselineInput {
            auto_flow: grid::LaneAutoFlow::Column,
            queried_axis: grid::GridAxis::Column,
            requested_alignment: grid::BaselineAlignment::Last,
            has_items: true,
        });

        assert!(!report.applies_item_offsets);
        assert_eq!(
            report.reason,
            Some(grid::GridLanesBaselineReason::WebKitMasonryFallback)
        );
    }

    #[test]
    fn oracle_grid_lanes_disables_item_baseline_offsets_for_all_axis_combinations() {
        let cases = [
            (grid::LaneAutoFlow::Row, grid::GridAxis::Row),
            (grid::LaneAutoFlow::Row, grid::GridAxis::Column),
            (grid::LaneAutoFlow::Column, grid::GridAxis::Row),
            (grid::LaneAutoFlow::Column, grid::GridAxis::Column),
        ];

        for (auto_flow, queried_axis) in cases {
            let report = grid::grid_lanes_baseline_policy(grid::GridLanesBaselineInput {
                auto_flow,
                queried_axis,
                requested_alignment: grid::BaselineAlignment::First,
                has_items: true,
            });

            assert!(!report.applies_item_offsets);
            assert_eq!(
                report.reason,
                Some(grid::GridLanesBaselineReason::WebKitMasonryFallback)
            );
        }
    }

    #[test]
    fn oracle_grid_lanes_can_synthesize_container_baselines_from_geometry() {
        let report = grid::grid_lanes_container_baselines(vec![
            grid::ContainerBaselineFallbackItem {
                id: "a",
                area: grid::GridArea::new(1, 1, 1, 1),
                block_offset: 0.0,
                first_baseline: 20.0,
                last_baseline: 0.0,
            },
            grid::ContainerBaselineFallbackItem {
                id: "b",
                area: grid::GridArea::new(2, 1, 1, 1),
                block_offset: 30.0,
                first_baseline: 30.0,
                last_baseline: 0.0,
            },
        ]);

        assert_eq!(report.first, Some(20.0));
        assert_eq!(report.last, Some(30.0));
    }

    #[test]
    fn oracle_grid_lanes_container_baselines_use_final_geometry_offsets() {
        let report = grid::grid_lanes_container_baselines(vec![
            grid::ContainerBaselineFallbackItem {
                id: "first",
                area: grid::GridArea::new(1, 1, 1, 1),
                block_offset: 12.0,
                first_baseline: 7.0,
                last_baseline: 0.0,
            },
            grid::ContainerBaselineFallbackItem {
                id: "middle",
                area: grid::GridArea::new(2, 1, 1, 1),
                block_offset: 30.0,
                first_baseline: 5.0,
                last_baseline: 8.0,
            },
            grid::ContainerBaselineFallbackItem {
                id: "last",
                area: grid::GridArea::new(3, 1, 1, 1),
                block_offset: 44.0,
                first_baseline: 3.0,
                last_baseline: 11.0,
            },
        ]);

        assert_eq!(report.first, Some(19.0));
        assert_eq!(report.last, Some(55.0));
    }

    #[test]
    fn oracle_grid_lanes_container_baselines_last_uses_spanned_end_edge() {
        let report = grid::grid_lanes_container_baselines(vec![
            grid::ContainerBaselineFallbackItem {
                id: "starts-later",
                area: grid::GridArea::new(1, 2, 1, 1),
                block_offset: 40.0,
                first_baseline: 8.0,
                last_baseline: 14.0,
            },
            grid::ContainerBaselineFallbackItem {
                id: "spans-to-last-row",
                area: grid::GridArea::new(2, 1, 1, 3),
                block_offset: 5.0,
                first_baseline: 3.0,
                last_baseline: 91.0,
            },
        ]);

        assert_eq!(report.first, Some(8.0));
        assert_eq!(report.last, Some(96.0));
    }

    #[test]
    fn oracle_grid_lanes_container_baselines_return_none_for_empty_input() {
        let report = grid::grid_lanes_container_baselines(Vec::new());

        assert_eq!(report.first, None);
        assert_eq!(report.last, None);
    }

    #[test]
    fn oracle_grid_lanes_baseline_policy_reports_no_items() {
        let report = grid::grid_lanes_baseline_policy(grid::GridLanesBaselineInput {
            auto_flow: grid::LaneAutoFlow::Row,
            queried_axis: grid::GridAxis::Row,
            requested_alignment: grid::BaselineAlignment::First,
            has_items: false,
        });

        assert!(!report.applies_item_offsets);
        assert_eq!(report.reason, Some(grid::GridLanesBaselineReason::NoItems));
    }

    #[test]
    fn oracle_grid_lanes_baseline_policy_reports_no_baseline_alignment_requested() {
        let report = grid::grid_lanes_baseline_policy(grid::GridLanesBaselineInput {
            auto_flow: grid::LaneAutoFlow::Column,
            queried_axis: grid::GridAxis::Column,
            requested_alignment: grid::BaselineAlignment::None,
            has_items: true,
        });

        assert!(!report.applies_item_offsets);
        assert_eq!(
            report.reason,
            Some(grid::GridLanesBaselineReason::NoBaselineAlignmentRequested)
        );
    }

    #[test]
    fn oracle_lanes_row_auto_flow_makes_rows_the_lane_axis() {
        assert_eq!(grid::lane_axis(grid::LaneAutoFlow::Row), GridAxis::Row);
        assert_eq!(
            grid::grid_axis_for_lanes(grid::LaneAutoFlow::Row),
            GridAxis::Column
        );
    }

    #[test]
    fn oracle_lanes_place_definite_and_indefinite_items_with_fixed_tolerance() {
        let report = grid::place_lanes(grid::LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: grid::LaneAutoFlow::Row,
            lane_gap: 10.0,
            tolerance: grid::LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                grid::LaneItemInput::definite("a", 1, 2, 40.0),
                grid::LaneItemInput::auto("b", 1, 20.0),
                grid::LaneItemInput::auto("c", 2, 30.0),
            ],
        })
        .unwrap();

        assert_eq!(report.item_offsets[0].offset, 0.0);
        assert_eq!(report.item_offsets[1].offset, 0.0);
        assert_eq!(report.item_offsets[2].offset, 50.0);
        assert_eq!(report.content_size, 80.0);
    }

    #[test]
    fn oracle_lanes_finite_search_does_not_wrap_candidate_span() {
        let report = grid::place_lanes(grid::LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: grid::LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: grid::LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                grid::LaneItemInput::auto("a", 2, 10.0),
                grid::LaneItemInput::auto("b", 2, 10.0),
            ],
        })
        .unwrap();

        assert!(
            report
                .item_offsets
                .iter()
                .all(|item| item.grid_axis_start + item.grid_axis_span <= 4)
        );
    }

    #[test]
    fn oracle_lanes_reject_definite_item_that_exceeds_grid_axis() {
        let err = grid::place_lanes(grid::LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: grid::LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: grid::LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![grid::LaneItemInput::definite("a", 3, 2, 10.0)],
        })
        .unwrap_err();

        assert_eq!(err, grid::OracleGridError::SpanOutOfRange);
    }

    #[test]
    fn oracle_lanes_infinite_tolerance_uses_round_robin_cursor() {
        let report = grid::place_lanes(grid::LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: grid::LaneAutoFlow::Column,
            lane_gap: 0.0,
            tolerance: grid::LaneFlowTolerance::Infinite,
            tolerance_basis: 0.0,
            items: vec![
                grid::LaneItemInput::auto("a", 1, 10.0),
                grid::LaneItemInput::auto("b", 1, 10.0),
                grid::LaneItemInput::auto("c", 1, 10.0),
            ],
        })
        .unwrap();

        assert_eq!(
            report
                .item_offsets
                .iter()
                .map(|item| item.grid_axis_start)
                .collect::<Vec<_>>(),
            vec![1, 2, 1]
        );
    }

    #[test]
    fn oracle_lanes_percentage_tolerance_resolves_against_basis() {
        let report = grid::place_lanes(grid::LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: grid::LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: grid::LaneFlowTolerance::Percent(0.25),
            tolerance_basis: 40.0,
            items: vec![
                grid::LaneItemInput::definite("a", 1, 1, 10.0),
                grid::LaneItemInput::auto("b", 1, 10.0),
            ],
        })
        .unwrap();

        assert_eq!(report.item_offsets[1].grid_axis_start, 2);
    }

    #[test]
    fn oracle_lanes_finite_tolerance_chooses_first_candidate_within_tolerance() {
        let report = grid::place_lanes(grid::LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: grid::LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: grid::LaneFlowTolerance::Fixed(10.0),
            tolerance_basis: 0.0,
            items: vec![
                grid::LaneItemInput::definite("a", 1, 1, 10.0),
                grid::LaneItemInput::definite("b", 2, 1, 20.0),
                grid::LaneItemInput::auto("c", 1, 10.0),
            ],
        })
        .unwrap();

        assert_eq!(report.item_offsets[2].grid_axis_start, 3);
    }

    fn oracle_lane_facts(min_content: f32, max_content: f32) -> ItemContributionFacts {
        ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content,
            max_content,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Infinite,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        }
    }

    #[test]
    fn oracle_lanes_intrinsic_keeps_definite_items_by_span() {
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![
                grid::LaneIntrinsicItem::definite(
                    "a",
                    grid::TrackSpan::new(1, 2),
                    oracle_lane_facts(20.0, 50.0),
                )
                .expect("valid oracle lane item"),
            ],
        })
        .unwrap();

        assert_eq!(report.definite_items.len(), 1);
        assert!(report.indefinite_groups.is_empty());
        assert_eq!(
            report.definite_items[0].contribution.area,
            GridArea::new(1, 1, 1, 1)
        );
    }

    #[test]
    fn oracle_lanes_intrinsic_rewrites_definite_item_area_from_span() {
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![
                grid::LaneIntrinsicItem::definite(
                    "a",
                    grid::TrackSpan::new(2, 3),
                    oracle_lane_facts(20.0, 50.0),
                )
                .expect("valid oracle lane item"),
            ],
        })
        .unwrap();

        assert_eq!(
            report.definite_items[0].contribution.area,
            GridArea::new(2, 1, 1, 1)
        );
    }

    #[test]
    fn oracle_lanes_intrinsic_rewrites_row_axis_areas_from_spans() {
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Row,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![
                grid::LaneIntrinsicItem::definite(
                    "a",
                    grid::TrackSpan::new(2, 3),
                    oracle_lane_facts(20.0, 50.0),
                )
                .expect("valid oracle lane item"),
            ],
        })
        .unwrap();

        assert_eq!(
            report.definite_items[0].contribution.area,
            GridArea::new(1, 2, 1, 1)
        );
    }

    #[test]
    fn oracle_lanes_intrinsic_groups_indefinite_items_by_span_length() {
        let facts = oracle_lane_facts(20.0, 50.0);
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1, 2],
            items: vec![
                grid::LaneIntrinsicItem::indefinite("a", oracle_lane_span(2), facts),
                grid::LaneIntrinsicItem::indefinite(
                    "b",
                    oracle_lane_span(2),
                    ItemContributionFacts {
                        min_content: 30.0,
                        max_content: 60.0,
                        ..facts
                    },
                ),
            ],
        })
        .unwrap();

        assert_eq!(report.indefinite_groups.len(), 1);
        assert_eq!(report.indefinite_groups[0].span, 2);
        assert_eq!(report.indefinite_groups[0].max_min_content, 30.0);
        assert_eq!(report.indefinite_groups[0].max_max_content, 60.0);
        assert_eq!(report.indefinite_groups[0].max_min_size, 30.0);
        assert_eq!(report.converted_indefinite_items.len(), 2);
        assert_eq!(report.final_track_report.final_tracks.len(), 3);
    }

    #[test]
    fn oracle_lanes_intrinsic_groups_indefinite_items_by_min_size() {
        let facts = ItemContributionFacts {
            automatic_minimum_applies: false,
            min_size: ContributionSize::Definite(12.0),
            ..oracle_lane_facts(100.0, 120.0)
        };
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(1),
                facts,
            )],
        })
        .unwrap();

        assert_eq!(report.indefinite_groups[0].max_min_size, 12.0);
        assert_eq!(
            report.converted_indefinite_items[0]
                .contribution
                .min_content,
            100.0
        );
        assert_eq!(
            report.converted_indefinite_items[0].contribution.min_size,
            ContributionSize::Definite(12.0)
        );
        assert!(
            !report.converted_indefinite_items[0]
                .contribution
                .automatic_minimum_applies
        );
        assert_eq!(report.final_track_report.final_tracks[0].size, 12.0);
    }

    #[test]
    fn oracle_lanes_intrinsic_uses_min_content_for_min_content_tracks() {
        let facts = ItemContributionFacts {
            automatic_minimum_applies: false,
            min_size: ContributionSize::Definite(12.0),
            ..oracle_lane_facts(100.0, 120.0)
        };
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::new(TrackMin::MinContent, TrackMax::MaxContent)],
            content_sized_tracks: vec![0],
            items: vec![grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(1),
                facts,
            )],
        })
        .unwrap();

        assert_eq!(report.indefinite_groups[0].max_min_size, 12.0);
        assert_eq!(report.indefinite_groups[0].max_min_content, 100.0);
        assert_eq!(report.final_track_report.final_tracks[0].size, 100.0);
    }

    #[test]
    fn oracle_lanes_intrinsic_converts_all_spans_that_overlap_content_tracks() {
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![
                GridTrack::fixed(20.0),
                GridTrack::auto(),
                GridTrack::fixed(20.0),
            ],
            content_sized_tracks: vec![1],
            items: vec![grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(2),
                oracle_lane_facts(30.0, 60.0),
            )],
        })
        .unwrap();

        assert_eq!(
            report
                .converted_indefinite_items
                .iter()
                .map(|item| item.span)
                .collect::<Vec<_>>(),
            vec![grid::TrackSpan::new(1, 3), grid::TrackSpan::new(2, 4),]
        );
        assert_eq!(report.final_track_report.final_tracks[1].size, 0.0);
    }

    #[test]
    fn oracle_lanes_intrinsic_distributes_converted_spanning_items() {
        let facts = ItemContributionFacts {
            automatic_minimum_applies: false,
            min_size: ContributionSize::Definite(70.0),
            ..oracle_lane_facts(90.0, 120.0)
        };
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(2),
                facts,
            )],
        })
        .unwrap();

        assert_eq!(report.converted_indefinite_items.len(), 1);
        assert_eq!(
            report.converted_indefinite_items[0].span,
            grid::TrackSpan::new(1, 3)
        );
        assert_eq!(report.final_track_report.final_tracks[0].size, 30.0);
        assert_eq!(report.final_track_report.final_tracks[1].size, 30.0);
    }

    #[test]
    fn oracle_lanes_intrinsic_splits_full_span_deficit_across_disjoint_content_tracks() {
        let facts = ItemContributionFacts {
            automatic_minimum_applies: false,
            min_size: ContributionSize::Definite(100.0),
            ..oracle_lane_facts(120.0, 160.0)
        };
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::fixed(20.0), GridTrack::auto()],
            content_sized_tracks: vec![0, 2],
            items: vec![grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(3),
                facts,
            )],
        })
        .unwrap();

        assert_eq!(report.final_track_report.final_tracks[0].size, 30.0);
        assert_eq!(report.final_track_report.final_tracks[1].size, 20.0);
        assert_eq!(report.final_track_report.final_tracks[2].size, 30.0);
    }

    #[test]
    fn oracle_lanes_intrinsic_clamps_oversized_indefinite_spans_before_reporting() {
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(5),
                oracle_lane_facts(30.0, 60.0),
            )],
        })
        .unwrap();

        assert_eq!(report.indefinite_groups[0].span, 2);
        assert_eq!(
            report
                .converted_indefinite_items
                .iter()
                .map(|item| item.span)
                .collect::<Vec<_>>(),
            vec![grid::TrackSpan::new(1, 3)]
        );
    }

    #[test]
    fn oracle_lanes_intrinsic_skips_definite_items_outside_content_tracks_for_sizing() {
        let report = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![1],
            items: vec![
                grid::LaneIntrinsicItem::definite(
                    "a",
                    grid::TrackSpan::new(1, 2),
                    oracle_lane_facts(80.0, 120.0),
                )
                .expect("valid oracle lane item"),
            ],
        })
        .unwrap();

        assert_eq!(report.definite_items.len(), 1);
        assert_eq!(report.final_track_report.final_tracks[0].size, 0.0);
        assert_eq!(report.final_track_report.final_tracks[1].size, 0.0);
    }

    #[test]
    fn oracle_lanes_intrinsic_rejects_invalid_definite_span() {
        let err = grid::LaneIntrinsicItem::definite(
            "bad",
            grid::TrackSpan::new(2, 2),
            oracle_lane_facts(20.0, 50.0),
        )
        .unwrap_err();

        assert_eq!(err, grid::OracleGridError::SpanOutOfRange);
    }

    #[test]
    fn oracle_lanes_intrinsic_rejects_definite_span_outside_tracks() {
        let err = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto()],
            content_sized_tracks: vec![0],
            items: vec![
                grid::LaneIntrinsicItem::definite(
                    "bad",
                    grid::TrackSpan::new(2, 3),
                    oracle_lane_facts(20.0, 50.0),
                )
                .expect("valid oracle lane item"),
            ],
        })
        .unwrap_err();

        assert_eq!(err, grid::OracleGridError::SpanOutOfRange);
    }

    #[test]
    fn oracle_lanes_intrinsic_rejects_invalid_content_sized_track() {
        let err = grid::lane_intrinsic_sizing(grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto()],
            content_sized_tracks: vec![1],
            items: vec![grid::LaneIntrinsicItem::indefinite(
                "bad",
                oracle_lane_span(1),
                oracle_lane_facts(20.0, 50.0),
            )],
        })
        .unwrap_err();

        assert_eq!(err, grid::OracleGridError::SpanOutOfRange);
    }

    #[test]
    fn oracle_scenario_composes_subgrid_rect_from_explicit_tracks_and_offsets() {
        let report = grid::compose_subgrid_item_rect(grid::SubgridItemRectInput {
            inherited_axis: GridAxis::Column,
            inherited_axis_offset: 20.0,
            standalone_axis_offset: 5.0,
            inherited_axis_size: 80.0,
            standalone_axis_size: 30.0,
            container_mbp_offset: grid::AxisEdges {
                start: 3.0,
                end: 0.0,
            },
            item_inline_offset: 7.0,
            item_block_offset: 11.0,
        });

        assert_eq!(report.inherited_axis_offset, 30.0);
        assert_eq!(report.standalone_axis_offset, 16.0);
        assert_eq!(report.rect, GridItemRect::new(30.0, 16.0, 80.0, 30.0));
    }

    #[test]
    fn oracle_scenario_composes_lane_rect_from_lane_offset_and_grid_axis_area() {
        let rect = grid::compose_lane_item_rect(grid::LaneItemRectInput {
            grid_axis_start: 12.0,
            grid_axis_size: 50.0,
            lane_axis_offset: 27.0,
            lane_axis_size: 40.0,
            grid_axis_is_column: true,
        });

        assert_eq!(rect, GridItemRect::new(12.0, 27.0, 50.0, 40.0));
    }

    #[test]
    fn oracle_scenario_offsets_grid_items_by_baseline_report() {
        let baseline_rect =
            grid::compose_baseline_aligned_item_rect(grid::BaselineAlignedItemRectInput {
                area_x: 10.0,
                area_y: 4.0,
                area_width: 50.0,
                area_height: 40.0,
                item_width: 20.0,
                item_height: 30.0,
                normal_x_offset: 3.0,
                normal_y_offset: 8.0,
                baseline_y_offset: Some(6.0),
            });

        assert_eq!(baseline_rect, GridItemRect::new(13.0, 10.0, 20.0, 30.0));

        let normal_rect =
            grid::compose_baseline_aligned_item_rect(grid::BaselineAlignedItemRectInput {
                area_x: 10.0,
                area_y: 4.0,
                area_width: 50.0,
                area_height: 40.0,
                item_width: 20.0,
                item_height: 30.0,
                normal_x_offset: 3.0,
                normal_y_offset: 8.0,
                baseline_y_offset: None,
            });

        assert_eq!(normal_rect, GridItemRect::new(13.0, 12.0, 20.0, 30.0));
    }

    #[test]
    fn oracle_direct_subgrid_inherited_columns_shape() {
        let inherited = grid::inherit_subgrid_tracks(grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![80.0, 120.0],
            parent_span: grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: grid::OracleGapReport::length(10.0),
            subgrid_gap: grid::OracleGapReport::normal_resolved_to(10.0),
        })
        .unwrap();

        assert_eq!(inherited.final_tracks, vec![80.0, 120.0]);
    }

    #[test]
    fn oracle_grid_lanes_three_item_shape() {
        let report = grid::place_lanes(grid::LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: grid::LaneAutoFlow::Row,
            lane_gap: 5.0,
            tolerance: grid::LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                grid::LaneItemInput::auto("a", 1, 20.0),
                grid::LaneItemInput::auto("b", 1, 30.0),
                grid::LaneItemInput::auto("c", 2, 10.0),
            ],
        })
        .unwrap();

        assert_eq!(report.item_offsets.len(), 3);
        assert_eq!(report.content_size, 45.0);
    }
}

mod root_layout_oracle {
    use crate::test_support::{
        grid_layout_comparison::{GridLayoutComparison, GridLayoutNode},
        layout_tree::{OracleMeasurement, OracleTree},
        oracle::grid::{
            self, AutoPlacer, AxisEdges, ContributionSize, Flow, GridArea, GridAxis, GridTrack,
            ItemContributionFacts, LaneAutoFlow, LaneFlowTolerance, LaneIntrinsicItem,
            LaneIntrinsicSizingInput, LaneItemInput, LanePlacementInput, SubgridItemRectInput,
            TrackMax, TrackMin, TrackSizingSlice, compose_subgrid_item_rect,
        },
    };
    use crate::{
        AlignContent, AlignItems, Available, ComputeInput, ComputeOutput, Direction, Display,
        Edges, GridAutoFlow, GridAxisKind as ProductionGridAxisKind, GridFlowTolerance,
        GridPlacement, GridTemplateAreaRow, GridTemplateAreas,
        LaneContributionFacts as ProductionLaneContributionFacts,
        LaneIntrinsicItem as ProductionLaneIntrinsicItem,
        LaneIntrinsicSizingInput as ProductionLaneIntrinsicSizingInput,
        LaneItem as ProductionLaneItem, LanePlacementInput as ProductionLanePlacementInput, Length,
        LengthAuto, MaxTrackSizing, MinTrackSizing, NodeInput, Point, Position, PreferredSize,
        RawGridLine, RawGridPlacement, RequestedAxis, RunMode, Size, SizingCalculation, SizingMode,
        TrackComponent, TrackFlexFactor, TrackSizing as ProductionTrackSizing, WritingMode,
        compute_root, lane_intrinsic_sizing as production_lane_intrinsic_sizing,
        place_lanes as production_place_lanes, round_layout,
    };

    fn track_component_flex(value: f32) -> TrackComponent {
        TrackComponent::flex(TrackFlexFactor::try_new(value).expect("valid test flex factor"))
    }

    fn oracle_lane_span(value: usize) -> grid::LaneTrackSpanLength {
        grid::LaneTrackSpanLength::new(value).expect("valid oracle lane span length")
    }

    fn production_lane_span(value: usize) -> crate::LaneTrackSpanLength {
        crate::LaneTrackSpanLength::new(value).expect("valid production lane span length")
    }

    fn fixed_rows(height: f32) -> grid::TrackSizingReport {
        TrackSizingSlice::definite_rows(height, 0.0)
            .track(GridTrack::fixed(height))
            .solve()
    }

    fn assert_layout_close(actual: f32, expected: f32, label: &str) {
        assert!(
            (actual - expected).abs() <= 0.000_1,
            "{label}: expected {expected}, got {actual}"
        );
    }

    fn named_grid_oracle_lines() -> grid::NamedGridLines {
        grid::NamedGridLines::new(
            GridAxis::Column,
            3,
            vec![
                vec!["a", "foo-start"],
                vec!["a", "foo", "foo-end"],
                vec!["a"],
                vec![],
            ],
        )
        .unwrap()
    }

    fn named_grid_track_components() -> Vec<TrackComponent> {
        vec![
            TrackComponent::line_names(["a", "foo-start"]),
            TrackComponent::px(40.0),
            TrackComponent::line_names(["a", "foo", "foo-end"]),
            TrackComponent::px(40.0),
            TrackComponent::line_names(["a"]),
            TrackComponent::px(40.0),
        ]
    }

    fn assert_named_grid_column_matches_oracle(
        raw_column: RawGridPlacement,
        oracle_column: grid::NamedAxisPlacement,
        auto_cursor_line: Option<isize>,
        grid_auto_columns: Vec<TrackComponent>,
        label: &str,
    ) {
        let expected = grid::resolve_named_axis_placement(
            &named_grid_oracle_lines(),
            oracle_column,
            auto_cursor_line,
        )
        .unwrap()
        .resolved;

        let mut tree = OracleTree::new()
            .children(1, [2])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(200.0), PreferredSize::px(20.0)),
                    grid_template_columns: named_grid_track_components(),
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    grid_auto_columns,
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    raw_grid_column: raw_column,
                    raw_grid_row: RawGridPlacement::line(1),
                    ..NodeInput::DEFAULT
                },
            );

        compute_root(
            &mut tree,
            1,
            Size::new(Available::Definite(200.0), Available::Definite(20.0)),
        )
        .unwrap();
        round_layout(&mut tree, 1).unwrap();
        let actual = tree.final_layout(2).expect("child layout");

        assert_layout_close(
            actual.location.x,
            (expected.start_line as f32 - 1.0) * 40.0,
            &format!("{label} x"),
        );
        assert_layout_close(
            actual.size.width,
            expected.span as f32 * 40.0,
            &format!("{label} width"),
        );
    }

    fn assert_named_grid_column_falls_back_to_auto_when_oracle_rejects(
        raw_column: RawGridPlacement,
        oracle_column: grid::NamedAxisPlacement,
        expected_error: grid::NamedGridError,
    ) {
        let oracle_error =
            grid::resolve_named_axis_placement(&named_grid_oracle_lines(), oracle_column, None)
                .unwrap_err();
        assert_eq!(oracle_error, expected_error);

        let layout_for = |raw_grid_column: RawGridPlacement| {
            let mut tree = OracleTree::new()
                .children(1, [2])
                .style(
                    1,
                    NodeInput {
                        display: Display::Grid,
                        size: Size::new(PreferredSize::px(200.0), PreferredSize::px(20.0)),
                        grid_template_columns: named_grid_track_components(),
                        grid_template_rows: vec![TrackComponent::px(20.0)],
                        ..NodeInput::DEFAULT
                    },
                )
                .style(
                    2,
                    NodeInput {
                        raw_grid_column,
                        raw_grid_row: RawGridPlacement::line(1),
                        ..NodeInput::DEFAULT
                    },
                );

            compute_root(
                &mut tree,
                1,
                Size::new(Available::Definite(200.0), Available::Definite(20.0)),
            )
            .unwrap();
            round_layout(&mut tree, 1).unwrap();
            tree.final_layout(2).expect("child layout")
        };

        let invalid_named = layout_for(raw_column);
        let plain_auto = layout_for(RawGridPlacement::AUTO);

        assert_layout_close(
            invalid_named.location.x,
            plain_auto.location.x,
            "fallback auto x",
        );
        assert_layout_close(
            invalid_named.size.width,
            plain_auto.size.width,
            "fallback auto width",
        );
    }

    #[test]
    fn named_grid_layout_oracle_matches_bare_explicit_and_repeated_names() {
        use grid::{NamedAxisPlacement, NamedGridLine};

        assert_named_grid_column_matches_oracle(
            RawGridPlacement::new(
                RawGridLine::BareIdent("foo".to_string()),
                RawGridLine::BareIdent("foo".to_string()),
            ),
            NamedAxisPlacement {
                start: NamedGridLine::BareIdent("foo".to_string()),
                end: NamedGridLine::BareIdent("foo".to_string()),
            },
            None,
            Vec::new(),
            "bare foo",
        );
        assert_named_grid_column_matches_oracle(
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "foo".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "foo".to_string(),
                    index: 1,
                },
            ),
            NamedAxisPlacement {
                start: NamedGridLine::Named {
                    name: "foo".to_string(),
                    occurrence: 1,
                },
                end: NamedGridLine::Named {
                    name: "foo".to_string(),
                    occurrence: 1,
                },
            },
            None,
            Vec::new(),
            "explicit foo",
        );
        assert_named_grid_column_matches_oracle(
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "a".to_string(),
                    index: 2,
                },
                RawGridLine::NamedSpan {
                    name: "a".to_string(),
                    index: 1,
                },
            ),
            NamedAxisPlacement {
                start: NamedGridLine::Named {
                    name: "a".to_string(),
                    occurrence: 2,
                },
                end: NamedGridLine::Span {
                    name: Some("a".to_string()),
                    count: 1,
                },
            },
            None,
            Vec::new(),
            "repeated named span",
        );
    }

    #[test]
    fn named_grid_layout_oracle_matches_negative_missing_and_backward_spans() {
        use grid::{NamedAxisPlacement, NamedGridLine};

        assert_named_grid_column_matches_oracle(
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "a".to_string(),
                    index: -1,
                },
                RawGridLine::Auto,
            ),
            NamedAxisPlacement {
                start: NamedGridLine::Named {
                    name: "a".to_string(),
                    occurrence: -1,
                },
                end: NamedGridLine::Auto,
            },
            None,
            Vec::new(),
            "negative occurrence",
        );
        assert_named_grid_column_matches_oracle(
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "a".to_string(),
                    index: 4,
                },
                RawGridLine::Auto,
            ),
            NamedAxisPlacement {
                start: NamedGridLine::Named {
                    name: "a".to_string(),
                    occurrence: 4,
                },
                end: NamedGridLine::Auto,
            },
            None,
            vec![TrackComponent::px(40.0)],
            "missing after occurrence",
        );
        assert_named_grid_column_falls_back_to_auto_when_oracle_rejects(
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "missing".to_string(),
                    index: -4,
                },
                RawGridLine::Auto,
            ),
            NamedAxisPlacement {
                start: NamedGridLine::Named {
                    name: "missing".to_string(),
                    occurrence: -4,
                },
                end: NamedGridLine::Auto,
            },
            grid::NamedGridError::LineBeforeFirst {
                axis: GridAxis::Column,
                start_line: -3,
                end_line: -2,
            },
        );
        assert_named_grid_column_matches_oracle(
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "a".to_string(),
                    index: 2,
                },
                RawGridLine::Line(3),
            ),
            NamedAxisPlacement {
                start: NamedGridLine::Span {
                    name: Some("a".to_string()),
                    count: 2,
                },
                end: NamedGridLine::Number(3),
            },
            None,
            Vec::new(),
            "backward named span",
        );
    }

    #[test]
    fn named_grid_layout_oracle_matches_auto_span_and_conflict_normalization() {
        use grid::{NamedAxisPlacement, NamedGridLine};

        assert_named_grid_column_matches_oracle(
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "a".to_string(),
                    index: 2,
                },
                RawGridLine::Auto,
            ),
            NamedAxisPlacement {
                start: NamedGridLine::Span {
                    name: Some("a".to_string()),
                    count: 2,
                },
                end: NamedGridLine::Auto,
            },
            Some(1),
            Vec::new(),
            "lone named span",
        );
        assert_named_grid_column_matches_oracle(
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "a".to_string(),
                    index: 2,
                },
                RawGridLine::Span(3),
            ),
            NamedAxisPlacement {
                start: NamedGridLine::Span {
                    name: Some("a".to_string()),
                    count: 2,
                },
                end: NamedGridLine::Span {
                    name: None,
                    count: 3,
                },
            },
            Some(1),
            Vec::new(),
            "mixed spans",
        );
        assert_named_grid_column_matches_oracle(
            RawGridPlacement::lines(3, 1),
            NamedAxisPlacement {
                start: NamedGridLine::Number(3),
                end: NamedGridLine::Number(1),
            },
            None,
            Vec::new(),
            "start after end",
        );
        assert_named_grid_column_matches_oracle(
            RawGridPlacement::lines(2, 2),
            NamedAxisPlacement {
                start: NamedGridLine::Number(2),
                end: NamedGridLine::Number(2),
            },
            None,
            Vec::new(),
            "equal lines",
        );
    }

    #[test]
    fn named_grid_layout_oracle_matches_template_area_generated_lines() {
        use grid::{NamedAxisPlacement, NamedGridLine};

        let areas = grid::TemplateAreas::new([vec!["foo", "foo", "bar"]]).unwrap();
        let columns = grid::area_generated_lines(
            GridAxis::Column,
            &areas,
            grid::NamedGridLines::empty(GridAxis::Column, 3),
        )
        .unwrap();
        let expected = grid::resolve_named_axis_placement(
            &columns,
            NamedAxisPlacement {
                start: NamedGridLine::BareIdent("foo".to_string()),
                end: NamedGridLine::BareIdent("foo".to_string()),
            },
            None,
        )
        .unwrap()
        .resolved;

        let mut tree = OracleTree::new()
            .children(1, [2])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                    grid_template_columns: vec![
                        TrackComponent::px(40.0),
                        TrackComponent::px(40.0),
                        TrackComponent::px(40.0),
                    ],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    grid_template_areas: GridTemplateAreas {
                        rows: vec![GridTemplateAreaRow {
                            cells: vec![
                                Some("foo".to_string()),
                                Some("foo".to_string()),
                                Some("bar".to_string()),
                            ],
                        }],
                    },
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::BareIdent("foo".to_string()),
                        RawGridLine::BareIdent("foo".to_string()),
                    ),
                    raw_grid_row: RawGridPlacement::line(1),
                    ..NodeInput::DEFAULT
                },
            );

        compute_root(
            &mut tree,
            1,
            Size::new(Available::Definite(120.0), Available::Definite(20.0)),
        )
        .unwrap();
        round_layout(&mut tree, 1).unwrap();
        let child = tree.final_layout(2).expect("child layout");

        assert_layout_close(
            child.location.x,
            (expected.start_line as f32 - 1.0) * 40.0,
            "area generated x",
        );
        assert_layout_close(
            child.size.width,
            expected.span as f32 * 40.0,
            "area generated width",
        );
    }

    #[test]
    fn subgrid_layout_oracle_matches_merged_local_and_inherited_area_lines() {
        use grid::{NamedAxisPlacement, NamedGridLine, TrackSpan};

        let parent_areas = grid::TemplateAreas::new([vec![".", "parent", "parent", "."]]).unwrap();
        let parent_facts = grid::area_generated_facts(
            &parent_areas,
            grid::NamedGridLines::empty(GridAxis::Column, 4),
            grid::NamedGridLines::empty(GridAxis::Row, 2),
        )
        .unwrap();
        let inherited = grid::inherit_named_subgrid_lines(
            &parent_facts.columns,
            TrackSpan::new(2, 4),
            false,
            vec![vec![], vec![], vec![]],
            Some(&parent_facts),
        )
        .unwrap();
        let local_areas = grid::TemplateAreas::new([vec!["local", "local"]]).unwrap();
        let merged_columns =
            grid::area_generated_lines(GridAxis::Column, &local_areas, inherited.lines).unwrap();

        let expected_local = grid::resolve_named_axis_placement(
            &merged_columns,
            NamedAxisPlacement {
                start: NamedGridLine::BareIdent("local".to_string()),
                end: NamedGridLine::BareIdent("local".to_string()),
            },
            None,
        )
        .unwrap()
        .resolved;
        let expected_parent = grid::resolve_named_axis_placement(
            &merged_columns,
            NamedAxisPlacement {
                start: NamedGridLine::BareIdent("parent".to_string()),
                end: NamedGridLine::BareIdent("parent".to_string()),
            },
            None,
        )
        .unwrap()
        .resolved;

        let mut tree = OracleTree::new()
            .children(1, [2])
            .children(2, [3, 4])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(160.0), PreferredSize::px(40.0)),
                    grid_template_columns: vec![
                        TrackComponent::px(40.0),
                        TrackComponent::px(40.0),
                        TrackComponent::px(40.0),
                        TrackComponent::px(40.0),
                    ],
                    grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                    grid_template_areas: GridTemplateAreas {
                        rows: vec![GridTemplateAreaRow {
                            cells: vec![
                                None,
                                Some("parent".to_string()),
                                Some("parent".to_string()),
                                None,
                            ],
                        }],
                    },
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(2, 4).expect("valid grid placement"),
                    grid_row: GridPlacement::try_lines(1, 3).expect("valid grid placement"),
                    grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })],
                    grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                    grid_template_areas: GridTemplateAreas {
                        rows: vec![GridTemplateAreaRow {
                            cells: vec![Some("local".to_string()), Some("local".to_string())],
                        }],
                    },
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                3,
                NodeInput {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::BareIdent("local".to_string()),
                        RawGridLine::BareIdent("local".to_string()),
                    ),
                    raw_grid_row: RawGridPlacement::line(1),
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                4,
                NodeInput {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::BareIdent("parent".to_string()),
                        RawGridLine::BareIdent("parent".to_string()),
                    ),
                    raw_grid_row: RawGridPlacement::line(2),
                    ..NodeInput::DEFAULT
                },
            );

        compute_root(
            &mut tree,
            1,
            Size::new(Available::Definite(160.0), Available::Definite(40.0)),
        )
        .unwrap();
        round_layout(&mut tree, 1).unwrap();

        for (node, expected, label) in [
            (3, expected_local, "local area"),
            (4, expected_parent, "inherited area"),
        ] {
            let child = tree.final_layout(node).expect("child layout");
            assert_layout_close(
                child.location.x,
                (expected.start_line as f32 - 1.0) * 40.0,
                &format!("{label} x"),
            );
            assert_layout_close(
                child.size.width,
                expected.span as f32 * 40.0,
                &format!("{label} width"),
            );
        }
    }

    #[test]
    fn subgrid_layout_oracle_matches_local_area_clamp_to_inherited_span() {
        use grid::{NamedAxisPlacement, NamedGridLine};

        let clamped_local_areas = grid::TemplateAreas::new([vec!["wide", "wide"]]).unwrap();
        let columns = grid::area_generated_lines(
            GridAxis::Column,
            &clamped_local_areas,
            grid::NamedGridLines::empty(GridAxis::Column, 2),
        )
        .unwrap();
        let expected = grid::resolve_named_axis_placement(
            &columns,
            NamedAxisPlacement {
                start: NamedGridLine::BareIdent("wide".to_string()),
                end: NamedGridLine::BareIdent("wide".to_string()),
            },
            None,
        )
        .unwrap()
        .resolved;

        let mut tree = OracleTree::new()
            .children(1, [2])
            .children(2, [3])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(160.0), PreferredSize::px(20.0)),
                    grid_template_columns: vec![
                        TrackComponent::px(40.0),
                        TrackComponent::px(40.0),
                        TrackComponent::px(40.0),
                        TrackComponent::px(40.0),
                    ],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 3).expect("valid grid placement"),
                    grid_row: GridPlacement::try_line(1).expect("valid grid placement"),
                    grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    grid_template_areas: GridTemplateAreas {
                        rows: vec![GridTemplateAreaRow {
                            cells: vec![
                                Some("wide".to_string()),
                                Some("wide".to_string()),
                                Some("wide".to_string()),
                                Some("wide".to_string()),
                            ],
                        }],
                    },
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                3,
                NodeInput {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::BareIdent("wide".to_string()),
                        RawGridLine::BareIdent("wide".to_string()),
                    ),
                    raw_grid_row: RawGridPlacement::line(1),
                    ..NodeInput::DEFAULT
                },
            );

        compute_root(
            &mut tree,
            1,
            Size::new(Available::Definite(160.0), Available::Definite(20.0)),
        )
        .unwrap();
        round_layout(&mut tree, 1).unwrap();
        let child = tree.final_layout(3).expect("child layout");

        assert_layout_close(
            child.location.x,
            (expected.start_line as f32 - 1.0) * 40.0,
            "clamped local area x",
        );
        assert_layout_close(
            child.size.width,
            expected.span as f32 * 40.0,
            "clamped local area width",
        );
    }

    #[test]
    fn oracle_layout_fixed_tracks_match_layout_child_rects() {
        let expected_columns = TrackSizingSlice::definite_columns(210.0, 10.0)
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(120.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(40.0, 0.0)
            .track(GridTrack::fixed(40.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(210.0, 40.0))
            .columns(vec![TrackComponent::px(80.0), TrackComponent::px(120.0)])
            .rows(vec![TrackComponent::px(40.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .child(GridArea::new(1, 1, 1, 1))
            .child(GridArea::new(2, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn fri08_c08_t06_suppression_cleanup_live_grid_support_matches_fixed_track_layout() {
        let expected_columns = TrackSizingSlice::definite_columns(90.0, 0.0)
            .track(GridTrack::fixed(90.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(35.0, 0.0)
            .track(GridTrack::fixed(35.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(90.0, 35.0))
            .columns(vec![TrackComponent::px(90.0)])
            .rows(vec![TrackComponent::px(35.0)])
            .expected_tracks(expected_columns, expected_rows)
            .child(GridArea::new(1, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_percent_and_flex_tracks_match_layout_child_rects() {
        let expected_columns = TrackSizingSlice::definite_columns(400.0, 20.0)
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::percent(0.25))
            .track(GridTrack::flex(1.0))
            .track(GridTrack::flex(3.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(400.0, 40.0))
            .columns(vec![
                TrackComponent::px(80.0),
                TrackComponent::percent(0.25),
                track_component_flex(1.0),
                track_component_flex(3.0),
            ])
            .rows(vec![TrackComponent::px(40.0)])
            .gap(Size::new(20.0, 0.0))
            .expected_tracks(expected_columns, fixed_rows(40.0))
            .child(GridArea::new(1, 1, 1, 1))
            .child(GridArea::new(2, 1, 1, 1))
            .child(GridArea::new(3, 1, 1, 1))
            .child(GridArea::new(4, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_sub_one_flex_track_uses_partial_leftover_space() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 0.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::flex(0.5))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![TrackComponent::px(50.0), track_component_flex(0.5)])
            .rows(vec![TrackComponent::px(30.0)])
            .expected_tracks(expected_columns, fixed_rows(30.0))
            .child(GridArea::new(1, 1, 1, 1))
            .child(GridArea::new(2, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_minmax_tracks_match_layout_child_rects() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 0.0)
            .track(GridTrack::new(
                grid::TrackMin::Fixed(40.0),
                grid::TrackMax::Fixed(90.0),
            ))
            .track(GridTrack::new(
                grid::TrackMin::Percent(0.25),
                grid::TrackMax::Auto,
            ))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::minmax(MinTrackSizing::px(40.0), MaxTrackSizing::px(90.0)),
                TrackComponent::minmax(MinTrackSizing::percent(0.25), MaxTrackSizing::AUTO),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .expected_tracks(expected_columns, fixed_rows(30.0))
            .child(GridArea::new(1, 1, 1, 1))
            .child(GridArea::new(2, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_stretch_expands_auto_tracks_like_layout() {
        let expected_columns = TrackSizingSlice::definite_columns(120.0, 20.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .stretch_auto_tracks()
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(120.0, 30.0))
            .columns(vec![TrackComponent::AUTO, TrackComponent::AUTO])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(20.0, 0.0))
            .justify_content(AlignContent::Stretch)
            .expected_tracks(expected_columns, fixed_rows(30.0))
            .child(GridArea::new(1, 1, 1, 1))
            .child(GridArea::new(2, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_explicit_line_span_matches_layout_area_rect() {
        let expected_columns = TrackSizingSlice::definite_columns(250.0, 10.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::fixed(70.0))
            .track(GridTrack::fixed(110.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(250.0, 40.0))
            .columns(vec![
                TrackComponent::px(50.0),
                TrackComponent::px(70.0),
                TrackComponent::px(110.0),
            ])
            .rows(vec![TrackComponent::px(40.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, fixed_rows(40.0))
            .child(GridArea::new(2, 1, 2, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_row_auto_flow_matches_oracle_placement() {
        let mut placement = AutoPlacer::try_new(2, 2, Flow::Row).unwrap();
        let first = placement.place(1, 1).unwrap();
        let second = placement.place(1, 1).unwrap();
        let third = placement.place(1, 1).unwrap();
        let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::fixed(50.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(45.0, 5.0)
            .track(GridTrack::fixed(20.0))
            .track(GridTrack::fixed(20.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(110.0, 45.0))
            .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
            .rows(vec![TrackComponent::px(20.0), TrackComponent::px(20.0)])
            .gap(Size::new(10.0, 5.0))
            .expected_tracks(expected_columns, expected_rows)
            .auto_child(first)
            .auto_child(second)
            .auto_child(third)
            .assert_layout();
    }

    #[test]
    fn oracle_layout_column_auto_flow_matches_oracle_placement() {
        let mut placement = AutoPlacer::try_new(2, 2, Flow::Column).unwrap();
        let first = placement.place(1, 1).unwrap();
        let second = placement.place(1, 1).unwrap();
        let third = placement.place(1, 1).unwrap();
        let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::fixed(50.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(45.0, 5.0)
            .track(GridTrack::fixed(20.0))
            .track(GridTrack::fixed(20.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(110.0, 45.0))
            .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
            .rows(vec![TrackComponent::px(20.0), TrackComponent::px(20.0)])
            .gap(Size::new(10.0, 5.0))
            .auto_flow(GridAutoFlow::Column)
            .expected_tracks(expected_columns, expected_rows)
            .auto_child(first)
            .auto_child(second)
            .auto_child(third)
            .assert_layout();
    }

    #[test]
    fn oracle_layout_dense_auto_flow_matches_spanning_oracle_placement() {
        let mut placement = AutoPlacer::try_new(3, 2, Flow::RowDense).unwrap();
        let first = placement.place(2, 1).unwrap();
        let second = placement.place(2, 1).unwrap();
        let third = placement.place(1, 1).unwrap();
        let expected_columns = TrackSizingSlice::definite_columns(150.0, 0.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(40.0, 0.0)
            .track(GridTrack::fixed(20.0))
            .track(GridTrack::fixed(20.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(150.0, 40.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(50.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(20.0), TrackComponent::px(20.0)])
            .auto_flow(GridAutoFlow::RowDense)
            .expected_tracks(expected_columns, expected_rows)
            .auto_spanning_child(first, 2, 1)
            .auto_spanning_child(second, 2, 1)
            .auto_child(third)
            .assert_layout();
    }

    #[test]
    fn oracle_layout_center_alignment_offsets_tracks_like_layout() {
        let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::fixed(50.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .justify_content(AlignContent::Center)
            .expected_tracks(expected_columns, fixed_rows(30.0))
            .child(GridArea::new(1, 1, 1, 1))
            .child(GridArea::new(2, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_space_between_alignment_offsets_tracks_like_layout() {
        let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::fixed(50.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .justify_content(AlignContent::SpaceBetween)
            .expected_tracks(expected_columns, fixed_rows(30.0))
            .child(GridArea::new(1, 1, 1, 1))
            .child(GridArea::new(2, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_safe_center_alignment_falls_back_on_overflow() {
        let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
            .track(GridTrack::fixed(50.0))
            .track(GridTrack::fixed(50.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(80.0, 30.0))
            .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .justify_content(AlignContent::SafeCenter)
            .expected_tracks(expected_columns, fixed_rows(30.0))
            .child(GridArea::new(1, 1, 1, 1))
            .child(GridArea::new(2, 1, 1, 1))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_auto_track_uses_supplied_intrinsic_measurement() {
        let expected_columns = TrackSizingSlice::definite_columns(80.0, 0.0)
            .track(GridTrack::auto())
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 1),
                min_content: 80.0,
                max_content: 80.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            })
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(80.0, 20.0))
            .columns(vec![TrackComponent::AUTO])
            .rows(vec![TrackComponent::px(20.0)])
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .measured_child(GridArea::new(1, 1, 1, 1), Size::new(80.0, 10.0))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_spanning_auto_tracks_distribute_intrinsic_deficit() {
        let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 2, 1),
                min_content: 110.0,
                max_content: 110.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            })
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(110.0, 20.0))
            .columns(vec![TrackComponent::AUTO, TrackComponent::AUTO])
            .rows(vec![TrackComponent::px(20.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .measured_child(GridArea::new(1, 1, 2, 1), Size::new(110.0, 10.0))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_fit_content_track_clamps_intrinsic_growth() {
        let expected_columns = TrackSizingSlice::definite_columns(40.0, 0.0)
            .track(GridTrack::new(TrackMin::Auto, TrackMax::FitContent(40.0)))
            .item(ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 1),
                min_content: 90.0,
                max_content: 90.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Auto,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            })
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(40.0, 20.0))
            .columns(vec![TrackComponent::minmax(
                MinTrackSizing::AUTO,
                MaxTrackSizing::fit_content(SizingCalculation::value(super::lp(40.0, 0.0))),
            )])
            .rows(vec![TrackComponent::px(20.0)])
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .measured_child(GridArea::new(1, 1, 1, 1), Size::new(90.0, 10.0))
            .assert_layout();
    }

    #[test]
    fn oracle_layout_harness_asserts_nested_grid_descendant_output() {
        let expected_columns = TrackSizingSlice::definite_columns(120.0, 0.0)
            .track(GridTrack::fixed(120.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(60.0, 0.0)
            .track(GridTrack::fixed(60.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(120.0, 60.0))
            .columns(vec![TrackComponent::px(120.0)])
            .rows(vec![TrackComponent::px(60.0)])
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::grid(GridArea::new(1, 1, 1, 1))
                    .margin(Edges::new(
                        LengthAuto::px(6.0),
                        LengthAuto::px(4.0),
                        LengthAuto::px(2.0),
                        LengthAuto::px(10.0),
                    ))
                    .expect_layout(Point::new(10.0, 6.0), Size::new(106.0, 52.0))
                    .columns(vec![TrackComponent::px(30.0), TrackComponent::px(76.0)])
                    .rows(vec![TrackComponent::px(52.0)])
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .size(Size::new(PreferredSize::px(76.0), PreferredSize::px(52.0)))
                            .expect_layout(Point::new(30.0, 0.0), Size::new(76.0, 52.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_child_rect_matches_oracle_composed_rect() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();
        let rect = compose_subgrid_item_rect(SubgridItemRectInput {
            inherited_axis: GridAxis::Column,
            inherited_axis_offset: 50.0,
            standalone_axis_offset: 0.0,
            inherited_axis_size: 60.0,
            standalone_axis_size: 30.0,
            container_mbp_offset: AxisEdges {
                start: 0.0,
                end: 0.0,
            },
            item_inline_offset: 90.0,
            item_block_offset: 0.0,
        })
        .rect;

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .expect_layout(
                                Point::new(rect.x - 50.0, rect.y),
                                Size::new(rect.width, rect.height),
                            )
                            .expect_final_layout(
                                Point::new(rect.x - 50.0, rect.y),
                                Size::new(rect.width, rect.height),
                            ),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_child_items_resolve_against_local_lines() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                            .expect_layout(Point::new(0.0, 0.0), Size::new(80.0, 30.0))
                            .expect_final_layout(Point::new(0.0, 0.0), Size::new(80.0, 30.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_standalone_axis_uses_ordinary_child_tracks() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(50.0, 0.0)
            .track(GridTrack::fixed(50.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 50.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(50.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .rows(vec![TrackComponent::px(12.0), TrackComponent::px(18.0)])
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 50.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 2, 1, 1))
                            .expect_layout(Point::new(0.0, 12.0), Size::new(80.0, 18.0))
                            .expect_final_layout(Point::new(0.0, 12.0), Size::new(80.0, 18.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_item_still_respects_parent_grid_placement() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0)),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_child_auto_margins_use_inherited_area_size() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .size(Size::new(PreferredSize::px(20.0), PreferredSize::px(30.0)))
                            .margin(Edges::new(
                                LengthAuto::px(0.0),
                                LengthAuto::auto(),
                                LengthAuto::px(0.0),
                                LengthAuto::auto(),
                            ))
                            .expect_layout(Point::new(110.0, 0.0), Size::new(20.0, 30.0))
                            .expect_final_layout(Point::new(110.0, 0.0), Size::new(20.0, 30.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_child_alignment_uses_inherited_area_size() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .size(Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)))
                            .justify_self(AlignItems::Center)
                            .align_self(AlignItems::End)
                            .expect_layout(Point::new(110.0, 20.0), Size::new(20.0, 10.0))
                            .expect_final_layout(Point::new(110.0, 20.0), Size::new(20.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_rtl_child_lines_use_reversed_inherited_columns() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .direction(Direction::Rtl)
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                            .expect_layout(Point::new(90.0, 0.0), Size::new(60.0, 30.0))
                            .expect_final_layout(Point::new(90.0, 0.0), Size::new(60.0, 30.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_explicit_zero_gap_overrides_parent_gap() {
        let expected_columns = TrackSizingSlice::definite_columns(220.0, 20.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(220.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(20.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .gap(Size::new(Length::ZERO, Length::ZERO))
                    .expect_layout(Point::new(60.0, 0.0), Size::new(160.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .expect_layout(Point::new(90.0, 0.0), Size::new(70.0, 30.0))
                            .expect_final_layout(Point::new(90.0, 0.0), Size::new(70.0, 30.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_percent_gap_uses_content_box_basis() {
        let expected_columns = TrackSizingSlice::definite_columns(220.0, 20.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(220.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(20.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .padding(Edges::new(
                        Length::ZERO,
                        Length::percent(0.1),
                        Length::ZERO,
                        Length::percent(0.1),
                    ))
                    .gap(Size::new(Length::percent(0.1), Length::ZERO))
                    .expect_layout(Point::new(60.0, 0.0), Size::new(160.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .expect_layout(Point::new(96.4, 0.0), Size::new(47.6, 30.0))
                            .expect_final_layout(Point::new(96.0, 0.0), Size::new(48.0, 30.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_percentage_padding_uses_grid_area_basis() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .padding(Edges::new(
                        Length::ZERO,
                        Length::percent(0.1),
                        Length::ZERO,
                        Length::percent(0.1),
                    ))
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                            .expect_layout(Point::new(15.0, 0.0), Size::new(65.0, 30.0))
                            .expect_final_layout(Point::new(15.0, 0.0), Size::new(65.0, 30.0)),
                    ),
            )
            .assert_layout();
    }

    fn intrinsic_item(area: GridArea, size: f32) -> ItemContributionFacts {
        ItemContributionFacts {
            area,
            min_content: size,
            max_content: size,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        }
    }

    #[test]
    fn subgrid_traversal_nested_inherited_leaf_contribution_grows_parent_auto_track() {
        let expected_columns = TrackSizingSlice::definite_columns(90.0, 0.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 90.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(90.0, 20.0))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(20.0)])
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(90.0, 20.0))
                    .child(
                        GridLayoutNode::subgrid(GridArea::new(2, 1, 1, 1))
                            .expect_layout(Point::new(0.0, 0.0), Size::new(90.0, 20.0))
                            .child(
                                GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                                    .measurement(Size::new(90.0, 10.0))
                                    .expect_layout(Point::new(0.0, 0.0), Size::new(90.0, 10.0)),
                            ),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_traversal_reversed_nested_inherited_subgrid_maps_to_mirrored_track() {
        let expected_columns = TrackSizingSlice::definite_columns(80.0, 0.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(3, 1, 1, 1), 80.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(80.0, 20.0))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(20.0)])
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                    .direction(Direction::Rtl)
                    .expect_layout(Point::new(0.0, 0.0), Size::new(80.0, 20.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                            .measurement(Size::new(80.0, 10.0))
                            .expect_layout(Point::new(0.0, 0.0), Size::new(80.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_traversal_nested_margin_border_padding_increases_contribution() {
        let expected_columns = TrackSizingSlice::definite_columns(85.0, 0.0)
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(1, 1, 1, 1), 85.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(85.0, 20.0))
            .columns(vec![TrackComponent::AUTO])
            .rows(vec![TrackComponent::px(20.0)])
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(1, 1, 1, 1))
                    .margin(Edges::new(
                        LengthAuto::px(0.0),
                        LengthAuto::px(8.0),
                        LengthAuto::px(0.0),
                        LengthAuto::px(5.0),
                    ))
                    .border(Edges::new(
                        Length::ZERO,
                        Length::px(9.0),
                        Length::ZERO,
                        Length::px(6.0),
                    ))
                    .padding(Edges::new(
                        Length::ZERO,
                        Length::px(10.0),
                        Length::ZERO,
                        Length::px(7.0),
                    ))
                    .expect_layout(Point::new(5.0, 0.0), Size::new(72.0, 20.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                            .measurement(Size::new(40.0, 10.0))
                            .expect_layout(Point::new(13.0, 0.0), Size::new(40.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_traversal_gap_difference_adjustment_accumulates_through_nested_subgrids() {
        let expected_columns = TrackSizingSlice::definite_columns(70.0, 10.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 50.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(70.0, 20.0))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(20.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                    .gap(Size::new(Length::px(20.0), Length::ZERO))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(70.0, 20.0))
                    .child(
                        GridLayoutNode::subgrid(GridArea::new(2, 1, 1, 1))
                            .gap(Size::new(Length::px(28.0), Length::ZERO))
                            .expect_layout(Point::new(15.0, 0.0), Size::new(60.0, 20.0))
                            .child(
                                GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                                    .measurement(Size::new(40.0, 10.0))
                                    .expect_layout(Point::new(0.0, 0.0), Size::new(40.0, 10.0)),
                            ),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_traversal_direct_leaf_uses_internal_gap_adjustment() {
        let expected_columns = TrackSizingSlice::definite_columns(70.0, 10.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 50.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(70.0, 20.0))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(20.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                    .gap(Size::new(Length::px(20.0), Length::ZERO))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(70.0, 20.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .measurement(Size::new(40.0, 10.0))
                            .expect_layout(Point::new(15.0, 0.0), Size::new(40.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_traversal_unsupported_sibling_does_not_drop_valid_contribution() {
        let expected_columns = TrackSizingSlice::definite_columns(140.0, 10.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(1, 1, 1, 1), 30.0))
            .item(intrinsic_item(GridArea::new(3, 1, 1, 1), 90.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(140.0, 20.0))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(20.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(1, 1, 1, 1)).child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .measurement(Size::new(30.0, 10.0))
                        .expect_layout(Point::new(0.0, 0.0), Size::new(30.0, 10.0)),
                ),
            )
            .node(
                GridLayoutNode::subgrid(GridArea::new(3, 1, 1, 1))
                    .writing_mode(WritingMode::VerticalRl)
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                            .measurement(Size::new(90.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_traversal_percent_padding_uses_definite_area_basis() {
        let expected_columns = TrackSizingSlice::definite_columns(100.0, 0.0)
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(1, 1, 1, 1), 60.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(100.0, 20.0))
            .columns(vec![TrackComponent::AUTO])
            .rows(vec![TrackComponent::px(20.0)])
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(1, 1, 1, 1))
                    .padding(Edges::new(
                        Length::ZERO,
                        Length::percent(0.1),
                        Length::ZERO,
                        Length::percent(0.1),
                    ))
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                            .measurement(Size::new(40.0, 10.0))
                            .expect_layout(Point::new(6.0, 0.0), Size::new(40.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_traversal_percent_gap_uses_definite_content_box_basis() {
        let expected_columns = TrackSizingSlice::definite_columns(100.0, 10.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 50.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(100.0, 20.0))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(20.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                    .gap(Size::new(Length::percent(0.2), Length::ZERO))
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .measurement(Size::new(40.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_traversal_translated_nested_edge_adjustments_land_on_ancestor_tracks() {
        let expected_columns = TrackSizingSlice::definite_columns(48.0, 0.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(3, 1, 1, 1), 48.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(48.0, 20.0))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(20.0)])
            .expected_tracks(expected_columns, fixed_rows(20.0))
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 3, 1))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(48.0, 20.0))
                    .child(
                        GridLayoutNode::subgrid(GridArea::new(2, 1, 1, 1))
                            .margin(Edges::new(
                                LengthAuto::px(0.0),
                                LengthAuto::px(5.0),
                                LengthAuto::px(0.0),
                                LengthAuto::px(3.0),
                            ))
                            .expect_layout(Point::new(3.0, 0.0), Size::new(40.0, 20.0))
                            .child(
                                GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                                    .measurement(Size::new(40.0, 10.0))
                                    .expect_layout(Point::new(0.0, 0.0), Size::new(40.0, 10.0)),
                            ),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_absolute_descendant_uses_existing_static_position_behavior() {
        let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .track(GridTrack::fixed(60.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .container(Size::new(200.0, 30.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(80.0),
                TrackComponent::px(60.0),
            ])
            .rows(vec![TrackComponent::px(30.0)])
            .gap(Size::new(10.0, 0.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                    .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .position(Position::Absolute)
                            .size(Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)))
                            .expect_layout(Point::new(90.0, 0.0), Size::new(10.0, 10.0))
                            .expect_final_layout(Point::new(90.0, 0.0), Size::new(10.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn subgrid_named_placement_clamp_matches_oracle() {
        let oracle = grid::resolve_named_subgrid_axis_placement(
            &grid::NamedGridLines::empty(GridAxis::Column, 1),
            grid::NamedAxisPlacement {
                start: grid::NamedGridLine::Number(2),
                end: grid::NamedGridLine::Span {
                    name: None,
                    count: 3,
                },
            },
            None,
        )
        .unwrap();

        let mut tree = OracleTree::new()
            .children(1, [2])
            .children(2, [3])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
                    grid_template_columns: vec![TrackComponent::px(40.0)],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 2).expect("valid grid placement"),
                    grid_row: GridPlacement::try_line(1).expect("valid grid placement"),
                    grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                3,
                NodeInput {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::Line(2),
                        RawGridLine::Span(3),
                    ),
                    raw_grid_row: RawGridPlacement::line(1),
                    ..NodeInput::DEFAULT
                },
            );

        compute_root(
            &mut tree,
            1,
            Size::new(Available::Definite(40.0), Available::Definite(20.0)),
        )
        .unwrap();
        round_layout(&mut tree, 1).unwrap();

        let child = tree
            .final_layout(3)
            .expect("subgrid child should be laid out");
        assert_eq!(oracle.clamped.resolved.start_line, 1);
        assert_eq!(oracle.clamped.resolved.end_line, 2);
        assert_eq!(child.location.x, 0.0);
        assert_eq!(
            child.size.width,
            (oracle.clamped.resolved.end_line - oracle.clamped.resolved.start_line) as f32 * 40.0
        );
    }

    #[test]
    fn oracle_layout_harness_can_compare_lane_reports() {
        let placement_report = grid::place_lanes(LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: LaneAutoFlow::Row,
            lane_gap: 4.0,
            tolerance: LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 100.0,
            items: vec![
                LaneItemInput::definite("a", 1, 1, 20.0),
                LaneItemInput::auto("b", 1, 10.0),
            ],
        })
        .unwrap();
        let intrinsic_report = grid::lane_intrinsic_sizing(LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(120.0),
            gap: 0.0,
            tracks: vec![GridTrack::auto(), GridTrack::fixed(40.0)],
            content_sized_tracks: vec![0],
            items: vec![LaneIntrinsicItem::indefinite(
                "b",
                oracle_lane_span(1),
                ItemContributionFacts {
                    area: GridArea::new(1, 1, 1, 1),
                    min_content: 24.0,
                    max_content: 30.0,
                    preferred: ContributionSize::Auto,
                    min_size: ContributionSize::Definite(18.0),
                    max_size: ContributionSize::Infinite,
                    margin_before: 0.0,
                    margin_after: 0.0,
                    automatic_minimum_applies: false,
                },
            )],
        })
        .unwrap();

        GridLayoutComparison::new()
            .expect_lane_placement_report(placement_report.clone())
            .expect_lane_intrinsic_report(intrinsic_report.clone())
            .assert_lane_reports(&[placement_report], &[intrinsic_report]);
    }

    #[test]
    fn lanes_row_auto_flow_matches_oracle_placement() {
        assert_production_lane_placement_matches_oracle(
            ProductionLanePlacementInput {
                grid_axis_tracks: 3,
                auto_flow: GridAutoFlow::Row,
                lane_gap: 10.0,
                tolerance: GridFlowTolerance::Length(Length::px(0.0)),
                tolerance_basis: 0.0,
                items: vec![
                    production_auto_lane_item("a", 1, 20.0),
                    production_auto_lane_item("b", 1, 30.0),
                    production_auto_lane_item("c", 2, 10.0),
                ],
            },
            LanePlacementInput {
                grid_axis_tracks: 3,
                auto_flow: LaneAutoFlow::Row,
                lane_gap: 10.0,
                tolerance: LaneFlowTolerance::Fixed(0.0),
                tolerance_basis: 0.0,
                items: vec![
                    LaneItemInput::auto("a", 1, 20.0),
                    LaneItemInput::auto("b", 1, 30.0),
                    LaneItemInput::auto("c", 2, 10.0),
                ],
            },
        );
    }

    #[test]
    fn lanes_column_auto_flow_matches_oracle_placement() {
        assert_production_lane_placement_matches_oracle(
            ProductionLanePlacementInput {
                grid_axis_tracks: 2,
                auto_flow: GridAutoFlow::Column,
                lane_gap: 4.0,
                tolerance: GridFlowTolerance::Length(Length::px(0.0)),
                tolerance_basis: 0.0,
                items: vec![
                    production_auto_lane_item("a", 1, 10.0),
                    production_auto_lane_item("b", 1, 20.0),
                    production_auto_lane_item("c", 1, 30.0),
                ],
            },
            LanePlacementInput {
                grid_axis_tracks: 2,
                auto_flow: LaneAutoFlow::Column,
                lane_gap: 4.0,
                tolerance: LaneFlowTolerance::Fixed(0.0),
                tolerance_basis: 0.0,
                items: vec![
                    LaneItemInput::auto("a", 1, 10.0),
                    LaneItemInput::auto("b", 1, 20.0),
                    LaneItemInput::auto("c", 1, 30.0),
                ],
            },
        );
    }

    #[test]
    fn lanes_definite_grid_axis_item_matches_oracle_placement() {
        assert_production_lane_placement_matches_oracle(
            ProductionLanePlacementInput {
                grid_axis_tracks: 3,
                auto_flow: GridAutoFlow::Row,
                lane_gap: 5.0,
                tolerance: GridFlowTolerance::Length(Length::px(0.0)),
                tolerance_basis: 0.0,
                items: vec![
                    production_definite_lane_item("a", 2, 2, 40.0),
                    production_auto_lane_item("b", 1, 20.0),
                ],
            },
            LanePlacementInput {
                grid_axis_tracks: 3,
                auto_flow: LaneAutoFlow::Row,
                lane_gap: 5.0,
                tolerance: LaneFlowTolerance::Fixed(0.0),
                tolerance_basis: 0.0,
                items: vec![
                    LaneItemInput::definite("a", 2, 2, 40.0),
                    LaneItemInput::auto("b", 1, 20.0),
                ],
            },
        );
    }

    #[test]
    fn lanes_auto_span_clamping_matches_oracle_placement() {
        assert_production_lane_placement_matches_oracle(
            ProductionLanePlacementInput {
                grid_axis_tracks: 2,
                auto_flow: GridAutoFlow::Row,
                lane_gap: 0.0,
                tolerance: GridFlowTolerance::Length(Length::px(0.0)),
                tolerance_basis: 0.0,
                items: vec![production_auto_lane_item("a", 7, 10.0)],
            },
            LanePlacementInput {
                grid_axis_tracks: 2,
                auto_flow: LaneAutoFlow::Row,
                lane_gap: 0.0,
                tolerance: LaneFlowTolerance::Fixed(0.0),
                tolerance_basis: 0.0,
                items: vec![LaneItemInput::auto("a", 7, 10.0)],
            },
        );
    }

    #[test]
    fn lanes_finite_tolerance_matches_oracle_placement() {
        assert_production_lane_placement_matches_oracle(
            ProductionLanePlacementInput {
                grid_axis_tracks: 3,
                auto_flow: GridAutoFlow::Row,
                lane_gap: 0.0,
                tolerance: GridFlowTolerance::Length(Length::px(10.0)),
                tolerance_basis: 0.0,
                items: vec![
                    production_definite_lane_item("a", 1, 1, 10.0),
                    production_definite_lane_item("b", 2, 1, 20.0),
                    production_auto_lane_item("c", 1, 10.0),
                ],
            },
            LanePlacementInput {
                grid_axis_tracks: 3,
                auto_flow: LaneAutoFlow::Row,
                lane_gap: 0.0,
                tolerance: LaneFlowTolerance::Fixed(10.0),
                tolerance_basis: 0.0,
                items: vec![
                    LaneItemInput::definite("a", 1, 1, 10.0),
                    LaneItemInput::definite("b", 2, 1, 20.0),
                    LaneItemInput::auto("c", 1, 10.0),
                ],
            },
        );
    }

    #[test]
    fn lanes_finite_search_does_not_wrap_candidate_span_across_grid_axis_end() {
        assert_production_lane_placement_matches_oracle(
            ProductionLanePlacementInput {
                grid_axis_tracks: 3,
                auto_flow: GridAutoFlow::Row,
                lane_gap: 0.0,
                tolerance: GridFlowTolerance::Length(Length::px(0.0)),
                tolerance_basis: 0.0,
                items: vec![
                    production_auto_lane_item("a", 2, 10.0),
                    production_auto_lane_item("b", 2, 10.0),
                ],
            },
            LanePlacementInput {
                grid_axis_tracks: 3,
                auto_flow: LaneAutoFlow::Row,
                lane_gap: 0.0,
                tolerance: LaneFlowTolerance::Fixed(0.0),
                tolerance_basis: 0.0,
                items: vec![
                    LaneItemInput::auto("a", 2, 10.0),
                    LaneItemInput::auto("b", 2, 10.0),
                ],
            },
        );
    }

    #[test]
    fn lanes_infinite_tolerance_matches_oracle_placement() {
        assert_production_lane_placement_matches_oracle(
            ProductionLanePlacementInput {
                grid_axis_tracks: 2,
                auto_flow: GridAutoFlow::Column,
                lane_gap: 0.0,
                tolerance: GridFlowTolerance::Infinite,
                tolerance_basis: 0.0,
                items: vec![
                    production_auto_lane_item("a", 1, 10.0),
                    production_auto_lane_item("b", 1, 10.0),
                    production_auto_lane_item("c", 1, 10.0),
                ],
            },
            LanePlacementInput {
                grid_axis_tracks: 2,
                auto_flow: LaneAutoFlow::Column,
                lane_gap: 0.0,
                tolerance: LaneFlowTolerance::Infinite,
                tolerance_basis: 0.0,
                items: vec![
                    LaneItemInput::auto("a", 1, 10.0),
                    LaneItemInput::auto("b", 1, 10.0),
                    LaneItemInput::auto("c", 1, 10.0),
                ],
            },
        );
    }

    #[test]
    fn lanes_intrinsic_groups_equivalent_items_without_source_offset() {
        let production_facts = production_lane_facts(20.0, 50.0);
        let report = production_lane_intrinsic_sizing(ProductionLaneIntrinsicSizingInput {
            axis: ProductionGridAxisKind::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![
                ProductionTrackSizing::AUTO,
                ProductionTrackSizing::AUTO,
                ProductionTrackSizing::AUTO,
            ],
            content_sized_tracks: vec![0, 1, 2],
            items: vec![
                ProductionLaneIntrinsicItem::indefinite(
                    "a",
                    production_lane_span(2),
                    production_facts,
                ),
                ProductionLaneIntrinsicItem::indefinite(
                    "b",
                    production_lane_span(2),
                    ProductionLaneContributionFacts {
                        min_content: 30.0,
                        max_content: 60.0,
                        ..production_facts
                    },
                ),
            ],
        })
        .expect("equivalent intrinsic values are finite")
        .expect("equivalent intrinsic spans are valid");

        assert_eq!(report.indefinite_groups.len(), 1);
        assert_eq!(report.indefinite_groups[0].item_ids, ["a", "b"]);
        assert_eq!(report.converted_indefinite_items.len(), 2);
        assert_eq!(report.final_track_sizes, [10.0, 10.0, 10.0]);
    }

    #[test]
    fn lanes_intrinsic_skips_definite_items_outside_content_sized_tracks() {
        let facts = oracle_lane_facts(80.0, 120.0);
        let production_facts = production_lane_facts(80.0, 120.0);
        assert_production_lane_intrinsic_matches_oracle(
            ProductionLaneIntrinsicSizingInput {
                axis: ProductionGridAxisKind::Column,
                available: Some(200.0),
                gap: 10.0,
                tracks: vec![ProductionTrackSizing::AUTO, ProductionTrackSizing::AUTO],
                content_sized_tracks: vec![1],
                items: vec![
                    ProductionLaneIntrinsicItem::definite(
                        "a",
                        crate::LaneTrackSpan::new(1, 2),
                        production_facts,
                    )
                    .expect("valid production lane item"),
                ],
            },
            LaneIntrinsicSizingInput {
                axis: GridAxis::Column,
                available: Some(200.0),
                gap: 10.0,
                tracks: vec![GridTrack::auto(), GridTrack::auto()],
                content_sized_tracks: vec![1],
                items: vec![
                    LaneIntrinsicItem::definite("a", grid::TrackSpan::new(1, 2), facts)
                        .expect("valid oracle lane item"),
                ],
            },
        );
    }

    #[test]
    fn lanes_intrinsic_projects_disjoint_content_sized_spans_like_oracle() {
        let facts = ItemContributionFacts {
            automatic_minimum_applies: false,
            min_size: ContributionSize::Definite(100.0),
            ..oracle_lane_facts(120.0, 160.0)
        };
        let production_facts = ProductionLaneContributionFacts {
            automatic_minimum_applies: false,
            min_size: 100.0,
            ..production_lane_facts(120.0, 160.0)
        };
        assert_production_lane_intrinsic_matches_oracle(
            ProductionLaneIntrinsicSizingInput {
                axis: ProductionGridAxisKind::Column,
                available: Some(300.0),
                gap: 10.0,
                tracks: vec![
                    ProductionTrackSizing::AUTO,
                    ProductionTrackSizing::px(20.0),
                    ProductionTrackSizing::AUTO,
                ],
                content_sized_tracks: vec![0, 2],
                items: vec![ProductionLaneIntrinsicItem::indefinite(
                    "a",
                    production_lane_span(3),
                    production_facts,
                )],
            },
            LaneIntrinsicSizingInput {
                axis: GridAxis::Column,
                available: Some(300.0),
                gap: 10.0,
                tracks: vec![GridTrack::auto(), GridTrack::fixed(20.0), GridTrack::auto()],
                content_sized_tracks: vec![0, 2],
                items: vec![LaneIntrinsicItem::indefinite(
                    "a",
                    oracle_lane_span(3),
                    facts,
                )],
            },
        );
    }

    #[test]
    fn lanes_intrinsic_clamps_oversized_indefinite_spans_like_oracle() {
        let facts = oracle_lane_facts(30.0, 60.0);
        let production_facts = production_lane_facts(30.0, 60.0);
        assert_production_lane_intrinsic_matches_oracle(
            ProductionLaneIntrinsicSizingInput {
                axis: ProductionGridAxisKind::Column,
                available: Some(300.0),
                gap: 10.0,
                tracks: vec![ProductionTrackSizing::AUTO, ProductionTrackSizing::AUTO],
                content_sized_tracks: vec![0, 1],
                items: vec![ProductionLaneIntrinsicItem::indefinite(
                    "a",
                    production_lane_span(5),
                    production_facts,
                )],
            },
            LaneIntrinsicSizingInput {
                axis: GridAxis::Column,
                available: Some(300.0),
                gap: 10.0,
                tracks: vec![GridTrack::auto(), GridTrack::auto()],
                content_sized_tracks: vec![0, 1],
                items: vec![LaneIntrinsicItem::indefinite(
                    "a",
                    oracle_lane_span(5),
                    facts,
                )],
            },
        );
    }

    #[test]
    fn lanes_intrinsic_preserves_min_content_track_behavior() {
        let facts = ItemContributionFacts {
            automatic_minimum_applies: false,
            min_size: ContributionSize::Definite(12.0),
            ..oracle_lane_facts(100.0, 120.0)
        };
        let production_facts = ProductionLaneContributionFacts {
            automatic_minimum_applies: false,
            min_size: 12.0,
            ..production_lane_facts(100.0, 120.0)
        };
        assert_production_lane_intrinsic_matches_oracle(
            ProductionLaneIntrinsicSizingInput {
                axis: ProductionGridAxisKind::Column,
                available: Some(300.0),
                gap: 10.0,
                tracks: vec![ProductionTrackSizing::new(
                    MinTrackSizing::MIN_CONTENT,
                    MaxTrackSizing::MAX_CONTENT,
                )],
                content_sized_tracks: vec![0],
                items: vec![ProductionLaneIntrinsicItem::indefinite(
                    "a",
                    production_lane_span(1),
                    production_facts,
                )],
            },
            LaneIntrinsicSizingInput {
                axis: GridAxis::Column,
                available: Some(300.0),
                gap: 10.0,
                tracks: vec![GridTrack::new(TrackMin::MinContent, TrackMax::MaxContent)],
                content_sized_tracks: vec![0],
                items: vec![LaneIntrinsicItem::indefinite(
                    "a",
                    oracle_lane_span(1),
                    facts,
                )],
            },
        );
    }

    #[test]
    fn lanes_content_size_contributes_to_indefinite_container_size() {
        let expected_columns = TrackSizingSlice::indefinite_columns(0.0)
            .track(GridTrack::fixed(40.0))
            .solve();
        let expected_rows = TrackSizingSlice::indefinite_rows(8.0)
            .track(GridTrack::fixed(10.0))
            .solve();

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(40.0, 0.0))
            .root_size(Size::new(PreferredSize::px(40.0), PreferredSize::AUTO))
            .columns(vec![TrackComponent::px(40.0)])
            .rows(vec![TrackComponent::px(10.0)])
            .gap(Size::new(0.0, 8.0))
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                    .measurement(Size::new(20.0, 30.0))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(20.0, 30.0)),
            )
            .node(
                GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                    .measurement(Size::new(20.0, 50.0))
                    .expect_layout(Point::new(0.0, 38.0), Size::new(20.0, 50.0)),
            )
            .assert_layout_size(Size::new(40.0, 88.0));
    }

    #[test]
    fn lanes_child_measurement_uses_resolved_grid_axis_span_size() {
        let expected_columns = TrackSizingSlice::definite_columns(100.0, 0.0)
            .track(GridTrack::fixed(100.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(10.0, 0.0)
            .track(GridTrack::fixed(10.0))
            .solve();
        let mut tree = OracleTree::new()
            .children(1, [2])
            .style(
                1,
                NodeInput {
                    display: Display::GridLanes,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    grid_template_columns: vec![TrackComponent::px(100.0)],
                    grid_template_rows: vec![TrackComponent::px(10.0)],
                    ..NodeInput::default()
                },
            )
            .style(2, NodeInput::default())
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 60.0),
                    Size::new(100.0, 60.0),
                ))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(Some(100.0), None))
                .parent(Size::new(Some(100.0), Some(100.0)))
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::MAX_CONTENT,
                )),
            )
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 60.0),
                    Size::new(100.0, 60.0),
                ))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(Some(100.0), None))
                .parent(Size::new(Some(100.0), Some(0.0)))
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::MAX_CONTENT,
                )),
            )
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 60.0),
                    Size::new(100.0, 60.0),
                ))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(Some(100.0), Some(60.0)))
                .parent(Size::new(Some(100.0), Some(100.0)))
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::Definite(100.0),
                )),
            );

        let output = crate::compute_grid(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(100.0), Some(100.0)),
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

        assert_eq!(output.size, Size::new(100.0, 100.0));
        let child = tree.layout(2).expect("lane child layout must be recorded");
        assert_eq!(child.location, Point::new(0.0, 0.0));
        assert_eq!(child.size, Size::new(100.0, 60.0));
        let compute_size_inputs = tree
            .inputs(2)
            .iter()
            .filter(|input| input.run_mode() == RunMode::ComputeSize)
            .collect::<Vec<_>>();
        assert!(
            compute_size_inputs.iter().any(|input| {
                input.known() == Size::new(Some(100.0), None)
                    && input.parent() == Size::new(Some(100.0), Some(100.0))
                    && input.available()
                        == Size::new(Available::Definite(100.0), Available::MAX_CONTENT)
            }),
            "lane placement should measure child against resolved grid-axis span: {compute_size_inputs:#?}"
        );

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(100.0, 100.0))
            .columns(vec![TrackComponent::px(100.0)])
            .rows(vec![TrackComponent::px(10.0)])
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                    .measurement(Size::new(100.0, 60.0))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(100.0, 60.0)),
            )
            .assert_layout();
    }

    #[test]
    fn lanes_auto_child_measurement_uses_final_auto_placement_span() {
        let mut tree = OracleTree::new()
            .children(1, [2, 3, 4])
            .style(
                1,
                NodeInput {
                    display: Display::GridLanes,
                    size: Size::new(PreferredSize::px(140.0), PreferredSize::px(140.0)),
                    grid_template_columns: vec![
                        TrackComponent::px(40.0),
                        TrackComponent::px(100.0),
                    ],
                    grid_template_rows: vec![TrackComponent::px(10.0)],
                    grid_flow_tolerance: GridFlowTolerance::Length(Length::px(0.0)),
                    ..NodeInput::default()
                },
            )
            .style(2, NodeInput::default())
            .style(3, NodeInput::default())
            .style(4, NodeInput::default());

        tree = tree
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(40.0, 100.0),
                    Size::new(40.0, 100.0),
                ))
                .run_mode(RunMode::ComputeSize)
                .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
            )
            .measure_when(
                3,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 10.0),
                    Size::new(100.0, 10.0),
                ))
                .run_mode(RunMode::ComputeSize)
                .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
            )
            .measure_when(
                4,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(40.0, 100.0),
                    Size::new(40.0, 100.0),
                ))
                .run_mode(RunMode::ComputeSize)
                .available(Size::new(Available::Definite(40.0), Available::MAX_CONTENT)),
            )
            .measure_when(
                4,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 10.0),
                    Size::new(100.0, 10.0),
                ))
                .run_mode(RunMode::ComputeSize)
                .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
            )
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(40.0, 100.0),
                    Size::new(40.0, 100.0),
                ))
                .available(Size::new(Available::Definite(40.0), Available::MAX_CONTENT)),
            )
            .measure_when(
                3,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 10.0),
                    Size::new(100.0, 10.0),
                ))
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::MAX_CONTENT,
                )),
            )
            .measure_when(
                4,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 10.0),
                    Size::new(100.0, 10.0),
                ))
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::MAX_CONTENT,
                )),
            )
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(40.0, 100.0),
                    Size::new(40.0, 100.0),
                ))
                .run_mode(RunMode::PerformLayout)
                .available(Size::new(
                    Available::Definite(40.0),
                    Available::Definite(140.0),
                )),
            )
            .measure_when(
                3,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 10.0),
                    Size::new(100.0, 10.0),
                ))
                .run_mode(RunMode::PerformLayout)
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::Definite(140.0),
                )),
            )
            .measure_when(
                4,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(100.0, 10.0),
                    Size::new(100.0, 10.0),
                ))
                .run_mode(RunMode::PerformLayout)
                .available(Size::new(
                    Available::Definite(100.0),
                    Available::Definite(140.0),
                )),
            );

        crate::compute_grid(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(140.0), Some(140.0)),
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

        let first = tree.layout(2).expect("first child layout");
        let second = tree.layout(3).expect("second child layout");
        let third = tree.layout(4).expect("third child layout");
        assert_eq!(first.location, Point::new(0.0, 0.0));
        assert_eq!(first.size, Size::new(40.0, 100.0));
        assert_eq!(second.location, Point::new(40.0, 0.0));
        assert_eq!(second.size, Size::new(100.0, 10.0));
        assert_eq!(third.location, Point::new(40.0, 10.0));
        assert_eq!(third.size, Size::new(100.0, 10.0));

        let third_compute_size_inputs = tree
            .inputs(4)
            .iter()
            .filter(|input| input.run_mode() == RunMode::ComputeSize)
            .collect::<Vec<_>>();
        assert!(
            third_compute_size_inputs
                .iter()
                .any(|input| input.available().width == Available::Definite(100.0)),
            "third auto lane item should be measured against its final 100px column: {third_compute_size_inputs:#?}"
        );
    }

    #[test]
    fn lanes_spanning_child_measurement_uses_distributed_grid_axis_gap() {
        let mut tree = OracleTree::new()
            .children(1, [2])
            .style(
                1,
                NodeInput {
                    display: Display::GridLanes,
                    size: Size::new(PreferredSize::px(120.0), PreferredSize::px(120.0)),
                    grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                    grid_template_rows: vec![TrackComponent::px(10.0)],
                    justify_content: Some(AlignContent::SpaceBetween),
                    ..NodeInput::default()
                },
            )
            .style(
                2,
                NodeInput {
                    grid_column: crate::GridPlacement::try_span(2).expect("valid grid placement"),
                    ..NodeInput::default()
                },
            );
        tree = tree
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(80.0, 40.0),
                    Size::new(80.0, 40.0),
                ))
                .run_mode(RunMode::ComputeSize)
                .available(Size::new(Available::Definite(80.0), Available::MAX_CONTENT)),
            )
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(120.0, 40.0),
                    Size::new(120.0, 40.0),
                ))
                .run_mode(RunMode::ComputeSize)
                .available(Size::new(
                    Available::Definite(120.0),
                    Available::MAX_CONTENT,
                )),
            )
            .measure_when(
                2,
                OracleMeasurement::new(ComputeOutput::from_sizes(
                    Size::new(120.0, 40.0),
                    Size::new(120.0, 40.0),
                ))
                .run_mode(RunMode::PerformLayout)
                .available(Size::new(
                    Available::Definite(120.0),
                    Available::Definite(120.0),
                )),
            );

        crate::compute_grid(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(120.0), Some(120.0)),
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

        let child = tree.layout(2).expect("spanning lane child layout");
        assert_eq!(child.location, Point::new(0.0, 0.0));
        assert_eq!(child.size, Size::new(120.0, 40.0));
        let compute_size_inputs = tree
            .inputs(2)
            .iter()
            .filter(|input| input.run_mode() == RunMode::ComputeSize)
            .collect::<Vec<_>>();
        assert!(
            compute_size_inputs
                .iter()
                .any(|input| input.available().width == Available::Definite(120.0)),
            "spanning lane child should be measured against distributed 120px grid-axis span: {compute_size_inputs:#?}"
        );
    }

    #[test]
    fn lanes_absolute_child_uses_grid_absolute_layout() {
        let expected_columns = TrackSizingSlice::definite_columns(100.0, 0.0)
            .track(GridTrack::fixed(100.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(40.0, 0.0)
            .track(GridTrack::fixed(40.0))
            .solve();

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(100.0, 40.0))
            .columns(vec![TrackComponent::px(100.0)])
            .rows(vec![TrackComponent::px(40.0)])
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                    .position(Position::Absolute)
                    .size(Size::new(PreferredSize::px(24.0), PreferredSize::px(12.0)))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(24.0, 12.0)),
            )
            .assert_layout();
    }

    #[test]
    fn fri08_c03_nested_empty_wrapper_does_not_substitute_its_zero_box() {
        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(0.0, 10.0))
            .root_size(Size::new(PreferredSize::AUTO, PreferredSize::px(10.0)))
            .columns(vec![TrackComponent::AUTO])
            .rows(vec![TrackComponent::px(10.0)])
            .expected_tracks(
                TrackSizingSlice::indefinite_columns(0.0)
                    .track(GridTrack::auto())
                    .solve(),
                TrackSizingSlice::definite_rows(10.0, 0.0)
                    .track(GridTrack::fixed(10.0))
                    .solve(),
            )
            .node(
                GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                    .display(crate::Display::GridLanes)
                    .columns(vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })])
                    .rows(vec![TrackComponent::px(10.0)])
                    .measurement(Size::new(90.0, 10.0))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(0.0, 10.0)),
            )
            .assert_layout_size(Size::new(0.0, 10.0));
    }

    #[test]
    fn fri08_c03_nested_automatic_wrapper_projects_descendant_intrinsic_size() {
        let expected_columns = TrackSizingSlice::indefinite_columns(0.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 90.0))
            .item(intrinsic_item(GridArea::new(3, 1, 1, 1), 90.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(10.0, 0.0)
            .track(GridTrack::fixed(10.0))
            .solve();

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(0.0, 10.0))
            .root_size(Size::new(PreferredSize::AUTO, PreferredSize::px(10.0)))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(10.0)])
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 2, 1), 2, 1)
                    .display(crate::Display::GridLanes)
                    .columns(vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })])
                    .rows(vec![TrackComponent::px(10.0)])
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .measurement(Size::new(90.0, 10.0)),
                    ),
            )
            .assert_layout_size(Size::new(180.0, 10.0));
    }

    #[test]
    fn fri08_c03_nested_definite_wrapper_bounds_automatic_descendant_candidates() {
        let expected_columns = TrackSizingSlice::indefinite_columns(0.0)
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .track(GridTrack::auto())
            .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 90.0))
            .item(intrinsic_item(GridArea::new(3, 1, 1, 1), 90.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(10.0, 0.0)
            .track(GridTrack::fixed(10.0))
            .solve();

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(0.0, 10.0))
            .root_size(Size::new(PreferredSize::AUTO, PreferredSize::px(10.0)))
            .columns(vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
            ])
            .rows(vec![TrackComponent::px(10.0)])
            .expected_tracks(expected_columns, expected_rows)
            .node(
                GridLayoutNode::item(GridArea::new(2, 1, 2, 1))
                    .display(crate::Display::GridLanes)
                    .columns(vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })])
                    .rows(vec![TrackComponent::px(10.0)])
                    .child(
                        GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                            .measurement(Size::new(90.0, 10.0)),
                    ),
            )
            .assert_layout_size(Size::new(180.0, 10.0));
    }

    #[test]
    fn lanes_child_subgrid_inherits_grid_axis_tracks() {
        let expected_columns = TrackSizingSlice::definite_columns(120.0, 0.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(120.0, 30.0))
            .columns(vec![TrackComponent::px(40.0), TrackComponent::px(80.0)])
            .rows(vec![TrackComponent::px(30.0)])
            .expected_tracks(expected_columns, expected_rows)
            .auto_flow(GridAutoFlow::Row)
            .node(
                GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 2, 1), 2, 1)
                    .display(crate::Display::Grid)
                    .columns(vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })])
                    .rows(vec![TrackComponent::px(30.0)])
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .measurement(Size::new(12.0, 10.0))
                            .expect_layout(Point::new(40.0, 0.0), Size::new(12.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn lanes_column_flow_child_subgrid_inherits_row_axis_tracks() {
        let expected_columns = TrackSizingSlice::definite_columns(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(120.0, 0.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .solve();

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(30.0, 120.0))
            .columns(vec![TrackComponent::px(30.0)])
            .rows(vec![TrackComponent::px(40.0), TrackComponent::px(80.0)])
            .expected_tracks(expected_columns, expected_rows)
            .auto_flow(GridAutoFlow::Column)
            .node(
                GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 1, 2), 1, 2)
                    .display(crate::Display::Grid)
                    .columns(vec![TrackComponent::px(30.0)])
                    .rows(vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })])
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 2, 1, 1))
                            .measurement(Size::new(12.0, 10.0))
                            .expect_layout(Point::new(0.0, 40.0), Size::new(12.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn lanes_child_subgrid_uses_report_matching_child_order_after_skipped_siblings() {
        let expected_columns = TrackSizingSlice::definite_columns(120.0, 0.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(80.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
            .track(GridTrack::fixed(30.0))
            .solve();

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(120.0, 30.0))
            .columns(vec![TrackComponent::px(40.0), TrackComponent::px(80.0)])
            .rows(vec![TrackComponent::px(30.0)])
            .expected_tracks(expected_columns, expected_rows)
            .auto_flow(GridAutoFlow::Row)
            .node(
                GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                    .display(crate::Display::None)
                    .expect_layout(Point::new(0.0, 0.0), Size::ZERO),
            )
            .node(
                GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                    .position(Position::Absolute)
                    .size(Size::new(PreferredSize::px(8.0), PreferredSize::px(6.0)))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(8.0, 6.0)),
            )
            .node(
                GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 2, 1), 2, 1)
                    .display(crate::Display::Grid)
                    .columns(vec![TrackComponent::Subgrid(crate::SubgridTrack {
                        name_components: Vec::new(),
                    })])
                    .rows(vec![TrackComponent::px(30.0)])
                    .child(
                        GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                            .measurement(Size::new(12.0, 10.0))
                            .expect_layout(Point::new(40.0, 0.0), Size::new(12.0, 10.0)),
                    ),
            )
            .assert_layout();
    }

    #[test]
    fn lanes_definite_lane_axis_container_lays_out_children_at_lane_offsets() {
        let expected_columns = TrackSizingSlice::definite_columns(120.0, 0.0)
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(40.0))
            .track(GridTrack::fixed(40.0))
            .solve();
        let expected_rows = TrackSizingSlice::definite_rows(90.0, 6.0)
            .track(GridTrack::fixed(10.0))
            .solve();

        GridLayoutComparison::new()
            .root_display(crate::Display::GridLanes)
            .container(Size::new(120.0, 90.0))
            .columns(vec![
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
                TrackComponent::px(40.0),
            ])
            .rows(vec![TrackComponent::px(10.0)])
            .gap(Size::new(0.0, 6.0))
            .expected_tracks(expected_columns, expected_rows)
            .auto_flow(GridAutoFlow::Row)
            .node(
                GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 2, 1), 2, 1)
                    .measurement(Size::new(20.0, 30.0))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(20.0, 30.0)),
            )
            .node(
                GridLayoutNode::auto_item(GridArea::new(3, 1, 1, 1))
                    .measurement(Size::new(20.0, 20.0))
                    .expect_layout(Point::new(80.0, 0.0), Size::new(20.0, 20.0)),
            )
            .node(
                GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                    .measurement(Size::new(20.0, 15.0))
                    .expect_layout(Point::new(0.0, 36.0), Size::new(20.0, 15.0)),
            )
            .assert_layout();
    }

    fn assert_production_lane_placement_matches_oracle(
        production_input: ProductionLanePlacementInput<&'static str>,
        oracle_input: LanePlacementInput,
    ) {
        let production = production_place_lanes(production_input).unwrap();
        let oracle = grid::place_lanes(oracle_input).unwrap();

        assert_eq!(
            production_grid_axis(production.lane_axis),
            oracle.lane_axis,
            "lane axis"
        );
        assert_eq!(
            production_grid_axis(production.grid_axis),
            oracle.grid_axis,
            "grid axis"
        );
        assert_eq!(production.content_size, oracle.content_size, "content size");
        assert_eq!(
            production
                .item_offsets
                .iter()
                .map(|item| (
                    item.item,
                    item.grid_axis_start,
                    item.grid_axis_span,
                    item.offset
                ))
                .collect::<Vec<_>>(),
            oracle
                .item_offsets
                .iter()
                .map(|item| (
                    item.id,
                    item.grid_axis_start,
                    item.grid_axis_span,
                    item.offset
                ))
                .collect::<Vec<_>>(),
            "item offsets"
        );
    }

    fn assert_production_lane_intrinsic_matches_oracle(
        production_input: ProductionLaneIntrinsicSizingInput,
        oracle_input: LaneIntrinsicSizingInput,
    ) {
        let production = production_lane_intrinsic_sizing(production_input)
            .unwrap()
            .unwrap();
        let oracle = grid::lane_intrinsic_sizing(oracle_input).unwrap();

        assert_eq!(
            production
                .definite_items
                .iter()
                .map(|item| (item.id, item.span.start, item.span.end))
                .collect::<Vec<_>>(),
            oracle
                .definite_items
                .iter()
                .map(|item| (item.id, item.span.start, item.span.end))
                .collect::<Vec<_>>(),
            "definite items"
        );
        assert_eq!(
            production
                .indefinite_groups
                .iter()
                .map(|group| {
                    (
                        group.span,
                        group.max_min_content,
                        group.max_max_content,
                        group.max_min_size,
                        group.item_ids.clone(),
                    )
                })
                .collect::<Vec<_>>(),
            oracle
                .indefinite_groups
                .iter()
                .map(|group| {
                    (
                        group.span,
                        group.max_min_content,
                        group.max_max_content,
                        group.max_min_size,
                        group.item_ids.clone(),
                    )
                })
                .collect::<Vec<_>>(),
            "indefinite groups"
        );
        assert_eq!(
            production
                .converted_indefinite_items
                .iter()
                .map(|item| (item.id, item.span.start, item.span.end))
                .collect::<Vec<_>>(),
            oracle
                .converted_indefinite_items
                .iter()
                .map(|item| (item.id, item.span.start, item.span.end))
                .collect::<Vec<_>>(),
            "converted indefinite items"
        );
        assert_eq!(
            production
                .final_track_sizes
                .iter()
                .map(|size| (size * 1000.0).round() / 1000.0)
                .collect::<Vec<_>>(),
            oracle
                .final_track_report
                .final_tracks
                .iter()
                .map(|track| (track.size * 1000.0).round() / 1000.0)
                .collect::<Vec<_>>(),
            "final track sizes"
        );
    }

    fn oracle_lane_facts(min_content: f32, max_content: f32) -> ItemContributionFacts {
        ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content,
            max_content,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Infinite,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        }
    }

    fn production_lane_facts(
        min_content: f32,
        max_content: f32,
    ) -> ProductionLaneContributionFacts {
        ProductionLaneContributionFacts {
            min_content,
            max_content,
            min_size: min_content,
            automatic_minimum_applies: true,
        }
    }

    fn production_grid_axis(axis: ProductionGridAxisKind) -> GridAxis {
        match axis {
            ProductionGridAxisKind::Column => GridAxis::Column,
            ProductionGridAxisKind::Row => GridAxis::Row,
        }
    }

    fn production_auto_lane_item(
        item: &'static str,
        grid_axis_span: usize,
        lane_axis_margin_box: f32,
    ) -> ProductionLaneItem<&'static str> {
        ProductionLaneItem {
            item,
            grid_axis_span,
            definite_grid_axis_start: None,
            lane_axis_margin_box,
        }
    }

    fn production_definite_lane_item(
        item: &'static str,
        grid_axis_start: usize,
        grid_axis_span: usize,
        lane_axis_margin_box: f32,
    ) -> ProductionLaneItem<&'static str> {
        ProductionLaneItem {
            item,
            grid_axis_span,
            definite_grid_axis_start: Some(grid_axis_start),
            lane_axis_margin_box,
        }
    }
}
