# P01-I06-S01-C12 Final Production Correction And Lineage

Status: in_progress

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`8b7d6e54bcc28cadfeffc8b444118095b3ecc93555618763b28b2cc8830b0724`,
commit `b671e68a7b547c22df5d6ef3947221d99801610c`: `FRI-06.4 D-01`,
`D-04`, `D-06`, `D-07`, `D-09`, `D-11`, `D-12`, `D-13`, and `D-16`;
line, metric-fragment, atomic-baseline, physical-placement, comparator,
fixture, and acceptance portions of `FRI-06.5`, `FRI-06.7`, `FRI-06.9`
through `FRI-06.11`, and `FRI-06.14`.

Reviewed implementation sequence:
`plans/P01-layout/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`426fd772d96dbea08a8366e4f41abf693f6dd8ed681a35984c08efd11ead83d1`,
commit `eaf68062c75b98a3e2be752cfce208d19fc6c238`, entry `P01/I06/S01/C12`.

## 1 Outcome

Complete the reviewed T01-T04 production corrections, retain T05's successful
full generation as diagnostic evidence of an invalid final-lineage assumption,
and restore honest finite fixture/comparator inputs. Use the resulting scoped
diagnostic census to realize only the next exact production tasks. T07 and T08
settle all inputs and production behavior. T09 owns the single final full
lineage run and the evidence required for review and publication.

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
partition is a clue, not acceptance. A restored 12-row trace then proved two
causes: atomic percentage has incomplete authored RTL levels `[1, 0, 0]` instead
of `[1, 1, 1]`, while float line/shape decrement logical slots after bidi visual
ordering. It also confirmed intrinsic 20/26 and refreshed 100x80/20x80 RED and
disproved the retained gap controls. Exact XML and trace residue were cleaned;
the tracked count and report hash remain unchanged.

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
- **Production:** T01-T04 remain scoped; T08 owns seven confirmed causes only.
- **Tests/fixtures:** T07's appended fix adds two atomic RTL records to the one
  existing marker; its prior comparator and float-line corrections are unchanged.
- **Generated artifacts:** none in T07/T08; T09 alone owns the final unfiltered
  lineage after all inputs and production settle.
- **Lint:** configured repository gates remain unchanged by this cycle.
- **Dependencies, features, docs, examples, MSRV, and root:** unchanged.
- **Safety:** Surgeist-owned code remains free of unsafe; no new `allow` or
  `expect` attribute is permitted. Classify every scan match and require zero
  executable unsafe constructs.

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

**Files/area:** T07's complete review range retains its historical 61 HTML
sources, three marker sources, helper, `support.rs`, comparator, and focused
generator/browser tests. The appended fix writes only
`tests/layout/browser_parity/html/block/fri06_atomic_inline_percentage_block_size.html`
and focused existing marker/input tests. It changes no other HTML, helper,
parser/adapter, comparator, generator logic, XML/report, production, manifest,
dependency, feature, or generator architecture.

**Outcome:** Retain the original correction, then add exact RTL-scoped nonzero
bidi records at atomic-percentage source indices 0, 1, and 2 and float-line
source index 4.
Direction activates only the authored `whenDirection` record and never derives a
level. A zero-size break touching a shared endpoint is `Same` with its previous
neighbor and `Later` than its next. Remove T05 residue only after readback.

**RED:** The prior reopened RED and committed fix remain evidence. The appended
RED is the two atomic-percentage RTL rows
(`a0620971c825fe0be6909c2331add26e26478c9970e1bd9eb4ff5b8d28321b40`):
the incomplete `[1, 0, 0]` levels preserve visual indices `[0, 1, 2]` and Range
start 180 instead of 73.296875. Three explicit level-one records must produce
`[2, 1, 0]`. The corrected float-line record and endpoint comparator remain
controls; the current float-line Range failure belongs to T08 slot placement.

**Acceptance:** Original T07 acceptance remains. The marker accepts only the two
closed record forms, validates inactive scoped records, consumes each applicable
record once, and rejects invalid direction, fields, target, duplicate, or unused
applicable data. Atomic LTR variants remain all level zero; each atomic RTL
variant carries level one on source indices 0, 1, and 2. Float-line source index
4 and previous/next endpoint controls remain unchanged. Exact inventory is nine
sources and four scoped record entries. No generation, source-name dispatch,
Rust HTML/CSS parsing, full run, or report write occurs in this fix.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t07_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c12_t07_
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

The prior T05 cleanup/readback remains evidence and no diagnostic residue is
present. The appended fix changes only the atomic HTML marker and focused
marker/input tests; broad configured checks run after T08 focused GREEN.

**Generation:** none. The authored-input fix remains pending the later single
full unfiltered generation after T08 production and all inputs settle.

**Dependency:** T05 is task-clean. The revised specification reopens T07; append
the fix span to its ordered task range and re-review the complete range.

**Realized commit:** `323d73afa98ddc73e65fd9c1da223a5fbd85875e`.

**Review-fix commit:** `fix(parity): close scoped bidi and control facts`.

**Appended-fix commit:** `fix(parity): complete atomic RTL input sequence`.

### 5.8 `P01/I06/S01/C12/T08` Close Confirmed Production Causes

**Files/area:** `src/inline.rs`, `src/block.rs`, `src/grid/child.rs`,
`src/grid/subgrid.rs`, `src/grid/tracks.rs`, and focused existing Rust tests.
No fixture, helper, adapter/parser, comparator, generated artifact, manifest,
dependency, public API, or generator-architecture change.

**Outcome/RED:** Correct seven confirmed causes over exactly 228 unique rows;
their sorted union hashes to
`d1fc209badd7aa2c34df042f39cdfc822e609a621b98f7e92e8e5d4e6132191d`.
Cause memberships overlap in the two float-shape RTL rows.

| Cause and deterministic rows | Count | Row hash | Probe |
| --- | ---: | --- | --- |
| Horizontal forced-break fallback envelope; `fri06_inline_unequal_line_alignment`, V4 | 4 | `2bd81de88087d7c2898c04c5736b3e4ec8b8bdb10f28d93255c57a6b22a53d8e` | one parent starts at 40 instead of 42; an incomplete correction publishes 42.4 and accumulates three parents to 127.2 instead of 126 |
| LTR post-exclusion continuation start; `fri06_float_line_exclusion`, LTR | 2 | `a130acbb6f747b8414a8014840712a78aaede0698d44bafc50b08a31e0bf7ac5` | expansion end-aligns at 90 instead of starting at 0 |
| Float-dominated terminal extent; `fri06_float_shape_exclusion`, V4 | 4 | `0b96e7d9a39716b0121017cdbe67345381d72044918c9cef5b31ec216364de18` | an unqualified line phase turns float-owned end 60 into 60.5; removing it globally also turns line-owned 62.5 into 62 |
| Parent baseline shim in intrinsic rows; `subgrid_baseline_auto_rows_{R12}`, V4 | 48 | `2714795b167ecd2062012cf97c4f232d77814057dcadfec6f7558feac9c28570` | nested 20@8 omits the 6px shim against direct 20@14, leaving auto row 20 instead of 26 and roots 324 instead of 411 |
| Refreshed sizing and offsets use the wrong coordinate phase: `inline_column_{C12}` RTL plus `vertical_auto_rows_{R12}` and `vertical_nested_{R12}` V4 | 120 | `05130449e6303bb52d061c71ff49e1265fb55b4e4f2c3b3237b7237516541aaf` | child axes size a parent-grid area before physical offsets are stored as logical; correct both stages and the exact nested/direct values above |
| Role-sensitive inherited gap transform; `nested_block_{R12}`, V4 | 48 | `64fbf855984169eea06abc65f31d1072fb1ced0a0cc6402b01d77b430a93e548` | for gap 10 to 20, major baseline 30 rebases to 25: positive/equal/negative controls are 72/62/82, with first/last edge roles and inverse publication |
| RTL float-band slot progression; `fri06_float_line_exclusion` RTL plus `fri06_float_shape_exclusion` RTL | 4 | `54e29c8b6d8759044b9ca6793a3eedc9fe80c6156162ce86e8eff2c92d27dbee` | after correct bidi visual order, logical starts are 78/50 and 114/0, yielding physical Range starts 102/130 and 66/180 |

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

Float-band placement consumes bidi visual ordering once in logical progression;
`FlowAxes` owns the later RTL physical projection, so slot filling never reverses
progression again.

**Acceptance:** Add the smallest public-front-door regression for each cause
before its correction and preserve nearest passing controls. Do not add content,
direction, fixture-family, or rounding special cases.

The retained test-first diff is the starting state. Before production resumes,
replace the disproved gap expectations with 72/62/82 and add role-sensitive
first/last, reversed, MBP, cycle, and publication-inverse RED controls. Complete
the intrinsic phase-local group controls, cross-writing refresh controls, and
RTL/LTR float-slot controls, then correct only the seven algorithms above. Every
focused T08 test and nearest control must be GREEN before configured checks.
Run no scoped or full generation; the later final task owns the single full run
after the T07 input and T08 production settle.

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c12_t08_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
rg --files -g '*.rs' -0 | xargs -0 rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'
git diff --check
```

**Dependency:** T07's complete ordered range is task-clean under the revised spec.

**Realized commits:** `9ff1b91dabd7d53b32ee0942a7e6962515a80b79`
(`fix(layout): close confirmed FRI-06 production causes`) and review fix
`5f7f72c45090d9c230f7a2957bffadd5904625b4`
(`fix(layout): preserve fractional fallback envelopes`).

### 5.9 `P01/I06/S01/C12/T09` Publish Final Browser Lineage

**Files/area:** the 5,712 manifest-owned generated XML files,
`xml/generation-reports/all.json`, and only narrowly stale final-lineage test
hashes caused by the reviewed T07 fixture input. No HTML, helper, parser,
serializer, comparator, production, API, manifest, dependency, feature, or
generator-architecture change.

**Outcome/RED:** Preserve the existing full-corpus failures as RED, verify the
settled source and helper freezes, then run the existing pinned browser exactly
once through unfiltered `generate-existing`. A failed acceptance check does not
authorize another generation; diagnose without generation and revise this plan
before any replacement run.

**Procedure:**

1. Prove a clean T08 head, no generation process or residue, exactly 5,324
   tracked XML bodies, the pinned Chrome version, and absence of root, filter,
   cache, and version override variables.
2. Update only T07-invalidated freeze constants from direct file hashes; run the
   focused source/input tests before generation. Do not change expected geometry.
3. Execute one full unfiltered existing-pinned generation with Chrome
   `149.0.7827.115` at the reviewed repository-relative path.
4. Validate and commit the generated corpus, report, and narrow freeze updates;
   record report, complete XML, preserved-body, and activation-body hashes.

**Acceptance:** Report `filter` is null; generated is 5,712; unsupported is the
same 16 missing-root variants; expected-fail, quarantine, and generation-failure
buckets are empty. All 388 activation rows pass the browser oracle with the
known-Chrome substitute registry empty. The other 5,324 XML bodies retain
aggregate `852d293828a4c1427f5adac38d0f764131bda298d37109479ec25cac207fa027`.
Marker, provenance, inventory, corpus, Taffy, default, generator-feature,
formatting, Clippy, unsafe-absence, and diff checks pass. The worktree contains
only the reviewed generated lineage and narrow freeze updates before commit and
is clean afterward.

```sh
"target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" --version
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

**Dependency:** T07 and T08 are independently task-clean. This is C12's eighth
and final task. **Intended commit:** `test(parity): publish final FRI-06 lineage`.

## 6 Cycle Completion

After T09 is task-clean, set `cycle_head`, record the final hashes and task
ranges in this plan's status-only `complete` commit, run the complete gates on a
clean worktree, and obtain a fresh holistic `CLEAN` review over
`cycle_base..cycle_head`. Rerun the gates at the reviewed head, fast-forward
local and remote `main`, read back equality, remove temporary resources, and
handoff the published leaf candidate to C13. Blocker: none.
