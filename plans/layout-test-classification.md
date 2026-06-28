# Layout Test Classification

## Baseline Test Counts

- `/Users/codex/Development/surgeist-layout`: `615` lines from `cargo test -- --list` in `/tmp/surgeist-layout-tests-before.txt`.
- `/Users/codex/Development/surgeist`: `460` lines from `cargo test -- --list` in `/tmp/surgeist-root-tests-before.txt`.

## Root Tests To Migrate Into Layout Unit Tests

- `/Users/codex/Development/surgeist/tests/oracle.rs`
  - oracle inline algorithm tests -> create and wire `/Users/codex/Development/surgeist-layout/src/inline_tests.rs`
  - oracle grid alignment, track sizing, baseline, placement, named-grid, subgrid, and lanes algorithm tests -> create and wire `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`
- `/Users/codex/Development/surgeist/tests/layout_oracle.rs`
  - inline/block layout versus oracle tests -> create and wire `/Users/codex/Development/surgeist-layout/src/block_tests.rs` and `/Users/codex/Development/surgeist-layout/src/inline_tests.rs`
  - grid, subgrid, lanes, and named-grid production-versus-oracle tests -> create and wire `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`

## Per-Test Migration Ledger

`Source` paths in this table are relative to the root repo `/Users/codex/Development/surgeist`, not the layout repo.

| Root test | Source | Disposition | Destination or covering test | Notes |
| --- | --- | --- | --- | --- |
| `oracle_atomic_inline_item_metrics_include_margins_and_baseline` | `tests/oracle.rs:24` | pending |  | |
| `oracle_atomic_inline_item_synthesizes_missing_baseline_from_bottom_edge` | `tests/oracle.rs:46` | pending |  | |
| `oracle_atomic_inline_item_rejects_baseline_before_top_edge` | `tests/oracle.rs:62` | pending |  | |
| `oracle_atomic_inline_item_rejects_baseline_after_bottom_edge` | `tests/oracle.rs:81` | pending |  | |
| `oracle_atomic_inline_line_aligns_items_to_max_baseline` | `tests/oracle.rs:100` | pending |  | |
| `oracle_atomic_inline_line_positions_margin_boxes_and_border_boxes` | `tests/oracle.rs:144` | pending |  | |
| `oracle_atomic_inline_wraps_between_items_for_definite_width` | `tests/oracle.rs:194` | pending |  | |
| `oracle_atomic_inline_intrinsic_widths_use_max_item_and_sum` | `tests/oracle.rs:243` | pending |  | |
| `oracle_atomic_inline_min_content_wraps_at_max_item_advance` | `tests/oracle.rs:274` | pending |  | |
| `oracle_atomic_inline_too_wide_item_overflows_without_empty_line` | `tests/oracle.rs:306` | pending |  | |
| `oracle_atomic_inline_wrapper_preserves_outer_and_inner_display_roles` | `tests/oracle.rs:330` | pending |  | |
| `oracle_atomic_inline_wrapper_metrics_use_outer_box_and_margins` | `tests/oracle.rs:370` | pending |  | |
| `oracle_atomic_inline_wrapper_produces_grid_contribution_facts` | `tests/oracle.rs:401` | pending |  | |
| `oracle_atomic_inline_grid_lanes_contribution_preserves_margins` | `tests/oracle.rs:439` | pending |  | |
| `oracle_baseline_geometry_uses_margin_box_contributions` | `tests/oracle.rs:546` | pending |  | |
| `oracle_baseline_geometry_rejects_non_participating_facts` | `tests/oracle.rs:557` | pending |  | |
| `oracle_baseline_offset_uses_whole_spanned_area_for_major_group` | `tests/oracle.rs:596` | pending |  | |
| `oracle_baseline_offset_uses_whole_spanned_area_for_minor_group` | `tests/oracle.rs:612` | pending |  | |
| `oracle_baseline_shim_grows_before_for_major_group` | `tests/oracle.rs:628` | pending |  | |
| `oracle_baseline_shim_grows_after_for_minor_group` | `tests/oracle.rs:650` | pending |  | |
| `oracle_baseline_shim_clamps_negative_major_growth_to_zero` | `tests/oracle.rs:672` | pending |  | |
| `oracle_baseline_shim_clamps_negative_minor_growth_to_zero` | `tests/oracle.rs:688` | pending |  | |
| `oracle_baseline_participation_rejects_out_of_flow_items` | `tests/oracle.rs:704` | pending |  | |
| `oracle_baseline_participation_rejects_auto_margins` | `tests/oracle.rs:718` | pending |  | |
| `oracle_baseline_participation_falls_back_for_synthesized_intrinsic_cycles` | `tests/oracle.rs:734` | pending |  | |
| `oracle_baseline_participation_falls_back_for_unavailable_subgrid_layout` | `tests/oracle.rs:750` | pending |  | |
| `oracle_baseline_participation_none_alignment_does_not_panic` | `tests/oracle.rs:768` | pending |  | |
| `oracle_baseline_predicates_ignore_unaligned_synthesized_cycle` | `tests/oracle.rs:779` | pending |  | |
| `oracle_baseline_predicates_reject_missing_aligned_first_intrinsic_cycle` | `tests/oracle.rs:795` | pending |  | |
| `oracle_baseline_predicates_reject_missing_aligned_last_intrinsic_cycle` | `tests/oracle.rs:816` | pending |  | |
| `oracle_baseline_groups_collect_major_group_on_start_track` | `tests/oracle.rs:837` | pending |  | |
| `oracle_baseline_groups_collect_minor_group_on_end_track_for_spanning_item` | `tests/oracle.rs:875` | pending |  | |
| `oracle_baseline_groups_preserve_nonparticipants_without_updating_group` | `tests/oracle.rs:899` | pending |  | |
| `oracle_baseline_groups_reject_invalid_track_and_row_spans` | `tests/oracle.rs:959` | pending |  | |
| `oracle_baseline_groups_collect_spanning_major_group_on_start_track` | `tests/oracle.rs:1023` | pending |  | |
| `oracle_container_baselines_prefer_major_and_minor_groups` | `tests/oracle.rs:1048` | pending |  | |
| `oracle_container_baselines_use_minor_group_for_first_when_major_missing` | `tests/oracle.rs:1082` | pending |  | |
| `oracle_container_baselines_use_major_group_for_last_when_minor_missing` | `tests/oracle.rs:1101` | pending |  | |
| `oracle_container_baselines_fallback_by_grid_order_and_synthesis` | `tests/oracle.rs:1120` | pending |  | |
| `oracle_container_baselines_last_fallback_uses_spanned_end_edge` | `tests/oracle.rs:1161` | pending |  | |
| `oracle_container_baselines_return_none_for_empty_input` | `tests/oracle.rs:1195` | pending |  | |
| `oracle_container_baselines_reject_vector_shape_mismatches` | `tests/oracle.rs:1214` | pending |  | |
| `oracle_container_baselines_reject_invalid_fallback_spans` | `tests/oracle.rs:1259` | pending |  | |
| `grid_definite_tracks_distribute_leftover_space_to_fr_tracks` | `tests/oracle.rs:1313` | pending |  | |
| `grid_explicit_tracks_resolve_percent_and_fr_after_fixed_tracks_and_gaps` | `tests/oracle.rs:1331` | pending |  | |
| `grid_fraction_tracks_do_not_expand_sub_one_factor_to_all_leftover_space` | `tests/oracle.rs:1350` | pending |  | |
| `grid_line_area_resolves_spans_across_tracks_and_gaps` | `tests/oracle.rs:1369` | pending |  | |
| `grid_auto_placement_places_row_column_and_dense_items` | `tests/oracle.rs:1383` | pending |  | |
| `grid_auto_placement_reports_zero_explicit_tracks` | `tests/oracle.rs:1410` | pending |  | |
| `grid_auto_placement_reports_row_flow_span_wider_than_columns` | `tests/oracle.rs:1422` | pending |  | |
| `grid_auto_placement_reports_column_flow_span_taller_than_rows` | `tests/oracle.rs:1436` | pending |  | |
| `grid_placement_resolves_start_and_end_lines` | `tests/oracle.rs:1450` | pending |  | |
| `grid_placement_resolves_start_line_plus_span` | `tests/oracle.rs:1461` | pending |  | |
| `grid_placement_resolves_span_plus_end_line` | `tests/oracle.rs:1472` | pending |  | |
| `grid_placement_defaults_auto_auto_to_one_track_span` | `tests/oracle.rs:1483` | pending |  | |
| `grid_placement_extends_implicit_tracks_after_explicit_grid` | `tests/oracle.rs:1492` | pending |  | |
| `grid_item_placement_resolves_two_axes_to_area` | `tests/oracle.rs:1502` | pending |  | |
| `oracle_named_grid_lines_empty_initializes_all_explicit_lines` | `tests/oracle.rs:1522` | pending |  | |
| `oracle_named_grid_lines_return_names_by_one_based_line` | `tests/oracle.rs:1532` | pending |  | |
| `oracle_named_grid_lines_reject_reserved_names` | `tests/oracle.rs:1542` | pending |  | |
| `oracle_named_line_occurrence_shape_is_exported` | `tests/oracle.rs:1563` | pending |  | |
| `oracle_named_grid_lines_preserve_repeated_names_in_source_order` | `tests/oracle.rs:1574` | pending |  | |
| `oracle_named_grid_lines_reject_mismatched_line_count` | `tests/oracle.rs:1582` | pending |  | |
| `oracle_named_fixed_repeat_expands_line_names_between_tracks` | `tests/oracle.rs:1601` | pending |  | |
| `oracle_named_fixed_repeat_merges_adjacent_line_name_lists` | `tests/oracle.rs:1622` | pending |  | |
| `oracle_named_fixed_repeat_rejects_zero_repeat` | `tests/oracle.rs:1642` | pending |  | |
| `oracle_named_fixed_repeat_rejects_reserved_line_names` | `tests/oracle.rs:1655` | pending |  | |
| `oracle_named_line_lookup_counts_positive_occurrences_from_start` | `tests/oracle.rs:1672` | pending |  | |
| `oracle_named_line_lookup_counts_negative_occurrences_from_end` | `tests/oracle.rs:1682` | pending |  | |
| `oracle_named_line_lookup_extends_after_for_missing_positive_occurrence` | `tests/oracle.rs:1691` | pending |  | |
| `oracle_named_line_lookup_extends_before_for_missing_negative_occurrence` | `tests/oracle.rs:1701` | pending |  | |
| `oracle_named_line_lookup_rejects_zero_occurrence` | `tests/oracle.rs:1711` | pending |  | |
| `oracle_named_line_lookup_rejects_reserved_custom_ident` | `tests/oracle.rs:1721` | pending |  | |
| `oracle_named_numeric_positive_line_passes_through` | `tests/oracle.rs:1739` | pending |  | |
| `oracle_named_numeric_negative_line_counts_from_explicit_end` | `tests/oracle.rs:1750` | pending |  | |
| `oracle_named_numeric_zero_line_is_invalid` | `tests/oracle.rs:1765` | pending |  | |
| `oracle_named_span_from_start_finds_nth_named_line_forward` | `tests/oracle.rs:1776` | pending |  | |
| `oracle_named_span_from_start_skips_explicit_end_line_for_implicit_names` | `tests/oracle.rs:1788` | pending |  | |
| `oracle_named_span_from_end_finds_nth_named_line_backward` | `tests/oracle.rs:1799` | pending |  | |
| `oracle_named_span_extends_implicitly_when_name_is_missing` | `tests/oracle.rs:1811` | pending |  | |
| `oracle_named_span_extends_implicitly_backward_when_name_is_missing` | `tests/oracle.rs:1821` | pending |  | |
| `oracle_named_span_rejects_zero_count` | `tests/oracle.rs:1832` | pending |  | |
| `oracle_named_span_rejects_reserved_custom_ident` | `tests/oracle.rs:1846` | pending |  | |
| `oracle_named_axis_resolves_named_start_and_named_end` | `tests/oracle.rs:1876` | pending |  | |
| `oracle_named_axis_resolves_line_to_named_span` | `tests/oracle.rs:1900` | pending |  | |
| `oracle_named_axis_resolves_required_mixed_forms` | `tests/oracle.rs:1920` | pending |  | |
| `oracle_named_axis_drops_end_span_when_both_sides_are_spans` | `tests/oracle.rs:2028` | pending |  | |
| `oracle_named_axis_records_ordered_span_span_normalizations` | `tests/oracle.rs:2056` | pending |  | |
| `oracle_named_axis_swaps_reversed_resolved_lines` | `tests/oracle.rs:2091` | pending |  | |
| `oracle_named_axis_drops_equal_end_line_to_span_one` | `tests/oracle.rs:2113` | pending |  | |
| `oracle_named_axis_clears_end_lookup_when_equal_line_drops_end` | `tests/oracle.rs:2135` | pending |  | |
| `oracle_named_axis_defaults_lone_start_named_span_to_one` | `tests/oracle.rs:2163` | pending |  | |
| `oracle_named_axis_defaults_lone_end_named_span_to_one` | `tests/oracle.rs:2187` | pending |  | |
| `oracle_named_axis_bare_ident_prefers_side_generated_line_name` | `tests/oracle.rs:2211` | pending |  | |
| `oracle_named_axis_bare_ident_falls_back_to_raw_name_without_side_names` | `tests/oracle.rs:2231` | pending |  | |
| `oracle_template_areas_generate_row_and_column_line_names` | `tests/oracle.rs:2249` | pending |  | |
| `oracle_template_areas_reject_non_rectangular_area` | `tests/oracle.rs:2282` | pending |  | |
| `oracle_template_areas_reject_empty_matrix` | `tests/oracle.rs:2295` | pending |  | |
| `oracle_template_areas_reject_mismatched_row_lengths` | `tests/oracle.rs:2303` | pending |  | |
| `oracle_template_areas_treat_dot_runs_as_null_cells` | `tests/oracle.rs:2317` | pending |  | |
| `oracle_template_areas_expand_base_line_map_to_template_size` | `tests/oracle.rs:2325` | pending |  | |
| `oracle_template_areas_preserve_larger_base_line_map` | `tests/oracle.rs:2340` | pending |  | |
| `oracle_template_areas_preserve_explicit_names_before_generated_names` | `tests/oracle.rs:2355` | pending |  | |
| `oracle_template_areas_generate_facts_for_both_axes` | `tests/oracle.rs:2372` | pending |  | |
| `oracle_template_areas_resolve_area_to_generated_named_lines` | `tests/oracle.rs:2389` | pending |  | |
| `oracle_template_areas_reject_missing_area_resolution` | `tests/oracle.rs:2410` | pending |  | |
| `oracle_named_grid_resolves_area_generated_names_to_grid_area` | `tests/oracle.rs:2422` | pending |  | |
| `oracle_axis_shorthand_repeats_omitted_custom_ident` | `tests/oracle.rs:2468` | pending |  | |
| `oracle_axis_shorthand_defaults_omitted_non_ident_to_auto` | `tests/oracle.rs:2484` | pending |  | |
| `oracle_grid_area_shorthand_repeats_single_custom_ident_to_all_sides` | `tests/oracle.rs:2500` | pending |  | |
| `oracle_grid_area_shorthand_expands_two_and_four_values` | `tests/oracle.rs:2525` | pending |  | |
| `oracle_grid_area_shorthand_defaults_omitted_non_idents_to_auto` | `tests/oracle.rs:2566` | pending |  | |
| `oracle_named_grid_resolves_subgrid_named_span_into_parent_space` | `tests/oracle.rs:2589` | pending |  | |
| `oracle_named_axis_auto_auto_with_cursor_resolves_one_track_span` | `tests/oracle.rs:2626` | pending |  | |
| `oracle_named_axis_unresolved_auto_without_cursor_returns_error` | `tests/oracle.rs:2645` | pending |  | |
| `oracle_named_axis_maps_line_before_first_error` | `tests/oracle.rs:2664` | pending |  | |
| `oracle_anonymous_span_offsets_from_known_edge` | `tests/oracle.rs:2687` | pending |  | |
| `oracle_anonymous_span_rejects_zero_count` | `tests/oracle.rs:2699` | pending |  | |
| `grid_track_report_initializes_fixed_percent_and_flex_tracks` | `tests/oracle.rs:2711` | pending |  | |
| `grid_track_report_initializes_auto_and_intrinsic_keywords` | `tests/oracle.rs:2740` | pending |  | |
| `grid_track_report_initializes_minmax_growth_limits` | `tests/oracle.rs:2765` | pending |  | |
| `grid_contributions_use_supplied_intrinsic_facts_and_margins` | `tests/oracle.rs:2790` | pending |  | |
| `grid_contributions_apply_min_max_and_preferred_limits` | `tests/oracle.rs:2817` | pending |  | |
| `grid_contributions_treat_explicit_infinite_max_as_unlimited` | `tests/oracle.rs:2844` | pending |  | |
| `grid_intrinsic_single_span_grows_minimum_and_content_phases` | `tests/oracle.rs:2863` | pending |  | |
| `grid_intrinsic_single_span_clamps_to_growth_limit` | `tests/oracle.rs:2891` | pending |  | |
| `grid_intrinsic_spanning_items_distribute_deficits_across_auto_tracks` | `tests/oracle.rs:2915` | pending |  | |
| `grid_intrinsic_row_spanning_items_use_row_axis` | `tests/oracle.rs:2944` | pending |  | |
| `grid_intrinsic_spanning_items_report_unsupported_mixed_track_categories` | `tests/oracle.rs:2973` | pending |  | |
| `grid_maximize_tracks_distributes_free_space_to_finite_growth_limits` | `tests/oracle.rs:3002` | pending |  | |
| `grid_flex_tracks_share_leftover_space_by_factor` | `tests/oracle.rs:3023` | pending |  | |
| `grid_flex_tracks_recompute_fraction_after_oversized_base_tracks` | `tests/oracle.rs:3042` | pending |  | |
| `grid_flex_tracks_report_zero_fraction_when_no_space_remains` | `tests/oracle.rs:3065` | pending |  | |
| `grid_stretch_grows_auto_tracks_after_flexing` | `tests/oracle.rs:3077` | pending |  | |
| `grid_auto_placement_reports_placed_areas_cursor_and_implicit_growth` | `tests/oracle.rs:3096` | pending |  | |
| `grid_equal_share_intrinsic_tracks_distribute_unbounded_spanning_deficits` | `tests/oracle.rs:3127` | pending |  | |
| `grid_auto_track_uses_stubbed_intrinsic_contribution_for_track_size` | `tests/oracle.rs:3143` | pending |  | |
| `grid_alignment_distributes_free_space_after_track_sizing` | `tests/oracle.rs:3203` | pending |  | |
| `grid_alignment_report_exposes_distribution_and_safe_fallback` | `tests/oracle.rs:3230` | pending |  | |
| `grid_scenario_composes_phase_reports_into_item_rects` | `tests/oracle.rs:3268` | pending |  | |
| `oracle_tree_stubs_child_measurements_and_records_layout_inputs` | `tests/oracle.rs:3312` | pending |  | |
| `oracle_axis_mapping_preserves_parallel_horizontal_axes` | `tests/oracle.rs:3359` | pending |  | |
| `oracle_axis_mapping_rejects_vertical_mapping_without_explicit_support` | `tests/oracle.rs:3377` | pending |  | |
| `oracle_axis_mapping_reports_reversed_when_flipped_states_differ` | `tests/oracle.rs:3396` | pending |  | |
| `oracle_subgrid_name_repeat_expands_to_used_span` | `tests/oracle.rs:3414` | pending |  | |
| `oracle_subgrid_auto_fill_name_repeat_pads_to_used_span` | `tests/oracle.rs:3436` | pending |  | |
| `oracle_subgrid_auto_fill_name_repeat_reserves_trailing_fixed_names` | `tests/oracle.rs:3454` | pending |  | |
| `oracle_subgrid_name_repeat_rejects_multiple_auto_fill_repeats` | `tests/oracle.rs:3479` | pending |  | |
| `oracle_subgrid_line_names_merge_parent_and_local_names` | `tests/oracle.rs:3501` | pending |  | |
| `oracle_subgrid_line_names_reverse_parent_line_order_when_axis_is_reversed` | `tests/oracle.rs:3527` | pending |  | |
| `oracle_subgrid_recomputes_area_generated_names_from_clipped_parent_areas` | `tests/oracle.rs:3544` | pending |  | |
| `oracle_subgrid_reversed_area_generated_names_follow_parent_boundaries` | `tests/oracle.rs:3573` | pending |  | |
| `oracle_subgrid_line_names_ignore_parent_area_generated_names_until_recomputed` | `tests/oracle.rs:3596` | pending |  | |
| `oracle_subgrid_line_names_order_area_generated_before_local_names` | `tests/oracle.rs:3619` | pending |  | |
| `oracle_subgrid_named_placement_clamps_to_subgrid_explicit_lines` | `tests/oracle.rs:3646` | pending |  | |
| `oracle_subgrid_named_placement_expands_collapsed_clamp_to_edge_track` | `tests/oracle.rs:3671` | pending |  | |
| `oracle_subgrid_eligibility_accepts_requested_axis_with_parent_grid` | `tests/oracle.rs:3694` | pending |  | |
| `oracle_subgrid_eligibility_rejects_lanes_parent_in_resolved_axis` | `tests/oracle.rs:3710` | pending |  | |
| `oracle_subgrid_eligibility_reports_first_blocking_reason` | `tests/oracle.rs:3729` | pending |  | |
| `oracle_subgrid_eligibility_reports_each_blocking_reason` | `tests/oracle.rs:3748` | pending |  | |
| `oracle_subgrid_copies_parent_tracks_for_span` | `tests/oracle.rs:3791` | pending |  | |
| `oracle_subgrid_reverses_copied_tracks_before_mbp_removal` | `tests/oracle.rs:3810` | pending |  | |
| `oracle_subgrid_resolves_normal_gap_to_parent_gap` | `tests/oracle.rs:3829` | pending |  | |
| `oracle_subgrid_baselines_slice_parent_groups_for_span` | `tests/oracle.rs:3848` | pending |  | |
| `oracle_subgrid_baselines_reverse_and_adjust_edges` | `tests/oracle.rs:3874` | pending |  | |
| `oracle_subgrid_baselines_reject_invalid_spans_and_group_shapes` | `tests/oracle.rs:3906` | pending |  | |
| `oracle_subgrid_baselines_preserve_none_through_mbp_and_gap_adjustment` | `tests/oracle.rs:3943` | pending |  | |
| `oracle_subgrid_baselines_adjust_each_internal_gap_edge` | `tests/oracle.rs:3966` | pending |  | |
| `oracle_subgrid_baselines_apply_signed_gap_differences` | `tests/oracle.rs:3987` | pending |  | |
| `oracle_subgrid_publishes_descendant_baseline_to_ancestor_track` | `tests/oracle.rs:4027` | pending |  | |
| `oracle_subgrid_publishes_reversed_descendant_baseline_to_ancestor_track` | `tests/oracle.rs:4052` | pending |  | |
| `oracle_subgrid_publishes_last_local_track_to_ancestor_track` | `tests/oracle.rs:4077` | pending |  | |
| `oracle_subgrid_publishes_reversed_last_local_track_to_ancestor_track` | `tests/oracle.rs:4102` | pending |  | |
| `oracle_subgrid_does_not_publish_synthesized_cycle_fallback` | `tests/oracle.rs:4127` | pending |  | |
| `oracle_subgrid_publish_rejects_zero_local_track` | `tests/oracle.rs:4150` | pending |  | |
| `oracle_subgrid_publish_rejects_local_track_beyond_span` | `tests/oracle.rs:4173` | pending |  | |
| `oracle_subgrid_applies_gap_difference_to_internal_edges` | `tests/oracle.rs:4196` | pending |  | |
| `oracle_subgrid_adds_negative_gap_difference_to_internal_edges` | `tests/oracle.rs:4215` | pending |  | |
| `oracle_subgrid_mbp_removal_clamps_tracks_to_zero` | `tests/oracle.rs:4234` | pending |  | |
| `oracle_subgrid_mbp_removal_consumes_across_tracks` | `tests/oracle.rs:4252` | pending |  | |
| `oracle_subgrid_traversal_collects_direct_leaf` | `tests/oracle.rs:4304` | pending |  | |
| `oracle_subgrid_traversal_accumulates_edge_mbp_for_intrinsic_tracks` | `tests/oracle.rs:4321` | pending |  | |
| `oracle_subgrid_traversal_swaps_edge_mbp_for_reversed_subgrid` | `tests/oracle.rs:4360` | pending |  | |
| `oracle_subgrid_traversal_accumulates_interior_edge_mbp_by_track` | `tests/oracle.rs:4399` | pending |  | |
| `oracle_subgrid_traversal_translates_leaf_span_through_child_subgrid` | `tests/oracle.rs:4432` | pending |  | |
| `oracle_subgrid_traversal_translates_reversed_leaf_span_from_end_edge` | `tests/oracle.rs:4453` | pending |  | |
| `oracle_subgrid_traversal_preserves_reversed_orientation_through_nested_subgrid` | `tests/oracle.rs:4482` | pending |  | |
| `oracle_subgrid_traversal_accumulates_gap_differences` | `tests/oracle.rs:4524` | pending |  | |
| `oracle_subgrid_traversal_skips_edge_mbp_for_non_intrinsic_tracks` | `tests/oracle.rs:4553` | pending |  | |
| `oracle_subgrid_traversal_requires_intrinsic_facts_for_edge_mbp` | `tests/oracle.rs:4582` | pending |  | |
| `oracle_subgrid_traversal_rejects_standalone_axis` | `tests/oracle.rs:4614` | pending |  | |
| `oracle_subgrid_traversal_rejects_invalid_leaf_span` | `tests/oracle.rs:4643` | pending |  | |
| `oracle_subgrid_traversal_supports_mixed_root_children` | `tests/oracle.rs:4656` | pending |  | |
| `oracle_subgrid_traversal_accumulates_nested_edge_adjustments` | `tests/oracle.rs:4682` | pending |  | |
| `oracle_subgrid_traversal_translates_nested_edge_adjustments_to_ancestor_tracks` | `tests/oracle.rs:4731` | pending |  | |
| `oracle_subgrid_traversal_applies_full_span_internal_gap` | `tests/oracle.rs:4777` | pending |  | |
| `oracle_grid_lanes_disables_row_axis_item_baseline_offsets` | `tests/oracle.rs:4803` | pending |  | |
| `oracle_grid_lanes_disables_column_axis_item_baseline_offsets` | `tests/oracle.rs:4821` | pending |  | |
| `oracle_grid_lanes_disables_item_baseline_offsets_for_all_axis_combinations` | `tests/oracle.rs:4839` | pending |  | |
| `oracle_grid_lanes_can_synthesize_container_baselines_from_geometry` | `tests/oracle.rs:4878` | pending |  | |
| `oracle_grid_lanes_container_baselines_use_final_geometry_offsets` | `tests/oracle.rs:4901` | pending |  | |
| `oracle_grid_lanes_container_baselines_last_uses_spanned_end_edge` | `tests/oracle.rs:4931` | pending |  | |
| `oracle_grid_lanes_container_baselines_return_none_for_empty_input` | `tests/oracle.rs:4954` | pending |  | |
| `oracle_grid_lanes_baseline_policy_reports_no_items` | `tests/oracle.rs:4962` | pending |  | |
| `oracle_grid_lanes_baseline_policy_reports_no_baseline_alignment_requested` | `tests/oracle.rs:4980` | pending |  | |
| `oracle_lanes_row_auto_flow_makes_rows_the_lane_axis` | `tests/oracle.rs:4998` | pending |  | |
| `oracle_lanes_place_definite_and_indefinite_items_with_fixed_tolerance` | `tests/oracle.rs:5010` | pending |  | |
| `oracle_lanes_finite_search_does_not_wrap_candidate_span` | `tests/oracle.rs:5032` | pending |  | |
| `oracle_lanes_reject_definite_item_that_exceeds_grid_axis` | `tests/oracle.rs:5055` | pending |  | |
| `oracle_lanes_infinite_tolerance_uses_round_robin_cursor` | `tests/oracle.rs:5072` | pending |  | |
| `oracle_lanes_percentage_tolerance_resolves_against_basis` | `tests/oracle.rs:5098` | pending |  | |
| `oracle_lanes_finite_tolerance_chooses_first_candidate_within_tolerance` | `tests/oracle.rs:5116` | pending |  | |
| `oracle_lanes_intrinsic_keeps_definite_items_by_span` | `tests/oracle.rs:5149` | pending |  | |
| `oracle_lanes_intrinsic_rewrites_definite_item_area_from_span` | `tests/oracle.rs:5178` | pending |  | |
| `oracle_lanes_intrinsic_rewrites_row_axis_areas_from_spans` | `tests/oracle.rs:5205` | pending |  | |
| `oracle_lanes_intrinsic_groups_indefinite_items_by_span_length` | `tests/oracle.rs:5232` | pending |  | |
| `oracle_lanes_intrinsic_groups_indefinite_items_by_min_size` | `tests/oracle.rs:5271` | pending |  | |
| `oracle_lanes_intrinsic_uses_min_content_for_min_content_tracks` | `tests/oracle.rs:5313` | pending |  | |
| `oracle_lanes_intrinsic_converts_all_spans_that_overlap_content_tracks` | `tests/oracle.rs:5341` | pending |  | |
| `oracle_lanes_intrinsic_distributes_converted_spanning_items` | `tests/oracle.rs:5377` | pending |  | |
| `oracle_lanes_intrinsic_splits_full_span_deficit_across_disjoint_content_tracks` | `tests/oracle.rs:5409` | pending |  | |
| `oracle_lanes_intrinsic_clamps_oversized_indefinite_spans_before_reporting` | `tests/oracle.rs:5437` | pending |  | |
| `oracle_lanes_intrinsic_skips_definite_items_outside_content_tracks_for_sizing` | `tests/oracle.rs:5466` | pending |  | |
| `oracle_lanes_intrinsic_reports_nested_indefinite_subgrid_unsupported` | `tests/oracle.rs:5492` | pending |  | |
| `oracle_lanes_intrinsic_rejects_invalid_definite_span` | `tests/oracle.rs:5518` | pending |  | |
| `oracle_lanes_intrinsic_rejects_definite_span_outside_tracks` | `tests/oracle.rs:5530` | pending |  | |
| `oracle_lanes_intrinsic_rejects_invalid_content_sized_track` | `tests/oracle.rs:5554` | pending |  | |
| `oracle_scenario_composes_subgrid_rect_from_explicit_tracks_and_offsets` | `tests/oracle.rs:5575` | pending |  | |
| `oracle_scenario_composes_lane_rect_from_lane_offset_and_grid_axis_area` | `tests/oracle.rs:5598` | pending |  | |
| `oracle_scenario_offsets_grid_items_by_baseline_report` | `tests/oracle.rs:5612` | pending |  | |
| `oracle_direct_subgrid_inherited_columns_shape` | `tests/oracle.rs:5647` | pending |  | |
| `oracle_grid_lanes_three_item_shape` | `tests/oracle.rs:5665` | pending |  | |
| `named_grid_layout_oracle_matches_bare_explicit_and_repeated_names` | `tests/layout_oracle.rs:279` | pending |  | |
| `named_grid_layout_oracle_matches_negative_missing_and_backward_spans` | `tests/layout_oracle.rs:348` | pending |  | |
| `named_grid_layout_oracle_matches_auto_span_and_conflict_normalization` | `tests/layout_oracle.rs:432` | pending |  | |
| `named_grid_layout_oracle_matches_template_area_generated_lines` | `tests/layout_oracle.rs:499` | pending |  | |
| `subgrid_layout_oracle_matches_merged_local_and_inherited_area_lines` | `tests/layout_oracle.rs:578` | pending |  | |
| `subgrid_layout_oracle_matches_local_area_clamp_to_inherited_span` | `tests/layout_oracle.rs:723` | pending |  | |
| `oracle_layout_inline_block_line_matches_layout` | `tests/layout_oracle.rs:821` | pending |  | |
| `oracle_layout_inline_grid_line_matches_layout` | `tests/layout_oracle.rs:826` | pending |  | |
| `oracle_layout_inline_grid_lanes_line_matches_layout` | `tests/layout_oracle.rs:831` | pending |  | |
| `oracle_layout_fixed_tracks_match_layout_child_rects` | `tests/layout_oracle.rs:836` | pending |  | |
| `oracle_layout_percent_and_flex_tracks_match_layout_child_rects` | `tests/layout_oracle.rs:857` | pending |  | |
| `oracle_layout_sub_one_flex_track_uses_partial_leftover_space` | `tests/layout_oracle.rs:884` | pending |  | |
| `oracle_layout_minmax_tracks_match_layout_child_rects` | `tests/layout_oracle.rs:901` | pending |  | |
| `oracle_layout_stretch_expands_auto_tracks_like_layout` | `tests/layout_oracle.rs:927` | pending |  | |
| `oracle_layout_explicit_line_span_matches_layout_area_rect` | `tests/layout_oracle.rs:947` | pending |  | |
| `oracle_layout_row_auto_flow_matches_oracle_placement` | `tests/layout_oracle.rs:969` | pending |  | |
| `oracle_layout_column_auto_flow_matches_oracle_placement` | `tests/layout_oracle.rs:996` | pending |  | |
| `oracle_layout_dense_auto_flow_matches_spanning_oracle_placement` | `tests/layout_oracle.rs:1024` | pending |  | |
| `oracle_layout_center_alignment_offsets_tracks_like_layout` | `tests/layout_oracle.rs:1056` | pending |  | |
| `oracle_layout_space_between_alignment_offsets_tracks_like_layout` | `tests/layout_oracle.rs:1075` | pending |  | |
| `oracle_layout_safe_center_alignment_falls_back_on_overflow` | `tests/layout_oracle.rs:1094` | pending |  | |
| `oracle_layout_auto_track_uses_supplied_intrinsic_measurement` | `tests/layout_oracle.rs:1113` | pending |  | |
| `oracle_layout_spanning_auto_tracks_distribute_intrinsic_deficit` | `tests/layout_oracle.rs:1139` | pending |  | |
| `oracle_layout_fit_content_track_clamps_intrinsic_growth` | `tests/layout_oracle.rs:1167` | pending |  | |
| `oracle_layout_harness_asserts_nested_grid_descendant_output` | `tests/layout_oracle.rs:1196` | pending |  | |
| `subgrid_child_rect_matches_oracle_composed_rect` | `tests/layout_oracle.rs:1230` | pending |  | |
| `layout_oracle_grid_baseline_offset_matches_oracle` | `tests/layout_oracle.rs:1284` | pending |  | |
| `subgrid_child_items_resolve_against_local_lines` | `tests/layout_oracle.rs:1301` | pending |  | |
| `subgrid_standalone_axis_uses_ordinary_child_tracks` | `tests/layout_oracle.rs:1334` | pending |  | |
| `subgrid_item_still_respects_parent_grid_placement` | `tests/layout_oracle.rs:1368` | pending |  | |
| `subgrid_child_auto_margins_use_inherited_area_size` | `tests/layout_oracle.rs:1396` | pending |  | |
| `subgrid_child_alignment_uses_inherited_area_size` | `tests/layout_oracle.rs:1436` | pending |  | |
| `subgrid_rtl_child_lines_use_reversed_inherited_columns` | `tests/layout_oracle.rs:1472` | pending |  | |
| `subgrid_explicit_zero_gap_overrides_parent_gap` | `tests/layout_oracle.rs:1506` | pending |  | |
| `subgrid_percent_gap_uses_content_box_basis` | `tests/layout_oracle.rs:1540` | pending |  | |
| `subgrid_percentage_padding_uses_grid_area_basis` | `tests/layout_oracle.rs:1580` | pending |  | |
| `subgrid_traversal_nested_inherited_leaf_contribution_grows_parent_auto_track` | `tests/layout_oracle.rs:1633` | pending |  | |
| `subgrid_traversal_reversed_nested_inherited_subgrid_maps_to_mirrored_track` | `tests/layout_oracle.rs:1667` | pending |  | |
| `subgrid_traversal_nested_margin_border_padding_increases_contribution` | `tests/layout_oracle.rs:1698` | pending |  | |
| `subgrid_traversal_gap_difference_adjustment_accumulates_through_nested_subgrids` | `tests/layout_oracle.rs:1740` | pending |  | |
| `subgrid_traversal_direct_leaf_uses_internal_gap_adjustment` | `tests/layout_oracle.rs:1777` | pending |  | |
| `subgrid_traversal_unsupported_sibling_does_not_drop_valid_contribution` | `tests/layout_oracle.rs:1809` | pending |  | |
| `subgrid_traversal_percent_padding_uses_definite_area_basis` | `tests/layout_oracle.rs:1847` | pending |  | |
| `subgrid_traversal_percent_gap_uses_definite_content_box_basis` | `tests/layout_oracle.rs:1876` | pending |  | |
| `subgrid_traversal_translated_nested_edge_adjustments_land_on_ancestor_tracks` | `tests/layout_oracle.rs:1906` | pending |  | |
| `subgrid_absolute_descendant_uses_existing_static_position_behavior` | `tests/layout_oracle.rs:1948` | pending |  | |
| `subgrid_named_placement_clamp_matches_oracle` | `tests/layout_oracle.rs:1983` | pending |  | |
| `oracle_layout_harness_can_compare_lane_reports` | `tests/layout_oracle.rs:2054` | pending |  | |
| `lanes_row_auto_flow_matches_oracle_placement` | `tests/layout_oracle.rs:2098` | pending |  | |
| `lanes_column_auto_flow_matches_oracle_placement` | `tests/layout_oracle.rs:2128` | pending |  | |
| `lanes_definite_grid_axis_item_matches_oracle_placement` | `tests/layout_oracle.rs:2158` | pending |  | |
| `lanes_auto_span_clamping_matches_oracle_placement` | `tests/layout_oracle.rs:2186` | pending |  | |
| `lanes_finite_tolerance_matches_oracle_placement` | `tests/layout_oracle.rs:2208` | pending |  | |
| `lanes_finite_search_does_not_wrap_candidate_span_across_grid_axis_end` | `tests/layout_oracle.rs:2238` | pending |  | |
| `lanes_infinite_tolerance_matches_oracle_placement` | `tests/layout_oracle.rs:2266` | pending |  | |
| `lanes_intrinsic_groups_indefinite_items_like_oracle` | `tests/layout_oracle.rs:2296` | pending |  | |
| `lanes_intrinsic_skips_definite_items_outside_content_sized_tracks` | `tests/layout_oracle.rs:2350` | pending |  | |
| `lanes_intrinsic_projects_disjoint_content_sized_spans_like_oracle` | `tests/layout_oracle.rs:2388` | pending |  | |
| `lanes_intrinsic_clamps_oversized_indefinite_spans_like_oracle` | `tests/layout_oracle.rs:2432` | pending |  | |
| `lanes_intrinsic_preserves_min_content_track_behavior` | `tests/layout_oracle.rs:2464` | pending |  | |
| `lanes_intrinsic_reports_nested_indefinite_subgrid_unsupported_like_oracle` | `tests/layout_oracle.rs:2507` | pending |  | |
| `lanes_content_size_contributes_to_indefinite_container_size` | `tests/layout_oracle.rs:2530` | pending |  | |
| `lanes_child_measurement_uses_resolved_grid_axis_span_size` | `tests/layout_oracle.rs:2560` | pending |  | |
| `lanes_auto_child_measurement_uses_final_auto_placement_span` | `tests/layout_oracle.rs:2669` | pending |  | |
| `lanes_spanning_child_measurement_uses_distributed_grid_axis_gap` | `tests/layout_oracle.rs:2828` | pending |  | |
| `lanes_absolute_child_uses_grid_absolute_layout` | `tests/layout_oracle.rs:2915` | pending |  | |
| `lanes_indefinite_nested_subgrid_does_not_contribute_as_ordinary_lane_item` | `tests/layout_oracle.rs:2939` | pending |  | |
| `lanes_child_subgrid_inherits_grid_axis_tracks` | `tests/layout_oracle.rs:2970` | pending |  | |
| `lanes_column_flow_child_subgrid_inherits_row_axis_tracks` | `tests/layout_oracle.rs:3005` | pending |  | |
| `lanes_child_subgrid_uses_report_matching_child_order_after_skipped_siblings` | `tests/layout_oracle.rs:3040` | pending |  | |
| `lanes_definite_lane_axis_container_lays_out_children_at_lane_offsets` | `tests/layout_oracle.rs:3086` | pending |  | |

Disposition lifecycle:

- `pending`: initial Task 1 state; the test has not yet been migrated, covered, or marked obsolete.

Allowed final dispositions:

- `migrated`: the test body was copied into the named layout source test file.
- `covered`: an equivalent existing layout test already covers the same behavior.
- `obsolete`: the test asserted an old harness behavior that no longer exists after this refactor.

Every `covered` row must name the exact covering test and why it is equivalent.
Every `obsolete` row must name the removed harness assumption and must be approved by the reviewer before root deletion.

Helper functions are not test ledger rows. Move helper functions together with the migrated tests that call them, or delete them only after `rg` proves there are no remaining call sites.

## Tests That Stay In Layout Integration Tests

- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity.rs`
- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/README.md`
- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/corpus.toml`
- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/support.rs`
- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/html/**`
- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/xml/**`

## Tests That Stay In Root

- Root facade smoke tests that verify `surgeist::layout` reexports compile.
- No root tests may path-import `/Users/codex/Development/surgeist/crates/surgeist-layout/tests/support`.
