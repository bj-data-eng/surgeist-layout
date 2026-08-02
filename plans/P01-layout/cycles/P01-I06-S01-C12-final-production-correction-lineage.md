# P01-I06-S01-C12 Final Production Correction And Lineage

Status: in_progress

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`4d383feef44dd58c53ea14ff9fd380effde9b42d5165fa3d8b1911ad56f5ab47`,
commit `82b163d93edafa0f2b1ee42f6f7c273de876829d`: `FRI-06.4 D-01`,
`D-04`, `D-06`, `D-07`, `D-09`, `D-11`, `D-12`, `D-13`, and `D-16`;
line, metric-fragment, atomic-baseline, physical-placement, comparator,
fixture, and acceptance portions of `FRI-06.5`, `FRI-06.7`, `FRI-06.9`
through `FRI-06.11`, and `FRI-06.14`.

Reviewed implementation sequence:
`plans/P01-layout/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`a5f7a219a49c2cf37dd14f73cedab328409415c60c82e9b87b9165e8ec17edf4`,
commit `858e23dfada37ab899130ea3251fc73ae192a6fe`, entry `P01/I06/S01/C12`.

## 1 Outcome

Complete the reviewed T01-T04 production corrections, retain T05's successful
full generation as diagnostic evidence of an invalid final-lineage assumption,
and restore honest finite fixture/comparator inputs. Use the resulting scoped
diagnostic census to realize only the next exact production tasks. After all
inputs and production behavior settle, one later plan amendment authorizes the
single final full lineage and publication gate.

## 2 Boundary And Evidence

C08 is complete and remotely verified at the cycle base. Its task-clean public
fixture-shaped characterization records every current and browser coordinate,
line band, baseline, clear effect, float endpoint, and root size for the ten
rows. The production-browser assertion
`fri06_c08_recovery_characterization_browser_geometry_remains_unaccepted_production`
is exact RED; the companion current-geometry assertion prevents an incomplete
diagnosis from being mistaken for acceptance.

T01 is task-clean at `70d6e048c249b45fe5202b03063a9c76a926501e`; the amended
fixture-input and Chrome-oracle contracts do not change its traversal outcome or
evidence. The direct RTL defect was a redundant physical-direction reversal in
`src/inline.rs`: `reordered_inline_unit_indices` already yields bidi visual
order, after which `visual_order_opposes_logical_progression` reverses that
order again for horizontal RTL. The correction removes that second reversal
without changing source identity, resolved bidi levels, or mixed-level visual
ordering.

The vertical defect is confined to line metric resolution and physical
projection. Combining the forced-break strut with the atomic fallback baseline
expands or anchors the vertical line away from its 24px containing line band.
The browser requires two 24px bands with the atomic's 3px block-axis leading:
first atomic x 75, second atomic x 51, and cleared box x 30. The correction
must preserve D-11 fallback semantics and horizontal, top, and bottom controls.

The float defect is confined to continuation line phase and terminal automatic
block-size derivation. The current boundary-strut topology places the second
line at y 24 and the third at y 40, then the content-type predicate
`terminal_inline_run_has_mixed_text_atomic_line` conditionally adds 0.5px.
The browser line phase is y 0, 21.2, 21.2, and 42, producing an unrounded
terminal extent of 62.5 and rounded root height 63. The correction derives the
extent from resolved line/float geometry and does not retain or replace an ad
hoc content-type or integer-rounding adjustment.

T02 owns its eight exact `FRI-06.11` HTML marker sources, helper, narrow
serialization for the three closed XML forms, strict parser support, and focused
fixture-input tests. Public API, browser policy, base style, dependencies, root
integration, and generator architecture remain unchanged.

T05's preflight and replacement runs exposed zero-fragment whitespace and empty
Range schema defects. Their focused corrections are committed. The next full run
succeeded structurally at 5,712 generated and 16 expected missing-root
unsupported variants, then honestly failed 314 of the 388 activation rows. Its
uncommitted XML/report residue and exact hashes are diagnostic evidence, not
lineage. Independent audits partition the failures into comparator transparency
and line identity, fixture adaptation, truthful bidi input, RTL projection,
float phase, and only then any residual production behavior.

T07 restored the 363 default-block parents, explicit bidi input, transparent
boundaries, and root-local Range identity. Its post-task scoped census generated
97 sources once, then compared all 388 rows: 134 PASS at
`850d819f2989b7600f5d46cef98031f7035eb7f73eedaae730910036b764caac`
and 254 FAIL at
`fa194548ede4759cceca86bce3ea30a2a7ac8a5a43436721ebbf5aef3089e190`;
membership remains
`3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1`.
Root-cause probes assign 4 rows to missing direction-scoped bidi records, 24 to
zero-size control comparison, and 226 to six production causes. Fixture parsing
and generator architecture are not causes. All scoped XML was removed and the
report remains
`4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab`.

Two T08 attempts stopped uncommitted. The first partition was 136 PASS and 252
FAIL at hashes `c2bca266cfd0092c9a08424ca9c9e362d2969e15b1d335199e3bd4ab2eb1a02e`
and `b57bf7ffb102565034cf740ebb5256ae0560a1405d7ca72ef00f3ee805b25396`.
The second ran its scoped diagnostic before focused GREEN, so its 142/246
partition is a clue, not acceptance. It leaves intrinsic 20/26 and refreshed
100x80/20x80 RED, disproves the retained gap controls, and exposes six RTL Range
starts without one proven producer. Exact XML was cleaned after each stop; the
tracked count and report hash remain unchanged.

Selectors below use `V4 = {border_box_ltr, content_box_ltr, border_box_rtl,
content_box_rtl}`, `LTR = {border_box_ltr, content_box_ltr}`, and
`RTL = {border_box_rtl, content_box_rtl}`. `R12` is `inner_row1`, `inner_row2`,
`outer_row3`, and `parent_row1` through `parent_row3`, each with `_first` and
`_last`; `C12` is the analogous `inner_col1`, `inner_col2`, `outer_col3`, and
`parent_col1` through `parent_col3` suffix set.

## 3 Known Chrome Measurement Failures

None. Chrome remains authoritative; an entry requires the full `FRI-06.11` gate.

## 4 Impacts

- **Public API and compatibility:** unchanged.
- **Production:** T01-T04 remain scoped; T08 owns six confirmed causes only.
- **Tests/fixtures:** T07 adds two scoped bidi records and one comparator rule.
- **Generated artifacts:** T05 residue is diagnostic and uncommitted; scoped
  generation remains diagnostic only; one later reviewed full run owns lineage.
- **Lint:** configured repository gates remain unchanged by this cycle.
- **Dependencies, features, docs, examples, MSRV, and root:** unchanged.
- **Safety:** Surgeist-owned code remains free of unsafe; no new `allow` or
  `expect` attribute is permitted.

## 5 Tasks

### 5.1 Completed Task Evidence

T01-T04 are historical, independently reviewed, task-clean contracts. They must
not be re-executed or widened by later work.

| Task | Realized commits | Clean outcome |
| --- | --- | --- |
| `T01` Correct Direct RTL Visual Traversal | `cff9204e`, `70d6e048` | Consume visual order once; composition-independent RTL controls pass |
| `T02` Replace Synthetic Fixture Dispatch With Explicit Inputs | `37a776b3`, `48f7bfac` | Explicit marker/XML lowering remains; rejected Rust HTML pre-parser is absent |
| `T03` Preserve Vertical Line-Band Phase | `7f6a0657`, `150a379a` | Canonical census path and vertical 24px line-band projection pass |
| `T04` Derive Float Continuation And Terminal Extent | `515b712c`, `90c7e861` | Float continuation and terminal geometry pass within T04 scope |

Each task's recorded RED/GREEN, focused/default gates, independent task review,
and commit validation remain authoritative.

### 5.5 `P01/I06/S01/C12/T05` Capture The Diagnostic Lineage Failure

**Files/area:** historical span `90c7e861..0bbfbc04`, whose T05 code commits are
exactly `5ba51d3f`, `78876ebe`, and `0bbfbc04`; their bounded
generator/parser/helper/comparator implementation and focused tests; the stale
serializer-freeze expectation corrected by sole review-fix commit
`40ccaeb2fbf012b017a58615a6f0f856e6918672`; ignored `.DS_Store` cleanup
in Surgeist-owned repository paths; and uncommitted diagnostic residue. The
review fix adds no production, generator logic, fixture, generated-artifact, or
generation change.

**Outcome:** Preserve the input freeze, zero-width whitespace, and empty Range
schema corrections. Record the structurally successful full run as diagnostic,
not final, because 314/388 activation rows fail.

**RED:** The preflight failed on two zero-fragment whitespace sources; the first
replacement failed on empty `rangeInks`; the structurally successful replacement
then failed the exact activation union at 314 rows. These historical failures and
their intervening focused GREEN commits are the task's TDD evidence.

**Acceptance:** Report has `filter:null`, 5,712 generated, 16 missing-root
unsupported, zero expected-fail/quarantine/generation failures, report SHA
`91b37caf03dcb418def6ecf45ebccc50705ab323c153a9b7f2cba7e9522f0028`,
XML aggregate
`19eba32b808f51e08fafd106878b3f93dee9e8e0bf3cef8580851fb0e7be5cc3`,
and activation aggregate
`3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1`.
The serializer-freeze expectation equals the post-empty-Range serializer hash
`5b03bacde641266c548871ab6c0d11d413b00e0b4199fff6c93ab732b7922716`.
Focused GREEN, corpus/Taffy, and format checks pass; the exact activation and full
verification failures are retained. No generated residue is committed.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08r_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

The layout command retains one aggregate RED; `verify` retains six and
`verify-generator` three. Every other result matches the recorded T05 evidence.
The sole review-fix span
`92bab66c8914cd1a8ac91edd71e1fea71a2a041d..40ccaeb2fbf012b017a58615a6f0f856e6918672`
updates only the stale serializer-freeze expectation. It is appended to T05's
ordered review range; no further T05 implementation commit is authorized.
Removing every ignored `.DS_Store` in Surgeist-owned repository paths is
workspace cleanup and creates no repository artifact; `.git`, `tmp`, and
`target` metadata and external source/cache trees are excluded. Neither action
authorizes generation or changes the diagnostic hashes.

**Dependency:** T01-T04 are task-clean.

**Realized review-fix commit:** `40ccaeb2fbf012b017a58615a6f0f856e6918672`
(`test(parity): refresh T05 serializer freeze`).

### 5.7 `P01/I06/S01/C12/T07` Restore Honest Fixture And Comparator Inputs

**Files/area:** the exact 61 HTML sources/363 direct BR-parent divs in the
reviewed specification; the three reviewed bidi-marker sources; browser helper;
`support.rs`; narrow serializer/accounting tests in the generator; focused
browser-parity tests; T05-owned uncommitted provenance edits in `generator.rs`,
all 5,712 generated XML paths under `tests/layout/browser_parity/xml/`, and
`xml/generation-reports/all.json`. No production, base style, manifest,
dependency, feature, or generator architecture.

**Outcome:** Retain the original correction, then add exact RTL-scoped nonzero
bidi records at atomic-percentage source index 0 and float-line source index 4.
Direction activates only the authored `whenDirection` record and never derives a
level. A zero-size break touching a shared endpoint is `Same` with its previous
neighbor and `Later` than its next. Remove T05 residue only after readback.

**RED:** Original RED remains. The reopened RED is the two atomic-percentage RTL
rows (`a0620971c825fe0be6909c2331add26e26478c9970e1bd9eb4ff5b8d28321b40`),
two float-line RTL rows
(`09256c6d96b712fe0094eee0b302954f17090760e8a72634bcc886425e3703ae`),
and 24 `subgrid_baseline_inline_column_{C12}` LTR rows
(`6600fa577693e69a132fff45bb5b784fe48e68e118dc92e8ad43522264f02ce7`).
Their sorted 28-row union hashes to
`fcfc328900a2eb44f683bd03c3009b576226f61575b3d054a8a5bd541df150be`.
Level-one probes produce Range starts 73.296875 and 130 instead of level-zero
180 and 102. A control at 25 touching previous `[5,25]` and next `[25,45]`
currently reports both `Same`; the next relation must be `Later`.

**Acceptance:** Original T07 acceptance remains. The marker accepts only the two
closed record forms, validates inactive scoped records, consumes each applicable
record once, and rejects invalid direction, fields, target, duplicate, or unused
applicable data. LTR variants remain level zero; the four RTL rows carry level
one. Previous/next endpoint controls pass without relaxing nonzero overlap or
tolerance. Exact inventory is nine sources. No generation, source-name dispatch,
Rust HTML/CSS parsing, full run, or report write occurs in this fix.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t07_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c12_t07_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

Before edits, read back the recorded T05 hashes, then restore the 5,324 modified
and 388 untracked XML files and report to `0bbfbc04`. Restore only the
uncommitted provenance hunk in `generator.rs` to the committed T05 review-fix
HEAD, retaining its corrected serializer-freeze expectation. Prove no diagnostic
residue remains before T07's authored changes. The two full verification commands
may fail only at the exact final activation aggregates; their corrected census is
T07 diagnostic output.

**Diagnostic:** After the committed task is clean, scoped generation may be used
only to census the changed 388-row activation union. Each run must follow a real
input/adapter iteration and must not be repeated over unchanged files. Its
output is diagnostic, is cleaned after hashes and categories are recorded, and
is never verification or lineage evidence.

**Dependency:** T05 is task-clean. The revised specification reopens T07; append
the fix span to its ordered task range and re-review the complete range.

**Realized commit:** `323d73afa98ddc73e65fd9c1da223a5fbd85875e`.

**Review-fix commit:** `fix(parity): close scoped bidi and control facts`.

### 5.8 `P01/I06/S01/C12/T08` Close Confirmed Production Causes

**Files/area:** `src/inline.rs`, `src/block.rs`, `src/grid/child.rs`,
`src/grid/subgrid.rs`, `src/grid/tracks.rs`, and focused existing Rust tests.
No fixture, helper, adapter/parser, comparator, generated artifact, manifest,
dependency, public API, or generator-architecture change.

**Outcome/RED:** Correct these six confirmed causes over exactly 226 rows; their
sorted union hashes to
`56234cee8300676e31882ac06136ba22b98fc2c2aa151c6380cf0f1e21045291`.

| Cause and deterministic rows | Count | Row hash | Probe |
| --- | ---: | --- | --- |
| Horizontal forced-break fallback envelope; `fri06_inline_unequal_line_alignment`, V4 | 4 | `2bd81de88087d7c2898c04c5736b3e4ec8b8bdb10f28d93255c57a6b22a53d8e` | one parent starts at 40 instead of 42; an incomplete correction publishes 42.4 and accumulates three parents to 127.2 instead of 126 |
| LTR post-exclusion continuation start; `fri06_float_line_exclusion`, LTR | 2 | `a130acbb6f747b8414a8014840712a78aaede0698d44bafc50b08a31e0bf7ac5` | expansion end-aligns at 90 instead of starting at 0 |
| Float-dominated terminal extent; `fri06_float_shape_exclusion`, V4 | 4 | `0b96e7d9a39716b0121017cdbe67345381d72044918c9cef5b31ec216364de18` | an unqualified line phase turns float-owned end 60 into 60.5; removing it globally also turns line-owned 62.5 into 62 |
| Parent baseline shim in intrinsic rows; `subgrid_baseline_auto_rows_{R12}`, V4 | 48 | `2714795b167ecd2062012cf97c4f232d77814057dcadfec6f7558feac9c28570` | nested 20@8 omits the 6px shim against direct 20@14, leaving auto row 20 instead of 26 and roots 324 instead of 411 |
| Refreshed sizing and offsets use the wrong coordinate phase: `inline_column_{C12}` RTL plus `vertical_auto_rows_{R12}` and `vertical_nested_{R12}` V4 | 120 | `05130449e6303bb52d061c71ff49e1265fb55b4e4f2c3b3237b7237516541aaf` | child axes size a parent-grid area before physical offsets are stored as logical; correct both stages and the exact nested/direct values above |
| Role-sensitive inherited gap transform; `nested_block_{R12}`, V4 | 48 | `64fbf855984169eea06abc65f31d1072fb1ced0a0cc6402b01d77b430a93e548` | for gap 10 to 20, major baseline 30 rebases to 25: positive/equal/negative controls are 72/62/82, with first/last edge roles and inverse publication |

The terminal extent is exactly the maximum of the resolved terminal line-envelope
end and the owned float margin-box end; any fractional phase stays inside its
line endpoint and never decorates the float endpoint. Refreshed grid-area sizing,
margins, self-alignment roles, available size, and known dimensions use the
containing grid's axes. Child axes govern only child-internal layout and baseline
interpretation; convert the resulting physical alignment offsets back to
container-logical values before storing `PendingGridItem` axes for the single
final physical projection.

Every cycle-eligible introspected subgrid leaf must enter the same phase-local
`RowIntrinsicContribution` baseline group as direct items before intrinsic row
sizing. Gap rebasing applies the signed half-gutter difference by mapped edge and
major/first versus minor/last role; reversed mappings change the local edge, and
publication applies the exact inverse.

**Acceptance:** Add the smallest public-front-door regression for each cause
before its correction and preserve nearest passing controls. Do not add content,
direction, fixture-family, or rounding special cases.

Before any further correction or generation, a fresh diagnostic worker may add
one temporary `#[cfg(test)]` observer in `src/inline.rs` and one focused test in
`src/root_tests.rs` for the atomic-percentage, float-line, and float-shape RTL
sources. Trace `border_box_rtl` and `content_box_rtl` plus their exact
`border_box_ltr` and `content_box_ltr` controls: 12 rows total. Capture atomic
size/fallback, line bands, cursor/float
transition/override/used extent, source/bidi/advance/visual/logical starts, and
published logical and final root-relative Range starts. Run only that exact
locked/offline test, remove the probe, and restore the complete implementation
diff byte-for-byte to
`613ef2e3f515f21dc367577d1e3f5bbf6cb425536ac3b1f21dcd75492feb19ae`.
Then stop for a reviewed T08 amendment naming the confirmed producer. No scoped
generation or production correction is authorized by this diagnostic substage.

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c12_t08_rtl_range_start_phase_trace -- --nocapture
```

**Dependency:** T07's complete ordered range is task-clean under the revised spec.

**Diagnostic commit:** none; restore the exact pre-probe implementation diff.

## 6 Current Revision Completion

This revision exits only when:

1. T05 has a clean independent task review, its recorded hashes are read back,
   and all T05 XML/report/provenance residue is removed before T07 edits;
2. T07's revised ordered range is committed and independently task-clean;
3. the exact 12-row RTL/LTR trace identifies the first incorrect producer or
   returns an evidence-backed insufficiency;
4. all observer/test residue is removed and the complete implementation diff
   again hashes to `613ef2e3f515f21dc367577d1e3f5bbf6cb425536ac3b1f21dcd75492feb19ae`;
5. after this reviewed plan is committed alone, status retains only the seven
   owned T08 implementation files, with zero XML or process residue; and
6. T08 is amended and independently reviewed before any correction resumes.

The final evidence commands for this revision are:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c12_t08_rtl_range_start_phase_trace -- --nocapture
git diff -- src/inline.rs src/block.rs src/grid/child.rs src/grid/subgrid.rs src/root_tests.rs src/block_tests.rs src/grid_tests.rs | shasum -a 256
git diff --check
git status --short
```

The handoff is the exact trace, confirmed producer or insufficiency, restored
diff/status evidence, and no commit. It authorizes no production correction,
generation, task review, C12 completion, publication, or C13 planning.

## 7 Successive Planning Gate

After the trace, amend and independently review T08 with only its confirmed
evidence. Once all 388 rows pass, a later reviewed amendment adds the eighth and
final C12 task for one full unfiltered existing-pinned generation, complete
verification/reviews, publication, remote readback, and C13 handoff.

Blocker: none.
