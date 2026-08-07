# P01-I06-S01-C12 Final Production Correction And Lineage

Status: in_progress

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/specs/P01-I06-inline-formatting-floats-bfcs.md`, normalized semantic-content
SHA-256 `ac08d4c8df8a8da7cd0698eb2618f1fa5478c5b477bdedc07800543804fbd9ea`,
commit `9cbd01560705b7d81579804eedf904999b82ee0c`: `FRI-06.3`,
`FRI-06.4 D-16`, `D-18`, and `D-19`; the private subgrid carrier/error contract
in `FRI-06.6`; control and subgrid portions of `FRI-06.7`; module/test contracts
in `FRI-06.9` and `.10`; browser/comparator/artifact contracts in `FRI-06.11`
through `.11.3`; and acceptance in `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`, normalized
SHA-256 `7d1cca5c49bdd7349bdaa402abb4db98dc39380a7b6f9fe86fcb5575ba9391eb`,
commit `3a208b983cb9449b9e095748d550b268f6890866`, entry
`P01/I06/S01/C12`.

## 1 Outcome

Preserve the task-clean T01-T06 results and the sole successful full browser
lineage. Reopen T08 first to complete D-18's settled axis-parametric baseline
placement. Then reopen T07 for D-19's typed endpoint-unobservable comparator
state and independent private line-membership proof. Finally adopt the
already-generated artifacts and exact evidence constants without another full
or scoped generation.

## 2 Boundary And Current Evidence

T01-T06 remain task-clean under their recorded ordered ranges. T07's reviewed
ordered implementation ranges are:

- `111492fbcb2df823e4b110f6199c929dd7478237..323d73afa98ddc73e65fd9c1da223a5fbd85875e`;
- `79d8c92043f5325bfe3d11d969974cf92a75279d..d42a667494055e3ed4bba4b8502220a214b97ef4`;
- `d6b0c1e3668665ba1747083cc76394125df9f137..89adbbc29ba3b2350c1fb64876a8a69520af8e07`;
- `bfd588ef8b28394bb2bed501240a08fd53ccc805..2ed9382d0f3b2f47c5701aa5290e26567b44ac3a`;
- `2ed9382d0f3b2f47c5701aa5290e26567b44ac3a..17fffd9374647633eb0a7dcd1ecbf56b0ed8a37c`;
  and
- `fb7335c47d2c160f9fa787e73d3ac750db3a75da..d58bc9e8e42ffd0f74fa5342556680f81f2b3d84`.

Those ranges already preserve honest layout-ready fixture input, browser-measured
BR metrics, finite physical baseline distances, and the closed interval relation.
D-19 appends one comparator/test-support correction range; it does not invalidate
those results.

T08 was task-clean at `8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`
before the full-fixture diagnosis. Its complete historical lineage is:

- `89adbbc29ba3b2350c1fb64876a8a69520af8e07..9ff1b91dabd7d53b32ee0942a7e6962515a80b79`;
- `9ff1b91dabd7d53b32ee0942a7e6962515a80b79..5f7f72c45090d9c230f7a2957bffadd5904625b4`;
- `a64b3272c675e52fecec61fa9617c9e972e2b514..e36830143235e28625ac010489d8c7aa998d714f`;
- `e36830143235e28625ac010489d8c7aa998d714f..f2a3e0485adbc63521276f688ddf7e1f71fa448e`;
- `f2a3e0485adbc63521276f688ddf7e1f71fa448e..e367a493f4d6b574a1d1a53b31314528a5e5a213`;
- `e367a493f4d6b574a1d1a53b31314528a5e5a213..8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`;
- `34294ee6d50d8e685a7b09f9b9a7a8671d62af29..881fd361c892cc6c043c8485f2ed7aad391b0392`;
  and
- `881fd361c892cc6c043c8485f2ed7aad391b0392..bf56ed87d537c484c3418bf2d9d2d0404aaaabcd`.

The first six ranges were task-clean. Review rejected the seventh because
reversal filtered a settled target and row-gap compensation occurred after
placement; review rejected the eighth because it cloned and mutated an
already-reduced group based on target inequality. A third no-commit attempt
removed that mutator, but pre-reduction and child-view variants violated existing
intrinsic or mapping controls; all diagnostic residue was removed. D-18 therefore
reopened at architecture/specification, and T08 now appends a ninth correction
range without discarding or rewriting history.

After the final T07 helper correction, one full unfiltered generation completed
successfully and remains in the canonical worktree as exactly 5,712 changed XML
files plus `xml/generation-reports/all.json`. Its report has `filter: null`, 5,712
generated, the exact 16 missing-root unsupported rows, and zero expected-fail,
quarantined, failed, or failed-to-generate rows. No generator process or scoped
report remains. Its evidence is:

| Evidence | SHA-256 |
| --- | --- |
| Browser helper | `42bf9ff77810b2e9fb5a184f525d9e22f74abae12a09f9486b3b49dc620188c2` |
| Full report | `8d59c87d1fcc185bda0372968ae81dbeff74f241c17335db98629ad49f1f463f` |
| Complete XML | `d2530aa79f214b536e46aee263095a6e7c0a1d7a329bdce7baeb194af3670896` |
| Activation bodies | `f3d9b41973e6b0e51e258f027496dc2651c4fba7d24567b05d4f088ee63de335` |
| Preserved bodies | `b2684877302ed7b1b6b1e52b5ae4c4ae4508ff425d6c34ff237b7e37440a3c79` |
| Inventory | `0c327c2d93b140ea5ed5660e45ad947a0afb583b9aa97b3163ea59b45d371715` |

The lineage closes the four unequal-line block-height rows. Activation is now
244 passing and 144 first-reported failures, partitioned exactly as 48
`subgrid_baseline_inline_column_*`, 48
`subgrid_baseline_vertical_auto_rows_*`, and 48
`subgrid_baseline_vertical_nested_*` variants. Every failure is only serialized
browser `next_line = Later` versus model closed-overlap `Same`. Temporarily
classifying that endpoint exposed ordinary x-coordinate mismatches in the same
files, so D-19 cannot close before D-18.

The D-18 diagnosis used temporary instrumentation only and left no residue. It
proved parsed layout inputs and final `FlowAxes` projection honest, then located
three production divergences:

- inline-column groups are produced as `[(112, 65), (80, 73), (75, 90)]`, but
  only row groups feed placement; column groups do not replace the logical inline
  offset. Direct block controls first diverge at x `470` versus `415`, and
  `415` versus `470`; six direct flex siblings are also wrong;
- vertical auto-row placement requires settled rows `[212, 194, 194]`, outer x
  `196` with width `371`, nested first x `308`, and nested last x `222`, while
  current track placement produces rows `[186, 203, 211]` and nested last x
  `206`; and
- vertical nested fixed tracks are correct, but the direct-member targets must be
  `[(66, 15), (43, 32), (38, 40)]`; all six later direct siblings have the same
  five-pixel half-gutter error.

The owners are `src/grid/tracks.rs`, `src/grid/subgrid.rs`, and
`src/grid/child.rs`. The correction retains complete immutable per-track target
records, then chooses either owner-direct consumption or a distinct checked
`InheritedCurrentGridBaselinePlacement`. For span `[start,end)`, that placement
derives local track as `selected-start` or `end-1-selected` under reversal,
maps First/Last to Start/End and swaps the edge under reversal, and applies
`edge_sign * (current_gap-parent_gap)/2` only when the mapped role edge crosses a
gutter. It derives every value internally from the group, mapping, and direct-item
witness. MBP remains in the owner target. A `ChildBaselineEnvelopeView` is solely
the downward child-internal phase. No target-value applicability test, group
mutation, fixed-point loop, publication inverse, parser/fixture change, or
generation is permitted.

The representative vertical-rl model geometry is previous atomic `[55, 75]`,
zero-size control `[55, 55]`, and next atomic `[35, 55]`. Closed overlap correctly
classifies both relations `Same`. Chrome independently reports its non-model BR
rectangle `[60, 70]` and categorical next-line effect `Later`. Production,
browser observation, model control geometry, neighboring node geometry, and the
closed relation are all correct; only the comparator equates distinct evidence
domains.

One failed D-19 correction attempt and one diagnostic assignment produced no
commit and left no source or test residue. The immutable cycle base predates the
realized C12 tests. Reopened task commands resolve against the current entry
state: the activation test and final-lineage freeze tests are preserved T07/T09
code already present at the plan commit.

## 3 Known Chrome Measurement Failures

None. This is not a Chrome failure, expected-fail, quarantine, or synthetic
substitute. Chrome's observation remains serialized and exact neighboring node
geometry remains directly compared.

## 4 Impacts

- **Public API and compatibility:** unchanged; D-18 changes private layout
  production and D-19 changes private test support.
- **Production/tests:** T08 may change `src/grid/tracks.rs`,
  `src/grid/subgrid.rs`, `src/grid/child.rs`, and `src/grid_tests.rs` only;
  existing strict fixture proof in `tests/layout/browser_parity.rs` is read-only
  acceptance.
- **Comparator/tests:** T07 may change
  `tests/layout/browser_parity/support.rs`,
  `tests/layout/browser_parity.rs`, and `src/inline_tests.rs` only.
- **Helper/parser/fixtures/generator logic:** unchanged; no HTML, JavaScript,
  fixture, parser, serializer, or generator-architecture change.
- **Generated artifacts:** T09 adopts the preserved 5,712 XML files and report
  byte-for-byte and updates only their Rust evidence constants.
- **Dependencies, features, docs, examples, MSRV, root:** unchanged.
- **Safety:** no `unsafe`, lint suppression, parser layer, public test API, or
  later-owned behavior is permitted.

## 5 Tasks

### 5.1 `P01/I06/S01/C12/T08` Close Settled Axis-Parametric Baseline Placement

**Files/area:** `src/grid/tracks.rs`, `src/grid/subgrid.rs`,
`src/grid/child.rs`, and `src/grid_tests.rs`. Existing browser-parity tests and
artifacts are read-only. Do not edit comparator support, helper, parser, fixtures,
generator logic/output, manifests, or public API.

**RED:** Add the exact specification tests first:

- `inherited_current_grid_baseline_placement_maps_row_and_column_half_gaps`;
- `inherited_current_grid_baseline_placement_maps_first_and_last_edges_through_reversal`;
- `inherited_current_grid_baseline_placement_is_zero_at_role_terminal_edges`;
- `inherited_current_grid_baseline_placement_is_zero_for_equal_gaps_and_owner_direct_items`;
- `inherited_current_grid_baseline_placement_keeps_mbp_in_base_mapping`;
- `inherited_current_grid_baseline_placement_repeat_is_identical_and_mutates_no_input`;
- `vertical_auto_rows_current_grid_first_moves_x126_to_x121_while_last_stays_x30`;
- `inherited_current_grid_baseline_placement_rejects_axis_mismatch_first`;
- `inherited_current_grid_baseline_placement_rejects_physical_axis_mismatch`;
- `inherited_current_grid_baseline_placement_rejects_span_out_of_range`;
- `inherited_current_grid_baseline_placement_rejects_selected_track_out_of_range`;
- `inherited_current_grid_baseline_placement_rejects_role_target_mismatch`;
- `inherited_current_grid_baseline_placement_rejects_ownership_mismatch`;
- `inherited_current_grid_baseline_placement_rejects_unusable_inherited_mapping`;
- `inherited_current_grid_baseline_placement_rejects_non_finite_last`;
- `subgrid_baseline_placement_error_propagates_with_node_site`;
- `subgrid_baseline_placement_error_propagates_with_container_subject_site`;
  and
- `late_subgrid_baseline_placement_error_after_prior_item_preparation_mutates_no_item_output_or_batch`.

Missing target records/checked placement or the legacy cloned-group mutator must
produce RED; no artifact edit, comparator exception, or generation supplies it.

**Outcome:** Preserve the pre-growth census and settled reduction, but replace
scalar-only group slots with complete immutable target records whose strictly
larger candidate replaces the whole record and whose equal candidate retains the
earliest record. Derive checked inherited-axis mapping and current-grid placement
internally by the Section 2/specification formula; owner-direct consumers use the
group target, current-grid direct consumers use placement, and only child-internal
layout uses the envelope view. Remove the cloned-group mutator and all target-value
ownership/applicability inference. Preserve non-inherited behavior and one final
`FlowAxes` projection.

**Acceptance:** The canonical target/offset matrix is exact for row/column,
first/last, reversed/non-reversed, positive/equal/negative half-gaps, terminal
edges, owner-direct items, and MBP; repeated derivation changes no input. Ordered
compound-invalid cases, node/container-subject propagation, and late failure are
atomic. Inline-column x `470/415` and RTL root-first x `527`, vertical auto rows
`[212,194,194]` with outer x/width `196/371`, nested x `308/222`, vertical fixed
targets `[(66,15),(43,32),(38,40)]`, all later siblings, and all ordinary fields
of the three representative XML controls pass. Generated artifacts are
byte-identical and only existing D-19 endpoint observations remain.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c12_t08_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib inherited_current_grid_baseline_placement_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib subgrid_baseline_placement_error_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c12_t08_representative_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

**Dependency:** Append the correction span after T08's eight historical ranges
and review the complete nine-range T08 lineage. T07 remains blocked until T08 is
task-clean. Preserve the successful generated lineage unchanged.

**Intended commit:** `fix(layout): close settled subgrid baseline placement`.

### 5.2 `P01/I06/S01/C12/T07` Classify Endpoint-Unobservable Controls

**Files/area:** `tests/layout/browser_parity/support.rs`, focused activation
accounting in `tests/layout/browser_parity.rs`, and a private line-builder
regression in `src/inline_tests.rs`. Do not edit production, helper, parser,
fixtures, generator logic, or generated artifacts.

**RED:** After T08 is task-clean and all ordinary geometry is strict-green, run
the focused activation test once. It enumerates exactly 388 rows and fails
exactly 144 `next_line` comparisons: the 48/48/48 families recorded above all
report browser `Later` versus model `Same`, while 244 rows pass. Then add
`fri06_c12_t07_endpoint_unobservable_requires_exact_shared_endpoint`,
`fri06_c12_t07_endpoint_accounting_is_exact`, and
`fri06_c12_t07_endpoint_break_commits_following_atomic_to_next_line` test-first;
their missing typed result/accounting fails before the comparator correction.
No generation command supplies RED.

**Outcome:** After ordinary node geometry compares, report only `next_line` as a
typed endpoint-unobservable field when every `FRI-06.11.3` predicate holds: a
visible forced break has zero-size model output, both adjacent model neighbors
have unrounded output, its point is within `0.1` of their exact shared physical
block endpoint, closed comparison yields `Same` for both neighbors, and the
serialized browser observation distinguishes the following line. Preserve the
browser value. Do not inspect fixture identity or expected geometry to select
layout input or comparator behavior.

**Acceptance:** Focused support negative controls reject any missing predicate,
wrong/missing neighboring geometry, non-endpoint mismatch, or widened skipped
field. The private line-builder regression proves the forced break ends the prior
line and the following atomic has the next private line index. Activation accepts
all 388 rows while reporting exactly 144 typed endpoint-unobservable `next_line`
fields in the exact 48/48/48 family partition; every other field is directly
compared. The closed interval relation, model geometry, browser XML, and known-
failure/expected-fail counts remain unchanged.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c12_t07_endpoint_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c12_t07_endpoint_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_final_activation_union_browser_passes_without_substitutes -- --nocapture
CARGO_NET_OFFLINE=true just parity-all
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

**Dependency:** T08 is task-clean. Append the correction span after T07's six
reviewed ranges and review the complete seven-range T07 lineage. Preserve the
successful generated lineage unchanged.

**Intended commit:** `fix(parity): classify endpoint-unobservable controls`.

### 5.3 `P01/I06/S01/C12/T09` Adopt Final FRI-06 Lineage

**Files/area:** test-only evidence constants and digest assertions in
`tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`, plus the already-preserved 5,712 XML files and
`tests/layout/browser_parity/xml/generation-reports/all.json`. Do not run the
generator binary or edit generator logic, helper, parser, fixtures, serializer,
comparator, production, manifest, dependency, or feature surface. Minimal
test-only digest helpers and assertions in the two named Rust test files are
authorized solely to bind the six final hashes below.

**RED:** After T07 is task-clean, run the focused final-lineage freeze before
editing constants. The existing
`fri06_c08r_lineage_helper_and_nine_html_inputs_are_byte_frozen`,
`fri06_c08r_final_lineage_report_closes_inventory_and_provenance`, and
`fri06_c08r_final_lineage_preserves_nonactivation_xml_semantics` tests fail on
their stale committed helper/report-body evidence. This is adoption RED, not
permission to generate or alter artifact bodies.

**Outcome:** Update only the exact evidence constants, then commit those constants
with the preserved XML/report as the final lineage. Do not run a full or scoped
generation: D-18 production and D-19 comparator changes alter no generator input
or output, so the already-successful full run is the sole acceptance lineage.

**Acceptance:** Focused lineage tests prove `filter: null`, 5,712 generated, the
exact 16 missing-root unsupported rows, zero other buckets, one full report, the
six exact hashes above, exact 388-row activation membership, exact 144 typed
endpoint-unobservable fields, and semantic preservation of the other 5,324 XML
bodies. Full configured verification is read-only and green. No process, scoped
report, temporary artifact, helper/parser/fixture change, or second generation
exists.

`fri06_c12_t09_final_lineage_hashes_match_preserved_run` binds the hashes with
these canonical procedures: helper and report hash exact file bytes; complete XML
hashes LF-terminated sorted records containing the file SHA-256, two ASCII spaces,
and repo-relative path; inventory hashes the LF-terminated sorted corpus-relative
XML paths; and activation/preserved bodies use the existing sorted aggregate of
path, NUL byte, body without its provenance line, and NUL byte over the exact
388/5,324 partition. The expected digests are the six values in Section 2.

**Commands:**
```sh
! pgrep -f '[s]urgeist-layout-generate'
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08r_final_lineage_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t09_final_lineage_hashes_match_preserved_run
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_final_activation_union_browser_passes_without_substitutes -- --nocapture
CARGO_NET_OFFLINE=true just parity-all
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --check
```

**Dependency:** T08 and T07 are task-clean. T09's complete
ordered lineage begins with diagnostic range
`a1d165e30f6abbc3ad1759504fcd9c90dc52a709..0a355604d0862a8f07811d323acfdece912921cd`
and appends the final adoption span.

**Intended commit:** `test(parity): adopt final FRI-06 lineage`.

## 6 Cycle Completion

After T08, T07, and T09 are independently task-clean, change only `Status` to
`complete` in a separate commit. At that exact head run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just parity-all
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
git diff --check 8ffb4bc551a24d2283ad54436870ab3f5e66a473..HEAD
test -z "$(git status --short)"
```

The unsafe scan returns no match and final status is clean. Record exact task
ranges and artifact hashes, obtain the mandatory holistic review, publish and
read back authority remote `main`, clean owned temporary resources, and hand the
frozen behavior-correct candidate to C13. Blocker: none.
