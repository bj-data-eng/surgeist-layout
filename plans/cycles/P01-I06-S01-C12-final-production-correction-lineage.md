# P01-I06-S01-C12 Final Production Correction And Lineage

Status: complete

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/specs/P01-I06-inline-formatting-floats-bfcs.md`, normalized semantic-content
SHA-256 `6135d2c542967d938508f6a15b9940dbe02b3b15221109911bbad611c67c25a6`,
commit `45b6a9b987b37328963c8c1feff67368e91de70b`: `FRI-06.3`,
`FRI-06.4 D-16`, `D-18`, and `D-19`; the private subgrid carrier/error contract
in `FRI-06.6`; control and subgrid portions of `FRI-06.7`; module/test contracts
in `FRI-06.9` and `.10`; browser/comparator/artifact contracts in `FRI-06.11`
through `.11.3`; and acceptance in `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`, normalized
SHA-256 `43f9443afe6297d493967c640ccb13e23d0c6532a2529ec3aa99bb41f327a393`,
commit `89f9c02d23cba29d51a1f7e5aa88d85f6c9ef5ee`, entry
`P01/I06/S01/C12`.

## 1 Outcome

Preserve the task-clean T01-T06 results and the sole successful full browser
lineage. Reopen T08 first to complete D-18's settled axis-parametric baseline
placement. Then reopen T07 for D-19's typed endpoint-unobservable comparator
state and independent private line-membership proof. Finally adopt the
already-generated artifacts and exact evidence constants, then reconcile the
stale Rust-only recovery evidence exposed by adoption, without generation.

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
D-19 appends one comparator/test-support correction range under Section 5.2's
files/area boundary; that boundary does not retroactively narrow the six historical
ranges or invalidate their separately reviewed contracts and results.

T08's complete historical lineage through the activation-invalidated current
implementation is:

- `89adbbc29ba3b2350c1fb64876a8a69520af8e07..9ff1b91dabd7d53b32ee0942a7e6962515a80b79`;
- `9ff1b91dabd7d53b32ee0942a7e6962515a80b79..5f7f72c45090d9c230f7a2957bffadd5904625b4`;
- `a64b3272c675e52fecec61fa9617c9e972e2b514..e36830143235e28625ac010489d8c7aa998d714f`;
- `e36830143235e28625ac010489d8c7aa998d714f..f2a3e0485adbc63521276f688ddf7e1f71fa448e`;
- `f2a3e0485adbc63521276f688ddf7e1f71fa448e..e367a493f4d6b574a1d1a53b31314528a5e5a213`;
- `e367a493f4d6b574a1d1a53b31314528a5e5a213..8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`;
- `34294ee6d50d8e685a7b09f9b9a7a8671d62af29..881fd361c892cc6c043c8485f2ed7aad391b0392`;
- `881fd361c892cc6c043c8485f2ed7aad391b0392..bf56ed87d537c484c3418bf2d9d2d0404aaaabcd`;
- `5db1d79e74614e58ab3b7c0ca3b2e47f9a62e017..85ec38151e675bc20a57370a4a9bbfe93b6f53aa`;
  and
- `85ec38151e675bc20a57370a4a9bbfe93b6f53aa..44fe9ff42c1bf3e466a307d59751de2ef5589e67`.

The first six ranges were task-clean. Review rejected the seventh and eighth for
post-placement compensation and cloned-group target inference. The ninth and
tenth preserve a raw target and passed focused review, but mandatory T07 entry
activation invalidated them: the parent-context carrier replaces the actual owner
group at each nesting level and retains only the newest one-boundary mapping. T08
therefore appends an eleventh correction range without rewriting history.

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

The lineage closes the four unequal-line block-height rows. At `44fe9ff42`, full
activation has 48 horizontal `subgrid_baseline_auto_rows_*` ordinary y mismatches
(`113` versus `128`) and 24 RTL `subgrid_baseline_inline_column_*` ordinary x
mismatches (`230` versus `265`) before endpoint classification. LTR inline-column
x `265` and vertical-auto-row x `308` remain correct. Once ordinary geometry is
strict, D-19 must expose exactly 144 endpoint-only fields partitioned as 48
inline-column, 48 vertical-auto-row, and 48 vertical-nested.

The residue-free diagnosis proved honest parsed input and final projection. For
horizontal auto rows, immutable owner target `53`, composed frame `-20`, and
accumulated gutter `+10` yield target `43`, offset `18`, and y `128`; the current
local envelope yields y `113`. For RTL inline columns, `53-18+0=35` yields offset
`10` and x `265`; the envelope yields x `230`. The unchanged LTR/vertical controls
prove placement cannot be unconditional or reconstructed from envelopes.

The owners are `src/grid/mod.rs`, `src/grid/subgrid.rs`, and `src/grid/child.rs`;
`src/grid/tracks.rs` changes only if the typed owner identity cannot be supplied
from existing settled reduction. The correction preserves one immutable typed-
owner group and a composed owner-to-current map through every parent context.
Each local track carries one owner track plus role-specific frame and separately
accumulated gutter translations derived only from checked spans, reversal,
progression, MBP, track frames, and gaps. Typed owner/current identity selects
owner-direct or inherited placement; only child-internal layout uses the envelope.
No placement flag, target comparison, local-group substitution, mutation,
fixed-point loop, publication inverse, parser/fixture change, or generation is
permitted.

The representative vertical-rl model geometry is previous atomic `[55, 75]`,
zero-size control `[55, 55]`, and next atomic `[35, 55]`. Closed overlap correctly
classifies both relations `Same`. Chrome independently reports its non-model BR
rectangle `[60, 70]` and categorical next-line effect `Later`. Production,
browser observation, model control geometry, neighboring node geometry, and the
closed relation are all correct; only the comparator equates distinct evidence
domains.

The repository-wide ignored `parity-all` diagnostic is not an FRI-06 acceptance
gate. At the current entry it reports 372 failures outside the exact 388-row
FRI-06 activation set; the preserved full lineage closes the four stale BR-height
rows from the committed corpus, and the activation set itself is strict-green.
C12 neither suppresses nor claims those 372 rows. `FRI-06.14` requires focused
FRI-06 parity plus default/generator, corpus, and Taffy verification, so the
diagnostic cannot widen this cycle into later-owned layout behavior.

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
- **Production/tests:** T08 may change `src/grid/mod.rs`, `src/grid/tracks.rs`,
  `src/grid/subgrid.rs`, `src/grid/child.rs`, and `src/grid_tests.rs` only;
  existing strict fixture proof in `tests/layout/browser_parity.rs` is read-only
  acceptance.
- **Comparator/tests:** T07 may change
  `tests/layout/browser_parity/support.rs`,
  `tests/layout/browser_parity.rs`, and `src/inline_tests.rs` only.
- **Recovery evidence:** T10 may update stale Rust-only report, helper, and
  synthetic-fixture assertions in the two T09 test files after adoption.
- **Helper/parser/fixtures/generator logic:** unchanged; no HTML, JavaScript,
  fixture, parser, serializer, or generator-architecture change.
- **Generated artifacts:** T09 adopts the preserved 5,712 XML files and report
  byte-for-byte and updates only their Rust evidence constants.
- **Dependencies, features, docs, examples, MSRV, root:** unchanged.
- **Safety:** no `unsafe`, lint suppression, parser layer, public test API, or
  later-owned behavior is permitted.

## 5 Tasks

### 5.1 `P01/I06/S01/C12/T08` Close Settled Axis-Parametric Baseline Placement

**Files/area:** private context plumbing in `src/grid/mod.rs` plus
`src/grid/tracks.rs`, `src/grid/subgrid.rs`, `src/grid/child.rs`, and
`src/grid_tests.rs`. Existing browser-parity tests and artifacts are read-only.
Do not edit comparator support, helper, parser, fixtures, generator logic/output,
manifests, or public API.

**RED:** Add
`owner_to_current_placement_map_identity_has_zero_translation`,
`owner_to_current_placement_map_composes_two_boundaries_by_track_and_role`,
`owner_to_current_placement_map_composes_reversal_and_physical_progression`,
`owner_to_current_placement_map_keeps_mbp_in_frame_not_gutter_translation`, and
`owner_to_current_placement_map_accumulates_positive_zero_and_negative_half_gaps`
first. Add the exact A/B/C/D front-door tests named in D-18 and the three map
identity/cardinality/range negative controls. They fail at `44fe9ff42` because no
typed composed map exists and the A/B coordinates remain y `113`/x `230`.
Preserve and run the existing placement error-order, propagation, repeatability,
late-atomicity, one-pass sizing, and envelope-separation suites. No artifact edit,
comparator exception, or generation supplies RED.

**Outcome:** Give the settled group typed owner identity and create its zero-
translation identity map. Parameterize the `src/grid/mod.rs` parent-context
carrier by node identity and retain exactly that immutable group plus the composed
map, without local reduced groups or placement-history flags. At each inherited
boundary, compose local-to-owner track mapping and role-specific frame/gutter
translations from checked span, reversal, progression, MBP, track-frame, and gap
facts. Owner-direct items use the raw target; different current-grid direct items
always use checked placement; child-internal layout alone uses the envelope.
Preserve non-inherited behavior, one-pass sizing, and one final projection.

**Acceptance:** Identity and two-boundary composition, row/column roles,
reversal/progression, positive/equal/negative half-gaps, MBP/frame separation,
repeatability, error precedence/propagation, and late failure are exact and
atomic. Horizontal `53-20+10=43` places offset `18` at y `128` without moving the
first row from y `13`; RTL `53-18+0=35` places offset `10` at x `265`; LTR x `265`
and vertical x `308` remain unchanged. Existing intrinsic, owner-direct, nested,
vertical-fixed, sibling, and envelope controls pass. One diagnostic activation
run has zero ordinary mismatches and only the exact 144 D-19 endpoint fields;
artifacts are byte-identical.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c12_t08_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib owner_to_current_placement_map_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib inherited_current_grid_baseline_placement_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib subgrid_baseline_placement_error_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c12_t08_representative_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_final_activation_union_browser_passes_without_substitutes -- --nocapture
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

**Dependency:** Append the correction span after T08's ten historical ranges and
review the complete eleven-range T08 lineage. T07 remains blocked until T08 is
task-clean. Preserve the successful generated lineage unchanged.

**Intended commit:** `fix(layout): compose inherited baseline placement frames`.

### 5.2 `P01/I06/S01/C12/T07` Classify Endpoint-Unobservable Controls

**Files/area:** the newly appended D-19 correction may change only
`tests/layout/browser_parity/support.rs`, focused activation accounting in
`tests/layout/browser_parity.rs`, and a private line-builder regression in
`src/inline_tests.rs`. It must not edit production, helper, parser, fixtures,
generator logic, or generated artifacts. The six historical T07 ranges remain
governed by their separately reviewed contracts recorded in Section 2.

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
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

**Dependency:** T08 is task-clean. Append the correction span after T07's six
reviewed ranges and review the complete seven-range T07 lineage against each
historical range's reviewed contract plus this D-19 correction contract. Preserve
the successful generated lineage unchanged.

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
bodies. Focused activation, corpus, and Taffy verification are read-only and
green. Broader Rust suites may retain only the exact stale T10 evidence below.
No process, scoped report, temporary artifact, helper/parser/fixture change, or
second generation exists.

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
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --check
```

**Dependency:** T08 and T07 are task-clean. T09's complete
ordered lineage begins with diagnostic range
`a1d165e30f6abbc3ad1759504fcd9c90dc52a709..0a355604d0862a8f07811d323acfdece912921cd`
and appends the final adoption span.

**Intended commit:** `test(parity): adopt final FRI-06 lineage`.

### 5.4 `P01/I06/S01/C12/T10` Align Final-Lineage Recovery Evidence

**Files/area:** Rust-only test evidence in
`tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`. Do not edit production, helper, parser,
serializer, HTML/XML/report artifacts, manifests, dependencies, or features.

**RED:** After T09 removes its rejected extra edits, default and generator
verification fail only stale report/source digests, helper-observation assertions,
and C08 float, shape, grid-range, and mixed-wrap synthetic XML expectations.
Record the exact failing tests before correction; no generation supplies RED.

**Outcome:** Update those existing Rust-only observations and synthetic inputs to
the already-preserved honest helper/XML lineage and current explicit-input model.
Preserve fixture-name independence, negative controls, and every production and
artifact byte. Do not add a parser layer, substitute oracle, or suppression.

**Acceptance:** Default and generator verification are green; T09's six hashes,
388-row activation, 144-field accounting, and 5,713 artifact bytes are unchanged.
Only the two named Rust test files change.

**Commands:**
```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t09_final_lineage_hashes_match_preserved_run
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_final_activation_union_browser_passes_without_substitutes -- --nocapture
git diff --check
```

**Dependency:** T09 is task-clean after its review-fix span.

**Intended commit:** `test(parity): align final lineage recovery evidence`.

## 6 Cycle Completion

After T08, T07, T09, and T10 are independently task-clean, change only `Status` to
`complete` in a separate commit. At that exact head run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_final_activation_union_browser_passes_without_substitutes -- --nocapture
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
git diff --check 8ffb4bc551a24d2283ad54436870ab3f5e66a473..HEAD
test -z "$(git status --short)"
```

The unsafe scan returns no match and final status is clean. Record exact task
ranges and artifact hashes, obtain the mandatory holistic review, publish and
read back authority remote `main`, clean owned temporary resources, and hand the
frozen behavior-correct candidate to C13. Blocker: none.
