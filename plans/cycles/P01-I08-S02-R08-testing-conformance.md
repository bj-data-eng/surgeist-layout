# P01/I08/S02/R08 Whole-Crate Testing-Reference Conformance

Cycle ID: `P01/I08/S02/R08`

Owning repository: `surgeist-layout`

Status: `draft`

Cycle base: `97a93e935a42a62d931192697abbafe382054806`

Specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`,
reviewed semantic SHA-256
`65050fe9723a62ef832badd02426c3fc2cb461f7931a4549a4c48c2ea39614e7`,
commit `98d67b05b7570e84490c6bf0121ba4a0cc2ec224`, sections `FRI-08.20`
row `AR-007`, `FRI-08.21`, `FRI-08.27.1`, and all of `FRI-08.28`.

Sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`,
reviewed semantic SHA-256
`bb3642deea547129932693820df949b3db365ba4e4f134814ab9611dbb2aa171`,
commit `5ec0eae900daac01d56dba8ea919080ea13be26e`, entry
`P01/I08/S02/R08`.

Testing authority: installed Surgeist reference
`/Users/codex/.codex/skills/surgeist-agent/references/testing.md`.

## 1 Bounded Outcome

The whole tracked Rust test suite asserts product contracts rather than source,
symbol, file-placement, initiative census, workflow state, or the
implementation's current output. Prohibited proxies are removed or replaced by
existing behavioral, compile-contract, oracle, or declared-artifact consumer
evidence. The only ignored test is the explicitly runnable full checked-in
browser-parity tier. No production behavior, public API, dependency, feature,
MSRV, fixture, generated artifact, ordinary verification command, or standing
Dylint selection changes.

The cycle does not add a permanent architecture rule. R06A's existing
planning-path Dylint remains opt-in and unselected. No new Dylint is required:
the useful node-projection question already has a compiler-semantic audit, and
the remaining source proxies either have real behavioral/compile evidence or
encode no product contract. If implementation discovers a distinct question
that cannot be classified this way, stop for a reviewed plan revision rather
than adding a lexical script, Rust source-parsing test, or unplanned lint.

## 2 Entry Inventory And Classification

R07 published the exact entry at the cycle base. Its locked/offline libtest
inventory is:

| Target/prefix | Tests | Sorted leaf digest |
| --- | ---: | --- |
| library | 2,087 | `e0da19f5f8ff509122a3b1f846f0257cbd1e2c0ff850bf3cbbdeaa84f59eb87b` |
| `grid::tests::` | 1,017 | `7d183fadd0668543df877f58247158c02a36541b3c327230b92eced892541adf` |
| `root_tests::` | 235 | `2e616af2e7c7b7a0f480cba90b63b7807831bc72da4d42d5464a9e49e04610dc` |
| `flex_tests::` | 169 | `5a1cd29203c4ee0eea91169e18512ffdd79bea40fe6237120d261490f8a6c0d5` |
| `block_tests::` | 212 | `a73897032730d7b3b0d3c10746a17f16403baab9ffb1de4fedcdca01e2b3822f` |

The default package discovers 2,403 tests. Its ignored leaves are exactly
`layout_oracle_grid_baseline_offset_matches_oracle` and
`runs_all_checked_in_browser_parity_xml`, digest
`235b22672841a0d7889b49ce4e7241b5abde0ba18311c8a23774d76597dfaf99`.
The generator binary target discovers 386 tests: 384 passing and the two
entry-preflight tests ignored.

All surviving tests fall into one of these evidence classes:

- validated scalar/value/model contracts in owning `src/` test modules;
- public or crate-boundary layout behavior, cache, invalidation, transaction,
  measurement, rounding, block, inline, float, flex, grid, subgrid, lanes, and
  scroll behavior;
- independently derived oracle comparisons;
- external-crate compile contracts and compile-fail doctests;
- parser, serializer, manifest, schema, report, provenance, corpus-consumer,
  import, locking, and deterministic generated-format contracts;
- checked-in browser-parity comparisons, with the complete corpus as one
  explicit expensive ignored tier.

The following ledger is exhaustive for prohibited entry evidence. Each exact
test is assigned once below. A removed test has no product contract unless a
replacement is named. Existing real behavior remains primary evidence and must
not be weakened to preserve a proxy's test count.

## 3 Implementation Tasks

### 3.1 `P01/I08/S02/R08/T01` Remove Core Source Proxies

**Paths:** `src/lib_tests.rs`, `src/contract_tests.rs`,
`src/cache_tests.rs`, and the test module in `src/scroll/construction.rs` only.

Remove these 29 `src/lib_tests.rs` source/token/owner tests and their now-dead
lexer/audit helpers:

- `fri06_c05_contract_float_exclusion_surface_is_opaque_cache_neutral_and_active`;
- `fri06_c01_contract_aggregate_public_surface_covers_every_cycle_break_and_addition`;
- `fri05_c01_node_input_removed_phase_unsafe_surfaces_are_absent_from_public_sources`;
- `fri05_c02_carrier_private_fields_constructors_and_no_default_are_static`;
- `fri05_c03_legacy_surface_rect_has_only_the_typed_public_constructor`;
- `fri05_c03_public_geometry_surface_has_exact_read_only_accessors`;
- `fri05_c03_legacy_surface_is_absent_from_public_source`;
- `fri05_c03_root_block_legacy_absence_production_paths_and_bridge_accounting`;
- `fri05_c04_flex_bridge_accounting_accepts_grid_family_closure`;
- `fri08_c03_public_removal_nested_lanes_unsupported_symbols_are_absent`;
- `fri05_c04_flex_round_cache_publication_has_one_canonical_geometry_path`;
- `fri05_c04_flex_legacy_absence_accepts_downstream_grid_closure`;
- `fri05_c05_grid_round_cache_has_no_independent_scrollbar_projection`;
- `fri05_c05_grid_legacy_absence_lexer_fails_closed_at_rust_token_boundaries`;
- `fri05_c05_grid_legacy_absence_cfg_attr_omits_exactly_one_test_only_item`;
- `fri05_c05_grid_legacy_absence_inventories_every_production_source`;
- `fri08_remediation_scroll_model_has_one_owner`;
- `fri08_remediation_scroll_box_geometry_has_one_owner`;
- `fri08_remediation_scroll_contribution_has_one_owner`;
- `fri08_remediation_scroll_construction_has_one_owner`;
- `fri08_remediation_scroll_construction_and_rounding_equivalence`;
- `fri08_remediation_engine_contract_is_algorithm_neutral`;
- `fri08_remediation_engine_validation_has_one_owner`;
- `fri08_remediation_measurement_has_one_owner`;
- `fri08_remediation_sizing_resolution_has_one_owner`;
- `fri08_remediation_engine_root_has_one_owner`;
- `fri08_remediation_engine_session_transaction_equivalence`;
- `fri08_remediation_engine_rounding_has_one_owner`; and
- `fri08_remediation_public_api_inventory_is_compatible`.

Remove these five `src/contract_tests.rs` source proxies:

- `fri06_c02_contract_block_has_no_c02_text_fallback_spelling`;
- `fri06_c02_contract_inline_has_no_shaping_or_measurement_path`;
- `fri06_c02_contract_text_source_has_no_owned_dead_code_allowance`;
- `fri06_c02_contract_cache_key_context_remains_one_unit_declaration`; and
- `fri05_c03_output_helper_no_geometry_fallback_saturates_each_scalar_lane`.

Remove
`fri06_c05_provider_cache_context_stays_unit_and_rounding_has_no_provider_path`
from `src/cache_tests.rs` and
`fri05_c03_root_block_legacy_absence_factory_has_no_migration_or_rounding_adapter`
from `src/scroll/construction.rs`.

**Disposition:** no source-shape contract survives. Public surface remains
compiled by the adjacent alias/reexport/type-composition tests, external compile
contracts, and compile-fail doctests. Cache context remains covered by
`f64_cache_context_remains_tree_context_only` and cold/warm cache behavior.
Canonical scroll construction remains covered by the existing canonical source,
geometry, contribution, and publication behavior families. The 13 remediation
anchors are migration evidence already independently reviewed, not product
contracts.

**Acceptance:** exactly 36 library tests are removed; no production item or
visibility changes; focused public-contract, cache, canonical-scroll, and
`fri08_remediation_` searches show the retained behavioral families and zero
remediation source anchors; full tests and strict gates pass.

Commit: `test(conformance): remove source proxy contracts`.

### 3.2 `P01/I08/S02/R08/T02` Replace Internal Architecture Proxies

**Paths:** `src/grid_tests/lanes_subgrid_tests.rs`,
`src/grid_tests/oracle_comparison_tests.rs`,
`src/root_tests/transaction_cache_tests.rs`, and
`src/block_tests/inline_runs_tests.rs` only.

Remove the four grid source proxies:

- `fri08_c06r_inherited_placement_architecture_has_no_residual_ordinary_estimator`;
- `fri08_c02r_lanes_track_phase_architecture_has_no_collection_fit_content_shortcut`;
- `fri08_c02r_lanes_track_phase_architecture_has_one_auto_maximum_predicate`; and
- `fri08_c02r_lanes_track_phase_architecture_has_one_policy_free_final_owner`.

Remove the two root source proxies:

- `fri06_c04_float_lifecycle_static_one_ledger_query_and_single_publication_paths`;
  and
- `fri06_c03_lifecycle_unified_mixed_publication_contributes_each_geometry_once`.

Remove ignored
`layout_oracle_grid_baseline_offset_matches_oracle`: it compares the oracle to a
copied `9.0`, does not invoke a production baseline helper, and has no product
contract. Do not replace it with another copied-oracle test.

Rename
`block_line_break_conversion_with_metadata_preserves_current_output` for its
condition and independently derived line-box outcome. Preserve its real layout
stimulus, but document/express the expected RTL position and 48px block extent
from the two line boxes, authored metrics, and fixed atomic sizes rather than
from implementation output.

**Disposition:** inherited placement, lane track phases, float lifecycle, and
mixed publication retain their adjacent scalar and failure-path behavior tests.
The renamed line-break test remains behavioral; its count is unchanged.

**Acceptance:** seven library tests are removed, the line-break behavior remains
one passing test with no `current_output` name/oracle, and the ignored default
list contains only the full browser-parity tier.

Commit: `test(conformance): replace internal architecture proxies`.

### 3.3 `P01/I08/S02/R08/T03` Remove Generator Source And Workflow Proxies

**Path:** the test module in
`tests/bin/surgeist-layout-generate/generator.rs` only.

Remove these five Rust-source byte freezes:

- `fri06_c08r_lineage_parser_and_serializer_implementations_are_frozen`;
- `fri08_c05_adapter_current_helper_and_template_area_parser_are_byte_frozen`;
- `fri08_c06_settled_sources_manifest_helper_and_production_are_byte_frozen`;
- `fri08_c06_manifest_and_settled_derivation_inputs_are_exact`; and
- `fri06_c08_recovery_inputs_owned_sources_match_reviewed_freeze`.

Remove these six plan/census/entry-state tests and their now-dead entry-report or
census helpers:

- `fri06_c08_new_exact_flow_root_census_sources_use_supported_overflow_bfcs`;
- `fri06_c08_t2_reconstructs_exact_input_census_membership`;
- `fri06_c08_existing_entry_report_reconstructs_exact_activation_and_baseline_matrices`;
- `fri06_c08_existing_census_intersection_assigns_only_exact_input_rows`;
- `fri06_c08_range_ink_census_partitions_retain_exact_selectors_counts_and_digests`;
  and
- `fri06_c08_recovery_characterization_reconciles_literal_and_executed_censuses`.

Remove ignored entry-only workflow tests
`stale_artifact_inventory_matches_committed_entry` and
`worktree_is_clean_before_input_recovery`.

**Disposition:** parser/serializer behavior remains covered by direct malformed,
round-trip, lowering, serialization, generated XML, and public-layout tests.
`track_definition_serializes_subgrid_line_names` is explicitly retained as a
valid serializer contract. Import tests that create isolated temporary Git
repositories remain product behavior; only initiative entry/history/worktree
evidence is removed.

**Acceptance:** exactly 13 generator-target tests are removed; no generator
production implementation changes; no ignored generator test remains; direct
parser/serializer/import/report-schema families and strict generator gates pass.

Commit: `test(conformance): remove generator workflow proxies`.

### 3.4 `P01/I08/S02/R08/T04` Remove Browser Source And Workflow Proxies

**Paths:** `tests/layout/browser_parity.rs` and the test module in
`tests/layout/browser_parity/support.rs` only.

Remove:

- `fri06_c08r_lineage_support_has_no_name_or_expectation_compatibility_path`;
- `fri06_c08_r0_control_probe_matrix_is_exact_72_plus_24_rows`;
- `fri06_c08r_final_activation_union_browser_passes_without_substitutes`;
- `fri06_c12_t07_endpoint_accounting_is_exact`; and
- `generation_report_uses_explicit_br_unsupported_buckets`.

The final item reads a historical report through `git show`; current unsupported
bucket parsing/validation remains covered by current manifest/report consumers.
The other four are source, plan census, or final-activation workflow evidence;
their actual lowering, parser, comparator, fragment, endpoint, and public-layout
behavior families remain.

**Acceptance:** four integration tests and one support test compiled in both the
library browser-control module and integration target disappear; there is no
Rust-source or historical Git-report read; focused browser/support behavior and
full matrices pass.

Commit: `test(conformance): remove browser workflow proxies`.

### 3.5 `P01/I08/S02/R08/T05` Make Parity Evidence Behavior-Owned

**Path:** `tests/layout/browser_parity.rs` only.

Remove these 24 tests whose evidence is raw scenario/file/count/placement or
current initiative inventory:

- `fri08_c05_inputs_exact_new_source_inventory_and_finite_structure_are_settled`;
- `fri08_c06_manifest_has_exact_active_rows_without_suppression`;
- `fri08_c06_owned_rows_are_unique_and_adopt_exact_control_inventory`;
- `fri08_c06_exact_seventy_two_owned_rows_exist_parse_and_are_comment_free`;
- `fri07_c04_fixture_input_exact_six_source_four_variant_inventory_is_bounded`;
- `fri08_c06_final_inventory_preserves_fri07_expected_fail_contract`;
- `fri08_c06_final_inventory_preserves_fri04_fixture_contract`;
- `fri05_c06_computed_overflow_corpus_has_exact_44_output_inventory`;
- `fri_03_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_outputs`;
- `fri04_c05_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_paths`;
- `fri08_c08_t02_all_axis_family_inventories_paths_topologies_and_mismatches_are_stable`;
- `grid_axes_fixture_matrix_is_generated`;
- `grid_lanes_axis_fixture_matrix`;
- `subgrid_axis_fixture_matrix`;
- `flex_axis_fixture_matrix_rejects_missing_duplicate_misplaced_and_leaf_lowered_topology_paths`;
- `grid_axis_fixture_matrix_rejects_invalid_paths_and_topology`;
- `grid_lanes_axis_fixture_matrix_rejects_invalid_paths_and_topology`;
- `subgrid_axis_fixture_matrix_rejects_invalid_paths_and_topology`;
- `block_axis_fixture_matrix_rejects_missing_duplicate_misplaced_and_topology_bypassed_paths`;
- `calc_fixture_family_rejects_misplaced_duplicate_variant`;
- `browser_parity_corpus_manifest_exists`;
- `browser_parity_html_corpus_inventory_is_documented`;
- `fri08_c06_generation_report_counts_full_scope`; and
- `browser_parity_generation_report_inventory_is_full_only`.

Delete helpers used only to synthesize expected path/census sets. Retain manifest,
report, provenance, local-path, XML parser, and referential-integrity tests that
consume declared artifacts. Retain every actual parse-and-layout comparison.
Rename/refactor count-named behavioral tests such as the active-output production
comparison and finite collapse ordinary/Chrome comparisons so their oracle is the
parsed fixture/manifest contract and layout result, not an asserted scenario
count. Do not remove behavior merely because its old name contains a count.

**Acceptance:** exactly 24 integration tests are removed; actual block/flex/grid/
lanes/subgrid/calc/collapse/overflow parse-and-layout families remain nonzero and
green; no test asserts raw fixture existence/count/placement; declared-artifact
consumer tests and `check-corpus` remain green.

Commit: `test(conformance): make parity evidence behavior owned`.

### 3.6 `P01/I08/S02/R08/T06` Consolidate Declared Generator Artifacts

**Path:** the test module in
`tests/bin/surgeist-layout-generate/generator.rs` only.

Remove these 22 current-inventory, lineage, digest, or marker-census tests and
their now-dead helpers/constants:

- `fri05_c06_manifest_freeze_requires_full_only_final_inventory`;
- `fri05_c06_manifest_freeze_has_exact_active_sources_and_final_buckets`;
- `fri04_c05_unsupported_report_projection_matches_published_digest`;
- `fri07_c04_fixture_sources_have_exact_owned_inventory_and_behavior_contract`;
- `fri07_c04_fixture_sources_map_to_exact_standard_twenty_four_variant_paths`;
- `fri07_c04_manifest_freezes_exact_three_active_and_three_expected_fail_cases`;
- `fri07_c04_corpus_has_exact_final_inventory_and_centralized_provenance`;
- `fri06_c08_t2_manifest_owns_exact_active_four_variant_matrix_and_counts`;
- `fri06_c08_t2_sources_have_exact_inventory_and_corrected_finite_behavior_facts`;
- `fri05_c06_fixture_sources_have_exact_owned_inventory_and_behavior_contract`;
- `fri05_c06_fixture_sources_map_to_exact_standard_four_variant_matrix`;
- `fri08_c06_base_lineage_hashes_and_preserved_bodies_remain_unchanged`;
- `fri06_c08r_lineage_environment_is_unfiltered_and_default_rooted`;
- `fri08_c06_lineage_provenance_inputs_match_settled_values`;
- `fri06_c08r_lineage_nine_html_inputs_are_byte_frozen`;
- `fri08_c06_final_report_and_xml_lineage_are_complete_and_exception_free`;
- `fri08_c06_final_lineage_report_closes_inventory_and_provenance`;
- `fri08_c06_final_lineage_preserves_base_nonactivation_xml_semantics`;
- `fri06_c08r_final_lineage_accounts_for_exact_marker_outputs`;
- `fri06_c08r_fixture_input_exact_source_marker_inventory_is_authored`;
- `fri06_c12_t07_default_block_parent_inventory_authors_block_role`; and
- `fri06_c12_t07_exact_nine_source_marker_inventory_has_four_scoped_bidi_records`.

Refactor/rename
`centralized_provenance_accepts_current_exact_inventory` to accept a
manifest-described synthetic inventory rather than freezing current repository
membership. Retain generic corpus-manifest schema, generation-report schema/hash/
referential-integrity, deterministic report/XML serialization, check-corpus,
import-Taffy, locking, helper runtime, parser, serializer, and generated-format
contracts.

**Acceptance:** exactly 22 more generator-target tests are removed and one is
renamed/refactored without count change. Together with T03, the generator binary
target discovers exactly 351 tests, all passing and none ignored. No production
generator code or declared artifact changes.

Commit: `test(conformance): consolidate declared artifacts`.

## 4 Task And Cycle Gates

Each task begins on clean `main`, records its exact base/head, proves its assigned
entry tests exist before mutation, and proves every named deletion and any
now-dead helper is absent afterward. A deletion-only conformance task does not
invent a behavioral RED: the valid entry evidence is the named prohibited test
executing on the base plus the external conformance predicate reporting it. The
GREEN is absence of the prohibited evidence with its independently authorized
replacement families still passing.

Each task runs locked/offline focused families, the affected target, full default
package tests, strict default Clippy, formatting, diff check, exact path scope,
no-new-suppression, and complete owned-Rust unsafe scans. T03 and T06 also run
the exact generator binary target and strict generator-feature Clippy. No worker
invokes browser execution, generation, import/acquisition, selected Dylint, or
`cargo clean`.

For each task, set exact `task_base`, `task_head`, and `task_id`, then run:

```sh
set -e
case "$task_id" in
  T01) allowed='^(src/(lib_tests|contract_tests|cache_tests)\.rs|src/scroll/construction\.rs)$' ;;
  T02) allowed='^(src/grid_tests/(lanes_subgrid|oracle_comparison)_tests\.rs|src/root_tests/transaction_cache_tests\.rs|src/block_tests/inline_runs_tests\.rs)$' ;;
  T03|T06) allowed='^tests/bin/surgeist-layout-generate/generator\.rs$' ;;
  T04) allowed='^tests/layout/browser_parity(\.rs|/support\.rs)$' ;;
  T05) allowed='^tests/layout/browser_parity\.rs$' ;;
  *) exit 1 ;;
esac
task_paths="$(git diff --name-only "$task_base..$task_head" | LC_ALL=C sort -u)"
test -n "$task_paths"
test -z "$(printf '%s\n' "$task_paths" | rg -v "$allowed")"
git diff --check "$task_base..$task_head"
```

For `src/scroll/construction.rs`, generator, and parity support, the production
prefix before the existing test-module marker remains byte-identical to the task
base. Reviewers compare the two prefixes directly; moving the marker or changing
production to evade that comparison is a defect. The exact markers are
`#[cfg(test)]\npub(super) mod fri05_c02_factory_tests {`,
`#[cfg(test)]\n#[path = "../../layout/browser_parity/support.rs"]`, and
`#[cfg(test)]\nmod tests {`, respectively.

The exact cycle path gate is:

```sh
cycle_paths="$(git diff --name-only \
  97a93e935a42a62d931192697abbafe382054806..HEAD | LC_ALL=C sort -u)"
test -z "$(printf '%s\n' "$cycle_paths" | rg -v \
  '^(plans/cycles/P01-I08-S02-R08-testing-conformance\.md|src/(lib_tests|contract_tests|cache_tests)\.rs|src/scroll/construction\.rs|src/grid_tests/(lanes_subgrid|oracle_comparison)_tests\.rs|src/root_tests/transaction_cache_tests\.rs|src/block_tests/inline_runs_tests\.rs|tests/bin/surgeist-layout-generate/generator\.rs|tests/layout/browser_parity(\.rs|/support\.rs))$')"
test "$(git diff 97a93e935a42a62d931192697abbafe382054806..HEAD \
  -- Cargo.toml Cargo.lock README.md Justfile src/lib.rs tools scripts \
     tests/layout/browser_parity/corpus.toml \
     tests/layout/browser_parity/html \
     tests/layout/browser_parity/xml \
     tests/layout/browser_parity/scripts | wc -l | tr -d ' ')" = 0
```

After all six independently CLEAN task ranges, set this plan to `complete` and
run the final inventory. It must be:

| Target/prefix | Final count |
| --- | ---: |
| library | 2,043 |
| `grid::tests::` | 1,011 |
| `root_tests::` | 233 |
| `flex_tests::` | 169 |
| `block_tests::` | 212 |
| integration target | 215 |
| default package including 72 doctests | 2,330 |
| generator binary target | 351 |

The exact final ignored list is only
`runs_all_checked_in_browser_parity_xml`, with its existing explicit reason and
runnable gate `CARGO_NET_OFFLINE=true just parity-all` (not invoked in this
browser-free cycle).

Final static conformance is workflow evidence outside `cargo test`. It proves:

```sh
set -e
test -z "$(rg -n --glob '*.rs' \
  'include_str!\([^)]*\.rs|read_to_string\([^\n]*\.rs|file!\(\)' src tests || true)"
test -z "$(rg -n --glob '*.rs' \
  'plans/|CYCLE_BASE|byte_frozen|current_output|architecture_has|has_one_owner|worktree_is_clean|stale_artifact_inventory' \
  src tests || true)"
test -z "$(rg -n --glob '*.rs' \
  'git show|entry-only|final-lineage evidence|reviewed .* census|published digest' \
  src tests || true)"
test "$(rg -n --glob '*.rs' '#\[ignore\b' src tests | wc -l | tr -d ' ')" = 1
rg -n --glob '*.rs' '#\[ignore\b' src tests \
  | rg 'runs_all_checked_in_browser_parity_xml|browser_parity.rs'
```

Any match that belongs to product import behavior or prose is adjudicated by the
reviewer against the testing reference; do not weaken the scan silently. The
reviewer also inspects every surviving test family and every task's exact
disposition list, because lexical absence alone is not conformance.

If the pinned Taffy checkout is absent after R07 cleanup, the coordinator may
reuse the user's exact preapproval and run only:

```sh
CARGO_NET_OFFLINE=true cargo run --locked --offline -p surgeist-layout \
  --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy
```

It must prove checkout HEAD
`d1ff7e339b9ee35b33858779f8d7653197e93d92`, clean checkout, and no repository
delta.

The complete final matrix is:

```sh
set -e
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout \
  --features layout-golden-generate --bin surgeist-layout-generate
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; \
  CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" \
  cargo +nightly-2026-05-28 test --locked --offline)
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; \
  CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" \
  RUSTFLAGS='-F unsafe-code -D warnings' \
  cargo +nightly-2026-05-28 check --locked --offline --all-targets)
(cd tools/surgeist-layout-audits && cargo +stable fmt --check)
cargo fmt --check
git diff --check
```

The selected R06A Dylint audit does not run. The matrix also proves public source
and API compatibility through compilation and external compile contracts;
unchanged Cargo manifests/lock, dependencies/features/MSRV, README, Justfile,
production `src/lib.rs` and algorithms, scripts, catalog, fixtures, and generated
artifacts; no new suppression; no owned unsafe; frozen hashes
`c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`
(`corpus.toml`),
`c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`
(helper), and
`c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`
(`all.json`); 1,448 HTML and 5,776 comment-free XML; exact six-task path
scope; and clean Git state.

After independent holistic CLEAN, rerun the complete matrix at the exact reviewed
head, publish immutable `main` with an explicit lease, and read back local,
tracking, and authority-remote equality. Prove no cycle-owned process remains,
run repository-root `cargo clean`, and prove both target paths absent and Git
clean. Record the final FRI-08 handoff and the still-paused reviewed FRI-09
sequence. Do not begin FRI-09 implementation or create the future cross-crate
skill reference.
