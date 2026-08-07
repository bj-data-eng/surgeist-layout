# P01-I06-S01-C12 Final Production Correction And Lineage

Status: in_progress

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/specs/P01-I06-inline-formatting-floats-bfcs.md`, normalized semantic-content
SHA-256 `c94b388c1bb31b94a1b09bb287440e7ca22338da60d6bf161d7d3b7d7ec0fe38`,
commit `967f3ca9d71ddec1451e69bc6cccf60d9399d511`: `FRI-06.3`,
`FRI-06.4 D-16`, `D-18`, and `D-19`, the control, comparator, fixture, and
subgrid portions of `FRI-06.7`, module/test contracts in `FRI-06.9` and `.10`,
browser/artifact contracts in `FRI-06.11` through `.11.3`, and acceptance in
`FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`, normalized
SHA-256 `de8476bc841610e6842cf370ab5af6820322b73bb88365e0a5137b2319a95ae6`,
commit `52de2e8ddeab8b2fc3e0885828017e67b6c7d116`, entry
`P01/I06/S01/C12`.

## 1 Outcome

Preserve the task-clean T01-T06 results, D-18-clean T08 production, and the sole
successful full browser lineage. Reopen T07 only to implement D-19's typed
endpoint-unobservable comparator state and independent private line-membership
proof. Then adopt the already-generated artifacts and exact evidence constants
without another full or scoped generation.

## 2 Boundary And Current Evidence

T01-T06 remain task-clean under their recorded ordered ranges. T07's reviewed
ordered implementation ranges are:

- `111492fbcd60251e5fd71bb76a889ced0508b6b3..323d73afa98ddc73e65fd9c1da223a5fbd85875e`;
- `79d8c9204039656438083328bb3cdde80c813051..d42a667494055e3ed4bba4b8502220a214b97ef4`;
- `d6b0c1e36bb7803a7a80480cf4d85a09c7b72753..89adbbc29ba3b2350c1fb64876a8a69520af8e07`;
- `bfd588ef87c393527f2f7f8fc65b0fa41a7bd066..2ed9382d0f3b2f47c5701aa5290e26567b44ac3a`;
- `2ed9382d0f3b2f47c5701aa5290e26567b44ac3a..17fffd9374647633eb0a7dcd1ecbf56b0ed8a37c`;
  and
- `fb7335c47d2c160f9fa787e73d3ac750db3a75da..d58bc9e8e42ffd0f74fa5342556680f81f2b3d84`.

Those ranges already preserve honest layout-ready fixture input, browser-measured
BR metrics, finite physical baseline distances, and the closed interval relation.
D-19 appends one comparator/test-support correction range; it does not invalidate
those results.

T08 remains task-clean at
`8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`. Its six reviewed ordered ranges are:

- `89adbbc29ba3b2350c1fb64876a8a69520af8e07..9ff1b91dabd7d53b32ee0942a7e6962515a80b79`;
- `9ff1b91dabd7d53b32ee0942a7e6962515a80b79..5f7f72c45090d9c230f7a2957bffadd5904625b4`;
- `a64b3272c675e52fecec61fa9617c9e972e2b514..e36830143235e28625ac010489d8c7aa998d714f`;
- `e36830143235e28625ac010489d8c7aa998d714f..f2a3e0485adbc63521276f688ddf7e1f71fa448e`;
- `f2a3e0485adbc63521276f688ddf7e1f71fa448e..e367a493f4d6b574a1d1a53b31314528a5e5a213`;
  and
- `e367a493f4d6b574a1d1a53b31314528a5e5a213..8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`.

D-18 production and exact public geometry are closed. No production file is
reopened.

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
244 passing and 144 failing, partitioned exactly as 48
`subgrid_baseline_inline_column_*`, 48
`subgrid_baseline_vertical_auto_rows_*`, and 48
`subgrid_baseline_vertical_nested_*` variants. Every failure is only serialized
browser `next_line = Later` versus model closed-overlap `Same`.

The representative vertical-rl model geometry is previous atomic `[55, 75]`,
zero-size control `[55, 55]`, and next atomic `[35, 55]`. Closed overlap correctly
classifies both relations `Same`. Chrome independently reports its non-model BR
rectangle `[60, 70]` and categorical next-line effect `Later`. Production,
browser observation, model control geometry, neighboring node geometry, and the
closed relation are all correct; only the comparator equates distinct evidence
domains.

The immutable cycle base predates the realized C12 tests. Reopened task commands
resolve against the current entry state: the activation test and final-lineage
freeze tests are preserved T07/T09 code already present at the plan commit. Only
the three D-19 tests named below are new.

## 3 Known Chrome Measurement Failures

None. This is not a Chrome failure, expected-fail, quarantine, or synthetic
substitute. Chrome's observation remains serialized and exact neighboring node
geometry remains directly compared.

## 4 Impacts

- **Public API and compatibility:** unchanged; D-19 is private test support.
- **Production:** unchanged; no `src/` behavior changes except a private unit
  regression in `src/inline_tests.rs`.
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

### 5.1 `P01/I06/S01/C12/T07` Classify Endpoint-Unobservable Controls

**Files/area:** `tests/layout/browser_parity/support.rs`, focused activation
accounting in `tests/layout/browser_parity.rs`, and a private line-builder
regression in `src/inline_tests.rs`. Do not edit production, helper, parser,
fixtures, generator logic, or generated artifacts.

**RED:** Against the preserved successful lineage, run the focused activation
test once. It enumerates exactly 388 rows and fails exactly 144 `next_line`
comparisons: the 48/48/48 families recorded above all report browser `Later`
versus model `Same`, while 244 rows pass. Then add
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

**Dependency:** Append the correction span after T07's six reviewed ranges and
review the complete seven-range T07 lineage. Preserve T08 and the successful
generated lineage unchanged.

**Intended commit:** `fix(parity): classify endpoint-unobservable controls`.

### 5.2 `P01/I06/S01/C12/T09` Adopt Final FRI-06 Lineage

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
generation: D-19 changes no generator input or output, so the already-successful
full run is the sole acceptance lineage.

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

**Dependency:** T07 is task-clean and T08 remains preserved-clean. T09's complete
ordered lineage begins with diagnostic range
`a1d165e30f6abbc3ad1759504fcd9c90dc52a709..0a355604d0862a8f07811d323acfdece912921cd`
and appends the final adoption span.

**Intended commit:** `test(parity): adopt final FRI-06 lineage`.

## 6 Cycle Completion

After T07 and T09 are independently task-clean, change only `Status` to
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
