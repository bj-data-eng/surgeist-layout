# P01/I08/S02/R08 Entry Test Disposition Ledger

Cycle: `P01/I08/S02/R08`

Entry revision: `97a93e935a42a62d931192697abbafe382054806`

Authority: `FRI-08.27.1` and the installed Surgeist testing reference.

This attachment is the exhaustive named disposition of prohibited entry tests.
It is reviewed with the R08 cycle plan. Tests not named here survive only in the
allowed evidence classes in the cycle plan; the final whole-crate static and
manual review audits check that classification rather than treating this ledger
as runtime evidence.

## T01 Core Source Proxies

Remove 30 tests from `src/lib_tests.rs`:

1. `fri06_c05_contract_float_exclusion_surface_is_opaque_cache_neutral_and_active`
2. `fri06_c01_contract_aggregate_public_surface_covers_every_cycle_break_and_addition`
3. `fri05_c01_node_input_removed_phase_unsafe_surfaces_are_absent_from_public_sources`
4. `fri05_c02_carrier_private_fields_constructors_and_no_default_are_static`
5. `fri05_c03_legacy_surface_rect_has_only_the_typed_public_constructor`
6. `fri05_c03_public_geometry_surface_has_exact_read_only_accessors`
7. `fri05_c03_legacy_surface_is_absent_from_public_source`
8. `fri05_c03_root_block_legacy_absence_production_paths_and_bridge_accounting`
9. `fri05_c04_flex_bridge_accounting_accepts_grid_family_closure`
10. `fri08_c03_public_removal_nested_lanes_unsupported_symbols_are_absent`
11. `fri05_c04_flex_round_cache_publication_has_one_canonical_geometry_path`
12. `fri05_c04_flex_legacy_absence_accepts_downstream_grid_closure`
13. `fri05_c05_grid_round_cache_has_no_independent_scrollbar_projection`
14. `fri05_c05_grid_legacy_absence_lexer_fails_closed_at_rust_token_boundaries`
15. `fri05_c05_grid_legacy_absence_cfg_attr_omits_exactly_one_test_only_item`
16. `fri05_c05_grid_legacy_absence_inventories_every_production_source`
17. `fri08_remediation_scroll_model_has_one_owner`
18. `fri08_remediation_scroll_box_geometry_has_one_owner`
19. `fri08_remediation_scroll_contribution_has_one_owner`
20. `fri08_remediation_scroll_construction_has_one_owner`
21. `fri08_remediation_scroll_construction_and_rounding_equivalence`
22. `fri08_remediation_engine_contract_is_algorithm_neutral`
23. `fri08_remediation_engine_validation_has_one_owner`
24. `fri08_remediation_measurement_has_one_owner`
25. `fri08_remediation_sizing_resolution_has_one_owner`
26. `fri08_remediation_engine_root_has_one_owner`
27. `fri08_remediation_engine_session_transaction_equivalence`
28. `fri08_remediation_engine_rounding_has_one_owner`
29. `fri08_remediation_public_api_inventory_is_compatible`
30. `fri05_c07_public_surface_removed_phase_unsafe_contracts_fail_closed`

Remove five tests from `src/contract_tests.rs`:

1. `fri06_c02_contract_block_has_no_c02_text_fallback_spelling`
2. `fri06_c02_contract_inline_has_no_shaping_or_measurement_path`
3. `fri06_c02_contract_text_source_has_no_owned_dead_code_allowance`
4. `fri06_c02_contract_cache_key_context_remains_one_unit_declaration`
5. `fri05_c03_output_helper_no_geometry_fallback_saturates_each_scalar_lane`

Remove `fri06_c05_provider_cache_context_stays_unit_and_rounding_has_no_provider_path`
from `src/cache_tests.rs` and
`fri05_c03_root_block_legacy_absence_factory_has_no_migration_or_rounding_adapter`
from `src/scroll/construction.rs`: 37 library tests total.

Disposition: public shape remains compiled by adjacent alias/reexport/type tests,
external compile contracts, and compile-fail doctests. Cache and scroll retain
cold/warm, failure, source, geometry, contribution, and publication behavior.
The remediation anchors and legacy lexers have no product contract.

## T02 Internal Architecture And Oracle Proxies

Remove:

1. `fri08_c06r_inherited_placement_architecture_has_no_residual_ordinary_estimator`
2. `fri08_c02r_lanes_track_phase_architecture_has_no_collection_fit_content_shortcut`
3. `fri08_c02r_lanes_track_phase_architecture_has_one_auto_maximum_predicate`
4. `fri08_c02r_lanes_track_phase_architecture_has_one_policy_free_final_owner`
5. `fri06_c04_float_lifecycle_static_one_ledger_query_and_single_publication_paths`
6. `fri06_c03_lifecycle_unified_mixed_publication_contributes_each_geometry_once`
7. ignored `layout_oracle_grid_baseline_offset_matches_oracle`

Disposition: adjacent scalar/failure-path layout tests remain. The ignored test
compares an oracle to copied `9.0` without a production subject and has no
replacement. Rename/re-oracle
`block_line_break_conversion_with_metadata_preserves_current_output` from the
authored metrics, fixed atomics, RTL line position, and two line-box extents;
its count remains one.

## T03 Generator Source And Workflow Proxies

Remove 13 generator-target tests:

1. `fri06_c08r_lineage_parser_and_serializer_implementations_are_frozen`
2. `fri08_c05_adapter_current_helper_and_template_area_parser_are_byte_frozen`
3. `fri08_c06_settled_sources_manifest_helper_and_production_are_byte_frozen`
4. `fri08_c06_manifest_and_settled_derivation_inputs_are_exact`
5. `fri06_c08_recovery_inputs_owned_sources_match_reviewed_freeze`
6. `fri06_c08_new_exact_flow_root_census_sources_use_supported_overflow_bfcs`
7. `fri06_c08_t2_reconstructs_exact_input_census_membership`
8. `fri06_c08_existing_entry_report_reconstructs_exact_activation_and_baseline_matrices`
9. `fri06_c08_existing_census_intersection_assigns_only_exact_input_rows`
10. `fri06_c08_range_ink_census_partitions_retain_exact_selectors_counts_and_digests`
11. `fri06_c08_recovery_characterization_reconciles_literal_and_executed_censuses`
12. ignored `stale_artifact_inventory_matches_committed_entry`
13. ignored `worktree_is_clean_before_input_recovery`

Disposition: direct parser, serializer, runtime-helper, generated-format,
import, and public-layout tests remain. Isolated temporary-Git import tests are
product behavior. `track_definition_serializes_subgrid_line_names` remains a
valid serializer contract.

## T04 Browser Source And Workflow Proxies

Remove:

1. `fri06_c08r_lineage_support_has_no_name_or_expectation_compatibility_path`
2. `fri06_c08_r0_control_probe_matrix_is_exact_72_plus_24_rows`
3. `fri06_c08r_final_activation_union_browser_passes_without_substitutes`
4. `fri06_c12_t07_endpoint_accounting_is_exact`
5. `generation_report_uses_explicit_br_unsupported_buckets`

Disposition: lowering, parser, comparator, fragment, endpoint, current-report,
and public-layout consumers remain. The support test's historical `git show`
report is workflow state, not report-schema evidence.

## T05 Parity Inventory Proxies

Remove 24 integration tests:

1. `fri08_c05_inputs_exact_new_source_inventory_and_finite_structure_are_settled`
2. `fri08_c06_manifest_has_exact_active_rows_without_suppression`
3. `fri08_c06_owned_rows_are_unique_and_adopt_exact_control_inventory`
4. `fri08_c06_exact_seventy_two_owned_rows_exist_parse_and_are_comment_free`
5. `fri07_c04_fixture_input_exact_six_source_four_variant_inventory_is_bounded`
6. `fri08_c06_final_inventory_preserves_fri07_expected_fail_contract`
7. `fri08_c06_final_inventory_preserves_fri04_fixture_contract`
8. `fri05_c06_computed_overflow_corpus_has_exact_44_output_inventory`
9. `fri_03_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_outputs`
10. `fri04_c05_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_paths`
11. `fri08_c08_t02_all_axis_family_inventories_paths_topologies_and_mismatches_are_stable`
12. `grid_axes_fixture_matrix_is_generated`
13. `grid_lanes_axis_fixture_matrix`
14. `subgrid_axis_fixture_matrix`
15. `flex_axis_fixture_matrix_rejects_missing_duplicate_misplaced_and_leaf_lowered_topology_paths`
16. `grid_axis_fixture_matrix_rejects_invalid_paths_and_topology`
17. `grid_lanes_axis_fixture_matrix_rejects_invalid_paths_and_topology`
18. `subgrid_axis_fixture_matrix_rejects_invalid_paths_and_topology`
19. `block_axis_fixture_matrix_rejects_missing_duplicate_misplaced_and_topology_bypassed_paths`
20. `calc_fixture_family_rejects_misplaced_duplicate_variant`
21. `browser_parity_corpus_manifest_exists`
22. `browser_parity_html_corpus_inventory_is_documented`
23. `fri08_c06_generation_report_counts_full_scope`
24. `browser_parity_generation_report_inventory_is_full_only`

Disposition: delete raw expected-path/census helpers used only here. Retain every
actual parse-and-layout comparison plus manifest/report/provenance/path/XML
consumer tests. Rename/refactor the active-output production comparison and the
finite collapse ordinary/Chrome comparisons to assert parsed fixture and layout
behavior without scenario counts; their counts remain unchanged.

## T06 Generator Inventory And Lineage Proxies

Remove 23 generator-target tests:

1. `fri05_c06_manifest_freeze_requires_full_only_final_inventory`
2. `fri05_c06_manifest_freeze_has_exact_active_sources_and_final_buckets`
3. `fri04_c05_unsupported_report_projection_matches_published_digest`
4. `fri07_c04_fixture_sources_have_exact_owned_inventory_and_behavior_contract`
5. `fri07_c04_fixture_sources_map_to_exact_standard_twenty_four_variant_paths`
6. `fri07_c04_manifest_freezes_exact_three_active_and_three_expected_fail_cases`
7. `fri07_c04_corpus_has_exact_final_inventory_and_centralized_provenance`
8. `fri06_c08_t2_manifest_owns_exact_active_four_variant_matrix_and_counts`
9. `fri06_c08_t2_sources_have_exact_inventory_and_corrected_finite_behavior_facts`
10. `fri05_c06_fixture_sources_have_exact_owned_inventory_and_behavior_contract`
11. `fri05_c06_fixture_sources_map_to_exact_standard_four_variant_matrix`
12. `fri08_c06_base_lineage_hashes_and_preserved_bodies_remain_unchanged`
13. `fri06_c08r_lineage_environment_is_unfiltered_and_default_rooted`
14. `fri08_c06_lineage_provenance_inputs_match_settled_values`
15. `fri06_c08r_zero_width_wrapping_sources_declare_block_wrapper_role`
16. `fri06_c08r_lineage_nine_html_inputs_are_byte_frozen`
17. `fri08_c06_final_report_and_xml_lineage_are_complete_and_exception_free`
18. `fri08_c06_final_lineage_report_closes_inventory_and_provenance`
19. `fri08_c06_final_lineage_preserves_base_nonactivation_xml_semantics`
20. `fri06_c08r_final_lineage_accounts_for_exact_marker_outputs`
21. `fri06_c08r_fixture_input_exact_source_marker_inventory_is_authored`
22. `fri06_c12_t07_default_block_parent_inventory_authors_block_role`
23. `fri06_c12_t07_exact_nine_source_marker_inventory_has_four_scoped_bidi_records`

Disposition: refactor/rename
`centralized_provenance_accepts_current_exact_inventory` to use a synthetic
manifest-described inventory. Retain generic manifest/report schemas, hashes and
referential integrity, deterministic serialization, XML generation, check-corpus,
import-Taffy, locking, helper runtime, parser, and serializer contracts.

## Reconciled Final Inventory

- library: 2,042 (entry 2,087 minus T01 37, T02 7, and the T04 support test);
- grid prefix: 1,011 (entry 1,017 minus T02's five grid tests and T04 support);
- root prefix: 233; flex prefix: 169; block prefix: 212;
- integration: 215 (entry 244 minus T04 five and T05 24);
- default package including 72 doctests: 2,329;
- generator binary: 350 (entry 386 minus T03 13 and T06 23);
- ignored: only `runs_all_checked_in_browser_parity_xml`.
