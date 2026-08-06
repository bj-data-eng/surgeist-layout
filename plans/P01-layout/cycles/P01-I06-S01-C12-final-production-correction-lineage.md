# P01-I06-S01-C12 Final Production Correction And Lineage

Status: in_progress

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I06-inline-formatting-floats-bfcs.md`,
normalized SHA-256
`c7b02e4ba50eef68a33f470f163d28e37ecc0cecc5d5d9ea8b7a30ca644ceaa6`,
commit `eb14ab2bcbb680bfda558050c35a6e8d8d8bcf37`: `FRI-06.4 D-01`,
`D-04`, `D-06`, `D-07`, `D-09`, `D-11` through `D-13`, `D-16`, `D-17`;
the comparator, fixture, module, evidence, and acceptance contracts in
`FRI-06.7`, `FRI-06.9` through `FRI-06.11`, and `FRI-06.14`.

Reviewed implementation sequence:
`plans/P01-layout/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`,
normalized SHA-256
`64ccfb57b99d2a481fa00efc1053584549eae55cf04f4aa0d3cb84599f7bf8f6`,
commit `f70b6a7e81dc12a13735e88200fad8d9cfcd4c35`, entry `P01/I06/S01/C12`.

## 1 Outcome

Retain T01-T05 and the validated portions of T07-T09. Reopen only T07, T08,
and T09 to replace the synthetic BR metric and directional comparator, implement
the typed axis-parametric subgrid baseline model, and publish one replacement
browser lineage after all inputs and production behavior settle.

## 2 Boundary And Current Evidence

T01-T05 remain independently task-clean. T07's three reviewed implementation
commits are `323d73afa98ddc73e65fd9c1da223a5fbd85875e`,
`d42a667494055e3ed4bba4b8502220a214b97ef4`, and
`89adbbc29ba3b2350c1fb64876a8a69520af8e07`.
T08's reviewed implementation commits are
`9ff1b91dabd7d53b32ee0942a7e6962515a80b79` and
`5f7f72c45090d9c230f7a2957bffadd5904625b4`. T09's first implementation
commit is `0a355604d0862a8f07811d323acfdece912921cd`; it contains the complete
5,712-file diagnostic lineage and no generator-architecture change.

The committed full run executed exactly once after its then-reviewed inputs:

- report `filter` is null, generated is 5,712, unsupported is the 16 exact
  missing-root variants, and all failure/status buckets are empty;
- 144 of the 388 activation rows pass and 244 fail;
- all 5,324 nonactivation XML bodies retain aggregate
  `852d293828a4c1427f5adac38d0f764131bda298d37109479ec25cac207fa027`;
- report SHA is
  `f46d8d8b50c722037127fdca79679649bd5cfd6db16fb24c0d69a7e5a082147a`;
- complete XML aggregate is
  `20c598e5483b53da5c015b1f2e393d6d4dacd3e9eb8efea977b4bfbccfde1908`;
- activation body aggregate is
  `8b09804ce30d1f3d2ec3e6f03eb959d6290fcf2d51329878cb6b79cef94018f6`.

The final oracle validates all eight float-line/shape rows. It disproves the
T08 forced-break rounding and synthetic gap expectations and partitions the 244
failures into four unequal-line rows plus five 48-row subgrid families:

| Family | Browser | Current | Confirmed cause |
| --- | --- | --- | --- |
| unequal line | root height 126 | 127 | helper estimates BR baseline 14.8 instead of measuring 15 |
| auto rows | root height 411 | 459 | leaf scalar size and group participation are conflated; parent envelope is missing |
| inline column | LTR x 470, RTL x 527 | 415, 570 | only row groups are populated and consumed |
| nested block | y 62 | 57 after comparator | untagged inherited values are transformed again during refresh |
| vertical auto rows | x 196 | 202 | omitted 18px parent envelope redistributes area 375 instead of 381 |
| vertical nested | x 153 | 168 | nested refresh phase error projected with the correct physical sign |

One-variable probes established the required transitions. Existing inline
boundaries cannot repair the BR input because line metrics take independent
maxima. A BR baseline of 15 alone produces exact 42px children and root 126.
Closed interval comparison makes either shared endpoint `Same`; a five-pixel gap
remains `Later`. For vertical auto rows, logical-distance conversion plus the
retained parent envelope changes pre-flex `[163,145,145]` to final
`[212,194,194]`, inherited area 381, child width 371, and x 196. Initial nested
placement is already y 62/x 153 before inherited refresh. The containing-grid
`FlowAxes` refresh conversion is confirmed correct.

Post-T07 read-only diagnostics expose one interim lineage constraint. All 48
inline-column and 48 vertical-nested checked-in rows stop before geometry on the
same stale browser-control fact: XML records a touching next interval as `Later`,
while T07's reviewed closed comparator correctly returns `Same`. T08 cannot edit
that T09-owned lineage, and grid production cannot repair it. These 96 rows use
public `compute_layout` geometry as T08 evidence and re-enter parity acceptance
only after T09's single replacement run.

Correction-attempt accounting: the committed T08 implementation and review fix
are the third failed correction at this boundary. The reviewed specification and
sequence therefore replace the old premise with `D-17`; no fourth correction may
reuse the disproved scalar, row-only, or untagged-phase model.

## 3 Known Chrome Measurement Failures

None. Chrome remains authoritative; no synthetic substitute or expected-fail
entry is authorized.

## 4 Impacts

- **Public API and compatibility:** unchanged; all new grid carriers are private.
- **Production:** T08 may change only `src/grid/{tracks,subgrid,child}.rs` and
  focused tests for `D-17`; validated float production remains unchanged.
- **Fixture/helper/comparator:** T07 changes the existing JavaScript helper,
  comparator, and focused tests. It adds no marker, schema, parser, HTML/CSS
  parser, fixture family, or generated output.
- **Generated artifacts:** T09 alone replaces the 5,712 XML files and `all.json`
  after T07/T08 settle. Its generator-Rust edits are test constants only.
- **Dependencies, features, docs, examples, MSRV, root:** unchanged.
- **Safety:** no `unsafe`, new lint suppression, dependency, or architecture
  expansion is permitted.

## 5 Tasks

### 5.1 Completed Task Evidence

| Task | Realized commits | Clean outcome |
| --- | --- | --- |
| `T01` Direct RTL Traversal | `cff9204e`, `70d6e048` | Consume visual order once |
| `T02` Explicit Fixture Inputs | `37a776b3`, `48f7bfac` | Closed markers/XML; no Rust HTML parser |
| `T03` Vertical Line Phase | `7f6a0657`, `150a379a` | Vertical 24px bands through public layout |
| `T04` Float Continuation | `515b712c`, `90c7e861` | Resolved continuation and terminal extent |
| `T05` Diagnostic Lineage | `5ba51d3f`, `78876ebe`, `0bbfbc04`, `40ccaeb2` | Honest structural generation failure retained |

Their recorded RED/GREEN, exact ranges, task reviews, and gates remain
authoritative and are not re-executed or widened.

### 5.7 `P01/I06/S01/C12/T07` Measure BR Inputs And Close Comparator Intervals

**Files/area:**
`tests/layout/browser_parity/scripts/gentest/test_helper.js`,
`tests/bin/surgeist-layout-generate/generator.rs`,
`tests/layout/browser_parity/support.rs`, and focused existing tests. No HTML,
XML/report, parser schema, serializer, production, manifest, or generator logic.

**Outcome:** Replace `estimateInlineBaselinePx` only for inline BR lowering with
an isolated browser-laid-out zero-size line-over/baseline marker pair using the
same computed font, line height, writing mode, and direction. Convert the marker
positions to a logical block distance, clamp it to the finite line height, remove
the probe immediately, and preserve the existing helper JSON/XML fields. Keep
zero line height exact. Remove `block_relation`'s directional zero-point branch;
the existing closed tolerance overlap owns `Same`, then separated centers and
block progression own `Earlier`/`Later`.

**RED:** Add
`fri06_c12_t07_br_inline_metrics_use_browser_measured_baseline` before changing
the helper; its 16px/20px stub measurement is 15 while the current result is
14.8. Change the forward and reverse shared-next endpoint assertions to `Same`;
the current comparator returns `Later`. Preserve 5px-gap and 0.1-tolerance
controls. Reconstruct RED at T07's current ordered head without generation.

**Acceptance:** The helper test proves 15/20, zero-height, vertical/sideways
logical distance, probe removal, and invalid/nonfinite measurement rejection.
No test name, source identity, expected geometry, glyph ink, or font ratio enters
the helper. Both touching neighbors are `Same`; the unequal-line gap is `Later`.
All historical T07 marker/bidi/parser-independence tests remain green.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t07_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c12_t07_
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

**Dependency:** T05 task-clean and the reviewed spec/sequence/plan revision.
Append the new implementation span to T07's three historical spans and obtain a
fresh task review over the complete ordered range.

**Intended commit:** `fix(parity): measure BR baselines and close intervals`.

### 5.8 `P01/I06/S01/C12/T08` Implement Typed Axis-Parametric Grid Baselines

**Files/area:** `src/grid/tracks.rs`, `src/grid/subgrid.rs`,
`src/grid/child.rs`, and `src/grid_tests.rs`. No inline/block float change,
fixture/helper/parser/comparator, generated artifact, public API, dependency,
feature, manifest, or generator architecture.

**Outcome:** Replace the disproved T08 grid model with `D-17`:

1. convert child physical baseline points once to containing-logical distances;
2. separate complete scalar size, baseline-group membership, and parent
   first/last envelope channels;
3. deduplicate a flattened root's scalar size while retaining its envelope on
   the inherited logical-start/end track;
4. populate and consume row or column groups through one `GridAxisKind` path;
5. require private parent-track/child-track carriers with axis, span, mapped
   edge, first/last role, distance, and no `Default`;
6. apply signed half-gutter/MBP once on inheritance and the exact inverse once on
   publication, making refresh convergence idempotent; and
7. preserve containing-grid refreshed sizing and final `FlowAxes` projection.

**RED:** Replace the inaccurate 26-only and 72/62/82 synthetic contracts before
production edits. Add public `compute_layout` regressions for scalar/group/
envelope separation, LTR/RTL inline-column groups, nested positive/equal/
negative/reversed/MBP refresh, vertical auto rows with stretch disabled, and
vertical nested projection. Current observable RED includes 459 versus 411,
empty column groups, post-refresh y 57 versus 62, area 375 versus 381 and x 202
versus 196, and x 168 versus 153. Existing ordinary-grid and refreshed-axis
controls remain green.

**Acceptance:** All focused T08 public-layout tests pass without direction,
source, family, rounding, or content special cases and prove all five D-17
families. The unaffected auto-row, nested-block, and vertical-auto-row checked-in
filters pass all 144 rows. The eight final-oracle float rows remain green and
their production blobs are unchanged. Inline-column and vertical-nested parity
are not interim T08 gates: their 96 rows retain only the exact stale
`Later`-versus-`Same` browser-control failure above until T09 replaces the XML.
The four unequal-line rows likewise retain only their pre-T07 helper metric.
T09's final lineage must make all 240 subgrid rows and all 388 activation rows
pass; no expected-fail or synthetic substitute is authorized.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c12_t08_
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid_baseline_auto_rows cargo test --locked --offline -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid_baseline_nested_block cargo test --locked --offline -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid_baseline_vertical_auto_rows cargo test --locked --offline -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=fri06_float_line_exclusion cargo test --locked --offline -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=fri06_float_shape_exclusion cargo test --locked --offline -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

Broad default and generator test suites remain deferred to T09 because their only
permitted failures are the 100 checked-in pre-T07 helper/comparator rows and six
stale evidence assertions that T09 owns. No generation is run in T08.

**Dependency:** T07's complete ordered range is task-clean. Append the correction
span after T08's two historical spans and obtain a fresh task review over the
complete ordered range.

**Intended commit:** `fix(layout): model subgrid baseline phases`.

### 5.9 `P01/I06/S01/C12/T09` Replace Final Browser Lineage

**Files/area:** `tests/bin/surgeist-layout-generate/generator.rs` test constants,
`tests/layout/browser_parity.rs` evidence constants, all 5,712 manifest-owned
generated XML files, and `xml/generation-reports/all.json`. No helper, HTML,
parser/serializer/comparator, production, API, manifest, dependency, feature, or
generator logic.

**RED:** With the committed T07/T08 head and a clean worktree, reproduce the six
stale evidence failures: helper/source-freeze digests, unsupported projection,
float synthetic expectations (62.5 versus final-oracle 62 and 60.5 versus 60),
and report-helper digest. Do not edit them before generation; test execution
leaves the clean precondition intact.

**Outcome:** With a clean worktree, absent filter/cache/version overrides, no
generator process, and pinned Chrome 149.0.7827.115, run unfiltered
`generate-existing` exactly once. Do not run scoped generation. If acceptance
fails, retain the evidence and return to plan review before any replacement run.
After the run, update only the six stale test/evidence assertions and actual
report/inventory/provenance digest constants, then commit them with the generated
outputs as the one replacement-lineage span.

**Acceptance:** Report remains 5,712/16 with null filter, the same exact missing-
root unsupported set, and empty expected-fail/quarantine/failure buckets. All 388
activation rows pass with no substitute. The 5,324 nonactivation body aggregate
remains exact. Record the new helper, report, XML, activation, preserved-body,
and inventory hashes. All focused and broad gates pass; no process or temporary
resource remains.

**Preflight, RED, and sole generation commands:**
```sh
git status --porcelain
git ls-files 'tests/layout/browser_parity/xml/**/*.xml' | wc -l
env | rg '^SURGEIST_(LAYOUT_BROWSER_PARITY_ROOT|LAYOUT_GENERATE_FILTER|BROWSER_CACHE|BROWSER_VERSION)='
pgrep -f '[s]urgeist-layout-generate'
"target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" --version
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08r_lineage_helper_and_nine_html_inputs_are_byte_frozen
git status --porcelain
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

Both status commands print nothing; the XML count is 5,712; environment and
process probes return exit 1 with no output; Chrome prints exactly
`Google Chrome for Testing 149.0.7827.115`. The two test commands return 101 and
account for exactly the six stale assertions before the second clean-status
proof. No command before `generate-existing` writes the worktree.

After updating and committing only the replacement lineage and its evidence
constants, run this reusable post-generation final-check set. It contains no
generation command:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08r_lineage_helper_and_nine_html_inputs_are_byte_frozen
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout report
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_final_activation_union_browser_passes_without_substitutes
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git ls-files -co --exclude-standard '*.rs'
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' $(git ls-files -co --exclude-standard '*.rs')
git diff --check 8ffb4bc551a24d2283ad54436870ab3f5e66a473..HEAD
git status --short
```

The manifest command records every tracked and nonignored untracked owned Rust
file. The scan consumes exactly that manifest, must return exit 1 with no match,
and any match requires classification and correction before review. Final status
is clean and no generator process or temporary resource remains.

**Dependency:** T07 and T08 are task-clean. Append the replacement span after
T09 commit `0a355604d0862a8f07811d323acfdece912921cd`; independently review the
complete ordered T09 range.

**Intended commit:** `test(parity): replace final FRI-06 lineage`.

## 6 Cycle Completion

After every reopened task is CLEAN, record exact ranges, hashes, and command
evidence in the completion record and candidate handoff, leaving this reviewed
plan unchanged except for its administrative status. Change only `Status` to
`complete` in a separate commit and set the cycle head. Run the post-generation
final-check set at that exact head, then obtain holistic `CLEAN` over
`cycle_base..cycle_head`. Rerun that same non-generating set after the holistic
verdict, push local `main` to its configured remote `main`, fetch/read back exact
equality, remove all temporary resources, and hand the published leaf candidate
to C13. Blocker: none.
